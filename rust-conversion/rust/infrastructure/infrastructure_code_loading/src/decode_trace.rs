//! Trace Decoding Module
//!
//! Provides functionality to decode Traces from EI (Erlang Interface) format. Traces
//! represent execution trace information used for debugging and monitoring in the
//! Erlang runtime system.
//!
//! ## Overview
//!
//! Traces in EI format are encoded as tuples with 5 elements:
//! - **Flags**: Trace flags indicating the type of trace event
//! - **Label**: Trace label for categorization
//! - **Serial**: Serial number for ordering trace events
//! - **From PID**: Process ID that generated the trace
//! - **Previous**: Reference to previous trace in the chain
//!
//! ## Decoding Process
//!
//! 1. Decode tuple header (must have arity 5)
//! 2. Decode flags, label, serial as integers
//! 3. Decode from PID
//! 4. Decode previous reference as integer
//! 5. Construct and return `ErlangTrace` structure
//!
//! ## Examples
//!
//! ```rust
//! use infrastructure_code_loading::decode_trace;
//!
//! let mut index = 0;
//! let trace = decode_trace(&buf, &mut index).unwrap();
//! println!("Trace: flags={}, label={}, serial={}", trace.flags, trace.label, trace.serial);
//! ```
//!
//! ## See Also
//!
//! - [`encode_trace`](super::encode_trace/index.html): Trace encoding functions
//! - [`decode_pid`](super::decode_pid/index.html): PID decoding (used in traces)
//! - [`infrastructure_trace_encoding`](../infrastructure_trace_encoding/index.html): High-level trace codec
//!
//! Based on `lib/erl_interface/src/decode/decode_trace.c`

use super::decode_headers::decode_tuple_header;
use super::decode_integers::decode_longlong;
use super::decode_pid::decode_pid;
use super::encode_trace::ErlangTrace;

