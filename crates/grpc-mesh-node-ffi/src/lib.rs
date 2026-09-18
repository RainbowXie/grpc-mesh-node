//! Stable C ABI over the gRPC-Mesh node core.
//!
//! This crate compiles the embedding [`grpc_mesh::embedded::EmbeddedNode`]
//! facade into a `cdylib`/`staticlib` with a small, versioned C surface:
//!
//! * ABI version negotiation ([`mesh_node_abi_version`]).
//! * Node lifecycle: [`mesh_node_new`], [`mesh_node_start`],
//!   [`mesh_node_stop`], [`mesh_node_free`], [`mesh_node_state`].
//! * Method registration before start:
//!   [`mesh_node_register_method`], [`mesh_node_unregister_method`].
//! * Host callbacks returning response handles:
//!   [`mesh_node_response_ok`], [`mesh_node_response_error`],
//!   [`mesh_node_response_len`], [`mesh_node_response_read`],
//!   [`mesh_node_response_release`].
//! * Thread-local diagnostics: [`mesh_node_last_error`].
//!
//! # Ownership and concurrency contract (ABI v1)
//!
//! * Node handles are opaque, monotonic `u64` ids; `0` is the failure
//!   sentinel and ids are never reused, so a stale handle cannot alias a
//!   live node.
//! * All pointers passed into an ABI call are borrowed for the duration of
//!   the call only. Data returned by the library (last error) is owned by
//!   the library. Response payloads travel through id handles, never raw
//!   pointers.
//! * Method callbacks may run concurrently on the node's runtime worker
//!   threads. Callback input is valid only for the duration of the
//!   callback. A callback returns a response handle created through the
//!   `mesh_node_response_*` constructors; ownership of that handle passes
//!   to the library when the callback returns it (the library reads and
//!   releases it).
//! * `mesh_node_stop` rejects new callbacks, cancels the tunnel and runtime,
//!   and waits (bounded by `timeout_ms`) for the supervisor to drain. On
//!   timeout it returns `MESH_NODE_ERR_SHUTDOWN_TIMEOUT` and may be retried.
//! * `mesh_node_free` removes the node from the handle table and performs a
//!   best-effort bounded stop. Releasing host-owned callback contexts is
//!   the caller's responsibility and is only safe after a successful stop.
//! * No Rust panic ever crosses the FFI boundary; panics are converted to
//!   `MESH_NODE_ERR_INTERNAL` plus a thread-local message.

mod cerror;
mod jni_shim;
mod response;

use std::collections::BTreeMap;
use std::ffi::{CStr, CString, c_char, c_void};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, LazyLock, Mutex};

use grpc_mesh::embedded::{EmbeddedError, EmbeddedNode, EmbeddedNodeConfig};
use grpc_mesh::{MethodHandlerWithRequest, RpcError, RpcResult};

use crate::response::{MeshNodeBusinessCode, MeshNodeResponse, ResponseBody};

/// ABI version: high 16 bits major, low 16 bits minor. Major must match
/// exactly; minor-only differences are backwards compatible additions.
pub const MESH_NODE_ABI_VERSION: u32 = (1 << 16) | 0;

// ---------------------------------------------------------------------------
// ABI status codes (mirrored verbatim in include/grpc_mesh_node.h)
// ---------------------------------------------------------------------------

/// Call completed successfully.
pub const MESH_NODE_OK: i32 = 0;
/// An argument (null pointer, empty string, malformed JSON) was invalid.
pub const MESH_NODE_ERR_INVALID_ARGUMENT: i32 = 1;
/// The operation does not match the node's current lifecycle state.
pub const MESH_NODE_ERR_INVALID_STATE: i32 = 2;
/// The named method (or handle) does not exist.
pub const MESH_NODE_ERR_NOT_FOUND: i32 = 3;
/// `stop` did not drain within the requested timeout; retryable.
pub const MESH_NODE_ERR_SHUTDOWN_TIMEOUT: i32 = 4;
/// Internal failure or caught panic.
pub const MESH_NODE_ERR_INTERNAL: i32 = 5;
/// Mirror of the ABI state machine for `mesh_node_state`.
pub const MESH_NODE_STATE_INVALID: i32 = 0;

// ---------------------------------------------------------------------------
// Business error codes for host callbacks (mesh_node_response_error)
// ---------------------------------------------------------------------------

