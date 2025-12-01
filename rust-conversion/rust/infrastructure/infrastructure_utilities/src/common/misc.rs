//! Miscellaneous Utilities
//!
//! Provides miscellaneous utility functions based on erl_misc_utils.c and other utility files.
//! These utilities handle various common operations.

/// Miscellaneous utility functions
pub struct MiscUtils;

impl MiscUtils {
    /// Check if a value is within a range (inclusive)
    ///
    /// # Arguments
    /// * `value` - Value to check
    /// * `min` - Minimum value (inclusive)
    /// * `max` - Maximum value (inclusive)
    ///
    /// # Returns
    /// `true` if value is within range, `false` otherwise
    ///
    /// # Examples
    /// ```
    /// use infrastructure_utilities::MiscUtils;
    ///
    /// assert!(MiscUtils::in_range(5, 1, 10));
    /// assert!(!MiscUtils::in_range(15, 1, 10));
    /// ```
    pub fn in_range<T: PartialOrd>(value: T, min: T, max: T) -> bool {
        value >= min && value <= max
    }

    /// Clamp a value to be within a range
    ///
    /// # Arguments
    /// * `value` - Value to clamp
    /// * `min` - Minimum value
    /// * `max` - Maximum value
    ///
    /// # Returns
    /// Clamped value
    ///
    /// # Examples
    /// ```
    /// use infrastructure_utilities::MiscUtils;
    ///
    /// assert_eq!(MiscUtils::clamp(5, 1, 10), 5);
    /// assert_eq!(MiscUtils::clamp(15, 1, 10), 10);
    /// assert_eq!(MiscUtils::clamp(0, 1, 10), 1);
    /// ```
    pub fn clamp<T: PartialOrd + Copy>(value: T, min: T, max: T) -> T {
        if value < min {
            min
        } else if value > max {
            max
        } else {
            value
        }
    }

    /// Check if a string is empty or whitespace only
    ///
    /// # Arguments
    /// * `s` - String to check
    ///
    /// # Returns
    /// `true` if string is empty or whitespace only
    pub fn is_empty_or_whitespace(s: &str) -> bool {
        s.trim().is_empty()
    }

    /// Convert bytes to a hex string
    ///
    /// # Arguments
    /// * `bytes` - Bytes to convert
    ///
    /// # Returns
    /// Hex string representation
    ///
    /// # Examples
    /// ```
    /// use infrastructure_utilities::MiscUtils;
    ///
    /// let bytes = vec![0x12, 0x34, 0xAB, 0xCD];
    /// assert_eq!(MiscUtils::bytes_to_hex(&bytes), "1234abcd");
    /// ```
    pub fn bytes_to_hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }

    /// Parse a hex string to bytes
    ///
    /// # Arguments
    /// * `hex` - Hex string to parse
    ///
    /// # Returns
    /// * `Some(bytes)` - If parsing succeeds
    /// * `None` - If parsing fails
    ///
    /// # Examples
    /// ```
    /// use infrastructure_utilities::MiscUtils;
    ///
    /// let hex = "1234abcd";
    /// let bytes = MiscUtils::hex_to_bytes(hex).unwrap();
    /// assert_eq!(bytes, vec![0x12, 0x34, 0xAB, 0xCD]);
    /// ```
    pub fn hex_to_bytes(hex: &str) -> Option<Vec<u8>> {
        if hex.len() % 2 != 0 {
            return None;
        }

        let mut bytes = Vec::new();
        for chunk in hex.as_bytes().chunks(2) {
            let byte_str = std::str::from_utf8(chunk).ok()?;
            let byte = u8::from_str_radix(byte_str, 16).ok()?;
            bytes.push(byte);
        }
        Some(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_range() {
        assert!(MiscUtils::in_range(5, 1, 10));
        assert!(MiscUtils::in_range(1, 1, 10));
        assert!(MiscUtils::in_range(10, 1, 10));
        assert!(!MiscUtils::in_range(0, 1, 10));
        assert!(!MiscUtils::in_range(11, 1, 10));
    }

    #[test]
    fn test_clamp() {
        assert_eq!(MiscUtils::clamp(5, 1, 10), 5);
        assert_eq!(MiscUtils::clamp(15, 1, 10), 10);
        assert_eq!(MiscUtils::clamp(0, 1, 10), 1);
    }

    #[test]
    fn test_is_empty_or_whitespace() {
        assert!(MiscUtils::is_empty_or_whitespace(""));
        assert!(MiscUtils::is_empty_or_whitespace("   "));
        assert!(MiscUtils::is_empty_or_whitespace("\t\n"));
        assert!(!MiscUtils::is_empty_or_whitespace("hello"));
    }

    #[test]
    fn test_bytes_to_hex() {
        let bytes = vec![0x12, 0x34, 0xAB, 0xCD];
        assert_eq!(MiscUtils::bytes_to_hex(&bytes), "1234abcd");
    }

    #[test]
    fn test_hex_to_bytes() {
        let hex = "1234abcd";
        let bytes = MiscUtils::hex_to_bytes(hex).unwrap();
        assert_eq!(bytes, vec![0x12, 0x34, 0xAB, 0xCD]);

        // Invalid hex
        assert!(MiscUtils::hex_to_bytes("xyz").is_none());
        assert!(MiscUtils::hex_to_bytes("123").is_none()); // odd length
    }
}

