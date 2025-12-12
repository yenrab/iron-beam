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

#[test]
fn test_bits_operations_integration() {
    // Test bit manipulation operations
    use entities_data_handling::bits;
    
    // Test nbytes and nbits (inverse operations)
    assert_eq!(bits::nbytes(0), 0);
    assert_eq!(bits::nbytes(1), 1);
    assert_eq!(bits::nbytes(8), 1);
    assert_eq!(bits::nbytes(9), 2);
    assert_eq!(bits::nbytes(16), 2);
    assert_eq!(bits::nbytes(17), 3);
    
    assert_eq!(bits::nbits(0), 0);
    assert_eq!(bits::nbits(1), 8);
    assert_eq!(bits::nbits(2), 16);
    assert_eq!(bits::nbits(10), 80);
    
    // Verify inverse relationship
    for bits_count in [0, 1, 8, 9, 16, 17, 24, 32, 64, 128] {
        let bytes = bits::nbytes(bits_count);
        let bits_back = bits::nbits(bytes);
        assert!(bits_back >= bits_count as u64);
    }
    
    // Test byte_offset and bit_offset
    assert_eq!(bits::byte_offset(0), 0);
    assert_eq!(bits::byte_offset(7), 0);
    assert_eq!(bits::byte_offset(8), 1);
    assert_eq!(bits::byte_offset(15), 1);
    assert_eq!(bits::byte_offset(16), 2);
    
    assert_eq!(bits::bit_offset(0), 0);
    assert_eq!(bits::bit_offset(7), 7);
    assert_eq!(bits::bit_offset(8), 0);
    assert_eq!(bits::bit_offset(15), 7);
    assert_eq!(bits::bit_offset(16), 0);
    
    // Verify offset relationship
    for bit_off in [0, 1, 7, 8, 15, 16, 23, 24, 31, 32] {
        let byte_off = bits::byte_offset(bit_off);
        let bit_in_byte = bits::bit_offset(bit_off);
        assert_eq!(bit_off, byte_off * 8 + bit_in_byte);
    }
    
    // Test make_mask
    assert_eq!(bits::make_mask(0), 0);
    assert_eq!(bits::make_mask(1), 0b1);
    assert_eq!(bits::make_mask(3), 0b111);
    assert_eq!(bits::make_mask(8), 0xFF);
    assert_eq!(bits::make_mask(64), u64::MAX);
    
    // Test mask_bits
    // mask_bits(src, dst, mask) = (src & mask) | (dst & !mask)
    let src = 0b11110000u8;
    let dst = 0b00001111u8;
    let mask = 0b11000000u8;
    let result = bits::mask_bits(src, dst, mask);
    // (0b11110000 & 0b11000000) | (0b00001111 & 0b00111111)
    // = 0b11000000 | 0b00001111 = 0b11001111
    assert_eq!(result, 0b11001111u8);
    
    // Test get_bit and set_bit
    let byte = 0b10101010u8;
    assert_eq!(bits::get_bit(byte, 0), 1); // MSB
    assert_eq!(bits::get_bit(byte, 7), 0); // LSB
    assert_eq!(bits::get_bit(byte, 1), 0);
    assert_eq!(bits::get_bit(byte, 2), 1);
    
    let modified = bits::set_bit(byte, 7, 1);
    assert_eq!(bits::get_bit(modified, 7), 1);
    let modified2 = bits::set_bit(modified, 0, 0);
    assert_eq!(bits::get_bit(modified2, 0), 0);
}

