//! Integration tests for infrastructure_nif_api crate
//!
//! These tests verify that NIF API functions work correctly
//! and test end-to-end workflows for term creation, decoding, error handling, and resource management.

use infrastructure_nif_api::*;
use entities_process::{Process, ProcessId};
use std::sync::Arc;
use infrastructure_utilities::process_table::get_global_process_table;
use infrastructure_nif_api::resource_management::ErlNifResourceType;

#[test]
fn test_nif_env_creation() {
    // Create a process and register it
    let process = Arc::new(Process::new(1));
    let table = get_global_process_table();
    table.insert(1, Arc::clone(&process));
    
    // Create NIF environment from process ID
    let env = NifEnv::from_process_id(1);
    assert!(env.is_some());
    let env = env.unwrap();
    assert_eq!(env.process_id(), 1);
    
    // Create NIF environment from process reference
    let env2 = NifEnv::from_process(Arc::clone(&process));
    assert_eq!(env2.process_id(), 1);
}

#[test]
fn test_nif_env_nonexistent_process() {
    // Try to create NIF environment from non-existent process
    let env = NifEnv::from_process_id(99999);
    assert!(env.is_none());
}

#[test]
fn test_nif_env_heap_operations() {
    let process = Arc::new(Process::new(2));
    let env = NifEnv::from_process(process);
    
    // Test heap allocation
    let index = env.allocate_heap(10);
    assert!(index.is_some());
    
    // Test available heap space
    let available = env.available_heap_space();
    assert!(available > 0);
    
    // Test heap top index
    let heap_top = env.heap_top_index();
    assert!(heap_top >= 0);
}

#[test]
fn test_enif_make_atom() {
    let process = Arc::new(Process::new(3));
    let env = NifEnv::from_process(process);
    
    // Create atom from string
    let atom_term = enif_make_atom(&env, "test");
    assert!(atom_term != 0); // Should not be zero (nil)
    
    // Create another atom
    let atom_term2 = enif_make_atom(&env, "hello");
    assert!(atom_term2 != 0);
    assert_ne!(atom_term, atom_term2); // Different atoms should have different terms
}

#[test]
fn test_enif_make_atom_len() {
    let process = Arc::new(Process::new(4));
    let env = NifEnv::from_process(process);
    
    // Create atom with Latin1 encoding
    let atom_term = enif_make_atom_len(&env, b"test", NifCharEncoding::Latin1);
    assert!(atom_term != 0);
    
    // Create atom with UTF-8 encoding
    let atom_term2 = enif_make_atom_len(&env, b"test", NifCharEncoding::Utf8);
    assert!(atom_term2 != 0);
}

#[test]
fn test_enif_make_int() {
    let process = Arc::new(Process::new(5));
    let env = NifEnv::from_process(process);
    
    // Create integer terms
    let int_term1 = enif_make_int(&env, 0);
    let int_term2 = enif_make_int(&env, 42);
    let int_term3 = enif_make_int(&env, -42);
    let int_term4 = enif_make_int(&env, 2147483647); // i32::MAX
    let int_term5 = enif_make_int(&env, -2147483648); // i32::MIN
    
    assert!(int_term1 != 0);
    assert!(int_term2 != 0);
    assert!(int_term3 != 0);
    assert!(int_term4 != 0);
    assert!(int_term5 != 0);
    
    // Different values should produce different terms
    assert_ne!(int_term1, int_term2);
    assert_ne!(int_term2, int_term3);
}

#[test]
fn test_enif_make_long() {
    let process = Arc::new(Process::new(6));
    let env = NifEnv::from_process(process);
    
    // Create long integer terms
    let long_term1 = enif_make_long(&env, 0);
    let long_term2 = enif_make_long(&env, 42);
    let long_term3 = enif_make_long(&env, -42);
    let long_term4 = enif_make_long(&env, 9223372036854775807i64); // i64::MAX
    let long_term5 = enif_make_long(&env, -9223372036854775808i64); // i64::MIN
    
    assert!(long_term1 != 0);
    assert!(long_term2 != 0);
    assert!(long_term3 != 0);
    assert!(long_term4 != 0);
    assert!(long_term5 != 0);
}

