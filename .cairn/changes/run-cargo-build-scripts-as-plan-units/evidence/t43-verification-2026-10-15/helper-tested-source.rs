use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use serde_json::Value;
use serde_json::json;

fn tool(name: &str) -> PathBuf {
    for path in std::env::split_paths(&std::env::var_os("PATH").unwrap()) {
        let executable = path.join(name);
        if executable.is_file() {
            return fs::canonicalize(executable).unwrap();
        }
    }
    panic!("missing declared test tool {name}")
}

fn helper(out: &Path, source: &Path, direct: &[&Path], externs: Value, arguments: &[&str]) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mantle-cargo-unit-helper"));
    command
        .env("out", out)
        .env("MANTLE_UNIT_RUSTC", tool("rustc"))
        .env("MANTLE_UNIT_LINKER", tool("gcc"))
        .env("MANTLE_UNIT_SOURCE", source)
        .env(
            "MANTLE_UNIT_DEPENDENCIES",
            serde_json::to_string(&direct.iter().map(|path| path.display().to_string()).collect::<Vec<_>>()).unwrap(),
        )
        .env("MANTLE_UNIT_EXTERNS", externs.to_string())
        .env("MANTLE_UNIT_COMPILE_ENV", "{\"CARGO_PKG_VERSION\":\"0.1.0\"}")
        .env("CARGO_PKG_VERSION", "spoofed")
        .env("CARGO_HOME", "/nonexistent/ambient-cargo-state")
        .args(arguments);
    command.output().unwrap()
}

fn fixture_root(scenario: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("mantle-cargo-unit-helper-{}-{scenario}", std::process::id()));
    if dir.exists() {
        fs::remove_dir_all(&dir).unwrap();
    }
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn three_unit_build_links_transitive_library_through_manifests() {
    let fixture = fixture_root("transitive");
    let store = fixture.join("store");
    let source = store.join("source");
    fs::create_dir_all(&source).unwrap();
    fs::write(source.join("leaf.rs"), "pub fn answer() -> u32 { env!(\"CARGO_PKG_VERSION\").len() as u32 + 37 }")
        .unwrap();
    fs::write(source.join("middle.rs"), "pub fn answer() -> u32 { leaf::answer() }").unwrap();
    fs::write(source.join("app.rs"), "fn main() { assert_eq!(middle::answer(), 42); }").unwrap();
    let leaf = store.join("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-leaf");
    let middle = store.join("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-middle");
    let app = store.join("cccccccccccccccccccccccccccccccc-app");
    let output = helper(&leaf, &source, &[], json!({}), &[
        "--crate-name=leaf",
        "--crate-type=rlib",
        "--edition=2024",
        source.join("leaf.rs").to_str().unwrap(),
    ]);
    assert!(output.status.success(), "leaf failed: {}", String::from_utf8_lossy(&output.stderr));
    let output = helper(&middle, &source, &[&leaf], json!({"leaf": leaf.display().to_string()}), &[
        "--crate-name=middle",
        "--crate-type=rlib",
        "--edition=2024",
        source.join("middle.rs").to_str().unwrap(),
    ]);
    assert!(output.status.success(), "middle failed: {}", String::from_utf8_lossy(&output.stderr));
    let middle_manifest: Value =
        serde_json::from_slice(&fs::read(middle.join("share/mantle/unit-dependencies-v1.json")).unwrap()).unwrap();
    assert_eq!(middle_manifest["direct_dependencies"], json!([leaf.display().to_string()]));
    let output = helper(&app, &source, &[&middle], json!({"middle": middle.display().to_string()}), &[
        "--crate-name=app",
        "--crate-type=bin",
        "--edition=2024",
        source.join("app.rs").to_str().unwrap(),
    ]);
    assert!(output.status.success(), "app failed: {}", String::from_utf8_lossy(&output.stderr));
    let built = Command::new(app.join("lib/app")).output().unwrap();
    assert!(built.status.success(), "compiled app exited {built:?}");
    let app_manifest: Value =
        serde_json::from_slice(&fs::read(app.join("share/mantle/unit-dependencies-v1.json")).unwrap()).unwrap();
    assert_eq!(app_manifest["direct_dependencies"], json!([middle.display().to_string()]));
    assert_eq!(app_manifest["artifacts"], json!(["lib/app"]));
    fs::remove_dir_all(fixture).unwrap();
}

