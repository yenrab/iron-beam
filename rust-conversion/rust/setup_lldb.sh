#!/bin/bash
cd /Volumes/Files_1/iron-beam/rust-conversion/rust

# Copy preloaded modules
cp target/otp_root/erts/preloaded/ebin/erl_init.beam target/debug/ 2>/dev/null || true
cp target/otp_root/erts/preloaded/ebin/init.beam target/debug/ 2>/dev/null || true

# Get JIT address
JIT_ADDR=$(echo "2+2." | target/debug/beam 2>&1 | grep "Instruction pointer: 0x" | grep -o "0x[0-9a-f]*")

echo "JIT Address: $JIT_ADDR"

# Launch LLDB with breakpoint
lldb target/debug/beam -o "breakpoint set -a $JIT_ADDR" -o "run"
