#ifndef GRPC_MESH_NODE_ABI_H
#define GRPC_MESH_NODE_ABI_H

#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * ABI version: high 16 bits major, low 16 bits minor. Major must match
 * exactly; minor-only differences are backwards compatible additions.
 */
#define MESH_NODE_ABI_VERSION ((1 << 16) | 0)

/**
 * Call completed successfully.
 */
#define MESH_NODE_OK 0

/**
 * An argument (null pointer, empty string, malformed JSON) was invalid.
 */
#define MESH_NODE_ERR_INVALID_ARGUMENT 1

/**
 * The operation does not match the node's current lifecycle state.
 */
#define MESH_NODE_ERR_INVALID_STATE 2

/**
 * The named method (or handle) does not exist.
 */
#define MESH_NODE_ERR_NOT_FOUND 3

/**
 * `stop` did not drain within the requested timeout; retryable.
 */
#define MESH_NODE_ERR_SHUTDOWN_TIMEOUT 4

/**
 * Internal failure or caught panic.
 */
#define MESH_NODE_ERR_INTERNAL 5

/**
 * Mirror of the ABI state machine for `mesh_node_state`.
 */
#define MESH_NODE_STATE_INVALID 0

/**
 * Business error code: configuration / precondition failure.
 */
#define MESH_NODE_BUSINESS_CONFIG 1

/**
 * Business error code: the requested entity does not exist.
 */
#define MESH_NODE_BUSINESS_NOT_FOUND 2

/**
 * Business error code: transport / downstream failure.
 */
#define MESH_NODE_BUSINESS_TRANSPORT 3

/**
 * Business error code: generic internal failure.
 */
#define MESH_NODE_BUSINESS_INTERNAL 4

/**
 * Node handle: opaque, monotonic, never reused; `0` is the failure sentinel.
 */
typedef uint64_t MeshNodeHandle;

/**
 * A response handle. `0` is the invalid sentinel.
 */
typedef uint64_t MeshNodeResponse;

/**
 * Read-only request view handed to a host callback. Every field is owned
 * by the library and only valid for the duration of the callback.
 */
typedef struct MeshNodeRequest {
  /**
   * Request payload bytes (may be empty, may contain NUL bytes).
   */
  const uint8_t *payload;
  /**
   * Length of `payload` in bytes.
   */
  size_t payload_len;
  /**
   * NUL-terminated fully-qualified method name.
   */
  const char *method;
  /**
   * NUL-terminated correlation id (may be empty).
   */
  const char *correlation_id;
  /**
   * Request timeout in milliseconds as seen by the invoker.
   */
  uint32_t timeout_ms;
} MeshNodeRequest;

/**
 * Host callback: process `request`, return a response handle created via
 * `mesh_node_response_ok` / `mesh_node_response_error`. Returning `0`
 * signals an internal error.
 *
 * The ABI is declared unwind-permitting (`C-unwind`): a Rust host callback
 * that panics is caught by the library and converted to an internal error
 * instead of aborting the process. Plain C callbacks that never unwind are
 * unaffected.
 */
typedef MeshNodeResponse (*MeshNodeMethodCallback)(const struct MeshNodeRequest *request,
                                                   void *user_data);

/**
 * Returns the ABI version of this library (major << 16 | minor).
 */
uint32_t mesh_node_abi_version(void);

/**
 * Creates a node from a JSON configuration string.
 *
 * Returns a non-zero handle, or `0` on failure with the reason available
 * via [`mesh_node_last_error`] on the same thread.
 *
 * # Safety
 * `config_json` must point to a valid NUL-terminated UTF-8 string for the
 * duration of the call.
 */
MeshNodeHandle mesh_node_new(const char *config_json);