/// Business error code: configuration / precondition failure.
pub const MESH_NODE_BUSINESS_CONFIG: i32 = 1;
/// Business error code: the requested entity does not exist.
pub const MESH_NODE_BUSINESS_NOT_FOUND: i32 = 2;
/// Business error code: transport / downstream failure.
pub const MESH_NODE_BUSINESS_TRANSPORT: i32 = 3;
/// Business error code: generic internal failure.
pub const MESH_NODE_BUSINESS_INTERNAL: i32 = 4;

// ---------------------------------------------------------------------------
// ABI types
// ---------------------------------------------------------------------------

/// Read-only request view handed to a host callback. Every field is owned
/// by the library and only valid for the duration of the callback.
#[repr(C)]
pub struct MeshNodeRequest {
    /// Request payload bytes (may be empty, may contain NUL bytes).
    pub payload: *const u8,
    /// Length of `payload` in bytes.
    pub payload_len: usize,
    /// NUL-terminated fully-qualified method name.
    pub method: *const c_char,
    /// NUL-terminated correlation id (may be empty).
    pub correlation_id: *const c_char,
    /// Request timeout in milliseconds as seen by the invoker.
    pub timeout_ms: u32,
}

/// Host callback: process `request`, return a response handle created via
/// `mesh_node_response_ok` / `mesh_node_response_error`. Returning `0`
/// signals an internal error.
///
/// The ABI is declared unwind-permitting (`C-unwind`): a Rust host callback
/// that panics is caught by the library and converted to an internal error
/// instead of aborting the process. Plain C callbacks that never unwind are
/// unaffected.
pub type MeshNodeMethodCallback =
    unsafe extern "C-unwind" fn(request: *const MeshNodeRequest, user_data: *mut c_void) -> MeshNodeResponse;

/// Node handle: opaque, monotonic, never reused; `0` is the failure sentinel.
pub type MeshNodeHandle = u64;

// ---------------------------------------------------------------------------
// Internal state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AbiState {
    Created = 1,
    Starting = 2,
    Running = 3,
    Stopping = 4,
    Stopped = 5,
}

/// Shared callback admission control, deliberately free of any reference to
/// `NodeEntry`/`EmbeddedNode` so handler closures cannot form a reference
/// cycle with the node's method registry.
struct CallbackGate {
    accepting: AtomicBool,
    inflight: Mutex<usize>,
    drained: Condvar,
}

impl CallbackGate {
    fn new() -> Self {
        Self {
            accepting: AtomicBool::new(true),
            inflight: Mutex::new(0),
            drained: Condvar::new(),
        }
    }

    /// Admits one callback invocation, returning whether it may proceed.
    /// The check-and-increment is atomic with respect to `close_gate`.
    fn enter(&self) -> bool {
        let mut count = self.inflight.lock().expect("inflight lock poisoned");
        if !self.accepting.load(Ordering::Acquire) {
            return false;
        }
        *count += 1;
        true
    }

    fn exit(&self) {
        let mut count = self.inflight.lock().expect("inflight lock poisoned");
        *count = count.saturating_sub(1);
        if *count == 0 {
            self.drained.notify_all();
        }
    }

    /// Permanently rejects new callbacks (idempotent).
    fn close_gate(&self) {
        self.accepting.store(false, Ordering::Release);
        self.drained.notify_all();
    }

    /// Waits until no callback is in flight. Returns false on timeout.
    fn wait_drained(&self, timeout: std::time::Duration) -> bool {
        let deadline = std::time::Instant::now() + timeout;
        let mut count = self.inflight.lock().expect("inflight lock poisoned");
        while *count > 0 {
            let now = std::time::Instant::now();
            if now >= deadline {
                return false;
            }
            let (guard, _timeout) = self
                .drained
                .wait_timeout(count, deadline - now)
                .expect("inflight lock poisoned");
            count = guard;
        }
        true
    }
}

struct MethodRegistration {
    callback: MeshNodeMethodCallback,
    /// Host-owned context pointer, treated as opaque. Safety: the host
    /// guarantees it stays valid until the node is freed *after* a
    /// successful stop.
    user_data: *mut c_void,
}

// SAFETY: `user_data` is never dereferenced by this crate, only handed back
// to the host's own callback (which is the party that created it).
unsafe impl Send for MethodRegistration {}
unsafe impl Sync for MethodRegistration {}

