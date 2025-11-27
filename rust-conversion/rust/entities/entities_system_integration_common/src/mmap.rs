//! Memory Mapping Module
//!
//! Provides memory mapping operations using Rust standard library.

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

/// Memory map representation
pub struct MemoryMap {
    data: Vec<u8>,
}

impl MemoryMap {
    /// Map a file into memory
    ///
    /// # Arguments
    /// * `path` - Path to file to map
    ///
    /// # Returns
    /// Result containing MemoryMap or error
    pub fn map_file<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let mut file = File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        Ok(Self { data })
    }

    /// Get mapped data
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get data length
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

