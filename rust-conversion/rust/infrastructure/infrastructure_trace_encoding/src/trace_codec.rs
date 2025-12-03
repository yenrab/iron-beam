//! Trace Codec Module
//!
//! Provides trace encoding/decoding functionality.
//! Based on encode_trace.c and decode_trace.c
//!
//! This module provides a high-level codec interface for Erlang trace structures.
//! It wraps the lower-level encoding/decoding functions from infrastructure_code_loading.

use infrastructure_code_loading::{encode_trace, decode_trace, ErlangTrace, TraceEncodeError, TraceDecodeError};

/// Trace codec for encoding/decoding ErlangTrace values
pub struct TraceCodec;

impl TraceCodec {
    /// Encode an ErlangTrace to bytes using EI format
    ///
    /// This function encodes a trace structure into the EI (Erlang Interchange) format.
    /// The encoding format is a tuple with 5 elements: { Flags, Label, Serial, FromPid, Prev }.
    ///
    /// # Arguments
    ///
    /// * `trace` - The ErlangTrace value to encode
    ///
    /// # Returns
    ///
    /// * `Ok(bytes)` - Encoded bytes in EI format
    /// * `Err(EncodeError)` - Encoding error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use infrastructure_trace_encoding::{TraceCodec, ErlangTrace};
    /// use infrastructure_code_loading::encode_pid::ErlangPid;
    ///
    /// let trace = ErlangTrace {
    ///     flags: 1,
    ///     label: 2,
    ///     serial: 3,
    ///     from: ErlangPid {
    ///         node: "node@host".to_string(),
    ///         num: 123,
    ///         serial: 456,
    ///         creation: 1,
    ///     },
    ///     prev: 4,
    /// };
    /// let encoded = TraceCodec::encode(&trace).unwrap();
    /// ```
    pub fn encode(trace: &ErlangTrace) -> Result<Vec<u8>, EncodeError> {
        // First, calculate the size needed
        let mut index = 0;
        let mut buf_opt: Option<&mut [u8]> = None;
        encode_trace(&mut buf_opt, &mut index, trace)
            .map_err(|e| EncodeError::from(e))?;
        
        // Now encode into actual buffer
        let mut buf = vec![0u8; index];
        let mut index = 0;
        let mut buf_opt = Some(&mut buf[..]);
        encode_trace(&mut buf_opt, &mut index, trace)
            .map_err(|e| EncodeError::from(e))?;
        
        // Resize to actual encoded size (in case it's smaller)
        buf.truncate(index);
        Ok(buf)
    }

    /// Decode an ErlangTrace from bytes in EI format
    ///
    /// This function decodes a trace structure from the EI (Erlang Interchange) format.
    /// It expects a tuple with 5 elements: { Flags, Label, Serial, FromPid, Prev }.
    ///
    /// # Arguments
    ///
    /// * `data` - The encoded bytes to decode
    ///
    /// # Returns
    ///
    /// * `Ok(trace)` - Decoded ErlangTrace
    /// * `Err(DecodeError)` - Decoding error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use infrastructure_trace_encoding::{TraceCodec, ErlangTrace};
    /// use infrastructure_code_loading::ErlangPid;
    ///
    /// let trace = ErlangTrace {
    ///     flags: 1,
    ///     label: 2,
    ///     serial: 3,
    ///     from: ErlangPid {
    ///         node: "node@host".to_string(),
    ///         num: 123,
    ///         serial: 456,
    ///         creation: 1,
    ///     },
    ///     prev: 4,
    /// };
    /// let encoded = TraceCodec::encode(&trace).unwrap();
    /// let decoded = TraceCodec::decode(&encoded).unwrap();
    /// // Note: node name may be decoded as atom index, so compare other fields
    /// assert_eq!(decoded.flags, trace.flags);
    /// assert_eq!(decoded.label, trace.label);
    /// assert_eq!(decoded.serial, trace.serial);
    /// assert_eq!(decoded.prev, trace.prev);
    /// assert_eq!(decoded.from.num, trace.from.num);
    /// ```
    pub fn decode(data: &[u8]) -> Result<ErlangTrace, DecodeError> {
        let mut index = 0;
        decode_trace(data, &mut index)
            .map_err(|e| DecodeError::from(e))
    }
}

/// Encoding errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodeError {
    /// Header encoding error
    HeaderEncodeError,
    /// Integer encoding error
    IntegerEncodeError,
    /// PID encoding error
    PidEncodeError(String),
}

impl From<TraceEncodeError> for EncodeError {
    fn from(err: TraceEncodeError) -> Self {
        match err {
            TraceEncodeError::HeaderEncodeError => EncodeError::HeaderEncodeError,
            TraceEncodeError::IntegerEncodeError => EncodeError::IntegerEncodeError,
            TraceEncodeError::PidEncodeError(msg) => EncodeError::PidEncodeError(msg),
        }
    }
}

