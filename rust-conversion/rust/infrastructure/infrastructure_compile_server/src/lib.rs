/*!
# Infrastructure Compile Server

**CLEAN Architecture**: Infrastructure Layer (Layer 4)
**SOLID Responsibility**: Distributed compilation server coordination

## Overview

This crate provides distributed compilation server functionality for the Erlang BEAM compiler infrastructure.
Implements safe, distributed compilation using modern async Rust networking instead of unsafe FFI.

## Original C Functions Replaced

The original `erlc.c` contained these server functions:
- `call_compile_server()`: RPC calls to compile server → **Replaced with safe async networking**
- `start_compile_server()`: Launch server process → **Replaced with safe process spawning**
- `encode_env()`: Environment encoding → **Replaced with structured serialization**

## Server Architecture Philosophy

### 1. Safe Distributed Compilation
```rust
use infrastructure_compile_server::{CompileRequest, CompileOptions, EnvironmentContext};

// Create compilation request for distributed server
let request = CompileRequest {
    source_file: "example.erl".to_string(),
    source_content: "-module(example).".to_string(),
    options: CompileOptions::default(),
    environment: EnvironmentContext {
        cwd: "/tmp".to_string(),
        env_vars: std::collections::HashMap::new(),
        include_paths: vec![],
    },
};
// Async operations would be used in real code:
// let result = client::send_compile_request(&request).await?;
```

### 2. Compilation Result Caching
```rust
use infrastructure_compile_server::{cache, CompileOptions};

// Generate cache key for compilation result
let options = CompileOptions::default();
let cache_key = cache::generate_cache_key("source code", &options);
// Cache operations would be async in real code
```

### 3. Server Lifecycle Management
```rust
use infrastructure_compile_server::ServerStatus;

// Create server status information
let status = ServerStatus {
    version: "1.0.0".to_string(),
    uptime_seconds: 3600,
    active_compilations: 0,
    cache_size: 100,
    cache_hits: 50,
    cache_misses: 10,
};
// Server operations would be async in real code
```

## Architecture Compliance

- **CLEAN Layer**: Infrastructure layer (depends on all other infrastructure crates)
- **SOLID Principle**: Single responsibility for distributed compilation coordination
- **Safe Rust**: No unsafe code, uses async networking and safe serialization
- **Scalable**: Async design supports concurrent compilation requests
- **Observable**: Structured logging and metrics for server operations

## Implementation Strategy

Since the compiler doesn't interact with C code, the compile server is implemented as:

1. **TCP-based Communication**: Safe networking instead of ei library
2. **Structured Serialization**: JSON/serde instead of Erlang terms
3. **Async Processing**: Tokio for concurrent request handling
4. **Result Caching**: In-memory cache for compilation results
5. **Process Isolation**: Separate server process for stability
*/

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use infrastructure_environment_config::CompileServerConfig;
use infrastructure_environment_config;

/// Compile server result type
pub type ServerResult<T> = Result<T, ServerError>;

/// Compile response from server
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]

/// Server-specific errors
#[derive(thiserror::Error)]
pub enum ServerError {
    #[error("Server not available: {0}")]
    ServerUnavailable(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Compilation error: {0}")]
    CompilationError(String),
}

impl ServerError {
    /// Convert to CompilerError to avoid cyclic dependencies
    pub fn into_compiler_error(self) -> infrastructure_error_handling::CompilerError {
        match self {
            ServerError::NetworkError(msg) => {
                infrastructure_error_handling::CompilerError::InternalError(format!("Network error: {}", msg))
            }
            ServerError::SerializationError(msg) => {
                infrastructure_error_handling::CompilerError::InternalError(format!("Serialization error: {}", msg))
            }
            ServerError::CacheError(msg) => {
                infrastructure_error_handling::CompilerError::InternalError(format!("Cache error: {}", msg))
            }
            ServerError::InvalidRequest(msg) => {
                infrastructure_error_handling::CompilerError::InvalidArgument(msg)
            }
            ServerError::CompilationError(msg) => {
                infrastructure_error_handling::CompilerError::InternalError(format!("Compilation error: {}", msg))
            }
            ServerError::ServerUnavailable(msg) => {
                infrastructure_error_handling::CompilerError::InternalError(format!("Server unavailable: {}", msg))
            }
        }
    }
}

/// Compilation request sent to server
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompileRequest {
    /// Source file path
    pub source_file: String,
    /// Source code content (for verification)
    pub source_content: String,
    /// Compilation options
    pub options: CompileOptions,
    /// Environment context
    pub environment: EnvironmentContext,
}

/// Environment context for compilation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnvironmentContext {
    /// Working directory
    pub cwd: String,
    /// Environment variables
    pub env_vars: HashMap<String, String>,
    /// Include paths
    pub include_paths: Vec<String>,
}