#[test]
fn test_enif_make_ulong() {
    let process = Arc::new(Process::new(7));
    let env = NifEnv::from_process(process);
    
    // Create unsigned long integer terms
    let ulong_term1 = enif_make_ulong(&env, 0);
    let ulong_term2 = enif_make_ulong(&env, 42);
    let ulong_term3 = enif_make_ulong(&env, 18446744073709551615u64); // u64::MAX
    
    assert!(ulong_term1 != 0);
    assert!(ulong_term2 != 0);
    assert!(ulong_term3 != 0);
}

#[test]
fn test_enif_make_binary() {
    let process = Arc::new(Process::new(8));
    let env = NifEnv::from_process(process);
    
    // Create binary terms
    let binary_term1 = enif_make_binary(&env, &[]);
    let binary_term2 = enif_make_binary(&env, b"hello");
    let binary_term3 = enif_make_binary(&env, &[0, 1, 2, 3, 255]);
    
    assert!(binary_term1 != 0);
    assert!(binary_term2 != 0);
    assert!(binary_term3 != 0);
}

#[test]
fn test_enif_make_string() {
    let process = Arc::new(Process::new(9));
    let env = NifEnv::from_process(process);
    
    // Create string terms
    let string_term1 = enif_make_string(&env, "", NifCharEncoding::Latin1);
    let string_term2 = enif_make_string(&env, "hello", NifCharEncoding::Latin1);
    let string_term3 = enif_make_string(&env, "world", NifCharEncoding::Utf8);
    
    assert!(string_term1 != 0);
    assert!(string_term2 != 0);
    assert!(string_term3 != 0);
}

#[test]
fn test_enif_make_tuple() {
    let process = Arc::new(Process::new(10));
    let env = NifEnv::from_process(process);
    
    // Create tuple terms
    let tuple_term1 = enif_make_tuple(&env, &[]);
    let tuple_term2 = enif_make_tuple(&env, &[enif_make_int(&env, 1)]);
    let tuple_term3 = enif_make_tuple(&env, &[
        enif_make_int(&env, 1),
        enif_make_int(&env, 2),
        enif_make_int(&env, 3),
    ]);
    
    // Note: term 0 is valid (heap_index 0). Nil is 0x3F, not 0.
    // Verify tuples can be decoded (this validates they're valid terms)
    assert!(enif_get_tuple(&env, tuple_term1).is_some());
    assert!(enif_get_tuple(&env, tuple_term2).is_some());
    assert!(enif_get_tuple(&env, tuple_term3).is_some());
}

#[test]
fn test_enif_make_list() {
    let process = Arc::new(Process::new(11));
    let env = NifEnv::from_process(process);
    
    // Create list terms
    let list_term1 = enif_make_list(&env, &[]);
    let list_term2 = enif_make_list(&env, &[enif_make_int(&env, 1)]);
    let list_term3 = enif_make_list(&env, &[
        enif_make_int(&env, 1),
        enif_make_int(&env, 2),
        enif_make_int(&env, 3),
    ]);
    
    assert!(list_term1 != 0);
    assert!(list_term2 != 0);
    assert!(list_term3 != 0);
}

#[test]
fn test_enif_make_list_cell() {
    let process = Arc::new(Process::new(12));
    let env = NifEnv::from_process(process);
    
    // Create list cell (head and tail)
    let head = enif_make_int(&env, 1);
    let tail = enif_make_list(&env, &[enif_make_int(&env, 2)]);
    let list_cell = enif_make_list_cell(&env, head, tail);
    
    assert!(list_cell != 0);
}

#[test]
fn test_encode_small_integer() {
    // Test encoding small integers
    let term1 = encode_small_integer(0);
    let term2 = encode_small_integer(42);
    let term3 = encode_small_integer(-42);
    
    assert!(term1 != 0);
    assert!(term2 != 0);
    assert!(term3 != 0);
    assert_ne!(term1, term2);
    assert_ne!(term2, term3);
}

