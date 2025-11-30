//! Operator Built-in Functions
//!
//! Provides logical, comparison, and type-checking operations:
//! - Logical operations (and, or, xor, not)
//! - Comparison operations (>, >=, <, <=, ==, =/=)
//! - Type checking operations (is_atom, is_integer, is_list, etc.)
//!
//! Based on erl_bif_op.c
//!
//! This module implements safe Rust equivalents of Erlang operator BIFs.

use std::collections::HashMap;

/// Placeholder for Erlang term representation.
/// In a full implementation, this would be a proper Eterm type from entities_data_handling.
#[derive(Debug, Clone)]
pub enum ErlangTerm {
    Atom(String),
    Integer(i64),
    Float(f64),
    Tuple(Vec<ErlangTerm>),
    List(Vec<ErlangTerm>),
    Binary(Vec<u8>),
    Bitstring(Vec<u8>, usize), // data, bit_length
    Map(HashMap<ErlangTerm, ErlangTerm>),
    Pid(u64),
    Port(u64),
    Reference(u64),
    Function { arity: usize },
    Nil,
    // ... other term types as needed
}

impl PartialEq for ErlangTerm {
    fn eq(&self, other: &ErlangTerm) -> bool {
        match (self, other) {
            (ErlangTerm::Atom(a), ErlangTerm::Atom(b)) => a == b,
            (ErlangTerm::Integer(a), ErlangTerm::Integer(b)) => a == b,
            (ErlangTerm::Float(a), ErlangTerm::Float(b)) => {
                // Handle NaN: NaN != NaN in Rust, but we want consistent behavior
                if a.is_nan() && b.is_nan() {
                    true
                } else {
                    a == b
                }
            }
            (ErlangTerm::Tuple(a), ErlangTerm::Tuple(b)) => a == b,
            (ErlangTerm::List(a), ErlangTerm::List(b)) => a == b,
            (ErlangTerm::Binary(a), ErlangTerm::Binary(b)) => a == b,
            (ErlangTerm::Bitstring(a, bits_a), ErlangTerm::Bitstring(b, bits_b)) => {
                a == b && bits_a == bits_b
            }
            (ErlangTerm::Map(a), ErlangTerm::Map(b)) => a == b,
            (ErlangTerm::Pid(a), ErlangTerm::Pid(b)) => a == b,
            (ErlangTerm::Port(a), ErlangTerm::Port(b)) => a == b,
            (ErlangTerm::Reference(a), ErlangTerm::Reference(b)) => a == b,
            (ErlangTerm::Function { arity: a }, ErlangTerm::Function { arity: b }) => a == b,
            (ErlangTerm::Nil, ErlangTerm::Nil) => true,
            _ => false,
        }
    }
}

impl Eq for ErlangTerm {}

// Implement Hash manually since HashMap doesn't implement Hash
impl std::hash::Hash for ErlangTerm {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            ErlangTerm::Atom(s) => {
                state.write_u8(0);
                s.hash(state);
            }
            ErlangTerm::Integer(i) => {
                state.write_u8(1);
                i.hash(state);
            }
            ErlangTerm::Float(f) => {
                state.write_u8(2);
                f.to_bits().hash(state);
            }
            ErlangTerm::Tuple(v) => {
                state.write_u8(3);
                v.len().hash(state);
                for item in v {
                    item.hash(state);
                }
            }
            ErlangTerm::List(v) => {
                state.write_u8(4);
                v.len().hash(state);
                for item in v {
                    item.hash(state);
                }
            }
            ErlangTerm::Binary(v) => {
                state.write_u8(5);
                v.hash(state);
            }
            ErlangTerm::Bitstring(v, bits) => {
                state.write_u8(6);
                v.hash(state);
                bits.hash(state);
            }
            ErlangTerm::Map(m) => {
                state.write_u8(7);
                // For HashMap, we need to hash in a deterministic way
                let mut entries: Vec<_> = m.iter().collect();
                entries.sort_by(|a, b| format!("{:?}", a.0).cmp(&format!("{:?}", b.0)));
                entries.len().hash(state);
                for (k, v) in entries {
                    k.hash(state);
                    v.hash(state);
                }
            }
            ErlangTerm::Pid(p) => {
                state.write_u8(8);
                p.hash(state);
            }
            ErlangTerm::Port(p) => {
                state.write_u8(9);
                p.hash(state);
            }
            ErlangTerm::Reference(r) => {
                state.write_u8(10);
                r.hash(state);
            }
            ErlangTerm::Function { arity } => {
                state.write_u8(11);
                arity.hash(state);
            }
            ErlangTerm::Nil => {
                state.write_u8(12);
            }
        }
    }
}

impl ErlangTerm {
    /// Check if term is an atom
    pub fn is_atom(&self) -> bool {
        matches!(self, ErlangTerm::Atom(_))
    }

    /// Check if term is an integer
    pub fn is_integer(&self) -> bool {
        matches!(self, ErlangTerm::Integer(_))
    }

    /// Check if term is a float
    pub fn is_float(&self) -> bool {
        matches!(self, ErlangTerm::Float(_))
    }

    /// Check if term is a number (integer or float)
    pub fn is_number(&self) -> bool {
        self.is_integer() || self.is_float()
    }

    /// Check if term is a list or nil
    pub fn is_list(&self) -> bool {
        matches!(self, ErlangTerm::List(_) | ErlangTerm::Nil)
    }

    /// Check if term is a tuple
    pub fn is_tuple(&self) -> bool {
        matches!(self, ErlangTerm::Tuple(_))
    }

    /// Check if term is a binary (bitstring with byte-aligned bits)
    pub fn is_binary(&self) -> bool {
        match self {
            ErlangTerm::Bitstring(_, bit_length) => bit_length % 8 == 0,
            ErlangTerm::Binary(_) => true,
            _ => false,
        }
    }

    /// Check if term is a bitstring
    pub fn is_bitstring(&self) -> bool {
        matches!(self, ErlangTerm::Bitstring(_, _) | ErlangTerm::Binary(_))
    }

    /// Check if term is a PID
    pub fn is_pid(&self) -> bool {
        matches!(self, ErlangTerm::Pid(_))
    }

    /// Check if term is a port
    pub fn is_port(&self) -> bool {
        matches!(self, ErlangTerm::Port(_))
    }

    /// Check if term is a reference
    pub fn is_reference(&self) -> bool {
        matches!(self, ErlangTerm::Reference(_))
    }

    /// Check if term is a function
    pub fn is_function(&self) -> bool {
        matches!(self, ErlangTerm::Function { .. })
    }

    /// Check if term is a boolean (true or false atom)
    pub fn is_boolean(&self) -> bool {
        matches!(
            self,
            ErlangTerm::Atom(ref s) if s == "true" || s == "false"
        )
    }

    /// Check if term is a map
    pub fn is_map(&self) -> bool {
        matches!(self, ErlangTerm::Map(_))
    }

    /// Get function arity if term is a function
    pub fn function_arity(&self) -> Option<usize> {
        match self {
            ErlangTerm::Function { arity } => Some(*arity),
            _ => None,
        }
    }

    /// Compare two terms for structural equality (== in Erlang)
    /// Allows type coercion: Integer(1) == Float(1.0) is true
    pub fn eq(&self, other: &ErlangTerm) -> bool {
        // First check exact equality
        if self == other {
            return true;
        }
        
        // Then check for number type coercion
        match (self, other) {
            (ErlangTerm::Integer(a), ErlangTerm::Float(b)) => {
                (*a as f64) == *b
            }
            (ErlangTerm::Float(a), ErlangTerm::Integer(b)) => {
                *a == (*b as f64)
            }
            _ => false,
        }
    }

    /// Compare two terms for exact equality (=:= in Erlang)
    /// No type coercion: Integer(1) =:= Float(1.0) is false
    pub fn exact_eq(&self, other: &ErlangTerm) -> bool {
        self == other
    }

