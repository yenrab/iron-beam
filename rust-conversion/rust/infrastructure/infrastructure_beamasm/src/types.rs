//! Type definitions for BeamAsm JIT
//!
//! Type-safe wrappers around BEAM type definitions, converted from C++ templates
//! to Rust enums and traits.

// BeamType would come from entities_data_handling, but for now we define a placeholder
// This will need to be updated when the actual BeamType structure is available
#[derive(Debug, Clone, Copy)]
pub struct BeamType {
    pub type_union: i32,
    pub metadata_flags: u32,
    pub max: i64,
    pub min: i64,
    pub size_unit: u32,
}

/// Type-safe wrapper around BEAM type IDs
///
/// Converted from C++ `enum class BeamTypeId` to Rust enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum BeamTypeId {
    None = 0,
    Atom = 1,
    Bitstring = 2,
    Cons = 4,
    Float = 8,
    Fun = 16,
    Integer = 32,
    Map = 64,
    Nil = 128,
    Pid = 256,
    Port = 512,
    Reference = 1024,
    Tuple = 2048,
    Any = 0x7FFFFFFF, // Max i32 value
}

impl BeamTypeId {
    /// Check if this type is an identifier (Pid, Port, or Reference)
    pub fn is_identifier(self) -> bool {
        matches!(self, BeamTypeId::Pid | BeamTypeId::Port | BeamTypeId::Reference)
    }

    /// Check if this type is a list (Cons or Nil)
    pub fn is_list(self) -> bool {
        matches!(self, BeamTypeId::Cons | BeamTypeId::Nil)
    }

    /// Check if this type is a number (Float or Integer)
    pub fn is_number(self) -> bool {
        matches!(self, BeamTypeId::Float | BeamTypeId::Integer)
    }

    /// Check if this type can be boxed
    pub fn maybe_boxed(self) -> bool {
        matches!(
            self,
            BeamTypeId::Bitstring
                | BeamTypeId::Float
                | BeamTypeId::Fun
                | BeamTypeId::Integer
                | BeamTypeId::Map
                | BeamTypeId::Pid
                | BeamTypeId::Port
                | BeamTypeId::Reference
                | BeamTypeId::Tuple
        )
    }

    /// Check if this type can be immediate
    pub fn maybe_immediate(self) -> bool {
        matches!(
            self,
            BeamTypeId::Atom | BeamTypeId::Integer | BeamTypeId::Nil | BeamTypeId::Pid | BeamTypeId::Port
        )
    }

    /// Check if this type is always boxed
    pub fn always_boxed(self) -> bool {
        self.maybe_boxed() && !self.maybe_immediate()
    }

    /// Check if this type is always immediate
    pub fn always_immediate(self) -> bool {
        self.maybe_immediate() && !self.maybe_boxed()
    }
}

impl std::ops::BitOr for BeamTypeId {
    type Output = i32;

    fn bitor(self, rhs: Self) -> Self::Output {
        (self as i32) | (rhs as i32)
    }
}

impl std::ops::BitAnd for BeamTypeId {
    type Output = i32;

    fn bitand(self, rhs: Self) -> Self::Output {
        (self as i32) & (rhs as i32)
    }
}

/// Type-safe wrapper around BeamType with additional metadata
///
/// Converted from C++ `struct BeamArgType`.
#[derive(Debug, Clone, Copy)]
pub struct BeamArgType {
    inner: BeamType,
}

impl BeamArgType {
    /// Create a new BeamArgType from a BeamType
    pub fn new(inner: BeamType) -> Self {
        Self { inner }
    }

    /// Get the type ID
    pub fn type_id(&self) -> BeamTypeId {
        // Convert from BeamType to BeamTypeId
        // This would need to match the actual BeamType structure
        BeamTypeId::None // Placeholder - needs actual conversion
    }

    /// Check if this type has a lower bound
    pub fn has_lower_bound(&self) -> bool {
        // Check metadata_flags for BEAM_TYPE_HAS_LOWER_BOUND
        false // Placeholder
    }

    /// Check if this type has an upper bound
    pub fn has_upper_bound(&self) -> bool {
        // Check metadata_flags for BEAM_TYPE_HAS_UPPER_BOUND
        false // Placeholder
    }

    /// Check if this type has a unit
    pub fn has_unit(&self) -> bool {
        // Check metadata_flags for BEAM_TYPE_HAS_UNIT
        false // Placeholder
    }

