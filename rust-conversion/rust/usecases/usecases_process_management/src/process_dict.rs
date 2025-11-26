//! Process Dictionary Module
//!
//! Provides process dictionary functionality.
//! Based on erl_process_dict.c

use std::collections::HashMap;

/// Process dictionary
pub struct ProcessDict {
    dict: HashMap<u64, u64>, // Placeholder - actual implementation needs proper term types
}

impl ProcessDict {
    /// Create a new process dictionary
    pub fn new() -> Self {
        Self {
            dict: HashMap::new(),
        }
    }

    /// Put a value in the dictionary
    pub fn put(&mut self, key: u64, value: u64) -> Option<u64> {
        self.dict.insert(key, value)
    }

    /// Get a value from the dictionary
    pub fn get(&self, key: u64) -> Option<u64> {
        self.dict.get(&key).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_dict() {
        let mut dict = ProcessDict::new();
        dict.put(1, 100);
        assert_eq!(dict.get(1), Some(100));
    }
}

