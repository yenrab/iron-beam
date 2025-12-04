//! Debug Utilities Module
//!
//! Provides debugging utility functions.
//! Based on erl_debug.c and beam_debug.c
//!
//! This module provides infrastructure for debugging:
//! - Debug output utilities
//! - Term display and formatting
//! - Debug state management
//! - Integration with debugging adapters

use std::sync::atomic::{AtomicBool, Ordering};
use infrastructure_data_handling::print_term::{print_term, s_print_term};
use entities_data_handling::term_hashing::Term;
use entities_utilities::BigNumber;

/// Global debug state
static DEBUG_ENABLED: AtomicBool = AtomicBool::new(false);
static VERBOSE_DEBUG: AtomicBool = AtomicBool::new(false);

/// Debug utilities for debugging operations
pub struct DebugUtils;

impl DebugUtils {
    /// Enable debug output
    ///
    /// When enabled, debug output functions will produce output.
    /// When disabled, debug output is suppressed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use infrastructure_debugging::DebugUtils;
    ///
    /// DebugUtils::enable();
    /// DebugUtils::debug_output("This will be printed");
    /// DebugUtils::disable();
    /// DebugUtils::debug_output("This will be suppressed");
    /// ```
    pub fn enable() {
        DEBUG_ENABLED.store(true, Ordering::Release);
    }

    /// Disable debug output
    ///
    /// When disabled, debug output functions will not produce output.
    pub fn disable() {
        DEBUG_ENABLED.store(false, Ordering::Release);
    }

    /// Check if debug output is enabled
    ///
    /// # Returns
    ///
    /// `true` if debug output is enabled, `false` otherwise
    pub fn is_enabled() -> bool {
        DEBUG_ENABLED.load(Ordering::Acquire)
    }

    /// Enable verbose debug output
    ///
    /// Verbose mode provides more detailed debug information.
    pub fn enable_verbose() {
        VERBOSE_DEBUG.store(true, Ordering::Release);
    }

    /// Disable verbose debug output
    pub fn disable_verbose() {
        VERBOSE_DEBUG.store(false, Ordering::Release);
    }

    /// Check if verbose debug is enabled
    ///
    /// # Returns
    ///
    /// `true` if verbose debug is enabled, `false` otherwise
    pub fn is_verbose() -> bool {
        VERBOSE_DEBUG.load(Ordering::Acquire)
    }

    /// Output a debug message
    ///
    /// This function outputs a debug message if debug output is enabled.
    /// Similar to `io:format` in Erlang or `printf` in C.
    ///
    /// # Arguments
    ///
    /// * `message` - The debug message to output
    ///
    /// # Examples
    ///
    /// ```rust
    /// use infrastructure_debugging::DebugUtils;
    ///
    /// DebugUtils::enable();
    /// DebugUtils::debug_output("Debug message");
    /// ```
    pub fn debug_output(message: &str) {
        if Self::is_enabled() {
            eprintln!("[DEBUG] {}", message);
        }
    }

    /// Output a formatted debug message
    ///
    /// Similar to `debug_output`, but supports formatting with arguments.
    ///
    /// # Arguments
    ///
    /// * `format` - Format string (supports {} placeholders)
    /// * `args` - Arguments to format
    ///
    /// # Examples
    ///
    /// ```rust
    /// use infrastructure_debugging::DebugUtils;
    ///
    /// DebugUtils::enable();
    /// DebugUtils::debug_format("Value: {}", &[&42]);
    /// ```
    pub fn debug_format(format: &str, args: &[&dyn std::fmt::Display]) {
        if Self::is_enabled() {
            let mut result = format.to_string();
            let mut arg_index = 0;
            
            while let Some(pos) = result.find("{}") {
                if arg_index < args.len() {
                    let replacement = format!("{}", args[arg_index]);
                    result.replace_range(pos..pos + 2, &replacement);
                    arg_index += 1;
                } else {
                    break;
                }
            }
            
            eprintln!("[DEBUG] {}", result);
        }
    }

