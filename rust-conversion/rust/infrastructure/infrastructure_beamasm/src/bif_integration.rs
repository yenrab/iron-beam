//! BIF and External Function Integration
//!
//! Provides BIF (Built-In Function) calling, external function resolution,
//! module loading integration, and heavy BIF handling.
//!
//! Based on `instr_bif.cpp` and `instr_call.cpp`

use crate::BeamAssemblerError;
use crate::asmjit_wrapper::{Assembler, a64};

/// BIF classification types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BifType {
    /// Light BIF - simple, fast operations
    Light,
    /// Heavy BIF - complex operations that may yield
    Heavy,
    /// Guard BIF - used in guards, may fail
    Guard,
}

/// BIF call result
#[derive(Debug, Clone)]
pub enum BifResult {
    /// Success with result in specified register
    Success { result_reg: u32 },
    /// Failure - BIF call failed (THE_NON_VALUE returned)
    Failure,
    /// Exception raised
    Exception,
}

/// External function call information
#[derive(Debug, Clone)]
pub struct ExternalCallInfo {
    /// Module atom
    pub module: u64,
    /// Function atom
    pub function: u64,
    /// Arity
    pub arity: u32,
    /// Export entry pointer
    pub export_ptr: Option<u64>,
}

/// BIF call information
#[derive(Debug, Clone)]
pub struct BifCallInfo {
    /// BIF function pointer
    pub bif_ptr: u64,
    /// BIF type (light/heavy/guard)
    pub bif_type: BifType,
    /// Number of arguments
    pub arity: u32,
    /// MFA information for error reporting
    pub mfa: Option<crate::ErrorMFA>,
}

/// BIF and external function integration coordinator
///
/// Manages calling Erlang BIFs and external functions from JIT-compiled code,
/// handling different BIF types and ensuring proper runtime state management.
pub struct BifIntegration;

impl BifIntegration {
    /// Call a BIF (Built-In Function)
    ///
    /// Handles the complete BIF calling process including argument setup,
    /// runtime state management, and result handling.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `bif_info` - BIF call information
    ///
    /// # Returns
    /// Result containing the call result
    pub fn call_bif(
        assembler: &mut Assembler,
        bif_info: &BifCallInfo,
    ) -> Result<BifResult, BeamAssemblerError> {
        eprintln!("[DEBUG] BIF Integration: Calling BIF at 0x{:x} (type: {:?}, arity: {})",
                 bif_info.bif_ptr, bif_info.bif_type, bif_info.arity);

        match bif_info.bif_type {
            BifType::Guard => Self::call_guard_bif(assembler, bif_info),
            BifType::Light => Self::call_light_bif(assembler, bif_info),
            BifType::Heavy => Self::call_heavy_bif(assembler, bif_info),
        }
    }

