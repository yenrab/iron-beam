//! Entities Layer: Process Management
//!
//! Provides process management entities for the Erlang runtime system.
//! Based on erts/emulator/beam/erl_process.h
//!
//! This is a minimal implementation with only the fields currently needed
//! by the Rust codebase. Additional fields can be added as needed.
//!
//! The heap is implemented using safe Rust (`Vec<Eterm>`) with index-based
//! access instead of raw pointers for maximum safety.

pub mod process;

// Re-export main types for convenience
pub use process::{Process, ProcessId, ProcessState, Eterm, ErtsCodePtr};
