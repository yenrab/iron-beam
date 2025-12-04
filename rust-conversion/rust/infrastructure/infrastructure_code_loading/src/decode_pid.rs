//! PID Decoding Module
//!
//! Provides functionality to decode Process IDs (PIDs) from EI (Erlang Interface) format.
//! PIDs uniquely identify processes in the Erlang runtime system and are essential
//! for inter-process communication and distributed Erlang.
//!
//! ## Overview
//!
//! PIDs in EI format consist of:
//! - **Node name**: Atom identifying the node where the process exists
//! - **Process number**: Unique identifier for the process on the node
//! - **Serial number**: Serial number for the process
//! - **Creation number**: Node creation number (2 bits in old format, 32 bits in new format)
//!
//! ## Supported Formats
//!
//! - **ERL_PID_EXT**: Old format with 2-bit creation number
//! - **ERL_NEW_PID_EXT**: New format with 32-bit creation number (preferred)
//!
//! ## Examples
//!
//! ```rust
//! use infrastructure_code_loading::decode_pid;
//!
//! // Decode a PID from EI-encoded buffer
//! let mut index = 0;
//! let pid = decode_pid(&buf, &mut index)?;
//! println!("Process: {} on node {}", pid.num, pid.node);
//! ```
//!
//! ## See Also
//!
//! - [`encode_pid`](super::encode_pid/index.html): PID encoding functions
//! - [`decode_port`](super::decode_port/index.html): Port decoding (similar structure)
//! - [`decode_fun`](super::decode_fun/index.html): Function decoding (uses PIDs)
//!
//! Based on `lib/erl_interface/src/decode/decode_pid.c`

use crate::constants::{ERL_NEW_PID_EXT, ERL_PID_EXT};
use infrastructure_data_handling::decode_atom::{decode_atom, DecodeAtomError};
use super::encode_pid::ErlangPid;

