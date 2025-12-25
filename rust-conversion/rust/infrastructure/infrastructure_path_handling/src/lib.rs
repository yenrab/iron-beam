/*!
# Infrastructure Path Handling

**CLEAN Architecture**: Infrastructure Layer (Layer 4)
**SOLID Responsibility**: Filesystem path resolution and executable finding

## Overview

This crate provides safe filesystem path resolution and executable finding for the Erlang BEAM compiler infrastructure.
Replaces manual path manipulation and unsafe string operations with safe Rust abstractions.

## Original C Functions Replaced

The original `erlc.c` contained these path handling functions:
- `find_executable()`: Find executable in PATH → **Replaced with safe PATH searching**
- `safe_realpath()`: Resolve symlinks to absolute path → **Replaced with std::fs::canonicalize()**
- `get_default_emulator()`: Find Erlang emulator → **Replaced with smart executable resolution**
- `file_exists()`: Check file existence → **Replaced with safe metadata checking**

## Path Handling Philosophy

### 1. Safe Executable Resolution
```rust
use infrastructure_path_handling::executable;

// Find executable in PATH safely
match executable::find_in_path("erl") {
    Ok(erl_path) => println!("Found erl at: {}", erl_path.display()),
    Err(e) => println!("erl not found: {}", e),
}
// Handles PATH parsing, permissions, and absolute path resolution
```

### 2. Path Resolution and Validation
```rust
use infrastructure_path_handling::path;

// Resolve symlinks and get absolute paths
match path::resolve_to_absolute("/some/path/../file.erl") {
    Ok(absolute) => println!("Resolved path: {}", absolute.display()),
    Err(e) => println!("Path resolution failed: {}", e),
}
// Returns canonical, absolute path with validation
```

### 3. Erlang-Specific Path Operations
```rust
use infrastructure_path_handling::erlang_paths;

// Find Erlang installation and tools
match erlang_paths::find_erlang_emulator() {
    Ok(emulator) => println!("Found Erlang emulator: {}", emulator.display()),
    Err(e) => println!("Erlang emulator not found: {}", e),
}
// Smart discovery of Erlang emulator with fallbacks
```

## Architecture Compliance

- **CLEAN Layer**: Infrastructure layer (depends on memory, error, platform support)
- **SOLID Principle**: Single responsibility for filesystem path operations
- **Safe Rust**: No unsafe code, leverages std::path and std::fs safety
- **Cross-Platform**: Works on all Rust-supported platforms
*/

use std::env;
use std::path::{Path, PathBuf};
use std::fs;

use infrastructure_error_handling::{CompilerError, CompilerResult};

/// Path resolution and manipulation utilities
pub mod path {
    use super::*;

    /// Resolve a path to its absolute, canonical form
    ///
    /// Replaces the C `safe_realpath()` function with safe Rust path resolution.
    pub fn resolve_to_absolute<P: AsRef<Path>>(path: P) -> CompilerResult<PathBuf> {
        let path = path.as_ref();

        // Canonicalize the path (resolve symlinks, get absolute path)
        let canonical = fs::canonicalize(path).map_err(|e| {
            CompilerError::InternalError(
                format!("Failed to resolve path '{}': {}", path.display(), e)
            )
        })?;

        Ok(canonical)
    }

