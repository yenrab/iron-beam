//! x86-64 architecture-specific code generation
//!
//! Implements x86-64 instruction emitters for BEAM instructions.
//! Converted from C++ code in erts/emulator/beam/jit/x86/.

mod assembler;
mod global;
mod module;
mod instructions;

pub use assembler::X86BeamAssembler;
pub use global::X86BeamGlobalAssembler;
pub use module::X86BeamModuleAssembler;

