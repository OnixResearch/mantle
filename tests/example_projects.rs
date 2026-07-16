use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command as StdCommand;

use assert_cmd::Command;

const PROJECTS_ROOT: &str = "examples/projects";
const GENERATED_SITE_PROJECT: &str = "generated-site";
const CODEGEN_PROJECT: &str = "codegen-pipeline";
const C_PROJECT: &str = "c-library-cli";
const RUST_PROJECT: &str = "rust-workspace";
const FETCHED_PATCHED_PROJECT: &str = "fetched-and-patched";
const MULTI_OUTPUT_SDK_PROJECT: &str = "multi-output-sdk";
const SCHEMA_CODEGEN_PROJECT: &str = "schema-codegen";
const REPRODUCIBLE_RELEASE_PROJECT: &str = "reproducible-release";
const SIGNED_CACHE_PROJECT: &str = "signed-cache-roundtrip";
const LOCKED_DEPENDENCY_PROJECT: &str = "locked-dependency-lifecycle";
const CROSS_COMPILED_PROJECT: &str = "cross-compiled-host-tool";
const STORE_GC_PROJECT: &str = "store-gc-lifecycle";
const DELTA_SUBSTITUTION_PROJECT: &str = "delta-substitution";
const RELEASE_WITNESS_PROJECT: &str = "release-witness-handoff";
const LOCK_RETENTION_ROOT_RECORD: &str =
    ".mantle/retention-roots/9dcaf0da80828fc8c2c8d41160c0d12f98bb1abc03c1d1f611bc534b579c00ad.json";
const SITE_TITLE: &str = "Mantle Project Gallery";
const SITE_CHECK_RESULT: &str = "generated-site-check: ok";
const CODEGEN_MESSAGE: &str = "Hello from generated project code";
const CODEGEN_CHECK_RESULT: &str = "codegen-demo-check: ok";
const C_GREETING: &str = "Hello, Mantle!";
const C_CHECK_RESULT: &str = "c-library-cli-check: ok";
const C_POSITIVE_TEST: &str = "positive_status == GREET_STATUS_OK";
const C_NEGATIVE_TEST: &str = "zero_capacity_status == GREET_STATUS_INVALID_ARGUMENT";
const RUST_POSITIVE_TEST: &str = "renders_a_normalized_name";
const RUST_NEGATIVE_TEST: &str = "rejects_an_empty_name";
const RUST_GREETING: &str = "Hello, Mantle!";
const RUST_CHECK_RESULT: &str = "rust-workspace-smoke-check: ok";
const FETCHED_PATCHED_CHECK_RESULT: &str = "fetched-and-patched-check: ok";
const MULTI_OUTPUT_RUNTIME_CHECK_RESULT: &str = "multi-output-sdk-runtime-check: ok";
const MULTI_OUTPUT_DEVELOPMENT_CHECK_RESULT: &str = "multi-output-sdk-development-check: ok";
const SCHEMA_CODEGEN_CHECK_RESULT: &str = "schema-codegen-integration-check: ok";
const RELEASE_CHECK_RESULT: &str = "reproducible-release-check: ok";
const RELEASE_TAMPER_CHECK_RESULT: &str = "reproducible-release-tamper-check: ok";
const RELEASE_ARCHIVE_BLAKE3: &str = "blake3-Roawb3qp5wK7exFYdIBJiEwtIjGz2YivY1sVtEI+YDI=";
const MISSING_SELECTOR: &str = ".#missing-package";
const DEFAULT_OUTPUT_LABEL: &str = "out";
const CACHED_OUTPUT_ANNOTATION: &str = "cached";
const MAX_PROJECT_OUTPUTS: usize = 16;

