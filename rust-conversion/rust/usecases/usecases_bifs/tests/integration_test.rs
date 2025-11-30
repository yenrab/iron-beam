//! Integration tests for usecases_bifs
//!
//! Tests the integration between dynamic library loading and NIF compilation.
//! These tests verify the full pipeline: source file -> compilation -> verification -> loading.
//!
//! Also includes integration tests for OS BIFs.

use usecases_bifs::dynamic_library::*;
use usecases_bifs::os::{OsBif, OsError};
use usecases_bifs::counters::{CountersBif, CounterRef, CountersError};
use usecases_bifs::unique::{UniqueBif, Reference, UniqueIntegerOption};
use usecases_bifs::op::{OpBif, OpError, ErlangTerm};
use usecases_bifs::guard::{GuardBif, GuardError};
use usecases_bifs::lists::{ListsBif, ListsError};
use usecases_bifs::persistent::{PersistentBif, PersistentError};
use usecases_nif_compilation::{NifCompiler, CompileOptions};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Helper to create a temporary Rust source file with the safety marker
fn create_test_rust_library(dir: &Path, name: &str, content: &str) -> PathBuf {
    let rs_path = dir.join(format!("{}.rs", name));
    let mut file = fs::File::create(&rs_path).unwrap();
    
    // Always include the safety marker
    let full_content = format!(
        r#"
/// Safety marker - REQUIRED for this library to be loadable
#[no_mangle]
pub extern "C" fn rust_safe_library_marker() -> u32 {{
    0x53414645 // "SAFE" in ASCII
}}

{}
"#,
        content
    );
    
    file.write_all(full_content.as_bytes()).unwrap();
    rs_path
}

/// Helper to create a safe Rust library (no unsafe code)
fn create_safe_rust_library(dir: &Path, name: &str) -> PathBuf {
    create_test_rust_library(
        dir,
        name,
        r#"
/// A safe function
#[no_mangle]
pub extern "C" fn safe_function(x: i32) -> i32 {
    x * 2
}

/// Another safe function
#[no_mangle]
pub extern "C" fn add_numbers(a: i32, b: i32) -> i32 {
    a + b
}
"#,
    )
}

/// Helper to create an unsafe Rust library (contains unsafe code)
fn create_unsafe_rust_library(dir: &Path, name: &str) -> PathBuf {
    create_test_rust_library(
        dir,
        name,
        r#"
/// An unsafe function
#[no_mangle]
pub unsafe extern "C" fn unsafe_function() {
    // unsafe code
}
"#,
    )
}

#[test]
fn test_full_compilation_and_loading_pipeline() {
    // Test the full pipeline: create source -> compile -> load
    let temp_dir = tempfile::tempdir().unwrap();
    let source_path = create_safe_rust_library(temp_dir.path(), "test_lib");
    
    // Step 1: Compile the source file
    let compiler = NifCompiler::new();
    let compile_options = CompileOptions {
        verify_safe: true,
        release: false,
        cargo_flags: Vec::new(),
        output_dir: None,
    };
    
    let compile_result = compiler.compile(&source_path, compile_options);
    
    // Compilation might succeed or fail depending on cargo availability
    // The important thing is it doesn't fail on unsafe code
    match compile_result {
        Ok(compiled_lib) => {
            // Compilation succeeded
            assert!(!compiled_lib.was_cached);
            // Note: The library path might be in a temp directory that gets cleaned up
            // So we just verify the result structure is correct
            let _ = compiled_lib.library_path;
        }
        Err(usecases_nif_compilation::CompileError::UnsafeCodeFound(_)) => {
            panic!("Compilation should not fail on unsafe code for safe Rust");
        }
        Err(usecases_nif_compilation::CompileError::CargoNotFound) => {
            // Cargo not available - acceptable for test environment
        }
        Err(e) => {
            // Other compilation errors are acceptable
            println!("Compilation failed (acceptable): {:?}", e);
        }
    }
}

#[test]
fn test_compile_and_load_via_dynamic_loader() {
    // Test that dynamic library loader can handle Rust source files
    let temp_dir = tempfile::tempdir().unwrap();
    let source_path = create_safe_rust_library(temp_dir.path(), "test_nif");
    
    let process_id = DynamicLibraryLoader::allocate_process_id();
    let options = LoadOptions::default();
    
    // Try to load the Rust source file - it should compile automatically
    let result = DynamicLibraryLoader::try_load(&source_path, "test_nif", options, process_id);
    
    // This will likely fail because:
    // 1. The compiled library needs to be in a specific location
    // 2. The library needs proper NIF initialization
    // But we can verify the compilation path was attempted
    // The error should be related to loading, not compilation
    assert!(result.is_err());
    
    // Verify the error is not a compilation error (compilation should succeed)
    match result {
        Err(LibraryError::LoadError(_)) => {
            // Expected - library compiled but couldn't be loaded (missing NIF init)
        }
        Err(LibraryError::CompilationError { .. }) => {
            panic!("Compilation should have succeeded for safe Rust code");
        }
        Err(LibraryError::UnsafeCodeInSource { .. }) => {
            panic!("Code should be safe");
        }
        _ => {
            // Other errors are acceptable (library not found, etc.)
        }
    }
}

#[test]
fn test_unsafe_code_rejection_in_pipeline() {
    // Test that unsafe code is rejected during compilation
    let temp_dir = tempfile::tempdir().unwrap();
    let source_path = create_unsafe_rust_library(temp_dir.path(), "unsafe_nif");
    
    let process_id = DynamicLibraryLoader::allocate_process_id();
    let options = LoadOptions::default();
    
    // Try to load the unsafe Rust source file
    let result = DynamicLibraryLoader::try_load(&source_path, "unsafe_nif", options, process_id);
    
    // Should fail with UnsafeCodeInSource
    assert!(result.is_err());
    match result {
        Err(LibraryError::UnsafeCodeInSource { locations }) => {
            assert!(!locations.is_empty(), "Should report unsafe code locations");
            // Verify the locations mention unsafe code
            let all_locations = locations.join(" ");
            assert!(
                all_locations.contains("unsafe") || all_locations.contains("Unsafe"),
                "Error should mention unsafe code"
            );
        }
        _ => {
            panic!("Expected UnsafeCodeInSource error, got: {:?}", result);
        }
    }
}

#[test]
fn test_verification_before_compilation() {
    // Test that verification happens before compilation
    let temp_dir = tempfile::tempdir().unwrap();
    let source_path = create_unsafe_rust_library(temp_dir.path(), "unsafe_lib");
    
    let compiler = NifCompiler::new();
    let compile_options = CompileOptions {
        verify_safe: true, // Enable verification
        release: false,
        cargo_flags: Vec::new(),
        output_dir: None,
    };
    
    // Compilation should fail due to unsafe code
    let result = compiler.compile(&source_path, compile_options);
    assert!(result.is_err());
    
    match result {
        Err(usecases_nif_compilation::CompileError::UnsafeCodeFound(locations)) => {
            assert!(!locations.is_empty());
        }
        _ => {
            panic!("Expected UnsafeCodeFound error");
        }
    }
}

#[test]
fn test_compilation_without_verification() {
    // Test compilation when verification is disabled
    let temp_dir = tempfile::tempdir().unwrap();
    let source_path = create_unsafe_rust_library(temp_dir.path(), "unsafe_lib_no_verify");
    
    let compiler = NifCompiler::new();
    let compile_options = CompileOptions {
        verify_safe: false, // Disable verification
        release: false,
        cargo_flags: Vec::new(),
        output_dir: None,
    };
    
    // Compilation should proceed (but will fail because we can't actually compile without cargo setup)
    // or succeed if cargo is available
    let result = compiler.compile(&source_path, compile_options);
    // This might succeed (if cargo works) or fail for other reasons (cargo not found, etc.)
    // The important thing is it doesn't fail on unsafe code check
    if let Err(usecases_nif_compilation::CompileError::UnsafeCodeFound(_)) = result {
        panic!("Should not fail on unsafe code when verification is disabled");
    }
}

#[test]
fn test_safe_rust_verification_integration() {
    // Test that safe Rust verification works correctly
    use usecases_nif_compilation::SafeRustVerifier;
    
    let verifier = SafeRustVerifier::new();
    let temp_dir = tempfile::tempdir().unwrap();
    
    // Test 1: Safe code should pass
    let safe_path = create_safe_rust_library(temp_dir.path(), "safe_lib");
    let result = verifier.verify_file(&safe_path).unwrap();
    assert_eq!(result, usecases_nif_compilation::VerificationResult::Safe);
    
    // Test 2: Unsafe code should fail
    let unsafe_path = create_unsafe_rust_library(temp_dir.path(), "unsafe_lib");
    let result = verifier.verify_file(&unsafe_path).unwrap();
    match result {
        usecases_nif_compilation::VerificationResult::Unsafe { locations } => {
            assert!(!locations.is_empty());
        }
        _ => panic!("Expected unsafe code to be detected"),
    }
}

#[test]
fn test_compilation_options_integration() {
    // Test different compilation options
    let temp_dir = tempfile::tempdir().unwrap();
    let source_path = create_safe_rust_library(temp_dir.path(), "options_test");
    
    let compiler = NifCompiler::new();
    
    // Test with release mode
    let release_options = CompileOptions {
        verify_safe: true,
        release: true,
        cargo_flags: Vec::new(),
        output_dir: None,
    };
    
    // This might succeed or fail depending on cargo availability
    let _result = compiler.compile(&source_path, release_options);
    
    // Test with custom output directory
    let output_dir = temp_dir.path().join("output");
    let output_options = CompileOptions {
        verify_safe: true,
        release: false,
        cargo_flags: Vec::new(),
        output_dir: Some(output_dir.clone()),
    };
    
    let result = compiler.compile(&source_path, output_options);
    if result.is_ok() {
        let compiled = result.unwrap();
        // If output_dir was specified, library should be there
        if output_dir.exists() {
            // Library might be in output_dir
        }
    }
}

#[test]
fn test_error_propagation_through_layers() {
    // Test that errors propagate correctly through the layers
    let temp_dir = tempfile::tempdir().unwrap();
    
    // Test 1: File not found error
    let nonexistent = temp_dir.path().join("nonexistent.rs");
    let process_id = DynamicLibraryLoader::allocate_process_id();
    let result = DynamicLibraryLoader::try_load(&nonexistent, "test", LoadOptions::default(), process_id);
    assert!(result.is_err());
    
    // Test 2: Not a Rust file
    let txt_file = temp_dir.path().join("test.txt");
    fs::write(&txt_file, b"not rust code").unwrap();
    let result = DynamicLibraryLoader::try_load(&txt_file, "test", LoadOptions::default(), process_id);
    assert!(result.is_err());
    
    // Test 3: Unsafe code error
    let unsafe_path = create_unsafe_rust_library(temp_dir.path(), "unsafe_test");
    let result = DynamicLibraryLoader::try_load(&unsafe_path, "unsafe_test", LoadOptions::default(), process_id);
    assert!(result.is_err());
    match result {
        Err(LibraryError::UnsafeCodeInSource { .. }) => {
            // Expected
        }
        _ => {
            // Other errors are also acceptable (compilation might fail for other reasons)
        }
    }
}

#[test]
fn test_multiple_library_operations() {
    // Test multiple library operations in sequence
    let temp_dir = tempfile::tempdir().unwrap();
    let process_id = DynamicLibraryLoader::allocate_process_id();
    
    // Test library info on non-existent library
    let info_result = DynamicLibraryLoader::library_info("nonexistent", "all");
    assert_eq!(info_result, Err(LibraryError::NotFound));
    
    // Test loaded libraries list
    let loaded = DynamicLibraryLoader::loaded_libraries();
    // Should be empty or contain libraries from other tests
    let _ = loaded.len();
    
    // Test unload on non-existent library
    let unload_result = DynamicLibraryLoader::try_unload("nonexistent", process_id);
    assert_eq!(unload_result, Err(LibraryError::NotFound));
}

#[test]
fn test_compilation_with_various_rust_constructs() {
    // Test compilation with various Rust language constructs
    let temp_dir = tempfile::tempdir().unwrap();
    
    let complex_code = r#"
/// Safety marker
#[no_mangle]
pub extern "C" fn rust_safe_library_marker() -> u32 {
    0x53414645
}

/// Function with match
#[no_mangle]
pub extern "C" fn match_function(x: i32) -> i32 {
    match x {
        0 => 0,
        1 => 1,
        n => n * 2,
    }
}

/// Function with if/else
#[no_mangle]
pub extern "C" fn conditional_function(x: i32) -> i32 {
    if x > 0 {
        x
    } else {
        -x
    }
}

/// Function with loop
#[no_mangle]
pub extern "C" fn loop_function(n: i32) -> i32 {
    let mut sum = 0;
    for i in 0..n {
        sum += i;
    }
    sum
}
"#;
    
    let source_path = create_test_rust_library(temp_dir.path(), "complex_lib", complex_code);
    
    let compiler = NifCompiler::new();
    let options = CompileOptions::default();
    
    // Should compile successfully (safe Rust)
    let result = compiler.compile(&source_path, options);
    // Might succeed or fail depending on cargo, but shouldn't fail on unsafe code
    if let Err(usecases_nif_compilation::CompileError::UnsafeCodeFound(_)) = result {
        panic!("Complex but safe Rust code should compile");
    }
}

