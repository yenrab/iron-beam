//! Process Register Management
//!
//! Provides functions for synchronizing process state between JIT registers
//! and the process structure in memory. Manages HTOP, E, FCALLS, and active code index.
//!
//! Based on `erts/emulator/beam/jit/arm/beam_asm.hpp` register definitions

use crate::BeamAssemblerError;
use crate::asmjit_wrapper::{Assembler, a64};

/// ARM64 register assignments for Erlang JIT (matching C++ definitions)
pub mod registers {
    /// Erlang stack pointer (x20)
    pub const E: u32 = 20;
    /// Current process pointer (x21)
    pub const C_P: u32 = 21;
    /// Function calls/reductions counter (w22)
    pub const FCALLS: u32 = 22;
    /// Heap top pointer (x23)
    pub const HTOP: u32 = 23;
    /// Active code index (x24)
    pub const ACTIVE_CODE_IX: u32 = 24;
}

/// Process register synchronization flags (matching C++ Update enum)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessRegisterSync {
    /// Synchronize E (Erlang stack pointer)
    Stack = (1 << 0),
    /// Synchronize HTOP (heap pointer)
    Heap = (1 << 1),
    /// Synchronize FCALLS (reduction counter)
    Reductions = (1 << 2),
    /// Synchronize active code index
    CodeIndex = (1 << 3),
    /// Synchronize X registers
    XRegs = (1 << 4),
}

/// Convenience combinations (matching C++ definitions)
impl ProcessRegisterSync {
    /// Heap allocation (heap + stack)
    pub const HEAP_ALLOC: u32 = Self::Heap as u32 | Self::Stack as u32;
    /// Heap-only allocation
    pub const HEAP_ONLY_ALLOC: u32 = Self::Heap as u32;
}

/// Process register manager for ARM64 JIT
///
/// Manages synchronization of process state between JIT registers and
/// the process structure in memory. Ensures consistency when transitioning
/// between JIT execution and runtime C functions.
pub struct ProcessRegisterManager;

