//! List Built-in Functions
//!
//! Provides list manipulation operations:
//! - List concatenation (append/2, ++ operator)
//! - List subtraction (subtract/2, -- operator)
//! - List membership (member/2)
//! - List reversal (reverse/2)
//! - Key-based tuple search (keyfind/3, keymember/3, keysearch/3)
//!
//! Based on erl_bif_lists.c
//!
//! This module implements safe Rust equivalents of Erlang list BIFs.

use crate::op::ErlangTerm;
use std::collections::HashMap;

/// Error type for list BIF operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListsError {
    /// Bad argument (e.g., non-list argument)
    BadArgument(String),
    /// Position out of range
    BadPosition(String),
}

/// List Built-in Functions
pub struct ListsBif;

impl ListsBif {
    /// Append two lists (++ operator)
    ///
    /// Concatenates LHS and RHS lists. For historical reasons, this does not
    /// validate that RHS is a proper list - it simply appends RHS to the end
    /// of LHS without checking.
    ///
    /// # Arguments
    /// * `lhs` - Left-hand side list
    /// * `rhs` - Right-hand side list (or any term)
    ///
    /// # Returns
    /// * `Ok(ErlangTerm::List)` - Result of concatenation
    /// * `Err(ListsError)` - If LHS is not a list
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::lists::ListsBif;
    /// use usecases_bifs::op::ErlangTerm;
    ///
    /// let lhs = ErlangTerm::List(vec![
    ///     ErlangTerm::Integer(1),
    ///     ErlangTerm::Integer(2),
    /// ]);
    /// let rhs = ErlangTerm::List(vec![
    ///     ErlangTerm::Integer(3),
    ///     ErlangTerm::Integer(4),
    /// ]);
    /// let result = ListsBif::append_2(&lhs, &rhs).unwrap();
    /// ```
    pub fn append_2(lhs: &ErlangTerm, rhs: &ErlangTerm) -> Result<ErlangTerm, ListsError> {
        match lhs {
            ErlangTerm::List(lhs_vec) => {
                let mut result = lhs_vec.clone();
                match rhs {
                    ErlangTerm::List(rhs_vec) => {
                        result.extend_from_slice(rhs_vec);
                    }
                    ErlangTerm::Nil => {
                        // Appending nil is a no-op
                    }
                    _ => {
                        // For historical compatibility, we allow appending non-lists
                        // This creates an improper list: [1,2|non_list]
                        result.push(rhs.clone());
                    }
                }
                Ok(ErlangTerm::List(result))
            }
            ErlangTerm::Nil => {
                // [] ++ RHS = RHS
                Ok(rhs.clone())
            }
            _ => Err(ListsError::BadArgument(
                "First argument must be a list".to_string(),
            )),
        }
    }

    /// Subtract elements from a list (-- operator)
    ///
    /// Removes the first occurrence of each element in RHS from LHS.
    /// Elements must match exactly (no type coercion).
    ///
    /// # Arguments
    /// * `lhs` - Left-hand side list
    /// * `rhs` - Right-hand side list of elements to remove
    ///
    /// # Returns
    /// * `Ok(ErlangTerm::List)` - Result list with elements removed
    /// * `Err(ListsError)` - If either argument is not a list
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::lists::ListsBif;
    /// use usecases_bifs::op::ErlangTerm;
    ///
    /// let lhs = ErlangTerm::List(vec![
    ///     ErlangTerm::Integer(1),
    ///     ErlangTerm::Integer(2),
    ///     ErlangTerm::Integer(3),
    /// ]);
    /// let rhs = ErlangTerm::List(vec![
    ///     ErlangTerm::Integer(2),
    /// ]);
    /// let result = ListsBif::subtract_2(&lhs, &rhs).unwrap();
    /// // Result: [1, 3]
    /// ```
    pub fn subtract_2(lhs: &ErlangTerm, rhs: &ErlangTerm) -> Result<ErlangTerm, ListsError> {
        let lhs_list = match lhs {
            ErlangTerm::List(l) => l,
            ErlangTerm::Nil => {
                return Ok(ErlangTerm::Nil);
            }
            _ => {
                return Err(ListsError::BadArgument(
                    "First argument must be a list".to_string(),
                ));
            }
        };

        let rhs_list = match rhs {
            ErlangTerm::List(l) => l,
            ErlangTerm::Nil => {
                return Ok(ErlangTerm::List(lhs_list.clone()));
            }
            _ => {
                return Err(ListsError::BadArgument(
                    "Second argument must be a list".to_string(),
                ));
            }
        };

        // Build a multiset of elements to remove for efficient lookup
        let mut to_remove: HashMap<ErlangTerm, usize> = HashMap::new();
        for elem in rhs_list {
            *to_remove.entry(elem.clone()).or_insert(0) += 1;
        }

        // Build result list, removing elements as we go
        let mut result = Vec::new();
        for elem in lhs_list {
            match to_remove.get_mut(elem) {
                Some(count) if *count > 0 => {
                    *count -= 1;
                    // Skip this element (remove it)
                }
                _ => {
                    // Keep this element
                    result.push(elem.clone());
                }
            }
        }

        Ok(ErlangTerm::List(result))
    }

