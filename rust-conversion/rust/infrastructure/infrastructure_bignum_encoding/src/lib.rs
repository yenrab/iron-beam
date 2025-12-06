//! Infrastructure Layer: Bignum Encoding
//!
//! Provides bignum encoding and decoding functionality for the Erlang/OTP runtime system.
//! This crate implements codecs for serializing and deserializing arbitrary precision
//! numbers in various formats.
//!
//! ## Overview
//!
//! The `infrastructure_bignum_encoding` crate is part of the infrastructure layer in
//! the CLEAN architecture implementation of Erlang/OTP. It provides encoding and decoding
//! operations for big numbers, enabling serialization for network transmission and storage.
//!
//! ## Codecs
//!
//! - **[`bignum_codec`](bignum_codec/index.html)**: Codec for arbitrary precision integers
//!   (bignum). Handles encoding and decoding of `BigNumber` types.
//!
//! - **[`rational_codec`](rational_codec/index.html)**: Codec for arbitrary precision
//!   rational numbers. Handles encoding and decoding of `BigRational` types.
//!
//! ## Architecture
//!
//! This crate is based on the C implementation in `decode_big.c` and `encode_bignum.c`.
//! It depends on the Entities layer for fundamental data types.
//!
//! ## See Also
//!
//! - [`entities_utilities`](../../entities/entities_utilities/index.html): BigNumber and BigRational types
//! - [`infrastructure_code_loading`](../infrastructure_code_loading/index.html): General code loading infrastructure

mod common;

pub mod bignum_codec;
pub mod rational_codec;

pub use bignum_codec::BignumCodec;
pub use rational_codec::RationalCodec;

// Re-export error types for convenience
pub use common::{EncodeError, DecodeError};

// Re-export byte conversion helpers for in-memory representations
pub use common::{integer_to_bytes, bytes_to_integer};

