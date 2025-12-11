//! Term Building Module
//!
//! Provides functions for building Erlang terms on the heap.
//! Based on erts_bld_* functions from utils.c

use entities_data_handling::term_hashing::Term;
use entities_data_handling::atom::{AtomTable, AtomEncoding};
use entities_utilities::BigNumber;

/// Term building error types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TermBuildingError {
    /// Heap too small
    HeapTooSmall,
    /// Invalid argument
    InvalidArgument(String),
    /// Atom not found
    AtomNotFound,
    /// Building failed
    BuildingFailed(String),
}

/// Heap pointer and size tracker
///
/// This struct tracks the heap pointer and size for term building operations.
/// It's used to simulate the C `hpp` (heap pointer pointer) and `szp` (size pointer)
/// parameters from the C functions.
pub struct HeapBuilder {
    /// Current heap position (index in heap)
    heap_pos: usize,
    /// Total size needed (in words)
    size: usize,
    /// Heap data (if building, None if just calculating size)
    heap_data: Option<Vec<Term>>,
}

impl HeapBuilder {
    /// Create a new heap builder for size calculation only
    pub fn new_size_calc() -> Self {
        Self {
            heap_pos: 0,
            size: 0,
            heap_data: None,
        }
    }
    
    /// Create a new heap builder for actual building
    pub fn new_build(initial_capacity: usize) -> Self {
        Self {
            heap_pos: 0,
            size: 0,
            heap_data: Some(Vec::with_capacity(initial_capacity)),
        }
    }
    
    /// Get the current size
    pub fn size(&self) -> usize {
        self.size
    }
    
    /// Get the built heap data (if building)
    pub fn into_heap_data(self) -> Option<Vec<Term>> {
        self.heap_data
    }
    
    /// Add size without building
    fn add_size(&mut self, words: usize) {
        self.size += words;
    }
    
    /// Add term to heap (if building)
    fn add_term(&mut self, term: Term) -> Result<(), TermBuildingError> {
        if let Some(ref mut heap) = self.heap_data {
            heap.push(term);
            self.heap_pos += 1;
            Ok(())
        } else {
            Err(TermBuildingError::BuildingFailed("Not in build mode".to_string()))
        }
    }
}

/// Build an atom term
///
/// Based on `erts_bld_atom()` from utils.c
///
/// # Arguments
/// * `builder` - Heap builder for tracking heap position and size
/// * `atom_table` - Atom table for looking up atoms
/// * `atom_name` - Atom name string
///
/// # Returns
/// * `Ok(Term)` - Built atom term
/// * `Err(TermBuildingError)` - Building error
pub fn erts_bld_atom(
    builder: &mut HeapBuilder,
    atom_table: &AtomTable,
    atom_name: &str,
) -> Result<Term, TermBuildingError> {
    // Look up atom in table
    let atom_bytes = atom_name.as_bytes();
    // Try to get atom index - if not found, create it
    let atom_index = match atom_table.get(atom_bytes, AtomEncoding::SevenBitAscii) {
        Some(idx) => idx,
        None => {
            // Atom not found, try to create it
            atom_table.put_index(atom_bytes, AtomEncoding::SevenBitAscii, false)
                .map_err(|_| TermBuildingError::AtomNotFound)?
        }
    };
    
    // Atoms are immediate values, no heap allocation needed
    // Just add 0 to size (atoms don't use heap)
    builder.add_size(0);
    
    Ok(Term::Atom(atom_index as u32))
}

/// Build an unsigned integer term
///
/// Based on `erts_bld_uint()` from utils.c
///
/// # Arguments
/// * `builder` - Heap builder for tracking heap position and size
/// * `value` - Unsigned integer value
///
/// # Returns
/// * `Ok(Term)` - Built integer term (Small or Big)
/// * `Err(TermBuildingError)` - Building error
pub fn erts_bld_uint(
    builder: &mut HeapBuilder,
    value: u64,
) -> Result<Term, TermBuildingError> {
    // Check if value fits in small integer
    // Small integers are signed i64 values in range [-2^59, 2^59-1]
    // For unsigned, we check if value <= i64::MAX
    if value <= i64::MAX as u64 {
        // Small integer - no heap allocation
        builder.add_size(0);
        Ok(Term::Small(value as i64))
    } else {
        // Big integer - needs heap allocation
        // Estimate size: 1 word header + words for value
        let words_needed = 1 + ((value.ilog2() / 64) as usize + 1);
        builder.add_size(words_needed);
        
        if builder.heap_data.is_some() {
            // Convert to BigNumber and build
            let big_num = BigNumber::from_u64(value);
            Ok(Term::Big(big_num))
        } else {
            // Size calculation only
            Ok(Term::Small(0)) // Placeholder
        }
    }
}

