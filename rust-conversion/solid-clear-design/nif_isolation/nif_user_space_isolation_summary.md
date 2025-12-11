# NIF User-Space Isolation - Executive Summary

## Problem

Currently, NIFs run directly in the BEAM VM kernel. When a NIF crashes (panic, segfault, etc.), the entire VM crashes, affecting all processes.

## Solution Overview

Run NIFs in isolated "user space" contexts with:
- **Separate stacks** - Each NIF call gets its own stack to prevent overflow
- **Panic recovery** - Catch Rust panics and convert to process termination
- **Signal handling** - Catch crashes (SIGSEGV, etc.) and convert to errors
- **Process isolation** - Only the calling process is affected, not the VM

## Architecture

```
┌─────────────────────────────────────┐
│      BEAM VM Kernel (Safe)         │
│  ┌───────────────────────────────┐ │
│  │   Scheduler                   │ │
│  │   Process Management          │ │
│  │   Memory Management           │ │
│  └───────────────────────────────┘ │
│           │                         │
│           │ NIF Call                │
│           ▼                         │
│  ┌───────────────────────────────┐ │
│  │   NIF Executor                │ │
│  │   (Isolation Layer)           │ │
│  └───────────────────────────────┘ │
└─────────────────────────────────────┘
           │
           │ Isolated Execution
           ▼
┌─────────────────────────────────────┐
│   NIF User Space (Isolated)         │
│  ┌───────────────────────────────┐ │
│  │   Isolated Stack              │ │
│  │   Panic Handler               │ │
│  │   Signal Handler              │ │
│  │   NIF Function                │ │
│  └───────────────────────────────┘ │
└─────────────────────────────────────┘
```

## Key Components

### 1. NIF Execution Context
- Wraps each NIF call with isolation
- Provides separate stack space
- Manages panic/signal handlers

### 2. Stack Isolation
- Each NIF gets its own stack (default 1MB)
- Guard pages detect overflow
- Stack switching for isolation

### 3. Panic Recovery
- Catches Rust panics using `catch_unwind`
- Converts panics to process termination
- Preserves panic information for debugging

### 4. Signal Handling (Unix)
- Catches SIGSEGV, SIGBUS, SIGFPE, etc.
- Converts signals to process termination
- Uses alternate signal stack

## Execution Flow

```
1. Process calls NIF
   │
   ▼
2. VM kernel creates NIF execution context
   │
   ▼
3. Set up isolation:
   - Allocate isolated stack
   - Install panic handler
   - Install signal handlers (Unix)
   │
   ▼
4. Switch to isolated stack
   │
   ▼
5. Execute NIF function
   │
   ├─► Success: Return result to process
   │
   ├─► Panic: Catch panic, terminate process
   │
   └─► Signal: Catch signal, terminate process
   │
   ▼
6. Restore original stack and handlers
   │
   ▼
7. Return to VM kernel (or process termination)
```

## Benefits

1. **Reliability**: NIF crashes don't crash the VM
2. **Isolation**: Each NIF call is isolated
3. **Recovery**: Processes can handle NIF failures gracefully
4. **Compatibility**: Existing NIF API remains unchanged
5. **Performance**: Minimal overhead (~100-500ns per call)

## Performance Impact

- **Stack switching**: ~100-500ns per NIF call
- **Panic handler setup**: ~10-50ns
- **Signal handler setup**: ~100-200ns (one-time)
- **Total overhead**: <1μs per NIF call (acceptable for most use cases)

## Implementation Phases

1. **Phase 1**: Basic isolation (stack + panic recovery)
2. **Phase 2**: Signal handling (Unix)
3. **Phase 3**: Memory protection
4. **Phase 4**: Performance optimization
5. **Phase 5**: Make default (with opt-out)

## Error Handling

When a NIF crashes:
1. Error is caught by isolation layer
2. Calling process is terminated with error reason
3. VM kernel continues running normally
4. Other processes are unaffected

Error format: `{nif_panic, Reason, StackTrace}` or `{nif_signal, SignalNumber}`

## Example Usage

```rust
// In VM kernel (beam_jit_call_nif)
let executor = get_nif_executor();

match executor.execute_nif(process, nif_fn, args, |ctx| {
    // Execute NIF in isolated context
    nif_function(ctx, args)
}) {
    Ok(result) => result,
    Err(NifExecutionError::Panic(info)) => {
        // Terminate calling process, VM continues
        process.exit(create_error_term(info));
        THE_NON_VALUE
    }
    Err(e) => {
        // Handle other errors similarly
        process.exit(create_error_term(e));
        THE_NON_VALUE
    }
}
```

## Security Benefits

- Prevents NIFs from corrupting kernel state
- Isolates memory access
- Prevents privilege escalation
- Enables better auditing and monitoring

## Future Enhancements

- NIF timeouts
- Resource limits (CPU, memory)
- Advanced sandboxing (seccomp, namespaces)
- NIF monitoring and metrics
- Better debugging support
