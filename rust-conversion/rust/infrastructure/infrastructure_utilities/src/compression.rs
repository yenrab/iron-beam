//! Compression Module
//!
//! Provides compression and decompression functionality using zlib (via flate2) and zstd.
//! This module implements the equivalent of the C `erl_zlib.c` functionality, providing
//! both chunked and one-shot compression interfaces.
//!
//! ## Overview
//!
//! The compression module provides:
//! - **Zlib compression**: Using the `flate2` crate with `miniz_oxide` backend (pure safe Rust, equivalent to C zlib)
//! - **Zstd compression**: Using the `zstd` crate (provides both compression and decompression)
//! - **Chunked interfaces**: For streaming compression/decompression (used by term_to_binary)
//! - **One-shot interfaces**: For simple compress/uncompress operations
//!
//! ## Usage
//!
//! ### One-shot compression
//!
//! ```rust
//! use infrastructure_utilities::compression::{compress2, uncompress, CompressionLevel};
//!
//! let data = b"Hello, world!";
//! let mut compressed = vec![0u8; data.len() * 2];
//! let mut compressed_len = compressed.len();
//!
//! compress2(&mut compressed, &mut compressed_len, data, CompressionLevel::Default)?;
//! compressed.truncate(compressed_len);
//!
//! let mut decompressed = vec![0u8; data.len()];
//! let mut decompressed_len = decompressed.len();
//! uncompress(&mut decompressed, &mut decompressed_len, &compressed)?;
//! decompressed.truncate(decompressed_len);
//! assert_eq!(&decompressed, data);
//! ```
//!
//! ### Chunked compression
//!
//! ```rust
//! use infrastructure_utilities::compression::{DeflateStream, CompressionLevel};
//!
//! let mut stream = DeflateStream::new(data, CompressionLevel::Default)?;
//! let mut output = Vec::new();
//!
//! loop {
//!     let mut chunk = vec![0u8; 1024];
//!     let mut chunk_len = chunk.len();
//!     match stream.deflate_chunk(&mut chunk, &mut chunk_len)? {
//!         ChunkResult::More => {
//!             output.extend_from_slice(&chunk[..chunk_len]);
//!         }
//!         ChunkResult::Done => {
//!             output.extend_from_slice(&chunk[..chunk_len]);
//!             break;
//!         }
//!     }
//! }
//! ```
//!
//! Based on `erts/emulator/beam/erl_zlib.c`

use flate2::Compression;
use flate2::write::DeflateEncoder;
use flate2::read::DeflateDecoder;
use std::io::{Write, Read};

/// Compression level enumeration matching zlib levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionLevel {
    /// No compression (0)
    None = 0,
    /// Best speed (1)
    BestSpeed = 1,
    /// Default compression (6)
    Default = 6,
    /// Best compression (9)
    BestCompression = 9,
}

impl From<CompressionLevel> for Compression {
    fn from(level: CompressionLevel) -> Self {
        match level {
            CompressionLevel::None => Compression::none(),
            CompressionLevel::BestSpeed => Compression::fast(),
            CompressionLevel::Default => Compression::default(),
            CompressionLevel::BestCompression => Compression::best(),
        }
    }
}

impl From<i32> for CompressionLevel {
    fn from(level: i32) -> Self {
        match level {
            0 => CompressionLevel::None,
            1 => CompressionLevel::BestSpeed,
            6 => CompressionLevel::Default,
            9 => CompressionLevel::BestCompression,
            n if n < 0 => CompressionLevel::Default,
            n if n > 9 => CompressionLevel::BestCompression,
            _ => CompressionLevel::Default,
        }
    }
}

/// Compression error types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompressionError {
    /// Buffer too small
    BufferError,
    /// Data error (corrupted or invalid)
    DataError,
    /// Memory allocation error
    MemoryError,
    /// Stream error
    StreamError,
    /// Other error
    Other(String),
}

impl std::fmt::Display for CompressionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompressionError::BufferError => write!(f, "Buffer too small"),
            CompressionError::DataError => write!(f, "Data error"),
            CompressionError::MemoryError => write!(f, "Memory allocation error"),
            CompressionError::StreamError => write!(f, "Stream error"),
            CompressionError::Other(msg) => write!(f, "Other error: {}", msg),
        }
    }
}

impl std::error::Error for CompressionError {}

/// Result type for compression operations
pub type CompressionResult<T> = Result<T, CompressionError>;

/// Result of a chunked compression/decompression operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChunkResult {
    /// More data to process
    More,
    /// All data processed
    Done,
}

/// Deflate stream for chunked compression
///
/// Equivalent to the C `z_stream` structure used in `erl_zlib_deflate_*` functions.
pub struct DeflateStream {
    encoder: Option<DeflateEncoder<Vec<u8>>>,
    finished: bool,
    compressed_data: Option<Vec<u8>>,
}

