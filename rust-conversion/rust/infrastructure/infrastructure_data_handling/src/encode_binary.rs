//! Encode Binary Module
//!
//! Provides functionality to encode binaries to EI format.
//! Based on lib/erl_interface/src/encode/encode_binary.c

/// Encode a binary to EI format
///
/// # Arguments
/// * `buf` - Buffer to write encoded data to
/// * `data` - Binary data to encode
///
/// # Returns
/// * `Ok(bytes_written)` - Number of bytes written
/// * `Err(EncodeBinaryError)` - Encoding error
pub fn encode_binary(buf: &mut Vec<u8>, data: &[u8]) -> Result<usize, EncodeBinaryError> {
    let initial_len = buf.len();

    // BINARY_EXT tag (109)
    buf.push(109);

    // Write 4-byte length (big-endian)
    let len = data.len() as u32;
    buf.extend_from_slice(&len.to_be_bytes());

    // Write binary data
    buf.extend_from_slice(data);

    Ok(buf.len() - initial_len)
}

/// Binary encoding errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodeBinaryError {
    /// Encoding failed (placeholder for future error types)
    EncodingFailed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_binary() {
        let mut buf = Vec::new();
        let data = vec![1, 2, 3, 4];
        let result = encode_binary(&mut buf, &data);
        assert!(result.is_ok());
        assert_eq!(buf[0], 109); // BINARY_EXT
        assert_eq!(u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]), 4);
        assert_eq!(&buf[5..], &data);
    }

    #[test]
    fn test_encode_binary_empty() {
        let mut buf = Vec::new();
        let data = vec![];
        let result = encode_binary(&mut buf, &data);
        assert!(result.is_ok());
        assert_eq!(buf[0], 109); // BINARY_EXT
        assert_eq!(u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]), 0);
        assert_eq!(buf.len(), 5);
    }
}

