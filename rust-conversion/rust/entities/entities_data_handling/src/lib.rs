//! Entities Layer: Data Handling
//!
//! This crate provides core data handling functionality for the Erlang/OTP runtime system.
//! It implements fundamental operations on Erlang terms, including hashing, atom management,
//! bit manipulation, binary data handling, map operations, and atomic operations.
//!
//! ## Overview
//!
//! The `entities_data_handling` crate is part of the entities layer in the CLEAN architecture
//! implementation of Erlang/OTP. As the innermost layer, it has no dependencies on other
//! crates in the system, making it a foundational building block for all higher layers.
//!
//! ## Modules
//!
//! - **[`term_hashing`](term_hashing/index.html)**: Hash functions for Erlang terms, including
//!   portable hash functions (`make_hash`, `make_hash2`), internal VM hash functions
//!   (`erts_internal_hash`, `erts_internal_salted_hash`), and map-specific hash functions
//!   (`erts_map_hash`). Also provides the `Term` enum representing Erlang term types.
//!
//! - **[`atom`](atom/index.html)**: Atom table management for creating, storing, and looking up
//!   atoms. Supports multiple encoding formats (7-bit ASCII, Latin1, UTF-8) and provides
//!   validation and encoding conversion functionality.
//!
//! - **[`bits`](bits/index.html)**: Low-level bit manipulation operations including bit copying,
//!   bit comparison, bit offset calculations, and mask generation. Essential for handling
//!   bit-aligned data in Erlang binaries and bitstrings.
//!
//! - **[`binary`](binary/index.html)**: Binary data structure for representing Erlang binaries
//!   and bitstrings. Provides basic binary data storage and retrieval operations.
//!
//! - **[`map`](map/index.html)**: Map data structure for key-value pairs where both keys and
//!   values are Erlang terms. Provides operations for insertion, lookup, update, removal, and
//!   iteration over map entries.
//!
//! - **[`atomics`](atomics/index.html)**: Atomic operations for double-word atomics, providing
//!   thread-safe operations for 64-bit values. Includes compare-and-exchange, load, and store
//!   operations with various memory ordering semantics.
//!
//! ## Usage
//!
//! ```rust
//! use entities_data_handling::{AtomTable, AtomEncoding, Map};
//! use entities_data_handling::term_hashing::{Term, make_hash};
//!
//! // Create an atom table and register atoms
//! let table = AtomTable::new(1000);
//! let atom_index = table.put_index(b"my_atom", AtomEncoding::SevenBitAscii, false).unwrap();
//!
//! // Create terms and hash them
//! let term = Term::Atom(atom_index as u32);
//! let hash = make_hash(term.clone());
//!
//! // Use maps to store key-value pairs
//! let mut map = Map::new();
//! map.put(term.clone(), Term::Small(42));
//! let value = map.get(&term);
//! ```
//!
//! ## Architecture
//!
//! This crate is the innermost layer of the CLEAN architecture with no dependencies on other
//! crates in the system. Higher layers (usecases, adapters, infrastructure) depend on this
//! crate to provide fundamental data handling operations.
//!
//! ## See Also
//!
//! - [`entities_utilities`](../entities_utilities/index.html): Utility functions including big
//!   number operations and register handling
//! - [`entities_io_operations`](../entities_io_operations/index.html): I/O operations including
//!   export table management

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

pub mod term_hashing;
pub mod atom;
pub mod bits;
pub mod binary;
pub mod map;
pub mod atomics;

// Re-export main types for convenience
pub use term_hashing::HashValue;
pub use atom::{AtomTable, AtomEncoding};
pub use map::{Map, MapError};

