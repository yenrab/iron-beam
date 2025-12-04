//! Adapters Layer: Common System Integration
//!
//! Provides common system integration functionality for the Erlang/OTP runtime system.
//! This crate implements adapters for platform-independent system integration operations.
//!
//! ## Overview
//!
//! The `adapters_system_integration_common` crate is part of the adapters layer in
//! the CLEAN architecture implementation of Erlang/OTP. It provides I/O adapters for
//! system integration operations that are common across all platforms.
//!
//! ## Modules
//!
//! - **[`check_io`](check_io/index.html)**: I/O checking functionality for monitoring
//!   and managing I/O operations in the runtime
//!
//! ## Architecture
//!
//! This crate is based on the C implementation in `erl_check_io.c`. It depends on
//! the Entities layer for fundamental data types.
//!
//! ## See Also
//!
//! - [`adapters_system_integration_unix`](../adapters_system_integration_unix/index.html): Unix-specific system integration
//! - [`frameworks_system_integration_common`](../../frameworks/frameworks_system_integration_common/index.html): Common system integration framework

pub mod check_io;

pub use check_io::CheckIo;

