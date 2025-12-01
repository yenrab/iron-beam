//! Infrastructure Layer: Utilities
//!
//! Provides common utility functions and helpers.
//! This is a large module with 224 files and 1754 functions.
//! Includes process table/registry (erl_ptab.c) - NOT pure data storage.
//! Depends on Entities layer only (dependencies flow inward).

pub mod common;
pub mod helpers;
pub mod process_table;

pub use common::CommonUtils;
pub use helpers::HelperFunctions;
pub use process_table::{ProcessTable, get_global_process_table, ProcessTableError};

