//! Integration tests for adapters_drivers crate
//!
//! These tests verify that driver adapters work correctly
//! and test end-to-end workflows for INET and RAM file drivers.

use adapters_drivers::*;

#[test]
fn test_inet_driver_new() {
    let driver = InetDriver::new();
    // Should not panic
    let _ = driver;
}

#[test]
fn test_ram_file_driver_new() {
    let driver = RamFileDriver::new();
    // Should not panic
    let _ = driver;
}

#[test]
fn test_ram_file_driver_create_read() {
    let mut driver = RamFileDriver::new();
    
    driver.create_file("test.txt".to_string(), b"test data".to_vec());
    let data = driver.read_file("test.txt");
    
    assert!(data.is_some());
    assert_eq!(data.unwrap(), b"test data");
}

#[test]
fn test_ram_file_driver_read_nonexistent() {
    let driver = RamFileDriver::new();
    
    let data = driver.read_file("nonexistent.txt");
    assert!(data.is_none());
}

#[test]
fn test_ram_file_driver_multiple_files() {
    let mut driver = RamFileDriver::new();
    
    driver.create_file("file1.txt".to_string(), b"data1".to_vec());
    driver.create_file("file2.txt".to_string(), b"data2".to_vec());
    
    assert_eq!(driver.read_file("file1.txt"), Some(b"data1".as_slice()));
    assert_eq!(driver.read_file("file2.txt"), Some(b"data2".as_slice()));
}

#[test]
fn test_ram_file_driver_overwrite() {
    let mut driver = RamFileDriver::new();
    
    driver.create_file("test.txt".to_string(), b"old data".to_vec());
    driver.create_file("test.txt".to_string(), b"new data".to_vec());
    
    assert_eq!(driver.read_file("test.txt"), Some(b"new data".as_slice()));
}

#[test]
fn test_ram_file_driver_empty_file() {
    let mut driver = RamFileDriver::new();
    
    driver.create_file("empty.txt".to_string(), vec![]);
    let data = driver.read_file("empty.txt");
    
    assert!(data.is_some());
    assert_eq!(data.unwrap(), b"");
}

#[test]
fn test_ram_file_driver_large_file() {
    let mut driver = RamFileDriver::new();
    
    let large_data = vec![0x42u8; 10000];
    driver.create_file("large.txt".to_string(), large_data.clone());
    
    let data = driver.read_file("large.txt");
    assert!(data.is_some());
    assert_eq!(data.unwrap(), large_data.as_slice());
}

