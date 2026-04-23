use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::path::Path;

const EXAMPLE_INVENTORY: &str = "examples/system-config/inventory.ncl";
const EXAMPLE_PARTIAL_FAILURE_INVENTORY: &str = "examples/system-config/inventory-partial-failure.ncl";
const EXAMPLE_MODULES: &str = "examples/system-config/modules";
const EXAMPLE_BAD_MODULES: &str = "examples/system-config/bad-modules";

#[test]
fn system_eval_produces_machine_envelope_for_two_machines() {
    let output = run_system_eval(&[
        "system",
        "eval",
        EXAMPLE_INVENTORY,
        "--modules",
        EXAMPLE_MODULES,
    ]);

    let stdout_json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(stdout_json["machines"]["server1"].is_object());
    assert!(stdout_json["machines"]["server2"].is_object());
    assert_eq!(stdout_json["machines"]["server1"]["kind"], "derivations");
    assert_eq!(stdout_json["machines"]["server2"]["kind"], "derivations");
}

#[test]
fn system_eval_stop_after_fragments_returns_merged_configs() {
    let output = run_system_eval(&[
        "system",
        "eval",
        EXAMPLE_INVENTORY,
        "--modules",
        EXAMPLE_MODULES,
        "--stop-after",
        "fragments",
    ]);

    let stdout_json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(stdout_json["machines"]["server1"]["kind"], "fragments");
    assert!(stdout_json["machines"]["server1"]["merged_config"]["data"].is_object());
}

#[test]
fn system_eval_machine_filter_limits_output() {
    let output = run_system_eval(&[
        "system",
        "eval",
        EXAMPLE_INVENTORY,
        "--modules",
        EXAMPLE_MODULES,
        "--machine",
        "server1",
    ]);

    let stdout_json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(stdout_json["machines"].get("server1").is_some());
    assert!(stdout_json["machines"].get("server2").is_none());
}

#[test]
fn system_eval_unknown_machine_fails_before_evaluation() {
    let mut cmd = crunch_command();
    cmd.args([
        "system",
        "eval",
        EXAMPLE_INVENTORY,
        "--modules",
        EXAMPLE_MODULES,
        "--machine",
        "server3",
    ]);
    cmd.assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("unknown machine 'server3'"));
}

#[test]
fn system_eval_nickel_format_is_explicitly_unimplemented() {
    let mut cmd = crunch_command();
    cmd.args([
        "system",
        "eval",
        EXAMPLE_INVENTORY,
        "--modules",
        EXAMPLE_MODULES,
        "--format",
        "nickel",
    ]);
    cmd.assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("not implemented yet"));
}

#[test]
fn system_build_json_outputs_build_envelope() {
    let mut cmd = crunch_command();
    cmd.args([
        "--json",
        "system",
        "build",
        EXAMPLE_INVENTORY,
        "--modules",
        EXAMPLE_MODULES,
    ]);
    let output = cmd.assert().success().get_output().clone();
    let stdout_json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let server1_report = &stdout_json["machines"]["server1"]["reports"][0];
    let server2_report = &stdout_json["machines"]["server2"]["reports"][0];
    assert_eq!(stdout_json["machines"]["server1"]["kind"], "build");
    assert_eq!(stdout_json["machines"]["server2"]["kind"], "build");
    assert_eq!(server1_report["schema"], "crunch-build-report-v1");
    assert_eq!(server2_report["schema"], "crunch-build-report-v1");
    assert!(server1_report["counts"].is_object());
    assert!(server2_report["counts"].is_object());
    assert!(server1_report["outcomes"].is_array());
    assert!(server2_report["outcomes"].is_array());
    assert!(server1_report["failed"].is_array());
    assert!(server2_report["failed"].is_array());
}

#[test]
fn system_build_human_summary_lists_successful_machines() {
    let mut cmd = crunch_command();
    cmd.args([
        "system",
        "build",
        EXAMPLE_INVENTORY,
        "--modules",
        EXAMPLE_MODULES,
    ]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("server1"))
        .stdout(predicate::str::contains("server2"));
}

