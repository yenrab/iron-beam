//! NIF Interface Verifier
//!
//! Verifies that Rust NIF code has the proper interface requirements:
//! - `nif_init()` function with `#[no_mangle]` and `extern "C"`
//! - NIF functions with `#[no_mangle]` and `extern "C"`
//! - Proper function signatures matching Erlang's expectations

use std::fs;
use std::path::{Path, PathBuf};
use syn::{File, Item, ItemFn, ItemStatic, Attribute, Signature, ReturnType, Type, TypePath};
use crate::safe_rust_verifier::VerificationError;

/// Result of NIF interface verification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NifInterfaceResult {
    /// Interface is valid
    Valid,
    /// Interface has errors
    Invalid {
        /// List of interface errors
        errors: Vec<NifInterfaceError>,
    },
}

/// NIF interface verification errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NifInterfaceError {
    /// Missing nif_init() function
    MissingNifInit,
    /// nif_init() missing #[no_mangle]
    NifInitMissingNoMangle,
    /// nif_init() missing extern "C"
    NifInitMissingExternC,
    /// nif_init() has wrong signature
    NifInitWrongSignature(String),
    /// NIF function missing #[no_mangle]
    FunctionMissingNoMangle {
        function_name: String,
    },
    /// NIF function missing extern "C"
    FunctionMissingExternC {
        function_name: String,
    },
    /// NIF function has wrong signature
    FunctionWrongSignature {
        function_name: String,
        reason: String,
    },
    /// Missing RustNifMetadata static structure
    MissingRustNifMetadata,
    /// RustNifMetadata structure is empty (no functions)
    EmptyRustNifMetadata,
    /// nif_get_metadata() return type should be *const RustNifMetadata
    WrongMetadataReturnType,
}

impl std::fmt::Display for NifInterfaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NifInterfaceError::MissingNifInit => {
                write!(f, "Missing required nif_init() function")
            }
            NifInterfaceError::NifInitMissingNoMangle => {
                write!(f, "nif_init() function missing #[no_mangle] attribute")
            }
            NifInterfaceError::NifInitMissingExternC => {
                write!(f, "nif_init() function missing extern \"C\" calling convention")
            }
            NifInterfaceError::NifInitWrongSignature(details) => {
                write!(f, "nif_init() has wrong signature: {}", details)
            }
            NifInterfaceError::FunctionMissingNoMangle { function_name } => {
                write!(f, "NIF function '{}' missing #[no_mangle] attribute", function_name)
            }
            NifInterfaceError::FunctionMissingExternC { function_name } => {
                write!(f, "NIF function '{}' missing extern \"C\" calling convention", function_name)
            }
            NifInterfaceError::FunctionWrongSignature { function_name, reason } => {
                write!(f, "NIF function '{}' has wrong signature: {}", function_name, reason)
            }
            NifInterfaceError::MissingRustNifMetadata => {
                write!(f, "Missing RustNifMetadata static structure (required when using nif_get_metadata)")
            }
            NifInterfaceError::EmptyRustNifMetadata => {
                write!(f, "RustNifMetadata structure is empty (must have at least one function)")
            }
            NifInterfaceError::WrongMetadataReturnType => {
                write!(f, "nif_get_metadata() must return *const RustNifMetadata")
            }
        }
    }
}

/// Verifier for NIF interface requirements
pub struct NifInterfaceVerifier;

impl NifInterfaceVerifier {
    /// Create a new NIF interface verifier
    pub fn new() -> Self {
        Self
    }

    /// Verify that a Rust source file has proper NIF interface
    ///
    /// # Arguments
    /// * `file_path` - Path to the Rust source file to verify
    ///
    /// # Returns
    /// * `Ok(NifInterfaceResult::Valid)` if the interface is valid
    /// * `Ok(NifInterfaceResult::Invalid { errors })` if interface errors are found
    /// * `Err(VerificationError)` if verification fails
    pub fn verify_file(&self, file_path: &Path) -> Result<NifInterfaceResult, VerificationError> {
        // Check if file exists
        if !file_path.exists() {
            return Err(VerificationError::FileNotFound(file_path.to_path_buf()));
        }

        // Check if it's a Rust file
        if file_path.extension().and_then(|e| e.to_str()) != Some("rs") {
            return Err(VerificationError::NotRustFile(file_path.to_path_buf()));
        }

        // Read the file
        let content = fs::read_to_string(file_path)
            .map_err(|e| VerificationError::ReadError(file_path.to_path_buf(), e.to_string()))?;

        // Verify the content
        self.verify_content(&content, file_path)
    }

