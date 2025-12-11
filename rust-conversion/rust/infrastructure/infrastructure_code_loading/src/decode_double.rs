//! Double Decoding Module
//!
//! Provides functionality to decode double-precision floating-point numbers from
//! EI (Erlang Interface) format. Supports both the new IEEE 754 format and the
//! legacy string-based format for compatibility.
//!
//! ## Overview
//!
//! Floating-point numbers in EI format can be encoded in two ways:
//! - **NEW_FLOAT_EXT**: 8-byte IEEE 754 double-precision value (preferred, big-endian)
//! - **ERL_FLOAT_EXT**: 31-byte string representation (legacy format)
//!
//! ## Decoding Process
//!
//! 1. Read the tag byte to determine the format
//! 2. For NEW_FLOAT_EXT: Read 8 bytes and convert from IEEE 754
//! 3. For ERL_FLOAT_EXT: Read 31 bytes, parse as string, convert to f64
//! 4. Return the decoded value and updated buffer position
//!
//! ## Examples
//!
//! ```rust
//! use infrastructure_code_loading::decode_double;
//!
//! // Note: This example requires valid EI-encoded double data
//! // In practice, you would decode from a real buffer:
//! // let mut index = 0;
//! // let value = decode_double(&buf, &mut index)?;
//! // println!("Decoded float: {}", value);
//! ```
//!
//! ## See Also
//!
//! - [`encode_double`](super::encode_double/index.html): Double encoding functions
//! - [`decode_integers`](super::decode_integers/index.html): Integer decoding functions
//!
//! Based on `lib/erl_interface/src/decode/decode_double.c`

use crate::constants::{ERL_FLOAT_EXT, NEW_FLOAT_EXT};

/// Decode a double-precision floating-point number from EI format
///
/// # Arguments
/// * `buf` - Buffer containing EI-encoded data
/// * `index` - Current index in buffer
///
/// # Returns
/// * `Ok((value, new_index))` - Decoded value and new index
/// * `Err(DecodeError)` - Decoding error
pub fn decode_double(buf: &[u8], index: &mut usize) -> Result<f64, DecodeError> {
    if *index >= buf.len() {
        return Err(DecodeError::BufferTooShort);
    }

    let tag = buf[*index];
    *index += 1;

    match tag {
        NEW_FLOAT_EXT => {
            if *index + 8 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let bits = u64::from_be_bytes([
                buf[*index],
                buf[*index + 1],
                buf[*index + 2],
                buf[*index + 3],
                buf[*index + 4],
                buf[*index + 5],
                buf[*index + 6],
                buf[*index + 7],
            ]);
            *index += 8;
            Ok(f64::from_bits(bits))
        }
        ERL_FLOAT_EXT => {
            // Old format: 31-byte string representation
            if *index + 31 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let float_str = std::str::from_utf8(&buf[*index..*index + 31])
                .map_err(|_| DecodeError::InvalidFormat("Invalid UTF-8 in float".to_string()))?;
            let value = float_str.trim_end_matches('\0')
                .parse::<f64>()
                .map_err(|_| DecodeError::InvalidFormat("Invalid float format".to_string()))?;
            *index += 31;
            Ok(value)
        }
        _ => Err(DecodeError::InvalidFormat(format!("Unexpected tag: {}", tag))),
    }
}

