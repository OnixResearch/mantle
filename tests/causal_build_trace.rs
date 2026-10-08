//! Real local build regression: diagnostic causality is opt-in and never receipt authority.

use std::path::Path;
use std::path::PathBuf;
use std::process::Output;

use mantle_causal_trace_core::ActionKind;
use mantle_causal_trace_core::Cause;
use mantle_causal_trace_core::Trace;
use mantle_causal_trace_core::chain_to_root;
use mantle_causal_trace_core::validate;

fn has_bwrap() -> bool {
    if !Path::new("/nix/store").exists() {
        return false;
    }
    std::process::Command::new("bwrap")
        .args(["--ro-bind", "/", "/", "--", "/bin/true"])
        .output()
        .is_ok_and(|output| output.status.success())
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name)
}

fn run_build(root: &Path, fixture_name: &str, traced: bool) -> (Output, serde_json::Value) {
    let mut command = assert_cmd::Command::cargo_bin("mantle").unwrap();
    command
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .arg("--json")
        .arg("--state-dir")
        .arg(root.join("state"))
        .arg("--store")
        .arg(root.join("store"))
        .arg("build")
        .arg(fixture(fixture_name))
        .args(["--no-substitute", "-j", "2"])
        .env("CRUNCH_CONFIG_DIR", root.join("config"))
        .env("CRUNCH_LOG_DIR", root.join("external-build-logs"))
        .env("TMPDIR", root.join("tmp"));
    if traced {
        command.arg("--causal-trace");
    }
    let output = command.output().unwrap();
    let report = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("build JSON {error}; stderr={}", String::from_utf8_lossy(&output.stderr)));
    (output, report)
}

