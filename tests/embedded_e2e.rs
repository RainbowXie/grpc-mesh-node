//! Full-path integration test for the `EmbeddedNode` facade.
//!
//! Spawns an in-process fake control plane (TLS + Yamux + handshake reader +
//! dial-back gRPC client) and drives a real `EmbeddedNode` through it:
//! generic Invoke success (including NUL bytes in the payload), unregistered
//! method semantics, business-error propagation, connection failure
//! tolerance, and clean shutdown with no residual connections.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Stdio;
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
            stop_grace: Duration::from_secs(2),
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
        // The driver signals drain readers before it exits: polling a yamux
        // stream after its connection driver is gone can spin inside yamux.
        let (driver_done_tx, mut driver_done_rx) = tokio::sync::watch::channel(false);
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
                let _ = driver_done_tx.send(true);
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
                    tokio::select! {
                        read = control.read(&mut buf) => match read {
                            Ok(0) | Err(_) => {
                                closed.store(true, std::sync::atomic::Ordering::SeqCst);
                                break;
                            }
                            Ok(n) => {
                                // Each heartbeat is a length-prefixed frame; a
                                // read batch may cover more than one.
                                if n > 0 {
                                    counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                                }
                            }
                        },
                        // The driver only exits when the connection itself
                        // ended — the same EOF proof the read path yields.
                        _ = driver_done_rx.changed() => {
                            closed.store(true, std::sync::atomic::Ordering::SeqCst);
                            break;
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
async fn handshake_advertises_normalized_features_and_verbatim_metadata() {
    let server = FakeMeshServer::start().await;
    let mut config = server.node_config("handshake-fields");
    config.features = vec![" DG ".into(), "dg".into(), "Report".into()];
    config.metadata = [
        ("capability".to_owned(), "dg".to_owned()),
        ("padded".to_owned(), "  text  ".to_owned()),
        ("empty".to_owned(), String::new()),
    ]
    .into_iter()
    .collect();
    let node = EmbeddedNode::new(config);

    // Register/replace/unregister before start: only the final live set may
    // be reported.
    node.register_method("calculator.v1.Calculator/Add", Arc::new(|p| Ok(p)))
        .expect("register Add");
    node.register_method("calculator.v1.Calculator/Health", Arc::new(|p| Ok(p)))
        .expect("register Health");
    node.unregister_method("calculator.v1.Calculator/Health")
        .expect("unregister Health");
    node.register_method("b.only", Arc::new(|p| Ok(p))).expect("register b.only");

    node.start().expect("node starts");
    let handshake = server.wait_handshake().await;

    // Features keep the existing Rust normalization: trim, lowercase, dedupe.
    assert_eq!(handshake.supported_features, vec!["dg".to_owned(), "report".to_owned()]);
    // Metadata values reach the wire character-for-character.
    assert_eq!(handshake.metadata.get("capability").map(String::as_str), Some("dg"));
    assert_eq!(handshake.metadata.get("padded").map(String::as_str), Some("  text  "));
    assert_eq!(handshake.metadata.get("empty").map(String::as_str), Some(""));
    // platform/arch stay framework-generated, not host-supplied.
    assert!(handshake.metadata.get("platform").is_some_and(|v| !v.is_empty()));
    assert!(handshake.metadata.get("arch").is_some_and(|v| !v.is_empty()));
    // mesh.methods reflects the real registry, not a forged value.
    assert_eq!(
        handshake.metadata.get("mesh.methods").map(String::as_str),
        Some("b.only,calculator.v1.Calculator/Add")
    );

    let (tx, rx) = tokio::sync::oneshot::channel();
    server.cmd_tx.send(ServerCmd::DropDialer { reply: tx }).unwrap();
    rx.await.expect("dialer dropped");
    tokio::time::sleep(Duration::from_millis(300)).await;
    node.stop(Duration::from_secs(5)).expect("clean stop");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn empty_registry_handshake_completes_with_empty_method_list() {
    let server = FakeMeshServer::start().await;
    let node = EmbeddedNode::new(server.node_config("empty-registry"));

    node.start().expect("node starts with no methods");
    let handshake = server.wait_handshake().await;
    assert_eq!(handshake.metadata.get("mesh.methods").map(String::as_str), Some(""));

    let (tx, rx) = tokio::sync::oneshot::channel();
    server.cmd_tx.send(ServerCmd::DropDialer { reply: tx }).unwrap();
    rx.await.expect("dialer dropped");
    tokio::time::sleep(Duration::from_millis(300)).await;
    node.stop(Duration::from_secs(5)).expect("clean stop");
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
        stop_grace: Duration::from_secs(2),
    };

    let node = EmbeddedNode::new(cfg);
    node.start().expect("start succeeds while server is unreachable");

    // The supervisor keeps retrying without dying.
    tokio::time::sleep(Duration::from_millis(600)).await;
    assert_eq!(node.state(), NodeLifecycle::Running);

    node.stop(Duration::from_secs(5)).expect("stop while never connected");
    assert!(node.supervisor_finished());
}

/// Full Java → JNI → Rust path over the real control protocol: a separate JVM
/// drives the public binding against the in-process TLS/yamux server, and this
/// test asserts the captured handshake, a live invoke, reserved-key rejection
/// and a clean close. It reuses [`FakeMeshServer`] rather than a capture-only
/// probe, so the close check is a real lifecycle assertion.
///
/// Prerequisites (fail loudly rather than skip — a missing prerequisite must
/// never silently drop the only Java-side evidence):
/// 1. Must run from inside the parent `grpc-mesh` checkout, where the binding
///    lives at `<parent>/bindings/java` (a sibling of this node submodule).
///    `java_bindings_dir()` resolves that path from `CARGO_MANIFEST_DIR`'s
///    parent, so a standalone node-repo checkout without the parent fails fast
///    in `collect_java_sources` instead of pretending to pass.
/// 2. A JDK 17 `javac` and `java` on `PATH` (the driver is compiled with
///    `javac --release 17` and run in a child JVM).
/// 3. A prebuilt host cdylib:
///    `cargo build -p grpc-mesh-node-ffi --target x86_64-unknown-linux-gnu`.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_binding_jni_handshake_over_real_control_protocol() {
    let server = FakeMeshServer::start().await;
    let scratch = std::env::temp_dir().join(format!(
        "grpc-mesh-java-e2e-{}-{}",
        std::process::id(),
        server.port
    ));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).expect("scratch dir");
    let classes = compile_java_driver(&scratch);
    let ca_file = scratch.join("ca.pem");
    std::fs::write(&ca_file, &server.ca_cert_pem).expect("write ca");
    let cdylib = find_host_cdylib();

    let log_path = scratch.join("java.log");
    let log = std::fs::File::create(&log_path).expect("java log");
    let mut child = std::process::Command::new("java")
        .arg("-cp")
        .arg(&classes)
        .arg(format!("-Dio.mesh.node.lib={}", cdylib.display()))
        .arg(format!("-De2e.dir={}", scratch.display()))
        .arg(format!("-De2e.caFile={}", ca_file.display()))
        .arg("io.mesh.node.e2e.JavaHandshakeDriver")
        .arg(format!("127.0.0.1:{}", server.port))
        .arg("java-e2e-node")
        .arg("test-token")
        .stdin(Stdio::piped())
        .stdout(Stdio::from(log.try_clone().expect("clone log")))
        .stderr(Stdio::from(log))
        .spawn()
        .expect("spawn java driver");

    let handshake = match tokio::time::timeout(Duration::from_secs(60), server.wait_handshake()).await {
        Ok(handshake) => handshake,
        Err(_) => {
            let _ = child.kill();
            panic!("java node did not complete the handshake; log:\n{}", read_log(&log_path));
        }
    };

    assert_eq!(handshake.node_id, "java-e2e-node");
    assert_eq!(
        handshake.supported_features,
        vec!["dg".to_owned(), "report".to_owned()]
    );
    assert_eq!(handshake.metadata.get("capability").map(String::as_str), Some("dg"));
    assert_eq!(handshake.metadata.get("padded").map(String::as_str), Some("  text  "));
    assert_eq!(handshake.metadata.get("empty").map(String::as_str), Some(""));
    assert_eq!(
        handshake.metadata.get("unicode").map(String::as_str),
        Some("a\u{00A0}b\u{2007}c")
    );
    // platform/arch stay framework-generated, not host-supplied.
    assert!(handshake.metadata.get("platform").is_some_and(|v| !v.is_empty()));
    assert!(handshake.metadata.get("arch").is_some_and(|v| !v.is_empty()));
    // register Health, register Add, unregister Health -> only Add remains.
    assert_eq!(
        handshake.metadata.get("mesh.methods").map(String::as_str),
        Some("calculator.v1.Calculator/Add")
    );

    // The Java handler actually serves an invoke over the tunnel.
    let resp = server.invoke("calculator.v1.Calculator/Add", b"ping".to_vec()).await;
    assert!(resp.success, "invoke should succeed: {:?}", resp.error);
    assert_eq!(resp.result, b"ping");

    // The reserved key was rejected by the Java Builder, so no second
    // connection was ever attempted.
    let reserved = std::fs::read_to_string(scratch.join("reserved.txt")).unwrap_or_default();
    assert_eq!(reserved, "REJECTED", "Java Builder must reject mesh.methods");

    // Close through the public close() -> bounded native stop.
    {
        let stdin = child.stdin.as_mut().expect("child stdin");
        stdin.write_all(b"stop\n").expect("signal stop");
        stdin.flush().expect("flush stop");
    }
    let status = tokio::task::spawn_blocking(move || child.wait())
        .await
        .expect("join child wait")
        .expect("wait java");
    assert!(
        status.success(),
        "java driver exit={status:?}; log:\n{}",
        read_log(&log_path)
    );

    // The control session is gone after the node stopped.
    let (tx, rx) = tokio::sync::oneshot::channel();
    server.cmd_tx.send(ServerCmd::DropDialer { reply: tx }).unwrap();
    rx.await.expect("dialer dropped");
    tokio::time::sleep(Duration::from_millis(400)).await;
    let (tx, rx) = tokio::sync::oneshot::channel();
    server.cmd_tx.send(ServerCmd::ControlClosed { reply: tx }).unwrap();
    assert!(
        rx.await.expect("closed probe"),
        "control session should close after stop"
    );

    let _ = std::fs::remove_dir_all(&scratch);
}

