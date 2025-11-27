//! Port BIF Module
//!
//! Provides built-in functions for port operations.
//! Based on erl_bif_port.c
//!
//! This module implements the Use Cases layer for I/O operations,
//! specifically port-related built-in functions (BIFs) that are called
//! from Erlang code. These functions handle:
//! - Opening ports
//! - Sending commands to ports
//! - Making calls to ports
//! - Port control operations
//! - Port information retrieval
//! - Port data management
//! - Packet decoding

// Entities layer dependencies
// Note: Process and Port types are now available from entities_process_port
// The actual implementation of port operations requires integration with
// the VM's port management system, which will be implemented in higher layers.
use entities_process_port::{Port, Process, PortId};

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

/// Port operation result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortOpResult {
    /// Operation completed successfully
    Done,
    /// Operation scheduled (async)
    Scheduled,
    /// Bad argument
    BadArg,
    /// Operation dropped
    Dropped,
    /// Port is busy
    Busy,
    /// Port is busy and scheduled
    BusyScheduled,
    /// Not supported
    NotSupported,
}

/// Port BIF operations
pub struct PortBif;

impl PortBif {
    /// Open a port (erts_internal_open_port_2)
    ///
    /// Opens a port with the given name and settings.
    /// This corresponds to the Erlang BIF `erlang:open_port/2`.
    ///
    /// # Arguments
    /// * `process` - The process opening the port (Process type from entities_process_port)
    /// * `name` - Port name (tuple with spawn/fd information)
    /// * `settings` - Port settings (list of options)
    ///
    /// # Returns
    /// Port ID or error
    ///
    /// # Errors
    /// Returns `PortError` if:
    /// - Invalid settings arguments
    /// - Bad argument format
    /// - System limit reached
    /// - Port cannot be opened
    ///
    /// # Note
    /// This function requires integration with the VM's port management system
    /// (Infrastructure/Adapters layers) to actually create and register the port.
    /// The Process and Port types are now available from entities_process_port.
    pub fn open_port_2(
        _process: &Process,
        _name: &PortName,
        _settings: &PortSettings,
    ) -> Result<PortId, PortError> {
        // TODO: Implement port opening logic
        // Requires: VM port table management, driver loading, process linking
        // These will be implemented in Infrastructure/Adapters layers
        Err(PortError::NotImplemented)
    }

    /// Send a command to a port (erts_internal_port_command_3)
    ///
    /// Sends a command to a port with optional flags.
    /// This corresponds to the Erlang BIF `erlang:port_command/3`.
    ///
    /// # Arguments
    /// * `process` - The process sending the command (Process type from entities_process_port)
    /// * `port` - Port structure (Port type from entities_process_port)
    /// * `data` - Data to send
    /// * `flags` - Optional flags (force, nosuspend)
    ///
    /// # Returns
    /// Operation result
    ///
    /// # Note
    /// This function requires integration with the VM's signal queue system
    /// to actually send the command. The Process and Port types are now available.
    pub fn port_command_3(
        _process: &Process,
        _port: &Port,
        _data: &[u8],
        _flags: Option<PortCommandFlags>,
    ) -> Result<PortOpResult, PortError> {
        // TODO: Implement port command logic
        // Requires: Signal queue management, port locking, driver callbacks
        // These will be implemented in Infrastructure/Adapters layers
        Err(PortError::NotImplemented)
    }

    /// Make a call to a port (erts_internal_port_call_3)
    ///
    /// Makes a synchronous call to a port.
    /// This corresponds to the Erlang BIF `erlang:port_call/3`.
    ///
    /// # Arguments
    /// * `process_id` - The process making the call
    /// * `port_id` - Port ID or name
    /// * `operation` - Operation code
    /// * `data` - Data for the operation
    ///
    /// # Returns
    /// Result value or error
    pub fn port_call_3(
        _process_id: u64,
        _port_id: PortIdentifier,
        _operation: u32,
        _data: &[u8],
    ) -> Result<PortCallResult, PortError> {
        // TODO: Implement port call logic
        Err(PortError::NotImplemented)
    }

