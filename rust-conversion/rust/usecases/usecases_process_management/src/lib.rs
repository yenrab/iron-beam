//! Use Cases Layer: Process Management
//!
//! Provides process management functionality:
//! - Process locks
//! - Process dumps
//! - Process dictionaries
//! - Process code tracking
//!
//! Based on erl_process_lock.c, erl_process_dump.c, erl_process_dict.c, beam_bif_load.c
//! Depends on Entities and Infrastructure layers (process table/registry).

pub mod process_lock;
pub mod process_dump;
pub mod process_dict;
pub mod process_code_tracking;

pub use process_lock::ProcessLock;
pub use process_dict::ProcessDict;
pub use process_dump::ProcessDump;
pub use process_code_tracking::{
    check_process_uses_module,
    any_process_uses_module,
    any_dirty_process_uses_module,
    check_nif_in_module_area,
    check_continuation_pointers_in_module,
    pointer_in_module_area,
    ModuleCodeArea,
};