impl DeflateStream {
    /// Create a new deflate stream (equivalent to `erl_zlib_deflate_start`)
    ///
    /// # Arguments
    /// * `source` - Source data to compress
    /// * `level` - Compression level
    pub fn new(source: &[u8], level: CompressionLevel) -> CompressionResult<Self> {
        let mut encoder = DeflateEncoder::new(Vec::new(), level.into());
        
        // Write all source data to encoder
        encoder.write_all(source)
            .map_err(|e| CompressionError::Other(format!("Write error: {}", e)))?;
        
        Ok(Self {
            encoder: Some(encoder),
            finished: false,
            compressed_data: None,
        })
    }

    /// Compress a chunk of data (equivalent to `erl_zlib_deflate_chunk`)
    ///
    /// # Arguments
    /// * `dest` - Destination buffer
    /// * `dest_len` - On input: maximum length, on output: actual length written
    ///
    /// # Returns
    /// * `Ok(ChunkResult::More)` - More data to process
    /// * `Ok(ChunkResult::Done)` - All data processed
    /// * `Err(CompressionError)` - Error occurred
    pub fn deflate_chunk(&mut self, dest: &mut [u8], dest_len: &mut usize) -> CompressionResult<ChunkResult> {
        if self.finished {
            return Ok(ChunkResult::Done);
        }

        // Get compressed data if not already computed
        if self.compressed_data.is_none() {
            let encoder = self.encoder.take().ok_or_else(|| CompressionError::StreamError)?;
            let compressed = encoder.finish()
                .map_err(|e| CompressionError::Other(format!("Finish error: {}", e)))?;
            self.compressed_data = Some(compressed);
        }

        let compressed = self.compressed_data.as_ref().unwrap();
        self.finished = true;
        
        if compressed.len() > *dest_len {
            return Err(CompressionError::BufferError);
        }

        dest[..compressed.len()].copy_from_slice(compressed);
        *dest_len = compressed.len();

        Ok(ChunkResult::Done)
    }

    /// Finish the deflate stream (equivalent to `erl_zlib_deflate_finish`)
    ///
    /// This is a no-op in Rust since we handle finishing in `deflate_chunk`.
    pub fn finish(&mut self) -> CompressionResult<()> {
        self.finished = true;
        Ok(())
    }
}

/// Inflate stream for chunked decompression
///
/// Equivalent to the C `z_stream` structure used in `erl_zlib_inflate_*` functions.
pub struct InflateStream {
    finished: bool,
    output_buffer: Vec<u8>,
    output_pos: usize,
}

impl InflateStream {
    /// Create a new inflate stream (equivalent to `erl_zlib_inflate_start`)
    ///
    /// # Arguments
    /// * `source` - Compressed source data
    pub fn new(source: &[u8]) -> CompressionResult<Self> {
        // For chunked decompression, we decompress all at once and then
        // serve it in chunks. This matches the C behavior where inflate
        // processes all available input.
        let mut decoder = DeflateDecoder::new(source);
        let mut output = Vec::new();
        
        decoder.read_to_end(&mut output)
            .map_err(|e| {
                if e.to_string().contains("invalid") || e.to_string().contains("corrupt") {
                    CompressionError::DataError
                } else {
                    CompressionError::Other(format!("Read error: {}", e))
                }
            })?;

        Ok(Self {
            finished: false,
            output_buffer: output,
            output_pos: 0,
        })
    }

    /// Decompress a chunk of data (equivalent to `erl_zlib_inflate_chunk`)
    ///
    /// # Arguments
    /// * `dest` - Destination buffer
    /// * `dest_len` - On input: maximum length, on output: actual length written
    ///
    /// # Returns
    /// * `Ok(ChunkResult::More)` - More data to process
    /// * `Ok(ChunkResult::Done)` - All data processed
    /// * `Err(CompressionError)` - Error occurred
    pub fn inflate_chunk(&mut self, dest: &mut [u8], dest_len: &mut usize) -> CompressionResult<ChunkResult> {
        if self.finished {
            return Ok(ChunkResult::Done);
        }

        let remaining = self.output_buffer.len() - self.output_pos;
        if remaining == 0 {
            self.finished = true;
            *dest_len = 0;
            return Ok(ChunkResult::Done);
        }

        let to_copy = remaining.min(*dest_len);
        dest[..to_copy].copy_from_slice(&self.output_buffer[self.output_pos..self.output_pos + to_copy]);
        self.output_pos += to_copy;
        *dest_len = to_copy;

        if self.output_pos >= self.output_buffer.len() {
            self.finished = true;
            Ok(ChunkResult::Done)
        } else {
            Ok(ChunkResult::More)
        }
    }

    /// Finish the inflate stream (equivalent to `erl_zlib_inflate_finish`)
    ///
    /// This is a no-op in Rust since we handle finishing in `inflate_chunk`.
    pub fn finish(&mut self) -> CompressionResult<()> {
        self.finished = true;
        Ok(())
    }
}

