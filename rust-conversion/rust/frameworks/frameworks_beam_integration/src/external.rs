/*!
# External Framework Bindings

This module handles integration with external frameworks, libraries,
and native code including NIFs (Native Implemented Functions).
*/

use super::*;

/// External bindings manager
pub struct ExternalBindings {
    nif_registry: NIFRegistry,
    port_registry: PortRegistry,
    driver_registry: DriverRegistry,
    loaded_libraries: HashMap<String, LoadedLibrary>,
}

impl ExternalBindings {
    pub fn new() -> Self {
        Self {
            nif_registry: NIFRegistry::new(),
            port_registry: PortRegistry::new(),
            driver_registry: DriverRegistry::new(),
            loaded_libraries: HashMap::new(),
        }
    }

    /// Register external bindings for a module
    pub fn register_module_bindings(&mut self, module: &LoadedModule) -> BeamResult<()> {
        // Check if module has external dependencies
        if let Some(nif_info) = self.extract_nif_info(module) {
            self.nif_registry.register_nif(&module.name, nif_info)?;
        }

        // Register any required drivers
        self.register_module_drivers(module)?;

        Ok(())
    }

    /// Unregister external bindings for a module
    pub fn unregister_module_bindings(&mut self, module: &LoadedModule) -> BeamResult<()> {
        self.nif_registry.unregister_nif(&module.name)?;
        self.port_registry.close_module_ports(&module.name)?;
        Ok(())
    }

    /// Load an external library
    pub fn load_library(&mut self, name: &str, path: &str) -> BeamResult<()> {
        // In a real implementation, this would use dlopen/LoadLibrary
        let library = LoadedLibrary {
            name: name.to_string(),
            path: path.to_string(),
            handle: 0, // Placeholder
            functions: Vec::new(),
        };

        self.loaded_libraries.insert(name.to_string(), library);
        Ok(())
    }

    /// Call a function from an external library
    pub fn call_external_function(
        &self,
        library_name: &str,
        function_name: &str,
        args: &[BeamValue],
    ) -> BeamResult<BeamValue> {
        let _library = self.loaded_libraries.get(library_name)
            .ok_or_else(|| BeamError::RuntimeError(format!("Library not found: {}", library_name)))?;

        // Simulate external function call
        match (library_name, function_name) {
            ("math", "sqrt") => {
                if args.len() == 1 {
                    if let BeamValue::Float(f) = &args[0] {
                        Ok(BeamValue::Float(f.sqrt()))
                    } else {
                        Err(BeamError::RuntimeError("sqrt expects float argument".to_string()))
                    }
                } else {
                    Err(BeamError::RuntimeError("sqrt expects one argument".to_string()))
                }
            }
            _ => Err(BeamError::FunctionNotFound(format!("{}:{}", library_name, function_name))),
        }
    }

    /// Shutdown all external bindings
    pub fn shutdown(&mut self) -> BeamResult<()> {
        // Clean up all external resources
        self.nif_registry.clear();
        self.port_registry.clear();
        self.driver_registry.clear();
        self.loaded_libraries.clear();
        Ok(())
    }

    fn extract_nif_info(&self, _module: &LoadedModule) -> Option<NIFInfo> {
        // In a real implementation, this would parse NIF declarations
        // from the module attributes or metadata
        None
    }

    fn register_module_drivers(&mut self, _module: &LoadedModule) -> BeamResult<()> {
        // Register any drivers required by the module
        Ok(())
    }
}

impl Default for ExternalBindings {
    fn default() -> Self {
        Self::new()
    }
}

/// Loaded external library
#[derive(Debug, Clone)]
pub struct LoadedLibrary {
    pub name: String,
    pub path: String,
    pub handle: usize, // Library handle (would be *mut c_void in real implementation)
    pub functions: Vec<String>,
}

/// NIF (Native Implemented Function) registry
pub struct NIFRegistry {
    nifs: HashMap<String, NIFInfo>,
}

impl NIFRegistry {
    pub fn new() -> Self {
        Self {
            nifs: HashMap::new(),
        }
    }

    /// Register a NIF for a module
    pub fn register_nif(&mut self, module_name: &str, nif_info: NIFInfo) -> BeamResult<()> {
        self.nifs.insert(module_name.to_string(), nif_info);
        Ok(())
    }

    /// Unregister a NIF
    pub fn unregister_nif(&mut self, module_name: &str) -> BeamResult<()> {
        self.nifs.remove(module_name);
        Ok(())
    }

    /// Call a NIF function
    pub fn call_nif(
        &self,
        module_name: &str,
        function_name: &str,
        args: &[BeamValue],
    ) -> BeamResult<BeamValue> {
        let nif_info = self.nifs.get(module_name)
            .ok_or_else(|| BeamError::RuntimeError(format!("NIF not found for module: {}", module_name)))?;

        nif_info.call_function(function_name, args)
    }

    /// Clear all NIFs
    pub fn clear(&mut self) {
        self.nifs.clear();
    }
}