fn mantle_cmd() -> Command {
    Command::cargo_bin("mantle").expect("mantle binary should be built")
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn project_root(project: &str) -> PathBuf {
    repo_root().join(PROJECTS_ROOT).join(project)
}

fn find_bwrap() -> Option<PathBuf> {
    let path_dirs = std::env::var_os("PATH")?;
    for directory in std::env::split_paths(&path_dirs) {
        let candidate = directory.join("bwrap");
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn find_static_busybox() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("SNIX_BUILD_SANDBOX_SHELL") {
        let candidate = PathBuf::from(path);
        if candidate.is_file() && candidate.to_string_lossy().contains("busybox-static") {
            return Some(candidate);
        }
    }

    let entries = std::fs::read_dir("/nix/store").ok()?;
    for entry in entries.flatten() {
        let candidate = entry.path().join("bin/busybox");
        if candidate.is_file() && candidate.to_string_lossy().contains("busybox-static") {
            return Some(candidate);
        }
    }
    None
}

fn can_build_fast_projects() -> bool {
    cfg!(target_os = "linux")
        && Path::new("/nix/store").is_dir()
        && find_bwrap().is_some()
        && find_static_busybox().is_some()
}

struct ProjectRun {
    _store: tempfile::TempDir,
    _state: tempfile::TempDir,
    output: std::process::Output,
}

fn run_project_build(project: &str, selector: &str) -> ProjectRun {
    run_project_build_with_verbosity(project, selector, false)
}

fn run_project_build_verbose(project: &str, selector: &str) -> ProjectRun {
    run_project_build_with_verbosity(project, selector, true)
}

fn run_project_build_with_verbosity(project: &str, selector: &str, verbose: bool) -> ProjectRun {
    let store = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    let mut command = mantle_cmd();
    command.current_dir(project_root(project));
    if verbose {
        command.arg("--verbose");
    }
    command.args([
        "build",
        selector,
        "--store",
        store.path().to_str().unwrap(),
        "--state-dir",
        state.path().to_str().unwrap(),
        "--no-substitute",
    ]);
    if let Some(bwrap) = find_bwrap() {
        let path = std::env::join_paths(
            std::iter::once(bwrap.parent().unwrap().to_path_buf())
                .chain(std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())),
        )
        .unwrap();
        command.env("PATH", path);
    }
    if let Some(busybox) = find_static_busybox() {
        command.env("SNIX_BUILD_SANDBOX_SHELL", busybox);
    }
    command.env("CRUNCH_NO_FUSE", "1");
    let output = command.output().expect("project build should run");
    ProjectRun {
        _store: store,
        _state: state,
        output,
    }
}

fn parse_output_paths(stdout: &[u8]) -> Result<BTreeMap<String, PathBuf>, String> {
    let text = std::str::from_utf8(stdout).map_err(|error| format!("build output is not UTF-8: {error}"))?;
    let mut paths = BTreeMap::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        if paths.len() >= MAX_PROJECT_OUTPUTS {
            return Err(format!("build reported more than {MAX_PROJECT_OUTPUTS} outputs"));
        }
        let (path, label) = match line.rsplit_once(" (") {
            Some((path, suffix)) => {
                let annotations = suffix.strip_suffix(')').ok_or_else(|| format!("malformed output label: {line}"))?;
                let first_annotation = annotations.split(',').next().unwrap_or_default().trim();
                let label = if first_annotation == CACHED_OUTPUT_ANNOTATION {
                    DEFAULT_OUTPUT_LABEL
                } else {
                    first_annotation
                };
                (path, label)
            }
            None => (line, DEFAULT_OUTPUT_LABEL),
        };
        if path.is_empty() || label.is_empty() {
            return Err(format!("empty output path or label: {line}"));
        }
        let path = PathBuf::from(path);
        if !path.is_absolute() {
            return Err(format!("output path is not absolute: {}", path.display()));
        }
        if paths.insert(label.to_string(), path).is_some() {
            return Err(format!("duplicate output label: {label}"));
        }
    }
    if paths.is_empty() {
        return Err("build did not report an output path".to_string());
    }
    Ok(paths)
}

fn output_path_for_label(run: &ProjectRun, label: &str) -> PathBuf {
    parse_output_paths(&run.output.stdout)
        .unwrap()
        .remove(label)
        .unwrap_or_else(|| panic!("missing output label {label}"))
}

fn output_path(run: &ProjectRun) -> PathBuf {
    output_path_for_label(run, DEFAULT_OUTPUT_LABEL)
}

fn assert_success(run: &ProjectRun, label: &str) {
    let stderr = String::from_utf8_lossy(&run.output.stderr);
    assert!(run.output.status.success(), "{label} failed:\n{stderr}");
    assert!(!run.output.stdout.is_empty(), "{label} should print an output path");
}

fn assert_project_evaluates(project: &str, label: &str) {
    let mut command = mantle_cmd();
    let output = command.current_dir(project_root(project)).args(["eval", "mantle-project.ncl"]).output().unwrap();
    assert!(output.status.success(), "{label} should evaluate: {}", String::from_utf8_lossy(&output.stderr));
    assert!(!output.stdout.is_empty(), "{label} evaluation should export JSON");
}

