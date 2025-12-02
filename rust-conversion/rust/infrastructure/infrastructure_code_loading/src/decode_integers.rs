//! Integer Decoding Module
//!
//! Provides functionality to decode integers from EI format.
//! Based on lib/erl_interface/src/decode/decode_longlong.c and decode_ulonglong.c

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
}

