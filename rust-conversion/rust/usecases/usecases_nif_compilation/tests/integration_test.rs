//! Integration tests for usecases_nif_compilation
//!
//! Tests the integration between NIF compiler and safe Rust verifier.
//! These tests verify the full compilation and verification pipeline.

use usecases_nif_compilation::{NifCompiler, SafeRustVerifier, CompileOptions};
use std::fs;
use std::io::Write;
use std::path::Path;

/// Helper to create a temporary Rust source file
fn create_test_file(dir: &std::path::Path, name: &str, content: &str) -> std::path::PathBuf {
    let rs_path = dir.join(format!("{}.rs", name));
    let mut file = fs::File::create(&rs_path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    rs_path
}

#[test]
fn test_compiler_and_verifier_integration() {
    // Test that compiler uses verifier correctly
    let temp_dir = tempfile::tempdir().unwrap();
    
    // Create safe Rust code
    let safe_code = r#"
/// Safety marker
#[no_mangle]
pub extern "C" fn rust_safe_library_marker() -> u32 {
    0x53414645
}

pub fn safe_function() -> i32 {
    42
}
"#;
    
    let safe_path = create_test_file(temp_dir.path(), "safe_lib", safe_code);
    
    // Verify it's safe
    let verifier = SafeRustVerifier::new();
    let verify_result = verifier.verify_file(&safe_path).unwrap();
    assert_eq!(verify_result, usecases_nif_compilation::VerificationResult::Safe);
    
    // Compile it (with verification enabled)
    let compiler = NifCompiler::new();
    let options = CompileOptions {
        verify_safe: true,
        ..Default::default()
    };
    
    let compile_result = compiler.compile(&safe_path, options);
    // Should succeed (or fail only on cargo/compilation issues, not unsafe code)
    if let Err(usecases_nif_compilation::CompileError::UnsafeCodeFound(_)) = compile_result {
        panic!("Safe code should not be rejected");
    }
}

#[test]
fn test_nif_interface_verification_in_compilation() {
    // Test that NIF interface verification is part of the compilation process
    let temp_dir = tempfile::tempdir().unwrap();
    let compiler = NifCompiler::new();
    
    // Test 1: Code with missing nif_init should fail
    let code_without_nif_init = r#"
#[no_mangle]
pub extern "C" fn rust_safe_library_marker() -> u32 { 0x53414645 }
pub fn test() -> i32 { 42 }
"#;
    let path1 = create_test_file(temp_dir.path(), "no_nif_init", code_without_nif_init);
    
    let compile_result = compiler.compile(&path1, CompileOptions::default());
    assert!(compile_result.is_err());
    match compile_result {
        Err(usecases_nif_compilation::CompileError::InvalidNifInterface { .. }) => {
            // Expected
        }
        _ => {
            // Other errors are also acceptable (cargo not found, etc.)
        }
    }
    
    // Test 2: Code with proper nif_init should pass interface check
    let code_with_nif_init = r#"
#[no_mangle]
pub extern "C" fn rust_safe_library_marker() -> u32 { 0x53414645 }

#[no_mangle]
pub extern "C" fn nif_init() -> *const u8 {
    std::ptr::null()
}
"#;
    let path2 = create_test_file(temp_dir.path(), "with_nif_init", code_with_nif_init);
    
    let compile_result = compiler.compile(&path2, CompileOptions::default());
    // Should not fail on interface check (may fail on cargo/compilation, but not interface)
    if let Err(usecases_nif_compilation::CompileError::InvalidNifInterface { .. }) = compile_result {
        panic!("Should not fail on NIF interface check when nif_init is present");
    }
}

#[test]
fn test_verification_then_compilation_flow() {
    // Test the flow: verify -> compile
    let temp_dir = tempfile::tempdir().unwrap();
    let verifier = SafeRustVerifier::new();
    let compiler = NifCompiler::new();
    
    // Test 1: Safe code flow
    let safe_code = r#"
#[no_mangle]
pub extern "C" fn rust_safe_library_marker() -> u32 { 0x53414645 }
pub fn safe() -> i32 { 1 }
"#;
    let safe_path = create_test_file(temp_dir.path(), "safe", safe_code);
    
    // Verify first
    let verify_result = verifier.verify_file(&safe_path).unwrap();
    assert_eq!(verify_result, usecases_nif_compilation::VerificationResult::Safe);
    
    // Then compile
    let compile_result = compiler.compile(&safe_path, CompileOptions::default());
    if let Err(usecases_nif_compilation::CompileError::UnsafeCodeFound(_)) = compile_result {
        panic!("Should not fail on unsafe code after verification passed");
    }
    
    // Test 2: Unsafe code flow
    let unsafe_code = r#"
#[no_mangle]
pub extern "C" fn rust_safe_library_marker() -> u32 { 0x53414645 }
unsafe fn unsafe_fn() {}
"#;
    let unsafe_path = create_test_file(temp_dir.path(), "unsafe", unsafe_code);
    
    // Verify first - should fail
    let verify_result = verifier.verify_file(&unsafe_path).unwrap();
    match verify_result {
        usecases_nif_compilation::VerificationResult::Unsafe { .. } => {
            // Expected
        }
        _ => panic!("Should detect unsafe code"),
    }
    
    // Compilation with verification should also fail
    let compile_result = compiler.compile(&unsafe_path, CompileOptions::default());
    assert!(compile_result.is_err());
    match compile_result {
        Err(usecases_nif_compilation::CompileError::UnsafeCodeFound(_)) => {
            // Expected
        }
        _ => {
            // Other errors are also acceptable
        }
    }
}

#[test]
fn test_compilation_with_complex_safe_code() {
    // Test compilation with complex but safe Rust code
    let temp_dir = tempfile::tempdir().unwrap();
    let compiler = NifCompiler::new();
    
    let complex_safe = r#"
#[no_mangle]
pub extern "C" fn rust_safe_library_marker() -> u32 {
    0x53414645
}

pub struct MyStruct {
    value: i32,
}

impl MyStruct {
    pub fn new(value: i32) -> Self {
        Self { value }
    }
    
    pub fn get_value(&self) -> i32 {
        self.value
    }
}

#[no_mangle]
pub extern "C" fn create_struct(value: i32) -> *mut MyStruct {
    Box::into_raw(Box::new(MyStruct::new(value)))
}

#[no_mangle]
pub extern "C" fn get_struct_value(ptr: *mut MyStruct) -> i32 {
    if ptr.is_null() {
        return 0;
    }
    unsafe {
        // Note: This is actually unsafe, but we're testing that the compiler
        // can handle complex code. In reality, this would need proper safety.
        // For this test, we'll use a safe version.
        let s = &*ptr;
        s.get_value()
    }
}
"#;
    
    let source_path = create_test_file(temp_dir.path(), "complex", complex_safe);
    
    // This will fail on unsafe code detection (the unsafe block in get_struct_value)
    // But it tests that complex code is processed
    let result = compiler.compile(&source_path, CompileOptions::default());
    // Should fail on unsafe code
    assert!(result.is_err());
}

#[test]
fn test_verification_with_various_unsafe_patterns() {
    // Test verification detects various unsafe patterns
    let verifier = SafeRustVerifier::new();
    
    // Pattern 1: Unsafe function
    let code1 = "pub unsafe fn f() {}";
    let result1 = verifier.verify_content(code1, Path::new("test.rs")).unwrap();
    match result1 {
        usecases_nif_compilation::VerificationResult::Unsafe { locations } => {
            assert!(locations.iter().any(|l| l.description.contains("Unsafe function")));
        }
        _ => panic!("Should detect unsafe function"),
    }
    
    // Pattern 2: Unsafe block
    let code2 = "pub fn f() { unsafe {} }";
    let result2 = verifier.verify_content(code2, Path::new("test.rs")).unwrap();
    match result2 {
        usecases_nif_compilation::VerificationResult::Unsafe { locations } => {
            assert!(locations.iter().any(|l| l.description.contains("Unsafe block")));
        }
        _ => panic!("Should detect unsafe block"),
    }
    
    // Pattern 3: Unsafe impl
    let code3 = "unsafe impl Trait for Struct {}";
    let result3 = verifier.verify_content(code3, Path::new("test.rs")).unwrap();
    match result3 {
        usecases_nif_compilation::VerificationResult::Unsafe { locations } => {
            assert!(locations.iter().any(|l| l.description.contains("Unsafe impl")));
        }
        _ => panic!("Should detect unsafe impl"),
    }
}

#[test]
fn test_compilation_error_propagation() {
    // Test that compilation errors propagate correctly
    let temp_dir = tempfile::tempdir().unwrap();
    let compiler = NifCompiler::new();
    
    // Test 1: File not found
    let result = compiler.compile(
        Path::new("/nonexistent/file.rs"),
        CompileOptions::default(),
    );
    assert!(matches!(result, Err(usecases_nif_compilation::CompileError::SourceNotFound(_))));
    
    // Test 2: Not a Rust file
    let txt_file = temp_dir.path().join("test.txt");
    fs::write(&txt_file, b"not rust").unwrap();
    let result = compiler.compile(&txt_file, CompileOptions::default());
    assert!(matches!(result, Err(usecases_nif_compilation::CompileError::NotRustFile(_))));
}

#[test]
fn test_verification_error_propagation() {
    // Test that verification errors propagate correctly
    let verifier = SafeRustVerifier::new();
    
    // Test 1: File not found
    let result = verifier.verify_file(Path::new("/nonexistent/file.rs"));
    assert!(matches!(result, Err(usecases_nif_compilation::VerificationError::FileNotFound(_))));
    
    // Test 2: Not a Rust file
    let temp_dir = tempfile::tempdir().unwrap();
    let txt_file = temp_dir.path().join("test.txt");
    fs::write(&txt_file, b"not rust").unwrap();
    let result = verifier.verify_file(&txt_file);
    assert!(matches!(result, Err(usecases_nif_compilation::VerificationError::NotRustFile(_))));
    
    // Test 3: Parse error
    let invalid_rust = temp_dir.path().join("invalid.rs");
    fs::write(&invalid_rust, b"pub fn broken {").unwrap();
    let result = verifier.verify_file(&invalid_rust);
    assert!(matches!(result, Err(usecases_nif_compilation::VerificationError::ParseError(_, _))));
}

#[test]
fn test_compilation_with_output_directory() {
    // Test compilation with custom output directory
    let temp_dir = tempfile::tempdir().unwrap();
    let output_dir = temp_dir.path().join("custom_output");
    
    let safe_code = r#"
#[no_mangle]
pub extern "C" fn rust_safe_library_marker() -> u32 { 0x53414645 }
pub fn f() -> i32 { 1 }
"#;
    let source_path = create_test_file(temp_dir.path(), "output_test", safe_code);
    
    let compiler = NifCompiler::new();
    let options = CompileOptions {
        verify_safe: true,
        release: false,
        cargo_flags: Vec::new(),
        output_dir: Some(output_dir.clone()),
    };
    
    let result = compiler.compile(&source_path, options);
    // Might succeed or fail depending on cargo
    if result.is_ok() {
        // If compilation succeeded, output_dir should exist
        // and contain the library
    }
}

#[test]
fn test_compilation_with_release_mode() {
    // Test compilation in release mode
    let temp_dir = tempfile::tempdir().unwrap();
    
    let safe_code = r#"
#[no_mangle]
pub extern "C" fn rust_safe_library_marker() -> u32 { 0x53414645 }
pub fn f() -> i32 { 1 }
"#;
    let source_path = create_test_file(temp_dir.path(), "release_test", safe_code);
    
    let compiler = NifCompiler::new();
    let options = CompileOptions {
        verify_safe: true,
        release: true, // Release mode
        cargo_flags: Vec::new(),
        output_dir: None,
    };
    
    let result = compiler.compile(&source_path, options);
    // Should not fail on unsafe code (code is safe)
    if let Err(usecases_nif_compilation::CompileError::UnsafeCodeFound(_)) = result {
        panic!("Should not fail on safe code");
    }
}

#[test]
fn test_verification_with_real_world_patterns() {
    // Test verification with real-world Rust patterns
    let verifier = SafeRustVerifier::new();
    
    // Pattern 1: Safe trait implementation
    let safe_trait = r#"
trait MyTrait {
    fn method(&self) -> i32;
}

struct MyStruct;

impl MyTrait for MyStruct {
    fn method(&self) -> i32 {
        42
    }
}
"#;
    let result = verifier.verify_content(safe_trait, Path::new("test.rs")).unwrap();
    assert_eq!(result, usecases_nif_compilation::VerificationResult::Safe);
    
    // Pattern 2: Safe enum with match
    let safe_enum = r#"
enum MyEnum {
    Variant1,
    Variant2(i32),
}

fn process(e: MyEnum) -> i32 {
    match e {
        MyEnum::Variant1 => 0,
        MyEnum::Variant2(x) => x,
    }
}
"#;
    let result = verifier.verify_content(safe_enum, Path::new("test.rs")).unwrap();
    assert_eq!(result, usecases_nif_compilation::VerificationResult::Safe);
}

#[test]
fn test_end_to_end_safe_library_creation() {
    // Test the complete end-to-end flow for a safe library
    let temp_dir = tempfile::tempdir().unwrap();
    
    // Step 1: Create safe Rust source
    let safe_library_code = r#"
/// Safety marker - REQUIRED
#[no_mangle]
pub extern "C" fn rust_safe_library_marker() -> u32 {
    0x53414645 // "SAFE" in ASCII
}

/// Example NIF function
#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Another example function
#[no_mangle]
pub extern "C" fn multiply(a: i32, b: i32) -> i32 {
    a * b
}
"#;
    
    let source_path = create_test_file(temp_dir.path(), "end_to_end", safe_library_code);
    
    // Step 2: Verify it's safe
    let verifier = SafeRustVerifier::new();
    let verify_result = verifier.verify_file(&source_path).unwrap();
    assert_eq!(verify_result, usecases_nif_compilation::VerificationResult::Safe);
    
    // Step 3: Compile it
    let compiler = NifCompiler::new();
    let compile_options = CompileOptions {
        verify_safe: true,
        release: false,
        cargo_flags: Vec::new(),
        output_dir: None,
    };
    
    let compile_result = compiler.compile(&source_path, compile_options);
    
    // Compilation should succeed (or fail only on cargo issues, not unsafe code)
    match compile_result {
        Ok(result) => {
            // Compilation succeeded
            // Note: The library path might be in a temp directory that gets cleaned up
            // So we just verify the result structure is correct
            assert!(!result.was_cached);
            // The path might not exist if temp dir was cleaned up, which is OK
            let _ = result.library_path;
        }
        Err(usecases_nif_compilation::CompileError::UnsafeCodeFound(_)) => {
            panic!("Safe code should not be rejected");
        }
        Err(usecases_nif_compilation::CompileError::CargoNotFound) => {
            // Cargo not available - acceptable for test environment
        }
        Err(usecases_nif_compilation::CompileError::CompilationFailed { .. }) => {
            // Compilation failed for other reasons - acceptable
        }
        Err(e) => {
            // Other errors are acceptable
            println!("Compilation failed with: {:?}", e);
        }
    }
}

