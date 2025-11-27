//! Port core data structure
//!
//! Provides the Port type definition based on struct _erl_drv_port from erl_port.h

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

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use crate::common::{ErtsPTabElementCommon, Eterm, PortId};

/// Port structure
///
/// Represents an Erlang port. This is a simplified version of the
/// full C struct _erl_drv_port. The common field must be first
/// (as required by the C code structure).
///
/// Based on struct _erl_drv_port in erl_port.h
#[derive(Debug)]
pub struct Port {
    /// Common element (MUST be first field)
    pub common: ErtsPTabElementCommon,
    
    /// Lock (mutex pointer - placeholder)
    pub lock: Option<*mut ()>, // erts_mtx_t type placeholder
    
    /// Run queue (atomic pointer)
    pub run_queue: AtomicU64,
    
    /// Connected process (atomic)
    pub connected: AtomicU64, // Eterm (process ID)
    
    /// Current caller
    pub caller: Eterm,
    
    /// Data associated with port (atomic)
    pub data: AtomicU64, // Eterm or pointer
    
    /// Number of bytes read
    pub bytes_in: u64,
    /// Number of bytes written
    pub bytes_out: u64,
    
    /// String used in the open
    pub name: Option<String>,
    
    /// Driver pointer (placeholder)
    pub drv_ptr: Option<*mut ()>, // erts_driver_t type placeholder
    
    /// Driver data
    pub drv_data: u64, // UWord
    
    /// Child process ID (OS PID)
    pub os_pid: i64, // SWord
    
    /// Line buffer (for line-oriented I/O)
    pub linebuf: Option<*mut ()>, // LineBuf type placeholder
    
    /// Status and type flags (atomic)
    pub state: AtomicU32,
    
    /// Flags for port_control()
    pub control_flags: i32,
    
    /// Port specific data (atomic)
    pub psd: AtomicU64, // erts_atomic_t
    
    /// Reductions (only used while executing driver callbacks)
    pub reds: i32,
    
    /// Async open port reference
    pub async_open_port: Option<AsyncOpenPort>,
}

/// Async open port structure
#[derive(Debug, Clone)]
pub struct AsyncOpenPort {
    /// Target process
    pub to: Eterm,
    /// Reference array
    pub ref_array: [u32; 3], // ERTS_REF_NUMBERS typically 3
}

impl Port {
    /// Create a new port
    pub fn new(id: PortId) -> Self {
        Self {
            common: ErtsPTabElementCommon::new(id),
            lock: None,
            run_queue: AtomicU64::new(0),
            connected: AtomicU64::new(0),
            caller: 0,
            data: AtomicU64::new(0),
            bytes_in: 0,
            bytes_out: 0,
            name: None,
            drv_ptr: None,
            drv_data: 0,
            os_pid: 0,
            linebuf: None,
            state: AtomicU32::new(0),
            control_flags: 0,
            psd: AtomicU64::new(0),
            reds: 0,
            async_open_port: None,
        }
    }

    /// Get port ID
    pub fn get_id(&self) -> PortId {
        self.common.get_id()
    }

    /// Get port state
    pub fn get_state(&self) -> PortState {
        PortState::from_bits(self.state.load(Ordering::Acquire))
    }

    /// Set port state
    pub fn set_state(&self, state: PortState) {
        self.state.store(state.bits(), Ordering::Release);
    }

    /// Get connected process
    pub fn get_connected(&self) -> Eterm {
        self.connected.load(Ordering::Acquire)
    }

    /// Set connected process
    pub fn set_connected(&self, pid: Eterm) {
        self.connected.store(pid, Ordering::Release);
    }

    /// Get port data
    pub fn get_data(&self) -> u64 {
        self.data.load(Ordering::Acquire)
    }

    /// Set port data
    pub fn set_data(&self, data: u64) {
        self.data.store(data, Ordering::Release);
    }
}

/// Port state flags
///
/// Based on ERTS_PORT_SFLG_* flags in erl_port.h
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortState {
    bits: u32,
}

impl PortState {
    /// Create from bits
    pub fn from_bits(bits: u32) -> Self {
        Self { bits }
    }

    /// Get bits
    pub fn bits(&self) -> u32 {
        self.bits
    }

    /// Check if port is connected
    pub fn is_connected(&self) -> bool {
        (self.bits & PortStatusFlags::CONNECTED.bits()) != 0
    }

    /// Check if port is exiting
    pub fn is_exiting(&self) -> bool {
        (self.bits & PortStatusFlags::EXITING.bits()) != 0
    }

