//! Infrastructure Layer: Trace Encoding
//!
//! Provides trace encoding/decoding functionality.
//! Based on encode_trace.c and decode_trace.c
//! Depends on Infrastructure layer (code_loading) and Entities layer.
//!
//! This crate provides a high-level codec interface for Erlang trace structures,
//! wrapping the lower-level encoding/decoding functions from infrastructure_code_loading.

pub mod trace_codec;

pub use trace_codec::{TraceCodec, EncodeError, DecodeError};
pub use infrastructure_code_loading::ErlangTrace;

