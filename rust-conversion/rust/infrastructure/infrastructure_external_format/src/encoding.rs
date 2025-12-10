//! Encoding Module
//!
//! Provides core encoding functions for external term format.
//! Based on enc_term(), enc_atom(), enc_pid(), and erts_encode_ext() from external.c

use entities_data_handling::term_hashing::Term;
use entities_data_handling::atom::{AtomTable, AtomEncoding};
use entities_process::Eterm;
use infrastructure_data_handling::{encode_atom, encode_binary};
use infrastructure_code_loading::constants::ERL_VERSION;
use infrastructure_code_loading::encode_integers::encode_longlong;
use infrastructure_code_loading::encode_headers::{encode_tuple_header, encode_map_header, encode_list_header};
use infrastructure_code_loading::encode_pid::{encode_pid, ErlangPid};
use infrastructure_code_loading::encode_port::{encode_port, ErlangPort};
use infrastructure_code_loading::encode_ref::{encode_ref, ErlangRef};
use infrastructure_code_loading::encode_fun::{encode_fun, ErlangFunType};
use infrastructure_bignum_encoding::BignumCodec;
use super::VERSION_MAGIC;

/// Encoding error types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodeError {
    /// Buffer too small
    BufferTooSmall,
    /// Encoding failed
    EncodingFailed(String),
    /// Invalid term
    InvalidTerm(String),
    /// Atom not found
    AtomNotFound,
}

/// Encode a term to external format
///
/// Based on `enc_term()` from external.c. This function encodes an Erlang term
/// to the external term format, which includes a version magic byte (131) followed
/// by the term data in EI format.
///
/// # Arguments
/// * `term` - The term to encode
/// * `atom_table` - Optional atom table for looking up atom names
///
/// # Returns
/// * `Ok(Vec<u8>)` - Encoded bytes in ETF format
/// * `Err(EncodeError)` - Encoding error
pub fn enc_term(term: &Term, atom_table: Option<&AtomTable>) -> Result<Vec<u8>, EncodeError> {
    // Start with version magic byte (131)
    let mut buf = vec![VERSION_MAGIC];
    
    // Encode the term using internal helper
    enc_term_int(&mut buf, term, atom_table)?;
    
    Ok(buf)
}

