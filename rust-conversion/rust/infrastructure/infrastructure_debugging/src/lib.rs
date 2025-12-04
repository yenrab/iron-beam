//! Infrastructure Layer: Debugging
//!
//! Provides debugging utilities for the Erlang/OTP runtime system. This crate implements
//! infrastructure for debugging operations, including debug output, term display, and
//! debug state management.
//!
//! ## Overview
//!
//! The `infrastructure_debugging` crate is part of the infrastructure layer in the CLEAN
//! architecture implementation of Erlang/OTP. It provides debugging infrastructure that
//! supports runtime inspection and debugging operations.
//!
//! ## Modules
//!
//! - **[`debug_utils`](debug_utils/index.html)**: Debugging utility functions including:
//!   - Debug output utilities (similar to printf debugging)
//!   - Term display and formatting (similar to `ptd()` in C)
//!   - Paranoid display for corrupted data structures
//!   - Debug state management
//!   - Integration with debugging adapters
//!
//! ## Architecture
//!
//! This crate is based on the C implementation in `erl_debug.c` and `beam_debug.c`. It
//! depends on the Entities, Infrastructure, and Adapters layers.
//!
//! ## See Also
//!
//! - [`adapters_debugging`](../../adapters/adapters_debugging/index.html): Debugging adapters
//! - [`usecases_bifs`](../../usecases/usecases_bifs/index.html): Trace BIF implementations

pub mod debug_utils;

pub use debug_utils::{DebugUtils, DebugError};

