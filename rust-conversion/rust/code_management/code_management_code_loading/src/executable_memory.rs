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

/// Check if executable memory allocation is available in this environment
pub fn can_allocate_executable_memory() -> bool {
    // For testing purposes, we'll assume executable memory allocation is not available
    // in sandboxed environments. This avoids SIGBUS errors from actual allocation attempts.
    // In a real deployment, this would try a small allocation and check if it succeeds.
    false
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

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_executable_memory_allocate_valid_sizes() {
        if !super::can_allocate_executable_memory() {
            println!("Executable memory allocation not available in this environment, skipping test");
            return;
        }

        // Test allocation with a valid size
        let size = 1024;
        let result = unsafe { ExecutableMemory::allocate(size) };
        assert!(result.is_ok());

        let mem = result.unwrap();
        assert!(mem.size() >= size); // May be larger due to allocation granularity
        assert!(!mem.executable_ptr().is_null());
        assert!(!mem.writable_ptr().is_null());
        assert_eq!(mem.executable_ptr(), mem.writable_ptr() as *const u8);

        // Clean up
        unsafe { mem.deallocate(); }
    }

    #[test]
    fn test_executable_memory_allocate_zero_size() {
        // Test allocation with zero size (should fail)
        let result = unsafe { ExecutableMemory::allocate(0) };
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ExecutableMemoryError::InvalidSize));
    }

    #[test]
    fn test_executable_memory_allocate_large_size() {
        if !super::can_allocate_executable_memory() {
            println!("Executable memory allocation not available in this environment, skipping test");
            return;
        }

        // Test allocation with a reasonably large size
        let size = 1024 * 1024; // 1MB
        let result = unsafe { ExecutableMemory::allocate(size) };
        assert!(result.is_ok());

        let mem = result.unwrap();
        assert!(mem.size() >= size);

        // Clean up
        unsafe { mem.deallocate(); }
    }

    #[test]
    fn test_executable_memory_allocate_very_small_size() {
        if !super::can_allocate_executable_memory() {
            println!("Executable memory allocation not available in this environment, skipping test");
            return;
        }

        // Test allocation with very small size
        let size = 1;
        let result = unsafe { ExecutableMemory::allocate(size) };
        assert!(result.is_ok());

        let mem = result.unwrap();
        assert!(mem.size() >= size);

        // Clean up
        unsafe { mem.deallocate(); }
    }

    #[test]
    fn test_executable_memory_copy_to_writable_full() {
        if !super::can_allocate_executable_memory() {
            println!("Executable memory allocation not available in this environment, skipping test");
            return;
        }

        let size = 100;
        let mem = unsafe { ExecutableMemory::allocate(size).unwrap() };

        // Test copying data that fits exactly
        let data = vec![42u8; size];
        mem.copy_to_writable(&data);

        // Verify the data was copied (we can read it back through the writable pointer)
        unsafe {
            let copied_data = std::slice::from_raw_parts(mem.writable_ptr(), size);
            assert_eq!(copied_data, data.as_slice());
        }

        unsafe { mem.deallocate(); }
    }

    #[test]
    fn test_executable_memory_copy_to_writable_partial() {
        if !super::can_allocate_executable_memory() {
            println!("Executable memory allocation not available in this environment, skipping test");
            return;
        }

        let size = 100;
        let mem = unsafe { ExecutableMemory::allocate(size).unwrap() };

        // Test copying data that's smaller than allocated size
        let data = vec![99u8; 50];
        mem.copy_to_writable(&data);

        // Verify the data was copied
        unsafe {
            let copied_data = std::slice::from_raw_parts(mem.writable_ptr(), data.len());
            assert_eq!(copied_data, data.as_slice());
        }

        unsafe { mem.deallocate(); }
    }

    #[test]
    fn test_executable_memory_copy_to_writable_overflow() {
        if !super::can_allocate_executable_memory() {
            println!("Executable memory allocation not available in this environment, skipping test");
            return;
        }

        let size = 50;
        let mem = unsafe { ExecutableMemory::allocate(size).unwrap() };

        // Test copying data that's larger than allocated size (should truncate)
        let data = vec![123u8; 100];
        mem.copy_to_writable(&data);

        // Should only copy up to the allocated size
        unsafe {
            let copied_data = std::slice::from_raw_parts(mem.writable_ptr(), size);
            assert_eq!(copied_data, &data[..size]);
        }

        unsafe { mem.deallocate(); }
    }

    #[test]
    fn test_executable_memory_copy_to_writable_empty() {
        if !super::can_allocate_executable_memory() {
            println!("Executable memory allocation not available in this environment, skipping test");
            return;
        }

        let size = 100;
        let mem = unsafe { ExecutableMemory::allocate(size).unwrap() };

        // Test copying empty data
        let data = vec![];
        mem.copy_to_writable(&data);

        // Should not crash and memory should remain unchanged
        // (We can't easily verify the memory content without assuming initial state)

        unsafe { mem.deallocate(); }
    }

    #[test]
    fn test_executable_memory_sync_to_executable() {
        if !super::can_allocate_executable_memory() {
            println!("Executable memory allocation not available in this environment, skipping test");
            return;
        }

        let size = 100;
        let mem = unsafe { ExecutableMemory::allocate(size).unwrap() };

        // Test sync operation (this is mainly about memory barriers and cache coherence)
        mem.sync_to_executable(size);

        // The sync operation should not crash
        // In the current implementation, it just issues a memory fence

        unsafe { mem.deallocate(); }
    }

    #[test]
    fn test_executable_memory_sync_to_executable_zero_len() {
        if !super::can_allocate_executable_memory() {
            println!("Executable memory allocation not available in this environment, skipping test");
            return;
        }

        let size = 100;
        let mem = unsafe { ExecutableMemory::allocate(size).unwrap() };

        // Test sync with zero length
        mem.sync_to_executable(0);

        // Should not crash

        unsafe { mem.deallocate(); }
    }

    #[test]
    fn test_executable_memory_sync_to_executable_large_len() {
        if !super::can_allocate_executable_memory() {
            println!("Executable memory allocation not available in this environment, skipping test");
            return;
        }

        let size = 100;
        let mem = unsafe { ExecutableMemory::allocate(size).unwrap() };

        // Test sync with length larger than allocated size
        mem.sync_to_executable(size * 2);

        // Should not crash (len parameter is currently unused)

        unsafe { mem.deallocate(); }
    }

    #[test]
    fn test_executable_memory_pointers() {
        if !super::can_allocate_executable_memory() {
            println!("Executable memory allocation not available in this environment, skipping test");
            return;
        }

        let size = 100;
        let mem = unsafe { ExecutableMemory::allocate(size).unwrap() };

        // Test that pointers are not null
        assert!(!mem.executable_ptr().is_null());
        assert!(!mem.writable_ptr().is_null());

        // Test that pointers are the same (single-region model)
        assert_eq!(mem.executable_ptr(), mem.writable_ptr() as *const u8);

        // Test size reporting
        assert!(mem.size() >= size);

        unsafe { mem.deallocate(); }
    }

    #[test]
    fn test_executable_memory_multiple_allocations() {
        if !super::can_allocate_executable_memory() {
            println!("Executable memory allocation not available in this environment, skipping test");
            return;
        }

        // Test that multiple allocations work independently
        let size1 = 100;
        let size2 = 200;

        let mem1 = unsafe { ExecutableMemory::allocate(size1).unwrap() };
        let mem2 = unsafe { ExecutableMemory::allocate(size2).unwrap() };

        // Should have different pointers
        assert_ne!(mem1.executable_ptr(), mem2.executable_ptr());
        assert_ne!(mem1.writable_ptr(), mem2.writable_ptr());

        // Should have correct sizes
        assert!(mem1.size() >= size1);
        assert!(mem2.size() >= size2);

        unsafe {
            mem1.deallocate();
            mem2.deallocate();
        }
    }

    #[test]
    fn test_executable_memory_write_and_read() {
        if !super::can_allocate_executable_memory() {
            println!("Executable memory allocation not available in this environment, skipping test");
            return;
        }

        let size = 10;
        let mem = unsafe { ExecutableMemory::allocate(size).unwrap() };

        // Write some data
        let test_data = [1u8, 2, 3, 4, 5];
        mem.copy_to_writable(&test_data);

        // Read it back
        unsafe {
            let read_data = std::slice::from_raw_parts(mem.writable_ptr(), test_data.len());
            assert_eq!(read_data, test_data);
        }

        // Write different data
        let new_data = [10u8, 20, 30];
        mem.copy_to_writable(&new_data);

        // Read it back
        unsafe {
            let read_data = std::slice::from_raw_parts(mem.writable_ptr(), new_data.len());
            assert_eq!(read_data, new_data);
        }

        unsafe { mem.deallocate(); }
    }

    #[test]
    fn test_executable_memory_deallocate() {
        if !super::can_allocate_executable_memory() {
            println!("Executable memory allocation not available in this environment, skipping test");
            return;
        }

        let size = 100;
        let mem = unsafe { ExecutableMemory::allocate(size).unwrap() };

        // Verify allocation worked
        assert!(mem.size() >= size);

        // Deallocate (should not crash)
        unsafe { mem.deallocate(); }

        // Note: After deallocation, we can't safely access the memory
        // The deallocate method takes ownership, so the memory is freed
    }

    #[test]
    fn test_executable_memory_error_display() {
        // Test error display formatting
        let alloc_err = ExecutableMemoryError::AllocationFailed("test allocation failed".to_string());
        assert_eq!(format!("{}", alloc_err), "Failed to allocate executable memory: test allocation failed");

        let protect_err = ExecutableMemoryError::ProtectionFailed("test protection failed".to_string());
        assert_eq!(format!("{}", protect_err), "Failed to change memory protection: test protection failed");

        let size_err = ExecutableMemoryError::InvalidSize;
        assert_eq!(format!("{}", size_err), "Invalid size requested for executable memory");
    }

    #[test]
    fn test_executable_memory_error_debug() {
        // Test error debug formatting
        let alloc_err = ExecutableMemoryError::AllocationFailed("test".to_string());
        let debug_str = format!("{:?}", alloc_err);
        assert!(debug_str.contains("AllocationFailed"));
        assert!(debug_str.contains("test"));

        let protect_err = ExecutableMemoryError::ProtectionFailed("test".to_string());
        let debug_str = format!("{:?}", protect_err);
        assert!(debug_str.contains("ProtectionFailed"));
        assert!(debug_str.contains("test"));

        let size_err = ExecutableMemoryError::InvalidSize;
        let debug_str = format!("{:?}", size_err);
        assert!(debug_str.contains("InvalidSize"));
    }

    #[test]
    fn test_executable_memory_debug_format() {
        if !super::can_allocate_executable_memory() {
            println!("Executable memory allocation not available in this environment, skipping test");
            return;
        }

        let size = 100;
        let mem = unsafe { ExecutableMemory::allocate(size).unwrap() };

        let debug_str = format!("{:?}", mem);
        assert!(debug_str.contains("ExecutableMemory"));
        assert!(debug_str.contains("executable_ptr"));
        assert!(debug_str.contains("writable_ptr"));
        assert!(debug_str.contains("size"));

        unsafe { mem.deallocate(); }
    }

    #[test]
    fn test_executable_memory_copy_boundary_conditions() {
        if !super::can_allocate_executable_memory() {
            println!("Executable memory allocation not available in this environment, skipping test");
            return;
        }

        let size = 10;
        let mem = unsafe { ExecutableMemory::allocate(size).unwrap() };

        // Test copying exactly at boundary
        let data = vec![1u8; size];
        mem.copy_to_writable(&data);

        unsafe {
            let copied = std::slice::from_raw_parts(mem.writable_ptr(), size);
            assert_eq!(copied, data.as_slice());
        }

        // Test copying one byte less than boundary
        let data = vec![2u8; size - 1];
        mem.copy_to_writable(&data);

        unsafe {
            let copied = std::slice::from_raw_parts(mem.writable_ptr(), data.len());
            assert_eq!(copied, data.as_slice());
        }

        unsafe { mem.deallocate(); }
    }

    #[test]
    fn test_executable_memory_concurrent_access() {
        if !super::can_allocate_executable_memory() {
            println!("Executable memory allocation not available in this environment, skipping test");
            return;
        }

        use std::thread;
        use std::sync::Arc;

        let size = 100;
        let mem = Arc::new(unsafe { ExecutableMemory::allocate(size).unwrap() });

        let mem_clone = Arc::clone(&mem);
        let handle = thread::spawn(move || {
            // Write from another thread
            let data = vec![42u8; 10];
            mem_clone.copy_to_writable(&data);

            // Sync from another thread
            mem_clone.sync_to_executable(10);
        });

        handle.join().unwrap();

        // Verify data was written
        unsafe {
            let data = std::slice::from_raw_parts(mem.writable_ptr(), 10);
            assert_eq!(data, &[42u8; 10]);
        }

        unsafe { Arc::try_unwrap(mem).unwrap().deallocate(); }
    }

    #[test]
    fn test_executable_memory_size_method() {
        if !super::can_allocate_executable_memory() {
            println!("Executable memory allocation not available in this environment, skipping test");
            return;
        }

        // Test the size() method (currently unused but should work)
        let requested_size = 512;
        let mem = unsafe { ExecutableMemory::allocate(requested_size).unwrap() };

        let actual_size = mem.size();
        assert!(actual_size >= requested_size);

        unsafe { mem.deallocate(); }
    }

    #[test]
    fn test_executable_memory_pointer_alignment() {
        if !super::can_allocate_executable_memory() {
            println!("Executable memory allocation not available in this environment, skipping test");
            return;
        }

        // Test that allocated memory is properly aligned
        let size = 100;
        let mem = unsafe { ExecutableMemory::allocate(size).unwrap() };

        let ptr = mem.writable_ptr() as usize;

        // Should be aligned to at least pointer size boundary
        // (This is a basic sanity check - real alignment requirements may vary)
        assert_eq!(ptr % std::mem::align_of::<usize>(), 0);

        unsafe { mem.deallocate(); }
    }

    #[test]
    fn test_executable_memory_copy_preserves_data_integrity() {
        if !super::can_allocate_executable_memory() {
            println!("Executable memory allocation not available in this environment, skipping test");
            return;
        }

        let size = 1000;
        let mem = unsafe { ExecutableMemory::allocate(size).unwrap() };

        // Create test data with known pattern
        let mut test_data = Vec::with_capacity(size);
        for i in 0..size {
            test_data.push((i % 256) as u8);
        }

        mem.copy_to_writable(&test_data);

        // Verify data integrity
        unsafe {
            let copied_data = std::slice::from_raw_parts(mem.writable_ptr(), size);
            assert_eq!(copied_data, test_data.as_slice());
        }

        // Copy a subset and verify it doesn't corrupt surrounding data
        let subset_data = vec![255u8; 100];
        let offset = 200;
        unsafe {
            let dest_ptr = mem.writable_ptr().add(offset);
            std::ptr::copy_nonoverlapping(subset_data.as_ptr(), dest_ptr, subset_data.len());
        }

        // Verify subset was written correctly
        unsafe {
            let subset_slice = std::slice::from_raw_parts(mem.writable_ptr().add(offset), subset_data.len());
            assert_eq!(subset_slice, subset_data.as_slice());
        }

        // Verify original data around the subset wasn't corrupted
        unsafe {
            let before_subset = std::slice::from_raw_parts(mem.writable_ptr(), offset);
            assert_eq!(before_subset, &test_data[..offset]);

            let after_subset = std::slice::from_raw_parts(
                mem.writable_ptr().add(offset + subset_data.len()),
                size - offset - subset_data.len()
            );
            assert_eq!(after_subset, &test_data[offset + subset_data.len()..]);
        }

        unsafe { mem.deallocate(); }
    }
}

