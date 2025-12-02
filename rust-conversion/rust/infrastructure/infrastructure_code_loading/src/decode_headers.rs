//! Header Decoding Module
//!
//! Provides functionality to decode tuple and list headers from EI format.
//! Based on lib/erl_interface/src/decode/decode_tuple_header.c

use crate::constants::*;

/// Decode a tuple header from EI format
///
/// # Arguments
/// * `buf` - Buffer containing EI-encoded data
/// * `index` - Current index in buffer
///
/// # Returns
/// * `Ok((arity, new_index))` - Arity and new index
/// * `Err(DecodeError)` - Decoding error
pub fn decode_tuple_header(buf: &[u8], index: &mut usize) -> Result<usize, DecodeError> {
    if *index >= buf.len() {
        return Err(DecodeError::BufferTooShort);
    }

    let tag = buf[*index];
    *index += 1;

    match tag {
        ERL_SMALL_TUPLE_EXT => {
            if *index >= buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let arity = buf[*index] as usize;
            *index += 1;
            Ok(arity)
        }
        ERL_LARGE_TUPLE_EXT => {
            if *index + 4 > buf.len() {
                return Err(DecodeError::BufferTooShort);
            }
            let arity = u32::from_be_bytes([
                buf[*index],
                buf[*index + 1],
                buf[*index + 2],
                buf[*index + 3],
            ]) as usize;
            *index += 4;
            Ok(arity)
        }
        _ => Err(DecodeError::InvalidFormat(format!("Unexpected tag: {}", tag))),
    }
}

/// Decode a map header from EI format
///
/// # Arguments
/// * `buf` - Buffer containing EI-encoded data
/// * `index` - Current index in buffer
///
/// # Returns
/// * `Ok((arity, new_index))` - Arity (number of key-value pairs) and new index
/// * `Err(DecodeError)` - Decoding error
pub fn decode_map_header(buf: &[u8], index: &mut usize) -> Result<usize, DecodeError> {
    if *index >= buf.len() {
        return Err(DecodeError::BufferTooShort);
    }

    let tag = buf[*index];
    *index += 1;

    if tag != ERL_MAP_EXT {
        return Err(DecodeError::InvalidFormat(format!("Unexpected tag: {}", tag)));
    }

    if *index + 4 > buf.len() {
        return Err(DecodeError::BufferTooShort);
    }

    let arity = u32::from_be_bytes([
        buf[*index],
        buf[*index + 1],
        buf[*index + 2],
        buf[*index + 3],
    ]) as usize;
    *index += 4;

    Ok(arity)
}

/// Decode a list header from EI format
///
/// # Arguments
/// * `buf` - Buffer containing EI-encoded data
/// * `index` - Current index in buffer
///
/// # Returns
/// * `Ok((length, new_index))` - Length and new index
/// * `Err(DecodeError)` - Decoding error
pub fn decode_list_header(buf: &[u8], index: &mut usize) -> Result<usize, DecodeError> {
    if *index >= buf.len() {
        return Err(DecodeError::BufferTooShort);
    }

    let tag = buf[*index];
    *index += 1;

    if tag != ERL_LIST_EXT {
        return Err(DecodeError::InvalidFormat(format!("Unexpected tag: {}", tag)));
    }

    if *index + 4 > buf.len() {
        return Err(DecodeError::BufferTooShort);
    }

    let length = u32::from_be_bytes([
        buf[*index],
        buf[*index + 1],
        buf[*index + 2],
        buf[*index + 3],
    ]) as usize;
    *index += 4;

    Ok(length)
}

/// Decoding errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// Buffer is too short
    BufferTooShort,
    /// Invalid format
    InvalidFormat(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_tuple_header_small() {
        let buf = vec![ERL_SMALL_TUPLE_EXT, 3];
        let mut index = 0;
        let arity = decode_tuple_header(&buf, &mut index).unwrap();
        assert_eq!(arity, 3);
        assert_eq!(index, 2);
    }

    #[test]
    fn test_decode_tuple_header_large() {
        let mut buf = vec![ERL_LARGE_TUPLE_EXT];
        buf.extend_from_slice(&300u32.to_be_bytes());
        let mut index = 0;
        let arity = decode_tuple_header(&buf, &mut index).unwrap();
        assert_eq!(arity, 300);
        assert_eq!(index, 5);
    }

    #[test]
    fn test_decode_map_header() {
        let mut buf = vec![ERL_MAP_EXT];
        buf.extend_from_slice(&5u32.to_be_bytes());
        let mut index = 0;
        let arity = decode_map_header(&buf, &mut index).unwrap();
        assert_eq!(arity, 5);
        assert_eq!(index, 5);
    }

    #[test]
    fn test_decode_list_header() {
        let mut buf = vec![ERL_LIST_EXT];
        buf.extend_from_slice(&10u32.to_be_bytes());
        let mut index = 0;
        let length = decode_list_header(&buf, &mut index).unwrap();
        assert_eq!(length, 10);
        assert_eq!(index, 5);
    }

    #[test]
    fn test_decode_roundtrip() {
        // Test tuple header roundtrip
        for arity in [3, 255, 256, 1000] {
            let mut buf = vec![0u8; 10];
            let mut encode_index = 0;
            crate::encode_headers::encode_tuple_header(&mut Some(&mut buf), &mut encode_index, arity).unwrap();
            
            let mut decode_index = 0;
            let decoded = decode_tuple_header(&buf, &mut decode_index).unwrap();
            assert_eq!(decoded, arity, "Roundtrip failed for arity {}", arity);
        }
    }
}

