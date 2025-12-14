//! BEAM Instruction Decoder
//!
//! Decodes BEAM instructions from memory. BEAM instructions are variable-length
//! and consist of an opcode followed by operands.
//!
//! Based on beam_emu.c instruction decoding

use entities_process::ErtsCodePtr;

/// BEAM instruction opcodes (from genop.tab)
/// These are the generic opcodes used in BEAM files
pub mod opcodes {
    pub const LABEL: u8 = 1;
    pub const FUNC_INFO: u8 = 2;
    pub const INT_CODE_END: u8 = 3;
    pub const CALL: u8 = 4;
    pub const CALL_LAST: u8 = 5;
    pub const CALL_ONLY: u8 = 6;
    pub const CALL_EXT: u8 = 7;
    // ... more opcodes ...
    pub const MOVE: u8 = 64;
    pub const RETURN: u8 = 75; // Approximate - return is a specific instruction
}

/// Decoded BEAM instruction
#[derive(Debug, Clone)]
pub struct DecodedInstruction {
    /// Instruction opcode
    pub opcode: u8,
    /// Operands (as raw Eterm values)
    pub operands: Vec<u64>,
    /// Size of instruction in bytes
    pub size: usize,
}

/// Decode a BEAM instruction from memory
///
/// BEAM instructions are stored as:
/// - First word: Lower 32 bits = opcode, upper 32 bits = handler address (for specific instructions)
///   OR just the opcode byte for generic instructions
/// - Following words: Operands (tagged Eterm values)
///
/// For now, we'll implement a simplified decoder that reads:
/// - First byte: opcode
/// - Following bytes: operands (simplified - actual BEAM uses tagged values)
///
/// # Arguments
/// * `instruction_ptr` - Pointer to instruction in memory
///
/// # Returns
/// Decoded instruction or error
pub fn decode_instruction(instruction_ptr: ErtsCodePtr) -> Result<DecodedInstruction, String> {
    if instruction_ptr.is_null() {
        return Err("Null instruction pointer".to_string());
    }

    unsafe {
        // BEAM instructions in the code chunk are stored as 32-bit words (4 bytes each)
        // The code chunk is byte-aligned, so we read 32-bit words from the byte stream
        // The first word contains:
        //   - For generic instructions: opcode in lower 32 bits
        //   - For specific instructions: handler address in upper 32 bits, specific opcode in lower 32 bits
        // Following words contain operands (tagged Eterm values, but stored as 32-bit in code chunk)
        
        // Read first 32-bit word (big-endian from BEAM file)
        // BEAM files are big-endian, but we're on a little-endian system
        let bytes = std::slice::from_raw_parts(instruction_ptr, 4);
        let first_word = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        
        // Extract opcode from the instruction word
        // BEAM instructions can be:
        // 1. Generic instructions: opcode in lower 8 bits (0x00-0xFF)
        // 2. Specific instructions: handler address in upper bits, specific opcode in lower bits
        // 
        // For generic instructions, the opcode is in the lowest byte
        // For specific instructions, we need to check the upper bits
        // For now, we'll check if upper 24 bits are zero (generic) or non-zero (specific)
        let is_generic = (first_word & 0xFFFFFF00) == 0;
        let opcode = if is_generic {
            (first_word & 0xFF) as u8
        } else {
            // Specific instruction - opcode is still in lower 8 bits, but we need to look it up differently
            // For now, extract lower byte but note it's a specific instruction
            (first_word & 0xFF) as u8
        };
        
        // Debug: log unusual instructions (but not opcode 32 which is common)
        if (!is_generic || opcode > 100) && opcode != 32 {
            eprintln!("[Decoder] Decoding instruction at {:p}: word=0x{:08x}, generic={}, opcode={} (0x{:02x})", 
                     instruction_ptr, first_word, is_generic, opcode, opcode);
        }
        
        // For now, we'll use a simplified decoding
        // In the full implementation, we'd need to:
        // 1. Check if this is a generic or specific instruction
        // 2. Look up the instruction arity from opc[] table
        // 3. Decode operands based on their tags
        
        // For basic instructions, we'll assume:
        // - move: 2 operands (source, destination) = 3 words total = 12 bytes
        // - call: 2 operands (arity, label) = 3 words total = 12 bytes
        // - return: 0 operands = 1 word total = 4 bytes
        // Each word is 4 bytes in the code chunk
        
        let (arity, size_words) = match opcode {
            opcodes::MOVE => (2, 3), // opcode + 2 operands = 3 words
            opcodes::CALL => (2, 3),
            opcodes::CALL_LAST => (3, 4),
            opcodes::CALL_ONLY => (2, 3),
            opcodes::CALL_EXT => (2, 3),
            opcodes::RETURN => (0, 1),
            opcodes::LABEL => (1, 2),
            opcodes::FUNC_INFO => (3, 4),
            _ => {
                // Unknown instruction - try to decode as generic instruction
                // For safety, assume 1 word (4 bytes) and no operands
                // Many BEAM instructions are not yet implemented, so we'll skip them safely
                eprintln!("[Decoder] Unknown opcode: {} (0x{:02x}) at {:p} - assuming 1 word instruction", opcode, opcode, instruction_ptr);
                return Ok(DecodedInstruction {
                    opcode,
                    operands: Vec::new(),
                    size: 4, // 1 word = 4 bytes in code chunk
                });
            }
        };
        
        // Read operands (each operand is one 32-bit word = 4 bytes)
        // In real BEAM, operands are tagged Eterm values
        let mut operands = Vec::new();
        for i in 0..arity {
            let operand_offset = 4 + (i * 4); // Skip first word (opcode), then read each operand
            if operand_offset + 4 <= size_words * 4 {
                let operand_bytes = std::slice::from_raw_parts(
                    instruction_ptr.add(operand_offset),
                    4
                );
                let operand_word = u32::from_be_bytes([
                    operand_bytes[0], operand_bytes[1], operand_bytes[2], operand_bytes[3]
                ]);
                // Convert to u64 for compatibility (Eterm is 64-bit on 64-bit systems)
                operands.push(operand_word as u64);
            }
        }
        
        Ok(DecodedInstruction {
            opcode,
            operands,
            size: size_words * 4, // Convert words to bytes (4 bytes per word in code chunk)
        })
    }
}

