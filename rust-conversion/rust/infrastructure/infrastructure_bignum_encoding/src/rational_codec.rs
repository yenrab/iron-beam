//! Rational Codec Module
//!
//! Provides BigRational encoding/decoding functionality.
//! Since there's no existing BEAM implementation for rational numbers,
//! this implements a custom encoding format.
//!
//! Format:
//! - Tag byte: Custom tag (using tuple format as a container)
//! - Numerator: Encoded as big integer (SMALL_BIG_EXT or LARGE_BIG_EXT)
//! - Denominator: Encoded as big integer (SMALL_BIG_EXT or LARGE_BIG_EXT)
//!
//! We use a tuple format: {numerator, denominator} where both are big integers.

use entities_utilities::BigRational;
use malachite::Integer;

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

/// Encode a malachite Integer as a big integer in EI format
fn encode_big_integer(
    buf: &mut Vec<u8>,
    index: &mut usize,
    value: &Integer,
) -> Result<usize, EncodeError> {
    let start_index = *index;
    
    // Get absolute value and sign
    let is_negative = *value < Integer::from(0);
    let abs_value = if is_negative {
        -value.clone()
    } else {
        value.clone()
    };
    
    // Convert Integer to bytes (little-endian, as per EI format)
    // Extract bytes manually by repeatedly dividing by 256
    let mut byte_vec = Vec::new();
    let mut v = abs_value.clone();
    let base = Integer::from(256u64);
    
    // Extract bytes (little-endian)
    if v == Integer::from(0) {
        byte_vec.push(0);
    } else {
        while v > Integer::from(0) {
            let remainder = &v % &base;
            // Remainder is always < 256, so it fits in u64
            let rem_u64 = u64::try_from(&remainder).unwrap_or(0);
            byte_vec.push(rem_u64 as u8);
            v = &v / &base;
        }
    }
    
    let arity = byte_vec.len();
    
    if arity > 255 {
        // Use LARGE_BIG_EXT
        let needed = 5 + 1 + arity; // tag(1) + arity(4) + sign(1) + bytes
        buf.resize(buf.len().max(*index + needed), 0);
        
        buf[*index] = 111; // ERL_LARGE_BIG_EXT = 111
        *index += 1;
        
        let arity_u32 = arity as u32;
        buf[*index..*index + 4].copy_from_slice(&arity_u32.to_be_bytes());
        *index += 4;
        
        buf[*index] = if is_negative { 1 } else { 0 };
        *index += 1;
        
        buf[*index..*index + arity].copy_from_slice(&byte_vec);
        *index += arity;
    } else {
        // Use SMALL_BIG_EXT
        let needed = 3 + arity; // tag(1) + arity(1) + sign(1) + bytes
        buf.resize(buf.len().max(*index + needed), 0);
        
        buf[*index] = 110; // ERL_SMALL_BIG_EXT = 110
        *index += 1;
        
        buf[*index] = arity as u8;
        *index += 1;
        
        buf[*index] = if is_negative { 1 } else { 0 };
        *index += 1;
        
        buf[*index..*index + arity].copy_from_slice(&byte_vec);
        *index += arity;
    }
    
    Ok(*index - start_index)
}

/// Decode a big integer from EI format
fn decode_big_integer(data: &[u8]) -> Result<(Integer, usize), DecodeError> {
    if data.is_empty() {
        return Err(DecodeError::BufferTooShort);
    }
    
    let mut index = 0;
    let tag = data[index];
    index += 1;
    
    let arity = if tag == 110 {
        // ERL_SMALL_BIG_EXT
        if index >= data.len() {
            return Err(DecodeError::BufferTooShort);
        }
        data[index] as usize
    } else if tag == 111 {
        // ERL_LARGE_BIG_EXT
        if index + 4 > data.len() {
            return Err(DecodeError::BufferTooShort);
        }
        u32::from_be_bytes([data[index], data[index + 1], data[index + 2], data[index + 3]])
            as usize
    } else {
        return Err(DecodeError::InvalidFormat(format!(
            "Expected big integer tag (110 or 111), got {}",
            tag
        )));
    };
    
    if tag == 111 {
        index += 4;
    } else {
        index += 1;
    }
    
    if index >= data.len() {
        return Err(DecodeError::BufferTooShort);
    }
    
    let is_negative = data[index] != 0;
    index += 1;
    
    if index + arity > data.len() {
        return Err(DecodeError::BufferTooShort);
    }
    
    // Read bytes (little-endian)
    let bytes = &data[index..index + arity];
    index += arity;
    
    // Convert bytes to Integer
    // For simplicity, we'll convert to i64 if it fits, otherwise use string parsing
    let mut value = Integer::from(0);
    let mut multiplier = Integer::from(1u64);
    
    for &byte in bytes {
        value += Integer::from(byte) * &multiplier;
        multiplier *= Integer::from(256u64);
    }
    
    if is_negative {
        value = -value;
    }
    
    Ok((value, index))
}

/// Encoding errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodeError {
    /// Buffer too small
    BufferTooSmall,
    /// Value too large to encode
    ValueTooLarge,
    /// Invalid value
    InvalidValue(String),
}

/// Decoding errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// Buffer too short
    BufferTooShort,
    /// Invalid format
    InvalidFormat(String),
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

