# Building and Running the Erlang/OTP Emulator

This document provides instructions for compiling and running the Rust-based Erlang/OTP emulator.

## Prerequisites

- Rust toolchain (cargo 1.91.1 or later)
- All dependencies are managed by Cargo

## Compilation

### Debug Build (Faster compilation, unoptimized)

```bash
cd /Volumes/LaCie/iron-beam/rust-conversion/rust
cargo build -p frameworks_emulator_init
```

The binary will be located at: `target/debug/beam`

### Release Build (Optimized, recommended for running)

```bash
cd /Volumes/LaCie/iron-beam/rust-conversion/rust
cargo build --release -p frameworks_emulator_init
```

The binary will be located at: `target/release/beam`

## Running the Emulator

### Basic Usage

```bash
./target/release/beam [options]
```

### Common Options

- `--help` - Show help message
- `--sname <name>@<host>` - Set short node name (e.g., `--sname mynode@localhost`)
- `--name <name>@<host>` - Set long node name
- `--boot <script>` - Boot script name (e.g., `--boot start`)
- `--config <file>` - Configuration file (e.g., `--config sys`)

### Example Commands

```bash
# Show help
./target/release/beam --help

# Start with short node name and boot script
./target/release/beam --sname mynode@localhost --boot start --config sys

# Start with default settings
./target/release/beam
```

## Quick Start

1. **Compile:**
   ```bash
   cd /Volumes/LaCie/iron-beam/rust-conversion/rust
   cargo build --release -p frameworks_emulator_init
   ```

2. **Run:**
   ```bash
   ./target/release/beam --sname mynode@localhost --boot start --config sys
   ```

## Troubleshooting

- If compilation fails, ensure all dependencies are available
- Check that you're in the correct directory (`rust-conversion/rust`)
- For verbose output, use `RUST_LOG=debug` environment variable
- The emulator will print progress messages to stderr during boot

## Current Status

The emulator currently supports:
- ✅ Command-line argument parsing
- ✅ Boot script loading and parsing
- ✅ Module loading from boot scripts
- ✅ Kernel process spawning
- ✅ Process name registration
- ✅ Code path management
- ✅ Function application framework

Note: Full REPL functionality is still under development.

