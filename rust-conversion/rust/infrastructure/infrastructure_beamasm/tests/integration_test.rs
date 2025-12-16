//! Integration tests for infrastructure_beamasm
//!
//! Tests the main functionality of the BeamAsm JIT system.

use infrastructure_beamasm::{
    beamasm_init, beamasm_new_assembler, BeamAsmLoader, BeamAssemblerError,
};

#[test]
fn test_beamasm_init() {
    // Test that initialization succeeds
    let result = beamasm_init();
    assert!(result.is_ok());
}

#[test]
fn test_beamasm_new_assembler() {
    // Initialize first
    beamasm_init().unwrap();

    // Test creating a new assembler
    let module = 0; // Placeholder Eterm
    let num_labels = 10;
    let num_functions = 5;
    let beam_file = b"BEAM"; // Placeholder BEAM file header

    let result = beamasm_new_assembler(module, num_labels, num_functions, beam_file);
    
    // Should succeed on supported architectures (x86_64 or aarch64)
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    {
        assert!(result.is_ok());
    }
    
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        assert!(matches!(
            result,
            Err(BeamAssemblerError::UnsupportedArchitecture)
        ));
    }
}

#[test]
fn test_loader_creation() {
    // Test creating a loader
    let result = BeamAsmLoader::new();
    assert!(result.is_ok());
}

#[test]
fn test_loader_prepare_emit() {
    // Initialize
    beamasm_init().unwrap();
    
    // Create loader
    let mut loader = BeamAsmLoader::new().unwrap();
    
    // Test prepare_emit
    let module = 0;
    let num_labels = 10;
    let num_functions = 5;
    let beam_file = b"BEAM";
    
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    {
        let result = loader.prepare_emit(module, num_labels, num_functions, beam_file);
        assert!(result.is_ok());
    }
}

#[test]
fn test_jit_allocator() {
    use infrastructure_beamasm::JitAllocator;
    
    // Test creating allocator
    let result = JitAllocator::new();
    assert!(result.is_ok());
    
    // Test allocation
    let mut allocator = JitAllocator::new().unwrap();
    let result = allocator.allocate(1024);
    assert!(result.is_ok());
    
    let (executable, writable, size) = result.unwrap();
    assert!(!executable.is_null());
    assert!(!writable.is_null());
    // Size may be rounded up to page size, so check it's at least what we requested
    assert!(size >= 1024);
}

#[test]
fn test_metadata_operations() {
    use infrastructure_beamasm::BeamAsmMetadata;
    use infrastructure_beamasm::metadata::{AsmRange, LineData};
    
    // Test inserting metadata
    let name = "test_module";
    let base = std::ptr::null();
    let size = 1024;
    let ranges = vec![AsmRange {
        start: std::ptr::null(),
        stop: std::ptr::null(),
        name: "test_range".to_string(),
        lines: vec![LineData {
            start: std::ptr::null(),
            file: "test.rs".to_string(),
            line: 1,
        }],
    }];
    
    let result = BeamAsmMetadata::insert(name, base, size, ranges);
    assert!(result.is_ok());
    
    // Test getting metadata
    let metadata = BeamAsmMetadata::get(name);
    assert!(metadata.is_some());
    
    // Test removing metadata
    BeamAsmMetadata::remove(name);
    let metadata_after = BeamAsmMetadata::get(name);
    assert!(metadata_after.is_none());
}

#[test]
fn test_arg_val_creation() {
    use infrastructure_beamasm::{ArgVal, ArgType};
    
    // Test creating different argument types
    let word = ArgVal::word(42);
    assert_eq!(word.value(), 42);
    assert!(word.tag_type() == ArgType::Word);
    
    let x_reg = ArgVal::x_reg(5);
    assert_eq!(x_reg.value(), 5);
    assert!(x_reg.tag_type() == ArgType::XReg);
    
    let label = ArgVal::label(10);
    assert_eq!(label.value(), 10);
    assert!(label.is_label());
    
    let literal = ArgVal::literal(3);
    assert_eq!(literal.value(), 3);
    assert!(literal.is_literal());
}

#[test]
fn test_type_id_operations() {
    use infrastructure_beamasm::types::BeamTypeId;
    
    // Test type checks
    assert!(BeamTypeId::Pid.is_identifier());
    assert!(BeamTypeId::Port.is_identifier());
    assert!(BeamTypeId::Reference.is_identifier());
    
    assert!(BeamTypeId::Cons.is_list());
    assert!(BeamTypeId::Nil.is_list());
    
    assert!(BeamTypeId::Float.is_number());
    assert!(BeamTypeId::Integer.is_number());
    
    assert!(BeamTypeId::Bitstring.maybe_boxed());
    assert!(BeamTypeId::Float.maybe_boxed());
    
    assert!(BeamTypeId::Atom.maybe_immediate());
    assert!(BeamTypeId::Integer.maybe_immediate());
}

