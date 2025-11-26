//! Adapters Layer: Unix System Integration
//!
//! Provides Unix-specific system integration.
//! Based on sys_drivers.c
//! Depends on Entities layer.

#[cfg(unix)]
pub mod sys_drivers;

#[cfg(unix)]
pub use sys_drivers::SysDrivers;

#[cfg(not(unix))]
/// Unix-specific functionality is only available on Unix systems
pub fn unix_only() {
    // Placeholder for non-Unix platforms
}

