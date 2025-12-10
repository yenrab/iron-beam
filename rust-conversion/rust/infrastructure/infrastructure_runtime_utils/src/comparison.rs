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
}

