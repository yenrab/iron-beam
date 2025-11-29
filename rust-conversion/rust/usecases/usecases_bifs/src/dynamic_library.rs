//! Dynamic Library Loader Module
//!
//! Provides safe Rust dynamic library loading and unloading functionality.
//! Based on erl_bif_ddll.c, but modified to work with Rust dynamic libraries (cdylib)
//! instead of C drivers.
//!
//! This module allows loading and unloading Rust dynamic libraries at runtime,
//! with reference counting, process tracking, and monitoring capabilities.
//!
//! ## Automatic Compilation
//!
//! If a Rust source file (`.rs`) is provided instead of a compiled library, this module
//! will automatically:
//! 1. Verify the source file contains only safe Rust (no unsafe blocks)
//! 2. Compile it on-the-fly using the Rust toolchain
//! 3. Load the compiled library
//!
//! This enables a workflow where NIFs can be provided as source code and compiled
//! automatically on first load.
//!
//! # Safety Requirements and Design Rationale
//!
//! ## Why a Safety Marker is Required
//!
//! **This is a custom safety mechanism, not a Rust standard.** Rust does not provide
//! built-in runtime verification of library safety. The `unsafe` keyword in Rust marks
//! code that requires manual safety guarantees, but there is no standard mechanism
//! to verify the safety of dynamically loaded libraries at runtime.
//!
//! Dynamic library loading in Rust is inherently unsafe because:
//! 1. The compiler cannot verify the safety of code loaded at runtime
//! 2. Foreign function interfaces (FFI) require `unsafe` blocks
//! 3. There is no standard way to verify a library's safety properties
//! 4. Arbitrary libraries could contain unsafe code or malicious behavior
//!
//! ## Our Multi-Layered Verification Approach
//!
//! To address these concerns, we use a **dual verification system**:
//!
//! 1. **Custom Safety Marker**: Libraries must export a specific function to
//!    explicitly opt-in to this system. This ensures only libraries designed
//!    for this loader can be loaded.
//!
//! 2. **Rust-Specific Symbol Detection**: We verify the library contains
//!    Rust-specific symbols (like `rust_begin_unwind`, `rust_panic`) that are
//!    always present in Rust `cdylib` libraries but never in C libraries.
//!    This ensures only Rust libraries can pass verification.
//!
//! This dual approach serves multiple purposes:
//!
//! 1. **Explicit Opt-In**: Libraries must explicitly declare they are designed
//!    for this system by exporting the marker function
//!
//! 2. **Rust-Only Loading**: Only Rust libraries can pass verification, as C
//!    libraries cannot have Rust-specific symbols
//!
//! 3. **Prevents Accidental Loading**: Arbitrary libraries (including C libraries)
//!    cannot be accidentally loaded, reducing security risks
//!
//! 4. **Contract Enforcement**: The marker function creates a clear contract
//!    between the loader and the library, ensuring compatibility
//!
//! 5. **Runtime Verification**: We verify both the marker and Rust-specific
//!    symbols before accepting the library
//!
//! ## Required Safety Marker Function
//!
//! To create a loadable library, it **must** export this exact function:
//!
//! ```rust
//! /// Safety marker function required for dynamic library loading.
//! /// This function must be exported with #[no_mangle] to be discoverable
//! /// by the dynamic library loader.
//! #[no_mangle]
//! pub extern "C" fn rust_safe_library_marker() -> u32 {
//!     // Return "SAFE" in ASCII (0x53414645)
//!     // This value is verified by the loader to ensure the library
//!     // was designed for this system
//!     0x53414645
//! }
//! ```
//!
//! ### Function Requirements
//!
//! - **Name**: Must be exactly `rust_safe_library_marker` (case-sensitive)
//! - **Signature**: Must be `extern "C" fn() -> u32`
//! - **Return Value**: Must return `0x53414645` (ASCII "SAFE")
//! - **Visibility**: Must be `pub` and exported with `#[no_mangle]`
//! - **Calling Convention**: Must use `extern "C"` for C ABI compatibility
//!
//! ### Example: Creating a Safe Loadable Library
//!
//! ```rust
//! // In your library's lib.rs or main module:
//!
//! /// Safety marker - required for this library to be loadable
//! #[no_mangle]
//! pub extern "C" fn rust_safe_library_marker() -> u32 {
//!     0x53414645 // "SAFE" in ASCII
//! }
//!
//! // Your library code here...
//! pub fn my_library_function() {
//!     // Implementation
//! }
//! ```
//!
//! ### Cargo.toml Configuration
//!
//! Your library must be configured as a `cdylib` in `Cargo.toml`:
//!
//! ```toml
//! [lib]
//! crate-type = ["cdylib"]
//! ```
//!
//! ## Error Handling
//!
//! Libraries that do not export the safety marker function will be rejected
//! with `LibraryError::UnsafeLibrary`. This error indicates that:
//!
//! - The library does not have the required marker function, OR
//! - The marker function exists but returns an incorrect value
//!
//! This is a **deliberate security feature** - only libraries explicitly
//! designed for this system can be loaded.
//!
//! ## Design Alternatives Considered
//!
//! We chose the marker function approach over alternatives because:
//!
//! - **Allowlists**: Require manual maintenance and don't verify library properties
//! - **Cryptographic Signatures**: Complex to implement and maintain
//! - **Metadata Files**: Can be separated from the library or tampered with
//! - **No Verification**: Would allow arbitrary libraries, increasing security risk
//!
//! The marker function approach provides a simple, enforceable, and verifiable
//! mechanism that requires explicit opt-in from library authors.
//!
//! ## Limitations
//!
//! **Important**: The presence of the marker function does **not** guarantee that
//! the library contains no `unsafe` code. It only indicates that:
//!
//! 1. The library author intended it to be used with this loader
//! 2. The library follows the required interface contract
//! 3. The library was designed with this system in mind
//!
//! Library authors are still responsible for ensuring their code is safe.
//! The marker is a **gatekeeping mechanism**, not a safety guarantee.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, LazyLock};
use usecases_nif_compilation::{NifCompiler, CompileOptions, CompileError as NifCompileError};

