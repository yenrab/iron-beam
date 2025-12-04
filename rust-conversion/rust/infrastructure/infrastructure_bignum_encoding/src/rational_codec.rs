//! Rational Codec Module
//!
//! Provides BigRational encoding and decoding functionality for arbitrary precision
//! rational numbers in the Erlang/OTP runtime system. Since there's no existing
//! BEAM implementation for rational numbers, this module implements a custom encoding
//! format using tuples.
//!
//! ## Overview
//!
//! Rational numbers are represented as fractions with arbitrary precision numerators
//! and denominators. This module provides codecs for serializing and deserializing
//! rational numbers in the EI format.
//!
//! ## Encoding Format
//!
//! Rational numbers are encoded as tuples containing two big integers:
//! - **Tuple header**: `ERL_SMALL_TUPLE_EXT` with arity 2
//! - **Numerator**: Encoded as big integer (SMALL_BIG_EXT or LARGE_BIG_EXT)
//! - **Denominator**: Encoded as big integer (SMALL_BIG_EXT or LARGE_BIG_EXT, always positive)
//!
//! The sign is encoded in the numerator (negative numerator = negative rational).
//!
//! ## Examples
//!
//! ```rust
//! use infrastructure_bignum_encoding::RationalCodec;
//! use entities_utilities::BigRational;
//!
//! // Encode a rational number
//! let rational = BigRational::from(22, 7); // Approximation of π
//! let encoded = RationalCodec::encode(&rational).unwrap();
//!
//! // Decode a rational number
//! let (decoded, bytes_consumed) = RationalCodec::decode(&encoded).unwrap();
//! assert_eq!(decoded, rational);
//! ```
//!
//! ## See Also
//!
//! - [`bignum_codec`](super::bignum_codec/index.html): Big integer codec
//! - [`entities_utilities::BigRational`](../../entities/entities_utilities/rational/index.html): BigRational type
//! - [`common`](super::common/index.html): Shared encoding/decoding utilities

use entities_utilities::BigRational;
use malachite::Integer;

use crate::common::{encode_big_integer, decode_big_integer, EncodeError, DecodeError};

/// Rational codec for encoding/decoding BigRational values
pub struct RationalCodec;

impl RationalCodec {
    /// Encode a BigRational to bytes using EI format
    ///
    /// The encoding format is a tuple containing two big integers:
    /// {numerator, denominator}
    ///
    /// # Arguments
    /// * `value` - The BigRational value to encode
    ///
    /// # Returns
    /// * `Ok(bytes)` - Encoded bytes
    /// * `Err(EncodeError)` - Encoding error
    pub fn encode(value: &BigRational) -> Result<Vec<u8>, EncodeError> {
        let mut buf = Vec::new();
        let mut index = 0;
        
        // Get numerator and denominator as Integers
        // Note: numerator() and denominator() return absolute values
        // The sign is stored in the Rational itself
        let numerator_abs = value.numerator();
        let denominator_abs = value.denominator();
        
        // Check if the rational is negative and apply sign to numerator
        let numerator = if value.is_negative() {
            -numerator_abs.clone()
        } else {
            numerator_abs.clone()
        };
        
        // Encode numerator as big integer (with sign)
        let num_bytes = encode_big_integer(&mut buf, &mut index, &numerator)?;
        
        // Encode denominator as big integer (always positive)
        let den_bytes = encode_big_integer(&mut buf, &mut index, &denominator_abs)?;
        
        // Wrap in a tuple: {numerator, denominator}
        // Calculate total size needed
        let tuple_size = 2 + num_bytes + den_bytes; // 2 bytes for small tuple header + data
        let mut result = Vec::with_capacity(tuple_size);
        
        // Write tuple header (small tuple with arity 2)
        result.push(104); // ERL_SMALL_TUPLE_EXT = 104
        result.push(2); // arity = 2
        
        // Append the encoded numerator and denominator
        result.extend_from_slice(&buf);
        
        Ok(result)
    }

