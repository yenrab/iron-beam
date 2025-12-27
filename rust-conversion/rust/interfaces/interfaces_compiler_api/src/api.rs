/*!
# Compiler API Implementation

Detailed implementation of the external compiler APIs for tool integration.
*/

use super::*;

/// Output from compiling a single module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationOutput {
    pub module_name: String,
    pub success: bool,
    pub bytecode: Option<Vec<u8>>,
    pub warnings: Vec<APIWarning>,
    pub errors: Vec<String>,
    pub compilation_time_ms: u64,
    pub metadata: HashMap<String, String>,
}

impl CompilationOutput {
    pub fn from_result(result: CompilationResult) -> Result<Self, String> {
        // Generate real BEAM bytecode using local generator
        let bytecode_generator = crate::bytecode::BytecodeGenerator::new();
        let beam_file = bytecode_generator.generate_beam_file(&result)
            .map_err(|e| format!("BEAM generation failed: {}", e))?;

        // Convert BeamFile to raw bytes
        let bytecode = beam_file.to_bytes();

        Ok(Self {
            module_name: result.module_name.to_string(),
            success: true, // CompilationResult doesn't have failure variants in this design
            bytecode: Some(bytecode),
            warnings: result.warnings.into_iter()
                .map(|w| APIWarning {
                    message: w.message,
                    line: w.position.line,
                    column: w.position.column,
                    code: format!("{:?}", w.code),
                })
                .collect(),
            errors: Vec::new(), // Would be populated if CompilationResult had errors
            compilation_time_ms: result.metadata.compilation_time_ms,
            metadata: HashMap::new(), // Would convert CompilationMetadata
        })
    }

    pub fn failure(module_name: &str, errors: Vec<String>) -> Self {
        Self {
            module_name: module_name.to_string(),
            success: false,
            bytecode: None,
            warnings: Vec::new(),
            errors,
            compilation_time_ms: 0,
            metadata: HashMap::new(),
        }
    }
}

/// Output from compiling multiple modules
#[derive(Debug, Clone, )]
pub struct BatchCompilationOutput {
    pub results: HashMap<String, CompilationOutput>,
    pub summary: BatchSummary,
}

impl BatchCompilationOutput {
    pub fn from_batch_result(result: BatchCompilationResult) -> Self {
        let success_count = result.success_count();
        let error_count = result.errors.len();
        let total_modules = result.results.len() + error_count;

        let results: HashMap<String, CompilationOutput> = result.results.into_iter()
            .map(|(k, v)| (k.to_string(), CompilationOutput::from_result(v).unwrap_or_else(|e| {
                CompilationOutput::failure(&k.to_string(), vec![e])
            })))
            .collect();

        let errors: HashMap<String, CompilationOutput> = result.errors.into_iter()
            .map(|(module, error)| (module.to_string(), CompilationOutput::failure(
                &module.to_string(),
                vec![error.to_string()]
            )))
            .collect();

        let mut all_results = results;
        all_results.extend(errors);

        Self {
            results: all_results,
            summary: BatchSummary {
                total_modules,
                successful: success_count,
                failed: error_count,
            },
        }
    }
}

/// Summary of batch compilation results
#[derive(Debug, Clone, )]
pub struct BatchSummary {
    pub total_modules: usize,
    pub successful: usize,
    pub failed: usize,
}

/// Warning information for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIWarning {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub code: String,
}

/// Analysis result for source code inspection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub module_name: Atom,
    pub syntax_valid: bool,
    pub warnings: Vec<APIWarning>,
    pub errors: Vec<String>,
    pub metrics: SourceMetrics,
}

/// Source code metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMetrics {
    pub lines_of_code: usize,
    pub functions: usize,
    pub complexity: usize,
}

/// Compiler information for API discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerInfo {
    pub version: String,
    pub supported_formats: Vec<String>,
    pub features: Vec<String>,
    pub config: APIConfig,
}

/// Compilation request for remote/API usage
#[derive(Debug, Clone, )]
pub struct CompilationRequest {
    pub module_name: String,
    pub source_code: String,
    pub options: CompilationOptions,
}

