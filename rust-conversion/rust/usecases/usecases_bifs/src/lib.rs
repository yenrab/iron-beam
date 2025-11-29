//! Use Cases Layer: Built-in Functions
//!
//! Provides built-in functions (BIFs):
//! - Regular expressions
//! - Checksums
//! - Tracing
//! - And many more
//!
//! Based on erl_bif_*.c files
//! Depends on Entities layer.

pub mod regex;
pub mod checksum;
pub mod trace;

pub use regex::{RegexBif, CompiledRegex, MatchResult, Capture, RegexError as RegexErr};
pub use checksum::ChecksumBif;
pub use trace::TraceBif;

