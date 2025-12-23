//! BEAM Instruction Definitions and Parser
//!
//! This module defines BEAM opcodes, instruction formats, and parsing logic
//! for converting BEAM bytecode into structured instruction representations.

pub mod opcodes;
pub mod parser;
pub mod types;

pub use opcodes::*;
pub use parser::*;
pub use types::*;