impl CompilationRequest {
    pub fn new(module_name: &str, source_code: &str) -> Self {
        Self {
            module_name: module_name.to_string(),
            source_code: source_code.to_string(),
            options: CompilationOptions::default(),
        }
    }

    pub fn with_options(mut self, options: CompilationOptions) -> Self {
        self.options = options;
        self
    }
}

/// Batch compilation request
#[derive(Debug, Clone, )]
pub struct BatchCompilationRequest {
    pub modules: Vec<CompilationRequest>,
    pub options: CompilationOptions,
}

/// LSP (Language Server Protocol) integration
pub mod lsp {
    use super::*;

    /// LSP completion request
    #[derive(Debug, Clone, )]
    pub struct CompletionRequest {
        pub module_name: String,
        pub source_code: String,
        pub position: Position,
        pub context: Option<String>,
    }

    /// LSP completion response
    #[derive(Debug, Clone, )]
    pub struct CompletionResponse {
        pub completions: Vec<CompletionItem>,
    }

    /// Completion item
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CompletionItem {
        pub label: String,
        pub kind: CompletionKind,
        pub detail: Option<String>,
        pub documentation: Option<String>,
    }

    /// Completion item kind
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum CompletionKind {
        Function,
        Module,
        Variable,
        Type,
        Macro,
        Keyword,
    }

    /// LSP diagnostics for a module
    #[derive(Debug, Clone, )]
    pub struct DiagnosticsResponse {
        pub module_name: String,
        pub diagnostics: Vec<Diagnostic>,
    }

    /// LSP diagnostic
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Diagnostic {
        pub range: Range,
        pub severity: DiagnosticSeverity,
        pub code: Option<String>,
        pub message: String,
        pub source: String,
    }

    /// Diagnostic severity levels
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum DiagnosticSeverity {
        Error = 1,
        Warning = 2,
        Information = 3,
        Hint = 4,
    }

    /// Text range
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Range {
        pub start: Position,
        pub end: Position,
    }
}

/// Build system integration
pub mod build {
    use super::*;

    /// Build request for incremental compilation
    #[derive(Debug, Clone, )]
    pub struct BuildRequest {
        pub target_modules: Vec<String>,
        pub changed_files: Vec<String>,
        pub build_options: BuildOptions,
    }

    /// Build options
    #[derive(Debug, Clone, )]
    pub struct BuildOptions {
        pub incremental: bool,
        pub parallel_jobs: usize,
        pub fail_fast: bool,
        pub clean_build: bool,
    }

    /// Build response
    #[derive(Debug, Clone, )]
    pub struct BuildResponse {
        pub success: bool,
        pub compiled_modules: Vec<String>,
        pub errors: Vec<String>,
        pub warnings: Vec<String>,
        pub build_time_ms: u64,
    }

    /// Dependency information
    #[derive(Debug, Clone, )]
    pub struct DependencyInfo {
        pub module: String,
        pub dependencies: Vec<String>,
        pub dependents: Vec<String>,
    }
}

/// Testing framework integration
pub mod testing {
    use super::*;

    /// Test execution request
    #[derive(Debug, Clone, )]
    pub struct TestRequest {
        pub module_pattern: Option<String>,
        pub function_pattern: Option<String>,
        pub options: TestOptions,
    }

    /// Test execution options
    #[derive(Debug, Clone, )]
    pub struct TestOptions {
        pub verbose: bool,
        pub cover: bool,
        pub shuffle: bool,
        pub repeat: usize,
    }

    /// Test results
    #[derive(Debug, Clone, )]
    pub struct TestResults {
        pub total_tests: usize,
        pub passed: usize,
        pub failed: usize,
        pub skipped: usize,
        pub coverage: Option<CoverageReport>,
        pub results: Vec<TestResult>,
    }

    /// Individual test result
    #[derive(Debug, Clone, )]
    pub struct TestResult {
        pub module: String,
        pub function: String,
        pub arity: usize,
        pub status: TestStatus,
        pub duration_ms: u64,
        pub error: Option<String>,
    }

    /// Test execution status
    #[derive(Debug, Clone, )]
    pub enum TestStatus {
        Passed,
        Failed,
        Skipped,
    }