/// Build a uword term
///
/// Based on `erts_bld_uword()` from utils.c
///
/// Similar to `erts_bld_uint()` but for pointer-sized values.
pub fn erts_bld_uword(
    builder: &mut HeapBuilder,
    value: usize,
) -> Result<Term, TermBuildingError> {
    erts_bld_uint(builder, value as u64)
}

/// Build a uint64 term
///
/// Based on `erts_bld_uint64()` from utils.c
pub fn erts_bld_uint64(
    builder: &mut HeapBuilder,
    value: u64,
) -> Result<Term, TermBuildingError> {
    erts_bld_uint(builder, value)
}

/// Build a sint64 term
///
/// Based on `erts_bld_sint64()` from utils.c
pub fn erts_bld_sint64(
    builder: &mut HeapBuilder,
    value: i64,
) -> Result<Term, TermBuildingError> {
    // Check if value fits in small integer
    // Small integers are in range [-2^59, 2^59-1]
    if value >= -(1i64 << 59) && value < (1i64 << 59) {
        // Small integer - no heap allocation
        builder.add_size(0);
        Ok(Term::Small(value))
    } else {
        // Big integer - needs heap allocation
        let abs_value = value.unsigned_abs();
        let words_needed = 1 + ((abs_value.ilog2() / 64) as usize + 1);
        builder.add_size(words_needed);
        
        if builder.heap_data.is_some() {
            let big_num = BigNumber::from_i64(value);
            Ok(Term::Big(big_num))
        } else {
            Ok(Term::Small(0)) // Placeholder
        }
    }
}

/// Build a cons cell
///
/// Based on `erts_bld_cons()` from utils.c
///
/// # Arguments
/// * `builder` - Heap builder
/// * `car` - Head of the list
/// * `cdr` - Tail of the list
///
/// # Returns
/// * `Ok(Term)` - Built cons cell
/// * `Err(TermBuildingError)` - Building error
pub fn erts_bld_cons(
    builder: &mut HeapBuilder,
    car: Term,
    cdr: Term,
) -> Result<Term, TermBuildingError> {
    // Cons cell needs 2 words (head and tail)
    builder.add_size(2);
    
    if builder.heap_data.is_some() {
        Ok(Term::List {
            head: Box::new(car),
            tail: Box::new(cdr),
        })
    } else {
        Ok(Term::Nil) // Placeholder for size calculation
    }
}

/// Build a tuple
///
/// Based on `erts_bld_tuple()` from utils.c
///
/// # Arguments
/// * `builder` - Heap builder
/// * `elements` - Tuple elements
///
/// # Returns
/// * `Ok(Term)` - Built tuple
/// * `Err(TermBuildingError)` - Building error
pub fn erts_bld_tuple(
    builder: &mut HeapBuilder,
    elements: Vec<Term>,
) -> Result<Term, TermBuildingError> {
    let arity = elements.len();
    
    if arity == 0 {
        // Empty tuple - special case, no heap allocation
        builder.add_size(0);
        return Ok(Term::Tuple(vec![]));
    }
    
    // Tuple needs: 1 word for header + arity words for elements
    builder.add_size(arity + 1);
    
    if builder.heap_data.is_some() {
        Ok(Term::Tuple(elements))
    } else {
        Ok(Term::Tuple(vec![])) // Placeholder for size calculation
    }
}

/// Build a tuple from a vector
///
/// Based on `erts_bld_tuplev()` from utils.c
pub fn erts_bld_tuplev(
    builder: &mut HeapBuilder,
    elements: Vec<Term>,
) -> Result<Term, TermBuildingError> {
    erts_bld_tuple(builder, elements)
}

