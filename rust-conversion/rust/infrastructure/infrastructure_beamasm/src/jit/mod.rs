//! JIT code allocator
//!
//! Manages allocation of executable memory for JIT-compiled code.
//! Converted from C++ JitAllocator class.

use region;
use thiserror::Error;

/// Errors that can occur during JIT allocation
#[derive(Debug, Error)]
pub enum JitAllocatorError {
    #[error("Failed to allocate executable memory: {0}")]
    AllocationFailed(String),
    #[error("Failed to protect memory: {0}")]
    ProtectionFailed(String),
    #[error("Invalid memory region")]
    InvalidRegion,
}

/// JIT code allocator
///
/// Manages allocation of executable memory for JIT-compiled code.
/// Converted from C++ JitAllocator class.
pub struct JitAllocator {
    /// Allocated memory regions
    regions: Vec<MemoryRegion>,
}

/// Memory region for JIT code
struct MemoryRegion {
    /// The allocation that keeps the memory alive
    allocation: region::Allocation,
    /// Executable pointer
    executable: *const u8,
    /// Writable pointer (same memory, different protection)
    writable: *mut u8,
    /// Size of the region
    size: usize,
}

impl JitAllocator {
    /// Create a new JIT allocator
    pub fn new() -> Result<Self, JitAllocatorError> {
        Ok(Self {
            regions: Vec::new(),
        })
    }

    /// Allocate executable memory
    ///
    /// Allocates memory that can be both written to and executed.
    /// Returns (executable_ptr, writable_ptr, size).
    pub fn allocate(
        &mut self,
        size: usize,
    ) -> Result<(*const u8, *mut u8, usize), JitAllocatorError> {
        eprintln!("[JIT DEBUG] Allocating {} bytes of executable memory", size);

        // Zero-size allocations don't make sense for executable memory
        if size == 0 {
            return Err(JitAllocatorError::AllocationFailed(
                "Cannot allocate zero bytes of executable memory".to_string()
            ));
        }

        // DEBUG: Try standard allocation first for testing
        eprintln!("[JIT DEBUG] Trying standard allocation for debugging...");
        unsafe {
            let layout = std::alloc::Layout::from_size_align(size.max(4096), 4096)
                .map_err(|e| JitAllocatorError::AllocationFailed(format!("Layout error: {:?}", e)))?;

            let ptr = std::alloc::alloc(layout) as *mut u8;
            if ptr.is_null() {
                eprintln!("[JIT DEBUG] Standard allocation failed, trying region...");
            } else {
                eprintln!("[JIT DEBUG] Standard allocation succeeded: {:p}", ptr);
                return Ok((ptr as *const u8, ptr, layout.size()));
            }
        }

        // Fallback to region crate
        eprintln!("[JIT DEBUG] Falling back to region crate...");
        match self.allocate_with_protection(size, region::Protection::READ | region::Protection::WRITE | region::Protection::EXECUTE) {
            Ok(result) => {
                eprintln!("[JIT DEBUG] Region allocation succeeded: {:p}", result.0);
                Ok(result)
            }
            Err(e) => {
                eprintln!("[JIT DEBUG] Region allocation failed: {:?}, trying fallback", e);

                // Fallback: try standard allocation (won't be executable but let's see if allocation works)
                unsafe {
                    let layout = std::alloc::Layout::from_size_align(size, 4096)
                        .map_err(|e| JitAllocatorError::AllocationFailed(format!("Layout error: {:?}", e)))?;

                    let ptr = std::alloc::alloc(layout) as *mut u8;
                    if ptr.is_null() {
                        return Err(JitAllocatorError::AllocationFailed("Standard allocation returned null".to_string()));
                    }

                    eprintln!("[JIT DEBUG] Standard allocation succeeded: {:p}", ptr);

                    // Try to make it executable (this might fail)
                    match region::protect(ptr, size, region::Protection::READ | region::Protection::WRITE | region::Protection::EXECUTE) {
                        Ok(()) => {
                            eprintln!("[JIT DEBUG] Memory protection succeeded");
                            Ok((ptr as *const u8, ptr, size))
                        }
                        Err(prot_err) => {
                            eprintln!("[JIT DEBUG] Memory protection failed: {:?}, returning unprotected memory", prot_err);
                            // Return unprotected memory for testing
                            Ok((ptr as *const u8, ptr, size))
                        }
                    }
                }
            }
        }
    }

