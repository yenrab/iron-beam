//! Integration tests for infrastructure_code_loading crate
//!
//! These tests verify that encoding and decoding operations work correctly
//! and test end-to-end workflows for all supported data types.

use infrastructure_code_loading::*;

use infrastructure_code_loading::constants::*;
use infrastructure_code_loading::code_loader::{CodeLoader, LoadError};
use infrastructure_code_loading::encode_pid::ErlangPid;
use infrastructure_code_loading::encode_port::ErlangPort;
use infrastructure_code_loading::encode_ref::ErlangRef;
use infrastructure_code_loading::encode_fun::ErlangFunType;
use infrastructure_code_loading::encode_trace::ErlangTrace;

#[test]
fn test_code_loader_basic() {
    use std::fs;
    use std::path::PathBuf;
    
    // Create a temporary test file
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("test_code_loader.bin");
    let test_data = b"test code data";
    
    fs::write(&test_file, test_data).unwrap();
    
    // Load the file
    let loaded = CodeLoader::load_from_file(&test_file).unwrap();
    assert_eq!(loaded, test_data);
    
    // Clean up
    let _ = fs::remove_file(&test_file);
}

#[test]
fn test_code_loader_file_not_found() {
    let result = CodeLoader::load_from_file("nonexistent_file.bin");
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), LoadError::FileError);
}

#[test]
fn test_encode_decode_integers_roundtrip() {
    let test_values = vec![
        0i64,
        1i64,
        -1i64,
        42i64,
        -42i64,
        255i64,
        256i64,
        1000i64,
        -1000i64,
        2147483647i64,  // ERL_MAX
        -2147483648i64, // ERL_MIN
    ];
    
    for value in test_values {
        // Encode
        let mut buf = vec![0u8; 100];
        let mut buf_slice = buf.as_mut_slice();
        let mut buf_opt = Some(&mut buf_slice);
        let mut index = 0;
        encode_longlong(&mut buf_opt, &mut index, value).unwrap();
        
        // Decode
        let mut decode_index = 0;
        let decoded = decode_longlong(&buf, &mut decode_index).unwrap();
        
        assert_eq!(decoded, value, "Roundtrip failed for value: {}", value);
        assert_eq!(index, decode_index, "Index mismatch for value: {}", value);
    }
}

#[test]
fn test_encode_decode_unsigned_integers_roundtrip() {
    let test_values = vec![
        0u64,
        1u64,
        42u64,
        255u64,
        256u64,
        1000u64,
        4294967295u64, // u32::MAX
        18446744073709551615u64, // u64::MAX
    ];
    
    for value in test_values {
        // Encode
        let mut buf = vec![0u8; 100];
        let mut index = 0;
        encode_ulonglong({
        let mut buf_slice = buf.as_mut_slice();
        let mut buf_opt = Some(&mut buf_slice);
        &mut buf_opt
    }, &mut index, value).unwrap();
        
        // Decode
        let mut decode_index = 0;
        let decoded = decode_ulonglong(&buf, &mut decode_index).unwrap();
        
        assert_eq!(decoded, value, "Roundtrip failed for value: {}", value);
        assert_eq!(index, decode_index, "Index mismatch for value: {}", value);
    }
}

#[test]
fn test_encode_decode_32bit_integers() {
    let test_values = vec![
        0i32,
        1i32,
        -1i32,
        42i32,
        -42i32,
        2147483647i32,  // i32::MAX
        -2147483648i32, // i32::MIN
    ];
    
    for value in test_values {
        // Encode
        let mut buf = vec![0u8; 10];
        let mut index = 0;
        encode_long({
        let mut buf_slice = buf.as_mut_slice();
        let mut buf_opt = Some(&mut buf_slice);
        &mut buf_opt
    }, &mut index, value).unwrap();
        
        // Decode
        let mut decode_index = 0;
        let decoded = decode_long(&buf, &mut decode_index).unwrap();
        
        assert_eq!(decoded, value, "Roundtrip failed for value: {}", value);
    }
}

#[test]
fn test_encode_decode_unsigned_32bit_integers() {
    let test_values = vec![
        0u32,
        1u32,
        42u32,
        4294967295u32, // u32::MAX
    ];
    
    for value in test_values {
        // Encode
        let mut buf = vec![0u8; 10];
        let mut index = 0;
        encode_ulong({
        let mut buf_slice = buf.as_mut_slice();
        let mut buf_opt = Some(&mut buf_slice);
        &mut buf_opt
    }, &mut index, value).unwrap();
        
        // Decode
        let mut decode_index = 0;
        let decoded = decode_ulong(&buf, &mut decode_index).unwrap();
        
        assert_eq!(decoded, value, "Roundtrip failed for value: {}", value);
    }
}

