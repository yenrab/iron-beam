# Entities Process Port

## Overview

This crate provides core data structure definitions for **Process** and **Port** - the fundamental VM data structures used throughout the Erlang/OTP runtime system.

## Purpose

In the CLEAN architecture design, the Process and Port struct definitions were not explicitly assigned to a behavior group. However, these are core data structures that should be in the **Entities layer** (innermost layer with no dependencies).

This crate fills that gap by providing:
- `Process` - Core Erlang process structure
- `Port` - Core Erlang port structure  
- Common types shared between Process and Port
- Process and Port flags, states, and constants

## Architecture

This crate is part of the **Entities layer** in CLEAN architecture:
- **Layer**: Entities (innermost)
- **Dependencies**: None (zero dependencies)
- **Used by**: Use Cases layer, Adapters layer, Infrastructure layer, Frameworks layer

## Structure

- `process.rs` - Process type definition and related types
- `port.rs` - Port type definition and related types
- `common.rs` - Common types shared by Process and Port (ErtsPTabElementCommon)

## Based On

- `erts/emulator/beam/erl_process.h` - Process struct definition
- `erts/emulator/beam/erl_port.h` - Port struct definition

## Usage

```rust
use entities_process_port::{Process, Port, PortId, ProcessId};

// Create a new process
let process = Process::new(123);

// Create a new port
let port = Port::new(456);

// Access process/port properties
let process_id = process.get_id();
let port_id = port.get_id();
```

## Note

This is a **simplified** version of the full C struct definitions. Some fields are represented as placeholders (e.g., `Option<*mut ()>`) that will be properly typed when the corresponding infrastructure is converted. The essential structure and types are in place and ready for use by higher layers.

