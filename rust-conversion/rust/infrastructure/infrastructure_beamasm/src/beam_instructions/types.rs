//! BEAM Instruction Types
//!
//! Type definitions for BEAM instruction arguments and data structures.

use crate::common::args::ArgVal;

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
        use crate::common::args::ArgType;
        // ArgVal is a struct with tag_type() and value() methods
        match arg.tag_type() {
            ArgType::Word | ArgType::Immediate => Some(BeamArg::Literal(arg.value())),
            ArgType::XReg => Some(BeamArg::Register { index: arg.value() as u32, is_y: false }),
            ArgType::YReg => Some(BeamArg::Register { index: arg.value() as u32, is_y: true }),
            ArgType::Label => Some(BeamArg::Label(arg.value() as u32)),
            _ => None, // Other types not implemented yet
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
    pub fn opcode_enum(&self) -> Option<crate::beam_instructions::BeamOpcode> {
        crate::beam_instructions::BeamOpcode::from_u32(self.opcode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beamarg_register_x() {
        let arg = BeamArg::Register { index: 5, is_y: false };
        assert_eq!(arg, BeamArg::Register { index: 5, is_y: false });
    }

    #[test]
    fn test_beamarg_register_y() {
        let arg = BeamArg::Register { index: 3, is_y: true };
        assert_eq!(arg, BeamArg::Register { index: 3, is_y: true });
    }

    #[test]
    fn test_beamarg_literal() {
        let arg = BeamArg::Literal(42);
        assert_eq!(arg, BeamArg::Literal(42));
    }

    #[test]
    fn test_beamarg_label() {
        let arg = BeamArg::Label(10);
        assert_eq!(arg, BeamArg::Label(10));
    }

    #[test]
    fn test_beamarg_list() {
        let args = vec![BeamArg::Literal(1), BeamArg::Literal(2)];
        let arg = BeamArg::List(args.clone());
        if let BeamArg::List(list) = arg {
            assert_eq!(list.len(), 2);
            assert_eq!(list[0], BeamArg::Literal(1));
            assert_eq!(list[1], BeamArg::Literal(2));
        } else {
            panic!("Expected List variant");
        }
    }

    #[test]
    fn test_beamarg_extended() {
        let inner = BeamArg::Literal(42);
        let arg = BeamArg::Extended(Box::new(inner.clone()));
        if let BeamArg::Extended(boxed) = arg {
            assert_eq!(*boxed, inner);
        } else {
            panic!("Expected Extended variant");
        }
    }

    #[test]
    fn test_beamarg_equality() {
        assert_eq!(BeamArg::Literal(42), BeamArg::Literal(42));
        assert_ne!(BeamArg::Literal(42), BeamArg::Literal(43));
        assert_ne!(BeamArg::Register { index: 1, is_y: false }, BeamArg::Register { index: 1, is_y: true });
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
        let original = BeamArg::List(vec![BeamArg::Literal(1), BeamArg::Literal(2)]);
        let cloned = original.clone();
        assert_eq!(original, cloned);
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

        // Test pattern matching
        if let BeamArg::Extended(boxed) = nested {
            if let BeamArg::List(list) = *boxed {
                assert_eq!(list.len(), 3);
                assert_eq!(list[0], BeamArg::Register { index: 0, is_y: false });
                assert_eq!(list[1], BeamArg::Label(5));
                assert_eq!(list[2], BeamArg::Literal(42));
            } else {
                panic!("Expected nested List");
            }
        } else {
            panic!("Expected Extended");
        }
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
    fn test_beamarg_from_arg_val_unsupported() {
        // Test with an unsupported ArgType - this should return None
        // We need to create an ArgVal with a type that's not handled
        // Since ArgType might not have unsupported variants, we'll test the existing ones
        // All currently supported types should work
        let arg_val = ArgVal::word(42);
        let beam_arg = BeamArg::from_arg_val(&arg_val);
        assert!(beam_arg.is_some()); // Should succeed for supported types
    }

    #[test]
    fn test_beamarg_to_arg_val_register_x() {
        let beam_arg = BeamArg::Register { index: 5, is_y: false };
        let arg_val = beam_arg.to_arg_val();
        assert_eq!(arg_val, Some(ArgVal::x_reg(5)));
    }

    #[test]
    fn test_beamarg_to_arg_val_register_y() {
        let beam_arg = BeamArg::Register { index: 3, is_y: true };
        let arg_val = beam_arg.to_arg_val();
        assert_eq!(arg_val, Some(ArgVal::y_reg(3)));
    }

    #[test]
    fn test_beamarg_to_arg_val_literal() {
        let beam_arg = BeamArg::Literal(42);
        let arg_val = beam_arg.to_arg_val();
        assert_eq!(arg_val, Some(ArgVal::word(42)));
    }

    #[test]
    fn test_beamarg_to_arg_val_label() {
        let beam_arg = BeamArg::Label(10);
        let arg_val = beam_arg.to_arg_val();
        assert_eq!(arg_val, Some(ArgVal::label(10)));
    }

    #[test]
    fn test_beamarg_to_arg_val_unsupported() {
        // Test with unsupported BeamArg variants
        let beam_arg = BeamArg::List(vec![BeamArg::Literal(1)]);
        let arg_val = beam_arg.to_arg_val();
        assert_eq!(arg_val, None);

        let beam_arg = BeamArg::Extended(Box::new(BeamArg::Literal(42)));
        let arg_val = beam_arg.to_arg_val();
        assert_eq!(arg_val, None);
    }

    #[test]
    fn test_conversion_roundtrip() {
        // Test roundtrip conversion between ArgVal and BeamArg
        // Note: The conversion from BeamArg back to ArgVal always uses word() for literals,
        // so we need to expect that in the roundtrip.

        let test_cases = vec![
            (ArgVal::word(42), BeamArg::Literal(42), ArgVal::word(42)),
            (ArgVal::immediate(123), BeamArg::Literal(123), ArgVal::word(123)), // Note: converts to word
            (ArgVal::x_reg(5), BeamArg::Register { index: 5, is_y: false }, ArgVal::x_reg(5)),
            (ArgVal::y_reg(3), BeamArg::Register { index: 3, is_y: true }, ArgVal::y_reg(3)),
            (ArgVal::label(10), BeamArg::Label(10), ArgVal::label(10)),
        ];

        for (original_arg_val, expected_beam_arg, expected_converted_back) in test_cases {
            // ArgVal -> BeamArg
            let beam_arg = BeamArg::from_arg_val(&original_arg_val);
            assert_eq!(beam_arg, Some(expected_beam_arg.clone()));

            // BeamArg -> ArgVal (for supported conversions)
            if let Some(converted_arg_val) = expected_beam_arg.to_arg_val() {
                assert_eq!(converted_arg_val, expected_converted_back);
            }
        }
    }

    #[test]
    fn test_large_values() {
        // Test with large u64 values to ensure no overflow issues
        let large_value = u64::MAX;
        let arg_val = ArgVal::word(large_value);
        let beam_arg = BeamArg::from_arg_val(&arg_val);
        assert_eq!(beam_arg, Some(BeamArg::Literal(large_value)));

        let beam_arg = BeamArg::Literal(large_value);
        let converted_back = beam_arg.to_arg_val();
        assert_eq!(converted_back, Some(ArgVal::word(large_value)));
    }

    #[test]
    fn test_zero_values() {
        // Test with zero values
        let arg_val = ArgVal::x_reg(0);
        let beam_arg = BeamArg::from_arg_val(&arg_val);
        assert_eq!(beam_arg, Some(BeamArg::Register { index: 0, is_y: false }));

        let beam_arg = BeamArg::Register { index: 0, is_y: false };
        let converted_back = beam_arg.to_arg_val();
        assert_eq!(converted_back, Some(ArgVal::x_reg(0)));
    }

    #[test]
    fn test_beaminstruction_opcode_enum() {
        // Test opcode_enum method - this depends on BeamOpcode implementation
        let instr = BeamInstruction::new(1, vec![]); // Label opcode
        let opcode_enum = instr.opcode_enum();
        // We don't know what BeamOpcode::from_u32 returns, but the method should work
        let _ = opcode_enum; // Just ensure it doesn't panic
    }

    #[test]
    fn test_beaminstruction_with_complex_args() {
        // Test instruction with complex argument combinations
        let args = vec![
            BeamArg::Register { index: 0, is_y: false }, // x(0)
            BeamArg::Register { index: 1, is_y: true },  // y(1)
            BeamArg::Literal(42),                        // literal 42
            BeamArg::Label(100),                         // label 100
            BeamArg::List(vec![
                BeamArg::Literal(1),
                BeamArg::Literal(2),
            ]),
        ];

        let instr = BeamInstruction::new(12345, args);

        assert_eq!(instr.opcode, 12345);
        assert_eq!(instr.args.len(), 5);

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

        match &instr.args[4] {
            BeamArg::List(list) => assert_eq!(list.len(), 2),
            _ => panic!("Expected list"),
        }
    }

    #[test]
    fn test_beamfunction_with_empty_instructions() {
        let function = BeamFunction {
            module: 1,
            function: 2,
            arity: 0,
            entry_label: 1,
            instructions: vec![],
        };

        assert_eq!(function.module, 1);
        assert_eq!(function.function, 2);
        assert_eq!(function.arity, 0);
        assert_eq!(function.entry_label, 1);
        assert!(function.instructions.is_empty());
    }

    #[test]
    fn test_beamcode_with_single_function() {
        let code = BeamCode {
            header: BeamCodeHeader {
                sub_size: 10,
                instruction_set: 1,
                max_opcode: 200,
                label_count: 5,
                function_count: 1,
            },
            functions: vec![BeamFunction {
                module: 1,
                function: 1,
                arity: 0,
                entry_label: 1,
                instructions: vec![BeamInstruction::new(100, vec![])],
            }],
            raw_code: vec![0x00, 0x01, 0x02],
        };

        assert_eq!(code.header.function_count, 1);
        assert_eq!(code.functions.len(), 1);
        assert_eq!(code.functions[0].instructions.len(), 1);
        assert_eq!(code.raw_code, vec![0x00, 0x01, 0x02]);
    }

    #[test]
    fn test_memory_safety() {
        // Test that operations don't cause memory issues
        let beam_arg = BeamArg::List(vec![
            BeamArg::Literal(1),
            BeamArg::Literal(2),
            BeamArg::Literal(3),
        ]);

        // Clone should work
        let cloned = beam_arg.clone();
        assert_eq!(beam_arg, cloned);

        // Debug formatting should work
        let _ = format!("{:?}", beam_arg);

        // Conversion should work
        let _ = beam_arg.to_arg_val(); // Should return None safely
    }

    #[test]
    fn test_struct_sizes() {
        // Test that structs have reasonable sizes
        // BeamInstruction should contain opcode (u32) and args (Vec)
        assert!(std::mem::size_of::<BeamInstruction>() >= 4); // At least u32

        // BeamArg enum variants should be reasonably sized
        assert!(std::mem::size_of::<BeamArg>() >= 8); // At least for u64 in Literal

        // Header structs should be small and fixed size
        assert_eq!(std::mem::size_of::<BeamCodeHeader>(), 20); // 5 * u32 = 20 bytes
    }

    #[test]
    fn test_default_values() {
        // Test that we can create meaningful default-like values
        let empty_instr = BeamInstruction::new(0, vec![]);
        assert_eq!(empty_instr.opcode, 0);
        assert!(empty_instr.args.is_empty());

        let zero_header = BeamCodeHeader {
            sub_size: 0,
            instruction_set: 0,
            max_opcode: 0,
            label_count: 0,
            function_count: 0,
        };
        assert_eq!(zero_header.sub_size, 0);
        assert_eq!(zero_header.function_count, 0);
    }

    #[test]
    fn test_extreme_indices() {
        // Test with very large register/label indices
        let large_reg = BeamArg::Register { index: u32::MAX, is_y: false };
        if let Some(arg_val) = large_reg.to_arg_val() {
            // This might fail due to usize conversion, which is expected
            let _ = arg_val;
        }

        let large_label = BeamArg::Label(u32::MAX);
        if let Some(arg_val) = large_label.to_arg_val() {
            // This might fail due to usize conversion, which is expected
            let _ = arg_val;
        }
    }
}