#[test]
fn system_eval_partial_failure_keeps_successful_machine_on_stdout() {
    let mut cmd = crunch_command();
    cmd.args([
        "--json",
        "system",
        "eval",
        EXAMPLE_PARTIAL_FAILURE_INVENTORY,
        "--modules",
        EXAMPLE_MODULES,
        "--assembler",
        "unknown-backend",
    ]);
    let output = cmd.assert().failure().get_output().clone();
    let stdout_json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let stderr_lines = std::str::from_utf8(&output.stderr).unwrap().lines().collect::<Vec<_>>();

    assert_eq!(stdout_json["machines"]["server1"]["kind"], "failed");
    assert_eq!(stdout_json["machines"]["server2"]["kind"], "failed");
    assert!(stdout_json["errors"].as_array().unwrap().iter().any(|error| {
        error["severity"] == "error" && error["layer"] == "assembler" && error["machine"] == "server1"
    }));
    assert!(stderr_lines.iter().any(|line| {
        let json: Value = serde_json::from_str(line).unwrap();
        json["severity"] == "error" && json["machine"] == "server1"
    }));
}

#[test]
fn system_eval_bad_module_contract_reports_module_name() {
    let mut cmd = crunch_command();
    cmd.args([
        "system",
        "eval",
        EXAMPLE_INVENTORY,
        "--modules",
        EXAMPLE_BAD_MODULES,
        "--machine",
        "server1",
    ]);
    cmd.assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("bad-contract"))
        .stderr(predicate::str::contains("not a function"));
}

#[test]
fn system_build_json_keeps_structured_diagnostics_off_stdout() {
    let mut cmd = crunch_command();
    cmd.args([
        "--json",
        "system",
        "build",
        EXAMPLE_INVENTORY,
        "--modules",
        EXAMPLE_MODULES,
        "--assembler",
        "unknown-backend",
    ]);
    let output = cmd.assert().failure().get_output().clone();
    let stdout_json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let stderr_lines = std::str::from_utf8(&output.stderr).unwrap().lines().collect::<Vec<_>>();

    assert!(stdout_json["machines"]["server1"].is_object());
    assert_eq!(stdout_json["machines"]["server1"]["kind"], "failed");
    assert_eq!(stdout_json["machines"]["server2"]["kind"], "failed");
    assert_eq!(stdout_json["errors"][0]["message"], "unknown assembler 'unknown-backend'");
    assert_eq!(stdout_json["errors"][0]["machine"], "server1");
    assert_eq!(stdout_json["errors"][0]["severity"], "error");
    assert_eq!(stdout_json["errors"][0]["layer"], "assembler");
    assert!(stderr_lines.iter().all(|line| serde_json::from_str::<Value>(line).is_ok()));
    assert!(stderr_lines.iter().any(|line| {
        let json: Value = serde_json::from_str(line).unwrap();
        json["severity"] == "warning" && json["layer"] == "eval"
    }));
    assert!(stderr_lines.iter().any(|line| {
        let json: Value = serde_json::from_str(line).unwrap();
        json["severity"] == "error" && json["layer"] == "assembler"
    }));
}

fn run_system_eval(args: &[&str]) -> std::process::Output {
    let mut cmd = crunch_command();
    cmd.args(args);
    cmd.assert().success().get_output().clone()
}

fn crunch_command() -> Command {
    let mut cmd = Command::cargo_bin("crunch").unwrap();
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    cmd.current_dir(repo_root);
    cmd.env(
        "PATH",
        format!(
            "{}/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:{}/.cargo/bin:{}",
            std::env::var("HOME").unwrap_or_else(|_| "/home/brittonr".to_string()),
            std::env::var("HOME").unwrap_or_else(|_| "/home/brittonr".to_string()),
            std::env::var("PATH").unwrap_or_default()
        ),
    );
    cmd.env("SNIX_BUILD_SANDBOX_SHELL", "/bin/sh");
    cmd
}
