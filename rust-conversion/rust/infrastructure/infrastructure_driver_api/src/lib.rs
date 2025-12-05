//! Infrastructure Layer: Erlang Driver API
//!
//! Provides the Rust Driver API - equivalent to C `erl_driver.h` API but implemented in pure Rust.
//! This module provides infrastructure for Erlang drivers, including port management, I/O operations,
//! queue management, and event selection.
//!
//! ## Overview
//!
//! The `infrastructure_driver_api` crate is part of the infrastructure layer in the CLEAN
//! architecture implementation of Erlang/OTP. It provides the driver API infrastructure that
//! adapters use to interact with the Erlang runtime system.
//!
//! ## Key Features
//!
//! - Port management (ErlDrvPort)
//! - Driver data types (ErlDrvData, ErlDrvEvent, ErlDrvSizeT)
//! - I/O operations (driver_output, driver_outputv)
//! - Queue management (driver_enq, driver_deq, driver_sizeq, driver_peekq, driver_enqv)
//! - Event selection (driver_select)
//! - Failure handling (driver_failure_*)
//! - Port control and initialization (erl_drv_init_ack, erl_drv_set_os_pid, erl_drv_port_control)
//! - Process data lock (driver_pdl_*)
//! - Async operations (driver_async_*)
//!
//! ## Architecture
//!
//! This is a new Rust implementation with no C code to be shipped. It replaces the C `erl_driver.h`
//! API with pure Rust functions. The C files are reference implementations only.
//!
//! ## Dependencies
//!
//! - `entities_data_handling` - For term type definitions
//! - `entities_utilities` - For utility types
//!
//! ## See Also
//!
//! - [`infrastructure_nif_api`](../infrastructure_nif_api/index.html): NIF API infrastructure
//! - [`adapters_system_integration_unix`](../../adapters/adapters_system_integration_unix/index.html): Uses driver API

pub mod types;
pub mod port;
pub mod queue;
pub mod select;
pub mod output;
pub mod failure;
pub mod control;

pub use types::*;
pub use port::*;
pub use queue::*;
pub use select::*;
pub use output::*;
pub use failure::*;
pub use control::*;


