use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;

use assert_cmd::Command;
use base64::Engine as _;
use predicates::prelude::*;
use sha2::Digest as _;
use tempfile::TempDir;

const PUBLIC_COMMAND_COUNT_MIN: usize = 160;

fn crunch() -> Command {
    Command::cargo_bin("crunch").unwrap()
}

fn contract_generator() -> Command {
    Command::cargo_bin("generate-operator-command-contract").unwrap()
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

#[test]
fn empty_selected_state_environment_rejects_artifact_import_before_any_write() {
    for name in ["CRUNCH_STATE_DIR", "XDG_STATE_HOME", "HOME"] {
        for verbose in [false, true] {
            let root = TempDir::new().unwrap();
            let source = root.path().join("source.txt");
            fs::write(&source, b"state admission regression\n").unwrap();
            let mut command = crunch();
            command
                .arg("--json")
                .args(["artifact", "import"])
                .arg(&source)
                .current_dir(root.path())
                .env_remove("CRUNCH_STATE_DIR")
                .env_remove("XDG_STATE_HOME")
                .env_remove("HOME")
                .env(name, "");
            if verbose {
                command.arg("--verbose");
            }
            let output = command.output().unwrap();
            assert_eq!(output.status.code(), Some(3), "{name}: {output:?}");
            assert!(output.stdout.is_empty(), "{name}: no success report");
            let stderr = String::from_utf8(output.stderr).unwrap();
            assert!(stderr.contains(&format!("empty-state-dir-env: {name}")), "{stderr}");
            assert!(!stderr.contains("panicked"), "{stderr}");
            assert!(!root.path().join("frontend-artifacts").exists(), "{name}: no CWD state");
            assert!(!root.path().join("crunch").exists(), "{name}: no relative XDG state");
            assert!(!root.path().join(".local").exists(), "{name}: no relative HOME state");
        }
    }
}

#[test]
fn relative_selected_state_environment_rejects_artifact_import_without_cwd_writes() {
    for name in ["CRUNCH_STATE_DIR", "XDG_STATE_HOME", "HOME"] {
        for value in ["relative/state", "."] {
            let root = TempDir::new().unwrap();
            let source = root.path().join("source.txt");
            fs::write(&source, b"relative environment must not become cwd state\n").unwrap();
            let output = crunch()
                .args(["--json", "artifact", "import"])
                .arg(&source)
                .current_dir(root.path())
                .env_remove("CRUNCH_STATE_DIR")
                .env_remove("XDG_STATE_HOME")
                .env_remove("HOME")
                .env(name, value)
                .output()
                .unwrap();
            assert_eq!(output.status.code(), Some(3), "{name}={value}: {output:?}");
            assert!(output.stdout.is_empty(), "{name}={value}: no success report");
            let stderr = String::from_utf8(output.stderr).unwrap();
            assert!(stderr.contains(&format!("relative-state-dir-env: {name}")), "{stderr}");
            assert!(!stderr.contains("panicked"), "{stderr}");
            assert_eq!(count_entries(root.path()), 1, "{name}={value}: only source.txt may exist");
        }
    }
}

#[test]
fn explicit_state_dir_overrides_empty_environment_and_preserves_relative_path() {
    for override_path in ["relative-state", "."] {
        let root = TempDir::new().unwrap();
        let source = root.path().join("source.txt");
        fs::write(&source, b"relative override admission\n").unwrap();
        let output = crunch()
            .args(["--json", "--state-dir", override_path, "artifact", "import"])
            .arg(&source)
            .current_dir(root.path())
            .env("CRUNCH_STATE_DIR", "")
            .env("XDG_STATE_HOME", "")
            .env("HOME", "")
            .output()
            .unwrap();
        assert!(output.status.success(), "{override_path}: {output:?}");
        assert!(root.path().join(override_path).join("frontend-artifacts").is_dir());
        if override_path != "." {
            assert!(!root.path().join("frontend-artifacts").exists());
        }
    }
}

#[test]
fn selected_crunch_state_dir_ignores_empty_lower_priority_fallbacks() {
    let root = TempDir::new().unwrap();
    let source = root.path().join("source.txt");
    fs::write(&source, b"crunch state precedence\n").unwrap();
    let selected_state = root.path().join("selected-state");
    let output = crunch()
        .args(["--json", "artifact", "import"])
        .arg(&source)
        .current_dir(root.path())
        .env("CRUNCH_STATE_DIR", &selected_state)
        .env("XDG_STATE_HOME", "")
        .env("HOME", "")
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(selected_state.join("frontend-artifacts").is_dir());
    assert!(!root.path().join("frontend-artifacts").exists());
}

#[test]
fn empty_state_environment_rejects_bootstrap_validate_before_evidence_write() {
    let root = TempDir::new().unwrap();
    let evidence = root.path().join("evidence");
    let output = crunch()
        .args(["--json", "bootstrap", "validate"])
        .arg(root.path().join("absent.ncl"))
        .arg("--evidence-dir")
        .arg(&evidence)
        .current_dir(root.path())
        .env("CRUNCH_STATE_DIR", "")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3), "{output:?}");
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("empty-state-dir-env: CRUNCH_STATE_DIR"), "{stderr}");
    assert!(!stderr.contains("panicked"), "{stderr}");
    assert!(!evidence.exists());
    assert_eq!(count_entries(root.path()), 0);
}

