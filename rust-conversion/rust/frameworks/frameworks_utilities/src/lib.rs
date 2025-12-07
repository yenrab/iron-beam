//! Frameworks Layer: Utilities
//!
//! Provides framework-level utility functions for the Erlang/OTP runtime system. This crate
//! implements utility functions that are used at the framework level, including execution
//! and process management utilities.
//!
//! ## Overview
//!
//! The `frameworks_utilities` crate is part of the frameworks layer in the CLEAN
//! architecture implementation of Erlang/OTP. It provides framework-level utility
//! functions that support runtime execution and management.
//!
//! ## Modules
//!
//! - **[`framework_utils`](framework_utils/index.html)**: Framework-level utility functions
//!   including execution utilities and process management helpers
//!
//! ## Architecture
//!
//! This crate is based on the C implementation in `win_erlexec.c` and related utility files.
//! It depends on the Entities and Infrastructure layers.
//!
//! ## See Also
//!
//! - [`infrastructure_utilities`](../../infrastructure/infrastructure_utilities/index.html): Infrastructure utilities
//! - [`entities_utilities`](../../entities/entities_utilities/index.html): Entity utilities

pub mod framework_utils;

pub use framework_utils::{FrameworkUtils, FrameworkError};

