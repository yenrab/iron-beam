//! API Facades Layer
//!
//! Provides API facades for external callers (functions called from Erlang code) in the
//! Erlang/OTP runtime system. This crate implements the outermost layer that maintains
//! exact C function signatures for Erlang/OTP compatibility while calling underlying
//! Rust implementations from inner layers.
//!
//! ## Overview
//!
//! The `api_facades` crate is the outermost layer in the CLEAN architecture implementation
//! of Erlang/OTP. It provides API facades for 52 external callers, maintaining exact C
//! function signatures for compatibility with existing Erlang code while delegating to
//! Rust implementations in inner layers.
//!
//! ## Key Features
//!
//! - **C Compatibility**: Maintains exact C function signatures for Erlang/OTP compatibility
//! - **No FFI Bindings**: All C code has been reengineered to Rust, eliminating the need
//!   for FFI bindings
//! - **Layer Delegation**: All facades call underlying Rust modules from inner layers
//!   (entities, usecases, adapters, infrastructure, frameworks)
//!
//! ## Modules
//!
//! - **[`nif_facades`](nif_facades/index.html)**: NIF (Native Implemented Function) facades
//!   for functions called from NIFs
//!
//! - **[`bif_facades`](bif_facades/index.html)**: BIF (Built-In Function) facades for
//!   functions called from BIFs
//!
//! - **[`common_facades`](common_facades/index.html)**: Common facades shared across
//!   multiple caller types
//!
//! ## Architecture
//!
//! This crate sits at the outermost layer and provides the interface between Erlang code
//! and the Rust runtime implementation. All facades delegate to inner layers, ensuring
//! that the C-compatible interface is maintained while leveraging the safety and
//! performance of Rust implementations.
//!
//! ## See Also
//!
//! - [`usecases_bifs`](../usecases/usecases_bifs/index.html): BIF implementations
//! - [`adapters_nifs`](../adapters/adapters_nifs/index.html): NIF adapters
//! - [`entities_data_handling`](../entities/entities_data_handling/index.html): Core data handling

pub mod nif_facades;
pub mod bif_facades;
pub mod common_facades;

// Re-export main facade types
pub use nif_facades::*;
pub use bif_facades::*;

