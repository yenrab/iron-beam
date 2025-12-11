//! Code Loader Module
//!
//! Provides code loading functionality for reading code files from the filesystem.
//! This module provides a simple interface for loading code files that will be
//! processed by higher-level code loading infrastructure.
//!
//! ## Overview
//!
//! The code loader provides basic file I/O operations for reading code files.
//! Higher-level code loading operations (parsing, validation, etc.) are handled
//! by the code management layer.
//!
//! ## Examples
//!
//! ```rust,no_run
//! use infrastructure_code_loading::code_loader::CodeLoader;
//!
//! // Load code from a file
//! let code = CodeLoader::load_from_file("module.beam").unwrap();
//! println!("Loaded {} bytes", code.len());
//! ```
//!
//! ## See Also
//!
//! - [`code_management_code_loading`](../../code_management/code_management_code_loading/index.html): High-level code loading
//! - [`beam_loader`](../../code_management/code_management_code_loading/beam_loader/index.html): BEAM file loading

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