#[test]
fn selected_xdg_and_home_fallbacks_store_artifacts_outside_working_directory() {
    for (selected, suffix) in [("XDG_STATE_HOME", "crunch"), ("HOME", ".local/state/crunch")] {
        let root = TempDir::new().unwrap();
        let source = root.path().join("source.txt");
        fs::write(&source, b"fallback admission\n").unwrap();
        let base = root.path().join("selected-base");
        let output = crunch()
            .args(["--json", "artifact", "import"])
            .arg(&source)
            .current_dir(root.path())
            .env_remove("CRUNCH_STATE_DIR")
            .env_remove("XDG_STATE_HOME")
            .env("HOME", "")
            .env(selected, &base)
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        assert!(base.join(suffix).join("frontend-artifacts").is_dir());
        assert!(!root.path().join("frontend-artifacts").exists());
    }
}

#[test]
fn unset_home_retains_tmp_state_fallback_without_creating_cwd_state() {
    let root = TempDir::new().unwrap();
    let output = crunch()
        .args(["--verbose", "refactor", "list"])
        .current_dir(root.path())
        .env_remove("CRUNCH_STATE_DIR")
        .env_remove("XDG_STATE_HOME")
        .env_remove("HOME")
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(r#""state_dir":"/tmp/.local/state/crunch""#), "{stderr}");
    assert_eq!(count_entries(root.path()), 0);
}

#[test]
fn empty_log_directory_environment_rejects_before_log_or_transcript_effects() {
    for command_args in [vec!["log", "--list"], vec![
        "transcript",
        "run",
        "missing.md",
        "--allow-in-place",
    ]] {
        let root = TempDir::new().unwrap();
        let output = crunch()
            .args(["--json", "--state-dir", "relative-state"])
            .args(&command_args)
            .current_dir(root.path())
            .env("CRUNCH_STATE_DIR", "")
            .env("CRUNCH_LOG_DIR", "")
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(3), "{command_args:?}: {output:?}");
        assert!(output.stdout.is_empty());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains("empty-state-dir-env: CRUNCH_LOG_DIR"), "{stderr}");
        assert!(!stderr.contains("panicked"), "{stderr}");
        assert_eq!(count_entries(root.path()), 0, "no transcript scratch or CWD log writes");
    }
}

#[test]
fn selected_log_directory_and_relative_state_override_preserve_log_output() {
    let root = TempDir::new().unwrap();
    let selected_logs = root.path().join("selected-logs");
    fs::create_dir(&selected_logs).unwrap();
    fs::write(selected_logs.join("selected.log"), "# status: success\n# derivation: selected-log\n").unwrap();
    let fallback_logs = root.path().join("relative-state/logs");
    fs::create_dir_all(&fallback_logs).unwrap();
    fs::write(fallback_logs.join("fallback.log"), "# status: success\n# derivation: wrong-log\n").unwrap();
    let output = crunch()
        .args(["--state-dir", "relative-state", "log", "--list"])
        .current_dir(root.path())
        .env("CRUNCH_STATE_DIR", "")
        .env("XDG_STATE_HOME", "")
        .env("HOME", "")
        .env("CRUNCH_LOG_DIR", &selected_logs)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("selected-log"), "{stdout}");
    assert!(!stdout.contains("wrong-log"), "{stdout}");
}

