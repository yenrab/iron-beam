//! Runtime Context Management
//!
//! Provides functions for entering and leaving runtime context in JIT-generated code.
//! This manages the transition between JIT-compiled Erlang code and runtime C functions.
//!
//! Based on `erts/emulator/beam/jit/x86/beam_asm.hpp` emit_enter_runtime/emit_leave_runtime

use crate::BeamAssemblerError;
use crate::asmjit_wrapper::{Assembler, a64};

/// Runtime specification flags for context management
///
/// Equivalent to C++ Update enum in beam_asm.hpp
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum RuntimeSpec {
    /// Update stack pointer (E register)
    Stack = 1 << 0,
    /// Update heap pointer (HTOP register)
    Heap = 1 << 1,
    /// Update reductions counter (FCALLS register)
    Reductions = 1 << 2,
    /// Update active code index
    CodeIndex = 1 << 3,
    /// Update both heap and stack (convenience combination)
    HeapAlloc = Self::Heap as u32 | Self::Stack as u32,
}

impl RuntimeSpec {
    /// Check if a specific flag is set
    pub fn has_flag(&self, flag: RuntimeSpec) -> bool {
        (*self as u32 & flag as u32) != 0
    }

    /// Combine multiple runtime specs
    pub fn combine(specs: &[RuntimeSpec]) -> u32 {
        specs.iter().fold(0u32, |acc, spec| acc | *spec as u32)
    }
}

/// Runtime context management for ARM64 JIT
///
/// Manages the transition between JIT-compiled Erlang code and runtime functions.
/// This ensures process state is properly saved and restored across runtime calls.
pub struct RuntimeContextManager;

