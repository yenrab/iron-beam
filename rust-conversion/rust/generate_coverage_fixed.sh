#!/bin/bash
# Generate HTML coverage report for entire workspace
# Handles macOS resource fork file issues by filtering profraw list

set -e

cd /Volumes/LaCie/iron-beam/rust-conversion/rust

# Clean previous runs
rm -rf target/llvm-cov-target coverage-html

# Set environment to prevent resource fork files
export COPYFILE_DISABLE=1
export RUST_TEST_THREADS=1

# Run coverage generation (this will create profraw files)
cargo llvm-cov --workspace --all-features --tests --lib --bins --html --output-dir coverage-html/llvm-cov 2>&1 | grep -v "\._rust-.*profraw" || true

# Clean up resource fork files
find target/llvm-cov-target -name "._*" -type f -delete 2>/dev/null || true

# Fix profraw list by filtering out resource fork files
if [ -f target/llvm-cov-target/rust-profraw-list ]; then
    # Filter out resource fork files and create clean list
    grep -v "\._rust-" target/llvm-cov-target/rust-profraw-list > target/llvm-cov-target/rust-profraw-list-clean.txt 2>/dev/null || \
    find target/llvm-cov-target -name "*.profraw" ! -name "._*" -type f 2>/dev/null > target/llvm-cov-target/rust-profraw-list-clean.txt 2>/dev/null || true
    
    if [ -s target/llvm-cov-target/rust-profraw-list-clean.txt ]; then
        # Replace the profraw list with cleaned version
        cp target/llvm-cov-target/rust-profraw-list-clean.txt target/llvm-cov-target/rust-profraw-list
        
        # Manually merge profraw files
        /Users/yenrab/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/aarch64-apple-darwin/bin/llvm-profdata merge -sparse -f target/llvm-cov-target/rust-profraw-list -o target/llvm-cov-target/rust.profdata 2>/dev/null || true
        
        # If merge succeeded, regenerate HTML report
        if [ -f target/llvm-cov-target/rust.profdata ]; then
            # Find test binaries and generate HTML
            find target/llvm-cov-target/debug/deps -name "*test*" -type f -executable 2>/dev/null | head -1 | while read binary; do
                mkdir -p coverage-html/llvm-cov/html
                llvm-cov show "$binary" -instr-profile=target/llvm-cov-target/rust.profdata --format=html -output-dir=coverage-html/llvm-cov/html 2>/dev/null || true
            done
        fi
    fi
fi

# Check if HTML report was generated (may be in nested directory)
if [ -f coverage-html/llvm-cov/html/html/index.html ]; then
    mv coverage-html/llvm-cov/html/html/* coverage-html/llvm-cov/html/ 2>/dev/null || true
    rmdir coverage-html/llvm-cov/html/html 2>/dev/null || true
fi

# Verify report exists
if [ -f coverage-html/llvm-cov/html/index.html ]; then
    echo ""
    echo "✅ HTML coverage report generated successfully!"
    echo "   Location: coverage-html/llvm-cov/html/index.html"
    realpath coverage-html/llvm-cov/html/index.html
else
    echo ""
    echo "⚠️  HTML report not found at expected location"
    find coverage-html -name "index.html" 2>/dev/null | head -3
fi




