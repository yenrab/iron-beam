/*!
# Core Compilation Logic

This module contains the main compilation execution logic that orchestrates
the transformation from Erlang source to BEAM bytecode.
*/

use super::*;

/// Result of compiling a single module
#[derive(Debug, Clone, )]
pub struct CompilationResult {
    pub module_name: Atom,
    pub ast: entities_erlang_syntax::Module,
    pub bytecode: Vec<u8>,
    pub warnings: Vec<CompilationWarning>,
    pub metadata: CompilationMetadata,
    pub context_metadata: std::collections::HashMap<String, String>,
}

/// Warning generated during compilation
#[derive(Debug, Clone, PartialEq, Eq, )]
pub struct CompilationWarning {
    pub message: String,
    pub position: Position,
    pub code: WarningCode,
}

/// Compilation warning codes
#[derive(Debug, Clone, PartialEq, Eq, )]
pub enum WarningCode {
    UnusedVariable,
    UnusedFunction,
    ShadowedVariable,
    MissingSpec,
    DeprecatedFunction,
    TypeMismatch,
    Other(String),
}

/// Metadata about the compilation process
#[derive(Debug, Clone, )]
pub struct CompilationMetadata {
    pub compilation_time_ms: u64,
    pub source_size: usize,
    pub bytecode_size: usize,
    pub optimization_level: OptimizationLevel,
}

impl Default for CompilationMetadata {
    fn default() -> Self {
        Self {
            compilation_time_ms: 0,
            source_size: 0,
            bytecode_size: 0,
            optimization_level: OptimizationLevel::Standard,
        }
    }
}

/// Result of compiling multiple modules
#[derive(Debug)]
pub struct BatchCompilationResult {
    pub results: HashMap<Atom, CompilationResult>,
    pub errors: Vec<(Atom, CompilerError)>,
}

impl BatchCompilationResult {
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn success_count(&self) -> usize {
        self.results.len()
    }

    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    pub fn total_modules(&self) -> usize {
        self.results.len() + self.errors.len()
    }
}

/// Compilation statistics
#[derive(Debug, Clone)]
pub struct CompilationStats {
    pub total_modules: usize,
    pub successful_compilations: usize,
    pub failed_compilations: usize,
    pub total_warnings: usize,
    pub total_compilation_time_ms: u64,
}

impl CompilationStats {
    pub fn from_batch_result(result: &BatchCompilationResult) -> Self {
        let total_warnings = result.results.values()
            .map(|r| r.warnings.len())
            .sum();

        let total_compilation_time = result.results.values()
            .map(|r| r.metadata.compilation_time_ms)
            .sum();

        Self {
            total_modules: result.total_modules(),
            successful_compilations: result.success_count(),
            failed_compilations: result.error_count(),
            total_warnings,
            total_compilation_time_ms: total_compilation_time,
        }
    }
}

/// Compilation phase tracking
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompilationPhase {
    Parsing,
    Analysis,
    Optimization,
    CodeGeneration,
    Linking,
    Complete,
}

impl std::fmt::Display for CompilationPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parsing => write!(f, "parsing"),
            Self::Analysis => write!(f, "analysis"),
            Self::Optimization => write!(f, "optimization"),
            Self::CodeGeneration => write!(f, "code generation"),
            Self::Linking => write!(f, "linking"),
            Self::Complete => write!(f, "complete"),
        }
    }
}

/// Progress callback for compilation monitoring
pub type CompilationProgressCallback = Box<dyn Fn(CompilationPhase, &str) + Send + Sync>;

/// Compilation configuration for different targets
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetConfig {
    pub platform: String,
    pub architecture: String,
    pub word_size: usize,
    pub endianness: Endianness,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Endianness {
    Little,
    Big,
}

impl Default for TargetConfig {
    fn default() -> Self {
        Self {
            platform: "generic".to_string(),
            architecture: "beam".to_string(),
            word_size: 64,
            endianness: Endianness::Little,
        }
    }
}

/// Compilation error with context
#[derive(Debug)]
pub struct CompilationError {
    pub phase: CompilationPhase,
    pub error: CompilerError,
    pub context: Option<String>,
}

impl CompilationError {
    pub fn new(phase: CompilationPhase, error: CompilerError) -> Self {
        Self {
            phase,
            error,
            context: None,
        }
    }

    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }
}

impl From<CompilationError> for CompilerError {
    fn from(err: CompilationError) -> Self {
        err.error
    }
}

/// Compilation artifact types
#[derive(Debug, Clone)]
pub enum CompilationArtifact {
    BeamBytecode(Vec<u8>),
    Assembly(String),
    CoreErlang(String),
    KernelErlang(String),
    AbstractSyntaxTree(Module),
}

impl CompilationArtifact {
    pub fn size(&self) -> usize {
        match self {
            Self::BeamBytecode(bytes) => bytes.len(),
            Self::Assembly(code) |
            Self::CoreErlang(code) |
            Self::KernelErlang(code) => code.len(),
            Self::AbstractSyntaxTree(_) => 0, // Would need to calculate AST size
        }
    }