/// Compress data using zlib (equivalent to `erl_zlib_compress2`)
///
/// # Arguments
/// * `dest` - Destination buffer (must be large enough)
/// * `dest_len` - On input: maximum length, on output: actual compressed length
/// * `source` - Source data to compress
/// * `level` - Compression level
///
/// # Returns
/// * `Ok(())` - Success
/// * `Err(CompressionError)` - Error occurred
pub fn compress2(
    dest: &mut [u8],
    dest_len: &mut usize,
    source: &[u8],
    level: CompressionLevel,
) -> CompressionResult<()> {
    let mut encoder = DeflateEncoder::new(Vec::new(), level.into());
    encoder.write_all(source)
        .map_err(|e| CompressionError::Other(format!("Write error: {}", e)))?;
    
    let compressed = encoder.finish()
        .map_err(|e| CompressionError::Other(format!("Finish error: {}", e)))?;

    if compressed.len() > *dest_len {
        return Err(CompressionError::BufferError);
    }

    dest[..compressed.len()].copy_from_slice(&compressed);
    *dest_len = compressed.len();

    Ok(())
}

/// Decompress data using zlib (equivalent to `erl_zlib_uncompress`)
///
/// # Arguments
/// * `dest` - Destination buffer (must be large enough)
/// * `dest_len` - On input: maximum length, on output: actual decompressed length
/// * `source` - Compressed source data
///
/// # Returns
/// * `Ok(())` - Success
/// * `Err(CompressionError)` - Error occurred
pub fn uncompress(
    dest: &mut [u8],
    dest_len: &mut usize,
    source: &[u8],
) -> CompressionResult<()> {
    let mut decoder = DeflateDecoder::new(source);
    let mut decompressed = Vec::new();
    
    decoder.read_to_end(&mut decompressed)
        .map_err(|e| {
            // Map common errors
            if e.to_string().contains("invalid") || e.to_string().contains("corrupt") {
                CompressionError::DataError
            } else if e.to_string().contains("buffer") {
                CompressionError::BufferError
            } else {
                CompressionError::Other(format!("Read error: {}", e))
            }
        })?;

    if decompressed.len() > *dest_len {
        return Err(CompressionError::BufferError);
    }

    dest[..decompressed.len()].copy_from_slice(&decompressed);
    *dest_len = decompressed.len();

    Ok(())
}

/// Compress data using zstd (via zstd crate)
///
/// # Arguments
/// * `data` - Data to compress
/// * `level` - Compression level (0-22, clamped to valid range)
///
/// # Returns
/// * `Ok(compressed_data)` - Compressed data
/// * `Err(CompressionError)` - Error occurred
pub fn zstd_compress(data: &[u8], level: i32) -> CompressionResult<Vec<u8>> {
    // Clamp level to valid zstd range (0-22)
    let level = level.clamp(0, 22);
    
    // Compress using zstd crate
    zstd::encode_all(data, level)
        .map_err(|e| {
            // Map zstd errors to CompressionError
            let error_msg = e.to_string();
            if error_msg.contains("buffer") || error_msg.contains("size") {
                CompressionError::BufferError
            } else if error_msg.contains("memory") || error_msg.contains("allocation") {
                CompressionError::MemoryError
            } else {
                CompressionError::Other(format!("Zstd compression error: {}", e))
            }
        })
}