#[test]
fn test_is_small_integer() {
    // Test small integer detection
    let small_int = encode_small_integer(42);
    assert!(is_small_integer(small_int));
    
    // Test with non-small integer (would need actual term creation)
    let process = Arc::new(Process::new(13));
    let env = NifEnv::from_process(process);
    let large_int = enif_make_long(&env, 1000000);
    // Large integers may or may not be small integers depending on encoding
    let _ = is_small_integer(large_int);
}

#[test]
fn test_decode_small_integer() {
    // Test decoding small integers
    let term1 = encode_small_integer(0);
    let term2 = encode_small_integer(42);
    let term3 = encode_small_integer(-42);
    
    assert_eq!(decode_small_integer(term1), 0);
    assert_eq!(decode_small_integer(term2), 42);
    assert_eq!(decode_small_integer(term3), -42);
}

#[test]
fn test_enif_get_atom() {
    let process = Arc::new(Process::new(14));
    let env = NifEnv::from_process(process);
    
    // Create atom and decode it
    let atom_term = enif_make_atom(&env, "test");
    let decoded = enif_get_atom(&env, atom_term);
    
    assert!(decoded.is_some());
    let (name, _encoding) = decoded.unwrap();
    assert_eq!(name, "test");
}

#[test]
fn test_enif_get_int() {
    let process = Arc::new(Process::new(15));
    let env = NifEnv::from_process(process);
    
    // Create integer and decode it
    let int_term = enif_make_int(&env, 42);
    let decoded = enif_get_int(&env, int_term);
    
    assert!(decoded.is_some());
    assert_eq!(decoded.unwrap(), 42);
}

#[test]
fn test_enif_get_ulong() {
    let process = Arc::new(Process::new(16));
    let env = NifEnv::from_process(process);
    
    // Create unsigned long and decode it
    let ulong_term = enif_make_ulong(&env, 100);
    let decoded = enif_get_ulong(&env, ulong_term);
    
    assert!(decoded.is_some());
    assert_eq!(decoded.unwrap(), 100);
}

#[test]
fn test_enif_get_binary() {
    let process = Arc::new(Process::new(17));
    let env = NifEnv::from_process(process);
    
    // Create binary and decode it
    let binary_data = b"hello world";
    let binary_term = enif_make_binary(&env, binary_data);
    let decoded = enif_get_binary(&env, binary_term);
    
    assert!(decoded.is_some());
    let data = decoded.unwrap();
    assert_eq!(data, binary_data);
}

#[test]
fn test_enif_get_string() {
    let process = Arc::new(Process::new(18));
    let env = NifEnv::from_process(process);
    
    // Create string and decode it
    let string_data = "hello";
    let string_term = enif_make_string(&env, string_data, NifCharEncoding::Latin1);
    let decoded = enif_get_string(&env, string_term);
    
    assert!(decoded.is_some());
    let (decoded_str, _encoding) = decoded.unwrap();
    assert_eq!(decoded_str, string_data);
}

#[test]
fn test_enif_get_tuple() {
    let process = Arc::new(Process::new(19));
    let env = NifEnv::from_process(process);
    
    // Create tuple and decode it
    let tuple_term = enif_make_tuple(&env, &[
        enif_make_int(&env, 1),
        enif_make_int(&env, 2),
        enif_make_int(&env, 3),
    ]);
    let decoded = enif_get_tuple(&env, tuple_term);
    
    assert!(decoded.is_some());
    let elements = decoded.unwrap();
    assert_eq!(elements.len(), 3);
}

#[test]
fn test_enif_get_list() {
    let process = Arc::new(Process::new(20));
    let env = NifEnv::from_process(process);
    
    // Create list and decode it
    let list_term = enif_make_list(&env, &[
        enif_make_int(&env, 1),
        enif_make_int(&env, 2),
        enif_make_int(&env, 3),
    ]);
    let decoded = enif_get_list(&env, list_term);
    
    assert!(decoded.is_some());
    let elements = decoded.unwrap();
    assert_eq!(elements.len(), 3);
}

#[test]
fn test_enif_make_badarg() {
    let process = Arc::new(Process::new(21));
    let env = NifEnv::from_process(process);
    
    // Create badarg exception
    let badarg_term = enif_make_badarg(&env);
    // Note: term 0 is valid (heap_index 0). Nil is 0x3F, not 0.
    // Verify it's an exception (this validates it's a valid term)
    let is_exception = enif_is_exception(&env, badarg_term);
    assert!(is_exception);
}

