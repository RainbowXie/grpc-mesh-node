use std::cmp::min;
use std::io::Cursor;
use std::sync::Arc;
use std::time::Duration;

use futures::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::watch;
use tokio::time::{interval, sleep, timeout};
use tokio_rustls::TlsConnector;
use tokio_rustls::rustls::{ClientConfig, RootCertStore, pki_types::ServerName};
use tokio_util::compat::{FuturesAsyncWriteCompatExt, TokioAsyncReadCompatExt};
use webpki_roots::TLS_SERVER_ROOTS;
use yamux::{Config as YamuxConfig, Connection, Mode, Stream};

use super::error::{Result, TunnelError};
use super::handshake::{Handshake, send_handshake};
use super::incoming::YamuxIncoming;

/// Configuration for establishing TLS+Yamux tunnel connections to the gRPC-Mesh server.
///
/// This configuration controls all aspects of the tunnel connection including TLS
/// settings, timeouts, retry behavior, and heartbeat intervals.
///
/// # Example
///
/// ```no_run
/// use grpc_mesh_node::tunnel::ConnectorConfig;
/// use std::time::Duration;
///
/// let config = ConnectorConfig {
///     server_addr: "mesh-server.example.com:8443".into(),
///     ca_certs: vec![std::fs::read("ca.crt").unwrap()],
///     sni: Some("mesh-server.example.com".into()),
///     connect_timeout: Duration::from_secs(10),
///     max_backoff: Duration::from_secs(30),
///     heartbeat_interval: Duration::from_secs(15),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct ConnectorConfig {
    /// TCP endpoint of the gRPC-Mesh server control plane.
    ///
    /// Format: "host:port" (e.g., "mesh-server.example.com:8443" or "127.0.0.1:8443")
    pub server_addr: String,

    /// Optional PEM-encoded CA certificates for TLS verification.
    ///
    /// When provided, these certificates are used exclusively for verifying the server's
    /// certificate (system root certificates are NOT included). This is essential for
    /// self-signed certificates in development or private CA deployments.
    ///
    /// If empty, the system's default root certificate store is used.
    pub ca_certs: Vec<Vec<u8>>,

    /// Override for TLS Server Name Indication (SNI).
    ///
    /// If not specified, the hostname part of `server_addr` is used as the SNI value.
    /// Set this when the server address is an IP but you need to verify against a
    /// specific hostname in the certificate.
    pub sni: Option<String>,

    /// Maximum time to wait for each connection attempt.
    ///
    /// This timeout applies to:
    /// - TCP connection establishment
    /// - TLS handshake
    /// - Yamux session setup
    /// - Initial handshake frame transmission
    ///
    /// Default: 10 seconds
    pub connect_timeout: Duration,

    /// Maximum delay between retry attempts.
    ///
    /// When connection attempts fail, the connector uses exponential backoff with
    /// this value as the upper bound. The actual delay starts at 250ms and doubles
    /// with each attempt up to this maximum.
    ///
    /// Default: 30 seconds
    pub max_backoff: Duration,

    /// Interval between automatic heartbeat messages.
    ///
    /// Heartbeats are sent automatically in the background after a successful
    /// connection. They keep the connection alive and allow the server to detect
    /// disconnected nodes.
    ///
    /// Default: 15 seconds
    pub heartbeat_interval: Duration,
}

impl Default for ConnectorConfig {
    fn default() -> Self {
        Self {
            server_addr: "127.0.0.1:8443".into(),
            ca_certs: Vec::new(),
            sni: None,
            connect_timeout: Duration::from_secs(10),
            max_backoff: Duration::from_secs(30),
            heartbeat_interval: Duration::from_secs(15),
        }
    }
}

/// An established TLS+Yamux tunnel to the gRPC-Mesh server.
///
/// A `Tunnel` represents a fully established and authenticated connection that includes:
/// - A negotiated handshake with the server
/// - A Yamux session for multiplexed streams
/// - A dedicated control stream for heartbeats and control messages
///
/// The tunnel is typically consumed by calling `start_heartbeat()` which spawns
/// automatic heartbeat sending and returns the `YamuxIncoming` adapter for serving
/// gRPC requests.
///
/// # Lifecycle
///
/// ```text
/// TunnelConnector::connect_with_backoff()
///     ↓
/// Tunnel (with handshake + incoming + control_stream)
///     ↓
/// tunnel.start_heartbeat()
///     ↓
/// YamuxIncoming (for tonic Server)
///     ↓
/// tonic::transport::Server::builder()
///     .serve_with_incoming(incoming)
/// ```
pub struct Tunnel {
    handshake: Arc<Handshake>,
    incoming: Option<YamuxIncoming>,
    control_stream: Stream,
}

