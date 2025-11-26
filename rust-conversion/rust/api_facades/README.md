# API Facades Layer

This crate provides API facades for 52 external callers (functions called from Erlang).

## Purpose

The API facades maintain **exact C function signatures** for Erlang/OTP compatibility while calling underlying Rust modules from inner layers.

## Key Principles

1. **Exact C Signatures**: All facades maintain the exact C function signatures (including parameter types) for Erlang compatibility
2. **No FFI Needed**: Since all C code has been reengineered to Rust, facades call Rust modules directly, not C code
3. **Type Mapping**: C types are mapped to Rust types (e.g., `i32` for C `int`, `u32` for C `unsigned int`)
4. **Safety**: Facades use `unsafe extern "C"` to maintain C calling convention

## Structure

- **nif_facades.rs**: NIF (Native Implemented Function) facades
- **driver_facades.rs**: Driver facades
- **bif_facades.rs**: BIF (Built-In Function) facades
- **common_facades.rs**: Common facades for other functions

## Implementation Status

This is a placeholder structure. The actual 52 external callers need to be:
1. Identified from the design files
2. Mapped to their C function signatures
3. Implemented as facades calling appropriate Rust modules

## Next Steps

1. Extract the 52 external caller function signatures from design files
2. Map each to appropriate Rust module(s)
3. Implement facades maintaining exact signatures
4. Add tests verifying Erlang compatibility

