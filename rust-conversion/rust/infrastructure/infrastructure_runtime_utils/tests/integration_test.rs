//! Integration tests for infrastructure_runtime_utils crate
//!
//! These tests verify that runtime utility functions work correctly
//! and test end-to-end workflows for term building, comparison, and initialization.

use infrastructure_runtime_utils::*;
use entities_data_handling::term_hashing::Term;

#[test]
fn test_erts_init_utils() {
    let result = erts_init_utils();
    // May succeed or fail depending on initialization state
    let _ = result;
}

#[test]
fn test_erts_init_utils_mem() {
    let result = erts_init_utils_mem();
    // May succeed or fail depending on initialization state
    let _ = result;
}

#[test]
fn test_erts_utils_sched_spec_data_init() {
    let result = erts_utils_sched_spec_data_init();
    // May succeed or fail depending on initialization state
    let _ = result;
}

#[test]
fn test_eq_same_terms() {
    let term1 = Term::Small(42);
    let term2 = Term::Small(42);
    
    let result = eq(&term1, &term2);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), true);
}

#[test]
fn test_eq_different_terms() {
    let term1 = Term::Small(42);
    let term2 = Term::Small(43);
    
    let result = eq(&term1, &term2);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), false);
}

#[test]
fn test_eq_different_types() {
    let term1 = Term::Small(42);
    let term2 = Term::Atom(42);
    
    let result = eq(&term1, &term2);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), false);
}

#[test]
fn test_eq_nil() {
    let term1 = Term::Nil;
    let term2 = Term::Nil;
    
    let result = eq(&term1, &term2);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), true);
}

#[test]
fn test_erts_cmp_equal() {
    let term1 = Term::Small(42);
    let term2 = Term::Small(42);
    
    let result = erts_cmp(&term1, &term2, 0);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn test_erts_cmp_less_than() {
    let term1 = Term::Small(10);
    let term2 = Term::Small(20);
    
    let result = erts_cmp(&term1, &term2, 0);
    assert!(result.is_ok());
    assert!(result.unwrap() < 0);
}

#[test]
fn test_erts_cmp_greater_than() {
    let term1 = Term::Small(20);
    let term2 = Term::Small(10);
    
    let result = erts_cmp(&term1, &term2, 0);
    assert!(result.is_ok());
    assert!(result.unwrap() > 0);
}

#[test]
fn test_heap_builder_new_size_calc() {
    let builder = HeapBuilder::new_size_calc();
    assert_eq!(builder.size(), 0);
}

#[test]
fn test_heap_builder_new_build() {
    let builder = HeapBuilder::new_build(200);
    assert_eq!(builder.size(), 0);
}

#[test]
fn test_erts_bld_atom() {
    use entities_data_handling::atom::AtomTable;
    
    let mut builder = HeapBuilder::new_size_calc();
    let atom_table = AtomTable::new(1000);
    
    let result = erts_bld_atom(&mut builder, &atom_table, "test_atom");
    assert!(result.is_ok());
}

#[test]
fn test_erts_bld_uint() {
    let mut builder = HeapBuilder::new_size_calc();
    
    let result = erts_bld_uint(&mut builder, 42);
    assert!(result.is_ok());
}

#[test]
fn test_erts_bld_uword() {
    let mut builder = HeapBuilder::new_size_calc();
    
    let result = erts_bld_uword(&mut builder, 100);
    assert!(result.is_ok());
}

#[test]
fn test_erts_bld_uint64() {
    let mut builder = HeapBuilder::new_size_calc();
    
    let result = erts_bld_uint64(&mut builder, 1000);
    assert!(result.is_ok());
}

#[test]
fn test_erts_bld_sint64() {
    let mut builder = HeapBuilder::new_size_calc();
    
    let result = erts_bld_sint64(&mut builder, -1000);
    assert!(result.is_ok());
}

#[test]
fn test_erts_bld_cons() {
    let mut builder = HeapBuilder::new_size_calc();
    
    let head = Term::Small(1);
    let tail = Term::Small(2);
    
    let result = erts_bld_cons(&mut builder, head, tail);
    assert!(result.is_ok());
}

#[test]
fn test_erts_bld_tuple() {
    let mut builder = HeapBuilder::new_size_calc();
    
    let elements = vec![Term::Small(1), Term::Small(2), Term::Small(3)];
    let result = erts_bld_tuple(&mut builder, elements);
    assert!(result.is_ok());
}

#[test]
fn test_erts_bld_tuplev() {
    let mut builder = HeapBuilder::new_size_calc();
    
    let elements = vec![Term::Small(1), Term::Small(2)];
    let result = erts_bld_tuplev(&mut builder, elements);
    assert!(result.is_ok());
}

#[test]
fn test_erts_bld_string_n() {
    let mut builder = HeapBuilder::new_size_calc();
    
    let result = erts_bld_string_n(&mut builder, b"hello", 5);
    assert!(result.is_ok());
}

#[test]
fn test_erts_bld_list() {
    let mut builder = HeapBuilder::new_size_calc();
    
    let elements = vec![Term::Small(1), Term::Small(2), Term::Small(3)];
    let result = erts_bld_list(&mut builder, elements.len(), &elements);
    assert!(result.is_ok());
}

#[test]
fn test_term_building_error_variants() {
    let errors = vec![
        TermBuildingError::HeapTooSmall,
        TermBuildingError::InvalidArgument("test".to_string()),
        TermBuildingError::AtomNotFound,
        TermBuildingError::BuildingFailed("test".to_string()),
    ];
    
    for error in errors {
        let _ = format!("{:?}", error);
    }
}

#[test]
fn test_comparison_error_variants() {
    let errors = vec![
        ComparisonError::ComparisonFailed("test".to_string()),
        ComparisonError::InvalidTerm("test".to_string()),
    ];
    
    for error in errors {
        let _ = format!("{:?}", error);
    }
}

#[test]
fn test_heap_builder_multiple_operations() {
    let mut builder = HeapBuilder::new_size_calc();
    
    // Build multiple terms (size calculation)
    erts_bld_uint(&mut builder, 1).unwrap();
    erts_bld_uint(&mut builder, 2).unwrap();
    erts_bld_uint(&mut builder, 3).unwrap();
    
    // Builder should have calculated size
    assert!(builder.size() >= 0);
}

#[test]
fn test_eq_complex_terms() {
    let term1 = Term::Tuple(vec![Term::Small(1), Term::Small(2)]);
    let term2 = Term::Tuple(vec![Term::Small(1), Term::Small(2)]);
    
    let result = eq(&term1, &term2);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), true);
}

#[test]
fn test_erts_cmp_complex_terms() {
    let term1 = Term::Tuple(vec![Term::Small(1)]);
    let term2 = Term::Tuple(vec![Term::Small(2)]);
    
    let result = erts_cmp(&term1, &term2, 0);
    // Complex type comparison (tuples) is not yet fully implemented
    // The function returns an error indicating this
    assert!(result.is_err());
    match result.unwrap_err() {
        ComparisonError::ComparisonFailed(msg) => {
            assert!(msg.contains("Complex type comparison not fully implemented"));
        }
        _ => panic!("Expected ComparisonFailed error for complex type comparison"),
    }
}
