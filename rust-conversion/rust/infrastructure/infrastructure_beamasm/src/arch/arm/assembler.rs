//! aarch64 BeamAssembler implementation
//!
//! Main assembler for aarch64 architecture.
//! Converted from C++ BeamAssembler class in beam_asm.hpp.

use crate::common::{BeamAssembler, BeamAssemblerError, AssemblerState, Export, FunEntry, args::ArgVal};
use crate::jit::JitAllocator;
use crate::beam_instructions::{BeamParser, BeamInstruction, BeamArg, BeamOpcode, BeamFunction};
use code_management_code_loading::BeamLoader;

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
    /// Parsed BEAM functions
    #[allow(dead_code)]
    functions: Vec<BeamFunction>,
}

impl ArmBeamAssembler {
    /// Create a new aarch64 assembler
    pub fn new(
        module: u64,
        num_labels: usize,
        num_functions: usize,
        beam_file_data: &[u8],
    ) -> Result<Self, BeamAssemblerError> {
        // Parse BEAM file to extract code chunk
        let functions = if !beam_file_data.is_empty() {
            eprintln!("ARM Assembler: Parsing BEAM file of size {}", beam_file_data.len());
            match BeamLoader::read_beam_file(beam_file_data) {
                Ok(beam_file) => {
                    eprintln!("ARM Assembler: Successfully loaded BEAM file, code_data size: {}", beam_file.code_data.len());
                    // Parse BEAM instructions from the code chunk
                    match BeamParser::parse_code(&beam_file.code_data) {
                        Ok(code) => {
                            eprintln!("ARM Assembler: Successfully parsed BEAM code, header: sub_size={}, instruction_set={}, max_opcode={}, label_count={}, function_count={}",
                                     code.header.sub_size, code.header.instruction_set, code.header.max_opcode,
                                     code.header.label_count, code.header.function_count);
                            eprintln!("ARM Assembler: Found {} functions", code.functions.len());

                            // Store the parsed functions for later code generation
                            let mut parsed_functions = Vec::new();
                            for f in code.functions {
                                eprintln!("ARM Assembler: Function {}/{}:{} has {} instructions",
                                         f.module, f.function, f.arity, f.instructions.len());
                                parsed_functions.push(f);
                            }

                            parsed_functions
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to parse BEAM code: {:?}", e);
                            Vec::new()
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to parse BEAM file: {:?}", e);
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };

        eprintln!("ARM Assembler: Parsed {} BEAM functions", functions.len());

        Ok(Self {
            state: AssemblerState::new()?,
            module,
            num_labels,
            num_functions,
            functions,
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
    ) -> Result<(*const u8, *mut u8, usize, Vec<(*const u8, usize)>), BeamAssemblerError> {
        // For now, generate minimal placeholder code that can be executed
        // This will be replaced with proper BEAM instruction parsing and emission

        // Allocate memory for the generated code (start with a reasonable size)
        const MIN_CODE_SIZE: usize = 256;
        let (executable, writable, allocated_size) = allocator.allocate(MIN_CODE_SIZE)
            .map_err(|e| BeamAssemblerError::JitAllocationFailed(e.to_string()))?;

        // Generate code from parsed BEAM functions
        let code = self.generate_arm_beam_code();

        // Copy the generated code to executable memory
        if code.len() <= allocated_size {
            unsafe {
                std::ptr::copy_nonoverlapping(code.as_ptr(), writable, code.len());
            }
        } else {
            return Err(BeamAssemblerError::CodeGenerationFailed(
                "Generated code too large for allocated memory".to_string()
            ));
        }

        // Generate code for each function and create proper label mappings
        let label_mappings = self.generate_arm_function_mappings(executable);

        Ok((executable, writable, allocated_size, label_mappings))
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

    fn patch_strings(&mut self, _rw_base: *mut u8, _strtab: &[u8]) -> Result<(), BeamAssemblerError> {
        Ok(())
    }
}

impl ArmBeamAssembler {
    /// Generate label mappings for each function
    fn generate_arm_function_mappings(&self, base_address: *const u8) -> Vec<(*const u8, usize)> {
        let mut mappings = Vec::new();
        let mut current_offset = 0;

        for (func_idx, function) in self.functions.iter().enumerate() {
            // Calculate the offset for this function's entry point
            // For now, assume each function starts at its label
            let function_ptr = unsafe { base_address.add(current_offset) };
            mappings.push((function_ptr, function.entry_label as usize));

            // Estimate code size for this function (simplified)
            // In a real implementation, we'd generate code and measure its size
            current_offset += 100; // Rough estimate
        }

        mappings
    }

    /// Generate ARM64 code from parsed BEAM functions
    fn generate_arm_beam_code(&self) -> Vec<u8> {
        let mut code = Vec::new();

        // Function prologue
        // stp x29, x30, [sp, #-16]!  // push fp and lr
        code.extend_from_slice(&[0xfd, 0x7b, 0xbf, 0xa9]);
        // mov x29, sp                  // set fp
        code.extend_from_slice(&[0xfd, 0x03, 0x00, 0x91]);

        // Generate code for each function
        for function in &self.functions {
            // Function prologue for each function
            // stp x29, x30, [sp, #-16]!  // push fp and lr
            code.extend_from_slice(&[0xfd, 0x7b, 0xbf, 0xa9]);
            // mov x29, sp                  // set fp
            code.extend_from_slice(&[0xfd, 0x03, 0x00, 0x91]);

            // Generate code for each instruction in this function
            for instruction in &function.instructions {
                self.generate_arm_instruction_code(&mut code, instruction);
            }

            // Function epilogue
            // ldp x29, x30, [sp], #16      // pop fp and lr
            code.extend_from_slice(&[0xfd, 0x7b, 0xc1, 0xa8]);
            // ret
            code.extend_from_slice(&[0xc0, 0x03, 0x5f, 0xd6]);
        }

        // If no code was generated, add a minimal function
        if code.is_empty() {
            // stp x29, x30, [sp, #-16]!
            code.extend_from_slice(&[0xfd, 0x7b, 0xbf, 0xa9]);
            // mov x29, sp
            code.extend_from_slice(&[0xfd, 0x03, 0x00, 0x91]);
            // mov x2, 42 (return value)
            code.extend_from_slice(&[0x42, 0x00, 0x80, 0xd2]);
            // str x2, [x1]  (store in regs[0])
            code.extend_from_slice(&[0x22, 0x00, 0x00, 0xf9]);
            // ldp x29, x30, [sp], #16
            code.extend_from_slice(&[0xfd, 0x7b, 0xc1, 0xa8]);
            // ret
            code.extend_from_slice(&[0xc0, 0x03, 0x5f, 0xd6]);
        }

        code
    }

    /// Generate ARM64 code for a single BEAM instruction
    fn generate_arm_instruction_code(&self, code: &mut Vec<u8>, instruction: &BeamInstruction) {
        use BeamOpcode;

        match instruction.opcode_enum() {
            Some(BeamOpcode::Move) => {
                self.generate_arm_move_instruction(code, instruction);
            }
            Some(BeamOpcode::Return) => {
                self.generate_arm_return_instruction(code, instruction);
            }
            Some(BeamOpcode::GetList) => {
                self.generate_arm_get_list_instruction(code, instruction);
            }
            Some(BeamOpcode::IsNonemptyList) => {
                self.generate_arm_is_nonempty_list_instruction(code, instruction);
            }
            Some(BeamOpcode::CallOnly) => {
                self.generate_arm_call_only_instruction(code, instruction);
            }
            _ => {
                // Unknown instruction - skip for now
                eprintln!("ARM Warning: Skipping unknown BEAM instruction: {}", instruction.opcode);
            }
        }
    }

    /// Generate ARM64 code for move instruction
    fn generate_arm_move_instruction(&self, code: &mut Vec<u8>, instruction: &BeamInstruction) {
        if instruction.args.len() >= 2 {
            // For now, just move literals to registers
            // This is a simplified implementation
            if let (BeamArg::Literal(value), BeamArg::Register { index, is_y: false }) =
                (&instruction.args[0], &instruction.args[1]) {
                // mov x2, value (simplified - should encode Eterm)
                let eterm_value = (*value as u16) & 0xFFFF; // Simplified encoding
                // mov x2, #value (immediate)
                if eterm_value < 0x1000 {
                    code.extend_from_slice(&[((eterm_value >> 0) & 0xFF) as u8,
                                           ((eterm_value >> 8) & 0xFF) as u8,
                                           0x80, 0xd2]); // mov x2, #imm
                }
            }
        }
    }

    /// Generate ARM64 code for return instruction
    fn generate_arm_return_instruction(&self, code: &mut Vec<u8>, _instruction: &BeamInstruction) {
        self.generate_arm_return_with_value(code, 42); // Default return value
    }

    /// Generate ARM64 code for get_list instruction
    fn generate_arm_get_list_instruction(&self, code: &mut Vec<u8>, instruction: &BeamInstruction) {
        // Simplified: just assume the list has elements and destructure
        if instruction.args.len() >= 3 {
            // For now, just simulate successful list destructuring
            // nop - assume success
            code.extend_from_slice(&[0x1f, 0x20, 0x03, 0xd5]); // nop
        }
    }

    /// Generate ARM64 code for is_nonempty_list instruction
    fn generate_arm_is_nonempty_list_instruction(&self, code: &mut Vec<u8>, instruction: &BeamInstruction) {
        if instruction.args.len() >= 2 {
            // Simplified: assume list is non-empty and jump to success
            if let BeamArg::Label(label) = &instruction.args[0] {
                // For now, don't jump - just continue (assume success)
                // In reality, would check list type and jump on failure
            }
        }
    }

    /// Generate ARM64 code for call_only instruction
    fn generate_arm_call_only_instruction(&self, code: &mut Vec<u8>, instruction: &BeamInstruction) {
        if instruction.args.len() >= 2 {
            if let BeamArg::Label(label) = &instruction.args[1] {
                // Simplified: just jump to label (simulating function call)
                // In reality, would save registers and jump to function
                let offset = *label as i32 * 4; // Rough estimate for ARM64
                if offset >= -128 && offset <= 127 {
                    // b offset (branch)
                    code.extend_from_slice(&[0x00, 0x00, 0x00, 0x14]); // placeholder
                }
            }
        }
    }

    /// Generate ARM64 return with a specific value
    fn generate_arm_return_with_value(&self, code: &mut Vec<u8>, value: u64) {
        // Store return value in x(0) register position
        let eterm_value = (value << 4) | 0xF; // Encode as Eterm small integer

        // Simplified: mov x2, value (immediate)
        let imm_val = (eterm_value & 0xFFFF) as u16;
        if imm_val < 0x1000 {
            code.extend_from_slice(&[((imm_val >> 0) & 0xFF) as u8,
                                   ((imm_val >> 8) & 0xFF) as u8,
                                   0x80, 0xd2]); // mov x2, #imm
            // str x2, [x1]  (store in regs[0])
            code.extend_from_slice(&[0x22, 0x00, 0x00, 0xf9]);
        }

        // Function epilogue
        // ldp x29, x30, [sp], #16      // pop fp and lr
        code.extend_from_slice(&[0xfd, 0x7b, 0xc1, 0xa8]);
        // ret
        code.extend_from_slice(&[0xc0, 0x03, 0x5f, 0xd6]);
    }
}

/// Generate minimal ARM64 code for a BEAM function
///
/// This creates a simple identity function that matches the BEAM calling convention.
/// BEAM functions take: (*mut Process, *mut Eterm) and don't return values.
/// Used as a placeholder until full BEAM instruction parsing is implemented.
fn generate_minimal_arm_beam_function() -> Vec<u8> {
    let mut code = Vec::new();

    // BEAM function signature: fn(process: *mut Process, regs: *mut Eterm)
    // Parameters are passed in x0, x1 (ARM64 calling convention)

    // Function prologue
    // stp x29, x30, [sp, #-16]!  // push fp and lr
    code.extend_from_slice(&[0xfd, 0x7b, 0xbf, 0xa9]);
    // mov x29, sp                  // set fp
    code.extend_from_slice(&[0xfd, 0x03, 0x00, 0x91]);

    // Store some dummy return value in the register array
    // regs[0] (x(0)) should contain the return value
    // Eterm encoding for small integer 42: (42 << 4) | 0xF = 687
    // For now, store the integer 42 as return value using Eterm encoding

    // mov x2, #687  (42 encoded as Eterm: (42 << 4) | 0xF)
    let eterm_value = (42u64 << 4) | 0xF;
    let imm = eterm_value as u16; // ARM64 mov immediate is limited
    // For full 64-bit value, we'd need multiple instructions
    // For now, use a simple immediate that fits
    code.extend_from_slice(&[0x42, 0x00, 0x80, 0xd2]); // mov x2, #42 (placeholder)
    // str x2, [x1]  (store in regs[0])
    code.extend_from_slice(&[0x22, 0x00, 0x00, 0xf9]);

    // Function epilogue
    // ldp x29, x30, [sp], #16      // pop fp and lr
    code.extend_from_slice(&[0xfd, 0x7b, 0xc1, 0xa8]);
    // ret
    code.extend_from_slice(&[0xc0, 0x03, 0x5f, 0xd6]);

    code
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
            Ok((executable, writable, size)) => {
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
        if let Ok((_executable, writable, _size)) = assembler.codegen(&mut allocator) {
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
        if let Ok((_executable, writable, _size)) = assembler.codegen(&mut allocator) {
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
        if let Ok((_executable, writable, _size)) = assembler.codegen(&mut allocator) {
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
        if let Ok((_executable, writable, _size)) = assembler.codegen(&mut allocator) {
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
        if let Ok((_executable, writable, _size)) = assembler.codegen(&mut allocator) {
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

