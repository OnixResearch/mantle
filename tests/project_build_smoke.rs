//! End-to-end smoke tests for project-aware build commands:
//! `crunch build` (bare), `crunch build .#name`, `crunch run .#name`.
//!
//! Each test scaffolds a temporary `crunch.ncl` project file using
//! `/bin/sh` as builder (no seed) and runs the crunch binary as a
//! subprocess. Requires bwrap + /nix/store.

mod audit_support;

use std::path::Path;
use std::path::PathBuf;

use assert_cmd::Command;
use audit_support::AuditArtifact;
use audit_support::write_command_audit;
use serde::Deserialize;

fn crunch_cmd() -> Command {
    Command::cargo_bin("crunch").expect("crunch binary should be built")
}

fn can_build() -> bool {
    Path::new("/nix/store").exists()
        && std::process::Command::new("bwrap").arg("--version").output().is_ok_and(|o| o.status.success())
}

/// Check if the sandbox shell has coreutils applets (mkdir, chmod).
/// Some static busybox builds are ash-only and lack these.
fn sandbox_has_coreutils() -> bool {
    let shell = std::env::var("SNIX_BUILD_SANDBOX_SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    // Run the shell as busybox and try mkdir --help
    std::process::Command::new(&shell)
        .args(["mkdir", "--help"])
        .output()
        .is_ok_and(|o| o.status.success() || !String::from_utf8_lossy(&o.stderr).contains("applet not found"))
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct BuildJsonReport {
    schema: String,
    counts: BuildJsonCounts,
    outcomes: Vec<BuildJsonOutcome>,
    failed: Vec<BuildJsonFailure>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct BuildJsonCounts {
    succeeded_total: u32,
    built_total: u32,
    cached_total: u32,
    failed_total: u32,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct BuildJsonOutcome {
    label: String,
    cached: bool,
    outputs: Vec<BuildJsonOutput>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct BuildJsonOutput {
    name: String,
    path: PathBuf,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct BuildJsonFailure {
    label: String,
    error: String,
}

/// Minimal crunch.ncl using flat outputs (shell builtins only, no mkdir/chmod).
const PROJECT_NCL: &str = r#"let crunch = import "lib.ncl" in

{
  packages = {
    hello = {
      name = "hello",
      builder = "/bin/sh",
      args = ["-c", "echo 'Hello from crunch project!' > $out"],
      addressing_mode = 'input-addressed,
    } | crunch.Derivation,

    goodbye = {
      name = "goodbye",
      builder = "/bin/sh",
      args = ["-c", "echo 'Goodbye!' > $out"],
      addressing_mode = 'input-addressed,
    } | crunch.Derivation,
  },

  checks = {
    lint = {
      name = "lint-check",
      builder = "/bin/sh",
      args = ["-c", "echo 'lint ok' > $out"],
      addressing_mode = 'input-addressed,
    } | crunch.Derivation,
  },

  default = {
    package = "hello",
  },
} | crunch.Project
"#;

/// Project NCL with directory outputs (needs full busybox with mkdir/chmod).
const PROJECT_NCL_DIR: &str = r#"let crunch = import "lib.ncl" in

{
  packages = {
    hello = {
      name = "hello",
      builder = "/bin/sh",
      args = ["-c", "BB=/bin/busybox; $BB mkdir -p $out/bin && printf '#!/bin/sh\necho \"Hello from crunch project!\"\n' > $out/bin/hello && $BB chmod +x $out/bin/hello"],
      addressing_mode = 'input-addressed,
    } | crunch.Derivation,
  },

  default = {
    package = "hello",
  },
} | crunch.Project
"#;

/// Scaffold a project dir with crunch.ncl, run a command, return output.
struct ProjectFixture {
    dir: tempfile::TempDir,
    store: tempfile::TempDir,
    state: tempfile::TempDir,
}

impl ProjectFixture {
    fn new(ncl: &str) -> Self {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("crunch.ncl"), ncl).unwrap();
        let store = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        Self { dir, store, state }
    }

    fn cmd(&self) -> Command {
        let mut cmd = crunch_cmd();
        cmd.current_dir(self.dir.path());
        cmd.arg("--json");
        cmd.arg("--store").arg(self.store.path());
        cmd.arg("--state-dir").arg(self.state.path());
        cmd
    }

    fn build(&self, extra_args: &[&str]) -> BuildResult {
        let mut cmd = self.cmd();
        cmd.arg("build").arg("--no-substitute");
        for arg in extra_args {
            cmd.arg(arg);
        }

        let command_strs = build_command_strings(self.store.path(), self.state.path());
        let output = cmd.output().expect("should execute");
        let stdout = String::from_utf8(output.stdout.clone()).unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

        let report: Option<BuildJsonReport> = serde_json::from_str(&stdout).ok();

        if let Some(ref r) = report {
            let artifacts: Vec<AuditArtifact<'_>> = r
                .outcomes
                .iter()
                .flat_map(|o| o.outputs.iter())
                .map(|o| AuditArtifact {
                    label: o.name.as_str(),
                    path: o.path.as_path(),
                })
                .collect();
            let _ = write_command_audit(
                "project_build_smoke",
                "build",
                self.dir.path(),
                &command_strs,
                &output,
                &artifacts,
                &[
                    ("CRUNCH_STATE_DIR", self.state.path().display().to_string()),
                    ("CRUNCH_STORE_DIR", self.store.path().display().to_string()),
                ],
            );
        }

        BuildResult {
            output,
            stdout,
            stderr,
            report,
        }
    }
}

struct BuildResult {
    output: std::process::Output,
    stdout: String,
    stderr: String,
    report: Option<BuildJsonReport>,
}

impl BuildResult {
    fn assert_success(&self) -> &BuildJsonReport {
        let report = self
            .report
            .as_ref()
            .unwrap_or_else(|| panic!("expected JSON report in stdout:\n{}\nstderr:\n{}", self.stdout, self.stderr));
        assert!(
            self.output.status.success(),
            "build failed (exit {}):\nstderr: {}",
            self.output.status.code().unwrap_or(-1),
            self.stderr
        );
        assert!(
            report.failed.is_empty(),
            "build had failures: {:?}",
            report.failed.iter().map(|f| format!("{}: {}", f.label, f.error)).collect::<Vec<_>>()
        );
        report
    }

    fn first_output_path(&self) -> PathBuf {
        let report = self.assert_success();
        report
            .outcomes
            .first()
            .and_then(|o| o.outputs.first())
            .map(|o| o.path.clone())
            .expect("should have at least one output")
    }
}

fn build_command_strings(store: &Path, state: &Path) -> Vec<String> {
    // assert_cmd doesn't expose the full argv easily; reconstruct
    vec![
        "crunch".into(),
        "--json".into(),
        "--store".into(),
        store.display().to_string(),
        "--state-dir".into(),
        state.display().to_string(),
        "build".into(),
        "--no-substitute".into(),
    ]
}

// ── Tests ──────────────────────────────────────────────────────────

#[test]
fn project_build_default_package() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let fixture = ProjectFixture::new(PROJECT_NCL);
    let result = fixture.build(&[]);
    let report = result.assert_success();

    // Default package is "hello"
    assert_eq!(report.counts.succeeded_total, 1);
    assert_eq!(report.outcomes.len(), 1);
    assert_eq!(report.outcomes[0].label, "hello");

    let out = result.first_output_path();
    assert!(out.exists(), "output should exist: {}", out.display());
    let content = std::fs::read_to_string(&out).unwrap();
    assert_eq!(content.trim(), "Hello from crunch project!");
}

#[test]
fn project_build_selector_by_name() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let fixture = ProjectFixture::new(PROJECT_NCL);
    let result = fixture.build(&[".#goodbye"]);
    let report = result.assert_success();

    assert_eq!(report.counts.succeeded_total, 1);
    assert_eq!(report.outcomes[0].label, "goodbye");

    let out = result.first_output_path();
    let content = std::fs::read_to_string(&out).unwrap();
    assert_eq!(content.trim(), "Goodbye!");
}

#[test]
fn project_build_qualified_selector() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let fixture = ProjectFixture::new(PROJECT_NCL);
    let result = fixture.build(&[".#packages.goodbye"]);
    let report = result.assert_success();

    assert_eq!(report.counts.succeeded_total, 1);
    assert_eq!(report.outcomes[0].label, "goodbye");
}

#[test]
fn project_build_check_selector() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let fixture = ProjectFixture::new(PROJECT_NCL);
    let result = fixture.build(&[".#lint"]);
    let report = result.assert_success();

    assert_eq!(report.counts.succeeded_total, 1);
    assert_eq!(report.outcomes[0].label, "lint-check");

    let out = result.first_output_path();
    let content = std::fs::read_to_string(&out).unwrap();
    assert_eq!(content.trim(), "lint ok");
}

#[test]
fn project_build_qualified_check_selector() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let fixture = ProjectFixture::new(PROJECT_NCL);
    let result = fixture.build(&[".#checks.lint"]);
    let report = result.assert_success();

    assert_eq!(report.counts.succeeded_total, 1);
}

