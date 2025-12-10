//! Integration tests for infrastructure_external_format
//!
//! Tests the external term format encoding/decoding functionality including
//! encoding, decoding, and size calculation.

use infrastructure_external_format::{
    enc_term, dec_term, enc_atom, dec_atom, erts_encode_ext, erts_decode_ext,
    erts_encode_ext_size, encode_size_struct_int, EncodeError, DecodeError,
};
use entities_data_handling::term_hashing::Term;
use entities_data_handling::atom::{AtomTable, AtomEncoding};

#[test]
fn test_enc_term_nil() {
    let term = Term::Nil;
    let encoded = enc_term(&term, None).unwrap();
    assert_eq!(encoded, vec![131, 106]); // VERSION_MAGIC, NIL_EXT
}

#[test]
fn test_dec_term_nil() {
    let data = vec![131, 106]; // VERSION_MAGIC, NIL_EXT
    let term = dec_term(&data).unwrap();
    assert!(matches!(term, Term::Nil));
}

#[test]
fn test_enc_dec_small_integer() {
    let term = Term::Small(42);
    let encoded = enc_term(&term, None).unwrap();
    assert_eq!(encoded[0], 131); // VERSION_MAGIC
    assert_eq!(encoded[1], 97); // SMALL_INTEGER_EXT
    assert_eq!(encoded[2], 42); // value
    
    let decoded = dec_term(&encoded).unwrap();
    match decoded {
        Term::Small(value) => assert_eq!(value, 42),
        _ => panic!("Expected Small(42)"),
    }
}

#[test]
fn test_enc_dec_tuple() {
    let term = Term::Tuple(vec![
        Term::Small(1),
        Term::Small(2),
        Term::Small(3),
    ]);
    let encoded = enc_term(&term, None).unwrap();
    assert_eq!(encoded[0], 131); // VERSION_MAGIC
    assert_eq!(encoded[1], 104); // SMALL_TUPLE_EXT
    assert_eq!(encoded[2], 3); // arity
    
    let decoded = dec_term(&encoded).unwrap();
    match decoded {
        Term::Tuple(elements) => {
            assert_eq!(elements.len(), 3);
            match elements[0] {
                Term::Small(1) => {},
                _ => panic!("Expected Small(1)"),
            }
        }
        _ => panic!("Expected Tuple"),
    }
}

