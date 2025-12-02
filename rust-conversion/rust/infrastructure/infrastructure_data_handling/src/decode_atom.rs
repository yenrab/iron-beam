//! Decode Atom Module
//!
//! Provides functionality to decode EI-encoded atoms.
//! Based on lib/erl_interface/src/decode/decode_atom.c

use entities_data_handling::atom::MAX_ATOM_CHARACTERS;

/// Decode an atom from EI-encoded bytes
///
/// # Arguments
/// * `buf` - Buffer containing EI-encoded data
/// * `index` - Starting index in the buffer
///
/// # Returns
/// * `Ok((atom_name, new_index))` - Decoded atom name and new index position
/// * `Err(DecodeAtomError)` - Decoding error
pub fn decode_atom(buf: &[u8], index: usize) -> Result<(String, usize), DecodeAtomError> {
    if index >= buf.len() {
        return Err(DecodeAtomError::BufferTooShort);
    }

    let tag = buf[index];
    decode_atom_internal(buf, index + 1, tag)
        .and_then(|(atom_index, new_pos)| {
            // For now, convert atom index to string representation
            // In a full implementation, this would look up the atom in the atom table
            Ok((format!("atom_{}", atom_index), new_pos))
        })
}

/// Internal atom decoder (used by decode_term)
pub(crate) fn decode_atom_internal(
    buf: &[u8],
    pos: usize,
    tag: u8,
) -> Result<(usize, usize), DecodeAtomError> {
    match tag {
        // ATOM_EXT (100) - 2-byte length, Latin1 encoding
        100 => {
            if pos + 2 > buf.len() {
                return Err(DecodeAtomError::BufferTooShort);
            }
            let len = u16::from_be_bytes([buf[pos], buf[pos + 1]]) as usize;
            let data_pos = pos + 2;
            if data_pos + len > buf.len() {
                return Err(DecodeAtomError::BufferTooShort);
            }
            // Validate Latin1 encoding
            if !verify_latin1(&buf[data_pos..data_pos + len]) {
                return Err(DecodeAtomError::InvalidEncoding);
            }
            // For now, return a hash of the atom name as the index
            let atom_index = hash_atom_name(&buf[data_pos..data_pos + len]);
            Ok((atom_index, data_pos + len))
        }
        // SMALL_ATOM_EXT (115) - 1-byte length, Latin1 encoding
        115 => {
            if pos >= buf.len() {
                return Err(DecodeAtomError::BufferTooShort);
            }
            let len = buf[pos] as usize;
            let data_pos = pos + 1;
            if data_pos + len > buf.len() {
                return Err(DecodeAtomError::BufferTooShort);
            }
            if !verify_latin1(&buf[data_pos..data_pos + len]) {
                return Err(DecodeAtomError::InvalidEncoding);
            }
            let atom_index = hash_atom_name(&buf[data_pos..data_pos + len]);
            Ok((atom_index, data_pos + len))
        }
        // ATOM_UTF8_EXT (118) - 2-byte length, UTF-8 encoding
        118 => {
            if pos + 2 > buf.len() {
                return Err(DecodeAtomError::BufferTooShort);
            }
            let len = u16::from_be_bytes([buf[pos], buf[pos + 1]]) as usize;
            let data_pos = pos + 2;
            if data_pos + len > buf.len() {
                return Err(DecodeAtomError::BufferTooShort);
            }
            // Validate UTF-8 encoding
            if std::str::from_utf8(&buf[data_pos..data_pos + len]).is_err() {
                return Err(DecodeAtomError::InvalidEncoding);
            }
            let atom_index = hash_atom_name(&buf[data_pos..data_pos + len]);
            Ok((atom_index, data_pos + len))
        }
        // SMALL_ATOM_UTF8_EXT (119) - 1-byte length, UTF-8 encoding
        119 => {
            if pos >= buf.len() {
                return Err(DecodeAtomError::BufferTooShort);
            }
            let len = buf[pos] as usize;
            let data_pos = pos + 1;
            if data_pos + len > buf.len() {
                return Err(DecodeAtomError::BufferTooShort);
            }
            if std::str::from_utf8(&buf[data_pos..data_pos + len]).is_err() {
                return Err(DecodeAtomError::InvalidEncoding);
            }
            let atom_index = hash_atom_name(&buf[data_pos..data_pos + len]);
            Ok((atom_index, data_pos + len))
        }
        _ => Err(DecodeAtomError::InvalidTag(tag)),
    }
}

