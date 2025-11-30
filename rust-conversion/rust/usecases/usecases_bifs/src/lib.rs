//! Use Cases Layer: Built-in Functions
//!
//! Provides built-in functions (BIFs):
//! - Regular expressions
//! - Checksums
//! - Tracing
//! - Dynamic library loading
//! - And many more
//!
//! Based on erl_bif_*.c files
//! Depends on Entities layer.

pub mod regex;
pub mod checksum;
pub mod trace;
pub mod dynamic_library;
pub mod os;
pub mod counters;
pub mod unique;
pub mod op;

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

