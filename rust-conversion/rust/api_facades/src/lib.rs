//! API Facades Layer
//!
//! Provides API facades for 52 external callers (functions called from Erlang).
//! These facades maintain exact C function signatures for Erlang/OTP compatibility.
//!
//! All facades call underlying Rust modules from inner layers.
//! No FFI bindings needed - all C code has been reengineered to Rust.

pub mod nif_facades;
pub mod driver_facades;
pub mod bif_facades;
pub mod common_facades;

// Re-export main facade types
pub use nif_facades::*;
pub use driver_facades::*;
pub use bif_facades::*;