/// Build a string (list of small integers)
///
/// Based on `erts_bld_string_n()` from utils.c
///
/// # Arguments
/// * `builder` - Heap builder
/// * `str` - String bytes
/// * `len` - String length
///
/// # Returns
/// * `Ok(Term)` - Built string (as a list)
/// * `Err(TermBuildingError)` - Building error
pub fn erts_bld_string_n(
    builder: &mut HeapBuilder,
    str: &[u8],
    len: usize,
) -> Result<Term, TermBuildingError> {
    // String is built as a list of small integers (bytes)
    // Each cons cell needs 2 words
    let words_needed = len * 2;
    builder.add_size(words_needed);
    
    if builder.heap_data.is_some() {
        // Build list backwards (as in C code)
        let mut list = Term::Nil;
        for &byte in str.iter().rev() {
            let head = Term::Small(byte as i64);
            list = Term::List {
                head: Box::new(head),
                tail: Box::new(list),
            };
        }
        Ok(list)
    } else {
        Ok(Term::Nil) // Placeholder
    }
}

/// Build a list from an array of terms
///
/// Based on `erts_bld_list()` from utils.c
///
/// # Arguments
/// * `builder` - Heap builder
/// * `length` - List length
/// * `terms` - Array of terms
///
/// # Returns
/// * `Ok(Term)` - Built list
/// * `Err(TermBuildingError)` - Building error
pub fn erts_bld_list(
    builder: &mut HeapBuilder,
    length: usize,
    terms: &[Term],
) -> Result<Term, TermBuildingError> {
    // Each cons cell needs 2 words
    let words_needed = length * 2;
    builder.add_size(words_needed);
    
    if builder.heap_data.is_some() {
        // Build list backwards (as in C code)
        let mut list = Term::Nil;
        for term in terms.iter().rev() {
            list = Term::List {
                head: Box::new(term.clone()),
                tail: Box::new(list),
            };
        }
        Ok(list)
    } else {
        Ok(Term::Nil) // Placeholder
    }
}

/// Build a list of 2-tuples
///
/// Based on `erts_bld_2tup_list()` from utils.c
pub fn erts_bld_2tup_list(
    builder: &mut HeapBuilder,
    length: usize,
    terms1: &[Term],
    terms2: &[u64],
) -> Result<Term, TermBuildingError> {
    // Each element is a 2-tuple (3 words: header + 2 elements)
    // Each cons cell needs 2 words
    // Total: length * (3 + 2) = length * 5
    let words_needed = length * 5;
    builder.add_size(words_needed);
    
    if builder.heap_data.is_some() {
        let mut list = Term::Nil;
        for i in (0..length).rev() {
            let term2 = erts_bld_uint(builder, terms2[i])?;
            let tuple = Term::Tuple(vec![terms1[i].clone(), term2]);
            list = Term::List {
                head: Box::new(tuple),
                tail: Box::new(list),
            };
        }
        Ok(list)
    } else {
        Ok(Term::Nil) // Placeholder
    }
}

/// Build a list of atom-uword 2-tuples
///
/// Based on `erts_bld_atom_uword_2tup_list()` from utils.c
pub fn erts_bld_atom_uword_2tup_list(
    builder: &mut HeapBuilder,
    atom_table: &AtomTable,
    length: usize,
    atoms: &[&str],
    uwords: &[usize],
) -> Result<Term, TermBuildingError> {
    // Convert atoms to terms
    let mut atom_terms = Vec::new();
    for atom_name in atoms {
        let atom_term = erts_bld_atom(builder, atom_table, atom_name)?;
        atom_terms.push(atom_term);
    }
    
    // Convert uwords to terms
    let mut uword_terms = Vec::new();
    for &uw in uwords {
        let uw_term = erts_bld_uword(builder, uw)?;
        uword_terms.push(uw_term);
    }
    
    // Build 2-tuple list
    let mut list = Term::Nil;
    for i in (0..length).rev() {
        let tuple = Term::Tuple(vec![atom_terms[i].clone(), uword_terms[i].clone()]);
        list = Term::List {
            head: Box::new(tuple),
            tail: Box::new(list),
        };
    }
    
    Ok(list)
}

