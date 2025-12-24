//! Initialization BIFs
//!
//! Provides built-in functions for initialization and process management.
//! Includes erl_init module functions that need to be implemented as BIFs
//! rather than JIT-compiled BEAM code.

/// Initialize BIF for erl_init module
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    // Stub for now - erl_init:start/2 is handled by avoiding JIT execution
    Ok(())
}