/// Internal encoding function
///
/// Based on `enc_term_int()` from external.c. This function encodes a term
/// without the version magic byte (used internally).
fn enc_term_int(buf: &mut Vec<u8>, term: &Term, atom_table: Option<&AtomTable>) -> Result<(), EncodeError> {
    match term {
        Term::Nil => {
            // NIL_EXT = 106
            buf.push(106);
            Ok(())
        }
        Term::Small(value) => {
            // Use existing integer encoder
            let start_index = buf.len();
            // Reserve space (will be at most 9 bytes for a big integer)
            buf.resize(buf.len() + 9, 0);
            let mut write_index = 0usize;
            let mut buf_slice = Some(&mut buf[start_index..]);
            encode_longlong(&mut buf_slice, &mut write_index, *value)
                .map_err(|_| EncodeError::EncodingFailed("Failed to encode integer".to_string()))?;
            // Truncate to actual size
            buf.truncate(start_index + write_index);
            Ok(())
        }
        Term::Atom(atom_index) => {
            enc_atom(*atom_index as usize, atom_table, buf)?;
            Ok(())
        }
        Term::Tuple(elements) => {
            // Encode tuple header
            let arity = elements.len();
            let start_index = buf.len();
            buf.resize(buf.len() + 5, 0); // Reserve space for header
            let mut write_index = 0usize;
            let mut buf_slice = Some(&mut buf[start_index..]);
            encode_tuple_header(&mut buf_slice, &mut write_index, arity)
                .map_err(|_| EncodeError::EncodingFailed("Failed to encode tuple header".to_string()))?;
            buf.truncate(start_index + write_index);
            
            // Encode each element
            for element in elements {
                enc_term_int(buf, element, atom_table)?;
            }
            Ok(())
        }
        Term::List { head, tail } => {
            // Encode list as a proper list structure
            // First, count the length by traversing the list
            let mut length = 0;
            let mut current = head.as_ref();
            loop {
                match current {
                    Term::Nil => break,
                    Term::List { head: _, tail: next } => {
                        length += 1;
                        current = next.as_ref();
                    }
                    _ => {
                        length += 1;
                        break;
                    }
                }
            }
            
            // Encode list header
            let start_index = buf.len();
            buf.resize(buf.len() + 5, 0); // Reserve space for header
            let mut write_index = 0usize;
            let mut buf_slice = Some(&mut buf[start_index..]);
            encode_list_header(&mut buf_slice, &mut write_index, length)
                .map_err(|_| EncodeError::EncodingFailed("Failed to encode list header".to_string()))?;
            buf.truncate(start_index + write_index);
            
            // Encode each element
            let mut current = head.as_ref();
            loop {
                match current {
                    Term::Nil => break,
                    Term::List { head: h, tail: t } => {
                        enc_term_int(buf, h, atom_table)?;
                        current = t.as_ref();
                    }
                    _ => {
                        enc_term_int(buf, current, atom_table)?;
                        break;
                    }
                }
            }
            
            // Encode tail (NIL)
            buf.push(106); // NIL_EXT
            Ok(())
        }
        Term::Binary { data, bit_offset: _, bit_size: _ } => {
            // Encode binary
            // encode_binary only takes buf and data, bit_size is not used in the current implementation
            encode_binary(buf, data)
                .map_err(|_| EncodeError::EncodingFailed("Failed to encode binary".to_string()))?;
            Ok(())
        }
        Term::Map(entries) => {
            // Encode map header
            let size = entries.len();
            let start_index = buf.len();
            buf.resize(buf.len() + 5, 0); // Reserve space for header
            let mut write_index = 0usize;
            let mut buf_slice = Some(&mut buf[start_index..]);
            encode_map_header(&mut buf_slice, &mut write_index, size)
                .map_err(|_| EncodeError::EncodingFailed("Failed to encode map header".to_string()))?;
            buf.truncate(start_index + write_index);
            
            // Encode each key-value pair
            for (key, value) in entries {
                enc_term_int(buf, key, atom_table)?;
                enc_term_int(buf, value, atom_table)?;
            }
            Ok(())
        }
        Term::Big(value) => {
            // Encode big integer using bignum codec
            let encoded = BignumCodec::encode(value)
                .map_err(|e| EncodeError::EncodingFailed(format!("Failed to encode big integer: {:?}", e)))?;
            buf.extend_from_slice(&encoded);
            Ok(())
        }
        Term::Float(value) => {
            // NEW_FLOAT_EXT = 70
            buf.push(70);
            // 8-byte IEEE 754 double precision float (big-endian)
            let bytes = value.to_be_bytes();
            buf.extend_from_slice(&bytes);
            Ok(())
        }
        // Note: PID, Port, Ref, and Fun encoding would require additional context
        // For now, we'll return an error for these types
        _ => Err(EncodeError::InvalidTerm(format!("Unsupported term type for encoding: {:?}", term))),
    }
}