#[test]
fn test_verification_with_nested_constructs() {
    // Test verification with nested Rust constructs
    use usecases_nif_compilation::SafeRustVerifier;
    
    let verifier = SafeRustVerifier::new();
    
    // Test nested unsafe blocks
    let nested_unsafe = r#"
pub fn outer() {
    if true {
        unsafe {
            unsafe {
                // nested unsafe
            }
        }
    }
}
"#;
    
    let result = verifier.verify_content(nested_unsafe, Path::new("test.rs")).unwrap();
    match result {
        usecases_nif_compilation::VerificationResult::Unsafe { locations } => {
            assert!(locations.len() >= 2, "Should detect both unsafe blocks");
        }
        _ => panic!("Should detect nested unsafe blocks"),
    }
}

#[test]
fn test_compilation_error_handling() {
    // Test various compilation error scenarios
    let temp_dir = tempfile::tempdir().unwrap();
    let compiler = NifCompiler::new();
    
    // Test 1: Invalid Rust syntax
    let invalid_rust = temp_dir.path().join("invalid.rs");
    fs::write(&invalid_rust, b"pub fn broken {").unwrap();
    
    let options = CompileOptions {
        verify_safe: false, // Skip verification to test compilation errors
        ..Default::default()
    };
    
    let result = compiler.compile(&invalid_rust, options);
    // Should fail with compilation error
    assert!(result.is_err());
    match result {
        Err(usecases_nif_compilation::CompileError::CompilationFailed { .. }) => {
            // Expected
        }
        _ => {
            // Other errors are also acceptable
        }
    }
}

#[test]
fn test_library_path_building_integration() {
    // Test library path building indirectly through try_load
    // build_library_path is private, so we test it through the public API
    let process_id = DynamicLibraryLoader::allocate_process_id();
    let base_path = Path::new("/tmp");
    
    // Test that try_load handles path building correctly
    // This will fail because library doesn't exist, but tests the path building
    let result = DynamicLibraryLoader::try_load(base_path, "test_lib", LoadOptions::default(), process_id);
    assert!(result.is_err()); // Expected - library doesn't exist
    
    // Test with different path formats
    let relative_path = Path::new(".");
    let result = DynamicLibraryLoader::try_load(relative_path, "lib", LoadOptions::default(), process_id);
    assert!(result.is_err()); // Expected - library doesn't exist
}

#[test]
fn test_process_id_management() {
    // Test process ID allocation and uniqueness
    let id1 = DynamicLibraryLoader::allocate_process_id();
    let id2 = DynamicLibraryLoader::allocate_process_id();
    let id3 = DynamicLibraryLoader::allocate_process_id();
    
    // All should be unique
    assert_ne!(id1, id2);
    assert_ne!(id2, id3);
    assert_ne!(id1, id3);
}

#[test]
fn test_load_options_combinations() {
    // Test various load option combinations
    let process_id = DynamicLibraryLoader::allocate_process_id();
    let path = Path::new("/tmp");
    
    // Test with monitor option
    let options1 = LoadOptions {
        monitor: Some(MonitorOption::PendingDriver),
        reload: None,
    };
    let _ = DynamicLibraryLoader::try_load(path, "test1", options1, process_id);
    
    // Test with reload option
    let options2 = LoadOptions {
        monitor: None,
        reload: Some(ReloadOption::PendingProcess),
    };
    let _ = DynamicLibraryLoader::try_load(path, "test2", options2, process_id);
    
    // Test with both options
    let options3 = LoadOptions {
        monitor: Some(MonitorOption::PendingProcess),
        reload: Some(ReloadOption::PendingDriver),
    };
    let _ = DynamicLibraryLoader::try_load(path, "test3", options3, process_id);
}

// ============================================================================
// OS BIF Integration Tests
// ============================================================================

#[test]
fn test_os_bif_environment_variable_operations() {
    // Test full environment variable workflow
    let test_key = "TEST_OS_BIF_INTEGRATION";
    let test_value = "integration_test_value";
    
    // Initially should not exist
    let initial = OsBif::getenv(test_key);
    assert_eq!(initial, None);
    
    // Set the variable
    OsBif::putenv(test_key, test_value).unwrap();
    
    // Should now exist
    let retrieved = OsBif::getenv(test_key);
    assert_eq!(retrieved, Some(test_value.to_string()));
    
    // Update the value
    let new_value = "updated_value";
    OsBif::putenv(test_key, new_value).unwrap();
    let updated = OsBif::getenv(test_key);
    assert_eq!(updated, Some(new_value.to_string()));
    
    // Unset the variable
    OsBif::unsetenv(test_key).unwrap();
    
    // Should no longer exist
    let after_unset = OsBif::getenv(test_key);
    assert_eq!(after_unset, None);
}

#[test]
fn test_os_bif_env_completeness() {
    // Test that os::env() returns all environment variables
    let all_env = OsBif::env();
    
    // Should have at least some environment variables
    assert!(!all_env.is_empty());
    
    // Verify structure: all entries should be (key, value) tuples
    for (key, _value) in &all_env {
        assert!(!key.is_empty());
        // Value can be empty, but key cannot
    }
    
    // Verify that getenv works for variables returned by env()
    if let Some((test_key, _)) = all_env.first() {
        let retrieved = OsBif::getenv(test_key);
        // Should match (or at least be Some)
        assert!(retrieved.is_some());
    }
}

#[test]
fn test_os_bif_getpid_consistency() {
    // Test that getpid returns consistent results
    let pid1 = OsBif::getpid();
    let pid2 = OsBif::getpid();
    
    // Should be the same (same process)
    assert_eq!(pid1, pid2);
    
    // Should be non-empty
    assert!(!pid1.is_empty());
    
    // Should be valid digits
    for &digit in &pid1 {
        assert!(digit < 10);
    }
}

#[test]
fn test_os_bif_timestamp_ordering() {
    // Test that timestamps are monotonically increasing
    let (m1, s1, u1) = OsBif::timestamp();
    std::thread::sleep(std::time::Duration::from_millis(50));
    let (m2, s2, u2) = OsBif::timestamp();
    
    // Second timestamp should be >= first
    let time1_total = m1 * 1_000_000 + s1;
    let time2_total = m2 * 1_000_000 + s2;
    
    if time2_total == time1_total {
        // Same second, microseconds should be >=
        assert!(u2 >= u1);
    } else {
        // Different second, total should be >
        assert!(time2_total > time1_total);
    }
}

#[test]
fn test_os_bif_signal_handling_validation() {
    // Test signal handling validation
    let valid_signals = ["SIGINT", "SIGTERM", "SIGHUP", "SIGUSR1", "SIGUSR2"];
    let valid_actions = ["ignore", "default", "handle"];
    
    // All combinations should be valid
    for signal in &valid_signals {
        for action in &valid_actions {
            let result = OsBif::set_signal(signal, action);
            assert!(result.is_ok(), "Signal {} with action {} should be valid", signal, action);
        }
    }
    
    // Invalid actions should fail
    let invalid_actions = ["invalid", "bad", "wrong", ""];
    for action in &invalid_actions {
        let result = OsBif::set_signal("SIGINT", action);
        assert!(result.is_err(), "Action '{}' should be invalid", action);
        match result {
            Err(OsError::InvalidArgument(_)) => {
                // Expected
            }
            _ => panic!("Expected InvalidArgument error"),
        }
    }
    
    // Empty signal should fail
    let result = OsBif::set_signal("", "ignore");
    assert!(result.is_err());
    match result {
        Err(OsError::InvalidArgument(_)) => {
            // Expected
        }
        _ => panic!("Expected InvalidArgument error for empty signal"),
    }
}

#[test]
fn test_os_bif_multiple_environment_operations() {
    // Test multiple environment variable operations in sequence
    let keys = ["TEST_OS_1", "TEST_OS_2", "TEST_OS_3"];
    let values = ["value1", "value2", "value3"];
    
    // Set multiple variables
    for (key, value) in keys.iter().zip(values.iter()) {
        OsBif::putenv(key, value).unwrap();
    }
    
    // Verify all are set
    for (key, expected_value) in keys.iter().zip(values.iter()) {
        let retrieved = OsBif::getenv(key);
        assert_eq!(retrieved, Some(expected_value.to_string()));
    }
    
    // Unset all
    for key in &keys {
        OsBif::unsetenv(key).unwrap();
    }
    
    // Verify all are unset
    for key in &keys {
        let retrieved = OsBif::getenv(key);
        assert_eq!(retrieved, None);
    }
}

#[test]
fn test_os_bif_error_types() {
    // Test that error types work correctly
    let err1 = OsError::InvalidArgument("test".to_string());
    let err2 = OsError::NotSupported("feature".to_string());
    let err3 = OsError::SystemError("error".to_string());
    
    // Test Display implementation
    let display1 = format!("{}", err1);
    assert!(display1.contains("Invalid argument"));
    
    let display2 = format!("{}", err2);
    assert!(display2.contains("Not supported"));
    
    let display3 = format!("{}", err3);
    assert!(display3.contains("System error"));
    
    // Test Error trait (errors implement Error but source() returns None by default)
    use std::error::Error;
    assert!(err1.source().is_none());
    assert!(err2.source().is_none());
    assert!(err3.source().is_none());
}

// ============================================================================
// Counters BIF Integration Tests
// ============================================================================

#[test]
fn test_counters_bif_full_workflow() {
    // Test complete counter workflow: create -> add -> get -> put -> info
    let counters = CountersBif::new(5).unwrap();
    
    // Initial values should be 0
    assert_eq!(CountersBif::get(&counters, 1).unwrap(), 0);
    
    // Add values
    CountersBif::add(&counters, 1, 10).unwrap();
    CountersBif::add(&counters, 2, 20).unwrap();
    CountersBif::add(&counters, 3, 30).unwrap();
    
    // Verify values
    assert_eq!(CountersBif::get(&counters, 1).unwrap(), 10);
    assert_eq!(CountersBif::get(&counters, 2).unwrap(), 20);
    assert_eq!(CountersBif::get(&counters, 3).unwrap(), 30);
    
    // Put new values
    CountersBif::put(&counters, 1, 100).unwrap();
    assert_eq!(CountersBif::get(&counters, 1).unwrap(), 100);
    
    // Get info
    let info = CountersBif::info(&counters);
    assert_eq!(info.size, 5);
    assert!(info.memory > 0);
}

#[test]
fn test_counters_bif_concurrent_access() {
    use std::thread;
    
    // Test concurrent access to counters
    let counters = CountersBif::new(3).unwrap();
    let counters_clone = counters.clone();
    
    // Spawn threads that modify different counters
    let handle1 = {
        let c = counters_clone.clone();
        thread::spawn(move || {
            for _ in 0..100 {
                CountersBif::add(&c, 1, 1).unwrap();
            }
        })
    };
    
    let handle2 = {
        let c = counters_clone.clone();
        thread::spawn(move || {
            for _ in 0..100 {
                CountersBif::add(&c, 2, 2).unwrap();
            }
        })
    };
    
    let handle3 = {
        let c = counters_clone;
        thread::spawn(move || {
            for _ in 0..100 {
                CountersBif::add(&c, 3, 3).unwrap();
            }
        })
    };
    
    // Wait for all threads
    handle1.join().unwrap();
    handle2.join().unwrap();
    handle3.join().unwrap();
    
    // Verify final values
    assert_eq!(CountersBif::get(&counters, 1).unwrap(), 100);
    assert_eq!(CountersBif::get(&counters, 2).unwrap(), 200);
    assert_eq!(CountersBif::get(&counters, 3).unwrap(), 300);
}

#[test]
fn test_counters_bif_error_handling() {
    // Test error handling for invalid operations
    let counters = CountersBif::new(5).unwrap();
    
    // Invalid index (0)
    assert!(matches!(
        CountersBif::get(&counters, 0),
        Err(CountersError::InvalidArgument(_))
    ));
    
    // Invalid index (too large)
    assert!(matches!(
        CountersBif::get(&counters, 6),
        Err(CountersError::InvalidArgument(_))
    ));
    
    // Invalid add
    assert!(matches!(
        CountersBif::add(&counters, 0, 10),
        Err(CountersError::InvalidArgument(_))
    ));
    
    // Invalid put
    assert!(matches!(
        CountersBif::put(&counters, 10, 10),
        Err(CountersError::InvalidArgument(_))
    ));
}

#[test]
fn test_counters_bif_negative_values() {
    // Test handling of negative values
    let counters = CountersBif::new(3).unwrap();
    
    // Start with positive
    CountersBif::put(&counters, 1, 100).unwrap();
    assert_eq!(CountersBif::get(&counters, 1).unwrap(), 100);
    
    // Add negative
    CountersBif::add(&counters, 1, -50).unwrap();
    assert_eq!(CountersBif::get(&counters, 1).unwrap(), 50);
    
    // Put negative
    CountersBif::put(&counters, 1, -10).unwrap();
    assert_eq!(CountersBif::get(&counters, 1).unwrap(), -10);
    
    // Add to negative
    CountersBif::add(&counters, 1, 5).unwrap();
    assert_eq!(CountersBif::get(&counters, 1).unwrap(), -5);
}

