//! Config-validation contract tests for the Node C ABI.
//!
//! Split out of `abi_contract.rs`, which sits near the 1000-line file budget,
//! so the metadata-validation cases stay in a focused module. `mesh_node_new`
//! must reject reserved/non-string/blank metadata at creation (null handle plus
//! a config error naming the key) instead of silently accepting a forged
//! method list or platform value.

use std::ffi::{CStr, CString};

use grpc_mesh_node::{mesh_node_last_error, mesh_node_new};

fn last_error() -> String {
    unsafe { CStr::from_ptr(mesh_node_last_error()).to_string_lossy().into_owned() }
}

fn config_with_metadata(metadata: &str) -> CString {
    CString::new(format!(
        r#"{{ "server": {{ "address": "127.0.0.1:9" }}, "node": {{ "id": "x", "token": "t", "metadata": {metadata} }} }}"#
    ))
    .expect("valid CString")
}

#[test]
fn reserved_metadata_keys_rejected_at_creation() {
    for key in ["mesh.methods", "platform", "arch"] {
        let cfg = config_with_metadata(&format!(r#"{{ "{key}": "forged" }}"#));
        assert_eq!(
            unsafe { mesh_node_new(cfg.as_ptr()) },
            0,
            "reserved key {key} must fail creation"
        );
        assert!(
            last_error().contains(key),
            "error {:?} must name {key}",
            last_error()
        );
    }
}

#[test]
fn invalid_metadata_key_and_value_rejected_at_creation() {
    for (metadata, needle) in [
        (r#"{ " padded ": "v" }"#, "padded"),
        (r#"{ "": "v" }"#, "key"),
        (r#"{ "k": null }"#, "k"),
        (r#"{ "k": 3 }"#, "k"),
    ] {
        let cfg = config_with_metadata(metadata);
        assert_eq!(unsafe { mesh_node_new(cfg.as_ptr()) }, 0);
        assert!(
            last_error().contains(needle),
            "error {:?} must name {needle}",
            last_error()
        );
    }
}
