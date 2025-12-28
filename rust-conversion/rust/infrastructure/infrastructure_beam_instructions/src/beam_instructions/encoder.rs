//! BEAM Instruction Encoder
//!
//! Encodes structured BEAM instructions back into bytecode.
//! This is the reverse of the parser - converts BeamInstruction to raw bytes.

use super::types::*;
use super::opcodes::BeamOpcode;
use std::io::{Cursor, Write};

/// Errors that can occur during BEAM encoding
#[derive(Debug, thiserror::Error)]
pub enum BeamEncodeError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Unsupported argument type: {0:?}")]
    UnsupportedArgType(BeamArg),
    #[error("Value too large for encoding: {0}")]
    ValueTooLarge(u64),
}

/// BEAM instruction encoder
pub struct BeamEncoder;

impl BeamEncoder {
    /// Encode a single BEAM instruction to bytes
    pub fn encode_instruction(instruction: &BeamInstruction) -> Result<Vec<u8>, BeamEncodeError> {
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);

        // Write opcode (1 byte in loaded BEAM code)
        Self::write_u8(&mut cursor, instruction.opcode as u8)?;

        // Encode each argument
        for arg in &instruction.args {
            Self::encode_arg(&mut cursor, arg)?;
        }

        Ok(buffer)
    }

    /// Encode a BEAM argument using proper BEAM format
    fn encode_arg(cursor: &mut Cursor<&mut Vec<u8>>, arg: &BeamArg) -> Result<(), BeamEncodeError> {
        match arg {
            BeamArg::Register { index, is_y } => {
                // BEAM register encoding: Y registers use different tag
                if *is_y {
                    // Y register (stack)
                    if *index <= 255 {
                        Self::write_u8(cursor, 0xE0 | (*index as u8 & 0x1F))?;
                        if *index >= 32 {
                            // Extended Y register encoding
                            Self::write_u8(cursor, *index as u8)?;
                        }
                    } else {
                        return Err(BeamEncodeError::UnsupportedArgType(arg.clone()));
                    }
                } else {
                    // X register (argument/result)
                    if *index <= 255 {
                        Self::write_u8(cursor, 0xC0 | (*index as u8 & 0x1F))?;
                        if *index >= 32 {
                            // Extended X register encoding
                            Self::write_u8(cursor, *index as u8)?;
                        }
                    } else {
                    return Err(BeamEncodeError::UnsupportedArgType(arg.clone()));
                    }
                }
            }

            BeamArg::Literal(value) => {
                // BEAM literal encoding - small integers are direct
                if *value >= 0 && *value <= 255 {
                    // Small integer literal
                    Self::write_u8(cursor, *value as u8)?;
                } else if (*value as i64) >= -128 && (*value as i64) <= 127 {
                    // Signed small integer - convert to i64 first for negative check
                    Self::write_u8(cursor, *value as u8)?;
                } else {
                    // Extended integer encoding
                    Self::write_u8(cursor, 0x80)?; // Integer tag
                    Self::write_u32_be(cursor, *value as u32)?;
                }
            }

            BeamArg::Label(index) => {
                // Labels in BEAM are encoded as small literals
                Self::encode_arg(cursor, &BeamArg::Literal(*index as u64))?;
            }

            BeamArg::List(args) => {
                // List arguments are encoded sequentially
                for arg in args {
                    Self::encode_arg(cursor, arg)?;
                }
            }
            BeamArg::Extended(_) => {
                // Extended arguments not implemented
                return Err(BeamEncodeError::UnsupportedArgType(arg.clone()));
            }
        }

        Ok(())
    }

    /// Encode a complete BEAM code chunk
    pub fn encode_code(code: &BeamCode) -> Result<Vec<u8>, BeamEncodeError> {
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);

        // Write header
        Self::write_u32_be(&mut cursor, code.header.sub_size)?;
        Self::write_u32_be(&mut cursor, code.header.instruction_set)?;
        Self::write_u32_be(&mut cursor, code.header.max_opcode)?;
        Self::write_u32_be(&mut cursor, code.header.label_count)?;
        Self::write_u32_be(&mut cursor, code.header.function_count)?;

        // Encode all instructions from all functions
        for function in &code.functions {
            for instruction in &function.instructions {
                let instr_bytes = Self::encode_instruction(instruction)?;
                cursor.write_all(&instr_bytes)?;
            }
        }

        Ok(buffer)
    }

    // Helper methods for writing binary data
    fn write_u8(cursor: &mut Cursor<&mut Vec<u8>>, value: u8) -> Result<(), BeamEncodeError> {
        cursor.write_all(&[value])?;
        Ok(())
    }

    fn write_u32_be(cursor: &mut Cursor<&mut Vec<u8>>, value: u32) -> Result<(), BeamEncodeError> {
        cursor.write_all(&value.to_be_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_move_instruction() {
        // Test encoding: move {x, 0}, {x, 1}
        let instruction = BeamInstruction::new(
            BeamOpcode::Move.to_c_opcode(),
            vec![
                BeamArg::Register { index: 0, is_y: false },
                BeamArg::Register { index: 1, is_y: false },
            ],
        );

        let result = BeamEncoder::encode_instruction(&instruction);
        assert!(result.is_ok());

        let bytes = result.unwrap();
        // Opcode (138) + X0 register (0xC0) + X1 register (0xC1)
        assert_eq!(bytes, vec![0x8A, 0xC0, 0xC1]);
    }

    #[test]
    fn test_encode_literal_small() {
        // Test encoding small literal (≤ 127)
        let instruction = BeamInstruction::new(
            BeamOpcode::Move as u32,
            vec![
                BeamArg::Literal(42),
                BeamArg::Register { index: 0, is_y: false },
            ],
        );

        let result = BeamEncoder::encode_instruction(&instruction);
        assert!(result.is_ok());

        let bytes = result.unwrap();
        // Opcode (64) + literal 42 (direct) + X0 register (0xC0)
        assert_eq!(bytes, vec![0x40, 42, 0xC0]);
    }

    #[test]
    fn test_encode_literal_extended() {
        // Test encoding extended literal (> 127)
        let instruction = BeamInstruction::new(
            BeamOpcode::Move as u32,
            vec![
                BeamArg::Literal(1000),
                BeamArg::Register { index: 0, is_y: false },
            ],
        );

        let result = BeamEncoder::encode_instruction(&instruction);
        assert!(result.is_ok());

        let bytes = result.unwrap();
        // Opcode (64) + extended literal tag (0x80) + value (0x000003E8) + X0 register (0xC0)
        assert_eq!(bytes, vec![0x40, 0x80, 0x00, 0x00, 0x03, 0xE8, 0xC0]);
    }
}
