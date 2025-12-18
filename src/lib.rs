#![deny(unsafe_code)]
#![warn(clippy::all, clippy::pedantic, missing_docs)]

//! Core abstractions for building gRPC-Mesh node applications.
//!
//! This library provides the foundational components for connecting to a gRPC-Mesh
//! server and exposing RPC methods for remote invocation. It includes:
//!
//! - **Method Registry**: Thread-safe registry for registering and invoking local RPC handlers
//! - **RPC Primitives**: Request/response types and error handling
//! - **Tunnel Support**: TLS+Yamux tunnel establishment and management
//! - **gRPC Service**: Ready-to-use `InvokePlane` service implementation
//!
//! # Quick Start
//!
//! ```no_run
//! use grpc_mesh_node::{MethodRegistry, RpcResult, tunnel, rpc};
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // 1. Create a method registry
//! let registry = MethodRegistry::default();
//!
//! // 2. Register your methods
//! registry.register("calculator.add", Arc::new(|payload: Vec<u8>| {
//!     // Parse input, do work, return result
//!     Ok(vec![42])
//! }));
//!
//! // 3. Connect to the mesh server
//! let config = tunnel::ConnectorConfig {
//!     server_addr: "mesh-server.example.com:8443".into(),
//!     ..Default::default()
//! };
//! let handshake = tunnel::HandshakeBuilder::new("my-node-id")
//!     .version("1.0.0")
//!     .build()?;
//!
//! let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
//! let mut connector = tunnel::TunnelConnector::new(config, shutdown_rx)?;
//! let incoming = connector.connect_with_backoff(handshake).await?;
//!
//! // 4. Serve RPCs using tonic
//! let service = rpc::InvokeService::new(registry).into_server();
//! tonic::transport::Server::builder()
//!     .add_service(service)
//!     .serve_with_incoming(incoming)
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Architecture
//!
//! The library follows a layered architecture:
//!
//! ```text
//! ┌─────────────────────────────────────┐
//! │   Your Application Logic           │
//! │   (Calculator, User Service, etc.)  │
//! └─────────────────┬───────────────────┘
//!                   │
//! ┌─────────────────▼───────────────────┐
//! │   MethodRegistry                    │
//! │   (Handler Registration & Dispatch) │
//! └─────────────────┬───────────────────┘
//!                   │
//! ┌─────────────────▼───────────────────┐
//! │   InvokePlane gRPC Service          │
//! │   (Tonic Service Implementation)    │
//! └─────────────────┬───────────────────┘
//!                   │
//! ┌─────────────────▼───────────────────┐
//! │   Yamux Multiplexed Streams         │
//! └─────────────────┬───────────────────┘
//!                   │
//! ┌─────────────────▼───────────────────┐
//! │   TLS Connection                    │
//! └─────────────────┬───────────────────┘
//!                   │
//!                   ▼
//!          gRPC-Mesh Server
//! ```

pub mod rpc;
pub mod tunnel;

use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// Convenient type alias for results returned by RPC operations.
///
/// This is used throughout the library to represent operations that may fail
/// with an [`RpcError`].
///
/// # Example
///
/// ```
/// use grpc_mesh_node::{RpcResult, RpcError};
///
/// fn validate_input(data: &[u8]) -> RpcResult<()> {
///     if data.is_empty() {
///         return Err(RpcError::Config("input cannot be empty".into()));
///     }
///     Ok(())
/// }
/// ```
pub type RpcResult<T> = Result<T, RpcError>;

/// Represents errors that may occur during RPC method invocation.
///
/// Each error variant maps to a specific error code that can be used for
/// telemetry, monitoring, and client-side error handling.
///
/// # Example
///
/// ```
/// use grpc_mesh_node::{RpcError, MethodRegistry};
/// use std::sync::Arc;
///
/// let registry = MethodRegistry::default();
/// registry.register("test.method", Arc::new(|_payload| {
///     Err(RpcError::Internal("database connection failed".into()))
/// }));
///
/// let result = registry.invoke("test.method", vec![]);
/// assert!(result.is_err());
/// ```
#[derive(Debug, Clone, thiserror::Error)]
pub enum RpcError {
    /// Configuration or initialization error.
    ///
    /// This variant is used when there are problems with:
    /// - Invalid configuration values
    /// - Missing required parameters
    /// - Setup/initialization failures
    #[error("rpc configuration error: {0}")]
    Config(String),