    /// Control a port (erts_internal_port_control_3)
    ///
    /// Sends a control message to a port.
    /// This corresponds to the Erlang BIF `erlang:port_control/3`.
    ///
    /// # Arguments
    /// * `process_id` - The process controlling the port
    /// * `port_id` - Port ID or name
    /// * `operation` - Operation code
    /// * `data` - Data for the operation
    ///
    /// # Returns
    /// Result value or error
    pub fn port_control_3(
        _process_id: u64,
        _port_id: PortIdentifier,
        _operation: u32,
        _data: &[u8],
    ) -> Result<PortCallResult, PortError> {
        // TODO: Implement port control logic
        Err(PortError::NotImplemented)
    }

    /// Close a port (erts_internal_port_close_1)
    ///
    /// Closes a port.
    /// This corresponds to the Erlang BIF `erlang:port_close/1`.
    ///
    /// # Arguments
    /// * `process_id` - The process closing the port
    /// * `port_id` - Port ID or name
    ///
    /// # Returns
    /// Operation result
    pub fn port_close_1(
        _process_id: u64,
        _port_id: PortIdentifier,
    ) -> Result<PortOpResult, PortError> {
        // TODO: Implement port close logic
        Err(PortError::NotImplemented)
    }

    /// Connect a process to a port (erts_internal_port_connect_2)
    ///
    /// Connects a process to a port.
    /// This corresponds to the Erlang BIF `erlang:port_connect/2`.
    ///
    /// # Arguments
    /// * `process_id` - The process connecting
    /// * `port_id` - Port ID or name
    /// * `target_process_id` - Process ID to connect
    ///
    /// # Returns
    /// Operation result
    pub fn port_connect_2(
        _process_id: u64,
        _port_id: PortIdentifier,
        _target_process_id: u64,
    ) -> Result<PortOpResult, PortError> {
        // TODO: Implement port connect logic
        Err(PortError::NotImplemented)
    }

    /// Get port information (erts_internal_port_info_1)
    ///
    /// Gets information about a port.
    /// This corresponds to the Erlang BIF `erlang:port_info/1`.
    ///
    /// # Arguments
    /// * `process_id` - The process requesting info
    /// * `port_id` - Port ID or name
    ///
    /// # Returns
    /// Port information or error
    pub fn port_info_1(
        _process_id: u64,
        _port_id: PortIdentifier,
    ) -> Result<PortInfo, PortError> {
        // TODO: Implement port info logic
        Err(PortError::NotImplemented)
    }

    /// Get specific port information (erts_internal_port_info_2)
    ///
    /// Gets specific information about a port.
    /// This corresponds to the Erlang BIF `erlang:port_info/2`.
    ///
    /// # Arguments
    /// * `process_id` - The process requesting info
    /// * `port_id` - Port ID or name
    /// * `item` - Information item to retrieve
    ///
    /// # Returns
    /// Port information item or error
    pub fn port_info_2(
        _process_id: u64,
        _port_id: PortIdentifier,
        _item: PortInfoItem,
    ) -> Result<PortInfoValue, PortError> {
        // TODO: Implement port info logic
        Err(PortError::NotImplemented)
    }

    /// Set port data (port_set_data_2)
    ///
    /// Sets data associated with a port.
    /// This corresponds to the Erlang BIF `erlang:port_set_data/2`.
    ///
    /// # Arguments
    /// * `process_id` - The process setting data
    /// * `port_id` - Port ID
    /// * `data` - Data to set
    ///
    /// # Returns
    /// Success or error
    pub fn port_set_data_2(
        _process_id: u64,
        _port_id: u64,
        _data: PortData,
    ) -> Result<(), PortError> {
        // TODO: Implement port data set logic
        Err(PortError::NotImplemented)
    }

    /// Get port data (port_get_data_1)
    ///
    /// Gets data associated with a port.
    /// This corresponds to the Erlang BIF `erlang:port_get_data/1`.
    ///
    /// # Arguments
    /// * `process_id` - The process getting data
    /// * `port_id` - Port ID
    ///
    /// # Returns
    /// Port data or error
    pub fn port_get_data_1(
        _process_id: u64,
        _port_id: u64,
    ) -> Result<PortData, PortError> {
        // TODO: Implement port data get logic
        Err(PortError::NotImplemented)
    }

