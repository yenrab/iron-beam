# Creating Safe Loadable Libraries

This guide explains how to create Rust dynamic libraries that can be loaded by the `DynamicLibraryLoader` module.

## Overview

The `DynamicLibraryLoader` uses a **multi-layered verification system** to ensure that only Rust libraries (not C libraries) that are explicitly designed for this system can be loaded. This verification consists of:

1. **Custom Safety Marker**: A required function that libraries must export to opt-in to this system
2. **Rust-Specific Symbol Detection**: Verification that the library is actually a Rust library by checking for Rust-specific symbols

This dual approach ensures that:
- Only Rust libraries can be loaded (C libraries are rejected)
- Only libraries explicitly designed for this system can be loaded
- The verification is reliable and difficult to bypass

## Automatic Compilation

**New Feature**: The `DynamicLibraryLoader` now supports automatic compilation of Rust source files!

If you provide a Rust source file (`.rs`) instead of a compiled library, the loader will automatically:

1. **Verify Safe Rust**: Check that the source file contains only safe Rust (no `unsafe` blocks)
2. **Compile On-The-Fly**: Compile the source file using the Rust toolchain (`cargo`)
3. **Load the Library**: Load the compiled library using the standard verification process

This enables a workflow where NIFs can be provided as source code and compiled automatically on first load, ensuring that only safe Rust code can be compiled and loaded.

### Using Automatic Compilation

Simply provide a path to a Rust source file (`.rs`) instead of a compiled library:

```rust
use usecases_bifs::DynamicLibraryLoader;

let process_id = DynamicLibraryLoader::allocate_process_id();
let source_path = std::path::Path::new("/path/to/my_nif.rs");  // Note: .rs extension
let options = usecases_bifs::LoadOptions::default();

match DynamicLibraryLoader::try_load(source_path, "my_nif", options, process_id) {
    Ok(result) => println!("Library compiled and loaded: {:?}", result),
    Err(usecases_bifs::LibraryError::UnsafeCodeInSource { locations }) => {
        println!("Unsafe code found in source:");
        for loc in locations {
            println!("  - {}", loc);
        }
    }
    Err(usecases_bifs::LibraryError::CompilationError { message, details }) => {
        println!("Compilation failed: {}\n{}", message, details);
    }
    Err(e) => println!("Error: {:?}", e),
}
```

### Compilation Requirements

For automatic compilation to work:

1. **Rust Toolchain**: The Rust toolchain (`cargo`) must be available in `PATH`
2. **Safe Rust Only**: The source file must contain only safe Rust (no `unsafe` blocks)
3. **Required Marker**: The compiled library must still export the safety marker function

### Source File Format

Your Rust source file should be a complete library crate that:

1. Exports the required safety marker function
2. Is configured as a `cdylib` (this is handled automatically during compilation)
3. Contains only safe Rust code

Example source file (`my_nif.rs`):

```rust
/// Safety marker - REQUIRED for this library to be loadable
#[no_mangle]
pub extern "C" fn rust_safe_library_marker() -> u32 {
    0x53414645 // "SAFE" in ASCII
}

/// Your NIF functions here
#[no_mangle]
pub extern "C" fn my_nif_function(input: i32) -> i32 {
    input * 2
}
```

When this file is loaded, it will be automatically compiled and the resulting library will be loaded.

## Why Multi-Layered Verification is Required

Rust does not provide built-in runtime verification of library safety. Dynamic library loading is inherently unsafe because:

1. The compiler cannot verify code loaded at runtime
2. Foreign function interfaces (FFI) require `unsafe` blocks
3. There is no standard way to verify a library's safety properties
4. Arbitrary libraries (including C libraries) could contain unsafe code or malicious behavior

Our multi-layered verification approach addresses these concerns by:
- Requiring explicit opt-in from library authors (custom marker)
- Verifying the library is actually Rust (Rust-specific symbol detection)
- Making it extremely difficult for C libraries to pass verification

## Required Safety Marker Function

Every loadable library **must** export this exact function:

```rust
/// Safety marker function required for dynamic library loading.
/// This function must be exported with #[no_mangle] to be discoverable
/// by the dynamic library loader.
#[no_mangle]
pub extern "C" fn rust_safe_library_marker() -> u32 {
    // Return "SAFE" in ASCII (0x53414645)
    // This value is verified by the loader to ensure the library
    // was designed for this system
    0x53414645
}
```

### Function Requirements

- **Name**: Must be exactly `rust_safe_library_marker` (case-sensitive)
- **Signature**: Must be `extern "C" fn() -> u32`
- **Return Value**: Must return `0x53414645` (ASCII "SAFE")
- **Visibility**: Must be `pub` and exported with `#[no_mangle]`
- **Calling Convention**: Must use `extern "C"` for C ABI compatibility

## Complete Example

Here's a complete example of a loadable library:

### Cargo.toml

```toml
[package]
name = "my_safe_library"
version = "0.1.0"
edition = "2021"

[lib]
# Must be cdylib for dynamic library support
crate-type = ["cdylib"]
```

### src/lib.rs

