//! Infrastructure Layer: Utilities
//!
//! Provides common utility functions and helpers for the Erlang/OTP runtime system. This
//! crate implements a comprehensive set of utility functions used throughout the runtime,
//! including process table management, formatting, math operations, and threading utilities.
//!
//! ## Overview
//!
//! The `infrastructure_utilities` crate is part of the infrastructure layer in the CLEAN
//! architecture implementation of Erlang/OTP. It provides a large collection of utility
//! functions that support various runtime operations.
//!
//! ## Modules
//!
//! - **[`common`](common/index.html)**: Common utility functions including:
//!   - Format utilities for string formatting
//!   - Math utilities for mathematical operations
//!   - Rational utilities for rational number operations
//!   - Hash utilities for hashing operations
//!   - Array utilities for array operations
//!   - Threading utilities for thread management
//!   - Time utilities for time operations
//!   - Path utilities for path manipulation
//!
//! - **[`helpers`](helpers/index.html)**: Helper functions for various runtime operations
//!
//! - **[`compression`](compression/index.html)**: Compression and decompression utilities
//!   using zlib (flate2) and zstd. Provides both chunked and one-shot interfaces.
//!
//! - **[`process_table`](process_table/index.html)**: Process table/registry implementation
//!   (based on `erl_ptab.c`). Note: This is NOT pure data storage; it includes process
//!   management operations.
//!
//! ## Architecture
//!
//! This crate is a large module with many utility functions. It depends only on the Entities
//! layer (dependencies flow inward), making it a foundational utility layer for the runtime.
//!
//! ## See Also
//!
//! - [`entities_utilities`](../../entities/entities_utilities/index.html): Entity-level utilities
//! - [`entities_process`](../../entities/entities_process/index.html): Process entities

pub mod common;
pub mod helpers;
pub mod compression;
pub mod process_table;
pub mod atom_table;

pub use common::{CommonUtils, FormatUtils, MathUtils, RationalUtils, MiscUtils, HashUtils, ArrayUtils, ThreadingUtils, TimeUtils, PathUtils, UtilityError};
pub use helpers::HelperFunctions;
pub use compression::{CompressionLevel, CompressionError, CompressionResult, ChunkResult, DeflateStream, InflateStream, compress2, uncompress, zstd_compress, zstd_decompress};
pub use process_table::{ProcessTable, get_global_process_table, ProcessTableError};
pub use atom_table::get_global_atom_table;

