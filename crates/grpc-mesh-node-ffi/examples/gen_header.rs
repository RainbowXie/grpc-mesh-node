//! Regenerates include/grpc_mesh_node.h from the crate's public ABI surface.
//!
//! Run from the crate directory:
//!
//! ```text
//! cargo run --example gen_header
//! ```
//!
//! The committed header is the source of truth for ABI review; the
//! deterministic-regeneration test (tests/header_contract.rs) fails when the
//! crate surface and the header drift apart.

use std::path::PathBuf;

fn main() {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let config = cbindgen::Config::from_file(crate_dir.join("cbindgen.toml"))
        .expect("read cbindgen.toml");
    cbindgen::Builder::new()
        .with_crate(&crate_dir)
        .with_config(config)
        .generate()
        .expect("generate grpc_mesh_node.h")
        .write_to_file(crate_dir.join("include").join("grpc_mesh_node.h"));
    println!("wrote include/grpc_mesh_node.h");
}
