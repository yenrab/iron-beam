//! Time Slice Module
//!
//! Provides time slice management.
//! Based on consume_timeslice_drv.c

/// Time slice manager
pub struct TimeSlice {
    remaining: u64, // Remaining time in microseconds
}

impl TimeSlice {
    /// Create a new time slice
    pub fn new(initial: u64) -> Self {
        Self {
            remaining: initial,
        }
    }

    /// Consume time from slice
    pub fn consume(&mut self, amount: u64) {
        if amount > self.remaining {
            self.remaining = 0;
        } else {
            self.remaining -= amount;
        }
    }

    /// Get remaining time
    pub fn remaining(&self) -> u64 {
        self.remaining
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeslice() {
        let mut slice = TimeSlice::new(1000);
        assert_eq!(slice.remaining(), 1000);
        slice.consume(300);
        assert_eq!(slice.remaining(), 700);
    }
}

