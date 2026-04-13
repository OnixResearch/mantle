fn main() {
    if std::env::var_os("CARGO_FEATURE_ASYNC_IO").is_some() {
        panic!(
            "vendored fuse-backend-rs disables feature `async-io` in this workspace; \
             upstream async trait/object-safety path does not compile on the pinned toolchain"
        );
    }

    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-lib=framework=DiskArbitration");
}
