//! Binary Erlang Term Decoder
//!
//! Decodes binary Erlang terms (External Term Format - ETF).
//! Used for parsing .boot files and other binary Erlang data.
//!
//! Based on the Erlang External Term Format specification.
//! Format: 131 (version byte) followed by encoded term

use std::io::{Cursor, Read};

/// Erlang term types
#[derive(Debug, Clone, PartialEq)]
pub enum ErlangTerm {
    /// Atom
    Atom(String),
    /// Integer (small or big)
    Integer(i64),
    /// Float
    Float(f64),
    /// List
    List(Vec<ErlangTerm>),
    /// Tuple
    Tuple(Vec<ErlangTerm>),
    /// Binary
    Binary(Vec<u8>),
    /// Nil (empty list)
    Nil,
}

/// Decoder error
#[derive(Debug, Clone)]
pub enum DecoderError {
    /// Unexpected end of data
    UnexpectedEof,
    /// Invalid format
    InvalidFormat(String),
    /// Unsupported term type
    UnsupportedType(u8),
    /// IO error
    IoError(String),
}

impl std::fmt::Display for DecoderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecoderError::UnexpectedEof => write!(f, "Unexpected end of data"),
            DecoderError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            DecoderError::UnsupportedType(tag) => write!(f, "Unsupported term type: 0x{:02x}", tag),
            DecoderError::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for DecoderError {}

impl From<std::io::Error> for DecoderError {
    fn from(err: std::io::Error) -> Self {
        DecoderError::IoError(err.to_string())
    }
}

/// Binary Erlang term decoder
pub struct TermDecoder<'a> {
    cursor: Cursor<&'a [u8]>,
}

