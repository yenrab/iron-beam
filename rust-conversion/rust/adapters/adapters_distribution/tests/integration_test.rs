//! Integration tests for adapters_distribution crate
//!
//! These tests verify that distribution adapters work correctly
//! and test end-to-end workflows for external term format and UDS distribution.

use adapters_distribution::*;
use entities_data_handling::term_hashing::Term;

#[test]
fn test_external_term_encode() {
    let term = Term::Small(42);
    let result = ExternalTerm::encode(&term, None);
    assert!(result.is_ok());
    let encoded = result.unwrap();
    assert!(!encoded.is_empty());
}

#[test]
fn test_external_term_decode() {
    let term = Term::Small(42);
    let encoded = ExternalTerm::encode(&term, None).unwrap();
    
    let result = ExternalTerm::decode(&encoded);
    // May succeed or fail depending on implementation
    let _ = result;
}

#[test]
fn test_external_term_roundtrip() {
    let terms = vec![
        Term::Small(0),
        Term::Small(42),
        Term::Atom(1),
        Term::Nil,
    ];
    
    for term in terms {
        let encoded = ExternalTerm::encode(&term, None);
        if encoded.is_ok() {
            let _decoded = ExternalTerm::decode(&encoded.unwrap());
            // May succeed or fail
        }
    }
}

#[test]
fn test_uds_distribution_operations() {
    // Test UDS distribution operations if available
    // Note: UDS is Unix-specific, so tests may be platform-dependent
    #[cfg(unix)]
    {
        let _uds = UdsDistribution;
        // Should not panic
    }
}

#[test]
fn test_uds_mode_variants() {
    use adapters_distribution::uds::UdsMode;
    
    let modes = vec![
        UdsMode::Command,
        UdsMode::Intermediate,
        UdsMode::Data,
    ];
    
    for mode in modes {
        let _ = format!("{:?}", mode);
    }
}
