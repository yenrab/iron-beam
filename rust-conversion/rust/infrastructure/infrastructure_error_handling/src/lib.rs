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
    ($err:expr) => {
        return Err($err.into());
    };
    ($fmt:literal $(, $arg:expr)* $(,)?) => {
        return Err($crate::CompilerError::InternalError(format!($fmt, $($arg),*)));
    };
}

#[macro_export]
macro_rules! ensure {
    ($cond:expr, $err:expr) => {
        if !($cond) {
            return Err($err.into());
        }
    };
    ($cond:expr, $fmt:literal $(, $arg:expr)* $(,)?) => {
        if !($cond) {
            return Err($crate::CompilerError::InternalError(format!($fmt, $($arg),*)));
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_error_creation() {
        let err = CompilerError::FileNotFound("test.erl".to_string());
        assert!(matches!(err, CompilerError::FileNotFound(_)));
        assert_eq!(err.to_string(), "File not found: test.erl");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let compiler_err = CompilerError::from(io_err);
        assert!(matches!(compiler_err, CompilerError::IoError(_)));
    }

    #[test]
    fn test_conversion_utilities() {
        let err = conversion::file_not_found("missing.erl");
        assert!(matches!(err, CompilerError::FileNotFound(_)));

        let err = conversion::invalid_argument("--invalid");
        assert!(matches!(err, CompilerError::InvalidArgument(_)));
    }

    #[test]
    fn test_error_reporting() {
        let err = CompilerError::InvalidArgument("bad arg".to_string());
        let result = reporting::report_error(&err);
        assert!(result.is_ok()); // Should succeed (writes to stderr)
    }

    #[test]
    fn test_ensure_macro_basic() {
        fn test_function(should_pass: bool) -> CompilerResult<()> {
            ensure!(should_pass, CompilerError::InvalidArgument("test".to_string()));
            Ok(())
        }

        assert!(test_function(true).is_ok());
        assert!(matches!(test_function(false), Err(CompilerError::InvalidArgument(_))));
    }

    #[test]
    fn test_ensure_macro() {
        fn divide(a: i32, b: i32) -> CompilerResult<i32> {
            ensure!(b != 0, CompilerError::InvalidArgument("Division by zero".to_string()));
            Ok(a / b)
        }

        assert_eq!(divide(10, 2).unwrap(), 5);
        assert!(matches!(divide(10, 0), Err(CompilerError::InvalidArgument(_))));
    }
}