```rust
/// Safety marker - REQUIRED for this library to be loadable
/// This function must be exported exactly as shown.
#[no_mangle]
pub extern "C" fn rust_safe_library_marker() -> u32 {
    0x53414645 // "SAFE" in ASCII - this exact value is required
}

/// Your library's public API
#[no_mangle]
pub extern "C" fn my_library_function(input: i32) -> i32 {
    input * 2
}

/// Another example function
#[no_mangle]
pub extern "C" fn add_numbers(a: i32, b: i32) -> i32 {
    a + b
}
```

## Building the Library

Build your library as a dynamic library:

```bash
cargo build --release
```

This will create:
- **Linux**: `target/release/libmy_safe_library.so`
- **macOS**: `target/release/libmy_safe_library.dylib`
- **Windows**: `target/release/my_safe_library.dll`

## Loading the Library

Once built, you can load it using `DynamicLibraryLoader`:

```rust
use usecases_bifs::DynamicLibraryLoader;

let process_id = DynamicLibraryLoader::allocate_process_id();
let path = std::path::Path::new("/path/to/library");
let options = usecases_bifs::LoadOptions::default();

match DynamicLibraryLoader::try_load(path, "my_safe_library", options, process_id) {
    Ok(result) => println!("Library loaded: {:?}", result),
    Err(usecases_bifs::LibraryError::UnsafeLibrary) => {
        println!("Library rejected: missing or invalid safety marker");
    }
    Err(e) => println!("Error loading library: {:?}", e),
}
```

## Verification Process

When you attempt to load a library, the `DynamicLibraryLoader` performs two checks:

1. **Custom Marker Check**: Verifies the library exports `rust_safe_library_marker()` and it returns `0x53414645`
2. **Rust Library Check**: Verifies the library contains Rust-specific symbols (like `rust_begin_unwind` or `rust_panic`)

**Both checks must pass** for the library to be loaded. This ensures:
- The library is a Rust library (not C)
- The library was explicitly designed for this system

### Why Rust-Specific Symbol Detection?

Rust `cdylib` libraries always export certain symbols that are part of Rust's standard library:
- `rust_begin_unwind`: Rust's panic unwinding entry point
- `rust_panic`: Rust's panic handler

These symbols are automatically linked into all Rust `cdylib` libraries and cannot be present in a pure C library. This makes it extremely difficult for a C library to pass verification, even if it exports our custom marker function.

## Common Errors

### LibraryError::UnsafeLibrary

This error means one of the following:
- The library does not export `rust_safe_library_marker`, OR
- The marker function returns a value other than `0x53414645`, OR
- The library does not contain Rust-specific symbols (likely a C library)

**Solution**: 
1. Add the required marker function to your library exactly as shown above
2. Ensure your library is built as a Rust `cdylib` (not a C library)
3. Verify your `Cargo.toml` has `crate-type = ["cdylib"]`

### LibraryError::LoadError

This error means:
- The library file doesn't exist at the specified path
- The library file is corrupted or invalid
- Platform-specific loading failed

**Solution**: Verify the library path and that the library was built correctly.

## Important Notes

### Verification Does Not Guarantee Safety

**Important**: Passing verification does **not** guarantee that the library contains no `unsafe` code. It only indicates that:

1. The library is a Rust library (not C)
2. The library author intended it to be used with this loader
3. The library follows the required interface contract
4. The library was designed with this system in mind

Library authors are still responsible for ensuring their code is safe. The verification is a **gatekeeping mechanism**, not a comprehensive safety guarantee.

### Why C Libraries Cannot Pass

C libraries cannot pass verification because:
1. They cannot export our custom marker function (unless they include Rust code, making them Rust libraries)
2. They do not contain Rust-specific symbols like `rust_begin_unwind` or `rust_panic`
3. Even if a C library exports a function with the same name as our marker, it will fail the Rust-specific symbol check

This ensures that only Rust libraries can be loaded, providing an additional layer of security and type safety.

### Unsafe Code in Libraries

Libraries can still contain `unsafe` code blocks. The marker function requirement ensures that:

- Only libraries designed for this system can be loaded
- There is a clear contract between loader and library
- Arbitrary libraries cannot be accidentally loaded

But it does not prevent the use of `unsafe` within the library itself. Library authors must follow Rust's safety rules when using `unsafe`.

## Design Rationale

We chose the marker function approach over alternatives because:

- **Simple**: Easy to implement and verify
- **Enforceable**: Can be checked at runtime
- **Explicit**: Requires library authors to opt-in
- **Verifiable**: The return value can be checked for correctness

Alternatives considered:
- **Allowlists**: Require manual maintenance
- **Cryptographic Signatures**: Complex to implement
- **Metadata Files**: Can be separated or tampered with
- **No Verification**: Would allow arbitrary libraries

## Testing Your Library

To verify your library has the correct marker:

```rust
use libloading::Library;

let lib = unsafe { Library::new("path/to/libmy_safe_library.so").unwrap() };
let marker: libloading::Symbol<unsafe extern "C" fn() -> u32> = 
    unsafe { lib.get(b"rust_safe_library_marker").unwrap() };
let value = unsafe { marker() };
assert_eq!(value, 0x53414645);
```

## Questions?

If you have questions about creating loadable libraries, refer to:
- The module documentation in `src/dynamic_library.rs`
- The `LibraryError::UnsafeLibrary` documentation
- The `verify_safe_library()` function documentation

