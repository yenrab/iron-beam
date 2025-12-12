//! Entities Layer: Process Management
//!
//! Provides process management entities for the Erlang/OTP runtime system. This crate
//! implements the core `Process` structure and related types that represent Erlang
//! processes in the runtime.
//!
//! ## Overview
//!
//! The `entities_process` crate is part of the entities layer in the CLEAN architecture
//! implementation of Erlang/OTP. It provides the fundamental data structures for
//! representing Erlang processes, including their heap, stack, and state information.
//!
//! ## Key Features
//!
//! - **Process Structure**: Core `Process` type representing an Erlang process
//! - **Process State**: Enumeration of all possible process states (Active, Running, Suspended, etc.)
//! - **Safe Heap Management**: Heap implemented using safe Rust `Vec<Eterm>` with index-based access
//! - **Type Safety**: Process ID and Eterm type aliases for type safety
//!
//! ## Safety
//!
//! This implementation prioritizes safety by using safe Rust data structures instead
//! of raw pointers. The heap is stored as a `Vec<Eterm>` with index-based access,
//! eliminating the need for unsafe code and raw pointer manipulation.
//!
//! ## Examples
//!
//! ```rust
//! use entities_process::{Process, ProcessState, ProcessId};
//!
//! // Create a new process
//! let process = Process::new(process_id, initial_heap_size);
//!
//! // Check process state
//! let state = process.get_state();
//! match state {
//!     ProcessState::Active => println!("Process is active"),
//!     ProcessState::Running => println!("Process is running"),
//!     _ => {}
//! }
//! ```
//!
//! ## See Also
//!
//! - [`entities_data_handling`](../entities_data_handling/index.html): Term types used in process heap
//! - [`usecases_process_management`](../../usecases/usecases_process_management/index.html): Process management use cases

pub mod process;
pub mod process_executor;

// Re-export main types for convenience
pub use process::{Process, ProcessId, ProcessState, Eterm, ErtsCodePtr};
pub use process_executor::{ProcessExecutor, ProcessExecutionResult, set_process_executor, execute_process};
