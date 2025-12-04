//! Character Encoding Module
//!
//! Provides functionality to encode characters (u8 values) to EI (Erlang Interface)
//! format. Characters are encoded as small integers (0-255) for efficiency.
//!
//! ## Overview
//!
//! Characters in EI format are encoded using the `ERL_SMALL_INTEGER_EXT` tag followed
//! by the byte value. This provides a compact 2-byte encoding for character values.
//!
//! ## Examples
//!
//! ```rust
//! use infrastructure_code_loading::encode_char;
//!
//! let mut buf = vec![0u8; 10];
//! let mut index = 0;
//!
//! // Encode a character
//! encode_char(&mut Some(&mut buf), &mut index, b'A')?;
//! assert_eq!(index, 2);
//!
//! // Calculate size without encoding
//! let mut size_index = 0;
//! encode_char(&mut None, &mut size_index, b'Z')?;
//! assert_eq!(size_index, 2);
//! ```
//!
//! ## See Also
//!
//! - [`decode_char`](super::decode_char/index.html): Character decoding functions
//! - [`encode_integers`](super::encode_integers/index.html): Integer encoding (similar format)
//!
//! Based on `lib/erl_interface/src/encode/encode_char.c`

use crate::constants::ERL_SMALL_INTEGER_EXT;

/// Encode a character (u8) to EI format
///
/// # Arguments
/// * `buf` - Optional buffer to write to (None for size calculation)
/// * `index` - Current index in buffer
/// * `value` - The character value to encode (0-255)
///
/// # Returns
/// * `Ok(())` - Success
/// * `Err(EncodeError)` - Encoding error
pub fn encode_char(buf: &mut Option<&mut [u8]>, index: &mut usize, value: u8) -> Result<(), EncodeError> {
    if let Some(b) = buf.as_mut() {
        if *index + 2 > b.len() {
            return Err(EncodeError::BufferTooSmall);
        }
        b[*index] = ERL_SMALL_INTEGER_EXT;
        b[*index + 1] = value;
    }
    *index += 2;

    Ok(())
}

/// Encoding errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodeError {
    /// Buffer is too small for the encoded value
    BufferTooSmall,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::ERL_SMALL_INTEGER_EXT;

    #[test]
    fn test_encode_char() {
        let mut buf = vec![0u8; 10];
        let mut index = 0;
        encode_char(&mut Some(&mut buf), &mut index, 65).unwrap();
        assert_eq!(index, 2);
        assert_eq!(buf[0], ERL_SMALL_INTEGER_EXT);
        assert_eq!(buf[1], 65);
    }

    #[test]
    fn test_encode_size_calculation() {
        let mut index = 0;
        encode_char(&mut None, &mut index, 65).unwrap();
        assert_eq!(index, 2);
    }
}