    /// Get the maximum value (if has_upper_bound)
    pub fn max(&self) -> Option<i64> {
        if self.has_upper_bound() {
            Some(self.inner.max)
        } else {
            None
        }
    }

    /// Get the minimum value (if has_lower_bound)
    pub fn min(&self) -> Option<i64> {
        if self.has_lower_bound() {
            Some(self.inner.min)
        } else {
            None
        }
    }

    /// Get the unit (if has_unit)
    pub fn unit(&self) -> Option<u32> {
        if self.has_unit() {
            Some(self.inner.size_unit)
        } else {
            None
        }
    }

    /// Get the inner BeamType
    pub fn inner(&self) -> &BeamType {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== BeamType Tests ====================

    #[test]
    fn test_beam_type_creation() {
        let bt = BeamType {
            type_union: 1,
            metadata_flags: 0,
            max: 100,
            min: -100,
            size_unit: 8,
        };
        assert_eq!(bt.type_union, 1);
        assert_eq!(bt.metadata_flags, 0);
        assert_eq!(bt.max, 100);
        assert_eq!(bt.min, -100);
        assert_eq!(bt.size_unit, 8);
    }

    #[test]
    fn test_beam_type_debug() {
        let bt = BeamType {
            type_union: 32,
            metadata_flags: 1,
            max: i64::MAX,
            min: i64::MIN,
            size_unit: 1,
        };
        let debug = format!("{:?}", bt);
        assert!(debug.contains("BeamType"));
        assert!(debug.contains("32"));
    }

    #[test]
    fn test_beam_type_clone() {
        let bt = BeamType {
            type_union: 42,
            metadata_flags: 0xFF,
            max: 1000,
            min: -1000,
            size_unit: 64,
        };
        let cloned = bt.clone();
        assert_eq!(bt.type_union, cloned.type_union);
        assert_eq!(bt.metadata_flags, cloned.metadata_flags);
        assert_eq!(bt.max, cloned.max);
        assert_eq!(bt.min, cloned.min);
        assert_eq!(bt.size_unit, cloned.size_unit);
    }

    #[test]
    fn test_beam_type_copy() {
        let bt = BeamType {
            type_union: 1,
            metadata_flags: 2,
            max: 3,
            min: 4,
            size_unit: 5,
        };
        let copied = bt;
        assert_eq!(bt.type_union, copied.type_union);
    }

    #[test]
    fn test_beam_type_extreme_values() {
        let bt = BeamType {
            type_union: i32::MAX,
            metadata_flags: u32::MAX,
            max: i64::MAX,
            min: i64::MIN,
            size_unit: u32::MAX,
        };
        assert_eq!(bt.type_union, i32::MAX);
        assert_eq!(bt.metadata_flags, u32::MAX);
        assert_eq!(bt.max, i64::MAX);
        assert_eq!(bt.min, i64::MIN);
        assert_eq!(bt.size_unit, u32::MAX);
    }

    #[test]
    fn test_beam_type_zero_values() {
        let bt = BeamType {
            type_union: 0,
            metadata_flags: 0,
            max: 0,
            min: 0,
            size_unit: 0,
        };
        assert_eq!(bt.type_union, 0);
        assert_eq!(bt.max, 0);
        assert_eq!(bt.min, 0);
    }

    // ==================== BeamTypeId Tests ====================

    #[test]
    fn test_beam_type_id_values() {
        assert_eq!(BeamTypeId::None as i32, 0);
        assert_eq!(BeamTypeId::Atom as i32, 1);
        assert_eq!(BeamTypeId::Bitstring as i32, 2);
        assert_eq!(BeamTypeId::Cons as i32, 4);
        assert_eq!(BeamTypeId::Float as i32, 8);
        assert_eq!(BeamTypeId::Fun as i32, 16);
        assert_eq!(BeamTypeId::Integer as i32, 32);
        assert_eq!(BeamTypeId::Map as i32, 64);
        assert_eq!(BeamTypeId::Nil as i32, 128);
        assert_eq!(BeamTypeId::Pid as i32, 256);
        assert_eq!(BeamTypeId::Port as i32, 512);
        assert_eq!(BeamTypeId::Reference as i32, 1024);
        assert_eq!(BeamTypeId::Tuple as i32, 2048);
        assert_eq!(BeamTypeId::Any as i32, 0x7FFFFFFF);
    }

    #[test]
    fn test_beam_type_id_debug() {
        assert_eq!(format!("{:?}", BeamTypeId::Atom), "Atom");
        assert_eq!(format!("{:?}", BeamTypeId::Integer), "Integer");
        assert_eq!(format!("{:?}", BeamTypeId::None), "None");
    }

    #[test]
    fn test_beam_type_id_clone() {
        let id = BeamTypeId::Float;
        let cloned = id.clone();
        assert_eq!(id, cloned);
    }

    #[test]
    fn test_beam_type_id_copy() {
        let id = BeamTypeId::Map;
        let copied = id;
        assert_eq!(id, copied);
    }

    #[test]
    fn test_beam_type_id_eq() {
        assert_eq!(BeamTypeId::Atom, BeamTypeId::Atom);
        assert_ne!(BeamTypeId::Atom, BeamTypeId::Integer);
    }

    // ==================== BeamTypeId::is_identifier Tests ====================

    #[test]
    fn test_is_identifier_pid() {
        assert!(BeamTypeId::Pid.is_identifier());
    }

    #[test]
    fn test_is_identifier_port() {
        assert!(BeamTypeId::Port.is_identifier());
    }

    #[test]
    fn test_is_identifier_reference() {
        assert!(BeamTypeId::Reference.is_identifier());
    }

    #[test]
    fn test_is_identifier_non_identifiers() {
        assert!(!BeamTypeId::None.is_identifier());
        assert!(!BeamTypeId::Atom.is_identifier());
        assert!(!BeamTypeId::Bitstring.is_identifier());
        assert!(!BeamTypeId::Cons.is_identifier());
        assert!(!BeamTypeId::Float.is_identifier());
        assert!(!BeamTypeId::Fun.is_identifier());
        assert!(!BeamTypeId::Integer.is_identifier());
        assert!(!BeamTypeId::Map.is_identifier());
        assert!(!BeamTypeId::Nil.is_identifier());
        assert!(!BeamTypeId::Tuple.is_identifier());
        assert!(!BeamTypeId::Any.is_identifier());
    }

    // ==================== BeamTypeId::is_list Tests ====================

    #[test]
    fn test_is_list_cons() {
        assert!(BeamTypeId::Cons.is_list());
    }

    #[test]
    fn test_is_list_nil() {
        assert!(BeamTypeId::Nil.is_list());
    }

    #[test]
    fn test_is_list_non_lists() {
        assert!(!BeamTypeId::None.is_list());
        assert!(!BeamTypeId::Atom.is_list());
        assert!(!BeamTypeId::Bitstring.is_list());
        assert!(!BeamTypeId::Float.is_list());
        assert!(!BeamTypeId::Fun.is_list());
        assert!(!BeamTypeId::Integer.is_list());
        assert!(!BeamTypeId::Map.is_list());
        assert!(!BeamTypeId::Pid.is_list());
        assert!(!BeamTypeId::Port.is_list());
        assert!(!BeamTypeId::Reference.is_list());
        assert!(!BeamTypeId::Tuple.is_list());
        assert!(!BeamTypeId::Any.is_list());
    }

    // ==================== BeamTypeId::is_number Tests ====================

    #[test]
    fn test_is_number_float() {
        assert!(BeamTypeId::Float.is_number());
    }

    #[test]
    fn test_is_number_integer() {
        assert!(BeamTypeId::Integer.is_number());
    }

    #[test]
    fn test_is_number_non_numbers() {
        assert!(!BeamTypeId::None.is_number());
        assert!(!BeamTypeId::Atom.is_number());
        assert!(!BeamTypeId::Bitstring.is_number());
        assert!(!BeamTypeId::Cons.is_number());
        assert!(!BeamTypeId::Fun.is_number());
        assert!(!BeamTypeId::Map.is_number());
        assert!(!BeamTypeId::Nil.is_number());
        assert!(!BeamTypeId::Pid.is_number());
        assert!(!BeamTypeId::Port.is_number());
        assert!(!BeamTypeId::Reference.is_number());
        assert!(!BeamTypeId::Tuple.is_number());
        assert!(!BeamTypeId::Any.is_number());
    }

    // ==================== BeamTypeId::maybe_boxed Tests ====================

    #[test]
    fn test_maybe_boxed_types() {
        assert!(BeamTypeId::Bitstring.maybe_boxed());
        assert!(BeamTypeId::Float.maybe_boxed());
        assert!(BeamTypeId::Fun.maybe_boxed());
        assert!(BeamTypeId::Integer.maybe_boxed());
        assert!(BeamTypeId::Map.maybe_boxed());
        assert!(BeamTypeId::Pid.maybe_boxed());
        assert!(BeamTypeId::Port.maybe_boxed());
        assert!(BeamTypeId::Reference.maybe_boxed());
        assert!(BeamTypeId::Tuple.maybe_boxed());
    }

    #[test]
    fn test_maybe_boxed_non_boxed() {
        assert!(!BeamTypeId::None.maybe_boxed());
        assert!(!BeamTypeId::Atom.maybe_boxed());
        assert!(!BeamTypeId::Cons.maybe_boxed());
        assert!(!BeamTypeId::Nil.maybe_boxed());
        assert!(!BeamTypeId::Any.maybe_boxed());
    }

    // ==================== BeamTypeId::maybe_immediate Tests ====================

    #[test]
    fn test_maybe_immediate_types() {
        assert!(BeamTypeId::Atom.maybe_immediate());
        assert!(BeamTypeId::Integer.maybe_immediate());
        assert!(BeamTypeId::Nil.maybe_immediate());
        assert!(BeamTypeId::Pid.maybe_immediate());
        assert!(BeamTypeId::Port.maybe_immediate());
    }

    #[test]
    fn test_maybe_immediate_non_immediate() {
        assert!(!BeamTypeId::None.maybe_immediate());
        assert!(!BeamTypeId::Bitstring.maybe_immediate());
        assert!(!BeamTypeId::Cons.maybe_immediate());
        assert!(!BeamTypeId::Float.maybe_immediate());
        assert!(!BeamTypeId::Fun.maybe_immediate());
        assert!(!BeamTypeId::Map.maybe_immediate());
        assert!(!BeamTypeId::Reference.maybe_immediate());
        assert!(!BeamTypeId::Tuple.maybe_immediate());
        assert!(!BeamTypeId::Any.maybe_immediate());
    }

    // ==================== BeamTypeId::always_boxed Tests ====================

    #[test]
    fn test_always_boxed() {
        // Types that are maybe_boxed but NOT maybe_immediate
        assert!(BeamTypeId::Bitstring.always_boxed());
        assert!(BeamTypeId::Float.always_boxed());
        assert!(BeamTypeId::Fun.always_boxed());
        assert!(BeamTypeId::Map.always_boxed());
        assert!(BeamTypeId::Reference.always_boxed());
        assert!(BeamTypeId::Tuple.always_boxed());
    }

    #[test]
    fn test_not_always_boxed() {
        // Integer, Pid, Port are both maybe_boxed and maybe_immediate
        assert!(!BeamTypeId::Integer.always_boxed());
        assert!(!BeamTypeId::Pid.always_boxed());
        assert!(!BeamTypeId::Port.always_boxed());
        
        // These are never boxed
        assert!(!BeamTypeId::None.always_boxed());
        assert!(!BeamTypeId::Atom.always_boxed());
        assert!(!BeamTypeId::Cons.always_boxed());
        assert!(!BeamTypeId::Nil.always_boxed());
    }

    // ==================== BeamTypeId::always_immediate Tests ====================

    #[test]
    fn test_always_immediate() {
        // Types that are maybe_immediate but NOT maybe_boxed
        assert!(BeamTypeId::Atom.always_immediate());
        assert!(BeamTypeId::Nil.always_immediate());
    }

    #[test]
    fn test_not_always_immediate() {
        // Integer, Pid, Port are both maybe_boxed and maybe_immediate
        assert!(!BeamTypeId::Integer.always_immediate());
        assert!(!BeamTypeId::Pid.always_immediate());
        assert!(!BeamTypeId::Port.always_immediate());
        
        // These are never immediate
        assert!(!BeamTypeId::None.always_immediate());
        assert!(!BeamTypeId::Bitstring.always_immediate());
        assert!(!BeamTypeId::Float.always_immediate());
    }

    // ==================== BitOr Tests ====================

    #[test]
    fn test_bitor_basic() {
        let result = BeamTypeId::Atom | BeamTypeId::Integer;
        assert_eq!(result, 1 | 32);
        assert_eq!(result, 33);
    }

    #[test]
    fn test_bitor_same_type() {
        let result = BeamTypeId::Atom | BeamTypeId::Atom;
        assert_eq!(result, 1);
    }

    #[test]
    fn test_bitor_multiple() {
        let result = BeamTypeId::Cons | BeamTypeId::Nil;
        assert_eq!(result, 4 | 128);
        assert_eq!(result, 132);
    }

    #[test]
    fn test_bitor_all_numbers() {
        let result = BeamTypeId::Float | BeamTypeId::Integer;
        assert_eq!(result, 8 | 32);
        assert_eq!(result, 40);
    }

    #[test]
    fn test_bitor_with_any() {
        let result = BeamTypeId::Atom | BeamTypeId::Any;
        assert_eq!(result, BeamTypeId::Any as i32);
    }

    // ==================== BitAnd Tests ====================

    #[test]
    fn test_bitand_basic() {
        let result = BeamTypeId::Atom & BeamTypeId::Integer;
        assert_eq!(result, 1 & 32);
        assert_eq!(result, 0);
    }

    #[test]
    fn test_bitand_same_type() {
        let result = BeamTypeId::Atom & BeamTypeId::Atom;
        assert_eq!(result, 1);
    }

    #[test]
    fn test_bitand_with_any() {
        let result = BeamTypeId::Atom & BeamTypeId::Any;
        assert_eq!(result, 1);
    }

    #[test]
    fn test_bitand_with_none() {
        let result = BeamTypeId::Atom & BeamTypeId::None;
        assert_eq!(result, 0);
    }

    // ==================== BeamArgType Tests ====================

    #[test]
    fn test_beam_arg_type_new() {
        let bt = BeamType {
            type_union: 32,
            metadata_flags: 0,
            max: 100,
            min: -100,
            size_unit: 8,
        };
        let arg = BeamArgType::new(bt);
        assert_eq!(arg.inner().type_union, 32);
    }

    #[test]
    fn test_beam_arg_type_debug() {
        let bt = BeamType {
            type_union: 1,
            metadata_flags: 2,
            max: 3,
            min: 4,
            size_unit: 5,
        };
        let arg = BeamArgType::new(bt);
        let debug = format!("{:?}", arg);
        assert!(debug.contains("BeamArgType"));
    }

    #[test]
    fn test_beam_arg_type_clone() {
        let bt = BeamType {
            type_union: 64,
            metadata_flags: 0xFF,
            max: 1000,
            min: -1000,
            size_unit: 16,
        };
        let arg = BeamArgType::new(bt);
        let cloned = arg.clone();
        assert_eq!(arg.inner().type_union, cloned.inner().type_union);
    }

    #[test]
    fn test_beam_arg_type_copy() {
        let bt = BeamType {
            type_union: 128,
            metadata_flags: 0,
            max: 0,
            min: 0,
            size_unit: 1,
        };
        let arg = BeamArgType::new(bt);
        let copied = arg;
        assert_eq!(arg.inner().type_union, copied.inner().type_union);
    }

    #[test]
    fn test_beam_arg_type_type_id() {
        let bt = BeamType {
            type_union: 32,
            metadata_flags: 0,
            max: 0,
            min: 0,
            size_unit: 0,
        };
        let arg = BeamArgType::new(bt);
        // Currently returns None as placeholder
        assert_eq!(arg.type_id(), BeamTypeId::None);
    }

    #[test]
    fn test_beam_arg_type_has_lower_bound() {
        let bt = BeamType {
            type_union: 32,
            metadata_flags: 0,
            max: 100,
            min: -100,
            size_unit: 0,
        };
        let arg = BeamArgType::new(bt);
        // Currently returns false as placeholder
        assert!(!arg.has_lower_bound());
    }

    #[test]
    fn test_beam_arg_type_has_upper_bound() {
        let bt = BeamType {
            type_union: 32,
            metadata_flags: 0,
            max: 100,
            min: -100,
            size_unit: 0,
        };
        let arg = BeamArgType::new(bt);
        // Currently returns false as placeholder
        assert!(!arg.has_upper_bound());
    }

    #[test]
    fn test_beam_arg_type_has_unit() {
        let bt = BeamType {
            type_union: 2,
            metadata_flags: 0,
            max: 0,
            min: 0,
            size_unit: 8,
        };
        let arg = BeamArgType::new(bt);
        // Currently returns false as placeholder
        assert!(!arg.has_unit());
    }

    #[test]
    fn test_beam_arg_type_max() {
        let bt = BeamType {
            type_union: 32,
            metadata_flags: 0,
            max: 1000,
            min: -1000,
            size_unit: 0,
        };
        let arg = BeamArgType::new(bt);
        // Returns None because has_upper_bound is false (placeholder)
        assert_eq!(arg.max(), None);
    }

    #[test]
    fn test_beam_arg_type_min() {
        let bt = BeamType {
            type_union: 32,
            metadata_flags: 0,
            max: 1000,
            min: -1000,
            size_unit: 0,
        };
        let arg = BeamArgType::new(bt);
        // Returns None because has_lower_bound is false (placeholder)
        assert_eq!(arg.min(), None);
    }

    #[test]
    fn test_beam_arg_type_unit() {
        let bt = BeamType {
            type_union: 2,
            metadata_flags: 0,
            max: 0,
            min: 0,
            size_unit: 8,
        };
        let arg = BeamArgType::new(bt);
        // Returns None because has_unit is false (placeholder)
        assert_eq!(arg.unit(), None);
    }

    #[test]
    fn test_beam_arg_type_inner() {
        let bt = BeamType {
            type_union: 42,
            metadata_flags: 0xABCD,
            max: 999,
            min: -999,
            size_unit: 64,
        };
        let arg = BeamArgType::new(bt);
        let inner = arg.inner();
        assert_eq!(inner.type_union, 42);
        assert_eq!(inner.metadata_flags, 0xABCD);
        assert_eq!(inner.max, 999);
        assert_eq!(inner.min, -999);
        assert_eq!(inner.size_unit, 64);
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_all_type_ids_are_powers_of_two_except_none_and_any() {
        let type_ids = [
            (BeamTypeId::Atom, 1u32),
            (BeamTypeId::Bitstring, 2u32),
            (BeamTypeId::Cons, 4u32),
            (BeamTypeId::Float, 8u32),
            (BeamTypeId::Fun, 16u32),
            (BeamTypeId::Integer, 32u32),
            (BeamTypeId::Map, 64u32),
            (BeamTypeId::Nil, 128u32),
            (BeamTypeId::Pid, 256u32),
            (BeamTypeId::Port, 512u32),
            (BeamTypeId::Reference, 1024u32),
            (BeamTypeId::Tuple, 2048u32),
        ];
        
        for (id, expected) in type_ids {
            assert_eq!(id as u32, expected);
            // All should be powers of two
            assert!(expected.is_power_of_two());
        }
    }

    #[test]
    fn test_type_classification_consistency() {
        // If always_boxed, then maybe_boxed must be true
        for id in [
            BeamTypeId::Bitstring,
            BeamTypeId::Float,
            BeamTypeId::Fun,
            BeamTypeId::Map,
            BeamTypeId::Reference,
            BeamTypeId::Tuple,
        ] {
            if id.always_boxed() {
                assert!(id.maybe_boxed());
            }
        }
    }

    #[test]
    fn test_type_classification_mutual_exclusion() {
        // always_boxed and always_immediate should be mutually exclusive
        for id in [
            BeamTypeId::None,
            BeamTypeId::Atom,
            BeamTypeId::Bitstring,
            BeamTypeId::Cons,
            BeamTypeId::Float,
            BeamTypeId::Fun,
            BeamTypeId::Integer,
            BeamTypeId::Map,
            BeamTypeId::Nil,
            BeamTypeId::Pid,
            BeamTypeId::Port,
            BeamTypeId::Reference,
            BeamTypeId::Tuple,
            BeamTypeId::Any,
        ] {
            assert!(!(id.always_boxed() && id.always_immediate()));
        }
    }

    #[test]
    fn test_bitor_creates_type_union() {
        // Create a type union representing "list or nil"
        let list_type = BeamTypeId::Cons | BeamTypeId::Nil;
        assert_eq!(list_type, 4 | 128);
        
        // Create a type union representing "number"
        let number_type = BeamTypeId::Integer | BeamTypeId::Float;
        assert_eq!(number_type, 32 | 8);
    }

    #[test]
    fn test_beam_arg_type_with_all_field_values() {
        let bt = BeamType {
            type_union: i32::MIN,
            metadata_flags: u32::MAX,
            max: i64::MAX,
            min: i64::MIN,
            size_unit: u32::MAX,
        };
        let arg = BeamArgType::new(bt);
        let inner = arg.inner();
        assert_eq!(inner.type_union, i32::MIN);
        assert_eq!(inner.metadata_flags, u32::MAX);
        assert_eq!(inner.max, i64::MAX);
        assert_eq!(inner.min, i64::MIN);
        assert_eq!(inner.size_unit, u32::MAX);
    }
}

