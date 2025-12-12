//! Test BEAM Code Generation
//!
//! Provides utilities for creating test BEAM code sequences.
//! This is used for testing instruction execution.

use entities_process::ErtsCodePtr;

/// Create a simple test BEAM code sequence
///
/// Creates a minimal BEAM code that:
/// 1. Moves a value between registers
/// 2. Returns
///
/// This is a simplified representation - real BEAM code is more complex.
///
/// # Returns
/// Pointer to allocated test code (must be kept alive during execution)
pub fn create_test_code() -> Vec<u64> {
    use infrastructure_emulator_loop::instruction_decoder::opcodes;
    
    // Create a simple test program:
    // move x(0) x(1)  - move register 0 to register 1
    // return          - return from function
    
    let mut code = Vec::new();
    
    // Instruction 1: move x(0) x(1)
    // Format: [opcode:u8, src:u64, dst:u64]
    // For simplicity, we'll pack this into u64 words
    // Word 0: opcode in lower byte, rest padding
    let move_instr = opcodes::MOVE as u64;
    code.push(move_instr);
    code.push(0); // src = x(0)
    code.push(1); // dst = x(1)
    
    // Instruction 2: return
    // Format: [opcode:u8]
    let return_instr = opcodes::RETURN as u64;
    code.push(return_instr);
    
    code
}

/// Get pointer to test code
///
/// # Arguments
/// * `code` - Code vector (must be kept alive)
///
/// # Returns
/// Pointer to first instruction
pub fn get_code_ptr(code: &[u64]) -> ErtsCodePtr {
    if code.is_empty() {
        return std::ptr::null();
    }
    code.as_ptr() as ErtsCodePtr
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_code() {
        let code = create_test_code();
        assert!(!code.is_empty());
        assert!(code.len() >= 3); // At least move + return
    }

    #[test]
    fn test_get_code_ptr() {
        let code = create_test_code();
        let ptr = get_code_ptr(&code);
        assert!(!ptr.is_null());
    }
}

