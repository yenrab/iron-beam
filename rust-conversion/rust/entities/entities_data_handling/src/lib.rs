//! Entities Layer: Data Handling
//!
//! This crate provides core data handling functionality for Erlang terms:
//! - Term hashing (portable and internal hash functions)
//! - Atom table management
//! - Bit manipulation operations
//! - Binary operations
//! - Map operations
//! - Atomic operations
//!
//! This is the innermost layer of CLEAN architecture with no dependencies.

pub mod term_hashing;
pub mod atom;
pub mod bits;
pub mod binary;
pub mod map;
pub mod atomics;

// Re-export main types for convenience
pub use term_hashing::HashValue;
pub use atom::{AtomTable, AtomEncoding};
pub use map::{Map, MapError};