    /// Method not found in the registry.
    ///
    /// Raised when attempting to invoke a method that has not been registered.
    /// The error message includes the method name that was requested.
    #[error("method not found: {0}")]
    MethodNotFound(String),

    /// Transport-level error.
    ///
    /// This variant wraps errors from the underlying transport layer, such as:
    /// - Network connectivity issues
    /// - Connection timeouts
    /// - Protocol errors
    #[error("transport error: {0}")]
    Transport(String),

    /// Internal processing error.
    ///
    /// Catch-all for errors that occur during method execution, such as:
    /// - Business logic failures
    /// - Database errors
    /// - Serialization/deserialization failures
    #[error("internal error: {0}")]
    Internal(String),
}

impl RpcError {
    /// Returns a stable error code suitable for telemetry and error categorization.
    ///
    /// The error code is a static string that identifies the error variant without
    /// exposing implementation details. These codes are safe to log and use in
    /// metrics systems.
    ///
    /// # Error Codes
    ///
    /// - `CONFIG`: Configuration or setup error
    /// - `NOT_FOUND`: Method not registered
    /// - `TRANSPORT`: Network or transport error
    /// - `INTERNAL`: Internal processing error
    ///
    /// # Example
    ///
    /// ```
    /// use grpc_mesh_node::RpcError;
    ///
    /// let err = RpcError::MethodNotFound("calculator.add".into());
    /// assert_eq!(err.code(), "NOT_FOUND");
    /// ```
    pub fn code(&self) -> &'static str {
        match self {
            Self::Config(_) => "CONFIG",
            Self::MethodNotFound(_) => "NOT_FOUND",
            Self::Transport(_) => "TRANSPORT",
            Self::Internal(_) => "INTERNAL",
        }
    }
}

/// Represents a remote procedure call request.
///
/// An `RpcRequest` encapsulates all the information needed to invoke a method
/// on a remote peer, including the method name, serialized payload, timeout,
/// and correlation ID for distributed tracing.
///
/// # Example
///
/// ```
/// use grpc_mesh_node::RpcRequest;
/// use std::time::Duration;
///
/// let request = RpcRequest {
///     method: "calculator.add".into(),
///     payload: serde_json::to_vec(&serde_json::json!({"a": 5, "b": 3})).unwrap(),
///     timeout: Duration::from_secs(5),
///     correlation_id: "req-12345".into(),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct RpcRequest {
    /// Fully qualified method name to invoke (e.g., "service.Method").
    ///
    /// The method name should follow a hierarchical naming convention for
    /// better organization and routing.
    pub method: String,

    /// Serialized request payload delivered to the remote handler.
    ///
    /// The payload format is opaque to the RPC layer and is typically JSON
    /// or Protocol Buffers serialized data.
    pub payload: Vec<u8>,

    /// Maximum time to wait for the response.
    ///
    /// If the remote method does not respond within this duration, the
    /// invocation is cancelled and an error is returned.
    pub timeout: Duration,

    /// Correlation identifier for distributed tracing and logging.
    ///
    /// This ID should be unique per request and is propagated through the
    /// system to enable request tracking across service boundaries.
    pub correlation_id: String,
}

/// Represents the result of a remote procedure call.
///
/// An `RpcResponse` contains either a successful result or an error, along with
/// timing information for observability.
///
/// # Example
///
/// ```
/// use grpc_mesh_node::{RpcResponse, RpcError};
/// use std::time::Duration;
///
/// // Successful response
/// let success = RpcResponse {
///     success: true,
///     payload: vec![42],
///     error: None,
///     elapsed: Duration::from_millis(150),
/// };
///
/// // Error response
/// let failure = RpcResponse {
///     success: false,
///     payload: vec![],
///     error: Some(RpcError::MethodNotFound("unknown.method".into())),
///     elapsed: Duration::from_millis(5),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct RpcResponse {
    /// Whether the invocation completed successfully.
    ///
    /// If `true`, the `payload` field contains the result.
    /// If `false`, the `error` field contains details about the failure.
    pub success: bool,

    /// Serialized response payload from the handler.
    ///
    /// Only populated when `success` is `true`. The format matches the
    /// method's return type (typically JSON or Protocol Buffers).
    pub payload: Vec<u8>,

    /// Error details if the invocation failed.
    ///
    /// Only populated when `success` is `false`. Contains structured error
    /// information including an error code and message.
    pub error: Option<RpcError>,

    /// Time elapsed from request initiation to response completion.
    ///
    /// This includes network round-trip time and remote processing time,
    /// useful for performance monitoring and SLA tracking.
    pub elapsed: Duration,
}

