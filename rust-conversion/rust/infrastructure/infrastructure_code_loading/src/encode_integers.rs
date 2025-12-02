//! Integer Encoding Module
//!
//! Provides functionality to encode integers to EI format.
//! Based on lib/erl_interface/src/encode/encode_longlong.c and encode_ulonglong.c

use crate::constants::*;

/// Encode a signed 64-bit integer (i64) to EI format
///
/// # Arguments
/// * `buf` - Optional buffer to write to (None for size calculation)
/// * `index` - Current index in buffer
/// * `value` - The integer value to encode
///
/// # Returns
/// * `Ok(new_index)` - New index after encoding
/// * `Err(EncodeError)` - Encoding error
pub fn encode_longlong(buf: &mut Option<&mut [u8]>, index: &mut usize, value: i64) -> Result<(), EncodeError> {
    let abs_value = value.abs() as u64;
    let is_negative = value < 0;

    if value >= 0 && value < 256 {
        // Small integer (0-255)
        if let Some(b) = buf.as_mut() {
            if *index + 2 > b.len() {
                return Err(EncodeError::BufferTooSmall);
            }
            b[*index] = ERL_SMALL_INTEGER_EXT;
            b[*index + 1] = value as u8;
        }
        *index += 2;
    } else if value >= ERL_MIN && value <= ERL_MAX {
        // 32-bit integer
        if let Some(b) = buf.as_mut() {
            if *index + 5 > b.len() {
                return Err(EncodeError::BufferTooSmall);
            }
            b[*index] = ERL_INTEGER_EXT;
            b[*index + 1..*index + 5].copy_from_slice(&(value as i32).to_be_bytes());
        }
        *index += 5;
    } else {
        // Big integer (SMALL_BIG_EXT)
        let mut bytes = Vec::new();
        let mut v = abs_value;
        while v > 0 {
            bytes.push((v & 0xFF) as u8);
            v >>= 8;
        }
        let arity = bytes.len();
        
        if arity > 255 {
            return Err(EncodeError::ValueTooLarge);
        }

        if let Some(b) = buf.as_mut() {
            let needed = 3 + arity; // tag + arity + sign + bytes
            if *index + needed > b.len() {
                return Err(EncodeError::BufferTooSmall);
            }
            b[*index] = ERL_SMALL_BIG_EXT;
            b[*index + 1] = arity as u8;
            b[*index + 2] = if is_negative { 1 } else { 0 };
            b[*index + 3..*index + 3 + arity].copy_from_slice(&bytes);
        } else {
            *index += 3 + arity;
        }
    }

    Ok(())
}

/// Encode an unsigned 64-bit integer (u64) to EI format
///
/// # Arguments
/// * `buf` - Optional buffer to write to (None for size calculation)
/// * `index` - Current index in buffer
/// * `value` - The unsigned integer value to encode
///
/// # Returns
/// * `Ok(new_index)` - New index after encoding
/// * `Err(EncodeError)` - Encoding error
pub fn encode_ulonglong(buf: &mut Option<&mut [u8]>, index: &mut usize, value: u64) -> Result<(), EncodeError> {
    if value < 256 {
        // Small integer (0-255)
        if let Some(b) = buf.as_mut() {
            if *index + 2 > b.len() {
                return Err(EncodeError::BufferTooSmall);
            }
            b[*index] = ERL_SMALL_INTEGER_EXT;
            b[*index + 1] = value as u8;
        }
        *index += 2;
    } else if value <= ERL_MAX as u64 {
        // 32-bit integer
        if let Some(b) = buf.as_mut() {
            if *index + 5 > b.len() {
                return Err(EncodeError::BufferTooSmall);
            }
            b[*index] = ERL_INTEGER_EXT;
            b[*index + 1..*index + 5].copy_from_slice(&(value as i32).to_be_bytes());
        }
        *index += 5;
    } else {
        // Big integer (SMALL_BIG_EXT)
        let mut bytes = Vec::new();
        let mut v = value;
        while v > 0 {
            bytes.push((v & 0xFF) as u8);
            v >>= 8;
        }
        let arity = bytes.len();
        
        if arity > 255 {
            return Err(EncodeError::ValueTooLarge);
        }

        if let Some(b) = buf.as_mut() {
            let needed = 3 + arity; // tag + arity + sign + bytes
            if *index + needed > b.len() {
                return Err(EncodeError::BufferTooSmall);
            }
            b[*index] = ERL_SMALL_BIG_EXT;
            b[*index + 1] = arity as u8;
            b[*index + 2] = 0; // unsigned, always positive
            b[*index + 3..*index + 3 + arity].copy_from_slice(&bytes);
        } else {
            *index += 3 + arity;
        }
    }

    Ok(())
}

