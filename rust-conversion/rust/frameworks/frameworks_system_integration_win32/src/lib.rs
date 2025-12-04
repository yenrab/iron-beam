//! Frameworks Layer: Windows System Integration
//!
//! Provides Windows-specific system integration functionality at the framework level for the
//! Erlang/OTP runtime system. This crate implements Windows-specific system operations.
//!
//! ## Overview
//!
//! The `frameworks_system_integration_win32` crate is part of the frameworks layer in the
//! CLEAN architecture implementation of Erlang/OTP. It provides Windows-specific system
//! integration operations at the framework level.
//!
//! ## Platform Support
//!
//! This crate is only available on Windows platforms. On non-Windows platforms, the crate
//! provides placeholder functions that indicate Windows-specific functionality is not available.
//!
//! ## Modules
//!
//! - **[`sys_integration`](sys_integration/index.html)**: Windows system integration framework
//!   providing time management and other Windows-specific operations
//!
//! ## Architecture
//!
//! This crate is based on the C implementation in `sys_time.c` and related Windows system files.
//! It depends on the Entities and Adapters layers.
//!
//! ## See Also
//!
//! - [`frameworks_system_integration_common`](../frameworks_system_integration_common/index.html): Common system integration
//! - [`entities_system_integration_win32`](../../entities/entities_system_integration_win32/index.html): Windows entity layer

#[cfg(windows)]
pub mod sys_integration;

#[cfg(windows)]
pub use sys_integration::SysIntegration;

#[cfg(not(windows))]
/// Windows-specific functionality is only available on Windows
pub fn windows_only() {
    // Placeholder for non-Windows platforms
}