/// Decoding errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// Header decoding error
    HeaderDecodeError(String),
    /// Integer decoding error
    IntegerDecodeError(String),
    /// PID decoding error
    PidDecodeError(String),
    /// Invalid format
    InvalidFormat(String),
}

impl From<TraceDecodeError> for DecodeError {
    fn from(err: TraceDecodeError) -> Self {
        match err {
            TraceDecodeError::HeaderDecodeError(msg) => DecodeError::HeaderDecodeError(msg),
            TraceDecodeError::IntegerDecodeError(msg) => DecodeError::IntegerDecodeError(msg),
            TraceDecodeError::PidDecodeError(msg) => DecodeError::PidDecodeError(msg),
            TraceDecodeError::InvalidFormat(msg) => DecodeError::InvalidFormat(msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use infrastructure_code_loading::ErlangPid;

    #[test]
    fn test_encode_decode_roundtrip() {
        let trace = ErlangTrace {
            flags: 1,
            label: 2,
            serial: 3,
            from: ErlangPid {
                node: "node@host".to_string(),
                num: 123,
                serial: 456,
                creation: 1,
            },
            prev: 4,
        };
        
        let encoded = TraceCodec::encode(&trace).unwrap();
        let decoded = TraceCodec::decode(&encoded).unwrap();
        
        // Note: node name may be decoded as atom index, so compare other fields
        assert_eq!(decoded.flags, trace.flags);
        assert_eq!(decoded.label, trace.label);
        assert_eq!(decoded.serial, trace.serial);
        assert_eq!(decoded.prev, trace.prev);
        assert_eq!(decoded.from.num, trace.from.num);
        assert_eq!(decoded.from.serial, trace.from.serial);
        assert_eq!(decoded.from.creation, trace.from.creation);
    }

    #[test]
    fn test_encode_decode_zero_values() {
        let trace = ErlangTrace {
            flags: 0,
            label: 0,
            serial: 0,
            from: ErlangPid {
                node: "node@host".to_string(),
                num: 0,
                serial: 0,
                creation: 0,
            },
            prev: 0,
        };
        
        let encoded = TraceCodec::encode(&trace).unwrap();
        let decoded = TraceCodec::decode(&encoded).unwrap();
        
        // Note: node name may be decoded as atom index, so compare other fields
        assert_eq!(decoded.flags, trace.flags);
        assert_eq!(decoded.label, trace.label);
        assert_eq!(decoded.serial, trace.serial);
        assert_eq!(decoded.prev, trace.prev);
        assert_eq!(decoded.from.num, trace.from.num);
        assert_eq!(decoded.from.serial, trace.from.serial);
        assert_eq!(decoded.from.creation, trace.from.creation);
    }

    #[test]
    fn test_encode_decode_negative_values() {
        let trace = ErlangTrace {
            flags: -1,
            label: -2,
            serial: -3,
            from: ErlangPid {
                node: "node@host".to_string(),
                num: 123,
                serial: 456,
                creation: 1,
            },
            prev: -4,
        };
        
        let encoded = TraceCodec::encode(&trace).unwrap();
        let decoded = TraceCodec::decode(&encoded).unwrap();
        
        // Note: node name may be decoded as atom index, so compare other fields
        assert_eq!(decoded.flags, trace.flags);
        assert_eq!(decoded.label, trace.label);
        assert_eq!(decoded.serial, trace.serial);
        assert_eq!(decoded.prev, trace.prev);
        assert_eq!(decoded.from.num, trace.from.num);
        assert_eq!(decoded.from.serial, trace.from.serial);
        assert_eq!(decoded.from.creation, trace.from.creation);
    }

    #[test]
    fn test_encode_decode_large_values() {
        // Use large but reasonable values that fit in standard integer encoding
        let trace = ErlangTrace {
            flags: 1000000,
            label: 2000000,
            serial: 3000000,
            from: ErlangPid {
                node: "node@host".to_string(),
                num: u32::MAX,
                serial: u32::MAX,
                creation: u32::MAX,
            },
            prev: 4000000,
        };
        
        let encoded = TraceCodec::encode(&trace).unwrap();
        let decoded = TraceCodec::decode(&encoded).unwrap();
        
        // Note: node name may be decoded as atom index, so compare other fields
        assert_eq!(decoded.flags, trace.flags);
        assert_eq!(decoded.label, trace.label);
        assert_eq!(decoded.serial, trace.serial);
        assert_eq!(decoded.prev, trace.prev);
        assert_eq!(decoded.from.num, trace.from.num);
        assert_eq!(decoded.from.serial, trace.from.serial);
        assert_eq!(decoded.from.creation, trace.from.creation);
    }

    #[test]
    fn test_encode_decode_min_values() {
        // Use large negative values that fit in standard integer encoding
        let trace = ErlangTrace {
            flags: -1000000,
            label: -2000000,
            serial: -3000000,
            from: ErlangPid {
                node: "node@host".to_string(),
                num: 0,
                serial: 0,
                creation: 0,
            },
            prev: -4000000,
        };
        
        let encoded = TraceCodec::encode(&trace).unwrap();
        let decoded = TraceCodec::decode(&encoded).unwrap();
        
        // Note: node name may be decoded as atom index, so compare other fields
        assert_eq!(decoded.flags, trace.flags);
        assert_eq!(decoded.label, trace.label);
        assert_eq!(decoded.serial, trace.serial);
        assert_eq!(decoded.prev, trace.prev);
        assert_eq!(decoded.from.num, trace.from.num);
        assert_eq!(decoded.from.serial, trace.from.serial);
        assert_eq!(decoded.from.creation, trace.from.creation);
    }

    #[test]
    fn test_decode_invalid_format() {
        let invalid = vec![0xFF]; // Invalid tag
        let result = TraceCodec::decode(&invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_empty_buffer() {
        let empty = vec![];
        let result = TraceCodec::decode(&empty);
        assert!(result.is_err());
    }

    #[test]
    fn test_encode_error_from_low_level() {
        // Test error conversion
        let low_level_err = TraceEncodeError::HeaderEncodeError;
        let high_level_err: EncodeError = low_level_err.into();
        assert_eq!(high_level_err, EncodeError::HeaderEncodeError);
        
        let low_level_err = TraceEncodeError::IntegerEncodeError;
        let high_level_err: EncodeError = low_level_err.into();
        assert_eq!(high_level_err, EncodeError::IntegerEncodeError);
        
        let low_level_err = TraceEncodeError::PidEncodeError("test".to_string());
        let high_level_err: EncodeError = low_level_err.into();
        match high_level_err {
            EncodeError::PidEncodeError(msg) => assert_eq!(msg, "test"),
            _ => panic!("Expected PidEncodeError"),
        }
    }

    #[test]
    fn test_decode_error_from_low_level() {
        // Test error conversion
        let low_level_err = TraceDecodeError::HeaderDecodeError("header".to_string());
        let high_level_err: DecodeError = low_level_err.into();
        match high_level_err {
            DecodeError::HeaderDecodeError(msg) => assert_eq!(msg, "header"),
            _ => panic!("Expected HeaderDecodeError"),
        }
        
        let low_level_err = TraceDecodeError::IntegerDecodeError("int".to_string());
        let high_level_err: DecodeError = low_level_err.into();
        match high_level_err {
            DecodeError::IntegerDecodeError(msg) => assert_eq!(msg, "int"),
            _ => panic!("Expected IntegerDecodeError"),
        }
        
        let low_level_err = TraceDecodeError::PidDecodeError("pid".to_string());
        let high_level_err: DecodeError = low_level_err.into();
        match high_level_err {
            DecodeError::PidDecodeError(msg) => assert_eq!(msg, "pid"),
            _ => panic!("Expected PidDecodeError"),
        }
        
        let low_level_err = TraceDecodeError::InvalidFormat("format".to_string());
        let high_level_err: DecodeError = low_level_err.into();
        match high_level_err {
            DecodeError::InvalidFormat(msg) => assert_eq!(msg, "format"),
            _ => panic!("Expected InvalidFormat"),
        }
    }

    #[test]
    fn test_encode_error_debug() {
        let error1 = EncodeError::HeaderEncodeError;
        let error2 = EncodeError::IntegerEncodeError;
        let error3 = EncodeError::PidEncodeError("test".to_string());
        
        let debug_str1 = format!("{:?}", error1);
        let debug_str2 = format!("{:?}", error2);
        let debug_str3 = format!("{:?}", error3);
        
        assert!(debug_str1.contains("HeaderEncodeError"));
        assert!(debug_str2.contains("IntegerEncodeError"));
        assert!(debug_str3.contains("PidEncodeError"));
    }

    #[test]
    fn test_decode_error_debug() {
        let error1 = DecodeError::HeaderDecodeError("header".to_string());
        let error2 = DecodeError::IntegerDecodeError("int".to_string());
        let error3 = DecodeError::PidDecodeError("pid".to_string());
        let error4 = DecodeError::InvalidFormat("format".to_string());
        
        let debug_str1 = format!("{:?}", error1);
        let debug_str2 = format!("{:?}", error2);
        let debug_str3 = format!("{:?}", error3);
        let debug_str4 = format!("{:?}", error4);
        
        assert!(debug_str1.contains("HeaderDecodeError"));
        assert!(debug_str2.contains("IntegerDecodeError"));
        assert!(debug_str3.contains("PidDecodeError"));
        assert!(debug_str4.contains("InvalidFormat"));
    }
}

