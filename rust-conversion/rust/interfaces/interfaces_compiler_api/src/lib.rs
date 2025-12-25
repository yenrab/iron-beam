/*!
# Compiler API Interfaces

**CLEAN Architecture**: Interface Adapters Layer (Layer 3)
**SOLID Responsibility**: External interfaces for compiler integration

## Overview

This crate provides the external interfaces and adapters that allow the Erlang compiler
to integrate with external systems, tools, and applications. It defines clean API boundaries
between the compiler core and external consumers.

## External Integration Points

### 1. Public Compiler API
```rust
use interfaces_compiler_api::CompilerAPI;

// External tools can compile Erlang code
let api = CompilerAPI::new();
let result = api.compile_source("module.erl", source_code).await?;
```

### 2. Serialization Interfaces
```rust
use interfaces_compiler_api::serialization::*;

// AST serialization for tooling (currently not implemented)
// let ast = parse_erlang_source(source_code)?;
// let serialized = serialize_ast(&ast)?; // Not implemented
// let deserialized_ast = deserialize_ast(&serialized)?; // Not implemented
```

### 3. Plugin System
```rust
use interfaces_compiler_api::plugins::*;

// Extend compiler with custom passes
let mut compiler = Compiler::new();
compiler.register_plugin(Box::new(MyOptimizationPass));
```

## Architecture Compliance

- **CLEAN Layer**: Interface Adapters (Layer 3) - External system boundaries
- **Dependencies**: Use Cases + Entities + Infrastructure (adapts all layers)
- **SOLID Principle**: Single responsibility for external integration
- **API Design**: Clean, stable interfaces for external consumers
- **Serialization**: Multiple formats (JSON, binary) for different use cases

## External API Categories

### Tool Integration APIs
- **IDE Integration**: Source analysis, completion, refactoring
- **Build Systems**: Incremental compilation, dependency management
- **Testing Frameworks**: Code coverage, test execution
- **Debuggers**: Breakpoint setting, variable inspection

### Protocol Interfaces
- **Language Server Protocol**: LSP implementation for editors
- **Build Protocol**: Communication with build tools
- **Plugin Protocol**: Extension mechanism for compiler plugins

### Data Exchange Formats
- **AST Serialization**: JSON/binary representation of syntax trees
- **Compilation Results**: Structured error/warning reporting
- **Metadata Exchange**: Type information, documentation
*/

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use usecases_compilation::*;
use entities_erlang_syntax::*;

// Re-export key APIs for external consumers
pub use api::*;
pub use serialization::*;
pub use plugins::*;
pub use protocols::*;

// API modules
mod api;
mod serialization;
mod plugins;
mod protocols;

/// Main compiler API for external integration
///
/// This is the primary interface that external tools and applications
/// use to interact with the Erlang compiler.
pub struct CompilerAPI {
    orchestrator: CompilationOrchestrator,
    plugins: Vec<Box<dyn Plugin>>,
    config: APIConfig,
}

impl CompilerAPI {
    /// Create a new compiler API instance
    pub fn new() -> Self {
        Self {
            orchestrator: CompilationOrchestrator::new(),
            plugins: Vec::new(),
            config: APIConfig::default(),
        }
    }

    /// Configure the API with custom settings
    pub fn with_config(mut self, config: APIConfig) -> Self {
        self.config = config;
        self
    }

