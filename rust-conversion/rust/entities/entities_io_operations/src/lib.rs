//! Entities Layer: I/O Operations
//!
//! Provides I/O operations for the Erlang/OTP runtime system, specifically focusing
//! on export table management for function call resolution.
//!
//! ## Overview
//!
//! The `entities_io_operations` crate is part of the entities layer in the CLEAN
//! architecture implementation of Erlang/OTP. It provides fundamental operations
//! for managing export entries, which represent callable functions in the system.
//!
//! ## Modules
//!
//! - **[`export`](export/index.html)**: Export table management for MFA (Module, Function, Arity)
//!   entries. The export table maps function identifiers to export entries, enabling
//!   efficient function lookup and call resolution.
//!
//! ## Usage
//!
//! ```rust
//! use entities_io_operations::{ExportTable, Mfa, Export};
//!
//! // Create an export table
//! let table = ExportTable::new();
//!
//! // Add an export entry (module=1, function=2, arity=3)
//! let export = table.put(1, 2, 3);
//!
//! // Look up an export
//! let found = table.get(1, 2, 3);
//!
//! // Or use MFA for convenience
//! let mfa = Mfa::new(1, 2, 3);
//! let found = table.get(mfa.module, mfa.function, mfa.arity);
//! ```
//!
//! ## See Also
//!
//! - [`entities_data_handling`](../entities_data_handling/index.html): Atom table used by export entries
//! - [`entities_utilities`](../entities_utilities/index.html): Big number and register operations

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

pub mod export;

pub use export::export_ops;
pub use export::{Export, ExportTable, Mfa, get_global_export_table};