#[test]
fn test_counters_bif_info_consistency() {
    // Test that info is consistent across operations
    let counters1 = CountersBif::new(10).unwrap();
    let info1 = CountersBif::info(&counters1);
    
    // Perform operations
    CountersBif::add(&counters1, 1, 100).unwrap();
    CountersBif::put(&counters1, 5, 50).unwrap();
    
    // Info should remain the same
    let info2 = CountersBif::info(&counters1);
    assert_eq!(info1.size, info2.size);
    assert_eq!(info1.memory, info2.memory);
}

#[test]
fn test_counters_bif_large_array() {
    // Test with large counter array
    let counters = CountersBif::new(1000).unwrap();
    let info = CountersBif::info(&counters);
    assert_eq!(info.size, 1000);
    
    // Test first and last
    CountersBif::put(&counters, 1, 1).unwrap();
    CountersBif::put(&counters, 1000, 1000).unwrap();
    
    assert_eq!(CountersBif::get(&counters, 1).unwrap(), 1);
    assert_eq!(CountersBif::get(&counters, 1000).unwrap(), 1000);
    
    // Test middle
    CountersBif::put(&counters, 500, 500).unwrap();
    assert_eq!(CountersBif::get(&counters, 500).unwrap(), 500);
}

#[test]
fn test_counters_bif_div_ceil_integration() {
    // Test div_ceil helper function
    assert_eq!(CountersBif::div_ceil(10, 3), 4);
    assert_eq!(CountersBif::div_ceil(10, 5), 2);
    assert_eq!(CountersBif::div_ceil(1, 1), 1);
    assert_eq!(CountersBif::div_ceil(0, 5), 0);
}

#[test]
fn test_counters_bif_clone_sharing() {
    // Test that cloned counters share the same underlying data
    let counters1 = CountersBif::new(5).unwrap();
    let counters2 = counters1.clone();
    
    // Modify via counters1
    CountersBif::put(&counters1, 1, 100).unwrap();
    
    // Should see change via counters2
    assert_eq!(CountersBif::get(&counters2, 1).unwrap(), 100);
    
    // Modify via counters2
    CountersBif::add(&counters2, 1, 50).unwrap();
    
    // Should see change via counters1
    assert_eq!(CountersBif::get(&counters1, 1).unwrap(), 150);
}

// ============================================================================
// Unique BIF Integration Tests
// ============================================================================

#[test]
fn test_unique_bif_make_ref_workflow() {
    // Test reference creation workflow
    let ref1 = UniqueBif::make_ref();
    let ref2 = UniqueBif::make_ref();
    let ref3 = UniqueBif::make_ref();
    
    // All should be unique
    assert_ne!(ref1, ref2);
    assert_ne!(ref2, ref3);
    assert_ne!(ref1, ref3);
    
    // Should have valid values
    assert!(ref1.value() > 0);
    assert!(ref2.value() > 0);
    assert!(ref3.value() > 0);
}

#[test]
fn test_unique_bif_unique_integer_workflow() {
    // Test unique integer generation workflow
    let int1 = UniqueBif::unique_integer();
    let int2 = UniqueBif::unique_integer();
    let int3 = UniqueBif::unique_integer();
    
    // All should be unique
    assert_ne!(int1, int2);
    assert_ne!(int2, int3);
    assert_ne!(int1, int3);
}

#[test]
fn test_unique_bif_unique_integer_with_options() {
    // Test unique integer with various option combinations
    let int1 = UniqueBif::unique_integer_with_options(&[]).unwrap();
    let int2 = UniqueBif::unique_integer_with_options(&[UniqueIntegerOption::Positive]).unwrap();
    let int3 = UniqueBif::unique_integer_with_options(&[UniqueIntegerOption::Monotonic]).unwrap();
    let int4 = UniqueBif::unique_integer_with_options(&[
        UniqueIntegerOption::Monotonic,
        UniqueIntegerOption::Positive,
    ]).unwrap();
    
    // All should be unique
    assert_ne!(int1, int2);
    assert_ne!(int2, int3);
    assert_ne!(int3, int4);
    
    // Positive option should generate positive values
    assert!(int2 > 0);
    assert!(int4 > 0);
}

#[test]
fn test_unique_bif_monotonic_ordering() {
    // Test that monotonic integers maintain strict ordering
    let mut prev = UniqueBif::unique_integer_with_options(&[UniqueIntegerOption::Monotonic]).unwrap();
    
    for _ in 0..50 {
        let current = UniqueBif::unique_integer_with_options(&[UniqueIntegerOption::Monotonic]).unwrap();
        assert!(current > prev, "Monotonic integers must be strictly increasing");
        prev = current;
    }
}

#[test]
fn test_unique_bif_concurrent_references() {
    use std::thread;
    
    // Test concurrent reference creation
    let mut handles = Vec::new();
    for _ in 0..10 {
        handles.push(thread::spawn(|| {
            let mut refs = Vec::new();
            for _ in 0..10 {
                refs.push(UniqueBif::make_ref());
            }
            refs
        }));
    }
    
    let mut all_refs = Vec::new();
    for handle in handles {
        all_refs.extend(handle.join().unwrap());
    }
    
    // All references should be unique
    for i in 0..all_refs.len() {
        for j in (i + 1)..all_refs.len() {
            assert_ne!(all_refs[i], all_refs[j], "All references must be unique");
        }
    }
}

#[test]
fn test_unique_bif_concurrent_unique_integers() {
    use std::thread;
    
    // Test concurrent unique integer generation
    let mut handles = Vec::new();
    for _ in 0..10 {
        handles.push(thread::spawn(|| {
            let mut ints = Vec::new();
            for _ in 0..10 {
                ints.push(UniqueBif::unique_integer());
            }
            ints
        }));
    }
    
    let mut all_ints = Vec::new();
    for handle in handles {
        all_ints.extend(handle.join().unwrap());
    }
    
    // All integers should be unique
    for i in 0..all_ints.len() {
        for j in (i + 1)..all_ints.len() {
            assert_ne!(all_ints[i], all_ints[j], "All unique integers must be unique");
        }
    }
}

#[test]
fn test_unique_bif_reference_fields() {
    // Test that reference fields are accessible and meaningful
    let reference = UniqueBif::make_ref();
    
    let thread_id = reference.thread_id();
    let value = reference.value();
    let ref_number = reference.ref_number();
    
    // Thread ID should be positive
    assert!(thread_id > 0);
    
    // Value should be positive
    assert!(value > 0);
    
    // Ref number should be valid
    assert!(ref_number > 0 || value > 0);
}

#[test]
fn test_unique_bif_positive_option_consistency() {
    // Test that positive option consistently generates positive values
    for _ in 0..100 {
        let value = UniqueBif::unique_integer_with_options(&[UniqueIntegerOption::Positive]).unwrap();
        assert!(value > 0, "Positive option must always generate positive integers");
    }
}

#[test]
fn test_unique_bif_monotonic_positive_combination() {
    // Test monotonic + positive combination
    let mut prev = 0;
    for _ in 0..50 {
        let current = UniqueBif::unique_integer_with_options(&[
            UniqueIntegerOption::Monotonic,
            UniqueIntegerOption::Positive,
        ]).unwrap();
        
        assert!(current > 0, "Must be positive");
        assert!(current > prev, "Must be monotonic (strictly increasing)");
        prev = current;
    }
}

// ============================================================================
// Op BIF Integration Tests
// ============================================================================

#[test]
fn test_op_bif_logical_operations() {
    let true_atom = ErlangTerm::Atom("true".to_string());
    let false_atom = ErlangTerm::Atom("false".to_string());

    // Test AND
    assert_eq!(
        OpBif::and(&true_atom, &true_atom).unwrap(),
        ErlangTerm::Atom("true".to_string())
    );
    assert_eq!(
        OpBif::and(&true_atom, &false_atom).unwrap(),
        ErlangTerm::Atom("false".to_string())
    );

    // Test OR
    assert_eq!(
        OpBif::or(&false_atom, &true_atom).unwrap(),
        ErlangTerm::Atom("true".to_string())
    );
    assert_eq!(
        OpBif::or(&false_atom, &false_atom).unwrap(),
        ErlangTerm::Atom("false".to_string())
    );

    // Test XOR
    assert_eq!(
        OpBif::xor(&true_atom, &false_atom).unwrap(),
        ErlangTerm::Atom("true".to_string())
    );
    assert_eq!(
        OpBif::xor(&true_atom, &true_atom).unwrap(),
        ErlangTerm::Atom("false".to_string())
    );

    // Test NOT
    assert_eq!(
        OpBif::not(&true_atom).unwrap(),
        ErlangTerm::Atom("false".to_string())
    );
    assert_eq!(
        OpBif::not(&false_atom).unwrap(),
        ErlangTerm::Atom("true".to_string())
    );
}

#[test]
fn test_op_bif_comparison_operations() {
    let int1 = ErlangTerm::Integer(5);
    let int2 = ErlangTerm::Integer(10);
    let int3 = ErlangTerm::Integer(5);

    // Test greater than
    assert_eq!(
        OpBif::sgt(&int2, &int1),
        ErlangTerm::Atom("true".to_string())
    );
    assert_eq!(
        OpBif::sgt(&int1, &int2),
        ErlangTerm::Atom("false".to_string())
    );

    // Test less than
    assert_eq!(
        OpBif::slt(&int1, &int2),
        ErlangTerm::Atom("true".to_string())
    );

    // Test equality
    assert_eq!(
        OpBif::seq(&int1, &int3),
        ErlangTerm::Atom("true".to_string())
    );
    assert_eq!(
        OpBif::seq(&int1, &int2),
        ErlangTerm::Atom("false".to_string())
    );

    // Test not equal
    assert_eq!(
        OpBif::sneq(&int1, &int2),
        ErlangTerm::Atom("true".to_string())
    );
}

#[test]
fn test_op_bif_type_checks() {
    // Test is_atom
    assert_eq!(
        OpBif::is_atom(&ErlangTerm::Atom("test".to_string())),
        ErlangTerm::Atom("true".to_string())
    );
    assert_eq!(
        OpBif::is_atom(&ErlangTerm::Integer(5)),
        ErlangTerm::Atom("false".to_string())
    );

    // Test is_integer
    assert_eq!(
        OpBif::is_integer(&ErlangTerm::Integer(42)),
        ErlangTerm::Atom("true".to_string())
    );
    assert_eq!(
        OpBif::is_integer(&ErlangTerm::Float(3.14)),
        ErlangTerm::Atom("false".to_string())
    );

    // Test is_number
    assert_eq!(
        OpBif::is_number(&ErlangTerm::Integer(42)),
        ErlangTerm::Atom("true".to_string())
    );
    assert_eq!(
        OpBif::is_number(&ErlangTerm::Float(3.14)),
        ErlangTerm::Atom("true".to_string())
    );
    assert_eq!(
        OpBif::is_number(&ErlangTerm::Atom("test".to_string())),
        ErlangTerm::Atom("false".to_string())
    );

    // Test is_list
    assert_eq!(
        OpBif::is_list(&ErlangTerm::List(vec![])),
        ErlangTerm::Atom("true".to_string())
    );
    assert_eq!(
        OpBif::is_list(&ErlangTerm::Nil),
        ErlangTerm::Atom("true".to_string())
    );

    // Test is_tuple
    assert_eq!(
        OpBif::is_tuple(&ErlangTerm::Tuple(vec![ErlangTerm::Integer(1)])),
        ErlangTerm::Atom("true".to_string())
    );

    // Test is_binary
    assert_eq!(
        OpBif::is_binary(&ErlangTerm::Binary(vec![1, 2, 3])),
        ErlangTerm::Atom("true".to_string())
    );

    // Test is_boolean
    assert_eq!(
        OpBif::is_boolean(&ErlangTerm::Atom("true".to_string())),
        ErlangTerm::Atom("true".to_string())
    );
    assert_eq!(
        OpBif::is_boolean(&ErlangTerm::Atom("false".to_string())),
        ErlangTerm::Atom("true".to_string())
    );
}

#[test]
fn test_op_bif_is_function_with_arity() {
    let func = ErlangTerm::Function { arity: 2 };
    let arity2 = ErlangTerm::Integer(2);
    let arity3 = ErlangTerm::Integer(3);

    assert_eq!(
        OpBif::is_function_with_arity(&func, &arity2).unwrap(),
        ErlangTerm::Atom("true".to_string())
    );
    assert_eq!(
        OpBif::is_function_with_arity(&func, &arity3).unwrap(),
        ErlangTerm::Atom("false".to_string())
    );
}

#[test]
fn test_op_bif_is_record() {
    let record_tag = ErlangTerm::Atom("person".to_string());
    let tuple = ErlangTerm::Tuple(vec![
        ErlangTerm::Atom("person".to_string()),
        ErlangTerm::Atom("John".to_string()),
        ErlangTerm::Integer(30),
    ]);

    assert_eq!(
        OpBif::is_record(&tuple, &record_tag).unwrap(),
        ErlangTerm::Atom("true".to_string())
    );

    let wrong_tag = ErlangTerm::Atom("animal".to_string());
    assert_eq!(
        OpBif::is_record(&tuple, &wrong_tag).unwrap(),
        ErlangTerm::Atom("false".to_string())
    );
}