/// Get instruction size in bytes
///
/// This is a helper to advance the instruction pointer.
/// In the full implementation, this would decode the instruction
/// to determine its actual size.
pub fn get_instruction_size(instruction_ptr: ErtsCodePtr) -> usize {
    match decode_instruction(instruction_ptr) {
        Ok(decoded) => decoded.size,
        Err(_) => 8, // Default to 8 bytes if decoding fails
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create a test instruction buffer
    fn create_instruction_buffer(opcode: u8, operands: &[u32]) -> Vec<u8> {
        let mut buffer = Vec::new();
        
        // First word: opcode in lower byte, rest zero (generic instruction)
        let first_word = opcode as u32;
        buffer.extend_from_slice(&first_word.to_be_bytes());
        
        // Add operand words (each 4 bytes, big-endian)
        for operand in operands {
            buffer.extend_from_slice(&operand.to_be_bytes());
        }
        
        buffer
    }

    #[test]
    fn test_decode_null_pointer() {
        let result = decode_instruction(std::ptr::null());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Null instruction pointer"));
    }

    #[test]
    fn test_decoded_instruction_debug() {
        let inst = DecodedInstruction {
            opcode: 64,
            operands: vec![1, 2],
            size: 12,
        };
        let debug_str = format!("{:?}", inst);
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_decoded_instruction_clone() {
        let inst1 = DecodedInstruction {
            opcode: 64,
            operands: vec![1, 2, 3],
            size: 16,
        };
        let inst2 = inst1.clone();
        assert_eq!(inst1.opcode, inst2.opcode);
        assert_eq!(inst1.operands, inst2.operands);
        assert_eq!(inst1.size, inst2.size);
    }

    #[test]
    fn test_decoded_instruction_fields() {
        let inst = DecodedInstruction {
            opcode: 75,
            operands: vec![42, 100],
            size: 12,
        };
        assert_eq!(inst.opcode, 75);
        assert_eq!(inst.operands.len(), 2);
        assert_eq!(inst.operands[0], 42);
        assert_eq!(inst.operands[1], 100);
        assert_eq!(inst.size, 12);
    }

    #[test]
    fn test_decode_move_instruction() {
        // MOVE instruction: opcode 64, 2 operands
        let buffer = create_instruction_buffer(opcodes::MOVE, &[1, 2]);
        let ptr = buffer.as_ptr() as ErtsCodePtr;
        
        let result = decode_instruction(ptr);
        assert!(result.is_ok());
        let decoded = result.unwrap();
        assert_eq!(decoded.opcode, opcodes::MOVE);
        assert_eq!(decoded.operands.len(), 2);
        assert_eq!(decoded.operands[0], 1);
        assert_eq!(decoded.operands[1], 2);
        assert_eq!(decoded.size, 12); // 3 words * 4 bytes = 12 bytes
    }

    #[test]
    fn test_decode_return_instruction() {
        // RETURN instruction: opcode 75, 0 operands
        let buffer = create_instruction_buffer(opcodes::RETURN, &[]);
        let ptr = buffer.as_ptr() as ErtsCodePtr;
        
        let result = decode_instruction(ptr);
        assert!(result.is_ok());
        let decoded = result.unwrap();
        assert_eq!(decoded.opcode, opcodes::RETURN);
        assert_eq!(decoded.operands.len(), 0);
        assert_eq!(decoded.size, 4); // 1 word * 4 bytes = 4 bytes
    }

    #[test]
    fn test_decode_call_instruction() {
        // CALL instruction: opcode 4, 2 operands
        let buffer = create_instruction_buffer(opcodes::CALL, &[3, 100]);
        let ptr = buffer.as_ptr() as ErtsCodePtr;
        
        let result = decode_instruction(ptr);
        assert!(result.is_ok());
        let decoded = result.unwrap();
        assert_eq!(decoded.opcode, opcodes::CALL);
        assert_eq!(decoded.operands.len(), 2);
        assert_eq!(decoded.size, 12); // 3 words * 4 bytes = 12 bytes
    }

    #[test]
    fn test_decode_call_last_instruction() {
        // CALL_LAST instruction: opcode 5, 3 operands
        let buffer = create_instruction_buffer(opcodes::CALL_LAST, &[2, 50, 100]);
        let ptr = buffer.as_ptr() as ErtsCodePtr;
        
        let result = decode_instruction(ptr);
        assert!(result.is_ok());
        let decoded = result.unwrap();
        assert_eq!(decoded.opcode, opcodes::CALL_LAST);
        assert_eq!(decoded.operands.len(), 3);
        assert_eq!(decoded.size, 16); // 4 words * 4 bytes = 16 bytes
    }

    #[test]
    fn test_decode_label_instruction() {
        // LABEL instruction: opcode 1, 1 operand
        let buffer = create_instruction_buffer(opcodes::LABEL, &[42]);
        let ptr = buffer.as_ptr() as ErtsCodePtr;
        
        let result = decode_instruction(ptr);
        assert!(result.is_ok());
        let decoded = result.unwrap();
        assert_eq!(decoded.opcode, opcodes::LABEL);
        assert_eq!(decoded.operands.len(), 1);
        assert_eq!(decoded.operands[0], 42);
        assert_eq!(decoded.size, 8); // 2 words * 4 bytes = 8 bytes
    }

    #[test]
    fn test_decode_func_info_instruction() {
        // FUNC_INFO instruction: opcode 2, 3 operands
        let buffer = create_instruction_buffer(opcodes::FUNC_INFO, &[10, 20, 30]);
        let ptr = buffer.as_ptr() as ErtsCodePtr;
        
        let result = decode_instruction(ptr);
        assert!(result.is_ok());
        let decoded = result.unwrap();
        assert_eq!(decoded.opcode, opcodes::FUNC_INFO);
        assert_eq!(decoded.operands.len(), 3);
        assert_eq!(decoded.size, 16); // 4 words * 4 bytes = 16 bytes
    }

    #[test]
    fn test_decode_unknown_opcode() {
        // Unknown opcode should return a valid DecodedInstruction with size 4
        let buffer = create_instruction_buffer(200, &[]);
        let ptr = buffer.as_ptr() as ErtsCodePtr;
        
        let result = decode_instruction(ptr);
        assert!(result.is_ok());
        let decoded = result.unwrap();
        assert_eq!(decoded.opcode, 200);
        assert_eq!(decoded.operands.len(), 0);
        assert_eq!(decoded.size, 4); // Default: 1 word = 4 bytes
    }

    #[test]
    fn test_decode_instruction_with_large_operands() {
        // Test with large operand values
        let buffer = create_instruction_buffer(opcodes::MOVE, &[0xFFFFFFFF, 0x12345678]);
        let ptr = buffer.as_ptr() as ErtsCodePtr;
        
        let result = decode_instruction(ptr);
        assert!(result.is_ok());
        let decoded = result.unwrap();
        assert_eq!(decoded.operands[0], 0xFFFFFFFF);
        assert_eq!(decoded.operands[1], 0x12345678);
    }

    #[test]
    fn test_decode_instruction_generic_vs_specific() {
        // Test generic instruction (upper 24 bits are zero)
        let buffer = create_instruction_buffer(64, &[1, 2]);
        let ptr = buffer.as_ptr() as ErtsCodePtr;
        
        let result = decode_instruction(ptr);
        assert!(result.is_ok());
        let decoded = result.unwrap();
        assert_eq!(decoded.opcode, 64);
    }

    #[test]
    fn test_get_instruction_size() {
        // Test with valid instruction
        let buffer = create_instruction_buffer(opcodes::MOVE, &[1, 2]);
        let ptr = buffer.as_ptr() as ErtsCodePtr;
        
        let size = get_instruction_size(ptr);
        assert_eq!(size, 12); // MOVE is 3 words = 12 bytes
    }

    #[test]
    fn test_get_instruction_size_return() {
        // Test with RETURN instruction (1 word)
        let buffer = create_instruction_buffer(opcodes::RETURN, &[]);
        let ptr = buffer.as_ptr() as ErtsCodePtr;
        
        let size = get_instruction_size(ptr);
        assert_eq!(size, 4); // RETURN is 1 word = 4 bytes
    }

    #[test]
    fn test_get_instruction_size_null_pointer() {
        // Test with null pointer (should return default)
        let size = get_instruction_size(std::ptr::null());
        assert_eq!(size, 8); // Default size when decoding fails
    }

    #[test]
    fn test_get_instruction_size_unknown_opcode() {
        // Test with unknown opcode
        let buffer = create_instruction_buffer(200, &[]);
        let ptr = buffer.as_ptr() as ErtsCodePtr;
        
        let size = get_instruction_size(ptr);
        assert_eq!(size, 4); // Unknown opcode defaults to 4 bytes
    }

    #[test]
    fn test_opcodes_constants() {
        // Test that opcode constants are defined
        assert_eq!(opcodes::LABEL, 1);
        assert_eq!(opcodes::FUNC_INFO, 2);
        assert_eq!(opcodes::INT_CODE_END, 3);
        assert_eq!(opcodes::CALL, 4);
        assert_eq!(opcodes::CALL_LAST, 5);
        assert_eq!(opcodes::CALL_ONLY, 6);
        assert_eq!(opcodes::CALL_EXT, 7);
        assert_eq!(opcodes::MOVE, 64);
        assert_eq!(opcodes::RETURN, 75);
    }

    #[test]
    fn test_decode_instruction_big_endian() {
        // Test that operands are read as big-endian
        // Create instruction with operand 0x12345678
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&(64u32).to_be_bytes()); // MOVE opcode
        buffer.extend_from_slice(&0x12345678u32.to_be_bytes()); // First operand
        buffer.extend_from_slice(&0xABCDEF01u32.to_be_bytes()); // Second operand
        
        let ptr = buffer.as_ptr() as ErtsCodePtr;
        let result = decode_instruction(ptr);
        assert!(result.is_ok());
        let decoded = result.unwrap();
        assert_eq!(decoded.operands[0], 0x12345678);
        assert_eq!(decoded.operands[1], 0xABCDEF01);
    }

    #[test]
    fn test_decode_instruction_zero_operands() {
        // Test instruction with zero operands
        let buffer = create_instruction_buffer(opcodes::RETURN, &[]);
        let ptr = buffer.as_ptr() as ErtsCodePtr;
        
        let result = decode_instruction(ptr);
        assert!(result.is_ok());
        let decoded = result.unwrap();
        assert_eq!(decoded.operands.len(), 0);
    }

    #[test]
    fn test_decode_instruction_multiple_operands() {
        // Test instruction with multiple operands
        let buffer = create_instruction_buffer(opcodes::CALL_LAST, &[1, 2, 3]);
        let ptr = buffer.as_ptr() as ErtsCodePtr;
        
        let result = decode_instruction(ptr);
        assert!(result.is_ok());
        let decoded = result.unwrap();
        assert_eq!(decoded.operands.len(), 3);
        assert_eq!(decoded.operands[0], 1);
        assert_eq!(decoded.operands[1], 2);
        assert_eq!(decoded.operands[2], 3);
    }

    #[test]
    fn test_decode_instruction_all_call_variants() {
        // Test all CALL instruction variants
        let call_variants = vec![
            (opcodes::CALL, 2, 12),
            (opcodes::CALL_LAST, 3, 16),
            (opcodes::CALL_ONLY, 2, 12),
            (opcodes::CALL_EXT, 2, 12),
        ];
        
        for (opcode, expected_arity, expected_size) in call_variants {
            let operands: Vec<u32> = (0..expected_arity).map(|i| i as u32).collect();
            let buffer = create_instruction_buffer(opcode, &operands);
            let ptr = buffer.as_ptr() as ErtsCodePtr;
            
            let result = decode_instruction(ptr);
            assert!(result.is_ok(), "Failed to decode opcode {}", opcode);
            let decoded = result.unwrap();
            assert_eq!(decoded.opcode, opcode);
            assert_eq!(decoded.operands.len(), expected_arity);
            assert_eq!(decoded.size, expected_size);
        }
    }

    #[test]
    fn test_decoded_instruction_empty_operands() {
        let inst = DecodedInstruction {
            opcode: 75,
            operands: vec![],
            size: 4,
        };
        assert_eq!(inst.operands.len(), 0);
        assert_eq!(inst.size, 4);
    }

    #[test]
    fn test_decoded_instruction_large_size() {
        let inst = DecodedInstruction {
            opcode: 64,
            operands: vec![1, 2, 3, 4, 5],
            size: 100,
        };
        assert_eq!(inst.size, 100);
        assert_eq!(inst.operands.len(), 5);
    }

    #[test]
    fn test_decode_instruction_opcode_zero() {
        // Test opcode 0
        let buffer = create_instruction_buffer(0, &[]);
        let ptr = buffer.as_ptr() as ErtsCodePtr;
        
        let result = decode_instruction(ptr);
        assert!(result.is_ok());
        let decoded = result.unwrap();
        assert_eq!(decoded.opcode, 0);
    }

    #[test]
    fn test_decode_instruction_opcode_max() {
        // Test opcode 255
        let buffer = create_instruction_buffer(255, &[]);
        let ptr = buffer.as_ptr() as ErtsCodePtr;
        
        let result = decode_instruction(ptr);
        assert!(result.is_ok());
        let decoded = result.unwrap();
        assert_eq!(decoded.opcode, 255);
    }

    #[test]
    fn test_get_instruction_size_all_opcodes() {
        // Test size calculation for all known opcodes
        let test_cases = vec![
            (opcodes::MOVE, 12),
            (opcodes::RETURN, 4),
            (opcodes::CALL, 12),
            (opcodes::CALL_LAST, 16),
            (opcodes::LABEL, 8),
            (opcodes::FUNC_INFO, 16),
        ];
        
        for (opcode, expected_size) in test_cases {
            let arity = match opcode {
                opcodes::MOVE => 2,
                opcodes::RETURN => 0,
                opcodes::CALL => 2,
                opcodes::CALL_LAST => 3,
                opcodes::LABEL => 1,
                opcodes::FUNC_INFO => 3,
                _ => 0,
            };
            let operands: Vec<u32> = (0..arity).map(|i| i as u32).collect();
            let buffer = create_instruction_buffer(opcode, &operands);
            let ptr = buffer.as_ptr() as ErtsCodePtr;
            
            let size = get_instruction_size(ptr);
            assert_eq!(size, expected_size, "Failed for opcode {}", opcode);
        }
    }

    #[test]
    fn test_decode_instruction_operand_conversion() {
        // Test that 32-bit operands are converted to u64 correctly
        let buffer = create_instruction_buffer(opcodes::MOVE, &[0xFFFFFFFF, 0x12345678]);
        let ptr = buffer.as_ptr() as ErtsCodePtr;
        
        let result = decode_instruction(ptr);
        assert!(result.is_ok());
        let decoded = result.unwrap();
        // Operands should be u64 values
        assert_eq!(decoded.operands[0], 0xFFFFFFFFu64);
        assert_eq!(decoded.operands[1], 0x12345678u64);
    }

    #[test]
    fn test_decode_instruction_error_message() {
        let result = decode_instruction(std::ptr::null());
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(!error.is_empty());
        assert!(error.contains("Null") || error.contains("null") || error.contains("pointer"));
    }
}