impl<'a> TermDecoder<'a> {
    /// Create a new decoder
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            cursor: Cursor::new(data),
        }
    }

    /// Decode a single term
    pub fn decode(&mut self) -> Result<ErlangTerm, DecoderError> {
        // Read version byte (should be 131)
        let version = self.read_u8()?;
        if version != 131 {
            return Err(DecoderError::InvalidFormat(format!(
                "Expected version byte 131, got {}",
                version
            )));
        }

        self.decode_term()
    }

    /// Decode a term (without version byte)
    fn decode_term(&mut self) -> Result<ErlangTerm, DecoderError> {
        let tag = self.read_u8()?;

        match tag {
            // Small integer (0-255)
            97 => {
                let value = self.read_u8()? as i64;
                Ok(ErlangTerm::Integer(value))
            }
            // Integer (32-bit signed)
            98 => {
                let value = self.read_i32_be()? as i64;
                Ok(ErlangTerm::Integer(value))
            }
            // Float (deprecated, but still used)
            99 => {
                let mut buf = [0u8; 31];
                self.cursor.read_exact(&mut buf)?;
                // Parse as string and convert to float
                // This is simplified - real implementation would parse IEEE 754
                Ok(ErlangTerm::Float(0.0)) // Placeholder
            }
            // Atom (small, < 256 chars)
            100 => {
                let len = self.read_u16_be()? as usize;
                let mut buf = vec![0u8; len];
                self.cursor.read_exact(&mut buf)?;
                let atom = String::from_utf8_lossy(&buf).to_string();
                Ok(ErlangTerm::Atom(atom))
            }
            // Small tuple (arity < 256)
            104 => {
                let arity = self.read_u8()? as usize;
                let mut elements = Vec::with_capacity(arity);
                for _ in 0..arity {
                    elements.push(self.decode_term()?);
                }
                Ok(ErlangTerm::Tuple(elements))
            }
            // Large tuple (arity >= 256)
            105 => {
                let arity = self.read_u32_be()? as usize;
                let mut elements = Vec::with_capacity(arity);
                for _ in 0..arity {
                    elements.push(self.decode_term()?);
                }
                Ok(ErlangTerm::Tuple(elements))
            }
            // Nil (empty list)
            106 => Ok(ErlangTerm::Nil),
            // String (list of small integers)
            107 => {
                let len = self.read_u16_be()? as usize;
                let mut bytes = vec![0u8; len];
                self.cursor.read_exact(&mut bytes)?;
                Ok(ErlangTerm::Binary(bytes))
            }
            // List
            108 => {
                let len = self.read_u32_be()? as usize;
                let mut elements = Vec::with_capacity(len);
                for _ in 0..len {
                    elements.push(self.decode_term()?);
                }
                // Read tail (usually nil)
                let _tail = self.decode_term()?;
                Ok(ErlangTerm::List(elements))
            }
            // Binary
            109 => {
                let len = self.read_u32_be()? as usize;
                let mut data = vec![0u8; len];
                self.cursor.read_exact(&mut data)?;
                Ok(ErlangTerm::Binary(data))
            }
            // Small big integer
            110 => {
                let n = self.read_u8()? as usize;
                let sign = self.read_u8()?;
                let mut bytes = vec![0u8; n];
                self.cursor.read_exact(&mut bytes)?;
                // Convert to i64 (simplified - real implementation would handle arbitrary precision)
                let mut value = 0i64;
                for (i, &byte) in bytes.iter().enumerate() {
                    value |= (byte as i64) << (i * 8);
                }
                if sign != 0 {
                    value = -value;
                }
                Ok(ErlangTerm::Integer(value))
            }
            // Large big integer
            111 => {
                let n = self.read_u32_be()? as usize;
                let sign = self.read_u8()?;
                let mut bytes = vec![0u8; n];
                self.cursor.read_exact(&mut bytes)?;
                // Convert to i64 (simplified)
                let mut value = 0i64;
                for (i, &byte) in bytes.iter().enumerate() {
                    if i < 8 {
                        value |= (byte as i64) << (i * 8);
                    }
                }
                if sign != 0 {
                    value = -value;
                }
                Ok(ErlangTerm::Integer(value))
            }
            _ => Err(DecoderError::UnsupportedType(tag)),
        }
    }

    /// Read a single byte
    fn read_u8(&mut self) -> Result<u8, DecoderError> {
        let mut buf = [0u8; 1];
        self.cursor
            .read_exact(&mut buf)
            .map_err(|_| DecoderError::UnexpectedEof)?;
        Ok(buf[0])
    }

    /// Read a u16 in big-endian
    fn read_u16_be(&mut self) -> Result<u16, DecoderError> {
        let mut buf = [0u8; 2];
        self.cursor
            .read_exact(&mut buf)
            .map_err(|_| DecoderError::UnexpectedEof)?;
        Ok(u16::from_be_bytes(buf))
    }

    /// Read a u32 in big-endian
    fn read_u32_be(&mut self) -> Result<u32, DecoderError> {
        let mut buf = [0u8; 4];
        self.cursor
            .read_exact(&mut buf)
            .map_err(|_| DecoderError::UnexpectedEof)?;
        Ok(u32::from_be_bytes(buf))
    }

    /// Read an i32 in big-endian
    fn read_i32_be(&mut self) -> Result<i32, DecoderError> {
        let mut buf = [0u8; 4];
        self.cursor
            .read_exact(&mut buf)
            .map_err(|_| DecoderError::UnexpectedEof)?;
        Ok(i32::from_be_bytes(buf))
    }
}

