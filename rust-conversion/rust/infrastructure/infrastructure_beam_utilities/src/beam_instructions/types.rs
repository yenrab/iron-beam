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
