/*!
# Compilation Results

This module provides types and utilities for handling compilation results,
artifacts, and reporting compilation outcomes.
*/

use super::*;
use std::collections::BTreeMap;

/// Comprehensive compilation report
#[derive(Debug)]
pub struct CompilationReport {
    pub summary: CompilationSummary,
    pub results: Vec<CompilationResult>,
    pub errors: Vec<CompilationError>,
    pub warnings: Vec<CompilationWarning>,
    pub statistics: CompilationStats,
}

impl CompilationReport {
    pub fn new() -> Self {
        Self {
            summary: CompilationSummary::new(),
            results: Vec::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
            statistics: CompilationStats {
                total_modules: 0,
                successful_compilations: 0,
                failed_compilations: 0,
                total_warnings: 0,
                total_compilation_time_ms: 0,
            },
        }
    }

    pub fn add_result(&mut self, result: CompilationResult) {
        let warning_count = result.warnings.len();
        self.results.push(result.clone());
        self.warnings.extend(result.warnings);
        self.statistics.successful_compilations += 1;
        self.statistics.total_warnings += warning_count;
        self.statistics.total_compilation_time_ms += result.metadata.compilation_time_ms;
    }

    pub fn add_error(&mut self, error: CompilationError) {
        self.errors.push(error);
        self.statistics.failed_compilations += 1;
    }

    pub fn finalize(&mut self) {
        self.statistics.total_modules = self.results.len() + self.errors.len();
        self.summary = CompilationSummary::from_stats(&self.statistics);
    }

    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }
}

/// Summary of compilation results
#[derive(Debug, Clone)]
pub struct CompilationSummary {
    pub status: CompilationStatus,
    pub message: String,
    pub success_rate: f64,
}

impl CompilationSummary {
    pub fn new() -> Self {
        Self {
            status: CompilationStatus::Unknown,
            message: "Compilation not started".to_string(),
            success_rate: 0.0,
        }
    }

    pub fn from_stats(stats: &CompilationStats) -> Self {
        let total = stats.total_modules;
        if total == 0 {
            return Self {
                status: CompilationStatus::NoModules,
                message: "No modules to compile".to_string(),
                success_rate: 0.0,
            };
        }

        let success_rate = (stats.successful_compilations as f64 / total as f64) * 100.0;

        let (status, message) = if stats.failed_compilations == 0 {
            (CompilationStatus::Success, "All modules compiled successfully".to_string())
        } else if stats.successful_compilations == 0 {
            (CompilationStatus::Failure, "All modules failed to compile".to_string())
        } else {
            (CompilationStatus::Partial,
             format!("{}/{} modules compiled successfully", stats.successful_compilations, total))
        };

        Self {
            status,
            message,
            success_rate,
        }
    }
}

/// Overall compilation status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompilationStatus {
    Unknown,
    NoModules,
    Success,
    Partial,
    Failure,
}

/// Compilation artifact collection
#[derive(Debug)]
pub struct CompilationArtifacts {
    pub modules: BTreeMap<Atom, CompilationArtifact>,
    pub report: CompilationReport,
}

impl CompilationArtifacts {
    pub fn new() -> Self {
        Self {
            modules: BTreeMap::new(),
            report: CompilationReport::new(),
        }
    }

    pub fn add_artifact(&mut self, module: Atom, artifact: CompilationArtifact) {
        self.modules.insert(module, artifact);
    }

    pub fn get_artifact(&self, module: &Atom) -> Option<&CompilationArtifact> {
        self.modules.get(module)
    }

    pub fn total_size(&self) -> usize {
        self.modules.values().map(|a| a.size()).sum()
    }

    pub fn module_count(&self) -> usize {
        self.modules.len()
    }
}

/// Error reporting utilities
pub struct ErrorReporter {
    pub errors: Vec<CompilationError>,
    pub warnings: Vec<CompilationWarning>,
}

