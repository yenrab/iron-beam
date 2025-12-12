//! Process Executor Trait
//!
//! Defines a trait for executing processes, allowing the scheduler to execute
//! processes without directly depending on the emulator loop implementation.
//! This breaks the circular dependency between usecases_scheduling and
//! infrastructure_emulator_loop.

use crate::Process;
use std::sync::Arc;

/// Result of executing a process
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessExecutionResult {
    /// Process yielded (out of reductions, needs rescheduling)
    Yield,
    /// Process exited normally
    NormalExit,
    /// Process exited with error
    ErrorExit,
}

/// Trait for executing processes
///
/// This trait allows the scheduler to execute processes without directly
/// depending on the emulator loop implementation. The emulator loop
/// implements this trait, and the scheduler uses it.
pub trait ProcessExecutor {
    /// Execute a process until it yields or exits
    ///
    /// # Arguments
    /// * `process` - Process to execute
    ///
    /// # Returns
    /// ProcessExecutionResult indicating what happened
    fn execute(&self, process: Arc<Process>) -> Result<ProcessExecutionResult, String>;
}

/// Global process executor (set during initialization)
static PROCESS_EXECUTOR: std::sync::OnceLock<Box<dyn ProcessExecutor + Send + Sync>> = std::sync::OnceLock::new();

/// Set the global process executor
///
/// This should be called during initialization, after the emulator loop is ready.
///
/// # Arguments
/// * `executor` - The process executor implementation
pub fn set_process_executor(executor: Box<dyn ProcessExecutor + Send + Sync>) -> Result<(), String> {
    PROCESS_EXECUTOR
        .set(executor)
        .map_err(|_| "Process executor already set".to_string())
}

/// Execute a process using the global executor
///
/// # Arguments
/// * `process` - Process to execute
///
/// # Returns
/// ProcessExecutionResult indicating what happened
///
/// # Errors
/// Returns an error if the executor has not been set or if execution fails
pub fn execute_process(process: Arc<Process>) -> Result<ProcessExecutionResult, String> {
    let executor = PROCESS_EXECUTOR
        .get()
        .ok_or("Process executor not set. Call set_process_executor() during initialization.")?;
    
    executor.execute(process)
}

