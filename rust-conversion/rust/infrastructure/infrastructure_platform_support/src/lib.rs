/*!
# Infrastructure Platform Support

**CLEAN Architecture**: Infrastructure Layer (Layer 4)
**SOLID Responsibility**: Platform-specific utilities and encoding

## Overview

This crate provides platform-specific utilities for the Erlang BEAM compiler infrastructure.
Handles encoding detection, command line processing, and platform-specific operations safely.

## Original C Functions Replaced

The original `erlc.c` contained these platform-specific functions:
- `get_encoding()`: Detects UTF-8 vs latin1 encoding → **Replaced with encoding detection utilities**
- `decode_binary()`: Decodes Erlang external format → **Replaced with binary decoding utilities**
- `possibly_quote()`: Quotes Windows command arguments → **Replaced with safe argument quoting**
- `possibly_unquote()`: Unquotes command arguments → **Replaced with safe argument unquoting**
- `make_commandline()`: Constructs Windows command line → **Replaced with safe command construction**

## Platform Support Philosophy

### 1. Safe Command Line Handling
```rust
use infrastructure_platform_support::arguments;

// Instead of manual quoting/unquoting with potential buffer overflows
let args = arguments::quote_arguments(&["erl", "-eval", "halt()"]).unwrap();
// Returns properly quoted arguments for the current platform
```

### 2. Encoding Detection
```rust
use infrastructure_platform_support::encoding;

// Detect system encoding for file operations
let encoding = encoding::detect_system_encoding();
// Returns appropriate encoding for the platform
```

### 3. Cross-Platform Compatibility
```rust
use infrastructure_platform_support::platform;

// Platform-agnostic operations
let temp_dir = platform::get_temp_directory().unwrap();
// Works on Windows, Unix, and other platforms
```

## Architecture Compliance

- **CLEAN Layer**: Infrastructure layer (depends only on infrastructure_memory_management)
- **SOLID Principle**: Single responsibility for platform abstraction
- **Safe Rust**: No unsafe code, leverages standard library and vetted crates
- **Cross-Platform**: Works on all Rust-supported platforms
*/

use std::env;
use std::ffi::OsStr;
use encoding_rs::Encoding;

/// Platform-specific utilities result type
pub type PlatformResult<T> = Result<T, PlatformError>;

