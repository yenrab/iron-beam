//! Size Calculation Module
//!
//! Provides size calculation functions for external term format encoding.
//! Based on erts_encode_ext_size() and encode_size_struct_int() from external.c

use entities_data_handling::term_hashing::Term;
use entities_data_handling::atom::AtomTable;

/// Size calculation error types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SizeCalculationError {
    /// Term too large
    TermTooLarge,
    /// Invalid term
    InvalidTerm(String),
    /// Calculation failed
    CalculationFailed(String),
}

/// Calculate the size needed to encode a term in external format
///
/// Based on `erts_encode_ext_size()` from external.c. This function calculates
/// the number of bytes needed to encode a term in external term format.
///
/// # Arguments
/// * `term` - The term to calculate size for
/// * `atom_table` - Optional atom table for looking up atom names
///
/// # Returns
/// * `Ok(usize)` - Size in bytes needed for encoding
/// * `Err(SizeCalculationError)` - Calculation error
pub fn erts_encode_ext_size(term: &Term, atom_table: Option<&AtomTable>) -> Result<usize, SizeCalculationError> {
    // Start with version magic byte (1 byte)
    let mut size = 1;
    
    // Add size for the term itself
    size += encode_size_struct_int(term, atom_table)?;
    
    Ok(size)
}

