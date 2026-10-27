use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::time::Duration;
use std::time::Instant;

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
    helper_with(out, source, direct, externs, arguments, &[])
}

fn helper_with(
    out: &Path,
    source: &Path,
    direct: &[&Path],
    externs: Value,
    arguments: &[&str],
    extra: &[(&str, String)],
) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mantle-cargo-unit-helper"));
    command.envs(extra.iter().map(|(key, value)| (*key, value)));
    command
        .env("out", out)
        .env("MANTLE_UNIT_RUSTC", tool("rustc"))
        .env("MANTLE_UNIT_LINKER", tool("clang"))
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
#[cfg(target_os = "linux")]
fn cdylib_link_arguments_control_real_shared_library_soname() {
    let fixture = fixture_root("cdylib-link-arguments");
    let source = fixture.join("store/source");
    fs::create_dir_all(&source).unwrap();
    fs::write(source.join("lib.rs"), "#[unsafe(no_mangle)] pub extern \"C\" fn answer() -> i32 { 42 }\n").unwrap();
    let out = fixture.join("shared");
    let compile = helper_with(
        &out,
        &source,
        &[],
        json!({}),
        &[
            "--crate-name=answer",
            "--crate-type=cdylib",
            "--edition=2024",
            source.join("lib.rs").to_str().unwrap(),
        ],
        &[("MANTLE_UNIT_LINK_ARGS", json!(["-Wl,-soname,libdeclared-answer.so"]).to_string())],
    );
    assert!(compile.status.success(), "{}", String::from_utf8_lossy(&compile.stderr));
    let inspect = Command::new(tool("readelf")).arg("--dynamic").arg(out.join("lib/libanswer.so")).output().unwrap();
    assert!(inspect.status.success());
    assert!(String::from_utf8_lossy(&inspect.stdout).contains("Library soname: [libdeclared-answer.so]"));
    let consumer = source.join("consumer.c");
    fs::write(&consumer, "extern int answer(void); int main(void) { return answer() == 42 ? 0 : 1; }\n").unwrap();
    let executable = fixture.join("consumer");
    let link = Command::new(tool("clang"))
        .arg(consumer)
        .arg(out.join("lib/libanswer.so"))
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(link.status.success(), "{}", String::from_utf8_lossy(&link.stderr));
    fs::copy(out.join("lib/libanswer.so"), out.join("lib/libdeclared-answer.so")).unwrap();
    let run = Command::new(executable).env("LD_LIBRARY_PATH", out.join("lib")).output().unwrap();
    assert!(run.status.success(), "{}", String::from_utf8_lossy(&run.stderr));
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

fn host_triple() -> String {
    let output = Command::new(tool("rustc")).arg("-vV").output().unwrap();
    let version = String::from_utf8(output.stdout).unwrap();
    version.lines().find_map(|line| line.strip_prefix("host: ")).unwrap().to_string()
}

fn mode_helper(mode: &str, out: &Path, extra: &[(&str, String)]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_mantle-cargo-unit-helper"))
        .arg(mode)
        .env("out", out)
        .env("CARGO_HOME", "/nonexistent/ambient-cargo-state")
        .env("AMBIENT_LEAK", "1")
        .envs(extra.iter().map(|(key, value)| (*key, value)))
        .output()
        .unwrap()
}

fn target_facts(store: &Path) -> PathBuf {
    let facts = store.join("ffffffffffffffffffffffffffffffff-target-facts");
    let host = host_triple();
    let output = mode_helper("target-facts", &facts, &[
        ("MANTLE_UNIT_RUSTC", tool("rustc").display().to_string()),
        ("MANTLE_UNIT_TARGET", host.clone()),
        ("MANTLE_UNIT_HOST", host),
    ]);
    assert!(output.status.success(), "target facts failed: {}", String::from_utf8_lossy(&output.stderr));
    facts
}

fn compile_script(store: &Path, source: &Path, name: &str) -> PathBuf {
    let out = store.join(format!("{}-{name}-script", "d".repeat(32)));
    let output = helper(&out, source, &[], json!({}), &[
        "--crate-name=build_script_build",
        "--crate-type=bin",
        "--edition=2024",
        source.join("build.rs").to_str().unwrap(),
    ]);
    assert!(output.status.success(), "script compile failed: {}", String::from_utf8_lossy(&output.stderr));
    out
}

struct ScriptRun<'a> {
    name: &'a str,
    source: &'a Path,
    script: &'a Path,
    facts: &'a Path,
    script_env: Value,
    links: Vec<String>,
    native_inputs: Vec<String>,
}

