//! Integration tests for usecases_memory_management crate
//!
//! These tests verify that memory allocation strategies work correctly
//! and test end-to-end allocation/deallocation workflows.
//!
//! Note: Tests for unsafe code (DefaultAllocator's unsafe blocks) are excluded
//! as they require manual testing.

use usecases_memory_management::{Allocator, AllocatorType, AllocationError};
use usecases_memory_management::goodfit::GoodFitAllocator;
use usecases_memory_management::bestfit::BestFitAllocator;
use usecases_memory_management::firstfit::FirstFitAllocator;
use usecases_memory_management::afit::AFitAllocator;

#[test]
fn test_allocator_type_enum() {
    // Test AllocatorType enum variants
    let types = vec![
        AllocatorType::GoodFit,
        AllocatorType::BestFit,
        AllocatorType::AFit,
        AllocatorType::FirstFit,
    ];
    
    // Verify all types can be created and compared
    for alloc_type in &types {
        let _ = format!("{:?}", alloc_type);
    }
    
    // Test equality
    assert_eq!(AllocatorType::GoodFit, AllocatorType::GoodFit);
    assert_ne!(AllocatorType::GoodFit, AllocatorType::BestFit);
}

#[test]
fn test_allocation_error_enum() {
    // Test AllocationError enum variants
    let errors = vec![
        AllocationError::OutOfMemory,
        AllocationError::InvalidSize,
        AllocationError::AllocatorNotAvailable,
    ];
    
    // Verify all errors can be created and compared
    for error in &errors {
        let _ = format!("{:?}", error);
    }
    
    // Test equality
    assert_eq!(AllocationError::OutOfMemory, AllocationError::OutOfMemory);
    assert_ne!(AllocationError::OutOfMemory, AllocationError::InvalidSize);
}

#[test]
fn test_goodfit_allocator_basic() {
    let allocator = GoodFitAllocator::new();
    
    // Test allocation
    let size = 1024;
    let ptr = allocator.alloc(size);
    assert!(ptr.is_ok());
    let ptr = ptr.unwrap();
    assert!(!ptr.is_null());
    
    // Test deallocation
    allocator.dealloc(ptr, size);
}

#[test]
fn test_goodfit_allocator_multiple_allocations() {
    let allocator = GoodFitAllocator::new();
    
    // Allocate multiple blocks
    let ptr1 = allocator.alloc(100).unwrap();
    let ptr2 = allocator.alloc(200).unwrap();
    let ptr3 = allocator.alloc(300).unwrap();
    
    assert!(!ptr1.is_null());
    assert!(!ptr2.is_null());
    assert!(!ptr3.is_null());
    
    // Deallocate in reverse order
    allocator.dealloc(ptr3, 300);
    allocator.dealloc(ptr2, 200);
    allocator.dealloc(ptr1, 100);
}

#[test]
fn test_goodfit_allocator_reallocation() {
    let allocator = GoodFitAllocator::new();
    
    // Allocate initial block
    let size1 = 100;
    let ptr1 = allocator.alloc(size1).unwrap();
    
    // Reallocate to larger size
    let size2 = 200;
    let ptr2 = allocator.realloc(ptr1, size1, size2);
    assert!(ptr2.is_ok());
    let ptr2 = ptr2.unwrap();
    assert!(!ptr2.is_null());
    
    // Deallocate
    allocator.dealloc(ptr2, size2);
}

#[test]
fn test_goodfit_allocator_error_cases() {
    let allocator = GoodFitAllocator::new();
    
    // Test allocation with zero size
    let result = allocator.alloc(0);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), AllocationError::InvalidSize);
    
    // Test deallocation with zero size
    let ptr = allocator.alloc(100).unwrap();
    let result = allocator.dealloc(ptr, 0);
    // Should handle gracefully (may succeed or fail depending on implementation)
    let _ = result;
}

#[test]
fn test_bestfit_allocator_basic() {
    let allocator = BestFitAllocator::new();
    
    // Test allocation
    let size = 1024;
    let ptr = allocator.alloc(size);
    assert!(ptr.is_ok());
    let ptr = ptr.unwrap();
    assert!(!ptr.is_null());
    
    // Test deallocation
    allocator.dealloc(ptr, size);
}

