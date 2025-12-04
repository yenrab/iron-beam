//! Frameworks Layer: Common System Integration
//!
//! Provides common system integration functionality at the framework level for the Erlang/OTP
//! runtime system. This crate implements platform-independent system integration operations.
//!
//! ## Overview
//!
//! The `frameworks_system_integration_common` crate is part of the frameworks layer in the
//! CLEAN architecture implementation of Erlang/OTP. It provides common system integration
//! operations that are shared across all platforms.
//!
//! ## Modules
//!
//! - **[`sys_common`](sys_common/index.html)**: Common system integration framework providing
//!   memory segment management and other platform-independent operations
//!
//! ## Architecture
//!
//! This crate is based on the C implementation in `erl_mseg.c`. It depends on the Entities
//! and Adapters layers.
//!
//! ## See Also
//!
//! - [`frameworks_system_integration`](../frameworks_system_integration/index.html): Base system integration
//! - [`entities_system_integration_common`](../../entities/entities_system_integration_common/index.html): Entity-level system integration

pub mod sys_common;

pub use sys_common::SysCommon;