#[test]
fn test_encode_decode_double_roundtrip() {
    let test_values = vec![
        0.0f64,
        1.0f64,
        -1.0f64,
        3.14159f64,
        -3.14159f64,
        1.7976931348623157e308f64, // f64::MAX
        -1.7976931348623157e308f64, // f64::MIN
    ];
    
    for value in test_values {
        // Encode
        let mut buf = vec![0u8; 20];
        let mut index = 0;
        encode_double({
        let mut buf_slice = buf.as_mut_slice();
        let mut buf_opt = Some(&mut buf_slice);
        &mut buf_opt
    }, &mut index, value).unwrap();
        
        // Decode
        let mut decode_index = 0;
        let decoded = decode_double(&buf, &mut decode_index).unwrap();
        
        // Use approximate equality for floating point
        assert!((decoded - value).abs() < 1e-10, "Roundtrip failed for value: {}", value);
        assert_eq!(index, decode_index, "Index mismatch for value: {}", value);
    }
}

#[test]
fn test_encode_decode_char_roundtrip() {
    for value in 0u8..=255u8 {
        // Encode
        let mut buf = vec![0u8; 10];
        let mut index = 0;
        encode_char({
        let mut buf_slice = buf.as_mut_slice();
        let mut buf_opt = Some(&mut buf_slice);
        &mut buf_opt
    }, &mut index, value).unwrap();
        
        // Decode
        let mut decode_index = 0;
        let decoded = decode_char(&buf, &mut decode_index).unwrap();
        
        assert_eq!(decoded, value, "Roundtrip failed for value: {}", value);
        assert_eq!(index, decode_index, "Index mismatch for value: {}", value);
    }
}

#[test]
fn test_encode_decode_tuple_header() {
    let test_arities = vec![0, 1, 2, 10, 255, 256, 1000];
    
    for arity in test_arities {
        // Encode
        let mut buf = vec![0u8; 100];
        let mut index = 0;
        encode_tuple_header({
        let mut buf_slice = buf.as_mut_slice();
        let mut buf_opt = Some(&mut buf_slice);
        &mut buf_opt
    }, &mut index, arity).unwrap();
        
        // Decode
        let mut decode_index = 0;
        let decoded = decode_tuple_header(&buf, &mut decode_index).unwrap();
        
        assert_eq!(decoded, arity, "Roundtrip failed for arity: {}", arity);
        assert_eq!(index, decode_index, "Index mismatch for arity: {}", arity);
    }
}

#[test]
fn test_encode_decode_map_header() {
    let test_arities = vec![0, 1, 2, 10, 255, 256, 1000];
    
    for arity in test_arities {
        // Encode
        let mut buf = vec![0u8; 100];
        let mut index = 0;
        encode_map_header({
        let mut buf_slice = buf.as_mut_slice();
        let mut buf_opt = Some(&mut buf_slice);
        &mut buf_opt
    }, &mut index, arity).unwrap();
        
        // Decode
        let mut decode_index = 0;
        let decoded = decode_map_header(&buf, &mut decode_index).unwrap();
        
        assert_eq!(decoded, arity, "Roundtrip failed for arity: {}", arity);
        assert_eq!(index, decode_index, "Index mismatch for arity: {}", arity);
    }
}

#[test]
fn test_encode_decode_list_header() {
    let test_lengths = vec![0, 1, 2, 10, 255, 256, 1000];
    
    for length in test_lengths {
        // Encode
        let mut buf = vec![0u8; 100];
        let mut index = 0;
        encode_list_header({
        let mut buf_slice = buf.as_mut_slice();
        let mut buf_opt = Some(&mut buf_slice);
        &mut buf_opt
    }, &mut index, length).unwrap();
        
        // Decode
        let mut decode_index = 0;
        let decoded = decode_list_header(&buf, &mut decode_index).unwrap();
        
        assert_eq!(decoded, length, "Roundtrip failed for length: {}", length);
        assert_eq!(index, decode_index, "Index mismatch for length: {}", length);
    }
}

