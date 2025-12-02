//! Double Encoding Module
//!
//! Provides functionality to encode floating-point numbers to EI format.
//! Based on lib/erl_interface/src/encode/encode_double.c

use crate::constants::NEW_FLOAT_EXT;

/// Encode a double-precision floating-point number to EI format
///
/// # Arguments
/// * `buf` - Optional buffer to write to (None for size calculation)
/// * `index` - Current index in buffer
/// * `value` - The floating-point value to encode
///
/// # Returns
/// * `Ok(())` - Success
/// * `Err(EncodeError)` - Encoding error (e.g., NaN or Infinity)
pub fn encode_double(buf: &mut Option<&mut [u8]>, index: &mut usize, value: f64) -> Result<(), EncodeError> {
    // Erlang does not handle Inf and NaN
    if !value.is_finite() {
        return Err(EncodeError::InvalidValue);
    }

    if let Some(b) = buf.as_mut() {
        if *index + 9 > b.len() {
            return Err(EncodeError::BufferTooSmall);
        }
        b[*index] = NEW_FLOAT_EXT;
        // IEEE 754 format (big-endian)
        b[*index + 1..*index + 9].copy_from_slice(&value.to_bits().to_be_bytes());
    }
    *index += 9;

    Ok(())
}

/// Encoding errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodeError {
    /// Buffer is too small for the encoded value
    BufferTooSmall,
    /// Invalid value (NaN or Infinity)
    InvalidValue,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::NEW_FLOAT_EXT;

    #[test]
    fn test_encode_double() {
        let mut buf = vec![0u8; 10];
        let mut index = 0;
        encode_double(&mut Some(&mut buf), &mut index, 3.14).unwrap();
        assert_eq!(index, 9);
        assert_eq!(buf[0], NEW_FLOAT_EXT);
    }

    #[test]
    fn test_encode_double_nan() {
        let mut buf = vec![0u8; 10];
        let mut index = 0;
        assert!(encode_double(&mut Some(&mut buf), &mut index, f64::NAN).is_err());
    }

    #[test]
    fn test_encode_double_infinity() {
        let mut buf = vec![0u8; 10];
        let mut index = 0;
        assert!(encode_double(&mut Some(&mut buf), &mut index, f64::INFINITY).is_err());
    }

    #[test]
    fn test_encode_size_calculation() {
        let mut index = 0;
        encode_double(&mut None, &mut index, 3.14).unwrap();
        assert_eq!(index, 9);
    }
}

