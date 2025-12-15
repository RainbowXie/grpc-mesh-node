//! TLS + Yamux tunnel primitives shared by the Rust agents.
//!
//! This module collects the building blocks required to establish outbound
//! tunnels towards the Go control plane:
//! - [`handshake`] provides the JSON control-frame schema and helpers to encode
//!   / decode framed payloads compatibly with the server-side implementation.
//! - [`error`] enumerates the error types emitted by the tunnel layer and
//!   exposes a convenient `Result` alias for downstream consumers.
//! - [`incoming`] adapts Yamux connections into tonic-friendly incoming streams
//!   while preserving handshake metadata for each transport.

pub mod connector;
pub mod error;
pub mod handshake;
pub mod incoming;

pub use connector::*;
pub use error::{Result as TunnelResult, TunnelError};
pub use handshake::{
    CONTROL_FRAME_HEADER_LEN, DEFAULT_MAX_FRAME_LEN, Handshake, HandshakeBuilder, HandshakeError,
    read_frame, recv_handshake, send_handshake, write_frame,
};
pub use incoming::{HandshakeConnectInfo, YamuxIncoming, YamuxTransport};
