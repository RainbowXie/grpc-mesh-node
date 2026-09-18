//! Full-path integration test for the `EmbeddedNode` facade.
//!
//! Spawns an in-process fake control plane (TLS + Yamux + handshake reader +
//! dial-back gRPC client) and drives a real `EmbeddedNode` through it:
//! generic Invoke success (including NUL bytes in the payload), unregistered
//! method semantics, business-error propagation, connection failure
//! tolerance, and clean shutdown with no residual connections.

use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use grpc_mesh::embedded::{EmbeddedNode, EmbeddedNodeConfig, NodeLifecycle};
use grpc_mesh::rpc::proto::grpc_mesh::rpc::v1::{
    InvokeRequest, invoke_plane_service_client::InvokePlaneServiceClient,
};
use grpc_mesh::tunnel::handshake::{Handshake, recv_handshake};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use tokio_rustls::rustls::pki_types::pem::PemObject;
use tokio_util::compat::{FuturesAsyncReadCompatExt, TokioAsyncReadCompatExt};
use tonic::transport::Channel;
use yamux::{Config as YamuxConfig, Connection, Mode, Stream as YamuxStream};

enum ServerCmd {
    Invoke {
        method: String,
        payload: Vec<u8>,
        reply: tokio::sync::oneshot::Sender<Result<grpc_mesh::rpc::proto::grpc_mesh::rpc::v1::InvokeResponse, String>>,
    },
    HeartbeatCount {
        reply: tokio::sync::oneshot::Sender<usize>,
    },
    ControlClosed {
        reply: tokio::sync::oneshot::Sender<bool>,
    },
    DropDialer {
        reply: tokio::sync::oneshot::Sender<()>,
    },
}

struct FakeMeshServer {
    cmd_tx: tokio::sync::mpsc::UnboundedSender<ServerCmd>,
    handshake_rx: tokio::sync::Mutex<Option<tokio::sync::oneshot::Receiver<Handshake>>>,
    port: u16,
    ca_cert_pem: String,
}

impl FakeMeshServer {
    /// Generates a private CA plus a "localhost" server certificate and
    /// starts accepting mesh node tunnels on a random loopback port.
    async fn start() -> Self {
        // A self-signed leaf cannot serve as its own trust anchor (rcgen
        // marks it CA:FALSE), so build a real CA and sign the server cert.
        let ca_key = rcgen::KeyPair::generate().expect("generate CA key");
        let mut ca_params = rcgen::CertificateParams::new(Vec::<String>::new())
            .expect("CA certificate params");
        ca_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        let ca_cert = ca_params.self_signed(&ca_key).expect("self-signed CA");

        let server_key = rcgen::KeyPair::generate().expect("generate server key");
        let server_params = rcgen::CertificateParams::new(vec!["localhost".to_string()])
            .expect("server certificate params");
        let server_cert = server_params
            .signed_by(&server_key, &ca_cert, &ca_key)
            .expect("CA-signed server certificate");

        let ca_cert_pem = ca_cert.pem();
        let server_cert_pem = server_cert.pem();
        let server_key_pem = server_key.serialize_pem();

        let certs = vec![tokio_rustls::rustls::pki_types::CertificateDer::from_pem_slice(
            server_cert_pem.as_bytes(),
        )
        .expect("parse cert")];
        let key = tokio_rustls::rustls::pki_types::PrivateKeyDer::from_pem_slice(
            server_key_pem.as_bytes(),
        )
        .expect("parse key");
        let tls_cfg = tokio_rustls::rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .expect("server tls config");
        let acceptor = TlsAcceptor::from(Arc::new(tls_cfg));

        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let port = listener.local_addr().unwrap().port();

        let (handshake_tx, handshake_rx) = tokio::sync::oneshot::channel();
        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();

        tokio::spawn(server_task(listener, acceptor, handshake_tx, cmd_rx));

        Self {
            cmd_tx,
            handshake_rx: tokio::sync::Mutex::new(Some(handshake_rx)),
            port,
            ca_cert_pem,
        }
    }