fn run_script(store: &Path, run: &ScriptRun<'_>) -> (std::process::Output, PathBuf, PathBuf) {
    let out = store.join(format!("{}-{}-run", "e".repeat(32), run.name));
    let flags = store.join(format!("{}-{}-run-flags", "e".repeat(32), run.name));
    let output = mode_helper("run-build-script", &out, &[
        ("flags", flags.display().to_string()),
        ("MANTLE_UNIT_SOURCE", run.source.display().to_string()),
        ("MANTLE_UNIT_BUILD_SCRIPT", run.script.display().to_string()),
        ("MANTLE_UNIT_RUSTC", tool("rustc").display().to_string()),
        ("MANTLE_UNIT_TARGET_FACTS", run.facts.display().to_string()),
        ("MANTLE_UNIT_SCRIPT_ENV", run.script_env.to_string()),
        ("MANTLE_UNIT_SCRIPT_PATH", "[]".into()),
        ("MANTLE_UNIT_NATIVE_INPUTS", serde_json::to_string(&run.native_inputs).unwrap()),
        ("MANTLE_UNIT_LINKS_DEPENDENCIES", serde_json::to_string(&run.links).unwrap()),
    ]);
    (output, out, flags)
}

#[test]
fn declared_native_archive_and_release_profile_reach_real_consumer() {
    let fixture = fixture_root("native-release");
    let store = fixture.join("store");
    let source = store.join(format!("{}-source", "a".repeat(32)));
    let native = store.join(format!("{}-native", "b".repeat(32)));
    fs::create_dir_all(&source).unwrap();
    fs::create_dir_all(native.join("lib")).unwrap();
    fs::write(source.join("answer.c"), "int answer(void) { return 42; }\n").unwrap();
    let cc = Command::new(tool("clang"))
        .arg("-c")
        .arg(source.join("answer.c"))
        .arg("-o")
        .arg(source.join("answer.o"))
        .output()
        .unwrap();
    assert!(cc.status.success(), "{cc:?}");
    let archive = Command::new(tool("ar"))
        .arg("rcs")
        .arg(native.join("lib/libanswer.a"))
        .arg(source.join("answer.o"))
        .output()
        .unwrap();
    assert!(archive.status.success(), "{archive:?}");
    fs::write(
        source.join("build.rs"),
        r#"fn main() {
        assert_eq!(std::env::var("PROFILE").unwrap(), "release");
        assert_eq!(std::env::var("OPT_LEVEL").unwrap(), "2");
        assert_eq!(std::env::var("DEBUG").unwrap(), "true");
        let root = std::env::var("LIBRARY_DIR").expect("declared native library");
        println!("cargo:rustc-link-search=native={root}/lib");
        println!("cargo:rustc-link-lib=static=answer");
    }"#,
    )
    .unwrap();
    let facts = target_facts(&store);
    let script = compile_script(&store, &source, "native");
    let (output, out, flags) = run_script(&store, &ScriptRun {
        name: "native",
        source: &source,
        script: &script,
        facts: &facts,
        script_env: json!({"PROFILE":"release", "OPT_LEVEL":"2", "DEBUG":"true", "LIBRARY_DIR":native.display().to_string()}),
        links: Vec::new(),
        native_inputs: vec![native.display().to_string()],
    });
    assert!(output.status.success(), "native script: {}", String::from_utf8_lossy(&output.stderr));
    fs::write(
        source.join("lib.rs"),
        "unsafe extern \"C\" { fn answer() -> i32; } pub fn value() -> i32 { unsafe { answer() } }\n",
    )
    .unwrap();
    let lib = store.join(format!("{}-sys", "c".repeat(32)));
    let built = helper_with(
        &lib,
        &source,
        &[],
        json!({}),
        &[
            "--crate-name=sys",
            "--crate-type=rlib",
            "--edition=2024",
            source.join("lib.rs").to_str().unwrap(),
        ],
        &[
            ("MANTLE_UNIT_BUILD_SCRIPT_OUT", out.display().to_string()),
            ("MANTLE_UNIT_BUILD_SCRIPT_FLAGS", flags.display().to_string()),
            ("MANTLE_UNIT_TARGET_KIND", "lib".into()),
            ("MANTLE_UNIT_TARGET_NAME", "sys".into()),
        ],
    );
    assert!(built.status.success(), "sys compile: {}", String::from_utf8_lossy(&built.stderr));
    fs::write(source.join("app.rs"), "fn main() { println!(\"native-answer={}\", sys::value()); }\n").unwrap();
    let app = store.join(format!("{}-app", "c".repeat(32)));
    let built = helper_with(
        &app,
        &source,
        &[&lib],
        json!({"sys":lib.display().to_string()}),
        &[
            "--crate-name=app",
            "--crate-type=bin",
            "--edition=2024",
            source.join("app.rs").to_str().unwrap(),
        ],
        &[("MANTLE_UNIT_TRANSITIVE_FLAGS", json!([flags.display().to_string()]).to_string())],
    );
    assert!(built.status.success(), "consumer compile: {}", String::from_utf8_lossy(&built.stderr));
    let executed = Command::new(app.join("lib/app")).output().unwrap();
    assert!(executed.status.success(), "{executed:?}");
    assert_eq!(executed.stdout, b"native-answer=42\n");
    let (missing, _, absent_flags) = run_script(&store, &ScriptRun {
        name: "missing-native",
        source: &source,
        script: &script,
        facts: &facts,
        script_env: json!({"PROFILE":"release", "OPT_LEVEL":"2", "DEBUG":"true"}),
        links: Vec::new(),
        native_inputs: Vec::new(),
    });
    assert!(!missing.status.success());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("build-script-nonzero-exit"));
    assert!(!absent_flags.exists());
    fs::remove_dir_all(fixture).unwrap();
}

