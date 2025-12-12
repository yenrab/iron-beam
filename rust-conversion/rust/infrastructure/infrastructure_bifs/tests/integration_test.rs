//! Integration tests for infrastructure_bifs crate
//!
//! These tests verify that BIF infrastructure works correctly
//! and test end-to-end workflows for BIF system initialization and state management.

use infrastructure_bifs::{BifInfrastructure, BifState, BifError};

#[test]
fn test_bif_infrastructure_init_integration() {
    // Note: Can't reset in integration tests, so we test the current state
    // If already initialized, second init should fail
    let result = BifInfrastructure::init();
    
    if result.is_ok() {
        // First initialization succeeded
        assert!(BifInfrastructure::is_initialized());
        let state = result.unwrap();
        assert!(state.is_initialized());
    } else {
        // Already initialized, which is also valid
        assert!(BifInfrastructure::is_initialized());
    }
}

#[test]
fn test_bif_infrastructure_double_init() {
    // Try to initialize (may already be initialized)
    let result1 = BifInfrastructure::init();
    
    // Second init should always fail with AlreadyInitialized
    let result2 = BifInfrastructure::init();
    assert!(result2.is_err());
    
    match result2.unwrap_err() {
        BifError::AlreadyInitialized => {}
        _ => panic!("Expected AlreadyInitialized error"),
    }
}

#[test]
fn test_bif_state_properties() {
    // Try to initialize (may already be initialized)
    let result = BifInfrastructure::init();
    
    if let Ok(state) = result {
        // Check state properties
        assert!(state.is_initialized());
        assert!(!state.is_sched_wall_time_enabled());
        assert!(!state.is_msacc_enabled());
    } else {
        // Already initialized, which is fine
        assert!(BifInfrastructure::is_initialized());
    }
}

#[test]
fn test_bif_error_variants() {
    // Test all error variants
    let errors = vec![
        BifError::InitFailed("test".to_string()),
        BifError::AlreadyInitialized,
        BifError::BifNotFound("test_bif".to_string()),
        BifError::BadArgument("invalid".to_string()),
        BifError::SystemLimit("too many".to_string()),
    ];
    
    for error in errors {
        let _ = format!("{}", error);
        let _ = format!("{:?}", error);
    }
}

#[test]
fn test_bif_error_display() {
    let error1 = BifError::InitFailed("test error".to_string());
    let error2 = BifError::AlreadyInitialized;
    let error3 = BifError::BifNotFound("my_bif".to_string());
    
    let str1 = format!("{}", error1);
    let str2 = format!("{}", error2);
    let str3 = format!("{}", error3);
    
    assert!(str1.contains("BIF initialization failed"));
    assert!(str1.contains("test error"));
    assert!(str2.contains("already initialized"));
    assert!(str3.contains("BIF not found"));
    assert!(str3.contains("my_bif"));
}

#[test]
fn test_bif_error_clone_eq() {
    let error1 = BifError::InitFailed("test".to_string());
    let error2 = BifError::InitFailed("test".to_string());
    let error3 = BifError::InitFailed("different".to_string());
    let error4 = BifError::AlreadyInitialized;
    
    assert_eq!(error1, error2);
    assert_ne!(error1, error3);
    assert_ne!(error1, error4);
    
    let cloned = error1.clone();
    assert_eq!(error1, cloned);
}

#[test]
fn test_bif_state_clone() {
    // Try to initialize (may already be initialized)
    let result = BifInfrastructure::init();
    
    if let Ok(state1) = result {
        let state2 = state1.clone();
        
        assert_eq!(state1.is_initialized(), state2.is_initialized());
        assert_eq!(state1.is_sched_wall_time_enabled(), state2.is_sched_wall_time_enabled());
        assert_eq!(state1.is_msacc_enabled(), state2.is_msacc_enabled());
    }
    // If already initialized, that's fine - we can't test clone without state
}

#[test]
fn test_bif_infrastructure_initialization_state() {
    // Test that we can check initialization state
    let is_init = BifInfrastructure::is_initialized();
    
    // Try to initialize
    let result = BifInfrastructure::init();
    
    if is_init {
        // Already initialized, so init should fail
        assert!(result.is_err());
        match result.unwrap_err() {
            BifError::AlreadyInitialized => {}
            _ => panic!("Expected AlreadyInitialized error"),
        }
    } else {
        // Not initialized, so init should succeed
        assert!(result.is_ok());
        assert!(BifInfrastructure::is_initialized());
    }
}

#[test]
fn test_bif_error_error_trait() {
    let error = BifError::InitFailed("test".to_string());
    let error_ref: &dyn std::error::Error = &error;
    let description = error_ref.to_string();
    assert!(description.contains("BIF initialization failed"));
}