#[test]
fn cargo_feature_cfg_quotes_reach_rustc_through_response_file() {
    let fixture = fixture_root("quoted-cfg");
    let source = fixture.join("source");
    fs::create_dir_all(&source).unwrap();
    fs::write(
        source.join("lib.rs"),
        r#"#[cfg(all(feature = "proc-macro", fixture_label = "hello world"))]
pub const ANSWER: u32 = 42;
#[cfg(not(all(feature = "proc-macro", fixture_label = "hello world")))]
compile_error!("quoted cfg values were not preserved");"#,
    )
    .unwrap();
    let output = fixture.join("store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-quoted-cfg");
    let built = helper(&output, &source, &[], json!({}), &[
        "--crate-name=quoted_cfg",
        "--crate-type=rlib",
        "--edition=2021",
        "--cfg",
        "feature=\"proc-macro\"",
        "--cfg",
        "fixture_label=\"hello world\"",
        source.join("lib.rs").to_str().unwrap(),
    ]);
    assert!(built.status.success(), "real rustc rejected Cargo cfgs: {}", String::from_utf8_lossy(&built.stderr));
    assert!(output.join("lib/libquoted_cfg.rlib").is_file());
    fs::remove_dir_all(fixture).unwrap();
}

#[test]
fn absent_direct_manifest_blocks_before_compilation() {
    let fixture = fixture_root("missing-manifest");
    let source = fixture.join("source");
    fs::create_dir_all(&source).unwrap();
    fs::write(source.join("app.rs"), "fn main() {}").unwrap();
    let dependency = fixture.join("store/01234567890123456789012345678901-absent");
    fs::create_dir_all(&dependency).unwrap();
    let output = helper(&fixture.join("out"), &source, &[&dependency], json!({}), &[
        "--crate-name=app",
        "--crate-type=bin",
        source.join("app.rs").to_str().unwrap(),
    ]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unit-helper-missing-manifest"));
    assert!(!fixture.join("out/lib").exists());
    fs::remove_dir_all(fixture).unwrap();
}

fn run_script_fixture(
    source_text: &str,
    scenario: &str,
    native_root: Option<&Path>,
    links_key: &str,
    profile: (&str, &str, &str),
    linked_flags: Value,
) -> (PathBuf, PathBuf, PathBuf, std::process::Output) {
    let fixture = fixture_root(scenario);
    let source = fixture.join("store/source");
    fs::create_dir_all(&source).unwrap();
    fs::write(source.join("build.rs"), source_text).unwrap();
    let compiled = fixture.join("store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-compiled-script");
    let build = helper(&compiled, &source, &[], json!({}), &[
        "--crate-name=build_script_build",
        "--crate-type=bin",
        "--edition=2021",
        source.join("build.rs").to_str().unwrap(),
    ]);
    assert!(build.status.success(), "script compilation: {}", String::from_utf8_lossy(&build.stderr));
    let target = fixture.join("store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-target-facts");
    let version = Command::new(tool("rustc")).arg("-vV").output().unwrap();
    let host = String::from_utf8(version.stdout).unwrap().lines().find_map(|line| line.strip_prefix("host: "))
        .unwrap().to_string();
    let toolchain = tool("rustc").parent().unwrap().parent().unwrap().to_path_buf();
    let target_result = Command::new(env!("CARGO_BIN_EXE_mantle-cargo-unit-helper"))
        .env("out", &target).env("MANTLE_UNIT_RUSTC", tool("rustc"))
        .env("MANTLE_UNIT_HOST", &host).env("MANTLE_UNIT_TARGET", &host)
        .arg("target-facts").output().unwrap();
    assert!(target_result.status.success(), "target facts: {}", String::from_utf8_lossy(&target_result.stderr));
    let out = fixture.join("store/cccccccccccccccccccccccccccccccc-script-output");
    let native = native_root.map_or_else(
        || json!({"inputs": {}, "env": {}, "path": []}),
        |root| json!({
            "inputs": {"library": root.display().to_string()},
            "env": {"LIBRARY_DIR": root.join("lib").display().to_string()},
            "path": []
        }),
    );
    let flags = fixture.join("store/dddddddddddddddddddddddddddddddd-script-flags");
    let result = Command::new(env!("CARGO_BIN_EXE_mantle-cargo-unit-helper"))
        .env("out", &out).env("flags", &flags)
        .env("MANTLE_UNIT_RUSTC", tool("rustc"))
        .env("MANTLE_UNIT_TOOLCHAIN", toolchain)
        .env("MANTLE_UNIT_SOURCE", source)
        .env("MANTLE_UNIT_SCRIPT", compiled.display().to_string())
        .env("MANTLE_UNIT_TARGET_FACTS", target)
        .env("MANTLE_UNIT_DEPENDENCIES", json!([compiled.display().to_string()]).to_string())
        .env("MANTLE_UNIT_COMPILE_ENV", json!({"PROFILE": profile.0, "OPT_LEVEL": profile.1, "DEBUG": profile.2}).to_string())
        .env("MANTLE_UNIT_LINKED_FLAGS", linked_flags.to_string())
        .env("MANTLE_UNIT_NATIVE", native.to_string())
        .env("MANTLE_UNIT_LINKS_KEY", links_key)
        .arg("run-build-script").output().unwrap();
    (fixture, out, flags, result)
}

#[test]
fn generated_source_cfg_and_env_flow_from_executed_script_to_compiler() {
    let script = r#"fn main() {
        std::fs::write(std::env::var("OUT_DIR").unwrap() + "/generated.rs", "pub const ANSWER: i32 = 42;").unwrap();
        println!("cargo:rustc-cfg=script_enabled");
        println!("cargo:rustc-check-cfg=cfg(script_enabled)");
        println!("cargo:rustc-cfg=second_enabled");
        println!("cargo::rustc-env=SCRIPT_ANSWER=42");
        println!("cargo:rerun-if-changed=build.rs");
        println!("cargo:rerun-if-env-changed=NATIVE_ANSWER_ROOT");
    }"#;
    let (fixture, out, flags, result) = run_script_fixture(script, "script-run-generated", None, "", ("debug", "0", "true"), json!({}));
    assert!(result.status.success(), "run: {}", String::from_utf8_lossy(&result.stderr));
    let parsed: Value = serde_json::from_slice(&fs::read(flags.join("flags.json")).unwrap()).unwrap();
    assert_eq!(parsed["rerun"], json!(["rerun-if-changed=build.rs", "rerun-if-env-changed=NATIVE_ANSWER_ROOT"]));
    assert_eq!(
        fs::read_to_string(flags.join("args-immediate")).unwrap(),
        "--cfg\nscript_enabled\n--check-cfg\ncfg(script_enabled)\n--cfg\nsecond_enabled"
    );
    assert_eq!(
        fs::read_to_string(flags.join("rerun")).unwrap(),
        "rerun-if-changed=build.rs\nrerun-if-env-changed=NATIVE_ANSWER_ROOT"
    );
    let source = fixture.join("store/source");
    fs::write(source.join("lib.rs"), r#"include!(concat!(env!("OUT_DIR"), "/generated.rs"));
#[cfg(all(script_enabled, second_enabled))] pub fn value() -> i32 { ANSWER + env!("SCRIPT_ANSWER").parse::<i32>().unwrap() }
"#).unwrap();
    let lib = fixture.join("store/eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee-generated-lib");
    let built = Command::new(env!("CARGO_BIN_EXE_mantle-cargo-unit-helper"))
        .env("out", &lib).env("MANTLE_UNIT_RUSTC", tool("rustc"))
        .env("MANTLE_UNIT_LINKER", tool("gcc")).env("MANTLE_UNIT_SOURCE", &source)
        .env("MANTLE_UNIT_DEPENDENCIES", json!([out.display().to_string()]).to_string())
        .env("MANTLE_UNIT_EXTERNS", "{}").env("MANTLE_UNIT_COMPILE_ENV", "{}")
        .env("MANTLE_UNIT_FLAGS", &flags).env("MANTLE_UNIT_BUILD_OUT", &out)
        .args(["--crate-name=generated", "--crate-type=rlib", "--edition=2021", source.join("lib.rs").to_str().unwrap()])
        .output().unwrap();
    assert!(built.status.success(), "dependent rustc: {}", String::from_utf8_lossy(&built.stderr));
    fs::write(
        source.join("app.rs"),
        "fn main() { let value = generated::value(); assert_eq!(value, 84); println!(\"generated-script={value}\"); }",
    ).unwrap();
    let app = fixture.join("store/ffffffffffffffffffffffffffffffff-generated-app");
    let built = helper(&app, &source, &[&lib], json!({"generated": lib.display().to_string()}), &[
        "--crate-name=generated_app",
        "--crate-type=bin",
        "--edition=2021",
        source.join("app.rs").to_str().unwrap(),
    ]);
    assert!(built.status.success(), "consumer rustc: {}", String::from_utf8_lossy(&built.stderr));
    let executed = Command::new(app.join("lib/generated_app")).output().unwrap();
    assert!(executed.status.success(), "generated-source consumer failed: {executed:?}");
    assert_eq!(executed.stdout, b"generated-script=84\n");
    fs::remove_dir_all(fixture).unwrap();
}

#[test]
fn run_script_reads_declared_release_profile_instead_of_compiler_defaults() {
    let script = r#"fn main() {
        assert_eq!(std::env::var("PROFILE").unwrap(), "release");
        assert_eq!(std::env::var("OPT_LEVEL").unwrap(), "3");
        assert_eq!(std::env::var("DEBUG").unwrap(), "false");
        println!("cargo:rustc-env=PROFILE_WITNESS=release");
    }"#;
    let (fixture, _out, flags, result) =
        run_script_fixture(script, "release-profile", None, "", ("release", "3", "false"), json!({}));
    assert!(result.status.success(), "run: {}", String::from_utf8_lossy(&result.stderr));
    let parsed: Value = serde_json::from_slice(&fs::read(flags.join("flags.json")).unwrap()).unwrap();
    assert_eq!(parsed["env"]["PROFILE_WITNESS"], "release");
    fs::remove_dir_all(fixture).unwrap();
}

