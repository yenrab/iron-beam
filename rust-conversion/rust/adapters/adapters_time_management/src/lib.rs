//! Adapters Layer: Time Management
//!
//! Provides time management functionality for the Erlang/OTP runtime system. This
//! crate implements adapters for timer operations and time slice management, enabling
//! scheduling and time-based operations in the runtime.
//!
//! ## Overview
//!
//! The `adapters_time_management` crate is part of the adapters layer in the CLEAN
//! architecture implementation of Erlang/OTP. It provides I/O adapters for time
//! management operations.
//!
//! ## Modules
//!
//! - **[`timer`](timer/index.html)**: Timer operations for scheduling time-based
//!   events and callbacks
//!
//! - **[`timeslice`](timeslice/index.html)**: Time slice management for controlling
//!   process execution time and scheduling fairness
//!
//! ## Architecture
//!
//! This crate is based on the C implementation in `timer_drv.c` and `consume_timeslice_drv.c`.
//! It depends on the Entities layer for fundamental data types.
//!
//! ## See Also
//!
//! - [`infrastructure_time_management`](../../infrastructure/infrastructure_time_management/index.html): Time management infrastructure
//! - [`entities_data_handling`](../../entities/entities_data_handling/index.html): Term types for time operations

pub mod timer;
pub mod timeslice;

pub use timer::Timer;
pub use timeslice::TimeSlice;

