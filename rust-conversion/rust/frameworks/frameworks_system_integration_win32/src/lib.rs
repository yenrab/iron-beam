//! Frameworks Layer: Windows System Integration
//!
//! Provides Windows-specific system integration at framework level.
//! Based on sys_time.c and related Windows system files.
//! Depends on Entities and Adapters layers.

#[cfg(windows)]
pub mod sys_integration;

#[cfg(windows)]
pub use sys_integration::SysIntegration;

#[cfg(not(windows))]
/// Windows-specific functionality is only available on Windows
pub fn windows_only() {
    // Placeholder for non-Windows platforms
}