#[test]
fn target_facts_mode_records_typed_rustc_cfg_and_checks_the_host() {
    let fixture = fixture_root("target-facts");
    let store = fixture.join("store");
    let facts = target_facts(&store);
    let written = fs::read_to_string(facts.join("target-facts.env")).unwrap();
    let host = host_triple();
    assert!(written.starts_with(&format!("TARGET={host}\nHOST={host}\n")), "{written}");
    assert!(written.contains("CARGO_CFG_TARGET_OS=linux\n"));
    assert!(!written.contains("CARGO_CFG_DEBUG_ASSERTIONS"));
    let mismatch = mode_helper("target-facts", &store.join("ffffffffffffffffffffffffffffffff-other"), &[
        ("MANTLE_UNIT_RUSTC", tool("rustc").display().to_string()),
        ("MANTLE_UNIT_TARGET", host),
        ("MANTLE_UNIT_HOST", "not-the-host".into()),
    ]);
    assert!(!mismatch.status.success());
    assert!(String::from_utf8_lossy(&mismatch.stderr).contains("unit-helper-host-mismatch"));
    fs::remove_dir_all(fixture).unwrap();
}

const SCRIPT_A: &str = r#"
use std::{env, fs, path::Path};
fn main() {
    let out = env::var("OUT_DIR").unwrap();
    assert!(env::var_os("CARGO_HOME").is_none() && env::var_os("AMBIENT_LEAK").is_none(), "ambient environment leaked");
    assert_eq!(env::var("TARGET").unwrap(), env::var("HOST").unwrap());
    assert_eq!(env::var("CARGO_CFG_TARGET_OS").unwrap(), "linux");
    assert_eq!(env::var("CARGO_PKG_NAME").unwrap(), "a");
    fs::write(Path::new(&out).join("generated.rs"), "pub const GENERATED: u32 = 40;").unwrap();
    fs::create_dir_all(Path::new(&out).join("include")).unwrap();
    println!("cargo::rustc-cfg=generated");
    println!("cargo::rustc-check-cfg=cfg(generated)");
    println!("cargo:rustc-env=GEN_EXTRA=2");
    println!("cargo::metadata=include={out}/include");
    println!("cargo:rustc-link-search=native={out}/include");
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::warning=generated code");
}
"#;

const SCRIPT_B: &str = r#"
fn main() {
    let include = std::env::var("DEP_FOO_INCLUDE").expect("links metadata from a");
    assert!(std::path::Path::new(&include).is_dir());
    println!("cargo:rustc-env=SEEN_INCLUDE={include}");
}
"#;

