use std::io;

use thiserror::Error;
use tokio::time::error::Elapsed;
use tokio_rustls::rustls::Error as RustlsError;
use yamux::ConnectionError;

use super::handshake::HandshakeError;

/// Convenient alias for results produced by tunnel components.
pub type Result<T, E = TunnelError> = std::result::Result<T, E>;

/// Enumerates all recoverable and terminal failures that can be emitted by the
/// TLS + Yamux tunnel layer.
#[derive(Debug, Error)]
pub enum TunnelError {
    /// Static or runtime configuration is invalid.
    #[error("tunnel configuration error: {0}")]
    Config(String),

    /// TCP socket or DNS resolution failed.
    #[error("socket error: {0}")]
    Network(#[from] io::Error),

    /// TLS stack rejected the handshake or encountered cryptographic issues.
    #[error("tls error: {0}")]
    Tls(#[from] RustlsError),

    /// Yamux reported a protocol-level failure.
    #[error("yamux error: {0}")]
    Yamux(#[from] ConnectionError),

    /// Control-stream handshake encoding/decoding failed.
    #[error("handshake error: {0}")]
    Handshake(#[from] HandshakeError),

    /// The operation exceeded its context or handshake deadline.
    #[error("operation timed out: {0}")]
    Timeout(#[from] Elapsed),

    /// The remote peer closed the control stream unexpectedly.
    #[error("control stream closed unexpectedly")]
    ControlStreamClosed,

    /// The connection manager is shutting down.
    #[error("tunnel manager stopped")]
    Shutdown,

    /// Protocol-level error or invalid state.
    #[error("protocol error: {0}")]
    Protocol(String),
}

impl TunnelError {
    /// Returns `true` if the error represents a transient state and the caller
    /// can safely attempt a reconnection after applying backoff.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            TunnelError::Network(_)
                | TunnelError::Tls(_)
                | TunnelError::Yamux(_)
                | TunnelError::Handshake(_)
                | TunnelError::Timeout(_)
        )
    }

    /// Describes whether the error signals that the tunnel should stop retrying.
    pub fn is_terminal(&self) -> bool {
        matches!(self, TunnelError::Config(_) | TunnelError::Shutdown)
    }
}