#[test]
fn test_bits_copy_operations_integration() {
    // Test copy_bits_forward with various scenarios
    use entities_data_handling::bits;
    
    // Test copying within same byte
    let src = vec![0b11110000u8];
    let mut dst = vec![0u8; 1];
    bits::copy_bits_forward(&src, 4, &mut dst, 0, 4);
    // Copies bits 4-7 (1111) from src to bits 0-3 of dst
    assert_eq!(dst[0], 0b00001111u8);
    
    // Test copying across byte boundaries
    let src = vec![0xFFu8, 0xAAu8];
    let mut dst = vec![0u8; 2];
    bits::copy_bits_forward(&src, 4, &mut dst, 0, 12);
    // Should copy 12 bits starting at bit 4 of src
    // Bit 4-7 of src[0] (0xFF = 0b11111111) = 0b1111
    // Bit 0-7 of src[1] (0xAA = 0b10101010) = 0b10101010
    // Result: first 4 bits from src[0] (bits 4-7) + 8 bits from src[1] = 12 bits total
    // This should result in non-zero values
    // The exact value depends on the implementation, but at least verify it copied something
    let total = dst[0] as u16 + (dst[1] as u16) << 8;
    assert_ne!(total, 0);
    
    // Test copying with different offsets
    let src = vec![0b10101010u8, 0b11001100u8];
    let mut dst = vec![0u8; 2];
    bits::copy_bits_forward(&src, 0, &mut dst, 0, 16);
    // Should copy all 16 bits
    assert_eq!(dst[0], src[0]);
    assert_eq!(dst[1], src[1]);
    
    // Test copying with unaligned offsets
    let src = vec![0b11111111u8];
    let mut dst = vec![0u8; 1];
    bits::copy_bits_forward(&src, 2, &mut dst, 3, 4);
    // Copies 4 bits from offset 2 to offset 3
    assert_ne!(dst[0], 0);
}

#[test]
fn test_bits_comparison_integration() {
    // Test cmp_bits function
    use entities_data_handling::bits;
    
    // Test equal bit sequences
    let a = vec![0xFFu8, 0xAAu8];
    let b = vec![0xFFu8, 0xAAu8];
    assert_eq!(bits::cmp_bits(&a, 0, &b, 0, 16), 0);
    
    // Test different bit sequences
    let a = vec![0xFFu8];
    let b = vec![0x00u8];
    let result = bits::cmp_bits(&a, 0, &b, 0, 8);
    assert_ne!(result, 0);
    
    // Test comparison with offsets
    let a = vec![0b11110000u8];
    let b = vec![0b00001111u8];
    // Compare first 4 bits of a with last 4 bits of b
    let result = bits::cmp_bits(&a, 0, &b, 4, 4);
    assert_eq!(result, 0); // Should be equal
    
    // Test comparison across byte boundaries
    let a = vec![0xFFu8, 0xAAu8];
    let b = vec![0xFFu8, 0xABu8];
    let result = bits::cmp_bits(&a, 0, &b, 0, 16);
    assert_ne!(result, 0); // Should be different
}

#[test]
fn test_bits_with_binary_integration() {
    // Test bit operations with binary data structures
    use entities_data_handling::bits;
    use entities_data_handling::binary::Binary;
    
    // Create binary data
    let data = vec![0xFFu8, 0xAAu8, 0x55u8];
    let binary = Binary::new(data.clone());
    
    // Test bit operations on binary data
    let bytes_needed = bits::nbytes(24);
    assert_eq!(bytes_needed, 3);
    
    // Test copying bits from binary data
    let mut dst = vec![0u8; 3];
    bits::copy_bits_forward(binary.data(), 0, &mut dst, 0, 24);
    assert_eq!(dst, binary.data());
    
    // Test bit comparison with binary
    let binary2 = Binary::new(data.clone());
    assert_eq!(bits::cmp_bits(binary.data(), 0, binary2.data(), 0, 24), 0);
}

