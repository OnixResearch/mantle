use mantle_cargo_unit_plan::parse_directives;

#[test]
fn directives_preserve_cfg_env_link_order_and_metadata() {
    let native = "/mantle/store/01234567890123456789012345678901-native/lib".to_string();
    let stdout = format!(
        "unrelated log\ncargo:rustc-cfg=has_native\ncargo::rustc-check-cfg=cfg(has_native)\n\
         cargo:rustc-env=NATIVE_VERSION=5\ncargo:rustc-link-search=native={native}\n\
         cargo:rustc-link-lib=static=answer\ncargo:rustc-link-arg=-Wl,--as-needed\n\
         cargo:metadata=include={native}/include\ncargo:rerun-if-changed=build.rs\n"
    );
    let parsed = parse_directives(&stdout, std::slice::from_ref(&native)).unwrap();
    assert_eq!(parsed.args_immediate[0..4], ["--cfg", "has_native", "--check-cfg", "cfg(has_native)"]);
    assert_eq!(parsed.args_transitive[0..4], ["-L", format!("native={native}").as_str(), "-l", "static=answer"]);
    assert_eq!(parsed.env["NATIVE_VERSION"], "5");
    assert_eq!(parsed.links["include"], format!("{native}/include"));
    assert_eq!(parsed.rerun, ["rerun-if-changed=build.rs"]);
}

#[test]
fn rejected_directives_cannot_cross_into_compilation_or_host_paths() {
    let root = "/mantle/store/01234567890123456789012345678901-native".to_string();
    for (stdout, blocker) in [
        ("cargo:rustc-cfgs=ignored", "build-script-directive-unsupported"),
        ("cargo::error=missing native library", "build-script-error-directive"),
        ("cargo:rustc-link-search=native=/usr/lib", "build-script-directive-unsupported"),
        ("cargo:rustc-env=PATH=/usr/bin", "build-script-directive-unsupported"),
    ] {
        let error = parse_directives(stdout, std::slice::from_ref(&root)).unwrap_err();
        assert_eq!(error.code, blocker, "{stdout}");
    }
}
