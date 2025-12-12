//! Integration tests for adapters_ets_tables crate
//!
//! These tests verify that ETS table debugging adapters work correctly.

use adapters_ets_tables::*;
use std::fs;
use std::path::Path;

#[test]
fn test_socket_debug_new() {
    let debug = SocketDebug::new();
    // Should not panic
    let _ = debug;
}

#[test]
fn test_socket_debug_init_stdout() {
    let mut debug = SocketDebug::new();
    assert!(debug.init(None));
    assert!(debug.init(Some("")));
}

#[test]
fn test_socket_debug_init_file() {
    let mut debug = SocketDebug::new();
    let test_file = "/tmp/test_socket_debug_integration.txt";
    
    // Clean up if exists
    let _ = fs::remove_file(test_file);
    
    assert!(debug.init(Some(test_file)));
    
    // Verify file was created
    assert!(Path::new(test_file).exists());
    
    // Clean up
    let _ = fs::remove_file(test_file);
}

#[test]
fn test_socket_debug_printf() {
    let mut debug = SocketDebug::new();
    assert!(debug.init(None));
    
    // Should not panic
    debug.printf("TEST", "Integration test message", &[]);
}

#[test]
fn test_esock_dbg_init() {
    let test_file = "/tmp/test_esock_dbg_init.txt";
    let _ = fs::remove_file(test_file);
    
    assert!(esock_dbg_init(Some(test_file)));
    assert!(Path::new(test_file).exists());
    
    // Clean up
    let _ = fs::remove_file(test_file);
}

#[test]
fn test_esock_dbg_printf() {
    // Should not panic
    esock_dbg_printf("TEST", "Global debug message");
}

#[test]
fn test_socket_debug_clone() {
    let debug1 = SocketDebug::new();
    let debug2 = debug1.clone();
    
    // Both should work
    debug1.printf("TEST", "Clone test 1", &[]);
    debug2.printf("TEST", "Clone test 2", &[]);
}

#[test]
fn test_socket_debug_default() {
    let debug = SocketDebug::default();
    debug.printf("TEST", "Default test", &[]);
}

#[test]
fn test_socket_debug_file_write() {
    let mut debug = SocketDebug::new();
    let test_file = "/tmp/test_socket_debug_write.txt";
    
    let _ = fs::remove_file(test_file);
    assert!(debug.init(Some(test_file)));
    
    debug.printf("TEST", "Write test", &[]);
    
    // Verify content was written
    if let Ok(content) = fs::read_to_string(test_file) {
        assert!(content.contains("Write test"));
    }
    
    // Clean up
    let _ = fs::remove_file(test_file);
}
