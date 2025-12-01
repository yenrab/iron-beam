//! Good-Fit Allocator
//!
//! Implements good-fit allocation strategy.
//! Based on erl_goodfit_alloc.c
//!
//! Good-fit uses segregated free lists with a limited search depth.
//! It tries to find the best fit but settles for a good fit found
//! during a limited search (default max depth: 3 blocks per list).
//! This provides a good balance between allocation speed and memory efficiency.

use super::allocator::{safe_copy_memory, Allocator, AllocationError};
use std::collections::BTreeMap;
use std::sync::{Mutex, LazyLock};

/// Maximum block search depth per size class
const MAX_BLOCK_SEARCH_DEPTH: usize = 3;

/// Good-fit allocator implementation
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

static GOODFIT_FREE_LISTS: LazyLock<Mutex<BTreeMap<usize, Vec<(usize, usize)>>>> = LazyLock::new(|| {
    Mutex::new(BTreeMap::new())
});

static GOODFIT_BY_ADDR: LazyLock<Mutex<BTreeMap<usize, usize>>> = LazyLock::new(|| {
    Mutex::new(BTreeMap::new())
});

impl GoodFitAllocator {
    /// Create a new good-fit allocator
    pub fn new() -> Self {
        Self {
            free_lists: &GOODFIT_FREE_LISTS,
            free_blocks_by_addr: &GOODFIT_BY_ADDR,
        }
    }

    /// Clear all free blocks (for testing isolation)
    #[cfg(test)]
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

    /// Add a free block to the appropriate size class
    fn add_free_block(&self, addr: usize, size: usize) {
        let aligned_size = (size + 7) & !7;
        let class = Self::size_class(aligned_size);

        let mut lists = self.free_lists.lock().unwrap();
        let mut by_addr = self.free_blocks_by_addr.lock().unwrap();

        lists.entry(class).or_insert_with(Vec::new).push((addr, aligned_size));
        by_addr.insert(addr, aligned_size);
    }

    /// Remove a free block
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

impl Allocator for GoodFitAllocator {
    fn alloc(&self, size: usize) -> Result<*mut u8, AllocationError> {
        if size == 0 {
            return Err(AllocationError::InvalidSize);
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
            let default_alloc = super::allocator::DefaultAllocator;
            default_alloc.alloc(size)
        }
    }

    fn realloc(&self, ptr: *mut u8, old_size: usize, new_size: usize) -> Result<*mut u8, AllocationError> {
        if new_size == 0 {
            self.dealloc(ptr, old_size);
            return Err(AllocationError::InvalidSize);
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
                let next_size = self.remove_free_block(next_addr).unwrap();
                merged_size += next_size;
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

    #[test]
    fn test_goodfit_allocator() {
        let allocator = GoodFitAllocator::new();
        allocator.clear(); // Ensure test isolation
        let ptr = allocator.alloc(100).unwrap();
        assert!(!ptr.is_null());
        allocator.dealloc(ptr, 100);
    }

    #[test]
    fn test_goodfit_reuse() {
        let allocator = GoodFitAllocator::new();
        allocator.clear(); // Ensure test isolation
        let ptr1 = allocator.alloc(100).unwrap();
        allocator.dealloc(ptr1, 100);
        
        // Should reuse the freed block
        let ptr2 = allocator.alloc(50).unwrap();
        assert_eq!(ptr1 as usize, ptr2 as usize);
        allocator.dealloc(ptr2, 50);
    }

    #[test]
    fn test_goodfit_finds_good_fit() {
        let allocator = GoodFitAllocator::new();
        allocator.clear(); // Ensure test isolation
        let ptr1 = allocator.alloc(100).unwrap();
        let ptr2 = allocator.alloc(200).unwrap();
        let ptr3 = allocator.alloc(300).unwrap();
        
        // Free all blocks
        allocator.dealloc(ptr1, 100);
        allocator.dealloc(ptr2, 200);
        allocator.dealloc(ptr3, 300);
        
        // Allocate 150 bytes - should find a good fit (200-byte block)
        let ptr4 = allocator.alloc(150).unwrap();
        assert_eq!(ptr2 as usize, ptr4 as usize);
        
        allocator.dealloc(ptr4, 150);
    }
}

