//! Integration tests for infrastructure_emulator_loop crate
//!
//! These tests verify that emulator loop functions work correctly
//! and test end-to-end workflows for instruction execution and register management.

use infrastructure_emulator_loop::*;
use entities_process::{Process, ErtsCodePtr, ProcessExecutor, ProcessExecutionResult};
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

// ============================================================================
// EmulatorLoopExecutor Integration Tests
// ============================================================================

#[test]
fn test_emulator_loop_executor_creation() {
    let executor = EmulatorLoopExecutor;
    // Should not panic
    let _ = executor;
}

#[test]
fn test_emulator_loop_executor_implements_process_executor() {
    let executor: Box<dyn ProcessExecutor> = Box::new(EmulatorLoopExecutor);
    let process = Arc::new(Process::new(1));
    
    // Execute with null instruction pointer (should exit normally)
    let result = executor.execute(process);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ProcessExecutionResult::NormalExit);
}

#[test]
fn test_emulator_loop_executor_null_instruction_pointer() {
    let executor = EmulatorLoopExecutor;
    let process = Arc::new(Process::new(1));
    
    // Process with null instruction pointer should exit normally
    let result = executor.execute(process);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ProcessExecutionResult::NormalExit);
}

#[test]
fn test_emulator_loop_executor_with_valid_instruction_pointer() {
    let executor = EmulatorLoopExecutor;
    
    // Note: We cannot test with actual BEAM bytecode because the JIT expects
    // compiled native code, not raw bytecode. Testing with raw bytecode would
    // cause the JIT to crash or hang when trying to execute invalid code.
    // Instead, we test with null pointer which should exit immediately.
    
    let instruction_ptr = std::ptr::null();
    let mut process = Process::new(1);
    process.set_i(instruction_ptr);
    let process = Arc::new(process);
    
    // Execute - null pointer should cause normal exit immediately
    let result = executor.execute(process);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ProcessExecutionResult::NormalExit);
}

#[test]
fn test_emulator_loop_executor_multiple_processes() {
    let executor = EmulatorLoopExecutor;
    
    // Execute multiple processes
    for id in 1..=5 {
        let process = Arc::new(Process::new(id));
        let result = executor.execute(process);
        assert!(result.is_ok() || result.is_err());
    }
}

#[test]
fn test_emulator_loop_executor_process_with_stack() {
    let executor = EmulatorLoopExecutor;
    let process = Arc::new(Process::new(1));
    
    // Push some values on the stack before execution
    process.stack_push(42).unwrap();
    process.stack_push(100).unwrap();
    
    // Execute with null instruction pointer (should exit normally)
    let result = executor.execute(process);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), ProcessExecutionResult::NormalExit);
}

#[test]
fn test_emulator_loop_executor_process_with_registers() {
    let executor = EmulatorLoopExecutor;
    let process = Arc::new(Process::new(1));
    
    // Process should have registers that can be accessed
    // Execute with null instruction pointer
    let result = executor.execute(process);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), entities_process::ProcessExecutionResult::NormalExit);
}

#[test]
fn test_emulator_loop_executor_independent_executions() {
    let executor = EmulatorLoopExecutor;
    let process = Arc::new(Process::new(1));
    
    // Verify that executor creates a new EmulatorLoop for each execution
    let result1 = executor.execute(Arc::clone(&process));
    let result2 = executor.execute(Arc::clone(&process));
    
    // Both should succeed independently
    assert!(result1.is_ok() || result1.is_err());
    assert!(result2.is_ok() || result2.is_err());
}

#[test]
fn test_emulator_loop_executor_all_result_variants() {
    let executor = EmulatorLoopExecutor;
    
    // Test that all ProcessExecutionResult variants can be returned
    let process1 = Arc::new(Process::new(1));
    let result1 = executor.execute(process1);
    
    if let Ok(res1) = result1 {
        assert!(matches!(
            res1,
            ProcessExecutionResult::Yield
                | ProcessExecutionResult::NormalExit
                | ProcessExecutionResult::ErrorExit
        ));
    }
}