struct NodeEntry {
    state: Mutex<AbiState>,
    node: EmbeddedNode,
    methods: Mutex<BTreeMap<String, MethodRegistration>>,
    gate: Arc<CallbackGate>,
}

struct NodeTable {
    next: AtomicU64,
    live: Mutex<BTreeMap<u64, Arc<NodeEntry>>>,
}

static NODES: LazyLock<NodeTable> = LazyLock::new(|| NodeTable {
    next: AtomicU64::new(1),
    live: Mutex::new(BTreeMap::new()),
});

fn lookup(node: MeshNodeHandle) -> Option<Arc<NodeEntry>> {
    NODES
        .live
        .lock()
        .expect("node table poisoned")
        .get(&node)
        .cloned()
}

// ---------------------------------------------------------------------------
// Error plumbing
// ---------------------------------------------------------------------------

struct FfiError {
    status: i32,
}

type FfiResult<T> = Result<T, FfiError>;

fn ffi_err(status: i32, message: impl std::fmt::Display) -> FfiError {
    cerror::set(message.to_string());
    FfiError { status }
}

fn embedded_status(err: &EmbeddedError) -> i32 {
    match err {
        EmbeddedError::Config(_) => MESH_NODE_ERR_INVALID_ARGUMENT,
        EmbeddedError::InvalidState(_) => MESH_NODE_ERR_INVALID_STATE,
        EmbeddedError::MethodNotRegistered(_) => MESH_NODE_ERR_NOT_FOUND,
        EmbeddedError::ShutdownTimeout { .. } => MESH_NODE_ERR_SHUTDOWN_TIMEOUT,
        EmbeddedError::Supervisor(_) => MESH_NODE_ERR_INTERNAL,
    }
}

fn embedded_err(context: &str, err: EmbeddedError) -> FfiError {
    ffi_err(embedded_status(&err), format!("{context}: {err}"))
}

fn panic_message(panic: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = panic.downcast_ref::<&str>() {
        (*s).to_owned()
    } else if let Some(s) = panic.downcast_ref::<String>() {
        s.clone()
    } else {
        "<non-string panic payload>".to_owned()
    }
}

fn guarded<T, F>(name: &str, body: F) -> FfiResult<T>
where
    F: FnOnce() -> FfiResult<T>,
{
    match catch_unwind(AssertUnwindSafe(body)) {
        Ok(result) => result,
        Err(panic) => Err(ffi_err(
            MESH_NODE_ERR_INTERNAL,
            format!("panic inside {name}: {}", panic_message(&panic)),
        )),
    }
}

fn borrow_cstr<'a>(ptr: *const c_char, name: &str) -> FfiResult<&'a CStr> {
    if ptr.is_null() {
        return Err(ffi_err(
            MESH_NODE_ERR_INVALID_ARGUMENT,
            format!("{name} must not be null"),
        ));
    }
    unsafe { Ok(CStr::from_ptr(ptr)) }
}

// ---------------------------------------------------------------------------
// Exported ABI
// ---------------------------------------------------------------------------

/// Returns the ABI version of this library (major << 16 | minor).
#[unsafe(no_mangle)]
pub extern "C" fn mesh_node_abi_version() -> u32 {
    MESH_NODE_ABI_VERSION
}

/// Creates a node from a JSON configuration string.
///
/// Returns a non-zero handle, or `0` on failure with the reason available
/// via [`mesh_node_last_error`] on the same thread.
///
/// # Safety
/// `config_json` must point to a valid NUL-terminated UTF-8 string for the
/// duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mesh_node_new(config_json: *const c_char) -> MeshNodeHandle {
    guarded("mesh_node_new", || {
        let json = borrow_cstr(config_json, "config_json")?
            .to_str()
            .map_err(|_| {
                ffi_err(MESH_NODE_ERR_INVALID_ARGUMENT, "config_json is not valid UTF-8")
            })?;
        let config = EmbeddedNodeConfig::from_json(json)
            .map_err(|err| embedded_err("invalid node configuration", err))?;

        let id = NODES.next.fetch_add(1, Ordering::Relaxed);
        let entry = Arc::new(NodeEntry {
            state: Mutex::new(AbiState::Created),
            node: EmbeddedNode::new(config),
            methods: Mutex::new(BTreeMap::new()),
            gate: Arc::new(CallbackGate::new()),
        });
        NODES
            .live
            .lock()
            .expect("node table poisoned")
            .insert(id, entry);
        Ok(id)
    })
    .unwrap_or(0)
}