impl ErrorReporter {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, phase: CompilationPhase, error: CompilerError) {
        self.errors.push(CompilationError::new(phase, error));
    }

    pub fn add_warning(&mut self, message: String, position: Position, code: WarningCode) {
        self.warnings.push(CompilationWarning {
            message,
            position,
            code,
        });
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    pub fn clear(&mut self) {
        self.errors.clear();
        self.warnings.clear();
    }
}

impl Default for ErrorReporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Progress tracking for long compilations
#[derive(Debug, Clone)]
pub struct CompilationProgress {
    pub phase: CompilationPhase,
    pub module: Option<Atom>,
    pub progress: f64, // 0.0 to 1.0
    pub message: String,
}

impl CompilationProgress {
    pub fn new(phase: CompilationPhase, message: String) -> Self {
        Self {
            phase,
            module: None,
            progress: 0.0,
            message,
        }
    }

    pub fn for_module(mut self, module: Atom) -> Self {
        self.module = Some(module);
        self
    }

    pub fn with_progress(mut self, progress: f64) -> Self {
        self.progress = progress.clamp(0.0, 1.0);
        self
    }
}

/// Compilation cache for incremental builds
#[derive(Debug)]
pub struct CompilationCache {
    cache: std::collections::HashMap<String, CachedCompilation>,
}

#[derive(Debug, Clone)]
pub struct CachedCompilation {
    bytecode: Vec<u8>,
    source_hash: String,
    timestamp: std::time::SystemTime,
    dependencies: Vec<Atom>,
}

impl CompilationCache {
    pub fn new() -> Self {
        Self {
            cache: std::collections::HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&CachedCompilation> {
        self.cache.get(key)
    }

    pub fn put(&mut self, key: String, compilation: CachedCompilation) {
        self.cache.insert(key, compilation);
    }

    pub fn invalidate(&mut self, module: &Atom) {
        // Remove entries that depend on this module
        self.cache.retain(|_, cached| {
            !cached.dependencies.contains(module)
        });
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }

    pub fn size(&self) -> usize {
        self.cache.len()
    }
}

/// Diagnostic information for debugging compilation issues
#[derive(Debug, Clone)]
pub struct CompilationDiagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    pub position: Position,
    pub code: Option<String>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Info,
    Debug,
}

impl CompilationDiagnostic {
    pub fn error(message: String, position: Position) -> Self {
        Self {
            level: DiagnosticLevel::Error,
            message,
            position,
            code: None,
            suggestions: Vec::new(),
        }
    }

    pub fn warning(message: String, position: Position) -> Self {
        Self {
            level: DiagnosticLevel::Warning,
            message,
            position,
            code: None,
            suggestions: Vec::new(),
        }
    }

    pub fn with_code(mut self, code: String) -> Self {
        self.code = Some(code);
        self
    }

    pub fn with_suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.suggestions = suggestions;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compilation_report() {
        let mut report = CompilationReport::new();

        use entities_erlang_syntax::*;

        let ast = Module::new(Atom::new("test"));
        let result = CompilationResult {
            module_name: Atom::new("test"),
            ast,
            bytecode: vec![1, 2, 3],
            warnings: vec![
                CompilationWarning {
                    message: "warning".to_string(),
                    position: Position::default(),
                    code: WarningCode::UnusedVariable,
                },
            ],
            metadata: CompilationMetadata {
                compilation_time_ms: 100,
                source_size: 50,
                bytecode_size: 25,
                optimization_level: OptimizationLevel::Standard,
            },
            context_metadata: std::collections::HashMap::new(),
        };

        report.add_result(result);
        report.finalize();

        assert!(report.is_success());
        assert!(report.has_warnings());
        assert_eq!(report.results.len(), 1);
        assert_eq!(report.warnings.len(), 1);
        assert_eq!(report.statistics.total_warnings, 1);
    }

    #[test]
    fn test_compilation_summary() {
        let stats = CompilationStats {
            total_modules: 5,
            successful_compilations: 4,
            failed_compilations: 1,
            total_warnings: 2,
            total_compilation_time_ms: 500,
        };

        let summary = CompilationSummary::from_stats(&stats);
        assert_eq!(summary.status, CompilationStatus::Partial);
        assert_eq!(summary.success_rate, 80.0);
        assert!(summary.message.contains("4/5"));
    }

