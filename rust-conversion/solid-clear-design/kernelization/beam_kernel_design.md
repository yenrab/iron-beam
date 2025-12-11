# BEAM Kernel Partitioning Design

## Executive Summary

This document describes how to partition the existing Rust codebase into:
1. **BEAM Kernel**: Language-agnostic core runtime (stays in kernel)
2. **Language Behavior Framework**: Common templates/behaviors for language implementations
3. **Language Implementations**: Language-specific callback implementations (Erl, Elixir, Gleam, etc.)

The goal is to extract a reusable kernel with a behavior framework that allows language developers to implement callbacks (similar to OTP behaviors) rather than reimplementing common functionality.

## 1. Partitioning Strategy

### 1.1 Overview

The current Rust codebase is organized in CLEAN architecture layers. The partitioning strategy identifies:
- Which crates stay in the kernel
- Which common functionality can be extracted into behavior templates
- What language-specific callbacks need to be implemented

### 1.2 Three-Layer Architecture

**Kernel (Language-Agnostic)**:
- All Entities layer crates
- All Infrastructure layer crates
- All Use Cases layer crates (except language-specific BIFs)
- Adapters layer crates
- Frameworks layer crates
- Code Management layer crates
- API Facades layer (kernel-facing APIs)

**Language Behavior Framework (Common Templates)**:
- Shell/REPL framework (common REPL loop, language-specific parsing/evaluation)
- Compiler integration framework (common interface, language-specific compiler)
- Application startup framework (common lifecycle, language-specific initialization)
- Distribution framework (common protocol, language-specific conventions)
- BIF registration framework (common dispatcher, language-specific BIFs)

**Language Implementations (Callbacks)**:
- Erlang: Implements language behavior callbacks
- Elixir: Implements language behavior callbacks
- Gleam: Implements language behavior callbacks
- etc.

## 2. Crate Partitioning

### 2.1 Entities Layer - All Stay in Kernel

All Entities layer crates remain in the kernel:

- `entities/entities_data_handling` → **Kernel**
- `entities/entities_system_integration_common` → **Kernel**
- `entities/entities_system_integration_win32` → **Kernel**
- `entities/entities_utilities` → **Kernel**
- `entities/entities_io_operations` → **Kernel**
- `entities/entities_process` → **Kernel**

**Rationale**: Core data structures are language-agnostic and fundamental to the kernel.

### 2.2 Infrastructure Layer - All Stay in Kernel

All Infrastructure layer crates remain in the kernel:

- `infrastructure/infrastructure_utilities` → **Kernel**
- `infrastructure/infrastructure_debugging` → **Kernel**
- `infrastructure/infrastructure_ets_tables` → **Kernel**
- `infrastructure/infrastructure_time_management` → **Kernel**
- `infrastructure/infrastructure_bifs` → **Kernel**
- `infrastructure/infrastructure_bif_dispatcher` → **Kernel**
- `infrastructure/infrastructure_emulator_loop` → **Kernel**
- `infrastructure/infrastructure_external_format` → **Kernel**
- `infrastructure/infrastructure_runtime_utils` → **Kernel**
- `infrastructure/infrastructure_code_loading` → **Kernel**
- `infrastructure/infrastructure_data_handling` → **Kernel**
- `infrastructure/infrastructure_bignum_encoding` → **Kernel**
- `infrastructure/infrastructure_trace_encoding` → **Kernel**
- `infrastructure/infrastructure_nif_api` → **Separate** (NIF API - see `../nif_isolation/`)

**Rationale**: Core runtime services are language-agnostic. NIF API is handled separately.

### 2.3 Use Cases Layer - Mostly Kernel

Most Use Cases layer crates stay in kernel:

- `usecases/usecases_memory_management` → **Kernel**
- `usecases/usecases_process_management` → **Kernel**
- `usecases/usecases_scheduling` → **Kernel**
- `usecases/usecases_io_operations` → **Kernel**
- `usecases/usecases_bifs` → **Kernel** (core BIF framework)
  - Language-specific BIF implementations → **Language Behavior Framework**
- `usecases/usecases_nif_compilation` → **Separate** (NIF API)

**Rationale**: Core business logic is language-agnostic. Language-specific BIFs use behavior framework.

### 2.4 Adapters Layer - All Stay in Kernel

All Adapters layer crates stay in kernel:

- `adapters/adapters_ets_tables` → **Kernel**
- `adapters/adapters_debugging` → **Kernel**
- `adapters/adapters_drivers` → **Kernel**
- `adapters/adapters_time_management` → **Kernel**
- `adapters/adapters_system_integration_unix` → **Kernel**
- `adapters/adapters_socket` → **Kernel**
- `adapters/adapters_distribution` → **Kernel**
- `adapters/adapters_nifs` → **Separate** (NIF API)
- `adapters/adapters_nif_io` → **Separate** (NIF API)

**Rationale**: External interfaces are language-agnostic. NIF adapters are separate.

### 2.5 Frameworks Layer - All Stay in Kernel

All Frameworks layer crates remain in the kernel:

- `frameworks/frameworks_utilities` → **Kernel**
- `frameworks/frameworks_emulator_init` → **Kernel**
- `frameworks/frameworks_system_integration_win32` → **Kernel**
- `frameworks/frameworks_system_integration_unix` → **Kernel**
- `frameworks/frameworks_system_integration_common` → **Kernel**
- `frameworks/frameworks_system_integration` → **Kernel**

**Rationale**: System integration is platform-specific but language-agnostic.

### 2.6 Code Management Layer - All Stay in Kernel

- `code_management/code_management_code_loading` → **Kernel**

**Rationale**: BEAM module loading is language-agnostic.

### 2.7 API Facades Layer - Kernel

- `api_facades` → **Kernel** (kernel-facing APIs)

**Rationale**: Kernel needs APIs for language implementations.

## 3. Language Behavior Framework

### 3.1 Concept

Similar to OTP behaviors (gen_server, gen_statem, etc.), the Language Behavior Framework provides:
- Common implementation for language tools
- Trait interfaces (behaviors) that languages must implement
- Language developers implement callbacks, framework handles common parts

### 3.2 Language Behavior Traits

#### 3.2.1 Shell Behavior

```rust
pub trait LanguageShell: Send + Sync {
    /// Parse a line of input into an expression
    fn parse_expression(&self, line: &str) -> Result<Expression, ParseError>;
    
    /// Compile an expression to BEAM code (or interpret)
    fn compile_expression(&self, expr: &Expression) -> Result<BeamCode, CompileError>;
    
    /// Format a term for display
    fn format_term(&self, term: &Term) -> Result<String, FormatError>;
    
    /// Get command history file path
    fn history_file(&self) -> Option<PathBuf>;
    
    /// Get prompt string
    fn prompt(&self) -> String;
}
```

**Common Framework**: REPL loop, command history, tab completion, line editing

**Language Implementation**: Parsing, compilation, formatting

#### 3.2.2 Compiler Integration Behavior

```rust
pub trait LanguageCompiler: Send + Sync {
    /// Find source file for a module
    fn find_source_file(&self, module: &str) -> Result<PathBuf, CompileError>;
    
    /// Compile source file to BEAM
    fn compile_file(&self, source: &Path) -> Result<PathBuf, CompileError>;
    
    /// Get compiler executable path
    fn compiler_path(&self) -> Option<PathBuf>;
    
    /// Get default code paths
    fn default_code_paths(&self) -> Vec<PathBuf>;
}
```

**Common Framework**: Code path management, module discovery, compilation orchestration

**Language Implementation**: Compiler invocation, source file location

#### 3.2.3 Application Behavior

```rust
pub trait LanguageApplication: Send + Sync {
    /// Load application specification
    fn load_app_spec(&self, app_name: &str) -> Result<AppSpec, AppError>;
    
    /// Get application callback module name
    fn app_callback_module(&self, app_name: &str) -> String;
    
    /// Get application start function
    fn app_start_function(&self) -> (Atom, Atom);
    
    /// Get preloaded modules
    fn preloaded_modules(&self) -> Vec<String>;
}
```

**Common Framework**: Application lifecycle, supervisor tree, boot script parsing

**Language Implementation**: Application structure, callback conventions

#### 3.2.4 Distribution Behavior

```rust
pub trait LanguageDistribution: Send + Sync {
    /// Get node name format
    fn node_name_format(&self) -> NodeNameFormat;
    
    /// Validate node name
    fn validate_node_name(&self, name: &str) -> Result<(), DistError>;
    
    /// Get distribution cookie
    fn get_cookie(&self) -> Option<String>;
    
    /// Handle distribution handshake
    fn handle_handshake(&self, peer: &NodeInfo) -> Result<(), DistError>;
}
```

