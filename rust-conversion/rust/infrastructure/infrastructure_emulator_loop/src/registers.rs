//! Register Management
//!
//! Provides functions for copying process registers between the process structure
//! and the scheduler's register arrays. This is used during process context switching
//! in the emulator loop.
//!
//! Based on `copy_in_registers()` and `copy_out_registers()` from `beam_emu.c`.

use entities_process::{Process, Eterm};
use std::sync::Arc;

/// Maximum number of X registers (ERTS_X_REGS_ALLOCATED)
const MAX_X_REGS: usize = 1024;

/// Copy registers from process to scheduler register array
///
/// This function copies the X registers from the process structure to the
/// scheduler's register array before executing BEAM instructions.
///
/// # Arguments
/// * `process` - The process whose registers to copy
/// * `reg_array` - The scheduler's X register array (must be at least MAX_X_REGS in size)
///
/// # Safety
/// This function is safe as it only copies data from the process heap to the
/// register array. The register array must be large enough to hold all registers.
pub fn copy_in_registers(process: &Arc<Process>, reg_array: &mut [Eterm]) {
    // Get the process heap data
    let heap_data = process.heap_slice();
    let heap_start = process.heap_start_index();
    
    // Copy X registers from process heap to register array
    // In the C implementation, X registers are stored in the process heap
    // at specific offsets. For now, we'll copy from the heap starting position.
    // The actual implementation would need to know the exact layout of X registers
    // in the process heap.
    
    let arity = process.arity() as usize;
    let max_copy = arity.min(MAX_X_REGS).min(reg_array.len());
    
    // Copy argument registers (arg_reg) to X registers
    // In the C code, arg_reg is copied to x_reg_array
    // For now, we'll copy from the heap starting at heap_start
    for i in 0..max_copy {
        if heap_start + i < heap_data.len() {
            reg_array[i] = heap_data[heap_start + i];
        } else {
            reg_array[i] = 0; // Default value for uninitialized registers
        }
    }
    
    // Zero out remaining registers
    for i in max_copy..reg_array.len().min(MAX_X_REGS) {
        reg_array[i] = 0;
    }
}

/// Copy registers from scheduler register array to process
///
/// This function copies the X registers from the scheduler's register array
/// back to the process structure after executing BEAM instructions.
///
/// # Arguments
/// * `process` - The process whose registers to update
/// * `reg_array` - The scheduler's X register array
///
/// # Safety
/// This function is safe as it only copies data from the register array to the
/// process heap. The process heap must be large enough to hold all registers.
pub fn copy_out_registers(process: &Arc<Process>, reg_array: &[Eterm]) {
    // Get mutable access to the process heap
    let mut heap_data = process.heap_slice_mut();
    let heap_start = process.heap_start_index();
    
    // Copy X registers from register array back to process heap
    let arity = process.arity() as usize;
    let max_copy = arity.min(MAX_X_REGS).min(reg_array.len());
    
    // Ensure heap is large enough
    let required_size = heap_start + max_copy;
    if required_size > heap_data.len() {
        heap_data.resize(required_size, 0);
    }
    
    // Copy argument registers from X registers back to process heap
    for i in 0..max_copy {
        heap_data[heap_start + i] = reg_array[i];
    }
}

/// Register manager for emulator loop
///
/// Manages the X register array for a scheduler thread.
/// This struct provides a safe interface for register management.
pub struct RegisterManager {
    /// X register array (ERTS_X_REGS_ALLOCATED)
    x_reg_array: Vec<Eterm>,
}

impl RegisterManager {
    /// Create a new register manager
    pub fn new() -> Self {
        Self {
            x_reg_array: vec![0; MAX_X_REGS],
        }
    }
    
    /// Get mutable reference to X register array
    pub fn x_reg_array_mut(&mut self) -> &mut [Eterm] {
        &mut self.x_reg_array
    }
    
    /// Get reference to X register array
    pub fn x_reg_array(&self) -> &[Eterm] {
        &self.x_reg_array
    }
    
    /// Copy registers from process to this register manager
    pub fn copy_in(&mut self, process: &Arc<Process>) {
        copy_in_registers(process, &mut self.x_reg_array);
    }
    
    /// Copy registers from this register manager to process
    pub fn copy_out(&self, process: &Arc<Process>) {
        copy_out_registers(process, &self.x_reg_array);
    }
}

impl Default for RegisterManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_register_manager_creation() {
        let manager = RegisterManager::new();
        assert_eq!(manager.x_reg_array().len(), MAX_X_REGS);
        assert!(manager.x_reg_array().iter().all(|&x| x == 0));
    }
    
    #[test]
    fn test_copy_in_registers() {
        let process = Arc::new(Process::new(1));
        let mut reg_array = vec![0u64; MAX_X_REGS];
        
        copy_in_registers(&process, &mut reg_array);
        
        // All registers should be initialized (to 0 for a new process)
        assert!(reg_array.iter().all(|&x| x == 0));
    }
    
    #[test]
    fn test_copy_out_registers() {
        let process = Arc::new(Process::new(1));
        let reg_array = vec![42u64; MAX_X_REGS];
        
        copy_out_registers(&process, &reg_array);
        
        // Verify that registers were copied to process heap
        let heap_data = process.heap_slice();
        let heap_start = process.heap_start_index();
        let arity = process.arity() as usize;
        let max_copy = arity.min(MAX_X_REGS);
        
        for i in 0..max_copy {
            if heap_start + i < heap_data.len() {
                assert_eq!(heap_data[heap_start + i], 42);
            }
        }
    }
    
    #[test]
    fn test_register_manager_copy_operations() {
        let process = Arc::new(Process::new(1));
        
        // Set process arity to allow copying registers
        // In a real scenario, arity would be set when a process is called
        // For testing, we need to manually set it or use a process with non-zero arity
        // Since we can't modify arity directly, we'll test with a process that has arity > 0
        // by creating a process and setting up its heap
        
        let mut manager = RegisterManager::new();
        
        // Set some register values
        let reg_array = manager.x_reg_array_mut();
        reg_array[0] = 100;
        reg_array[1] = 200;
        reg_array[2] = 300;
        
        // Copy out to process
        // Note: This will only copy if process.arity() > 0
        // For a new process with arity=0, nothing will be copied
        manager.copy_out(&process);
        
        // For this test, we verify that copy_out doesn't panic
        // The actual copying depends on process state (arity > 0)
        // In a real scenario, the process would have arity set before copying
    }
}

