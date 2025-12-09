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

    #[test]
    fn test_priority_levels() {
        assert_eq!(Priority::Max as usize, 0);
        assert_eq!(Priority::High as usize, 1);
        assert_eq!(Priority::Normal as usize, 2);
        assert_eq!(Priority::Low as usize, 3);
    }

    #[test]
    fn test_run_queue_info() {
        let mut info = RunQueueInfo::new();
        assert_eq!(info.len(), 0);
        
        info.inc_len();
        assert_eq!(info.len(), 1);
        
        info.dec_len();
        assert_eq!(info.len(), 0);
        
        info.add_reds(100);
        assert_eq!(info.reds(), 100);
    }

    #[test]
    fn test_prio_queue() {
        let queue = RunPrioQueue::new();
        assert!(queue.is_empty());
        
        // Create a dummy process (would need Process::new in real implementation)
        // For now, we'll test the queue structure
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn test_run_queue() {
        let runq = RunQueue::new(0, 1000);
        assert_eq!(runq.index(), 0);
        assert_eq!(runq.total_len(), 0);
    }
}

