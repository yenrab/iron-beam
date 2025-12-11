//! Reference Encoding Module
//!
//! Provides functionality to encode References to EI (Erlang Interface) format.
//! References are unique identifiers used for message passing and process communication
//! in distributed Erlang systems.
//!
//! ## Overview
//!
//! References in EI format consist of:
//! - **Node name**: Atom identifying the node where the reference was created
//! - **Length**: Number of ID integers (up to 5 for old format, variable for new format)
//! - **Creation number**: Node creation number (2 bits in old format, 32 bits in new format)
//! - **ID integers**: Array of 32-bit integers that uniquely identify the reference
//!
//! ## Encoding Format
//!
//! This module uses the `ERL_NEWER_REFERENCE_EXT` format, which supports:
//! - Variable-length ID arrays
//! - 32-bit creation numbers
//! - UTF-8 node names
//!
//! ## Examples
//!
//! ```rust
//! use infrastructure_code_loading::encode_ref::{encode_ref, ErlangRef};
//!
//! let r#ref = ErlangRef {
//!     node: "node@host".to_string(),
//!     len: 3,
//!     creation: 789,
//!     ids: vec![100, 200, 300],
//! };
//!
//! let mut buf = vec![0u8; 100];
//! let mut index = 0;
//! encode_ref(&mut Some(&mut buf), &mut index, &r#ref).unwrap();
//! ```
//!
//! ## See Also
//!
//! - [`decode_ref`](super::decode_ref/index.html): Reference decoding functions
//! - [`encode_pid`](super::encode_pid/index.html): PID encoding (similar structure)
//!
//! Based on `lib/erl_interface/src/encode/encode_ref.c`

use crate::constants::ERL_NEWER_REFERENCE_EXT;
use infrastructure_data_handling::encode_atom::encode_atom;
use entities_data_handling::atom::AtomEncoding;

/// Reference structure
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErlangRef {
    /// Node name
    pub node: String,
    /// Number of ID integers
    pub len: u16,
    /// Creation number (32 bits for NEWER_REFERENCE_EXT)
    pub creation: u32,
    /// ID integers (up to 5 for old format, variable for new format)
    pub ids: Vec<u32>,
}

