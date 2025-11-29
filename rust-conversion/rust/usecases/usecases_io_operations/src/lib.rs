//! Use Cases Layer: I/O Operations
//!
//! Provides I/O operations:
//! - Helper functions for environment and argument handling
//!
//! Depends on Entities layer (entities_data_handling, entities_io_operations, entities_utilities).

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

pub mod helpers;

pub use helpers::{
    Environment, merge_global_environment, convert_args, free_args,
    PacketCallbackArgs, PacketResult, HttpUri, http_bld_string, http_bld_uri,
    HelperError,
};

