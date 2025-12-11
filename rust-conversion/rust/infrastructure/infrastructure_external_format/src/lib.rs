//! Infrastructure Layer: External Term Format
//!
//! Provides external term format (ETF) encoding/decoding infrastructure for the Erlang/OTP
//! runtime system. This crate implements the core encoding and decoding functions used
//! for serializing Erlang terms to/from the external term format.
//!
//! ## Overview
//!
//! The `infrastructure_external_format` crate is part of the infrastructure layer in the
//! CLEAN architecture implementation of Erlang/OTP. It provides the core infrastructure
//! for encoding and decoding Erlang terms in the External Term Format (ETF), which is
//! used for:
//! - Distribution between BEAM nodes
//! - `erlang:term_to_binary/1` and `erlang:binary_to_term/1` BIFs
//! - Persistent storage of Erlang terms
//!
//! ## Modules
//!
//! - **[`encoding`](encoding/index.html)**: Core encoding functions
//!   (enc_term, enc_atom, enc_pid, erts_encode_ext)
//!
//! - **[`decoding`](decoding/index.html)**: Core decoding functions
//!   (dec_term, dec_atom, dec_pid, erts_decode_ext)
//!
//! - **[`size_calculation`](size_calculation/index.html)**: Size calculation functions
//!   (erts_encode_ext_size, encode_size_struct_int)
//!
//! ## Architecture
//!
//! This crate is based on the C implementation in `external.c`. It depends on:
//! - `infrastructure_data_handling` for EI format encoding/decoding
//! - `infrastructure_code_loading` for low-level encoding primitives
//! - `entities_data_handling` for term types
//!
//! The external term format is essentially the same as EI (Erlang Interface) format,
//! but with a version magic byte (131) prefix. This crate provides the infrastructure
//! functions that handle the version byte and coordinate encoding/decoding.
//!
//! ## See Also
//!
//! - [`infrastructure_data_handling`](../infrastructure_data_handling/index.html): EI format encoding/decoding
//! - [`infrastructure_code_loading`](../infrastructure_code_loading/index.html): Low-level encoding primitives
//! - [`usecases_bifs`](../../usecases/usecases_bifs/index.html): BIF wrappers (term_to_binary, binary_to_term)

pub mod encoding;
pub mod decoding;
pub mod size_calculation;

pub use encoding::{enc_term, enc_atom, enc_pid, erts_encode_ext, EncodeError};
pub use decoding::{dec_term, dec_atom, dec_pid, erts_decode_ext, DecodeError};
pub use size_calculation::{erts_encode_ext_size, encode_size_struct_int, SizeCalculationError};

/// External term format version magic byte
/// This is the first byte in ETF-encoded data (value 131)
pub const VERSION_MAGIC: u8 = 131;


