//! Port Encoding Module
//!
//! Provides functionality to encode Ports to EI (Erlang Interface) format.
//! Ports represent external resources (files, sockets, etc.) that can communicate
//! with Erlang processes.
//!
//! ## Overview
//!
//! Ports in EI format consist of:
//! - **Node name**: Atom identifying the node where the port exists
//! - **Port ID**: Unique identifier for the port (32-bit or 64-bit)
//! - **Creation number**: Node creation number (32 bits in NEW/V4 formats)
//!
//! ## Encoding Format
//!
//! The encoder automatically selects the format based on port ID size:
//! - **V4_PORT_EXT**: For port IDs > 0x0FFFFFFF (64-bit ID)
//! - **NEW_PORT_EXT**: For port IDs ≤ 0x0FFFFFFF (32-bit ID)
//!
//! ## Examples
//!
//! ```rust
//! use infrastructure_code_loading::encode_port::{encode_port, ErlangPort};
//!
//! let port = ErlangPort {
//!     node: "node@host".to_string(),
//!     id: 12345,
//!     creation: 789,
//! };
//!
//! let mut buf = vec![0u8; 100];
//! let mut index = 0;
//! encode_port(&mut Some(&mut buf), &mut index, &port)?;
//! ```
//!
//! ## See Also
//!
//! - [`decode_port`](super::decode_port/index.html): Port decoding functions
//! - [`encode_pid`](super::encode_pid/index.html): PID encoding (similar structure)
//!
//! Based on `lib/erl_interface/src/encode/encode_port.c`

use crate::constants::{ERL_V4_PORT_EXT, ERL_NEW_PORT_EXT};
use infrastructure_data_handling::encode_atom::encode_atom;
use entities_data_handling::atom::AtomEncoding;

/// Port structure
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErlangPort {
    /// Node name
    pub node: String,
    /// Port ID (64-bit for V4, 32-bit for NEW)
    pub id: u64,
    /// Creation number (32 bits for NEW/V4, 2 bits for old PORT_EXT)
    pub creation: u32,
}

/// Encode a port to EI format
///
/// # Arguments
/// * `buf` - Optional buffer to write to (None for size calculation)
/// * `index` - Current index in buffer
/// * `port` - The port to encode
///
/// # Returns
/// * `Ok(())` - Success
/// * `Err(EncodeError)` - Encoding error
pub fn encode_port(buf: &mut Option<&mut [u8]>, index: &mut usize, port: &ErlangPort) -> Result<(), EncodeError> {
    // Reserve space for tag
    let tag_pos = *index;
    *index += 1;

    // Encode node atom
    let mut atom_buf = Vec::new();
    let atom_bytes = encode_atom(&mut atom_buf, &port.node, AtomEncoding::Utf8)
        .map_err(|e| EncodeError::AtomEncodeError(format!("{:?}", e)))?;
    
    if let Some(b) = buf.as_mut() {
        if *index + atom_bytes > b.len() {
            return Err(EncodeError::BufferTooSmall);
        }
        b[*index..*index + atom_bytes].copy_from_slice(&atom_buf);
    }
    *index += atom_bytes;

    // Choose format based on ID size
    if port.id > 0x0FFFFFFF {
        // V4_PORT_EXT (64-bit id)
        if let Some(b) = buf.as_mut() {
            if *index + 12 > b.len() {
                return Err(EncodeError::BufferTooSmall);
            }
            b[tag_pos] = ERL_V4_PORT_EXT;
            b[*index..*index + 8].copy_from_slice(&port.id.to_be_bytes());
            b[*index + 8..*index + 12].copy_from_slice(&port.creation.to_be_bytes());
        }
        *index += 12;
    } else {
        // NEW_PORT_EXT (32-bit id)
        if let Some(b) = buf.as_mut() {
            if *index + 8 > b.len() {
                return Err(EncodeError::BufferTooSmall);
            }
            b[tag_pos] = ERL_NEW_PORT_EXT;
            b[*index..*index + 4].copy_from_slice(&(port.id as u32).to_be_bytes());
            b[*index + 4..*index + 8].copy_from_slice(&port.creation.to_be_bytes());
        }
        *index += 8;
    }

    Ok(())
}

