# Kernel API Design

## Executive Summary

The Kernel API provides a standardized interface for language implementations (Erl tool, Elixir, Gleam, etc.) to interact with the BEAM kernel. This API wraps the existing Rust kernel implementation and is designed to be **language-agnostic** and **extensible**, enabling the creation of new languages that compile to BEAM bytecode.

**Note**: This document defines the API interface. The actual kernel implementation already exists in the Rust codebase. This API provides a clean abstraction layer for language implementations.

## 1. Overview

### 1.1 Purpose

The Kernel API enables:
- Language implementations (Erl tool, etc.) to use the BEAM kernel
- New languages to be created easily
- Language-specific features to be built on top of the kernel
- Interoperability between languages

**Implementation Note**: This API wraps the existing Rust kernel crates (entities, infrastructure, usecases, adapters, frameworks, code_management). The API does not reimplement functionality - it provides a clean interface to existing kernel code.

### 1.2 Kernel Scope

The Kernel API provides access to the complete BEAM kernel, which includes all existing Rust crates:
- ✅ All Entities layer crates (core data structures)
- ✅ All Infrastructure layer crates (core runtime services)
- ✅ All Use Cases layer crates (business logic)
- ✅ Adapters layer crates (external interfaces, except NIF adapters)
- ✅ Frameworks layer crates (system integration)
- ✅ Code Management layer crates (module loading)

**Note**: NIFs are handled separately via the NIF API (see `../nif_isolation/`). The kernel provides the runtime environment for NIFs but does not include NIF implementations.

### 1.3 Design Principles

1. **Language Agnostic**: No assumptions about source language
2. **Type Safe**: Leverage Rust's type system
3. **Extensible**: Easy to add new features
4. **Performant**: Minimal overhead
5. **Ergonomic**: Easy to use from Rust
6. **FFI-Friendly**: Can be exposed via C FFI for other languages

### 1.3 API Layers

```
┌─────────────────────────────────────────┐
│      Language Implementation            │
│  (Erl, Elixir, Gleam, etc.)            │
└──────────────┬──────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────┐
│         Kernel API                      │
│  (This Document)                        │
└──────────────┬──────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────┐
│         BEAM Kernel                     │
│  (Core Runtime)                         │
└─────────────────────────────────────────┘
```

## 2. Core API Structure

### 2.1 Kernel Instance

```rust
/// Main kernel instance
pub struct Kernel {
    // Private fields
}

impl Kernel {
    /// Create a new kernel instance with configuration
    pub fn new(config: KernelConfig) -> Result<Self, KernelError>;
    
    /// Initialize the kernel (must be called before use)
    pub fn initialize(&mut self) -> Result<(), KernelError>;
    
    /// Start the kernel runtime
    pub fn start(&mut self) -> Result<(), KernelError>;
    
    /// Shutdown the kernel
    pub fn shutdown(&mut self) -> Result<(), KernelError>;
    
    /// Get kernel configuration
    pub fn config(&self) -> &KernelConfig;
    
    /// Check if kernel is running
    pub fn is_running(&self) -> bool;
}
```

### 2.2 Configuration

```rust
/// Kernel configuration
#[derive(Debug, Clone)]
pub struct KernelConfig {
    /// Number of scheduler threads (None = auto-detect)
    pub scheduler_count: Option<usize>,
    
    /// Maximum number of processes
    pub max_processes: usize,
    
    
    /// Memory limit in bytes (None = unlimited)
    pub memory_limit: Option<usize>,
    
    /// Atom table initial size
    pub atom_table_size: usize,
    
    /// Enable distribution
    pub distribution_enabled: bool,
    
    /// Distribution node name
    pub node_name: Option<String>,
    
    /// Distribution cookie
    pub distribution_cookie: Option<String>,
    
    /// Language-specific callbacks
    pub language_callbacks: LanguageCallbacks,
    
    /// Custom BIF registrations
    pub custom_bifs: Vec<BifRegistration>,
}

impl Default for KernelConfig {
    fn default() -> Self {
        KernelConfig {
            scheduler_count: None,
            max_processes: 1_000_000,
            memory_limit: None,
            atom_table_size: 8192,
            distribution_enabled: false,
            node_name: None,
            distribution_cookie: None,
            language_callbacks: LanguageCallbacks::default(),
            custom_bifs: Vec::new(),
        }
    }
}
```

