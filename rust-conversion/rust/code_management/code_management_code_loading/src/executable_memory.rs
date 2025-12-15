//! Executable Memory Management
//!
//! Provides allocation and management of executable memory regions for BEAM code.
//! This module handles the dual-memory model where code is written to a writable
//! region and then synced to an executable region.
//!
//! ## Overview
//!
//! The `ExecutableMemory` type manages memory regions that can be both written to
//! and executed. This is essential for loading BEAM code into memory for execution.
//!
//! ## Safety
//!
//! All operations involving executable memory are inherently unsafe and require
//! careful synchronization. The caller must ensure:
//! - No code is executing from a memory region when it's being modified
//! - Memory regions are properly deallocated when no longer needed
//! - Proper synchronization is used when accessing memory regions from multiple threads

use region::{Allocation, Protection};
use std::ptr;

/// Errors that can occur during executable memory operations
#[derive(Debug)]
pub enum ExecutableMemoryError {
    /// Memory allocation failed
    AllocationFailed(String),
    /// Memory protection change failed
    ProtectionFailed(String),
    /// Invalid size requested
    InvalidSize,
}

impl std::fmt::Display for ExecutableMemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutableMemoryError::AllocationFailed(msg) => {
                write!(f, "Failed to allocate executable memory: {}", msg)
            }
            ExecutableMemoryError::ProtectionFailed(msg) => {
                write!(f, "Failed to change memory protection: {}", msg)
            }
            ExecutableMemoryError::InvalidSize => {
                write!(f, "Invalid size requested for executable memory")
            }
        }
    }
}

impl std::error::Error for ExecutableMemoryError {}

/// Executable memory region
///
/// Manages a memory region that can be written to and then executed.
/// Uses a dual-region model: writable region for code generation, executable region for execution.
///
/// ## Memory Model
///
/// In the current implementation, we use a single memory region with read-write-execute
/// permissions. The `writable_ptr` and `executable_ptr` point to the same memory, but
/// conceptually represent the dual-region model used in the original Erlang implementation.
///
/// ## Safety
///
/// This type contains raw pointers and manages executable memory. The caller must ensure:
/// - No code is executing from this memory when it's being modified
/// - The memory is properly deallocated when no longer needed
/// - Proper synchronization is used for multi-threaded access
pub struct ExecutableMemory {
    /// The underlying memory allocation
    allocation: Allocation,
    /// Pointer to the executable region (read-only view)
    executable_ptr: *const u8,
    /// Pointer to the writable region (mutable view)
    writable_ptr: *mut u8,
    /// Size of the allocated region in bytes
    size: usize,
}

impl ExecutableMemory {
    /// Allocate executable memory
    ///
    /// Allocates a memory region with read-write-execute permissions that can be
    /// used for loading and executing BEAM code.
    ///
    /// # Arguments
    /// * `size` - Size of the memory region to allocate in bytes
    ///
    /// # Returns
    /// `Ok(ExecutableMemory)` if allocation succeeds, `Err(ExecutableMemoryError)` otherwise
    ///
    /// # Safety
    /// This function allocates executable memory which is inherently unsafe.
    /// The caller must ensure proper synchronization and that the memory is
    /// properly deallocated when no longer needed.
    pub unsafe fn allocate(size: usize) -> Result<Self, ExecutableMemoryError> {
        if size == 0 {
            return Err(ExecutableMemoryError::InvalidSize);
        }

        // Allocate memory with read-write-execute permissions initially
        let protection = Protection::READ | Protection::WRITE | Protection::EXECUTE;
        let mut allocation = region::alloc(size, protection)
            .map_err(|e| ExecutableMemoryError::AllocationFailed(e.to_string()))?;

        let executable_ptr = allocation.as_ptr() as *const u8;
        let writable_ptr = allocation.as_mut_ptr() as *mut u8;
        let allocated_size = allocation.len();

        Ok(Self {
            allocation,
            executable_ptr,
            writable_ptr,
            size: allocated_size,
        })
    }

