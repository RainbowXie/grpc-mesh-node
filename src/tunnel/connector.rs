use std::cmp::min;
use std::io::Cursor;
use std::sync::Arc;
use std::time::Duration;

use futures::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::watch;
use tokio::time::{interval, sleep, timeout, timeout_at};
use tokio_rustls::TlsConnector;
use tokio_rustls::rustls::client::danger::{
    HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier,
};
use tokio_rustls::rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use tokio_rustls::rustls::{
    ClientConfig, DigitallySignedStruct, Error as RustlsError, RootCertStore, SignatureScheme,
};
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

    /// Skip TLS certificate verification entirely.
    ///
    /// This is intended only for controlled development scenarios where the server
    /// rotates self-signed certificates dynamically and the client cannot preload
    /// the active CA bundle.
    pub insecure_skip_verify: bool,
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
            insecure_skip_verify: false,
        }
    }
}

#[derive(Debug)]
struct NoCertificateVerification;

impl ServerCertVerifier for NoCertificateVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> std::result::Result<ServerCertVerified, RustlsError> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, RustlsError> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> std::result::Result<HandshakeSignatureValid, RustlsError> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::ECDSA_NISTP384_SHA384,
            SignatureScheme::ED25519,
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::RSA_PSS_SHA384,
            SignatureScheme::RSA_PSS_SHA512,
            SignatureScheme::RSA_PKCS1_SHA256,
            SignatureScheme::RSA_PKCS1_SHA384,
            SignatureScheme::RSA_PKCS1_SHA512,
        ]
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
    /// interval. The heartbeat loop continues until the shutdown signal is received, or until
    /// a heartbeat write fails or stalls past its own interval — in that case the tunnel is
    /// presumed dead and `tunnel_dead` is flagged so the caller's supervisor can tear the
    /// connection down and reconnect.
    ///
    /// # Parameters
    ///
    /// - `shutdown_rx`: Watch channel receiver for shutdown coordination
    /// - `heartbeat_interval`: Time between heartbeat messages
    /// - `tunnel_dead`: Watch channel sender flagged when heartbeat writes can no longer complete
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
    /// This is called automatically by `TunnelConnector::connect_with_backoff_watched()`.
    /// Most applications don't need to call this directly.
    fn start_heartbeat(
        mut self,
        shutdown_rx: watch::Receiver<bool>,
        heartbeat_interval: Duration,
        tunnel_dead: watch::Sender<bool>,
    ) -> Result<YamuxIncoming> {
        let node_id = self.handshake.node_id.clone();
        let control_stream = self.control_stream;
        let incoming = self
            .incoming
            .take()
            .ok_or(TunnelError::Protocol("incoming already taken".into()))?;

        tokio::spawn(run_heartbeat(
            control_stream,
            node_id,
            heartbeat_interval,
            shutdown_rx,
            tunnel_dead,
        ));

        Ok(incoming)
    }
}

/// Sends heartbeat frames on the control stream until shutdown, or until a
/// write fails or stalls past `heartbeat_interval`.
///
/// A heartbeat write that cannot complete within its own interval means the
/// tunnel is unusable — the peer stopped reading and the yamux send window is
/// exhausted, or the connection is gone. The loop then closes the stream
/// (best-effort FIN) and flags `tunnel_dead`; the previous behavior of logging
/// a warning and looping forever left the task frozen on a wedged connection.
async fn run_heartbeat<W>(
    mut writer: W,
    node_id: String,
    heartbeat_interval: Duration,
    shutdown_rx: watch::Receiver<bool>,
    tunnel_dead: watch::Sender<bool>,
) where
    W: futures::io::AsyncWrite + Unpin,
{
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

        // One heartbeat must complete within its own interval on a healthy
        // tunnel; anything slower is a wedged or dead connection.
        let write_result = timeout(heartbeat_interval, async {
            writer.write_all(&frame).await?;
            writer.flush().await
        })
        .await;

        match write_result {
            Ok(Ok(())) => {
                tracing::debug!("💓 Heartbeat sent (seq={})", sequence - 1);
            }
            Ok(Err(err)) => {
                tracing::error!(
                    sequence,
                    error = %err,
                    "grpc-mesh heartbeat write failed; tunnel presumed dead"
                );
                break;
            }
            Err(elapsed) => {
                tracing::error!(
                    sequence,
                    elapsed = ?elapsed,
                    "grpc-mesh heartbeat write timed out; tunnel presumed dead"
                );
                break;
            }
        }
    }

    if !*shutdown_rx.borrow() {
        // Best-effort FIN so a live server notices immediately; the supervisor
        // drops the whole connection (and its socket) when it acts on the flag.
        let _ = timeout(Duration::from_secs(1), writer.close()).await;
        tunnel_dead.send_replace(true);
    }
}

