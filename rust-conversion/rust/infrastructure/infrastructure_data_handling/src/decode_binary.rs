//! Decode Binary Module
//!
//! Provides functionality to decode EI-encoded binaries.
//! Based on lib/erl_interface/src/decode/decode_binary.c

/// Decode a binary from EI-encoded bytes
///
/// # Arguments
/// * `buf` - Buffer containing EI-encoded data
/// * `index` - Starting index in the buffer
///
/// # Returns
/// * `Ok((binary_data, new_index))` - Decoded binary data and new index position
/// * `Err(DecodeBinaryError)` - Decoding error
pub fn decode_binary(buf: &[u8], index: usize) -> Result<(Vec<u8>, usize), DecodeBinaryError> {
    if index >= buf.len() {
        return Err(DecodeBinaryError::BufferTooShort);
    }

    let tag = buf[index];
    if tag != 109 {
        // BINARY_EXT = 109
        return Err(DecodeBinaryError::InvalidTag(tag));
    }

    decode_binary_internal(buf, index + 1)
}

/// Internal binary decoder (used by decode_term)
pub(crate) fn decode_binary_internal(
    buf: &[u8],
    pos: usize,
) -> Result<(Vec<u8>, usize), DecodeBinaryError> {
    // BINARY_EXT format: 4-byte length (big-endian) followed by data
    if pos + 4 > buf.len() {
        return Err(DecodeBinaryError::BufferTooShort);
    }

    let len = u32::from_be_bytes([buf[pos], buf[pos + 1], buf[pos + 2], buf[pos + 3]]) as usize;
    let data_pos = pos + 4;

    if data_pos + len > buf.len() {
        return Err(DecodeBinaryError::BufferTooShort);
    }

    let data = buf[data_pos..data_pos + len].to_vec();
    Ok((data, data_pos + len))
}

/// Binary decoding errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeBinaryError {
    /// Buffer too short
    BufferTooShort,
    /// Invalid tag
    InvalidTag(u8),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_binary() {
        // BINARY_EXT (109) + length 4 (4 bytes) + data
        let buf = vec![109, 0, 0, 0, 4, 1, 2, 3, 4];
        let result = decode_binary(&buf, 0);
        assert!(result.is_ok());
        let (data, pos) = result.unwrap();
        assert_eq!(data, vec![1u8, 2, 3, 4]);
        assert_eq!(pos, 9);
    }

    #[test]
    fn test_decode_binary_empty() {
        let buf = vec![109, 0, 0, 0, 0];
        let result = decode_binary(&buf, 0);
        assert!(result.is_ok());
        let (data, pos) = result.unwrap();
        assert_eq!(data, Vec::<u8>::new());
        assert_eq!(pos, 5);
    }

    #[test]
    fn test_decode_binary_buffer_too_short() {
        let buf = vec![109, 0, 0, 0, 10]; // Length but no data
        let result = decode_binary(&buf, 0);
        assert!(matches!(result, Err(DecodeBinaryError::BufferTooShort)));
    }

    #[test]
    fn test_decode_binary_invalid_tag() {
        let buf = vec![100, 0, 0, 0, 4, 1, 2, 3, 4]; // Wrong tag
        let result = decode_binary(&buf, 0);
        assert!(matches!(result, Err(DecodeBinaryError::InvalidTag(100))));
    }
}