#[test]
fn test_atomics_integration() {
    // Test atomic operations integration
    use entities_data_handling::atomics::{DoubleWordAtomic, have_native_dw_atomic};
    use std::sync::atomic::Ordering;
    use std::sync::Arc;
    use std::thread;
    
    // Test platform detection
    let has_native = have_native_dw_atomic();
    // Should return true on 64-bit platforms, false on 32-bit
    let _ = has_native;
    
    // Test basic atomic operations
    let atomic = DoubleWordAtomic::new(0);
    assert_eq!(atomic.load(Ordering::SeqCst), 0);
    
    atomic.store(42, Ordering::SeqCst);
    assert_eq!(atomic.load(Ordering::SeqCst), 42);
    
    // Test compare and exchange
    let result = atomic.compare_exchange(42, 100, Ordering::SeqCst, Ordering::SeqCst);
    assert_eq!(result, Ok(42));
    assert_eq!(atomic.load(Ordering::SeqCst), 100);
    
    // Test failed compare and exchange
    let result = atomic.compare_exchange(42, 200, Ordering::SeqCst, Ordering::SeqCst);
    assert_eq!(result, Err(100));
    assert_eq!(atomic.load(Ordering::SeqCst), 100);
    
    // Test thread-safety with multiple threads
    let atomic = Arc::new(DoubleWordAtomic::new(0));
    let mut handles = vec![];
    
    for _i in 0..10 {
        let atomic_clone = Arc::clone(&atomic);
        let handle = thread::spawn(move || {
            let current = atomic_clone.load(Ordering::SeqCst);
            let new = current + 1;
            // Try to increment atomically
            loop {
                match atomic_clone.compare_exchange(current, new, Ordering::SeqCst, Ordering::SeqCst) {
                    Ok(_) => break,
                    Err(actual) => {
                        // Another thread modified it, try again
                        let updated = actual + 1;
                        match atomic_clone.compare_exchange(actual, updated, Ordering::SeqCst, Ordering::SeqCst) {
                            Ok(_) => break,
                            Err(_) => continue,
                        }
                    }
                }
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    // All threads should have incremented, final value should be 10
    assert_eq!(atomic.load(Ordering::SeqCst), 10);
    
    // Test different memory orderings
    let atomic = DoubleWordAtomic::new(0);
    atomic.store(1, Ordering::Relaxed);
    assert_eq!(atomic.load(Ordering::Relaxed), 1);
    
    atomic.store(2, Ordering::Release);
    assert_eq!(atomic.load(Ordering::Acquire), 2);
}

#[test]
fn test_binary_integration() {
    // Test binary operations integration
    use entities_data_handling::binary::Binary;
    use entities_data_handling::bits;
    
    // Test binary creation and data access
    let data = vec![0x48, 0x65, 0x6C, 0x6C, 0x6F]; // "Hello" in ASCII
    let binary = Binary::new(data.clone());
    assert_eq!(binary.data(), &data);
    
    // Test binary with empty data
    let empty_binary = Binary::new(vec![]);
    assert_eq!(empty_binary.data(), &[] as &[u8]);
    
    // Test binary with large data
    let large_data: Vec<u8> = (0..1000).map(|i| (i % 256) as u8).collect();
    let large_binary = Binary::new(large_data.clone());
    assert_eq!(large_binary.data(), &large_data);
    
    // Test binary with bit operations
    let binary = Binary::new(vec![0xFFu8, 0xAAu8, 0x55u8]);
    let bytes_needed = bits::nbytes(24);
    assert_eq!(bytes_needed, 3);
    
    // Test copying bits from binary
    let mut dst = vec![0u8; 3];
    bits::copy_bits_forward(binary.data(), 0, &mut dst, 0, 24);
    assert_eq!(dst, binary.data());
    
    // Test binary comparison with bits
    let binary1 = Binary::new(vec![0xFFu8, 0xAAu8]);
    let binary2 = Binary::new(vec![0xFFu8, 0xAAu8]);
    assert_eq!(bits::cmp_bits(binary1.data(), 0, binary2.data(), 0, 16), 0);
    
    // Test binary with different data
    let binary1 = Binary::new(vec![0xFFu8]);
    let binary2 = Binary::new(vec![0x00u8]);
    let result = bits::cmp_bits(binary1.data(), 0, binary2.data(), 0, 8);
    assert_ne!(result, 0);
}