fn copy_locked_dependency_project(destination: &Path) {
    const PROJECT_FILES: &[&str] = &[
        "mantle-project.ncl",
        "mantle.lock",
        ".mantle/inputs.ncl",
        ".mantle/retention.json",
        LOCK_RETENTION_ROOT_RECORD,
        "fixtures/unresolved-revision.ncl",
        "patches/message.patch",
        "sources/message.txt",
    ];
    assert!(destination.is_dir(), "fixture destination must exist");
    assert!(!PROJECT_FILES.is_empty(), "locked fixture must copy project files");
    let source = project_root(LOCKED_DEPENDENCY_PROJECT);
    for relative_path in PROJECT_FILES {
        let target = destination.join(relative_path);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::copy(source.join(relative_path), target).unwrap();
    }
}

fn run_project_command(current_dir: &Path, args: &[&str]) -> std::process::Output {
    assert!(current_dir.is_dir(), "project command directory must exist");
    assert!(!args.is_empty(), "project command arguments must not be empty");
    let mut command = mantle_cmd();
    command.current_dir(current_dir).args(args).output().unwrap()
}

fn check_result(run: &ProjectRun) -> String {
    std::fs::read_to_string(output_path(run).join("result.txt")).unwrap().trim().to_string()
}

#[test]
fn project_output_parser_accepts_named_outputs_and_rejects_duplicates() {
    let parsed =
        parse_output_paths(b"/tmp/example-dev (dev, cached)\n/tmp/example-doc (doc)\n/tmp/example-out (cached)\n")
            .unwrap();
    assert_eq!(parsed.get("dev"), Some(&PathBuf::from("/tmp/example-dev")));
    assert_eq!(parsed.get("doc"), Some(&PathBuf::from("/tmp/example-doc")));
    assert_eq!(parsed.get(DEFAULT_OUTPUT_LABEL), Some(&PathBuf::from("/tmp/example-out")));

    let duplicate = parse_output_paths(b"/tmp/first\n/tmp/second\n").unwrap_err();
    assert!(duplicate.contains("duplicate output label"), "unexpected duplicate error: {duplicate}");
    let relative = parse_output_paths(b"relative-output\n").unwrap_err();
    assert!(relative.contains("not absolute"), "unexpected relative path error: {relative}");
}

#[test]
fn production_workflow_projects_evaluate_and_keep_negative_paths() {
    for (project, label) in [
        (SIGNED_CACHE_PROJECT, "signed cache project"),
        (LOCKED_DEPENDENCY_PROJECT, "locked dependency project"),
        (CROSS_COMPILED_PROJECT, "cross-compiled project"),
        (STORE_GC_PROJECT, "store GC project"),
        (DELTA_SUBSTITUTION_PROJECT, "delta substitution project"),
        (RELEASE_WITNESS_PROJECT, "release witness project"),
    ] {
        assert_project_evaluates(project, label);
    }

    let cache = std::fs::read_to_string(project_root(SIGNED_CACHE_PROJECT).join("README.md")).unwrap();
    assert!(cache.contains("unknown signer is skipped"));
    assert!(cache.contains("wrong key material fails signature verification"));
    assert!(cache.contains("corrupted NAR is rejected"));
    let cross = std::fs::read_to_string(project_root(CROSS_COMPILED_PROJECT).join("mantle-project.ncl")).unwrap();
    assert!(cross.contains("host/target role mismatch"));
    assert!(cross.contains("-x c -std=c11"));
    let gc = std::fs::read_to_string(project_root(STORE_GC_PROJECT).join("mantle-project.ncl")).unwrap();
    assert!(gc.contains("lock-holder"));
    let delta = std::fs::read_to_string(project_root(DELTA_SUBSTITUTION_PROJECT).join("demo.rs")).unwrap();
    assert!(delta.contains("FullArtifactFallback"));
    assert!(delta.contains("MissingSenderChunk"));
    let release = std::fs::read_to_string(project_root(RELEASE_WITNESS_PROJECT).join("demo.rs")).unwrap();
    assert!(release.contains("InsufficientQuorum"));
    assert!(release.contains("ReleaseRevocations"));
}

