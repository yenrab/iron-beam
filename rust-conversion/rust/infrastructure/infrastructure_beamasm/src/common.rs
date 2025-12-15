//! Common assembler functionality
//!
//! Base functionality shared across all architecture-specific assemblers.
//! Converted from C++ `BeamAssemblerCommon` class.

use thiserror::Error;

use crate::jit::JitAllocator;
use crate::asmjit_wrapper::{CodeHolder, Assembler};

/// Errors that can occur during assembly
#[derive(Debug, Error)]
pub enum BeamAssemblerError {
    #[error("Unsupported architecture")]
    UnsupportedArchitecture,
    #[error("Code generation failed: {0}")]
    CodeGenerationFailed(String),
    #[error("JIT allocation failed: {0}")]
    JitAllocationFailed(String),
    #[error("Invalid label")]
    InvalidLabel,
    #[error("Invalid function index")]
    InvalidFunctionIndex,
}

/// Trait for architecture-specific assemblers
///
/// Converted from C++ `BeamAssemblerCommon` base class.
/// Note: Methods that return raw pointers are safe to call from any thread,
/// but the pointers themselves are not Send/Sync. The trait is Send + Sync
/// because the assembler state can be moved between threads.
pub trait BeamAssembler: Send + Sync {
    /// Get the base address of the generated code
    fn get_base_address(&self) -> *const u8;

    /// Get the current offset in the code
    fn get_offset(&self) -> usize;

    /// Generate code and allocate executable memory
    fn codegen(
        &mut self,
        allocator: &mut JitAllocator,
    ) -> Result<(*const u8, *mut u8), BeamAssemblerError>;

    /// Get code pointer for a label
    fn get_code(&self, label: usize) -> Result<*const u8, BeamAssemblerError>;

    /// Get code pointer for a lambda
    fn get_lambda(&self, index: usize) -> Result<*const u8, BeamAssemblerError>;

    /// Get read-only data pointer
    fn get_rodata(&self, label: &str) -> Option<*const u8>;

    /// Embed read-only data
    fn embed_rodata(&mut self, label: &str, data: &[u8]) -> Result<(), BeamAssemblerError>;

    /// Embed BSS (uninitialized data)
    fn embed_bss(&mut self, label: &str, size: usize) -> Result<(), BeamAssemblerError>;

    /// Emit a BEAM instruction
    fn emit(
        &mut self,
        opcode: u32,
        args: &[crate::args::ArgVal],
    ) -> Result<(), BeamAssemblerError>;

    /// Patch catch handlers
    fn patch_catches(&mut self, rw_base: *mut u8) -> Result<usize, BeamAssemblerError>;

    /// Patch import entry
    fn patch_import(
        &mut self,
        rw_base: *mut u8,
        index: usize,
        export: &Export,
    ) -> Result<(), BeamAssemblerError>;

    /// Patch literal entry
    fn patch_literal(
        &mut self,
        rw_base: *mut u8,
        index: usize,
        literal: u64, // Eterm
    ) -> Result<(), BeamAssemblerError>;

    /// Patch lambda entry
    fn patch_lambda(
        &mut self,
        rw_base: *mut u8,
        index: usize,
        fun_entry: &FunEntry,
    ) -> Result<(), BeamAssemblerError>;

    /// Patch string table
    fn patch_strings(&mut self, rw_base: *mut u8, strtab: &[u8]) -> Result<(), BeamAssemblerError>;
}

/// Export entry structure
#[derive(Debug, Clone)]
pub struct Export {
    pub module: u64, // Eterm (atom)
    pub function: u64, // Eterm (atom)
    pub arity: u32,
    pub address: *const u8,
}

/// Function entry structure
#[derive(Debug, Clone)]
pub struct FunEntry {
    pub address: *const u8,
    pub arity: u32,
    pub index: usize,
}

/// Common assembler state
///
/// Shared state for all assemblers, converted from C++ member variables.
pub struct AssemblerState {
    /// asmjit Assembler (contains CodeHolder)
    assembler: Assembler,
    /// Labels map (label index -> asmjit label ID)
    labels: std::collections::HashMap<usize, u32>,
    /// Lambda entries
    lambdas: Vec<LambdaEntry>,
    /// Read-only data section
    rodata: std::collections::HashMap<String, Vec<u8>>,
    /// BSS section
    bss: std::collections::HashMap<String, usize>,
}

impl AssemblerState {
    /// Create a new assembler state
    pub fn new() -> Result<Self, BeamAssemblerError> {
        let code_holder = CodeHolder::new()
            .map_err(|e| BeamAssemblerError::CodeGenerationFailed(e.to_string()))?;
        
        let assembler = Assembler::new(code_holder)
            .map_err(|e| BeamAssemblerError::CodeGenerationFailed(e.to_string()))?;

        Ok(Self {
            assembler,
            labels: std::collections::HashMap::new(),
            lambdas: Vec::new(),
            rodata: std::collections::HashMap::new(),
            bss: std::collections::HashMap::new(),
        })
    }

    /// Create a new label
    pub fn new_label(&mut self, index: usize) -> Result<u32, BeamAssemblerError> {
        let mut label = self.assembler.new_label()
            .map_err(|e| BeamAssemblerError::CodeGenerationFailed(e.to_string()))?;
        let label_id = label.id();
        self.labels.insert(index, label_id);
        Ok(label_id)
    }

    /// Get a label by index
    pub fn get_label(&self, index: usize) -> Option<u32> {
        self.labels.get(&index).copied()
    }

    /// Get the assembler (mutable)
    pub fn assembler_mut(&mut self) -> &mut Assembler {
        &mut self.assembler
    }

    /// Get the code holder (mutable)
    pub fn code_holder_mut(&mut self) -> &mut CodeHolder {
        self.assembler.code_holder_mut()
    }
    
    /// Get the code holder (immutable)
    pub fn code_holder(&self) -> &CodeHolder {
        self.assembler.code_holder()
    }
}

/// Lambda entry structure
#[derive(Debug, Clone)]
pub struct LambdaEntry {
    pub trampoline: usize,
    pub arity: u32,
    pub index: usize,
}

impl Default for AssemblerState {
    fn default() -> Self {
        Self::new().expect("Failed to create assembler state")
    }
}