fn trace_file(state: &Path) -> PathBuf {
    let files = std::fs::read_dir(state.join("logs"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.file_name().unwrap().to_string_lossy().starts_with("build-trace-"))
        .collect::<Vec<_>>();
    assert_eq!(files.len(), 1, "expected exactly one opt-in trace artifact");
    files.into_iter().next().unwrap()
}

fn trace_for(state: &Path) -> (PathBuf, Trace) {
    let path = trace_file(state);
    let trace = serde_json::from_slice::<Trace>(&std::fs::read(&path).unwrap()).unwrap();
    validate(&trace).unwrap();
    (path, trace)
}

#[test]
fn real_failure_has_causal_leaf_chain_without_changing_report_or_receipt() {
    if !has_bwrap() {
        eprintln!("skipping: bwrap user namespaces unavailable");
        return;
    }
    let root = tempfile::tempdir().unwrap();
    for name in ["state", "store", "tmp", "config", "external-build-logs"] {
        std::fs::create_dir(root.path().join(name)).unwrap();
    }
    let (before, untraced) = run_build(root.path(), "causal-parallel.ncl", false);
    assert!(!before.status.success());
    assert_eq!(untraced["counts"]["succeeded_total"], 1);
    assert_eq!(untraced["counts"]["failed_total"], 1);
    let trace_dir = root.path().join("state/logs");
    assert!(
        !trace_dir.exists()
            || !trace_dir.read_dir().unwrap().any(|entry| entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with("build-trace-"))
    );
    let receipt_path =
        PathBuf::from(untraced["outcomes"][0]["outputs"][0]["artifact_attestation"]["path"].as_str().unwrap());
    let receipt_before = std::fs::read(&receipt_path).unwrap();

    let (after, traced) = run_build(root.path(), "causal-parallel.ncl", true);
    assert!(!after.status.success());
    assert_eq!(traced["counts"]["succeeded_total"], 1);
    assert_eq!(traced["counts"]["failed_total"], 1);
    assert_eq!(traced["counts"]["cached_total"], 1);
    assert_eq!(traced["outcomes"][0]["cached"], true);
    assert_eq!(std::fs::read(receipt_path).unwrap(), receipt_before, "tracing must not change attestation bytes");
    assert_eq!(
        traced.as_object().unwrap().keys().collect::<Vec<_>>(),
        untraced.as_object().unwrap().keys().collect::<Vec<_>>()
    );
    assert_eq!(traced["schema"], untraced["schema"]);

    let (path, trace) = trace_for(&root.path().join("state"));
    assert!(
        !root.path().join("external-build-logs").read_dir().unwrap().any(|entry| entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with("build-trace-"))
    );
    let failed_key = traced["failed"][0]["drv_key"].as_str().unwrap();
    let failed_goal = blake3::hash(failed_key.as_bytes()).to_hex().to_string();
    let failed = trace
        .records
        .iter()
        .find(|record| record.kind == ActionKind::Failed && record.goal_blake3.as_deref() == Some(failed_goal.as_str()))
        .unwrap();
    let chain = chain_to_root(&trace, failed.action_id).unwrap();
    assert!(chain.iter().any(|id| trace.records[*id as usize].kind == ActionKind::RootRequirement
        && trace.records[*id as usize].goal_blake3.as_deref() == Some(failed_goal.as_str())));
    assert!(chain.iter().any(|id| trace.records[*id as usize].kind == ActionKind::Dispatch
        && trace.records[*id as usize].goal_blake3.as_deref() != Some(failed_goal.as_str())));
    assert_eq!(trace.records[0].kind, ActionKind::ExternalTrigger);

    let cached_key = traced["outcomes"][0]["drv_key"].as_str().unwrap();
    let cached_goal = blake3::hash(cached_key.as_bytes()).to_hex().to_string();
    let hit = trace
        .records
        .iter()
        .find(|record| {
            record.kind == ActionKind::CacheHit && record.goal_blake3.as_deref() == Some(cached_goal.as_str())
        })
        .unwrap();
    assert_eq!(hit.cause, Some(Cause::CacheDecision));
    assert_eq!(trace.records[hit.caused_by.unwrap() as usize].kind, ActionKind::CacheCheck);
    assert!(
        !chain.contains(&hit.action_id),
        "unrelated parallel root must not be invented as a dependency cause"
    );

    let receipt_check = assert_cmd::Command::cargo_bin("mantle")
        .unwrap()
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args(["receipt", "bundle", "verify", "--from"])
        .arg(&path)
        .output()
        .unwrap();
    assert!(!receipt_check.status.success(), "receipt verifier must reject diagnostic trace");
    assert!(receipt_check.stdout.is_empty(), "rejected trace must not be reported as accepted receipt evidence");
}

#[test]
fn traced_store_preflight_preserves_failed_report_without_inventing_a_worker_trace() {
    let root = tempfile::tempdir().unwrap();
    for name in ["state", "store", "tmp", "config"] {
        std::fs::create_dir(root.path().join(name)).unwrap();
    }
    std::fs::create_dir(root.path().join("state/pathinfo.redb")).unwrap();
    let run = |traced| {
        let mut command = assert_cmd::Command::cargo_bin("mantle").unwrap();
        command
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .arg("--json")
            .arg("--state-dir")
            .arg(root.path().join("state"))
            .arg("--store")
            .arg(root.path().join("store"))
            .arg("build")
            .arg(fixture("causal-parallel.ncl"))
            .args(["--no-substitute", "--strict-hermetic"])
            .env("CRUNCH_CONFIG_DIR", root.path().join("config"))
            .env_remove("CRUNCH_LOG_DIR")
            .env("TMPDIR", root.path().join("tmp"));
        if traced {
            command.arg("--causal-trace");
        }
        command.output().unwrap()
    };
    let untraced = run(false);
    let traced = run(true);
    assert_eq!(traced.status.code(), untraced.status.code());
    let original_report: serde_json::Value = serde_json::from_slice(&untraced.stdout).unwrap();
    let traced_report: serde_json::Value = serde_json::from_slice(&traced.stdout).unwrap();
    assert_eq!(traced_report, original_report, "trace request must not suppress or alter the preflight report");
    assert!(traced_report["failed"][0].to_string().contains("in-memory PathInfo fallback"));
    assert!(String::from_utf8_lossy(&traced.stderr).contains("causal trace unavailable: worker not started"));
    let diagnostic_dir = root.path().join("state/logs");
    assert!(
        !diagnostic_dir.exists()
            || !diagnostic_dir
                .read_dir()
                .unwrap()
                .any(|entry| { entry.unwrap().file_name().to_string_lossy().starts_with("build-trace-") })
    );
}

#[test]
fn trace_flag_rejects_evaluation_stream() {
    let output = assert_cmd::Command::cargo_bin("mantle")
        .unwrap()
        .args([
            "build",
            "--causal-trace",
            "--evaluation-stream",
            "tests/fixtures/causal-parallel.ncl",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot be used with"));
}
