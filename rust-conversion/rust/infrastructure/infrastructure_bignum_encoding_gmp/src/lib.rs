//! Infrastructure Layer: Bignum Encoding with GMP
//!
//! Provides bignum encoding/decoding using GMP library.
//! Based on decode_bignum.c
//! Depends on Entities and infrastructure_bignum_encoding layers.

pub mod bignum_gmp;

pub use bignum_gmp::BignumGmp;