/// Build a list of atom-2uint 3-tuples
///
/// Based on `erts_bld_atom_2uint_3tup_list()` from utils.c
pub fn erts_bld_atom_2uint_3tup_list(
    builder: &mut HeapBuilder,
    atom_table: &AtomTable,
    length: usize,
    atoms: &[&str],
    uints1: &[u64],
    uints2: &[u64],
) -> Result<Term, TermBuildingError> {
    // Convert atoms to terms
    let mut atom_terms = Vec::new();
    for atom_name in atoms {
        let atom_term = erts_bld_atom(builder, atom_table, atom_name)?;
        atom_terms.push(atom_term);
    }
    
    // Convert uints to terms
    let mut uint1_terms = Vec::new();
    for &u1 in uints1 {
        let u1_term = erts_bld_uint(builder, u1)?;
        uint1_terms.push(u1_term);
    }
    
    let mut uint2_terms = Vec::new();
    for &u2 in uints2 {
        let u2_term = erts_bld_uint(builder, u2)?;
        uint2_terms.push(u2_term);
    }
    
    // Build 3-tuple list
    let mut list = Term::Nil;
    for i in (0..length).rev() {
        let tuple = Term::Tuple(vec![
            atom_terms[i].clone(),
            uint1_terms[i].clone(),
            uint2_terms[i].clone(),
        ]);
        list = Term::List {
            head: Box::new(tuple),
            tail: Box::new(list),
        };
    }
    
    Ok(list)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_erts_bld_atom() {
        let mut atom_table = AtomTable::new(100);
        let _atom_index = atom_table.put_index(b"test", AtomEncoding::SevenBitAscii, false).unwrap();
        
        let mut builder = HeapBuilder::new_size_calc();
        let term = erts_bld_atom(&mut builder, &atom_table, "test").unwrap();
        
        match term {
            Term::Atom(_) => {},
            _ => panic!("Expected Atom"),
        }
    }
    
    #[test]
    fn test_erts_bld_uint_small() {
        let mut builder = HeapBuilder::new_size_calc();
        let term = erts_bld_uint(&mut builder, 42).unwrap();
        
        match term {
            Term::Small(42) => {},
            _ => panic!("Expected Small(42)"),
        }
        assert_eq!(builder.size(), 0); // Small integers don't use heap
    }
    
    #[test]
    fn test_erts_bld_tuple() {
        let mut builder = HeapBuilder::new_size_calc();
        let elements = vec![Term::Small(1), Term::Small(2), Term::Small(3)];
        let term = erts_bld_tuple(&mut builder, elements.clone()).unwrap();
        
        // For size calculation mode, we get a placeholder
        // Just verify size is correct
        // Tuple needs: 1 (header) + 3 (elements) = 4 words
        assert_eq!(builder.size(), 4);
        
        // Test actual building
        let mut builder2 = HeapBuilder::new_build(100);
        let term2 = erts_bld_tuple(&mut builder2, elements).unwrap();
        match term2 {
            Term::Tuple(elements) => assert_eq!(elements.len(), 3),
            _ => panic!("Expected Tuple with 3 elements"),
        }
    }
    
    #[test]
    fn test_erts_bld_cons() {
        let mut builder = HeapBuilder::new_size_calc();
        let car = Term::Small(1);
        let cdr = Term::Nil;
        let term = erts_bld_cons(&mut builder, car, cdr).unwrap();
        
        match term {
            Term::List { .. } => {},
            Term::Nil => {
                // For size calculation, we might get a placeholder
                // Just verify size is correct
            }
            _ => panic!("Expected List or Nil"),
        }
        assert_eq!(builder.size(), 2); // Cons cell needs 2 words
    }
    
    #[test]
    fn test_erts_bld_cons_build_mode() {
        let mut builder = HeapBuilder::new_build(100);
        let car = Term::Small(1);
        let cdr = Term::Small(2);
        let term = erts_bld_cons(&mut builder, car, cdr).unwrap();
        
        match term {
            Term::List { head, tail } => {
                match *head {
                    Term::Small(1) => {},
                    _ => panic!("Expected Small(1)"),
                }
                match *tail {
                    Term::Small(2) => {},
                    _ => panic!("Expected Small(2)"),
                }
            }
            _ => panic!("Expected List"),
        }
        assert_eq!(builder.size(), 2);
    }
    
    #[test]
    fn test_erts_bld_atom_new_atom() {
        let mut atom_table = AtomTable::new(100);
        let mut builder = HeapBuilder::new_size_calc();
        
        // Build atom that doesn't exist yet (should create it)
        let term = erts_bld_atom(&mut builder, &atom_table, "new_atom").unwrap();
        match term {
            Term::Atom(_) => {},
            _ => panic!("Expected Atom"),
        }
        assert_eq!(builder.size(), 0);
    }
    
    #[test]
    fn test_erts_bld_atom_build_mode() {
        let mut atom_table = AtomTable::new(100);
        let mut builder = HeapBuilder::new_build(100);
        let term = erts_bld_atom(&mut builder, &atom_table, "test").unwrap();
        match term {
            Term::Atom(_) => {},
            _ => panic!("Expected Atom"),
        }
    }
    
    #[test]
    fn test_erts_bld_uint_large() {
        let mut builder = HeapBuilder::new_size_calc();
        let large_value = u64::MAX;
        let term = erts_bld_uint(&mut builder, large_value).unwrap();
        
        // For size calculation, we get a placeholder
        assert!(builder.size() > 0); // Big integers use heap
        
        // Test actual building
        let mut builder2 = HeapBuilder::new_build(100);
        let term2 = erts_bld_uint(&mut builder2, large_value).unwrap();
        match term2 {
            Term::Big(_) => {},
            _ => panic!("Expected Big for large value"),
        }
    }
    
    #[test]
    fn test_erts_bld_uint_zero() {
        let mut builder = HeapBuilder::new_size_calc();
        let term = erts_bld_uint(&mut builder, 0).unwrap();
        match term {
            Term::Small(0) => {},
            _ => panic!("Expected Small(0)"),
        }
        assert_eq!(builder.size(), 0);
    }
    
    #[test]
    fn test_erts_bld_uint_boundary() {
        // Test boundary between small and big integers
        let mut builder = HeapBuilder::new_size_calc();
        let boundary = i64::MAX as u64;
        let term = erts_bld_uint(&mut builder, boundary).unwrap();
        match term {
            Term::Small(_) => {},
            _ => panic!("Expected Small for boundary value"),
        }
        assert_eq!(builder.size(), 0);
    }
    
    #[test]
    fn test_erts_bld_uword() {
        let mut builder = HeapBuilder::new_size_calc();
        let term = erts_bld_uword(&mut builder, 42).unwrap();
        match term {
            Term::Small(42) => {},
            _ => panic!("Expected Small(42)"),
        }
    }
    
    #[test]
    fn test_erts_bld_uint64() {
        let mut builder = HeapBuilder::new_size_calc();
        let term = erts_bld_uint64(&mut builder, 123).unwrap();
        match term {
            Term::Small(123) => {},
            _ => panic!("Expected Small(123)"),
        }
    }
    
    #[test]
    fn test_erts_bld_sint64_small() {
        let mut builder = HeapBuilder::new_size_calc();
        let term = erts_bld_sint64(&mut builder, 42).unwrap();
        match term {
            Term::Small(42) => {},
            _ => panic!("Expected Small(42)"),
        }
        assert_eq!(builder.size(), 0);
    }
    
    #[test]
    fn test_erts_bld_sint64_negative() {
        let mut builder = HeapBuilder::new_size_calc();
        let term = erts_bld_sint64(&mut builder, -42).unwrap();
        match term {
            Term::Small(-42) => {},
            _ => panic!("Expected Small(-42)"),
        }
        assert_eq!(builder.size(), 0);
    }
    
    #[test]
    fn test_erts_bld_sint64_large() {
        // Test with a value that requires big integer
        let mut builder = HeapBuilder::new_size_calc();
        // Use a value outside small integer range
        let large_value = 1i64 << 60; // Larger than 2^59
        let term = erts_bld_sint64(&mut builder, large_value).unwrap();
        assert!(builder.size() > 0); // Should use heap
        
        // Test actual building
        let mut builder2 = HeapBuilder::new_build(100);
        let term2 = erts_bld_sint64(&mut builder2, large_value).unwrap();
        match term2 {
            Term::Big(_) => {},
            _ => panic!("Expected Big for large value"),
        }
    }
    
    #[test]
    fn test_erts_bld_sint64_negative_large() {
        let mut builder = HeapBuilder::new_size_calc();
        let large_negative = -(1i64 << 60);
        let term = erts_bld_sint64(&mut builder, large_negative).unwrap();
        assert!(builder.size() > 0);
        
        let mut builder2 = HeapBuilder::new_build(100);
        let term2 = erts_bld_sint64(&mut builder2, large_negative).unwrap();
        match term2 {
            Term::Big(_) => {},
            _ => panic!("Expected Big for large negative value"),
        }
    }
    
    #[test]
    fn test_erts_bld_tuple_empty() {
        let mut builder = HeapBuilder::new_size_calc();
        let term = erts_bld_tuple(&mut builder, vec![]).unwrap();
        match term {
            Term::Tuple(elements) => assert_eq!(elements.len(), 0),
            _ => panic!("Expected empty Tuple"),
        }
        assert_eq!(builder.size(), 0); // Empty tuple doesn't use heap
    }
    
    #[test]
    fn test_erts_bld_tuple_single_element() {
        let mut builder = HeapBuilder::new_build(100);
        let term = erts_bld_tuple(&mut builder, vec![Term::Small(1)]).unwrap();
        match term {
            Term::Tuple(elements) => {
                assert_eq!(elements.len(), 1);
                match elements[0] {
                    Term::Small(1) => {},
                    _ => panic!("Expected Small(1)"),
                }
            }
            _ => panic!("Expected Tuple"),
        }
        assert_eq!(builder.size(), 2); // 1 header + 1 element
    }
    
    #[test]
    fn test_erts_bld_tuplev() {
        let mut builder = HeapBuilder::new_size_calc();
        let elements = vec![Term::Small(1), Term::Small(2)];
        let term = erts_bld_tuplev(&mut builder, elements).unwrap();
        // Should be same as erts_bld_tuple
        assert_eq!(builder.size(), 3); // 1 header + 2 elements
    }
    
    #[test]
    fn test_erts_bld_string_n() {
        let mut builder = HeapBuilder::new_size_calc();
        let str_bytes = b"hello";
        let term = erts_bld_string_n(&mut builder, str_bytes, str_bytes.len()).unwrap();
        // String needs 2 words per character
        assert_eq!(builder.size(), str_bytes.len() * 2);
        
        // Test actual building
        let mut builder2 = HeapBuilder::new_build(100);
        let term2 = erts_bld_string_n(&mut builder2, str_bytes, str_bytes.len()).unwrap();
        match term2 {
            Term::List { .. } => {},
            _ => panic!("Expected List"),
        }
    }
    
    #[test]
    fn test_erts_bld_string_n_empty() {
        let mut builder = HeapBuilder::new_size_calc();
        let term = erts_bld_string_n(&mut builder, b"", 0).unwrap();
        match term {
            Term::Nil => {},
            _ => panic!("Expected Nil for empty string"),
        }
        assert_eq!(builder.size(), 0);
    }
    
    #[test]
    fn test_erts_bld_list() {
        let mut builder = HeapBuilder::new_size_calc();
        let terms = vec![Term::Small(1), Term::Small(2), Term::Small(3)];
        let term = erts_bld_list(&mut builder, terms.len(), &terms).unwrap();
        assert_eq!(builder.size(), terms.len() * 2); // 2 words per cons cell
        
        // Test actual building
        let mut builder2 = HeapBuilder::new_build(100);
        let term2 = erts_bld_list(&mut builder2, terms.len(), &terms).unwrap();
        match term2 {
            Term::List { .. } => {},
            _ => panic!("Expected List"),
        }
    }
    
    #[test]
    fn test_erts_bld_list_empty() {
        let mut builder = HeapBuilder::new_size_calc();
        let terms = vec![];
        let term = erts_bld_list(&mut builder, 0, &terms).unwrap();
        match term {
            Term::Nil => {},
            _ => panic!("Expected Nil for empty list"),
        }
        assert_eq!(builder.size(), 0);
    }
    
    #[test]
    fn test_erts_bld_2tup_list() {
        let mut builder = HeapBuilder::new_size_calc();
        let terms1 = vec![Term::Small(1), Term::Small(2)];
        let terms2 = vec![10u64, 20u64];
        let term = erts_bld_2tup_list(&mut builder, 2, &terms1, &terms2).unwrap();
        // Each element: 3 words (tuple) + 2 words (cons) = 5 words
        assert_eq!(builder.size(), 2 * 5);
        
        // Test actual building
        let mut builder2 = HeapBuilder::new_build(100);
        let term2 = erts_bld_2tup_list(&mut builder2, 2, &terms1, &terms2).unwrap();
        match term2 {
            Term::List { .. } => {},
            _ => panic!("Expected List"),
        }
    }
    
    #[test]
    fn test_erts_bld_2tup_list_empty() {
        let mut builder = HeapBuilder::new_size_calc();
        let terms1 = vec![];
        let terms2 = vec![];
        let term = erts_bld_2tup_list(&mut builder, 0, &terms1, &terms2).unwrap();
        match term {
            Term::Nil => {},
            _ => panic!("Expected Nil for empty list"),
        }
        assert_eq!(builder.size(), 0);
    }
    
    #[test]
    fn test_erts_bld_atom_uword_2tup_list() {
        let mut atom_table = AtomTable::new(100);
        let mut builder = HeapBuilder::new_build(100);
        let atoms = vec!["atom1", "atom2"];
        let uwords = vec![10usize, 20usize];
        let term = erts_bld_atom_uword_2tup_list(&mut builder, &atom_table, 2, &atoms, &uwords).unwrap();
        match term {
            Term::List { .. } => {},
            _ => panic!("Expected List"),
        }
    }
    
    #[test]
    fn test_erts_bld_atom_uword_2tup_list_empty() {
        let mut atom_table = AtomTable::new(100);
        let mut builder = HeapBuilder::new_size_calc();
        let atoms = vec![];
        let uwords = vec![];
        let term = erts_bld_atom_uword_2tup_list(&mut builder, &atom_table, 0, &atoms, &uwords).unwrap();
        match term {
            Term::Nil => {},
            _ => panic!("Expected Nil for empty list"),
        }
    }
    
    #[test]
    fn test_erts_bld_atom_2uint_3tup_list() {
        let mut atom_table = AtomTable::new(100);
        let mut builder = HeapBuilder::new_build(100);
        let atoms = vec!["atom1", "atom2"];
        let uints1 = vec![10u64, 20u64];
        let uints2 = vec![30u64, 40u64];
        let term = erts_bld_atom_2uint_3tup_list(&mut builder, &atom_table, 2, &atoms, &uints1, &uints2).unwrap();
        match term {
            Term::List { .. } => {},
            _ => panic!("Expected List"),
        }
    }
    
    #[test]
    fn test_erts_bld_atom_2uint_3tup_list_empty() {
        let mut atom_table = AtomTable::new(100);
        let mut builder = HeapBuilder::new_size_calc();
        let atoms = vec![];
        let uints1 = vec![];
        let uints2 = vec![];
        let term = erts_bld_atom_2uint_3tup_list(&mut builder, &atom_table, 0, &atoms, &uints1, &uints2).unwrap();
        match term {
            Term::Nil => {},
            _ => panic!("Expected Nil for empty list"),
        }
    }
    
    #[test]
    fn test_heap_builder_new_size_calc() {
        let builder = HeapBuilder::new_size_calc();
        assert_eq!(builder.size(), 0);
        assert!(builder.heap_data.is_none());
    }
    
    #[test]
    fn test_heap_builder_new_build() {
        let builder = HeapBuilder::new_build(100);
        assert_eq!(builder.size(), 0);
        assert!(builder.heap_data.is_some());
    }
    
    #[test]
    fn test_heap_builder_into_heap_data() {
        let builder = HeapBuilder::new_build(100);
        let heap_data = builder.into_heap_data();
        assert!(heap_data.is_some());
        
        let builder2 = HeapBuilder::new_size_calc();
        let heap_data2 = builder2.into_heap_data();
        assert!(heap_data2.is_none());
    }
    
    #[test]
    fn test_term_building_error_debug() {
        let error1 = TermBuildingError::HeapTooSmall;
        let error2 = TermBuildingError::InvalidArgument("test".to_string());
        let error3 = TermBuildingError::AtomNotFound;
        let error4 = TermBuildingError::BuildingFailed("test".to_string());
        
        let debug_str1 = format!("{:?}", error1);
        let debug_str2 = format!("{:?}", error2);
        let debug_str3 = format!("{:?}", error3);
        let debug_str4 = format!("{:?}", error4);
        
        assert!(debug_str1.contains("HeapTooSmall"));
        assert!(debug_str2.contains("InvalidArgument"));
        assert!(debug_str3.contains("AtomNotFound"));
        assert!(debug_str4.contains("BuildingFailed"));
    }
    
    #[test]
    fn test_term_building_error_clone() {
        let error1 = TermBuildingError::HeapTooSmall;
        let error2 = TermBuildingError::InvalidArgument("test".to_string());
        let error3 = TermBuildingError::AtomNotFound;
        let error4 = TermBuildingError::BuildingFailed("test".to_string());
        
        let cloned1 = error1.clone();
        let cloned2 = error2.clone();
        let cloned3 = error3.clone();
        let cloned4 = error4.clone();
        
        assert_eq!(error1, cloned1);
        assert_eq!(error2, cloned2);
        assert_eq!(error3, cloned3);
        assert_eq!(error4, cloned4);
    }
    
    #[test]
    fn test_term_building_error_partial_eq() {
        let error1 = TermBuildingError::HeapTooSmall;
        let error2 = TermBuildingError::HeapTooSmall;
        let error3 = TermBuildingError::InvalidArgument("test".to_string());
        let error4 = TermBuildingError::InvalidArgument("test".to_string());
        let error5 = TermBuildingError::InvalidArgument("different".to_string());
        
        assert_eq!(error1, error2);
        assert_eq!(error3, error4);
        assert_ne!(error3, error5);
        assert_ne!(error1, error3);
    }
    
    #[test]
    fn test_term_building_error_eq() {
        let error1 = TermBuildingError::HeapTooSmall;
        let error2 = TermBuildingError::HeapTooSmall;
        let error3 = TermBuildingError::AtomNotFound;
        
        assert!(error1 == error2);
        assert!(error1 != error3);
    }
    
    #[test]
    fn test_erts_bld_cons_with_nil_tail() {
        let mut builder = HeapBuilder::new_build(100);
        let car = Term::Small(1);
        let cdr = Term::Nil;
        let term = erts_bld_cons(&mut builder, car, cdr).unwrap();
        match term {
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
    fn test_erts_bld_string_n_partial_length() {
        let mut builder = HeapBuilder::new_build(100);
        let str_bytes = b"hello";
        // Use length less than actual bytes
        let term = erts_bld_string_n(&mut builder, str_bytes, 3).unwrap();
        match term {
            Term::List { .. } => {},
            _ => panic!("Expected List"),
        }
        // Should only use 3 * 2 = 6 words
        assert_eq!(builder.size(), 6);
    }
    
    #[test]
    fn test_erts_bld_list_with_atoms() {
        let mut atom_table = AtomTable::new(100);
        let mut builder = HeapBuilder::new_build(100);
        let atom1 = erts_bld_atom(&mut builder, &atom_table, "atom1").unwrap();
        let atom2 = erts_bld_atom(&mut builder, &atom_table, "atom2").unwrap();
        let terms = vec![atom1, atom2];
        let term = erts_bld_list(&mut builder, terms.len(), &terms).unwrap();
        match term {
            Term::List { .. } => {},
            _ => panic!("Expected List"),
        }
    }
    
    #[test]
    fn test_erts_bld_tuple_with_nested_terms() {
        let mut builder = HeapBuilder::new_build(100);
        let inner_tuple = Term::Tuple(vec![Term::Small(1), Term::Small(2)]);
        let outer_tuple = erts_bld_tuple(&mut builder, vec![inner_tuple, Term::Small(3)]).unwrap();
        match outer_tuple {
            Term::Tuple(elements) => {
                assert_eq!(elements.len(), 2);
            }
            _ => panic!("Expected Tuple"),
        }
    }
}

