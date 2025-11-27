#!/bin/bash
# Generate code coverage report and update the web page
# This script runs llvm-cov to generate coverage and updates the HTML report
#
# Usage: ./generate_coverage.sh
# Or: cargo coverage (if alias is set up)

set -e

# Get the script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Navigate from entities_data_handling to rust-conversion/rust
RUST_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
COVERAGE_DIR="$RUST_DIR/coverage-html/html"

# Set LLVM tools path
RUST_TOOLCHAIN=$(rustup show active-toolchain | cut -d' ' -f1)
RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}"
HOST_TARGET=$(rustc -vV | grep host | awk '{print $2}')
LLVM_COV="$RUSTUP_HOME/toolchains/$RUST_TOOLCHAIN/lib/rustlib/$HOST_TARGET/bin/llvm-cov"
LLVM_PROFDATA="$RUSTUP_HOME/toolchains/$RUST_TOOLCHAIN/lib/rustlib/$HOST_TARGET/bin/llvm-profdata"

# Check if llvm-cov exists
if [ ! -f "$LLVM_COV" ]; then
    echo "Error: llvm-cov not found at $LLVM_COV"
    echo "Please install llvm-tools-preview: rustup component add llvm-tools-preview"
    exit 1
fi

# Export for cargo-llvm-cov
export LLVM_COV
export LLVM_PROFDATA

echo "========================================="
echo "Generating Code Coverage Report"
echo "========================================="
cd "$SCRIPT_DIR"

# Generate coverage report
echo "Running tests with coverage instrumentation..."
cargo llvm-cov --lib --tests --html --output-dir "$COVERAGE_DIR/html" 2>&1 | tee /tmp/coverage_output.log

# Update the main index.html with tool information
MAIN_INDEX="$COVERAGE_DIR/index.html"
HTML_INDEX="$COVERAGE_DIR/html/index.html"

if [ -f "$HTML_INDEX" ]; then
    # Copy the generated index.html to the main location
    echo "Updating coverage web page..."
    cp "$HTML_INDEX" "$MAIN_INDEX"
    
    # Ensure the tool information is present (if not already there)
    if ! grep -q "Coverage tool: llvm-cov" "$MAIN_INDEX"; then
        # Add tool information after the Created line
        if [[ "$OSTYPE" == "darwin"* ]]; then
            # macOS sed
            sed -i '' 's|<h4>Created:.*</h4>|&\n<p>Coverage tool: llvm-cov (LLVM source-based code coverage)</p>|' "$MAIN_INDEX"
        else
            # Linux sed
            sed -i 's|<h4>Created:.*</h4>|&\n<p>Coverage tool: llvm-cov (LLVM source-based code coverage)</p>|' "$MAIN_INDEX"
        fi
    fi
    
    # Extract test summary from the output
    TEST_SUMMARY=$(grep -E "test result.*passed" /tmp/coverage_output.log | tail -1 || echo "")
    
    echo ""
    echo "========================================="
    echo "Coverage Report Generated Successfully!"
    echo "========================================="
    echo "Main report: $MAIN_INDEX"
    echo "HTML files:  $COVERAGE_DIR/html/"
    if [ -n "$TEST_SUMMARY" ]; then
        echo "Test status: $TEST_SUMMARY"
    fi
    echo ""
    echo "Open the report in your browser:"
    echo "  file://$MAIN_INDEX"
    echo ""
else
    echo "Error: Generated index.html not found at $HTML_INDEX"
    exit 1
fi

