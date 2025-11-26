//! Timer Module
//!
//! Provides timer functionality.
//! Based on timer_drv.c

use std::time::{Duration, Instant};

/// Timer implementation
pub struct Timer {
    start: Instant,
    duration: Duration,
}

impl Timer {
    /// Create a new timer with duration
    pub fn new(duration: Duration) -> Self {
        Self {
            start: Instant::now(),
            duration,
        }
    }

    /// Check if timer has expired
    pub fn expired(&self) -> bool {
        self.start.elapsed() >= self.duration
    }

    /// Get remaining time
    pub fn remaining(&self) -> Duration {
        let elapsed = self.start.elapsed();
        if elapsed >= self.duration {
            Duration::ZERO
        } else {
            self.duration - elapsed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer() {
        let timer = Timer::new(Duration::from_millis(100));
        assert!(!timer.expired());
        assert!(timer.remaining() > Duration::ZERO);
    }
}

