//! Unicode Handler Module
//!
//! Provides Unicode handling functionality.
//! Based on erl_unicode.c - Unicode conversion and validation.

/// Unicode handler
pub struct UnicodeHandler;

impl UnicodeHandler {
    /// Validate UTF-8 string
    ///
    /// # Arguments
    /// * `data` - UTF-8 bytes to validate
    ///
    /// # Returns
    /// true if valid UTF-8, false otherwise
    pub fn validate_utf8(data: &[u8]) -> bool {
        std::str::from_utf8(data).is_ok()
    }

    /// Convert bytes to UTF-8 string
    ///
    /// # Arguments
    /// * `data` - Bytes to convert
    ///
    /// # Returns
    /// UTF-8 string or error
    pub fn to_utf8_string(data: &[u8]) -> Result<String, UnicodeError> {
        std::str::from_utf8(data)
            .map(|s| s.to_string())
            .map_err(|_| UnicodeError::InvalidUtf8)
    }

    /// Calculate UTF-8 byte length for a character
    ///
    /// # Arguments
    /// * `ch` - Unicode character code point
    ///
    /// # Returns
    /// Number of bytes needed to encode the character in UTF-8
    pub fn utf8_char_length(ch: u32) -> usize {
        if ch < 0x80 {
            1
        } else if ch < 0x800 {
            2
        } else if ch < 0x10000 {
            3
        } else if ch < 0x110000 {
            4
        } else {
            0 // Invalid code point
        }
    }

    /// Count UTF-8 characters in a byte slice
    ///
    /// # Arguments
    /// * `data` - UTF-8 bytes
    ///
    /// # Returns
    /// Number of characters or error if invalid UTF-8
    pub fn count_utf8_chars(data: &[u8]) -> Result<usize, UnicodeError> {
        let s = std::str::from_utf8(data)
            .map_err(|_| UnicodeError::InvalidUtf8)?;
        Ok(s.chars().count())
    }

    /// Check if UTF-8 string is valid Latin1 (ISO-8859-1)
    ///
    /// Latin1 is a subset of Unicode where each character is a single byte.
    ///
    /// # Arguments
    /// * `data` - Bytes to check
    ///
    /// # Returns
    /// true if all bytes are valid Latin1 (0x00-0xFF), false otherwise
    pub fn is_latin1(data: &[u8]) -> bool {
        // All bytes 0x00-0xFF are valid Latin1
        // UTF-8 validation ensures proper encoding
        Self::validate_utf8(data) && data.iter().all(|&b| b < 0x80)
    }

    /// Analyze UTF-8 and return character count
    ///
    /// Similar to erts_analyze_utf8 in the C code.
    ///
    /// # Arguments
    /// * `data` - UTF-8 bytes to analyze
    ///
    /// # Returns
    /// Number of characters if valid UTF-8, error otherwise
    pub fn analyze_utf8(data: &[u8]) -> Result<usize, UnicodeError> {
        Self::count_utf8_chars(data)
    }
}

/// Unicode operation errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnicodeError {
    /// Invalid UTF-8 encoding
    InvalidUtf8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unicode_handler() {
        let valid_utf8 = "Hello, 世界!".as_bytes();
        assert!(UnicodeHandler::validate_utf8(valid_utf8));
        assert!(UnicodeHandler::to_utf8_string(valid_utf8).is_ok());
        
        let invalid_utf8 = &[0xFF, 0xFE, 0xFD];
        assert!(!UnicodeHandler::validate_utf8(invalid_utf8));
    }

    #[test]
    fn test_utf8_char_length() {
        assert_eq!(UnicodeHandler::utf8_char_length(0x41), 1); // 'A'
        assert_eq!(UnicodeHandler::utf8_char_length(0xA2), 2); // Cent sign
        assert_eq!(UnicodeHandler::utf8_char_length(0x20AC), 3); // Euro sign
        assert_eq!(UnicodeHandler::utf8_char_length(0x1F600), 4); // Emoji
        assert_eq!(UnicodeHandler::utf8_char_length(0x110000), 0); // Invalid
    }

    #[test]
    fn test_count_utf8_chars() {
        let ascii = b"Hello";
        assert_eq!(UnicodeHandler::count_utf8_chars(ascii).unwrap(), 5);
        
        let utf8 = "Hello, 世界!".as_bytes();
        assert_eq!(UnicodeHandler::count_utf8_chars(utf8).unwrap(), 10);
        
        let invalid = &[0xFF, 0xFE];
        assert!(UnicodeHandler::count_utf8_chars(invalid).is_err());
    }

    #[test]
    fn test_is_latin1() {
        let latin1 = b"Hello";
        assert!(UnicodeHandler::is_latin1(latin1));
        
        let utf8 = "世界".as_bytes();
        assert!(!UnicodeHandler::is_latin1(utf8));
        
        // Test with byte array containing non-ASCII byte
        let mixed = &[0x48, 0x65, 0x6C, 0x6C, 0x6F, 0xA2]; // "Hello" + cent sign byte
        assert!(!UnicodeHandler::is_latin1(mixed));
    }

    #[test]
    fn test_analyze_utf8() {
        let utf8 = "Hello, 世界!".as_bytes();
        assert_eq!(UnicodeHandler::analyze_utf8(utf8).unwrap(), 10);
        
        let invalid = &[0xFF, 0xFE];
        assert!(UnicodeHandler::analyze_utf8(invalid).is_err());
    }
}

