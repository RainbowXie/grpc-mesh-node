//! Embeddable node facade: one object owning the complete node lifecycle.
//!
//! [`EmbeddedNode`] composes the same building blocks the standalone
//! `reverse_gateway` binary assembles by hand — configuration parsing,
//! handshake construction, a [`MethodRegistry`], a [`TunnelConnector`], a
//! tonic server and the reconnect supervision loop — behind a small,
//! state-checked API that non-Rust hosts (the C ABI and the language
//! bindings built on it) can drive.
//!
//! Ownership model:
//!
//! - The facade owns a private [`MethodRegistry`]. Handlers may only be
//!   registered or removed while the node is [`NodeLifecycle::Created`];
//!   [`EmbeddedNode::start`] takes a sorted snapshot of the method names and
//!   reports it through the `mesh.methods` handshake metadata key, so the
//!   method list reported to the control plane always matches the set of
//!   handlers actually served.
//! - [`EmbeddedNode::start`] spawns a dedicated supervisor thread with its
//!   own Tokio runtime; the caller's thread and runtime are never occupied
//!   by the node.
//! - [`EmbeddedNode::stop`] is bounded: it signals shutdown, then waits for
//!   the supervisor to exit within the given timeout. A timeout leaves the
//!   node in [`NodeLifecycle::Stopping`] and the call may be retried; it
//!   never force-frees resources that are still in use.
//!
//! # Example
//!
//! ```no_run
//! use std::sync::Arc;
//! use std::time::Duration;
//! use grpc_mesh::embedded::{EmbeddedNode, EmbeddedNodeConfig};
//!
//! let config = EmbeddedNodeConfig::from_json(r#"{
//!     "server": { "address": "127.0.0.1:8443" },
//!     "node":   { "id": "my-node", "token": "secret" }
//! }"#).expect("valid config");
//!
//! let node = EmbeddedNode::new(config);
//! node.register_method("echo", Arc::new(|payload| Ok(payload)))
//!     .expect("register before start");
//! node.start().expect("start");
//! // ... serve until done ...
//! node.stop(Duration::from_secs(5)).expect("stop");
//! ```

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::Deserialize;
use thiserror::Error;
use tokio::sync::watch;

use crate::rpc::InvokeService;
use crate::tunnel::{ConnectorConfig, Handshake, TunnelConnector, TunnelError};
use crate::{MethodHandler, MethodRegistry};

/// Errors surfaced by the [`EmbeddedNode`] facade.
#[derive(Debug, Error)]
pub enum EmbeddedError {
    /// Configuration was missing required fields or contained invalid values.
    #[error("configuration error: {0}")]
    Config(String),

    /// The requested operation does not match the node's current lifecycle state.
    #[error("invalid state: {0}")]
    InvalidState(String),

    /// `unregister` targeted a method that was never registered.
    #[error("method not registered: {0}")]
    MethodNotRegistered(String),

    /// The supervisor thread did not exit within the stop deadline. The stop
    /// may be retried; resources are retained until it succeeds.
    #[error("shutdown timed out after {timeout:?}: in-flight work has not drained; stop() may be retried")]
    ShutdownTimeout {
        /// The deadline that was exceeded.
        timeout: Duration,
    },

    /// The supervisor thread failed after startup.
    #[error("node supervisor failed: {0}")]
    Supervisor(String),
}

/// Lifecycle states of an [`EmbeddedNode`].
///
/// The facade enforces `Created -> Running -> Stopping -> Stopped`; there is
/// no restart — create a new node instead. The C ABI layers a `Freed`
/// terminal state on top of this.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeLifecycle {
    /// Constructed, handlers may still be registered. Not started.
    Created,
    /// Supervisor thread and Tokio runtime are alive.
    Running,
    /// A stop was requested and is still draining.
    Stopping,
    /// Terminal: stopped or never-started-after-failure. No restart allowed.
    Stopped,
}