#[test]
fn test_bestfit_allocator_multiple_allocations() {
    let allocator = BestFitAllocator::new();
    
    // Allocate multiple blocks of different sizes
    let ptr1 = allocator.alloc(50).unwrap();
    let ptr2 = allocator.alloc(150).unwrap();
    let ptr3 = allocator.alloc(250).unwrap();
    
    assert!(!ptr1.is_null());
    assert!(!ptr2.is_null());
    assert!(!ptr3.is_null());
    
    // Deallocate
    allocator.dealloc(ptr1, 50);
    allocator.dealloc(ptr2, 150);
    allocator.dealloc(ptr3, 250);
}

#[test]
fn test_bestfit_allocator_reallocation() {
    let allocator = BestFitAllocator::new();
    
    // Allocate initial block
    let size1 = 100;
    let ptr1 = allocator.alloc(size1).unwrap();
    
    // Reallocate to larger size
    let size2 = 200;
    let ptr2 = allocator.realloc(ptr1, size1, size2);
    assert!(ptr2.is_ok());
    let ptr2 = ptr2.unwrap();
    assert!(!ptr2.is_null());
    
    // Deallocate
    allocator.dealloc(ptr2, size2);
}

#[test]
fn test_bestfit_allocator_error_cases() {
    let allocator = BestFitAllocator::new();
    
    // Test allocation with zero size
    let result = allocator.alloc(0);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), AllocationError::InvalidSize);
}

#[test]
fn test_firstfit_allocator_basic() {
    let allocator = FirstFitAllocator::new();
    
    // Test allocation
    let size = 1024;
    let ptr = allocator.alloc(size);
    assert!(ptr.is_ok());
    let ptr = ptr.unwrap();
    assert!(!ptr.is_null());
    
    // Test deallocation
    allocator.dealloc(ptr, size);
}

#[test]
fn test_firstfit_allocator_multiple_allocations() {
    let allocator = FirstFitAllocator::new();
    
    // Allocate multiple blocks
    let ptr1 = allocator.alloc(100).unwrap();
    let ptr2 = allocator.alloc(200).unwrap();
    let ptr3 = allocator.alloc(300).unwrap();
    
    assert!(!ptr1.is_null());
    assert!(!ptr2.is_null());
    assert!(!ptr3.is_null());
    
    // Deallocate
    allocator.dealloc(ptr1, 100);
    allocator.dealloc(ptr2, 200);
    allocator.dealloc(ptr3, 300);
}

#[test]
fn test_firstfit_allocator_reallocation() {
    let allocator = FirstFitAllocator::new();
    
    // Allocate initial block
    let size1 = 100;
    let ptr1 = allocator.alloc(size1).unwrap();
    
    // Reallocate to larger size
    let size2 = 200;
    let ptr2 = allocator.realloc(ptr1, size1, size2);
    assert!(ptr2.is_ok());
    let ptr2 = ptr2.unwrap();
    assert!(!ptr2.is_null());
    
    // Deallocate
    allocator.dealloc(ptr2, size2);
}

#[test]
fn test_firstfit_allocator_error_cases() {
    let allocator = FirstFitAllocator::new();
    
    // Test allocation with zero size
    let result = allocator.alloc(0);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), AllocationError::InvalidSize);
}

#[test]
fn test_afit_allocator_basic() {
    let allocator = AFitAllocator::new();
    
    // Test allocation
    let size = 1024;
    let ptr = allocator.alloc(size);
    assert!(ptr.is_ok());
    let ptr = ptr.unwrap();
    assert!(!ptr.is_null());
    
    // Test deallocation
    allocator.dealloc(ptr, size);
}

