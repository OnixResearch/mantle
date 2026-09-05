use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use assert_cmd::Command;
use serde_json::Value;
use tempfile::TempDir;

use super::mantle_cmd;

const CLI_TIMEOUT_SECONDS: u64 = 30;
const MISSING_REVISION: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const AUTHORITY_SCHEMA: &str = "mantle-source-built-rust-child-action-authority-v1";
const TEST_TARGET: &str = "x86_64-unknown-linux-gnu";
const TEST_FD_MAX: u32 = 64;
const TEST_STORAGE_BYTES_MAX: u64 = 1_048_576;
const TEST_EXEC_EVENTS_MAX: u32 = 64;
const TEST_EXECUTABLE_MODE: u32 = 0o755;

fn fixture(missing_dependency: bool) -> TempDir {
    let dir = TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("src")).unwrap();
    let dependencies = if missing_dependency {
        format!(
            "[dependencies]\ngit-dep = {{ git = \"https://example.invalid/git-dep\", rev = \"{MISSING_REVISION}\" }}\n"
        )
    } else {
        String::new()
    };
    std::fs::write(
        dir.path().join("Cargo.toml"),
        format!("[package]\nname = \"preflight-probe\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n{dependencies}",),
    )
    .unwrap();
    std::fs::write(
        dir.path().join("Cargo.lock"),
        "version = 4\n\n[[package]]\nname = \"preflight-probe\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    std::fs::write(dir.path().join("src/main.rs"), "fn main() {}\n").unwrap();
    write_authority(dir.path());
    dir
}

fn fixed_executable_json(id: &str, kind: &str, path: &Path, digest: &str) -> String {
    let path = serde_json::to_string(path).unwrap();
    format!(
        "{{\"authority_id\":\"{id}\",\"producer_action_id\":\"fixture-provider\",\"output_identity_blake3\":\"{digest}\",\"path\":{path},\"digest_blake3\":\"{digest}\",\"kind\":\"{kind}\"}}",
    )
}

fn write_authority(root: &Path) {
    // Independent wire fixture for the existing ordered JSON authority contract.
    // The rustc identity check can read these bytes but must never execute them.
    let rustc = root.join("fixture-rustc");
    let bytes = b"fixture compiler identity, not an executable";
    std::fs::write(&rustc, bytes).unwrap();
    std::fs::set_permissions(&rustc, std::fs::Permissions::from_mode(TEST_EXECUTABLE_MODE)).unwrap();
    let digest = blake3::hash(bytes).to_hex().to_string();
    let rustc = std::fs::canonicalize(rustc).unwrap();
    let compiler = fixed_executable_json("a-rustc", "rustc", &rustc, &digest);
    let helper = fixed_executable_json("b-helper", "native-helper", &root.join("missing-helper"), &digest);
    let canonical = format!(
        "{{\"schema\":\"{AUTHORITY_SCHEMA}\",\"stage_id\":\"mantle-stage1\",\"resources\":{{\"parallel_jobs_max\":1,\"open_file_descriptors_max\":{TEST_FD_MAX},\"storage_bytes_max\":{TEST_STORAGE_BYTES_MAX},\"exec_events_per_action_max\":{TEST_EXEC_EVENTS_MAX}}},\"fixed_executables\":[{compiler},{helper}],\"authority_digest_blake3\":\"\"}}",
    );
    let mut hasher = blake3::Hasher::new();
    hasher.update(AUTHORITY_SCHEMA.as_bytes());
    hasher.update(&[0]);
    hasher.update(canonical.as_bytes());
    let mut authority: Value = serde_json::from_str(&canonical).unwrap();
    authority["authority_digest_blake3"] = Value::String(hasher.finalize().to_hex().to_string());
    std::fs::write(root.join("authority.json"), serde_json::to_vec(&authority).unwrap()).unwrap();
}

fn command(root: &Path) -> Command {
    let mut command = mantle_cmd();
    command.timeout(std::time::Duration::from_secs(CLI_TIMEOUT_SECONDS));
    command
        .args(["--json", "rust-plan", "--no-cargo-oracle", "--root"])
        .arg(root)
        .args(["--target", TEST_TARGET])
        .arg("--rustc")
        .arg(root.join("fixture-rustc"))
        .arg("--cargo")
        .arg(root.join("cargo-must-not-run"))
        .arg("--execute-topology")
        .arg("--execution-output-root")
        .arg(root.join("execution"))
        .arg("--rust-child-action-authority")
        .arg(root.join("authority.json"))
        .arg("--rust-child-action-evidence-dir")
        .arg(root.join("actions"));
    command
}

#[test]
fn blocked_graph_retains_its_receipt_before_child_action_admission() {
    let dir = fixture(true);
    let output = command(dir.path()).output().unwrap();
    assert!(!output.status.success());
    let receipt: Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!("missing planning receipt: {error}; stderr: {}", String::from_utf8_lossy(&output.stderr))
    });
    let blockers = receipt["rust_plan"]["native_package_target_planning"]["blockers"].as_array().unwrap();
    assert!(blockers.iter().any(|blocker| blocker["class"] == "missing-captured-git-source-fact"));
    assert_eq!(receipt["topology_execution"]["execution_status"], "blocked");
    assert!(receipt["topology_execution"]["unit_executions"].as_array().unwrap().is_empty());
    assert!(receipt["topology_execution"]["build_script_metadata_runs"].as_array().unwrap().is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("planning blocked before child-action admission"), "{stderr}");
    assert!(!stderr.contains("missing-supported-unit"), "{stderr}");
    assert!(!stderr.contains("missing-helper"), "{stderr}");
    assert!(!dir.path().join("execution").exists());
    assert!(!dir.path().join("actions").exists());
}

#[test]
fn valid_plan_is_ready_without_compilation_or_child_action_effects() {
    let dir = fixture(false);
    let output = mantle_cmd()
        .timeout(std::time::Duration::from_secs(CLI_TIMEOUT_SECONDS))
        .args(["--json", "rust-plan", "--no-cargo-oracle", "--root"])
        .arg(dir.path())
        .arg("--cargo")
        .arg(dir.path().join("cargo-must-not-run"))
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let receipt: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(receipt["unit_derivation_graph"]["ready"], true);
    assert_eq!(receipt["unit_derivation_graph"]["derivation_count"], 1);
    assert!(!dir.path().join("execution").exists());
    assert!(!dir.path().join("actions").exists());
}

#[test]
fn ready_graph_still_requires_all_fixed_executable_bytes() {
    let dir = fixture(false);
    let output = command(dir.path()).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("missing-helper"), "{stderr}");
    assert!(!stderr.contains("planning blocked before child-action admission"), "{stderr}");
    assert!(!dir.path().join("execution").exists());
    assert!(!dir.path().join("actions").exists());
}
