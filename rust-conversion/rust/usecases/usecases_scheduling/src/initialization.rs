//! Scheduling Initialization
//!
//! Provides initialization functions for the scheduling system.
//! Based on erts_init_scheduling() from erl_process.c

use crate::scheduler::Scheduler;
use std::sync::{Arc, Mutex};

/// Global schedulers (initialized by erts_init_scheduling)
static GLOBAL_SCHEDULERS: std::sync::OnceLock<Arc<Mutex<Vec<Scheduler>>>> = std::sync::OnceLock::new();

/// Initialize scheduling system
///
/// Based on `erts_init_scheduling()` from erl_process.c
///
/// This function initializes the scheduler system, creating run queues and
/// setting up scheduler structures. It must be called after process
/// management initialization.
///
/// # Arguments
/// * `no_schedulers` - Number of schedulers to create
/// * `no_schedulers_online` - Number of schedulers to start online
/// * `no_poll_threads` - Number of poll threads
/// * `no_dirty_cpu_schedulers` - Number of dirty CPU schedulers
/// * `no_dirty_cpu_schedulers_online` - Number of dirty CPU schedulers online
/// * `no_dirty_io_schedulers` - Number of dirty IO schedulers
///
/// # Returns
/// * `Ok(())` - Initialization successful
/// * `Err(String)` - Initialization error
///
/// # Note
/// In the C implementation, this also:
/// - Initializes misc op list allocator
/// - Initializes process system task queues allocator
/// - Sets up wakeup other data
/// - Creates and initializes run queues
/// - Sets up scheduler data structures
pub fn erts_init_scheduling(
    no_schedulers: usize,
    no_schedulers_online: usize,
    _no_poll_threads: usize,
    _no_dirty_cpu_schedulers: usize,
    _no_dirty_cpu_schedulers_online: usize,
    _no_dirty_io_schedulers: usize,
) -> Result<(), String> {
    // Validate parameters
    if no_schedulers_online > no_schedulers {
        return Err(format!(
            "no_schedulers_online ({}) cannot exceed no_schedulers ({})",
            no_schedulers_online, no_schedulers
        ));
    }
    if no_schedulers_online < 1 {
        return Err("no_schedulers_online must be at least 1".to_string());
    }
    if no_schedulers < 1 {
        return Err("no_schedulers must be at least 1".to_string());
    }

    // Initialize misc op list allocator (if needed)
    // In C: init_misc_op_list_alloc()
    // For now, we use standard Rust allocators

    // Initialize process system task queues allocator (if needed)
    // In C: init_proc_sys_task_queues_alloc()
    // For now, we use standard Rust allocators

    // Set up wakeup other data (if needed)
    // In C: set_wakeup_other_data()
    // For now, this is handled by the scheduler implementation

    // Create schedulers
    let mut schedulers = Vec::with_capacity(no_schedulers);
    for i in 0..no_schedulers {
        // Create scheduler with default max queue length
        // In a full implementation, this would be configurable
        let scheduler = Scheduler::new(i, 0); // 0 = unlimited queue length
        
        // Set scheduler online state
        if i < no_schedulers_online {
            scheduler.set_active(true);
            scheduler.set_sleeping(false);
        } else {
            scheduler.set_active(false);
            scheduler.set_sleeping(true);
        }
        
        schedulers.push(scheduler);
    }

    // Store global schedulers
    GLOBAL_SCHEDULERS
        .set(Arc::new(Mutex::new(schedulers)))
        .map_err(|_| "Schedulers already initialized".to_string())?;

    // In a full implementation, we would also:
    // - Create dirty CPU schedulers
    // - Create dirty IO schedulers
    // - Set up scheduler thread data structures
    // - Initialize scheduler-specific atomic counters

    Ok(())
}

/// Get the global schedulers
///
/// # Returns
/// Reference to the global schedulers vector
pub fn get_global_schedulers() -> Option<&'static Arc<Mutex<Vec<Scheduler>>>> {
    GLOBAL_SCHEDULERS.get()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_erts_init_scheduling() {
        let result = erts_init_scheduling(4, 4, 1, 0, 0, 0);
        assert!(result.is_ok());
        
        // Verify schedulers are accessible
        let schedulers = get_global_schedulers();
        assert!(schedulers.is_some());
        let schedulers = schedulers.unwrap();
        let scheds = schedulers.lock().unwrap();
        assert_eq!(scheds.len(), 4);
    }

    #[test]
    fn test_erts_init_scheduling_validation() {
        // Test invalid: online > total
        let result = erts_init_scheduling(2, 3, 1, 0, 0, 0);
        assert!(result.is_err());

        // Test invalid: online < 1
        let result = erts_init_scheduling(2, 0, 1, 0, 0, 0);
        assert!(result.is_err());

        // Test invalid: total < 1
        let result = erts_init_scheduling(0, 0, 1, 0, 0, 0);
        assert!(result.is_err());
    }
}

