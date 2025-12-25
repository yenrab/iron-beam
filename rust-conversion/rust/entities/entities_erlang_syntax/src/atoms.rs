/*!
# Erlang Atoms and Identifiers

Atoms are unique, immutable constants in Erlang. They are used for module names,
function names, and symbolic constants throughout the language.
*/

use std::fmt;
use serde::{Deserialize, Serialize};

/// Erlang atom - an immutable, unique symbolic constant
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Atom {
    /// The string representation of the atom
    pub name: String,
}

impl Atom {
    /// Create a new atom from a string
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
        }
    }

    /// Create an atom from a string slice
    pub fn from_str(s: &str) -> Self {
        Self::new(s)
    }

    /// Get the atom as a string slice
    pub fn as_str(&self) -> &str {
        &self.name
    }

    /// Check if this is a boolean atom (true/false)
    pub fn is_boolean(&self) -> bool {
        self.name == "true" || self.name == "false"
    }

    /// Convert to boolean if this is a boolean atom
    pub fn as_boolean(&self) -> Option<bool> {
        match self.name.as_str() {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        }
    }
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl From<String> for Atom {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for Atom {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

/// Variable identifier in Erlang
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, )]
pub struct Variable {
    /// Variable name (starts with uppercase or underscore)
    pub name: String,
}

impl Variable {
    /// Create a new variable
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
        }
    }

    /// Check if this is an anonymous variable (_)
    pub fn is_anonymous(&self) -> bool {
        self.name == "_"
    }

    /// Check if this is a don't care variable (_Name)
    pub fn is_dont_care(&self) -> bool {
        self.name.starts_with('_')
    }
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl From<String> for Variable {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

/// Function name identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, )]
pub struct FunctionName {
    pub atom: Atom,
    pub arity: usize,
}

impl FunctionName {
    pub fn new(atom: Atom, arity: usize) -> Self {
        Self { atom, arity }
    }

    pub fn from_str<S: Into<String>>(name: S, arity: usize) -> Self {
        Self {
            atom: Atom::new(name),
            arity,
        }
    }
}

impl fmt::Display for FunctionName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.atom, self.arity)
    }
}

/// Module-qualified function name (module:function/arity)
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, )]
pub struct QualifiedFunctionName {
    pub module: Atom,
    pub function: FunctionName,
}

impl QualifiedFunctionName {
    pub fn new(module: Atom, function: FunctionName) -> Self {
        Self { module, function }
    }

    pub fn from_strs(module: &str, function: &str, arity: usize) -> Self {
        Self {
            module: Atom::from_str(module),
            function: FunctionName::from_str(function, arity),
        }
    }
}

impl fmt::Display for QualifiedFunctionName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.module, self.function)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atom_creation() {
        let atom = Atom::new("hello");
        assert_eq!(atom.as_str(), "hello");
        assert_eq!(atom.to_string(), "hello");
    }

    #[test]
    fn test_atom_boolean() {
        assert!(Atom::new("true").is_boolean());
        assert!(Atom::new("false").is_boolean());
        assert!(!Atom::new("maybe").is_boolean());

        assert_eq!(Atom::new("true").as_boolean(), Some(true));
        assert_eq!(Atom::new("false").as_boolean(), Some(false));
        assert_eq!(Atom::new("maybe").as_boolean(), None);
    }

    #[test]
    fn test_atom_equality() {
        let atom1 = Atom::from_str("test");
        let atom2 = Atom::from_str("test");
        let atom3 = Atom::from_str("other");

        assert_eq!(atom1, atom2);
        assert_ne!(atom1, atom3);
    }

    #[test]
    fn test_variable_anonymous() {
        let anon = Variable::new("_");
        let dont_care = Variable::new("_Var");
        let normal = Variable::new("Var");

        assert!(anon.is_anonymous());
        assert!(dont_care.is_dont_care());
        assert!(!normal.is_anonymous());
        assert!(!normal.is_dont_care());
    }

    #[test]
    fn test_function_name() {
        let fname = FunctionName::from_str("add", 2);
        assert_eq!(fname.atom.as_str(), "add");
        assert_eq!(fname.arity, 2);
        assert_eq!(fname.to_string(), "add/2");
    }

    #[test]
    fn test_qualified_function_name() {
        let qfname = QualifiedFunctionName::from_strs("math", "add", 2);
        assert_eq!(qfname.module.as_str(), "math");
        assert_eq!(qfname.function.atom.as_str(), "add");
        assert_eq!(qfname.function.arity, 2);
        assert_eq!(qfname.to_string(), "math:add/2");
    }
}