    /// Check if an element is a member of a list
    ///
    /// Returns `true` if the element is found in the list, `false` otherwise.
    /// Uses structural equality (==) for comparison.
    ///
    /// # Arguments
    /// * `term` - Element to search for
    /// * `list` - List to search in
    ///
    /// # Returns
    /// * `Ok(ErlangTerm::Atom("true"))` - If element is found
    /// * `Ok(ErlangTerm::Atom("false"))` - If element is not found
    /// * `Err(ListsError)` - If list is not a proper list
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::lists::ListsBif;
    /// use usecases_bifs::op::ErlangTerm;
    ///
    /// let list = ErlangTerm::List(vec![
    ///     ErlangTerm::Integer(1),
    ///     ErlangTerm::Integer(2),
    ///     ErlangTerm::Integer(3),
    /// ]);
    /// let result = ListsBif::member_2(&ErlangTerm::Integer(2), &list).unwrap();
    /// // Result: Atom("true")
    /// ```
    pub fn member_2(term: &ErlangTerm, list: &ErlangTerm) -> Result<ErlangTerm, ListsError> {
        match list {
            ErlangTerm::Nil => Ok(ErlangTerm::Atom("false".to_string())),
            ErlangTerm::List(list_vec) => {
                for elem in list_vec {
                    // Use structural equality (==)
                    if elem.eq(term) {
                        return Ok(ErlangTerm::Atom("true".to_string()));
                    }
                }
                Ok(ErlangTerm::Atom("false".to_string()))
            }
            _ => Err(ListsError::BadArgument(
                "Second argument must be a list".to_string(),
            )),
        }
    }

    /// Reverse a list with an optional tail
    ///
    /// Reverses the first list and appends the tail to the result.
    /// If the first argument is an empty list, returns the tail.
    ///
    /// # Arguments
    /// * `list` - List to reverse
    /// * `tail` - Optional tail to append (defaults to Nil if not provided)
    ///
    /// # Returns
    /// * `Ok(ErlangTerm::List)` - Reversed list with tail appended
    /// * `Err(ListsError)` - If list is not a proper list
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::lists::ListsBif;
    /// use usecases_bifs::op::ErlangTerm;
    ///
    /// let list = ErlangTerm::List(vec![
    ///     ErlangTerm::Integer(1),
    ///     ErlangTerm::Integer(2),
    ///     ErlangTerm::Integer(3),
    /// ]);
    /// let tail = ErlangTerm::Nil;
    /// let result = ListsBif::reverse_2(&list, &tail).unwrap();
    /// // Result: [3, 2, 1]
    /// ```
    pub fn reverse_2(list: &ErlangTerm, tail: &ErlangTerm) -> Result<ErlangTerm, ListsError> {
        match list {
            ErlangTerm::Nil => Ok(tail.clone()),
            ErlangTerm::List(list_vec) => {
                let mut reversed: Vec<ErlangTerm> = list_vec.iter().rev().cloned().collect();
                
                // Append tail
                match tail {
                    ErlangTerm::List(tail_vec) => {
                        reversed.extend_from_slice(tail_vec);
                    }
                    ErlangTerm::Nil => {
                        // No tail to append
                    }
                    _ => {
                        // For compatibility, allow non-list tails (creates improper list)
                        reversed.push(tail.clone());
                    }
                }
                
                Ok(ErlangTerm::List(reversed))
            }
            _ => Err(ListsError::BadArgument(
                "First argument must be a list".to_string(),
            )),
        }
    }

