/*!
# Compilation Use Cases

**CLEAN Architecture**: Use Cases Layer (Layer 2)
**SOLID Responsibility**: Compilation business logic and workflows

## Overview

This crate implements the core compilation business logic that orchestrates the transformation
of Erlang source code into BEAM bytecode. It coordinates between the Entity layer (AST structures)
and the Infrastructure layer (system services).

## Compilation Pipeline

The compilation process follows these phases:
1. **Parsing**: Source text → AST (handled by entities)
2. **Analysis**: AST validation and semantic analysis
3. **Optimization**: Code optimization passes
4. **Code Generation**: AST → BEAM bytecode
5. **Linking**: Module linking and validation

## Business Logic Components

### 1. Compilation Orchestrator
```rust
use usecases_compilation::CompilationOrchestrator;

// Main compilation entry point
let orchestrator = CompilationOrchestrator::new();
// Async compilation would be used in real code:
// let result = orchestrator.compile_module("source", &"module".into()).await?;
```

### 2. Pipeline Management
```rust
use usecases_compilation::*;

// Configurable compilation pipeline
let pipeline = CompilationPipeline::new();
// Passes would be added in real code:
// .add_pass(Box::new(AnalysisPass))
// .add_pass(Box::new(OptimizationPass))
// .add_pass(Box::new(CodeGenerationPass))
```

### 3. Error Recovery
```rust
use usecases_compilation::CompilationResult;

// Robust error handling with recovery
let result = CompilationResult {
    module_name: "test".into(),
    bytecode: vec![],
    warnings: vec![],
    metadata: Default::default(),
};

println!("Compiled module: {}", result.module_name.as_str());
println!("Warnings: {}", result.warnings.len());
```

## Architecture Compliance

- **CLEAN Layer**: Use Cases (Layer 2) - Business logic orchestration
- **Dependencies**: Entities + Infrastructure (no outward dependencies)
- **SOLID Principle**: Single responsibility for compilation workflows
- **Error Handling**: Comprehensive error recovery and reporting
- **Extensibility**: Pluggable compilation passes and strategies

## Integration Points

- **Entities**: Consumes AST structures, produces compilation artifacts
- **Infrastructure**: Uses all infrastructure services (file I/O, process execution, etc.)
- **Future Adapters**: Will interface with external tools and frameworks
*/

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use entities_erlang_syntax::*;
use infrastructure_error_handling::{CompilerError, CompilerResult};
use infrastructure_path_handling::path;
use infrastructure_utilities::{erl_parse, erl_scan};

// Re-export key types for convenience
pub use compilation::*;
pub use pipeline::*;
pub use passes::*;
pub use results::*;

// Compilation modules
mod compilation;
mod pipeline;
mod passes;
mod results;

/// Main compilation orchestrator - the primary use case interface
pub struct CompilationOrchestrator {
    pipeline: CompilationPipeline,
    options: CompilationOptions,
}

impl CompilationOrchestrator {
    /// Create a new compilation orchestrator with default settings
    pub fn new() -> Self {
        Self {
            pipeline: CompilationPipeline::default(),
            options: CompilationOptions::default(),
        }
    }

    /// Configure the compilation pipeline
    pub fn with_pipeline(mut self, pipeline: CompilationPipeline) -> Self {
        self.pipeline = pipeline;
        self
    }

    /// Configure compilation options
    pub fn with_options(mut self, options: CompilationOptions) -> Self {
        self.options = options;
        self
    }

    /// Compile a single Erlang module from source text
    pub async fn compile_module(
        &self,
        source_text: &str,
        module_name: &Atom,
    ) -> CompilerResult<CompilationResult> {
        // This would normally parse the source text into AST
        // For now, we'll simulate the compilation process

        let context = CompilationContext {
            module_name: module_name.clone(),
            source_text: source_text.to_string(),
            ast: None,
            options: self.options.clone(),
            metadata: HashMap::new(),
        };

        self.pipeline.execute(context).await
    }

