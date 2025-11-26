//! RAM File Driver Module
//!
//! Provides RAM file driver functionality.
//! Based on ram_file_drv.c

use std::collections::HashMap;

/// RAM file driver
pub struct RamFileDriver {
    files: HashMap<String, Vec<u8>>,
}

impl RamFileDriver {
    /// Create a new RAM file driver
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }

    /// Create a file
    pub fn create_file(&mut self, name: String, data: Vec<u8>) {
        self.files.insert(name, data);
    }

    /// Read a file
    pub fn read_file(&self, name: &str) -> Option<&[u8]> {
        self.files.get(name).map(|v| v.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ram_file_driver() {
        let mut driver = RamFileDriver::new();
        driver.create_file("test.txt".to_string(), b"test data".to_vec());
        assert_eq!(driver.read_file("test.txt"), Some(b"test data".as_slice()));
    }
}