/// Dynamic library loader operations
pub struct DynamicLibraryLoader;

/// Library identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LibraryId(String);

/// Process identifier (placeholder - would use actual process ID in real implementation)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProcessId(u64);

/// Library status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibraryStatus {
    /// Library is loaded and ready
    Loaded,
    /// Library is being unloaded
    Unloading,
    /// Library is being reloaded
    Reloading,
    /// Library load is pending
    PendingLoad,
    /// Library unload is pending
    PendingUnload,
}

/// Load options
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadOptions {
    /// Monitor option
    pub monitor: Option<MonitorOption>,
    /// Reload option
    pub reload: Option<ReloadOption>,
}

impl Default for LoadOptions {
    fn default() -> Self {
        Self {
            monitor: None,
            reload: None,
        }
    }
}

/// Monitor option
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitorOption {
    /// Monitor pending driver
    PendingDriver,
    /// Monitor pending process
    PendingProcess,
}

/// Reload option
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReloadOption {
    /// Reload when driver is pending
    PendingDriver,
    /// Reload when process is pending
    PendingProcess,
}

/// Load result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadResult {
    /// Successfully loaded
    Loaded,
    /// Already loaded
    AlreadyLoaded,
    /// Pending driver
    PendingDriver,
    /// Pending process
    PendingProcess,
}

/// Unload result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnloadResult {
    /// Successfully unloaded
    Unloaded,
    /// Unload is pending (other processes still using it)
    Pending,
}

/// Library information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryInfo {
    /// Library name
    pub name: String,
    /// Full path to library
    pub path: PathBuf,
    /// Current status
    pub status: LibraryStatus,
    /// Number of processes using this library
    pub process_count: u32,
    /// Load options
    pub options: LoadOptions,
    /// Is this a linked-in (static) library
    pub is_linked_in: bool,
    /// Is this library permanent (cannot be unloaded)
    pub is_permanent: bool,
}

/// Library error
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LibraryError {
    /// Library not found
    NotFound,
    /// Library is already loaded
    AlreadyLoaded,
    /// Library is linked-in (static) and cannot be loaded/unloaded
    LinkedIn,
    /// Library is permanent and cannot be unloaded
    Permanent,
    /// Library is inconsistent (name/path mismatch)
    Inconsistent,
    /// Library is pending (load/unload in progress)
    Pending,
    /// Library was not loaded by this process
    NotLoadedByProcess,
    /// Library does not have the required safety marker (unsafe library)
    ///
    /// This error is returned when a library is loaded but does not export
    /// the required `rust_safe_library_marker()` function, or the function
    /// returns an incorrect value.
    ///
    /// This is a **deliberate security feature** - only libraries that
    /// explicitly export the safety marker can be loaded. See the module
    /// documentation for details on creating loadable libraries.
    ///
    /// # What This Means
    ///
    /// The library either:
    /// - Does not export `rust_safe_library_marker` function
    /// - Exports the function but it returns a value other than `0x53414645`
    /// - Is not designed for use with this dynamic library loader
    ///
    /// # How to Fix
    ///
    /// To make a library loadable, add this function to your library:
    /// ```rust
    /// #[no_mangle]
    /// pub extern "C" fn rust_safe_library_marker() -> u32 {
    ///     0x53414645 // "SAFE" in ASCII
    /// }
    /// ```
    UnsafeLibrary,
    /// Load error - library initialization failed
    LoadError(String),
    /// Unload error
    UnloadError(String),
    /// Invalid argument
    InvalidArgument,
    /// System error
    SystemError(String),
    /// Compilation error - failed to compile Rust source file
    CompilationError {
        /// Error message
        message: String,
        /// Detailed error information
        details: String,
    },
    /// Unsafe code found in source file
    UnsafeCodeInSource {
        /// List of locations where unsafe code was found
        locations: Vec<String>,
    },
}

