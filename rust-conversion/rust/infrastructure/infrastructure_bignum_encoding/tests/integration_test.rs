//! Integration tests for infrastructure_bignum_encoding crate
//!
//! These tests verify that bignum and rational encoding/decoding work correctly
//! and test end-to-end workflows for arbitrary precision numbers.

use infrastructure_bignum_encoding::*;
use entities_utilities::{BigNumber, BigRational};

#[test]
fn test_bignum_codec_encode_decode_zero() {
    let zero = BigNumber::from_i64(0);
    let encoded = BignumCodec::encode(&zero).unwrap();
    
    // Zero should be encoded as SMALL_INTEGER_EXT
    assert_eq!(encoded, vec![97, 0]);
    
    let (decoded, bytes_consumed) = BignumCodec::decode(&encoded).unwrap();
    assert_eq!(decoded, zero);
    assert_eq!(bytes_consumed, 2);
}

#[test]
fn test_bignum_codec_encode_decode_small_positive() {
    let num = BigNumber::from_i64(42);
    let encoded = BignumCodec::encode(&num).unwrap();
    let (decoded, bytes_consumed) = BignumCodec::decode(&encoded).unwrap();
    
    assert_eq!(decoded, num);
    assert_eq!(bytes_consumed, encoded.len());
}

#[test]
fn test_bignum_codec_encode_decode_small_negative() {
    let num = BigNumber::from_i64(-42);
    let encoded = BignumCodec::encode(&num).unwrap();
    let (decoded, bytes_consumed) = BignumCodec::decode(&encoded).unwrap();
    
    assert_eq!(decoded, num);
    assert_eq!(bytes_consumed, encoded.len());
}

#[test]
fn test_bignum_codec_encode_decode_large_positive() {
    let num = BigNumber::from_i64(i64::MAX);
    let encoded = BignumCodec::encode(&num).unwrap();
    let (decoded, bytes_consumed) = BignumCodec::decode(&encoded).unwrap();
    
    assert_eq!(decoded, num);
    assert_eq!(bytes_consumed, encoded.len());
}

#[test]
fn test_bignum_codec_encode_decode_large_negative() {
    let num = BigNumber::from_i64(i64::MIN);
    let encoded = BignumCodec::encode(&num).unwrap();
    let (decoded, bytes_consumed) = BignumCodec::decode(&encoded).unwrap();
    
    assert_eq!(decoded, num);
    assert_eq!(bytes_consumed, encoded.len());
}

#[test]
fn test_bignum_codec_roundtrip_various_values() {
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
        
        assert_eq!(decoded, value);
        assert_eq!(bytes_consumed, encoded.len());
    }
}

#[test]
fn test_bignum_codec_decode_invalid_format() {
    let invalid = vec![100, 2]; // Wrong tag
    let result = BignumCodec::decode(&invalid);
    assert!(result.is_err());
}

#[test]
fn test_bignum_codec_decode_buffer_too_short() {
    let incomplete = vec![110]; // SMALL_BIG_EXT but no data
    let result = BignumCodec::decode(&incomplete);
    assert!(result.is_err());
}

#[test]
fn test_bignum_codec_decode_empty_buffer() {
    let empty = vec![];
    let result = BignumCodec::decode(&empty);
    assert!(result.is_err());
}

#[test]
fn test_rational_codec_encode_decode_simple() {
    let rational = BigRational::from_fraction(1, 2).unwrap();
    let encoded = RationalCodec::encode(&rational).unwrap();
    let (decoded, bytes_consumed) = RationalCodec::decode(&encoded).unwrap();
    
    assert_eq!(decoded, rational);
    assert_eq!(bytes_consumed, encoded.len());
}

#[test]
fn test_rational_codec_encode_decode_negative() {
    let rational = BigRational::from_fraction(-22, 7).unwrap();
    let encoded = RationalCodec::encode(&rational).unwrap();
    let (decoded, bytes_consumed) = RationalCodec::decode(&encoded).unwrap();
    
    assert_eq!(decoded, rational);
    assert_eq!(bytes_consumed, encoded.len());
}