/// Encode an atom to external format
///
/// Based on `enc_atom()` from external.c. This function encodes an atom
/// to the external term format.
///
/// # Arguments
/// * `atom_index` - Atom index
/// * `atom_table` - Optional atom table for looking up atom names
/// * `buf` - Buffer to write encoded bytes to
///
/// # Returns
/// * `Ok(())` - Encoding successful
/// * `Err(EncodeError)` - Encoding error
pub fn enc_atom(atom_index: usize, atom_table: Option<&AtomTable>, buf: &mut Vec<u8>) -> Result<(), EncodeError> {
    // Try to get atom name from atom table if available
    let atom_name = if let Some(table) = atom_table {
        if let Some(name_bytes) = table.get_name(atom_index) {
            // Convert Vec<u8> to String for encode_atom
            match String::from_utf8(name_bytes.clone()) {
                Ok(name) => name,
                Err(_) => {
                    // Invalid UTF-8, use Latin1 encoding
                    infrastructure_data_handling::encode_atom::encode_atom_len(
                        buf, &name_bytes, AtomEncoding::Latin1
                    ).map_err(|_| EncodeError::EncodingFailed("Failed to encode atom".to_string()))?;
                    return Ok(());
                }
            }
        } else {
            return Err(EncodeError::AtomNotFound);
        }
    } else {
        // No atom table available, use placeholder
        format!("atom_{}", atom_index)
    };
    
    // Determine encoding: try UTF-8, fall back to Latin1
    let encoding = if atom_name.is_ascii() {
        AtomEncoding::SevenBitAscii
    } else if atom_name.as_bytes().iter().all(|&b| b < 0x80 || b >= 0xA0) {
        AtomEncoding::Latin1
    } else {
        AtomEncoding::Utf8
    };
    
    encode_atom(buf, &atom_name, encoding)
        .map_err(|_| EncodeError::EncodingFailed("Failed to encode atom".to_string()))?;
    Ok(())
}

/// Encode a PID to external format
///
/// Based on `enc_pid()` from external.c. This function encodes a PID
/// to the external term format.
///
/// # Arguments
/// * `pid` - PID to encode
/// * `buf` - Buffer to write encoded bytes to
///
/// # Returns
/// * `Ok(())` - Encoding successful
/// * `Err(EncodeError)` - Encoding error
///
/// # Note
/// This is a simplified version. The full implementation would need
/// process context to get PID details.
pub fn enc_pid(pid: Eterm, buf: &mut Vec<u8>) -> Result<(), EncodeError> {
    // For now, this is a placeholder
    // The full implementation would extract PID details from the Eterm
    // and encode them using encode_pid()
    Err(EncodeError::InvalidTerm("PID encoding not yet fully implemented".to_string()))
}

/// Encode a term to external format (high-level interface)
///
/// Based on `erts_encode_ext()` from external.c. This is the main entry point
/// for encoding terms to external format.
///
/// # Arguments
/// * `term` - The term to encode
/// * `atom_table` - Optional atom table for looking up atom names
///
/// # Returns
/// * `Ok(Vec<u8>)` - Encoded bytes in ETF format
/// * `Err(EncodeError)` - Encoding error
pub fn erts_encode_ext(term: &Term, atom_table: Option<&AtomTable>) -> Result<Vec<u8>, EncodeError> {
    enc_term(term, atom_table)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_enc_term_nil() {
        let term = Term::Nil;
        let encoded = enc_term(&term, None).unwrap();
        assert_eq!(encoded, vec![131, 106]); // VERSION_MAGIC, NIL_EXT
    }
    
    #[test]
    fn test_enc_term_small_integer() {
        let term = Term::Small(42);
        let encoded = enc_term(&term, None).unwrap();
        assert_eq!(encoded[0], 131); // VERSION_MAGIC
        assert_eq!(encoded[1], 97); // SMALL_INTEGER_EXT
        assert_eq!(encoded[2], 42); // value
    }
    
    #[test]
    fn test_enc_term_tuple() {
        let term = Term::Tuple(vec![
            Term::Small(1),
            Term::Small(2),
            Term::Small(3),
        ]);
        let encoded = enc_term(&term, None).unwrap();
        assert_eq!(encoded[0], 131); // VERSION_MAGIC
        assert_eq!(encoded[1], 104); // SMALL_TUPLE_EXT
        assert_eq!(encoded[2], 3); // arity
    }
    
    #[test]
    fn test_enc_atom() {
        let mut atom_table = AtomTable::new(100);
        let atom_index = atom_table.put_index(b"test", AtomEncoding::SevenBitAscii, false).unwrap();
        
        let mut buf = Vec::new();
        enc_atom(atom_index, Some(&atom_table), &mut buf).unwrap();
        
        // Should encode atom (SMALL_ATOM_EXT = 115 or ATOM_EXT = 100)
        assert!(buf[0] == 115 || buf[0] == 100);
    }
}

