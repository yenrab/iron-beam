//! Common Utilities Module
//!
//! Provides common utility functions from 224 C files (1754 functions).
//! This module includes:
//! - String formatting and printing (erl_printf.c, erl_printf_format.c)
//! - Math utilities (erl_arith.c, erl_math.c)
//! - Miscellaneous utilities (erl_misc_utils.c)
//! - Threading utilities (ethr_*.c)
//! - And many more utility functions
//!
//! Functions are organized by category for maintainability.

#[path = "formatting.rs"]
mod formatting;
#[path = "math.rs"]
mod math;
#[path = "misc.rs"]
mod misc;
#[path = "hash.rs"]
mod hash;
#[path = "array.rs"]
mod array;
#[path = "threading.rs"]
mod threading;
#[path = "time.rs"]
mod time;
#[path = "path.rs"]
mod path;

pub use formatting::FormatUtils;
pub use math::{MathUtils, RationalUtils};
pub use misc::MiscUtils;
pub use hash::HashUtils;
pub use array::ArrayUtils;
pub use threading::ThreadingUtils;
pub use time::TimeUtils;
pub use path::PathUtils;

/// Common utility functions
///
/// This is a convenience struct that provides access to all utility categories.
/// For better organization, use the specific utility modules directly.
pub struct CommonUtils;

impl CommonUtils {
    /// Check if a utility operation succeeded
    pub fn check_operation(result: bool) -> Result<(), UtilityError> {
        if result {
            Ok(())
        } else {
            Err(UtilityError::Failed)
        }
    }
}

/// Utility operation errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UtilityError {
    /// Operation failed
    Failed,
    /// Invalid argument
    InvalidArgument,
    /// Out of memory
    OutOfMemory,
    /// Operation not supported
    NotSupported,
}

impl std::fmt::Display for UtilityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UtilityError::Failed => write!(f, "Utility operation failed"),
            UtilityError::InvalidArgument => write!(f, "Invalid argument"),
            UtilityError::OutOfMemory => write!(f, "Out of memory"),
            UtilityError::NotSupported => write!(f, "Operation not supported"),
        }
    }
}

impl std::error::Error for UtilityError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_utils_check_operation() {
        let result = CommonUtils::check_operation(true);
        assert!(result.is_ok());

        let result = CommonUtils::check_operation(false);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), UtilityError::Failed);
    }

    #[test]
    fn test_utility_error_display() {
        let err = UtilityError::Failed;
        assert_eq!(format!("{}", err), "Utility operation failed");
    }
}

