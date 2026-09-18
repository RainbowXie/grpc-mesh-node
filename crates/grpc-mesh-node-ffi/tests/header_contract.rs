//! Header contract tests for the Node C ABI.
//!
//! 1. `committed_header_matches_cbindgen`: regenerating the header with the
//!    pinned cbindgen config must be byte-identical to the committed
//!    `include/grpc_mesh_node.h`. This is the deterministic-regeneration
//!    gate: the committed header cannot drift from the crate surface.
//! 2. `symbol_sets_agree`: the exported-symbol set declared in the header,
//!    the `#[unsafe(no_mangle)]` functions in `src/lib.rs`, and the expected
//!    ABI v1 symbol list must all be exactly equal — no accidental exports,
//!    no silent omissions.

use std::collections::BTreeSet;
use std::path::PathBuf;

const EXPECTED_SYMBOLS: &[&str] = &[
    "mesh_node_abi_version",
    "mesh_node_free",
    "mesh_node_last_error",
    "mesh_node_new",
    "mesh_node_register_method",
    "mesh_node_response_error",
    "mesh_node_response_len",
    "mesh_node_response_ok",
    "mesh_node_response_read",
    "mesh_node_response_release",
    "mesh_node_start",
    "mesh_node_state",
    "mesh_node_stop",
    "mesh_node_unregister_method",
];

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn committed_header_matches_cbindgen() {
    let dir = crate_dir();
    let config = cbindgen::Config::from_file(dir.join("cbindgen.toml")).expect("cbindgen.toml");
    let bindings = cbindgen::Builder::new()
        .with_crate(&dir)
        .with_config(config)
        .generate()
        .expect("cbindgen must be able to generate the header");
    let mut generated_bytes = Vec::new();
    use std::io::Write as _;
    bindings.write(&mut generated_bytes);
    let generated = String::from_utf8(generated_bytes).expect("header is UTF-8");

    let committed =
        std::fs::read_to_string(dir.join("include").join("grpc_mesh_node.h")).expect(
            "include/grpc_mesh_node.h must be committed; run \
             `cargo run --example gen_header` and commit the result",
        );

    assert_eq!(
        committed.trim_end(),
        generated.trim_end(),
        "committed header is stale: run `cargo run --example gen_header` and commit"
    );
}

#[test]
fn symbol_sets_agree() {
    let dir = crate_dir();
    let header =
        std::fs::read_to_string(dir.join("include").join("grpc_mesh_node.h")).expect("header");
    let source = std::fs::read_to_string(dir.join("src").join("lib.rs")).expect("lib.rs");

    // Function declarations in the header (not the typedef'd callback).
    // Handles cbindgen's multi-line signatures; doc-comment references in
    // backticks never have a '(' right after the name, so they are excluded.
    let header_symbols = extract_call_symbols(&header);

    // no_mangle exports in the implementation.
    let lines: Vec<&str> = source.lines().collect();
    let mut source_symbols: BTreeSet<String> = BTreeSet::new();
    for (index, line) in lines.iter().enumerate() {
        if line.trim() != "#[unsafe(no_mangle)]" {
            continue;
        }
        let Some(signature) = lines.get(index + 1) else { continue };
        let Some(rest) = signature.split("fn ").nth(1) else {
            continue;
        };
        let name = rest.split('(').next().unwrap_or("").trim();
        if name.starts_with("mesh_node_") {
            source_symbols.insert(name.to_owned());
        }
    }

    let expected: BTreeSet<String> = EXPECTED_SYMBOLS
        .iter()
        .map(|s| (*s).to_owned())
        .collect();

    assert_eq!(header_symbols, expected, "header symbol set mismatch");
    assert_eq!(source_symbols, expected, "implementation symbol set mismatch");
}

/// Collects every `mesh_node_*` identifier that is immediately called
/// (followed by optional spaces and `(`) anywhere in `text`.
fn extract_call_symbols(text: &str) -> BTreeSet<String> {
    let bytes = text.as_bytes();
    let mut out = BTreeSet::new();
    let mut search_from = 0;
    while let Some(found) = text[search_from..].find("mesh_node_") {
        let start = search_from + found;
        let mut end = start + "mesh_node_".len();
        while end < bytes.len()
            && (bytes[end].is_ascii_lowercase() || bytes[end] == b'_')
        {
            end += 1;
        }
        let mut cursor = end;
        while cursor < bytes.len() && bytes[cursor] == b' ' {
            cursor += 1;
        }
        if cursor < bytes.len() && bytes[cursor] == b'(' {
            out.insert(text[start..end].to_owned());
        }
        search_from = end;
    }
    out
}
