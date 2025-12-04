//! Infrastructure Layer: Trace Encoding
//!
//! Provides trace encoding and decoding functionality for the Erlang/OTP runtime system.
//! This crate implements a high-level codec interface for Erlang trace structures, wrapping
//! the lower-level encoding/decoding functions from the code loading infrastructure.
//!
//! ## Overview
//!
//! The `infrastructure_trace_encoding` crate is part of the infrastructure layer in the
//! CLEAN architecture implementation of Erlang/OTP. It provides a convenient interface for
//! encoding and decoding trace structures used in runtime tracing and debugging.
//!
//! ## Modules
//!
//! - **[`trace_codec`](trace_codec/index.html)**: High-level trace codec interface that
//!   wraps the lower-level encoding/decoding functions from `infrastructure_code_loading`
//!
//! ## Architecture
//!
//! This crate is based on the C implementation in `encode_trace.c` and `decode_trace.c`.
//! It depends on the Infrastructure layer (`infrastructure_code_loading`) and the Entities
//! layer. It provides a high-level interface that simplifies trace encoding/decoding operations.
//!
//! ## See Also
//!
//! - [`infrastructure_code_loading`](../infrastructure_code_loading/index.html): Low-level trace encoding/decoding
//! - [`adapters_debugging`](../../adapters/adapters_debugging/index.html): Debugging adapters that use traces

pub mod trace_codec;

pub use trace_codec::{TraceCodec, EncodeError, DecodeError};
pub use infrastructure_code_loading::ErlangTrace;

