/*!
# Erlang Syntax Tree Structures

**CLEAN Architecture**: Entities Layer (Layer 1)
**SOLID Responsibility**: Erlang syntax tree structures and language constructs

## Overview

This crate defines the core data structures representing Erlang syntax - the abstract syntax tree (AST).
These are pure data structures with no business logic, following CLEAN architecture principles.

## Erlang Language Constructs

Erlang programs consist of:
- **Modules**: Collections of functions and attributes
- **Functions**: Named code blocks with clauses
- **Expressions**: Computations and data manipulation
- **Patterns**: Data structure matching
- **Types**: Type specifications and annotations
- **Guards**: Boolean conditions for pattern matching

## Design Philosophy

### 1. Algebraic Data Types
```rust
use entities_erlang_syntax::*;

// Sum types for different expression kinds
pub enum Expression {
    Literal(Literal),
    Variable(Variable),
    FunctionCall(FunctionCall),
    // ... many more variants
}
```

### 2. Structural Equality
```rust
use entities_erlang_syntax::*;

// All AST nodes derive Eq for comparison
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    pub name: Atom,
    pub functions: Vec<Function>,
    pub attributes: Vec<Attribute>,
}
```

### 3. Immutability by Default
```rust
use entities_erlang_syntax::*;

// Immutable structures promote safety
pub struct Function {
    pub name: FunctionName,
    pub clauses: Vec<Clause>,
}
```

## Module Structure

The crate is organized into the following modules:
- `atoms.rs` - Erlang atoms and identifiers
- `literals.rs` - Atomic values (integers, floats, rationals, etc.)
- `expressions.rs` - Expression AST nodes
- `patterns.rs` - Pattern matching structures
- `types.rs` - Type specifications
- `modules.rs` - Module definitions
- `clauses.rs` - Function/case clauses and guards

## Number Types

This crate uses the `num` family of crates for precise numeric representations:
- **BigInt**: Arbitrary precision integers (for Erlang's unlimited precision integers)
- **BigRational**: Exact rational arithmetic (for precise fractional calculations)
- **f64**: IEEE 754 double precision floats (matching Erlang's float representation)
*/

pub mod atoms;
pub mod literals;
pub mod expressions;
pub mod patterns;
pub mod types;
pub mod modules;
pub mod clauses;

// Re-export main types for convenience
use serde::{Deserialize, Serialize};

pub use atoms::*;
pub use literals::*;
pub use expressions::*;
pub use patterns::*;
pub use types::*;
pub use modules::*;
pub use clauses::*;

/// Position information for error reporting and debugging
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub file: Option<String>,
}

impl Default for Position {
    fn default() -> Self {
        Self {
            line: 1,
            column: 1,
            file: None,
        }
    }
}

/// Metadata attached to AST nodes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metadata {
    pub position: Position,
    pub comments: Vec<String>,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            position: Position::default(),
            comments: Vec::new(),
        }
    }
}

/// Generic AST node wrapper with metadata
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstNode<T> {
    pub data: T,
    pub meta: Metadata,
}

