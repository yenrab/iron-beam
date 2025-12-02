//! Code Loader Module
//!
//! Provides code loading functionality.

use std::fs;
use std::path::Path;

/// Code loader
pub struct CodeLoader;

impl CodeLoader {
    /// Load code from file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, LoadError> {
        fs::read(path).map_err(|_| LoadError::FileError)
    }
}

/// Code loading errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadError {
    /// File error
    FileError,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_code_loader() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_code.bin");
        
        fs::write(&test_file, b"test code").unwrap();
        let code = CodeLoader::load_from_file(&test_file).unwrap();
        assert_eq!(code, b"test code");
        
        let _ = fs::remove_file(&test_file);
    }
}