#[test]
fn locked_dependency_lifecycle_detects_staleness_refreshes_and_upgrades() {
    let fixture = tempfile::tempdir().unwrap();
    copy_locked_dependency_project(fixture.path());
    let check = run_project_command(fixture.path(), &["check"]);
    assert!(check.status.success(), "initial check failed: {}", String::from_utf8_lossy(&check.stderr));

    let lock_before = std::fs::read(fixture.path().join("mantle.lock")).unwrap();
    let retention_path = fixture.path().join(".mantle/retention.json");
    assert!(fixture.path().join(LOCK_RETENTION_ROOT_RECORD).is_file());
    let retention_before: serde_json::Value = serde_json::from_slice(&std::fs::read(&retention_path).unwrap()).unwrap();
    std::fs::OpenOptions::new()
        .append(true)
        .open(fixture.path().join("sources/message.txt"))
        .unwrap()
        .write_all(b"changed locally\n")
        .unwrap();
    let stale = run_project_command(fixture.path(), &["list-stale", "--no-network"]);
    assert!(stale.status.success(), "stale listing failed: {}", String::from_utf8_lossy(&stale.stderr));
    assert!(String::from_utf8_lossy(&stale.stdout).contains("message-source"));
    assert_eq!(std::fs::read(fixture.path().join("mantle.lock")).unwrap(), lock_before);

    let refresh = run_project_command(fixture.path(), &["refresh", "--no-network", "message-source"]);
    assert!(refresh.status.success(), "refresh failed: {}", String::from_utf8_lossy(&refresh.stderr));
    assert_ne!(std::fs::read(fixture.path().join("mantle.lock")).unwrap(), lock_before);
    let retention_after: serde_json::Value = serde_json::from_slice(&std::fs::read(&retention_path).unwrap()).unwrap();
    assert_ne!(retention_after["records"][0]["content_digest"], retention_before["records"][0]["content_digest"]);
    assert_eq!(
        retention_after["records"][0]["generation"].as_u64(),
        retention_before["records"][0]["generation"]
            .as_u64()
            .and_then(|generation| generation.checked_add(1))
    );
    let no_stale = run_project_command(fixture.path(), &["list-stale", "--no-network"]);
    assert!(no_stale.status.success());
    assert_eq!(String::from_utf8_lossy(&no_stale.stdout).trim(), "all inputs up to date");

    let lock_path = fixture.path().join("mantle.lock");
    let mut old_lock: serde_json::Value = serde_json::from_slice(&std::fs::read(&lock_path).unwrap()).unwrap();
    old_lock["version"] = serde_json::Value::String("0.9.0".to_string());
    std::fs::write(&lock_path, serde_json::to_vec_pretty(&old_lock).unwrap()).unwrap();
    let upgrade = run_project_command(fixture.path(), &["upgrade"]);
    assert!(upgrade.status.success(), "upgrade failed: {}", String::from_utf8_lossy(&upgrade.stderr));
    let upgraded: serde_json::Value = serde_json::from_slice(&std::fs::read(&lock_path).unwrap()).unwrap();
    assert_eq!(upgraded["version"], "1.0.0");
}

#[test]
fn locked_dependency_refresh_rejects_a_missing_local_patch() {
    let fixture = tempfile::tempdir().unwrap();
    copy_locked_dependency_project(fixture.path());
    let lock_path = fixture.path().join("mantle.lock");
    let mut lock: serde_json::Value = serde_json::from_slice(&std::fs::read(&lock_path).unwrap()).unwrap();
    lock["patches"] = serde_json::json!({});
    std::fs::write(&lock_path, serde_json::to_vec_pretty(&lock).unwrap()).unwrap();
    std::fs::remove_file(fixture.path().join("patches/message.patch")).unwrap();
    std::fs::OpenOptions::new()
        .append(true)
        .open(fixture.path().join("sources/message.txt"))
        .unwrap()
        .write_all(b"force refresh\n")
        .unwrap();

    let refresh = run_project_command(fixture.path(), &["refresh", "--no-network", "message-source"]);

    assert!(!refresh.status.success(), "missing patch unexpectedly refreshed");
    let stderr = String::from_utf8_lossy(&refresh.stderr);
    assert!(stderr.contains("message.patch"), "missing patch diagnostic omitted its path: {stderr}");
}

