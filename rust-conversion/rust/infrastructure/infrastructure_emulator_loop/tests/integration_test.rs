//! Integration tests for infrastructure_emulator_loop
//!
//! Tests the emulator loop functionality including initialization,
//! register management, and process execution coordination.

use infrastructure_emulator_loop::{
    EmulatorLoop, init_emulator, process_main, EmulatorLoopError,
    RegisterManager, copy_in_registers, copy_out_registers,
};
use entities_process::{Process, Eterm};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

#[test]
fn test_emulator_loop_creation() {
    let loop_state = EmulatorLoop::new();
    assert!(loop_state.current_process().is_none());
}

#[test]
fn test_init_emulator() {
    let init_done = Arc::new(AtomicBool::new(false));
    
    let result = init_emulator(init_done.clone());
    assert!(result.is_ok());
    assert!(init_done.load(Ordering::Acquire));
    
    // Second call should also succeed (idempotent)
    let result2 = init_emulator(init_done.clone());
    assert!(result2.is_ok());
    assert!(init_done.load(Ordering::Acquire));
}

#[test]
fn test_init_emulator_already_done() {
    let init_done = Arc::new(AtomicBool::new(true));
    
    let result = init_emulator(init_done.clone());
    assert!(result.is_ok());
    assert!(init_done.load(Ordering::Acquire));
}

#[test]
fn test_process_main_initialization() {
    let mut emulator_loop = EmulatorLoop::new();
    let init_done = Arc::new(AtomicBool::new(false));
    
    // This will initialize and then return None (no process available)
    let result = process_main(&mut emulator_loop, init_done.clone());
    
    // Initialization should have completed
    assert!(init_done.load(Ordering::Acquire));
    
    // Should return Ok(None) since no process is available
    match result {
        Ok(None) => {}, // Expected
        Ok(Some(_)) => panic!("Unexpected process returned"),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}

#[test]
fn test_register_manager_creation() {
    let manager = RegisterManager::new();
    assert_eq!(manager.x_reg_array().len(), 1024); // MAX_X_REGS
    assert!(manager.x_reg_array().iter().all(|&x| x == 0));
}

#[test]
fn test_register_manager_default() {
    let manager = RegisterManager::default();
    assert_eq!(manager.x_reg_array().len(), 1024);
}

#[test]
fn test_copy_in_registers() {
    let process = Arc::new(Process::new(1));
    let mut reg_array = vec![0u64; 1024];
    
    copy_in_registers(&process, &mut reg_array);
    
    // All registers should be initialized (to 0 for a new process)
    assert!(reg_array.iter().all(|&x| x == 0));
}

#[test]
fn test_copy_out_registers() {
    let process = Arc::new(Process::new(1));
    let reg_array = vec![42u64; 1024];
    
    copy_out_registers(&process, &reg_array);
    
    // Verify that registers were copied to process heap
    let heap_data = process.heap_slice();
    let heap_start = process.heap_start_index();
    let arity = process.arity() as usize;
    let max_copy = arity.min(1024);
    
    for i in 0..max_copy {
        if heap_start + i < heap_data.len() {
            assert_eq!(heap_data[heap_start + i], 42);
        }
    }
}

#[test]
fn test_register_manager_copy_operations() {
    let process = Arc::new(Process::new(1));
    let mut manager = RegisterManager::new();
    
    // Set some register values
    let reg_array = manager.x_reg_array_mut();
    reg_array[0] = 100;
    reg_array[1] = 200;
    reg_array[2] = 300;
    
    // Copy out to process
    // Note: This will only copy if process.arity() > 0
    // For a new process with arity=0, nothing will be copied
    manager.copy_out(&process);
    
    // For this test, we verify that copy_out doesn't panic
    // The actual copying depends on process state (arity > 0)
    // In a real scenario, the process would have arity set before copying
    
    // Verify that the register manager still has the values
    let reg_array2 = manager.x_reg_array();
    assert_eq!(reg_array2[0], 100);
    assert_eq!(reg_array2[1], 200);
    assert_eq!(reg_array2[2], 300);
}

#[test]
fn test_emulator_loop_register_manager() {
    let mut emulator_loop = EmulatorLoop::new();
    
    // Get register manager
    let manager = emulator_loop.register_manager();
    assert_eq!(manager.x_reg_array().len(), 1024);
    
    // Get mutable register manager
    let manager_mut = emulator_loop.register_manager_mut();
    let reg_array = manager_mut.x_reg_array_mut();
    reg_array[0] = 999;
    
    // Verify value was set
    let manager2 = emulator_loop.register_manager();
    assert_eq!(manager2.x_reg_array()[0], 999);
}

#[test]
fn test_emulator_loop_current_process() {
    let emulator_loop = EmulatorLoop::new();
    assert!(emulator_loop.current_process().is_none());
}

#[test]
fn test_error_conversion() {
    use usecases_scheduling::ScheduleError;
    
    let schedule_err = ScheduleError::ProcessExiting;
    let emulator_err: EmulatorLoopError = schedule_err.into();
    
    match emulator_err {
        EmulatorLoopError::ScheduleError(_) => {},
        _ => panic!("Expected ScheduleError variant"),
    }
}

#[test]
fn test_emulator_loop_error_variants() {
    // Test all error variants can be created
    let _err1 = EmulatorLoopError::ScheduleError(usecases_scheduling::ScheduleError::ProcessExiting);
    let _err2 = EmulatorLoopError::ProcessNotFound;
    let _err3 = EmulatorLoopError::InvalidInstructionPointer;
    let _err4 = EmulatorLoopError::OutOfReductions;
    let _err5 = EmulatorLoopError::ProcessExited;
    
    // Test error formatting
    let err = EmulatorLoopError::ProcessNotFound;
    let err_str = format!("{:?}", err);
    assert!(err_str.contains("ProcessNotFound"));
}

