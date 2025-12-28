//! BEAM Instruction Utilities
//!
//! Shared utilities for BEAM instruction parsing, opcode definitions,
//! and related functionality used across the codebase.
//!
//! Note: The beam_instructions module has been moved to infrastructure_beam_instructions
//! to break circular dependencies.

// Re-export from the dedicated crate
pub use infrastructure_beam_instructions::beam_instructions;
