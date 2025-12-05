//! Use Cases Layer: Rust NIF Compilation and Safe Loading
//!
//! Provides functionality for compiling and safely loading Rust Native Implemented Functions
//! (NIFs) in the Erlang/OTP runtime system. This crate enables on-the-fly compilation
//! of Rust NIFs with safety verification.
//!
//! ## Overview
//!
//! The `usecases_nif_compilation` crate is part of the use cases layer in the CLEAN
//! architecture implementation of Erlang/OTP. It provides business logic for compiling
//! Rust NIFs and verifying that they contain only safe Rust code.
//!
//! ## Modules
//!
//! - **[`nif_compiler`](nif_compiler/index.html)**: Rust NIF compilation using cargo.
//!   Detects Rust source files, compiles them on-the-fly, and loads the resulting
//!   libraries safely.
//!
//! - **[`safe_rust_verifier`](safe_rust_verifier/index.html)**: Verification of Rust
//!   code to ensure it contains only safe Rust (no unsafe blocks). This provides
//!   an additional safety layer before loading NIFs.
//!
//! - **[`nif_interface_verifier`](nif_interface_verifier/index.html)**: Verification
//!   that Rust NIF code has the proper interface requirements, including `nif_init()`
//!   function with `#[no_mangle]` and `extern "C"`, and proper NIF function signatures.
//!
//! ## Safety
//!
//! This crate prioritizes safety by:
//! - Verifying that Rust code contains no unsafe blocks
//! - Verifying that NIF code has proper interface requirements (`nif_init()`, `#[no_mangle]`, `extern "C"`)
//! - Using safe compilation and loading procedures
//! - Providing clear error messages for unsafe code and interface issues
//!
//! ## Examples
//!
//! ```rust
//! use usecases_nif_compilation::{NifCompiler, SafeRustVerifier, CompileOptions};
//!
//! // Verify Rust code is safe
//! let verifier = SafeRustVerifier::new();
//! let result = verifier.verify("path/to/nif.rs")?;
//!
//! // Compile NIF
//! let compiler = NifCompiler::new();
//! let compile_result = compiler.compile("path/to/nif.rs", CompileOptions::default())?;
//! ```
//!
//! ## See Also
//!
//! - [`adapters_nifs`](../../adapters/adapters_nifs/index.html): NIF adapter layer
//! - [`code_management_code_loading`](../../code_management/code_management_code_loading/index.html): Code loading infrastructure

pub mod nif_compiler;
pub mod safe_rust_verifier;
pub mod nif_interface_verifier;

pub use nif_compiler::{NifCompiler, CompileOptions, CompileResult, CompileError};
pub use safe_rust_verifier::{SafeRustVerifier, VerificationResult, VerificationError};
pub use nif_interface_verifier::{
    NifInterfaceVerifier, NifInterfaceResult, NifInterfaceError,
};

