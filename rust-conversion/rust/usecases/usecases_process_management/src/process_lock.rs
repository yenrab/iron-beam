//! Process Lock Module
//!
//! Provides process-level locking functionality.
//! Based on erl_process_lock.c

use std::sync::{Mutex, Condvar};
use std::collections::HashMap;

/// Process lock implementation
pub struct ProcessLock {
    locks: Mutex<HashMap<u32, LockState>>,
    waiters: Condvar,
}

struct LockState {
    locked: bool,
    waiters: Vec<u32>, // Process IDs waiting on this lock
}

impl ProcessLock {
    /// Create a new process lock manager
    pub fn new() -> Self {
        Self {
            locks: Mutex::new(HashMap::new()),
            waiters: Condvar::new(),
        }
    }

    /// Acquire a lock for a process
    ///
    /// # Arguments
    /// * `process_id` - Process ID
    /// * `lock_id` - Lock identifier
    pub fn acquire(&self, process_id: u32, lock_id: u32) {
        let mut locks = self.locks.lock().unwrap();
        let mut state = locks.entry(lock_id).or_insert_with(|| LockState {
            locked: false,
            waiters: Vec::new(),
        });

        if state.locked {
            // Lock is held, add to wait queue
            state.waiters.push(process_id);
            // Wait for lock to be released
            while state.locked {
                locks = self.waiters.wait(locks).unwrap();
                state = locks.get_mut(&lock_id).unwrap();
            }
            // Remove from wait queue
            state.waiters.retain(|&id| id != process_id);
        }

        state.locked = true;
    }

    /// Release a lock
    ///
    /// # Arguments
    /// * `lock_id` - Lock identifier
    pub fn release(&self, lock_id: u32) {
        let mut locks = self.locks.lock().unwrap();
        if let Some(state) = locks.get_mut(&lock_id) {
            state.locked = false;
            self.waiters.notify_all();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_lock() {
        let lock = ProcessLock::new();
        lock.acquire(1, 0);
        lock.release(0);
    }
}