    /// Output a verbose debug message
    ///
    /// Only outputs if both debug and verbose modes are enabled.
    ///
    /// # Arguments
    ///
    /// * `message` - The verbose debug message
    pub fn verbose_output(message: &str) {
        if Self::is_enabled() && Self::is_verbose() {
            eprintln!("[VERBOSE] {}", message);
        }
    }

    /// Display a term (print term display)
    ///
    /// This function displays a term in readable format, similar to `ptd()` in the C code.
    /// It uses the print_term functionality from infrastructure_data_handling.
    ///
    /// # Arguments
    ///
    /// * `term` - The term to display
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Success
    /// * `Err(DebugError)` - Display error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use infrastructure_debugging::DebugUtils;
    /// use entities_data_handling::term_hashing::Term;
    ///
    /// let term = Term::Small(42);
    /// DebugUtils::display_term(&term).unwrap();
    /// ```
    pub fn display_term(term: &Term) -> Result<(), DebugError> {
        if Self::is_enabled() {
            print_term(term).map_err(|e| DebugError::PrintError(format!("{:?}", e)))?;
            eprintln!(); // Newline after term
        }
        Ok(())
    }

    /// Display a term as a string (safe version)
    ///
    /// This function converts a term to a string representation without printing.
    /// Similar to `s_print_term` but with error handling.
    ///
    /// # Arguments
    ///
    /// * `term` - The term to convert
    ///
    /// # Returns
    ///
    /// * `Ok(string)` - String representation of the term
    /// * `Err(DebugError)` - Conversion error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use infrastructure_debugging::DebugUtils;
    /// use entities_data_handling::term_hashing::Term;
    ///
    /// let term = Term::Small(42);
    /// let s = DebugUtils::term_to_string(&term).unwrap();
    /// assert_eq!(s, "42");
    /// ```
    pub fn term_to_string(term: &Term) -> Result<String, DebugError> {
        s_print_term(term).map_err(|e| DebugError::PrintError(format!("{:?}", e)))
    }

    /// Paranoid display of a term
    ///
    /// This function attempts to display a term even if there are errors in the
    /// data structures. Similar to `paranoid_display()` in the C code.
    ///
    /// # Arguments
    ///
    /// * `term` - The term to display (may be corrupted)
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Success (or safely handled error)
    /// * `Err(DebugError)` - Critical error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use infrastructure_debugging::DebugUtils;
    /// use entities_data_handling::term_hashing::Term;
    ///
    /// let term = Term::Small(42);
    /// // Even if term is corrupted, this should handle it gracefully
    /// let _ = DebugUtils::paranoid_display(&term);
    /// ```
    pub fn paranoid_display(term: &Term) -> Result<(), DebugError> {
        // Try to display, but catch any panics or errors
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            Self::display_term(term)
        })) {
            Ok(result) => result,
            Err(_) => {
                // If display fails, output a safe message
                if Self::is_enabled() {
                    eprintln!("[DEBUG] <corrupted term - cannot display>");
                }
                Ok(())
            }
        }
    }
}

/// Debug operation errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DebugError {
    /// Print/display error
    PrintError(String),
    /// Invalid term structure
    InvalidTerm(String),
    /// Debug operation failed
    OperationFailed(String),
}

impl std::fmt::Display for DebugError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DebugError::PrintError(msg) => write!(f, "Print error: {}", msg),
            DebugError::InvalidTerm(msg) => write!(f, "Invalid term: {}", msg),
            DebugError::OperationFailed(msg) => write!(f, "Operation failed: {}", msg),
        }
    }
}

impl std::error::Error for DebugError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_enable_disable() {
        // Reset state
        DebugUtils::disable();
        assert!(!DebugUtils::is_enabled());
        
        DebugUtils::enable();
        assert!(DebugUtils::is_enabled());
        
