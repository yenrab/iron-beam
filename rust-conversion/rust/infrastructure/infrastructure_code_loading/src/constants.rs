//! EI Format Constants
//!
//! Defines the tag constants used in the Erlang Interface (EI) format for encoding
//! and decoding Erlang terms. These constants match the definitions in
//! `lib/erl_interface/src/eidef.h` and are used throughout the code loading
//! infrastructure.
//!
//! ## Overview
//!
//! The EI (Erlang Interface) format is a binary serialization format used for
//! encoding Erlang terms for network transmission, file storage, and inter-process
//! communication. Each term type has a unique tag byte that identifies its type
//! and encoding format.
//!
//! ## Tag Categories
//!
//! - **Immediate Values**: Small integers, atoms, nil
//! - **Boxed Values**: Large integers, floats, binaries
//! - **Compound Types**: Tuples, lists, maps
//! - **Process Types**: PIDs, ports, references
//! - **Special Types**: Functions, exports, traces
//!
//! ## Usage
//!
//! These constants are used by encoding and decoding functions to identify term types:
//!
//! ```rust
//! use infrastructure_code_loading::constants::*;
//!
//! // Check if a tag represents a small integer
//! if tag == ERL_SMALL_INTEGER_EXT {
//!     // Handle small integer
//! }
//! ```
//!
//! ## See Also
//!
//! - [`encode_integers`](super::encode_integers/index.html): Integer encoding functions
//! - [`decode_integers`](super::decode_integers/index.html): Integer decoding functions
//! - [`infrastructure_data_handling`](../infrastructure_data_handling/index.html): High-level term encoding/decoding

/// Small integer (0-255)
pub const ERL_SMALL_INTEGER_EXT: u8 = 97;

/// Integer (32-bit signed)
pub const ERL_INTEGER_EXT: u8 = 98;

/// Atom (old format)
pub const ERL_ATOM_EXT: u8 = 100;

/// Small tuple (arity <= 255)
pub const ERL_SMALL_TUPLE_EXT: u8 = 104;

/// Large tuple (arity > 255)
pub const ERL_LARGE_TUPLE_EXT: u8 = 105;

/// Nil (empty list)
pub const ERL_NIL_EXT: u8 = 106;

/// List
pub const ERL_LIST_EXT: u8 = 108;

/// Binary
pub const ERL_BINARY_EXT: u8 = 109;

/// Small atom (old format, length <= 255)
pub const ERL_SMALL_ATOM_EXT: u8 = 115;

/// Atom UTF-8
pub const ERL_ATOM_UTF8_EXT: u8 = 118;

/// Small atom UTF-8
pub const ERL_SMALL_ATOM_UTF8_EXT: u8 = 119;

/// Map
pub const ERL_MAP_EXT: u8 = 116;

/// Small big integer (arity <= 255 bytes)
pub const ERL_SMALL_BIG_EXT: u8 = 110;

/// Large big integer (arity > 255 bytes)
pub const ERL_LARGE_BIG_EXT: u8 = 111;

/// Float (old format, 31 bytes)
pub const ERL_FLOAT_EXT: u8 = 99;

/// New float (IEEE 754, 8 bytes)
pub const NEW_FLOAT_EXT: u8 = 70;

/// PID
pub const ERL_PID_EXT: u8 = 103;

/// Port
pub const ERL_PORT_EXT: u8 = 102;

/// Reference
pub const ERL_REFERENCE_EXT: u8 = 101;

/// New reference
pub const ERL_NEW_REFERENCE_EXT: u8 = 114;

/// Newer reference
pub const ERL_NEWER_REFERENCE_EXT: u8 = 90;

/// Function
pub const ERL_FUN_EXT: u8 = 117;

/// New function
pub const ERL_NEW_FUN_EXT: u8 = 112;

/// Export
pub const ERL_EXPORT_EXT: u8 = 113;

/// Trace
pub const ERL_TRACE_EXT: u8 = 120;

/// Version
pub const ERL_VERSION: u8 = 131;

/// New PID (32-bit creation)
pub const ERL_NEW_PID_EXT: u8 = 88;

/// V4 Port (64-bit id)
pub const ERL_V4_PORT_EXT: u8 = 120;

/// New Port (32-bit id and creation)
pub const ERL_NEW_PORT_EXT: u8 = 89;

/// Maximum value for ERL_INTEGER_EXT (2^31 - 1)
pub const ERL_MAX: i64 = 2_147_483_647;

/// Minimum value for ERL_INTEGER_EXT (-2^31)
pub const ERL_MIN: i64 = -2_147_483_648;

