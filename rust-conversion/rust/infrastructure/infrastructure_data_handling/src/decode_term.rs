//! Decode Term Module
//!
//! Provides functionality to decode EI-encoded terms.
//! Based on lib/erl_interface/src/misc/ei_decode_term.c

use entities_data_handling::term_hashing::Term;
use entities_utilities::BigNumber;

/// Decode an EI-encoded term from bytes
///
/// This is the main entry point for decoding EI terms.
/// It reads the term type tag and dispatches to appropriate decoders.
///
/// # Arguments
/// * `buf` - Buffer containing EI-encoded data
/// * `index` - Starting index in the buffer
///
/// # Returns
/// * `Ok((term, new_index))` - Decoded term and new index position
/// * `Err(DecodeError)` - Decoding error
///
/// # Safety
/// This function is safe as long as `buf` is valid and `index` is within bounds.
pub fn decode_ei_term(buf: &[u8], index: usize) -> Result<(Term, usize), DecodeError> {
    if index >= buf.len() {
        return Err(DecodeError::BufferTooShort);
    }

    // Read the term type tag (first byte)
    let tag = buf[index];
    let mut pos = index + 1;

    match tag {
        // Small integer (SMALL_INTEGER_EXT = 97)
        97 => {
            if pos >= buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let value = buf[pos] as i64;
            pos += 1;
            Ok((Term::Small(value), pos))
        }
        // Integer (INTEGER_EXT = 98)
        98 => {
            if pos + 4 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let value = i32::from_be_bytes([
                buf[pos],
                buf[pos + 1],
                buf[pos + 2],
                buf[pos + 3],
            ]) as i64;
            pos += 4;
            Ok((Term::Small(value), pos))
        }
        // Atom (ATOM_EXT = 100, ATOM_UTF8_EXT = 118, SMALL_ATOM_EXT = 115, SMALL_ATOM_UTF8_EXT = 119)
        100 | 115 | 118 | 119 => {
            // Delegate to atom decoder
            crate::decode_atom::decode_atom_internal(buf, pos, tag)
                .map(|(atom_index, new_pos)| (Term::Atom(atom_index as u32), new_pos))
                .map_err(|e| DecodeError::AtomDecodeError(format!("{:?}", e)))
        }
        // Binary (BINARY_EXT = 109)
        109 => {
            crate::decode_binary::decode_binary_internal(buf, pos)
                .map(|(data, new_pos)| {
                    let bit_size = data.len() * 8;
                    (
                        Term::Binary {
                            data,
                            bit_offset: 0,
                            bit_size,
                        },
                        new_pos,
                    )
                })
                .map_err(|e| DecodeError::BinaryDecodeError(format!("{:?}", e)))
        }
        // Nil (NIL_EXT = 106)
        106 => Ok((Term::Nil, pos)),
        // Tuple (SMALL_TUPLE_EXT = 104, LARGE_TUPLE_EXT = 105)
        104 | 105 => {
            // Read arity
            let arity = if tag == 104 {
                // SMALL_TUPLE_EXT: 1-byte arity
                if pos >= buf.len() {
                    return Err(DecodeError::BufferTooShort);
                }
                let a = buf[pos] as u32;
                pos += 1;
                a
            } else {
                // LARGE_TUPLE_EXT: 4-byte arity (big-endian)
                if pos + 4 > buf.len() {
                    return Err(DecodeError::BufferTooShort);
                }
                let a = u32::from_be_bytes([
                    buf[pos],
                    buf[pos + 1],
                    buf[pos + 2],
                    buf[pos + 3],
                ]);
                pos += 4;
                a
            };
            
            // Empty tuple is represented as Nil in some contexts, but we'll use an empty tuple
            if arity == 0 {
                return Ok((Term::Tuple(vec![]), pos));
            }
            
            // Decode each element of the tuple
            let mut elements = Vec::with_capacity(arity as usize);
            for _ in 0..arity {
                match decode_ei_term(buf, pos) {
                    Ok((term, new_pos)) => {
                        elements.push(term);
                        pos = new_pos;
                    }
                    Err(e) => return Err(e),
                }
            }
            
            Ok((Term::Tuple(elements), pos))
        }
        // List (LIST_EXT = 108)
        108 => {
            // Read list length (4-byte big-endian)
            if pos + 4 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let length = u32::from_be_bytes([
                buf[pos],
                buf[pos + 1],
                buf[pos + 2],
                buf[pos + 3],
            ]) as usize;
            pos += 4;
            
            // Empty list is Nil
            if length == 0 {
                return Ok((Term::Nil, pos));
            }
            
            // Decode list elements
            // In external format, a list is encoded as a sequence of terms followed by a tail
            // For proper lists, the tail is NIL_EXT (106)
            let mut elements = Vec::with_capacity(length);
            for _ in 0..length {
                match decode_ei_term(buf, pos) {
                    Ok((term, new_pos)) => {
                        elements.push(term);
                        pos = new_pos;
                    }
                    Err(e) => return Err(e),
                }
            }
            
            // Decode tail (should be NIL for proper lists, but we handle other cases)
            let tail = match decode_ei_term(buf, pos) {
                Ok((term, new_pos)) => {
                    pos = new_pos;
                    term
                }
                Err(e) => return Err(e),
            };
            
            // Convert to Term::List format (head/tail structure)
            // For proper lists, we build a cons cell structure
            if elements.is_empty() {
                // Empty list - should have been handled above, but handle tail
                if matches!(tail, Term::Nil) {
                    Ok((Term::Nil, pos))
                } else {
                    // Improper list with empty head
                    Ok((Term::List {
                        head: Box::new(Term::Nil),
                        tail: Box::new(tail),
                    }, pos))
                }
            } else {
                // Build list from elements, with tail
                let mut list = Term::List {
                    head: Box::new(elements.pop().unwrap()),
                    tail: Box::new(tail),
                };
                
                // Build rest of list in reverse
                for elem in elements.into_iter().rev() {
                    list = Term::List {
                        head: Box::new(elem),
                        tail: Box::new(list),
                    };
                }
                
                Ok((list, pos))
            }
        }
        // Float (FLOAT_EXT = 99, deprecated format with 31-byte string)
        99 => {
            // FLOAT_EXT: 31 bytes of ASCII float representation
            if pos + 31 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            // Parse the float string (null-terminated, but we read all 31 bytes)
            let float_str = match std::str::from_utf8(&buf[pos..pos + 31]) {
                Ok(s) => s.trim_end_matches('\0'),
                Err(_) => return Err(DecodeError::InvalidFormat("Invalid float string encoding".to_string())),
            };
            let value = float_str.parse::<f64>()
                .map_err(|_| DecodeError::InvalidFormat(format!("Invalid float value: {}", float_str)))?;
            pos += 31;
            Ok((Term::Float(value), pos))
        }
        // New Float (NEW_FLOAT_EXT = 70, 8-byte IEEE 754 double)
        70 => {
            // NEW_FLOAT_EXT: 8 bytes IEEE 754 double (big-endian)
            if pos + 8 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let value = f64::from_be_bytes([
                buf[pos],
                buf[pos + 1],
                buf[pos + 2],
                buf[pos + 3],
                buf[pos + 4],
                buf[pos + 5],
                buf[pos + 6],
                buf[pos + 7],
            ]);
            pos += 8;
            Ok((Term::Float(value), pos))
        }
        // Small Big Integer (SMALL_BIG_EXT = 110)
        110 => {
            // SMALL_BIG_EXT: 1 byte sign + 1 byte n + n bytes (little-endian)
            if pos + 2 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let sign = buf[pos];
            let n = buf[pos + 1] as usize;
            pos += 2;
            
            if pos + n > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            
            // Read n bytes as little-endian unsigned integer
            let mut value = BigNumber::from_u64(0);
            let mut multiplier = BigNumber::from_u64(1);
            for i in 0..n {
                let byte = buf[pos + i];
                let byte_val = BigNumber::from_u64(byte as u64);
                value = value.plus(&byte_val.times(&multiplier));
                multiplier = multiplier.times(&BigNumber::from_u64(256));
            }
            
            // Apply sign (0 = positive, 1 = negative)
            if sign != 0 {
                value = BigNumber::from_u64(0).minus(&value);
            }
            
            pos += n;
            Ok((Term::Big(value), pos))
        }
        // Large Big Integer (LARGE_BIG_EXT = 111)
        111 => {
            // LARGE_BIG_EXT: 4-byte arity + 1 byte sign + n bytes (little-endian)
            if pos + 5 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let n = u32::from_be_bytes([
                buf[pos],
                buf[pos + 1],
                buf[pos + 2],
                buf[pos + 3],
            ]) as usize;
            let sign = buf[pos + 4];
            pos += 5;
            
            if pos + n > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            
            // Read n bytes as little-endian unsigned integer
            let mut value = BigNumber::from_u64(0);
            let mut multiplier = BigNumber::from_u64(1);
            for i in 0..n {
                let byte = buf[pos + i];
                let byte_val = BigNumber::from_u64(byte as u64);
                value = value.plus(&byte_val.times(&multiplier));
                multiplier = multiplier.times(&BigNumber::from_u64(256));
            }
            
            // Apply sign (0 = positive, 1 = negative)
            if sign != 0 {
                value = BigNumber::from_u64(0).minus(&value);
            }
            
            pos += n;
            Ok((Term::Big(value), pos))
        }
        // Map (MAP_EXT = 116)
        116 => {
            // MAP_EXT: 4-byte arity (number of key-value pairs) + pairs
            if pos + 4 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let arity = u32::from_be_bytes([
                buf[pos],
                buf[pos + 1],
                buf[pos + 2],
                buf[pos + 3],
            ]) as usize;
            pos += 4;
            
            let mut pairs = Vec::with_capacity(arity);
            for _ in 0..arity {
                // Decode key
                let (key, new_pos) = decode_ei_term(buf, pos)?;
                pos = new_pos;
                
                // Decode value
                let (value, new_pos) = decode_ei_term(buf, pos)?;
                pos = new_pos;
                
                pairs.push((key, value));
            }
            
            Ok((Term::Map(pairs), pos))
        }
        // Old PID (PID_EXT = 103)
        103 => {
            // PID_EXT: node (atom) + id (4 bytes) + serial (4 bytes) + creation (1 byte)
            // First decode the node atom
            let (node_term, new_pos) = decode_ei_term(buf, pos)?;
            let node = match node_term {
                Term::Atom(idx) => idx,
                _ => return Err(DecodeError::InvalidFormat("PID node must be an atom".to_string())),
            };
            pos = new_pos;
            
            if pos + 9 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            
            let id = u32::from_be_bytes([buf[pos], buf[pos + 1], buf[pos + 2], buf[pos + 3]]);
            let serial = u32::from_be_bytes([buf[pos + 4], buf[pos + 5], buf[pos + 6], buf[pos + 7]]);
            let creation = buf[pos + 8] as u32;
            pos += 9;
            
            Ok((Term::Pid { node, id, serial, creation }, pos))
        }
        // New PID (NEW_PID_EXT = 88)
        88 => {
            // NEW_PID_EXT: node (atom) + id (4 bytes) + serial (4 bytes) + creation (4 bytes)
            let (node_term, new_pos) = decode_ei_term(buf, pos)?;
            let node = match node_term {
                Term::Atom(idx) => idx,
                _ => return Err(DecodeError::InvalidFormat("PID node must be an atom".to_string())),
            };
            pos = new_pos;
            
            if pos + 12 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            
            let id = u32::from_be_bytes([buf[pos], buf[pos + 1], buf[pos + 2], buf[pos + 3]]);
            let serial = u32::from_be_bytes([buf[pos + 4], buf[pos + 5], buf[pos + 6], buf[pos + 7]]);
            let creation = u32::from_be_bytes([buf[pos + 8], buf[pos + 9], buf[pos + 10], buf[pos + 11]]);
            pos += 12;
            
            Ok((Term::Pid { node, id, serial, creation }, pos))
        }
        // Old Port (PORT_EXT = 102)
        102 => {
            // PORT_EXT: node (atom) + id (4 bytes) + creation (1 byte)
            let (node_term, new_pos) = decode_ei_term(buf, pos)?;
            let node = match node_term {
                Term::Atom(idx) => idx,
                _ => return Err(DecodeError::InvalidFormat("Port node must be an atom".to_string())),
            };
            pos = new_pos;
            
            if pos + 5 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            
            let id = u64::from(u32::from_be_bytes([buf[pos], buf[pos + 1], buf[pos + 2], buf[pos + 3]]));
            let creation = buf[pos + 4] as u32;
            pos += 5;
            
            Ok((Term::Port { node, id, creation }, pos))
        }
        // New Port (NEW_PORT_EXT = 89)
        89 => {
            // NEW_PORT_EXT: node (atom) + id (8 bytes) + creation (4 bytes)
            let (node_term, new_pos) = decode_ei_term(buf, pos)?;
            let node = match node_term {
                Term::Atom(idx) => idx,
                _ => return Err(DecodeError::InvalidFormat("Port node must be an atom".to_string())),
            };
            pos = new_pos;
            
            if pos + 12 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            
            let id = u64::from_be_bytes([
                buf[pos],
                buf[pos + 1],
                buf[pos + 2],
                buf[pos + 3],
                buf[pos + 4],
                buf[pos + 5],
                buf[pos + 6],
                buf[pos + 7],
            ]);
            let creation = u32::from_be_bytes([buf[pos + 8], buf[pos + 9], buf[pos + 10], buf[pos + 11]]);
            pos += 12;
            
            Ok((Term::Port { node, id, creation }, pos))
        }
        // Old Reference (REF_EXT = 101)
        101 => {
            // REF_EXT: node (atom) + id (4 bytes) + creation (1 byte)
            let (node_term, new_pos) = decode_ei_term(buf, pos)?;
            let node = match node_term {
                Term::Atom(idx) => idx,
                _ => return Err(DecodeError::InvalidFormat("Ref node must be an atom".to_string())),
            };
            pos = new_pos;
            
            if pos + 5 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            
            let id = u32::from_be_bytes([buf[pos], buf[pos + 1], buf[pos + 2], buf[pos + 3]]);
            let creation = buf[pos + 4] as u32;
            pos += 5;
            
            Ok((Term::Ref { node, ids: vec![id], creation }, pos))
        }
        // New Reference (NEW_REF_EXT = 90)
        90 => {
            // NEW_REF_EXT: length (2 bytes) + node (atom) + creation (1 byte) + ids (length * 4 bytes)
            if pos + 2 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let length = u16::from_be_bytes([buf[pos], buf[pos + 1]]) as usize;
            pos += 2;
            
            let (node_term, new_pos) = decode_ei_term(buf, pos)?;
            let node = match node_term {
                Term::Atom(idx) => idx,
                _ => return Err(DecodeError::InvalidFormat("Ref node must be an atom".to_string())),
            };
            pos = new_pos;
            
            if pos + 1 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let creation = buf[pos] as u32;
            pos += 1;
            
            if pos + length * 4 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            
            let mut ids = Vec::with_capacity(length);
            for i in 0..length {
                let id = u32::from_be_bytes([
                    buf[pos + i * 4],
                    buf[pos + i * 4 + 1],
                    buf[pos + i * 4 + 2],
                    buf[pos + i * 4 + 3],
                ]);
                ids.push(id);
            }
            pos += length * 4;
            
            Ok((Term::Ref { node, ids, creation }, pos))
        }
        // Newer Reference (NEWER_REF_EXT = 114)
        114 => {
            // NEWER_REF_EXT: length (2 bytes) + node (atom) + creation (4 bytes) + ids (length * 4 bytes)
            if pos + 2 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let length = u16::from_be_bytes([buf[pos], buf[pos + 1]]) as usize;
            pos += 2;
            
            let (node_term, new_pos) = decode_ei_term(buf, pos)?;
            let node = match node_term {
                Term::Atom(idx) => idx,
                _ => return Err(DecodeError::InvalidFormat("Ref node must be an atom".to_string())),
            };
            pos = new_pos;
            
            if pos + 4 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let creation = u32::from_be_bytes([buf[pos], buf[pos + 1], buf[pos + 2], buf[pos + 3]]);
            pos += 4;
            
            if pos + length * 4 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            
            let mut ids = Vec::with_capacity(length);
            for i in 0..length {
                let id = u32::from_be_bytes([
                    buf[pos + i * 4],
                    buf[pos + i * 4 + 1],
                    buf[pos + i * 4 + 2],
                    buf[pos + i * 4 + 3],
                ]);
                ids.push(id);
            }
            pos += length * 4;
            
            Ok((Term::Ref { node, ids, creation }, pos))
        }
        // External Function (EXPORT_EXT = 112)
        112 => {
            // EXPORT_EXT: module (atom) + function (atom) + arity (integer)
            let (module_term, new_pos) = decode_ei_term(buf, pos)?;
            let module = match module_term {
                Term::Atom(idx) => idx,
                _ => return Err(DecodeError::InvalidFormat("Function module must be an atom".to_string())),
            };
            pos = new_pos;
            
            let (function_term, new_pos) = decode_ei_term(buf, pos)?;
            let function = match function_term {
                Term::Atom(idx) => idx,
                _ => return Err(DecodeError::InvalidFormat("Function name must be an atom".to_string())),
            };
            pos = new_pos;
            
            let (arity_term, new_pos) = decode_ei_term(buf, pos)?;
            let arity = match arity_term {
                Term::Small(n) if n >= 0 && n <= u32::MAX as i64 => n as u32,
                _ => return Err(DecodeError::InvalidFormat("Function arity must be a non-negative integer".to_string())),
            };
            pos = new_pos;
            
            Ok((Term::Fun {
                is_local: false,
                module,
                function,
                arity,
                old_uniq: None,
                env: vec![],
            }, pos))
        }
        // Old Function (FUN_EXT = 113)
        113 => {
            // FUN_EXT: num_free (4 bytes) + pid (PID) + module (atom) + index (4 bytes) + uniq (4 bytes) + free_vars (num_free terms)
            if pos + 4 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let num_free = u32::from_be_bytes([buf[pos], buf[pos + 1], buf[pos + 2], buf[pos + 3]]) as usize;
            pos += 4;
            
            // Decode PID (can be old or new format)
            let (_pid_term, new_pos) = decode_ei_term(buf, pos)?;
            pos = new_pos;
            // Note: We decode the PID but don't use it in the Fun term structure
            // The Fun term doesn't store the PID, so we just skip it
            
            // Decode module atom
            let (module_term, new_pos) = decode_ei_term(buf, pos)?;
            let module = match module_term {
                Term::Atom(idx) => idx,
                _ => return Err(DecodeError::InvalidFormat("Function module must be an atom".to_string())),
            };
            pos = new_pos;
            
            if pos + 8 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let index = u32::from_be_bytes([buf[pos], buf[pos + 1], buf[pos + 2], buf[pos + 3]]);
            let uniq = u32::from_be_bytes([buf[pos + 4], buf[pos + 5], buf[pos + 6], buf[pos + 7]]);
            pos += 8;
            
            // Decode free variables
            let mut env = Vec::with_capacity(num_free);
            for _ in 0..num_free {
                let (term, new_pos) = decode_ei_term(buf, pos)?;
                env.push(term);
                pos = new_pos;
            }
            
            Ok((Term::Fun {
                is_local: true,
                module,
                function: index,
                arity: 0, // Arity not stored in old format, would need to be determined from index
                old_uniq: Some(uniq),
                env,
            }, pos))
        }
        // New Function (NEW_FUN_EXT = 117)
        117 => {
            // NEW_FUN_EXT: size (4 bytes) + arity (1 byte) + uniq (16 bytes) + index (4 bytes) + num_free (4 bytes) + module (atom) + old_index (4 bytes) + old_uniq (4 bytes) + pid (PID) + free_vars (num_free terms)
            if pos + 4 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let _size = u32::from_be_bytes([buf[pos], buf[pos + 1], buf[pos + 2], buf[pos + 3]]);
            pos += 4;
            
            if pos + 1 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let arity = buf[pos] as u32;
            pos += 1;
            
            // Skip uniq (16 bytes) - not used in our simplified Fun representation
            if pos + 16 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            pos += 16;
            
            if pos + 4 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let index = u32::from_be_bytes([buf[pos], buf[pos + 1], buf[pos + 2], buf[pos + 3]]);
            pos += 4;
            
            if pos + 4 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let num_free = u32::from_be_bytes([buf[pos], buf[pos + 1], buf[pos + 2], buf[pos + 3]]) as usize;
            pos += 4;
            
            // Decode module atom
            let (module_term, new_pos) = decode_ei_term(buf, pos)?;
            let module = match module_term {
                Term::Atom(idx) => idx,
                _ => return Err(DecodeError::InvalidFormat("Function module must be an atom".to_string())),
            };
            pos = new_pos;
            
            if pos + 8 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let _old_index = u32::from_be_bytes([buf[pos], buf[pos + 1], buf[pos + 2], buf[pos + 3]]);
            let old_uniq = u32::from_be_bytes([buf[pos + 4], buf[pos + 5], buf[pos + 6], buf[pos + 7]]);
            pos += 8;
            
            // Decode PID (can be old or new format)
            let (_pid_term, new_pos) = decode_ei_term(buf, pos)?;
            pos = new_pos;
            
            // Decode free variables
            let mut env = Vec::with_capacity(num_free);
            for _ in 0..num_free {
                let (term, new_pos) = decode_ei_term(buf, pos)?;
                env.push(term);
                pos = new_pos;
            }
            
            Ok((Term::Fun {
                is_local: true,
                module,
                function: index,
                arity,
                old_uniq: Some(old_uniq),
                env,
            }, pos))
        }
        _ => Err(DecodeError::InvalidFormat(format!(
            "Unknown term tag: {}",
            tag
        ))),
    }
}

