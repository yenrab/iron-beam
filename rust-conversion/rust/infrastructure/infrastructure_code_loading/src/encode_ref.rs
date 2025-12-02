//! Reference Encoding Module
//!
//! Provides functionality to encode references to EI format.
//! Based on lib/erl_interface/src/encode/encode_ref.c

use crate::constants::ERL_NEWER_REFERENCE_EXT;
use infrastructure_data_handling::encode_atom::{encode_atom, EncodeAtomError};
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
}

