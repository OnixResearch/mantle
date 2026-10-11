//! Build-script directive grammar, propagation, target facts and native inputs.

use std::collections::BTreeMap;

use mantle_rust_plan_core::ArgScope;
use mantle_rust_plan_core::ImmediateArgTarget;
use mantle_rust_plan_core::NativeInputDeclaration;
use mantle_rust_plan_core::check_link_search_roots;
use mantle_rust_plan_core::check_links_unique;
use mantle_rust_plan_core::merge_links_env;
use mantle_rust_plan_core::parse_build_script_output;
use mantle_rust_plan_core::parse_flags_env;
use mantle_rust_plan_core::parse_target_facts;
use mantle_rust_plan_core::parse_transitive_args;
use mantle_rust_plan_core::render_build_script_flags;
use mantle_rust_plan_core::render_target_facts;
use mantle_rust_plan_core::resolve_native_inputs;
use mantle_rust_plan_core::select_immediate_args;

const SCRIPT: &str = "\
noise that cargo ignores
cargo:rustc-cfg=has_foo
cargo::rustc-check-cfg=cfg(has_foo, values(\"a\", \"b\"))
cargo:rustc-env=GENERATED=yes
cargo::rustc-link-lib=static:+whole-archive=foo
cargo:rustc-link-search=native=/out/lib
cargo::rustc-link-arg=-Wl,--as-needed
cargo::rustc-link-arg-bin=tool=-Wl,-z,now
cargo:rustc-cdylib-link-arg=-Wl,-soname,libx.so
cargo:rustc-flags=-l dylib=z -L/out/z
cargo::metadata=include=/out/include
cargo:version=1.2
cargo:warning=be careful
cargo::rerun-if-changed=src/x
cargo:rerun-if-env-changed=FOO_DIR
";

fn lib_target() -> ImmediateArgTarget<'static> {
    ImmediateArgTarget {
        kind: "lib",
        name: "pkg",
        is_cdylib: false,
    }
}

#[test]
fn supported_directives_parse_in_output_order_for_both_syntaxes() {
    let parsed = parse_build_script_output(SCRIPT).unwrap();
    let args = parsed
        .immediate_args
        .iter()
        .map(|scoped| (scoped.scope.clone(), scoped.arg.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(args, vec![
        (ArgScope::All, "--cfg=has_foo"),
        (ArgScope::All, "--check-cfg=cfg(has_foo, values(\"a\", \"b\"))"),
        (ArgScope::All, "-lstatic:+whole-archive=foo"),
        (ArgScope::All, "-Lnative=/out/lib"),
        (ArgScope::Link, "-Clink-arg=-Wl,--as-needed"),
        (ArgScope::Bin("tool".into()), "-Clink-arg=-Wl,-z,now"),
        (ArgScope::Cdylib, "-Clink-arg=-Wl,-soname,libx.so"),
        (ArgScope::All, "-ldylib=z"),
        (ArgScope::All, "-L/out/z"),
    ]);
    assert_eq!(parsed.transitive_args, vec!["-Lnative=/out/lib", "-L/out/z"]);
    assert_eq!(parsed.env, vec![("GENERATED".to_string(), "yes".to_string())]);
    assert_eq!(parsed.metadata, vec![
        ("include".to_string(), "/out/include".to_string()),
        ("version".to_string(), "1.2".to_string()),
    ]);
    assert_eq!(parsed.warnings, vec!["be careful"]);
    assert_eq!(parsed.rerun, vec!["rerun-if-changed=src/x", "rerun-if-env-changed=FOO_DIR"]);
}

#[test]
fn build_script_values_preserve_significant_trailing_spaces_through_dependent_flags() {
    let parsed = parse_build_script_output(
        "cargo::rustc-env=SUFFIX=with spaces  \r\ncargo::metadata=include=/store/foo/include \r\ncargo:revision=patched  \n",
    )
    .unwrap();
    let flags = render_build_script_flags(&parsed, Some("foo"));
    assert_eq!(parse_flags_env(&flags.env).unwrap(), vec![("SUFFIX".to_string(), "with spaces  ".to_string())]);
    let dependent = merge_links_env(&[&flags.links]).unwrap();
    assert_eq!(dependent.get("DEP_FOO_INCLUDE").map(String::as_str), Some("/store/foo/include "));
    assert_eq!(dependent.get("DEP_FOO_REVISION").map(String::as_str), Some("patched  "));
}

#[test]
fn immediate_arguments_are_filtered_by_target_kind() {
    let files = render_build_script_flags(&parse_build_script_output(SCRIPT).unwrap(), Some("foo"));
    let lib = select_immediate_args(&files.args_immediate, &lib_target()).unwrap();
    assert!(lib.contains(&"--cfg=has_foo".to_string()));
    assert!(lib.iter().all(|arg| !arg.starts_with("-Clink-arg")), "{lib:?}");
    let tool = ImmediateArgTarget {
        kind: "bin",
        name: "tool",
        is_cdylib: false,
    };
    let tool_args = select_immediate_args(&files.args_immediate, &tool).unwrap();
    assert!(tool_args.contains(&"-Clink-arg=-Wl,--as-needed".to_string()));
    assert!(tool_args.contains(&"-Clink-arg=-Wl,-z,now".to_string()));
    assert!(!tool_args.contains(&"-Clink-arg=-Wl,-soname,libx.so".to_string()));
    let other = ImmediateArgTarget { name: "other", ..tool };
    assert!(
        !select_immediate_args(&files.args_immediate, &other)
            .unwrap()
            .contains(&"-Clink-arg=-Wl,-z,now".to_string())
    );
    let cdylib = ImmediateArgTarget {
        is_cdylib: true,
        ..lib_target()
    };
    let cdylib_args = select_immediate_args(&files.args_immediate, &cdylib).unwrap();
    assert!(cdylib_args.contains(&"-Clink-arg=-Wl,-soname,libx.so".to_string()));
    assert!(select_immediate_args("unknown\t--cfg=x\n", &lib_target()).is_err());
}

#[test]
fn flags_files_have_stable_line_formats() {
    let files = render_build_script_flags(&parse_build_script_output(SCRIPT).unwrap(), Some("foo-sys"));
    assert!(files.args_immediate.starts_with("all\t--cfg=has_foo\n"));
    assert!(files.args_immediate.contains("bin:tool\t-Clink-arg=-Wl,-z,now\n"));
    assert_eq!(files.args_transitive, "-Lnative=/out/lib\n-L/out/z\n");
    assert_eq!(files.env, "GENERATED=yes\n");
    assert_eq!(files.links, "DEP_FOO_SYS_INCLUDE=/out/include\nDEP_FOO_SYS_VERSION=1.2\n");
    assert_eq!(files.rerun, "rerun-if-changed=src/x\nrerun-if-env-changed=FOO_DIR\n");
    assert_eq!(files.warnings, "be careful\n");
    assert_eq!(parse_transitive_args(&files.args_transitive).unwrap(), vec!["-Lnative=/out/lib", "-L/out/z"]);
    assert_eq!(parse_flags_env(&files.env).unwrap(), vec![("GENERATED".to_string(), "yes".to_string())]);
    assert!(parse_transitive_args("--cfg=x\n").is_err());
    let unlinked = render_build_script_flags(&parse_build_script_output(SCRIPT).unwrap(), None);
    assert!(unlinked.links.is_empty(), "metadata without links must not propagate");
}

#[test]
fn rerun_directives_are_recorded_without_changing_compilation() {
    let base = parse_build_script_output("cargo:rustc-cfg=x\n").unwrap();
    let rerun = parse_build_script_output("cargo:rustc-cfg=x\ncargo:rerun-if-changed=build.rs\n").unwrap();
    let (base_files, rerun_files) = (render_build_script_flags(&base, None), render_build_script_flags(&rerun, None));
    assert_eq!(base_files.args_immediate, rerun_files.args_immediate);
    assert_eq!(base_files.args_transitive, rerun_files.args_transitive);
    assert_eq!(rerun_files.rerun, "rerun-if-changed=build.rs\n");
    assert!(base_files.rerun.is_empty());
}

#[test]
fn links_metadata_reaches_a_dependent_script_as_dep_environment() {
    let a = parse_build_script_output("cargo::metadata=include=/store/a-out/include\n").unwrap();
    let a_links = render_build_script_flags(&a, Some("foo")).links;
    let b = parse_build_script_output("cargo:root=/store/b-out\n").unwrap();
    let b_links = render_build_script_flags(&b, Some("bar")).links;
    let merged = merge_links_env(&[&a_links, &b_links]).unwrap();
    assert_eq!(merged.get("DEP_FOO_INCLUDE").map(String::as_str), Some("/store/a-out/include"));
    assert_eq!(merged.get("DEP_BAR_ROOT").map(String::as_str), Some("/store/b-out"));
    assert_eq!(merged.len(), 2);
    let repeated = merge_links_env(&["DEP_FOO_X=1\nDEP_FOO_X=2\n"]).unwrap();
    assert_eq!(repeated.get("DEP_FOO_X").map(String::as_str), Some("2"));
    assert!(merge_links_env(&["CARGO_X=1\n"]).is_err());
}

#[test]
fn transitive_link_search_paths_must_stay_inside_admitted_roots() {
    let parsed = parse_build_script_output("cargo:rustc-link-search=native=/store/aaa-out/lib\n").unwrap();
    assert!(check_link_search_roots(&parsed, &["/store/aaa-out"]).is_ok());
    let error = check_link_search_roots(&parsed, &["/store/aaa-outside"]).unwrap_err();
    assert_eq!(error.code(), "build-script-directive-unsupported");
    let host = parse_build_script_output("cargo:rustc-link-search=/usr/lib\n").unwrap();
    assert!(check_link_search_roots(&host, &["/store/aaa-out"]).is_err());
}

#[test]
fn unknown_directive_fails_naming_the_directive() {
    let error = parse_build_script_output("cargo::rustc-unknown=1\n").unwrap_err();
    assert_eq!(error.code(), "build-script-directive-unsupported");
    assert!(error.to_string().contains("rustc-unknown"), "{error}");
    let legacy_reserved = parse_build_script_output("cargo:rustc-link-whatever=1\n").unwrap_err();
    assert_eq!(legacy_reserved.code(), "build-script-directive-unsupported");
    assert!(legacy_reserved.to_string().contains("rustc-link-whatever"));
    let no_value = parse_build_script_output("cargo::rustc-cfg\n").unwrap_err();
    assert_eq!(no_value.code(), "build-script-directive-unsupported");
    let bad_flags = parse_build_script_output("cargo:rustc-flags=-C opt-level=3\n").unwrap_err();
    assert_eq!(bad_flags.code(), "build-script-directive-unsupported");
    let rename = parse_build_script_output("cargo:rustc-link-lib=static=foo:bar\n").unwrap_err();
    assert_eq!(rename.code(), "build-script-directive-unsupported");
    let parent = parse_build_script_output("cargo:rustc-link-search=native=/out/../etc\n").unwrap_err();
    assert_eq!(parent.code(), "build-script-directive-unsupported");
}

#[test]
fn error_directive_fails_and_legacy_error_key_is_metadata() {
    let error = parse_build_script_output("cargo:rustc-cfg=x\ncargo::error=missing libfoo\n").unwrap_err();
    assert_eq!(error.code(), "build-script-error-directive");
    assert!(error.to_string().contains("line 2"));
    assert!(error.to_string().contains("missing libfoo"));
    let legacy = parse_build_script_output("cargo:error=value\n").unwrap();
    assert_eq!(legacy.metadata, vec![("error".to_string(), "value".to_string())]);
}

#[test]
fn duplicate_links_keys_conflict() {
    let error = check_links_unique(&[("pkg-a 0.1.0", "foo"), ("pkg-b 0.1.0", "foo")]).unwrap_err();
    assert_eq!(error.code(), "build-script-links-conflict");
    assert!(error.to_string().contains("foo"));
    assert!(check_links_unique(&[("pkg-a 0.1.0", "foo"), ("pkg-a 0.1.0", "foo"), ("pkg-b 0.1.0", "bar")]).is_ok());
    let merged = merge_links_env(&["DEP_FOO_INCLUDE=/a\n", "DEP_FOO_INCLUDE=/b\n"]).unwrap_err();
    assert_eq!(merged.code(), "build-script-links-conflict");
}

#[test]
fn output_bounds_fail_closed() {
    let many_lines = "x\n".repeat(65_537);
    assert_eq!(parse_build_script_output(&many_lines).unwrap_err().code(), "build-script-output-limit");
    let long_value = format!("cargo:rustc-cfg={}\n", "a".repeat(16_385));
    assert_eq!(parse_build_script_output(&long_value).unwrap_err().code(), "build-script-output-limit");
    assert!(parse_build_script_output(&"x\n".repeat(65_536)).is_ok());
}

#[test]
fn target_facts_are_typed_and_exclude_profile_cfg() {
    let cfg = "debug_assertions\npanic=\"unwind\"\ntarget_arch=\"x86_64\"\ntarget_feature=\"fxsr\"\ntarget_feature=\"sse2\"\nunix\nfeature=\"x\"\n";
    let facts = render_target_facts(cfg, "x86_64-unknown-linux-gnu", "x86_64-unknown-linux-gnu").unwrap();
    assert_eq!(
        facts,
        "TARGET=x86_64-unknown-linux-gnu\nHOST=x86_64-unknown-linux-gnu\nCARGO_CFG_PANIC=unwind\nCARGO_CFG_TARGET_ARCH=x86_64\nCARGO_CFG_TARGET_FEATURE=fxsr,sse2\nCARGO_CFG_UNIX=\n"
    );
    let parsed = parse_target_facts(&facts).unwrap();
    assert_eq!(parsed.len(), 6);
    assert!(parse_target_facts("TARGET=x\n").is_err());
    assert!(parse_target_facts("TARGET=x\nHOST=y\nPATH=/bin\n").is_err());
    assert!(render_target_facts("target_os=linux\n", "t", "h").is_err());
}

fn declaration(env: &[(&str, &str)], path: &[&str]) -> NativeInputDeclaration {
    NativeInputDeclaration {
        inputs: BTreeMap::from([
            ("zlib".to_string(), "/nix/store/00000000000000000000000000000000-zlib-1.3".to_string()),
            ("pkg-config".to_string(), "/nix/store/11111111111111111111111111111111-pkg-config".to_string()),
        ]),
        env: env.iter().map(|(key, value)| ((*key).to_string(), (*value).to_string())).collect(),
        path: path.iter().map(ToString::to_string).collect(),
    }
}

#[test]
fn native_role_markers_resolve_to_declared_store_paths() {
    let resolved = resolve_native_inputs(&declaration(
        &[
            ("PKG_CONFIG_PATH", "{{role:zlib}}/lib/pkgconfig"),
            ("ZLIB_FLAGS", "-I{{role:zlib}}/include -lz"),
        ],
        &["{{role:pkg-config}}/bin"],
    ))
    .unwrap();
    assert_eq!(
        resolved.env["PKG_CONFIG_PATH"],
        "/nix/store/00000000000000000000000000000000-zlib-1.3/lib/pkgconfig"
    );
    assert_eq!(resolved.env["ZLIB_FLAGS"], "-I/nix/store/00000000000000000000000000000000-zlib-1.3/include -lz");
    assert_eq!(resolved.path, vec!["/nix/store/11111111111111111111111111111111-pkg-config/bin"]);
    assert_eq!(resolved.inputs.len(), 2);
}

#[test]
fn undeclared_role_marker_and_host_paths_fail() {
    let undeclared = resolve_native_inputs(&declaration(&[("OPENSSL_DIR", "{{role:openssl}}")], &[])).unwrap_err();
    assert_eq!(undeclared.code(), "native-input-role-undeclared");
    assert!(undeclared.to_string().contains("openssl"));
    let path_role = resolve_native_inputs(&declaration(&[], &["{{role:cc}}/bin"])).unwrap_err();
    assert_eq!(path_role.code(), "native-input-role-undeclared");
    for host in [
        "/usr/lib/pkgconfig",
        "-I/usr/include",
        "{{role:zlib}}/lib:/usr/lib",
        "{{role:zlib}}/../other",
        "{{role:zlib}}/lib/../../other",
    ] {
        let error = resolve_native_inputs(&declaration(&[("FLAGS", host)], &[])).unwrap_err();
        assert_eq!(error.code(), "native-input-host-path", "{host}");
    }
    assert_eq!(
        resolve_native_inputs(&declaration(&[], &["/usr/bin"])).unwrap_err().code(),
        "native-input-host-path"
    );
    let mut relative = declaration(&[], &[]);
    relative.inputs.insert("bad".into(), "relative/path".into());
    assert_eq!(resolve_native_inputs(&relative).unwrap_err().code(), "native-input-host-path");
    for reserved in ["OUT_DIR", "PATH", "CARGO_FEATURE_X", "DEP_FOO_INCLUDE", "TARGET"] {
        let error = resolve_native_inputs(&declaration(&[(reserved, "x")], &[])).unwrap_err();
        assert_eq!(error.code(), "native-input-env-reserved", "{reserved}");
    }
    let unterminated = resolve_native_inputs(&declaration(&[("X", "{{role:zlib")], &[])).unwrap_err();
    assert_eq!(unterminated.code(), "native-input-invalid");
}