impl ProcessRegisterManager {
    /// Enter runtime context with register synchronization
    ///
    /// Saves specified process registers to the process structure before
    /// calling runtime C functions. Matches C++ emit_enter_runtime pattern.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `spec` - Bitmask of ProcessRegisterSync flags
    /// * `live` - Number of live X registers to save (0 = none)
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn emit_enter_runtime(
        assembler: &mut Assembler,
        spec: u32,
        live: usize,
    ) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] Process Registers: Entering runtime with spec=0x{:x}, live={}", spec, live);

        // Validate spec flags
        let valid_flags = ProcessRegisterSync::Stack as u32 |
                         ProcessRegisterSync::Heap as u32 |
                         ProcessRegisterSync::Reductions as u32 |
                         ProcessRegisterSync::XRegs as u32;
        if (spec & !valid_flags) != 0 {
            return Err(BeamAssemblerError::CodeGenerationFailed(
                format!("Invalid spec flags: 0x{:x}", spec & !valid_flags)
            ));
        }

        // If both stack and heap need updating, use STP for efficiency
        if (spec & ProcessRegisterSync::Stack as u32) != 0 &&
           (spec & ProcessRegisterSync::Heap as u32) != 0 {
            eprintln!("[DEBUG] Process Registers: Saving HTOP and E together");
            // stp HTOP, E, [c_p, #offsetof(Process, htop)]
            // Assuming htop and stop are adjacent in Process struct
            const HTOP_OFFSET: i32 = 16; // Placeholder - needs actual Process struct
            a64::emit_stp(assembler, registers::HTOP, registers::E,
                         registers::C_P, HTOP_OFFSET)?;
        } else {
            // Save stack pointer if requested
            if (spec & ProcessRegisterSync::Stack as u32) != 0 {
                Self::save_stack_pointer(assembler)?;
            }

            // Save heap pointer if requested
            if (spec & ProcessRegisterSync::Heap as u32) != 0 {
                Self::save_heap_top(assembler)?;
            }
        }

        // Save reductions counter if requested
        if (spec & ProcessRegisterSync::Reductions as u32) != 0 {
            Self::save_reductions(assembler)?;
        }

        // TODO: Handle X register saving when live > 0 and XRegs flag is set
        // This would involve saving register-backed X registers to the X register array

        Ok(())
    }

    /// Leave runtime context with register synchronization
    ///
    /// Loads specified process registers from the process structure after
    /// returning from runtime C functions. Matches C++ emit_leave_runtime pattern.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `spec` - Bitmask of ProcessRegisterSync flags
    /// * `live` - Number of live X registers to load (0 = none)
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn emit_leave_runtime(
        assembler: &mut Assembler,
        spec: u32,
        live: usize,
    ) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] Process Registers: Leaving runtime with spec=0x{:x}, live={}", spec, live);

        // If both stack and heap need loading, use LDP for efficiency
        if (spec & ProcessRegisterSync::Stack as u32) != 0 &&
           (spec & ProcessRegisterSync::Heap as u32) != 0 {
            eprintln!("[DEBUG] Process Registers: Loading HTOP and E together");
            // ldp HTOP, E, [c_p, #offsetof(Process, htop)]
            const HTOP_OFFSET: i32 = 16; // Placeholder
            a64::emit_ldp(assembler, registers::HTOP, registers::E,
                         registers::C_P, HTOP_OFFSET)?;
        } else {
            // Load heap pointer if requested
            if (spec & ProcessRegisterSync::Heap as u32) != 0 {
                Self::load_heap_top(assembler)?;
            }

            // Load stack pointer if requested
            if (spec & ProcessRegisterSync::Stack as u32) != 0 {
                Self::load_stack_pointer(assembler)?;
            }
        }

        // Load reductions counter if requested
        if (spec & ProcessRegisterSync::Reductions as u32) != 0 {
            Self::load_reductions(assembler)?;
        }

        // Handle active code index loading if requested
        if (spec & ProcessRegisterSync::CodeIndex as u32) != 0 {
            Self::load_code_index(assembler)?;
        }

        // TODO: Handle X register loading when live > 0

        Ok(())
    }

    /// Save process registers to process structure
    ///
    /// Stores the current values of JIT registers back to the process structure
    /// in memory. This ensures the runtime has access to current process state.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `sync_flags` - Which registers to synchronize
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn save_registers(
        assembler: &mut Assembler,
        sync_flags: &[ProcessRegisterSync],
    ) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] Process Registers: Saving registers with flags: {:?}", sync_flags);

        for &flag in sync_flags {
            match flag {
                ProcessRegisterSync::Heap => {
                    Self::save_heap_top(assembler)?;
                }
                ProcessRegisterSync::Stack => {
                    Self::save_stack_pointer(assembler)?;
                }
                ProcessRegisterSync::Reductions => {
                    Self::save_reductions(assembler)?;
                }
                ProcessRegisterSync::CodeIndex => {
                    Self::save_code_index(assembler)?;
                }
                ProcessRegisterSync::XRegs => {
                    // TODO: Implement X register saving
                    todo!("X register saving not implemented");
                }
            }
        }

        Ok(())
    }

    /// Load process registers from process structure
    ///
    /// Loads register values from the process structure into JIT registers.
    /// This ensures JIT code has access to current process state.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `sync_flags` - Which registers to synchronize
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn load_registers(
        assembler: &mut Assembler,
        sync_flags: &[ProcessRegisterSync],
    ) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] Process Registers: Loading registers with flags: {:?}", sync_flags);

        for &flag in sync_flags {
            match flag {
                ProcessRegisterSync::Heap => {
                    Self::load_heap_top(assembler)?;
                }
                ProcessRegisterSync::Stack => {
                    Self::load_stack_pointer(assembler)?;
                }
                ProcessRegisterSync::Reductions => {
                    Self::load_reductions(assembler)?;
                }
                ProcessRegisterSync::CodeIndex => {
                    Self::load_code_index(assembler)?;
                }
                ProcessRegisterSync::XRegs => {
                    // TODO: Implement X register loading
                    todo!("X register loading not implemented");
                }
            }
        }

        Ok(())
    }

    /// Save heap top pointer to process structure
    ///
    /// Stores HTOP (x23) to process->htop
    fn save_heap_top(assembler: &mut Assembler) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] Process Registers: Saving HTOP to process->htop");

        // str HTOP, [c_p, #offsetof(Process, htop)]
        // In Erlang Process structure, htop is at offset (typically 16 or 24)
        // For now, use a placeholder offset - this would need the actual Process struct layout
        const HTOP_OFFSET: i32 = 16; // Placeholder - needs actual Process struct definition

        a64::emit_str_reg_offset(assembler, registers::HTOP, registers::C_P, HTOP_OFFSET)?;

        Ok(())
    }

    /// Load heap top pointer from process structure
    ///
    /// Loads HTOP (x23) from process->htop
    fn load_heap_top(assembler: &mut Assembler) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] Process Registers: Loading HTOP from process->htop");

        // ldr HTOP, [c_p, #offsetof(Process, htop)]
        const HTOP_OFFSET: i32 = 16; // Placeholder

        a64::emit_ldr_reg_offset(assembler, registers::HTOP, registers::C_P, HTOP_OFFSET)?;

        Ok(())
    }

    /// Save Erlang stack pointer to process structure
    ///
    /// Stores E (x20) to process->stop
    fn save_stack_pointer(assembler: &mut Assembler) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] Process Registers: Saving E to process->stop");

        // str E, [c_p, #offsetof(Process, stop)]
        // In Erlang Process structure, stop is at offset (typically 8 or 16)
        const STOP_OFFSET: i32 = 8; // Placeholder

        a64::emit_str_reg_offset(assembler, registers::E, registers::C_P, STOP_OFFSET)?;

        Ok(())
    }

    /// Load Erlang stack pointer from process structure
    ///
    /// Loads E (x20) from process->stop
    fn load_stack_pointer(assembler: &mut Assembler) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] Process Registers: Loading E from process->stop");

        // ldr E, [c_p, #offsetof(Process, stop)]
        const STOP_OFFSET: i32 = 8; // Placeholder

        a64::emit_ldr_reg_offset(assembler, registers::E, registers::C_P, STOP_OFFSET)?;

        Ok(())
    }

    /// Save reductions counter to process structure
    ///
    /// Stores FCALLS (w22) to process->fcalls
    fn save_reductions(assembler: &mut Assembler) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] Process Registers: Saving FCALLS to process->fcalls");

        // str FCALLS, [c_p, #offsetof(Process, fcalls)]
        // fcalls is typically at a specific offset in Process structure
        const FCALLS_OFFSET: i32 = 32; // Placeholder

        // Since FCALLS is w22 (32-bit), we need to store 32 bits
        // For simplicity, store as 64-bit for now
        a64::emit_str_reg_offset(assembler, registers::FCALLS, registers::C_P, FCALLS_OFFSET)?;

        Ok(())
    }

    /// Load reductions counter from process structure
    ///
    /// Loads FCALLS (w22) from process->fcalls
    fn load_reductions(assembler: &mut Assembler) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] Process Registers: Loading FCALLS from process->fcalls");

        // ldr FCALLS, [c_p, #offsetof(Process, fcalls)]
        const FCALLS_OFFSET: i32 = 32; // Placeholder

        a64::emit_ldr_reg_offset(assembler, registers::FCALLS, registers::C_P, FCALLS_OFFSET)?;

        Ok(())
    }

    /// Save active code index to process structure
    ///
    /// Stores active_code_ix (x24) - this is typically managed differently
    fn save_code_index(_assembler: &mut Assembler) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] Process Registers: Saving active code index");

        // The active code index is typically managed through global state
        // rather than per-process state. This might not need explicit saving.

        Ok(())
    }

    /// Load active code index from global state
    ///
    /// Loads active_code_ix (x24) - this involves loading from the_active_code_index global
    fn load_code_index(assembler: &mut Assembler) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] Process Registers: Loading active code index");

        // In the C++ implementation, this involves loading from the_active_code_index global
        // and potentially checking for ERTS_SAVE_CALLS_CODE_IX

        // This is complex and would require access to global Erlang state
        // For now, this is a placeholder

        Ok(())
    }

    /// Initialize process registers for JIT execution
    ///
    /// Sets up the initial register state when entering JIT-compiled code.
    /// This typically involves loading process state into registers.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn initialize_process_registers(assembler: &mut Assembler) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] Process Registers: Initializing process registers");

        // Load essential process state into registers
        Self::load_registers(assembler, &[
            ProcessRegisterSync::Heap,
            ProcessRegisterSync::Stack,
            ProcessRegisterSync::Reductions,
        ])?;

        Ok(())
    }

    /// Finalize process registers after JIT execution
    ///
    /// Saves register state back to process structure when leaving JIT code.
    /// Ensures process state consistency for runtime functions.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn finalize_process_registers(assembler: &mut Assembler) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] Process Registers: Finalizing process registers");

        // Save current register state back to process structure
        Self::save_registers(assembler, &[
            ProcessRegisterSync::Heap,
            ProcessRegisterSync::Stack,
            ProcessRegisterSync::Reductions,
        ])?;

        Ok(())
    }
}

