//! Queue Management
//!
//! Provides queue management functions for driver I/O operations.

use super::types::*;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

/// Output queue for a driver port
struct OutputQueue {
    data: VecDeque<u8>,
}

impl OutputQueue {
    fn new() -> Self {
        Self {
            data: VecDeque::new(),
        }
    }

    fn enqueue(&mut self, buf: &[u8]) {
        self.data.extend(buf);
    }

    fn enqueue_vec(&mut self, iov: &DriverIOVec, skip: DriverSizeT) {
        let mut skipped = 0;
        for vec in &iov.iov {
            if skipped + vec.len <= skip {
                skipped += vec.len;
                continue;
            }
            let start = if skipped < skip { skip - skipped } else { 0 };
            if start < vec.len {
                unsafe {
                    let slice = std::slice::from_raw_parts(vec.base.add(start), vec.len - start);
                    self.data.extend(slice);
                }
            }
            skipped += vec.len;
        }
    }

    fn dequeue(&mut self, size: DriverSizeT) -> DriverSizeT {
        let dequeue_size = size.min(self.data.len());
        for _ in 0..dequeue_size {
            self.data.pop_front();
        }
        dequeue_size
    }

    fn size(&self) -> DriverSizeT {
        self.data.len()
    }

    fn peek_vec(&self, max_vecs: usize) -> Option<(Vec<IoVec>, usize)> {
        if self.data.is_empty() {
            return None;
        }

        let mut iov = Vec::new();
        let (slice1, slice2) = self.data.as_slices();
        let mut total_size = 0;

        // Process first slice
        let mut remaining = slice1;
        while !remaining.is_empty() && iov.len() < max_vecs {
            let len = remaining.len();
            let ptr = remaining.as_ptr() as *mut u8;
            iov.push(IoVec {
                base: ptr,
                len,
            });
            total_size += len;
            break; // Take the whole slice
        }

        // Process second slice if available
        if !slice2.is_empty() && iov.len() < max_vecs {
            let len = slice2.len();
            let ptr = slice2.as_ptr() as *mut u8;
            iov.push(IoVec {
                base: ptr,
                len,
            });
            total_size += len;
        }

        if iov.is_empty() {
            None
        } else {
            Some((iov, total_size))
        }
    }
}

lazy_static::lazy_static! {
    static ref OUTPUT_QUEUES: Arc<Mutex<std::collections::HashMap<u64, OutputQueue>>> = 
        Arc::new(Mutex::new(std::collections::HashMap::new()));
}

// Queue access is handled directly through OUTPUT_QUEUES

/// Enqueue data to the output queue
///
/// Equivalent to C's `driver_enq`.
pub fn driver_enq(port: DriverPort, buf: &[u8]) -> Result<(), ()> {
    let mut queues = OUTPUT_QUEUES.lock().unwrap();
    let queue = queues.entry(port.id()).or_insert_with(OutputQueue::new);
    queue.enqueue(buf);
    Ok(())
}

/// Dequeue data from the output queue
///
/// Equivalent to C's `driver_deq`.
pub fn driver_deq(port: DriverPort, size: DriverSizeT) -> DriverSizeT {
    let mut queues = OUTPUT_QUEUES.lock().unwrap();
    if let Some(queue) = queues.get_mut(&port.id()) {
        queue.dequeue(size)
    } else {
        0
    }
}

/// Get the size of the output queue
///
/// Equivalent to C's `driver_sizeq`.
pub fn driver_sizeq(port: DriverPort) -> DriverSizeT {
    let queues = OUTPUT_QUEUES.lock().unwrap();
    if let Some(queue) = queues.get(&port.id()) {
        queue.size()
    } else {
        0
    }
}

/// Peek at the output queue as I/O vectors
///
/// Equivalent to C's `driver_peekq`.
pub fn driver_peekq(port: DriverPort) -> Option<(Vec<IoVec>, usize)> {
    let queues = OUTPUT_QUEUES.lock().unwrap();
    if let Some(queue) = queues.get(&port.id()) {
        queue.peek_vec(16) // MAX_VSIZE from C code
    } else {
        None
    }
}

/// Enqueue I/O vectors to the output queue
///
/// Equivalent to C's `driver_enqv`.
pub fn driver_enqv(port: DriverPort, iov: &DriverIOVec, skip: DriverSizeT) -> Result<(), ()> {
    let mut queues = OUTPUT_QUEUES.lock().unwrap();
    let queue = queues.entry(port.id()).or_insert_with(OutputQueue::new);
    queue.enqueue_vec(iov, skip);
    Ok(())
}