#[test]
fn test_op_bif_is_record_with_size() {
    let record_tag = ErlangTerm::Atom("point".to_string());
    let size3 = ErlangTerm::Integer(3);
    let tuple = ErlangTerm::Tuple(vec![
        ErlangTerm::Atom("point".to_string()),
        ErlangTerm::Integer(10),
        ErlangTerm::Integer(20),
    ]);

    assert_eq!(
        OpBif::is_record_with_size(&tuple, &record_tag, &size3).unwrap(),
        ErlangTerm::Atom("true".to_string())
    );

    let wrong_size = ErlangTerm::Integer(2);
    assert_eq!(
        OpBif::is_record_with_size(&tuple, &record_tag, &wrong_size).unwrap(),
        ErlangTerm::Atom("false".to_string())
    );
}

#[test]
fn test_op_bif_error_handling() {
    // Test invalid boolean arguments
    let non_bool = ErlangTerm::Integer(5);
    let true_atom = ErlangTerm::Atom("true".to_string());

    assert!(OpBif::and(&non_bool, &true_atom).is_err());
    assert!(OpBif::or(&non_bool, &true_atom).is_err());
    assert!(OpBif::not(&non_bool).is_err());

    // Test invalid arity argument
    let negative = ErlangTerm::Integer(-1);
    assert!(OpBif::is_function_with_arity(&ErlangTerm::Function { arity: 2 }, &negative).is_err());

    // Test invalid record arguments
    let non_atom = ErlangTerm::Integer(5);
    assert!(OpBif::is_record(&ErlangTerm::Tuple(vec![]), &non_atom).is_err());
    assert!(OpBif::is_record_with_size(
        &ErlangTerm::Tuple(vec![]),
        &non_atom,
        &ErlangTerm::Integer(1)
    ).is_err());
}

#[test]
fn test_op_bif_mixed_type_comparisons() {
    let int = ErlangTerm::Integer(5);
    let float = ErlangTerm::Float(5.0);
    let atom = ErlangTerm::Atom("test".to_string());

    // Integer and float comparisons
    assert_eq!(
        OpBif::slt(&int, &ErlangTerm::Float(10.0)),
        ErlangTerm::Atom("true".to_string())
    );

    // Atom comparisons
    let atom_a = ErlangTerm::Atom("a".to_string());
    let atom_b = ErlangTerm::Atom("b".to_string());
    assert_eq!(
        OpBif::slt(&atom_a, &atom_b),
        ErlangTerm::Atom("true".to_string())
    );
}

#[test]
fn test_guard_bif_math_operations() {
    // Test abs
    assert_eq!(
        GuardBif::abs(&ErlangTerm::Integer(-5)).unwrap(),
        ErlangTerm::Integer(5)
    );
    assert_eq!(
        GuardBif::abs(&ErlangTerm::Float(-5.5)).unwrap(),
        ErlangTerm::Float(5.5)
    );

    // Test float conversion
    assert_eq!(
        GuardBif::float(&ErlangTerm::Integer(5)).unwrap(),
        ErlangTerm::Float(5.0)
    );

    // Test trunc
    assert_eq!(
        GuardBif::trunc(&ErlangTerm::Float(5.7)).unwrap(),
        ErlangTerm::Integer(5)
    );
    assert_eq!(
        GuardBif::trunc(&ErlangTerm::Float(-5.7)).unwrap(),
        ErlangTerm::Integer(-5)
    );

    // Test floor
    assert_eq!(
        GuardBif::floor(&ErlangTerm::Float(5.7)).unwrap(),
        ErlangTerm::Integer(5)
    );
    assert_eq!(
        GuardBif::floor(&ErlangTerm::Float(-5.7)).unwrap(),
        ErlangTerm::Integer(-6)
    );

    // Test ceil
    assert_eq!(
        GuardBif::ceil(&ErlangTerm::Float(5.3)).unwrap(),
        ErlangTerm::Integer(6)
    );
    assert_eq!(
        GuardBif::ceil(&ErlangTerm::Float(-5.3)).unwrap(),
        ErlangTerm::Integer(-5)
    );

    // Test round
    assert_eq!(
        GuardBif::round(&ErlangTerm::Float(5.7)).unwrap(),
        ErlangTerm::Integer(6)
    );
    assert_eq!(
        GuardBif::round(&ErlangTerm::Float(5.3)).unwrap(),
        ErlangTerm::Integer(5)
    );
}

#[test]
fn test_guard_bif_size_operations() {
    // Test length
    assert_eq!(
        GuardBif::length(&ErlangTerm::List(vec![
            ErlangTerm::Integer(1),
            ErlangTerm::Integer(2),
            ErlangTerm::Integer(3),
        ]))
        .unwrap(),
        ErlangTerm::Integer(3)
    );
    assert_eq!(
        GuardBif::length(&ErlangTerm::Nil).unwrap(),
        ErlangTerm::Integer(0)
    );

    // Test size
    assert_eq!(
        GuardBif::size(&ErlangTerm::Tuple(vec![
            ErlangTerm::Integer(1),
            ErlangTerm::Integer(2),
        ]))
        .unwrap(),
        ErlangTerm::Integer(2)
    );
    assert_eq!(
        GuardBif::size(&ErlangTerm::Binary(vec![1, 2, 3, 4])).unwrap(),
        ErlangTerm::Integer(4)
    );

    // Test bit_size
    assert_eq!(
        GuardBif::bit_size(&ErlangTerm::Binary(vec![1, 2])).unwrap(),
        ErlangTerm::Integer(16)
    );
    assert_eq!(
        GuardBif::bit_size(&ErlangTerm::Bitstring(vec![1, 2], 15)).unwrap(),
        ErlangTerm::Integer(15)
    );

    // Test byte_size
    assert_eq!(
        GuardBif::byte_size(&ErlangTerm::Binary(vec![1, 2, 3])).unwrap(),
        ErlangTerm::Integer(3)
    );
    assert_eq!(
        GuardBif::byte_size(&ErlangTerm::Bitstring(vec![1, 2], 15)).unwrap(),
        ErlangTerm::Integer(2)
    );
}

#[test]
fn test_guard_bif_comparison_operations() {
    // Test min
    assert_eq!(
        GuardBif::min(&ErlangTerm::Integer(5), &ErlangTerm::Integer(10)).unwrap(),
        ErlangTerm::Integer(5)
    );
    assert_eq!(
        GuardBif::min(&ErlangTerm::Float(5.5), &ErlangTerm::Float(10.5)).unwrap(),
        ErlangTerm::Float(5.5)
    );

    // Test max
    assert_eq!(
        GuardBif::max(&ErlangTerm::Integer(5), &ErlangTerm::Integer(10)).unwrap(),
        ErlangTerm::Integer(10)
    );
    assert_eq!(
        GuardBif::max(&ErlangTerm::Float(5.5), &ErlangTerm::Float(10.5)).unwrap(),
        ErlangTerm::Float(10.5)
    );
}

#[test]
fn test_guard_bif_type_checking() {
    // Test is_integer_3
    assert_eq!(
        GuardBif::is_integer_3(
            &ErlangTerm::Integer(5),
            &ErlangTerm::Integer(1),
            &ErlangTerm::Integer(10)
        )
        .unwrap(),
        ErlangTerm::Atom("true".to_string())
    );

    assert_eq!(
        GuardBif::is_integer_3(
            &ErlangTerm::Integer(0),
            &ErlangTerm::Integer(1),
            &ErlangTerm::Integer(10)
        )
        .unwrap(),
        ErlangTerm::Atom("false".to_string())
    );

    assert_eq!(
        GuardBif::is_integer_3(
            &ErlangTerm::Float(5.0),
            &ErlangTerm::Integer(1),
            &ErlangTerm::Integer(10)
        )
        .unwrap(),
        ErlangTerm::Atom("false".to_string())
    );
}

#[test]
fn test_guard_bif_binary_operations() {
    let binary = ErlangTerm::Binary(vec![1, 2, 3, 4, 5, 6, 7, 8]);

    // Test binary_part_3
    assert_eq!(
        GuardBif::binary_part_3(
            &binary,
            &ErlangTerm::Integer(2),
            &ErlangTerm::Integer(3)
        )
        .unwrap(),
        ErlangTerm::Binary(vec![3, 4, 5])
    );

    // Test binary_part_2
    let tuple = ErlangTerm::Tuple(vec![ErlangTerm::Integer(1), ErlangTerm::Integer(2)]);
    assert_eq!(
        GuardBif::binary_part_2(&binary, &tuple).unwrap(),
        ErlangTerm::Binary(vec![2, 3])
    );
}

#[test]
fn test_guard_bif_error_handling() {
    // Test abs with non-number
    assert!(GuardBif::abs(&ErlangTerm::Atom("test".to_string())).is_err());

    // Test length with non-list
    assert!(GuardBif::length(&ErlangTerm::Integer(5)).is_err());

    // Test size with invalid type
    assert!(GuardBif::size(&ErlangTerm::Atom("test".to_string())).is_err());

    // Test is_integer_3 with invalid min/max
    assert!(GuardBif::is_integer_3(
        &ErlangTerm::Integer(5),
        &ErlangTerm::Float(1.0),
        &ErlangTerm::Integer(10)
    )
    .is_err());

    // Test binary_part_3 with out of bounds
    let binary = ErlangTerm::Binary(vec![1, 2, 3]);
    assert!(GuardBif::binary_part_3(
        &binary,
        &ErlangTerm::Integer(10),
        &ErlangTerm::Integer(2)
    )
    .is_err());
}

#[test]
fn test_guard_bif_integer_edge_cases() {
    // Test abs with i64::MIN (special case - now returns BigInteger)
    let result = GuardBif::abs(&ErlangTerm::Integer(i64::MIN)).unwrap();
    // Should return BigInteger (since i64::MIN.abs() exceeds i64::MAX)
    assert!(matches!(result, ErlangTerm::BigInteger(_)));

    // Test integer operations that return same value
    assert_eq!(
        GuardBif::trunc(&ErlangTerm::Integer(5)).unwrap(),
        ErlangTerm::Integer(5)
    );
    assert_eq!(
        GuardBif::floor(&ErlangTerm::Integer(5)).unwrap(),
        ErlangTerm::Integer(5)
    );
    assert_eq!(
        GuardBif::ceil(&ErlangTerm::Integer(5)).unwrap(),
        ErlangTerm::Integer(5)
    );
    assert_eq!(
        GuardBif::round(&ErlangTerm::Integer(5)).unwrap(),
        ErlangTerm::Integer(5)
    );
}

#[test]
fn test_guard_bif_rational_support() {
    // Test Rational number support in guard BIFs
    use entities_utilities::BigRational;
    
    // Test abs with Rational
    let rational = ErlangTerm::Rational(BigRational::from_fraction(-22, 7).unwrap());
    let result = GuardBif::abs(&rational).unwrap();
    match result {
        ErlangTerm::Rational(_) | ErlangTerm::Integer(_) | ErlangTerm::BigInteger(_) | ErlangTerm::Float(_) => {
            // All valid return types for abs(Rational)
        }
        _ => panic!("abs(Rational) should return Rational, Integer, BigInteger, or Float"),
    }
    
    // Test float with Rational
    let rational_float = ErlangTerm::Rational(BigRational::from_fraction(1, 2).unwrap());
    let result = GuardBif::float(&rational_float).unwrap();
    match result {
        ErlangTerm::Float(f) => {
            assert!((f - 0.5).abs() < 1e-10);
        }
        _ => panic!("float(Rational) should return Float"),
    }
    
    // Test trunc with Rational
    let rational_trunc = ErlangTerm::Rational(BigRational::from_f64(3.7).unwrap());
    assert_eq!(GuardBif::trunc(&rational_trunc).unwrap(), ErlangTerm::Integer(3));
    
    // Test floor with Rational
    let rational_floor = ErlangTerm::Rational(BigRational::from_f64(3.7).unwrap());
    assert_eq!(GuardBif::floor(&rational_floor).unwrap(), ErlangTerm::Integer(3));
    
    // Test ceil with Rational
    let rational_ceil = ErlangTerm::Rational(BigRational::from_f64(3.2).unwrap());
    assert_eq!(GuardBif::ceil(&rational_ceil).unwrap(), ErlangTerm::Integer(4));
    
    // Test round with Rational
    let rational_round = ErlangTerm::Rational(BigRational::from_f64(3.5).unwrap());
    assert_eq!(GuardBif::round(&rational_round).unwrap(), ErlangTerm::Integer(4));
}

#[test]
fn test_guard_bif_big_integer_support() {
    // Test BigInteger support in guard BIFs
    use entities_utilities::BigNumber;
    
    // Test abs with BigInteger
    let big_int = ErlangTerm::BigInteger(BigNumber::from_i64(-1000));
    let result = GuardBif::abs(&big_int).unwrap();
    match result {
        ErlangTerm::Integer(1000) | ErlangTerm::BigInteger(_) => {
            // Valid return types
        }
        _ => panic!("abs(BigInteger) should return Integer or BigInteger"),
    }
    
    // Test float with BigInteger
    let big_int_float = ErlangTerm::BigInteger(BigNumber::from_i64(100));
    let result = GuardBif::float(&big_int_float).unwrap();
    assert_eq!(result, ErlangTerm::Float(100.0));
    
    // Test trunc with BigInteger (should return as-is)
    let big_int_trunc = ErlangTerm::BigInteger(BigNumber::from_i64(1000));
    let result = GuardBif::trunc(&big_int_trunc).unwrap();
    match result {
        ErlangTerm::BigInteger(_) => {
            // Should return as BigInteger
        }
        _ => panic!("trunc(BigInteger) should return BigInteger"),
    }
    
    // Test floor, ceil, round with BigInteger (should return as-is)
    let big_int_ops = ErlangTerm::BigInteger(BigNumber::from_i64(1000));
    assert!(matches!(GuardBif::floor(&big_int_ops).unwrap(), ErlangTerm::BigInteger(_)));
    assert!(matches!(GuardBif::ceil(&big_int_ops).unwrap(), ErlangTerm::BigInteger(_)));
    assert!(matches!(GuardBif::round(&big_int_ops).unwrap(), ErlangTerm::BigInteger(_)));
}

