//! Integration tests for usecases_scheduling
//!
//! Tests the scheduling functionality with real Process instances
//! and verifies the run queue and scheduler behavior.

use std::sync::Arc;
use usecases_scheduling::*;
use entities_process::{Process, ProcessId};

#[test]
fn test_enqueue_dequeue_process() {
    let runq = RunQueue::new(0, 1000);
    
    // Create a test process
    let process_id: ProcessId = 1;
    let process = Arc::new(Process::new(process_id));
    
    // Enqueue at NORMAL priority
    enqueue_process(&runq, Priority::Normal, Arc::clone(&process));
    assert_eq!(runq.total_len(), 1);
    
    // Dequeue the process
    let dequeued = dequeue_process(&runq, Priority::Normal);
    assert!(dequeued.is_some());
    assert_eq!(dequeued.unwrap().id(), process_id);
    assert_eq!(runq.total_len(), 0);
}

#[test]
fn test_priority_ordering() {
    let runq = RunQueue::new(0, 1000);
    
    // Create processes with different priorities
    let p_low = Arc::new(Process::new(1));
    let p_normal = Arc::new(Process::new(2));
    let p_high = Arc::new(Process::new(3));
    let p_max = Arc::new(Process::new(4));
    
    // Enqueue in reverse priority order
    enqueue_process(&runq, Priority::Low, Arc::clone(&p_low));
    enqueue_process(&runq, Priority::Normal, Arc::clone(&p_normal));
    enqueue_process(&runq, Priority::High, Arc::clone(&p_high));
    enqueue_process(&runq, Priority::Max, Arc::clone(&p_max));
    
    assert_eq!(runq.total_len(), 4);
    
    // Dequeue should follow priority order: MAX, HIGH, NORMAL, LOW
    let first = dequeue_process(&runq, Priority::Max);
    assert!(first.is_some());
    assert_eq!(first.unwrap().id(), 4);
    
    let second = dequeue_process(&runq, Priority::High);
    assert!(second.is_some());
    assert_eq!(second.unwrap().id(), 3);
    
    let third = dequeue_process(&runq, Priority::Normal);
    assert!(third.is_some());
    // LOW priority processes are in the NORMAL queue
    let third_id = third.as_ref().unwrap().id();
    assert!(third_id == 2 || third_id == 1);
}

#[test]
fn test_schedule_process() {
    let runq = RunQueue::new(0, 1000);
    let process = Arc::new(Process::new(1));
    
    // Schedule the process
    let result = schedule_process(Arc::clone(&process), &runq, Priority::Normal);
    assert!(result.is_ok());
    assert_eq!(runq.total_len(), 1);
}

#[test]
fn test_scheduler_basic() {
    let scheduler = Scheduler::new(0, 1000);
    assert_eq!(scheduler.index(), 0);
    assert!(!scheduler.is_active());
    assert!(!scheduler.is_sleeping());
}

#[test]
fn test_scheduler_wake_sleep() {
    let scheduler = Scheduler::new(0, 1000);
    
    // Put scheduler to sleep
    scheduler.set_sleeping(true);
    assert!(scheduler.is_sleeping());
    
    // Wake scheduler
    wake_scheduler(&scheduler);
    assert!(!scheduler.is_sleeping());
    assert!(scheduler.is_active());
}

#[test]
fn test_scheduler_suspend() {
    let scheduler = Scheduler::new(0, 1000);
    scheduler.set_active(true);
    
    init_scheduler_suspend(&scheduler);
    assert!(!scheduler.is_active());
    assert!(scheduler.is_sleeping());
}

#[test]
fn test_erts_schedule_with_processes() {
    let scheduler = Scheduler::new(0, 1000);
    let runq = scheduler.runq();
    
    // Add some processes to the run queue
    let p1 = Arc::new(Process::new(1));
    let p2 = Arc::new(Process::new(2));
    let p3 = Arc::new(Process::new(3));
    
    {
        let runq_guard = runq.lock().unwrap();
        enqueue_process(&runq_guard, Priority::Normal, Arc::clone(&p1));
        enqueue_process(&runq_guard, Priority::High, Arc::clone(&p2));
        enqueue_process(&runq_guard, Priority::Max, Arc::clone(&p3));
    }
    
    // Run scheduler
    let executed = erts_schedule(&scheduler, 10);
    assert_eq!(executed, 3); // Should execute all 3 processes
}

#[test]
fn test_erts_schedule_empty() {
    let scheduler = Scheduler::new(0, 1000);
    let executed = erts_schedule(&scheduler, 10);
    assert_eq!(executed, 0); // No processes to execute
}

#[test]
fn test_run_queue_info() {
    let mut info = RunQueueInfo::new();
    assert_eq!(info.len(), 0);
    assert_eq!(info.reds(), 0);
    
    info.inc_len();
    info.inc_len();
    assert_eq!(info.len(), 2);
    
    info.dec_len();
    assert_eq!(info.len(), 1);
    
    info.add_reds(100);
    info.add_reds(50);
    assert_eq!(info.reds(), 150);
}

#[test]
fn test_priority_conversion() {
    assert_eq!(Priority::Max.as_index(), 0);
    assert_eq!(Priority::High.as_index(), 1);
    assert_eq!(Priority::Normal.as_index(), 2);
    assert_eq!(Priority::Low.as_index(), 3);
    
    assert_eq!(Priority::from_index(0), Some(Priority::Max));
    assert_eq!(Priority::from_index(1), Some(Priority::High));
    assert_eq!(Priority::from_index(2), Some(Priority::Normal));
    assert_eq!(Priority::from_index(3), Some(Priority::Low));
    assert_eq!(Priority::from_index(4), None);
}