    #[test]
    fn test_compilation_artifacts() {
        let mut artifacts = CompilationArtifacts::new();

        artifacts.add_artifact(
            Atom::new("module1"),
            CompilationArtifact::BeamBytecode(vec![1, 2, 3]),
        );

        assert_eq!(artifacts.module_count(), 1);
        assert_eq!(artifacts.total_size(), 3);

        let artifact = artifacts.get_artifact(&Atom::new("module1"));
        assert!(artifact.is_some());
        assert_eq!(artifact.unwrap().size(), 3);
    }

    #[test]
    fn test_error_reporter() {
        let mut reporter = ErrorReporter::new();

        reporter.add_error(
            CompilationPhase::Analysis,
            CompilerError::InvalidArgument("test error".to_string()),
        );

        reporter.add_warning(
            "test warning".to_string(),
            Position::default(),
            WarningCode::UnusedVariable,
        );

        assert!(reporter.has_errors());
        assert!(reporter.has_warnings());
        assert_eq!(reporter.error_count(), 1);
        assert_eq!(reporter.warning_count(), 1);
    }

    #[test]
    fn test_compilation_progress() {
        let progress = CompilationProgress::new(
            CompilationPhase::CodeGeneration,
            "Generating bytecode".to_string(),
        )
        .for_module(Atom::new("test"))
        .with_progress(0.75);

        assert_eq!(progress.phase, CompilationPhase::CodeGeneration);
        assert_eq!(progress.module.as_ref().unwrap().as_str(), "test");
        assert_eq!(progress.progress, 0.75);
    }

    #[test]
    fn test_compilation_cache() {
        let mut cache = CompilationCache::new();

        let cached = CachedCompilation {
            bytecode: vec![1, 2, 3],
            source_hash: "hash123".to_string(),
            timestamp: std::time::SystemTime::now(),
            dependencies: vec![Atom::new("dep1")],
        };

        cache.put("key1".to_string(), cached.clone());

        assert_eq!(cache.size(), 1);
        assert!(cache.get("key1").is_some());
        assert!(cache.get("key2").is_none());

        cache.invalidate(&Atom::new("dep1"));
        assert_eq!(cache.size(), 0);
    }

    #[test]
    fn test_compilation_diagnostic() {
        let diagnostic = CompilationDiagnostic::error(
            "Syntax error".to_string(),
            Position { line: 10, column: 5, file: None },
        )
        .with_code("E001".to_string())
        .with_suggestions(vec![
            "Add missing semicolon".to_string(),
            "Check parentheses balance".to_string(),
        ]);

        assert_eq!(diagnostic.level, DiagnosticLevel::Error);
        assert_eq!(diagnostic.message, "Syntax error");
        assert_eq!(diagnostic.position.line, 10);
        assert_eq!(diagnostic.code.as_ref().unwrap(), "E001");
        assert_eq!(diagnostic.suggestions.len(), 2);
    }

    #[test]
    fn test_compilation_status() {
        let summary = CompilationSummary::from_stats(&CompilationStats {
            total_modules: 0,
            successful_compilations: 0,
            failed_compilations: 0,
            total_warnings: 0,
            total_compilation_time_ms: 0,
        });

        assert_eq!(summary.status, CompilationStatus::NoModules);

        let summary = CompilationSummary::from_stats(&CompilationStats {
            total_modules: 2,
            successful_compilations: 2,
            failed_compilations: 0,
            total_warnings: 0,
            total_compilation_time_ms: 100,
        });

        assert_eq!(summary.status, CompilationStatus::Success);

        let summary = CompilationSummary::from_stats(&CompilationStats {
            total_modules: 2,
            successful_compilations: 0,
            failed_compilations: 2,
            total_warnings: 0,
            total_compilation_time_ms: 50,
        });

        assert_eq!(summary.status, CompilationStatus::Failure);
    }
}