#[test]
fn test_encode_decode_pid_roundtrip() {
    let pid = ErlangPid {
        node: 1,
        id: 2,
        serial: 3,
        creation: 4,
    };
    
    // Encode
    let mut buf = vec![0u8; 100];
    let mut index = 0;
    encode_pid({
        let mut buf_slice = buf.as_mut_slice();
        let mut buf_opt = Some(&mut buf_slice);
        &mut buf_opt
    }, &mut index, &pid).unwrap();
    
    // Decode
    let mut decode_index = 0;
    let decoded = decode_pid(&buf, &mut decode_index).unwrap();
    
    assert_eq!(decoded.node, pid.node);
    assert_eq!(decoded.id, pid.id);
    assert_eq!(decoded.serial, pid.serial);
    assert_eq!(decoded.creation, pid.creation);
    assert_eq!(index, decode_index);
}

#[test]
fn test_encode_decode_port_roundtrip() {
    let port = ErlangPort {
        node: 1,
        id: 2,
        creation: 3,
    };
    
    // Encode
    let mut buf = vec![0u8; 100];
    let mut index = 0;
    encode_port({
        let mut buf_slice = buf.as_mut_slice();
        let mut buf_opt = Some(&mut buf_slice);
        &mut buf_opt
    }, &mut index, &port).unwrap();
    
    // Decode
    let mut decode_index = 0;
    let decoded = decode_port(&buf, &mut decode_index).unwrap();
    
    assert_eq!(decoded.node, port.node);
    assert_eq!(decoded.id, port.id);
    assert_eq!(decoded.creation, port.creation);
    assert_eq!(index, decode_index);
}

