//! Entities Layer: Utilities
//!
//! Provides utility functions:
//! - Big number operations
//! - Register handling
//!
//! Based on big.c and register.c

pub mod big;
pub mod register;

pub use big::BigNumber;
pub use register::Register;