    /// Check if port is closing
    pub fn is_closing(&self) -> bool {
        (self.bits & PortStatusFlags::CLOSING.bits()) != 0
    }

    /// Check if port has binary I/O
    pub fn has_binary_io(&self) -> bool {
        (self.bits & PortStatusFlags::BINARY_IO.bits()) != 0
    }

    /// Check if port has line buffer I/O
    pub fn has_linebuf_io(&self) -> bool {
        (self.bits & PortStatusFlags::LINEBUF_IO.bits()) != 0
    }
}

impl Default for PortState {
    fn default() -> Self {
        Self { bits: 0 }
    }
}

/// Port status flags
///
/// Based on ERTS_PORT_SFLG_* definitions in erl_port.h
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortStatusFlags {
    /// Port is connected
    CONNECTED = 1 << 0,
    /// Port has begun exiting
    EXITING = 1 << 1,
    /// Distribution port
    DISTRIBUTION = 1 << 2,
    /// Binary I/O
    BINARY_IO = 1 << 3,
    /// Soft EOF
    SOFT_EOF = 1 << 4,
    /// Port is closing (no i/o accepted)
    CLOSING = 1 << 5,
    /// Send a closed message when terminating
    SEND_CLOSED = 1 << 6,
    /// Line oriented I/O on port
    LINEBUF_IO = 1 << 7,
    /// Immortal port (only certain system ports)
    FREE = 1 << 8,
    /// Port is initializing
    INITIALIZING = 1 << 9,
    /// Port uses port specific locking
    PORT_SPECIFIC_LOCK = 1 << 10,
    /// Port is invalid
    INVALID = 1 << 11,
    /// Last port to terminate halts the emulator
    HALT = 1 << 12,
    /// Check if the event in ready_input should be cleaned
    CHECK_FD_CLEANUP = 1 << 13,
}

impl PortStatusFlags {
    /// Get flag bits
    pub fn bits(&self) -> u32 {
        *self as u32
    }

    /// Check if flag is set in bits
    pub fn is_set(&self, bits: u32) -> bool {
        (bits & self.bits()) != 0
    }
}

/// Port flags
///
/// Convenience wrapper for port status flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortFlags {
    /// Binary I/O flag
    pub binary_io: bool,
    /// Line buffer I/O flag
    pub linebuf_io: bool,
    /// Soft EOF flag
    pub soft_eof: bool,
}

impl PortFlags {
    /// Create new flags
    pub fn new() -> Self {
        Self {
            binary_io: false,
            linebuf_io: false,
            soft_eof: false,
        }
    }

    /// Create from port state
    pub fn from_state(state: &PortState) -> Self {
        Self {
            binary_io: state.has_binary_io(),
            linebuf_io: state.has_linebuf_io(),
            soft_eof: false, // Would need to check SOFT_EOF flag
        }
    }
}

impl Default for PortFlags {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_creation() {
        let port = Port::new(123);
        assert_eq!(port.get_id(), 123);
        assert_eq!(port.bytes_in, 0);
        assert_eq!(port.bytes_out, 0);
    }

    #[test]
    fn test_port_state() {
        let port = Port::new(456);
        let state = port.get_state();
        assert!(!state.is_connected());
        assert!(!state.is_exiting());
        assert!(!state.is_closing());
    }

    #[test]
    fn test_port_connected() {
        let port = Port::new(789);
        assert_eq!(port.get_connected(), 0);
        
        port.set_connected(100);
        assert_eq!(port.get_connected(), 100);
    }

    #[test]
    fn test_port_data() {
        let port = Port::new(101);
        assert_eq!(port.get_data(), 0);
        
        port.set_data(0x12345678);
        assert_eq!(port.get_data(), 0x12345678);
    }

    #[test]
    fn test_port_status_flags() {
        assert_eq!(PortStatusFlags::CONNECTED.bits(), 1);
        assert_eq!(PortStatusFlags::EXITING.bits(), 2);
        assert_eq!(PortStatusFlags::BINARY_IO.bits(), 8);
        assert_eq!(PortStatusFlags::LINEBUF_IO.bits(), 128);
    }

    #[test]
    fn test_port_flags() {
        let flags = PortFlags::new();
        assert!(!flags.binary_io);
        assert!(!flags.linebuf_io);
        assert!(!flags.soft_eof);
    }

