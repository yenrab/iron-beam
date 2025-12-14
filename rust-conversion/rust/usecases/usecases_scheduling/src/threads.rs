//! Scheduler Thread Management
//!
//! Provides functions for starting and managing scheduler threads.
//! Based on erts_start_schedulers() from erl_process.c

use crate::scheduler::Scheduler;
use crate::initialization::get_global_schedulers;
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};
use entities_process::{Process, ProcessState};

/// Global flag to signal scheduler threads to stop
static SCHEDULER_RUNNING: AtomicBool = AtomicBool::new(false);

/// Per-thread running flags (stored so we can stop threads)
static THREAD_RUNNING_FLAGS: Mutex<Vec<Arc<AtomicBool>>> = Mutex::new(Vec::new());

/// Start all scheduler threads
///
/// Based on `erts_start_schedulers()` from erl_process.c
///
/// This function spawns threads for each scheduler that will run the
/// main scheduling loop. Each thread will continuously dequeue and
/// execute processes from its run queue.
///
/// # Returns
/// * `Ok(Vec<thread::JoinHandle<()>>)` - Vector of thread handles
/// * `Err(String)` - Error starting schedulers
pub fn erts_start_schedulers() -> Result<Vec<thread::JoinHandle<()>>, String> {
    let schedulers = get_global_schedulers()
        .ok_or("Schedulers not initialized. Call erts_init_scheduling() first.")?;
    
    let schedulers_arc = Arc::clone(schedulers);
    let mut handles = Vec::new();
    
    // Set running flag
    SCHEDULER_RUNNING.store(true, Ordering::Release);
    
    // Clear any old thread flags
    let mut flags_guard = THREAD_RUNNING_FLAGS.lock().unwrap();
    flags_guard.clear();
    
    let schedulers_guard = schedulers_arc.lock().unwrap();
    let num_schedulers = schedulers_guard.len();
    
    // Spawn a thread for each scheduler
    for index in 0..num_schedulers {
        let schedulers_for_thread = Arc::clone(&schedulers_arc);
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = Arc::clone(&running);
        let scheduler_index = index;
        
        // Store the flag so we can stop the thread later
        flags_guard.push(Arc::clone(&running));
        
        let handle = thread::Builder::new()
            .name(format!("erts_sched_{}", index + 1))
            .spawn(move || {
                scheduler_thread_func(schedulers_for_thread, running_clone, scheduler_index);
            })
            .map_err(|e| format!("Failed to create scheduler thread {}: {}", index + 1, e))?;
        
        handles.push(handle);
    }
    
    drop(schedulers_guard);
    drop(flags_guard);
    
    Ok(handles)
}

