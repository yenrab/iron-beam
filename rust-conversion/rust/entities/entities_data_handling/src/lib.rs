//! Entities Layer: Data Handling
//!
//! This crate provides core data handling functionality for Erlang terms:
//! - Term hashing (portable and internal hash functions)
//! - Atom table management
//! - Bit manipulation operations
//! - Binary operations
//! - Map operations
//! - Atomic operations
//!
//! This is the innermost layer of CLEAN architecture with no dependencies.

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