#[test]
fn test_enif_make_badarg_atom() {
    let process = Arc::new(Process::new(22));
    let env = NifEnv::from_process(process);
    
    // Create badarg atom
    let badarg_atom = enif_make_badarg_atom(&env);
    assert!(badarg_atom != 0);
}

#[test]
fn test_enif_is_exception() {
    let process = Arc::new(Process::new(23));
    let env = NifEnv::from_process(process);
    
    // Regular term should not be exception
    let normal_term = enif_make_int(&env, 42);
    assert!(!enif_is_exception(&env, normal_term));
    
    // Badarg should be exception
    let badarg_term = enif_make_badarg(&env);
    assert!(enif_is_exception(&env, badarg_term));
}

#[test]
fn test_enif_alloc_resource() {
    // Test resource allocation
    let resource_type = ErlNifResourceType::new("test_resource".to_string(), "test_module".to_string());
    let resource = enif_alloc_resource(&resource_type, 100);
    assert!(resource.is_ok());
    let resource = resource.unwrap();
    assert_eq!(resource.len(), 100);
}

#[test]
fn test_enif_release_resource() {
    // Test resource release
    let resource_type = ErlNifResourceType::new("test_resource".to_string(), "test_module".to_string());
    let resource = enif_alloc_resource(&resource_type, 50);
    assert!(resource.is_ok());
    let resource = resource.unwrap();
    enif_release_resource(resource);
    // Should not panic
}

#[test]
fn test_enif_make_resource() {
    let process = Arc::new(Process::new(26));
    let env = NifEnv::from_process(process);
    
    // Allocate and create resource term
    let resource_type = ErlNifResourceType::new("test_resource".to_string(), "test_module".to_string());
    let resource = enif_alloc_resource(&resource_type, 100);
    assert!(resource.is_ok());
    let resource = Arc::new(resource.unwrap());
    let resource_term = enif_make_resource(&env, &resource);
    
    assert!(resource_term != 0);
}

#[test]
fn test_roundtrip_atom() {
    let process = Arc::new(Process::new(25));
    let env = NifEnv::from_process(process);
    
    // Create atom and decode it back
    let atom_name = "test_atom";
    let atom_term = enif_make_atom(&env, atom_name);
    let decoded = enif_get_atom(&env, atom_term);
    
    assert!(decoded.is_some());
    let (decoded_name, _encoding) = decoded.unwrap();
    assert_eq!(decoded_name, atom_name);
}

#[test]
fn test_roundtrip_int() {
    let process = Arc::new(Process::new(26));
    let env = NifEnv::from_process(process);
    
    // Create integer and decode it back
    let int_value = 42;
    let int_term = enif_make_int(&env, int_value);
    let decoded = enif_get_int(&env, int_term);
    
    assert!(decoded.is_some());
    assert_eq!(decoded.unwrap(), int_value);
}

#[test]
fn test_roundtrip_binary() {
    let process = Arc::new(Process::new(27));
    let env = NifEnv::from_process(process);
    
    // Create binary and decode it back
    let binary_data = b"test binary data";
    let binary_term = enif_make_binary(&env, binary_data);
    let decoded = enif_get_binary(&env, binary_term);
    
    assert!(decoded.is_some());
    let data = decoded.unwrap();
    assert_eq!(data, binary_data);
}

#[test]
fn test_roundtrip_string() {
    let process = Arc::new(Process::new(28));
    let env = NifEnv::from_process(process);
    
    // Create string and decode it back
    let string_data = "hello world";
    let string_term = enif_make_string(&env, string_data, NifCharEncoding::Latin1);
    let decoded = enif_get_string(&env, string_term);
    
    assert!(decoded.is_some());
    let (decoded_str, _encoding) = decoded.unwrap();
    assert_eq!(decoded_str, string_data);
}

