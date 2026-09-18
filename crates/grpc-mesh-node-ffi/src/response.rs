//! Response handle registry for the Node C ABI.
//!
//! Host callbacks describe their result by constructing a response handle
//! ([`mesh_node_response_ok`] / [`mesh_node_response_error`]) and returning
//! it from the callback. Handles are monotonic ids backed by a global
//! registry — never raw pointers — so a stale or duplicated release can
//! never alias a live allocation, even when the allocator reuses addresses.

use std::collections::BTreeMap;
use std::ffi::CString;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex};

/// A response handle. `0` is the invalid sentinel.
pub type MeshNodeResponse = u64;

/// Business error codes a host callback can report. These map one-to-one
/// onto the node core's `RpcError` variants so the control plane sees the
/// same error taxonomy as native Rust nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum MeshNodeBusinessCode {
    /// Configuration / precondition failure.
    Config = 1,
    /// The requested entity does not exist.
    NotFound = 2,
    /// Transport / downstream failure.
    Transport = 3,
    /// Generic internal failure.
    Internal = 4,
}

impl TryFrom<i32> for MeshNodeBusinessCode {
    type Error = i32;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Config),
            2 => Ok(Self::NotFound),
            3 => Ok(Self::Transport),
            4 => Ok(Self::Internal),
            other => Err(other),
        }
    }
}

pub(crate) enum ResponseBody {
    Ok(Vec<u8>),
    Err {
        code: MeshNodeBusinessCode,
        message: CString,
    },
}

struct ResponseTable {
    next: AtomicU64,
    live: Mutex<BTreeMap<u64, ResponseBody>>,
}

static TABLE: LazyLock<ResponseTable> = LazyLock::new(|| ResponseTable {
    next: AtomicU64::new(1),
    live: Mutex::new(BTreeMap::new()),
});

/// Registers a successful response carrying `data` and returns its handle.
pub fn response_ok(data: Vec<u8>) -> MeshNodeResponse {
    register(ResponseBody::Ok(data))
}

/// Registers a failure response and returns its handle.
pub fn response_error(code: MeshNodeBusinessCode, message: CString) -> MeshNodeResponse {
    register(ResponseBody::Err { code, message })
}

fn register(body: ResponseBody) -> MeshNodeResponse {
    let id = TABLE.next.fetch_add(1, Ordering::Relaxed);
    TABLE
        .live
        .lock()
        .expect("response table poisoned")
        .insert(id, body);
    id
}

/// Number of payload bytes in a success response (0 for errors and stale
/// handles).
pub fn response_len(handle: MeshNodeResponse) -> usize {
    match lookup(handle) {
        Some(ResponseBody::Ok(data)) => data.len(),
        _ => 0,
    }
}

/// Copies the response payload into `out` (at most `cap` bytes). Returns the
/// number of bytes copied; stale handles and error responses yield 0.
pub fn response_read(handle: MeshNodeResponse, out: &mut [u8]) -> usize {
    match lookup(handle) {
        Some(ResponseBody::Ok(data)) => {
            let n = data.len().min(out.len());
            out[..n].copy_from_slice(&data[..n]);
            n
        }
        _ => 0,
    }
}

/// Consumes the response, returning its body. Returns `None` for stale or
/// already-released handles. The C ABI uses this when a callback returns a
/// handle; the library then owns releasing it.
pub fn take(handle: MeshNodeResponse) -> Option<ResponseBody> {
    TABLE
        .live
        .lock()
        .expect("response table poisoned")
        .remove(&handle)
}

/// Releases a response handle. Idempotent and stale-safe: ids are never
/// reused, so releasing an already-released (or never-created) handle is a
/// no-op that cannot affect any live response.
pub fn release(handle: MeshNodeResponse) {
    // Deliberately ignore the result: double release is a no-op by design.
    let _ = take(handle);
}

/// Number of live (unreleased) response allocations. Test/diagnostic aid.
pub fn live_count() -> usize {
    TABLE.live.lock().expect("response table poisoned").len()
}

fn lookup(handle: MeshNodeResponse) -> Option<ResponseBody> {
    TABLE
        .live
        .lock()
        .expect("response table poisoned")
        .get(&handle)
        .cloned()
}