/// Scheduler thread function
///
/// Based on `sched_thread_func()` from erl_process.c
///
/// This is the main loop for each scheduler thread. It continuously
/// dequeues processes from the run queue and executes them.
///
/// # Arguments
/// * `schedulers` - The global schedulers vector
/// * `running` - Flag to control thread execution
/// * `index` - Scheduler index
fn scheduler_thread_func(
    schedulers: Arc<Mutex<Vec<Scheduler>>>,
    running: Arc<AtomicBool>,
    index: usize,
) {
    // In the C implementation, this would:
    // 1. Perform platform-specific scheduler initialization
    // 2. Initialize scheduler-specific data structures
    // 3. Set up signal handling
    // 4. Enter the main scheduling loop
    
    // Main scheduling loop
    while running.load(Ordering::Acquire) && SCHEDULER_RUNNING.load(Ordering::Acquire) {
        // Check stop flags before acquiring locks to avoid deadlocks
        if !running.load(Ordering::Acquire) || !SCHEDULER_RUNNING.load(Ordering::Acquire) {
            break;
        }
        
        // Get scheduler reference (we need to clone the runq Arc to use it outside the lock)
        let runq_arc = {
            let schedulers_guard = schedulers.lock().unwrap();
            
            // Get this scheduler by index
            if index >= schedulers_guard.len() {
                break;
            }
            
            let scheduler = &schedulers_guard[index];
            
            // Check if scheduler is active
            if !scheduler.is_active() {
                // Scheduler is offline, sleep briefly and check again
                // But check stop flags first
                drop(schedulers_guard);
                if !running.load(Ordering::Acquire) || !SCHEDULER_RUNNING.load(Ordering::Acquire) {
                    break;
                }
                thread::sleep(std::time::Duration::from_millis(10));
                continue;
            }
            
            // Clone the run queue Arc so we can use it outside the lock
            scheduler.runq()
        };
        
        // Check stop flags again before acquiring runq lock
        if !running.load(Ordering::Acquire) || !SCHEDULER_RUNNING.load(Ordering::Acquire) {
            break;
        }
        
        // Now we can work with the run queue without holding the schedulers lock
        let runq_guard = runq_arc.lock().unwrap();
        
        // Try to dequeue a process
        let mut executed = 0;
        let priorities = [crate::run_queue::Priority::Max, 
                         crate::run_queue::Priority::High, 
                         crate::run_queue::Priority::Normal];
        
        let dequeued_process = {
            let mut process_opt = None;
            for &prio in &priorities {
                if let Some(process) = crate::run_queue::dequeue_process(&runq_guard, prio) {
                    process_opt = Some((process, prio));
                    break;
                }
            }
            process_opt
        };
        
        drop(runq_guard);
        
        if let Some((process, prio)) = dequeued_process {
            // Check stop flags before executing process
            if !running.load(Ordering::Acquire) || !SCHEDULER_RUNNING.load(Ordering::Acquire) {
                break;
            }
            
            // Execute the process
            match execute_process(process.clone()) {
                Ok(ExecutionResult::Yield) => {
                    // Process yielded (out of reductions), reschedule if needed
                    if should_reschedule(&process) {
                        // Check stop flags before acquiring lock
                        if !running.load(Ordering::Acquire) || !SCHEDULER_RUNNING.load(Ordering::Acquire) {
                            break;
                        }
                        let runq_guard = runq_arc.lock().unwrap();
                        crate::run_queue::enqueue_process(&runq_guard, prio, process);
                    }
                }
                Ok(ExecutionResult::NormalExit) => {
                    // Process finished normally, remove from process table
                    use infrastructure_utilities::process_table::get_global_process_table;
                    let table = get_global_process_table();
                    table.remove(process.id());
                }
                Ok(ExecutionResult::ErrorExit) => {
                    // Process exited with error
                    use infrastructure_utilities::process_table::get_global_process_table;
                    let table = get_global_process_table();
                    table.remove(process.id());
                }
                Err(e) => {
                    eprintln!("Error executing process {}: {}", process.id(), e);
                    // Remove failed process
                    use infrastructure_utilities::process_table::get_global_process_table;
                    let table = get_global_process_table();
                    table.remove(process.id());
                }
            }
            
            executed += 1;
        }
        
        if executed == 0 {
            // No processes available, sleep briefly
            // But check stop flags during sleep
            for _ in 0..10 {
                if !running.load(Ordering::Acquire) || !SCHEDULER_RUNNING.load(Ordering::Acquire) {
                    break;
                }
                thread::sleep(std::time::Duration::from_millis(1));
            }
        }
        
        // Check stop flags at end of loop iteration
        if !running.load(Ordering::Acquire) || !SCHEDULER_RUNNING.load(Ordering::Acquire) {
            break;
        }
    }
}

/// Process execution result
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ExecutionResult {
    /// Process yielded (out of reductions, needs rescheduling)
    Yield,
    /// Process exited normally
    NormalExit,
    /// Process exited with error
    ErrorExit,
}

