//! Use Cases Layer: Process Management
//!
//! Provides process management functionality for the Erlang/OTP runtime system. This
//! crate implements business logic for process lifecycle management, including locking,
//! dumping, dictionary management, and code tracking.
//!
//! ## Overview
//!
//! The `usecases_process_management` crate is part of the use cases layer in the CLEAN
//! architecture implementation of Erlang/OTP. It provides business logic for managing
//! Erlang processes, including their state, memory, and code dependencies.
//!
//! ## Modules
//!
//! - **[`process_lock`](process_lock/index.html)**: Process locking mechanisms for
//!   thread-safe process access and state management
//!
//! - **[`process_dump`](process_dump/index.html)**: Process dumping functionality for
//!   debugging and inspection. Allows serialization of process state for analysis.
//!
//! - **[`process_dict`](process_dict/index.html)**: Process dictionary management.
//!   Processes maintain a dictionary of key-value pairs for storing process-local data.
//!
//! - **[`process_code_tracking`](process_code_tracking/index.html)**: Code tracking
//!   functionality for monitoring which modules and code areas processes are using.
//!   Essential for safe code loading and hot code swapping.
//!
//! ## Architecture
//!
//! This crate is based on the C implementation in `erl_process_lock.c`, `erl_process_dump.c`,
//! `erl_process_dict.c`, and `beam_bif_load.c`. It depends on both the Entities and
//! Infrastructure layers for process table and registry operations.
//!
//! ## See Also
//!
//! - [`entities_process`](../../entities/entities_process/index.html): Process entity layer
//! - [`infrastructure_code_loading`](../../infrastructure/infrastructure_code_loading/index.html): Code loading infrastructure

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