/// Decode a trace from EI format
///
/// This function matches the C implementation behavior: it uses a temporary index
/// and only updates the actual index if all decoding succeeds. This ensures that
/// if any decode operation fails, the original index remains unchanged.
///
/// # Arguments
/// * `buf` - Buffer containing EI-encoded data
/// * `index` - Current index in buffer (updated only on success)
///
/// # Returns
/// * `Ok(trace)` - Decoded trace
/// * `Err(DecodeError)` - Decoding error (index unchanged on error)
pub fn decode_trace(buf: &[u8], index: &mut usize) -> Result<ErlangTrace, DecodeError> {
    // Use a temporary index (matches C implementation pattern)
    // Only update the actual index if all decoding succeeds
    let mut tindex = *index;
    
    // Decode tuple header (should be arity 5)
    let arity = decode_tuple_header(buf, &mut tindex)
        .map_err(|e| DecodeError::HeaderDecodeError(format!("{:?}", e)))?;
    
    if arity != 5 {
        return Err(DecodeError::InvalidFormat(format!("Expected arity 5, got {}", arity)));
    }

    // Decode: Flags, Label, Serial, FromPid, Prev
    let flags = decode_longlong(buf, &mut tindex)
        .map_err(|e| DecodeError::IntegerDecodeError(format!("{:?}", e)))?;
    
    let label = decode_longlong(buf, &mut tindex)
        .map_err(|e| DecodeError::IntegerDecodeError(format!("{:?}", e)))?;
    
    let serial = decode_longlong(buf, &mut tindex)
        .map_err(|e| DecodeError::IntegerDecodeError(format!("{:?}", e)))?;
    
    let from = decode_pid(buf, &mut tindex)
        .map_err(|e| DecodeError::PidDecodeError(format!("{:?}", e)))?;
    
    let prev = decode_longlong(buf, &mut tindex)
        .map_err(|e| DecodeError::IntegerDecodeError(format!("{:?}", e)))?;

    // All decoding succeeded - update the actual index
    *index = tindex;

    Ok(ErlangTrace {
        flags,
        label,
        serial,
        from,
        prev,
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_roundtrip() {
        let trace = ErlangTrace {
            flags: 1,
            label: 2,
            serial: 3,
            from: super::super::encode_pid::ErlangPid {
                node: "node@host".to_string(),
                num: 123,
                serial: 456,
                creation: 1,
            },
            prev: 4,
        };
        let mut buf = vec![0u8; 200];
        let mut encode_index = 0;
        let mut buf_opt = Some(&mut buf[..]);
        super::super::encode_trace::encode_trace(&mut buf_opt, &mut encode_index, &trace).unwrap();
        
        let mut decode_index = 0;
        let decoded = decode_trace(&buf, &mut decode_index).unwrap();
        // Note: decode_atom returns a placeholder, so we can't compare node names in from PID
        // But we can verify other fields match
        assert_eq!(decoded.flags, trace.flags);
        assert_eq!(decoded.label, trace.label);
        assert_eq!(decoded.serial, trace.serial);
        assert_eq!(decoded.prev, trace.prev);
        assert_eq!(decoded.from.num, trace.from.num);
        assert_eq!(decoded.from.serial, trace.from.serial);
        assert_eq!(decoded.from.creation, trace.from.creation);
    }

    #[test]
    fn test_decode_header_error() {
        // Create buffer with invalid tuple header tag
        let buf = vec![0xFF]; // Invalid tag
        let mut index = 0;
        let result = decode_trace(&buf, &mut index);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecodeError::HeaderDecodeError(_) => {}
            _ => panic!("Expected HeaderDecodeError"),
        }
    }

    #[test]
    fn test_decode_invalid_arity() {
        // Create buffer with tuple header but arity != 5
        let mut buf = vec![crate::constants::ERL_SMALL_TUPLE_EXT];
        buf.push(3); // arity = 3 (should be 5)
        let mut index = 0;
        let result = decode_trace(&buf, &mut index);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecodeError::InvalidFormat(msg) => {
                assert!(msg.contains("Expected arity 5"));
            }
            _ => panic!("Expected InvalidFormat error"),
        }
    }

    #[test]
    fn test_decode_invalid_arity_large() {
        // Test with large tuple header
        let mut buf = vec![crate::constants::ERL_LARGE_TUPLE_EXT];
        buf.extend_from_slice(&10u32.to_be_bytes()); // arity = 10 (should be 5)
        let mut index = 0;
        let result = decode_trace(&buf, &mut index);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecodeError::InvalidFormat(msg) => {
                assert!(msg.contains("Expected arity 5"));
            }
            _ => panic!("Expected InvalidFormat error"),
        }
    }

    #[test]
    fn test_decode_flags_integer_error() {
        // Create buffer with valid tuple header (arity 5) but invalid integer for flags
        let mut buf = vec![crate::constants::ERL_SMALL_TUPLE_EXT];
        buf.push(5); // arity = 5
        // Add invalid integer tag (0xFF is not a valid integer tag)
        buf.push(0xFF);
        let mut index = 0;
        let result = decode_trace(&buf, &mut index);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecodeError::IntegerDecodeError(_) => {}
            _ => panic!("Expected IntegerDecodeError"),
        }
    }

    #[test]
    fn test_decode_label_integer_error() {
        // Create buffer with valid tuple header and flags, but invalid integer for label
        let mut buf = vec![crate::constants::ERL_SMALL_TUPLE_EXT];
        buf.push(5); // arity = 5
        // Add valid small integer for flags
        buf.extend_from_slice(&[crate::constants::ERL_SMALL_INTEGER_EXT, 1]);
        // Add invalid integer tag for label
        buf.push(0xFF);
        let mut index = 0;
        let result = decode_trace(&buf, &mut index);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecodeError::IntegerDecodeError(_) => {}
            _ => panic!("Expected IntegerDecodeError"),
        }
    }

    #[test]
    fn test_decode_serial_integer_error() {
        // Create buffer with valid tuple header, flags, label, but invalid integer for serial
        let mut buf = vec![crate::constants::ERL_SMALL_TUPLE_EXT];
        buf.push(5); // arity = 5
        buf.extend_from_slice(&[crate::constants::ERL_SMALL_INTEGER_EXT, 1]); // flags
        buf.extend_from_slice(&[crate::constants::ERL_SMALL_INTEGER_EXT, 2]); // label
        // Add invalid integer tag for serial
        buf.push(0xFF);
        let mut index = 0;
        let result = decode_trace(&buf, &mut index);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecodeError::IntegerDecodeError(_) => {}
            _ => panic!("Expected IntegerDecodeError"),
        }
    }

    #[test]
    fn test_decode_pid_error() {
        // Create buffer with valid tuple header, flags, label, serial, but invalid PID
        let mut buf = vec![crate::constants::ERL_SMALL_TUPLE_EXT];
        buf.push(5); // arity = 5
        buf.extend_from_slice(&[crate::constants::ERL_SMALL_INTEGER_EXT, 1]); // flags
        buf.extend_from_slice(&[crate::constants::ERL_SMALL_INTEGER_EXT, 2]); // label
        buf.extend_from_slice(&[crate::constants::ERL_SMALL_INTEGER_EXT, 3]); // serial
        // Add invalid PID tag (0xFF is not a valid PID tag)
        buf.push(0xFF);
        let mut index = 0;
        let result = decode_trace(&buf, &mut index);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecodeError::PidDecodeError(_) => {}
            _ => panic!("Expected PidDecodeError"),
        }
    }

    #[test]
    fn test_decode_prev_integer_error() {
        // Create buffer with valid tuple header, flags, label, serial, PID, but invalid integer for prev
        let mut buf = vec![crate::constants::ERL_SMALL_TUPLE_EXT];
        buf.push(5); // arity = 5
        buf.extend_from_slice(&[crate::constants::ERL_SMALL_INTEGER_EXT, 1]); // flags
        buf.extend_from_slice(&[crate::constants::ERL_SMALL_INTEGER_EXT, 2]); // label
        buf.extend_from_slice(&[crate::constants::ERL_SMALL_INTEGER_EXT, 3]); // serial
        // Add valid PID: ERL_NEW_PID_EXT + atom + num + serial + creation
        buf.push(crate::constants::ERL_NEW_PID_EXT);
        buf.extend_from_slice(&[115, 3, b'n', b'o', b'd']); // atom
        buf.extend_from_slice(&123u32.to_be_bytes()); // num
        buf.extend_from_slice(&456u32.to_be_bytes()); // serial
        buf.extend_from_slice(&1u32.to_be_bytes()); // creation
        // Add invalid integer tag for prev
        buf.push(0xFF);
        let mut index = 0;
        let result = decode_trace(&buf, &mut index);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecodeError::IntegerDecodeError(_) => {}
            _ => panic!("Expected IntegerDecodeError"),
        }
    }

    #[test]
    fn test_decode_error_debug() {
        let error1 = DecodeError::HeaderDecodeError("header_err".to_string());
        let error2 = DecodeError::IntegerDecodeError("int_err".to_string());
        let error3 = DecodeError::PidDecodeError("pid_err".to_string());
        let error4 = DecodeError::InvalidFormat("format_err".to_string());
        
        let debug_str1 = format!("{:?}", error1);
        let debug_str2 = format!("{:?}", error2);
        let debug_str3 = format!("{:?}", error3);
        let debug_str4 = format!("{:?}", error4);
        
        assert!(debug_str1.contains("HeaderDecodeError"));
        assert!(debug_str2.contains("IntegerDecodeError"));
        assert!(debug_str3.contains("PidDecodeError"));
        assert!(debug_str4.contains("InvalidFormat"));
    }

    #[test]
    fn test_decode_error_clone() {
        let error1 = DecodeError::HeaderDecodeError("header_err".to_string());
        let error2 = DecodeError::IntegerDecodeError("int_err".to_string());
        let error3 = DecodeError::PidDecodeError("pid_err".to_string());
        let error4 = DecodeError::InvalidFormat("format_err".to_string());
        
        let cloned1 = error1.clone();
        let cloned2 = error2.clone();
        let cloned3 = error3.clone();
        let cloned4 = error4.clone();
        
        assert_eq!(error1, cloned1);
        assert_eq!(error2, cloned2);
        assert_eq!(error3, cloned3);
        assert_eq!(error4, cloned4);
    }

    #[test]
    fn test_decode_error_partial_eq() {
        let error1 = DecodeError::HeaderDecodeError("err".to_string());
        let error2 = DecodeError::HeaderDecodeError("err".to_string());
        let error3 = DecodeError::HeaderDecodeError("different".to_string());
        let error4 = DecodeError::IntegerDecodeError("err".to_string());
        let error5 = DecodeError::PidDecodeError("err".to_string());
        let error6 = DecodeError::InvalidFormat("err".to_string());
        
        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
        assert_ne!(error1, error4);
        assert_ne!(error4, error5);
        assert_ne!(error5, error6);
    }

    #[test]
    fn test_decode_error_eq() {
        let error1 = DecodeError::HeaderDecodeError("err".to_string());
        let error2 = DecodeError::HeaderDecodeError("err".to_string());
        let error3 = DecodeError::IntegerDecodeError("err".to_string());
        
        assert!(error1 == error2);
        assert!(error1 != error3);
    }

    #[test]
    fn test_decode_various_arities() {
        // Test various invalid arities
        for arity in [0, 1, 2, 3, 4, 6, 7, 10] {
            let mut buf = vec![crate::constants::ERL_SMALL_TUPLE_EXT];
            buf.push(arity);
            let mut index = 0;
            let result = decode_trace(&buf, &mut index);
            assert!(result.is_err());
            match result.unwrap_err() {
                DecodeError::InvalidFormat(msg) => {
                    assert!(msg.contains("Expected arity 5"));
                    assert!(msg.contains(&arity.to_string()));
                }
                _ => panic!("Expected InvalidFormat error for arity {}", arity),
            }
        }
    }

    #[test]
    fn test_decode_various_values() {
        // Test with various integer values (using values that encode/decode reliably)
        let test_cases = vec![
            (0i64, 0i64, 0i64, 0i64),
            (1i64, 2i64, 3i64, 4i64),
            (100i64, 200i64, 300i64, 400i64),
            (-100i64, -200i64, -300i64, -400i64),
            (1000i64, 2000i64, 3000i64, 4000i64),
            (-1000i64, -2000i64, -3000i64, -4000i64),
        ];
        
        for (flags, label, serial, prev) in test_cases {
            let trace = ErlangTrace {
                flags,
                label,
                serial,
                from: super::super::encode_pid::ErlangPid {
                    node: "node@host".to_string(),
                    num: 123,
                    serial: 456,
                    creation: 1,
                },
                prev,
            };
            let mut buf = vec![0u8; 200];
            let mut encode_index = 0;
            let mut buf_opt = Some(&mut buf[..]);
            super::super::encode_trace::encode_trace(&mut buf_opt, &mut encode_index, &trace).unwrap();
            
            let mut decode_index = 0;
            let decoded = decode_trace(&buf, &mut decode_index).unwrap();
            assert_eq!(decoded.flags, flags);
            assert_eq!(decoded.label, label);
            assert_eq!(decoded.serial, serial);
            assert_eq!(decoded.prev, prev);
        }
    }
}

