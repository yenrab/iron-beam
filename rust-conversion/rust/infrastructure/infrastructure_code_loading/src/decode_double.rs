//! Double Decoding Module
//!
//! Provides functionality to decode floating-point numbers from EI format.
//! Based on lib/erl_interface/src/decode/decode_double.c

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
}