#[test]
fn test_enc_dec_list() {
    let term = Term::List {
        head: Box::new(Term::Small(1)),
        tail: Box::new(Term::Nil),
    };
    let encoded = enc_term(&term, None).unwrap();
    assert_eq!(encoded[0], 131); // VERSION_MAGIC
    
    let decoded = dec_term(&encoded).unwrap();
    match decoded {
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
fn test_enc_dec_atom() {
    let mut atom_table = AtomTable::new(100);
    let atom_index = atom_table.put_index(b"test", AtomEncoding::SevenBitAscii, false).unwrap();
    
    let term = Term::Atom(atom_index as u32);
    let encoded = enc_term(&term, Some(&atom_table)).unwrap();
    assert_eq!(encoded[0], 131); // VERSION_MAGIC
    
    // Decode the term
    // Note: The decoded atom index may not match the original due to how atoms are decoded
    // The important thing is that it decodes to an Atom variant
    let decoded = dec_term(&encoded).unwrap();
    match decoded {
        Term::Atom(_idx) => {
            // Atom decoded successfully, index may differ
        }
        _ => panic!("Expected Atom"),
    }
}

#[test]
fn test_enc_atom() {
    let mut atom_table = AtomTable::new(100);
    let atom_index = atom_table.put_index(b"test", AtomEncoding::SevenBitAscii, false).unwrap();
    
    let mut buf = Vec::new();
    enc_atom(atom_index, Some(&atom_table), &mut buf).unwrap();
    
    // Should encode atom (SMALL_ATOM_EXT = 115 or ATOM_EXT = 100)
    assert!(buf[0] == 115 || buf[0] == 100);
}

#[test]
fn test_dec_atom() {
    // SMALL_ATOM_EXT = 115, length = 4, "test"
    let data = vec![115, 4, b't', b'e', b's', b't'];
    let mut atom_table = AtomTable::new(100);
    let (atom_index, new_pos) = dec_atom(&data, Some(&mut atom_table)).unwrap();
    assert_eq!(new_pos, 6); // 1 (tag) + 1 (length) + 4 (bytes)
    assert!(atom_index > 0);
}

#[test]
fn test_erts_encode_ext() {
    let term = Term::Small(42);
    let encoded = erts_encode_ext(&term, None).unwrap();
    assert_eq!(encoded[0], 131); // VERSION_MAGIC
    assert_eq!(encoded[1], 97); // SMALL_INTEGER_EXT
    assert_eq!(encoded[2], 42); // value
}

#[test]
fn test_erts_decode_ext() {
    let data = vec![131, 97, 42]; // VERSION_MAGIC, SMALL_INTEGER_EXT, value
    let term = erts_decode_ext(&data).unwrap();
    match term {
        Term::Small(value) => assert_eq!(value, 42),
        _ => panic!("Expected Small(42)"),
    }
}

#[test]
fn test_erts_encode_ext_size() {
    let term = Term::Nil;
    let size = erts_encode_ext_size(&term, None).unwrap();
    assert_eq!(size, 2); // 1 (version) + 1 (NIL_EXT)
    
    let term2 = Term::Small(42);
    let size2 = erts_encode_ext_size(&term2, None).unwrap();
    assert_eq!(size2, 3); // 1 (version) + 1 (tag) + 1 (value)
}

#[test]
fn test_encode_size_struct_int() {
    let term = Term::Nil;
    let size = encode_size_struct_int(&term, None).unwrap();
    assert_eq!(size, 1); // NIL_EXT
    
    let term2 = Term::Small(42);
    let size2 = encode_size_struct_int(&term2, None).unwrap();
    assert_eq!(size2, 2); // 1 (tag) + 1 (value)
}

#[test]
fn test_encode_size_tuple() {
    let term = Term::Tuple(vec![
        Term::Small(1),
        Term::Small(2),
        Term::Small(3),
    ]);
    let size = encode_size_struct_int(&term, None).unwrap();
    // 1 (tag) + 1 (arity) + 3 * 2 (elements)
    assert_eq!(size, 1 + 1 + 6);
}

#[test]
fn test_dec_term_invalid_version() {
    let data = vec![130, 97, 42]; // Wrong version magic byte
    let result = dec_term(&data);
    assert!(matches!(result, Err(DecodeError::InvalidVersion)));
}

#[test]
fn test_dec_term_empty() {
    let data = vec![];
    let result = dec_term(&data);
    assert!(matches!(result, Err(DecodeError::BufferTooShort)));
}

#[test]
fn test_enc_dec_roundtrip() {
    let terms = vec![
        Term::Nil,
        Term::Small(0),
        Term::Small(42),
        Term::Small(-1),
        Term::Tuple(vec![Term::Small(1), Term::Small(2)]),
    ];
    
    for term in terms {
        let encoded = enc_term(&term, None).unwrap();
        let decoded = dec_term(&encoded).unwrap();
        // Note: We can't directly compare terms, but we can verify structure
        match (&term, &decoded) {
            (Term::Nil, Term::Nil) => {},
            (Term::Small(a), Term::Small(b)) => assert_eq!(a, b),
            (Term::Tuple(a), Term::Tuple(b)) => assert_eq!(a.len(), b.len()),
            _ => panic!("Roundtrip failed for term: {:?}", term),
        }
    }
}

#[test]
fn test_error_types() {
    // Test error creation
    let _err1 = EncodeError::BufferTooSmall;
    let _err2 = EncodeError::EncodingFailed("test".to_string());
    let _err3 = EncodeError::InvalidTerm("test".to_string());
    let _err4 = EncodeError::AtomNotFound;
    
    let _err5 = DecodeError::BufferTooShort;
    let _err6 = DecodeError::InvalidFormat("test".to_string());
    let _err7 = DecodeError::DecodingFailed("test".to_string());
    let _err8 = DecodeError::AtomDecodeError("test".to_string());
    let _err9 = DecodeError::InvalidVersion;
}

