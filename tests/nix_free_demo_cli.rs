use assert_cmd::Command;
use serde_json::Value;
use tempfile::TempDir;

const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const SUMMARY_FILE_NAME: &str = "summary.json";
const MISSING_FIXED_POINT: &str = "missing-fixed-point-evidence";
const MISSING_GUARD: &str = "missing-guard-evidence";
const SUCCESS_VERDICT: &str = "success";
const DEMO_PROFILE: &str = "source-root-cargo-free-fixed-point";
const CLAIMABLE_TEXT: &str = "Nix-free fixed-point demo: claimable";

fn mantle_cmd() -> Command {
    Command::cargo_bin("mantle").expect("mantle binary should be built")
}

#[test]
fn nix_free_demo_cli_validates_claimable_bundle_as_json() {
    let temp = TempDir::new().expect("tempdir should be created");
    let summary_path = write_summary(temp.path(), claimable_summary());

    let output = mantle_cmd()
        .args(["--json", "nix-free-demo", "validate", path_str(&summary_path)])
        .assert()
        .success()
        .get_output()
        .clone();
    let report: Value = serde_json::from_slice(&output.stdout).expect("stdout should be JSON");

    assert_eq!(report["schema"], "mantle-nix-free-demo-cli-v1");
    assert_eq!(report["profile"], DEMO_PROFILE);
    assert_eq!(report["verdict"], SUCCESS_VERDICT);
    assert_eq!(report["demo_claimable"], true);
    assert!(report["diagnostics"].as_array().unwrap().is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn nix_free_demo_cli_rejects_missing_fixed_point_without_success_claim() {
    let temp = TempDir::new().expect("tempdir should be created");
    let mut summary = claimable_summary();
    summary["stage2_binary_blake3"] = Value::String(DIGEST_B.to_string());
    let summary_path = write_summary(temp.path(), summary);

    let output = mantle_cmd()
        .args(["nix-free-demo", "validate", path_str(&summary_path)])
        .assert()
        .failure()
        .get_output()
        .clone();
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");

    assert!(stdout.contains("Nix-free demo bundle: not claimable"));
    assert!(stdout.contains(MISSING_FIXED_POINT));
    assert!(!stdout.contains(CLAIMABLE_TEXT));
    assert!(output.stderr.is_empty());
}

#[test]
fn nix_free_demo_cli_rejects_missing_guard_as_json() {
    let temp = TempDir::new().expect("tempdir should be created");
    let mut summary = claimable_summary();
    let guards = summary["guards"].as_array_mut().expect("guards should be an array");
    guards.retain(|guard| guard["guard"] != "rustup");
    let summary_path = write_summary(temp.path(), summary);

    let output = mantle_cmd()
        .args(["--json", "nix-free-demo", "validate", path_str(&summary_path)])
        .assert()
        .failure()
        .get_output()
        .clone();
    let report: Value = serde_json::from_slice(&output.stdout).expect("stdout should be JSON");
    let diagnostics = report["diagnostics"].as_array().expect("diagnostics should be an array");

    assert_eq!(report["demo_claimable"], false);
    assert!(diagnostics.iter().any(|diagnostic| diagnostic["code"] == MISSING_GUARD));
    assert!(!String::from_utf8(output.stdout).unwrap().contains(CLAIMABLE_TEXT));
    assert!(output.stderr.is_empty());
}

#[test]
fn nix_free_demo_cli_renders_readme_from_summary() {
    let temp = TempDir::new().expect("tempdir should be created");
    let summary_path = write_summary(temp.path(), claimable_summary());

    let output = mantle_cmd()
        .args(["nix-free-demo", "readme", path_str(&summary_path)])
        .assert()
        .success()
        .get_output()
        .clone();
    let stdout = String::from_utf8(output.stdout).expect("stdout should be UTF-8");

    assert!(stdout.contains("# Mantle fixed-point demo bundle"));
    assert!(stdout.contains(CLAIMABLE_TEXT));
    assert!(stdout.contains(DIGEST_A));
    assert!(stdout.contains("not release reproducibility"));
    assert!(output.stderr.is_empty());
}

fn write_summary(root: &std::path::Path, summary: Value) -> std::path::PathBuf {
    let path = root.join(SUMMARY_FILE_NAME);
    let bytes = serde_json::to_vec_pretty(&summary).expect("summary should serialize");
    std::fs::write(&path, bytes).expect("summary should be written");
    path
}

fn claimable_summary() -> Value {
    serde_json::json!({
        "schema": "mantle-nix-free-demo-summary-v1",
        "profile": DEMO_PROFILE,
        "fixed_point_verdict": SUCCESS_VERDICT,
        "stage1_binary_blake3": DIGEST_A,
        "stage2_binary_blake3": DIGEST_A,
        "source_root_identity": "source-root-v1",
        "toolchain_policy_digest_blake3": DIGEST_B,
        "command_owned_wrappers": [],
        "guards": [
            guard("cargo"),
            guard("nix"),
            guard("rustup"),
            guard("ambient-wrapper")
        ],
        "replay_hints": ["mantle self-build --cargo-free --fixed-point"],
        "non_claims": ["not release reproducibility"]
    })
}

fn guard(name: &str) -> Value {
    serde_json::json!({
        "guard": name,
        "status": "denied",
        "diagnostic": format!("{name} denied by fixture")
    })
}

fn path_str(path: &std::path::Path) -> &str {
    path.to_str().expect("test paths should be UTF-8")
}
