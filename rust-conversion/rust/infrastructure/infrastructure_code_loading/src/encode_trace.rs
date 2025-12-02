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
/// # Arguments
/// * `buf` - Optional buffer to write to (None for size calculation)
/// * `index` - Current index in buffer
/// * `trace` - The trace to encode
///
/// # Returns
/// * `Ok(())` - Success
/// * `Err(EncodeError)` - Encoding error
pub fn encode_trace(buf: &mut Option<&mut [u8]>, index: &mut usize, trace: &ErlangTrace) -> Result<(), EncodeError> {
    // Encode as tuple: { Flags, Label, Serial, FromPid, Prev }
    encode_tuple_header(buf, index, 5)
        .map_err(|e| EncodeError::HeaderEncodeError)?;
    
    encode_longlong(buf, index, trace.flags)
        .map_err(|e| EncodeError::IntegerEncodeError)?;
    
    encode_longlong(buf, index, trace.label)
        .map_err(|e| EncodeError::IntegerEncodeError)?;
    
    encode_longlong(buf, index, trace.serial)
        .map_err(|e| EncodeError::IntegerEncodeError)?;
    
    encode_pid(buf, index, &trace.from)
        .map_err(|e| EncodeError::PidEncodeError(format!("{:?}", e)))?;
    
    encode_longlong(buf, index, trace.prev)
        .map_err(|e| EncodeError::IntegerEncodeError)?;

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
}

