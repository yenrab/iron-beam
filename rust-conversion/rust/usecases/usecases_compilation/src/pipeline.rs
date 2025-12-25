/*!
# Compilation Pipeline

This module implements the compilation pipeline that orchestrates the various
compilation passes and phases. It provides a flexible, extensible framework
for building compilation workflows.
*/

use super::*;

/// Compilation pipeline that executes a series of passes
pub struct CompilationPipeline {
    passes: Vec<Box<dyn CompilationPass>>,
}

impl CompilationPipeline {
    /// Create a new empty pipeline
    pub fn new() -> Self {
        Self {
            passes: Vec::new(),
        }
    }

    /// Create a default pipeline with standard passes
    pub fn default() -> Self {
        let mut pipeline = Self::new();
        pipeline
            .add_pass(Box::new(ParsingPass))
            .add_pass(Box::new(AnalysisPass))
            .add_pass(Box::new(OptimizationPass))
            .add_pass(Box::new(CodeGenerationPass));
        pipeline
    }

    /// Add a compilation pass to the pipeline
    pub fn add_pass(&mut self, pass: Box<dyn CompilationPass>) -> &mut Self {
        self.passes.push(pass);
        self
    }

    /// Execute the pipeline on a compilation context
    pub async fn execute(&self, mut context: CompilationContext) -> CompilerResult<CompilationResult> {
        let start_time = std::time::Instant::now();

        for pass in &self.passes {
            pass.execute(&mut context).await?;
        }

        let compilation_time = start_time.elapsed().as_millis() as u64;

        // Generate mock result (would be populated by the actual passes)
        Ok(CompilationResult {
            module_name: context.module_name.clone(),
            bytecode: generate_mock_bytecode(&context),
            warnings: vec![], // Would be collected from passes
            metadata: CompilationMetadata {
                compilation_time_ms: compilation_time,
                source_size: context.source_text.len(),
                bytecode_size: generate_mock_bytecode(&context).len(),
                optimization_level: context.options.optimization_level,
            },
        })
    }

    /// Get the number of passes in the pipeline
    pub fn len(&self) -> usize {
        self.passes.len()
    }

    /// Check if the pipeline is empty
    pub fn is_empty(&self) -> bool {
        self.passes.is_empty()
    }
}

impl Default for CompilationPipeline {
    fn default() -> Self {
        Self::default()
    }
}

/// Trait for compilation passes
#[async_trait::async_trait]
pub trait CompilationPass: Send + Sync {
    /// Execute this pass on the compilation context
    async fn execute(&self, context: &mut CompilationContext) -> CompilerResult<()>;

    /// Get the name of this pass
    fn name(&self) -> &'static str;

    /// Get the phase this pass belongs to
    fn phase(&self) -> CompilationPhase;
}

#[async_trait::async_trait]
impl<T: CompilationPass + ?Sized> CompilationPass for Box<T> {
    async fn execute(&self, context: &mut CompilationContext) -> CompilerResult<()> {
        (**self).execute(context).await
    }

    fn name(&self) -> &'static str {
        (**self).name()
    }

    fn phase(&self) -> CompilationPhase {
        (**self).phase()
    }
}

/// Standard compilation passes

/// Parsing pass - converts source text to AST
pub struct ParsingPass;

#[async_trait::async_trait]
impl CompilationPass for ParsingPass {
    async fn execute(&self, context: &mut CompilationContext) -> CompilerResult<()> {
        // In a real implementation, this would parse the source text
        // For now, we'll just validate that source exists
        if context.source_text.is_empty() {
            return Err(CompilerError::InvalidArgument(
                "Empty source text".to_string()
            ));
        }

        // Add metadata about parsing
        context.metadata.insert("parsed".to_string(), "true".to_string());

        Ok(())
    }

    fn name(&self) -> &'static str {
        "parsing"
    }

    fn phase(&self) -> CompilationPhase {
        CompilationPhase::Parsing
    }
}

/// Analysis pass - validates and analyzes the AST
pub struct AnalysisPass;

#[async_trait::async_trait]
impl CompilationPass for AnalysisPass {
    async fn execute(&self, context: &mut CompilationContext) -> CompilerResult<()> {
        // In a real implementation, this would perform semantic analysis
        // For now, we'll simulate basic validation

        if context.source_text.contains("error") {
            return Err(CompilerError::InvalidArgument(
                "Simulated analysis error".to_string()
            ));
        }

        context.metadata.insert("analyzed".to_string(), "true".to_string());

        Ok(())
    }

    fn name(&self) -> &'static str {
        "analysis"
    }

    fn phase(&self) -> CompilationPhase {
        CompilationPhase::Analysis
    }
}

/// Optimization pass - applies code optimizations
pub struct OptimizationPass;

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

/// Code generation pass - converts AST to bytecode
pub struct CodeGenerationPass;

