//! Adapters Layer: Drivers
//!
//! Provides driver implementations for the Erlang/OTP runtime system. This crate
//! implements adapters for various I/O drivers, including network and file system
//! drivers.
//!
//! ## Overview
//!
//! The `adapters_drivers` crate is part of the adapters layer in the CLEAN
//! architecture implementation of Erlang/OTP. It provides I/O adapters for various
//! driver implementations that interface with the operating system.
//!
//! ## Modules
//!
//! - **[`inet`](inet/index.html)**: INET driver for network operations, providing
//!   TCP/UDP socket functionality
//!
//! - **[`ram_file`](ram_file/index.html)**: RAM file driver for in-memory file
//!   operations, useful for testing and temporary storage
//!
//! ## Architecture
//!
//! This crate is based on the C implementation in `inet_drv.c` and `ram_file_drv.c`.
//! It depends on the Entities and Use Cases layers for fundamental operations.
//!
//! ## See Also
//!
//! - [`frameworks_system_integration`](../../frameworks/frameworks_system_integration/index.html): System integration framework

pub mod inet;
pub mod ram_file;

pub use inet::InetDriver;
pub use ram_file::RamFileDriver;

