//! Argument value types for BEAM instructions
//!
//! Type-safe wrappers for BEAM operation arguments, converted from C++ ArgVal.

/// Tag values for argument types
mod tags {
    pub const TAG_U: u64 = 0;
    pub const TAG_X: u64 = 1;
    pub const TAG_Y: u64 = 2;
    pub const TAG_L: u64 = 3;
    pub const TAG_F: u64 = 4;
    pub const TAG_Q: u64 = 5;
    
    pub const TAG_BYTE_PTR: u64 = b'M' as u64;
    pub const TAG_CATCH: u64 = b'H' as u64;
    pub const TAG_EXPORT: u64 = b'E' as u64;
    pub const TAG_FUN_ENTRY: u64 = b'F' as u64;
    pub const TAG_IMMEDIATE: u64 = b'I' as u64;
}

/// Argument value type
///
/// Converted from C++ `struct ArgVal : public BeamOpArg`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArgVal {
    tag: u64,
    value: u64,
}

impl ArgVal {
    /// Create a new ArgVal with a specific tag and value
    pub fn new(tag: u64, value: u64) -> Self {
        Self { tag, value }
    }

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

    /// Create an F register argument
    pub fn f_reg(index: usize) -> Self {
        Self {
            tag: tags::TAG_L,
            value: index as u64,
        }
    }

    /// Create a label argument
    pub fn label(index: usize) -> Self {
        Self {
            tag: tags::TAG_F,
            value: index as u64,
        }
    }

    /// Create a literal argument
    pub fn literal(index: usize) -> Self {
        Self {
            tag: tags::TAG_Q,
            value: index as u64,
        }
    }

    /// Create a byte pointer argument
    pub fn byte_ptr(ptr: *const u8) -> Self {
        Self {
            tag: tags::TAG_BYTE_PTR,
            value: ptr as u64,
        }
    }

    /// Create a catch argument
    pub fn catch_(index: usize) -> Self {
        Self {
            tag: tags::TAG_CATCH,
            value: index as u64,
        }
    }

    /// Create an export argument
    pub fn export(index: usize) -> Self {
        Self {
            tag: tags::TAG_EXPORT,
            value: index as u64,
        }
    }

    /// Create a function entry argument
    pub fn fun_entry(index: usize) -> Self {
        Self {
            tag: tags::TAG_FUN_ENTRY,
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
            tags::TAG_L => ArgType::FReg,
            tags::TAG_F => ArgType::Label,
            tags::TAG_Q => ArgType::Literal,
            tags::TAG_BYTE_PTR => ArgType::BytePtr,
            tags::TAG_CATCH => ArgType::Catch,
            tags::TAG_EXPORT => ArgType::Export,
            tags::TAG_FUN_ENTRY => ArgType::FunEntry,
            tags::TAG_IMMEDIATE => ArgType::Immediate,
            _ => ArgType::Word, // Default fallback
        }
    }

    /// Get the value
    pub fn value(&self) -> u64 {
        self.value
    }

    /// Check if this is an atom
    pub fn is_atom(&self) -> bool {
        self.is_immediate() && is_atom_value(self.value)
    }

    /// Check if this is a byte pointer
    pub fn is_byte_ptr(&self) -> bool {
        self.tag == tags::TAG_BYTE_PTR
    }

    /// Check if this is a catch
    pub fn is_catch(&self) -> bool {
        self.tag == tags::TAG_CATCH
    }

    /// Check if this is a constant (immediate or literal)
    pub fn is_constant(&self) -> bool {
        self.is_immediate() || self.is_literal()
    }

    /// Check if this is an export
    pub fn is_export(&self) -> bool {
        self.tag == tags::TAG_EXPORT
    }

    /// Check if this is immediate
    pub fn is_immediate(&self) -> bool {
        self.tag == tags::TAG_IMMEDIATE
    }

    /// Check if this is a label
    pub fn is_label(&self) -> bool {
        self.tag == tags::TAG_F
    }

    /// Check if this is a lambda
    pub fn is_lambda(&self) -> bool {
        self.tag == tags::TAG_FUN_ENTRY
    }

    /// Check if this is a literal
    pub fn is_literal(&self) -> bool {
        self.tag == tags::TAG_Q
    }

    /// Check if this is nil
    pub fn is_nil(&self) -> bool {
        self.is_immediate() && self.value == 0 // NIL constant
    }

    /// Check if this is a small integer
    pub fn is_small(&self) -> bool {
        self.is_immediate() && is_small_value(self.value)
    }
}

/// Argument type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgType {
    Word,
    XReg,
    YReg,
    FReg,
    Label,
    Literal,
    BytePtr,
    Catch,
    Export,
    FunEntry,
    Immediate,
}

/// Check if a value is an atom
fn is_atom_value(value: u64) -> bool {
    // Check if value is tagged as an atom
    // This would need to match the actual atom tagging scheme
    (value & 0x3) == 0 && value != 0 // Placeholder
}

/// Check if a value is a small integer
fn is_small_value(value: u64) -> bool {
    // Check if value is tagged as a small integer
    // This would need to match the actual small integer tagging scheme
    (value & 0x3) == 1 // Placeholder
}

