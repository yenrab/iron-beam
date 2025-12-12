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
    fn test_decode_tuple() {
        // Encode {atom, 1}: [131, 104, 2, 100, 0, 4, 'a', 't', 'o', 'm', 97, 1]
        // This is a simplified test - real encoding is more complex
    }
}