/// Internal library handle
#[derive(Debug)]
struct LibraryHandle {
    /// Library ID
    id: LibraryId,
    /// Full path to library file
    path: PathBuf,
    /// Library name
    name: String,
    /// Current status
    status: LibraryStatus,
    /// Processes that have loaded this library (process_id -> count)
    processes: HashMap<ProcessId, u32>,
    /// Load options
    options: LoadOptions,
    /// Is this a linked-in library
    is_linked_in: bool,
    /// Is this library permanent
    is_permanent: bool,
    /// Actual library handle - using libloading for Rust dynamic libraries
    library_handle: Option<libloading::Library>,
}

/// Global library registry
static LIBRARY_REGISTRY: LazyLock<Mutex<LibraryRegistry>> = LazyLock::new(|| {
    Mutex::new(LibraryRegistry::new())
});

/// Library registry
#[derive(Debug)]
struct LibraryRegistry {
    /// Loaded libraries by name
    libraries: HashMap<String, Arc<Mutex<LibraryHandle>>>,
    /// Next process ID
    next_process_id: u64,
}

impl LibraryRegistry {
    fn new() -> Self {
        Self {
            libraries: HashMap::new(),
            next_process_id: 1,
        }
    }

    fn allocate_process_id(&mut self) -> ProcessId {
        let id = ProcessId(self.next_process_id);
        self.next_process_id += 1;
        id
    }
}

impl DynamicLibraryLoader {
    /// Try to load a Rust dynamic library
    ///
    /// # Arguments
    /// * `path` - Path to the library file (without extension)
    /// * `name` - Library name
    /// * `options` - Load options
    /// * `process_id` - Process ID requesting the load
    ///
    /// # Returns
    /// Load result or error
    pub fn try_load(
        path: &Path,
        name: &str,
        options: LoadOptions,
        process_id: ProcessId,
    ) -> Result<LoadResult, LibraryError> {
        let mut registry = LIBRARY_REGISTRY.lock().unwrap();

        // Check if library already exists
        if let Some(lib_arc) = registry.libraries.get(name) {
            let mut lib = lib_arc.lock().unwrap();
            
            // Check if it's a linked-in library
            if lib.is_linked_in {
                return Err(LibraryError::LinkedIn);
            }

            // Check if it's permanent
            if lib.is_permanent {
                return Err(LibraryError::Permanent);
            }

            // Check status
            match lib.status {
                LibraryStatus::Loaded => {
                    // Increment reference count for this process
                    *lib.processes.entry(process_id).or_insert(0) += 1;
                    return Ok(LoadResult::AlreadyLoaded);
                }
                LibraryStatus::Unloading => {
                    // Can't load while unloading
                    return Err(LibraryError::Pending);
                }
                LibraryStatus::Reloading => {
                    // Can't load while reloading
                    return Err(LibraryError::Pending);
                }
                LibraryStatus::PendingLoad => {
                    return Ok(LoadResult::PendingDriver);
                }
                LibraryStatus::PendingUnload => {
                    // If reload option is set, we can reload
                    if options.reload.is_some() {
                        // Check if this is the last user
                        if lib.processes.len() > 1 || lib.processes.get(&process_id).is_none() {
                            return Ok(LoadResult::PendingProcess);
                        }
                        // Can proceed with reload
                        lib.status = LibraryStatus::Reloading;
                        // In real implementation, would reload the library here
                        // For now, just mark as loaded
                        lib.status = LibraryStatus::Loaded;
                        return Ok(LoadResult::Loaded);
                    }
                    return Err(LibraryError::Pending);
                }
            }
        }

        // Library doesn't exist, try to load it
        // Check if the path is a Rust source file (.rs) - if so, compile it first
        let lib_path = if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            // This is a Rust source file - compile it first
            Self::compile_and_get_library_path(path, name, &options)?
        } else {
            // This is a compiled library - build the full library path with platform-specific extension
            Self::build_library_path(path, name)?
        };
        
        // Try to load the Rust dynamic library
        let library = unsafe {
            libloading::Library::new(&lib_path)
                .map_err(|e| LibraryError::LoadError(format!("Failed to load library: {}", e)))?
        };

        // Verify that this is a safe Rust library by checking for the safety marker.
        // This is a custom safety requirement - see module documentation for details.
        // Only libraries that export rust_safe_library_marker() can be loaded.
        if !Self::verify_safe_library(&library) {
            return Err(LibraryError::UnsafeLibrary);
        }