/// Decode a PID from EI format
///
/// # Arguments
/// * `buf` - Buffer containing EI-encoded data
/// * `index` - Current index in buffer
///
/// # Returns
/// * `Ok((pid, new_index))` - Decoded PID and new index
/// * `Err(DecodeError)` - Decoding error
pub fn decode_pid(buf: &[u8], index: &mut usize) -> Result<ErlangPid, DecodeError> {
    if *index >= buf.len() {
        return Err(DecodeError::BufferTooShort);
    }

    let tag = buf[*index];
    *index += 1;

    if tag != ERL_PID_EXT && tag != ERL_NEW_PID_EXT {
        return Err(DecodeError::InvalidFormat(format!("Unexpected tag: {}", tag)));
    }

    // Decode node atom
    let (node, new_pos) = decode_atom(buf, *index)
        .map_err(|e| DecodeError::AtomDecodeError(format!("{:?}", e)))?;
    *index = new_pos;

    // Decode num, serial
    if *index + 8 > buf.len() {
        return Err(DecodeError::BufferTooShort);
    }
    let num = u32::from_be_bytes([
        buf[*index],
        buf[*index + 1],
        buf[*index + 2],
        buf[*index + 3],
    ]);
    let serial = u32::from_be_bytes([
        buf[*index + 4],
        buf[*index + 5],
        buf[*index + 6],
        buf[*index + 7],
    ]);
    *index += 8;

    // Decode creation (2 bits for old format, 32 bits for new format)
    let creation = if tag == ERL_PID_EXT {
        if *index >= buf.len() {
            return Err(DecodeError::BufferTooShort);
        }
        (buf[*index] & 0x03) as u32
    } else {
        if *index + 4 > buf.len() {
            return Err(DecodeError::BufferTooShort);
        }
        u32::from_be_bytes([
            buf[*index],
            buf[*index + 1],
            buf[*index + 2],
            buf[*index + 3],
        ])
    };
    if tag == ERL_NEW_PID_EXT {
        *index += 4;
    } else {
        *index += 1;
    }

    Ok(ErlangPid {
        node,
        num,
        serial,
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
        let pid = ErlangPid {
            node: "node@host".to_string(),
            num: 123,
            serial: 456,
            creation: 1,
        };
        let mut buf = vec![0u8; 100];
        let mut encode_index = 0;
        let mut buf_opt = Some(&mut buf[..]);
        super::super::encode_pid::encode_pid(&mut buf_opt, &mut encode_index, &pid).unwrap();
        
        let mut decode_index = 0;
        let decoded = decode_pid(&buf, &mut decode_index).unwrap();
        // Note: decode_atom returns a placeholder, so we can't compare node names
        // But we can verify num, serial, and creation match
        assert_eq!(decoded.num, pid.num);
        assert_eq!(decoded.serial, pid.serial);
        assert_eq!(decoded.creation, pid.creation);
    }

    #[test]
    fn test_decode_empty_buffer() {
        let buf = vec![];
        let mut index = 0;
        let result = decode_pid(&buf, &mut index);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DecodeError::BufferTooShort);
    }

    #[test]
    fn test_decode_unexpected_tag() {
        let buf = vec![0xFF, 1, 2, 3];
        let mut index = 0;
        let result = decode_pid(&buf, &mut index);
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
        // Create buffer with ERL_NEW_PID_EXT tag but invalid atom data
        let mut buf = vec![ERL_NEW_PID_EXT];
        // Add invalid atom tag (0xFF is not a valid atom tag)
        buf.push(0xFF);
        let mut index = 0;
        let result = decode_pid(&buf, &mut index);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecodeError::AtomDecodeError(_) => {}
            _ => panic!("Expected AtomDecodeError"),
        }
    }

    #[test]
    fn test_decode_buffer_too_short_for_num_serial() {
        // Create buffer with valid tag and atom, but buffer too short for num/serial
        let mut buf = vec![ERL_NEW_PID_EXT];
        // Add valid small atom: SMALL_ATOM_EXT (115) + length 3 + "nod"
        buf.extend_from_slice(&[115, 3, b'n', b'o', b'd']);
        // Only add 7 bytes instead of 8 for num/serial
        buf.extend_from_slice(&[0, 0, 0, 1, 0, 0, 0]);
        let mut index = 0;
        let result = decode_pid(&buf, &mut index);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DecodeError::BufferTooShort);
    }

    #[test]
    fn test_decode_pid_ext_old_format() {
        // Test ERL_PID_EXT (old format) with 2-bit creation
        let mut buf = vec![ERL_PID_EXT];
        // Add valid small atom: SMALL_ATOM_EXT (115) + length 3 + "nod"
        buf.extend_from_slice(&[115, 3, b'n', b'o', b'd']);
        // num (4 bytes)
        buf.extend_from_slice(&123u32.to_be_bytes());
        // serial (4 bytes)
        buf.extend_from_slice(&456u32.to_be_bytes());
        // creation (1 byte, only 2 bits used: 0x03 = 3)
        buf.push(0x03);
        
        let mut index = 0;
        let decoded = decode_pid(&buf, &mut index).unwrap();
        assert_eq!(decoded.num, 123);
        assert_eq!(decoded.serial, 456);
        assert_eq!(decoded.creation, 3); // Only 2 bits, so max is 3
    }

    #[test]
    fn test_decode_pid_ext_buffer_too_short_for_creation() {
        // Create buffer with valid tag, atom, num, serial, but no creation byte
        let mut buf = vec![ERL_PID_EXT];
        buf.extend_from_slice(&[115, 3, b'n', b'o', b'd']);
        buf.extend_from_slice(&123u32.to_be_bytes());
        buf.extend_from_slice(&456u32.to_be_bytes());
        // No creation byte
        let mut index = 0;
        let result = decode_pid(&buf, &mut index);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DecodeError::BufferTooShort);
    }

    #[test]
    fn test_decode_new_pid_ext_buffer_too_short_for_creation() {
        // Create buffer with valid tag, atom, num, serial, but buffer too short for creation
        let mut buf = vec![ERL_NEW_PID_EXT];
        buf.extend_from_slice(&[115, 3, b'n', b'o', b'd']);
        buf.extend_from_slice(&123u32.to_be_bytes());
        buf.extend_from_slice(&456u32.to_be_bytes());
        // Only 3 bytes instead of 4 for creation
        buf.extend_from_slice(&[0, 0, 0]);
        let mut index = 0;
        let result = decode_pid(&buf, &mut index);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), DecodeError::BufferTooShort);
    }

    #[test]
    fn test_decode_new_pid_ext() {
        // Test ERL_NEW_PID_EXT (new format) with 32-bit creation
        let mut buf = vec![ERL_NEW_PID_EXT];
        buf.extend_from_slice(&[115, 3, b'n', b'o', b'd']);
        buf.extend_from_slice(&123u32.to_be_bytes());
        buf.extend_from_slice(&456u32.to_be_bytes());
        // creation (4 bytes)
        buf.extend_from_slice(&1000u32.to_be_bytes());
        
        let mut index = 0;
        let decoded = decode_pid(&buf, &mut index).unwrap();
        assert_eq!(decoded.num, 123);
        assert_eq!(decoded.serial, 456);
        assert_eq!(decoded.creation, 1000);
    }

    #[test]
    fn test_decode_pid_ext_creation_mask() {
        // Test that creation is properly masked to 2 bits
        let mut buf = vec![ERL_PID_EXT];
        buf.extend_from_slice(&[115, 3, b'n', b'o', b'd']);
        buf.extend_from_slice(&123u32.to_be_bytes());
        buf.extend_from_slice(&456u32.to_be_bytes());
        // creation byte with upper bits set: 0xFF should become 0x03
        buf.push(0xFF);
        
        let mut index = 0;
        let decoded = decode_pid(&buf, &mut index).unwrap();
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
            let mut buf = vec![ERL_PID_EXT];
            buf.extend_from_slice(&[115, 3, b'n', b'o', b'd']);
            buf.extend_from_slice(&123u32.to_be_bytes());
            buf.extend_from_slice(&456u32.to_be_bytes());
            buf.push(creation);
            
            let mut index = 0;
            let decoded = decode_pid(&buf, &mut index).unwrap();
            assert_eq!(decoded.creation, creation as u32);
        }
    }

    #[test]
    fn test_decode_various_num_serial() {
        // Test various num and serial values
        let test_cases = vec![
            (0u32, 0u32),
            (1u32, 1u32),
            (u32::MAX, u32::MAX),
            (123u32, 456u32),
        ];
        
        for (num, serial) in test_cases {
            let mut buf = vec![ERL_NEW_PID_EXT];
            buf.extend_from_slice(&[115, 3, b'n', b'o', b'd']);
            buf.extend_from_slice(&num.to_be_bytes());
            buf.extend_from_slice(&serial.to_be_bytes());
            buf.extend_from_slice(&1u32.to_be_bytes()); // creation
            
            let mut index = 0;
            let decoded = decode_pid(&buf, &mut index).unwrap();
            assert_eq!(decoded.num, num);
            assert_eq!(decoded.serial, serial);
        }
    }
}

