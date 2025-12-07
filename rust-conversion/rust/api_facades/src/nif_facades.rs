//! NIF (Native Implemented Function) Facades
//!
//! Provides API facades for NIF functions called from Erlang.
//! Maintains exact C function signatures for compatibility.

// use adapters_nifs::{BufferNif, FileNif}; // TODO: Will be used when implementing NIF facades

/// NIF facade functions
/// These maintain exact C function signatures and call Rust NIF implementations

/// NIF load function facade
/// 
/// # Safety
/// This function maintains the C calling convention for Erlang compatibility
#[no_mangle]
pub unsafe extern "C" fn nif_load(
    _env: *mut std::ffi::c_void,
    _priv_data: *mut *mut std::ffi::c_void,
    _load_info: *mut std::ffi::c_void,
) -> i32 {
    // TODO: Implement NIF load facade
    // Calls adapters_nifs::NifEnv::load() or equivalent
    0 // Return 0 for success
}

/// NIF unload function facade
#[no_mangle]
pub unsafe extern "C" fn nif_unload(
    _env: *mut std::ffi::c_void,
    _priv_data: *mut std::ffi::c_void,
) -> i32 {
    // TODO: Implement NIF unload facade
    0
}

// TODO: Add remaining NIF facade functions
// Each facade maintains exact C signature and calls Rust implementation

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nif_load() {
        // Test that nif_load can be called with null pointers
        let result = unsafe { nif_load(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        ) };
        assert_eq!(result, 0, "nif_load should return 0 for success");
    }

    #[test]
    fn test_nif_unload() {
        // Test that nif_unload can be called with null pointers
        let result = unsafe { nif_unload(
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        ) };
        assert_eq!(result, 0, "nif_unload should return 0 for success");
    }

    #[test]
    fn test_nif_facades_signature_compatibility() {
        // Test that facades maintain correct C function signatures
        // These functions should be callable from C/Erlang code
        unsafe {
            let load_result = nif_load(
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            let unload_result = nif_unload(
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            assert_eq!(load_result, 0);
            assert_eq!(unload_result, 0);
        }
    }

    #[test]
    fn test_nif_load_multiple_calls() {
        // Test multiple calls to nif_load
        for _ in 0..10 {
            let result = unsafe { nif_load(
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            ) };
            assert_eq!(result, 0);
        }
    }

    #[test]
    fn test_nif_unload_after_load() {
        // Test calling unload after load
        unsafe {
            let load_result = nif_load(
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            assert_eq!(load_result, 0);
            
            let unload_result = nif_unload(
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            assert_eq!(unload_result, 0);
        }
    }
}

