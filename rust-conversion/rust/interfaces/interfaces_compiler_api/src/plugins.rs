/*!
# Compiler Plugin System

Extensible plugin architecture for customizing compiler behavior.
Plugins can add new compilation passes, modify ASTs, or extend functionality.
*/

use super::*;

/// Plugin trait for extending compiler functionality
#[async_trait::async_trait]
pub trait Plugin: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> PluginMetadata;

    /// Called before compilation starts
    async fn pre_compile(&self, _module: &Atom, _source: &str) -> APIResult<()> {
        Ok(())
    }

    /// Called after compilation completes
    async fn post_compile(&self, _module: &Atom, output: CompilationOutput) -> APIResult<CompilationOutput> {
        Ok(output)
    }

    /// Called during AST analysis phase
    async fn analyze_ast(&self, _module: &Module) -> APIResult<Vec<APIWarning>> {
        Ok(Vec::new())
    }

    /// Called during optimization phase
    async fn optimize(&self, _module: &Module) -> APIResult<Module> {
        Err(APIError::PluginError("Optimization not implemented".to_string()))
    }
}

/// Plugin metadata for identification and configuration
#[derive(Debug, Clone, )]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub capabilities: Vec<PluginCapability>,
}

impl PluginMetadata {
    pub fn new(name: &str, version: &str, description: &str, author: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            description: description.to_string(),
            author: author.to_string(),
            capabilities: Vec::new(),
        }
    }

    pub fn with_capabilities(mut self, capabilities: Vec<PluginCapability>) -> Self {
        self.capabilities = capabilities;
        self
    }
}

/// Plugin capabilities
#[derive(Debug, Clone, PartialEq, Eq, )]
pub enum PluginCapability {
    PreCompilation,
    PostCompilation,
    ASTAnalysis,
    Optimization,
    CodeGeneration,
    CustomPass(String),
}

/// Compilation pass plugin
pub struct CompilationPassPlugin {
    metadata: PluginMetadata,
    pass: Box<dyn usecases_compilation::CompilationPass>,
}

impl CompilationPassPlugin {
    pub fn new(name: &str, pass: Box<dyn usecases_compilation::CompilationPass>) -> Self {
        Self {
            metadata: PluginMetadata::new(
                name,
                "1.0.0",
                "Custom compilation pass",
                "Unknown",
            ).with_capabilities(vec![PluginCapability::CustomPass(name.to_string())]),
            pass,
        }
    }
}

#[async_trait::async_trait]
impl Plugin for CompilationPassPlugin {
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }

    async fn pre_compile(&self, module: &Atom, _source: &str) -> APIResult<()> {
        // Could integrate the pass into the compilation pipeline here
        println!("Custom pass '{}' activated for module {}", self.metadata.name, module);
        Ok(())
    }
}

/// AST transformation plugin
pub struct ASTTransformPlugin {
    metadata: PluginMetadata,
    transformer: Box<dyn Fn(&Module) -> APIResult<Module> + Send + Sync>,
}

impl ASTTransformPlugin {
    pub fn new<F>(name: &str, transformer: F) -> Self
    where
        F: Fn(&Module) -> APIResult<Module> + Send + Sync + 'static,
    {
        Self {
            metadata: PluginMetadata::new(
                name,
                "1.0.0",
                "AST transformation plugin",
                "Unknown",
            ).with_capabilities(vec![PluginCapability::Optimization]),
            transformer: Box::new(transformer),
        }
    }
}

#[async_trait::async_trait]
impl Plugin for ASTTransformPlugin {
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }

    async fn optimize(&self, module: &Module) -> APIResult<Module> {
        (self.transformer)(module)
    }
}

/// Warning generation plugin
pub struct WarningPlugin {
    metadata: PluginMetadata,
    warning_generator: Box<dyn Fn(&Module) -> Vec<APIWarning> + Send + Sync>,
}

impl WarningPlugin {
    pub fn new<F>(name: &str, generator: F) -> Self
    where
        F: Fn(&Module) -> Vec<APIWarning> + Send + Sync + 'static,
    {
        Self {
            metadata: PluginMetadata::new(
                name,
                "1.0.0",
                "Custom warning generation",
                "Unknown",
            ).with_capabilities(vec![PluginCapability::ASTAnalysis]),
            warning_generator: Box::new(generator),
        }
    }
}

#[async_trait::async_trait]
impl Plugin for WarningPlugin {
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }

    async fn analyze_ast(&self, module: &Module) -> APIResult<Vec<APIWarning>> {
        Ok((self.warning_generator)(module))
    }
}

