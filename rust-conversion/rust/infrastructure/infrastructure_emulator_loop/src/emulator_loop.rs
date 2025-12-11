//! Emulator Loop
//!
//! Provides the main emulator execution loop for BEAM instruction execution.
//! This module implements `process_main()`, the core function that executes
//! BEAM instructions for Erlang processes.
//!
//! Based on `process_main()` and `init_emulator()` from `beam_emu.c`.

use entities_process::{Process, ProcessId, ErtsCodePtr, Eterm};
use usecases_scheduling::{Scheduler, ScheduleError, RunQueue, Priority, dequeue_process};
use std::sync::Mutex;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use super::registers::RegisterManager;

/// Emulator loop error types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EmulatorLoopError {
    /// Scheduler error
    ScheduleError(ScheduleError),
    /// Process not found
    ProcessNotFound,
    /// Invalid instruction pointer
    InvalidInstructionPointer,
    /// Out of reductions
    OutOfReductions,
    /// Process exited
    ProcessExited,
}

impl From<ScheduleError> for EmulatorLoopError {
    fn from(err: ScheduleError) -> Self {
        EmulatorLoopError::ScheduleError(err)
    }
}

/// Emulator loop state
///
/// Manages the state of the emulator loop for a scheduler thread.
/// This struct coordinates process execution, register management, and
/// instruction dispatch.
///
/// Based on the scheduler data structure in the C implementation.
pub struct EmulatorLoop {
    /// Register manager for this scheduler thread
    register_manager: RegisterManager,
    /// Current process being executed
    current_process: Option<Arc<Process>>,
    /// Number of reductions used in current execution
    reds_used: i32,
    /// Initialization flag (stored for future use)
    #[allow(dead_code)]
    init_done: Arc<AtomicBool>,
    /// Current instruction pointer
    instruction_ptr: ErtsCodePtr,
    /// Reductions remaining (FCALLS in C code)
    fcalls: i32,
    /// Reductions at start of execution (REDS_IN in C code)
    reds_in: i32,
}

impl EmulatorLoop {
    /// Create a new emulator loop
    pub fn new() -> Self {
        Self {
            register_manager: RegisterManager::new(),
            current_process: None,
            reds_used: 0,
            init_done: Arc::new(AtomicBool::new(false)),
            instruction_ptr: std::ptr::null(),
            fcalls: 0,
            reds_in: 0,
        }
    }
    
    /// Get the register manager
    pub fn register_manager(&self) -> &RegisterManager {
        &self.register_manager
    }
    
    /// Get mutable reference to the register manager
    pub fn register_manager_mut(&mut self) -> &mut RegisterManager {
        &mut self.register_manager
    }
    
    /// Get the current process
    pub fn current_process(&self) -> Option<&Arc<Process>> {
        self.current_process.as_ref()
    }
    
    /// Set the current process
    pub fn set_current_process(&mut self, process: Option<Arc<Process>>) {
        self.current_process = process;
    }
    
    /// Get current instruction pointer
    pub fn instruction_ptr(&self) -> ErtsCodePtr {
        self.instruction_ptr
    }
    
    /// Set instruction pointer
    pub fn set_instruction_ptr(&mut self, ptr: ErtsCodePtr) {
        self.instruction_ptr = ptr;
    }
    
    /// Get reductions remaining (FCALLS)
    pub fn fcalls(&self) -> i32 {
        self.fcalls
    }
    
    /// Set reductions remaining (FCALLS)
    pub fn set_fcalls(&mut self, fcalls: i32) {
        self.fcalls = fcalls;
    }
    
    /// Get reductions at start (REDS_IN)
    pub fn reds_in(&self) -> i32 {
        self.reds_in
    }
    
    /// Set reductions at start (REDS_IN)
    pub fn set_reds_in(&mut self, reds: i32) {
        self.reds_in = reds;
    }
    
    /// Get reductions used
    pub fn reds_used(&self) -> i32 {
        self.reds_used
    }
    
    /// Set reductions used
    pub fn set_reds_used(&mut self, reds: i32) {
        self.reds_used = reds;
    }
    
    /// Calculate reductions used based on current state
    ///
    /// Based on the reduction calculation in beam_emu.c:
    /// - If no saved calls buffer: reds_used = REDS_IN - FCALLS
    /// - If saved calls buffer: reds_used = REDS_IN - (CONTEXT_REDS + FCALLS)
    pub fn calculate_reds_used(&mut self, has_saved_calls_buf: bool) {
        if has_saved_calls_buf {
            // CONTEXT_REDS is typically -10 in the C code
            const CONTEXT_REDS: i32 = -10;
            self.reds_used = self.reds_in - (CONTEXT_REDS + self.fcalls);
        } else {
            self.reds_used = self.reds_in - self.fcalls;
        }
    }
    
    /// Check if process is out of reductions
    ///
    /// Based on ERTS_IS_PROC_OUT_OF_REDS from bif.h
    pub fn is_out_of_reds(&self, has_saved_calls_buf: bool) -> bool {
        if has_saved_calls_buf {
            const CONTEXT_REDS: i32 = -10;
            self.fcalls == CONTEXT_REDS
        } else {
            self.fcalls <= 0
        }
    }
}

