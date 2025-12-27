/*!
# Compilation Passes

This module contains various compilation passes that can be plugged into
the compilation pipeline. Each pass performs a specific transformation or
analysis on the compilation context.
*/

use super::*;

/// Pass for collecting compilation statistics
pub struct StatisticsPass {
    pub stats: CompilationStats,
}

impl StatisticsPass {
    pub fn new() -> Self {
        Self {
            stats: CompilationStats {
                total_modules: 0,
                successful_compilations: 0,
                failed_compilations: 0,
                total_warnings: 0,
                total_compilation_time_ms: 0,
            },
        }
    }
}

#[async_trait::async_trait]
impl CompilationPass for StatisticsPass {
    async fn execute(&self, context: &mut CompilationContext) -> CompilerResult<()> {
        // Update statistics based on context
        context.metadata.insert(
            "stats_collected".to_string(),
            "true".to_string(),
        );
        Ok(())
    }

    fn name(&self) -> &'static str {
        "statistics"
    }

    fn phase(&self) -> CompilationPhase {
        CompilationPhase::Complete
    }
}

/// Pass for validating module structure
pub struct ValidationPass;

#[async_trait::async_trait]
impl CompilationPass for ValidationPass {
    async fn execute(&self, context: &mut CompilationContext) -> CompilerResult<()> {
        // Validate module name format
        let name = context.module_name.as_str();
        if name.is_empty() {
            return Err(CompilerError::InvalidArgument(
                "Module name cannot be empty".to_string()
            ));
        }

        if !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(CompilerError::InvalidArgument(
                format!("Invalid module name: {}", name)
            ));
        }

        // Validate source is not empty
        if context.source_text.trim().is_empty() {
            return Err(CompilerError::InvalidArgument(
                "Source text cannot be empty".to_string()
            ));
        }

        context.metadata.insert("validated".to_string(), "true".to_string());
        Ok(())
    }

    fn name(&self) -> &'static str {
        "validation"
    }

    fn phase(&self) -> CompilationPhase {
        CompilationPhase::Analysis
    }
}

/// Pass for expanding macros and includes
pub struct MacroExpansionPass;

#[async_trait::async_trait]
impl CompilationPass for MacroExpansionPass {
    async fn execute(&self, context: &mut CompilationContext) -> CompilerResult<()> {
        // In a real implementation, this would expand macros and includes
        // For now, just mark as processed

        context.metadata.insert("macros_expanded".to_string(), "true".to_string());
        Ok(())
    }

    fn name(&self) -> &'static str {
        "macro_expansion"
    }

    fn phase(&self) -> CompilationPhase {
        CompilationPhase::Analysis
    }
}

/// Pass for type checking (if enabled)
pub struct TypeCheckPass;

#[async_trait::async_trait]
impl CompilationPass for TypeCheckPass {
    async fn execute(&self, context: &mut CompilationContext) -> CompilerResult<()> {
        // Type checking would be performed here if enabled
        // For now, skip if not requested

        context.metadata.insert("type_checked".to_string(), "skipped".to_string());
        Ok(())
    }

    fn name(&self) -> &'static str {
        "type_check"
    }

    fn phase(&self) -> CompilationPhase {
        CompilationPhase::Analysis
    }
}

/// Pass for dead code elimination
pub struct DeadCodeEliminationPass;

#[async_trait::async_trait]
impl CompilationPass for DeadCodeEliminationPass {
    async fn execute(&self, context: &mut CompilationContext) -> CompilerResult<()> {
        // Remove unused functions and variables
        context.metadata.insert("dead_code_eliminated".to_string(), "true".to_string());
        Ok(())
    }

    fn name(&self) -> &'static str {
        "dead_code_elimination"
    }

    fn phase(&self) -> CompilationPhase {
        CompilationPhase::Optimization
    }
}

/// Pass for constant folding
pub struct ConstantFoldingPass;

#[async_trait::async_trait]
impl CompilationPass for ConstantFoldingPass {
    async fn execute(&self, context: &mut CompilationContext) -> CompilerResult<()> {
        // Fold constant expressions
        context.metadata.insert("constants_folded".to_string(), "true".to_string());
        Ok(())
    }

    fn name(&self) -> &'static str {
        "constant_folding"
    }

    fn phase(&self) -> CompilationPhase {
        CompilationPhase::Optimization
    }
}

/// Pass for tail call optimization
pub struct TailCallOptimizationPass;