/// Decoding errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// Buffer is too short
    BufferTooShort,
    /// Invalid format
    InvalidFormat(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_double() {
        let value: f64 = 3.14;
        let mut buf = vec![NEW_FLOAT_EXT];
        buf.extend_from_slice(&value.to_bits().to_be_bytes());
        let mut index = 0;
        let decoded = decode_double(&buf, &mut index).unwrap();
        assert!((decoded - value).abs() < 1e-10);
        assert_eq!(index, 9);
    }

    #[test]
    fn test_decode_roundtrip() {
        let test_values = vec![0.0, 3.14, -3.14, 1.0, -1.0, 1e10, -1e10];
        
        for original in test_values {
            let mut buf = vec![0u8; 10];
            let mut encode_index = 0;
            crate::encode_double::encode_double(&mut Some(&mut buf), &mut encode_index, original).unwrap();
            
            let mut decode_index = 0;
            let decoded = decode_double(&buf, &mut decode_index).unwrap();
            assert!((decoded - original).abs() < 1e-10, "Roundtrip failed for {}", original);
        }
    }

    #[test]
    fn test_decode_empty_buffer() {
        let buf = vec![];
        let mut index = 0;
        let result = decode_double(&buf, &mut index);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DecodeError::BufferTooShort);
    }

    #[test]
    fn test_decode_buffer_too_short_for_new_float() {
        // Buffer has tag but not enough bytes for the float
        let buf = vec![NEW_FLOAT_EXT, 0, 1, 2, 3, 4, 5, 6]; // Only 7 bytes after tag, need 8
        let mut index = 0;
        let result = decode_double(&buf, &mut index);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DecodeError::BufferTooShort);
    }

    #[test]
    fn test_decode_old_float_format() {
        // Test ERL_FLOAT_EXT format (31-byte string)
        let value = 3.14159;
        let mut buf = vec![ERL_FLOAT_EXT];
        let float_str = "3.14159"; // Short string to ensure padding loop executes
        let mut float_bytes = float_str.as_bytes().to_vec();
        // Pad to 31 bytes with nulls
        while float_bytes.len() < 31 {
            float_bytes.push(0);
        }
        buf.extend_from_slice(&float_bytes[..31]);
        
        let mut index = 0;
        let decoded = decode_double(&buf, &mut index).unwrap();
        assert!((decoded - value).abs() < 1e-5); // Less precision in string format
        assert_eq!(index, 32);
    }

    #[test]
    fn test_decode_old_float_format_with_trailing_nulls() {
        // Test ERL_FLOAT_EXT format with trailing nulls
        let value = 42.0;
        let mut buf = vec![ERL_FLOAT_EXT];
        let float_str = "42.0";
        let mut float_bytes = float_str.as_bytes().to_vec();
        // Pad to 31 bytes with nulls
        while float_bytes.len() < 31 {
            float_bytes.push(0);
        }
        buf.extend_from_slice(&float_bytes[..31]);
        
        let mut index = 0;
        let decoded = decode_double(&buf, &mut index).unwrap();
        assert!((decoded - value).abs() < 1e-10);
        assert_eq!(index, 32);
    }

    #[test]
    fn test_decode_old_float_format_negative() {
        // Test ERL_FLOAT_EXT format with negative number
        let value = -123.456;
        let mut buf = vec![ERL_FLOAT_EXT];
        let float_str = "-123.456"; // Short string to ensure padding loop executes
        let mut float_bytes = float_str.as_bytes().to_vec();
        // Pad to 31 bytes with nulls
        while float_bytes.len() < 31 {
            float_bytes.push(0);
        }
        buf.extend_from_slice(&float_bytes[..31]);
        
        let mut index = 0;
        let decoded = decode_double(&buf, &mut index).unwrap();
        assert!((decoded - value).abs() < 1e-5);
        assert_eq!(index, 32);
    }

    #[test]
    fn test_decode_old_float_buffer_too_short() {
        // Buffer has tag but not enough bytes for the 31-byte float
        let buf = vec![ERL_FLOAT_EXT, 0, 1, 2]; // Only 3 bytes after tag, need 31
        let mut index = 0;
        let result = decode_double(&buf, &mut index);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DecodeError::BufferTooShort);
    }

    #[test]
    fn test_decode_old_float_invalid_utf8() {
        // Test ERL_FLOAT_EXT format with invalid UTF-8
        let mut buf = vec![ERL_FLOAT_EXT];
        // Invalid UTF-8 sequence
        buf.push(0xFF);
        buf.push(0xFE);
        // Fill rest with zeros
        while buf.len() < 32 {
            buf.push(0);
        }
        
        let mut index = 0;
        let result = decode_double(&buf, &mut index);
        assert!(result.is_err());
        if let DecodeError::InvalidFormat(msg) = result.unwrap_err() {
            assert!(msg.contains("Invalid UTF-8"));
        } else {
            panic!("Expected InvalidFormat error");
        }
    }

    #[test]
    fn test_decode_old_float_invalid_format() {
        // Test ERL_FLOAT_EXT format with invalid float string
        let mut buf = vec![ERL_FLOAT_EXT];
        let invalid_str = "not a number";
        let mut float_bytes = invalid_str.as_bytes().to_vec();
        // Pad to 31 bytes with nulls
        while float_bytes.len() < 31 {
            float_bytes.push(0);
        }
        buf.extend_from_slice(&float_bytes[..31]);
        
        let mut index = 0;
        let result = decode_double(&buf, &mut index);
        assert!(result.is_err());
        if let DecodeError::InvalidFormat(msg) = result.unwrap_err() {
            assert!(msg.contains("Invalid float format"));
        } else {
            panic!("Expected InvalidFormat error");
        }
    }

    #[test]
    fn test_decode_unexpected_tag() {
        // Test with an unexpected tag
        let buf = vec![0xAA, 1, 2, 3, 4, 5, 6, 7, 8]; // Invalid tag
        let mut index = 0;
        let result = decode_double(&buf, &mut index);
        assert!(result.is_err());
        if let DecodeError::InvalidFormat(msg) = result.unwrap_err() {
            assert!(msg.contains("Unexpected tag"));
        } else {
            panic!("Expected InvalidFormat error");
        }
    }

    #[test]
    fn test_decode_error_debug() {
        let error1 = DecodeError::BufferTooShort;
        let error2 = DecodeError::InvalidFormat("test".to_string());
        let debug_str1 = format!("{:?}", error1);
        let debug_str2 = format!("{:?}", error2);
        assert!(debug_str1.contains("BufferTooShort"));
        assert!(debug_str2.contains("InvalidFormat"));
        assert!(debug_str2.contains("test"));
    }

    #[test]
    fn test_decode_error_clone() {
        let error1 = DecodeError::BufferTooShort;
        let error2 = DecodeError::InvalidFormat("test".to_string());
        let cloned1 = error1.clone();
        let cloned2 = error2.clone();
        assert_eq!(error1, cloned1);
        assert_eq!(error2, cloned2);
    }

    #[test]
    fn test_decode_error_partial_eq() {
        let error1 = DecodeError::BufferTooShort;
        let error2 = DecodeError::BufferTooShort;
        let error3 = DecodeError::InvalidFormat("test".to_string());
        let error4 = DecodeError::InvalidFormat("test".to_string());
        let error5 = DecodeError::InvalidFormat("different".to_string());
        
        assert_eq!(error1, error2);
        assert_eq!(error3, error4);
        assert_ne!(error1, error3);
        assert_ne!(error4, error5);
    }

    #[test]
    fn test_decode_error_eq() {
        let error1 = DecodeError::BufferTooShort;
        let error2 = DecodeError::BufferTooShort;
        let error3 = DecodeError::InvalidFormat("test".to_string());
        
        assert!(error1 == error2);
        assert!(error1 != error3);
    }

    #[test]
    fn test_decode_various_float_values() {
        // Test various float values with NEW_FLOAT_EXT
        let test_values = vec![
            0.0,
            -0.0,
            1.0,
            -1.0,
            std::f64::consts::PI,
            -std::f64::consts::PI,
            std::f64::MAX,
            std::f64::MIN,
            std::f64::EPSILON,
        ];
        
        for value in test_values {
            let mut buf = vec![NEW_FLOAT_EXT];
            buf.extend_from_slice(&value.to_bits().to_be_bytes());
            let mut index = 0;
            let decoded = decode_double(&buf, &mut index).unwrap();
            // For special values, check exact equality
            if value == 0.0 || value == -0.0 || value.is_infinite() || value.is_nan() {
                assert_eq!(decoded.to_bits(), value.to_bits());
            } else {
                assert!((decoded - value).abs() < 1e-10);
            }
            assert_eq!(index, 9);
        }
    }

    #[test]
    fn test_decode_index_at_end() {
        // Test when index is at the end of buffer
        let buf = vec![NEW_FLOAT_EXT];
        let mut index = 1; // Already past the tag
        let result = decode_double(&buf, &mut index);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DecodeError::BufferTooShort);
    }
}