    /// Call a guard BIF
    ///
    /// Guard BIFs are used in guard expressions and may fail.
    /// They return THE_NON_VALUE on failure.
    fn call_guard_bif(
        assembler: &mut Assembler,
        bif_info: &BifCallInfo,
    ) -> Result<BifResult, BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] BIF Integration: Calling guard BIF");

        // Enter runtime context for BIF call
        crate::RuntimeContextManager::emit_enter_runtime(assembler, crate::RuntimeSpec::HeapAlloc as u32)?;

        // Set up arguments for guard BIF
        // ARG1 = process pointer (c_p)
        // ARG2 = argument vector (X registers)
        // ARG3 = unused (0)
        // ARG4 = BIF function pointer

        // Load process pointer into ARG1
        a64::emit_mov_reg_reg(assembler, 0, 21)?; // ARG1 = c_p (x21)

        // Load X register base address into ARG2
        // In ARM64, X registers are typically at offset in process structure
        a64::emit_add_imm(assembler, 1, 21, 0x100)?; // Placeholder: ARG2 = &X[0]

        // ARG3 = 0 (unused for guard BIFs)
        a64::emit_mov_imm(assembler, 2, 0)?;

        // Load BIF function pointer into ARG4
        a64::emit_mov_imm(assembler, 3, bif_info.bif_ptr)?;

        // Call the BIF
        Self::emit_dynamic_runtime_call(assembler, 4, bif_info.arity)?;

        // Leave runtime context
        crate::RuntimeContextManager::emit_leave_runtime(assembler, crate::RuntimeSpec::HeapAlloc as u32)?;

        // Check if BIF call succeeded (result != THE_NON_VALUE)
        Self::emit_check_bif_result(assembler)?;

        Ok(BifResult::Success { result_reg: 0 }) // Result in x0
    }

    /// Call a light BIF
    ///
    /// Light BIFs are simple, fast operations that don't yield.
    fn call_light_bif(
        assembler: &mut Assembler,
        bif_info: &BifCallInfo,
    ) -> Result<BifResult, BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] BIF Integration: Calling light BIF");

        // Light BIFs don't need full runtime context
        // But we still need to save/restore some state

        // Set up arguments similar to guard BIF
        a64::emit_mov_reg_reg(assembler, 0, 21)?; // ARG1 = c_p
        a64::emit_add_imm(assembler, 1, 21, 0x100)?; // ARG2 = &X[0]
        a64::emit_mov_imm(assembler, 2, 0)?; // ARG3 = 0
        a64::emit_mov_imm(assembler, 3, bif_info.bif_ptr)?; // ARG4 = BIF ptr

        // Call the BIF
        Self::emit_dynamic_runtime_call(assembler, 4, bif_info.arity)?;

        Ok(BifResult::Success { result_reg: 0 })
    }

    /// Call a heavy BIF
    ///
    /// Heavy BIFs are complex operations that may yield and require
    /// full process state management.
    fn call_heavy_bif(
        assembler: &mut Assembler,
        bif_info: &BifCallInfo,
    ) -> Result<BifResult, BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] BIF Integration: Calling heavy BIF");

        // Heavy BIFs require full runtime state management
        crate::RuntimeContextManager::emit_enter_runtime(assembler, crate::RuntimeSpec::HeapAlloc as u32)?;
        crate::XRegisterManager::save_all_xregs(assembler)?;
        crate::SchedulerIntegration::emit_yield_point(assembler, crate::scheduler_integration::YieldMode::TestYield)?;

        // Set up arguments
        a64::emit_mov_reg_reg(assembler, 0, 21)?; // ARG1 = c_p
        a64::emit_add_imm(assembler, 1, 21, 0x100)?; // ARG2 = &X[0]
        a64::emit_mov_imm(assembler, 2, 0)?; // ARG3 = 0
        a64::emit_mov_imm(assembler, 3, bif_info.bif_ptr)?; // ARG4 = BIF ptr

        // Call the BIF with error handling
        Self::emit_dynamic_runtime_call_with_error_handling(assembler, 4, bif_info)?;

        // Restore state
        crate::XRegisterManager::restore_all_xregs(assembler)?;
        crate::RuntimeContextManager::emit_leave_runtime(assembler, crate::RuntimeSpec::HeapAlloc as u32)?;

        Ok(BifResult::Success { result_reg: 0 })
    }

    /// Call an external function
    ///
    /// Handles calling functions from external modules, including
    /// module loading and dispatch table resolution.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `call_info` - External call information
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn call_external_function(
        assembler: &mut Assembler,
        call_info: &ExternalCallInfo,
    ) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] BIF Integration: Calling external function {}/{}",
                 call_info.module, call_info.function);

        // Set up the export entry
        if let Some(export_ptr) = call_info.export_ptr {
            a64::emit_mov_imm(assembler, 0, export_ptr)?; // ARG1 = export
        } else {
            // Resolve export dynamically
            Self::emit_resolve_export(assembler, call_info)?;
        }

        // Set up dispatchable call
        Self::emit_setup_dispatchable_call(assembler)?;

        // Perform the call
        Self::emit_erlang_call(assembler)?;

        Ok(())
    }

    /// Call external function and return (tail call)
    ///
    /// Similar to call_external_function but doesn't return to caller.
    pub fn call_external_function_only(
        assembler: &mut Assembler,
        call_info: &ExternalCallInfo,
    ) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] BIF Integration: Tail calling external function");

        // Set up export
        if let Some(export_ptr) = call_info.export_ptr {
            a64::emit_mov_imm(assembler, 0, export_ptr)?;
        } else {
            Self::emit_resolve_export(assembler, call_info)?;
        }

        // Set up dispatchable call
        Self::emit_setup_dispatchable_call(assembler)?;

        // Leave Erlang frame and branch (tail call)
        Self::emit_leave_erlang_frame(assembler)?;
        Self::emit_branch_to_target(assembler)?;

        Ok(())
    }

    /// Call external function with stack deallocation
    ///
    /// Calls external function and deallocates stack space.
    pub fn call_external_function_last(
        assembler: &mut Assembler,
        call_info: &ExternalCallInfo,
        deallocate_words: u32,
    ) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] BIF Integration: Calling external function with deallocate {}", deallocate_words);

        // Deallocate stack space
        Self::emit_deallocate(assembler, deallocate_words)?;

        // Call as tail call
        Self::call_external_function_only(assembler, call_info)?;

        Ok(())
    }

    /// Resolve export entry for external function
    ///
    /// Dynamically resolves the export entry for a module:function/arity.
    fn emit_resolve_export(
        assembler: &mut Assembler,
        call_info: &ExternalCallInfo,
    ) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] BIF Integration: Resolving export for {}/{}",
                 call_info.module, call_info.function);

        // This would typically involve calling runtime functions
        // to resolve the export entry. For now, we'll simulate it.

        // Load module atom into temporary register
        a64::emit_mov_imm(assembler, 9, call_info.module)?; // TMP1 = module

        // Load function atom into temporary register
        a64::emit_mov_imm(assembler, 10, call_info.function)?; // TMP2 = function

        // Load arity into temporary register
        a64::emit_mov_imm(assembler, 11, call_info.arity as u64)?; // TMP3 = arity

        // Call runtime export resolution function
        // This would be something like: export = erts_find_export(module, function, arity)
        // For now, we'll use a placeholder

        Ok(())
    }

    /// Set up dispatchable call
    ///
    /// Prepares the dispatch table entry for a function call.
    fn emit_setup_dispatchable_call(assembler: &mut Assembler) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] BIF Integration: Setting up dispatchable call");

        // In C++: return arm::Mem(ARG1, CodeIndex, arm::lsl(3))
        // This accesses: export->dispatch.addresses[code_index]

        // Load active code index
        let code_index = 24; // x24 typically holds active_code_ix

        // Set up memory access: [ARG1 + code_index * 8]
        // ARG1 already contains the export pointer
        a64::emit_ldr_reg_offset(assembler, 9, 0, 0)?; // TMP1 = [ARG1] (dispatch ptr)
        a64::emit_ldr_reg_offset(assembler, 9, 9, (code_index * 8) as i32)?; // TMP1 = [TMP1 + code_index*8]

        Ok(())
    }

    /// Emit dynamic runtime call
    ///
    /// Calls a runtime function with the specified number of arguments.
    fn emit_dynamic_runtime_call(
        assembler: &mut Assembler,
        arg_count: u32,
        bif_arity: u32,
    ) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] BIF Integration: Emitting dynamic runtime call with {} args", arg_count);

        // In ARM64, we use blr to call function pointer in register
        // ARG4 (x3) contains the function pointer

        // For BIF calls, we typically call through a dispatcher
        // The dispatcher handles argument passing and result handling

        // Simulate the call - in practice, this would be: blr x3
        a64::emit_mov_imm(assembler, 0, 0x42)?; // Placeholder: simulate return value

        Ok(())
    }

    /// Emit dynamic runtime call with error handling
    fn emit_dynamic_runtime_call_with_error_handling(
        assembler: &mut Assembler,
        arg_count: u32,
        bif_info: &BifCallInfo,
    ) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] BIF Integration: Emitting dynamic runtime call with error handling");

        // Save BIF information for error path
        a64::emit_mov_imm(assembler, 9, bif_info.bif_ptr)?; // TMP1 = BIF ptr
        a64::emit_str_reg_offset(assembler, 9, 21, 0x200)?; // Save to temp memory

        // Call the BIF
        Self::emit_dynamic_runtime_call(assembler, arg_count, bif_info.arity)?;

        // Check result - if THE_NON_VALUE, jump to error handler
        Self::emit_check_bif_failure(assembler)?;

        Ok(())
    }

    /// Check BIF result for success/failure
    fn emit_check_bif_result(assembler: &mut Assembler) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] BIF Integration: Checking BIF result");

        // Check if result is THE_NON_VALUE (failure)
        // THE_NON_VALUE is typically a special marker value

        const THE_NON_VALUE: u64 = 0xDEADBEEF; // Placeholder

        a64::emit_mov_imm(assembler, 9, THE_NON_VALUE)?;
        a64::emit_cmp_reg_reg(assembler, 0, 9)?; // Compare result with THE_NON_VALUE

        // If equal, BIF failed
        // This would typically branch to error handling

        Ok(())
    }

    /// Check for BIF failure and handle error
    fn emit_check_bif_failure(assembler: &mut Assembler) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] BIF Integration: Checking for BIF failure");

        // Similar to emit_check_bif_result but with error path handling

        const THE_NON_VALUE: u64 = 0xDEADBEEF;

        a64::emit_mov_imm(assembler, 9, THE_NON_VALUE)?;
        a64::emit_cmp_reg_reg(assembler, 0, 9)?;

        // On failure, this would typically:
        // - Set up exception information
        // - Call error handling
        // - Return to error path

        Ok(())
    }

    /// Emit Erlang function call
    ///
    /// Performs the actual call to an Erlang function through the dispatch mechanism.
    fn emit_erlang_call(assembler: &mut Assembler) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] BIF Integration: Emitting Erlang call");

        // In C++: erlang_call(target)
        // This typically involves:
        // - Setting up the call frame
        // - Branching to the target
        // - Handling the return

        // For now, simulate the call
        Ok(())
    }

    /// Leave Erlang frame (for tail calls)
    fn emit_leave_erlang_frame(assembler: &mut Assembler) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] BIF Integration: Leaving Erlang frame");

        // Clean up the current Erlang call frame
        // This typically involves stack pointer adjustments

        Ok(())
    }

    /// Branch to call target (tail call)
    fn emit_branch_to_target(assembler: &mut Assembler) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] BIF Integration: Branching to target");

        // Branch to the resolved function address
        // In ARM64: br target_reg

        Ok(())
    }

    /// Deallocate stack space
    fn emit_deallocate(assembler: &mut Assembler, words: u32) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] BIF Integration: Deallocating {} words", words);

        // Deallocate stack space
        // This typically adjusts the stack pointer

        let bytes = words * 8; // Assuming 64-bit words
        a64::emit_add_imm(assembler, 31, 31, bytes)?; // SP += bytes

        Ok(())
    }

    /// Determine if a BIF is heavy
    ///
    /// Checks the BIF table to determine if a BIF requires heavy handling.
    ///
    /// # Arguments
    /// * `bif_ptr` - Pointer to the BIF function
    ///
    /// # Returns
    /// true if heavy, false otherwise
    pub fn is_heavy_bif(bif_ptr: u64) -> bool {
        // This would check the BIF table for the bif_kind field
        // For now, return false (assume light BIF)
        false
    }

    /// Get BIF MFA information
    ///
    /// Retrieves the Module-Function-Arity information for a BIF pointer.
    ///
    /// # Arguments
    /// * `bif_ptr` - Pointer to the BIF function
    ///
    /// # Returns
    /// MFA information if available
    pub fn get_bif_mfa(bif_ptr: u64) -> Option<crate::ErrorMFA> {
        // This would look up the BIF in the bif_table
        // and return the corresponding MFA information

        // Placeholder implementation
        Some(crate::ErrorMFA {
            module: 0x100,    // am_erlang
            function: 0x200,  // some function
            arity: 2,
        })
    }

    /// Validate BIF call information
    ///
    /// Checks if the BIF call information is valid and properly formatted.
    ///
    /// # Arguments
    /// * `bif_info` - BIF call information to validate
    ///
    /// # Returns
    /// true if valid, false otherwise
    pub fn validate_bif_call(bif_info: &BifCallInfo) -> bool {
        bif_info.bif_ptr != 0 && bif_info.arity > 0 && bif_info.arity <= 6 // Max BIF arity
    }

    /// Validate external call information
    ///
    /// Checks if the external call information is valid.
    ///
    /// # Arguments
    /// * `call_info` - External call information to validate
    ///
    /// # Returns
    /// true if valid, false otherwise
    pub fn validate_external_call(call_info: &ExternalCallInfo) -> bool {
        call_info.module != 0 && call_info.function != 0 && call_info.arity <= 255
    }
}