## 3. Module Management API

### 3.1 Module Loading

```rust
impl Kernel {
    /// Load a BEAM module from bytes
    pub fn load_module(&mut self, beam_data: &[u8]) -> Result<ModuleId, KernelError>;
    
    /// Load a BEAM module from file
    pub fn load_module_from_file(&mut self, path: &Path) -> Result<ModuleId, KernelError>;
    
    /// Get module information
    pub fn get_module_info(&self, module_id: ModuleId) -> Option<ModuleInfo>;
    
    /// Get module by atom name
    pub fn get_module_by_name(&self, name: Atom) -> Option<ModuleId>;
    
    /// Purge a module (remove old code)
    pub fn purge_module(&mut self, module_id: ModuleId) -> Result<(), KernelError>;
    
    /// Delete a module (remove all code)
    pub fn delete_module(&mut self, module_id: ModuleId) -> Result<(), KernelError>;
}

/// Module information
#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub id: ModuleId,
    pub name: Atom,
    pub exports: Vec<ExportInfo>,
    pub imports: Vec<ImportInfo>,
    pub attributes: HashMap<Atom, Term>,
    pub compile_info: HashMap<Atom, Term>,
}

/// Export information
#[derive(Debug, Clone)]
pub struct ExportInfo {
    pub function: Atom,
    pub arity: u32,
    pub label: i32,
}

/// Import information
#[derive(Debug, Clone)]
pub struct ImportInfo {
    pub module: Atom,
    pub function: Atom,
    pub arity: u32,
}
```

### 3.2 Code Management

```rust
impl Kernel {
    /// Get export entry for a function
    pub fn get_export(&self, module: Atom, function: Atom, arity: u32) -> Option<Export>;
    
    /// Check if a module is loaded
    pub fn is_module_loaded(&self, module: Atom) -> bool;
    
    /// Get all loaded modules
    pub fn get_loaded_modules(&self) -> Vec<ModuleId>;
}
```

## 4. Process Management API

### 4.1 Process Creation

```rust
impl Kernel {
    /// Create a new process
    pub fn spawn_process(&mut self, config: ProcessConfig) -> Result<ProcessId, KernelError>;
    
    /// Spawn a process with initial function
    pub fn spawn_function(
        &mut self,
        module: Atom,
        function: Atom,
        args: &[Term],
    ) -> Result<ProcessId, KernelError>;
    
    /// Spawn a process with initial code
    pub fn spawn_code(
        &mut self,
        module: ModuleId,
        function: Atom,
        args: &[Term],
    ) -> Result<ProcessId, KernelError>;
}

/// Process configuration
#[derive(Debug, Clone)]
pub struct ProcessConfig {
    /// Initial heap size
    pub heap_size: usize,
    
    /// Initial stack size
    pub stack_size: usize,
    
    /// Priority (low, normal, high, max)
    pub priority: ProcessPriority,
    
    /// Fullsweep after N garbage collections
    pub fullsweep_after: Option<u32>,
    
    /// Language-specific options
    pub language_options: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessPriority {
    Low,
    Normal,
    High,
    Max,
}
```

### 4.2 Process Control

```rust
impl Kernel {
    /// Get process information
    pub fn get_process_info(&self, pid: ProcessId) -> Option<ProcessInfo>;
    
    /// Check if process is alive
    pub fn is_process_alive(&self, pid: ProcessId) -> bool;
    
    /// Exit a process
    pub fn exit_process(&mut self, pid: ProcessId, reason: Term) -> Result<(), KernelError>;
    
    /// Link two processes
    pub fn link_processes(&mut self, pid1: ProcessId, pid2: ProcessId) -> Result<(), KernelError>;
    
    /// Unlink two processes
    pub fn unlink_processes(&mut self, pid1: ProcessId, pid2: ProcessId) -> Result<(), KernelError>;
    
    /// Monitor a process
    pub fn monitor_process(&mut self, pid: ProcessId) -> Result<MonitorRef, KernelError>;
    
    /// Demonitor a process
    pub fn demonitor_process(&mut self, monitor_ref: MonitorRef) -> Result<(), KernelError>;
}

/// Process information
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: ProcessId,
    pub status: ProcessStatus,
    pub message_queue_len: usize,
    pub heap_size: usize,
    pub stack_size: usize,
    pub reductions: u64,
    pub current_function: Option<FunctionRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessStatus {
    Running,
    Waiting,
    GarbageCollecting,
    Exited,
}
```

