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
}