/// Encoding errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodeError {
    /// Buffer is too small for the encoded value
    BufferTooSmall,
    /// Atom encoding error
    AtomEncodeError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_port_new() {
        let port = ErlangPort {
            node: "node@host".to_string(),
            id: 123,
            creation: 1,
        };
        let mut buf = vec![0u8; 100];
        let mut index = 0;
        encode_port(&mut Some(&mut buf), &mut index, &port).unwrap();
        assert_eq!(buf[0], ERL_NEW_PORT_EXT);
    }

    #[test]
    fn test_encode_port_v4() {
        let port = ErlangPort {
            node: "node@host".to_string(),
            id: 0x10000000, // > 28 bits
            creation: 1,
        };
        let mut buf = vec![0u8; 100];
        let mut index = 0;
        encode_port(&mut Some(&mut buf), &mut index, &port).unwrap();
        assert_eq!(buf[0], ERL_V4_PORT_EXT);
    }

    #[test]
    fn test_encode_port_size_calculation() {
        let port = ErlangPort {
            node: "node".to_string(),
            id: 123,
            creation: 1,
        };
        let mut index = 0;
        let mut buf_opt = None;
        encode_port(&mut buf_opt, &mut index, &port).unwrap();
        assert!(index > 0);
    }

    #[test]
    fn test_encode_port_buffer_too_small_for_atom() {
        let port = ErlangPort {
            node: "node@host".to_string(),
            id: 123,
            creation: 1,
        };
        // Calculate size needed
        let mut size_index = 0;
        encode_port(&mut None, &mut size_index, &port).unwrap();
        // Use a buffer that's too small (only room for tag)
        let mut buf = vec![0u8; 1];
        let mut index = 0;
        let mut buf_opt = Some(&mut buf[..]);
        let result = encode_port(&mut buf_opt, &mut index, &port);
        assert!(result.is_err());
        match result.unwrap_err() {
            EncodeError::BufferTooSmall => {}
            _ => panic!("Expected BufferTooSmall"),
        }
    }

    #[test]
    fn test_encode_port_new_buffer_too_small_for_id_creation() {
        let port = ErlangPort {
            node: "node".to_string(), // Short node name
            id: 123,
            creation: 1,
        };
        // Calculate size needed for tag + atom
        let mut atom_buf = Vec::new();
        let atom_bytes = encode_atom(&mut atom_buf, "node", AtomEncoding::Utf8).unwrap();
        // Use a buffer that's too small (only room for tag + atom, not id + creation)
        let mut buf = vec![0u8; 1 + atom_bytes];
        let mut index = 0;
        let mut buf_opt = Some(&mut buf[..]);
        let result = encode_port(&mut buf_opt, &mut index, &port);
        assert!(result.is_err());
        match result.unwrap_err() {
            EncodeError::BufferTooSmall => {}
            _ => panic!("Expected BufferTooSmall"),
        }
    }

    #[test]
    fn test_encode_port_v4_buffer_too_small_for_id_creation() {
        let port = ErlangPort {
            node: "node".to_string(), // Short node name
            id: 0x10000000, // > 28 bits, uses V4 format
            creation: 1,
        };
        // Calculate size needed for tag + atom
        let mut atom_buf = Vec::new();
        let atom_bytes = encode_atom(&mut atom_buf, "node", AtomEncoding::Utf8).unwrap();
        // Use a buffer that's too small (only room for tag + atom, not id + creation)
        let mut buf = vec![0u8; 1 + atom_bytes];
        let mut index = 0;
        let mut buf_opt = Some(&mut buf[..]);
        let result = encode_port(&mut buf_opt, &mut index, &port);
        assert!(result.is_err());
        match result.unwrap_err() {
            EncodeError::BufferTooSmall => {}
            _ => panic!("Expected BufferTooSmall"),
        }
    }

    #[test]
    fn test_encode_port_boundary_id() {
        // Test boundary value: exactly 0x0FFFFFFF (should use NEW_PORT_EXT)
        let port1 = ErlangPort {
            node: "node@host".to_string(),
            id: 0x0FFFFFFF,
            creation: 1,
        };
        let mut buf1 = vec![0u8; 100];
        let mut index1 = 0;
        encode_port(&mut Some(&mut buf1), &mut index1, &port1).unwrap();
        assert_eq!(buf1[0], ERL_NEW_PORT_EXT);

        // Test boundary value: 0x0FFFFFFF + 1 (should use V4_PORT_EXT)
        let port2 = ErlangPort {
            node: "node@host".to_string(),
            id: 0x0FFFFFFF + 1,
            creation: 1,
        };
        let mut buf2 = vec![0u8; 100];
        let mut index2 = 0;
        encode_port(&mut Some(&mut buf2), &mut index2, &port2).unwrap();
        assert_eq!(buf2[0], ERL_V4_PORT_EXT);
    }

    #[test]
    fn test_encode_port_various_values() {
        let test_cases = vec![
            (0u64, 0u32, ERL_NEW_PORT_EXT),
            (1u64, 1u32, ERL_NEW_PORT_EXT),
            (100u64, 200u32, ERL_NEW_PORT_EXT),
            (0x0FFFFFFFu64, u32::MAX, ERL_NEW_PORT_EXT),
            (0x10000000u64, 1u32, ERL_V4_PORT_EXT),
            (u64::MAX, u32::MAX, ERL_V4_PORT_EXT),
        ];
        
        for (id, creation, expected_tag) in test_cases {
            let port = ErlangPort {
                node: "node@host".to_string(),
                id,
                creation,
            };
            let mut buf = vec![0u8; 100];
            let mut index = 0;
            let mut buf_opt = Some(&mut buf[..]);
            encode_port(&mut buf_opt, &mut index, &port).unwrap();
            assert_eq!(buf[0], expected_tag);
        }
    }

    #[test]
    fn test_encode_error_debug() {
        let error1 = EncodeError::BufferTooSmall;
        let error2 = EncodeError::AtomEncodeError("atom_err".to_string());
        
        let debug_str1 = format!("{:?}", error1);
        let debug_str2 = format!("{:?}", error2);
        
        assert!(debug_str1.contains("BufferTooSmall"));
        assert!(debug_str2.contains("AtomEncodeError"));
    }

    #[test]
    fn test_encode_error_clone() {
        let error1 = EncodeError::BufferTooSmall;
        let error2 = EncodeError::AtomEncodeError("atom_err".to_string());
        
        let cloned1 = error1.clone();
        let cloned2 = error2.clone();
        
        assert_eq!(error1, cloned1);
        assert_eq!(error2, cloned2);
    }

    #[test]
    fn test_encode_error_partial_eq() {
        let error1 = EncodeError::BufferTooSmall;
        let error2 = EncodeError::BufferTooSmall;
        let error3 = EncodeError::AtomEncodeError("err".to_string());
        let error4 = EncodeError::AtomEncodeError("err".to_string());
        let error5 = EncodeError::AtomEncodeError("different".to_string());
        
        assert_eq!(error1, error2);
        assert_eq!(error3, error4);
        assert_ne!(error3, error5);
        assert_ne!(error1, error3);
    }

    #[test]
    fn test_encode_error_eq() {
        let error1 = EncodeError::BufferTooSmall;
        let error2 = EncodeError::BufferTooSmall;
        let error3 = EncodeError::AtomEncodeError("err".to_string());
        
        assert!(error1 == error2);
        assert!(error1 != error3);
    }

    #[test]
    fn test_erlang_port_debug() {
        let port = ErlangPort {
            node: "node@host".to_string(),
            id: 123,
            creation: 1,
        };
        
        let debug_str = format!("{:?}", port);
        assert!(debug_str.contains("ErlangPort"));
    }

    #[test]
    fn test_erlang_port_clone() {
        let port = ErlangPort {
            node: "node@host".to_string(),
            id: 123,
            creation: 1,
        };
        
        let cloned = port.clone();
        assert_eq!(port, cloned);
    }

    #[test]
    fn test_erlang_port_partial_eq() {
        let port1 = ErlangPort {
            node: "node@host".to_string(),
            id: 123,
            creation: 1,
        };
        let port2 = ErlangPort {
            node: "node@host".to_string(),
            id: 123,
            creation: 1,
        };
        let port3 = ErlangPort {
            node: "node@host".to_string(),
            id: 124,
            creation: 1,
        };
        let port4 = ErlangPort {
            node: "different@host".to_string(),
            id: 123,
            creation: 1,
        };
        let port5 = ErlangPort {
            node: "node@host".to_string(),
            id: 123,
            creation: 2,
        };
        
        assert_eq!(port1, port2);
        assert_ne!(port1, port3);
        assert_ne!(port1, port4);
        assert_ne!(port1, port5);
    }

    #[test]
    fn test_erlang_port_eq() {
        let port1 = ErlangPort {
            node: "node@host".to_string(),
            id: 123,
            creation: 1,
        };
        let port2 = ErlangPort {
            node: "node@host".to_string(),
            id: 123,
            creation: 1,
        };
        let port3 = ErlangPort {
            node: "node@host".to_string(),
            id: 124,
            creation: 1,
        };
        
        assert!(port1 == port2);
        assert!(port1 != port3);
    }
}

