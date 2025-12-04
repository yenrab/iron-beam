//! Frameworks Layer: System Integration Base
//!
//! Provides base system integration functionality for the Erlang/OTP runtime system. This
//! crate implements the foundational framework for system integration operations.
//!
//! ## Overview
//!
//! The `frameworks_system_integration` crate is part of the frameworks layer in the CLEAN
//! architecture implementation of Erlang/OTP. It provides the base framework for system
//! integration, which is extended by platform-specific implementations.
//!
//! ## Modules
//!
//! - **[`sys_base`](sys_base/index.html)**: Base system integration framework providing
//!   fundamental system operations and shell integration
//!
//! ## Architecture
//!
//! This crate is based on the C implementation in `sys_shell.c`. It depends on the Entities
//! and Frameworks Common layers.
//!
//! ## See Also
//!
//! - [`frameworks_system_integration_common`](../frameworks_system_integration_common/index.html): Common system integration
//! - [`frameworks_system_integration_unix`](../frameworks_system_integration_unix/index.html): Unix-specific integration
//! - [`frameworks_system_integration_win32`](../frameworks_system_integration_win32/index.html): Windows-specific integration

pub mod sys_base;

pub use sys_base::SysBase;