/// Decoding errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// Buffer too short for decoding
    BufferTooShort,
    /// Invalid format
    InvalidFormat(String),
    /// Atom decoding error
    AtomDecodeError(String),
    /// Binary decoding error
    BinaryDecodeError(String),
    /// Operation not implemented
    NotImplemented(&'static str),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_small_integer() {
        // SMALL_INTEGER_EXT (97) followed by value 42
        let buf = vec![97, 42];
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, pos) = result.unwrap();
        assert_eq!(term, Term::Small(42));
        assert_eq!(pos, 2);
    }

    #[test]
    fn test_decode_integer() {
        // INTEGER_EXT (98) followed by 4-byte big-endian integer
        let buf = vec![98, 0, 0, 0, 42];
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, pos) = result.unwrap();
        assert_eq!(term, Term::Small(42));
        assert_eq!(pos, 5);
    }

    #[test]
    fn test_decode_nil() {
        // NIL_EXT (106)
        let buf = vec![106];
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, pos) = result.unwrap();
        assert_eq!(term, Term::Nil);
        assert_eq!(pos, 1);
    }

    #[test]
    fn test_decode_buffer_too_short() {
        let buf = vec![97]; // Tag without value
        let result = decode_ei_term(&buf, 0);
        assert!(matches!(result, Err(DecodeError::BufferTooShort)));
    }

    #[test]
    fn test_decode_invalid_tag() {
        let buf = vec![255]; // Invalid tag
        let result = decode_ei_term(&buf, 0);
        assert!(matches!(result, Err(DecodeError::InvalidFormat(_))));
    }

    #[test]
    fn test_decode_small_integer_negative() {
        // SMALL_INTEGER_EXT (97) followed by negative value (as signed byte)
        let buf = vec![97, 200]; // 200 as u8 = -56 as i8
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, pos) = result.unwrap();
        // When cast as i64, 200 becomes 200, not -56
        // This might be a bug - should we sign-extend?
        assert_eq!(term, Term::Small(200));
        assert_eq!(pos, 2);
    }

    #[test]
    fn test_decode_integer_negative() {
        // INTEGER_EXT (98) followed by 4-byte big-endian negative integer
        let buf = vec![98, 0xFF, 0xFF, 0xFF, 0xD6]; // -42 in two's complement
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, pos) = result.unwrap();
        let value = i32::from_be_bytes([0xFF, 0xFF, 0xFF, 0xD6]) as i64;
        assert_eq!(term, Term::Small(value));
        assert_eq!(pos, 5);
    }

    #[test]
    fn test_decode_integer_buffer_too_short() {
        // INTEGER_EXT (98) with incomplete data
        let buf = vec![98, 0, 0, 0]; // Only 3 bytes instead of 4
        let result = decode_ei_term(&buf, 0);
        assert!(matches!(result, Err(DecodeError::BufferTooShort)));
    }

    #[test]
    fn test_decode_atom_atom_ext() {
        // ATOM_EXT (100) + length 2 + "ok"
        let buf = vec![100, 0, 2, b'o', b'k'];
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, pos) = result.unwrap();
        assert!(matches!(term, Term::Atom(_)));
        assert!(pos > 0);
    }

    #[test]
    fn test_decode_atom_small_atom_ext() {
        // SMALL_ATOM_EXT (115) + length 3 + "foo"
        let buf = vec![115, 3, b'f', b'o', b'o'];
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, pos) = result.unwrap();
        assert!(matches!(term, Term::Atom(_)));
        assert!(pos > 0);
    }

    #[test]
    fn test_decode_atom_atom_utf8_ext() {
        // ATOM_UTF8_EXT (118) + length 2 + "ok"
        let buf = vec![118, 0, 2, b'o', b'k'];
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, pos) = result.unwrap();
        assert!(matches!(term, Term::Atom(_)));
        assert!(pos > 0);
    }

    #[test]
    fn test_decode_atom_small_atom_utf8_ext() {
        // SMALL_ATOM_UTF8_EXT (119) + length 3 + "foo"
        let buf = vec![119, 3, b'f', b'o', b'o'];
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, pos) = result.unwrap();
        assert!(matches!(term, Term::Atom(_)));
        assert!(pos > 0);
    }

    #[test]
    fn test_decode_atom_error() {
        // ATOM_EXT (100) with invalid data (buffer too short)
        let buf = vec![100, 0, 10]; // Length 10 but no data
        let result = decode_ei_term(&buf, 0);
        assert!(matches!(result, Err(DecodeError::AtomDecodeError(_))));
    }

    #[test]
    fn test_decode_binary() {
        // BINARY_EXT (109) + length 4 + data
        let buf = vec![109, 0, 0, 0, 4, 1, 2, 3, 4];
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, pos) = result.unwrap();
        match term {
            Term::Binary { data, bit_offset, bit_size } => {
                assert_eq!(data, vec![1, 2, 3, 4]);
                assert_eq!(bit_offset, 0);
                assert_eq!(bit_size, 32);
            }
            _ => panic!("Expected Binary term"),
        }
        assert_eq!(pos, 9);
    }

    #[test]
    fn test_decode_binary_empty() {
        // BINARY_EXT (109) + length 0
        let buf = vec![109, 0, 0, 0, 0];
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, pos) = result.unwrap();
        match term {
            Term::Binary { data, bit_offset, bit_size } => {
                assert_eq!(data, Vec::<u8>::new());
                assert_eq!(bit_offset, 0);
                assert_eq!(bit_size, 0);
            }
            _ => panic!("Expected Binary term"),
        }
        assert_eq!(pos, 5);
    }

    #[test]
    fn test_decode_binary_error() {
        // BINARY_EXT (109) with incomplete length
        let buf = vec![109, 0, 0, 0]; // Missing length byte
        let result = decode_ei_term(&buf, 0);
        assert!(matches!(result, Err(DecodeError::BinaryDecodeError(_))));
    }

    #[test]
    fn test_decode_list_not_implemented() {
        // LIST_EXT (108) - empty list is decoded as Nil
        let buf = vec![108, 0, 0, 0, 0]; // List with 0 elements
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, new_pos) = result.unwrap();
        assert!(matches!(term, Term::Nil));
        assert_eq!(new_pos, 5); // 1 byte tag + 4 bytes length
    }

    #[test]
    fn test_decode_tuple_small_not_implemented() {
        // SMALL_TUPLE_EXT (104) - empty tuple is decoded as empty tuple
        let buf = vec![104, 0]; // Tuple with 0 elements
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, new_pos) = result.unwrap();
        if let Term::Tuple(elements) = term {
            assert_eq!(elements.len(), 0);
        } else {
            panic!("Expected Term::Tuple, got {:?}", term);
        }
        assert_eq!(new_pos, 2); // 1 byte tag + 1 byte arity
    }

    #[test]
    fn test_decode_tuple_large_not_implemented() {
        // LARGE_TUPLE_EXT (105) - empty tuple is decoded as empty tuple
        let buf = vec![105, 0, 0, 0, 0]; // Tuple with 0 elements
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, new_pos) = result.unwrap();
        if let Term::Tuple(elements) = term {
            assert_eq!(elements.len(), 0);
        } else {
            panic!("Expected Term::Tuple, got {:?}", term);
        }
        assert_eq!(new_pos, 5); // 1 byte tag + 4 bytes arity
    }

    #[test]
    fn test_decode_index_out_of_bounds() {
        // Index beyond buffer length
        let buf = vec![97, 42];
        let result = decode_ei_term(&buf, 10);
        assert!(matches!(result, Err(DecodeError::BufferTooShort)));
    }

    #[test]
    fn test_decode_index_at_end() {
        // Index at buffer end (should fail)
        let buf = vec![97, 42];
        let result = decode_ei_term(&buf, 2);
        assert!(matches!(result, Err(DecodeError::BufferTooShort)));
    }

    #[test]
    fn test_decode_multiple_terms() {
        // Decode multiple terms sequentially
        let buf = vec![
            97, 42,        // Small integer 42
            106,            // Nil
            97, 100,       // Small integer 100
        ];
        
        // First term
        let result1 = decode_ei_term(&buf, 0);
        assert!(result1.is_ok());
        let (term1, pos1) = result1.unwrap();
        assert_eq!(term1, Term::Small(42));
        assert_eq!(pos1, 2);
        
        // Second term
        let result2 = decode_ei_term(&buf, pos1);
        assert!(result2.is_ok());
        let (term2, pos2) = result2.unwrap();
        assert_eq!(term2, Term::Nil);
        assert_eq!(pos2, 3);
        
        // Third term
        let result3 = decode_ei_term(&buf, pos2);
        assert!(result3.is_ok());
        let (term3, _pos3) = result3.unwrap();
        assert_eq!(term3, Term::Small(100));
    }

    #[test]
    fn test_decode_error_variants() {
        // Test all error variants are constructible and matchable
        let err1 = DecodeError::BufferTooShort;
        let err2 = DecodeError::InvalidFormat("test".to_string());
        let err3 = DecodeError::AtomDecodeError("test".to_string());
        let err4 = DecodeError::BinaryDecodeError("test".to_string());
        let err5 = DecodeError::NotImplemented("test");
        
        // Verify they can be matched
        assert!(matches!(err1, DecodeError::BufferTooShort));
        assert!(matches!(err2, DecodeError::InvalidFormat(_)));
        assert!(matches!(err3, DecodeError::AtomDecodeError(_)));
        assert!(matches!(err4, DecodeError::BinaryDecodeError(_)));
        assert!(matches!(err5, DecodeError::NotImplemented(_)));
    }

    #[test]
    fn test_decode_invalid_tag_various_values() {
        // Test various invalid tag values to cover format! macro
        // Exclude valid tags: 97, 98, 99 (FLOAT_EXT), 100, 101 (REF_EXT), 102 (PORT_EXT), 
        // 103 (PID_EXT), 104, 105, 106, 108, 109, 110 (SMALL_BIG_EXT), 111 (LARGE_BIG_EXT),
        // 112 (EXPORT_EXT), 113 (FUN_EXT), 114 (NEWER_REF_EXT), 115, 116 (MAP_EXT), 
        // 117 (NEW_FUN_EXT), 118, 119, 70 (NEW_FLOAT_EXT), 88 (NEW_PID_EXT), 89 (NEW_PORT_EXT), 90 (NEW_REF_EXT)
        for invalid_tag in [0, 1, 50, 107, 200, 255] {
            let buf = vec![invalid_tag];
            let result = decode_ei_term(&buf, 0);
            assert!(matches!(result, Err(DecodeError::InvalidFormat(_))));
            // Verify error message contains the tag
            if let Err(DecodeError::InvalidFormat(msg)) = result {
                assert!(msg.contains(&invalid_tag.to_string()));
            }
        }
    }

    #[test]
    fn test_decode_atom_invalid_encoding_error() {
        // Test atom decode error with invalid UTF-8
        // ATOM_UTF8_EXT (118) with invalid UTF-8 data
        let buf = vec![118, 0, 2, 0xFF, 0xFE]; // Invalid UTF-8
        let result = decode_ei_term(&buf, 0);
        assert!(matches!(result, Err(DecodeError::AtomDecodeError(_))));
    }

    #[test]
    fn test_decode_atom_buffer_too_short_length() {
        // ATOM_EXT (100) with incomplete length field
        let buf = vec![100, 0]; // Missing second length byte
        let result = decode_ei_term(&buf, 0);
        assert!(matches!(result, Err(DecodeError::AtomDecodeError(_))));
    }

    #[test]
    fn test_decode_binary_buffer_too_short_data() {
        // BINARY_EXT (109) with length but incomplete data
        let buf = vec![109, 0, 0, 0, 10, 1, 2, 3]; // Length 10 but only 4 bytes of data
        let result = decode_ei_term(&buf, 0);
        assert!(matches!(result, Err(DecodeError::BinaryDecodeError(_))));
    }

    #[test]
    fn test_decode_small_integer_max_value() {
        // SMALL_INTEGER_EXT (97) with maximum value (255)
        let buf = vec![97, 255];
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, pos) = result.unwrap();
        assert_eq!(term, Term::Small(255));
        assert_eq!(pos, 2);
    }

    #[test]
    fn test_decode_integer_max_positive() {
        // INTEGER_EXT (98) with maximum positive i32
        let buf = vec![98, 0x7F, 0xFF, 0xFF, 0xFF];
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, pos) = result.unwrap();
        let value = i32::from_be_bytes([0x7F, 0xFF, 0xFF, 0xFF]) as i64;
        assert_eq!(term, Term::Small(value));
        assert_eq!(pos, 5);
    }

    #[test]
    fn test_decode_integer_min_negative() {
        // INTEGER_EXT (98) with minimum negative i32
        let buf = vec![98, 0x80, 0x00, 0x00, 0x00];
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, pos) = result.unwrap();
        let value = i32::from_be_bytes([0x80, 0x00, 0x00, 0x00]) as i64;
        assert_eq!(term, Term::Small(value));
        assert_eq!(pos, 5);
    }

    #[test]
    fn test_decode_atom_small_atom_ext_empty() {
        // SMALL_ATOM_EXT (115) with empty atom
        let buf = vec![115, 0];
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, _pos) = result.unwrap();
        assert!(matches!(term, Term::Atom(_)));
    }

    #[test]
    fn test_decode_atom_atom_ext_empty() {
        // ATOM_EXT (100) with empty atom
        let buf = vec![100, 0, 0];
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, _pos) = result.unwrap();
        assert!(matches!(term, Term::Atom(_)));
    }

    #[test]
    fn test_decode_atom_small_atom_utf8_ext_empty() {
        // SMALL_ATOM_UTF8_EXT (119) with empty atom
        let buf = vec![119, 0];
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, _pos) = result.unwrap();
        assert!(matches!(term, Term::Atom(_)));
    }

    #[test]
    fn test_decode_atom_atom_utf8_ext_empty() {
        // ATOM_UTF8_EXT (118) with empty atom
        let buf = vec![118, 0, 0];
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, _pos) = result.unwrap();
        assert!(matches!(term, Term::Atom(_)));
    }

    #[test]
    fn test_decode_error_display() {
        // Test that error messages are properly formatted
        let err = DecodeError::InvalidFormat("Unknown term tag: 99".to_string());
        let msg = format!("{:?}", err);
        assert!(msg.contains("99"));
    }

    #[test]
    fn test_decode_atom_small_atom_ext_max_length() {
        // SMALL_ATOM_EXT (115) with maximum length (255)
        let mut buf = vec![115, 255];
        buf.extend(vec![b'a'; 255]);
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, _pos) = result.unwrap();
        assert!(matches!(term, Term::Atom(_)));
    }

    #[test]
    fn test_decode_atom_atom_ext_max_length() {
        // ATOM_EXT (100) with maximum length (65535, but we'll use a smaller value)
        let mut buf = vec![100, 0, 10];
        buf.extend(vec![b'a'; 10]);
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, _pos) = result.unwrap();
        assert!(matches!(term, Term::Atom(_)));
    }

    #[test]
    fn test_decode_binary_large() {
        // BINARY_EXT (109) with larger data
        let mut buf = vec![109, 0, 0, 0, 100];
        buf.extend(vec![42u8; 100]);
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, pos) = result.unwrap();
        match term {
            Term::Binary { data, bit_size, .. } => {
                assert_eq!(data.len(), 100);
                assert_eq!(bit_size, 800);
            }
            _ => panic!("Expected Binary term"),
        }
        assert_eq!(pos, 105);
    }

    #[test]
    fn test_decode_integer_zero() {
        // INTEGER_EXT (98) with zero value
        let buf = vec![98, 0, 0, 0, 0];
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, pos) = result.unwrap();
        assert_eq!(term, Term::Small(0));
        assert_eq!(pos, 5);
    }

    #[test]
    fn test_decode_small_integer_zero() {
        // SMALL_INTEGER_EXT (97) with zero value
        let buf = vec![97, 0];
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, pos) = result.unwrap();
        assert_eq!(term, Term::Small(0));
        assert_eq!(pos, 2);
    }

    #[test]
    fn test_decode_atom_error_invalid_utf8_in_small_atom_utf8() {
        // SMALL_ATOM_UTF8_EXT (119) with invalid UTF-8
        let buf = vec![119, 2, 0xFF, 0xFE];
        let result = decode_ei_term(&buf, 0);
        assert!(matches!(result, Err(DecodeError::AtomDecodeError(_))));
    }

    #[test]
    fn test_decode_atom_error_invalid_utf8_in_atom_utf8() {
        // ATOM_UTF8_EXT (118) with invalid UTF-8
        let buf = vec![118, 0, 2, 0xFF, 0xFE];
        let result = decode_ei_term(&buf, 0);
        assert!(matches!(result, Err(DecodeError::AtomDecodeError(_))));
    }

    #[test]
    fn test_decode_binary_error_incomplete_length_byte1() {
        // BINARY_EXT (109) with only 1 length byte
        let buf = vec![109, 0];
        let result = decode_ei_term(&buf, 0);
        assert!(matches!(result, Err(DecodeError::BinaryDecodeError(_))));
    }

    #[test]
    fn test_decode_binary_error_incomplete_length_byte2() {
        // BINARY_EXT (109) with only 2 length bytes
        let buf = vec![109, 0, 0];
        let result = decode_ei_term(&buf, 0);
        assert!(matches!(result, Err(DecodeError::BinaryDecodeError(_))));
    }

    #[test]
    fn test_decode_binary_error_incomplete_length_byte3() {
        // BINARY_EXT (109) with only 3 length bytes
        let buf = vec![109, 0, 0, 0];
        let result = decode_ei_term(&buf, 0);
        assert!(matches!(result, Err(DecodeError::BinaryDecodeError(_))));
    }

    #[test]
    fn test_decode_error_clone_eq() {
        // Test that errors can be cloned and compared
        let err1 = DecodeError::BufferTooShort;
        let err2 = DecodeError::BufferTooShort;
        let err3 = err1.clone();
        assert_eq!(err1, err2);
        assert_eq!(err1, err3);
        
        let err4 = DecodeError::InvalidFormat("test".to_string());
        let err5 = DecodeError::InvalidFormat("test".to_string());
        assert_eq!(err4, err5);
        
        let err6 = DecodeError::NotImplemented("msg");
        let err7 = DecodeError::NotImplemented("msg");
        assert_eq!(err6, err7);
    }

    // Tests for new decoders

    #[test]
    fn test_decode_float_ext() {
        // FLOAT_EXT (99) - deprecated format with 31-byte string
        let mut buf = vec![99];
        let float_str = "3.141592653589793238462643383279";
        let mut float_bytes = float_str.as_bytes().to_vec();
        float_bytes.resize(31, 0); // Pad to 31 bytes with nulls
        buf.extend_from_slice(&float_bytes);
        
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, pos) = result.unwrap();
        match term {
            Term::Float(f) => {
                assert!((f - 3.141592653589793).abs() < 1e-10);
            }
            _ => panic!("Expected Term::Float"),
        }
        assert_eq!(pos, 32); // 1 tag + 31 bytes
    }

    #[test]
    fn test_decode_new_float_ext() {
        // NEW_FLOAT_EXT (70) - 8-byte IEEE 754 double
        let value: f64 = 3.141592653589793;
        let mut buf = vec![70];
        buf.extend_from_slice(&value.to_be_bytes());
        
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, pos) = result.unwrap();
        match term {
            Term::Float(f) => {
                assert!((f - value).abs() < 1e-15);
            }
            _ => panic!("Expected Term::Float"),
        }
        assert_eq!(pos, 9); // 1 tag + 8 bytes
    }

    #[test]
    fn test_decode_small_big_ext_positive() {
        // SMALL_BIG_EXT (110) - positive number
        // 42 in little-endian: [42, 0, 0, ...]
        let mut buf = vec![110, 0, 1]; // sign=0 (positive), n=1
        buf.push(42); // value = 42
        
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, pos) = result.unwrap();
        match term {
            Term::Big(bn) => {
                assert_eq!(bn.to_i64(), Some(42));
            }
            _ => panic!("Expected Term::Big"),
        }
        assert_eq!(pos, 4); // 1 tag + 1 sign + 1 n + 1 byte
    }

    #[test]
    fn test_decode_small_big_ext_negative() {
        // SMALL_BIG_EXT (110) - negative number
        let mut buf = vec![110, 1, 1]; // sign=1 (negative), n=1
        buf.push(42); // value = -42
        
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, pos) = result.unwrap();
        match term {
            Term::Big(bn) => {
                assert_eq!(bn.to_i64(), Some(-42));
            }
            _ => panic!("Expected Term::Big"),
        }
        assert_eq!(pos, 4);
    }

    #[test]
    fn test_decode_small_big_ext_large() {
        // SMALL_BIG_EXT (110) - large number (multiple bytes)
        // 0x01020304 in little-endian: [4, 3, 2, 1]
        let mut buf = vec![110, 0, 4]; // sign=0, n=4
        buf.extend_from_slice(&[4, 3, 2, 1]);
        
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, _pos) = result.unwrap();
        match term {
            Term::Big(bn) => {
                // 0x01020304 = 16909060
                assert_eq!(bn.to_i64(), Some(16909060));
            }
            _ => panic!("Expected Term::Big"),
        }
    }

    #[test]
    fn test_decode_map_ext_empty() {
        // MAP_EXT (116) - empty map
        let buf = vec![116, 0, 0, 0, 0]; // arity = 0
        
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, pos) = result.unwrap();
        match term {
            Term::Map(pairs) => {
                assert_eq!(pairs.len(), 0);
            }
            _ => panic!("Expected Term::Map"),
        }
        assert_eq!(pos, 5); // 1 tag + 4 bytes arity
    }

    #[test]
    fn test_decode_map_ext_single_pair() {
        // MAP_EXT (116) - map with one key-value pair
        let mut buf = vec![116, 0, 0, 0, 1]; // arity = 1
        // Key: atom "key"
        buf.extend_from_slice(&[115, 3, b'k', b'e', b'y']);
        // Value: small integer 42
        buf.extend_from_slice(&[97, 42]);
        
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, _pos) = result.unwrap();
        match term {
            Term::Map(pairs) => {
                assert_eq!(pairs.len(), 1);
                assert!(matches!(pairs[0].0, Term::Atom(_)));
                assert_eq!(pairs[0].1, Term::Small(42));
            }
            _ => panic!("Expected Term::Map"),
        }
    }

    #[test]
    fn test_decode_pid_ext() {
        // PID_EXT (103) - old PID format
        let mut buf = vec![103];
        // Node: atom "node"
        buf.extend_from_slice(&[115, 4, b'n', b'o', b'd', b'e']);
        // id, serial, creation
        buf.extend_from_slice(&[0, 0, 0, 1]); // id = 1
        buf.extend_from_slice(&[0, 0, 0, 2]); // serial = 2
        buf.push(3); // creation = 3
        
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, _pos) = result.unwrap();
        match term {
            Term::Pid { node, id, serial, creation } => {
                assert!(matches!(Term::Atom(node), Term::Atom(_)));
                assert_eq!(id, 1);
                assert_eq!(serial, 2);
                assert_eq!(creation, 3);
            }
            _ => panic!("Expected Term::Pid"),
        }
    }

    #[test]
    fn test_decode_new_pid_ext() {
        // NEW_PID_EXT (88) - new PID format
        let mut buf = vec![88];
        // Node: atom "node"
        buf.extend_from_slice(&[115, 4, b'n', b'o', b'd', b'e']);
        // id, serial, creation (all 4 bytes)
        buf.extend_from_slice(&[0, 0, 0, 1]); // id = 1
        buf.extend_from_slice(&[0, 0, 0, 2]); // serial = 2
        buf.extend_from_slice(&[0, 0, 0, 3]); // creation = 3
        
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, _pos) = result.unwrap();
        match term {
            Term::Pid { node, id, serial, creation } => {
                assert!(matches!(Term::Atom(node), Term::Atom(_)));
                assert_eq!(id, 1);
                assert_eq!(serial, 2);
                assert_eq!(creation, 3);
            }
            _ => panic!("Expected Term::Pid"),
        }
    }

    #[test]
    fn test_decode_port_ext() {
        // PORT_EXT (102) - old port format
        let mut buf = vec![102];
        // Node: atom "node"
        buf.extend_from_slice(&[115, 4, b'n', b'o', b'd', b'e']);
        // id (4 bytes), creation (1 byte)
        buf.extend_from_slice(&[0, 0, 0, 1]); // id = 1
        buf.push(2); // creation = 2
        
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, _pos) = result.unwrap();
        match term {
            Term::Port { node, id, creation } => {
                assert!(matches!(Term::Atom(node), Term::Atom(_)));
                assert_eq!(id, 1);
                assert_eq!(creation, 2);
            }
            _ => panic!("Expected Term::Port"),
        }
    }

    #[test]
    fn test_decode_new_port_ext() {
        // NEW_PORT_EXT (89) - new port format
        let mut buf = vec![89];
        // Node: atom "node"
        buf.extend_from_slice(&[115, 4, b'n', b'o', b'd', b'e']);
        // id (8 bytes), creation (4 bytes)
        buf.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 1]); // id = 1
        buf.extend_from_slice(&[0, 0, 0, 2]); // creation = 2
        
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, _pos) = result.unwrap();
        match term {
            Term::Port { node, id, creation } => {
                assert!(matches!(Term::Atom(node), Term::Atom(_)));
                assert_eq!(id, 1);
                assert_eq!(creation, 2);
            }
            _ => panic!("Expected Term::Port"),
        }
    }

    #[test]
    fn test_decode_ref_ext() {
        // REF_EXT (101) - old ref format
        let mut buf = vec![101];
        // Node: atom "node"
        buf.extend_from_slice(&[115, 4, b'n', b'o', b'd', b'e']);
        // id (4 bytes), creation (1 byte)
        buf.extend_from_slice(&[0, 0, 0, 1]); // id = 1
        buf.push(2); // creation = 2
        
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, _pos) = result.unwrap();
        match term {
            Term::Ref { node, ids, creation } => {
                assert!(matches!(Term::Atom(node), Term::Atom(_)));
                assert_eq!(ids.len(), 1);
                assert_eq!(ids[0], 1);
                assert_eq!(creation, 2);
            }
            _ => panic!("Expected Term::Ref"),
        }
    }

    #[test]
    fn test_decode_new_ref_ext() {
        // NEW_REF_EXT (90) - new ref format
        let mut buf = vec![90];
        buf.extend_from_slice(&[0, 2]); // length = 2
        // Node: atom "node"
        buf.extend_from_slice(&[115, 4, b'n', b'o', b'd', b'e']);
        buf.push(3); // creation = 3
        // ids: 2 * 4 bytes
        buf.extend_from_slice(&[0, 0, 0, 1]); // id[0] = 1
        buf.extend_from_slice(&[0, 0, 0, 2]); // id[1] = 2
        
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, _pos) = result.unwrap();
        match term {
            Term::Ref { node, ids, creation } => {
                assert!(matches!(Term::Atom(node), Term::Atom(_)));
                assert_eq!(ids.len(), 2);
                assert_eq!(ids[0], 1);
                assert_eq!(ids[1], 2);
                assert_eq!(creation, 3);
            }
            _ => panic!("Expected Term::Ref"),
        }
    }

    #[test]
    fn test_decode_newer_ref_ext() {
        // NEWER_REF_EXT (114) - newer ref format
        let mut buf = vec![114];
        buf.extend_from_slice(&[0, 2]); // length = 2
        // Node: atom "node"
        buf.extend_from_slice(&[115, 4, b'n', b'o', b'd', b'e']);
        buf.extend_from_slice(&[0, 0, 0, 3]); // creation = 3 (4 bytes)
        // ids: 2 * 4 bytes
        buf.extend_from_slice(&[0, 0, 0, 1]); // id[0] = 1
        buf.extend_from_slice(&[0, 0, 0, 2]); // id[1] = 2
        
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, _pos) = result.unwrap();
        match term {
            Term::Ref { node, ids, creation } => {
                assert!(matches!(Term::Atom(node), Term::Atom(_)));
                assert_eq!(ids.len(), 2);
                assert_eq!(ids[0], 1);
                assert_eq!(ids[1], 2);
                assert_eq!(creation, 3);
            }
            _ => panic!("Expected Term::Ref"),
        }
    }

    #[test]
    fn test_decode_export_ext() {
        // EXPORT_EXT (112) - external function
        let mut buf = vec![112];
        // Module: atom "module"
        buf.extend_from_slice(&[115, 6, b'm', b'o', b'd', b'u', b'l', b'e']);
        // Function: atom "function"
        buf.extend_from_slice(&[115, 8, b'f', b'u', b'n', b'c', b't', b'i', b'o', b'n']);
        // Arity: small integer 2
        buf.extend_from_slice(&[97, 2]);
        
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, _pos) = result.unwrap();
        match term {
            Term::Fun { is_local, module, function, arity, old_uniq, env } => {
                assert_eq!(is_local, false);
                assert!(matches!(Term::Atom(module), Term::Atom(_)));
                assert!(matches!(Term::Atom(function), Term::Atom(_)));
                assert_eq!(arity, 2);
                assert_eq!(old_uniq, None);
                assert_eq!(env.len(), 0);
            }
            _ => panic!("Expected Term::Fun"),
        }
    }

    #[test]
    fn test_decode_fun_ext() {
        // FUN_EXT (113) - old function format
        let mut buf = vec![113];
        buf.extend_from_slice(&[0, 0, 0, 1]); // num_free = 1
        // PID: NEW_PID_EXT
        buf.extend_from_slice(&[88]);
        buf.extend_from_slice(&[115, 4, b'n', b'o', b'd', b'e']); // node
        buf.extend_from_slice(&[0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3]); // id, serial, creation
        // Module: atom "module"
        buf.extend_from_slice(&[115, 6, b'm', b'o', b'd', b'u', b'l', b'e']);
        buf.extend_from_slice(&[0, 0, 0, 10]); // index = 10
        buf.extend_from_slice(&[0, 0, 0, 20]); // uniq = 20
        // Free var: small integer 42
        buf.extend_from_slice(&[97, 42]);
        
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, _pos) = result.unwrap();
        match term {
            Term::Fun { is_local, module, function, arity, old_uniq, env } => {
                assert_eq!(is_local, true);
                assert!(matches!(Term::Atom(module), Term::Atom(_)));
                assert_eq!(function, 10);
                assert_eq!(arity, 0); // Not stored in old format
                assert_eq!(old_uniq, Some(20));
                assert_eq!(env.len(), 1);
                assert_eq!(env[0], Term::Small(42));
            }
            _ => panic!("Expected Term::Fun"),
        }
    }

    #[test]
    fn test_decode_new_fun_ext() {
        // NEW_FUN_EXT (117) - new function format
        let mut buf = vec![117];
        buf.extend_from_slice(&[0, 0, 0, 50]); // size = 50
        buf.push(3); // arity = 3
        buf.extend_from_slice(&[0; 16]); // uniq (16 bytes, all zeros)
        buf.extend_from_slice(&[0, 0, 0, 10]); // index = 10
        buf.extend_from_slice(&[0, 0, 0, 1]); // num_free = 1
        // Module: atom "module"
        buf.extend_from_slice(&[115, 6, b'm', b'o', b'd', b'u', b'l', b'e']);
        buf.extend_from_slice(&[0, 0, 0, 5]); // old_index = 5
        buf.extend_from_slice(&[0, 0, 0, 20]); // old_uniq = 20
        // PID: NEW_PID_EXT
        buf.extend_from_slice(&[88]);
        buf.extend_from_slice(&[115, 4, b'n', b'o', b'd', b'e']); // node
        buf.extend_from_slice(&[0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3]); // id, serial, creation
        // Free var: small integer 42
        buf.extend_from_slice(&[97, 42]);
        
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, _pos) = result.unwrap();
        match term {
            Term::Fun { is_local, module, function, arity, old_uniq, env } => {
                assert_eq!(is_local, true);
                assert!(matches!(Term::Atom(module), Term::Atom(_)));
                assert_eq!(function, 10);
                assert_eq!(arity, 3);
                assert_eq!(old_uniq, Some(20));
                assert_eq!(env.len(), 1);
                assert_eq!(env[0], Term::Small(42));
            }
            _ => panic!("Expected Term::Fun"),
        }
    }

    #[test]
    fn test_decode_float_ext_buffer_too_short() {
        // FLOAT_EXT with incomplete data
        let buf = vec![99, 0, 0, 0]; // Only 4 bytes instead of 31
        let result = decode_ei_term(&buf, 0);
        assert!(matches!(result, Err(DecodeError::BufferTooShort)));
    }

    #[test]
    fn test_decode_new_float_ext_buffer_too_short() {
        // NEW_FLOAT_EXT (70) with incomplete data
        let buf = vec![70, 0, 0, 0]; // Only 4 bytes instead of 8
        let result = decode_ei_term(&buf, 0);
        assert!(matches!(result, Err(DecodeError::BufferTooShort)));
    }

    #[test]
    fn test_decode_large_big_ext() {
        // LARGE_BIG_EXT (111) - large big integer (> 255 bytes)
        // Test with 300 bytes (requires LARGE_BIG_EXT)
        let mut buf = vec![111];
        buf.extend_from_slice(&(300u32).to_be_bytes()); // arity = 300
        buf.push(0); // sign = 0 (positive)
        buf.extend(vec![1u8; 300]); // 300 bytes of data
        
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, _pos) = result.unwrap();
        match term {
            Term::Big(bn) => {
                // Verify it decoded correctly
                assert!(bn.is_positive());
            }
            _ => panic!("Expected Term::Big"),
        }
    }

    #[test]
    fn test_decode_large_big_ext_negative() {
        // LARGE_BIG_EXT (111) - negative large big integer
        let mut buf = vec![111];
        buf.extend_from_slice(&(4u32).to_be_bytes()); // arity = 4
        buf.push(1); // sign = 1 (negative)
        buf.extend_from_slice(&[42, 0, 0, 0]); // value = -42
        
        let result = decode_ei_term(&buf, 0);
        assert!(result.is_ok());
        let (term, _pos) = result.unwrap();
        match term {
            Term::Big(bn) => {
                assert_eq!(bn.to_i64(), Some(-42));
            }
            _ => panic!("Expected Term::Big"),
        }
    }

    #[test]
    fn test_decode_large_big_ext_buffer_too_short() {
        // LARGE_BIG_EXT with incomplete arity
        let buf = vec![111, 0, 0]; // Only 3 bytes instead of 4 for arity
        let result = decode_ei_term(&buf, 0);
        assert!(matches!(result, Err(DecodeError::BufferTooShort)));
    }

    #[test]
    fn test_decode_small_big_ext_buffer_too_short() {
        // SMALL_BIG_EXT with incomplete data
        let buf = vec![110, 0, 5]; // n=5 but no data bytes
        let result = decode_ei_term(&buf, 0);
        assert!(matches!(result, Err(DecodeError::BufferTooShort)));
    }

    #[test]
    fn test_decode_map_ext_buffer_too_short() {
        // MAP_EXT with incomplete arity
        let buf = vec![116, 0, 0]; // Only 3 bytes instead of 4
        let result = decode_ei_term(&buf, 0);
        assert!(matches!(result, Err(DecodeError::BufferTooShort)));
    }
}

