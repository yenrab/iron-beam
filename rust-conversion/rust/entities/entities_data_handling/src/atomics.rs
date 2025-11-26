//! Atomic Operations Module
//!
//! Provides atomic operations for double-word atomics.
//! Based on ethr_atomics.c

use std::sync::atomic::{AtomicU64, Ordering};

/// Double-word atomic operations
pub struct DoubleWordAtomic {
    value: AtomicU64,
}

impl DoubleWordAtomic {
    /// Create a new double-word atomic
    pub fn new(value: u64) -> Self {
        Self {
            value: AtomicU64::new(value),
        }
    }

    /// Compare and exchange
    pub fn compare_exchange(
        &self,
        current: u64,
        new: u64,
        success: Ordering,
        failure: Ordering,
    ) -> Result<u64, u64> {
        self.value.compare_exchange(current, new, success, failure)
    }

    /// Load value
    pub fn load(&self, order: Ordering) -> u64 {
        self.value.load(order)
    }

    /// Store value
    pub fn store(&self, value: u64, order: Ordering) {
        self.value.store(value, order);
    }
}

/// Check if native double-word atomics are available
pub fn have_native_dw_atomic() -> bool {
    // Rust's AtomicU64 provides native support on 64-bit platforms
    cfg!(target_pointer_width = "64")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    #[test]
    fn test_dw_atomic_operations() {
        let atomic = DoubleWordAtomic::new(0);
        assert_eq!(atomic.load(Ordering::SeqCst), 0);
        atomic.store(42, Ordering::SeqCst);
        assert_eq!(atomic.load(Ordering::SeqCst), 42);
    }

    #[test]
    fn test_compare_exchange() {
        let atomic = DoubleWordAtomic::new(10);
        let result = atomic.compare_exchange(10, 20, Ordering::SeqCst, Ordering::SeqCst);
        assert_eq!(result, Ok(10));
        assert_eq!(atomic.load(Ordering::SeqCst), 20);
    }
}

