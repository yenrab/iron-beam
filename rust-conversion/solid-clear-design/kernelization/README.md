# BEAM Kernel and Language Behavior Framework Design

This directory contains design documents for partitioning the existing Rust codebase into a language-agnostic BEAM kernel with a behavior framework for language implementations.

## Overview

The kernelization effort aims to:

1. **Partition existing Rust code** into kernel vs language-specific components
2. **Create a language behavior framework** (similar to OTP behaviors) with common templates
3. **Define behavior traits** that language implementations must provide
4. **Enable language developers** to implement callbacks rather than reimplementing common functionality

## Design Documents

### 1. [BEAM Kernel Partitioning Design](./beam_kernel_design.md)
Design for partitioning the existing Rust codebase into kernel and language behavior framework.

**Key Content:**
- Crate-by-crate partitioning strategy
- Language behavior framework concept (similar to OTP behaviors)
- Behavior traits for shell, compiler, application, distribution, BIFs, initialization
- Common framework implementations
- Language implementation structure
- File structure after partitioning

### 2. [Kernel API Design](./kernel_api_design.md)
Standard API specification for language implementations to interact with the BEAM kernel.

**Key Content:**
- Language-agnostic API interface
- Module management API
- Process management API
- BIF registration API
- Event and callback system
- Configuration API

## Architecture Principles

### Behavior Pattern (Similar to OTP Behaviors)

Just like OTP uses behaviors (gen_server, gen_statem, etc.) where you implement callbacks and the framework handles common parts, the language behavior framework provides:

- **Common Implementation**: Framework handles repetitive/common code
- **Trait Interfaces**: Languages implement callbacks
- **Code Reuse**: Less duplication, easier maintenance

### Three-Layer Architecture

1. **Kernel**: Language-agnostic core runtime (existing Rust crates)
2. **Language Behavior Framework**: Common templates/behaviors (new framework crates)
3. **Language Implementations**: Language-specific callbacks (Erl, Elixir, Gleam, etc.)

### Behavior Traits

Language implementations provide callbacks for:

- **Shell Behavior**: Parsing, compilation, formatting (framework handles REPL loop)
- **Compiler Behavior**: Source file location, compiler invocation (framework handles code paths)
- **Application Behavior**: App structure, callbacks (framework handles lifecycle)
- **Distribution Behavior**: Node naming, cookies (framework handles protocol)
- **BIF Behavior**: Language-specific BIFs (framework handles registration)
- **Initialization Behavior**: Language setup (framework handles orchestration)

## Benefits

### Code Reuse

- Common functionality implemented once in framework
- Languages only implement callbacks
- Less duplication, easier maintenance

### Consistency

- All languages follow same patterns
- Common behavior across languages
- Easier to understand and maintain

### Extensibility

- Easy to add new languages
- Just implement behavior traits
- Framework handles common parts

## Implementation Status

**Status**: Design Phase

These documents represent the design specification for partitioning the existing Rust codebase. Implementation will follow after design review and approval.

## Related Documents

- [NIF User-Space Isolation](../nif_isolation/README.md) - Design for NIF handling
- [SOLID/CLEAN Architecture](../README.md) - Overall architecture principles
- [Behavior Groups Mapping](../behavior-groups-mapping.jsonld) - Component organization

## Next Steps

1. Review and refine partitioning strategy
2. Define detailed behavior trait interfaces
3. Design common framework implementations
4. Plan migration path from current implementation
5. Create proof-of-concept implementation