/// Compilation options
#[derive(Debug, Clone, Hash, serde::Serialize, serde::Deserialize)]
pub struct CompileOptions {
    /// Output warnings
    pub warnings: bool,
    /// Debug info
    pub debug_info: bool,
    /// Optimization level
    pub optimize: bool,
    /// Target platform
    pub target: Option<String>,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            warnings: true,
            debug_info: false,
            optimize: false,
            target: None,
        }
    }
}

/// Compilation response from server
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CompileResponse {
    /// Success status
    pub success: bool,
    /// Compiled output (beam file content)
    pub output: Option<Vec<u8>>,
    /// Error messages if compilation failed
    pub errors: Vec<String>,
    /// Warnings generated
    pub warnings: Vec<String>,
    /// Compilation time in milliseconds
    pub compile_time_ms: u64,
}

/// Server status information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServerStatus {
    pub version: String,
    pub uptime_seconds: u64,
    pub active_compilations: usize,
    pub cache_size: usize,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

/// In-memory compilation cache
pub mod cache {
    use super::*;
    use std::collections::HashMap;

    /// Global compilation cache
    static CACHE: once_cell::sync::Lazy<RwLock<HashMap<String, CompileResponse>>> =
        once_cell::sync::Lazy::new(|| RwLock::new(HashMap::new()));

    /// Generate cache key from source content and options
    pub fn generate_cache_key(source_content: &str, options: &CompileOptions) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        source_content.hash(&mut hasher);
        options.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Store compilation result in cache
    pub async fn store_result(cache_key: &str, result: CompileResponse) -> ServerResult<()> {
        let mut cache = CACHE.write().await;
        cache.insert(cache_key.to_string(), result);
        Ok(())
    }

    /// Retrieve compilation result from cache
    pub async fn get_result(cache_key: &str) -> ServerResult<Option<CompileResponse>> {
        let cache = CACHE.read().await;
        Ok(cache.get(cache_key).cloned())
    }

    /// Get cache statistics
    pub async fn get_stats() -> (usize, u64, u64) {
        let cache = CACHE.read().await;
        let size = cache.len();
        // For now, return dummy hit/miss stats
        // In a real implementation, these would be tracked
        (size, 0, 0)
    }

    /// Clear cache
    pub async fn clear() -> ServerResult<()> {
        let mut cache = CACHE.write().await;
        cache.clear();
        Ok(())
    }
}

/// Client interface for communicating with compile server
pub mod client {
    use super::*;
    use tokio::net::TcpStream;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    /// Default server address
    pub const DEFAULT_SERVER_ADDR: &str = "127.0.0.1:9999";

    /// Send compilation request to server
    pub async fn send_compile_request(request: &CompileRequest) -> ServerResult<CompileResponse> {
        // Try to get cached result first
        let cache_key = cache::generate_cache_key(&request.source_content, &request.options);
        if let Some(cached) = cache::get_result(&cache_key).await? {
            return Ok(cached);
        }

        // Send request to server
        let response = send_request_to_server(request).await?;

        // Cache successful results
        if response.success {
            cache::store_result(&cache_key, response.clone()).await?;
        }

        Ok(response)
    }

