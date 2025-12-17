//! Common assembler functionality
//!
//! Base functionality shared across all architecture-specific assemblers.
//! Converted from C++ `BeamAssemblerCommon` class.

pub mod args;

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
    ) -> Result<(*const u8, *mut u8, usize, Vec<(*const u8, usize)>), BeamAssemblerError>;

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
        args: &[args::ArgVal],
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
    #[allow(dead_code)]
    lambdas: Vec<LambdaEntry>,
    /// Read-only data section
    #[allow(dead_code)]
    rodata: std::collections::HashMap<String, Vec<u8>>,
    /// BSS section
    #[allow(dead_code)]
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
        let label = self.assembler.new_label()
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

    /// Finalize code generation
    pub fn finalize_code(&mut self) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] AssemblerState: Flattening code");
        self.code_holder_mut().flatten()
            .map_err(|e| BeamAssemblerError::CodeGenerationFailed(e.to_string()))?;
        eprintln!("[DEBUG] AssemblerState: Resolving unresolved links");
        self.code_holder_mut().resolve_unresolved_links()
            .map_err(|e| BeamAssemblerError::CodeGenerationFailed(e.to_string()))?;
        eprintln!("[DEBUG] AssemblerState: Code finalization completed");
        Ok(())
    }

    /// Get the code size
    pub fn code_size(&self) -> usize {
        self.code_holder().code_size()
    }

    /// Get the base address of the generated code
    pub fn base_address(&self) -> *const u8 {
        self.code_holder().base_address()
    }
}

