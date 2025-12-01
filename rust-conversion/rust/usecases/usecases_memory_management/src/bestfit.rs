//! Best-Fit Allocator
//!
//! Implements best-fit allocation strategy.
//! Based on erl_bestfit_alloc.c
//!
//! Best-fit finds the smallest free block that is large enough to satisfy
//! the allocation request. This minimizes wasted space but can lead to
//! many small fragments.

use super::allocator::{safe_copy_memory, Allocator, AllocationError};
use std::collections::BTreeMap;
use std::sync::{Mutex, LazyLock};

/// Best-fit allocator implementation
///
/// Uses a BTreeMap ordered by size to efficiently find the smallest
/// block that fits. When allocating, finds the block with minimum size
/// that is still large enough.
pub struct BestFitAllocator {
    /// Free blocks ordered by size, then by address
    /// Key: (size, address), Value: (address, size) - for efficient lookup
    free_blocks_by_size: &'static Mutex<BTreeMap<(usize, usize), usize>>,
    /// Free blocks by address for deallocation
    free_blocks_by_addr: &'static Mutex<BTreeMap<usize, usize>>,
}

static BESTFIT_BY_SIZE: LazyLock<Mutex<BTreeMap<(usize, usize), usize>>> = LazyLock::new(|| {
    Mutex::new(BTreeMap::new())
});

static BESTFIT_BY_ADDR: LazyLock<Mutex<BTreeMap<usize, usize>>> = LazyLock::new(|| {
    Mutex::new(BTreeMap::new())
});

impl BestFitAllocator {
    pub fn new() -> Self {
        Self {
            free_blocks_by_size: &BESTFIT_BY_SIZE,
            free_blocks_by_addr: &BESTFIT_BY_ADDR,
        }
    }

    /// Clear all free blocks (for testing isolation)
    #[cfg(test)]
    pub fn clear(&self) {
        let mut by_size = self.free_blocks_by_size.lock().unwrap();
        let mut by_addr = self.free_blocks_by_addr.lock().unwrap();
        by_size.clear();
        by_addr.clear();
    }

    fn add_free_block(&self, addr: usize, size: usize) {
        let mut by_size = self.free_blocks_by_size.lock().unwrap();
        let mut by_addr = self.free_blocks_by_addr.lock().unwrap();
        
        by_size.insert((size, addr), addr);
        by_addr.insert(addr, size);
    }

    fn remove_free_block(&self, addr: usize) -> Option<usize> {
        let mut by_size = self.free_blocks_by_size.lock().unwrap();
        let mut by_addr = self.free_blocks_by_addr.lock().unwrap();
        
        if let Some(size) = by_addr.remove(&addr) {
            by_size.remove(&(size, addr));
            Some(size)
        } else {
            None
        }
    }
}

impl Allocator for BestFitAllocator {
    fn alloc(&self, size: usize) -> Result<*mut u8, AllocationError> {
        if size == 0 {
            return Err(AllocationError::InvalidSize);
        }

        // Align size to 8 bytes
        let aligned_size = (size + 7) & !7;

        let mut by_size = self.free_blocks_by_size.lock().unwrap();

        // Best-fit: find smallest block that fits
        // Search for first block with size >= aligned_size
        let found_block = by_size
            .range((aligned_size, 0)..)
            .next()
            .map(|(&(block_size, addr), _)| (addr, block_size));

        if let Some((addr, block_size)) = found_block {
            // Remove from both maps
            by_size.remove(&(block_size, addr));
            drop(by_size);
            
            let mut by_addr = self.free_blocks_by_addr.lock().unwrap();
            by_addr.remove(&addr);
            drop(by_addr);

            // If there's leftover space, add it back as a free block
            if block_size > aligned_size {
                let remaining_addr = addr + aligned_size;
                let remaining_size = block_size - aligned_size;
                self.add_free_block(remaining_addr, remaining_size);
            }

            Ok(addr as *mut u8)
        } else {
            // No free block found, allocate new memory
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
        let default_alloc = super::allocator::DefaultAllocator;
        let new_ptr = default_alloc.alloc(new_size)?;
        
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
    fn test_bestfit_allocator() {
        let allocator = BestFitAllocator::new();
        allocator.clear(); // Ensure test isolation
        let ptr = allocator.alloc(100).unwrap();
        assert!(!ptr.is_null());
        allocator.dealloc(ptr, 100);
    }

    #[test]
    fn test_bestfit_finds_smallest() {
        let allocator = BestFitAllocator::new();
        allocator.clear(); // Ensure test isolation
        let ptr1 = allocator.alloc(100).unwrap();
        let ptr2 = allocator.alloc(200).unwrap();
        let ptr3 = allocator.alloc(300).unwrap();
        
        // Free all blocks
        allocator.dealloc(ptr1, 100);
        allocator.dealloc(ptr2, 200);
        allocator.dealloc(ptr3, 300);
        
        // Allocate 50 bytes - should use the 100-byte block (best fit)
        let ptr4 = allocator.alloc(50).unwrap();
        // Should use the smallest block that fits (100 bytes)
        assert_eq!(ptr1 as usize, ptr4 as usize);
        
        allocator.dealloc(ptr4, 50);
    }
}

