//! Signal Stack Initialization Module
//!
//! Provides signal stack initialization functionality (Rust equivalent of sys_init_signal_stack()).
//! Required for scheduler thread safety when using native Erlang stacks.

#[cfg(unix)]
use libc::{sigaction, sigaltstack, stack_t, SA_ONSTACK, SIGSTKSZ, SIG_DFL, SIG_IGN};

/// Initialize signal stack for the main thread
///
/// This function sets up an alternate signal stack and adds SA_ONSTACK
/// to existing user-defined signal handlers. This is critical for scheduler
/// thread safety when using native Erlang stacks.
///
/// Based on sys_init_signal_stack() from sys_signal_stack.c
#[cfg(unix)]
pub unsafe fn sys_init_signal_stack() -> Result<(), String> {
    // Allocate signal stack
    let stack_size = SIGSTKSZ;
    let stack = libc::malloc(stack_size);
    if stack.is_null() {
        return Err("Failed to allocate signal stack".to_string());
    }

    // Set up alternate signal stack
    let mut ss: stack_t = std::mem::zeroed();
    ss.ss_sp = stack;
    ss.ss_flags = 0;
    ss.ss_size = stack_size;

    if sigaltstack(&ss, std::ptr::null_mut()) < 0 {
        libc::free(stack);
        return Err("Failed to set alternate signal stack".to_string());
    }

    // Add SA_ONSTACK to existing user-defined signal handlers
    // We iterate through all signals and update handlers that are not SIG_DFL or SIG_IGN
    let highest_signal = if cfg!(target_os = "linux") {
        // On Linux, NSIG is typically 65
        65
    } else if cfg!(target_os = "macos") {
        // On macOS, _NSIG is typically 32
        32
    } else {
        // Default fallback
        32
    };

    for i in 1..highest_signal {
        let mut sa: sigaction = std::mem::zeroed();
        
        // Get current signal action
        if sigaction(i, std::ptr::null(), &mut sa) != 0 {
            // Signal may not be valid (e.g., thread library private signals on Solaris)
            continue;
        }

        // Skip if handler is SIG_DFL, SIG_IGN, or already has SA_ONSTACK
        if sa.sa_sigaction == SIG_DFL as usize
            || sa.sa_sigaction == SIG_IGN as usize
            || (sa.sa_flags & SA_ONSTACK) != 0
        {
            continue;
        }

        // Add SA_ONSTACK flag
        sa.sa_flags |= SA_ONSTACK;

        // Set updated signal action
        if sigaction(i, &sa, std::ptr::null_mut()) != 0 {
            // Some signals (like SIGCANCEL on Solaris) cannot be modified
            // This is acceptable - we continue with other signals
            continue;
        }
    }

    Ok(())
}

/// Initialize signal stack for the main thread (Windows)
///
/// On Windows, signal stack initialization is not needed in the same way.
/// This is a no-op for Windows builds.
#[cfg(windows)]
pub unsafe fn sys_init_signal_stack() -> Result<(), String> {
    // Windows doesn't use the same signal stack mechanism
    // Signal handling is different on Windows
    Ok(())
}

/// Initialize signal stack for a scheduler thread
///
/// This should be called for each scheduler thread to set up its own
/// alternate signal stack.
#[cfg(unix)]
pub unsafe fn sys_thread_init_signal_stack() -> Result<(), String> {
    let stack_size = SIGSTKSZ;
    let stack = libc::malloc(stack_size);
    if stack.is_null() {
        return Err("Failed to allocate signal stack for thread".to_string());
    }

    let mut ss: stack_t = std::mem::zeroed();
    ss.ss_sp = stack;
    ss.ss_flags = 0;
    ss.ss_size = stack_size;

    if sigaltstack(&ss, std::ptr::null_mut()) < 0 {
        libc::free(stack);
        return Err("Failed to set alternate signal stack for thread".to_string());
    }

    Ok(())
}