    /// Find a tuple in a list by key at a given position
    ///
    /// Searches through a list of tuples for a tuple whose element at the
    /// specified position matches the given key. Returns the first matching
    /// tuple, or `false` if no match is found.
    ///
    /// # Arguments
    /// * `key` - Key value to search for
    /// * `pos` - Position (1-indexed) in tuple to compare
    /// * `list` - List of tuples to search
    ///
    /// # Returns
    /// * `Ok(ErlangTerm::Tuple)` - First matching tuple
    /// * `Ok(ErlangTerm::Atom("false"))` - If no match found
    /// * `Err(ListsError)` - If arguments are invalid
    ///
    /// # Examples
    /// ```
    /// use usecases_bifs::lists::ListsBif;
    /// use usecases_bifs::op::ErlangTerm;
    ///
    /// let list = ErlangTerm::List(vec![
    ///     ErlangTerm::Tuple(vec![
    ///         ErlangTerm::Atom("a".to_string()),
    ///         ErlangTerm::Integer(1),
    ///     ]),
    ///     ErlangTerm::Tuple(vec![
    ///         ErlangTerm::Atom("b".to_string()),
    ///         ErlangTerm::Integer(2),
    ///     ]),
    /// ]);
    /// let result = ListsBif::keyfind_3(
    ///     &ErlangTerm::Atom("b".to_string()),
    ///     &ErlangTerm::Integer(1),
    ///     &list,
    /// ).unwrap();
    /// ```
    pub fn keyfind_3(
        key: &ErlangTerm,
        pos: &ErlangTerm,
        list: &ErlangTerm,
    ) -> Result<ErlangTerm, ListsError> {
        // Validate position
        let pos_val = match pos {
            ErlangTerm::Integer(n) if *n >= 1 => *n as usize,
            ErlangTerm::BigInteger(bn) => {
                if let Some(n) = bn.to_i64() {
                    if n >= 1 {
                        n as usize
                    } else {
                        return Err(ListsError::BadPosition(
                            "Position must be >= 1".to_string(),
                        ));
                    }
                } else {
                    return Err(ListsError::BadPosition(
                        "Position too large".to_string(),
                    ));
                }
            }
            _ => {
                return Err(ListsError::BadPosition(
                    "Position must be an integer >= 1".to_string(),
                ));
            }
        };

        // Search through list
        let list_vec = match list {
            ErlangTerm::List(l) => l,
            ErlangTerm::Nil => {
                return Ok(ErlangTerm::Atom("false".to_string()));
            }
            _ => {
                return Err(ListsError::BadArgument(
                    "Third argument must be a list".to_string(),
                ));
            }
        };

        for elem in list_vec {
            if let ErlangTerm::Tuple(tuple_vec) = elem {
                if pos_val <= tuple_vec.len() {
                    let element = &tuple_vec[pos_val - 1]; // Convert to 0-indexed
                    // Use structural equality (==)
                    if element.eq(key) {
                        return Ok(elem.clone());
                    }
                }
            }
        }

        Ok(ErlangTerm::Atom("false".to_string()))
    }

    /// Check if a tuple with the given key exists in a list
    ///
    /// Returns `true` if a tuple with the key at the specified position
    /// exists in the list, `false` otherwise.
    ///
    /// # Arguments
    /// * `key` - Key value to search for
    /// * `pos` - Position (1-indexed) in tuple to compare
    /// * `list` - List of tuples to search
    ///
    /// # Returns
    /// * `Ok(ErlangTerm::Atom("true"))` - If tuple found
    /// * `Ok(ErlangTerm::Atom("false"))` - If tuple not found
    /// * `Err(ListsError)` - If arguments are invalid
    pub fn keymember_3(
        key: &ErlangTerm,
        pos: &ErlangTerm,
        list: &ErlangTerm,
    ) -> Result<ErlangTerm, ListsError> {
        let result = Self::keyfind_3(key, pos, list)?;
        match result {
            ErlangTerm::Tuple(_) => Ok(ErlangTerm::Atom("true".to_string())),
            _ => Ok(ErlangTerm::Atom("false".to_string())),
        }
    }

