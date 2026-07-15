use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;

use assert_cmd::Command;
use base64::Engine as _;
use predicates::prelude::*;
use sha2::Digest as _;
use tempfile::TempDir;

fn crunch() -> Command {
    Command::cargo_bin("crunch").unwrap()
}

fn write_executable(dir: &Path, name: &str, body: &str) {
    let path = dir.join(name);
    fs::write(&path, body).unwrap();
    let mut permissions = fs::metadata(&path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&path, permissions).unwrap();
}

fn count_entries(dir: &Path) -> usize {
    fs::read_dir(dir).unwrap().count()
}

fn fake_ok_script() -> &'static str {
    "#!/bin/sh\nexit 0\n"
}

fn repo_lib_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib").join("lib.ncl")
}

fn sha256_sri(bytes: &[u8]) -> String {
    let digest = sha2::Sha256::digest(bytes);
    format!("sha256-{}", base64::engine::general_purpose::STANDARD.encode(digest))
}

fn write_fetchurl_ncl(path: &Path, source_path: &Path, hash_sri: &str) {
    let source_url = format!("file://{}", source_path.display());
    let lib_path = repo_lib_path();
    fs::write(
        path,
        format!(
            r#"let crunch = import "{}" in
crunch.fetchurl {{
  url = "{}",
  hash = "{}",
}}"#,
            lib_path.display(),
            source_url,
            hash_sri,
        ),
    )
    .unwrap();
}

fn write_shell_derivation_ncl(path: &Path) {
    let lib_path = repo_lib_path();
    fs::write(
        path,
        format!(
            r#"let crunch = import "{}" in
{{
  name = "plan-build-test",
  builder = "/bin/sh",
  args = ["-c", "echo hi > $out"],
  addressing_mode = 'input-addressed,
}} | crunch.Derivation"#,
            lib_path.display(),
        ),
    )
    .unwrap();
}

fn write_fetchurl_bad_hash_ncl(path: &Path, source_path: &Path) {
    let source_url = format!("file://{}", source_path.display());
    let lib_path = repo_lib_path();
    fs::write(
        path,
        format!(
            r#"let crunch = import "{}" in
{{
  bad = crunch.fetchurl {{
    url = "{}",
    hash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
  }},
}}"#,
            lib_path.display(),
            source_url,
        ),
    )
    .unwrap();
}

fn write_fetchurl_missing_source_ncl(path: &Path, source_path: &Path) {
    let source_url = format!("file://{}", source_path.display());
    let lib_path = repo_lib_path();
    fs::write(
        path,
        format!(
            r#"let crunch = import "{}" in
{{
  missing = crunch.fetchurl {{
    url = "{}",
    hash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
  }},
}}"#,
            lib_path.display(),
            source_url,
        ),
    )
    .unwrap();
}

#[test]
fn doctor_default_profile_is_build_and_does_not_mutate_paths() {
    let root = TempDir::new().unwrap();
    let tool_dir = root.path().join("tools");
    let store_dir = root.path().join("store");
    let state_dir = root.path().join("state");
    fs::create_dir_all(&tool_dir).unwrap();
    fs::create_dir_all(&store_dir).unwrap();
    fs::create_dir_all(&state_dir).unwrap();
    write_executable(&tool_dir, "bwrap", fake_ok_script());
    write_executable(&tool_dir, "fusermount3", fake_ok_script());
    write_executable(&tool_dir, "busybox", fake_ok_script());

    crunch()
        .arg("--state-dir")
        .arg(&state_dir)
        .arg("--store")
        .arg(&store_dir)
        .arg("doctor")
        .env("PATH", &tool_dir)
        .env("SNIX_BUILD_SANDBOX_SHELL", tool_dir.join("busybox"))
        .current_dir(root.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("doctor profile: build"))
        .stdout(predicate::str::contains("status: ok"));

    assert_eq!(count_entries(&store_dir), 0, "doctor must not mutate store dir");
    assert_eq!(count_entries(&state_dir), 0, "doctor must not mutate state dir");
}

