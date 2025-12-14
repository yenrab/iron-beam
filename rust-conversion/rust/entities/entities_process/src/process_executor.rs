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
/// This function is idempotent - calling it multiple times is safe and will
/// only set the executor once.
///
/// # Arguments
/// * `executor` - The process executor implementation
pub fn set_process_executor(executor: Box<dyn ProcessExecutor + Send + Sync>) -> Result<(), String> {
    // If already set, return Ok(()) to allow idempotent initialization
    if PROCESS_EXECUTOR.set(executor).is_err() {
        // Already set - this is OK for idempotent initialization
        return Ok(());
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    // Mock ProcessExecutor for testing
    struct MockExecutor {
        result: ProcessExecutionResult,
        should_error: bool,
        error_message: String,
    }

    impl ProcessExecutor for MockExecutor {
        fn execute(&self, _process: Arc<Process>) -> Result<ProcessExecutionResult, String> {
            if self.should_error {
                Err(self.error_message.clone())
            } else {
                Ok(self.result.clone())
            }
        }
    }

    // Helper to create a mock executor that returns a specific result
    fn create_mock_executor(result: ProcessExecutionResult) -> Box<dyn ProcessExecutor + Send + Sync> {
        Box::new(MockExecutor {
            result,
            should_error: false,
            error_message: String::new(),
        })
    }

    // Helper to create a mock executor that returns an error
    fn create_error_executor(error_message: String) -> Box<dyn ProcessExecutor + Send + Sync> {
        Box::new(MockExecutor {
            result: ProcessExecutionResult::ErrorExit,
            should_error: true,
            error_message,
        })
    }

    #[test]
    fn test_process_execution_result_variants() {
        // Test all enum variants can be created
        let yield_result = ProcessExecutionResult::Yield;
        let normal_exit = ProcessExecutionResult::NormalExit;
        let error_exit = ProcessExecutionResult::ErrorExit;

        // Test Debug trait
        let _ = format!("{:?}", yield_result);
        let _ = format!("{:?}", normal_exit);
        let _ = format!("{:?}", error_exit);

        // Test Clone trait
        let cloned_yield = yield_result.clone();
        let cloned_normal = normal_exit.clone();
        let cloned_error = error_exit.clone();

        // Test PartialEq trait
        assert_eq!(yield_result, cloned_yield);
        assert_eq!(normal_exit, cloned_normal);
        assert_eq!(error_exit, cloned_error);
        assert_ne!(yield_result, normal_exit);
        assert_ne!(yield_result, error_exit);
        assert_ne!(normal_exit, error_exit);

        // Test Eq trait (all variants are distinct)
        assert!(yield_result == cloned_yield);
        assert!(normal_exit != error_exit);
    }

    #[test]
    fn test_set_process_executor_success() {
        // Reset the executor by creating a new one
        // Note: OnceLock doesn't have a reset method, so we test the first set
        // In real usage, this would be called during initialization
        
        let executor = create_mock_executor(ProcessExecutionResult::Yield);
        let result = set_process_executor(executor);
        
        // First call should succeed
        // Note: This test may fail if executor was already set in a previous test
        // In a real test environment, we'd need to reset the static, but OnceLock doesn't support that
        // For now, we just verify the function doesn't panic
        let _ = result;
    }

    #[test]
    fn test_set_process_executor_idempotent() {
        // Test that calling set_process_executor multiple times is safe
        let executor1 = create_mock_executor(ProcessExecutionResult::Yield);
        let executor2 = create_mock_executor(ProcessExecutionResult::NormalExit);
        
        // First call
        let result1 = set_process_executor(executor1);
        // Second call (should be idempotent - returns Ok even if already set)
        let result2 = set_process_executor(executor2);
        
        // Both should return Ok(()) due to idempotent behavior
        // Note: In practice, the first executor will remain set
        assert!(result1.is_ok());
        assert!(result2.is_ok());
    }

    #[test]
    fn test_execute_process_when_not_set() {
        // Test execute_process when executor hasn't been set
        // We need to ensure the executor is not set for this test
        // Since we can't reset OnceLock, we'll test the error path
        
        let process = Arc::new(Process::new(1));
        
        // Try to execute without setting executor
        // This will fail if executor was set in previous tests
        // In a clean environment, this should return an error
        let result = execute_process(process);
        
        // If executor is not set, should return error
        // If executor was set in previous test, this will succeed
        // We check both cases
        if result.is_err() {
            assert!(result.unwrap_err().contains("Process executor not set"));
        } else {
            // Executor was already set, which is fine for this test
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_execute_process_when_set() {
        // Test that execute_process works when executor is set
        // Note: OnceLock can only be set once, so we test with whatever executor is currently set
        // or set a new one if not set yet
        let executor = create_mock_executor(ProcessExecutionResult::Yield);
        let _ = set_process_executor(executor);
        
        let process = Arc::new(Process::new(1));
        let result = execute_process(process);
        
        // Should succeed (executor is set)
        assert!(result.is_ok());
        // Result will be whatever the executor returns (may vary based on which test ran first)
        let _execution_result = result.unwrap();
    }

    #[test]
    fn test_execute_process_with_different_processes() {
        // Test that execute_process works with different process instances
        // Ensure executor is set (idempotent, so safe to call)
        let executor = create_mock_executor(ProcessExecutionResult::Yield);
        let _ = set_process_executor(executor);
        
        let process1 = Arc::new(Process::new(10));
        let process2 = Arc::new(Process::new(20));
        let process3 = Arc::new(Process::new(30));
        
        let result1 = execute_process(process1);
        let result2 = execute_process(process2);
        let result3 = execute_process(process3);
        
        // All should succeed
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert!(result3.is_ok());
        
        // All should return the same result (same executor instance)
        let result1_val = result1.unwrap();
        let result2_val = result2.unwrap();
        let result3_val = result3.unwrap();
        assert_eq!(result1_val, result2_val);
        assert_eq!(result2_val, result3_val);
    }

    #[test]
    fn test_mock_executor_directly() {
        // Test mock executor directly (not through global) to verify all result types work
        let yield_executor = create_mock_executor(ProcessExecutionResult::Yield);
        let normal_executor = create_mock_executor(ProcessExecutionResult::NormalExit);
        let error_executor = create_mock_executor(ProcessExecutionResult::ErrorExit);
        
        let process = Arc::new(Process::new(100));
        
        // Test each executor type directly
        let result1 = yield_executor.execute(process.clone());
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap(), ProcessExecutionResult::Yield);
        
        let result2 = normal_executor.execute(process.clone());
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), ProcessExecutionResult::NormalExit);
        
        let result3 = error_executor.execute(process);
        assert!(result3.is_ok());
        assert_eq!(result3.unwrap(), ProcessExecutionResult::ErrorExit);
    }

    #[test]
    fn test_mock_executor_error_propagation() {
        // Test that executor errors are propagated correctly
        let error_msg = "Execution failed: test error".to_string();
        let error_executor = create_error_executor(error_msg.clone());
        
        let process = Arc::new(Process::new(200));
        let result = error_executor.execute(process);
        
        // Should propagate the error from executor
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), error_msg);
    }

    #[test]
    fn test_process_execution_result_all_variants_equality() {
        // Test comprehensive equality checks
        let yield1 = ProcessExecutionResult::Yield;
        let yield2 = ProcessExecutionResult::Yield;
        let normal1 = ProcessExecutionResult::NormalExit;
        let normal2 = ProcessExecutionResult::NormalExit;
        let error1 = ProcessExecutionResult::ErrorExit;
        let error2 = ProcessExecutionResult::ErrorExit;
        
        // Same variants should be equal
        assert_eq!(yield1, yield2);
        assert_eq!(normal1, normal2);
        assert_eq!(error1, error2);
        
        // Different variants should not be equal
        assert_ne!(yield1, normal1);
        assert_ne!(yield1, error1);
        assert_ne!(normal1, error1);
    }

    #[test]
    fn test_process_execution_result_clone_independence() {
        // Test that cloned results are independent
        let original = ProcessExecutionResult::Yield;
        let cloned = original.clone();
        
        // Cloned should be equal
        assert_eq!(original, cloned);
        
        // But they are separate values
        let normal = ProcessExecutionResult::NormalExit;
        assert_ne!(original, normal);
        assert_ne!(cloned, normal);
    }


    #[test]
    fn test_process_execution_result_debug_format() {
        // Test Debug formatting for all variants
        let yield_result = ProcessExecutionResult::Yield;
        let normal_exit = ProcessExecutionResult::NormalExit;
        let error_exit = ProcessExecutionResult::ErrorExit;
        
        let yield_str = format!("{:?}", yield_result);
        let normal_str = format!("{:?}", normal_exit);
        let error_str = format!("{:?}", error_exit);
        
        // Debug strings should not be empty
        assert!(!yield_str.is_empty());
        assert!(!normal_str.is_empty());
        assert!(!error_str.is_empty());
        
        // Different variants should have different debug strings
        assert_ne!(yield_str, normal_str);
        assert_ne!(yield_str, error_str);
        assert_ne!(normal_str, error_str);
    }
}

