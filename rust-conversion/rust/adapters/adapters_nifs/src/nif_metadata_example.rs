//! Example: How Rust NIF authors would use the metadata system
//!
//! This file shows the pattern that Rust NIF libraries should follow
//! to work with the Rust-native metadata discovery system.
//!
//! ## Usage Pattern
//!
//! Rust NIF libraries should:
//! 1. Define a static `RustNifMetadata` structure
//! 2. Export `nif_get_metadata()` function that returns a pointer to it
//! 3. Ensure all NIF functions have `#[no_mangle]` and `extern "C"`
//!
//! ## Example
//!
//! ```rust
//! use adapters_nifs::{RustNifMetadata, FunctionMetadata};
//! use std::os::raw::{c_void, c_int};
//!
//! // Example NIF function
//! #[no_mangle]
//! pub extern "C" fn nif_my_function(
//!     _env: *mut c_void,
//!     _argc: c_int,
//!     _argv: *const u64,
//! ) -> u64 {
//!     // NIF implementation
//!     0
//! }
//!
//! // Define metadata
//! static NIF_METADATA: RustNifMetadata = RustNifMetadata {
//!     module_name: "my_module".to_string(),
//!     version: (2, 17),
//!     min_erts_version: Some("erts-14.0".to_string()),
//!     functions: vec![
//!         FunctionMetadata {
//!             name: "my_function".to_string(),
//!             arity: 2,
//!             symbol_name: "nif_my_function".to_string(),  // Symbol name in library
//!             flags: 0,  // 0 = normal, 1 = dirty CPU, 2 = dirty IO
//!         },
//!     ],
//! };
//!
//! // Export metadata accessor function
//! #[no_mangle]
//! pub extern "C" fn nif_get_metadata() -> *const RustNifMetadata {
//!     &NIF_METADATA
//! }
//!
//! // Alternative: If using nif_init for compatibility
//! #[no_mangle]
//! pub extern "C" fn nif_init() -> *const RustNifMetadata {
//!     &NIF_METADATA
//! }
//! ```

// This file is for documentation purposes only
// It shows the pattern that NIF authors should follow

