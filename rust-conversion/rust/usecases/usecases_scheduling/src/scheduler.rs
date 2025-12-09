//! Scheduler Functions
//!
//! Provides scheduler functions for coordinating process execution.
//! Based on scheduler functions from erl_process.c
//!
//! This module implements the main scheduler loop, process scheduling,
//! and scheduler state management.

use std::sync::{Arc, Mutex};
use entities_process::{Process, ProcessState};
use crate::run_queue::{RunQueue, Priority, dequeue_process, enqueue_process};

/// Scheduler state
///
/// Tracks the state of a scheduler thread, including the run queue
/// and scheduler-specific data.
pub struct Scheduler {
    /// Run queue for this scheduler
    runq: Arc<Mutex<RunQueue>>,
    /// Scheduler index
    index: usize,
    /// Whether scheduler is active
    active: Mutex<bool>,
    /// Whether scheduler is sleeping
    sleeping: Mutex<bool>,
}

impl Scheduler {
    /// Create a new scheduler
    ///
    /// # Arguments
    /// * `index` - Scheduler index
    /// * `max_queue_len` - Maximum run queue length (0 = unlimited)
    pub fn new(index: usize, max_queue_len: usize) -> Self {
        Self {
            runq: Arc::new(Mutex::new(RunQueue::new(index, max_queue_len))),
            index,
            active: Mutex::new(false),
            sleeping: Mutex::new(false),
        }
    }

    /// Get scheduler index
    pub fn index(&self) -> usize {
        self.index
    }

    /// Get the run queue
    pub fn runq(&self) -> Arc<Mutex<RunQueue>> {
        Arc::clone(&self.runq)
    }

    /// Check if scheduler is active
    pub fn is_active(&self) -> bool {
        *self.active.lock().unwrap()
    }

    /// Set scheduler active state
    pub fn set_active(&self, active: bool) {
        *self.active.lock().unwrap() = active;
    }

    /// Check if scheduler is sleeping
    pub fn is_sleeping(&self) -> bool {
        *self.sleeping.lock().unwrap()
    }

    /// Set scheduler sleeping state
    pub fn set_sleeping(&self, sleeping: bool) {
        *self.sleeping.lock().unwrap() = sleeping;
    }
}

/// Schedule a process
///
/// Based on schedule_process() and erts_schedule_process() from erl_process.c
///
/// This function marks a process as active and enqueues it into the appropriate
/// run queue based on its priority.
///
/// # Arguments
/// * `process` - Process to schedule
/// * `runq` - Run queue to enqueue into
/// * `priority` - Priority level for the process
///
/// # Note
/// The process must be in a state that allows scheduling (not exiting, etc.)
pub fn schedule_process(
    process: Arc<Process>,
    runq: &RunQueue,
    priority: Priority,
) -> Result<(), ScheduleError> {
    // Check if process can be scheduled
    let state = process.get_state();
    if matches!(state, ProcessState::Exiting | ProcessState::Free) {
        return Err(ScheduleError::ProcessExiting);
    }

    // Enqueue the process
    enqueue_process(runq, priority, process);
    
    Ok(())
}

/// Main scheduler function
///
/// Based on erts_schedule() from erl_process.c
///
/// This is the main scheduler loop that dequeues processes from the run queue
/// and executes them. This is a simplified version that focuses on the
/// core scheduling logic.
///
/// # Arguments
/// * `scheduler` - Scheduler to run
/// * `max_iterations` - Maximum number of iterations (0 = unlimited)
///
/// # Returns
/// * Number of processes executed
///
/// # Note
/// The full implementation would include:
/// - Process execution (calling process_main())
/// - Reduction counting
/// - Time slice management
/// - System task handling
/// - Migration and load balancing
pub fn erts_schedule(
    scheduler: &Scheduler,
    max_iterations: usize,
) -> usize {
    let mut executed = 0;
    let runq = scheduler.runq();
    let runq_guard = runq.lock().unwrap();

    // Try to dequeue processes from highest to lowest priority
    let priorities = [Priority::Max, Priority::High, Priority::Normal];
    
        for _ in 0..max_iterations {
            let mut found = false;
            
            for &prio in &priorities {
                if let Some(_process) = dequeue_process(&runq_guard, prio) {
                    // In the full implementation, we would:
                    // 1. Execute the process (process_main())
                    // 2. Handle reductions
                    // 3. Check if process should be rescheduled
                    // 4. Handle system tasks
                    
                    executed += 1;
                    found = true;
                    break;
                }
            }
            
            if !found {
                // No processes available
                break;
            }
        }
    
    executed
}

/// Wake a scheduler
///
/// Based on wake_scheduler() from erl_process.c
///
/// Wakes up a sleeping scheduler when there are processes to execute.
///
/// # Arguments
/// * `scheduler` - Scheduler to wake
pub fn wake_scheduler(scheduler: &Scheduler) {
    scheduler.set_sleeping(false);
    scheduler.set_active(true);
    
    // In the full implementation, this would:
    // 1. Signal the scheduler thread
    // 2. Update scheduler state
    // 3. Notify other schedulers if needed
}

/// Initialize scheduler suspend
///
/// Based on init_scheduler_suspend() from erl_process.c
///
/// Prepares a scheduler for suspension (e.g., during system shutdown).
///
/// # Arguments
/// * `scheduler` - Scheduler to suspend
pub fn init_scheduler_suspend(scheduler: &Scheduler) {
    scheduler.set_active(false);
    scheduler.set_sleeping(true);
    
    // In the full implementation, this would:
    // 1. Drain the run queue
    // 2. Wait for processes to finish
    // 3. Update scheduler state
}

/// Scheduler error types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScheduleError {
    /// Process is exiting and cannot be scheduled
    ProcessExiting,
    /// Run queue is full
    QueueFull,
    /// Invalid priority level
    InvalidPriority,
    /// Scheduler is not active
    SchedulerInactive,
}

impl std::fmt::Display for ScheduleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScheduleError::ProcessExiting => write!(f, "Process is exiting"),
            ScheduleError::QueueFull => write!(f, "Run queue is full"),
            ScheduleError::InvalidPriority => write!(f, "Invalid priority level"),
            ScheduleError::SchedulerInactive => write!(f, "Scheduler is not active"),
        }
    }
}

impl std::error::Error for ScheduleError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduler_creation() {
        let scheduler = Scheduler::new(0, 1000);
        assert_eq!(scheduler.index(), 0);
        assert!(!scheduler.is_active());
        assert!(!scheduler.is_sleeping());
    }

    #[test]
    fn test_scheduler_state() {
        let scheduler = Scheduler::new(0, 1000);
        
        scheduler.set_active(true);
        assert!(scheduler.is_active());
        
        scheduler.set_sleeping(true);
        assert!(scheduler.is_sleeping());
    }

    #[test]
    fn test_wake_scheduler() {
        let scheduler = Scheduler::new(0, 1000);
        scheduler.set_sleeping(true);
        
        wake_scheduler(&scheduler);
        assert!(!scheduler.is_sleeping());
        assert!(scheduler.is_active());
    }

    #[test]
    fn test_init_scheduler_suspend() {
        let scheduler = Scheduler::new(0, 1000);
        scheduler.set_active(true);
        
        init_scheduler_suspend(&scheduler);
        assert!(!scheduler.is_active());
        assert!(scheduler.is_sleeping());
    }

    #[test]
    fn test_erts_schedule_empty_queue() {
        let scheduler = Scheduler::new(0, 1000);
        let executed = erts_schedule(&scheduler, 10);
        assert_eq!(executed, 0);
    }
}

