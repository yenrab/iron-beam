# NIF User-Space Isolation Design

## Overview

This document describes a design for running NIFs (Native Implemented Functions) in "user space" within the Rust BEAM VM, such that when a NIF crashes, it does not crash the entire VM kernel. This design provides process-level isolation for NIF execution while maintaining performance and compatibility with the existing NIF API.

## Problem Statement

Currently, all NIFs run directly in the kernel space of the BEAM VM. When a NIF:
- Panics (in Rust)
- Crashes (segfault, abort, etc.)
- Triggers undefined behavior
- Runs out of stack space

The entire VM crashes, affecting all processes and the scheduler. This is a critical reliability issue.

## Design Goals

1. **Isolation**: NIF crashes should only affect the calling process, not the VM kernel
2. **Performance**: Minimal overhead for NIF execution
3. **Compatibility**: Maintain compatibility with existing NIF API
4. **Recovery**: Graceful error handling and process termination on NIF failure
5. **Safety**: Prevent NIFs from corrupting VM kernel state

## Architecture Overview

The design uses a **sandboxed execution model** where NIFs run in isolated contexts with:
- Separate stack space
- Signal/panic handlers
- Memory protection boundaries
- Error recovery mechanisms

## Key Components

### 1. NIF Execution Context (NifExecutionContext)

A per-NIF-call execution context that provides isolation:

```rust
pub struct NifExecutionContext {
    /// Isolated stack for NIF execution
    stack: NifStack,
    /// Signal/panic handler state
    panic_handler: Arc<PanicHandler>,
    /// Process reference (for heap access)
    process: Arc<Process>,
    /// NIF function pointer
    nif_fn: NifFunctionPtr,
    /// Execution state
    state: NifExecutionState,
    /// Error recovery information
    recovery: Option<NifRecoveryInfo>,
}

pub enum NifExecutionState {
    /// NIF is ready to execute
    Ready,
    /// NIF is currently executing
    Executing,
    /// NIF completed successfully
    Completed,
    /// NIF panicked/crashed
    Panicked,
    /// NIF was aborted due to timeout or resource limits
    Aborted,
}
```

### 2. NIF Stack Isolation

Each NIF call gets its own stack space to prevent stack overflow from affecting the kernel:

```rust
pub struct NifStack {
    /// Stack memory (guarded page at bottom for overflow detection)
    memory: Vec<u8>,
    /// Stack pointer bounds
    stack_ptr: *mut u8,
    /// Stack size
    size: usize,
    /// Guard page size (for overflow detection)
    guard_size: usize,
}

impl NifStack {
    /// Create a new isolated stack for NIF execution
    pub fn new(size: usize) -> Result<Self, NifStackError> {
        // Allocate stack with guard page
        // Use mmap/mprotect for memory protection on Unix
        // Use VirtualAlloc/VirtualProtect on Windows
    }
    
    /// Switch to this stack for NIF execution
    pub unsafe fn switch_to(&mut self, f: impl FnOnce()) -> Result<(), NifStackError> {
        // Save current stack pointer
        // Switch to NIF stack
        // Execute function
        // Restore original stack
    }
}
```

### 3. Panic Handler

Catches Rust panics and converts them to recoverable errors:

```rust
pub struct PanicHandler {
    /// Panic information if panic occurred
    panic_info: Arc<RwLock<Option<PanicInfo>>>,
    /// Panic hook (replaces default panic hook during NIF execution)
    original_hook: Option<Box<dyn Fn(&PanicInfo) + Send + Sync>>,
}

impl PanicHandler {
    /// Set up panic handler for NIF execution
    pub fn setup(&self) {
        // Replace panic hook with our handler
        // Store original hook for restoration
    }
    
    /// Restore original panic handler
    pub fn restore(&self) {
        // Restore original panic hook
    }
    
    /// Check if panic occurred
    pub fn check_panic(&self) -> Option<PanicInfo> {
        // Return panic information if available
    }
}
```

