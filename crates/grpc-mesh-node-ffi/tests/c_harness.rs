//! Compiles and runs the pure-C contract harness (tests/fixtures/c_harness.c)
//! against the real cdylib and the committed header.
//!
//! The harness binary is built with AddressSanitizer + UndefinedBehaviorSanitizer
//! (the environment has no valgrind); the Rust side of the library uses the
//! system allocator, so heap misuse crossing the ABI is intercepted.

use std::path::PathBuf;
use std::process::Command;

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn find_cdylib() -> PathBuf {
    // Walk up from the crate directory to the workspace root, then scan the
    // target directory for both host (target/debug) and cross
    // (target/<triple>/debug) layouts.
    let mut dir = crate_dir();
    while dir.pop() {
        let target = dir.join("target");
        if !target.is_dir() {
            continue;
        }
        let mut candidates = vec![target.join("debug"), target.join("release")];
        if let Ok(entries) = std::fs::read_dir(&target) {
            for entry in entries.flatten() {
                candidates.push(entry.path().join("debug"));
                candidates.push(entry.path().join("release"));
            }
        }
        for candidate in candidates {
            for lib in ["libgrpc_mesh_node.so", "libgrpc_mesh_node.dylib"] {
                let path = candidate.join(lib);
                if path.exists() {
                    return path;
                }
            }
        }
    }
    panic!("cdylib not found; build with `cargo build -p grpc-mesh-node-ffi`");
}

fn sanitize_env_present() -> bool {
    std::env::var("GRPC_MESH_NODE_C_HARNESS_NO_SANITIZE").is_err()
}

#[test]
fn c_harness_against_real_cdylib() {
    let crate_root = crate_dir();
    let lib = find_cdylib();
    let lib_dir = lib.parent().expect("lib dir").to_path_buf();

    let out_dir = std::env::temp_dir().join(format!(
        "grpc-mesh-node-c-harness-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&out_dir).expect("create temp dir");

    let bin = out_dir.join("c_harness");
    let mut build = Command::new(std::env::var("CC").unwrap_or_else(|_| "cc".into()));
    build
        .arg(crate_root.join("tests").join("fixtures").join("c_harness.c"))
        .arg("-o").arg(&bin)
        .arg("-I").arg(crate_root.join("include"))
        .arg(format!("-L{}", lib_dir.display()))
        .arg("-lgrpc_mesh_node")
        .arg(format!("-Wl,-rpath,{}", lib_dir.display()))
        .arg("-pthread")
        .arg("-ldl") // ASAN runtime dependency on some toolchains
        .arg("-Werror=implicit-function-declaration");
    if sanitize_env_present() {
        build
            .arg("-fsanitize=address,undefined")
            .arg("-fno-omit-frame-pointer")
            .arg("-g");
    }
    let status = build.status().expect("cc must be available to build the C harness");
    assert!(status.success(), "C harness failed to compile: {build:?}");

    let run = Command::new(&bin)
        .env("ASAN_OPTIONS", "detect_leaks=0") // the Rust runtime may hold allocations
        .status()
        .expect("run c harness");
    assert!(
        run.success(),
        "C harness failed (see FAIL lines above); sanitize flags: {}",
        sanitize_env_present()
    );

    let _ = std::fs::remove_dir_all(&out_dir);
}
