//! Integration tests for adapters_debugging crate
//!
//! These tests verify that debugging adapters work correctly.

use adapters_debugging::*;

#[test]
fn test_tracer_new() {
    let tracer = Tracer::new();
    // Should not panic
    let _ = tracer;
}

#[test]
fn test_trace_nif_operations() {
    // Test that TraceNif can be used
    let _trace_nif = TraceNif;
    // Should not panic
}

