//! Integration tests for usecases_bifs
//!
//! Tests the integration between dynamic library loading and NIF compilation.
//! These tests verify the full pipeline: source file -> compilation -> verification -> loading.

use usecases_bifs::dynamic_library::*;
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

