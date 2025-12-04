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
    fn test_time_sup() {
        let now1 = TimeSup::now_micros();
        let now2 = TimeSup::now_micros();
        assert!(now2 >= now1);
    }
}

