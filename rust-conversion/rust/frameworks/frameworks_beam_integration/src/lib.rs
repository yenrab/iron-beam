/*!
# BEAM Integration Framework

**CLEAN Architecture**: Frameworks & Drivers Layer (Layer 5)
**SOLID Responsibility**: BEAM virtual machine integration and runtime services

## Overview

This crate provides the final integration layer for the Erlang BEAM virtual machine.
It handles bytecode generation, runtime loading, external framework bindings, and
JIT compilation interfaces. This is the outermost layer that connects the compiler
to the actual BEAM runtime environment.

## BEAM Integration Points

### 1. Bytecode Generation
```rust
use frameworks_beam_integration::bytecode::*;

// Generate BEAM bytecode from compilation results
let generator = BytecodeGenerator::new();
let beam_file = generator.generate_beam_file(&compilation_result)?;
```

### 2. Runtime Loading
```rust
use frameworks_beam_integration::runtime::*;

// Load compiled modules into BEAM runtime
let loader = ModuleLoader::new();
loader.load_module(&beam_file)?;
loader.call_function("my_module", "start", &[])?;
```

### 3. JIT Compilation
```rust
use frameworks_beam_integration::*;

// Enable JIT compilation for performance
let jit_allocator = infrastructure_beamasm::jit::JitAllocator::new()?;
let (exec_ptr, write_ptr, size) = jit_allocator.allocate(4096)?;
```

## Architecture Compliance

- **CLEAN Layer**: Frameworks & Drivers (Layer 5) - External system frameworks
- **Dependencies**: All previous layers (Entities → Use Cases → Interface Adapters → Infrastructure)
- **SOLID Principle**: Single responsibility for BEAM VM integration
- **Runtime Integration**: Clean separation between compilation and execution
- **External Frameworks**: Bindings to external libraries and systems

## BEAM Components

### Core Runtime Services
- **Module Loading**: Dynamic loading of compiled BEAM modules
- **Function Calling**: Runtime function invocation across module boundaries
- **Process Management**: BEAM process lifecycle and communication
- **Memory Management**: Integration with BEAM's garbage collection

### Bytecode Generation
- **BEAM File Format**: Standard .beam file generation
- **Opcode Encoding**: BEAM instruction set encoding
- **Metadata Attachment**: Debug info, line numbers, type information
- **Optimization**: Bytecode-level optimizations

### External Integrations
- **NIFs (Native Implemented Functions)**: C/C++/Rust native code integration
- **Ports**: External program communication
- **Drivers**: Low-level system interfaces
- **JIT Compiler**: Just-in-time compilation for performance

## Runtime Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Compiler      │───▶│  BEAM Runtime   │───▶│  External       │
│   (Rust)        │    │  Integration     │    │  Frameworks     │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                              │
                              ▼
                       ┌─────────────────┐
                       │   BEAM VM       │
                       │   (Execution)   │
                       └─────────────────┘
```

## Integration Workflow

1. **Compilation**: Source → AST → Bytecode (previous layers)
2. **Generation**: Bytecode → BEAM file format (this layer)
3. **Loading**: BEAM file → Runtime module loading (this layer)
4. **Execution**: Function calls and process management (this layer)
5. **Integration**: External frameworks and native code (this layer)
*/

use std::collections::HashMap;
use std::path::Path;
use entities_erlang_syntax::*;
use entities_process::*;
use usecases_compilation::*;
use interfaces_compiler_api::*;

// Re-export key framework components
pub use runtime::*;
pub use external::*;

// Re-export JIT from infrastructure_beamasm
pub use infrastructure_beamasm::jit;

// Framework modules
mod runtime;
mod external;

/// Main BEAM integration coordinator
///
/// This is the primary interface for BEAM virtual machine integration.
/// It coordinates runtime loading and external framework bindings.
/// BEAM bytecode generation is handled at the interfaces layer.
pub struct BeamIntegration {
    module_loader: ModuleLoader,
    jit_allocator: Option<infrastructure_beamasm::jit::JitAllocator>,
    external_bindings: ExternalBindings,
    loaded_modules: HashMap<String, LoadedModule>,
}

impl BeamIntegration {
    /// Create a new BEAM integration instance
    pub fn new() -> Self {
        Self {
            module_loader: ModuleLoader::new(),
            jit_allocator: None,
            external_bindings: ExternalBindings::new(),
            loaded_modules: HashMap::new(),
        }
    }

    /// Enable JIT compilation
    pub fn with_jit(mut self) -> BeamResult<Self> {
        self.jit_allocator = Some(infrastructure_beamasm::jit::JitAllocator::new()?);
        Ok(self)
    }

    /// Load a BEAM module into the runtime
    ///
    /// This method takes a BeamFile and loads it into the BEAM runtime environment.
    pub async fn load_beam_file(
        &mut self,
        beam_file: &BeamFile,
    ) -> BeamResult<LoadedModule> {
        // Load into runtime
        let loaded_module = self.module_loader.load_module(beam_file)?;

        // Register external bindings if needed
        self.external_bindings.register_module_bindings(&loaded_module)?;

        // Store reference
        let module_name = beam_file.module_name.clone();
        self.loaded_modules.insert(module_name, loaded_module.clone());

        Ok(loaded_module)
    }

