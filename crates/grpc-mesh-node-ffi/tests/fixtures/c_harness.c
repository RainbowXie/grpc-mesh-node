/*
 * Pure-C contract harness for the gRPC-Mesh Node C ABI.
 *
 * Compiled and executed by tests/c_harness.rs against the real cdylib and
 * the committed header. Exercises lifecycle rules, handle hygiene and the
 * response-handle ownership model exactly the way a C host would.
 */
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include "grpc_mesh_node.h"

static int checked = 0;
#define CHECK(cond)                                                     \
    do {                                                                \
        if (!(cond)) {                                                  \
            fprintf(stderr, "FAIL line %d: %s\n", __LINE__, #cond);    \
            return 1;                                                   \
        }                                                               \
        checked++;                                                      \
    } while (0)

static const char *UNREACHABLE_CONFIG =
    "{"
    "\"server\": { \"address\": \"127.0.0.1:9\" },"
    "\"node\": { \"id\": \"c-harness\", \"token\": \"t\" },"
    "\"connect\": { \"timeout_secs\": 1 },"
    "\"reconnect\": { \"base_delay_ms\": 50, \"max_delay_ms\": 100 }"
    "}";

/* Echo callback: also proves NUL bytes survive the request view. */
static MeshNodeResponse echo_cb(const MeshNodeRequest *request, void *user_data) {
    (void)user_data;
    if (request->payload == NULL || request->payload_len == 0) {
        return mesh_node_response_error(MESH_NODE_BUSINESS_INTERNAL, "no payload");
    }
    if (strcmp(request->method, "c.echo") != 0) {
        return mesh_node_response_error(MESH_NODE_BUSINESS_INTERNAL, "method mismatch");
    }
    /* Payload may contain NUL bytes; mirror it back by pointer+length. */
    return mesh_node_response_ok(request->payload, request->payload_len);
}

int main(void) {
    /* --- ABI version negotiation --- */
    uint32_t version = mesh_node_abi_version();
    CHECK((version >> 16) == 1);          /* major must be 1 */
    CHECK((version & 0xFFFFu) == 0);       /* minor starts at 0 */

    /* --- invalid creation paths --- */
    CHECK(mesh_node_new(NULL) == 0);
    CHECK(strlen(mesh_node_last_error()) > 0);
    CHECK(mesh_node_new("{ not json") == 0);
    CHECK(strlen(mesh_node_last_error()) > 0);
    CHECK(mesh_node_new("{}") == 0);      /* missing server/node sections */
    CHECK(mesh_node_new(
              "{\"server\":{\"address\":\"127.0.0.1:9\"},\"node\":{\"id\":\"x\"}}") == 0);

    /* --- lifecycle --- */
    MeshNodeHandle node = mesh_node_new(UNREACHABLE_CONFIG);
    CHECK(node != 0);
    CHECK(mesh_node_state(node) == 1 /* CREATED */);
    CHECK(mesh_node_start(node) == MESH_NODE_OK);
    CHECK(mesh_node_state(node) == 3 /* RUNNING */);

    /* Registration is frozen after start. */
    CHECK(mesh_node_register_method(node, "late", echo_cb, NULL, NULL)
          == MESH_NODE_ERR_INVALID_STATE);
    CHECK(mesh_node_unregister_method(node, "late", NULL)
          == MESH_NODE_ERR_INVALID_STATE);
    CHECK(mesh_node_start(node) == MESH_NODE_ERR_INVALID_STATE);

    /* Duplicate stop is a stable no-op. */
    CHECK(mesh_node_stop(node, 5000) == MESH_NODE_OK);
    CHECK(mesh_node_state(node) == 5 /* STOPPED */);
    CHECK(mesh_node_stop(node, 5000) == MESH_NODE_OK);

    mesh_node_free(node);

    /* --- stale handles never alias a new node --- */
    MeshNodeHandle other = mesh_node_new(UNREACHABLE_CONFIG);
    CHECK(other != 0);
    CHECK(other != node);
    CHECK(mesh_node_state(node) == MESH_NODE_STATE_INVALID);
    CHECK(mesh_node_register_method(node, "c.echo", echo_cb, NULL, NULL)
          == MESH_NODE_ERR_NOT_FOUND);
    CHECK(mesh_node_stop(node, 100) == MESH_NODE_ERR_NOT_FOUND);
    mesh_node_free(node);   /* double free: no-op, must not crash */

    /* The live node is unaffected by all of the above. */
    void *replaced = NULL;
    CHECK(mesh_node_register_method(other, "c.echo", echo_cb, NULL, &replaced)
          == MESH_NODE_OK);
    CHECK(replaced == NULL);   /* fresh registration reports no predecessor */
    CHECK(mesh_node_start(other) == MESH_NODE_OK);
    CHECK(mesh_node_stop(other, 5000) == MESH_NODE_OK);
    mesh_node_free(other);

    /* --- registration validation --- */
    MeshNodeHandle strict = mesh_node_new(UNREACHABLE_CONFIG);
    CHECK(strict != 0);
    CHECK(mesh_node_register_method(strict, NULL, echo_cb, NULL, NULL)
          == MESH_NODE_ERR_INVALID_ARGUMENT);
    CHECK(mesh_node_register_method(strict, "", echo_cb, NULL, NULL)
          == MESH_NODE_ERR_INVALID_ARGUMENT);
    CHECK(mesh_node_unregister_method(strict, "missing", NULL)
          == MESH_NODE_ERR_NOT_FOUND);
    mesh_node_free(strict);

    /* --- response handles: ownership, staleness, double release --- */
    static const uint8_t nul_payload[7] = {'a', 0, 'b', 0, 0, 'c', 0};
    MeshNodeResponse r1 = mesh_node_response_ok(nul_payload, sizeof nul_payload);
    CHECK(r1 != 0);
    CHECK(mesh_node_response_len(r1) == 7);
    uint8_t buf[16];
    CHECK(mesh_node_response_read(r1, buf, 3) == 3);
    CHECK(mesh_node_response_read(r1, buf, sizeof buf) == 7);
    CHECK(memcmp(buf, nul_payload, 7) == 0);

    mesh_node_response_release(r1);
    mesh_node_response_release(r1);              /* double release: no-op */
    CHECK(mesh_node_response_read(r1, buf, sizeof buf) == 0);  /* stale read */
    CHECK(mesh_node_response_len(r1) == 0);

    /* Stale release must not affect a later allocation: allocate a fresh
     * response, re-release the old id, then verify the new one is intact. */
    MeshNodeResponse r2 = mesh_node_response_ok((const uint8_t *)"intact", 6);
    CHECK(r2 != r1);
    mesh_node_response_release(r1);
    CHECK(mesh_node_response_len(r2) == 6);
    CHECK(mesh_node_response_read(r2, buf, sizeof buf) == 6);
    CHECK(memcmp(buf, "intact", 6) == 0);
    mesh_node_response_release(r2);

    /* Error responses carry no payload; unknown business codes rejected. */
    MeshNodeResponse err = mesh_node_response_error(MESH_NODE_BUSINESS_INTERNAL, "boom");
    CHECK(err != 0);
    CHECK(mesh_node_response_len(err) == 0);
    CHECK(mesh_node_response_read(err, buf, sizeof buf) == 0);
    mesh_node_response_release(err);
    CHECK(mesh_node_response_error(9999, "bad code") == 0);
    CHECK(mesh_node_response_error(MESH_NODE_BUSINESS_INTERNAL, NULL) == 0);
    CHECK(mesh_node_response_ok(NULL, 5) == 0);

    printf("c_harness: %d checks passed\n", checked);
    return 0;
}
