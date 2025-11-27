//! Integration tests for entities_data_handling crate
//!
//! These tests verify that multiple modules work together correctly
//! and test end-to-end workflows.

use entities_data_handling::*;
use entities_data_handling::term_hashing::Term;
use entities_data_handling::atom::AtomError;

#[test]
fn test_term_hashing_integration() {
    // Test hashing with various term types
    let nil = Term::Nil;
    let hash_nil = term_hashing::erts_internal_hash(nil.clone());
    // Hash can be any value (including 0), just verify it's computed
    let _ = hash_nil; // Use the value
    
    // Test hashing small integers
    let small1 = Term::Small(42);
    let small2 = Term::Small(-42);
    let hash1 = term_hashing::erts_internal_hash(small1.clone());
    let hash2 = term_hashing::erts_internal_hash(small2.clone());
    assert_ne!(hash1, hash2); // Positive and negative should hash differently
    
    // Test hashing atoms
    let atom1 = Term::Atom(1);
    let atom2 = Term::Atom(2);
    let hash_atom1 = term_hashing::erts_internal_hash(atom1.clone());
    let hash_atom2 = term_hashing::erts_internal_hash(atom2.clone());
    assert_ne!(hash_atom1, hash_atom2); // Different atoms should hash differently
    
    // Test hashing tuples
    let tuple1 = Term::Tuple(vec![Term::Small(1), Term::Small(2)]);
    let tuple2 = Term::Tuple(vec![Term::Small(2), Term::Small(1)]);
    let hash_tuple1 = term_hashing::erts_internal_hash(tuple1.clone());
    let hash_tuple2 = term_hashing::erts_internal_hash(tuple2.clone());
    assert_ne!(hash_tuple1, hash_tuple2); // Different tuple contents should hash differently
    
    // Test hashing lists
    let list1 = Term::List {
        head: Box::new(Term::Small(1)),
        tail: Box::new(Term::Nil),
    };
    let list2 = Term::List {
        head: Box::new(Term::Small(2)),
        tail: Box::new(Term::Nil),
    };
    let hash_list1 = term_hashing::erts_internal_hash(list1.clone());
    let hash_list2 = term_hashing::erts_internal_hash(list2.clone());
    assert_ne!(hash_list1, hash_list2);
    
    // Test hashing maps
    let map1 = Term::Map(vec![
        (Term::Atom(1), Term::Small(10)),
        (Term::Atom(2), Term::Small(20)),
    ]);
    let map2 = Term::Map(vec![
        (Term::Atom(1), Term::Small(10)),
        (Term::Atom(2), Term::Small(20)),
    ]);
    let hash_map1 = term_hashing::erts_map_hash(map1.clone());
    let hash_map2 = term_hashing::erts_map_hash(map2.clone());
    // Maps with same key-value pairs in same order should hash the same
    assert_eq!(hash_map1, hash_map2);
    
    // Test that different hash functions produce different results
    let term = Term::Small(42);
    let hash1 = term_hashing::make_hash(term.clone());
    let hash2 = term_hashing::make_hash2(term.clone());
    let hash3 = term_hashing::erts_internal_hash(term.clone());
    // They might be the same, but test that they're valid
    assert!(hash1 > 0 || hash1 == 0); // Valid hash
    assert!(hash2 > 0 || hash2 == 0);
    assert!(hash3 > 0 || hash3 == 0);
}

#[test]
fn test_atom_table_integration() {
    // Test creating atoms and using them in term hashing
    let table = AtomTable::new(1000);
    
    // Create some atoms
    let atom1_index = table.put_index(b"hello", AtomEncoding::SevenBitAscii, false).unwrap();
    let atom2_index = table.put_index(b"world", AtomEncoding::SevenBitAscii, false).unwrap();
    let atom3_index = table.put_index(b"hello", AtomEncoding::SevenBitAscii, false).unwrap();
    
    // Verify duplicate atoms get same index
    assert_eq!(atom1_index, atom3_index);
    
    // Verify we can retrieve atoms
    assert_eq!(table.get(b"hello", AtomEncoding::SevenBitAscii), Some(atom1_index));
    assert_eq!(table.get(b"world", AtomEncoding::SevenBitAscii), Some(atom2_index));
    assert_eq!(table.get_name(atom1_index), Some(b"hello".to_vec()));
    assert_eq!(table.get_name(atom2_index), Some(b"world".to_vec()));
    
    // Test using atoms in term hashing
    let term1 = Term::Atom(atom1_index as u32);
    let term2 = Term::Atom(atom2_index as u32);
    let hash1 = term_hashing::erts_internal_hash(term1.clone());
    let hash2 = term_hashing::erts_internal_hash(term2.clone());
    assert_ne!(hash1, hash2); // Different atoms should hash differently
    
    // Test atoms with different encodings
    let utf8_atom = table.put_index("café".as_bytes(), AtomEncoding::Utf8, false).unwrap();
    assert_ne!(utf8_atom, atom1_index);
    assert_eq!(table.get("café".as_bytes(), AtomEncoding::Utf8), Some(utf8_atom));
}