    #[test]
    fn test_port_state_with_flags() {
        let port = Port::new(100);
        
        // Test with CONNECTED flag
        let state = PortState::from_bits(PortStatusFlags::CONNECTED.bits());
        port.set_state(state);
        assert!(port.get_state().is_connected());
        assert!(!port.get_state().is_exiting());
        assert!(!port.get_state().is_closing());
        
        // Test with EXITING flag
        let state = PortState::from_bits(PortStatusFlags::EXITING.bits());
        port.set_state(state);
        assert!(port.get_state().is_exiting());
        assert!(!port.get_state().is_connected());
        
        // Test with CLOSING flag
        let state = PortState::from_bits(PortStatusFlags::CLOSING.bits());
        port.set_state(state);
        assert!(port.get_state().is_closing());
        
        // Test with BINARY_IO flag
        let state = PortState::from_bits(PortStatusFlags::BINARY_IO.bits());
        port.set_state(state);
        assert!(port.get_state().has_binary_io());
        
        // Test with LINEBUF_IO flag
        let state = PortState::from_bits(PortStatusFlags::LINEBUF_IO.bits());
        port.set_state(state);
        assert!(port.get_state().has_linebuf_io());
    }

    #[test]
    fn test_port_state_combined_flags() {
        let port = Port::new(200);
        
        // Test with multiple flags combined
        let combined = PortStatusFlags::CONNECTED.bits() 
            | PortStatusFlags::BINARY_IO.bits()
            | PortStatusFlags::EXITING.bits();
        let state = PortState::from_bits(combined);
        port.set_state(state);
        
        let current_state = port.get_state();
        assert!(current_state.is_connected());
        assert!(current_state.has_binary_io());
        assert!(current_state.is_exiting());
    }

    #[test]
    fn test_port_status_flags_all() {
        // Test all status flags
        assert_eq!(PortStatusFlags::CONNECTED.bits(), 1);
        assert_eq!(PortStatusFlags::EXITING.bits(), 2);
        assert_eq!(PortStatusFlags::DISTRIBUTION.bits(), 4);
        assert_eq!(PortStatusFlags::BINARY_IO.bits(), 8);
        assert_eq!(PortStatusFlags::SOFT_EOF.bits(), 16);
        assert_eq!(PortStatusFlags::CLOSING.bits(), 32);
        assert_eq!(PortStatusFlags::SEND_CLOSED.bits(), 64);
        assert_eq!(PortStatusFlags::LINEBUF_IO.bits(), 128);
        assert_eq!(PortStatusFlags::FREE.bits(), 256);
        assert_eq!(PortStatusFlags::INITIALIZING.bits(), 512);
        assert_eq!(PortStatusFlags::PORT_SPECIFIC_LOCK.bits(), 1024);
        assert_eq!(PortStatusFlags::INVALID.bits(), 2048);
        assert_eq!(PortStatusFlags::HALT.bits(), 4096);
        assert_eq!(PortStatusFlags::CHECK_FD_CLEANUP.bits(), 8192);
    }

    #[test]
    fn test_port_status_flags_is_set() {
        let bits = PortStatusFlags::CONNECTED.bits() 
            | PortStatusFlags::BINARY_IO.bits()
            | PortStatusFlags::EXITING.bits();
        
        assert!(PortStatusFlags::CONNECTED.is_set(bits));
        assert!(PortStatusFlags::BINARY_IO.is_set(bits));
        assert!(PortStatusFlags::EXITING.is_set(bits));
        assert!(!PortStatusFlags::CLOSING.is_set(bits));
        assert!(!PortStatusFlags::LINEBUF_IO.is_set(bits));
        assert!(!PortStatusFlags::DISTRIBUTION.is_set(bits));
    }

    #[test]
    fn test_port_flags_from_state() {
        // Test PortFlags::from_state with binary I/O
        let state = PortState::from_bits(PortStatusFlags::BINARY_IO.bits());
        let flags = PortFlags::from_state(&state);
        assert!(flags.binary_io);
        assert!(!flags.linebuf_io);
        
        // Test PortFlags::from_state with linebuf I/O
        let state = PortState::from_bits(PortStatusFlags::LINEBUF_IO.bits());
        let flags = PortFlags::from_state(&state);
        assert!(!flags.binary_io);
        assert!(flags.linebuf_io);
        
        // Test PortFlags::from_state with both
        let state = PortState::from_bits(
            PortStatusFlags::BINARY_IO.bits() | PortStatusFlags::LINEBUF_IO.bits()
        );
        let flags = PortFlags::from_state(&state);
        assert!(flags.binary_io);
        assert!(flags.linebuf_io);
        
        // Test PortFlags::from_state with no flags
        let state = PortState::default();
        let flags = PortFlags::from_state(&state);
        assert!(!flags.binary_io);
        assert!(!flags.linebuf_io);
    }