/// Tunable configuration for an [`EmbeddedNode`].
///
/// Rust hosts construct this directly; JSON parsing (used by the C ABI) goes
/// through [`EmbeddedNodeConfig::from_json`].
#[derive(Debug, Clone)]
pub struct EmbeddedNodeConfig {
    /// Control plane address in `host:port` form.
    pub server_addr: String,
    /// TLS server name override; defaults to the `host` part of `server_addr`.
    pub sni: Option<String>,
    /// PEM-encoded CA bundles used exclusively for server verification.
    /// Empty means the system/webpki root store.
    pub ca_certs: Vec<Vec<u8>>,
    /// Skip TLS verification entirely (development only).
    pub insecure_skip_verify: bool,
    /// Node identity reported in the handshake; must be unique per server.
    pub node_id: String,
    /// Authentication token bound to `node_id` by the server's token table.
    pub token: String,
    /// Node version string reported in the handshake.
    pub version: String,
    /// Feature tags reported in the handshake.
    pub features: Vec<String>,
    /// Extra handshake metadata entries.
    pub metadata: BTreeMap<String, String>,
    /// Budget for one full connect attempt (TCP + TLS + yamux + handshake).
    pub connect_timeout: Duration,
    /// Ceiling for the connector's internal retry backoff.
    pub max_backoff: Duration,
    /// Interval between automatic heartbeat frames once connected.
    pub heartbeat_interval: Duration,
    /// Base delay of the supervision reconnect backoff.
    pub reconnect_base_delay: Duration,
    /// Ceiling of the supervision reconnect backoff.
    pub reconnect_max_delay: Duration,
    /// A connection healthy for at least this long resets the backoff.
    pub healthy_reset_after: Duration,
}

impl EmbeddedNodeConfig {
    /// Default values for every optional field, mirroring the shipped
    /// `reverse_gateway` binary.
    pub const DEFAULT_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(15);
    /// Default reconnect pacing (250ms base, 30s ceiling, 60s reset window).
    pub const DEFAULT_RECONNECT_BASE_DELAY: Duration = Duration::from_millis(250);
    /// See [`Self::DEFAULT_RECONNECT_BASE_DELAY`].
    pub const DEFAULT_RECONNECT_MAX_DELAY: Duration = Duration::from_secs(30);
    /// See [`Self::DEFAULT_RECONNECT_BASE_DELAY`].
    pub const HEALTHY_RESET_AFTER: Duration = Duration::from_secs(60);

    /// Parses the JSON configuration format accepted by the C ABI.
    ///
    /// Required: `server.address`, `node.id`, `node.token`. Optional sections
    /// (`server.tls`, `heartbeat`, `reconnect`, `connect`) fall back to the
    /// same defaults the standalone binary uses.
    pub fn from_json(json: &str) -> Result<Self, EmbeddedError> {
        let file: ConfigFile = serde_json::from_str(json)
            .map_err(|err| EmbeddedError::Config(format!("invalid JSON config: {err}")))?;
        Self::from_file(file)
    }

