//! Adapters Layer: ETS Tables
//!
//! Provides ETS (Erlang Term Storage) table debugging functionality for the Erlang/OTP
//! runtime system. This crate implements adapters for debugging and inspecting ETS tables.
//!
//! ## Overview
//!
//! The `adapters_ets_tables` crate is part of the adapters layer in the CLEAN
//! architecture implementation of Erlang/OTP. It provides I/O adapters for ETS table
//! debugging operations.
//!
//! ## Modules
//!
//! - **[`ets_debug`](ets_debug/index.html)**: ETS table debugging functionality for
//!   inspecting table state, contents, and performance metrics
//!
//! ## Architecture
//!
//! This crate is based on the C implementation in `socket_dbg.c`. It depends on the
//! Entities layer for fundamental data types.
//!
//! ## See Also
//!
//! - [`infrastructure_ets_tables`](../../infrastructure/infrastructure_ets_tables/index.html): ETS table infrastructure
//! - [`entities_data_handling`](../../entities/entities_data_handling/index.html): Term types for ETS

pub mod ets_debug;

pub use ets_debug::EtsDebug;

