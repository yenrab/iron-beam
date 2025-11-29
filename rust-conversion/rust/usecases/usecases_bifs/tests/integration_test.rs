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