    fn from_file(file: ConfigFile) -> Result<Self, EmbeddedError> {
        let node = file.node;
        if node.id.trim().is_empty() {
            return Err(EmbeddedError::Config("missing required field node.id".into()));
        }
        if node.token.trim().is_empty() {
            return Err(EmbeddedError::Config("missing required field node.token".into()));
        }
        let server = file.server.ok_or_else(|| {
            EmbeddedError::Config("missing required field server.address".into())
        })?;
        if server.address.trim().is_empty() {
            return Err(EmbeddedError::Config(
                "missing required field server.address".into(),
            ));
        }
        if !server.address.contains(':') {
            return Err(EmbeddedError::Config(format!(
                "server.address {:?} must be in host:port form",
                server.address
            )));
        }

        let tls = server.tls.unwrap_or_default();
        if tls.ca_cert.is_some() && tls.ca_cert_path.is_some() {
            return Err(EmbeddedError::Config(
                "server.tls.ca_cert and server.tls.ca_cert_path are mutually exclusive".into(),
            ));
        }
        let mut ca_certs: Vec<Vec<u8>> = Vec::new();
        if let Some(pem) = tls.ca_cert {
            ca_certs.push(pem.into_bytes());
        }
        if let Some(path) = tls.ca_cert_path {
            let bytes = std::fs::read(&path).map_err(|err| {
                EmbeddedError::Config(format!("cannot read CA bundle {path}: {err}"))
            })?;
            ca_certs.push(bytes);
        }
        // Fail fast on CA material that cannot yield a certificate; the same
        // guard the shipped binary applies before handing PEM to the connector.
        for pem in &ca_certs {
            let mut cursor = std::io::Cursor::new(pem);
            let parsed_any = rustls_pemfile::certs(&mut cursor).any(|cert| cert.is_ok());
            if !parsed_any {
                return Err(EmbeddedError::Config(
                    "invalid CA bundle: no certificate could be parsed from the configured PEM"
                        .into(),
                ));
            }
        }

        Ok(Self {
            server_addr: server.address,
            sni: tls.server_name.filter(|name| !name.trim().is_empty()),
            ca_certs,
            insecure_skip_verify: tls.insecure_skip_verify,
            node_id: node.id.trim().to_owned(),
            token: node.token.trim().to_owned(),
            version: if node.version.trim().is_empty() {
                "0.0.1".to_owned()
            } else {
                node.version.trim().to_owned()
            },
            features: node
                .features
                .into_iter()
                .filter(|f| !f.trim().is_empty())
                .collect(),
            metadata: node.metadata,
            connect_timeout: Duration::from_secs(file.connect.timeout_secs),
            max_backoff: Duration::from_millis(file.reconnect.max_delay_ms),
            heartbeat_interval: Duration::from_secs(file.heartbeat.interval_secs),
            reconnect_base_delay: Duration::from_millis(file.reconnect.base_delay_ms),
            reconnect_max_delay: Duration::from_millis(file.reconnect.max_delay_ms),
            healthy_reset_after: Duration::from_secs(file.reconnect.healthy_reset_secs),
        })
    }

    fn handshake(&self, methods: &[String]) -> Result<Handshake, EmbeddedError> {
        let mut builder = Handshake::builder(self.node_id.clone(), self.version.clone())
            .token(self.token.clone())
            .metadata_entry("platform", std::env::consts::OS)
            .metadata_entry("arch", std::env::consts::ARCH)
            .metadata_entry("mesh.methods", methods.join(","));
        for feature in &self.features {
            builder = builder.add_feature(feature.clone());
        }
        for (key, value) in &self.metadata {
            builder = builder.metadata_entry(key.clone(), value.clone());
        }
        builder.build().map_err(|err| EmbeddedError::Config(err.to_string()))
    }

    fn connector_config(&self) -> ConnectorConfig {
        ConnectorConfig {
            server_addr: self.server_addr.clone(),
            ca_certs: self.ca_certs.clone(),
            sni: self.sni.clone(),
            connect_timeout: self.connect_timeout,
            max_backoff: self.max_backoff,
            heartbeat_interval: self.heartbeat_interval,
            insecure_skip_verify: self.insecure_skip_verify,
        }
    }
}

#[derive(Deserialize)]
struct ConfigFile {
    #[serde(default)]
    server: Option<ServerSection>,
    node: NodeSection,
    #[serde(default)]
    heartbeat: HeartbeatSection,
    #[serde(default)]
    reconnect: ReconnectSection,
    #[serde(default)]
    connect: ConnectSection,
}