#[test]
fn links_metadata_from_producer_script_reaches_dependent_script() {
    let a = r#"fn main() {
        println!("cargo:metadata=include={}", std::env::var("OUT_DIR").unwrap());
    }"#;
    let (upstream, upstream_out, upstream_flags, result) =
        run_script_fixture(a, "links-owner", None, "foo", ("debug", "0", "true"), json!({}));
    assert!(result.status.success(), "owner: {}", String::from_utf8_lossy(&result.stderr));
    let b = r#"fn main() {
        let include = std::env::var("DEP_FOO_INCLUDE").expect("declared links metadata");
        assert!(std::path::Path::new(&include).is_dir());
        println!("cargo:rustc-env=DEP_WITNESS=applied");
    }"#;
    let (downstream, _out, flags, result) = run_script_fixture(
        b, "links-dependent", None, "", ("debug", "0", "true"),
        json!({"foo": upstream_flags.display().to_string()}),
    );
    assert!(result.status.success(), "dependent: {}", String::from_utf8_lossy(&result.stderr));
    let parsed: Value = serde_json::from_slice(&fs::read(flags.join("flags.json")).unwrap()).unwrap();
    assert_eq!(parsed["env"]["DEP_WITNESS"], "applied");
    assert_eq!(parsed["rerun"], json!([]));
    assert_eq!(
        serde_json::from_slice::<Value>(&fs::read(upstream_flags.join("flags.json")).unwrap()).unwrap()["links"]["include"],
        upstream_out.display().to_string()
    );
    fs::remove_dir_all(downstream).unwrap();
    fs::remove_dir_all(upstream).unwrap();
}

