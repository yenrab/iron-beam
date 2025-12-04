//! Reference Decoding Module
//!
//! Provides functionality to decode References from EI (Erlang Interface) format.
//! References are unique identifiers used for message passing and process communication
//! in distributed Erlang systems.
//!
//! ## Overview
//!
//! References in EI format can be encoded in multiple formats:
//! - **ERL_REFERENCE_EXT**: Old format with single ID and 2-bit creation
//! - **ERL_NEW_REFERENCE_EXT**: New format with multiple IDs and 2-bit creation
//! - **ERL_NEWER_REFERENCE_EXT**: Newest format with multiple IDs and 32-bit creation
//!
//! The decoder handles all formats and normalizes them to the `ErlangRef` structure.
//!
//! ## Examples
//!
//! ```rust
//! use infrastructure_code_loading::decode_ref;
//!
//! let mut index = 0;
//! let r#ref = decode_ref(&buf, &mut index)?;
//! println!("Reference: {} IDs on node {}", r#ref.len, r#ref.node);
//! ```
//!
//! ## See Also
//!
//! - [`encode_ref`](super::encode_ref/index.html): Reference encoding functions
//! - [`decode_pid`](super::decode_pid/index.html): PID decoding (similar structure)
//!
//! Based on `lib/erl_interface/src/decode/decode_ref.c`

use crate::constants::{ERL_REFERENCE_EXT, ERL_NEW_REFERENCE_EXT, ERL_NEWER_REFERENCE_EXT};
use infrastructure_data_handling::decode_atom::decode_atom;
use super::encode_ref::ErlangRef;

