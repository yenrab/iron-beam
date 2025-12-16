//! aarch64 BeamAssembler implementation
//!
//! Main assembler for aarch64 architecture.
//! Converted from C++ BeamAssembler class in beam_asm.hpp.

use crate::common::{BeamAssembler, BeamAssemblerError, AssemblerState, Export, FunEntry, args::ArgVal};
use crate::jit::JitAllocator;

/// aarch64 BeamAssembler
///
/// Architecture-specific assembler for aarch64.
pub struct ArmBeamAssembler {
    /// Common assembler state
    state: AssemblerState,
    /// Module atom
    #[allow(dead_code)]
    module: u64, // Eterm
    /// Number of labels
    #[allow(dead_code)]
    num_labels: usize,
    /// Number of functions
    #[allow(dead_code)]
    num_functions: usize,
}

impl ArmBeamAssembler {
    /// Create a new aarch64 assembler
    pub fn new(
        module: u64,
        num_labels: usize,
        num_functions: usize,
        _beam_file: &[u8],
    ) -> Result<Self, BeamAssemblerError> {
        Ok(Self {
            state: AssemblerState::new()?,
            module,
            num_labels,
            num_functions,
        })
    }
}

impl BeamAssembler for ArmBeamAssembler {
    fn get_base_address(&self) -> *const u8 {
        self.state.code_holder().base_address()
    }

    fn get_offset(&self) -> usize {
        // Note: This requires mutable access but trait only provides &self
        // In actual implementation, offset would be tracked separately
        0 // Placeholder
    }

    fn codegen(
        &mut self,
        allocator: &mut JitAllocator,
    ) -> Result<(*const u8, *mut u8), BeamAssemblerError> {
        // Flatten and resolve links
        self.state.code_holder_mut().flatten()
            .map_err(|e| BeamAssemblerError::CodeGenerationFailed(format!("Flatten failed: {:?}", e)))?;
        self.state.code_holder_mut().resolve_unresolved_links()
            .map_err(|e| BeamAssemblerError::CodeGenerationFailed(format!("Resolve links failed: {:?}", e)))?;
        
        // Get code size and allocate memory
        let code_size = self.state.code_holder().code_size();
        let (executable, writable, _) = allocator.allocate(code_size)
            .map_err(|e| BeamAssemblerError::JitAllocationFailed(e.to_string()))?;
        
        // Copy code to executable memory
        unsafe {
            let base = self.state.code_holder().base_address();
            if !base.is_null() {
                std::ptr::copy_nonoverlapping(base, writable, code_size);
            }
        }
        
        Ok((executable, writable))
    }

    fn get_code(&self, _label: usize) -> Result<*const u8, BeamAssemblerError> {
        Err(BeamAssemblerError::InvalidLabel)
    }

    fn get_lambda(&self, _index: usize) -> Result<*const u8, BeamAssemblerError> {
        Err(BeamAssemblerError::InvalidFunctionIndex)
    }

    fn get_rodata(&self, _label: &str) -> Option<*const u8> {
        None
    }

    fn embed_rodata(
        &mut self,
        _label: &str,
        _data: &[u8],
    ) -> Result<(), BeamAssemblerError> {
        Ok(())
    }

    fn embed_bss(&mut self, _label: &str, _size: usize) -> Result<(), BeamAssemblerError> {
        Ok(())
    }

    fn emit(
        &mut self,
        _opcode: u32,
        _args: &[ArgVal],
    ) -> Result<(), BeamAssemblerError> {
        Ok(())
    }

    fn patch_catches(&mut self, _rw_base: *mut u8) -> Result<usize, BeamAssemblerError> {
        Ok(0)
    }

    fn patch_import(
        &mut self,
        _rw_base: *mut u8,
        _index: usize,
        _export: &Export,
    ) -> Result<(), BeamAssemblerError> {
        Ok(())
    }

    fn patch_literal(
        &mut self,
        _rw_base: *mut u8,
        _index: usize,
        _literal: u64,
    ) -> Result<(), BeamAssemblerError> {
        Ok(())
    }

    fn patch_lambda(
        &mut self,
        _rw_base: *mut u8,
        _index: usize,
        _fun_entry: &FunEntry,
    ) -> Result<(), BeamAssemblerError> {
        Ok(())
    }

    fn patch_strings(
        &mut self,
        _rw_base: *mut u8,
        _strtab: &[u8],
    ) -> Result<(), BeamAssemblerError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::args::ArgVal;
    use crate::jit::JitAllocator;

