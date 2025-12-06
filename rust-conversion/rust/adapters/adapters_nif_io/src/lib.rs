//! Adapters Layer: NIF I/O Polling
//!
//! Provides I/O polling and event management for NIFs (Native Implemented Functions)
//! and network communication in the Erlang/OTP runtime system. This crate implements
//! the cross-platform I/O event infrastructure used by NIFs through `enif_select`
//! and by network communication modules.
//!
//! ## Overview
//!
//! The `adapters_nif_io` crate is part of the adapters layer in the CLEAN architecture
//! implementation of Erlang/OTP. It provides I/O polling functionality for:
//! - **NIFs**: Through `enif_select` API for monitoring file descriptors
//! - **Network communication**: Used by `gen_tcp`, `gen_udp`, `gen_sctp`, and `socket`
//!
//! ## Modules
//!
//! - **[`nif_io`](nif_io/index.html)**: I/O polling and event management for NIFs
//!   and network communication
//!
//! ## Architecture
//!
//! This crate is based on the C implementation in `erl_check_io.c`. The infrastructure
//! manages file descriptor event state and dispatches events to NIFs and network
//! communication modules. It depends on the Entities layer for fundamental data types.
//!
//! ## See Also
//!
//! - [`adapters_nifs`](../adapters_nifs/index.html): NIF implementations
//! - [`adapters_system_integration_unix`](../adapters_system_integration_unix/index.html): Unix-specific system integration

pub mod nif_io;

pub use nif_io::{
    CheckIo, CheckIoConfig, CheckIoInfo, CheckIoError,
    PollThreadId, IoEvent, IoEventType,
    NifIOQueue, NifIOQueueOpts, NifIOVec, NifBinary, SysIOVec,
};