    /// Decode a BigRational from bytes in EI format
    ///
    /// # Arguments
    /// * `data` - The encoded bytes
    ///
    /// # Returns
    /// * `Ok((rational, bytes_consumed))` - Decoded BigRational and bytes consumed
    /// * `Err(DecodeError)` - Decoding error
    pub fn decode(data: &[u8]) -> Result<(BigRational, usize), DecodeError> {
        if data.is_empty() {
            return Err(DecodeError::BufferTooShort);
        }
        
        let mut index = 0;
        
        // Decode tuple header
        let tag = data[index];
        index += 1;
        
        if tag != 104 {
            // ERL_SMALL_TUPLE_EXT = 104
            return Err(DecodeError::InvalidFormat(format!(
                "Expected tuple tag (104), got {}",
                tag
            )));
        }
        
        if index >= data.len() {
            return Err(DecodeError::BufferTooShort);
        }
        
        let arity = data[index] as usize;
        index += 1;
        
        if arity != 2 {
            return Err(DecodeError::InvalidFormat(format!(
                "Expected tuple arity 2, got {}",
                arity
            )));
        }
        
        // Decode numerator
        let (numerator, num_bytes) = decode_big_integer(&data[index..])?;
        index += num_bytes;
        
        // Decode denominator (should always be positive for rational numbers)
        let (denominator_raw, den_bytes) = decode_big_integer(&data[index..])?;
        index += den_bytes;
        
        // Ensure denominator is positive (rational numbers always have positive denominators)
        let denominator = if denominator_raw < Integer::from(0) {
            -denominator_raw
        } else {
            denominator_raw
        };
        
        // Create BigRational from numerator and denominator
        // Check for zero denominator
        if denominator == Integer::from(0) {
            return Err(DecodeError::InvalidFormat(
                "Denominator cannot be zero".to_string(),
            ));
        }
        
        // Convert malachite Integer to Rational and create BigRational
        // We need to create a Rational from two Integers
        use malachite::Rational as MalachiteRational;
        let malachite_rational = MalachiteRational::from(numerator.clone()) / MalachiteRational::from(denominator.clone());
        let rational = BigRational::from_rational(malachite_rational);
        
        Ok((rational, index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_simple() {
        let rational = BigRational::from_fraction(1, 2).unwrap();
        let encoded = RationalCodec::encode(&rational).unwrap();
        let (decoded, bytes_consumed) = RationalCodec::decode(&encoded).unwrap();
        
        assert_eq!(decoded, rational);
        assert_eq!(bytes_consumed, encoded.len());
    }

    #[test]
    fn test_encode_decode_negative() {
        let rational = BigRational::from_fraction(-22, 7).unwrap();
        let encoded = RationalCodec::encode(&rational).unwrap();
        let (decoded, bytes_consumed) = RationalCodec::decode(&encoded).unwrap();
        
        assert_eq!(decoded, rational);
        assert_eq!(bytes_consumed, encoded.len());
    }

    #[test]
    fn test_encode_decode_integer() {
        let rational = BigRational::from_i64(42);
        let encoded = RationalCodec::encode(&rational).unwrap();
        let (decoded, bytes_consumed) = RationalCodec::decode(&encoded).unwrap();
        
        assert_eq!(decoded, rational);
        assert_eq!(bytes_consumed, encoded.len());
    }

    #[test]
    fn test_encode_decode_large() {
        let rational = BigRational::from_fraction(123456789, 987654321).unwrap();
        let encoded = RationalCodec::encode(&rational).unwrap();
        let (decoded, bytes_consumed) = RationalCodec::decode(&encoded).unwrap();
        
        assert_eq!(decoded, rational);
        assert_eq!(bytes_consumed, encoded.len());
    }

    #[test]
    fn test_decode_invalid_format() {
        let invalid = vec![100, 2]; // Wrong tag
        let result = RationalCodec::decode(&invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_zero_denominator() {
        // Create invalid encoding with zero denominator
        // This is a bit tricky since we need to manually construct invalid data
        // For now, just test that our code handles it
        let mut invalid = vec![104, 2]; // tuple header
        invalid.extend_from_slice(&[110, 1, 0, 1]); // numerator = 1 (SMALL_BIG_EXT)
        invalid.extend_from_slice(&[110, 1, 0, 0]); // denominator = 0 (SMALL_BIG_EXT)
        
        let result = RationalCodec::decode(&invalid);
        assert!(result.is_err());
    }
}