### 4.3 Message Passing

```rust
impl Kernel {
    /// Send a message to a process
    pub fn send(&mut self, pid: ProcessId, message: Term) -> Result<(), KernelError>;
    
    /// Receive messages for a process (non-blocking)
    pub fn receive(&mut self, pid: ProcessId) -> Result<Option<Term>, KernelError>;
    
    /// Get message queue length
    pub fn message_queue_len(&self, pid: ProcessId) -> Option<usize>;
}
```

## 5. BIF Registration API

### 5.1 BIF Registration

```rust
impl Kernel {
    /// Register a custom BIF
    pub fn register_bif(
        &mut self,
        module: Atom,
        function: Atom,
        arity: u32,
        implementation: BifImplementation,
    ) -> Result<(), KernelError>;
    
    /// Unregister a custom BIF
    pub fn unregister_bif(
        &mut self,
        module: Atom,
        function: Atom,
        arity: u32,
    ) -> Result<(), KernelError>;
    
    /// Check if a BIF is registered
    pub fn is_bif_registered(&self, module: Atom, function: Atom, arity: u32) -> bool;
}

/// BIF implementation trait
pub trait BifImplementation: Send + Sync {
    /// Execute the BIF
    fn call(&self, args: &[Term]) -> Result<Term, BifError>;
    
    /// Get BIF metadata
    fn metadata(&self) -> BifMetadata;
}

/// BIF metadata
#[derive(Debug, Clone)]
pub struct BifMetadata {
    pub module: Atom,
    pub function: Atom,
    pub arity: u32,
    pub is_guarded: bool,
    pub is_pure: bool,
}

/// BIF error
#[derive(Debug, Clone)]
pub enum BifError {
    BadArg,
    BadArity,
    SystemLimit,
    NotImplemented,
    Custom(String),
}
```

### 5.2 Built-in BIFs

The kernel provides standard BIFs (erlang module, etc.). Language implementations can:
- Use standard BIFs as-is
- Override standard BIFs with custom implementations
- Add language-specific BIFs

## 6. Language Callbacks

### 6.1 Callback Interface

```rust
/// Language-specific callbacks
pub struct LanguageCallbacks {
    /// Called when a process is created
    pub on_process_created: Option<Box<dyn Fn(ProcessId) + Send + Sync>>,
    
    /// Called when a process exits
    pub on_process_exited: Option<Box<dyn Fn(ProcessId, Term) + Send + Sync>>,
    
    /// Called when a module is loaded
    pub on_module_loaded: Option<Box<dyn Fn(ModuleId) + Send + Sync>>,
    
    /// Called when a module is purged
    pub on_module_purged: Option<Box<dyn Fn(ModuleId) + Send + Sync>>,
    
    /// Called for custom error handling
    pub on_error: Option<Box<dyn Fn(&KernelError) + Send + Sync>>,
    
    /// Called for custom logging
    pub on_log: Option<Box<dyn Fn(LogLevel, &str) + Send + Sync>>,
}

impl Default for LanguageCallbacks {
    fn default() -> Self {
        LanguageCallbacks {
            on_process_created: None,
            on_process_exited: None,
            on_module_loaded: None,
            on_module_purged: None,
            on_error: None,
            on_log: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}
```

### 6.2 Event System

```rust
impl Kernel {
    /// Register an event handler
    pub fn register_event_handler(
        &mut self,
        event_type: EventType,
        handler: Box<dyn EventHandler + Send + Sync>,
    ) -> Result<EventHandlerId, KernelError>;
    
    /// Unregister an event handler
    pub fn unregister_event_handler(&mut self, handler_id: EventHandlerId) -> Result<(), KernelError>;
}

pub trait EventHandler: Send + Sync {
    fn handle(&self, event: &Event) -> Result<(), EventError>;
}

pub enum Event {
    ProcessCreated { pid: ProcessId },
    ProcessExited { pid: ProcessId, reason: Term },
    ModuleLoaded { module_id: ModuleId },
    ModulePurged { module_id: ModuleId },
    MessageSent { from: ProcessId, to: ProcessId },
    // ... other events
}
```

## 7. Term and Atom API

### 7.1 Term Creation