    /// Code coverage report
    #[derive(Debug, Clone, )]
    pub struct CoverageReport {
        pub module_coverage: HashMap<String, f64>,
        pub overall_coverage: f64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compilation_output_from_result() {
        let result = CompilationResult {
            module_name: Atom::new("test_module"),
            bytecode: vec![1, 2, 3, 4],
            warnings: vec![
                CompilationWarning {
                    message: "test warning".to_string(),
                    position: Position { line: 10, column: 5, file: None },
                    code: WarningCode::UnusedVariable,
                },
            ],
            metadata: CompilationMetadata {
                compilation_time_ms: 100,
                source_size: 1000,
                bytecode_size: 500,
                optimization_level: OptimizationLevel::Standard,
            },
        };

        let output = CompilationOutput::from_result(result).unwrap();
        assert_eq!(output.module_name, "test_module");
        assert!(output.success);
        assert_eq!(output.bytecode, Some(vec![1, 2, 3, 4]));
        assert_eq!(output.warnings.len(), 1);
        assert_eq!(output.compilation_time_ms, 100);
    }

    #[test]
    fn test_compilation_output_failure() {
        let output = CompilationOutput::failure("failed_module", vec!["compile error".to_string()]);
        assert_eq!(output.module_name, "failed_module");
        assert!(!output.success);
        assert!(output.bytecode.is_none());
        assert_eq!(output.errors, vec!["compile error"]);
    }

    #[test]
    fn test_batch_compilation_output() {
        let mut results = HashMap::new();
        results.insert(Atom::new("mod1"), CompilationResult {
            module_name: Atom::new("mod1"),
            bytecode: vec![],
            warnings: vec![],
            metadata: CompilationMetadata {
                compilation_time_ms: 50,
                source_size: 100,
                bytecode_size: 50,
                optimization_level: OptimizationLevel::Standard,
            },
        });

        let batch_result = BatchCompilationResult { results, errors: vec![] };
        let output = BatchCompilationOutput::from_batch_result(batch_result);

        assert_eq!(output.results.len(), 1);
        assert_eq!(output.summary.total_modules, 1);
        assert_eq!(output.summary.successful, 1);
        assert_eq!(output.summary.failed, 0);
    }

    #[test]
    fn test_compilation_request() {
        let request = CompilationRequest::new("test.erl", "module test.\nstart() -> ok.")
            .with_options(CompilationOptions {
                optimization_level: OptimizationLevel::Aggressive,
                warnings: false,
                ..Default::default()
            });

        assert_eq!(request.module_name, "test.erl");
        assert_eq!(request.source_code, "module test.\nstart() -> ok.");
        assert_eq!(request.options.optimization_level, OptimizationLevel::Aggressive);
        assert!(!request.options.warnings);
    }

    #[test]
    fn test_analysis_result() {
        let result = AnalysisResult {
            module_name: Atom::new("analysis_test"),
            syntax_valid: true,
            warnings: vec![],
            errors: vec![],
            metrics: SourceMetrics {
                lines_of_code: 42,
                functions: 3,
                complexity: 5,
            },
        };

        assert_eq!(result.module_name.as_str(), "analysis_test");
        assert!(result.syntax_valid);
        assert_eq!(result.metrics.lines_of_code, 42);
    }

    #[test]
    fn test_lsp_completion_item() {
        let item = lsp::CompletionItem {
            label: "lists:map/2".to_string(),
            kind: lsp::CompletionKind::Function,
            detail: Some("Apply function to each element".to_string()),
            documentation: Some("maps a function over a list".to_string()),
        };

        assert_eq!(item.label, "lists:map/2");
        assert!(matches!(item.kind, lsp::CompletionKind::Function));
        assert!(item.detail.is_some());
        assert!(item.documentation.is_some());
    }

    #[test]
    fn test_diagnostic_severity() {
        let diagnostic = lsp::Diagnostic {
            range: lsp::Range {
                start: Position { line: 1, column: 0, file: None },
                end: Position { line: 1, column: 10, file: None },
            },
            severity: lsp::DiagnosticSeverity::Error,
            code: Some("E001".to_string()),
            message: "Syntax error".to_string(),
            source: "erlc".to_string(),
        };

        assert_eq!(diagnostic.severity as u8, 1);
        assert_eq!(diagnostic.message, "Syntax error");
    }
}
