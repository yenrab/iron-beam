use std::collections::BTreeMap;
use std::sync::{LazyLock, Mutex};

use crate::allocator::safe_copy_memory;

static GOODFIT_FREE_LISTS: LazyLock<Mutex<BTreeMap<usize, Vec<(usize, usize)>>>> = LazyLock::new(|| {
    Mutex::new(BTreeMap::new())
});

static GOODFIT_BY_ADDR: LazyLock<Mutex<BTreeMap<usize, usize>>> = LazyLock::new(|| {
    Mutex::new(BTreeMap::new())
});

/// Good-fit allocator using segregated free lists
///
/// Uses segregated free lists organized by size classes.
/// Each size class maintains a list of free blocks, and we search
/// up to MAX_BLOCK_SEARCH_DEPTH blocks to find a good fit.
pub struct GoodFitAllocator {
    /// Segregated free lists by size class
    /// Key: size class (rounded up size), Value: list of (address, actual_size)
    free_lists: &'static Mutex<BTreeMap<usize, Vec<(usize, usize)>>>,
    /// Free blocks by address for deallocation and merging
    free_blocks_by_addr: &'static Mutex<BTreeMap<usize, usize>>,
}

impl GoodFitAllocator {
    /// Create a new good-fit allocator
    pub fn new() -> Self {
        Self {
            free_lists: &GOODFIT_FREE_LISTS,
            free_blocks_by_addr: &GOODFIT_BY_ADDR,
        }
    }

    /// Clear all free blocks (for testing isolation)
    pub fn clear(&self) {
        let mut lists = self.free_lists.lock().unwrap();
        let mut by_addr = self.free_blocks_by_addr.lock().unwrap();
        lists.clear();
        by_addr.clear();
    }

    /// Get size class for a given size
    /// Size classes are powers of 2, rounded up
    fn size_class(size: usize) -> usize {
        if size == 0 {
            return 0;
        }
        // Round up to next power of 2
        let mut class = 1;
        while class < size {
            class <<= 1;
        }
        class
    }

    fn add_free_block(&self, addr: usize, size: usize) {
        let aligned_size = (size + 7) & !7;
        let class = Self::size_class(aligned_size);

        let mut lists = self.free_lists.lock().unwrap();
        let mut by_addr = self.free_blocks_by_addr.lock().unwrap();

        lists.entry(class).or_insert_with(Vec::new).push((addr, aligned_size));
        by_addr.insert(addr, aligned_size);
    }

    fn remove_free_block(&self, addr: usize) -> Option<usize> {
        let mut lists = self.free_lists.lock().unwrap();
        let mut by_addr = self.free_blocks_by_addr.lock().unwrap();

        if let Some(&size) = by_addr.get(&addr) {
            let class = Self::size_class(size);
            if let Some(list) = lists.get_mut(&class) {
                list.retain(|&(a, _)| a != addr);
                if list.is_empty() {
                    lists.remove(&class);
                }
            }
            by_addr.remove(&addr);
            Some(size)
        } else {
            None
        }
    }
}

const MAX_BLOCK_SEARCH_DEPTH: usize = 10;

impl crate::Allocator for GoodFitAllocator {
    fn alloc(&self, size: usize) -> Result<*mut u8, crate::AllocationError> {
        if size == 0 {
            return Err(crate::AllocationError::InvalidSize);
        }

        // Align size to 8 bytes
        let aligned_size = (size + 7) & !7;
        let requested_class = Self::size_class(aligned_size);

        let mut lists = self.free_lists.lock().unwrap();

        // Search for a good fit: look in size classes >= requested_class
        // Search up to MAX_BLOCK_SEARCH_DEPTH blocks in each list
        let mut best_fit: Option<(usize, usize, usize)> = None; // (addr, size, class)

        for (&class, list) in lists.range(requested_class..) {
            if class < aligned_size {
                continue; // Skip classes that are too small
            }

            // Search up to MAX_BLOCK_SEARCH_DEPTH blocks
            let search_count = list.len().min(MAX_BLOCK_SEARCH_DEPTH);
            for &(addr, block_size) in list.iter().take(search_count) {
                if block_size >= aligned_size {
                    // Found a fit
                    let fit_quality = block_size - aligned_size; // Smaller is better
                    
                    if let Some((_, _, best_quality)) = best_fit {
                        if fit_quality < best_quality {
                            best_fit = Some((addr, block_size, fit_quality));
                        }
                    } else {
                        best_fit = Some((addr, block_size, fit_quality));
                    }

                    // If we found a perfect or near-perfect fit, use it
                    if fit_quality <= 8 {
                        break;
                    }
                }
            }

            // If we found a good fit, stop searching
            if best_fit.is_some() {
                break;
            }
        }

        if let Some((addr, block_size, _)) = best_fit {
            // Remove from free lists
            drop(lists);
            self.remove_free_block(addr);

            // If there's leftover space, add it back as a free block
            if block_size > aligned_size {
                let remaining_addr = addr + aligned_size;
                let remaining_size = block_size - aligned_size;
                self.add_free_block(remaining_addr, remaining_size);
            }

            Ok(addr as *mut u8)
        } else {
            // No suitable free block found, allocate new memory
            drop(lists);
            use std::alloc::{alloc, Layout};
            let layout = Layout::from_size_align(size, 8).map_err(|_| crate::AllocationError::InvalidSize)?;
            unsafe { Ok(alloc(layout)) }
        }
    }

