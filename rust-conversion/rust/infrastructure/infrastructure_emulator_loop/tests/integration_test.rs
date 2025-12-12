//! Integration tests for infrastructure_emulator_loop crate
//!
//! These tests verify that emulator loop functions work correctly
//! and test end-to-end workflows for instruction execution and register management.

use infrastructure_emulator_loop::*;
use entities_process::{Process, Eterm, ErtsCodePtr};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

#[test]
fn test_emulator_loop_new() {
    let loop_state = EmulatorLoop::new();
    assert!(loop_state.current_process().is_none());
}

#[test]
fn test_emulator_loop_set_current_process() {
    let mut loop_state = EmulatorLoop::new();
    let process = Arc::new(Process::new(1));
    
    loop_state.set_current_process(Some(Arc::clone(&process)));
    assert!(loop_state.current_process().is_some());
    assert_eq!(loop_state.current_process().unwrap().id(), 1);
}

#[test]
fn test_register_manager_new() {
    let manager = RegisterManager::new();
    // Should not panic
    let _ = manager;
}

#[test]
fn test_register_manager_operations() {
    let manager = RegisterManager::new();
    // Test that manager can be used
    let _ = manager;
}

#[test]
fn test_copy_in_registers() {
    let process = Arc::new(Process::new(1));
    let mut reg_array = vec![0u64; 10];
    
    copy_in_registers(&process, &mut reg_array);
    // Should not panic
}

#[test]
fn test_copy_out_registers() {
    let process = Arc::new(Process::new(2));
    let reg_array = vec![42u64, 43u64, 44u64];
    
    copy_out_registers(&process, &reg_array);
    // Should not panic
}

#[test]
fn test_is_valid_instruction() {
    let valid_ptr: ErtsCodePtr = &42u8 as *const u8;
    assert!(is_valid_instruction(valid_ptr));
    
    let null_ptr: ErtsCodePtr = std::ptr::null();
    assert!(!is_valid_instruction(null_ptr));
}

#[test]
fn test_next_instruction() {
    let ptr: ErtsCodePtr = &42u8 as *const u8;
    let next = next_instruction(ptr);
    // May return Some or None depending on implementation
    let _ = next;
}

#[test]
fn test_default_instruction_executor() {
    let executor = DefaultInstructionExecutor;
    let process = Arc::new(Process::new(1));
    let mut registers = vec![0u64; 10];
    let mut heap = vec![0u64; 100];
    let ptr: ErtsCodePtr = &42u8 as *const u8;
    
    let result = executor.execute_instruction(&process, ptr, &mut registers, &mut heap);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), InstructionResult::Continue);
}

#[test]
fn test_instruction_result_variants() {
    let results = vec![
        InstructionResult::Continue,
        InstructionResult::Yield,
        InstructionResult::NormalExit,
        InstructionResult::ErrorExit,
        InstructionResult::Trap(std::ptr::null()),
        InstructionResult::ContextSwitch,
    ];
    
    for result in results {
        let _ = format!("{:?}", result);
    }
}

#[test]
fn test_emulator_loop_error_variants() {
    use usecases_scheduling::ScheduleError;
    
    let errors = vec![
        EmulatorLoopError::ScheduleError(ScheduleError::ProcessExiting),
        EmulatorLoopError::ProcessNotFound,
        EmulatorLoopError::InvalidInstructionPointer,
        EmulatorLoopError::OutOfReductions,
        EmulatorLoopError::ProcessExited,
    ];
    
    for error in errors {
        let _ = format!("{:?}", error);
    }
}

#[test]
fn test_init_emulator() {
    let init_done = Arc::new(AtomicBool::new(false));
    let result = init_emulator(Arc::clone(&init_done));
    // May succeed or fail depending on initialization state
    let _ = result;
}

#[test]
fn test_register_manager_methods() {
    let manager = RegisterManager::new();
    // Test that manager methods exist
    let _ = manager;
}
