//! Entities Layer: Common System Integration
//!
//! Provides common system integration functionality for the Erlang/OTP runtime system.
//! This crate contains platform-independent system operations that are shared across
//! all platforms.
//!
//! ## Overview
//!
//! The `entities_system_integration_common` crate is part of the entities layer in
//! the CLEAN architecture implementation of Erlang/OTP. It provides fundamental
//! system integration operations that are common to all platforms, avoiding
//! platform-specific code in the entities layer.
//!
//! ## Modules
//!
//! - **[`mmap`](mmap/index.html)**: Memory mapping operations for allocating and
//!   managing memory-mapped regions. Provides a platform-independent interface
//!   for memory mapping operations.
//!
//! ## Usage
//!
//! ```rust
//! use entities_system_integration_common::MemoryMap;
//!
//! // Create a memory map
//! let mmap = MemoryMap::new(size)?;
//!
//! // Access the mapped memory
//! let data = mmap.as_slice();
//! ```
//!
//! ## See Also
//!
//! - [`entities_system_integration_win32`](../entities_system_integration_win32/index.html): Windows-specific system integration
//! - [`frameworks_system_integration`](../../frameworks/frameworks_system_integration/index.html): Framework-level system integration

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

pub mod mmap;

pub use mmap::MemoryMap;