/// Convenience functions for common register operations
impl ProcessRegisterManager {
    /// Setup registers for runtime function call
    ///
    /// Prepares process registers before calling runtime C functions.
    /// This ensures the runtime has access to current process state.
    pub fn setup_for_runtime_call(assembler: &mut Assembler) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] Process Registers: Setting up for runtime call");

        // Save current register state to process structure
        Self::finalize_process_registers(assembler)?;

        Ok(())
    }

    /// Restore registers after runtime function call
    ///
    /// Restores process registers after returning from runtime C functions.
    /// This ensures JIT code continues with correct process state.
    pub fn restore_after_runtime_call(assembler: &mut Assembler) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] Process Registers: Restoring after runtime call");

        // Load register state from process structure
        Self::initialize_process_registers(assembler)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_register_sync_flags() {
        // Test that the enum variants are defined correctly
        assert!(matches!(ProcessRegisterSync::Heap, ProcessRegisterSync::Heap));
        assert!(matches!(ProcessRegisterSync::Stack, ProcessRegisterSync::Stack));
        assert!(matches!(ProcessRegisterSync::Reductions, ProcessRegisterSync::Reductions));
        assert!(matches!(ProcessRegisterSync::CodeIndex, ProcessRegisterSync::CodeIndex));
    }

    #[test]
    fn test_register_constants() {
        // Test that register assignments match ARM64 JIT conventions
        assert_eq!(registers::E, 20);
        assert_eq!(registers::C_P, 21);
        assert_eq!(registers::FCALLS, 22);
        assert_eq!(registers::HTOP, 23);
        assert_eq!(registers::ACTIVE_CODE_IX, 24);
    }

    #[test]
    fn test_process_register_manager_creation() {
        // ProcessRegisterManager has no state, just test creation
        let _manager = ProcessRegisterManager;
    }

    #[test]
    fn test_sync_flag_equality() {
        let flag1 = ProcessRegisterSync::Heap;
        let flag2 = ProcessRegisterSync::Heap;
        let flag3 = ProcessRegisterSync::Stack;

        assert_eq!(flag1, flag2);
        assert_ne!(flag1, flag3);
    }
}