**Common Framework**: Distribution protocol, connection management, message routing

**Language Implementation**: Node naming conventions, cookie handling

#### 3.2.5 BIF Registration Behavior

```rust
pub trait LanguageBifs: Send + Sync {
    /// Register language-specific BIFs
    fn register_bifs(&self, kernel: &mut Kernel) -> Result<(), BifError>;
    
    /// Get language-specific BIF module
    fn bif_module(&self) -> Atom;
}
```

**Common Framework**: BIF dispatcher, BIF registration, error handling

**Language Implementation**: Language-specific BIF implementations

#### 3.2.6 Initialization Behavior

```rust
pub trait LanguageInit: Send + Sync {
    /// Perform language-specific initialization
    fn initialize(&self, kernel: &mut Kernel) -> Result<(), InitError>;
    
    /// Get language name
    fn language_name(&self) -> &str;
    
    /// Get language version
    fn language_version(&self) -> &str;
    
    /// Configure kernel for this language
    fn configure_kernel(&self, config: &mut KernelConfig) -> Result<(), InitError>;
}
```

**Common Framework**: Kernel initialization orchestration, configuration management

**Language Implementation**: Language-specific setup, preloaded modules

### 3.3 Language Implementation Structure

A language implementation would look like:

```rust
pub struct ErlangLanguage {
    // Language-specific state
}

impl LanguageShell for ErlangLanguage {
    fn parse_expression(&self, line: &str) -> Result<Expression, ParseError> {
        // Erlang-specific parsing
    }
    // ... other callbacks
}

impl LanguageCompiler for ErlangLanguage {
    // Erlang-specific compiler integration
}

impl LanguageApplication for ErlangLanguage {
    // Erlang-specific application handling
}

// ... implement other behaviors

// Main language struct that combines all behaviors
pub struct ErlangLanguageImpl {
    shell: Box<dyn LanguageShell>,
    compiler: Box<dyn LanguageCompiler>,
    application: Box<dyn LanguageApplication>,
    distribution: Box<dyn LanguageDistribution>,
    bifs: Box<dyn LanguageBifs>,
    init: Box<dyn LanguageInit>,
}
```

### 3.4 Common Framework Crates

New crates in kernel for common language tool functionality:

- `language_behaviors/shell_framework` - Common REPL/shell framework
- `language_behaviors/compiler_framework` - Common compiler integration
- `language_behaviors/application_framework` - Common application lifecycle
- `language_behaviors/distribution_framework` - Common distribution handling
- `language_behaviors/bif_framework` - Common BIF registration
- `language_behaviors/init_framework` - Common initialization

## 4. Kernel API Design

### 4.1 Purpose

The Kernel API provides a standardized interface for language implementations to interact with the kernel. This API is defined in `kernel_api_design.md`.

### 4.2 API Location

The Kernel API will be exposed through:
- A new `beam_kernel_api` crate in the kernel
- Language implementations use this API instead of directly calling kernel internals

### 4.3 Migration Path

1. Create `beam_kernel_api` crate
2. Define API interface (see `kernel_api_design.md`)
3. Implement API by wrapping existing kernel functionality
4. Create language behavior framework crates
5. Implement Erlang language behaviors
6. Update Erl tool to use behaviors instead of direct implementation

## 5. Dependencies After Partitioning

### 5.1 Kernel Dependencies

The kernel will have:
- All Entities, Infrastructure, Use Cases, Adapters, Frameworks, Code Management crates
- `beam_kernel_api` crate (public API)
- `language_behaviors/*` crates (common frameworks)
- No dependencies on specific language implementations

### 5.2 Language Implementation Dependencies

A language implementation (e.g., Erl tool) will have:
- Dependency on `beam_kernel_api` (kernel API)
- Dependency on `language_behaviors/*` (common frameworks)
- Implementation of language behavior traits
- No direct dependencies on kernel internals

### 5.3 NIF API Dependencies

NIF API (separate) will have:
- Dependency on `beam_kernel_api` (kernel API)
- NIF-specific crates
- No dependencies on language implementations

## 6. Implementation Steps

### 6.1 Phase 1: Create Kernel API

1. Create `beam_kernel_api` crate
2. Define API interface (see `kernel_api_design.md`)
3. Implement API by wrapping existing functionality
4. Add tests for API

### 6.2 Phase 2: Create Language Behavior Framework

