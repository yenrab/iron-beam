#!/bin/bash
# Generate HTML coverage report for entire workspace
# Handles macOS resource fork file issues

set -e

cd /Volumes/LaCie/iron-beam/rust-conversion/rust

# Clean previous runs
rm -rf target/llvm-cov-target coverage-html

# Set environment to prevent resource fork files
export COPYFILE_DISABLE=1
export RUST_TEST_THREADS=1

# Run coverage generation
cargo llvm-cov --workspace --all-features --tests --lib --bins --html --output-dir coverage-html/llvm-cov 2>&1 | while IFS= read -r line; do
    # Filter out resource fork warnings but keep other output
    if [[ ! "$line" =~ "\._rust-.*profraw" ]]; then
        echo "$line"
    fi
done

# Clean up resource fork files that may have been created
find target/llvm-cov-target -name "._*" -type f -delete 2>/dev/null || true

# Fix profraw list if needed
if [ -f target/llvm-cov-target/rust-profraw-list ]; then
    find target/llvm-cov-target -name "*.profraw" ! -name "._*" -type f 2>/dev/null > target/llvm-cov-target/rust-profraw-list-fixed.txt 2>/dev/null || true
    if [ -s target/llvm-cov-target/rust-profraw-list-fixed.txt ]; then
        cp target/llvm-cov-target/rust-profraw-list-fixed.txt target/llvm-cov-target/rust-profraw-list
        # Try to merge manually
        /Users/yenrab/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/aarch64-apple-darwin/bin/llvm-profdata merge -sparse -f target/llvm-cov-target/rust-profraw-list -o target/llvm-cov-target/rust.profdata 2>/dev/null || true
    fi
fi

# Check if HTML report was generated
if [ -f coverage-html/llvm-cov/html/html/index.html ]; then
    # Move nested html directory contents up
    mv coverage-html/llvm-cov/html/html/* coverage-html/llvm-cov/html/ 2>/dev/null || true
    rmdir coverage-html/llvm-cov/html/html 2>/dev/null || true
fi

# Verify report exists
if [ -f coverage-html/llvm-cov/html/index.html ]; then
    echo ""
    echo "✅ HTML coverage report generated successfully!"
    echo "   Location: coverage-html/llvm-cov/html/index.html"
    echo "   Full path: $(realpath coverage-html/llvm-cov/html/index.html)"
else
    echo ""
    echo "⚠️  HTML report not found at expected location"
    echo "   Checking for alternative locations..."
    find coverage-html -name "index.html" 2>/dev/null | head -5
fi
