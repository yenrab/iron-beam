//! Adapters Layer: Unix System Integration
//!
//! Provides Unix-specific system integration functionality for the Erlang/OTP
//! runtime system. This crate implements adapters for Unix-specific system operations.
//!
//! ## Overview
//!
//! The `adapters_system_integration_unix` crate is part of the adapters layer in
//! the CLEAN architecture implementation of Erlang/OTP. It provides I/O adapters
//! for Unix-specific system integration operations.
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
//!   system operations
//!
//! ## Architecture
//!
//! This crate is based on the C implementation in `sys_drivers.c`. It depends on
//! the Entities layer for fundamental data types.
//!
//! ## See Also
//!
//! - [`adapters_system_integration_common`](../adapters_system_integration_common/index.html): Common system integration
//! - [`frameworks_system_integration_unix`](../../frameworks/frameworks_system_integration_unix/index.html): Unix system integration framework

#[cfg(unix)]
pub mod sys_drivers;

#[cfg(unix)]
pub use sys_drivers::SysDrivers;

#[cfg(not(unix))]
/// Unix-specific functionality is only available on Unix systems
pub fn unix_only() {
    // Placeholder for non-Unix platforms
}