/// Type alias for RPC method handler functions.
///
/// A method handler is a function that processes a serialized request payload
/// and returns either a serialized response or an error. Handlers must be:
/// - Thread-safe (`Send + Sync`)
/// - Clonable via `Arc`
/// - Pure functions of their input (for testability)
///
/// # Example
///
/// ```
/// use grpc_mesh_node::{MethodHandler, RpcResult};
/// use std::sync::Arc;
///
/// let add_handler: MethodHandler = Arc::new(|payload: Vec<u8>| -> RpcResult<Vec<u8>> {
///     let input: serde_json::Value = serde_json::from_slice(&payload)?;
///     let a = input["a"].as_i64().unwrap_or(0);
///     let b = input["b"].as_i64().unwrap_or(0);
///     let result = serde_json::json!({"result": a + b});
///     Ok(serde_json::to_vec(&result)?)
/// });
/// ```
pub type MethodHandler = Arc<dyn Fn(Vec<u8>) -> RpcResult<Vec<u8>> + Send + Sync>;

/// Thread-safe registry for storing and dispatching RPC method handlers.
///
/// The `MethodRegistry` is the central component for managing method handlers
/// in a mesh node. It provides:
/// - Registration of method handlers by name
/// - Thread-safe concurrent access
/// - Method lookup and invocation
/// - Enumeration of registered methods
///
/// All operations are thread-safe and can be performed concurrently from
/// multiple tasks without external synchronization.
///
/// # Example
///
/// ```
/// use grpc_mesh_node::{MethodRegistry, RpcResult};
/// use std::sync::Arc;
///
/// let registry = MethodRegistry::default();
///
/// // Register handlers
/// registry.register("calculator.add", Arc::new(|payload| {
///     // Implementation...
///     Ok(vec![42])
/// }));
///
/// registry.register("calculator.multiply", Arc::new(|payload| {
///     // Implementation...
///     Ok(vec![84])
/// }));
///
/// // List registered methods
/// let methods = registry.methods();
/// assert_eq!(methods.len(), 2);
///
/// // Invoke a method
/// let result = registry.invoke("calculator.add", vec![1, 2, 3]);
/// ```
#[derive(Default, Clone)]
pub struct MethodRegistry {
    inner: Arc<RwLock<HashMap<String, MethodHandler>>>,
}

impl MethodRegistry {
    /// Registers a method handler in the registry.
    ///
    /// If a handler is already registered for the given method name, it will be
    /// replaced with the new handler. This allows for dynamic handler updates.
    ///
    /// # Parameters
    ///
    /// - `method`: Method name (e.g., "service.Method" or "calculator.add")
    /// - `handler`: Function that processes requests for this method
    ///
    /// # Panics
    ///
    /// Panics if the internal lock is poisoned (which indicates a panic occurred
    /// while holding the lock).
    ///
    /// # Example
    ///
    /// ```
    /// use grpc_mesh_node::{MethodRegistry, RpcResult};
    /// use std::sync::Arc;
    ///
    /// let registry = MethodRegistry::default();
    ///
    /// registry.register("calculator.add", Arc::new(|payload: Vec<u8>| -> RpcResult<Vec<u8>> {
    ///     let input: serde_json::Value = serde_json::from_slice(&payload)?;
    ///     let a = input["a"].as_i64().unwrap_or(0);
    ///     let b = input["b"].as_i64().unwrap_or(0);
    ///     let result = serde_json::json!({"result": a + b});
    ///     Ok(serde_json::to_vec(&result)?)
    /// }));
    /// ```
    pub fn register<S: Into<String>>(&self, method: S, handler: MethodHandler) {
        self.inner
            .write()
            .expect("registry poisoned")
            .insert(method.into(), handler);
    }

