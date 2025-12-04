//! Header Encoding Module
//!
//! Provides functionality to encode compound type headers (tuples, lists, maps) to
//! EI (Erlang Interface) format. Headers specify the structure and size of compound
//! types before encoding their elements.
//!
//! ## Overview
//!
//! Compound types in EI format require headers that specify:
//! - **Type tag**: Identifies the type (tuple, list, map)
//! - **Arity/Length**: Number of elements or key-value pairs
//!
//! ## Header Types
//!
//! - **Tuple Headers**: `ERL_SMALL_TUPLE_EXT` (≤ 255 elements) or `ERL_LARGE_TUPLE_EXT` (> 255)
//! - **List Headers**: `ERL_LIST_EXT` (with length) or `ERL_NIL_EXT` (empty list)
//! - **Map Headers**: `ERL_MAP_EXT` (always 4-byte arity)
//!
//! ## Examples
//!
//! ```rust
//! use infrastructure_code_loading::encode_headers::*;
//!
//! let mut buf = vec![0u8; 100];
//! let mut index = 0;
//!
//! // Encode a tuple header
//! encode_tuple_header(&mut Some(&mut buf), &mut index, 3)?;
//! // Then encode 3 elements...
//!
//! // Encode a list header
//! encode_list_header(&mut Some(&mut buf), &mut index, 5)?;
//! // Then encode 5 elements...
//! ```
//!
//! ## See Also
//!
//! - [`decode_headers`](super::decode_headers/index.html): Header decoding functions
//! - [`encode_integers`](super::encode_integers/index.html): Integer encoding for arity values
//!
//! Based on `lib/erl_interface/src/encode/encode_tuple_header.c`

use crate::constants::*;

/// Encode a tuple header to EI format
///
/// # Arguments
/// * `buf` - Optional buffer to write to (None for size calculation)
/// * `index` - Current index in buffer
/// * `arity` - Number of elements in the tuple
///
/// # Returns
/// * `Ok(())` - Success
/// * `Err(EncodeError)` - Encoding error
pub fn encode_tuple_header(buf: &mut Option<&mut [u8]>, index: &mut usize, arity: usize) -> Result<(), EncodeError> {
    if arity <= 0xFF {
        // Small tuple
        if let Some(b) = buf.as_mut() {
            if *index + 2 > b.len() {
                return Err(EncodeError::BufferTooSmall);
            }
            b[*index] = ERL_SMALL_TUPLE_EXT;
            b[*index + 1] = arity as u8;
        }
        *index += 2;
    } else {
        // Large tuple
        if let Some(b) = buf.as_mut() {
            if *index + 5 > b.len() {
                return Err(EncodeError::BufferTooSmall);
            }
            b[*index] = ERL_LARGE_TUPLE_EXT;
            b[*index + 1..*index + 5].copy_from_slice(&(arity as u32).to_be_bytes());
        }
        *index += 5;
    }

    Ok(())
}

/// Encode a map header to EI format
///
/// # Arguments
/// * `buf` - Optional buffer to write to (None for size calculation)
/// * `index` - Current index in buffer
/// * `arity` - Number of key-value pairs in the map
///
/// # Returns
/// * `Ok(())` - Success
/// * `Err(EncodeError)` - Encoding error
pub fn encode_map_header(buf: &mut Option<&mut [u8]>, index: &mut usize, arity: usize) -> Result<(), EncodeError> {
    if let Some(b) = buf {
        if *index + 5 > b.len() {
            return Err(EncodeError::BufferTooSmall);
        }
        b[*index] = ERL_MAP_EXT;
        b[*index + 1..*index + 5].copy_from_slice(&(arity as u32).to_be_bytes());
    }
    *index += 5;

    Ok(())
}