#[test]
fn build_script_output_reaches_dependent_compilations_and_scripts() {
    let fixture = fixture_root("build-script");
    let store = fixture.join("store");
    let source_a = store.join(format!("{}-source-a", "a".repeat(32)));
    let source_b = store.join(format!("{}-source-b", "b".repeat(32)));
    fs::create_dir_all(&source_a).unwrap();
    fs::create_dir_all(&source_b).unwrap();
    fs::write(source_a.join("build.rs"), SCRIPT_A).unwrap();
    fs::write(
        source_a.join("lib.rs"),
        "include!(concat!(env!(\"OUT_DIR\"), \"/generated.rs\"));\n#[cfg(generated)]\npub fn answer() -> u32 { GENERATED + env!(\"GEN_EXTRA\").parse::<u32>().unwrap() }\n",
    )
    .unwrap();
    fs::write(source_b.join("build.rs"), SCRIPT_B).unwrap();
    fs::write(source_b.join("app.rs"), "fn main() { assert_eq!(a::answer(), 42); }\n").unwrap();
    let facts = target_facts(&store);
    let script_a = compile_script(&store, &source_a, "a");
    let (output, out_a, flags_a) = run_script(&store, &ScriptRun {
        name: "a",
        source: &source_a,
        script: &script_a,
        facts: &facts,
        script_env: json!({"CARGO_PKG_NAME": "a", "CARGO_MANIFEST_LINKS": "foo"}),
        links: Vec::new(),
        native_inputs: Vec::new(),
    });
    assert!(output.status.success(), "script a failed: {}", String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stderr).contains("warning: generated code"));
    let include = format!("{}/include", out_a.display());
    assert!(fs::read_to_string(flags_a.join("args-immediate")).unwrap().starts_with("all\t--cfg=generated\n"));
    assert_eq!(fs::read_to_string(flags_a.join("args-transitive")).unwrap(), format!("-Lnative={include}\n"));
    assert_eq!(fs::read_to_string(flags_a.join("links")).unwrap(), format!("DEP_FOO_INCLUDE={include}\n"));
    assert_eq!(fs::read_to_string(flags_a.join("rerun")).unwrap(), "rerun-if-changed=build.rs\n");

    let lib_a = store.join(format!("{}-a-lib", "c".repeat(32)));
    let output = helper_with(
        &lib_a,
        &source_a,
        &[],
        json!({}),
        &[
            "--crate-name=a",
            "--crate-type=rlib",
            "--edition=2024",
            source_a.join("lib.rs").to_str().unwrap(),
        ],
        &[
            ("MANTLE_UNIT_BUILD_SCRIPT_OUT", out_a.display().to_string()),
            ("MANTLE_UNIT_BUILD_SCRIPT_FLAGS", flags_a.display().to_string()),
            ("MANTLE_UNIT_TRANSITIVE_FLAGS", "[]".into()),
            ("MANTLE_UNIT_TARGET_KIND", "lib".into()),
            ("MANTLE_UNIT_TARGET_NAME", "a".into()),
        ],
    );
    assert!(output.status.success(), "lib a failed: {}", String::from_utf8_lossy(&output.stderr));

    let script_b = compile_script(&store, &source_b, "b");
    let (output, _, flags_b) = run_script(&store, &ScriptRun {
        name: "b",
        source: &source_b,
        script: &script_b,
        facts: &facts,
        script_env: json!({"CARGO_PKG_NAME": "b"}),
        links: vec![flags_a.display().to_string()],
        native_inputs: Vec::new(),
    });
    assert!(output.status.success(), "script b failed: {}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(fs::read_to_string(flags_b.join("env")).unwrap(), format!("SEEN_INCLUDE={include}\n"));

    let app = store.join(format!("{}-app", "c".repeat(32)));
    let transitive = serde_json::to_string(&vec![flags_a.display().to_string()]).unwrap();
    let output = helper_with(
        &app,
        &source_b,
        &[&lib_a],
        json!({"a": lib_a.display().to_string()}),
        &[
            "--crate-name=app",
            "--crate-type=bin",
            "--edition=2024",
            source_b.join("app.rs").to_str().unwrap(),
        ],
        &[
            ("MANTLE_UNIT_TRANSITIVE_FLAGS", transitive),
            ("MANTLE_UNIT_TARGET_KIND", "bin".into()),
            ("MANTLE_UNIT_TARGET_NAME", "app".into()),
        ],
    );
    assert!(output.status.success(), "app failed: {}", String::from_utf8_lossy(&output.stderr));
    let built = Command::new(app.join("lib/app")).output().unwrap();
    assert!(built.status.success(), "compiled app exited {built:?}");
    fs::remove_dir_all(fixture).unwrap();
}