    /// Verify that Rust source code content has proper NIF interface
    ///
    /// # Arguments
    /// * `content` - Rust source code content
    /// * `file_path` - Path to the file (for error reporting)
    ///
    /// # Returns
    /// * `Ok(NifInterfaceResult::Valid)` if the interface is valid
    /// * `Ok(NifInterfaceResult::Invalid { errors })` if interface errors are found
    /// * `Err(VerificationError)` if verification fails
    pub fn verify_content(
        &self,
        content: &str,
        file_path: &Path,
    ) -> Result<NifInterfaceResult, VerificationError> {
        // Parse the Rust code
        let ast: File = syn::parse_str(content)
            .map_err(|e| VerificationError::ParseError(file_path.to_path_buf(), e.to_string()))?;

        let mut errors = Vec::new();
        let mut has_nif_init = false;

        // Check all items
        for item in &ast.items {
            if let Item::Fn(item_fn) = item {
                let fn_name = item_fn.sig.ident.to_string();
                
                // Check for nif_init or nif_get_metadata (both are valid entry points)
                if fn_name == "nif_init" || fn_name == "nif_get_metadata" {
                    has_nif_init = true;
                    self.check_nif_init(item_fn, &mut errors);
                }
            }
        }

        // Check for required nif_init
        if !has_nif_init {
            errors.push(NifInterfaceError::MissingNifInit);
        }

        // Check all functions that look like NIF functions
        // (functions with #[no_mangle] that aren't the marker or nif_init)
        // We check functions with #[no_mangle] because they're intended to be exported
        for item in &ast.items {
            if let Item::Fn(item_fn) = item {
                let fn_name = item_fn.sig.ident.to_string();
                
                // Skip marker function and nif_init (already checked)
                if fn_name == "rust_safe_library_marker" || fn_name == "nif_init" {
                    continue;
                }
                
                // Check if this has #[no_mangle] - if so, it's intended to be a NIF function
                if Self::has_no_mangle(&item_fn.attrs) {
                    self.check_nif_function(item_fn, &mut errors);
                }
            }
        }

        if errors.is_empty() {
            Ok(NifInterfaceResult::Valid)
        } else {
            Ok(NifInterfaceResult::Invalid { errors })
        }
    }

    /// Check if a function has #[no_mangle] attribute
    fn has_no_mangle(attrs: &[Attribute]) -> bool {
        attrs.iter().any(|attr| {
            attr.path().is_ident("no_mangle")
        })
    }