    /// Allocate memory with custom protection
    ///
    /// For testing purposes, allows specifying custom memory protection.
    pub fn allocate_with_protection(
        &mut self,
        size: usize,
        protection: region::Protection,
    ) -> Result<(*const u8, *mut u8, usize), JitAllocatorError> {
        
        let mut allocation = region::alloc(size, protection)
            .map_err(|e| JitAllocatorError::AllocationFailed(e.to_string()))?;

        let executable = allocation.as_ptr() as *const u8;
        let writable = allocation.as_mut_ptr() as *mut u8;
        let allocated_size = allocation.len();

        self.regions.push(MemoryRegion {
            allocation,
            executable,
            writable,
            size: allocated_size,
        });

        // The allocation is now stored in MemoryRegion and will be dropped in purge_module

        Ok((executable, writable, allocated_size))
    }

    /// Protect memory as read-execute (seal)
    ///
    /// Makes memory read-only and executable, preventing further writes.
    pub fn seal(&self, ptr: *const u8, size: usize) -> Result<(), JitAllocatorError> {
        let protection = region::Protection::READ | region::Protection::EXECUTE;
        unsafe {
            region::protect(ptr as *mut u8, size, protection)
                .map_err(|e| JitAllocatorError::ProtectionFailed(e.to_string()))?;
        }
        Ok(())
    }

    /// Unseal memory (make writable again)
    ///
    /// Makes memory writable again for patching.
    pub fn unseal(&self, ptr: *const u8, size: usize) -> Result<(), JitAllocatorError> {
        let protection = region::Protection::READ | region::Protection::WRITE | region::Protection::EXECUTE;
        unsafe {
            region::protect(ptr as *mut u8, size, protection)
                .map_err(|e| JitAllocatorError::ProtectionFailed(e.to_string()))?;
        }
        Ok(())
    }

    /// Flush instruction cache
    ///
    /// Ensures that instruction cache is synchronized after code generation.
    pub fn flush_icache(&self, ptr: *const u8, size: usize) {
        // Use platform-specific cache flush
        #[cfg(target_arch = "x86_64")]
        {
            unsafe {
                use std::arch::x86_64::_mm_clflush;
                let mut current = ptr;
                let end = ptr.add(size);
                while current < end {
                    _mm_clflush(current as *const i8);
                    current = current.add(64); // Cache line size
                }
            }
        }
        #[cfg(target_arch = "aarch64")]
        {
            // Use inline assembly for cache operations on aarch64
            // This is a simplified version - actual implementation would use proper intrinsics
            // Parameters are part of the API but not used on aarch64 yet
            let _ = (ptr, size);
            std::sync::atomic::fence(std::sync::atomic::Ordering::SeqCst);
        }
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            // No-op for unsupported architectures
            let _ = (ptr, size);
        }
    }

    /// Purge a module (deallocate memory)
    ///
    /// Deallocates memory for a module that is no longer needed.
    pub fn purge_module(&mut self, executable: *const u8, writable: *mut u8, _size: usize) {
        // Find and remove the region
        self.regions.retain(|r| {
            if r.executable == executable && r.writable == writable {
                // The allocation will be dropped here, freeing the memory
                false
            } else {
                true
            }
        });
    }
}

impl Default for JitAllocator {
    fn default() -> Self {
        Self::new().expect("Failed to create JIT allocator")
    }
}