#[test]
fn test_rational_codec_encode_decode_integer() {
    let rational = BigRational::from_i64(42);
    let encoded = RationalCodec::encode(&rational).unwrap();
    let (decoded, bytes_consumed) = RationalCodec::decode(&encoded).unwrap();
    
    assert_eq!(decoded, rational);
    assert_eq!(bytes_consumed, encoded.len());
}

#[test]
fn test_rational_codec_encode_decode_large() {
    let rational = BigRational::from_fraction(123456789, 987654321).unwrap();
    let encoded = RationalCodec::encode(&rational).unwrap();
    let (decoded, bytes_consumed) = RationalCodec::decode(&encoded).unwrap();
    
    assert_eq!(decoded, rational);
    assert_eq!(bytes_consumed, encoded.len());
}

#[test]
fn test_rational_codec_roundtrip_various_values() {
    let test_values = vec![
        BigRational::from_i64(0),
        BigRational::from_i64(1),
        BigRational::from_i64(-1),
        BigRational::from_fraction(1, 2).unwrap(),
        BigRational::from_fraction(-1, 2).unwrap(),
        BigRational::from_fraction(22, 7).unwrap(), // Approximation of π
        BigRational::from_fraction(355, 113).unwrap(), // Better approximation of π
        BigRational::from_fraction(123456789, 987654321).unwrap(),
    ];
    
    for value in test_values {
        let encoded = RationalCodec::encode(&value).unwrap();
        let (decoded, bytes_consumed) = RationalCodec::decode(&encoded).unwrap();
        
        assert_eq!(decoded, value);
        assert_eq!(bytes_consumed, encoded.len());
    }
}

#[test]
fn test_rational_codec_decode_invalid_format() {
    let invalid = vec![100, 2]; // Wrong tag (not tuple)
    let result = RationalCodec::decode(&invalid);
    assert!(result.is_err());
}

#[test]
fn test_rational_codec_decode_wrong_arity() {
    // Create invalid encoding with wrong tuple arity
    let mut invalid = vec![104, 3]; // tuple header with arity 3 instead of 2
    invalid.extend_from_slice(&[110, 1, 0, 1]); // numerator
    invalid.extend_from_slice(&[110, 1, 0, 1]); // denominator
    invalid.extend_from_slice(&[110, 1, 0, 1]); // extra element
    
    let result = RationalCodec::decode(&invalid);
    assert!(result.is_err());
}

#[test]
fn test_rational_codec_decode_zero_denominator() {
    // Create invalid encoding with zero denominator
    let mut invalid = vec![104, 2]; // tuple header
    invalid.extend_from_slice(&[110, 1, 0, 1]); // numerator = 1 (SMALL_BIG_EXT)
    invalid.extend_from_slice(&[110, 1, 0, 0]); // denominator = 0 (SMALL_BIG_EXT)
    
    let result = RationalCodec::decode(&invalid);
    assert!(result.is_err());
}

#[test]
fn test_rational_codec_decode_buffer_too_short() {
    let incomplete = vec![104]; // Tuple tag but no data
    let result = RationalCodec::decode(&incomplete);
    assert!(result.is_err());
}

#[test]
fn test_rational_codec_decode_empty_buffer() {
    let empty = vec![];
    let result = RationalCodec::decode(&empty);
    assert!(result.is_err());
}

// Note: encode_big_integer and decode_big_integer are in private common module
// Tests for these are covered by BignumCodec tests which use them internally

#[test]
fn test_integer_to_bytes_positive() {
    use malachite::Integer;
    use infrastructure_bignum_encoding::integer_to_bytes;
    
    let value = Integer::from(42);
    let (bytes, is_negative) = integer_to_bytes(&value);
    
    assert!(!is_negative);
    assert!(!bytes.is_empty());
}

