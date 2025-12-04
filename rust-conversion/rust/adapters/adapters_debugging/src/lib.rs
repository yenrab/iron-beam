//! Adapters Layer: Debugging
//!
//! Provides debugging functionality for the Erlang/OTP runtime system. This crate
//! implements adapters for tracing and debugging operations, enabling runtime
//! inspection and monitoring capabilities.
//!
//! ## Overview
//!
//! The `adapters_debugging` crate is part of the adapters layer in the CLEAN
//! architecture implementation of Erlang/OTP. It provides I/O adapters for debugging
//! and tracing functionality.
//!
//! ## Modules
//!
//! - **[`trace_nif`](trace_nif/index.html)**: Tracing NIF implementations for runtime
//!   tracing and monitoring
//!
//! - **[`tracer`](tracer/index.html)**: Tracer operations for collecting and managing
//!   trace data
//!
//! ## Architecture
//!
//! This crate is based on the C implementation in `trace_nif.c` and `erl_tracer_nif.c`.
//! It depends on the Entities and Use Cases layers for fundamental operations.
//!
//! ## See Also
//!
//! - [`usecases_bifs`](../../usecases/usecases_bifs/index.html): Trace BIF implementations
//! - [`infrastructure_debugging`](../../infrastructure/infrastructure_debugging/index.html): Debugging infrastructure

pub mod trace_nif;
pub mod tracer;

pub use trace_nif::TraceNif;
pub use tracer::Tracer;

