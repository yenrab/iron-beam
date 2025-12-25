/*!
# Communication Protocols

Protocol implementations for external communication with the compiler,
including Language Server Protocol (LSP), build protocols, and custom protocols.
*/

use super::*;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

/// Language Server Protocol implementation
pub mod lsp_protocol {
    use super::*;

    /// LSP server for IDE integration
    pub struct LanguageServer {
        compiler_api: CompilerAPI,
    }

    impl LanguageServer {
        pub fn new(compiler_api: CompilerAPI) -> Self {
            Self { compiler_api }
        }

        /// Handle LSP initialize request
        pub async fn initialize(&self, _params: serde_json::Value) -> serde_json::Value {
            serde_json::json!({
                "capabilities": {
                    "textDocumentSync": 1,
                    "completionProvider": {
                        "triggerCharacters": [":"]
                    },
                    "diagnosticProvider": true,
                    "hoverProvider": true
                },
                "serverInfo": {
                    "name": "Erlang Compiler",
                    "version": self.compiler_api.get_info().version
                }
            })
        }

        /// Handle text document completion
        pub async fn completion(&self, params: serde_json::Value) -> APIResult<serde_json::Value> {
            let position = params["position"].as_object()
                .and_then(|pos| {
                    let line = pos["line"].as_u64()?;
                    let character = pos["character"].as_u64()?;
                    Some(Position {
                        line: line as usize,
                        column: character as usize,
                        file: None,
                    })
                })
                .unwrap_or_default();

            // In a real implementation, this would analyze the source at the position
            // and provide completion suggestions
            let completions = vec![
                lsp::CompletionItem {
                    label: "lists:map/2".to_string(),
                    kind: lsp::CompletionKind::Function,
                    detail: Some("Apply function to each element".to_string()),
                    documentation: Some("maps a function over a list".to_string()),
                },
                lsp::CompletionItem {
                    label: "io:format/1".to_string(),
                    kind: lsp::CompletionKind::Function,
                    detail: Some("Format and print".to_string()),
                    documentation: Some("prints formatted text".to_string()),
                },
            ];

            Ok(serde_json::json!({
                "items": completions
            }))
        }

        /// Handle text document diagnostics
        pub async fn diagnostics(&self, params: serde_json::Value) -> APIResult<serde_json::Value> {
            let uri = params["textDocument"]["uri"].as_str()
                .unwrap_or("file:///unknown.erl");

            // In a real implementation, this would compile the document and return diagnostics
            let diagnostics = vec![
                lsp::Diagnostic {
                    range: lsp::Range {
                        start: Position { line: 1, column: 0, file: None },
                        end: Position { line: 1, column: 10, file: None },
                    },
                    severity: lsp::DiagnosticSeverity::Warning,
                    code: Some("unused_variable".to_string()),
                    message: "Variable 'X' is unused".to_string(),
                    source: "erlc".to_string(),
                }
            ];

            Ok(serde_json::json!({
                "diagnostics": diagnostics
            }))
        }

        /// Handle shutdown request
        pub async fn shutdown(&self) -> serde_json::Value {
            serde_json::json!({})
        }
    }
}

/// Build protocol for build system integration
pub mod build_protocol {
    use super::*;

    /// Build server for handling build requests
    pub struct BuildServer {
        compiler_api: CompilerAPI,
    }

    impl BuildServer {
        pub fn new(compiler_api: CompilerAPI) -> Self {
            Self { compiler_api }
        }

        /// Handle build request
        pub async fn build(&self, request: build::BuildRequest) -> APIResult<build::BuildResponse> {
            let mut compiled_modules = Vec::new();
            let mut errors = Vec::new();
            let mut warnings = Vec::new();
            let start_time = std::time::Instant::now();

            // Convert request to batch compilation
            let sources: HashMap<String, String> = request.target_modules.into_iter()
                .map(|module| (module.clone(), "".to_string())) // Would need actual source
                .collect();

            match self.compiler_api.compile_modules(sources).await {
                Ok(result) => {
                    for (module_name, output) in result.results {
                        compiled_modules.push(module_name);
                        warnings.extend(output.warnings.into_iter().map(|w| w.message));
                    }
                    // Errors would be in result.errors
                }
                Err(e) => {
                    errors.push(e.to_string());
                }
            }

            let build_time = start_time.elapsed().as_millis() as u64;

            Ok(build::BuildResponse {
                success: errors.is_empty(),
                compiled_modules,
                errors,
                warnings,
                build_time_ms: build_time,
            })
        }

        /// Get dependency information
        pub async fn get_dependencies(&self, module: &str) -> APIResult<build::DependencyInfo> {
            // In a real implementation, this would analyze the module's dependencies
            Ok(build::DependencyInfo {
                module: module.to_string(),
                dependencies: vec!["lists".to_string(), "io".to_string()],
                dependents: vec!["my_app".to_string()],
            })
        }
    }
}

/// TCP-based protocol server
pub struct ProtocolServer {
    listener: TcpListener,
    compiler_api: CompilerAPI,
}

impl ProtocolServer {
    pub async fn new(address: &str, compiler_api: CompilerAPI) -> APIResult<Self> {
        let listener = TcpListener::bind(address)
            .map_err(|e| APIError::InvalidRequest(format!("Failed to bind to {}: {}", address, e)))?;

        Ok(Self {
            listener,
            compiler_api,
        })
    }