        DebugUtils::disable();
        assert!(!DebugUtils::is_enabled());
    }

    #[test]
    fn test_verbose_enable_disable() {
        // Reset state
        DebugUtils::disable_verbose();
        assert!(!DebugUtils::is_verbose());
        
        DebugUtils::enable_verbose();
        assert!(DebugUtils::is_verbose());
        
        DebugUtils::disable_verbose();
        assert!(!DebugUtils::is_verbose());
    }

    #[test]
    fn test_debug_output() {
        DebugUtils::enable();
        // Should not panic
        DebugUtils::debug_output("test message");
        
        DebugUtils::disable();
        // Should not output but also not panic
        DebugUtils::debug_output("suppressed message");
    }

    #[test]
    fn test_debug_format() {
        DebugUtils::enable();
        DebugUtils::debug_format("Value: {}", &[&42]);
        DebugUtils::debug_format("Name: {}, Age: {}", &[&"Alice", &30]);
        
        DebugUtils::disable();
        DebugUtils::debug_format("Suppressed: {}", &[&"test"]);
    }

    #[test]
    fn test_verbose_output() {
        DebugUtils::disable();
        DebugUtils::disable_verbose();
        DebugUtils::verbose_output("should not appear");
        
        DebugUtils::enable();
        DebugUtils::verbose_output("should not appear (verbose disabled)");
        
        DebugUtils::enable_verbose();
        DebugUtils::verbose_output("should appear");
    }

    #[test]
    fn test_display_term() {
        DebugUtils::enable();
        
        let term = Term::Small(42);
        let result = DebugUtils::display_term(&term);
        assert!(result.is_ok());
        
        let term2 = Term::Atom(123);
        let result2 = DebugUtils::display_term(&term2);
        assert!(result2.is_ok());
    }

    #[test]
    fn test_term_to_string() {
        let term = Term::Small(42);
        let result = DebugUtils::term_to_string(&term);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "42");
        
        let term2 = Term::Atom(123);
        let result2 = DebugUtils::term_to_string(&term2);
        assert!(result2.is_ok());
        let s = result2.unwrap();
        assert!(s.starts_with("atom_"));
    }

    #[test]
    fn test_term_to_string_list() {
        let term = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::List {
                head: Box::new(Term::Small(2)),
                tail: Box::new(Term::Nil),
            }),
        };
        let result = DebugUtils::term_to_string(&term);
        assert!(result.is_ok());
        let s = result.unwrap();
        assert!(s.contains("1"));
        assert!(s.contains("2"));
    }

    #[test]
    fn test_term_to_string_tuple() {
        let term = Term::Tuple(vec![
            Term::Small(1),
            Term::Atom(2),
            Term::Small(3),
        ]);
        let result = DebugUtils::term_to_string(&term);
        assert!(result.is_ok());
        let s = result.unwrap();
        assert!(s.starts_with("{"));
        assert!(s.ends_with("}"));
        assert!(s.contains("1"));
    }

    #[test]
    fn test_paranoid_display() {
        DebugUtils::enable();
        
        let term = Term::Small(42);
        let result = DebugUtils::paranoid_display(&term);
        assert!(result.is_ok());
        
        // Test with various term types
        let terms = vec![
            Term::Small(0),
            Term::Atom(0),
            Term::Float(3.14),
            Term::Nil,
            Term::Tuple(vec![Term::Small(1)]),
        ];
        
        for term in terms {
            let result = DebugUtils::paranoid_display(&term);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_debug_error_display() {
        let error1 = DebugError::PrintError("test error".to_string());
        let error2 = DebugError::InvalidTerm("bad term".to_string());
        let error3 = DebugError::OperationFailed("failed".to_string());
        
        let str1 = format!("{}", error1);
        let str2 = format!("{}", error2);
        let str3 = format!("{}", error3);
        
        assert!(str1.contains("Print error"));
        assert!(str1.contains("test error"));
        assert!(str2.contains("Invalid term"));
        assert!(str2.contains("bad term"));
        assert!(str3.contains("Operation failed"));
        assert!(str3.contains("failed"));
    }

    #[test]
    fn test_debug_error_clone() {
        let error1 = DebugError::PrintError("test".to_string());
        let error2 = DebugError::InvalidTerm("bad".to_string());
        
        let cloned1 = error1.clone();
        let cloned2 = error2.clone();
        
        assert_eq!(error1, cloned1);
        assert_eq!(error2, cloned2);
    }

    #[test]
    fn test_debug_error_partial_eq() {
        let error1 = DebugError::PrintError("test".to_string());
        let error2 = DebugError::PrintError("test".to_string());
        let error3 = DebugError::PrintError("different".to_string());
        let error4 = DebugError::InvalidTerm("test".to_string());
        
        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
        assert_ne!(error1, error4);
    }

    #[test]
    fn test_debug_error_eq() {
        let error1 = DebugError::PrintError("test".to_string());
        let error2 = DebugError::PrintError("test".to_string());
        let error3 = DebugError::InvalidTerm("test".to_string());
        
        assert!(error1 == error2);
        assert!(error1 != error3);
    }

    #[test]
    fn test_debug_error_debug() {
        let error1 = DebugError::PrintError("test".to_string());
        let error2 = DebugError::InvalidTerm("bad".to_string());
        let error3 = DebugError::OperationFailed("failed".to_string());
        
        let debug_str1 = format!("{:?}", error1);
        let debug_str2 = format!("{:?}", error2);
        let debug_str3 = format!("{:?}", error3);
        
        assert!(debug_str1.contains("PrintError"));
        assert!(debug_str2.contains("InvalidTerm"));
        assert!(debug_str3.contains("OperationFailed"));
    }

    #[test]
    fn test_debug_error_error_trait() {
        let error = DebugError::PrintError("test error".to_string());
        // Test that Error trait is implemented
        let error_ref: &dyn std::error::Error = &error;
        let description = error_ref.to_string();
        assert!(description.contains("Print error"));
        assert!(description.contains("test error"));
    }

    #[test]
    fn test_debug_format_no_placeholders() {
        DebugUtils::enable();
        // Format string with no placeholders
        DebugUtils::debug_format("Simple message", &[]);
        DebugUtils::debug_format("Another message", &[&42]); // Extra args ignored
    }

    #[test]
    fn test_debug_format_more_placeholders_than_args() {
        DebugUtils::enable();
        // More placeholders than args - should stop when args run out
        DebugUtils::debug_format("Value: {} and {} and {}", &[&42]);
    }

    #[test]
    fn test_debug_format_multiple_placeholders() {
        DebugUtils::enable();
        DebugUtils::debug_format("A: {}, B: {}, C: {}", &[&1, &2, &3]);
        DebugUtils::debug_format("X: {}, Y: {}", &[&"hello", &"world"]);
    }

    #[test]
    fn test_debug_format_empty_args() {
        DebugUtils::enable();
        DebugUtils::debug_format("Message with {} placeholder", &[]);
    }

    #[test]
    fn test_display_term_disabled() {
        DebugUtils::disable();
        // Should succeed but not output anything
        let term = Term::Small(42);
        let result = DebugUtils::display_term(&term);
        assert!(result.is_ok());
    }

    #[test]
    fn test_display_term_various_types() {
        DebugUtils::enable();
        
        let terms = vec![
            Term::Small(0),
            Term::Small(42),
            Term::Small(-100),
            Term::Atom(0),
            Term::Atom(123),
            Term::Float(3.14),
            Term::Float(-2.5),
            Term::Nil,
            Term::Binary {
                data: vec![1, 2, 3],
                bit_offset: 0,
                bit_size: 24,
            },
        ];
        
        for term in terms {
            let result = DebugUtils::display_term(&term);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_term_to_string_various_types() {
        let terms = vec![
            Term::Small(0),
            Term::Small(42),
            Term::Small(-100),
            Term::Atom(0),
            Term::Atom(123),
            Term::Float(3.14),
            Term::Float(-2.5),
            Term::Nil,
        ];
        
        for term in terms {
            let result = DebugUtils::term_to_string(&term);
            assert!(result.is_ok());
            let s = result.unwrap();
            assert!(!s.is_empty());
        }
    }

    #[test]
    fn test_paranoid_display_disabled() {
        DebugUtils::disable();
        // Should succeed even when disabled
        let term = Term::Small(42);
        let result = DebugUtils::paranoid_display(&term);
        assert!(result.is_ok());
    }

    #[test]
    fn test_paranoid_display_complex_terms() {
        DebugUtils::enable();
        
        // Test with complex nested structures
        let term1 = Term::Tuple(vec![
            Term::Small(1),
            Term::List {
                head: Box::new(Term::Small(2)),
                tail: Box::new(Term::Nil),
            },
            Term::Atom(3),
        ]);
        
        let result1 = DebugUtils::paranoid_display(&term1);
        assert!(result1.is_ok());
        
        let term2 = Term::List {
            head: Box::new(Term::Tuple(vec![Term::Small(1)])),
            tail: Box::new(Term::List {
                head: Box::new(Term::Small(2)),
                tail: Box::new(Term::Nil),
            }),
        };
        
        let result2 = DebugUtils::paranoid_display(&term2);
        assert!(result2.is_ok());
    }

    #[test]
    fn test_debug_error_all_variants_eq() {
        let error1 = DebugError::PrintError("msg".to_string());
        let error2 = DebugError::PrintError("msg".to_string());
        let error3 = DebugError::InvalidTerm("msg".to_string());
        let error4 = DebugError::InvalidTerm("msg".to_string());
        let error5 = DebugError::OperationFailed("msg".to_string());
        let error6 = DebugError::OperationFailed("msg".to_string());
        
        assert_eq!(error1, error2);
        assert_eq!(error3, error4);
        assert_eq!(error5, error6);
        assert_ne!(error1, error3);
        assert_ne!(error1, error5);
        assert_ne!(error3, error5);
    }

    #[test]
    fn test_term_to_string_binary() {
        let term = Term::Binary {
            data: vec![1, 2, 3],
            bit_offset: 0,
            bit_size: 24,
        };
        let result = DebugUtils::term_to_string(&term);
        assert!(result.is_ok());
        let s = result.unwrap();
        assert!(s.starts_with("<<"));
        assert!(s.ends_with(">>"));
    }

    #[test]
    fn test_term_to_string_empty_binary() {
        let term = Term::Binary {
            data: vec![],
            bit_offset: 0,
            bit_size: 0,
        };
        let result = DebugUtils::term_to_string(&term);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "<<>>");
    }

    #[test]
    fn test_display_term_binary() {
        DebugUtils::enable();
        let term = Term::Binary {
            data: vec![1, 2, 3],
            bit_offset: 0,
            bit_size: 24,
        };
        let result = DebugUtils::display_term(&term);
        assert!(result.is_ok());
    }

    #[test]
    fn test_display_term_map() {
        DebugUtils::enable();
        let term = Term::Map(vec![
            (Term::Small(1), Term::Small(2)),
            (Term::Atom(3), Term::Atom(4)),
        ]);
        let result = DebugUtils::display_term(&term);
        assert!(result.is_ok());
    }

    #[test]
    fn test_display_term_ref() {
        DebugUtils::enable();
        let term = Term::Ref {
            node: 1,
            ids: vec![2, 3],
            creation: 4,
        };
        let result = DebugUtils::display_term(&term);
        assert!(result.is_ok());
    }

    #[test]
    fn test_display_term_fun() {
        DebugUtils::enable();
        let term = Term::Fun {
            is_local: true,
            module: 1,
            function: 2,
            arity: 3,
            old_uniq: None,
            env: vec![],
        };
        let result = DebugUtils::display_term(&term);
        assert!(result.is_ok());
    }

    #[test]
    fn test_display_term_big() {
        DebugUtils::enable();
        let term = Term::Big(BigNumber::from_u64(1234567890));
        let result = DebugUtils::display_term(&term);
        assert!(result.is_ok());
    }

    #[test]
    fn test_term_to_string_map() {
        let term = Term::Map(vec![
            (Term::Small(1), Term::Small(2)),
        ]);
        let result = DebugUtils::term_to_string(&term);
        assert!(result.is_ok());
        let s = result.unwrap();
        assert!(s.starts_with("#"));
    }

    #[test]
    fn test_term_to_string_ref() {
        let term = Term::Ref {
            node: 1,
            ids: vec![2],
            creation: 3,
        };
        let result = DebugUtils::term_to_string(&term);
        assert!(result.is_ok());
        let s = result.unwrap();
        assert!(s.starts_with("#Ref<"));
    }

    #[test]
    fn test_term_to_string_fun() {
        let term = Term::Fun {
            is_local: true,
            module: 1,
            function: 2,
            arity: 3,
            old_uniq: None,
            env: vec![],
        };
        let result = DebugUtils::term_to_string(&term);
        assert!(result.is_ok());
        let s = result.unwrap();
        assert!(s.contains("fun"));
    }

    #[test]
    fn test_term_to_string_big() {
        let term = Term::Big(BigNumber::from_u64(1234567890));
        let result = DebugUtils::term_to_string(&term);
        assert!(result.is_ok());
        let s = result.unwrap();
        assert_eq!(s, "<bignum>");
    }

    #[test]
    fn test_paranoid_display_map() {
        DebugUtils::enable();
        let term = Term::Map(vec![
            (Term::Small(1), Term::Small(2)),
        ]);
        let result = DebugUtils::paranoid_display(&term);
        assert!(result.is_ok());
    }

    #[test]
    fn test_paranoid_display_ref() {
        DebugUtils::enable();
        let term = Term::Ref {
            node: 1,
            ids: vec![2, 3],
            creation: 4,
        };
        let result = DebugUtils::paranoid_display(&term);
        assert!(result.is_ok());
    }

    #[test]
    fn test_paranoid_display_fun() {
        DebugUtils::enable();
        let term = Term::Fun {
            is_local: false,
            module: 1,
            function: 2,
            arity: 3,
            old_uniq: None,
            env: vec![],
        };
        let result = DebugUtils::paranoid_display(&term);
        assert!(result.is_ok());
    }

    #[test]
    fn test_paranoid_display_big() {
        DebugUtils::enable();
        let term = Term::Big(BigNumber::from_u64(0).minus(&BigNumber::from_u64(1234567890)));
        let result = DebugUtils::paranoid_display(&term);
        assert!(result.is_ok());
    }

    #[test]
    fn test_debug_format_consecutive_placeholders() {
        DebugUtils::enable();
        // Test consecutive placeholders
        DebugUtils::debug_format("{}{}", &[&1, &2]);
    }

    #[test]
    fn test_debug_format_escaped_braces() {
        DebugUtils::enable();
        // Test format string with literal braces (not placeholders)
        // Note: current implementation only handles {}, not {{ or }}
        DebugUtils::debug_format("Value: {}", &[&42]);
    }

    #[test]
    fn test_debug_format_numeric_types() {
        DebugUtils::enable();
        DebugUtils::debug_format("i32: {}, i64: {}, u32: {}, u64: {}", &[&1i32, &2i64, &3u32, &4u64]);
    }

    #[test]
    fn test_verbose_output_disabled_debug() {
        // Test verbose_output when debug is disabled (even if verbose is enabled)
        DebugUtils::disable();
        DebugUtils::enable_verbose();
        DebugUtils::verbose_output("should not appear");
    }

    #[test]
    fn test_verbose_output_disabled_verbose() {
        // Test verbose_output when verbose is disabled (even if debug is enabled)
        DebugUtils::enable();
        DebugUtils::disable_verbose();
        DebugUtils::verbose_output("should not appear");
    }
}