impl Default for EmulatorLoop {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize the emulator
///
/// This function is called once during emulator initialization to set up
/// instruction labels and perform one-time initialization tasks.
///
/// Based on `init_emulator()` and the initialization phase of `process_main()`.
///
/// # Arguments
/// * `init_done` - Shared atomic flag to track initialization state
///
/// # Returns
/// * `Ok(())` - Initialization successful
/// * `Err(EmulatorLoopError)` - Initialization failed
pub fn init_emulator(init_done: Arc<AtomicBool>) -> Result<(), EmulatorLoopError> {
    // Check if already initialized
    if init_done.load(Ordering::Acquire) {
        return Ok(());
    }
    
    // Perform initialization tasks
    // In the C code, this phase exports instruction labels to the loader
    // For now, we just mark initialization as done
    
    init_done.store(true, Ordering::Release);
    
    Ok(())
}

/// Finish emulator initialization
///
/// This function completes the initialization phase of the emulator.
/// It is called after `init_emulator()` to finalize setup.
///
/// Based on `init_emulator_finish()` from `beam_emu.c`.
fn init_emulator_finish() -> Result<(), EmulatorLoopError> {
    // Perform final initialization tasks
    // In the C code, this sets up opcode tables and other structures
    
    Ok(())
}

/// Main emulator loop function
///
/// This is the core function that executes BEAM instructions for processes.
/// It is called by the scheduler to execute a process until it yields or exits.
///
/// Based on `process_main()` from `beam_emu.c`.
///
/// # Arguments
/// * `emulator_loop` - The emulator loop state
/// * `init_done` - Shared atomic flag for initialization state
///
/// # Returns
/// * `Ok(Arc<Process>)` - The next process to execute (or None if scheduler should sleep)
/// * `Err(EmulatorLoopError)` - Error during execution
///
/// # Note
///
/// This function never returns during normal operation. It continuously
/// executes processes until the emulator is shut down. The return value
/// is used for error handling and testing.
pub fn process_main(
    _emulator_loop: &mut EmulatorLoop,
    init_done: Arc<AtomicBool>,
) -> Result<Option<Arc<Process>>, EmulatorLoopError> {
    // Check if initialization is needed
    if !init_done.load(Ordering::Acquire) {
        init_emulator(init_done.clone())?;
        init_emulator_finish()?;
        // After initialization, continue to main loop
    }
    
    // Main execution loop
    // This loop continues until the emulator is shut down
    // Note: In the actual implementation, this would be called by the scheduler
    // and would have access to the scheduler's run queue
    
    // The actual erts_schedule() in the C code has signature:
    // Process* erts_schedule(ErtsSchedulerData *esdp, Process *p, int calls)
    // It returns the next process to execute
    
    // Since our Rust erts_schedule() has a different signature (it's a scheduler loop),
    // we need to use the run queue directly. For now, we'll provide a simplified
    // interface that requires the scheduler to be passed in
    
    // This is a placeholder - the actual implementation would integrate with the scheduler
    // and use the run queue to get the next process
    
    // For now, return None to indicate no process available
    // The caller (scheduler) should handle getting the next process from the run queue
    // 
    // Note: The full implementation would:
    // 1. Get next process from scheduler's run queue
    // 2. Copy registers from process to scheduler registers
    // 3. Set up instruction pointer and reductions
    // 4. Execute instructions in a loop until process yields
    // 5. Copy registers back to process
    // 6. Return to scheduler for next process
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_emulator_loop_creation() {
        let loop_state = EmulatorLoop::new();
        assert!(loop_state.current_process().is_none());
        assert_eq!(loop_state.reds_used(), 0);
        assert_eq!(loop_state.fcalls(), 0);
        assert_eq!(loop_state.reds_in(), 0);
        assert!(loop_state.instruction_ptr().is_null());
    }
    
    #[test]
    fn test_emulator_loop_reductions() {
        let mut loop_state = EmulatorLoop::new();
        
        // Set initial reductions
        loop_state.set_reds_in(1000);
        loop_state.set_fcalls(500);
        
        // Calculate reductions used (no saved calls buffer)
        loop_state.calculate_reds_used(false);
        assert_eq!(loop_state.reds_used(), 500);
        
        // Check if out of reductions
        assert!(!loop_state.is_out_of_reds(false));
        
        // Set fcalls to 0 (out of reductions)
        loop_state.set_fcalls(0);
        assert!(loop_state.is_out_of_reds(false));
        
        // Test with saved calls buffer
        loop_state.set_fcalls(-10); // CONTEXT_REDS
        assert!(loop_state.is_out_of_reds(true));
    }
    
    #[test]
    fn test_emulator_loop_instruction_ptr() {
        let mut loop_state = EmulatorLoop::new();
        assert!(loop_state.instruction_ptr().is_null());
        
        // In a real scenario, we'd set a valid instruction pointer
        // For testing, we just verify the setter/getter works
        let test_ptr = 0x1000 as ErtsCodePtr;
        loop_state.set_instruction_ptr(test_ptr);
        assert_eq!(loop_state.instruction_ptr(), test_ptr);
    }
    
    #[test]
    fn test_emulator_loop_current_process() {
        let mut loop_state = EmulatorLoop::new();
        assert!(loop_state.current_process().is_none());
        
        let process = Arc::new(Process::new(1));
        loop_state.set_current_process(Some(process.clone()));
        
        assert!(loop_state.current_process().is_some());
        assert_eq!(loop_state.current_process().unwrap().id(), process.id());
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
    }
    
    #[test]
    fn test_process_main_initialization() {
        let mut emulator_loop = EmulatorLoop::new();
        let init_done = Arc::new(AtomicBool::new(false));
        
        // This will initialize and then try to schedule, which may fail
        // if scheduler is not properly set up, but initialization should work
        let result = process_main(&mut emulator_loop, init_done.clone());
        
        // The function may return an error if scheduler is not available,
        // but initialization should have completed
        assert!(init_done.load(Ordering::Acquire));
    }
}

