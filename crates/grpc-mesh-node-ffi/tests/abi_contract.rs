//! Rust contract tests for the Node C ABI.
//!
//! These call the exported ABI functions directly (the test links the same
//! crate that backs the cdylib) and cover the falsification scenarios from
//! the spec: invalid configs, state-machine violations, stale handles,
//! response-handle hygiene, thread-local errors, and — through a real
//! TLS+yamux fake control plane — host callback dispatch, error mapping,
//! panic conversion, concurrency, stop-during-callback and big payloads.

use std::ffi::{CStr, CString, c_void};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use grpc_mesh_node::{
    MESH_NODE_BUSINESS_CONFIG, MESH_NODE_BUSINESS_INTERNAL, MESH_NODE_BUSINESS_NOT_FOUND,
    MESH_NODE_BUSINESS_TRANSPORT, MESH_NODE_ERR_INVALID_ARGUMENT, MESH_NODE_ERR_INVALID_STATE,
    MESH_NODE_ERR_NOT_FOUND, MESH_NODE_ERR_SHUTDOWN_TIMEOUT, MESH_NODE_OK,
    MESH_NODE_STATE_INVALID, MeshNodeMethodCallback, MeshNodeRequest, mesh_node_abi_version,
    mesh_node_free, mesh_node_last_error, mesh_node_new, mesh_node_register_method,
    mesh_node_response_error, mesh_node_response_len, mesh_node_response_ok,
    mesh_node_response_read, mesh_node_response_release, mesh_node_state, mesh_node_stop,
    mesh_node_unregister_method,
};

fn last_error() -> String {
    unsafe { CStr::from_ptr(mesh_node_last_error()).to_string_lossy().into_owned() }
}

fn unreachable_config(node_id: &str) -> CString {
    CString::new(format!(
        r#"{{
            "server": {{ "address": "127.0.0.1:9" }},
            "node": {{ "id": "{node_id}", "token": "t" }},
            "connect": {{ "timeout_secs": 1 }},
            "reconnect": {{ "base_delay_ms": 50, "max_delay_ms": 100 }}
        }}"#
    ))
    .expect("valid CString")
}

// ---------------------------------------------------------------------------
// Unit-level contract
// ---------------------------------------------------------------------------

#[test]
fn abi_version_encoding() {
    let version = mesh_node_abi_version();
    assert_eq!(version >> 16, 1, "major version must be 1");
    assert_eq!(version & 0xFFFF, 0, "minor starts at 0");
}

