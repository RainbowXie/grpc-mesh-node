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

/// High-level configuration for the outbound TLS + Yamux tunnel.
#[derive(Debug, Clone)]
pub struct ConnectorConfig {
    /// TCP endpoint exposed by the Go control plane.
    pub server_addr: String,
    /// Optional PEM-encoded CA certificates appended to the default root store.
    pub ca_certs: Vec<Vec<u8>>,
    /// Override for TLS SNI (falls back to the host part of `server_addr`).
    pub sni: Option<String>,
    /// Per-attempt dial timeout.
    pub connect_timeout: Duration,
    /// Maximum delay when exponentially backing off between retries.
    pub max_backoff: Duration,
    /// Heartbeat interval (default: 15 seconds).
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

/// Represents an established TLS + Yamux tunnel.
pub struct Tunnel {
    handshake: Arc<Handshake>,
    incoming: Option<YamuxIncoming>,
    control_stream: Stream,
}

impl Tunnel {
    /// Returns the handshake metadata negotiated with the control plane.
    pub fn handshake(&self) -> &Handshake {
        &self.handshake
    }

    /// Provides mutable access to the control stream (heartbeats, registry sync, etc.).
    pub fn control_stream(&mut self) -> &mut Stream {
        &mut self.control_stream
    }

    /// Transfers ownership of the Yamux incoming adapter to the caller.
    pub fn take_incoming(&mut self) -> Option<YamuxIncoming> {
        self.incoming.take()
    }

    /// Consumes the tunnel and returns all owned pieces.
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
    /// Returns the incoming adapter and spawns a task to send heartbeats periodically.
    /// This is called automatically by `connect_with_backoff` and should not be called directly.
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

/// Drives the dial/handshake/retry lifecycle for a single Rust node.
pub struct TunnelConnector {
    cfg: ConnectorConfig,
    tls: Arc<ClientConfig>,
    yamux_cfg: YamuxConfig,
    shutdown_rx: watch::Receiver<bool>,
}

impl TunnelConnector {
    /// Builds a new connector from the provided configuration and shutdown signal.
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

    /// Attempts to establish a tunnel, retrying with exponential backoff on transient failures.
    /// Automatically starts heartbeat in the background and returns the incoming adapter.
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

fn backoff_delay(attempt: u32, max_delay: Duration) -> Duration {
    let capped = attempt.min(6);
    let millis = 250u64.saturating_mul(1u64 << capped);
    min(Duration::from_millis(millis), max_delay)
}
