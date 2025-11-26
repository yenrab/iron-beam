//! Entities Layer: Common System Integration
//!
//! Provides common system integration functionality:
//! - Memory mapping operations
//!
//! Based on erl_mmap.c

pub mod mmap;

pub use mmap::MemoryMap;

