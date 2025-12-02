//! PID Encoding Module
//!
//! Provides functionality to encode PIDs to EI format.
//! Based on lib/erl_interface/src/encode/encode_pid.c

use crate::constants::{ERL_NEW_PID_EXT, ERL_PID_EXT};
use infrastructure_data_handling::encode_atom::{encode_atom, EncodeAtomError};
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
}