/// Encode a reference to EI format
///
/// # Arguments
/// * `buf` - Optional buffer to write to (None for size calculation)
/// * `index` - Current index in buffer
/// * `r#ref` - The reference to encode
///
/// # Returns
/// * `Ok(())` - Success
/// * `Err(EncodeError)` - Encoding error
pub fn encode_ref(buf: &mut Option<&mut [u8]>, index: &mut usize, r#ref: &ErlangRef) -> Result<(), EncodeError> {
    if r#ref.ids.len() != r#ref.len as usize {
        return Err(EncodeError::InvalidLength);
    }

    // Reserve space for tag and length
    let tag_pos = *index;
    *index += 1 + 2; // tag + 2 bytes for length

    // Encode node atom
    let mut atom_buf = Vec::new();
    let atom_bytes = encode_atom(&mut atom_buf, &r#ref.node, AtomEncoding::Utf8)
        .map_err(|e| EncodeError::AtomEncodeError(format!("{:?}", e)))?;
    
    if let Some(b) = buf.as_mut() {
        if *index + atom_bytes > b.len() {
            return Err(EncodeError::BufferTooSmall);
        }
        b[*index..*index + atom_bytes].copy_from_slice(&atom_buf);
    }
    *index += atom_bytes;

    // Encode creation and IDs
    let data_size = 4 + (r#ref.len as usize * 4); // creation (4) + ids (4 each)
    if let Some(b) = buf.as_mut() {
        if *index + data_size > b.len() {
            return Err(EncodeError::BufferTooSmall);
        }
        b[tag_pos] = ERL_NEWER_REFERENCE_EXT;
        b[tag_pos + 1..tag_pos + 3].copy_from_slice(&r#ref.len.to_be_bytes());
        b[*index..*index + 4].copy_from_slice(&r#ref.creation.to_be_bytes());
        for (i, &id) in r#ref.ids.iter().enumerate() {
            b[*index + 4 + i * 4..*index + 4 + (i + 1) * 4].copy_from_slice(&id.to_be_bytes());
        }
    }
    *index += data_size;

    Ok(())
}

/// Encoding errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodeError {
    /// Buffer is too small for the encoded value
    BufferTooSmall,
    /// Atom encoding error
    AtomEncodeError(String),
    /// Invalid length (ids.len() != len)
    InvalidLength,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_ref() {
        let r#ref = ErlangRef {
            node: "node@host".to_string(),
            len: 1,
            creation: 1,
            ids: vec![123],
        };
        let mut buf = vec![0u8; 100];
        let mut index = 0;
        encode_ref(&mut Some(&mut buf), &mut index, &r#ref).unwrap();
        assert_eq!(buf[0], ERL_NEWER_REFERENCE_EXT);
    }

    #[test]
    fn test_encode_ref_multiple_ids() {
        let r#ref = ErlangRef {
            node: "node@host".to_string(),
            len: 3,
            creation: 1,
            ids: vec![123, 456, 789],
        };
        let mut buf = vec![0u8; 100];
        let mut index = 0;
        encode_ref(&mut Some(&mut buf), &mut index, &r#ref).unwrap();
        assert_eq!(buf[0], ERL_NEWER_REFERENCE_EXT);
    }

    #[test]
    fn test_encode_ref_size_calculation() {
        let r#ref = ErlangRef {
            node: "node".to_string(),
            len: 1,
            creation: 1,
            ids: vec![123],
        };
        let mut index = 0;
        let mut buf_opt = None;
        encode_ref(&mut buf_opt, &mut index, &r#ref).unwrap();
        assert!(index > 0);
    }

    #[test]
    fn test_encode_ref_invalid_length() {
        let r#ref = ErlangRef {
            node: "node@host".to_string(),
            len: 2,
            creation: 1,
            ids: vec![123], // len is 2 but ids.len() is 1
        };
        let mut buf = vec![0u8; 100];
        let mut index = 0;
        let result = encode_ref(&mut Some(&mut buf), &mut index, &r#ref);
        assert!(result.is_err());
        match result.unwrap_err() {
            EncodeError::InvalidLength => {}
            _ => panic!("Expected InvalidLength"),
        }
    }

    #[test]
    fn test_encode_ref_invalid_length_too_many_ids() {
        let r#ref = ErlangRef {
            node: "node@host".to_string(),
            len: 1,
            creation: 1,
            ids: vec![123, 456], // len is 1 but ids.len() is 2
        };
        let mut buf = vec![0u8; 100];
        let mut index = 0;
        let result = encode_ref(&mut Some(&mut buf), &mut index, &r#ref);
        assert!(result.is_err());
        match result.unwrap_err() {
            EncodeError::InvalidLength => {}
            _ => panic!("Expected InvalidLength"),
        }
    }

    #[test]
    fn test_encode_ref_buffer_too_small_for_atom() {
        let r#ref = ErlangRef {
            node: "node@host".to_string(),
            len: 1,
            creation: 1,
            ids: vec![123],
        };
        // Calculate size needed
        let mut size_index = 0;
        encode_ref(&mut None, &mut size_index, &r#ref).unwrap();
        // Use a buffer that's too small (only room for tag + length)
        let mut buf = vec![0u8; 3];
        let mut index = 0;
        let mut buf_opt = Some(&mut buf[..]);
        let result = encode_ref(&mut buf_opt, &mut index, &r#ref);
        assert!(result.is_err());
        match result.unwrap_err() {
            EncodeError::BufferTooSmall => {}
            _ => panic!("Expected BufferTooSmall"),
        }
    }

    #[test]
    fn test_encode_ref_buffer_too_small_for_creation_ids() {
        let r#ref = ErlangRef {
            node: "node".to_string(), // Short node name
            len: 1,
            creation: 1,
            ids: vec![123],
        };
        // Calculate size needed for tag + length + atom
        let mut atom_buf = Vec::new();
        let atom_bytes = encode_atom(&mut atom_buf, "node", AtomEncoding::Utf8).unwrap();
        // Use a buffer that's too small (only room for tag + length + atom, not creation + ids)
        let mut buf = vec![0u8; 3 + atom_bytes];
        let mut index = 0;
        let mut buf_opt = Some(&mut buf[..]);
        let result = encode_ref(&mut buf_opt, &mut index, &r#ref);
        assert!(result.is_err());
        match result.unwrap_err() {
            EncodeError::BufferTooSmall => {}
            _ => panic!("Expected BufferTooSmall"),
        }
    }

    #[test]
    fn test_encode_ref_various_values() {
        let test_cases = vec![
            (1u16, 0u32, vec![0u32]),
            (1u16, 1u32, vec![1u32]),
            (1u16, 100u32, vec![100u32]),
            (1u16, u32::MAX, vec![u32::MAX]),
            (2u16, 1u32, vec![1u32, 2u32]),
            (3u16, 1u32, vec![1u32, 2u32, 3u32]),
            (5u16, 1u32, vec![1u32, 2u32, 3u32, 4u32, 5u32]),
        ];
        
        for (len, creation, ids) in test_cases {
            let r#ref = ErlangRef {
                node: "node@host".to_string(),
                len,
                creation,
                ids: ids.clone(),
            };
            let mut buf = vec![0u8; 200];
            let mut index = 0;
            let mut buf_opt = Some(&mut buf[..]);
            encode_ref(&mut buf_opt, &mut index, &r#ref).unwrap();
            assert_eq!(buf[0], ERL_NEWER_REFERENCE_EXT);
        }
    }

    #[test]
    fn test_encode_error_debug() {
        let error1 = EncodeError::BufferTooSmall;
        let error2 = EncodeError::AtomEncodeError("atom_err".to_string());
        let error3 = EncodeError::InvalidLength;
        
        let debug_str1 = format!("{:?}", error1);
        let debug_str2 = format!("{:?}", error2);
        let debug_str3 = format!("{:?}", error3);
        
        assert!(debug_str1.contains("BufferTooSmall"));
        assert!(debug_str2.contains("AtomEncodeError"));
        assert!(debug_str3.contains("InvalidLength"));
    }

    #[test]
    fn test_encode_error_clone() {
        let error1 = EncodeError::BufferTooSmall;
        let error2 = EncodeError::AtomEncodeError("atom_err".to_string());
        let error3 = EncodeError::InvalidLength;
        
        let cloned1 = error1.clone();
        let cloned2 = error2.clone();
        let cloned3 = error3.clone();
        
        assert_eq!(error1, cloned1);
        assert_eq!(error2, cloned2);
        assert_eq!(error3, cloned3);
    }

    #[test]
    fn test_encode_error_partial_eq() {
        let error1 = EncodeError::BufferTooSmall;
        let error2 = EncodeError::BufferTooSmall;
        let error3 = EncodeError::AtomEncodeError("err".to_string());
        let error4 = EncodeError::AtomEncodeError("err".to_string());
        let error5 = EncodeError::AtomEncodeError("different".to_string());
        let error6 = EncodeError::InvalidLength;
        
        assert_eq!(error1, error2);
        assert_eq!(error3, error4);
        assert_ne!(error3, error5);
        assert_ne!(error1, error3);
        assert_ne!(error1, error6);
        assert_ne!(error6, error3);
    }

    #[test]
    fn test_encode_error_eq() {
        let error1 = EncodeError::BufferTooSmall;
        let error2 = EncodeError::BufferTooSmall;
        let error3 = EncodeError::InvalidLength;
        
        assert!(error1 == error2);
        assert!(error1 != error3);
    }

    #[test]
    fn test_erlang_ref_debug() {
        let r#ref = ErlangRef {
            node: "node@host".to_string(),
            len: 1,
            creation: 1,
            ids: vec![123],
        };
        
        let debug_str = format!("{:?}", r#ref);
        assert!(debug_str.contains("ErlangRef"));
    }

    #[test]
    fn test_erlang_ref_clone() {
        let r#ref = ErlangRef {
            node: "node@host".to_string(),
            len: 1,
            creation: 1,
            ids: vec![123],
        };
        
        let cloned = r#ref.clone();
        assert_eq!(r#ref, cloned);
    }

    #[test]
    fn test_erlang_ref_partial_eq() {
        let ref1 = ErlangRef {
            node: "node@host".to_string(),
            len: 1,
            creation: 1,
            ids: vec![123],
        };
        let ref2 = ErlangRef {
            node: "node@host".to_string(),
            len: 1,
            creation: 1,
            ids: vec![123],
        };
        let ref3 = ErlangRef {
            node: "node@host".to_string(),
            len: 2,
            creation: 1,
            ids: vec![123, 456],
        };
        let ref4 = ErlangRef {
            node: "different@host".to_string(),
            len: 1,
            creation: 1,
            ids: vec![123],
        };
        let ref5 = ErlangRef {
            node: "node@host".to_string(),
            len: 1,
            creation: 2,
            ids: vec![123],
        };
        let ref6 = ErlangRef {
            node: "node@host".to_string(),
            len: 1,
            creation: 1,
            ids: vec![124],
        };
        
        assert_eq!(ref1, ref2);
        assert_ne!(ref1, ref3);
        assert_ne!(ref1, ref4);
        assert_ne!(ref1, ref5);
        assert_ne!(ref1, ref6);
    }

    #[test]
    fn test_erlang_ref_eq() {
        let ref1 = ErlangRef {
            node: "node@host".to_string(),
            len: 1,
            creation: 1,
            ids: vec![123],
        };
        let ref2 = ErlangRef {
            node: "node@host".to_string(),
            len: 1,
            creation: 1,
            ids: vec![123],
        };
        let ref3 = ErlangRef {
            node: "node@host".to_string(),
            len: 1,
            creation: 1,
            ids: vec![124],
        };
        
        assert!(ref1 == ref2);
        assert!(ref1 != ref3);
    }
}