    /// Removes a method handler from the registry.
    ///
    /// After calling this method, invocations of the specified method will fail
    /// with a `MethodNotFound` error.
    ///
    /// If no handler is registered for the given method name, this operation is
    /// a no-op.
    ///
    /// # Parameters
    ///
    /// - `method`: Name of the method to unregister
    ///
    /// # Panics
    ///
    /// Panics if the internal lock is poisoned.
    ///
    /// # Example
    ///
    /// ```
    /// use grpc_mesh_node::MethodRegistry;
    /// use std::sync::Arc;
    ///
    /// let registry = MethodRegistry::default();
    /// registry.register("test.method", Arc::new(|_| Ok(vec![])));
    ///
    /// assert_eq!(registry.methods().len(), 1);
    /// registry.unregister("test.method");
    /// assert_eq!(registry.methods().len(), 0);
    /// ```
    pub fn unregister<S: AsRef<str>>(&self, method: S) {
        self.inner
            .write()
            .expect("registry poisoned")
            .remove(method.as_ref());
    }

    /// Invokes a registered method handler with the given payload.
    ///
    /// This method looks up the handler by name and executes it with the provided
    /// payload. The handler's result (success or error) is returned directly.
    ///
    /// # Parameters
    ///
    /// - `method`: Name of the method to invoke
    /// - `payload`: Serialized request payload to pass to the handler
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<u8>)`: Serialized response from the handler
    /// - `Err(RpcError::MethodNotFound)`: If no handler is registered for the method
    /// - `Err(RpcError::*)`: Any error returned by the handler
    ///
    /// # Panics
    ///
    /// Panics if the internal lock is poisoned.
    ///
    /// # Example
    ///
    /// ```
    /// use grpc_mesh_node::{MethodRegistry, RpcError};
    /// use std::sync::Arc;
    ///
    /// let registry = MethodRegistry::default();
    /// registry.register("echo", Arc::new(|payload| Ok(payload)));
    ///
    /// let result = registry.invoke("echo", vec![1, 2, 3]);
    /// assert_eq!(result.unwrap(), vec![1, 2, 3]);
    ///
    /// let not_found = registry.invoke("unknown", vec![]);
    /// assert!(matches!(not_found, Err(RpcError::MethodNotFound(_))));
    /// ```
    pub fn invoke(&self, method: &str, payload: Vec<u8>) -> RpcResult<Vec<u8>> {
        let handler = self
            .inner
            .read()
            .expect("registry poisoned")
            .get(method)
            .cloned()
            .ok_or_else(|| RpcError::MethodNotFound(method.to_owned()))?;

        handler(payload)
    }

    /// Returns a list of all currently registered method names.
    ///
    /// This method is useful for:
    /// - Health checks and diagnostics
    /// - Service discovery and registration
    /// - Debugging and monitoring
    ///
    /// The order of methods in the returned vector is not guaranteed.
    ///
    /// # Returns
    ///
    /// A vector of method names (never `None`, but may be empty).
    ///
    /// # Panics
    ///
    /// Panics if the internal lock is poisoned.
    ///
    /// # Example
    ///
    /// ```
    /// use grpc_mesh_node::MethodRegistry;
    /// use std::sync::Arc;
    ///
    /// let registry = MethodRegistry::default();
    /// registry.register("service.method1", Arc::new(|_| Ok(vec![])));
    /// registry.register("service.method2", Arc::new(|_| Ok(vec![])));
    ///
    /// let methods = registry.methods();
    /// assert_eq!(methods.len(), 2);
    /// assert!(methods.contains(&"service.method1".to_string()));
    /// assert!(methods.contains(&"service.method2".to_string()));
    /// ```
    pub fn methods(&self) -> Vec<String> {
        self.inner
            .read()
            .expect("registry poisoned")
            .keys()
            .cloned()
            .collect()
    }
}