#[test]
fn project_build_no_crunch_ncl_fails() {
    // Even without bwrap, project resolution should fail with a clear error
    let dir = tempfile::tempdir().unwrap();
    let store = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();

    let output = crunch_cmd()
        .current_dir(dir.path())
        .arg("--json")
        .arg("--store")
        .arg(store.path())
        .arg("--state-dir")
        .arg(state.path())
        .arg("build")
        .arg("--no-substitute")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("crunch.ncl") && (stderr.contains("not found") || stderr.contains("no crunch.ncl")),
        "error should mention missing crunch.ncl, got: {stderr}"
    );
}

#[test]
fn project_build_invalid_selector_fails() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let fixture = ProjectFixture::new(PROJECT_NCL);
    let result = fixture.build(&[".#nonexistent"]);

    assert!(!result.output.status.success(), "build with nonexistent selector should fail");
}

#[test]
fn project_run_hello() {
    if !can_build() || !sandbox_has_coreutils() {
        eprintln!("skipping: bwrap, /nix/store, or full busybox not available");
        return;
    }

    let fixture = ProjectFixture::new(PROJECT_NCL_DIR);

    let output = crunch_cmd()
        .current_dir(fixture.dir.path())
        .arg("--store")
        .arg(fixture.store.path())
        .arg("--state-dir")
        .arg(fixture.state.path())
        .arg("run")
        .arg("--no-substitute")
        .arg(".#hello")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "crunch run should succeed, exit={}, stderr:\n{}",
        output.status.code().unwrap_or(-1),
        stderr,
    );
    assert!(
        stdout.contains("Hello from crunch project!"),
        "stdout should contain the hello message, got:\n{stdout}\nstderr:\n{stderr}",
    );
}

