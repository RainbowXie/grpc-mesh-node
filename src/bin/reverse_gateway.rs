use std::{collections::HashMap, sync::Arc};

use grpc_mesh::{
    MethodRegistry, RpcResult,
    rpc::InvokeService,
    tunnel::{ConnectorConfig, Handshake, TunnelConnector},
};
use serde::Deserialize;
use thiserror::Error;
use tokio::{select, signal, sync::watch};
use tonic::transport::Server;
use tracing::{Level, error, info, warn};
use uuid::Uuid;

// ============================================================================
// EMBEDDED CONFIGURATION AND CERTIFICATES
// ============================================================================

/// Embedded configuration file (compiled into binary)
const EMBEDDED_CONFIG: &str = include_str!("../../config/reverse_gateway_config.json");

/// Embedded CA certificate (compiled into binary)
const EMBEDDED_CA_CERT: &str = include_str!("../../config/ca.crt");

// ============================================================================
// CONFIGURATION STRUCTURES
// ============================================================================

#[derive(Debug, Deserialize)]
struct EmbeddedConfig {
    server: ServerConfig,
    node: NodeConfig,
    heartbeat: HeartbeatConfig,
    reconnect: ReconnectConfig,
    #[serde(default)]
    yamux: YamuxConfig,
    #[serde(default)]
    logging: LoggingConfig,
}

#[derive(Debug, Deserialize)]
struct ServerConfig {
    address: String,
    tls: TlsConfig,
}

#[derive(Debug, Deserialize)]
struct TlsConfig {
    #[serde(skip)]
    ca_cert_path: String, // Not used, we use embedded cert
    server_name: String,
}

#[derive(Debug, Deserialize)]
struct NodeConfig {
    id: String,
    token: String,
    version: String,
    supported_features: Vec<String>,
    #[serde(default)]
    metadata: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct HeartbeatConfig {
    interval_secs: u64,
}

#[derive(Debug, Deserialize)]
struct ReconnectConfig {
    base_delay_secs: u64,
    max_delay_secs: u64,
    max_retries: u32,
}

#[derive(Debug, Deserialize, Default)]
struct YamuxConfig {
    #[serde(default)]
    window_update_mode: String,
    #[serde(default)]
    enable_keepalive: bool,
}

#[derive(Debug, Deserialize)]
struct LoggingConfig {
    level: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Print banner
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  wa-emu Reverse Gateway (Embedded Config)                  ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();

    // Parse embedded configuration
    let embedded_cfg: EmbeddedConfig = serde_json::from_str(EMBEDDED_CONFIG)
        .map_err(|e| format!("Failed to parse embedded config: {}", e))?;

    // Initialize tracing with configured level
    init_tracing_with_level(&embedded_cfg.logging.level);

    let cfg = GatewayConfig::from_embedded(embedded_cfg)?;

    println!("Configuration (embedded):");
    println!("  Server:   {}", cfg.server_addr);
    println!(
        "  Node ID:  {} {}",
        cfg.node_id,
        if cfg.is_auto_generated {
            "(auto-generated)"
        } else {
            ""
        }
    );
    println!("  Version:  {}", cfg.version);
    println!(
        "  Token:    {}",
        if cfg.token.is_empty() {
            "<none>"
        } else {
            "<configured>"
        }
    );
    println!("  Features: {:?}", cfg.features);
    println!();

    info!(
        target = "reverse-gateway",
        server = cfg.server_addr,
        node_id = cfg.node_id,
        auto_generated = cfg.is_auto_generated,
        "Starting reverse gateway with embedded config"
    );
    let handshake = cfg.build_handshake()?;

    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    tokio::spawn(ctrl_c_listener(shutdown_tx.clone()));

    let mut connector = TunnelConnector::new(cfg.connector_config()?, shutdown_rx.clone())?;

    // Tunnel supervision: an established tunnel that drops or wedges is
    // reconnected with backoff instead of exiting. `tunnel_dead` is flagged by
    // the heartbeat task when its writes stall or fail — covering peers that
    // vanish without a FIN, where serving would otherwise wait forever.
    let (tunnel_dead_tx, tunnel_dead_rx) = watch::channel(false);
    let mut reconnect_attempt: u32 = 0;

