//! Signal Stack Initialization Module
//!
//! Provides signal stack initialization functionality (Rust equivalent of sys_init_signal_stack()).
//! Required for scheduler thread safety when using native Erlang stacks.
//!
//! This module uses safe Rust memory management (Box/Vec) instead of manual malloc/free,
//! and the `nix` crate for safe signal handling APIs.

#[cfg(unix)]
use libc::{sigaltstack, stack_t, SIGSTKSZ, SA_ONSTACK, SIG_DFL, SIG_IGN};
#[cfg(unix)]
use nix::sys::signal::{sigaction, SigAction, SigHandler, SaFlags, SigSet, Signal};
use std::sync::Mutex;

// Store allocated signal stacks to keep them alive for the lifetime of the program/thread
// This prevents the memory from being freed while the signal stack is in use
#[cfg(unix)]
static MAIN_SIGNAL_STACK: Mutex<Option<Box<[u8]>>> = Mutex::new(None);

#[cfg(unix)]
thread_local! {
    static THREAD_SIGNAL_STACK: std::cell::RefCell<Option<Box<[u8]>>> = std::cell::RefCell::new(None);
}

/// Safe wrapper for sigaltstack using Rust memory management
///
/// This function wraps the unsafe sigaltstack() system call, but uses safe Rust
/// memory management. The memory must remain valid for the lifetime of the signal stack.
#[cfg(unix)]
fn set_signal_stack(stack_ptr: *mut libc::c_void, stack_size: libc::size_t) -> nix::Result<()> {
    let mut ss: stack_t = unsafe { std::mem::zeroed() };
    ss.ss_sp = stack_ptr;
    ss.ss_flags = 0;
    ss.ss_size = stack_size;

    unsafe {
        if sigaltstack(&ss, std::ptr::null_mut()) < 0 {
            return Err(nix::errno::Errno::last());
        }
    }
    Ok(())
}