#[test]
fn test_roundtrip_tuple() {
    let process = Arc::new(Process::new(29));
    let env = NifEnv::from_process(process);
    
    // Create tuple and decode it back
    let elements = vec![
        enif_make_int(&env, 1),
        enif_make_int(&env, 2),
        enif_make_atom(&env, "test"),
    ];
    let tuple_term = enif_make_tuple(&env, &elements);
    let decoded = enif_get_tuple(&env, tuple_term);
    
    assert!(decoded.is_some());
    let decoded_elements = decoded.unwrap();
    assert_eq!(decoded_elements.len(), elements.len());
}

#[test]
fn test_roundtrip_list() {
    let process = Arc::new(Process::new(30));
    let env = NifEnv::from_process(process);
    
    // Create list and decode it back
    let elements = vec![
        enif_make_int(&env, 1),
        enif_make_int(&env, 2),
        enif_make_int(&env, 3),
    ];
    let list_term = enif_make_list(&env, &elements);
    let decoded = enif_get_list(&env, list_term);
    
    assert!(decoded.is_some());
    let elements = decoded.unwrap();
    assert_eq!(elements.len(), 3);
}

#[test]
fn test_nested_structures() {
    let process = Arc::new(Process::new(31));
    let env = NifEnv::from_process(process);
    
    // Create nested tuple containing list
    let inner_list = enif_make_list(&env, &[
        enif_make_int(&env, 1),
        enif_make_int(&env, 2),
    ]);
    let nested_tuple = enif_make_tuple(&env, &[
        enif_make_atom(&env, "key"),
        inner_list,
    ]);
    
    assert!(nested_tuple != 0);
    
    // Decode nested structure
    let decoded = enif_get_tuple(&env, nested_tuple);
    assert!(decoded.is_some());
    let elements = decoded.unwrap();
    assert_eq!(elements.len(), 2);
}

#[test]
fn test_get_process() {
    let process = Arc::new(Process::new(32));
    let env = NifEnv::from_process(Arc::clone(&process));
    
    // Get process from environment
    let retrieved_process = get_process(&env);
    assert_eq!(retrieved_process.id(), process.id());
}

#[test]
fn test_get_process_id() {
    let process = Arc::new(Process::new(33));
    let env = NifEnv::from_process(process);
    
    // Get process ID from environment
    let process_id = get_process_id(&env);
    assert_eq!(process_id, 33);
}

#[test]
fn test_nif_char_encoding_enum() {
    // Test NifCharEncoding enum
    let latin1 = NifCharEncoding::Latin1;
    let utf8 = NifCharEncoding::Utf8;
    
    assert_ne!(latin1, utf8);
    assert_eq!(latin1, NifCharEncoding::Latin1);
    assert_eq!(utf8, NifCharEncoding::Utf8);
}

#[test]
fn test_error_cases() {
    let process = Arc::new(Process::new(34));
    let env = NifEnv::from_process(process);
    
    // Test decoding terms that are not ints or atoms
    // Use nil (0x3F) - it's a valid term but not an int or atom,
    // so enif_get_int and enif_get_atom should return None
    let invalid_term = 0x3Fu64; // Nil term
    let decoded_int = enif_get_int(&env, invalid_term);
    assert!(decoded_int.is_none());
    
    let decoded_atom = enif_get_atom(&env, invalid_term);
    assert!(decoded_atom.is_none());
}

#[test]
fn test_multiple_terms_creation() {
    let process = Arc::new(Process::new(35));
    let env = NifEnv::from_process(process);
    
    // Create multiple different types of terms
    let atom = enif_make_atom(&env, "atom");
    let int = enif_make_int(&env, 42);
    let binary = enif_make_binary(&env, b"data");
    let string = enif_make_string(&env, "string", NifCharEncoding::Latin1);
    let tuple = enif_make_tuple(&env, &[int]);
    let list = enif_make_list(&env, &[int]);
    
    // All should be valid terms
    assert!(atom != 0);
    assert!(int != 0);
    assert!(binary != 0);
    assert!(string != 0);
    assert!(tuple != 0);
    assert!(list != 0);
    
    // All should be different
    assert_ne!(atom, int);
    assert_ne!(int, binary);
    assert_ne!(binary, string);
}

