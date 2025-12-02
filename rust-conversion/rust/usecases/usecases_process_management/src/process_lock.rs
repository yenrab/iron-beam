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
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_process_lock() {
        let lock = ProcessLock::new();
        lock.acquire(1, 0);
        lock.release(0);
    }

    #[test]
    fn test_process_lock_multiple_locks() {
        let lock = ProcessLock::new();
        // Acquire multiple different locks
        lock.acquire(1, 0);
        lock.acquire(2, 1);
        lock.acquire(3, 2);
        
        // Release them
        lock.release(0);
        lock.release(1);
        lock.release(2);
    }

    #[test]
    fn test_process_lock_release_nonexistent() {
        let lock = ProcessLock::new();
        // Release a lock that was never acquired (should not panic)
        lock.release(999);
    }

    #[test]
    fn test_process_lock_reacquire_after_release() {
        let lock = ProcessLock::new();
        // Acquire, release, then reacquire
        lock.acquire(1, 0);
        lock.release(0);
        lock.acquire(1, 0);
        lock.release(0);
    }

    #[test]
    fn test_process_lock_concurrent_access() {
        let lock = Arc::new(ProcessLock::new());
        let lock_clone = Arc::clone(&lock);
        
        // Spawn a thread that acquires and releases a lock
        let handle = thread::spawn(move || {
            lock_clone.acquire(1, 0);
            thread::sleep(Duration::from_millis(10));
            lock_clone.release(0);
        });

        // Main thread waits a bit then acquires the same lock
        thread::sleep(Duration::from_millis(5));
        lock.acquire(2, 0);
        lock.release(0);
        
        handle.join().unwrap();
    }

    #[test]
    fn test_process_lock_waiting_path() {
        let lock = Arc::new(ProcessLock::new());
        let lock_clone = Arc::clone(&lock);
        
        // First thread acquires lock and holds it briefly
        let handle1 = thread::spawn(move || {
            lock_clone.acquire(1, 0);
            thread::sleep(Duration::from_millis(50));
            lock_clone.release(0);
        });

        // Second thread tries to acquire the same lock (will wait)
        let lock_clone2 = Arc::clone(&lock);
        let handle2 = thread::spawn(move || {
            thread::sleep(Duration::from_millis(10));
            lock_clone2.acquire(2, 0);
            lock_clone2.release(0);
        });

        handle1.join().unwrap();
        handle2.join().unwrap();
    }

    #[test]
    fn test_process_lock_multiple_waiters() {
        let lock = Arc::new(ProcessLock::new());
        let lock_clone = Arc::clone(&lock);
        
        // First thread acquires and holds the lock
        let handle1 = thread::spawn(move || {
            lock_clone.acquire(1, 0);
            thread::sleep(Duration::from_millis(100));
            lock_clone.release(0);
        });

        // Multiple threads wait for the lock
        let mut handles = Vec::new();
        for i in 2..=4 {
            let lock_clone = Arc::clone(&lock);
            let handle = thread::spawn(move || {
                thread::sleep(Duration::from_millis(10));
                lock_clone.acquire(i, 0);
                lock_clone.release(0);
            });
            handles.push(handle);
        }

        handle1.join().unwrap();
        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_process_lock_different_lock_ids() {
        let lock = Arc::new(ProcessLock::new());
        
        // Acquire different locks concurrently - should not block each other
        let lock_clone1 = Arc::clone(&lock);
        let handle1 = thread::spawn(move || {
            lock_clone1.acquire(1, 0);
            thread::sleep(Duration::from_millis(50));
            lock_clone1.release(0);
        });

        let lock_clone2 = Arc::clone(&lock);
        let handle2 = thread::spawn(move || {
            lock_clone2.acquire(2, 1); // Different lock ID
            thread::sleep(Duration::from_millis(50));
            lock_clone2.release(1);
        });

        handle1.join().unwrap();
        handle2.join().unwrap();
    }
}

