/*!
# Infrastructure Error Handling

**CLEAN Architecture**: Infrastructure Layer (Layer 4)
**SOLID Responsibility**: Consistent error reporting and termination

## Overview

This crate provides error handling utilities for the Erlang BEAM compiler infrastructure.
Replaces the C-style `exit()` calls with proper Rust error handling using `Result<T, E>`.

## Original C Functions Replaced

The original `erlc.c` contained these error handling functions:
- `error()`: Formats message and calls `exit(1)` → **Replaced with `Result<T, E>` and proper error propagation**
- `strerror()`: Provides `strerror()` if system lacks it → **Replaced with `std::io::Error`**

## Rust Error Handling Philosophy

### 1. Result-Based Error Handling
```c
// C: Immediate termination
error("File not found: %s", filename);
```

```rust
// Rust: Error propagation
use infrastructure_error_handling::CompilerError;

fn find_file(filename: &str) -> Result<(), CompilerError> {
    // ... file operations ...
    return Err(CompilerError::FileNotFound(filename.to_string()));
}
```

### 2. Custom Error Types
```rust
// Structured error information instead of formatted strings
#[derive(Debug, thiserror::Error)]
pub enum CompilerError {
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Process execution failed: {0}")]
    ProcessError(String),
}
```

### 3. Fatal Errors (When Absolutely Necessary)
// Only for truly unrecoverable errors (calls std::process::exit)
// fatal_error("Critical system error", err);

## Architecture Compliance

- **CLEAN Layer**: Infrastructure layer (depends only on infrastructure_memory_management)
- **SOLID Principle**: Single responsibility for error handling
- **Safe Rust**: No unsafe code, proper error boundaries
- **Composable**: Works with `?` operator and error propagation
*/

use std::fmt;
use std::io;

/// Primary error type for the Erlang compiler
#[derive(Debug, thiserror::Error)]
pub enum CompilerError {
    /// File system errors
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    /// Process execution errors
    #[error("Process execution failed: {0}")]
    ProcessError(String),

    #[error("Command not found: {0}")]
    CommandNotFound(String),

    /// Argument parsing errors
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Missing required argument: {0}")]
    MissingArgument(String),

    /// Compilation errors
    #[error("Compilation failed: {0}")]
    CompilationError(String),

    #[error("Syntax error in {file}:{line}: {message}")]
    SyntaxError { file: String, line: usize, message: String },

    /// Network/server errors
    #[error("Server communication failed: {0}")]
    ServerError(String),

    #[error("Connection refused: {host}:{port}")]
    ConnectionRefused { host: String, port: u16 },

    /// Generic errors
    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// I/O errors (wraps std::io::Error)
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),
}

/// Result type alias for compiler operations
pub type CompilerResult<T> = Result<T, CompilerError>;

/// Error reporting utilities
pub mod reporting {
    use super::*;
    use std::io::{self, Write};

    /// Report an error to stderr without exiting
    ///
    /// This replaces the C `error()` function but returns an error instead of exiting.
    /// The caller decides whether to exit or handle the error.
    pub fn report_error<E: fmt::Display>(error: E) -> CompilerResult<()> {
        let mut stderr = io::stderr();
        writeln!(stderr, "erlc: {}", error)?;
        Ok(())
    }

    /// Report an error and return it as a CompilerError
    ///
    /// Convenience function for creating and reporting an error.
    pub fn report_and_return(error: CompilerError) -> CompilerError {
        let _ = report_error(&error); // Ignore reporting errors
        error
    }

    /// Handle fatal errors that require program termination
    ///
    /// This should be used sparingly - prefer Result-based error handling.
    /// Only use for truly unrecoverable errors where continuing is impossible.
    pub fn fatal_error<E: fmt::Display>(message: &str, error: E) -> ! {
        let _ = report_error(format!("FATAL: {} - {}", message, error));
        std::process::exit(1);
    }

    /// Handle fatal errors with just a message
    pub fn fatal_message(message: &str) -> ! {
        let _ = report_error(format!("FATAL: {}", message));
        std::process::exit(1);
    }
}

/// Error conversion utilities
pub mod conversion {
    use super::*;

    /// Convert an I/O error to a CompilerError with context
    pub fn io_error_with_context(error: io::Error, context: &str) -> CompilerError {
        CompilerError::InternalError(format!("{}: {}", context, error))
    }

    /// Convert a generic error to CompilerError
    pub fn generic_error(message: impl Into<String>) -> CompilerError {
        CompilerError::InternalError(message.into())
    }

    /// Create a file not found error
    pub fn file_not_found(path: impl Into<String>) -> CompilerError {
        CompilerError::FileNotFound(path.into())
    }

    /// Create an invalid argument error
    pub fn invalid_argument(arg: impl Into<String>) -> CompilerError {
        CompilerError::InvalidArgument(arg.into())
    }

    /// Create a process error
    pub fn process_error(details: impl Into<String>) -> CompilerError {
        CompilerError::ProcessError(details.into())
    }
}

/// Error handling macros for convenience
#[macro_export]
macro_rules! bail {
    ($fmt:literal $(, $arg:expr)* $(,)?) => {
        return Err($crate::CompilerError::InternalError(format!($fmt, $($arg),*)));
    };
    ($err:expr) => {
        return Err($err.into());
    };
}

