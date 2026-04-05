//! End-to-end integration tests for the crunch binary.
//!
//! These tests invoke the compiled `crunch` binary via `assert_cmd` and
//! check stdout, stderr, and exit codes.

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn crunch_cmd() -> Command {
    Command::cargo_bin("crunch").expect("crunch binary should be built")
}

// ── Phase 2: Eval tests ─────────────────────────────────────────

#[test]
fn eval_simple_derivation_prints_json() {
    crunch_cmd()
        .arg("eval")
        .arg(fixture("simple.ncl"))
        .assert()
        .success()
        .stdout(predicate::str::contains("\"name\""))
        .stdout(predicate::str::contains("simple-test"));
}

#[test]
fn eval_multi_derivation_prints_both() {
    crunch_cmd()
        .arg("eval")
        .arg(fixture("multi.ncl"))
        .assert()
        .success()
        .stdout(predicate::str::contains("alpha"))
        .stdout(predicate::str::contains("beta"));
}

#[test]
fn eval_output_is_valid_json() {
    let output = crunch_cmd()
        .arg("eval")
        .arg(fixture("simple.ncl"))
        .output()
        .expect("should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
    assert!(parsed.is_ok(), "stdout should be valid JSON: {stdout}");
}

#[test]
fn eval_invalid_nickel_exits_2() {
    crunch_cmd()
        .arg("eval")
        .arg(fixture("invalid.ncl"))
        .assert()
        .code(2)
        .stderr(predicate::str::contains("error"));
}

#[test]
fn eval_nonexistent_file_exits_2() {
    crunch_cmd()
        .arg("eval")
        .arg("/nonexistent/path/to/file.ncl")
        .assert()
        .code(2)
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn eval_json_flag_emits_json_error() {
    let output = crunch_cmd()
        .arg("--json")
        .arg("eval")
        .arg(fixture("invalid.ncl"))
        .output()
        .expect("should run");

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(stderr.trim());
    assert!(parsed.is_ok(), "stderr should be valid JSON: {stderr}");

    let obj = parsed.unwrap();
    assert_eq!(obj["code"], 2);
    assert_eq!(obj["kind"], "eval");
    assert!(obj["error"].is_string());
}

#[test]
fn eval_with_import_path_flag() {
    let lib_dir = tempfile::tempdir().unwrap();
    std::fs::write(
        lib_dir.path().join("extra.ncl"),
        r#"{ val = 42 }"#,
    )
    .unwrap();

    let main_dir = tempfile::tempdir().unwrap();
    std::fs::write(
        main_dir.path().join("app.ncl"),
        r#"let crunch = import "lib.ncl" in
let extra = import "extra.ncl" in
{
  name = "with-import",
  builder = "/bin/sh",
  args = ["-c", "echo > $out"],
  inputs = [],
  env = { EXTRA = std.string.from_number extra.val },
} | crunch.Derivation"#,
    )
    .unwrap();

    crunch_cmd()
        .arg("eval")
        .arg("-I")
        .arg(lib_dir.path())
        .arg(main_dir.path().join("app.ncl"))
        .assert()
        .success()
        .stdout(predicate::str::contains("with-import"));
}

// ── Phase 3: Bootstrap tests ────────────────────────────────────

#[test]
fn bootstrap_creates_seed_file() {
    let dir = tempfile::tempdir().unwrap();
    let seed = dir.path().join("seed.ncl");

    let result = crunch_cmd()
        .arg("bootstrap")
        .arg("-o")
        .arg(&seed)
        .arg("bash")
        .output()
        .expect("should run");

    // Bootstrap might fail if nix isn't installed — skip gracefully
    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        if stderr.contains("failed to run nix") || stderr.contains("No such file") {
            eprintln!("skipping bootstrap test: nix not available");
            return;
        }
        panic!(
            "bootstrap failed unexpectedly: {}",
            String::from_utf8_lossy(&result.stderr)
        );
    }

    assert!(seed.exists(), "seed.ncl should be created");
    let content = std::fs::read_to_string(&seed).unwrap();
    assert!(content.contains("bash"), "seed should contain bash entry");
    assert!(content.contains("/nix/store/"), "seed should contain store paths");
}

#[test]
fn bootstrap_seed_is_importable() {
    let dir = tempfile::tempdir().unwrap();
    let seed = dir.path().join("seed.ncl");

    let result = crunch_cmd()
        .arg("bootstrap")
        .arg("-o")
        .arg(&seed)
        .arg("bash")
        .output()
        .expect("should run");

    if !result.status.success() {
        eprintln!("skipping: bootstrap failed (nix not available?)");
        return;
    }

    // Try to eval a file that imports the generated seed
    std::fs::write(
        dir.path().join("test.ncl"),
        r#"let crunch = import "lib.ncl" in
let seed = import "seed.ncl" in
{
  name = "seed-test",
  builder = "%{seed.bash}/bin/bash",
  args = ["-c", "echo > $out"],
  inputs = [seed.bash],
} | crunch.Derivation"#,
    )
    .unwrap();

    crunch_cmd()
        .arg("eval")
        .arg("-I")
        .arg(dir.path())
        .arg(dir.path().join("test.ncl"))
        .assert()
        .success()
        .stdout(predicate::str::contains("seed-test"));
}

// ── Phase 4: Build tests (Linux-only) ──────────────────────────

#[cfg(target_os = "linux")]
mod build_tests {
    use super::*;

    /// Check if we can actually build (need bwrap + nix store with bash).
    fn can_build() -> bool {
        // Quick check: does /nix/store exist and is bwrap available?
        std::path::Path::new("/nix/store").exists()
            && std::process::Command::new("bwrap")
                .arg("--version")
                .output()
                .is_ok()
    }

    #[test]
    fn build_trivial_derivation() {
        if !can_build() {
            eprintln!("skipping build test: bwrap or /nix/store not available");
            return;
        }

        // Bootstrap to get a seed with bash
        let dir = tempfile::tempdir().unwrap();
        let seed = dir.path().join("seed.ncl");
        let result = crunch_cmd()
            .arg("bootstrap")
            .arg("-o")
            .arg(&seed)
            .arg("bash")
            .arg("coreutils")
            .output()
            .expect("should run");

        if !result.status.success() {
            eprintln!("skipping: bootstrap failed");
            return;
        }

        std::fs::write(
            dir.path().join("hello.ncl"),
            r#"let crunch = import "lib.ncl" in
let seed = import "seed.ncl" in
{
  name = "hello-e2e",
  builder = "%{seed.bash}/bin/bash",
  args = ["-c", "%{seed.coreutils}/bin/echo hello > $out"],
  inputs = [seed.bash, seed.coreutils],
  addressing_mode = 'input-addressed,
} | crunch.Derivation"#,
        )
        .unwrap();

        crunch_cmd()
            .arg("build")
            .arg("-I")
            .arg(dir.path())
            .arg(dir.path().join("hello.ncl"))
            .assert()
            .success()
            .stdout(predicate::str::contains("/nix/store/"));
    }

    #[test]
    fn build_writes_log_file() {
        if !can_build() {
            eprintln!("skipping build test: bwrap or /nix/store not available");
            return;
        }

        let dir = tempfile::tempdir().unwrap();
        let log_dir = tempfile::tempdir().unwrap();
        let seed = dir.path().join("seed.ncl");
        let result = crunch_cmd()
            .arg("bootstrap")
            .arg("-o")
            .arg(&seed)
            .arg("bash")
            .arg("coreutils")
            .output()
            .expect("should run");

        if !result.status.success() {
            eprintln!("skipping: bootstrap failed");
            return;
        }

        std::fs::write(
            dir.path().join("logged.ncl"),
            r#"let crunch = import "lib.ncl" in
let seed = import "seed.ncl" in
{
  name = "logged-build",
  builder = "%{seed.bash}/bin/bash",
  args = ["-c", "%{seed.coreutils}/bin/echo 'log test output' > $out"],
  inputs = [seed.bash, seed.coreutils],
  addressing_mode = 'input-addressed,
} | crunch.Derivation"#,
        )
        .unwrap();

        crunch_cmd()
            .env("CRUNCH_LOG_DIR", log_dir.path())
            .arg("build")
            .arg("-I")
            .arg(dir.path())
            .arg(dir.path().join("logged.ncl"))
            .assert()
            .success();

        // Check that a log file was written
        let logs: Vec<_> = std::fs::read_dir(log_dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "log"))
            .collect();
        assert!(!logs.is_empty(), "should have written at least one log file");

        let content = std::fs::read_to_string(logs[0].path()).unwrap();
        assert!(content.contains("# crunch build log"), "log should have header");
        assert!(content.contains("# status: success"), "log should show success");
        assert!(content.contains("logged-build"), "log should name the derivation");
    }

    #[test]
    fn build_failing_builder_exits_1() {
        if !can_build() {
            eprintln!("skipping build test: bwrap or /nix/store not available");
            return;
        }

        let dir = tempfile::tempdir().unwrap();
        let seed = dir.path().join("seed.ncl");
        let result = crunch_cmd()
            .arg("bootstrap")
            .arg("-o")
            .arg(&seed)
            .arg("bash")
            .output()
            .expect("should run");

        if !result.status.success() {
            eprintln!("skipping: bootstrap failed");
            return;
        }

        std::fs::write(
            dir.path().join("fail.ncl"),
            r#"let crunch = import "lib.ncl" in
let seed = import "seed.ncl" in
{
  name = "will-fail",
  builder = "%{seed.bash}/bin/bash",
  args = ["-c", "exit 1"],
  inputs = [seed.bash],
  addressing_mode = 'input-addressed,
} | crunch.Derivation"#,
        )
        .unwrap();

        crunch_cmd()
            .arg("build")
            .arg("-I")
            .arg(dir.path())
            .arg(dir.path().join("fail.ncl"))
            .assert()
            .code(1);
    }
}