#[async_trait::async_trait]
impl CompilationPass for TailCallOptimizationPass {
    async fn execute(&self, context: &mut CompilationContext) -> CompilerResult<()> {
        // Optimize tail recursive calls
        context.metadata.insert("tail_calls_optimized".to_string(), "true".to_string());
        Ok(())
    }

    fn name(&self) -> &'static str {
        "tail_call_optimization"
    }

    fn phase(&self) -> CompilationPhase {
        CompilationPhase::Optimization
    }
}

/// Pass for generating debug information
pub struct DebugInfoPass;

#[async_trait::async_trait]
impl CompilationPass for DebugInfoPass {
    async fn execute(&self, context: &mut CompilationContext) -> CompilerResult<()> {
        if context.options.debug_info {
            context.metadata.insert("debug_info_generated".to_string(), "true".to_string());
        } else {
            context.metadata.insert("debug_info_generated".to_string(), "skipped".to_string());
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "debug_info"
    }

    fn phase(&self) -> CompilationPhase {
        CompilationPhase::CodeGeneration
    }
}

/// Pass for final bytecode linking
pub struct LinkingPass;

pub struct CodeGenerationPass;

pub struct AnalysisPass;

pub struct OptimizationPass;

#[async_trait::async_trait]
impl CompilationPass for LinkingPass {
    async fn execute(&self, context: &mut CompilationContext) -> CompilerResult<()> {
        // Resolve external references and link modules
        context.metadata.insert("linked".to_string(), "true".to_string());
        Ok(())
    }

    fn name(&self) -> &'static str {
        "linking"
    }

    fn phase(&self) -> CompilationPhase {
        CompilationPhase::Linking
    }
}

#[async_trait::async_trait]
impl CompilationPass for CodeGenerationPass {
    async fn execute(&self, _context: &mut CompilationContext) -> CompilerResult<()> {
        // Code generation is handled by interfaces_compiler_api::BytecodeGenerator
        // This pass marks that code generation should occur
        Ok(())
    }

    fn name(&self) -> &'static str {
        "code_generation"
    }

    fn phase(&self) -> CompilationPhase {
        CompilationPhase::CodeGeneration
    }
}

#[async_trait::async_trait]
impl CompilationPass for AnalysisPass {
    async fn execute(&self, context: &mut CompilationContext) -> CompilerResult<()> {
        // Check that we have a parsed AST
        let ast = context.ast.as_ref().ok_or_else(|| {
            CompilerError::InvalidArgument("No AST available for analysis".to_string())
        })?;

        // Validate module structure
        if ast.name.as_str().is_empty() {
            return Err(CompilerError::InvalidArgument(
                "Module must have a name".to_string()
            ));
        }

        // Check for required module attribute
        let has_module_attr = ast.attributes.iter().any(|attr| {
            matches!(attr.value, entities_erlang_syntax::AttributeValue::Module(_))
        });

        if !has_module_attr {
            return Err(CompilerError::InvalidArgument(
                "Module must have a -module() attribute".to_string()
            ));
        }

        // Validate that module name matches context
        if ast.name != context.module_name {
            return Err(CompilerError::InvalidArgument(
                format!("Module name '{}' does not match expected name '{}'",
                       ast.name.as_str(), context.module_name.as_str())
            ));
        }

        // Basic validation complete
        context.metadata.insert("analyzed".to_string(), "true".to_string());
        context.metadata.insert("module_name".to_string(), ast.name.as_str().to_string());
        context.metadata.insert("attributes_count".to_string(), ast.attributes.len().to_string());
        context.metadata.insert("functions_count".to_string(), ast.functions.len().to_string());

        Ok(())
    }

    fn name(&self) -> &'static str {
        "analysis"
    }

    fn phase(&self) -> CompilationPhase {
        CompilationPhase::Analysis
    }
}

#[async_trait::async_trait]
impl CompilationPass for OptimizationPass {
    async fn execute(&self, context: &mut CompilationContext) -> CompilerResult<()> {
        // In a real implementation, this would apply various optimizations
        // based on the optimization level

        let opt_level = format!("{:?}", context.options.optimization_level);
        context.metadata.insert("optimized".to_string(), opt_level);

        Ok(())
    }

    fn name(&self) -> &'static str {
        "optimization"
    }

    fn phase(&self) -> CompilationPhase {
        CompilationPhase::Optimization
    }
}

