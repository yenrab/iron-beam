//! Adapters Layer: Unix System Integration
//!
//! Provides Unix-specific system integration functionality for the Erlang/OTP
//! runtime system. This crate implements adapters for Unix-specific system operations.
//!
//! ## Overview
//!
//! The `adapters_system_integration_unix` crate is part of the adapters layer in
//! the CLEAN architecture implementation of Erlang/OTP. It provides I/O adapters
//! for Unix-specific system integration operations, including file descriptor
//! management, pipe operations, and terminal window size queries.
//!
//! ## Platform Support
//!
//! This crate is only available on Unix platforms. On non-Unix platforms, the
//! crate provides placeholder functions that indicate Unix-specific functionality
//! is not available.
//!
//! ## Modules
//!
//! - **[`sys_drivers`](sys_drivers/index.html)**: Unix system drivers for low-level
//!   system operations including file descriptor management, pipe operations,
//!   terminal window size queries, and driver data management
//!
//! ## Functions
//!
//! - `init_fd_data`: Initialize file descriptor data structure
//! - `close_pipes`: Close pipe file descriptors
//! - `fd_get_window_size`: Get terminal window size
//! - `clear_fd_data`: Clear file descriptor data
//! - `nbio_stop_fd`: Stop non-blocking I/O on a file descriptor
//! - `fd_flush`: Mark driver for flushing/termination
//!
//! ## Architecture
//!
//! This crate is based on the C implementation in `sys_drivers.c`. It depends on
//! the Entities layer for fundamental data types.
//!
//! ## See Also
//!
//! - [`adapters_nif_io`](../adapters_nif_io/index.html): NIF I/O polling and event management
//! - [`frameworks_system_integration_unix`](../../frameworks/frameworks_system_integration_unix/index.html): Unix system integration framework

#[cfg(unix)]
pub mod sys_drivers;

#[cfg(unix)]
pub use sys_drivers::{
    SysDrivers, FdData, DriverError,
    init_fd_data, close_pipes, fd_get_window_size,
    clear_fd_data, nbio_stop_fd, fd_flush,
};

#[cfg(not(unix))]
/// Unix-specific functionality is only available on Unix systems
pub fn unix_only() {
    // Placeholder for non-Unix platforms
}

