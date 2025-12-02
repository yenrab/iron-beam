# Erlang/OTP C-to-Rust Conversion

This directory contains the Rust implementation of Erlang/OTP C code, organized following CLEAN architecture principles.

## Structure

The code is organized into CLEAN architecture layers:

- **entities/**: Innermost layer (5 crates) - Core data structures
- **usecases/**: Business logic (4 crates)
- **adapters/**: I/O and external interfaces (9 crates)
- **infrastructure/**: Utilities and helpers (10 crates)
- **frameworks/**: System integration (5 crates)
- **code_management/**: Code loading (1 crate)
- **api_facades/**: Erlang compatibility layer (1 crate)

## Compiling

### Compile All Crates

From the `rust-conversion/rust/` directory:

```bash
# Check all crates compile (fast)
cargo check --workspace

# Build all crates in debug mode
cargo build --workspace

# Build all crates in release mode (optimized)
cargo build --workspace --release

# Run all tests
cargo test --workspace

# Format all code
cargo fmt --workspace

# Check for linting issues
cargo clippy --workspace
```

### Compile Individual Crate

To compile a specific crate:

```bash
# Example: Compile entities_data_handling
cargo build -p entities_data_handling

# Example: Compile with tests
cargo test -p entities_data_handling
```

### Compile by Layer

To compile all crates in a specific layer:

```bash
# Compile all Entities layer crates
cargo build -p entities_data_handling -p entities_utilities -p entities_io_operations -p entities_system_integration_common -p entities_system_integration_win32

# Or use workspace and filter
cargo build --workspace --filter entities_*
```

## Dependencies

All crates follow CLEAN architecture dependency rules:
- Inner layers have no dependencies on outer layers
- Dependencies flow inward (outer → inner)
- Zero circular dependencies

## Testing

Run tests for all crates:

```bash
cargo test --workspace
```

Run tests for a specific crate:

```bash
cargo test -p entities_data_handling
```

## Code Coverage

To generate code coverage reports using `cargo-llvm-cov`:

```bash
# Setup environment (first time or when needed)
source ./setup_llvm_cov.sh

# Generate HTML coverage report
./coverage.sh

# Or generate LCOV format
./coverage.sh --lcov

# Or show summary only
./coverage.sh --summary

# Continue even if some tests fail
./coverage.sh --no-fail-fast
```

The coverage report will be generated in `coverage-html/llvm-cov/`.

**Requirements:**
- `cargo-llvm-cov` installed: `cargo install cargo-llvm-cov`
- `llvm-tools-preview` component OR LLVM installed via Homebrew

The `setup_llvm_cov.sh` script will automatically detect and configure LLVM tools.

## Requirements

- Rust 1.70+ (edition 2021)
- Cargo

## Status

- ✅ All 35 crates created
- ✅ All crates compile successfully
- ⏳ Implementation in progress (placeholder code)
- ⏳ Tests to be expanded
- ⏳ API facades (52 external callers) to be fully implemented

