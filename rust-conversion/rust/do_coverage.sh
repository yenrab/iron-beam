cd /Volumes/Files_1/iron-beam/rust-conversion/rust
LLVM_COV=/Users/leebarney/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/aarch64-apple-darwin/bin/llvm-cov \
LLVM_PROFDATA=/Users/leebarney/.rustup/toolchains/stable-aarch64-apple-darwin/lib/rustlib/aarch64-apple-darwin/bin/llvm-profdata \
cargo llvm-cov --workspace --all-features --tests --lib --bins --html --output-dir coverage-html/llvm-cov -j1