/// Decode a binary Erlang term
///
/// # Arguments
/// * `data` - Binary data containing encoded term
///
/// # Returns
/// Decoded Erlang term or error
pub fn decode_term(data: &[u8]) -> Result<ErlangTerm, DecoderError> {
    let mut decoder = TermDecoder::new(data);
    decoder.decode()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_atom() {
        // Encode atom "test": [131, 100, 0, 4, 't', 'e', 's', 't']
        let data = vec![131, 100, 0, 4, b't', b'e', b's', b't'];
        let term = decode_term(&data).unwrap();
        assert_eq!(term, ErlangTerm::Atom("test".to_string()));
    }

    #[test]
    fn test_decode_small_integer() {
        // Encode small integer 42: [131, 97, 42]
        let data = vec![131, 97, 42];
        let term = decode_term(&data).unwrap();
        assert_eq!(term, ErlangTerm::Integer(42));
    }

    #[test]
    fn test_decode_small_integer_zero() {
        let data = vec![131, 97, 0];
        let term = decode_term(&data).unwrap();
        assert_eq!(term, ErlangTerm::Integer(0));
    }

    #[test]
    fn test_decode_small_integer_max() {
        let data = vec![131, 97, 255];
        let term = decode_term(&data).unwrap();
        assert_eq!(term, ErlangTerm::Integer(255));
    }

    #[test]
    fn test_decode_integer() {
        // INTEGER_EXT (98) followed by 4-byte big-endian i32
        // Encode 12345: [131, 98, 0, 0, 48, 57]
        let data = vec![131, 98, 0, 0, 48, 57];
        let term = decode_term(&data).unwrap();
        assert_eq!(term, ErlangTerm::Integer(12345));
    }

    #[test]
    fn test_decode_integer_negative() {
        // Encode -12345: [131, 98, 255, 255, 207, 199]
        let data = vec![131, 98, 255, 255, 207, 199];
        let term = decode_term(&data).unwrap();
        assert_eq!(term, ErlangTerm::Integer(-12345));
    }

    #[test]
    fn test_decode_integer_max() {
        // Encode i32::MAX: [131, 98, 127, 255, 255, 255]
        let data = vec![131, 98, 127, 255, 255, 255];
        let term = decode_term(&data).unwrap();
        assert_eq!(term, ErlangTerm::Integer(2147483647));
    }

    #[test]
    fn test_decode_integer_min() {
        // Encode i32::MIN: [131, 98, 128, 0, 0, 0]
        let data = vec![131, 98, 128, 0, 0, 0];
        let term = decode_term(&data).unwrap();
        assert_eq!(term, ErlangTerm::Integer(-2147483648));
    }

    #[test]
    fn test_decode_float() {
        // FLOAT_EXT (99) - deprecated format, returns placeholder 0.0
        let mut data = vec![131, 99];
        data.extend_from_slice(&[0u8; 31]); // 31 bytes of float data
        let term = decode_term(&data).unwrap();
        match term {
            ErlangTerm::Float(f) => assert!((f - 0.0).abs() < f64::EPSILON),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_decode_atom_empty() {
        // Empty atom: [131, 100, 0, 0]
        let data = vec![131, 100, 0, 0];
        let term = decode_term(&data).unwrap();
        assert_eq!(term, ErlangTerm::Atom("".to_string()));
    }

    #[test]
    fn test_decode_atom_long() {
        // Atom "hello_world": [131, 100, 0, 11, 'h', 'e', 'l', 'l', 'o', '_', 'w', 'o', 'r', 'l', 'd']
        let data = vec![131, 100, 0, 11, b'h', b'e', b'l', b'l', b'o', b'_', b'w', b'o', b'r', b'l', b'd'];
        let term = decode_term(&data).unwrap();
        assert_eq!(term, ErlangTerm::Atom("hello_world".to_string()));
    }

    #[test]
    fn test_decode_nil() {
        // NIL_EXT (106): [131, 106]
        let data = vec![131, 106];
        let term = decode_term(&data).unwrap();
        assert_eq!(term, ErlangTerm::Nil);
    }

    #[test]
    fn test_decode_string() {
        // STRING_EXT (107) - list of small integers
        // Encode "hello": [131, 107, 0, 5, 'h', 'e', 'l', 'l', 'o']
        let data = vec![131, 107, 0, 5, b'h', b'e', b'l', b'l', b'o'];
        let term = decode_term(&data).unwrap();
        match term {
            ErlangTerm::Binary(bytes) => {
                assert_eq!(bytes, vec![b'h', b'e', b'l', b'l', b'o']);
            }
            _ => panic!("Expected Binary"),
        }
    }

    #[test]
    fn test_decode_string_empty() {
        // Empty string: [131, 107, 0, 0]
        let data = vec![131, 107, 0, 0];
        let term = decode_term(&data).unwrap();
        match term {
            ErlangTerm::Binary(bytes) => assert_eq!(bytes, Vec::<u8>::new()),
            _ => panic!("Expected Binary"),
        }
    }

    #[test]
    fn test_decode_binary() {
        // BINARY_EXT (109) - binary data
        // Encode 3 bytes: [131, 109, 0, 0, 0, 3, 1, 2, 3]
        let data = vec![131, 109, 0, 0, 0, 3, 1, 2, 3];
        let term = decode_term(&data).unwrap();
        match term {
            ErlangTerm::Binary(bytes) => assert_eq!(bytes, vec![1, 2, 3]),
            _ => panic!("Expected Binary"),
        }
    }

    #[test]
    fn test_decode_binary_empty() {
        // Empty binary: [131, 109, 0, 0, 0, 0]
        let data = vec![131, 109, 0, 0, 0, 0];
        let term = decode_term(&data).unwrap();
        match term {
            ErlangTerm::Binary(bytes) => assert_eq!(bytes, Vec::<u8>::new()),
            _ => panic!("Expected Binary"),
        }
    }

    #[test]
    fn test_decode_small_tuple() {
        // SMALL_TUPLE_EXT (104) with arity 2
        // Encode {atom, 42}: [131, 104, 2, 100, 0, 4, 'a', 't', 'o', 'm', 97, 42]
        let data = vec![131, 104, 2, 100, 0, 4, b'a', b't', b'o', b'm', 97, 42];
        let term = decode_term(&data).unwrap();
        match term {
            ErlangTerm::Tuple(elements) => {
                assert_eq!(elements.len(), 2);
                assert_eq!(elements[0], ErlangTerm::Atom("atom".to_string()));
                assert_eq!(elements[1], ErlangTerm::Integer(42));
            }
            _ => panic!("Expected Tuple"),
        }
    }

    #[test]
    fn test_decode_small_tuple_empty() {
        // Empty tuple: [131, 104, 0]
        let data = vec![131, 104, 0];
        let term = decode_term(&data).unwrap();
        match term {
            ErlangTerm::Tuple(elements) => assert_eq!(elements.len(), 0),
            _ => panic!("Expected Tuple"),
        }
    }

    #[test]
    fn test_decode_large_tuple() {
        // LARGE_TUPLE_EXT (105) with arity 256
        // This is a simplified test - we'll use a smaller arity for testing
        // Encode tuple with arity 1: [131, 105, 0, 0, 0, 1, 97, 42]
        let data = vec![131, 105, 0, 0, 0, 1, 97, 42];
        let term = decode_term(&data).unwrap();
        match term {
            ErlangTerm::Tuple(elements) => {
                assert_eq!(elements.len(), 1);
                assert_eq!(elements[0], ErlangTerm::Integer(42));
            }
            _ => panic!("Expected Tuple"),
        }
    }

    #[test]
    fn test_decode_list() {
        // LIST_EXT (108) with 2 elements
        // Encode [42, 43]: [131, 108, 0, 0, 0, 2, 97, 42, 97, 43, 106]
        let data = vec![131, 108, 0, 0, 0, 2, 97, 42, 97, 43, 106];
        let term = decode_term(&data).unwrap();
        match term {
            ErlangTerm::List(elements) => {
                assert_eq!(elements.len(), 2);
                assert_eq!(elements[0], ErlangTerm::Integer(42));
                assert_eq!(elements[1], ErlangTerm::Integer(43));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_decode_list_empty() {
        // Empty list: [131, 108, 0, 0, 0, 0, 106]
        let data = vec![131, 108, 0, 0, 0, 0, 106];
        let term = decode_term(&data).unwrap();
        match term {
            ErlangTerm::List(elements) => assert_eq!(elements.len(), 0),
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_decode_small_big_integer() {
        // SMALL_BIG_EXT (110) - 1 byte, positive
        // Encode 42: [131, 110, 1, 0, 42]
        let data = vec![131, 110, 1, 0, 42];
        let term = decode_term(&data).unwrap();
        assert_eq!(term, ErlangTerm::Integer(42));
    }

    #[test]
    fn test_decode_small_big_integer_negative() {
        // SMALL_BIG_EXT (110) - 1 byte, negative
        // Encode -42: [131, 110, 1, 1, 42]
        let data = vec![131, 110, 1, 1, 42];
        let term = decode_term(&data).unwrap();
        assert_eq!(term, ErlangTerm::Integer(-42));
    }

    #[test]
    fn test_decode_large_big_integer() {
        // LARGE_BIG_EXT (111) - 4-byte length, positive
        // Encode 42: [131, 111, 0, 0, 0, 1, 0, 42]
        let data = vec![131, 111, 0, 0, 0, 1, 0, 42];
        let term = decode_term(&data).unwrap();
        assert_eq!(term, ErlangTerm::Integer(42));
    }

    #[test]
    fn test_decode_large_big_integer_negative() {
        // LARGE_BIG_EXT (111) - 4-byte length, negative
        // Encode -42: [131, 111, 0, 0, 0, 1, 1, 42]
        let data = vec![131, 111, 0, 0, 0, 1, 1, 42];
        let term = decode_term(&data).unwrap();
        assert_eq!(term, ErlangTerm::Integer(-42));
    }

    #[test]
    fn test_decode_nested_tuple() {
        // Nested tuple: {{1, 2}, 3}
        // [131, 104, 2, 104, 2, 97, 1, 97, 2, 97, 3]
        let data = vec![131, 104, 2, 104, 2, 97, 1, 97, 2, 97, 3];
        let term = decode_term(&data).unwrap();
        match term {
            ErlangTerm::Tuple(elements) => {
                assert_eq!(elements.len(), 2);
                match &elements[0] {
                    ErlangTerm::Tuple(inner) => {
                        assert_eq!(inner.len(), 2);
                        assert_eq!(inner[0], ErlangTerm::Integer(1));
                        assert_eq!(inner[1], ErlangTerm::Integer(2));
                    }
                    _ => panic!("Expected nested tuple"),
                }
                assert_eq!(elements[1], ErlangTerm::Integer(3));
            }
            _ => panic!("Expected Tuple"),
        }
    }

    #[test]
    fn test_decode_list_with_atoms() {
        // List of atoms: [atom1, atom2]
        // [131, 108, 0, 0, 0, 2, 100, 0, 5, 'a', 't', 'o', 'm', '1', 100, 0, 5, 'a', 't', 'o', 'm', '2', 106]
        let data = vec![
            131, 108, 0, 0, 0, 2,
            100, 0, 5, b'a', b't', b'o', b'm', b'1',
            100, 0, 5, b'a', b't', b'o', b'm', b'2',
            106
        ];
        let term = decode_term(&data).unwrap();
        match term {
            ErlangTerm::List(elements) => {
                assert_eq!(elements.len(), 2);
                assert_eq!(elements[0], ErlangTerm::Atom("atom1".to_string()));
                assert_eq!(elements[1], ErlangTerm::Atom("atom2".to_string()));
            }
            _ => panic!("Expected List"),
        }
    }

    #[test]
    fn test_decode_error_invalid_version() {
        // Wrong version byte (130 instead of 131)
        let data = vec![130, 97, 42];
        let result = decode_term(&data);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecoderError::InvalidFormat(msg) => {
                assert!(msg.contains("Expected version byte 131"));
            }
            _ => panic!("Expected InvalidFormat error"),
        }
    }

    #[test]
    fn test_decode_error_empty() {
        // Empty data
        let data = vec![];
        let result = decode_term(&data);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecoderError::UnexpectedEof => {}
            _ => panic!("Expected UnexpectedEof error"),
        }
    }

    #[test]
    fn test_decode_error_incomplete_version() {
        // Only version byte, no term
        let data = vec![131];
        let result = decode_term(&data);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecoderError::UnexpectedEof => {}
            _ => panic!("Expected UnexpectedEof error"),
        }
    }

    #[test]
    fn test_decode_error_incomplete_atom() {
        // Atom tag but incomplete length
        let data = vec![131, 100, 0];
        let result = decode_term(&data);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecoderError::UnexpectedEof => {}
            _ => panic!("Expected UnexpectedEof error"),
        }
    }

    #[test]
    fn test_decode_error_incomplete_integer() {
        // Integer tag but incomplete data
        let data = vec![131, 98, 0, 0];
        let result = decode_term(&data);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecoderError::UnexpectedEof => {}
            _ => panic!("Expected UnexpectedEof error"),
        }
    }

    #[test]
    fn test_decode_error_unsupported_type() {
        // Unsupported tag (255)
        let data = vec![131, 255];
        let result = decode_term(&data);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecoderError::UnsupportedType(tag) => assert_eq!(tag, 255),
            _ => panic!("Expected UnsupportedType error"),
        }
    }

    #[test]
    fn test_decode_error_incomplete_tuple() {
        // Tuple tag with arity but incomplete elements
        let data = vec![131, 104, 2, 97, 42]; // Only one element, needs two
        let result = decode_term(&data);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecoderError::UnexpectedEof => {}
            _ => panic!("Expected UnexpectedEof error"),
        }
    }

    #[test]
    fn test_decode_error_incomplete_list() {
        // List tag with length but incomplete elements
        let data = vec![131, 108, 0, 0, 0, 2, 97, 42]; // Only one element, needs two
        let result = decode_term(&data);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecoderError::UnexpectedEof => {}
            _ => panic!("Expected UnexpectedEof error"),
        }
    }

    #[test]
    fn test_decode_error_incomplete_binary() {
        // Binary tag with length but incomplete data
        let data = vec![131, 109, 0, 0, 0, 3, 1, 2]; // Length 3 but only 2 bytes
        let result = decode_term(&data);
        assert!(result.is_err());
        // The error could be UnexpectedEof or IoError depending on how read_exact handles it
        match result.unwrap_err() {
            DecoderError::UnexpectedEof | DecoderError::IoError(_) => {}
            e => panic!("Expected UnexpectedEof or IoError, got {:?}", e),
        }
    }

    #[test]
    fn test_term_decoder_new() {
        let data = vec![131, 97, 42];
        let decoder = TermDecoder::new(&data);
        assert_eq!(decoder.cursor.position(), 0);
    }

    #[test]
    fn test_term_decoder_decode() {
        let data = vec![131, 97, 42];
        let mut decoder = TermDecoder::new(&data);
        let term = decoder.decode().unwrap();
        assert_eq!(term, ErlangTerm::Integer(42));
    }

    #[test]
    fn test_erlang_term_debug() {
        let terms = vec![
            ErlangTerm::Atom("test".to_string()),
            ErlangTerm::Integer(42),
            ErlangTerm::Float(3.14),
            ErlangTerm::Nil,
            ErlangTerm::List(vec![]),
            ErlangTerm::Tuple(vec![]),
            ErlangTerm::Binary(vec![1, 2, 3]),
        ];
        
        for term in terms {
            let debug_str = format!("{:?}", term);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_erlang_term_clone() {
        let term1 = ErlangTerm::Atom("test".to_string());
        let term2 = term1.clone();
        assert_eq!(term1, term2);
    }

    #[test]
    fn test_erlang_term_partial_eq() {
        assert_eq!(
            ErlangTerm::Integer(42),
            ErlangTerm::Integer(42)
        );
        assert_ne!(
            ErlangTerm::Integer(42),
            ErlangTerm::Integer(43)
        );
        assert_eq!(
            ErlangTerm::Atom("test".to_string()),
            ErlangTerm::Atom("test".to_string())
        );
        assert_ne!(
            ErlangTerm::Atom("test".to_string()),
            ErlangTerm::Atom("other".to_string())
        );
        assert_eq!(ErlangTerm::Nil, ErlangTerm::Nil);
        assert_ne!(ErlangTerm::Nil, ErlangTerm::Integer(0));
    }

    #[test]
    fn test_decoder_error_display() {
        let errors = vec![
            DecoderError::UnexpectedEof,
            DecoderError::InvalidFormat("test".to_string()),
            DecoderError::UnsupportedType(255),
            DecoderError::IoError("test".to_string()),
        ];
        
        for error in errors {
            let display_str = format!("{}", error);
            assert!(!display_str.is_empty());
        }
    }

    #[test]
    fn test_decoder_error_clone() {
        let error1 = DecoderError::UnexpectedEof;
        let error2 = error1.clone();
        // Can't compare DecoderError with PartialEq, but clone should work
        let display1 = format!("{}", error1);
        let display2 = format!("{}", error2);
        assert_eq!(display1, display2);
    }

    #[test]
    fn test_decoder_error_debug() {
        let error = DecoderError::InvalidFormat("test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_decoder_error_error_trait() {
        let error = DecoderError::UnexpectedEof;
        let error_ref: &dyn std::error::Error = &error;
        let display_str = format!("{}", error_ref);
        assert!(!display_str.is_empty());
    }

    #[test]
    fn test_decoder_error_from_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::Other, "test");
        let decoder_error: DecoderError = io_error.into();
        match decoder_error {
            DecoderError::IoError(msg) => assert!(msg.contains("test")),
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_read_u8() {
        let data = vec![42];
        let mut decoder = TermDecoder::new(&data);
        let value = decoder.read_u8().unwrap();
        assert_eq!(value, 42);
    }

    #[test]
    fn test_read_u8_eof() {
        let data = vec![];
        let mut decoder = TermDecoder::new(&data);
        let result = decoder.read_u8();
        assert!(result.is_err());
        match result.unwrap_err() {
            DecoderError::UnexpectedEof => {}
            _ => panic!("Expected UnexpectedEof"),
        }
    }

    #[test]
    fn test_read_u16_be() {
        let data = vec![0x12, 0x34];
        let mut decoder = TermDecoder::new(&data);
        let value = decoder.read_u16_be().unwrap();
        assert_eq!(value, 0x1234);
    }

    #[test]
    fn test_read_u16_be_eof() {
        let data = vec![0x12];
        let mut decoder = TermDecoder::new(&data);
        let result = decoder.read_u16_be();
        assert!(result.is_err());
        match result.unwrap_err() {
            DecoderError::UnexpectedEof => {}
            _ => panic!("Expected UnexpectedEof"),
        }
    }

    #[test]
    fn test_read_u32_be() {
        let data = vec![0x12, 0x34, 0x56, 0x78];
        let mut decoder = TermDecoder::new(&data);
        let value = decoder.read_u32_be().unwrap();
        assert_eq!(value, 0x12345678);
    }

    #[test]
    fn test_read_u32_be_eof() {
        let data = vec![0x12, 0x34, 0x56];
        let mut decoder = TermDecoder::new(&data);
        let result = decoder.read_u32_be();
        assert!(result.is_err());
        match result.unwrap_err() {
            DecoderError::UnexpectedEof => {}
            _ => panic!("Expected UnexpectedEof"),
        }
    }

    #[test]
    fn test_read_i32_be() {
        let data = vec![0x12, 0x34, 0x56, 0x78];
        let mut decoder = TermDecoder::new(&data);
        let value = decoder.read_i32_be().unwrap();
        assert_eq!(value, 0x12345678);
    }

    #[test]
    fn test_read_i32_be_negative() {
        let data = vec![0xFF, 0xFF, 0xFF, 0xFF];
        let mut decoder = TermDecoder::new(&data);
        let value = decoder.read_i32_be().unwrap();
        assert_eq!(value, -1);
    }

    #[test]
    fn test_read_i32_be_eof() {
        let data = vec![0x12, 0x34, 0x56];
        let mut decoder = TermDecoder::new(&data);
        let result = decoder.read_i32_be();
        assert!(result.is_err());
        match result.unwrap_err() {
            DecoderError::UnexpectedEof => {}
            _ => panic!("Expected UnexpectedEof"),
        }
    }

    #[test]
    fn test_decode_complex_nested_structure() {
        // Complex structure: [{1, 2}, [3, 4], {atom, 5}]
        // This is a simplified encoding - real encoding would be more complex
        // We'll test with a simpler nested structure
        // List with tuple: [{1, 2}]
        // [131, 108, 0, 0, 0, 1, 104, 2, 97, 1, 97, 2, 106]
        let data = vec![131, 108, 0, 0, 0, 1, 104, 2, 97, 1, 97, 2, 106];
        let term = decode_term(&data).unwrap();
        match term {
            ErlangTerm::List(elements) => {
                assert_eq!(elements.len(), 1);
                match &elements[0] {
                    ErlangTerm::Tuple(inner) => {
                        assert_eq!(inner.len(), 2);
                        assert_eq!(inner[0], ErlangTerm::Integer(1));
                        assert_eq!(inner[1], ErlangTerm::Integer(2));
                    }
                    _ => panic!("Expected Tuple in list"),
                }
            }
            _ => panic!("Expected List"),
        }
    }
}