#[test]
fn project_run_default() {
    if !can_build() || !sandbox_has_coreutils() {
        eprintln!("skipping: bwrap, /nix/store, or full busybox not available");
        return;
    }

    let fixture = ProjectFixture::new(PROJECT_NCL_DIR);

    // Bare `crunch run` should use default.package = "hello"
    let output = crunch_cmd()
        .current_dir(fixture.dir.path())
        .arg("--store")
        .arg(fixture.store.path())
        .arg("--state-dir")
        .arg(fixture.state.path())
        .arg("run")
        .arg("--no-substitute")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "crunch run (default) should succeed, stderr:\n{stderr}",);
    assert!(stdout.contains("Hello from crunch project!"), "default run should execute hello, got:\n{stdout}",);
}

#[test]
fn project_run_with_args() {
    if !can_build() || !sandbox_has_coreutils() {
        eprintln!("skipping: bwrap, /nix/store, or full busybox not available");
        return;
    }

    // Build a project whose executable echoes its arguments
    let ncl = r#"let crunch = import "lib.ncl" in

{
  packages.echo-args = {
    name = "echo-args",
    builder = "/bin/sh",
    args = ["-c", "BB=/bin/busybox; $BB mkdir -p $out/bin && printf '#!/bin/sh\necho \"args: $*\"\n' > $out/bin/echo-args && $BB chmod +x $out/bin/echo-args"],
    addressing_mode = 'input-addressed,
  } | crunch.Derivation,

  default.package = "echo-args",
} | crunch.Project
"#;

    let fixture = ProjectFixture::new(ncl);

    let output = crunch_cmd()
        .current_dir(fixture.dir.path())
        .arg("--store")
        .arg(fixture.store.path())
        .arg("--state-dir")
        .arg(fixture.state.path())
        .arg("run")
        .arg("--no-substitute")
        .arg("--")
        .arg("foo")
        .arg("bar")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "crunch run with args should succeed, stderr:\n{stderr}",);
    assert!(stdout.contains("args: foo bar"), "should pass through args, got:\n{stdout}",);
}

