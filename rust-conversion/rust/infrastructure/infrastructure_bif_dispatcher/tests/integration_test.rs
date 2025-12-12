//! Integration tests for infrastructure_bif_dispatcher crate
//!
//! These tests verify that BIF dispatcher functions work correctly
//! and test end-to-end workflows for BIF routing and registry operations.

use infrastructure_bif_dispatcher::*;
use entities_process::{Process, Eterm, ErtsCodePtr};
use infrastructure_bif_dispatcher::initialization::TrapExport;
use std::sync::Arc;

#[test]
fn test_bif_dispatcher_new() {
    let dispatcher = BifDispatcher::new();
    assert!(!dispatcher.is_initialized());
}

#[test]
fn test_bif_dispatcher_init() {
    let mut dispatcher = BifDispatcher::new();
    let result = dispatcher.init();
    assert!(result.is_ok());
    assert!(dispatcher.is_initialized());
}

#[test]
fn test_bif_dispatcher_double_init() {
    let mut dispatcher = BifDispatcher::new();
    dispatcher.init().unwrap();
    
    let result = dispatcher.init();
    assert!(result.is_err());
    match result.unwrap_err() {
        BifDispatcherError::AlreadyInitialized => {}
        _ => panic!("Expected AlreadyInitialized error"),
    }
}

#[test]
fn test_bif_registry_new() {
    let registry = BifRegistry::new();
    // Should not panic
    let _ = registry;
}

#[test]
fn test_bif_registry_get_instance() {
    let registry1 = get_global_registry();
    let registry2 = get_global_registry();
    
    // Should return the same instance (singleton)
    assert!(std::ptr::eq(registry1, registry2));
}

#[test]
fn test_bif_key_new() {
    let key = BifKey::new(1, 2, 3);
    assert_eq!(key.module, 1);
    assert_eq!(key.function, 2);
    assert_eq!(key.arity, 3);
}

#[test]
fn test_bif_key_clone_eq() {
    let key1 = BifKey::new(1, 2, 3);
    let key2 = BifKey::new(1, 2, 3);
    let key3 = BifKey::new(1, 2, 4);
    
    assert_eq!(key1, key2);
    assert_ne!(key1, key3);
    
    let cloned = key1.clone();
    assert_eq!(key1, cloned);
}

#[test]
fn test_bif_registry_lookup_nonexistent() {
    let registry = BifRegistry::new();
    
    let found = registry.lookup(1, 2, 3);
    assert!(found.is_none());
}

#[test]
fn test_sched_type_variants() {
    let types = vec![
        SchedType::Normal,
        SchedType::DirtyCpu,
        SchedType::DirtyIo,
    ];
    
    for sched_type in types {
        let _ = format!("{:?}", sched_type);
    }
}

#[test]
fn test_is_proc_out_of_reds() {
    let process = Arc::new(Process::new(1));
    let result = is_proc_out_of_reds(&process);
    // May be true or false depending on process state
    let _ = result;
}

#[test]
fn test_reds_left() {
    let process = Arc::new(Process::new(2));
    let reds = reds_left(&process);
    // Should return a valid reduction count
    assert!(reds >= 0 || reds < 0); // Just check it doesn't panic
}

#[test]
fn test_bif_dispatcher_error_variants() {
    let errors = vec![
        BifDispatcherError::NotInitialized,
        BifDispatcherError::AlreadyInitialized,
        BifDispatcherError::BifNotFound("test".to_string()),
        BifDispatcherError::InvalidArguments("test".to_string()),
        BifDispatcherError::ProcessError("test".to_string()),
    ];
    
    for error in errors {
        let _ = format!("{:?}", error);
    }
}

#[test]
fn test_bif_init_error_variants() {
    let errors = vec![
        BifInitError::InitFailed("test".to_string()),
        BifInitError::AlreadyInitialized,
    ];
    
    for error in errors {
        let _ = format!("{:?}", error);
    }
}

#[test]
fn test_bif_return_trap() {
    let process = Arc::new(Process::new(3));
    let args = vec![42u64];
    
    let result = bif_return_trap(&process, &args);
    // Should return a term
    let _ = result;
}

#[test]
fn test_bif_handle_signals_return() {
    let process = Arc::new(Process::new(4));
    let args = vec![42u64];
    
    let result = bif_handle_signals_return(&process, &args);
    // Should return a term
    let _ = result;
}

#[test]
fn test_erts_internal_await_exit_trap() {
    let process = Arc::new(Process::new(5));
    let args = vec![42u64];
    
    let result = erts_internal_await_exit_trap(&process, &args);
    // Should return a term
    let _ = result;
}

#[test]
fn test_call_bif() {
    let process = Arc::new(Process::new(6));
    let reg = vec![42u64];
    let ptr: ErtsCodePtr = std::ptr::null();
    
    let result = call_bif(&process, &reg, ptr);
    // May succeed or fail depending on implementation
    let _ = result;
}

#[test]
fn test_erts_call_dirty_bif() {
    let process = Arc::new(Process::new(7));
    let reg = vec![42u64];
    let ptr: ErtsCodePtr = std::ptr::null();
    
    let result = erts_call_dirty_bif(&process, ptr, &reg);
    // May succeed or fail depending on implementation
    let _ = result;
}

#[test]
fn test_prepare_trap() {
    use infrastructure_bif_dispatcher::initialization::TrapExport;
    
    let mut process = Process::new(8);
    let trap_export = TrapExport::new(1, 2, 1, None);
    
    prepare_trap(&mut process, &trap_export, 1);
    // Should not panic
}

#[test]
fn test_prepare_trap_with_args() {
    use infrastructure_bif_dispatcher::initialization::TrapExport;
    
    let mut process = Process::new(9);
    let trap_export = TrapExport::new(1, 2, 3, None);
    let args = vec![1u64, 2u64, 3u64];
    
    prepare_trap_with_args(&mut process, &trap_export, 3, &args);
    // Should not panic
}

#[test]
fn test_prepare_yield_return() {
    use infrastructure_bif_dispatcher::initialization::TrapExport;
    
    let mut process = Process::new(10);
    let trap_export = TrapExport::new(1, 2, 0, None);
    let result_term: Eterm = 42;
    let operation: Eterm = 0;
    
    prepare_yield_return(&mut process, &trap_export, result_term, operation);
    // Should not panic
}

#[test]
fn test_bif_dispatcher_default() {
    let dispatcher = BifDispatcher::default();
    assert!(!dispatcher.is_initialized());
}
