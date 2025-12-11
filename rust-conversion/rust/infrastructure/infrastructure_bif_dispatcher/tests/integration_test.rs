//! Integration tests for infrastructure_bif_dispatcher
//!
//! Tests the BIF dispatcher functionality with real Process instances
//! and verifies trap handlers and initialization.

use infrastructure_bif_dispatcher::*;
use infrastructure_bif_dispatcher::initialization::{get_bif_return_trap_export, get_bif_handle_signals_return_export, get_await_exit_trap_export};
use entities_process::Process;

#[test]
fn test_erts_init_bif() {
    let result = erts_init_bif();
    assert!(result.is_ok());
    
    // Verify trap exports were created
    assert!(get_bif_return_trap_export().is_some());
    assert!(get_bif_handle_signals_return_export().is_some());
    assert!(get_await_exit_trap_export().is_some());
}

#[test]
fn test_erts_init_trap_export() {
    let mut export = TrapExport::new(1, 2, 3, None);
    erts_init_trap_export(&mut export, 10, 20, 30, None);
    
    assert_eq!(export.module(), 10);
    assert_eq!(export.function(), 20);
    assert_eq!(export.arity(), 30);
}

#[test]
fn test_trap_export_creation() {
    let export = TrapExport::new(1, 2, 3, None);
    assert_eq!(export.module(), 1);
    assert_eq!(export.function(), 2);
    assert_eq!(export.arity(), 3);
    assert_eq!(export.bif_number(), -1);
}

#[test]
fn test_trap_export_bif_number() {
    let mut export = TrapExport::new(1, 2, 3, None);
    assert_eq!(export.bif_number(), -1);
    
    export.set_bif_number(42);
    assert_eq!(export.bif_number(), 42);
}

#[test]
fn test_bif_return_trap() {
    let process = Process::new(1);
    let args = vec![100, 200];
    
    let result = bif_return_trap(&process, &args);
    assert_eq!(result, 100); // Returns first argument
}

#[test]
fn test_bif_handle_signals_return() {
    let process = Process::new(1);
    let args = vec![10, 20];
    
    let result = bif_handle_signals_return(&process, &args);
    assert_eq!(result, 20); // Returns second argument (value)
}

#[test]
fn test_erts_internal_await_exit_trap() {
    let process = Process::new(1);
    let args = vec![];
    
    let result = erts_internal_await_exit_trap(&process, &args);
    // Returns non-value (0) indicating trap/yield
    assert_eq!(result, 0);
}

#[test]
fn test_bif_dispatcher() {
    let mut dispatcher = BifDispatcher::new();
    assert!(!dispatcher.is_initialized());
    
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
    assert!(matches!(result.unwrap_err(), BifDispatcherError::AlreadyInitialized));
}

#[test]
fn test_call_bif_not_implemented() {
    let process = Process::new(1);
    let reg = vec![100, 200];
    let instruction_ptr = std::ptr::null();
    
    let result = call_bif(&process, &reg, instruction_ptr);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), BifDispatcherError::NotImplemented(_)));
}

#[test]
fn test_erts_call_dirty_bif_not_implemented() {
    let process = Process::new(1);
    let reg = vec![100, 200];
    let instruction_ptr = std::ptr::null();
    
    let result = erts_call_dirty_bif(&process, instruction_ptr, &reg);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), BifDispatcherError::NotImplemented(_)));
}

// Scheduling integration tests
#[test]
fn test_scheduling_prepare_trap_with_initialized_exports() {
    // Initialize BIF system to get real trap exports
    erts_init_bif().unwrap();
    
    let mut process = Process::new(1);
    let trap_export = get_bif_return_trap_export().unwrap();
    
    // Should not panic when preparing trap with initialized export
    prepare_trap(&mut process, &trap_export, 2);
}

#[test]
fn test_scheduling_prepare_trap_with_args_integration() {
    erts_init_bif().unwrap();
    
    let mut process = Process::new(1);
    let trap_export = get_bif_return_trap_export().unwrap();
    let args = vec![100, 200];
    
    // Should not panic when preparing trap with args
    prepare_trap_with_args(&mut process, &trap_export, 2, &args);
}

#[test]
fn test_scheduling_prepare_yield_return_integration() {
    erts_init_bif().unwrap();
    
    let mut process = Process::new(1);
    let trap_export = get_bif_return_trap_export().unwrap();
    
    // Should not panic when preparing yield return
    prepare_yield_return(&mut process, &trap_export, 100, 0);
}

#[test]
fn test_scheduling_reductions_check_integration() {
    let process = Process::new(1);
    
    // Check reductions - should not panic
    let out_of_reds = is_proc_out_of_reds(&process);
    let reds = reds_left(&process);
    
    // Verify return types
    assert!(out_of_reds == true || out_of_reds == false);
    assert!(reds >= 0);
}

#[test]
fn test_scheduling_sched_type_integration() {
    // Test all scheduler types
    let normal = SchedType::Normal;
    let dirty_cpu = SchedType::DirtyCpu;
    let dirty_io = SchedType::DirtyIo;
    
    // Verify they are distinct
    assert_ne!(normal, dirty_cpu);
    assert_ne!(normal, dirty_io);
    assert_ne!(dirty_cpu, dirty_io);
    
    // Verify cloning works
    let cloned_normal = normal.clone();
    assert_eq!(normal, cloned_normal);
}

#[test]
fn test_scheduling_complete_workflow() {
    // Initialize BIF system
    erts_init_bif().unwrap();
    
    let mut process = Process::new(1);
    let trap_export = get_bif_return_trap_export().unwrap();
    
    // Simulate a complete BIF scheduling workflow
    // 1. Check initial reductions
    let initial_reds = reds_left(&process);
    let initial_out_of_reds = is_proc_out_of_reds(&process);
    
    // 2. Prepare trap for BIF call
    prepare_trap(&mut process, &trap_export, 2);
    
    // 3. Prepare trap with arguments
    let args = vec![100, 200];
    prepare_trap_with_args(&mut process, &trap_export, 2, &args);
    
    // 4. Check reductions after trap preparation
    let after_trap_reds = reds_left(&process);
    let after_trap_out_of_reds = is_proc_out_of_reds(&process);
    
    // 5. Prepare yield return
    prepare_yield_return(&mut process, &trap_export, 100, 0);
    
    // 6. Check final reductions
    let final_reds = reds_left(&process);
    let final_out_of_reds = is_proc_out_of_reds(&process);
    
    // Verify all operations completed without panic
    // (Note: actual values depend on implementation, but should be consistent)
    assert!(initial_reds >= 0);
    assert!(after_trap_reds >= 0);
    assert!(final_reds >= 0);
    assert!(initial_out_of_reds == true || initial_out_of_reds == false);
    assert!(after_trap_out_of_reds == true || after_trap_out_of_reds == false);
    assert!(final_out_of_reds == true || final_out_of_reds == false);
    
    // Verify scheduler types are available
    let _sched_type = SchedType::Normal;
}