/// Encode a signed 32-bit integer (i32) to EI format
pub fn encode_long(buf: &mut Option<&mut [u8]>, index: &mut usize, value: i32) -> Result<(), EncodeError> {
    encode_longlong(buf, index, value as i64)
}

/// Encode an unsigned 32-bit integer (u32) to EI format
pub fn encode_ulong(buf: &mut Option<&mut [u8]>, index: &mut usize, value: u32) -> Result<(), EncodeError> {
    encode_ulonglong(buf, index, value as u64)
}

/// Encoding errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodeError {
    /// Buffer is too small for the encoded value
    BufferTooSmall,
    /// Value is too large to encode
    ValueTooLarge,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_longlong_small() {
        let mut buf = vec![0u8; 10];
        let mut index = 0;
        encode_longlong(&mut Some(&mut buf), &mut index, 42).unwrap();
        assert_eq!(index, 2);
        assert_eq!(buf[0], ERL_SMALL_INTEGER_EXT);
        assert_eq!(buf[1], 42);
    }

    #[test]
    fn test_encode_longlong_negative_small() {
        let mut buf = vec![0u8; 10];
        let mut index = 0;
        encode_longlong(&mut Some(&mut buf), &mut index, -42).unwrap();
        // Negative values in i32 range are encoded as INTEGER_EXT
        assert_eq!(buf[0], ERL_INTEGER_EXT);
        let value = i32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]);
        assert_eq!(value, -42);
    }

    #[test]
    fn test_encode_longlong_32bit() {
        let mut buf = vec![0u8; 10];
        let mut index = 0;
        encode_longlong(&mut Some(&mut buf), &mut index, 1000).unwrap();
        assert_eq!(index, 5);
        assert_eq!(buf[0], ERL_INTEGER_EXT);
        let value = i32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]);
        assert_eq!(value, 1000);
    }

    #[test]
    fn test_encode_longlong_big() {
        let mut buf = vec![0u8; 20];
        let mut index = 0;
        let large_value: i64 = 1_000_000_000_000;
        encode_longlong(&mut Some(&mut buf), &mut index, large_value).unwrap();
        assert_eq!(buf[0], ERL_SMALL_BIG_EXT);
        assert_eq!(buf[1], 5); // 5 bytes for this value
        assert_eq!(buf[2], 0); // positive
    }

    #[test]
    fn test_encode_ulonglong_small() {
        let mut buf = vec![0u8; 10];
        let mut index = 0;
        encode_ulonglong(&mut Some(&mut buf), &mut index, 42).unwrap();
        assert_eq!(index, 2);
        assert_eq!(buf[0], ERL_SMALL_INTEGER_EXT);
        assert_eq!(buf[1], 42);
    }

    #[test]
    fn test_encode_ulonglong_32bit() {
        let mut buf = vec![0u8; 10];
        let mut index = 0;
        encode_ulonglong(&mut Some(&mut buf), &mut index, 1000).unwrap();
        assert_eq!(index, 5);
        assert_eq!(buf[0], ERL_INTEGER_EXT);
    }

    #[test]
    fn test_encode_size_calculation() {
        let mut index = 0;
        encode_longlong(&mut None, &mut index, 42).unwrap();
        assert_eq!(index, 2);
        
        let mut index = 0;
        encode_longlong(&mut None, &mut index, 1000).unwrap();
        assert_eq!(index, 5);
    }

    #[test]
    fn test_encode_longlong_small_buffer_too_small() {
        let mut buf = vec![0u8; 1]; // Too small for small integer (needs 2 bytes)
        let mut index = 0;
        let result = encode_longlong(&mut Some(&mut buf), &mut index, 42);
        assert!(result.is_err());
        match result.unwrap_err() {
            EncodeError::BufferTooSmall => {}
            _ => panic!("Expected BufferTooSmall"),
        }
    }

    #[test]
    fn test_encode_longlong_32bit_buffer_too_small() {
        let mut buf = vec![0u8; 4]; // Too small for 32-bit integer (needs 5 bytes)
        let mut index = 0;
        let result = encode_longlong(&mut Some(&mut buf), &mut index, 1000);
        assert!(result.is_err());
        match result.unwrap_err() {
            EncodeError::BufferTooSmall => {}
            _ => panic!("Expected BufferTooSmall"),
        }
    }

    #[test]
    fn test_encode_longlong_big_buffer_too_small() {
        let mut buf = vec![0u8; 5]; // Too small for big integer
        let mut index = 0;
        let result = encode_longlong(&mut Some(&mut buf), &mut index, 1_000_000_000_000);
        assert!(result.is_err());
        match result.unwrap_err() {
            EncodeError::BufferTooSmall => {}
            _ => panic!("Expected BufferTooSmall"),
        }
    }

    #[test]
    fn test_encode_longlong_big_size_calculation() {
        let mut index = 0;
        encode_longlong(&mut None, &mut index, 1_000_000_000_000).unwrap();
        // Should calculate size: 3 + 5 = 8 bytes (tag + arity + sign + 5 bytes)
        assert_eq!(index, 8);
    }

    #[test]
    fn test_encode_longlong_negative_big() {
        let mut buf = vec![0u8; 20];
        let mut index = 0;
        let large_value: i64 = -1_000_000_000_000;
        encode_longlong(&mut Some(&mut buf), &mut index, large_value).unwrap();
        assert_eq!(buf[0], ERL_SMALL_BIG_EXT);
        assert_eq!(buf[1], 5); // 5 bytes for this value
        assert_eq!(buf[2], 1); // negative
    }

    #[test]
    fn test_encode_longlong_boundary_values() {
        // Test boundary: exactly 255 (should use small format)
        let mut buf = vec![0u8; 10];
        let mut index = 0;
        encode_longlong(&mut Some(&mut buf), &mut index, 255).unwrap();
        assert_eq!(buf[0], ERL_SMALL_INTEGER_EXT);
        assert_eq!(buf[1], 255);
        
        // Test boundary: exactly 256 (should use 32-bit format)
        let mut buf2 = vec![0u8; 10];
        let mut index2 = 0;
        encode_longlong(&mut Some(&mut buf2), &mut index2, 256).unwrap();
        assert_eq!(buf2[0], ERL_INTEGER_EXT);
        
        // Test boundary: exactly ERL_MAX (should use 32-bit format)
        let mut buf3 = vec![0u8; 10];
        let mut index3 = 0;
        encode_longlong(&mut Some(&mut buf3), &mut index3, ERL_MAX).unwrap();
        assert_eq!(buf3[0], ERL_INTEGER_EXT);
        
        // Test boundary: exactly ERL_MIN (should use 32-bit format)
        let mut buf4 = vec![0u8; 10];
        let mut index4 = 0;
        encode_longlong(&mut Some(&mut buf4), &mut index4, ERL_MIN).unwrap();
        assert_eq!(buf4[0], ERL_INTEGER_EXT);
        
        // Test boundary: ERL_MAX + 1 (should use big format)
        let mut buf5 = vec![0u8; 20];
        let mut index5 = 0;
        encode_longlong(&mut Some(&mut buf5), &mut index5, ERL_MAX + 1).unwrap();
        assert_eq!(buf5[0], ERL_SMALL_BIG_EXT);
    }

    #[test]
    fn test_encode_ulonglong_small_buffer_too_small() {
        let mut buf = vec![0u8; 1]; // Too small for small integer (needs 2 bytes)
        let mut index = 0;
        let result = encode_ulonglong(&mut Some(&mut buf), &mut index, 42);
        assert!(result.is_err());
        match result.unwrap_err() {
            EncodeError::BufferTooSmall => {}
            _ => panic!("Expected BufferTooSmall"),
        }
    }

    #[test]
    fn test_encode_ulonglong_32bit_buffer_too_small() {
        let mut buf = vec![0u8; 4]; // Too small for 32-bit integer (needs 5 bytes)
        let mut index = 0;
        let result = encode_ulonglong(&mut Some(&mut buf), &mut index, 1000);
        assert!(result.is_err());
        match result.unwrap_err() {
            EncodeError::BufferTooSmall => {}
            _ => panic!("Expected BufferTooSmall"),
        }
    }

    #[test]
    fn test_encode_ulonglong_big_buffer_too_small() {
        let mut buf = vec![0u8; 5]; // Too small for big integer
        let mut index = 0;
        let result = encode_ulonglong(&mut Some(&mut buf), &mut index, 1_000_000_000_000);
        assert!(result.is_err());
        match result.unwrap_err() {
            EncodeError::BufferTooSmall => {}
            _ => panic!("Expected BufferTooSmall"),
        }
    }

    #[test]
    fn test_encode_ulonglong_big_size_calculation() {
        let mut index = 0;
        encode_ulonglong(&mut None, &mut index, 1_000_000_000_000).unwrap();
        // Should calculate size: 3 + 5 = 8 bytes (tag + arity + sign + 5 bytes)
        assert_eq!(index, 8);
    }

    #[test]
    fn test_encode_ulonglong_small_size_calculation() {
        let mut index = 0;
        encode_ulonglong(&mut None, &mut index, 42).unwrap();
        assert_eq!(index, 2);
    }

    #[test]
    fn test_encode_ulonglong_32bit_size_calculation() {
        let mut index = 0;
        encode_ulonglong(&mut None, &mut index, 1000).unwrap();
        assert_eq!(index, 5);
    }

    #[test]
    fn test_encode_ulonglong_boundary_values() {
        // Test boundary: exactly 255 (should use small format)
        let mut buf = vec![0u8; 10];
        let mut index = 0;
        encode_ulonglong(&mut Some(&mut buf), &mut index, 255).unwrap();
        assert_eq!(buf[0], ERL_SMALL_INTEGER_EXT);
        assert_eq!(buf[1], 255);
        
        // Test boundary: exactly 256 (should use 32-bit format)
        let mut buf2 = vec![0u8; 10];
        let mut index2 = 0;
        encode_ulonglong(&mut Some(&mut buf2), &mut index2, 256).unwrap();
        assert_eq!(buf2[0], ERL_INTEGER_EXT);
        
        // Test boundary: exactly ERL_MAX as u64 (should use 32-bit format)
        let mut buf3 = vec![0u8; 10];
        let mut index3 = 0;
        encode_ulonglong(&mut Some(&mut buf3), &mut index3, ERL_MAX as u64).unwrap();
        assert_eq!(buf3[0], ERL_INTEGER_EXT);
        
        // Test boundary: ERL_MAX + 1 as u64 (should use big format)
        let mut buf4 = vec![0u8; 20];
        let mut index4 = 0;
        encode_ulonglong(&mut Some(&mut buf4), &mut index4, (ERL_MAX as u64) + 1).unwrap();
        assert_eq!(buf4[0], ERL_SMALL_BIG_EXT);
    }

    #[test]
    fn test_encode_long() {
        let mut buf = vec![0u8; 10];
        let mut index = 0;
        encode_long(&mut Some(&mut buf), &mut index, 42).unwrap();
        assert_eq!(buf[0], ERL_SMALL_INTEGER_EXT);
        assert_eq!(buf[1], 42);
    }

    #[test]
    fn test_encode_ulong() {
        let mut buf = vec![0u8; 10];
        let mut index = 0;
        encode_ulong(&mut Some(&mut buf), &mut index, 42).unwrap();
        assert_eq!(buf[0], ERL_SMALL_INTEGER_EXT);
        assert_eq!(buf[1], 42);
    }

    #[test]
    fn test_encode_error_debug() {
        let error1 = EncodeError::BufferTooSmall;
        let error2 = EncodeError::ValueTooLarge;
        
        let debug_str1 = format!("{:?}", error1);
        let debug_str2 = format!("{:?}", error2);
        
        assert!(debug_str1.contains("BufferTooSmall"));
        assert!(debug_str2.contains("ValueTooLarge"));
    }

    #[test]
    fn test_encode_error_clone() {
        let error1 = EncodeError::BufferTooSmall;
        let error2 = EncodeError::ValueTooLarge;
        
        let cloned1 = error1.clone();
        let cloned2 = error2.clone();
        
        assert_eq!(error1, cloned1);
        assert_eq!(error2, cloned2);
    }

    #[test]
    fn test_encode_error_copy() {
        let error1 = EncodeError::BufferTooSmall;
        let error2 = EncodeError::ValueTooLarge;
        
        let copied1 = error1; // Copy trait
        let copied2 = error2; // Copy trait
        
        assert_eq!(error1, copied1);
        assert_eq!(error2, copied2);
    }

    #[test]
    fn test_encode_error_partial_eq() {
        let error1 = EncodeError::BufferTooSmall;
        let error2 = EncodeError::BufferTooSmall;
        let error3 = EncodeError::ValueTooLarge;
        let error4 = EncodeError::ValueTooLarge;
        
        assert_eq!(error1, error2);
        assert_eq!(error3, error4);
        assert_ne!(error1, error3);
    }

    #[test]
    fn test_encode_error_eq() {
        let error1 = EncodeError::BufferTooSmall;
        let error2 = EncodeError::BufferTooSmall;
        let error3 = EncodeError::ValueTooLarge;
        
        assert!(error1 == error2);
        assert!(error1 != error3);
    }
}