#[test]
fn error_directive_and_nonzero_script_exit_fail_without_flags() {
    for (scenario, script, blocker) in [
        ("script-error", "fn main() { println!(\"cargo::error=bad native config\"); }", "build-script-error-directive"),
        ("script-exit", "fn main() { std::process::exit(7); }", "build-script-nonzero-exit"),
        ("script-orphan-links", "fn main() { println!(\"cargo:metadata=include=include\"); }", "build-script-links-conflict"),
    ] {
        let (fixture, _out, flags, result) = run_script_fixture(script, scenario, None, "", ("debug", "0", "true"), json!({}));
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stderr).contains(blocker), "{result:?}");
        assert!(!flags.join("flags.json").exists());
        fs::remove_dir_all(fixture).unwrap();
    }
}

#[test]
fn oversized_stdout_and_stderr_kill_and_reap_scripts_without_flags() {
    for (stream, writes) in [("stdout", 513), ("stderr", 129)] {
        let script = format!(r#"fn main() {{
            use std::io::Write;
            let out = std::env::var("OUT_DIR").unwrap();
            std::fs::write(format!("{{out}}/script.pid"), std::process::id().to_string()).unwrap();
            let mut pipe = std::io::{stream}();
            for _ in 0..{writes} {{
                pipe.write_all(&[b'x'; 8192]).unwrap();
            }}
            pipe.flush().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(500));
            std::fs::write(format!("{{out}}/survived"), b"unreaped child").unwrap();
        }}"#);
        let (fixture, out, flags, result) = run_script_fixture(
            &script, &format!("oversized-{stream}"), None, "", ("debug", "0", "true"), json!({}),
        );
        assert!(!result.status.success(), "{stream} limit accepted");
        assert!(
            String::from_utf8_lossy(&result.stderr).contains(&format!("build-script-output-limit: {stream}")),
            "{result:?}"
        );
        assert!(!flags.join("flags.json").exists(), "overflow created flags");
        let pid = fs::read_to_string(out.join("script.pid")).unwrap();
        assert!(!Path::new("/proc").join(pid.trim()).exists(), "script {pid} was not reaped");
        assert!(!out.join("survived").exists(), "script survived overflow");
        fs::remove_dir_all(fixture).unwrap();
    }
}