/// Registers (or replaces) a host callback for `method` on a `Created` node.
///
/// Replacements are allowed while the node has not started; when
/// `replaced_user_data` is non-null it receives the previous registration's
/// user data (or NULL when this was a fresh registration) so hosts can
/// release associated resources.
///
/// # Safety
/// `method` must be a valid NUL-terminated string. `user_data` must stay
/// valid (per the host's own contract) until the node is freed after a
/// successful stop. `replaced_user_data`, when non-null, must point to
/// writable memory.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mesh_node_register_method(
    node: MeshNodeHandle,
    method: *const c_char,
    callback: MeshNodeMethodCallback,
    user_data: *mut c_void,
    replaced_user_data: *mut *mut c_void,
) -> i32 {
    guarded("mesh_node_register_method", || {
        let entry = lookup(node)
            .ok_or_else(|| ffi_err(MESH_NODE_ERR_NOT_FOUND, "unknown node handle"))?;
        let method_name = borrow_cstr(method, "method")?
            .to_str()
            .map_err(|_| {
                ffi_err(MESH_NODE_ERR_INVALID_ARGUMENT, "method is not valid UTF-8")
            })?
            .to_owned();
        if method_name.trim().is_empty() {
            return Err(ffi_err(
                MESH_NODE_ERR_INVALID_ARGUMENT,
                "method name must not be empty",
            ));
        }

        let state = entry.state.lock().expect("state lock poisoned");
        if *state != AbiState::Created {
            return Err(ffi_err(
                MESH_NODE_ERR_INVALID_STATE,
                format!("methods can only be registered before start (state: {:?})", *state),
            ));
        }
        let previous = entry
            .methods
            .lock()
            .expect("methods lock poisoned")
            .insert(
                method_name,
                MethodRegistration {
                    callback,
                    user_data,
                },
            );
        if !replaced_user_data.is_null() {
            unsafe {
                *replaced_user_data = previous
                    .map(|old| old.user_data)
                    .unwrap_or(std::ptr::null_mut());
            }
        }
        Ok(MESH_NODE_OK)
    })
    .map_or_else(|err| err.status, |status| status)
}

/// Removes the callback registered for `method` on a `Created` node.
///
/// When `removed_user_data` is non-null it receives the removed
/// registration's user data (or NULL when nothing was registered), enabling
/// hosts to release associated resources.
///
/// # Safety
/// `method` must be a valid NUL-terminated string; `removed_user_data`,
/// when non-null, must point to writable memory.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mesh_node_unregister_method(
    node: MeshNodeHandle,
    method: *const c_char,
    removed_user_data: *mut *mut c_void,
) -> i32 {
    guarded("mesh_node_unregister_method", || {
        let entry = lookup(node)
            .ok_or_else(|| ffi_err(MESH_NODE_ERR_NOT_FOUND, "unknown node handle"))?;
        let method_name = borrow_cstr(method, "method")?
            .to_str()
            .map_err(|_| {
                ffi_err(MESH_NODE_ERR_INVALID_ARGUMENT, "method is not valid UTF-8")
            })?
            .to_owned();

        let state = entry.state.lock().expect("state lock poisoned");
        if *state != AbiState::Created {
            return Err(ffi_err(
                MESH_NODE_ERR_INVALID_STATE,
                format!("methods can only be removed before start (state: {:?})", *state),
            ));
        }
        let removed = entry
            .methods
            .lock()
            .expect("methods lock poisoned")
            .remove(&method_name);
        if removed.is_none() {
            return Err(ffi_err(
                MESH_NODE_ERR_NOT_FOUND,
                format!("method {method_name:?} is not registered"),
            ));
        }
        if !removed_user_data.is_null() {
            unsafe {
                *removed_user_data = removed
                    .map(|old| old.user_data)
                    .unwrap_or(std::ptr::null_mut());
            }
        }
        Ok(MESH_NODE_OK)
    })
    .map_or_else(|err| err.status, |status| status)
}