#[cfg(unix)]
#[test]
fn selected_non_utf8_state_environment_keeps_its_exact_path() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let root = TempDir::new().unwrap();
    let selected_state = root.path().join(OsString::from_vec(b"state-\xff".to_vec()));
    let selected_logs = selected_state.join("logs");
    fs::create_dir_all(&selected_logs).unwrap();
    fs::write(selected_logs.join("selected.log"), "# status: success\n# derivation: non-utf8-selected\n").unwrap();
    let home_fallback = root.path().join("home");
    let home_logs = home_fallback.join(".local/state/crunch/logs");
    fs::create_dir_all(&home_logs).unwrap();
    fs::write(home_logs.join("wrong-home.log"), "# status: success\n# derivation: wrong-home\n").unwrap();
    let output = crunch()
        .args(["log", "--list"])
        .current_dir(root.path())
        .env("CRUNCH_STATE_DIR", &selected_state)
        .env("XDG_STATE_HOME", "")
        .env("HOME", &home_fallback)
        .env_remove("CRUNCH_LOG_DIR")
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("non-utf8-selected"), "{stdout}");
    assert!(!stdout.contains("wrong-home"), "{stdout}");
    assert!(!root.path().join("logs").exists());
}

#[cfg(unix)]
#[test]
fn selected_non_utf8_log_environment_keeps_its_exact_path() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let root = TempDir::new().unwrap();
    let selected_logs = root.path().join(OsString::from_vec(b"logs-\xff".to_vec()));
    fs::create_dir(&selected_logs).unwrap();
    fs::write(selected_logs.join("selected.log"), "# status: success\n# derivation: non-utf8-logs\n").unwrap();
    let fallback_logs = root.path().join("relative-state/logs");
    fs::create_dir_all(&fallback_logs).unwrap();
    fs::write(fallback_logs.join("wrong.log"), "# status: success\n# derivation: wrong-state-logs\n").unwrap();
    let output = crunch()
        .args(["--state-dir", "relative-state", "log", "--list"])
        .current_dir(root.path())
        .env("CRUNCH_STATE_DIR", "")
        .env("CRUNCH_LOG_DIR", &selected_logs)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("non-utf8-logs"), "{stdout}");
    assert!(!stdout.contains("wrong-state-logs"), "{stdout}");
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

#[test]
fn operator_contract_exports_sorted_public_clap_paths() {
    let output = crunch()
        .arg("__operator-contract")
        .arg("--mode")
        .arg("raw-descriptors")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let descriptors: Vec<serde_json::Value> = serde_json::from_slice(&output).unwrap();
    let paths = descriptors.iter().map(|descriptor| descriptor["path"].as_str().unwrap()).collect::<Vec<_>>();

    assert!(paths.len() >= PUBLIC_COMMAND_COUNT_MIN);
    assert!(paths.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(paths.contains(&"doctor"));
    assert!(paths.contains(&"attest show"));
    assert!(!paths.iter().any(|path| path.contains("__operator-contract")));
    assert!(!paths.iter().any(|path| path.split_whitespace().any(|part| part == "help")));
}

#[test]
fn operator_contract_checked_files_match_clap_and_policy() {
    crunch()
        .arg("__operator-contract")
        .arg("--mode")
        .arg("check")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .assert()
        .success()
        .stdout(predicate::str::contains("operator command contract: PASS"));
}

#[test]
fn operator_contract_generator_runs_positive_and_negative_self_test() {
    contract_generator()
        .arg("--self-test")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .assert()
        .success()
        .stdout(predicate::str::contains("generator self-test: PASS"));
}