    /// Copy data to the writable region
    ///
    /// Copies data from the source slice into the writable memory region.
    /// The copy operation respects the bounds of the allocated region.
    ///
    /// # Arguments
    /// * `src` - Source data to copy into the writable region
    ///
    /// # Safety
    /// The caller must ensure that `src` is valid and that the writable region
    /// has sufficient space for the data. This function will copy at most `min(src.len(), self.size)`
    /// bytes to prevent buffer overflows.
    pub fn copy_to_writable(&self, src: &[u8]) {
        unsafe {
            let dst = self.writable_ptr;
            let copy_len = src.len().min(self.size);
            ptr::copy_nonoverlapping(src.as_ptr(), dst, copy_len);
        }
    }

    /// Sync writable region to executable region
    ///
    /// In the dual-region model, this ensures the executable region has the latest code.
    /// For a single-region model (like we're using), this ensures memory barriers are
    /// respected and instruction cache is synchronized.
    ///
    /// # Arguments
    /// * `len` - Length of the data that was written (for cache flushing)
    ///
    /// # Platform Notes
    /// - On x86-64: Memory barriers are typically sufficient due to hardware cache coherence
    /// - On AArch64: May require explicit instruction cache invalidation (placeholder for now)
    pub fn sync_to_executable(&self, len: usize) {
        // Ensure memory visibility with a memory barrier
        std::sync::atomic::fence(std::sync::atomic::Ordering::SeqCst);
        
        // Flush instruction cache if needed (platform-specific)
        #[cfg(target_arch = "aarch64")]
        {
            // On AArch64, we may need explicit cache flushing
            // This is a placeholder - actual implementation would use platform-specific intrinsics
            // For example: __builtin___clear_cache() or sys_icache_invalidate()
            // For now, we rely on the OS to handle cache coherence
        }
        
        let _ = len; // Suppress unused warning (len may be used for cache flushing in future)
    }

    /// Get the executable pointer
    ///
    /// Returns a read-only pointer to the executable memory region.
    /// This pointer can be used to execute code from the region.
    ///
    /// # Returns
    /// Pointer to the executable memory region
    pub fn executable_ptr(&self) -> *const u8 {
        self.executable_ptr
    }

    /// Get the writable pointer
    ///
    /// Returns a mutable pointer to the writable memory region.
    /// This pointer can be used to write code into the region.
    ///
    /// # Returns
    /// Mutable pointer to the writable memory region
    pub fn writable_ptr(&self) -> *mut u8 {
        self.writable_ptr
    }

    /// Get the size of the allocated region
    ///
    /// # Returns
    /// Size of the allocated region in bytes
    pub fn size(&self) -> usize {
        self.size
    }

    /// Deallocate the memory region
    ///
    /// Frees the allocated executable memory. After calling this function,
    /// the memory region is no longer valid and must not be accessed.
    ///
    /// # Safety
    /// The caller must ensure that:
    /// - No code is executing from this memory region
    /// - No references to this memory exist
    /// - Proper synchronization is used if multiple threads may access this memory
    pub unsafe fn deallocate(self) {
        // The Allocation will be dropped here, which deallocates the memory
        // Explicitly drop to make it clear
        drop(self.allocation);
    }
}

// Safety: ExecutableMemory contains raw pointers but doesn't share them unsafely.
// The pointers are only accessed through safe methods that ensure proper bounds checking.
// The type can be safely sent between threads as long as proper synchronization is used
// when accessing the memory regions.
unsafe impl Send for ExecutableMemory {}
unsafe impl Sync for ExecutableMemory {}

impl std::fmt::Debug for ExecutableMemory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExecutableMemory")
            .field("executable_ptr", &self.executable_ptr)
            .field("writable_ptr", &self.writable_ptr)
            .field("size", &self.size)
            .finish()
    }
}