/// Configuration for connecting to the gRPC-Mesh server.
///
/// This configuration is used by the legacy `RpcClient` which has been superseded
/// by the `tunnel` module. For new applications, use `tunnel::ConnectorConfig`
/// instead.
///
/// # Deprecated
///
/// Use `tunnel::ConnectorConfig` for new code, which provides:
/// - TLS support
/// - Automatic reconnection with backoff
/// - Certificate management
/// - Heartbeat automation
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Server address in "host:port" format.
    pub server_addr: String,

    /// Unique identifier for this node.
    pub peer_id: String,

    /// Optional authentication token.
    pub token: Option<String>,

    /// Timeout for establishing the initial connection.
    pub connect_timeout: Duration,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            server_addr: "127.0.0.1:50051".into(),
            peer_id: "local-peer".into(),
            token: None,
            connect_timeout: Duration::from_secs(5),
        }
    }
}

/// Legacy RPC client implementation.
///
/// **Note**: This client is deprecated and should not be used in new code.
/// It provides a simple loopback implementation for testing the registry,
/// but does not include actual network transport.
///
/// For production use, see the `tunnel` module which provides:
/// - Real TLS+Yamux transport
/// - Connection management and automatic reconnection
/// - Heartbeat automation
/// - Integration with tonic gRPC server
///
/// # Deprecated
///
/// Use the `tunnel` module with `InvokeService` for new applications.
pub struct RpcClient {
    cfg: ClientConfig,
    registry: MethodRegistry,
}

impl RpcClient {
    /// Creates a new client with the given configuration and method registry.
    ///
    /// # Deprecated
    ///
    /// This client is for testing only. Use the `tunnel` module for production.
    pub fn new(cfg: ClientConfig, registry: MethodRegistry) -> Self {
        Self { cfg, registry }
    }

    /// Validates configuration (does not establish actual connection).
    ///
    /// # Deprecated
    ///
    /// This method performs minimal validation only. For real connections,
    /// use `tunnel::TunnelConnector::connect_with_backoff()`.
    pub fn connect(&self) -> RpcResult<()> {
        if self.cfg.server_addr.is_empty() {
            return Err(RpcError::Config(
                "server address must not be empty".to_owned(),
            ));
        }

        if self.cfg.peer_id.is_empty() {
            return Err(RpcError::Config("peer_id must not be empty".to_owned()));
        }

        // TODO: establish libp2p stream / gRPC channel.
        Ok(())
    }

    /// Invokes a method locally (loopback only, no network transport).
    ///
    /// This implementation routes calls to the local registry for testing purposes.
    /// It does not perform any network communication.
    ///
    /// # Deprecated
    ///
    /// For real remote invocations, use the `tunnel` module with tonic gRPC.
    pub fn invoke(&self, request: RpcRequest) -> RpcResult<RpcResponse> {
        let started = std::time::Instant::now();
        let result = self.registry.invoke(&request.method, request.payload);

        match result {
            Ok(payload) => Ok(RpcResponse {
                success: true,
                payload,
                error: None,
                elapsed: started.elapsed(),
            }),
            Err(err) => Ok(RpcResponse {
                success: false,
                payload: Vec::new(),
                error: Some(err),
                elapsed: started.elapsed(),
            }),
        }
    }
}

/// Creates a displayable wrapper for a duration.
///
/// This helper formats durations in a human-readable format (e.g., "1.234s")
/// suitable for logging and debugging.
///
/// # Example
///
/// ```
/// use grpc_mesh_node::fmt_duration;
/// use std::time::Duration;
///
/// let d = Duration::from_millis(1234);
/// println!("Request took: {}", fmt_duration(d)); // "Request took: 1.234s"
/// ```
pub fn fmt_duration(d: Duration) -> FmtDuration {
    FmtDuration(d)
}

/// Wrapper struct that implements [`Display`] for [`Duration`].
///
/// Created by [`fmt_duration()`]. Formats durations as "seconds.milliseconds"
/// (e.g., "1.234s" for 1234 milliseconds).
pub struct FmtDuration(Duration);

impl Display for FmtDuration {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let secs = self.0.as_secs();
        let millis = self.0.subsec_millis();
        write!(f, "{}.{:03}s", secs, millis)
    }
}
