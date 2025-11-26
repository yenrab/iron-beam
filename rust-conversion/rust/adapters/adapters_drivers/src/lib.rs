//! Adapters Layer: Drivers
//!
//! Provides driver implementations:
//! - INET driver
//! - RAM file driver
//! - And other drivers
//!
//! Based on inet_drv.c, ram_file_drv.c
//! Depends on Entities and Use Cases layers.

pub mod inet;
pub mod ram_file;

pub use inet::InetDriver;
pub use ram_file::RamFileDriver;