    /// Read port data (erts_port_data_read)
    ///
    /// Reads data associated with a port (internal function).
    ///
    /// # Arguments
    /// * `port_id` - Port ID
    ///
    /// # Returns
    /// Port data or undefined
    pub fn port_data_read(_port_id: u64) -> Option<PortData> {
        // TODO: Implement port data read logic
        None
    }

    /// Decode packet (decode_packet_3)
    ///
    /// Decodes a packet according to the specified type.
    /// This corresponds to the Erlang BIF `erlang:decode_packet/3`.
    ///
    /// # Arguments
    /// * `process_id` - The process decoding the packet
    /// * `packet_type` - Type of packet to decode
    /// * `data` - Binary data to decode
    /// * `options` - Decoding options
    ///
    /// # Returns
    /// Decoded packet result
    pub fn decode_packet_3(
        _process_id: u64,
        _packet_type: PacketType,
        _data: &[u8],
        _options: &PacketOptions,
    ) -> Result<PacketDecodeResult, PortError> {
        // TODO: Implement packet decoding logic
        Err(PortError::NotImplemented)
    }
}

/// Port identifier (ID or name)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortIdentifier {
    /// Port ID
    Id(u64),
    /// Port name (atom)
    Name(String),
}

/// Port name specification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortName {
    /// Spawn a process port
    Spawn { command: String },
    /// Spawn a driver port
    SpawnDriver { driver: String },
    /// Spawn an executable port
    SpawnExecutable { executable: String },
    /// File descriptor port
    Fd { input_fd: u32, output_fd: u32 },
}

/// Port settings
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortSettings {
    /// Packet size (1, 2, 4, or 0 for stream)
    pub packet_bytes: u8,
    /// Line buffer size
    pub line_buffer: Option<u32>,
    /// Environment variables
    pub environment: Vec<(String, String)>,
    /// Command arguments
    pub arguments: Vec<String>,
    /// Argument 0 (program name)
    pub arg0: Option<String>,
    /// Working directory
    pub working_directory: Option<String>,
    /// Use stdio
    pub use_stdio: bool,
    /// Redirect stderr to stdout
    pub redirect_stderr: bool,
    /// Binary I/O
    pub binary_io: bool,
    /// Read mode
    pub read: bool,
    /// Write mode
    pub write: bool,
    /// EOF handling
    pub eof: bool,
    /// Hide window (Windows)
    pub hide_window: bool,
    /// Exit status
    pub exit_status: bool,
    /// Overlapped I/O (Windows)
    pub overlapped_io: bool,
    /// Parallelism
    pub parallelism: bool,
    /// Port busy limits
    pub port_busy_limits: Option<(u32, u32)>,
    /// Message queue busy limits
    pub msgq_busy_limits: Option<(u32, u32)>,
}

impl Default for PortSettings {
    fn default() -> Self {
        Self {
            packet_bytes: 0,
            line_buffer: None,
            environment: Vec::new(),
            arguments: Vec::new(),
            arg0: None,
            working_directory: None,
            use_stdio: true,
            redirect_stderr: false,
            binary_io: false,
            read: false,
            write: false,
            eof: false,
            hide_window: false,
            exit_status: false,
            overlapped_io: false,
            parallelism: false,
            port_busy_limits: None,
            msgq_busy_limits: None,
        }
    }
}

/// Port command flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortCommandFlags {
    /// Force command even if port is busy
    pub force: bool,
    /// Don't suspend if port is busy
    pub nosuspend: bool,
}

/// Port call result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortCallResult {
    /// Call completed with result
    Done(Vec<u8>),
    /// Call scheduled (async)
    Scheduled(u64), // Reference
}

/// Port information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortInfo {
    /// Port ID
    pub id: u64,
    /// Port name
    pub name: Option<String>,
    /// Connected processes
    pub connected: Vec<u64>,
    /// Links
    pub links: Vec<u64>,
    /// Port flags
    pub flags: PortFlags,
}

/// Port information item
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortInfoItem {
    /// Port ID
    Id,
    /// Port name
    Name,
    /// Connected processes
    Connected,
    /// Links
    Links,
    /// Input count
    Input,
    /// Output count
    Output,
    /// Queue size
    QueueSize,
    /// Queue data
    QueueData,
    /// Exit status
    ExitStatus,
}