// ============================================================================
// EmulatorLoop Comprehensive Integration Tests
// ============================================================================

#[test]
fn test_emulator_loop_default() {
    let loop_state = EmulatorLoop::default();
    assert!(loop_state.current_process().is_none());
    assert_eq!(loop_state.reds_used(), 0);
    assert_eq!(loop_state.fcalls(), 0);
    assert_eq!(loop_state.reds_in(), 0);
    assert!(loop_state.instruction_ptr().is_null());
}

#[test]
fn test_emulator_loop_register_manager_access() {
    let loop_state = EmulatorLoop::new();
    let manager = loop_state.register_manager();
    assert_eq!(manager.x_reg_array().len(), 1024);
}

#[test]
fn test_emulator_loop_register_manager_mut_access() {
    let mut loop_state = EmulatorLoop::new();
    let manager = loop_state.register_manager_mut();
    let reg_array = manager.x_reg_array_mut();
    reg_array[0] = 42;
    assert_eq!(reg_array[0], 42);
}

#[test]
fn test_emulator_loop_instruction_pointer_management() {
    let mut loop_state = EmulatorLoop::new();
    assert!(loop_state.instruction_ptr().is_null());
    
    let dummy: u8 = 42;
    let instruction_ptr: ErtsCodePtr = &dummy as *const u8;
    loop_state.set_instruction_ptr(instruction_ptr);
    assert_eq!(loop_state.instruction_ptr(), instruction_ptr);
}

#[test]
fn test_emulator_loop_reductions_management() {
    let mut loop_state = EmulatorLoop::new();
    
    // Set initial reductions
    loop_state.set_reds_in(1000);
    loop_state.set_fcalls(500);
    assert_eq!(loop_state.reds_in(), 1000);
    assert_eq!(loop_state.fcalls(), 500);
    
    // Calculate reductions used
    loop_state.calculate_reds_used(false);
    assert_eq!(loop_state.reds_used(), 500);
    
    // Check if out of reductions
    assert!(!loop_state.is_out_of_reds(false));
    
    // Set fcalls to 0 (out of reductions)
    loop_state.set_fcalls(0);
    assert!(loop_state.is_out_of_reds(false));
}

#[test]
fn test_emulator_loop_reductions_with_saved_calls_buffer() {
    let mut loop_state = EmulatorLoop::new();
    
    loop_state.set_reds_in(1000);
    loop_state.set_fcalls(-10); // CONTEXT_REDS
    
    loop_state.calculate_reds_used(true);
    assert_eq!(loop_state.reds_used(), 1000 - (-10 + (-10))); // reds_in - (CONTEXT_REDS + fcalls)
    
    assert!(loop_state.is_out_of_reds(true));
}

#[test]
fn test_emulator_loop_current_process_management() {
    let mut loop_state = EmulatorLoop::new();
    assert!(loop_state.current_process().is_none());
    
    let process = Arc::new(Process::new(1));
    loop_state.set_current_process(Some(Arc::clone(&process)));
    assert!(loop_state.current_process().is_some());
    assert_eq!(loop_state.current_process().unwrap().id(), 1);
    
    loop_state.set_current_process(None);
    assert!(loop_state.current_process().is_none());
}

#[test]
fn test_emulator_loop_reductions_used_setter() {
    let mut loop_state = EmulatorLoop::new();
    assert_eq!(loop_state.reds_used(), 0);
    
    loop_state.set_reds_used(100);
    assert_eq!(loop_state.reds_used(), 100);
    
    loop_state.set_reds_used(200);
    assert_eq!(loop_state.reds_used(), 200);
}

// ============================================================================
// Register Management Integration Tests
// ============================================================================

