//! Infrastructure Layer: Utilities
//!
//! Provides common utility functions and helpers.
//! This is a large module with 224 files and 1754 functions.
//! Includes process table/registry (erl_ptab.c) - NOT pure data storage.
//! Depends on Entities layer only (dependencies flow inward).

pub mod common;
pub mod helpers;

pub use common::CommonUtils;
pub use helpers::HelperFunctions;