impl Tunnel {
    /// Returns the handshake metadata negotiated with the server.
    ///
    /// The handshake contains node identity, version, features, and metadata
    /// that was agreed upon during connection establishment.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use grpc_mesh_node::tunnel::*;
    /// # async fn example(tunnel: &Tunnel) {
    /// let handshake = tunnel.handshake();
    /// println!("Connected as node: {}", handshake.node_id);
    /// println!("Protocol version: {}", handshake.version);
    /// # }
    /// ```
    pub fn handshake(&self) -> &Handshake {
        &self.handshake
    }

    /// Provides mutable access to the control stream.
    ///
    /// The control stream is used for sending control-plane messages such as:
    /// - Heartbeats (sent automatically by `start_heartbeat()`)
    /// - Configuration updates
    /// - Control commands from the server
    ///
    /// # Note
    ///
    /// Most applications should not need to access this directly, as heartbeats
    /// are managed automatically. This is primarily for advanced use cases.
    pub fn control_stream(&mut self) -> &mut Stream {
        &mut self.control_stream
    }

    /// Transfers ownership of the Yamux incoming adapter to the caller.
    ///
    /// This method can only be called once successfully. Subsequent calls return `None`.
    ///
    /// The incoming adapter is typically passed to `tonic::transport::Server::serve_with_incoming()`
    /// to serve gRPC requests over the tunnel.
    ///
    /// # Returns
    ///
    /// - `Some(YamuxIncoming)` on the first call
    /// - `None` if already taken
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use grpc_mesh_node::tunnel::*;
    /// # async fn example(mut tunnel: Tunnel) -> Result<(), Box<dyn std::error::Error>> {
    /// let incoming = tunnel.take_incoming().expect("incoming already taken");
    /// // Use with tonic server
    /// # Ok(())
    /// # }
    /// ```
    pub fn take_incoming(&mut self) -> Option<YamuxIncoming> {
        self.incoming.take()
    }

    /// Consumes the tunnel and returns all its components.
    ///
    /// This method is useful when you need full control over the tunnel's components
    /// rather than using the automatic heartbeat mechanism.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// 1. The negotiated handshake (wrapped in Arc)
    /// 2. The Yamux incoming adapter for serving requests
    /// 3. The control stream for sending control messages
    ///
    /// # Panics
    ///
    /// Panics if the incoming adapter has already been taken via `take_incoming()`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use grpc_mesh_node::tunnel::*;
    /// # async fn example(tunnel: Tunnel) {
    /// let (handshake, incoming, mut control_stream) = tunnel.into_parts();
    /// println!("Node ID: {}", handshake.node_id);
    /// // Manually manage control_stream and incoming...
    /// # }
    /// ```
    pub fn into_parts(self) -> (Arc<Handshake>, YamuxIncoming, Stream) {
        let Tunnel {
            handshake,
            incoming,
            control_stream,
        } = self;
        (
            Arc::clone(&handshake),
            incoming.expect("incoming adapter already taken"),
            control_stream,
        )
    }

    /// Starts automatic heartbeat sending in the background.
    ///
    /// This method spawns a background task that sends heartbeat messages at the configured
    /// interval. The heartbeat loop continues until the shutdown signal is received.
    ///
    /// # Parameters
    ///
    /// - `shutdown_rx`: Watch channel receiver for shutdown coordination
    /// - `heartbeat_interval`: Time between heartbeat messages
    ///
    /// # Returns
    ///
    /// The Yamux incoming adapter ready for use with tonic's server
    ///
    /// # Errors
    ///
    /// Returns an error if the incoming adapter has already been taken.
    ///
    /// # Note
    ///
    /// This is called automatically by `TunnelConnector::connect_with_backoff()`.
    /// Most applications don't need to call this directly.
    fn start_heartbeat(
        mut self,
        shutdown_rx: watch::Receiver<bool>,
        heartbeat_interval: Duration,
    ) -> Result<YamuxIncoming> {
        let node_id = self.handshake.node_id.clone();
        let mut control_stream = self.control_stream;
        let incoming = self
            .incoming
            .take()
            .ok_or(TunnelError::Protocol("incoming already taken".into()))?;

        tokio::spawn(async move {
            let mut heartbeat_ticker = interval(heartbeat_interval);
            let mut sequence = 0u64;

            loop {
                heartbeat_ticker.tick().await;

                // Check shutdown signal
                if *shutdown_rx.borrow() {
                    tracing::info!("💓 Heartbeat loop shutting down");
                    break;
                }

                // Build heartbeat message
                let heartbeat_msg = serde_json::json!({
                    "type": "heartbeat",
                    "heartbeat": {
                        "node_id": node_id,
                        "timestamp_unix_sec": std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        "sequence": sequence,
                    }
                });

                sequence += 1;

                // Serialize to JSON
                let heartbeat_json = match serde_json::to_vec(&heartbeat_msg) {
                    Ok(json) => json,
                    Err(e) => {
                        tracing::warn!("💔 Failed to serialize heartbeat: {}", e);
                        continue;
                    }
                };

                // Build frame: 4-byte big-endian length prefix + JSON payload
                let payload_len = heartbeat_json.len() as u32;
                let mut frame = Vec::with_capacity(4 + heartbeat_json.len());
                frame.extend_from_slice(&payload_len.to_be_bytes());
                frame.extend_from_slice(&heartbeat_json);

                // Send heartbeat frame
                if let Err(e) = control_stream.write_all(&frame).await {
                    tracing::warn!("💔 Failed to send heartbeat: {}", e);
                } else {
                    if let Err(e) = control_stream.flush().await {
                        tracing::warn!("💔 Failed to flush heartbeat: {}", e);
                    } else {
                        tracing::debug!("💓 Heartbeat sent (seq={})", sequence - 1);
                    }
                }
            }
        });

        Ok(incoming)
    }
}