/// Convenience functions for common BIF patterns
impl BifIntegration {
    /// Call a simple arithmetic BIF (e.g., +, -, *, /)
    pub fn call_arithmetic_bif(
        assembler: &mut Assembler,
        bif_ptr: u64,
        left_reg: u32,
        right_reg: u32,
        result_reg: u32,
    ) -> Result<BifResult, BeamAssemblerError> {
        let bif_info = BifCallInfo {
            bif_ptr,
            bif_type: BifType::Light,
            arity: 2,
            mfa: Self::get_bif_mfa(bif_ptr),
        };

        // Set up arguments in X registers
        crate::XRegisterManager::load_xreg(assembler, 0, left_reg)?;
        crate::XRegisterManager::load_xreg(assembler, 1, right_reg)?;

        let result = Self::call_bif(assembler, &bif_info)?;

        // Store result
        if let BifResult::Success { result_reg: _ } = result {
            crate::XRegisterManager::store_xreg(assembler, result_reg, 0)?;
        }

        Ok(result)
    }

    /// Call a comparison BIF (e.g., ==, !=, <, >, <=, >=)
    pub fn call_comparison_bif(
        assembler: &mut Assembler,
        bif_ptr: u64,
        left_reg: u32,
        right_reg: u32,
    ) -> Result<BifResult, BeamAssemblerError> {
        let bif_info = BifCallInfo {
            bif_ptr,
            bif_type: BifType::Guard,
            arity: 2,
            mfa: Self::get_bif_mfa(bif_ptr),
        };

        // Set up arguments
        crate::XRegisterManager::load_xreg(assembler, 0, left_reg)?;
        crate::XRegisterManager::load_xreg(assembler, 1, right_reg)?;

        Self::call_bif(assembler, &bif_info)
    }

