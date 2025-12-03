//! Infrastructure Layer: Bignum Encoding
//!
//! Provides bignum encoding/decoding.
//! Based on decode_big.c and encode_bignum.c
//! Depends on Entities layer.
//!
//! This crate provides codecs for:
//! - BigNumber: Arbitrary precision integers (bignum)
//! - BigRational: Arbitrary precision rational numbers

mod common;

pub mod bignum_codec;
pub mod rational_codec;

pub use bignum_codec::BignumCodec;
pub use rational_codec::RationalCodec;

// Re-export error types for convenience
pub use common::{EncodeError, DecodeError};