#[test]
fn locked_dependency_refresh_rejects_an_unresolved_revision() {
    let fixture = tempfile::tempdir().unwrap();
    copy_locked_dependency_project(fixture.path());
    std::fs::copy(fixture.path().join("fixtures/unresolved-revision.ncl"), fixture.path().join("mantle-project.ncl"))
        .unwrap();

    let refresh = run_project_command(fixture.path(), &["refresh", "--no-network", "unresolved-revision"]);

    assert!(!refresh.status.success(), "empty Git revision unexpectedly refreshed");
    let stderr = String::from_utf8_lossy(&refresh.stderr);
    assert!(stderr.contains("git rev must be 40 hex characters"), "unexpected revision diagnostic: {stderr}");
}

#[test]
fn generated_site_project_builds_package_and_check() {
    if !can_build_fast_projects() {
        eprintln!("SKIP: generated site project requires Linux, bwrap, and static BusyBox");
        return;
    }

    let package = run_project_build(GENERATED_SITE_PROJECT, ".#site");
    assert_success(&package, "generated site package");
    let html = std::fs::read_to_string(output_path(&package).join("index.html")).unwrap();
    assert!(html.contains(SITE_TITLE), "generated site title missing: {html}");

    let check = run_project_build(GENERATED_SITE_PROJECT, ".#checks.site-content");
    assert_success(&check, "generated site check");
    let result = std::fs::read_to_string(output_path(&check).join("result.txt")).unwrap();
    assert_eq!(result.trim(), SITE_CHECK_RESULT);
}

#[test]
fn codegen_project_builds_runnable_package_and_check() {
    if !can_build_fast_projects() {
        eprintln!("SKIP: codegen project requires Linux, bwrap, and static BusyBox");
        return;
    }

    let package = run_project_build(CODEGEN_PROJECT, ".#app");
    assert_success(&package, "codegen package");
    let binary = output_path(&package).join("bin/codegen-demo");
    let execution = StdCommand::new(&binary).output().unwrap();
    assert!(execution.status.success(), "generated application should run");
    assert_eq!(String::from_utf8(execution.stdout).unwrap().trim(), CODEGEN_MESSAGE);

    let check = run_project_build(CODEGEN_PROJECT, ".#checks.app");
    assert_success(&check, "codegen check");
    let result = std::fs::read_to_string(output_path(&check).join("result.txt")).unwrap();
    assert_eq!(result.trim(), CODEGEN_CHECK_RESULT);
}

#[test]
fn fixed_source_projects_assemble_authoritative_files() {
    if !can_build_fast_projects() {
        eprintln!("SKIP: fixed source assembly requires Linux, bwrap, and static BusyBox");
        return;
    }

    let c_source = run_project_build(C_PROJECT, ".#source");
    assert_success(&c_source, "C source assembly");
    let c_output = output_path(&c_source);
    assert!(c_output.join("include/greet.h").is_file(), "assembled C header missing");
    assert!(c_output.join("tests/test_greet.c").is_file(), "assembled C test missing");

    let rust_source = run_project_build(RUST_PROJECT, ".#source");
    assert_success(&rust_source, "Rust source assembly");
    let rust_output = output_path(&rust_source);
    assert!(rust_output.join("Cargo.lock").is_file(), "assembled Rust lockfile missing");
    assert!(rust_output.join("greeting/src/lib.rs").is_file(), "assembled Rust library missing");

    let delta_source = run_project_build(DELTA_SUBSTITUTION_PROJECT, ".#source");
    assert_success(&delta_source, "delta adaptor source assembly");
    let delta_text = std::fs::read_to_string(output_path(&delta_source).join("demo.rs")).unwrap();
    assert!(delta_text.contains("DeltaAcceptanceMode::Delta"));

    let release_source = run_project_build(RELEASE_WITNESS_PROJECT, ".#source");
    assert_success(&release_source, "release witness source assembly");
    let release_text = std::fs::read_to_string(output_path(&release_source).join("demo.rs")).unwrap();
    assert!(release_text.contains("VerificationDirectory"));
}

#[test]
fn c_library_project_exports_and_keeps_positive_and_negative_tests() {
    let project = project_root(C_PROJECT);
    let header = std::fs::read_to_string(project.join("include/greet.h")).unwrap();
    let test_source = std::fs::read_to_string(project.join("tests/test_greet.c")).unwrap();
    assert!(header.contains("GreetStatus greet_format"), "public C API missing");
    assert!(test_source.contains(C_POSITIVE_TEST), "positive C test missing");
    assert!(test_source.contains(C_NEGATIVE_TEST), "negative C test missing");

    let mut command = mantle_cmd();
    let output = command.current_dir(project).args(["eval", "mantle-project.ncl"]).output().unwrap();
    assert!(output.status.success(), "C project should evaluate: {}", String::from_utf8_lossy(&output.stderr));
    assert!(!output.stdout.is_empty(), "C project evaluation should export JSON");
}

