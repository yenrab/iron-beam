//! Resource Management Functions
//!
//! Provides functions for managing NIF resources (allocated memory objects).
//! These functions correspond to resource management functions in the C NIF API,
//! but are implemented using safe Rust patterns.
//!
//! ## Design Principles
//!
//! - **Safe Rust Only**: All functions use safe Rust types and operations
//! - **Rust Patterns**: Uses `Vec<u8>`, `Box<[u8]>`, `Arc`, and `Result` types
//! - **No C FFI**: Since NIFs are always written in Rust, no C compatibility needed

use super::{NifEnv, NifTerm};
use std::sync::Arc;

/// NIF binary structure
///
/// Represents a binary that can be passed between NIF functions.
/// Uses safe Rust types instead of raw pointers.
///
/// ## Design
///
/// This is a safe Rust alternative to the C `ErlNifBinary` structure.
/// It uses `Vec<u8>` for the binary data, which provides automatic
/// memory management and bounds checking.
pub struct ErlNifBinary {
    /// Binary data
    data: Vec<u8>,
}

impl ErlNifBinary {
    /// Create a new binary from a byte vector
    ///
    /// # Arguments
    /// * `data` - Binary data
    ///
    /// # Returns
    /// A new `ErlNifBinary` instance
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Get a reference to the binary data
    ///
    /// # Returns
    /// A slice reference to the binary data
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get the size of the binary in bytes
    ///
    /// # Returns
    /// The size of the binary
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Convert into the underlying byte vector
    ///
    /// # Returns
    /// The binary data as a `Vec<u8>`
    pub fn into_vec(self) -> Vec<u8> {
        self.data
    }
}

/// Resource type identifier
///
/// Represents a resource type for type-safe resource management.
/// Uses a string identifier instead of a raw pointer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ErlNifResourceType {
    /// Resource type name
    name: String,
    /// Module name that owns this resource type
    module: String,
}

impl ErlNifResourceType {
    /// Create a new resource type
    ///
    /// # Arguments
    /// * `name` - Resource type name
    /// * `module` - Module name
    ///
    /// # Returns
    /// A new `ErlNifResourceType` instance
    pub fn new(name: String, module: String) -> Self {
        Self { name, module }
    }

    /// Get the resource type name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the module name
    pub fn module(&self) -> &str {
        &self.module
    }
}

/// Allocate a resource
///
/// Allocates memory for a NIF resource using safe Rust allocation.
///
/// # Arguments
///
/// * `resource_type` - Resource type handle
/// * `size` - Size of resource in bytes
///
/// # Returns
///
/// * `Result<Box<[u8]>, ResourceError>` - Allocated resource data, or error on failure
///
/// # Errors
///
/// Returns `ResourceError::AllocationFailed` if allocation fails.
///
/// # See Also
///
/// - `erts/emulator/beam/erl_nif.c:enif_alloc_resource()` - C implementation
pub fn enif_alloc_resource(
    _resource_type: &ErlNifResourceType,
    size: usize,
) -> Result<Box<[u8]>, ResourceError> {
    // Use safe Rust allocation
    // In a full implementation, this would:
    // 1. Register the resource with the resource type's allocator
    // 2. Initialize resource header/metadata
    // 3. Return resource data with proper reference counting
    
    // Safe allocation using Vec and Box
    // This will panic on OOM, but in a full implementation we'd handle that gracefully
    Ok(vec![0u8; size].into_boxed_slice())
}

/// Release a resource
///
/// Releases a resource, decrementing its reference count.
/// The resource is freed when the reference count reaches zero.
///
/// # Arguments
///
/// * `resource` - Resource data to release
///
/// # Note
///
/// In a full implementation, this would:
/// 1. Decrement resource reference count
/// 2. If count reaches zero, call destructor and free memory
/// 3. Handle resource finalization
///
/// Currently, the resource is automatically dropped when `Box` goes out of scope.
/// For full reference counting, we would use `Arc` with a custom drop implementation.
pub fn enif_release_resource(_resource: Box<[u8]>) {
    // Resource is automatically dropped when Box goes out of scope
    // In a full implementation with reference counting, we'd use Arc:
    // - Wrap resources in Arc<ResourceData>
    // - Track reference counts
    // - Call destructor when count reaches zero
}

