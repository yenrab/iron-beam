//! First-Fit Allocator
//!
//! Implements first-fit allocation strategy.
//! Based on erl_ao_firstfit_alloc.c
//!
//! First-fit finds the first free block that is large enough to satisfy
//! the allocation request. This is efficient for allocation but can lead
//! to fragmentation over time.

use super::allocator::{safe_copy_memory, Allocator, AllocationError};
use std::collections::BTreeMap;
use std::sync::{Mutex, LazyLock};

/// Free block entry in address-ordered tree
#[derive(Debug, Clone, Copy)]
struct FreeBlock {
    /// Start address of the free block
    addr: usize,
    /// Size of the free block in bytes
    size: usize,
}

/// First-fit allocator implementation
///
/// Uses a BTreeMap ordered by address to maintain free blocks.
/// When allocating, finds the first block (lowest address) that fits.
pub struct FirstFitAllocator {
    /// Free blocks ordered by address
    /// Key: address, Value: size
    free_blocks: &'static Mutex<BTreeMap<usize, usize>>,
}

static FIRSTFIT_FREE_BLOCKS: LazyLock<Mutex<BTreeMap<usize, usize>>> = LazyLock::new(|| {
    Mutex::new(BTreeMap::new())
});

impl FirstFitAllocator {
    pub fn new() -> Self {
        Self {
            free_blocks: &FIRSTFIT_FREE_BLOCKS,
        }
    }

    /// Clear all free blocks (for testing isolation)
    #[cfg(test)]
    pub fn clear(&self) {
        let mut blocks = self.free_blocks.lock().unwrap();
        blocks.clear();
    }
}

impl Allocator for FirstFitAllocator {
    fn alloc(&self, size: usize) -> Result<*mut u8, AllocationError> {
        if size == 0 {
            return Err(AllocationError::InvalidSize);
        }

        // Align size to 8 bytes
        let aligned_size = (size + 7) & !7;

        let mut blocks = self.free_blocks.lock().unwrap();

        // First-fit: find first block (lowest address) that fits
        let mut found_block: Option<(usize, usize)> = None;
        for (&addr, &block_size) in blocks.iter() {
            if block_size >= aligned_size {
                found_block = Some((addr, block_size));
                break; // Found first fit
            }
        }

        if let Some((addr, block_size)) = found_block {
            // Remove the block from free list
            blocks.remove(&addr);

            // If there's leftover space, add it back as a free block
            if block_size > aligned_size {
                let remaining_addr = addr + aligned_size;
                let remaining_size = block_size - aligned_size;
                blocks.insert(remaining_addr, remaining_size);
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

        let mut blocks = self.free_blocks.lock().unwrap();

        // Try to merge with adjacent free blocks
        let mut merged_addr = addr;
        let mut merged_size = aligned_size;

        // Check if there's a block right before this one
        if let Some((&prev_addr, &prev_size)) = blocks.range(..addr).next_back() {
            if prev_addr + prev_size == addr {
                // Merge with previous block
                blocks.remove(&prev_addr);
                merged_addr = prev_addr;
                merged_size = prev_size + aligned_size;
            }
        }

        // Check if there's a block right after this one
        if let Some((&next_addr, &_)) = blocks.range(addr + aligned_size..).next() {
            if addr + aligned_size == next_addr {
                // Merge with next block
                let next_size = blocks.remove(&next_addr).unwrap();
                merged_size += next_size;
            }
        }

        // Add the (possibly merged) free block back
        blocks.insert(merged_addr, merged_size);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_firstfit_allocator() {
        let allocator = FirstFitAllocator::new();
        allocator.clear(); // Ensure test isolation
        let ptr = allocator.alloc(100).unwrap();
        assert!(!ptr.is_null());
        allocator.dealloc(ptr, 100);
    }

    #[test]
    fn test_firstfit_reuse() {
        let allocator = FirstFitAllocator::new();
        allocator.clear(); // Ensure test isolation
        let ptr1 = allocator.alloc(100).unwrap();
        allocator.dealloc(ptr1, 100);
        
        // Should reuse the freed block
        let ptr2 = allocator.alloc(50).unwrap();
        assert_eq!(ptr1 as usize, ptr2 as usize);
        allocator.dealloc(ptr2, 50);
    }

    #[test]
    fn test_firstfit_fragmentation() {
        let allocator = FirstFitAllocator::new();
        allocator.clear(); // Ensure test isolation
        let ptr1 = allocator.alloc(100).unwrap();
        let ptr2 = allocator.alloc(100).unwrap();
        let ptr3 = allocator.alloc(100).unwrap();
        
        // Free middle block
        allocator.dealloc(ptr2, 100);
        
        // Allocate should use the freed block (first fit)
        let ptr4 = allocator.alloc(50).unwrap();
        assert_eq!(ptr2 as usize, ptr4 as usize);
        
        allocator.dealloc(ptr1, 100);
        allocator.dealloc(ptr3, 100);
        allocator.dealloc(ptr4, 50);
    }
}