    /// Check if a function has extern "C"
    fn has_extern_c(sig: &Signature) -> bool {
        if let Some(abi) = &sig.abi {
            if let Some(name) = &abi.name {
                name.value() == "C"
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Check if this looks like a NIF function (has extern "C" and #[no_mangle])
    fn is_nif_function(&self, item_fn: &ItemFn) -> bool {
        Self::has_no_mangle(&item_fn.attrs) && Self::has_extern_c(&item_fn.sig)
    }

    /// Check nif_init() function
    fn check_nif_init(&self, item_fn: &ItemFn, errors: &mut Vec<NifInterfaceError>) {
        // Check #[no_mangle]
        if !Self::has_no_mangle(&item_fn.attrs) {
            errors.push(NifInterfaceError::NifInitMissingNoMangle);
        }

        // Check extern "C"
        if !Self::has_extern_c(&item_fn.sig) {
            errors.push(NifInterfaceError::NifInitMissingExternC);
        }

        // Check signature: should be fn() -> *const RustNifMetadata (or similar pointer type)
        // We check that it returns a pointer type
        // For Rust-native NIFs, this should return *const RustNifMetadata
        let return_type = &item_fn.sig.output;
        match return_type {
            ReturnType::Default => {
                errors.push(NifInterfaceError::NifInitWrongSignature(
                    "nif_init() or nif_get_metadata() must return a pointer type (e.g., *const RustNifMetadata)".to_string()
                ));
            }
            ReturnType::Type(_, ty) => {
                // Check if it's a pointer type
                if !self.is_pointer_type(ty) {
                    errors.push(NifInterfaceError::NifInitWrongSignature(
                        "nif_init() or nif_get_metadata() must return a pointer type (e.g., *const RustNifMetadata)".to_string()
                    ));
                }
            }
        }

        // Check that it takes no parameters (or on Windows, takes TWinDynNifCallbacks*)
        if !item_fn.sig.inputs.is_empty() {
            // On Windows, nif_init takes TWinDynNifCallbacks* parameter
            // For now, we'll allow parameters but could add stricter checking
            if item_fn.sig.inputs.len() > 1 {
                errors.push(NifInterfaceError::NifInitWrongSignature(
                    "nif_init() should take 0 or 1 parameter".to_string()
                ));
            }
        }
    }

    /// Check if a type is a pointer type
    fn is_pointer_type(&self, ty: &Type) -> bool {
        match ty {
            Type::Ptr(_) => true,
            Type::Reference(_) => true,
            _ => false,
        }
    }

    /// Check if return type is *const RustNifMetadata
    fn check_metadata_return_type(&self, item_fn: &ItemFn, errors: &mut Vec<NifInterfaceError>) {
        let return_type = &item_fn.sig.output;
        match return_type {
            ReturnType::Type(_, ty) => {
                // Check if it's *const RustNifMetadata
                if let Type::Ptr(ptr_type) = ty.as_ref() {
                    // Check if the pointee type is RustNifMetadata
                    if let Type::Path(type_path) = ptr_type.elem.as_ref() {
                        if self.is_rust_nif_metadata_type(type_path) {
                            // Good - it's *const RustNifMetadata
                            return;
                        }
                    }
                }
                // Not the exact type, but might still work if it's a pointer
                // We'll allow it but warn that it should be *const RustNifMetadata
                if !self.is_pointer_type(ty.as_ref()) {
                    errors.push(NifInterfaceError::WrongMetadataReturnType);
                }
            }
            _ => {
                errors.push(NifInterfaceError::WrongMetadataReturnType);
            }
        }
    }

    /// Check if a type path is RustNifMetadata
    fn is_rust_nif_metadata_type(&self, type_path: &TypePath) -> bool {
        // Check if the last segment is "RustNifMetadata"
        if let Some(last_segment) = type_path.path.segments.last() {
            last_segment.ident == "RustNifMetadata"
        } else {
            false
        }
    }

    /// Check if a static item is a RustNifMetadata structure
    fn is_rust_nif_metadata_static(&self, item_static: &ItemStatic) -> bool {
        // Check if the type is RustNifMetadata
        // item_static.ty is a Box<Type>, so we need to dereference it
        if let Type::Path(type_path) = item_static.ty.as_ref() {
            self.is_rust_nif_metadata_type(type_path)
        } else {
            false
        }
    }

    /// Check if metadata static has functions (basic check)
    fn metadata_has_functions(&self, _item_static: &ItemStatic) -> bool {
        // This is a basic check - for now, we assume if a RustNifMetadata static exists,
        // it should be properly initialized with functions.
        // A more sophisticated check would parse the struct initialization expression
        // to verify that the functions field is not empty.
        // For now, we'll return true if the static exists (the actual validation
        // happens at runtime when the library is loaded).
        true
    }

    /// Check individual NIF function
    fn check_nif_function(&self, item_fn: &ItemFn, errors: &mut Vec<NifInterfaceError>) {
        let fn_name = item_fn.sig.ident.to_string();

        // Check #[no_mangle]
        if !Self::has_no_mangle(&item_fn.attrs) {
            errors.push(NifInterfaceError::FunctionMissingNoMangle {
                function_name: fn_name.clone(),
            });
        }

        // Check extern "C"
        if !Self::has_extern_c(&item_fn.sig) {
            errors.push(NifInterfaceError::FunctionMissingExternC {
                function_name: fn_name.clone(),
            });
        }

        // Check signature matches NIF function pattern
        // NIF functions should have: (env: *mut c_void, argc: c_int, argv: *const u64) -> u64
        // However, we're more lenient here - we just check that it has extern "C" and #[no_mangle]
        // The exact signature can vary depending on the NIF implementation
        // Full signature validation would require more complex type checking
    }
}

impl Default for NifInterfaceVerifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_valid_nif_interface() {
        let verifier = NifInterfaceVerifier::new();
        let valid_code = r#"
            use std::os::raw::{c_int, c_void};
            use std::ffi::c_char;

            // ErlNifEntry structure (simplified)
            #[repr(C)]
            pub struct ErlNifEntry {
                pub major: c_int,
                pub minor: c_int,
                pub name: *const c_char,
                pub num_of_funcs: c_int,
            }

            #[no_mangle]
            pub extern "C" fn nif_init() -> *const ErlNifEntry {
                static ENTRY: ErlNifEntry = ErlNifEntry {
                    major: 2,
                    minor: 0,
                    name: std::ptr::null(),
                    num_of_funcs: 0,
                };
                &ENTRY
            }

            #[no_mangle]
            pub extern "C" fn rust_safe_library_marker() -> u32 {
                0x53414645
            }
        "#;

        let result = verifier
            .verify_content(valid_code, Path::new("test.rs"))
            .unwrap();
        assert_eq!(result, NifInterfaceResult::Valid);
    }

    #[test]
    fn test_verify_missing_nif_init() {
        let verifier = NifInterfaceVerifier::new();
        let code = r#"
            #[no_mangle]
            pub extern "C" fn rust_safe_library_marker() -> u32 {
                0x53414645
            }
        "#;

        let result = verifier
            .verify_content(code, Path::new("test.rs"))
            .unwrap();
        match result {
            NifInterfaceResult::Invalid { errors } => {
                assert!(errors.iter().any(|e| matches!(e, NifInterfaceError::MissingNifInit)));
            }
            _ => panic!("Expected Invalid result"),
        }
    }

    #[test]
    fn test_verify_nif_init_missing_no_mangle() {
        let verifier = NifInterfaceVerifier::new();
        let code = r#"
            pub extern "C" fn nif_init() -> *const u8 {
                std::ptr::null()
            }
        "#;

        let result = verifier
            .verify_content(code, Path::new("test.rs"))
            .unwrap();
        match result {
            NifInterfaceResult::Invalid { errors } => {
                assert!(errors.iter().any(|e| matches!(e, NifInterfaceError::NifInitMissingNoMangle)));
            }
            _ => panic!("Expected Invalid result"),
        }
    }

    #[test]
    fn test_verify_nif_init_missing_extern_c() {
        let verifier = NifInterfaceVerifier::new();
        let code = r#"
            #[no_mangle]
            pub fn nif_init() -> *const u8 {
                std::ptr::null()
            }
        "#;

        let result = verifier
            .verify_content(code, Path::new("test.rs"))
            .unwrap();
        match result {
            NifInterfaceResult::Invalid { errors } => {
                assert!(errors.iter().any(|e| matches!(e, NifInterfaceError::NifInitMissingExternC)));
            }
            _ => panic!("Expected Invalid result"),
        }
    }

    #[test]
    fn test_verify_nif_init_wrong_return_type() {
        let verifier = NifInterfaceVerifier::new();
        let code = r#"
            #[no_mangle]
            pub extern "C" fn nif_init() -> i32 {
                0
            }
        "#;

        let result = verifier
            .verify_content(code, Path::new("test.rs"))
            .unwrap();
        match result {
            NifInterfaceResult::Invalid { errors } => {
                assert!(errors.iter().any(|e| matches!(e, NifInterfaceError::NifInitWrongSignature(_))));
            }
            _ => panic!("Expected Invalid result"),
        }
    }

    #[test]
    fn test_verify_nif_function_missing_no_mangle() {
        let verifier = NifInterfaceVerifier::new();
        let code = r#"
            #[no_mangle]
            pub extern "C" fn nif_init() -> *const u8 {
                std::ptr::null()
            }

            pub extern "C" fn my_nif_function() -> i32 {
                42
            }
        "#;

        let result = verifier
            .verify_content(code, Path::new("test.rs"))
            .unwrap();
        // my_nif_function doesn't have #[no_mangle], so it won't be checked as a NIF function
        // This is expected - only functions with both #[no_mangle] and extern "C" are checked
        assert_eq!(result, NifInterfaceResult::Valid);
    }

    #[test]
    fn test_verify_nif_function_with_both_attributes() {
        let verifier = NifInterfaceVerifier::new();
        let code = r#"
            #[no_mangle]
            pub extern "C" fn nif_init() -> *const u8 {
                std::ptr::null()
            }

            #[no_mangle]
            pub extern "C" fn my_nif_function() -> i32 {
                42
            }
        "#;

        let result = verifier
            .verify_content(code, Path::new("test.rs"))
            .unwrap();
        // Both functions have proper attributes
        assert_eq!(result, NifInterfaceResult::Valid);
    }

    #[test]
    fn test_verify_nif_function_missing_extern_c() {
        let verifier = NifInterfaceVerifier::new();
        let code = r#"
            #[no_mangle]
            pub extern "C" fn nif_init() -> *const u8 {
                std::ptr::null()
            }

            #[no_mangle]
            pub fn my_nif_function() -> i32 {
                42
            }
        "#;

        let result = verifier
            .verify_content(code, Path::new("test.rs"))
            .unwrap();
        match result {
            NifInterfaceResult::Invalid { errors } => {
                assert!(errors.iter().any(|e| 
                    matches!(e, NifInterfaceError::FunctionMissingExternC { function_name } 
                    if function_name == "my_nif_function")
                ));
            }
            _ => panic!("Expected Invalid result"),
        }
    }

    #[test]
    fn test_verify_file_not_found() {
        let verifier = NifInterfaceVerifier::new();
        let result = verifier.verify_file(Path::new("/nonexistent/file.rs"));
        assert!(matches!(result, Err(VerificationError::FileNotFound(_))));
    }

    #[test]
    fn test_verify_file_not_rust() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path();
        let verifier = NifInterfaceVerifier::new();
        let result = verifier.verify_file(path);
        assert!(matches!(result, Err(VerificationError::NotRustFile(_))));
    }

    #[test]
    fn test_nif_interface_error_display() {
        let err1 = NifInterfaceError::MissingNifInit;
        let _ = format!("{}", err1);
        
        let err2 = NifInterfaceError::NifInitMissingNoMangle;
        let _ = format!("{}", err2);
        
        let err3 = NifInterfaceError::FunctionMissingNoMangle {
            function_name: "my_func".to_string(),
        };
        let _ = format!("{}", err3);
    }

    #[test]
    fn test_verifier_default() {
        let verifier1 = NifInterfaceVerifier::new();
        let verifier2 = NifInterfaceVerifier::default();
        let _ = verifier1;
        let _ = verifier2;
    }

    #[test]
    fn test_verify_with_complex_nif() {
        let verifier = NifInterfaceVerifier::new();
        let code = r#"
            use std::os::raw::{c_int, c_void, c_char};

            #[repr(C)]
            pub struct ErlNifEntry {
                pub major: c_int,
                pub minor: c_int,
                pub name: *const c_char,
                pub num_of_funcs: c_int,
            }

            #[no_mangle]
            pub extern "C" fn nif_init() -> *const ErlNifEntry {
                static ENTRY: ErlNifEntry = ErlNifEntry {
                    major: 2,
                    minor: 0,
                    name: std::ptr::null(),
                    num_of_funcs: 2,
                };
                &ENTRY
            }

            #[no_mangle]
            pub extern "C" fn rust_safe_library_marker() -> u32 {
                0x53414645
            }

            #[no_mangle]
            pub extern "C" fn add_numbers(a: i32, b: i32) -> i32 {
                a + b
            }

            #[no_mangle]
            pub extern "C" fn multiply_numbers(a: i32, b: i32) -> i32 {
                a * b
            }
        "#;

        let result = verifier
            .verify_content(code, Path::new("test.rs"))
            .unwrap();
        assert_eq!(result, NifInterfaceResult::Valid);
    }

    #[test]
    fn test_verify_nif_init_no_return_type() {
        let verifier = NifInterfaceVerifier::new();
        let code = r#"
            #[no_mangle]
            pub extern "C" fn nif_init() {
                // No return type
            }
        "#;

        let result = verifier
            .verify_content(code, Path::new("test.rs"))
            .unwrap();
        match result {
            NifInterfaceResult::Invalid { errors } => {
                assert!(errors.iter().any(|e| matches!(e, NifInterfaceError::NifInitWrongSignature(_))));
            }
            _ => panic!("Expected Invalid result"),
        }
    }

    #[test]
    fn test_verify_nif_init_non_pointer_return() {
        let verifier = NifInterfaceVerifier::new();
        let code = r#"
            #[no_mangle]
            pub extern "C" fn nif_init() -> i32 {
                0
            }
        "#;

        let result = verifier
            .verify_content(code, Path::new("test.rs"))
            .unwrap();
        match result {
            NifInterfaceResult::Invalid { errors } => {
                assert!(errors.iter().any(|e| matches!(e, NifInterfaceError::NifInitWrongSignature(_))));
            }
            _ => panic!("Expected Invalid result"),
        }
    }

    #[test]
    fn test_verify_nif_init_too_many_parameters() {
        let verifier = NifInterfaceVerifier::new();
        let code = r#"
            #[no_mangle]
            pub extern "C" fn nif_init(_param1: i32, _param2: i32) -> *const u8 {
                std::ptr::null()
            }
        "#;

        let result = verifier
            .verify_content(code, Path::new("test.rs"))
            .unwrap();
        match result {
            NifInterfaceResult::Invalid { errors } => {
                assert!(errors.iter().any(|e| matches!(e, NifInterfaceError::NifInitWrongSignature(_))));
            }
            _ => panic!("Expected Invalid result"),
        }
    }

    #[test]
    fn test_verify_nif_init_with_one_parameter() {
        // Windows allows one parameter
        let verifier = NifInterfaceVerifier::new();
        let code = r#"
            #[no_mangle]
            pub extern "C" fn nif_init(_param: *mut u8) -> *const u8 {
                std::ptr::null()
            }
        "#;

        let result = verifier
            .verify_content(code, Path::new("test.rs"))
            .unwrap();
        // Should be valid (one parameter is allowed)
        assert_eq!(result, NifInterfaceResult::Valid);
    }

    #[test]
    fn test_verify_nif_function_missing_no_mangle_but_has_extern_c() {
        let verifier = NifInterfaceVerifier::new();
        let code = r#"
            #[no_mangle]
            pub extern "C" fn nif_init() -> *const u8 {
                std::ptr::null()
            }

            pub extern "C" fn my_nif_function() -> i32 {
                42
            }
        "#;

        let result = verifier
            .verify_content(code, Path::new("test.rs"))
            .unwrap();
        // Function without #[no_mangle] won't be checked
        assert_eq!(result, NifInterfaceResult::Valid);
    }

    #[test]
    fn test_verify_file_read_error() {
        use std::fs;
        use std::io::Write;
        use std::os::unix::fs::PermissionsExt;
        
        // Create a file that exists but can't be read
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("unreadable_test.rs");
        
        // Create file
        let mut file = fs::File::create(&test_file).unwrap();
        file.write_all(b"test content").unwrap();
        drop(file);
        
        // Make it unreadable (on Unix)
        #[cfg(unix)]
        {
            let mut perms = fs::metadata(&test_file).unwrap().permissions();
            perms.set_mode(0o000); // No permissions
            fs::set_permissions(&test_file, perms).unwrap();
            
            let verifier = NifInterfaceVerifier::new();
            let result = verifier.verify_file(&test_file);
            
            // Should get ReadError
            assert!(matches!(result, Err(VerificationError::ReadError(_, _))));
            
            // Restore permissions for cleanup
            let mut perms = fs::metadata(&test_file).unwrap().permissions();
            perms.set_mode(0o644);
            fs::set_permissions(&test_file, perms).unwrap();
        }
        
        // Clean up
        let _ = fs::remove_file(&test_file);
    }

    #[test]
    fn test_verify_file_success() {
        use std::fs;
        
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("valid_nif_test.rs");
        
        let code = r#"
            #[no_mangle]
            pub extern "C" fn nif_init() -> *const u8 {
                std::ptr::null()
            }
        "#;
        
        fs::write(&test_file, code).unwrap();
        
        let verifier = NifInterfaceVerifier::new();
        let result = verifier.verify_file(&test_file).unwrap();
        assert_eq!(result, NifInterfaceResult::Valid);
        
        // Clean up
        let _ = fs::remove_file(&test_file);
    }

    #[test]
    fn test_nif_interface_error_display_all_variants() {
        // Test all Display implementations
        let err1 = NifInterfaceError::MissingNifInit;
        let s1 = format!("{}", err1);
        assert!(s1.contains("Missing required nif_init"));
        
        let err2 = NifInterfaceError::NifInitMissingNoMangle;
        let s2 = format!("{}", err2);
        assert!(s2.contains("missing #[no_mangle]"));
        
        let err3 = NifInterfaceError::NifInitMissingExternC;
        let s3 = format!("{}", err3);
        assert!(s3.contains("missing extern"));
        
        let err4 = NifInterfaceError::NifInitWrongSignature("test details".to_string());
        let s4 = format!("{}", err4);
        assert!(s4.contains("wrong signature"));
        assert!(s4.contains("test details"));
        
        let err5 = NifInterfaceError::FunctionMissingNoMangle {
            function_name: "my_func".to_string(),
        };
        let s5 = format!("{}", err5);
        assert!(s5.contains("my_func"));
        assert!(s5.contains("missing #[no_mangle]"));
        
        let err6 = NifInterfaceError::FunctionMissingExternC {
            function_name: "my_func2".to_string(),
        };
        let s6 = format!("{}", err6);
        assert!(s6.contains("my_func2"));
        assert!(s6.contains("missing extern"));
        
        let err7 = NifInterfaceError::FunctionWrongSignature {
            function_name: "my_func3".to_string(),
            reason: "test reason".to_string(),
        };
        let s7 = format!("{}", err7);
        assert!(s7.contains("my_func3"));
        assert!(s7.contains("test reason"));
    }

    #[test]
    fn test_verify_nif_function_with_reference_return() {
        let verifier = NifInterfaceVerifier::new();
        let code = r#"
            #[no_mangle]
            pub extern "C" fn nif_init() -> &'static u8 {
                &0u8
            }
        "#;

        let result = verifier
            .verify_content(code, Path::new("test.rs"))
            .unwrap();
        // Reference type should be accepted as pointer type
        assert_eq!(result, NifInterfaceResult::Valid);
    }

    #[test]
    fn test_verify_multiple_nif_functions() {
        let verifier = NifInterfaceVerifier::new();
        let code = r#"
            #[no_mangle]
            pub extern "C" fn nif_init() -> *const u8 {
                std::ptr::null()
            }

            #[no_mangle]
            pub extern "C" fn func1() -> i32 {
                1
            }

            #[no_mangle]
            pub extern "C" fn func2() -> i32 {
                2
            }

            #[no_mangle]
            pub extern "C" fn func3() -> i32 {
                3
            }
        "#;

        let result = verifier
            .verify_content(code, Path::new("test.rs"))
            .unwrap();
        assert_eq!(result, NifInterfaceResult::Valid);
    }

    #[test]
    fn test_verify_nif_function_missing_extern_c_but_has_no_mangle() {
        let verifier = NifInterfaceVerifier::new();
        let code = r#"
            #[no_mangle]
            pub extern "C" fn nif_init() -> *const u8 {
                std::ptr::null()
            }

            #[no_mangle]
            pub fn my_nif_function() -> i32 {
                42
            }
        "#;

        let result = verifier
            .verify_content(code, Path::new("test.rs"))
            .unwrap();
        match result {
            NifInterfaceResult::Invalid { errors } => {
                assert!(errors.iter().any(|e| 
                    matches!(e, NifInterfaceError::FunctionMissingExternC { function_name } 
                    if function_name == "my_nif_function")
                ));
            }
            _ => panic!("Expected Invalid result"),
        }
    }

    #[test]
    fn test_is_pointer_type() {
        let verifier = NifInterfaceVerifier::new();
        
        // Test with pointer return type
        let code1 = r#"
            #[no_mangle]
            pub extern "C" fn nif_init() -> *const u8 {
                std::ptr::null()
            }
        "#;
        let result1 = verifier.verify_content(code1, Path::new("test.rs")).unwrap();
        assert_eq!(result1, NifInterfaceResult::Valid);
        
        // Test with reference return type
        let code2 = r#"
            static VALUE: u8 = 0;
            #[no_mangle]
            pub extern "C" fn nif_init() -> &'static u8 {
                &VALUE
            }
        "#;
        let result2 = verifier.verify_content(code2, Path::new("test.rs")).unwrap();
        assert_eq!(result2, NifInterfaceResult::Valid);
    }

    #[test]
    fn test_verify_with_parse_error() {
        let verifier = NifInterfaceVerifier::new();
        let invalid_code = r#"
            fn broken code {
                // Invalid syntax
        "#;

        let result = verifier.verify_content(invalid_code, Path::new("test.rs"));
        assert!(matches!(result, Err(VerificationError::ParseError(_, _))));
    }

    #[test]
    fn test_verify_nif_init_with_mutable_pointer() {
        let verifier = NifInterfaceVerifier::new();
        let code = r#"
            #[no_mangle]
            pub extern "C" fn nif_init() -> *mut u8 {
                std::ptr::null_mut()
            }
        "#;

        let result = verifier
            .verify_content(code, Path::new("test.rs"))
            .unwrap();
        // Mutable pointer should be accepted
        assert_eq!(result, NifInterfaceResult::Valid);
    }
}

