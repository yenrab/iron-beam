//! aarch64 architecture-specific code generation
//!
//! Implements aarch64 instruction emitters for BEAM instructions.
//! Converted from C++ code in erts/emulator/beam/jit/arm/.

mod assembler;
mod runtime_context;
mod runtime_calls;
mod stack_frames;
mod x_register_management;

pub use assembler::ArmBeamAssembler;
pub use runtime_context::{RuntimeContextManager, RuntimeSpec};
pub use runtime_calls::{RuntimeCallManager, RuntimeCallBuilder, RuntimeArg};
pub use stack_frames::StackFrameManager;
pub use x_register_management::{XRegisterManager, LiveRegisterInfo, XRegisterAllocation};