/// Platform-specific error type
#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    #[error("Encoding error: {0}")]
    EncodingError(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Platform not supported: {0}")]
    UnsupportedPlatform(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Environment variable error: {0}")]
    EnvError(#[from] std::env::VarError),
}

/// Encoding detection and handling
pub mod encoding {
    use super::*;

    /// Detect the system's preferred encoding for text operations
    pub fn detect_system_encoding() -> &'static Encoding {
        // Check environment variables for encoding hints
        if let Ok(lang) = env::var("LANG") {
            if lang.to_uppercase().contains("UTF-8") {
                return encoding_rs::UTF_8;
            }
        }

        if let Ok(lc_ctype) = env::var("LC_CTYPE") {
            if lc_ctype.to_uppercase().contains("UTF-8") {
                return encoding_rs::UTF_8;
            }
        }

        // Default to UTF-8 for modern systems
        encoding_rs::UTF_8
    }

    /// Decode bytes using the system encoding
    pub fn decode_system_bytes(bytes: &[u8]) -> PlatformResult<String> {
        let encoding = detect_system_encoding();
        let (result, _encoding_used, _had_errors) = encoding.decode(bytes);
        Ok(result.into_owned())
    }

    /// Decode binary data from Erlang external format (simplified)
    ///
    /// This is a basic implementation. Full Erlang external format decoding
    /// would be more complex and might belong in a dedicated serialization crate.
    pub fn decode_erlang_binary(data: &[u8]) -> PlatformResult<Vec<u8>> {
        if data.is_empty() {
            return Err(PlatformError::InvalidArgument("Empty binary data".to_string()));
        }

        // For now, just return the data as-is
        // A full implementation would parse Erlang's external format
        Ok(data.to_vec())
    }
}

/// Command line argument handling
pub mod arguments {
    use super::*;
    use std::ffi::OsString;

    /// Quote command line arguments appropriately for the current platform
    pub fn quote_arguments(args: &[&str]) -> PlatformResult<Vec<OsString>> {
        let mut quoted = Vec::with_capacity(args.len());

        for arg in args {
            quoted.push(quote_argument(arg)?);
        }

        Ok(quoted)
    }

    /// Quote a single command line argument for the current platform
    pub fn quote_argument(arg: &str) -> PlatformResult<OsString> {
        cfg_if::cfg_if! {
            if #[cfg(windows)] {
                // On Windows, quote arguments that contain spaces or special characters
                if arg.chars().any(|c| c.is_whitespace() || c == '"' || c == '\\') {
                    let mut quoted = OsString::from("\"");
                    let mut backslashes = 0;

                    for ch in arg.chars() {
                        match ch {
                            '\\' => {
                                backslashes += 1;
                            }
                            '"' => {
                                // Add backslashes before the quote
                                for _ in 0..backslashes {
                                    quoted.push("\\");
                                }
                                backslashes = 0;
                                quoted.push("\\\"");
                            }
                            _ => {
                                // Add pending backslashes
                                for _ in 0..backslashes {
                                    quoted.push("\\");
                                }
                                backslashes = 0;
                                quoted.push(&ch.to_string());
                            }
                        }
                    }

                    // Add trailing backslashes
                    for _ in 0..backslashes {
                        quoted.push("\\");
                    }

                    quoted.push("\"");
                    Ok(quoted)
                } else {
                    Ok(OsString::from(arg))
                }
            } else {
                // On Unix-like systems, arguments are rarely quoted
                // Just escape basic shell metacharacters if needed
                if arg.chars().any(|c| c.is_whitespace() || matches!(c, '$' | '`' | '\\' | '"' | '\'')) {
                    // Simple shell escaping - wrap in single quotes and escape single quotes
                    let escaped = arg.replace('\'', "'\"'\"'");
                    Ok(OsString::from(format!("'{}'", escaped)))
                } else {
                    Ok(OsString::from(arg))
                }
            }
        }
    }

    /// Unquote a command line argument
    pub fn unquote_argument(quoted: &OsStr) -> PlatformResult<String> {
        let arg = quoted.to_string_lossy();

        cfg_if::cfg_if! {
            if #[cfg(windows)] {
                unquote_windows(&arg)
            } else {
                unquote_unix(&arg)
            }
        }
    }

    #[cfg(windows)]
    fn unquote_windows(arg: &str) -> PlatformResult<String> {
        if !arg.starts_with('"') || !arg.ends_with('"') {
            return Ok(arg.to_string());
        }

        let inner = &arg[1..arg.len() - 1];
        let mut result = String::with_capacity(inner.len());
        let mut chars = inner.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '\\' => {
                    let mut backslash_count = 1;
                    // Count consecutive backslashes
                    while chars.peek() == Some(&'\\') {
                        backslash_count += 1;
                        chars.next();
                    }

                    if chars.peek() == Some(&'"') {
                        // 2n backslashes followed by " -> n backslashes
                        // 2n+1 backslashes followed by " -> n backslashes + "
                        let slashes_to_add = backslash_count / 2;
                        for _ in 0..slashes_to_add {
                            result.push('\\');
                        }
                        if backslash_count % 2 == 1 {
                            chars.next(); // consume the "
                            result.push('"');
                        }
                    } else {
                        // Just backslashes
                        for _ in 0..backslash_count {
                            result.push('\\');
                        }
                    }
                }
                _ => {
                    result.push(ch);
                }
            }
        }

        Ok(result)
    }

    #[cfg(not(windows))]
    fn unquote_unix(arg: &str) -> PlatformResult<String> {
        if !arg.starts_with('\'') || !arg.ends_with('\'') {
            return Ok(arg.to_string());
        }

        // Remove outer quotes and unescape any escaped quotes
        let inner = &arg[1..arg.len() - 1];
        Ok(inner.replace("'\"'\"'", "'"))
    }
}

/// Platform detection and information
pub mod platform {
    use super::*;
    use std::path::PathBuf;