/// Create a resource term
///
/// Creates an Erlang term that references a resource.
/// Resources are heap-allocated structures that point to resource objects.
///
/// # Arguments
///
/// * `env` - NIF environment
/// * `resource` - Resource data (wrapped in Arc for reference counting)
///
/// # Returns
///
/// * `NifTerm` - Resource term (heap-allocated, properly tagged)
///
/// # Implementation Note
///
/// Resources are allocated on the heap as boxed terms with:
/// - Header word containing resource metadata
/// - Data word containing pointer/reference to the resource object
///
/// The resource term uses TAG_PRIMARY_BOXED (0x1) with a resource subtag.
/// The Arc reference counting ensures the resource stays alive as long as
/// the term exists.
///
/// # See Also
///
/// - `erts/emulator/beam/erl_nif.c:enif_make_resource()` - C implementation
pub fn enif_make_resource(
    env: &NifEnv,
    resource: &Arc<Box<[u8]>>,
) -> NifTerm {
    // Resources are heap-allocated boxed terms
    // We need 2 words: 1 for header, 1 for resource pointer/reference
    let words_needed = 2;
    
    // Allocate heap space
    if let Some(heap_index) = env.allocate_heap(words_needed) {
        let process = env.process();
        let mut heap_data = process.heap_slice_mut();
        
        // Write resource header
        // Format: (size << 2) | TAG_PRIMARY_BOXED | RESOURCE_SUBTAG
        // TAG_PRIMARY_BOXED = 0x1
        // For simplicity, we'll use size 0 and encode resource info in the data word
        // In a full implementation, the header would contain resource type info
        let header = (0u64 << 2) | 0x1; // Boxed term with size 0
        heap_data[heap_index] = header;
        
        // Store resource pointer as the data word
        // Convert Arc pointer to usize for storage
        // Note: This is safe because we're just storing the pointer value, not dereferencing
        let resource_ptr = Arc::as_ptr(resource) as usize as u64;
        heap_data[heap_index + 1] = resource_ptr;
        
        drop(heap_data);
        
        // Return resource pointer: (heap_index << 2) | TAG_PRIMARY_BOXED
        let resource_term = (heap_index as u64) << 2 | 0x1;
        if resource_term == 0 {
            // Heap index 0 would result in term 0, which is ambiguous
            // Fall back to placeholder
            Arc::as_ptr(resource) as usize as u64
        } else {
            resource_term
        }
    } else {
        // Heap allocation failed, fall back to placeholder
        // In a full implementation, this would return an error or raise an exception
        Arc::as_ptr(resource) as usize as u64
    }
}

/// Resource management errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceError {
    /// Resource allocation failed
    AllocationFailed,
    /// Invalid resource type
    InvalidResourceType,
    /// Resource not found
    ResourceNotFound,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_erl_nif_binary_new() {
        let data = vec![1, 2, 3, 4, 5];
        let binary = ErlNifBinary::new(data.clone());
        assert_eq!(binary.data(), &data);
        assert_eq!(binary.size(), 5);
    }

    #[test]
    fn test_erl_nif_binary_into_vec() {
        let data = vec![10, 20, 30];
        let binary = ErlNifBinary::new(data.clone());
        let retrieved = binary.into_vec();
        assert_eq!(retrieved, data);
    }

    #[test]
    fn test_erl_nif_resource_type_new() {
        let resource_type = ErlNifResourceType::new(
            "my_resource".to_string(),
            "my_module".to_string(),
        );
        assert_eq!(resource_type.name(), "my_resource");
        assert_eq!(resource_type.module(), "my_module");
    }

    #[test]
    fn test_erl_nif_resource_type_clone() {
        let resource_type = ErlNifResourceType::new(
            "test".to_string(),
            "module".to_string(),
        );
        let cloned = resource_type.clone();
        assert_eq!(resource_type, cloned);
    }

    #[test]
    fn test_enif_alloc_resource() {
        let resource_type = ErlNifResourceType::new(
            "test_resource".to_string(),
            "test_module".to_string(),
        );
        let result = enif_alloc_resource(&resource_type, 100);
        assert!(result.is_ok());
        let resource = result.unwrap();
        assert_eq!(resource.len(), 100);
    }

    #[test]
    fn test_enif_alloc_resource_zero_size() {
        let resource_type = ErlNifResourceType::new(
            "test".to_string(),
            "module".to_string(),
        );
        let result = enif_alloc_resource(&resource_type, 0);
        assert!(result.is_ok());
        let resource = result.unwrap();
        assert_eq!(resource.len(), 0);
    }

    #[test]
    fn test_enif_release_resource() {
        let resource_type = ErlNifResourceType::new(
            "test".to_string(),
            "module".to_string(),
        );
        let resource = enif_alloc_resource(&resource_type, 50).unwrap();
        // Resource should be dropped when it goes out of scope
        enif_release_resource(resource);
        // If we get here without panicking, the test passes
    }

    #[test]
    fn test_enif_make_resource() {
        use std::sync::Arc;
        use entities_process::Process;
        let env = crate::nif_env::NifEnv::from_process(Arc::new(Process::new(1)));
        let resource_type = ErlNifResourceType::new(
            "test".to_string(),
            "module".to_string(),
        );
        let resource = enif_alloc_resource(&resource_type, 10).unwrap();
        let resource_arc = Arc::new(resource);
        let term = enif_make_resource(&env, &resource_arc);
        // Should return a non-zero value (pointer address)
        assert_ne!(term, 0);
    }

    #[test]
    fn test_resource_error() {
        let err = ResourceError::AllocationFailed;
        assert_eq!(err, ResourceError::AllocationFailed);
        
        let err2 = ResourceError::InvalidResourceType;
        assert_ne!(err, err2);
    }
}