1. Create `language_behaviors` workspace
2. Define behavior traits for each area (shell, compiler, application, etc.)
3. Implement common frameworks for each behavior
4. Add tests for frameworks

### 6.3 Phase 3: Implement Erlang Language Behaviors

1. Create `erl_language` crate
2. Implement `LanguageShell` for Erlang
3. Implement `LanguageCompiler` for Erlang
4. Implement `LanguageApplication` for Erlang
5. Implement `LanguageDistribution` for Erlang
6. Implement `LanguageBifs` for Erlang
7. Implement `LanguageInit` for Erlang
8. Create Erl tool that uses behaviors

### 6.4 Phase 4: Refactor and Test

1. Remove language-specific code from kernel
2. Ensure kernel has no language-specific dependencies
3. Test Erlang implementation using behaviors
4. Test compatibility with existing BEAM files
5. Test distribution between kernel instances

## 7. File Structure After Partitioning

```
rust-conversion/
├── rust/
│   └── beam-kernel/          # Kernel workspace
│       ├── entities/         # All entities crates
│       ├── infrastructure/   # All infrastructure crates (except NIF)
│       ├── usecases/         # All usecases crates (except NIF)
│       ├── adapters/         # All adapters crates (except NIF)
│       ├── frameworks/       # All frameworks crates
│       ├── code_management/  # All code management crates
│       ├── beam_kernel_api/  # Kernel API crate
│       └── language_behaviors/  # Language behavior framework
│           ├── shell_framework/
│           ├── compiler_framework/
│           ├── application_framework/
│           ├── distribution_framework/
│           ├── bif_framework/
│           └── init_framework/
│
├── erl-tool/                  # Erlang language implementation
│   ├── erl_language/        # Erlang behavior implementations
│   └── erl_cli/              # Command-line interface using behaviors
│
└── nif-api/                   # NIF API workspace (separate)
    └── ...                    # NIF-related crates
```

## 8. Benefits of Behavior Pattern

### 8.1 Code Reuse

- Common functionality implemented once in framework
- Languages only implement callbacks
- Less duplication, easier maintenance

### 8.2 Consistency

- All languages follow same patterns
- Common behavior across languages
- Easier to understand and maintain

### 8.3 Extensibility

- Easy to add new languages
- Just implement behavior traits
- Framework handles common parts

### 8.4 Testing

- Test frameworks independently
- Test language implementations independently
- Clear separation of concerns

## 9. Example: Shell Implementation

### 9.1 Framework (Common)

```rust
pub struct ShellFramework {
    kernel: Arc<Mutex<Kernel>>,
}

impl ShellFramework {
    pub fn run<L: LanguageShell>(&mut self, language: &L) -> Result<(), ShellError> {
        loop {
            let prompt = language.prompt();
            let line = self.read_line(&prompt)?;
            
            if line.trim().is_empty() {
                continue;
            }
            
            // Use language callback to parse
            let expr = language.parse_expression(&line)?;
            
            // Use language callback to compile
            let code = language.compile_expression(&expr)?;
            
            // Execute in kernel
            let result = self.execute(&code)?;
            
            // Use language callback to format
            let formatted = language.format_term(&result)?;
            println!("{}", formatted);
        }
    }
}
```

### 9.2 Erlang Implementation (Callbacks)

```rust
impl LanguageShell for ErlangLanguage {
    fn parse_expression(&self, line: &str) -> Result<Expression, ParseError> {
        // Erlang-specific parsing
        erl_parse::parse_expr(line)
    }
    
    fn compile_expression(&self, expr: &Expression) -> Result<BeamCode, CompileError> {
        // Erlang-specific compilation
        erl_compile::compile_expr(expr)
    }
    
    fn format_term(&self, term: &Term) -> Result<String, FormatError> {
        // Erlang-specific formatting
        erl_io::format_term(term)
    }
    
    fn prompt(&self) -> String {
        "1> ".to_string()
    }
}
```

## 10. Backward Compatibility

### 10.1 BEAM File Compatibility

- Kernel must maintain compatibility with existing BEAM files
- No changes to BEAM file format
- Module loading must work as before

### 10.2 External Term Format

- Kernel must maintain compatibility with external term format
- No changes to encoding/decoding

### 10.3 Distribution Protocol

- Kernel must maintain compatibility with Erlang distribution protocol
- Language behaviors implement language-specific conventions on top

## 11. Related Documents

- `kernel_api_design.md` - Detailed API specification
- `../nif_isolation/` - NIF API design
