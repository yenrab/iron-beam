//! Frameworks Layer: Unix System Integration
//!
//! Provides Unix-specific system integration functionality at the framework level for the
//! Erlang/OTP runtime system. This crate implements Unix-specific system operations.
//!
//! ## Overview
//!
//! The `frameworks_system_integration_unix` crate is part of the frameworks layer in the
//! CLEAN architecture implementation of Erlang/OTP. It provides Unix-specific system
//! integration operations at the framework level.
//!
//! ## Platform Support
//!
//! This crate is only available on Unix platforms. On non-Unix platforms, the crate provides
//! placeholder functions that indicate Unix-specific functionality is not available.
//!
//! ## Modules
//!
//! - **[`sys_integration`](sys_integration/index.html)**: Unix system integration framework
//!   providing environment variable handling and other Unix-specific operations
//!
//! ## Architecture
//!
//! This crate is based on the C implementation in `sys_env.c` and related Unix system files.
//! It depends on the Entities and Adapters layers.
//!
//! ## See Also
//!
//! - [`frameworks_system_integration_common`](../frameworks_system_integration_common/index.html): Common system integration
//! - [`adapters_system_integration_unix`](../../adapters/adapters_system_integration_unix/index.html): Unix adapter layer

#[cfg(unix)]
pub mod sys_integration;

#[cfg(unix)]
pub use sys_integration::{SysIntegration, SysError};

#[cfg(not(unix))]
/// Unix-specific functionality is only available on Unix systems
pub fn unix_only() {
    // Placeholder for non-Unix platforms
}

