//! File NIF Module
//!
//! Provides file operations for NIFs.
//! Based on prim_file_nif.c

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

/// File NIF operations
pub struct FileNif;

impl FileNif {
    /// Open a file
    pub fn open<P: AsRef<Path>>(path: P) -> Result<FileHandle, NifError> {
        File::open(path)
            .map(|f| FileHandle { file: f })
            .map_err(|_| NifError::BadArg)
    }

    /// Create a file
    pub fn create<P: AsRef<Path>>(path: P) -> Result<FileHandle, NifError> {
        File::create(path)
            .map(|f| FileHandle { file: f })
            .map_err(|_| NifError::BadArg)
    }
}

/// File handle wrapper
pub struct FileHandle {
    file: File,
}

impl FileHandle {
    /// Read from file
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, NifError> {
        self.file.read(buf).map_err(|_| NifError::BadArg)
    }

    /// Write to file
    pub fn write(&mut self, buf: &[u8]) -> Result<usize, NifError> {
        self.file.write(buf).map_err(|_| NifError::BadArg)
    }
}

use super::buffer::NifError;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;

    #[test]
    fn test_file_nif() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_nif_file");
        
        // Create and write
        let mut handle = FileNif::create(&test_file).unwrap();
        handle.write(b"test data").unwrap();
        
        // Read back
        let mut handle = FileNif::open(&test_file).unwrap();
        let mut buf = [0u8; 9];
        let len = handle.read(&mut buf).unwrap();
        assert_eq!(&buf[..len], b"test data");
        
        // Cleanup
        let _ = fs::remove_file(&test_file);
    }
}

