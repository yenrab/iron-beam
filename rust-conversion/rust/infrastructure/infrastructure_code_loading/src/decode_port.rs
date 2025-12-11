//! Port Decoding Module
//!
//! Provides functionality to decode Ports from EI (Erlang Interface) format.
//! Ports represent external resources (files, sockets, etc.) that can communicate
//! with Erlang processes.
//!
//! ## Overview
//!
//! Ports in EI format consist of:
//! - **Node name**: Atom identifying the node where the port exists
//! - **Port ID**: Unique identifier for the port (32-bit or 64-bit depending on format)
//! - **Creation number**: Node creation number (2 bits in old format, 32 bits in new formats)
//!
//! ## Supported Formats
//!
//! - **ERL_PORT_EXT**: Old format with 2-bit creation number
//! - **ERL_NEW_PORT_EXT**: New format with 32-bit ID and 32-bit creation
//! - **ERL_V4_PORT_EXT**: V4 format with 64-bit ID and 32-bit creation
//!
//! ## Examples
//!
//! ```rust
//! use infrastructure_code_loading::decode_port;
//!
//! // Note: This example requires valid EI-encoded port data
//! // In practice, you would decode from a real buffer:
//! // let mut index = 0;
//! // let port = decode_port(&buf, &mut index)?;
//! // println!("Port: {} on node {}", port.id, port.node);
//! ```
//!
//! ## See Also
//!
//! - [`encode_port`](super::encode_port/index.html): Port encoding functions
//! - [`decode_pid`](super::decode_pid/index.html): PID decoding (similar structure)
//!
//! Based on `lib/erl_interface/src/decode/decode_port.c`

use crate::constants::{ERL_V4_PORT_EXT, ERL_NEW_PORT_EXT, ERL_PORT_EXT};
use infrastructure_data_handling::decode_atom::decode_atom;
use super::encode_port::ErlangPort;

