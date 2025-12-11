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
    
    #[test]
    fn test_dec_atom_no_table() {
        // SMALL_ATOM_EXT = 115, length = 4, "test"
        let data = vec![115, 4, b't', b'e', b's', b't'];
        let (atom_index, new_pos) = dec_atom(&data, None).unwrap();
        assert_eq!(new_pos, 6);
        assert!(atom_index > 0);
    }
    
    #[test]
    fn test_dec_atom_empty_buffer() {
        let data = vec![];
        let result = dec_atom(&data, None);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DecodeError::BufferTooShort));
    }
    
    #[test]
    fn test_dec_pid_old_format() {
        // PID_EXT = 103
        // Minimum size: tag (1) + node atom (at least 3 bytes) + id (4) + serial (4) + creation (1) = 13
        let data = vec![103, 115, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]; // Minimal valid format
        let result = dec_pid(&data);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecodeError::DecodingFailed(msg) => {
                assert!(msg.contains("PID decoding"));
            }
            _ => panic!("Expected DecodingFailed error"),
        }
    }
    
    #[test]
    fn test_dec_pid_new_format() {
        // NEW_PID_EXT = 88
        // Minimum size: tag (1) + node atom (at least 3 bytes) + id (4) + serial (4) + creation (4) = 17
        let data = vec![88, 115, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]; // Minimal valid format
        let result = dec_pid(&data);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecodeError::DecodingFailed(msg) => {
                assert!(msg.contains("PID decoding"));
            }
            _ => panic!("Expected DecodingFailed error"),
        }
    }
    
    #[test]
    fn test_dec_pid_invalid_tag() {
        let data = vec![99, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let result = dec_pid(&data);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecodeError::InvalidFormat(msg) => {
                assert!(msg.contains("Invalid PID tag"));
            }
            _ => panic!("Expected InvalidFormat error"),
        }
    }
    
    #[test]
    fn test_dec_pid_empty_buffer() {
        let data = vec![];
        let result = dec_pid(&data);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DecodeError::BufferTooShort));
    }
    
    #[test]
    fn test_dec_pid_old_format_buffer_too_short() {
        // PID_EXT = 103, but buffer too short
        let data = vec![103, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]; // 12 bytes, need 13
        let result = dec_pid(&data);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DecodeError::BufferTooShort));
    }
    
    #[test]
    fn test_dec_pid_new_format_buffer_too_short() {
        // NEW_PID_EXT = 88, but buffer too short
        let data = vec![88, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]; // 16 bytes, need 17
        let result = dec_pid(&data);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DecodeError::BufferTooShort));
    }
    
    #[test]
    fn test_erts_decode_ext() {
        let data = vec![131, 106]; // VERSION_MAGIC, NIL_EXT
        let term = erts_decode_ext(&data).unwrap();
        assert!(matches!(term, Term::Nil));
        
        // Should be same as dec_term
        let term2 = dec_term(&data).unwrap();
        match (&term, &term2) {
            (Term::Nil, Term::Nil) => {},
            _ => panic!("Results should match"),
        }
    }
    
    #[test]
    fn test_decode_error_from_ei_decode_error() {
        use infrastructure_data_handling::DecodeError as EiDecodeError;
        
        // Test BufferTooShort
        let ei_err = EiDecodeError::BufferTooShort;
        let decode_err: DecodeError = ei_err.into();
        assert!(matches!(decode_err, DecodeError::BufferTooShort));
        
        // Test InvalidFormat
        let ei_err = EiDecodeError::InvalidFormat("test".to_string());
        let decode_err: DecodeError = ei_err.into();
        match decode_err {
            DecodeError::InvalidFormat(msg) => assert_eq!(msg, "test"),
            _ => panic!("Expected InvalidFormat"),
        }
        
        // Test AtomDecodeError
        let ei_err = EiDecodeError::AtomDecodeError("atom_err".to_string());
        let decode_err: DecodeError = ei_err.into();
        match decode_err {
            DecodeError::AtomDecodeError(msg) => assert!(msg.contains("atom_err")),
            _ => panic!("Expected AtomDecodeError"),
        }
        
        // Test BinaryDecodeError
        let ei_err = EiDecodeError::BinaryDecodeError("binary_err".to_string());
        let decode_err: DecodeError = ei_err.into();
        match decode_err {
            DecodeError::DecodingFailed(msg) => assert!(msg.contains("Binary decode error")),
            _ => panic!("Expected DecodingFailed"),
        }
    }
    
    #[test]
    fn test_decode_error_debug() {
        let error1 = DecodeError::BufferTooShort;
        let error2 = DecodeError::InvalidFormat("test".to_string());
        let error3 = DecodeError::DecodingFailed("test".to_string());
        let error4 = DecodeError::AtomDecodeError("test".to_string());
        let error5 = DecodeError::InvalidVersion;
        
        let debug_str1 = format!("{:?}", error1);
        let debug_str2 = format!("{:?}", error2);
        let debug_str3 = format!("{:?}", error3);
        let debug_str4 = format!("{:?}", error4);
        let debug_str5 = format!("{:?}", error5);
        
        assert!(debug_str1.contains("BufferTooShort"));
        assert!(debug_str2.contains("InvalidFormat"));
        assert!(debug_str3.contains("DecodingFailed"));
        assert!(debug_str4.contains("AtomDecodeError"));
        assert!(debug_str5.contains("InvalidVersion"));
    }
    
    #[test]
    fn test_decode_error_clone() {
        let error1 = DecodeError::BufferTooShort;
        let error2 = DecodeError::InvalidFormat("test".to_string());
        let error3 = DecodeError::DecodingFailed("test".to_string());
        let error4 = DecodeError::AtomDecodeError("test".to_string());
        let error5 = DecodeError::InvalidVersion;
        
        let cloned1 = error1.clone();
        let cloned2 = error2.clone();
        let cloned3 = error3.clone();
        let cloned4 = error4.clone();
        let cloned5 = error5.clone();
        
        assert_eq!(error1, cloned1);
        assert_eq!(error2, cloned2);
        assert_eq!(error3, cloned3);
        assert_eq!(error4, cloned4);
        assert_eq!(error5, cloned5);
    }
    
    #[test]
    fn test_decode_error_partial_eq() {
        let error1 = DecodeError::BufferTooShort;
        let error2 = DecodeError::BufferTooShort;
        let error3 = DecodeError::InvalidFormat("test".to_string());
        let error4 = DecodeError::InvalidFormat("test".to_string());
        let error5 = DecodeError::InvalidFormat("different".to_string());
        let error6 = DecodeError::InvalidVersion;
        
        assert_eq!(error1, error2);
        assert_eq!(error3, error4);
        assert_ne!(error3, error5);
        assert_ne!(error1, error3);
        assert_ne!(error1, error6);
    }
    
    #[test]
    fn test_decode_error_eq() {
        let error1 = DecodeError::BufferTooShort;
        let error2 = DecodeError::BufferTooShort;
        let error3 = DecodeError::InvalidVersion;
        
        assert!(error1 == error2);
        assert!(error1 != error3);
    }
    
    #[test]
    fn test_dec_term_tuple() {
        // Encode a tuple first, then decode it
        use super::super::encoding::enc_term;
        let term = Term::Tuple(vec![
            Term::Small(1),
            Term::Small(2),
            Term::Small(3),
        ]);
        let encoded = enc_term(&term, None).unwrap();
        let decoded = dec_term(&encoded).unwrap();
        match decoded {
            Term::Tuple(elements) => {
                assert_eq!(elements.len(), 3);
            }
            _ => panic!("Expected Tuple"),
        }
    }
    
    #[test]
    fn test_dec_term_list() {
        // Encode a list first, then decode it
        use super::super::encoding::enc_term;
        let term = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::Nil),
        };
        let encoded = enc_term(&term, None).unwrap();
        let decoded = dec_term(&encoded).unwrap();
        match decoded {
            Term::List { head, tail } => {
                match *head {
                    Term::Small(1) => {},
                    _ => panic!("Expected Small(1)"),
                }
                match *tail {
                    Term::Nil => {},
                    _ => panic!("Expected Nil"),
                }
            }
            _ => panic!("Expected List"),
        }
    }
    
    #[test]
    fn test_dec_term_binary() {
        // Encode a binary first, then decode it
        use super::super::encoding::enc_term;
        let term = Term::Binary {
            data: vec![1, 2, 3, 4],
            bit_offset: 0,
            bit_size: 32,
        };
        let encoded = enc_term(&term, None).unwrap();
        let decoded = dec_term(&encoded).unwrap();
        match decoded {
            Term::Binary { data, .. } => {
                assert_eq!(data, vec![1, 2, 3, 4]);
            }
            _ => panic!("Expected Binary"),
        }
    }
    
    #[test]
    fn test_dec_term_map() {
        // Encode a map first, then decode it
        use super::super::encoding::enc_term;
        let term = Term::Map(vec![
            (Term::Small(1), Term::Small(2)),
            (Term::Small(3), Term::Small(4)),
        ]);
        let encoded = enc_term(&term, None).unwrap();
        let decoded = dec_term(&encoded).unwrap();
        match decoded {
            Term::Map(entries) => {
                assert_eq!(entries.len(), 2);
            }
            _ => panic!("Expected Map"),
        }
    }
    
    #[test]
    fn test_dec_term_float() {
        // Encode a float first, then decode it
        use super::super::encoding::enc_term;
        let term = Term::Float(3.14159);
        let encoded = enc_term(&term, None).unwrap();
        let decoded = dec_term(&encoded).unwrap();
        match decoded {
            Term::Float(value) => {
                assert!((value - 3.14159).abs() < 0.0001);
            }
            _ => panic!("Expected Float"),
        }
    }
    
    #[test]
    fn test_dec_term_big_integer() {
        // Encode a big integer first, then decode it
        use super::super::encoding::enc_term;
        use entities_utilities::BigNumber;
        let big_num = BigNumber::from_i64(i64::MAX);
        let term = Term::Big(big_num);
        let encoded = enc_term(&term, None).unwrap();
        let decoded = dec_term(&encoded).unwrap();
        match decoded {
            Term::Big(_) => {
                // Big integer decoded successfully
            }
            _ => panic!("Expected Big"),
        }
    }
    
    #[test]
    fn test_dec_term_atom() {
        // Encode an atom first, then decode it
        use super::super::encoding::enc_term;
        use entities_data_handling::atom::{AtomTable, AtomEncoding};
        let mut atom_table = AtomTable::new(100);
        let atom_index = atom_table.put_index(b"hello", AtomEncoding::SevenBitAscii, false).unwrap();
        let term = Term::Atom(atom_index as u32);
        let encoded = enc_term(&term, Some(&atom_table)).unwrap();
        let decoded = dec_term(&encoded).unwrap();
        match decoded {
            Term::Atom(_) => {
                // Atom decoded successfully
            }
            _ => panic!("Expected Atom"),
        }
    }
    
    #[test]
    fn test_dec_term_negative_integer() {
        // Encode a negative integer first, then decode it
        use super::super::encoding::enc_term;
        let term = Term::Small(-42);
        let encoded = enc_term(&term, None).unwrap();
        let decoded = dec_term(&encoded).unwrap();
        match decoded {
            Term::Small(value) => assert_eq!(value, -42),
            _ => panic!("Expected Small(-42)"),
        }
    }
    
    #[test]
    fn test_dec_term_zero() {
        // Encode zero first, then decode it
        use super::super::encoding::enc_term;
        let term = Term::Small(0);
        let encoded = enc_term(&term, None).unwrap();
        let decoded = dec_term(&encoded).unwrap();
        match decoded {
            Term::Small(value) => assert_eq!(value, 0),
            _ => panic!("Expected Small(0)"),
        }
    }
    
    #[test]
    fn test_dec_term_large_integer() {
        // Encode a large integer first, then decode it
        use super::super::encoding::enc_term;
        let term = Term::Small(256);
        let encoded = enc_term(&term, None).unwrap();
        let decoded = dec_term(&encoded).unwrap();
        match decoded {
            Term::Small(value) => assert_eq!(value, 256),
            _ => panic!("Expected Small(256)"),
        }
    }
    
    #[test]
    fn test_dec_atom_atom_ext() {
        // ATOM_EXT = 100, length (2 bytes) = 4, "test"
        let data = vec![100, 0, 4, b't', b'e', b's', b't'];
        let (atom_index, new_pos) = dec_atom(&data, None).unwrap();
        assert_eq!(new_pos, 7); // 1 (tag) + 2 (length) + 4 (bytes) = 7
        assert!(atom_index > 0);
    }
    
    #[test]
    fn test_dec_term_only_version_magic() {
        // Only version magic byte, no term data
        let data = vec![131];
        let result = dec_term(&data);
        // This should fail because there's no term data after the version magic
        assert!(result.is_err());
    }
}