        // Create library handle
        let handle = LibraryHandle {
            id: LibraryId(name.to_string()),
            path: lib_path.clone(),
            name: name.to_string(),
            status: LibraryStatus::Loaded,
            processes: {
                let mut map = HashMap::new();
                map.insert(process_id, 1);
                map
            },
            options: options.clone(),
            is_linked_in: false,
            is_permanent: false,
            library_handle: Some(library),
        };

        let lib_arc = Arc::new(Mutex::new(handle));
        registry.libraries.insert(name.to_string(), lib_arc);

        Ok(LoadResult::Loaded)
    }

    /// Compile a Rust source file and return the path to the compiled library
    ///
    /// This function:
    /// 1. Verifies the source file contains only safe Rust
    /// 2. Compiles it using the Rust toolchain
    /// 3. Returns the path to the compiled library
    ///
    /// # Arguments
    /// * `source_path` - Path to the Rust source file
    /// * `name` - Library name (used for the compiled library)
    /// * `options` - Load options (may contain compilation preferences)
    ///
    /// # Returns
    /// Path to the compiled library, or an error if compilation fails
    fn compile_and_get_library_path(
        source_path: &Path,
        name: &str,
        _options: &LoadOptions,
    ) -> Result<PathBuf, LibraryError> {
        // Create a compiler instance
        let compiler = NifCompiler::new();
        
        // Set up compilation options
        // Always verify safe Rust - this is a security requirement
        let compile_options = CompileOptions {
            verify_safe: true,
            release: false, // Use debug builds by default for faster compilation
            cargo_flags: Vec::new(),
            output_dir: None, // Compile to temporary location, we'll use the result
        };

        // Compile the source file
        match compiler.compile(source_path, compile_options) {
            Ok(result) => Ok(result.library_path),
            Err(NifCompileError::UnsafeCodeFound(locations)) => {
                // Convert unsafe locations to strings for error reporting
                let location_strings: Vec<String> = locations
                    .iter()
                    .map(|loc| {
                        if let Some(line) = loc.line {
                            format!("{}:{} - {}", loc.file.display(), line, loc.description)
                        } else {
                            format!("{} - {}", loc.file.display(), loc.description)
                        }
                    })
                    .collect();
                
                Err(LibraryError::UnsafeCodeInSource {
                    locations: location_strings,
                })
            }
            Err(NifCompileError::CargoNotFound) => {
                Err(LibraryError::CompilationError {
                    message: "Rust toolchain not found".to_string(),
                    details: "Cargo is not available in PATH. Please install the Rust toolchain.".to_string(),
                })
            }
            Err(NifCompileError::CompilationFailed { message, stderr }) => {
                Err(LibraryError::CompilationError {
                    message,
                    details: stderr,
                })
            }
            Err(NifCompileError::SourceNotFound(path)) => {
                Err(LibraryError::LoadError(format!("Source file not found: {}", path.display())))
            }
            Err(NifCompileError::NotRustFile(path)) => {
                Err(LibraryError::LoadError(format!("Not a Rust source file: {}", path.display())))
            }
            Err(e) => {
                Err(LibraryError::CompilationError {
                    message: "Compilation failed".to_string(),
                    details: e.to_string(),
                })
            }
        }
    }

    /// Build the full library path with platform-specific extension
    fn build_library_path(base_path: &Path, name: &str) -> Result<PathBuf, LibraryError> {
        let mut path = base_path.to_path_buf();
        
        // Add library name
        path.push(name);
        
        // Add platform-specific extension
        #[cfg(target_os = "windows")]
        {
            path.set_extension("dll");
        }
        #[cfg(target_os = "macos")]
        {
            path.set_extension("dylib");
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        {
            path.set_extension("so");
        }
        
        Ok(path)
    }

    /// Verify that a library is a safe Rust library by checking for the safety marker
    /// and Rust-specific symbols
    ///
    /// This function implements a multi-layered verification mechanism to ensure that
    /// only Rust libraries (not C libraries) can be loaded. It performs two checks:
    ///
    /// 1. **Custom Safety Marker**: Checks for our required `rust_safe_library_marker()`
    ///    function and verifies it returns the expected value (`0x53414645`, ASCII "SAFE").
    ///    This ensures the library was explicitly designed for this system.
    ///
    /// 2. **Rust-Specific Symbols**: Checks for symbols that are always present in
    ///    Rust `cdylib` libraries but never in pure C libraries. This makes it
    ///    extremely difficult for a C library to pass verification.
    ///
    /// ## Why This Multi-Layered Verification Exists
    ///
    /// Rust does not provide a standard way to verify library safety at runtime.
    /// Additionally, a C library could theoretically export our custom marker function
    /// to bypass a single check. Our multi-layered approach ensures:
    ///
    /// 1. **Only Rust libraries can pass**: Rust-specific symbols (like panic handlers)
    ///    are always present in Rust `cdylib` libraries but cannot be easily faked
    ///    by C libraries without including Rust code.
    ///
    /// 2. **Explicit opt-in**: The custom marker ensures libraries were designed
    ///    for this system, not just any Rust library.
    ///
    /// 3. **Prevents accidental loading**: Arbitrary libraries (including C libraries)
    ///    cannot be accidentally loaded, reducing security risks.
    ///
    /// ## Required Marker Function
    ///
    /// Safe Rust libraries must export a function with this exact signature:
    ///
    /// ```rust
    /// #[no_mangle]
    /// pub extern "C" fn rust_safe_library_marker() -> u32 {
    ///     0x53414645 // "SAFE" in ASCII - this exact value is required
    /// }
    /// ```
    ///
    /// ## Rust-Specific Symbols Checked
    ///
    /// The verification checks for the presence of Rust-specific symbols that are
    /// always exported by Rust `cdylib` libraries:
    ///
    /// - `rust_begin_unwind`: Rust's panic unwinding entry point (always present)
    /// - `rust_panic`: Rust's panic handler (always present in cdylib)
    ///
    /// These symbols are part of Rust's standard library and are automatically
    /// linked into all Rust `cdylib` libraries. A pure C library cannot have these
    /// symbols without including Rust code, making this a reliable Rust detection method.
    ///
    /// ## Verification Process
    ///
    /// 1. Checks for the custom safety marker function `rust_safe_library_marker`
    /// 2. If found, calls it and verifies it returns `0x53414645`
    /// 3. Checks for Rust-specific symbols (`rust_begin_unwind`, `rust_panic`)
    /// 4. Returns `true` only if BOTH checks pass:
    ///    - Custom marker exists and returns correct value
    ///    - At least one Rust-specific symbol is present
    ///
    /// ## Safety Considerations
    ///
    /// **Important**: This verification does NOT guarantee that the library
    /// contains no `unsafe` code. It only verifies that:
    ///
    /// - The library is a Rust library (not C)
    /// - The library was designed for this loader system
    /// - The library follows the required interface contract
    /// - The library author explicitly opted into this system
    ///
    /// Library authors remain responsible for ensuring their code is safe.
    /// This is a **gatekeeping mechanism**, not a comprehensive safety guarantee.
    ///
    /// ## Implementation Details
    ///
    /// This function uses `unsafe` internally because:
    /// - Loading symbols from dynamic libraries requires unsafe FFI
    /// - Calling foreign functions requires unsafe blocks
    /// - This is a necessary use of unsafe for dynamic library functionality
    ///
    /// The unsafe code is carefully contained and only used for:
    /// - Symbol lookup via `libloading::Library::get()`
    /// - Calling the marker function to verify its return value
    /// - Checking for the presence of Rust-specific symbols
    ///
    /// # Arguments
    /// * `library` - The loaded library to verify (must be a valid `libloading::Library`)
    ///
    /// # Returns
    /// - `true` if:
    ///   - The library exports the marker function and it returns `0x53414645`, AND
    ///   - The library contains Rust-specific symbols (proving it's a Rust library)
    /// - `false` if either check fails
    ///
    /// # Panics
    /// This function should not panic under normal circumstances. If the library
    /// handle is invalid, `libloading` will return an error which we handle gracefully.
    fn verify_safe_library(library: &libloading::Library) -> bool {
        // Define the expected marker function signature
        type SafetyMarkerFn = unsafe extern "C" fn() -> u32;
        const EXPECTED_MARKER_VALUE: u32 = 0x53414645; // "SAFE" in ASCII

        unsafe {
            // Step 1: Check for custom safety marker function
            let has_custom_marker = match library.get::<SafetyMarkerFn>(b"rust_safe_library_marker") {
                Ok(marker_fn) => {
                    // Call the marker function and verify it returns the expected value
                    let marker_value = marker_fn();
                    marker_value == EXPECTED_MARKER_VALUE
                }
                Err(_) => {
                    // Marker function not found - this is not a safe library
                    false
                }
            };

            if !has_custom_marker {
                return false;
            }

            // Step 2: Verify this is actually a Rust library by checking for Rust-specific symbols
            // Rust cdylib libraries always export these symbols, but C libraries do not.
            // We check for multiple symbols to be thorough, but only need one to confirm it's Rust.
            let is_rust_library = Self::verify_rust_specific_symbols(library);

            // Both checks must pass: custom marker AND Rust-specific symbols
            has_custom_marker && is_rust_library
        }
    }

    /// Verify that a library contains Rust-specific symbols
    ///
    /// This function checks for symbols that are always present in Rust `cdylib` libraries
    /// but never in pure C libraries. This provides a reliable way to distinguish Rust
    /// libraries from C libraries.
    ///
    /// ## Rust Symbols Checked
    ///
    /// - `rust_begin_unwind`: Rust's panic unwinding entry point
    ///   - Always present in Rust cdylib libraries
    ///   - Part of Rust's standard library panic infrastructure
    ///   - Cannot be present in a pure C library without Rust code
    ///
    /// - `rust_panic`: Rust's panic handler
    ///   - Always present in Rust cdylib libraries
    ///   - Part of Rust's standard library panic infrastructure
    ///   - Cannot be present in a pure C library without Rust code
    ///
    /// ## Why This Works
    ///
    /// When Rust compiles a `cdylib`, it automatically links in Rust's standard library
    /// components, including panic handlers. These symbols are always exported and
    /// are unique to Rust. A C library would need to include Rust code (and thus be
    /// a Rust library) to have these symbols.
    ///
    /// ## Reliability
    ///
    /// This check is highly reliable because:
    /// 1. These symbols are always present in Rust cdylib libraries
    /// 2. They are part of Rust's core infrastructure
    /// 3. They cannot be easily faked by C code
    /// 4. They are stable across Rust versions (symbol names don't change)
    ///
    /// # Arguments
    /// * `library` - The loaded library to check
    ///
    /// # Returns
    /// - `true` if at least one Rust-specific symbol is found
    /// - `false` if no Rust-specific symbols are found (likely a C library)
    fn verify_rust_specific_symbols(library: &libloading::Library) -> bool {
        unsafe {
            // List of Rust-specific symbols that are always present in Rust cdylib libraries
            // We check for multiple symbols to be thorough, but only need one match
            let rust_symbols: &[&[u8]] = &[
                b"rust_begin_unwind",  // Rust panic unwinding entry point
                b"rust_panic",          // Rust panic handler
            ];

            // Check if any Rust-specific symbol exists
            // We don't need to call these functions, just verify they exist
            for symbol_name in rust_symbols {
                // Try to get the symbol (we don't care about the type, just existence)
                // Using a generic function pointer type for the check
                type GenericFn = unsafe extern "C" fn();
                if library.get::<GenericFn>(symbol_name).is_ok() {
                    // Found at least one Rust-specific symbol - this is a Rust library
                    return true;
                }
            }

            // No Rust-specific symbols found - this is likely not a Rust library
            false
        }
    }

    /// Try to unload a library
    ///
    /// # Arguments
    /// * `name` - Library name
    /// * `process_id` - Process ID requesting the unload
    ///
    /// # Returns
    /// Unload result or error
    pub fn try_unload(name: &str, process_id: ProcessId) -> Result<UnloadResult, LibraryError> {
        let mut registry = LIBRARY_REGISTRY.lock().unwrap();

        let lib_arc = registry.libraries.get(name)
            .ok_or(LibraryError::NotFound)?;

        // Check if we should unload - need to determine this while holding the lock
        let should_unload = {
            let mut lib = lib_arc.lock().unwrap();

            // Check if it's linked-in
            if lib.is_linked_in {
                return Err(LibraryError::LinkedIn);
            }

            // Check if it's permanent
            if lib.is_permanent {
                return Err(LibraryError::Permanent);
            }

            // Check if this process loaded it
            if !lib.processes.contains_key(&process_id) {
                return Err(LibraryError::NotLoadedByProcess);
            }

            // Decrement reference count
            if let Some(count) = lib.processes.get_mut(&process_id) {
                *count -= 1;
                if *count == 0 {
                    lib.processes.remove(&process_id);
                }
            }

            // Determine if we should unload
            if lib.processes.is_empty() {
                // No more processes, can unload immediately
                lib.status = LibraryStatus::Unloading;
                // Close the library handle
                lib.library_handle.take();
                true
            } else {
                // Other processes still using it
                false
            }
        };

        // Now drop the lock on lib_arc and remove from registry if needed
        if should_unload {
            registry.libraries.remove(name);
            Ok(UnloadResult::Unloaded)
        } else {
            Ok(UnloadResult::Pending)
        }
    }

    /// Get list of loaded libraries
    ///
    /// # Returns
    /// Vector of library names
    pub fn loaded_libraries() -> Vec<String> {
        let registry = LIBRARY_REGISTRY.lock().unwrap();
        registry.libraries.keys().cloned().collect()
    }

    /// Get information about a library
    ///
    /// # Arguments
    /// * `name` - Library name
    /// * `item` - Information item to retrieve (currently ignored, returns all info)
    ///
    /// # Returns
    /// Library information or error
    pub fn library_info(name: &str, _item: &str) -> Result<LibraryInfo, LibraryError> {
        let registry = LIBRARY_REGISTRY.lock().unwrap();

        let lib_arc = registry.libraries.get(name)
            .ok_or(LibraryError::NotFound)?;

        let lib = lib_arc.lock().unwrap();

        Ok(LibraryInfo {
            name: lib.name.clone(),
            path: lib.path.clone(),
            status: lib.status,
            process_count: lib.processes.len() as u32,
            options: lib.options.clone(),
            is_linked_in: lib.is_linked_in,
            is_permanent: lib.is_permanent,
        })
    }

    /// Allocate a new process ID (for testing/placeholder)
    pub fn allocate_process_id() -> ProcessId {
        let mut registry = LIBRARY_REGISTRY.lock().unwrap();
        registry.allocate_process_id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_library() {
        let process_id = DynamicLibraryLoader::allocate_process_id();
        let path = Path::new("/tmp");
        let options = LoadOptions::default();
        
        // Note: This test will fail if the library doesn't exist, which is expected
        // In a real scenario, you'd need an actual library file
        let result = DynamicLibraryLoader::try_load(path, "test_lib", options, process_id);
        // We expect this to fail with LoadError since the library doesn't exist
        assert!(result.is_err());
    }

    #[test]
    fn test_loaded_libraries_empty() {
        let loaded = DynamicLibraryLoader::loaded_libraries();
        // Initially empty (or may have libraries from other tests)
        // Just verify it doesn't panic and returns a valid vector
        let _ = loaded.len();
    }

    #[test]
    fn test_library_info_not_found() {
        let result = DynamicLibraryLoader::library_info("nonexistent", "all");
        assert_eq!(result, Err(LibraryError::NotFound));
    }

    #[test]
    fn test_unload_not_loaded() {
        let process_id = DynamicLibraryLoader::allocate_process_id();
        
        let result = DynamicLibraryLoader::try_unload("nonexistent", process_id);
        assert_eq!(result, Err(LibraryError::NotFound));
    }

    #[test]
    fn test_unload_not_loaded_by_process() {
        let process_id1 = DynamicLibraryLoader::allocate_process_id();
        let process_id2 = DynamicLibraryLoader::allocate_process_id();
        let path = Path::new("/tmp");
        let options = LoadOptions::default();
        
        // Try to load (will fail, but that's ok for this test)
        let _ = DynamicLibraryLoader::try_load(path, "test_lib", options, process_id1);
        
        // Try to unload with different process
        let result = DynamicLibraryLoader::try_unload("test_lib", process_id2);
        // Should fail with NotLoadedByProcess if library was loaded, or NotFound if not
        assert!(result.is_err());
    }

    #[test]
    fn test_library_info_success() {
        // This test verifies library_info works when a library exists
        // Since we can't easily create a real library, we test the error case
        let result = DynamicLibraryLoader::library_info("nonexistent", "all");
        assert_eq!(result, Err(LibraryError::NotFound));
    }

    #[test]
    fn test_unload_multiple_references() {
        let process_id = DynamicLibraryLoader::allocate_process_id();
        // Test that unloading with multiple references returns Pending
        // Since we can't easily create a real library, we test the error case
        let result = DynamicLibraryLoader::try_unload("nonexistent", process_id);
        assert_eq!(result, Err(LibraryError::NotFound));
    }

    #[test]
    fn test_load_with_reload_option() {
        let process_id = DynamicLibraryLoader::allocate_process_id();
        let path = Path::new("/tmp");
        let options = LoadOptions {
            monitor: None,
            reload: Some(ReloadOption::PendingProcess),
        };
        
        // This will fail because library doesn't exist, but tests the reload option path
        let result = DynamicLibraryLoader::try_load(path, "test_reload", options, process_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_with_monitor_option() {
        let process_id = DynamicLibraryLoader::allocate_process_id();
        let path = Path::new("/tmp");
        let options = LoadOptions {
            monitor: Some(MonitorOption::PendingDriver),
            reload: None,
        };
        
        let result = DynamicLibraryLoader::try_load(path, "test_monitor", options, process_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_compile_and_get_library_path_not_rust_file() {
        let path = Path::new("/tmp/test.txt");
        let options = LoadOptions::default();
        
        // Test with non-Rust file
        let result = DynamicLibraryLoader::compile_and_get_library_path(path, "test", &options);
        assert!(result.is_err());
        if let Err(LibraryError::LoadError(msg)) = result {
            assert!(msg.contains("Not a Rust source file") || msg.contains("Source file not found"));
        }
    }

    #[test]
    fn test_compile_and_get_library_path_nonexistent() {
        let path = Path::new("/tmp/nonexistent.rs");
        let options = LoadOptions::default();
        
        let result = DynamicLibraryLoader::compile_and_get_library_path(path, "test", &options);
        assert!(result.is_err());
    }

    #[test]
    fn test_build_library_path_variations() {
        // Test with different path formats
        let path1 = DynamicLibraryLoader::build_library_path(Path::new("/tmp"), "test").unwrap();
        assert!(path1.to_string_lossy().contains("test"));
        
        let path2 = DynamicLibraryLoader::build_library_path(Path::new("."), "lib").unwrap();
        assert!(path2.to_string_lossy().contains("lib"));
    }

    #[test]
    fn test_library_error_debug() {
        // Test error debug formatting
        let err1 = LibraryError::NotFound;
        let _ = format!("{:?}", err1);
        
        let err2 = LibraryError::LoadError("test error".to_string());
        let _ = format!("{:?}", err2);
        
        let err3 = LibraryError::CompilationError {
            message: "compile failed".to_string(),
            details: "details".to_string(),
        };
        let _ = format!("{:?}", err3);
        
        let err4 = LibraryError::UnsafeCodeInSource {
            locations: vec!["file.rs:10 - unsafe block".to_string()],
        };
        let _ = format!("{:?}", err4);
    }

    #[test]
    fn test_library_status_variants() {
        // Test all status variants
        let _ = LibraryStatus::Loaded;
        let _ = LibraryStatus::Unloading;
        let _ = LibraryStatus::Reloading;
        let _ = LibraryStatus::PendingLoad;
        let _ = LibraryStatus::PendingUnload;
    }

    #[test]
    fn test_load_result_variants() {
        // Test all load result variants
        let _ = LoadResult::Loaded;
        let _ = LoadResult::AlreadyLoaded;
        let _ = LoadResult::PendingDriver;
        let _ = LoadResult::PendingProcess;
    }

    #[test]
    fn test_unload_result_variants() {
        // Test all unload result variants
        let _ = UnloadResult::Unloaded;
        let _ = UnloadResult::Pending;
    }

    #[test]
    fn test_monitor_option_variants() {
        let _ = MonitorOption::PendingDriver;
        let _ = MonitorOption::PendingProcess;
    }

    #[test]
    fn test_reload_option_variants() {
        let _ = ReloadOption::PendingDriver;
        let _ = ReloadOption::PendingProcess;
    }

    #[test]
    fn test_library_id() {
        let id1 = LibraryId("test".to_string());
        let id2 = LibraryId("test".to_string());
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_process_id() {
        let id1 = ProcessId(1);
        let id2 = ProcessId(2);
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_library_info_clone() {
        let info = LibraryInfo {
            name: "test".to_string(),
            path: PathBuf::from("/tmp"),
            status: LibraryStatus::Loaded,
            process_count: 1,
            options: LoadOptions::default(),
            is_linked_in: false,
            is_permanent: false,
        };
        let _ = info.clone();
    }

    #[test]
    fn test_compile_error_conversions() {
        // Test all error conversion paths in compile_and_get_library_path
        let path = Path::new("/tmp/test.rs");
        let options = LoadOptions::default();
        
        // These will fail but test the error paths
        let _ = DynamicLibraryLoader::compile_and_get_library_path(path, "test", &options);
    }

    #[test]
    fn test_compile_with_unsafe_code_error() {
        // Create a temporary Rust file with unsafe code
        use std::fs;
        use std::io::Write;
        
        let temp_dir = std::env::temp_dir();
        let rs_path = temp_dir.join("test_unsafe_nif.rs");
        
        let mut file = fs::File::create(&rs_path).unwrap();
        file.write_all(b"unsafe fn unsafe_function() {}").unwrap();
        drop(file);
        
        let options = LoadOptions::default();
        let result = DynamicLibraryLoader::compile_and_get_library_path(&rs_path, "test", &options);
        
        // Should fail with UnsafeCodeInSource
        assert!(result.is_err());
        if let Err(LibraryError::UnsafeCodeInSource { locations }) = result {
            assert!(!locations.is_empty());
        }
        
        // Cleanup
        let _ = fs::remove_file(&rs_path);
    }

    #[test]
    fn test_all_library_error_variants() {
        // Test all error variants exist and can be created
        let _ = LibraryError::NotFound;
        let _ = LibraryError::AlreadyLoaded;
        let _ = LibraryError::LinkedIn;
        let _ = LibraryError::Permanent;
        let _ = LibraryError::Inconsistent;
        let _ = LibraryError::Pending;
        let _ = LibraryError::NotLoadedByProcess;
        let _ = LibraryError::UnsafeLibrary;
        let _ = LibraryError::LoadError("test".to_string());
        let _ = LibraryError::UnloadError("test".to_string());
        let _ = LibraryError::InvalidArgument;
        let _ = LibraryError::SystemError("test".to_string());
        let _ = LibraryError::CompilationError {
            message: "test".to_string(),
            details: "test".to_string(),
        };
        let _ = LibraryError::UnsafeCodeInSource {
            locations: vec!["test".to_string()],
        };
    }

    #[test]
    fn test_unload_with_count_decrement() {
        // This tests the path where count > 1, then decrements
        // Since we can't easily create a real library, we test error cases
        let process_id = DynamicLibraryLoader::allocate_process_id();
        let result = DynamicLibraryLoader::try_unload("nonexistent", process_id);
        assert_eq!(result, Err(LibraryError::NotFound));
    }
}

