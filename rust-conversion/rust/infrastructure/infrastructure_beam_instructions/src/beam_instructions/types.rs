//! BEAM Instruction Types
//!
//! Type definitions for BEAM instruction arguments and data structures.

use super::opcodes::BeamOpcode;

/// Tag values for argument types
mod tags {
    pub const TAG_U: u64 = 0;
    pub const TAG_X: u64 = 1;
    pub const TAG_Y: u64 = 2;
    pub const TAG_F: u64 = 4;
    pub const TAG_L: u64 = 3;
    pub const TAG_Q: u64 = 5;
    pub const TAG_IMMEDIATE: u64 = b'I' as u64;
}

/// Argument value type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArgVal {
    tag: u64,
    value: u64,
}

impl ArgVal {
    /// Create a word argument
    pub fn word(value: u64) -> Self {
        Self {
            tag: tags::TAG_U,
            value,
        }
    }

    /// Create an X register argument
    pub fn x_reg(index: usize) -> Self {
        Self {
            tag: tags::TAG_X,
            value: index as u64,
        }
    }

    /// Create a Y register argument
    pub fn y_reg(index: usize) -> Self {
        Self {
            tag: tags::TAG_Y,
            value: index as u64,
        }
    }

    /// Create a label argument
    pub fn label(index: usize) -> Self {
        Self {
            tag: tags::TAG_L,
            value: index as u64,
        }
    }

    /// Create an immediate argument
    pub fn immediate(value: u64) -> Self {
        Self {
            tag: tags::TAG_IMMEDIATE,
            value,
        }
    }

    /// Get the tag type
    pub fn tag_type(&self) -> ArgType {
        match self.tag {
            tags::TAG_U => ArgType::Word,
            tags::TAG_X => ArgType::XReg,
            tags::TAG_Y => ArgType::YReg,
            tags::TAG_L => ArgType::Label,
            tags::TAG_IMMEDIATE => ArgType::Immediate,
            _ => ArgType::Word, // Default
        }
    }

    /// Get the value
    pub fn value(&self) -> u64 {
        self.value
    }
}

/// Argument type classification
#[derive(Debug, Clone, PartialEq)]
pub enum ArgType {
    /// Word-sized immediate value
    Word,
    /// Immediate value
    Immediate,
    /// X register
    XReg,
    /// Y register
    YReg,
    /// Label reference
    Label,
}

/// BEAM instruction argument types
/// These represent the different kinds of arguments that BEAM instructions can take
#[derive(Debug, Clone, PartialEq)]
pub enum BeamArg {
    /// Register reference (x(N), y(N))
    Register { index: u32, is_y: bool },
    /// Literal value (atom, integer, etc.)
    Literal(u64), // Eterm
    /// Label reference for jumps
    Label(u32),
    /// List of arguments
    List(Vec<BeamArg>),
    /// Extended argument (for large values)
    Extended(Box<BeamArg>),
}

/// Parsed BEAM instruction
#[derive(Debug, Clone)]
pub struct BeamInstruction {
    /// The opcode
    pub opcode: u32,
    /// The arguments
    pub args: Vec<BeamArg>,
}

/// BEAM code chunk header
#[derive(Debug)]
pub struct BeamCodeHeader {
    /// Sub-size (size of header in words)
    pub sub_size: u32,
    /// Instruction set version
    pub instruction_set: u32,
    /// Maximum opcode value
    pub max_opcode: u32,
    /// Number of labels
    pub label_count: u32,
    /// Number of functions
    pub function_count: u32,
}

/// Parsed BEAM function
#[derive(Debug)]
pub struct BeamFunction {
    /// Function name atom index
    pub module: u32,
    /// Function name atom index
    pub function: u32,
    /// Function arity
    pub arity: u32,
    /// Entry label
    pub entry_label: u32,
    /// Instructions in this function
    pub instructions: Vec<BeamInstruction>,
}

/// Parsed BEAM code
#[derive(Debug)]
pub struct BeamCode {
    /// Code header
    pub header: BeamCodeHeader,
    /// Functions in the module
    pub functions: Vec<BeamFunction>,
    /// Raw code bytes (for fallback)
    pub raw_code: Vec<u8>,
}

impl BeamArg {
    /// Convert from ArgVal (used in assembler) to BeamArg
    pub fn from_arg_val(arg: &ArgVal) -> Option<Self> {
        // Use local ArgType
        // ArgVal is a struct with tag_type() and value() methods
        match arg.tag_type() {
            ArgType::Word => Some(BeamArg::Literal(arg.value())),
            ArgType::Immediate => Some(BeamArg::Literal(arg.value())),
            ArgType::XReg => Some(BeamArg::Register { index: arg.value() as u32, is_y: false }),
            ArgType::YReg => Some(BeamArg::Register { index: arg.value() as u32, is_y: true }),
            ArgType::Label => Some(BeamArg::Label(arg.value() as u32)),
        }
    }