#[test]
fn project_run_no_bin_dir_fails() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    // Package produces a flat file, not a directory with bin/
    let ncl = r#"let crunch = import "lib.ncl" in

{
  packages.flat = {
    name = "flat-output",
    builder = "/bin/sh",
    args = ["-c", "echo 'just a file' > $out"],
    addressing_mode = 'input-addressed,
  } | crunch.Derivation,

  default.package = "flat",
} | crunch.Project
"#;

    let fixture = ProjectFixture::new(ncl);

    let output = crunch_cmd()
        .current_dir(fixture.dir.path())
        .arg("--store")
        .arg(fixture.store.path())
        .arg("--state-dir")
        .arg(fixture.state.path())
        .arg("run")
        .arg("--no-substitute")
        .output()
        .unwrap();

    assert!(!output.status.success(), "crunch run should fail without bin/ dir");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("bin") || stderr.contains("no executables") || stderr.contains("no bin"),
        "error should mention missing bin dir, got:\n{stderr}",
    );
}

#[test]
fn project_build_deterministic_paths() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    // Two independent builds of the same project should produce the
    // same output path (same derivation hash).
    let fixture1 = ProjectFixture::new(PROJECT_NCL);
    let fixture2 = ProjectFixture::new(PROJECT_NCL);

    let r1 = fixture1.build(&[".#hello"]);
    let r2 = fixture2.build(&[".#hello"]);

    let _report1 = r1.assert_success();
    let _report2 = r2.assert_success();

    let name1 = r1.first_output_path().file_name().unwrap().to_owned();
    let name2 = r2.first_output_path().file_name().unwrap().to_owned();
    assert_eq!(name1, name2, "same project should produce same hash-name");
}

#[test]
fn project_build_from_subdirectory() {
    if !can_build() {
        eprintln!("skipping: bwrap or /nix/store not available");
        return;
    }

    let fixture = ProjectFixture::new(PROJECT_NCL);

    // Create a subdirectory and run from there — should walk up to find crunch.ncl
    let sub = fixture.dir.path().join("src").join("deep");
    std::fs::create_dir_all(&sub).unwrap();

    let output = crunch_cmd()
        .current_dir(&sub)
        .arg("--json")
        .arg("--store")
        .arg(fixture.store.path())
        .arg("--state-dir")
        .arg(fixture.state.path())
        .arg("build")
        .arg("--no-substitute")
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "build from subdirectory should succeed, stderr:\n{stderr}",);

    let report: BuildJsonReport =
        serde_json::from_str(&stdout).unwrap_or_else(|e| panic!("should parse JSON report: {e}\nstdout:\n{stdout}"));
    assert_eq!(report.outcomes[0].label, "hello", "should find default package");
}
