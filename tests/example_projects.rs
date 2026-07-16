use std::path::Path;
use std::path::PathBuf;
use std::process::Command as StdCommand;

use assert_cmd::Command;

const PROJECTS_ROOT: &str = "examples/projects";
const GENERATED_SITE_PROJECT: &str = "generated-site";
const CODEGEN_PROJECT: &str = "codegen-pipeline";
const C_PROJECT: &str = "c-library-cli";
const RUST_PROJECT: &str = "rust-workspace";
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
const MISSING_SELECTOR: &str = ".#missing-package";

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
    let store = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    let mut command = mantle_cmd();
    command.current_dir(project_root(project));
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

fn output_path(run: &ProjectRun) -> PathBuf {
    let stdout = String::from_utf8(run.output.stdout.clone()).unwrap();
    let line = stdout.lines().next().expect("project build should print an output path");
    PathBuf::from(line.split_once(" (").map(|(path, _label)| path).unwrap_or(line))
}

fn assert_success(run: &ProjectRun, label: &str) {
    let stderr = String::from_utf8_lossy(&run.output.stderr);
    assert!(run.output.status.success(), "{label} failed:\n{stderr}");
    assert!(!run.output.stdout.is_empty(), "{label} should print an output path");
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
fn missing_project_selector_fails_closed() {
    let run = run_project_build(GENERATED_SITE_PROJECT, MISSING_SELECTOR);
    let stderr = String::from_utf8_lossy(&run.output.stderr);
    assert!(!run.output.status.success(), "missing selector should fail");
    assert!(stderr.contains("missing-package"), "selector error should name the missing package: {stderr}");
}