/// Connection manager that establishes and maintains tunnels to the gRPC-Mesh server.
///
/// The `TunnelConnector` handles the complete connection lifecycle:
/// - TLS connection establishment
/// - Yamux session setup
/// - Handshake negotiation
/// - Automatic retry with exponential backoff
/// - Heartbeat automation
///
/// # Architecture
///
/// ```text
/// TunnelConnector
///     ↓
/// [TCP Connect] → [TLS Handshake] → [Yamux Session] → [Control Stream]
///     ↓                                                      ↓
/// [Send Handshake] ← ← ← ← ← ← ← ← ← ← ← ← ← ← ← ← ← ← ← ┘
///     ↓
/// [Spawn Heartbeat Task] + [Return YamuxIncoming]
/// ```
///
/// # Example
///
/// ```no_run
/// use grpc_mesh_node::tunnel::{ConnectorConfig, TunnelConnector, HandshakeBuilder};
/// use tokio::sync::watch;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = ConnectorConfig::default();
/// let handshake = HandshakeBuilder::new("my-node-id").build()?;
/// let (shutdown_tx, shutdown_rx) = watch::channel(false);
///
/// let mut connector = TunnelConnector::new(config, shutdown_rx)?;
/// let incoming = connector.connect_with_backoff(handshake).await?;
///
/// // Use incoming with tonic server...
/// # Ok(())
/// # }
/// ```
pub struct TunnelConnector {
    cfg: ConnectorConfig,
    tls: Arc<ClientConfig>,
    yamux_cfg: YamuxConfig,
    shutdown_rx: watch::Receiver<bool>,
}

impl TunnelConnector {
    /// Creates a new tunnel connector with the provided configuration.
    ///
    /// This method validates the configuration and builds the TLS client configuration.
    /// It does not establish a connection; call `connect_with_backoff()` to connect.
    ///
    /// # Parameters
    ///
    /// - `cfg`: Connection configuration (server address, TLS settings, timeouts)
    /// - `shutdown_rx`: Watch channel receiver for coordinated shutdown
    ///
    /// # Returns
    ///
    /// - `Ok(TunnelConnector)`: Ready to connect
    /// - `Err(TunnelError)`: If TLS configuration is invalid or CA certificates cannot be loaded
    ///
    /// # Example
    ///
    /// ```no_run
    /// use grpc_mesh_node::tunnel::{ConnectorConfig, TunnelConnector};
    /// use tokio::sync::watch;
    ///
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = ConnectorConfig {
    ///     server_addr: "localhost:8443".into(),
    ///     ca_certs: vec![std::fs::read("ca.crt")?],
    ///     ..Default::default()
    /// };
    /// let (_shutdown_tx, shutdown_rx) = watch::channel(false);
    /// let connector = TunnelConnector::new(config, shutdown_rx)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(cfg: ConnectorConfig, shutdown_rx: watch::Receiver<bool>) -> Result<Self> {
        let tls = Arc::new(build_tls_config(&cfg)?);

        // Configure yamux
        let yamux_cfg = YamuxConfig::default();

        tracing::debug!("Yamux config created with defaults");

        Ok(Self {
            cfg,
            tls,
            yamux_cfg,
            shutdown_rx,
        })
    }