#[test]
fn invalid_config_creation_fails_with_reasons() {
    assert_eq!(unsafe { mesh_node_new(std::ptr::null()) }, 0);
    assert!(last_error().contains("config_json"));

    let bad = CString::new("{ nope").unwrap();
    assert_eq!(unsafe { mesh_node_new(bad.as_ptr()) }, 0);
    assert!(last_error().to_lowercase().contains("json"));

    for (json, needle) in [
        (r#"{ "server": { "address": "127.0.0.1:9" }, "node": { "id": "x" } }"#, "token"),
        (r#"{ "server": { "address": "127.0.0.1:9" }, "node": { "token": "t" } }"#, "id"),
        (r#"{ "node": { "id": "x", "token": "t" } }"#, "address"),
    ] {
        let cfg = CString::new(json).unwrap();
        assert_eq!(unsafe { mesh_node_new(cfg.as_ptr()) }, 0);
        assert!(
            last_error().contains(needle),
            "expected {needle:?} in {:?}",
            last_error()
        );
    }

    let cfg = CString::new(
        r#"{ "server": { "address": "127.0.0.1:9", "tls": { "ca_cert": "garbage" } },
            "node": { "id": "x", "token": "t" } }"#,
    )
    .unwrap();
    assert_eq!(unsafe { mesh_node_new(cfg.as_ptr()) }, 0);
    assert!(last_error().to_lowercase().contains("ca"));
}

#[test]
fn registration_validation_and_state_freeze() {
    let cfg = unreachable_config("reg-validation");
    let node = unsafe { mesh_node_new(cfg.as_ptr()) };
    assert_ne!(node, 0);

    let echo: MeshNodeMethodCallback = echo_callback;
    let name = CString::new("test.echo").unwrap();

    assert_eq!(
        unsafe {
            mesh_node_register_method(
                999_999,
                name.as_ptr(),
                echo,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        },
        MESH_NODE_ERR_NOT_FOUND
    );
    assert_eq!(
        unsafe {
            mesh_node_register_method(
                node,
                std::ptr::null(),
                echo,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        },
        MESH_NODE_ERR_INVALID_ARGUMENT
    );
    let empty = CString::new("").unwrap();
    assert_eq!(
        unsafe {
            mesh_node_register_method(
                node,
                empty.as_ptr(),
                echo,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        },
        MESH_NODE_ERR_INVALID_ARGUMENT
    );
    let missing = CString::new("missing.method").unwrap();
    assert_eq!(
        unsafe { mesh_node_unregister_method(node, missing.as_ptr(), std::ptr::null_mut()) },
        MESH_NODE_ERR_NOT_FOUND
    );

    assert_eq!(
        unsafe {
            mesh_node_register_method(
                node,
                name.as_ptr(),
                echo,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        },
        MESH_NODE_OK
    );
    let started = unsafe { grpc_mesh_node::mesh_node_start(node) };
    if started != MESH_NODE_OK {
        panic!("start failed: {started}: {}", last_error());
    }
    assert_eq!(
        unsafe {
            mesh_node_register_method(
                node,
                name.as_ptr(),
                echo,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        },
        MESH_NODE_ERR_INVALID_STATE
    );
    assert_eq!(unsafe { grpc_mesh_node::mesh_node_start(node) }, MESH_NODE_ERR_INVALID_STATE);
    assert_eq!(unsafe { mesh_node_stop(node, 5000) }, MESH_NODE_OK);
    assert_eq!(unsafe { grpc_mesh_node::mesh_node_start(node) }, MESH_NODE_ERR_INVALID_STATE);
    assert_eq!(unsafe { mesh_node_stop(node, 1000) }, MESH_NODE_OK);
    unsafe { mesh_node_free(node) };
}

#[test]
fn stale_handles_never_alias_live_nodes() {
    let a = unsafe { mesh_node_new(unreachable_config("stale-a").as_ptr()) };
    let b = unsafe { mesh_node_new(unreachable_config("stale-b").as_ptr()) };
    assert_ne!(a, 0);
    assert_ne!(b, 0);
    assert_ne!(a, b);

    unsafe { mesh_node_free(a) };
    assert_eq!(unsafe { mesh_node_state(a) }, MESH_NODE_STATE_INVALID);

    let name = CString::new("x").unwrap();
    let echo: MeshNodeMethodCallback = echo_callback;
    assert_eq!(
        unsafe {
            mesh_node_register_method(
                a,
                name.as_ptr(),
                echo,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            )
        },
        MESH_NODE_ERR_NOT_FOUND
    );
    assert_eq!(unsafe { mesh_node_stop(a, 100) }, MESH_NODE_ERR_NOT_FOUND);
    unsafe { mesh_node_free(a) }; // double free: no-op

    assert_eq!(unsafe { mesh_node_state(b) }, 1 /* CREATED */);
    assert_eq!(unsafe { grpc_mesh_node::mesh_node_start(b) }, MESH_NODE_OK);
    assert_eq!(unsafe { mesh_node_stop(b, 5000) }, MESH_NODE_OK);
    unsafe { mesh_node_free(b) };
}

#[test]
fn two_nodes_are_isolated() {
    let a = unsafe { mesh_node_new(unreachable_config("iso-a").as_ptr()) };
    let b = unsafe { mesh_node_new(unreachable_config("iso-b").as_ptr()) };
    let echo: MeshNodeMethodCallback = echo_callback;
    let name_a = CString::new("a.only").unwrap();
    let name_b = CString::new("b.only").unwrap();
    unsafe {
        assert_eq!(
            mesh_node_register_method(a, name_a.as_ptr(), echo, std::ptr::null_mut(), std::ptr::null_mut()),
            MESH_NODE_OK
        );
        assert_eq!(
            mesh_node_register_method(b, name_b.as_ptr(), echo, std::ptr::null_mut(), std::ptr::null_mut()),
            MESH_NODE_OK
        );
    }
    assert_eq!(unsafe { grpc_mesh_node::mesh_node_start(a) }, MESH_NODE_OK);
    assert_eq!(unsafe { grpc_mesh_node::mesh_node_start(b) }, MESH_NODE_OK);
    // Stopping and freeing A must not disturb B.
    assert_eq!(unsafe { mesh_node_stop(a, 5000) }, MESH_NODE_OK);
    unsafe { mesh_node_free(a) };
    assert_eq!(unsafe { mesh_node_state(b) }, 3 /* RUNNING */);
    assert_eq!(unsafe { mesh_node_stop(b, 5000) }, MESH_NODE_OK);
    unsafe { mesh_node_free(b) };
}

#[test]
fn replacement_registration_reports_previous_user_data() {
    let node = unsafe { mesh_node_new(unreachable_config("replacement").as_ptr()) };
    let name = CString::new("dup.method").unwrap();
    let echo: MeshNodeMethodCallback = echo_callback;

    let marker_one = Box::into_raw(Box::new(1usize)) as *mut c_void;
    let marker_two = Box::into_raw(Box::new(2usize)) as *mut c_void;
    let mut replaced: *mut c_void = std::ptr::null_mut();

    unsafe {
        assert_eq!(
            mesh_node_register_method(node, name.as_ptr(), echo, marker_one, &mut replaced),
            MESH_NODE_OK
        );
        assert!(replaced.is_null(), "fresh registration has no predecessor");

        assert_eq!(
            mesh_node_register_method(node, name.as_ptr(), echo, marker_two, &mut replaced),
            MESH_NODE_OK
        );
        assert_eq!(replaced, marker_one, "replacement returns previous user_data");

        let mut removed: *mut c_void = std::ptr::null_mut();
        assert_eq!(
            mesh_node_unregister_method(node, name.as_ptr(), &mut removed),
            MESH_NODE_OK
        );
        assert_eq!(removed, marker_two);

        drop(Box::from_raw(marker_one as *mut usize));
        drop(Box::from_raw(marker_two as *mut usize));
        mesh_node_free(node);
    }
}

#[test]
fn response_handle_hygiene() {
    let payload: &[u8] = b"a\0b\0\0c";
    let r1 = unsafe { mesh_node_response_ok(payload.as_ptr(), payload.len()) };
    assert_ne!(r1, 0);
    assert_eq!(unsafe { mesh_node_response_len(r1) }, payload.len());
    let mut buf = [0u8; 32];
    assert_eq!(unsafe { mesh_node_response_read(r1, buf.as_mut_ptr(), buf.len()) }, payload.len());
    assert_eq!(&buf[..payload.len()], payload);

    let msg = CString::new("boom").unwrap();
    let err = unsafe { mesh_node_response_error(MESH_NODE_BUSINESS_INTERNAL, msg.as_ptr()) };
    assert_ne!(err, 0);
    assert_eq!(unsafe { mesh_node_response_len(err) }, 0);
    assert_eq!(unsafe { mesh_node_response_read(err, buf.as_mut_ptr(), buf.len()) }, 0);
    assert_eq!(unsafe { mesh_node_response_error(9999, msg.as_ptr()) }, 0);
    assert_eq!(
        unsafe { mesh_node_response_error(MESH_NODE_BUSINESS_INTERNAL, std::ptr::null()) },
        0
    );
    assert_eq!(unsafe { mesh_node_response_ok(std::ptr::null(), 5) }, 0);

    unsafe { mesh_node_response_release(r1) };
    unsafe { mesh_node_response_release(r1) };
    assert_eq!(unsafe { mesh_node_response_read(r1, buf.as_mut_ptr(), buf.len()) }, 0);

    let r2 = unsafe { mesh_node_response_ok(b"intact".as_ptr(), 6) };
    assert_ne!(r2, r1);
    unsafe { mesh_node_response_release(r1) }; // stale release after new alloc
    assert_eq!(unsafe { mesh_node_response_len(r2) }, 6);
    assert_eq!(unsafe { mesh_node_response_read(r2, buf.as_mut_ptr(), buf.len()) }, 6);
    assert_eq!(&buf[..6], b"intact");
    unsafe { mesh_node_response_release(r2) };
}

#[test]
fn last_error_is_thread_local() {
    let node = unsafe { mesh_node_new(unreachable_config("thread-err").as_ptr()) };
    let mut handles = Vec::new();
    for thread_index in 0..4u32 {
        handles.push(std::thread::spawn(move || {
            let method = CString::new(format!("err.thread.{thread_index}")).unwrap();
            assert_eq!(
                unsafe { mesh_node_unregister_method(node, method.as_ptr(), std::ptr::null_mut()) },
                MESH_NODE_ERR_NOT_FOUND
            );
            let message = last_error();
            assert!(
                message.contains(&format!("err.thread.{thread_index}")),
                "thread {thread_index} saw a foreign error: {message:?}"
            );
        }));
    }
    for handle in handles {
        handle.join().expect("thread must not panic");
    }
    unsafe { mesh_node_free(node) };
}

// ---------------------------------------------------------------------------
// Callbacks
// ---------------------------------------------------------------------------

/// Request-view capture shared with echo_callback. Callbacks receive a
/// borrowed pointer to the mutex inside the Arc; the Arc outlives the node
/// (dropped only after a successful stop).
pub struct Captured {
    pub method: String,
    pub correlation_id: String,
    pub timeout_ms: u32,
    pub payload: Vec<u8>,
}

type CaptureSlot = std::sync::Mutex<Option<Captured>>;

unsafe extern "C-unwind" fn echo_callback(
    request: *const MeshNodeRequest,
    user_data: *mut c_void,
) -> u64 {
    let request = unsafe { &*request };
    if user_data.is_null() {
        return unsafe { mesh_node_response_ok(request.payload, request.payload_len) };
    }
    let captured = unsafe { &*(user_data as *const CaptureSlot) };
    let method = unsafe { CStr::from_ptr(request.method) }.to_string_lossy().into_owned();
    let correlation =
        unsafe { CStr::from_ptr(request.correlation_id) }.to_string_lossy().into_owned();
    let payload =
        unsafe { std::slice::from_raw_parts(request.payload, request.payload_len) }.to_vec();
    *captured.lock().unwrap() = Some(Captured {
        method,
        correlation_id: correlation,
        timeout_ms: request.timeout_ms,
        payload,
    });
    unsafe { mesh_node_response_ok(request.payload, request.payload_len) }
}

unsafe extern "C-unwind" fn business_error_callback(
    _request: *const MeshNodeRequest,
    _user_data: *mut c_void,
) -> u64 {
    let msg = CString::new("business failure detail").unwrap();
    unsafe { mesh_node_response_error(MESH_NODE_BUSINESS_CONFIG, msg.as_ptr()) }
}

unsafe extern "C-unwind" fn transport_error_callback(
    _request: *const MeshNodeRequest,
    _user_data: *mut c_void,
) -> u64 {
    let msg = CString::new("downstream broke").unwrap();
    unsafe { mesh_node_response_error(MESH_NODE_BUSINESS_TRANSPORT, msg.as_ptr()) }
}

unsafe extern "C-unwind" fn not_found_error_callback(
    _request: *const MeshNodeRequest,
    _user_data: *mut c_void,
) -> u64 {
    let msg = CString::new("entity 42 missing").unwrap();
    unsafe { mesh_node_response_error(MESH_NODE_BUSINESS_NOT_FOUND, msg.as_ptr()) }
}

unsafe extern "C-unwind" fn panicking_callback(
    _request: *const MeshNodeRequest,
    _user_data: *mut c_void,
) -> u64 {
    panic!("host callback panic payload");
}

unsafe extern "C-unwind" fn null_handle_callback(
    _request: *const MeshNodeRequest,
    _user_data: *mut c_void,
) -> u64 {
    0
}

unsafe extern "C-unwind" fn stale_handle_callback(
    _request: *const MeshNodeRequest,
    _user_data: *mut c_void,
) -> u64 {
    let handle = unsafe { mesh_node_response_ok(b"already-gone".as_ptr(), 13) };
    unsafe { mesh_node_response_release(handle) };
    handle
}

struct BlockCtx {
    entered: AtomicBool,
    exited: AtomicBool,
    release: AtomicBool,
    calls: std::sync::atomic::AtomicUsize,
}

unsafe extern "C-unwind" fn blocking_callback(
    _request: *const MeshNodeRequest,
    user_data: *mut c_void,
) -> u64 {
    let ctx = unsafe { &*(user_data as *const BlockCtx) };
    ctx.calls.fetch_add(1, Ordering::SeqCst);
    ctx.entered.store(true, Ordering::SeqCst);
    while !ctx.release.load(Ordering::SeqCst) {
        std::thread::sleep(Duration::from_millis(10));
    }
    ctx.exited.store(true, Ordering::SeqCst);
    unsafe { mesh_node_response_ok(b"unblocked".as_ptr(), 10) }
}

// ---------------------------------------------------------------------------
// Network suite: real TLS + yamux fake control plane
// ---------------------------------------------------------------------------

mod fake_server {
    use std::sync::Arc;
    use std::task::{Context, Poll};

    use grpc_mesh::tunnel::handshake::{Handshake, recv_handshake};
    use tokio::net::TcpListener;
    use tokio_rustls::TlsAcceptor;
    use tokio_rustls::rustls::pki_types::pem::PemObject;
    use tokio_util::compat::{FuturesAsyncReadCompatExt, TokioAsyncReadCompatExt};
    use tonic::transport::Channel;

    type InvokeClient = grpc_mesh::rpc::proto::grpc_mesh::rpc::v1::invoke_plane_service_client::InvokePlaneServiceClient<Channel>;
    type InvokeResponse = grpc_mesh::rpc::proto::grpc_mesh::rpc::v1::InvokeResponse;

    pub enum ServerCmd {
        Invoke {
            method: String,
            payload: Vec<u8>,
            reply: tokio::sync::oneshot::Sender<Result<InvokeResponse, String>>,
        },
        DropDialer {
            reply: tokio::sync::oneshot::Sender<()>,
        },
    }

    pub struct FakeMeshServer {
        pub cmd_tx: tokio::sync::mpsc::UnboundedSender<ServerCmd>,
        handshake_rx: tokio::sync::Mutex<Option<tokio::sync::oneshot::Receiver<Handshake>>>,
        port: u16,
        ca_cert_pem: String,
    }

    impl FakeMeshServer {
        pub async fn start() -> Self {
            let ca_key = rcgen::KeyPair::generate().expect("CA key");
            let mut ca_params =
                rcgen::CertificateParams::new(Vec::<String>::new()).expect("CA params");
            ca_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
            let ca_cert = ca_params.self_signed(&ca_key).expect("CA cert");

            let server_key = rcgen::KeyPair::generate().expect("server key");
            let server_params = rcgen::CertificateParams::new(vec!["localhost".to_string()])
                .expect("server params");
            let server_cert = server_params
                .signed_by(&server_key, &ca_cert, &ca_key)
                .expect("server cert");

            let ca_cert_pem = ca_cert.pem();
            let certs = vec![tokio_rustls::rustls::pki_types::CertificateDer::from_pem_slice(
                server_cert.pem().as_bytes(),
            )
            .expect("cert")];
            let key = tokio_rustls::rustls::pki_types::PrivateKeyDer::from_pem_slice(
                server_key.serialize_pem().as_bytes(),
            )
            .expect("key");
            let tls = tokio_rustls::rustls::ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(certs, key)
                .expect("tls config");
            let acceptor = TlsAcceptor::from(Arc::new(tls));

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

        /// JSON config with the CA PEM embedded (newlines escaped for JSON).
        pub fn config_json(&self, node_id: &str) -> String {
            let ca = self.ca_cert_pem.replace('\n', "\\n");
            format!(
                r#"{{"server":{{"address":"127.0.0.1:{port}","tls":{{"server_name":"localhost","ca_cert":"{ca}"}}}},"node":{{"id":"{node_id}","token":"t"}},"heartbeat":{{"interval_secs":1}},"connect":{{"timeout_secs":5}},"reconnect":{{"base_delay_ms":50,"max_delay_ms":200}}}}"#,
                port = self.port,
                ca = ca,
                node_id = node_id,
            )
        }

        pub async fn wait_handshake(&self) -> Handshake {
            let rx = self.handshake_rx.lock().await.take().expect("once");
            rx.await.expect("handshake")
        }

        pub async fn invoke(&self, method: &str, payload: Vec<u8>) -> InvokeResponse {
            self.try_invoke(method, payload).await.expect("invoke ok")
        }

        /// Like [`Self::invoke`] but surfaces transport failures instead of
        /// panicking, for probes where a refusal is an acceptable outcome.
        pub async fn try_invoke(
            &self,
            method: &str,
            payload: Vec<u8>,
        ) -> Result<InvokeResponse, String> {
            let (tx, rx) = tokio::sync::oneshot::channel();
            self.cmd_tx
                .send(ServerCmd::Invoke { method: method.into(), payload, reply: tx })
                .map_err(|e| format!("server gone: {e}"))?;
            rx.await.map_err(|e| format!("reply dropped: {e}"))?
        }

        pub async fn drop_dialer(&self) {
            let (tx, rx) = tokio::sync::oneshot::channel();
            self.cmd_tx.send(ServerCmd::DropDialer { reply: tx }).expect("server alive");
            rx.await.expect("drop");
        }
    }

    async fn server_task(
        listener: TcpListener,
        acceptor: TlsAcceptor,
        handshake_tx: tokio::sync::oneshot::Sender<Handshake>,
        mut cmd_rx: tokio::sync::mpsc::UnboundedReceiver<ServerCmd>,
    ) {
        use futures::future::poll_fn;
        loop {
            let (tcp, _) = match listener.accept().await {
                Ok(x) => x,
                Err(_) => break,
            };
            let tls = match acceptor.accept(tcp).await {
                Ok(t) => t,
                Err(_) => continue,
            };
            let mut conn =
                yamux::Connection::new(tls.compat(), yamux::Config::default(), yamux::Mode::Server);

            let control = loop {
                let poll = {
                    let mut cx = Context::from_waker(futures::task::noop_waker_ref());
                    conn.poll_next_inbound(&mut cx)
                };
                match poll {
                    Poll::Ready(Some(Ok(stream))) => break Some(stream),
                    Poll::Ready(_) => break None,
                    Poll::Pending => tokio::task::yield_now().await,
                }
            };
            let control = match control {
                Some(stream) => stream,
                None => continue,
            };

            // yamux streams only progress while the connection is polled;
            // keep a driver task for the whole session. Polling a stream
            // after its connection driver exits can spin inside yamux, so
            // the driver signals drain readers before it returns.
            let conn = Arc::new(std::sync::Mutex::new(conn));
            let (driver_done_tx, mut driver_done_rx) = tokio::sync::watch::channel(false);
            {
                let conn = Arc::clone(&conn);
                tokio::spawn(async move {
                    loop {
                        let item =
                            poll_fn(|cx| conn.lock().expect("conn").poll_next_inbound(cx)).await;
                        match item {
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
            tokio::spawn(async move {
                use tokio::io::AsyncReadExt;
                let mut buf = [0u8; 4096];
                loop {
                    tokio::select! {
                        read = control.read(&mut buf) => {
                            if read.is_err() {
                                break;
                            }
                        }
                        _ = driver_done_rx.changed() => break,
                    }
                }
            });

            // The command loop must never block on one invoke: a parked
            // node callback would starve every later command (exactly the
            // situation the stop-during-callback test creates).
            let dialer: std::sync::Arc<tokio::sync::Mutex<Option<InvokeClient>>> =
                std::sync::Arc::new(tokio::sync::Mutex::new(None));
            loop {
                let Some(cmd) = cmd_rx.recv().await else {
                    return;
                };
                match cmd {
                    ServerCmd::Invoke { method, payload, reply } => {
                        let dialer = std::sync::Arc::clone(&dialer);
                        let conn = std::sync::Arc::clone(&conn);
                        tokio::spawn(async move {
                            // Clone the client and release the lock before
                            // awaiting the invoke: holding it across a parked
                            // callback would serialize everything again.
                            let mut client = {
                                let mut guard = dialer.lock().await;
                                if guard.is_none() {
                                    match build_dialer(&conn).await {
                                        Ok(client) => *guard = Some(client),
                                        Err(err) => {
                                            let _ = reply.send(Err(err));
                                            return;
                                        }
                                    }
                                }
                                guard.as_ref().expect("dialer built").clone()
                            };
                            let result = do_invoke(&mut client, &method, payload).await;
                            let _ = reply.send(result);
                        });
                    }
                    ServerCmd::DropDialer { reply } => {
                        // Drop the shared client. Invoke tasks keep their own
                        // clones (possibly attached to a dead connection
                        // after a shutdown); waiting for them would deadlock.
                        *dialer.lock().await = None;
                        let _ = reply.send(());
                    }
                }
            }
        }
    }

    async fn build_dialer<T>(
        conn: &Arc<std::sync::Mutex<yamux::Connection<T>>>,
    ) -> Result<InvokeClient, String>
    where
        T: futures::io::AsyncRead + futures::io::AsyncWrite + Unpin + Send + 'static,
    {
        use futures::future::poll_fn;
        let stream = poll_fn(|cx| conn.lock().expect("conn").poll_new_outbound(cx))
            .await
            .map_err(|e| format!("open stream: {e}"))?;
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
                        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "consumed"))
                }
            }))
            .await
            .map_err(|e| format!("channel: {e}"))?;
        Ok(InvokeClient::new(channel))
    }

    async fn do_invoke(
        client: &mut InvokeClient,
        method: &str,
        payload: Vec<u8>,
    ) -> Result<InvokeResponse, String> {
        let response = client
            .invoke(grpc_mesh::rpc::proto::grpc_mesh::rpc::v1::InvokeRequest {
                peer_id: "server".into(),
                method: method.into(),
                payload,
                correlation_id: "corr-42".into(),
                timeout_ms: 5_000,
            })
            .await
            .map_err(|status| format!("status: {status}"))?;
        Ok(response.into_inner())
    }
}

use fake_server::FakeMeshServer;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn full_abi_stack_over_real_tunnel() {
    let server = Arc::new(FakeMeshServer::start().await);
    let cfg = CString::new(server.config_json("abi-node")).unwrap();
    let node = unsafe { mesh_node_new(cfg.as_ptr()) };
    assert_ne!(node, 0);

    // Registration through the ABI. The echo capture slot is borrowed for
    // the node's lifetime (dropped after the successful stop below).
    let captured_slot: Arc<CaptureSlot> = Arc::new(std::sync::Mutex::new(None));
    unsafe {
        for (name, callback) in [
            ("z.echo", echo_callback as MeshNodeMethodCallback),
            ("a.config_error", business_error_callback),
            ("a.transport_error", transport_error_callback),
            ("a.not_found_error", not_found_error_callback),
            ("a.panic", panicking_callback),
            ("a.null_handle", null_handle_callback),
            ("a.stale_handle", stale_handle_callback),
        ] {
            let c_name = CString::new(name).unwrap();
            let user_data = if name == "z.echo" {
                Arc::as_ptr(&captured_slot) as *mut c_void
            } else {
                std::ptr::null_mut()
            };
            assert_eq!(
                mesh_node_register_method(
                    node,
                    c_name.as_ptr(),
                    callback,
                    user_data,
                    std::ptr::null_mut(),
                ),
                MESH_NODE_OK
            );
        }
    }

    assert_eq!(unsafe { grpc_mesh_node::mesh_node_start(node) }, MESH_NODE_OK);
    let handshake = server.wait_handshake().await;

    assert_eq!(
        handshake.metadata.get("mesh.methods").map(String::as_str),
        Some(
            "a.config_error,a.not_found_error,a.null_handle,a.panic,a.stale_handle,a.transport_error,z.echo"
        )
    );

    // Successful callback: binary-safe round trip, request view verbatim.
    let payload = b"bin\x00ary\xff".to_vec();
    let resp = server.invoke("z.echo", payload.clone()).await;
    assert!(resp.success, "{:?}", resp.error);
    assert_eq!(resp.result, payload);
    {
        let guard = captured_slot.lock().unwrap();
        let view = guard.as_ref().expect("callback captured request view");
        assert_eq!(view.method, "z.echo");
        assert_eq!(view.correlation_id, "corr-42");
        assert_eq!(view.timeout_ms, 5_000);
        assert_eq!(view.payload, payload);
    }
    let resp = server.invoke("a.config_error", vec![]).await;
    assert!(!resp.success);
    assert_eq!(resp.error.as_ref().unwrap().code, "CONFIG");
    assert!(resp.error.as_ref().unwrap().message.contains("business failure detail"));
    let resp = server.invoke("a.transport_error", vec![]).await;
    assert!(!resp.success);
    assert_eq!(resp.error.as_ref().unwrap().code, "TRANSPORT");
    let resp = server.invoke("a.not_found_error", vec![]).await;
    assert!(!resp.success);
    let err = resp.error.as_ref().unwrap();
    assert_eq!(err.code, "NOT_FOUND");
    assert!(
        err.message.contains("entity 42 missing"),
        "host message must survive mapping: {err:?}"
    );
    let resp = server.invoke("a.panic", vec![]).await;
    assert!(!resp.success);
    let err = resp.error.as_ref().unwrap();
    assert_eq!(err.code, "INTERNAL");
    assert!(err.message.contains("panicked"), "{err:?}");
    let resp = server.invoke("a.null_handle", vec![]).await;
    assert!(!resp.success);
    assert!(resp.error.as_ref().unwrap().message.contains("no response"));
    let resp = server.invoke("a.stale_handle", vec![]).await;
    assert!(!resp.success);
    assert!(resp.error.as_ref().unwrap().message.contains("stale"));
    let resp = server.invoke("never.registered", vec![]).await;
    assert!(!resp.success);
    assert_eq!(resp.error.as_ref().unwrap().code, "NOT_FOUND");

    // Concurrent invocations of the same method stay isolated.
    let mut tasks = Vec::new();
    for index in 0..8u8 {
        let server = Arc::clone(&server);
        tasks.push(tokio::spawn(async move {
            let tag = vec![index; 4096];
            let resp = server.invoke("z.echo", tag.clone()).await;
            assert!(resp.success, "{:?}", resp.error);
            assert_eq!(resp.result, tag);
        }));
    }
    for task in tasks {
        task.await.expect("concurrent invoke task");
    }
    // Large payload boundary: 1 MiB echo.
    let big = vec![0xA5u8; 1024 * 1024];
    let resp = server.invoke("z.echo", big.clone()).await;
    assert!(resp.success, "{:?}", resp.error);
    assert_eq!(resp.result.len(), big.len());
    assert_eq!(resp.result, big);
    server.drop_dialer().await;
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!(unsafe { mesh_node_stop(node, 5000) }, MESH_NODE_OK);
    assert_eq!(unsafe { mesh_node_state(node) }, 5 /* STOPPED */);

    // Host releases its callback context only after the successful stop.
    drop(captured_slot);
    unsafe { mesh_node_free(node) };
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn stop_during_callback_is_bounded_and_retryable() {
    let server = Arc::new(FakeMeshServer::start().await);
    let cfg = CString::new(server.config_json("block-node")).unwrap();
    let node = unsafe { mesh_node_new(cfg.as_ptr()) };
    assert_ne!(node, 0);

    let block_ctx = Arc::new(BlockCtx {
        entered: AtomicBool::new(false),
        exited: AtomicBool::new(false),
        release: AtomicBool::new(false),
        calls: std::sync::atomic::AtomicUsize::new(0),
    });
    let echo_slot: Arc<CaptureSlot> = Arc::new(std::sync::Mutex::new(None));

    unsafe {
        let name = CString::new("block.on_me").unwrap();
        let cb: MeshNodeMethodCallback = blocking_callback;
        assert_eq!(
            mesh_node_register_method(
                node,
                name.as_ptr(),
                cb,
                Arc::as_ptr(&block_ctx) as *mut c_void,
                std::ptr::null_mut(),
            ),
            MESH_NODE_OK
        );
        let echo_name = CString::new("plain.echo").unwrap();
        let echo: MeshNodeMethodCallback = echo_callback;
        assert_eq!(
            mesh_node_register_method(
                node,
                echo_name.as_ptr(),
                echo,
                Arc::as_ptr(&echo_slot) as *mut c_void,
                std::ptr::null_mut(),
            ),
            MESH_NODE_OK
        );
    }

    assert_eq!(unsafe { grpc_mesh_node::mesh_node_start(node) }, MESH_NODE_OK);
    server.wait_handshake().await;

    let invoke_server = Arc::clone(&server);
    let invoke_task =
        tokio::spawn(async move { invoke_server.invoke("block.on_me", b"x".to_vec()).await });

    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(5);
    while !block_ctx.entered.load(Ordering::SeqCst) {
        assert!(tokio::time::Instant::now() < deadline, "callback never entered");
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    // stop must time out while the callback is parked.
    let stop_result = run_blocking(move || unsafe { mesh_node_stop(node, 300) });
    assert_eq!(stop_result, MESH_NODE_ERR_SHUTDOWN_TIMEOUT);
    assert_eq!(unsafe { mesh_node_state(node) }, 4 /* STOPPING */);

    // While stopping, new work is rejected. The node's gate rejects
    // dispatches with a TRANSPORT error; tonic's graceful shutdown may
    // additionally refuse the stream at the h2 layer (GOAWAY). Both are
    // valid rejections — bounded either way.
    let during_stop = tokio::time::timeout(
        Duration::from_secs(3),
        server.try_invoke("plain.echo", b"y".to_vec()),
    )
    .await
    .expect("bounded probe");
    match during_stop {
        Ok(resp) => {
            assert!(!resp.success, "gate must reject during shutdown");
            assert_eq!(resp.error.as_ref().unwrap().code, "TRANSPORT");
        }
        Err(_) => {} // transport-level refusal: also a rejection
    }

    // Release the parked callback; it must get its completion window even
    // though tonic's graceful shutdown may already have torn the response
    // path down (delivery after shutdown is not part of the ABI contract).
    block_ctx.release.store(true, Ordering::SeqCst);
    let exit_deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    while !block_ctx.exited.load(Ordering::SeqCst) {
        assert!(tokio::time::Instant::now() < exit_deadline, "callback never exited");
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert_eq!(block_ctx.calls.load(Ordering::SeqCst), 1);
    match tokio::time::timeout(Duration::from_secs(2), invoke_task).await {
        Ok(Ok(resp)) => {
            assert!(resp.success, "{:?}", resp.error);
            assert_eq!(resp.result, b"unblocked");
        }
        _ => {} // reply not delivered post-shutdown: accepted
    }
    server.drop_dialer().await;
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!(unsafe { mesh_node_stop(node, 5000) }, MESH_NODE_OK);

    drop(echo_slot);
    drop(block_ctx);
    unsafe { mesh_node_free(node) };
}

/// Runs a blocking ABI call on a worker thread so the async test runtime is
/// not starved while `stop` waits.
fn run_blocking<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::spawn(f).join().expect("blocking ABI thread")
}