    fn node_config(&self, node_id: &str) -> EmbeddedNodeConfig {
        EmbeddedNodeConfig {
            server_addr: format!("127.0.0.1:{}", self.port),
            sni: Some("localhost".into()),
            ca_certs: vec![self.ca_cert_pem.as_bytes().to_vec()],
            insecure_skip_verify: false,
            node_id: node_id.into(),
            token: "test-token".into(),
            version: "0.1.0-test".into(),
            features: vec!["yamux-reverse-grpc".into()],
            metadata: Default::default(),
            connect_timeout: Duration::from_secs(5),
            max_backoff: Duration::from_millis(500),
            heartbeat_interval: Duration::from_millis(400),
            reconnect_base_delay: Duration::from_millis(50),
            reconnect_max_delay: Duration::from_millis(200),
            healthy_reset_after: Duration::from_secs(60),
        }
    }

    async fn wait_handshake(&self) -> Handshake {
        let rx = self.handshake_rx.lock().await.take().expect("handshake awaited once");
        rx.await.expect("handshake received")
    }

    async fn invoke(
        &self,
        method: &str,
        payload: Vec<u8>,
    ) -> grpc_mesh::rpc::proto::grpc_mesh::rpc::v1::InvokeResponse {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.cmd_tx
            .send(ServerCmd::Invoke { method: method.into(), payload, reply: tx })
            .expect("server alive");
        rx.await.expect("invoke reply").expect("invoke ok")
    }
}

async fn server_task(
    listener: TcpListener,
    acceptor: TlsAcceptor,
    handshake_tx: tokio::sync::oneshot::Sender<Handshake>,
    mut cmd_rx: tokio::sync::mpsc::UnboundedReceiver<ServerCmd>,
) {
    // The supervision loop may reconnect; each connection gets its own session.
    loop {
        let (tcp, _) = match listener.accept().await {
            Ok(x) => x,
            Err(_) => break,
        };
        let tls = match acceptor.accept(tcp).await {
            Ok(t) => t,
            Err(_) => continue,
        };
        let mut conn = Connection::new(tls.compat(), YamuxConfig::default(), Mode::Server);

        // First inbound stream is the control stream carrying the handshake.
        let control = match accept_inbound(&mut conn).await {
            Ok(s) => s,
            Err(_) => continue,
        };

        // yamux streams only make progress while the connection object is
        // being polled (tonic serves that role for real control planes via
        // YamuxIncoming). Keep a driver task on the connection for the whole
        // session and share it with the dial-back helper through a mutex.
        let conn = Arc::new(std::sync::Mutex::new(conn));
        {
            let conn = Arc::clone(&conn);
            tokio::spawn(async move {
                use futures::future::poll_fn;
                loop {
                    let item = poll_fn(|cx| {
                        let mut guard = conn.lock().expect("connection lock poisoned");
                        guard.poll_next_inbound(cx)
                    })
                    .await;
                    match item {
                        // Unexpected inbound streams are not part of this
                        // fake control plane; dropping them is fine.
                        Some(Ok(_)) => {}
                        Some(Err(_)) | None => break,
                    }
                }
            });
        }

        let mut control = control.compat();
        let handshake = match recv_handshake(&mut control).await {
            Ok(h) => h,
            Err(_) => continue,
        };
        let _ = handshake_tx.send(handshake);

        // Drain heartbeats, count them, and flag when the control stream
        // reaches EOF — the observable proof that stop closed the tunnel.
        let heartbeat_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let control_closed = Arc::new(std::sync::atomic::AtomicBool::new(false));
        {
            let counter = Arc::clone(&heartbeat_count);
            let closed = Arc::clone(&control_closed);
            tokio::spawn(async move {
                use tokio::io::AsyncReadExt;
                let mut buf = [0u8; 4096];
                loop {
                    match control.read(&mut buf).await {
                        Ok(0) | Err(_) => {
                            closed.store(true, std::sync::atomic::Ordering::SeqCst);
                            break;
                        }
                        Ok(n) => {
                            // Each heartbeat is a length-prefixed frame; a read
                            // batch may cover more than one, so count bytes>0.
                            if n > 0 {
                                counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                            }
                        }
                    }
                }
            });
        }

        // Serve commands for the lifetime of this session. The dial-back
        // client is session-scoped (like the Go gateway's pooled channel);
        // dropping it is what lets the node's graceful shutdown finish.
        let mut dialer: Option<InvokePlaneServiceClient<Channel>> = None;
        loop {
            let Some(cmd) = cmd_rx.recv().await else { return };
            match cmd {
                ServerCmd::Invoke { method, payload, reply } => {
                    if dialer.is_none() {
                        match build_dialer(&conn).await {
                            Ok(client) => dialer = Some(client),
                            Err(err) => {
                                let _ = reply.send(Err(err));
                                continue;
                            }
                        }
                    }
                    let result = match dialer.as_mut() {
                        Some(client) => do_invoke(client, &method, payload).await,
                        None => Err("dialer unavailable".to_string()),
                    };
                    let _ = reply.send(result);
                }
                ServerCmd::DropDialer { reply } => {
                    dialer = None;
                    let _ = reply.send(());
                }
                ServerCmd::HeartbeatCount { reply } => {
                    let _ = reply.send(heartbeat_count.load(std::sync::atomic::Ordering::SeqCst));
                }
                ServerCmd::ControlClosed { reply } => {
                    let _ = reply.send(control_closed.load(std::sync::atomic::Ordering::SeqCst));
                }
            }
        }
    }
}