/// Lambda entry structure
#[derive(Debug, Clone)]
#[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    // ==================== BeamAssemblerError Tests ====================

    #[test]
    fn test_error_unsupported_architecture_display() {
        let error = BeamAssemblerError::UnsupportedArchitecture;
        let display = format!("{}", error);
        assert!(display.contains("Unsupported architecture"));
    }

    #[test]
    fn test_error_code_generation_failed_display() {
        let error = BeamAssemblerError::CodeGenerationFailed("test failure".to_string());
        let display = format!("{}", error);
        assert!(display.contains("Code generation failed"));
        assert!(display.contains("test failure"));
    }

    #[test]
    fn test_error_jit_allocation_failed_display() {
        let error = BeamAssemblerError::JitAllocationFailed("memory error".to_string());
        let display = format!("{}", error);
        assert!(display.contains("JIT allocation failed"));
        assert!(display.contains("memory error"));
    }

    #[test]
    fn test_error_invalid_label_display() {
        let error = BeamAssemblerError::InvalidLabel;
        let display = format!("{}", error);
        assert!(display.contains("Invalid label"));
    }

    #[test]
    fn test_error_invalid_function_index_display() {
        let error = BeamAssemblerError::InvalidFunctionIndex;
        let display = format!("{}", error);
        assert!(display.contains("Invalid function index"));
    }

    #[test]
    fn test_error_debug() {
        let error = BeamAssemblerError::CodeGenerationFailed("debug test".to_string());
        let debug = format!("{:?}", error);
        assert!(debug.contains("CodeGenerationFailed"));
        assert!(debug.contains("debug test"));
    }

    #[test]
    fn test_error_all_variants_debug() {
        let errors = [
            BeamAssemblerError::UnsupportedArchitecture,
            BeamAssemblerError::CodeGenerationFailed(String::new()),
            BeamAssemblerError::JitAllocationFailed(String::new()),
            BeamAssemblerError::InvalidLabel,
            BeamAssemblerError::InvalidFunctionIndex,
        ];
        for err in &errors {
            let _ = format!("{:?}", err);
        }
    }

    #[test]
    fn test_error_is_std_error() {
        let error: Box<dyn Error> = Box::new(BeamAssemblerError::InvalidLabel);
        let _ = error.to_string();
    }

    #[test]
    fn test_error_source_is_none() {
        let error = BeamAssemblerError::InvalidLabel;
        assert!(error.source().is_none());
    }

    #[test]
    fn test_error_code_generation_empty_message() {
        let error = BeamAssemblerError::CodeGenerationFailed(String::new());
        let display = format!("{}", error);
        assert!(display.contains("Code generation failed"));
    }

    #[test]
    fn test_error_jit_allocation_empty_message() {
        let error = BeamAssemblerError::JitAllocationFailed(String::new());
        let display = format!("{}", error);
        assert!(display.contains("JIT allocation failed"));
    }

    #[test]
    fn test_error_with_special_characters() {
        let error = BeamAssemblerError::CodeGenerationFailed("error: <test> \"special\" chars!".to_string());
        let display = format!("{}", error);
        assert!(display.contains("error: <test> \"special\" chars!"));
    }

    // ==================== Export Tests ====================

    #[test]
    fn test_export_creation() {
        let export = Export {
            module: 123,
            function: 456,
            arity: 2,
            address: std::ptr::null(),
        };
        assert_eq!(export.module, 123);
        assert_eq!(export.function, 456);
        assert_eq!(export.arity, 2);
        assert!(export.address.is_null());
    }

    #[test]
    fn test_export_with_address() {
        let data: u8 = 42;
        let ptr = &data as *const u8;
        let export = Export {
            module: 1,
            function: 2,
            arity: 3,
            address: ptr,
        };
        assert_eq!(export.address, ptr);
    }

    #[test]
    fn test_export_debug() {
        let export = Export {
            module: 100,
            function: 200,
            arity: 1,
            address: std::ptr::null(),
        };
        let debug = format!("{:?}", export);
        assert!(debug.contains("Export"));
        assert!(debug.contains("100"));
        assert!(debug.contains("200"));
        assert!(debug.contains("1"));
    }

    #[test]
    fn test_export_clone() {
        let export = Export {
            module: 10,
            function: 20,
            arity: 3,
            address: std::ptr::null(),
        };
        let cloned = export.clone();
        assert_eq!(export.module, cloned.module);
        assert_eq!(export.function, cloned.function);
        assert_eq!(export.arity, cloned.arity);
        assert_eq!(export.address, cloned.address);
    }

    #[test]
    fn test_export_large_values() {
        let export = Export {
            module: u64::MAX,
            function: u64::MAX,
            arity: u32::MAX,
            address: usize::MAX as *const u8,
        };
        assert_eq!(export.module, u64::MAX);
        assert_eq!(export.function, u64::MAX);
        assert_eq!(export.arity, u32::MAX);
    }

    // ==================== FunEntry Tests ====================

    #[test]
    fn test_fun_entry_creation() {
        let entry = FunEntry {
            address: std::ptr::null(),
            arity: 5,
            index: 10,
        };
        assert!(entry.address.is_null());
        assert_eq!(entry.arity, 5);
        assert_eq!(entry.index, 10);
    }

    #[test]
    fn test_fun_entry_with_address() {
        let data: u8 = 99;
        let ptr = &data as *const u8;
        let entry = FunEntry {
            address: ptr,
            arity: 2,
            index: 0,
        };
        assert_eq!(entry.address, ptr);
    }

    #[test]
    fn test_fun_entry_debug() {
        let entry = FunEntry {
            address: std::ptr::null(),
            arity: 3,
            index: 7,
        };
        let debug = format!("{:?}", entry);
        assert!(debug.contains("FunEntry"));
        assert!(debug.contains("3"));
        assert!(debug.contains("7"));
    }

    #[test]
    fn test_fun_entry_clone() {
        let entry = FunEntry {
            address: std::ptr::null(),
            arity: 4,
            index: 8,
        };
        let cloned = entry.clone();
        assert_eq!(entry.address, cloned.address);
        assert_eq!(entry.arity, cloned.arity);
        assert_eq!(entry.index, cloned.index);
    }

    #[test]
    fn test_fun_entry_large_values() {
        let entry = FunEntry {
            address: usize::MAX as *const u8,
            arity: u32::MAX,
            index: usize::MAX,
        };
        assert_eq!(entry.arity, u32::MAX);
        assert_eq!(entry.index, usize::MAX);
    }

    // ==================== LambdaEntry Tests ====================

    #[test]
    fn test_lambda_entry_creation() {
        let entry = LambdaEntry {
            trampoline: 100,
            arity: 2,
            index: 5,
        };
        assert_eq!(entry.trampoline, 100);
        assert_eq!(entry.arity, 2);
        assert_eq!(entry.index, 5);
    }

    #[test]
    fn test_lambda_entry_debug() {
        let entry = LambdaEntry {
            trampoline: 50,
            arity: 1,
            index: 3,
        };
        let debug = format!("{:?}", entry);
        assert!(debug.contains("LambdaEntry"));
        assert!(debug.contains("50"));
        assert!(debug.contains("1"));
        assert!(debug.contains("3"));
    }

    #[test]
    fn test_lambda_entry_clone() {
        let entry = LambdaEntry {
            trampoline: 200,
            arity: 4,
            index: 9,
        };
        let cloned = entry.clone();
        assert_eq!(entry.trampoline, cloned.trampoline);
        assert_eq!(entry.arity, cloned.arity);
        assert_eq!(entry.index, cloned.index);
    }

    #[test]
    fn test_lambda_entry_large_values() {
        let entry = LambdaEntry {
            trampoline: usize::MAX,
            arity: u32::MAX,
            index: usize::MAX,
        };
        assert_eq!(entry.trampoline, usize::MAX);
        assert_eq!(entry.arity, u32::MAX);
        assert_eq!(entry.index, usize::MAX);
    }

    #[test]
    fn test_lambda_entry_zero_values() {
        let entry = LambdaEntry {
            trampoline: 0,
            arity: 0,
            index: 0,
        };
        assert_eq!(entry.trampoline, 0);
        assert_eq!(entry.arity, 0);
        assert_eq!(entry.index, 0);
    }

    // ==================== BeamAssembler Trait Tests ====================

    #[test]
    fn test_beam_assembler_trait_is_object_safe() {
        // This test verifies that BeamAssembler is object-safe
        // by ensuring we can use it as a trait object type
        fn _accepts_dyn_beam_assembler(_asm: &dyn BeamAssembler) {}
    }

    #[test]
    fn test_beam_assembler_trait_is_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        
        // Can't directly test the trait bounds, but we can test
        // that a type implementing it would need to be Send + Sync
        fn _accepts_send_sync<T: BeamAssembler>() {
            assert_send::<T>();
            assert_sync::<T>();
        }
    }

    // ==================== args Module Re-export Tests ====================

    #[test]
    fn test_args_module_accessible() {
        // Verify args module is accessible through mod.rs
        let arg = args::ArgVal::word(42);
        assert_eq!(arg.value(), 42);
    }

    #[test]
    fn test_args_argtype_accessible() {
        let arg = args::ArgVal::x_reg(5);
        assert_eq!(arg.tag_type(), args::ArgType::XReg);
    }

    // ==================== Pointer Safety Tests ====================

    #[test]
    fn test_export_null_address_is_safe() {
        let export = Export {
            module: 0,
            function: 0,
            arity: 0,
            address: std::ptr::null(),
        };
        assert!(export.address.is_null());
    }

    #[test]
    fn test_fun_entry_null_address_is_safe() {
        let entry = FunEntry {
            address: std::ptr::null(),
            arity: 0,
            index: 0,
        };
        assert!(entry.address.is_null());
    }

    // ==================== Collection Type Tests ====================

    #[test]
    fn test_exports_in_vec() {
        let exports: Vec<Export> = vec![
            Export {
                module: 1,
                function: 2,
                arity: 0,
                address: std::ptr::null(),
            },
            Export {
                module: 3,
                function: 4,
                arity: 1,
                address: std::ptr::null(),
            },
        ];
        assert_eq!(exports.len(), 2);
        assert_eq!(exports[0].module, 1);
        assert_eq!(exports[1].module, 3);
    }

    #[test]
    fn test_fun_entries_in_vec() {
        let entries: Vec<FunEntry> = vec![
            FunEntry {
                address: std::ptr::null(),
                arity: 0,
                index: 0,
            },
            FunEntry {
                address: std::ptr::null(),
                arity: 1,
                index: 1,
            },
        ];
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].index, 0);
        assert_eq!(entries[1].index, 1);
    }

    #[test]
    fn test_lambda_entries_in_vec() {
        let entries: Vec<LambdaEntry> = vec![
            LambdaEntry {
                trampoline: 100,
                arity: 2,
                index: 0,
            },
            LambdaEntry {
                trampoline: 200,
                arity: 3,
                index: 1,
            },
        ];
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].trampoline, 100);
        assert_eq!(entries[1].trampoline, 200);
    }
}

