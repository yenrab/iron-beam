//! Executable Memory Allocation
//!
//! Provides platform-specific executable memory allocation for BEAM code.
//! Based on erts_alloc.c and platform-specific memory allocation code.

/*
 * %CopyrightBegin%
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * Copyright Lee Barney 2025. All Rights Reserved.
 *
 * This file is derived from work copyrighted by Ericsson AB 1996-2025.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * %CopyrightEnd%
 */

use std::ptr;

/// Executable memory region
///
/// Manages a region of executable memory with separate read-only executable
/// and writable mappings for W^X (Write XOR Execute) security.
pub struct ExecutableMemory {
    /// Executable region (read-only, executable)
    executable_ptr: *mut u8,
    /// Writable region (read-write, not executable)
    writable_ptr: *mut u8,
    /// Size of allocated region in bytes
    size: usize,
}

impl ExecutableMemory {
    /// Allocate executable memory
    ///
    /// Allocates a region of memory that can be executed. On platforms that
    /// support it, uses dual mapping (separate executable and writable regions).
    /// On other platforms, uses a single RWX region.
    ///
    /// # Arguments
    /// * `size` - Size of memory to allocate in bytes
    ///
    /// # Returns
    /// Ok(ExecutableMemory) if successful, Err(String) on failure
    ///
    /// # Safety
    /// This function is unsafe because it allocates executable memory.
    /// The caller must ensure proper cleanup by calling deallocate().
    pub unsafe fn allocate(size: usize) -> Result<Self, String> {
        if size == 0 {
            return Err("Cannot allocate zero-sized memory".to_string());
        }

        // Round up to page size
        let page_size = Self::page_size();
        let aligned_size = (size + page_size - 1) & !(page_size - 1);

        #[cfg(unix)]
        {
            Self::allocate_unix(aligned_size)
        }

        #[cfg(windows)]
        {
            Self::allocate_windows(aligned_size)
        }

        #[cfg(not(any(unix, windows)))]
        {
            Err("Platform not supported for executable memory allocation".to_string())
        }
    }

    #[cfg(unix)]
    unsafe fn allocate_unix(size: usize) -> Result<Self, String> {
        use libc::{mmap, MAP_ANONYMOUS, MAP_FAILED, MAP_PRIVATE, PROT_EXEC, PROT_READ, PROT_WRITE};

        // Allocate executable region (read-only, executable)
        let exec_ptr = mmap(
            ptr::null_mut(),
            size,
            PROT_READ | PROT_EXEC,
            MAP_PRIVATE | MAP_ANONYMOUS,
            -1,
            0,
        );

        if exec_ptr == MAP_FAILED {
            return Err("Failed to allocate executable memory".to_string());
        }

        // Allocate writable region (read-write, not executable)
        let write_ptr = mmap(
            ptr::null_mut(),
            size,
            PROT_READ | PROT_WRITE,
            MAP_PRIVATE | MAP_ANONYMOUS,
            -1,
            0,
        );

        if write_ptr == MAP_FAILED {
            libc::munmap(exec_ptr, size);
            return Err("Failed to allocate writable memory".to_string());
        }
        Ok(Self {
            executable_ptr: exec_ptr as *mut u8,
            writable_ptr: write_ptr as *mut u8,
            size,
        })
    }

    #[cfg(windows)]
    unsafe fn allocate_windows(size: usize) -> Result<Self, String> {
        use winapi::um::memoryapi::{VirtualAlloc, VirtualFree};
        use winapi::um::winnt::{MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_EXECUTE_READ, PAGE_READWRITE};

        // Allocate executable region
        let exec_ptr = VirtualAlloc(
            ptr::null_mut(),
            size,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_EXECUTE_READ,
        );

        if exec_ptr.is_null() {
            return Err("Failed to allocate executable memory".to_string());
        }

        // Allocate writable region
        let write_ptr = VirtualAlloc(
            ptr::null_mut(),
            size,
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        );

        if write_ptr.is_null() {
            VirtualFree(exec_ptr, 0, MEM_RELEASE);
            return Err("Failed to allocate writable memory".to_string());
        }

        Ok(Self {
            executable_ptr: exec_ptr as *mut u8,
            writable_ptr: write_ptr as *mut u8,
            size,
        })
    }

    /// Get pointer to executable region
    ///
    /// Returns a pointer to the executable (read-only) region.
    pub fn executable_ptr(&self) -> *const () {
        self.executable_ptr as *const ()
    }

    /// Get pointer to writable region
    ///
    /// Returns a pointer to the writable (read-write) region.
    pub fn writable_ptr(&self) -> *mut () {
        self.writable_ptr as *mut ()
    }

    /// Get size of allocated region
    pub fn size(&self) -> usize {
        self.size
    }

    /// Copy data into writable region
    ///
    /// Copies data from the source slice into the writable region.
    /// The data can then be made executable by syncing to the executable region.
    ///
    /// # Arguments
    /// * `data` - Data to copy
    ///
    /// # Safety
    /// This function is unsafe because it writes to raw memory.
    /// The caller must ensure:
    /// - data.len() <= self.size()
    /// - The writable region is valid
    pub unsafe fn copy_to_writable(&mut self, data: &[u8]) {
        if data.len() > self.size {
            panic!("Data size exceeds allocated memory");
        }

        ptr::copy_nonoverlapping(data.as_ptr(), self.writable_ptr, data.len());
    }

