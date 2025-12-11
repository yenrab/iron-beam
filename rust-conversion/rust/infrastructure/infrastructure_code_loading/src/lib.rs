//! Infrastructure Layer: Code Loading
//!
//! Provides code loading utilities and Erlang Interface (EI) format encoding/decoding
//! primitives for the Erlang/OTP runtime system. This crate implements low-level
//! encoding and decoding operations for all Erlang term types.
//!
//! ## Overview
//!
//! The `infrastructure_code_loading` crate is part of the infrastructure layer in the
//! CLEAN architecture implementation of Erlang/OTP. It provides the fundamental
//! encoding and decoding operations needed for code loading, serialization, and
//! network communication.
//!
//! ## Modules
//!
//! - **[`code_loader`](code_loader/index.html)**: High-level code loading interface
//! - **[`encode_integers`](encode_integers/index.html)**: Integer encoding (long, ulong, longlong, ulonglong)
//! - **[`decode_integers`](decode_integers/index.html)**: Integer decoding
//! - **[`encode_double`](encode_double/index.html)**: Double-precision floating-point encoding
//! - **[`decode_double`](decode_double/index.html)**: Double-precision floating-point decoding
//! - **[`encode_char`](encode_char/index.html)**: Character encoding
//! - **[`decode_char`](decode_char/index.html)**: Character decoding
//! - **[`encode_headers`](encode_headers/index.html)**: Tuple, map, and list header encoding
//! - **[`decode_headers`](decode_headers/index.html)**: Tuple, map, and list header decoding
//! - **[`encode_pid`](encode_pid/index.html)**: Process ID encoding
//! - **[`decode_pid`](decode_pid/index.html)**: Process ID decoding
//! - **[`encode_port`](encode_port/index.html)**: Port encoding
//! - **[`decode_port`](decode_port/index.html)**: Port decoding
//! - **[`encode_ref`](encode_ref/index.html)**: Reference encoding
//! - **[`decode_ref`](decode_ref/index.html)**: Reference decoding
//! - **[`encode_fun`](encode_fun/index.html)**: Function encoding
//! - **[`decode_fun`](decode_fun/index.html)**: Function decoding
//! - **[`encode_trace`](encode_trace/index.html)**: Trace encoding
//! - **[`decode_trace`](decode_trace/index.html)**: Trace decoding
//!
//! ## Architecture
//!
//! This crate is based on the C implementation in `lib/erl_interface/src/encode/*.c` and
//! `lib/erl_interface/src/decode/*.c`. It depends on the Entities layer for fundamental
//! data types.
//!
//! ## See Also
//!
//! - [`code_management_code_loading`](../../code_management/code_management_code_loading/index.html): High-level code loading
//! - [`infrastructure_data_handling`](../infrastructure_data_handling/index.html): Term encoding/decoding

pub mod constants;
pub mod code_loader;
pub mod encode_integers;
pub mod decode_integers;
pub mod encode_double;
pub mod decode_double;
pub mod encode_char;
pub mod decode_char;
pub mod encode_headers;
pub mod decode_headers;
pub mod encode_pid;
pub mod decode_pid;
pub mod encode_port;
pub mod decode_port;
pub mod encode_ref;
pub mod decode_ref;
pub mod encode_fun;
pub mod decode_fun;
pub mod encode_trace;
pub mod decode_trace;
pub mod decode_skip;

pub use code_loader::CodeLoader;
pub use encode_integers::{encode_long, encode_ulong, encode_longlong, encode_ulonglong, EncodeError as IntegerEncodeError};
pub use decode_integers::{decode_long, decode_ulong, decode_longlong, decode_ulonglong, DecodeError as IntegerDecodeError};
pub use encode_double::{encode_double, EncodeError as DoubleEncodeError};
pub use decode_double::{decode_double, DecodeError as DoubleDecodeError};
pub use encode_char::{encode_char, EncodeError as CharEncodeError};
pub use decode_char::{decode_char, DecodeError as CharDecodeError};
pub use encode_headers::{encode_tuple_header, encode_map_header, encode_list_header, EncodeError as HeaderEncodeError};
pub use decode_headers::{decode_tuple_header, decode_map_header, decode_list_header, DecodeError as HeaderDecodeError};
pub use encode_pid::{encode_pid, ErlangPid, EncodeError as PidEncodeError};
pub use decode_pid::{decode_pid, DecodeError as PidDecodeError};
pub use encode_port::{encode_port, ErlangPort, EncodeError as PortEncodeError};
pub use decode_port::{decode_port, DecodeError as PortDecodeError};
pub use encode_ref::{encode_ref, ErlangRef, EncodeError as RefEncodeError};
pub use decode_ref::{decode_ref, DecodeError as RefDecodeError};
pub use encode_fun::{encode_fun, ErlangFunType, EncodeError as FunEncodeError};
pub use decode_fun::{decode_fun, DecodeError as FunDecodeError};
pub use encode_trace::{encode_trace, ErlangTrace, EncodeError as TraceEncodeError};
pub use decode_trace::{decode_trace, DecodeError as TraceDecodeError};