/// Plugin manager for loading and managing plugins
pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
    enabled: bool,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            enabled: true,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn register_plugin(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub fn unregister_plugin(&mut self, name: &str) {
        self.plugins.retain(|p| p.metadata().name != name);
    }

    pub fn get_plugins(&self) -> Vec<PluginMetadata> {
        self.plugins.iter()
            .map(|p| p.metadata())
            .collect()
    }

    pub fn find_plugin(&self, name: &str) -> Option<&Box<dyn Plugin>> {
        self.plugins.iter()
            .find(|p| p.metadata().name == name)
    }

    pub fn clear(&mut self) {
        self.plugins.clear();
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Plugin loading and discovery utilities
pub mod loader {
    use super::*;

    /// Load plugin from dynamic library (placeholder for future implementation)
    pub fn load_from_library(_path: &str) -> APIResult<Box<dyn Plugin>> {
        Err(APIError::PluginError("Dynamic loading not implemented".to_string()))
    }

    /// Load plugin from configuration
    pub fn load_from_config(_config: &HashMap<String, String>) -> APIResult<Box<dyn Plugin>> {
        Err(APIError::PluginError("Configuration loading not implemented".to_string()))
    }

    /// Validate plugin compatibility
    pub fn validate_plugin(_plugin: &dyn Plugin) -> APIResult<()> {
        // Check version compatibility, capabilities, etc.
        Ok(())
    }

    /// Get plugin loading errors (for debugging)
    pub fn get_loading_errors() -> Vec<String> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPlugin;

    #[async_trait::async_trait]
    impl Plugin for TestPlugin {
        fn metadata(&self) -> PluginMetadata {
            PluginMetadata::new(
                "test_plugin",
                "1.0.0",
                "Test plugin",
                "Test Author",
            ).with_capabilities(vec![PluginCapability::PreCompilation])
        }
    }

    #[test]
    fn test_plugin_metadata() {
        let plugin = TestPlugin;
        let metadata = plugin.metadata();

        assert_eq!(metadata.name, "test_plugin");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.capabilities.len(), 1);
    }

    #[test]
    fn test_plugin_manager() {
        let mut manager = PluginManager::new();

        manager.register_plugin(Box::new(TestPlugin));
        assert_eq!(manager.get_plugins().len(), 1);

        let plugin = manager.find_plugin("test_plugin");
        assert!(plugin.is_some());
        assert_eq!(plugin.unwrap().metadata().name, "test_plugin");

        manager.unregister_plugin("test_plugin");
        assert_eq!(manager.get_plugins().len(), 0);
    }

    #[test]
    fn test_ast_transform_plugin() {
        let transformer = |module: &Module| {
            // Simple identity transformation
            Ok(module.clone())
        };

        let plugin = ASTTransformPlugin::new("identity", transformer);
        assert_eq!(plugin.metadata().name, "identity");
        assert!(plugin.metadata().capabilities.contains(&PluginCapability::Optimization));
    }

    #[test]
    fn test_warning_plugin() {
        let generator = |module: &Module| {
            vec![
                APIWarning {
                    message: format!("Warning for module {}", module.name),
                    line: 1,
                    column: 1,
                    code: "TEST".to_string(),
                }
            ]
        };

        let plugin = WarningPlugin::new("warning_test", generator);
        assert_eq!(plugin.metadata().name, "warning_test");
        assert!(plugin.metadata().capabilities.contains(&PluginCapability::ASTAnalysis));
    }

    #[tokio::test]
    async fn test_warning_plugin_execution() {
        let generator = |module: &Module| {
            vec![
                APIWarning {
                    message: format!("Warning for module {}", module.name),
                    line: 1,
                    column: 1,
                    code: "TEST".to_string(),
                }
            ]
        };

        let plugin = WarningPlugin::new("warning_test", generator);
        let module = Module::new(Atom::new("test_module"));

        let warnings = plugin.analyze_ast(&module).await.unwrap();
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("test_module"));
    }

    #[test]
    fn test_plugin_capabilities() {
        let capabilities = vec![
            PluginCapability::PreCompilation,
            PluginCapability::PostCompilation,
            PluginCapability::ASTAnalysis,
            PluginCapability::Optimization,
            PluginCapability::CodeGeneration,
            PluginCapability::CustomPass("special_pass".to_string()),
        ];

        assert_eq!(capabilities.len(), 6);
    }

    #[test]
    fn test_plugin_manager_enable_disable() {
        let mut manager = PluginManager::new();
        assert!(manager.enabled);

        manager.set_enabled(false);
        assert!(!manager.enabled);

        manager.set_enabled(true);
        assert!(manager.enabled);
    }
}
