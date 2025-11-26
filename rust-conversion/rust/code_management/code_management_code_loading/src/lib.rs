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

pub use code_loader::CodeLoader;
pub use unicode::UnicodeHandler;

