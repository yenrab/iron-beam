//! Encode Atom Module
//!
//! Provides functionality to encode atoms to EI format.
//! Based on lib/erl_interface/src/encode/encode_atom.c

use entities_data_handling::atom::{AtomEncoding, MAX_ATOM_CHARACTERS};

/// Encode an atom to EI format
///
/// # Arguments
/// * `buf` - Buffer to write encoded data to
/// * `atom_name` - Atom name as string
/// * `encoding` - Encoding type to use
///
/// # Returns
/// * `Ok(bytes_written)` - Number of bytes written
/// * `Err(EncodeAtomError)` - Encoding error
pub fn encode_atom(
    buf: &mut Vec<u8>,
    atom_name: &str,
    encoding: AtomEncoding,
) -> Result<usize, EncodeAtomError> {
    let name_bytes = atom_name.as_bytes();
    encode_atom_len(buf, name_bytes, encoding)
}

/// Encode an atom with explicit length
///
/// # Arguments
/// * `buf` - Buffer to write encoded data to
/// * `atom_name` - Atom name as bytes
/// * `encoding` - Encoding type to use
///
/// # Returns
/// * `Ok(bytes_written)` - Number of bytes written
/// * `Err(EncodeAtomError)` - Encoding error
pub fn encode_atom_len(
    buf: &mut Vec<u8>,
    atom_name: &[u8],
    encoding: AtomEncoding,
) -> Result<usize, EncodeAtomError> {
    // Validate atom name length
    if atom_name.len() > MAX_ATOM_CHARACTERS {
        return Err(EncodeAtomError::AtomTooLong);
    }

    // Verify encoding
    match encoding {
        AtomEncoding::SevenBitAscii => {
            if !verify_ascii_atom(atom_name) {
                return Err(EncodeAtomError::InvalidEncoding);
            }
        }
        AtomEncoding::Latin1 => {
            // Latin1 is always valid (all bytes 0-255)
        }
        AtomEncoding::Utf8 => {
            if !verify_utf8_atom(atom_name) {
                return Err(EncodeAtomError::InvalidEncoding);
            }
        }
    }

    let len = atom_name.len();
    let initial_len = buf.len();

    // Choose encoding format based on length and encoding type
    if len <= 255 {
        // Use SMALL_ATOM_EXT (115) or SMALL_ATOM_UTF8_EXT (119)
        match encoding {
            AtomEncoding::Utf8 => {
                buf.push(119); // SMALL_ATOM_UTF8_EXT
            }
            _ => {
                buf.push(115); // SMALL_ATOM_EXT (Latin1/ASCII)
            }
        }
        buf.push(len as u8);
    } else {
        // Use ATOM_EXT (100) or ATOM_UTF8_EXT (118)
        match encoding {
            AtomEncoding::Utf8 => {
                buf.push(118); // ATOM_UTF8_EXT
            }
            _ => {
                buf.push(100); // ATOM_EXT (Latin1/ASCII)
            }
        }
        buf.extend_from_slice(&(len as u16).to_be_bytes());
    }

    // Write atom name bytes
    buf.extend_from_slice(atom_name);

    Ok(buf.len() - initial_len)
}

/// Verify ASCII atom (7-bit ASCII only)
fn verify_ascii_atom(name: &[u8]) -> bool {
    name.iter().all(|&b| b < 128)
}

/// Verify UTF-8 atom
fn verify_utf8_atom(name: &[u8]) -> bool {
    std::str::from_utf8(name).is_ok()
}