### 4. Signal Handler (Unix)

Catches signals (SIGSEGV, SIGBUS, etc.) and converts them to errors:

```rust
#[cfg(unix)]
pub struct NifSignalHandler {
    /// Original signal handlers
    original_handlers: HashMap<i32, SignalHandler>,
    /// Signal mask for NIF execution
    signal_mask: SigSet,
}

#[cfg(unix)]
impl NifSignalHandler {
    /// Set up signal handlers for NIF execution
    pub fn setup(&mut self) -> Result<(), SignalError> {
        // Install signal handlers for:
        // - SIGSEGV (segmentation fault)
        // - SIGBUS (bus error)
        // - SIGFPE (floating point exception)
        // - SIGILL (illegal instruction)
        // - SIGABRT (abort)
    }
    
    /// Restore original signal handlers
    pub fn restore(&mut self) -> Result<(), SignalError> {
        // Restore original handlers
    }
}
```

### 5. NIF Execution Wrapper

The main wrapper that executes NIFs in isolation:

```rust
pub struct NifExecutor {
    /// Default stack size for NIF execution
    default_stack_size: usize,
    /// Maximum stack size allowed
    max_stack_size: usize,
    /// Signal handler (Unix only)
    #[cfg(unix)]
    signal_handler: NifSignalHandler,
}

impl NifExecutor {
    /// Execute a NIF in isolated context
    pub fn execute_nif<F, R>(
        &self,
        process: Arc<Process>,
        nif_fn: NifFunctionPtr,
        args: &[Eterm],
        f: F,
    ) -> Result<R, NifExecutionError>
    where
        F: FnOnce(&NifExecutionContext) -> R,
    {
        // 1. Create execution context with isolated stack
        let mut ctx = NifExecutionContext::new(process, nif_fn)?;
        
        // 2. Set up panic handler
        let panic_handler = ctx.panic_handler.clone();
        panic_handler.setup();
        
        // 3. Set up signal handlers (Unix)
        #[cfg(unix)]
        self.signal_handler.setup()?;
        
        // 4. Execute NIF in isolated context
        let result = unsafe {
            ctx.stack.switch_to(|| {
                // Execute the actual NIF function
                f(&ctx)
            })
        };
        
        // 5. Check for panics
        if let Some(panic_info) = panic_handler.check_panic() {
            return Err(NifExecutionError::Panic(panic_info));
        }
        
        // 6. Restore handlers
        panic_handler.restore();
        #[cfg(unix)]
        self.signal_handler.restore()?;
        
        // 7. Return result
        result
    }
}
```

### 6. Integration with VM Kernel

Modify the NIF calling code in the VM kernel to use the isolated executor:

```rust
// In beam_jit_call_nif or equivalent
pub fn beam_jit_call_nif(
    c_p: Arc<Process>,
    I: ErtsCodePtr,
    reg: &mut [Eterm],
    fp: BeamJitNifF,
    nif_mod: &ErlModuleNif,
) -> Eterm {
    let executor = get_nif_executor(); // Global or per-scheduler executor
    
    // Prepare NIF environment
    let env = create_nif_env(c_p.clone());
    
    // Execute NIF in isolated context
    match executor.execute_nif(c_p.clone(), fp as NifFunctionPtr, reg, |ctx| {
        // Call the actual NIF function
        let nif_result = unsafe {
            (fp)(&env, reg.len() as i32, reg.as_mut_ptr())
        };
        nif_result
    }) {
        Ok(result) => {
            // NIF completed successfully
            if is_non_value(result) {
                // Handle exception
                handle_nif_exception(c_p, result);
            }
            result
        }
        Err(NifExecutionError::Panic(panic_info)) => {
            // NIF panicked - terminate calling process
            terminate_process_on_nif_panic(c_p, panic_info);
            THE_NON_VALUE
        }
        Err(NifExecutionError::Signal(signal)) => {
            // NIF crashed with signal - terminate calling process
            terminate_process_on_nif_signal(c_p, signal);
            THE_NON_VALUE
        }
        Err(NifExecutionError::StackOverflow) => {
            // NIF stack overflow - terminate calling process
            terminate_process_on_nif_stack_overflow(c_p);
            THE_NON_VALUE
        }
        Err(e) => {
            // Other error
            terminate_process_on_nif_error(c_p, e);
            THE_NON_VALUE
        }
    }
}
```