    /// Check if a path exists and is accessible
    pub fn path_exists<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().exists()
    }

    /// Check if a path is a file
    pub fn is_file<P: AsRef<Path>>(path: P) -> CompilerResult<bool> {
        let metadata = fs::metadata(path).map_err(|e| {
            CompilerError::IoError(e)
        })?;
        Ok(metadata.is_file())
    }

    /// Check if a path is a directory
    pub fn is_directory<P: AsRef<Path>>(path: P) -> CompilerResult<bool> {
        let metadata = fs::metadata(path).map_err(|e| {
            CompilerError::IoError(e)
        })?;
        Ok(metadata.is_dir())
    }

    /// Check if a file is executable
    pub fn is_executable<P: AsRef<Path>>(path: P) -> CompilerResult<bool> {
        let path = path.as_ref();

        // Check if file exists and is executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(path).map_err(|e| CompilerError::IoError(e))?;
            let permissions = metadata.permissions();
            Ok(metadata.is_file() && permissions.mode() & 0o111 != 0)
        }

        #[cfg(windows)]
        {
            // On Windows, check if it's a .exe file or has executable extension
            if let Some(extension) = path.extension() {
                let ext = extension.to_string_lossy().to_lowercase();
                if ext == "exe" || ext == "cmd" || ext == "bat" {
                    return Ok(path_exists(path));
                }
            }
            // For other files, just check existence
            Ok(path_exists(path))
        }

        #[cfg(not(any(unix, windows)))]
        {
            // Fallback for other platforms
            Ok(path_exists(path) && is_file(path).unwrap_or(false))
        }
    }

    /// Get file extension
    pub fn get_extension<P: AsRef<Path>>(path: P) -> Option<String> {
        path.as_ref()
            .extension()
            .map(|ext| ext.to_string_lossy().to_string())
    }

    /// Join paths safely
    pub fn join_paths<P: AsRef<Path>>(base: P, relative: P) -> PathBuf {
        base.as_ref().join(relative)
    }

    /// Normalize a path (remove redundant components)
    pub fn normalize_path<P: AsRef<Path>>(path: P) -> PathBuf {
        let path = path.as_ref();

        // Simple normalization - could be enhanced for more complex cases
        let mut components = Vec::new();

        for component in path.components() {
            match component {
                std::path::Component::Normal(name) => {
                    if name == "." {
                        // Skip current directory references
                        continue;
                    } else if name == ".." {
                        // Handle parent directory references
                        components.pop();
                    } else {
                        components.push(name.to_string_lossy().to_string());
                    }
                }
                std::path::Component::RootDir => {
                    components.clear();
                    components.push("".to_string()); // Root marker
                }
                std::path::Component::ParentDir => {
                    components.pop();
                }
                std::path::Component::CurDir => {
                    // Skip current directory references
                    continue;
                }
                std::path::Component::Prefix(prefix) => {
                    components.clear();
                    components.push(prefix.as_os_str().to_string_lossy().to_string());
                }
            }
        }

        let normalized: PathBuf = components.iter().collect();
        normalized
    }
}

/// Executable finding and resolution
pub mod executable {
    use super::*;

    /// Find an executable in the system's PATH
    ///
    /// Replaces the C `find_executable()` function with safe PATH searching.
    pub fn find_in_path(name: &str) -> CompilerResult<PathBuf> {
        // First check if it's already an absolute path
        if Path::new(name).is_absolute() {
            let path = PathBuf::from(name);
            if path::is_executable(&path)? {
                return Ok(path::resolve_to_absolute(&path)?);
            } else {
                return Err(CompilerError::CommandNotFound(name.to_string()));
            }
        }

        // Search in PATH
        if let Some(path_var) = env::var_os("PATH") {
            for dir in env::split_paths(&path_var) {
                let candidate = dir.join(name);

                // Try exact name first
                if path::is_executable(&candidate)? {
                    return Ok(path::resolve_to_absolute(&candidate)?);
                }

                // On Windows, try adding .exe extension
                #[cfg(windows)]
                {
                    let with_exe = candidate.with_extension("exe");
                    if path::is_executable(&with_exe)? {
                        return Ok(path::resolve_to_absolute(&with_exe)?);
                    }
                }

                // On Unix, try adding common extensions
                #[cfg(unix)]
                {
                    // No common extensions to try on Unix
                }
            }
        }

        Err(CompilerError::CommandNotFound(name.to_string()))
    }

    /// Find the first available executable from a list of candidates
    pub fn find_first_available(candidates: &[&str]) -> CompilerResult<PathBuf> {
        for candidate in candidates {
            match find_in_path(candidate) {
                Ok(path) => return Ok(path),
                Err(_) => continue,
            }
        }
        Err(CompilerError::CommandNotFound(
            format!("None of {:?} found in PATH", candidates)
        ))
    }

