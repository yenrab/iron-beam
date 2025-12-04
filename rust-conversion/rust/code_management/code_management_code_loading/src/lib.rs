//! Code Management Layer: Code Loading
//!
//! Provides code loading and management functionality for the Erlang/OTP runtime system.
//! This crate implements high-level code management operations including module loading,
//! code organization, Unicode handling, and code save/restore functionality.
//!
//! ## Overview
//!
//! The `code_management_code_loading` crate is part of the code management layer in the
//! CLEAN architecture implementation of Erlang/OTP. It provides comprehensive code loading
//! and management capabilities that coordinate between use cases and infrastructure layers.
//!
//! ## Modules
//!
//! - **[`code_loader`](code_loader/index.html)**: High-level code loading interface
//! - **[`unicode`](unicode/index.html)**: Unicode handling for code and string processing
//! - **[`code_save_restore`](code_save_restore/index.html)**: Code save and restore operations
//! - **[`module_management`](module_management/index.html)**: Module table management for
//!   tracking loaded modules and their instances
//! - **[`code_index`](code_index/index.html)**: Code index management for organizing and
//!   accessing code versions
//! - **[`beam_loader`](beam_loader/index.html)**: BEAM file loading and parsing
//! - **[`code_permissions`](code_permissions/index.html)**: Code permission management for
//!   controlling code access
//! - **[`code_barriers`](code_barriers/index.html)**: Code barriers for safe code loading
//!   and hot code swapping
//! - **[`beam_debug`](beam_debug/index.html)**: BEAM debugging and tracing functionality
//!
//! ## Architecture
//!
//! This crate is based on the C implementation in `erl_unicode.c` and related code loading
//! files. It depends on the Entities, Use Cases, and Infrastructure layers, coordinating
//! between them to provide high-level code management operations.
//!
//! ## See Also
//!
//! - [`infrastructure_code_loading`](../../infrastructure/infrastructure_code_loading/index.html): Low-level code loading
//! - [`usecases_nif_compilation`](../../usecases/usecases_nif_compilation/index.html): NIF compilation use cases

pub mod code_loader;
pub mod unicode;
pub mod code_save_restore;
pub mod module_management;
pub mod code_index;
pub mod beam_loader;
pub mod code_permissions;
pub mod code_barriers;
pub mod beam_debug;

pub use code_loader::CodeLoader;
pub use unicode::UnicodeHandler;
pub use module_management::{ModuleTableManager, ModuleTable, Module, ModuleInstance, get_global_module_manager};
pub use code_index::{CodeIndexManager, CodeIndex, get_global_code_ix, NUM_CODE_IX};
pub use beam_loader::{BeamLoader, BeamFile, BeamFileReadResult, BeamLoadError};
pub use code_permissions::{CodePermissionManager, ProcessId, get_global_code_permissions};
pub use code_barriers::{CodeBarrier, CodeBarrierManager, get_global_code_barriers, debug_require_code_barrier, debug_check_code_barrier};
pub use beam_debug::{BeamDebugTracer, get_global_debug_tracer, dbg_set_traced_mfa, dbg_is_traced_mfa, dbg_vtrace_mfa};