#[test]
fn test_integer_to_bytes_negative() {
    use malachite::Integer;
    use infrastructure_bignum_encoding::integer_to_bytes;
    
    let value = Integer::from(-42);
    let (bytes, is_negative) = integer_to_bytes(&value);
    
    assert!(is_negative);
    assert!(!bytes.is_empty());
}

#[test]
fn test_integer_to_bytes_zero() {
    use malachite::Integer;
    use infrastructure_bignum_encoding::integer_to_bytes;
    
    let value = Integer::from(0);
    let (bytes, is_negative) = integer_to_bytes(&value);
    
    assert!(!is_negative);
    assert_eq!(bytes, vec![0]);
}

#[test]
fn test_bytes_to_integer_roundtrip() {
    use malachite::Integer;
    use infrastructure_bignum_encoding::{integer_to_bytes, bytes_to_integer};
    
    let test_values = vec![
        Integer::from(0),
        Integer::from(1),
        Integer::from(-1),
        Integer::from(42),
        Integer::from(-42),
        Integer::from(255),
        Integer::from(256),
    ];
    
    for value in test_values {
        let (bytes, is_negative) = integer_to_bytes(&value);
        let decoded = bytes_to_integer(&bytes, is_negative);
        assert_eq!(decoded, value);
    }
}

#[test]
fn test_encode_error_enum() {
    // Test EncodeError enum variants
    // Note: Check actual error variants from the crate
    let error1 = EncodeError::BufferTooSmall;
    let _ = format!("{:?}", error1);
}

#[test]
fn test_decode_error_enum() {
    // Test DecodeError enum variants
    let errors = vec![
        DecodeError::BufferTooShort,
        DecodeError::InvalidFormat("test".to_string()),
    ];
    
    for error in &errors {
        let _ = format!("{:?}", error);
    }
}

#[test]
fn test_bignum_codec_very_large_number() {
    // Test with a number that requires LARGE_BIG_EXT
    let mut num = BigNumber::from_i64(1);
    // Multiply by 2^100 to get a large number
    for _ in 0..100 {
        num = num.times(&BigNumber::from_i64(2));
    }
    
    let encoded = BignumCodec::encode(&num).unwrap();
    // Should use LARGE_BIG_EXT (tag 111) or SMALL_BIG_EXT (tag 110) depending on size
    assert!(encoded[0] == 110 || encoded[0] == 111);
    
    let (decoded, bytes_consumed) = BignumCodec::decode(&encoded).unwrap();
    assert_eq!(decoded, num);
    assert_eq!(bytes_consumed, encoded.len());
}

#[test]
fn test_rational_codec_complex_fractions() {
    // Test with complex fractions
    let rational1 = BigRational::from_fraction(22, 7).unwrap(); // π approximation
    let rational2 = BigRational::from_fraction(355, 113).unwrap(); // Better π approximation
    
    let encoded1 = RationalCodec::encode(&rational1).unwrap();
    let encoded2 = RationalCodec::encode(&rational2).unwrap();
    
    let (decoded1, _) = RationalCodec::decode(&encoded1).unwrap();
    let (decoded2, _) = RationalCodec::decode(&encoded2).unwrap();
    
    assert_eq!(decoded1, rational1);
    assert_eq!(decoded2, rational2);
}

#[test]
fn test_bignum_rational_integration() {
    // Test that bignums and rationals work together
    let bignum = BigNumber::from_i64(42);
    let rational = BigRational::from_i64(42);
    
    // Both should encode/decode correctly
    let bignum_encoded = BignumCodec::encode(&bignum).unwrap();
    let rational_encoded = RationalCodec::encode(&rational).unwrap();
    
    let (bignum_decoded, _) = BignumCodec::decode(&bignum_encoded).unwrap();
    let (rational_decoded, _) = RationalCodec::decode(&rational_encoded).unwrap();
    
    assert_eq!(bignum_decoded, bignum);
    assert_eq!(rational_decoded, rational);
}

