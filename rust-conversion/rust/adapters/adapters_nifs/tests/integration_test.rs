//! Integration tests for adapters_nifs crate
//!
//! These tests verify that NIF adapter functions work correctly
//! and test end-to-end workflows for NIF loading, file operations, and buffer operations.

use adapters_nifs::*;
use adapters_nifs::nif_common::NifEnv;
use adapters_nifs::buffer::BufferNifError;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_nif_env_creation() {
    let env = NifEnv::new();
    // Should not panic
    let _ = env;
}

#[test]
fn test_buffer_nif_new() {
    // Test creating buffers of various sizes
    let sizes = vec![0, 1, 10, 100, 1000, 10000];
    
    for size in sizes {
        let result = BufferNif::new(size);
        assert!(result.is_ok(), "Failed to create buffer of size {}", size);
        let buffer = result.unwrap();
        assert_eq!(buffer.data().len(), 0); // Buffer is empty initially
    }
}

#[test]
fn test_buffer_nif_data_access() {
    let buffer = BufferNif::new(100).unwrap();
    let data = buffer.data();
    assert_eq!(data.len(), 0);
    assert!(data.is_empty());
}

#[test]
fn test_buffer_nif_error_variants() {
    // Test BufferNifError enum variants
    let errors = vec![
        BufferNifError::NotImplemented,
        BufferNifError::BadArg,
        BufferNifError::SystemLimit,
    ];
    
    for error in errors {
        let _ = format!("{:?}", error);
    }
}

#[test]
fn test_file_nif_create_and_open() {
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("test_nif_file_integration");
    
    // Clean up if exists
    let _ = fs::remove_file(&test_file);
    
    // Create file
    let mut handle = FileNif::create(&test_file).unwrap();
    
    // Write data
    let data = b"Hello, NIF!";
    let written = handle.write(data).unwrap();
    assert_eq!(written, data.len());
    
    // Open and read back
    let mut handle = FileNif::open(&test_file).unwrap();
    let mut buf = vec![0u8; data.len()];
    let read = handle.read(&mut buf).unwrap();
    assert_eq!(read, data.len());
    assert_eq!(&buf[..read], data);
    
    // Cleanup
    let _ = fs::remove_file(&test_file);
}

#[test]
fn test_file_nif_read_partial() {
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("test_nif_file_partial");
    
    // Clean up if exists
    let _ = fs::remove_file(&test_file);
    
    // Create and write data
    let data = b"Hello, World!";
    let mut handle = FileNif::create(&test_file).unwrap();
    handle.write(data).unwrap();
    
    // Read partial data
    let mut handle = FileNif::open(&test_file).unwrap();
    let mut buf = vec![0u8; 5];
    let read = handle.read(&mut buf).unwrap();
    assert_eq!(read, 5);
    assert_eq!(&buf[..read], b"Hello");
    
    // Cleanup
    let _ = fs::remove_file(&test_file);
}

#[test]
fn test_file_nif_error_cases() {
    // Try to open non-existent file
    let non_existent = PathBuf::from("/nonexistent/path/file.txt");
    let result = FileNif::open(&non_existent);
    assert!(result.is_err());
}

#[test]
fn test_file_nif_multiple_operations() {
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("test_nif_file_multiple");
    
    // Clean up if exists
    let _ = fs::remove_file(&test_file);
    
    // Create, write, close, open, read
    {
        let mut handle = FileNif::create(&test_file).unwrap();
        handle.write(b"First write").unwrap();
    }
    
    {
        let mut handle = FileNif::open(&test_file).unwrap();
        let mut buf = vec![0u8; 11];
        let read = handle.read(&mut buf).unwrap();
        assert_eq!(&buf[..read], b"First write");
    }
    
    // Cleanup
    let _ = fs::remove_file(&test_file);
}

#[test]
fn test_nif_registry_get_instance() {
    let registry1 = NifRegistry::get_instance();
    let registry2 = NifRegistry::get_instance();
    
    // Should return the same instance (singleton)
    assert!(std::ptr::eq(registry1, registry2));
}

#[test]
fn test_nif_registry_get_library() {
    let registry = NifRegistry::get_instance();
    
    // Get non-existent library
    let library = registry.get_library("nonexistent");
    assert!(library.is_none());
}

#[test]
fn test_nif_registry_register_function() {
    let registry = NifRegistry::get_instance();
    
    // Create a mock function pointer
    let func_ptr: NifFunctionPtr = std::ptr::null();
    
    // Create function metadata
    let function = NifFunction {
        pointer: func_ptr,
        module: "test_module".to_string(),
        name: "test_function".to_string(),
        arity: 1,
        is_dirty: false,
    };
    
    // Register function
    registry.register_function(function.clone());
    
    // Get function by pointer
    let found = registry.get_function(func_ptr);
    assert!(found.is_some());
    let found_func = found.unwrap();
    assert_eq!(found_func.module, "test_module");
    assert_eq!(found_func.name, "test_function");
    assert_eq!(found_func.arity, 1);
}

#[test]
fn test_nif_registry_get_function_nonexistent() {
    let registry = NifRegistry::get_instance();
    
    // Get non-existent function
    let func_ptr: NifFunctionPtr = unsafe { std::ptr::null::<u8>().add(999) };
    let found = registry.get_function(func_ptr);
    assert!(found.is_none());
}

