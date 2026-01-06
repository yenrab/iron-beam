//! Infrastructure Layer: BeamAsm JIT Execution
//!
//! Provides load-time conversion of BEAM instructions into native code on x86-64 and aarch64.
//! This crate implements the BeamAsm JIT compiler that eliminates instruction dispatching
//! overhead and specializes each instruction on their argument types.
//!
//! ## Overview
//!
//! The `infrastructure_beamasm` crate is part of the infrastructure layer in the CLEAN
//! architecture implementation of Erlang/OTP. It provides:
//! - JIT code generation for BEAM instructions
//! - Architecture-specific instruction emitters (x86-64, aarch64)
//! - Code loading and module management
//! - Metadata tracking for debugging and tracing
//!
//! ## REPL Integration
//!
//! The REPL (Read-Eval-Print Loop) uses this crate for JIT compilation in the same way
//! as the C version, but reengineered for Rust:
//!
//! 1. REPL expressions are scanned and parsed (`infrastructure_utilities::erl_scan`, `erl_parse`)
//! 2. Parsed expressions are compiled to BEAM bytecode
//! 3. BEAM bytecode is JIT-compiled via `BeamAsmLoader` in this crate
//! 4. JIT-compiled code is executed via `infrastructure_emulator_loop::process_main()`
//!
//! This ensures the REPL uses the same JIT execution path as regular Erlang code.
//!
//! ## Architecture
//!
//! This crate is based on the C++ implementation in `erts/emulator/beam/jit/`. It uses
//! Cranelift for JIT code generation (Rust alternative to asmjit).
//!
//! ## Modules
//!
//! - **[`common`](common/index.html)**: Common assembler functionality
//! - **[`jit`](jit/index.html)**: JIT code generation and allocation
//! - **[`arch`](arch/index.html)**: Architecture-specific code emitters
//! - **[`loader`](loader/index.html)**: Code loading and module management
//! - **[`metadata`](metadata/index.html)**: Metadata tracking for debugging
//!
//! ## Dependencies
//!
//! - `entities_data_handling`: Core data structures and types

pub mod beam_instructions;
pub mod common;
pub mod jit;
pub mod loader;
pub mod metadata;
pub mod types;
pub mod asmjit_wrapper;
pub mod scheduler_data;
pub mod process_registers;
pub mod heap_allocation;
pub mod scheduler_integration;
pub mod context_switching;
pub mod exception_handling;
pub mod error_integration;
pub mod bif_integration;
pub mod bit_syntax_operations;
pub mod map_operations;

#[cfg(target_arch = "x86_64")]
pub mod arch {
    pub mod x86;
}

#[cfg(target_arch = "aarch64")]
pub mod arch {
    pub mod arm;
}

// Re-export main types
pub use common::{BeamAssembler, BeamAssemblerError};
pub use common::args::{ArgVal, ArgType};
pub use jit::{JitAllocator, JitAllocatorError};
pub use loader::{BeamAsmLoader, LoaderState};
pub use metadata::BeamAsmMetadata;
pub use scheduler_data::{ErtsSchedulerData, ErtsSchedulerRegisters, JitProcessMain, JitBeamFunction};
pub use process_registers::{ProcessRegisterManager, ProcessRegisterSync};
pub use heap_allocation::{HeapAllocationCoordinator, HeapAllocRequest, GcRootInfo};
pub use scheduler_integration::{SchedulerIntegration, YieldMode, ReductionOperation};
pub use context_switching::{ContextSwitching, ContextSwitchMode, ProcessState};
pub use exception_handling::{ExceptionHandling, ExceptionClass, ExceptionState, CatchInfo};
pub use error_integration::{ErrorIntegration, ErrorContext, ErrorMFA};
pub use bif_integration::{BifIntegration, BifCallInfo, ExternalCallInfo, BifType, BifResult};
pub use bit_syntax_operations::{BitSyntaxOperations, BitSyntaxContext, BitFieldSpec, BinaryConstructionState, BinaryMatchResult, BitFieldType, BitEndianness};
pub use map_operations::{MapOperations, MapOperationContext, MapCreationSpec, MapIterationContext, MapOperationResult, MapOperation, MapType, MapKeyValuePair};

#[cfg(target_arch = "x86_64")]
pub use arch::x86::global::generate_process_main;

#[cfg(target_arch = "aarch64")]
pub use arch::arm::{RuntimeContextManager, RuntimeSpec, RuntimeCallManager, RuntimeCallBuilder, RuntimeArg, StackFrameManager, XRegisterManager, LiveRegisterInfo, XRegisterAllocation};

// Additional types are accessible through their respective modules

/// Initialize the BeamAsm JIT system
///
/// This function must be called before using any JIT functionality.
/// It initializes global assemblers and sets up architecture-specific code.
pub fn beamasm_init() -> Result<(), BeamAssemblerError> {
    // Initialize global assemblers
    // Set up architecture-specific code
    // Initialize metadata tracking
    Ok(())
}

/// Create a new assembler for a module
///
/// # Arguments
/// * `module` - Module atom
/// * `num_labels` - Number of labels in the module
/// * `num_functions` - Number of functions in the module
/// * `beam_file` - BEAM file structure
pub fn beamasm_new_assembler(
    module: u64, // Eterm
    num_labels: usize,
    num_functions: usize,
    beam_file: &[u8], // BeamFile representation
    is_repl_module: bool,
) -> Result<Box<dyn BeamAssembler>, BeamAssemblerError> {
    // Create architecture-specific assembler
    #[cfg(target_arch = "x86_64")]
    {
        use arch::x86::X86BeamAssembler;
        Ok(Box::new(X86BeamAssembler::new(
            module,
            num_labels,
            num_functions,
            beam_file,
            is_repl_module,
        )?))
    }
    
    #[cfg(target_arch = "aarch64")]
    {
        use arch::arm::ArmBeamAssembler;
        Ok(Box::new(ArmBeamAssembler::new(
            module,
            num_labels,
            num_functions,
            beam_file,
            is_repl_module,
        )?))
    }
    
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        Err(BeamAssemblerError::UnsupportedArchitecture)
    }
}