/// Atom encoding errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodeAtomError {
    /// Atom name too long
    AtomTooLong,
    /// Invalid encoding
    InvalidEncoding,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_small_atom() {
        let mut buf = Vec::new();
        let result = encode_atom(&mut buf, "foo", AtomEncoding::Latin1);
        assert!(result.is_ok());
        assert_eq!(buf[0], 115); // SMALL_ATOM_EXT
        assert_eq!(buf[1], 3); // length
        assert_eq!(&buf[2..], b"foo");
    }

    #[test]
    fn test_encode_atom_utf8() {
        let mut buf = Vec::new();
        let result = encode_atom(&mut buf, "foo", AtomEncoding::Utf8);
        assert!(result.is_ok());
        assert_eq!(buf[0], 119); // SMALL_ATOM_UTF8_EXT
        assert_eq!(buf[1], 3); // length
        assert_eq!(&buf[2..], b"foo");
    }

    #[test]
    fn test_encode_atom_too_long() {
        let mut buf = Vec::new();
        let long_name = "a".repeat(MAX_ATOM_CHARACTERS + 1);
        let result = encode_atom(&mut buf, &long_name, AtomEncoding::Latin1);
        assert!(matches!(result, Err(EncodeAtomError::AtomTooLong)));
    }

    #[test]
    fn test_encode_atom_invalid_utf8() {
        let mut buf = Vec::new();
        let invalid_utf8 = vec![0xFF, 0xFE];
        let result = encode_atom_len(&mut buf, &invalid_utf8, AtomEncoding::Utf8);
        assert!(matches!(result, Err(EncodeAtomError::InvalidEncoding)));
    }

    #[test]
    fn test_encode_atom_seven_bit_ascii() {
        let mut buf = Vec::new();
        let result = encode_atom(&mut buf, "hello", AtomEncoding::SevenBitAscii);
        assert!(result.is_ok());
        assert_eq!(buf[0], 115); // SMALL_ATOM_EXT
        assert_eq!(buf[1], 5); // length
        assert_eq!(&buf[2..], b"hello");
    }

    #[test]
    fn test_encode_atom_seven_bit_ascii_invalid() {
        let mut buf = Vec::new();
        // Byte 128 is invalid for 7-bit ASCII
        let invalid_ascii = vec![0x80];
        let result = encode_atom_len(&mut buf, &invalid_ascii, AtomEncoding::SevenBitAscii);
        assert!(matches!(result, Err(EncodeAtomError::InvalidEncoding)));
    }

    #[test]
    fn test_encode_atom_seven_bit_ascii_boundary() {
        let mut buf = Vec::new();
        // Byte 127 is valid (max 7-bit)
        let valid_ascii = vec![0x7F];
        let result = encode_atom_len(&mut buf, &valid_ascii, AtomEncoding::SevenBitAscii);
        assert!(result.is_ok());
        assert_eq!(buf[0], 115); // SMALL_ATOM_EXT
        assert_eq!(buf[1], 1); // length
        assert_eq!(buf[2], 0x7F);
    }

    #[test]
    fn test_encode_atom_latin1() {
        let mut buf = Vec::new();
        // Latin1 can include bytes 0-255
        let latin1_bytes = vec![0xFF, 0x80, 0x00];
        let result = encode_atom_len(&mut buf, &latin1_bytes, AtomEncoding::Latin1);
        assert!(result.is_ok());
        assert_eq!(buf[0], 115); // SMALL_ATOM_EXT
        assert_eq!(buf[1], 3); // length
        assert_eq!(&buf[2..], &[0xFF, 0x80, 0x00]);
    }

    #[test]
    fn test_encode_atom_empty() {
        let mut buf = Vec::new();
        let result = encode_atom(&mut buf, "", AtomEncoding::Latin1);
        assert!(result.is_ok());
        assert_eq!(buf[0], 115); // SMALL_ATOM_EXT
        assert_eq!(buf[1], 0); // length
        assert_eq!(buf.len(), 2);
    }

    #[test]
    fn test_encode_atom_exactly_255_bytes() {
        let mut buf = Vec::new();
        let atom_name = vec![b'a'; 255];
        let result = encode_atom_len(&mut buf, &atom_name, AtomEncoding::Latin1);
        assert!(result.is_ok());
        assert_eq!(buf[0], 115); // SMALL_ATOM_EXT (<= 255 uses small format)
        assert_eq!(buf[1], 255); // length
        assert_eq!(buf.len(), 257); // 1 tag + 1 length + 255 data
    }

    #[test]
    fn test_encode_atom_exactly_256_bytes() {
        let mut buf = Vec::new();
        let atom_name = vec![b'a'; 256];
        let result = encode_atom_len(&mut buf, &atom_name, AtomEncoding::Latin1);
        // MAX_ATOM_CHARACTERS is 255, so 256 should fail
        assert!(matches!(result, Err(EncodeAtomError::AtomTooLong)));
    }

    #[test]
    fn test_encode_atom_large_latin1() {
        let mut buf = Vec::new();
        let atom_name = vec![b'x'; 500];
        let result = encode_atom_len(&mut buf, &atom_name, AtomEncoding::Latin1);
        // MAX_ATOM_CHARACTERS is 255, so 500 should fail
        assert!(matches!(result, Err(EncodeAtomError::AtomTooLong)));
    }

    #[test]
    fn test_encode_atom_large_utf8() {
        let mut buf = Vec::new();
        // Create a valid UTF-8 string of 300 characters (300 bytes for 'a')
        let atom_name = "a".repeat(300);
        let result = encode_atom(&mut buf, &atom_name, AtomEncoding::Utf8);
        // MAX_ATOM_CHARACTERS is 255, so 300 should fail
        assert!(matches!(result, Err(EncodeAtomError::AtomTooLong)));
    }

    #[test]
    fn test_encode_atom_utf8_255_bytes() {
        let mut buf = Vec::new();
        // Create a valid UTF-8 string of exactly 255 bytes
        let atom_name = "a".repeat(255);
        let result = encode_atom(&mut buf, &atom_name, AtomEncoding::Utf8);
        assert!(result.is_ok());
        assert_eq!(buf[0], 119); // SMALL_ATOM_UTF8_EXT (<= 255)
        assert_eq!(buf[1], 255); // length
        assert_eq!(&buf[2..], atom_name.as_bytes());
    }

    #[test]
    fn test_encode_atom_utf8_256_bytes() {
        let mut buf = Vec::new();
        // Create a valid UTF-8 string of exactly 256 bytes
        let atom_name = "a".repeat(256);
        let result = encode_atom(&mut buf, &atom_name, AtomEncoding::Utf8);
        // MAX_ATOM_CHARACTERS is 255, so 256 should fail
        assert!(matches!(result, Err(EncodeAtomError::AtomTooLong)));
    }

    #[test]
    fn test_encode_atom_seven_bit_ascii_large() {
        let mut buf = Vec::new();
        let atom_name = "a".repeat(300);
        let result = encode_atom(&mut buf, &atom_name, AtomEncoding::SevenBitAscii);
        // MAX_ATOM_CHARACTERS is 255, so 300 should fail
        assert!(matches!(result, Err(EncodeAtomError::AtomTooLong)));
    }

    #[test]
    fn test_encode_atom_bytes_written() {
        let mut buf = Vec::new();
        let result = encode_atom(&mut buf, "test", AtomEncoding::Latin1);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 6); // 1 tag + 1 length + 4 data
    }

    #[test]
    fn test_encode_atom_bytes_written_large() {
        let mut buf = Vec::new();
        let atom_name = vec![b'a'; 500];
        let result = encode_atom_len(&mut buf, &atom_name, AtomEncoding::Latin1);
        // MAX_ATOM_CHARACTERS is 255, so 500 should fail
        assert!(matches!(result, Err(EncodeAtomError::AtomTooLong)));
    }

    #[test]
    fn test_encode_atom_with_existing_buffer() {
        let mut buf = vec![1, 2, 3];
        let initial_len = buf.len();
        let result = encode_atom(&mut buf, "foo", AtomEncoding::Latin1);
        assert!(result.is_ok());
        assert_eq!(buf.len(), initial_len + 5); // 1 tag + 1 length + 3 data
        assert_eq!(&buf[0..3], &[1, 2, 3]); // Original content preserved
        assert_eq!(&buf[3..], &[115, 3, b'f', b'o', b'o']);
    }

    #[test]
    fn test_encode_atom_utf8_multibyte() {
        let mut buf = Vec::new();
        // UTF-8 string with multibyte characters
        let atom_name = "café";
        let result = encode_atom(&mut buf, atom_name, AtomEncoding::Utf8);
        assert!(result.is_ok());
        assert_eq!(buf[0], 119); // SMALL_ATOM_UTF8_EXT
        assert_eq!(buf[1], 5); // length (café is 5 bytes in UTF-8)
        assert_eq!(&buf[2..], atom_name.as_bytes());
    }

    #[test]
    fn test_encode_atom_utf8_emoji() {
        let mut buf = Vec::new();
        // UTF-8 string with emoji
        let atom_name = "hello😀";
        let result = encode_atom(&mut buf, atom_name, AtomEncoding::Utf8);
        assert!(result.is_ok());
        assert_eq!(buf[0], 119); // SMALL_ATOM_UTF8_EXT
        // "hello" is 5 bytes, 😀 is 4 bytes = 9 total
        assert_eq!(buf[1], 9); // length
        assert_eq!(&buf[2..], atom_name.as_bytes());
    }

    #[test]
    fn test_encode_atom_max_length() {
        let mut buf = Vec::new();
        let atom_name = vec![b'a'; MAX_ATOM_CHARACTERS];
        let result = encode_atom_len(&mut buf, &atom_name, AtomEncoding::Latin1);
        assert!(result.is_ok());
        // MAX_ATOM_CHARACTERS is 255, so should use SMALL_ATOM_EXT (<= 255)
        assert_eq!(buf[0], 115); // SMALL_ATOM_EXT
        assert_eq!(buf[1], 255); // length
    }

    #[test]
    fn test_encode_atom_max_length_plus_one() {
        let mut buf = Vec::new();
        let atom_name = vec![b'a'; MAX_ATOM_CHARACTERS + 1];
        let result = encode_atom_len(&mut buf, &atom_name, AtomEncoding::Latin1);
        assert!(matches!(result, Err(EncodeAtomError::AtomTooLong)));
    }

    #[test]
    fn test_encode_atom_invalid_ascii_high_byte() {
        let mut buf = Vec::new();
        // Test various invalid ASCII bytes
        for invalid_byte in 128..=255 {
            let invalid_ascii = vec![invalid_byte];
            let result = encode_atom_len(&mut buf, &invalid_ascii, AtomEncoding::SevenBitAscii);
            assert!(matches!(result, Err(EncodeAtomError::InvalidEncoding)), 
                    "Byte {} should be invalid for 7-bit ASCII", invalid_byte);
            buf.clear();
        }
    }

    #[test]
    fn test_encode_atom_verify_ascii_all_valid() {
        let mut buf = Vec::new();
        // Test all valid ASCII bytes (0-127)
        for valid_byte in 0..=127 {
            let valid_ascii = vec![valid_byte];
            let result = encode_atom_len(&mut buf, &valid_ascii, AtomEncoding::SevenBitAscii);
            assert!(result.is_ok(), "Byte {} should be valid for 7-bit ASCII", valid_byte);
            buf.clear();
        }
    }

    #[test]
    fn test_encode_atom_error_variants() {
        // Test that error variants can be constructed and compared
        let err1 = EncodeAtomError::AtomTooLong;
        let err2 = EncodeAtomError::AtomTooLong;
        let err3 = EncodeAtomError::InvalidEncoding;
        let err4 = EncodeAtomError::InvalidEncoding;
        
        assert_eq!(err1, err2);
        assert_eq!(err3, err4);
        assert_ne!(err1, err3);
        
        // Test Clone
        let err5 = err1.clone();
        assert_eq!(err1, err5);
        
        // Test Debug
        let _ = format!("{:?}", err1);
        let _ = format!("{:?}", err3);
    }

    #[test]
    fn test_encode_atom_utf8_invalid_sequences() {
        let mut buf = Vec::new();
        // Various invalid UTF-8 sequences
        let invalid_sequences = vec![
            vec![0xFF, 0xFE], // Invalid start byte
            vec![0xC0, 0x80], // Overlong encoding
            vec![0xE0, 0x80, 0x80], // Overlong encoding
            vec![0xF0, 0x80, 0x80, 0x80], // Overlong encoding
            vec![0xC2], // Incomplete sequence
            vec![0xE0, 0x80], // Incomplete sequence
            vec![0xF0, 0x80, 0x80], // Incomplete sequence
        ];
        
        for invalid_seq in invalid_sequences {
            let result = encode_atom_len(&mut buf, &invalid_seq, AtomEncoding::Utf8);
            assert!(matches!(result, Err(EncodeAtomError::InvalidEncoding)),
                    "Sequence {:?} should be invalid UTF-8", invalid_seq);
            buf.clear();
        }
    }

    #[test]
    fn test_encode_atom_utf8_valid_sequences() {
        let mut buf = Vec::new();
        // Various valid UTF-8 sequences
        let valid_sequences = vec![
            vec![0x00], // NULL
            vec![0x7F], // DEL
            vec![0xC2, 0xA0], // Non-breaking space
            vec![0xE2, 0x82, 0xAC], // Euro sign
            vec![0xF0, 0x9F, 0x98, 0x80], // 😀 emoji
        ];
        
        for valid_seq in valid_sequences {
            let result = encode_atom_len(&mut buf, &valid_seq, AtomEncoding::Utf8);
            assert!(result.is_ok(), "Sequence {:?} should be valid UTF-8", valid_seq);
            buf.clear();
        }
    }

    #[test]
    fn test_encode_atom_boundary_255_256() {
        // Test the boundary - 255 is max allowed, 256 should fail
        let mut buf1 = Vec::new();
        let atom_255 = vec![b'a'; 255];
        let result1 = encode_atom_len(&mut buf1, &atom_255, AtomEncoding::Latin1);
        assert!(result1.is_ok());
        assert_eq!(buf1[0], 115); // SMALL_ATOM_EXT
        
        let mut buf2 = Vec::new();
        let atom_256 = vec![b'a'; 256];
        let result2 = encode_atom_len(&mut buf2, &atom_256, AtomEncoding::Latin1);
        // MAX_ATOM_CHARACTERS is 255, so 256 should fail
        assert!(matches!(result2, Err(EncodeAtomError::AtomTooLong)));
    }

    #[test]
    fn test_encode_atom_utf8_boundary_255_256() {
        // Test the boundary for UTF-8 encoding - 255 is max, 256 should fail
        let mut buf1 = Vec::new();
        let atom_255 = "a".repeat(255);
        let result1 = encode_atom(&mut buf1, &atom_255, AtomEncoding::Utf8);
        assert!(result1.is_ok());
        assert_eq!(buf1[0], 119); // SMALL_ATOM_UTF8_EXT
        
        let mut buf2 = Vec::new();
        let atom_256 = "a".repeat(256);
        let result2 = encode_atom(&mut buf2, &atom_256, AtomEncoding::Utf8);
        // MAX_ATOM_CHARACTERS is 255, so 256 should fail
        assert!(matches!(result2, Err(EncodeAtomError::AtomTooLong)));
    }
}