    loop {
        if *shutdown_rx.borrow() {
            break;
        }
        tunnel_dead_tx.send_replace(false);
        let connected_at = std::time::Instant::now();

        let incoming = match connector
            .connect_with_backoff_watched(handshake.clone(), tunnel_dead_tx.clone())
            .await
        {
            Ok(incoming) => incoming,
            Err(grpc_mesh::tunnel::TunnelError::Shutdown) => break,
            Err(err) => return Err(err.into()),
        };
        info!(
            target = "reverse-gateway",
            node_id = %handshake.node_id,
            version = %handshake.version,
            "Tunnel established with automatic heartbeat"
        );

        // Handlers registered by setup_registry are stateless, so a fresh
        // registry per connection is equivalent to reusing one.
        let grpc = Server::builder()
            .add_service(InvokeService::new(setup_registry()).into_server())
            .serve_with_incoming_shutdown(incoming, shutdown_rx_changed(shutdown_rx.clone()));

        select! {
            result = grpc => match result {
                Ok(()) => warn!(target = "reverse-gateway", "tunnel ended; reconnecting"),
                Err(err) => {
                    warn!(target = "reverse-gateway", %err, "gRPC serving over tunnel failed; reconnecting")
                }
            },
            _ = wait_tunnel_dead(tunnel_dead_rx.clone()) => {
                warn!(
                    target = "reverse-gateway",
                    "heartbeat writes stalled; tunnel presumed dead, reconnecting"
                );
            }
        }

        // Connections that die quickly escalate backoff (a broken peer must
        // not cause a hot reconnect loop); a healthy stretch resets it.
        reconnect_attempt = if connected_at.elapsed() >= std::time::Duration::from_secs(60) {
            0
        } else {
            reconnect_attempt.saturating_add(1)
        };
        let delay = reconnect_delay(reconnect_attempt);
        info!(
            target = "reverse-gateway",
            ?delay,
            attempt = reconnect_attempt,
            "waiting before reconnecting"
        );
        select! {
            _ = tokio::time::sleep(delay) => {}
            _ = shutdown_rx_changed(shutdown_rx.clone()) => break,
        }
    }

    info!(target = "reverse-gateway", "Gateway shut down cleanly");
    Ok(())
}

fn init_tracing() {
    init_tracing_with_level("info");
}

fn init_tracing_with_level(level: &str) {
    let level = match level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    if tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| format!("{}", level).into()),
            )
            .with_max_level(level)
            .finish(),
    )
    .is_err()
    {
        eprintln!("tracing already initialized");
    }
}

fn setup_registry() -> MethodRegistry {
    let registry = MethodRegistry::default();
    registry.register(
        "health.echo",
        Arc::new(|payload: Vec<u8>| -> RpcResult<Vec<u8>> { Ok(payload) }),
    );
    registry.register(
        "health.ping",
        Arc::new(|_payload: Vec<u8>| -> RpcResult<Vec<u8>> { Ok(b"PONG".to_vec()) }),
    );
    registry
}

async fn ctrl_c_listener(mut shutdown_tx: watch::Sender<bool>) {
    if signal::ctrl_c().await.is_ok() {
        let _ = shutdown_tx.send(true);
    }
}

