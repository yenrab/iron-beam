//! Memory Mapping Module
//!
//! Provides memory mapping operations using Rust standard library.
//! Based on erl_mmap.c

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