    #[test]
    fn test_arm_assembler_new() {
        let module = 0x12345678;
        let num_labels = 10;
        let num_functions = 5;
        let beam_file = b"BEAM";

        let result = ArmBeamAssembler::new(module, num_labels, num_functions, beam_file);
        assert!(result.is_ok());

        let assembler = result.unwrap();
        assert_eq!(assembler.module, module);
        assert_eq!(assembler.num_labels, num_labels);
        assert_eq!(assembler.num_functions, num_functions);
    }

    #[test]
    fn test_get_base_address() {
        let assembler = ArmBeamAssembler::new(0, 0, 0, b"").unwrap();
        let base = assembler.get_base_address();
        // Base address may be null initially, but should not crash
        let _ = base;
    }

    #[test]
    fn test_get_offset() {
        let assembler = ArmBeamAssembler::new(0, 0, 0, b"").unwrap();
        let offset = assembler.get_offset();
        assert_eq!(offset, 0); // Currently returns placeholder 0
    }

    #[test]
    fn test_codegen() {
        let mut assembler = ArmBeamAssembler::new(0, 0, 0, b"").unwrap();
        let mut allocator = JitAllocator::new().unwrap();

        let result = assembler.codegen(&mut allocator);
        // Codegen may fail if code_size is 0 (allocator requires non-zero size)
        // This is expected behavior - we test that the function handles it correctly
        match result {
            Ok((executable, writable)) => {
                // If successful, both pointers should be valid
                assert!(!executable.is_null());
                assert!(!writable.is_null());
            }
            Err(BeamAssemblerError::JitAllocationFailed(_)) => {
                // This is acceptable when code_size is 0
            }
            Err(e) => {
                panic!("Unexpected error: {:?}", e);
            }
        }
    }

