//! Frameworks Layer: Unix System Integration
//!
//! Provides Unix-specific system integration at framework level.
//! Based on sys_env.c and related Unix system files.
//! Depends on Entities and Adapters layers.

#[cfg(unix)]
pub mod sys_integration;

#[cfg(unix)]
pub use sys_integration::SysIntegration;

#[cfg(not(unix))]
/// Unix-specific functionality is only available on Unix systems
pub fn unix_only() {
    // Placeholder for non-Unix platforms
}