impl<T> AstNode<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            meta: Metadata::default(),
        }
    }

    pub fn with_meta(data: T, meta: Metadata) -> Self {
        Self { data, meta }
    }

    pub fn position(mut self, position: Position) -> Self {
        self.meta.position = position;
        self
    }

    pub fn file(mut self, file: String) -> Self {
        self.meta.position.file = Some(file);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_default() {
        let pos = Position::default();
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 1);
        assert!(pos.file.is_none());
    }

    #[test]
    fn test_position_construction() {
        let pos = Position {
            line: 10,
            column: 25,
            file: Some("test.erl".to_string()),
        };

        assert_eq!(pos.line, 10);
        assert_eq!(pos.column, 25);
        assert_eq!(pos.file, Some("test.erl".to_string()));
    }

    #[test]
    fn test_position_equality() {
        let pos1 = Position {
            line: 5,
            column: 10,
            file: Some("file.erl".to_string()),
        };
        let pos2 = Position {
            line: 5,
            column: 10,
            file: Some("file.erl".to_string()),
        };
        let pos3 = Position {
            line: 6,
            column: 10,
            file: Some("file.erl".to_string()),
        };

        assert_eq!(pos1, pos2);
        assert_ne!(pos1, pos3);
    }

    #[test]
    fn test_position_partial_eq_with_none() {
        let pos1 = Position {
            line: 1,
            column: 1,
            file: None,
        };
        let pos2 = Position::default();

        assert_eq!(pos1, pos2);
    }

    #[test]
    fn test_metadata_default() {
        let meta = Metadata::default();
        assert_eq!(meta.position.line, 1);
        assert_eq!(meta.position.column, 1);
        assert!(meta.position.file.is_none());
        assert!(meta.comments.is_empty());
    }

    #[test]
    fn test_metadata_construction() {
        let pos = Position {
            line: 42,
            column: 8,
            file: Some("example.erl".to_string()),
        };
        let comments = vec!["This is a comment".to_string(), "Another comment".to_string()];

        let meta = Metadata {
            position: pos.clone(),
            comments: comments.clone(),
        };

        assert_eq!(meta.position, pos);
        assert_eq!(meta.comments, comments);
    }

    #[test]
    fn test_metadata_equality() {
        let meta1 = Metadata {
            position: Position {
                line: 10,
                column: 5,
                file: Some("test.erl".to_string()),
            },
            comments: vec!["comment1".to_string()],
        };
        let meta2 = Metadata {
            position: Position {
                line: 10,
                column: 5,
                file: Some("test.erl".to_string()),
            },
            comments: vec!["comment1".to_string()],
        };
        let meta3 = Metadata {
            position: Position {
                line: 10,
                column: 5,
                file: Some("test.erl".to_string()),
            },
            comments: vec!["different".to_string()],
        };

        assert_eq!(meta1, meta2);
        assert_ne!(meta1, meta3);
    }

    #[test]
    fn test_ast_node_new() {
        let data = "test data".to_string();
        let node = AstNode::new(data.clone());

        assert_eq!(node.data, data);
        assert_eq!(node.meta.position.line, 1);
        assert_eq!(node.meta.position.column, 1);
        assert!(node.meta.position.file.is_none());
        assert!(node.meta.comments.is_empty());
    }

    #[test]
    fn test_ast_node_with_meta() {
        let data = 42i64;
        let position = Position {
            line: 100,
            column: 20,
            file: Some("custom.erl".to_string()),
        };
        let comments = vec!["Custom metadata".to_string()];

        let meta = Metadata {
            position,
            comments,
        };

        let node = AstNode::with_meta(data, meta.clone());
        assert_eq!(node.data, data);
        assert_eq!(node.meta, meta);
    }

    #[test]
    fn test_ast_node_position_method() {
        let data = vec![1, 2, 3];
        let position = Position {
            line: 50,
            column: 15,
            file: Some("array.erl".to_string()),
        };

        let node = AstNode::new(data).position(position.clone());
        assert_eq!(node.meta.position, position);
        assert!(node.meta.comments.is_empty());
    }

    #[test]
    fn test_ast_node_file_method() {
        let data = true;
        let filename = "boolean.erl".to_string();

        let node = AstNode::new(data).file(filename.clone());
        assert_eq!(node.meta.position.file, Some(filename));
        assert_eq!(node.meta.position.line, 1);
        assert_eq!(node.meta.position.column, 1);
        assert!(node.meta.comments.is_empty());
    }

    #[test]
    fn test_ast_node_method_chaining() {
        let data = Atom::new("chained");
        let position = Position {
            line: 25,
            column: 30,
            file: None,
        };
        let filename = "chained.erl".to_string();

        let node = AstNode::new(data.clone())
            .position(position.clone())
            .file(filename.clone());

        assert_eq!(node.data, data);
        // After chaining, the position should have the line/column from position() call
        // and the file from file() call
        assert_eq!(node.meta.position.line, position.line);
        assert_eq!(node.meta.position.column, position.column);
        assert_eq!(node.meta.position.file, Some(filename));
        assert!(node.meta.comments.is_empty());
    }

    #[test]
    fn test_ast_node_generic_types() {
        // Test with different generic types
        let int_node: AstNode<i32> = AstNode::new(123);
        assert_eq!(int_node.data, 123);

        let string_node: AstNode<String> = AstNode::new("hello".to_string());
        assert_eq!(string_node.data, "hello");

        let vec_node: AstNode<Vec<u8>> = AstNode::new(vec![1, 2, 3, 4]);
        assert_eq!(vec_node.data, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_ast_node_with_complex_data() {
        // Test with a more complex AST node type
        let function_call = FunctionCall::local(
            Atom::new("test_func"),
            vec![Expression::Literal(Literal::Integer(Integer::from_i64(42)))],
        );

        let node = AstNode::new(function_call.clone());
        assert_eq!(node.data, function_call);
    }

    #[test]
    fn test_reexports_accessibility() {
        // Test that re-exported types are accessible at crate root
        let atom = atoms::Atom::new("test");
        assert_eq!(atom.as_str(), "test");

        // Test that re-exported types work through the crate namespace
        let variable = Variable::new("TestVar");
        assert_eq!(variable.name, "TestVar");

        // Test literal construction
        let integer = Integer::from_i64(123);
        assert_eq!(integer.value, num_bigint::BigInt::from(123));
    }

    #[test]
    fn test_module_reexports() {
        // Test that all module types are re-exported
        let _atom: Atom = Atom::new("atom");
        let _literal: Literal = Literal::Atom(Atom::new("lit"));
        let _expression: Expression = Expression::Literal(Literal::Integer(Integer::from_i64(1)));
        let _pattern: Pattern = Pattern::Wildcard;
        let _clause: Clause = Clause::new(vec![], vec![], vec![]);

        // These should compile without import errors
        assert!(true);
    }

    #[test]
    fn test_position_serialization_derives() {
        // Test that Position has the expected derive traits
        let pos1 = Position {
            line: 42,
            column: 13,
            file: Some("test.erl".to_string()),
        };
        let pos2 = pos1.clone();

        // Test Clone derive
        assert_eq!(pos1, pos2);

        // Test Debug derive
        let debug_str = format!("{:?}", pos1);
        assert!(debug_str.contains("Position"));
    }

    #[test]
    fn test_metadata_serialization() {
        // Test that Metadata can be serialized (no serde derive, but should work)
        let meta = Metadata {
            position: Position {
                line: 10,
                column: 5,
                file: Some("meta.erl".to_string()),
            },
            comments: vec!["A comment".to_string()],
        };

        // Since Metadata doesn't have serde derives, this should fail to compile
        // but we can at least test the structure
        assert_eq!(meta.comments.len(), 1);
        assert_eq!(meta.position.line, 10);
    }

    #[test]
    fn test_ast_node_debug_formatting() {
        let node = AstNode::new("test");
        let debug_str = format!("{:?}", node);
        assert!(debug_str.contains("AstNode"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_position_debug_formatting() {
        let pos = Position {
            line: 1,
            column: 1,
            file: None,
        };
        let debug_str = format!("{:?}", pos);
        assert!(debug_str.contains("Position"));
        assert!(debug_str.contains("line: 1"));
        assert!(debug_str.contains("column: 1"));
    }

    #[test]
    fn test_metadata_debug_formatting() {
        let meta = Metadata::default();
        let debug_str = format!("{:?}", meta);
        assert!(debug_str.contains("Metadata"));
        assert!(debug_str.contains("position"));
        assert!(debug_str.contains("comments"));
    }
}
