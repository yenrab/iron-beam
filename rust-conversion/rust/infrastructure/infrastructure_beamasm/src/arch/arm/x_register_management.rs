//! X Register Backing Store Management
//!
//! Manages Erlang X registers that are backed by CPU registers and memory.
//! Provides save/restore operations for register-backed X registers.
//!
//! Based on `erts/emulator/beam/jit/arm/beam_asm.hpp` X register definitions

use crate::BeamAssemblerError;
use crate::asmjit_wrapper::{Assembler, a64};

/// ARM64 X register assignments for Erlang JIT
///
/// Some X registers are kept in CPU registers for performance,
/// while others are stored in memory. This matches the C++ implementation.
pub mod x_registers {
    // Register-backed X registers (kept in CPU registers)
    pub const XREG0: u32 = 25;  // x25
    pub const XREG1: u32 = 26;  // x26
    pub const XREG2: u32 = 27;  // x27

    #[cfg(debug_assertions)]
    pub const XREG3: u32 = 15;  // x15 (caller-save in debug)
    #[cfg(not(debug_assertions))]
    pub const XREG3: u32 = 28;  // x28 (callee-save in release)

    pub const XREG4: u32 = 15;  // x15 (caller-save)
    pub const XREG5: u32 = 16;  // x16 (caller-save)

    /// Total number of register-backed X registers
    pub const NUM_REGISTER_BACKED_XREGS: usize = 6;

    /// Array of register-backed X register numbers
    pub const REGISTER_BACKED_XREGS: [u32; NUM_REGISTER_BACKED_XREGS] = [
        XREG0, XREG1, XREG2, XREG3, XREG4, XREG5
    ];

    /// Highest callee-save X register index
    #[cfg(debug_assertions)]
    pub const HIGHEST_CALLEE_SAVE_XREG: usize = 2;
    #[cfg(not(debug_assertions))]
    pub const HIGHEST_CALLEE_SAVE_XREG: usize = 3;

    /// Lowest caller-save X register index
    pub const LOWEST_CALLER_SAVE_XREG: usize = HIGHEST_CALLEE_SAVE_XREG + 1;
}

/// X register allocation strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XRegisterAllocation {
    /// Callee-save register (preserved across function calls)
    CalleeSave,
    /// Caller-save register (not preserved across function calls)
    CallerSave,
}

/// Live register tracking information
#[derive(Debug, Clone)]
pub struct LiveRegisterInfo {
    /// Which X registers are currently live (in use)
    pub live_xregs: [bool; x_registers::NUM_REGISTER_BACKED_XREGS],
    /// Total number of live registers
    pub live_count: usize,
}

impl LiveRegisterInfo {
    /// Create a new live register info with no live registers
    pub fn new() -> Self {
        Self {
            live_xregs: [false; x_registers::NUM_REGISTER_BACKED_XREGS],
            live_count: 0,
        }
    }

    /// Mark an X register as live
    pub fn set_live(&mut self, xreg_index: usize) {
        if xreg_index < x_registers::NUM_REGISTER_BACKED_XREGS && !self.live_xregs[xreg_index] {
            self.live_xregs[xreg_index] = true;
            self.live_count += 1;
        }
    }

    /// Mark an X register as dead
    pub fn set_dead(&mut self, xreg_index: usize) {
        if xreg_index < x_registers::NUM_REGISTER_BACKED_XREGS && self.live_xregs[xreg_index] {
            self.live_xregs[xreg_index] = false;
            self.live_count -= 1;
        }
    }

    /// Check if an X register is live
    pub fn is_live(&self, xreg_index: usize) -> bool {
        xreg_index < x_registers::NUM_REGISTER_BACKED_XREGS && self.live_xregs[xreg_index]
    }

    /// Get allocation strategy for an X register
    pub fn get_allocation_strategy(xreg_index: usize) -> XRegisterAllocation {
        if xreg_index <= x_registers::HIGHEST_CALLEE_SAVE_XREG {
            XRegisterAllocation::CalleeSave
        } else {
            XRegisterAllocation::CallerSave
        }
    }
}