impl RuntimeContextManager {
    /// Emit code to enter runtime context
    ///
    /// Saves process state before calling runtime C functions.
    /// This ensures the runtime has access to current process state.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `spec` - Which parts of process state to save
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn emit_enter_runtime(
        assembler: &mut Assembler,
        spec: u32,
    ) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] Runtime Context: Entering runtime with spec 0x{:x}", spec);

        // Assert we're on Erlang stack (in debug builds)
        Self::emit_assert_erlang_stack(assembler)?;

        // Validate spec flags - only allow valid combinations
        let valid_flags = RuntimeSpec::Stack as u32
                        | RuntimeSpec::Heap as u32
                        | RuntimeSpec::Reductions as u32
                        | RuntimeSpec::CodeIndex as u32;
        if (spec & !valid_flags) != 0 {
            return Err(BeamAssemblerError::CodeGenerationFailed(
                format!("Invalid runtime spec flags: 0x{:x}", spec)
            ));
        }

        // ARM64 register mappings (from C++ code):
        // HTOP = x23 (heap top pointer)
        // E = x20 (stack pointer)
        // FCALLS = w22 (reductions counter)
        // c_p = x21 (process pointer)

        // Update heap and stack pointers if both are requested
        if (spec & (RuntimeSpec::Heap as u32 | RuntimeSpec::Stack as u32))
            == (RuntimeSpec::Heap as u32 | RuntimeSpec::Stack as u32) {

            eprintln!("[DEBUG] Runtime Context: Updating both heap and stack pointers");

            // Store HTOP and E in one go using STP (store pair)
            // Equivalent to: stp x23, x20, [x21, htop_offset]
            // Where htop_offset = offsetof(Process, htop)
            const HTOP_OFFSET: i32 = 0; // Placeholder - need actual Process struct offset
            a64::emit_stp_pre_idx(assembler, 23, 20, 21, HTOP_OFFSET)?;

        } else {
            // Update individual components
            if (spec & RuntimeSpec::Stack as u32) != 0 {
                eprintln!("[DEBUG] Runtime Context: Updating stack pointer");
                // Store E to process->stop
                // str x20, [x21, stop_offset]
                const STOP_OFFSET: i32 = 8; // Placeholder
                a64::emit_str_reg_offset(assembler, 20, 21, STOP_OFFSET)?;
            }

            if (spec & RuntimeSpec::Heap as u32) != 0 {
                eprintln!("[DEBUG] Runtime Context: Updating heap pointer");
                // Store HTOP to process->htop
                // str x23, [x21, htop_offset]
                const HTOP_OFFSET: i32 = 0; // Placeholder
                a64::emit_str_reg_offset(assembler, 23, 21, HTOP_OFFSET)?;
            }
        }

        // Update reductions counter if requested
        if (spec & RuntimeSpec::Reductions as u32) != 0 {
            eprintln!("[DEBUG] Runtime Context: Updating reductions counter");
            // Store FCALLS to process->fcalls
            // str w22, [x21, fcalls_offset]
            const FCALLS_OFFSET: i32 = 16; // Placeholder
            a64::emit_str_reg_offset(assembler, 22, 21, FCALLS_OFFSET)?;
        }

        // Handle stack alignment for runtime calls
        // ARM64 ABI requires 16-byte stack alignment
        Self::emit_align_runtime_stack(assembler)?;

        Ok(())
    }

    /// Emit code to leave runtime context
    ///
    /// Restores process state after returning from runtime C functions.
    /// This ensures the JIT code continues with correct process state.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `spec` - Which parts of process state to restore
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn emit_leave_runtime(
        assembler: &mut Assembler,
        spec: u32,
    ) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] Runtime Context: Leaving runtime with spec 0x{:x}", spec);

        // Assert we're on runtime stack (in debug builds)
        Self::emit_assert_runtime_stack(assembler)?;

        // Validate spec flags
        let valid_flags = RuntimeSpec::Stack as u32
                        | RuntimeSpec::Heap as u32
                        | RuntimeSpec::Reductions as u32
                        | RuntimeSpec::CodeIndex as u32;
        if (spec & !valid_flags) != 0 {
            return Err(BeamAssemblerError::CodeGenerationFailed(
                format!("Invalid runtime spec flags: 0x{:x}", spec)
            ));
        }

        // ARM64 register mappings:
        // HTOP = x23 (heap top pointer)
        // E = x20 (stack pointer)
        // FCALLS = w22 (reductions counter)
        // c_p = x21 (process pointer)

        // Restore reductions counter if requested
        if (spec & RuntimeSpec::Reductions as u32) != 0 {
            eprintln!("[DEBUG] Runtime Context: Restoring reductions counter");
            // Load FCALLS from process->fcalls
            // ldr w22, [x21, fcalls_offset]
            const FCALLS_OFFSET: i32 = 16; // Placeholder
            a64::emit_ldr_reg_offset(assembler, 22, 21, FCALLS_OFFSET)?;
        }

        // Handle code index updates if requested
        if (spec & RuntimeSpec::CodeIndex as u32) != 0 {
            eprintln!("[DEBUG] Runtime Context: Updating code index");
            // This involves loading from the_active_code_index global
            // and updating active_code_ix. This is complex and simplified for now.
            // In practice, this would involve:
            // - Loading the_active_code_index address
            // - Loading the value
            // - Conditional selection based on current active_code_ix
        }

        // Restore heap and stack pointers if both are requested
        if (spec & (RuntimeSpec::Heap as u32 | RuntimeSpec::Stack as u32))
            == (RuntimeSpec::Heap as u32 | RuntimeSpec::Stack as u32) {

            eprintln!("[DEBUG] Runtime Context: Restoring both heap and stack pointers");

            // Load HTOP and E in one go using LDP (load pair)
            // Equivalent to: ldp x23, x20, [x21, htop_offset]
            // Where htop_offset = offsetof(Process, htop)
            const HTOP_OFFSET: i32 = 0; // Placeholder
            a64::emit_ldp_post_idx(assembler, 23, 20, 21, HTOP_OFFSET)?;

        } else {
            // Restore individual components
            if (spec & RuntimeSpec::Heap as u32) != 0 {
                eprintln!("[DEBUG] Runtime Context: Restoring heap pointer");
                // Load HTOP from process->htop
                // ldr x23, [x21, htop_offset]
                const HTOP_OFFSET: i32 = 0; // Placeholder
                a64::emit_ldr_reg_offset(assembler, 23, 21, HTOP_OFFSET)?;
            }

            if (spec & RuntimeSpec::Stack as u32) != 0 {
                eprintln!("[DEBUG] Runtime Context: Restoring stack pointer");
                // Load E from process->stop
                // ldr x20, [x21, stop_offset]
                const STOP_OFFSET: i32 = 8; // Placeholder
                a64::emit_ldr_reg_offset(assembler, 20, 21, STOP_OFFSET)?;
            }
        }

        // Restore stack alignment
        Self::emit_restore_runtime_stack(assembler)?;

        Ok(())
    }

    /// Assert that we're currently on the Erlang stack
    ///
    /// In debug builds, this verifies stack consistency before runtime calls.
    fn emit_assert_erlang_stack(_assembler: &mut Assembler) -> Result<(), BeamAssemblerError> {
        // In debug builds, we would add assertions here
        // For now, this is a no-op
        Ok(())
    }

    /// Assert that we're currently on the runtime stack
    ///
    /// In debug builds, this verifies stack consistency after runtime calls.
    fn emit_assert_runtime_stack(_assembler: &mut Assembler) -> Result<(), BeamAssemblerError> {
        // In debug builds, we would add assertions here
        // For now, this is a no-op
        Ok(())
    }

    /// Align stack for runtime function calls
    ///
    /// ARM64 ABI requires 16-byte stack alignment for function calls.
    fn emit_align_runtime_stack(assembler: &mut Assembler) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] Runtime Context: Aligning stack for runtime call");

        // ARM64 requires 16-byte stack alignment
        // We save the current stack pointer and align it

        // This is a simplified version - in practice, we would:
        // 1. Save current SP to a temporary location
        // 2. Align SP to 16-byte boundary
        // 3. Store the original SP for restoration

        // For now, assume the stack is already aligned
        // In a full implementation, this would involve:
        // str x29, [sp, -16]!  // Save frame pointer and allocate space
        // sub sp, sp, #16       // Ensure alignment
        // and sp, sp, #-16      // Align to 16 bytes

        Ok(())
    }

    /// Restore stack after runtime function calls
    ///
    /// Restores the original stack alignment after runtime calls complete.
    fn emit_restore_runtime_stack(_assembler: &mut Assembler) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] Runtime Context: Restoring stack after runtime call");

        // Restore the stack pointer saved during alignment
        // This would typically involve: ldr x29, [sp], 16

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asmjit_wrapper::Assembler;

    #[test]
    fn test_runtime_spec_flags() {
        let stack = RuntimeSpec::Stack;
        let heap = RuntimeSpec::Heap;
        let _reductions = RuntimeSpec::Reductions;

        assert!(stack.has_flag(RuntimeSpec::Stack));
        assert!(!stack.has_flag(RuntimeSpec::Heap));

        let combined = RuntimeSpec::combine(&[stack, heap]);
        assert_eq!(combined, RuntimeSpec::HeapAlloc as u32);
    }

    #[test]
    fn test_runtime_context_manager_creation() {
        // RuntimeContextManager has no state, so this is just a smoke test
        let _manager = RuntimeContextManager;
    }
}