```rust
impl Kernel {
    /// Create an atom term
    pub fn make_atom(&self, name: &str) -> Result<Term, KernelError>;
    
    /// Create an integer term
    pub fn make_integer(&self, value: i64) -> Term;
    
    /// Create a float term
    pub fn make_float(&self, value: f64) -> Term;
    
    /// Create a binary term
    pub fn make_binary(&self, data: &[u8]) -> Result<Term, KernelError>;
    
    /// Create a list term
    pub fn make_list(&self, elements: &[Term]) -> Result<Term, KernelError>;
    
    /// Create a tuple term
    pub fn make_tuple(&self, elements: &[Term]) -> Result<Term, KernelError>;
    
    /// Create a map term
    pub fn make_map(&self, entries: &[(Term, Term)]) -> Result<Term, KernelError>;
}

/// Term type (simplified - actual implementation more complex)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Term {
    Atom(Atom),
    Integer(i64),
    Float(f64),
    Binary(BinaryRef),
    List(ListRef),
    Tuple(TupleRef),
    Map(MapRef),
    // ... other types
}
```

### 7.2 Term Inspection

```rust
impl Kernel {
    /// Get term type
    pub fn term_type(&self, term: Term) -> TermType;
    
    /// Convert term to atom (if possible)
    pub fn term_to_atom(&self, term: Term) -> Option<Atom>;
    
    /// Convert term to integer (if possible)
    pub fn term_to_integer(&self, term: Term) -> Option<i64>;
    
    /// Convert term to float (if possible)
    pub fn term_to_float(&self, term: Term) -> Option<f64>;
    
    /// Convert term to binary (if possible)
    pub fn term_to_binary(&self, term: Term) -> Option<Vec<u8>>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TermType {
    Atom,
    Integer,
    Float,
    Binary,
    List,
    Tuple,
    Map,
    Pid,
    Reference,
    Fun,
    // ... other types
}
```

### 7.3 Atom Management

```rust
impl Kernel {
    /// Get or create an atom
    pub fn get_atom(&self, name: &str) -> Option<Atom>;
    
    /// Create an atom (fails if atom table is full)
    pub fn create_atom(&mut self, name: &str) -> Result<Atom, KernelError>;
    
    /// Get atom name
    pub fn atom_name(&self, atom: Atom) -> Option<String>;
}
```

## 8. External Term Format API

### 8.1 Encoding/Decoding

```rust
impl Kernel {
    /// Encode a term to external format
    pub fn term_to_binary(&self, term: Term) -> Result<Vec<u8>, KernelError>;
    
    /// Decode external format to term
    pub fn binary_to_term(&self, data: &[u8]) -> Result<Term, KernelError>;
    
    /// Encode with compression
    pub fn term_to_binary_compressed(&self, term: Term, level: u8) -> Result<Vec<u8>, KernelError>;
}
```

## 9. Distribution API

### 9.1 Distribution Management

```rust
impl Kernel {
    /// Enable distribution
    pub fn enable_distribution(&mut self, node_name: String, cookie: String) -> Result<(), KernelError>;
    
    /// Get node name
    pub fn node_name(&self) -> Option<&str>;
    
    /// Send message to remote process
    pub fn send_remote(&mut self, node: &str, pid: ProcessId, message: Term) -> Result<(), KernelError>;
    
    /// Connect to remote node
    pub fn connect_node(&mut self, node: &str) -> Result<bool, KernelError>;
    
    /// Disconnect from remote node
    pub fn disconnect_node(&mut self, node: &str) -> Result<(), KernelError>;
}
```

## 11. Error Handling

### 11.1 Error Types

```rust
#[derive(Debug, Clone)]
pub enum KernelError {
    /// Initialization error
    InitializationError(String),
    
    /// Process error
    ProcessError {
        pid: Option<ProcessId>,
        reason: String,
    },
    
    /// Module error
    ModuleError {
        module: Option<Atom>,
        reason: String,
    },
    
    /// Memory error
    MemoryError(String),
    
    /// BIF error
    BifError {
        module: Atom,
        function: Atom,
        arity: u32,
        reason: String,
    },
    
    /// Configuration error
    ConfigurationError(String),
    
    /// System limit error
    SystemLimitError(String),
    
    /// Invalid argument
    BadArg(String),
    
    /// Not implemented
    NotImplemented(String),
    
    /// Custom error
    Custom(String),
}
```

