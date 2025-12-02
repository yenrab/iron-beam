//! Infrastructure Layer: Data Handling
//!
//! Provides data handling utilities for Erlang Interface (EI) format:
//! - Decoding EI-encoded terms (atoms, binaries, terms)
//! - Encoding terms to EI format (atoms, binaries)
//! - Printing terms in readable format
//!
//! Based on:
//! - lib/erl_interface/src/misc/ei_decode_term.c
//! - lib/erl_interface/src/encode/encode_atom.c
//! - lib/erl_interface/src/decode/decode_binary.c
//! - lib/erl_interface/src/encode/encode_binary.c
//! - lib/erl_interface/src/decode/decode_atom.c
//! - lib/erl_interface/src/misc/ei_printterm.c
//!
//! Depends on Entities layer (entities_data_handling).

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
