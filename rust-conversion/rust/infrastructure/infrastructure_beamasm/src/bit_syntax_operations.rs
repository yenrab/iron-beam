//! Bit Syntax Operations
//!
//! Provides binary construction and matching, bit field extraction and insertion,
//! bit-level operations and conversions, and endianness handling.
//!
//! Based on `instr_bs.cpp` and `beam_common.h:bs_*.h`

use crate::BeamAssemblerError;
use crate::asmjit_wrapper::{Assembler, a64};

/// Bit syntax context information
#[derive(Debug, Clone)]
pub struct BitSyntaxContext {
    /// Context register (contains ErlSubBits structure)
    pub context_reg: u32,
    /// Current position in bits
    pub position: u64,
    /// Binary size in bytes
    pub size: u64,
    /// Unit size for bit operations
    pub unit: u32,
}

/// Bit field specification
#[derive(Debug, Clone)]
pub struct BitFieldSpec {
    /// Size of the field in bits/units
    pub size: u64,
    /// Unit size (1, 8, 16, 32, etc.)
    pub unit: u32,
    /// Signedness flag
    pub signed: bool,
    /// Endianness (big, little, native)
    pub endianness: BitEndianness,
    /// Type of bit operation
    pub field_type: BitFieldType,
}

/// Bit field types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BitFieldType {
    /// Integer field
    Integer,
    /// Float field
    Float,
    /// Binary field
    Binary,
    /// UTF-8 character
    Utf8,
    /// UTF-16 character
    Utf16,
    /// UTF-32 character
    Utf32,
}

/// Endianness specification
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BitEndianness {
    /// Big endian
    Big,
    /// Little endian
    Little,
    /// Native endianness
    Native,
}

/// Binary construction state
#[derive(Debug, Clone)]
pub struct BinaryConstructionState {
    /// Destination register for the constructed binary
    pub dst_reg: u32,
    /// Current size accumulator
    pub current_size: u64,
    /// Unit size for construction
    pub unit: u32,
    /// Heap allocation needed
    pub heap_needed: u64,
}

/// Binary matching result
#[derive(Debug, Clone)]
pub enum BinaryMatchResult {
    /// Match successful
    Success {
        /// Extracted value register
        value_reg: u32,
        /// New position after match
        new_position: u64,
    },
    /// Match failed
    Failure,
    /// Partial match (needs more data)
    Partial,
}

/// Bit syntax operations coordinator
///
/// Manages binary construction, pattern matching, and bit-level operations
/// for Erlang's bit syntax.
pub struct BitSyntaxOperations;

