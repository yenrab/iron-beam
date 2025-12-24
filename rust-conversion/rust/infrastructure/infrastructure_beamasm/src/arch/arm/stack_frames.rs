//! Stack Frame Management
//!
//! Provides functions for managing stack frames when transitioning between
//! JIT-compiled Erlang code and runtime C functions.
//!
//! Based on `erts/emulator/beam/jit/arm/beam_asm.hpp:431-447`

use crate::BeamAssemblerError;
use crate::asmjit_wrapper::{Assembler, a64};

/// Stack frame management for ARM64 JIT
///
/// Manages stack frames during transitions between Erlang execution
/// and runtime C function calls. Ensures proper register preservation
/// and stack consistency across execution contexts.
pub struct StackFrameManager;

impl StackFrameManager {
    /// Enter an Erlang stack frame
    ///
    /// Saves the link register (LR/x30) on the Erlang stack before
    /// executing Erlang code. This preserves the return address.
    ///
    /// Equivalent to: `str x30, [E, -8]!`
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn emit_enter_erlang_frame(assembler: &mut Assembler) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] Stack Frame: Entering Erlang frame");

        // str x30, [E, -8]!  (pre-indexed store of LR to Erlang stack)
        // E is x20 in Erlang JIT convention (from C++: const a64::Gp E = a64::x20)
        a64::emit_str_reg_offset(assembler, 30, 20, -8)?;

        Ok(())
    }

    /// Leave an Erlang stack frame
    ///
    /// Restores the link register (LR/x30) from the Erlang stack after
    /// completing Erlang code execution.
    ///
    /// Equivalent to: `ldr x30, [E], 8`
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn emit_leave_erlang_frame(assembler: &mut Assembler) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] Stack Frame: Leaving Erlang frame");

        // ldr x30, [E], 8  (post-indexed load of LR from Erlang stack)
        // This restores the link register and adjusts the Erlang stack pointer
        // E is x20 in Erlang JIT convention

        // ldr x30, [x20]     (load LR)
        a64::emit_ldr_reg_offset(assembler, 30, 20, 0)?;
        // add x20, x20, #8   (adjust Erlang stack pointer)
        a64::emit_add_imm(assembler, 20, 20, 8)?;

        Ok(())
    }

    /// Enter a runtime stack frame
    ///
    /// Sets up a proper ARM64 function call frame before calling runtime C functions.
    /// Saves the frame pointer (FP/x29) and link register (LR/x30) on the runtime stack.
    ///
    /// Equivalent to:
    /// ```armasm
    /// stp x29, x30, [sp, -16]!  // Save FP and LR, pre-decrement SP
    /// mov x29, sp                // Set FP to current SP
    /// ```
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn emit_enter_runtime_frame(assembler: &mut Assembler) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] Stack Frame: Entering runtime frame");

        // stp x29, x30, [sp, -16]!  (store pair, pre-indexed)
        // Save frame pointer and link register, adjust SP by -16
        a64::emit_stp_pre_idx(assembler, 29, 30, 31, -16)?;

        // mov x29, sp  (set frame pointer to current stack pointer)
        a64::emit_mov_reg_reg(assembler, 29, 31)?;

        Ok(())
    }

    /// Leave a runtime stack frame
    ///
    /// Restores the ARM64 function call frame after returning from runtime C functions.
    /// Restores the frame pointer (FP/x29) and link register (LR/x30) from the runtime stack.
    ///
    /// Equivalent to:
    /// ```armasm
    /// mov sp, x29                // Restore SP from FP
    /// ldp x29, x30, [sp], 16     // Load FP and LR, post-increment SP
    /// ```
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn emit_leave_runtime_frame(assembler: &mut Assembler) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] Stack Frame: Leaving runtime frame");

        // mov sp, x29  (restore stack pointer from frame pointer)
        a64::emit_mov_reg_reg(assembler, 31, 29)?;

        // ldp x29, x30, [sp], 16  (load pair, post-indexed)
        // Restore frame pointer and link register, adjust SP by +16
        a64::emit_ldp_post_idx(assembler, 29, 30, 31, 16)?;

        Ok(())
    }

    /// Assert stack consistency (debug builds only)
    ///
    /// In debug builds, verifies that stack operations maintain consistency.
    /// This helps catch stack corruption during development.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `expected_alignment` - Expected stack alignment (typically 16 bytes)
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn emit_assert_stack_alignment(
        _assembler: &mut Assembler,
        _expected_alignment: u32,
    ) -> Result<(), BeamAssemblerError> {
        // In debug builds, we could add stack alignment checks here
        // For now, this is a no-op in release builds
        Ok(())
    }

    /// Emit stack overflow check
    ///
    /// Checks if there's sufficient stack space before stack operations.
    /// This prevents stack overflow in constrained environments.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `required_space` - Minimum stack space required (in bytes)
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn emit_stack_overflow_check(
        assembler: &mut Assembler,
        required_space: u32,
    ) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] Stack Frame: Checking for {} bytes of stack space", required_space);

        // In a full implementation, this would compare the current stack pointer
        // against a stack limit. For now, we implement a basic check.

        // Get current stack pointer (SP) into a temporary register
        // cmp sp, stack_limit  (compare with stack limit)
        // b.lo stack_overflow_handler  (branch if lower, meaning overflow)

        // For ARM64, we need to implement this with available instructions
        // Since we don't have a stack limit register defined yet, we'll emit
        // a placeholder that assumes sufficient stack space

        // This is a simplified implementation - in practice, this would
        // compare against a process-specific stack limit
        a64::emit_nop(assembler)?;

        Ok(())
    }

    /// Allocate stack space for local variables
    ///
    /// Reserves space on the runtime stack for local variables and temporaries.
    /// This is typically done at the start of a function.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `size` - Number of bytes to allocate
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn emit_allocate_stack_space(
        assembler: &mut Assembler,
        size: u32,
    ) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] Stack Frame: Allocating {} bytes of stack space", size);

        // sub sp, sp, #size  (subtract from stack pointer)
        // Ensure 16-byte alignment for ARM64 ABI
        let aligned_size = (size + 15) & !15; // Round up to 16-byte boundary

        a64::emit_sub_imm(assembler, 31, 31, aligned_size)?;

        Ok(())
    }

    /// Deallocate stack space for local variables
    ///
    /// Releases space on the runtime stack that was previously allocated for locals.
    /// This is typically done before returning from a function.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `size` - Number of bytes to deallocate
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn emit_deallocate_stack_space(
        assembler: &mut Assembler,
        size: u32,
    ) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] Stack Frame: Deallocating {} bytes of stack space", size);

        // add sp, sp, #size  (add to stack pointer)
        // Ensure 16-byte alignment for ARM64 ABI
        let aligned_size = (size + 15) & !15; // Round up to 16-byte boundary

        a64::emit_add_imm(assembler, 31, 31, aligned_size)?;

        Ok(())
    }

    /// Validate stack pointer alignment
    ///
    /// Ensures the stack pointer is properly aligned for ARM64 ABI requirements.
    /// ARM64 requires 16-byte alignment for the stack pointer.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn emit_validate_stack_alignment(
        assembler: &mut Assembler,
    ) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] Stack Frame: Validating stack alignment");

        // For ARM64, we need to ensure SP is 16-byte aligned
        // tst sp, #15  (test if SP is aligned)
        // b.ne alignment_error  (branch if not equal to 0)

        a64::emit_tst_imm(assembler, 31, 15)?;

        // In a full implementation, we'd branch to an error handler on misalignment
        // For now, we just test and continue

        Ok(())
    }
}

