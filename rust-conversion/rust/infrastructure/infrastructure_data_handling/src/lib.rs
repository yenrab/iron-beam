//! Infrastructure Layer: Data Handling
//!
//! Provides data handling utilities for Erlang Interface (EI) format in the Erlang/OTP
//! runtime system. This crate implements encoding, decoding, and printing operations for
//! Erlang terms in the EI format.
//!
//! ## Overview
//!
//! The `infrastructure_data_handling` crate is part of the infrastructure layer in the
//! CLEAN architecture implementation of Erlang/OTP. It provides high-level operations
//! for working with Erlang terms in the EI format, which is used for serialization and
//! network communication.
//!
//! ## Modules
//!
//! - **[`decode_term`](decode_term/index.html)**: Decode complete Erlang terms from EI format
//! - **[`decode_atom`](decode_atom/index.html)**: Decode atoms from EI format
//! - **[`decode_binary`](decode_binary/index.html)**: Decode binaries from EI format
//! - **[`encode_atom`](encode_atom/index.html)**: Encode atoms to EI format
//! - **[`encode_binary`](encode_binary/index.html)**: Encode binaries to EI format
//! - **[`print_term`](print_term/index.html)**: Print terms in human-readable format
//!
//! ## Architecture
//!
//! This crate is based on the C implementation in various `lib/erl_interface/src/` files.
//! It depends on the Entities layer (`entities_data_handling`) for fundamental term types.
//!
//! ## See Also
//!
//! - [`entities_data_handling`](../../entities/entities_data_handling/index.html): Core term types
//! - [`infrastructure_code_loading`](../infrastructure_code_loading/index.html): Low-level encoding/decoding

/*
 * %CopyrightBegin%
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * Copyright Lee Barney 2025. All Rights Reserved.
 *
 * This file is derived from work copyrighted by Ericsson AB 1996-2025.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * %CopyrightEnd%
 */

pub mod decode_term;
pub mod decode_atom;
pub mod decode_binary;
pub mod encode_atom;
pub mod encode_binary;
pub mod print_term;

// Re-export main types
pub use decode_term::{decode_ei_term, DecodeError};
pub use decode_atom::{decode_atom, DecodeAtomError};
pub use decode_binary::{decode_binary, DecodeBinaryError};
pub use encode_atom::{encode_atom, encode_atom_len, EncodeAtomError};
pub use encode_binary::{encode_binary, EncodeBinaryError};
pub use print_term::{print_term, s_print_term, PrintError};
