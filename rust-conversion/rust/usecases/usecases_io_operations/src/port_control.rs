//! Port Control Module
//!
//! Provides port control operations for test drivers.
//! Based on control_drv.c

/*
 * %CopyrightBegin%
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * Copyright Lee Barney 2025. All Rights Reserved.
 *
 * This file is derived from work copyrighted by Ericsson AB 1996-2025.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * %CopyrightEnd%
 */

use std::sync::atomic::{AtomicU64, Ordering};

/// Control driver state
pub struct ControlDriver {
    /// The Erlang port associated with this driver
    erlang_port: AtomicU64,
}

impl ControlDriver {
    /// Create a new control driver
    pub fn new() -> Self {
        Self {
            erlang_port: AtomicU64::new(u64::MAX), // -1 in C
        }
    }

    /// Start the control driver (control_start)
    ///
    /// Associates a port with this driver.
    ///
    /// # Arguments
    /// * `port` - Port ID
    ///
    /// # Returns
    /// Port data or error
    pub fn start(&self, port: u64) -> Result<u64, ControlError> {
        let expected = u64::MAX;
        match self.erlang_port.compare_exchange(
            expected,
            port,
            Ordering::Acquire,
            Ordering::Relaxed,
        ) {
            Ok(_) => Ok(port),
            Err(_) => Err(ControlError::AlreadyStarted),
        }
    }

    /// Stop the control driver (control_stop)
    ///
    /// Disassociates the port from this driver.
    ///
    /// # Arguments
    /// * `port` - Port ID
    pub fn stop(&self, _port: u64) {
        self.erlang_port.store(u64::MAX, Ordering::Release);
    }

    /// Read from control driver (control_read)
    ///
    /// Reads data and outputs it to the Erlang port.
    ///
    /// # Arguments
    /// * `port` - Port ID
    /// * `data` - Data to read
    pub fn read(&self, _port: u64, _data: &[u8]) -> Result<(), ControlError> {
        let erlang_port = self.erlang_port.load(Ordering::Acquire);
        if erlang_port == u64::MAX {
            return Err(ControlError::NotStarted);
        }
        
        // TODO: Output data to erlang_port
        // This requires integration with the VM's port output system
        Ok(())
    }

    /// Control operation (control_control)
    ///
    /// Handles control operations for the driver.
    ///
    /// # Arguments
    /// * `port` - Port ID
    /// * `command` - Control command
    /// * `data` - Command data
    /// * `response_size` - Maximum response size
    ///
    /// # Returns
    /// Response data and size
    pub fn control(
        &self,
        _port: u64,
        command: u32,
        data: &[u8],
        response_size: usize,
    ) -> Result<(Vec<u8>, usize), ControlError> {
        let erlang_port = self.erlang_port.load(Ordering::Acquire);
        if erlang_port == u64::MAX {
            return Err(ControlError::NotStarted);
        }

        match command as u8 as char {
            'e' => {
                // Echo command: return the data
                if data.len() > response_size {
                    // Need to allocate
                    Ok((data.to_vec(), data.len()))
                } else {
                    // Can use provided buffer
                    let mut response = vec![0u8; data.len()];
                    response.copy_from_slice(data);
                    Ok((response, data.len()))
                }
            }
            'b' => {
                // Set busy port
                if !data.is_empty() {
                    let _busy = data[0] != 0;
                    // TODO: set_busy_port(erlang_port, busy)
                }
                Ok((Vec::new(), 0))
            }
            'i' => {
                // Output data to port
                // TODO: driver_output(erlang_port, data)
                Ok((Vec::new(), 0))
            }
            _ => {
                if command < 256 {
                    Err(ControlError::InvalidCommand)
                } else {
                    // Return command as 4-byte big-endian
                    let mut response = vec![0u8; 4];
                    response[0] = ((command >> 24) & 0xFF) as u8;
                    response[1] = ((command >> 16) & 0xFF) as u8;
                    response[2] = ((command >> 8) & 0xFF) as u8;
                    response[3] = (command & 0xFF) as u8;
                    Ok((response, 4))
                }
            }
        }
    }
}

impl Default for ControlDriver {
    fn default() -> Self {
        Self::new()
    }
}

/// Control driver errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlError {
    /// Driver already started
    AlreadyStarted,
    /// Driver not started
    NotStarted,
    /// Invalid command
    InvalidCommand,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_control_driver_new() {
        let driver = ControlDriver::new();
        let port = driver.erlang_port.load(Ordering::Acquire);
        assert_eq!(port, u64::MAX);
    }

    #[test]
    fn test_control_driver_start_stop() {
        let driver = ControlDriver::new();
        
        // Start with port 123
        let result = driver.start(123);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 123);
        
        // Try to start again (should fail)
        let result = driver.start(456);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ControlError::AlreadyStarted);
        
        // Stop
        driver.stop(123);
        
        // Can start again after stop
        let result = driver.start(456);
        assert!(result.is_ok());
    }

    #[test]
    fn test_control_driver_read_not_started() {
        let driver = ControlDriver::new();
        let result = driver.read(123, b"test");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ControlError::NotStarted);
    }

    #[test]
    fn test_control_driver_control_echo() {
        let driver = ControlDriver::new();
        driver.start(123).unwrap();
        
        let (response, size) = driver.control(123, 'e' as u32, b"hello", 10).unwrap();
        assert_eq!(size, 5);
        assert_eq!(response, b"hello");
    }

    #[test]
    fn test_control_driver_control_large_echo() {
        let driver = ControlDriver::new();
        driver.start(123).unwrap();
        
        let data = b"this is a large string that exceeds the response buffer";
        let (response, size) = driver.control(123, 'e' as u32, data, 10).unwrap();
        assert_eq!(size, data.len());
        assert_eq!(response, data);
    }

    #[test]
    fn test_control_driver_control_busy() {
        let driver = ControlDriver::new();
        driver.start(123).unwrap();
        
        let (response, size) = driver.control(123, 'b' as u32, &[1], 10).unwrap();
        assert_eq!(size, 0);
        assert!(response.is_empty());
    }

    #[test]
    fn test_control_driver_control_output() {
        let driver = ControlDriver::new();
        driver.start(123).unwrap();
        
        let (response, size) = driver.control(123, 'i' as u32, b"output", 10).unwrap();
        assert_eq!(size, 0);
        assert!(response.is_empty());
    }

    #[test]
    fn test_control_driver_control_invalid_single_byte() {
        let driver = ControlDriver::new();
        driver.start(123).unwrap();
        
        let result = driver.control(123, 'x' as u32, b"", 10);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ControlError::InvalidCommand);
    }

    #[test]
    fn test_control_driver_control_large_command() {
        let driver = ControlDriver::new();
        driver.start(123).unwrap();
        
        let command = 0x12345678u32;
        let (response, size) = driver.control(123, command, b"", 10).unwrap();
        assert_eq!(size, 4);
        assert_eq!(response, [0x12, 0x34, 0x56, 0x78]);
    }

    #[test]
    fn test_control_driver_control_not_started() {
        let driver = ControlDriver::new();
        let result = driver.control(123, 'e' as u32, b"test", 10);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ControlError::NotStarted);
    }
}
