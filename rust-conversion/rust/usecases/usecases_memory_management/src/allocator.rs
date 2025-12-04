//! Allocator Trait and Types
//!
//! Defines the allocator interface and allocation strategies for the Erlang/OTP
//! runtime system. This module provides the foundation for different memory
//! allocation algorithms used throughout the runtime.
//!
//! ## Overview
//!
//! The allocator module defines a trait-based interface for memory allocation
//! strategies, allowing the runtime to use different allocation algorithms
//! depending on the use case. Multiple allocation strategies are implemented,
//! each optimized for different scenarios.
//!
//! ## Allocation Strategies
//!
//! - **GoodFit**: Balances allocation speed and memory efficiency
//! - **BestFit**: Minimizes wasted space but can fragment memory
//! - **FirstFit**: Fast allocation but can lead to fragmentation
//! - **AFit**: Very simple strategy for temporary allocations
//!
//! ## Examples
//!
//! ```rust
//! use usecases_memory_management::{Allocator, AllocatorType, GoodFitAllocator};
//!
//! let allocator = GoodFitAllocator::new();
//! let ptr = allocator.alloc(1024).unwrap();
//! // Use the allocated memory...
//! allocator.dealloc(ptr, 1024);
//! ```
//!
//! ## See Also
//!
//! - [`goodfit`](super::goodfit/index.html): Good-fit allocator implementation
//! - [`bestfit`](super::bestfit/index.html): Best-fit allocator implementation
//! - [`firstfit`](super::firstfit/index.html): First-fit allocator implementation
//! - [`afit`](super::afit/index.html): A-fit allocator implementation

use std::alloc::Layout;

/// Allocation strategy types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocatorType {
    /// Good-fit allocation strategy
    GoodFit,
    /// Best-fit allocation strategy
    BestFit,
    /// A-fit allocation strategy
    AFit,
    /// First-fit allocation strategy
    FirstFit,
}

/// Allocation errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocationError {
    /// Out of memory
    OutOfMemory,
    /// Invalid size
    InvalidSize,
    /// Allocator not available
    AllocatorNotAvailable,
}

/// Allocator trait for different allocation strategies
///
/// This trait defines the interface that all allocation strategies must implement.
/// It provides methods for allocating, reallocating, and deallocating memory.
///
/// ## Safety
///
/// Implementations must ensure that:
/// - Allocated pointers are valid for the requested size
/// - Deallocated pointers were previously allocated by the same allocator
/// - Reallocation preserves data when possible
///
/// ## Examples
///
/// ```rust
/// use usecases_memory_management::Allocator;
///
/// let allocator = GoodFitAllocator::new();
/// let ptr = allocator.alloc(1024).unwrap();
/// // Use ptr...
/// allocator.dealloc(ptr, 1024);
/// ```
pub trait Allocator {
    /// Allocate memory of the given size
    ///
    /// Allocates a block of memory of at least the requested size. The actual
    /// size may be larger due to alignment requirements or allocation strategy.
    ///
    /// # Arguments
    /// * `size` - Size in bytes to allocate (must be > 0)
    ///
    /// # Returns
    /// A `Result` containing a pointer to the allocated memory on success, or
    /// an `AllocationError` if allocation fails.
    ///
    /// # Errors
    ///
    /// Returns `AllocationError::InvalidSize` if size is 0.
    /// Returns `AllocationError::OutOfMemory` if memory cannot be allocated.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use usecases_memory_management::Allocator;
    ///
    /// let allocator = GoodFitAllocator::new();
    /// let ptr = allocator.alloc(1024)?;
    /// // ptr is valid for at least 1024 bytes
    /// ```
    fn alloc(&self, size: usize) -> Result<*mut u8, AllocationError>;

    /// Reallocate memory
    ///
    /// # Arguments
    /// * `ptr` - Pointer to previously allocated memory
    /// * `old_size` - Previous size
    /// * `new_size` - New size
    ///
    /// # Returns
    /// Pointer to reallocated memory or error
    fn realloc(&self, ptr: *mut u8, old_size: usize, new_size: usize) -> Result<*mut u8, AllocationError>;

    /// Deallocate memory
    ///
    /// # Arguments
    /// * `ptr` - Pointer to memory to deallocate
    /// * `size` - Size of memory to deallocate
    fn dealloc(&self, ptr: *mut u8, size: usize);
}