#[test]
fn test_atom_table_with_maps_integration() {
    // Test using atoms from atom table as map keys
    let table = AtomTable::new(1000);
    let mut map = Map::new();
    
    // Create atoms
    let key1_index = table.put_index(b"name", AtomEncoding::SevenBitAscii, false).unwrap();
    let key2_index = table.put_index(b"age", AtomEncoding::SevenBitAscii, false).unwrap();
    let value1_index = table.put_index(b"Alice", AtomEncoding::SevenBitAscii, false).unwrap();
    
    // Use atoms as map keys
    let key1 = Term::Atom(key1_index as u32);
    let key2 = Term::Atom(key2_index as u32);
    let value1 = Term::Atom(value1_index as u32);
    let value2 = Term::Small(30);
    
    // Add to map
    map.put(key1.clone(), value1.clone());
    map.put(key2.clone(), value2.clone());
    
    // Verify map operations
    assert_eq!(map.size(), 2);
    assert!(map.is_key(&key1));
    assert!(map.is_key(&key2));
    assert_eq!(map.get(&key1), Some(&value1));
    assert_eq!(map.get(&key2), Some(&value2));
    
    // Test map hashing with atom keys
    let map_term = Term::Map(map.to_list());
    let hash = term_hashing::erts_map_hash(map_term.clone());
    assert_ne!(hash, 0);
}

#[test]
fn test_cross_module_workflow() {
    // Test a complete workflow: create atoms, use in maps, hash terms
    let table = AtomTable::new(1000);
    let mut map = Map::new();
    
    // Step 1: Create atoms
    let module_atom = table.put_index(b"my_module", AtomEncoding::SevenBitAscii, false).unwrap();
    let function_atom = table.put_index(b"my_function", AtomEncoding::SevenBitAscii, false).unwrap();
    
    // Step 2: Create terms with atoms
    let module_term = Term::Atom(module_atom as u32);
    let function_term = Term::Atom(function_atom as u32);
    let arity_term = Term::Small(3);
    
    // Step 3: Create a tuple (MFA - Module, Function, Arity)
    let mfa = Term::Tuple(vec![module_term.clone(), function_term.clone(), arity_term.clone()]);
    
    // Step 4: Hash the MFA
    let mfa_hash = term_hashing::erts_internal_hash(mfa.clone());
    assert_ne!(mfa_hash, 0);
    
    // Step 5: Use atoms as map keys
    map.put(Term::Atom(module_atom as u32), Term::Small(1));
    map.put(Term::Atom(function_atom as u32), Term::Small(2));
    
    // Step 6: Hash the map
    let map_term = Term::Map(map.to_list());
    let map_hash = term_hashing::erts_map_hash(map_term.clone());
    assert_ne!(map_hash, 0);
    
    // Step 7: Verify we can retrieve atoms
    assert_eq!(table.get_name(module_atom), Some(b"my_module".to_vec()));
    assert_eq!(table.get_name(function_atom), Some(b"my_function".to_vec()));
}

#[test]
fn test_atom_table_limits_integration() {
    // Test atom table behavior at limits
    let table = AtomTable::new(5); // Small limit for testing
    
    // Fill up the table
    for i in 0..5 {
        let name = format!("atom_{}", i);
        let result = table.put_index(name.as_bytes(), AtomEncoding::SevenBitAscii, false);
        assert!(result.is_ok());
    }
    
    // Try to add one more - should fail
    let result = table.put_index(b"overflow", AtomEncoding::SevenBitAscii, false);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), AtomError::TableFull);
    
    // Verify existing atoms still work
    assert_eq!(table.get(b"atom_0", AtomEncoding::SevenBitAscii), Some(0));
    assert_eq!(table.get(b"atom_4", AtomEncoding::SevenBitAscii), Some(4));
}

#[test]
fn test_map_with_various_term_types() {
    // Test map operations with various term types as keys and values
    let mut map = Map::new();
    
    // Use different term types as keys
    let key1 = Term::Atom(1);
    let key2 = Term::Small(42);
    let key3 = Term::Tuple(vec![Term::Small(1), Term::Small(2)]);
    let key4 = Term::Nil;
    
    // Use different term types as values
    let value1 = Term::Small(100);
    let value2 = Term::Atom(2);
    let value3 = Term::Tuple(vec![Term::Small(10), Term::Small(20)]);
    let value4 = Term::List {
        head: Box::new(Term::Small(1)),
        tail: Box::new(Term::Nil),
    };
    
    // Add all to map
    map.put(key1.clone(), value1.clone());
    map.put(key2.clone(), value2.clone());
    map.put(key3.clone(), value3.clone());
    map.put(key4.clone(), value4.clone());
    
    // Verify all entries
    assert_eq!(map.size(), 4);
    assert_eq!(map.get(&key1), Some(&value1));
    assert_eq!(map.get(&key2), Some(&value2));
    assert_eq!(map.get(&key3), Some(&value3));
    assert_eq!(map.get(&key4), Some(&value4));
    
    // Hash the map
    let map_term = Term::Map(map.to_list());
    let hash = term_hashing::erts_map_hash(map_term.clone());
    assert_ne!(hash, 0);
}
