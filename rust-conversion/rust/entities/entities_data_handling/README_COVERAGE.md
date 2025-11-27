# Code Coverage

This directory includes automated code coverage generation and web page updates.

## Quick Start

To generate a coverage report and automatically update the web page:

```bash
# Run the script from the entities_data_handling directory
./generate_coverage.sh
```

The script will:
- Run all tests with coverage instrumentation
- Generate HTML coverage reports
- Automatically update the main web page at `../../coverage-html/html/index.html`
- Display a summary with test results

## What It Does

1. **Runs tests with coverage instrumentation** using `cargo llvm-cov`
2. **Generates HTML coverage report** in `../../coverage-html/html/html/`
3. **Updates the main web page** at `../../coverage-html/html/index.html`
4. **Displays summary** with test results and file locations

## Coverage Report Location

- **Main report**: `rust-conversion/rust/coverage-html/html/index.html`
- **Detailed HTML files**: `rust-conversion/rust/coverage-html/html/html/`

Open the main report in your browser:
```bash
open rust-conversion/rust/coverage-html/html/index.html
# or
file:///Volumes/Files_1/iron-beam/rust-conversion/rust/coverage-html/html/index.html
```

## Requirements

- `cargo-llvm-cov` installed: `cargo install cargo-llvm-cov`
- `llvm-tools-preview` component: `rustup component add llvm-tools-preview`

## Integration

The script automatically:
- Detects LLVM tools from your Rust toolchain
- Updates the web page with latest coverage data
- Preserves tool information (llvm-cov)
- Shows test summary in the output

## Manual Update

If you need to manually run coverage without the script:

```bash
export LLVM_COV=/path/to/llvm-cov
export LLVM_PROFDATA=/path/to/llvm-profdata
cargo llvm-cov --lib --tests --html --output-dir ../../coverage-html/html/html
```

Then copy `../../coverage-html/html/html/index.html` to `../../coverage-html/html/index.html`

