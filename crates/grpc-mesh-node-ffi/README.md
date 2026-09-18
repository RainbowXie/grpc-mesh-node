# grpc-mesh-node FFI — Node C ABI v1

`crates/grpc-mesh-node-ffi` compiles the gRPC-Mesh node core
(`grpc_mesh::embedded::EmbeddedNode`) into a `cdylib`/`staticlib` with a
small, versioned C surface plus a JNI shim for the Java/Android binding.
The generated header is [`include/grpc_mesh_node.h`](include/grpc_mesh_node.h)
(byte-identical regeneration is enforced by `tests/header_contract.rs`).

## Symbols (ABI 1.0)

| Symbol | Purpose |
| --- | --- |
| `mesh_node_abi_version` | `(major << 16) \| minor`; callers must verify the major before any other call |
| `mesh_node_new` | create a node from JSON config; `0` on failure (`mesh_node_last_error` on the same thread carries the reason) |
| `mesh_node_register_method` | register/replace a host callback on a `Created` node; optional out-param returns the replaced registration's `user_data` |
| `mesh_node_unregister_method` | remove a callback; optional out-param returns the removed `user_data` |
| `mesh_node_start` | validate config, freeze + report the method snapshot, spawn the supervisor thread with its own Tokio runtime |
| `mesh_node_stop` | bounded stop (`timeout_ms`); `MESH_NODE_ERR_SHUTDOWN_TIMEOUT` is retryable |
| `mesh_node_free` | remove the node from the handle table (best-effort bounded stop first) |
| `mesh_node_state` | 1 Created / 2 Starting / 3 Running / 4 Stopping / 5 Stopped; 0 = invalid handle |
| `mesh_node_last_error` | thread-local, library-owned string valid until the next ABI call on that thread |
| `mesh_node_response_ok` / `_error` | build a callback response handle (monotonic ids) |
| `mesh_node_response_len` / `_read` / `_release` | read/release response handles; release is idempotent and stale-safe |

Business error codes for `mesh_node_response_error`:
`MESH_NODE_BUSINESS_CONFIG=1`, `_NOT_FOUND=2`, `_TRANSPORT=3`,
`_INTERNAL=4` — they map one-to-one onto the node core's `RpcError`
taxonomy, so the control plane sees the same error codes as from a native
Rust node.

## State machine

```
Created -> Starting -> Running -> Stopping -> Stopped -> (freed)
```

* Method registration/removal is legal only in `Created`. `start` freezes
  the method table and reports the sorted names via the `mesh.methods`
  handshake metadata key — the reported list always matches what is served.
* `start` is once-only and terminal on failure (create a new node).
* `stop` is idempotent (`Created`/`Stopped` are no-ops) and retryable after
  a timeout. On timeout the state stays `Stopping` and all resources are
  retained; nothing is force-freed.
* Handles are monotonic `u64` ids and never reused; a stale handle yields
  `MESH_NODE_ERR_NOT_FOUND` (or state 0) and can never alias a live node.
* `free` closes the callback gate, performs a bounded best-effort stop and
  drops the node. Host-owned `user_data` pointers are never freed by the
  library; releasing them is only safe after a successful stop.

## Threading model

* Every export catches Rust panics; nothing unwinds across the FFI
  boundary. `MESH_NODE_ERR_INTERNAL` plus a thread-local message is
  returned instead.
* Each node owns a supervisor thread and its own Tokio runtime; the caller's
  threads are never occupied by the node.
* Method callbacks run concurrently on the node's runtime workers. The
  callback type is declared `C-unwind`: a Rust host callback may panic
  (caught and converted); a plain C callback that never unwind is unaffected.
* `mesh_node_last_error` is thread-local: concurrent host threads never
  overwrite each other's diagnostics.

## Callback contract

```c
MeshNodeResponse cb(const MeshNodeRequest *request, void *user_data);
```

* `request` fields (payload pointer+length, method, correlation id,
  timeout) are borrowed for the duration of the callback only. Payloads are
  binary: NUL bytes are delivered intact via explicit pointer+length.
* Build the return value with `mesh_node_response_ok`/`_error` and return
  the handle; ownership of that handle passes to the library, which reads
  and releases it. Returning `0` (or a stale, already-released handle)
  becomes a structured `INTERNAL` error.
* During a stop the admission gate rejects new callbacks (`TRANSPORT`
  error to the invoker); in-flight callbacks are counted and a stop only
  succeeds after they drained.

## Memory ownership

* Everything passed in is borrowed for the call/callback only.
* Everything the library returns travels through id handles (nodes,
  responses) or a library-owned string (last error). Callers never `free`
  Rust memory.
* Response ids are monotonic: double release is a no-op and a stale release
  cannot affect a later allocation, even when the allocator reuses addresses.

## ABI compatibility

The major version must match exactly; minor versions only add. Any
incompatible change must bump the major version (loading checks reject a
mismatched caller). Regenerate the header after changing the surface:

```sh
cargo run -p grpc-mesh-node-ffi --example gen_header
```

`tests/header_contract.rs` fails the build when the committed header, the
crate surface and the expected 14-symbol set disagree.

## Building

```sh
# host (tests + C harness; the harness builds with ASAN/UBSAN)
cargo test  -p grpc-mesh-node-ffi --target x86_64-unknown-linux-gnu
# Android arm64 (the only ABI the Alpha AAR ships)
cargo build -p grpc-mesh-node-ffi --release --target aarch64-linux-android
```

The Android target is wired via the crate's `.cargo/config.toml` (NDK 29,
API 34); the host target must always be passed explicitly because the
repo's default target is Android.
