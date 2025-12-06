//! Infrastructure NIF API
//!
//! Provides the Rust NIF API - equivalent to the C `erl_nif.h` API but implemented in pure Rust.
//!
//! ## Purpose
//!
//! NIFs (Native Implemented Functions) need to create and decode Erlang terms in memory.
//! This crate provides functions that work with in-memory Erlang terms (Eterm/u64 values),
//! unlike `infrastructure_data_handling` which provides EI format encoding/decoding for serialization.
//!
//! ## Overview
//!
//! The NIF API provides:
//! - **Term Creation**: Functions to create Erlang terms (`enif_make_*`)
//! - **Term Decoding**: Functions to decode Erlang terms (`enif_get_*`)
//! - **Error Handling**: Functions for exception handling
//! - **Resource Management**: Functions for managing NIF resources
//!
//! ## Term Representation
//!
//! Erlang terms are represented as `u64` values (Eterm). Terms use a tagged pointer scheme
//! where the lower bits indicate the term type:
//! - Immediate values (small integers, atoms, nil) are encoded directly
//! - Boxed values (tuples, lists, binaries) are pointers to heap structures
//!
//! ## NIF Environment
//!
//! Functions take a `*mut c_void` env parameter representing the NIF environment, which
//! provides access to the process heap and other runtime resources.
//!
//! ## See Also
//!
//! - [`entities_data_handling`](../../entities/entities_data_handling/index.html): Term type definitions
//! - [`infrastructure_data_handling`](../infrastructure_data_handling/index.html): EI format encoding/decoding
//! - `erts/emulator/beam/erl_nif.c` - C reference implementation
//! - `erts/emulator/beam/erl_nif.h` - C header

pub mod term_creation;
pub mod term_decoding;
pub mod error_handling;
pub mod resource_management;
pub mod nif_env;

pub use term_creation::*;
pub use term_decoding::*;
pub use error_handling::*;
pub use resource_management::*;
pub use nif_env::*;

/// NIF term type (Eterm)
///
/// Represents an Erlang term as a u64 value. This matches the C `ERL_NIF_TERM` type.
pub type NifTerm = u64;

/// Character encoding for atoms and strings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NifCharEncoding {
    /// Latin1 encoding (ISO-8859-1)
    Latin1,
    /// UTF-8 encoding
    Utf8,
}