#[test]
fn declared_native_archive_links_and_missing_declaration_fails_script() {
    let native_fixture = fixture_root("native-library");
    let library = native_fixture.join("store/ffffffffffffffffffffffffffffffff-native");
    fs::create_dir_all(library.join("lib")).unwrap();
    fs::write(native_fixture.join("answer.c"), "int answer(void) { return 42; }\n").unwrap();
    let compiled = Command::new(tool("gcc")).args(["-c", native_fixture.join("answer.c").to_str().unwrap(),
        "-o", native_fixture.join("answer.o").to_str().unwrap()]).output().unwrap();
    assert!(compiled.status.success(), "{compiled:?}");
    let archive = Command::new(tool("ar")).args(["rcs", library.join("lib/libanswer.a").to_str().unwrap(),
        native_fixture.join("answer.o").to_str().unwrap()]).output().unwrap();
    assert!(archive.status.success(), "{archive:?}");
    let script = r#"fn main() {
        let directory = std::env::var("LIBRARY_DIR").expect("native library must be declared");
        println!("cargo:rustc-link-search=native={directory}");
        println!("cargo:rustc-link-lib=static=answer");
        println!("cargo:metadata=include={directory}");
    }"#;
    let (fixture, out, flags, result) = run_script_fixture(script, "declared-sys", Some(&library), "answer", ("debug", "0", "true"), json!({}));
    assert!(result.status.success(), "declared native: {}", String::from_utf8_lossy(&result.stderr));
    let parsed: Value = serde_json::from_slice(&fs::read(flags.join("flags.json")).unwrap()).unwrap();
    assert!(parsed["links"]["include"].as_str().unwrap().ends_with("/lib"));
    assert_eq!(
        fs::read_to_string(flags.join("args-transitive")).unwrap(),
        format!("-L\nnative={}/lib\n-l\nstatic=answer", library.display())
    );
    let source = fixture.join("store/source");
    fs::write(source.join("sys.rs"), r#"unsafe extern "C" { fn answer() -> i32; }
pub fn value() -> i32 { unsafe { answer() } }
"#).unwrap();
    fs::write(source.join("app.rs"), "fn main() { assert_eq!(sys::value(), 42); }\n").unwrap();
    let sys = fixture.join("store/eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee-sys");
    let unit = Command::new(env!("CARGO_BIN_EXE_mantle-cargo-unit-helper"))
        .env("out", &sys).env("MANTLE_UNIT_RUSTC", tool("rustc"))
        .env("MANTLE_UNIT_LINKER", tool("gcc")).env("MANTLE_UNIT_SOURCE", &source)
        .env("MANTLE_UNIT_DEPENDENCIES", json!([out.display().to_string()]).to_string())
        .env("MANTLE_UNIT_EXTERNS", "{}").env("MANTLE_UNIT_COMPILE_ENV", "{}")
        .env("MANTLE_UNIT_FLAGS", &flags).env("MANTLE_UNIT_BUILD_OUT", &out)
        .args(["--crate-name=sys", "--crate-type=rlib", "--edition=2021", source.join("sys.rs").to_str().unwrap()])
        .output().unwrap();
    assert!(unit.status.success(), "sys crate: {}", String::from_utf8_lossy(&unit.stderr));
    let app = fixture.join("store/gggggggggggggggggggggggggggggggg-app");
    let built = helper(&app, &source, &[&sys], json!({"sys": sys.display().to_string()}), &[
        "--crate-name=app", "--crate-type=bin", "--edition=2021", source.join("app.rs").to_str().unwrap()
    ]);
    assert!(built.status.success(), "native-linked app: {}", String::from_utf8_lossy(&built.stderr));
    let execution = Command::new(app.join("lib/app")).output().unwrap();
    assert!(execution.status.success(), "native-linked runtime: {execution:?}");
    fs::remove_dir_all(fixture).unwrap();
    let (missing, _out, no_flags, result) = run_script_fixture(script, "missing-sys", None, "answer", ("debug", "0", "true"), json!({}));
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("build-script-nonzero-exit"));
    assert!(!no_flags.join("flags.json").exists());
    fs::remove_dir_all(missing).unwrap();
    fs::remove_dir_all(native_fixture).unwrap();
}