fn read_log(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|err| format!("<cannot read log: {err}>"))
}

fn java_bindings_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace parent")
        .join("bindings")
        .join("java")
}

fn collect_java_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("read java sources") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            collect_java_sources(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "java") {
            out.push(path);
        }
    }
}

fn compile_java_driver(scratch: &Path) -> PathBuf {
    let bindings = java_bindings_dir();
    let mut sources = Vec::new();
    collect_java_sources(
        &bindings.join("node-core").join("src").join("main").join("java"),
        &mut sources,
    );
    sources.push(bindings.join("e2e").join("JavaHandshakeDriver.java"));
    let classes = scratch.join("classes");
    std::fs::create_dir_all(&classes).expect("classes dir");
    let status = std::process::Command::new("javac")
        .arg("--release")
        .arg("17")
        .arg("-d")
        .arg(&classes)
        .args(&sources)
        .status()
        .expect("run javac (JDK 17 required for the host e2e harness)");
    assert!(
        status.success(),
        "javac failed compiling the Java binding + driver"
    );
    classes
}

fn find_host_cdylib() -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let candidates = [
        root.join("target/x86_64-unknown-linux-gnu/debug/libgrpc_mesh_node.so"),
        root.join("target/debug/libgrpc_mesh_node.so"),
    ];
    for candidate in candidates {
        if candidate.exists() {
            return candidate;
        }
    }
    panic!(
        "host cdylib not found; build it first with `cargo build -p grpc-mesh-node-ffi \
         --target x86_64-unknown-linux-gnu`"
    );
}
