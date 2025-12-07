//! PID Encoding Module
//!
//! Provides functionality to encode Process IDs (PIDs) to EI (Erlang Interface) format.
//! PIDs uniquely identify processes in the Erlang runtime system and are essential
//! for inter-process communication and distributed Erlang.
//!
//! ## Overview
//!
//! PIDs in EI format consist of:
//! - **Node name**: Atom identifying the node where the process exists
//! - **Process number**: Unique identifier for the process on the node
//! - **Serial number**: Serial number for the process
//! - **Creation number**: Node creation number (32 bits in NEW_PID_EXT format)
//!
//! ## Encoding Format
//!
//! This module uses the `ERL_NEW_PID_EXT` format, which supports 32-bit creation
//! numbers. The old `ERL_PID_EXT` format (2-bit creation) is not supported.
//!
//! ## Examples
//!
//! ```rust
//! use infrastructure_code_loading::encode_pid::{encode_pid, ErlangPid};
//!
//! let pid = ErlangPid {
//!     node: "node@host".to_string(),
//!     num: 123,
//!     serial: 456,
//!     creation: 789,
//! };
//!
//! let mut buf = vec![0u8; 100];
//! let mut index = 0;
//! encode_pid(&mut Some(&mut buf), &mut index, &pid)?;
//! ```
//!
//! ## See Also
//!
//! - [`decode_pid`](super::decode_pid/index.html): PID decoding functions
//! - [`encode_port`](super::encode_port/index.html): Port encoding (similar structure)
//! - [`encode_fun`](super::encode_fun/index.html): Function encoding (uses PIDs)
//!
//! Based on `lib/erl_interface/src/encode/encode_pid.c`

use crate::constants::ERL_NEW_PID_EXT;
use infrastructure_data_handling::encode_atom::encode_atom;
use entities_data_handling::atom::AtomEncoding;

/// PID structure
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErlangPid {
    /// Node name
    pub node: String,
    /// Process number
    pub num: u32,
    /// Serial number
    pub serial: u32,
    /// Creation number (32 bits for NEW_PID_EXT, 2 bits for old PID_EXT)
    pub creation: u32,
}

