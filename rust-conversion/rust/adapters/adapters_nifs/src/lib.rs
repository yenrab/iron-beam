//! Adapters Layer: NIFs (Native Implemented Functions)
//!
//! Provides NIF implementations:
//! - Buffer NIFs
//! - File NIFs
//! - And many other NIF modules
//!
//! Based on erts/emulator/nifs/common/*.c
//! Depends on Entities and Use Cases layers.

pub mod buffer;
pub mod file;
pub mod nif_common;

pub use buffer::BufferNif;
pub use file::FileNif;