### 7. Process Termination on NIF Failure

When a NIF crashes, terminate only the calling process:

```rust
fn terminate_process_on_nif_panic(
    process: Arc<Process>,
    panic_info: PanicInfo,
) {
    // Create error term
    let error_term = create_nif_panic_error(panic_info);
    
    // Terminate the process with 'nif_panic' reason
    process.exit(error_term);
    
    // Log the error (but don't crash VM)
    log::error!(
        "NIF panic in process {}: {:?}",
        process.id(),
        panic_info
    );
}

fn terminate_process_on_nif_signal(
    process: Arc<Process>,
    signal: i32,
) {
    // Create error term
    let error_term = create_nif_signal_error(signal);
    
    // Terminate the process
    process.exit(error_term);
    
    // Log the error
    log::error!(
        "NIF signal {} in process {}",
        signal,
        process.id()
    );
}
```

## Implementation Details

### Stack Switching

Stack switching is platform-specific:

**Unix (Linux/macOS):**
- Use `mmap` to allocate stack memory
- Use `mprotect` to set guard page as non-readable/non-writable
- Use inline assembly or `libc::setjmp`/`libc::longjmp` for stack switching
- Or use `pthread_attr_setstack` with a custom stack

**Windows:**
- Use `VirtualAlloc` to allocate stack memory
- Use `VirtualProtect` to set guard page protection
- Use structured exception handling (SEH) for error recovery
- Use `_chkstk` for stack overflow detection

### Panic Recovery

Rust panics can be caught using `std::panic::catch_unwind`:

```rust
pub fn execute_nif_with_panic_recovery<F, R>(f: F) -> Result<R, PanicInfo>
where
    F: FnOnce() -> R + UnwindSafe,
{
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(result) => Ok(result),
        Err(panic) => {
            // Extract panic information
            let panic_info = extract_panic_info(panic);
            Err(panic_info)
        }
    }
}
```

### Signal Handling (Unix)

Use `sigaction` to install signal handlers:

```rust
#[cfg(unix)]
fn setup_signal_handlers() -> Result<(), SignalError> {
    let signals = [SIGSEGV, SIGBUS, SIGFPE, SIGILL, SIGABRT];
    
    for signal in &signals {
        let mut sa: libc::sigaction = unsafe { std::mem::zeroed() };
        sa.sa_sigaction = nif_signal_handler as usize;
        sa.sa_flags = libc::SA_SIGINFO | libc::SA_ONSTACK;
        
        // Set alternate stack for signal handling
        set_alternate_signal_stack()?;
        
        unsafe {
            if libc::sigaction(*signal, &sa, std::ptr::null_mut()) != 0 {
                return Err(SignalError::SetupFailed);
            }
        }
    }
    
    Ok(())
}

#[cfg(unix)]
extern "C" fn nif_signal_handler(
    sig: i32,
    info: *mut libc::siginfo_t,
    _context: *mut libc::c_void,
) {
    // Store signal information in thread-local storage
    // Use setjmp/longjmp or similar to return to safe code
    // This should be coordinated with the NIF executor
}
```

### Memory Protection

Use memory protection to prevent NIFs from accessing kernel memory:

```rust
pub struct MemoryProtection {
    /// Protected memory regions (kernel structures, etc.)
    protected_regions: Vec<MemoryRegion>,
}

impl MemoryProtection {
    /// Check if an address is in a protected region
    pub fn is_protected(&self, addr: *const u8) -> bool {
        self.protected_regions.iter().any(|region| {
            region.contains(addr)
        })
    }
    
    /// Set up memory protection for NIF execution
    pub fn setup(&mut self) -> Result<(), MemoryProtectionError> {
        // Mark kernel memory regions as protected
        // This is primarily for documentation/logging
        // Actual protection relies on address space isolation
    }
}
```

## Performance Considerations

### Stack Allocation

- **Pooled stacks**: Reuse stack memory across NIF calls to reduce allocation overhead
- **Stack size**: Default to reasonable size (e.g., 1MB), allow configuration
- **Guard pages**: Use OS guard pages for automatic overflow detection

### Overhead

- **Stack switching**: ~100-500ns overhead per NIF call
- **Panic handler setup**: ~10-50ns (mostly just setting a flag)
- **Signal handler setup**: ~100-200ns (one-time per scheduler thread)

### Optimization Strategies

1. **Lazy stack allocation**: Only allocate stack when NIF is actually called
2. **Stack pooling**: Reuse stacks across NIF calls
3. **Fast path**: For trusted NIFs, skip some isolation checks
4. **Batch execution**: Group multiple NIF calls to amortize setup costs

## Error Handling

### Error Types

```rust
#[derive(Debug)]
pub enum NifExecutionError {
    /// NIF panicked (Rust panic)
    Panic(PanicInfo),
    /// NIF crashed with signal (Unix)
    #[cfg(unix)]
    Signal(i32),
    /// Stack overflow
    StackOverflow,
    /// Memory protection violation
    MemoryProtectionViolation,
    /// Timeout (if timeout mechanism is implemented)
    Timeout,
    /// Resource limit exceeded
    ResourceLimitExceeded,
    /// Other error
    Other(String),
}
```

### Error Propagation

Errors are converted to Erlang terms and cause the calling process to exit:

```rust
fn create_nif_panic_error(panic_info: PanicInfo) -> Eterm {
    // Create {nif_panic, Reason, StackTrace} tuple
    // This is similar to how Erlang handles errors
}
```

## Security Considerations

1. **Address Space Layout Randomization (ASLR)**: Ensure NIFs can't predict kernel addresses
2. **Memory Protection**: Prevent NIFs from accessing kernel data structures directly
3. **Stack Canaries**: Use stack canaries to detect stack corruption
4. **Control Flow Integrity**: Consider CFI for additional protection

## Testing Strategy

1. **Unit tests**: Test stack switching, panic recovery, signal handling
2. **Integration tests**: Test NIF execution with various failure modes
3. **Stress tests**: Test with many concurrent NIF calls
4. **Crash tests**: Intentionally crash NIFs to verify isolation

## Migration Path

1. **Phase 1**: Implement basic isolation (stack switching, panic recovery)
2. **Phase 2**: Add signal handling (Unix)
3. **Phase 3**: Add memory protection and advanced features
4. **Phase 4**: Optimize performance
5. **Phase 5**: Make it the default (with opt-out for trusted NIFs)

## Future Enhancements

1. **NIF Timeouts**: Add timeout mechanism for long-running NIFs
2. **Resource Limits**: Limit CPU time, memory usage per NIF
3. **NIF Sandboxing**: Further isolation using seccomp, namespaces, etc.
4. **NIF Monitoring**: Metrics and observability for NIF execution
5. **NIF Debugging**: Better debugging support for isolated NIFs

## References

- [Erlang NIF Documentation](https://www.erlang.org/doc/man/erl_nif.html)
- [Rust Panic Recovery](https://doc.rust-lang.org/std/panic/fn.catch_unwind.html)
- [Signal Handling in Rust](https://docs.rs/signal-hook/latest/signal_hook/)
- [Stack Switching Techniques](https://en.wikipedia.org/wiki/Setjmp.h)
