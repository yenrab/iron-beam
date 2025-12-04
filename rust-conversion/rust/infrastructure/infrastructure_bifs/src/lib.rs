//! Infrastructure Layer: BIFs Infrastructure
//!
//! Provides infrastructure for built-in functions.
//! Based on bif.c
//! Depends on Entities and Use Cases layers.
//!
//! This crate provides the infrastructure layer for BIFs:
//! - BIF system initialization
//! - BIF state management
//! - BIF error handling
//!
//! Note: Actual BIF implementations are in the usecases layer (usecases_bifs).
//! This infrastructure layer provides the framework and mechanisms.

pub mod bif_infra;

pub use bif_infra::{BifInfrastructure, BifState, BifError};

