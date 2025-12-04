//! Entities Layer: Windows System Integration
//!
//! Provides Windows-specific system integration functionality for the Erlang/OTP
//! runtime system. This crate contains platform-specific operations that are only
//! available on Windows platforms.
//!
//! ## Overview
//!
//! The `entities_system_integration_win32` crate is part of the entities layer in
//! the CLEAN architecture implementation of Erlang/OTP. It provides Windows-specific
//! system integration operations that are not available on other platforms.
//!
//! ## Platform Support
//!
//! This crate is only available on Windows platforms. On non-Windows platforms, the
//! crate provides placeholder functions that indicate Windows-specific functionality
//! is not available.
//!
//! ## Modules
//!
//! - **[`dosmap`](dosmap/index.html)**: DOS memory mapping operations specific to
//!   Windows. Provides Windows-specific memory mapping functionality.
//!
//! ## Usage
//!
//! ```rust
//! #[cfg(windows)]
//! use entities_system_integration_win32::dosmap;
//!
//! #[cfg(windows)]
//! {
//!     // Use Windows-specific functionality
//! }
//! ```
//!
//! ## See Also
//!
//! - [`entities_system_integration_common`](../entities_system_integration_common/index.html): Common system integration operations
//! - [`frameworks_system_integration_win32`](../../frameworks/frameworks_system_integration_win32/index.html): Windows framework integration

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

#[cfg(windows)]
pub mod dosmap;

#[cfg(windows)]
pub use dosmap::*;

#[cfg(not(windows))]
/// Windows-specific functionality is only available on Windows
pub fn windows_only() {
    // Placeholder for non-Windows platforms
}