async fn shutdown_rx_changed(mut rx: watch::Receiver<bool>) {
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

/// Reconnect pacing for tunnels that die quickly, capped like the
/// connector's own connect backoff (250ms base, 30s ceiling).
fn reconnect_delay(attempt: u32) -> std::time::Duration {
    let capped = attempt.min(6);
    let millis = 250u64.saturating_mul(1 << capped);
    std::time::Duration::from_millis(millis).min(std::time::Duration::from_secs(30))
}

#[derive(Debug, Clone)]
struct GatewayConfig {
    server_addr: String,
    node_id: String,
    version: String,
    token: String,
    sni: Option<String>,
    features: Vec<String>,
    ca_bundle_path: Option<String>,
    max_backoff_secs: u64,
    is_auto_generated: bool,
}

impl GatewayConfig {
    fn from_embedded(cfg: EmbeddedConfig) -> Result<Self, ConfigError> {
        // Determine node_id with priority: ENV > config (if not auto) > auto-generate
        let (node_id, is_auto_generated) = if let Ok(env_id) = std::env::var("WA_NODE_ID") {
            warn!(
                "Using node_id from environment variable WA_NODE_ID: {}",
                env_id
            );
            (env_id, false)
        } else if cfg.node.id.is_empty() || cfg.node.id == "auto" {
            let generated = generate_node_id();
            info!("Auto-generated node_id: {}", generated);
            (generated, true)
        } else {
            (cfg.node.id, false)
        };

        Ok(Self {
            server_addr: cfg.server.address,
            node_id,
            version: cfg.node.version,
            token: cfg.node.token,
            sni: Some(cfg.server.tls.server_name),
            features: cfg.node.supported_features,
            ca_bundle_path: None, // We use embedded CA cert
            max_backoff_secs: cfg.reconnect.max_delay_secs,
            is_auto_generated,
        })
    }

    fn from_env() -> Result<Self, ConfigError> {
        let node_id = std::env::var("WA_NODE_ID").unwrap_or_else(|_| generate_node_id());
        let is_auto_generated = std::env::var("WA_NODE_ID").is_err();

        Ok(Self {
            server_addr: std::env::var("WA_SERVER_ADDR")
                .unwrap_or_else(|_| "127.0.0.1:8443".into()),
            node_id,
            version: std::env::var("WA_NODE_VERSION").unwrap_or_else(|_| "0.0.1".into()),
            token: std::env::var("WA_NODE_TOKEN")
                .map_err(|_| ConfigError::Missing("WA_NODE_TOKEN"))?,
            sni: std::env::var("WA_TLS_SNI").ok(),
            features: std::env::var("WA_NODE_FEATURES")
                .map(|value| {
                    value
                        .split(',')
                        .filter_map(|item| {
                            let trimmed = item.trim();
                            (!trimmed.is_empty()).then(|| trimmed.to_lowercase())
                        })
                        .collect()
                })
                .unwrap_or_else(|_| vec!["yamux-reverse-grpc".into()]),
            ca_bundle_path: std::env::var("WA_TLS_CA_BUNDLE").ok(),
            max_backoff_secs: std::env::var("WA_MAX_BACKOFF_SECS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
            is_auto_generated,
        })
    }

    fn build_handshake(&self) -> Result<Handshake, ConfigError> {
        let mut builder = Handshake::builder(self.node_id.clone(), self.version.clone())
            .token(self.token.clone())
            .metadata_entry("platform", std::env::consts::OS)
            .metadata_entry("arch", std::env::consts::ARCH);

        for feature in &self.features {
            builder = builder.add_feature(feature.clone());
        }

        builder.build().map_err(ConfigError::from)
    }

    fn connector_config(&self) -> Result<ConnectorConfig, ConfigError> {
        // Pass PEM bytes through untouched: ConnectorConfig.ca_certs is
        // documented as PEM and the connector owns parsing. The previous
        // code pre-parsed to DER here, which the connector then re-parsed
        // as PEM — silently yielding an empty trust store, so custom CAs
        // never verified and self-signed servers retried forever.
        let ca_certs: Vec<Vec<u8>> = if let Some(path) = &self.ca_bundle_path {
            vec![std::fs::read(path).map_err(|err| ConfigError::Io {
                path: path.clone(),
                err,
            })?]
        } else {
            vec![EMBEDDED_CA_CERT.as_bytes().to_vec()]
        };

        // Fail fast on material that cannot yield a certificate.
        for pem in &ca_certs {
            let mut cursor = std::io::Cursor::new(pem);
            let parsed_any = rustls_pemfile::certs(&mut cursor)
                .any(|cert| cert.is_ok());
            if !parsed_any {
                return Err(ConfigError::InvalidCa("<configured CA bundle>".into()));
            }
        }

        Ok(ConnectorConfig {
            server_addr: self.server_addr.clone(),
            ca_certs,
            sni: self.sni.clone(),
            connect_timeout: std::time::Duration::from_secs(10),
            max_backoff: std::time::Duration::from_secs(self.max_backoff_secs),
            heartbeat_interval: std::time::Duration::from_secs(15),
            // The shipped binary never weakens verification; there is deliberately no CLI flag for this.
            insecure_skip_verify: false,
        })
    }
}

/// Generate a unique node ID using hostname + short UUID
/// Format: {hostname}-{uuid_prefix}
/// Example: myserver-a1b2c3d4
fn generate_node_id() -> String {
    let hostname = hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "node".to_string());

    // Sanitize hostname: replace non-alphanumeric chars with dash
    let sanitized_hostname: String = hostname
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();

    // Generate short UUID (first 8 characters)
    let uuid = Uuid::new_v4();
    let uuid_short = format!("{}", uuid).chars().take(8).collect::<String>();

    format!("{}-{}", sanitized_hostname, uuid_short)
}

#[derive(Debug, Error)]
enum ConfigError {
    #[error("missing environment variable: {0}")]
    Missing(&'static str),
    #[error("handshake error: {0}")]
    Handshake(#[from] grpc_mesh::tunnel::HandshakeError),
    #[error("io error reading {path}: {err}")]
    Io { path: String, err: std::io::Error },
    #[error("invalid CA bundle: {0}")]
    InvalidCa(String),
}