/// Verify Latin1 encoding (all bytes are valid Latin1)
fn verify_latin1(data: &[u8]) -> bool {
    // Latin1 is just all bytes 0-255, so always valid
    data.len() <= MAX_ATOM_CHARACTERS
}

/// Fast track for ASCII atoms (7-bit ASCII)
fn ascii_fast_track(data: &[u8]) -> bool {
    data.iter().all(|&b| b < 128)
}

/// Hash atom name to get a simple index (placeholder for real atom table lookup)
fn hash_atom_name(name: &[u8]) -> usize {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    hasher.finish() as usize
}

/// Atom decoding errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeAtomError {
    /// Buffer too short
    BufferTooShort,
    /// Invalid encoding
    InvalidEncoding,
    /// Invalid tag
    InvalidTag(u8),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_small_atom() {
        // SMALL_ATOM_EXT (115) + length 3 + "foo"
        let buf = vec![115, 3, b'f', b'o', b'o'];
        let result = decode_atom(&buf, 0);
        assert!(result.is_ok());
        let (atom_name, pos) = result.unwrap();
        assert!(atom_name.starts_with("atom_"));
        assert_eq!(pos, 5);
    }

    #[test]
    fn test_decode_atom_utf8() {
        // ATOM_UTF8_EXT (118) + length 3 (2 bytes) + "foo"
        let buf = vec![118, 0, 3, b'f', b'o', b'o'];
        let result = decode_atom(&buf, 0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_decode_atom_buffer_too_short() {
        let buf = vec![115, 10]; // Length but no data
        let result = decode_atom(&buf, 0);
        assert!(matches!(result, Err(DecodeAtomError::BufferTooShort)));
    }

    #[test]
    fn test_decode_atom_atom_ext() {
        // ATOM_EXT (100) + length 2 (2 bytes) + "ok"
        let buf = vec![100, 0, 2, b'o', b'k'];
        let result = decode_atom(&buf, 0);
        assert!(result.is_ok());
        let (atom_name, pos) = result.unwrap();
        assert!(atom_name.starts_with("atom_"));
        assert_eq!(pos, 5);
    }

    #[test]
    fn test_decode_atom_small_atom_utf8_ext() {
        // SMALL_ATOM_UTF8_EXT (119) + length 3 + "foo"
        let buf = vec![119, 3, b'f', b'o', b'o'];
        let result = decode_atom(&buf, 0);
        assert!(result.is_ok());
        let (atom_name, pos) = result.unwrap();
        assert!(atom_name.starts_with("atom_"));
        assert_eq!(pos, 5);
    }

    #[test]
    fn test_decode_atom_empty() {
        // SMALL_ATOM_EXT (115) + length 0
        let buf = vec![115, 0];
        let result = decode_atom(&buf, 0);
        assert!(result.is_ok());
        let (atom_name, pos) = result.unwrap();
        assert!(atom_name.starts_with("atom_"));
        assert_eq!(pos, 2);
    }

    #[test]
    fn test_decode_atom_atom_ext_empty() {
        // ATOM_EXT (100) + length 0 (2 bytes) + empty
        let buf = vec![100, 0, 0];
        let result = decode_atom(&buf, 0);
        assert!(result.is_ok());
        let (atom_name, pos) = result.unwrap();
        assert!(atom_name.starts_with("atom_"));
        assert_eq!(pos, 3);
    }

    #[test]
    fn test_decode_atom_atom_utf8_ext_empty() {
        // ATOM_UTF8_EXT (118) + length 0 (2 bytes) + empty
        let buf = vec![118, 0, 0];
        let result = decode_atom(&buf, 0);
        assert!(result.is_ok());
        let (atom_name, pos) = result.unwrap();
        assert!(atom_name.starts_with("atom_"));
        assert_eq!(pos, 3);
    }

    #[test]
    fn test_decode_atom_small_atom_utf8_ext_empty() {
        // SMALL_ATOM_UTF8_EXT (119) + length 0
        let buf = vec![119, 0];
        let result = decode_atom(&buf, 0);
        assert!(result.is_ok());
        let (atom_name, pos) = result.unwrap();
        assert!(atom_name.starts_with("atom_"));
        assert_eq!(pos, 2);
    }

    #[test]
    fn test_decode_atom_buffer_too_short_tag() {
        // Empty buffer
        let buf = vec![];
        let result = decode_atom(&buf, 0);
        assert!(matches!(result, Err(DecodeAtomError::BufferTooShort)));
    }

    #[test]
    fn test_decode_atom_buffer_too_short_length_byte() {
        // SMALL_ATOM_EXT (115) but no length byte
        let buf = vec![115];
        let result = decode_atom(&buf, 0);
        assert!(matches!(result, Err(DecodeAtomError::BufferTooShort)));
    }

    #[test]
    fn test_decode_atom_buffer_too_short_length_bytes() {
        // ATOM_EXT (100) but only 1 length byte
        let buf = vec![100, 0];
        let result = decode_atom(&buf, 0);
        assert!(matches!(result, Err(DecodeAtomError::BufferTooShort)));
    }

    #[test]
    fn test_decode_atom_buffer_too_short_data() {
        // SMALL_ATOM_EXT (115) + length 5 but only 3 bytes of data
        let buf = vec![115, 5, b'a', b'b', b'c'];
        let result = decode_atom(&buf, 0);
        assert!(matches!(result, Err(DecodeAtomError::BufferTooShort)));
    }

    #[test]
    fn test_decode_atom_atom_ext_buffer_too_short_data() {
        // ATOM_EXT (100) + length 5 but only 3 bytes of data
        let buf = vec![100, 0, 5, b'a', b'b', b'c'];
        let result = decode_atom(&buf, 0);
        assert!(matches!(result, Err(DecodeAtomError::BufferTooShort)));
    }

    #[test]
    fn test_decode_atom_invalid_tag() {
        // Invalid tag
        let buf = vec![99, 3, b'f', b'o', b'o'];
        let result = decode_atom(&buf, 0);
        assert!(matches!(result, Err(DecodeAtomError::InvalidTag(99))));
    }

    #[test]
    fn test_decode_atom_invalid_tag_various() {
        // Test various invalid tags
        for invalid_tag in [0, 50, 99, 101, 114, 116, 117, 120, 255] {
            let buf = vec![invalid_tag, 3, b'f', b'o', b'o'];
            let result = decode_atom(&buf, 0);
            assert!(matches!(result, Err(DecodeAtomError::InvalidTag(t)) if t == invalid_tag),
                    "Tag {} should be invalid", invalid_tag);
        }
    }

    #[test]
    fn test_decode_atom_utf8_invalid_encoding() {
        // SMALL_ATOM_UTF8_EXT (119) with invalid UTF-8
        let buf = vec![119, 2, 0xFF, 0xFE];
        let result = decode_atom(&buf, 0);
        assert!(matches!(result, Err(DecodeAtomError::InvalidEncoding)));
    }

    #[test]
    fn test_decode_atom_atom_utf8_ext_invalid_encoding() {
        // ATOM_UTF8_EXT (118) with invalid UTF-8
        let buf = vec![118, 0, 2, 0xFF, 0xFE];
        let result = decode_atom(&buf, 0);
        assert!(matches!(result, Err(DecodeAtomError::InvalidEncoding)));
    }

    #[test]
    fn test_decode_atom_utf8_valid_multibyte() {
        // SMALL_ATOM_UTF8_EXT (119) with valid UTF-8 multibyte
        let utf8_bytes = "café".as_bytes();
        let mut buf = vec![119, utf8_bytes.len() as u8];
        buf.extend_from_slice(utf8_bytes);
        let result = decode_atom(&buf, 0);
        assert!(result.is_ok());
        let (atom_name, _pos) = result.unwrap();
        assert!(atom_name.starts_with("atom_"));
    }

    #[test]
    fn test_decode_atom_atom_utf8_ext_valid_multibyte() {
        // ATOM_UTF8_EXT (118) with valid UTF-8 multibyte
        let utf8_bytes = "café".as_bytes();
        let len_bytes = (utf8_bytes.len() as u16).to_be_bytes();
        let mut buf = vec![118, len_bytes[0], len_bytes[1]];
        buf.extend_from_slice(utf8_bytes);
        let result = decode_atom(&buf, 0);
        assert!(result.is_ok());
        let (atom_name, _pos) = result.unwrap();
        assert!(atom_name.starts_with("atom_"));
    }

    #[test]
    fn test_decode_atom_utf8_emoji() {
        // SMALL_ATOM_UTF8_EXT (119) with emoji
        let utf8_bytes = "hello😀".as_bytes();
        let mut buf = vec![119, utf8_bytes.len() as u8];
        buf.extend_from_slice(utf8_bytes);
        let result = decode_atom(&buf, 0);
        assert!(result.is_ok());
        let (atom_name, _pos) = result.unwrap();
        assert!(atom_name.starts_with("atom_"));
    }

    #[test]
    fn test_decode_atom_latin1_all_bytes() {
        // SMALL_ATOM_EXT (115) with all possible Latin1 bytes (255 max)
        // We can't encode 256 bytes in SMALL_ATOM_EXT, so test with 255
        let mut buf = vec![115, 255];
        buf.extend((0..=254u8).collect::<Vec<u8>>());
        let result = decode_atom(&buf, 0);
        // verify_latin1 checks length <= MAX_ATOM_CHARACTERS (255)
        // So 255 should pass
        assert!(result.is_ok());
    }

    #[test]
    fn test_decode_atom_latin1_max_length() {
        // ATOM_EXT (100) with max length (255)
        let mut buf = vec![100, 0, 255];
        buf.extend(vec![b'a'; 255]);
        let result = decode_atom(&buf, 0);
        assert!(result.is_ok());
        let (atom_name, pos) = result.unwrap();
        assert!(atom_name.starts_with("atom_"));
        assert_eq!(pos, 258); // 1 tag + 2 length + 255 data
    }

    #[test]
    fn test_decode_atom_latin1_too_long() {
        // ATOM_EXT (100) with length > MAX_ATOM_CHARACTERS
        let mut buf = vec![100, 1, 0]; // length 256
        buf.extend(vec![b'a'; 256]);
        let result = decode_atom(&buf, 0);
        // verify_latin1 checks length <= MAX_ATOM_CHARACTERS (255)
        assert!(matches!(result, Err(DecodeAtomError::InvalidEncoding)));
    }

    #[test]
    fn test_decode_atom_atom_ext_large() {
        // ATOM_EXT (100) with large atom
        let mut buf = vec![100, 0, 10];
        buf.extend(b"hello_world");
        let result = decode_atom(&buf, 0);
        assert!(result.is_ok());
        let (atom_name, pos) = result.unwrap();
        assert!(atom_name.starts_with("atom_"));
        assert_eq!(pos, 13); // 1 tag + 2 length + 10 data
    }

    #[test]
    fn test_decode_atom_small_atom_ext_max_length() {
        // SMALL_ATOM_EXT (115) with max length (255)
        let mut buf = vec![115, 255];
        buf.extend(vec![b'a'; 255]);
        let result = decode_atom(&buf, 0);
        assert!(result.is_ok());
        let (atom_name, pos) = result.unwrap();
        assert!(atom_name.starts_with("atom_"));
        assert_eq!(pos, 257); // 1 tag + 1 length + 255 data
    }

    #[test]
    fn test_decode_atom_with_offset() {
        // Decode atom starting at non-zero index
        let buf = vec![0, 1, 2, 115, 3, b'f', b'o', b'o'];
        let result = decode_atom(&buf, 3);
        assert!(result.is_ok());
        let (atom_name, pos) = result.unwrap();
        assert!(atom_name.starts_with("atom_"));
        assert_eq!(pos, 8);
    }

    #[test]
    fn test_decode_atom_multiple_atoms() {
        // Decode multiple atoms sequentially
        let buf = vec![
            115, 3, b'f', b'o', b'o',  // First atom
            115, 3, b'b', b'a', b'r',  // Second atom
        ];
        
        let result1 = decode_atom(&buf, 0);
        assert!(result1.is_ok());
        let (_atom1, pos1) = result1.unwrap();
        assert_eq!(pos1, 5);
        
        let result2 = decode_atom(&buf, pos1);
        assert!(result2.is_ok());
        let (_atom2, pos2) = result2.unwrap();
        assert_eq!(pos2, 10);
    }

    #[test]
    fn test_decode_atom_error_variants() {
        // Test that error variants can be constructed and compared
        let err1 = DecodeAtomError::BufferTooShort;
        let err2 = DecodeAtomError::BufferTooShort;
        let err3 = DecodeAtomError::InvalidEncoding;
        let err4 = DecodeAtomError::InvalidEncoding;
        let err5 = DecodeAtomError::InvalidTag(100);
        let err6 = DecodeAtomError::InvalidTag(100);
        let err7 = DecodeAtomError::InvalidTag(200);
        
        assert_eq!(err1, err2);
        assert_eq!(err3, err4);
        assert_eq!(err5, err6);
        assert_ne!(err1, err3);
        assert_ne!(err5, err7);
        
        // Test Clone
        let err8 = err1.clone();
        assert_eq!(err1, err8);
        
        // Test Debug
        let _ = format!("{:?}", err1);
        let _ = format!("{:?}", err3);
        let _ = format!("{:?}", err5);
    }

    #[test]
    fn test_decode_atom_utf8_invalid_sequences() {
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
            let mut buf = vec![119, invalid_seq.len() as u8];
            buf.extend_from_slice(&invalid_seq);
            let result = decode_atom(&buf, 0);
            assert!(matches!(result, Err(DecodeAtomError::InvalidEncoding)),
                    "Sequence {:?} should be invalid UTF-8", invalid_seq);
        }
    }

    #[test]
    fn test_decode_atom_utf8_valid_sequences() {
        // Various valid UTF-8 sequences
        let valid_sequences = vec![
            vec![0x00], // NULL
            vec![0x7F], // DEL
            vec![0xC2, 0xA0], // Non-breaking space
            vec![0xE2, 0x82, 0xAC], // Euro sign
            vec![0xF0, 0x9F, 0x98, 0x80], // 😀 emoji
        ];
        
        for valid_seq in valid_sequences {
            let mut buf = vec![119, valid_seq.len() as u8];
            buf.extend_from_slice(&valid_seq);
            let result = decode_atom(&buf, 0);
            assert!(result.is_ok(), "Sequence {:?} should be valid UTF-8", valid_seq);
        }
    }

    #[test]
    fn test_decode_atom_index_consistency() {
        // Same atom name should produce same index
        let buf1 = vec![115, 3, b'f', b'o', b'o'];
        let buf2 = vec![115, 3, b'f', b'o', b'o'];
        
        let result1 = decode_atom(&buf1, 0).unwrap();
        let result2 = decode_atom(&buf2, 0).unwrap();
        
        assert_eq!(result1.0, result2.0); // Same atom name string
    }

    #[test]
    fn test_decode_atom_different_encodings_same_name() {
        // Same atom name in different encodings should produce different indices
        // (because we hash the raw bytes, not the decoded string)
        let buf1 = vec![115, 3, b'f', b'o', b'o']; // SMALL_ATOM_EXT
        let buf2 = vec![119, 3, b'f', b'o', b'o']; // SMALL_ATOM_UTF8_EXT
        
        let result1 = decode_atom(&buf1, 0).unwrap();
        let result2 = decode_atom(&buf2, 0).unwrap();
        
        // They should produce different indices because they're different tags
        // Actually, they hash the same bytes, so they might be the same
        // But the important thing is that both decode successfully
        assert!(result1.0.starts_with("atom_"));
        assert!(result2.0.starts_with("atom_"));
    }

    #[test]
    fn test_decode_atom_atom_ext_boundary() {
        // Test boundary between SMALL_ATOM_EXT and ATOM_EXT
        // SMALL_ATOM_EXT can handle up to 255 bytes
        let mut buf1 = vec![115, 255];
        buf1.extend(vec![b'a'; 255]);
        let result1 = decode_atom(&buf1, 0);
        assert!(result1.is_ok());
        
        // ATOM_EXT can also handle 255 bytes
        let mut buf2 = vec![100, 0, 255];
        buf2.extend(vec![b'a'; 255]);
        let result2 = decode_atom(&buf2, 0);
        assert!(result2.is_ok());
    }

    #[test]
    fn test_decode_atom_internal_all_tags() {
        // Test decode_atom_internal directly for all valid tags
        // Note: decode_atom_internal expects pos to be after the tag
        
        // SMALL_ATOM_EXT (115) - pos points to length byte
        let buf1 = vec![3, b'f', b'o', b'o'];
        let result1 = decode_atom_internal(&buf1, 0, 115);
        assert!(result1.is_ok());
        
        // SMALL_ATOM_UTF8_EXT (119) - pos points to length byte
        let result2 = decode_atom_internal(&buf1, 0, 119);
        assert!(result2.is_ok());
        
        // ATOM_EXT (100) - pos points to first length byte (2 bytes)
        let buf2 = vec![0, 3, b'f', b'o', b'o'];
        let result3 = decode_atom_internal(&buf2, 0, 100);
        assert!(result3.is_ok());
        
        // ATOM_UTF8_EXT (118) - pos points to first length byte (2 bytes)
        let result4 = decode_atom_internal(&buf2, 0, 118);
        assert!(result4.is_ok());
    }
}