/// Decode a port from EI format
///
/// # Arguments
/// * `buf` - Buffer containing EI-encoded data
/// * `index` - Current index in buffer
///
/// # Returns
/// * `Ok((port, new_index))` - Decoded port and new index
/// * `Err(DecodeError)` - Decoding error
pub fn decode_port(buf: &[u8], index: &mut usize) -> Result<ErlangPort, DecodeError> {
    if *index >= buf.len() {
        return Err(DecodeError::BufferTooShort);
    }

    let tag = buf[*index];
    *index += 1;

    match tag {
        ERL_V4_PORT_EXT | ERL_NEW_PORT_EXT | ERL_PORT_EXT => {}
        _ => return Err(DecodeError::InvalidFormat(format!("Unexpected tag: {}", tag))),
    }

    // Decode node atom
    let (node, new_pos) = decode_atom(buf, *index)
        .map_err(|e| DecodeError::AtomDecodeError(format!("{:?}", e)))?;
    *index = new_pos;

    // Decode id and creation based on format
    let (id, creation) = match tag {
        ERL_V4_PORT_EXT => {
            if *index + 12 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let id = u64::from_be_bytes([
                buf[*index], buf[*index + 1], buf[*index + 2], buf[*index + 3],
                buf[*index + 4], buf[*index + 5], buf[*index + 6], buf[*index + 7],
            ]);
            let creation = u32::from_be_bytes([
                buf[*index + 8], buf[*index + 9], buf[*index + 10], buf[*index + 11],
            ]);
            *index += 12;
            (id, creation)
        }
        ERL_NEW_PORT_EXT => {
            if *index + 8 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let id = u32::from_be_bytes([
                buf[*index], buf[*index + 1], buf[*index + 2], buf[*index + 3],
            ]) as u64;
            let creation = u32::from_be_bytes([
                buf[*index + 4], buf[*index + 5], buf[*index + 6], buf[*index + 7],
            ]);
            *index += 8;
            (id, creation)
        }
        ERL_PORT_EXT => {
            if *index + 5 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let id = u32::from_be_bytes([
                buf[*index], buf[*index + 1], buf[*index + 2], buf[*index + 3],
            ]) as u64;
            let creation = (buf[*index + 4] & 0x03) as u32;
            *index += 5;
            (id, creation)
        }
        _ => unreachable!(),
    };

    Ok(ErlangPort {
        node,
        id,
        creation,
    })
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_roundtrip() {
        let port = ErlangPort {
            node: "node@host".to_string(),
            id: 123,
            creation: 1,
        };
        let mut buf = vec![0u8; 100];
        let mut encode_index = 0;
        let mut buf_opt = Some(&mut buf[..]);
        super::super::encode_port::encode_port(&mut buf_opt, &mut encode_index, &port).unwrap();
        
        let mut decode_index = 0;
        let decoded = decode_port(&buf, &mut decode_index).unwrap();
        
        // Note: decode_atom returns a placeholder, so we can't compare node names
        // But we can verify id and creation match
        assert_eq!(decoded.id, port.id);
        assert_eq!(decoded.creation, port.creation);
    }

    #[test]
    fn test_decode_empty_buffer() {
        let buf = vec![];
        let mut index = 0;
        let result = decode_port(&buf, &mut index);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DecodeError::BufferTooShort);
    }

    #[test]
    fn test_decode_unexpected_tag() {
        let buf = vec![0xFF, 1, 2, 3];
        let mut index = 0;
        let result = decode_port(&buf, &mut index);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecodeError::InvalidFormat(msg) => {
                assert!(msg.contains("Unexpected tag"));
            }
            _ => panic!("Expected InvalidFormat error"),
        }
    }

    #[test]
    fn test_decode_atom_error() {
        // Create buffer with ERL_NEW_PORT_EXT tag but invalid atom data
        let mut buf = vec![ERL_NEW_PORT_EXT];
        // Add invalid atom tag (0xFF is not a valid atom tag)
        buf.push(0xFF);
        let mut index = 0;
        let result = decode_port(&buf, &mut index);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecodeError::AtomDecodeError(_) => {}
            _ => panic!("Expected AtomDecodeError"),
        }
    }

    #[test]
    fn test_decode_v4_port_ext() {
        // Test ERL_V4_PORT_EXT format with 64-bit id and 32-bit creation
        let mut buf = vec![ERL_V4_PORT_EXT];
        // Add valid small atom: SMALL_ATOM_EXT (115) + length 3 + "nod"
        buf.extend_from_slice(&[115, 3, b'n', b'o', b'd']);
        // id (8 bytes)
        buf.extend_from_slice(&12345678901234567890u64.to_be_bytes());
        // creation (4 bytes)
        buf.extend_from_slice(&1000u32.to_be_bytes());
        
        let mut index = 0;
        let decoded = decode_port(&buf, &mut index).unwrap();
        assert_eq!(decoded.id, 12345678901234567890);
        assert_eq!(decoded.creation, 1000);
    }

    #[test]
    fn test_decode_v4_port_ext_buffer_too_short() {
        // Create buffer with valid tag and atom, but buffer too short for id/creation
        let mut buf = vec![ERL_V4_PORT_EXT];
        buf.extend_from_slice(&[115, 3, b'n', b'o', b'd']);
        // Only add 11 bytes instead of 12
        buf.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        let mut index = 0;
        let result = decode_port(&buf, &mut index);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DecodeError::BufferTooShort);
    }

    #[test]
    fn test_decode_new_port_ext_buffer_too_short() {
        // Create buffer with valid tag and atom, but buffer too short for id/creation
        let mut buf = vec![ERL_NEW_PORT_EXT];
        buf.extend_from_slice(&[115, 3, b'n', b'o', b'd']);
        // Only add 7 bytes instead of 8
        buf.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0]);
        let mut index = 0;
        let result = decode_port(&buf, &mut index);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DecodeError::BufferTooShort);
    }

    #[test]
    fn test_decode_port_ext() {
        // Test ERL_PORT_EXT (old format) with 32-bit id and 2-bit creation
        let mut buf = vec![ERL_PORT_EXT];
        buf.extend_from_slice(&[115, 3, b'n', b'o', b'd']);
        // id (4 bytes)
        buf.extend_from_slice(&123u32.to_be_bytes());
        // creation (1 byte, only 2 bits used: 0x03 = 3)
        buf.push(0x03);
        
        let mut index = 0;
        let decoded = decode_port(&buf, &mut index).unwrap();
        assert_eq!(decoded.id, 123);
        assert_eq!(decoded.creation, 3); // Only 2 bits, so max is 3
    }

    #[test]
    fn test_decode_port_ext_buffer_too_short() {
        // Create buffer with valid tag and atom, but buffer too short for id/creation
        let mut buf = vec![ERL_PORT_EXT];
        buf.extend_from_slice(&[115, 3, b'n', b'o', b'd']);
        // Only add 4 bytes instead of 5
        buf.extend_from_slice(&[0, 0, 0, 0]);
        let mut index = 0;
        let result = decode_port(&buf, &mut index);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DecodeError::BufferTooShort);
    }

    #[test]
    fn test_decode_port_ext_creation_mask() {
        // Test that creation is properly masked to 2 bits
        let mut buf = vec![ERL_PORT_EXT];
        buf.extend_from_slice(&[115, 3, b'n', b'o', b'd']);
        buf.extend_from_slice(&123u32.to_be_bytes());
        // creation byte with upper bits set: 0xFF should become 0x03
        buf.push(0xFF);
        
        let mut index = 0;
        let decoded = decode_port(&buf, &mut index).unwrap();
        assert_eq!(decoded.creation, 3); // Only 2 bits, so 0xFF & 0x03 = 3
    }

    #[test]
    fn test_decode_error_debug() {
        let error1 = DecodeError::BufferTooShort;
        let error2 = DecodeError::InvalidFormat("test".to_string());
        let error3 = DecodeError::AtomDecodeError("atom_err".to_string());
        
        let debug_str1 = format!("{:?}", error1);
        let debug_str2 = format!("{:?}", error2);
        let debug_str3 = format!("{:?}", error3);
        
        assert!(debug_str1.contains("BufferTooShort"));
        assert!(debug_str2.contains("InvalidFormat"));
        assert!(debug_str3.contains("AtomDecodeError"));
    }

    #[test]
    fn test_decode_error_clone() {
        let error1 = DecodeError::BufferTooShort;
        let error2 = DecodeError::InvalidFormat("test".to_string());
        let error3 = DecodeError::AtomDecodeError("atom_err".to_string());
        
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
        let error6 = DecodeError::AtomDecodeError("err".to_string());
        
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
        let error3 = DecodeError::InvalidFormat("test".to_string());
        
        assert!(error1 == error2);
        assert!(error1 != error3);
    }

    #[test]
    fn test_decode_various_creations() {
        // Test various creation values for old format
        for creation in 0..=3 {
            let mut buf = vec![ERL_PORT_EXT];
            buf.extend_from_slice(&[115, 3, b'n', b'o', b'd']);
            buf.extend_from_slice(&123u32.to_be_bytes());
            buf.push(creation);
            
            let mut index = 0;
            let decoded = decode_port(&buf, &mut index).unwrap();
            assert_eq!(decoded.creation, creation as u32);
        }
    }

    #[test]
    fn test_decode_various_ids() {
        // Test various id values for different formats
        let test_cases = vec![
            (ERL_NEW_PORT_EXT, 0u32, 1u32),
            (ERL_NEW_PORT_EXT, 123u32, 456u32),
            (ERL_NEW_PORT_EXT, u32::MAX, u32::MAX),
        ];
        
        for (tag, id, creation) in test_cases {
            let mut buf = vec![tag];
            buf.extend_from_slice(&[115, 3, b'n', b'o', b'd']);
            buf.extend_from_slice(&id.to_be_bytes());
            buf.extend_from_slice(&creation.to_be_bytes());
            
            let mut index = 0;
            let decoded = decode_port(&buf, &mut index).unwrap();
            assert_eq!(decoded.id, id as u64);
            assert_eq!(decoded.creation, creation);
        }
    }

    #[test]
    fn test_decode_v4_port_ext_large_id() {
        // Test ERL_V4_PORT_EXT with large 64-bit id
        let large_id: u64 = u64::MAX;
        let mut buf = vec![ERL_V4_PORT_EXT];
        buf.extend_from_slice(&[115, 3, b'n', b'o', b'd']);
        buf.extend_from_slice(&large_id.to_be_bytes());
        buf.extend_from_slice(&1000u32.to_be_bytes());
        
        let mut index = 0;
        let decoded = decode_port(&buf, &mut index).unwrap();
        assert_eq!(decoded.id, large_id);
        assert_eq!(decoded.creation, 1000);
    }
}