impl Default for NIFRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// NIF information
#[derive(Debug, Clone)]
pub struct NIFInfo {
    pub module: String,
    pub functions: Vec<NIFunction>,
}

impl NIFInfo {
    pub fn new(module: String) -> Self {
        Self {
            module,
            functions: Vec::new(),
        }
    }

    pub fn add_function(&mut self, function: NIFunction) {
        self.functions.push(function);
    }

    pub fn call_function(&self, name: &str, args: &[BeamValue]) -> BeamResult<BeamValue> {
        let function = self.functions.iter()
            .find(|f| f.name == name)
            .ok_or_else(|| BeamError::FunctionNotFound(format!("NIF function: {}", name)))?;

        function.call(args)
    }
}

/// NIF function definition
#[derive(Debug, Clone)]
pub struct NIFunction {
    pub name: String,
    pub arity: usize,
    pub implementation: Box<dyn Fn(&[BeamValue]) -> BeamResult<BeamValue> + Send + Sync>,
}

impl NIFunction {
    pub fn new<F>(name: &str, arity: usize, implementation: F) -> Self
    where
        F: Fn(&[BeamValue]) -> BeamResult<BeamValue> + Send + Sync + 'static,
    {
        Self {
            name: name.to_string(),
            arity,
            implementation: Box::new(implementation),
        }
    }

    pub fn call(&self, args: &[BeamValue]) -> BeamResult<BeamValue> {
        if args.len() != self.arity {
            return Err(BeamError::RuntimeError(
                format!("NIF {} expects {} arguments, got {}", self.name, self.arity, args.len())
            ));
        }
        (self.implementation)(args)
    }
}

/// Port registry for external program communication
pub struct PortRegistry {
    ports: HashMap<String, PortInfo>,
}

impl PortRegistry {
    pub fn new() -> Self {
        Self {
            ports: HashMap::new(),
        }
    }

    /// Open a port to an external program
    pub fn open_port(&mut self, command: &str, args: &[String]) -> BeamResult<String> {
        let port_id = format!("port_{}", self.ports.len());
        let port_info = PortInfo {
            id: port_id.clone(),
            command: command.to_string(),
            args: args.to_vec(),
            status: PortStatus::Open,
        };

        self.ports.insert(port_id.clone(), port_info);
        Ok(port_id)
    }

    /// Send data to a port
    pub async fn send_to_port(&self, port_id: &str, data: &[u8]) -> BeamResult<()> {
        let _port = self.ports.get(port_id)
            .ok_or_else(|| BeamError::RuntimeError(format!("Port not found: {}", port_id)))?;

        // Simulate sending data
        println!("Sending {} bytes to port {}", data.len(), port_id);
        Ok(())
    }

    /// Receive data from a port
    pub async fn receive_from_port(&self, port_id: &str) -> BeamResult<Vec<u8>> {
        let _port = self.ports.get(port_id)
            .ok_or_else(|| BeamError::RuntimeError(format!("Port not found: {}", port_id)))?;

        // Simulate receiving data
        Ok(vec![1, 2, 3, 4]) // Placeholder data
    }

    /// Close a port
    pub fn close_port(&mut self, port_id: &str) -> BeamResult<()> {
        if let Some(port) = self.ports.get_mut(port_id) {
            port.status = PortStatus::Closed;
        }
        Ok(())
    }

    /// Close all ports for a module
    pub fn close_module_ports(&mut self, module_name: &str) -> BeamResult<()> {
        // Close ports associated with the module
        // In a real implementation, this would track port ownership
        Ok(())
    }

    /// Clear all ports
    pub fn clear(&mut self) {
        self.ports.clear();
    }
}

impl Default for PortRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Port information
#[derive(Debug, Clone)]
pub struct PortInfo {
    pub id: String,
    pub command: String,
    pub args: Vec<String>,
    pub status: PortStatus,
}

/// Port status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortStatus {
    Open,
    Closed,
    Error,
}

/// Driver registry for low-level system interfaces
pub struct DriverRegistry {
    drivers: HashMap<String, DriverInfo>,
}

impl DriverRegistry {
    pub fn new() -> Self {
        Self {
            drivers: HashMap::new(),
        }
    }

    /// Register a driver
    pub fn register_driver(&mut self, name: &str, driver: DriverInfo) -> BeamResult<()> {
        self.drivers.insert(name.to_string(), driver);
        Ok(())
    }

    /// Call a driver function
    pub fn call_driver(&self, name: &str, command: &str, data: &[u8]) -> BeamResult<Vec<u8>> {
        let driver = self.drivers.get(name)
            .ok_or_else(|| BeamError::RuntimeError(format!("Driver not found: {}", name)))?;

        driver.call(command, data)
    }

    /// Clear all drivers
    pub fn clear(&mut self) {
        self.drivers.clear();
    }
}

impl Default for DriverRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Driver information
#[derive(Debug, Clone)]
pub struct DriverInfo {
    pub name: String,
    pub implementation: Box<dyn Fn(&str, &[u8]) -> BeamResult<Vec<u8>> + Send + Sync>,
}

