//! Trace Encoding Module
//!
//! Provides functionality to encode traces to EI format.
//! Based on lib/erl_interface/src/encode/encode_trace.c

use super::encode_pid::{encode_pid, ErlangPid, EncodeError as PidEncodeError};
use super::encode_headers::encode_tuple_header;
use super::encode_integers::encode_longlong;

/// Trace structure
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErlangTrace {
    /// Flags
    pub flags: i64,
    /// Label
    pub label: i64,
    /// Serial number
    pub serial: i64,
    /// From PID
    pub from: ErlangPid,
    /// Previous
    pub prev: i64,
}

/// Encode a trace to EI format
///
/// This function matches the C implementation `ei_encode_trace()`.
/// It encodes a trace as a tuple with 5 elements: { Flags, Label, Serial, FromPid, Prev }.
///
/// The C code uses `ei_encode_long()` which is a wrapper around `ei_encode_longlong()`.
/// This Rust implementation uses `encode_longlong()` directly, which is equivalent.
///
/// # Arguments
/// * `buf` - Optional buffer to write to (None for size calculation)
/// * `index` - Current index in buffer (updated as encoding progresses)
/// * `trace` - The trace to encode
///
/// # Returns
/// * `Ok(())` - Success (index updated)
/// * `Err(EncodeError)` - Encoding error (index may be partially updated)
pub fn encode_trace(buf: &mut Option<&mut [u8]>, index: &mut usize, trace: &ErlangTrace) -> Result<(), EncodeError> {
    // Encode as tuple: { Flags, Label, Serial, FromPid, Prev }
    // Matches C: ei_encode_tuple_header(buf,index,5)
    encode_tuple_header(buf, index, 5)
        .map_err(|_e| EncodeError::HeaderEncodeError)?;
    
    // Matches C: ei_encode_long(buf,index,p->flags) -> calls ei_encode_longlong
    encode_longlong(buf, index, trace.flags)
        .map_err(|_e| EncodeError::IntegerEncodeError)?;
    
    // Matches C: ei_encode_long(buf,index,p->label)
    encode_longlong(buf, index, trace.label)
        .map_err(|_e| EncodeError::IntegerEncodeError)?;
    
    // Matches C: ei_encode_long(buf,index,p->serial)
    encode_longlong(buf, index, trace.serial)
        .map_err(|_e| EncodeError::IntegerEncodeError)?;
    
    // Matches C: ei_encode_pid(buf,index,&p->from)
    encode_pid(buf, index, &trace.from)
        .map_err(|e| EncodeError::PidEncodeError(format!("{:?}", e)))?;
    
    // Matches C: ei_encode_long(buf,index,p->prev)
    encode_longlong(buf, index, trace.prev)
        .map_err(|_e| EncodeError::IntegerEncodeError)?;

    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_trace() {
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
        let mut buf = vec![0u8; 200];
        let mut index = 0;
        encode_trace(&mut Some(&mut buf), &mut index, &trace).unwrap();
        assert!(index > 0);
    }

    #[test]
    fn test_encode_trace_size_calculation() {
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
        let mut index = 0;
        let mut buf_opt = None;
        encode_trace(&mut buf_opt, &mut index, &trace).unwrap();
        assert!(index > 0);
    }

    #[test]
    fn test_encode_trace_various_values() {
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
                from: ErlangPid {
                    node: "node@host".to_string(),
                    num: 123,
                    serial: 456,
                    creation: 1,
                },
                prev,
            };
            let mut buf = vec![0u8; 200];
            let mut index = 0;
            encode_trace(&mut Some(&mut buf), &mut index, &trace).unwrap();
            assert!(index > 0);
        }
    }

    #[test]
    fn test_encode_error_debug() {
        let error1 = EncodeError::HeaderEncodeError;
        let error2 = EncodeError::IntegerEncodeError;
        let error3 = EncodeError::PidEncodeError("pid_err".to_string());
        
        let debug_str1 = format!("{:?}", error1);
        let debug_str2 = format!("{:?}", error2);
        let debug_str3 = format!("{:?}", error3);
        
        assert!(debug_str1.contains("HeaderEncodeError"));
        assert!(debug_str2.contains("IntegerEncodeError"));
        assert!(debug_str3.contains("PidEncodeError"));
    }

    #[test]
    fn test_encode_error_clone() {
        let error1 = EncodeError::HeaderEncodeError;
        let error2 = EncodeError::IntegerEncodeError;
        let error3 = EncodeError::PidEncodeError("pid_err".to_string());
        
        let cloned1 = error1.clone();
        let cloned2 = error2.clone();
        let cloned3 = error3.clone();
        
        assert_eq!(error1, cloned1);
        assert_eq!(error2, cloned2);
        assert_eq!(error3, cloned3);
    }

    #[test]
    fn test_encode_error_partial_eq() {
        let error1 = EncodeError::HeaderEncodeError;
        let error2 = EncodeError::HeaderEncodeError;
        let error3 = EncodeError::IntegerEncodeError;
        let error4 = EncodeError::IntegerEncodeError;
        let error5 = EncodeError::PidEncodeError("err".to_string());
        let error6 = EncodeError::PidEncodeError("err".to_string());
        let error7 = EncodeError::PidEncodeError("different".to_string());
        
        assert_eq!(error1, error2);
        assert_eq!(error3, error4);
        assert_eq!(error5, error6);
        assert_ne!(error5, error7);
        assert_ne!(error1, error3);
        assert_ne!(error3, error5);
    }

    #[test]
    fn test_encode_error_eq() {
        let error1 = EncodeError::HeaderEncodeError;
        let error2 = EncodeError::HeaderEncodeError;
        let error3 = EncodeError::IntegerEncodeError;
        
        assert!(error1 == error2);
        assert!(error1 != error3);
    }

    #[test]
    fn test_erlang_trace_debug() {
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
        
        let debug_str = format!("{:?}", trace);
        assert!(debug_str.contains("ErlangTrace"));
    }

    #[test]
    fn test_erlang_trace_clone() {
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
        
        let cloned = trace.clone();
        assert_eq!(trace, cloned);
    }

    #[test]
    fn test_erlang_trace_partial_eq() {
        let trace1 = ErlangTrace {
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
        let trace2 = ErlangTrace {
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
        let trace3 = ErlangTrace {
            flags: 2,
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
        let trace4 = ErlangTrace {
            flags: 1,
            label: 3,
            serial: 3,
            from: ErlangPid {
                node: "node@host".to_string(),
                num: 123,
                serial: 456,
                creation: 1,
            },
            prev: 4,
        };
        let trace5 = ErlangTrace {
            flags: 1,
            label: 2,
            serial: 4,
            from: ErlangPid {
                node: "node@host".to_string(),
                num: 123,
                serial: 456,
                creation: 1,
            },
            prev: 4,
        };
        let trace6 = ErlangTrace {
            flags: 1,
            label: 2,
            serial: 3,
            from: ErlangPid {
                node: "node@host".to_string(),
                num: 123,
                serial: 456,
                creation: 1,
            },
            prev: 5,
        };
        let trace7 = ErlangTrace {
            flags: 1,
            label: 2,
            serial: 3,
            from: ErlangPid {
                node: "different@host".to_string(),
                num: 123,
                serial: 456,
                creation: 1,
            },
            prev: 4,
        };
        
        assert_eq!(trace1, trace2);
        assert_ne!(trace1, trace3);
        assert_ne!(trace1, trace4);
        assert_ne!(trace1, trace5);
        assert_ne!(trace1, trace6);
        assert_ne!(trace1, trace7);
    }

    #[test]
    fn test_erlang_trace_eq() {
        let trace1 = ErlangTrace {
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
        let trace2 = ErlangTrace {
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
        let trace3 = ErlangTrace {
            flags: 2,
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
        
        assert!(trace1 == trace2);
        assert!(trace1 != trace3);
    }

    #[test]
    fn test_encode_trace_header_error() {
        // Buffer too small for tuple header
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
        let mut buf = vec![0u8; 1]; // Too small for tuple header
        let mut index = 0;
        let result = encode_trace(&mut Some(&mut buf), &mut index, &trace);
        assert!(result.is_err());
        match result.unwrap_err() {
            EncodeError::HeaderEncodeError => {}
            _ => panic!("Expected HeaderEncodeError"),
        }
    }

    #[test]
    fn test_encode_trace_flags_integer_error() {
        // Buffer too small for flags encoding (after tuple header)
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
        let mut buf = vec![0u8; 3]; // Enough for small tuple header (2 bytes) but not for flags
        let mut index = 0;
        let result = encode_trace(&mut Some(&mut buf), &mut index, &trace);
        assert!(result.is_err());
        match result.unwrap_err() {
            EncodeError::IntegerEncodeError => {}
            _ => panic!("Expected IntegerEncodeError"),
        }
    }

    #[test]
    fn test_encode_trace_label_integer_error() {
        // Buffer too small for label encoding (after flags)
        let trace = ErlangTrace {
            flags: 0, // Small integer (2 bytes)
            label: 1, // Small integer (2 bytes)
            serial: 3,
            from: ErlangPid {
                node: "node@host".to_string(),
                num: 123,
                serial: 456,
                creation: 1,
            },
            prev: 4,
        };
        let mut buf = vec![0u8; 5]; // Enough for header (2) + flags (2) but not for label
        let mut index = 0;
        let result = encode_trace(&mut Some(&mut buf), &mut index, &trace);
        assert!(result.is_err());
        match result.unwrap_err() {
            EncodeError::IntegerEncodeError => {}
            _ => panic!("Expected IntegerEncodeError"),
        }
    }

    #[test]
    fn test_encode_trace_serial_integer_error() {
        // Buffer too small for serial encoding (after label)
        let trace = ErlangTrace {
            flags: 0, // Small integer (2 bytes)
            label: 0, // Small integer (2 bytes)
            serial: 1, // Small integer (2 bytes)
            from: ErlangPid {
                node: "node@host".to_string(),
                num: 123,
                serial: 456,
                creation: 1,
            },
            prev: 4,
        };
        let mut buf = vec![0u8; 7]; // Enough for header (2) + flags (2) + label (2) but not for serial
        let mut index = 0;
        let result = encode_trace(&mut Some(&mut buf), &mut index, &trace);
        assert!(result.is_err());
        match result.unwrap_err() {
            EncodeError::IntegerEncodeError => {}
            _ => panic!("Expected IntegerEncodeError"),
        }
    }

    #[test]
    fn test_encode_trace_pid_error() {
        // Buffer too small for PID encoding (after serial)
        let trace = ErlangTrace {
            flags: 0, // Small integer (2 bytes)
            label: 0, // Small integer (2 bytes)
            serial: 0, // Small integer (2 bytes)
            from: ErlangPid {
                node: "node@host".to_string(),
                num: 123,
                serial: 456,
                creation: 1,
            },
            prev: 4,
        };
        // Need enough for: header (2) + flags (2) + label (2) + serial (2) = 8 bytes
        // PID encoding needs at least: tag (1) + atom length (2) + atom bytes (9 for "node@host") + num (4) + serial (4) + creation (4) = 24 bytes
        let mut buf = vec![0u8; 9]; // Enough for header + integers but not for PID
        let mut index = 0;
        let result = encode_trace(&mut Some(&mut buf), &mut index, &trace);
        assert!(result.is_err());
        match result.unwrap_err() {
            EncodeError::PidEncodeError(_) => {}
            _ => panic!("Expected PidEncodeError"),
        }
    }

    #[test]
    fn test_encode_trace_prev_integer_error() {
        // Buffer too small for prev encoding (after PID)
        let trace = ErlangTrace {
            flags: 0, // Small integer (2 bytes)
            label: 0, // Small integer (2 bytes)
            serial: 0, // Small integer (2 bytes)
            from: ErlangPid {
                node: "a".to_string(), // Short node name to minimize PID size
                num: 0,
                serial: 0,
                creation: 0,
            },
            prev: 1, // Small integer (2 bytes)
        };
        // Calculate minimum size needed:
        // header (2) + flags (2) + label (2) + serial (2) = 8
        // PID: tag (1) + atom len (2) + "a" (1) + num (4) + serial (4) + creation (4) = 16
        // Total so far: 8 + 16 = 24, need 2 more for prev = 26
        let mut buf = vec![0u8; 25]; // Enough for everything except prev
        let mut index = 0;
        let result = encode_trace(&mut Some(&mut buf), &mut index, &trace);
        assert!(result.is_err());
        match result.unwrap_err() {
            EncodeError::IntegerEncodeError => {}
            _ => panic!("Expected IntegerEncodeError"),
        }
    }

    #[test]
    fn test_encode_trace_large_integers() {
        // Test with large integers that require INTEGER_EXT (5 bytes) or SMALL_BIG_EXT
        let trace = ErlangTrace {
            flags: 256, // Requires INTEGER_EXT (5 bytes)
            label: 256,
            serial: 256,
            from: ErlangPid {
                node: "node@host".to_string(),
                num: 123,
                serial: 456,
                creation: 1,
            },
            prev: 256,
        };
        let mut buf = vec![0u8; 200];
        let mut index = 0;
        let result = encode_trace(&mut Some(&mut buf), &mut index, &trace);
        assert!(result.is_ok());
        assert!(index > 0);
    }

    #[test]
    fn test_encode_trace_negative_integers() {
        // Test with negative integers
        let trace = ErlangTrace {
            flags: -1,
            label: -100,
            serial: -1000,
            from: ErlangPid {
                node: "node@host".to_string(),
                num: 123,
                serial: 456,
                creation: 1,
            },
            prev: -10000,
        };
        let mut buf = vec![0u8; 200];
        let mut index = 0;
        let result = encode_trace(&mut Some(&mut buf), &mut index, &trace);
        assert!(result.is_ok());
        assert!(index > 0);
    }

    #[test]
    fn test_encode_trace_max_values() {
        // Test with maximum i64 values
        let trace = ErlangTrace {
            flags: i64::MAX,
            label: i64::MAX,
            serial: i64::MAX,
            from: ErlangPid {
                node: "node@host".to_string(),
                num: 123,
                serial: 456,
                creation: 1,
            },
            prev: i64::MAX,
        };
        let mut buf = vec![0u8; 500]; // Large buffer for big integers
        let mut index = 0;
        let result = encode_trace(&mut Some(&mut buf), &mut index, &trace);
        assert!(result.is_ok());
        assert!(index > 0);
    }

    #[test]
    fn test_encode_trace_min_values() {
        // Test with minimum i64 values (except i64::MIN which might cause issues)
        let trace = ErlangTrace {
            flags: i64::MIN + 1,
            label: i64::MIN + 1,
            serial: i64::MIN + 1,
            from: ErlangPid {
                node: "node@host".to_string(),
                num: 123,
                serial: 456,
                creation: 1,
            },
            prev: i64::MIN + 1,
        };
        let mut buf = vec![0u8; 500]; // Large buffer for big integers
        let mut index = 0;
        let result = encode_trace(&mut Some(&mut buf), &mut index, &trace);
        assert!(result.is_ok());
        assert!(index > 0);
    }
}