/// Classifies a TLS handshake failure into a retry decision.
///
/// tokio-rustls wraps rustls protocol errors in `io::Error`; certificate
/// rejections, peer alerts and peer misbehavior are configuration problems
/// (wrong CA, expired cert, SNI mismatch, ...) that retrying cannot fix, so
/// they surface as terminal `TunnelError::Config` instead of feeding the
/// connect-retry loop forever. Network-level failures stay retryable.
fn classify_tls_handshake_error(err: std::io::Error) -> TunnelError {
    if let Some(rustls_err) = err.get_ref().and_then(|e| e.downcast_ref::<RustlsError>()) {
        let permanent = matches!(
            rustls_err,
            RustlsError::InvalidCertificate(_)
                | RustlsError::AlertReceived(_)
                | RustlsError::PeerMisbehaved(_)
                | RustlsError::General(_)
        );
        if permanent {
            return TunnelError::Config(format!(
                "TLS handshake rejected: {rustls_err} — check the CA bundle, certificate validity and SNI; not retryable"
            ));
        }
        return TunnelError::Tls(rustls_err.clone());
    }
    TunnelError::Network(err)
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
        let (tunnel_dead, _dead_rx) = watch::channel(false);
        self.connect_with_backoff_watched(handshake, tunnel_dead).await
    }

    /// Establishes a tunnel with automatic retry, and reports tunnel death.
    ///
    /// Behaves like [`connect_with_backoff`], but the spawned heartbeat task
    /// flags the provided `tunnel_dead` watch channel when its writes stall or
    /// fail after the connection is established. A supervisor that selects on
    /// the receiver can then drop the tunnel (closing its socket) and
    /// reconnect; without this, a peer that vanishes without a FIN leaves the
    /// serving side waiting forever.
    ///
    /// The receiver may be dropped; signalling into a dropped watch channel
    /// is a no-op.
    pub async fn connect_with_backoff_watched(
        &mut self,
        handshake: Handshake,
        tunnel_dead: watch::Sender<bool>,
    ) -> Result<YamuxIncoming> {
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
                    return tunnel.start_heartbeat(shutdown_rx, heartbeat_interval, tunnel_dead);
                }
                Err(err) if err.is_retryable() => {
                    attempt = attempt.saturating_add(1);
                    let delay = backoff_delay(attempt, self.cfg.max_backoff);
                    // Greppable prefix keeps node-side connect/handshake failures correlatable with server-side tunnel logs during cross-side diagnosis.
                    tracing::warn!(
                        attempt,
                        server_addr = %self.cfg.server_addr,
                        sni = ?self.cfg.sni,
                        error = %err,
                        "grpc-mesh retryable connect failure"
                    );
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
        // One budget for the whole establishment: TCP + TLS + yamux setup +
        // handshake frame, as documented on ConnectorConfig::connect_timeout.
        let connect_deadline = tokio::time::Instant::now()
            .checked_add(self.cfg.connect_timeout)
            .ok_or_else(|| {
                TunnelError::Config("connect_timeout overflowed the clock".to_string())
            })?;

        tracing::debug!("Attempting TCP connection to {}", self.cfg.server_addr);
        let stream = match timeout_at(
            connect_deadline,
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
        tracing::info!(
            server_addr = %self.cfg.server_addr,
            server_name = %server_name_str,
            "grpc-mesh TLS handshake start"
        );

        tracing::debug!("Starting TLS handshake with SNI: {}", server_name_str);
        let tls_stream = match timeout_at(
            connect_deadline,
            TlsConnector::from(self.tls.clone()).connect(server_name, stream),
        )
        .await
        {
            Ok(Ok(tls_stream)) => tls_stream,
            Ok(Err(err)) => {
                tracing::error!(
                    server_addr = %self.cfg.server_addr,
                    server_name = %server_name_str,
                    error = ?err,
                    "grpc-mesh TLS handshake failed"
                );
                return Err(classify_tls_handshake_error(err));
            }
            Err(elapsed) => {
                tracing::error!(
                    server_addr = %self.cfg.server_addr,
                    server_name = %server_name_str,
                    elapsed = ?elapsed,
                    "grpc-mesh TLS handshake timed out"
                );
                return Err(TunnelError::Timeout(elapsed));
            }
        };
        tracing::info!(
            server_addr = %self.cfg.server_addr,
            server_name = %server_name_str,
            "grpc-mesh TLS handshake success"
        );
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
            timeout_at(connect_deadline, send_handshake(&mut compat, &handshake))
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
    if cfg.insecure_skip_verify {
        tracing::warn!("TLS certificate verification is disabled for this connector");
        return Ok(ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(NoCertificateVerification))
            .with_no_client_auth());
    }

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
        // An empty trust store would fail every handshake with a misleading
        // "unknown issuer" and feed the retry loop forever; fail fast instead.
        if cert_count == 0 {
            return Err(TunnelError::Config(
                "ca_certs contained PEM data but no certificates could be parsed; \
                 refusing to build an empty trust store"
                    .into(),
            ));
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

#[cfg(test)]
mod tests {
    use super::*;
    use futures::io::AsyncWrite;
    use std::pin::Pin;
    use std::task::{Context, Poll};

    /// A writer whose writes never complete — the on-wire equivalent of a
    /// yamux stream whose send window is exhausted because the peer stopped
    /// reading (the original permanent-freeze failure mode).
    struct StuckWriter;

    impl AsyncWrite for StuckWriter {
        fn poll_write(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            _buf: &[u8],
        ) -> Poll<std::io::Result<usize>> {
            Poll::Pending
        }

        fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Pending
        }

        fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Pending
        }
    }

    #[tokio::test]
    async fn heartbeat_escapes_stuck_writer() {
        let (_shutdown_tx, shutdown_rx) = watch::channel(false);
        let (dead_tx, mut dead_rx) = watch::channel(false);

        // 10ms interval doubles as the per-write budget; the outer timeout is
        // the assertion: without the write timeout the loop hangs forever.
        tokio::time::timeout(
            Duration::from_secs(3),
            run_heartbeat(
                StuckWriter,
                "node-1".into(),
                Duration::from_millis(10),
                shutdown_rx,
                dead_tx,
            ),
        )
        .await
        .expect("heartbeat loop must escape a stuck writer instead of freezing");

        assert!(*dead_rx.borrow(), "tunnel_dead must be signalled");
    }

    #[test]
    fn tls_error_classification() {
        use tokio_rustls::rustls::CertificateError;

        let expired = classify_tls_handshake_error(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            RustlsError::InvalidCertificate(CertificateError::Expired),
        ));
        assert!(
            matches!(expired, TunnelError::Config(_)),
            "certificate rejection must be terminal, got {expired:?}"
        );
        assert!(!expired.is_retryable());

        let reset = classify_tls_handshake_error(std::io::Error::new(
            std::io::ErrorKind::ConnectionReset,
            "connection reset",
        ));
        assert!(reset.is_retryable(), "network failure must stay retryable");
    }

    #[test]
    fn empty_ca_bundle_is_rejected() {
        let cfg = ConnectorConfig {
            ca_certs: vec![b"not a pem certificate".to_vec()],
            ..Default::default()
        };
        let (_tx, rx) = watch::channel(false);
        let err = match TunnelConnector::new(cfg, rx) {
            Ok(_) => panic!("empty CA bundle must be rejected at construction"),
            Err(err) => err,
        };
        assert!(matches!(err, TunnelError::Config(_)), "got {err:?}");
    }
}