const SCRIPT_MODES: &str = r#"
fn main() {
    match std::env::var("MODE").unwrap().as_str() {
        "exit" => { eprintln!("script refused"); std::process::exit(3) }
        "unknown" => println!("cargo::rustc-unknown=1"),
        "error" => println!("cargo::error=libfoo is missing"),
        "host-search" => println!("cargo:rustc-link-search=native=/usr/lib"),
        _ => println!("cargo:rustc-cfg=ok"),
    }
}
"#;

#[test]
fn failing_and_unsupported_build_scripts_fail_their_unit() {
    let fixture = fixture_root("build-script-negative");
    let store = fixture.join("store");
    let source = store.join(format!("{}-source", "a".repeat(32)));
    fs::create_dir_all(&source).unwrap();
    fs::write(source.join("build.rs"), SCRIPT_MODES).unwrap();
    let facts = target_facts(&store);
    let script = compile_script(&store, &source, "modes");
    for (mode, code, detail) in [
        ("exit", "build-script-nonzero-exit", "script refused"),
        ("unknown", "build-script-directive-unsupported", "rustc-unknown"),
        ("error", "build-script-error-directive", "libfoo is missing"),
        ("host-search", "build-script-directive-unsupported", "rustc-link-search"),
    ] {
        let (output, _, flags) = run_script(&store, &ScriptRun {
            name: mode,
            source: &source,
            script: &script,
            facts: &facts,
            script_env: json!({ "MODE": mode }),
            links: Vec::new(),
            native_inputs: Vec::new(),
        });
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success(), "{mode} unexpectedly succeeded");
        assert!(stderr.contains(code) && stderr.contains(detail), "{mode}: {stderr}");
        assert!(!flags.exists(), "{mode} published flags");
    }
    let (output, _, flags) = run_script(&store, &ScriptRun {
        name: "ok",
        source: &source,
        script: &script,
        facts: &facts,
        script_env: json!({ "MODE": "ok", "OUT_DIR": "/spoofed" }),
        links: Vec::new(),
        native_inputs: Vec::new(),
    });
    assert!(!output.status.success(), "lane-owned OUT_DIR must not be declared by the script environment");
    assert!(!flags.exists());
    fs::remove_dir_all(fixture).unwrap();
}

#[test]
fn over_limit_build_script_is_stopped_without_waiting_for_its_completion() {
    for stream in ["stdout", "stderr"] {
        over_limit_build_script_stream(stream);
    }
}

