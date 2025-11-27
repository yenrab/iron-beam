//! Integration tests for entities_system_integration_common crate
//!
//! These tests verify memory mapping operations end-to-end.

use entities_system_integration_common::*;
use std::fs;

#[test]
fn test_memory_map_lifecycle() {
    // Test complete lifecycle: create file, map, read, access
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("test_mmap_lifecycle");
    
    // Create test file with some data
    let test_data = b"Hello, Memory Mapping!";
    fs::write(&test_file, test_data).unwrap();
    
    // Map the file
    let mmap = MemoryMap::map_file(&test_file).unwrap();
    
    // Verify data
    assert_eq!(mmap.data(), test_data);
    assert_eq!(mmap.len(), test_data.len());
    
    // Access data multiple times
    let data1 = mmap.data();
    let data2 = mmap.data();
    assert_eq!(data1, data2);
    assert_eq!(data1, test_data);
    
    // Cleanup
    let _ = fs::remove_file(&test_file);
}

#[test]
fn test_memory_map_different_file_sizes() {
    // Test mapping files of different sizes
    let temp_dir = std::env::temp_dir();
    
    // Test small file
    let small_file = temp_dir.join("test_mmap_small");
    let small_data = b"small";
    fs::write(&small_file, small_data).unwrap();
    let mmap_small = MemoryMap::map_file(&small_file).unwrap();
    assert_eq!(mmap_small.len(), 5);
    assert_eq!(mmap_small.data(), small_data);
    let _ = fs::remove_file(&small_file);
    
    // Test medium file
    let medium_file = temp_dir.join("test_mmap_medium");
    let medium_data = vec![0u8; 1024]; // 1KB
    fs::write(&medium_file, &medium_data).unwrap();
    let mmap_medium = MemoryMap::map_file(&medium_file).unwrap();
    assert_eq!(mmap_medium.len(), 1024);
    assert_eq!(mmap_medium.data(), &medium_data[..]);
    let _ = fs::remove_file(&medium_file);
    
    // Test larger file
    let large_file = temp_dir.join("test_mmap_large");
    let large_data = vec![42u8; 10240]; // 10KB (10 * 1024)
    fs::write(&large_file, &large_data).unwrap();
    let mmap_large = MemoryMap::map_file(&large_file).unwrap();
    assert_eq!(mmap_large.len(), 10 * 1024);
    assert_eq!(mmap_large.data(), &large_data[..]);
    let _ = fs::remove_file(&large_file);
}

#[test]
fn test_memory_map_empty_file() {
    // Test mapping an empty file
    let temp_dir = std::env::temp_dir();
    let empty_file = temp_dir.join("test_mmap_empty");
    
    // Create empty file
    fs::File::create(&empty_file).unwrap();
    
    // Map it
    let mmap = MemoryMap::map_file(&empty_file).unwrap();
    
    // Should be empty but valid
    assert_eq!(mmap.len(), 0);
    assert!(mmap.data().is_empty());
    
    // Cleanup
    let _ = fs::remove_file(&empty_file);
}

#[test]
fn test_memory_map_binary_data() {
    // Test mapping file with binary data (non-text)
    let temp_dir = std::env::temp_dir();
    let binary_file = temp_dir.join("test_mmap_binary");
    
    // Create file with binary data
    let binary_data: Vec<u8> = (0..=255).collect(); // All byte values 0-255
    fs::write(&binary_file, &binary_data).unwrap();
    
    // Map it
    let mmap = MemoryMap::map_file(&binary_file).unwrap();
    
    // Verify all bytes
    assert_eq!(mmap.len(), 256);
    assert_eq!(mmap.data(), &binary_data[..]);
    
    // Verify specific bytes
    assert_eq!(mmap.data()[0], 0);
    assert_eq!(mmap.data()[255], 255);
    assert_eq!(mmap.data()[128], 128);
    
    // Cleanup
    let _ = fs::remove_file(&binary_file);
}

#[test]
fn test_memory_map_multiple_accesses() {
    // Test that we can access mapped data multiple times
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("test_mmap_multiple");
    
    let test_data = b"Multiple access test data";
    fs::write(&test_file, test_data).unwrap();
    
    let mmap = MemoryMap::map_file(&test_file).unwrap();
    
    // Access data multiple times
    for _ in 0..10 {
        let data = mmap.data();
        assert_eq!(data, test_data);
        assert_eq!(data.len(), test_data.len());
    }
    
    // Verify len() is consistent
    for _ in 0..10 {
        assert_eq!(mmap.len(), test_data.len());
    }
    
    // Cleanup
    let _ = fs::remove_file(&test_file);
}

#[test]
fn test_memory_map_error_handling() {
    // Test error handling for non-existent file
    let non_existent = std::env::temp_dir().join("test_mmap_nonexistent_12345");
    
    let result = MemoryMap::map_file(&non_existent);
    
    // Verify it's an IO error
    assert!(result.is_err());
    // The error kind should be NotFound
    match result {
        Err(e) => assert_eq!(e.kind(), std::io::ErrorKind::NotFound),
        Ok(_) => panic!("Expected error for non-existent file"),
    }
}