fn noop_cx() -> Context<'static> {
    Context::from_waker(futures::task::noop_waker_ref())
}

/// Accepts one inbound (peer-initiated) yamux stream, mirroring how the
/// Go control plane reads the node's control stream.
async fn accept_inbound<T>(conn: &mut Connection<T>) -> yamux::Result<YamuxStream>
where
    T: futures::io::AsyncRead + futures::io::AsyncWrite + Unpin,
{
    loop {
        // The context (raw-waker backed) must not live across the await.
        let poll = {
            let mut cx = noop_cx();
            conn.poll_next_inbound(&mut cx)
        };
        match poll {
            Poll::Ready(Some(Ok(stream))) => return Ok(stream),
            Poll::Ready(Some(Err(err))) => return Err(err),
            Poll::Ready(None) => {
                return Err(yamux::ConnectionError::Closed);
            }
            Poll::Pending => tokio::task::yield_now().await,
        }
    }
}

/// Opens a server-initiated yamux stream (the dial-back path) and builds a
/// real tonic client over it.
async fn build_dialer<T>(
    conn: &Arc<std::sync::Mutex<Connection<T>>>,
) -> Result<InvokePlaneServiceClient<Channel>, String>
where
    T: futures::io::AsyncRead + futures::io::AsyncWrite + Unpin + Send + 'static,
{
    use futures::future::poll_fn;
    let stream = poll_fn(|cx| {
        let mut guard = conn.lock().expect("connection lock poisoned");
        guard.poll_new_outbound(cx)
    })
    .await
    .map_err(|e| format!("open stream: {e}"))?;

    // yamux streams are not cloneable, so the connector hands the single IO
    // over exactly once; a second dial (which this harness never issues)
    // errors.
    let io_cell = Arc::new(tokio::sync::Mutex::new(Some(hyper_util::rt::TokioIo::new(
        stream.compat(),
    ))));
    let channel = tonic::transport::Endpoint::from_static("http://node")
        .connect_with_connector(tower::service_fn(move |_: tonic::transport::Uri| {
            let cell = Arc::clone(&io_cell);
            async move {
                cell.lock()
                    .await
                    .take()
                    .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "io consumed"))
            }
        }))
        .await
        .map_err(|e| format!("tonic channel: {e}"))?;
    Ok(InvokePlaneServiceClient::new(channel))
}

