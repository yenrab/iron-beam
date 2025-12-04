//! Infrastructure Layer: BIFs Infrastructure
//!
//! Provides infrastructure for built-in functions (BIFs) in the Erlang/OTP runtime system.
//! This crate implements the framework and mechanisms that support BIF operations, including
//! initialization, state management, and error handling.
//!
//! ## Overview
//!
//! The `infrastructure_bifs` crate is part of the infrastructure layer in the CLEAN
//! architecture implementation of Erlang/OTP. It provides the infrastructure framework
//! for BIFs, while the actual BIF implementations are in the usecases layer.
//!
//! ## Key Features
//!
//! - **BIF System Initialization**: Framework for initializing the BIF system
//! - **BIF State Management**: Managing BIF state and lifecycle
//! - **BIF Error Handling**: Error handling infrastructure for BIF operations
//!
//! ## Architecture
//!
//! This crate is based on the C implementation in `bif.c`. It depends on the Entities
//! and Use Cases layers. Note that actual BIF implementations are in the usecases layer
//! (`usecases_bifs`); this infrastructure layer provides the framework and mechanisms.
//!
//! ## See Also
//!
//! - [`usecases_bifs`](../../usecases/usecases_bifs/index.html): BIF implementations
//! - [`api_facades`](../../api_facades/index.html): BIF facade layer

pub mod bif_infra;

pub use bif_infra::{BifInfrastructure, BifState, BifError};