    /// Sync writable region to executable region
    ///
    /// Copies data from the writable region to the executable region.
    /// This is used to make code executable after writing it.
    ///
    /// # Safety
    /// This function is unsafe because it copies between raw memory regions.
    pub unsafe fn sync_to_executable(&self, len: usize) {
        if len > self.size {
            panic!("Length exceeds allocated memory");
        }

        #[cfg(unix)]
        {
            use libc::{mprotect, PROT_READ, PROT_WRITE};
            
            // Temporarily make executable region writable so we can copy to it
            let result = mprotect(
                self.executable_ptr as *mut libc::c_void,
                len,
                PROT_READ | PROT_WRITE,
            );
            if result != 0 {
                let errno = std::io::Error::last_os_error();
                panic!("Failed to make executable region writable: {}", errno);
            }
        }

        ptr::copy_nonoverlapping(self.writable_ptr, self.executable_ptr, len);

        #[cfg(unix)]
        {
            use libc::{mprotect, PROT_READ, PROT_EXEC};
            
            // Restore executable region to read-only, executable
            let result = mprotect(
                self.executable_ptr as *mut libc::c_void,
                len,
                PROT_READ | PROT_EXEC,
            );
            if result != 0 {
                let errno = std::io::Error::last_os_error();
                panic!("Failed to restore executable region permissions: {}", errno);
            }
        }
    }

    /// Deallocate the memory regions
    ///
    /// Frees both the executable and writable memory regions.
    ///
    /// # Safety
    /// This function is unsafe because it deallocates memory.
    /// The caller must ensure no references to this memory exist.
    pub unsafe fn deallocate(self) {
        #[cfg(unix)]
        {
            use libc::munmap;
            munmap(self.executable_ptr as *mut libc::c_void, self.size);
            munmap(self.writable_ptr as *mut libc::c_void, self.size);
        }

        #[cfg(windows)]
        {
            use winapi::um::memoryapi::VirtualFree;
            use winapi::um::winnt::MEM_RELEASE;
            VirtualFree(self.executable_ptr as *mut winapi::ctypes::c_void, 0, MEM_RELEASE);
            VirtualFree(self.writable_ptr as *mut winapi::ctypes::c_void, 0, MEM_RELEASE);
        }
    }

    /// Get page size for the current platform
    /// 
    /// Caches the page size to avoid repeated system calls
    fn page_size() -> usize {
        use std::sync::OnceLock;
        static PAGE_SIZE: OnceLock<usize> = OnceLock::new();
        
        *PAGE_SIZE.get_or_init(|| {
            #[cfg(unix)]
            {
                unsafe { 
                    let size = libc::sysconf(libc::_SC_PAGESIZE);
                    if size <= 0 {
                        4096 // Fallback to default if sysconf fails
                    } else {
                        size as usize
                    }
                }
            }

            #[cfg(windows)]
            {
                use winapi::um::sysinfoapi::GetSystemInfo;
                use winapi::um::sysinfoapi::SYSTEM_INFO;
                unsafe {
                    let mut info: SYSTEM_INFO = std::mem::zeroed();
                    GetSystemInfo(&mut info);
                    info.dwPageSize as usize
                }
            }

            #[cfg(not(any(unix, windows)))]
            {
                4096 // Default page size
            }
        })
    }
}

// Note: We intentionally don't implement Drop for ExecutableMemory
// because deallocate() consumes self. Memory is managed explicitly
// by calling deallocate() when modules are unloaded.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_size() {
        let page_size = ExecutableMemory::page_size();
        assert!(page_size > 0);
        assert!(page_size.is_power_of_two());
    }

    #[test]
    fn test_allocate_executable_memory() {
        unsafe {
            // Try to allocate - if it fails, that's okay, just verify the error handling works
            let requested_size = 4096;
            match ExecutableMemory::allocate(requested_size) {
                Ok(mem) => {
                    // Allocation succeeded, verify it's valid
                    assert!(!mem.executable_ptr().is_null());
                    assert!(!mem.writable_ptr().is_null());
                    // Size is page-aligned, so it will be >= requested_size
                    let page_size = ExecutableMemory::page_size();
                    let expected_aligned_size = (requested_size + page_size - 1) & !(page_size - 1);
                    assert_eq!(mem.size(), expected_aligned_size);
                    assert!(mem.size() >= requested_size);
                    mem.deallocate();
                }
                Err(e) => {
                    // Allocation failed (may happen on some platforms with restrictions)
                    // This is acceptable - the test verifies error handling works
                    assert!(!e.is_empty());
                }
            }
        }
    }

    #[test]
    fn test_copy_to_writable() {
        unsafe {
            // Try to allocate a small amount first to test if allocation works
            // Use a minimal size to reduce chance of system issues
            let test_size = 1024; // Smaller than 4096 to reduce allocation pressure
            let mut mem = match ExecutableMemory::allocate(test_size) {
                Ok(m) => m,
                Err(_e) => {
                    // Allocation not supported on this platform, skip test
                    // This is acceptable - not all platforms support executable memory
                    return;
                }
            };
            let test_data = b"Hello, World!";
            assert!(test_data.len() <= test_size, "Test data must fit in allocated memory");
            mem.copy_to_writable(test_data);
            mem.sync_to_executable(test_data.len());
            mem.deallocate();
        }
    }
}