#[test]
#[ignore = "realizes the pinned bootstrap C toolchain on first build"]
fn c_library_project_builds_library_cli_and_check() {
    if !can_build_fast_projects() {
        eprintln!("SKIP: C project build requires Linux, bwrap, and static BusyBox");
        return;
    }

    let package = run_project_build(C_PROJECT, ".#greet");
    assert_success(&package, "C library package");
    let output = output_path(&package);
    assert!(output.join("lib/libgreet.a").is_file(), "static library missing");
    assert!(output.join("include/greet.h").is_file(), "public header missing");
    let execution = StdCommand::new(output.join("bin/greet")).arg("Mantle").output().unwrap();
    assert!(execution.status.success(), "C CLI should run");
    assert_eq!(String::from_utf8(execution.stdout).unwrap().trim(), C_GREETING);

    let check = run_project_build(C_PROJECT, ".#checks.test-greet");
    assert_success(&check, "C project check");
    let result = std::fs::read_to_string(output_path(&check).join("result.txt")).unwrap();
    assert_eq!(result.trim(), C_CHECK_RESULT);
}

#[test]
fn rust_workspace_project_exports_and_keeps_positive_and_negative_tests() {
    let project = project_root(RUST_PROJECT);
    let library_source = std::fs::read_to_string(project.join("greeting/src/lib.rs")).unwrap();
    let app_source = std::fs::read_to_string(project.join("workspace-app/src/main.rs")).unwrap();
    assert!(library_source.contains(RUST_POSITIVE_TEST), "positive Rust test missing");
    assert!(library_source.contains(RUST_NEGATIVE_TEST), "negative Rust test missing");
    assert!(app_source.contains("rejects_extra_arguments"), "negative CLI test missing");

    let mut command = mantle_cmd();
    let output = command.current_dir(project).args(["eval", "mantle-project.ncl"]).output().unwrap();
    assert!(output.status.success(), "Rust project should evaluate: {}", String::from_utf8_lossy(&output.stderr));
    assert!(!output.stdout.is_empty(), "Rust project evaluation should export JSON");
}

#[test]
#[ignore = "realizes the source-built Rust, seed-toolchain, and musl closure on first build"]
fn rust_workspace_project_builds_package_and_smoke_check() {
    if !can_build_fast_projects() {
        eprintln!("SKIP: Rust workspace build requires Linux, bwrap, and static BusyBox");
        return;
    }

    let package = run_project_build(RUST_PROJECT, ".#workspace-app");
    assert_success(&package, "Rust workspace package");
    let binary = output_path(&package).join("bin/workspace-app");
    let execution = StdCommand::new(&binary).arg("Mantle").output().unwrap();
    assert!(execution.status.success(), "Rust workspace application should run");
    assert_eq!(String::from_utf8(execution.stdout).unwrap().trim(), RUST_GREETING);

    let check = run_project_build(RUST_PROJECT, ".#checks.smoke");
    assert_success(&check, "Rust workspace check");
    let result = std::fs::read_to_string(output_path(&check).join("result.txt")).unwrap();
    assert_eq!(result.trim(), RUST_CHECK_RESULT);
}

#[test]
fn fetched_and_patched_project_declares_fixed_positive_and_negative_patches() {
    let project = project_root(FETCHED_PATCHED_PROJECT);
    let project_source = std::fs::read_to_string(project.join("mantle-project.ncl")).unwrap();
    let positive_patch = std::fs::read_to_string(project.join("patches/readme.patch")).unwrap();
    let negative_patch = std::fs::read_to_string(project.join("patches/invalid-context.patch")).unwrap();
    assert!(project_source.contains("crc64-2.0.0.crate"), "pinned upstream crate missing");
    assert!(project_source.contains("sha256-G2t3VkrTsVRXQrfE4NYeYPnQUgnzxpGz2eI3QlVE/rA="));
    assert!(project_source.contains("sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="));
    assert!(positive_patch.contains("patched reproducibly by the Mantle example project"));
    assert!(negative_patch.contains("DOES-NOT-EXIST"));
    assert_project_evaluates(FETCHED_PATCHED_PROJECT, "fetched and patched project");
}

