//! Output Operations
//!
//! Provides output functions for sending data from drivers to Erlang.

use super::types::*;
use super::queue::*;

/// Output data to Erlang
///
/// Equivalent to C's `driver_output`.
///
/// # Arguments
///
/// * `port` - Driver port
/// * `buf` - Buffer containing data to output
/// * `len` - Length of data to output
///
/// # Returns
///
/// Returns 0 on success, -1 on error.
pub fn driver_output(port: DriverPort, buf: &[u8]) -> i32 {
    // In a real implementation, this would send data directly to Erlang.
    // For now, we enqueue it to the output queue.
    driver_enq(port, buf).map(|_| 0).unwrap_or(-1)
}

/// Output data with header to Erlang
///
/// Equivalent to C's `driver_output2`.
pub fn driver_output2(port: DriverPort, hbuf: &[u8], buf: &[u8]) -> i32 {
    let mut combined = Vec::with_capacity(hbuf.len() + buf.len());
    combined.extend_from_slice(hbuf);
    combined.extend_from_slice(buf);
    driver_output(port, &combined)
}

/// Output binary to Erlang
///
/// Equivalent to C's `driver_output_binary`.
pub fn driver_output_binary(port: DriverPort, hbuf: &[u8], bin: &DriverBinary) -> i32 {
    let mut combined = Vec::with_capacity(hbuf.len() + bin.data.len());
    combined.extend_from_slice(hbuf);
    combined.extend_from_slice(&bin.data);
    driver_output(port, &combined)
}

/// Output I/O vectors to Erlang
///
/// Equivalent to C's `driver_outputv`.
pub fn driver_outputv(port: DriverPort, hbuf: &[u8], iov: &DriverIOVec) -> i32 {
    // Combine header and I/O vectors
    let mut combined = Vec::with_capacity(hbuf.len() + iov.size);
    combined.extend_from_slice(hbuf);
    
    for vec in &iov.iov {
        unsafe {
            let slice = std::slice::from_raw_parts(vec.base, vec.len);
            combined.extend_from_slice(slice);
        }
    }
    
    driver_output(port, &combined)
}


