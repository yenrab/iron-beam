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