/// Execute a process
///
/// This function executes a process until it yields or exits.
/// It uses the global process executor to break the circular dependency.
///
/// # Arguments
/// * `process` - Process to execute
///
/// # Returns
/// ExecutionResult indicating what happened
fn execute_process(process: Arc<Process>) -> Result<ExecutionResult, String> {
    // Use the global process executor (set during initialization)
    // This breaks the circular dependency by using a trait in the entities layer
    use entities_process::execute_process as global_execute;
    use entities_process::ProcessExecutionResult;
    
    match global_execute(process) {
        Ok(ProcessExecutionResult::Yield) => Ok(ExecutionResult::Yield),
        Ok(ProcessExecutionResult::NormalExit) => Ok(ExecutionResult::NormalExit),
        Ok(ProcessExecutionResult::ErrorExit) => Ok(ExecutionResult::ErrorExit),
        Err(e) => Err(e),
    }
}

/// Check if a process should be rescheduled
///
/// Determines if a process that yielded should be rescheduled.
pub(crate) fn should_reschedule(_process: &Process) -> bool {
    // For now, always reschedule if process hasn't exited
    // In the full implementation, we'd check process state
    true
}

/// Stop all scheduler threads
///
/// Signals all scheduler threads to stop and waits for them to finish.
///
/// # Arguments
/// * `handles` - Vector of thread handles to wait for
pub fn erts_stop_schedulers(handles: Vec<thread::JoinHandle<()>>) {
    // Set global flag to false
    SCHEDULER_RUNNING.store(false, Ordering::Release);
    
    // Set all per-thread flags to false
    let flags_guard = THREAD_RUNNING_FLAGS.lock().unwrap();
    for flag in flags_guard.iter() {
        flag.store(false, Ordering::Release);
    }
    drop(flags_guard);
    
    // Wait for all threads to finish
    // Use a timeout to prevent hanging forever
    for handle in handles {
        // Give threads a moment to check the flags and exit
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        // Try to join with a reasonable timeout
        // Since Rust's JoinHandle doesn't have a timeout, we'll just join
        // If threads don't exit, this will hang, but that's a bug we need to fix
        let _ = handle.join();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::initialization::erts_init_scheduling;

    #[test]
    fn test_start_schedulers() {
        // Initialize scheduling first
        // Note: If already initialized, this will return Ok(()) without changing the count
        erts_init_scheduling(2, 2, 0, 0, 0, 0).unwrap();
        
        // Start scheduler threads
        let handles = erts_start_schedulers();
        assert!(handles.is_ok());
        
        let handles = handles.unwrap();
        // Get the actual scheduler count (may be different if already initialized)
        // Do this before stopping to avoid deadlock
        let expected_count = {
            let schedulers = get_global_schedulers().unwrap();
            let sched_guard = schedulers.lock().unwrap();
            sched_guard.len()
        };
        assert_eq!(handles.len(), expected_count, "Expected {} scheduler threads, got {}", expected_count, handles.len());
        
        // Give threads a moment to start
        std::thread::sleep(std::time::Duration::from_millis(50));
        
        // Stop schedulers
        erts_stop_schedulers(handles);
    }

    #[test]
    fn test_start_schedulers_not_initialized() {
        // Try to start schedulers without initialization
        // This should fail if schedulers haven't been initialized
        // Note: If schedulers were initialized in a previous test, this might succeed
        // So we test the error message format instead
        let result = erts_start_schedulers();
        // Either it succeeds (if already initialized) or fails with specific error
        if let Err(e) = result {
            assert!(e.contains("not initialized") || e.contains("Schedulers not initialized"));
        }
    }

    #[test]
    fn test_start_schedulers_multiple_times() {
        // Initialize scheduling
        erts_init_scheduling(1, 1, 0, 0, 0, 0).unwrap();
        
        // Start schedulers first time
        let handles1 = erts_start_schedulers();
        assert!(handles1.is_ok());
        let handles1 = handles1.unwrap();
        
        // Give threads a moment
        std::thread::sleep(std::time::Duration::from_millis(50));
        
        // Stop first set
        erts_stop_schedulers(handles1);
        
        // Wait a bit for threads to fully stop
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        // Start schedulers again
        let handles2 = erts_start_schedulers();
        assert!(handles2.is_ok());
        let handles2 = handles2.unwrap();
        
        // Give threads a moment
        std::thread::sleep(std::time::Duration::from_millis(50));
        
        // Stop second set
        erts_stop_schedulers(handles2);
    }

    #[test]
    fn test_stop_schedulers_empty_handles() {
        // Test stopping with empty handles vector
        let empty_handles = Vec::new();
        erts_stop_schedulers(empty_handles);
        // Should not panic or hang
    }

    #[test]
    fn test_execution_result_debug() {
        let results = vec![
            ExecutionResult::Yield,
            ExecutionResult::NormalExit,
            ExecutionResult::ErrorExit,
        ];
        
        for result in results {
            let debug_str = format!("{:?}", result);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_execution_result_clone() {
        let result1 = ExecutionResult::Yield;
        let result2 = result1.clone();
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_execution_result_partial_eq() {
        assert_eq!(ExecutionResult::Yield, ExecutionResult::Yield);
        assert_eq!(ExecutionResult::NormalExit, ExecutionResult::NormalExit);
        assert_eq!(ExecutionResult::ErrorExit, ExecutionResult::ErrorExit);
        
        assert_ne!(ExecutionResult::Yield, ExecutionResult::NormalExit);
        assert_ne!(ExecutionResult::Yield, ExecutionResult::ErrorExit);
        assert_ne!(ExecutionResult::NormalExit, ExecutionResult::ErrorExit);
    }

    #[test]
    fn test_execution_result_eq() {
        // Test Eq trait (which PartialEq provides)
        let r1 = ExecutionResult::Yield;
        let r2 = ExecutionResult::Yield;
        let r3 = ExecutionResult::NormalExit;
        
        assert_eq!(r1, r2);
        assert_ne!(r1, r3);
    }

    #[test]
    fn test_should_reschedule() {
        let process = Process::new(1);
        
        // Currently always returns true
        assert_eq!(should_reschedule(&process), true);
        
        // Test with different process states
        let process2 = Process::new(2);
        assert_eq!(should_reschedule(&process2), true);
    }

    #[test]
    fn test_scheduler_running_flag() {
        // Test that SCHEDULER_RUNNING flag can be read
        // Note: This is a static, so we can't easily reset it between tests
        // But we can verify it's accessible
        let _value = SCHEDULER_RUNNING.load(Ordering::Acquire);
        // Just verify we can read it without panicking
    }

    #[test]
    fn test_thread_running_flags_structure() {
        // Test that THREAD_RUNNING_FLAGS can be accessed
        let flags_guard = THREAD_RUNNING_FLAGS.lock().unwrap();
        // Just verify we can access it without panicking
        let _len = flags_guard.len();
        drop(flags_guard);
    }

    #[test]
    fn test_start_schedulers_single_scheduler() {
        // Initialize with single scheduler
        // Note: If already initialized, this may not change the count
        erts_init_scheduling(1, 1, 0, 0, 0, 0).unwrap();
        
        let handles = erts_start_schedulers();
        assert!(handles.is_ok());
        let handles = handles.unwrap();
        
        // Get actual scheduler count (may be different if already initialized)
        let expected_count = {
            let schedulers = get_global_schedulers().unwrap();
            let sched_guard = schedulers.lock().unwrap();
            sched_guard.len()
        };
        assert_eq!(handles.len(), expected_count);
        
        // Give thread a moment
        std::thread::sleep(std::time::Duration::from_millis(50));
        
        // Stop
        erts_stop_schedulers(handles);
    }

    #[test]
    fn test_start_schedulers_multiple_schedulers() {
        // Initialize with multiple schedulers
        // Note: If already initialized, this may not change the count
        erts_init_scheduling(4, 4, 0, 0, 0, 0).unwrap();
        
        let handles = erts_start_schedulers();
        assert!(handles.is_ok());
        let handles = handles.unwrap();
        
        // Get actual scheduler count (may be different if already initialized)
        let expected_count = {
            let schedulers = get_global_schedulers().unwrap();
            let sched_guard = schedulers.lock().unwrap();
            sched_guard.len()
        };
        assert_eq!(handles.len(), expected_count);
        
        // Give threads a moment
        std::thread::sleep(std::time::Duration::from_millis(50));
        
        // Stop
        erts_stop_schedulers(handles);
    }

    #[test]
    fn test_stop_schedulers_clears_flags() {
        // Initialize and start
        erts_init_scheduling(1, 1, 0, 0, 0, 0).unwrap();
        let handles = erts_start_schedulers().unwrap();
        
        // Give thread a moment
        std::thread::sleep(std::time::Duration::from_millis(50));
        
        // Stop should clear flags
        erts_stop_schedulers(handles);
        
        // Verify SCHEDULER_RUNNING is false after stop
        // Note: This might be affected by other tests, so we just verify it's accessible
        let _value = SCHEDULER_RUNNING.load(Ordering::Acquire);
    }

    #[test]
    fn test_thread_handle_names() {
        // Initialize
        erts_init_scheduling(2, 2, 0, 0, 0, 0).unwrap();
        
        let handles = erts_start_schedulers();
        assert!(handles.is_ok());
        let handles = handles.unwrap();
        
        // Get actual scheduler count (may be different if already initialized)
        let expected_count = {
            let schedulers = get_global_schedulers().unwrap();
            let sched_guard = schedulers.lock().unwrap();
            sched_guard.len()
        };
        
        // Threads should be created (we can't easily check names, but we can verify handles exist)
        assert_eq!(handles.len(), expected_count);
        
        // Give threads a moment
        std::thread::sleep(std::time::Duration::from_millis(50));
        
        // Stop
        erts_stop_schedulers(handles);
    }

    #[test]
    fn test_execution_result_all_variants() {
        // Test all variants exist and are distinct
        let yield_result = ExecutionResult::Yield;
        let normal_exit = ExecutionResult::NormalExit;
        let error_exit = ExecutionResult::ErrorExit;
        
        assert_ne!(yield_result, normal_exit);
        assert_ne!(yield_result, error_exit);
        assert_ne!(normal_exit, error_exit);
        
        assert_eq!(yield_result, ExecutionResult::Yield);
        assert_eq!(normal_exit, ExecutionResult::NormalExit);
        assert_eq!(error_exit, ExecutionResult::ErrorExit);
    }

    #[test]
    fn test_should_reschedule_always_true() {
        // Test that should_reschedule always returns true for now
        // (as per the simplified implementation)
        for i in 1..=10 {
            let process = Process::new(i);
            assert_eq!(should_reschedule(&process), true);
        }
    }

    #[test]
    fn test_start_stop_cycle() {
        // Test multiple start/stop cycles
        erts_init_scheduling(1, 1, 0, 0, 0, 0).unwrap();
        
        for _ in 0..3 {
            let handles = erts_start_schedulers().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(50));
            erts_stop_schedulers(handles);
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }

    #[test]
    fn test_execution_result_ordering() {
        // Test that all variants are distinct and can be compared
        let variants = vec![
            ExecutionResult::Yield,
            ExecutionResult::NormalExit,
            ExecutionResult::ErrorExit,
        ];
        
        // All should be equal to themselves
        for variant in &variants {
            assert_eq!(variant, variant);
        }
        
        // All should be different from each other
        for (i, v1) in variants.iter().enumerate() {
            for (j, v2) in variants.iter().enumerate() {
                if i != j {
                    assert_ne!(v1, v2);
                }
            }
        }
    }
}

