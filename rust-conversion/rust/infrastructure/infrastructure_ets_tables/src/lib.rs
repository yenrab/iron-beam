//! Infrastructure Layer: ETS Tables
//!
//! Provides ETS (Erlang Term Storage) table implementation.
//! Based on cgi_echo.c and related ETS files.
//! Depends on Entities and Adapters layers.

pub mod ets_table;

pub use ets_table::EtsTable;

