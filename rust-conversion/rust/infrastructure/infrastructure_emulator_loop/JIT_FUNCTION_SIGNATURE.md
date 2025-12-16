# JIT Function Signature Documentation

## C Implementation Analysis

Based on the C code in `erts/emulator/beam/jit/beam_jit_main.cpp`:

```cpp
void process_main(ErtsSchedulerData *esdp) {
    typedef void(ERTS_CCONV_JIT * pmain_type)(ErtsSchedulerData *);
    pmain_type pmain = (pmain_type)bga->get_process_main();
    pmain(esdp);
}
```

The JIT-compiled `process_main` function has the signature:
```cpp
void(ERTS_CCONV_JIT *)(ErtsSchedulerData *)
```

## Function Behavior

The JIT-compiled `process_main`:
1. Takes `ErtsSchedulerData *esdp` as parameter
2. Manages the entire execution loop internally
3. Reads the current process from scheduler data
4. Copies registers in/out
5. Reads `c_p->i` (instruction pointer) from the process
6. Jumps to the instruction pointer: `a.jmp(RET)` where RET contains `c_p->i`
7. The instruction pointer points to the start of a BEAM function's native code

## Rust Implementation Considerations

### Current Approach (Per-Process Execution)

The current Rust implementation calls `process_main()` per-process, which is different from the C approach. The C code executes the entire scheduler loop, while Rust executes individual processes.

### Options

#### Option 1: Match C Signature Exactly
- Create `ErtsSchedulerData` equivalent in Rust
- Call JIT-compiled `process_main` with scheduler data
- JIT code manages entire execution loop

**Function Signature:**
```rust
type JitProcessMain = unsafe extern "C" fn(*mut ErtsSchedulerData);
```

#### Option 2: Simplified Per-Process Execution
- Generate simpler JIT functions for individual BEAM functions
- Instruction pointer points to a BEAM function entry point
- Call the function directly

**Function Signature:**
```rust
// For individual BEAM functions
type JitBeamFunction = unsafe extern "C" fn(*mut Process, *mut Eterm) -> i32;
// Returns: 0 = continue, 1 = exit, -1 = error
```

#### Option 3: Hybrid Approach
- Keep `process_main` for scheduler loop (matches C)
- Use instruction pointer for individual BEAM functions
- Adapt Rust code to work with scheduler-based execution

## Recommendation

**Option 1** matches the C implementation most closely and ensures compatibility. However, it requires:
- Creating `ErtsSchedulerData` structure in Rust
- Adapting the Rust scheduler to work with this structure
- Ensuring the JIT code can access scheduler data correctly

## Current Implementation Status

The current Rust code uses **Option 2** approach (per-process execution), but this needs to be aligned with how `infrastructure_beamasm` actually generates code. The function signature should match what the JIT compiler produces.

## Next Steps

1. Determine how `infrastructure_beamasm` generates code:
   - Does it generate a full `process_main` loop?
   - Or does it generate individual BEAM function entry points?
   
2. Update `process_main()` in Rust to match the actual JIT function signature

3. Ensure instruction pointer handling matches JIT code expectations

