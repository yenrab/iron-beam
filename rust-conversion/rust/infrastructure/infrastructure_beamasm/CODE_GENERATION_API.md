# Code Generation API Documentation

## Overview

This document describes the code generation implementation that matches the C BeamAsm JIT API. The generated code's API matches the C implementation exactly to ensure compatibility.

## C API Reference

### process_main Function

From `erts/emulator/beam/jit/beam_jit_main.cpp`:

```cpp
void process_main(ErtsSchedulerData *esdp) {
    typedef void(ERTS_CCONV_JIT * pmain_type)(ErtsSchedulerData *);
    pmain_type pmain = (pmain_type)bga->get_process_main();
    pmain(esdp);
}
```

**Signature**: `void(ERTS_CCONV_JIT *)(ErtsSchedulerData *)`

**Behavior**:
1. Takes `ErtsSchedulerData *esdp` as parameter
2. Allocates `ErtsSchedulerRegisters` on the stack
3. Sets up register pointers (x_reg_array, etc.)
4. Enters main execution loop
5. Reads `c_p->i` (instruction pointer) from process
6. Jumps to instruction pointer: `a.jmp(RET)`
7. Handles reductions, scheduling, and process state

## Rust Implementation

### Structures

#### ErtsSchedulerRegisters

Located in `src/scheduler_data.rs`, matches C structure exactly:

```rust
#[repr(C, align(64))]
pub struct ErtsSchedulerRegisters {
    pub aux_regs: AuxRegs,
    pub x_reg_array: [Eterm; ERTS_X_REGS_ALLOCATED],
    pub f_reg_array: [FloatDef; MAX_REG],
    pub start_time_i: *const c_void,
    pub start_time: u64,
}
```

#### ErtsSchedulerData

Located in `src/scheduler_data.rs`, matches C structure:

```rust
#[repr(C)]
pub struct ErtsSchedulerData {
    pub registers: *mut ErtsSchedulerRegisters,
    // ... other fields matching C structure
}
```

### Function Pointer Types

#### JitProcessMain

Matches C `void(ERTS_CCONV_JIT *)(ErtsSchedulerData *)`:

```rust
pub type JitProcessMain = unsafe extern "C" fn(*mut ErtsSchedulerData);
```

#### JitBeamFunction

For individual BEAM function entry points:

```rust
pub type JitBeamFunction = unsafe extern "C" fn(*mut c_void, *mut Eterm); // Process*, Eterm*
```

### Code Generation API

#### generate_process_main

Main entry point for generating process_main:

```rust
pub fn generate_process_main(allocator: &mut JitAllocator) -> Result<JitProcessMain, BeamAssemblerError>
```

**Usage**:
```rust
use infrastructure_beamasm::{generate_process_main, JitAllocator};

let mut allocator = JitAllocator::new()?;
let process_main_fn = generate_process_main(&mut allocator)?;

// Call process_main with scheduler data
let mut scheduler_data = ErtsSchedulerData::new();
unsafe {
    process_main_fn(&mut scheduler_data);
}
```

#### X86BeamGlobalAssembler

Global assembler that generates shared code fragments:

```rust
pub struct X86BeamGlobalAssembler {
    assembler: Assembler,
    process_main_ptr: Option<*const u8>,
}

impl X86BeamGlobalAssembler {
    pub fn emit_process_main(&mut self) -> Result<(), BeamAssemblerError>;
    pub fn codegen(&mut self, allocator: &mut JitAllocator) -> Result<JitProcessMain, BeamAssemblerError>;
    pub fn get_process_main(&self) -> Option<JitProcessMain>;
}
```

## Implementation Status

### Completed

1. ✅ **ErtsSchedulerData structure** - Matches C structure exactly
2. ✅ **ErtsSchedulerRegisters structure** - Matches C structure exactly
3. ✅ **Function pointer types** - JitProcessMain and JitBeamFunction match C API
4. ✅ **Global assembler infrastructure** - X86BeamGlobalAssembler structure
5. ✅ **Code generation API** - generate_process_main() function

### Pending

1. ⏳ **process_main instruction emission** - Actual x86-64 instruction generation
2. ⏳ **BEAM function entry point generation** - Individual function code generation
3. ⏳ **Instruction emitters** - BEAM opcode to x86-64 instruction conversion
4. ⏳ **Register setup code** - x_reg_array pointer setup, stack allocation
5. ⏳ **Execution loop code** - Reading c_p->i, jumping to instruction pointer

## Next Steps

1. **Implement x86-64 instruction emission**:
   - Stack allocation for ErtsSchedulerRegisters
   - Register pointer setup (centered at x_reg_array)
   - Main execution loop with jump to instruction pointer

2. **Implement BEAM function entry points**:
   - Function prologue/epilogue
   - Register access patterns
   - Instruction emission for BEAM opcodes

3. **Integration with emulator_loop**:
   - Update emulator_loop to use process_main API
   - Create ErtsSchedulerData from Rust Process structures
   - Handle scheduler loop execution

## Compatibility Notes

- All structures use `#[repr(C)]` to match C layout exactly
- Function pointers use `unsafe extern "C"` to match C calling convention
- Memory layout matches C structures to ensure JIT code can access them correctly
- The API matches the C code exactly to ensure generated code is compatible

## References

- C implementation: `erts/emulator/beam/jit/x86/process_main.cpp`
- C structures: `erts/emulator/beam/erl_process.h`
- C API: `erts/emulator/beam/jit/beam_jit_main.cpp`