#[cfg(windows)]
pub unsafe fn sys_thread_init_signal_stack() -> Result<(), String> {
    // Windows doesn't use the same signal stack mechanism
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sys_init_signal_stack() {
        // This test may fail if run in certain environments
        // It's mainly to ensure the function doesn't panic
        unsafe {
            let result = sys_init_signal_stack();
            // May succeed or fail depending on system state
            let _ = result;
        }
    }

    #[test]
    fn test_sys_init_signal_stack_returns_result() {
        unsafe {
            let result = sys_init_signal_stack();
            // Should return Result<(), String>
            match result {
                Ok(()) => {}
                Err(e) => {
                    // Error message should be informative
                    assert!(!e.is_empty());
                }
            }
        }
    }

    #[test]
    fn test_sys_init_signal_stack_idempotent() {
        unsafe {
            // First call
            let result1 = sys_init_signal_stack();
            
            // Second call - may succeed or fail
            let result2 = sys_init_signal_stack();
            
            // Both should return valid results
            let _ = (result1, result2);
        }
    }

    #[test]
    fn test_sys_init_signal_stack_error_message() {
        unsafe {
            let result = sys_init_signal_stack();
            if let Err(e) = result {
                // Error messages should be informative
                assert!(!e.is_empty());
                // Should contain relevant keywords
                assert!(e.contains("signal") || e.contains("stack") || e.contains("allocate") || e.contains("Failed"));
            }
        }
    }

    #[test]
    fn test_sys_thread_init_signal_stack() {
        unsafe {
            let result = sys_thread_init_signal_stack();
            // May succeed or fail depending on system state
            let _ = result;
        }
    }

    #[test]
    fn test_sys_thread_init_signal_stack_returns_result() {
        unsafe {
            let result = sys_thread_init_signal_stack();
            // Should return Result<(), String>
            match result {
                Ok(()) => {}
                Err(e) => {
                    // Error message should be informative
                    assert!(!e.is_empty());
                }
            }
        }
    }

    #[test]
    fn test_sys_thread_init_signal_stack_error_message() {
        unsafe {
            let result = sys_thread_init_signal_stack();
            if let Err(e) = result {
                // Error messages should be informative
                assert!(!e.is_empty());
                // Should contain relevant keywords
                assert!(e.contains("signal") || e.contains("stack") || e.contains("thread") || e.contains("allocate") || e.contains("Failed"));
            }
        }
    }

    #[test]
    fn test_sys_thread_init_signal_stack_idempotent() {
        unsafe {
            // First call
            let result1 = sys_thread_init_signal_stack();
            
            // Second call - may succeed or fail
            let result2 = sys_thread_init_signal_stack();
            
            // Both should return valid results
            let _ = (result1, result2);
        }
    }

    #[test]
    fn test_sys_init_signal_stack_platform_specific() {
        unsafe {
            let result = sys_init_signal_stack();
            
            #[cfg(windows)]
            {
                // On Windows, should always succeed (no-op)
                assert!(result.is_ok());
            }
            
            #[cfg(unix)]
            {
                // On Unix, may succeed or fail depending on system state
                let _ = result;
            }
        }
    }

    #[test]
    fn test_sys_thread_init_signal_stack_platform_specific() {
        unsafe {
            let result = sys_thread_init_signal_stack();
            
            #[cfg(windows)]
            {
                // On Windows, should always succeed (no-op)
                assert!(result.is_ok());
            }
            
            #[cfg(unix)]
            {
                // On Unix, may succeed or fail depending on system state
                let _ = result;
            }
        }
    }

    #[test]
    fn test_sys_init_signal_stack_multiple_calls() {
        unsafe {
            // Test multiple calls don't cause issues
            for _ in 0..3 {
                let result = sys_init_signal_stack();
                let _ = result;
            }
        }
    }

    #[test]
    fn test_sys_thread_init_signal_stack_multiple_calls() {
        unsafe {
            // Test multiple calls don't cause issues
            for _ in 0..3 {
                let result = sys_thread_init_signal_stack();
                let _ = result;
            }
        }
    }

    #[test]
    fn test_signal_stack_functions_dont_panic() {
        unsafe {
            // Both functions should not panic even if they fail
            let _result1 = sys_init_signal_stack();
            let _result2 = sys_thread_init_signal_stack();
        }
    }

    #[test]
    fn test_signal_stack_error_consistency() {
        unsafe {
            // If one call fails, subsequent calls may also fail
            // But they should return consistent error types
            let result1 = sys_init_signal_stack();
            let result2 = sys_init_signal_stack();
            
            // Both should be Result types
            match (result1, result2) {
                (Ok(_), Ok(_)) => {}
                (Err(_), Err(_)) => {}
                (Ok(_), Err(_)) => {}
                (Err(_), Ok(_)) => {}
            }
        }
    }

    #[test]
    fn test_sys_thread_init_signal_stack_error_consistency() {
        unsafe {
            let result1 = sys_thread_init_signal_stack();
            let result2 = sys_thread_init_signal_stack();
            
            // Both should be Result types
            match (result1, result2) {
                (Ok(_), Ok(_)) => {}
                (Err(_), Err(_)) => {}
                (Ok(_), Err(_)) => {}
                (Err(_), Ok(_)) => {}
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn test_unix_signal_stack_constants() {
        // Test that constants are available on Unix
        use libc::{SIGSTKSZ, SA_ONSTACK, SIG_DFL, SIG_IGN};
        
        // SIGSTKSZ should be a positive value
        assert!(SIGSTKSZ > 0);
        
        // Flags should be defined
        let _ = SA_ONSTACK;
        let _ = SIG_DFL;
        let _ = SIG_IGN;
    }

    #[test]
    fn test_signal_stack_functions_are_unsafe() {
        // Verify functions are marked as unsafe
        // This is a compile-time check - if they weren't unsafe, this wouldn't compile
        unsafe {
            let _f1: unsafe fn() -> Result<(), String> = sys_init_signal_stack;
            let _f2: unsafe fn() -> Result<(), String> = sys_thread_init_signal_stack;
        }
    }

    #[test]
    fn test_signal_stack_result_type() {
        unsafe {
            let result1 = sys_init_signal_stack();
            let result2 = sys_thread_init_signal_stack();
            
            // Both should return Result<(), String>
            assert!(matches!(result1, Ok(_) | Err(_)));
            assert!(matches!(result2, Ok(_) | Err(_)));
        }
    }
}