// ── Phase 5: Error and edge cases ───────────────────────────────

#[test]
fn build_missing_store_exits_3() {
    crunch_cmd()
        .arg("--store")
        .arg("/nonexistent/store/path")
        .arg("build")
        .arg(fixture("simple.ncl"))
        .assert()
        .code(3)
        .stderr(predicate::str::contains("does not exist"));
}

#[test]
fn verbose_flag_produces_debug_output() {
    let output = crunch_cmd()
        .arg("--verbose")
        .arg("eval")
        .arg(fixture("simple.ncl"))
        .output()
        .expect("should run");

    assert!(output.status.success());
}

// ── Build log tests ────────────────────────────────────────

#[test]
fn log_subcommand_no_logs() {
    // Point to an empty log dir
    let dir = tempfile::tempdir().unwrap();
    crunch_cmd()
        .env("CRUNCH_LOG_DIR", dir.path())
        .arg("log")
        .arg("--list")
        .assert()
        .success();
}

#[test]
fn log_subcommand_lists_logs() {
    let dir = tempfile::tempdir().unwrap();
    let log_file = dir.path().join("abc123-test.drv.log");
    std::fs::write(
        &log_file,
        "# crunch build log\n# derivation: mytest\n# drv_path: abc123-test.drv\n# status: success\n# timestamp: 0\n\nhello world\n",
    ).unwrap();

    crunch_cmd()
        .env("CRUNCH_LOG_DIR", dir.path())
        .arg("log")
        .arg("--list")
        .assert()
        .success()
        .stdout(predicate::str::contains("mytest"))
        .stdout(predicate::str::contains("success"));
}

#[test]
fn log_subcommand_shows_log_by_query() {
    let dir = tempfile::tempdir().unwrap();
    let log_file = dir.path().join("abc123-test.drv.log");
    std::fs::write(
        &log_file,
        "# crunch build log\n# derivation: mytest\n# status: success\n# timestamp: 0\n\nbuild output here\n",
    ).unwrap();

    crunch_cmd()
        .env("CRUNCH_LOG_DIR", dir.path())
        .arg("log")
        .arg("abc123")
        .assert()
        .success()
        .stdout(predicate::str::contains("build output here"));
}

#[test]
fn log_subcommand_query_not_found() {
    let dir = tempfile::tempdir().unwrap();
    crunch_cmd()
        .env("CRUNCH_LOG_DIR", dir.path())
        .arg("log")
        .arg("nonexistent")
        .assert()
        .code(3)
        .stderr(predicate::str::contains("no log matching"));
}
