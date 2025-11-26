//! Adapters Layer: I/O Operations
//!
//! Provides I/O adapter operations:
//! - Port I/O
//! - Driver I/O
//!
//! Based on ei_portio.c and port_driver.c
//! Depends on Entities and Use Cases layers.

pub mod port_io;
pub mod driver_io;

pub use port_io::PortIo;
pub use driver_io::DriverIo;

