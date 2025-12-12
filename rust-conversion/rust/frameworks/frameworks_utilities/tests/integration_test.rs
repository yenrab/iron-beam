//! Integration tests for frameworks_utilities crate
//!
//! These tests verify that framework utility functions work correctly.

use frameworks_utilities::*;

#[test]
fn test_framework_utils_operations() {
    // Test that FrameworkUtils can be used
    let _utils = FrameworkUtils;
    // Should not panic
}

#[test]
fn test_framework_error_variants() {
    // Test FrameworkError enum variants
    let error = FrameworkError::Failed;
    let _ = format!("{:?}", error);
    // Note: FrameworkError doesn't implement Display, only Debug
}

#[test]
fn test_framework_error_debug() {
    let error = FrameworkError::Failed;
    let str = format!("{:?}", error);
    assert!(!str.is_empty());
}

#[test]
fn test_framework_error_clone_eq() {
    let error1 = FrameworkError::Failed;
    let error2 = FrameworkError::Failed;
    
    assert_eq!(error1, error2);
    
    // FrameworkError is Copy, not Clone
    let copied = error1;
    assert_eq!(error1, copied);
}

#[test]
fn test_framework_utils_set_get_env() {
    let test_key = "FRAMEWORK_TEST_VAR";
    let test_value = "test_value_123";
    
    // Set the environment variable
    FrameworkUtils::set_env(test_key, test_value);
    
    // Get it back
    let retrieved = FrameworkUtils::get_env(test_key);
    assert_eq!(retrieved, Some(test_value.to_string()));
    
    // Clean up
    std::env::remove_var(test_key);
}

#[test]
fn test_framework_utils_get_env_nonexistent() {
    let result = FrameworkUtils::get_env("FRAMEWORK_NONEXISTENT_VAR_XYZ");
    assert_eq!(result, None);
}

#[test]
fn test_framework_utils_quote_arguments() {
    let args = vec!["hello", "world with spaces", "normal_arg"];
    let quoted = FrameworkUtils::quote_arguments(&args);
    
    assert_eq!(quoted[0], "hello");
    assert_eq!(quoted[1], "\"world with spaces\"");
    assert_eq!(quoted[2], "normal_arg");
}

#[test]
fn test_framework_utils_quote_arguments_with_quotes() {
    let args = vec!["arg with \"quotes\""];
    let quoted = FrameworkUtils::quote_arguments(&args);
    
    assert_eq!(quoted[0], "\"arg with \\\"quotes\\\"\"");
}

#[test]
fn test_framework_utils_has_console() {
    let has_console = FrameworkUtils::has_console();
    // Just verify it doesn't panic
    assert!(has_console || !has_console);
}

#[test]
fn test_framework_utils_utility() {
    let result = FrameworkUtils::utility();
    assert!(result.is_ok());
}
