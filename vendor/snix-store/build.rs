use std::io::Result;
use std::path::PathBuf;

fn main() -> Result<()> {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let generated = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("src/generated/snix.store.v1.rs");

    if has_protoc() {
        generate_from_proto(&out_dir)?;
    } else {
        eprintln!("cargo:warning=protoc not found, using pre-generated proto files");
        std::fs::copy(&generated, out_dir.join("snix.store.v1.rs"))?;
    }

    Ok(())
}

fn has_protoc() -> bool {
    std::process::Command::new("protoc")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn generate_from_proto(out_dir: &std::path::Path) -> Result<()> {
    #[allow(unused_mut)]
    let mut builder = tonic_build::configure();

    #[cfg(feature = "tonic-reflection")]
    {
        let descriptor_path = out_dir.join("snix.store.v1.bin");
        builder = builder.file_descriptor_set_path(descriptor_path);
    };

    let proto_root = match std::env::var_os("PROTO_ROOT") {
        Some(proto_root) => proto_root.to_str().unwrap().to_owned(),
        None => {
            let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
            format!("{manifest_dir}/../proto")
        }
    };

    builder
        .build_server(true)
        .build_client(true)
        .emit_rerun_if_changed(false)
        .bytes(["."])
        .extern_path(".snix.castore.v1", "::snix_castore::proto")
        .compile_protos(
            &[
                "snix/store/protos/pathinfo.proto",
                "snix/store/protos/rpc_pathinfo.proto",
            ],
            &[&proto_root],
        )
}
