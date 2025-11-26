//! Time Supervisor Module
//!
//! Provides time supervision functionality.
//! Based on erl_time_sup.c

use std::time::{SystemTime, UNIX_EPOCH};

/// Time supervisor
pub struct TimeSup;

impl TimeSup {
    /// Get current system time in microseconds
    pub fn now_micros() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64
    }

    /// Get current system time in milliseconds
    pub fn now_millis() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_sup() {
        let now1 = TimeSup::now_micros();
        let now2 = TimeSup::now_micros();
        assert!(now2 >= now1);
    }
}

