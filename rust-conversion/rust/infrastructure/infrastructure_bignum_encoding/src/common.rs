//! Common Encoding/Decoding Utilities
//!
//! Provides shared helper functions for encoding and decoding big integers in
//! EI (Erlang Interface) format. This module contains the core encoding/decoding
//! logic used by both bignum and rational codecs.
//!
//! ## Overview
//!
//! This module provides low-level functions for encoding and decoding malachite
//! `Integer` values in the EI format. The functions handle:
//! - Format selection (SMALL_BIG_EXT vs LARGE_BIG_EXT)
//! - Sign encoding
//! - Byte extraction (little-endian)
//! - Error handling
//!
//! ## Encoding Format
//!
//! - **SMALL_BIG_EXT** (tag 110): 1 byte tag + 1 byte arity + 1 byte sign + n bytes (little-endian)
//! - **LARGE_BIG_EXT** (tag 111): 1 byte tag + 4 bytes arity (big-endian) + 1 byte sign + n bytes (little-endian)
//!
//! ## See Also
//!
//! - [`bignum_codec`](super::bignum_codec/index.html): Big number codec (uses these functions)
//! - [`rational_codec`](super::rational_codec/index.html): Rational codec (uses these functions)

use malachite::Integer;

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

/// Encode a malachite Integer as a big integer in EI format
///
/// This function encodes a malachite Integer into the EI (Erlang Interchange)
/// format, which supports both SMALL_BIG_EXT (for values with ≤255 bytes)
/// and LARGE_BIG_EXT (for larger values).
///
/// # Format
///
/// - SMALL_BIG_EXT (tag 110): 1 byte tag + 1 byte arity + 1 byte sign + n bytes (little-endian)
/// - LARGE_BIG_EXT (tag 111): 1 byte tag + 4 bytes arity (big-endian) + 1 byte sign + n bytes (little-endian)
///
/// # Arguments
///
/// * `buf` - Buffer to write encoded bytes to
/// * `index` - Current position in buffer (updated after encoding)
/// * `value` - The Integer value to encode
///
/// # Returns
///
/// * `Ok(bytes_written)` - Number of bytes written
/// * `Err(EncodeError)` - Encoding error
pub fn encode_big_integer(
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
///
/// This function decodes a malachite Integer from the EI (Erlang Interchange)
/// format, supporting both SMALL_BIG_EXT and LARGE_BIG_EXT formats.
///
/// # Arguments
///
/// * `data` - The encoded bytes to decode
///
/// # Returns
///
/// * `Ok((integer, bytes_consumed))` - Decoded Integer and number of bytes consumed
/// * `Err(DecodeError)` - Decoding error
pub fn decode_big_integer(data: &[u8]) -> Result<(Integer, usize), DecodeError> {
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

/// Extract bytes from a malachite Integer (little-endian)
///
/// This function extracts the raw bytes from an Integer value without
/// any EI format tags. It's useful for in-memory representations like
/// heap-allocated bignums.
///
/// # Arguments
///
/// * `value` - The Integer value
///
/// # Returns
///
/// * `(bytes, is_negative)` - Byte vector (little-endian) and sign flag
pub fn integer_to_bytes(value: &Integer) -> (Vec<u8>, bool) {
    // Get absolute value and sign
    let is_negative = *value < Integer::from(0);
    let abs_value = if is_negative {
        -value.clone()
    } else {
        value.clone()
    };
    
    // Convert Integer to bytes (little-endian)
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
    
    (byte_vec, is_negative)
}

/// Convert bytes to a malachite Integer (little-endian)
///
/// This function reconstructs an Integer from raw bytes without
/// any EI format parsing. It's useful for in-memory representations
/// like heap-allocated bignums.
///
/// # Arguments
///
/// * `bytes` - Byte vector (little-endian)
/// * `is_negative` - Whether the value is negative
///
/// # Returns
///
/// * `Integer` - The reconstructed Integer value
pub fn bytes_to_integer(bytes: &[u8], is_negative: bool) -> Integer {
    // Convert bytes to Integer (little-endian)
    let mut value = Integer::from(0);
    let mut multiplier = Integer::from(1u64);
    
    for &byte in bytes {
        value += Integer::from(byte) * &multiplier;
        multiplier *= Integer::from(256u64);
    }
    
    if is_negative {
        -value
    } else {
        value
    }
}