#[test]
fn doctor_explicit_self_build_profile_checks_nightly_visibility() {
    let root = TempDir::new().unwrap();
    let tool_dir = root.path().join("tools");
    let store_dir = root.path().join("store");
    let state_dir = root.path().join("state");
    fs::create_dir_all(&tool_dir).unwrap();
    fs::create_dir_all(&store_dir).unwrap();
    fs::create_dir_all(&state_dir).unwrap();
    for name in ["cargo", "rustc", "bwrap", "fusermount3", "busybox"] {
        write_executable(&tool_dir, name, fake_ok_script());
    }

    crunch()
        .arg("--state-dir")
        .arg(&state_dir)
        .arg("--store")
        .arg(&store_dir)
        .arg("doctor")
        .arg("--profile")
        .arg("self-build")
        .env("PATH", &tool_dir)
        .env("SNIX_BUILD_SANDBOX_SHELL", tool_dir.join("busybox"))
        .current_dir(root.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("doctor profile: self-build"))
        .stdout(predicate::str::contains("nightly-toolchain"))
        .stdout(predicate::str::contains("status: ok"));

    assert_eq!(count_entries(&store_dir), 0, "doctor must not mutate store dir");
    assert_eq!(count_entries(&state_dir), 0, "doctor must not mutate state dir");
}

#[test]
fn doctor_json_output_reports_selected_profile() {
    let root = TempDir::new().unwrap();
    let tool_dir = root.path().join("tools");
    let store_dir = root.path().join("store");
    let state_dir = root.path().join("state");
    fs::create_dir_all(&tool_dir).unwrap();
    fs::create_dir_all(&store_dir).unwrap();
    fs::create_dir_all(&state_dir).unwrap();
    for name in ["cargo", "rustc", "bwrap", "fusermount3", "busybox"] {
        write_executable(&tool_dir, name, fake_ok_script());
    }

    crunch()
        .arg("--json")
        .arg("--state-dir")
        .arg(&state_dir)
        .arg("--store")
        .arg(&store_dir)
        .arg("doctor")
        .arg("--profile")
        .arg("self-build")
        .env("PATH", &tool_dir)
        .env("SNIX_BUILD_SANDBOX_SHELL", tool_dir.join("busybox"))
        .current_dir(root.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"schema\": \"crunch-doctor-report-v1\""))
        .stdout(predicate::str::contains("\"profile\": \"self-build\""))
        .stdout(predicate::str::contains("\"ok\": true"));
}

#[test]
fn doctor_json_failure_reports_missing_prerequisite() {
    let root = TempDir::new().unwrap();
    let tool_dir = root.path().join("tools");
    let store_dir = root.path().join("store");
    let state_dir = root.path().join("state");
    fs::create_dir_all(&tool_dir).unwrap();
    fs::create_dir_all(&store_dir).unwrap();
    fs::create_dir_all(&state_dir).unwrap();
    write_executable(&tool_dir, "bwrap", fake_ok_script());
    write_executable(&tool_dir, "fusermount3", fake_ok_script());

    crunch()
        .arg("--json")
        .arg("--state-dir")
        .arg(&state_dir)
        .arg("--store")
        .arg(&store_dir)
        .arg("doctor")
        .env("PATH", &tool_dir)
        .env("SNIX_BUILD_SANDBOX_SHELL", root.path().join("missing-busybox"))
        .current_dir(root.path())
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"schema\": \"crunch-doctor-report-v1\""))
        .stdout(predicate::str::contains("\"profile\": \"build\""))
        .stdout(predicate::str::contains("\"ok\": false"))
        .stdout(predicate::str::contains("\"id\": \"sandbox-shell\""))
        .stdout(predicate::str::contains("missing or non-executable shell"))
        .stderr(predicate::str::is_empty());

    assert_eq!(count_entries(&store_dir), 0, "doctor must not mutate store dir");
    assert_eq!(count_entries(&state_dir), 0, "doctor must not mutate state dir");
}

