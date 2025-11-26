//! Adapters Layer: Common System Integration
//!
//! Provides common system integration functionality:
//! - I/O checking
//!
//! Based on erl_check_io.c
//! Depends on Entities layer.

pub mod check_io;

pub use check_io::CheckIo;