#[derive(Deserialize)]
struct ServerSection {
    address: String,
    #[serde(default)]
    tls: Option<TlsSection>,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct TlsSection {
    #[serde(default)]
    server_name: Option<String>,
    #[serde(default)]
    ca_cert: Option<String>,
    #[serde(default)]
    ca_cert_path: Option<String>,
    #[serde(default)]
    insecure_skip_verify: bool,
}

#[derive(Deserialize)]
struct NodeSection {
    #[serde(default)]
    id: String,
    #[serde(default)]
    token: String,
    #[serde(default)]
    version: String,
    #[serde(default)]
    features: Vec<String>,
    #[serde(default)]
    metadata: BTreeMap<String, String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HeartbeatSection {
    #[serde(default = "default_heartbeat_secs")]
    interval_secs: u64,
}

impl Default for HeartbeatSection {
    fn default() -> Self {
        Self {
            interval_secs: default_heartbeat_secs(),
        }
    }
}

fn default_heartbeat_secs() -> u64 {
    15
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReconnectSection {
    #[serde(default = "default_reconnect_base_ms")]
    base_delay_ms: u64,
    #[serde(default = "default_reconnect_max_ms")]
    max_delay_ms: u64,
    #[serde(default = "default_healthy_reset_secs")]
    healthy_reset_secs: u64,
}

impl Default for ReconnectSection {
    fn default() -> Self {
        Self {
            base_delay_ms: default_reconnect_base_ms(),
            max_delay_ms: default_reconnect_max_ms(),
            healthy_reset_secs: default_healthy_reset_secs(),
        }
    }
}

fn default_reconnect_base_ms() -> u64 {
    250
}

fn default_reconnect_max_ms() -> u64 {
    30_000
}

fn default_healthy_reset_secs() -> u64 {
    60
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ConnectSection {
    #[serde(default = "default_connect_timeout_secs")]
    timeout_secs: u64,
}

impl Default for ConnectSection {
    fn default() -> Self {
        Self {
            timeout_secs: default_connect_timeout_secs(),
        }
    }
}

fn default_connect_timeout_secs() -> u64 {
    10
}

struct NodeInner {
    config: EmbeddedNodeConfig,
    registry: MethodRegistry,
    state: Mutex<NodeLifecycle>,
    fatal: Mutex<Option<String>>,
    shutdown_tx: watch::Sender<bool>,
    supervisor: Mutex<Option<std::thread::JoinHandle<()>>>,
}

/// A node instance owning its configuration, methods and supervision thread.
///
/// See the [module documentation](self) for the ownership model.
pub struct EmbeddedNode {
    inner: Arc<NodeInner>,
}

impl EmbeddedNode {
    /// Creates a node in the [`NodeLifecycle::Created`] state.
    ///
    /// This validates nothing beyond what the caller already did: connection
    /// material is checked lazily by [`EmbeddedNode::start`], which reports
    /// configuration problems before spawning the supervisor.
    pub fn new(config: EmbeddedNodeConfig) -> Self {
        let (shutdown_tx, _) = watch::channel(false);
        Self {
            inner: Arc::new(NodeInner {
                config,
                registry: MethodRegistry::default(),
                state: Mutex::new(NodeLifecycle::Created),
                fatal: Mutex::new(None),
                shutdown_tx,
                supervisor: Mutex::new(None),
            }),
        }
    }

    /// The node's current lifecycle state.
    pub fn state(&self) -> NodeLifecycle {
        *self.inner.state.lock().expect("state lock poisoned")
    }

    /// Sorted snapshot of the currently registered method names.
    pub fn methods(&self) -> Vec<String> {
        let mut methods = self.inner.registry.methods();
        methods.sort();
        methods
    }

    /// Registers (or replaces) a handler for `method`.
    ///
    /// Only legal while [`NodeLifecycle::Created`]; once started the method
    /// table is frozen so the reported `mesh.methods` list stays truthful.
    pub fn register_method(
        &self,
        method: impl Into<String>,
        handler: MethodHandler,
    ) -> Result<(), EmbeddedError> {
        let state = self.inner.state.lock().expect("state lock poisoned");
        match *state {
            NodeLifecycle::Created => {
                self.inner.registry.register(method, handler);
                Ok(())
            }
            other => Err(EmbeddedError::InvalidState(format!(
                "method registration is only allowed before start (current state: {other:?})"
            ))),
        }
    }

    /// Registers (or replaces) a handler that receives the full
    /// [`crate::RpcRequest`] (method name, payload, timeout, correlation id).
    ///
    /// Subject to the same `Created`-only freeze as
    /// [`EmbeddedNode::register_method`]. This is the registration path the
    /// C ABI uses, since host callbacks need the request metadata.
    pub fn register_method_with_request(
        &self,
        method: impl Into<String>,
        handler: crate::MethodHandlerWithRequest,
    ) -> Result<(), EmbeddedError> {
        let state = self.inner.state.lock().expect("state lock poisoned");
        match *state {
            NodeLifecycle::Created => {
                self.inner.registry.register_with_request(method, handler);
                Ok(())
            }
            other => Err(EmbeddedError::InvalidState(format!(
                "method registration is only allowed before start (current state: {other:?})"
            ))),
        }
    }

    /// Removes the handler for `method`.
    ///
    /// Only legal while [`NodeLifecycle::Created`]. Fails with
    /// [`EmbeddedError::MethodNotRegistered`] when nothing is registered
    /// under that name.
    pub fn unregister_method(&self, method: impl AsRef<str>) -> Result<(), EmbeddedError> {
        let state = self.inner.state.lock().expect("state lock poisoned");
        match *state {
            NodeLifecycle::Created => {
                let name = method.as_ref();
                if self
                    .inner
                    .registry
                    .methods()
                    .iter()
                    .any(|registered| registered == name)
                {
                    self.inner.registry.unregister(name);
                    Ok(())
                } else {
                    Err(EmbeddedError::MethodNotRegistered(name.to_owned()))
                }
            }
            other => Err(EmbeddedError::InvalidState(format!(
                "method removal is only allowed before start (current state: {other:?})"
            ))),
        }
    }

    /// Starts the node: validates configuration, snapshots the method table,
    /// spawns the supervisor thread with its own Tokio runtime and returns
    /// once the runtime is alive.
    ///
    /// A configuration failure (unparseable CA, invalid identity, unusable
    /// TLS material) is reported synchronously and moves the node to
    /// [`NodeLifecycle::Stopped`] — it is terminal, create a new node.
    pub fn start(&self) -> Result<(), EmbeddedError> {
        let mut state = self.inner.state.lock().expect("state lock poisoned");
        match *state {
            NodeLifecycle::Created => {}
            NodeLifecycle::Running | NodeLifecycle::Stopping => {
                return Err(EmbeddedError::InvalidState(format!(
                    "start called on an already started node (current state: {state:?})"
                )));
            }
            NodeLifecycle::Stopped => {
                return Err(EmbeddedError::InvalidState(
                    "node has been stopped; create a new node instead of restarting".into(),
                ));
            }
        }

        // Everything cheap-but-fallible happens before the thread is spawned,
        // so a bad config never leaves a half-started node behind.
        let methods = self.methods();
        let handshake = self.inner.config.handshake(&methods)?;
        let connector = match TunnelConnector::new(
            self.inner.config.connector_config(),
            self.inner.shutdown_tx.subscribe(),
        ) {
            Ok(connector) => connector,
            Err(TunnelError::Config(msg)) => {
                *state = NodeLifecycle::Stopped;
                return Err(EmbeddedError::Config(msg));
            }
            Err(err) => {
                *state = NodeLifecycle::Stopped;
                return Err(EmbeddedError::Config(err.to_string()));
            }
        };

        let (ack_tx, ack_rx) = std::sync::mpsc::channel::<Result<(), String>>();
        let inner = Arc::clone(&self.inner);
        let handle = std::thread::Builder::new()
            .name("mesh-node-supervisor".to_owned())
            .spawn(move || {
                let runtime = match tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(2)
                    .enable_all()
                    .build()
                {
                    Ok(runtime) => runtime,
                    Err(err) => {
                        let _ = ack_tx.send(Err(format!("runtime build failed: {err}")));
                        record_fatal(&inner, format!("runtime build failed: {err}"));
                        return;
                    }
                };
                // The supervisor future is driven to completion on this
                // thread; ack unblocks start() the moment the runtime exists.
                let _ = ack_tx.send(Ok(()));
                let outcome = runtime.block_on(supervise(connector, handshake, &inner));
                runtime.shutdown_timeout(Duration::from_secs(5));
                if let Err(err) = outcome {
                    record_fatal(&inner, err.to_string());
                }
            })
            .map_err(|err| {
                *state = NodeLifecycle::Stopped;
                EmbeddedError::Config(format!("failed to spawn supervisor thread: {err}"))
            })?;

        match ack_rx.recv_timeout(Duration::from_secs(30)) {
            Ok(Ok(())) => {
                *self
                    .inner
                    .supervisor
                    .lock()
                    .expect("supervisor lock poisoned") = Some(handle);
                *state = NodeLifecycle::Running;
                Ok(())
            }
            Ok(Err(msg)) => {
                // Runtime construction failed on the supervisor thread.
                let _ = handle.join();
                *state = NodeLifecycle::Stopped;
                Err(EmbeddedError::Config(msg))
            }
            Err(_) => {
                // Thread died before acking (panic) or is wedged; join tells us which.
                let _ = handle.join();
                *state = NodeLifecycle::Stopped;
                Err(EmbeddedError::Config(
                    "supervisor thread exited before the runtime was ready".into(),
                ))
            }
        }
    }

    /// Stops the node within `timeout`.
    ///
    /// Idempotent: stopping a `Created` or already-`Stopped` node is a
    /// no-op. On timeout the node stays [`NodeLifecycle::Stopping`] and
    /// [`EmbeddedError::ShutdownTimeout`] is returned; retrying with a
    /// longer budget is always safe.
    pub fn stop(&self, timeout: Duration) -> Result<(), EmbeddedError> {
        {
            let mut state = self.inner.state.lock().expect("state lock poisoned");
            match *state {
                NodeLifecycle::Created => {
                    *state = NodeLifecycle::Stopped;
                    return Ok(());
                }
                NodeLifecycle::Stopped => return Ok(()),
                NodeLifecycle::Running | NodeLifecycle::Stopping => {
                    *state = NodeLifecycle::Stopping;
                }
            }
        }

        let _ = self.inner.shutdown_tx.send(true);

        let deadline = Instant::now() + timeout;
        loop {
            let finished = {
                let guard = self.inner.supervisor.lock().expect("supervisor lock poisoned");
                guard.as_ref().is_none_or(|handle| handle.is_finished())
            };
            if finished {
                let handle = self
                    .inner
                    .supervisor
                    .lock()
                    .expect("supervisor lock poisoned")
                    .take();
                if let Some(handle) = handle {
                    if let Err(panic) = handle.join() {
                        let msg = panic_message(&panic);
                        *self.inner.state.lock().expect("state lock poisoned") =
                            NodeLifecycle::Stopped;
                        return Err(EmbeddedError::Supervisor(format!(
                            "supervisor thread panicked: {msg}"
                        )));
                    }
                }
                *self.inner.state.lock().expect("state lock poisoned") = NodeLifecycle::Stopped;
                return Ok(());
            }
            if Instant::now() >= deadline {
                return Err(EmbeddedError::ShutdownTimeout { timeout });
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    /// The fatal supervision error, if the supervisor died on its own.
    pub fn fatal_error(&self) -> Option<String> {
        self.inner.fatal.lock().expect("fatal lock poisoned").clone()
    }

    /// Whether the supervisor thread has exited (diagnostics and tests).
    pub fn supervisor_finished(&self) -> bool {
        let guard = self.inner.supervisor.lock().expect("supervisor lock poisoned");
        guard.as_ref().is_none_or(|handle| handle.is_finished())
    }
}

impl Drop for EmbeddedNode {
    fn drop(&mut self) {
        if let Err(EmbeddedError::ShutdownTimeout { timeout }) = self.stop(Duration::from_secs(5)) {
            tracing::warn!(
                timeout = ?timeout,
                "EmbeddedNode dropped while the supervisor had not drained; resources retained"
            );
        }
    }
}

fn record_fatal(inner: &Arc<NodeInner>, message: String) {
    tracing::error!(error = %message, "embedded node supervisor exited with failure");
    *inner.fatal.lock().expect("fatal lock poisoned") = Some(message);
    *inner.state.lock().expect("state lock poisoned") = NodeLifecycle::Stopped;
}

fn panic_message(panic: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = panic.downcast_ref::<&str>() {
        (*s).to_owned()
    } else if let Some(s) = panic.downcast_ref::<String>() {
        s.clone()
    } else {
        "<non-string panic payload>".to_owned()
    }
}

/// The supervision loop, a faithful port of the standalone gateway binary:
/// connect with backoff, serve generic Invoke over the tunnel, reconnect
/// when the tunnel drops or the heartbeat stalls, escalate the reconnect
/// delay for short-lived connections.
async fn supervise(
    mut connector: TunnelConnector,
    handshake: Handshake,
    inner: &Arc<NodeInner>,
) -> Result<(), TunnelError> {
    let shutdown_rx = inner.shutdown_tx.subscribe();
    let (tunnel_dead_tx, tunnel_dead_rx) = watch::channel(false);
    let mut reconnect_attempt: u32 = 0;

    loop {
        if *shutdown_rx.borrow() {
            return Ok(());
        }
        tunnel_dead_tx.send_replace(false);
        let connected_at = Instant::now();

        let incoming = match connector
            .connect_with_backoff_watched(handshake.clone(), tunnel_dead_tx.clone())
            .await
        {
            Ok(incoming) => incoming,
            Err(TunnelError::Shutdown) => return Ok(()),
            Err(err) => return Err(err),
        };
        tracing::info!(
            node_id = %handshake.node_id,
            "embedded node tunnel established with automatic heartbeat"
        );

        let grpc = tonic::transport::Server::builder()
            .add_service(InvokeService::new(inner.registry.clone()).into_server())
            .serve_with_incoming_shutdown(incoming, shutdown_changed(shutdown_rx.clone()));

        tokio::select! {
            result = grpc => match result {
                Ok(()) => tracing::warn!("embedded node tunnel ended; reconnecting"),
                Err(err) => tracing::warn!(%err, "embedded node gRPC serving failed; reconnecting"),
            },
            _ = wait_tunnel_dead(tunnel_dead_rx.clone()) => {
                tracing::warn!("embedded node heartbeat writes stalled; tunnel presumed dead, reconnecting");
            }
        }

        // Connections that die quickly escalate backoff; a healthy stretch
        // resets it so transient failures recover at base speed.
        reconnect_attempt = if connected_at.elapsed() >= inner.config.healthy_reset_after {
            0
        } else {
            reconnect_attempt.saturating_add(1)
        };
        let delay = reconnect_delay(
            reconnect_attempt,
            inner.config.reconnect_base_delay,
            inner.config.reconnect_max_delay,
        );
        tracing::debug!(?delay, attempt = reconnect_attempt, "waiting before reconnecting");
        tokio::select! {
            _ = tokio::time::sleep(delay) => {}
            _ = shutdown_changed(shutdown_rx.clone()) => return Ok(()),
        }
    }
}

async fn shutdown_changed(mut rx: watch::Receiver<bool>) {
    while rx.changed().await.is_ok() {
        if *rx.borrow() {
            break;
        }
    }
}

/// Resolves once the heartbeat task flags the tunnel as dead.
async fn wait_tunnel_dead(mut rx: watch::Receiver<bool>) {
    while rx.changed().await.is_ok() {
        if *rx.borrow() {
            break;
        }
    }
}

/// Exponential reconnect pacing bounded by the configured ceiling.
fn reconnect_delay(attempt: u32, base: Duration, max: Duration) -> Duration {
    let capped = attempt.min(7);
    let millis = (base.as_millis() as u64).saturating_mul(1 << capped);
    Duration::from_millis(millis).min(max)
}
