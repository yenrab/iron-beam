//! Infrastructure Layer: Time Management
//!
//! Provides time management functionality for the Erlang/OTP runtime system. This crate
//! implements infrastructure for time-related operations, including time supervision and
//! time-based scheduling.
//!
//! ## Overview
//!
//! The `infrastructure_time_management` crate is part of the infrastructure layer in the
//! CLEAN architecture implementation of Erlang/OTP. It provides time management infrastructure
//! that supports timer operations and time-based scheduling.
//!
//! ## Modules
//!
//! - **[`time_sup`](time_sup/index.html)**: Time supervision functionality for managing
//!   time-related operations and ensuring time consistency across the runtime
//!
//! ## Architecture
//!
//! This crate is based on the C implementation in `erl_time_sup.c`. It depends on the
//! Entities and Adapters layers.
//!
//! ## See Also
//!
//! - [`adapters_time_management`](../../adapters/adapters_time_management/index.html): Time management adapters
//! - [`entities_data_handling`](../../entities/entities_data_handling/index.html): Term types for time operations

pub mod time_sup;

pub use time_sup::TimeSup;

