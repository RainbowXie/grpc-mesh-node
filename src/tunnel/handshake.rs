use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Number of bytes used by the length-prefixed control frame header.
pub const CONTROL_FRAME_HEADER_LEN: usize = 4;

/// Default upper bound for any control frame payload.
pub const DEFAULT_MAX_FRAME_LEN: usize = 1 << 20; // 1 MiB

/// Errors that may occur while constructing or transmitting handshake frames.
#[derive(Debug, Error)]
pub enum HandshakeError {
    /// Required field was missing or empty.
    #[error("missing field: {0}")]
    MissingField(&'static str),
    /// A field contained invalid characters or formatting.
    #[error("invalid field {field}: {reason}")]
    InvalidField { field: &'static str, reason: String },
    /// Serializing the handshake payload as JSON failed.
    #[error("serialize handshake: {0}")]
    Serialize(#[from] serde_json::Error),
    /// The frame exceeds the configured maximum length.
    #[error("frame too large ({actual} > {max})")]
    FrameTooLarge { actual: usize, max: usize },
    /// I/O errors encountered while sending/receiving frames.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// System time was before UNIX_EPOCH when generating timestamps.
    #[error("system clock error: {0}")]
    Clock(#[from] std::time::SystemTimeError),
}

/// JSON payload exchanged over the control stream when a Rust node connects.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Handshake {
    #[serde(rename = "node_id")]
    pub node_id: String,
    pub token: String,
    pub version: String,
    #[serde(default, rename = "supported_features")]
    pub supported_features: Vec<String>,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
    #[serde(rename = "timestamp_unix_sec")]
    pub timestamp_unix_sec: i64,
}

impl Handshake {
    /// Creates a builder with the minimum required fields.
    pub fn builder(node_id: impl Into<String>, version: impl Into<String>) -> HandshakeBuilder {
        HandshakeBuilder::new(node_id, version)
    }

    /// Ensures `timestamp_unix_sec` is set to the current wall-clock value if zero.
    pub fn ensure_timestamp(mut self) -> Result<Self, HandshakeError> {
        if self.timestamp_unix_sec == 0 {
            self.timestamp_unix_sec = current_unix_timestamp()?;
        }
        Ok(self)
    }

    /// Serializes the handshake as JSON.
    pub fn to_payload(&self) -> Result<Vec<u8>, HandshakeError> {
        Ok(serde_json::to_vec(self)?)
    }

    /// Serializes the handshake as a length-prefixed frame.
    pub fn to_frame(&self) -> Result<Vec<u8>, HandshakeError> {
        let payload = self.to_payload()?;
        if payload.len() > DEFAULT_MAX_FRAME_LEN {
            return Err(HandshakeError::FrameTooLarge {
                actual: payload.len(),
                max: DEFAULT_MAX_FRAME_LEN,
            });
        }

        let mut frame = Vec::with_capacity(CONTROL_FRAME_HEADER_LEN + payload.len());
        frame.extend_from_slice(&(payload.len() as u32).to_be_bytes());
        frame.extend_from_slice(&payload);
        Ok(frame)
    }

    /// Reads a handshake from an incoming control frame.
    pub fn from_payload(bytes: &[u8]) -> Result<Self, HandshakeError> {
        let handshake: Handshake = serde_json::from_slice(bytes)?;
        handshake.validate()
    }

    /// Validates required fields and normalizes features.
    pub fn validate(mut self) -> Result<Self, HandshakeError> {
        if self.node_id.trim().is_empty() {
            return Err(HandshakeError::MissingField("node_id"));
        }
        if self.version.trim().is_empty() {
            return Err(HandshakeError::MissingField("version"));
        }
        if self.token.trim().is_empty() {
            return Err(HandshakeError::MissingField("token"));
        }

        self.supported_features = normalize_features(self.supported_features);
        if self.timestamp_unix_sec == 0 {
            self = self.ensure_timestamp()?;
        }
        Ok(self)
    }
}

/// Builder providing a fluent API for constructing [`Handshake`] instances.
#[derive(Debug, Default)]
pub struct HandshakeBuilder {
    node_id: String,
    token: Option<String>,
    version: String,
    features: Vec<String>,
    metadata: BTreeMap<String, String>,
    timestamp: Option<i64>,
}

impl HandshakeBuilder {
    fn new(node_id: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            node_id: node_id.into(),
            version: version.into(),
            ..Default::default()
        }
    }

    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    pub fn add_feature(mut self, feature: impl Into<String>) -> Self {
        self.features.push(feature.into());
        self
    }

    pub fn features<I, S>(mut self, features: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.features.extend(features.into_iter().map(Into::into));
        self
    }

    pub fn metadata_entry(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn metadata(mut self, map: BTreeMap<String, String>) -> Self {
        self.metadata.extend(map);
        self
    }

    pub fn timestamp(mut self, ts: i64) -> Self {
        self.timestamp = Some(ts);
        self
    }

    pub fn build(self) -> Result<Handshake, HandshakeError> {
        let token = self
            .token
            .ok_or(HandshakeError::MissingField("token"))?
            .trim()
            .to_owned();

        let handshake = Handshake {
            node_id: self.node_id.trim().to_owned(),
            token,
            version: self.version.trim().to_owned(),
            supported_features: normalize_features(self.features),
            metadata: self.metadata,
            timestamp_unix_sec: self.timestamp.unwrap_or_default(),
        };

        handshake.validate()
    }
}

/// Writes a length-prefixed frame to the provided async writer.
pub async fn write_frame<W>(writer: &mut W, payload: &[u8]) -> Result<(), HandshakeError>
where
    W: AsyncWrite + Unpin,
{
    if payload.len() > DEFAULT_MAX_FRAME_LEN {
        return Err(HandshakeError::FrameTooLarge {
            actual: payload.len(),
            max: DEFAULT_MAX_FRAME_LEN,
        });
    }

    writer
        .write_u32(payload.len() as u32)
        .await
        .map_err(HandshakeError::Io)?;
    writer
        .write_all(payload)
        .await
        .map_err(HandshakeError::Io)?;
    writer.flush().await.map_err(HandshakeError::Io)?;
    Ok(())
}

/// Reads a length-prefixed frame from the provided async reader.
pub async fn read_frame<R>(reader: &mut R, max_len: usize) -> Result<Vec<u8>, HandshakeError>
where
    R: AsyncRead + Unpin,
{
    let len = reader.read_u32().await.map_err(HandshakeError::Io)? as usize;
    if len == 0 || len > max_len {
        return Err(HandshakeError::FrameTooLarge {
            actual: len,
            max: max_len,
        });
    }

    let mut buf = vec![0u8; len];
    reader
        .read_exact(&mut buf)
        .await
        .map_err(HandshakeError::Io)?;
    Ok(buf)
}

/// Serializes and writes the handshake to the control stream.
pub async fn send_handshake<W>(writer: &mut W, handshake: &Handshake) -> Result<(), HandshakeError>
where
    W: AsyncWrite + Unpin,
{
    let payload = handshake.to_payload()?;
    write_frame(writer, &payload).await
}

/// Reads, parses, and validates a handshake frame from the control stream.
pub async fn recv_handshake<R>(reader: &mut R) -> Result<Handshake, HandshakeError>
where
    R: AsyncRead + Unpin,
{
    let payload = read_frame(reader, DEFAULT_MAX_FRAME_LEN).await?;
    Handshake::from_payload(&payload)
}

fn normalize_features(features: impl Into<Vec<String>>) -> Vec<String> {
    let mut dedup = BTreeMap::new();
    for feature in features.into() {
        let normalized = feature.trim().to_lowercase();
        if !normalized.is_empty() {
            dedup.entry(normalized).or_insert(true);
        }
    }
    dedup.into_keys().collect()
}

fn current_unix_timestamp() -> Result<i64, HandshakeError> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_populates_defaults() {
        let handshake = Handshake::builder("node-123", "1.0.0")
            .token("secret")
            .add_feature("ReverseRPC")
            .metadata_entry("region", "us-east-1")
            .timestamp(1_700_000_000)
            .build()
            .expect("valid handshake");

        assert_eq!(handshake.node_id, "node-123");
        assert_eq!(handshake.token, "secret");
        assert_eq!(handshake.supported_features, vec!["reverserpc"]);
        assert_eq!(handshake.metadata.get("region").unwrap(), "us-east-1");
        assert_eq!(handshake.timestamp_unix_sec, 1_700_000_000);
    }

    #[tokio::test]
    async fn frame_round_trip() {
        let handshake = Handshake::builder("n1", "0.0.1")
            .token("tkn")
            .add_feature("A")
            .build()
            .unwrap();

        let payload = handshake.to_payload().unwrap();
        let frame = handshake.to_frame().unwrap();
        assert_eq!(frame.len(), payload.len() + CONTROL_FRAME_HEADER_LEN);

        let mut cursor = tokio::io::BufWriter::new(Vec::new());
        write_frame(&mut cursor, &payload).await.unwrap();
    }
}