#[test]
fn build_plan_reports_build_for_uncached_root() {
    let root = TempDir::new().unwrap();
    let tool_dir = root.path().join("tools");
    let store_dir = root.path().join("store");
    let state_dir = root.path().join("state");
    let ncl_file = root.path().join("plan-build.ncl");
    fs::create_dir_all(&tool_dir).unwrap();
    fs::create_dir_all(&store_dir).unwrap();
    fs::create_dir_all(&state_dir).unwrap();
    write_executable(&tool_dir, "bwrap", fake_ok_script());
    write_executable(&tool_dir, "fusermount3", fake_ok_script());
    write_executable(&tool_dir, "busybox", fake_ok_script());
    write_shell_derivation_ncl(&ncl_file);

    crunch()
        .arg("--state-dir")
        .arg(&state_dir)
        .arg("--store")
        .arg(&store_dir)
        .arg("build")
        .arg("--plan")
        .arg("--no-substitute")
        .arg(&ncl_file)
        .env("PATH", &tool_dir)
        .env("SNIX_BUILD_SANDBOX_SHELL", tool_dir.join("busybox"))
        .current_dir(root.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("build plan:"))
        .stdout(predicate::str::contains("route: selected=local-build"))
        .stdout(predicate::str::contains("trusted-substitute:trusted-substitute-missing"));

    assert_eq!(count_entries(&store_dir), 0, "plan must not mutate empty store dir");
    assert_eq!(count_entries(&state_dir), 0, "plan must not mutate empty state dir");
}

#[test]
fn build_json_failure_envelope_includes_saved_log_path() {
    let root = TempDir::new().unwrap();
    let store_dir = root.path().join("store");
    let state_dir = root.path().join("state");
    let source_path = root.path().join("source-bad.txt");
    let ncl_file = root.path().join("bad-fetch.ncl");
    fs::create_dir_all(&store_dir).unwrap();
    fs::create_dir_all(&state_dir).unwrap();
    fs::write(&source_path, b"bad-fetch-content\n").unwrap();
    write_fetchurl_bad_hash_ncl(&ncl_file, &source_path);

    let output = crunch()
        .arg("--json")
        .arg("--state-dir")
        .arg(&state_dir)
        .arg("--store")
        .arg(&store_dir)
        .arg("build")
        .arg(&ncl_file)
        .current_dir(root.path())
        .output()
        .unwrap();
    assert!(!output.status.success(), "bad hash build must fail");

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let failure = &json["failed"][0];
    assert_eq!(failure["root"], "bad");
    assert_eq!(failure["phase"], "build");
    assert_eq!(failure["error_class"], "fixed-output-hash-mismatch");
    let saved_log_path = failure["saved_log_path"].as_str().expect("saved_log_path");
    assert!(Path::new(saved_log_path).exists(), "saved log path must exist");
}

#[test]
fn build_human_failure_summary_matches_json_facts() {
    let root = TempDir::new().unwrap();
    let store_dir = root.path().join("store");
    let state_dir = root.path().join("state");
    let source_path = root.path().join("missing-source.txt");
    let ncl_file = root.path().join("bad-human-fetch.ncl");
    fs::create_dir_all(&store_dir).unwrap();
    fs::create_dir_all(&state_dir).unwrap();
    write_fetchurl_missing_source_ncl(&ncl_file, &source_path);

    let json_output = crunch()
        .arg("--json")
        .arg("--state-dir")
        .arg(&state_dir)
        .arg("--store")
        .arg(&store_dir)
        .arg("build")
        .arg(&ncl_file)
        .current_dir(root.path())
        .output()
        .unwrap();
    assert!(!json_output.status.success());
    let json_stdout = String::from_utf8(json_output.stdout).unwrap();
    let json_value: serde_json::Value = serde_json::from_str(&json_stdout).unwrap();
    let failure = &json_value["failed"][0];
    let root_name = failure["root"].as_str().unwrap().to_string();
    let phase = failure["phase"].as_str().unwrap().to_string();
    let error_class = failure["error_class"].as_str().unwrap().to_string();
    let saved_log_path = failure["saved_log_path"].as_str().unwrap().to_string();

    let human_output = crunch()
        .arg("--state-dir")
        .arg(&state_dir)
        .arg("--store")
        .arg(&store_dir)
        .arg("build")
        .arg(&ncl_file)
        .current_dir(root.path())
        .output()
        .unwrap();
    assert!(!human_output.status.success());
    let stderr = String::from_utf8(human_output.stderr).unwrap();
    assert!(stderr.contains(&format!("FAILED root: {root_name}")));
    assert!(stderr.contains(&format!("phase: {phase}")));
    assert!(stderr.contains(&format!("error_class: {error_class}")));
    assert!(stderr.contains(&format!("saved_log_path: {saved_log_path}")));
}

