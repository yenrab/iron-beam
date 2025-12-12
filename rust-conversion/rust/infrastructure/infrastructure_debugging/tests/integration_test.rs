//! Integration tests for infrastructure_debugging crate
//!
//! These tests verify that debugging utilities work correctly
//! and test end-to-end workflows for debug operations.

use infrastructure_debugging::{DebugUtils, DebugError};
use entities_data_handling::term_hashing::Term;

#[test]
fn test_debug_utils_enable_disable_integration() {
    // Reset state
    DebugUtils::disable();
    assert!(!DebugUtils::is_enabled());
    
    DebugUtils::enable();
    assert!(DebugUtils::is_enabled());
    
    DebugUtils::disable();
    assert!(!DebugUtils::is_enabled());
}

#[test]
fn test_debug_utils_verbose_integration() {
    // Reset state
    DebugUtils::disable_verbose();
    assert!(!DebugUtils::is_verbose());
    
    DebugUtils::enable_verbose();
    assert!(DebugUtils::is_verbose());
    
    DebugUtils::disable_verbose();
    assert!(!DebugUtils::is_verbose());
}

#[test]
fn test_debug_output_integration() {
    DebugUtils::enable();
    // Should not panic
    DebugUtils::debug_output("Integration test message");
    
    DebugUtils::disable();
    // Should not output but also not panic
    DebugUtils::debug_output("Suppressed message");
}

#[test]
fn test_debug_format_integration() {
    DebugUtils::enable();
    DebugUtils::debug_format("Value: {}", &[&42]);
    DebugUtils::debug_format("Name: {}, Age: {}", &[&"Alice", &30]);
    
    DebugUtils::disable();
    DebugUtils::debug_format("Suppressed: {}", &[&"test"]);
}

#[test]
fn test_verbose_output_integration() {
    DebugUtils::disable();
    DebugUtils::disable_verbose();
    DebugUtils::verbose_output("should not appear");
    
    DebugUtils::enable();
    DebugUtils::verbose_output("should not appear (verbose disabled)");
    
    DebugUtils::enable_verbose();
    DebugUtils::verbose_output("should appear");
}

#[test]
fn test_display_term_integration() {
    DebugUtils::enable();
    
    let term = Term::Small(42);
    let result = DebugUtils::display_term(&term);
    assert!(result.is_ok());
    
    let term2 = Term::Atom(123);
    let result2 = DebugUtils::display_term(&term2);
    assert!(result2.is_ok());
}

#[test]
fn test_term_to_string_integration() {
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
fn test_paranoid_display_integration() {
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
fn test_debug_error_integration() {
    let errors = vec![
        DebugError::PrintError("test error".to_string()),
        DebugError::InvalidTerm("bad term".to_string()),
        DebugError::OperationFailed("failed".to_string()),
    ];
    
    for error in errors {
        let _ = format!("{}", error);
        let _ = format!("{:?}", error);
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
fn test_debug_error_clone_eq() {
    let error1 = DebugError::PrintError("test".to_string());
    let error2 = DebugError::PrintError("test".to_string());
    let error3 = DebugError::PrintError("different".to_string());
    let error4 = DebugError::InvalidTerm("test".to_string());
    
    assert_eq!(error1, error2);
    assert_ne!(error1, error3);
    assert_ne!(error1, error4);
    
    let cloned = error1.clone();
    assert_eq!(error1, cloned);
}

#[test]
fn test_term_to_string_complex_terms() {
    let term1 = Term::List {
        head: Box::new(Term::Small(1)),
        tail: Box::new(Term::List {
            head: Box::new(Term::Small(2)),
            tail: Box::new(Term::Nil),
        }),
    };
    let result1 = DebugUtils::term_to_string(&term1);
    assert!(result1.is_ok());
    let s1 = result1.unwrap();
    assert!(s1.contains("1"));
    assert!(s1.contains("2"));
    
    let term2 = Term::Tuple(vec![
        Term::Small(1),
        Term::Atom(2),
        Term::Small(3),
    ]);
    let result2 = DebugUtils::term_to_string(&term2);
    assert!(result2.is_ok());
    let s2 = result2.unwrap();
    assert!(s2.starts_with("{"));
    assert!(s2.ends_with("}"));
    assert!(s2.contains("1"));
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
fn test_paranoid_display_complex_terms() {
    DebugUtils::enable();
    
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
fn test_debug_error_error_trait() {
    let error = DebugError::PrintError("test error".to_string());
    let error_ref: &dyn std::error::Error = &error;
    let description = error_ref.to_string();
    assert!(description.contains("Print error"));
    assert!(description.contains("test error"));
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
    ];
    
    for term in terms {
        let result = DebugUtils::display_term(&term);
        assert!(result.is_ok());
    }
}
