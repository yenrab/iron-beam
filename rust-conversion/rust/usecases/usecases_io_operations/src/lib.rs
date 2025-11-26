//! Use Cases Layer: I/O Operations
//!
//! Provides I/O operations:
//! - Port BIFs
//! - Port control operations
//!
//! Based on erl_bif_port.c
//! Depends on Entities layer.

pub mod port_bif;
pub mod port_control;

pub use port_bif::PortBif;

