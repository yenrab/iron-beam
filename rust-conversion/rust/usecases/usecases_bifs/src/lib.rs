//! Use Cases Layer: Built-in Functions
//!
//! Provides built-in functions (BIFs):
//! - Regular expressions
//! - Checksums
//! - Tracing
//! - Dynamic library loading
//! - And many more
//!
//! Depends on Entities layer.

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
 *
 * Creation productivity increased for code in this file by using AALang and GAB.
 * See https://github.com/yenrab/AALang-Gab
 */

pub mod regex;
pub mod checksum;
pub mod trace;
pub mod dynamic_library;
pub mod os;
pub mod counters;
pub mod unique;
pub mod op;
pub mod guard;
pub mod lists;
pub mod persistent;
pub mod load;
pub mod info;

pub use regex::{RegexBif, CompiledRegex, MatchResult, Capture, RegexError as RegexErr};
pub use checksum::ChecksumBif;
pub use trace::TraceBif;
pub use dynamic_library::{
    DynamicLibraryLoader, LibraryId, ProcessId, LibraryStatus, LoadOptions,
    MonitorOption, ReloadOption, LoadResult, UnloadResult, LibraryInfo, LibraryError
};
pub use os::{OsBif, OsError};
pub use counters::{CountersBif, CounterRef, CounterInfo, CountersError};
pub use unique::{UniqueBif, Reference, UniqueIntegerOption, UniqueError};
pub use op::{OpBif, OpError};
pub use guard::{GuardBif, GuardError};
pub use lists::{ListsBif, ListsError};
pub use persistent::{PersistentBif, PersistentError};
pub use load::{LoadBif, LoadError, ModuleStatus};
pub use info::{InfoBif, InfoError};

