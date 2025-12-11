//! Frameworks Layer: Emulator Initialization
//!
//! Provides emulator initialization functions from erl_init.c for the Erlang/OTP runtime system.
//! This crate coordinates the initialization sequence of all runtime components.
//!
//! ## Overview
//!
//! The `frameworks_emulator_init` crate is part of the frameworks layer in the CLEAN
//! architecture implementation of Erlang/OTP. It provides the main initialization
//! entry point (`erl_start`) and coordinates the initialization of all runtime components
//! in the correct order.
//!
//! ## Modules
//!
//! - **[`early_init`](early_init/index.html)**: Early initialization phase
//!   (before main initialization)
//!
//! - **[`main_init`](main_init/index.html)**: Main initialization phase
//!   (coordinates all component initialization)
//!
//! - **[`initialization`](initialization/index.html)**: Initialization state management
//!
//! ## Architecture
//!
//! This crate is based on the C implementation in `erl_init.c`. It coordinates
//! initialization across all layers:
//! - Entities layer: Core data structures
//! - Infrastructure layer: Runtime utilities, scheduling, BIF dispatcher
//! - Use cases layer: Process management, scheduling
//!
//! ## Initialization Sequence
//!
//! 1. **Early Initialization** (`early_init`):
//!    - Parse command line arguments
//!    - Initialize memory allocators
//!    - Set up thread progress
//!    - Initialize CPU topology
//!
//! 2. **Main Initialization** (`erl_init`):
//!    - Initialize global literals
//!    - Initialize process management
//!    - Initialize scheduling
//!    - Initialize BIF dispatcher
//!    - Initialize emulator loop
//!    - Initialize all other components
//!
//! ## See Also
//!
//! - [`infrastructure_runtime_utils`](../../infrastructure/infrastructure_runtime_utils/index.html): Runtime utilities
//! - [`usecases_scheduling`](../../usecases/usecases_scheduling/index.html): Scheduling use cases
//! - [`infrastructure_emulator_loop`](../../infrastructure/infrastructure_emulator_loop/index.html): Emulator loop

pub mod early_init;
pub mod main_init;
pub mod initialization;

pub use early_init::{early_init, EarlyInitResult};
pub use main_init::{erl_init, erl_start, InitConfig, TimeWarpMode};
pub use initialization::{InitializationState, is_initialized, set_initialized};

