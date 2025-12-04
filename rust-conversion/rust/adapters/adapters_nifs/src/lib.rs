//! Adapters Layer: NIFs (Native Implemented Functions)
//!
//! Provides NIF implementations for the Erlang/OTP runtime system. This crate
//! implements adapters for various NIF modules that provide native functionality
//! to Erlang code.
//!
//! ## Overview
//!
//! The `adapters_nifs` crate is part of the adapters layer in the CLEAN
//! architecture implementation of Erlang/OTP. It provides I/O adapters for
//! Native Implemented Functions (NIFs), which allow Erlang code to call native
//! functions written in Rust or C.
//!
//! ## Modules
//!
//! - **[`buffer`](buffer/index.html)**: Buffer NIFs for efficient binary data
//!   manipulation operations
//!
//! - **[`file`](file/index.html)**: File NIFs for file system operations
//!
//! - **[`nif_common`](nif_common/index.html)**: Common NIF infrastructure and
//!   utilities shared across NIF modules
//!
//! ## Architecture
//!
//! This crate is based on the C implementation in `erts/emulator/nifs/common/*.c`.
//! It depends on the Entities and Use Cases layers for fundamental operations.
//!
//! ## See Also
//!
//! - [`usecases_nif_compilation`](../../usecases/usecases_nif_compilation/index.html): NIF compilation use cases
//! - [`entities_data_handling`](../../entities/entities_data_handling/index.html): Term types for NIFs

pub mod buffer;
pub mod file;
pub mod nif_common;

pub use buffer::BufferNif;
pub use file::FileNif;