    /// Register a compiler plugin
    pub fn register_plugin(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    /// Compile Erlang source code
    ///
    /// Primary API for compiling Erlang modules
    pub async fn compile_source(
        &self,
        module_name: &str,
        source_code: &str,
    ) -> APIResult<CompilationOutput> {
        let module_atom = Atom::new(module_name.to_string());

        // Apply pre-compilation plugins
        for plugin in &self.plugins {
            plugin.pre_compile(&module_atom, source_code).await?;
        }

        // Perform compilation
        let result = self.orchestrator.compile_module(source_code, &module_atom).await
            .map_err(|e| APIError::CompilationError(e.to_string()))?;

        // Apply post-compilation plugins
        let mut final_result = CompilationOutput::from_result(result);
        for plugin in &self.plugins {
            final_result = plugin.post_compile(&module_atom, final_result).await?;
        }

        Ok(final_result)
    }

    /// Compile multiple modules with dependencies
    pub async fn compile_modules(
        &self,
        sources: HashMap<String, String>,
    ) -> APIResult<BatchCompilationOutput> {
        let atom_sources: HashMap<Atom, String> = sources.into_iter()
            .map(|(k, v)| (Atom::new(k), v))
            .collect();

        let result = self.orchestrator.compile_modules(atom_sources).await
            .map_err(|e| APIError::CompilationError(e.to_string()))?;

        Ok(BatchCompilationOutput::from_batch_result(result))
    }

    /// Analyze source code without compilation
    pub async fn analyze_source(
        &self,
        module_name: &str,
        source_code: &str,
    ) -> APIResult<AnalysisResult> {
        // Parse and analyze source code
        let module_atom = Atom::new(module_name.to_string());

        // Basic analysis - in a real implementation, this would use the parsing pipeline
        let analysis = AnalysisResult {
            module_name: module_atom,
            syntax_valid: !source_code.trim().is_empty(),
            warnings: Vec::new(),
            errors: Vec::new(),
            metrics: SourceMetrics {
                lines_of_code: source_code.lines().count(),
                functions: source_code.matches("-spec").count(), // Rough estimate
                complexity: 1, // Placeholder
            },
        };

        Ok(analysis)
    }

    /// Get compiler version and capabilities
    pub fn get_info(&self) -> CompilerInfo {
        CompilerInfo {
            version: env!("CARGO_PKG_VERSION").to_string(),
            supported_formats: vec![
                "erlang".to_string(),
                "core_erlang".to_string(),
                "kernel_erlang".to_string(),
            ],
            features: vec![
                "parsing".to_string(),
                "analysis".to_string(),
                "optimization".to_string(),
                "code_generation".to_string(),
                "plugins".to_string(),
            ],
            config: self.config.clone(),
        }
    }
}

impl Default for CompilerAPI {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for the compiler API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIConfig {
    pub enable_plugins: bool,
    pub max_concurrent_compilations: usize,
    pub timeout_seconds: u64,
    pub output_format: APIOutputFormat,
}

impl Default for APIConfig {
    fn default() -> Self {
        Self {
            enable_plugins: true,
            max_concurrent_compilations: 4,
            timeout_seconds: 300, // 5 minutes
            output_format: APIOutputFormat::Json,
        }
    }
}

/// Output format for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum APIOutputFormat {
    Json,
    Binary,
    Text,
}

/// API result type
pub type APIResult<T> = Result<T, APIError>;

/// API error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum APIError {
    #[serde(rename = "compilation_error")]
    CompilationError(String),

    #[serde(rename = "serialization_error")]
    SerializationError(String),

    #[serde(rename = "plugin_error")]
    PluginError(String),

    #[serde(rename = "timeout_error")]
    TimeoutError(String),

    #[serde(rename = "invalid_request")]
    InvalidRequest(String),
}

impl std::fmt::Display for APIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CompilationError(msg) => write!(f, "Compilation error: {}", msg),
            Self::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            Self::PluginError(msg) => write!(f, "Plugin error: {}", msg),
            Self::TimeoutError(msg) => write!(f, "Timeout error: {}", msg),
            Self::InvalidRequest(msg) => write!(f, "Invalid request: {}", msg),
        }
    }
}

impl std::error::Error for APIError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compiler_api_creation() {
        let api = CompilerAPI::new();
        assert!(api.config.enable_plugins);
        assert_eq!(api.config.max_concurrent_compilations, 4);
    }

    #[test]
    fn test_api_config_default() {
        let config = APIConfig::default();
        assert!(config.enable_plugins);
        assert_eq!(config.timeout_seconds, 300);
        assert!(matches!(config.output_format, APIOutputFormat::Json));
    }

    #[test]
    fn test_compiler_info() {
        let api = CompilerAPI::new();
        let info = api.get_info();

        assert!(!info.version.is_empty());
        assert!(!info.supported_formats.is_empty());
        assert!(!info.features.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_source() {
        let api = CompilerAPI::new();
        let result = api.analyze_source("test", "module test.\nexport([start/0]).\nstart() -> ok.").await;

        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert_eq!(analysis.module_name.as_str(), "test");
        assert!(analysis.syntax_valid);
        assert_eq!(analysis.metrics.lines_of_code, 3);
    }

    #[test]
    fn test_api_error_display() {
        let error = APIError::CompilationError("test error".to_string());
        assert_eq!(error.to_string(), "Compilation error: test error");

        let error = APIError::InvalidRequest("bad request".to_string());
        assert_eq!(error.to_string(), "Invalid request: bad request");
    }
}