/// X register management for ARM64 JIT
///
/// Manages the backing store for Erlang X registers, providing save/restore
/// operations and register allocation strategies.
pub struct XRegisterManager;

impl XRegisterManager {
    /// Save all live X registers to the backing store
    ///
    /// Saves register-backed X registers to the X register array in scheduler registers.
    /// Uses efficient STP (store pair) instructions when possible for consecutive registers.
    /// This ensures X register values are preserved across runtime calls.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `live_info` - Information about which registers are currently live
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn save_live_xregs(
        assembler: &mut Assembler,
        live_info: &LiveRegisterInfo,
    ) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] X Register: Saving {} live X registers", live_info.live_count);

        // Use efficient STP (store pair) for consecutive live registers
        let mut i = 0;
        while i < x_registers::NUM_REGISTER_BACKED_XREGS {
            if live_info.is_live(i) {
                // Check if next register is also live for STP optimization
                if i + 1 < x_registers::NUM_REGISTER_BACKED_XREGS && live_info.is_live(i + 1) {
                    // Use STP for pair of registers
                    Self::save_xreg_pair(assembler, i)?;
                    i += 2; // Skip next register since it was saved in pair
                } else {
                    // Save single register
                    let reg_num = x_registers::REGISTER_BACKED_XREGS[i];
                    Self::save_single_xreg(assembler, i, reg_num)?;
                    i += 1;
                }
            } else {
                i += 1;
            }
        }

        Ok(())
    }

    /// Restore all live X registers from the backing store
    ///
    /// Loads register-backed X registers from the X register array in scheduler registers.
    /// Uses efficient LDP (load pair) instructions when possible for consecutive registers.
    /// This restores X register values after runtime calls.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `live_info` - Information about which registers to restore
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn restore_live_xregs(
        assembler: &mut Assembler,
        live_info: &LiveRegisterInfo,
    ) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] X Register: Restoring {} live X registers", live_info.live_count);

        // Use efficient LDP (load pair) for consecutive live registers
        let mut i = 0;
        while i < x_registers::NUM_REGISTER_BACKED_XREGS {
            if live_info.is_live(i) {
                // Check if next register is also live for LDP optimization
                if i + 1 < x_registers::NUM_REGISTER_BACKED_XREGS && live_info.is_live(i + 1) {
                    // Use LDP for pair of registers
                    Self::restore_xreg_pair(assembler, i)?;
                    i += 2; // Skip next register since it was restored in pair
                } else {
                    // Restore single register
                    let reg_num = x_registers::REGISTER_BACKED_XREGS[i];
                    Self::restore_single_xreg(assembler, i, reg_num)?;
                    i += 1;
                }
            } else {
                i += 1;
            }
        }

        Ok(())
    }

    /// Save all register-backed X registers (bulk operation)
    ///
    /// Saves all register-backed X registers regardless of liveness.
    /// Used when maximum safety is required.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn save_all_xregs(assembler: &mut Assembler) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] X Register: Saving all register-backed X registers");

        for (i, &reg_num) in x_registers::REGISTER_BACKED_XREGS.iter().enumerate() {
            Self::save_single_xreg(assembler, i, reg_num)?;
        }

        Ok(())
    }

    /// Restore all register-backed X registers (bulk operation)
    ///
    /// Restores all register-backed X registers regardless of previous liveness.
    /// Used when maximum safety is required.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn restore_all_xregs(assembler: &mut Assembler) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] X Register: Restoring all register-backed X registers");

        for (i, &reg_num) in x_registers::REGISTER_BACKED_XREGS.iter().enumerate() {
            Self::restore_single_xreg(assembler, i, reg_num)?;
        }

        Ok(())
    }

    /// Save a single X register to the backing store
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `xreg_index` - Index of the X register (0-5 for register-backed)
    /// * `cpu_reg` - CPU register number containing the X register value
    ///
    /// # Returns
    /// Result indicating success or failure
    fn save_single_xreg(
        assembler: &mut Assembler,
        xreg_index: usize,
        cpu_reg: u32,
    ) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] X Register: Saving X{} (CPU x{}) to backing store", xreg_index, cpu_reg);

        // Get reference to X register in backing store
        // This would be: [scheduler_registers + x_reg_array.d + (xreg_index * sizeof(Eterm))]
        // For now, we'll use placeholder offsets

        const SCHEDULER_REGISTERS_OFFSET: i32 = 0; // Placeholder
        const X_REG_ARRAY_OFFSET: i32 = 8; // Placeholder
        const ETERM_SIZE: i32 = 8; // 64-bit Eterms

        let xreg_offset = X_REG_ARRAY_OFFSET + (xreg_index as i32 * ETERM_SIZE);
        let total_offset = SCHEDULER_REGISTERS_OFFSET + xreg_offset;

        // Store the CPU register value to the backing store
        // str cpu_reg, [scheduler_registers, #total_offset]
        a64::emit_str_reg_offset(assembler, cpu_reg, 19, total_offset)?; // x19 = scheduler_registers

        Ok(())
    }

    /// Restore a single X register from the backing store
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `xreg_index` - Index of the X register (0-5 for register-backed)
    /// * `cpu_reg` - CPU register number to load the X register value into
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn restore_single_xreg(
        assembler: &mut Assembler,
        xreg_index: usize,
        cpu_reg: u32,
    ) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] X Register: Restoring X{} (CPU x{}) from backing store", xreg_index, cpu_reg);

        // Get reference to X register in backing store
        const SCHEDULER_REGISTERS_OFFSET: i32 = 0; // Placeholder
        const X_REG_ARRAY_OFFSET: i32 = 8; // Placeholder
        const ETERM_SIZE: i32 = 8; // 64-bit Eterms

        let xreg_offset = X_REG_ARRAY_OFFSET + (xreg_index as i32 * ETERM_SIZE);
        let total_offset = SCHEDULER_REGISTERS_OFFSET + xreg_offset;

        // Load from backing store into CPU register
        // ldr cpu_reg, [scheduler_registers, #total_offset]
        a64::emit_ldr_reg_offset(assembler, cpu_reg, 19, total_offset)?; // x19 = scheduler_registers

        Ok(())
    }

    /// Save a pair of consecutive X registers to the backing store
    ///
    /// Uses STP (store pair) for efficient saving of two consecutive X registers.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `xreg_index` - Index of the first X register in the pair (0-4 for register-backed)
    ///
    /// # Returns
    /// Result indicating success or failure
    fn save_xreg_pair(
        assembler: &mut Assembler,
        xreg_index: usize,
    ) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] X Register: Saving X{} and X{} pair to backing store",
                 xreg_index, xreg_index + 1);

        let reg1_num = x_registers::REGISTER_BACKED_XREGS[xreg_index];
        let reg2_num = x_registers::REGISTER_BACKED_XREGS[xreg_index + 1];

        // Get memory reference for the first X register in the pair
        let xreg_offset = Self::get_xreg_ref(xreg_index);

        // Use STP to store both registers efficiently
        // stp reg1, reg2, [scheduler_registers, #xreg_offset]
        a64::emit_stp(assembler, reg1_num, reg2_num, 19, xreg_offset)?;

        Ok(())
    }

    /// Restore a pair of consecutive X registers from the backing store
    ///
    /// Uses LDP (load pair) for efficient restoration of two consecutive X registers.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `xreg_index` - Index of the first X register in the pair (0-4 for register-backed)
    ///
    /// # Returns
    /// Result indicating success or failure
    fn restore_xreg_pair(
        assembler: &mut Assembler,
        xreg_index: usize,
    ) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] X Register: Restoring X{} and X{} pair from backing store",
                 xreg_index, xreg_index + 1);

        let reg1_num = x_registers::REGISTER_BACKED_XREGS[xreg_index];
        let reg2_num = x_registers::REGISTER_BACKED_XREGS[xreg_index + 1];

        // Get memory reference for the first X register in the pair
        let xreg_offset = Self::get_xreg_ref(xreg_index);

        // Use LDP to load both registers efficiently
        // ldp reg1, reg2, [scheduler_registers, #xreg_offset]
        a64::emit_ldp(assembler, reg1_num, reg2_num, 19, xreg_offset)?;

        Ok(())
    }

    /// Load a single X register from the backing store
    ///
    /// Loads an X register value from the backing store into a CPU register.
    /// This is a public wrapper around restore_single_xreg.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `xreg_index` - Index of the X register (0-5 for register-backed)
    /// * `cpu_reg` - CPU register number to load the value into
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn load_xreg(
        assembler: &mut Assembler,
        xreg_index: usize,
        cpu_reg: u32,
    ) -> Result<(), BeamAssemblerError> {
        if xreg_index >= x_registers::NUM_REGISTER_BACKED_XREGS {
            return Err(BeamAssemblerError::CodeGenerationFailed(
                format!("Invalid X register index: {}", xreg_index)
            ));
        }

        let reg_num = x_registers::REGISTER_BACKED_XREGS[xreg_index];
        Self::restore_single_xreg(assembler, xreg_index, reg_num)?;

        // If the target CPU register is different, move the value
        if reg_num != cpu_reg {
            use crate::asmjit_wrapper as a64;
            a64::emit_mov_reg_reg(assembler, cpu_reg, reg_num)?;
        }

        Ok(())
    }

    /// Store a single X register to the backing store
    ///
    /// Stores an X register value from a CPU register to the backing store.
    /// This is the counterpart to load_xreg.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `cpu_reg` - CPU register number containing the value to store
    /// * `xreg_index` - Index of the X register (0-5 for register-backed)
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn store_xreg(
        assembler: &mut Assembler,
        cpu_reg: u32,
        xreg_index: usize,
    ) -> Result<(), BeamAssemblerError> {
        if xreg_index >= x_registers::NUM_REGISTER_BACKED_XREGS {
            return Err(BeamAssemblerError::CodeGenerationFailed(
                format!("Invalid X register index: {}", xreg_index)
            ));
        }

        let reg_num = x_registers::REGISTER_BACKED_XREGS[xreg_index];
        Self::save_single_xreg(assembler, xreg_index, reg_num)?;

        Ok(())
    }

    /// Get memory reference for an X register
    ///
    /// Returns the memory location where an X register should be stored.
    /// This is used for memory-backed X registers.
    ///
    /// # Arguments
    /// * `xreg_index` - Index of the X register
    ///
    /// # Returns
    /// Memory offset for the X register
    pub fn get_xreg_ref(xreg_index: usize) -> i32 {
        // Base offset to x_reg_array.d in scheduler registers
        const X_REG_ARRAY_BASE: i32 = 8; // Placeholder
        const ETERM_SIZE: i32 = 8;

        X_REG_ARRAY_BASE + (xreg_index as i32 * ETERM_SIZE)
    }

    /// Load the address of the X register array
    ///
    /// Loads the address of the X register backing store into a register.
    /// This is used for bulk operations like copy_in_registers/copy_out_registers.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `dest_reg` - Register to load the array address into
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn load_xreg_array_address(
        assembler: &mut Assembler,
        dest_reg: u32,
    ) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] X Register: Loading X register array address into x{}", dest_reg);

        // Load address of x_reg_array.d from scheduler registers
        // This would be: lea dest_reg, [scheduler_registers + x_reg_array.d]
        // For now, use placeholder calculation

        const SCHEDULER_REGISTERS_OFFSET: i32 = 0; // Placeholder
        const X_REG_ARRAY_OFFSET: i32 = 8; // Placeholder

        // add dest_reg, scheduler_registers, #(SCHEDULER_REGISTERS_OFFSET + X_REG_ARRAY_OFFSET)
        a64::emit_add_imm(assembler, dest_reg, 19, (SCHEDULER_REGISTERS_OFFSET + X_REG_ARRAY_OFFSET) as u32)?;

        Ok(())
    }

    /// Flush caller-save X registers before C calls
    ///
    /// Ensures that caller-save X registers are saved to the backing store
    /// before calling C functions, as they may be clobbered.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn flush_caller_save_xregs(assembler: &mut Assembler) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] X Register: Flushing caller-save X registers");

        // Save all caller-save X registers (XREG4, XREG5, and conditionally XREG3)
        for i in x_registers::LOWEST_CALLER_SAVE_XREG..x_registers::NUM_REGISTER_BACKED_XREGS {
            let reg_num = x_registers::REGISTER_BACKED_XREGS[i];
            Self::save_single_xreg(assembler, i, reg_num)?;
        }

        Ok(())
    }

    /// Spill all register-backed X registers
    ///
    /// Forces all register-backed X registers to be saved to memory.
    /// Used when registers need to be preserved across complex operations.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn spill_all_xregs(assembler: &mut Assembler) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] X Register: Spilling all register-backed X registers");

        Self::save_all_xregs(assembler)
    }

    /// Fill all register-backed X registers
    ///
    /// Loads all register-backed X registers from memory.
    /// Used when registers need to be restored after complex operations.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn fill_all_xregs(assembler: &mut Assembler) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] X Register: Filling all register-backed X registers");

        Self::restore_all_xregs(assembler)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_x_register_constants() {
        // Test that register assignments are correct
        assert_eq!(x_registers::XREG0, 25);
        assert_eq!(x_registers::XREG1, 26);
        assert_eq!(x_registers::XREG2, 27);

        // Test register array
        assert_eq!(x_registers::REGISTER_BACKED_XREGS.len(), x_registers::NUM_REGISTER_BACKED_XREGS);
        assert_eq!(x_registers::REGISTER_BACKED_XREGS[0], x_registers::XREG0);
        assert_eq!(x_registers::REGISTER_BACKED_XREGS[1], x_registers::XREG1);
    }

    #[test]
    fn test_live_register_info() {
        let mut live_info = LiveRegisterInfo::new();

        assert_eq!(live_info.live_count, 0);
        assert!(!live_info.is_live(0));

        live_info.set_live(0);
        assert_eq!(live_info.live_count, 1);
        assert!(live_info.is_live(0));

        live_info.set_live(2);
        assert_eq!(live_info.live_count, 2);
        assert!(live_info.is_live(2));

        live_info.set_dead(0);
        assert_eq!(live_info.live_count, 1);
        assert!(!live_info.is_live(0));
    }

    #[test]
    fn test_allocation_strategy() {
        // Test callee-save registers (first few)
        assert_eq!(LiveRegisterInfo::get_allocation_strategy(0), XRegisterAllocation::CalleeSave);
        assert_eq!(LiveRegisterInfo::get_allocation_strategy(1), XRegisterAllocation::CalleeSave);
        assert_eq!(LiveRegisterInfo::get_allocation_strategy(2), XRegisterAllocation::CalleeSave);

        // Test caller-save registers (later ones)
        assert_eq!(LiveRegisterInfo::get_allocation_strategy(3), XRegisterAllocation::CallerSave);
        assert_eq!(LiveRegisterInfo::get_allocation_strategy(4), XRegisterAllocation::CallerSave);
        assert_eq!(LiveRegisterInfo::get_allocation_strategy(5), XRegisterAllocation::CallerSave);
    }

    #[test]
    fn test_xreg_ref_calculation() {
        // Test that X register references are calculated correctly
        let ref0 = XRegisterManager::get_xreg_ref(0);
        let ref1 = XRegisterManager::get_xreg_ref(1);

        // Each Eterm is 8 bytes, so references should be 8 bytes apart
        assert_eq!(ref1 - ref0, 8);
    }

    #[test]
    fn test_x_register_manager_creation() {
        // XRegisterManager has no state, just test creation
        let _manager = XRegisterManager;
    }
}
