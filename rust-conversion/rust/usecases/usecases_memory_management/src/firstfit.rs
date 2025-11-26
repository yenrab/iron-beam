//! First-Fit Allocator
//!
//! Implements first-fit allocation strategy.

use super::allocator::{Allocator, AllocationError};

/// First-fit allocator implementation
pub struct FirstFitAllocator;

impl FirstFitAllocator {
    pub fn new() -> Self {
        Self
    }
}

impl Allocator for FirstFitAllocator {
    fn alloc(&self, size: usize) -> Result<*mut u8, AllocationError> {
        // TODO: Implement first-fit algorithm
        super::allocator::DefaultAllocator.alloc(size)
    }

    fn realloc(&self, ptr: *mut u8, old_size: usize, new_size: usize) -> Result<*mut u8, AllocationError> {
        super::allocator::DefaultAllocator.realloc(ptr, old_size, new_size)
    }

    fn dealloc(&self, ptr: *mut u8, size: usize) {
        super::allocator::DefaultAllocator.dealloc(ptr, size)
    }
}

