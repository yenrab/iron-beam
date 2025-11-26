//! Entities Layer: Windows System Integration
//!
//! Provides Windows-specific system integration functionality.
//! Based on dosmap.c

#[cfg(windows)]
pub mod dosmap;

#[cfg(windows)]
pub use dosmap::*;

#[cfg(not(windows))]
/// Windows-specific functionality is only available on Windows
pub fn windows_only() {
    // Placeholder for non-Windows platforms
}