    #[test]
    fn test_get_code() {
        let assembler = ArmBeamAssembler::new(0, 0, 0, b"").unwrap();
        
        // Should return InvalidLabel error for any label
        let result = assembler.get_code(0);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BeamAssemblerError::InvalidLabel));

        let result = assembler.get_code(42);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BeamAssemblerError::InvalidLabel));
    }

    #[test]
    fn test_get_lambda() {
        let assembler = ArmBeamAssembler::new(0, 0, 0, b"").unwrap();
        
        // Should return InvalidFunctionIndex error for any index
        let result = assembler.get_lambda(0);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BeamAssemblerError::InvalidFunctionIndex));

        let result = assembler.get_lambda(10);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BeamAssemblerError::InvalidFunctionIndex));
    }

    #[test]
    fn test_get_rodata() {
        let assembler = ArmBeamAssembler::new(0, 0, 0, b"").unwrap();
        
        // Should return None for any label
        assert!(assembler.get_rodata("test_label").is_none());
        assert!(assembler.get_rodata("").is_none());
        assert!(assembler.get_rodata("another_label").is_none());
    }

    #[test]
    fn test_embed_rodata() {
        let mut assembler = ArmBeamAssembler::new(0, 0, 0, b"").unwrap();
        
        // Should succeed (currently a no-op)
        let data = b"test data";
        let result = assembler.embed_rodata("test_label", data);
        assert!(result.is_ok());

        let empty_data = b"";
        let result = assembler.embed_rodata("empty_label", empty_data);
        assert!(result.is_ok());

        let large_data = vec![0u8; 1024];
        let result = assembler.embed_rodata("large_label", &large_data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_embed_bss() {
        let mut assembler = ArmBeamAssembler::new(0, 0, 0, b"").unwrap();
        
        // Should succeed (currently a no-op)
        let result = assembler.embed_bss("test_bss", 1024);
        assert!(result.is_ok());

        let result = assembler.embed_bss("empty_bss", 0);
        assert!(result.is_ok());

        let result = assembler.embed_bss("large_bss", 10000);
        assert!(result.is_ok());
    }

    #[test]
    fn test_emit() {
        let mut assembler = ArmBeamAssembler::new(0, 0, 0, b"").unwrap();
        
        // Should succeed (currently a no-op)
        let args = vec![ArgVal::word(42)];
        let result = assembler.emit(0, &args);
        assert!(result.is_ok());

        let args = vec![ArgVal::x_reg(5), ArgVal::word(10)];
        let result = assembler.emit(1, &args);
        assert!(result.is_ok());

        let empty_args = vec![];
        let result = assembler.emit(100, &empty_args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_patch_catches() {
        let mut assembler = ArmBeamAssembler::new(0, 0, 0, b"").unwrap();
        
        // Test with null pointer (patch_catches should handle this)
        let result = assembler.patch_catches(std::ptr::null_mut());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        
        // If codegen succeeds, test with actual pointer
        let mut allocator = JitAllocator::new().unwrap();
        if let Ok((_executable, writable)) = assembler.codegen(&mut allocator) {
            let result = assembler.patch_catches(writable);
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), 0);
        }
    }

    #[test]
    fn test_patch_import() {
        let mut assembler = ArmBeamAssembler::new(0, 0, 0, b"").unwrap();
        
        let export = Export {
            module: 0x1234,
            function: 0x5678,
            arity: 2,
            address: std::ptr::null(),
        };
        
        // Test with null pointer (patch_import should handle this)
        let result = assembler.patch_import(std::ptr::null_mut(), 0, &export);
        assert!(result.is_ok());

        // If codegen succeeds, test with actual pointer
        let mut allocator = JitAllocator::new().unwrap();
        if let Ok((_executable, writable)) = assembler.codegen(&mut allocator) {
            let result = assembler.patch_import(writable, 0, &export);
            assert!(result.is_ok());

            let result = assembler.patch_import(writable, 10, &export);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_patch_literal() {
        let mut assembler = ArmBeamAssembler::new(0, 0, 0, b"").unwrap();
        
        // Test with null pointer (patch_literal should handle this)
        let result = assembler.patch_literal(std::ptr::null_mut(), 0, 0x12345678);
        assert!(result.is_ok());

        // If codegen succeeds, test with actual pointer
        let mut allocator = JitAllocator::new().unwrap();
        if let Ok((_executable, writable)) = assembler.codegen(&mut allocator) {
            let result = assembler.patch_literal(writable, 0, 0x12345678);
            assert!(result.is_ok());

            let result = assembler.patch_literal(writable, 5, 0xABCDEF00);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_patch_lambda() {
        let mut assembler = ArmBeamAssembler::new(0, 0, 0, b"").unwrap();
        
        let fun_entry = FunEntry {
            address: std::ptr::null(),
            arity: 3,
            index: 0,
        };
        
        // Test with null pointer (patch_lambda should handle this)
        let result = assembler.patch_lambda(std::ptr::null_mut(), 0, &fun_entry);
        assert!(result.is_ok());

        // If codegen succeeds, test with actual pointer
        let mut allocator = JitAllocator::new().unwrap();
        if let Ok((_executable, writable)) = assembler.codegen(&mut allocator) {
            let result = assembler.patch_lambda(writable, 0, &fun_entry);
            assert!(result.is_ok());

            let result = assembler.patch_lambda(writable, 10, &fun_entry);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_patch_strings() {
        let mut assembler = ArmBeamAssembler::new(0, 0, 0, b"").unwrap();
        
        // Test with null pointer (patch_strings should handle this)
        let strtab = b"test string table";
        let result = assembler.patch_strings(std::ptr::null_mut(), strtab);
        assert!(result.is_ok());

        let empty_strtab = b"";
        let result = assembler.patch_strings(std::ptr::null_mut(), empty_strtab);
        assert!(result.is_ok());

        // If codegen succeeds, test with actual pointer
        let mut allocator = JitAllocator::new().unwrap();
        if let Ok((_executable, writable)) = assembler.codegen(&mut allocator) {
            let result = assembler.patch_strings(writable, strtab);
            assert!(result.is_ok());

            let empty_strtab = b"";
            let result = assembler.patch_strings(writable, empty_strtab);
            assert!(result.is_ok());

            let large_strtab = vec![0u8; 1024];
            let result = assembler.patch_strings(writable, &large_strtab);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_multiple_operations() {
        let mut assembler = ArmBeamAssembler::new(0xABCD, 20, 10, b"BEAM").unwrap();
        
        // Test multiple operations in sequence
        assert_eq!(assembler.get_offset(), 0);
        
        let args = vec![ArgVal::word(1), ArgVal::x_reg(2)];
        assert!(assembler.emit(0, &args).is_ok());
        
        assert!(assembler.embed_rodata("label1", b"data1").is_ok());
        assert!(assembler.embed_bss("bss1", 100).is_ok());
        
        // Codegen may fail if code_size is 0, which is acceptable
        let mut allocator = JitAllocator::new().unwrap();
        let _ = assembler.codegen(&mut allocator);
    }

    #[test]
    fn test_assembler_state_preservation() {
        let assembler1 = ArmBeamAssembler::new(0x1111, 5, 3, b"").unwrap();
        let assembler2 = ArmBeamAssembler::new(0x2222, 10, 7, b"").unwrap();
        
        // Each assembler should maintain its own state
        assert_eq!(assembler1.module, 0x1111);
        assert_eq!(assembler1.num_labels, 5);
        assert_eq!(assembler1.num_functions, 3);
        
        assert_eq!(assembler2.module, 0x2222);
        assert_eq!(assembler2.num_labels, 10);
        assert_eq!(assembler2.num_functions, 7);
    }
}