    /// Establishes a tunnel connection with automatic retry and backoff.
    ///
    /// This method attempts to connect to the gRPC-Mesh server, automatically retrying
    /// on transient failures with exponential backoff. Once connected, it starts the
    /// automatic heartbeat loop and returns the incoming adapter for serving requests.
    ///
    /// # Retry Behavior
    ///
    /// - Retries indefinitely on transient errors (network, timeout)
    /// - Uses exponential backoff starting at 250ms, doubling each attempt
    /// - Caps delay at `ConnectorConfig.max_backoff`
    /// - Stops immediately on non-retryable errors (handshake validation, shutdown)
    ///
    /// # Parameters
    ///
    /// - `handshake`: Node identity and metadata to send during handshake
    ///
    /// # Returns
    ///
    /// - `Ok(YamuxIncoming)`: Connected and ready to serve, with heartbeats running
    /// - `Err(TunnelError::Shutdown)`: Shutdown signal received
    /// - `Err(TunnelError::*)`: Non-retryable error occurred
    ///
    /// # Example
    ///
    /// ```no_run
    /// use grpc_mesh_node::tunnel::{ConnectorConfig, TunnelConnector, HandshakeBuilder};
    /// use grpc_mesh_node::rpc::InvokeService;
    /// use grpc_mesh_node::MethodRegistry;
    /// use tokio::sync::watch;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = ConnectorConfig::default();
    /// let handshake = HandshakeBuilder::new("my-node-id")
    ///     .version("1.0.0")
    ///     .build()?;
    /// let (_tx, rx) = watch::channel(false);
    ///
    /// let mut connector = TunnelConnector::new(config, rx)?;
    /// let incoming = connector.connect_with_backoff(handshake).await?;
    ///
    /// // Serve gRPC requests
    /// let registry = MethodRegistry::default();
    /// let service = InvokeService::new(registry).into_server();
    /// tonic::transport::Server::builder()
    ///     .add_service(service)
    ///     .serve_with_incoming(incoming)
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect_with_backoff(&mut self, handshake: Handshake) -> Result<YamuxIncoming> {
        let mut attempt: u32 = 0;

        loop {
            if *self.shutdown_rx.borrow() {
                return Err(TunnelError::Shutdown);
            }

            match self.try_connect(handshake.clone()).await {
                Ok(tunnel) => {
                    let heartbeat_interval = self.cfg.heartbeat_interval;
                    let shutdown_rx = self.shutdown_rx.clone();
                    tracing::info!(
                        "💓 Starting automatic heartbeat (interval: {:?})",
                        heartbeat_interval
                    );
                    return tunnel.start_heartbeat(shutdown_rx, heartbeat_interval);
                }
                Err(err) if err.is_retryable() => {
                    attempt = attempt.saturating_add(1);
                    let delay = backoff_delay(attempt, self.cfg.max_backoff);
                    tokio::select! {
                        _ = sleep(delay) => {}
                        _ = self.shutdown_rx.changed() => {
                            if *self.shutdown_rx.borrow() {
                                return Err(TunnelError::Shutdown);
                            }
                        }
                    }
                }
                Err(err) => return Err(err),
            }
        }
    }

