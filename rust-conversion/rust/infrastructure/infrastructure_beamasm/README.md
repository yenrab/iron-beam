# infrastructure_beamasm

Infrastructure Layer: BeamAsm JIT Execution

This crate provides load-time conversion of BEAM instructions into native code on x86-64 and aarch64 using the asmjit C++ library.

## Dependencies

This crate depends on the asmjit C++ library. The asmjit library must be:
1. Built and installed on the system, or
2. Built as part of the build process

## FFI Bindings

The crate uses FFI bindings to call asmjit C++ functions through a C++ wrapper library.

### C++ Wrapper

The C++ wrapper (`cpp/asmjit_wrapper.cpp`) exposes C functions that wrap asmjit C++ API calls. This allows Rust to call asmjit without dealing with C++ name mangling directly.

### Build Process

The build script (`build.rs`):
1. Compiles the C++ wrapper (`cpp/asmjit_wrapper.cpp`)
2. Links against the asmjit headers (embedded in `erts/emulator/asmjit/`)
3. Links against the C++ standard library

The asmjit library is header-only and embedded in the Erlang source tree, so no separate library linking is needed.

## Building

The build script (`build.rs`):
1. Compiles all asmjit core source files (`asmjit/core/*.cpp`)
2. Compiles architecture-specific asmjit files (`asmjit/x86/*.cpp` or `asmjit/arm/*.cpp`)
3. Compiles the C++ wrapper (`cpp/asmjit_wrapper.cpp`)
4. Links everything together

The asmjit library is embedded in the Erlang source tree at `erts/emulator/asmjit/`, so no separate installation is needed.

## Usage

```rust
use infrastructure_beamasm::{beamasm_init, beamasm_new_assembler};

// Initialize the JIT system
beamasm_init()?;

// Create an assembler for a module
let assembler = beamasm_new_assembler(module, num_labels, num_functions, beam_file)?;
```

## Architecture

- `asmjit_wrapper.rs` - Rust wrappers around asmjit C++ library
- `common.rs` - Common assembler functionality
- `jit.rs` - JIT code allocator
- `loader.rs` - Code loading and module management
- `metadata.rs` - Metadata tracking
- `arch/x86/` - x86-64 specific code
- `arch/arm/` - aarch64 specific code

