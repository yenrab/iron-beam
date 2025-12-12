//! Instruction Execution Framework
//!
//! Provides the framework for executing BEAM instructions. This module
//! defines the instruction execution interface and helpers for the emulator loop.
//!
//! Based on the instruction execution framework in beam_emu.c

use entities_process::{Process, ErtsCodePtr, Eterm};
use crate::instruction_decoder::{decode_instruction, opcodes};

/// Instruction execution result
///
/// Represents the result of executing a BEAM instruction. This is used
/// by the emulator loop to determine what to do next.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstructionResult {
    /// Continue executing (normal flow)
    Continue,
    /// Process should yield (out of reductions)
    Yield,
    /// Process exited normally
    NormalExit,
    /// Process exited with error
    ErrorExit,
    /// Trap to BIF or export
    Trap(ErtsCodePtr),
    /// Context switch needed
    ContextSwitch,
    /// Jump to new instruction pointer (for call/return)
    Jump(ErtsCodePtr),
}

/// Instruction executor trait
///
/// Trait for executing BEAM instructions. The emulator loop uses
/// implementations of this trait to execute instructions.
pub trait InstructionExecutor {
    /// Execute a single BEAM instruction
    ///
    /// # Arguments
    /// * `process` - Process executing the instruction
    /// * `instruction_ptr` - Pointer to the instruction
    /// * `registers` - X register array
    /// * `heap` - Process heap
    ///
    /// # Returns
    /// InstructionResult indicating what to do next
    fn execute_instruction(
        &self,
        process: &Process,
        instruction_ptr: ErtsCodePtr,
        registers: &mut [Eterm],
        heap: &mut [Eterm],
    ) -> Result<InstructionResult, String>;
}

/// Default instruction executor
///
/// Executes BEAM instructions by decoding them and dispatching to handlers.
pub struct DefaultInstructionExecutor;

impl InstructionExecutor for DefaultInstructionExecutor {
    fn execute_instruction(
        &self,
        _process: &Process,
        instruction_ptr: ErtsCodePtr,
        registers: &mut [Eterm],
        _heap: &mut [Eterm],
    ) -> Result<InstructionResult, String> {
        // Decode the instruction
        let decoded = decode_instruction(instruction_ptr)?;
        
        // Dispatch based on opcode
        match decoded.opcode {
            opcodes::MOVE => {
                // move Src Dst
                // Move value from source to destination register
                if decoded.operands.len() >= 2 {
                    let src = decoded.operands[0] as usize;
                    let dst = decoded.operands[1] as usize;
                    
                    if src < registers.len() && dst < registers.len() {
                        // For now, assume both are X registers
                        // In full implementation, we'd decode operand types (x, y, c, etc.)
                        registers[dst] = registers[src];
                    }
                }
                Ok(InstructionResult::Continue)
            }
            opcodes::CALL => {
                // call Arity Label
                // Call function at Label, save return address
                if decoded.operands.len() >= 2 {
                    let _arity = decoded.operands[0];
                    let label_offset = decoded.operands[1] as isize;
                    
                    // Calculate jump target (relative to current instruction)
                    unsafe {
                        let target = instruction_ptr.offset(label_offset);
                        return Ok(InstructionResult::Jump(target));
                    }
                }
                Ok(InstructionResult::Continue)
            }
            opcodes::CALL_LAST => {
                // call_last Arity Label Deallocate
                // Tail call - deallocate stack and jump
                if decoded.operands.len() >= 3 {
                    let _arity = decoded.operands[0];
                    let label_offset = decoded.operands[1] as isize;
                    let _deallocate = decoded.operands[2];
                    
                    // Calculate jump target
                    unsafe {
                        let target = instruction_ptr.offset(label_offset);
                        return Ok(InstructionResult::Jump(target));
                    }
                }
                Ok(InstructionResult::Continue)
            }
            opcodes::CALL_ONLY => {
                // call_only Arity Label
                // Tail call without deallocation
                if decoded.operands.len() >= 2 {
                    let _arity = decoded.operands[0];
                    let label_offset = decoded.operands[1] as isize;
                    
                    unsafe {
                        let target = instruction_ptr.offset(label_offset);
                        return Ok(InstructionResult::Jump(target));
                    }
                }
                Ok(InstructionResult::Continue)
            }
            opcodes::RETURN => {
                // return - exit function normally
                Ok(InstructionResult::NormalExit)
            }
            _ => {
                // Unknown instruction - continue for now
                // In full implementation, we'd handle all opcodes
                Ok(InstructionResult::Continue)
            }
        }
    }
}

/// Check if instruction pointer is valid
///
/// Based on VALID_INSTR macro from beam_emu.c
///
/// # Arguments
/// * `instruction_ptr` - Instruction pointer to validate
///
/// # Returns
/// * `true` - Instruction pointer is valid
/// * `false` - Instruction pointer is invalid
pub fn is_valid_instruction(instruction_ptr: ErtsCodePtr) -> bool {
    // In the C implementation:
    // VALID_INSTR(IP) checks if IP is within the valid instruction range
    // For now, we just check if it's not null
    !instruction_ptr.is_null()
}

/// Get next instruction pointer
///
/// Advances the instruction pointer to the next instruction.
/// Uses the instruction decoder to determine the actual instruction size.
///
/// # Arguments
/// * `instruction_ptr` - Current instruction pointer
///
/// # Returns
/// Next instruction pointer (or None if invalid)
pub fn next_instruction(instruction_ptr: ErtsCodePtr) -> Option<ErtsCodePtr> {
    if !is_valid_instruction(instruction_ptr) {
        return None;
    }
    
    // Use decoder to get actual instruction size
    use crate::instruction_decoder::get_instruction_size;
    let size = get_instruction_size(instruction_ptr);
    
    unsafe {
        Some(instruction_ptr.add(size / 8)) // Convert bytes to words (8 bytes per word)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_instruction() {
        assert!(!is_valid_instruction(std::ptr::null()));
        // Note: We can't easily test valid pointers without actual code
    }

    #[test]
    fn test_instruction_result_variants() {
        let _r1 = InstructionResult::Continue;
        let _r2 = InstructionResult::Yield;
        let _r3 = InstructionResult::NormalExit;
        let _r4 = InstructionResult::ErrorExit;
        let _r5 = InstructionResult::Trap(std::ptr::null());
        let _r6 = InstructionResult::ContextSwitch;
    }

    #[test]
    fn test_default_instruction_executor() {
        let executor = DefaultInstructionExecutor;
        let process = Process::new(1);
        let mut registers = vec![0u64; 10];
        let mut heap = vec![0u64; 100];
        
        let result = executor.execute_instruction(
            &process,
            std::ptr::null(),
            &mut registers,
            &mut heap,
        );
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), InstructionResult::Continue);
    }
}
