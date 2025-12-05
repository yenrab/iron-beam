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
//! - **[`nif_loader`](nif_loader/index.html)**: NIF library loading and tracking
//!   infrastructure for dynamic library loading and process-NIF association
//!
//! ## Architecture
//!
//! This crate is based on the C implementation in `erts/emulator/nifs/common/*.c`.
//! The `nif_loader` module is a new Rust implementation with no direct C source file.
//! It depends on the Entities and Use Cases layers for fundamental operations.
//!
//! ## See Also
//!
//! - [`usecases_nif_compilation`](../../usecases/usecases_nif_compilation/index.html): NIF compilation use cases
//! - [`entities_data_handling`](../../entities/entities_data_handling/index.html): Term types for NIFs

pub mod buffer;
pub mod file;
pub mod nif_common;
pub mod nif_loader;

pub use buffer::BufferNif;
pub use file::FileNif;
pub use nif_loader::{
    NifLoader, NifLibrary, NifLibraryRef, NifFunction, NifRegistry, NifFunctionPtr,
    NifLoadError, NifUnloadError, NifError,
    RustNifMetadata, FunctionMetadata, NifGetMetadataFn,
};