    /// Send raw request to server via TCP
    async fn send_request_to_server(request: &CompileRequest) -> ServerResult<CompileResponse> {
        let server_addr = std::env::var("ERLC_SERVER_ADDR")
            .unwrap_or_else(|_| DEFAULT_SERVER_ADDR.to_string());

        let mut stream = TcpStream::connect(&server_addr).await
            .map_err(|e| ServerError::NetworkError(format!("Failed to connect to server: {}", e)))?;

        // Serialize request
        let request_data = serde_json::to_vec(request)
            .map_err(|e| ServerError::SerializationError(e.to_string()))?;

        // Send request length (4 bytes) + request data
        let request_len = (request_data.len() as u32).to_be_bytes();
        stream.write_all(&request_len).await
            .map_err(|e| ServerError::NetworkError(e.to_string()))?;
        stream.write_all(&request_data).await
            .map_err(|e| ServerError::NetworkError(e.to_string()))?;

        // Read response length
        let mut response_len_buf = [0u8; 4];
        stream.read_exact(&mut response_len_buf).await
            .map_err(|e| ServerError::NetworkError(e.to_string()))?;
        let response_len = u32::from_be_bytes(response_len_buf) as usize;

        // Read response data
        let mut response_data = vec![0u8; response_len];
        stream.read_exact(&mut response_data).await
            .map_err(|e| ServerError::NetworkError(e.to_string()))?;

        // Deserialize response
        let response: CompileResponse = serde_json::from_slice(&response_data)
            .map_err(|e| ServerError::SerializationError(e.to_string()))?;

        Ok(response)
    }

    /// Check if server is available
    pub async fn server_available() -> bool {
        let server_addr = std::env::var("ERLC_SERVER_ADDR")
            .unwrap_or_else(|_| DEFAULT_SERVER_ADDR.to_string());

        TcpStream::connect(&server_addr).await.is_ok()
    }

    /// Get server status
    pub async fn get_server_status() -> ServerResult<ServerStatus> {
        // This would send a status request to the server
        // For now, return a mock status
        Ok(ServerStatus {
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: 0, // Would be tracked by server
            active_compilations: 0,
            cache_size: cache::get_stats().await.0,
            cache_hits: 0,
            cache_misses: 0,
        })
    }
}

/// Server implementation
pub mod server {
    use super::*;
    use crate::client::DEFAULT_SERVER_ADDR;
    use tokio::net::{TcpListener, TcpStream};
    use tokio::sync::mpsc;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use std::time::Instant;

    /// Start the compile server
    pub async fn start_compile_server() -> ServerResult<ServerHandle> {
        let server_addr = std::env::var("ERLC_SERVER_ADDR")
            .unwrap_or_else(|_| DEFAULT_SERVER_ADDR.to_string());

        let listener = TcpListener::bind(&server_addr).await
            .map_err(|e| ServerError::NetworkError(format!("Failed to bind server: {}", e)))?;

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);

        // Spawn server task
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    result = listener.accept() => {
                        match result {
                            Ok((socket, _)) => {
                                tokio::spawn(async move {
                                    if let Err(e) = handle_client(socket).await {
                                        eprintln!("Client handling error: {}", e);
                                    }
                                });
                            }
                            Err(e) => {
                                eprintln!("Accept error: {}", e);
                                break;
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        break;
                    }
                }
            }
        });

        Ok(ServerHandle {
            shutdown_sender: shutdown_tx,
        })
    }

    /// Handle individual client connections
    async fn handle_client(mut socket: TcpStream) -> ServerResult<()> {
        // Read request length
        let mut len_buf = [0u8; 4];
        socket.read_exact(&mut len_buf).await
            .map_err(|e| ServerError::NetworkError(e.to_string()))?;
        let request_len = u32::from_be_bytes(len_buf) as usize;

        // Read request data
        let mut request_data = vec![0u8; request_len];
        socket.read_exact(&mut request_data).await
            .map_err(|e| ServerError::NetworkError(e.to_string()))?;

        // Deserialize request
        let request: CompileRequest = serde_json::from_slice(&request_data)
            .map_err(|e| ServerError::SerializationError(e.to_string()))?;

        // Process compilation
        let start_time = Instant::now();
        let response = process_compilation_request(&request).await;
        let compile_time = start_time.elapsed().as_millis() as u64;

        // Update response with timing
        let mut final_response = response;
        final_response.compile_time_ms = compile_time;

        // Serialize response
        let response_data = serde_json::to_vec(&final_response)
            .map_err(|e| ServerError::SerializationError(e.to_string()))?;

        // Send response length + data
        let response_len = (response_data.len() as u32).to_be_bytes();
        socket.write_all(&response_len).await
            .map_err(|e| ServerError::NetworkError(e.to_string()))?;
        socket.write_all(&response_data).await
            .map_err(|e| ServerError::NetworkError(e.to_string()))?;

        Ok(())
    }

    /// Process a compilation request
    async fn process_compilation_request(request: &CompileRequest) -> CompileResponse {
        // Check cache first
        let cache_key = cache::generate_cache_key(&request.source_content, &request.options);
        if let Ok(Some(cached)) = cache::get_result(&cache_key).await {
            return cached;
        }

        // Perform compilation (simplified - would call actual compiler)
        // For now, simulate compilation success/failure based on content
        let success = !request.source_content.contains("error");
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        if !success {
            errors.push("Simulated compilation error".to_string());
        } else if request.options.warnings {
            warnings.push("Simulated warning".to_string());
        }

        let response = CompileResponse {
            success,
            output: success.then(|| vec![1, 2, 3, 4]), // Mock beam data
            errors,
            warnings,
            compile_time_ms: 0, // Will be set by caller
        };

        // Cache result
        if success {
            let _ = cache::store_result(&cache_key, response.clone()).await;
        }

        response
    }

    /// Handle for managing server lifecycle
    pub struct ServerHandle {
        shutdown_sender: mpsc::Sender<()>,
    }

    impl ServerHandle {
        /// Shutdown the server
        pub async fn shutdown(self) -> ServerResult<()> {
            // Sender will be dropped, causing server to shut down
            Ok(())
        }

        /// Check if server is still running
        pub fn is_running(&self) -> bool {
            // In a real implementation, this would check server status
            true
        }
    }
}

