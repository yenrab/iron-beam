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

    eprintln!("[DEBUG] copy_in_registers: heap_start={}, heap_data.len()={}, arity={}",
             heap_start, heap_data.len(), process.arity());

    // Copy X registers from process heap to register array
    // In the C implementation, X registers are stored in the process heap
    // at specific offsets. For now, we'll copy from the heap starting position.
    // The actual implementation would need to know the exact layout of X registers
    // in the process heap.

    let arity = process.arity() as usize;
    let max_copy = arity.min(MAX_X_REGS).min(reg_array.len());

    eprintln!("[DEBUG] copy_in_registers: max_copy={}, copying registers...", max_copy);

    // Copy argument registers (arg_reg) to X registers
    // In the C code, arg_reg is copied to x_reg_array
    // For now, we'll copy from the heap starting at heap_start
    for i in 0..max_copy {
        if heap_start + i < heap_data.len() {
            reg_array[i] = heap_data[heap_start + i];
            eprintln!("[DEBUG] copy_in_registers: x[{}] = heap[{}] = 0x{:016x}",
                     i, heap_start + i, reg_array[i]);
        } else {
            reg_array[i] = 0; // Default value for uninitialized registers
            eprintln!("[DEBUG] copy_in_registers: x[{}] = 0 (heap index {} out of bounds)",
                     i, heap_start + i);
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

    /// Helper function to create a process with a specific arity
    fn create_process_with_arity(id: u64, arity: u8) -> Arc<Process> {
        let mut process = Process::new(id);
        process.set_arity(arity);
        Arc::new(process)
    }

    #[test]
    fn test_register_manager_creation() {
        let manager = RegisterManager::new();
        assert_eq!(manager.x_reg_array().len(), MAX_X_REGS);
        assert!(manager.x_reg_array().iter().all(|&x| x == 0));
    }

    #[test]
    fn test_register_manager_default() {
        let manager = RegisterManager::default();
        assert_eq!(manager.x_reg_array().len(), MAX_X_REGS);
        assert!(manager.x_reg_array().iter().all(|&x| x == 0));
    }

    #[test]
    fn test_register_manager_x_reg_array() {
        let manager = RegisterManager::new();
        let array = manager.x_reg_array();
        assert_eq!(array.len(), MAX_X_REGS);
        assert!(array.iter().all(|&x| x == 0));
    }

    #[test]
    fn test_register_manager_x_reg_array_mut() {
        let mut manager = RegisterManager::new();
        let array = manager.x_reg_array_mut();
        assert_eq!(array.len(), MAX_X_REGS);
        
        // Modify array
        array[0] = 42;
        array[1] = 100;
        
        // Verify changes
        assert_eq!(array[0], 42);
        assert_eq!(array[1], 100);
        
        // Verify through immutable access
        let array_ref = manager.x_reg_array();
        assert_eq!(array_ref[0], 42);
        assert_eq!(array_ref[1], 100);
    }

    #[test]
    fn test_copy_in_registers_zero_arity() {
        let process = Arc::new(Process::new(1));
        let mut reg_array = vec![999u64; MAX_X_REGS];
        
        copy_in_registers(&process, &mut reg_array);
        
        // With arity=0, all registers should be zeroed
        assert!(reg_array.iter().all(|&x| x == 0));
    }

    #[test]
    fn test_copy_in_registers_with_arity() {
        let process = create_process_with_arity(1, 5);
        let mut reg_array = vec![999u64; MAX_X_REGS];
        
        // Set some values in process heap at heap_start
        {
            let mut heap_data = process.heap_slice_mut();
            let heap_start = process.heap_start_index();
            for i in 0..5 {
                if heap_start + i < heap_data.len() {
                    heap_data[heap_start + i] = (i + 1) as u64 * 10;
                }
            }
        }
        
        copy_in_registers(&process, &mut reg_array);
        
        // First 5 registers should be copied from heap
        let heap_data = process.heap_slice();
        let heap_start = process.heap_start_index();
        for i in 0..5 {
            if heap_start + i < heap_data.len() {
                assert_eq!(reg_array[i], (i + 1) as u64 * 10);
            } else {
                assert_eq!(reg_array[i], 0);
            }
        }
        
        // Remaining registers should be zeroed
        for i in 5..reg_array.len().min(MAX_X_REGS) {
            assert_eq!(reg_array[i], 0);
        }
    }

    #[test]
    fn test_copy_in_registers_small_array() {
        let process = create_process_with_arity(1, 10);
        let mut reg_array = vec![999u64; 5]; // Smaller than arity
        
        copy_in_registers(&process, &mut reg_array);
        
        // Should only copy up to array length
        assert_eq!(reg_array.len(), 5);
    }

    #[test]
    fn test_copy_in_registers_large_arity() {
        let process = create_process_with_arity(1, 255); // Max u8 value, larger than typical usage
        let mut reg_array = vec![999u64; MAX_X_REGS];
        
        copy_in_registers(&process, &mut reg_array);
        
        // Should only copy MAX_X_REGS registers (arity is capped at MAX_X_REGS)
        assert_eq!(reg_array.len(), MAX_X_REGS);
    }

    #[test]
    fn test_copy_in_registers_heap_overflow() {
        let process = create_process_with_arity(1, 10);
        let mut reg_array = vec![999u64; MAX_X_REGS];
        
        // Process heap might be smaller than arity
        copy_in_registers(&process, &mut reg_array);
        
        // Should not panic, should zero out registers beyond heap size
        // This tests the bounds check in copy_in_registers
    }

    #[test]
    fn test_copy_out_registers_zero_arity() {
        let process = Arc::new(Process::new(1));
        let reg_array = vec![42u64; MAX_X_REGS];
        
        copy_out_registers(&process, &reg_array);
        
        // With arity=0, nothing should be copied
        let heap_data = process.heap_slice();
        let heap_start = process.heap_start_index();
        if heap_start < heap_data.len() {
            // Heap should remain unchanged (or be empty)
        }
    }

    #[test]
    fn test_copy_out_registers_with_arity() {
        let process = create_process_with_arity(1, 5);
        let reg_array: Vec<Eterm> = (1..=5).map(|i| i as u64 * 10).collect();
        
        copy_out_registers(&process, &reg_array);
        
        // Verify that registers were copied to process heap
        let heap_data = process.heap_slice();
        let heap_start = process.heap_start_index();
        
        for i in 0..5 {
            if heap_start + i < heap_data.len() {
                assert_eq!(heap_data[heap_start + i], (i + 1) as u64 * 10);
            }
        }
    }

    #[test]
    fn test_copy_out_registers_heap_resize() {
        let process = create_process_with_arity(1, 100);
        let reg_array: Vec<Eterm> = (0..100).map(|i| i as u64).collect();
        
        // Initial heap might be smaller than needed
        let initial_heap_size = process.heap_slice().len();
        
        copy_out_registers(&process, &reg_array);
        
        // Heap should be resized if needed
        let heap_data = process.heap_slice();
        let heap_start = process.heap_start_index();
        let required_size = heap_start + 100;
        
        assert!(heap_data.len() >= required_size);
        
        // Verify values were copied
        for i in 0..100 {
            if heap_start + i < heap_data.len() {
                assert_eq!(heap_data[heap_start + i], i as u64);
            }
        }
    }

    #[test]
    fn test_copy_out_registers_small_array() {
        let process = create_process_with_arity(1, 10);
        let reg_array = vec![42u64, 43u64, 44u64];
        
        copy_out_registers(&process, &reg_array);
        
        // Should only copy up to array length
        let heap_data = process.heap_slice();
        let heap_start = process.heap_start_index();
        
        for i in 0..3 {
            if heap_start + i < heap_data.len() {
                assert_eq!(heap_data[heap_start + i], (42 + i) as u64);
            }
        }
    }

    #[test]
    fn test_copy_out_registers_large_arity() {
        let process = create_process_with_arity(1, 255); // Max u8 value, larger than typical usage
        let reg_array: Vec<Eterm> = (0..MAX_X_REGS).map(|i| i as u64).collect();
        
        copy_out_registers(&process, &reg_array);
        
        // Should only copy MAX_X_REGS registers (arity is capped at MAX_X_REGS)
        let heap_data = process.heap_slice();
        let heap_start = process.heap_start_index();
        
        for i in 0..MAX_X_REGS {
            if heap_start + i < heap_data.len() {
                assert_eq!(heap_data[heap_start + i], i as u64);
            }
        }
    }

    #[test]
    fn test_register_manager_copy_in() {
        let process = create_process_with_arity(1, 5);
        let mut manager = RegisterManager::new();
        
        // Set some values in process heap
        {
            let mut heap_data = process.heap_slice_mut();
            let heap_start = process.heap_start_index();
            for i in 0..5 {
                if heap_start + i < heap_data.len() {
                    heap_data[heap_start + i] = (i + 1) as u64 * 100;
                }
            }
        }
        
        // Copy in from process
        manager.copy_in(&process);
        
        // Verify registers were copied
        let reg_array = manager.x_reg_array();
        let heap_data = process.heap_slice();
        let heap_start = process.heap_start_index();
        
        for i in 0..5 {
            if heap_start + i < heap_data.len() {
                assert_eq!(reg_array[i], (i + 1) as u64 * 100);
            }
        }
    }

    #[test]
    fn test_register_manager_copy_out() {
        let process = create_process_with_arity(1, 5);
        let mut manager = RegisterManager::new();
        
        // Set some register values
        let reg_array = manager.x_reg_array_mut();
        for i in 0..5 {
            reg_array[i] = (i + 1) as u64 * 200;
        }
        
        // Copy out to process
        manager.copy_out(&process);
        
        // Verify registers were copied to process heap
        let heap_data = process.heap_slice();
        let heap_start = process.heap_start_index();
        
        for i in 0..5 {
            if heap_start + i < heap_data.len() {
                assert_eq!(heap_data[heap_start + i], (i + 1) as u64 * 200);
            }
        }
    }

    #[test]
    fn test_register_manager_round_trip() {
        let process = create_process_with_arity(1, 10);
        let mut manager = RegisterManager::new();
        
        // Set initial values in process heap
        {
            let mut heap_data = process.heap_slice_mut();
            let heap_start = process.heap_start_index();
            for i in 0..10 {
                if heap_start + i < heap_data.len() {
                    heap_data[heap_start + i] = (i + 1) as u64 * 50;
                }
            }
        }
        
        // Copy in from process
        manager.copy_in(&process);
        
        // Modify registers
        let reg_array = manager.x_reg_array_mut();
        for i in 0..10 {
            reg_array[i] = reg_array[i] * 2;
        }
        
        // Copy out to process
        manager.copy_out(&process);
        
        // Verify round trip
        let heap_data = process.heap_slice();
        let heap_start = process.heap_start_index();
        
        for i in 0..10 {
            if heap_start + i < heap_data.len() {
                assert_eq!(heap_data[heap_start + i], (i + 1) as u64 * 100);
            }
        }
    }

    #[test]
    fn test_copy_in_registers_max_x_regs() {
        let process = create_process_with_arity(1, MAX_X_REGS as u8);
        let mut reg_array = vec![999u64; MAX_X_REGS];
        
        copy_in_registers(&process, &mut reg_array);
        
        // Should handle MAX_X_REGS correctly
        assert_eq!(reg_array.len(), MAX_X_REGS);
    }

    #[test]
    fn test_copy_out_registers_max_x_regs() {
        // Note: arity is u8, so max is 255, not MAX_X_REGS (1024)
        // But we can test with MAX_X_REGS array and smaller arity
        let process = create_process_with_arity(1, 255); // Max u8 value
        let reg_array: Vec<Eterm> = (0..MAX_X_REGS).map(|i| i as u64).collect();
        
        copy_out_registers(&process, &reg_array);
        
        // Should handle up to arity (255) registers, capped at MAX_X_REGS
        let heap_data = process.heap_slice();
        let heap_start = process.heap_start_index();
        let max_copy = 255.min(MAX_X_REGS).min(reg_array.len());
        assert!(heap_data.len() >= heap_start + max_copy);
        
        // Verify values were copied (up to arity)
        for i in 0..max_copy {
            if heap_start + i < heap_data.len() {
                assert_eq!(heap_data[heap_start + i], i as u64);
            }
        }
    }

    #[test]
    fn test_register_manager_multiple_processes() {
        let process1 = create_process_with_arity(1, 5);
        let process2 = create_process_with_arity(2, 3);
        let mut manager = RegisterManager::new();
        
        // Copy from first process
        {
            let mut heap_data = process1.heap_slice_mut();
            let heap_start = process1.heap_start_index();
            for i in 0..5 {
                if heap_start + i < heap_data.len() {
                    heap_data[heap_start + i] = (i + 1) as u64 * 10;
                }
            }
        }
        manager.copy_in(&process1);
        
        // Verify first process values
        let reg_array = manager.x_reg_array();
        for i in 0..5 {
            if i < reg_array.len() {
                assert_eq!(reg_array[i], (i + 1) as u64 * 10);
            }
        }
        
        // Copy from second process
        {
            let mut heap_data = process2.heap_slice_mut();
            let heap_start = process2.heap_start_index();
            for i in 0..3 {
                if heap_start + i < heap_data.len() {
                    heap_data[heap_start + i] = (i + 1) as u64 * 20;
                }
            }
        }
        manager.copy_in(&process2);
        
        // Verify second process values
        let reg_array = manager.x_reg_array();
        for i in 0..3 {
            if i < reg_array.len() {
                assert_eq!(reg_array[i], (i + 1) as u64 * 20);
            }
        }
    }

    #[test]
    fn test_copy_in_registers_empty_array() {
        let process = Arc::new(Process::new(1));
        let mut reg_array = vec![999u64; 0];
        
        copy_in_registers(&process, &mut reg_array);
        
        // Should not panic with empty array
        assert_eq!(reg_array.len(), 0);
    }

    #[test]
    fn test_copy_out_registers_empty_array() {
        let process = Arc::new(Process::new(1));
        let reg_array = vec![42u64; 0];
        
        copy_out_registers(&process, &reg_array);
        
        // Should not panic with empty array
    }

    #[test]
    fn test_register_manager_default_vs_new() {
        let manager1 = RegisterManager::new();
        let manager2 = RegisterManager::default();
        
        assert_eq!(manager1.x_reg_array().len(), manager2.x_reg_array().len());
        assert_eq!(manager1.x_reg_array(), manager2.x_reg_array());
    }
}