#[test]
#[ignore = "uses the live crates.io fixed-output source"]
fn fetched_and_patched_project_builds_check_and_rejects_invalid_patch() {
    if !can_build_fast_projects() {
        eprintln!("SKIP: fetched project requires Linux, bwrap, and static BusyBox");
        return;
    }

    let check = run_project_build(FETCHED_PATCHED_PROJECT, ".#checks.patch");
    assert_success(&check, "fetched and patched check");
    assert_eq!(check_result(&check), FETCHED_PATCHED_CHECK_RESULT);

    let invalid = run_project_build_verbose(FETCHED_PATCHED_PROJECT, ".#invalid-patch");
    let stderr = String::from_utf8_lossy(&invalid.output.stderr);
    assert!(!invalid.output.status.success(), "invalid patch unexpectedly succeeded");
    assert!(stderr.contains("DOES-NOT-EXIST"), "invalid patch error missing failed target: {stderr}");

    let invalid_hash = run_project_build_verbose(FETCHED_PATCHED_PROJECT, ".#invalid-hash");
    let stderr = String::from_utf8_lossy(&invalid_hash.output.stderr);
    assert!(!invalid_hash.output.status.success(), "invalid fixed-output hash unexpectedly succeeded");
    assert!(stderr.contains("hash mismatch"), "fixed-output mismatch diagnostic missing: {stderr}");
}

#[test]
fn multi_output_sdk_source_assembles_and_project_evaluates() {
    assert_project_evaluates(MULTI_OUTPUT_SDK_PROJECT, "multi-output SDK project");
    if !can_build_fast_projects() {
        eprintln!("SKIP: SDK source assembly requires Linux, bwrap, and static BusyBox");
        return;
    }

    let source = run_project_build(MULTI_OUTPUT_SDK_PROJECT, ".#source");
    assert_success(&source, "multi-output SDK source assembly");
    let output = output_path(&source);
    assert!(output.join("include/mantle_sdk.h").is_file(), "SDK header missing");
    assert!(output.join("tests/test_sdk.c").is_file(), "SDK negative tests missing");
    assert!(output.join("docs/mantle-sdk.1").is_file(), "SDK manual page missing");
}

#[test]
#[ignore = "realizes the pinned bootstrap C toolchain and all selected SDK outputs"]
fn multi_output_sdk_builds_outputs_and_selected_consumers() {
    if !can_build_fast_projects() {
        eprintln!("SKIP: SDK project build requires Linux, bwrap, and static BusyBox");
        return;
    }

    let package = run_project_build(MULTI_OUTPUT_SDK_PROJECT, ".#sdk");
    assert_success(&package, "multi-output SDK package");
    let runtime = output_path_for_label(&package, "out");
    let development = output_path_for_label(&package, "dev");
    let documentation = output_path_for_label(&package, "doc");
    let debug = output_path_for_label(&package, "debug");
    assert!(runtime.join("bin/mantle-sdk-greet").is_file());
    assert!(development.join("include/mantle_sdk.h").is_file());
    assert!(development.join("lib/libmantle_sdk.a").is_file());
    assert!(documentation.join("share/man/man1/mantle-sdk-greet.1").is_file());
    assert!(debug.join("lib/debug/mantle-sdk-greet").is_file());
    let execution = StdCommand::new(runtime.join("bin/mantle-sdk-greet")).arg("Mantle").output().unwrap();
    assert!(execution.status.success(), "SDK runtime should execute");
    assert_eq!(String::from_utf8(execution.stdout).unwrap().trim(), "Hello, Mantle!");

    let runtime_check = run_project_build(MULTI_OUTPUT_SDK_PROJECT, ".#checks.runtime");
    assert_success(&runtime_check, "SDK runtime consumer");
    assert_eq!(check_result(&runtime_check), MULTI_OUTPUT_RUNTIME_CHECK_RESULT);
    let development_check = run_project_build(MULTI_OUTPUT_SDK_PROJECT, ".#checks.development");
    assert_success(&development_check, "SDK development consumer");
    assert_eq!(check_result(&development_check), MULTI_OUTPUT_DEVELOPMENT_CHECK_RESULT);
}