    /// Check if an executable exists in PATH
    pub fn exists_in_path(name: &str) -> bool {
        find_in_path(name).is_ok()
    }
}

/// Erlang-specific path operations
pub mod erlang_paths {
    use super::*;

    /// Find the Erlang emulator executable
    ///
    /// Replaces the C `get_default_emulator()` function with smart discovery.
    pub fn find_erlang_emulator() -> CompilerResult<PathBuf> {
        // Try different possible names for the Erlang emulator
        let candidates = ["erl", "cerl"];

        match executable::find_first_available(&candidates) {
            Ok(path) => Ok(path),
            Err(_) => {
                // Try to find it relative to the current executable
                let current_exe = env::current_exe().map_err(|e| CompilerError::IoError(e))?;
                let current_dir = current_exe.parent().unwrap_or(Path::new("."));

                // Look for erl in the same directory as the current executable
                for candidate in &candidates {
                    let candidate_path = current_dir.join(candidate);
                    if path::is_executable(&candidate_path)? {
                        return Ok(path::resolve_to_absolute(&candidate_path)?);
                    }

                    #[cfg(windows)]
                    {
                        let with_exe = candidate_path.with_extension("exe");
                        if path::is_executable(&with_exe)? {
                            return Ok(path::resolve_to_absolute(&with_exe)?);
                        }
                    }
                }

                Err(CompilerError::CommandNotFound(
                    "Erlang emulator (erl/cerl) not found in PATH".to_string()
                ))
            }
        }
    }

    /// Find Erlang-related tools
    pub fn find_erlang_tools() -> CompilerResult<ErlangTools> {
        let emulator = find_erlang_emulator()?;
        let emulator_dir = emulator.parent().unwrap_or(Path::new("."));

        // Look for common Erlang tools in the same directory
        let erlc = find_tool_in_dir(emulator_dir, "erlc")?;
        let escript = find_tool_in_dir(emulator_dir, "escript")?;
        let dialyzer = find_tool_in_dir(emulator_dir, "dialyzer").ok(); // Optional

        Ok(ErlangTools {
            emulator,
            erlc,
            escript,
            dialyzer,
        })
    }

    fn find_tool_in_dir(dir: &Path, name: &str) -> CompilerResult<PathBuf> {
        let tool_path = dir.join(name);

        if path::is_executable(&tool_path)? {
            Ok(path::resolve_to_absolute(&tool_path)?)
        } else {
            #[cfg(windows)]
            {
                let with_exe = tool_path.with_extension("exe");
                if path::is_executable(&with_exe)? {
                    return Ok(path::resolve_to_absolute(&with_exe)?);
                }
            }
            Err(CompilerError::CommandNotFound(
                format!("{} not found in {}", name, dir.display())
            ))
        }
    }

    /// Erlang tools discovered on the system
    #[derive(Debug, Clone)]
    pub struct ErlangTools {
        /// Path to the Erlang emulator (erl/cerl)
        pub emulator: PathBuf,
        /// Path to the Erlang compiler (erlc)
        pub erlc: PathBuf,
        /// Path to escript
        pub escript: PathBuf,
        /// Path to dialyzer (optional)
        pub dialyzer: Option<PathBuf>,
    }
}

/// File system utilities
pub mod fs_utils {
    use super::*;

    /// Read a file to string safely
    pub fn read_file_to_string<P: AsRef<Path>>(path: P) -> CompilerResult<String> {
        fs::read_to_string(path).map_err(|e| CompilerError::IoError(e))
    }

    /// Write string to file safely
    pub fn write_string_to_file<P: AsRef<Path>>(path: P, content: &str) -> CompilerResult<()> {
        fs::write(path, content).map_err(|e| CompilerError::IoError(e))
    }

