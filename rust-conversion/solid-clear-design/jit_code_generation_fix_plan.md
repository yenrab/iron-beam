# JIT Code Generation Fix Plan

## Overview

The preload system is now fully functional - modules load, JIT compile, and export tables are populated with real code pointers. However, the JIT-generated code crashes with "illegal hardware instruction" when executed. This plan outlines the steps needed to fix the JIT code generation to produce valid ARM64 machine code.

## Current Issue

- ✅ Export table works perfectly
- ✅ JIT compilation succeeds (generates 4-byte code)
- ✅ Code pointers are valid and executable
- ❌ Generated code causes "illegal hardware instruction" crash

## Step 1: Debug Current JIT Code Generation

### Goal
Understand what ARM64 instructions are actually being generated and why they cause crashes.

### Implementation Actions
- Add debug output to dump the raw bytes of generated machine code
- Use ARM64 disassembler to examine the actual instructions
- Compare with known valid ARM64 `ret` instruction encoding
- Check asmjit code generation, flattening, and relocation process

### Verification
- Identify the exact ARM64 instruction bytes being generated
- Confirm whether the issue is in asmjit usage, relocation, or instruction encoding

---

## Step 2: Fix ARM64 Code Generation

### Goal
Ensure asmjit generates valid, executable ARM64 machine code.

### Implementation Actions
- Verify asmjit `emit_ret()` generates correct ARM64 `ret` instruction (0xC0035FD6)
- Check that code flattening and unresolved link resolution work correctly
- Ensure proper memory alignment and permissions for executable code
- Validate that the code pointer points to the correct instruction

### Verification
- Generated code disassembles to valid ARM64 instructions
- Simple `ret` function executes without crashing
- Code pointer arithmetic is correct

---

## Step 3: Implement BEAM Calling Convention

### Goal
Match the BEAM runtime's function calling conventions and register usage.

### Implementation Actions
- Research BEAM function prologue/epilogue requirements
- Implement proper stack frame setup for BEAM functions
- Handle BEAM register mapping to ARM64 registers
- Add proper function return sequence beyond just `ret`

### Verification
- JIT functions can be called and return without crashing
- Function entry/exit preserves required BEAM state
- Register allocation follows BEAM conventions

---

## Step 4: Add Basic BEAM Opcode Translation

### Goal
Translate simple BEAM opcodes to ARM64 instead of just returning.

### Implementation Actions
- Implement translation for basic opcodes like `move`, `return`, `call`
- Create opcode dispatch mechanism in JIT compiler
- Add register allocation for BEAM virtual registers
- Implement basic control flow (function calls, returns)

### Verification
- Simple BEAM functions (just `return`) execute correctly
- Basic arithmetic and data movement works
- Function calls between JIT-compiled modules succeed

---

## Implementation Notes

### Debugging Strategy
- Use `objdump` or similar tools to disassemble generated code
- Add runtime instruction tracing
- Compare with working C implementation code generation

### Safety Considerations
- Ensure executable memory is properly protected
- Validate all code pointers before execution
- Add bounds checking for JIT-generated code

### Testing Approach
- Start with minimal functions (just `return`)
- Gradually add complexity (data movement, arithmetic)
- Test with real BEAM modules, not just synthetic ones

## Success Criteria

### Functional Requirements
- ✅ JIT-generated code executes without "illegal hardware instruction" errors
- ✅ Basic BEAM functions (return, move) work correctly
- ✅ Function calls between JIT modules succeed
- ✅ Erlang shell can execute simple expressions

### Performance Requirements
- ✅ JIT compilation overhead is reasonable (< 100ms per module)
- ✅ Generated code performance comparable to interpreted execution

### Reliability Requirements
- ✅ Invalid BEAM code fails compilation, not execution
- ✅ JIT failures don't crash the runtime
- ✅ Memory safety maintained in generated code
