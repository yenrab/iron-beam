//! Integer Decoding Module
//!
//! Provides functionality to decode integers from EI (Erlang Interface) format.
//! This module implements decoding for signed and unsigned integers of various
//! sizes, automatically handling all supported integer encoding formats.
//!
//! ## Overview
//!
//! The decoder handles all integer formats supported by the EI specification:
//! - **Small Integer** (`ERL_SMALL_INTEGER_EXT`): 0-255
//! - **32-bit Integer** (`ERL_INTEGER_EXT`): -2^31 to 2^31-1
//! - **Big Integer** (`ERL_SMALL_BIG_EXT`, `ERL_LARGE_BIG_EXT`): Arbitrary precision
//!
//! ## Decoding Process
//!
//! 1. Read the tag byte to determine the integer format
//! 2. Decode the value based on the format
//! 3. Update the buffer index to point past the decoded value
//! 4. Return the decoded value and new index
//!
//! ## Examples
//!
//! ```rust
//! use infrastructure_code_loading::decode_integers::*;
//!
//! let buf = vec![97, 42]; // ERL_SMALL_INTEGER_EXT, value 42
//! let mut index = 0;
//!
//! let value = decode_long(&buf, &mut index)?;
//! assert_eq!(value, 42);
//! ```
//!
//! ## See Also
//!
//! - [`encode_integers`](super::encode_integers/index.html): Integer encoding functions
//! - [`constants`](super::constants/index.html): EI format tag constants
//! - [`infrastructure_bignum_encoding`](../infrastructure_bignum_encoding/index.html): Big number decoding
//!
//! Based on `lib/erl_interface/src/decode/decode_longlong.c` and `decode_ulonglong.c`

use crate::constants::*;

/// Decode a signed 64-bit integer (i64) from EI format
///
/// # Arguments
/// * `buf` - Buffer containing EI-encoded data
/// * `index` - Current index in buffer
///
/// # Returns
/// * `Ok((value, new_index))` - Decoded value and new index
/// * `Err(DecodeError)` - Decoding error
pub fn decode_longlong(buf: &[u8], index: &mut usize) -> Result<i64, DecodeError> {
    if *index >= buf.len() {
        return Err(DecodeError::BufferTooShort);
    }

    let tag = buf[*index];
    *index += 1;

    match tag {
        ERL_SMALL_INTEGER_EXT => {
            if *index >= buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let value = buf[*index] as i64;
            *index += 1;
            Ok(value)
        }
        ERL_INTEGER_EXT => {
            if *index + 4 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let value = i32::from_be_bytes([
                buf[*index],
                buf[*index + 1],
                buf[*index + 2],
                buf[*index + 3],
            ]) as i64;
            *index += 4;
            Ok(value)
        }
        ERL_SMALL_BIG_EXT | ERL_LARGE_BIG_EXT => {
            let arity = if tag == ERL_SMALL_BIG_EXT {
                if *index >= buf.len() {
                    return Err(DecodeError::BufferTooShort);
                }
                let a = buf[*index] as usize;
                *index += 1;
                a
            } else {
                if *index + 4 > buf.len() {
                    return Err(DecodeError::BufferTooShort);
                }
                let a = u32::from_be_bytes([
                    buf[*index],
                    buf[*index + 1],
                    buf[*index + 2],
                    buf[*index + 3],
                ]) as usize;
                *index += 4;
                a
            };

            if *index >= buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let sign = buf[*index];
            *index += 1;

            if *index + arity > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }

            let mut value: u64 = 0;
            for i in 0..arity.min(8) {
                value |= (buf[*index + i] as u64) << (i * 8);
            }

            // Check for overflow beyond i64
            if arity > 8 {
                for i in 8..arity {
                    if buf[*index + i] != 0 {
                        return Err(DecodeError::ValueTooLarge);
                    }
                }
            }

            *index += arity;

            let result = if sign != 0 {
                if value > 0x8000000000000000 {
                    return Err(DecodeError::ValueTooLarge);
                }
                -(value as i64)
            } else {
                if value > 0x7FFFFFFFFFFFFFFF {
                    return Err(DecodeError::ValueTooLarge);
                }
                value as i64
            };

            Ok(result)
        }
        _ => Err(DecodeError::InvalidFormat(format!("Unexpected tag: {}", tag))),
    }
}