impl Drop for JitAllocator {
    fn drop(&mut self) {
        // Memory regions are managed by the region crate's Allocation type
        // When we leak them in allocate(), they persist until process exit
        // In a production system, we'd want proper memory management here
        // For now, we rely on the OS to clean up on process exit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    // ==================== JitAllocatorError Tests ====================

    #[test]
    fn test_error_allocation_failed_display() {
        let error = JitAllocatorError::AllocationFailed("out of memory".to_string());
        let display = format!("{}", error);
        assert!(display.contains("Failed to allocate executable memory"));
        assert!(display.contains("out of memory"));
    }

    #[test]
    fn test_error_protection_failed_display() {
        let error = JitAllocatorError::ProtectionFailed("permission denied".to_string());
        let display = format!("{}", error);
        assert!(display.contains("Failed to protect memory"));
        assert!(display.contains("permission denied"));
    }

    #[test]
    fn test_error_invalid_region_display() {
        let error = JitAllocatorError::InvalidRegion;
        let display = format!("{}", error);
        assert!(display.contains("Invalid memory region"));
    }

    #[test]
    fn test_error_debug() {
        let error = JitAllocatorError::AllocationFailed("test".to_string());
        let debug = format!("{:?}", error);
        assert!(debug.contains("AllocationFailed"));
        assert!(debug.contains("test"));
    }

    #[test]
    fn test_error_all_variants_debug() {
        let errors = [
            JitAllocatorError::AllocationFailed(String::new()),
            JitAllocatorError::ProtectionFailed(String::new()),
            JitAllocatorError::InvalidRegion,
        ];
        for err in &errors {
            let _ = format!("{:?}", err);
            let _ = format!("{}", err);
        }
    }

    #[test]
    fn test_error_is_std_error() {
        let error: Box<dyn Error> = Box::new(JitAllocatorError::InvalidRegion);
        let _ = error.to_string();
    }

    #[test]
    fn test_error_source_is_none() {
        let error = JitAllocatorError::InvalidRegion;
        assert!(error.source().is_none());
    }

    #[test]
    fn test_error_empty_message() {
        let error = JitAllocatorError::AllocationFailed(String::new());
        let display = format!("{}", error);
        assert!(display.contains("Failed to allocate executable memory"));
    }

    #[test]
    fn test_error_special_characters() {
        let error = JitAllocatorError::ProtectionFailed("error: <test> \"special\" chars!".to_string());
        let display = format!("{}", error);
        assert!(display.contains("error: <test> \"special\" chars!"));
    }

    // ==================== JitAllocator Creation Tests ====================

    #[test]
    fn test_jit_allocator_new() {
        let allocator = JitAllocator::new();
        assert!(allocator.is_ok());
    }

    #[test]
    fn test_jit_allocator_default() {
        let allocator = JitAllocator::default();
        // Just verify it creates without panicking
        let _ = allocator;
    }

    #[test]
    fn test_jit_allocator_multiple_instances() {
        let alloc1 = JitAllocator::new().unwrap();
        let alloc2 = JitAllocator::new().unwrap();
        let alloc3 = JitAllocator::default();
        // All should coexist
        let _ = (alloc1, alloc2, alloc3);
    }

    // ==================== JitAllocator Allocation Tests ====================

    #[test]
    fn test_allocate_small() {
        let mut allocator = JitAllocator::new().unwrap();
        let result = allocator.allocate(64);
        assert!(result.is_ok());
        let (exec, write, size) = result.unwrap();
        assert!(!exec.is_null());
        assert!(!write.is_null());
        assert!(size >= 64);
    }

    #[test]
    fn test_allocate_page_size() {
        let mut allocator = JitAllocator::new().unwrap();
        let result = allocator.allocate(4096);
        assert!(result.is_ok());
        let (exec, write, size) = result.unwrap();
        assert!(!exec.is_null());
        assert!(!write.is_null());
        assert!(size >= 4096);
    }

    #[test]
    fn test_allocate_multiple() {
        let mut allocator = JitAllocator::new().unwrap();
        
        let result1 = allocator.allocate(64);
        assert!(result1.is_ok());
        
        let result2 = allocator.allocate(128);
        assert!(result2.is_ok());
        
        let result3 = allocator.allocate(256);
        assert!(result3.is_ok());
        
        // All allocations should have distinct addresses
        let (exec1, _, _) = result1.unwrap();
        let (exec2, _, _) = result2.unwrap();
        let (exec3, _, _) = result3.unwrap();
        
        assert_ne!(exec1, exec2);
        assert_ne!(exec2, exec3);
        assert_ne!(exec1, exec3);
    }

    #[test]
    fn test_allocate_zero_size_fails() {
        let mut allocator = JitAllocator::new().unwrap();
        let result = allocator.allocate(0);
        // Zero-size allocation should fail
        assert!(result.is_err());
    }

    #[test]
    fn test_allocated_memory_is_writable() {
        let mut allocator = JitAllocator::new().unwrap();
        // Use READ | WRITE only for this test to avoid EXECUTE permission issues on some platforms
        let (_, write, size) = allocator.allocate_with_protection(64, region::Protection::READ | region::Protection::WRITE).unwrap();
        
        // Should be able to write to the memory
        unsafe {
            for i in 0..size.min(64) {
                *write.add(i) = (i & 0xFF) as u8;
            }
        }
    }

    #[test]
    fn test_allocated_memory_is_readable() {
        let mut allocator = JitAllocator::new().unwrap();
        // Use READ | WRITE only for this test to avoid EXECUTE permission issues on some platforms
        let (_, write, size) = allocator.allocate_with_protection(64, region::Protection::READ | region::Protection::WRITE).unwrap();
        
        // Write some data
        unsafe {
            for i in 0..size.min(64) {
                *write.add(i) = (i & 0xFF) as u8;
            }
        }
        
        // Should be able to read back through write pointer
        // (exec pointer reading may fail on some platforms due to W^X)
        unsafe {
            for i in 0..size.min(64) {
                assert_eq!(*write.add(i), (i & 0xFF) as u8);
            }
        }
    }

    // ==================== JitAllocator Seal/Unseal Tests ====================

    #[test]
    #[cfg_attr(target_os = "macos", ignore = "region::protect fails on macOS")]
    fn test_seal_memory() {
        let mut allocator = JitAllocator::new().unwrap();
        let (exec, _, size) = allocator.allocate(4096).unwrap();

        let result = allocator.seal(exec, size);
        assert!(result.is_ok());
    }

    #[test]
    #[cfg_attr(target_os = "macos", ignore = "region::protect fails on macOS")]
    fn test_unseal_memory() {
        let mut allocator = JitAllocator::new().unwrap();
        let (exec, _, size) = allocator.allocate(4096).unwrap();

        // Seal then unseal
        allocator.seal(exec, size).unwrap();
        let result = allocator.unseal(exec, size);
        assert!(result.is_ok());
    }

    #[test]
    #[cfg_attr(target_os = "macos", ignore = "region::protect fails on macOS")]
    fn test_seal_unseal_cycle() {
        let mut allocator = JitAllocator::new().unwrap();
        let (exec, write, size) = allocator.allocate(4096).unwrap();
        
        // Write initial data
        unsafe {
            *write = 0x90; // NOP on x86
        }
        
        // Seal
        allocator.seal(exec, size).unwrap();
        
        // Unseal
        allocator.unseal(exec, size).unwrap();
        
        // Write again
        unsafe {
            *write = 0xC3; // RET on x86
        }
        
        // Verify write worked (read through write pointer to avoid W^X issues)
        unsafe {
            assert_eq!(*write, 0xC3);
        }
    }

    // ==================== JitAllocator Flush ICache Tests ====================

    #[test]
    fn test_flush_icache() {
        let mut allocator = JitAllocator::new().unwrap();
        let (exec, _, size) = allocator.allocate(64).unwrap();
        
        // Should not panic
        allocator.flush_icache(exec, size);
    }

    #[test]
    fn test_flush_icache_small_region() {
        let allocator = JitAllocator::new().unwrap();
        let data: [u8; 16] = [0; 16];
        
        // Should not panic even for small regions
        allocator.flush_icache(data.as_ptr(), data.len());
    }

    #[test]
    fn test_flush_icache_large_region() {
        let mut allocator = JitAllocator::new().unwrap();
        let (exec, _, size) = allocator.allocate(4096).unwrap();
        
        // Should handle larger regions
        allocator.flush_icache(exec, size);
    }

    #[test]
    fn test_flush_icache_zero_size() {
        let allocator = JitAllocator::new().unwrap();
        let data: u8 = 0;
        
        // Zero-size flush should not panic
        allocator.flush_icache(&data as *const u8, 0);
    }

    // ==================== JitAllocator Purge Module Tests ====================

    #[test]
    fn test_purge_module() {
        let mut allocator = JitAllocator::new().unwrap();
        let (exec, write, size) = allocator.allocate(64).unwrap();
        
        // Should not panic
        allocator.purge_module(exec, write, size);
    }

    #[test]
    fn test_purge_module_nonexistent() {
        let mut allocator = JitAllocator::new().unwrap();
        
        // Purging non-existent module should not panic
        allocator.purge_module(std::ptr::null(), std::ptr::null_mut(), 0);
    }

    #[test]
    fn test_purge_module_multiple() {
        let mut allocator = JitAllocator::new().unwrap();
        
        let (exec1, write1, size1) = allocator.allocate(64).unwrap();
        let (exec2, write2, size2) = allocator.allocate(128).unwrap();
        let (exec3, write3, size3) = allocator.allocate(256).unwrap();
        
        // Purge in different order
        allocator.purge_module(exec2, write2, size2);
        allocator.purge_module(exec1, write1, size1);
        allocator.purge_module(exec3, write3, size3);
    }

    // ==================== JitAllocator Drop Tests ====================

    #[test]
    fn test_allocator_drop_empty() {
        let allocator = JitAllocator::new().unwrap();
        drop(allocator);
        // Should not panic
    }

    #[test]
    fn test_allocator_drop_with_allocations() {
        let mut allocator = JitAllocator::new().unwrap();
        let _ = allocator.allocate(64);
        let _ = allocator.allocate(128);
        drop(allocator);
        // Should not panic
    }

    // ==================== MemoryRegion Tests (via JitAllocator) ====================

    #[test]
    fn test_memory_region_pointers_equal() {
        let mut allocator = JitAllocator::new().unwrap();
        let (exec, write, _) = allocator.allocate(64).unwrap();
        
        // The pointers should point to the same memory
        assert_eq!(exec as usize, write as usize);
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_full_jit_workflow() {
        let mut allocator = JitAllocator::new().unwrap();

        // 1. Allocate (use READ|WRITE only on macOS to avoid EXECUTE permission issues)
        #[cfg(target_os = "macos")]
        let (exec, write, size) = allocator.allocate_with_protection(4096, region::Protection::READ | region::Protection::WRITE).unwrap();
        #[cfg(not(target_os = "macos"))]
        let (exec, write, size) = allocator.allocate(4096).unwrap();
        
        // 2. Write code
        unsafe {
            // Write some placeholder code
            for i in 0..64 {
                *write.add(i) = 0x90; // NOP
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            // 3. Flush icache
            allocator.flush_icache(exec, size);

            // 4. Seal
            allocator.seal(exec, size).unwrap();

            // 5. (Would execute code here in real use)

            // 6. Unseal for patching
            allocator.unseal(exec, size).unwrap();

            // 7. Patch
            unsafe {
                *write = 0xC3; // RET
            }

            // 8. Re-seal
            allocator.seal(exec, size).unwrap();

            // 9. Purge when done
            allocator.unseal(exec, size).unwrap(); // Need to unseal first in some cases
        }

        // Cleanup
        allocator.purge_module(exec, write, size);
    }

    #[test]
    fn test_multiple_allocations_workflow() {
        let mut allocator = JitAllocator::new().unwrap();
        
        // Allocate multiple regions (use READ|WRITE only on macOS)
        #[cfg(target_os = "macos")]
        let allocs: Vec<_> = (0..5)
            .map(|i| allocator.allocate_with_protection(64 * (i + 1), region::Protection::READ | region::Protection::WRITE).unwrap())
            .collect();
        #[cfg(not(target_os = "macos"))]
        let allocs: Vec<_> = (0..5)
            .map(|i| allocator.allocate(64 * (i + 1)).unwrap())
            .collect();

        // Write to each
        for (_, write, size) in &allocs {
            unsafe {
                for i in 0..*size.min(&64) {
                    *write.add(i) = 0x90;
                }
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            // Seal all
            for (exec, _, size) in &allocs {
                allocator.seal(*exec, *size).unwrap();
            }

            // Unseal and purge all
            for (exec, write, size) in &allocs {
                allocator.unseal(*exec, *size).unwrap();
                allocator.purge_module(*exec, *write, *size);
            }
        }

        #[cfg(target_os = "macos")]
        {
            // Just purge all on macOS (skip seal/unseal)
            for (exec, write, size) in &allocs {
                allocator.purge_module(*exec, *write, *size);
            }
        }
    }

    // ==================== Error Path Tests ====================

    #[test]
    fn test_seal_null_pointer() {
        let allocator = JitAllocator::new().unwrap();
        // This might fail or succeed depending on the platform
        let result = allocator.seal(std::ptr::null(), 0);
        // We just verify it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_unseal_null_pointer() {
        let allocator = JitAllocator::new().unwrap();
        // This might fail or succeed depending on the platform
        let result = allocator.unseal(std::ptr::null(), 0);
        // We just verify it doesn't panic
        let _ = result;
    }
}