    /// Search for a tuple in a list by key, returning {value, Tuple} or false
    ///
    /// Similar to `keyfind_3`, but returns `{value, Tuple}` if found,
    /// or `false` if not found.
    ///
    /// # Arguments
    /// * `key` - Key value to search for
    /// * `pos` - Position (1-indexed) in tuple to compare
    /// * `list` - List of tuples to search
    ///
    /// # Returns
    /// * `Ok(ErlangTerm::Tuple)` - `{value, Tuple}` if found
    /// * `Ok(ErlangTerm::Atom("false"))` - If not found
    /// * `Err(ListsError)` - If arguments are invalid
    pub fn keysearch_3(
        key: &ErlangTerm,
        pos: &ErlangTerm,
        list: &ErlangTerm,
    ) -> Result<ErlangTerm, ListsError> {
        let result = Self::keyfind_3(key, pos, list)?;
        match result {
            ErlangTerm::Tuple(tuple) => {
                // Return {value, Tuple}
                Ok(ErlangTerm::Tuple(vec![
                    ErlangTerm::Atom("value".to_string()),
                    ErlangTerm::Tuple(tuple),
                ]))
            }
            _ => Ok(ErlangTerm::Atom("false".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append_2_basic() {
        let lhs = ErlangTerm::List(vec![
            ErlangTerm::Integer(1),
            ErlangTerm::Integer(2),
        ]);
        let rhs = ErlangTerm::List(vec![
            ErlangTerm::Integer(3),
            ErlangTerm::Integer(4),
        ]);
        let result = ListsBif::append_2(&lhs, &rhs).unwrap();
        
        if let ErlangTerm::List(result_vec) = result {
            assert_eq!(result_vec.len(), 4);
            assert_eq!(result_vec[0], ErlangTerm::Integer(1));
            assert_eq!(result_vec[1], ErlangTerm::Integer(2));
            assert_eq!(result_vec[2], ErlangTerm::Integer(3));
            assert_eq!(result_vec[3], ErlangTerm::Integer(4));
        } else {
            panic!("Expected List");
        }
    }

    #[test]
    fn test_append_2_empty_lhs() {
        let lhs = ErlangTerm::Nil;
        let rhs = ErlangTerm::List(vec![ErlangTerm::Integer(1)]);
        let result = ListsBif::append_2(&lhs, &rhs).unwrap();
        assert_eq!(result, rhs);
    }

    #[test]
    fn test_append_2_improper_list() {
        // Historical behavior: [] ++ non_list = non_list
        let lhs = ErlangTerm::Nil;
        let rhs = ErlangTerm::Integer(42);
        let result = ListsBif::append_2(&lhs, &rhs).unwrap();
        assert_eq!(result, rhs);
    }

    #[test]
    fn test_append_2_error() {
        let lhs = ErlangTerm::Integer(1);
        let rhs = ErlangTerm::List(vec![ErlangTerm::Integer(2)]);
        let result = ListsBif::append_2(&lhs, &rhs);
        assert!(result.is_err());
    }

    #[test]
    fn test_append_2_nil_rhs() {
        // Appending nil to a list should be a no-op
        let lhs = ErlangTerm::List(vec![
            ErlangTerm::Integer(1),
            ErlangTerm::Integer(2),
        ]);
        let rhs = ErlangTerm::Nil;
        let result = ListsBif::append_2(&lhs, &rhs).unwrap();
        assert_eq!(result, lhs);
    }

    #[test]
    fn test_append_2_non_list_rhs() {
        // Appending non-list to non-empty list creates improper list
        let lhs = ErlangTerm::List(vec![
            ErlangTerm::Integer(1),
            ErlangTerm::Integer(2),
        ]);
        let rhs = ErlangTerm::Integer(42);
        let result = ListsBif::append_2(&lhs, &rhs).unwrap();
        
        if let ErlangTerm::List(result_vec) = result {
            assert_eq!(result_vec.len(), 3);
            assert_eq!(result_vec[0], ErlangTerm::Integer(1));
            assert_eq!(result_vec[1], ErlangTerm::Integer(2));
            assert_eq!(result_vec[2], ErlangTerm::Integer(42));
        } else {
            panic!("Expected List");
        }
    }

    #[test]
    fn test_subtract_2_basic() {
        let lhs = ErlangTerm::List(vec![
            ErlangTerm::Integer(1),
            ErlangTerm::Integer(2),
            ErlangTerm::Integer(3),
        ]);
        let rhs = ErlangTerm::List(vec![ErlangTerm::Integer(2)]);
        let result = ListsBif::subtract_2(&lhs, &rhs).unwrap();
        
        if let ErlangTerm::List(result_vec) = result {
            assert_eq!(result_vec.len(), 2);
            assert_eq!(result_vec[0], ErlangTerm::Integer(1));
            assert_eq!(result_vec[1], ErlangTerm::Integer(3));
        } else {
            panic!("Expected List");
        }
    }

    #[test]
    fn test_subtract_2_empty_lhs() {
        let lhs = ErlangTerm::Nil;
        let rhs = ErlangTerm::List(vec![ErlangTerm::Integer(1)]);
        let result = ListsBif::subtract_2(&lhs, &rhs).unwrap();
        assert_eq!(result, ErlangTerm::Nil);
    }

    #[test]
    fn test_subtract_2_empty_rhs() {
        let lhs = ErlangTerm::List(vec![ErlangTerm::Integer(1)]);
        let rhs = ErlangTerm::Nil;
        let result = ListsBif::subtract_2(&lhs, &rhs).unwrap();
        assert_eq!(result, lhs);
    }

    #[test]
    fn test_subtract_2_multiple_occurrences() {
        let lhs = ErlangTerm::List(vec![
            ErlangTerm::Integer(1),
            ErlangTerm::Integer(2),
            ErlangTerm::Integer(2),
            ErlangTerm::Integer(3),
        ]);
        let rhs = ErlangTerm::List(vec![ErlangTerm::Integer(2)]);
        let result = ListsBif::subtract_2(&lhs, &rhs).unwrap();
        
        // Should remove only the first occurrence
        if let ErlangTerm::List(result_vec) = result {
            assert_eq!(result_vec.len(), 3);
            assert_eq!(result_vec[0], ErlangTerm::Integer(1));
            assert_eq!(result_vec[1], ErlangTerm::Integer(2));
            assert_eq!(result_vec[2], ErlangTerm::Integer(3));
        } else {
            panic!("Expected List");
        }
    }

    #[test]
    fn test_subtract_2_multiple_rhs_occurrences() {
        // Test removing multiple occurrences from RHS
        let lhs = ErlangTerm::List(vec![
            ErlangTerm::Integer(1),
            ErlangTerm::Integer(2),
            ErlangTerm::Integer(2),
            ErlangTerm::Integer(3),
        ]);
        let rhs = ErlangTerm::List(vec![
            ErlangTerm::Integer(2),
            ErlangTerm::Integer(2),
        ]);
        let result = ListsBif::subtract_2(&lhs, &rhs).unwrap();
        
        // Should remove both occurrences of 2
        if let ErlangTerm::List(result_vec) = result {
            assert_eq!(result_vec.len(), 2);
            assert_eq!(result_vec[0], ErlangTerm::Integer(1));
            assert_eq!(result_vec[1], ErlangTerm::Integer(3));
        } else {
            panic!("Expected List");
        }
    }

    #[test]
    fn test_subtract_2_nonexistent_elements() {
        // Test removing elements that don't exist in LHS
        let lhs = ErlangTerm::List(vec![
            ErlangTerm::Integer(1),
            ErlangTerm::Integer(2),
        ]);
        let rhs = ErlangTerm::List(vec![
            ErlangTerm::Integer(3),
            ErlangTerm::Integer(4),
        ]);
        let result = ListsBif::subtract_2(&lhs, &rhs).unwrap();
        
        // Should return LHS unchanged
        assert_eq!(result, lhs);
    }

    #[test]
    fn test_subtract_2_complex_types() {
        // Test with tuples
        let tuple1 = ErlangTerm::Tuple(vec![ErlangTerm::Integer(1)]);
        let tuple2 = ErlangTerm::Tuple(vec![ErlangTerm::Integer(2)]);
        let tuple3 = ErlangTerm::Tuple(vec![ErlangTerm::Integer(3)]);
        
        let lhs = ErlangTerm::List(vec![
            tuple1.clone(),
            tuple2.clone(),
            tuple3.clone(),
        ]);
        let rhs = ErlangTerm::List(vec![tuple2.clone()]);
        let result = ListsBif::subtract_2(&lhs, &rhs).unwrap();
        
        if let ErlangTerm::List(result_vec) = result {
            assert_eq!(result_vec.len(), 2);
            assert_eq!(result_vec[0], tuple1);
            assert_eq!(result_vec[1], tuple3);
        } else {
            panic!("Expected List");
        }
    }

    #[test]
    fn test_member_2_found() {
        let list = ErlangTerm::List(vec![
            ErlangTerm::Integer(1),
            ErlangTerm::Integer(2),
            ErlangTerm::Integer(3),
        ]);
        let result = ListsBif::member_2(&ErlangTerm::Integer(2), &list).unwrap();
        assert_eq!(result, ErlangTerm::Atom("true".to_string()));
    }

    #[test]
    fn test_member_2_not_found() {
        let list = ErlangTerm::List(vec![
            ErlangTerm::Integer(1),
            ErlangTerm::Integer(2),
        ]);
        let result = ListsBif::member_2(&ErlangTerm::Integer(3), &list).unwrap();
        assert_eq!(result, ErlangTerm::Atom("false".to_string()));
    }

    #[test]
    fn test_member_2_empty_list() {
        let list = ErlangTerm::Nil;
        let result = ListsBif::member_2(&ErlangTerm::Integer(1), &list).unwrap();
        assert_eq!(result, ErlangTerm::Atom("false".to_string()));
    }

    #[test]
    fn test_member_2_error() {
        // Test with non-list argument
        let result = ListsBif::member_2(&ErlangTerm::Integer(1), &ErlangTerm::Integer(1));
        assert!(result.is_err());
    }

    #[test]
    fn test_member_2_type_coercion() {
        // Test member with type coercion (Integer == Float)
        let list = ErlangTerm::List(vec![
            ErlangTerm::Integer(1),
            ErlangTerm::Float(2.0),
        ]);
        
        // Integer(1) should match Integer(1)
        let result1 = ListsBif::member_2(&ErlangTerm::Integer(1), &list).unwrap();
        assert_eq!(result1, ErlangTerm::Atom("true".to_string()));
        
        // Integer(2) should match Float(2.0) due to structural equality
        let result2 = ListsBif::member_2(&ErlangTerm::Integer(2), &list).unwrap();
        assert_eq!(result2, ErlangTerm::Atom("true".to_string()));
    }

    #[test]
    fn test_reverse_2_basic() {
        let list = ErlangTerm::List(vec![
            ErlangTerm::Integer(1),
            ErlangTerm::Integer(2),
            ErlangTerm::Integer(3),
        ]);
        let tail = ErlangTerm::Nil;
        let result = ListsBif::reverse_2(&list, &tail).unwrap();
        
        if let ErlangTerm::List(result_vec) = result {
            assert_eq!(result_vec.len(), 3);
            assert_eq!(result_vec[0], ErlangTerm::Integer(3));
            assert_eq!(result_vec[1], ErlangTerm::Integer(2));
            assert_eq!(result_vec[2], ErlangTerm::Integer(1));
        } else {
            panic!("Expected List");
        }
    }

    #[test]
    fn test_reverse_2_with_tail() {
        let list = ErlangTerm::List(vec![ErlangTerm::Integer(1)]);
        let tail = ErlangTerm::List(vec![ErlangTerm::Integer(2)]);
        let result = ListsBif::reverse_2(&list, &tail).unwrap();
        
        if let ErlangTerm::List(result_vec) = result {
            assert_eq!(result_vec.len(), 2);
            assert_eq!(result_vec[0], ErlangTerm::Integer(1));
            assert_eq!(result_vec[1], ErlangTerm::Integer(2));
        } else {
            panic!("Expected List");
        }
    }

    #[test]
    fn test_reverse_2_empty_list() {
        let list = ErlangTerm::Nil;
        let tail = ErlangTerm::List(vec![ErlangTerm::Integer(1)]);
        let result = ListsBif::reverse_2(&list, &tail).unwrap();
        assert_eq!(result, tail);
    }

    #[test]
    fn test_reverse_2_non_list_tail() {
        // Test reverse with non-list tail (creates improper list)
        let list = ErlangTerm::List(vec![ErlangTerm::Integer(1)]);
        let tail = ErlangTerm::Integer(42);
        let result = ListsBif::reverse_2(&list, &tail).unwrap();
        
        if let ErlangTerm::List(result_vec) = result {
            assert_eq!(result_vec.len(), 2);
            assert_eq!(result_vec[0], ErlangTerm::Integer(1));
            assert_eq!(result_vec[1], ErlangTerm::Integer(42));
        } else {
            panic!("Expected List");
        }
    }

    #[test]
    fn test_reverse_2_error() {
        // Test with non-list first argument
        let result = ListsBif::reverse_2(&ErlangTerm::Integer(1), &ErlangTerm::Nil);
        assert!(result.is_err());
    }

    #[test]
    fn test_keyfind_3_found() {
        let list = ErlangTerm::List(vec![
            ErlangTerm::Tuple(vec![
                ErlangTerm::Atom("a".to_string()),
                ErlangTerm::Integer(1),
            ]),
            ErlangTerm::Tuple(vec![
                ErlangTerm::Atom("b".to_string()),
                ErlangTerm::Integer(2),
            ]),
        ]);
        let result = ListsBif::keyfind_3(
            &ErlangTerm::Atom("b".to_string()),
            &ErlangTerm::Integer(1),
            &list,
        )
        .unwrap();
        
        if let ErlangTerm::Tuple(tuple_vec) = result {
            assert_eq!(tuple_vec[0], ErlangTerm::Atom("b".to_string()));
            assert_eq!(tuple_vec[1], ErlangTerm::Integer(2));
        } else {
            panic!("Expected Tuple");
        }
    }

    #[test]
    fn test_keyfind_3_not_found() {
        let list = ErlangTerm::List(vec![
            ErlangTerm::Tuple(vec![
                ErlangTerm::Atom("a".to_string()),
                ErlangTerm::Integer(1),
            ]),
        ]);
        let result = ListsBif::keyfind_3(
            &ErlangTerm::Atom("b".to_string()),
            &ErlangTerm::Integer(1),
            &list,
        )
        .unwrap();
        assert_eq!(result, ErlangTerm::Atom("false".to_string()));
    }

    #[test]
    fn test_keyfind_3_big_integer_position() {
        use entities_utilities::BigNumber;
        
        let list = ErlangTerm::List(vec![
            ErlangTerm::Tuple(vec![
                ErlangTerm::Integer(1),
                ErlangTerm::Atom("a".to_string()),
            ]),
        ]);
        
        // Test with BigInteger position
        let pos = ErlangTerm::BigInteger(BigNumber::from_i64(1));
        let result = ListsBif::keyfind_3(
            &ErlangTerm::Integer(1),
            &pos,
            &list,
        )
        .unwrap();
        
        if let ErlangTerm::Tuple(tuple_vec) = result {
            assert_eq!(tuple_vec[0], ErlangTerm::Integer(1));
        } else {
            panic!("Expected Tuple");
        }
    }

    #[test]
    fn test_keyfind_3_position_zero() {
        let list = ErlangTerm::List(vec![
            ErlangTerm::Tuple(vec![ErlangTerm::Integer(1)]),
        ]);
        
        // Position 0 should error
        let result = ListsBif::keyfind_3(
            &ErlangTerm::Integer(1),
            &ErlangTerm::Integer(0),
            &list,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_keyfind_3_position_negative() {
        let list = ErlangTerm::List(vec![
            ErlangTerm::Tuple(vec![ErlangTerm::Integer(1)]),
        ]);
        
        // Negative position should error
        let result = ListsBif::keyfind_3(
            &ErlangTerm::Integer(1),
            &ErlangTerm::Integer(-1),
            &list,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_keyfind_3_big_integer_position_negative() {
        use entities_utilities::BigNumber;
        
        let list = ErlangTerm::List(vec![
            ErlangTerm::Tuple(vec![ErlangTerm::Integer(1)]),
        ]);
        
        // BigInteger position < 1 should error
        let pos = ErlangTerm::BigInteger(BigNumber::from_i64(0));
        let result = ListsBif::keyfind_3(
            &ErlangTerm::Integer(1),
            &pos,
            &list,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_keyfind_3_big_integer_position_too_large() {
        use entities_utilities::BigNumber;
        
        let list = ErlangTerm::List(vec![
            ErlangTerm::Tuple(vec![ErlangTerm::Integer(1)]),
        ]);
        
        // BigInteger position too large should error
        // Create a BigInteger that's too large to convert to i64
        let large_bn = BigNumber::from_u64(u64::MAX);
        let pos = ErlangTerm::BigInteger(large_bn);
        let result = ListsBif::keyfind_3(
            &ErlangTerm::Integer(1),
            &pos,
            &list,
        );
        // Should error because u64::MAX > i64::MAX, so to_i64() returns None
        assert!(result.is_err());
        if let Err(ListsError::BadPosition(msg)) = result {
            assert!(msg.contains("too large") || msg.contains("Position"));
        } else {
            panic!("Expected BadPosition error");
        }
    }

    #[test]
    fn test_keyfind_3_big_integer_position_exact_boundary() {
        use entities_utilities::BigNumber;
        
        let list = ErlangTerm::List(vec![
            ErlangTerm::Tuple(vec![
                ErlangTerm::Integer(1),
                ErlangTerm::Integer(2),
            ]),
        ]);
        
        // Test BigInteger position at exact boundary (position 2)
        let pos = ErlangTerm::BigInteger(BigNumber::from_i64(2));
        let result = ListsBif::keyfind_3(
            &ErlangTerm::Integer(2),
            &pos,
            &list,
        )
        .unwrap();
        
        if let ErlangTerm::Tuple(tuple_vec) = result {
            assert_eq!(tuple_vec[1], ErlangTerm::Integer(2));
        } else {
            panic!("Expected Tuple");
        }
    }

    #[test]
    fn test_keyfind_3_non_integer_position() {
        let list = ErlangTerm::List(vec![
            ErlangTerm::Tuple(vec![ErlangTerm::Integer(1)]),
        ]);
        
        // Non-integer position should error
        let result = ListsBif::keyfind_3(
            &ErlangTerm::Integer(1),
            &ErlangTerm::Atom("one".to_string()),
            &list,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_keyfind_3_non_list_argument() {
        // Non-list third argument should error
        let result = ListsBif::keyfind_3(
            &ErlangTerm::Integer(1),
            &ErlangTerm::Integer(1),
            &ErlangTerm::Integer(1),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_keyfind_3_empty_list() {
        // Empty list should return false
        let result = ListsBif::keyfind_3(
            &ErlangTerm::Integer(1),
            &ErlangTerm::Integer(1),
            &ErlangTerm::Nil,
        )
        .unwrap();
        assert_eq!(result, ErlangTerm::Atom("false".to_string()));
    }

    #[test]
    fn test_keyfind_3_non_tuple_elements() {
        // List with non-tuple elements should skip them
        let list = ErlangTerm::List(vec![
            ErlangTerm::Integer(1),
            ErlangTerm::Tuple(vec![
                ErlangTerm::Atom("a".to_string()),
                ErlangTerm::Integer(1),
            ]),
        ]);
        let result = ListsBif::keyfind_3(
            &ErlangTerm::Atom("a".to_string()),
            &ErlangTerm::Integer(1),
            &list,
        )
        .unwrap();
        
        if let ErlangTerm::Tuple(tuple_vec) = result {
            assert_eq!(tuple_vec[0], ErlangTerm::Atom("a".to_string()));
        } else {
            panic!("Expected Tuple");
        }
    }

    #[test]
    fn test_keyfind_3_position_out_of_bounds() {
        // Position beyond tuple size should skip
        let list = ErlangTerm::List(vec![
            ErlangTerm::Tuple(vec![
                ErlangTerm::Integer(1),
                ErlangTerm::Integer(2),
            ]),
        ]);
        // Position 3 is out of bounds (tuple only has 2 elements)
        let result = ListsBif::keyfind_3(
            &ErlangTerm::Integer(1),
            &ErlangTerm::Integer(3),
            &list,
        )
        .unwrap();
        // Should return false (position out of bounds)
        assert_eq!(result, ErlangTerm::Atom("false".to_string()));
    }

    #[test]
    fn test_keyfind_3_type_coercion() {
        // Test keyfind with type coercion
        let list = ErlangTerm::List(vec![
            ErlangTerm::Tuple(vec![
                ErlangTerm::Integer(1),
                ErlangTerm::Float(2.0),
            ]),
        ]);
        
        // Integer(1) should match Integer(1)
        let result1 = ListsBif::keyfind_3(
            &ErlangTerm::Integer(1),
            &ErlangTerm::Integer(1),
            &list,
        )
        .unwrap();
        assert!(matches!(result1, ErlangTerm::Tuple(_)));
        
        // Integer(2) should match Float(2.0) due to structural equality
        let result2 = ListsBif::keyfind_3(
            &ErlangTerm::Integer(2),
            &ErlangTerm::Integer(2),
            &list,
        )
        .unwrap();
        assert!(matches!(result2, ErlangTerm::Tuple(_)));
    }

    #[test]
    fn test_keymember_3_found() {
        let list = ErlangTerm::List(vec![
            ErlangTerm::Tuple(vec![
                ErlangTerm::Atom("a".to_string()),
                ErlangTerm::Integer(1),
            ]),
        ]);
        let result = ListsBif::keymember_3(
            &ErlangTerm::Atom("a".to_string()),
            &ErlangTerm::Integer(1),
            &list,
        )
        .unwrap();
        assert_eq!(result, ErlangTerm::Atom("true".to_string()));
    }

    #[test]
    fn test_keysearch_3_found() {
        let list = ErlangTerm::List(vec![
            ErlangTerm::Tuple(vec![
                ErlangTerm::Atom("a".to_string()),
                ErlangTerm::Integer(1),
            ]),
        ]);
        let result = ListsBif::keysearch_3(
            &ErlangTerm::Atom("a".to_string()),
            &ErlangTerm::Integer(1),
            &list,
        )
        .unwrap();
        
        if let ErlangTerm::Tuple(tuple_vec) = result {
            assert_eq!(tuple_vec.len(), 2);
            assert_eq!(tuple_vec[0], ErlangTerm::Atom("value".to_string()));
            if let ErlangTerm::Tuple(inner_tuple) = &tuple_vec[1] {
                assert_eq!(inner_tuple[0], ErlangTerm::Atom("a".to_string()));
            } else {
                panic!("Expected inner Tuple");
            }
        } else {
            panic!("Expected Tuple");
        }
    }

    #[test]
    fn test_keysearch_3_not_found() {
        // Test keysearch when key not found
        let list = ErlangTerm::List(vec![
            ErlangTerm::Tuple(vec![
                ErlangTerm::Atom("a".to_string()),
                ErlangTerm::Integer(1),
            ]),
        ]);
        let result = ListsBif::keysearch_3(
            &ErlangTerm::Atom("b".to_string()),
            &ErlangTerm::Integer(1),
            &list,
        )
        .unwrap();
        assert_eq!(result, ErlangTerm::Atom("false".to_string()));
    }

    #[test]
    fn test_keymember_3_not_found() {
        // Test keymember when key not found
        let list = ErlangTerm::List(vec![
            ErlangTerm::Tuple(vec![
                ErlangTerm::Atom("a".to_string()),
                ErlangTerm::Integer(1),
            ]),
        ]);
        let result = ListsBif::keymember_3(
            &ErlangTerm::Atom("b".to_string()),
            &ErlangTerm::Integer(1),
            &list,
        )
        .unwrap();
        assert_eq!(result, ErlangTerm::Atom("false".to_string()));
    }
}