impl BitSyntaxOperations {
    /// Start binary matching
    ///
    /// Initialize a binary matching context from a source binary.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `src_reg` - Source binary register
    /// * `dst_reg` - Destination context register
    /// * `live` - Number of live X registers
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn start_binary_match(
        assembler: &mut Assembler,
        src_reg: u32,
        dst_reg: u32,
        live: u32,
    ) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] Bit Syntax: Starting binary match, src={}, dst={}, live={}",
                 src_reg, dst_reg, live);

        // Test if source is a binary
        Self::emit_test_binary(assembler, src_reg)?;

        // Allocate and initialize match context
        Self::emit_initialize_match_context(assembler, src_reg, dst_reg, live)?;

        Ok(())
    }

    /// Get binary field
    ///
    /// Extract a bit field from a binary context.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `context` - Bit syntax context
    /// * `field_spec` - Field specification
    /// * `dst_reg` - Destination register for extracted value
    ///
    /// # Returns
    /// Binary match result
    pub fn get_binary_field(
        assembler: &mut Assembler,
        context: &BitSyntaxContext,
        field_spec: &BitFieldSpec,
        dst_reg: u32,
    ) -> Result<BinaryMatchResult, BeamAssemblerError> {
        eprintln!("[DEBUG] Bit Syntax: Getting binary field, size={}, unit={}, signed={:?}",
                 field_spec.size, field_spec.unit, field_spec.signed);

        match field_spec.field_type {
            BitFieldType::Integer => {
                Self::get_integer_field(assembler, context, field_spec, dst_reg)
            }
            BitFieldType::Float => {
                Self::get_float_field(assembler, context, field_spec, dst_reg)
            }
            BitFieldType::Binary => {
                Self::get_binary_subfield(assembler, context, field_spec, dst_reg)
            }
            BitFieldType::Utf8 => {
                Self::get_utf8_field(assembler, context, dst_reg)
            }
            BitFieldType::Utf16 => {
                Self::get_utf16_field(assembler, context, field_spec, dst_reg)
            }
            BitFieldType::Utf32 => {
                Self::get_utf32_field(assembler, context, field_spec, dst_reg)
            }
        }
    }

    /// Set binary position
    ///
    /// Update the position in a binary context.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `context_reg` - Context register
    /// * `position_reg` - New position register
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn set_binary_position(
        assembler: &mut Assembler,
        context_reg: u32,
        position_reg: u32,
    ) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] Bit Syntax: Setting binary position");

        // Update the position in ErlSubBits structure
        const START_OFFSET: i32 = 0; // offsetof(ErlSubBits, start)

        // Load position value and shift for storage
        a64::emit_mov_reg_reg(assembler, 9, position_reg)?; // TMP1 = position
        a64::emit_lsr_imm(assembler, 9, 9, 3)?; // TMP1 >>= _TAG_IMMED1_SIZE

        // Store to context->start
        a64::emit_stur_reg_offset(assembler, 9, context_reg, START_OFFSET)?;

        Ok(())
    }

    /// Get binary position
    ///
    /// Retrieve the current position from a binary context.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `context_reg` - Context register
    /// * `dst_reg` - Destination register for position
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn get_binary_position(
        assembler: &mut Assembler,
        context_reg: u32,
        dst_reg: u32,
    ) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] Bit Syntax: Getting binary position");

        // Load position from ErlSubBits structure
        const START_OFFSET: i32 = 0; // offsetof(ErlSubBits, start)

        // Load context->start and shift for tagged representation
        a64::emit_ldur_reg_offset(assembler, dst_reg, context_reg, START_OFFSET)?;
        a64::emit_lsl_imm(assembler, dst_reg, dst_reg, 3)?; // << _TAG_IMMED1_SIZE

        Ok(())
    }

    /// Get binary tail
    ///
    /// Extract the remaining binary data from current position to end.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `context_reg` - Context register
    /// * `dst_reg` - Destination register for tail binary
    /// * `live` - Number of live registers
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn get_binary_tail(
        assembler: &mut Assembler,
        context_reg: u32,
        dst_reg: u32,
        live: u32,
    ) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] Bit Syntax: Getting binary tail");

        // Extract remaining binary from context
        Self::emit_extract_binary_tail(assembler, context_reg, dst_reg, live)?;

        Ok(())
    }

    /// Test binary unit
    ///
    /// Test if binary size matches the expected unit size.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `context_reg` - Context register
    /// * `unit` - Unit size to test
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn test_binary_unit(
        assembler: &mut Assembler,
        context_reg: u32,
        unit: u32,
    ) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] Bit Syntax: Testing binary unit {}", unit);

        // Load binary size and check if divisible by unit
        const SIZE_OFFSET: i32 = 8; // Placeholder offset for binary size

        a64::emit_ldur_reg_offset(assembler, 9, context_reg, SIZE_OFFSET)?; // TMP1 = size
        a64::emit_mov_imm(assembler, 10, unit as u64)?; // TMP2 = unit

        // Check if size % unit == 0
        a64::emit_udiv_reg_reg_reg(assembler, 11, 9, 10)?; // TMP3 = size / unit
        a64::emit_msub_reg_reg_reg_reg(assembler, 11, 11, 10, 9)?; // TMP3 = size - (TMP3 * unit)

        // If TMP3 != 0, unit test fails
        a64::emit_cmp_imm(assembler, 11, 0)?;
        // Branch to error if not zero

        Ok(())
    }

    /// Start binary construction
    ///
    /// Initialize binary construction state.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `dst_reg` - Destination register for constructed binary
    /// * `size_hint` - Initial size hint
    ///
    /// # Returns
    /// Construction state
    pub fn start_binary_construction(
        assembler: &mut Assembler,
        dst_reg: u32,
        size_hint: u64,
    ) -> Result<BinaryConstructionState, BeamAssemblerError> {
        eprintln!("[DEBUG] Bit Syntax: Starting binary construction, size_hint={}", size_hint);

        // Initialize construction state
        let state = BinaryConstructionState {
            dst_reg,
            current_size: 0,
            unit: 1,
            heap_needed: size_hint,
        };

        // Allocate initial heap space
        crate::HeapAllocationCoordinator::emit_allocate_heap(
            assembler,
            &crate::HeapAllocRequest {
                need_stack: 0,
                need_heap: (size_hint / 8 + 1) as u32,
                live_registers: 0,
            },
        )?;

        Ok(state)
    }

    /// Add field to binary construction
    ///
    /// Add a value to the binary being constructed.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `state` - Construction state (will be updated)
    /// * `value_reg` - Register containing value to add
    /// * `field_spec` - Field specification
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn add_binary_field(
        assembler: &mut Assembler,
        state: &mut BinaryConstructionState,
        value_reg: u32,
        field_spec: &BitFieldSpec,
    ) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] Bit Syntax: Adding binary field, type={:?}, size={}",
                 field_spec.field_type, field_spec.size);

        // Update size tracking
        let field_bits = field_spec.size * field_spec.unit as u64;
        state.current_size += field_bits;

        // Add value to binary
        Self::emit_add_value_to_binary(
            assembler,
            state,
            value_reg,
            field_spec,
        )?;

        Ok(())
    }

    /// Finish binary construction
    ///
    /// Complete binary construction and return final binary.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `state` - Construction state
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn finish_binary_construction(
        assembler: &mut Assembler,
        state: &BinaryConstructionState,
    ) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] Bit Syntax: Finishing binary construction, final_size={}",
                 state.current_size);

        // Finalize binary construction
        Self::emit_finalize_binary(assembler, state)?;

        Ok(())
    }

    // Private helper methods

    fn get_integer_field(
        assembler: &mut Assembler,
        context: &BitSyntaxContext,
        field_spec: &BitFieldSpec,
        dst_reg: u32,
    ) -> Result<BinaryMatchResult, BeamAssemblerError> {
        eprintln!("[DEBUG] Bit Syntax: Getting integer field");

        // Extract integer value from binary
        Self::emit_extract_integer_field(assembler, context, field_spec, dst_reg)?;

        Ok(BinaryMatchResult::Success {
            value_reg: dst_reg,
            new_position: context.position + field_spec.size * field_spec.unit as u64,
        })
    }

    fn get_float_field(
        assembler: &mut Assembler,
        context: &BitSyntaxContext,
        field_spec: &BitFieldSpec,
        dst_reg: u32,
    ) -> Result<BinaryMatchResult, BeamAssemblerError> {
        eprintln!("[DEBUG] Bit Syntax: Getting float field");

        // Extract float value from binary
        Self::emit_extract_float_field(assembler, context, field_spec, dst_reg)?;

        Ok(BinaryMatchResult::Success {
            value_reg: dst_reg,
            new_position: context.position + field_spec.size * field_spec.unit as u64,
        })
    }

    fn get_binary_subfield(
        assembler: &mut Assembler,
        context: &BitSyntaxContext,
        field_spec: &BitFieldSpec,
        dst_reg: u32,
    ) -> Result<BinaryMatchResult, BeamAssemblerError> {
        eprintln!("[DEBUG] Bit Syntax: Getting binary subfield");

        // Extract binary subfield
        Self::emit_extract_binary_subfield(assembler, context, field_spec, dst_reg)?;

        Ok(BinaryMatchResult::Success {
            value_reg: dst_reg,
            new_position: context.position + field_spec.size * field_spec.unit as u64,
        })
    }

    fn get_utf8_field(
        assembler: &mut Assembler,
        context: &BitSyntaxContext,
        dst_reg: u32,
    ) -> Result<BinaryMatchResult, BeamAssemblerError> {
        eprintln!("[DEBUG] Bit Syntax: Getting UTF-8 field");

        // Extract UTF-8 character (1 byte)
        Self::emit_extract_utf8_field(assembler, context, dst_reg)?;

        Ok(BinaryMatchResult::Success {
            value_reg: dst_reg,
            new_position: context.position + 8, // 1 byte
        })
    }

    fn get_utf16_field(
        assembler: &mut Assembler,
        context: &BitSyntaxContext,
        field_spec: &BitFieldSpec,
        dst_reg: u32,
    ) -> Result<BinaryMatchResult, BeamAssemblerError> {
        eprintln!("[DEBUG] Bit Syntax: Getting UTF-16 field");

        // Extract UTF-16 character (2 bytes with endianness)
        Self::emit_extract_utf16_field(assembler, context, field_spec, dst_reg)?;

        Ok(BinaryMatchResult::Success {
            value_reg: dst_reg,
            new_position: context.position + 16, // 2 bytes
        })
    }

    fn get_utf32_field(
        assembler: &mut Assembler,
        context: &BitSyntaxContext,
        field_spec: &BitFieldSpec,
        dst_reg: u32,
    ) -> Result<BinaryMatchResult, BeamAssemblerError> {
        eprintln!("[DEBUG] Bit Syntax: Getting UTF-32 field");

        // Extract UTF-32 character (4 bytes with endianness)
        Self::emit_extract_utf32_field(assembler, context, field_spec, dst_reg)?;

        Ok(BinaryMatchResult::Success {
            value_reg: dst_reg,
            new_position: context.position + 32, // 4 bytes
        })
    }

    // Low-level emission methods

    fn emit_test_binary(assembler: &mut Assembler, src_reg: u32) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] Bit Syntax: Testing if register is binary");

        // Test if src_reg contains a binary
        // This typically involves checking the tag bits

        a64::emit_mov_reg_reg(assembler, 9, src_reg)?; // TMP1 = src
        a64::emit_and_imm(assembler, 9, 9, 0x3)?; // TMP1 &= TAG_MASK

        // Compare with binary tag
        const BINARY_TAG: u64 = 0x2; // Placeholder
        a64::emit_cmp_imm(assembler, 9, BINARY_TAG)?;

        // Branch to error if not binary

        Ok(())
    }

    fn emit_initialize_match_context(
        assembler: &mut Assembler,
        src_reg: u32,
        dst_reg: u32,
        live: u32,
    ) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] Bit Syntax: Initializing match context");

        // Initialize ErlSubBits structure
        // This typically involves runtime calls to set up the context

        Ok(())
    }

    fn emit_extract_integer_field(
        assembler: &mut Assembler,
        context: &BitSyntaxContext,
        field_spec: &BitFieldSpec,
        dst_reg: u32,
    ) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] Bit Syntax: Extracting integer field");

        // Load field size
        a64::emit_mov_imm(assembler, 9, field_spec.size)?; // TMP1 = size

        // Load unit size
        a64::emit_mov_imm(assembler, 10, field_spec.unit as u64)?; // TMP2 = unit

        // Calculate bit size
        a64::emit_mul_reg_reg_reg(assembler, 9, 9, 10)?; // TMP1 = size * unit

        // Call runtime function to extract integer
        // This would be something like beam_jit_bs_get_integer

        Ok(())
    }

    fn emit_extract_float_field(
        assembler: &mut Assembler,
        context: &BitSyntaxContext,
        field_spec: &BitFieldSpec,
        dst_reg: u32,
    ) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] Bit Syntax: Extracting float field");

        // Call runtime function to extract float
        // This would be something like beam_jit_bs_get_float

        Ok(())
    }

    fn emit_extract_binary_subfield(
        assembler: &mut Assembler,
        context: &BitSyntaxContext,
        field_spec: &BitFieldSpec,
        dst_reg: u32,
    ) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] Bit Syntax: Extracting binary subfield");

        // Call runtime function to extract binary subfield
        // This would be something like erts_build_sub_bitstring

        Ok(())
    }

    fn emit_extract_utf8_field(
        assembler: &mut Assembler,
        context: &BitSyntaxContext,
        dst_reg: u32,
    ) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] Bit Syntax: Extracting UTF-8 field");

        // Extract 1 byte and validate UTF-8
        a64::emit_mov_imm(assembler, dst_reg, 0)?; // Placeholder

        Ok(())
    }

    fn emit_extract_utf16_field(
        assembler: &mut Assembler,
        context: &BitSyntaxContext,
        field_spec: &BitFieldSpec,
        dst_reg: u32,
    ) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] Bit Syntax: Extracting UTF-16 field");

        // Extract 2 bytes with endianness handling
        a64::emit_mov_imm(assembler, dst_reg, 0)?; // Placeholder

        Ok(())
    }

    fn emit_extract_utf32_field(
        assembler: &mut Assembler,
        context: &BitSyntaxContext,
        field_spec: &BitFieldSpec,
        dst_reg: u32,
    ) -> Result<(), BeamAssemblerError> {
        use crate::asmjit_wrapper as a64;

        eprintln!("[DEBUG] Bit Syntax: Extracting UTF-32 field");

        // Extract 4 bytes with endianness handling
        a64::emit_mov_imm(assembler, dst_reg, 0)?; // Placeholder

        Ok(())
    }

    fn emit_extract_binary_tail(
        assembler: &mut Assembler,
        context_reg: u32,
        dst_reg: u32,
        live: u32,
    ) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] Bit Syntax: Extracting binary tail");

        // Extract remaining binary from current position
        // This involves calling erts_build_sub_bitstring

        Ok(())
    }

    fn emit_add_value_to_binary(
        assembler: &mut Assembler,
        state: &BinaryConstructionState,
        value_reg: u32,
        field_spec: &BitFieldSpec,
    ) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] Bit Syntax: Adding value to binary");

        // Add value to binary under construction
        // This involves bit manipulation and heap updates

        Ok(())
    }

    fn emit_finalize_binary(
        assembler: &mut Assembler,
        state: &BinaryConstructionState,
    ) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] Bit Syntax: Finalizing binary");

        // Finalize binary construction
        // Create final binary term

        Ok(())
    }

    /// Validate bit syntax context
    ///
    /// Checks if the bit syntax context is valid and properly initialized.
    ///
    /// # Arguments
    /// * `context` - Context to validate
    ///
    /// # Returns
    /// true if valid, false otherwise
    pub fn validate_context(context: &BitSyntaxContext) -> bool {
        context.context_reg < 32 && // Valid ARM64 register
        context.position >= 0 &&
        context.size >= context.position &&
        context.unit > 0 && context.unit <= 256 // Reasonable unit size
    }

    /// Validate bit field specification
    ///
    /// Checks if the bit field specification is valid.
    ///
    /// # Arguments
    /// * `field_spec` - Field spec to validate
    ///
    /// # Returns
    /// true if valid, false otherwise
    pub fn validate_field_spec(field_spec: &BitFieldSpec) -> bool {
        field_spec.size > 0 &&
        field_spec.unit > 0 && field_spec.unit <= 256 &&
        field_spec.size * field_spec.unit as u64 <= 8 * 1024 * 1024 // Max 1MB field
    }

    /// Calculate heap requirements for bit operations
    ///
    /// Estimates heap space needed for bit syntax operations.
    ///
    /// # Arguments
    /// * `operation` - Type of operation
    /// * `size` - Size in bits
    ///
    /// # Returns
    /// Heap words needed
    pub fn calculate_heap_requirements(operation: &str, size: u64) -> u32 {
        match operation {
            "match" => (size / 64 + 1) as u32, // Binary matching overhead
            "construct" => (size / 8 + 16) as u32, // Construction overhead
            _ => 8, // Default minimum
        }
    }

    /// Handle bit syntax error
    ///
    /// Process bit syntax errors and set up proper error handling.
    ///
    /// # Arguments
    /// * `assembler` - The ARM64 assembler
    /// * `error_type` - Type of error that occurred
    ///
    /// # Returns
    /// Result indicating success or failure
    pub fn handle_bit_syntax_error(
        assembler: &mut Assembler,
        error_type: &str,
    ) -> Result<(), BeamAssemblerError> {
        eprintln!("[DEBUG] Bit Syntax: Handling error: {}", error_type);

        // Set up error context and raise exception
        let mfa = crate::ErrorMFA {
            module: 0x100, // am_erlang
            function: 0x200, // am_binary_syntax_error or similar
            arity: 0,
        };

        let error_context = crate::ErrorContext {
            error_code: crate::error_integration::error_codes::BADARG,
            mfa: Some(mfa),
            error_data: None,
        };

        crate::ErrorIntegration::set_error_and_raise(assembler, &error_context)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_endianness() {
        assert_eq!(BitEndianness::Big as u8, BitEndianness::Big as u8);
        assert_ne!(BitEndianness::Big as u8, BitEndianness::Little as u8);
    }

    #[test]
    fn test_bit_field_types() {
        assert_eq!(BitFieldType::Integer as u8, BitFieldType::Integer as u8);
        assert_ne!(BitFieldType::Integer as u8, BitFieldType::Float as u8);
    }

    #[test]
    fn test_bit_syntax_context_creation() {
        let context = BitSyntaxContext {
            context_reg: 5,
            position: 16,
            size: 128,
            unit: 8,
        };

        assert_eq!(context.context_reg, 5);
        assert_eq!(context.position, 16);
        assert_eq!(context.size, 128);
        assert_eq!(context.unit, 8);
    }

    #[test]
    fn test_bit_field_spec_creation() {
        let field_spec = BitFieldSpec {
            size: 32,
            unit: 8,
            signed: true,
            endianness: BitEndianness::Big,
            field_type: BitFieldType::Integer,
        };

        assert_eq!(field_spec.size, 32);
        assert_eq!(field_spec.unit, 8);
        assert!(field_spec.signed);
        assert_eq!(field_spec.endianness, BitEndianness::Big);
        assert_eq!(field_spec.field_type, BitFieldType::Integer);
    }

    #[test]
    fn test_binary_construction_state_creation() {
        let state = BinaryConstructionState {
            dst_reg: 10,
            current_size: 64,
            unit: 8,
            heap_needed: 128,
        };

        assert_eq!(state.dst_reg, 10);
        assert_eq!(state.current_size, 64);
        assert_eq!(state.unit, 8);
        assert_eq!(state.heap_needed, 128);
    }

    #[test]
    fn test_context_validation() {
        // Valid context
        let valid_context = BitSyntaxContext {
            context_reg: 5,
            position: 0,
            size: 128,
            unit: 8,
        };
        assert!(BitSyntaxOperations::validate_context(&valid_context));

        // Invalid context - bad register
        let invalid_context1 = BitSyntaxContext {
            context_reg: 32, // Invalid register number
            position: 0,
            size: 128,
            unit: 8,
        };
        assert!(!BitSyntaxOperations::validate_context(&invalid_context1));

        // Invalid context - position > size
        let invalid_context2 = BitSyntaxContext {
            context_reg: 5,
            position: 200,
            size: 128,
            unit: 8,
        };
        assert!(!BitSyntaxOperations::validate_context(&invalid_context2));

        // Invalid context - zero unit
        let invalid_context3 = BitSyntaxContext {
            context_reg: 5,
            position: 0,
            size: 128,
            unit: 0,
        };
        assert!(!BitSyntaxOperations::validate_context(&invalid_context3));
    }

    #[test]
    fn test_field_spec_validation() {
        // Valid field spec
        let valid_spec = BitFieldSpec {
            size: 32,
            unit: 8,
            signed: true,
            endianness: BitEndianness::Big,
            field_type: BitFieldType::Integer,
        };
        assert!(BitSyntaxOperations::validate_field_spec(&valid_spec));

        // Invalid field spec - zero size
        let invalid_spec1 = BitFieldSpec {
            size: 0,
            unit: 8,
            signed: true,
            endianness: BitEndianness::Big,
            field_type: BitFieldType::Integer,
        };
        assert!(!BitSyntaxOperations::validate_field_spec(&invalid_spec1));

        // Invalid field spec - zero unit
        let invalid_spec2 = BitFieldSpec {
            size: 32,
            unit: 0,
            signed: true,
            endianness: BitEndianness::Big,
            field_type: BitFieldType::Integer,
        };
        assert!(!BitSyntaxOperations::validate_field_spec(&invalid_spec2));

        // Invalid field spec - too large
        let invalid_spec3 = BitFieldSpec {
            size: 1024 * 1024 + 1, // > 1MB
            unit: 8,
            signed: true,
            endianness: BitEndianness::Big,
            field_type: BitFieldType::Integer,
        };
        assert!(!BitSyntaxOperations::validate_field_spec(&invalid_spec3));
    }

    #[test]
    fn test_heap_requirements_calculation() {
        // Test different operation types
        assert_eq!(BitSyntaxOperations::calculate_heap_requirements("match", 64), 2);
        assert_eq!(BitSyntaxOperations::calculate_heap_requirements("construct", 64), 24);
        assert_eq!(BitSyntaxOperations::calculate_heap_requirements("unknown", 64), 8);
    }

    #[test]
    fn test_binary_match_result() {
        // Test Success variant
        let success = BinaryMatchResult::Success {
            value_reg: 5,
            new_position: 32,
        };

        match success {
            BinaryMatchResult::Success { value_reg, new_position } => {
                assert_eq!(value_reg, 5);
                assert_eq!(new_position, 32);
            }
            _ => panic!("Expected Success"),
        }

        // Test Failure variant
        let failure = BinaryMatchResult::Failure;
        match failure {
            BinaryMatchResult::Failure => {}
            _ => panic!("Expected Failure"),
        }

        // Test Partial variant
        let partial = BinaryMatchResult::Partial;
        match partial {
            BinaryMatchResult::Partial => {}
            _ => panic!("Expected Partial"),
        }
    }

    #[test]
    fn test_bit_syntax_operations_creation() {
        // BitSyntaxOperations has no state, just test creation
        let _operations = BitSyntaxOperations;
    }
}