#[test]
fn build_json_preflight_failure_omits_saved_log_path() {
    let root = TempDir::new().unwrap();
    let store_dir = root.path().join("store");
    let state_dir = root.path().join("state");
    let source_path = root.path().join("source-preflight.txt");
    let ncl_file = root.path().join("preflight-fetch.ncl");
    fs::create_dir_all(&store_dir).unwrap();
    fs::create_dir_all(&state_dir).unwrap();
    fs::create_dir(state_dir.join("pathinfo.redb")).unwrap();
    fs::write(&source_path, b"preflight-content\n").unwrap();
    write_fetchurl_bad_hash_ncl(&ncl_file, &source_path);

    let output = crunch()
        .arg("--json")
        .arg("--state-dir")
        .arg(&state_dir)
        .arg("--store")
        .arg(&store_dir)
        .arg("build")
        .arg("--strict-hermetic")
        .arg(&ncl_file)
        .current_dir(root.path())
        .output()
        .unwrap();
    assert!(!output.status.success(), "strict preflight build must fail");

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let failure = &json["failed"][0];
    assert_eq!(failure["root"], "bad");
    assert_eq!(failure["phase"], "preflight");
    assert_eq!(failure["error_class"], "preflight");
    assert!(failure.get("saved_log_path").is_none(), "preflight failure must omit saved_log_path");
}

#[test]
fn build_json_success_reports_log_persistence_failure_without_fake_log_file() {
    let root = TempDir::new().unwrap();
    let store_dir = root.path().join("store");
    let state_dir = root.path().join("state");
    let source_path = root.path().join("source.txt");
    let ncl_file = root.path().join("fetch.ncl");
    let log_target = root.path().join("occupied-log-path");
    fs::create_dir_all(&store_dir).unwrap();
    fs::create_dir_all(&state_dir).unwrap();
    fs::write(&source_path, b"diagnostic-content\n").unwrap();
    fs::write(&log_target, b"occupied\n").unwrap();
    let hash_sri = sha256_sri(&fs::read(&source_path).unwrap());
    write_fetchurl_ncl(&ncl_file, &source_path, &hash_sri);

    let output = crunch()
        .arg("--json")
        .arg("--state-dir")
        .arg(&state_dir)
        .arg("--store")
        .arg(&store_dir)
        .arg("build")
        .arg(&ncl_file)
        .env("CRUNCH_LOG_DIR", &log_target)
        .current_dir(root.path())
        .output()
        .unwrap();
    assert!(output.status.success(), "fetchurl build should succeed");

    let stdout = String::from_utf8(output.stdout).unwrap();
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let diagnostic = &json["diagnostic_persistence_failures"][0];
    assert_eq!(diagnostic["operation"], "create-log-dir");
    assert_eq!(diagnostic["artifact"], "build-log");
    assert_eq!(diagnostic["attempted_path"], log_target.display().to_string());
    assert!(json["outcomes"][0]["log_file"].is_null(), "missing log file must stay null");
    let built_path = json["outcomes"][0]["outputs"][0]["path"].as_str().unwrap();
    assert!(Path::new(built_path).exists(), "built output must still exist");
    assert!(String::from_utf8_lossy(&output.stderr).trim().is_empty(), "json stderr must stay empty");
}

