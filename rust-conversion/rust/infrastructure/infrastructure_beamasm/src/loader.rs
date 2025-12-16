//! Code loading functionality
//!
//! Handles loading of BEAM modules and preparation for JIT compilation.
//! Converted from C code in asm_load.c.

use crate::common::{BeamAssembler, BeamAssemblerError};
use crate::jit::JitAllocator;
use thiserror::Error;

/// Errors that can occur during code loading
#[derive(Debug, Error)]
pub enum LoaderError {
    #[error("Assembler error: {0}")]
    AssemblerError(#[from] BeamAssemblerError),
    #[error("Invalid BEAM file format")]
    InvalidBeamFile,
    #[error("Module initialization failed")]
    ModuleInitFailed,
}

/// Loader state
///
/// Maintains state during code loading process.
/// Converted from C `LoaderState` structure.
pub struct LoaderState {
    /// Module atom
    pub module: u64, // Eterm
    /// Assembler instance
    pub assembler: Box<dyn BeamAssembler>,
    /// Code header
    pub code_header: *mut u8,
    /// Labels
    pub labels: Vec<Label>,
    /// Lambda literals
    pub lambda_literals: Vec<i64>,
    /// Coverage data
    pub coverage: Option<*mut u8>,
    /// Line coverage valid flags
    pub line_coverage_valid: Option<*mut u8>,
    /// Location index to cover ID mapping
    pub loc_index_to_cover_id: Option<*mut u32>,
}

/// Label structure
#[derive(Debug, Clone)]
pub struct Label {
    /// Label index
    pub index: usize,
    /// Code pointer (set after code generation)
    pub code_ptr: Option<*const u8>,
}

/// BeamAsm loader
///
/// Main loader interface for BEAM code loading.
pub struct BeamAsmLoader {
    /// JIT allocator
    allocator: JitAllocator,
}

impl BeamAsmLoader {
    /// Create a new loader
    pub fn new() -> Result<Self, LoaderError> {
        Ok(Self {
            allocator: JitAllocator::new()
                .map_err(|e| LoaderError::AssemblerError(BeamAssemblerError::JitAllocationFailed(e.to_string())))?,
        })
    }

    /// Prepare for code emission
    ///
    /// Sets up the assembler and initializes code header.
    /// Converted from `beam_load_prepare_emit`.
    pub fn prepare_emit(
        &mut self,
        module: u64,
        num_labels: usize,
        num_functions: usize,
        beam_file: &[u8],
    ) -> Result<LoaderState, LoaderError> {
        // Create assembler
        let assembler = crate::beamasm_new_assembler(module, num_labels, num_functions, beam_file)
            .map_err(LoaderError::AssemblerError)?;

        // Initialize labels
        let labels = (0..num_labels)
            .map(|i| Label {
                index: i,
                code_ptr: None,
            })
            .collect();

        // Initialize lambda literals
        let lambda_literals = Vec::new();

        Ok(LoaderState {
            module,
            assembler,
            code_header: std::ptr::null_mut(),
            labels,
            lambda_literals,
            coverage: None,
            line_coverage_valid: None,
            loc_index_to_cover_id: None,
        })
    }

    /// Generate code
    ///
    /// Generates native code from BEAM instructions.
    /// Converted from `beam_load_finish_emit`.
    pub fn finish_emit(
        &mut self,
        state: &mut LoaderState,
    ) -> Result<(*const u8, *mut u8, usize, Vec<(*const u8, usize)>), LoaderError> {
        // Generate code using the assembler
        let (executable, writable, size, label_mappings) = state.assembler.codegen(&mut self.allocator)
            .map_err(LoaderError::AssemblerError)?;

        // Update label code pointers from assembler mappings
        for (code_ptr, label_index) in &label_mappings {
            if *label_index < state.labels.len() {
                state.labels[*label_index].code_ptr = Some(*code_ptr);
            }
        }

        Ok((executable, writable, size, label_mappings))
    }