/**
 * Registers (or replaces) a host callback for `method` on a `Created` node.
 *
 * Replacements are allowed while the node has not started; when
 * `replaced_user_data` is non-null it receives the previous registration's
 * user data (or NULL when this was a fresh registration) so hosts can
 * release associated resources.
 *
 * # Safety
 * `method` must be a valid NUL-terminated string. `user_data` must stay
 * valid (per the host's own contract) until the node is freed after a
 * successful stop. `replaced_user_data`, when non-null, must point to
 * writable memory.
 */
int32_t mesh_node_register_method(MeshNodeHandle node,
                                  const char *method,
                                  MeshNodeMethodCallback callback,
                                  void *user_data,
                                  void **replaced_user_data);

/**
 * Removes the callback registered for `method` on a `Created` node.
 *
 * When `removed_user_data` is non-null it receives the removed
 * registration's user data (or NULL when nothing was registered), enabling
 * hosts to release associated resources.
 *
 * # Safety
 * `method` must be a valid NUL-terminated string; `removed_user_data`,
 * when non-null, must point to writable memory.
 */
int32_t mesh_node_unregister_method(MeshNodeHandle node,
                                    const char *method,
                                    void **removed_user_data);

/**
 * Starts the node: validates the configuration, freezes and reports the
 * method snapshot, spawns the supervisor thread with its own runtime.
 *
 * Blocks until the runtime is alive (or configuration fails, which is
 * terminal — create a new node).
 */
int32_t mesh_node_start(MeshNodeHandle node);

/**
 * Requests a bounded stop.
 *
 * Rejects new callbacks immediately, then waits up to `timeout_ms` for the
 * supervisor to drain. Returns `MESH_NODE_ERR_SHUTDOWN_TIMEOUT` on timeout
 * (retryable; resources are retained). Stopping a `Created` or already
 * stopped node is a successful no-op.
 */
int32_t mesh_node_stop(MeshNodeHandle node, uint32_t timeout_ms);

/**
 * Removes a node from the handle table. The handle becomes invalid
 * immediately and is never reused.
 *
 * If the node is still running, a bounded best-effort stop is attempted
 * and the in-flight callback gate is closed; hosts that care about
 * deterministic teardown should call [`mesh_node_stop`] to success first.
 *
 * # Safety
 * Host-owned `user_data` pointers are not freed by the library; releasing
 * them is only safe after a successful stop.
 */
void mesh_node_free(MeshNodeHandle node);

/**
 * Returns the node's lifecycle state, or `MESH_NODE_STATE_INVALID` (0) for
 * an unknown handle.
 */
int32_t mesh_node_state(MeshNodeHandle node);

/**
 * Returns this thread's last error message, or an empty string.
 *
 * The string is library-owned and stays valid until the next ABI call on
 * this thread. Callers must not free or mutate it.
 */
const char *mesh_node_last_error(void);

/**
 * Creates a successful response carrying `data` (`len` bytes). The data is
 * copied; the caller retains ownership of the input buffer.
 *
 * # Safety
 * `data` must be readable for `len` bytes when `len > 0`.
 */
MeshNodeResponse mesh_node_response_ok(const uint8_t *data, size_t len);

/**
 * Creates a structured failure response.
 *
 * # Safety
 * `message` must be a valid NUL-terminated UTF-8 string.
 */
MeshNodeResponse mesh_node_response_error(int32_t code, const char *message);

/**
 * Payload length of a success response (0 otherwise, including stale handles).
 */
size_t mesh_node_response_len(MeshNodeResponse handle);

/**
 * Copies up to `cap` payload bytes into `out`; returns the byte count
 * copied (0 for stale handles or error responses).
 *
 * # Safety
 * `out` must be writable for `cap` bytes.
 */
size_t mesh_node_response_read(MeshNodeResponse handle, uint8_t *out, size_t cap);

/**
 * Releases a response handle. Idempotent and stale-safe: ids are never
 * reused, so releasing twice (or releasing a handle the library already
 * consumed) can never affect a live response.
 */
void mesh_node_response_release(MeshNodeResponse handle);

#endif  /* GRPC_MESH_NODE_ABI_H */