#[test]
fn test_guard_bif_rational_in_comparisons() {
    // Test Rational in min/max operations
    use entities_utilities::BigRational;
    
    let r1 = ErlangTerm::Rational(BigRational::from_fraction(1, 2).unwrap()); // 0.5
    let r2 = ErlangTerm::Rational(BigRational::from_fraction(3, 4).unwrap()); // 0.75
    let int = ErlangTerm::Integer(1);
    
    // Rational vs Rational
    assert_eq!(GuardBif::min(&r1, &r2).unwrap(), r1);
    assert_eq!(GuardBif::max(&r1, &r2).unwrap(), r2);
    
    // Rational vs Integer
    assert_eq!(GuardBif::min(&r1, &int).unwrap(), r1);
    assert_eq!(GuardBif::max(&r1, &int).unwrap(), int);
}

#[test]
fn test_guard_bif_rational_in_is_integer_3() {
    // Test Rational in is_integer_3
    use entities_utilities::BigRational;
    
    // Rational that is an integer
    let rational_int = ErlangTerm::Rational(BigRational::from_i64(5));
    assert_eq!(
        GuardBif::is_integer_3(
            &rational_int,
            &ErlangTerm::Integer(1),
            &ErlangTerm::Integer(10)
        ).unwrap(),
        ErlangTerm::Atom("true".to_string())
    );
    
    // Rational that is NOT an integer
    let rational_frac = ErlangTerm::Rational(BigRational::from_fraction(22, 7).unwrap());
    assert_eq!(
        GuardBif::is_integer_3(
            &rational_frac,
            &ErlangTerm::Integer(1),
            &ErlangTerm::Integer(10)
        ).unwrap(),
        ErlangTerm::Atom("false".to_string())
    );
    
    // Rational that is an integer but out of range
    let rational_out = ErlangTerm::Rational(BigRational::from_i64(100));
    assert_eq!(
        GuardBif::is_integer_3(
            &rational_out,
            &ErlangTerm::Integer(1),
            &ErlangTerm::Integer(10)
        ).unwrap(),
        ErlangTerm::Atom("false".to_string())
    );
}

#[test]
fn test_guard_bif_big_integer_in_is_integer_3() {
    // Test BigInteger in is_integer_3
    use entities_utilities::BigNumber;
    
    // BigInteger that fits in i64 range
    let big_int = ErlangTerm::BigInteger(BigNumber::from_i64(5));
    assert_eq!(
        GuardBif::is_integer_3(
            &big_int,
            &ErlangTerm::Integer(1),
            &ErlangTerm::Integer(10)
        ).unwrap(),
        ErlangTerm::Atom("true".to_string())
    );
    
    // BigInteger that fits in i64 but out of range
    let big_int_out = ErlangTerm::BigInteger(BigNumber::from_i64(100));
    assert_eq!(
        GuardBif::is_integer_3(
            &big_int_out,
            &ErlangTerm::Integer(1),
            &ErlangTerm::Integer(10)
        ).unwrap(),
        ErlangTerm::Atom("false".to_string())
    );
}

#[test]
fn test_guard_bif_binary_part_3_edge_cases() {
    // Test binary_part_3 with various edge cases
    let binary = ErlangTerm::Binary(vec![1, 2, 3, 4, 5]);
    
    // Test with negative start (should fail)
    assert!(GuardBif::binary_part_3(
        &binary,
        &ErlangTerm::Integer(-1),
        &ErlangTerm::Integer(2)
    ).is_err());
    
    // Test with negative length (should fail)
    assert!(GuardBif::binary_part_3(
        &binary,
        &ErlangTerm::Integer(0),
        &ErlangTerm::Integer(-1)
    ).is_err());
    
    // Test with non-integer start (should fail)
    assert!(GuardBif::binary_part_3(
        &binary,
        &ErlangTerm::Float(1.5),
        &ErlangTerm::Integer(2)
    ).is_err());
    
    // Test with non-integer length (should fail)
    assert!(GuardBif::binary_part_3(
        &binary,
        &ErlangTerm::Integer(0),
        &ErlangTerm::Float(2.5)
    ).is_err());
    
    // Test with bitstring
    let bitstring = ErlangTerm::Bitstring(vec![1, 2, 3], 24);
    let result = GuardBif::binary_part_3(
        &bitstring,
        &ErlangTerm::Integer(0),
        &ErlangTerm::Integer(2)
    ).unwrap();
    match result {
        ErlangTerm::Bitstring(data, bits) => {
            assert_eq!(data.len(), 2);
            assert_eq!(bits, 16);
        }
        _ => panic!("Expected Bitstring"),
    }
    
    // Test with bitstring out of bounds
    assert!(GuardBif::binary_part_3(
        &bitstring,
        &ErlangTerm::Integer(2),
        &ErlangTerm::Integer(5)
    ).is_err());
}

#[test]
fn test_guard_bif_binary_part_2_edge_cases() {
    // Test binary_part_2 with edge cases
    let binary = ErlangTerm::Binary(vec![1, 2, 3, 4, 5]);
    
    // Test with wrong tuple size (1 element)
    let tuple1 = ErlangTerm::Tuple(vec![ErlangTerm::Integer(1)]);
    assert!(GuardBif::binary_part_2(&binary, &tuple1).is_err());
    
    // Test with wrong tuple size (3 elements)
    let tuple3 = ErlangTerm::Tuple(vec![
        ErlangTerm::Integer(1),
        ErlangTerm::Integer(2),
        ErlangTerm::Integer(3)
    ]);
    assert!(GuardBif::binary_part_2(&binary, &tuple3).is_err());
    
    // Test with non-tuple
    assert!(GuardBif::binary_part_2(&binary, &ErlangTerm::Integer(5)).is_err());
}

#[test]
fn test_guard_bif_min_max_non_comparable() {
    // Test min/max with non-comparable types
    let atom = ErlangTerm::Atom("test".to_string());
    let pid = ErlangTerm::Pid(1);
    
    // These should fail
    assert!(GuardBif::min(&atom, &pid).is_err());
    assert!(GuardBif::max(&atom, &pid).is_err());
}

#[test]
fn test_guard_bif_zero_values() {
    // Test operations with zero values
    use entities_utilities::BigRational;
    
    // Test abs with Float(0.0)
    assert_eq!(
        GuardBif::abs(&ErlangTerm::Float(0.0)).unwrap(),
        ErlangTerm::Float(0.0)
    );
    
    // Test trunc, floor, ceil, round with Float(0.0)
    assert_eq!(GuardBif::trunc(&ErlangTerm::Float(0.0)).unwrap(), ErlangTerm::Integer(0));
    assert_eq!(GuardBif::floor(&ErlangTerm::Float(0.0)).unwrap(), ErlangTerm::Integer(0));
    assert_eq!(GuardBif::ceil(&ErlangTerm::Float(0.0)).unwrap(), ErlangTerm::Integer(0));
    assert_eq!(GuardBif::round(&ErlangTerm::Float(0.0)).unwrap(), ErlangTerm::Integer(0));
    
    // Test with Rational zero
    let rational_zero = ErlangTerm::Rational(BigRational::from_i64(0));
    assert_eq!(GuardBif::trunc(&rational_zero).unwrap(), ErlangTerm::Integer(0));
}

#[test]
fn test_guard_bif_large_float_conversions() {
    // Test large float values that convert to BigInteger
    let large_float = ErlangTerm::Float(1e20);
    let result = GuardBif::trunc(&large_float).unwrap();
    assert!(matches!(result, ErlangTerm::BigInteger(_)));
    
    let result = GuardBif::floor(&large_float).unwrap();
    assert!(matches!(result, ErlangTerm::BigInteger(_)));
    
    let result = GuardBif::ceil(&large_float).unwrap();
    assert!(matches!(result, ErlangTerm::BigInteger(_)));
    
    let result = GuardBif::round(&large_float).unwrap();
    assert!(matches!(result, ErlangTerm::BigInteger(_)));
}

#[test]
fn test_guard_bif_size_with_bitstring() {
    // Test size operations with bitstring
    let bitstring = ErlangTerm::Bitstring(vec![1, 2, 3], 24);
    
    // size should return byte size
    assert_eq!(GuardBif::size(&bitstring).unwrap(), ErlangTerm::Integer(3));
    
    // bit_size should return bit size
    assert_eq!(GuardBif::bit_size(&bitstring).unwrap(), ErlangTerm::Integer(24));
    
    // byte_size should return byte size
    assert_eq!(GuardBif::byte_size(&bitstring).unwrap(), ErlangTerm::Integer(3));
}

#[test]
fn test_guard_bif_rational_arithmetic_integration() {
    // Test Rational arithmetic operations in guard context
    use entities_utilities::BigRational;
    
    // Test with Rational that represents exact integer
    let rational_int = ErlangTerm::Rational(BigRational::from_i64(42));
    assert_eq!(GuardBif::trunc(&rational_int).unwrap(), ErlangTerm::Integer(42));
    assert_eq!(GuardBif::floor(&rational_int).unwrap(), ErlangTerm::Integer(42));
    assert_eq!(GuardBif::ceil(&rational_int).unwrap(), ErlangTerm::Integer(42));
    assert_eq!(GuardBif::round(&rational_int).unwrap(), ErlangTerm::Integer(42));
    
    // Test with Rational fraction
    let rational_frac = ErlangTerm::Rational(BigRational::from_fraction(1, 3).unwrap());
    let result = GuardBif::float(&rational_frac).unwrap();
    match result {
        ErlangTerm::Float(f) => {
            assert!((f - 1.0/3.0).abs() < 1e-10);
        }
        _ => panic!("Expected Float"),
    }
}

#[test]
fn test_guard_bif_comprehensive_binary_operations() {
    // Comprehensive binary operations testing
    let binary = ErlangTerm::Binary(vec![1, 2, 3, 4, 5, 6, 7, 8]);
    
    // Test extracting from start
    assert_eq!(
        GuardBif::binary_part_3(
            &binary,
            &ErlangTerm::Integer(0),
            &ErlangTerm::Integer(2)
        ).unwrap(),
        ErlangTerm::Binary(vec![1, 2])
    );
    
    // Test extracting to end
    assert_eq!(
        GuardBif::binary_part_3(
            &binary,
            &ErlangTerm::Integer(6),
            &ErlangTerm::Integer(2)
        ).unwrap(),
        ErlangTerm::Binary(vec![7, 8])
    );
    
    // Test extracting middle
    assert_eq!(
        GuardBif::binary_part_3(
            &binary,
            &ErlangTerm::Integer(2),
            &ErlangTerm::Integer(3)
        ).unwrap(),
        ErlangTerm::Binary(vec![3, 4, 5])
    );
    
    // Test binary_part_2 with various tuples
    let tuple1 = ErlangTerm::Tuple(vec![ErlangTerm::Integer(0), ErlangTerm::Integer(1)]);
    assert_eq!(
        GuardBif::binary_part_2(&binary, &tuple1).unwrap(),
        ErlangTerm::Binary(vec![1])
    );
    
    let tuple2 = ErlangTerm::Tuple(vec![ErlangTerm::Integer(4), ErlangTerm::Integer(4)]);
    assert_eq!(
        GuardBif::binary_part_2(&binary, &tuple2).unwrap(),
        ErlangTerm::Binary(vec![5, 6, 7, 8])
    );
}

// ============================================================================
// Lists BIF Integration Tests
// ============================================================================

#[test]
fn test_lists_bif_append_2_workflow() {
    // Test append operation workflow
    let lhs = ErlangTerm::List(vec![
        ErlangTerm::Integer(1),
        ErlangTerm::Integer(2),
    ]);
    let rhs = ErlangTerm::List(vec![
        ErlangTerm::Integer(3),
        ErlangTerm::Integer(4),
    ]);
    
    let result = ListsBif::append_2(&lhs, &rhs).unwrap();
    
    if let ErlangTerm::List(result_vec) = result {
        assert_eq!(result_vec.len(), 4);
        assert_eq!(result_vec[0], ErlangTerm::Integer(1));
        assert_eq!(result_vec[1], ErlangTerm::Integer(2));
        assert_eq!(result_vec[2], ErlangTerm::Integer(3));
        assert_eq!(result_vec[3], ErlangTerm::Integer(4));
    } else {
        panic!("Expected List");
    }
}

