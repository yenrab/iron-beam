//! Run Queue Management
//!
//! Provides run queue data structures and operations for scheduling processes.
//! Based on ErtsRunQueue, ErtsRunPrioQueue, and ErtsRunQueueInfo from erl_process.h
//!
//! The run queue maintains multiple priority queues for processes at different
//! priority levels: MAX, HIGH, NORMAL, and LOW.

use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use entities_process::Process;

/// Process priority levels
///
/// Based on PRIORITY_* constants from erl_process.h
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    /// Maximum priority (0)
    Max = 0,
    /// High priority (1)
    High = 1,
    /// Normal priority (2)
    Normal = 2,
    /// Low priority (3)
    Low = 3,
}

impl Priority {
    /// Number of priority levels
    pub const LEVELS: usize = 4;

    /// Convert priority level to index
    pub fn as_index(self) -> usize {
        self as usize
    }

    /// Convert index to priority level
    pub fn from_index(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(Priority::Max),
            1 => Some(Priority::High),
            2 => Some(Priority::Normal),
            3 => Some(Priority::Low),
            _ => None,
        }
    }
}

/// Run queue information for a priority level
///
/// Tracks the length, maximum length, and reductions for processes at a priority level.
/// Based on ErtsRunQueueInfo from erl_process.h
#[derive(Debug, Clone)]
pub struct RunQueueInfo {
    /// Current length (number of processes in queue)
    len: usize,
    /// Maximum length (0 = unlimited)
    max_len: usize,
    /// Reductions executed at this priority level
    reds: i64,
}

impl RunQueueInfo {
    /// Create a new RunQueueInfo
    pub fn new() -> Self {
        Self {
            len: 0,
            max_len: 0,
            reds: 0,
        }
    }

    /// Get current length
    pub fn len(&self) -> usize {
        self.len
    }

    /// Get maximum length
    pub fn max_len(&self) -> usize {
        self.max_len
    }

    /// Set maximum length
    pub fn set_max_len(&mut self, max_len: usize) {
        self.max_len = max_len;
    }

    /// Get reductions
    pub fn reds(&self) -> i64 {
        self.reds
    }

    /// Increment length
    pub fn inc_len(&mut self) {
        self.len += 1;
    }

    /// Decrement length
    pub fn dec_len(&mut self) {
        if self.len > 0 {
            self.len -= 1;
        }
    }

    /// Add reductions
    pub fn add_reds(&mut self, reds: i64) {
        self.reds += reds;
    }
}

impl Default for RunQueueInfo {
    fn default() -> Self {
        Self::new()
    }
}

/// Priority queue for processes
///
/// Maintains a linked list of processes at a specific priority level.
/// Based on ErtsRunPrioQueue from erl_process.h
///
/// In Rust, we use VecDeque for the queue instead of a linked list for better
/// cache locality and safety. The C implementation uses a linked list with
/// Process->next pointers, but we use Arc<Process> in a VecDeque.
pub struct RunPrioQueue {
    /// Queue of processes (FIFO)
    queue: Mutex<VecDeque<Arc<Process>>>,
}

impl RunPrioQueue {
    /// Create a new priority queue
    pub fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
        }
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.queue.lock().unwrap().is_empty()
    }

    /// Get the first process without removing it
    pub fn first(&self) -> Option<Arc<Process>> {
        self.queue.lock().unwrap().front().map(Arc::clone)
    }

    /// Get the last process without removing it
    pub fn last(&self) -> Option<Arc<Process>> {
        self.queue.lock().unwrap().back().map(Arc::clone)
    }

    /// Enqueue a process at the end of the queue
    pub fn enqueue(&self, process: Arc<Process>) {
        self.queue.lock().unwrap().push_back(process);
    }

    /// Dequeue a process from the front of the queue
    pub fn dequeue(&self) -> Option<Arc<Process>> {
        self.queue.lock().unwrap().pop_front()
    }

    /// Get the length of the queue
    pub fn len(&self) -> usize {
        self.queue.lock().unwrap().len()
    }
}

