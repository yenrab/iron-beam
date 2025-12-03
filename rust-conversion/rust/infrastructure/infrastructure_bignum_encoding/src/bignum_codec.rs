//! Bignum Codec Module
//!
//! Provides bignum encoding/decoding functionality.
//! Based on decode_big.c and encode_bignum.c
//!
//! This module encodes/decodes BigNumber values (arbitrary precision integers)
//! using the EI (Erlang Interchange) format:
//!
//! - SMALL_BIG_EXT (tag 110): For values with ≤255 bytes
//! - LARGE_BIG_EXT (tag 111): For larger values
//!
//! The encoding format matches the C implementation in encode_bignum.c.

use entities_utilities::BigNumber;
use malachite::Integer;

use crate::common::{encode_big_integer, decode_big_integer, EncodeError, DecodeError};

/// Bignum codec for encoding/decoding BigNumber values
pub struct BignumCodec;

impl BignumCodec {
    /// Encode a BigNumber to bytes using EI format
    ///
    /// This function encodes a BigNumber into the EI (Erlang Interchange) format.
    /// The encoding uses SMALL_BIG_EXT for values with ≤255 bytes, and LARGE_BIG_EXT
    /// for larger values. Zero values are encoded as SMALL_INTEGER_EXT (tag 97, value 0).
    ///
    /// # Arguments
    ///
    /// * `value` - The BigNumber value to encode
    ///
    /// # Returns
    ///
    /// * `Ok(bytes)` - Encoded bytes in EI format
    /// * `Err(EncodeError)` - Encoding error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use infrastructure_bignum_encoding::BignumCodec;
    /// use entities_utilities::BigNumber;
    ///
    /// let num = BigNumber::from_i64(12345);
    /// let encoded = BignumCodec::encode(&num).unwrap();
    /// ```
    pub fn encode(value: &BigNumber) -> Result<Vec<u8>, EncodeError> {
        let integer = value.as_integer();
        
        // Special case: zero is encoded as SMALL_INTEGER_EXT (tag 97, value 0)
        // This matches the C implementation behavior
        if *integer == Integer::from(0) {
            return Ok(vec![97, 0]); // ERL_SMALL_INTEGER_EXT = 97
        }
        
        let mut buf = Vec::new();
        let mut index = 0;
        
        // Encode the integer using the shared helper function
        encode_big_integer(&mut buf, &mut index, integer)?;
        
        Ok(buf)
    }