/// Encode a PID to EI format
///
/// # Arguments
/// * `buf` - Optional buffer to write to (None for size calculation)
/// * `index` - Current index in buffer
/// * `pid` - The PID to encode
///
/// # Returns
/// * `Ok(())` - Success
/// * `Err(EncodeError)` - Encoding error
pub fn encode_pid(buf: &mut Option<&mut [u8]>, index: &mut usize, pid: &ErlangPid) -> Result<(), EncodeError> {
    // Always use NEW_PID_EXT format (32-bit creation)
    if let Some(b) = buf.as_mut() {
        if *index >= b.len() {
            return Err(EncodeError::BufferTooSmall);
        }
        b[*index] = ERL_NEW_PID_EXT;
    }
    *index += 1;

    // Encode node atom
    let mut atom_buf = Vec::new();
    let atom_bytes = encode_atom(&mut atom_buf, &pid.node, AtomEncoding::Utf8)
        .map_err(|e| EncodeError::AtomEncodeError(format!("{:?}", e)))?;
    
    if let Some(b) = buf.as_mut() {
        if *index + atom_bytes > b.len() {
            return Err(EncodeError::BufferTooSmall);
        }
        b[*index..*index + atom_bytes].copy_from_slice(&atom_buf);
    }
    *index += atom_bytes;

    // Encode num, serial, creation (all 32-bit big-endian)
    if let Some(b) = buf.as_mut() {
        if *index + 12 > b.len() {
            return Err(EncodeError::BufferTooSmall);
        }
        b[*index..*index + 4].copy_from_slice(&pid.num.to_be_bytes());
        b[*index + 4..*index + 8].copy_from_slice(&pid.serial.to_be_bytes());
        b[*index + 8..*index + 12].copy_from_slice(&pid.creation.to_be_bytes());
    }
    *index += 12;

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
    fn test_encode_pid() {
        let pid = ErlangPid {
            node: "node@host".to_string(),
            num: 123,
            serial: 456,
            creation: 1,
        };
        let mut buf = vec![0u8; 100];
        let mut index = 0;
        let mut buf_opt = Some(&mut buf[..]);
        encode_pid(&mut buf_opt, &mut index, &pid).unwrap();
        assert_eq!(buf[0], ERL_NEW_PID_EXT);
        assert!(index > 0);
    }

    #[test]
    fn test_encode_size_calculation() {
        let pid = ErlangPid {
            node: "node".to_string(),
            num: 123,
            serial: 456,
            creation: 1,
        };
        let mut index = 0;
        let mut buf_opt = None;
        encode_pid(&mut buf_opt, &mut index, &pid).unwrap();
        assert!(index > 0);
    }

    #[test]
    fn test_encode_pid_buffer_too_small_at_start() {
        let pid = ErlangPid {
            node: "node@host".to_string(),
            num: 123,
            serial: 456,
            creation: 1,
        };
        let mut buf = vec![0u8; 0]; // Empty buffer
        let mut index = 0;
        let mut buf_opt = Some(&mut buf[..]);
        let result = encode_pid(&mut buf_opt, &mut index, &pid);
        assert!(result.is_err());
        match result.unwrap_err() {
            EncodeError::BufferTooSmall => {}
            _ => panic!("Expected BufferTooSmall"),
        }
    }

    #[test]
    fn test_encode_pid_buffer_too_small_for_atom() {
        let pid = ErlangPid {
            node: "node@host".to_string(),
            num: 123,
            serial: 456,
            creation: 1,
        };
        // Calculate size needed
        let mut size_index = 0;
        encode_pid(&mut None, &mut size_index, &pid).unwrap();
        // Use a buffer that's too small (only room for tag)
        let mut buf = vec![0u8; 1];
        let mut index = 0;
        let mut buf_opt = Some(&mut buf[..]);
        let result = encode_pid(&mut buf_opt, &mut index, &pid);
        assert!(result.is_err());
        match result.unwrap_err() {
            EncodeError::BufferTooSmall => {}
            _ => panic!("Expected BufferTooSmall"),
        }
    }

    #[test]
    fn test_encode_pid_buffer_too_small_for_num_serial_creation() {
        let pid = ErlangPid {
            node: "node".to_string(), // Short node name
            num: 123,
            serial: 456,
            creation: 1,
        };
        // Calculate size needed for tag + atom
        let mut atom_buf = Vec::new();
        let atom_bytes = encode_atom(&mut atom_buf, "node", AtomEncoding::Utf8).unwrap();
        // Use a buffer that's too small (only room for tag + atom, not num/serial/creation)
        let mut buf = vec![0u8; 1 + atom_bytes];
        let mut index = 0;
        let mut buf_opt = Some(&mut buf[..]);
        let result = encode_pid(&mut buf_opt, &mut index, &pid);
        assert!(result.is_err());
        match result.unwrap_err() {
            EncodeError::BufferTooSmall => {}
            _ => panic!("Expected BufferTooSmall"),
        }
    }

    #[test]
    fn test_encode_pid_various_values() {
        let test_cases = vec![
            (0u32, 0u32, 0u32),
            (1u32, 2u32, 3u32),
            (100u32, 200u32, 300u32),
            (u32::MAX, u32::MAX, u32::MAX),
        ];
        
        for (num, serial, creation) in test_cases {
            let pid = ErlangPid {
                node: "node@host".to_string(),
                num,
                serial,
                creation,
            };
            let mut buf = vec![0u8; 100];
            let mut index = 0;
            let mut buf_opt = Some(&mut buf[..]);
            encode_pid(&mut buf_opt, &mut index, &pid).unwrap();
            assert_eq!(buf[0], ERL_NEW_PID_EXT);
            
            // Verify num, serial, creation are encoded correctly
            let decoded_num = u32::from_be_bytes([buf[index - 12], buf[index - 11], buf[index - 10], buf[index - 9]]);
            let decoded_serial = u32::from_be_bytes([buf[index - 8], buf[index - 7], buf[index - 6], buf[index - 5]]);
            let decoded_creation = u32::from_be_bytes([buf[index - 4], buf[index - 3], buf[index - 2], buf[index - 1]]);
            assert_eq!(decoded_num, num);
            assert_eq!(decoded_serial, serial);
            assert_eq!(decoded_creation, creation);
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
    fn test_erlang_pid_debug() {
        let pid = ErlangPid {
            node: "node@host".to_string(),
            num: 123,
            serial: 456,
            creation: 1,
        };
        
        let debug_str = format!("{:?}", pid);
        assert!(debug_str.contains("ErlangPid"));
    }

    #[test]
    fn test_erlang_pid_clone() {
        let pid = ErlangPid {
            node: "node@host".to_string(),
            num: 123,
            serial: 456,
            creation: 1,
        };
        
        let cloned = pid.clone();
        assert_eq!(pid, cloned);
    }

    #[test]
    fn test_erlang_pid_partial_eq() {
        let pid1 = ErlangPid {
            node: "node@host".to_string(),
            num: 123,
            serial: 456,
            creation: 1,
        };
        let pid2 = ErlangPid {
            node: "node@host".to_string(),
            num: 123,
            serial: 456,
            creation: 1,
        };
        let pid3 = ErlangPid {
            node: "node@host".to_string(),
            num: 124,
            serial: 456,
            creation: 1,
        };
        let pid4 = ErlangPid {
            node: "different@host".to_string(),
            num: 123,
            serial: 456,
            creation: 1,
        };
        
        assert_eq!(pid1, pid2);
        assert_ne!(pid1, pid3);
        assert_ne!(pid1, pid4);
    }

    #[test]
    fn test_erlang_pid_eq() {
        let pid1 = ErlangPid {
            node: "node@host".to_string(),
            num: 123,
            serial: 456,
            creation: 1,
        };
        let pid2 = ErlangPid {
            node: "node@host".to_string(),
            num: 123,
            serial: 456,
            creation: 1,
        };
        let pid3 = ErlangPid {
            node: "node@host".to_string(),
            num: 124,
            serial: 456,
            creation: 1,
        };
        
        assert!(pid1 == pid2);
        assert!(pid1 != pid3);
    }
}

