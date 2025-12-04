//! Infrastructure Layer: ETS Tables
//!
//! Provides ETS (Erlang Term Storage) table implementation for the Erlang/OTP runtime
//! system. This crate implements the infrastructure for ETS tables, which provide
//! in-memory key-value storage for Erlang terms.
//!
//! ## Overview
//!
//! The `infrastructure_ets_tables` crate is part of the infrastructure layer in the CLEAN
//! architecture implementation of Erlang/OTP. It provides the infrastructure for ETS
//! tables, enabling efficient in-memory storage and retrieval of Erlang terms.
//!
//! ## Modules
//!
//! - **[`ets_table`](ets_table/index.html)**: ETS table implementation providing
//!   key-value storage with various table types (set, ordered_set, bag, duplicate_bag)
//!
//! ## Architecture
//!
//! This crate is based on the C implementation in `cgi_echo.c` and related ETS files.
//! It depends on the Entities and Adapters layers.
//!
//! ## See Also
//!
//! - [`adapters_ets_tables`](../../adapters/adapters_ets_tables/index.html): ETS table debugging adapters
//! - [`entities_data_handling`](../../entities/entities_data_handling/index.html): Term types for ETS

pub mod ets_table;

pub use ets_table::EtsTable;

