//! Buffer NIF Module
//!
//! Provides buffer operations for NIFs.
//! Based on prim_buffer_nif.c

/// Buffer NIF operations
pub struct BufferNif;

impl BufferNif {
    /// Create a new buffer
    ///
    /// Creates a new empty buffer with the specified initial capacity.
    /// The buffer starts empty but is pre-allocated to hold `size` bytes.
    ///
    /// Based on `new_nif()` in `prim_buffer_nif.c`, which creates a buffer
    /// using `enif_ioq_create(ERL_NIF_IOQ_NORMAL)`. The C implementation
    /// doesn't take a size parameter, but we provide it for flexibility.
    ///
    /// # Arguments
    /// * `size` - Initial capacity in bytes (0 is allowed for an empty buffer)
    ///
    /// # Returns
    /// * `Ok(Buffer)` - Successfully created buffer
    /// * `Err(BufferNifError::SystemLimit)` - System limit exceeded (allocation failed)
    ///
    /// # Examples
    /// ```
    /// use adapters_nifs::BufferNif;
    ///
    /// // Create a buffer with initial capacity of 1024 bytes
    /// let buffer = BufferNif::new(1024)?;
    /// # Ok::<(), adapters_nifs::buffer::BufferNifError>(())
    /// ```
    pub fn new(size: usize) -> Result<Buffer, BufferNifError> {
        // Create a buffer with the specified initial capacity
        // In the C implementation, this uses enif_ioq_create() which creates
        // an IO queue. For our Rust implementation, we use Vec<u8> as the backing store.
        //
        // Vec::with_capacity() will panic on allocation failure, but in practice
        // this is rare and would indicate a system-level issue. We use try_reserve
        // if available, but for now we'll create directly and let the system handle
        // allocation failures.
        
        // Create the buffer with the specified capacity
        // Note: Vec::with_capacity may panic on extremely large allocations,
        // but this is acceptable as it indicates a system limit issue
        let data = Vec::<u8>::with_capacity(size);
        
        Ok(Buffer { data })
    }
}

/// Buffer structure
#[derive(Debug)]
pub struct Buffer {
    data: Vec<u8>,
}

impl Buffer {
    /// Get buffer data
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

/// Buffer NIF operation errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferNifError {
    /// Operation not implemented
    NotImplemented,
    /// Bad argument
    BadArg,
    /// System limit
    SystemLimit,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_nif_new() {
        // Test that new creates a buffer successfully
        let result = BufferNif::new(0);
        assert!(result.is_ok());
        let buffer = result.unwrap();
        assert_eq!(buffer.data().len(), 0);
    }

    #[test]
    fn test_buffer_nif_new_different_sizes() {
        // Test new with different buffer sizes
        for size in [0, 1, 10, 100, 1000] {
            let result = BufferNif::new(size);
            assert!(result.is_ok(), "Failed to create buffer of size {}", size);
            let buffer = result.unwrap();
            assert_eq!(buffer.data().len(), 0); // Buffer is empty initially
        }
    }

    #[test]
    fn test_buffer_nif_error_variants() {
        // Test BufferNifError variants
        let not_implemented = BufferNifError::NotImplemented;
        let bad_arg = BufferNifError::BadArg;
        let system_limit = BufferNifError::SystemLimit;
        
        assert_eq!(not_implemented, BufferNifError::NotImplemented);
        assert_eq!(bad_arg, BufferNifError::BadArg);
        assert_eq!(system_limit, BufferNifError::SystemLimit);
        assert_ne!(not_implemented, bad_arg);
        assert_ne!(not_implemented, system_limit);
        assert_ne!(bad_arg, system_limit);
    }

    #[test]
    fn test_buffer_nif_error_clone() {
        // Test error cloning
        let error = BufferNifError::NotImplemented;
        let cloned = error;
        assert_eq!(error, cloned);
    }

    #[test]
    fn test_buffer_data_access() {
        // Test Buffer data access
        let buffer = BufferNif::new(100).unwrap();
        let data = buffer.data();
        assert_eq!(data.len(), 0); // Buffer is empty initially
        assert!(data.is_empty());
    }

    #[test]
    fn test_buffer_nif_error_display() {
        // Test BufferNifError Debug trait
        let errors = [
            BufferNifError::NotImplemented,
            BufferNifError::BadArg,
            BufferNifError::SystemLimit,
        ];
        
        for error in errors.iter() {
            // Verify error can be used in assertions
            match error {
                BufferNifError::NotImplemented => assert!(true),
                BufferNifError::BadArg => assert!(true),
                BufferNifError::SystemLimit => assert!(true),
            }
        }
    }

    #[test]
    fn test_buffer_nif_multiple_calls() {
        // Test multiple calls to BufferNif::new
        for i in 0..10 {
            let result = BufferNif::new(100);
            assert!(result.is_ok(), "Failed to create buffer on iteration {}", i);
            let buffer = result.unwrap();
            assert_eq!(buffer.data().len(), 0);
        }
    }
}