fn over_limit_build_script_stream(stream: &str) {
    let fixture = fixture_root(&format!("build-script-{stream}-limit"));
    let store = fixture.join("store");
    let source = store.join(format!("{}-source", "a".repeat(32)));
    fs::create_dir_all(&source).unwrap();
    fs::write(
        source.join("build.rs"),
        r#"
use std::{env, io::{self, Write}, path::Path, thread, time::Duration};
fn main() {
    let bytes = vec![b'x'; 8 * 1024 * 1024 + 1];
    let mut pipe: Box<dyn Write> = if env::var("STREAM").unwrap() == "stderr" {
        Box::new(io::stderr())
    } else {
        Box::new(io::stdout())
    };
    pipe.write_all(&bytes).unwrap();
    pipe.flush().unwrap();
    let release = env::var("RELEASE_FILE").unwrap();
    while !Path::new(&release).exists() {
        thread::sleep(Duration::from_millis(10));
    }
}
"#,
    )
    .unwrap();
    let script = compile_script(&store, &source, "flood");
    let facts = target_facts(&store);
    let out = store.join(format!("{}-flood-run", "e".repeat(32)));
    let flags = store.join(format!("{}-flood-flags", "e".repeat(32)));
    let release = fixture.join("release-script");
    let mut child = Command::new(env!("CARGO_BIN_EXE_mantle-cargo-unit-helper"))
        .arg("run-build-script")
        .env("out", &out)
        .env("flags", &flags)
        .env("MANTLE_UNIT_SOURCE", &source)
        .env("MANTLE_UNIT_BUILD_SCRIPT", &script)
        .env("MANTLE_UNIT_RUSTC", tool("rustc"))
        .env("MANTLE_UNIT_TARGET_FACTS", &facts)
        .env(
            "MANTLE_UNIT_SCRIPT_ENV",
            json!({"RELEASE_FILE": release.display().to_string(), "STREAM": stream}).to_string(),
        )
        .env("MANTLE_UNIT_SCRIPT_PATH", "[]")
        .env("MANTLE_UNIT_NATIVE_INPUTS", "[]")
        .env("MANTLE_UNIT_LINKS_DEPENDENCIES", "[]")
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        if child.try_wait().unwrap().is_some() {
            let output = child.wait_with_output().unwrap();
            assert!(!output.status.success());
            assert!(
                String::from_utf8_lossy(&output.stderr).contains("build-script-output-limit"),
                "expected bounded output rejection: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(!flags.exists(), "over-limit script published flags");
            break;
        }
        if Instant::now() >= deadline {
            fs::write(&release, "release").unwrap();
            let output = child.wait_with_output().unwrap();
            fs::remove_dir_all(&fixture).unwrap();
            panic!(
                "helper waited for unbounded script after exceeding output limit: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    fs::remove_dir_all(fixture).unwrap();
}

#[test]
fn build_script_cannot_probe_network_or_undeclared_host_path_in_native_namespace() {
    // Rebind the fixture after masking /tmp; the sibling host secret stays unmounted.
    let fixture = fixture_root("sandbox-boundary");
    let sandbox_helper = fixture.join("unit-helper");
    fs::copy(env!("CARGO_BIN_EXE_mantle-cargo-unit-helper"), &sandbox_helper).unwrap();
    let sandbox_true = fixture.join("true");
    fs::copy(tool("true"), &sandbox_true).unwrap();
    let store = fixture.join("store");
    let source = store.join(format!("{}-source", "a".repeat(32)));
    fs::create_dir_all(&source).unwrap();
    fs::write(
        source.join("build.rs"),
        r#"
fn main() {
    assert!(std::fs::read_to_string(std::env::var("PROBE_PATH").unwrap()).is_err(),
        "undeclared host path became visible");
    assert!(std::net::TcpStream::connect(std::env::var("PROBE_ADDR").unwrap()).is_err(),
        "network became visible");
    println!("cargo::rustc-cfg=sandboxed");
}
"#,
    )
    .unwrap();
    let secret = std::env::temp_dir().join(format!("mantle-build-script-host-secret-{}", std::process::id()));
    fs::write(&secret, "host-only").unwrap();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    std::net::TcpStream::connect(listener.local_addr().unwrap()).expect("sandbox control must be reachable on host");
    let script = compile_script(&store, &source, "sandbox");
    let facts = target_facts(&store);
    let out = store.join(format!("{}-script-run", "e".repeat(32)));
    let flags = store.join(format!("{}-script-flags", "e".repeat(32)));
    let mount = || {
        let mut command = Command::new("bwrap");
        command
            .args([
                "--ro-bind",
                "/nix/store",
                "/nix/store",
                "--tmpfs",
                "/tmp",
                "--proc",
                "/proc",
                "--dev",
                "/dev",
                "--bind",
            ])
            .arg(&fixture)
            .arg(&fixture)
            .args(["--unshare-net", "--"]);
        command
    };
    let preflight = mount().arg(&sandbox_true).output();
    if !preflight.as_ref().is_ok_and(|result| result.status.success()) {
        eprintln!("skipping: native bwrap user namespaces unavailable");
        fs::remove_file(secret).unwrap();
        fs::remove_dir_all(fixture).unwrap();
        return;
    }
    let result = mount()
        .arg(&sandbox_helper)
        .arg("run-build-script")
        .env("out", &out)
        .env("flags", &flags)
        .env("MANTLE_UNIT_SOURCE", &source)
        .env("MANTLE_UNIT_BUILD_SCRIPT", &script)
        .env("MANTLE_UNIT_RUSTC", tool("rustc"))
        .env("MANTLE_UNIT_TARGET_FACTS", &facts)
        .env(
            "MANTLE_UNIT_SCRIPT_ENV",
            json!({
                "PROBE_PATH":secret.to_string_lossy(),
                "PROBE_ADDR":listener.local_addr().unwrap().to_string()
            })
            .to_string(),
        )
        .env("MANTLE_UNIT_SCRIPT_PATH", "[]")
        .env("MANTLE_UNIT_NATIVE_INPUTS", "[]")
        .env("MANTLE_UNIT_LINKS_DEPENDENCIES", "[]")
        .output()
        .unwrap();
    assert!(result.status.success(), "sandboxed script failed: {}", String::from_utf8_lossy(&result.stderr));
    assert_eq!(fs::read_to_string(flags.join("args-immediate")).unwrap(), "all\t--cfg=sandboxed\n");
    fs::remove_file(secret).unwrap();
    fs::remove_dir_all(fixture).unwrap();
}