/// Starts the node: validates the configuration, freezes and reports the
/// method snapshot, spawns the supervisor thread with its own runtime.
///
/// Blocks until the runtime is alive (or configuration fails, which is
/// terminal — create a new node).
#[unsafe(no_mangle)]
pub extern "C" fn mesh_node_start(node: MeshNodeHandle) -> i32 {
    guarded("mesh_node_start", || {
        let entry = lookup(node)
            .ok_or_else(|| ffi_err(MESH_NODE_ERR_NOT_FOUND, "unknown node handle"))?;
        {
            let mut state = entry.state.lock().expect("state lock poisoned");
            match *state {
                AbiState::Created => *state = AbiState::Starting,
                other => {
                    return Err(ffi_err(
                        MESH_NODE_ERR_INVALID_STATE,
                        format!("start is only valid once, from Created (state: {other:?})"),
                    ));
                }
            }
        }

        // Freeze registrations into the facade's method table.
        let registrations: Vec<(String, MethodRegistration)> = entry
            .methods
            .lock()
            .expect("methods lock poisoned")
            .iter()
            .map(|(name, reg)| (name.clone(), MethodRegistration {
                callback: reg.callback,
                user_data: reg.user_data,
            }))
            .collect();
        for (name, reg) in registrations {
            let gate = Arc::clone(&entry.gate);
            let handler: MethodHandlerWithRequest = Arc::new(move |request| {
                // Borrow the whole registration so the closure captures the
                // struct (whose unsafe Send/Sync impl covers the raw pointer)
                // instead of disjoint-capturing the raw `user_data` field.
                dispatch_callback(&gate, &reg, request)
            });
            if let Err(err) = entry.node.register_method_with_request(name, handler) {
                let mut state = entry.state.lock().expect("state lock poisoned");
                *state = AbiState::Stopped;
                return Err(embedded_err("register frozen method", err));
            }
        }

        match entry.node.start() {
            Ok(()) => {
                *entry.state.lock().expect("state lock poisoned") = AbiState::Running;
                Ok(MESH_NODE_OK)
            }
            Err(err) => {
                *entry.state.lock().expect("state lock poisoned") = AbiState::Stopped;
                Err(embedded_err("node start failed", err))
            }
        }
    })
    .map_or_else(|err| err.status, |status| status)
}

/// Requests a bounded stop.
///
/// Rejects new callbacks immediately, then waits up to `timeout_ms` for the
/// supervisor to drain. Returns `MESH_NODE_ERR_SHUTDOWN_TIMEOUT` on timeout
/// (retryable; resources are retained). Stopping a `Created` or already
/// stopped node is a successful no-op.
#[unsafe(no_mangle)]
pub extern "C" fn mesh_node_stop(node: MeshNodeHandle, timeout_ms: u32) -> i32 {
    guarded("mesh_node_stop", || {
        let entry = lookup(node)
            .ok_or_else(|| ffi_err(MESH_NODE_ERR_NOT_FOUND, "unknown node handle"))?;
        {
            let mut state = entry.state.lock().expect("state lock poisoned");
            match *state {
                AbiState::Created => {
                    *state = AbiState::Stopped;
                    return Ok(MESH_NODE_OK);
                }
                AbiState::Stopped => return Ok(MESH_NODE_OK),
                AbiState::Starting | AbiState::Stopping | AbiState::Running => {
                    *state = AbiState::Stopping;
                }
            }
        }
        entry.gate.close_gate();
        match entry
            .node
            .stop(std::time::Duration::from_millis(u64::from(timeout_ms)))
        {
            Ok(()) => {
                *entry.state.lock().expect("state lock poisoned") = AbiState::Stopped;
                Ok(MESH_NODE_OK)
            }
            Err(EmbeddedError::ShutdownTimeout { timeout }) => Err(ffi_err(
                MESH_NODE_ERR_SHUTDOWN_TIMEOUT,
                format!(
                    "node did not drain within {timeout:?}; stop may be retried with a longer timeout"
                ),
            )),
            Err(err) => {
                *entry.state.lock().expect("state lock poisoned") = AbiState::Stopped;
                Err(embedded_err("node stop failed", err))
            }
        }
    })
    .map_or_else(|err| err.status, |status| status)
}

/// Removes a node from the handle table. The handle becomes invalid
/// immediately and is never reused.
///
/// If the node is still running, a bounded best-effort stop is attempted
/// and the in-flight callback gate is closed; hosts that care about
/// deterministic teardown should call [`mesh_node_stop`] to success first.
///
/// # Safety
/// Host-owned `user_data` pointers are not freed by the library; releasing
/// them is only safe after a successful stop.
#[unsafe(no_mangle)]
pub extern "C" fn mesh_node_free(node: MeshNodeHandle) {
    let outcome = guarded("mesh_node_free", || {
        let entry = NODES
            .live
            .lock()
            .expect("node table poisoned")
            .remove(&node);
        let entry = match entry {
            Some(entry) => entry,
            None => return Ok(()), // freeing an unknown/stale handle is a no-op
        };
        entry.gate.close_gate();
        // Dropping the entry drops the EmbeddedNode, whose Drop performs a
        // bounded stop. After it returns, the supervisor has drained.
        drop(entry);
        Ok(())
    });
    if let Err(err) = outcome {
        // free returns nothing; surface the reason for diagnostics only.
        let _ = err;
    }
}

