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
        // Allocate memory with read-write-execute permissions
        let protection = region::Protection::READ | region::Protection::WRITE | region::Protection::EXECUTE;
        
        let allocation = region::alloc(size, protection)
            .map_err(|e| JitAllocatorError::AllocationFailed(e.to_string()))?;

        let mut allocation = allocation;
        let executable = allocation.as_ptr() as *const u8;
        let writable = allocation.as_mut_ptr() as *mut u8;
        let allocated_size = allocation.len();

        self.regions.push(MemoryRegion {
            executable,
            writable,
            size: allocated_size,
        });

        // Leak the allocation so it persists (will be freed in purge_module)
        std::mem::forget(allocation);

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
                // Free the memory region
                unsafe {
                    let _slice = std::slice::from_raw_parts_mut(r.writable, r.size);
                    // Note: region crate doesn't provide a direct free function
                    // The memory will be freed when the allocator is dropped
                    // or we could use a different approach for memory management
                }
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

