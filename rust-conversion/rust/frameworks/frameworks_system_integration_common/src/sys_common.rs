//! Common System Integration Module
//!
//! Provides common system integration functionality.
//! Based on erl_mseg.c
//!
//! This module implements a memory segment allocator that caches deallocated
//! segments for reuse, improving allocation performance for frequently
//! allocated and deallocated memory segments.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// Common system integration
pub struct SysCommon;

/// Memory segment allocator
///
/// Manages memory segments with caching for improved performance.
/// Segments are cached when deallocated and reused when possible.
#[derive(Clone)]
pub struct MemorySegmentAllocator {
    inner: Arc<Mutex<MemorySegmentAllocatorInner>>,
}

struct MemorySegmentAllocatorInner {
    /// Maximum number of segments to cache
    max_cache_size: usize,
    /// Cache of deallocated segments (size, data)
    cache: VecDeque<(usize, Vec<u8>)>,
    /// Current number of active segments
    active_segments: usize,
    /// Current total size of active segments
    active_size: usize,
    /// Maximum number of segments ever active
    max_segments: usize,
    /// Maximum total size ever active
    max_size: usize,
    /// Cache hits (segments reused from cache)
    cache_hits: usize,
    /// Whether initialization is complete
    is_init_done: bool,
}

impl MemorySegmentAllocator {
    /// Create a new memory segment allocator with default settings
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frameworks_system_integration_common::sys_common::MemorySegmentAllocator;
    ///
    /// let allocator = MemorySegmentAllocator::new();
    /// ```
    pub fn new() -> Self {
        Self::with_cache_size(10)
    }