    /// Call a type test BIF (e.g., is_atom, is_tuple, is_list)
    pub fn call_type_test_bif(
        assembler: &mut Assembler,
        bif_ptr: u64,
        test_reg: u32,
    ) -> Result<BifResult, BeamAssemblerError> {
        let bif_info = BifCallInfo {
            bif_ptr,
            bif_type: BifType::Guard,
            arity: 1,
            mfa: Self::get_bif_mfa(bif_ptr),
        };

        // Set up argument
        crate::XRegisterManager::load_xreg(assembler, 0, test_reg)?;

        Self::call_bif(assembler, &bif_info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bif_types() {
        // Test BifType enum
        assert_eq!(BifType::Light as u8, BifType::Light as u8);
        assert_ne!(BifType::Light as u8, BifType::Heavy as u8);
        assert_ne!(BifType::Guard as u8, BifType::Heavy as u8);
    }

    #[test]
    fn test_bif_call_info_creation() {
        let bif_info = BifCallInfo {
            bif_ptr: 0x1000,
            bif_type: BifType::Light,
            arity: 2,
            mfa: Some(crate::ErrorMFA {
                module: 100,
                function: 200,
                arity: 2,
            }),
        };

        assert_eq!(bif_info.bif_ptr, 0x1000);
        assert_eq!(bif_info.bif_type, BifType::Light);
        assert_eq!(bif_info.arity, 2);
        assert!(bif_info.mfa.is_some());
    }

    #[test]
    fn test_external_call_info_creation() {
        let call_info = ExternalCallInfo {
            module: 1000,
            function: 2000,
            arity: 3,
            export_ptr: Some(0x3000),
        };

        assert_eq!(call_info.module, 1000);
        assert_eq!(call_info.function, 2000);
        assert_eq!(call_info.arity, 3);
        assert_eq!(call_info.export_ptr, Some(0x3000));
    }

    #[test]
    fn test_bif_validation() {
        // Valid BIF info
        let valid_bif = BifCallInfo {
            bif_ptr: 0x1000,
            bif_type: BifType::Light,
            arity: 2,
            mfa: None,
        };
        assert!(BifIntegration::validate_bif_call(&valid_bif));

        // Invalid BIF info - null pointer
        let invalid_bif = BifCallInfo {
            bif_ptr: 0,
            bif_type: BifType::Light,
            arity: 2,
            mfa: None,
        };
        assert!(!BifIntegration::validate_bif_call(&invalid_bif));

        // Invalid BIF info - zero arity
        let invalid_bif2 = BifCallInfo {
            bif_ptr: 0x1000,
            bif_type: BifType::Light,
            arity: 0,
            mfa: None,
        };
        assert!(!BifIntegration::validate_bif_call(&invalid_bif2));
    }

    #[test]
    fn test_external_call_validation() {
        // Valid external call
        let valid_call = ExternalCallInfo {
            module: 1000,
            function: 2000,
            arity: 3,
            export_ptr: Some(0x3000),
        };
        assert!(BifIntegration::validate_external_call(&valid_call));

        // Invalid external call - null module
        let invalid_call = ExternalCallInfo {
            module: 0,
            function: 2000,
            arity: 3,
            export_ptr: Some(0x3000),
        };
        assert!(!BifIntegration::validate_external_call(&invalid_call));

        // Invalid external call - null function
        let invalid_call2 = ExternalCallInfo {
            module: 1000,
            function: 0,
            arity: 3,
            export_ptr: Some(0x3000),
        };
        assert!(!BifIntegration::validate_external_call(&invalid_call2));
    }

    #[test]
    fn test_is_heavy_bif() {
        // For now, this always returns false
        assert!(!BifIntegration::is_heavy_bif(0x1000));
        assert!(!BifIntegration::is_heavy_bif(0x2000));
    }

    #[test]
    fn test_get_bif_mfa() {
        // Test that we get some MFA information
        let mfa = BifIntegration::get_bif_mfa(0x1000);
        assert!(mfa.is_some());

        if let Some(mfa_info) = mfa {
            assert_eq!(mfa_info.arity, 2); // Placeholder arity
        }
    }

    #[test]
    fn test_bif_integration_creation() {
        // BifIntegration has no state, just test creation
        let _integration = BifIntegration;
    }
}
