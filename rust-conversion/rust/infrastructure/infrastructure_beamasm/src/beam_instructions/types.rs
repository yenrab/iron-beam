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
