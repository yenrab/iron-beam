//! Use Cases Layer: Process Management
//!
//! Provides process management functionality:
//! - Process locks
//! - Process dumps
//! - Process dictionaries
//!
//! Based on erl_process_lock.c, erl_process_dump.c, erl_process_dict.c
//! Depends on Entities and Infrastructure layers (process table/registry).

pub mod process_lock;
pub mod process_dump;
pub mod process_dict;

pub use process_lock::ProcessLock;
pub use process_dict::ProcessDict;
pub use process_dump::ProcessDump;