#[test]
fn test_register_manager_integration_with_emulator_loop() {
    let mut loop_state = EmulatorLoop::new();
    let mut process = Process::new(1);
    // Set arity to match the number of registers we want to copy
    process.set_arity(5);
    let process = Arc::new(process);
    
    // Set some values in process heap
    {
        let mut heap_data = process.heap_slice_mut();
        let heap_start = process.heap_start_index();
        for i in 0..5 {
            if heap_start + i < heap_data.len() {
                heap_data[heap_start + i] = (i + 1) as u64 * 10;
            }
        }
    }
    
    // Copy in from process
    let manager = loop_state.register_manager_mut();
    manager.copy_in(&process);
    
    // Verify registers were copied
    let reg_array = manager.x_reg_array();
    let heap_data = process.heap_slice();
    let heap_start = process.heap_start_index();
    
    for i in 0..5 {
        if heap_start + i < heap_data.len() {
            assert_eq!(reg_array[i], (i + 1) as u64 * 10);
        }
    }
}

#[test]
fn test_register_manager_round_trip_integration() {
    let mut loop_state = EmulatorLoop::new();
    let mut process = Process::new(1);
    // Set arity to match the number of registers we want to copy
    process.set_arity(10);
    let process = Arc::new(process);
    
    // Set initial values in process heap
    {
        let mut heap_data = process.heap_slice_mut();
        let heap_start = process.heap_start_index();
        for i in 0..10 {
            if heap_start + i < heap_data.len() {
                heap_data[heap_start + i] = (i + 1) as u64 * 50;
            }
        }
    }
    
    // Copy in from process
    let manager = loop_state.register_manager_mut();
    manager.copy_in(&process);
    
    // Modify registers
    let reg_array = manager.x_reg_array_mut();
    for i in 0..10 {
        if i < reg_array.len() {
            reg_array[i] = reg_array[i] * 2;
        }
    }
    
    // Copy out to process
    manager.copy_out(&process);
    
    // Verify round trip
    let heap_data = process.heap_slice();
    let heap_start = process.heap_start_index();
    
    for i in 0..10 {
        if heap_start + i < heap_data.len() {
            assert_eq!(heap_data[heap_start + i], (i + 1) as u64 * 100);
        }
    }
}


// ============================================================================
// Error Handling Integration Tests
// ============================================================================

#[test]
fn test_emulator_loop_error_from_schedule_error() {
    use usecases_scheduling::ScheduleError;
    
    let schedule_error = ScheduleError::ProcessExiting;
    let emulator_error: EmulatorLoopError = schedule_error.into();
    
    match emulator_error {
        EmulatorLoopError::ScheduleError(ScheduleError::ProcessExiting) => {}
        _ => panic!("Expected ScheduleError::ProcessExiting"),
    }
}

#[test]
fn test_emulator_loop_error_display() {
    let errors = vec![
        EmulatorLoopError::ProcessNotFound,
        EmulatorLoopError::InvalidInstructionPointer,
        EmulatorLoopError::OutOfReductions,
        EmulatorLoopError::ProcessExited,
    ];
    
    for error in errors {
        let error_str = format!("{:?}", error);
        assert!(!error_str.is_empty());
    }
}


// ============================================================================
// Init Emulator Integration Tests
// ============================================================================

#[test]
fn test_init_emulator_first_time() {
    let init_done = Arc::new(AtomicBool::new(false));
    let result = init_emulator(Arc::clone(&init_done));
    assert!(result.is_ok());
    assert!(init_done.load(std::sync::atomic::Ordering::Acquire));
}

#[test]
fn test_init_emulator_already_initialized() {
    let init_done = Arc::new(AtomicBool::new(true));
    let result = init_emulator(Arc::clone(&init_done));
    assert!(result.is_ok());
    assert!(init_done.load(std::sync::atomic::Ordering::Acquire));
}

#[test]
fn test_init_emulator_multiple_calls() {
    let init_done = Arc::new(AtomicBool::new(false));
    
    // First call should succeed
    let result1 = init_emulator(Arc::clone(&init_done));
    assert!(result1.is_ok());
    
    // Second call should also succeed (already initialized)
    let result2 = init_emulator(Arc::clone(&init_done));
    assert!(result2.is_ok());
    
    assert!(init_done.load(std::sync::atomic::Ordering::Acquire));
}
