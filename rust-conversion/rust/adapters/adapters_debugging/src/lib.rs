//! Adapters Layer: Debugging
//!
//! Provides debugging functionality:
//! - Tracing NIFs
//! - Tracer operations
//!
//! Based on trace_nif.c and erl_tracer_nif.c
//! Depends on Entities and Use Cases layers.

pub mod trace_nif;
pub mod tracer;

pub use trace_nif::TraceNif;
pub use tracer::Tracer;