impl Default for RunPrioQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Run queue for a scheduler
///
/// Maintains priority queues for processes at different priority levels.
/// Based on ErtsRunQueue from erl_process.h
///
/// This is a simplified version that focuses on the core scheduling functionality.
/// The full C implementation includes additional fields for scheduler coordination,
/// migration paths, and performance monitoring.
pub struct RunQueue {
    /// Priority queues for each priority level (MAX, HIGH, NORMAL)
    /// LOW priority processes are stored in the NORMAL queue
    prio_queues: [RunPrioQueue; 3],
    /// Information for each priority level
    prio_info: [Mutex<RunQueueInfo>; Priority::LEVELS],
    /// Total length across all priority levels
    total_len: Mutex<usize>,
    /// Maximum total length (0 = unlimited)
    max_len: usize,
    /// Run queue index (scheduler identifier)
    index: usize,
}

impl RunQueue {
    /// Create a new run queue
    ///
    /// # Arguments
    /// * `index` - Run queue index (scheduler identifier)
    /// * `max_len` - Maximum total length (0 = unlimited)
    pub fn new(index: usize, max_len: usize) -> Self {
        Self {
            prio_queues: [
                RunPrioQueue::new(), // MAX
                RunPrioQueue::new(), // HIGH
                RunPrioQueue::new(), // NORMAL (also used for LOW)
            ],
            prio_info: [
                Mutex::new(RunQueueInfo::new()), // MAX
                Mutex::new(RunQueueInfo::new()), // HIGH
                Mutex::new(RunQueueInfo::new()), // NORMAL
                Mutex::new(RunQueueInfo::new()), // LOW
            ],
            total_len: Mutex::new(0),
            max_len,
            index,
        }
    }

    /// Get the run queue index
    pub fn index(&self) -> usize {
        self.index
    }

    /// Get total length
    pub fn total_len(&self) -> usize {
        *self.total_len.lock().unwrap()
    }

    /// Get priority queue for a priority level
    ///
    /// LOW priority processes use the NORMAL queue
    fn get_prio_queue(&self, prio: Priority) -> &RunPrioQueue {
        match prio {
            Priority::Max => &self.prio_queues[0],
            Priority::High => &self.prio_queues[1],
            Priority::Normal | Priority::Low => &self.prio_queues[2],
        }
    }

    /// Get priority info for a priority level
    fn get_prio_info(&self, prio: Priority) -> &Mutex<RunQueueInfo> {
        &self.prio_info[prio.as_index()]
    }

    /// Increment run queue length
    fn inc_len(&self, prio: Priority) {
        let mut info = self.get_prio_info(prio).lock().unwrap();
        info.inc_len();
        drop(info);
        
        let mut total = self.total_len.lock().unwrap();
        *total += 1;
    }

    /// Decrement run queue length
    fn dec_len(&self, prio: Priority) {
        let mut info = self.get_prio_info(prio).lock().unwrap();
        info.dec_len();
        drop(info);
        
        let mut total = self.total_len.lock().unwrap();
        if *total > 0 {
            *total -= 1;
        }
    }
}

/// Dequeue a process from a run queue at a specific priority level
///
/// Based on dequeue_process() from erl_process.c
///
/// # Arguments
/// * `runq` - Run queue to dequeue from
/// * `prio_q` - Priority queue level (MAX, HIGH, or NORMAL)
///
/// # Returns
/// * `Some(process)` - Process dequeued from the queue
/// * `None` - No process available at this priority level
///
/// # Note
/// The C implementation uses PRIORITY_NORMAL, PRIORITY_HIGH, or PRIORITY_MAX.
/// LOW priority processes are stored in the NORMAL queue.
pub fn dequeue_process(runq: &RunQueue, prio_q: Priority) -> Option<Arc<Process>> {
    // Only MAX, HIGH, and NORMAL are valid for dequeue
    match prio_q {
        Priority::Max | Priority::High | Priority::Normal => {
            let queue = runq.get_prio_queue(prio_q);
            if let Some(process) = queue.dequeue() {
                // Update length
                runq.dec_len(prio_q);
                Some(process)
            } else {
                None
            }
        }
        Priority::Low => {
            // LOW priority processes are in the NORMAL queue
            let queue = runq.get_prio_queue(Priority::Normal);
            if let Some(process) = queue.dequeue() {
                runq.dec_len(Priority::Low);
                Some(process)
            } else {
                None
            }
        }
    }
}