    /// Compile multiple modules with dependency resolution
    pub async fn compile_modules(
        &self,
        sources: HashMap<Atom, String>,
    ) -> CompilerResult<BatchCompilationResult> {
        let mut results = HashMap::new();
        let mut errors = Vec::new();

        // Simple sequential compilation (could be parallelized)
        for (module_name, source_text) in sources {
            match self.compile_module(&source_text, &module_name).await {
                Ok(result) => {
                    results.insert(module_name, result);
                }
                Err(err) => {
                    errors.push((module_name, err));
                }
            }
        }

        Ok(BatchCompilationResult { results, errors })
    }

    /// Validate module dependencies
    pub fn validate_dependencies(&self, modules: &HashMap<Atom, Module>) -> CompilerResult<DependencyGraph> {
        // Analyze module imports and exports to build dependency graph
        let mut graph = DependencyGraph::new();

        for (name, module) in modules {
            let deps = self.extract_dependencies(module)?;
            graph.add_module(name.clone(), deps);
        }

        graph.validate_no_cycles()?;
        Ok(graph)
    }

    /// Extract dependencies from a module
    fn extract_dependencies(&self, module: &Module) -> CompilerResult<Vec<Atom>> {
        let mut deps = Vec::new();

        // Check module attributes for imports
        for attr in &module.attributes {
            if let AttributeValue::Import(module_name, _) = &attr.value {
                deps.push(module_name.clone());
            }
        }

        // Check function bodies for external calls (simplified)
        for function in &module.functions {
            for clause in &function.clauses {
                deps.extend(self.extract_function_dependencies(&clause.body));
            }
        }

        // Remove duplicates
        deps.sort();
        deps.dedup();
        Ok(deps)
    }

    /// Extract external module dependencies from expressions
    fn extract_function_dependencies(&self, expressions: &[Expression]) -> Vec<Atom> {
        let mut deps = Vec::new();

        for expr in expressions {
            match expr {
                Expression::FunctionCall(call) => {
                    if let Some(module) = &call.module {
                        deps.push(module.clone());
                    }
                }
                Expression::Case(case) => {
                    deps.extend(self.extract_function_dependencies(&[case.expression.as_ref().clone()]));
                    for clause in &case.clauses {
                        deps.extend(self.extract_function_dependencies(&clause.body));
                    }
                }
                Expression::If(if_expr) => {
                    for clause in &if_expr.clauses {
                        deps.extend(self.extract_function_dependencies(&clause.body));
                    }
                }
                // Add other expression types as needed
                _ => {}
            }
        }

        deps
    }
}

/// Compilation options and configuration
#[derive(Debug, Clone, PartialEq, Eq, )]
pub struct CompilationOptions {
    pub optimization_level: OptimizationLevel,
    pub warnings: bool,
    pub debug_info: bool,
    pub target_platform: Option<String>,
    pub output_format: OutputFormat,
}

impl Default for CompilationOptions {
    fn default() -> Self {
        Self {
            optimization_level: OptimizationLevel::default(),
            warnings: true,
            debug_info: false,
            target_platform: None,
            output_format: OutputFormat::Beam,
        }
    }
}

/// Optimization levels
#[derive(Debug, Clone, PartialEq, Eq, )]
pub enum OptimizationLevel {
    None,
    Basic,
    Standard,
    Aggressive,
}

impl Default for OptimizationLevel {
    fn default() -> Self {
        Self::Standard
    }
}

/// Output formats for compilation
#[derive(Debug, Clone, PartialEq, Eq, )]
pub enum OutputFormat {
    Beam,      // BEAM bytecode
    Asm,       // Assembly listing
    Core,      // Core Erlang
    Kernel,    // Kernel Erlang
}

/// Compilation context passed through the pipeline
#[derive(Debug, Clone)]
pub struct CompilationContext {
    pub module_name: Atom,
    pub source_text: String,
    pub ast: Option<entities_erlang_syntax::Module>,
    pub options: CompilationOptions,
    pub metadata: HashMap<String, String>,
}