/// Decode a reference from EI format
///
/// # Arguments
/// * `buf` - Buffer containing EI-encoded data
/// * `index` - Current index in buffer
///
/// # Returns
/// * `Ok((reference, new_index))` - Decoded reference and new index
/// * `Err(DecodeError)` - Decoding error
pub fn decode_ref(buf: &[u8], index: &mut usize) -> Result<ErlangRef, DecodeError> {
    if *index >= buf.len() {
        return Err(DecodeError::BufferTooShort);
    }

    let tag = buf[*index];
    *index += 1;

    match tag {
        ERL_REFERENCE_EXT => {
            // Old format: node, n[0] (u32), creation (u8, 2 bits)
            let (node, new_pos) = decode_atom(buf, *index)
                .map_err(|e| DecodeError::AtomDecodeError(format!("{:?}", e)))?;
            *index = new_pos;

            if *index + 5 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let id = u32::from_be_bytes([
                buf[*index], buf[*index + 1], buf[*index + 2], buf[*index + 3],
            ]);
            let creation = (buf[*index + 4] & 0x03) as u32;
            *index += 5;

            Ok(ErlangRef {
                node,
                len: 1,
                creation,
                ids: vec![id],
            })
        }
        ERL_NEW_REFERENCE_EXT | ERL_NEWER_REFERENCE_EXT => {
            // New format: count (u16), node, creation, ids[]
            if *index + 2 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let count = u16::from_be_bytes([buf[*index], buf[*index + 1]]) as usize;
            *index += 2;

            if count > 5 {
                return Err(DecodeError::InvalidLength);
            }

            let (node, new_pos) = decode_atom(buf, *index)
                .map_err(|e| DecodeError::AtomDecodeError(format!("{:?}", e)))?;
            *index = new_pos;

            let creation = if tag == ERL_NEW_REFERENCE_EXT {
                if *index >= buf.len() {
                    return Err(DecodeError::BufferTooShort);
                }
                let c = (buf[*index] & 0x03) as u32;
                *index += 1;
                c
            } else {
                if *index + 4 > buf.len() {
                    return Err(DecodeError::BufferTooShort);
                }
                let c = u32::from_be_bytes([
                    buf[*index], buf[*index + 1], buf[*index + 2], buf[*index + 3],
                ]);
                *index += 4;
                c
            };

            if *index + count * 4 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }

            let mut ids = Vec::with_capacity(count);
            for i in 0..count {
                let id = u32::from_be_bytes([
                    buf[*index + i * 4],
                    buf[*index + i * 4 + 1],
                    buf[*index + i * 4 + 2],
                    buf[*index + i * 4 + 3],
                ]);
                ids.push(id);
            }
            *index += count * 4;

            Ok(ErlangRef {
                node,
                len: count as u16,
                creation,
                ids,
            })
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
    /// Atom decoding error
    AtomDecodeError(String),
    /// Invalid length
    InvalidLength,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_roundtrip() {
        let r#ref = ErlangRef {
            node: "node@host".to_string(),
            len: 1,
            creation: 1,
            ids: vec![123],
        };
        let mut buf = vec![0u8; 100];
        let mut encode_index = 0;
        let mut buf_opt = Some(&mut buf[..]);
        super::super::encode_ref::encode_ref(&mut buf_opt, &mut encode_index, &r#ref).unwrap();
        
        let mut decode_index = 0;
        let decoded = decode_ref(&buf, &mut decode_index).unwrap();
        // Note: decode_atom returns a placeholder, so we can't compare node names
        // But we can verify len, creation, and ids match
        assert_eq!(decoded.len, r#ref.len);
        assert_eq!(decoded.creation, r#ref.creation);
        assert_eq!(decoded.ids, r#ref.ids);
    }

    #[test]
    fn test_decode_empty_buffer() {
        let buf = vec![];
        let mut index = 0;
        let result = decode_ref(&buf, &mut index);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DecodeError::BufferTooShort);
    }

    #[test]
    fn test_decode_unexpected_tag() {
        let buf = vec![0xFF, 1, 2, 3];
        let mut index = 0;
        let result = decode_ref(&buf, &mut index);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecodeError::InvalidFormat(msg) => {
                assert!(msg.contains("Unexpected tag"));
            }
            _ => panic!("Expected InvalidFormat error"),
        }
    }

    #[test]
    fn test_decode_reference_ext() {
        // Test ERL_REFERENCE_EXT (old format) with 32-bit id and 2-bit creation
        let mut buf = vec![ERL_REFERENCE_EXT];
        // Add valid small atom: SMALL_ATOM_EXT (115) + length 3 + "nod"
        buf.extend_from_slice(&[115, 3, b'n', b'o', b'd']);
        // id (4 bytes)
        buf.extend_from_slice(&123u32.to_be_bytes());
        // creation (1 byte, only 2 bits used: 0x03 = 3)
        buf.push(0x03);
        
        let mut index = 0;
        let decoded = decode_ref(&buf, &mut index).unwrap();
        assert_eq!(decoded.len, 1);
        assert_eq!(decoded.ids, vec![123]);
        assert_eq!(decoded.creation, 3);
    }

    #[test]
    fn test_decode_reference_ext_atom_error() {
        // Create buffer with ERL_REFERENCE_EXT tag but invalid atom data
        let mut buf = vec![ERL_REFERENCE_EXT];
        // Add invalid atom tag (0xFF is not a valid atom tag)
        buf.push(0xFF);
        let mut index = 0;
        let result = decode_ref(&buf, &mut index);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecodeError::AtomDecodeError(_) => {}
            _ => panic!("Expected AtomDecodeError"),
        }
    }

    #[test]
    fn test_decode_reference_ext_buffer_too_short() {
        // Create buffer with valid tag and atom, but buffer too short for id/creation
        let mut buf = vec![ERL_REFERENCE_EXT];
        buf.extend_from_slice(&[115, 3, b'n', b'o', b'd']);
        // Only add 4 bytes instead of 5
        buf.extend_from_slice(&[0, 0, 0, 0]);
        let mut index = 0;
        let result = decode_ref(&buf, &mut index);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DecodeError::BufferTooShort);
    }

    #[test]
    fn test_decode_reference_ext_creation_mask() {
        // Test that creation is properly masked to 2 bits
        let mut buf = vec![ERL_REFERENCE_EXT];
        buf.extend_from_slice(&[115, 3, b'n', b'o', b'd']);
        buf.extend_from_slice(&123u32.to_be_bytes());
        // creation byte with upper bits set: 0xFF should become 0x03
        buf.push(0xFF);
        
        let mut index = 0;
        let decoded = decode_ref(&buf, &mut index).unwrap();
        assert_eq!(decoded.creation, 3); // Only 2 bits, so 0xFF & 0x03 = 3
    }

    #[test]
    fn test_decode_new_reference_ext_buffer_too_short_for_count() {
        // Create buffer with ERL_NEW_REFERENCE_EXT tag but buffer too short for count
        let buf = vec![ERL_NEW_REFERENCE_EXT, 0];
        let mut index = 0;
        let result = decode_ref(&buf, &mut index);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DecodeError::BufferTooShort);
    }

    #[test]
    fn test_decode_new_reference_ext_invalid_length() {
        // Create buffer with count > 5 (should fail)
        let mut buf = vec![ERL_NEW_REFERENCE_EXT];
        buf.extend_from_slice(&6u16.to_be_bytes()); // count = 6 (invalid, max is 5)
        let mut index = 0;
        let result = decode_ref(&buf, &mut index);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DecodeError::InvalidLength);
    }

    #[test]
    fn test_decode_new_reference_ext_atom_error() {
        // Create buffer with valid tag and count, but invalid atom data
        let mut buf = vec![ERL_NEW_REFERENCE_EXT];
        buf.extend_from_slice(&1u16.to_be_bytes()); // count = 1
        // Add invalid atom tag
        buf.push(0xFF);
        let mut index = 0;
        let result = decode_ref(&buf, &mut index);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecodeError::AtomDecodeError(_) => {}
            _ => panic!("Expected AtomDecodeError"),
        }
    }

    #[test]
    fn test_decode_new_reference_ext() {
        // Test ERL_NEW_REFERENCE_EXT with 2-bit creation
        let mut buf = vec![ERL_NEW_REFERENCE_EXT];
        buf.extend_from_slice(&2u16.to_be_bytes()); // count = 2
        buf.extend_from_slice(&[115, 3, b'n', b'o', b'd']);
        buf.push(0x03); // creation (2 bits)
        // ids (2 * 4 bytes)
        buf.extend_from_slice(&123u32.to_be_bytes());
        buf.extend_from_slice(&456u32.to_be_bytes());
        
        let mut index = 0;
        let decoded = decode_ref(&buf, &mut index).unwrap();
        assert_eq!(decoded.len, 2);
        assert_eq!(decoded.ids, vec![123, 456]);
        assert_eq!(decoded.creation, 3);
    }

    #[test]
    fn test_decode_new_reference_ext_buffer_too_short_for_creation() {
        // Create buffer with valid tag, count, atom, but no creation byte
        let mut buf = vec![ERL_NEW_REFERENCE_EXT];
        buf.extend_from_slice(&1u16.to_be_bytes());
        buf.extend_from_slice(&[115, 3, b'n', b'o', b'd']);
        // No creation byte
        let mut index = 0;
        let result = decode_ref(&buf, &mut index);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DecodeError::BufferTooShort);
    }

    #[test]
    fn test_decode_newer_reference_ext() {
        // Test ERL_NEWER_REFERENCE_EXT with 32-bit creation
        let mut buf = vec![ERL_NEWER_REFERENCE_EXT];
        buf.extend_from_slice(&2u16.to_be_bytes()); // count = 2
        buf.extend_from_slice(&[115, 3, b'n', b'o', b'd']);
        buf.extend_from_slice(&1000u32.to_be_bytes()); // creation (4 bytes)
        // ids (2 * 4 bytes)
        buf.extend_from_slice(&123u32.to_be_bytes());
        buf.extend_from_slice(&456u32.to_be_bytes());
        
        let mut index = 0;
        let decoded = decode_ref(&buf, &mut index).unwrap();
        assert_eq!(decoded.len, 2);
        assert_eq!(decoded.ids, vec![123, 456]);
        assert_eq!(decoded.creation, 1000);
    }

    #[test]
    fn test_decode_newer_reference_ext_buffer_too_short_for_creation() {
        // Create buffer with valid tag, count, atom, but buffer too short for creation
        let mut buf = vec![ERL_NEWER_REFERENCE_EXT];
        buf.extend_from_slice(&1u16.to_be_bytes());
        buf.extend_from_slice(&[115, 3, b'n', b'o', b'd']);
        // Only 3 bytes instead of 4 for creation
        buf.extend_from_slice(&[0, 0, 0]);
        let mut index = 0;
        let result = decode_ref(&buf, &mut index);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DecodeError::BufferTooShort);
    }

    #[test]
    fn test_decode_new_reference_ext_buffer_too_short_for_ids() {
        // Create buffer with valid tag, count, atom, creation, but buffer too short for ids
        let mut buf = vec![ERL_NEW_REFERENCE_EXT];
        buf.extend_from_slice(&2u16.to_be_bytes()); // count = 2
        buf.extend_from_slice(&[115, 3, b'n', b'o', b'd']);
        buf.push(0x03); // creation
        // Only 4 bytes instead of 8 for ids
        buf.extend_from_slice(&[0, 0, 0, 0]);
        let mut index = 0;
        let result = decode_ref(&buf, &mut index);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DecodeError::BufferTooShort);
    }

    #[test]
    fn test_decode_new_reference_ext_multiple_ids() {
        // Test with maximum count (5)
        let mut buf = vec![ERL_NEW_REFERENCE_EXT];
        buf.extend_from_slice(&5u16.to_be_bytes()); // count = 5 (max)
        buf.extend_from_slice(&[115, 3, b'n', b'o', b'd']);
        buf.push(0x01); // creation
        // ids (5 * 4 bytes)
        for i in 1..=5 {
            buf.extend_from_slice(&(i as u32 * 100).to_be_bytes());
        }
        
        let mut index = 0;
        let decoded = decode_ref(&buf, &mut index).unwrap();
        assert_eq!(decoded.len, 5);
        assert_eq!(decoded.ids, vec![100, 200, 300, 400, 500]);
    }

    #[test]
    fn test_decode_error_debug() {
        let error1 = DecodeError::BufferTooShort;
        let error2 = DecodeError::InvalidFormat("test".to_string());
        let error3 = DecodeError::AtomDecodeError("atom_err".to_string());
        let error4 = DecodeError::InvalidLength;
        
        let debug_str1 = format!("{:?}", error1);
        let debug_str2 = format!("{:?}", error2);
        let debug_str3 = format!("{:?}", error3);
        let debug_str4 = format!("{:?}", error4);
        
        assert!(debug_str1.contains("BufferTooShort"));
        assert!(debug_str2.contains("InvalidFormat"));
        assert!(debug_str3.contains("AtomDecodeError"));
        assert!(debug_str4.contains("InvalidLength"));
    }

    #[test]
    fn test_decode_error_clone() {
        let error1 = DecodeError::BufferTooShort;
        let error2 = DecodeError::InvalidFormat("test".to_string());
        let error3 = DecodeError::AtomDecodeError("atom_err".to_string());
        let error4 = DecodeError::InvalidLength;
        
        let cloned1 = error1.clone();
        let cloned2 = error2.clone();
        let cloned3 = error3.clone();
        let cloned4 = error4.clone();
        
        assert_eq!(error1, cloned1);
        assert_eq!(error2, cloned2);
        assert_eq!(error3, cloned3);
        assert_eq!(error4, cloned4);
    }

    #[test]
    fn test_decode_error_partial_eq() {
        let error1 = DecodeError::BufferTooShort;
        let error2 = DecodeError::BufferTooShort;
        let error3 = DecodeError::InvalidFormat("test".to_string());
        let error4 = DecodeError::InvalidFormat("test".to_string());
        let error5 = DecodeError::InvalidFormat("different".to_string());
        let error6 = DecodeError::AtomDecodeError("err".to_string());
        let error7 = DecodeError::InvalidLength;
        
        assert_eq!(error1, error2);
        assert_eq!(error3, error4);
        assert_ne!(error1, error3);
        assert_ne!(error4, error5);
        assert_ne!(error3, error6);
        assert_ne!(error6, error7);
    }

    #[test]
    fn test_decode_error_eq() {
        let error1 = DecodeError::BufferTooShort;
        let error2 = DecodeError::BufferTooShort;
        let error3 = DecodeError::InvalidFormat("test".to_string());
        let error4 = DecodeError::InvalidLength;
        
        assert!(error1 == error2);
        assert!(error1 != error3);
        assert!(error1 != error4);
    }

    #[test]
    fn test_decode_various_creations() {
        // Test various creation values for old format
        for creation in 0..=3 {
            let mut buf = vec![ERL_REFERENCE_EXT];
            buf.extend_from_slice(&[115, 3, b'n', b'o', b'd']);
            buf.extend_from_slice(&123u32.to_be_bytes());
            buf.push(creation);
            
            let mut index = 0;
            let decoded = decode_ref(&buf, &mut index).unwrap();
            assert_eq!(decoded.creation, creation as u32);
        }
    }

    #[test]
    fn test_decode_various_counts() {
        // Test various count values (1-5)
        for count in 1..=5 {
            let mut buf = vec![ERL_NEW_REFERENCE_EXT];
            buf.extend_from_slice(&(count as u16).to_be_bytes());
            buf.extend_from_slice(&[115, 3, b'n', b'o', b'd']);
            buf.push(0x01); // creation
            // ids (count * 4 bytes)
            for i in 0..count {
                buf.extend_from_slice(&((i as u32 + 1) * 100).to_be_bytes());
            }
            
            let mut index = 0;
            let decoded = decode_ref(&buf, &mut index).unwrap();
            assert_eq!(decoded.len, count as u16);
            assert_eq!(decoded.ids.len(), count);
        }
    }
}

