//! Decode Term Module
//!
//! Provides functionality to decode EI-encoded terms.
//! Based on lib/erl_interface/src/misc/ei_decode_term.c

use entities_data_handling::term_hashing::Term;

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
        for invalid_tag in [0, 1, 50, 99, 101, 103, 107, 110, 200, 255] {
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
}

