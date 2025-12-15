//! aarch64 architecture-specific code generation
//!
//! Implements aarch64 instruction emitters for BEAM instructions.
//! Converted from C++ code in erts/emulator/beam/jit/arm/.

mod assembler;

pub use assembler::ArmBeamAssembler;