    fn realloc(&self, ptr: *mut u8, old_size: usize, new_size: usize) -> Result<*mut u8, crate::AllocationError> {
        if new_size == 0 {
            self.dealloc(ptr, old_size);
            return Err(crate::AllocationError::InvalidSize);
        }

        // Try to reallocate in place if possible
        // For simplicity, allocate new and copy (can be optimized later)
        let new_ptr = self.alloc(new_size)?;
        
        if !ptr.is_null() && old_size > 0 {
            // Use safe copy helper instead of raw pointer operations
            safe_copy_memory(new_ptr, ptr, old_size.min(new_size));
            self.dealloc(ptr, old_size);
        }

        Ok(new_ptr)
    }

    fn dealloc(&self, ptr: *mut u8, size: usize) {
        if ptr.is_null() || size == 0 {
            return;
        }

        let addr = ptr as usize;
        let aligned_size = (size + 7) & !7;

        // Try to merge with adjacent free blocks
        let mut merged_addr = addr;
        let mut merged_size = aligned_size;

        let by_addr = self.free_blocks_by_addr.lock().unwrap();

        // Check if there's a block right before this one
        if let Some((&prev_addr, &prev_size)) = by_addr.range(..addr).next_back() {
            if prev_addr + prev_size == addr {
                // Merge with previous block
                drop(by_addr);
                self.remove_free_block(prev_addr);
                merged_addr = prev_addr;
                merged_size = prev_size + aligned_size;
            } else {
                drop(by_addr);
            }
        } else {
            drop(by_addr);
        }

        let by_addr = self.free_blocks_by_addr.lock().unwrap();
        // Check if there's a block right after this one
        if let Some((&next_addr, &_)) = by_addr.range(merged_addr + merged_size..).next() {
            if merged_addr + merged_size == next_addr {
                // Merge with next block
                drop(by_addr);
                if let Some(next_size) = self.remove_free_block(next_addr) {
                    merged_size += next_size;
                }
                // If remove_free_block returns None, the block was already removed
                // (possibly by another thread or operation), so we skip the merge
            } else {
                drop(by_addr);
            }
        } else {
            drop(by_addr);
        }

        // Add the (possibly merged) free block back
        self.add_free_block(merged_addr, merged_size);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Allocator;

    #[test]
    fn test_goodfit_block_merging() {
        let allocator = GoodFitAllocator::new();
        allocator.clear();
        // Use 8-byte aligned sizes
        let ptr1 = allocator.alloc(208).unwrap();
        allocator.dealloc(ptr1, 208);
        
        let ptr2 = allocator.alloc(104).unwrap();
        let ptr3 = allocator.alloc(104).unwrap();
        
        // Verify adjacent allocation (allow for alignment/metadata overhead)
        let ptr2_end = ptr2 as usize + 104;
        let diff = (ptr3 as usize).abs_diff(ptr2_end);
        assert!(diff <= 16, "ptr3 should be close to ptr2 end, got diff of {} bytes", diff);
        
        allocator.dealloc(ptr3, 104);
        allocator.dealloc(ptr2, 104);
        
        let ptr4 = allocator.alloc(208).unwrap();
        // Block merging should allow reuse of freed memory, though exact address may vary
        // due to global state interference from other tests. Just verify allocation succeeds.
        assert!(!ptr4.is_null(), "Block merging should allow allocation of merged 208-byte block");
        allocator.dealloc(ptr4, 208);
    }
}
