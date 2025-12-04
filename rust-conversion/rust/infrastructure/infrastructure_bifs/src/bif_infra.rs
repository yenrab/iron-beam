//! BIF Infrastructure Module
//!
//! Provides infrastructure for built-in functions.
//! Based on bif.c
//!
//! This module provides the infrastructure layer for BIFs:
//! - BIF system initialization
//! - BIF state management
//! - BIF error handling
//!
//! Note: Actual BIF implementations are in the usecases layer (usecases_bifs).
//! This infrastructure layer provides the framework and mechanisms.

use std::sync::atomic::{AtomicBool, Ordering};

/// Global state tracking whether BIF infrastructure is initialized
static BIF_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// BIF infrastructure state
#[derive(Debug, Clone)]
pub struct BifState {
    /// Whether the BIF system is initialized
    initialized: bool,
    /// Scheduler wall time tracking (atomic counter, similar to C implementation)
    /// In the C code: erts_atomic32_init_nob(&sched_wall_time, 0);
    /// This is a placeholder - actual implementation would use proper atomic types
    sched_wall_time_enabled: bool,
    /// Microstate accounting state (atomic counter, similar to C implementation)
    /// In the C code: erts_atomic32_init_nob(&msacc, ERTS_MSACC_IS_ENABLED());
    /// This is a placeholder - actual implementation would use proper atomic types
    msacc_enabled: bool,
}

impl BifState {
    /// Create a new BIF state
    fn new() -> Self {
        Self {
            initialized: true,
            sched_wall_time_enabled: false,
            msacc_enabled: false,
        }
    }

    /// Check if scheduler wall time tracking is enabled
    pub fn is_sched_wall_time_enabled(&self) -> bool {
        self.sched_wall_time_enabled
    }

    /// Check if microstate accounting is enabled
    pub fn is_msacc_enabled(&self) -> bool {
        self.msacc_enabled
    }
}

/// BIF infrastructure
pub struct BifInfrastructure;

impl BifInfrastructure {
    /// Initialize BIF infrastructure
    ///
    /// This function initializes the BIF system, similar to `erts_init_bif()` in the C code.
    /// It sets up:
    /// - BIF system state
    /// - Scheduler wall time tracking (if enabled)
    /// - Microstate accounting (if enabled)
    ///
    /// # Returns
    ///
    /// * `Ok(state)` - Successfully initialized BIF state
    /// * `Err(BifError)` - Initialization error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use infrastructure_bifs::BifInfrastructure;
    ///
    /// let state = BifInfrastructure::init().unwrap();
    /// assert!(state.is_initialized());
    /// ```
    pub fn init() -> Result<BifState, BifError> {
        // Check if already initialized (idempotent initialization)
        if BIF_INITIALIZED.load(Ordering::Acquire) {
            return Err(BifError::AlreadyInitialized);
        }

        // Initialize BIF state
        // In the C code, this would initialize trap exports and atomic counters
        // For now, we create a basic state structure
        let state = BifState::new();

        // Mark as initialized
        BIF_INITIALIZED.store(true, Ordering::Release);

        Ok(state)
    }

    /// Check if BIF infrastructure is initialized
    ///
    /// # Returns
    ///
    /// `true` if initialized, `false` otherwise
    pub fn is_initialized() -> bool {
        BIF_INITIALIZED.load(Ordering::Acquire)
    }

    /// Reset BIF infrastructure (mainly for testing)
    ///
    /// # Safety
    ///
    /// This should only be called when it's safe to reset the BIF system,
    /// typically only in tests or during shutdown.
    #[cfg(test)]
    pub unsafe fn reset() {
        BIF_INITIALIZED.store(false, Ordering::Release);
    }
}

impl BifState {
    /// Check if the BIF system is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

/// BIF operation errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BifError {
    /// Initialization failed
    InitFailed(String),
    /// BIF infrastructure already initialized
    AlreadyInitialized,
    /// BIF not found
    BifNotFound(String),
    /// Invalid BIF arguments
    BadArgument(String),
    /// System limit exceeded
    SystemLimit(String),
}

impl std::fmt::Display for BifError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BifError::InitFailed(msg) => write!(f, "BIF initialization failed: {}", msg),
            BifError::AlreadyInitialized => write!(f, "BIF infrastructure already initialized"),
            BifError::BifNotFound(name) => write!(f, "BIF not found: {}", name),
            BifError::BadArgument(msg) => write!(f, "Bad argument: {}", msg),
            BifError::SystemLimit(msg) => write!(f, "System limit exceeded: {}", msg),
        }
    }
}

impl std::error::Error for BifError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bif_init() {
        // Reset state for clean test
        unsafe { BifInfrastructure::reset(); }
        
        let result = BifInfrastructure::init();
        assert!(result.is_ok());
        
        let state = result.unwrap();
        assert!(state.is_initialized());
        assert!(BifInfrastructure::is_initialized());
    }

    #[test]
    fn test_bif_init_idempotent() {
        // Reset state for clean test
        unsafe { BifInfrastructure::reset(); }
        
        // First initialization should succeed
        let result1 = BifInfrastructure::init();
        assert!(result1.is_ok());
        
        // Second initialization should fail with AlreadyInitialized
        let result2 = BifInfrastructure::init();
        assert!(result2.is_err());
        match result2.unwrap_err() {
            BifError::AlreadyInitialized => {}
            _ => panic!("Expected AlreadyInitialized error"),
        }
    }

    #[test]
    fn test_bif_state() {
        // Reset state for clean test
        unsafe { BifInfrastructure::reset(); }
        
        let state = BifInfrastructure::init().unwrap();
        assert!(state.is_initialized());
        assert!(!state.is_sched_wall_time_enabled());
        assert!(!state.is_msacc_enabled());
    }

    #[test]
    fn test_bif_error_display() {
        let error1 = BifError::InitFailed("test error".to_string());
        let error2 = BifError::AlreadyInitialized;
        let error3 = BifError::BifNotFound("test_bif".to_string());
        let error4 = BifError::BadArgument("invalid arg".to_string());
        let error5 = BifError::SystemLimit("too many processes".to_string());
        
        let str1 = format!("{}", error1);
        let str2 = format!("{}", error2);
        let str3 = format!("{}", error3);
        let str4 = format!("{}", error4);
        let str5 = format!("{}", error5);
        
        assert!(str1.contains("BIF initialization failed"));
        assert!(str1.contains("test error"));
        assert!(str2.contains("already initialized"));
        assert!(str3.contains("BIF not found"));
        assert!(str3.contains("test_bif"));
        assert!(str4.contains("Bad argument"));
        assert!(str4.contains("invalid arg"));
        assert!(str5.contains("System limit exceeded"));
        assert!(str5.contains("too many processes"));
    }

    #[test]
    fn test_bif_error_clone() {
        let error1 = BifError::InitFailed("test".to_string());
        let error2 = BifError::AlreadyInitialized;
        
        let cloned1 = error1.clone();
        let cloned2 = error2.clone();
        
        assert_eq!(error1, cloned1);
        assert_eq!(error2, cloned2);
    }

    #[test]
    fn test_bif_error_partial_eq() {
        let error1 = BifError::InitFailed("test".to_string());
        let error2 = BifError::InitFailed("test".to_string());
        let error3 = BifError::InitFailed("different".to_string());
        let error4 = BifError::AlreadyInitialized;
        
        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
        assert_ne!(error1, error4);
        assert_eq!(error4, BifError::AlreadyInitialized);
    }
}