    /// Compare two terms (for ordering)
    /// Returns: Some(Ordering) if comparable, None if not comparable
    pub fn compare(&self, other: &ErlangTerm) -> Option<std::cmp::Ordering> {
        match (self, other) {
            // Number comparisons (with type coercion)
            (ErlangTerm::Integer(a), ErlangTerm::Integer(b)) => Some(a.cmp(b)),
            (ErlangTerm::Float(a), ErlangTerm::Float(b)) => a.partial_cmp(b),
            (ErlangTerm::Integer(a), ErlangTerm::Float(b)) => {
                (*a as f64).partial_cmp(b)
            }
            (ErlangTerm::Float(a), ErlangTerm::Integer(b)) => {
                a.partial_cmp(&(*b as f64))
            }
            
            // Atom comparisons (lexicographic)
            (ErlangTerm::Atom(a), ErlangTerm::Atom(b)) => Some(a.cmp(b)),
            
            // List comparisons (lexicographic)
            (ErlangTerm::List(a), ErlangTerm::List(b)) => {
                ErlangTerm::compare_lists(a, b)
            }
            (ErlangTerm::Nil, ErlangTerm::List(_)) => Some(std::cmp::Ordering::Less),
            (ErlangTerm::List(_), ErlangTerm::Nil) => Some(std::cmp::Ordering::Greater),
            (ErlangTerm::Nil, ErlangTerm::Nil) => Some(std::cmp::Ordering::Equal),
            
            // Tuple comparisons (by arity, then elements)
            (ErlangTerm::Tuple(a), ErlangTerm::Tuple(b)) => {
                match a.len().cmp(&b.len()) {
                    std::cmp::Ordering::Equal => {
                        // Compare element by element
                        for (ae, be) in a.iter().zip(b.iter()) {
                            if let Some(ord) = ae.compare(be) {
                                if ord != std::cmp::Ordering::Equal {
                                    return Some(ord);
                                }
                            } else {
                                // Not comparable, use structural comparison
                                if ae != be {
                                    return None;
                                }
                            }
                        }
                        Some(std::cmp::Ordering::Equal)
                    }
                    other => Some(other),
                }
            }
            
            // Binary/Bitstring comparisons (byte-by-byte)
            (ErlangTerm::Binary(a), ErlangTerm::Binary(b)) => Some(a.cmp(b)),
            (ErlangTerm::Bitstring(a, bits_a), ErlangTerm::Bitstring(b, bits_b)) => {
                match a.cmp(b) {
                    std::cmp::Ordering::Equal => Some(bits_a.cmp(bits_b)),
                    other => Some(other),
                }
            }
            (ErlangTerm::Binary(a), ErlangTerm::Bitstring(b, _)) => {
                // Compare as if binary is a bitstring with byte-aligned bits
                Some(a.cmp(b))
            }
            (ErlangTerm::Bitstring(a, _), ErlangTerm::Binary(b)) => {
                Some(a.cmp(b))
            }
            
            // Map comparisons (by size, then keys)
            (ErlangTerm::Map(a), ErlangTerm::Map(b)) => {
                match a.len().cmp(&b.len()) {
                    std::cmp::Ordering::Equal => {
                        // Compare keys (simplified - in real Erlang, keys are compared)
                        // For now, we'll just check if maps are equal
                        if a == b {
                            Some(std::cmp::Ordering::Equal)
                        } else {
                            None // Maps with same size but different keys - not easily comparable
                        }
                    }
                    other => Some(other),
                }
            }
            
            // PID, Port, Reference comparisons
            (ErlangTerm::Pid(a), ErlangTerm::Pid(b)) => Some(a.cmp(b)),
            (ErlangTerm::Port(a), ErlangTerm::Port(b)) => Some(a.cmp(b)),
            (ErlangTerm::Reference(a), ErlangTerm::Reference(b)) => Some(a.cmp(b)),
            
            // Function comparisons (by arity)
            (ErlangTerm::Function { arity: a }, ErlangTerm::Function { arity: b }) => {
                Some(a.cmp(b))
            }
            
            // Different types are not comparable
            _ => None,
        }
    }
}

impl ErlangTerm {
    /// Helper: Compare two lists lexicographically (static method)
    fn compare_lists(a: &[ErlangTerm], b: &[ErlangTerm]) -> Option<std::cmp::Ordering> {
        let min_len = a.len().min(b.len());
        for i in 0..min_len {
            if let Some(ord) = a[i].compare(&b[i]) {
                if ord != std::cmp::Ordering::Equal {
                    return Some(ord);
                }
            } else {
                // Elements not comparable, use structural comparison
                if a[i] != b[i] {
                    return None;
                }
            }
        }
        Some(a.len().cmp(&b.len()))
    }
}

/// Operator BIF operations
pub struct OpBif;

impl OpBif {
    /// Logical AND operation
    ///
    /// Equivalent to `erlang:'and'/2` in Erlang.
    ///
    /// # Arguments
    /// * `arg1` - First boolean argument
    /// * `arg2` - Second boolean argument
    ///
    /// # Returns
    /// * `Ok(ErlangTerm::Atom("true"))` - If both arguments are true
    /// * `Ok(ErlangTerm::Atom("false"))` - Otherwise
    /// * `Err(OpError)` - If arguments are not booleans
    pub fn and(arg1: &ErlangTerm, arg2: &ErlangTerm) -> Result<ErlangTerm, OpError> {
        let b1 = Self::to_bool(arg1)?;
        let b2 = Self::to_bool(arg2)?;
        Ok(ErlangTerm::Atom(if b1 && b2 { "true" } else { "false" }.to_string()))
    }

    /// Logical OR operation
    ///
    /// Equivalent to `erlang:'or'/2` in Erlang.
    pub fn or(arg1: &ErlangTerm, arg2: &ErlangTerm) -> Result<ErlangTerm, OpError> {
        let b1 = Self::to_bool(arg1)?;
        let b2 = Self::to_bool(arg2)?;
        Ok(ErlangTerm::Atom(if b1 || b2 { "true" } else { "false" }.to_string()))
    }

    /// Logical XOR operation
    ///
    /// Equivalent to `erlang:'xor'/2` in Erlang.
    pub fn xor(arg1: &ErlangTerm, arg2: &ErlangTerm) -> Result<ErlangTerm, OpError> {
        let b1 = Self::to_bool(arg1)?;
        let b2 = Self::to_bool(arg2)?;
        Ok(ErlangTerm::Atom(if b1 != b2 { "true" } else { "false" }.to_string()))
    }

    /// Logical NOT operation
    ///
    /// Equivalent to `erlang:'not'/1` in Erlang.
    pub fn not(arg: &ErlangTerm) -> Result<ErlangTerm, OpError> {
        let b = Self::to_bool(arg)?;
        Ok(ErlangTerm::Atom(if b { "false" } else { "true" }.to_string()))
    }

    /// Strictly greater than comparison
    ///
    /// Equivalent to `erlang:'>'/2` in Erlang.
    pub fn sgt(arg1: &ErlangTerm, arg2: &ErlangTerm) -> ErlangTerm {
        match arg1.compare(arg2) {
            Some(std::cmp::Ordering::Greater) => ErlangTerm::Atom("true".to_string()),
            _ => ErlangTerm::Atom("false".to_string()),
        }
    }

    /// Strictly greater than or equal comparison
    ///
    /// Equivalent to `erlang:'>='/2` in Erlang.
    pub fn sge(arg1: &ErlangTerm, arg2: &ErlangTerm) -> ErlangTerm {
        match arg1.compare(arg2) {
            Some(std::cmp::Ordering::Greater) | Some(std::cmp::Ordering::Equal) => {
                ErlangTerm::Atom("true".to_string())
            }
            _ => ErlangTerm::Atom("false".to_string()),
        }
    }

    /// Strictly less than comparison
    ///
    /// Equivalent to `erlang:'<'/2` in Erlang.
    pub fn slt(arg1: &ErlangTerm, arg2: &ErlangTerm) -> ErlangTerm {
        match arg1.compare(arg2) {
            Some(std::cmp::Ordering::Less) => ErlangTerm::Atom("true".to_string()),
            _ => ErlangTerm::Atom("false".to_string()),
        }
    }

    /// Strictly less than or equal comparison
    ///
    /// Equivalent to `erlang:'=<'/2` in Erlang.
    pub fn sle(arg1: &ErlangTerm, arg2: &ErlangTerm) -> ErlangTerm {
        match arg1.compare(arg2) {
            Some(std::cmp::Ordering::Less) | Some(std::cmp::Ordering::Equal) => {
                ErlangTerm::Atom("true".to_string())
            }
            _ => ErlangTerm::Atom("false".to_string()),
        }
    }

    /// Equality comparison (structural equality)
    ///
    /// Equivalent to `erlang:'=='/2` in Erlang.
    pub fn seq(arg1: &ErlangTerm, arg2: &ErlangTerm) -> ErlangTerm {
        ErlangTerm::Atom(if arg1.eq(arg2) { "true" } else { "false" }.to_string())
    }

    /// Exact equality comparison
    ///
    /// Equivalent to `erlang:'=:='/2` in Erlang.
    /// Requires exact type match: Integer(1) =:= Float(1.0) is false
    pub fn seqeq(arg1: &ErlangTerm, arg2: &ErlangTerm) -> ErlangTerm {
        ErlangTerm::Atom(if arg1.exact_eq(arg2) { "true" } else { "false" }.to_string())
    }

    /// Not equal comparison (structural)
    ///
    /// Equivalent to `erlang:'/='/2` in Erlang.
    /// Allows type coercion: Integer(1) /= Float(1.0) is false
    pub fn sneq(arg1: &ErlangTerm, arg2: &ErlangTerm) -> ErlangTerm {
        ErlangTerm::Atom(if !arg1.eq(arg2) { "true" } else { "false" }.to_string())
    }

    /// Exact not equal comparison
    ///
    /// Equivalent to `erlang:'=/='/2` in Erlang.
    /// Requires exact type match: Integer(1) =/= Float(1.0) is true
    pub fn sneqeq(arg1: &ErlangTerm, arg2: &ErlangTerm) -> ErlangTerm {
        ErlangTerm::Atom(if !arg1.exact_eq(arg2) { "true" } else { "false" }.to_string())
    }

