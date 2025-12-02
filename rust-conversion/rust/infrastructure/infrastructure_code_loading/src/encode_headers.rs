//! Header Encoding Module
//!
//! Provides functionality to encode tuple and list headers to EI format.
//! Based on lib/erl_interface/src/encode/encode_tuple_header.c

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
}

