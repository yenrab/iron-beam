//! Character Decoding Module
//!
//! Provides functionality to decode characters (u8 values) from EI (Erlang Interface)
//! format. Characters can be encoded as small integers, regular integers, or big integers,
//! and this module handles all formats.
//!
//! ## Overview
//!
//! Characters in EI format can be encoded in multiple ways:
//! - **ERL_SMALL_INTEGER_EXT**: Single byte value (0-255) - most common
//! - **ERL_INTEGER_EXT**: 32-bit signed integer (must be 0-255)
//! - **ERL_SMALL_BIG_EXT** / **ERL_LARGE_BIG_EXT**: Big integer (must be 0-255)
//!
//! The decoder accepts all formats but validates that the result is in the valid
//! character range (0-255).
//!
//! ## Examples
//!
//! ```rust
//! use infrastructure_code_loading::decode_char;
//!
//! let mut index = 0;
//! let ch = decode_char(&buf, &mut index)?;
//! println!("Decoded character: {}", ch as char);
//! ```
//!
//! ## See Also
//!
//! - [`encode_char`](super::encode_char/index.html): Character encoding functions
//! - [`decode_integers`](super::decode_integers/index.html): Integer decoding (similar logic)
//!
//! Based on `lib/erl_interface/src/decode/decode_char.c`

use crate::constants::*;

/// Decode a character (u8) from EI format
///
/// # Arguments
/// * `buf` - Buffer containing EI-encoded data
/// * `index` - Current index in buffer
///
/// # Returns
/// * `Ok((value, new_index))` - Decoded value and new index
/// * `Err(DecodeError)` - Decoding error
pub fn decode_char(buf: &[u8], index: &mut usize) -> Result<u8, DecodeError> {
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
            let value = buf[*index];
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
            if signed < 0 || signed > 255 {
                return Err(DecodeError::ValueOutOfRange);
            }
            *index += 4;
            Ok(signed as u8)
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
                return Err(DecodeError::ValueOutOfRange); // Char is always > 0
            }

            if *index >= buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let value = buf[*index];
            *index += 1;

            // Check that remaining bytes are zero
            for _ in 1..arity {
                if *index >= buf.len() {
                    return Err(DecodeError::BufferTooShort);
                }
                if buf[*index] != 0 {
                    return Err(DecodeError::ValueOutOfRange);
                }
                *index += 1;
            }

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
    /// Value is out of range for u8 (0-255)
    ValueOutOfRange,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_char() {
        let buf = vec![ERL_SMALL_INTEGER_EXT, 65];
        let mut index = 0;
        let value = decode_char(&buf, &mut index).unwrap();
        assert_eq!(value, 65);
        assert_eq!(index, 2);
    }

    #[test]
    fn test_decode_roundtrip() {
        for original in 0..=255u8 {
            let mut buf = vec![0u8; 10];
            let mut encode_index = 0;
            crate::encode_char::encode_char(&mut Some(&mut buf), &mut encode_index, original).unwrap();
            
            let mut decode_index = 0;
            let decoded = decode_char(&buf, &mut decode_index).unwrap();
            assert_eq!(decoded, original, "Roundtrip failed for {}", original);
        }
    }
}