### 11.2 Error Conversion

```rust
impl From<ProcessError> for KernelError { ... }
impl From<ModuleError> for KernelError { ... }
impl From<MemoryError> for KernelError { ... }
// ... other conversions
```

## 12. FFI Interface (C API)

### 12.1 C Bindings

For languages that can't use Rust directly, a C FFI interface is provided:

```c
// C header (simplified)
typedef void* KernelHandle;
typedef void* ProcessId;
typedef void* ModuleId;
typedef void* Term;

KernelHandle kernel_new(KernelConfig* config);
int kernel_initialize(KernelHandle kernel);
int kernel_start(KernelHandle kernel);
int kernel_shutdown(KernelHandle kernel);

ProcessId kernel_spawn_process(KernelHandle kernel, ProcessConfig* config);
int kernel_send(KernelHandle kernel, ProcessId pid, Term message);
int kernel_load_module(KernelHandle kernel, const uint8_t* data, size_t len, ModuleId* out);
// ... other C functions
```

### 12.2 FFI Safety

- All FFI functions are marked `unsafe`
- Rust wrapper provides safe API
- Memory management handled by Rust
- Error codes returned for C compatibility

## 13. Usage Examples

### 13.1 Basic Usage

```rust
use beam_kernel::{Kernel, KernelConfig, LanguageCallbacks};

// Create kernel
let config = KernelConfig::default();
let mut kernel = Kernel::new(config)?;

// Initialize
kernel.initialize()?;

// Start
kernel.start()?;

// Load module
let module_id = kernel.load_module_from_file("my_module.beam")?;

// Spawn process
let pid = kernel.spawn_function(
    kernel.get_atom("my_module")?,
    kernel.get_atom("start")?,
    &[],
)?;

// Shutdown
kernel.shutdown()?;
```

### 13.2 Custom BIF Registration

```rust
use beam_kernel::{Kernel, BifImplementation, BifMetadata, BifError, Term};

struct MyBif;

impl BifImplementation for MyBif {
    fn call(&self, args: &[Term]) -> Result<Term, BifError> {
        // Implementation
        Ok(/* result */)
    }
    
    fn metadata(&self) -> BifMetadata {
        BifMetadata {
            module: /* atom */,
            function: /* atom */,
            arity: 1,
            is_guarded: false,
            is_pure: true,
        }
    }
}

let mut kernel = Kernel::new(config)?;
kernel.register_bif(
    kernel.get_atom("my_module")?,
    kernel.get_atom("my_function")?,
    1,
    Box::new(MyBif),
)?;
```

### 13.3 Language Callbacks

```rust
let callbacks = LanguageCallbacks {
    on_process_created: Some(Box::new(|pid| {
        println!("Process created: {:?}", pid);
    })),
    on_process_exited: Some(Box::new(|pid, reason| {
        println!("Process exited: {:?}, reason: {:?}", pid, reason);
    })),
    ..Default::default()
};

let config = KernelConfig {
    language_callbacks: callbacks,
    ..Default::default()
};
```

## 14. API Stability

### 14.1 Versioning

- Major version: Breaking changes
- Minor version: New features, backward compatible
- Patch version: Bug fixes, backward compatible

### 14.2 Deprecation Policy

- Deprecated APIs marked with `#[deprecated]`
- Deprecated APIs removed after 2 major versions
- Migration guides provided

## 15. Performance Considerations

### 15.1 Zero-Copy

- Terms are reference-counted
- Binary data is zero-copy where possible
- Lists and tuples use efficient representations

### 15.2 Caching

- Atom table cached
- Module table cached
- Export table cached

### 15.3 Lock-Free Operations

- Message passing uses lock-free queues
- Process registry uses lock-free data structures
- Scheduler uses work-stealing queues

## 16. Testing

### 16.1 API Tests

- Unit tests for each API function
- Integration tests for API combinations
- Property-based tests for term operations

### 16.2 Compatibility Tests

- Test against Erlang/OTP API
- Test BEAM file compatibility
- Test external term format compatibility

## 17. Documentation

### 17.1 API Documentation

- Complete Rust documentation
- Usage examples
- Error handling guide
- Performance guide

### 17.2 Language Binding Guide

- How to create language bindings
- Best practices
- Common patterns
- Troubleshooting
