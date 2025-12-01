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

    /// Reverse a string
    ///
    /// # Arguments
    /// * `s` - String to reverse
    ///
    /// # Returns
    /// Reversed string
    pub fn reverse_string(s: &str) -> String {
        s.chars().rev().collect()
    }

    /// Count occurrences of a substring in a string
    ///
    /// # Arguments
    /// * `s` - String to search in
    /// * `substring` - Substring to count
    ///
    /// # Returns
    /// Number of occurrences
    pub fn count_substring(s: &str, substring: &str) -> usize {
        if substring.is_empty() {
            return s.len() + 1;
        }
        s.matches(substring).count()
    }

    /// Check if a string starts with a prefix (case-insensitive)
    ///
    /// # Arguments
    /// * `s` - String to check
    /// * `prefix` - Prefix to check for
    ///
    /// # Returns
    /// `true` if string starts with prefix (case-insensitive)
    pub fn starts_with_ignore_case(s: &str, prefix: &str) -> bool {
        s.len() >= prefix.len() && s[..prefix.len()].eq_ignore_ascii_case(prefix)
    }

    /// Check if a string ends with a suffix (case-insensitive)
    ///
    /// # Arguments
    /// * `s` - String to check
    /// * `suffix` - Suffix to check for
    ///
    /// # Returns
    /// `true` if string ends with suffix (case-insensitive)
    pub fn ends_with_ignore_case(s: &str, suffix: &str) -> bool {
        s.len() >= suffix.len() && s[s.len() - suffix.len()..].eq_ignore_ascii_case(suffix)
    }

    /// Split a string by whitespace
    ///
    /// # Arguments
    /// * `s` - String to split
    ///
    /// # Returns
    /// Vector of non-empty words
    pub fn split_whitespace(s: &str) -> Vec<String> {
        s.split_whitespace().map(|s| s.to_string()).collect()
    }

    /// Join strings with a separator
    ///
    /// # Arguments
    /// * `strings` - Vector of strings to join
    /// * `separator` - Separator string
    ///
    /// # Returns
    /// Joined string
    pub fn join_strings(strings: &[String], separator: &str) -> String {
        strings.join(separator)
    }

    /// Remove leading and trailing whitespace from a string
    ///
    /// # Arguments
    /// * `s` - String to trim
    ///
    /// # Returns
    /// Trimmed string
    pub fn trim(s: &str) -> String {
        s.trim().to_string()
    }

    /// Convert string to lowercase
    ///
    /// # Arguments
    /// * `s` - String to convert
    ///
    /// # Returns
    /// Lowercase string
    pub fn to_lowercase(s: &str) -> String {
        s.to_lowercase()
    }

    /// Convert string to uppercase
    ///
    /// # Arguments
    /// * `s` - String to convert
    ///
    /// # Returns
    /// Uppercase string
    pub fn to_uppercase(s: &str) -> String {
        s.to_uppercase()
    }

    /// Check if a string contains a substring (case-insensitive)
    ///
    /// # Arguments
    /// * `s` - String to search in
    /// * `substring` - Substring to find
    ///
    /// # Returns
    /// `true` if substring is found (case-insensitive)
    pub fn contains_ignore_case(s: &str, substring: &str) -> bool {
        s.to_lowercase().contains(&substring.to_lowercase())
    }

    /// Replace all occurrences of a substring
    ///
    /// # Arguments
    /// * `s` - String to modify
    /// * `from` - Substring to replace
    /// * `to` - Replacement string
    ///
    /// # Returns
    /// String with replacements
    pub fn replace_all(s: &str, from: &str, to: &str) -> String {
        s.replace(from, to)
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

    #[test]
    fn test_reverse_string() {
        assert_eq!(MiscUtils::reverse_string("hello"), "olleh");
        assert_eq!(MiscUtils::reverse_string(""), "");
    }

    #[test]
    fn test_count_substring() {
        assert_eq!(MiscUtils::count_substring("hello hello", "hello"), 2);
        // matches() doesn't count overlapping matches, so "aaa" with "aa" gives 1
        assert_eq!(MiscUtils::count_substring("aaa", "aa"), 1);
        assert_eq!(MiscUtils::count_substring("ababab", "ab"), 3);
    }

    #[test]
    fn test_starts_with_ignore_case() {
        assert!(MiscUtils::starts_with_ignore_case("Hello", "he"));
        assert!(!MiscUtils::starts_with_ignore_case("Hello", "lo"));
    }

    #[test]
    fn test_ends_with_ignore_case() {
        assert!(MiscUtils::ends_with_ignore_case("Hello", "LO"));
        assert!(!MiscUtils::ends_with_ignore_case("Hello", "he"));
    }

    #[test]
    fn test_split_whitespace() {
        let words = MiscUtils::split_whitespace("hello world  test");
        assert_eq!(words, vec!["hello", "world", "test"]);
    }

    #[test]
    fn test_join_strings() {
        let strings = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        assert_eq!(MiscUtils::join_strings(&strings, "-"), "a-b-c");
    }

    #[test]
    fn test_trim() {
        assert_eq!(MiscUtils::trim("  hello  "), "hello");
        assert_eq!(MiscUtils::trim("hello"), "hello");
    }

    #[test]
    fn test_to_lowercase_uppercase() {
        assert_eq!(MiscUtils::to_lowercase("HELLO"), "hello");
        assert_eq!(MiscUtils::to_uppercase("hello"), "HELLO");
    }

    #[test]
    fn test_contains_ignore_case() {
        assert!(MiscUtils::contains_ignore_case("Hello", "ELL"));
        assert!(!MiscUtils::contains_ignore_case("Hello", "xyz"));
    }

    #[test]
    fn test_replace_all() {
        assert_eq!(MiscUtils::replace_all("hello hello", "hello", "hi"), "hi hi");
    }
}

