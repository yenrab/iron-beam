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
use entities_process_port::{Port, Process, PortId, PortState, PortStatusFlags};

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
    /// * `process` - The process making the call
    /// * `port` - Port structure
    /// * `operation` - Operation code
    /// * `data` - Data for the operation
    ///
    /// # Returns
    /// Result value or error
    ///
    /// # Note
    /// This function requires integration with the VM's port driver system
    /// to actually make the call and wait for the response. The Process and
    /// Port types are now available.
    pub fn port_call_3(
        _process: &Process,
        _port: &Port,
        _operation: u32,
        _data: &[u8],
    ) -> Result<PortCallResult, PortError> {
        // TODO: Implement port call logic
        // Requires: Driver callback invocation, synchronous waiting for response
        // These will be implemented in Infrastructure/Adapters layers
        Err(PortError::NotImplemented)
    }

    /// Control a port (erts_internal_port_control_3)
    ///
    /// Sends a control message to a port.
    /// This corresponds to the Erlang BIF `erlang:port_control/3`.
    ///
    /// # Arguments
    /// * `process` - The process controlling the port
    /// * `port` - Port structure
    /// * `operation` - Operation code
    /// * `data` - Data for the operation
    ///
    /// # Returns
    /// Result value or error
    ///
    /// # Note
    /// This function requires integration with the VM's port driver system
    /// to actually send the control message. The Process and Port types are now available.
    pub fn port_control_3(
        _process: &Process,
        _port: &Port,
        _operation: u32,
        _data: &[u8],
    ) -> Result<PortCallResult, PortError> {
        // TODO: Implement port control logic
        // Requires: Driver control callback invocation
        // These will be implemented in Infrastructure/Adapters layers
        Err(PortError::NotImplemented)
    }

    /// Close a port (erts_internal_port_close_1)
    ///
    /// Closes a port.
    /// This corresponds to the Erlang BIF `erlang:port_close/1`.
    ///
    /// # Arguments
    /// * `process` - The process closing the port
    /// * `port` - Port structure
    ///
    /// # Returns
    /// Operation result
    ///
    /// # Note
    /// This function requires integration with the VM's port management system
    /// to actually close and clean up the port. The Process and Port types are now available.
    pub fn port_close_1(
        _process: &Process,
        port: &Port,
    ) -> Result<PortOpResult, PortError> {
        // Check if port is already closed
        let state = port.get_state();
        if state.is_closing() || state.is_exiting() {
            return Ok(PortOpResult::Done);
        }

        // TODO: Implement port close logic
        // Requires: Signal queue to send close signal, driver cleanup, port table removal
        // These will be implemented in Infrastructure/Adapters layers
        
        // For now, mark port as closing
        let new_state = PortState::from_bits(
            state.bits() | PortStatusFlags::CLOSING.bits()
        );
        port.set_state(new_state);
        
        Ok(PortOpResult::Done)
    }

    /// Connect a process to a port (erts_internal_port_connect_2)
    ///
    /// Connects a process to a port.
    /// This corresponds to the Erlang BIF `erlang:port_connect/2`.
    ///
    /// # Arguments
    /// * `process` - The process connecting
    /// * `port` - Port structure
    /// * `target_process` - Target process to connect
    ///
    /// # Returns
    /// Operation result
    ///
    /// # Note
    /// This function uses the Port's connected field to store the connected process.
    /// The Process and Port types are now available.
    pub fn port_connect_2(
        _process: &Process,
        port: &Port,
        target_process: &Process,
    ) -> Result<PortOpResult, PortError> {
        let target_pid = target_process.get_id();
        
        // Set the connected process atomically
        port.set_connected(target_pid);
        
        // Update port state to mark as connected
        let state = port.get_state();
        let new_state = PortState::from_bits(
            state.bits() | PortStatusFlags::CONNECTED.bits()
        );
        port.set_state(new_state);
        
        Ok(PortOpResult::Done)
    }

    /// Get port information (erts_internal_port_info_1)
    ///
    /// Gets information about a port.
    /// This corresponds to the Erlang BIF `erlang:port_info/1`.
    ///
    /// # Arguments
    /// * `process` - The process requesting info
    /// * `port` - Port structure
    ///
    /// # Returns
    /// Port information or error
    ///
    /// # Note
    /// This function extracts information from the Port structure.
    /// Some information (like links) requires VM integration to get the full list.
    pub fn port_info_1(
        _process: &Process,
        port: &Port,
    ) -> Result<PortInfo, PortError> {
        let port_id = port.get_id();
        let connected_pid = port.get_connected();
        
        // Build connected processes list
        let mut connected = Vec::new();
        if connected_pid != 0 {
            connected.push(connected_pid);
        }

        // Extract port flags from state
        let state = port.get_state();
        let flags = PortFlags {
            binary_io: state.has_binary_io(),
            linebuf_io: state.has_linebuf_io(),
            soft_eof: false, // Would need to check SOFT_EOF flag
        };

        // Links would require VM integration to get the full list
        // For now, return empty list
        let links = Vec::new();

        Ok(PortInfo {
            id: port_id,
            name: port.name.clone(),
            connected,
            links,
            flags,
        })
    }

    /// Get specific port information (erts_internal_port_info_2)
    ///
    /// Gets specific information about a port.
    /// This corresponds to the Erlang BIF `erlang:port_info/2`.
    ///
    /// # Arguments
    /// * `process` - The process requesting info
    /// * `port` - Port structure
    /// * `item` - Information item to retrieve
    ///
    /// # Returns
    /// Port information item or error
    pub fn port_info_2(
        _process: &Process,
        port: &Port,
        item: PortInfoItem,
    ) -> Result<PortInfoValue, PortError> {
        match item {
            PortInfoItem::Id => {
                Ok(PortInfoValue::Integer(port.get_id()))
            }
            PortInfoItem::Name => {
                match &port.name {
                    Some(name) => Ok(PortInfoValue::Atom(name.clone())),
                    None => Ok(PortInfoValue::Undefined),
                }
            }
            PortInfoItem::Connected => {
                let connected_pid = port.get_connected();
                if connected_pid != 0 {
                    Ok(PortInfoValue::List(vec![connected_pid]))
                } else {
                    Ok(PortInfoValue::List(Vec::new()))
                }
            }
            PortInfoItem::Links => {
                // Links require VM integration - return empty for now
                Ok(PortInfoValue::List(Vec::new()))
            }
            PortInfoItem::Input => {
                Ok(PortInfoValue::Integer(port.bytes_in))
            }
            PortInfoItem::Output => {
                Ok(PortInfoValue::Integer(port.bytes_out))
            }
            PortInfoItem::QueueSize => {
                // Queue size requires VM integration - return undefined for now
                Ok(PortInfoValue::Undefined)
            }
            PortInfoItem::QueueData => {
                // Queue data requires VM integration - return undefined for now
                Ok(PortInfoValue::Undefined)
            }
            PortInfoItem::ExitStatus => {
                // Exit status requires VM integration - return undefined for now
                Ok(PortInfoValue::Undefined)
            }
        }
    }

    /// Set port data (port_set_data_2)
    ///
    /// Sets data associated with a port.
    /// This corresponds to the Erlang BIF `erlang:port_set_data/2`.
    ///
    /// # Arguments
    /// * `process` - The process setting data
    /// * `port` - Port structure
    /// * `data` - Data to set
    ///
    /// # Returns
    /// Success or error
    ///
    /// # Note
    /// This function uses the Port's atomic data field. The data can be
    /// either an immediate value (small integer/atom) or heap data.
    /// In the C code, heap data is stored as a pointer to ErtsPortDataHeap.
    /// For now, we store immediate values directly and heap data as a pointer value.
    pub fn port_set_data_2(
        _process: &Process,
        port: &Port,
        data: PortData,
    ) -> Result<(), PortError> {
        // Convert PortData to u64 for storage
        let data_value = match data {
            PortData::Immediate(val) => {
                // Immediate values must have tag bits set (low 2 bits != 0)
                if val & 0x3 == 0 {
                    return Err(PortError::InvalidArgument);
                }
                val
            }
            PortData::Heap(heap_data) => {
                // For heap data, we would normally allocate and store a pointer
                // For now, we'll use a simple encoding: store length in upper bits
                // This is a placeholder - full implementation requires heap management
                if heap_data.len() > (u64::MAX as usize >> 16) {
                    return Err(PortError::SystemLimit);
                }
                // Encode: upper 16 bits = length, lower 48 bits = hash of data
                // This is a simplified representation
                let len = heap_data.len() as u64;
                let hash = heap_data.iter().fold(0u64, |acc, &b| acc.wrapping_mul(31).wrapping_add(b as u64));
                (len << 48) | (hash & 0x0000FFFFFFFFFFFF)
            }
            PortData::Undefined => {
                // Setting to undefined means clearing the data
                0
            }
        };

        // Exchange the old data with the new data atomically
        let _old_data = port.data.swap(data_value, std::sync::atomic::Ordering::AcqRel);
        
        // If old_data was 0, the port might have been terminated
        // In the C code, this would trigger cleanup_old_port_data
        // For now, we just return success
        Ok(())
    }

    /// Get port data (port_get_data_1)
    ///
    /// Gets data associated with a port.
    /// This corresponds to the Erlang BIF `erlang:port_get_data/1`.
    ///
    /// # Arguments
    /// * `process` - The process getting data
    /// * `port` - Port structure
    ///
    /// # Returns
    /// Port data or error
    ///
    /// # Note
    /// This function reads the Port's atomic data field. If the data is
    /// an immediate value (tagged), it returns it directly. If it's heap
    /// data, it would need to dereference the pointer (requires heap management).
    pub fn port_get_data_1(
        _process: &Process,
        port: &Port,
    ) -> Result<PortData, PortError> {
        let data = port.get_data();
        
        if data == 0 {
            // Port data is undefined or port was terminated
            return Err(PortError::PortNotFound);
        }

        // Check if it's an immediate value (tagged - low 2 bits != 0)
        if data & 0x3 != 0 {
            Ok(PortData::Immediate(data))
        } else {
            // It's heap data - decode our simplified representation
            // In full implementation, this would dereference the pointer
            let len = (data >> 48) as usize;
            if len == 0 {
                Ok(PortData::Undefined)
            } else {
                // For now, return empty heap data as placeholder
                // Full implementation would read from heap
                Ok(PortData::Heap(Vec::new()))
            }
        }
    }

    /// Read port data (erts_port_data_read)
    ///
    /// Reads data associated with a port (internal function).
    /// Similar to port_get_data_1 but returns Option instead of Result.
    ///
    /// # Arguments
    /// * `port` - Port structure
    ///
    /// # Returns
    /// Port data or None if undefined/not found
    pub fn port_data_read(port: &Port) -> Option<PortData> {
        let data = port.get_data();
        
        if data == 0 {
            return None;
        }

        // Check if it's an immediate value (tagged - low 2 bits != 0)
        if data & 0x3 != 0 {
            Some(PortData::Immediate(data))
        } else {
            // It's heap data
            let len = (data >> 48) as usize;
            if len == 0 {
                None
            } else {
                // For now, return empty heap data as placeholder
                Some(PortData::Heap(Vec::new()))
            }
        }
    }

    /// Decode packet (decode_packet_3)
    ///
    /// Decodes a packet according to the specified type.
    /// This corresponds to the Erlang BIF `erlang:decode_packet/3`.
    ///
    /// # Arguments
    /// * `process` - The process decoding the packet
    /// * `packet_type` - Type of packet to decode
    /// * `data` - Binary data to decode
    /// * `options` - Decoding options
    ///
    /// # Returns
    /// Decoded packet result
    ///
    /// # Note
    /// This implements the core packet decoding logic. HTTP/SSL parsing
    /// requires more complex parsing that would be in Infrastructure layer.
    pub fn decode_packet_3(
        _process: &Process,
        packet_type: PacketType,
        data: &[u8],
        options: &PacketOptions,
    ) -> Result<PacketDecodeResult, PortError> {
        let max_packet_length = options.max_packet_length.unwrap_or(0);
        let line_length = options.line_length.unwrap_or(0);
        let delimiter = options.line_delimiter.unwrap_or(b'\n');

        // Get packet length based on type
        let packet_length = match packet_type {
            PacketType::Raw => {
                // Raw: return all available data
                if data.is_empty() {
                    return Ok(PacketDecodeResult::More { size: None });
                }
                data.len()
            }
            PacketType::One => {
                // 1-byte length prefix
                if data.len() < 1 {
                    return Ok(PacketDecodeResult::More { size: Some(1) });
                }
                let len = data[0] as usize;
                if len == 0 {
                    return Err(PortError::InvalidPacketData);
                }
                let total_len = 1 + len; // header + data
                if data.len() < total_len {
                    return Ok(PacketDecodeResult::More { size: Some(total_len as u32) });
                }
                if max_packet_length > 0 && total_len > max_packet_length as usize {
                    return Err(PortError::InvalidPacketData);
                }
                total_len
            }
            PacketType::Two => {
                // 2-byte length prefix (big-endian)
                if data.len() < 2 {
                    return Ok(PacketDecodeResult::More { size: Some(2) });
                }
                let len = ((data[0] as usize) << 8) | (data[1] as usize);
                if len == 0 {
                    return Err(PortError::InvalidPacketData);
                }
                let total_len = 2 + len;
                if data.len() < total_len {
                    return Ok(PacketDecodeResult::More { size: Some(total_len as u32) });
                }
                if max_packet_length > 0 && total_len > max_packet_length as usize {
                    return Err(PortError::InvalidPacketData);
                }
                total_len
            }
            PacketType::Four => {
                // 4-byte length prefix (big-endian)
                if data.len() < 4 {
                    return Ok(PacketDecodeResult::More { size: Some(4) });
                }
                let len = ((data[0] as usize) << 24)
                    | ((data[1] as usize) << 16)
                    | ((data[2] as usize) << 8)
                    | (data[3] as usize);
                if len == 0 {
                    return Err(PortError::InvalidPacketData);
                }
                let total_len = 4 + len;
                if data.len() < total_len {
                    return Ok(PacketDecodeResult::More { size: Some(total_len as u32) });
                }
                if max_packet_length > 0 && total_len > max_packet_length as usize {
                    return Err(PortError::InvalidPacketData);
                }
                total_len
            }
            PacketType::Line => {
                // Line-delimited (LF by default)
                if data.is_empty() {
                    return Ok(PacketDecodeResult::More { size: None });
                }
                // Find delimiter
                match data.iter().position(|&b| b == delimiter) {
                    Some(pos) => {
                        let len = pos + 1; // Include delimiter
                        if max_packet_length > 0 && len > max_packet_length as usize {
                            return Err(PortError::InvalidPacketData);
                        }
                        if line_length > 0 && len > line_length as usize {
                            // Truncate to line_length
                            line_length as usize
                        } else {
                            len
                        }
                    }
                    None => {
                        // No delimiter found
                        if max_packet_length > 0 && data.len() >= max_packet_length as usize {
                            return Err(PortError::InvalidPacketData);
                        }
                        if line_length > 0 && data.len() >= line_length as usize {
                            // Buffer full, return truncated
                            return Ok(PacketDecodeResult::Ok {
                                body: data[..line_length as usize].to_vec(),
                                rest: data[line_length as usize..].to_vec(),
                            });
                        }
                        return Ok(PacketDecodeResult::More { size: None });
                    }
                }
            }
            PacketType::Asn1 | PacketType::SunRm | PacketType::Cdr
            | PacketType::Fcgi | PacketType::Tpkt | PacketType::Http
            | PacketType::HttpH | PacketType::HttpBin | PacketType::HttpHBin
            | PacketType::SslTls => {
                // Complex packet types require more sophisticated parsing
                // For now, return error indicating not fully implemented
                // These would be implemented in Infrastructure layer
                return Err(PortError::NotSupported);
            }
        };

        // Extract packet body and remaining data
        let body_start = match packet_type {
            PacketType::Raw => 0,
            PacketType::One => 1,
            PacketType::Two => 2,
            PacketType::Four => 4,
            PacketType::Line => 0, // For line, body includes the delimiter
            _ => unreachable!(),
        };

        let body = if packet_type == PacketType::Line {
            // For line, body is everything up to and including delimiter
            data[..packet_length].to_vec()
        } else {
            // For length-prefixed, body starts after header
            data[body_start..packet_length].to_vec()
        };

        let rest = data[packet_length..].to_vec();

        Ok(PacketDecodeResult::Ok { body, rest })
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