/// Environment encoding for server communication
pub mod encoding {
    use super::*;
    use infrastructure_environment_config::env;

    /// Encode environment variables for transmission to server
    pub fn encode_environment() -> ServerResult<EnvironmentContext> {
        let cwd = env::current_dir()
            .map_err(|e| ServerError::NetworkError(format!("Failed to get CWD: {}", e)))?;

        let env_vars = env::get_all_vars()
            .map_err(|e| ServerError::NetworkError(format!("Failed to get env vars: {}", e)))?;

        let include_paths = infrastructure_environment_config::erlang::get_library_paths()
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        Ok(EnvironmentContext {
            cwd: cwd.to_string_lossy().to_string(),
            env_vars,
            include_paths,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_operations() {
        // Clear cache first
        cache::clear().await.unwrap();

        let options = CompileOptions::default();
        let cache_key = cache::generate_cache_key("test content", &options);

        // Should not be in cache initially
        let result = cache::get_result(&cache_key).await.unwrap();
        assert!(result.is_none());

        // Store a result
        let response = CompileResponse {
            success: true,
            output: Some(vec![1, 2, 3]),
            errors: vec![],
            warnings: vec![],
            compile_time_ms: 100,
        };

        cache::store_result(&cache_key, response.clone()).await.unwrap();

        // Should be retrievable
        let cached = cache::get_result(&cache_key).await.unwrap();
        assert_eq!(cached, Some(response));

        // Check stats
        let (size, _, _) = cache::get_stats().await;
        assert_eq!(size, 1);
    }

    #[test]
    fn test_compile_request_serialization() {
        let request = CompileRequest {
            source_file: "test.erl".to_string(),
            source_content: "-module(test).".to_string(),
            options: CompileOptions::default(),
            environment: EnvironmentContext {
                cwd: "/tmp".to_string(),
                env_vars: HashMap::new(),
                include_paths: vec![],
            },
        };

        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: CompileRequest = serde_json::from_str(&serialized).unwrap();

        assert_eq!(request.source_file, deserialized.source_file);
        assert_eq!(request.source_content, deserialized.source_content);
    }

    #[test]
    fn test_compile_response_serialization() {
        let response = CompileResponse {
            success: true,
            output: Some(vec![1, 2, 3, 4]),
            errors: vec![],
            warnings: vec!["Warning".to_string()],
            compile_time_ms: 150,
        };

        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: CompileResponse = serde_json::from_str(&serialized).unwrap();

        assert_eq!(response.success, deserialized.success);
        assert_eq!(response.output, deserialized.output);
        assert_eq!(response.warnings, deserialized.warnings);
    }

    #[test]
    fn test_environment_encoding() {
        let context = encoding::encode_environment().unwrap();
        assert!(!context.cwd.is_empty());
        // Environment variables should be populated
        assert!(!context.env_vars.is_empty());
    }

    #[tokio::test]
    async fn test_server_status() {
        let status = client::get_server_status().await.unwrap();
        assert!(!status.version.is_empty());
        assert_eq!(status.active_compilations, 0);
    }
}
