//! Adapters Layer: I/O Operations
//!
//! Provides I/O adapter operations for the Erlang/OTP runtime system. This crate
//! implements adapters for port and driver I/O operations, enabling communication
//! between Erlang processes and external systems.
//!
//! ## Overview
//!
//! The `adapters_io_operations` crate is part of the adapters layer in the CLEAN
//! architecture implementation of Erlang/OTP. It provides I/O adapters for port
//! and driver communication.
//!
//! ## Modules
//!
//! - **[`port_io`](port_io/index.html)**: Port I/O operations for communicating
//!   with external programs through Erlang ports
//!
//! - **[`driver_io`](driver_io/index.html)**: Driver I/O operations for communicating
//!   with linked-in drivers
//!
//! ## Architecture
//!
//! This crate is based on the C implementation in `ei_portio.c` and `port_driver.c`.
//! It depends on the Entities and Use Cases layers for fundamental operations.
//!
//! ## See Also
//!
//! - [`entities_io_operations`](../../entities/entities_io_operations/index.html): I/O entity layer
//! - [`usecases_io_operations`](../../usecases/usecases_io_operations/index.html): I/O use cases

pub mod port_io;
pub mod driver_io;

pub use port_io::PortIo;
pub use driver_io::DriverIo;

