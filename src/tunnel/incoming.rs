use std::io;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use futures::{
    Stream,
    io::{AsyncRead as FuturesAsyncRead, AsyncWrite as FuturesAsyncWrite},
};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio_util::compat::{Compat, FuturesAsyncReadCompatExt};
use tonic::transport::server::Connected;
use yamux::{self, Connection, Stream as YamuxStream};

use super::handshake::Handshake;

/// Wrapper to convert yamux Connection into a Stream
struct YamuxConnectionStream<T> {
    connection: Connection<T>,
}

impl<T> Stream for YamuxConnectionStream<T>
where
    T: FuturesAsyncRead + FuturesAsyncWrite + Unpin,
{
    type Item = Result<YamuxStream, yamux::ConnectionError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.connection.poll_next_inbound(cx) {
            Poll::Ready(Some(result)) => Poll::Ready(Some(result)),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Adapts a Yamux [`Connection`] into a stream of Tokio-compatible transports
/// while retaining the session handshake metadata.
pub struct YamuxIncoming {
    handshake: Arc<Handshake>,
    inner: Pin<Box<dyn Stream<Item = Result<YamuxStream, yamux::ConnectionError>> + Send>>,
}

impl YamuxIncoming {
    /// Consumes the provided Yamux connection and begins yielding transports for tonic.
    pub fn new<T>(connection: Connection<T>, handshake: Arc<Handshake>) -> Self
    where
        T: FuturesAsyncRead + FuturesAsyncWrite + Send + Unpin + 'static,
    {
        let inner = Box::pin(YamuxConnectionStream { connection });
        Self { handshake, inner }
    }

    /// Returns the handshake metadata associated with this incoming stream.
    pub fn handshake(&self) -> &Handshake {
        &self.handshake
    }
}

/// Tokio-compatible duplex transport backed by a single Yamux stream.
pub struct YamuxTransport {
    inner: Compat<YamuxStream>,
    handshake: Arc<Handshake>,
}

impl YamuxTransport {
    fn new(stream: YamuxStream, handshake: Arc<Handshake>) -> Self {
        Self {
            inner: stream.compat(),
            handshake,
        }
    }

    /// Exposes the handshake metadata for observability layers.
    pub fn handshake(&self) -> &Handshake {
        &self.handshake
    }
}

#[derive(Clone, Debug)]
pub struct HandshakeConnectInfo {
    handshake: Arc<Handshake>,
}

impl HandshakeConnectInfo {
    /// Returns the control-plane handshake metadata associated with this connection.
    pub fn handshake(&self) -> &Handshake {
        &self.handshake
    }
}

impl Connected for YamuxTransport {
    type ConnectInfo = HandshakeConnectInfo;

    fn connect_info(&self) -> Self::ConnectInfo {
        HandshakeConnectInfo {
            handshake: Arc::clone(&self.handshake),
        }
    }
}

impl AsyncRead for YamuxTransport {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl AsyncWrite for YamuxTransport {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        data: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, data)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

impl Stream for YamuxIncoming {
    type Item = io::Result<YamuxTransport>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        match this.inner.as_mut().poll_next(cx) {
            Poll::Ready(Some(Ok(stream))) => {
                let transport = YamuxTransport::new(stream, Arc::clone(&this.handshake));
                Poll::Ready(Some(Ok(transport)))
            }
            Poll::Ready(Some(Err(err))) => {
                let io_err = io::Error::new(io::ErrorKind::Other, err);
                Poll::Ready(Some(Err(io_err)))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