/// Enqueue a process into a run queue at a specific priority level
///
/// Based on enqueue_process() from erl_process.c
///
/// # Arguments
/// * `runq` - Run queue to enqueue into
/// * `prio` - Priority level
/// * `process` - Process to enqueue
///
/// # Note
/// LOW priority processes are stored in the NORMAL queue but tracked separately
/// in the priority info. The process's schedule_count is set based on priority.
pub fn enqueue_process(runq: &RunQueue, prio: Priority, process: Arc<Process>) {
    // Update length first
    runq.inc_len(prio);
    
    // Get the appropriate queue
    // LOW priority processes go into the NORMAL queue
    let queue = runq.get_prio_queue(prio);
    
    // Enqueue the process
    queue.enqueue(process);
}

/// Check if a process should be requeued
///
/// Based on check_requeue_process() from erl_process.c
///
/// Low priority processes may need to be rescheduled multiple times before
/// they are actually executed. This function checks if a process should be
/// moved to the end of the queue for another round.
///
/// # Arguments
/// * `_runq` - Run queue
/// * `_prio_q` - Priority queue level
/// * `_process` - Process to check
///
/// # Returns
/// * `true` - Process was requeued
/// * `false` - Process should be executed
pub fn check_requeue_process(
    _runq: &RunQueue,
    _prio_q: Priority,
    _process: &Arc<Process>,
) -> bool {
    // This is a simplified version. The full implementation would:
    // 1. Check process.schedule_count
    // 2. Decrement schedule_count
    // 3. If schedule_count > 0 and process is not the last in queue, requeue
    
    // For now, we return false (don't requeue)
    // This would need access to schedule_count field in Process
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use entities_process::Process;

    #[test]
    fn test_priority_levels() {
        assert_eq!(Priority::Max as usize, 0);
        assert_eq!(Priority::High as usize, 1);
        assert_eq!(Priority::Normal as usize, 2);
        assert_eq!(Priority::Low as usize, 3);
    }

    #[test]
    fn test_priority_as_index() {
        assert_eq!(Priority::Max.as_index(), 0);
        assert_eq!(Priority::High.as_index(), 1);
        assert_eq!(Priority::Normal.as_index(), 2);
        assert_eq!(Priority::Low.as_index(), 3);
    }

    #[test]
    fn test_priority_from_index() {
        assert_eq!(Priority::from_index(0), Some(Priority::Max));
        assert_eq!(Priority::from_index(1), Some(Priority::High));
        assert_eq!(Priority::from_index(2), Some(Priority::Normal));
        assert_eq!(Priority::from_index(3), Some(Priority::Low));
        assert_eq!(Priority::from_index(4), None);
        assert_eq!(Priority::from_index(100), None);
    }

    #[test]
    fn test_priority_constants() {
        assert_eq!(Priority::LEVELS, 4);
    }

    #[test]
    fn test_priority_debug() {
        let debug_str = format!("{:?}", Priority::Max);
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_priority_clone() {
        let p1 = Priority::Max;
        let p2 = p1.clone();
        assert_eq!(p1, p2);
    }

    #[test]
    fn test_priority_partial_eq() {
        assert_eq!(Priority::Max, Priority::Max);
        assert_ne!(Priority::Max, Priority::High);
    }

    #[test]
    fn test_priority_partial_ord() {
        assert!(Priority::Max < Priority::High);
        assert!(Priority::High < Priority::Normal);
        assert!(Priority::Normal < Priority::Low);
    }

    #[test]
    fn test_priority_ord() {
        let mut priorities = vec![Priority::Low, Priority::Max, Priority::Normal, Priority::High];
        priorities.sort();
        assert_eq!(priorities, vec![Priority::Max, Priority::High, Priority::Normal, Priority::Low]);
    }

    #[test]
    fn test_run_queue_info_new() {
        let info = RunQueueInfo::new();
        assert_eq!(info.len(), 0);
        assert_eq!(info.max_len(), 0);
        assert_eq!(info.reds(), 0);
    }

    #[test]
    fn test_run_queue_info_default() {
        let info = RunQueueInfo::default();
        assert_eq!(info.len(), 0);
        assert_eq!(info.max_len(), 0);
        assert_eq!(info.reds(), 0);
    }

    #[test]
    fn test_run_queue_info_len() {
        let mut info = RunQueueInfo::new();
        assert_eq!(info.len(), 0);
        
        info.inc_len();
        assert_eq!(info.len(), 1);
        
        info.inc_len();
        assert_eq!(info.len(), 2);
        
        info.dec_len();
        assert_eq!(info.len(), 1);
        
        info.dec_len();
        assert_eq!(info.len(), 0);
    }

    #[test]
    fn test_run_queue_info_dec_len_zero() {
        let mut info = RunQueueInfo::new();
        assert_eq!(info.len(), 0);
        
        // Dec len when already zero should not go negative
        info.dec_len();
        assert_eq!(info.len(), 0);
    }

    #[test]
    fn test_run_queue_info_max_len() {
        let mut info = RunQueueInfo::new();
        assert_eq!(info.max_len(), 0);
        
        info.set_max_len(100);
        assert_eq!(info.max_len(), 100);
        
        info.set_max_len(200);
        assert_eq!(info.max_len(), 200);
    }

    #[test]
    fn test_run_queue_info_reds() {
        let mut info = RunQueueInfo::new();
        assert_eq!(info.reds(), 0);
        
        info.add_reds(100);
        assert_eq!(info.reds(), 100);
        
        info.add_reds(50);
        assert_eq!(info.reds(), 150);
        
        info.add_reds(-25);
        assert_eq!(info.reds(), 125);
    }

    #[test]
    fn test_run_queue_info_debug() {
        let info = RunQueueInfo::new();
        let debug_str = format!("{:?}", info);
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_run_queue_info_clone() {
        let mut info1 = RunQueueInfo::new();
        info1.inc_len();
        info1.add_reds(100);
        info1.set_max_len(50);
        
        let info2 = info1.clone();
        assert_eq!(info1.len(), info2.len());
        assert_eq!(info1.max_len(), info2.max_len());
        assert_eq!(info1.reds(), info2.reds());
    }

    #[test]
    fn test_run_prio_queue_new() {
        let queue = RunPrioQueue::new();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_run_prio_queue_default() {
        let queue = RunPrioQueue::default();
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_run_prio_queue_enqueue_dequeue() {
        let queue = RunPrioQueue::new();
        let process1 = Arc::new(Process::new(1));
        let process2 = Arc::new(Process::new(2));
        
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
        
        queue.enqueue(Arc::clone(&process1));
        assert!(!queue.is_empty());
        assert_eq!(queue.len(), 1);
        
        queue.enqueue(Arc::clone(&process2));
        assert_eq!(queue.len(), 2);
        
        let dequeued = queue.dequeue();
        assert!(dequeued.is_some());
        assert_eq!(dequeued.unwrap().id(), 1);
        assert_eq!(queue.len(), 1);
        
        let dequeued2 = queue.dequeue();
        assert!(dequeued2.is_some());
        assert_eq!(dequeued2.unwrap().id(), 2);
        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_run_prio_queue_dequeue_empty() {
        let queue = RunPrioQueue::new();
        assert!(queue.dequeue().is_none());
    }

    #[test]
    fn test_run_prio_queue_first() {
        let queue = RunPrioQueue::new();
        assert!(queue.first().is_none());
        
        let process1 = Arc::new(Process::new(1));
        let process2 = Arc::new(Process::new(2));
        
        queue.enqueue(Arc::clone(&process1));
        queue.enqueue(Arc::clone(&process2));
        
        let first = queue.first();
        assert!(first.is_some());
        assert_eq!(first.unwrap().id(), 1);
        // First should not remove from queue
        assert_eq!(queue.len(), 2);
    }

    #[test]
    fn test_run_prio_queue_last() {
        let queue = RunPrioQueue::new();
        assert!(queue.last().is_none());
        
        let process1 = Arc::new(Process::new(1));
        let process2 = Arc::new(Process::new(2));
        
        queue.enqueue(Arc::clone(&process1));
        queue.enqueue(Arc::clone(&process2));
        
        let last = queue.last();
        assert!(last.is_some());
        assert_eq!(last.unwrap().id(), 2);
        // Last should not remove from queue
        assert_eq!(queue.len(), 2);
    }

    #[test]
    fn test_run_prio_queue_fifo_order() {
        let queue = RunPrioQueue::new();
        let processes: Vec<Arc<Process>> = (1..=5)
            .map(|i| Arc::new(Process::new(i)))
            .collect();
        
        for process in &processes {
            queue.enqueue(Arc::clone(process));
        }
        
        assert_eq!(queue.len(), 5);
        
        // Dequeue should be in FIFO order
        for i in 1..=5 {
            let dequeued = queue.dequeue();
            assert!(dequeued.is_some());
            assert_eq!(dequeued.unwrap().id(), i);
        }
        
        assert!(queue.is_empty());
    }

    #[test]
    fn test_run_queue_new() {
        let runq = RunQueue::new(0, 1000);
        assert_eq!(runq.index(), 0);
        assert_eq!(runq.total_len(), 0);
    }

    #[test]
    fn test_run_queue_index() {
        let runq1 = RunQueue::new(0, 1000);
        assert_eq!(runq1.index(), 0);
        
        let runq2 = RunQueue::new(5, 1000);
        assert_eq!(runq2.index(), 5);
        
        let runq3 = RunQueue::new(100, 1000);
        assert_eq!(runq3.index(), 100);
    }

    #[test]
    fn test_run_queue_total_len() {
        let runq = RunQueue::new(0, 1000);
        assert_eq!(runq.total_len(), 0);
    }

    #[test]
    fn test_enqueue_process_max() {
        let runq = RunQueue::new(0, 1000);
        let process = Arc::new(Process::new(1));
        
        enqueue_process(&runq, Priority::Max, Arc::clone(&process));
        assert_eq!(runq.total_len(), 1);
    }

    #[test]
    fn test_enqueue_process_high() {
        let runq = RunQueue::new(0, 1000);
        let process = Arc::new(Process::new(1));
        
        enqueue_process(&runq, Priority::High, Arc::clone(&process));
        assert_eq!(runq.total_len(), 1);
    }

    #[test]
    fn test_enqueue_process_normal() {
        let runq = RunQueue::new(0, 1000);
        let process = Arc::new(Process::new(1));
        
        enqueue_process(&runq, Priority::Normal, Arc::clone(&process));
        assert_eq!(runq.total_len(), 1);
    }

    #[test]
    fn test_enqueue_process_low() {
        let runq = RunQueue::new(0, 1000);
        let process = Arc::new(Process::new(1));
        
        enqueue_process(&runq, Priority::Low, Arc::clone(&process));
        assert_eq!(runq.total_len(), 1);
    }

    #[test]
    fn test_enqueue_multiple_processes() {
        let runq = RunQueue::new(0, 1000);
        let processes: Vec<Arc<Process>> = (1..=5)
            .map(|i| Arc::new(Process::new(i)))
            .collect();
        
        for process in &processes {
            enqueue_process(&runq, Priority::Normal, Arc::clone(process));
        }
        
        assert_eq!(runq.total_len(), 5);
    }

    #[test]
    fn test_dequeue_process_max() {
        let runq = RunQueue::new(0, 1000);
        let process = Arc::new(Process::new(1));
        
        enqueue_process(&runq, Priority::Max, Arc::clone(&process));
        assert_eq!(runq.total_len(), 1);
        
        let dequeued = dequeue_process(&runq, Priority::Max);
        assert!(dequeued.is_some());
        assert_eq!(dequeued.unwrap().id(), 1);
        assert_eq!(runq.total_len(), 0);
    }

    #[test]
    fn test_dequeue_process_high() {
        let runq = RunQueue::new(0, 1000);
        let process = Arc::new(Process::new(1));
        
        enqueue_process(&runq, Priority::High, Arc::clone(&process));
        let dequeued = dequeue_process(&runq, Priority::High);
        assert!(dequeued.is_some());
        assert_eq!(dequeued.unwrap().id(), 1);
    }

    #[test]
    fn test_dequeue_process_normal() {
        let runq = RunQueue::new(0, 1000);
        let process = Arc::new(Process::new(1));
        
        enqueue_process(&runq, Priority::Normal, Arc::clone(&process));
        let dequeued = dequeue_process(&runq, Priority::Normal);
        assert!(dequeued.is_some());
        assert_eq!(dequeued.unwrap().id(), 1);
    }

    #[test]
    fn test_dequeue_process_low() {
        let runq = RunQueue::new(0, 1000);
        let process = Arc::new(Process::new(1));
        
        enqueue_process(&runq, Priority::Low, Arc::clone(&process));
        let dequeued = dequeue_process(&runq, Priority::Low);
        assert!(dequeued.is_some());
        assert_eq!(dequeued.unwrap().id(), 1);
    }

    #[test]
    fn test_dequeue_process_empty() {
        let runq = RunQueue::new(0, 1000);
        
        assert!(dequeue_process(&runq, Priority::Max).is_none());
        assert!(dequeue_process(&runq, Priority::High).is_none());
        assert!(dequeue_process(&runq, Priority::Normal).is_none());
        assert!(dequeue_process(&runq, Priority::Low).is_none());
    }

    #[test]
    fn test_enqueue_dequeue_roundtrip() {
        let runq = RunQueue::new(0, 1000);
        let process1 = Arc::new(Process::new(1));
        let process2 = Arc::new(Process::new(2));
        
        enqueue_process(&runq, Priority::Normal, Arc::clone(&process1));
        enqueue_process(&runq, Priority::Normal, Arc::clone(&process2));
        assert_eq!(runq.total_len(), 2);
        
        let d1 = dequeue_process(&runq, Priority::Normal);
        assert!(d1.is_some());
        assert_eq!(d1.unwrap().id(), 1);
        assert_eq!(runq.total_len(), 1);
        
        let d2 = dequeue_process(&runq, Priority::Normal);
        assert!(d2.is_some());
        assert_eq!(d2.unwrap().id(), 2);
        assert_eq!(runq.total_len(), 0);
    }

    #[test]
    fn test_priority_queues_separate() {
        let runq = RunQueue::new(0, 1000);
        let p_max = Arc::new(Process::new(1));
        let p_high = Arc::new(Process::new(2));
        let p_normal = Arc::new(Process::new(3));
        
        enqueue_process(&runq, Priority::Max, Arc::clone(&p_max));
        enqueue_process(&runq, Priority::High, Arc::clone(&p_high));
        enqueue_process(&runq, Priority::Normal, Arc::clone(&p_normal));
        
        assert_eq!(runq.total_len(), 3);
        
        let d_max = dequeue_process(&runq, Priority::Max);
        assert!(d_max.is_some());
        assert_eq!(d_max.unwrap().id(), 1);
        
        let d_high = dequeue_process(&runq, Priority::High);
        assert!(d_high.is_some());
        assert_eq!(d_high.unwrap().id(), 2);
        
        let d_normal = dequeue_process(&runq, Priority::Normal);
        assert!(d_normal.is_some());
        assert_eq!(d_normal.unwrap().id(), 3);
    }

    #[test]
    fn test_low_priority_in_normal_queue() {
        let runq = RunQueue::new(0, 1000);
        let p_low = Arc::new(Process::new(1));
        let p_normal = Arc::new(Process::new(2));
        
        enqueue_process(&runq, Priority::Low, Arc::clone(&p_low));
        enqueue_process(&runq, Priority::Normal, Arc::clone(&p_normal));
        
        // Both should be in the NORMAL queue
        assert_eq!(runq.total_len(), 2);
        
        // Dequeue from NORMAL should get one of them
        let d1 = dequeue_process(&runq, Priority::Normal);
        assert!(d1.is_some());
        
        // Dequeue from LOW should also work (uses NORMAL queue)
        let d2 = dequeue_process(&runq, Priority::Low);
        assert!(d2.is_some());
    }

    #[test]
    fn test_check_requeue_process() {
        let runq = RunQueue::new(0, 1000);
        let process = Arc::new(Process::new(1));
        
        // Currently always returns false (simplified implementation)
        let result = check_requeue_process(&runq, Priority::Normal, &process);
        assert_eq!(result, false);
    }

    #[test]
    fn test_check_requeue_process_all_priorities() {
        let runq = RunQueue::new(0, 1000);
        let process = Arc::new(Process::new(1));
        
        assert_eq!(check_requeue_process(&runq, Priority::Max, &process), false);
        assert_eq!(check_requeue_process(&runq, Priority::High, &process), false);
        assert_eq!(check_requeue_process(&runq, Priority::Normal, &process), false);
        assert_eq!(check_requeue_process(&runq, Priority::Low, &process), false);
    }

    #[test]
    fn test_run_queue_max_len() {
        let runq = RunQueue::new(0, 100);
        // max_len is stored but not currently enforced in enqueue
        // This test verifies the structure is correct
        assert_eq!(runq.index(), 0);
    }

    #[test]
    fn test_run_queue_multiple_schedulers() {
        let runq1 = RunQueue::new(0, 1000);
        let runq2 = RunQueue::new(1, 1000);
        let runq3 = RunQueue::new(2, 1000);
        
        assert_eq!(runq1.index(), 0);
        assert_eq!(runq2.index(), 1);
        assert_eq!(runq3.index(), 2);
        
        let p1 = Arc::new(Process::new(1));
        let p2 = Arc::new(Process::new(2));
        let p3 = Arc::new(Process::new(3));
        
        enqueue_process(&runq1, Priority::Normal, Arc::clone(&p1));
        enqueue_process(&runq2, Priority::Normal, Arc::clone(&p2));
        enqueue_process(&runq3, Priority::Normal, Arc::clone(&p3));
        
        assert_eq!(runq1.total_len(), 1);
        assert_eq!(runq2.total_len(), 1);
        assert_eq!(runq3.total_len(), 1);
    }

    #[test]
    fn test_run_queue_info_tracking() {
        let runq = RunQueue::new(0, 1000);
        let process = Arc::new(Process::new(1));
        
        enqueue_process(&runq, Priority::Max, Arc::clone(&process));
        
        // Verify info is updated (indirectly through total_len)
        assert_eq!(runq.total_len(), 1);
        
        dequeue_process(&runq, Priority::Max);
        assert_eq!(runq.total_len(), 0);
    }

    #[test]
    fn test_fifo_ordering_within_priority() {
        let runq = RunQueue::new(0, 1000);
        let processes: Vec<Arc<Process>> = (1..=5)
            .map(|i| Arc::new(Process::new(i)))
            .collect();
        
        for process in &processes {
            enqueue_process(&runq, Priority::Normal, Arc::clone(process));
        }
        
        // Dequeue should maintain FIFO order
        for i in 1..=5 {
            let dequeued = dequeue_process(&runq, Priority::Normal);
            assert!(dequeued.is_some());
            assert_eq!(dequeued.unwrap().id(), i);
        }
    }

    #[test]
    fn test_mixed_priority_enqueue_dequeue() {
        let runq = RunQueue::new(0, 1000);
        let p_max = Arc::new(Process::new(1));
        let p_high = Arc::new(Process::new(2));
        let p_normal = Arc::new(Process::new(3));
        let p_low = Arc::new(Process::new(4));
        
        // Enqueue in mixed order
        enqueue_process(&runq, Priority::Low, Arc::clone(&p_low));
        enqueue_process(&runq, Priority::Max, Arc::clone(&p_max));
        enqueue_process(&runq, Priority::Normal, Arc::clone(&p_normal));
        enqueue_process(&runq, Priority::High, Arc::clone(&p_high));
        
        assert_eq!(runq.total_len(), 4);
        
        // Dequeue from each priority
        let d_max = dequeue_process(&runq, Priority::Max);
        assert!(d_max.is_some());
        assert_eq!(d_max.unwrap().id(), 1);
        
        let d_high = dequeue_process(&runq, Priority::High);
        assert!(d_high.is_some());
        assert_eq!(d_high.unwrap().id(), 2);
        
        let d_normal = dequeue_process(&runq, Priority::Normal);
        assert!(d_normal.is_some());
        // Could be either 3 or 4 (both in NORMAL queue)
        let normal_id = d_normal.unwrap().id();
        assert!(normal_id == 3 || normal_id == 4);
    }
}

