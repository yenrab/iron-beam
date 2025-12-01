//! A-Fit Allocator
//!
//! Implements A-fit allocation strategy.
//! Based on erl_afit_alloc.c
//!
//! A-fit (Almost-fit) is a very simple strategy: it only checks the first
//! free block in the free list. If it fits, use it. Otherwise, allocate
//! a new block. This is intended for temporary allocations only.
//!
//! Time complexity: O(1) - constant time

use super::allocator::{safe_copy_memory, Allocator, AllocationError};
use std::collections::VecDeque;
use std::sync::{Mutex, LazyLock};

/// Free block entry
#[derive(Debug, Clone, Copy)]
struct FreeBlock {
    /// Start address of the free block
    addr: usize,
    /// Size of the free block in bytes
    size: usize,
}

/// A-fit allocator implementation
///
/// Maintains a simple queue of free blocks. When allocating, only checks
/// the first block. If it fits, use it. Otherwise, allocate new memory.
pub struct AFitAllocator {
    /// Queue of free blocks (FIFO)
    free_blocks: &'static Mutex<VecDeque<FreeBlock>>,
}

static AFIT_FREE_BLOCKS: LazyLock<Mutex<VecDeque<FreeBlock>>> = LazyLock::new(|| {
    Mutex::new(VecDeque::new())
});

impl AFitAllocator {
    pub fn new() -> Self {
        Self {
            free_blocks: &AFIT_FREE_BLOCKS,
        }
    }

    /// Clear all free blocks (for testing isolation)
    #[cfg(test)]
    pub fn clear(&self) {
        let mut blocks = self.free_blocks.lock().unwrap();
        blocks.clear();
    }
}

impl Allocator for AFitAllocator {
    fn alloc(&self, size: usize) -> Result<*mut u8, AllocationError> {
        if size == 0 {
            return Err(AllocationError::InvalidSize);
        }

        // Align size to 8 bytes
        let aligned_size = (size + 7) & !7;

        let mut blocks = self.free_blocks.lock().unwrap();

        // A-fit: only check the first block
        if let Some(block) = blocks.front() {
            if block.size >= aligned_size {
                // First block fits, use it
                let FreeBlock { addr, size: block_size } = blocks.pop_front().unwrap();

                // If there's leftover space, add it back to the front
                if block_size > aligned_size {
                    let remaining = FreeBlock {
                        addr: addr + aligned_size,
                        size: block_size - aligned_size,
                    };
                    blocks.push_front(remaining);
                }

                return Ok(addr as *mut u8);
            }
        }

        // No suitable free block found, allocate new memory
        drop(blocks);
        let default_alloc = super::allocator::DefaultAllocator;
        default_alloc.alloc(size)
    }

    fn realloc(&self, ptr: *mut u8, old_size: usize, new_size: usize) -> Result<*mut u8, AllocationError> {
        if new_size == 0 {
            self.dealloc(ptr, old_size);
            return Err(AllocationError::InvalidSize);
        }

        // For A-fit, reallocation is simple: allocate new and copy
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

        let mut blocks = self.free_blocks.lock().unwrap();

        // Add freed block to the back of the queue
        // (A-fit doesn't merge blocks, just adds them to the queue)
        blocks.push_back(FreeBlock {
            addr,
            size: aligned_size,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_afit_allocator() {
        let allocator = AFitAllocator::new();
        allocator.clear(); // Ensure test isolation
        let ptr = allocator.alloc(100).unwrap();
        assert!(!ptr.is_null());
        allocator.dealloc(ptr, 100);
    }

    #[test]
    fn test_afit_reuses_first() {
        let allocator = AFitAllocator::new();
        allocator.clear(); // Ensure test isolation
        let ptr1 = allocator.alloc(100).unwrap();
        allocator.dealloc(ptr1, 100);
        
        // Should reuse the first (and only) free block
        // When allocating exactly the same size, should get the same address
        let ptr2 = allocator.alloc(100).unwrap();
        assert_eq!(ptr1 as usize, ptr2 as usize);
        allocator.dealloc(ptr2, 100);
    }

    #[test]
    fn test_afit_allocates_new_when_first_doesnt_fit() {
        let allocator = AFitAllocator::new();
        allocator.clear(); // Ensure test isolation
        let ptr1 = allocator.alloc(50).unwrap();
        allocator.dealloc(ptr1, 50);
        
        // Request larger than first block - should allocate new
        let ptr2 = allocator.alloc(200).unwrap();
        // Should be different address (new allocation)
        assert_ne!(ptr1 as usize, ptr2 as usize);
        
        allocator.dealloc(ptr2, 200);
    }

    #[test]
    fn test_afit_alloc_zero_size() {
        let allocator = AFitAllocator::new();
        allocator.clear();
        let result = allocator.alloc(0);
        assert_eq!(result, Err(AllocationError::InvalidSize));
    }

    #[test]
    fn test_afit_realloc_zero_size() {
        let allocator = AFitAllocator::new();
        allocator.clear();
        let ptr = allocator.alloc(100).unwrap();
        let result = allocator.realloc(ptr, 100, 0);
        assert_eq!(result, Err(AllocationError::InvalidSize));
    }

    #[test]
    fn test_afit_realloc_null_pointer() {
        let allocator = AFitAllocator::new();
        allocator.clear();
        let new_ptr = allocator.realloc(std::ptr::null_mut(), 0, 100).unwrap();
        assert!(!new_ptr.is_null());
        allocator.dealloc(new_ptr, 100);
    }

    #[test]
    fn test_afit_dealloc_null_pointer() {
        let allocator = AFitAllocator::new();
        allocator.clear();
        allocator.dealloc(std::ptr::null_mut(), 100);
    }

    #[test]
    fn test_afit_dealloc_zero_size() {
        let allocator = AFitAllocator::new();
        allocator.clear();
        let ptr = allocator.alloc(100).unwrap();
        allocator.dealloc(ptr, 0);
        allocator.dealloc(ptr, 100);
    }

    #[test]
    fn test_afit_exact_size_allocation() {
        let allocator = AFitAllocator::new();
        allocator.clear();
        // Allocate and free a block
        let ptr1 = allocator.alloc(104).unwrap(); // 104 is 8-byte aligned
        allocator.dealloc(ptr1, 104);
        
        // Allocate exactly the same size - should reuse without splitting
        let ptr2 = allocator.alloc(104).unwrap();
        assert_eq!(ptr1 as usize, ptr2 as usize);
        allocator.dealloc(ptr2, 104);
    }

    #[test]
    fn test_afit_block_splitting() {
        let allocator = AFitAllocator::new();
        allocator.clear();
        // Allocate a larger block
        let ptr1 = allocator.alloc(200).unwrap();
        allocator.dealloc(ptr1, 200);
        
        // Allocate smaller - should split the block
        let ptr2 = allocator.alloc(100).unwrap();
        assert_eq!(ptr1 as usize, ptr2 as usize);
        
        // The remaining part should be available
        allocator.dealloc(ptr2, 100);
    }
}

