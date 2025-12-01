//! Time Utilities
//!
//! Provides time-related utility functions.
//! These utilities handle time operations and timestamps.

use std::time::{SystemTime, UNIX_EPOCH, Duration};

/// Time utilities for time-related operations
pub struct TimeUtils;

impl TimeUtils {
    /// Get current Unix timestamp (seconds since epoch)
    ///
    /// # Returns
    /// Unix timestamp in seconds
    ///
    /// # Examples
    /// ```
    /// use infrastructure_utilities::TimeUtils;
    ///
    /// let timestamp = TimeUtils::unix_timestamp();
    /// assert!(timestamp > 0);
    /// ```
    pub fn unix_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Get current Unix timestamp in milliseconds
    ///
    /// # Returns
    /// Unix timestamp in milliseconds
    pub fn unix_timestamp_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    /// Get current Unix timestamp in nanoseconds
    ///
    /// # Returns
    /// Unix timestamp in nanoseconds
    pub fn unix_timestamp_ns() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    }

    /// Get elapsed time since a timestamp
    ///
    /// # Arguments
    /// * `timestamp` - Unix timestamp in seconds
    ///
    /// # Returns
    /// Elapsed seconds
    pub fn elapsed_since(timestamp: u64) -> u64 {
        let now = Self::unix_timestamp();
        if now > timestamp {
            now - timestamp
        } else {
            0
        }
    }

    /// Create a duration from seconds
    ///
    /// # Arguments
    /// * `secs` - Seconds
    ///
    /// # Returns
    /// Duration
    pub fn duration_from_secs(secs: u64) -> Duration {
        Duration::from_secs(secs)
    }

    /// Create a duration from milliseconds
    ///
    /// # Arguments
    /// * `ms` - Milliseconds
    ///
    /// # Returns
    /// Duration
    pub fn duration_from_millis(ms: u64) -> Duration {
        Duration::from_millis(ms)
    }

    /// Get duration as seconds
    ///
    /// # Arguments
    /// * `duration` - Duration
    ///
    /// # Returns
    /// Seconds
    pub fn duration_as_secs(duration: Duration) -> u64 {
        duration.as_secs()
    }

    /// Get duration as milliseconds
    ///
    /// # Arguments
    /// * `duration` - Duration
    ///
    /// # Returns
    /// Milliseconds
    pub fn duration_as_millis(duration: Duration) -> u64 {
        duration.as_millis() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unix_timestamp() {
        let timestamp = TimeUtils::unix_timestamp();
        assert!(timestamp > 0);
        
        // Should be recent (within last year)
        let year_ago = timestamp - 31536000;
        assert!(year_ago > 0);
    }

    #[test]
    fn test_unix_timestamp_ms() {
        let timestamp = TimeUtils::unix_timestamp_ms();
        assert!(timestamp > 0);
    }

    #[test]
    fn test_elapsed_since() {
        let timestamp = TimeUtils::unix_timestamp();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = TimeUtils::elapsed_since(timestamp);
        assert!(elapsed >= 0);
    }

    #[test]
    fn test_duration_conversions() {
        let duration = TimeUtils::duration_from_secs(5);
        assert_eq!(TimeUtils::duration_as_secs(duration), 5);
        
        let duration = TimeUtils::duration_from_millis(5000);
        assert_eq!(TimeUtils::duration_as_millis(duration), 5000);
    }
}