    /// Patch code
    ///
    /// Patches code with imports, literals, lambdas, and strings.
    /// Converted from `beam_load_patch`.
    pub fn patch(
        &mut self,
        state: &mut LoaderState,
        rw_base: *mut u8,
    ) -> Result<(), LoaderError> {
        // Patch catches
        state.assembler.patch_catches(rw_base)
            .map_err(LoaderError::AssemblerError)?;

        // Patch imports, literals, lambdas, and strings would be done here
        // This is a placeholder - actual implementation would iterate through
        // imports, literals, lambdas, and strings and call the appropriate
        // patch methods.

        Ok(())
    }

    /// Purge a module
    ///
    /// Deallocates memory for a module that is no longer needed.
    /// Converted from `beamasm_purge_module`.
    pub fn purge_module(
        &mut self,
        executable: *const u8,
        writable: *mut u8,
        size: usize,
    ) {
        self.allocator.purge_module(executable, writable, size);
    }
}

impl Default for BeamAsmLoader {
    fn default() -> Self {
        Self::new().expect("Failed to create loader")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    // ==================== LoaderError Tests ====================

    #[test]
    fn test_error_assembler_error_display() {
        let inner = BeamAssemblerError::InvalidLabel;
        let error = LoaderError::AssemblerError(inner);
        let display = format!("{}", error);
        assert!(display.contains("Assembler error"));
    }

    #[test]
    fn test_error_invalid_beam_file_display() {
        let error = LoaderError::InvalidBeamFile;
        let display = format!("{}", error);
        assert!(display.contains("Invalid BEAM file format"));
    }

    #[test]
    fn test_error_module_init_failed_display() {
        let error = LoaderError::ModuleInitFailed;
        let display = format!("{}", error);
        assert!(display.contains("Module initialization failed"));
    }

    #[test]
    fn test_error_debug() {
        let error = LoaderError::InvalidBeamFile;
        let debug = format!("{:?}", error);
        assert!(debug.contains("InvalidBeamFile"));
    }

    #[test]
    fn test_error_all_variants_debug() {
        let errors = [
            LoaderError::AssemblerError(BeamAssemblerError::InvalidLabel),
            LoaderError::InvalidBeamFile,
            LoaderError::ModuleInitFailed,
        ];
        for err in &errors {
            let _ = format!("{:?}", err);
            let _ = format!("{}", err);
        }
    }

    #[test]
    fn test_error_is_std_error() {
        let error: Box<dyn Error> = Box::new(LoaderError::InvalidBeamFile);
        let _ = error.to_string();
    }

    #[test]
    fn test_error_from_beam_assembler_error() {
        let beam_error = BeamAssemblerError::CodeGenerationFailed("test".to_string());
        let loader_error: LoaderError = beam_error.into();
        let display = format!("{}", loader_error);
        assert!(display.contains("Assembler error"));
        assert!(display.contains("test"));
    }

    #[test]
    fn test_error_from_various_beam_assembler_errors() {
        let errors = [
            BeamAssemblerError::UnsupportedArchitecture,
            BeamAssemblerError::CodeGenerationFailed(String::new()),
            BeamAssemblerError::JitAllocationFailed(String::new()),
            BeamAssemblerError::InvalidLabel,
            BeamAssemblerError::InvalidFunctionIndex,
        ];
        
        for beam_err in errors {
            let loader_err: LoaderError = beam_err.into();
            let _ = format!("{}", loader_err);
        }
    }

    #[test]
    fn test_error_source() {
        let error = LoaderError::InvalidBeamFile;
        // InvalidBeamFile has no source
        assert!(error.source().is_none());
    }

    #[test]
    fn test_error_assembler_error_source() {
        let inner = BeamAssemblerError::InvalidLabel;
        let error = LoaderError::AssemblerError(inner);
        // AssemblerError wraps another error, check source
        let source = error.source();
        assert!(source.is_some());
    }

    // ==================== Label Tests ====================

    #[test]
    fn test_label_creation() {
        let label = Label {
            index: 42,
            code_ptr: None,
        };
        assert_eq!(label.index, 42);
        assert!(label.code_ptr.is_none());
    }

    #[test]
    fn test_label_with_code_ptr() {
        let data: u8 = 0x90;
        let ptr = &data as *const u8;
        let label = Label {
            index: 10,
            code_ptr: Some(ptr),
        };
        assert_eq!(label.index, 10);
        assert_eq!(label.code_ptr, Some(ptr));
    }

    #[test]
    fn test_label_debug() {
        let label = Label {
            index: 5,
            code_ptr: None,
        };
        let debug = format!("{:?}", label);
        assert!(debug.contains("Label"));
        assert!(debug.contains("5"));
        assert!(debug.contains("None"));
    }

    #[test]
    fn test_label_clone() {
        let label = Label {
            index: 100,
            code_ptr: None,
        };
        let cloned = label.clone();
        assert_eq!(label.index, cloned.index);
        assert_eq!(label.code_ptr, cloned.code_ptr);
    }

    #[test]
    fn test_label_clone_with_ptr() {
        let data: u8 = 0xC3;
        let ptr = &data as *const u8;
        let label = Label {
            index: 50,
            code_ptr: Some(ptr),
        };
        let cloned = label.clone();
        assert_eq!(label.code_ptr, cloned.code_ptr);
    }

    #[test]
    fn test_label_zero_index() {
        let label = Label {
            index: 0,
            code_ptr: None,
        };
        assert_eq!(label.index, 0);
    }

    #[test]
    fn test_label_max_index() {
        let label = Label {
            index: usize::MAX,
            code_ptr: None,
        };
        assert_eq!(label.index, usize::MAX);
    }

    #[test]
    fn test_label_null_ptr() {
        let label = Label {
            index: 0,
            code_ptr: Some(std::ptr::null()),
        };
        assert!(label.code_ptr.unwrap().is_null());
    }

    #[test]
    fn test_labels_in_vec() {
        let labels: Vec<Label> = (0..10)
            .map(|i| Label {
                index: i,
                code_ptr: None,
            })
            .collect();
        
        assert_eq!(labels.len(), 10);
        for (i, label) in labels.iter().enumerate() {
            assert_eq!(label.index, i);
        }
    }

    // ==================== LoaderState Tests ====================

    #[test]
    fn test_loader_state_fields() {
        // We can't easily create a LoaderState without a real assembler,
        // but we can test the Label struct which is part of it
        let labels: Vec<Label> = vec![
            Label { index: 0, code_ptr: None },
            Label { index: 1, code_ptr: None },
        ];
        assert_eq!(labels.len(), 2);
    }

    #[test]
    fn test_loader_state_lambda_literals() {
        let lambda_literals: Vec<i64> = vec![1, 2, 3, -1, -2];
        assert_eq!(lambda_literals.len(), 5);
        assert_eq!(lambda_literals[3], -1);
    }

    // ==================== BeamAsmLoader Tests ====================

    #[test]
    fn test_beam_asm_loader_new() {
        let loader = BeamAsmLoader::new();
        assert!(loader.is_ok());
    }

    #[test]
    fn test_beam_asm_loader_default() {
        let loader = BeamAsmLoader::default();
        // Should not panic
        let _ = loader;
    }

    #[test]
    fn test_beam_asm_loader_multiple_instances() {
        let loader1 = BeamAsmLoader::new().unwrap();
        let loader2 = BeamAsmLoader::new().unwrap();
        let loader3 = BeamAsmLoader::default();
        // All should coexist
        let _ = (loader1, loader2, loader3);
    }

    #[test]
    fn test_beam_asm_loader_purge_module_null() {
        let mut loader = BeamAsmLoader::new().unwrap();
        // Purging null pointers should not panic
        loader.purge_module(std::ptr::null(), std::ptr::null_mut(), 0);
    }

    // ==================== Integration-like Tests ====================

    #[test]
    fn test_label_update_code_ptr() {
        let mut label = Label {
            index: 0,
            code_ptr: None,
        };
        
        assert!(label.code_ptr.is_none());
        
        let data: [u8; 4] = [0x90, 0x90, 0x90, 0xC3];
        label.code_ptr = Some(data.as_ptr());
        
        assert!(label.code_ptr.is_some());
        assert_eq!(label.code_ptr.unwrap(), data.as_ptr());
    }

    #[test]
    fn test_labels_update_batch() {
        let mut labels: Vec<Label> = (0..5)
            .map(|i| Label {
                index: i,
                code_ptr: None,
            })
            .collect();
        
        let code: [u8; 100] = [0x90; 100];
        
        // Simulate updating labels with code pointers
        for (i, label) in labels.iter_mut().enumerate() {
            unsafe {
                label.code_ptr = Some(code.as_ptr().add(i * 10));
            }
        }
        
        // Verify all labels have code pointers now
        for label in &labels {
            assert!(label.code_ptr.is_some());
        }
    }

    // ==================== Error Conversion Tests ====================

    #[test]
    fn test_loader_error_from_jit_allocation_failed() {
        let beam_err = BeamAssemblerError::JitAllocationFailed("memory exhausted".to_string());
        let loader_err: LoaderError = beam_err.into();
        let display = format!("{}", loader_err);
        assert!(display.contains("memory exhausted"));
    }

    #[test]
    fn test_loader_error_from_code_generation_failed() {
        let beam_err = BeamAssemblerError::CodeGenerationFailed("invalid instruction".to_string());
        let loader_err: LoaderError = beam_err.into();
        let display = format!("{}", loader_err);
        assert!(display.contains("invalid instruction"));
    }

    #[test]
    fn test_loader_error_from_unsupported_architecture() {
        let beam_err = BeamAssemblerError::UnsupportedArchitecture;
        let loader_err: LoaderError = beam_err.into();
        let display = format!("{}", loader_err);
        assert!(display.contains("Unsupported architecture"));
    }

    // ==================== Pointer Tests ====================

    #[test]
    fn test_label_code_ptr_high_address() {
        let high_addr = usize::MAX as *const u8;
        let label = Label {
            index: 0,
            code_ptr: Some(high_addr),
        };
        assert_eq!(label.code_ptr, Some(high_addr));
    }

    #[test]
    fn test_multiple_labels_different_ptrs() {
        let code1: u8 = 0x90;
        let code2: u8 = 0xC3;
        let code3: u8 = 0x00;
        
        let labels = vec![
            Label { index: 0, code_ptr: Some(&code1 as *const u8) },
            Label { index: 1, code_ptr: Some(&code2 as *const u8) },
            Label { index: 2, code_ptr: Some(&code3 as *const u8) },
        ];
        
        // All pointers should be different
        assert_ne!(labels[0].code_ptr, labels[1].code_ptr);
        assert_ne!(labels[1].code_ptr, labels[2].code_ptr);
        assert_ne!(labels[0].code_ptr, labels[2].code_ptr);
    }

    // ==================== LoaderState Field Tests ====================

    #[test]
    fn test_coverage_option() {
        let coverage: Option<*mut u8> = None;
        assert!(coverage.is_none());
        
        let mut data: u8 = 0;
        let coverage: Option<*mut u8> = Some(&mut data as *mut u8);
        assert!(coverage.is_some());
    }

    #[test]
    fn test_loc_index_to_cover_id_option() {
        let mapping: Option<*mut u32> = None;
        assert!(mapping.is_none());
        
        let mut data: u32 = 42;
        let mapping: Option<*mut u32> = Some(&mut data as *mut u32);
        assert!(mapping.is_some());
    }

    #[test]
    fn test_line_coverage_valid_option() {
        let valid: Option<*mut u8> = None;
        assert!(valid.is_none());
    }
}

