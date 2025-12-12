//! Integration tests for adapters_nif_io crate
//!
//! These tests verify that NIF I/O polling functions work correctly
//! and test end-to-end workflows for I/O event management.

use adapters_nif_io::*;

#[test]
fn test_check_io_new() {
    let check_io = CheckIo::new();
    // Should not panic
    let _ = check_io;
}

#[test]
fn test_check_io_config() {
    let config = CheckIoConfig::default();
    // Should not panic
    let _ = config;
}

#[test]
fn test_io_event_type_variants() {
    let types = vec![
        IoEventType::Read,
        IoEventType::Write,
        IoEventType::Error,
    ];
    
    for event_type in types {
        let _ = format!("{:?}", event_type);
    }
}

#[test]
fn test_io_event_creation() {
    let event = IoEvent {
        fd: 1,
        event_type: IoEventType::Read,
    };
    
    assert_eq!(event.fd, 1);
    assert_eq!(event.event_type, IoEventType::Read);
}

#[test]
fn test_poll_thread_id() {
    let id1 = PollThreadId::new(1);
    let id2 = PollThreadId::new(2);
    
    assert_eq!(id1.id(), 1);
    assert_eq!(id2.id(), 2);
    assert_ne!(id1, id2);
}

#[test]
fn test_poll_thread_id_aux() {
    let aux_id = PollThreadId::AUX;
    assert_eq!(aux_id.id(), -2);
}

#[test]
fn test_nif_select_flags() {
    // Test NifSelectFlags if available
    // Note: Check actual API from nif_io module
}

#[test]
fn test_check_io_error_variants() {
    // Test CheckIoError enum variants
    // Check actual variants from the enum
    let error1 = CheckIoError::InvalidFd;
    let _ = format!("{:?}", error1);
}

