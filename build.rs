use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let proto_dir = manifest_dir
        .join("..")
        .join("grpc-mesh-server")
        .join("pkg")
        .join("rpc");
    let proto_file = proto_dir.join("waemu.proto");
    let generated_dir = manifest_dir.join("src").join("generated");

    let protoc_path = protoc_bin_vendored::protoc_bin_path()?;
    unsafe {
        env::set_var("PROTOC", protoc_path);
    }

    ensure_exists(&proto_file)?;
    fs::create_dir_all(&generated_dir)?;

    println!("cargo:rerun-if-changed={}", proto_file.display());
    if let Some(entries) = proto_dir.read_dir().ok() {
        for entry in entries.flatten() {
            println!("cargo:rerun-if-changed={}", entry.path().display());
        }
    }

    tonic_prost_build::configure()
        .build_client(true)
        .build_server(true)
        .out_dir(&generated_dir)
        .compile_protos(&[proto_file], &[proto_dir])?;

    let default_output = generated_dir.join("waemu.rpc.v1.rs");
    let named_output = generated_dir.join("waemu.rs");
    if default_output.exists() && default_output != named_output {
        if named_output.exists() {
            fs::remove_file(&named_output)?;
        }
        fs::rename(&default_output, &named_output)?;
    }

    Ok(())
}

fn ensure_exists(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if !path.exists() {
        return Err(format!("missing proto file: {}", path.display()).into());
    }
    Ok(())
}
