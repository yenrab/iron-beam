//! Port Encoding Module
//!
//! Provides functionality to encode ports to EI format.
//! Based on lib/erl_interface/src/encode/encode_port.c

use crate::constants::{ERL_V4_PORT_EXT, ERL_NEW_PORT_EXT};
use infrastructure_data_handling::encode_atom::{encode_atom, EncodeAtomError};
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
}