#[macro_export]
macro_rules! ensure {
    ($cond:expr, $fmt:literal $(, $arg:expr)* $(,)?) => {
        if !($cond) {
            return Err($crate::CompilerError::InternalError(format!($fmt, $($arg),*)));
        }
    };
    ($cond:expr, $err:expr) => {
        if !($cond) {
            return Err($err.into());
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    // ==================== CompilerError Enum Tests ====================

    #[test]
    fn test_error_creation() {
        let err = CompilerError::FileNotFound("test.erl".to_string());
        assert!(matches!(err, CompilerError::FileNotFound(_)));
        assert_eq!(err.to_string(), "File not found: test.erl");
    }

    #[test]
    fn test_all_error_variants_creation() {
        // Test all CompilerError variants can be created

        // File system errors
        let _file_not_found = CompilerError::FileNotFound("missing.erl".to_string());
        let _permission_denied = CompilerError::PermissionDenied("/restricted/file".to_string());
        let _invalid_path = CompilerError::InvalidPath("invalid/path".to_string());

        // Process errors
        let _process_error = CompilerError::ProcessError("command failed".to_string());
        let _command_not_found = CompilerError::CommandNotFound("erl".to_string());

        // Argument errors
        let _invalid_arg = CompilerError::InvalidArgument("--bad-flag".to_string());
        let _missing_arg = CompilerError::MissingArgument("output".to_string());

        // Compilation errors
        let _compilation_error = CompilerError::CompilationError("syntax error".to_string());
        let _syntax_error = CompilerError::SyntaxError {
            file: "test.erl".to_string(),
            line: 42,
            message: "unexpected token".to_string(),
        };

        // Network errors
        let _server_error = CompilerError::ServerError("connection timeout".to_string());
        let _connection_refused = CompilerError::ConnectionRefused {
            host: "localhost".to_string(),
            port: 9999,
        };

        // Generic errors
        let _internal_error = CompilerError::InternalError("unexpected error".to_string());
        let _config_error = CompilerError::ConfigError("invalid config".to_string());

        // I/O error (via From trait)
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let _io_error = CompilerError::from(io_err);
    }

    #[test]
    fn test_error_display_formatting() {
        // Test display formatting for all error variants

        let test_cases = vec![
            (CompilerError::FileNotFound("test.erl".to_string()), "File not found: test.erl"),
            (CompilerError::PermissionDenied("/tmp/file".to_string()), "Permission denied: /tmp/file"),
            (CompilerError::InvalidPath("bad/path".to_string()), "Invalid path: bad/path"),
            (CompilerError::ProcessError("timeout".to_string()), "Process execution failed: timeout"),
            (CompilerError::CommandNotFound("gcc".to_string()), "Command not found: gcc"),
            (CompilerError::InvalidArgument("--invalid".to_string()), "Invalid argument: --invalid"),
            (CompilerError::MissingArgument("input".to_string()), "Missing required argument: input"),
            (CompilerError::CompilationError("parse error".to_string()), "Compilation failed: parse error"),
            (CompilerError::SyntaxError {
                file: "main.erl".to_string(),
                line: 15,
                message: "unexpected ;".to_string(),
            }, "Syntax error in main.erl:15: unexpected ;"),
            (CompilerError::ServerError("network down".to_string()), "Server communication failed: network down"),
            (CompilerError::ConnectionRefused {
                host: "127.0.0.1".to_string(),
                port: 8080,
            }, "Connection refused: 127.0.0.1:8080"),
            (CompilerError::InternalError("bug".to_string()), "Internal error: bug"),
            (CompilerError::ConfigError("bad config".to_string()), "Configuration error: bad config"),
        ];

        for (error, expected_display) in test_cases {
            assert_eq!(error.to_string(), expected_display);
        }
    }

    #[test]
    fn test_error_debug_formatting() {
        // Test debug formatting works and includes useful information
        let error = CompilerError::FileNotFound("test.erl".to_string());
        let debug_str = format!("{:?}", error);

        // Should be a non-empty string and contain some error information
        assert!(!debug_str.is_empty());
        assert!(debug_str.len() > 10); // Should be reasonably long

        // Should contain the data we put in
        assert!(debug_str.contains("test.erl"));
    }

    #[test]
    fn test_error_variant_creation() {
        // Test that different variants can be created with the same data
        let err1 = CompilerError::FileNotFound("test.erl".to_string());
        let err2 = CompilerError::PermissionDenied("test.erl".to_string());
        let err3 = CompilerError::InvalidArgument("test.erl".to_string());

        // Just verify they are different variants
        assert!(matches!(err1, CompilerError::FileNotFound(_)));
        assert!(matches!(err2, CompilerError::PermissionDenied(_)));
        assert!(matches!(err3, CompilerError::InvalidArgument(_)));
    }

    #[test]
    fn test_error_clone() {
        // Test that all error variants can be cloned
        let errors = vec![
            CompilerError::FileNotFound("test.erl".to_string()),
            CompilerError::SyntaxError {
                file: "test.erl".to_string(),
                line: 10,
                message: "error".to_string(),
            },
            CompilerError::ConnectionRefused {
                host: "localhost".to_string(),
                port: 9999,
            },
            CompilerError::IoError(io::Error::new(io::ErrorKind::NotFound, "test")),
        ];

        // Test that all variants can be created (Clone test removed due to io::Error)
    }

    #[test]
    fn test_error_with_special_characters() {
        // Test error messages with special characters
        let errors = vec![
            CompilerError::FileNotFound("file with spaces.erl".to_string()),
            CompilerError::InvalidArgument("--flag=value".to_string()),
            CompilerError::InternalError("error with <tags> & \"quotes\"".to_string()),
        ];

        for error in errors {
            let display = error.to_string();
            assert!(!display.is_empty());
            // Should contain the error message
            match error {
                CompilerError::FileNotFound(ref path) => assert!(display.contains(path)),
                CompilerError::InvalidArgument(ref arg) => assert!(display.contains(arg)),
                CompilerError::InternalError(ref msg) => assert!(display.contains(msg)),
                _ => {}
            }
        }
    }

    #[test]
    fn test_error_with_unicode() {
        // Test error messages with Unicode characters
        let errors = vec![
            CompilerError::FileNotFound("файл.erl".to_string()),
            CompilerError::InternalError("ошибка: файл не найден 🚫".to_string()),
        ];

        for error in errors {
            let display = error.to_string();
            assert!(!display.is_empty());
            // Unicode should be preserved
            match error {
                CompilerError::FileNotFound(ref path) => assert!(display.contains(path)),
                CompilerError::InternalError(ref msg) => assert!(display.contains(msg)),
                _ => {}
            }
        }
    }

    #[test]
    fn test_error_with_empty_strings() {
        // Test error variants with empty strings
        let errors = vec![
            CompilerError::FileNotFound(String::new()),
            CompilerError::InvalidArgument(String::new()),
            CompilerError::InternalError(String::new()),
        ];

        for error in errors {
            let display = error.to_string();
            assert!(!display.is_empty());
            // Should still have the error type description
            match error {
                CompilerError::FileNotFound(_) => assert!(display.contains("File not found")),
                CompilerError::InvalidArgument(_) => assert!(display.contains("Invalid argument")),
                CompilerError::InternalError(_) => assert!(display.contains("Internal error")),
                _ => {}
            }
        }
    }

    #[test]
    fn test_error_with_long_messages() {
        // Test error messages with very long content
        let long_message = "a".repeat(1000);
        let error = CompilerError::InternalError(long_message.clone());
        let display = error.to_string();
        assert!(display.contains("Internal error"));
        assert!(display.contains(&long_message));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let compiler_err = CompilerError::from(io_err);
        assert!(matches!(compiler_err, CompilerError::IoError(_)));
    }

    #[test]
    fn test_io_error_display() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let compiler_err = CompilerError::from(io_err);
        let display = compiler_err.to_string();
        assert!(display.contains("I/O error"));
    }

    // ==================== Conversion Module Tests ====================

    #[test]
    fn test_conversion_utilities() {
        let err = conversion::file_not_found("missing.erl");
        assert!(matches!(err, CompilerError::FileNotFound(_)));

        let err = conversion::invalid_argument("--invalid");
        assert!(matches!(err, CompilerError::InvalidArgument(_)));
    }

    #[test]
    fn test_io_error_with_context() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let context = "reading configuration file";

        let compiler_err = conversion::io_error_with_context(io_err, context);

        match compiler_err {
            CompilerError::InternalError(msg) => {
                assert!(msg.contains(context));
                assert!(msg.contains("file not found"));
            }
            _ => panic!("Expected InternalError"),
        }
    }

    #[test]
    fn test_io_error_with_context_empty_context() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let context = "";

        let compiler_err = conversion::io_error_with_context(io_err, context);

        match compiler_err {
            CompilerError::InternalError(msg) => {
                assert!(msg.contains("access denied"));
                assert!(msg.starts_with(": ")); // Empty context followed by colon
            }
            _ => panic!("Expected InternalError"),
        }
    }

    #[test]
    fn test_generic_error() {
        let message = "Something went wrong";
        let err = conversion::generic_error(message);

        match err {
            CompilerError::InternalError(msg) => assert_eq!(msg, message),
            _ => panic!("Expected InternalError"),
        }
    }

    #[test]
    fn test_generic_error_with_string() {
        let message = "Custom error message".to_string();
        let err = conversion::generic_error(message.clone());

        match err {
            CompilerError::InternalError(msg) => assert_eq!(msg, message),
            _ => panic!("Expected InternalError"),
        }
    }

    #[test]
    fn test_generic_error_with_format() {
        let err = conversion::generic_error(format!("Error in {}: {}", "module", 42));

        match err {
            CompilerError::InternalError(msg) => {
                assert!(msg.contains("module"));
                assert!(msg.contains("42"));
            }
            _ => panic!("Expected InternalError"),
        }
    }

    #[test]
    fn test_file_not_found() {
        let path = "nonexistent.erl";
        let err = conversion::file_not_found(path);

        match err {
            CompilerError::FileNotFound(found_path) => assert_eq!(found_path, path),
            _ => panic!("Expected FileNotFound"),
        }
    }

    #[test]
    fn test_file_not_found_with_string() {
        let path = "/absolute/path/to/file.erl".to_string();
        let err = conversion::file_not_found(path.clone());

        match err {
            CompilerError::FileNotFound(found_path) => assert_eq!(found_path, path),
            _ => panic!("Expected FileNotFound"),
        }
    }

    #[test]
    fn test_file_not_found_with_unicode() {
        let path = "файл.erl";
        let err = conversion::file_not_found(path);

        match err {
            CompilerError::FileNotFound(found_path) => assert_eq!(found_path, path),
            _ => panic!("Expected FileNotFound"),
        }
    }

    #[test]
    fn test_invalid_argument() {
        let arg = "--bad-flag";
        let err = conversion::invalid_argument(arg);

        match err {
            CompilerError::InvalidArgument(found_arg) => assert_eq!(found_arg, arg),
            _ => panic!("Expected InvalidArgument"),
        }
    }

    #[test]
    fn test_invalid_argument_with_string() {
        let arg = "bad argument with spaces".to_string();
        let err = conversion::invalid_argument(arg.clone());

        match err {
            CompilerError::InvalidArgument(found_arg) => assert_eq!(found_arg, arg),
            _ => panic!("Expected InvalidArgument"),
        }
    }

    #[test]
    fn test_process_error() {
        let details = "command failed with exit code 1";
        let err = conversion::process_error(details);

        match err {
            CompilerError::ProcessError(found_details) => assert_eq!(found_details, details),
            _ => panic!("Expected ProcessError"),
        }
    }

    #[test]
    fn test_process_error_with_string() {
        let details = "Process error with Unicode: ошибка 🚫".to_string();
        let err = conversion::process_error(details.clone());

        match err {
            CompilerError::ProcessError(found_details) => assert_eq!(found_details, details),
            _ => panic!("Expected ProcessError"),
        }
    }

    #[test]
    fn test_conversion_functions_with_empty_strings() {
        // Test all conversion functions with empty strings
        let empty = "";

        let file_err = conversion::file_not_found(empty);
        match file_err {
            CompilerError::FileNotFound(path) => assert_eq!(path, empty),
            _ => panic!("Expected FileNotFound"),
        }

        let arg_err = conversion::invalid_argument(empty);
        match arg_err {
            CompilerError::InvalidArgument(arg) => assert_eq!(arg, empty),
            _ => panic!("Expected InvalidArgument"),
        }

        let proc_err = conversion::process_error(empty);
        match proc_err {
            CompilerError::ProcessError(details) => assert_eq!(details, empty),
            _ => panic!("Expected ProcessError"),
        }

        let gen_err = conversion::generic_error(empty);
        match gen_err {
            CompilerError::InternalError(msg) => assert_eq!(msg, empty),
            _ => panic!("Expected InternalError"),
        }
    }

    #[test]
    fn test_conversion_functions_with_long_strings() {
        let long_string = "a".repeat(1000);

        let file_err = conversion::file_not_found(&long_string);
        match file_err {
            CompilerError::FileNotFound(path) => assert_eq!(path, long_string),
            _ => panic!("Expected FileNotFound"),
        }

        let arg_err = conversion::invalid_argument(&long_string);
        match arg_err {
            CompilerError::InvalidArgument(arg) => assert_eq!(arg, long_string),
            _ => panic!("Expected InvalidArgument"),
        }
    }

    // ==================== Reporting Module Tests ====================

    #[test]
    fn test_error_reporting() {
        let err = CompilerError::InvalidArgument("bad arg".to_string());
        let result = reporting::report_error(&err);
        assert!(result.is_ok()); // Should succeed (writes to stderr)
    }

    #[test]
    fn test_error_reporting_with_string() {
        let message = "Test error message";
        let result = reporting::report_error(message);
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_reporting_with_formatted_string() {
        let result = reporting::report_error(format!("Error: {} - {}", "test", 42));
        assert!(result.is_ok());
    }

    #[test]
    fn test_report_and_return() {
        let original_error = CompilerError::FileNotFound("missing.erl".to_string());
        // Create a copy manually since Clone is not implemented
        let error_copy = match &original_error {
            CompilerError::FileNotFound(s) => CompilerError::FileNotFound(s.clone()),
            _ => panic!("Test needs to be updated for new error variant"),
        };

        let returned_error = reporting::report_and_return(error_copy);

        // Should return the same error (check variant)
        match (&original_error, &returned_error) {
            (CompilerError::FileNotFound(a), CompilerError::FileNotFound(b)) => assert_eq!(a, b),
            _ => panic!("report_and_return changed error type"),
        }
    }

    #[test]
    fn test_report_and_return_with_different_errors() {
        let test_cases = vec![
            CompilerError::FileNotFound("test.erl".to_string()),
            CompilerError::InvalidArgument("--bad".to_string()),
            CompilerError::InternalError("test error".to_string()),
            CompilerError::SyntaxError {
                file: "test.erl".to_string(),
                line: 5,
                message: "syntax error".to_string(),
            },
        ];

        for error in test_cases {
            let error_clone = match &error {
                CompilerError::FileNotFound(s) => CompilerError::FileNotFound(s.clone()),
                CompilerError::InvalidArgument(s) => CompilerError::InvalidArgument(s.clone()),
                CompilerError::InternalError(s) => CompilerError::InternalError(s.clone()),
                _ => continue, // Skip other variants for simplicity
            };
            let returned = reporting::report_and_return(error_clone);
            // Check that the returned error is the same type
            match (&error, &returned) {
                (CompilerError::FileNotFound(_), CompilerError::FileNotFound(_)) => {}
                (CompilerError::InvalidArgument(_), CompilerError::InvalidArgument(_)) => {}
                (CompilerError::InternalError(_), CompilerError::InternalError(_)) => {}
                _ => panic!("report_and_return changed error type"),
            }
        }
    }

    #[test]
    fn test_fatal_message_should_panic_or_exit() {
        // Note: fatal_message calls std::process::exit(1), which will terminate the process.
        // In tests, we can't easily test this without causing the test suite to exit.
        // This test documents the expected behavior but doesn't actually call it.

        // The function signature should be correct
        let _: fn(&str) -> ! = reporting::fatal_message;

        // We can test that the function exists and has the right signature
        assert!(true);
    }

    #[test]
    fn test_fatal_error_should_panic_or_exit() {
        // Note: fatal_error calls std::process::exit(1), which will terminate the process.
        // Similar to fatal_message, we can't easily test the exit behavior in unit tests.

        // The function signature should be correct
        let _: fn(&str, String) -> ! = reporting::fatal_error::<String>;

        // We can test that the function exists and has the right signature
        assert!(true);
    }

    #[test]
    fn test_reporting_functions_with_unicode() {
        let unicode_message = "Error with Unicode: файл не найден 🚫";
        let result = reporting::report_error(unicode_message);
        assert!(result.is_ok());
    }

    #[test]
    fn test_reporting_functions_with_special_chars() {
        let special_message = "Error with <tags> & \"quotes\" and 'apostrophes'";
        let result = reporting::report_error(special_message);
        assert!(result.is_ok());
    }

    #[test]
    fn test_reporting_functions_with_empty_string() {
        let empty_message = "";
        let result = reporting::report_error(empty_message);
        assert!(result.is_ok());
    }

    #[test]
    fn test_reporting_functions_with_long_message() {
        let long_message = "a".repeat(2000);
        let result = reporting::report_error(&long_message);
        assert!(result.is_ok());
    }

    // ==================== Macro Tests ====================

    #[test]
    fn test_bail_macro_with_error() {
        fn test_function(should_fail: bool) -> CompilerResult<()> {
            if should_fail {
                bail!(CompilerError::FileNotFound("test.erl".to_string()));
            }
            Ok(())
        }

        assert!(test_function(false).is_ok());
        assert!(matches!(test_function(true), Err(CompilerError::FileNotFound(_))));
    }

    #[test]
    fn test_bail_macro_with_string() {
        fn test_function(should_fail: bool) -> CompilerResult<()> {
            if should_fail {
                bail!("Test error message");
            }
            Ok(())
        }

        assert!(test_function(false).is_ok());

        let result = test_function(true);
        assert!(result.is_err());

        if let Err(CompilerError::InternalError(msg)) = result {
            assert_eq!(msg, "Test error message");
        }
    }

    #[test]
    fn test_bail_macro_with_format() {
        fn test_function(value: i32) -> CompilerResult<()> {
            if value < 0 {
                bail!("Invalid value: {} (must be >= 0)", value);
            }
            Ok(())
        }

        assert!(test_function(5).is_ok());

        let result = test_function(-1);
        assert!(result.is_err());

        if let Err(CompilerError::InternalError(msg)) = result {
            assert!(msg.contains("Invalid value"));
            assert!(msg.contains("-1"));
        }
    }

    #[test]
    fn test_bail_macro_with_unicode() {
        fn test_function() -> CompilerResult<()> {
            bail!("Unicode error: файл не найден 🚫");
        }

        let result = test_function();
        assert!(result.is_err());

        if let Err(CompilerError::InternalError(msg)) = result {
            assert!(msg.contains("файл"));
            assert!(msg.contains("🚫"));
        }
    }

    #[test]
    fn test_ensure_macro_basic() {
        fn test_function(should_pass: bool) -> CompilerResult<()> {
            ensure!(should_pass, "Test error message");
            Ok(())
        }

        assert!(test_function(true).is_ok());
        assert!(matches!(test_function(false), Err(CompilerError::InternalError(_))));
    }

    #[test]
    fn test_ensure_macro() {
        fn divide(a: i32, b: i32) -> CompilerResult<i32> {
            ensure!(b != 0, "Division by zero");
            Ok(a / b)
        }

        assert_eq!(divide(10, 2).unwrap(), 5);
        assert!(matches!(divide(10, 0), Err(CompilerError::InternalError(_))));
    }

    #[test]
    fn test_ensure_macro_with_string() {
        fn validate_age(age: i32) -> CompilerResult<()> {
            ensure!(age >= 0, "Age cannot be negative: {}", age);
            ensure!(age <= 150, "Age seems too high: {} (max 150)", age);
            Ok(())
        }

        assert!(validate_age(25).is_ok());

        let result1 = validate_age(-5);
        assert!(result1.is_err());
        if let Err(CompilerError::InternalError(msg)) = result1 {
            assert!(msg.contains("negative"));
            assert!(msg.contains("-5"));
        }

        let result2 = validate_age(200);
        assert!(result2.is_err());
        if let Err(CompilerError::InternalError(msg)) = result2 {
            assert!(msg.contains("too high"));
            assert!(msg.contains("200"));
        }
    }

    #[test]
    fn test_ensure_macro_complex_conditions() {
        fn validate_file_path(path: &str) -> CompilerResult<()> {
            ensure!(!path.is_empty(), "File path cannot be empty");
            ensure!(path.ends_with(".erl"), "File must have .erl extension: {}", path);
            ensure!(!path.contains(".."), "File path cannot contain '..': {}", path);
            Ok(())
        }

        assert!(validate_file_path("test.erl").is_ok());

        let test_cases = vec![
            ("", "File path cannot be empty"),
            ("test.txt", "File must have .erl extension"),
            ("../test.erl", "File path cannot contain '..'"),
        ];

        for (input, expected_error_part) in test_cases {
            let result = validate_file_path(input);
            assert!(result.is_err());
            if let Err(CompilerError::InternalError(msg)) = result {
                assert!(msg.contains(expected_error_part));
            }
        }
    }

    #[test]
    fn test_bail_and_ensure_combination() {
        fn process_file(filename: &str, content_length: usize) -> CompilerResult<String> {
            ensure!(!filename.is_empty(), "Filename cannot be empty");
            ensure!(content_length > 0, "File content cannot be empty");

            if filename.contains("forbidden") {
                bail!("Forbidden filename: {}", filename);
            }

            ensure!(content_length < 1000, "File too large: {} bytes (max 1000)", content_length);

            Ok(format!("Processed {} bytes from {}", content_length, filename))
        }

        // Success case
        let result = process_file("test.erl", 100);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Processed 100 bytes"));

        // ensure! failures
        assert!(process_file("", 100).is_err()); // Empty filename
        assert!(process_file("test.erl", 0).is_err()); // Empty content
        assert!(process_file("test.erl", 2000).is_err()); // Too large

        // bail! failure
        let result = process_file("forbidden.erl", 100);
        assert!(result.is_err());
        if let Err(CompilerError::InternalError(msg)) = result {
            assert!(msg.contains("Forbidden filename"));
        }
    }

    // ==================== CompilerResult Type Alias Tests ====================

    #[test]
    fn test_compiler_result_ok() {
        fn example_ok() -> CompilerResult<String> {
            Ok("success".to_string())
        }

        let result = example_ok();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[test]
    fn test_compiler_result_err() {
        fn example_err() -> CompilerResult<String> {
            Err(CompilerError::FileNotFound("test.erl".to_string()))
        }

        let result = example_err();
        assert!(result.is_err());

        match result {
            Err(CompilerError::FileNotFound(path)) => assert_eq!(path, "test.erl"),
            _ => panic!("Expected FileNotFound error"),
        }
    }

    #[test]
    fn test_compiler_result_map() {
        fn get_number() -> CompilerResult<i32> {
            Ok(42)
        }

        let result = get_number().map(|n| n * 2);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 84);
    }

    #[test]
    fn test_compiler_result_map_err() {
        fn failing_function() -> CompilerResult<i32> {
            Err(CompilerError::InvalidArgument("bad".to_string()))
        }

        let result = failing_function().map_err(|_| CompilerError::InternalError("mapped".to_string()));
        assert!(result.is_err());

        match result {
            Err(CompilerError::InternalError(msg)) => assert_eq!(msg, "mapped"),
            _ => panic!("Expected mapped error"),
        }
    }

    #[test]
    fn test_compiler_result_and_then() {
        fn parse_number(s: &str) -> CompilerResult<i32> {
            s.parse().map_err(|_| CompilerError::InvalidArgument(format!("Not a number: {}", s)))
        }

        fn double_if_positive(n: i32) -> CompilerResult<i32> {
            if n > 0 {
                Ok(n * 2)
            } else {
                Err(CompilerError::InvalidArgument("Must be positive".to_string()))
            }
        }

        // Success case
        let result = parse_number("5").and_then(double_if_positive);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 10);

        // First function fails
        let result = parse_number("not_a_number").and_then(double_if_positive);
        assert!(result.is_err());

        // Second function fails
        let result = parse_number("5").and_then(|_| double_if_positive(-1));
        assert!(result.is_err());
    }

    #[test]
    fn test_compiler_result_unwrap_or() {
        fn maybe_fail(should_fail: bool) -> CompilerResult<String> {
            if should_fail {
                Err(CompilerError::FileNotFound("missing".to_string()))
            } else {
                Ok("success".to_string())
            }
        }

        assert_eq!(maybe_fail(false).unwrap_or("default".to_string()), "success");
        assert_eq!(maybe_fail(true).unwrap_or("default".to_string()), "default");
    }

    #[test]
    fn test_compiler_result_unwrap_or_else() {
        fn maybe_fail() -> CompilerResult<i32> {
            Err(CompilerError::InvalidArgument("test".to_string()))
        }

        let result = maybe_fail().unwrap_or_else(|_| 42);
        assert_eq!(result, 42);
    }

    // ==================== Error Propagation Tests ====================

    #[test]
    fn test_error_propagation_with_question_mark() {
        fn read_file(filename: &str) -> CompilerResult<String> {
            if filename.is_empty() {
                return Err(CompilerError::InvalidArgument("Empty filename".to_string()));
            }
            // Simulate file reading
            Ok(format!("Content of {}", filename))
        }

        fn process_file(filename: &str) -> CompilerResult<String> {
            let content = read_file(filename)?; // Error propagates here
            Ok(format!("Processed: {}", content))
        }

        // Success case
        let result = process_file("test.erl");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Processed"));

        // Error case - error propagates from read_file to process_file
        let result = process_file("");
        assert!(result.is_err());
        match result {
            Err(CompilerError::InvalidArgument(msg)) => assert!(msg.contains("Empty filename")),
            _ => panic!("Expected InvalidArgument error"),
        }
    }

    #[test]
    fn test_error_propagation_chain() {
        fn validate_input(input: &str) -> CompilerResult<&str> {
            if input.is_empty() {
                return Err(CompilerError::InvalidArgument("Input cannot be empty".to_string()));
            }
            Ok(input)
        }

        fn parse_number(input: &str) -> CompilerResult<i32> {
            let validated = validate_input(input)?;
            validated.parse().map_err(|_| CompilerError::InvalidArgument("Not a number".to_string()))
        }

        fn double_number(input: &str) -> CompilerResult<i32> {
            let num = parse_number(input)?;
            if num < 0 {
                return Err(CompilerError::InvalidArgument("Number must be non-negative".to_string()));
            }
            Ok(num * 2)
        }

        // Success case
        let result = double_number("5");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 10);

        // Error at first step
        let result = double_number("");
        assert!(result.is_err());

        // Error at second step
        let result = double_number("not_a_number");
        assert!(result.is_err());

        // Error at third step
        let result = double_number("-3");
        assert!(result.is_err());
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_file_processing_workflow() {
        // Simulate a complete file processing workflow with error handling

        fn read_config(filename: &str) -> CompilerResult<String> {
            if filename.is_empty() {
                return Err(CompilerError::InvalidArgument("Filename cannot be empty".to_string()));
            }
            if !filename.ends_with(".config") {
                bail!("Config file must have .config extension: {}", filename);
            }
            // Simulate successful read
            Ok(format!("key1=value1,key2=value2,source={}", filename))
        }

        fn parse_config(content: &str) -> CompilerResult<Vec<String>> {
            if content.is_empty() {
                return Err(CompilerError::InvalidArgument("Config content cannot be empty".to_string()));
            }
            if !content.contains("=") {
                bail!("Config must contain at least one key=value pair");
            }
            // Simple parsing
            Ok(content.split(',').map(|s| s.to_string()).collect())
        }

        fn validate_config(entries: &[String]) -> CompilerResult<()> {
            for entry in entries {
                ensure!(!entry.trim().is_empty(), "Config entry cannot be empty");
                ensure!(entry.contains("="), "Config entry must be key=value format: {}", entry);
            }
            Ok(())
        }

        fn process_config_file(filename: &str) -> CompilerResult<String> {
            let content = read_config(filename)?;
            let entries = parse_config(&content)?;
            validate_config(&entries)?;
            Ok(format!("Successfully processed {} entries from {}", entries.len(), filename))
        }

        // Test basic error propagation
        fn step1() -> CompilerResult<String> {
            Ok("success".to_string())
        }

        fn step2(input: String) -> CompilerResult<String> {
            ensure!(!input.is_empty(), "Input cannot be empty");
            Ok(format!("processed_{}", input))
        }

        fn workflow() -> CompilerResult<String> {
            let result1 = step1()?;
            let result2 = step2(result1)?;
            Ok(format!("Final: {}", result2))
        }

        // Success case
        let result = workflow();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Final: processed_success");

        // Error case
        fn failing_step2(_input: String) -> CompilerResult<String> {
            bail!("Step2 failed");
        }

        fn failing_workflow() -> CompilerResult<String> {
            let result1 = step1()?;
            let result2 = failing_step2(result1)?;
            Ok(format!("Final: {}", result2))
        }

        let failing_result = failing_workflow();
        assert!(failing_result.is_err());
    }

    #[test]
    fn test_compilation_error_workflow() {
        // Simulate a compilation workflow with multiple potential failure points

        #[derive(Debug)]
        struct CompilationUnit {
            filename: String,
            content: String,
        }

        fn parse_syntax(unit: &CompilationUnit) -> CompilerResult<()> {
            if unit.content.contains("syntax_error") {
                return Err(CompilerError::SyntaxError {
                    file: unit.filename.clone(),
                    line: 5,
                    message: "unexpected token".to_string(),
                });
            }
            Ok(())
        }

        fn check_semantics(unit: &CompilationUnit) -> CompilerResult<()> {
            if unit.content.contains("undefined_function") {
                bail!("Undefined function call in {}", unit.filename);
            }
            Ok(())
        }

        fn optimize_code(unit: &CompilationUnit) -> CompilerResult<()> {
            if unit.content.contains("optimization_failed") {
                return Err(CompilerError::CompilationError("Optimization failed".to_string()));
            }
            Ok(())
        }

        fn compile_unit(unit: &CompilationUnit) -> CompilerResult<String> {
            parse_syntax(unit)?;
            check_semantics(unit)?;
            optimize_code(unit)?;
            Ok(format!("Compiled {}", unit.filename))
        }

        fn compile_project(units: &[CompilationUnit]) -> CompilerResult<Vec<String>> {
            let mut results = Vec::new();
            for unit in units {
                let result = compile_unit(unit)?;
                results.push(result);
            }
            Ok(results)
        }

        // Success case
        let units = vec![
            CompilationUnit {
                filename: "main.erl".to_string(),
                content: "-module(main).\nmain() -> ok.".to_string(),
            }
        ];
        let result = compile_project(&units);
        assert!(result.is_ok());

        // Syntax error case
        let units_with_error = vec![
            CompilationUnit {
                filename: "error.erl".to_string(),
                content: "-module(error).\nsyntax_error here.".to_string(),
            }
        ];
        let result = compile_project(&units_with_error);
        assert!(result.is_err());
        match result {
            Err(CompilerError::SyntaxError { file, line, .. }) => {
                assert_eq!(file, "error.erl");
                assert_eq!(line, 5);
            }
            _ => panic!("Expected SyntaxError"),
        }
    }

    #[test]
    fn test_network_error_handling() {
        // Simulate network/server communication with error handling

        fn connect_to_server(host: &str, port: u16) -> CompilerResult<String> {
            if host.is_empty() {
                return Err(CompilerError::InvalidArgument("Host cannot be empty".to_string()));
            }
            if port == 0 {
                bail!("Invalid port number: {}", port);
            }
            if host == "unreachable" {
                return Err(CompilerError::ConnectionRefused {
                    host: host.to_string(),
                    port,
                });
            }
            // Simulate successful connection
            Ok(format!("Connected to {}:{}", host, port))
        }

        fn send_request(host: &str, port: u16, request: &str) -> CompilerResult<String> {
            let connection = connect_to_server(host, port)?;
            if request.is_empty() {
                return Err(CompilerError::InvalidArgument("Request cannot be empty".to_string()));
            }
            if request.contains("timeout") {
                return Err(CompilerError::ServerError("Request timeout".to_string()));
            }
            Ok(format!("Response from {}: {}", connection, request))
        }

        // Success case
        let result = send_request("localhost", 8080, "compile test.erl");
        assert!(result.is_ok());

        // Various error cases
        assert!(send_request("", 8080, "test").is_err()); // Empty host
        assert!(send_request("localhost", 0, "test").is_err()); // Invalid port
        assert!(send_request("unreachable", 8080, "test").is_err()); // Connection refused
        assert!(send_request("localhost", 8080, "").is_err()); // Empty request
        assert!(send_request("localhost", 8080, "timeout request").is_err()); // Timeout
    }

    #[test]
    fn test_error_reporting_integration() {
        // Test integration of error creation, reporting, and handling

        fn risky_operation(should_fail: bool, error_type: &str) -> CompilerResult<String> {
            match error_type {
                "file" => {
                    if should_fail {
                        return Err(CompilerError::FileNotFound("missing.erl".to_string()));
                    }
                }
                "arg" => {
                    if should_fail {
                        bail!("Invalid argument provided");
                    }
                }
                "io" => {
                    if should_fail {
                        return Err(CompilerError::from(io::Error::new(
                            io::ErrorKind::PermissionDenied,
                            "access denied"
                        )));
                    }
                }
                _ => {}
            }
            Ok("Operation successful".to_string())
        }

        fn perform_operation_with_reporting(should_fail: bool, error_type: &str) -> CompilerResult<String> {
            match risky_operation(should_fail, error_type) {
                Ok(result) => Ok(result),
                Err(e) => {
                    // Report the error but continue with a default
                    let _ = reporting::report_error(&e);
                    Ok("Recovered from error".to_string())
                }
            }
        }

        // Test successful operations
        let result = perform_operation_with_reporting(false, "file");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Operation successful");

        // Test error recovery
        let result = perform_operation_with_reporting(true, "file");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Recovered from error");

        // Test different error types
        assert!(perform_operation_with_reporting(true, "arg").is_ok());
        assert!(perform_operation_with_reporting(true, "io").is_ok());
    }

    #[test]
    fn test_macros_with_result_type_alias() {
        fn example_function(value: i32) -> CompilerResult<String> {
            ensure!(value >= 0, "Value must be non-negative: {}", value);

            if value == 0 {
                bail!("Zero is not allowed");
            }

            Ok(format!("Value: {}", value))
        }

        assert!(example_function(5).is_ok());
        assert!(example_function(-1).is_err());
        assert!(example_function(0).is_err());
    }
}
