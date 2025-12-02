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
}