#[test]
fn schema_codegen_builds_bindings_and_rejects_invalid_schema() {
    assert_project_evaluates(SCHEMA_CODEGEN_PROJECT, "schema codegen project");
    if !can_build_fast_projects() {
        eprintln!("SKIP: schema generation requires Linux, bwrap, and static BusyBox");
        return;
    }

    let bindings = run_project_build(SCHEMA_CODEGEN_PROJECT, ".#bindings");
    assert_success(&bindings, "schema generated bindings");
    let output = output_path(&bindings);
    assert!(output.join("c/greeting.h").is_file(), "generated C header missing");
    assert!(output.join("rust/greeting.rs").is_file(), "generated Rust module missing");
    let manifest = std::fs::read_to_string(output.join("bindings.manifest")).unwrap();
    assert!(manifest.contains("schema_version=1"));
    assert!(manifest.contains("default_name=World"));

    let invalid = run_project_build_verbose(SCHEMA_CODEGEN_PROJECT, ".#invalid-schema");
    let stderr = String::from_utf8_lossy(&invalid.output.stderr);
    assert!(!invalid.output.status.success(), "invalid schema unexpectedly generated bindings");
    assert!(
        stderr.contains("requires non-empty prefix and default_name"),
        "invalid schema error missing: {stderr}"
    );

    let unsafe_schema = run_project_build_verbose(SCHEMA_CODEGEN_PROJECT, ".#unsafe-schema");
    let stderr = String::from_utf8_lossy(&unsafe_schema.output.stderr);
    assert!(!unsafe_schema.output.status.success(), "unsafe schema unexpectedly generated source");
    assert!(stderr.contains("unsupported characters"), "unsafe schema error missing: {stderr}");
}

#[test]
#[ignore = "realizes both bootstrap C and source-built Rust toolchains"]
fn schema_codegen_builds_both_languages_and_integration_check() {
    if !can_build_fast_projects() {
        eprintln!("SKIP: schema integration requires Linux, bwrap, and static BusyBox");
        return;
    }

    let check = run_project_build(SCHEMA_CODEGEN_PROJECT, ".#checks.integration");
    assert_success(&check, "schema C/Rust integration check");
    assert_eq!(check_result(&check), SCHEMA_CODEGEN_CHECK_RESULT);
}

#[test]
fn reproducible_release_matches_blake3_and_detects_tampering() {
    assert_project_evaluates(REPRODUCIBLE_RELEASE_PROJECT, "reproducible release project");
    if !can_build_fast_projects() {
        eprintln!("SKIP: release project requires Linux, bwrap, and static BusyBox");
        return;
    }

    let release_a = run_project_build(REPRODUCIBLE_RELEASE_PROJECT, ".#release-a");
    let release_b = run_project_build(REPRODUCIBLE_RELEASE_PROJECT, ".#release-b");
    assert_success(&release_a, "release A");
    assert_success(&release_b, "release B");
    let archive_a = std::fs::read(output_path(&release_a).join("release-demo.tar")).unwrap();
    let archive_b = std::fs::read(output_path(&release_b).join("release-demo.tar")).unwrap();
    assert_eq!(archive_a, archive_b, "independent release archives differ");
    let digest = blake3::hash(&archive_a);
    let digest_sri = format!("blake3-{}", data_encoding::BASE64.encode(digest.as_bytes()));
    assert_eq!(digest_sri, RELEASE_ARCHIVE_BLAKE3);
    let sidecar = std::fs::read_to_string(output_path(&release_a).join("release-demo.tar.blake3")).unwrap();
    assert_eq!(sidecar.trim(), RELEASE_ARCHIVE_BLAKE3);

    let reproducibility = run_project_build(REPRODUCIBLE_RELEASE_PROJECT, ".#checks.reproducible");
    assert_success(&reproducibility, "reproducibility check");
    assert_eq!(check_result(&reproducibility), RELEASE_CHECK_RESULT);
    let tamper = run_project_build(REPRODUCIBLE_RELEASE_PROJECT, ".#checks.tamper-detection");
    assert_success(&tamper, "release tamper detection");
    assert_eq!(check_result(&tamper), RELEASE_TAMPER_CHECK_RESULT);
}

#[test]
fn missing_project_selector_fails_closed() {
    let run = run_project_build(GENERATED_SITE_PROJECT, MISSING_SELECTOR);
    let stderr = String::from_utf8_lossy(&run.output.stderr);
    assert!(!run.output.status.success(), "missing selector should fail");
    assert!(stderr.contains("missing-package"), "selector error should name the missing package: {stderr}");
}
