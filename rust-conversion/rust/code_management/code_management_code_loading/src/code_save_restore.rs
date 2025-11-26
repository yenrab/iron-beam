//! Code Save/Restore Module
//!
//! Provides code save and restore functionality.
//! Based on custom_code_save_restore_yield_state_alt_syntax.c

use std::fs;
use std::path::Path;

/// Code save/restore manager
pub struct CodeSaveRestore;

impl CodeSaveRestore {
    /// Save code to file
    ///
    /// # Arguments
    /// * `code` - Code bytes to save
    /// * `path` - Path to save file
    ///
    /// # Returns
    /// Result indicating success or error
    pub fn save_code<P: AsRef<Path>>(code: &[u8], path: P) -> Result<(), SaveError> {
        fs::write(path, code).map_err(|_| SaveError::FileError)
    }

    /// Restore code from file
    ///
    /// # Arguments
    /// * `path` - Path to code file
    ///
    /// # Returns
    /// Restored code bytes or error
    pub fn restore_code<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, RestoreError> {
        fs::read(path).map_err(|_| RestoreError::FileError)
    }
}

/// Save operation errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveError {
    /// File error
    FileError,
}

/// Restore operation errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestoreError {
    /// File error
    FileError,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_code_save_restore() {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_save_restore.bin");
        let test_code = b"test code data";
        
        CodeSaveRestore::save_code(test_code, &test_file).unwrap();
        let restored = CodeSaveRestore::restore_code(&test_file).unwrap();
        assert_eq!(restored, test_code);
        
        let _ = fs::remove_file(&test_file);
    }
}

