//! Binary Operations Module
//!
//! Provides binary data handling for Erlang terms.
//! Based on binary.c and erl_bif_binary.c

/// Binary data structure
pub struct Binary {
    data: Vec<u8>,
}

impl Binary {
    /// Create a new binary
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Get binary data
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_creation() {
        let data = vec![1, 2, 3, 4];
        let binary = Binary::new(data.clone());
        assert_eq!(binary.data(), &data);
    }
}