impl CompilationContext {
    pub fn new(module_name: Atom, source_text: String) -> Self {
        Self {
            module_name,
            source_text,
            ast: None,
            options: CompilationOptions::default(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_options(mut self, options: CompilationOptions) -> Self {
        self.options = options;
        self
    }

    pub fn add_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// Dependency graph for module compilation order
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    pub modules: HashMap<Atom, Vec<Atom>>, // module -> dependencies
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    pub fn add_module(&mut self, module: Atom, dependencies: Vec<Atom>) {
        self.modules.insert(module, dependencies);
    }

    pub fn validate_no_cycles(&self) -> CompilerResult<()> {
        // Simple cycle detection (could be more sophisticated)
        for (module, deps) in &self.modules {
            if deps.contains(module) {
                return Err(CompilerError::InvalidArgument(
                    format!("Module {} has a self-dependency", module)
                ));
            }

            // Check for direct cycles
            for dep in deps {
                if let Some(dep_deps) = self.modules.get(dep) {
                    if dep_deps.contains(module) {
                        return Err(CompilerError::InvalidArgument(
                            format!("Circular dependency between {} and {}", module, dep)
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    pub fn get_compilation_order(&self) -> Vec<Atom> {
        // Simple topological sort (could be more sophisticated)
        let mut order = Vec::new();
        let mut visited = std::collections::HashSet::new();

        for module in self.modules.keys() {
            if !visited.contains(module) {
                self.visit_module(module, &mut visited, &mut order);
            }
        }

        order
    }

    fn visit_module(&self, module: &Atom, visited: &mut std::collections::HashSet<Atom>, order: &mut Vec<Atom>) {
        if visited.contains(module) {
            return;
        }

        visited.insert(module.clone());

        if let Some(deps) = self.modules.get(module) {
            for dep in deps {
                self.visit_module(dep, visited, order);
            }
        }

        order.push(module.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compilation_orchestrator_creation() {
        let orchestrator = CompilationOrchestrator::new();
        assert_eq!(orchestrator.options.optimization_level, OptimizationLevel::Standard);
        assert!(orchestrator.options.warnings);
    }

    #[test]
    fn test_compilation_options_default() {
        let options = CompilationOptions::default();
        assert_eq!(options.optimization_level, OptimizationLevel::Standard);
        assert!(options.warnings);
        assert!(!options.debug_info);
        assert_eq!(options.output_format, OutputFormat::Beam);
    }

    #[test]
    fn test_dependency_graph_simple() {
        let mut graph = DependencyGraph::new();

        let module_a = Atom::new("module_a");
        let module_b = Atom::new("module_b");

        graph.add_module(module_a.clone(), vec![module_b.clone()]);
        graph.add_module(module_b.clone(), vec![]);

        assert!(graph.validate_no_cycles().is_ok());

        let order = graph.get_compilation_order();
        // module_b should come before module_a
        assert_eq!(order.len(), 2);
        assert!(order.iter().position(|m| m == &module_b).unwrap() <
                order.iter().position(|m| m == &module_a).unwrap());
    }

    #[test]
    fn test_dependency_graph_cycle_detection() {
        let mut graph = DependencyGraph::new();

        let module_a = Atom::new("module_a");
        let module_b = Atom::new("module_b");

        graph.add_module(module_a.clone(), vec![module_b.clone()]);
        graph.add_module(module_b.clone(), vec![module_a.clone()]);

        assert!(graph.validate_no_cycles().is_err());
    }

    #[test]
    fn test_compilation_context() {
        let context = CompilationContext::new(
            Atom::new("test_module"),
            "test source".to_string(),
        );

        assert_eq!(context.module_name.as_str(), "test_module");
        assert_eq!(context.source_text, "test source");
        assert_eq!(context.options.optimization_level, OptimizationLevel::Standard);
    }

    #[tokio::test]
    async fn test_compile_module_simulation() {
        let orchestrator = CompilationOrchestrator::new();

        // This will currently fail because we haven't implemented the full pipeline
        // but it tests that the orchestrator interface works
        let result = orchestrator.compile_module("dummy source", &Atom::new("test")).await;

        // Now the pipeline is implemented (mock), so it should succeed
        assert!(result.is_ok());
    }
}
