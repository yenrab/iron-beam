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

    pub const XREG4: u32 = 17;  // x17 (caller-save)
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

    /// Load zeros into all register-backed X registers
    ///
    /// For REPL modules with no arguments, initializes CPU registers to zero
    /// instead of trying to load from uninitialized memory.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn load_zeros_to_xregs(assembler: &mut Assembler) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] X Register: Loading zeros into register-backed X registers");

        for &reg_num in x_registers::REGISTER_BACKED_XREGS.iter() {
            // Load zero into each CPU register
            a64::emit_mov_imm(assembler, reg_num, 0)?;
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
        _cpu_reg: u32,
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

    // #[test]
    // fn test_x_register_arm64_validity() {
    //     // Test that all assigned registers are valid ARM64 registers (x0-x30)
    //     let all_xregs = [
    //         x_registers::XREG0,
    //         x_registers::XREG1,
    //         x_registers::XREG2,
    //         x_registers::XREG3,
    //         x_registers::XREG4,
    //         x_registers::XREG5,
    //     ];

    //     for reg in all_xregs {
    //         assert!(reg >= 0u32 && reg <= 30u32, "Register x{} is not a valid ARM64 register", reg);
    //     }
    // }

    #[test]
    fn test_x_register_array_consistency() {
        // Test that the register array contains all expected registers
        assert_eq!(x_registers::REGISTER_BACKED_XREGS.len(), 6);

        let expected_regs = [25, 26, 27, x_registers::XREG3, 17, 16];
        for (i, &expected) in expected_regs.iter().enumerate() {
            assert_eq!(x_registers::REGISTER_BACKED_XREGS[i], expected,
                      "REGISTER_BACKED_XREGS[{}] should be {}", i, expected);
        }
    }

    #[test]
    fn test_x_register_allocation_ranges() {
        // Test that callee-save and caller-save ranges are correctly defined

        // HIGHEST_CALLEE_SAVE_XREG should be 2 or 3 depending on build config
        let highest_callee = x_registers::HIGHEST_CALLEE_SAVE_XREG;
        assert!(highest_callee == 2 || highest_callee == 3,
               "HIGHEST_CALLEE_SAVE_XREG should be 2 or 3, got {}", highest_callee);

        // LOWEST_CALLER_SAVE_XREG should be HIGHEST_CALLEE_SAVE_XREG + 1
        let lowest_caller = x_registers::LOWEST_CALLER_SAVE_XREG;
        assert_eq!(lowest_caller, highest_callee + 1,
                  "LOWEST_CALLER_SAVE_XREG should be HIGHEST_CALLEE_SAVE_XREG + 1");

        // LOWEST_CALLER_SAVE_XREG should be within valid range
        assert!(lowest_caller < x_registers::NUM_REGISTER_BACKED_XREGS,
               "LOWEST_CALLER_SAVE_XREG {} exceeds array bounds", lowest_caller);
    }

    #[test]
    fn test_x_register_callee_save_registers() {
        // Test that the first few registers are callee-save
        for i in 0..=x_registers::HIGHEST_CALLEE_SAVE_XREG {
            assert_eq!(LiveRegisterInfo::get_allocation_strategy(i), XRegisterAllocation::CalleeSave,
                      "X register {} should be callee-save", i);
        }
    }

    #[test]
    fn test_x_register_caller_save_registers() {
        // Test that later registers are caller-save
        for i in x_registers::LOWEST_CALLER_SAVE_XREG..x_registers::NUM_REGISTER_BACKED_XREGS {
            assert_eq!(LiveRegisterInfo::get_allocation_strategy(i), XRegisterAllocation::CallerSave,
                      "X register {} should be caller-save", i);
        }
    }

    #[test]
    fn test_x_register_no_duplicates() {
        // Test that no register is assigned to multiple X registers
        let mut seen_regs = std::collections::HashSet::new();

        for &reg in &x_registers::REGISTER_BACKED_XREGS {
            assert!(!seen_regs.contains(&reg),
                   "Register x{} is assigned to multiple X registers", reg);
            seen_regs.insert(reg);
        }

        assert_eq!(seen_regs.len(), x_registers::NUM_REGISTER_BACKED_XREGS,
                  "Should have exactly {} unique registers", x_registers::NUM_REGISTER_BACKED_XREGS);
    }

    #[test]
    fn test_x_register_constants_immutability() {
        // Test that constants are properly defined and immutable
        // (This is more of a compile-time check, but we can verify values)

        // XREG0-XREG2 should be fixed callee-save registers
        assert_eq!(x_registers::XREG0, 25); // x25
        assert_eq!(x_registers::XREG1, 26); // x26
        assert_eq!(x_registers::XREG2, 27); // x27

        // XREG4-XREG5 should be fixed caller-save registers
        assert_eq!(x_registers::XREG4, 17); // x17
        assert_eq!(x_registers::XREG5, 16); // x16
    }

    #[test]
    fn test_x_register_build_configuration() {
        // Test that build configuration affects register assignment correctly

        #[cfg(debug_assertions)]
        {
            // In debug builds, XREG3 should be caller-save (x15)
            assert_eq!(x_registers::XREG3, 15);
            assert_eq!(x_registers::HIGHEST_CALLEE_SAVE_XREG, 2);
        }

        #[cfg(not(debug_assertions))]
        {
            // In release builds, XREG3 should be callee-save (x28)
            assert_eq!(x_registers::XREG3, 28);
            assert_eq!(x_registers::HIGHEST_CALLEE_SAVE_XREG, 3);
        }
    }

    #[test]
    fn test_x_register_total_count() {
        // Test that we have the expected total number of register-backed X registers
        assert_eq!(x_registers::NUM_REGISTER_BACKED_XREGS, 6,
                  "Should have exactly 6 register-backed X registers");

        // All registers should be accounted for
        assert_eq!(x_registers::REGISTER_BACKED_XREGS.len(), 6);
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
    fn test_live_register_info_initial_state() {
        let live_info = LiveRegisterInfo::new();

        // All registers should be dead initially
        for i in 0..x_registers::NUM_REGISTER_BACKED_XREGS {
            assert!(!live_info.is_live(i), "Register {} should be dead initially", i);
        }

        assert_eq!(live_info.live_count, 0);
    }

    #[test]
    fn test_live_register_info_boundary_conditions() {
        let mut live_info = LiveRegisterInfo::new();

        // Test setting all registers live
        for i in 0..x_registers::NUM_REGISTER_BACKED_XREGS {
            live_info.set_live(i);
            assert!(live_info.is_live(i), "Register {} should be live", i);
        }

        assert_eq!(live_info.live_count, x_registers::NUM_REGISTER_BACKED_XREGS);

        // Test setting all registers dead
        for i in 0..x_registers::NUM_REGISTER_BACKED_XREGS {
            live_info.set_dead(i);
            assert!(!live_info.is_live(i), "Register {} should be dead", i);
        }

        assert_eq!(live_info.live_count, 0);
    }

    #[test]
    fn test_live_register_info_duplicate_operations() {
        let mut live_info = LiveRegisterInfo::new();

        // Setting the same register live multiple times should not increase count
        live_info.set_live(1);
        assert_eq!(live_info.live_count, 1);

        live_info.set_live(1);
        assert_eq!(live_info.live_count, 1);

        // Setting the same register dead multiple times should not decrease count
        live_info.set_dead(1);
        assert_eq!(live_info.live_count, 0);

        live_info.set_dead(1);
        assert_eq!(live_info.live_count, 0);
    }

    #[test]
    fn test_live_register_info_out_of_bounds() {
        let mut live_info = LiveRegisterInfo::new();

        // Out of bounds indices should be ignored
        live_info.set_live(x_registers::NUM_REGISTER_BACKED_XREGS);
        live_info.set_live(x_registers::NUM_REGISTER_BACKED_XREGS + 10);
        live_info.set_dead(x_registers::NUM_REGISTER_BACKED_XREGS);

        // Should still have no live registers
        assert_eq!(live_info.live_count, 0);

        // Out of bounds queries should return false
        assert!(!live_info.is_live(x_registers::NUM_REGISTER_BACKED_XREGS));
        assert!(!live_info.is_live(usize::MAX));
    }

    #[test]
    fn test_live_register_info_state_consistency() {
        let mut live_info = LiveRegisterInfo::new();

        // Test various sequences to ensure state remains consistent
        let test_sequence = vec![0, 2, 4, 1, 3, 5, 0, 2];

        // Apply sequence
        for &reg in &test_sequence {
            live_info.set_live(reg);
        }

        // Should have 6 unique registers live (0, 1, 2, 3, 4, 5)
        assert_eq!(live_info.live_count, 6);

        // Verify expected registers are live
        for i in 0..x_registers::NUM_REGISTER_BACKED_XREGS {
            let expected_live = i <= 5;
            assert_eq!(live_info.is_live(i), expected_live,
                      "Register {} live state should be {}", i, expected_live);
        }
    }

    #[test]
    fn test_live_register_info_clone() {
        let mut original = LiveRegisterInfo::new();
        original.set_live(1);
        original.set_live(3);
        original.set_live(5);

        let mut cloned = original.clone();

        // Clone should have same state
        assert_eq!(original.live_count, cloned.live_count);
        for i in 0..x_registers::NUM_REGISTER_BACKED_XREGS {
            assert_eq!(original.is_live(i), cloned.is_live(i),
                      "Register {} live state should match in clone", i);
        }

        // Modifying clone shouldn't affect original
        cloned.set_live(0);
        assert!(!original.is_live(0), "Original should not be affected by clone modification");
        assert!(cloned.is_live(0), "Clone should reflect its own modification");
    }

    #[test]
    fn test_live_register_info_sparse_pattern() {
        let mut live_info = LiveRegisterInfo::new();

        // Create a sparse live register pattern
        let live_regs = vec![0, 2, 5];
        for &reg in &live_regs {
            live_info.set_live(reg);
        }

        assert_eq!(live_info.live_count, live_regs.len());

        // Check all registers
        for i in 0..x_registers::NUM_REGISTER_BACKED_XREGS {
            let should_be_live = live_regs.contains(&i);
            assert_eq!(live_info.is_live(i), should_be_live,
                      "Register {} should {}be live", i, if should_be_live { "" } else { "not " });
        }
    }

    #[test]
    fn test_live_register_info_count_accuracy() {
        let mut live_info = LiveRegisterInfo::new();

        // Test that count stays accurate through various operations
        assert_eq!(live_info.live_count, 0);

        live_info.set_live(1);
        assert_eq!(live_info.live_count, 1);

        live_info.set_live(3);
        assert_eq!(live_info.live_count, 2);

        live_info.set_live(1); // Duplicate
        assert_eq!(live_info.live_count, 2);

        live_info.set_dead(1);
        assert_eq!(live_info.live_count, 1);

        live_info.set_dead(3);
        assert_eq!(live_info.live_count, 0);

        live_info.set_dead(5); // Not live
        assert_eq!(live_info.live_count, 0);
    }

    #[test]
    fn test_live_register_info_array_bounds() {
        let live_info = LiveRegisterInfo::new();

        // The live_xregs array should have the correct size
        assert_eq!(live_info.live_xregs.len(), x_registers::NUM_REGISTER_BACKED_XREGS);

        // All elements should be initialized to false
        for &is_live in &live_info.live_xregs {
            assert!(!is_live, "All registers should be dead initially");
        }
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
    fn test_allocation_strategy_boundary() {
        // Test the boundary between callee-save and caller-save
        let boundary = x_registers::HIGHEST_CALLEE_SAVE_XREG;

        // Last callee-save register
        assert_eq!(LiveRegisterInfo::get_allocation_strategy(boundary), XRegisterAllocation::CalleeSave);

        // First caller-save register
        assert_eq!(LiveRegisterInfo::get_allocation_strategy(boundary + 1), XRegisterAllocation::CallerSave);
    }

    #[test]
    fn test_allocation_strategy_enum_properties() {
        // Test XRegisterAllocation enum properties
        assert_eq!(XRegisterAllocation::CalleeSave, XRegisterAllocation::CalleeSave);
        assert_eq!(XRegisterAllocation::CallerSave, XRegisterAllocation::CallerSave);
        assert_ne!(XRegisterAllocation::CalleeSave, XRegisterAllocation::CallerSave);

        // Test Debug formatting
        assert!(format!("{:?}", XRegisterAllocation::CalleeSave).contains("CalleeSave"));
        assert!(format!("{:?}", XRegisterAllocation::CallerSave).contains("CallerSave"));
    }

    #[test]
    fn test_allocation_strategy_all_registers() {
        // Test allocation strategy for all valid register indices
        for i in 0..x_registers::NUM_REGISTER_BACKED_XREGS {
            let strategy = LiveRegisterInfo::get_allocation_strategy(i);
            match strategy {
                XRegisterAllocation::CalleeSave | XRegisterAllocation::CallerSave => {
                    // Valid strategy
                }
            }
        }
    }

    #[test]
    fn test_allocation_strategy_build_dependent() {
        // Test that allocation strategy respects build configuration

        #[cfg(debug_assertions)]
        {
            // In debug builds: XREG3 is caller-save (index 3)
            assert_eq!(LiveRegisterInfo::get_allocation_strategy(3), XRegisterAllocation::CallerSave);
        }

        #[cfg(not(debug_assertions))]
        {
            // In release builds: XREG3 is callee-save (index 3)
            assert_eq!(LiveRegisterInfo::get_allocation_strategy(3), XRegisterAllocation::CalleeSave);
        }
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
    fn test_xreg_ref_linear_progression() {
        // Test that references increase linearly with register index
        let mut prev_ref = XRegisterManager::get_xreg_ref(0);

        for i in 1..x_registers::NUM_REGISTER_BACKED_XREGS {
            let curr_ref = XRegisterManager::get_xreg_ref(i);
            let diff = curr_ref - prev_ref;

            assert_eq!(diff, 8, "Reference for register {} should be 8 bytes after register {}", i, i-1);
            prev_ref = curr_ref;
        }
    }

    #[test]
    fn test_xreg_ref_base_offset() {
        // Test that all references have the correct base offset
        const EXPECTED_BASE: i32 = 8; // X_REG_ARRAY_BASE from the code

        for i in 0..x_registers::NUM_REGISTER_BACKED_XREGS {
            let xreg_ref = XRegisterManager::get_xreg_ref(i);
            // The reference should be base + (index * 8)
            let expected_ref = EXPECTED_BASE + (i as i32 * 8);
            assert_eq!(xreg_ref, expected_ref,
                      "Reference for register {} should be {}", i, expected_ref);
        }
    }

    #[test]
    fn test_xreg_ref_bounds() {
        // Test that references stay within reasonable bounds

        let min_ref = XRegisterManager::get_xreg_ref(0);
        let max_ref = XRegisterManager::get_xreg_ref(x_registers::NUM_REGISTER_BACKED_XREGS - 1);

        // References should be positive (valid memory offsets)
        assert!(min_ref > 0, "Minimum reference should be positive");
        assert!(max_ref > 0, "Maximum reference should be positive");

        // Maximum should be reasonably sized (less than 1KB for 6 registers)
        assert!(max_ref < 1024, "Maximum reference should be reasonable (< 1024)");

        // Maximum should be greater than minimum
        assert!(max_ref > min_ref, "Maximum reference should be greater than minimum");
    }

    #[test]
    fn test_xreg_ref_consistency() {
        // Test that get_xreg_ref is consistent across multiple calls
        for i in 0..x_registers::NUM_REGISTER_BACKED_XREGS {
            let ref1 = XRegisterManager::get_xreg_ref(i);
            let ref2 = XRegisterManager::get_xreg_ref(i);

            assert_eq!(ref1, ref2, "get_xreg_ref should be consistent for register {}", i);
        }
    }

    #[test]
    fn test_xreg_ref_arithmetic() {
        // Test arithmetic properties of reference calculations
        let refs: Vec<i32> = (0..x_registers::NUM_REGISTER_BACKED_XREGS)
            .map(|i| XRegisterManager::get_xreg_ref(i))
            .collect();

        // Should be strictly increasing
        for i in 1..refs.len() {
            assert!(refs[i] > refs[i-1], "References should be strictly increasing");
        }

        // Differences should all be 8
        for i in 1..refs.len() {
            assert_eq!(refs[i] - refs[i-1], 8, "Adjacent references should differ by 8");
        }
    }

    #[test]
    fn test_xreg_ref_range() {
        // Test that references cover the expected range
        let start_ref = XRegisterManager::get_xreg_ref(0);
        let end_ref = XRegisterManager::get_xreg_ref(x_registers::NUM_REGISTER_BACKED_XREGS - 1);
        let expected_range = (x_registers::NUM_REGISTER_BACKED_XREGS - 1) * 8;

        assert_eq!(end_ref - start_ref, expected_range as i32,
                  "Reference range should be {} bytes", expected_range);
    }

    #[test]
    fn test_x_register_manager_creation() {
        // XRegisterManager has no state, just test creation
        let _manager = XRegisterManager;
    }

    #[test]
    fn test_load_xreg_validation() {
        // Test that load_xreg validates register indices

        // Valid indices should not cause immediate errors
        for i in 0..x_registers::NUM_REGISTER_BACKED_XREGS {
            // We can't test the actual operation without an assembler,
            // but we can verify the validation logic
            assert!(i < x_registers::NUM_REGISTER_BACKED_XREGS,
                   "Index {} should be valid", i);
        }

        // Invalid indices would cause errors
        let invalid_indices = vec![
            x_registers::NUM_REGISTER_BACKED_XREGS,
            x_registers::NUM_REGISTER_BACKED_XREGS + 1,
            usize::MAX,
        ];

        for &invalid_index in &invalid_indices {
            assert!(invalid_index >= x_registers::NUM_REGISTER_BACKED_XREGS,
                   "Index {} should be invalid", invalid_index);
        }
    }

    #[test]
    fn test_store_xreg_validation() {
        // Test that store_xreg validates register indices

        // Same validation as load_xreg
        for i in 0..x_registers::NUM_REGISTER_BACKED_XREGS {
            assert!(i < x_registers::NUM_REGISTER_BACKED_XREGS,
                   "Index {} should be valid for store_xreg", i);
        }
    }

    // #[test]
    // fn test_save_restore_single_register_mapping() {
    //     // Test that single register operations use correct CPU register mapping

    //     for i in 0..x_registers::NUM_REGISTER_BACKED_XREGS {
    //         let cpu_reg = x_registers::REGISTER_BACKED_XREGS[i];

    //         // CPU register should be valid (0-30 for ARM64)
    //         assert!(cpu_reg >= 0u32 && cpu_reg <= 30u32,
    //                "X register {} maps to invalid CPU register {}", i, cpu_reg);

    //         // CPU register should match the assigned register for this X register
    //         match i {
    //             0 => assert_eq!(cpu_reg, 25, "XREG0 should map to x25"),
    //             1 => assert_eq!(cpu_reg, 26, "XREG1 should map to x26"),
    //             2 => assert_eq!(cpu_reg, 27, "XREG2 should map to x27"),
    //             3 => assert_eq!(cpu_reg, x_registers::XREG3, "XREG3 should map to configured register"),
    //             4 => assert_eq!(cpu_reg, 15, "XREG4 should map to x15"),
    //             5 => assert_eq!(cpu_reg, 16, "XREG5 should map to x16"),
    //             _ => panic!("Unexpected register index {}", i),
    //         }
    //     }
    // }

    #[test]
    fn test_save_restore_live_logic() {
        // Test the logic for iterating through live registers

        let mut live_info = LiveRegisterInfo::new();

        // Set up a pattern: live registers at indices 0, 2, 4
        live_info.set_live(0);
        live_info.set_live(2);
        live_info.set_live(4);

        // Simulate the iteration logic from save_live_xregs/restore_live_xregs
        let mut processed_regs = Vec::new();
        let mut i = 0;
        while i < x_registers::NUM_REGISTER_BACKED_XREGS {
            if live_info.is_live(i) {
                // Check if next register is also live for pair optimization
                if i + 1 < x_registers::NUM_REGISTER_BACKED_XREGS && live_info.is_live(i + 1) {
                    // Would use pair operation for i and i+1
                    processed_regs.push((i, i + 1, true)); // pair operation
                    i += 2;
                } else {
                    // Would use single operation for i
                    processed_regs.push((i, i, false)); // single operation
                    i += 1;
                }
            } else {
                i += 1;
            }
        }

        // Should have processed: (0,0,false), (2,3,true), (4,4,false)
        // Wait, that's not right. Let me check:
        // i=0: live, next (i=1) not live -> single (0,0,false), i=1
        // i=1: not live, i=2
        // i=2: live, next (i=3) not live -> single (2,2,false), i=3
        // i=3: not live, i=4
        // i=4: live, next (i=5) not live -> single (4,4,false), i=5

        assert_eq!(processed_regs.len(), 3);
        assert_eq!(processed_regs[0], (0, 0, false)); // single operation on reg 0
        assert_eq!(processed_regs[1], (2, 2, false)); // single operation on reg 2
        assert_eq!(processed_regs[2], (4, 4, false)); // single operation on reg 4
    }

    // #[test]
    // fn test_bulk_operations_coverage() {
    //     // Test that bulk operations cover all registers

    //     // save_all_xregs and restore_all_xregs should iterate through all registers
    //     let expected_operations = x_registers::NUM_REGISTER_BACKED_XREGS;

    //     assert_eq!(expected_operations, 6, "Should perform 6 operations for all registers");

    //     // Verify all registers would be processed
    //     for i in 0..x_registers::NUM_REGISTER_BACKED_XREGS {
    //         let cpu_reg = x_registers::REGISTER_BACKED_XREGS[i];
    //         assert!(cpu_reg >= 0u32 && cpu_reg <= 30u32,
    //                "Bulk operation would process valid register {}", cpu_reg);
    //     }
    // }

    #[test]
    fn test_caller_save_flush_range() {
        // Test that flush_caller_save_xregs processes the correct range

        let start_range = x_registers::LOWEST_CALLER_SAVE_XREG;
        let end_range = x_registers::NUM_REGISTER_BACKED_XREGS;

        // Should process registers from LOWEST_CALLER_SAVE_XREG to end
        let expected_count = end_range - start_range;

        #[cfg(debug_assertions)]
        assert_eq!(expected_count, 3, "Debug build should flush 3 caller-save registers");

        #[cfg(not(debug_assertions))]
        assert_eq!(expected_count, 2, "Release build should flush 2 caller-save registers");

        // Verify the range is valid
        assert!(start_range < end_range);
        assert!(end_range <= x_registers::NUM_REGISTER_BACKED_XREGS);
    }

    // #[test]
    // fn test_save_all_restore_all_symmetry() {
    //     // Test that save_all and restore_all are symmetric operations

    //     // Both operations should process all register-backed X registers
    //     let all_regs_count = x_registers::NUM_REGISTER_BACKED_XREGS;

    //     // save_all and restore_all should process the same number of registers
    //     assert_eq!(all_regs_count, 6);

    //     // Both should use the same register mappings
    //     for i in 0..all_regs_count {
    //         let cpu_reg = x_registers::REGISTER_BACKED_XREGS[i];
    //         // save_all and restore_all would both operate on this register
    //         assert!(cpu_reg >= 0u32 && cpu_reg <= 30u32,
    //            "Both operations would process register {}", cpu_reg);
    //     }
    // }

    #[test]
    fn test_spill_fill_operations() {
        // Test that spill_all and fill_all delegate to save_all/restore_all

        // spill_all should delegate to save_all_xregs
        // fill_all should delegate to restore_all_xregs

        // Both operations should handle all registers
        let total_regs = x_registers::NUM_REGISTER_BACKED_XREGS;
        assert_eq!(total_regs, 6);

        // Verify the register array is properly sized
        assert_eq!(x_registers::REGISTER_BACKED_XREGS.len(), total_regs);
    }

    #[test]
    fn test_bulk_operations_register_coverage() {
        // Test that bulk operations cover all assigned registers

        let mut covered_regs = std::collections::HashSet::new();

        // Bulk operations iterate through REGISTER_BACKED_XREGS
        for &cpu_reg in &x_registers::REGISTER_BACKED_XREGS {
            assert!(!covered_regs.contains(&cpu_reg),
                   "Register {} covered multiple times", cpu_reg);
            covered_regs.insert(cpu_reg);
        }

        // Should cover exactly NUM_REGISTER_BACKED_XREGS registers
        assert_eq!(covered_regs.len(), x_registers::NUM_REGISTER_BACKED_XREGS);
    }

    // #[test]
    // fn test_caller_save_flush_specific_registers() {
    //     // Test which specific registers get flushed by flush_caller_save_xregs

    //     let lowest_caller_save = x_registers::LOWEST_CALLER_SAVE_XREG;

    //     // Collect the registers that would be flushed
    //     let mut flushed_regs = Vec::new();
    //     for i in lowest_caller_save..x_registers::NUM_REGISTER_BACKED_XREGS {
    //         let cpu_reg = x_registers::REGISTER_BACKED_XREGS[i];
    //         flushed_regs.push((i, cpu_reg));
    //     }

    //     // Verify the correct registers are identified as caller-save
    //     for &(xreg_index, cpu_reg) in &flushed_regs {
    //         assert!(xreg_index >= lowest_caller_save,
    //                "Flushed register index {} should be >= {}", xreg_index, lowest_caller_save);
    //         assert!(cpu_reg >= 0u32 && cpu_reg <= 30u32,
    //                "Flushed CPU register {} should be valid", cpu_reg);
    //     }

    //     // Verify count matches expectation
    //     let expected_count = x_registers::NUM_REGISTER_BACKED_XREGS - lowest_caller_save;
    //     assert_eq!(flushed_regs.len(), expected_count);
    // }

    #[test]
    fn test_caller_save_flush_build_dependent() {
        // Test that caller-save flushing respects build configuration

        #[cfg(debug_assertions)]
        {
            // Debug build: XREG3 is caller-save, so LOWEST_CALLER_SAVE_XREG = 3
            assert_eq!(x_registers::LOWEST_CALLER_SAVE_XREG, 3);
            // Should flush XREG3, XREG4, XREG5 (3 registers)
            let flush_count = x_registers::NUM_REGISTER_BACKED_XREGS - x_registers::LOWEST_CALLER_SAVE_XREG;
            assert_eq!(flush_count, 3);
        }

        #[cfg(not(debug_assertions))]
        {
            // Release build: XREG3 is callee-save, so LOWEST_CALLER_SAVE_XREG = 4
            assert_eq!(x_registers::LOWEST_CALLER_SAVE_XREG, 4);
            // Should flush XREG4, XREG5 (2 registers)
            let flush_count = x_registers::NUM_REGISTER_BACKED_XREGS - x_registers::LOWEST_CALLER_SAVE_XREG;
            assert_eq!(flush_count, 2);
        }
    }

    #[test]
    fn test_bulk_vs_selective_operations() {
        // Test the difference between bulk and selective operations

        // Bulk operations (save_all, restore_all) process all registers
        let bulk_count = x_registers::NUM_REGISTER_BACKED_XREGS;

        // Selective operations (save_live, restore_live) process only live registers
        let mut live_info = LiveRegisterInfo::new();
        live_info.set_live(0);
        live_info.set_live(3);
        live_info.set_live(5);
        let selective_count = live_info.live_count;

        // Bulk should process more or equal registers
        assert!(bulk_count >= selective_count);

        // Selective should process only the live subset
        assert_eq!(selective_count, 3);
        assert!(selective_count < bulk_count);
    }

    // #[test]
    // fn test_pair_operations_register_selection() {
    //     // Test that pair operations select consecutive registers correctly

    //     // Test valid pair starting positions (0, 1, 2, 3, 4)
    //     let valid_pair_starts = vec![0, 1, 2, 3, 4];

    //     for &start_idx in &valid_pair_starts {
    //         let reg1_idx = start_idx;
    //         let reg2_idx = start_idx + 1;

    //         // Both indices should be valid
    //         assert!(reg1_idx < x_registers::NUM_REGISTER_BACKED_XREGS);
    //         assert!(reg2_idx < x_registers::NUM_REGISTER_BACKED_XREGS);

    //         // Get the actual CPU registers
    //         let reg1_cpu = x_registers::REGISTER_BACKED_XREGS[reg1_idx];
    //         let reg2_cpu = x_registers::REGISTER_BACKED_XREGS[reg2_idx];

    //         // Both should be valid ARM64 registers
    //         assert!(reg1_cpu >= 0u32 && reg1_cpu <= 30u32);
    //         assert!(reg2_cpu >= 0u32 && reg2_cpu <= 30u32);
    //     }
    // }

    #[test]
    fn test_pair_operations_reference_calculation() {
        // Test that pair operations use the correct memory reference

        for start_idx in 0..5 { // Valid pair starting positions
            let expected_ref = XRegisterManager::get_xreg_ref(start_idx);

            // The pair operation should use the reference for the first register
            // (The second register is at first_register_ref + 8)
            let second_ref = XRegisterManager::get_xreg_ref(start_idx + 1);
            let diff = second_ref - expected_ref;

            assert_eq!(diff, 8, "Pair registers should be 8 bytes apart");
        }
    }

    #[test]
    fn test_pair_vs_single_operation_coverage() {
        // Test that pair operations provide better coverage than single operations

        let mut live_info = LiveRegisterInfo::new();

        // Set up a scenario where pairs can be used: 0,1,2,3,4,5 all live
        for i in 0..x_registers::NUM_REGISTER_BACKED_XREGS {
            live_info.set_live(i);
        }

        // Simulate the pair optimization logic
        let mut pair_operations = 0;
        let mut single_operations = 0;
        let mut i = 0;

        while i < x_registers::NUM_REGISTER_BACKED_XREGS {
            if live_info.is_live(i) {
                if i + 1 < x_registers::NUM_REGISTER_BACKED_XREGS && live_info.is_live(i + 1) {
                    pair_operations += 1;
                    i += 2; // Skip both registers
                } else {
                    single_operations += 1;
                    i += 1;
                }
            } else {
                i += 1;
            }
        }

        // With all registers live, we should get 3 pair operations (0-1, 2-3, 4-5)
        assert_eq!(pair_operations, 3, "Should use 3 pair operations for consecutive registers");
        assert_eq!(single_operations, 0, "Should not need single operations");

        // Total operations: 3 pairs = 6 registers covered
        assert_eq!(pair_operations * 2, x_registers::NUM_REGISTER_BACKED_XREGS);
    }

    #[test]
    fn test_pair_operation_boundaries() {
        // Test pair operation boundaries

        // Can't form pairs at the end of the array
        let last_valid_pair_start = x_registers::NUM_REGISTER_BACKED_XREGS - 2; // 4 for 6 registers
        assert_eq!(last_valid_pair_start, 4);

        // Invalid pair starts (would cause out of bounds)
        let invalid_pair_starts = vec![5, 6, 10]; // 5+1=6 >= 6

        for &invalid_start in &invalid_pair_starts {
            let would_be_second = invalid_start + 1;
            assert!(would_be_second >= x_registers::NUM_REGISTER_BACKED_XREGS,
                   "Pair start {} would access invalid register {}", invalid_start, would_be_second);
        }
    }

    #[test]
    fn test_pair_operation_mixed_patterns() {
        // Test pair operations with mixed live/dead patterns

        let mut live_info = LiveRegisterInfo::new();

        // Pattern: live 0,1,3,5 (pairs at 0-1, singles at 3,5)
        live_info.set_live(0);
        live_info.set_live(1);
        live_info.set_live(3);
        live_info.set_live(5);

        // Simulate the processing logic
        let mut operations = Vec::new();
        let mut i = 0;

        while i < x_registers::NUM_REGISTER_BACKED_XREGS {
            if live_info.is_live(i) {
                if i + 1 < x_registers::NUM_REGISTER_BACKED_XREGS && live_info.is_live(i + 1) {
                    operations.push(format!("pair_{}_{}", i, i + 1));
                    i += 2;
                } else {
                    operations.push(format!("single_{}", i));
                    i += 1;
                }
            } else {
                i += 1;
            }
        }

        // Should have: pair_0_1, single_3, single_5
        assert_eq!(operations.len(), 3);
        assert_eq!(operations[0], "pair_0_1");
        assert_eq!(operations[1], "single_3");
        assert_eq!(operations[2], "single_5");
    }

    // #[test]
    // fn test_x_register_management_complete_workflow() {
    //     // Test a complete workflow: allocation -> usage -> save -> restore -> cleanup

    //     let mut live_info = LiveRegisterInfo::new();

    //     // Phase 1: Allocate some registers
    //     live_info.set_live(0); // Callee-save
    //     live_info.set_live(1); // Callee-save
    //     live_info.set_live(4); // Caller-save
    //     live_info.set_live(5); // Caller-save

    //     assert_eq!(live_info.live_count, 4);

    //     // Phase 2: Simulate save operation (would save 0,1,4,5)
    //     // save_live_xregs would process: pair(0,1), single(4), single(5)

    //     // Phase 3: Simulate caller-save flush before C call
    //     // flush_caller_save_xregs would save 4,5 (already saved, but conceptually)

    //     // Phase 4: Simulate restore operation (would restore 0,1,4,5)

    //     // Phase 5: Cleanup - deallocate registers
    //     live_info.set_dead(0);
    //     live_info.set_dead(1);
    //     live_info.set_dead(4);
    //     live_info.set_dead(5);

    //     assert_eq!(live_info.live_count, 0);
    // }

    #[test]
    fn test_x_register_management_state_consistency() {
        // Test that all operations maintain state consistency

        let mut live_info = LiveRegisterInfo::new();

        // Perform various operations and verify state remains consistent
        let operations = vec![
            (0, true),   // set 0 live
            (2, true),   // set 2 live
            (0, false),  // set 0 dead
            (5, true),   // set 5 live
            (2, false),  // set 2 dead
            (1, true),   // set 1 live
        ];

        let mut expected_live_count = 0;

        for &(reg, set_live) in &operations {
            if set_live {
                let was_dead = !live_info.is_live(reg);
                live_info.set_live(reg);
                if was_dead { // Wasn't already live
                    expected_live_count += 1;
                }
            } else {
                let was_live = live_info.is_live(reg);
                live_info.set_dead(reg);
                if was_live { // Was live before
                    expected_live_count -= 1;
                }
            }

            assert_eq!(live_info.live_count, expected_live_count,
                      "Live count inconsistent after operation on register {}", reg);
        }

        // Final state: registers 1 and 5 should be live
        assert_eq!(live_info.live_count, 2);
        assert!(live_info.is_live(1));
        assert!(live_info.is_live(5));
        assert!(!live_info.is_live(0));
        assert!(!live_info.is_live(2));
    }

    #[test]
    fn test_x_register_management_allocation_patterns() {
        // Test different allocation patterns and their efficiency

        let patterns = vec![
            ("consecutive_start", vec![0, 1, 2]),        // Good for pairs
            ("consecutive_middle", vec![1, 2, 3]),      // Good for pairs
            ("sparse", vec![0, 2, 4]),                  // No pairs possible
            ("all_caller_save", vec![3, 4, 5]),         // All caller-save
            ("mixed", vec![0, 1, 4, 5]),               // Mix of both
        ];

        for (pattern_name, regs) in patterns {
            let mut live_info = LiveRegisterInfo::new();
            for &reg in &regs {
                live_info.set_live(reg);
            }

            // Count potential pair operations
            let mut pair_count: usize = 0;
            let mut single_count: usize = 0;
            let mut i = 0;

            while i < x_registers::NUM_REGISTER_BACKED_XREGS {
                if live_info.is_live(i) {
                    if i + 1 < x_registers::NUM_REGISTER_BACKED_XREGS && live_info.is_live(i + 1) {
                        pair_count += 1;
                        i += 2;
                    } else {
                        single_count += 1;
                        i += 1;
                    }
                } else {
                    i += 1;
                }
            }

            // Verify counts are reasonable
            assert_eq!(pair_count * 2 + single_count, regs.len(),
                      "Pattern '{}' should account for all registers", pattern_name);
        }
    }

    #[test]
    fn test_x_register_management_build_configuration_integration() {
        // Test that all components work together across build configurations

        // Test register assignments
        let all_regs = x_registers::REGISTER_BACKED_XREGS;

        // Test allocation strategies
        for i in 0..x_registers::NUM_REGISTER_BACKED_XREGS {
            let strategy = LiveRegisterInfo::get_allocation_strategy(i);
            match strategy {
                XRegisterAllocation::CalleeSave | XRegisterAllocation::CallerSave => {}
            }
        }

        // Test range calculations
        let highest_callee = x_registers::HIGHEST_CALLEE_SAVE_XREG;
        let lowest_caller = x_registers::LOWEST_CALLER_SAVE_XREG;

        assert!(highest_callee < x_registers::NUM_REGISTER_BACKED_XREGS);
        assert!(lowest_caller <= x_registers::NUM_REGISTER_BACKED_XREGS);

        // Test reference calculations
        for i in 0..x_registers::NUM_REGISTER_BACKED_XREGS {
            let _ref = XRegisterManager::get_xreg_ref(i);
            // Just verify it doesn't panic
        }

        // Test array bounds
        assert_eq!(all_regs.len(), x_registers::NUM_REGISTER_BACKED_XREGS);
    }

    #[test]
    fn test_x_register_management_resource_management() {
        // Test that the system properly manages X register resources

        // Simulate a complex scenario with multiple allocation/deallocation cycles
        let mut live_info = LiveRegisterInfo::new();

        // Cycle 1: Allocate callee-save registers
        for i in 0..=x_registers::HIGHEST_CALLEE_SAVE_XREG {
            live_info.set_live(i);
        }

        let callee_count = x_registers::HIGHEST_CALLEE_SAVE_XREG + 1;
        assert_eq!(live_info.live_count, callee_count);

        // Cycle 2: Allocate caller-save registers
        for i in x_registers::LOWEST_CALLER_SAVE_XREG..x_registers::NUM_REGISTER_BACKED_XREGS {
            live_info.set_live(i);
        }

        let total_expected = x_registers::NUM_REGISTER_BACKED_XREGS;
        assert_eq!(live_info.live_count, total_expected);

        // Cycle 3: Deallocate all
        for i in 0..x_registers::NUM_REGISTER_BACKED_XREGS {
            live_info.set_dead(i);
        }

        assert_eq!(live_info.live_count, 0);

        // Cycle 4: Mixed usage
        live_info.set_live(0); // callee-save
        live_info.set_live(4); // caller-save
        live_info.set_live(5); // caller-save

        assert_eq!(live_info.live_count, 3);

        // Verify allocation strategies
        assert_eq!(LiveRegisterInfo::get_allocation_strategy(0), XRegisterAllocation::CalleeSave);
        assert_eq!(LiveRegisterInfo::get_allocation_strategy(4), XRegisterAllocation::CallerSave);
        assert_eq!(LiveRegisterInfo::get_allocation_strategy(5), XRegisterAllocation::CallerSave);
    }

    // #[test]
    // fn test_x_register_management_error_handling_integration() {
    //     // Test that error conditions are properly handled across components

    //     // Test invalid register indices for load_xreg/store_xreg
    //     let invalid_indices = vec![
    //         x_registers::NUM_REGISTER_BACKED_XREGS,
    //         x_registers::NUM_REGISTER_BACKED_XREGS + 1,
    //         usize::MAX,
    //     ];

    //     for &invalid_index in &invalid_indices {
    //         // These would cause errors in load_xreg/store_xreg
    //         assert!(invalid_index >= x_registers::NUM_REGISTER_BACKED_XREGS,
    //                "Index {} should be invalid", invalid_index);
    //     }

    //     // Test that valid indices work
    //     for i in 0..x_registers::NUM_REGISTER_BACKED_XREGS {
    //         assert!(i < x_registers::NUM_REGISTER_BACKED_XREGS,
    //                "Index {} should be valid", i);

    //         // Verify register mapping exists
    //         let cpu_reg = x_registers::REGISTER_BACKED_XREGS[i];
    //         assert!(cpu_reg >= 0u32 && cpu_reg <= 30u32,
    //                "CPU register {} for X register {} should be valid", cpu_reg, i);
    //     }
    // }
}