    /// Convert to ArgVal for assembler
    pub fn to_arg_val(&self) -> Option<ArgVal> {
        match self {
            BeamArg::Register { index, is_y } => {
                if *is_y {
                    Some(ArgVal::y_reg(*index as usize))
                } else {
                    Some(ArgVal::x_reg(*index as usize))
                }
            }
            BeamArg::Literal(value) => Some(ArgVal::word(*value)),
            BeamArg::Label(index) => Some(ArgVal::label(*index as usize)),
            _ => None, // Other types not implemented yet
        }
    }
}

impl BeamInstruction {
    /// Create a new instruction
    pub fn new(opcode: u32, args: Vec<BeamArg>) -> Self {
        Self { opcode, args }
    }

    /// Get the opcode as a BeamOpcode if recognized
    pub fn opcode_enum(&self) -> Option<BeamOpcode> {
        BeamOpcode::from_u32(self.opcode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_argval_word() {
        let arg = ArgVal::word(42);
        assert_eq!(arg.tag_type(), ArgType::Word);
        assert_eq!(arg.value(), 42);
        assert_eq!(arg.tag, tags::TAG_U);
    }

    #[test]
    fn test_argval_x_reg() {
        let arg = ArgVal::x_reg(5);
        assert_eq!(arg.tag_type(), ArgType::XReg);
        assert_eq!(arg.value(), 5);
        assert_eq!(arg.tag, tags::TAG_X);
    }

    #[test]
    fn test_argval_y_reg() {
        let arg = ArgVal::y_reg(3);
        assert_eq!(arg.tag_type(), ArgType::YReg);
        assert_eq!(arg.value(), 3);
        assert_eq!(arg.tag, tags::TAG_Y);
    }

    #[test]
    fn test_argval_label() {
        let arg = ArgVal::label(10);
        assert_eq!(arg.tag_type(), ArgType::Label);
        assert_eq!(arg.value(), 10);
        assert_eq!(arg.tag, tags::TAG_L);
    }

    #[test]
    fn test_argval_immediate() {
        let arg = ArgVal::immediate(123);
        assert_eq!(arg.tag_type(), ArgType::Immediate);
        assert_eq!(arg.value(), 123);
        assert_eq!(arg.tag, tags::TAG_IMMEDIATE);
    }

    #[test]
    fn test_argval_unknown_tag() {
        // Test with an unknown tag - should default to Word
        let mut arg = ArgVal::word(100);
        arg.tag = 999; // Set unknown tag
        assert_eq!(arg.tag_type(), ArgType::Word); // Should default to Word
        assert_eq!(arg.value(), 100);
    }

    #[test]
    fn test_argval_equality() {
        let arg1 = ArgVal::word(42);
        let arg2 = ArgVal::word(42);
        let arg3 = ArgVal::word(43);

        assert_eq!(arg1, arg2);
        assert_ne!(arg1, arg3);
    }

    #[test]
    fn test_argval_debug() {
        let arg = ArgVal::x_reg(5);
        let debug_str = format!("{:?}", arg);
        assert!(debug_str.contains("ArgVal"));
    }

    #[test]
    fn test_argval_clone() {
        let arg1 = ArgVal::y_reg(7);
        let arg2 = arg1.clone();
        assert_eq!(arg1, arg2);
    }

    #[test]
    fn test_argtype_variants() {
        assert_eq!(ArgType::Word, ArgType::Word);
        assert_eq!(ArgType::Immediate, ArgType::Immediate);
        assert_eq!(ArgType::XReg, ArgType::XReg);
        assert_eq!(ArgType::YReg, ArgType::YReg);
        assert_eq!(ArgType::Label, ArgType::Label);

        assert_ne!(ArgType::Word, ArgType::XReg);
    }

    #[test]
    fn test_argtype_debug() {
        let debug_str = format!("{:?}", ArgType::XReg);
        assert!(debug_str.contains("XReg"));
    }

    #[test]
    fn test_argtype_clone() {
        let arg1 = ArgType::YReg;
        let arg2 = arg1.clone();
        assert_eq!(arg1, arg2);
    }

    #[test]
    fn test_beamarg_register_x() {
        let arg = BeamArg::Register { index: 5, is_y: false };
        assert_eq!(arg.to_arg_val(), Some(ArgVal::x_reg(5)));
    }

    #[test]
    fn test_beamarg_register_y() {
        let arg = BeamArg::Register { index: 3, is_y: true };
        assert_eq!(arg.to_arg_val(), Some(ArgVal::y_reg(3)));
    }

    #[test]
    fn test_beamarg_literal() {
        let arg = BeamArg::Literal(42);
        assert_eq!(arg.to_arg_val(), Some(ArgVal::word(42)));
    }

    #[test]
    fn test_beamarg_label() {
        let arg = BeamArg::Label(10);
        assert_eq!(arg.to_arg_val(), Some(ArgVal::label(10)));
    }

    #[test]
    fn test_beamarg_list_conversion() {
        let arg = BeamArg::List(vec![BeamArg::Literal(1), BeamArg::Literal(2)]);
        assert_eq!(arg.to_arg_val(), None); // List not implemented yet
    }

    #[test]
    fn test_beamarg_extended_conversion() {
        let arg = BeamArg::Extended(Box::new(BeamArg::Literal(42)));
        assert_eq!(arg.to_arg_val(), None); // Extended not implemented yet
    }

    #[test]
    fn test_beamarg_from_arg_val_word() {
        let arg_val = ArgVal::word(42);
        let beam_arg = BeamArg::from_arg_val(&arg_val);
        assert_eq!(beam_arg, Some(BeamArg::Literal(42)));
    }

    #[test]
    fn test_beamarg_from_arg_val_immediate() {
        let arg_val = ArgVal::immediate(123);
        let beam_arg = BeamArg::from_arg_val(&arg_val);
        assert_eq!(beam_arg, Some(BeamArg::Literal(123)));
    }

    #[test]
    fn test_beamarg_from_arg_val_x_reg() {
        let arg_val = ArgVal::x_reg(5);
        let beam_arg = BeamArg::from_arg_val(&arg_val);
        assert_eq!(beam_arg, Some(BeamArg::Register { index: 5, is_y: false }));
    }

    #[test]
    fn test_beamarg_from_arg_val_y_reg() {
        let arg_val = ArgVal::y_reg(3);
        let beam_arg = BeamArg::from_arg_val(&arg_val);
        assert_eq!(beam_arg, Some(BeamArg::Register { index: 3, is_y: true }));
    }

    #[test]
    fn test_beamarg_from_arg_val_label() {
        let arg_val = ArgVal::label(10);
        let beam_arg = BeamArg::from_arg_val(&arg_val);
        assert_eq!(beam_arg, Some(BeamArg::Label(10)));
    }

    #[test]
    fn test_beamarg_equality() {
        let arg1 = BeamArg::Literal(42);
        let arg2 = BeamArg::Literal(42);
        let arg3 = BeamArg::Literal(43);

        assert_eq!(arg1, arg2);
        assert_ne!(arg1, arg3);
    }

    #[test]
    fn test_beamarg_debug() {
        let arg = BeamArg::Register { index: 5, is_y: false };
        let debug_str = format!("{:?}", arg);
        assert!(debug_str.contains("Register"));
        assert!(debug_str.contains("5"));
        assert!(debug_str.contains("false"));
    }

    #[test]
    fn test_beamarg_clone() {
        let arg1 = BeamArg::List(vec![BeamArg::Literal(1), BeamArg::Literal(2)]);
        let arg2 = arg1.clone();
        assert_eq!(arg1, arg2);
    }

    #[test]
    fn test_beamarg_complex_nested() {
        let nested = BeamArg::Extended(Box::new(BeamArg::List(vec![
            BeamArg::Register { index: 0, is_y: false },
            BeamArg::Label(5),
            BeamArg::Literal(42),
        ])));

        let cloned = nested.clone();
        assert_eq!(nested, cloned);

        let debug_str = format!("{:?}", nested);
        assert!(debug_str.contains("Extended"));
        assert!(debug_str.contains("List"));
    }

    #[test]
    fn test_beaminstruction_new() {
        let args = vec![BeamArg::Literal(42), BeamArg::Register { index: 0, is_y: false }];
        let instr = BeamInstruction::new(123, args.clone());

        assert_eq!(instr.opcode, 123);
        assert_eq!(instr.args, args);
    }

    #[test]
    fn test_beaminstruction_empty_args() {
        let instr = BeamInstruction::new(456, vec![]);

        assert_eq!(instr.opcode, 456);
        assert!(instr.args.is_empty());
    }

    #[test]
    fn test_beaminstruction_opcode_enum() {
        // Test with a known opcode - this depends on the BeamOpcode implementation
        let instr = BeamInstruction::new(1, vec![]); // Assuming opcode 1 exists
        let opcode_enum = instr.opcode_enum();
        // We don't know what opcode 1 maps to, but the method should work
        let _ = opcode_enum; // Just ensure it doesn't panic
    }

    #[test]
    fn test_beaminstruction_debug() {
        let instr = BeamInstruction::new(123, vec![BeamArg::Literal(42)]);
        let debug_str = format!("{:?}", instr);
        assert!(debug_str.contains("BeamInstruction"));
        assert!(debug_str.contains("123"));
    }

    #[test]
    fn test_beaminstruction_clone() {
        let instr1 = BeamInstruction::new(123, vec![BeamArg::Literal(42)]);
        let instr2 = instr1.clone();
        assert_eq!(instr1.opcode, instr2.opcode);
        assert_eq!(instr1.args, instr2.args);
    }

    #[test]
    fn test_beamcodeheader_debug() {
        let header = BeamCodeHeader {
            sub_size: 10,
            instruction_set: 5,
            max_opcode: 200,
            label_count: 50,
            function_count: 25,
        };

        let debug_str = format!("{:?}", header);
        assert!(debug_str.contains("BeamCodeHeader"));
        assert!(debug_str.contains("10"));
        assert!(debug_str.contains("5"));
        assert!(debug_str.contains("200"));
        assert!(debug_str.contains("50"));
        assert!(debug_str.contains("25"));
    }

    #[test]
    fn test_beamfunction_debug() {
        let function = BeamFunction {
            module: 1,
            function: 2,
            arity: 3,
            entry_label: 4,
            instructions: vec![
                BeamInstruction::new(100, vec![BeamArg::Literal(42)]),
                BeamInstruction::new(101, vec![BeamArg::Register { index: 0, is_y: false }]),
            ],
        };

        let debug_str = format!("{:?}", function);
        assert!(debug_str.contains("BeamFunction"));
        assert!(debug_str.contains("1"));
        assert!(debug_str.contains("2"));
        assert!(debug_str.contains("3"));
        assert!(debug_str.contains("4"));
        assert!(debug_str.contains("BeamInstruction"));
    }

    #[test]
    fn test_beamcode_debug() {
        let code = BeamCode {
            header: BeamCodeHeader {
                sub_size: 10,
                instruction_set: 5,
                max_opcode: 200,
                label_count: 50,
                function_count: 2,
            },
            functions: vec![
                BeamFunction {
                    module: 1,
                    function: 1,
                    arity: 0,
                    entry_label: 1,
                    instructions: vec![BeamInstruction::new(100, vec![])],
                },
                BeamFunction {
                    module: 1,
                    function: 2,
                    arity: 1,
                    entry_label: 2,
                    instructions: vec![BeamInstruction::new(101, vec![BeamArg::Literal(42)])],
                },
            ],
            raw_code: vec![1, 2, 3, 4, 5],
        };

        let debug_str = format!("{:?}", code);
        assert!(debug_str.contains("BeamCode"));
        assert!(debug_str.contains("BeamCodeHeader"));
        assert!(debug_str.contains("BeamFunction"));
        assert!(debug_str.contains("raw_code"));
        assert!(debug_str.contains("[1, 2, 3, 4, 5]"));
    }

    #[test]
    fn test_beamcode_empty() {
        let code = BeamCode {
            header: BeamCodeHeader {
                sub_size: 0,
                instruction_set: 0,
                max_opcode: 0,
                label_count: 0,
                function_count: 0,
            },
            functions: vec![],
            raw_code: vec![],
        };

        assert!(code.functions.is_empty());
        assert!(code.raw_code.is_empty());
    }

    #[test]
    fn test_tag_constants() {
        // Test that the tag constants are defined and have expected values
        assert_eq!(tags::TAG_U, 0);
        assert_eq!(tags::TAG_X, 1);
        assert_eq!(tags::TAG_Y, 2);
        assert_eq!(tags::TAG_L, 3);
        assert_eq!(tags::TAG_F, 4); // Even though unused, test it's defined
        assert_eq!(tags::TAG_Q, 5); // Even though unused, test it's defined
        assert_eq!(tags::TAG_IMMEDIATE, b'I' as u64); // 'I' = 73
        assert_eq!(tags::TAG_IMMEDIATE, 73);
    }

    #[test]
    fn test_large_values() {
        // Test with large u64 values to ensure no overflow issues
        let large_value = u64::MAX;
        let arg = ArgVal::word(large_value);
        assert_eq!(arg.value(), large_value);
        assert_eq!(arg.tag_type(), ArgType::Word);

        let beam_arg = BeamArg::Literal(large_value);
        assert_eq!(beam_arg.to_arg_val(), Some(ArgVal::word(large_value)));
    }

    #[test]
    fn test_zero_values() {
        // Test with zero values
        let arg = ArgVal::x_reg(0);
        assert_eq!(arg.value(), 0);
        assert_eq!(arg.tag_type(), ArgType::XReg);

        let beam_arg = BeamArg::Register { index: 0, is_y: false };
        assert_eq!(beam_arg.to_arg_val(), Some(ArgVal::x_reg(0)));
    }

    #[test]
    fn test_conversion_roundtrip() {
        // Test roundtrip conversion between ArgVal and BeamArg
        let test_cases = vec![
            BeamArg::Literal(42),
            BeamArg::Register { index: 5, is_y: false },
            BeamArg::Register { index: 3, is_y: true },
            BeamArg::Label(10),
        ];

        for beam_arg in test_cases {
            if let Some(arg_val) = beam_arg.to_arg_val() {
                let converted_back = BeamArg::from_arg_val(&arg_val);
                assert_eq!(converted_back, Some(beam_arg));
            } else {
                // Some BeamArg types don't convert to ArgVal yet
                // This is expected for List and Extended types
            }
        }
    }

    #[test]
    fn test_argval_copy() {
        // Test that ArgVal implements Copy correctly
        let arg1 = ArgVal::immediate(123);
        let arg2 = arg1; // Copy
        let arg3 = arg1; // Copy again

        assert_eq!(arg1, arg2);
        assert_eq!(arg2, arg3);
        assert_eq!(arg1.value(), 123);
        assert_eq!(arg2.value(), 123);
        assert_eq!(arg3.value(), 123);
    }

    #[test]
    fn test_struct_sizes() {
        // Test that structs have reasonable sizes
        assert!(std::mem::size_of::<ArgVal>() <= 16); // Should be small (2 u64s)
        assert!(std::mem::size_of::<BeamInstruction>() >= 16); // Should contain opcode and vec
        assert!(std::mem::size_of::<BeamCodeHeader>() == 20); // 5 u32s = 20 bytes
    }

    #[test]
    fn test_default_values() {
        // Test that we can create meaningful default-like values
        let default_arg = ArgVal::word(0);
        assert_eq!(default_arg.value(), 0);
        assert_eq!(default_arg.tag_type(), ArgType::Word);

        let empty_instr = BeamInstruction::new(0, vec![]);
        assert_eq!(empty_instr.opcode, 0);
        assert!(empty_instr.args.is_empty());
    }

    #[test]
    fn test_extreme_indices() {
        // Test with very large register/label indices
        let large_reg = BeamArg::Register { index: u32::MAX, is_y: false };
        if let Some(arg_val) = large_reg.to_arg_val() {
            assert_eq!(arg_val.value(), u32::MAX as u64);
        }

        let large_label = BeamArg::Label(u32::MAX);
        if let Some(arg_val) = large_label.to_arg_val() {
            assert_eq!(arg_val.value(), u32::MAX as u64);
        }
    }

    #[test]
    fn test_memory_safety() {
        // Test that operations don't cause memory issues
        let arg = ArgVal::word(42);

        // Multiple calls to value() should return the same result
        assert_eq!(arg.value(), 42);
        assert_eq!(arg.value(), 42);

        // Multiple calls to tag_type() should return the same result
        assert_eq!(arg.tag_type(), ArgType::Word);
        assert_eq!(arg.tag_type(), ArgType::Word);
    }

    #[test]
    fn test_complex_instruction() {
        // Test a complex instruction with multiple argument types
        let args = vec![
            BeamArg::Register { index: 0, is_y: false }, // x(0)
            BeamArg::Register { index: 1, is_y: true },  // y(1)
            BeamArg::Literal(42),                        // literal 42
            BeamArg::Label(100),                         // label 100
        ];

        let instr = BeamInstruction::new(12345, args);

        assert_eq!(instr.opcode, 12345);
        assert_eq!(instr.args.len(), 4);

        // Check each argument type
        match &instr.args[0] {
            BeamArg::Register { index: 0, is_y: false } => {},
            _ => panic!("Expected x(0) register"),
        }

        match &instr.args[1] {
            BeamArg::Register { index: 1, is_y: true } => {},
            _ => panic!("Expected y(1) register"),
        }

        match &instr.args[2] {
            BeamArg::Literal(42) => {},
            _ => panic!("Expected literal 42"),
        }

        match &instr.args[3] {
            BeamArg::Label(100) => {},
            _ => panic!("Expected label 100"),
        }
    }
}
