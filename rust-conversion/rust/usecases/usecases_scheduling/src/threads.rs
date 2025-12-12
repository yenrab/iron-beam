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
    
    let schedulers_guard = schedulers_arc.lock().unwrap();
    let num_schedulers = schedulers_guard.len();
    
    // Spawn a thread for each scheduler
    for index in 0..num_schedulers {
        let schedulers_for_thread = Arc::clone(&schedulers_arc);
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = Arc::clone(&running);
        let scheduler_index = index;
        
        let handle = thread::Builder::new()
            .name(format!("erts_sched_{}", index + 1))
            .spawn(move || {
                scheduler_thread_func(schedulers_for_thread, running_clone, scheduler_index);
            })
            .map_err(|e| format!("Failed to create scheduler thread {}: {}", index + 1, e))?;
        
        handles.push(handle);
    }
    
    drop(schedulers_guard);
    
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
                thread::sleep(std::time::Duration::from_millis(10));
                continue;
            }
            
            // Clone the run queue Arc so we can use it outside the lock
            scheduler.runq()
        };
        
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
            // Execute the process
            match execute_process(process.clone()) {
                Ok(ExecutionResult::Yield) => {
                    // Process yielded (out of reductions), reschedule if needed
                    if should_reschedule(&process) {
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
            thread::sleep(std::time::Duration::from_millis(1));
        }
    }
}

/// Process execution result
#[derive(Debug, Clone, PartialEq, Eq)]
enum ExecutionResult {
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
fn should_reschedule(_process: &Process) -> bool {
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
    SCHEDULER_RUNNING.store(false, Ordering::Release);
    
    // Wait for all threads to finish
    for handle in handles {
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
        erts_init_scheduling(2, 2, 0, 0, 0, 0).unwrap();
        
        // Start scheduler threads
        let handles = erts_start_schedulers();
        assert!(handles.is_ok());
        
        let handles = handles.unwrap();
        assert_eq!(handles.len(), 2);
        
        // Stop schedulers
        erts_stop_schedulers(handles);
    }
}