/// Decode an unsigned 64-bit integer (u64) from EI format
///
/// # Arguments
/// * `buf` - Buffer containing EI-encoded data
/// * `index` - Current index in buffer
///
/// # Returns
/// * `Ok((value, new_index))` - Decoded value and new index
/// * `Err(DecodeError)` - Decoding error
pub fn decode_ulonglong(buf: &[u8], index: &mut usize) -> Result<u64, DecodeError> {
    if *index >= buf.len() {
        return Err(DecodeError::BufferTooShort);
    }

    let tag = buf[*index];
    *index += 1;

    match tag {
        ERL_SMALL_INTEGER_EXT => {
            if *index >= buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let value = buf[*index] as u64;
            *index += 1;
            Ok(value)
        }
        ERL_INTEGER_EXT => {
            if *index + 4 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let signed = i32::from_be_bytes([
                buf[*index],
                buf[*index + 1],
                buf[*index + 2],
                buf[*index + 3],
            ]);
            if signed < 0 {
                return Err(DecodeError::InvalidFormat("Negative value for unsigned".to_string()));
            }
            *index += 4;
            Ok(signed as u64)
        }
        ERL_SMALL_BIG_EXT | ERL_LARGE_BIG_EXT => {
            let arity = if tag == ERL_SMALL_BIG_EXT {
                if *index >= buf.len() {
                    return Err(DecodeError::BufferTooShort);
                }
                let a = buf[*index] as usize;
                *index += 1;
                a
            } else {
                if *index + 4 > buf.len() {
                    return Err(DecodeError::BufferTooShort);
                }
                let a = u32::from_be_bytes([
                    buf[*index],
                    buf[*index + 1],
                    buf[*index + 2],
                    buf[*index + 3],
                ]) as usize;
                *index += 4;
                a
            };

            if *index >= buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let sign = buf[*index];
            *index += 1;

            if sign != 0 {
                return Err(DecodeError::InvalidFormat("Negative value for unsigned".to_string()));
            }

            if *index + arity > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }

            let mut value: u64 = 0;
            for i in 0..arity.min(8) {
                value |= (buf[*index + i] as u64) << (i * 8);
            }

            // Check for overflow beyond u64
            if arity > 8 {
                for i in 8..arity {
                    if buf[*index + i] != 0 {
                        return Err(DecodeError::ValueTooLarge);
                    }
                }
            }

            *index += arity;
            Ok(value)
        }
        _ => Err(DecodeError::InvalidFormat(format!("Unexpected tag: {}", tag))),
    }
}

/// Decode a signed 32-bit integer (i32) from EI format
pub fn decode_long(buf: &[u8], index: &mut usize) -> Result<i32, DecodeError> {
    decode_longlong(buf, index).and_then(|v| {
        if v < i32::MIN as i64 || v > i32::MAX as i64 {
            Err(DecodeError::ValueTooLarge)
        } else {
            Ok(v as i32)
        }
    })
}

/// Decode an unsigned 32-bit integer (u32) from EI format
pub fn decode_ulong(buf: &[u8], index: &mut usize) -> Result<u32, DecodeError> {
    decode_ulonglong(buf, index).and_then(|v| {
        if v > u32::MAX as u64 {
            Err(DecodeError::ValueTooLarge)
        } else {
            Ok(v as u32)
        }
    })
}