impl DriverInfo {
    pub fn new<F>(name: &str, implementation: F) -> Self
    where
        F: Fn(&str, &[u8]) -> BeamResult<Vec<u8>> + Send + Sync + 'static,
    {
        Self {
            name: name.to_string(),
            implementation: Box::new(implementation),
        }
    }

    pub fn call(&self, command: &str, data: &[u8]) -> BeamResult<Vec<u8>> {
        (self.implementation)(command, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_external_bindings_creation() {
        let bindings = ExternalBindings::new();
        assert!(bindings.loaded_libraries.is_empty());
    }

    #[test]
    fn test_load_library() {
        let mut bindings = ExternalBindings::new();

        let result = bindings.load_library("math", "/lib/libmath.so");
        assert!(result.is_ok());

        assert!(bindings.loaded_libraries.contains_key("math"));
        let lib = &bindings.loaded_libraries["math"];
        assert_eq!(lib.path, "/lib/libmath.so");
    }

    #[test]
    fn test_call_external_function() {
        let mut bindings = ExternalBindings::new();
        bindings.load_library("math", "/lib/libmath.so").unwrap();

        let result = bindings.call_external_function("math", "sqrt", &[BeamValue::Float(16.0)]);
        assert!(result.is_ok());

        if let BeamValue::Float(val) = result.unwrap() {
            assert!((val - 4.0).abs() < 0.001);
        } else {
            panic!("Expected float result");
        }
    }

    #[test]
    fn test_nif_registry() {
        let mut registry = NIFRegistry::new();

        let mut nif_info = NIFInfo::new("test_module".to_string());
        let nif_func = NIFunction::new("double", 1, |args| {
            match &args[0] {
                BeamValue::Integer(i) => Ok(BeamValue::Integer(i * 2)),
                _ => Err(BeamError::RuntimeError("Expected integer".to_string())),
            }
        });
        nif_info.add_function(nif_func);

        registry.register_nif("test_module", nif_info).unwrap();

        let result = registry.call_nif("test_module", "double", &[BeamValue::Integer(5)]);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), BeamValue::Integer(10)));
    }

    #[test]
    fn test_port_registry() {
        let mut registry = PortRegistry::new();

        let port_id = registry.open_port("cat", &["/etc/hosts".to_string()]).unwrap();
        assert!(registry.ports.contains_key(&port_id));

        registry.close_port(&port_id).unwrap();
        assert_eq!(registry.ports[&port_id].status, PortStatus::Closed);
    }

    #[test]
    fn test_driver_registry() {
        let mut registry = DriverRegistry::new();

        let driver = DriverInfo::new("file", |command, data| {
            match command {
                "read" => Ok(data.to_vec()),
                _ => Err(BeamError::RuntimeError("Unknown command".to_string())),
            }
        });

        registry.register_driver("file", driver).unwrap();

        let result = registry.call_driver("file", "read", &[1, 2, 3]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![1, 2, 3]);
    }

    #[test]
    fn test_nif_function() {
        let nif_func = NIFunction::new("identity", 1, |args| {
            Ok(args[0].clone())
        });

        let result = nif_func.call(&[BeamValue::Integer(42)]);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), BeamValue::Integer(42)));
    }

    #[test]
    fn test_nif_function_wrong_arity() {
        let nif_func = NIFunction::new("test", 2, |_args| {
            Ok(BeamValue::Atom("ok".to_string()))
        });

        let result = nif_func.call(&[BeamValue::Integer(1)]);
        assert!(result.is_err());
    }

    #[test]
    fn test_loaded_library() {
        let library = LoadedLibrary {
            name: "test".to_string(),
            path: "/path/to/lib".to_string(),
            handle: 12345,
            functions: vec!["func1".to_string(), "func2".to_string()],
        };

        assert_eq!(library.name, "test");
        assert_eq!(library.handle, 12345);
        assert_eq!(library.functions.len(), 2);
    }

    #[test]
    fn test_port_info() {
        let info = PortInfo {
            id: "port_1".to_string(),
            command: "ls".to_string(),
            args: vec!["-la".to_string()],
            status: PortStatus::Open,
        };

        assert_eq!(info.id, "port_1");
        assert_eq!(info.command, "ls");
        assert_eq!(info.status, PortStatus::Open);
    }

    #[test]
    fn test_driver_info() {
        let driver = DriverInfo::new("echo", |_cmd, data| {
            Ok(data.to_vec())
        });

        let result = driver.call("test", &[72, 101, 108, 108, 111]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![72, 101, 108, 108, 111]);
    }

    #[test]
    fn test_shutdown_bindings() {
        let mut bindings = ExternalBindings::new();
        bindings.load_library("test", "/tmp/lib").unwrap();

        assert!(!bindings.loaded_libraries.is_empty());

        bindings.shutdown().unwrap();
        assert!(bindings.loaded_libraries.is_empty());
    }
}
