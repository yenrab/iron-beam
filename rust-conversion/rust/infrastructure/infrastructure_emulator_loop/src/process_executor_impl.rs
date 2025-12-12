//! Process Executor Implementation
//!
//! Implements the ProcessExecutor trait for the emulator loop.
//! This allows the scheduler to execute processes without directly
//! depending on the emulator loop.

use entities_process::{Process, ProcessExecutor, ProcessExecutionResult};
use crate::{EmulatorLoop, process_main, EmulatorLoopError};
use std::sync::{Arc, atomic::AtomicBool};

/// Emulator loop process executor
///
/// Implements ProcessExecutor using the emulator loop's process_main function.
pub struct EmulatorLoopExecutor;

impl ProcessExecutor for EmulatorLoopExecutor {
    fn execute(&self, process: Arc<Process>) -> Result<ProcessExecutionResult, String> {
        // Check if process has code to execute
        // Process has field `i` which is the program counter (instruction pointer)
        let instruction_ptr = process.i();
        
        if instruction_ptr.is_null() {
            // Process has no code, exit normally
            return Ok(ProcessExecutionResult::NormalExit);
        }
        
        // Create emulator loop for this execution
        let mut emulator_loop = EmulatorLoop::new();
        emulator_loop.set_current_process(Some(process.clone()));
        
        // Set instruction pointer from process
        emulator_loop.set_instruction_ptr(instruction_ptr);
        
        // Set up init_done flag (assume already initialized)
        let init_done = Arc::new(AtomicBool::new(true));
        
        // Execute the process using the emulator loop
        match process_main(&mut emulator_loop, init_done) {
            Ok(Some(_next_process)) => {
                // Process yielded, return to scheduler
                Ok(ProcessExecutionResult::Yield)
            }
            Ok(None) => {
                // Process finished
                Ok(ProcessExecutionResult::NormalExit)
            }
            Err(EmulatorLoopError::ProcessExited) => {
                // Process exited with error
                Ok(ProcessExecutionResult::ErrorExit)
            }
            Err(e) => {
                Err(format!("Process execution error: {:?}", e))
            }
        }
    }
}

