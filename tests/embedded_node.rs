//! State-machine and configuration tests for the `EmbeddedNode` facade.
//!
//! These tests exercise the lifecycle contract that the C ABI (and every
//! language binding built on it) depends on: registration is frozen at
//! start, state transitions are enforced, stop is idempotent, start
//! failures clean up, and stop timeouts are retryable.

use std::sync::Arc;
use std::time::Duration;

use grpc_mesh::embedded::{EmbeddedError, EmbeddedNode, EmbeddedNodeConfig, NodeLifecycle};
use grpc_mesh::{MethodRegistry, RpcError};

fn offline_config() -> EmbeddedNodeConfig {
    EmbeddedNodeConfig {
        server_addr: "127.0.0.1:1".into(),
        sni: None,
        ca_certs: Vec::new(),
        insecure_skip_verify: false,
        node_id: "test-node".into(),
        token: "test-token".into(),
        version: "0.1.0-test".into(),
        features: vec!["yamux-reverse-grpc".into()],
        metadata: Default::default(),
        connect_timeout: Duration::from_secs(1),
        max_backoff: Duration::from_millis(200),
        heartbeat_interval: Duration::from_secs(15),
        reconnect_base_delay: Duration::from_millis(50),
        reconnect_max_delay: Duration::from_millis(200),
        healthy_reset_after: Duration::from_secs(60),
        stop_grace: Duration::from_secs(2),
    }
}

#[test]
fn lifecycle_happy_path() {
    let node = EmbeddedNode::new(offline_config());
    assert_eq!(node.state(), NodeLifecycle::Created);

    node.register_method("echo", Arc::new(|p| Ok(p)))
        .expect("register before start");
    node.unregister_method("echo").expect("unregister before start");
    assert!(node.methods().is_empty());

    node.start().expect("start from Created");
    assert_eq!(node.state(), NodeLifecycle::Running);

    node.stop(Duration::from_secs(5)).expect("stop from Running");
    assert_eq!(node.state(), NodeLifecycle::Stopped);
}

#[test]
fn duplicate_stop_is_noop() {
    let node = EmbeddedNode::new(offline_config());
    node.start().expect("start");
    node.stop(Duration::from_secs(5)).expect("first stop");
    node.stop(Duration::from_secs(5)).expect("second stop is a no-op");
    assert_eq!(node.state(), NodeLifecycle::Stopped);
}

#[test]
fn stop_never_started_node_is_noop() {
    let node = EmbeddedNode::new(offline_config());
    node.stop(Duration::from_secs(1)).expect("stop on Created");
    assert_eq!(node.state(), NodeLifecycle::Stopped);
}

#[test]
fn start_reentry_and_post_stop_are_rejected() {
    let node = EmbeddedNode::new(offline_config());
    node.start().expect("first start");
    match node.start() {
        Err(EmbeddedError::InvalidState(msg)) => {
            assert!(msg.contains("start"), "unexpected message: {msg}");
        }
        other => panic!("expected InvalidState, got {other:?}"),
    }
    node.stop(Duration::from_secs(5)).expect("stop");

    match node.start() {
        Err(EmbeddedError::InvalidState(_)) => {}
        other => panic!("expected InvalidState after stop, got {other:?}"),
    }
}

#[test]
fn registration_is_frozen_after_start() {
    let node = EmbeddedNode::new(offline_config());
    node.start().expect("start");
    match node.register_method("late", Arc::new(|p| Ok(p))) {
        Err(EmbeddedError::InvalidState(msg)) => {
            assert!(msg.contains("before start"), "unexpected message: {msg}");
        }
        other => panic!("expected InvalidState, got {other:?}"),
    }
    match node.unregister_method("echo") {
        Err(EmbeddedError::InvalidState(_)) => {}
        other => panic!("expected InvalidState, got {other:?}"),
    }
}

#[test]
fn methods_snapshot_is_sorted_and_deduped_by_replacement() {
    let node = EmbeddedNode::new(offline_config());
    node.register_method("zeta.echo", Arc::new(|p| Ok(p)))
        .expect("register");
    node.register_method("alpha.echo", Arc::new(|p| Ok(p)))
        .expect("register");
    node.register_method("alpha.echo", Arc::new(|_| Ok(vec![2])))
        .expect("replacement register");

    assert_eq!(node.methods(), vec!["alpha.echo".to_string(), "zeta.echo".to_string()]);
}

#[test]
fn unregister_unknown_method_fails() {
    let node = EmbeddedNode::new(offline_config());
    match node.unregister_method("missing") {
        Err(EmbeddedError::MethodNotRegistered(m)) => assert_eq!(m, "missing"),
        other => panic!("expected MethodNotRegistered, got {other:?}"),
    }
}

#[test]
fn start_failure_cleans_up_and_is_terminal() {
    let mut cfg = offline_config();
    // A CA bundle that parses no certificate must fail config validation.
    cfg.ca_certs = vec![b"not a pem certificate".to_vec()];

    let node = EmbeddedNode::new(cfg);
    match node.start() {
        Err(EmbeddedError::Config(msg)) => {
            assert!(msg.to_lowercase().contains("ca"), "unexpected message: {msg}");
        }
        other => panic!("expected Config error, got {other:?}"),
    }
    assert_eq!(node.state(), NodeLifecycle::Stopped);
    // Cleanup: stop is a no-op, restart is rejected.
    node.stop(Duration::from_secs(1)).expect("stop after failed start");
    assert!(matches!(node.start(), Err(EmbeddedError::InvalidState(_))));
}

