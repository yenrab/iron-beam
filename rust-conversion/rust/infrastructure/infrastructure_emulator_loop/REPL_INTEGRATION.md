# REPL Integration with BeamAsm JIT

## Overview

The Rust REPL implementation uses BeamAsm JIT execution in the same way as the C version, but reengineered as needed for Rust. This document describes how the REPL integrates with `infrastructure_beamasm` and `infrastructure_emulator_loop`.

## REPL Execution Flow

The REPL follows this flow, matching the C implementation:

1. **Scanning & Parsing** (`infrastructure_utilities::erl_scan`, `infrastructure_utilities::erl_parse`)
   - User input is scanned into tokens
   - Tokens are parsed into Erlang expressions (AST)

2. **BEAM Bytecode Compilation**
   - Parsed expressions are compiled to BEAM bytecode
   - This creates a BEAM module structure with instructions

3. **JIT Compilation** (`infrastructure_beamasm`)
   - BEAM bytecode is loaded via `BeamAsmLoader::prepare_emit()`
   - JIT compiler generates native code from BEAM instructions
   - Code is finalized via `BeamAsmLoader::finish_emit()`
   - Returns executable native code pointer

4. **Execution** (`infrastructure_emulator_loop`)
   - `process_main()` is called with the JIT-compiled code pointer
   - The emulator loop executes the native code directly
   - Results are returned and printed

## Key Differences from C Version

### C Version
- Has build switch (`BEAMASM`) between interpreter and JIT
- REPL can use either interpreter (`beam_emu.c`) or JIT (`beamasm`)
- Code loading happens in `beam_load.c` which calls `asm_load.c` for JIT

### Rust Version
- **Uses ONLY JIT execution** (no interpreter fallback)
- REPL expressions are always compiled to JIT code via `infrastructure_beamasm`
- Execution happens via `infrastructure_emulator_loop` calling JIT-compiled code
- `infrastructure_emulator_loop` depends on `infrastructure_beamasm` for JIT functionality

## Integration Points

### 1. Code Loading (`infrastructure_beamasm::BeamAsmLoader`)

```rust
use infrastructure_beamasm::BeamAsmLoader;

// Create loader
let mut loader = BeamAsmLoader::new()?;

// Prepare for code emission (compile BEAM bytecode to JIT)
let mut loader_state = loader.prepare_emit(
    module_atom,
    num_labels,
    num_functions,
    beam_bytecode,
)?;

// Generate native code
let (executable_ptr, writable_ptr, size) = loader.finish_emit(&mut loader_state)?;
```

### 2. Execution (`infrastructure_emulator_loop::process_main`)

```rust
use infrastructure_emulator_loop::{process_main, EmulatorLoop};

// Create emulator loop
let mut emulator_loop = EmulatorLoop::new();
emulator_loop.set_current_process(Some(process.clone()));

// Set instruction pointer to JIT-compiled code
emulator_loop.set_instruction_ptr(executable_ptr);

// Execute the JIT-compiled code
let result = process_main(&mut emulator_loop, init_done)?;
```

## REPL Implementation Requirements

When implementing the REPL shell:

1. **Use `infrastructure_beamasm` for JIT compilation**
   - All REPL expressions must be compiled via `BeamAsmLoader`
   - Do NOT use direct AST evaluation (like `erl_eval`)
   - Do NOT use interpreter-based execution

2. **Use `infrastructure_emulator_loop` for execution**
   - Call `process_main()` with JIT-compiled code pointers
   - The emulator loop executes native code, not bytecode

3. **Follow C version patterns**
   - Match the code loading sequence from `beam_load.c` → `asm_load.c`
   - Use the same JIT compilation pipeline
   - Execute via the same emulator loop interface

## Example REPL Flow

```rust
// 1. Scan and parse user input
let tokens = erl_scan::scan_until_dot(input)?;
let exprs = erl_parse::parse_repl_exprs(tokens)?;

// 2. Compile expressions to BEAM bytecode
let beam_bytecode = compile_expressions_to_beam(exprs)?;

// 3. JIT compile via infrastructure_beamasm
let mut loader = BeamAsmLoader::new()?;
let mut loader_state = loader.prepare_emit(module, num_labels, num_functions, &beam_bytecode)?;
let (executable_ptr, _, _) = loader.finish_emit(&mut loader_state)?;

// 4. Execute via infrastructure_emulator_loop
let mut emulator_loop = EmulatorLoop::new();
emulator_loop.set_current_process(Some(process.clone()));
emulator_loop.set_instruction_ptr(executable_ptr);
let result = process_main(&mut emulator_loop, init_done)?;

// 5. Print result
println!("{}", format_result(result));
```

## Notes

- The REPL uses the same JIT execution path as regular Erlang code
- No special REPL-only execution path exists
- All code execution goes through `infrastructure_beamasm` → `infrastructure_emulator_loop`
- This ensures consistency with the C implementation while leveraging Rust's type safety

