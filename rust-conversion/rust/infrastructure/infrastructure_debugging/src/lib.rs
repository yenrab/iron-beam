//! Infrastructure Layer: Debugging
//!
//! Provides debugging utilities.
//! Based on erl_debug.c and beam_debug.c
//! Depends on Entities, Infrastructure, and Adapters layers.
//!
//! This crate provides infrastructure for debugging:
//! - Debug output utilities (similar to printf debugging)
//! - Term display and formatting (similar to ptd() in C)
//! - Paranoid display for corrupted data structures
//! - Debug state management
//! - Integration with debugging adapters

pub mod debug_utils;

pub use debug_utils::{DebugUtils, DebugError};

