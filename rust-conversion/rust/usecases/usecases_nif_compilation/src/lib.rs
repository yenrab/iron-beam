//! Use Cases Layer: Rust NIF Compilation and Safe Loading
//!
//! Provides functionality to:
//! - Detect Rust source files (.rs)
//! - Verify code contains only safe Rust (no unsafe blocks)
//! - Compile Rust NIFs on-the-fly using cargo
//! - Load compiled NIF libraries safely

pub mod nif_compiler;
pub mod safe_rust_verifier;

pub use nif_compiler::{NifCompiler, CompileOptions, CompileResult, CompileError};
pub use safe_rust_verifier::{SafeRustVerifier, VerificationResult, VerificationError};

