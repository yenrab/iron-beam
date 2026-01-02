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
let ast = entities_erlang_syntax::Module::new(entities_erlang_syntax::Atom::new("test".to_string()));
let result = CompilationResult {
    module_name: entities_erlang_syntax::Atom::new("test".to_string()),
    ast,
    bytecode: vec![],
    warnings: vec![],
    metadata: Default::default(),
    context_metadata: std::collections::HashMap::new(),
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

/// Result of compiling multiple modules
#[derive(Debug)]
pub struct BatchCompilationResult {
    pub results: HashMap<Atom, CompilationResult>,
    pub errors: Vec<(Atom, CompilerError)>,
}

impl BatchCompilationResult {
    /// Get the number of successfully compiled modules
    pub fn success_count(&self) -> usize {
        self.results.len()
    }

    /// Get the number of failed compilations
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Get the total number of modules processed
    pub fn total_count(&self) -> usize {
        self.results.len() + self.errors.len()
    }
}

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
        use std::collections::HashSet;

        // Check for self-dependencies first
        for (module, deps) in &self.modules {
            if deps.contains(module) {
                return Err(CompilerError::InvalidArgument(
                    format!("Module {} has a self-dependency", module)
                ));
            }
        }

        // Use DFS to detect cycles in the dependency graph
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        fn has_cycle(
            module: &Atom,
            graph: &HashMap<Atom, Vec<Atom>>,
            visited: &mut HashSet<Atom>,
            rec_stack: &mut HashSet<Atom>,
        ) -> bool {
            visited.insert(module.clone());
            rec_stack.insert(module.clone());

            if let Some(deps) = graph.get(module) {
                for dep in deps {
                    if !visited.contains(dep) && has_cycle(dep, graph, visited, rec_stack) {
                        return true;
                    } else if rec_stack.contains(dep) {
                        return true;
                    }
                }
            }

            rec_stack.remove(module);
            false
        }

        for module in self.modules.keys() {
            if !visited.contains(module) {
                if has_cycle(module, &self.modules, &mut visited, &mut rec_stack) {
                    return Err(CompilerError::InvalidArgument(
                        "Circular dependency detected in module dependencies".to_string()
                    ));
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
    use std::collections::HashMap;

    // ==================== CompilationOrchestrator Tests ====================

    #[test]
    fn test_compilation_orchestrator_creation() {
        let orchestrator = CompilationOrchestrator::new();
        assert_eq!(orchestrator.options.optimization_level, OptimizationLevel::Standard);
        assert!(orchestrator.options.warnings);
        assert!(!orchestrator.options.debug_info);
        assert_eq!(orchestrator.options.output_format, OutputFormat::Beam);
        assert!(orchestrator.options.target_platform.is_none());
    }

    #[test]
    fn test_compilation_orchestrator_with_custom_pipeline() {
        let custom_pipeline = CompilationPipeline::new();
        let orchestrator = CompilationOrchestrator::new()
            .with_pipeline(custom_pipeline);

        // Pipeline should be replaced
        assert_eq!(orchestrator.pipeline.pass_count(), 0);
    }

    #[test]
    fn test_compilation_orchestrator_with_custom_options() {
        let custom_options = CompilationOptions {
            optimization_level: OptimizationLevel::Aggressive,
            warnings: false,
            debug_info: true,
            target_platform: Some("arm64".to_string()),
            output_format: OutputFormat::Core,
        };

        let orchestrator = CompilationOrchestrator::new()
            .with_options(custom_options.clone());

        assert_eq!(orchestrator.options.optimization_level, OptimizationLevel::Aggressive);
        assert!(!orchestrator.options.warnings);
        assert!(orchestrator.options.debug_info);
        assert_eq!(orchestrator.options.target_platform, Some("arm64".to_string()));
        assert_eq!(orchestrator.options.output_format, OutputFormat::Core);
    }

    #[tokio::test]
    async fn test_compile_module_simulation() {
        let orchestrator = CompilationOrchestrator::new();

        let source = "-module(test).\n-export([]).\n";
        let result = orchestrator.compile_module(source, &Atom::new("test")).await;

        // The pipeline is implemented (mock), so it should succeed
        assert!(result.is_ok());

        let compilation_result = result.unwrap();
        assert_eq!(compilation_result.module_name.as_str(), "test");
        // source_text is not stored in CompilationResult
        assert!(compilation_result.warnings.is_empty());
        assert_eq!(compilation_result.metadata.optimization_level, OptimizationLevel::Standard);
    }

    #[tokio::test]
    async fn test_compile_module_with_custom_options() {
        let custom_options = CompilationOptions {
            optimization_level: OptimizationLevel::None,
            warnings: false,
            debug_info: true,
            target_platform: Some("x86_64".to_string()),
            output_format: OutputFormat::Asm,
        };

        let orchestrator = CompilationOrchestrator::new()
            .with_options(custom_options);

        let source = "-module(custom_test).\n-export([test/0]).\ntest() -> ok.\n";
        let result = orchestrator.compile_module(source, &Atom::new("custom_test")).await;

        assert!(result.is_ok());
        let compilation_result = result.unwrap();
        assert_eq!(compilation_result.metadata.optimization_level, OptimizationLevel::None);
        // Note: warnings and output_format are not stored in CompilationResult
        // They are only used during compilation process
    }

    #[tokio::test]
    async fn test_compile_modules_empty() {
        let orchestrator = CompilationOrchestrator::new();
        let sources = HashMap::new();
        let result = orchestrator.compile_modules(sources).await;

        assert!(result.is_ok());
        let report = result.unwrap();
        assert_eq!(report.results.len(), 0);
        assert_eq!(report.errors.len(), 0);
    }

    #[tokio::test]
    async fn test_compile_modules_single() {
        let orchestrator = CompilationOrchestrator::new();
        let mut sources = HashMap::new();
        sources.insert(Atom::new("single_test"), "-module(single_test).\n-export([]).\n".to_string());

        let result = orchestrator.compile_modules(sources).await;

        assert!(result.is_ok());
        let report = result.unwrap();
        assert_eq!(report.results.len(), 1);
        let first_result = report.results.values().next().unwrap();
        assert_eq!(first_result.module_name.as_str(), "single_test");
    }

    #[tokio::test]
    async fn test_compile_modules_multiple() {
        let orchestrator = CompilationOrchestrator::new();
        let mut sources = HashMap::new();
        sources.insert(Atom::new("mod1"), "-module(mod1).\n-export([]).\n".to_string());
        sources.insert(Atom::new("mod2"), "-module(mod2).\n-export([]).\n".to_string());
        sources.insert(Atom::new("mod3"), "-module(mod3).\n-export([]).\n".to_string());

        let result = orchestrator.compile_modules(sources).await;

        assert!(result.is_ok());
        let report = result.unwrap();
        assert_eq!(report.results.len(), 3);

        // Check that all modules are present
        let module_names: Vec<&str> = report.results.values()
            .map(|r| r.module_name.as_str())
            .collect();
        assert!(module_names.contains(&"mod1"));
        assert!(module_names.contains(&"mod2"));
        assert!(module_names.contains(&"mod3"));
    }

    // ==================== CompilationOptions Tests ====================

    #[test]
    fn test_compilation_options_default() {
        let options = CompilationOptions::default();
        assert_eq!(options.optimization_level, OptimizationLevel::Standard);
        assert!(options.warnings);
        assert!(!options.debug_info);
        assert_eq!(options.output_format, OutputFormat::Beam);
        assert!(options.target_platform.is_none());
    }

    #[test]
    fn test_compilation_options_custom() {
        let options = CompilationOptions {
            optimization_level: OptimizationLevel::Aggressive,
            warnings: false,
            debug_info: true,
            target_platform: Some("wasm32".to_string()),
            output_format: OutputFormat::Kernel,
        };

        assert_eq!(options.optimization_level, OptimizationLevel::Aggressive);
        assert!(!options.warnings);
        assert!(options.debug_info);
        assert_eq!(options.target_platform, Some("wasm32".to_string()));
        assert_eq!(options.output_format, OutputFormat::Kernel);
    }

    #[test]
    fn test_compilation_options_clone() {
        let options = CompilationOptions {
            optimization_level: OptimizationLevel::Basic,
            warnings: true,
            debug_info: false,
            target_platform: Some("aarch64".to_string()),
            output_format: OutputFormat::Core,
        };

        let cloned = options.clone();
        assert_eq!(options.optimization_level, cloned.optimization_level);
        assert_eq!(options.warnings, cloned.warnings);
        assert_eq!(options.debug_info, cloned.debug_info);
        assert_eq!(options.target_platform, cloned.target_platform);
        assert_eq!(options.output_format, cloned.output_format);
    }

    // ==================== OptimizationLevel Tests ====================

    #[test]
    fn test_optimization_level_default() {
        assert_eq!(OptimizationLevel::default(), OptimizationLevel::Standard);
    }

    #[test]
    fn test_optimization_level_variants() {
        assert_eq!(OptimizationLevel::None as u8, 0);
        assert_eq!(OptimizationLevel::Basic as u8, 1);
        assert_eq!(OptimizationLevel::Standard as u8, 2);
        assert_eq!(OptimizationLevel::Aggressive as u8, 3);
    }

    #[test]
    fn test_optimization_level_debug() {
        assert_eq!(format!("{:?}", OptimizationLevel::None), "None");
        assert_eq!(format!("{:?}", OptimizationLevel::Basic), "Basic");
        assert_eq!(format!("{:?}", OptimizationLevel::Standard), "Standard");
        assert_eq!(format!("{:?}", OptimizationLevel::Aggressive), "Aggressive");
    }

    #[test]
    fn test_optimization_level_clone() {
        let level = OptimizationLevel::Aggressive;
        let cloned = level.clone();
        assert_eq!(level, cloned);
    }

    // ==================== OutputFormat Tests ====================

    #[test]
    fn test_output_format_variants() {
        assert_eq!(OutputFormat::Beam as u8, 0);
        assert_eq!(OutputFormat::Asm as u8, 1);
        assert_eq!(OutputFormat::Core as u8, 2);
        assert_eq!(OutputFormat::Kernel as u8, 3);
    }

    #[test]
    fn test_output_format_debug() {
        assert_eq!(format!("{:?}", OutputFormat::Beam), "Beam");
        assert_eq!(format!("{:?}", OutputFormat::Asm), "Asm");
        assert_eq!(format!("{:?}", OutputFormat::Core), "Core");
        assert_eq!(format!("{:?}", OutputFormat::Kernel), "Kernel");
    }

    #[test]
    fn test_output_format_clone() {
        let format = OutputFormat::Core;
        let cloned = format.clone();
        assert_eq!(format, cloned);
    }

    // ==================== CompilationContext Tests ====================

    #[test]
    fn test_compilation_context_new() {
        let context = CompilationContext::new(
            Atom::new("test_module"),
            "test source".to_string(),
        );

        assert_eq!(context.module_name.as_str(), "test_module");
        assert_eq!(context.source_text, "test source");
        assert!(context.ast.is_none());
        assert_eq!(context.options.optimization_level, OptimizationLevel::Standard);
        assert!(context.metadata.is_empty());
    }

    #[test]
    fn test_compilation_context_with_options() {
        let custom_options = CompilationOptions {
            optimization_level: OptimizationLevel::None,
            warnings: false,
            debug_info: true,
            target_platform: Some("test_platform".to_string()),
            output_format: OutputFormat::Asm,
        };

        let context = CompilationContext::new(
            Atom::new("test_module"),
            "test source".to_string(),
        ).with_options(custom_options.clone());

        assert_eq!(context.options.optimization_level, OptimizationLevel::None);
        assert!(!context.options.warnings);
        assert!(context.options.debug_info);
        assert_eq!(context.options.target_platform, Some("test_platform".to_string()));
        assert_eq!(context.options.output_format, OutputFormat::Asm);
    }

    #[test]
    fn test_compilation_context_add_metadata() {
        let context = CompilationContext::new(
            Atom::new("test_module"),
            "test source".to_string(),
        ).add_metadata("key1", "value1")
         .add_metadata("key2", "value2");

        assert_eq!(context.metadata.len(), 2);
        assert_eq!(context.metadata.get("key1"), Some(&"value1".to_string()));
        assert_eq!(context.metadata.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_compilation_context_clone() {
        let mut context = CompilationContext::new(
            Atom::new("test_module"),
            "test source".to_string(),
        );
        context.metadata.insert("test_key".to_string(), "test_value".to_string());

        let cloned = context.clone();
        assert_eq!(context.module_name, cloned.module_name);
        assert_eq!(context.source_text, cloned.source_text);
        assert_eq!(context.ast, cloned.ast);
        assert_eq!(context.options.optimization_level, cloned.options.optimization_level);
        assert_eq!(context.metadata, cloned.metadata);
    }

    // ==================== DependencyGraph Tests ====================

    #[test]
    fn test_dependency_graph_new() {
        let graph = DependencyGraph::new();
        assert!(graph.modules.is_empty());
    }

    #[test]
    fn test_dependency_graph_add_module() {
        let mut graph = DependencyGraph::new();

        let module_a = Atom::new("module_a");
        let module_b = Atom::new("module_b");
        let module_c = Atom::new("module_c");

        graph.add_module(module_a.clone(), vec![module_b.clone(), module_c.clone()]);
        graph.add_module(module_b.clone(), vec![module_c.clone()]);
        graph.add_module(module_c.clone(), vec![]);

        assert_eq!(graph.modules.len(), 3);
        assert_eq!(graph.modules.get(&module_a), Some(&vec![module_b.clone(), module_c.clone()]));
        assert_eq!(graph.modules.get(&module_b), Some(&vec![module_c.clone()]));
        assert_eq!(graph.modules.get(&module_c), Some(&vec![]));
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
    fn test_dependency_graph_complex() {
        let mut graph = DependencyGraph::new();

        let module_a = Atom::new("module_a");
        let module_b = Atom::new("module_b");
        let module_c = Atom::new("module_c");
        let module_d = Atom::new("module_d");

        // A depends on B and C
        graph.add_module(module_a.clone(), vec![module_b.clone(), module_c.clone()]);
        // B depends on D
        graph.add_module(module_b.clone(), vec![module_d.clone()]);
        // C depends on nothing
        graph.add_module(module_c.clone(), vec![]);
        // D depends on nothing
        graph.add_module(module_d.clone(), vec![]);

        assert!(graph.validate_no_cycles().is_ok());

        let order = graph.get_compilation_order();
        assert_eq!(order.len(), 4);

        // Find positions
        let pos_d = order.iter().position(|m| m == &module_d).unwrap();
        let pos_c = order.iter().position(|m| m == &module_c).unwrap();
        let pos_b = order.iter().position(|m| m == &module_b).unwrap();
        let pos_a = order.iter().position(|m| m == &module_a).unwrap();

        // D and C should be first (no dependencies)
        // B should be after D
        // A should be last (depends on B and C)
        assert!(pos_d < pos_b);
        assert!(pos_c < pos_a);
        assert!(pos_b < pos_a);
    }

    #[test]
    fn test_dependency_graph_empty() {
        let graph = DependencyGraph::new();
        assert!(graph.validate_no_cycles().is_ok());
        let order = graph.get_compilation_order();
        assert!(order.is_empty());
    }

    #[test]
    fn test_dependency_graph_single_module() {
        let mut graph = DependencyGraph::new();
        let module = Atom::new("single");

        graph.add_module(module.clone(), vec![]);

        assert!(graph.validate_no_cycles().is_ok());
        let order = graph.get_compilation_order();
        assert_eq!(order.len(), 1);
        assert_eq!(order[0], module);
    }

    #[test]
    fn test_dependency_graph_complex_cycle() {
        let mut graph = DependencyGraph::new();

        let mod1 = Atom::new("mod1");
        let mod2 = Atom::new("mod2");
        let mod3 = Atom::new("mod3");

        // Create a cycle: mod1 -> mod2 -> mod3 -> mod1
        graph.add_module(mod1.clone(), vec![mod2.clone()]);
        graph.add_module(mod2.clone(), vec![mod3.clone()]);
        graph.add_module(mod3.clone(), vec![mod1.clone()]);

        assert!(graph.validate_no_cycles().is_err());
    }

    // ==================== Compilation Pipeline Tests ====================

    #[test]
    fn test_compilation_pipeline_new() {
        let pipeline = CompilationPipeline::new();
        assert_eq!(pipeline.pass_count(), 0);
    }

    #[test]
    fn test_compilation_pipeline_default() {
        let pipeline = CompilationPipeline::default();
        // Default pipeline has 4 passes: Parsing, Analysis, Optimization, CodeGeneration
        assert_eq!(pipeline.pass_count(), 4);
    }

    #[test]
    fn test_compilation_pipeline_add_pass() {
        let mut pipeline = CompilationPipeline::new();
        pipeline.add_pass(Box::new(crate::passes::StatisticsPass::new()));
        assert_eq!(pipeline.pass_count(), 1);
    }

    #[tokio::test]
    async fn test_compilation_pipeline_execute() {
        let pipeline = CompilationPipeline::default();
        let context = CompilationContext::new(
            Atom::new("pipeline_test"),
            "-module(pipeline_test).\n-export([]).\n".to_string(),
        );

        let result = pipeline.execute(context).await;
        // The mock pipeline should succeed
        assert!(result.is_ok());
    }

    // ==================== Compilation Result Tests ====================

    #[test]
    fn test_compilation_result_creation() {
        // Test basic CompilationResult creation
        let ast = entities_erlang_syntax::Module::new(Atom::new("test"));
        let result = CompilationResult {
            module_name: Atom::new("test"),
            ast,
            bytecode: vec![1, 2, 3, 4],
            warnings: vec![],
            metadata: CompilationMetadata {
                compilation_time_ms: 100,
                source_size: 50,
                bytecode_size: 200,
                optimization_level: OptimizationLevel::Standard,
            },
            context_metadata: HashMap::new(),
        };

        assert_eq!(result.module_name.as_str(), "test");
        assert_eq!(result.bytecode, vec![1, 2, 3, 4]);
        assert!(result.warnings.is_empty());
        assert_eq!(result.metadata.compilation_time_ms, 100);
        assert_eq!(result.metadata.optimization_level, OptimizationLevel::Standard);
    }

    // ==================== Warning Tests ====================

    #[test]
    fn test_compilation_warning_creation() {
        let warning = CompilationWarning {
            message: "Unused variable".to_string(),
            position: entities_erlang_syntax::Position { line: 10, column: 5, file: None },
            code: WarningCode::UnusedVariable,
        };

        assert_eq!(warning.message, "Unused variable");
        assert_eq!(warning.code, WarningCode::UnusedVariable);
    }

    #[test]
    fn test_warning_code_variants() {
        assert_eq!(WarningCode::UnusedVariable, WarningCode::UnusedVariable);
        assert_eq!(WarningCode::UnusedFunction, WarningCode::UnusedFunction);
        assert_eq!(WarningCode::ShadowedVariable, WarningCode::ShadowedVariable);
        assert_eq!(WarningCode::MissingSpec, WarningCode::MissingSpec);
        assert_eq!(WarningCode::DeprecatedFunction, WarningCode::DeprecatedFunction);
        assert_eq!(WarningCode::TypeMismatch, WarningCode::TypeMismatch);
        assert_eq!(WarningCode::Other("test".to_string()), WarningCode::Other("test".to_string()));
    }

    // ==================== Metadata Tests ====================

    #[test]
    fn test_compilation_metadata() {
        let metadata = CompilationMetadata {
            compilation_time_ms: 150,
            source_size: 100,
            bytecode_size: 300,
            optimization_level: OptimizationLevel::Aggressive,
        };

        assert_eq!(metadata.compilation_time_ms, 150);
        assert_eq!(metadata.source_size, 100);
        assert_eq!(metadata.bytecode_size, 300);
        assert_eq!(metadata.optimization_level, OptimizationLevel::Aggressive);
    }

    #[test]
    fn test_compilation_metadata_default() {
        let metadata = CompilationMetadata::default();
        assert_eq!(metadata.compilation_time_ms, 0);
        assert_eq!(metadata.source_size, 0);
        assert_eq!(metadata.bytecode_size, 0);
        assert_eq!(metadata.optimization_level, OptimizationLevel::Standard);
    }

    // ==================== Error Handling Tests ====================

    #[tokio::test]
    async fn test_compile_module_empty_source() {
        let orchestrator = CompilationOrchestrator::new();
        let result = orchestrator.compile_module("", &Atom::new("empty")).await;

        // Empty source should fail during parsing
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_compile_module_large_source() {
        let orchestrator = CompilationOrchestrator::new();
        let large_source = "-module(large).\n".repeat(1000);
        let result = orchestrator.compile_module(&large_source, &Atom::new("large")).await;

        assert!(result.is_ok());
    }

    // ==================== Edge Cases and Boundary Conditions ====================

    #[test]
    fn test_dependency_graph_with_self_dependency() {
        let mut graph = DependencyGraph::new();
        let module = Atom::new("self_dep");

        // Module depends on itself - should be detected as a cycle
        graph.add_module(module.clone(), vec![module.clone()]);

        assert!(graph.validate_no_cycles().is_err());
    }

    #[test]
    fn test_dependency_graph_isolated_modules() {
        let mut graph = DependencyGraph::new();

        let mod1 = Atom::new("isolated1");
        let mod2 = Atom::new("isolated2");

        graph.add_module(mod1.clone(), vec![]);
        graph.add_module(mod2.clone(), vec![]);

        assert!(graph.validate_no_cycles().is_ok());
        let order = graph.get_compilation_order();
        assert_eq!(order.len(), 2);
        // Order doesn't matter for isolated modules
        assert!(order.contains(&mod1));
        assert!(order.contains(&mod2));
    }

    #[test]
    fn test_compilation_context_with_ast() {
        let context = CompilationContext::new(
            Atom::new("test"),
            "test source".to_string(),
        );

        // AST starts as None and gets populated during compilation
        assert!(context.ast.is_none());
    }

    #[tokio::test]
    async fn test_compilation_orchestrator_multiple_calls() {
        let orchestrator = CompilationOrchestrator::new();

        // Multiple calls should work independently
        let result1 = orchestrator.compile_module("-module(test1).\n", &Atom::new("test1")).await;
        let result2 = orchestrator.compile_module("-module(test2).\n", &Atom::new("test2")).await;

        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert_ne!(result1.unwrap().module_name.as_str(), result2.unwrap().module_name.as_str());
    }

    // ==================== Concurrent Compilation Tests ====================

    #[tokio::test]
    async fn test_multiple_module_compilation() {
        let orchestrator = CompilationOrchestrator::new();

        // Compile multiple modules sequentially (simplified from concurrent)
        let source1 = "-module(mod1).\n-export([]).\n";
        let source2 = "-module(mod2).\n-export([]).\n";

        let result1 = orchestrator.compile_module(source1, &Atom::new("mod1")).await;
        let result2 = orchestrator.compile_module(source2, &Atom::new("mod2")).await;

        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert_ne!(result1.unwrap().module_name.as_str(), result2.unwrap().module_name.as_str());
    }

    // ==================== Integration Tests ====================

    #[tokio::test]
    async fn test_full_compilation_workflow() {
        let orchestrator = CompilationOrchestrator::new();

        // Create a simple Erlang module
        let erlang_source = r#"
-module(full_test).
-export([hello/0, add/2]).

hello() ->
    "Hello, World!".

add(X, Y) ->
    X + Y.
"#;

        let result = orchestrator.compile_module(erlang_source, &Atom::new("full_test")).await;

        assert!(result.is_ok());
        let compilation = result.unwrap();

        assert_eq!(compilation.module_name.as_str(), "full_test");
        // source_text is not stored in CompilationResult
        // bytecode is empty in mock implementation (generated by interfaces layer)
        assert!(compilation.bytecode.is_empty());
        assert_eq!(compilation.metadata.optimization_level, OptimizationLevel::Standard);
    }

    #[tokio::test]
    async fn test_compilation_with_different_output_formats() {
        let formats = vec![
            OutputFormat::Beam,
            OutputFormat::Asm,
            OutputFormat::Core,
            OutputFormat::Kernel,
        ];

        for format in formats {
            let options = CompilationOptions {
                output_format: format.clone(),
                ..CompilationOptions::default()
            };

            let orchestrator = CompilationOrchestrator::new().with_options(options);
            let source = "-module(format_test).\n-export([]).\n";
            let result = orchestrator.compile_module(source, &Atom::new("format_test")).await;

            // Test that compilation succeeds with different output formats
            // Note: output_format is not stored in CompilationResult
            assert!(result.is_ok());
        }
    }

    #[tokio::test]
    async fn test_compilation_with_different_optimization_levels() {
        let levels = vec![
            OptimizationLevel::None,
            OptimizationLevel::Basic,
            OptimizationLevel::Standard,
            OptimizationLevel::Aggressive,
        ];

        for level in levels {
            let options = CompilationOptions {
                optimization_level: level.clone(),
                ..CompilationOptions::default()
            };

            let orchestrator = CompilationOrchestrator::new().with_options(options);
            let source = "-module(opt_test).\n-export([]).\n";
            let result = orchestrator.compile_module(source, &Atom::new("opt_test")).await;

            assert!(result.is_ok());
            let compilation = result.unwrap();
            assert_eq!(compilation.metadata.optimization_level, level);
        }
    }
}
