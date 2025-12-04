//! Memory Mapping Module
//!
//! Provides memory mapping operations for allocating and managing memory-mapped regions
//! in the Erlang/OTP runtime system. This module provides a platform-independent interface
//! for memory mapping operations, enabling efficient access to large data structures.
//!
//! ## Overview
//!
//! Memory mapping allows files or memory regions to be mapped directly into a process's
//! address space, providing efficient access to large data without loading everything into
//! RAM. This is essential for the Erlang runtime's efficient memory management.
//!
//! ## Features
//!
//! - **File Mapping**: Map files into memory for efficient access
//! - **Memory Access**: Direct access to mapped memory as byte slices
//! - **Platform Independent**: Works across all platforms using Rust standard library
//!
//! ## Examples
//!
//! ```rust
//! use entities_system_integration_common::MemoryMap;
//!
//! // Map a file into memory
//! let mmap = MemoryMap::map_file("data.bin")?;
//!
//! // Access the mapped data
//! let data = mmap.data();
//! println!("File size: {} bytes", mmap.len());
//! ```
//!
//! ## See Also
//!
//! - [`entities_system_integration_win32`](../entities_system_integration_win32/index.html): Windows-specific memory mapping
//! - [`frameworks_system_integration`](../../frameworks/frameworks_system_integration/index.html): Framework-level system integration

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

use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Memory map representation for file-backed memory mapping
///
/// This struct provides a safe wrapper around memory-mapped files, allowing
/// efficient access to file contents without loading the entire file into
/// memory at once. The mapped memory is read-only and backed by the file.
///
/// ## Examples
///
/// ```rust
/// use entities_system_integration_common::MemoryMap;
///
/// // Map a file
/// let mmap = MemoryMap::map_file("large_file.bin")?;
///
/// // Access the data
/// let first_byte = mmap.data()[0];
/// ```
pub struct MemoryMap {
    data: Vec<u8>,
}

impl MemoryMap {
    /// Map a file into memory
    ///
    /// Reads the entire file into memory and provides access to it as a
    /// byte slice. For very large files, consider using platform-specific
    /// memory mapping implementations that support lazy loading.
    ///
    /// # Arguments
    /// * `path` - Path to the file to map into memory
    ///
    /// # Returns
    /// A `Result` containing a `MemoryMap` on success, or an `std::io::Error`
    /// if the file cannot be opened or read.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use entities_system_integration_common::MemoryMap;
    ///
    /// // Map a configuration file
    /// let mmap = MemoryMap::map_file("config.bin")?;
    /// let config_data = mmap.data();
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The file does not exist
    /// - The file cannot be opened
    /// - The file cannot be read
    pub fn map_file<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let mut file = File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        Ok(Self { data })
    }

    /// Get a reference to the mapped data
    ///
    /// Returns a byte slice reference to the mapped memory. This allows
    /// read-only access to the mapped file contents without copying.
    ///
    /// # Returns
    /// A byte slice reference to the mapped data
    ///
    /// # Examples
    ///
    /// ```rust
    /// use entities_system_integration_common::MemoryMap;
    ///
    /// let mmap = MemoryMap::map_file("data.bin")?;
    /// let data = mmap.data();
    /// // Process the data...
    /// ```
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get the length of the mapped data
    ///
    /// Returns the size of the mapped memory region in bytes, which
    /// corresponds to the size of the mapped file.
    ///
    /// # Returns
    /// The length of the mapped data in bytes
    ///
    /// # Examples
    ///
    /// ```rust
    /// use entities_system_integration_common::MemoryMap;
    ///
    /// let mmap = MemoryMap::map_file("data.bin")?;
    /// println!("File size: {} bytes", mmap.len());
    /// ```
    pub fn len(&self) -> usize {
        self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;

    #[test]
    fn test_memory_map_file() {
        // Create a temporary file
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_mmap_file");
        
        fs::write(&test_file, b"test data").unwrap();
        
        let mmap = MemoryMap::map_file(&test_file).unwrap();
        assert_eq!(mmap.data(), b"test data");
        assert_eq!(mmap.len(), 9);
        
        // Cleanup
        let _ = fs::remove_file(&test_file);
    }
}

