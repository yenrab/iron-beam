//! Initialization State Management
//!
//! Provides state tracking for emulator initialization.

use std::sync::atomic::{AtomicBool, Ordering};

/// Global initialization state
static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Check if the emulator has been initialized
///
/// # Returns
/// * `true` - Emulator is initialized
/// * `false` - Emulator is not initialized
pub fn is_initialized() -> bool {
    INITIALIZED.load(Ordering::Acquire)
}

/// Set the initialization state
///
/// # Arguments
/// * `value` - New initialization state
pub fn set_initialized(value: bool) {
    INITIALIZED.store(value, Ordering::Release);
}

/// Initialization state tracker
///
/// This struct tracks the initialization state of the emulator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitializationState {
    /// Not initialized
    NotInitialized,
    /// Early initialization complete
    EarlyInitComplete,
    /// Main initialization complete
    MainInitComplete,
    /// Fully initialized
    FullyInitialized,
}

impl InitializationState {
    /// Check if initialization is complete
    pub fn is_complete(&self) -> bool {
        matches!(self, InitializationState::FullyInitialized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_is_initialized() {
        set_initialized(false);
        assert!(!is_initialized());
        
        set_initialized(true);
        assert!(is_initialized());
    }
    
    #[test]
    fn test_initialization_state() {
        let state = InitializationState::NotInitialized;
        assert!(!state.is_complete());
        
        let state = InitializationState::FullyInitialized;
        assert!(state.is_complete());
    }
}