#[test]
fn test_encode_decode_ref_roundtrip() {
    let r#ref = ErlangRef {
        node: 1,
        id: vec![2, 3, 4],
        creation: 5,
    };
    
    // Encode
    let mut buf = vec![0u8; 100];
    let mut index = 0;
    encode_ref({
        let mut buf_slice = buf.as_mut_slice();
        let mut buf_opt = Some(&mut buf_slice);
        &mut buf_opt
    }, &mut index, &r#ref).unwrap();
    
    // Decode
    let mut decode_index = 0;
    let decoded = decode_ref(&buf, &mut decode_index).unwrap();
    
    assert_eq!(decoded.node, r#ref.node);
    assert_eq!(decoded.id, r#ref.id);
    assert_eq!(decoded.creation, r#ref.creation);
    assert_eq!(index, decode_index);
}

#[test]
fn test_encode_decode_trace_roundtrip() {
    let trace = ErlangTrace {
        label: 1,
        serial: 2,
        prev: 3,
    };
    
    // Encode
    let mut buf = vec![0u8; 100];
    let mut index = 0;
    encode_trace({
        let mut buf_slice = buf.as_mut_slice();
        let mut buf_opt = Some(&mut buf_slice);
        &mut buf_opt
    }, &mut index, &trace).unwrap();
    
    // Decode
    let mut decode_index = 0;
    let decoded = decode_trace(&buf, &mut decode_index).unwrap();
    
    assert_eq!(decoded.label, trace.label);
    assert_eq!(decoded.serial, trace.serial);
    assert_eq!(decoded.prev, trace.prev);
    assert_eq!(index, decode_index);
}

#[test]
fn test_encode_decode_fun_roundtrip() {
    let fun = ErlangFunType::Export {
        module: 1,
        function: 2,
        arity: 3,
    };
    
    // Encode
    let mut buf = vec![0u8; 100];
    let mut index = 0;
    encode_fun({
        let mut buf_slice = buf.as_mut_slice();
        let mut buf_opt = Some(&mut buf_slice);
        &mut buf_opt
    }, &mut index, &fun).unwrap();
    
    // Decode
    let mut decode_index = 0;
    let decoded = decode_fun(&buf, &mut decode_index).unwrap();
    
    // Compare based on variant
    match (decoded, fun) {
        (ErlangFunType::Export { module: m1, function: f1, arity: a1 },
         ErlangFunType::Export { module: m2, function: f2, arity: a2 }) => {
            assert_eq!(m1, m2);
            assert_eq!(f1, f2);
            assert_eq!(a1, a2);
        }
        _ => panic!("Function type mismatch"),
    }
    assert_eq!(index, decode_index);
}

#[test]
fn test_buffer_too_small_errors() {
    // Test encoding with buffer that's too small
    let mut buf = vec![0u8; 1];
    let mut index = 0;
    
    let result = encode_longlong({
        let mut buf_slice = buf.as_mut_slice();
        let mut buf_opt = Some(&mut buf_slice);
        &mut buf_opt
    }, &mut index, 1000);
    assert!(result.is_err());
    
    let result = encode_double({
        let mut buf_slice = buf.as_mut_slice();
        let mut buf_opt = Some(&mut buf_slice);
        &mut buf_opt
    }, &mut index, 3.14);
    assert!(result.is_err());
    
    let result = encode_tuple_header({
        let mut buf_slice = buf.as_mut_slice();
        let mut buf_opt = Some(&mut buf_slice);
        &mut buf_opt
    }, &mut index, 10);
    assert!(result.is_err());
}

#[test]
fn test_buffer_too_short_decode_errors() {
    // Test decoding with buffer that's too short
    let buf = vec![ERL_SMALL_INTEGER_EXT]; // Missing value byte
    let mut index = 0;
    
    let result = decode_longlong(&buf, &mut index);
    assert!(result.is_err());
    
    let buf = vec![ERL_INTEGER_EXT]; // Missing 4 bytes
    let mut index = 0;
    
    let result = decode_longlong(&buf, &mut index);
    assert!(result.is_err());
}

#[test]
fn test_skip_term() {
    use infrastructure_code_loading::decode_skip::skip_term;
    
    // Create a buffer with a small integer
    let mut buf = vec![0u8; 10];
    let mut index = 0;
    encode_longlong({
        let mut buf_slice = buf.as_mut_slice();
        let mut buf_opt = Some(&mut buf_slice);
        &mut buf_opt
    }, &mut index, 42).unwrap();
    
    // Skip the term
    let mut skip_index = 0;
    skip_term(&buf, &mut skip_index).unwrap();
    
    assert_eq!(skip_index, index);
}

#[test]
fn test_constants_values() {
    // Verify constant values are correct
    assert_eq!(ERL_SMALL_INTEGER_EXT, 97);
    assert_eq!(ERL_INTEGER_EXT, 98);
    assert_eq!(ERL_ATOM_EXT, 100);
    assert_eq!(ERL_SMALL_TUPLE_EXT, 104);
    assert_eq!(ERL_LARGE_TUPLE_EXT, 105);
    assert_eq!(ERL_NIL_EXT, 106);
    assert_eq!(ERL_STRING_EXT, 107);
    assert_eq!(ERL_LIST_EXT, 108);
    assert_eq!(ERL_BINARY_EXT, 109);
}

#[test]
fn test_encode_with_none_buffer() {
    // Test encoding with None buffer (size calculation mode)
    let mut index = 0;
    let result = encode_longlong(None, &mut index, 42);
    assert!(result.is_ok());
    assert!(index > 0); // Index should advance
    
    let mut index = 0;
    let result = encode_double(None, &mut index, 3.14);
    assert!(result.is_ok());
    assert!(index > 0);
}

#[test]
fn test_multiple_encodings_in_sequence() {
    // Encode multiple values in sequence
    let mut buf = vec![0u8; 200];
    let mut index = 0;
    
    encode_longlong({
        let mut buf_slice = buf.as_mut_slice();
        let mut buf_opt = Some(&mut buf_slice);
        &mut buf_opt
    }, &mut index, 42).unwrap();
    encode_double({
        let mut buf_slice = buf.as_mut_slice();
        let mut buf_opt = Some(&mut buf_slice);
        &mut buf_opt
    }, &mut index, 3.14).unwrap();
    encode_char({
        let mut buf_slice = buf.as_mut_slice();
        let mut buf_opt = Some(&mut buf_slice);
        &mut buf_opt
    }, &mut index, 65).unwrap();
    encode_tuple_header({
        let mut buf_slice = buf.as_mut_slice();
        let mut buf_opt = Some(&mut buf_slice);
        &mut buf_opt
    }, &mut index, 3).unwrap();
    
    // Decode in sequence
    let mut decode_index = 0;
    let int_val = decode_longlong(&buf, &mut decode_index).unwrap();
    let double_val = decode_double(&buf, &mut decode_index).unwrap();
    let char_val = decode_char(&buf, &mut decode_index).unwrap();
    let tuple_arity = decode_tuple_header(&buf, &mut decode_index).unwrap();
    
    assert_eq!(int_val, 42);
    assert!((double_val - 3.14).abs() < 1e-10);
    assert_eq!(char_val, 65);
    assert_eq!(tuple_arity, 3);
    assert_eq!(index, decode_index);
}