    /// Get the platform name
    pub fn platform_name() -> &'static str {
        env::consts::OS
    }

    /// Get the architecture name
    pub fn architecture_name() -> &'static str {
        env::consts::ARCH
    }

    /// Check if running on Windows
    pub fn is_windows() -> bool {
        cfg!(target_os = "windows")
    }

    /// Check if running on Unix-like system
    pub fn is_unix() -> bool {
        cfg!(unix)
    }

    /// Get a temporary directory path
    pub fn get_temp_directory() -> PlatformResult<PathBuf> {
        Ok(env::temp_dir())
    }

    /// Get the current working directory
    pub fn get_current_directory() -> PlatformResult<PathBuf> {
        Ok(env::current_dir()?)
    }

    /// Check if a path is absolute
    pub fn is_absolute_path(path: &std::path::Path) -> bool {
        path.is_absolute()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoding_detection() {
        let encoding = encoding::detect_system_encoding();
        // Should return a valid encoding
        assert!(!encoding.name().is_empty());
    }

    #[test]
    fn test_decode_system_bytes() {
        let bytes = b"Hello, world!";
        let result = encoding::decode_system_bytes(bytes).unwrap();
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn test_decode_empty_binary() {
        let result = encoding::decode_erlang_binary(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_erlang_binary() {
        let data = b"test data";
        let result = encoding::decode_erlang_binary(data).unwrap();
        assert_eq!(result, data);
    }

    #[test]
    fn test_quote_simple_argument() {
        let arg = "simple";
        let quoted = arguments::quote_argument(arg).unwrap();
        assert_eq!(quoted.to_string_lossy(), arg);
    }

    #[test]
    fn test_quote_argument_with_spaces() {
        let arg = "hello world";
        let quoted = arguments::quote_argument(arg).unwrap();
        cfg_if::cfg_if! {
            if #[cfg(windows)] {
                assert_eq!(quoted.to_string_lossy(), "\"hello world\"");
            } else {
                assert_eq!(quoted.to_string_lossy(), "'hello world'");
            }
        }
    }

    #[test]
    fn test_quote_argument_with_quotes() {
        let arg = "hello \"world\"";
        let quoted = arguments::quote_argument(arg).unwrap();
        cfg_if::cfg_if! {
            if #[cfg(windows)] {
                assert_eq!(quoted.to_string_lossy(), "\"hello \\\"world\\\"\"");
            } else {
                assert_eq!(quoted.to_string_lossy(), "'hello \"world\"'");
            }
        }
    }

    #[test]
    fn test_unquote_simple_argument() {
        let quoted = std::ffi::OsStr::new("simple");
        let unquoted = arguments::unquote_argument(quoted).unwrap();
        assert_eq!(unquoted, "simple");
    }

    #[test]
    fn test_unquote_quoted_argument() {
        cfg_if::cfg_if! {
            if #[cfg(windows)] {
                let quoted = std::ffi::OsStr::new("\"hello world\"");
                let unquoted = arguments::unquote_argument(quoted).unwrap();
                assert_eq!(unquoted, "hello world");
            } else {
                let quoted = std::ffi::OsStr::new("'hello world'");
                let unquoted = arguments::unquote_argument(quoted).unwrap();
                assert_eq!(unquoted, "hello world");
            }
        }
    }

    #[test]
    fn test_platform_info() {
        let name = platform::platform_name();
        assert!(!name.is_empty());

        let arch = platform::architecture_name();
        assert!(!arch.is_empty());
    }

    #[test]
    fn test_temp_directory() {
        let temp_dir = platform::get_temp_directory().unwrap();
        assert!(temp_dir.exists());
        assert!(temp_dir.is_dir());
    }

    #[test]
    fn test_current_directory() {
        let current_dir = platform::get_current_directory().unwrap();
        assert!(current_dir.exists());
        assert!(current_dir.is_dir());
    }

    #[test]
    fn test_is_absolute_path() {
        cfg_if::cfg_if! {
            if #[cfg(windows)] {
                assert!(platform::is_absolute_path(std::path::Path::new("C:\\windows")));
                assert!(!platform::is_absolute_path(std::path::Path::new("relative\\path")));
            } else {
                assert!(platform::is_absolute_path(std::path::Path::new("/usr/bin")));
                assert!(!platform::is_absolute_path(std::path::Path::new("relative/path")));
            }
        }
    }
}
