//! Metadata tracking for JIT code
//!
//! Tracks metadata for debugging, tracing, and performance analysis.
//! Converted from C++ code in beam_jit_metadata.cpp.

use std::collections::HashMap;
use std::sync::RwLock;

/// Assembly range information
///
/// Tracks a range of code with associated metadata.
/// Note: Raw pointers are not Send/Sync, but we use them here for JIT code addresses.
/// The metadata registry is thread-safe, and pointers are only accessed from the
/// thread that created them or after proper synchronization.
#[derive(Debug, Clone)]
pub struct AsmRange {
    /// Start address
    pub start: *const u8,
    /// Stop address
    pub stop: *const u8,
    /// Name of the range
    pub name: String,
    /// Line data for debugging
    pub lines: Vec<LineData>,
}

// Safety: AsmRange contains raw pointers, but they are only used for tracking
// JIT code addresses. The pointers are not dereferenced across threads without
// proper synchronization.
unsafe impl Send for AsmRange {}
unsafe impl Sync for AsmRange {}

/// Line data for debugging
#[derive(Debug, Clone)]
pub struct LineData {
    /// Start address
    pub start: *const u8,
    /// Source file
    pub file: String,
    /// Line number
    pub line: u32,
}

// Safety: LineData contains raw pointers, but they are only used for tracking
// JIT code addresses. The pointers are not dereferenced across threads without
// proper synchronization.
unsafe impl Send for LineData {}
unsafe impl Sync for LineData {}

/// Global metadata registry
///
/// Thread-safe registry for tracking all JIT code metadata.
static METADATA_REGISTRY: OnceLock<RwLock<MetadataRegistry>> = OnceLock::new();

fn get_registry() -> &'static RwLock<MetadataRegistry> {
    METADATA_REGISTRY.get_or_init(|| RwLock::new(MetadataRegistry::new()))
}

use std::sync::OnceLock;

/// Metadata registry
struct MetadataRegistry {
    /// Module metadata by name
    modules: HashMap<String, ModuleMetadata>,
}

unsafe impl Sync for MetadataRegistry {}

/// Module metadata
struct ModuleMetadata {
    /// Base address
    base: *const u8,
    /// Size
    size: usize,
    /// Ranges
    ranges: Vec<AsmRange>,
}

// Safety: ModuleMetadata contains raw pointers, but they are only used for tracking
// JIT code addresses. The pointers are not dereferenced across threads without
// proper synchronization.
unsafe impl Send for ModuleMetadata {}
unsafe impl Sync for ModuleMetadata {}

impl MetadataRegistry {
    fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    fn insert(
        &mut self,
        name: &str,
        base: *const u8,
        size: usize,
        ranges: Vec<AsmRange>,
    ) {
        self.modules.insert(
            name.to_string(),
            ModuleMetadata { base, size, ranges },
        );
    }

    fn remove(&mut self, name: &str) {
        self.modules.remove(name);
    }

    fn get(&self, name: &str) -> Option<&ModuleMetadata> {
        self.modules.get(name)
    }
}

/// BeamAsm metadata manager
///
/// Public interface for metadata management.
pub struct BeamAsmMetadata;

impl BeamAsmMetadata {
    /// Insert metadata for a module
    ///
    /// Registers metadata for a JIT-compiled module.
    pub fn insert(
        name: &str,
        base: *const u8,
        size: usize,
        ranges: Vec<AsmRange>,
    ) -> Result<(), ()> {
        let mut registry = get_registry().write().unwrap();
        registry.insert(name, base, size, ranges);
        Ok(())
    }

    /// Remove metadata for a module
    ///
    /// Unregisters metadata when a module is purged.
    pub fn remove(name: &str) {
        let mut registry = get_registry().write().unwrap();
        registry.remove(name);
    }

    /// Get metadata for a module
    ///
    /// Retrieves metadata for debugging or analysis.
    pub fn get(name: &str) -> Option<(*const u8, usize, Vec<AsmRange>)> {
        let registry = get_registry().read().unwrap();
        registry.get(name).map(|m| (m.base, m.size, m.ranges.clone()))
    }

    /// Find range containing an address
    ///
    /// Finds the code range containing a given address, useful for debugging.
    pub fn find_range(addr: *const u8) -> Option<AsmRange> {
        let registry = get_registry().read().unwrap();
        for module in registry.modules.values() {
            for range in &module.ranges {
                if addr >= range.start && addr < range.stop {
                    return Some(range.clone());
                }
            }
        }
        None
    }
}

