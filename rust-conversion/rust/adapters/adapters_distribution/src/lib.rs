//! Adapters Layer: Distribution
//!
//! Provides distribution functionality for the Erlang/OTP runtime system. This crate
//! implements adapters for distributed Erlang communication, including external term
//! format encoding/decoding and Unix Domain Socket distribution.
//!
//! ## Overview
//!
//! The `adapters_distribution` crate is part of the adapters layer in the CLEAN
//! architecture implementation of Erlang/OTP. It provides I/O adapters for distributed
//! Erlang communication protocols.
//!
//! ## Modules
//!
//! - **[`external`](external/index.html)**: External term format (ETF) encoding and
//!   decoding for serializing Erlang terms for network transmission
//!
//! - **[`uds`](uds/index.html)**: Unix Domain Socket distribution driver for local
//!   inter-process communication
//!
//! ## Architecture
//!
//! This crate is based on the C implementation in `external.c` and `uds_drv.c`.
//! It depends on the Entities layer for fundamental data types.
//!
//! ## See Also
//!
//! - [`entities_data_handling`](../../entities/entities_data_handling/index.html): Term types for distribution
//! - [`frameworks_system_integration`](../../frameworks/frameworks_system_integration/index.html): System integration framework

pub mod external;
pub mod uds;

pub use external::ExternalTerm;
pub use uds::UdsDistribution;

