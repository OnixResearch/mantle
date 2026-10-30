//! Consumer-visible boundary tests for the pure and local-fixture `remote gateway` operations.
//!
//! Each operation plans its effect, runs it through the gateway port and classifies the actual
//! evidence before it prints a terminal report. These tests drive the real binary and pin what an
//! operator or script can observe: the report on stdout, the exit code, and the absence of side
//! effects when an operation is refused. They do not pin diagnostic wording.
//! r[application_architecture.effect_observation_boundary]
#![cfg(target_os = "linux")]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use assert_cmd::Command;
use serde_json::Value;
use tempfile::TempDir;

const STORE_PATH: &str = "/nix/store/00000000000000000000000000000000-fixture";
const EXIT_INTERNAL: i32 = 3;
const EXIT_USAGE: i32 = 2;

fn mantle(state: &Path) -> Command {
    let mut command = Command::cargo_bin("mantle").unwrap();
    command.arg("--state-dir").arg(state).args(["remote", "gateway"]);
    command
}

fn read_binding(state: &Path, operation: &str, generation: u64) -> Value {
    let output = mantle(state)
        .args(["nix-store-read-binding", "--path", STORE_PATH, "--operation", operation])
        .args(["--endpoint-generation", &generation.to_string()])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    serde_json::from_slice(&output.stdout).unwrap()
}

fn assert_refused(output: &std::process::Output, code: i32) {
    assert_eq!(output.status.code(), Some(code), "{output:?}");
    assert!(output.stdout.is_empty(), "a refused operation must not print a success report: {output:?}");
}

#[test]
fn store_read_binding_digest_is_deterministic_and_bound_to_generation_and_opcode() {
    let root = TempDir::new().unwrap();
    let state = root.path().join("state");
    let first = read_binding(&state, "is-valid-path", 1);
    assert_eq!(first["operation"], "store-read");
    assert_eq!(first["target"], STORE_PATH);
    assert_eq!(first["request_digest"].as_array().unwrap().len(), 32);
    assert_eq!(read_binding(&state, "is-valid-path", 1), first, "the binding must be reproducible");
    assert_ne!(read_binding(&state, "is-valid-path", 2)["request_digest"], first["request_digest"]);
    assert_ne!(read_binding(&state, "query-path-info", 1)["request_digest"], first["request_digest"]);
    assert!(!state.exists(), "a pure binding must not create state");
}

#[test]
fn store_read_binding_refuses_zero_generation_and_non_store_paths_without_a_report() {
    let root = TempDir::new().unwrap();
    let state = root.path().join("state");
    for (path, generation) in [(STORE_PATH, "0"), ("relative/path", "1"), ("", "1")] {
        let output = mantle(&state)
            .args(["nix-store-read-binding", "--path", path, "--operation", "is-valid-path"])
            .args(["--endpoint-generation", generation])
            .output()
            .unwrap();
        assert_refused(&output, EXIT_INTERNAL);
    }
    let unknown = mantle(&state)
        .args([
            "nix-store-read-binding",
            "--path",
            STORE_PATH,
            "--operation",
            "not-an-opcode",
        ])
        .args(["--endpoint-generation", "1"])
        .output()
        .unwrap();
    assert_refused(&unknown, EXIT_USAGE);
    assert!(!state.exists());
}

fn upload_header(directory: &Path, nar_size: u64, mode: u32) -> std::path::PathBuf {
    let header = serde_json::json!({
        "version": 1,
        "logical_store_path": STORE_PATH,
        "nar_hash": "a".repeat(64),
        "nar_size": nar_size,
        "references": [],
        "deriver": "",
        "content_address": "",
        "endpoint_generation": 1,
    });
    let path = directory.join(format!("header-{nar_size}-{mode:o}.json"));
    fs::write(&path, serde_json::to_vec(&header).unwrap()).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(mode)).unwrap();
    path
}

fn upload_binding(state: &Path, header: &Path, max_bytes: u64) -> std::process::Output {
    mantle(state)
        .args(["nix-upload-binding", "--header"])
        .arg(header)
        .args(["--max-bytes", &max_bytes.to_string()])
        .output()
        .unwrap()
}