/// Decoding errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// Buffer is too short
    BufferTooShort,
    /// Invalid format
    InvalidFormat(String),
    /// Value is too large to fit in target type
    ValueTooLarge,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_longlong_small() {
        let buf = vec![ERL_SMALL_INTEGER_EXT, 42];
        let mut index = 0;
        let value = decode_longlong(&buf, &mut index).unwrap();
        assert_eq!(value, 42);
        assert_eq!(index, 2);
    }

    #[test]
    fn test_decode_longlong_32bit() {
        let value = 1000i32;
        let mut buf = vec![ERL_INTEGER_EXT];
        buf.extend_from_slice(&value.to_be_bytes());
        let mut index = 0;
        let decoded = decode_longlong(&buf, &mut index).unwrap();
        assert_eq!(decoded, 1000);
        assert_eq!(index, 5);
    }

    #[test]
    fn test_decode_ulonglong_small() {
        let buf = vec![ERL_SMALL_INTEGER_EXT, 42];
        let mut index = 0;
        let value = decode_ulonglong(&buf, &mut index).unwrap();
        assert_eq!(value, 42);
        assert_eq!(index, 2);
    }

    #[test]
    fn test_decode_roundtrip() {
        let test_values = vec![0i64, 42, 255, 256, 1000, -1000, 1_000_000, -1_000_000];
        
        for original in test_values {
            let mut buf = vec![0u8; 20];
            let mut encode_index = 0;
            crate::encode_integers::encode_longlong(&mut Some(&mut buf), &mut encode_index, original).unwrap();
            
            let mut decode_index = 0;
            let decoded = decode_longlong(&buf, &mut decode_index).unwrap();
            assert_eq!(decoded, original, "Roundtrip failed for {}", original);
        }
    }

    #[test]
    fn test_decode_longlong_empty_buffer() {
        let buf = vec![];
        let mut index = 0;
        let result = decode_longlong(&buf, &mut index);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DecodeError::BufferTooShort);
    }

    #[test]
    fn test_decode_longlong_small_buffer_too_short() {
        let buf = vec![ERL_SMALL_INTEGER_EXT];
        let mut index = 0;
        let result = decode_longlong(&buf, &mut index);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DecodeError::BufferTooShort);
    }

    #[test]
    fn test_decode_longlong_integer_buffer_too_short() {
        let buf = vec![ERL_INTEGER_EXT, 1, 2, 3]; // Only 3 bytes, need 4
        let mut index = 0;
        let result = decode_longlong(&buf, &mut index);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DecodeError::BufferTooShort);
    }

    #[test]
    fn test_decode_longlong_small_big() {
        // Test SMALL_BIG_EXT with a positive value
        let value: i64 = 1_000_000_000_000;
        let mut buf = vec![0u8; 20];
        let mut encode_index = 0;
        crate::encode_integers::encode_longlong(&mut Some(&mut buf), &mut encode_index, value).unwrap();
        
        let mut decode_index = 0;
        let decoded = decode_longlong(&buf, &mut decode_index).unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    fn test_decode_longlong_small_big_negative() {
        // Test SMALL_BIG_EXT with a negative value
        let value: i64 = -1_000_000_000_000;
        let mut buf = vec![0u8; 20];
        let mut encode_index = 0;
        crate::encode_integers::encode_longlong(&mut Some(&mut buf), &mut encode_index, value).unwrap();
        
        let mut decode_index = 0;
        let decoded = decode_longlong(&buf, &mut decode_index).unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    fn test_decode_longlong_small_big_buffer_too_short_for_arity() {
        let buf = vec![ERL_SMALL_BIG_EXT];
        let mut index = 0;
        let result = decode_longlong(&buf, &mut index);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DecodeError::BufferTooShort);
    }

    #[test]
    fn test_decode_longlong_small_big_buffer_too_short_for_sign() {
        let buf = vec![ERL_SMALL_BIG_EXT, 5];
        let mut index = 0;
        let result = decode_longlong(&buf, &mut index);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DecodeError::BufferTooShort);
    }

    #[test]
    fn test_decode_longlong_small_big_buffer_too_short_for_data() {
        let buf = vec![ERL_SMALL_BIG_EXT, 5, 0]; // arity=5, sign=0, but no data
        let mut index = 0;
        let result = decode_longlong(&buf, &mut index);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DecodeError::BufferTooShort);
    }

    #[test]
    fn test_decode_longlong_large_big() {
        // Test LARGE_BIG_EXT - manually construct it
        let mut buf = vec![ERL_LARGE_BIG_EXT];
        // arity as u32 (4 bytes) - use 5 bytes
        buf.extend_from_slice(&5u32.to_be_bytes());
        buf.push(0); // sign (positive)
        // 5 bytes of data: value = 0x0102030405
        buf.extend_from_slice(&[0x05, 0x04, 0x03, 0x02, 0x01]);
        
        let mut index = 0;
        let decoded = decode_longlong(&buf, &mut index).unwrap();
        assert_eq!(decoded, 0x0102030405);
    }

    #[test]
    fn test_decode_longlong_large_big_negative() {
        // Test LARGE_BIG_EXT with negative value
        let mut buf = vec![ERL_LARGE_BIG_EXT];
        buf.extend_from_slice(&5u32.to_be_bytes());
        buf.push(1); // sign (negative)
        buf.extend_from_slice(&[0x05, 0x04, 0x03, 0x02, 0x01]);
        
        let mut index = 0;
        let decoded = decode_longlong(&buf, &mut index).unwrap();
        assert_eq!(decoded, -0x0102030405);
    }

    #[test]
    fn test_decode_longlong_large_big_buffer_too_short_for_arity() {
        let buf = vec![ERL_LARGE_BIG_EXT, 0, 0, 0]; // Only 3 bytes, need 4
        let mut index = 0;
        let result = decode_longlong(&buf, &mut index);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DecodeError::BufferTooShort);
    }

    #[test]
    fn test_decode_longlong_value_too_large_positive() {
        // Create a value that exceeds i64::MAX
        let mut buf = vec![ERL_SMALL_BIG_EXT];
        buf.push(8); // arity = 8 bytes
        buf.push(0); // sign = positive
        // Value = 0x8000000000000000 (i64::MAX + 1)
        buf.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80]);
        
        let mut index = 0;
        let result = decode_longlong(&buf, &mut index);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DecodeError::ValueTooLarge);
    }

    #[test]
    fn test_decode_longlong_value_too_large_negative() {
        // Create a value that exceeds i64::MIN
        let mut buf = vec![ERL_SMALL_BIG_EXT];
        buf.push(8); // arity = 8 bytes
        buf.push(1); // sign = negative
        // Value = 0x8000000000000001 (too large for negative i64)
        buf.extend_from_slice(&[0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80]);
        
        let mut index = 0;
        let result = decode_longlong(&buf, &mut index);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DecodeError::ValueTooLarge);
    }

    #[test]
    fn test_decode_longlong_arity_too_large() {
        // Create a value with arity > 8 and non-zero bytes beyond 8
        let mut buf = vec![ERL_SMALL_BIG_EXT];
        buf.push(10); // arity = 10 bytes
        buf.push(0); // sign = positive
        // First 8 bytes: valid value
        buf.extend_from_slice(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);
        // Bytes 9-10: non-zero (should trigger ValueTooLarge)
        buf.extend_from_slice(&[0x01, 0x02]);
        
        let mut index = 0;
        let result = decode_longlong(&buf, &mut index);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DecodeError::ValueTooLarge);
    }

    #[test]
    fn test_decode_longlong_unexpected_tag() {
        let buf = vec![0xFF, 1, 2, 3];
        let mut index = 0;
        let result = decode_longlong(&buf, &mut index);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecodeError::InvalidFormat(msg) => {
                assert!(msg.contains("Unexpected tag"));
            }
            _ => panic!("Expected InvalidFormat error"),
        }
    }

    #[test]
    fn test_decode_ulonglong_empty_buffer() {
        let buf = vec![];
        let mut index = 0;
        let result = decode_ulonglong(&buf, &mut index);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DecodeError::BufferTooShort);
    }

    #[test]
    fn test_decode_ulonglong_small_buffer_too_short() {
        let buf = vec![ERL_SMALL_INTEGER_EXT];
        let mut index = 0;
        let result = decode_ulonglong(&buf, &mut index);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DecodeError::BufferTooShort);
    }

    #[test]
    fn test_decode_ulonglong_integer() {
        let value = 1000u32;
        let mut buf = vec![ERL_INTEGER_EXT];
        buf.extend_from_slice(&(value as i32).to_be_bytes());
        let mut index = 0;
        let decoded = decode_ulonglong(&buf, &mut index).unwrap();
        assert_eq!(decoded, 1000);
        assert_eq!(index, 5);
    }

    #[test]
    fn test_decode_ulonglong_integer_negative() {
        // Negative value should fail for unsigned
        let value = -1000i32;
        let mut buf = vec![ERL_INTEGER_EXT];
        buf.extend_from_slice(&value.to_be_bytes());
        let mut index = 0;
        let result = decode_ulonglong(&buf, &mut index);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecodeError::InvalidFormat(msg) => {
                assert!(msg.contains("Negative value"));
            }
            _ => panic!("Expected InvalidFormat error"),
        }
    }

    #[test]
    fn test_decode_ulonglong_integer_buffer_too_short() {
        let buf = vec![ERL_INTEGER_EXT, 1, 2, 3];
        let mut index = 0;
        let result = decode_ulonglong(&buf, &mut index);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DecodeError::BufferTooShort);
    }

    #[test]
    fn test_decode_ulonglong_small_big() {
        let value: u64 = 1_000_000_000_000;
        let mut buf = vec![0u8; 20];
        let mut encode_index = 0;
        crate::encode_integers::encode_ulonglong(&mut Some(&mut buf), &mut encode_index, value).unwrap();
        
        let mut decode_index = 0;
        let decoded = decode_ulonglong(&buf, &mut decode_index).unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    fn test_decode_ulonglong_small_big_negative_sign() {
        // SMALL_BIG_EXT with negative sign should fail for unsigned
        let mut buf = vec![ERL_SMALL_BIG_EXT];
        buf.push(1); // arity = 1
        buf.push(1); // sign = negative (should fail)
        buf.push(42); // value
        
        let mut index = 0;
        let result = decode_ulonglong(&buf, &mut index);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecodeError::InvalidFormat(msg) => {
                assert!(msg.contains("Negative value"));
            }
            _ => panic!("Expected InvalidFormat error"),
        }
    }

    #[test]
    fn test_decode_ulonglong_small_big_buffer_too_short() {
        let buf = vec![ERL_SMALL_BIG_EXT, 5, 0]; // arity=5, sign=0, but no data
        let mut index = 0;
        let result = decode_ulonglong(&buf, &mut index);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DecodeError::BufferTooShort);
    }

    #[test]
    fn test_decode_ulonglong_large_big() {
        let mut buf = vec![ERL_LARGE_BIG_EXT];
        buf.extend_from_slice(&5u32.to_be_bytes());
        buf.push(0); // sign (positive)
        buf.extend_from_slice(&[0x05, 0x04, 0x03, 0x02, 0x01]);
        
        let mut index = 0;
        let decoded = decode_ulonglong(&buf, &mut index).unwrap();
        assert_eq!(decoded, 0x0102030405);
    }

    #[test]
    fn test_decode_ulonglong_large_big_negative_sign() {
        let mut buf = vec![ERL_LARGE_BIG_EXT];
        buf.extend_from_slice(&1u32.to_be_bytes());
        buf.push(1); // sign = negative (should fail)
        buf.push(42);
        
        let mut index = 0;
        let result = decode_ulonglong(&buf, &mut index);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecodeError::InvalidFormat(msg) => {
                assert!(msg.contains("Negative value"));
            }
            _ => panic!("Expected InvalidFormat error"),
        }
    }

    #[test]
    fn test_decode_ulonglong_value_too_large() {
        // Create a value that exceeds u64::MAX
        let mut buf = vec![ERL_SMALL_BIG_EXT];
        buf.push(9); // arity = 9 bytes
        buf.push(0); // sign = positive
        // First 8 bytes: valid value
        buf.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
        // Byte 9: non-zero (should trigger ValueTooLarge)
        buf.push(0x01);
        
        let mut index = 0;
        let result = decode_ulonglong(&buf, &mut index);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DecodeError::ValueTooLarge);
    }

    #[test]
    fn test_decode_ulonglong_unexpected_tag() {
        let buf = vec![0xFF, 1, 2, 3];
        let mut index = 0;
        let result = decode_ulonglong(&buf, &mut index);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecodeError::InvalidFormat(msg) => {
                assert!(msg.contains("Unexpected tag"));
            }
            _ => panic!("Expected InvalidFormat error"),
        }
    }

    #[test]
    fn test_decode_long() {
        let value = 1000i32;
        let mut buf = vec![0u8; 20];
        let mut encode_index = 0;
        crate::encode_integers::encode_long(&mut Some(&mut buf), &mut encode_index, value).unwrap();
        
        let mut decode_index = 0;
        let decoded = decode_long(&buf, &mut decode_index).unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    fn test_decode_long_value_too_large() {
        // Encode a value that's too large for i32
        let value: i64 = i32::MAX as i64 + 1;
        let mut buf = vec![0u8; 20];
        let mut encode_index = 0;
        crate::encode_integers::encode_longlong(&mut Some(&mut buf), &mut encode_index, value).unwrap();
        
        let mut decode_index = 0;
        let result = decode_long(&buf, &mut decode_index);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DecodeError::ValueTooLarge);
    }

    #[test]
    fn test_decode_long_value_too_small() {
        // Encode a value that's too small for i32
        let value: i64 = i32::MIN as i64 - 1;
        let mut buf = vec![0u8; 20];
        let mut encode_index = 0;
        crate::encode_integers::encode_longlong(&mut Some(&mut buf), &mut encode_index, value).unwrap();
        
        let mut decode_index = 0;
        let result = decode_long(&buf, &mut decode_index);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DecodeError::ValueTooLarge);
    }

    #[test]
    fn test_decode_ulong() {
        let value = 1000u32;
        let mut buf = vec![0u8; 20];
        let mut encode_index = 0;
        crate::encode_integers::encode_ulong(&mut Some(&mut buf), &mut encode_index, value).unwrap();
        
        let mut decode_index = 0;
        let decoded = decode_ulong(&buf, &mut decode_index).unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    fn test_decode_ulong_value_too_large() {
        // Encode a value that's too large for u32
        let value: u64 = u32::MAX as u64 + 1;
        let mut buf = vec![0u8; 20];
        let mut encode_index = 0;
        crate::encode_integers::encode_ulonglong(&mut Some(&mut buf), &mut encode_index, value).unwrap();
        
        let mut decode_index = 0;
        let result = decode_ulong(&buf, &mut decode_index);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DecodeError::ValueTooLarge);
    }

    #[test]
    fn test_decode_error_debug() {
        let error1 = DecodeError::BufferTooShort;
        let error2 = DecodeError::InvalidFormat("test".to_string());
        let error3 = DecodeError::ValueTooLarge;
        
        let debug_str1 = format!("{:?}", error1);
        let debug_str2 = format!("{:?}", error2);
        let debug_str3 = format!("{:?}", error3);
        
        assert!(debug_str1.contains("BufferTooShort"));
        assert!(debug_str2.contains("InvalidFormat"));
        assert!(debug_str3.contains("ValueTooLarge"));
    }

    #[test]
    fn test_decode_error_clone() {
        let error1 = DecodeError::BufferTooShort;
        let error2 = DecodeError::InvalidFormat("test".to_string());
        let error3 = DecodeError::ValueTooLarge;
        
        let cloned1 = error1.clone();
        let cloned2 = error2.clone();
        let cloned3 = error3.clone();
        
        assert_eq!(error1, cloned1);
        assert_eq!(error2, cloned2);
        assert_eq!(error3, cloned3);
    }

    #[test]
    fn test_decode_error_partial_eq() {
        let error1 = DecodeError::BufferTooShort;
        let error2 = DecodeError::BufferTooShort;
        let error3 = DecodeError::InvalidFormat("test".to_string());
        let error4 = DecodeError::InvalidFormat("test".to_string());
        let error5 = DecodeError::InvalidFormat("different".to_string());
        let error6 = DecodeError::ValueTooLarge;
        
        assert_eq!(error1, error2);
        assert_eq!(error3, error4);
        assert_ne!(error1, error3);
        assert_ne!(error4, error5);
        assert_ne!(error3, error6);
    }

    #[test]
    fn test_decode_error_eq() {
        let error1 = DecodeError::BufferTooShort;
        let error2 = DecodeError::BufferTooShort;
        let error3 = DecodeError::ValueTooLarge;
        
        assert!(error1 == error2);
        assert!(error1 != error3);
    }

    #[test]
    fn test_decode_longlong_i64_max() {
        // Test decoding i64::MAX
        let value = i64::MAX;
        let mut buf = vec![0u8; 20];
        let mut encode_index = 0;
        crate::encode_integers::encode_longlong(&mut Some(&mut buf), &mut encode_index, value).unwrap();
        
        let mut decode_index = 0;
        let decoded = decode_longlong(&buf, &mut decode_index).unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    fn test_decode_longlong_i64_min_plus_one() {
        // Test decoding i64::MIN + 1 (i64::MIN itself causes overflow in negation)
        // i64::MIN + 1 = -0x7FFFFFFFFFFFFFFF
        let mut buf = vec![ERL_SMALL_BIG_EXT];
        buf.push(8); // arity = 8 bytes
        buf.push(1); // sign = negative
        // Value = 0x7FFFFFFFFFFFFFFF (i64::MIN + 1 when negated)
        buf.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F]);
        
        let mut index = 0;
        let decoded = decode_longlong(&buf, &mut index).unwrap();
        assert_eq!(decoded, i64::MIN + 1);
    }

    #[test]
    fn test_decode_ulonglong_u64_max() {
        // Test decoding u64::MAX
        let value = u64::MAX;
        let mut buf = vec![0u8; 20];
        let mut encode_index = 0;
        crate::encode_integers::encode_ulonglong(&mut Some(&mut buf), &mut encode_index, value).unwrap();
        
        let mut decode_index = 0;
        let decoded = decode_ulonglong(&buf, &mut decode_index).unwrap();
        assert_eq!(decoded, value);
    }
}