async fn do_invoke(
    client: &mut InvokePlaneServiceClient<Channel>,
    method: &str,
    payload: Vec<u8>,
) -> Result<grpc_mesh::rpc::proto::grpc_mesh::rpc::v1::InvokeResponse, String> {
    let response = client
        .invoke(InvokeRequest {
            peer_id: "server".into(),
            method: method.into(),
            payload,
            correlation_id: "test-corr".into(),
            timeout_ms: 5_000,
        })
        .await
        .map_err(|status| format!("tonic status: {status}"))?;

    Ok(response.into_inner())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn embedded_node_serves_generic_invoke_over_real_tunnel() {
    let server = FakeMeshServer::start().await;
    let node = EmbeddedNode::new(server.node_config("e2e-node"));

    node.register_method("test.echo", Arc::new(|payload| Ok(payload)))
        .expect("register echo");
    node.register_method(
        "test.boom",
        Arc::new(|_| Err(grpc_mesh::RpcError::Internal("exploded".into()))),
    )
    .expect("register boom");

    node.start().expect("node starts");
    let handshake = server.wait_handshake().await;

    // Task 1.5: sorted method snapshot reported via mesh.methods.
    assert_eq!(
        handshake.metadata.get("mesh.methods").map(String::as_str),
        Some("test.boom,test.echo")
    );
    assert_eq!(handshake.node_id, "e2e-node");

    // Heartbeats flow while running.
    tokio::time::sleep(Duration::from_millis(1_200)).await;
    let (tx, rx) = tokio::sync::oneshot::channel();
    server.cmd_tx.send(ServerCmd::HeartbeatCount { reply: tx }).unwrap();
    let beats = rx.await.expect("count");
    assert!(beats >= 1, "expected at least one heartbeat frame, saw {beats}");

    // Successful invoke round-trips binary data including NUL bytes.
    let payload = b"hello\x00world\xff".to_vec();
    let resp = server.invoke("test.echo", payload.clone()).await;
    assert!(resp.success, "echo should succeed: {:?}", resp.error);
    assert_eq!(resp.result, payload, "binary payload must survive the round trip");

    // Unregistered method keeps the existing NOT_FOUND semantics.
    let resp = server.invoke("nope.missing", vec![]).await;
    assert!(!resp.success);
    let err = resp.error.expect("error detail");
    assert_eq!(err.code, "NOT_FOUND");

    // Business errors reach the caller verbatim.
    let resp = server.invoke("test.boom", vec![]).await;
    assert!(!resp.success);
    let err = resp.error.expect("error detail");
    assert_eq!(err.code, "INTERNAL");
    assert!(err.message.contains("exploded"), "message: {}", err.message);

    // Release the dial-back channel so the node's graceful shutdown is not
    // held open by our own client connection (mirrors the control plane
    // closing its pooled channel before expecting nodes to drain).
    let (tx, rx) = tokio::sync::oneshot::channel();
    server.cmd_tx.send(ServerCmd::DropDialer { reply: tx }).unwrap();
    rx.await.expect("dialer dropped");
    tokio::time::sleep(Duration::from_millis(300)).await;

    // Clean stop: bounded, and the tunnel actually goes away.
    node.stop(Duration::from_secs(5)).expect("clean stop");
    assert!(node.supervisor_finished(), "supervisor thread must exit after stop");

    // Give the OS a moment to propagate the close, then verify EOF.
    tokio::time::sleep(Duration::from_millis(500)).await;
    let (tx, rx) = tokio::sync::oneshot::channel();
    server.cmd_tx.send(ServerCmd::ControlClosed { reply: tx }).unwrap();
    let closed = rx.await.expect("closed probe");
    assert!(closed, "control session should be closed after stop");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn embedded_node_tolerates_unreachable_server() {
    let cfg = EmbeddedNodeConfig {
        server_addr: "127.0.0.1:1".into(),
        sni: None,
        ca_certs: Vec::new(),
        insecure_skip_verify: true,
        node_id: "offline-node".into(),
        token: "t".into(),
        version: "0.1.0-test".into(),
        features: vec![],
        metadata: Default::default(),
        connect_timeout: Duration::from_millis(300),
        max_backoff: Duration::from_millis(100),
        heartbeat_interval: Duration::from_secs(15),
        reconnect_base_delay: Duration::from_millis(50),
        reconnect_max_delay: Duration::from_millis(100),
        healthy_reset_after: Duration::from_secs(60),
    };

    let node = EmbeddedNode::new(cfg);
    node.start().expect("start succeeds while server is unreachable");

    // The supervisor keeps retrying without dying.
    tokio::time::sleep(Duration::from_millis(600)).await;
    assert_eq!(node.state(), NodeLifecycle::Running);

    node.stop(Duration::from_secs(5)).expect("stop while never connected");
    assert!(node.supervisor_finished());
}