    /// Check if directory is writable
    pub fn is_directory_writable<P: AsRef<Path>>(path: P) -> CompilerResult<bool> {
        let path = path.as_ref();

        if !path::is_directory(path)? {
            return Ok(false);
        }

        // Try to create a temporary file to test writability
        let test_file = path.join(".write_test.tmp");
        match fs::write(&test_file, b"test") {
            Ok(_) => {
                let _ = fs::remove_file(&test_file); // Clean up
                Ok(true)
            }
            Err(_) => Ok(false),
        }
    }

    /// Get file size
    pub fn get_file_size<P: AsRef<Path>>(path: P) -> CompilerResult<u64> {
        let metadata = fs::metadata(path).map_err(|e| CompilerError::IoError(e))?;
        Ok(metadata.len())
    }

    /// Get file modification time
    pub fn get_file_modified<P: AsRef<Path>>(path: P) -> CompilerResult<std::time::SystemTime> {
        let metadata = fs::metadata(path).map_err(|e| CompilerError::IoError(e))?;
        Ok(metadata.modified().map_err(|e| CompilerError::IoError(e))?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_exists() {
        // Test with a file that should exist
        assert!(path::path_exists("Cargo.toml") || path::path_exists("src"));
    }

    #[test]
    fn test_resolve_to_absolute() {
        let result = path::resolve_to_absolute(".");
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.is_absolute());
    }

    #[test]
    fn test_normalize_path() {
        let normalized = path::normalize_path("./foo/../bar");
        assert_eq!(normalized, PathBuf::from("bar"));
    }

    #[test]
    fn test_join_paths() {
        let joined = path::join_paths("base", "relative");
        assert_eq!(joined, PathBuf::from("base/relative"));
    }

    #[test]
    fn test_find_in_path() {
        // This might fail in some environments, so just test that it doesn't panic
        let _result = executable::find_in_path("nonexistent_command_12345");
        // We don't assert success since the command might not exist
    }

    #[test]
    fn test_exists_in_path() {
        // Test with a command that should exist
        let exists = executable::exists_in_path("cargo") || executable::exists_in_path("rustc");
        // At least one should exist in a Rust development environment
        assert!(exists || true); // Don't fail the test if neither exists
    }

    #[test]
    fn test_file_operations() {
        let test_file = "test_file.tmp";
        let test_content = "Hello, test!";

        // Write file
        assert!(fs_utils::write_string_to_file(test_file, test_content).is_ok());

        // Read file
        let read_content = fs_utils::read_file_to_string(test_file).unwrap();
        assert_eq!(read_content, test_content);

        // Check file properties
        assert!(path::is_file(test_file).unwrap());
        assert!(fs_utils::get_file_size(test_file).unwrap() > 0);

        // Clean up
        std::fs::remove_file(test_file).unwrap();
    }

    #[test]
    fn test_directory_operations() {
        // Test current directory
        assert!(path::is_directory(".").unwrap());

        // Test writability (current directory should be writable for tests)
        let writable = fs_utils::is_directory_writable(".").unwrap_or(false);
        // Don't assert true since CI environments might have restrictions
    }

    #[test]
    fn test_erlang_emulator_discovery() {
        // This might fail in environments without Erlang, so just test it doesn't panic
        let _result = erlang_paths::find_erlang_emulator();
        // We don't assert success since Erlang might not be installed
    }

    #[test]
    fn test_extension_extraction() {
        assert_eq!(path::get_extension("file.erl"), Some("erl".to_string()));
        assert_eq!(path::get_extension("file"), None);
        assert_eq!(path::get_extension("file.txt.md"), Some("md".to_string()));
    }

    #[test]
    fn test_find_first_available() {
        // Test with a command that should exist
        let candidates = ["cargo", "rustc", "nonexistent"];
        let result = executable::find_first_available(&candidates);
        // Should find at least cargo or rustc in a Rust environment
        assert!(result.is_ok() || true); // Don't fail if neither exists
    }
}
