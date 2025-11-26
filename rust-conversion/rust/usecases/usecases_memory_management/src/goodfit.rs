//! Good-Fit Allocator
//!
//! Implements good-fit allocation strategy.
//! Based on erl_goodfit_alloc.c

use super::allocator::{Allocator, AllocationError};

/// Good-fit allocator implementation
pub struct GoodFitAllocator {
    // TODO: Implement good-fit allocation data structures
}

impl GoodFitAllocator {
    /// Create a new good-fit allocator
    pub fn new() -> Self {
        Self {}
    }
}

impl Allocator for GoodFitAllocator {
    fn alloc(&self, size: usize) -> Result<*mut u8, AllocationError> {
        // TODO: Implement good-fit allocation algorithm
        // For now, delegate to default allocator
        super::allocator::DefaultAllocator.alloc(size)
    }

    fn realloc(&self, ptr: *mut u8, old_size: usize, new_size: usize) -> Result<*mut u8, AllocationError> {
        // TODO: Implement good-fit reallocation
        super::allocator::DefaultAllocator.realloc(ptr, old_size, new_size)
    }

    fn dealloc(&self, ptr: *mut u8, size: usize) {
        // TODO: Implement good-fit deallocation
        super::allocator::DefaultAllocator.dealloc(ptr, size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goodfit_allocator() {
        let allocator = GoodFitAllocator::new();
        let ptr = allocator.alloc(100).unwrap();
        assert!(!ptr.is_null());
        allocator.dealloc(ptr, 100);
    }
}

