//! Time Supervisor Module
//!
//! Provides time supervision functionality for the Erlang/OTP runtime system. This module
//! manages time-related operations and ensures time consistency across the runtime,
//! providing high-precision time measurements for scheduling and monitoring.
//!
//! ## Overview
//!
//! The time supervisor provides:
//! - **High-precision time**: Microsecond and millisecond precision time measurements
//! - **System time access**: Direct access to system time for scheduling
//! - **Time consistency**: Ensures consistent time across the runtime
//!
//! ## Examples
//!
//! ```rust
//! use infrastructure_time_management::TimeSup;
//!
//! // Get current time in microseconds
//! let now_micros = TimeSup::now_micros();
//!
//! // Get current time in milliseconds
//! let now_millis = TimeSup::now_millis();
//! ```
//!
//! ## See Also
//!
//! - [`adapters_time_management`](../../adapters/adapters_time_management/index.html): Time management adapters
//! - [`entities_data_handling`](../../entities/entities_data_handling/index.html): Term types for time operations
//!
//! Based on `erl_time_sup.c`

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
    fn test_time_sup_now_micros() {
        let now1 = TimeSup::now_micros();
        let now2 = TimeSup::now_micros();
        assert!(now2 >= now1, "Time should be monotonic");
    }

    #[test]
    fn test_time_sup_now_millis() {
        let now1 = TimeSup::now_millis();
        let now2 = TimeSup::now_millis();
        assert!(now2 >= now1, "Time should be monotonic");
    }

    #[test]
    fn test_time_sup_micros_vs_millis() {
        let micros = TimeSup::now_micros();
        let millis = TimeSup::now_millis();
        // Millis should be approximately micros / 1000 (within 1ms tolerance)
        let millis_from_micros = micros / 1000;
        assert!((millis as i64 - millis_from_micros as i64).abs() <= 1,
            "Millis should be approximately micros / 1000");
    }

    #[test]
    fn test_time_sup_multiple_calls_micros() {
        let mut times = Vec::new();
        for _ in 0..10 {
            times.push(TimeSup::now_micros());
        }
        // Verify times are non-decreasing
        for i in 1..times.len() {
            assert!(times[i] >= times[i-1], "Time should be non-decreasing");
        }
    }

    #[test]
    fn test_time_sup_multiple_calls_millis() {
        let mut times = Vec::new();
        for _ in 0..10 {
            times.push(TimeSup::now_millis());
        }
        // Verify times are non-decreasing
        for i in 1..times.len() {
            assert!(times[i] >= times[i-1], "Time should be non-decreasing");
        }
    }

    #[test]
    fn test_time_sup_micros_is_positive() {
        let time = TimeSup::now_micros();
        assert!(time > 0, "Time should be positive (Unix epoch)");
    }

    #[test]
    fn test_time_sup_millis_is_positive() {
        let time = TimeSup::now_millis();
        assert!(time > 0, "Time should be positive (Unix epoch)");
    }

    #[test]
    fn test_time_sup_consistency() {
        // Test that calling both methods gives consistent results
        let micros1 = TimeSup::now_micros();
        let millis = TimeSup::now_millis();
        let micros2 = TimeSup::now_micros();
        
        // All times should be in order
        assert!(micros2 >= micros1);
        // Millis should be between the two micros calls (within 1ms)
        let millis_micros = millis * 1000;
        assert!(millis_micros >= micros1.saturating_sub(1000));
        assert!(millis_micros <= micros2.saturating_add(1000));
    }
}