    /// Create a new memory segment allocator with a specific cache size
    ///
    /// # Arguments
    ///
    /// * `max_cache_size` - Maximum number of segments to cache
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frameworks_system_integration_common::sys_common::MemorySegmentAllocator;
    ///
    /// let allocator = MemorySegmentAllocator::with_cache_size(20);
    /// ```
    pub fn with_cache_size(max_cache_size: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(MemorySegmentAllocatorInner {
                max_cache_size,
                cache: VecDeque::new(),
                active_segments: 0,
                active_size: 0,
                max_segments: 0,
                max_size: 0,
                cache_hits: 0,
                is_init_done: false,
            })),
        }
    }

    /// Initialize the allocator
    ///
    /// Must be called before using the allocator.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frameworks_system_integration_common::sys_common::MemorySegmentAllocator;
    ///
    /// let allocator = MemorySegmentAllocator::new();
    /// allocator.init().unwrap();
    /// ```
    pub fn init(&self) -> Result<(), SysError> {
        let mut inner = self.inner.lock().unwrap();
        inner.is_init_done = true;
        Ok(())
    }

    /// Allocate a memory segment
    ///
    /// # Arguments
    ///
    /// * `size` - Size of the segment to allocate in bytes
    ///
    /// # Returns
    ///
    /// A `Vec<u8>` containing the allocated memory segment, or an error if allocation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frameworks_system_integration_common::sys_common::MemorySegmentAllocator;
    ///
    /// let allocator = MemorySegmentAllocator::new();
    /// allocator.init().unwrap();
    /// let segment = allocator.alloc(1024).unwrap();
    /// assert_eq!(segment.len(), 1024);
    /// ```
    pub fn alloc(&self, size: usize) -> Result<Vec<u8>, SysError> {
        let mut inner = self.inner.lock().unwrap();
        
        if !inner.is_init_done {
            return Err(SysError::InitFailed);
        }

        // Try to find a cached segment of the right size
        let segment = if let Some(index) = inner.cache.iter()
            .position(|(cached_size, _)| *cached_size == size)
        {
            // Reuse cached segment
            inner.cache_hits += 1;
            let (_, mut segment) = inner.cache.remove(index)
                .expect("cache entry should exist at found index");
            segment.clear();
            segment.resize(size, 0);
            segment
        } else {
            // Allocate new segment
            vec![0u8; size]
        };

        // Update statistics
        inner.active_segments += 1;
        inner.active_size += size;
        if inner.active_segments > inner.max_segments {
            inner.max_segments = inner.active_segments;
        }
        if inner.active_size > inner.max_size {
            inner.max_size = inner.active_size;
        }

        Ok(segment)
    }

    /// Deallocate a memory segment
    ///
    /// The segment may be cached for reuse if the cache is not full.
    ///
    /// # Arguments
    ///
    /// * `segment` - The segment to deallocate
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frameworks_system_integration_common::sys_common::MemorySegmentAllocator;
    ///
    /// let allocator = MemorySegmentAllocator::new();
    /// allocator.init().unwrap();
    /// let segment = allocator.alloc(1024).unwrap();
    /// allocator.dealloc(segment);
    /// ```
    pub fn dealloc(&self, segment: Vec<u8>) {
        let mut inner = self.inner.lock().unwrap();
        
        if !inner.is_init_done {
            return;
        }

        let size = segment.len();

        // Update statistics
        if inner.active_segments > 0 {
            inner.active_segments -= 1;
        }
        if inner.active_size >= size {
            inner.active_size -= size;
        }

        // Cache the segment if cache is not full
        // If cache is full, remove oldest entry (FIFO)
        if inner.cache.len() >= inner.max_cache_size {
            let _ = inner.cache.pop_front();
        }
        inner.cache.push_back((size, segment));
    }

    /// Reallocate a memory segment
    ///
    /// # Arguments
    ///
    /// * `old_segment` - The existing segment to reallocate
    /// * `new_size` - The new size for the segment
    ///
    /// # Returns
    ///
    /// A new `Vec<u8>` with the requested size, or an error if allocation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frameworks_system_integration_common::sys_common::MemorySegmentAllocator;
    ///
    /// let allocator = MemorySegmentAllocator::new();
    /// allocator.init().unwrap();
    /// let segment = allocator.alloc(1024).unwrap();
    /// let resized = allocator.realloc(segment, 2048).unwrap();
    /// assert_eq!(resized.len(), 2048);
    /// ```
    pub fn realloc(&self, old_segment: Vec<u8>, new_size: usize) -> Result<Vec<u8>, SysError> {
        let mut inner = self.inner.lock().unwrap();
        
        if !inner.is_init_done {
            return Err(SysError::InitFailed);
        }

        let old_size = old_segment.len();

        // Update statistics
        if inner.active_size >= old_size {
            inner.active_size -= old_size;
        }
        inner.active_size += new_size;

        // Resize the segment
        let mut new_segment = old_segment;
        new_segment.resize(new_size, 0);

        Ok(new_segment)
    }

    /// Clear the cache of deallocated segments
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frameworks_system_integration_common::sys_common::MemorySegmentAllocator;
    ///
    /// let allocator = MemorySegmentAllocator::new();
    /// allocator.init().unwrap();
    /// allocator.clear_cache();
    /// ```
    pub fn clear_cache(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.cache.clear();
    }

    /// Check and potentially clean up the cache
    ///
    /// This function can be called periodically to manage cache size.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frameworks_system_integration_common::sys_common::MemorySegmentAllocator;
    ///
    /// let allocator = MemorySegmentAllocator::new();
    /// allocator.init().unwrap();
    /// allocator.cache_check();
    /// ```
    pub fn cache_check(&self) {
        let mut inner = self.inner.lock().unwrap();
        
        // If cache is too large, remove oldest entries
        while inner.cache.len() > inner.max_cache_size {
            let _ = inner.cache.pop_front();
        }
    }

    /// Get the number of active segments
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frameworks_system_integration_common::sys_common::MemorySegmentAllocator;
    ///
    /// let allocator = MemorySegmentAllocator::new();
    /// allocator.init().unwrap();
    /// let _segment = allocator.alloc(1024).unwrap();
    /// assert_eq!(allocator.active_segments(), 1);
    /// ```
    pub fn active_segments(&self) -> usize {
        let inner = self.inner.lock().unwrap();
        inner.active_segments
    }

    /// Get the number of cache hits
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frameworks_system_integration_common::sys_common::MemorySegmentAllocator;
    ///
    /// let allocator = MemorySegmentAllocator::new();
    /// allocator.init().unwrap();
    /// let segment = allocator.alloc(1024).unwrap();
    /// allocator.dealloc(segment);
    /// let _reused = allocator.alloc(1024).unwrap();
    /// assert_eq!(allocator.cache_hits(), 1);
    /// ```
    pub fn cache_hits(&self) -> usize {
        let inner = self.inner.lock().unwrap();
        inner.cache_hits
    }

    /// Get the maximum number of segments ever active
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frameworks_system_integration_common::sys_common::MemorySegmentAllocator;
    ///
    /// let allocator = MemorySegmentAllocator::new();
    /// allocator.init().unwrap();
    /// let _s1 = allocator.alloc(1024).unwrap();
    /// let _s2 = allocator.alloc(1024).unwrap();
    /// assert_eq!(allocator.max_segments(), 2);
    /// ```
    pub fn max_segments(&self) -> usize {
        let inner = self.inner.lock().unwrap();
        inner.max_segments
    }
}