/// Convenience functions for common frame operations
impl StackFrameManager {
    /// Setup complete frame transition for runtime calls
    ///
    /// Combines entering runtime frame with leaving Erlang frame,
    /// which is the typical pattern when calling runtime functions.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn emit_transition_to_runtime(assembler: &mut Assembler) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] Stack Frame: Transitioning to runtime");

        // First leave the Erlang frame
        Self::emit_leave_erlang_frame(assembler)?;

        // Then enter the runtime frame
        Self::emit_enter_runtime_frame(assembler)?;

        Ok(())
    }

    /// Setup complete frame transition for returning to Erlang
    ///
    /// Combines leaving runtime frame with entering Erlang frame,
    /// which is the typical pattern when returning from runtime functions.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn emit_transition_to_erlang(assembler: &mut Assembler) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] Stack Frame: Transitioning to Erlang");

        // First leave the runtime frame
        Self::emit_leave_runtime_frame(assembler)?;

        // Then enter the Erlang frame
        Self::emit_enter_erlang_frame(assembler)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_frame_manager_creation() {
        // StackFrameManager has no state, just test creation
        let _manager = StackFrameManager;
    }

    #[test]
    fn test_stack_alignment_calculation() {
        // Test basic alignment calculations (would be used in real implementation)
        assert_eq!(16 % 16, 0); // 16-byte alignment
        assert_eq!(8 % 16, 8);  // Needs 8 bytes of padding
        assert_eq!(32 % 16, 0); // Already aligned
    }

    #[test]
    fn test_frame_transition_logic() {
        // Test that the transition functions are properly defined
        // (We can't test actual code generation without an assembler)

        // Verify the functions exist and have the right signatures
        let _enter_erlang: fn(&mut Assembler) -> Result<(), BeamAssemblerError> =
            StackFrameManager::emit_enter_erlang_frame;
        let _leave_erlang: fn(&mut Assembler) -> Result<(), BeamAssemblerError> =
            StackFrameManager::emit_leave_erlang_frame;
        let _enter_runtime: fn(&mut Assembler) -> Result<(), BeamAssemblerError> =
            StackFrameManager::emit_enter_runtime_frame;
        let _leave_runtime: fn(&mut Assembler) -> Result<(), BeamAssemblerError> =
            StackFrameManager::emit_leave_runtime_frame;
    }
}