    async fn try_connect(&self, handshake: Handshake) -> Result<Tunnel> {
        tracing::debug!("Attempting TCP connection to {}", self.cfg.server_addr);
        let stream = match timeout(
            self.cfg.connect_timeout,
            TcpStream::connect(&self.cfg.server_addr),
        )
        .await
        {
            Ok(Ok(stream)) => stream,
            Ok(Err(err)) => return Err(TunnelError::Network(err)),
            Err(elapsed) => return Err(TunnelError::Timeout(elapsed)),
        };
        tracing::debug!("TCP connection established");

        stream.set_nodelay(true).map_err(TunnelError::Network)?;

        let server_name_str = self.cfg.sni.clone().unwrap_or_else(|| {
            self.cfg
                .server_addr
                .split(':')
                .next()
                .unwrap_or("localhost")
                .to_owned()
        });

        let server_name = ServerName::try_from(server_name_str.clone())
            .map_err(|_| TunnelError::Config(format!("invalid SNI hostname: {server_name_str}")))?;

        tracing::debug!("Starting TLS handshake with SNI: {}", server_name_str);
        let tls_stream = TlsConnector::from(self.tls.clone())
            .connect(server_name, stream)
            .await
            .map_err(|e| {
                tracing::error!("TLS handshake failed: {:?}", e);
                e
            })?;
        tracing::debug!("TLS handshake successful");

        // Convert the Tokio IO into a futures-compatible IO for yamux.
        tracing::debug!("Creating yamux connection");
        let yamux_io = tls_stream.compat();
        let yamux_conn = Connection::new(yamux_io, self.yamux_cfg.clone(), Mode::Client);

        let handshake = Arc::new(handshake.ensure_timestamp()?);

        // Open the control stream to send the registration handshake.
        tracing::debug!("Opening control stream");
        let mut yamux_conn = yamux_conn;
        let control_stream = loop {
            match yamux_conn.poll_new_outbound(&mut std::task::Context::from_waker(
                futures::task::noop_waker_ref(),
            )) {
                std::task::Poll::Ready(Ok(stream)) => break stream,
                std::task::Poll::Ready(Err(e)) => {
                    tracing::error!("Failed to open yamux control stream: {:?}", e);
                    return Err(TunnelError::Yamux(e));
                }
                std::task::Poll::Pending => {
                    tokio::task::yield_now().await;
                }
            }
        };
        let mut control_stream = control_stream;
        tracing::debug!("Control stream opened successfully");
        {
            tracing::debug!("Sending handshake");
            let mut compat = control_stream.compat_write();
            timeout(
                self.cfg.connect_timeout,
                send_handshake(&mut compat, &handshake),
            )
            .await
            .map_err(TunnelError::Timeout)??;
            control_stream = compat.into_inner();
            tracing::debug!("Handshake sent successfully");
        }

        let incoming = YamuxIncoming::new(yamux_conn, Arc::clone(&handshake));

        tracing::info!("Tunnel connection established successfully");
        Ok(Tunnel {
            handshake,
            incoming: Some(incoming),
            control_stream,
        })
    }
}

/// Builds a TLS client configuration from the connector config.
///
/// This function creates a rustls `ClientConfig` with appropriate root certificates:
/// - If custom CA certificates are provided in the config, only those are used
/// - Otherwise, the system's default root certificate store is used
///
/// # Parameters
///
/// - `cfg`: Connector configuration containing CA certificates
///
/// # Returns
///
/// - `Ok(ClientConfig)`: Ready-to-use TLS configuration
/// - `Err(TunnelError)`: If CA certificates are invalid or cannot be parsed
fn build_tls_config(cfg: &ConnectorConfig) -> Result<ClientConfig> {
    let mut roots = RootCertStore::empty();

    // If custom CA certs are provided, use only those (don't include system roots)
    // This is important for self-signed certificates
    if !cfg.ca_certs.is_empty() {
        tracing::debug!("Loading {} custom CA certificate(s)", cfg.ca_certs.len());
        let mut cert_count = 0;
        for pem in &cfg.ca_certs {
            let mut cursor = Cursor::new(pem);
            for cert in rustls_pemfile::certs(&mut cursor) {
                let cert =
                    cert.map_err(|err| TunnelError::Config(format!("parse CA cert: {err}")))?;
                tracing::debug!("Adding CA certificate, size: {} bytes", cert.as_ref().len());
                roots
                    .add(cert)
                    .map_err(|err| TunnelError::Config(format!("invalid CA certificate: {err}")))?;
                cert_count += 1;
            }
        }
        tracing::debug!(
            "Successfully loaded {} CA certificate(s) into trust store",
            cert_count
        );
    } else {
        // Only use system roots if no custom CA is provided
        tracing::debug!("Using system root certificates");
        roots.extend(TLS_SERVER_ROOTS.iter().cloned());
    }

    tracing::debug!("TLS config built with {} root certificates", roots.len());

    Ok(ClientConfig::builder()
        .with_root_certificates(Arc::new(roots))
        .with_no_client_auth())
}

/// Calculates exponential backoff delay for retry attempts.
///
/// The delay starts at 250ms and doubles with each attempt (up to attempt 6),
/// then is capped at the maximum delay.
///
/// # Parameters
///
/// - `attempt`: Retry attempt number (0-indexed)
/// - `max_delay`: Maximum delay to return
///
/// # Returns
///
/// Delay duration for this attempt, between 250ms and max_delay
///
/// # Example Delays
///
/// - Attempt 0: 250ms
/// - Attempt 1: 500ms
/// - Attempt 2: 1s
/// - Attempt 3: 2s
/// - Attempt 4: 4s
/// - Attempt 5: 8s
/// - Attempt 6+: 16s (or max_delay if lower)
fn backoff_delay(attempt: u32, max_delay: Duration) -> Duration {
    let capped = attempt.min(6);
    let millis = 250u64.saturating_mul(1u64 << capped);
    min(Duration::from_millis(millis), max_delay)
}