#[test]
fn test_afit_allocator_multiple_allocations() {
    let allocator = AFitAllocator::new();
    
    // Allocate multiple blocks
    let ptr1 = allocator.alloc(100).unwrap();
    let ptr2 = allocator.alloc(200).unwrap();
    let ptr3 = allocator.alloc(300).unwrap();
    
    assert!(!ptr1.is_null());
    assert!(!ptr2.is_null());
    assert!(!ptr3.is_null());
    
    // Deallocate
    allocator.dealloc(ptr1, 100);
    allocator.dealloc(ptr2, 200);
    allocator.dealloc(ptr3, 300);
}

#[test]
fn test_afit_allocator_reallocation() {
    let allocator = AFitAllocator::new();
    
    // Allocate initial block
    let size1 = 100;
    let ptr1 = allocator.alloc(size1).unwrap();
    
    // Reallocate to larger size
    let size2 = 200;
    let ptr2 = allocator.realloc(ptr1, size1, size2);
    assert!(ptr2.is_ok());
    let ptr2 = ptr2.unwrap();
    assert!(!ptr2.is_null());
    
    // Deallocate
    allocator.dealloc(ptr2, size2);
}

#[test]
fn test_afit_allocator_error_cases() {
    let allocator = AFitAllocator::new();
    
    // Test allocation with zero size
    let result = allocator.alloc(0);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), AllocationError::InvalidSize);
}

#[test]
fn test_allocator_trait_consistency() {
    // Test that all allocators implement the Allocator trait consistently
    let allocators: Vec<Box<dyn Allocator>> = vec![
        Box::new(GoodFitAllocator::new()),
        Box::new(BestFitAllocator::new()),
        Box::new(FirstFitAllocator::new()),
        Box::new(AFitAllocator::new()),
    ];
    
    for allocator in allocators {
        // All should handle zero-size allocation the same way
        let result = allocator.alloc(0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AllocationError::InvalidSize);
        
        // All should handle normal allocation
        let ptr = allocator.alloc(100);
        assert!(ptr.is_ok());
        let ptr = ptr.unwrap();
        assert!(!ptr.is_null());
        
        // All should handle deallocation
        allocator.dealloc(ptr, 100);
    }
}

#[test]
fn test_allocator_reallocation_preserves_data() {
    // Test that reallocation preserves data when possible
    let allocator = GoodFitAllocator::new();
    
    // Allocate and write data
    let size1 = 100;
    let ptr1 = allocator.alloc(size1).unwrap();
    unsafe {
        let slice = std::slice::from_raw_parts_mut(ptr1, size1);
        for (i, byte) in slice.iter_mut().enumerate() {
            *byte = (i % 256) as u8;
        }
    }
    
    // Reallocate to larger size
    let size2 = 200;
    let ptr2 = allocator.realloc(ptr1, size1, size2).unwrap();
    
    // Verify data is preserved in the first part
    unsafe {
        let slice = std::slice::from_raw_parts(ptr2, size1);
        for (i, &byte) in slice.iter().enumerate() {
            assert_eq!(byte, (i % 256) as u8);
        }
    }
    
    // Deallocate
    allocator.dealloc(ptr2, size2);
}

#[test]
fn test_allocator_concurrent_access() {
    use std::sync::Arc;
    use std::thread;
    
    let allocator = Arc::new(GoodFitAllocator::new());
    
    // Test concurrent allocations
    let mut handles = vec![];
    for i in 0..10 {
        let alloc = allocator.clone();
        let handle = thread::spawn(move || {
            let ptr = alloc.alloc(100).unwrap();
            // Simulate some work
            thread::sleep(std::time::Duration::from_millis(10));
            alloc.dealloc(ptr, 100);
            i
        });
        handles.push(handle);
    }
    
    // Wait for all threads
    for handle in handles {
        let _ = handle.join().unwrap();
    }
}

#[test]
fn test_allocator_different_sizes() {
    let allocator = GoodFitAllocator::new();
    
    // Test various allocation sizes
    let sizes = vec![1, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096];
    
    let mut pointers = vec![];
    for size in &sizes {
        let ptr = allocator.alloc(*size).unwrap();
        assert!(!ptr.is_null());
        pointers.push((ptr, *size));
    }
    
    // Deallocate all
    for (ptr, size) in pointers {
        allocator.dealloc(ptr, size);
    }
}
