# Design Documents

This directory contains design documents for the Rust BEAM VM implementation.

## NIF User-Space Isolation

Design for running NIFs in isolated "user space" contexts to prevent NIF crashes from bringing down the entire VM.

### Documents

1. **[nif_user_space_isolation.md](./nif_user_space_isolation.md)** - Complete design document
   - Detailed architecture
   - Component specifications
   - Implementation details
   - Performance considerations
   - Security considerations
   - Migration path

2. **[nif_user_space_isolation_summary.md](./nif_user_space_isolation_summary.md)** - Executive summary
   - High-level overview
   - Key concepts
   - Architecture diagram
   - Benefits and trade-offs
   - Quick reference

3. **[nif_user_space_isolation_example.rs](./nif_user_space_isolation_example.rs)** - Code examples
   - Simplified implementation examples
   - Key concepts in code
   - Integration patterns
   - Test examples

### Quick Start

1. Read the **summary** for a high-level understanding
2. Review the **code examples** to see how it works
3. Read the **full design** for implementation details

### Key Concepts

- **Isolation**: NIFs run in isolated contexts with separate stacks
- **Panic Recovery**: Rust panics are caught and converted to process termination
- **Signal Handling**: Crashes (SIGSEGV, etc.) are caught and handled gracefully
- **Process Isolation**: Only the calling process is affected, not the VM kernel

### Status

This is a design document. Implementation status: **Not Started**
