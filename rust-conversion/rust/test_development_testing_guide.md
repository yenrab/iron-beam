/*!
# Rust Erlang Compiler Development Testing Guide

This file documents the comprehensive testing methodology used during the development
of the Rust erlc compiler. This approach ensures that each feature implementation
produces correct, executable BEAM bytecode.

## Overview

The testing methodology focuses on **end-to-end validation** of the complete compilation
pipeline: Erlang Source → AST → Instructions → BEAM Bytecode. This approach validates
not just syntax correctness, but actual binary compatibility with the Erlang runtime.

## Testing Workflow

### 1. Feature Implementation
- Implement the new feature in the Rust compiler
- Add necessary AST parsing, instruction generation, and opcode encoding
- Build the project to ensure compilation succeeds

### 2. Create Test Erlang Module
Create a test `.erl` file that exercises the new feature:

```erlang
-module(test_feature_name).
-export([test_function/arity]).

test_function(Args) ->
    % Code that uses the new feature
    Result.
```

**Naming Convention:** Use descriptive names that indicate what feature is being tested.

### 3. Compile with Rust erlc
Run the Rust compiler on the test file:

```bash
cd /Volumes/Files_1/iron-beam/rust-conversion/rust
./target/debug/erlc -v test_feature_name.erl
```

**Expected Output:**
```
Compiling test_feature_name.erl
  → test_feature_name.beam
✓ Compiled test_feature_name.erl
Compiled 1 files: 1 succeeded, 0 failed
```

**Failure Indicators:**
- Compilation errors indicate syntax or implementation issues
- Missing success message means the feature isn't working

### 4. Validate BEAM File Structure
Examine the generated `.beam` file using hexdump:

```bash
hexdump -C test_feature_name.beam | head -20
```

**Verify BEAM Sections:**
- **IFF Header**: `46 4f 52 31` (FOR1) followed by size
- **BEAM Magic**: `42 45 41 4d` (BEAM)
- **Atom Table**: `41 74 55 38` (AtU8) with atom count and atoms
- **Code Section**: `43 6f 64 65` (Code) with header and instructions
- **Export Table**: `45 78 70 54` (ExpT) with exported functions
- **Function Table**: `46 75 6e 54` (FunT) with function metadata

### 5. Verify Feature-Specific Content
Check that the new feature generates correct opcodes and data:

**For Arithmetic Operations:**
- Look for opcodes: 0x1b (add), 0x1c (subtract), 0x1d (multiply), 0x1e (divide)
- Verify 4 arguments per instruction

**For Control Flow:**
- Case expressions: Look for case_end opcodes (0x4a)
- If expressions: Look for if_end opcodes (0x49)
- Labels and jumps: Verify label/jump instruction pairs

**For Function Calls:**
- Local calls: Call opcode (0x04) with label and arity
- External calls: CallExt opcode (0x07) with module/function atoms
- BIF calls: Bif0/Bif1/Bif2 opcodes (0x09-0x0b) with BIF indices

### 6. Run Encoder Tests
Execute the infrastructure tests to ensure opcode encoding works correctly:

```bash
cargo test -p infrastructure_beam_utilities beam_encoder
```

**Expected:** All tests pass without failures.

### 7. Clean Up Test Files
Remove test artifacts:

```bash
rm test_feature_name.erl test_feature_name.beam
```

## Feature-Specific Testing Examples

### Arithmetic Operations Testing
```erlang
-module(test_arithmetic).
-export([add/2, divide/2]).

add(X, Y) -> X + Y.
divide(X, Y) -> X / Y.
```

**Verification:**
- Code section contains arithmetic opcodes (0x1b, 0x1e)
- Atom table includes "add", "divide"
- Export table references correct atom indices

### Control Flow Testing
```erlang
-module(test_control_flow).
-export([case_test/1]).

case_test(X) ->
    case X of
        1 -> one;
        2 -> two;
        _ -> other
    end.
```

**Verification:**
- Code section contains case_end opcode (0x4a)
- Label instructions for clause entry points
- Proper jump/branch structure

### Function Calls Testing
```erlang
-module(test_calls).
-export([local/1, external/1]).

local(X) -> double(X).
double(Y) -> Y * 2.

external(X) -> math:sqrt(X).
```

**Verification:**
- Call opcodes (0x04) for local functions
- CallExt opcodes (0x07) for external functions
- Proper argument register setup

## Validation Checklist

### Compilation Phase
- [ ] `cargo build --bin erlc` succeeds
- [ ] No compilation errors or warnings in new code
- [ ] Dependencies resolve correctly

### Erlang Compilation Phase
- [ ] `./target/debug/erlc -v test_file.erl` succeeds
- [ ] Success message: "✓ Compiled test_file.erl"
- [ ] `.beam` file is generated with correct size

### BEAM Structure Validation
- [ ] IFF header present (FOR1 + size)
- [ ] BEAM magic bytes (BEAM)
- [ ] All required sections present (AtU8, Code, ExpT, FunT)
- [ ] Atom table contains expected symbols
- [ ] Export/function tables have correct counts and indices

### Feature-Specific Validation
- [ ] New opcodes appear in code section
- [ ] Correct argument counts and types
- [ ] Cross-references between tables are valid
- [ ] Version compatibility markers included

### Testing Infrastructure
- [ ] `cargo test -p infrastructure_beam_utilities beam_encoder` passes
- [ ] No test failures or panics
- [ ] All encoding edge cases covered

## Why This Testing Approach Works

### 1. End-to-End Validation
Tests the complete pipeline from source to executable bytecode, catching issues
that unit tests might miss.

### 2. Binary Correctness
Validates actual BEAM file generation, not just syntax or intermediate representations.

### 3. Runtime Compatibility
Ensures generated code is compatible with Erlang's BEAM loader and execution engine.

### 4. Regression Prevention
Comprehensive validation prevents new features from breaking existing functionality.

### 5. Incremental Development
Allows testing each feature in isolation while maintaining overall system integrity.

## Common Issues and Solutions

### Compilation Fails
- Check for syntax errors in test Erlang code
- Verify AST parsing handles new constructs
- Ensure instruction generation doesn't crash

### Wrong Opcodes Generated
- Verify BeamOpcode enum values match genop.tab
- Check argument encoding in BeamEncoder
- Validate instruction structure in expression compiler

### BEAM File Corrupt
- Check section ordering (AtU8, Code, StrT, ImpT, ExpT, FunT, Attr)
- Verify atom indices are correct (1-based)
- Ensure all required chunks are present

### Encoder Tests Fail
- Check BeamInstruction creation uses correct opcodes
- Verify BeamArg types match expected formats
- Ensure encoding handles all argument types

## Best Practices

1. **Test Incrementally**: Test each new feature before moving to the next
2. **Use Descriptive Names**: Test files should clearly indicate what's being tested
3. **Check All Sections**: Validate atom, export, function, and code sections
4. **Verify Cross-References**: Ensure table indices match atom table contents
5. **Clean Up**: Remove test files to keep workspace tidy
6. **Document Findings**: Note any edge cases or limitations discovered

## Maintenance

This testing methodology should be updated as the compiler evolves:
- Add new validation steps for major features
- Update opcode verification as new instructions are added
- Enhance structure validation for complex features
- Include performance validation for optimization features

Following this guide ensures that the Rust Erlang compiler produces correct,
compatible, and executable BEAM bytecode for all implemented language features.
*/