        /// Run the server (simplified implementation)
        pub async fn run(self) -> APIResult<()> {
            println!("Protocol server listening on {}", self.listener.local_addr().unwrap());

            loop {
                // For now, just handle one connection at a time
                // In a real implementation, you'd need to share the API properly
                match self.listener.accept() {
                    Ok((stream, addr)) => {
                        println!("New connection from {}", addr);
                        // Note: We're not cloning the API here to avoid complexity
                        // In production, you'd use Arc<Mutex<CompilerAPI>> or similar
                        break; // Exit after one connection for now
                    }
                    Err(e) => {
                        eprintln!("Accept error: {}", e);
                        break;
                    }
                }
            }

            Ok(())
        }
}

/// Handle individual client connections (placeholder)
async fn handle_connection(_stream: TcpStream, _api: &CompilerAPI) -> APIResult<()> {
    // Placeholder - in a real implementation, this would handle the protocol
    Ok(())
}

/// Process incoming requests (placeholder)
async fn process_request(request: &str, api: &CompilerAPI) -> serde_json::Value {
    // Simple JSON-RPC like processing
    match serde_json::from_str::<serde_json::Value>(request) {
        Ok(req) => {
            let method = req["method"].as_str().unwrap_or("");
            let params = req["params"].as_object().cloned().unwrap_or_default();

            match method {
                "compile" => {
                    // Extract module name and source
                    let module_name = params.get("module").and_then(|v| v.as_str()).unwrap_or("unknown");
                    let source = params.get("source").and_then(|v| v.as_str()).unwrap_or("");

                    match api.compile_source(module_name, source).await {
                        Ok(output) => serde_json::to_value(output).unwrap_or_default(),
                        Err(e) => serde_json::json!({"error": e.to_string()}),
                    }
                }
                "analyze" => {
                    let module_name = params.get("module").and_then(|v| v.as_str()).unwrap_or("unknown");
                    let source = params.get("source").and_then(|v| v.as_str()).unwrap_or("");

                    match api.analyze_source(module_name, source).await {
                        Ok(result) => serde_json::to_value(result).unwrap_or_default(),
                        Err(e) => serde_json::json!({"error": e.to_string()}),
                    }
                }
                "info" => {
                    serde_json::to_value(api.get_info()).unwrap_or_default()
                }
                _ => serde_json::json!({"error": "Unknown method"}),
            }
        }
        Err(_) => serde_json::json!({"error": "Invalid JSON"}),
    }
}

/// HTTP REST API server (placeholder for future implementation)
pub mod rest_api {
    use super::*;

    pub struct RESTServer {
        compiler_api: CompilerAPI,
    }

    impl RESTServer {
        pub fn new(compiler_api: CompilerAPI) -> Self {
            Self { compiler_api }
        }

        /// Placeholder for REST endpoints
        pub fn routes(&self) -> Vec<String> {
            vec![
                "POST /compile".to_string(),
                "POST /analyze".to_string(),
                "GET /info".to_string(),
                "GET /health".to_string(),
            ]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_server_creation() {
        // Can't easily test TCP server creation in unit tests
        // This would require integration tests
    }

    #[tokio::test]
    async fn test_lsp_server() {
        let api = CompilerAPI::new();
        let lsp_server = lsp_protocol::LanguageServer::new(api);

        let init_params = serde_json::json!({
            "processId": 12345,
            "clientInfo": {
                "name": "test-client"
            }
        });

        let response = lsp_server.initialize(init_params).await;

        assert!(response["capabilities"]["completionProvider"].is_object());
        assert!(response["serverInfo"]["name"].as_str().unwrap().contains("Erlang"));
    }

    #[test]
    fn test_process_request_compile() {
        // Test the request processing logic
        let request = r#"{"method": "compile", "params": {"module": "test", "source": "module test.\nstart() -> ok."}}"#;
        let api = CompilerAPI::new();

        // This would normally be async, but we're testing the JSON parsing
        let parsed: serde_json::Value = serde_json::from_str(request).unwrap();
        assert_eq!(parsed["method"], "compile");
        assert_eq!(parsed["params"]["module"], "test");
    }

    #[test]
    fn test_process_request_info() {
        let request = r#"{"method": "info"}"#;
        let api = CompilerAPI::new();

        // Test JSON parsing
        let parsed: serde_json::Value = serde_json::from_str(request).unwrap();
        assert_eq!(parsed["method"], "info");
    }

    #[test]
    fn test_rest_api_routes() {
        let api = CompilerAPI::new();
        let rest_server = rest_api::RESTServer::new(api);

        let routes = rest_server.routes();
        assert!(routes.contains(&"POST /compile".to_string()));
        assert!(routes.contains(&"GET /info".to_string()));
        assert!(routes.contains(&"GET /health".to_string()));
    }

    #[tokio::test]
    async fn test_build_server_dependencies() {
        let api = CompilerAPI::new();
        let build_server = build_protocol::BuildServer::new(api);

        let deps = build_server.get_dependencies("test_module").await.unwrap();
        assert_eq!(deps.module, "test_module");
        assert!(!deps.dependencies.is_empty());
    }

    #[test]
    fn test_lsp_completion_structure() {
        // Test that LSP completion items have the expected structure
        let item = lsp::CompletionItem {
            label: "test".to_string(),
            kind: lsp::CompletionKind::Function,
            detail: None,
            documentation: None,
        };

        // This would be serialized to JSON for LSP communication
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("test"));
        assert!(json.contains("Function"));
    }
}
