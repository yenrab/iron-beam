//! Code Management Layer: Code Loading
//!
//! Provides code loading and management functionality:
//! - Module loading
//! - Code organization
//! - Unicode handling
//! - Code save/restore
//!
//! Based on erl_unicode.c and related code loading files.
//! Depends on Entities, Use Cases, and Infrastructure layers.

pub mod code_loader;
pub mod unicode;
pub mod code_save_restore;
pub mod module_management;
pub mod code_index;
pub mod beam_loader;

pub use code_loader::CodeLoader;
pub use unicode::UnicodeHandler;
pub use module_management::{ModuleTableManager, ModuleTable, Module, ModuleInstance};
pub use code_index::{CodeIndexManager, CodeIndex, get_global_code_ix, NUM_CODE_IX};
pub use beam_loader::{BeamLoader, BeamFile, BeamFileReadResult, BeamLoadError};