    #[test]
    fn test_port_flags_default() {
        let flags = PortFlags::default();
        assert!(!flags.binary_io);
        assert!(!flags.linebuf_io);
        assert!(!flags.soft_eof);
    }

    #[test]
    fn test_port_state_default() {
        let state = PortState::default();
        assert_eq!(state.bits(), 0);
        assert!(!state.is_connected());
        assert!(!state.is_exiting());
        assert!(!state.is_closing());
        assert!(!state.has_binary_io());
        assert!(!state.has_linebuf_io());
    }

    #[test]
    fn test_port_state_bits() {
        let state = PortState::from_bits(0x1234);
        assert_eq!(state.bits(), 0x1234);
        
        let state2 = PortState::from_bits(0x5678);
        assert_eq!(state2.bits(), 0x5678);
        assert_ne!(state.bits(), state2.bits());
    }

    #[test]
    fn test_port_with_name() {
        let mut port = Port::new(300);
        assert!(port.name.is_none());
        
        port.name = Some("test_port".to_string());
        assert_eq!(port.name.as_ref().unwrap(), "test_port");
    }

    #[test]
    fn test_port_bytes_counters() {
        let mut port = Port::new(400);
        assert_eq!(port.bytes_in, 0);
        assert_eq!(port.bytes_out, 0);
        
        port.bytes_in = 1000;
        port.bytes_out = 2000;
        assert_eq!(port.bytes_in, 1000);
        assert_eq!(port.bytes_out, 2000);
    }

    #[test]
    fn test_port_os_pid() {
        let mut port = Port::new(500);
        assert_eq!(port.os_pid, 0);
        
        port.os_pid = 12345;
        assert_eq!(port.os_pid, 12345);
    }

    #[test]
    fn test_port_driver_data() {
        let mut port = Port::new(600);
        assert_eq!(port.drv_data, 0);
        
        port.drv_data = 0xDEADBEEF;
        assert_eq!(port.drv_data, 0xDEADBEEF);
    }

    #[test]
    fn test_port_control_flags() {
        let mut port = Port::new(700);
        assert_eq!(port.control_flags, 0);
        
        port.control_flags = 42;
        assert_eq!(port.control_flags, 42);
    }

    #[test]
    fn test_port_reds() {
        let mut port = Port::new(800);
        assert_eq!(port.reds, 0);
        
        port.reds = 100;
        assert_eq!(port.reds, 100);
    }

    #[test]
    fn test_port_caller() {
        let mut port = Port::new(900);
        assert_eq!(port.caller, 0);
        
        port.caller = 0x12345678;
        assert_eq!(port.caller, 0x12345678);
    }

    #[test]
    fn test_port_psd() {
        let port = Port::new(1000);
        assert_eq!(port.psd.load(Ordering::Acquire), 0);
        
        port.psd.store(0xABCDEF00, Ordering::Release);
        assert_eq!(port.psd.load(Ordering::Acquire), 0xABCDEF00);
    }

    #[test]
    fn test_port_run_queue() {
        let port = Port::new(1100);
        assert_eq!(port.run_queue.load(Ordering::Acquire), 0);
        
        port.run_queue.store(0x1234567890ABCDEF, Ordering::Release);
        assert_eq!(port.run_queue.load(Ordering::Acquire), 0x1234567890ABCDEF);
    }

    #[test]
    fn test_async_open_port() {
        let async_port = AsyncOpenPort {
            to: 0x1234,
            ref_array: [1, 2, 3],
        };
        
        assert_eq!(async_port.to, 0x1234);
        assert_eq!(async_port.ref_array, [1, 2, 3]);
        
        // Test clone
        let cloned = async_port.clone();
        assert_eq!(cloned.to, async_port.to);
        assert_eq!(cloned.ref_array, async_port.ref_array);
    }

    #[test]
    fn test_port_with_async_open_port() {
        let mut port = Port::new(1200);
        assert!(port.async_open_port.is_none());
        
        let async_port = AsyncOpenPort {
            to: 0x5678,
            ref_array: [10, 20, 30],
        };
        port.async_open_port = Some(async_port);
        
        assert!(port.async_open_port.is_some());
        let ref_port = port.async_open_port.as_ref().unwrap();
        assert_eq!(ref_port.to, 0x5678);
        assert_eq!(ref_port.ref_array, [10, 20, 30]);
    }
}