/// Port information value
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortInfoValue {
    /// Integer value
    Integer(u64),
    /// List value
    List(Vec<u64>),
    /// Binary value
    Binary(Vec<u8>),
    /// Atom value
    Atom(String),
    /// Undefined
    Undefined,
}

/// Port flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortFlags {
    /// Binary I/O
    pub binary_io: bool,
    /// Line buffer I/O
    pub linebuf_io: bool,
    /// Soft EOF
    pub soft_eof: bool,
}

/// Port data
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortData {
    /// Immediate value (small integer, atom, etc.)
    Immediate(u64),
    /// Heap data
    Heap(Vec<u8>),
    /// Undefined
    Undefined,
}

/// Packet type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketType {
    /// Raw (no packet)
    Raw,
    /// 1-byte length prefix
    One,
    /// 2-byte length prefix
    Two,
    /// 4-byte length prefix
    Four,
    /// ASN.1
    Asn1,
    /// Sun RPC
    SunRm,
    /// CDR
    Cdr,
    /// FastCGI
    Fcgi,
    /// Line (LF delimited)
    Line,
    /// TPKT
    Tpkt,
    /// HTTP
    Http,
    /// HTTP header
    HttpH,
    /// HTTP binary
    HttpBin,
    /// HTTP header binary
    HttpHBin,
    /// SSL/TLS
    SslTls,
}

/// Packet options
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketOptions {
    /// Maximum packet length (0 = no limit)
    pub max_packet_length: Option<u32>,
    /// Line truncation length (0 = no limit)
    pub line_length: Option<u32>,
    /// Line delimiter
    pub line_delimiter: Option<u8>,
}

impl Default for PacketOptions {
    fn default() -> Self {
        Self {
            max_packet_length: None,
            line_length: None,
            line_delimiter: None,
        }
    }
}

/// Packet decode result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PacketDecodeResult {
    /// Successfully decoded packet
    Ok {
        /// Packet body
        body: Vec<u8>,
        /// Remaining data
        rest: Vec<u8>,
    },
    /// Need more data
    More {
        /// Required size (if known)
        size: Option<u32>,
    },
    /// Error
    Error {
        /// Error reason
        reason: String,
    },
}

/// Port operation errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortError {
    /// Operation not implemented
    NotImplemented,
    /// Invalid argument
    InvalidArgument,
    /// System limit reached
    SystemLimit,
    /// Bad option
    BadOption,
    /// Port not found
    PortNotFound,
    /// Port already closed
    PortClosed,
    /// Operation not supported
    NotSupported,
    /// Invalid packet type
    InvalidPacketType,
    /// Invalid packet data
    InvalidPacketData,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_settings_default() {
        let settings = PortSettings::default();
        assert_eq!(settings.packet_bytes, 0);
        assert_eq!(settings.use_stdio, true);
        assert_eq!(settings.binary_io, false);
    }

    #[test]
    fn test_port_identifier() {
        let id = PortIdentifier::Id(123);
        let name = PortIdentifier::Name("test_port".to_string());
        
        assert_eq!(id, PortIdentifier::Id(123));
        assert_eq!(name, PortIdentifier::Name("test_port".to_string()));
    }

    #[test]
    fn test_port_name() {
        let spawn = PortName::Spawn {
            command: "ls".to_string(),
        };
        let fd = PortName::Fd {
            input_fd: 0,
            output_fd: 1,
        };
        
        match spawn {
            PortName::Spawn { command } => assert_eq!(command, "ls"),
            _ => panic!("Expected Spawn"),
        }
        
        match fd {
            PortName::Fd { input_fd, output_fd } => {
                assert_eq!(input_fd, 0);
                assert_eq!(output_fd, 1);
            }
            _ => panic!("Expected Fd"),
        }
    }

    #[test]
    fn test_packet_options_default() {
        let options = PacketOptions::default();
        assert_eq!(options.max_packet_length, None);
        assert_eq!(options.line_length, None);
        assert_eq!(options.line_delimiter, None);
    }

    #[test]
    fn test_port_error() {
        let err = PortError::InvalidArgument;
        assert_eq!(err, PortError::InvalidArgument);
    }
}