#[async_trait::async_trait]
impl CompilationPass for CodeGenerationPass {
    async fn execute(&self, context: &mut CompilationContext) -> CompilerResult<()> {
        // In a real implementation, this would generate BEAM bytecode
        // For now, we'll just mark it as generated

        context.metadata.insert("code_generated".to_string(), "true".to_string());

        Ok(())
    }

    fn name(&self) -> &'static str {
        "code_generation"
    }

    fn phase(&self) -> CompilationPhase {
        CompilationPhase::CodeGeneration
    }
}

/// Utility function to generate mock bytecode for testing
fn generate_mock_bytecode(context: &CompilationContext) -> Vec<u8> {
    // Generate some mock bytecode based on the module name and source
    let mut bytecode = Vec::new();

    // Simple mock: length of module name + source size
    let name_len = context.module_name.as_str().len() as u8;
    let source_len = (context.source_text.len() % 256) as u8;

    bytecode.extend_from_slice(&[0xBE, 0xA5, 0x00]); // Mock BEAM header
    bytecode.push(name_len);
    bytecode.push(source_len);

    // Add some padding to make it look like real bytecode
    while bytecode.len() < 64 {
        bytecode.push(0);
    }

    bytecode
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_creation() {
        let pipeline = CompilationPipeline::new();
        assert!(pipeline.is_empty());
        assert_eq!(pipeline.len(), 0);
    }

    #[test]
    fn test_default_pipeline() {
        let pipeline = CompilationPipeline::default();
        assert!(!pipeline.is_empty());
        assert_eq!(pipeline.len(), 4); // parsing, analysis, optimization, codegen
    }

    #[test]
    fn test_add_pass() {
        let mut pipeline = CompilationPipeline::new();
        pipeline.add_pass(Box::new(ParsingPass));

        assert_eq!(pipeline.len(), 1);
    }

    #[tokio::test]
    async fn test_parsing_pass() {
        let pass = ParsingPass;
        let mut context = CompilationContext::new(
            Atom::new("test"),
            "valid source".to_string(),
        );

        let result = pass.execute(&mut context).await;
        assert!(result.is_ok());
        assert_eq!(pass.name(), "parsing");
        assert_eq!(pass.phase(), CompilationPhase::Parsing);
        assert_eq!(context.metadata.get("parsed"), Some(&"true".to_string()));
    }

    #[tokio::test]
    async fn test_parsing_pass_empty_source() {
        let pass = ParsingPass;
        let mut context = CompilationContext::new(
            Atom::new("test"),
            "".to_string(), // Empty source
        );

        let result = pass.execute(&mut context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_analysis_pass() {
        let pass = AnalysisPass;
        let mut context = CompilationContext::new(
            Atom::new("test"),
            "valid source".to_string(),
        );

        let result = pass.execute(&mut context).await;
        assert!(result.is_ok());
        assert_eq!(pass.name(), "analysis");
        assert_eq!(pass.phase(), CompilationPhase::Analysis);
    }

    #[tokio::test]
    async fn test_analysis_pass_with_error() {
        let pass = AnalysisPass;
        let mut context = CompilationContext::new(
            Atom::new("test"),
            "source with error".to_string(), // Contains "error"
        );

        let result = pass.execute(&mut context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_optimization_pass() {
        let pass = OptimizationPass;
        let mut context = CompilationContext::new(
            Atom::new("test"),
            "source".to_string(),
        );

        let result = pass.execute(&mut context).await;
        assert!(result.is_ok());
        assert_eq!(pass.name(), "optimization");
        assert_eq!(pass.phase(), CompilationPhase::Optimization);
        assert!(context.metadata.contains_key("optimized"));
    }

    #[tokio::test]
    async fn test_code_generation_pass() {
        let pass = CodeGenerationPass;
        let mut context = CompilationContext::new(
            Atom::new("test"),
            "source".to_string(),
        );

        let result = pass.execute(&mut context).await;
        assert!(result.is_ok());
        assert_eq!(pass.name(), "code_generation");
        assert_eq!(pass.phase(), CompilationPhase::CodeGeneration);
        assert_eq!(context.metadata.get("code_generated"), Some(&"true".to_string()));
    }

    #[tokio::test]
    async fn test_pipeline_execution() {
        let pipeline = CompilationPipeline::default();
        let context = CompilationContext::new(
            Atom::new("test_module"),
            "test source code".to_string(),
        );

        let result = pipeline.execute(context).await;
        assert!(result.is_ok());

        let compilation_result = result.unwrap();
        assert_eq!(compilation_result.module_name.as_str(), "test_module");
        assert!(!compilation_result.bytecode.is_empty());
        assert!(compilation_result.metadata.compilation_time_ms >= 0);
    }

    #[test]
    fn test_mock_bytecode_generation() {
        let context = CompilationContext::new(
            Atom::new("test_module"),
            "test source".to_string(),
        );

        let bytecode = generate_mock_bytecode(&context);
        assert!(!bytecode.is_empty());
        assert!(bytecode.len() >= 64); // Minimum size from our mock

        // Check header bytes
        assert_eq!(&bytecode[0..3], &[0xBE, 0xA5, 0x00]);
    }
}