/// Encode a list header to EI format
///
/// # Arguments
/// * `buf` - Optional buffer to write to (None for size calculation)
/// * `index` - Current index in buffer
/// * `length` - Number of elements in the list
///
/// # Returns
/// * `Ok(())` - Success
/// * `Err(EncodeError)` - Encoding error
pub fn encode_list_header(buf: &mut Option<&mut [u8]>, index: &mut usize, length: usize) -> Result<(), EncodeError> {
    if let Some(b) = buf {
        if *index + 5 > b.len() {
            return Err(EncodeError::BufferTooSmall);
        }
        b[*index] = ERL_LIST_EXT;
        b[*index + 1..*index + 5].copy_from_slice(&(length as u32).to_be_bytes());
    }
    *index += 5;

    Ok(())
}

/// Encoding errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodeError {
    /// Buffer is too small for the encoded value
    BufferTooSmall,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::{ERL_SMALL_TUPLE_EXT, ERL_LARGE_TUPLE_EXT, ERL_MAP_EXT, ERL_LIST_EXT};

    #[test]
    fn test_encode_tuple_header_small() {
        let mut buf = vec![0u8; 10];
        let mut index = 0;
        encode_tuple_header(&mut Some(&mut buf), &mut index, 3).unwrap();
        assert_eq!(index, 2);
        assert_eq!(buf[0], ERL_SMALL_TUPLE_EXT);
        assert_eq!(buf[1], 3);
    }

    #[test]
    fn test_encode_tuple_header_large() {
        let mut buf = vec![0u8; 10];
        let mut index = 0;
        encode_tuple_header(&mut Some(&mut buf), &mut index, 300).unwrap();
        assert_eq!(index, 5);
        assert_eq!(buf[0], ERL_LARGE_TUPLE_EXT);
        let arity = u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]);
        assert_eq!(arity, 300);
    }

    #[test]
    fn test_encode_map_header() {
        let mut buf = vec![0u8; 10];
        let mut index = 0;
        encode_map_header(&mut Some(&mut buf), &mut index, 5).unwrap();
        assert_eq!(index, 5);
        assert_eq!(buf[0], ERL_MAP_EXT);
    }

    #[test]
    fn test_encode_list_header() {
        let mut buf = vec![0u8; 10];
        let mut index = 0;
        encode_list_header(&mut Some(&mut buf), &mut index, 10).unwrap();
        assert_eq!(index, 5);
        assert_eq!(buf[0], ERL_LIST_EXT);
    }

    #[test]
    fn test_encode_tuple_header_small_buffer_too_small() {
        let mut buf = vec![0u8; 1]; // Too small for small tuple (needs 2 bytes)
        let mut index = 0;
        let result = encode_tuple_header(&mut Some(&mut buf), &mut index, 3);
        assert!(result.is_err());
        match result.unwrap_err() {
            EncodeError::BufferTooSmall => {}
        }
    }

    #[test]
    fn test_encode_tuple_header_large_buffer_too_small() {
        let mut buf = vec![0u8; 4]; // Too small for large tuple (needs 5 bytes)
        let mut index = 0;
        let result = encode_tuple_header(&mut Some(&mut buf), &mut index, 300);
        assert!(result.is_err());
        match result.unwrap_err() {
            EncodeError::BufferTooSmall => {}
        }
    }

    #[test]
    fn test_encode_map_header_buffer_too_small() {
        let mut buf = vec![0u8; 4]; // Too small for map header (needs 5 bytes)
        let mut index = 0;
        let result = encode_map_header(&mut Some(&mut buf), &mut index, 5);
        assert!(result.is_err());
        match result.unwrap_err() {
            EncodeError::BufferTooSmall => {}
        }
    }

    #[test]
    fn test_encode_list_header_buffer_too_small() {
        let mut buf = vec![0u8; 4]; // Too small for list header (needs 5 bytes)
        let mut index = 0;
        let result = encode_list_header(&mut Some(&mut buf), &mut index, 10);
        assert!(result.is_err());
        match result.unwrap_err() {
            EncodeError::BufferTooSmall => {}
        }
    }

    #[test]
    fn test_encode_tuple_header_size_calculation() {
        let mut index = 0;
        encode_tuple_header(&mut None, &mut index, 3).unwrap();
        assert_eq!(index, 2);
        
        let mut index2 = 0;
        encode_tuple_header(&mut None, &mut index2, 300).unwrap();
        assert_eq!(index2, 5);
    }

    #[test]
    fn test_encode_map_header_size_calculation() {
        let mut index = 0;
        encode_map_header(&mut None, &mut index, 5).unwrap();
        assert_eq!(index, 5);
    }

    #[test]
    fn test_encode_list_header_size_calculation() {
        let mut index = 0;
        encode_list_header(&mut None, &mut index, 10).unwrap();
        assert_eq!(index, 5);
    }

    #[test]
    fn test_encode_tuple_header_boundary() {
        // Test boundary value: exactly 0xFF (should use small format)
        let mut buf = vec![0u8; 10];
        let mut index = 0;
        encode_tuple_header(&mut Some(&mut buf), &mut index, 0xFF).unwrap();
        assert_eq!(buf[0], ERL_SMALL_TUPLE_EXT);
        assert_eq!(buf[1], 0xFF);
        
        // Test boundary value: 0xFF + 1 (should use large format)
        let mut buf2 = vec![0u8; 10];
        let mut index2 = 0;
        encode_tuple_header(&mut Some(&mut buf2), &mut index2, 0x100).unwrap();
        assert_eq!(buf2[0], ERL_LARGE_TUPLE_EXT);
    }

    #[test]
    fn test_encode_tuple_header_various_values() {
        let test_cases = vec![
            (0usize, ERL_SMALL_TUPLE_EXT),
            (1usize, ERL_SMALL_TUPLE_EXT),
            (100usize, ERL_SMALL_TUPLE_EXT),
            (0xFFusize, ERL_SMALL_TUPLE_EXT),
            (0x100usize, ERL_LARGE_TUPLE_EXT),
            (1000usize, ERL_LARGE_TUPLE_EXT),
            (usize::MAX, ERL_LARGE_TUPLE_EXT),
        ];
        
        for (arity, expected_tag) in test_cases {
            let mut buf = vec![0u8; 10];
            let mut index = 0;
            encode_tuple_header(&mut Some(&mut buf), &mut index, arity).unwrap();
            assert_eq!(buf[0], expected_tag);
        }
    }

    #[test]
    fn test_encode_map_header_various_values() {
        let test_cases = vec![0usize, 1usize, 100usize, 1000usize, usize::MAX];
        
        for arity in test_cases {
            let mut buf = vec![0u8; 10];
            let mut index = 0;
            encode_map_header(&mut Some(&mut buf), &mut index, arity).unwrap();
            assert_eq!(buf[0], ERL_MAP_EXT);
            let decoded_arity = u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]);
            assert_eq!(decoded_arity, arity as u32);
        }
    }

    #[test]
    fn test_encode_list_header_various_values() {
        let test_cases = vec![0usize, 1usize, 100usize, 1000usize, usize::MAX];
        
        for length in test_cases {
            let mut buf = vec![0u8; 10];
            let mut index = 0;
            encode_list_header(&mut Some(&mut buf), &mut index, length).unwrap();
            assert_eq!(buf[0], ERL_LIST_EXT);
            let decoded_length = u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]);
            assert_eq!(decoded_length, length as u32);
        }
    }

    #[test]
    fn test_encode_error_debug() {
        let error = EncodeError::BufferTooSmall;
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("BufferTooSmall"));
    }

    #[test]
    fn test_encode_error_clone() {
        let error = EncodeError::BufferTooSmall;
        let cloned = error.clone();
        assert_eq!(error, cloned);
    }

    #[test]
    fn test_encode_error_copy() {
        let error = EncodeError::BufferTooSmall;
        let copied = error; // Copy trait
        assert_eq!(error, copied);
    }

    #[test]
    fn test_encode_error_partial_eq() {
        let error1 = EncodeError::BufferTooSmall;
        let error2 = EncodeError::BufferTooSmall;
        assert_eq!(error1, error2);
    }

    #[test]
    fn test_encode_error_eq() {
        let error1 = EncodeError::BufferTooSmall;
        let error2 = EncodeError::BufferTooSmall;
        assert!(error1 == error2);
    }
}

