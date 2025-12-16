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

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Tag Constants Tests ====================

    #[test]
    fn test_tag_constants() {
        assert_eq!(tags::TAG_U, 0);
        assert_eq!(tags::TAG_X, 1);
        assert_eq!(tags::TAG_Y, 2);
        assert_eq!(tags::TAG_L, 3);
        assert_eq!(tags::TAG_F, 4);
        assert_eq!(tags::TAG_Q, 5);
        assert_eq!(tags::TAG_BYTE_PTR, b'M' as u64);
        assert_eq!(tags::TAG_CATCH, b'H' as u64);
        assert_eq!(tags::TAG_EXPORT, b'E' as u64);
        assert_eq!(tags::TAG_FUN_ENTRY, b'F' as u64);
        assert_eq!(tags::TAG_IMMEDIATE, b'I' as u64);
    }

    // ==================== ArgVal Constructor Tests ====================

    #[test]
    fn test_argval_new() {
        let arg = ArgVal::new(42, 100);
        assert_eq!(arg.value(), 100);
    }

    #[test]
    fn test_argval_word() {
        let arg = ArgVal::word(12345);
        assert_eq!(arg.tag_type(), ArgType::Word);
        assert_eq!(arg.value(), 12345);
    }

    #[test]
    fn test_argval_word_zero() {
        let arg = ArgVal::word(0);
        assert_eq!(arg.tag_type(), ArgType::Word);
        assert_eq!(arg.value(), 0);
    }

    #[test]
    fn test_argval_word_max() {
        let arg = ArgVal::word(u64::MAX);
        assert_eq!(arg.tag_type(), ArgType::Word);
        assert_eq!(arg.value(), u64::MAX);
    }

    #[test]
    fn test_argval_x_reg() {
        let arg = ArgVal::x_reg(5);
        assert_eq!(arg.tag_type(), ArgType::XReg);
        assert_eq!(arg.value(), 5);
    }

    #[test]
    fn test_argval_x_reg_zero() {
        let arg = ArgVal::x_reg(0);
        assert_eq!(arg.tag_type(), ArgType::XReg);
        assert_eq!(arg.value(), 0);
    }

    #[test]
    fn test_argval_y_reg() {
        let arg = ArgVal::y_reg(10);
        assert_eq!(arg.tag_type(), ArgType::YReg);
        assert_eq!(arg.value(), 10);
    }

    #[test]
    fn test_argval_y_reg_zero() {
        let arg = ArgVal::y_reg(0);
        assert_eq!(arg.tag_type(), ArgType::YReg);
        assert_eq!(arg.value(), 0);
    }

    #[test]
    fn test_argval_f_reg() {
        let arg = ArgVal::f_reg(3);
        assert_eq!(arg.tag_type(), ArgType::FReg);
        assert_eq!(arg.value(), 3);
    }

    #[test]
    fn test_argval_label() {
        let arg = ArgVal::label(100);
        assert_eq!(arg.tag_type(), ArgType::Label);
        assert_eq!(arg.value(), 100);
    }

    #[test]
    fn test_argval_label_zero() {
        let arg = ArgVal::label(0);
        assert_eq!(arg.tag_type(), ArgType::Label);
        assert_eq!(arg.value(), 0);
    }

    #[test]
    fn test_argval_literal() {
        let arg = ArgVal::literal(50);
        assert_eq!(arg.tag_type(), ArgType::Literal);
        assert_eq!(arg.value(), 50);
    }

    #[test]
    fn test_argval_byte_ptr() {
        let data: u8 = 42;
        let ptr = &data as *const u8;
        let arg = ArgVal::byte_ptr(ptr);
        assert_eq!(arg.tag_type(), ArgType::BytePtr);
        assert_eq!(arg.value(), ptr as u64);
    }

    #[test]
    fn test_argval_byte_ptr_null() {
        let arg = ArgVal::byte_ptr(std::ptr::null());
        assert_eq!(arg.tag_type(), ArgType::BytePtr);
        assert_eq!(arg.value(), 0);
    }

    #[test]
    fn test_argval_catch() {
        let arg = ArgVal::catch_(25);
        assert_eq!(arg.tag_type(), ArgType::Catch);
        assert_eq!(arg.value(), 25);
    }

    #[test]
    fn test_argval_export() {
        let arg = ArgVal::export(75);
        assert_eq!(arg.tag_type(), ArgType::Export);
        assert_eq!(arg.value(), 75);
    }

    #[test]
    fn test_argval_fun_entry() {
        let arg = ArgVal::fun_entry(30);
        assert_eq!(arg.tag_type(), ArgType::FunEntry);
        assert_eq!(arg.value(), 30);
    }

    #[test]
    fn test_argval_immediate() {
        let arg = ArgVal::immediate(999);
        assert_eq!(arg.tag_type(), ArgType::Immediate);
        assert_eq!(arg.value(), 999);
    }

    #[test]
    fn test_argval_immediate_zero() {
        let arg = ArgVal::immediate(0);
        assert_eq!(arg.tag_type(), ArgType::Immediate);
        assert_eq!(arg.value(), 0);
    }

    // ==================== ArgVal Type Check Tests ====================

    #[test]
    fn test_is_byte_ptr() {
        let arg = ArgVal::byte_ptr(std::ptr::null());
        assert!(arg.is_byte_ptr());
        
        let non_byte_ptr = ArgVal::word(0);
        assert!(!non_byte_ptr.is_byte_ptr());
    }

    #[test]
    fn test_is_catch() {
        let arg = ArgVal::catch_(0);
        assert!(arg.is_catch());
        
        let non_catch = ArgVal::word(0);
        assert!(!non_catch.is_catch());
    }

    #[test]
    fn test_is_export() {
        let arg = ArgVal::export(0);
        assert!(arg.is_export());
        
        let non_export = ArgVal::word(0);
        assert!(!non_export.is_export());
    }

    #[test]
    fn test_is_immediate() {
        let arg = ArgVal::immediate(100);
        assert!(arg.is_immediate());
        
        let non_immediate = ArgVal::word(100);
        assert!(!non_immediate.is_immediate());
    }

    #[test]
    fn test_is_label() {
        let arg = ArgVal::label(10);
        assert!(arg.is_label());
        
        let non_label = ArgVal::word(10);
        assert!(!non_label.is_label());
    }

    #[test]
    fn test_is_lambda() {
        let arg = ArgVal::fun_entry(5);
        assert!(arg.is_lambda());
        
        let non_lambda = ArgVal::word(5);
        assert!(!non_lambda.is_lambda());
    }

    #[test]
    fn test_is_literal() {
        let arg = ArgVal::literal(20);
        assert!(arg.is_literal());
        
        let non_literal = ArgVal::word(20);
        assert!(!non_literal.is_literal());
    }

    #[test]
    fn test_is_constant_immediate() {
        let arg = ArgVal::immediate(100);
        assert!(arg.is_constant());
    }

    #[test]
    fn test_is_constant_literal() {
        let arg = ArgVal::literal(50);
        assert!(arg.is_constant());
    }

    #[test]
    fn test_is_constant_non_constant() {
        let arg = ArgVal::x_reg(0);
        assert!(!arg.is_constant());
    }

    #[test]
    fn test_is_nil() {
        let arg = ArgVal::immediate(0);
        assert!(arg.is_nil());
        
        let non_nil = ArgVal::immediate(1);
        assert!(!non_nil.is_nil());
        
        let non_immediate = ArgVal::word(0);
        assert!(!non_immediate.is_nil());
    }

    #[test]
    fn test_is_small() {
        // Value with low bits == 1 is small
        let small = ArgVal::immediate(0b101); // 5, low bits = 01
        assert!(small.is_small());
        
        // Value with low bits != 1 is not small
        let not_small = ArgVal::immediate(0b100); // 4, low bits = 00
        assert!(!not_small.is_small());
        
        // Non-immediate is not small
        let non_immediate = ArgVal::word(0b101);
        assert!(!non_immediate.is_small());
    }

    #[test]
    fn test_is_atom() {
        // Value with low bits == 0 and non-zero is atom
        let atom = ArgVal::immediate(4); // low bits = 00, non-zero
        assert!(atom.is_atom());
        
        // Zero value is not atom
        let zero = ArgVal::immediate(0);
        assert!(!zero.is_atom());
        
        // Value with low bits != 0 is not atom
        let not_atom = ArgVal::immediate(5); // low bits = 01
        assert!(!not_atom.is_atom());
        
        // Non-immediate is not atom
        let non_immediate = ArgVal::word(4);
        assert!(!non_immediate.is_atom());
    }

    // ==================== ArgVal tag_type Tests ====================

    #[test]
    fn test_tag_type_all_types() {
        assert_eq!(ArgVal::word(0).tag_type(), ArgType::Word);
        assert_eq!(ArgVal::x_reg(0).tag_type(), ArgType::XReg);
        assert_eq!(ArgVal::y_reg(0).tag_type(), ArgType::YReg);
        assert_eq!(ArgVal::f_reg(0).tag_type(), ArgType::FReg);
        assert_eq!(ArgVal::label(0).tag_type(), ArgType::Label);
        assert_eq!(ArgVal::literal(0).tag_type(), ArgType::Literal);
        assert_eq!(ArgVal::byte_ptr(std::ptr::null()).tag_type(), ArgType::BytePtr);
        assert_eq!(ArgVal::catch_(0).tag_type(), ArgType::Catch);
        assert_eq!(ArgVal::export(0).tag_type(), ArgType::Export);
        assert_eq!(ArgVal::fun_entry(0).tag_type(), ArgType::FunEntry);
        assert_eq!(ArgVal::immediate(0).tag_type(), ArgType::Immediate);
    }

    #[test]
    fn test_tag_type_unknown_defaults_to_word() {
        // Create an ArgVal with an unknown tag
        let arg = ArgVal::new(9999, 42);
        assert_eq!(arg.tag_type(), ArgType::Word);
    }

    // ==================== ArgVal Trait Tests ====================

    #[test]
    fn test_argval_debug() {
        let arg = ArgVal::x_reg(5);
        let debug = format!("{:?}", arg);
        assert!(debug.contains("ArgVal"));
    }

    #[test]
    fn test_argval_clone() {
        let arg = ArgVal::immediate(100);
        let cloned = arg.clone();
        assert_eq!(arg, cloned);
    }

    #[test]
    fn test_argval_copy() {
        let arg = ArgVal::y_reg(3);
        let copied = arg;
        assert_eq!(arg, copied);
    }

    #[test]
    fn test_argval_eq() {
        let arg1 = ArgVal::x_reg(5);
        let arg2 = ArgVal::x_reg(5);
        let arg3 = ArgVal::x_reg(6);
        let arg4 = ArgVal::y_reg(5);
        
        assert_eq!(arg1, arg2);
        assert_ne!(arg1, arg3);
        assert_ne!(arg1, arg4);
    }

    // ==================== ArgType Tests ====================

    #[test]
    fn test_argtype_debug() {
        let arg_type = ArgType::XReg;
        let debug = format!("{:?}", arg_type);
        assert_eq!(debug, "XReg");
    }

    #[test]
    fn test_argtype_clone() {
        let arg_type = ArgType::Label;
        let cloned = arg_type.clone();
        assert_eq!(arg_type, cloned);
    }

    #[test]
    fn test_argtype_copy() {
        let arg_type = ArgType::Immediate;
        let copied = arg_type;
        assert_eq!(arg_type, copied);
    }

    #[test]
    fn test_argtype_eq() {
        assert_eq!(ArgType::Word, ArgType::Word);
        assert_ne!(ArgType::Word, ArgType::XReg);
    }

    #[test]
    fn test_argtype_all_variants() {
        // Ensure all variants exist and can be matched
        let variants = [
            ArgType::Word,
            ArgType::XReg,
            ArgType::YReg,
            ArgType::FReg,
            ArgType::Label,
            ArgType::Literal,
            ArgType::BytePtr,
            ArgType::Catch,
            ArgType::Export,
            ArgType::FunEntry,
            ArgType::Immediate,
        ];
        
        for v in variants.iter() {
            match v {
                ArgType::Word => {},
                ArgType::XReg => {},
                ArgType::YReg => {},
                ArgType::FReg => {},
                ArgType::Label => {},
                ArgType::Literal => {},
                ArgType::BytePtr => {},
                ArgType::Catch => {},
                ArgType::Export => {},
                ArgType::FunEntry => {},
                ArgType::Immediate => {},
            }
        }
    }

    // ==================== Helper Function Tests ====================

    #[test]
    fn test_is_atom_value() {
        // Low bits == 0 and non-zero is atom
        assert!(is_atom_value(4)); // 0b100
        assert!(is_atom_value(8)); // 0b1000
        assert!(is_atom_value(0x100)); // large even value
        
        // Zero is not atom
        assert!(!is_atom_value(0));
        
        // Low bits != 0 is not atom
        assert!(!is_atom_value(1)); // 0b01
        assert!(!is_atom_value(2)); // 0b10
        assert!(!is_atom_value(3)); // 0b11
        assert!(!is_atom_value(5)); // 0b101
    }

    #[test]
    fn test_is_small_value() {
        // Low bits == 1 is small
        assert!(is_small_value(1)); // 0b01
        assert!(is_small_value(5)); // 0b101
        assert!(is_small_value(9)); // 0b1001
        assert!(is_small_value(0x101)); // larger with low bits = 01
        
        // Low bits != 1 is not small
        assert!(!is_small_value(0)); // 0b00
        assert!(!is_small_value(2)); // 0b10
        assert!(!is_small_value(3)); // 0b11
        assert!(!is_small_value(4)); // 0b100
    }

    // ==================== Edge Case Tests ====================

    #[test]
    fn test_large_register_index() {
        let arg = ArgVal::x_reg(usize::MAX);
        assert_eq!(arg.value(), usize::MAX as u64);
    }

    #[test]
    fn test_large_label_index() {
        let arg = ArgVal::label(usize::MAX);
        assert_eq!(arg.value(), usize::MAX as u64);
    }

    #[test]
    fn test_argval_value_roundtrip() {
        let values = [0u64, 1, 100, 1000, u64::MAX / 2, u64::MAX];
        for &val in &values {
            let arg = ArgVal::word(val);
            assert_eq!(arg.value(), val);
        }
    }

    #[test]
    fn test_multiple_type_checks_false() {
        // An XReg should not match any other type checks
        let arg = ArgVal::x_reg(5);
        assert!(!arg.is_byte_ptr());
        assert!(!arg.is_catch());
        assert!(!arg.is_constant());
        assert!(!arg.is_export());
        assert!(!arg.is_immediate());
        assert!(!arg.is_label());
        assert!(!arg.is_lambda());
        assert!(!arg.is_literal());
        assert!(!arg.is_nil());
        assert!(!arg.is_small());
        assert!(!arg.is_atom());
    }

    #[test]
    fn test_byte_ptr_high_address() {
        // Test with a high memory address (simulated)
        let high_addr: u64 = 0xFFFF_FFFF_FFFF_0000;
        let arg = ArgVal::new(tags::TAG_BYTE_PTR, high_addr);
        assert!(arg.is_byte_ptr());
        assert_eq!(arg.value(), high_addr);
    }
}

