//! Serialization Interfaces and Cross-Cutting Concerns
//!
//! This crate provides serialization interfaces and handles cross-cutting
//! serialization concerns in the CLEAN architecture. It centralizes all
//! serialization logic including compression, encoding formats, and data
//! transformation between layers.
//!
//! ## Overview
//!
//! The `interfaces_serialization` crate is part of the Interface Adapters layer
//! in CLEAN architecture. It provides:
//!
//! - **Compression algorithms**: Zlib, Zstd for data compression/decompression
//! - **Serialization formats**: Centralized format handling and conversion
//! - **Cross-cutting concerns**: Data transformation between architectural layers
//! - **Error handling**: Unified error types for serialization operations
//!
//! ## Architecture
//!
//! This crate depends on entities for data types but does not depend on
//! higher-level business logic. It serves as the bridge between the domain
//! model and external representations.
//!
//! ## Modules
//!
//! - **[`compression`](compression/index.html)**: Compression and decompression
//!   algorithms (Zlib, Zstd)
//!
//! ## See Also
//!
//! - [`entities_data_handling`](../../entities/entities_data_handling/index.html): Core data types
//! - [`interfaces_compiler_api`](../interfaces_compiler_api/index.html): API interfaces

pub mod compression;

pub use compression::*;