#[test]
fn test_lists_bif_append_2_nested_lists() {
    // Test appending nested lists
    let lhs = ErlangTerm::List(vec![
        ErlangTerm::List(vec![ErlangTerm::Integer(1)]),
    ]);
    let rhs = ErlangTerm::List(vec![
        ErlangTerm::List(vec![ErlangTerm::Integer(2)]),
    ]);
    
    let result = ListsBif::append_2(&lhs, &rhs).unwrap();
    
    if let ErlangTerm::List(result_vec) = result {
        assert_eq!(result_vec.len(), 2);
        // Both elements should be lists
        assert!(matches!(result_vec[0], ErlangTerm::List(_)));
        assert!(matches!(result_vec[1], ErlangTerm::List(_)));
    } else {
        panic!("Expected List");
    }
}

#[test]
fn test_lists_bif_subtract_2_workflow() {
    // Test subtract operation workflow
    let lhs = ErlangTerm::List(vec![
        ErlangTerm::Integer(1),
        ErlangTerm::Integer(2),
        ErlangTerm::Integer(3),
        ErlangTerm::Integer(4),
    ]);
    let rhs = ErlangTerm::List(vec![
        ErlangTerm::Integer(2),
        ErlangTerm::Integer(4),
    ]);
    
    let result = ListsBif::subtract_2(&lhs, &rhs).unwrap();
    
    if let ErlangTerm::List(result_vec) = result {
        assert_eq!(result_vec.len(), 2);
        assert_eq!(result_vec[0], ErlangTerm::Integer(1));
        assert_eq!(result_vec[1], ErlangTerm::Integer(3));
    } else {
        panic!("Expected List");
    }
}

#[test]
fn test_lists_bif_subtract_2_with_atoms() {
    // Test subtract with atoms
    let lhs = ErlangTerm::List(vec![
        ErlangTerm::Atom("a".to_string()),
        ErlangTerm::Atom("b".to_string()),
        ErlangTerm::Atom("c".to_string()),
    ]);
    let rhs = ErlangTerm::List(vec![ErlangTerm::Atom("b".to_string())]);
    
    let result = ListsBif::subtract_2(&lhs, &rhs).unwrap();
    
    if let ErlangTerm::List(result_vec) = result {
        assert_eq!(result_vec.len(), 2);
        assert_eq!(result_vec[0], ErlangTerm::Atom("a".to_string()));
        assert_eq!(result_vec[1], ErlangTerm::Atom("c".to_string()));
    } else {
        panic!("Expected List");
    }
}

#[test]
fn test_lists_bif_member_2_workflow() {
    // Test member operation workflow
    let list = ErlangTerm::List(vec![
        ErlangTerm::Integer(1),
        ErlangTerm::Integer(2),
        ErlangTerm::Integer(3),
    ]);
    
    // Element found
    let result1 = ListsBif::member_2(&ErlangTerm::Integer(2), &list).unwrap();
    assert_eq!(result1, ErlangTerm::Atom("true".to_string()));
    
    // Element not found
    let result2 = ListsBif::member_2(&ErlangTerm::Integer(5), &list).unwrap();
    assert_eq!(result2, ErlangTerm::Atom("false".to_string()));
}

#[test]
fn test_lists_bif_member_2_with_type_coercion() {
    // Test member with type coercion (Integer == Float)
    let list = ErlangTerm::List(vec![
        ErlangTerm::Integer(1),
        ErlangTerm::Float(2.0),
        ErlangTerm::Integer(3),
    ]);
    
    // Integer(1) should match Integer(1)
    let result1 = ListsBif::member_2(&ErlangTerm::Integer(1), &list).unwrap();
    assert_eq!(result1, ErlangTerm::Atom("true".to_string()));
    
    // Integer(2) should match Float(2.0) due to structural equality
    let result2 = ListsBif::member_2(&ErlangTerm::Integer(2), &list).unwrap();
    assert_eq!(result2, ErlangTerm::Atom("true".to_string()));
}

#[test]
fn test_lists_bif_reverse_2_workflow() {
    // Test reverse operation workflow
    let list = ErlangTerm::List(vec![
        ErlangTerm::Integer(1),
        ErlangTerm::Integer(2),
        ErlangTerm::Integer(3),
    ]);
    let tail = ErlangTerm::Nil;
    
    let result = ListsBif::reverse_2(&list, &tail).unwrap();
    
    if let ErlangTerm::List(result_vec) = result {
        assert_eq!(result_vec.len(), 3);
        assert_eq!(result_vec[0], ErlangTerm::Integer(3));
        assert_eq!(result_vec[1], ErlangTerm::Integer(2));
        assert_eq!(result_vec[2], ErlangTerm::Integer(1));
    } else {
        panic!("Expected List");
    }
}

#[test]
fn test_lists_bif_reverse_2_with_tail() {
    // Test reverse with tail
    let list = ErlangTerm::List(vec![
        ErlangTerm::Integer(1),
        ErlangTerm::Integer(2),
    ]);
    let tail = ErlangTerm::List(vec![
        ErlangTerm::Integer(3),
        ErlangTerm::Integer(4),
    ]);
    
    let result = ListsBif::reverse_2(&list, &tail).unwrap();
    
    if let ErlangTerm::List(result_vec) = result {
        assert_eq!(result_vec.len(), 4);
        assert_eq!(result_vec[0], ErlangTerm::Integer(2));
        assert_eq!(result_vec[1], ErlangTerm::Integer(1));
        assert_eq!(result_vec[2], ErlangTerm::Integer(3));
        assert_eq!(result_vec[3], ErlangTerm::Integer(4));
    } else {
        panic!("Expected List");
    }
}

#[test]
fn test_lists_bif_keyfind_3_workflow() {
    // Test keyfind operation workflow
    let list = ErlangTerm::List(vec![
        ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("name".to_string()),
            ErlangTerm::Atom("alice".to_string()),
            ErlangTerm::Integer(25),
        ]),
        ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("name".to_string()),
            ErlangTerm::Atom("bob".to_string()),
            ErlangTerm::Integer(30),
        ]),
    ]);
    
    // Find by key at position 1
    let result = ListsBif::keyfind_3(
        &ErlangTerm::Atom("name".to_string()),
        &ErlangTerm::Integer(1),
        &list,
    )
    .unwrap();
    
    // Should return first tuple
    if let ErlangTerm::Tuple(tuple_vec) = result {
        assert_eq!(tuple_vec[0], ErlangTerm::Atom("name".to_string()));
        assert_eq!(tuple_vec[1], ErlangTerm::Atom("alice".to_string()));
    } else {
        panic!("Expected Tuple");
    }
}

#[test]
fn test_lists_bif_keyfind_3_not_found() {
    // Test keyfind when key not found
    let list = ErlangTerm::List(vec![
        ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("name".to_string()),
            ErlangTerm::Atom("alice".to_string()),
        ]),
    ]);
    
    let result = ListsBif::keyfind_3(
        &ErlangTerm::Atom("age".to_string()),
        &ErlangTerm::Integer(1),
        &list,
    )
    .unwrap();
    
    assert_eq!(result, ErlangTerm::Atom("false".to_string()));
}

#[test]
fn test_lists_bif_keymember_3_workflow() {
    // Test keymember operation workflow
    let list = ErlangTerm::List(vec![
        ErlangTerm::Tuple(vec![
            ErlangTerm::Integer(1),
            ErlangTerm::Atom("one".to_string()),
        ]),
        ErlangTerm::Tuple(vec![
            ErlangTerm::Integer(2),
            ErlangTerm::Atom("two".to_string()),
        ]),
    ]);
    
    // Key exists
    let result1 = ListsBif::keymember_3(
        &ErlangTerm::Integer(1),
        &ErlangTerm::Integer(1),
        &list,
    )
    .unwrap();
    assert_eq!(result1, ErlangTerm::Atom("true".to_string()));
    
    // Key doesn't exist
    let result2 = ListsBif::keymember_3(
        &ErlangTerm::Integer(3),
        &ErlangTerm::Integer(1),
        &list,
    )
    .unwrap();
    assert_eq!(result2, ErlangTerm::Atom("false".to_string()));
}

#[test]
fn test_lists_bif_keysearch_3_workflow() {
    // Test keysearch operation workflow
    let list = ErlangTerm::List(vec![
        ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("key".to_string()),
            ErlangTerm::Integer(42),
        ]),
    ]);
    
    let result = ListsBif::keysearch_3(
        &ErlangTerm::Atom("key".to_string()),
        &ErlangTerm::Integer(1),
        &list,
    )
    .unwrap();
    
    // Should return {value, Tuple}
    if let ErlangTerm::Tuple(tuple_vec) = result {
        assert_eq!(tuple_vec.len(), 2);
        assert_eq!(tuple_vec[0], ErlangTerm::Atom("value".to_string()));
        if let ErlangTerm::Tuple(inner_tuple) = &tuple_vec[1] {
            assert_eq!(inner_tuple[0], ErlangTerm::Atom("key".to_string()));
            assert_eq!(inner_tuple[1], ErlangTerm::Integer(42));
        } else {
            panic!("Expected inner Tuple");
        }
    } else {
        panic!("Expected Tuple");
    }
}

#[test]
fn test_lists_bif_error_handling() {
    // Test error handling for invalid arguments
    
    // append_2 with non-list LHS
    let result = ListsBif::append_2(&ErlangTerm::Integer(1), &ErlangTerm::Nil);
    assert!(result.is_err());
    
    // subtract_2 with non-list LHS
    let result = ListsBif::subtract_2(&ErlangTerm::Integer(1), &ErlangTerm::Nil);
    assert!(result.is_err());
    
    // subtract_2 with non-list RHS
    let result = ListsBif::subtract_2(
        &ErlangTerm::List(vec![ErlangTerm::Integer(1)]),
        &ErlangTerm::Integer(1),
    );
    assert!(result.is_err());
    
    // member_2 with non-list
    let result = ListsBif::member_2(&ErlangTerm::Integer(1), &ErlangTerm::Integer(1));
    assert!(result.is_err());
    
    // reverse_2 with non-list
    let result = ListsBif::reverse_2(&ErlangTerm::Integer(1), &ErlangTerm::Nil);
    assert!(result.is_err());
    
    // keyfind_3 with invalid position
    let result = ListsBif::keyfind_3(
        &ErlangTerm::Integer(1),
        &ErlangTerm::Integer(0), // Position must be >= 1
        &ErlangTerm::Nil,
    );
    assert!(result.is_err());
    
    // keyfind_3 with non-list
    let result = ListsBif::keyfind_3(
        &ErlangTerm::Integer(1),
        &ErlangTerm::Integer(1),
        &ErlangTerm::Integer(1),
    );
    assert!(result.is_err());
}

#[test]
fn test_lists_bif_keyfind_3_position_bounds() {
    // Test keyfind with position out of bounds
    let list = ErlangTerm::List(vec![
        ErlangTerm::Tuple(vec![
            ErlangTerm::Integer(1),
            ErlangTerm::Integer(2),
        ]),
    ]);
    
    // Position 3 is out of bounds (tuple only has 2 elements)
    let result = ListsBif::keyfind_3(
        &ErlangTerm::Integer(1),
        &ErlangTerm::Integer(3),
        &list,
    )
    .unwrap();
    // Should return false (position out of bounds)
    assert_eq!(result, ErlangTerm::Atom("false".to_string()));
}

#[test]
fn test_lists_bif_large_lists() {
    // Test operations with larger lists
    let mut large_list = Vec::new();
    for i in 0..100 {
        large_list.push(ErlangTerm::Integer(i));
    }
    let list = ErlangTerm::List(large_list);
    
    // Test member
    let result = ListsBif::member_2(&ErlangTerm::Integer(50), &list).unwrap();
    assert_eq!(result, ErlangTerm::Atom("true".to_string()));
    
    // Test reverse
    let reversed = ListsBif::reverse_2(&list, &ErlangTerm::Nil).unwrap();
    if let ErlangTerm::List(reversed_vec) = reversed {
        assert_eq!(reversed_vec.len(), 100);
        assert_eq!(reversed_vec[0], ErlangTerm::Integer(99));
        assert_eq!(reversed_vec[99], ErlangTerm::Integer(0));
    } else {
        panic!("Expected List");
    }
}

#[test]
fn test_lists_bif_subtract_2_preserves_order() {
    // Test that subtract preserves order of remaining elements
    let lhs = ErlangTerm::List(vec![
        ErlangTerm::Integer(1),
        ErlangTerm::Integer(2),
        ErlangTerm::Integer(3),
        ErlangTerm::Integer(4),
        ErlangTerm::Integer(5),
    ]);
    let rhs = ErlangTerm::List(vec![
        ErlangTerm::Integer(2),
        ErlangTerm::Integer(4),
    ]);
    
    let result = ListsBif::subtract_2(&lhs, &rhs).unwrap();
    
    if let ErlangTerm::List(result_vec) = result {
        assert_eq!(result_vec.len(), 3);
        assert_eq!(result_vec[0], ErlangTerm::Integer(1));
        assert_eq!(result_vec[1], ErlangTerm::Integer(3));
        assert_eq!(result_vec[2], ErlangTerm::Integer(5));
    } else {
        panic!("Expected List");
    }
}


// ============================================================================
// Persistent Term BIF Integration Tests
// ============================================================================

#[test]
fn test_persistent_bif_put_2_and_get_1() {
    // Clear storage first
    let _ = PersistentBif::erase_all_0();
    
    let key = ErlangTerm::Atom("test_key".to_string());
    let value = ErlangTerm::Integer(42);
    
    // Put a value
    let put_result = PersistentBif::put_2(&key, &value).unwrap();
    assert_eq!(put_result, ErlangTerm::Atom("ok".to_string()));
    
    // Get it back
    let get_result = PersistentBif::get_1(&key).unwrap();
    assert_eq!(get_result, value);
}

