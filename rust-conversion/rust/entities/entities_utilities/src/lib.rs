//! Entities Layer: Utilities
//!
//! This crate provides utility functions for the Erlang/OTP runtime system,
//! specifically focusing on big number operations and register handling.
//!
//! # Purpose
//!
//! The `entities_utilities` crate is part of the entities layer in the CLEAN
//! architecture implementation of Erlang/OTP. It provides fundamental utility
//! operations that are used throughout the runtime system:
//!
//! - **Big Number Operations**: Arbitrary precision integer arithmetic operations
//!   that support values beyond the range of standard integer types. This is
//!   essential for Erlang's bignum support, allowing calculations with numbers
//!   of any size. The implementation uses the `malachite` crate for high-performance
//!   arbitrary-precision arithmetic, which provides behavior compatible with the C
//!   implementation's two's complement semantics for operations.
//!
//! - **Register Handling**: Management of the process and port registration table,
//!   which maps atom names to process/port identifiers. This enables Erlang's
//!   `register/2` and `whereis/1` functionality, allowing processes to be found
//!   by name. The register table maintains bidirectional mappings between names
//!   and IDs, ensuring that each name maps to exactly one ID and each ID maps to
//!   at most one name.
//!
//! # Architecture
//!
//! This crate is part of the innermost layer of the CLEAN architecture with
//! minimal dependencies. It uses the `malachite` crate for high-performance
//! arbitrary-precision arithmetic, which provides behavior compatible with
//! the C implementation's two's complement semantics. The register implementation
//! uses Rust's standard `HashMap` for efficient name-to-ID lookups.
//!
//! # Examples
//!
//! ## Big Number Operations
//!
//! ```rust
//! use entities_utilities::BigNumber;
//!
//! // Create big numbers from various types
//! let a = BigNumber::from_i64(1234567890123456789);
//! let b = BigNumber::from_i64(9876543210987654321);
//!
//! // Perform arithmetic operations
//! let sum = a.plus(&b);
//! let product = a.times(&b);
//! let quotient = a.div(&b).unwrap();
//!
//! // Convert to string in different bases
//! assert_eq!(a.to_string_base(16), "112210f47de98115");
//! ```
//!
//! ## Register Operations
//!
//! ```rust
//! use entities_utilities::{Register, RegisterResult};
//!
//! let mut reg = Register::new();
//!
//! // Register a process with a name
//! reg.register_name("my_process", 123);
//!
//! // Look up the process by name
//! let id = reg.whereis_name("my_process");
//! assert_eq!(id, Some(123));
//!
//! // Find the name for an ID
//! let name = reg.get_name_for_id(123);
//! assert_eq!(name, Some("my_process".to_string()));
//! ```
//!
//! ## Cross-Module Usage
//!
//! ```rust
//! use entities_utilities::{BigNumber, Register};
//!
//! // Use big numbers as register IDs
//! let mut reg = Register::new();
//! let large_id = BigNumber::from_u64(u64::MAX);
//!
//! // Convert to u64 for registration (if within range)
//! if let Some(id) = large_id.to_u32() {
//!     reg.register_name("large_process", id as u64);
//! }
//! ```
//!
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

pub mod big;
pub mod rational;
pub mod register;

pub use big::BigNumber;
pub use rational::BigRational;
pub use register::{Register, RegisterResult};
