//! Size Calculation Module
//!
//! Provides size calculation functions for external term format encoding.
//! Based on erts_encode_ext_size() and encode_size_struct_int() from external.c

use entities_data_handling::term_hashing::Term;
use entities_data_handling::atom::AtomTable;
use std::collections::HashSet;

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
            // Count list length by traversing with cycle detection
            let mut length = 0;
            let mut current = head.as_ref();
            let mut visited = std::collections::HashSet::new();
            let mut max_depth = 10000; // Prevent infinite loops
            
            loop {
                if max_depth == 0 {
                    return Err(SizeCalculationError::InvalidTerm("List too deep or circular".to_string()));
                }
                max_depth -= 1;
                
                // Use pointer address to detect cycles
                let ptr = current as *const Term as usize;
                if visited.contains(&ptr) {
                    // Circular list detected
                    return Err(SizeCalculationError::InvalidTerm("Circular list detected".to_string()));
                }
                visited.insert(ptr);
                
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
            
            // Calculate size for each element with cycle detection
            let mut elements_size = 0;
            let mut current = head.as_ref();
            let mut visited = std::collections::HashSet::new();
            let mut max_depth = 10000; // Prevent infinite loops
            
            loop {
                if max_depth == 0 {
                    return Err(SizeCalculationError::InvalidTerm("List too deep or circular".to_string()));
                }
                max_depth -= 1;
                
                // Use pointer address to detect cycles
                let ptr = current as *const Term as usize;
                if visited.contains(&ptr) {
                    // Circular list detected
                    return Err(SizeCalculationError::InvalidTerm("Circular list detected".to_string()));
                }
                visited.insert(ptr);
                
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
    use entities_data_handling::atom::AtomEncoding;
    
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
    
    #[test]
    fn test_encode_size_struct_int_small_integer_range() {
        // Test value < 256 (SMALL_INTEGER_EXT)
        let term = Term::Small(255);
        let size = encode_size_struct_int(&term, None).unwrap();
        assert_eq!(size, 2); // 1 (tag) + 1 (value)
        
        // Test value = 256 (should use INTEGER_EXT)
        let term2 = Term::Small(256);
        let size2 = encode_size_struct_int(&term2, None).unwrap();
        assert_eq!(size2, 5); // 1 (tag) + 4 (value)
    }
    
    #[test]
    fn test_encode_size_struct_int_integer_ext_range() {
        // Test value in i32 range (INTEGER_EXT)
        let term = Term::Small(i32::MAX as i64);
        let size = encode_size_struct_int(&term, None).unwrap();
        assert_eq!(size, 5); // 1 (tag) + 4 (value)
        
        let term2 = Term::Small(i32::MIN as i64);
        let size2 = encode_size_struct_int(&term2, None).unwrap();
        assert_eq!(size2, 5); // 1 (tag) + 4 (value)
    }
    
    #[test]
    fn test_encode_size_struct_int_big_integer() {
        // Test value outside i32 range (big integer)
        // Use a value that's larger than i32::MAX but won't cause overflow in abs()
        let term = Term::Small(i32::MAX as i64 + 1);
        let size = encode_size_struct_int(&term, None).unwrap();
        // Should use SMALL_BIG_EXT or LARGE_BIG_EXT
        assert!(size > 5);
        
        // Test with a large positive value
        let term2 = Term::Small(i64::MAX);
        let size2 = encode_size_struct_int(&term2, None).unwrap();
        assert!(size2 > 5);
    }
    
    #[test]
    fn test_encode_size_struct_int_big_integer_zero() {
        // Test zero as big integer (edge case)
        let term = Term::Small(0);
        let size = encode_size_struct_int(&term, None).unwrap();
        assert_eq!(size, 2); // Should be SMALL_INTEGER_EXT
    }
    
    #[test]
    fn test_encode_size_struct_int_atom_with_table() {
        let mut atom_table = AtomTable::new(100);
        let atom_index = atom_table.put_index(b"test", AtomEncoding::SevenBitAscii, false).unwrap();
        let term = Term::Atom(atom_index as u32);
        let size = encode_size_struct_int(&term, Some(&atom_table)).unwrap();
        // SMALL_ATOM_EXT: 1 (tag) + 1 (length) + 4 (bytes)
        assert_eq!(size, 6);
    }
    
    #[test]
    fn test_encode_size_struct_int_atom_large() {
        let mut atom_table = AtomTable::new(100);
        // Create a large atom name (255 bytes, which is MAX_ATOM_CHARACTERS)
        // This will use SMALL_ATOM_EXT since name_len <= 255
        let large_name = "a".repeat(255);
        let atom_index = atom_table.put_index(large_name.as_bytes(), AtomEncoding::SevenBitAscii, false).unwrap();
        let term = Term::Atom(atom_index as u32);
        let size = encode_size_struct_int(&term, Some(&atom_table)).unwrap();
        // SMALL_ATOM_EXT: 1 (tag) + 1 (length) + 255 (bytes)
        assert_eq!(size, 257);
    }
    
    #[test]
    fn test_encode_size_struct_int_atom_without_table() {
        let term = Term::Atom(42);
        let size = encode_size_struct_int(&term, None).unwrap();
        // Should use estimate: 1 (tag) + 1 (length) + 10 (estimate)
        assert_eq!(size, 12);
    }
    
    #[test]
    fn test_encode_size_struct_int_atom_not_found() {
        let mut atom_table = AtomTable::new(100);
        let term = Term::Atom(999); // Not in table
        let size = encode_size_struct_int(&term, Some(&atom_table)).unwrap();
        // Should use estimate
        assert_eq!(size, 12);
    }
    
    #[test]
    fn test_encode_size_struct_int_tuple_empty() {
        let term = Term::Tuple(vec![]);
        let size = encode_size_struct_int(&term, None).unwrap();
        // SMALL_TUPLE_EXT: 1 (tag) + 1 (arity) = 2
        assert_eq!(size, 2);
    }
    
    #[test]
    fn test_encode_size_struct_int_tuple_large() {
        // Create tuple with arity > 255 (LARGE_TUPLE_EXT)
        let elements: Vec<Term> = (0..256).map(|i| Term::Small(i)).collect();
        let term = Term::Tuple(elements);
        let size = encode_size_struct_int(&term, None).unwrap();
        // LARGE_TUPLE_EXT: 1 (tag) + 4 (arity) + elements
        assert!(size > 5);
    }
    
    #[test]
    fn test_encode_size_struct_int_tuple_nested() {
        let term = Term::Tuple(vec![
            Term::Small(1),
            Term::Tuple(vec![Term::Small(2), Term::Small(3)]),
        ]);
        let size = encode_size_struct_int(&term, None).unwrap();
        // Outer tuple: 1 (tag) + 1 (arity) + 2 (elements)
        // Inner tuple: 1 (tag) + 1 (arity) + 4 (elements)
        assert!(size > 5);
    }
    
    #[test]
    fn test_encode_size_struct_int_list() {
        let term = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::Nil),
        };
        let size = encode_size_struct_int(&term, None).unwrap();
        // LIST_EXT: 1 (tag) + 4 (length) + 4 (tail) + element size + 1 (NIL)
        assert!(size > 10);
    }
    
    #[test]
    fn test_encode_size_struct_int_list_multiple_elements() {
        let term = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::List {
                head: Box::new(Term::Small(2)),
                tail: Box::new(Term::Nil),
            }),
        };
        let size = encode_size_struct_int(&term, None).unwrap();
        assert!(size > 10);
    }
    
    #[test]
    fn test_encode_size_struct_int_list_improper() {
        let term = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::Small(2)),
        };
        let size = encode_size_struct_int(&term, None).unwrap();
        assert!(size > 10);
    }
    
    #[test]
    fn test_encode_size_struct_int_binary() {
        let term = Term::Binary {
            data: vec![1, 2, 3, 4],
            bit_offset: 0,
            bit_size: 32,
        };
        let size = encode_size_struct_int(&term, None).unwrap();
        // BINARY_EXT: 1 (tag) + 4 (length) + 4 (bytes)
        assert_eq!(size, 9);
    }
    
    #[test]
    fn test_encode_size_struct_int_binary_partial_bits() {
        let term = Term::Binary {
            data: vec![1, 2],
            bit_offset: 0,
            bit_size: 12, // 12 bits = 2 bytes (rounded up)
        };
        let size = encode_size_struct_int(&term, None).unwrap();
        // BINARY_EXT: 1 (tag) + 4 (length) + 2 (bytes)
        assert_eq!(size, 7);
    }
    
    #[test]
    fn test_encode_size_struct_int_binary_empty() {
        let term = Term::Binary {
            data: vec![],
            bit_offset: 0,
            bit_size: 0,
        };
        let size = encode_size_struct_int(&term, None).unwrap();
        // BINARY_EXT: 1 (tag) + 4 (length) + 0 (bytes)
        assert_eq!(size, 5);
    }
    
    #[test]
    fn test_encode_size_struct_int_map() {
        let term = Term::Map(vec![
            (Term::Small(1), Term::Small(2)),
            (Term::Small(3), Term::Small(4)),
        ]);
        let size = encode_size_struct_int(&term, None).unwrap();
        // MAP_EXT: 1 (tag) + 4 (size) + key-value pairs
        assert!(size > 5);
    }
    
    #[test]
    fn test_encode_size_struct_int_map_empty() {
        let term = Term::Map(vec![]);
        let size = encode_size_struct_int(&term, None).unwrap();
        // MAP_EXT: 1 (tag) + 4 (size)
        assert_eq!(size, 5);
    }
    
    #[test]
    fn test_encode_size_struct_int_map_nested() {
        let term = Term::Map(vec![
            (Term::Small(1), Term::Tuple(vec![Term::Small(2)])),
        ]);
        let size = encode_size_struct_int(&term, None).unwrap();
        assert!(size > 5);
    }
    
    #[test]
    fn test_encode_size_struct_int_big() {
        use entities_utilities::BigNumber;
        let term = Term::Big(BigNumber::from_i64(123456789));
        let size = encode_size_struct_int(&term, None).unwrap();
        // Estimate: 1 (tag) + 4 (arity) + 1 (sign) + 16 (bytes)
        assert_eq!(size, 22);
    }
    
    #[test]
    fn test_encode_size_struct_int_float() {
        let term = Term::Float(3.14159);
        let size = encode_size_struct_int(&term, None).unwrap();
        // NEW_FLOAT_EXT: 1 (tag) + 8 (bytes)
        assert_eq!(size, 9);
    }
    
    #[test]
    fn test_encode_size_struct_int_unsupported_type() {
        // Test unsupported term types
        let term = Term::Pid {
            node: 0,
            id: 0,
            serial: 0,
            creation: 0,
        };
        let result = encode_size_struct_int(&term, None);
        assert!(result.is_err());
        match result.unwrap_err() {
            SizeCalculationError::InvalidTerm(msg) => {
                assert!(msg.contains("Unsupported term type"));
            }
            _ => panic!("Expected InvalidTerm error"),
        }
    }
    
    #[test]
    fn test_erts_encode_ext_size_atom() {
        let mut atom_table = AtomTable::new(100);
        let atom_index = atom_table.put_index(b"hello", AtomEncoding::SevenBitAscii, false).unwrap();
        let term = Term::Atom(atom_index as u32);
        let size = erts_encode_ext_size(&term, Some(&atom_table)).unwrap();
        // 1 (version) + 1 (tag) + 1 (length) + 5 (bytes)
        assert_eq!(size, 8);
    }
    
    #[test]
    fn test_erts_encode_ext_size_list() {
        let term = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::Nil),
        };
        let size = erts_encode_ext_size(&term, None).unwrap();
        // 1 (version) + list size
        assert!(size > 1);
    }
    
    #[test]
    fn test_erts_encode_ext_size_map() {
        let term = Term::Map(vec![
            (Term::Small(1), Term::Small(2)),
        ]);
        let size = erts_encode_ext_size(&term, None).unwrap();
        // 1 (version) + map size
        assert!(size > 1);
    }
    
    #[test]
    fn test_erts_encode_ext_size_binary() {
        let term = Term::Binary {
            data: vec![1, 2, 3],
            bit_offset: 0,
            bit_size: 24,
        };
        let size = erts_encode_ext_size(&term, None).unwrap();
        // 1 (version) + 1 (tag) + 4 (length) + 3 (bytes)
        assert_eq!(size, 9);
    }
    
    #[test]
    fn test_erts_encode_ext_size_float() {
        let term = Term::Float(3.14);
        let size = erts_encode_ext_size(&term, None).unwrap();
        // 1 (version) + 1 (tag) + 8 (bytes)
        assert_eq!(size, 10);
    }
    
    #[test]
    fn test_erts_encode_ext_size_big() {
        use entities_utilities::BigNumber;
        let term = Term::Big(BigNumber::from_i64(123456789));
        let size = erts_encode_ext_size(&term, None).unwrap();
        // 1 (version) + big integer size
        assert!(size > 1);
    }
    
    #[test]
    fn test_size_calculation_error_debug() {
        let error1 = SizeCalculationError::TermTooLarge;
        let error2 = SizeCalculationError::InvalidTerm("test".to_string());
        let error3 = SizeCalculationError::CalculationFailed("test".to_string());
        
        let debug_str1 = format!("{:?}", error1);
        let debug_str2 = format!("{:?}", error2);
        let debug_str3 = format!("{:?}", error3);
        
        assert!(debug_str1.contains("TermTooLarge"));
        assert!(debug_str2.contains("InvalidTerm"));
        assert!(debug_str3.contains("CalculationFailed"));
    }
    
    #[test]
    fn test_size_calculation_error_clone() {
        let error1 = SizeCalculationError::TermTooLarge;
        let error2 = SizeCalculationError::InvalidTerm("test".to_string());
        let error3 = SizeCalculationError::CalculationFailed("test".to_string());
        
        let cloned1 = error1.clone();
        let cloned2 = error2.clone();
        let cloned3 = error3.clone();
        
        assert_eq!(error1, cloned1);
        assert_eq!(error2, cloned2);
        assert_eq!(error3, cloned3);
    }
    
    #[test]
    fn test_size_calculation_error_partial_eq() {
        let error1 = SizeCalculationError::TermTooLarge;
        let error2 = SizeCalculationError::TermTooLarge;
        let error3 = SizeCalculationError::InvalidTerm("test".to_string());
        let error4 = SizeCalculationError::InvalidTerm("test".to_string());
        let error5 = SizeCalculationError::InvalidTerm("different".to_string());
        
        assert_eq!(error1, error2);
        assert_eq!(error3, error4);
        assert_ne!(error3, error5);
        assert_ne!(error1, error3);
    }
    
    #[test]
    fn test_size_calculation_error_eq() {
        let error1 = SizeCalculationError::TermTooLarge;
        let error2 = SizeCalculationError::TermTooLarge;
        let error3 = SizeCalculationError::InvalidTerm("test".to_string());
        
        assert!(error1 == error2);
        assert!(error1 != error3);
    }
    
    #[test]
    fn test_encode_size_struct_int_negative_small() {
        // Test negative small integer
        let term = Term::Small(-42);
        let size = encode_size_struct_int(&term, None).unwrap();
        // Negative values < 256 still use SMALL_INTEGER_EXT
        // But -42 is negative, so it might use INTEGER_EXT
        // Actually, -42 is in i32 range, so should use INTEGER_EXT
        assert!(size >= 2);
    }
    
    #[test]
    fn test_encode_size_struct_int_zero() {
        let term = Term::Small(0);
        let size = encode_size_struct_int(&term, None).unwrap();
        assert_eq!(size, 2); // SMALL_INTEGER_EXT
    }
    
    #[test]
    fn test_encode_size_struct_int_tuple_single_element() {
        let term = Term::Tuple(vec![Term::Small(1)]);
        let size = encode_size_struct_int(&term, None).unwrap();
        // 1 (tag) + 1 (arity) + 2 (element)
        assert_eq!(size, 4);
    }
    
    #[test]
    fn test_encode_size_struct_int_list_empty() {
        // Empty list is just Nil
        let term = Term::Nil;
        let size = encode_size_struct_int(&term, None).unwrap();
        assert_eq!(size, 1);
    }
    
    #[test]
    fn test_encode_size_struct_int_atom_small_vs_large() {
        let mut atom_table = AtomTable::new(100);
        
        // Small atom (100 bytes)
        let small_name = "a".repeat(100);
        let small_index = atom_table.put_index(small_name.as_bytes(), AtomEncoding::SevenBitAscii, false).unwrap();
        let small_term = Term::Atom(small_index as u32);
        let small_size = encode_size_struct_int(&small_term, Some(&atom_table)).unwrap();
        // SMALL_ATOM_EXT: 1 + 1 + 100
        assert_eq!(small_size, 102);
        
        // Large atom (255 bytes, MAX_ATOM_CHARACTERS)
        let large_name = "a".repeat(255);
        let large_index = atom_table.put_index(large_name.as_bytes(), AtomEncoding::SevenBitAscii, false).unwrap();
        let large_term = Term::Atom(large_index as u32);
        let large_size = encode_size_struct_int(&large_term, Some(&atom_table)).unwrap();
        // SMALL_ATOM_EXT: 1 + 1 + 255 (still uses SMALL_ATOM_EXT since <= 255)
        assert_eq!(large_size, 257);
        assert!(large_size > small_size); // Should be larger than small atom
    }
    
    #[test]
    fn test_encode_size_struct_int_big_integer_small_big_ext() {
        // Test value that uses SMALL_BIG_EXT (bytes <= 255)
        let term = Term::Small(1000000i64); // Large but fits in SMALL_BIG_EXT
        let size = encode_size_struct_int(&term, None).unwrap();
        // Should use SMALL_BIG_EXT: 1 (tag) + 1 (arity) + 1 (sign) + bytes
        assert!(size >= 4);
    }
    
    #[test]
    fn test_encode_size_struct_int_big_integer_large_big_ext() {
        // Test value that uses LARGE_BIG_EXT (bytes > 255)
        // This would require a very large integer
        let term = Term::Small(i64::MAX);
        let size = encode_size_struct_int(&term, None).unwrap();
        // Should calculate bytes needed
        assert!(size > 5);
    }
    
    #[test]
    fn test_erts_encode_ext_size_nested_structures() {
        let term = Term::Tuple(vec![
            Term::Small(1),
            Term::List {
                head: Box::new(Term::Small(2)),
                tail: Box::new(Term::Nil),
            },
        ]);
        let size = erts_encode_ext_size(&term, None).unwrap();
        assert!(size > 1);
    }
    
    #[test]
    fn test_erts_encode_ext_size_unsupported_type() {
        let term = Term::Pid {
            node: 0,
            id: 0,
            serial: 0,
            creation: 0,
        };
        let result = erts_encode_ext_size(&term, None);
        assert!(result.is_err());
    }
}