/// Calculate the size needed to encode a term structure (without version byte)
///
/// Based on `encode_size_struct_int()` from external.c. This function calculates
/// the size needed for the term data itself, excluding the version magic byte.
///
/// # Arguments
/// * `term` - The term to calculate size for
/// * `atom_table` - Optional atom table for looking up atom names
///
/// # Returns
/// * `Ok(usize)` - Size in bytes needed for term encoding
/// * `Err(SizeCalculationError)` - Calculation error
pub fn encode_size_struct_int(term: &Term, atom_table: Option<&AtomTable>) -> Result<usize, SizeCalculationError> {
    match term {
        Term::Nil => {
            // NIL_EXT = 1 byte
            Ok(1)
        }
        Term::Small(value) => {
            // SMALL_INTEGER_EXT = 1 byte tag + 1 byte value (if value < 256)
            // INTEGER_EXT = 1 byte tag + 4 bytes value (if value fits in i32)
            // Otherwise, use big integer encoding
            if *value >= 0 && *value < 256 {
                Ok(2) // SMALL_INTEGER_EXT
            } else if *value >= i32::MIN as i64 && *value <= i32::MAX as i64 {
                Ok(5) // INTEGER_EXT
            } else {
                // Big integer encoding
                // SMALL_BIG_EXT = 1 byte tag + 1 byte arity + 1 byte sign + n bytes
                // LARGE_BIG_EXT = 1 byte tag + 4 bytes arity + 1 byte sign + n bytes
                // Estimate: calculate bytes needed for the value
                let abs_value = value.abs();
                let bytes_needed = if abs_value == 0 {
                    1
                } else {
                    (abs_value.ilog2() / 8 + 1) as usize
                };
                
                if bytes_needed <= 255 {
                    Ok(1 + 1 + 1 + bytes_needed) // SMALL_BIG_EXT
                } else {
                    Ok(1 + 4 + 1 + bytes_needed) // LARGE_BIG_EXT
                }
            }
        }
        Term::Atom(atom_index) => {
            // Try to get atom name from atom table if available
            if let Some(table) = atom_table {
                if let Some(name_bytes) = table.get_name(*atom_index as usize) {
                    let name_len = name_bytes.len();
                    if name_len <= 255 {
                        // SMALL_ATOM_EXT = 1 byte tag + 1 byte length + n bytes
                        Ok(1 + 1 + name_len)
                    } else {
                        // ATOM_EXT = 1 byte tag + 2 bytes length + n bytes
                        Ok(1 + 2 + name_len)
                    }
                } else {
                    // Atom not found, estimate
                    Ok(1 + 1 + 10) // Estimate
                }
            } else {
                // No atom table, estimate
                Ok(1 + 1 + 10) // Estimate
            }
        }
        Term::Tuple(elements) => {
            // Tuple header: SMALL_TUPLE_EXT (1 byte) + 1 byte arity, or
            // LARGE_TUPLE_EXT (1 byte) + 4 bytes arity
            let arity = elements.len();
            let header_size = if arity <= 255 {
                1 + 1 // SMALL_TUPLE_EXT
            } else {
                1 + 4 // LARGE_TUPLE_EXT
            };
            
            // Calculate size for each element
            let mut elements_size = 0;
            for element in elements {
                elements_size += encode_size_struct_int(element, atom_table)?;
            }
            
            Ok(header_size + elements_size)
        }
        Term::List { head, tail } => {
            // Count list length by traversing
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
            
            // List header: LIST_EXT = 1 byte tag + 4 bytes length + 4 bytes tail
            let header_size = 1 + 4 + 4; // tag + length + tail
            
            // Calculate size for each element
            let mut elements_size = 0;
            let mut current = head.as_ref();
            loop {
                match current {
                    Term::Nil => break,
                    Term::List { head: h, tail: t } => {
                        elements_size += encode_size_struct_int(h, atom_table)?;
                        current = t.as_ref();
                    }
                    _ => {
                        elements_size += encode_size_struct_int(current, atom_table)?;
                        break;
                    }
                }
            }
            
            // Add NIL_EXT for tail (1 byte)
            Ok(header_size + elements_size + 1)
        }
        Term::Binary { data, bit_offset: _, bit_size } => {
            // BINARY_EXT = 1 byte tag + 4 bytes length + n bytes
            let byte_size = (bit_size + 7) / 8; // Round up to bytes
            Ok(1 + 4 + byte_size)
        }
        Term::Map(entries) => {
            // MAP_EXT = 1 byte tag + 4 bytes size
            let map_size = entries.len();
            let header_size = 1 + 4;
            
            // Calculate size for each key-value pair
            let mut entries_size = 0;
            for (key, value) in entries {
                entries_size += encode_size_struct_int(key, atom_table)?;
                entries_size += encode_size_struct_int(value, atom_table)?;
            }
            
            Ok(header_size + entries_size)
        }
        Term::Big(_) => {
            // Big integer encoding (estimate)
            // SMALL_BIG_EXT or LARGE_BIG_EXT
            Ok(1 + 4 + 1 + 16) // Estimate: tag + arity + sign + 16 bytes
        }
        Term::Float(_) => {
            // NEW_FLOAT_EXT = 1 byte tag + 8 bytes (IEEE 754 double)
            Ok(1 + 8)
        }
        _ => Err(SizeCalculationError::InvalidTerm(format!("Unsupported term type for size calculation: {:?}", term))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encode_ext_size_nil() {
        let term = Term::Nil;
        let size = erts_encode_ext_size(&term, None).unwrap();
        assert_eq!(size, 2); // 1 (version) + 1 (NIL_EXT)
    }
    
    #[test]
    fn test_encode_ext_size_small_integer() {
        let term = Term::Small(42);
        let size = erts_encode_ext_size(&term, None).unwrap();
        assert_eq!(size, 3); // 1 (version) + 1 (tag) + 1 (value)
    }
    
    #[test]
    fn test_encode_ext_size_tuple() {
        let term = Term::Tuple(vec![
            Term::Small(1),
            Term::Small(2),
            Term::Small(3),
        ]);
        let size = erts_encode_ext_size(&term, None).unwrap();
        // 1 (version) + 1 (tag) + 1 (arity) + 3 * 2 (elements)
        assert_eq!(size, 1 + 1 + 1 + 6);
    }
    
    #[test]
    fn test_encode_size_struct_int_nil() {
        let term = Term::Nil;
        let size = encode_size_struct_int(&term, None).unwrap();
        assert_eq!(size, 1); // NIL_EXT
    }
    
    #[test]
    fn test_encode_size_struct_int_small_integer() {
        let term = Term::Small(42);
        let size = encode_size_struct_int(&term, None).unwrap();
        assert_eq!(size, 2); // 1 (tag) + 1 (value)
    }
}