    pub fn format_name(&self) -> &'static str {
        match self {
            Self::BeamBytecode(_) => "BEAM",
            Self::Assembly(_) => "ASM",
            Self::CoreErlang(_) => "Core Erlang",
            Self::KernelErlang(_) => "Kernel Erlang",
            Self::AbstractSyntaxTree(_) => "AST",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compilation_result_creation() {
        use entities_erlang_syntax::*;

        let ast = Module::new(Atom::new("test_module"));
        let result = CompilationResult {
            module_name: Atom::new("test_module"),
            ast,
            bytecode: vec![1, 2, 3, 4],
            warnings: vec![],
            metadata: CompilationMetadata {
                compilation_time_ms: 100,
                source_size: 1000,
                bytecode_size: 500,
                optimization_level: OptimizationLevel::Standard,
            },
            context_metadata: std::collections::HashMap::new(),
        };

        assert_eq!(result.module_name.as_str(), "test_module");
        assert_eq!(result.bytecode.len(), 4);
        assert!(result.warnings.is_empty());
        assert_eq!(result.metadata.compilation_time_ms, 100);
    }

    #[test]
    fn test_batch_compilation_result() {
        let mut results = HashMap::new();
        results.insert(Atom::new("module1"), CompilationResult {
            module_name: Atom::new("module1"),
            ast: entities_erlang_syntax::Module::new(Atom::new("module1")),
            bytecode: vec![],
            warnings: vec![],
            metadata: CompilationMetadata {
                compilation_time_ms: 50,
                source_size: 100,
                bytecode_size: 50,
                optimization_level: OptimizationLevel::Standard,
            },
            context_metadata: std::collections::HashMap::new(),
        });

        let errors = vec![
            (Atom::new("module2"), CompilerError::InvalidArgument("test error".to_string())),
        ];

        let batch_result = BatchCompilationResult { results, errors };

        assert!(batch_result.has_errors());
        assert_eq!(batch_result.success_count(), 1);
        assert_eq!(batch_result.error_count(), 1);
        assert_eq!(batch_result.total_modules(), 2);
    }

    #[test]
    fn test_compilation_stats() {
        let batch_result = BatchCompilationResult {
            results: {
                let mut map = HashMap::new();
                map.insert(Atom::new("mod1"), CompilationResult {
                    module_name: Atom::new("mod1"),
                    ast: entities_erlang_syntax::Module::new(Atom::new("mod1")),
                    bytecode: vec![],
                    warnings: vec![
                        CompilationWarning {
                            message: "warning1".to_string(),
                            position: Position::default(),
                            code: WarningCode::UnusedVariable,
                        },
                        CompilationWarning {
                            message: "warning2".to_string(),
                            position: Position::default(),
                            code: WarningCode::UnusedFunction,
                        },
                    ],
                    metadata: CompilationMetadata {
                        compilation_time_ms: 100,
                        source_size: 200,
                        bytecode_size: 100,
                        optimization_level: OptimizationLevel::Standard,
                    },
                    context_metadata: std::collections::HashMap::new(),
                });
                map
            },
            errors: vec![(Atom::new("mod2"), CompilerError::InvalidArgument("error".to_string()))],
        };

        let stats = CompilationStats::from_batch_result(&batch_result);

        assert_eq!(stats.total_modules, 2);
        assert_eq!(stats.successful_compilations, 1);
        assert_eq!(stats.failed_compilations, 1);
        assert_eq!(stats.total_warnings, 2);
        assert_eq!(stats.total_compilation_time_ms, 100);
    }

    #[test]
    fn test_compilation_warning() {
        let warning = CompilationWarning {
            message: "Variable X is unused".to_string(),
            position: Position { line: 10, column: 5, file: None },
            code: WarningCode::UnusedVariable,
        };

        assert_eq!(warning.message, "Variable X is unused");
        assert_eq!(warning.position.line, 10);
        assert_eq!(warning.position.column, 5);
        assert_eq!(warning.code, WarningCode::UnusedVariable);
    }

    #[test]
    fn test_compilation_phases() {
        assert_eq!(CompilationPhase::Parsing.to_string(), "parsing");
        assert_eq!(CompilationPhase::Analysis.to_string(), "analysis");
        assert_eq!(CompilationPhase::Optimization.to_string(), "optimization");
        assert_eq!(CompilationPhase::CodeGeneration.to_string(), "code generation");
        assert_eq!(CompilationPhase::Linking.to_string(), "linking");
        assert_eq!(CompilationPhase::Complete.to_string(), "complete");
    }

    #[test]
    fn test_target_config() {
        let config = TargetConfig::default();
        assert_eq!(config.platform, "generic");
        assert_eq!(config.architecture, "beam");
        assert_eq!(config.word_size, 64);
        assert_eq!(config.endianness, Endianness::Little);
    }

    #[test]
    fn test_compilation_artifact_sizes() {
        let bytecode = CompilationArtifact::BeamBytecode(vec![1, 2, 3]);
        assert_eq!(bytecode.size(), 3);

        let assembly = CompilationArtifact::Assembly("code".to_string());
        assert_eq!(assembly.size(), 4);
    }

    #[test]
    fn test_compilation_artifact_format_names() {
        assert_eq!(CompilationArtifact::BeamBytecode(vec![]).format_name(), "BEAM");
        assert_eq!(CompilationArtifact::Assembly("".to_string()).format_name(), "ASM");
        assert_eq!(CompilationArtifact::CoreErlang("".to_string()).format_name(), "Core Erlang");
    }
}
