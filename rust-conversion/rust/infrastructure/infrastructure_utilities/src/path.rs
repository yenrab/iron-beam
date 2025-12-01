//! Path Utilities
//!
//! Provides path-related utility functions.
//! These utilities handle file path operations.

use std::path::{Path, PathBuf};

/// Path utilities for file path operations
pub struct PathUtils;

impl PathUtils {
    /// Join path components
    ///
    /// # Arguments
    /// * `components` - Path components to join
    ///
    /// # Returns
    /// Joined path
    ///
    /// # Examples
    /// ```
    /// use infrastructure_utilities::PathUtils;
    ///
    /// let path = PathUtils::join(&["/usr", "local", "bin"]);
    /// assert!(path.to_string_lossy().contains("usr"));
    /// ```
    pub fn join(components: &[&str]) -> PathBuf {
        let mut path = PathBuf::new();
        for component in components {
            path.push(component);
        }
        path
    }

    /// Get the file name from a path
    ///
    /// # Arguments
    /// * `path` - Path
    ///
    /// # Returns
    /// * `Some(name)` - File name
    /// * `None` - If no file name
    pub fn file_name(path: &Path) -> Option<String> {
        path.file_name().and_then(|n| n.to_str()).map(|s| s.to_string())
    }

    /// Get the parent directory of a path
    ///
    /// # Arguments
    /// * `path` - Path
    ///
    /// # Returns
    /// * `Some(parent)` - Parent path
    /// * `None` - If no parent
    pub fn parent(path: &Path) -> Option<PathBuf> {
        path.parent().map(|p| p.to_path_buf())
    }

    /// Get the file extension
    ///
    /// # Arguments
    /// * `path` - Path
    ///
    /// # Returns
    /// * `Some(ext)` - Extension
    /// * `None` - If no extension
    pub fn extension(path: &Path) -> Option<String> {
        path.extension().and_then(|e| e.to_str()).map(|s| s.to_string())
    }

    /// Check if a path is absolute
    ///
    /// # Arguments
    /// * `path` - Path
    ///
    /// # Returns
    /// `true` if absolute
    pub fn is_absolute(path: &Path) -> bool {
        path.is_absolute()
    }

    /// Normalize a path (remove redundant separators, etc.)
    ///
    /// # Arguments
    /// * `path` - Path to normalize
    ///
    /// # Returns
    /// Normalized path
    pub fn normalize(path: &Path) -> PathBuf {
        path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
    }

    /// Convert a path to a string
    ///
    /// # Arguments
    /// * `path` - Path
    ///
    /// # Returns
    /// * `Some(string)` - Path as string
    /// * `None` - If path contains invalid UTF-8
    pub fn to_string(path: &Path) -> Option<String> {
        path.to_str().map(|s| s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join() {
        let path = PathUtils::join(&["usr", "local", "bin"]);
        assert!(path.to_string_lossy().contains("usr"));
    }

    #[test]
    fn test_file_name() {
        let path = Path::new("/usr/local/bin/file.txt");
        assert_eq!(PathUtils::file_name(path), Some("file.txt".to_string()));
    }

    #[test]
    fn test_parent() {
        let path = Path::new("/usr/local/bin");
        let parent = PathUtils::parent(path);
        assert!(parent.is_some());
    }

    #[test]
    fn test_extension() {
        let path = Path::new("file.txt");
        assert_eq!(PathUtils::extension(path), Some("txt".to_string()));
        
        let path = Path::new("file");
        assert_eq!(PathUtils::extension(path), None);
    }

    #[test]
    fn test_is_absolute() {
        #[cfg(unix)]
        {
            assert!(PathUtils::is_absolute(Path::new("/usr")));
            assert!(!PathUtils::is_absolute(Path::new("usr")));
        }
    }

    #[test]
    fn test_to_string() {
        let path = Path::new("test");
        assert_eq!(PathUtils::to_string(path), Some("test".to_string()));
    }
}

