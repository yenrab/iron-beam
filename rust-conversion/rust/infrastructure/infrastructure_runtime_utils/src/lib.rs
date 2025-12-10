//! Infrastructure Layer: Runtime Utilities
//!
//! Provides runtime utility functions from utils.c for the Erlang/OTP runtime system.
//! This crate implements term building, comparison, and initialization utilities
//! used throughout the runtime.
//!
//! ## Overview
//!
//! The `infrastructure_runtime_utils` crate is part of the infrastructure layer in the
//! CLEAN architecture implementation of Erlang/OTP. It provides utility functions from
//! `utils.c` that are used for:
//! - Building Erlang terms (atoms, integers, tuples, lists, etc.)
//! - Comparing Erlang terms (eq, erts_cmp)
//! - Runtime initialization
//! - General-purpose runtime utilities
//!
//! ## Modules
//!
//! - **[`term_building`](term_building/index.html)**: Term building functions
//!   (erts_bld_atom, erts_bld_uint, erts_bld_tuple, etc.)
//!
//! - **[`comparison`](comparison/index.html)**: Term comparison functions
//!   (eq, erts_cmp)
//!
//! - **[`initialization`](initialization/index.html)**: Runtime initialization functions
//!   (erts_init_utils, erts_init_utils_mem)
//!
//! ## Architecture
//!
//! This crate is based on the C implementation in `utils.c`. It depends on:
//! - `entities_data_handling` for term types
//! - `entities_process` for process-related types
//! - `entities_utilities` for utility types
//!
//! ## See Also
//!
//! - [`entities_data_handling`](../../entities/entities_data_handling/index.html): Core term types
//! - [`infrastructure_utilities`](../infrastructure_utilities/index.html): Other utility functions

pub mod term_building;
pub mod comparison;
pub mod initialization;

pub use term_building::{
    erts_bld_atom, erts_bld_uint, erts_bld_uword, erts_bld_uint64, erts_bld_sint64,
    erts_bld_cons, erts_bld_tuple, erts_bld_tuplev, erts_bld_string_n, erts_bld_list,
    erts_bld_2tup_list, erts_bld_atom_uword_2tup_list, erts_bld_atom_2uint_3tup_list,
    TermBuildingError, HeapBuilder,
};
pub use comparison::{eq, erts_cmp, ComparisonError};
pub use initialization::{erts_init_utils, erts_init_utils_mem, erts_utils_sched_spec_data_init};

