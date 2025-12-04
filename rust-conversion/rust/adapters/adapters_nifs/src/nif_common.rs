//! NIF Common Utilities
//!
//! Provides common utilities and types for NIF (Native Implemented Function) implementations
//! in the Erlang/OTP runtime system. This module defines shared infrastructure that all
//! NIF implementations use.
//!
//! ## Overview
//!
//! NIFs allow Erlang code to call native Rust functions directly, providing a bridge
//! between the Erlang runtime and native code. This module provides:
//! - **NIF Environment**: Context for NIF operations
//! - **Common Types**: Shared types used across NIF implementations
//! - **Utility Functions**: Helper functions for NIF development
//!
//! ## Examples
//!
//! ```rust
//! use adapters_nifs::nif_common::NifEnv;
//!
//! // Create a NIF environment
//! let env = NifEnv::new();
//! // Use env for NIF operations...
//! ```
//!
//! ## See Also
//!
//! - [`adapters_nifs::file`](super::file/index.html): File NIF operations
//! - [`adapters_nifs::buffer`](super::buffer/index.html): Buffer NIF operations
//! - [`usecases_nif_compilation`](../../usecases/usecases_nif_compilation/index.html): NIF compilation utilities

/// NIF environment (placeholder)
pub struct NifEnv;

impl NifEnv {
    /// Create a new NIF environment
    pub fn new() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nif_env() {
        let _env = NifEnv::new();
    }
}

