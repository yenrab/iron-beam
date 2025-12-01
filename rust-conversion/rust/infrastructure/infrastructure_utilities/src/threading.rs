//! Threading Utilities
//!
//! Provides threading utility functions based on ethr_*.c files.
//! These utilities handle thread-related operations.

use std::sync::{Arc, Mutex, RwLock};
use std::thread;

/// Threading utilities for thread-related operations
pub struct ThreadingUtils;

impl ThreadingUtils {
    /// Execute a function in a new thread
    ///
    /// # Arguments
    /// * `f` - Function to execute
    ///
    /// # Returns
    /// Thread handle
    pub fn spawn<F, T>(f: F) -> thread::JoinHandle<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        thread::spawn(f)
    }

    /// Get the number of available CPU cores
    ///
    /// # Returns
    /// Number of CPU cores
    pub fn num_cpus() -> usize {
        num_cpus::get()
    }

    /// Sleep for a specified number of milliseconds
    ///
    /// # Arguments
    /// * `ms` - Milliseconds to sleep
    pub fn sleep_ms(ms: u64) {
        std::thread::sleep(std::time::Duration::from_millis(ms));
    }

    /// Sleep for a specified number of seconds
    ///
    /// # Arguments
    /// * `secs` - Seconds to sleep
    pub fn sleep_secs(secs: u64) {
        std::thread::sleep(std::time::Duration::from_secs(secs));
    }

    /// Create a new mutex
    ///
    /// # Arguments
    /// * `value` - Value to wrap in mutex
    ///
    /// # Returns
    /// Mutex-wrapped value
    pub fn new_mutex<T>(value: T) -> Arc<Mutex<T>> {
        Arc::new(Mutex::new(value))
    }

    /// Create a new read-write lock
    ///
    /// # Arguments
    /// * `value` - Value to wrap in rwlock
    ///
    /// # Returns
    /// RwLock-wrapped value
    pub fn new_rwlock<T>(value: T) -> Arc<RwLock<T>> {
        Arc::new(RwLock::new(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn() {
        let handle = ThreadingUtils::spawn(|| 42);
        let result = handle.join().unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_num_cpus() {
        let cpus = ThreadingUtils::num_cpus();
        assert!(cpus > 0);
    }

    #[test]
    fn test_new_mutex() {
        let mutex = ThreadingUtils::new_mutex(42);
        let value = mutex.lock().unwrap();
        assert_eq!(*value, 42);
    }

    #[test]
    fn test_new_rwlock() {
        let rwlock = ThreadingUtils::new_rwlock(42);
        let value = rwlock.read().unwrap();
        assert_eq!(*value, 42);
    }
}

