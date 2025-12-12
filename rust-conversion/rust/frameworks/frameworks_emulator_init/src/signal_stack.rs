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
            let _result = sys_init_signal_stack();
        }
    }
}

