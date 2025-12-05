//! Adapters Layer: ETS Tables
//!
//! Provides socket debugging functionality for the Erlang/OTP runtime system.
//! This crate implements adapters for debug output initialization and formatted
//! debug printing with timestamps and thread names.
//!
//! ## Overview
//!
//! The `adapters_ets_tables` crate is part of the adapters layer in the CLEAN
//! architecture implementation of Erlang/OTP. It provides I/O adapters for socket
//! debugging operations, including debug output file management and formatted
//! debug message printing.
//!
//! ## Modules
//!
//! - **[`ets_debug`](ets_debug/index.html)**: Socket debugging functionality for
//!   initializing debug output and printing formatted debug messages with timestamps
//!   and thread names
//!
//! ## Architecture
//!
//! This crate is based on the C implementation in `socket_dbg.c`. It depends on the
//! Entities layer for fundamental data types.
//!
//! ## Functions
//!
//! - `esock_dbg_init`: Initialize debug output to a file or stdout
//! - `esock_dbg_printf`: Print formatted debug messages with timestamp and thread name
//!
//! ## See Also
//!
//! - [`infrastructure_ets_tables`](../../infrastructure/infrastructure_ets_tables/index.html): ETS table infrastructure
//! - [`entities_data_handling`](../../entities/entities_data_handling/index.html): Term types

pub mod ets_debug;

pub use ets_debug::{SocketDebug, esock_dbg_init, esock_dbg_printf};