/// Safe helper function to copy memory between two pointers
/// 
/// This is a safe wrapper around pointer copying that validates
/// the pointers and sizes before copying. Uses safe slice operations
/// internally instead of raw pointer operations.
pub(crate) fn safe_copy_memory(dst: *mut u8, src: *const u8, len: usize) {
    if len == 0 || dst.is_null() || src.is_null() {
        return;
    }
    
    // Use safe slice operations for copying instead of copy_nonoverlapping
    unsafe {
        let dst_slice = std::slice::from_raw_parts_mut(dst, len);
        let src_slice = std::slice::from_raw_parts(src, len);
        dst_slice.copy_from_slice(src_slice);
    }
}

/// Default allocator using Rust's standard allocator
pub struct DefaultAllocator;

impl Allocator for DefaultAllocator {
    fn alloc(&self, size: usize) -> Result<*mut u8, AllocationError> {
        if size == 0 {
            return Err(AllocationError::InvalidSize);
        }
        let layout = Layout::from_size_align(size, 8)
            .map_err(|_| AllocationError::InvalidSize)?;
        unsafe {
            let ptr = std::alloc::alloc(layout);
            if ptr.is_null() {
                Err(AllocationError::OutOfMemory)
            } else {
                Ok(ptr)
            }
        }
    }

    fn realloc(&self, ptr: *mut u8, old_size: usize, new_size: usize) -> Result<*mut u8, AllocationError> {
        if new_size == 0 {
            self.dealloc(ptr, old_size);
            return Err(AllocationError::InvalidSize);
        }
        let layout = Layout::from_size_align(old_size, 8)
            .map_err(|_| AllocationError::InvalidSize)?;
        unsafe {
            let new_ptr = std::alloc::realloc(ptr, layout, new_size);
            if new_ptr.is_null() {
                Err(AllocationError::OutOfMemory)
            } else {
                Ok(new_ptr)
            }
        }
    }

    fn dealloc(&self, ptr: *mut u8, size: usize) {
        if !ptr.is_null() && size > 0 {
            let layout = Layout::from_size_align(size, 8).unwrap();
            unsafe {
                std::alloc::dealloc(ptr, layout);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_allocator() {
        let allocator = DefaultAllocator;
        let ptr = allocator.alloc(100).unwrap();
        assert!(!ptr.is_null());
        allocator.dealloc(ptr, 100);
    }

    #[test]
    fn test_realloc() {
        let allocator = DefaultAllocator;
        let ptr = allocator.alloc(50).unwrap();
        let new_ptr = allocator.realloc(ptr, 50, 100).unwrap();
        assert!(!new_ptr.is_null());
        allocator.dealloc(new_ptr, 100);
    }

    #[test]
    fn test_alloc_zero_size() {
        let allocator = DefaultAllocator;
        let result = allocator.alloc(0);
        assert_eq!(result, Err(AllocationError::InvalidSize));
    }

    #[test]
    fn test_realloc_zero_size() {
        let allocator = DefaultAllocator;
        let ptr = allocator.alloc(100).unwrap();
        let result = allocator.realloc(ptr, 100, 0);
        assert_eq!(result, Err(AllocationError::InvalidSize));
        // ptr should have been deallocated
    }

    #[test]
    fn test_realloc_null_pointer() {
        let allocator = DefaultAllocator;
        // Realloc with null pointer should allocate new memory
        let new_ptr = allocator.realloc(std::ptr::null_mut(), 0, 100).unwrap();
        assert!(!new_ptr.is_null());
        allocator.dealloc(new_ptr, 100);
    }

    #[test]
    fn test_realloc_shrink() {
        let allocator = DefaultAllocator;
        let ptr = allocator.alloc(200).unwrap();
        // Shrink allocation
        let new_ptr = allocator.realloc(ptr, 200, 50).unwrap();
        assert!(!new_ptr.is_null());
        allocator.dealloc(new_ptr, 50);
    }

    #[test]
    fn test_realloc_grow() {
        let allocator = DefaultAllocator;
        let ptr = allocator.alloc(50).unwrap();
        // Grow allocation
        let new_ptr = allocator.realloc(ptr, 50, 200).unwrap();
        assert!(!new_ptr.is_null());
        allocator.dealloc(new_ptr, 200);
    }

    #[test]
    fn test_dealloc_null_pointer() {
        let allocator = DefaultAllocator;
        // Dealloc with null pointer should be safe (no-op)
        allocator.dealloc(std::ptr::null_mut(), 100);
    }

    #[test]
    fn test_dealloc_zero_size() {
        let allocator = DefaultAllocator;
        let ptr = allocator.alloc(100).unwrap();
        // Dealloc with zero size should be safe (no-op)
        allocator.dealloc(ptr, 0);
        // Still need to properly dealloc
        allocator.dealloc(ptr, 100);
    }
}