/// Returns the node's lifecycle state, or `MESH_NODE_STATE_INVALID` (0) for
/// an unknown handle.
#[unsafe(no_mangle)]
pub extern "C" fn mesh_node_state(node: MeshNodeHandle) -> i32 {
    guarded("mesh_node_state", || {
        let entry = lookup(node)
            .ok_or_else(|| ffi_err(MESH_NODE_ERR_NOT_FOUND, "unknown node handle"))?;
        let state = *entry.state.lock().expect("state lock poisoned");
        Ok(state as i32)
    })
    .unwrap_or(MESH_NODE_STATE_INVALID)
}

/// Returns this thread's last error message, or an empty string.
///
/// The string is library-owned and stays valid until the next ABI call on
/// this thread. Callers must not free or mutate it.
#[unsafe(no_mangle)]
pub extern "C" fn mesh_node_last_error() -> *const c_char {
    guarded("mesh_node_last_error", || Ok::<(), FfiError>(()))
        .ok();
    cerror::last_ptr()
}

// ---------------------------------------------------------------------------
// Response construction / consumption (host-facing)
// ---------------------------------------------------------------------------

/// Creates a successful response carrying `data` (`len` bytes). The data is
/// copied; the caller retains ownership of the input buffer.
///
/// # Safety
/// `data` must be readable for `len` bytes when `len > 0`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mesh_node_response_ok(
    data: *const u8,
    len: usize,
) -> MeshNodeResponse {
    guarded("mesh_node_response_ok", || {
        if len == 0 {
            return Ok(response::response_ok(Vec::new()));
        }
        if data.is_null() {
            return Err(ffi_err(
                MESH_NODE_ERR_INVALID_ARGUMENT,
                "data must not be null when len > 0",
            ));
        }
        let bytes = unsafe { std::slice::from_raw_parts(data, len) }.to_vec();
        Ok(response::response_ok(bytes))
    })
    .unwrap_or(0)
}

/// Creates a structured failure response.
///
/// # Safety
/// `message` must be a valid NUL-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mesh_node_response_error(
    code: i32,
    message: *const c_char,
) -> MeshNodeResponse {
    guarded("mesh_node_response_error", || {
        let text = borrow_cstr(message, "message")?
            .to_str()
            .map_err(|_| {
                ffi_err(MESH_NODE_ERR_INVALID_ARGUMENT, "message is not valid UTF-8")
            })?
            .to_owned();
        let business = MeshNodeBusinessCode::try_from(code).map_err(|_| {
            ffi_err(
                MESH_NODE_ERR_INVALID_ARGUMENT,
                format!("unknown business error code {code}"),
            )
        })?;
        let cstring = CString::new(text).map_err(|_| {
            ffi_err(MESH_NODE_ERR_INVALID_ARGUMENT, "message contains interior NUL")
        })?;
        Ok(response::response_error(business, cstring))
    })
    .unwrap_or(0)
}

/// Payload length of a success response (0 otherwise, including stale handles).
#[unsafe(no_mangle)]
pub extern "C" fn mesh_node_response_len(handle: MeshNodeResponse) -> usize {
    guarded("mesh_node_response_len", || Ok(response::response_len(handle)))
        .unwrap_or(0)
}

/// Copies up to `cap` payload bytes into `out`; returns the byte count
/// copied (0 for stale handles or error responses).
///
/// # Safety
/// `out` must be writable for `cap` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mesh_node_response_read(
    handle: MeshNodeResponse,
    out: *mut u8,
    cap: usize,
) -> usize {
    guarded("mesh_node_response_read", || {
        if cap == 0 {
            return Ok(0);
        }
        if out.is_null() {
            return Err(ffi_err(
                MESH_NODE_ERR_INVALID_ARGUMENT,
                "out must not be null when cap > 0",
            ));
        }
        let buffer = unsafe { std::slice::from_raw_parts_mut(out, cap) };
        Ok(response::response_read(handle, buffer))
    })
    .unwrap_or(0)
}