#[test]
fn build_human_failure_reports_log_persistence_failure_without_saved_log_path() {
    let root = TempDir::new().unwrap();
    let store_dir = root.path().join("store");
    let state_dir = root.path().join("state");
    let source_path = root.path().join("missing-source.txt");
    let ncl_file = root.path().join("missing-fetch.ncl");
    let log_target = root.path().join("occupied-log-path");
    fs::create_dir_all(&store_dir).unwrap();
    fs::create_dir_all(&state_dir).unwrap();
    fs::write(&log_target, b"occupied\n").unwrap();
    write_fetchurl_missing_source_ncl(&ncl_file, &source_path);

    let output = crunch()
        .arg("--state-dir")
        .arg(&state_dir)
        .arg("--store")
        .arg(&store_dir)
        .arg("build")
        .arg(&ncl_file)
        .env("CRUNCH_LOG_DIR", &log_target)
        .current_dir(root.path())
        .output()
        .unwrap();
    assert!(!output.status.success(), "missing source build must fail");

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("WARNING: diagnostic persistence failed"));
    assert!(stderr.contains("operation: create-log-dir"));
    assert!(stderr.contains("artifact: build-log"));
    assert!(!stderr.contains("saved_log_path:"), "failed log write must not invent saved_log_path");
}

#[test]
fn build_plan_json_reports_action_schema() {
    let root = TempDir::new().unwrap();
    let tool_dir = root.path().join("tools");
    let store_dir = root.path().join("store");
    let state_dir = root.path().join("state");
    let ncl_file = root.path().join("plan-build-json.ncl");
    fs::create_dir_all(&tool_dir).unwrap();
    fs::create_dir_all(&store_dir).unwrap();
    fs::create_dir_all(&state_dir).unwrap();
    write_executable(&tool_dir, "bwrap", fake_ok_script());
    write_executable(&tool_dir, "fusermount3", fake_ok_script());
    write_executable(&tool_dir, "busybox", fake_ok_script());
    write_shell_derivation_ncl(&ncl_file);

    let output = crunch()
        .arg("--json")
        .arg("--state-dir")
        .arg(&state_dir)
        .arg("--store")
        .arg(&store_dir)
        .arg("build")
        .arg("--plan")
        .arg("--no-substitute")
        .arg(&ncl_file)
        .env("PATH", &tool_dir)
        .env("SNIX_BUILD_SANDBOX_SHELL", tool_dir.join("busybox"))
        .current_dir(root.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["schema"], "crunch-build-plan-v1");
    assert_eq!(json["entries"][0]["action"], "build");
    assert_eq!(json["entries"][0]["route_plan"]["schema"], "mantle-realization-route-plan-v1");
    assert_eq!(json["entries"][0]["route_plan"]["selected_route"], "local-build");
    assert_eq!(json["entries"][0]["route_plan"]["rejected_routes"][1]["route"], "trusted-substitute");
}

#[test]
fn build_plan_reports_cached_after_fetchurl_build() {
    let root = TempDir::new().unwrap();
    let store_dir = root.path().join("store");
    let state_dir = root.path().join("state");
    let source_path = root.path().join("source.txt");
    let ncl_file = root.path().join("fetch-plan.ncl");
    fs::create_dir_all(&store_dir).unwrap();
    fs::create_dir_all(&state_dir).unwrap();
    fs::write(&source_path, b"fetch-plan-content\n").unwrap();
    let hash_sri = sha256_sri(&fs::read(&source_path).unwrap());
    write_fetchurl_ncl(&ncl_file, &source_path, &hash_sri);

    crunch()
        .arg("--state-dir")
        .arg(&state_dir)
        .arg("--store")
        .arg(&store_dir)
        .arg("build")
        .arg(&ncl_file)
        .current_dir(root.path())
        .assert()
        .success();

    let store_snapshot = count_entries(&store_dir);
    let state_snapshot = count_entries(&state_dir);

    crunch()
        .arg("--state-dir")
        .arg(&state_dir)
        .arg("--store")
        .arg(&store_dir)
        .arg("build")
        .arg("--plan")
        .arg(&ncl_file)
        .current_dir(root.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("cached"));

    assert_eq!(count_entries(&store_dir), store_snapshot, "plan must not add top-level store entries");
    assert_eq!(count_entries(&state_dir), state_snapshot, "plan must not add top-level state entries");
}