#[test]
fn test_nif_function_metadata() {
    let metadata = FunctionMetadata {
        name: "test_function".to_string(),
        arity: 2,
        symbol_name: "test_function_2".to_string(),
        flags: 0,
    };
    
    assert_eq!(metadata.name, "test_function");
    assert_eq!(metadata.arity, 2);
    assert_eq!(metadata.symbol_name, "test_function_2");
    assert_eq!(metadata.flags, 0);
}

#[test]
fn test_nif_function_metadata_clone() {
    let metadata1 = FunctionMetadata {
        name: "test".to_string(),
        arity: 1,
        symbol_name: "test_1".to_string(),
        flags: 0,
    };
    
    let metadata2 = metadata1.clone();
    assert_eq!(metadata1.name, metadata2.name);
    assert_eq!(metadata1.arity, metadata2.arity);
}

#[test]
fn test_rust_nif_metadata() {
    let metadata = RustNifMetadata {
        module_name: "test_module".to_string(),
        version: (2, 0),
        min_erts_version: Some("12.0".to_string()),
        functions: vec![
            FunctionMetadata {
                name: "func1".to_string(),
                arity: 1,
                symbol_name: "func1_1".to_string(),
                flags: 0,
            },
        ],
    };
    
    assert_eq!(metadata.module_name, "test_module");
    assert_eq!(metadata.version, (2, 0));
    assert_eq!(metadata.functions.len(), 1);
}

#[test]
fn test_rust_nif_metadata_clone() {
    let metadata1 = RustNifMetadata {
        module_name: "test".to_string(),
        version: (1, 0),
        min_erts_version: None,
        functions: vec![],
    };
    
    let metadata2 = metadata1.clone();
    assert_eq!(metadata1.module_name, metadata2.module_name);
    assert_eq!(metadata1.version, metadata2.version);
}

#[test]
fn test_nif_load_error_variants() {
    // Test NifLoadError enum variants
    // Check actual variants from the enum
    let errors = vec![
        NifLoadError::LibraryNotFound(PathBuf::from("test.so")),
        NifLoadError::LoadFailed("error".to_string()),
        NifLoadError::InvalidFormat("invalid".to_string()),
        NifLoadError::EntryPointNotFound("entry_point".to_string()),
        NifLoadError::ModuleAlreadyLoaded("test".to_string()),
    ];
    
    for error in errors {
        let _ = format!("{:?}", error);
    }
}

#[test]
fn test_nif_unload_error_variants() {
    // Test NifUnloadError enum variants
    let errors = vec![
        NifUnloadError::LibraryNotFound("test.so".to_string()),
        NifUnloadError::ProcessesStillUsing,
        NifUnloadError::UnloadFailed("error".to_string()),
    ];
    
    for error in errors {
        let _ = format!("{:?}", error);
    }
}

#[test]
fn test_nif_error_variants() {
    // Test NifError enum variants
    use std::path::PathBuf;
    let errors = vec![
        NifError::InvalidPointer,
        NifError::ProcessNotFound,
        NifError::AssociationError("test error".to_string()),
    ];
    
    for error in errors {
        let _ = format!("{:?}", error);
    }
}

#[test]
fn test_nif_loader_static_methods() {
    // NifLoader is a struct with only static methods
    // Test that we can call static methods
    let _ = NifLoader;
}

#[test]
fn test_nif_registry_multiple_functions() {
    let registry = NifRegistry::get_instance();
    
    let func1: NifFunctionPtr = std::ptr::null();
    let func2: NifFunctionPtr = unsafe { std::ptr::null::<u8>().add(1) };
    
    let function1 = NifFunction {
        pointer: func1,
        module: "module1".to_string(),
        name: "func1".to_string(),
        arity: 1,
        is_dirty: false,
    };
    
    let function2 = NifFunction {
        pointer: func2,
        module: "module1".to_string(),
        name: "func2".to_string(),
        arity: 2,
        is_dirty: false,
    };
    
    registry.register_function(function1);
    registry.register_function(function2);
    
    // Get both functions
    assert!(registry.get_function(func1).is_some());
    assert!(registry.get_function(func2).is_some());
}

#[test]
fn test_file_nif_large_write() {
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("test_nif_file_large");
    
    // Clean up if exists
    let _ = fs::remove_file(&test_file);
    
    // Write large data
    let large_data = vec![0x42u8; 10000];
    let mut handle = FileNif::create(&test_file).unwrap();
    let written = handle.write(&large_data).unwrap();
    assert_eq!(written, large_data.len());
    
    // Read back
    let mut handle = FileNif::open(&test_file).unwrap();
    let mut buf = vec![0u8; large_data.len()];
    let read = handle.read(&mut buf).unwrap();
    assert_eq!(read, large_data.len());
    assert_eq!(buf, large_data);
    
    // Cleanup
    let _ = fs::remove_file(&test_file);
}

#[test]
fn test_buffer_nif_error_display() {
    let error1 = BufferNifError::NotImplemented;
    let error2 = BufferNifError::BadArg;
    let error3 = BufferNifError::SystemLimit;
    
    let _ = format!("{:?}", error1);
    let _ = format!("{:?}", error2);
    let _ = format!("{:?}", error3);
}

#[test]
fn test_buffer_nif_error_eq() {
    assert_eq!(BufferNifError::BadArg, BufferNifError::BadArg);
    assert_ne!(BufferNifError::BadArg, BufferNifError::NotImplemented);
}
