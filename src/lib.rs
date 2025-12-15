#![deny(unsafe_code)]
#![warn(clippy::all, clippy::pedantic, missing_docs)]

//! Minimal RPC skeleton shared by `grpc-mesh-node` binaries.
//!
//! This module does **not** contain a transport implementation yet; it only
//! offers core abstractions—client configuration, request/response shapes,
//! handler registry, and an in-memory dispatcher that higher layers can build on
//! top of (libp2p, gRPC, etc.).

pub mod rpc;
pub mod tunnel;

use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// Convenient alias for results returned by this module.
pub type RpcResult<T> = Result<T, RpcError>;

/// Represents any failure that may occur while invoking remote methods.
#[derive(Debug, Clone, thiserror::Error)]
pub enum RpcError {
    /// Generic configuration or initialization issue.
    #[error("rpc configuration error: {0}")]
    Config(String),

    /// Raised when the requested method is unknown.
    #[error("method not found: {0}")]
    MethodNotFound(String),

    /// Wraps transport-level failures.
    #[error("transport error: {0}")]
    Transport(String),

    /// Catch-all internal error.
    #[error("internal error: {0}")]
    Internal(String),
}

impl RpcError {
    /// Returns a stable error code that callers can use for telemetry or mapping.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Config(_) => "CONFIG",
            Self::MethodNotFound(_) => "NOT_FOUND",
            Self::Transport(_) => "TRANSPORT",
            Self::Internal(_) => "INTERNAL",
        }
    }
}

/// Describes a unit of work sent to a remote peer.
#[derive(Debug, Clone)]
pub struct RpcRequest {
    /// Fully qualified method name that the remote peer should execute.
    pub method: String,
    /// Serialized request body delivered to the remote handler.
    pub payload: Vec<u8>,
    /// Maximum duration the caller is willing to wait for the response.
    pub timeout: Duration,
    /// Identifier used to correlate server-side logs and traces.
    pub correlation_id: String,
}

/// Represents the output of a remote invocation.
#[derive(Debug, Clone)]
pub struct RpcResponse {
    /// Indicates whether the remote execution completed without error.
    pub success: bool,
    /// Serialized result payload returned by the handler.
    pub payload: Vec<u8>,
    /// Optional structured error describing the failure cause.
    pub error: Option<RpcError>,
    /// Time spent processing the request, as observed by the caller.
    pub elapsed: Duration,
}

/// Function signature every method handler must satisfy.
///
/// Implementations receive the serialized request payload and must return either
/// a serialized response body or an [`RpcError`].
pub type MethodHandler = Arc<dyn Fn(Vec<u8>) -> RpcResult<Vec<u8>> + Send + Sync>;

/// Thread-safe registry storing all exported RPC methods.
#[derive(Default, Clone)]
pub struct MethodRegistry {
    inner: Arc<RwLock<HashMap<String, MethodHandler>>>,
}

impl MethodRegistry {
    /// Registers (or replaces) a method handler.
    pub fn register<S: Into<String>>(&self, method: S, handler: MethodHandler) {
        self.inner
            .write()
            .expect("registry poisoned")
            .insert(method.into(), handler);
    }

    /// Removes the handler for the given method name.
    pub fn unregister<S: AsRef<str>>(&self, method: S) {
        self.inner
            .write()
            .expect("registry poisoned")
            .remove(method.as_ref());
    }

    /// Executes the handler for the specified method, if present.
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

    /// Returns the currently registered method names.
    pub fn methods(&self) -> Vec<String> {
        self.inner
            .read()
            .expect("registry poisoned")
            .keys()
            .cloned()
            .collect()
    }
}

/// Configuration required to connect to the wa-emu server.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Address of the grpc-mesh-server control plane (host:port or multiaddr string).
    pub server_addr: String,
    /// Logical identifier advertised to the control plane for this client.
    pub peer_id: String,
    /// Optional bearer token or credential for authenticating to the server.
    pub token: Option<String>,
    /// Maximum duration allowed when establishing the initial connection.
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

/// Barebone RPC client stub. The transport is intentionally abstract so that
/// the main binary can plug in libp2p, WebRTC, or gRPC without touching the
/// registry logic.
pub struct RpcClient {
    cfg: ClientConfig,
    registry: MethodRegistry,
}

impl RpcClient {
    /// Creates a new client with the given configuration and method registry.
    pub fn new(cfg: ClientConfig, registry: MethodRegistry) -> Self {
        Self { cfg, registry }
    }

    /// Performs any required handshake with the control plane.
    ///
    /// Currently just validates configuration. Replace with actual transport
    /// initialization as the project evolves.
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

    /// Sends an invocation request to the remote grpc-mesh-server.
    ///
    /// The current implementation simply routes the call to the local registry,
    /// acting as a loopback dispatcher. This keeps the API stable while the
    /// transport integration is still in progress.
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

/// Helper to format durations in logs/debug output.
pub fn fmt_duration(d: Duration) -> FmtDuration {
    FmtDuration(d)
}

/// Wrapper struct implementing [`Display`] for [`Duration`].
pub struct FmtDuration(Duration);

impl Display for FmtDuration {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let secs = self.0.as_secs();
        let millis = self.0.subsec_millis();
        write!(f, "{}.{:03}s", secs, millis)
    }
}
