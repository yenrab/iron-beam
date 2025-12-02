//! Infrastructure Layer: Bignum Encoding
//!
//! Provides bignum encoding/decoding.
//! Based on decode_big.c
//! Depends on Entities layer.

pub mod bignum_codec;
pub mod rational_codec;

pub use bignum_codec::BignumCodec;
pub use rational_codec::RationalCodec;

