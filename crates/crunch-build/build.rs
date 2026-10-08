use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=audit/finish_dlopen_audit.c");
    println!("cargo:rustc-check-cfg=cfg(mantle_native_dlopen)");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("linux")
        || env::var("CARGO_CFG_TARGET_ENV").as_deref() != Ok("gnu")
    {
        return;
    }
    let target = env::var("TARGET").expect("Cargo target triple");
    let host = env::var("HOST").expect("Cargo host triple");
    if target != host {
        return;
    }
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo build output directory"))
        .join("mantle-finish-dlopen-audit.so");
    let status = Command::new("cc")
        .args(["-std=c11", "-O2", "-fPIC", "-shared", "-Wall", "-Wextra", "-Werror"])
        .arg("audit/finish_dlopen_audit.c")
        .args(["-o"])
        .arg(&output)
        .status()
        .expect("host C compiler for loader audit module");
    assert!(status.success(), "building native GNU/Linux loader audit module failed");
    println!("cargo:rustc-cfg=mantle_native_dlopen");
}