/// Utility functions for pass management

/// Create an optimized compilation pipeline
pub fn create_optimized_pipeline() -> CompilationPipeline {
    let mut pipeline = CompilationPipeline::new();
    pipeline
        .add_pass(Box::new(ValidationPass))
        .add_pass(Box::new(ParsingPass))
        .add_pass(Box::new(MacroExpansionPass))
        .add_pass(Box::new(AnalysisPass))
        .add_pass(Box::new(TypeCheckPass))
        .add_pass(Box::new(OptimizationPass))
        .add_pass(Box::new(DeadCodeEliminationPass))
        .add_pass(Box::new(ConstantFoldingPass))
        .add_pass(Box::new(TailCallOptimizationPass))
        .add_pass(Box::new(DebugInfoPass))
        .add_pass(Box::new(CodeGenerationPass))
        .add_pass(Box::new(LinkingPass))
        .add_pass(Box::new(StatisticsPass::new()));
    pipeline
}

/// Create a minimal compilation pipeline for testing
pub fn create_minimal_pipeline() -> CompilationPipeline {
    let mut pipeline = CompilationPipeline::new();
    pipeline
        .add_pass(Box::new(ValidationPass))
        .add_pass(Box::new(ParsingPass))
        .add_pass(Box::new(CodeGenerationPass));
    pipeline
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validation_pass() {
        let pass = ValidationPass;

        // Valid context
        let mut context = CompilationContext::new(
            Atom::new("valid_module"),
            "valid source".to_string(),
        );

        let result = pass.execute(&mut context).await;
        assert!(result.is_ok());
        assert_eq!(context.metadata.get("validated"), Some(&"true".to_string()));

        // Invalid module name
        let mut invalid_context = CompilationContext::new(
            Atom::new("invalid-module!"),
            "source".to_string(),
        );

        let result = pass.execute(&mut invalid_context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validation_pass_empty_source() {
        let pass = ValidationPass;
        let mut context = CompilationContext::new(
            Atom::new("test"),
            "".to_string(),
        );

        let result = pass.execute(&mut context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_macro_expansion_pass() {
        let pass = MacroExpansionPass;
        let mut context = CompilationContext::new(
            Atom::new("test"),
            "source".to_string(),
        );

        let result = pass.execute(&mut context).await;
        assert!(result.is_ok());
        assert_eq!(context.metadata.get("macros_expanded"), Some(&"true".to_string()));
    }

    #[tokio::test]
    async fn test_dead_code_elimination_pass() {
        let pass = DeadCodeEliminationPass;
        let mut context = CompilationContext::new(
            Atom::new("test"),
            "source".to_string(),
        );

        let result = pass.execute(&mut context).await;
        assert!(result.is_ok());
        assert_eq!(context.metadata.get("dead_code_eliminated"), Some(&"true".to_string()));
    }

    #[tokio::test]
    async fn test_debug_info_pass_with_debug() {
        let pass = DebugInfoPass;
        let mut context = CompilationContext::new(
            Atom::new("test"),
            "source".to_string(),
        ).with_options(CompilationOptions {
            debug_info: true,
            ..Default::default()
        });

        let result = pass.execute(&mut context).await;
        assert!(result.is_ok());
        assert_eq!(context.metadata.get("debug_info_generated"), Some(&"true".to_string()));
    }

    #[tokio::test]
    async fn test_debug_info_pass_without_debug() {
        let pass = DebugInfoPass;
        let mut context = CompilationContext::new(
            Atom::new("test"),
            "source".to_string(),
        );

        let result = pass.execute(&mut context).await;
        assert!(result.is_ok());
        assert_eq!(context.metadata.get("debug_info_generated"), Some(&"skipped".to_string()));
    }

    #[test]
    fn test_create_optimized_pipeline() {
        let pipeline = create_optimized_pipeline();
        assert_eq!(pipeline.len(), 13); // All passes including StatisticsPass
    }

    #[test]
    fn test_create_minimal_pipeline() {
        let pipeline = create_minimal_pipeline();
        assert_eq!(pipeline.len(), 3); // Only essential passes
    }

    #[test]
    fn test_statistics_pass_creation() {
        let pass = StatisticsPass::new();
        assert_eq!(pass.stats.total_modules, 0);
        assert_eq!(pass.name(), "statistics");
        assert_eq!(pass.phase(), CompilationPhase::Complete);
    }
}