    /// Call a function in a loaded module
    pub async fn call_function(
        &self,
        module_name: &str,
        function_name: &str,
        args: &[BeamValue],
    ) -> BeamResult<BeamValue> {
        let module = self.loaded_modules.get(module_name)
            .ok_or_else(|| BeamError::ModuleNotFound(module_name.to_string()))?;

        self.module_loader.call_function(module, function_name, args).await
    }

    /// Get information about loaded modules
    pub fn get_loaded_modules(&self) -> Vec<&LoadedModule> {
        self.loaded_modules.values().collect()
    }

    /// Unload a module from the runtime
    pub fn unload_module(&mut self, module_name: &str) -> BeamResult<()> {
        if let Some(module) = self.loaded_modules.remove(module_name) {
            self.module_loader.unload_module(&module)?;
            self.external_bindings.unregister_module_bindings(&module)?;
        }
        Ok(())
    }

    /// Get runtime statistics
    pub fn get_runtime_stats(&self) -> RuntimeStats {
        RuntimeStats {
            loaded_modules: self.loaded_modules.len(),
            total_memory: self.module_loader.get_memory_usage(),
            active_processes: self.module_loader.get_process_count(),
            jit_enabled: self.jit_allocator.is_some(),
        }
    }

    /// Shutdown the BEAM integration
    pub async fn shutdown(mut self) -> BeamResult<()> {
        // Clean up all loaded modules
        for module in self.loaded_modules.values() {
            self.module_loader.unload_module(module)?;
        }

        // Shutdown external bindings
        self.external_bindings.shutdown()?;

        Ok(())
    }
}

impl Default for BeamIntegration {
    fn default() -> Self {
        Self::new()
    }
}

/// BEAM value types for runtime communication
#[derive(Debug, Clone)]
pub enum BeamValue {
    Integer(i64),
    Float(f64),
    Atom(String),
    String(String),
    List(Vec<BeamValue>),
    Tuple(Vec<BeamValue>),
    Binary(Vec<u8>),
    Pid(entities_process::ProcessId),
    Reference(Reference),
}

/// BEAM reference
#[derive(Debug, Clone)]
pub struct Reference(pub Vec<u8>);

/// Loaded module information
#[derive(Debug, Clone)]
pub struct LoadedModule {
    pub name: String,
    pub functions: Vec<String>,
    pub attributes: HashMap<String, String>,
    pub memory_size: usize,
}

/// Runtime statistics
#[derive(Debug, Clone)]
pub struct RuntimeStats {
    pub loaded_modules: usize,
    pub total_memory: usize,
    pub active_processes: usize,
    pub jit_enabled: bool,
}

/// BEAM result type
pub type BeamResult<T> = Result<T, BeamError>;

/// BEAM error types
#[derive(Debug, thiserror::Error)]
pub enum BeamError {
    #[error("Module not found: {0}")]
    ModuleNotFound(String),

    #[error("Function not found: {0}")]
    FunctionNotFound(String),

    #[error("Invalid bytecode: {0}")]
    InvalidBytecode(String),

    #[error("Runtime error: {0}")]
    RuntimeError(String),

    #[error("JIT compilation failed: {0}")]
    JITError(String),

    #[error("External binding error: {0}")]
    ExternalBindingError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beam_integration_creation() {
        let integration = BeamIntegration::new();
        assert!(integration.loaded_modules.is_empty());
        assert!(integration.jit_allocator.is_none());
    }

    #[test]
    fn test_beam_integration_with_jit() {
        let integration = BeamIntegration::new().with_jit().unwrap();
        assert!(integration.jit_allocator.is_some());
    }

    #[test]
    fn test_runtime_stats() {
        let integration = BeamIntegration::new();
        let stats = integration.get_runtime_stats();

        assert_eq!(stats.loaded_modules, 0);
        assert_eq!(stats.active_processes, 0);
        assert!(!stats.jit_enabled);
    }

    #[test]
    fn test_loaded_modules_empty() {
        let integration = BeamIntegration::new();
        let modules = integration.get_loaded_modules();
        assert!(modules.is_empty());
    }

    #[test]
    fn test_beam_values() {
        let int_val = BeamValue::Integer(42);
        let atom_val = BeamValue::Atom("ok".to_string());
        let pid_val = BeamValue::Pid(123);
        let list_val = BeamValue::List(vec![int_val.clone(), atom_val.clone()]);

        match list_val {
            BeamValue::List(items) => assert_eq!(items.len(), 2),
            _ => panic!("Expected list"),
        }

        match pid_val {
            BeamValue::Pid(pid) => assert_eq!(pid, 123),
            _ => panic!("Expected pid"),
        }
    }

    #[test]
    fn test_beam_error_display() {
        let error = BeamError::ModuleNotFound("test_module".to_string());
        assert!(error.to_string().contains("test_module"));
    }

    #[test]
    fn test_loaded_module_creation() {
        let module = LoadedModule {
            name: "test".to_string(),
            functions: vec!["func1".to_string(), "func2".to_string()],
            attributes: HashMap::new(),
            memory_size: 1024,
        };

        assert_eq!(module.name, "test");
        assert_eq!(module.functions.len(), 2);
        assert_eq!(module.memory_size, 1024);
    }

    #[test]
    fn test_process_id_and_reference() {
        let pid = ProcessId(12345);
        let reference = Reference(vec![1, 2, 3, 4]);

        assert_eq!(pid.0, 12345);
        assert_eq!(reference.0.len(), 4);
    }
}
