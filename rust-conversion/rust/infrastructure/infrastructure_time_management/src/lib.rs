//! Infrastructure Layer: Time Management
//!
//! Provides time management functionality.
//! Based on erl_time_sup.c
//! Depends on Entities and Adapters layers.

pub mod time_sup;

pub use time_sup::TimeSup;

