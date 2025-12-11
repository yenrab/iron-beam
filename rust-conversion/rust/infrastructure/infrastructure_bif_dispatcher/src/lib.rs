//! Infrastructure Layer: BIF Dispatcher
//!
//! Provides BIF (Built-In Function) dispatcher infrastructure for routing calls
//! to BIF implementations. This crate implements the dispatcher mechanism that
//! routes BIF calls from the emulator to the actual BIF implementations.
//!
//! ## Overview
//!
//! The `infrastructure_bif_dispatcher` crate is part of the infrastructure layer
//! in the CLEAN architecture implementation of Erlang/OTP. It provides the
//! dispatcher mechanism that routes BIF calls, handles trap exports, and
//! manages BIF call flow.
//!
//! ## Modules
//!
//! - **[`dispatcher`](dispatcher/index.html)**: Core BIF call dispatcher
//!   functions (call_bif, erts_call_dirty_bif)
//!
//! - **[`trap_handlers`](trap_handlers/index.html)**: Trap handlers for BIF
//!   return traps, signal handling, and await exit traps
//!
//! - **[`initialization`](initialization/index.html)**: BIF system initialization
//!   and trap export setup
//!
//! - **[`registry`](registry/index.html)**: BIF registry for storing and
//!   looking up BIF functions by module, function name, and arity
//!
//! - **[`scheduling`](scheduling/index.html)**: Helper functions for scheduling
//!   BIFs, trap preparation, and yield handling
//!
//! ## Architecture
//!
//! This crate is based on the C implementation in `bif.c`. It depends on:
//! - `infrastructure_bifs` for BIF infrastructure framework
//! - `usecases_bifs` for actual BIF implementations
//! - `entities_process` for Process structures
//!
//! The dispatcher routes calls from the emulator to BIF implementations in
//! the usecases layer, maintaining separation of concerns.
//!
//! ## See Also
//!
//! - [`infrastructure_bifs`](../infrastructure_bifs/index.html): BIF infrastructure framework
//! - [`usecases_bifs`](../../usecases/usecases_bifs/index.html): BIF implementations
//! - [`entities_process`](../../entities/entities_process/index.html): Process entities

pub mod dispatcher;
pub mod trap_handlers;
pub mod initialization;
pub mod registry;
pub mod scheduling;

pub use dispatcher::{call_bif, erts_call_dirty_bif, BifDispatcher, BifDispatcherError};
pub use trap_handlers::{bif_return_trap, bif_handle_signals_return, erts_internal_await_exit_trap};
pub use initialization::{erts_init_bif, erts_init_trap_export, TrapExport, BifInitError};
pub use registry::{BifRegistry, BifKey, get_global_registry};
pub use scheduling::{SchedType, prepare_trap, prepare_trap_with_args, prepare_yield_return, is_proc_out_of_reds, reds_left};