    /// Decode a BigNumber from bytes in EI format
    ///
    /// This function decodes a BigNumber from the EI (Erlang Interchange) format.
    /// It supports both SMALL_BIG_EXT and LARGE_BIG_EXT formats, as well as
    /// SMALL_INTEGER_EXT for zero values.
    ///
    /// # Arguments
    ///
    /// * `data` - The encoded bytes to decode
    ///
    /// # Returns
    ///
    /// * `Ok((bignum, bytes_consumed))` - Decoded BigNumber and number of bytes consumed
    /// * `Err(DecodeError)` - Decoding error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use infrastructure_bignum_encoding::BignumCodec;
    /// use entities_utilities::BigNumber;
    ///
    /// let num = BigNumber::from_i64(12345);
    /// let encoded = BignumCodec::encode(&num).unwrap();
    /// let (decoded, bytes_consumed) = BignumCodec::decode(&encoded).unwrap();
    /// assert_eq!(decoded, num);
    /// ```
    pub fn decode(data: &[u8]) -> Result<(BigNumber, usize), DecodeError> {
        if data.is_empty() {
            return Err(DecodeError::BufferTooShort);
        }
        
        let tag = data[0];
        
        // Handle SMALL_INTEGER_EXT for zero (tag 97)
        if tag == 97 && data.len() >= 2 && data[1] == 0 {
            return Ok((BigNumber::from_i64(0), 2));
        }
        
        // Decode using the shared helper function
        let (integer, bytes_consumed) = decode_big_integer(data)?;
        let bignum = BigNumber::from_integer(integer);
        
        Ok((bignum, bytes_consumed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_zero() {
        let zero = BigNumber::from_i64(0);
        let encoded = BignumCodec::encode(&zero).unwrap();
        // Zero should be encoded as SMALL_INTEGER_EXT
        assert_eq!(encoded, vec![97, 0]);
        
        let (decoded, bytes_consumed) = BignumCodec::decode(&encoded).unwrap();
        assert_eq!(decoded, zero);
        assert_eq!(bytes_consumed, 2);
    }

    #[test]
    fn test_encode_decode_small_positive() {
        let num = BigNumber::from_i64(42);
        let encoded = BignumCodec::encode(&num).unwrap();
        let (decoded, bytes_consumed) = BignumCodec::decode(&encoded).unwrap();
        
        assert_eq!(decoded, num);
        assert_eq!(bytes_consumed, encoded.len());
    }

    #[test]
    fn test_encode_decode_small_negative() {
        let num = BigNumber::from_i64(-42);
        let encoded = BignumCodec::encode(&num).unwrap();
        let (decoded, bytes_consumed) = BignumCodec::decode(&encoded).unwrap();
        
        assert_eq!(decoded, num);
        assert_eq!(bytes_consumed, encoded.len());
    }

    #[test]
    fn test_encode_decode_large_positive() {
        // Create a number that requires SMALL_BIG_EXT
        let num = BigNumber::from_i64(i64::MAX);
        let encoded = BignumCodec::encode(&num).unwrap();
        let (decoded, bytes_consumed) = BignumCodec::decode(&encoded).unwrap();
        
        assert_eq!(decoded, num);
        assert_eq!(bytes_consumed, encoded.len());
    }

    #[test]
    fn test_encode_decode_large_negative() {
        // Create a number that requires SMALL_BIG_EXT
        let num = BigNumber::from_i64(i64::MIN);
        let encoded = BignumCodec::encode(&num).unwrap();
        let (decoded, bytes_consumed) = BignumCodec::decode(&encoded).unwrap();
        
        assert_eq!(decoded, num);
        assert_eq!(bytes_consumed, encoded.len());
    }

    #[test]
    fn test_encode_decode_very_large() {
        // Create a number that requires LARGE_BIG_EXT (>255 bytes)
        // We'll create a number with 256 bytes worth of data
        let mut num = BigNumber::from_i64(1);
        // Multiply by 2^2048 to get a large number
        for _ in 0..2048 {
            num = num.times(&BigNumber::from_i64(2));
        }
        
        let encoded = BignumCodec::encode(&num).unwrap();
        // Should use LARGE_BIG_EXT (tag 111)
        assert_eq!(encoded[0], 111);
        
        let (decoded, bytes_consumed) = BignumCodec::decode(&encoded).unwrap();
        assert_eq!(decoded, num);
        assert_eq!(bytes_consumed, encoded.len());
    }

    #[test]
    fn test_decode_invalid_format() {
        let invalid = vec![100, 2]; // Wrong tag
        let result = BignumCodec::decode(&invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_buffer_too_short() {
        let incomplete = vec![110]; // SMALL_BIG_EXT but no data
        let result = BignumCodec::decode(&incomplete);
        assert!(result.is_err());
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let test_values = vec![
            BigNumber::from_i64(0),
            BigNumber::from_i64(1),
            BigNumber::from_i64(-1),
            BigNumber::from_i64(255),
            BigNumber::from_i64(256),
            BigNumber::from_i64(-256),
            BigNumber::from_i64(i32::MAX as i64),
            BigNumber::from_i64(i32::MIN as i64),
            BigNumber::from_i64(i64::MAX),
            BigNumber::from_i64(i64::MIN),
        ];
        
        for value in test_values {
            let encoded = BignumCodec::encode(&value).unwrap();
            let (decoded, bytes_consumed) = BignumCodec::decode(&encoded).unwrap();
            assert_eq!(decoded, value, "Roundtrip failed for {}", value.to_i64().unwrap_or(0));
            assert_eq!(bytes_consumed, encoded.len());
        }
    }
}