#[test]
fn upload_binding_commits_to_the_byte_bound_and_refuses_an_unbounded_or_exposed_header() {
    let root = TempDir::new().unwrap();
    let state = root.path().join("state");
    let header = upload_header(root.path(), 5, 0o600);
    let ten = upload_binding(&state, &header, 10);
    assert_eq!(ten.status.code(), Some(0), "{ten:?}");
    let report: Value = serde_json::from_slice(&ten.stdout).unwrap();
    assert_eq!(report["operation"], "input-upload");
    assert_eq!(report["target"], STORE_PATH);
    let eleven: Value = serde_json::from_slice(&upload_binding(&state, &header, 11).stdout).unwrap();
    assert_ne!(report["request_digest"], eleven["request_digest"], "the signed tuple must include the bound");
    // The declared NAR is larger than the operator's bound: admission must fail before any digest is
    // printed.
    assert_refused(&upload_binding(&state, &header, 4), EXIT_INTERNAL);
    // A header another local user could have written is not an owner-private authority input.
    let exposed = upload_header(root.path(), 5, 0o644);
    assert_refused(&upload_binding(&state, &exposed, 10), EXIT_INTERNAL);
    assert_refused(&upload_binding(&state, &root.path().join("missing.json"), 10), EXIT_INTERNAL);
    assert!(!state.exists());
}

#[test]
fn provision_quota_initializes_once_and_never_rewrites_an_existing_ledger() {
    let root = TempDir::new().unwrap();
    let state = root.path().join("state");
    let directory = root.path().join("quota");
    fs::create_dir(&directory).unwrap();
    fs::set_permissions(&directory, fs::Permissions::from_mode(0o700)).unwrap();
    let first = mantle(&state).args(["provision-quota", "--directory"]).arg(&directory).output().unwrap();
    assert_eq!(first.status.code(), Some(0), "{first:?}");
    let report: Value = serde_json::from_slice(&first.stdout).unwrap();
    assert_eq!(report["state"], "provisioned");
    let ledger = fs::read(directory.join("resource-quota.json")).unwrap();
    assert!(!ledger.is_empty());
    let second = mantle(&state).args(["provision-quota", "--directory"]).arg(&directory).output().unwrap();
    assert_eq!(second.status.code(), Some(0), "{second:?}");
    assert_eq!(
        fs::read(directory.join("resource-quota.json")).unwrap(),
        ledger,
        "an existing ledger must be preserved"
    );
}

#[test]
fn provision_quota_refuses_a_missing_directory_without_creating_it() {
    let root = TempDir::new().unwrap();
    let missing = root.path().join("absent");
    let output = mantle(&root.path().join("state"))
        .args(["provision-quota", "--directory"])
        .arg(&missing)
        .output()
        .unwrap();
    assert_refused(&output, EXIT_INTERNAL);
    assert!(!missing.exists());
}

fn campaign(state: &Path, directory: &Path, seed: u64) -> std::process::Output {
    mantle(state)
        .args(["quota-fault-campaign", "--directory"])
        .arg(directory)
        .args(["--seed", &seed.to_string()])
        .output()
        .unwrap()
}

#[test]
fn quota_fault_campaign_is_deterministic_per_seed_and_labelled_a_local_fixture() {
    let root = TempDir::new().unwrap();
    let state = root.path().join("state");
    let a = campaign(&state, &root.path().join("a"), 7);
    let b = campaign(&state, &root.path().join("b"), 7);
    let other = campaign(&state, &root.path().join("c"), 8);
    for output in [&a, &b, &other] {
        assert_eq!(output.status.code(), Some(0), "{output:?}");
    }
    assert_eq!(a.stdout, b.stdout, "the same seed must reproduce the same evidence");
    assert_ne!(a.stdout, other.stdout, "the seed must select the fixture identity");
    let report: Value = serde_json::from_slice(&a.stdout).unwrap();
    assert_eq!(report["evidence_class"], "local-deterministic-fixture-only");
    assert!(report["ledger"]["entries"].as_object().is_some_and(|entries| !entries.is_empty()));
}

#[test]
fn quota_fault_campaign_refuses_to_reuse_a_directory_and_leaves_it_untouched() {
    let root = TempDir::new().unwrap();
    let state = root.path().join("state");
    let directory = root.path().join("existing");
    fs::create_dir(&directory).unwrap();
    fs::write(directory.join("operator-evidence"), b"keep").unwrap();
    assert_refused(&campaign(&state, &directory, 7), EXIT_INTERNAL);
    let names: Vec<_> = fs::read_dir(&directory).unwrap().map(|entry| entry.unwrap().file_name()).collect();
    assert_eq!(names, vec![std::ffi::OsString::from("operator-evidence")]);
    assert_eq!(fs::read(directory.join("operator-evidence")).unwrap(), b"keep");
}