#[test]
fn stop_timeout_is_reported_and_retryable() {
    // A listener that accepts TCP and reads the ClientHello but never
    // completes the TLS handshake pins the supervisor inside the connect
    // budget (an uninterruptible section), so a short stop deadline is
    // guaranteed to time out. The ClientHello arrival is also the
    // synchronization point proving the supervisor reached that section
    // before stop() fires. Retrying with a deadline beyond the connect
    // timeout must succeed.
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().unwrap().port();
    let client_hello_seen = Arc::new(std::sync::Mutex::new(false));
    let signal = Arc::clone(&client_hello_seen);
    let acceptor = std::thread::spawn(move || {
        // Hold every accepted socket open without speaking TLS.
        let mut held = Vec::new();
        for stream in listener.incoming() {
            match stream {
                Ok(mut s) => {
                    // The first bytes from the node are its TLS ClientHello.
                    let mut probe = [0u8; 1];
                    use std::io::Read;
                    if s.read(&mut probe).is_ok() {
                        *signal.lock().unwrap() = true;
                    }
                    held.push(s);
                }
                Err(_) => break,
            }
        }
        held
    });

    let mut cfg = offline_config();
    cfg.server_addr = format!("127.0.0.1:{port}");
    cfg.connect_timeout = Duration::from_secs(2);

    let node = EmbeddedNode::new(cfg);
    node.start().expect("start");
    assert_eq!(node.state(), NodeLifecycle::Running);

    // Determinism: only stop once the supervisor is provably inside the
    // (uninterruptible) TLS handshake wait.
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while !*client_hello_seen.lock().unwrap() {
        assert!(
            std::time::Instant::now() < deadline,
            "node never sent its ClientHello"
        );
        std::thread::sleep(Duration::from_millis(5));
    }

    match node.stop(Duration::from_millis(200)) {
        Err(EmbeddedError::ShutdownTimeout { timeout }) => {
            assert_eq!(timeout, Duration::from_millis(200));
        }
        other => panic!("expected ShutdownTimeout, got {other:?}"),
    }
    assert_eq!(node.state(), NodeLifecycle::Stopping);

    node.stop(Duration::from_secs(10))
        .expect("retry stop after timeout");
    assert_eq!(node.state(), NodeLifecycle::Stopped);

    drop(acceptor);
}

#[test]
fn config_json_validation() {
    let base = r#"{
        "server": { "address": "127.0.0.1:8443" },
        "node": { "id": "n1", "token": "t1" }
    }"#;
    let cfg = EmbeddedNodeConfig::from_json(base).expect("minimal valid config");
    assert_eq!(cfg.server_addr, "127.0.0.1:8443");
    assert_eq!(cfg.node_id, "n1");
    assert!(!cfg.version.is_empty(), "version defaults");

    // Missing token.
    let bad = r#"{ "server": { "address": "127.0.0.1:8443" }, "node": { "id": "n1" } }"#;
    match EmbeddedNodeConfig::from_json(bad) {
        Err(EmbeddedError::Config(msg)) => assert!(msg.contains("token"), "{msg}"),
        other => panic!("expected Config, got {other:?}"),
    }

    // Missing node id.
    let bad = r#"{ "server": { "address": "127.0.0.1:8443" }, "node": { "token": "t" } }"#;
    match EmbeddedNodeConfig::from_json(bad) {
        Err(EmbeddedError::Config(msg)) => assert!(msg.contains("id"), "{msg}"),
        other => panic!("expected Config, got {other:?}"),
    }

    // Missing server address.
    let bad = r#"{ "node": { "id": "n1", "token": "t" } }"#;
    match EmbeddedNodeConfig::from_json(bad) {
        Err(EmbeddedError::Config(msg)) => assert!(msg.contains("address"), "{msg}"),
        other => panic!("expected Config, got {other:?}"),
    }

    // Broken JSON.
    let bad = "{ not json";
    match EmbeddedNodeConfig::from_json(bad) {
        Err(EmbeddedError::Config(_)) => {}
        other => panic!("expected Config, got {other:?}"),
    }

    // Inline PEM CA that cannot parse.
    let bad = r#"{
        "server": { "address": "127.0.0.1:8443", "tls": { "ca_cert": "garbage" } },
        "node": { "id": "n1", "token": "t1" }
    }"#;
    match EmbeddedNodeConfig::from_json(bad) {
        Err(EmbeddedError::Config(msg)) => assert!(msg.to_lowercase().contains("ca"), "{msg}"),
        other => panic!("expected Config, got {other:?}"),
    }
}

#[test]
fn registry_errors_reach_callers_unchanged() {
    // Guards the facade contract that business errors are surfaced verbatim
    // through the existing MethodRegistry error model (the C ABI maps the
    // same variants to structured host errors).
    let registry = MethodRegistry::default();
    registry.register(
        "boom",
        Arc::new(|_| Err(RpcError::Internal("exploded".into()))),
    );
    match registry.invoke("boom", vec![]) {
        Err(RpcError::Internal(msg)) => assert_eq!(msg, "exploded"),
        other => panic!("expected Internal, got {other:?}"),
    }
}
