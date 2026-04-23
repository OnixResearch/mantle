use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::path::Path;

const EXAMPLE_INVENTORY: &str = "examples/system-config/inventory.ncl";
const EXAMPLE_MODULES: &str = "examples/system-config/modules";

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
    assert_eq!(stdout_json["machines"]["server1"]["kind"], "build");
    assert_eq!(stdout_json["machines"]["server2"]["kind"], "build");
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