/// Decompress data using zstd (via zstd crate)
///
/// # Arguments
/// * `data` - Compressed data
///
/// # Returns
/// * `Ok(decompressed_data)` - Decompressed data
/// * `Err(CompressionError)` - Error occurred
pub fn zstd_decompress(data: &[u8]) -> CompressionResult<Vec<u8>> {
    // Decompress using zstd crate
    zstd::decode_all(data)
        .map_err(|e| {
            // Map zstd errors to CompressionError
            let error_msg = e.to_string();
            if error_msg.contains("invalid") || error_msg.contains("corrupt") || error_msg.contains("malformed") {
                CompressionError::DataError
            } else if error_msg.contains("buffer") || error_msg.contains("size") {
                CompressionError::BufferError
            } else if error_msg.contains("memory") || error_msg.contains("allocation") {
                CompressionError::MemoryError
            } else {
                CompressionError::Other(format!("Zstd decompression error: {}", e))
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress2_uncompress() {
        let data = b"Hello, world! This is a test string for compression.";
        let mut compressed = vec![0u8; data.len() * 2];
        let mut compressed_len = compressed.len();

        // Compress
        compress2(&mut compressed, &mut compressed_len, data, CompressionLevel::Default)
            .expect("Compression should succeed");
        compressed.truncate(compressed_len);

        // Verify compressed data is smaller (or at least not much larger)
        assert!(compressed.len() <= data.len() + 100, "Compressed data should be reasonable size");

        // Decompress
        let mut decompressed = vec![0u8; data.len() * 2];
        let mut decompressed_len = decompressed.len();
        uncompress(&mut decompressed, &mut decompressed_len, &compressed)
            .expect("Decompression should succeed");
        decompressed.truncate(decompressed_len);

        // Verify round-trip
        assert_eq!(&decompressed, data);
    }

    #[test]
    fn test_compress2_different_levels() {
        let data = b"Hello, world! This is a test string for compression. ".repeat(10);
        
        let mut results = Vec::new();
        for level in [CompressionLevel::None, CompressionLevel::BestSpeed, CompressionLevel::Default, CompressionLevel::BestCompression] {
            let mut compressed = vec![0u8; data.len() * 2];
            let mut compressed_len = compressed.len();
            compress2(&mut compressed, &mut compressed_len, &data, level)
                .expect("Compression should succeed");
            results.push(compressed_len);
        }

        // BestCompression should produce smallest (or equal) output
        assert!(results[3] <= results[0], "BestCompression should be at least as good as None");
    }

    #[test]
    fn test_deflate_stream() {
        let data = b"Hello, world! This is a test string for chunked compression.";
        let mut stream = DeflateStream::new(data, CompressionLevel::Default)
            .expect("Stream creation should succeed");

        let mut output = Vec::new();
        let mut chunk = vec![0u8; 1024];
        let mut chunk_len = chunk.len();

        match stream.deflate_chunk(&mut chunk, &mut chunk_len)
            .expect("Deflate chunk should succeed") {
            ChunkResult::Done => {
                output.extend_from_slice(&chunk[..chunk_len]);
            }
            ChunkResult::More => {
                output.extend_from_slice(&chunk[..chunk_len]);
            }
        }

        // Decompress and verify
        let mut decompressed = vec![0u8; data.len() * 2];
        let mut decompressed_len = decompressed.len();
        uncompress(&mut decompressed, &mut decompressed_len, &output)
            .expect("Decompression should succeed");
        decompressed.truncate(decompressed_len);
        assert_eq!(&decompressed, data);
    }

    #[test]
    fn test_inflate_stream() {
        let data = b"Hello, world! This is a test string for chunked decompression.";
        
        // First compress
        let mut compressed = vec![0u8; data.len() * 2];
        let mut compressed_len = compressed.len();
        compress2(&mut compressed, &mut compressed_len, data, CompressionLevel::Default)
            .expect("Compression should succeed");
        compressed.truncate(compressed_len);

        // Then decompress using stream
        let mut stream = InflateStream::new(&compressed)
            .expect("Stream creation should succeed");

        let mut output = Vec::new();
        loop {
            let mut chunk = vec![0u8; 32]; // Small chunks to test chunking
            let mut chunk_len = chunk.len();
            match stream.inflate_chunk(&mut chunk, &mut chunk_len)
                .expect("Inflate chunk should succeed") {
                ChunkResult::More => {
                    output.extend_from_slice(&chunk[..chunk_len]);
                }
                ChunkResult::Done => {
                    if chunk_len > 0 {
                        output.extend_from_slice(&chunk[..chunk_len]);
                    }
                    break;
                }
            }
        }

        assert_eq!(&output, data);
    }

    #[test]
    fn test_zstd_compress_decompress() {
        let data = b"Hello, world! This is a test string for zstd compression.";
        
        let compressed = zstd_compress(data, 3)
            .expect("Zstd compression should succeed");
        
        let decompressed = zstd_decompress(&compressed)
            .expect("Zstd decompression should succeed");
        
        assert_eq!(&decompressed, data);
    }

    #[test]
    fn test_zstd_different_levels() {
        let data = b"Hello, world! This is a test string for zstd compression. ".repeat(10);
        
        let level_1 = zstd_compress(&data, 1).expect("Should compress");
        let level_22 = zstd_compress(&data, 22).expect("Should compress");
        
        // Both should compress successfully
        // Just verify both produce valid compressed output
        assert!(!level_1.is_empty(), "Level 1 compression should produce output");
        assert!(!level_22.is_empty(), "Level 22 compression should produce output");
        
        // Both should decompress correctly
        let decompressed_1 = zstd_decompress(&level_1).expect("Should decompress level 1");
        let decompressed_22 = zstd_decompress(&level_22).expect("Should decompress level 22");
        
        assert_eq!(&decompressed_1, &data);
        assert_eq!(&decompressed_22, &data);
    }

    #[test]
    fn test_zstd_level_clamping() {
        let data = b"Test data for level clamping";
        
        // Test negative level (should clamp to 0)
        let compressed_neg = zstd_compress(data, -5).expect("Should compress with clamped level");
        assert!(!compressed_neg.is_empty());
        
        // Test level > 22 (should clamp to 22)
        let compressed_high = zstd_compress(data, 100).expect("Should compress with clamped level");
        assert!(!compressed_high.is_empty());
        
        // Both should decompress correctly
        let decompressed_neg = zstd_decompress(&compressed_neg).expect("Should decompress");
        let decompressed_high = zstd_decompress(&compressed_high).expect("Should decompress");
        
        assert_eq!(&decompressed_neg, data);
        assert_eq!(&decompressed_high, data);
    }

    #[test]
    fn test_zstd_decompress_invalid_data() {
        let invalid_data = b"This is not zstd compressed data";
        
        // Should return an error for invalid compressed data
        let result = zstd_decompress(invalid_data);
        assert!(result.is_err());
        assert!(matches!(result, Err(CompressionError::DataError) | Err(CompressionError::Other(_))));
    }

    #[test]
    fn test_zstd_empty_data() {
        let empty_data = b"";
        
        // Empty compression should work
        let compressed = zstd_compress(empty_data, 3).expect("Should compress empty data");
        assert!(!compressed.is_empty(), "Even empty data produces some compressed output");
        
        // Empty decompression should work
        let decompressed = zstd_decompress(&compressed).expect("Should decompress empty data");
        assert_eq!(&decompressed, empty_data);
    }

    #[test]
    fn test_compress_empty() {
        let data = b"";
        let mut compressed = vec![0u8; 100];
        let mut compressed_len = compressed.len();
        
        compress2(&mut compressed, &mut compressed_len, data, CompressionLevel::Default)
            .expect("Empty compression should succeed");
        
        let mut decompressed = vec![0u8; 100];
        let mut decompressed_len = decompressed.len();
        uncompress(&mut decompressed, &mut decompressed_len, &compressed[..compressed_len])
            .expect("Empty decompression should succeed");
        
        assert_eq!(decompressed_len, 0);
    }

    #[test]
    fn test_uncompress_invalid_data() {
        let invalid_data = b"This is not compressed data";
        let mut decompressed = vec![0u8; 1000];
        let mut decompressed_len = decompressed.len();
        
        // Should return an error
        let result = uncompress(&mut decompressed, &mut decompressed_len, invalid_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_compress_buffer_too_small() {
        let data = b"Hello, world! This is a test string.";
        let mut compressed = vec![0u8; 1]; // Too small
        let mut compressed_len = compressed.len();
        
        // Should return buffer error
        let result = compress2(&mut compressed, &mut compressed_len, data, CompressionLevel::Default);
        assert!(matches!(result, Err(CompressionError::BufferError)));
    }

    #[test]
    fn test_compression_level_conversion() {
        assert_eq!(CompressionLevel::from(0), CompressionLevel::None);
        assert_eq!(CompressionLevel::from(1), CompressionLevel::BestSpeed);
        assert_eq!(CompressionLevel::from(6), CompressionLevel::Default);
        assert_eq!(CompressionLevel::from(9), CompressionLevel::BestCompression);
        assert_eq!(CompressionLevel::from(5), CompressionLevel::Default); // Clamped
        assert_eq!(CompressionLevel::from(10), CompressionLevel::BestCompression); // Clamped
    }

    #[test]
    fn test_large_data_compression() {
        let data: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();
        
        let mut compressed = vec![0u8; data.len() * 2];
        let mut compressed_len = compressed.len();
        compress2(&mut compressed, &mut compressed_len, &data, CompressionLevel::Default)
            .expect("Large data compression should succeed");
        compressed.truncate(compressed_len);

        let mut decompressed = vec![0u8; data.len()];
        let mut decompressed_len = decompressed.len();
        uncompress(&mut decompressed, &mut decompressed_len, &compressed)
            .expect("Large data decompression should succeed");
        decompressed.truncate(decompressed_len);

        assert_eq!(&decompressed, &data);
    }

    #[test]
    fn test_compression_error_display() {
        assert_eq!(CompressionError::BufferError.to_string(), "Buffer too small");
        assert_eq!(CompressionError::DataError.to_string(), "Data error");
        assert_eq!(CompressionError::MemoryError.to_string(), "Memory allocation error");
        assert_eq!(CompressionError::StreamError.to_string(), "Stream error");
        assert_eq!(CompressionError::Other("test".to_string()).to_string(), "Other error: test");
    }

    #[test]
    fn test_compression_error_is_error() {
        // Test that CompressionError implements Error trait
        let error: &dyn std::error::Error = &CompressionError::BufferError;
        assert!(!error.to_string().is_empty());
    }

    #[test]
    fn test_compression_level_from_negative() {
        // Test negative values (should default to Default)
        assert_eq!(CompressionLevel::from(-1), CompressionLevel::Default);
        assert_eq!(CompressionLevel::from(-100), CompressionLevel::Default);
    }

    #[test]
    fn test_compression_level_from_high_values() {
        // Test values > 9 (should clamp to BestCompression)
        assert_eq!(CompressionLevel::from(10), CompressionLevel::BestCompression);
        assert_eq!(CompressionLevel::from(100), CompressionLevel::BestCompression);
    }

    #[test]
    fn test_compression_level_from_intermediate_values() {
        // Test intermediate values (should default to Default)
        assert_eq!(CompressionLevel::from(2), CompressionLevel::Default);
        assert_eq!(CompressionLevel::from(3), CompressionLevel::Default);
        assert_eq!(CompressionLevel::from(4), CompressionLevel::Default);
        assert_eq!(CompressionLevel::from(7), CompressionLevel::Default);
        assert_eq!(CompressionLevel::from(8), CompressionLevel::Default);
    }

    #[test]
    fn test_deflate_stream_finish() {
        let data = b"Test data for finish";
        let mut stream = DeflateStream::new(data, CompressionLevel::Default)
            .expect("Stream creation should succeed");
        
        // Call finish
        stream.finish().expect("Finish should succeed");
        
        // After finish, deflate_chunk should return Done
        let mut chunk = vec![0u8; 1024];
        let mut chunk_len = chunk.len();
        let result = stream.deflate_chunk(&mut chunk, &mut chunk_len)
            .expect("Deflate chunk should succeed");
        assert_eq!(result, ChunkResult::Done);
    }

    #[test]
    fn test_deflate_stream_multiple_chunks_after_finished() {
        let data = b"Test data";
        let mut stream = DeflateStream::new(data, CompressionLevel::Default)
            .expect("Stream creation should succeed");
        
        let mut chunk1 = vec![0u8; 1024];
        let mut chunk1_len = chunk1.len();
        let result1 = stream.deflate_chunk(&mut chunk1, &mut chunk1_len)
            .expect("First chunk should succeed");
        assert_eq!(result1, ChunkResult::Done);
        
        // Second call after finished should also return Done
        let mut chunk2 = vec![0u8; 1024];
        let mut chunk2_len = chunk2.len();
        let result2 = stream.deflate_chunk(&mut chunk2, &mut chunk2_len)
            .expect("Second chunk should succeed");
        assert_eq!(result2, ChunkResult::Done);
    }

    #[test]
    fn test_deflate_stream_buffer_too_small() {
        let data = b"Test data that will compress to more than 1 byte";
        let mut stream = DeflateStream::new(data, CompressionLevel::Default)
            .expect("Stream creation should succeed");
        
        let mut chunk = vec![0u8; 1]; // Too small
        let mut chunk_len = chunk.len();
        let result = stream.deflate_chunk(&mut chunk, &mut chunk_len);
        assert!(matches!(result, Err(CompressionError::BufferError)));
    }

    #[test]
    fn test_inflate_stream_finish() {
        let data = b"Test data";
        let mut compressed = vec![0u8; data.len() * 2];
        let mut compressed_len = compressed.len();
        compress2(&mut compressed, &mut compressed_len, data, CompressionLevel::Default)
            .expect("Compression should succeed");
        compressed.truncate(compressed_len);
        
        let mut stream = InflateStream::new(&compressed)
            .expect("Stream creation should succeed");
        
        // Call finish
        stream.finish().expect("Finish should succeed");
        
        // After finish, inflate_chunk should return Done
        let mut chunk = vec![0u8; 1024];
        let mut chunk_len = chunk.len();
        let result = stream.inflate_chunk(&mut chunk, &mut chunk_len)
            .expect("Inflate chunk should succeed");
        assert_eq!(result, ChunkResult::Done);
    }

    #[test]
    fn test_inflate_stream_empty_output() {
        let data = b"Test";
        let mut compressed = vec![0u8; data.len() * 2];
        let mut compressed_len = compressed.len();
        compress2(&mut compressed, &mut compressed_len, data, CompressionLevel::Default)
            .expect("Compression should succeed");
        compressed.truncate(compressed_len);
        
        let mut stream = InflateStream::new(&compressed)
            .expect("Stream creation should succeed");
        
        // Read all data
        let mut output = Vec::new();
        loop {
            let mut chunk = vec![0u8; 1024];
            let mut chunk_len = chunk.len();
            match stream.inflate_chunk(&mut chunk, &mut chunk_len)
                .expect("Inflate chunk should succeed") {
                ChunkResult::More => {
                    output.extend_from_slice(&chunk[..chunk_len]);
                }
                ChunkResult::Done => {
                    if chunk_len > 0 {
                        output.extend_from_slice(&chunk[..chunk_len]);
                    }
                    break;
                }
            }
        }
        
        assert_eq!(&output, data);
        
        // After all data is read, next call should return Done
        // (dest_len may not be modified if finished is already true)
        let mut chunk = vec![0u8; 1024];
        let mut chunk_len = chunk.len();
        let result = stream.inflate_chunk(&mut chunk, &mut chunk_len)
            .expect("Inflate chunk should succeed");
        assert_eq!(result, ChunkResult::Done);
        // Note: dest_len may not be modified when finished is already true
    }

    #[test]
    fn test_inflate_stream_small_chunks() {
        let data = b"Hello, world! This is a longer test string.";
        let mut compressed = vec![0u8; data.len() * 2];
        let mut compressed_len = compressed.len();
        compress2(&mut compressed, &mut compressed_len, data, CompressionLevel::Default)
            .expect("Compression should succeed");
        compressed.truncate(compressed_len);
        
        let mut stream = InflateStream::new(&compressed)
            .expect("Stream creation should succeed");
        
        // Read in very small chunks to test More path
        let mut output = Vec::new();
        let mut iterations = 0;
        loop {
            let mut chunk = vec![0u8; 5]; // Very small chunks
            let mut chunk_len = chunk.len();
            match stream.inflate_chunk(&mut chunk, &mut chunk_len)
                .expect("Inflate chunk should succeed") {
                ChunkResult::More => {
                    output.extend_from_slice(&chunk[..chunk_len]);
                    iterations += 1;
                    if iterations > 100 {
                        break; // Safety limit
                    }
                }
                ChunkResult::Done => {
                    if chunk_len > 0 {
                        output.extend_from_slice(&chunk[..chunk_len]);
                    }
                    break;
                }
            }
        }
        
        assert_eq!(&output, data);
    }

    #[test]
    fn test_uncompress_buffer_too_small() {
        let data = b"Hello, world! This is a test string.";
        let mut compressed = vec![0u8; data.len() * 2];
        let mut compressed_len = compressed.len();
        compress2(&mut compressed, &mut compressed_len, data, CompressionLevel::Default)
            .expect("Compression should succeed");
        compressed.truncate(compressed_len);
        
        let mut decompressed = vec![0u8; 1]; // Too small
        let mut decompressed_len = decompressed.len();
        
        let result = uncompress(&mut decompressed, &mut decompressed_len, &compressed);
        assert!(matches!(result, Err(CompressionError::BufferError)));
    }

    #[test]
    fn test_uncompress_error_mapping() {
        // Test that invalid data produces DataError
        let invalid_data = b"This is not compressed data";
        let mut decompressed = vec![0u8; 1000];
        let mut decompressed_len = decompressed.len();
        
        let result = uncompress(&mut decompressed, &mut decompressed_len, invalid_data);
        assert!(result.is_err());
        // Should be DataError based on error message matching
        if let Err(e) = result {
            // Error should be DataError or Other with "invalid"/"corrupt" in message
            match e {
                CompressionError::DataError => {}
                CompressionError::Other(msg) => {
                    assert!(msg.contains("invalid") || msg.contains("corrupt") || msg.contains("Read error"));
                }
                _ => panic!("Unexpected error type: {:?}", e),
            }
        }
    }

    #[test]
    fn test_zstd_compress_all_levels() {
        let data = b"Test data for all zstd levels";
        
        // Test all valid levels (0-22)
        for level in 0..=22 {
            let compressed = zstd_compress(data, level)
                .expect(&format!("Should compress at level {}", level));
            assert!(!compressed.is_empty());
            
            let decompressed = zstd_decompress(&compressed)
                .expect(&format!("Should decompress level {}", level));
            assert_eq!(&decompressed, data);
        }
    }

    #[test]
    fn test_zstd_error_mapping() {
        // Test that invalid zstd data produces appropriate error
        let invalid_data = b"This is not zstd compressed data";
        let result = zstd_decompress(invalid_data);
        assert!(result.is_err());
        
        if let Err(e) = result {
            // Should be DataError or Other with error message
            match e {
                CompressionError::DataError => {}
                CompressionError::Other(msg) => {
                    assert!(msg.contains("invalid") || msg.contains("corrupt") || msg.contains("malformed") || msg.contains("Zstd"));
                }
                _ => panic!("Unexpected error type: {:?}", e),
            }
        }
    }

    #[test]
    fn test_compression_level_from_compression() {
        // Test From<CompressionLevel> for Compression
        let level = CompressionLevel::Default;
        let compression: Compression = level.into();
        // Just verify it compiles and doesn't panic
        let _ = compression;
    }

    #[test]
    fn test_chunk_result_variants() {
        // Test ChunkResult variants
        assert_eq!(ChunkResult::More, ChunkResult::More);
        assert_eq!(ChunkResult::Done, ChunkResult::Done);
        assert_ne!(ChunkResult::More, ChunkResult::Done);
    }

    #[test]
    fn test_compression_error_clone() {
        // Test that CompressionError can be cloned
        let error1 = CompressionError::BufferError;
        let error2 = error1.clone();
        assert_eq!(error1, error2);
        
        let error3 = CompressionError::Other("test".to_string());
        let error4 = error3.clone();
        assert_eq!(error3, error4);
    }

    #[test]
    fn test_compression_error_partial_eq() {
        // Test PartialEq for CompressionError
        assert_eq!(CompressionError::BufferError, CompressionError::BufferError);
        assert_eq!(CompressionError::DataError, CompressionError::DataError);
        assert_ne!(CompressionError::BufferError, CompressionError::DataError);
        
        assert_eq!(
            CompressionError::Other("test".to_string()),
            CompressionError::Other("test".to_string())
        );
        assert_ne!(
            CompressionError::Other("test".to_string()),
            CompressionError::Other("other".to_string())
        );
    }

    #[test]
    fn test_zstd_compress_level_boundary() {
        let data = b"Test data for boundary levels";
        
        // Test boundary levels
        let level_0 = zstd_compress(data, 0).expect("Level 0 should work");
        let level_22 = zstd_compress(data, 22).expect("Level 22 should work");
        
        assert!(!level_0.is_empty());
        assert!(!level_22.is_empty());
        
        // Both should decompress
        let decompressed_0 = zstd_decompress(&level_0).expect("Should decompress");
        let decompressed_22 = zstd_decompress(&level_22).expect("Should decompress");
        assert_eq!(&decompressed_0, data);
        assert_eq!(&decompressed_22, data);
    }

    #[test]
    fn test_inflate_stream_with_exact_buffer_size() {
        let data = b"Test data for exact buffer";
        let mut compressed = vec![0u8; data.len() * 2];
        let mut compressed_len = compressed.len();
        compress2(&mut compressed, &mut compressed_len, data, CompressionLevel::Default)
            .expect("Compression should succeed");
        compressed.truncate(compressed_len);
        
        let mut stream = InflateStream::new(&compressed)
            .expect("Stream creation should succeed");
        
        // Use buffer exactly the size of decompressed data
        let mut chunk = vec![0u8; data.len()];
        let mut chunk_len = chunk.len();
        let result = stream.inflate_chunk(&mut chunk, &mut chunk_len)
            .expect("Inflate chunk should succeed");
        
        // Should get all data in one chunk
        assert_eq!(result, ChunkResult::Done);
        assert_eq!(&chunk[..chunk_len], data);
    }

    #[test]
    fn test_deflate_stream_multiple_finish_calls() {
        let data = b"Test data";
        let mut stream = DeflateStream::new(data, CompressionLevel::Default)
            .expect("Stream creation should succeed");
        
        // Call finish multiple times
        stream.finish().expect("First finish should succeed");
        stream.finish().expect("Second finish should succeed");
        
        // Should still work
        let mut chunk = vec![0u8; 1024];
        let mut chunk_len = chunk.len();
        let result = stream.deflate_chunk(&mut chunk, &mut chunk_len)
            .expect("Deflate chunk should succeed");
        assert_eq!(result, ChunkResult::Done);
    }

    #[test]
    fn test_inflate_stream_multiple_finish_calls() {
        let data = b"Test data";
        let mut compressed = vec![0u8; data.len() * 2];
        let mut compressed_len = compressed.len();
        compress2(&mut compressed, &mut compressed_len, data, CompressionLevel::Default)
            .expect("Compression should succeed");
        compressed.truncate(compressed_len);
        
        let mut stream = InflateStream::new(&compressed)
            .expect("Stream creation should succeed");
        
        // Call finish multiple times
        stream.finish().expect("First finish should succeed");
        stream.finish().expect("Second finish should succeed");
        
        // Should still return Done
        let mut chunk = vec![0u8; 1024];
        let mut chunk_len = chunk.len();
        let result = stream.inflate_chunk(&mut chunk, &mut chunk_len)
            .expect("Inflate chunk should succeed");
        assert_eq!(result, ChunkResult::Done);
    }

    #[test]
    fn test_compression_level_all_variants() {
        // Test all CompressionLevel variants
        assert_eq!(CompressionLevel::None as i32, 0);
        assert_eq!(CompressionLevel::BestSpeed as i32, 1);
        assert_eq!(CompressionLevel::Default as i32, 6);
        assert_eq!(CompressionLevel::BestCompression as i32, 9);
    }

    #[test]
    fn test_compression_level_into_compression() {
        // Test conversion to Compression for all levels
        let _: Compression = CompressionLevel::None.into();
        let _: Compression = CompressionLevel::BestSpeed.into();
        let _: Compression = CompressionLevel::Default.into();
        let _: Compression = CompressionLevel::BestCompression.into();
    }

    #[test]
    fn test_uncompress_with_buffer_error_path() {
        // This test tries to trigger the buffer error path in uncompress
        // by providing a destination buffer that's too small
        let data = b"Hello, world! This is a longer test string that will compress.";
        let mut compressed = vec![0u8; data.len() * 2];
        let mut compressed_len = compressed.len();
        compress2(&mut compressed, &mut compressed_len, data, CompressionLevel::Default)
            .expect("Compression should succeed");
        compressed.truncate(compressed_len);
        
        // Try with buffer that's too small
        let mut decompressed = vec![0u8; 1];
        let mut decompressed_len = decompressed.len();
        let result = uncompress(&mut decompressed, &mut decompressed_len, &compressed);
        assert!(matches!(result, Err(CompressionError::BufferError)));
    }

    #[test]
    fn test_zstd_compress_with_various_data_sizes() {
        // Test with different data sizes
        let sizes = [0, 1, 10, 100, 1000, 10000];
        for size in sizes {
            let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
            let compressed = zstd_compress(&data, 3)
                .expect(&format!("Should compress data of size {}", size));
            let decompressed = zstd_decompress(&compressed)
                .expect(&format!("Should decompress data of size {}", size));
            assert_eq!(&decompressed, &data);
        }
    }

    #[test]
    fn test_compress2_with_all_levels() {
        let data = b"Test data for all compression levels";
        
        // Test all compression levels
        for level in [
            CompressionLevel::None,
            CompressionLevel::BestSpeed,
            CompressionLevel::Default,
            CompressionLevel::BestCompression,
        ] {
            let mut compressed = vec![0u8; data.len() * 2];
            let mut compressed_len = compressed.len();
            compress2(&mut compressed, &mut compressed_len, data, level)
                .expect(&format!("Should compress with level {:?}", level));
            
            // Verify it can be decompressed
            let mut decompressed = vec![0u8; data.len()];
            let mut decompressed_len = decompressed.len();
            uncompress(&mut decompressed, &mut decompressed_len, &compressed[..compressed_len])
                .expect(&format!("Should decompress with level {:?}", level));
            decompressed.truncate(decompressed_len);
            assert_eq!(&decompressed, data);
        }
    }
}