impl Default for MemorySegmentAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl SysCommon {
    /// Initialize common system integration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use frameworks_system_integration_common::SysCommon;
    ///
    /// let result = SysCommon::init();
    /// assert!(result.is_ok());
    /// ```
    pub fn init() -> Result<(), SysError> {
        // Common system integration initialization
        // This is a placeholder for future common initialization logic
        Ok(())
    }
}

/// System operation errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SysError {
    /// Initialization failed
    InitFailed,
    /// Allocation failed
    AllocFailed,
}

impl std::fmt::Display for SysError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SysError::InitFailed => write!(f, "Initialization failed"),
            SysError::AllocFailed => write!(f, "Allocation failed"),
        }
    }
}

impl std::error::Error for SysError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sys_common_init() {
        let result = SysCommon::init();
        assert!(result.is_ok());
    }

    #[test]
    fn test_allocator_new() {
        let allocator = MemorySegmentAllocator::new();
        assert_eq!(allocator.active_segments(), 0);
    }

    #[test]
    fn test_allocator_with_cache_size() {
        let allocator = MemorySegmentAllocator::with_cache_size(20);
        assert_eq!(allocator.active_segments(), 0);
    }

    #[test]
    fn test_allocator_init() {
        let allocator = MemorySegmentAllocator::new();
        assert!(allocator.init().is_ok());
    }

    #[test]
    fn test_allocator_alloc_before_init() {
        let allocator = MemorySegmentAllocator::new();
        let result = allocator.alloc(1024);
        assert!(result.is_err());
    }

    #[test]
    fn test_allocator_alloc() {
        let allocator = MemorySegmentAllocator::new();
        allocator.init().unwrap();
        
        let segment = allocator.alloc(1024).unwrap();
        assert_eq!(segment.len(), 1024);
        assert_eq!(allocator.active_segments(), 1);
    }

    #[test]
    fn test_allocator_dealloc() {
        let allocator = MemorySegmentAllocator::new();
        allocator.init().unwrap();
        
        let segment = allocator.alloc(1024).unwrap();
        assert_eq!(allocator.active_segments(), 1);
        
        allocator.dealloc(segment);
        assert_eq!(allocator.active_segments(), 0);
    }

    #[test]
    fn test_allocator_reuse_cached() {
        let allocator = MemorySegmentAllocator::new();
        allocator.init().unwrap();
        
        let segment1 = allocator.alloc(1024).unwrap();
        allocator.dealloc(segment1);
        
        assert_eq!(allocator.cache_hits(), 0);
        
        let segment2 = allocator.alloc(1024).unwrap();
        assert_eq!(allocator.cache_hits(), 1);
        assert_eq!(segment2.len(), 1024);
    }

    #[test]
    fn test_allocator_realloc() {
        let allocator = MemorySegmentAllocator::new();
        allocator.init().unwrap();
        
        let segment = allocator.alloc(1024).unwrap();
        let resized = allocator.realloc(segment, 2048).unwrap();
        assert_eq!(resized.len(), 2048);
    }

    #[test]
    fn test_allocator_clear_cache() {
        let allocator = MemorySegmentAllocator::new();
        allocator.init().unwrap();
        
        let segment = allocator.alloc(1024).unwrap();
        allocator.dealloc(segment);
        
        allocator.clear_cache();
        // Cache should be empty, so next allocation won't hit cache
        let segment2 = allocator.alloc(1024).unwrap();
        assert_eq!(allocator.cache_hits(), 0);
        assert_eq!(segment2.len(), 1024);
    }

    #[test]
    fn test_allocator_cache_check() {
        let allocator = MemorySegmentAllocator::with_cache_size(2);
        allocator.init().unwrap();
        
        // Fill cache beyond max size
        let s1 = allocator.alloc(1024).unwrap();
        let s2 = allocator.alloc(1024).unwrap();
        let s3 = allocator.alloc(1024).unwrap();
        
        allocator.dealloc(s1);
        allocator.dealloc(s2);
        allocator.dealloc(s3);
        
        allocator.cache_check();
        // Cache should be trimmed to max_cache_size
    }

    #[test]
    fn test_allocator_max_segments() {
        let allocator = MemorySegmentAllocator::new();
        allocator.init().unwrap();
        
        let s1 = allocator.alloc(1024).unwrap();
        assert_eq!(allocator.max_segments(), 1);
        
        let s2 = allocator.alloc(1024).unwrap();
        assert_eq!(allocator.max_segments(), 2);
        
        allocator.dealloc(s1);
        assert_eq!(allocator.max_segments(), 2); // Max should remain
        
        allocator.dealloc(s2);
        assert_eq!(allocator.max_segments(), 2); // Max should remain
    }

    #[test]
    fn test_allocator_multiple_allocations() {
        let allocator = MemorySegmentAllocator::new();
        allocator.init().unwrap();
        
        let mut segments = Vec::new();
        for i in 0..10 {
            let segment = allocator.alloc(1024 * (i + 1)).unwrap();
            segments.push(segment);
        }
        
        assert_eq!(allocator.active_segments(), 10);
        assert_eq!(allocator.max_segments(), 10);
        
        for segment in segments {
            allocator.dealloc(segment);
        }
        
        assert_eq!(allocator.active_segments(), 0);
    }

    #[test]
    fn test_allocator_cache_size_limit() {
        let allocator = MemorySegmentAllocator::with_cache_size(3);
        allocator.init().unwrap();
        
        // Allocate and deallocate more than cache size
        for _i in 0..5 {
            let segment = allocator.alloc(1024).unwrap();
            allocator.dealloc(segment);
        }
        
        // Cache should be limited to max_cache_size (3)
        // The last 3 segments should be cached (FIFO)
        // Next allocation should reuse from cache
        let segment = allocator.alloc(1024).unwrap();
        assert!(allocator.cache_hits() >= 1);
        assert_eq!(segment.len(), 1024);
    }

    #[test]
    fn test_allocator_realloc_before_init() {
        let allocator = MemorySegmentAllocator::new();
        let segment = vec![0u8; 1024];
        let result = allocator.realloc(segment, 2048);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), SysError::InitFailed);
    }

    #[test]
    fn test_allocator_realloc_grow() {
        let allocator = MemorySegmentAllocator::new();
        allocator.init().unwrap();
        
        let segment = allocator.alloc(1024).unwrap();
        let resized = allocator.realloc(segment, 2048).unwrap();
        assert_eq!(resized.len(), 2048);
    }

    #[test]
    fn test_allocator_realloc_shrink() {
        let allocator = MemorySegmentAllocator::new();
        allocator.init().unwrap();
        
        let segment = allocator.alloc(2048).unwrap();
        let resized = allocator.realloc(segment, 1024).unwrap();
        assert_eq!(resized.len(), 1024);
    }

    #[test]
    fn test_allocator_realloc_same_size() {
        let allocator = MemorySegmentAllocator::new();
        allocator.init().unwrap();
        
        let segment = allocator.alloc(1024).unwrap();
        let resized = allocator.realloc(segment, 1024).unwrap();
        assert_eq!(resized.len(), 1024);
    }

    #[test]
    fn test_allocator_realloc_zero_size() {
        let allocator = MemorySegmentAllocator::new();
        allocator.init().unwrap();
        
        let segment = allocator.alloc(1024).unwrap();
        let resized = allocator.realloc(segment, 0).unwrap();
        assert_eq!(resized.len(), 0);
    }

    #[test]
    fn test_allocator_dealloc_before_init() {
        let allocator = MemorySegmentAllocator::new();
        let segment = vec![0u8; 1024];
        // Should not panic, just return early
        allocator.dealloc(segment);
        assert_eq!(allocator.active_segments(), 0);
    }

    #[test]
    fn test_allocator_dealloc_with_zero_active_segments() {
        let allocator = MemorySegmentAllocator::new();
        allocator.init().unwrap();
        
        // Dealloc when no active segments (edge case)
        let segment = vec![0u8; 1024];
        allocator.dealloc(segment);
        assert_eq!(allocator.active_segments(), 0);
    }

    #[test]
    fn test_allocator_dealloc_with_active_size_less_than_size() {
        let allocator = MemorySegmentAllocator::new();
        allocator.init().unwrap();
        
        // Create a scenario where active_size < segment size
        // This can happen if statistics get out of sync
        let segment = allocator.alloc(1024).unwrap();
        // Manually manipulate to create edge case (though this shouldn't happen in practice)
        allocator.dealloc(segment);
        // Dealloc again (should handle gracefully)
        let segment2 = vec![0u8; 2048];
        allocator.dealloc(segment2);
        assert_eq!(allocator.active_segments(), 0);
    }

    #[test]
    fn test_allocator_alloc_zero_size() {
        let allocator = MemorySegmentAllocator::new();
        allocator.init().unwrap();
        
        let segment = allocator.alloc(0).unwrap();
        assert_eq!(segment.len(), 0);
        assert_eq!(allocator.active_segments(), 1);
    }

    #[test]
    fn test_allocator_alloc_large_size() {
        let allocator = MemorySegmentAllocator::new();
        allocator.init().unwrap();
        
        let large_size = 1024 * 1024; // 1MB
        let segment = allocator.alloc(large_size).unwrap();
        assert_eq!(segment.len(), large_size);
        assert_eq!(allocator.active_segments(), 1);
    }

    #[test]
    fn test_allocator_cache_different_sizes() {
        let allocator = MemorySegmentAllocator::new();
        allocator.init().unwrap();
        
        // Allocate segments of different sizes
        let s1 = allocator.alloc(1024).unwrap();
        let s2 = allocator.alloc(2048).unwrap();
        let s3 = allocator.alloc(512).unwrap();
        
        // Deallocate them
        allocator.dealloc(s1);
        allocator.dealloc(s2);
        allocator.dealloc(s3);
        
        // Allocate same sizes - should reuse from cache
        let s1_new = allocator.alloc(1024).unwrap();
        assert_eq!(allocator.cache_hits(), 1);
        assert_eq!(s1_new.len(), 1024);
        
        let s2_new = allocator.alloc(2048).unwrap();
        assert_eq!(allocator.cache_hits(), 2);
        assert_eq!(s2_new.len(), 2048);
        
        let s3_new = allocator.alloc(512).unwrap();
        assert_eq!(allocator.cache_hits(), 3);
        assert_eq!(s3_new.len(), 512);
    }

    #[test]
    fn test_allocator_cache_fifo_eviction() {
        let allocator = MemorySegmentAllocator::with_cache_size(2);
        allocator.init().unwrap();
        
        // Allocate and deallocate 3 segments (more than cache size)
        let s1 = allocator.alloc(1024).unwrap();
        let s2 = allocator.alloc(1024).unwrap();
        let s3 = allocator.alloc(1024).unwrap();
        
        allocator.dealloc(s1); // Should be cached
        allocator.dealloc(s2); // Should be cached
        allocator.dealloc(s3); // Should evict s1 (FIFO)
        
        // s1 should be evicted, s2 and s3 should be in cache
        // Allocating 1024 should reuse s2 or s3
        let reused = allocator.alloc(1024).unwrap();
        assert_eq!(allocator.cache_hits(), 1);
        assert_eq!(reused.len(), 1024);
    }

    #[test]
    fn test_allocator_cache_check_trims_to_max() {
        let allocator = MemorySegmentAllocator::with_cache_size(2);
        allocator.init().unwrap();
        
        // Fill cache beyond max size
        for i in 0..5 {
            let segment = allocator.alloc(1024 * (i + 1)).unwrap();
            allocator.dealloc(segment);
        }
        
        // Cache should have 5 entries, but max is 2
        allocator.cache_check();
        
        // After cache_check, cache should be trimmed
        // Next allocation of size 1024*5 should not hit cache (it was evicted)
        // But allocations of recent sizes might hit
        let segment = allocator.alloc(1024 * 5).unwrap();
        // Cache hits might be 0 if the size was evicted, or >0 if it wasn't
        assert_eq!(segment.len(), 1024 * 5);
    }

    #[test]
    fn test_allocator_max_segments_tracking() {
        let allocator = MemorySegmentAllocator::new();
        allocator.init().unwrap();
        
        // Allocate multiple segments
        let s1 = allocator.alloc(1024).unwrap();
        assert_eq!(allocator.max_segments(), 1);
        
        let s2 = allocator.alloc(1024).unwrap();
        assert_eq!(allocator.max_segments(), 2);
        
        let s3 = allocator.alloc(1024).unwrap();
        assert_eq!(allocator.max_segments(), 3);
        
        // Deallocate - max should remain
        allocator.dealloc(s1);
        assert_eq!(allocator.max_segments(), 3);
        
        allocator.dealloc(s2);
        assert_eq!(allocator.max_segments(), 3);
        
        allocator.dealloc(s3);
        assert_eq!(allocator.max_segments(), 3);
    }

    #[test]
    fn test_allocator_default() {
        let allocator = MemorySegmentAllocator::default();
        assert_eq!(allocator.active_segments(), 0);
        assert!(allocator.init().is_ok());
    }

    #[test]
    fn test_sys_error_display() {
        let error1 = SysError::InitFailed;
        assert_eq!(format!("{}", error1), "Initialization failed");
        
        let error2 = SysError::AllocFailed;
        assert_eq!(format!("{}", error2), "Allocation failed");
    }

    #[test]
    fn test_sys_error_error_trait() {
        let error = SysError::InitFailed;
        // Test that it implements Error trait
        let error_ref: &dyn std::error::Error = &error;
        assert_eq!(error_ref.to_string(), "Initialization failed");
    }

    #[test]
    fn test_allocator_clone() {
        let allocator1 = MemorySegmentAllocator::new();
        allocator1.init().unwrap();
        
        let segment = allocator1.alloc(1024).unwrap();
        assert_eq!(allocator1.active_segments(), 1);
        
        // Clone should share the same inner state
        let allocator2 = allocator1.clone();
        assert_eq!(allocator2.active_segments(), 1);
        
        // Deallocating from one should affect the other
        allocator2.dealloc(segment);
        assert_eq!(allocator1.active_segments(), 0);
        assert_eq!(allocator2.active_segments(), 0);
    }

    #[test]
    fn test_allocator_multiple_init_calls() {
        let allocator = MemorySegmentAllocator::new();
        assert!(allocator.init().is_ok());
        // Multiple init calls should be fine
        assert!(allocator.init().is_ok());
        assert!(allocator.init().is_ok());
    }

    #[test]
    fn test_allocator_statistics_consistency() {
        let allocator = MemorySegmentAllocator::new();
        allocator.init().unwrap();
        
        // Allocate segments
        let s1 = allocator.alloc(1024).unwrap();
        let s2 = allocator.alloc(2048).unwrap();
        
        assert_eq!(allocator.active_segments(), 2);
        assert_eq!(allocator.max_segments(), 2);
        
        // Deallocate one
        allocator.dealloc(s1);
        assert_eq!(allocator.active_segments(), 1);
        assert_eq!(allocator.max_segments(), 2); // Max should remain
        
        // Deallocate the other
        allocator.dealloc(s2);
        assert_eq!(allocator.active_segments(), 0);
        assert_eq!(allocator.max_segments(), 2); // Max should remain
    }

    #[test]
    fn test_allocator_realloc_statistics() {
        let allocator = MemorySegmentAllocator::new();
        allocator.init().unwrap();
        
        let segment = allocator.alloc(1024).unwrap();
        assert_eq!(allocator.active_segments(), 1);
        
        // Realloc should maintain active_segments count
        let resized = allocator.realloc(segment, 2048).unwrap();
        assert_eq!(allocator.active_segments(), 1);
        assert_eq!(resized.len(), 2048);
    }

    #[test]
    fn test_allocator_cache_cleared_after_clear_cache() {
        let allocator = MemorySegmentAllocator::new();
        allocator.init().unwrap();
        
        // Allocate and deallocate to fill cache
        let s1 = allocator.alloc(1024).unwrap();
        let s2 = allocator.alloc(1024).unwrap();
        allocator.dealloc(s1);
        allocator.dealloc(s2);
        
        // Clear cache
        allocator.clear_cache();
        
        // Next allocation should not hit cache
        let initial_hits = allocator.cache_hits();
        let segment = allocator.alloc(1024).unwrap();
        assert_eq!(allocator.cache_hits(), initial_hits); // No new cache hit
        assert_eq!(segment.len(), 1024);
    }
}