#[test]
fn test_persistent_bif_get_2_with_default() {
    let _ = PersistentBif::erase_all_0();
    
    let key = ErlangTerm::Atom("default_key".to_string());
    let default = ErlangTerm::Integer(0);
    
    // Get with default when key doesn't exist
    let result1 = PersistentBif::get_2(&key, &default).unwrap();
    assert_eq!(result1, default);
    
    // Store a value
    let value = ErlangTerm::Integer(100);
    PersistentBif::put_2(&key, &value).unwrap();
    
    // Get with default when key exists (should return stored value)
    let result2 = PersistentBif::get_2(&key, &default).unwrap();
    assert_eq!(result2, value);
}

#[test]
fn test_persistent_bif_get_0_workflow() {
    let _ = PersistentBif::erase_all_0();
    
    // Store multiple entries
    PersistentBif::put_2(
        &ErlangTerm::Atom("key1".to_string()),
        &ErlangTerm::Integer(1),
    ).unwrap();
    PersistentBif::put_2(
        &ErlangTerm::Atom("key2".to_string()),
        &ErlangTerm::Integer(2),
    ).unwrap();
    
    // Get all entries
    let all = PersistentBif::get_0().unwrap();
    if let ErlangTerm::List(list) = all {
        assert!(list.len() >= 2);
        // Verify both entries are present
        let mut found_key1 = false;
        let mut found_key2 = false;
        for entry in &list {
            if let ErlangTerm::Tuple(tuple) = entry {
                if tuple.len() == 2 {
                    if tuple[0] == ErlangTerm::Atom("key1".to_string()) {
                        found_key1 = true;
                    }
                    if tuple[0] == ErlangTerm::Atom("key2".to_string()) {
                        found_key2 = true;
                    }
                }
            }
        }
        assert!(found_key1);
        assert!(found_key2);
    } else {
        panic!("Expected List");
    }
}

#[test]
fn test_persistent_bif_erase_1_workflow() {
    let _ = PersistentBif::erase_all_0();
    
    let key = ErlangTerm::Atom("erase_key".to_string());
    let value = ErlangTerm::Integer(200);
    
    // Store a value
    PersistentBif::put_2(&key, &value).unwrap();
    
    // Verify it exists
    let get_result = PersistentBif::get_1(&key).unwrap();
    assert_eq!(get_result, value);
    
    // Erase it
    let erase_result = PersistentBif::erase_1(&key).unwrap();
    assert_eq!(erase_result, ErlangTerm::Atom("true".to_string()));
    
    // Verify it's gone
    let get_result_after = PersistentBif::get_1(&key);
    assert!(get_result_after.is_err());
}

#[test]
fn test_persistent_bif_erase_1_not_found() {
    let _ = PersistentBif::erase_all_0();
    
    let key = ErlangTerm::Atom("nonexistent".to_string());
    
    // Try to erase a non-existent key
    let result = PersistentBif::erase_1(&key).unwrap();
    assert_eq!(result, ErlangTerm::Atom("false".to_string()));
}

#[test]
fn test_persistent_bif_erase_all_0_workflow() {
    let _ = PersistentBif::erase_all_0();
    
    // Store multiple entries
    PersistentBif::put_2(
        &ErlangTerm::Atom("key1".to_string()),
        &ErlangTerm::Integer(1),
    ).unwrap();
    PersistentBif::put_2(
        &ErlangTerm::Atom("key2".to_string()),
        &ErlangTerm::Integer(2),
    ).unwrap();
    
    // Verify they exist
    let all_before = PersistentBif::get_0().unwrap();
    if let ErlangTerm::List(list) = all_before {
        assert!(list.len() >= 2);
    } else {
        panic!("Expected List");
    }
    
    // Erase all
    let erase_result = PersistentBif::erase_all_0().unwrap();
    assert_eq!(erase_result, ErlangTerm::Atom("true".to_string()));
    
    // Verify they're gone
    let all_after = PersistentBif::get_0().unwrap();
    if let ErlangTerm::List(list) = all_after {
        assert_eq!(list.len(), 0);
    } else {
        panic!("Expected List");
    }
}

#[test]
fn test_persistent_bif_info_0_workflow() {
    let _ = PersistentBif::erase_all_0();
    
    // Store some entries
    PersistentBif::put_2(
        &ErlangTerm::Atom("info_key1".to_string()),
        &ErlangTerm::Integer(1),
    ).unwrap();
    PersistentBif::put_2(
        &ErlangTerm::Atom("info_key2".to_string()),
        &ErlangTerm::Integer(2),
    ).unwrap();
    
    // Get info
    let info = PersistentBif::info_0().unwrap();
    if let ErlangTerm::Map(map) = info {
        let count = map.get(&ErlangTerm::Atom("count".to_string()));
        assert!(count.is_some());
        if let Some(ErlangTerm::Integer(count_val)) = count {
            assert!(*count_val >= 2);
        }
        
        let memory = map.get(&ErlangTerm::Atom("memory".to_string()));
        assert!(memory.is_some());
    } else {
        panic!("Expected Map");
    }
}

#[test]
fn test_persistent_bif_update_existing_key() {
    let _ = PersistentBif::erase_all_0();
    
    let key = ErlangTerm::Atom("update_key".to_string());
    let value1 = ErlangTerm::Integer(1);
    let value2 = ErlangTerm::Integer(2);
    
    // Store initial value
    PersistentBif::put_2(&key, &value1).unwrap();
    let get1 = PersistentBif::get_1(&key).unwrap();
    assert_eq!(get1, value1);
    
    // Update with new value
    PersistentBif::put_2(&key, &value2).unwrap();
    let get2 = PersistentBif::get_1(&key).unwrap();
    assert_eq!(get2, value2);
}

#[test]
fn test_persistent_bif_different_key_types() {
    let _ = PersistentBif::erase_all_0();
    
    // Store with atom key
    PersistentBif::put_2(
        &ErlangTerm::Atom("atom_key".to_string()),
        &ErlangTerm::Integer(1),
    ).unwrap();
    
    // Store with integer key
    PersistentBif::put_2(
        &ErlangTerm::Integer(123),
        &ErlangTerm::Atom("int_value".to_string()),
    ).unwrap();
    
    // Store with tuple key
    let tuple_key = ErlangTerm::Tuple(vec![
        ErlangTerm::Atom("tuple".to_string()),
        ErlangTerm::Integer(456),
    ]);
    PersistentBif::put_2(
        &tuple_key,
        &ErlangTerm::Float(3.14),
    ).unwrap();
    
    // Retrieve all
    let atom_val = PersistentBif::get_1(&ErlangTerm::Atom("atom_key".to_string())).unwrap();
    assert_eq!(atom_val, ErlangTerm::Integer(1));
    
    let int_val = PersistentBif::get_1(&ErlangTerm::Integer(123)).unwrap();
    assert_eq!(int_val, ErlangTerm::Atom("int_value".to_string()));
    
    let tuple_val = PersistentBif::get_1(&tuple_key).unwrap();
    assert_eq!(tuple_val, ErlangTerm::Float(3.14));
}

#[test]
fn test_persistent_bif_complex_values() {
    let _ = PersistentBif::erase_all_0();
    
    let key = ErlangTerm::Atom("complex_key".to_string());
    let complex_value = ErlangTerm::List(vec![
        ErlangTerm::Integer(1),
        ErlangTerm::Integer(2),
        ErlangTerm::Tuple(vec![
            ErlangTerm::Atom("nested".to_string()),
            ErlangTerm::Integer(3),
        ]),
    ]);
    
    // Store complex value
    PersistentBif::put_2(&key, &complex_value).unwrap();
    
    // Retrieve it
    let retrieved = PersistentBif::get_1(&key).unwrap();
    assert_eq!(retrieved, complex_value);
}

#[test]
fn test_persistent_bif_get_1_error() {
    let _ = PersistentBif::erase_all_0();
    
    let key = ErlangTerm::Atom("nonexistent_key".to_string());
    
    // Try to get a non-existent key
    let result = PersistentBif::get_1(&key);
    assert!(result.is_err());
    if let Err(PersistentError::BadArgument(msg)) = result {
        assert!(msg.contains("not found"));
    } else {
        panic!("Expected BadArgument error");
    }
}

#[test]
fn test_persistent_bif_info_0_empty() {
    let _ = PersistentBif::erase_all_0();
    
    let info = PersistentBif::info_0().unwrap();
    if let ErlangTerm::Map(map) = info {
        let count = map.get(&ErlangTerm::Atom("count".to_string()));
        assert_eq!(count, Some(&ErlangTerm::Integer(0)));
    } else {
        panic!("Expected Map");
    }
}

// ============================================================================
// Code Loading BIF Integration Tests
// ============================================================================

#[test]
fn test_load_bif_delete_module_1_workflow() {
    use usecases_bifs::load::{LoadBif, LoadError};
    use usecases_bifs::op::ErlangTerm;
    use usecases_bifs::load::ModuleStatus;
    
    LoadBif::clear_all();
    
    // Register a module
    LoadBif::register_module("test_module", ModuleStatus::Loaded, false, false);
    
    // Verify it's loaded
    let loaded = LoadBif::module_loaded_1(&ErlangTerm::Atom("test_module".to_string())).unwrap();
    assert_eq!(loaded, ErlangTerm::Atom("true".to_string()));
    
    // Delete it
    let result = LoadBif::delete_module_1(&ErlangTerm::Atom("test_module".to_string())).unwrap();
    assert_eq!(result, ErlangTerm::Atom("true".to_string()));
    
    // Verify it's gone
    let loaded_after = LoadBif::module_loaded_1(&ErlangTerm::Atom("test_module".to_string())).unwrap();
    assert_eq!(loaded_after, ErlangTerm::Atom("false".to_string()));
}

#[test]
fn test_load_bif_loaded_0_workflow() {
    use usecases_bifs::load::{LoadBif, LoadError};
    use usecases_bifs::op::ErlangTerm;
    use usecases_bifs::load::ModuleStatus;
    
    LoadBif::clear_all();
    
    // Register some modules
    LoadBif::register_module("module1", ModuleStatus::Loaded, false, false);
    LoadBif::register_module("module2", ModuleStatus::Loaded, false, false);
    
    // Get all loaded modules
    let loaded = LoadBif::loaded_0().unwrap();
    if let ErlangTerm::List(list) = loaded {
        assert!(list.len() >= 2);
        assert!(list.contains(&ErlangTerm::Atom("module1".to_string())));
        assert!(list.contains(&ErlangTerm::Atom("module2".to_string())));
    } else {
        panic!("Expected List");
    }
}

#[test]
fn test_load_bif_pre_loaded_0_workflow() {
    use usecases_bifs::load::{LoadBif, LoadError};
    use usecases_bifs::op::ErlangTerm;
    use usecases_bifs::load::ModuleStatus;
    
    LoadBif::clear_all();
    
    // Register pre-loaded modules
    LoadBif::register_module("preloaded1", ModuleStatus::PreLoaded, false, false);
    LoadBif::mark_preloaded("preloaded1");
    LoadBif::register_module("preloaded2", ModuleStatus::PreLoaded, false, false);
    LoadBif::mark_preloaded("preloaded2");
    
    // Get pre-loaded modules
    let preloaded = LoadBif::pre_loaded_0().unwrap();
    if let ErlangTerm::List(list) = preloaded {
        assert!(list.len() >= 2);
        assert!(list.contains(&ErlangTerm::Atom("preloaded1".to_string())));
        assert!(list.contains(&ErlangTerm::Atom("preloaded2".to_string())));
    } else {
        panic!("Expected List");
    }
}

#[test]
fn test_load_bif_finish_after_on_load_2_workflow() {
    use usecases_bifs::load::{LoadBif, LoadError};
    use usecases_bifs::op::ErlangTerm;
    use usecases_bifs::load::ModuleStatus;
    
    LoadBif::clear_all();
    
    // Register module with on_load pending
    LoadBif::register_module("onload_module", ModuleStatus::OnLoadPending, false, true);
    
    // Finish on_load with success
    let result = LoadBif::finish_after_on_load_2(
        &ErlangTerm::Atom("onload_module".to_string()),
        &ErlangTerm::Atom("true".to_string()),
    ).unwrap();
    assert_eq!(result, ErlangTerm::Atom("ok".to_string()));
    
    // Verify module is now loaded
    let loaded = LoadBif::module_loaded_1(&ErlangTerm::Atom("onload_module".to_string())).unwrap();
    assert_eq!(loaded, ErlangTerm::Atom("true".to_string()));
}

#[test]
fn test_load_bif_code_get_debug_info_1_workflow() {
    use usecases_bifs::load::{LoadBif, LoadError};
    use usecases_bifs::op::ErlangTerm;
    use usecases_bifs::load::ModuleStatus;
    use std::collections::HashMap;
    
    LoadBif::clear_all();
    
    // Register module
    LoadBif::register_module("debug_module", ModuleStatus::Loaded, false, false);
    
    // Set debug info
    let debug_info = ErlangTerm::Map({
        let mut map = HashMap::new();
        map.insert(
            ErlangTerm::Atom("source".to_string()),
            ErlangTerm::Atom("test.erl".to_string()),
        );
        map.insert(
            ErlangTerm::Atom("line".to_string()),
            ErlangTerm::Integer(42),
        );
        map
    });
    LoadBif::set_debug_info("debug_module", debug_info.clone());
    
    // Get debug info
    let result = LoadBif::code_get_debug_info_1(&ErlangTerm::Atom("debug_module".to_string())).unwrap();
    assert_eq!(result, debug_info);
}

