//! Unicode Handler Module
//!
//! Provides Unicode handling functionality.
//! Based on erl_unicode.c

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
}