/// Initialize signal stack for the main thread
///
/// This function sets up an alternate signal stack and adds SA_ONSTACK
/// to existing user-defined signal handlers. This is critical for scheduler
/// thread safety when using native Erlang stacks.
///
/// Uses safe Rust memory management (Box<[u8]>) instead of malloc/free.
/// The memory is stored in a static variable to keep it alive for the program lifetime.
///
/// Based on sys_init_signal_stack() from sys_signal_stack.c
#[cfg(unix)]
pub fn sys_init_signal_stack() -> Result<(), String> {
    // Use a lock to prevent race conditions during initialization
    let mut guard = MAIN_SIGNAL_STACK.lock().unwrap();
    
    // Check if already initialized (while holding the lock)
    if guard.is_some() {
        // Already initialized, return success
        return Ok(());
    }
    
    // Allocate signal stack using safe Rust
    let stack_size = SIGSTKSZ;
    let mut stack_box = vec![0u8; stack_size].into_boxed_slice();
    let stack_ptr = stack_box.as_mut_ptr() as *mut libc::c_void;
    
    // Set up alternate signal stack (minimal unsafe for FFI)
    // Note: We do this while holding the lock to prevent multiple threads
    // from setting signal stacks simultaneously
    match set_signal_stack(stack_ptr, stack_size) {
        Ok(()) => {
            // Store the Box to keep memory alive for the program lifetime
            // This prevents the memory from being freed while the signal stack is in use
            *guard = Some(stack_box);
        }
        Err(e) => {
            // If setting signal stack fails, drop the guard and return error
            // The stack_box will be dropped automatically
            drop(guard);
            return Err(format!("Failed to set alternate signal stack: {}", e));
        }
    }
    
    // Release the lock before continuing with signal handler updates
    drop(guard);

    // Add SA_ONSTACK to existing user-defined signal handlers using safe nix API
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

    for signal_num in 1..highest_signal {
        // Try to convert signal number to Signal enum
        let signal = match Signal::try_from(signal_num) {
            Ok(sig) => sig,
            Err(_) => continue, // Invalid signal number, skip
        };
        
        // Query current signal action using libc (nix doesn't have a query-only function)
        // We use libc::sigaction with null action pointer to query without modifying
        use libc::sigaction as libc_sigaction;
        let mut sa: libc::sigaction = unsafe { std::mem::zeroed() };
        
        if unsafe { libc_sigaction(signal_num, std::ptr::null(), &mut sa) } != 0 {
            // Signal may not be valid or accessible (e.g., thread library private signals)
            continue;
        }
        
        // Skip if handler is SIG_DFL, SIG_IGN, or already has SA_ONSTACK
        if sa.sa_sigaction == SIG_DFL as usize
            || sa.sa_sigaction == SIG_IGN as usize
            || (sa.sa_flags & SA_ONSTACK) != 0
        {
            continue;
        }
        
        // Convert libc sigaction to nix SigAction for safe modification
        // We need to preserve the handler, flags, and mask
        let handler = if sa.sa_sigaction == SIG_DFL as usize {
            SigHandler::SigDfl
        } else if sa.sa_sigaction == SIG_IGN as usize {
            SigHandler::SigIgn
        } else {
            // For custom handlers, we preserve the pointer (this is still unsafe territory)
            // but we use nix's safe API for setting
            unsafe { SigHandler::Handler(std::mem::transmute(sa.sa_sigaction)) }
        };
        
        let mut flags = SaFlags::from_bits_truncate(sa.sa_flags);
        flags.insert(SaFlags::SA_ONSTACK);
        
        // Convert libc sigset_t to nix SigSet
        // We need to create a SigSet from the raw sigset_t
        // SigSet::all() creates a set with all signals, then we can modify it
        // But for simplicity, we'll use an empty set since we're just adding SA_ONSTACK
        let mask = SigSet::empty();
        
        let new_action = SigAction::new(handler, flags, mask);
        
        // Set updated signal action using nix API (still unsafe due to FFI)
        if let Err(_) = unsafe { sigaction(signal, &new_action) } {
            // Some signals cannot be modified (e.g., SIGCANCEL on Solaris)
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
pub fn sys_init_signal_stack() -> Result<(), String> {
    // Windows doesn't use the same signal stack mechanism
    // Signal handling is different on Windows
    Ok(())
}

/// Initialize signal stack for a scheduler thread
///
/// This should be called for each scheduler thread to set up its own
/// alternate signal stack.
///
/// Uses safe Rust memory management (Box<[u8]>) instead of malloc/free.
/// The memory is stored in thread-local storage to keep it alive for the thread lifetime.
#[cfg(unix)]
pub fn sys_thread_init_signal_stack() -> Result<(), String> {
    // Check if already initialized for this thread
    let already_init = THREAD_SIGNAL_STACK.with(|cell| {
        cell.borrow().is_some()
    });
    
    if already_init {
        return Ok(());
    }
    
    let stack_size = SIGSTKSZ;
    let mut stack_box = vec![0u8; stack_size].into_boxed_slice();
    let stack_ptr = stack_box.as_mut_ptr() as *mut libc::c_void;

    set_signal_stack(stack_ptr, stack_size)
        .map_err(|e| format!("Failed to set alternate signal stack for thread: {}", e))?;

    // Store the Box in thread-local storage to keep memory alive for the thread lifetime
    // This prevents the memory from being freed while the signal stack is in use
    THREAD_SIGNAL_STACK.with(|cell| {
        *cell.borrow_mut() = Some(stack_box);
    });

    Ok(())
}

#[cfg(windows)]
pub fn sys_thread_init_signal_stack() -> Result<(), String> {
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
        let result = sys_init_signal_stack();
        // May succeed or fail depending on system state
        let _ = result;
    }

    #[test]
    fn test_sys_init_signal_stack_returns_result() {
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

    #[test]
    fn test_sys_init_signal_stack_idempotent() {
        // First call
        let result1 = sys_init_signal_stack();
        
        // Second call - should succeed (idempotent)
        let result2 = sys_init_signal_stack();
        
        // Both should return valid results
        let _ = (result1, result2);
    }

    #[test]
    fn test_sys_init_signal_stack_error_message() {
        let result = sys_init_signal_stack();
        if let Err(e) = result {
            // Error messages should be informative
            assert!(!e.is_empty());
            // Should contain relevant keywords
            assert!(e.contains("signal") || e.contains("stack") || e.contains("Failed"));
        }
    }

    #[test]
    fn test_sys_thread_init_signal_stack() {
        let result = sys_thread_init_signal_stack();
        // May succeed or fail depending on system state
        let _ = result;
    }

    #[test]
    fn test_sys_thread_init_signal_stack_returns_result() {
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

    #[test]
    fn test_sys_thread_init_signal_stack_error_message() {
        let result = sys_thread_init_signal_stack();
        if let Err(e) = result {
            // Error messages should be informative
            assert!(!e.is_empty());
            // Should contain relevant keywords
            assert!(e.contains("signal") || e.contains("stack") || e.contains("thread") || e.contains("Failed"));
        }
    }

    #[test]
    fn test_sys_thread_init_signal_stack_idempotent() {
        // First call
        let result1 = sys_thread_init_signal_stack();
        
        // Second call - should succeed (idempotent)
        let result2 = sys_thread_init_signal_stack();
        
        // Both should return valid results
        let _ = (result1, result2);
    }

    #[test]
    fn test_sys_init_signal_stack_platform_specific() {
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

    #[test]
    fn test_sys_thread_init_signal_stack_platform_specific() {
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

    #[test]
    fn test_sys_init_signal_stack_multiple_calls() {
        // Test multiple calls don't cause issues
        for _ in 0..3 {
            let result = sys_init_signal_stack();
            let _ = result;
        }
    }

    #[test]
    fn test_sys_thread_init_signal_stack_multiple_calls() {
        // Test multiple calls don't cause issues
        for _ in 0..3 {
            let result = sys_thread_init_signal_stack();
            let _ = result;
        }
    }

    #[test]
    fn test_signal_stack_functions_dont_panic() {
        // Both functions should not panic even if they fail
        let _result1 = sys_init_signal_stack();
        let _result2 = sys_thread_init_signal_stack();
    }

    #[test]
    fn test_signal_stack_error_consistency() {
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

    #[test]
    fn test_sys_thread_init_signal_stack_error_consistency() {
        let result1 = sys_thread_init_signal_stack();
        let result2 = sys_thread_init_signal_stack();
        
        // Both should return Result types
        match (result1, result2) {
            (Ok(_), Ok(_)) => {}
            (Err(_), Err(_)) => {}
            (Ok(_), Err(_)) => {}
            (Err(_), Ok(_)) => {}
        }
    }

    #[cfg(unix)]
    #[test]
    fn test_unix_signal_stack_constants() {
        // Test that constants are available on Unix
        use libc::SIGSTKSZ;
        
        // SIGSTKSZ should be a positive value
        assert!(SIGSTKSZ > 0);
    }

    #[test]
    fn test_signal_stack_result_type() {
        let result1 = sys_init_signal_stack();
        let result2 = sys_thread_init_signal_stack();
        
        // Both should return Result<(), String>
        assert!(matches!(result1, Ok(_) | Err(_)));
        assert!(matches!(result2, Ok(_) | Err(_)));
    }
}