/// Releases a response handle. Idempotent and stale-safe: ids are never
/// reused, so releasing twice (or releasing a handle the library already
/// consumed) can never affect a live response.
#[unsafe(no_mangle)]
pub extern "C" fn mesh_node_response_release(handle: MeshNodeResponse) {
    let _ = guarded("mesh_node_response_release", || {
        response::release(handle);
        Ok(())
    });
}

// ---------------------------------------------------------------------------
// Callback bridge
// ---------------------------------------------------------------------------

fn map_business_error(body: ResponseBody) -> RpcError {
    let ResponseBody::Err { code, message } = body else {
        return RpcError::Internal("internal mismatch: expected error response".to_owned());
    };
    match code {
        MeshNodeBusinessCode::Config => RpcError::Config(message.to_string_lossy().into_owned()),
        MeshNodeBusinessCode::Transport => {
            RpcError::Transport(message.to_string_lossy().into_owned())
        }
        // NOT_FOUND is rendered through the node's method-not-found
        // taxonomy with the host message preserved in the text.
        MeshNodeBusinessCode::NotFound => RpcError::MethodNotFound(
            message.to_string_lossy().into_owned(),
        ),
        MeshNodeBusinessCode::Internal => {
            RpcError::Internal(message.to_string_lossy().into_owned())
        }
    }
}

/// Runs one host callback under the admission gate and converts its result
/// into the core's `RpcResult`. Panics never escape this boundary.
fn dispatch_callback(
    gate: &Arc<CallbackGate>,
    registration: &MethodRegistration,
    request: grpc_mesh::RpcRequest,
) -> RpcResult<Vec<u8>> {
    if !gate.enter() {
        return Err(RpcError::Transport(
            "node is shutting down; callback rejected".to_owned(),
        ));
    }

    let outcome = run_callback(
        gate,
        registration.callback,
        registration.user_data,
        request,
    );
    gate.exit();
    outcome
}

fn run_callback(
    _gate: &Arc<CallbackGate>,
    callback: MeshNodeMethodCallback,
    user_data: *mut c_void,
    request: grpc_mesh::RpcRequest,
) -> RpcResult<Vec<u8>> {
    let method = CString::new(request.method.as_str())
        .map_err(|_| RpcError::Internal("method name contains interior NUL".to_owned()))?;
    let correlation = CString::new(request.correlation_id.as_str())
        .map_err(|_| RpcError::Internal("correlation id contains interior NUL".to_owned()))?;
    let timeout_ms = u32::try_from(request.timeout.as_millis()).unwrap_or(u32::MAX);

    let view = MeshNodeRequest {
        payload: if request.payload.is_empty() {
            std::ptr::null()
        } else {
            request.payload.as_ptr()
        },
        payload_len: request.payload.len(),
        method: method.as_ptr(),
        correlation_id: correlation.as_ptr(),
        timeout_ms,
    };

    let returned = catch_unwind(AssertUnwindSafe(|| unsafe { callback(&view, user_data) }));
    match returned {
        Ok(handle) if handle != 0 => match response::take(handle) {
            Some(ResponseBody::Ok(bytes)) => Ok(bytes),
            Some(err @ ResponseBody::Err { .. }) => Err(map_business_error(err)),
            None => Err(RpcError::Internal(
                "host callback returned a stale or already-consumed response handle".to_owned(),
            )),
        },
        Ok(_) => Err(RpcError::Internal(
            "host callback returned no response (null handle)".to_owned(),
        )),
        Err(panic) => Err(RpcError::Internal(format!(
            "host callback panicked: {}",
            panic_message(&panic)
        ))),
    }
}

// ---------------------------------------------------------------------------
// Internal accessors for tests and the JNI shim
// ---------------------------------------------------------------------------

/// Number of live nodes (test/diagnostic aid).
pub fn live_node_count() -> usize {
    NODES.live.lock().expect("node table poisoned").len()
}

/// Waits until no callback is in flight for `node`. Returns false on
/// timeout or unknown handle (test/diagnostic aid).
pub fn wait_callbacks_drained(node: MeshNodeHandle, timeout: std::time::Duration) -> bool {
    match lookup(node) {
        Some(entry) => entry.gate.wait_drained(timeout),
        None => false,
    }
}
