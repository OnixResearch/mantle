use std::env;
use std::io;
use std::path::PathBuf;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=audit/finish_dlopen_audit.c");
    println!("cargo:rustc-check-cfg=cfg(mantle_native_dlopen)");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("linux")
        || env::var("CARGO_CFG_TARGET_ENV").as_deref() != Ok("gnu")
    {
        return Ok(());
    }
    let target = env::var("TARGET")?;
    let host = env::var("HOST")?;
    assert!(!target.is_empty(), "Cargo target triple cannot be empty");
    assert!(!host.is_empty(), "Cargo host triple cannot be empty");
    if target != host {
        return Ok(());
    }
    let output = PathBuf::from(
        env::var_os("OUT_DIR")
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Cargo build output directory"))?,
    )
    .join("mantle-finish-dlopen-audit.so");
    let status = Command::new("cc")
        .args(["-std=c11", "-O2", "-fPIC", "-shared", "-Wall", "-Wextra", "-Werror"])
        .arg("audit/finish_dlopen_audit.c")
        .args(["-o"])
        .arg(&output)
        .status()?;
    assert!(status.success(), "building native GNU/Linux loader audit module failed");
    println!("cargo:rustc-cfg=mantle_native_dlopen");
    Ok(())
}