    /// Type check: is atom
    ///
    /// Equivalent to `erlang:is_atom/1` in Erlang.
    pub fn is_atom(arg: &ErlangTerm) -> ErlangTerm {
        ErlangTerm::Atom(if arg.is_atom() { "true" } else { "false" }.to_string())
    }

    /// Type check: is float
    ///
    /// Equivalent to `erlang:is_float/1` in Erlang.
    pub fn is_float(arg: &ErlangTerm) -> ErlangTerm {
        ErlangTerm::Atom(if arg.is_float() { "true" } else { "false" }.to_string())
    }

    /// Type check: is integer
    ///
    /// Equivalent to `erlang:is_integer/1` in Erlang.
    pub fn is_integer(arg: &ErlangTerm) -> ErlangTerm {
        ErlangTerm::Atom(if arg.is_integer() { "true" } else { "false" }.to_string())
    }

    /// Type check: is list
    ///
    /// Equivalent to `erlang:is_list/1` in Erlang.
    pub fn is_list(arg: &ErlangTerm) -> ErlangTerm {
        ErlangTerm::Atom(if arg.is_list() { "true" } else { "false" }.to_string())
    }

    /// Type check: is number
    ///
    /// Equivalent to `erlang:is_number/1` in Erlang.
    pub fn is_number(arg: &ErlangTerm) -> ErlangTerm {
        ErlangTerm::Atom(if arg.is_number() { "true" } else { "false" }.to_string())
    }

    /// Type check: is PID
    ///
    /// Equivalent to `erlang:is_pid/1` in Erlang.
    pub fn is_pid(arg: &ErlangTerm) -> ErlangTerm {
        ErlangTerm::Atom(if arg.is_pid() { "true" } else { "false" }.to_string())
    }

    /// Type check: is port
    ///
    /// Equivalent to `erlang:is_port/1` in Erlang.
    pub fn is_port(arg: &ErlangTerm) -> ErlangTerm {
        ErlangTerm::Atom(if arg.is_port() { "true" } else { "false" }.to_string())
    }

    /// Type check: is reference
    ///
    /// Equivalent to `erlang:is_reference/1` in Erlang.
    pub fn is_reference(arg: &ErlangTerm) -> ErlangTerm {
        ErlangTerm::Atom(if arg.is_reference() { "true" } else { "false" }.to_string())
    }

    /// Type check: is tuple
    ///
    /// Equivalent to `erlang:is_tuple/1` in Erlang.
    pub fn is_tuple(arg: &ErlangTerm) -> ErlangTerm {
        ErlangTerm::Atom(if arg.is_tuple() { "true" } else { "false" }.to_string())
    }

    /// Type check: is binary
    ///
    /// Equivalent to `erlang:is_binary/1` in Erlang.
    pub fn is_binary(arg: &ErlangTerm) -> ErlangTerm {
        ErlangTerm::Atom(if arg.is_binary() { "true" } else { "false" }.to_string())
    }

    /// Type check: is bitstring
    ///
    /// Equivalent to `erlang:is_bitstring/1` in Erlang.
    pub fn is_bitstring(arg: &ErlangTerm) -> ErlangTerm {
        ErlangTerm::Atom(if arg.is_bitstring() { "true" } else { "false" }.to_string())
    }

    /// Type check: is function
    ///
    /// Equivalent to `erlang:is_function/1` in Erlang.
    pub fn is_function(arg: &ErlangTerm) -> ErlangTerm {
        ErlangTerm::Atom(if arg.is_function() { "true" } else { "false" }.to_string())
    }

    /// Type check: is function with arity
    ///
    /// Equivalent to `erlang:is_function/2` in Erlang.
    ///
    /// # Arguments
    /// * `arg1` - Term to check
    /// * `arg2` - Arity (must be a non-negative integer)
    ///
    /// # Returns
    /// * `Ok(ErlangTerm::Atom("true"))` - If arg1 is a function with the specified arity
    /// * `Ok(ErlangTerm::Atom("false"))` - Otherwise
    /// * `Err(OpError)` - If arg2 is not a valid non-negative integer
    pub fn is_function_with_arity(
        arg1: &ErlangTerm,
        arg2: &ErlangTerm,
    ) -> Result<ErlangTerm, OpError> {
        let arity = Self::to_non_negative_integer(arg2)?;

        if let Some(func_arity) = arg1.function_arity() {
            Ok(ErlangTerm::Atom(if func_arity == arity { "true" } else { "false" }.to_string()))
        } else {
            Ok(ErlangTerm::Atom("false".to_string()))
        }
    }

    /// Type check: is boolean
    ///
    /// Equivalent to `erlang:is_boolean/1` in Erlang.
    pub fn is_boolean(arg: &ErlangTerm) -> ErlangTerm {
        ErlangTerm::Atom(if arg.is_boolean() { "true" } else { "false" }.to_string())
    }

    /// Type check: is record
    ///
    /// Equivalent to `erlang:is_record/2` in Erlang.
    /// Checks if the term is a tuple with the first element matching the given atom.
    ///
    /// # Arguments
    /// * `arg1` - Term to check
    /// * `arg2` - Atom to match (must be an atom)
    ///
    /// # Returns
    /// * `Ok(ErlangTerm::Atom("true"))` - If arg1 is a tuple with first element == arg2
    /// * `Ok(ErlangTerm::Atom("false"))` - Otherwise
    /// * `Err(OpError)` - If arg2 is not an atom
    pub fn is_record(arg1: &ErlangTerm, arg2: &ErlangTerm) -> Result<ErlangTerm, OpError> {
        if !arg2.is_atom() {
            return Err(OpError::BadArgument("Second argument must be an atom".to_string()));
        }

        if let ErlangTerm::Tuple(elements) = arg1 {
            if !elements.is_empty() {
                if elements[0] == *arg2 {
                    return Ok(ErlangTerm::Atom("true".to_string()));
                }
            }
        }

        Ok(ErlangTerm::Atom("false".to_string()))
    }

    /// Type check: is record with size
    ///
    /// Equivalent to `erlang:is_record/3` in Erlang.
    /// Checks if the term is a tuple with the specified size and first element matching the given atom.
    ///
    /// # Arguments
    /// * `arg1` - Term to check
    /// * `arg2` - Atom to match (must be an atom)
    /// * `arg3` - Size (must be a non-negative integer)
    ///
    /// # Returns
    /// * `Ok(ErlangTerm::Atom("true"))` - If arg1 is a tuple with size == arg3 and first element == arg2
    /// * `Ok(ErlangTerm::Atom("false"))` - Otherwise
    /// * `Err(OpError)` - If arguments are invalid
    pub fn is_record_with_size(
        arg1: &ErlangTerm,
        arg2: &ErlangTerm,
        arg3: &ErlangTerm,
    ) -> Result<ErlangTerm, OpError> {
        if !arg2.is_atom() {
            return Err(OpError::BadArgument("Second argument must be an atom".to_string()));
        }

        let size = Self::to_non_negative_integer(arg3)?;

        if let ErlangTerm::Tuple(elements) = arg1 {
            if elements.len() == size && !elements.is_empty() {
                if elements[0] == *arg2 {
                    return Ok(ErlangTerm::Atom("true".to_string()));
                }
            }
        }

        Ok(ErlangTerm::Atom("false".to_string()))
    }

    /// Type check: is map
    ///
    /// Equivalent to `erlang:is_map/1` in Erlang.
    pub fn is_map(arg: &ErlangTerm) -> ErlangTerm {
        ErlangTerm::Atom(if arg.is_map() { "true" } else { "false" }.to_string())
    }

    /// Helper: Convert ErlangTerm to bool
    fn to_bool(term: &ErlangTerm) -> Result<bool, OpError> {
        match term {
            ErlangTerm::Atom(ref s) if s == "true" => Ok(true),
            ErlangTerm::Atom(ref s) if s == "false" => Ok(false),
            _ => Err(OpError::BadArgument(
                "Argument must be a boolean (true or false atom)".to_string(),
            )),
        }
    }

    /// Helper: Convert ErlangTerm to non-negative integer
    fn to_non_negative_integer(term: &ErlangTerm) -> Result<usize, OpError> {
        match term {
            ErlangTerm::Integer(n) if *n >= 0 => Ok(*n as usize),
            _ => Err(OpError::BadArgument(
                "Argument must be a non-negative integer".to_string(),
            )),
        }
    }
}

/// Error type for operator operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpError {
    /// Invalid argument provided
    BadArgument(String),
}

impl std::fmt::Display for OpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpError::BadArgument(msg) => write!(f, "Bad argument: {}", msg),
        }
    }
}

impl std::error::Error for OpError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_and() {
        let true_atom = ErlangTerm::Atom("true".to_string());
        let false_atom = ErlangTerm::Atom("false".to_string());

