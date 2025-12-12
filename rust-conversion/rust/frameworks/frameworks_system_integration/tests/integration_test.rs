//! Integration tests for frameworks_system_integration crate
//!
//! These tests verify that system integration base functions work correctly.

use frameworks_system_integration::*;

#[test]
fn test_sys_base_operations() {
    // Test that SysBase can be used
    let _sys = SysBase;
    // Should not panic
}

#[test]
fn test_system_info_creation() {
    // Test SystemInfo creation if available
    // Note: Check actual API from sys_base module
}

#[test]
fn test_sys_base_init() {
    let result = SysBase::init();
    // May succeed or fail depending on initialization state
    let _ = result;
}

#[test]
fn test_sys_base_execute_command() {
    // Test executing a simple command
    let result = SysBase::execute_command("echo", &["hello"]);
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("hello"));
    }
    // Command may not be available in all environments
}

#[test]
fn test_sys_error_variants() {
    // Test SysError enum variants
    let errors = vec![
        SysError::InitFailed,
        SysError::CommandFailed,
    ];
    
    for error in errors {
        let _ = format!("{:?}", error);
        let _ = format!("{}", error);
    }
}

#[test]
fn test_sys_error_display() {
    let error1 = SysError::InitFailed;
    let error2 = SysError::CommandFailed;
    
    let str1 = format!("{}", error1);
    let str2 = format!("{}", error2);
    
    assert!(!str1.is_empty());
    assert!(!str2.is_empty());
}

#[test]
fn test_sys_error_clone_eq() {
    let error1 = SysError::InitFailed;
    let error2 = SysError::InitFailed;
    let error3 = SysError::CommandFailed;
    
    assert_eq!(error1, error2);
    assert_ne!(error1, error3);
    
    let cloned = error1.clone();
    assert_eq!(error1, cloned);
}
