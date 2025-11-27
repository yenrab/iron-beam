//! Entities Layer: Process and Port Core Data Structures
//!
//! This crate provides core data structure definitions for:
//! - Process (Erlang process structure)
//! - Port (Erlang port structure)
//!
//! These are fundamental VM data structures that are used throughout
//! the runtime system. This is the innermost layer of CLEAN architecture
//! with no dependencies.

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

pub mod process;
pub mod port;
pub mod common;

pub use process::{Process, ProcessFlags, ProcessState, ProcessPriority};
pub use port::{Port, PortFlags, PortState, PortStatusFlags};
pub use common::{ProcessId, PortId, Eterm, ErtsPTabElementCommon};