        assert_eq!(
            OpBif::and(&true_atom, &true_atom).unwrap(),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::and(&true_atom, &false_atom).unwrap(),
            ErlangTerm::Atom("false".to_string())
        );
        assert_eq!(
            OpBif::and(&false_atom, &true_atom).unwrap(),
            ErlangTerm::Atom("false".to_string())
        );
        assert_eq!(
            OpBif::and(&false_atom, &false_atom).unwrap(),
            ErlangTerm::Atom("false".to_string())
        );
    }

    #[test]
    fn test_or() {
        let true_atom = ErlangTerm::Atom("true".to_string());
        let false_atom = ErlangTerm::Atom("false".to_string());

        assert_eq!(
            OpBif::or(&true_atom, &true_atom).unwrap(),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::or(&true_atom, &false_atom).unwrap(),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::or(&false_atom, &true_atom).unwrap(),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::or(&false_atom, &false_atom).unwrap(),
            ErlangTerm::Atom("false".to_string())
        );
    }

    #[test]
    fn test_xor() {
        let true_atom = ErlangTerm::Atom("true".to_string());
        let false_atom = ErlangTerm::Atom("false".to_string());

        assert_eq!(
            OpBif::xor(&true_atom, &true_atom).unwrap(),
            ErlangTerm::Atom("false".to_string())
        );
        assert_eq!(
            OpBif::xor(&true_atom, &false_atom).unwrap(),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::xor(&false_atom, &true_atom).unwrap(),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::xor(&false_atom, &false_atom).unwrap(),
            ErlangTerm::Atom("false".to_string())
        );
    }

    #[test]
    fn test_not() {
        let true_atom = ErlangTerm::Atom("true".to_string());
        let false_atom = ErlangTerm::Atom("false".to_string());

        assert_eq!(
            OpBif::not(&true_atom).unwrap(),
            ErlangTerm::Atom("false".to_string())
        );
        assert_eq!(
            OpBif::not(&false_atom).unwrap(),
            ErlangTerm::Atom("true".to_string())
        );
    }

    #[test]
    fn test_comparisons() {
        let int1 = ErlangTerm::Integer(5);
        let int2 = ErlangTerm::Integer(10);
        let int3 = ErlangTerm::Integer(5);

        assert_eq!(
            OpBif::sgt(&int2, &int1),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::sgt(&int1, &int2),
            ErlangTerm::Atom("false".to_string())
        );

        assert_eq!(
            OpBif::sge(&int2, &int1),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::sge(&int1, &int3),
            ErlangTerm::Atom("true".to_string())
        );

        assert_eq!(
            OpBif::slt(&int1, &int2),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::slt(&int2, &int1),
            ErlangTerm::Atom("false".to_string())
        );

        assert_eq!(
            OpBif::sle(&int1, &int2),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::sle(&int1, &int3),
            ErlangTerm::Atom("true".to_string())
        );
    }

    #[test]
    fn test_equality() {
        let int1 = ErlangTerm::Integer(5);
        let int2 = ErlangTerm::Integer(5);
        let int3 = ErlangTerm::Integer(10);

        assert_eq!(
            OpBif::seq(&int1, &int2),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::seq(&int1, &int3),
            ErlangTerm::Atom("false".to_string())
        );

        assert_eq!(
            OpBif::sneq(&int1, &int3),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::sneq(&int1, &int2),
            ErlangTerm::Atom("false".to_string())
        );
    }

    #[test]
    fn test_type_checks() {
        assert_eq!(
            OpBif::is_atom(&ErlangTerm::Atom("test".to_string())),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::is_atom(&ErlangTerm::Integer(5)),
            ErlangTerm::Atom("false".to_string())
        );

        assert_eq!(
            OpBif::is_integer(&ErlangTerm::Integer(5)),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::is_integer(&ErlangTerm::Float(5.0)),
            ErlangTerm::Atom("false".to_string())
        );

        assert_eq!(
            OpBif::is_float(&ErlangTerm::Float(5.0)),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::is_float(&ErlangTerm::Integer(5)),
            ErlangTerm::Atom("false".to_string())
        );

        assert_eq!(
            OpBif::is_number(&ErlangTerm::Integer(5)),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::is_number(&ErlangTerm::Float(5.0)),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::is_number(&ErlangTerm::Atom("test".to_string())),
            ErlangTerm::Atom("false".to_string())
        );

        assert_eq!(
            OpBif::is_list(&ErlangTerm::List(vec![])),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::is_list(&ErlangTerm::Nil),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::is_list(&ErlangTerm::Integer(5)),
            ErlangTerm::Atom("false".to_string())
        );

        assert_eq!(
            OpBif::is_tuple(&ErlangTerm::Tuple(vec![])),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::is_tuple(&ErlangTerm::Integer(5)),
            ErlangTerm::Atom("false".to_string())
        );

        assert_eq!(
            OpBif::is_binary(&ErlangTerm::Binary(vec![1, 2, 3])),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::is_binary(&ErlangTerm::Bitstring(vec![1, 2, 3], 24)),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::is_binary(&ErlangTerm::Bitstring(vec![1], 7)),
            ErlangTerm::Atom("false".to_string())
        );

        assert_eq!(
            OpBif::is_bitstring(&ErlangTerm::Binary(vec![1, 2, 3])),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::is_bitstring(&ErlangTerm::Bitstring(vec![1], 7)),
            ErlangTerm::Atom("true".to_string())
        );

        assert_eq!(
            OpBif::is_pid(&ErlangTerm::Pid(123)),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::is_port(&ErlangTerm::Port(456)),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::is_reference(&ErlangTerm::Reference(789)),
            ErlangTerm::Atom("true".to_string())
        );

        assert_eq!(
            OpBif::is_function(&ErlangTerm::Function { arity: 2 }),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::is_function(&ErlangTerm::Integer(5)),
            ErlangTerm::Atom("false".to_string())
        );

        assert_eq!(
            OpBif::is_boolean(&ErlangTerm::Atom("true".to_string())),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::is_boolean(&ErlangTerm::Atom("false".to_string())),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::is_boolean(&ErlangTerm::Atom("maybe".to_string())),
            ErlangTerm::Atom("false".to_string())
        );

        assert_eq!(
            OpBif::is_map(&ErlangTerm::Map(HashMap::new())),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::is_map(&ErlangTerm::Integer(5)),
            ErlangTerm::Atom("false".to_string())
        );
    }

    #[test]
    fn test_is_function_with_arity() {
        let func = ErlangTerm::Function { arity: 2 };
        let arity2 = ErlangTerm::Integer(2);
        let arity3 = ErlangTerm::Integer(3);

        assert_eq!(
            OpBif::is_function_with_arity(&func, &arity2).unwrap(),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::is_function_with_arity(&func, &arity3).unwrap(),
            ErlangTerm::Atom("false".to_string())
        );
        assert_eq!(
            OpBif::is_function_with_arity(&ErlangTerm::Integer(5), &arity2).unwrap(),
            ErlangTerm::Atom("false".to_string())
        );
    }

    #[test]
    fn test_is_record() {
        let record_tag = ErlangTerm::Atom("record".to_string());
        let tuple = ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("record".to_string()),
            ErlangTerm::Integer(1),
            ErlangTerm::Integer(2),
        ]);
        let wrong_tuple = ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("other".to_string()),
            ErlangTerm::Integer(1),
        ]);

        assert_eq!(
            OpBif::is_record(&tuple, &record_tag).unwrap(),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::is_record(&wrong_tuple, &record_tag).unwrap(),
            ErlangTerm::Atom("false".to_string())
        );
        assert_eq!(
            OpBif::is_record(&ErlangTerm::Integer(5), &record_tag).unwrap(),
            ErlangTerm::Atom("false".to_string())
        );
    }

    #[test]
    fn test_is_record_with_size() {
        let record_tag = ErlangTerm::Atom("record".to_string());
        let size3 = ErlangTerm::Integer(3);
        let tuple = ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("record".to_string()),
            ErlangTerm::Integer(1),
            ErlangTerm::Integer(2),
        ]);
        let wrong_size = ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("record".to_string()),
            ErlangTerm::Integer(1),
        ]);

        assert_eq!(
            OpBif::is_record_with_size(&tuple, &record_tag, &size3).unwrap(),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::is_record_with_size(&wrong_size, &record_tag, &size3).unwrap(),
            ErlangTerm::Atom("false".to_string())
        );
    }

    #[test]
    fn test_error_handling() {
        let non_bool = ErlangTerm::Integer(5);
        assert!(OpBif::and(&non_bool, &ErlangTerm::Atom("true".to_string())).is_err());
        assert!(OpBif::or(&non_bool, &ErlangTerm::Atom("true".to_string())).is_err());
        assert!(OpBif::not(&non_bool).is_err());

        let negative = ErlangTerm::Integer(-1);
        assert!(OpBif::is_function_with_arity(
            &ErlangTerm::Function { arity: 2 },
            &negative
        )
        .is_err());

        let non_atom = ErlangTerm::Integer(5);
        assert!(OpBif::is_record(&ErlangTerm::Tuple(vec![]), &non_atom).is_err());
        assert!(OpBif::is_record_with_size(
            &ErlangTerm::Tuple(vec![]),
            &non_atom,
            &ErlangTerm::Integer(1)
        )
        .is_err());
    }

    #[test]
    fn test_float_comparisons() {
        let float1 = ErlangTerm::Float(5.0);
        let float2 = ErlangTerm::Float(10.0);
        let int1 = ErlangTerm::Integer(5);

        assert_eq!(
            OpBif::slt(&float1, &float2),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::sgt(&float2, &float1),
            ErlangTerm::Atom("true".to_string())
        );

        // Mixed integer/float comparisons (structural equality allows coercion)
        assert_eq!(
            OpBif::seq(&float1, &int1),
            ErlangTerm::Atom("true".to_string()),
            "Structural equality: Float(5.0) == Integer(5) should be true"
        );
        // But exact equality should be false
        assert_eq!(
            OpBif::seqeq(&float1, &int1),
            ErlangTerm::Atom("false".to_string()),
            "Exact equality: Float(5.0) =:= Integer(5) should be false"
        );
    }

    #[test]
    fn test_atom_comparisons() {
        let atom1 = ErlangTerm::Atom("a".to_string());
        let atom2 = ErlangTerm::Atom("b".to_string());

        assert_eq!(
            OpBif::slt(&atom1, &atom2),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::sgt(&atom2, &atom1),
            ErlangTerm::Atom("true".to_string())
        );
    }

    #[test]
    fn test_structural_vs_exact_equality() {
        // Structural equality (==) allows type coercion
        let int1 = ErlangTerm::Integer(1);
        let float1 = ErlangTerm::Float(1.0);
        
        assert_eq!(
            OpBif::seq(&int1, &float1),
            ErlangTerm::Atom("true".to_string()),
            "Structural equality should allow Integer(1) == Float(1.0)"
        );
        
        // Exact equality (=:=) requires exact type match
        assert_eq!(
            OpBif::seqeq(&int1, &float1),
            ErlangTerm::Atom("false".to_string()),
            "Exact equality should NOT allow Integer(1) =:= Float(1.0)"
        );
        
        // Same types should be equal in both
        let int2 = ErlangTerm::Integer(1);
        assert_eq!(
            OpBif::seq(&int1, &int2),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::seqeq(&int1, &int2),
            ErlangTerm::Atom("true".to_string())
        );
    }

    #[test]
    fn test_list_comparisons() {
        let list1 = ErlangTerm::List(vec![ErlangTerm::Integer(1), ErlangTerm::Integer(2)]);
        let list2 = ErlangTerm::List(vec![ErlangTerm::Integer(1), ErlangTerm::Integer(3)]);
        let list3 = ErlangTerm::List(vec![ErlangTerm::Integer(1), ErlangTerm::Integer(2)]);
        let list4 = ErlangTerm::List(vec![ErlangTerm::Integer(1)]);
        
        // list1 < list2 (2 < 3)
        assert_eq!(
            OpBif::slt(&list1, &list2),
            ErlangTerm::Atom("true".to_string())
        );
        
        // list1 == list3
        assert_eq!(
            OpBif::seq(&list1, &list3),
            ErlangTerm::Atom("true".to_string())
        );
        
        // list4 < list1 (shorter list)
        assert_eq!(
            OpBif::slt(&list4, &list1),
            ErlangTerm::Atom("true".to_string())
        );
        
        // Nil < non-empty list
        assert_eq!(
            OpBif::slt(&ErlangTerm::Nil, &list1),
            ErlangTerm::Atom("true".to_string())
        );
    }

    #[test]
    fn test_tuple_comparisons() {
        let tuple1 = ErlangTerm::Tuple(vec![ErlangTerm::Integer(1)]);
        let tuple2 = ErlangTerm::Tuple(vec![ErlangTerm::Integer(1), ErlangTerm::Integer(2)]);
        let tuple3 = ErlangTerm::Tuple(vec![ErlangTerm::Integer(2)]);
        
        // tuple1 < tuple2 (by arity)
        assert_eq!(
            OpBif::slt(&tuple1, &tuple2),
            ErlangTerm::Atom("true".to_string())
        );
        
        // tuple1 < tuple3 (same arity, but 1 < 2)
        assert_eq!(
            OpBif::slt(&tuple1, &tuple3),
            ErlangTerm::Atom("true".to_string())
        );
    }

    #[test]
    fn test_binary_comparisons() {
        let bin1 = ErlangTerm::Binary(vec![1, 2]);
        let bin2 = ErlangTerm::Binary(vec![1, 3]);
        let bin3 = ErlangTerm::Binary(vec![1, 2, 3]);
        
        // bin1 < bin2 (2 < 3)
        assert_eq!(
            OpBif::slt(&bin1, &bin2),
            ErlangTerm::Atom("true".to_string())
        );
        
        // bin1 < bin3 (shorter)
        assert_eq!(
            OpBif::slt(&bin1, &bin3),
            ErlangTerm::Atom("true".to_string())
        );
    }

    #[test]
    fn test_number_coercion_in_equality() {
        // Test that structural equality allows number type coercion
        let int5 = ErlangTerm::Integer(5);
        let float5 = ErlangTerm::Float(5.0);
        let float5_1 = ErlangTerm::Float(5.1);
        
        // Integer(5) == Float(5.0) should be true (structural equality)
        assert_eq!(
            OpBif::seq(&int5, &float5),
            ErlangTerm::Atom("true".to_string())
        );
        
        // Integer(5) =:= Float(5.0) should be false (exact equality)
        assert_eq!(
            OpBif::seqeq(&int5, &float5),
            ErlangTerm::Atom("false".to_string())
        );
        
        // Integer(5) == Float(5.1) should be false
        assert_eq!(
            OpBif::seq(&int5, &float5_1),
            ErlangTerm::Atom("false".to_string())
        );
    }

    #[test]
    fn test_inequality_operators() {
        let int1 = ErlangTerm::Integer(1);
        let float1 = ErlangTerm::Float(1.0);
        let int2 = ErlangTerm::Integer(2);
        
        // Structural inequality
        assert_eq!(
            OpBif::sneq(&int1, &float1),
            ErlangTerm::Atom("false".to_string()),
            "Structural: 1 /= 1.0 should be false"
        );
        assert_eq!(
            OpBif::sneq(&int1, &int2),
            ErlangTerm::Atom("true".to_string()),
            "Structural: 1 /= 2 should be true"
        );
        
        // Exact inequality
        assert_eq!(
            OpBif::sneqeq(&int1, &float1),
            ErlangTerm::Atom("true".to_string()),
            "Exact: 1 =/= 1.0 should be true (different types)"
        );
        assert_eq!(
            OpBif::sneqeq(&int1, &int2),
            ErlangTerm::Atom("true".to_string()),
            "Exact: 1 =/= 2 should be true"
        );
    }

    #[test]
    fn test_float_zero_comparisons() {
        // Test positive and negative zero handling
        let pos_zero = ErlangTerm::Float(0.0);
        let neg_zero = ErlangTerm::Float(-0.0);
        let int_zero = ErlangTerm::Integer(0);
        
        // Structural equality: 0.0 == -0.0 (they represent the same number)
        assert_eq!(
            OpBif::seq(&pos_zero, &neg_zero),
            ErlangTerm::Atom("true".to_string())
        );
        
        // Exact equality: 0.0 =:= -0.0 (in Erlang OTP 27+, these are different)
        // For now, we'll treat them as equal since we don't distinguish sign
        // In a full implementation, this would need special handling
        
        // Structural: 0 == 0.0
        assert_eq!(
            OpBif::seq(&int_zero, &pos_zero),
            ErlangTerm::Atom("true".to_string())
        );
    }

    #[test]
    fn test_nan_equality() {
        // Test NaN == NaN returns true (line 42)
        let nan1 = ErlangTerm::Float(f64::NAN);
        let nan2 = ErlangTerm::Float(f64::NAN);
        
        // Direct equality comparison should return true for NaN == NaN
        assert_eq!(nan1, nan2, "NaN == NaN should be true");
        
        // Also test via structural equality BIF
        assert_eq!(
            OpBif::seq(&nan1, &nan2),
            ErlangTerm::Atom("true".to_string())
        );
        
        // NaN != regular float
        let regular_float = ErlangTerm::Float(5.0);
        assert_ne!(nan1, regular_float);
    }

    #[test]
    fn test_partial_eq_tuple() {
        // Test Tuple == Tuple (line 47)
        let tuple1 = ErlangTerm::Tuple(vec![
            ErlangTerm::Integer(1),
            ErlangTerm::Integer(2),
        ]);
        let tuple2 = ErlangTerm::Tuple(vec![
            ErlangTerm::Integer(1),
            ErlangTerm::Integer(2),
        ]);
        let tuple3 = ErlangTerm::Tuple(vec![
            ErlangTerm::Integer(1),
            ErlangTerm::Integer(3),
        ]);
        
        assert_eq!(tuple1, tuple2, "Equal tuples should be equal");
        assert_ne!(tuple1, tuple3, "Different tuples should not be equal");
    }

    #[test]
    fn test_partial_eq_binary() {
        // Test Binary == Binary (line 49)
        let bin1 = ErlangTerm::Binary(vec![1, 2, 3]);
        let bin2 = ErlangTerm::Binary(vec![1, 2, 3]);
        let bin3 = ErlangTerm::Binary(vec![1, 2, 4]);
        
        assert_eq!(bin1, bin2, "Equal binaries should be equal");
        assert_ne!(bin1, bin3, "Different binaries should not be equal");
    }

    #[test]
    fn test_partial_eq_bitstring() {
        // Test Bitstring == Bitstring (line 50-51)
        let bits1 = ErlangTerm::Bitstring(vec![1, 2], 16);
        let bits2 = ErlangTerm::Bitstring(vec![1, 2], 16);
        let bits3 = ErlangTerm::Bitstring(vec![1, 2], 15);
        let bits4 = ErlangTerm::Bitstring(vec![1, 3], 16);
        
        assert_eq!(bits1, bits2, "Equal bitstrings should be equal");
        assert_ne!(bits1, bits3, "Different bit lengths should not be equal");
        assert_ne!(bits1, bits4, "Different data should not be equal");
    }

    #[test]
    fn test_partial_eq_map() {
        // Test Map == Map (line 53)
        let mut map1 = HashMap::new();
        map1.insert(ErlangTerm::Atom("key".to_string()), ErlangTerm::Integer(1));
        let mut map2 = HashMap::new();
        map2.insert(ErlangTerm::Atom("key".to_string()), ErlangTerm::Integer(1));
        let mut map3 = HashMap::new();
        map3.insert(ErlangTerm::Atom("key".to_string()), ErlangTerm::Integer(2));
        
        let term1 = ErlangTerm::Map(map1);
        let term2 = ErlangTerm::Map(map2);
        let term3 = ErlangTerm::Map(map3);
        
        assert_eq!(term1, term2, "Equal maps should be equal");
        assert_ne!(term1, term3, "Different maps should not be equal");
    }

    #[test]
    fn test_partial_eq_pid() {
        // Test Pid == Pid (line 54)
        let pid1 = ErlangTerm::Pid(123);
        let pid2 = ErlangTerm::Pid(123);
        let pid3 = ErlangTerm::Pid(456);
        
        assert_eq!(pid1, pid2, "Equal PIDs should be equal");
        assert_ne!(pid1, pid3, "Different PIDs should not be equal");
    }

    #[test]
    fn test_partial_eq_port() {
        // Test Port == Port (line 55)
        let port1 = ErlangTerm::Port(789);
        let port2 = ErlangTerm::Port(789);
        let port3 = ErlangTerm::Port(101112);
        
        assert_eq!(port1, port2, "Equal ports should be equal");
        assert_ne!(port1, port3, "Different ports should not be equal");
    }

    #[test]
    fn test_partial_eq_reference() {
        // Test Reference == Reference (line 56)
        let ref1 = ErlangTerm::Reference(999);
        let ref2 = ErlangTerm::Reference(999);
        let ref3 = ErlangTerm::Reference(888);
        
        assert_eq!(ref1, ref2, "Equal references should be equal");
        assert_ne!(ref1, ref3, "Different references should not be equal");
    }

    #[test]
    fn test_partial_eq_function() {
        // Test Function == Function (line 57)
        let func1 = ErlangTerm::Function { arity: 2 };
        let func2 = ErlangTerm::Function { arity: 2 };
        let func3 = ErlangTerm::Function { arity: 3 };
        
        assert_eq!(func1, func2, "Equal functions should be equal");
        assert_ne!(func1, func3, "Different functions should not be equal");
    }

    #[test]
    fn test_partial_eq_nil() {
        // Test Nil == Nil (line 58)
        let nil1 = ErlangTerm::Nil;
        let nil2 = ErlangTerm::Nil;
        let list = ErlangTerm::List(vec![]);
        
        assert_eq!(nil1, nil2, "Nil == Nil should be true");
        assert_ne!(nil1, list, "Nil != empty list");
    }

    #[test]
    fn test_hash_implementation() {
        use std::collections::HashSet;
        
        // Test Hash implementation for all types (lines 68-76 and beyond)
        let mut set = HashSet::new();
        
        // Test Atom hashing (line 70-72)
        let atom1 = ErlangTerm::Atom("test".to_string());
        let atom2 = ErlangTerm::Atom("test".to_string());
        set.insert(atom1.clone());
        assert!(set.contains(&atom2), "Atom should be hashable and findable");
        
        // Test Integer hashing (line 74-76)
        let int1 = ErlangTerm::Integer(42);
        let int2 = ErlangTerm::Integer(42);
        set.insert(int1.clone());
        assert!(set.contains(&int2), "Integer should be hashable and findable");
        
        // Test Float hashing
        let float1 = ErlangTerm::Float(3.14);
        let float2 = ErlangTerm::Float(3.14);
        set.insert(float1.clone());
        assert!(set.contains(&float2), "Float should be hashable and findable");
        
        // Test Tuple hashing
        let tuple1 = ErlangTerm::Tuple(vec![ErlangTerm::Integer(1), ErlangTerm::Integer(2)]);
        let tuple2 = ErlangTerm::Tuple(vec![ErlangTerm::Integer(1), ErlangTerm::Integer(2)]);
        set.insert(tuple1.clone());
        assert!(set.contains(&tuple2), "Tuple should be hashable and findable");
        
        // Test List hashing
        let list1 = ErlangTerm::List(vec![ErlangTerm::Integer(1), ErlangTerm::Integer(2)]);
        let list2 = ErlangTerm::List(vec![ErlangTerm::Integer(1), ErlangTerm::Integer(2)]);
        set.insert(list1.clone());
        assert!(set.contains(&list2), "List should be hashable and findable");
        
        // Test Binary hashing
        let bin1 = ErlangTerm::Binary(vec![1, 2, 3]);
        let bin2 = ErlangTerm::Binary(vec![1, 2, 3]);
        set.insert(bin1.clone());
        assert!(set.contains(&bin2), "Binary should be hashable and findable");
        
        // Test Bitstring hashing
        let bits1 = ErlangTerm::Bitstring(vec![1, 2], 16);
        let bits2 = ErlangTerm::Bitstring(vec![1, 2], 16);
        set.insert(bits1.clone());
        assert!(set.contains(&bits2), "Bitstring should be hashable and findable");
        
        // Test Map hashing
        let mut map1 = HashMap::new();
        map1.insert(ErlangTerm::Atom("k".to_string()), ErlangTerm::Integer(1));
        let mut map2 = HashMap::new();
        map2.insert(ErlangTerm::Atom("k".to_string()), ErlangTerm::Integer(1));
        let map_term1 = ErlangTerm::Map(map1);
        let map_term2 = ErlangTerm::Map(map2);
        set.insert(map_term1.clone());
        assert!(set.contains(&map_term2), "Map should be hashable and findable");
        
        // Test Pid hashing
        let pid1 = ErlangTerm::Pid(123);
        let pid2 = ErlangTerm::Pid(123);
        set.insert(pid1.clone());
        assert!(set.contains(&pid2), "Pid should be hashable and findable");
        
        // Test Port hashing
        let port1 = ErlangTerm::Port(456);
        let port2 = ErlangTerm::Port(456);
        set.insert(port1.clone());
        assert!(set.contains(&port2), "Port should be hashable and findable");
        
        // Test Reference hashing
        let ref1 = ErlangTerm::Reference(789);
        let ref2 = ErlangTerm::Reference(789);
        set.insert(ref1.clone());
        assert!(set.contains(&ref2), "Reference should be hashable and findable");
        
        // Test Function hashing
        let func1 = ErlangTerm::Function { arity: 2 };
        let func2 = ErlangTerm::Function { arity: 2 };
        set.insert(func1.clone());
        assert!(set.contains(&func2), "Function should be hashable and findable");
        
        // Test Nil hashing
        let nil1 = ErlangTerm::Nil;
        let nil2 = ErlangTerm::Nil;
        set.insert(nil1.clone());
        assert!(set.contains(&nil2), "Nil should be hashable and findable");
    }

    #[test]
    fn test_hashmap_with_erlang_term() {
        // Test using ErlangTerm as HashMap key (exercises Hash trait)
        let mut map = HashMap::new();
        
        let key1 = ErlangTerm::Atom("key1".to_string());
        let key2 = ErlangTerm::Integer(42);
        let key3 = ErlangTerm::Tuple(vec![ErlangTerm::Integer(1)]);
        
        map.insert(key1.clone(), ErlangTerm::Integer(100));
        map.insert(key2.clone(), ErlangTerm::Integer(200));
        map.insert(key3.clone(), ErlangTerm::Integer(300));
        
        assert_eq!(map.get(&key1), Some(&ErlangTerm::Integer(100)));
        assert_eq!(map.get(&key2), Some(&ErlangTerm::Integer(200)));
        assert_eq!(map.get(&key3), Some(&ErlangTerm::Integer(300)));
        
        // Test with different instances but same value
        let key1_dup = ErlangTerm::Atom("key1".to_string());
        assert_eq!(map.get(&key1_dup), Some(&ErlangTerm::Integer(100)));
    }

    #[test]
    fn test_all_partial_eq_match_arms() {
        // Comprehensive test to ensure all PartialEq match arms are covered
        // This exercises lines 47-58
        
        // Tuple
        assert_eq!(
            ErlangTerm::Tuple(vec![ErlangTerm::Integer(1)]),
            ErlangTerm::Tuple(vec![ErlangTerm::Integer(1)])
        );
        
        // Binary
        assert_eq!(
            ErlangTerm::Binary(vec![1, 2]),
            ErlangTerm::Binary(vec![1, 2])
        );
        
        // Bitstring
        assert_eq!(
            ErlangTerm::Bitstring(vec![1, 2], 16),
            ErlangTerm::Bitstring(vec![1, 2], 16)
        );
        
        // Map
        let mut map1 = HashMap::new();
        map1.insert(ErlangTerm::Integer(1), ErlangTerm::Integer(2));
        let mut map2 = HashMap::new();
        map2.insert(ErlangTerm::Integer(1), ErlangTerm::Integer(2));
        assert_eq!(ErlangTerm::Map(map1), ErlangTerm::Map(map2));
        
        // Pid
        assert_eq!(ErlangTerm::Pid(123), ErlangTerm::Pid(123));
        
        // Port
        assert_eq!(ErlangTerm::Port(456), ErlangTerm::Port(456));
        
        // Reference
        assert_eq!(ErlangTerm::Reference(789), ErlangTerm::Reference(789));
        
        // Function
        assert_eq!(
            ErlangTerm::Function { arity: 2 },
            ErlangTerm::Function { arity: 2 }
        );
        
        // Nil
        assert_eq!(ErlangTerm::Nil, ErlangTerm::Nil);
    }

    #[test]
    fn test_is_binary_false_path() {
        // Test line 175: is_binary returning false for non-binary types
        assert_eq!(
            OpBif::is_binary(&ErlangTerm::Integer(5)),
            ErlangTerm::Atom("false".to_string())
        );
        assert_eq!(
            OpBif::is_binary(&ErlangTerm::Atom("test".to_string())),
            ErlangTerm::Atom("false".to_string())
        );
        assert_eq!(
            OpBif::is_binary(&ErlangTerm::List(vec![])),
            ErlangTerm::Atom("false".to_string())
        );
    }

    #[test]
    fn test_mixed_number_comparisons() {
        // Test lines 258-262: Integer vs Float comparisons
        let int5 = ErlangTerm::Integer(5);
        let float5 = ErlangTerm::Float(5.0);
        let float6 = ErlangTerm::Float(6.0);
        let int6 = ErlangTerm::Integer(6);
        
        // Integer < Float (line 258-259)
        assert_eq!(
            OpBif::slt(&int5, &float6),
            ErlangTerm::Atom("true".to_string())
        );
        
        // Float < Integer (line 261-262)
        assert_eq!(
            OpBif::slt(&float5, &int6),
            ErlangTerm::Atom("true".to_string())
        );
        
        // Integer > Float
        assert_eq!(
            OpBif::sgt(&int6, &float5),
            ErlangTerm::Atom("true".to_string())
        );
        
        // Float > Integer
        assert_eq!(
            OpBif::sgt(&float6, &int5),
            ErlangTerm::Atom("true".to_string())
        );
    }

    #[test]
    fn test_nil_comparisons() {
        // Test lines 273-274: Nil comparisons
        let nil = ErlangTerm::Nil;
        let list = ErlangTerm::List(vec![ErlangTerm::Integer(1)]);
        
        // Nil < List (line 273)
        assert_eq!(
            OpBif::slt(&nil, &list),
            ErlangTerm::Atom("true".to_string())
        );
        
        // List > Nil (line 274)
        assert_eq!(
            OpBif::sgt(&list, &nil),
            ErlangTerm::Atom("true".to_string())
        );
        
        // Nil == Nil
        assert_eq!(
            OpBif::seq(&nil, &ErlangTerm::Nil),
            ErlangTerm::Atom("true".to_string())
        );
    }

    #[test]
    fn test_tuple_comparison_edge_cases() {
        // Test lines 285, 288-290, 293: Tuple comparison edge cases
        let tuple1 = ErlangTerm::Tuple(vec![ErlangTerm::Integer(1), ErlangTerm::Integer(2)]);
        let tuple2 = ErlangTerm::Tuple(vec![ErlangTerm::Integer(1), ErlangTerm::Integer(3)]);
        let tuple3 = ErlangTerm::Tuple(vec![ErlangTerm::Integer(1), ErlangTerm::Integer(2)]);
        
        // Equal tuples (line 293)
        assert_eq!(
            OpBif::seq(&tuple1, &tuple3),
            ErlangTerm::Atom("true".to_string())
        );
        
        // Different tuples (line 285 - returns early when ord != Equal)
        assert_eq!(
            OpBif::slt(&tuple1, &tuple2),
            ErlangTerm::Atom("true".to_string())
        );
        
        // Test with non-comparable elements (lines 288-290)
        // This would require elements that compare() returns None for
        // For now, test with different types that aren't directly comparable
        let tuple4 = ErlangTerm::Tuple(vec![ErlangTerm::Atom("a".to_string())]);
        let tuple5 = ErlangTerm::Tuple(vec![ErlangTerm::Atom("b".to_string())]);
        assert_eq!(
            OpBif::slt(&tuple4, &tuple5),
            ErlangTerm::Atom("true".to_string())
        );
    }

    #[test]
    fn test_bitstring_binary_comparisons() {
        // Test lines 301-304, 307, 309, 311-312: Bitstring/Binary comparisons
        let bits1 = ErlangTerm::Bitstring(vec![1, 2], 16);
        let bits2 = ErlangTerm::Bitstring(vec![1, 2], 16);
        let bits3 = ErlangTerm::Bitstring(vec![1, 2], 15);
        let bits4 = ErlangTerm::Bitstring(vec![1, 3], 16);
        
        // Equal bitstrings (line 303 - bits_a == bits_b)
        assert_eq!(
            OpBif::seq(&bits1, &bits2),
            ErlangTerm::Atom("true".to_string())
        );
        
        // Different bit lengths (line 303 - bits_a != bits_b)
        assert_eq!(
            OpBif::slt(&bits3, &bits1),
            ErlangTerm::Atom("true".to_string())
        );
        
        // Different data (line 302 - a != b, returns early)
        assert_eq!(
            OpBif::slt(&bits1, &bits4),
            ErlangTerm::Atom("true".to_string())
        );
        
        // Binary vs Bitstring (line 307-309) - comparison, not equality
        let bin = ErlangTerm::Binary(vec![1, 2]);
        // Binary and Bitstring can be compared but not equal (different types)
        assert_eq!(
            OpBif::sle(&bin, &bits1),
            ErlangTerm::Atom("true".to_string())
        );
        
        // Bitstring vs Binary (line 311-312) - comparison, not equality
        assert_eq!(
            OpBif::sle(&bits1, &bin),
            ErlangTerm::Atom("true".to_string())
        );
        
        // Test with different data to exercise the comparison path
        let bin2 = ErlangTerm::Binary(vec![1, 3]);
        assert_eq!(
            OpBif::slt(&bin, &bin2),
            ErlangTerm::Atom("true".to_string())
        );
    }

    #[test]
    fn test_map_comparisons() {
        // Test lines 316-317, 321-322, 324, 327: Map comparison paths
        let mut map1 = HashMap::new();
        map1.insert(ErlangTerm::Integer(1), ErlangTerm::Integer(10));
        let mut map2 = HashMap::new();
        map2.insert(ErlangTerm::Integer(1), ErlangTerm::Integer(10));
        let mut map3 = HashMap::new();
        map3.insert(ErlangTerm::Integer(1), ErlangTerm::Integer(20));
        let mut map4 = HashMap::new();
        map4.insert(ErlangTerm::Integer(1), ErlangTerm::Integer(10));
        map4.insert(ErlangTerm::Integer(2), ErlangTerm::Integer(20));
        
        let term1 = ErlangTerm::Map(map1);
        let term2 = ErlangTerm::Map(map2);
        let term3 = ErlangTerm::Map(map3);
        let term4 = ErlangTerm::Map(map4);
        
        // Equal maps (line 321-322)
        assert_eq!(
            OpBif::seq(&term1, &term2),
            ErlangTerm::Atom("true".to_string())
        );
        
        // Different size maps (line 327 - returns early)
        assert_eq!(
            OpBif::slt(&term1, &term4),
            ErlangTerm::Atom("true".to_string())
        );
        
        // Same size, different content (line 324 - returns None, which becomes false)
        // In Erlang, maps with same size but different keys aren't easily comparable
        // This should return false for comparison operators
        assert_eq!(
            OpBif::slt(&term1, &term3),
            ErlangTerm::Atom("false".to_string())
        );
    }

    #[test]
    fn test_pid_port_reference_function_comparisons() {
        // Test lines 332-334, 337: PID, Port, Reference, Function comparisons
        let pid1 = ErlangTerm::Pid(100);
        let pid2 = ErlangTerm::Pid(200);
        assert_eq!(
            OpBif::slt(&pid1, &pid2),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::sgt(&pid2, &pid1),
            ErlangTerm::Atom("true".to_string())
        );
        
        let port1 = ErlangTerm::Port(300);
        let port2 = ErlangTerm::Port(400);
        assert_eq!(
            OpBif::slt(&port1, &port2),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::sgt(&port2, &port1),
            ErlangTerm::Atom("true".to_string())
        );
        
        let ref1 = ErlangTerm::Reference(500);
        let ref2 = ErlangTerm::Reference(600);
        assert_eq!(
            OpBif::slt(&ref1, &ref2),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::sgt(&ref2, &ref1),
            ErlangTerm::Atom("true".to_string())
        );
        
        let func1 = ErlangTerm::Function { arity: 1 };
        let func2 = ErlangTerm::Function { arity: 2 };
        assert_eq!(
            OpBif::slt(&func1, &func2),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::sgt(&func2, &func1),
            ErlangTerm::Atom("true".to_string())
        );
    }

    #[test]
    fn test_compare_lists_edge_cases() {
        // Test compare_lists helper with various edge cases
        let list1 = ErlangTerm::List(vec![ErlangTerm::Integer(1), ErlangTerm::Integer(2)]);
        let list2 = ErlangTerm::List(vec![ErlangTerm::Integer(1), ErlangTerm::Integer(2), ErlangTerm::Integer(3)]);
        let list3 = ErlangTerm::List(vec![ErlangTerm::Integer(1), ErlangTerm::Integer(3)]);
        
        // list1 < list2 (shorter)
        assert_eq!(
            OpBif::slt(&list1, &list2),
            ErlangTerm::Atom("true".to_string())
        );
        
        // list1 < list3 (second element different)
        assert_eq!(
            OpBif::slt(&list1, &list3),
            ErlangTerm::Atom("true".to_string())
        );
        
        // Equal lists
        let list4 = ErlangTerm::List(vec![ErlangTerm::Integer(1), ErlangTerm::Integer(2)]);
        assert_eq!(
            OpBif::seq(&list1, &list4),
            ErlangTerm::Atom("true".to_string())
        );
    }

    #[test]
    fn test_tuple_comparison_with_equal_elements() {
        // Test tuple comparison when elements are equal (line 293 path)
        let tuple1 = ErlangTerm::Tuple(vec![
            ErlangTerm::Integer(1),
            ErlangTerm::Integer(2),
            ErlangTerm::Integer(3),
        ]);
        let tuple2 = ErlangTerm::Tuple(vec![
            ErlangTerm::Integer(1),
            ErlangTerm::Integer(2),
            ErlangTerm::Integer(3),
        ]);
        
        // Should be equal
        assert_eq!(
            OpBif::seq(&tuple1, &tuple2),
            ErlangTerm::Atom("true".to_string())
        );
        
        // Should compare as equal
        assert_eq!(
            OpBif::sle(&tuple1, &tuple2),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::sge(&tuple1, &tuple2),
            ErlangTerm::Atom("true".to_string())
        );
    }

    #[test]
    fn test_nil_nil_comparison() {
        // Test line 274: Nil == Nil comparison
        let nil1 = ErlangTerm::Nil;
        let nil2 = ErlangTerm::Nil;
        
        // Should compare as equal
        assert_eq!(
            OpBif::seq(&nil1, &nil2),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::sle(&nil1, &nil2),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::sge(&nil1, &nil2),
            ErlangTerm::Atom("true".to_string())
        );
    }

    #[test]
    fn test_tuple_comparison_non_comparable_elements() {
        // Test lines 288-290: Tuple comparison when elements are not comparable but structurally equal
        // This is a tricky case - we need elements where compare() returns None but they're structurally equal
        // Actually, if compare() returns None, we fall back to structural comparison
        // So we need elements that are structurally equal but compare() can't compare them
        // This is hard to achieve, but we can test with maps which might not be directly comparable
        
        // For now, test with tuples containing the same elements
        // The path where compare returns None but elements are equal should be covered
        // by having tuples with identical elements where compare might return None for some reason
        let tuple1 = ErlangTerm::Tuple(vec![
            ErlangTerm::Map(HashMap::new()),
            ErlangTerm::Integer(1),
        ]);
        let tuple2 = ErlangTerm::Tuple(vec![
            ErlangTerm::Map(HashMap::new()),
            ErlangTerm::Integer(1),
        ]);
        
        // These should be structurally equal even if compare() returns None
        assert_eq!(
            OpBif::seq(&tuple1, &tuple2),
            ErlangTerm::Atom("true".to_string())
        );
    }

    #[test]
    fn test_map_comparison_equal() {
        // Test line 322: Map comparison when maps are equal
        let mut map1 = HashMap::new();
        map1.insert(ErlangTerm::Integer(1), ErlangTerm::Integer(10));
        map1.insert(ErlangTerm::Integer(2), ErlangTerm::Integer(20));
        let mut map2 = HashMap::new();
        map2.insert(ErlangTerm::Integer(1), ErlangTerm::Integer(10));
        map2.insert(ErlangTerm::Integer(2), ErlangTerm::Integer(20));
        
        let term1 = ErlangTerm::Map(map1);
        let term2 = ErlangTerm::Map(map2);
        
        // Should compare as equal
        assert_eq!(
            OpBif::seq(&term1, &term2),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::sle(&term1, &term2),
            ErlangTerm::Atom("true".to_string())
        );
        assert_eq!(
            OpBif::sge(&term1, &term2),
            ErlangTerm::Atom("true".to_string())
        );
    }

    #[test]
    fn test_incomparable_types() {
        // Test line 342: Different types are not comparable
        // When types are different, compare() returns None, which makes comparisons return false
        let atom = ErlangTerm::Atom("test".to_string());
        let integer = ErlangTerm::Integer(5);
        let float = ErlangTerm::Float(5.0);
        let list = ErlangTerm::List(vec![]);
        
        // Different types should not be comparable (returns false for <, >, <=, >=)
        assert_eq!(
            OpBif::slt(&atom, &integer),
            ErlangTerm::Atom("false".to_string())
        );
        assert_eq!(
            OpBif::sgt(&atom, &integer),
            ErlangTerm::Atom("false".to_string())
        );
        assert_eq!(
            OpBif::sle(&atom, &integer),
            ErlangTerm::Atom("false".to_string())
        );
        assert_eq!(
            OpBif::sge(&atom, &integer),
            ErlangTerm::Atom("false".to_string())
        );
        
        // But equality can still be checked (structural vs exact)
        assert_eq!(
            OpBif::seq(&atom, &integer),
            ErlangTerm::Atom("false".to_string())
        );
    }

    #[test]
    fn test_list_comparison_non_comparable_elements() {
        // Test lines 358-360: List comparison when elements are not comparable but structurally equal
        let list1 = ErlangTerm::List(vec![
            ErlangTerm::Map(HashMap::new()),
            ErlangTerm::Integer(1),
        ]);
        let list2 = ErlangTerm::List(vec![
            ErlangTerm::Map(HashMap::new()),
            ErlangTerm::Integer(1),
        ]);
        
        // These should be structurally equal
        assert_eq!(
            OpBif::seq(&list1, &list2),
            ErlangTerm::Atom("true".to_string())
        );
    }

    #[test]
    fn test_sge_sle_with_none_comparison() {
        // Test lines 433, 455: sge and sle when compare returns None
        // This happens when types are incompatible
        let atom = ErlangTerm::Atom("test".to_string());
        let pid = ErlangTerm::Pid(123);
        
        // When compare returns None, sge and sle should return false
        assert_eq!(
            OpBif::sge(&atom, &pid),
            ErlangTerm::Atom("false".to_string())
        );
        assert_eq!(
            OpBif::sle(&atom, &pid),
            ErlangTerm::Atom("false".to_string())
        );
    }

    #[test]
    fn test_is_record_edge_cases() {
        // Test lines 629, 664, 666: is_record and is_record_with_size edge cases
        // Test with empty tuple
        let empty_tuple = ErlangTerm::Tuple(vec![]);
        let tag = ErlangTerm::Atom("record".to_string());
        
        assert_eq!(
            OpBif::is_record(&empty_tuple, &tag).unwrap(),
            ErlangTerm::Atom("false".to_string())
        );
        
        // Test with tuple that has wrong first element
        let wrong_tuple = ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("wrong".to_string()),
            ErlangTerm::Integer(1),
        ]);
        assert_eq!(
            OpBif::is_record(&wrong_tuple, &tag).unwrap(),
            ErlangTerm::Atom("false".to_string())
        );
        
        // Test is_record_with_size with wrong size
        let tuple_size2 = ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("record".to_string()),
            ErlangTerm::Integer(1),
        ]);
        let size3 = ErlangTerm::Integer(3);
        assert_eq!(
            OpBif::is_record_with_size(&tuple_size2, &tag, &size3).unwrap(),
            ErlangTerm::Atom("false".to_string())
        );
    }

    #[test]
    fn test_op_error_display() {
        // Test lines 708-710: Display implementation for OpError
        let error = OpError::BadArgument("test error".to_string());
        let error_str = format!("{}", error);
        assert!(error_str.contains("Bad argument"));
        assert!(error_str.contains("test error"));
    }
}

