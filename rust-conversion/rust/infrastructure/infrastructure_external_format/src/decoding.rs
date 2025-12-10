//! Decoding Module
//!
//! Provides core decoding functions for external term format.
//! Based on dec_term(), dec_atom(), dec_pid(), and erts_decode_ext() from external.c

use entities_data_handling::term_hashing::Term;
use entities_data_handling::atom::AtomTable;
use entities_process::Eterm;
use infrastructure_data_handling::{decode_ei_term, DecodeError as EiDecodeError};
use super::VERSION_MAGIC;

/// Decoding error types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// Buffer too short
    BufferTooShort,
    /// Invalid format
    InvalidFormat(String),
    /// Decoding failed
    DecodingFailed(String),
    /// Atom decode error
    AtomDecodeError(String),
    /// Invalid version magic byte
    InvalidVersion,
}

impl From<EiDecodeError> for DecodeError {
    fn from(err: EiDecodeError) -> Self {
        match err {
            EiDecodeError::BufferTooShort => DecodeError::BufferTooShort,
            EiDecodeError::InvalidFormat(msg) => DecodeError::InvalidFormat(msg),
            EiDecodeError::AtomDecodeError(msg) => DecodeError::AtomDecodeError(msg),
            EiDecodeError::BinaryDecodeError(msg) => DecodeError::DecodingFailed(format!("Binary decode error: {}", msg)),
            _ => DecodeError::DecodingFailed(format!("{:?}", err)),
        }
    }
}

/// Decode a term from external format
///
/// Based on `dec_term()` from external.c. This function decodes an Erlang term
/// from the external term format, which includes a version magic byte (131) followed
/// by the term data in EI format.
///
/// # Arguments
/// * `data` - The encoded bytes in ETF format
///
/// # Returns
/// * `Ok(Term)` - Decoded term
/// * `Err(DecodeError)` - Decoding error
pub fn dec_term(data: &[u8]) -> Result<Term, DecodeError> {
    if data.is_empty() {
        return Err(DecodeError::BufferTooShort);
    }

    // Check version magic byte (131)
    if data[0] != VERSION_MAGIC {
        return Err(DecodeError::InvalidVersion);
    }

    // Decode the term using existing infrastructure (skip version byte)
    let (term, _) = decode_ei_term(data, 1)
        .map_err(|e| DecodeError::from(e))?;

    Ok(term)
}

/// Decode an atom from external format
///
/// Based on `dec_atom()` from external.c. This function decodes an atom
/// from the external term format.
///
/// # Arguments
/// * `data` - The encoded bytes (starting at the atom tag)
/// * `atom_table` - Optional atom table for storing decoded atoms
///
/// # Returns
/// * `Ok((atom_index, new_pos))` - Decoded atom index and new position
/// * `Err(DecodeError)` - Decoding error
pub fn dec_atom(data: &[u8], atom_table: Option<&mut AtomTable>) -> Result<(usize, usize), DecodeError> {
    if data.is_empty() {
        return Err(DecodeError::BufferTooShort);
    }

    let tag = data[0];
    let pos = 1;

    // Use existing atom decoder from infrastructure_data_handling
    // decode_atom expects the tag byte to be at index, so we pass pos - 1
    let (atom_name, new_pos) = infrastructure_data_handling::decode_atom::decode_atom(data, pos - 1)
        .map_err(|e| DecodeError::AtomDecodeError(format!("{:?}", e)))?;
    
    // For now, we'll use a hash of the atom name as the index
    // In the full implementation, we would look up or create the atom in the atom table
    let atom_index = atom_name.len(); // Placeholder: use name length as index

    // If atom table is provided, store the atom
    if let Some(table) = atom_table {
        // The atom name would need to be extracted from the encoded data
        // For now, we just return the index
        // In the full implementation, we would decode the atom name and store it
    }

    Ok((atom_index, new_pos))
}

/// Decode a PID from external format
///
/// Based on `dec_pid()` from external.c. This function decodes a PID
/// from the external term format.
///
/// # Arguments
/// * `data` - The encoded bytes (starting at the PID tag)
///
/// # Returns
/// * `Ok((pid, new_pos))` - Decoded PID and new position
/// * `Err(DecodeError)` - Decoding error
///
/// # Note
/// This is a simplified version. The full implementation would need
/// process context to create the PID.
pub fn dec_pid(data: &[u8]) -> Result<(Eterm, usize), DecodeError> {
    if data.is_empty() {
        return Err(DecodeError::BufferTooShort);
    }

    let tag = data[0];
    
    // Check for PID tags
    match tag {
        103 => { // PID_EXT
            // Old format PID
            if data.len() < 13 {
                return Err(DecodeError::BufferTooShort);
            }
            // PID format: tag + node (atom) + id (4 bytes) + serial (4 bytes) + creation (1 byte)
            // For now, return a placeholder
            // In the full implementation, we would decode the PID details
            Err(DecodeError::DecodingFailed("PID decoding not yet fully implemented".to_string()))
        }
        88 => { // NEW_PID_EXT
            // New format PID
            if data.len() < 17 {
                return Err(DecodeError::BufferTooShort);
            }
            // NEW_PID format: tag + node (atom) + id (4 bytes) + serial (4 bytes) + creation (4 bytes)
            // For now, return a placeholder
            Err(DecodeError::DecodingFailed("PID decoding not yet fully implemented".to_string()))
        }
        _ => Err(DecodeError::InvalidFormat(format!("Invalid PID tag: {}", tag))),
    }
}

/// Decode a term from external format (high-level interface)
///
/// Based on `erts_decode_ext()` from external.c. This is the main entry point
/// for decoding terms from external format.
///
/// # Arguments
/// * `data` - The encoded bytes in ETF format
///
/// # Returns
/// * `Ok(Term)` - Decoded term
/// * `Err(DecodeError)` - Decoding error
pub fn erts_decode_ext(data: &[u8]) -> Result<Term, DecodeError> {
    dec_term(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dec_term_nil() {
        let data = vec![131, 106]; // VERSION_MAGIC, NIL_EXT
        let term = dec_term(&data).unwrap();
        assert!(matches!(term, Term::Nil));
    }
    
    #[test]
    fn test_dec_term_small_integer() {
        let data = vec![131, 97, 42]; // VERSION_MAGIC, SMALL_INTEGER_EXT, value
        let term = dec_term(&data).unwrap();
        match term {
            Term::Small(value) => assert_eq!(value, 42),
            _ => panic!("Expected Small(42)"),
        }
    }
    
    #[test]
    fn test_dec_term_invalid_version() {
        let data = vec![130, 97, 42]; // Wrong version magic byte
        let result = dec_term(&data);
        assert!(matches!(result, Err(DecodeError::InvalidVersion)));
    }
    
    #[test]
    fn test_dec_term_empty() {
        let data = vec![];
        let result = dec_term(&data);
        assert!(matches!(result, Err(DecodeError::BufferTooShort)));
    }
    
    #[test]
    fn test_dec_atom() {
        // SMALL_ATOM_EXT = 115, length = 4, "test"
        let data = vec![115, 4, b't', b'e', b's', b't'];
        let mut atom_table = AtomTable::new(100);
        let (atom_index, new_pos) = dec_atom(&data, Some(&mut atom_table)).unwrap();
        assert_eq!(new_pos, 6); // 1 (tag) + 1 (length) + 4 (bytes)
        assert!(atom_index > 0);
    }
}

