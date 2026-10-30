//! Consumer-visible boundary tests for the `profile` command.
//!
//! The command maps its Clap DTO to a profile operation, runs it through the Rust-operation port
//! and classifies the actual report before printing it. These tests drive the real binary and pin
//! the report on stdout, the exit class, and the absence of side effects when a request is refused.
//! They do not pin diagnostic wording.
//! r[application_architecture.effect_observation_boundary]
//! r[application_architecture.typed_error_ownership]

use std::path::Path;
use std::path::PathBuf;

use assert_cmd::Command;
use serde_json::Value;
use tempfile::TempDir;

/// An invalid DTO or an unadmitted request: the typed internal error class.
const EXIT_REFUSED: i32 = 3;
/// A well-formed request the profile domain blocks (nothing to roll back, unknown generation,
/// unknown slot).
const EXIT_BLOCKED: i32 = 1;

struct Fixture {
    root: TempDir,
}

impl Fixture {
    fn new() -> Self {
        Self {
            root: TempDir::new().unwrap(),
        }
    }

    fn profile(&self) -> PathBuf {
        self.root.path().join("profiles").join("user")
    }

    fn run(&self, profile: &Path, extra: &[&str]) -> std::process::Output {
        Command::cargo_bin("mantle")
            .unwrap()
            .arg("--state-dir")
            .arg(self.root.path().join("state"))
            .arg("profile")
            .arg("--profile")
            .arg(profile)
            .args(extra)
            .current_dir(self.root.path())
            .output()
            .unwrap()
    }
}

fn report(output: &std::process::Output) -> Value {
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn inspection_reports_each_requested_operation_for_an_empty_profile_without_applying() {
    let fixture = Fixture::new();
    let profile = fixture.profile();
    for operation in ["list", "generations"] {
        let value = report(&fixture.run(&profile, &[operation]));
        assert_eq!(value["schema"], "mantle-user-profile-report-v1");
        assert_eq!(value["operation"], operation, "the report must describe the requested operation");
        assert_eq!(value["applied"], false);
        assert_eq!(value["active"], Value::Null);
        assert_eq!(value["retained_generations"], serde_json::json!([]));
        assert_eq!(value["profile"], profile.display().to_string());
    }
}

fn assert_refused_without_report(output: &std::process::Output, code: i32) {
    assert_eq!(output.status.code(), Some(code), "{output:?}");
    assert!(output.stdout.is_empty(), "a refused request must not print a success report: {output:?}");
}

#[test]
fn a_relative_profile_target_is_refused_before_any_filesystem_effect() {
    let fixture = Fixture::new();
    let relative = Path::new("profiles/relative");
    for extra in [&["list"][..], &["--expect-generation", "empty", "rollback"][..]] {
        assert_refused_without_report(&fixture.run(relative, extra), EXIT_REFUSED);
    }
    assert!(!fixture.root.path().join("profiles").exists(), "no directory may be created for a refused target");
}

#[test]
fn a_mutation_without_an_expected_generation_is_refused_before_any_filesystem_effect() {
    let fixture = Fixture::new();
    let profile = fixture.profile();
    for extra in [
        &["rollback"][..],
        &["switch", "1"][..],
        &["remove", "slot"][..],
        &["remove-generation", "1"][..],
    ] {
        assert_refused_without_report(&fixture.run(&profile, extra), EXIT_REFUSED);
    }
    assert!(!profile.exists(), "a refused mutation must not create the profile");
}

#[test]
fn a_non_canonical_expected_generation_is_refused() {
    let fixture = Fixture::new();
    let profile = fixture.profile();
    for generation in ["0", "01", "latest", ""] {
        let output = fixture.run(&profile, &["--expect-generation", generation, "rollback"]);
        assert_refused_without_report(&output, EXIT_REFUSED);
    }
    assert!(!profile.exists());
}

#[test]
fn domain_blockers_exit_as_blocked_and_print_no_success_report() {
    let fixture = Fixture::new();
    let profile = fixture.profile();
    for extra in [
        &["--expect-generation", "empty", "rollback"][..],
        &["--expect-generation", "empty", "--apply", "rollback"][..],
        &["--expect-generation", "empty", "remove-generation", "3"][..],
        &["--expect-generation", "empty", "remove", "absent-slot"][..],
    ] {
        assert_refused_without_report(&fixture.run(&profile, extra), EXIT_BLOCKED);
    }
    // An empty profile stays empty: no generation appeared as a side effect of the blocked requests.
    let value = report(&fixture.run(&profile, &["generations"]));
    assert_eq!(value["retained_generations"], serde_json::json!([]));
    assert_eq!(value["pending_generations"], serde_json::json!([]));
    assert_eq!(value["active"], Value::Null);
}

#[test]
fn an_unknown_subcommand_is_a_usage_error() {
    let fixture = Fixture::new();
    let output = fixture.run(&fixture.profile(), &["install-everything"]);
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert!(output.stdout.is_empty());
}
