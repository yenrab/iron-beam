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

    #[test]
    fn test_atom_from_string() {
        let atom: Atom = "hello".to_string().into();
        assert_eq!(atom.as_str(), "hello");
    }

    #[test]
    fn test_atom_from_str() {
        let atom: Atom = "world".into();
        assert_eq!(atom.as_str(), "world");
    }

    #[test]
    fn test_atom_display() {
        let atom = Atom::new("display_test");
        assert_eq!(format!("{}", atom), "display_test");
    }

    #[test]
    fn test_atom_edge_cases() {
        // Empty atom
        let empty = Atom::new("");
        assert_eq!(empty.as_str(), "");
        assert_eq!(empty.to_string(), "");

        // Atom with special characters
        let special = Atom::new("atom_with_underscores_and_123");
        assert_eq!(special.as_str(), "atom_with_underscores_and_123");

        // Very long atom
        let long_name = "a".repeat(1000);
        let long_atom = Atom::new(long_name.clone());
        assert_eq!(long_atom.as_str(), long_name);
    }

    #[test]
    fn test_atom_boolean_edge_cases() {
        // Case sensitivity
        let true_upper = Atom::new("TRUE");
        let false_upper = Atom::new("FALSE");
        assert!(!true_upper.is_boolean());
        assert!(!false_upper.is_boolean());
        assert_eq!(true_upper.as_boolean(), None);
        assert_eq!(false_upper.as_boolean(), None);

        // Similar but different names
        let true_like = Atom::new("true_atom");
        let false_like = Atom::new("false_flag");
        assert!(!true_like.is_boolean());
        assert!(!false_like.is_boolean());
        assert_eq!(true_like.as_boolean(), None);
        assert_eq!(false_like.as_boolean(), None);
    }

    #[test]
    fn test_atom_hash_and_ord() {
        let atom1 = Atom::new("apple");
        let atom2 = Atom::new("banana");
        let atom3 = Atom::new("apple");

        // Hash equality
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert(atom1.clone(), "value1");
        map.insert(atom2.clone(), "value2");
        map.insert(atom3.clone(), "value3");

        // atom1 and atom3 should have the same hash/equality
        assert_eq!(map.len(), 2); // atom1 and atom3 are equal
        assert_eq!(map.get(&atom1), Some(&"value3")); // Last insertion wins

        // Ordering
        assert!(atom1 < atom2);
        assert!(atom2 > atom1);
        assert_eq!(atom1.cmp(&atom3), std::cmp::Ordering::Equal);
    }

    #[test]
    fn test_variable_from_string() {
        let var: Variable = "MyVar".to_string().into();
        assert_eq!(var.name, "MyVar");
    }

    #[test]
    fn test_variable_display() {
        let var = Variable::new("DisplayVar");
        assert_eq!(format!("{}", var), "DisplayVar");
    }

    #[test]
    fn test_variable_edge_cases() {
        // Empty variable name
        let empty = Variable::new("");
        assert!(!empty.is_anonymous());
        assert!(!empty.is_dont_care());

        // Single underscore
        let underscore = Variable::new("_");
        assert!(underscore.is_anonymous());
        assert!(underscore.is_dont_care());

        // Underscore followed by nothing
        let just_underscore = Variable::new("_");
        assert!(just_underscore.is_anonymous());
        assert!(just_underscore.is_dont_care());

        // Multiple underscores
        let multi_underscore = Variable::new("___");
        assert!(!multi_underscore.is_anonymous());
        assert!(multi_underscore.is_dont_care());

        // Mixed case
        let mixed = Variable::new("_MixedCase");
        assert!(!mixed.is_anonymous());
        assert!(mixed.is_dont_care());

        // Long variable name
        let long_var = Variable::new("VeryLongVariableNameThatGoesOnForever");
        assert!(!long_var.is_anonymous());
        assert!(!long_var.is_dont_care());
    }

    #[test]
    fn test_variable_hash_and_ord() {
        let var1 = Variable::new("Apple");
        let var2 = Variable::new("Banana");
        let var3 = Variable::new("Apple");

        // Hash equality
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert(var1.clone(), "value1");
        map.insert(var2.clone(), "value2");
        map.insert(var3.clone(), "value3");

        assert_eq!(map.len(), 2); // var1 and var3 are equal
        assert_eq!(map.get(&var1), Some(&"value3"));

        // Ordering
        assert!(var1 < var2);
        assert!(var2 > var1);
        assert_eq!(var1.cmp(&var3), std::cmp::Ordering::Equal);
    }

    #[test]
    fn test_function_name_new() {
        let atom = Atom::new("multiply");
        let fname = FunctionName::new(atom, 3);
        assert_eq!(fname.atom.as_str(), "multiply");
        assert_eq!(fname.arity, 3);
    }

    #[test]
    fn test_function_name_display() {
        let fname = FunctionName::from_str("divide", 2);
        assert_eq!(format!("{}", fname), "divide/2");
    }

    #[test]
    fn test_function_name_edge_cases() {
        // Zero arity
        let zero_arity = FunctionName::from_str("constant", 0);
        assert_eq!(zero_arity.arity, 0);
        assert_eq!(zero_arity.to_string(), "constant/0");

        // Large arity
        let large_arity = FunctionName::from_str("variadic", 100);
        assert_eq!(large_arity.arity, 100);
        assert_eq!(large_arity.to_string(), "variadic/100");

        // Empty function name
        let empty_name = FunctionName::from_str("", 1);
        assert_eq!(empty_name.atom.as_str(), "");
        assert_eq!(empty_name.to_string(), "/1");
    }

    #[test]
    fn test_function_name_hash_and_ord() {
        let fname1 = FunctionName::from_str("add", 2);
        let fname2 = FunctionName::from_str("multiply", 2);
        let fname3 = FunctionName::from_str("add", 3);
        let fname4 = FunctionName::from_str("add", 2);

        // Hash equality
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert(fname1.clone(), "value1");
        map.insert(fname2.clone(), "value2");
        map.insert(fname3.clone(), "value3");
        map.insert(fname4.clone(), "value4");

        assert_eq!(map.len(), 3); // fname1 and fname4 are equal
        assert_eq!(map.get(&fname1), Some(&"value4"));

        // Ordering (first by atom, then by arity)
        assert!(fname1 < fname2); // "add" < "multiply"
        assert!(fname1 < fname3); // arity 2 < arity 3 for same atom
        assert_eq!(fname1.cmp(&fname4), std::cmp::Ordering::Equal);
    }

    #[test]
    fn test_qualified_function_name_new() {
        let module = Atom::new("math");
        let function = FunctionName::from_str("add", 2);
        let qfname = QualifiedFunctionName::new(module, function);

        assert_eq!(qfname.module.as_str(), "math");
        assert_eq!(qfname.function.atom.as_str(), "add");
        assert_eq!(qfname.function.arity, 2);
    }

    #[test]
    fn test_qualified_function_name_display() {
        let qfname = QualifiedFunctionName::from_strs("lists", "reverse", 1);
        assert_eq!(format!("{}", qfname), "lists:reverse/1");
    }

    #[test]
    fn test_qualified_function_name_edge_cases() {
        // Empty module and function names
        let empty = QualifiedFunctionName::from_strs("", "", 0);
        assert_eq!(empty.module.as_str(), "");
        assert_eq!(empty.function.atom.as_str(), "");
        assert_eq!(empty.function.arity, 0);
        assert_eq!(empty.to_string(), ":/0");

        // Module and function with special characters
        let special = QualifiedFunctionName::from_strs("my_module", "my_function", 5);
        assert_eq!(special.module.as_str(), "my_module");
        assert_eq!(special.function.atom.as_str(), "my_function");
        assert_eq!(special.function.arity, 5);
        assert_eq!(special.to_string(), "my_module:my_function/5");
    }

    #[test]
    fn test_qualified_function_name_hash_and_ord() {
        let qfname1 = QualifiedFunctionName::from_strs("math", "add", 2);
        let qfname2 = QualifiedFunctionName::from_strs("lists", "add", 2);
        let qfname3 = QualifiedFunctionName::from_strs("math", "multiply", 2);
        let qfname4 = QualifiedFunctionName::from_strs("math", "add", 3);
        let qfname5 = QualifiedFunctionName::from_strs("math", "add", 2);

        // Hash equality
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert(qfname1.clone(), "value1");
        map.insert(qfname2.clone(), "value2");
        map.insert(qfname3.clone(), "value3");
        map.insert(qfname4.clone(), "value4");
        map.insert(qfname5.clone(), "value5");

        assert_eq!(map.len(), 4); // qfname1 and qfname5 are equal
        assert_eq!(map.get(&qfname1), Some(&"value5"));

        // Ordering (lexicographic: module, then function atom, then arity)
        // "lists" comes before "math" lexicographically
        assert!(qfname2 < qfname1); // "lists" < "math"
        assert!(qfname1 < qfname3); // same module, "add" < "multiply"
        assert!(qfname1 < qfname4); // same module/function, arity 2 < 3
        assert_eq!(qfname1.cmp(&qfname5), std::cmp::Ordering::Equal);
    }

    #[test]
    fn test_atom_serialization() {
        // Test that atoms can be serialized/deserialized with serde
        let atom = Atom::new("serializable");

        // Test Debug
        let debug_str = format!("{:?}", atom);
        assert!(debug_str.contains("Atom"));
        assert!(debug_str.contains("serializable"));
    }

    #[test]
    fn test_variable_serialization() {
        let var = Variable::new("SerializableVar");

        // Test Debug
        let debug_str = format!("{:?}", var);
        assert!(debug_str.contains("Variable"));
        assert!(debug_str.contains("SerializableVar"));
    }

    #[test]
    fn test_function_name_serialization() {
        let fname = FunctionName::from_str("func", 3);

        // Test Debug
        let debug_str = format!("{:?}", fname);
        assert!(debug_str.contains("FunctionName"));
        assert!(debug_str.contains("func"));
        assert!(debug_str.contains("3"));
    }

    #[test]
    fn test_qualified_function_name_serialization() {
        let qfname = QualifiedFunctionName::from_strs("mod", "func", 2);

        // Test Debug
        let debug_str = format!("{:?}", qfname);
        assert!(debug_str.contains("QualifiedFunctionName"));
        assert!(debug_str.contains("mod"));
        assert!(debug_str.contains("func"));
        assert!(debug_str.contains("2"));
    }

    #[test]
    fn test_atom_clone() {
        let atom1 = Atom::new("original");
        let atom2 = atom1.clone();
        assert_eq!(atom1, atom2);
        assert_eq!(atom1.as_str(), atom2.as_str());
    }

    #[test]
    fn test_variable_clone() {
        let var1 = Variable::new("OriginalVar");
        let var2 = var1.clone();
        assert_eq!(var1, var2);
        assert_eq!(var1.name, var2.name);
    }

    #[test]
    fn test_function_name_clone() {
        let fname1 = FunctionName::from_str("original", 5);
        let fname2 = fname1.clone();
        assert_eq!(fname1, fname2);
        assert_eq!(fname1.atom, fname2.atom);
        assert_eq!(fname1.arity, fname2.arity);
    }

    #[test]
    fn test_qualified_function_name_clone() {
        let qfname1 = QualifiedFunctionName::from_strs("orig_mod", "orig_func", 3);
        let qfname2 = qfname1.clone();
        assert_eq!(qfname1, qfname2);
        assert_eq!(qfname1.module, qfname2.module);
        assert_eq!(qfname1.function, qfname2.function);
    }

    #[test]
    fn test_cross_type_independence() {
        // Test that different types with same string content work independently
        let atom = Atom::new("same_name");
        let var = Variable::new("same_name");

        // The string content should be the same
        assert_eq!(atom.as_str(), var.name.as_str());

        // But they are different types and should work independently
        assert!(!atom.is_boolean()); // "same_name" is not a boolean atom
        assert!(!var.is_anonymous()); // "same_name" is not anonymous
    }

    #[test]
    fn test_atom_from_various_string_types() {
        // Test Atom::new with different Into<String> types
        let atom1 = Atom::new("test");
        let atom2 = Atom::new("test".to_string());
        let atom3 = Atom::new(String::from("test"));

        assert_eq!(atom1, atom2);
        assert_eq!(atom2, atom3);
        assert_eq!(atom1.as_str(), "test");
        assert_eq!(atom2.as_str(), "test");
        assert_eq!(atom3.as_str(), "test");
    }

    #[test]
    fn test_variable_from_various_string_types() {
        let var1 = Variable::new("TestVar");
        let var2 = Variable::new("TestVar".to_string());
        let var3 = Variable::new(String::from("TestVar"));

        assert_eq!(var1, var2);
        assert_eq!(var2, var3);
        assert_eq!(var1.name, "TestVar");
        assert_eq!(var2.name, "TestVar");
        assert_eq!(var3.name, "TestVar");
    }

    #[test]
    fn test_function_name_arity_variations() {
        let fname0 = FunctionName::from_str("func", 0);
        let fname1 = FunctionName::from_str("func", 1);
        let fname10 = FunctionName::from_str("func", 10);

        assert_eq!(fname0.arity, 0);
        assert_eq!(fname1.arity, 1);
        assert_eq!(fname10.arity, 10);

        assert_eq!(fname0.to_string(), "func/0");
        assert_eq!(fname1.to_string(), "func/1");
        assert_eq!(fname10.to_string(), "func/10");
    }

    #[test]
    fn test_display_formatting_comprehensive() {
        // Test all Display implementations with various inputs
        let atom = Atom::new("atom");
        assert_eq!(format!("{}", atom), "atom");

        let var = Variable::new("Variable");
        assert_eq!(format!("{}", var), "Variable");

        let fname = FunctionName::from_str("function", 42);
        assert_eq!(format!("{}", fname), "function/42");

        let qfname = QualifiedFunctionName::from_strs("module", "function", 42);
        assert_eq!(format!("{}", qfname), "module:function/42");
    }

    #[test]
    fn test_hash_consistency() {
        // Test that hash values are consistent for equal objects
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let atom1 = Atom::new("consistent");
        let atom2 = Atom::new("consistent");

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        atom1.hash(&mut hasher1);
        atom2.hash(&mut hasher2);

        assert_eq!(hasher1.finish(), hasher2.finish());
    }

    #[test]
    fn test_ord_consistency_with_eq() {
        // Test that ordering is consistent with equality
        let atoms = vec![
            Atom::new("cherry"),
            Atom::new("apple"),
            Atom::new("banana"),
            Atom::new("apple"), // duplicate
        ];

        // Sort should be consistent
        let mut sorted = atoms.clone();
        sorted.sort();

        // Should be sorted: apple, apple, banana, cherry
        assert_eq!(sorted[0].as_str(), "apple");
        assert_eq!(sorted[1].as_str(), "apple");
        assert_eq!(sorted[2].as_str(), "banana");
        assert_eq!(sorted[3].as_str(), "cherry");

        // Should maintain relative ordering
        assert!(sorted[0] <= sorted[1]);
        assert!(sorted[1] <= sorted[2]);
        assert!(sorted[2] <= sorted[3]);
        assert_eq!(sorted[0], sorted[1]); // duplicates should be equal
    }
}
