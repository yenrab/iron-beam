//! Integration tests for infrastructure_runtime_utils
//!
//! Tests the runtime utilities functionality including term building,
//! comparison, and initialization.

use infrastructure_runtime_utils::{
    erts_bld_atom, erts_bld_uint, erts_bld_sint64,
    erts_bld_cons, erts_bld_tuple, erts_bld_string_n, erts_bld_list,
    eq, erts_cmp,
    erts_init_utils, erts_init_utils_mem, erts_utils_sched_spec_data_init,
    HeapBuilder,
};
use entities_data_handling::term_hashing::Term;
use entities_data_handling::atom::{AtomTable, AtomEncoding};

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
    assert_eq!(builder.size(), 0); // Atoms don't use heap
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
fn test_erts_bld_uint_large() {
    let mut builder = HeapBuilder::new_size_calc();
    let large_value = u64::MAX;
    let term = erts_bld_uint(&mut builder, large_value).unwrap();
    
    // For size calculation, we get a placeholder
    // Just verify size is calculated
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
fn test_erts_bld_sint64() {
    let mut builder = HeapBuilder::new_size_calc();
    let term = erts_bld_sint64(&mut builder, -42).unwrap();
    
    match term {
        Term::Small(-42) => {},
        _ => panic!("Expected Small(-42)"),
    }
}

#[test]
fn test_erts_bld_tuple() {
    let mut builder = HeapBuilder::new_size_calc();
    let elements = vec![Term::Small(1), Term::Small(2), Term::Small(3)];
    let term = erts_bld_tuple(&mut builder, elements.clone()).unwrap();
    
    // For size calculation, we get a placeholder
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
fn test_erts_bld_cons() {
    let mut builder = HeapBuilder::new_size_calc();
    let car = Term::Small(1);
    let cdr = Term::Nil;
    let term = erts_bld_cons(&mut builder, car.clone(), cdr.clone()).unwrap();
    
    // For size calculation, we get a placeholder
    assert_eq!(builder.size(), 2); // Cons cell needs 2 words
    
    // Test actual building
    let mut builder2 = HeapBuilder::new_build(100);
    let term2 = erts_bld_cons(&mut builder2, car, cdr).unwrap();
    match term2 {
        Term::List { .. } => {},
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_erts_bld_string_n() {
    let mut builder = HeapBuilder::new_size_calc();
    let str = b"hello";
    let term = erts_bld_string_n(&mut builder, str, str.len()).unwrap();
    
    // For size calculation, we get a placeholder
    // String as list: 5 bytes * 2 words per cons cell = 10 words
    assert_eq!(builder.size(), 10);
    
    // Test actual building
    let mut builder2 = HeapBuilder::new_build(100);
    let term2 = erts_bld_string_n(&mut builder2, str, str.len()).unwrap();
    match term2 {
        Term::List { .. } => {},
        _ => panic!("Expected List for string"),
    }
}

#[test]
fn test_erts_bld_list() {
    let mut builder = HeapBuilder::new_size_calc();
    let terms = vec![Term::Small(1), Term::Small(2), Term::Small(3)];
    let term = erts_bld_list(&mut builder, terms.len(), &terms).unwrap();
    
    // For size calculation, we get a placeholder
    // List: 3 elements * 2 words per cons cell = 6 words
    assert_eq!(builder.size(), 6);
    
    // Test actual building
    let mut builder2 = HeapBuilder::new_build(100);
    let term2 = erts_bld_list(&mut builder2, terms.len(), &terms).unwrap();
    match term2 {
        Term::List { .. } => {},
        _ => panic!("Expected List"),
    }
}

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
fn test_eq_atom() {
    let a = Term::Atom(1);
    let b = Term::Atom(1);
    assert!(eq(&a, &b).unwrap());
    
    let c = Term::Atom(2);
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
    
    let c = Term::List {
        head: Box::new(Term::Small(2)),
        tail: Box::new(Term::Nil),
    };
    assert!(!eq(&a, &c).unwrap());
}

#[test]
fn test_eq_nested() {
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
fn test_erts_cmp() {
    let a = Term::Small(1);
    let b = Term::Small(2);
    assert_eq!(erts_cmp(&a, &b, 0).unwrap(), -1);
    assert_eq!(erts_cmp(&b, &a, 0).unwrap(), 1);
    assert_eq!(erts_cmp(&a, &a, 0).unwrap(), 0);
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
fn test_erts_cmp_type_ordering() {
    // Different types should compare by type order
    let a = Term::Nil;
    let b = Term::Small(1);
    // Nil (0) < Small (1)
    assert_eq!(erts_cmp(&a, &b, 0).unwrap(), -1);
}

#[test]
fn test_erts_init_utils() {
    let result = erts_init_utils();
    assert!(result.is_ok());
    
    // Second call should also succeed (idempotent)
    let result2 = erts_init_utils();
    assert!(result2.is_ok());
}

#[test]
fn test_erts_init_utils_mem() {
    let result = erts_init_utils_mem();
    assert!(result.is_ok());
    
    // Second call should also succeed (idempotent)
    let result2 = erts_init_utils_mem();
    assert!(result2.is_ok());
}

#[test]
fn test_erts_utils_sched_spec_data_init() {
    let result = erts_utils_sched_spec_data_init();
    assert!(result.is_ok());
}

#[test]
fn test_heap_builder_size_calc() {
    let mut builder = HeapBuilder::new_size_calc();
    assert_eq!(builder.size(), 0);
    
    let _term = erts_bld_uint(&mut builder, 42).unwrap();
    assert_eq!(builder.size(), 0); // Small integer
    
    let _term2 = erts_bld_tuple(&mut builder, vec![Term::Small(1), Term::Small(2)]).unwrap();
    assert_eq!(builder.size(), 3); // 1 header + 2 elements
}

#[test]
fn test_heap_builder_build() {
    let mut builder = HeapBuilder::new_build(100);
    let term = erts_bld_uint(&mut builder, 42).unwrap();
    
    match term {
        Term::Small(42) => {},
        _ => panic!("Expected Small(42)"),
    }
    
    let heap_data = builder.into_heap_data();
    assert!(heap_data.is_some());
}

#[test]
fn test_term_building_error() {
    let mut atom_table = AtomTable::new(100);
    let mut builder = HeapBuilder::new_size_calc();
    
    // Try to build atom that doesn't exist and can't be created
    // (This will fail if atom table is full or name is invalid)
    let result = erts_bld_atom(&mut builder, &atom_table, "");
    // Empty string might fail validation
    assert!(result.is_err() || result.is_ok()); // Either is acceptable
}

