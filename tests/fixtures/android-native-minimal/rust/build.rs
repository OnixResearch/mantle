fn main() {
    let host = std::env::var("HOST").expect("declared HOST");
    let target = std::env::var("TARGET").expect("declared TARGET");
    assert_eq!(host, "x86_64-unknown-linux-gnu", "fixture host toolchain drift");
    assert_eq!(target, "aarch64-linux-android", "fixture target toolchain drift");
    assert_ne!(host, target, "Android build.rs must execute as a host unit");
    if std::env::var_os("CARGO_FEATURE_TRY_NETWORK").is_some() {
        let address = "127.0.0.1:41237".parse().unwrap();
        std::net::TcpStream::connect_timeout(&address, std::time::Duration::from_secs(1))
            .expect("network attempt denied by native sandbox");
        panic!("network isolation violated: host-only listener was reachable");
    }
    if std::env::var_os("CARGO_FEATURE_TRY_UNDECLARED").is_some() {
        std::fs::read("/tmp/mantle-native-undeclared-proof")
            .expect("undeclared input denied by native sandbox");
        panic!("input isolation violated: host-only marker was readable");
    }
    println!("cargo:rerun-if-changed=build.rs");
}