#[test]
fn build_plan_reports_preflight_error_for_missing_store_dir() {
    let root = TempDir::new().unwrap();
    let state_dir = root.path().join("state");
    let missing_store_dir = root.path().join("missing-store");
    let ncl_file = root.path().join("plan-preflight.ncl");
    fs::create_dir_all(&state_dir).unwrap();
    write_shell_derivation_ncl(&ncl_file);

    crunch()
        .arg("--state-dir")
        .arg(&state_dir)
        .arg("--store")
        .arg(&missing_store_dir)
        .arg("build")
        .arg("--plan")
        .arg(&ncl_file)
        .current_dir(root.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("preflight-error"))
        .stderr(predicate::str::contains("route: selected=preflight-error"))
        .stderr(predicate::str::contains("local-build:local-preflight-failed"))
        .stderr(predicate::str::contains("output store directory"));

    assert!(!missing_store_dir.exists(), "plan preflight must not create missing store dir");
    assert_eq!(count_entries(&state_dir), 0, "plan preflight must not mutate state dir");
}

#[test]
fn build_without_plan_still_executes_fetchurl() {
    let root = TempDir::new().unwrap();
    let store_dir = root.path().join("store");
    let state_dir = root.path().join("state");
    let source_path = root.path().join("source-build.txt");
    let ncl_file = root.path().join("fetch-build.ncl");
    fs::create_dir_all(&store_dir).unwrap();
    fs::create_dir_all(&state_dir).unwrap();
    fs::write(&source_path, b"fetch-build-content\n").unwrap();
    let hash_sri = sha256_sri(&fs::read(&source_path).unwrap());
    write_fetchurl_ncl(&ncl_file, &source_path, &hash_sri);

    let output = crunch()
        .arg("--state-dir")
        .arg(&state_dir)
        .arg("--store")
        .arg(&store_dir)
        .arg("build")
        .arg(&ncl_file)
        .current_dir(root.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let built_path = String::from_utf8(output).unwrap();
    let built_path = built_path.lines().next().unwrap().trim();
    assert!(Path::new(built_path).exists(), "plain build must still realize fetchurl output");
}

#[test]
fn doctor_failure_names_missing_prerequisite_and_profile() {
    let root = TempDir::new().unwrap();
    let empty_path = root.path().join("empty-path");
    let store_dir = root.path().join("store");
    let state_dir = root.path().join("state");
    fs::create_dir_all(&empty_path).unwrap();
    fs::create_dir_all(&store_dir).unwrap();
    fs::create_dir_all(&state_dir).unwrap();

    crunch()
        .arg("--state-dir")
        .arg(&state_dir)
        .arg("--store")
        .arg(&store_dir)
        .arg("doctor")
        .arg("--profile")
        .arg("self-build")
        .env("PATH", &empty_path)
        .env_remove("SNIX_BUILD_SANDBOX_SHELL")
        .current_dir(root.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("doctor profile: self-build"))
        .stderr(predicate::str::contains("status: failed"))
        .stderr(predicate::str::contains("nightly-toolchain"));

    assert_eq!(count_entries(&store_dir), 0, "doctor must not mutate store dir");
    assert_eq!(count_entries(&state_dir), 0, "doctor must not mutate state dir");
}
