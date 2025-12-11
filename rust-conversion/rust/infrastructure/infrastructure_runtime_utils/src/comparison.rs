//! Comparison Module
//!
//! Provides term comparison functions.
//! Based on eq() and erts_cmp() from utils.c

use entities_data_handling::term_hashing::Term;
use entities_utilities::{BigNumber, BigRational};

/// Comparison error types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComparisonError {
    /// Comparison failed
    ComparisonFailed(String),
    /// Invalid term
    InvalidTerm(String),
}

/// Compare two terms for equality
///
/// Based on `eq()` from utils.c. This function performs deep equality comparison
/// of two Erlang terms, handling all term types including nested structures.
///
/// # Arguments
/// * `a` - First term
/// * `b` - Second term
///
/// # Returns
/// * `Ok(bool)` - True if terms are equal, false otherwise
/// * `Err(ComparisonError)` - Comparison error
pub fn eq(a: &Term, b: &Term) -> Result<bool, ComparisonError> {
    // Use a stack-based approach for recursive structures (matches C implementation)
    let mut stack: Vec<(Term, Term)> = Vec::new();
    let mut current_a = a.clone();
    let mut current_b = b.clone();
    
    loop {
        // Check if terms are the same (pointer equality or immediate value equality)
        if is_same(&current_a, &current_b) {
            // Pop next pair from stack or return true if stack is empty
            match stack.pop() {
                Some((next_a, next_b)) => {
                    current_a = next_a;
                    current_b = next_b;
                    continue;
                }
                None => return Ok(true),
            }
        }
        
        // Compare based on term type
        match (&current_a, &current_b) {
            // Immediate values
            (Term::Nil, Term::Nil) => {
                match stack.pop() {
                    Some((next_a, next_b)) => {
                        current_a = next_a;
                        current_b = next_b;
                        continue;
                    }
                    None => return Ok(true),
                }
            }
            (Term::Small(a_val), Term::Small(b_val)) => {
                if a_val == b_val {
                    match stack.pop() {
                        Some((next_a, next_b)) => {
                            current_a = next_a;
                            current_b = next_b;
                            continue;
                        }
                        None => return Ok(true),
                    }
                } else {
                    return Ok(false);
                }
            }
            (Term::Atom(a_idx), Term::Atom(b_idx)) => {
                if a_idx == b_idx {
                    match stack.pop() {
                        Some((next_a, next_b)) => {
                            current_a = next_a;
                            current_b = next_b;
                            continue;
                        }
                        None => return Ok(true),
                    }
                } else {
                    return Ok(false);
                }
            }
            (Term::Float(a_val), Term::Float(b_val)) => {
                if a_val == b_val {
                    match stack.pop() {
                        Some((next_a, next_b)) => {
                            current_a = next_a;
                            current_b = next_b;
                            continue;
                        }
                        None => return Ok(true),
                    }
                } else {
                    return Ok(false);
                }
            }
            
            // Lists
            (Term::List { head: a_head, tail: a_tail }, Term::List { head: b_head, tail: b_tail }) => {
                // Compare heads
                if !eq(a_head, b_head)? {
                    return Ok(false);
                }
                // Push tails to stack
                stack.push((*a_tail.clone(), *b_tail.clone()));
                match stack.pop() {
                    Some((next_a, next_b)) => {
                        current_a = next_a;
                        current_b = next_b;
                        continue;
                    }
                    None => return Ok(true),
                }
            }
            
            // Tuples
            (Term::Tuple(a_elements), Term::Tuple(b_elements)) => {
                if a_elements.len() != b_elements.len() {
                    return Ok(false);
                }
                // Compare elements
                for (a_elem, b_elem) in a_elements.iter().zip(b_elements.iter()) {
                    if !eq(a_elem, b_elem)? {
                        return Ok(false);
                    }
                }
                match stack.pop() {
                    Some((next_a, next_b)) => {
                        current_a = next_a;
                        current_b = next_b;
                        continue;
                    }
                    None => return Ok(true),
                }
            }
            
            // Maps
            (Term::Map(a_entries), Term::Map(b_entries)) => {
                if a_entries.len() != b_entries.len() {
                    return Ok(false);
                }
                // Compare entries (order-independent)
                for (a_key, a_val) in a_entries {
                    let mut found = false;
                    for (b_key, b_val) in b_entries {
                        if eq(a_key, b_key)? && eq(a_val, b_val)? {
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        return Ok(false);
                    }
                }
                match stack.pop() {
                    Some((next_a, next_b)) => {
                        current_a = next_a;
                        current_b = next_b;
                        continue;
                    }
                    None => return Ok(true),
                }
            }
            
            // Binaries
            (Term::Binary { data: a_data, bit_offset: a_offset, bit_size: a_size },
             Term::Binary { data: b_data, bit_offset: b_offset, bit_size: b_size }) => {
                if a_size != b_size || a_offset != b_offset {
                    return Ok(false);
                }
                if a_data == b_data {
                    match stack.pop() {
                        Some((next_a, next_b)) => {
                            current_a = next_a;
                            current_b = next_b;
                            continue;
                        }
                        None => return Ok(true),
                    }
                } else {
                    return Ok(false);
                }
            }
            
            // Big integers
        (Term::Big(a_big), Term::Big(b_big)) => {
            // Compare BigNumber using PartialEq
            if a_big.as_integer() == b_big.as_integer() {
                match stack.pop() {
                    Some((next_a, next_b)) => {
                        current_a = next_a;
                        current_b = next_b;
                        continue;
                    }
                    None => return Ok(true),
                }
            } else {
                return Ok(false);
            }
        }
        
        // Rationals
        (Term::Rational(a_rat), Term::Rational(b_rat)) => {
            // Compare BigRational using PartialEq
            if a_rat.numerator() == b_rat.numerator() && a_rat.denominator() == b_rat.denominator() {
                match stack.pop() {
                    Some((next_a, next_b)) => {
                        current_a = next_a;
                        current_b = next_b;
                        continue;
                    }
                    None => return Ok(true),
                }
            } else {
                return Ok(false);
            }
        }
            
            // Different types or incompatible values
            _ => return Ok(false),
        }
    }
}

/// Check if two terms are the same (pointer equality or immediate value equality)
fn is_same(a: &Term, b: &Term) -> bool {
    match (a, b) {
        (Term::Nil, Term::Nil) => true,
        (Term::Small(a_val), Term::Small(b_val)) => a_val == b_val,
        (Term::Atom(a_idx), Term::Atom(b_idx)) => a_idx == b_idx,
        (Term::Float(a_val), Term::Float(b_val)) => a_val == b_val,
        _ => false, // For boxed values, we need deep comparison
    }
}

/// Compare two terms (returns -1, 0, or 1)
///
/// Based on `erts_cmp()` from utils.c. This function performs comparison
/// of two Erlang terms, returning:
/// - -1 if a < b
/// - 0 if a == b
/// - 1 if a > b
///
/// # Arguments
/// * `a` - First term
/// * `b` - Second term
/// * `order` - Comparison order flags (for future use)
///
/// # Returns
/// * `Ok(i32)` - Comparison result (-1, 0, or 1)
/// * `Err(ComparisonError)` - Comparison error
pub fn erts_cmp(a: &Term, b: &Term, _order: i32) -> Result<i32, ComparisonError> {
    // First check equality
    if eq(a, b)? {
        return Ok(0);
    }
    
    // Compare term types first (type ordering)
    let a_type_order = term_type_order(a);
    let b_type_order = term_type_order(b);
    
    if a_type_order != b_type_order {
        return Ok(if a_type_order < b_type_order { -1 } else { 1 });
    }
    
    // Same type, compare values
    match (a, b) {
        (Term::Small(a_val), Term::Small(b_val)) => {
            Ok(if a_val < b_val { -1 } else { 1 })
        }
        (Term::Atom(a_idx), Term::Atom(b_idx)) => {
            Ok(if a_idx < b_idx { -1 } else { 1 })
        }
        (Term::Float(a_val), Term::Float(b_val)) => {
            Ok(if a_val < b_val { -1 } else { 1 })
        }
        (Term::Big(a_big), Term::Big(b_big)) => {
            // Compare BigNumber using integer comparison
            let a_int = a_big.as_integer();
            let b_int = b_big.as_integer();
            Ok(if a_int < b_int { -1 } else { 1 })
        }
        (Term::Rational(a_rat), Term::Rational(b_rat)) => {
            // Compare BigRational by comparing as fractions
            // For simplicity, compare numerators and denominators
            let a_num = a_rat.numerator();
            let b_num = b_rat.numerator();
            let a_den = a_rat.denominator();
            let b_den = b_rat.denominator();
            // Compare as: a_num * b_den vs b_num * a_den
            let a_cross = a_num * b_den;
            let b_cross = b_num * a_den;
            Ok(if a_cross < b_cross { -1 } else { 1 })
        }
        // For complex types, we'd need more sophisticated comparison
        _ => Err(ComparisonError::ComparisonFailed("Complex type comparison not fully implemented".to_string())),
    }
}

/// Get type order for comparison
fn term_type_order(term: &Term) -> u32 {
    match term {
        Term::Nil => 0,
        Term::Small(_) => 1,
        Term::Atom(_) => 2,
        Term::Float(_) => 3,
        Term::Big(_) => 4,
        Term::Rational(_) => 5,
        Term::Binary { .. } => 6,
        Term::List { .. } => 7,
        Term::Tuple(_) => 8,
        Term::Map(_) => 9,
        _ => 10,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_eq_nil() {
        let a = Term::Nil;
        let b = Term::Nil;
        assert!(eq(&a, &b).unwrap());
    }
    
    #[test]
    fn test_eq_small_integer() {
        let a = Term::Small(42);
        let b = Term::Small(42);
        assert!(eq(&a, &b).unwrap());
        
        let c = Term::Small(43);
        assert!(!eq(&a, &c).unwrap());
    }
    
    #[test]
    fn test_eq_tuple() {
        let a = Term::Tuple(vec![Term::Small(1), Term::Small(2)]);
        let b = Term::Tuple(vec![Term::Small(1), Term::Small(2)]);
        assert!(eq(&a, &b).unwrap());
        
        let c = Term::Tuple(vec![Term::Small(1), Term::Small(3)]);
        assert!(!eq(&a, &c).unwrap());
    }
    
    #[test]
    fn test_eq_list() {
        let a = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::Nil),
        };
        let b = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::Nil),
        };
        assert!(eq(&a, &b).unwrap());
    }
    
    #[test]
    fn test_erts_cmp() {
        let a = Term::Small(1);
        let b = Term::Small(2);
        assert_eq!(erts_cmp(&a, &b, 0).unwrap(), -1);
        assert_eq!(erts_cmp(&b, &a, 0).unwrap(), 1);
        assert_eq!(erts_cmp(&a, &a, 0).unwrap(), 0);
    }
    
    #[test]
    fn test_eq_atom() {
        let a = Term::Atom(1);
        let b = Term::Atom(1);
        assert!(eq(&a, &b).unwrap());
        
        let c = Term::Atom(2);
        assert!(!eq(&a, &c).unwrap());
    }
    
    #[test]
    fn test_eq_float() {
        let a = Term::Float(3.14);
        let b = Term::Float(3.14);
        assert!(eq(&a, &b).unwrap());
        
        let c = Term::Float(3.15);
        assert!(!eq(&a, &c).unwrap());
    }
    
    #[test]
    fn test_eq_float_special_values() {
        // Test NaN (NaN != NaN in IEEE 754, but our implementation might handle it differently)
        let a = Term::Float(f64::NAN);
        let b = Term::Float(f64::NAN);
        // NaN comparison behavior depends on implementation
        let result = eq(&a, &b).unwrap();
        // Just verify it doesn't panic
        
        // Test infinity
        let c = Term::Float(f64::INFINITY);
        let d = Term::Float(f64::INFINITY);
        assert!(eq(&c, &d).unwrap());
        
        // Test negative infinity
        let e = Term::Float(f64::NEG_INFINITY);
        let f = Term::Float(f64::NEG_INFINITY);
        assert!(eq(&e, &f).unwrap());
    }
    
    #[test]
    fn test_eq_big_integer() {
        use entities_utilities::BigNumber;
        let a = Term::Big(BigNumber::from_i64(123456789));
        let b = Term::Big(BigNumber::from_i64(123456789));
        assert!(eq(&a, &b).unwrap());
        
        let c = Term::Big(BigNumber::from_i64(123456790));
        assert!(!eq(&a, &c).unwrap());
    }
    
    #[test]
    fn test_eq_rational() {
        use entities_utilities::BigRational;
        let a = Term::Rational(BigRational::from_fraction(1, 2).unwrap());
        let b = Term::Rational(BigRational::from_fraction(1, 2).unwrap());
        assert!(eq(&a, &b).unwrap());
        
        let c = Term::Rational(BigRational::from_fraction(1, 3).unwrap());
        assert!(!eq(&a, &c).unwrap());
    }
    
    #[test]
    fn test_eq_binary() {
        let a = Term::Binary {
            data: vec![1, 2, 3, 4],
            bit_offset: 0,
            bit_size: 32,
        };
        let b = Term::Binary {
            data: vec![1, 2, 3, 4],
            bit_offset: 0,
            bit_size: 32,
        };
        assert!(eq(&a, &b).unwrap());
        
        let c = Term::Binary {
            data: vec![1, 2, 3, 5],
            bit_offset: 0,
            bit_size: 32,
        };
        assert!(!eq(&a, &c).unwrap());
    }
    
    #[test]
    fn test_eq_binary_different_size() {
        let a = Term::Binary {
            data: vec![1, 2, 3, 4],
            bit_offset: 0,
            bit_size: 32,
        };
        let b = Term::Binary {
            data: vec![1, 2, 3, 4],
            bit_offset: 0,
            bit_size: 31,
        };
        assert!(!eq(&a, &b).unwrap());
    }
    
    #[test]
    fn test_eq_binary_different_offset() {
        let a = Term::Binary {
            data: vec![1, 2, 3, 4],
            bit_offset: 0,
            bit_size: 32,
        };
        let b = Term::Binary {
            data: vec![1, 2, 3, 4],
            bit_offset: 1,
            bit_size: 32,
        };
        assert!(!eq(&a, &b).unwrap());
    }
    
    #[test]
    fn test_eq_map() {
        let a = Term::Map(vec![
            (Term::Small(1), Term::Small(2)),
            (Term::Small(3), Term::Small(4)),
        ]);
        let b = Term::Map(vec![
            (Term::Small(1), Term::Small(2)),
            (Term::Small(3), Term::Small(4)),
        ]);
        assert!(eq(&a, &b).unwrap());
    }
    
    #[test]
    fn test_eq_map_different_order() {
        // Maps should be equal regardless of entry order
        let a = Term::Map(vec![
            (Term::Small(1), Term::Small(2)),
            (Term::Small(3), Term::Small(4)),
        ]);
        let b = Term::Map(vec![
            (Term::Small(3), Term::Small(4)),
            (Term::Small(1), Term::Small(2)),
        ]);
        assert!(eq(&a, &b).unwrap());
    }
    
    #[test]
    fn test_eq_map_different_length() {
        let a = Term::Map(vec![
            (Term::Small(1), Term::Small(2)),
        ]);
        let b = Term::Map(vec![
            (Term::Small(1), Term::Small(2)),
            (Term::Small(3), Term::Small(4)),
        ]);
        assert!(!eq(&a, &b).unwrap());
    }
    
    #[test]
    fn test_eq_map_different_value() {
        let a = Term::Map(vec![
            (Term::Small(1), Term::Small(2)),
        ]);
        let b = Term::Map(vec![
            (Term::Small(1), Term::Small(3)),
        ]);
        assert!(!eq(&a, &b).unwrap());
    }
    
    #[test]
    fn test_eq_tuple_different_length() {
        let a = Term::Tuple(vec![Term::Small(1), Term::Small(2)]);
        let b = Term::Tuple(vec![Term::Small(1)]);
        assert!(!eq(&a, &b).unwrap());
    }
    
    #[test]
    fn test_eq_tuple_empty() {
        let a = Term::Tuple(vec![]);
        let b = Term::Tuple(vec![]);
        assert!(eq(&a, &b).unwrap());
    }
    
    #[test]
    fn test_eq_list_multiple_elements() {
        let a = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::List {
                head: Box::new(Term::Small(2)),
                tail: Box::new(Term::Nil),
            }),
        };
        let b = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::List {
                head: Box::new(Term::Small(2)),
                tail: Box::new(Term::Nil),
            }),
        };
        assert!(eq(&a, &b).unwrap());
    }
    
    #[test]
    fn test_eq_list_different_length() {
        let a = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::Nil),
        };
        let b = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::List {
                head: Box::new(Term::Small(2)),
                tail: Box::new(Term::Nil),
            }),
        };
        assert!(!eq(&a, &b).unwrap());
    }
    
    #[test]
    fn test_eq_nested_structures() {
        let a = Term::Tuple(vec![
            Term::Small(1),
            Term::List {
                head: Box::new(Term::Small(2)),
                tail: Box::new(Term::Nil),
            },
        ]);
        let b = Term::Tuple(vec![
            Term::Small(1),
            Term::List {
                head: Box::new(Term::Small(2)),
                tail: Box::new(Term::Nil),
            },
        ]);
        assert!(eq(&a, &b).unwrap());
    }
    
    #[test]
    fn test_eq_different_types() {
        let a = Term::Small(42);
        let b = Term::Atom(42);
        assert!(!eq(&a, &b).unwrap());
        
        let c = Term::Float(42.0);
        assert!(!eq(&a, &c).unwrap());
        
        let d = Term::Nil;
        assert!(!eq(&a, &d).unwrap());
    }
    
    #[test]
    fn test_erts_cmp_atom() {
        let a = Term::Atom(1);
        let b = Term::Atom(2);
        assert_eq!(erts_cmp(&a, &b, 0).unwrap(), -1);
        assert_eq!(erts_cmp(&b, &a, 0).unwrap(), 1);
        assert_eq!(erts_cmp(&a, &a, 0).unwrap(), 0);
    }
    
    #[test]
    fn test_erts_cmp_float() {
        let a = Term::Float(1.0);
        let b = Term::Float(2.0);
        assert_eq!(erts_cmp(&a, &b, 0).unwrap(), -1);
        assert_eq!(erts_cmp(&b, &a, 0).unwrap(), 1);
        assert_eq!(erts_cmp(&a, &a, 0).unwrap(), 0);
    }
    
    #[test]
    fn test_erts_cmp_big_integer() {
        use entities_utilities::BigNumber;
        let a = Term::Big(BigNumber::from_i64(100));
        let b = Term::Big(BigNumber::from_i64(200));
        assert_eq!(erts_cmp(&a, &b, 0).unwrap(), -1);
        assert_eq!(erts_cmp(&b, &a, 0).unwrap(), 1);
        assert_eq!(erts_cmp(&a, &a, 0).unwrap(), 0);
    }
    
    #[test]
    fn test_erts_cmp_rational() {
        use entities_utilities::BigRational;
        let a = Term::Rational(BigRational::from_fraction(1, 2).unwrap());
        let b = Term::Rational(BigRational::from_fraction(2, 3).unwrap());
        // 1/2 < 2/3 (0.5 < 0.666...)
        assert_eq!(erts_cmp(&a, &b, 0).unwrap(), -1);
        assert_eq!(erts_cmp(&b, &a, 0).unwrap(), 1);
        assert_eq!(erts_cmp(&a, &a, 0).unwrap(), 0);
    }
    
    #[test]
    fn test_erts_cmp_type_ordering() {
        // Test that different types are ordered correctly
        let nil = Term::Nil;
        let small = Term::Small(1);
        let atom = Term::Atom(1);
        let float = Term::Float(1.0);
        
        // Nil < Small
        assert_eq!(erts_cmp(&nil, &small, 0).unwrap(), -1);
        assert_eq!(erts_cmp(&small, &nil, 0).unwrap(), 1);
        
        // Small < Atom
        assert_eq!(erts_cmp(&small, &atom, 0).unwrap(), -1);
        assert_eq!(erts_cmp(&atom, &small, 0).unwrap(), 1);
        
        // Atom < Float
        assert_eq!(erts_cmp(&atom, &float, 0).unwrap(), -1);
        assert_eq!(erts_cmp(&float, &atom, 0).unwrap(), 1);
    }
    
    #[test]
    fn test_erts_cmp_complex_types() {
        // Complex types should return an error
        let a = Term::Tuple(vec![Term::Small(1)]);
        let b = Term::Tuple(vec![Term::Small(2)]);
        let result = erts_cmp(&a, &b, 0);
        assert!(result.is_err());
        match result.unwrap_err() {
            ComparisonError::ComparisonFailed(msg) => {
                assert!(msg.contains("Complex type"));
            }
            _ => panic!("Expected ComparisonFailed error"),
        }
    }
    
    #[test]
    fn test_erts_cmp_list() {
        // Lists should return an error (complex type)
        let a = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::Nil),
        };
        let b = Term::List {
            head: Box::new(Term::Small(2)),
            tail: Box::new(Term::Nil),
        };
        let result = erts_cmp(&a, &b, 0);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_erts_cmp_map() {
        // Maps should return an error (complex type)
        let a = Term::Map(vec![(Term::Small(1), Term::Small(2))]);
        let b = Term::Map(vec![(Term::Small(1), Term::Small(3))]);
        let result = erts_cmp(&a, &b, 0);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_erts_cmp_binary() {
        // Binaries should return an error (complex type)
        let a = Term::Binary {
            data: vec![1, 2],
            bit_offset: 0,
            bit_size: 16,
        };
        let b = Term::Binary {
            data: vec![1, 3],
            bit_offset: 0,
            bit_size: 16,
        };
        let result = erts_cmp(&a, &b, 0);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_erts_cmp_negative_integers() {
        let a = Term::Small(-10);
        let b = Term::Small(-5);
        // -10 < -5
        assert_eq!(erts_cmp(&a, &b, 0).unwrap(), -1);
        assert_eq!(erts_cmp(&b, &a, 0).unwrap(), 1);
        
        let c = Term::Small(0);
        // -10 < 0
        assert_eq!(erts_cmp(&a, &c, 0).unwrap(), -1);
        // 0 > -10
        assert_eq!(erts_cmp(&c, &a, 0).unwrap(), 1);
    }
    
    #[test]
    fn test_erts_cmp_zero() {
        let a = Term::Small(0);
        let b = Term::Small(0);
        assert_eq!(erts_cmp(&a, &b, 0).unwrap(), 0);
    }
    
    #[test]
    fn test_comparison_error_debug() {
        let error1 = ComparisonError::ComparisonFailed("test".to_string());
        let error2 = ComparisonError::InvalidTerm("test".to_string());
        
        let debug_str1 = format!("{:?}", error1);
        let debug_str2 = format!("{:?}", error2);
        
        assert!(debug_str1.contains("ComparisonFailed"));
        assert!(debug_str2.contains("InvalidTerm"));
    }
    
    #[test]
    fn test_comparison_error_clone() {
        let error1 = ComparisonError::ComparisonFailed("test".to_string());
        let error2 = ComparisonError::InvalidTerm("test".to_string());
        
        let cloned1 = error1.clone();
        let cloned2 = error2.clone();
        
        assert_eq!(error1, cloned1);
        assert_eq!(error2, cloned2);
    }
    
    #[test]
    fn test_comparison_error_partial_eq() {
        let error1 = ComparisonError::ComparisonFailed("test".to_string());
        let error2 = ComparisonError::ComparisonFailed("test".to_string());
        let error3 = ComparisonError::ComparisonFailed("different".to_string());
        let error4 = ComparisonError::InvalidTerm("test".to_string());
        
        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
        assert_ne!(error1, error4);
    }
    
    #[test]
    fn test_comparison_error_eq() {
        let error1 = ComparisonError::ComparisonFailed("test".to_string());
        let error2 = ComparisonError::ComparisonFailed("test".to_string());
        let error3 = ComparisonError::InvalidTerm("test".to_string());
        
        assert!(error1 == error2);
        assert!(error1 != error3);
    }
    
    #[test]
    fn test_eq_deeply_nested() {
        // Test deeply nested structures
        let a = Term::Tuple(vec![
            Term::Small(1),
            Term::List {
                head: Box::new(Term::Tuple(vec![
                    Term::Small(2),
                    Term::Map(vec![
                        (Term::Small(3), Term::Small(4)),
                    ]),
                ])),
                tail: Box::new(Term::Nil),
            },
        ]);
        let b = Term::Tuple(vec![
            Term::Small(1),
            Term::List {
                head: Box::new(Term::Tuple(vec![
                    Term::Small(2),
                    Term::Map(vec![
                        (Term::Small(3), Term::Small(4)),
                    ]),
                ])),
                tail: Box::new(Term::Nil),
            },
        ]);
        assert!(eq(&a, &b).unwrap());
    }
    
    #[test]
    fn test_eq_list_with_non_nil_tail() {
        // Test improper list
        let a = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::Small(2)),
        };
        let b = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::Small(2)),
        };
        assert!(eq(&a, &b).unwrap());
        
        let c = Term::List {
            head: Box::new(Term::Small(1)),
            tail: Box::new(Term::Small(3)),
        };
        assert!(!eq(&a, &c).unwrap());
    }
    
    #[test]
    fn test_eq_big_integer_negative() {
        use entities_utilities::BigNumber;
        let a = Term::Big(BigNumber::from_i64(-100));
        let b = Term::Big(BigNumber::from_i64(-100));
        assert!(eq(&a, &b).unwrap());
        
        let c = Term::Big(BigNumber::from_i64(-101));
        assert!(!eq(&a, &c).unwrap());
    }
    
    #[test]
    fn test_erts_cmp_big_integer_negative() {
        use entities_utilities::BigNumber;
        let a = Term::Big(BigNumber::from_i64(-100));
        let b = Term::Big(BigNumber::from_i64(-50));
        // -100 < -50
        assert_eq!(erts_cmp(&a, &b, 0).unwrap(), -1);
        assert_eq!(erts_cmp(&b, &a, 0).unwrap(), 1);
    }
    
    #[test]
    fn test_erts_cmp_big_integer_zero() {
        use entities_utilities::BigNumber;
        let a = Term::Big(BigNumber::from_i64(0));
        let b = Term::Big(BigNumber::from_i64(0));
        assert_eq!(erts_cmp(&a, &b, 0).unwrap(), 0);
    }
    
    #[test]
    fn test_term_type_order_all_types() {
        // Test that term_type_order returns correct values for all types
        assert_eq!(term_type_order(&Term::Nil), 0);
        assert_eq!(term_type_order(&Term::Small(1)), 1);
        assert_eq!(term_type_order(&Term::Atom(1)), 2);
        assert_eq!(term_type_order(&Term::Float(1.0)), 3);
        assert_eq!(term_type_order(&Term::Big(BigNumber::from_i64(1))), 4);
        assert_eq!(term_type_order(&Term::Rational(BigRational::from_i64(1))), 5);
        assert_eq!(term_type_order(&Term::Binary { data: vec![], bit_offset: 0, bit_size: 0 }), 6);
        assert_eq!(term_type_order(&Term::List { head: Box::new(Term::Nil), tail: Box::new(Term::Nil) }), 7);
        assert_eq!(term_type_order(&Term::Tuple(vec![])), 8);
        assert_eq!(term_type_order(&Term::Map(vec![])), 9);
    }
    
    #[test]
    fn test_is_same() {
        // Test is_same function indirectly through eq
        // is_same is used internally in eq for optimization
        let a = Term::Nil;
        let b = Term::Nil;
        assert!(eq(&a, &b).unwrap());
        
        let c = Term::Small(42);
        let d = Term::Small(42);
        assert!(eq(&c, &d).unwrap());
        
        let e = Term::Atom(1);
        let f = Term::Atom(1);
        assert!(eq(&e, &f).unwrap());
        
        let g = Term::Float(3.14);
        let h = Term::Float(3.14);
        assert!(eq(&g, &h).unwrap());
    }
    
    #[test]
    fn test_eq_empty_map() {
        let a = Term::Map(vec![]);
        let b = Term::Map(vec![]);
        assert!(eq(&a, &b).unwrap());
    }
    
    #[test]
    fn test_eq_map_with_nested_values() {
        let a = Term::Map(vec![
            (Term::Small(1), Term::Tuple(vec![Term::Small(2)])),
        ]);
        let b = Term::Map(vec![
            (Term::Small(1), Term::Tuple(vec![Term::Small(2)])),
        ]);
        assert!(eq(&a, &b).unwrap());
    }
    
    #[test]
    fn test_erts_cmp_float_negative() {
        let a = Term::Float(-10.0);
        let b = Term::Float(-5.0);
        // -10.0 < -5.0
        assert_eq!(erts_cmp(&a, &b, 0).unwrap(), -1);
        assert_eq!(erts_cmp(&b, &a, 0).unwrap(), 1);
    }
    
    #[test]
    fn test_erts_cmp_float_zero() {
        let a = Term::Float(0.0);
        let b = Term::Float(0.0);
        assert_eq!(erts_cmp(&a, &b, 0).unwrap(), 0);
    }
}