#[test]
fn test_load_bif_delete_module_with_old_code() {
    use usecases_bifs::load::{LoadBif, LoadError};
    use usecases_bifs::op::ErlangTerm;
    use usecases_bifs::load::ModuleStatus;
    
    LoadBif::clear_all();
    
    // Register module with old code
    LoadBif::register_module("old_module", ModuleStatus::Loaded, true, false);
    
    // Try to delete (should fail)
    let result = LoadBif::delete_module_1(&ErlangTerm::Atom("old_module".to_string()));
    assert!(result.is_err());
    
    // Purge old code
    LoadBif::erts_internal_purge_module_2(
        &ErlangTerm::Atom("old_module".to_string()),
        &ErlangTerm::Atom("force".to_string()),
    ).unwrap();
    
    // Now delete should succeed
    let delete_result = LoadBif::delete_module_1(&ErlangTerm::Atom("old_module".to_string())).unwrap();
    assert_eq!(delete_result, ErlangTerm::Atom("true".to_string()));
}

#[test]
fn test_load_bif_finish_after_on_load_2_failure() {
    use usecases_bifs::load::{LoadBif, LoadError};
    use usecases_bifs::op::ErlangTerm;
    use usecases_bifs::load::ModuleStatus;
    
    LoadBif::clear_all();
    
    // Register module with on_load pending
    LoadBif::register_module("failed_module", ModuleStatus::OnLoadPending, false, true);
    
    // Finish on_load with failure
    let result = LoadBif::finish_after_on_load_2(
        &ErlangTerm::Atom("failed_module".to_string()),
        &ErlangTerm::Atom("false".to_string()),
    ).unwrap();
    assert_eq!(result, ErlangTerm::Atom("ok".to_string()));
    
    // Verify module is removed
    let loaded = LoadBif::module_loaded_1(&ErlangTerm::Atom("failed_module".to_string())).unwrap();
    assert_eq!(loaded, ErlangTerm::Atom("false".to_string()));
}

#[test]
fn test_load_bif_module_loaded_1_states() {
    use usecases_bifs::load::{LoadBif, LoadError};
    use usecases_bifs::op::ErlangTerm;
    use usecases_bifs::load::ModuleStatus;
    
    LoadBif::clear_all();
    
    // Test Loaded status
    LoadBif::register_module("loaded", ModuleStatus::Loaded, false, false);
    let result = LoadBif::module_loaded_1(&ErlangTerm::Atom("loaded".to_string())).unwrap();
    assert_eq!(result, ErlangTerm::Atom("true".to_string()));
    
    // Test PreLoaded status
    LoadBif::register_module("preloaded", ModuleStatus::PreLoaded, false, false);
    let result = LoadBif::module_loaded_1(&ErlangTerm::Atom("preloaded".to_string())).unwrap();
    assert_eq!(result, ErlangTerm::Atom("true".to_string()));
    
    // Test OnLoadPending status
    LoadBif::register_module("pending", ModuleStatus::OnLoadPending, false, true);
    let result = LoadBif::module_loaded_1(&ErlangTerm::Atom("pending".to_string())).unwrap();
    assert_eq!(result, ErlangTerm::Atom("false".to_string()));
}

#[test]
fn test_load_bif_code_get_debug_info_1_none() {
    use usecases_bifs::load::{LoadBif, LoadError};
    use usecases_bifs::op::ErlangTerm;
    use usecases_bifs::load::ModuleStatus;
    
    LoadBif::clear_all();
    
    // Register module without debug info
    LoadBif::register_module("no_debug", ModuleStatus::Loaded, false, false);
    
    // Get debug info (should return "none")
    let result = LoadBif::code_get_debug_info_1(&ErlangTerm::Atom("no_debug".to_string())).unwrap();
    assert_eq!(result, ErlangTerm::Atom("none".to_string()));
}

#[test]
fn test_load_bif_loaded_0_empty() {
    use usecases_bifs::load::{LoadBif, LoadError};
    
    LoadBif::clear_all();
    
    let result = LoadBif::loaded_0().unwrap();
    if let ErlangTerm::List(list) = result {
        assert_eq!(list.len(), 0);
    } else {
        panic!("Expected List");
    }
}

#[test]
fn test_load_bif_pre_loaded_0_empty() {
    use usecases_bifs::load::{LoadBif, LoadError};
    
    LoadBif::clear_all();
    
    let result = LoadBif::pre_loaded_0().unwrap();
    if let ErlangTerm::List(list) = result {
        assert_eq!(list.len(), 0);
    } else {
        panic!("Expected List");
    }
}

// ============================================================================
// Code Loading BIF Integration Tests - Full Implementation
// ============================================================================

#[test]
fn test_load_bif_prepare_loading_workflow() {
    use usecases_bifs::load::LoadBif;
    use usecases_bifs::op::ErlangTerm;
    
    LoadBif::clear_all();
    
    // Prepare code for loading
    let code = vec![0x00, 0x01, 0x02, 0x03];
    let prepared_ref = LoadBif::erts_internal_prepare_loading_2(
        &ErlangTerm::Atom("test_module".to_string()),
        &ErlangTerm::Binary(code),
    ).unwrap();
    
    // Check if it has on_load
    let has_on_load = LoadBif::has_prepared_code_on_load_1(&prepared_ref).unwrap();
    assert_eq!(has_on_load, ErlangTerm::Atom("false".to_string()));
    
    // Finish loading
    let result = LoadBif::finish_loading_1(&ErlangTerm::List(vec![prepared_ref])).unwrap();
    assert_eq!(result, ErlangTerm::Atom("ok".to_string()));
    
    // Verify module is loaded
    let loaded = LoadBif::module_loaded_1(&ErlangTerm::Atom("test_module".to_string())).unwrap();
    assert_eq!(loaded, ErlangTerm::Atom("true".to_string()));
}

#[test]
fn test_load_bif_check_old_code_workflow() {
    use usecases_bifs::load::LoadBif;
    use usecases_bifs::op::ErlangTerm;
    use usecases_bifs::load::ModuleStatus;
    
    LoadBif::clear_all();
    
    // Register module with old code
    LoadBif::register_module("old_module", ModuleStatus::Loaded, true, false);
    
    // Check for old code
    let has_old = LoadBif::check_old_code_1(&ErlangTerm::Atom("old_module".to_string())).unwrap();
    assert_eq!(has_old, ErlangTerm::Atom("true".to_string()));
    
    // Purge old code
    LoadBif::erts_internal_purge_module_2(
        &ErlangTerm::Atom("old_module".to_string()),
        &ErlangTerm::Atom("force".to_string()),
    ).unwrap();
    
    // Check again - should be false now
    let has_old_after = LoadBif::check_old_code_1(&ErlangTerm::Atom("old_module".to_string())).unwrap();
    assert_eq!(has_old_after, ErlangTerm::Atom("false".to_string()));
}

#[test]
fn test_load_bif_beamfile_md5_workflow() {
    use usecases_bifs::load::LoadBif;
    use usecases_bifs::op::ErlangTerm;
    
    LoadBif::clear_all();
    
    // Get MD5 of BEAM file
    let code = vec![0xBE, 0x41, 0x4D, 0x01, 0x00, 0x02, 0x03];
    let md5 = LoadBif::erts_internal_beamfile_module_md5_1(&ErlangTerm::Binary(code)).unwrap();
    
    // Should return a binary
    assert!(matches!(md5, ErlangTerm::Binary(_)));
}

#[test]
fn test_load_bif_beamfile_chunk_workflow() {
    use usecases_bifs::load::LoadBif;
    use usecases_bifs::op::ErlangTerm;
    
    LoadBif::clear_all();
    
    // Extract chunk from BEAM file
    let code = vec![0x00, 0x01, 0xBE, 0x41, 0x4D, 0x01, 0x00, 0x02, 0x03];
    let chunk_id = ErlangTerm::List(vec![
        ErlangTerm::Integer(0xBE),
        ErlangTerm::Integer(0x41),
        ErlangTerm::Integer(0x4D),
        ErlangTerm::Integer(0x01),
    ]);
    
    let chunk = LoadBif::erts_internal_beamfile_chunk_2(&ErlangTerm::Binary(code), &chunk_id).unwrap();
    
    // Should return a binary (chunk data) or undefined
    assert!(matches!(chunk, ErlangTerm::Binary(_) | ErlangTerm::Atom(_)));
}

#[test]
fn test_load_bif_prepare_loading_with_on_load() {
    use usecases_bifs::load::LoadBif;
    use usecases_bifs::op::ErlangTerm;
    
    LoadBif::clear_all();
    
    // Prepare code with on_load (code starting with 0xBE indicates on_load)
    let code = vec![0xBE, 0x41, 0x4D, 0x01, 0x00];
    let prepared_ref = LoadBif::erts_internal_prepare_loading_2(
        &ErlangTerm::Atom("onload_module".to_string()),
        &ErlangTerm::Binary(code),
    ).unwrap();
    
    // Check if it has on_load
    let has_on_load = LoadBif::has_prepared_code_on_load_1(&prepared_ref).unwrap();
    assert_eq!(has_on_load, ErlangTerm::Atom("true".to_string()));
    
    // Finish loading
    let result = LoadBif::finish_loading_1(&ErlangTerm::List(vec![prepared_ref])).unwrap();
    assert_eq!(result, ErlangTerm::Atom("ok".to_string()));
    
    // Verify module is in OnLoadPending state
    let loaded = LoadBif::module_loaded_1(&ErlangTerm::Atom("onload_module".to_string())).unwrap();
    assert_eq!(loaded, ErlangTerm::Atom("false".to_string())); // Not fully loaded yet
}

#[test]
fn test_load_bif_finish_loading_multiple_modules() {
    use usecases_bifs::load::LoadBif;
    use usecases_bifs::op::ErlangTerm;
    
    LoadBif::clear_all();
    
    // Prepare multiple modules
    let code1 = vec![0x00, 0x01, 0x02];
    let ref1 = LoadBif::erts_internal_prepare_loading_2(
        &ErlangTerm::Atom("module1".to_string()),
        &ErlangTerm::Binary(code1),
    ).unwrap();
    
    let code2 = vec![0x03, 0x04, 0x05];
    let ref2 = LoadBif::erts_internal_prepare_loading_2(
        &ErlangTerm::Atom("module2".to_string()),
        &ErlangTerm::Binary(code2),
    ).unwrap();
    
    // Finish loading both
    let result = LoadBif::finish_loading_1(&ErlangTerm::List(vec![ref1, ref2])).unwrap();
    assert_eq!(result, ErlangTerm::Atom("ok".to_string()));
    
    // Verify both are loaded
    let loaded1 = LoadBif::module_loaded_1(&ErlangTerm::Atom("module1".to_string())).unwrap();
    let loaded2 = LoadBif::module_loaded_1(&ErlangTerm::Atom("module2".to_string())).unwrap();
    assert_eq!(loaded1, ErlangTerm::Atom("true".to_string()));
    assert_eq!(loaded2, ErlangTerm::Atom("true".to_string()));
}

#[test]
fn test_load_bif_literal_area_collector_workflow() {
    use usecases_bifs::load::LoadBif;
    use usecases_bifs::op::ErlangTerm;
    
    LoadBif::clear_all();
    
    // Test literal area collector operations
    let result1 = LoadBif::erts_literal_area_collector_send_copy_request_3(
        &ErlangTerm::Pid(123),
        &ErlangTerm::Integer(456),
        &ErlangTerm::Atom("init".to_string()),
    ).unwrap();
    assert_eq!(result1, ErlangTerm::Atom("ok".to_string()));
    
    let result2 = LoadBif::erts_literal_area_collector_release_area_switch_0().unwrap();
    assert_eq!(result2, ErlangTerm::Atom("false".to_string()));
}

#[test]
fn test_load_bif_check_dirty_process_code() {
    use usecases_bifs::load::LoadBif;
    use usecases_bifs::op::ErlangTerm;
    
    LoadBif::clear_all();
    
    // Check dirty process code (simplified - always returns false)
    let result = LoadBif::erts_internal_check_dirty_process_code_2(
        &ErlangTerm::Pid(123),
        &ErlangTerm::Atom("test_module".to_string()),
    ).unwrap();
    
    assert_eq!(result, ErlangTerm::Atom("false".to_string()));
}

#[test]
fn test_load_bif_call_on_load_function_not_supported() {
    use usecases_bifs::load::{LoadBif, LoadError};
    use usecases_bifs::op::ErlangTerm;
    
    LoadBif::clear_all();
    
    // call_on_load_function is not supported as a BIF
    let result = LoadBif::call_on_load_function_1(&ErlangTerm::Atom("test_module".to_string()));
    assert!(result.is_err());
    if let Err(LoadError::NotSupported(_)) = result {
        // Expected
    } else {
        panic!("Expected NotSupported error");
    }
}
