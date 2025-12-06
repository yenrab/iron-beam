//! Adapters Layer: Socket and TCP/IP Networking
//!
//! Provides socket and TCP/IP networking functionality for the Erlang/OTP runtime system.
//! This crate implements socket operations using Rust's standard library and the `socket2`
//! crate for safe, cross-platform socket operations.
//!
//! ## Overview
//!
//! The `adapters_socket` crate provides:
//! - **TCP sockets**: Stream-based reliable communication
//! - **UDP sockets**: Datagram-based communication
//! - **Socket operations**: bind, listen, accept, connect, send, recv
//! - **Integration with NIF I/O**: Uses `adapters_nif_io` for I/O polling
//!
//! ## Architecture
//!
//! This crate is part of the adapters layer in the CLEAN architecture implementation.
//! It depends on:
//! - `adapters_nif_io`: For I/O polling and event management
//! - `adapters_nifs`: For NIF infrastructure
//! - `entities_data_handling`: For Erlang term representation
//!
//! ## See Also
//!
//! - [`adapters_nif_io`](../adapters_nif_io/index.html): I/O polling infrastructure
//! - [`adapters_nifs`](../adapters_nifs/index.html): NIF implementations

pub mod socket;
pub mod tcp;
pub mod udp;

pub use socket::{Socket, SocketError, SocketType, AddressFamily, Protocol};
pub use tcp::TcpSocket;
pub use udp::UdpSocket;
