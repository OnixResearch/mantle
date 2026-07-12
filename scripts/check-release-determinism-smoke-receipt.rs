#!/usr/bin/env -S cargo -Zscript
---
[package]
edition = "2024"

[dependencies]
blake3 = "1.8.2"
serde_json = "1"
---

//! Validate the repo-owned release determinism smoke rail receipt.
//!
//! This is intentionally narrower than release proof validation. It checks the
//! operator rail receipt emitted by `scripts/release-determinism-smoke.rs` so
//! schema drift in that rail is caught before operators rely on stale evidence.

use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use serde_json::Value;
use serde_json::json;

const WORKFLOW: &str = "mantle-release-determinism-smoke-v1";
const VERSION: u64 = 1;
const TEST_NAME: &str = "release_reproduce_generated_two_clean_store_proof_verifies_deterministic_release";
const EXPECTED_COMMAND: &[&str] = &["cargo", "test", "--test", "release_cli", TEST_NAME, "--", "--nocapture"];
const REQUIRED_CLAIM_FRAGMENTS: &[&str] = &[
    "mantle-deterministic-proof-receipt-v2",
    "two clean proof stores",
    "published target authority",
    "BLAKE3 artifact digest sets",
    "supported sandbox evidence",
    "release verification accepted",
];

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let args = Args::parse(env::args().skip(1))?;
    if args.help {
        print_usage();
        return Ok(());
    }
    if args.self_test {
        return run_self_test();
    }
    let receipt = args.receipt.as_ref().ok_or("receipt path is required unless --self-test is used")?;
    validate_receipt_file(receipt, args.skip_log_check)?;
    println!("release determinism smoke receipt valid: {}", receipt.display());
    Ok(())
}

#[derive(Debug)]
struct Args {
    receipt: Option<PathBuf>,
    skip_log_check: bool,
    self_test: bool,
    help: bool,
}

impl Args {
    fn parse<I>(args: I) -> Result<Self, String>
    where I: Iterator<Item = String> {
        let mut parsed = Args {
            receipt: None,
            skip_log_check: false,
            self_test: false,
            help: false,
        };
        for arg in args {
            match arg.as_str() {
                "--skip-log-check" => parsed.skip_log_check = true,
                "--self-test" => parsed.self_test = true,
                "-h" | "--help" => parsed.help = true,
                other if other.starts_with('-') => return Err(format!("unknown argument: {other}")),
                path => {
                    if parsed.receipt.is_some() {
                        return Err(format!("unexpected extra argument: {path}"));
                    }
                    parsed.receipt = Some(PathBuf::from(path));
                }
            }
        }
        Ok(parsed)
    }
}

fn print_usage() {
    println!("Usage: cargo -Zscript scripts/check-release-determinism-smoke-receipt.rs [--skip-log-check] RECEIPT");
    println!("       cargo -Zscript scripts/check-release-determinism-smoke-receipt.rs --self-test");
}

fn validate_receipt_file(path: &Path, skip_log_check: bool) -> Result<(), String> {
    let text = fs::read_to_string(path).map_err(|err| format!("read {}: {err}", path.display()))?;
    let value: Value = serde_json::from_str(&text).map_err(|err| format!("parse {} as JSON: {err}", path.display()))?;
    validate_receipt_value(&value, path.parent().unwrap_or_else(|| Path::new(".")), skip_log_check)
}

fn validate_receipt_value(value: &Value, receipt_dir: &Path, skip_log_check: bool) -> Result<(), String> {
    require_str(value, "workflow", WORKFLOW)?;
    require_u64(value, "version", VERSION)?;
    require_str(value, "verdict", "passed")?;
    require_u64(value, "exit_code", 0)?;
    require_str(value, "test_name", TEST_NAME)?;
    require_non_empty_str(value, "repo_root")?;
    require_non_empty_str(value, "git_head")?;
    require_non_empty_str(value, "git_status")?;
    require_ordered_time(value)?;
    require_command(value)?;
    require_bounded_claim(value)?;
    require_log_digest(value, receipt_dir, skip_log_check)?;
    Ok(())
}

fn require_command(value: &Value) -> Result<(), String> {
    let command = value.get("command").and_then(Value::as_array).ok_or("missing command array")?;
    if command.len() != EXPECTED_COMMAND.len() {
        return Err(format!("command length mismatch: expected {}, got {}", EXPECTED_COMMAND.len(), command.len()));
    }
    for (idx, expected) in EXPECTED_COMMAND.iter().enumerate() {
        let actual = command[idx].as_str().ok_or_else(|| format!("command[{idx}] is not a string"))?;
        if actual != *expected {
            return Err(format!("command[{idx}] mismatch: expected {expected:?}, got {actual:?}"));
        }
    }
    Ok(())
}

fn require_bounded_claim(value: &Value) -> Result<(), String> {
    let claim = field_str(value, "bounded_claim")?;
    for fragment in REQUIRED_CLAIM_FRAGMENTS {
        if !claim.contains(fragment) {
            return Err(format!("bounded_claim missing required fragment {fragment:?}"));
        }
    }
    Ok(())
}

fn require_log_digest(value: &Value, receipt_dir: &Path, skip_log_check: bool) -> Result<(), String> {
    let digest = field_str(value, "log_blake3")?;
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("log_blake3 must be 64 hex characters".to_string());
    }
    let log_path = field_str(value, "log_path")?;
    if skip_log_check {
        return Ok(());
    }
    let path = absolutize(receipt_dir, Path::new(log_path));
    let bytes = fs::read(&path).map_err(|err| format!("read log_path {}: {err}", path.display()))?;
    let actual = blake3::hash(&bytes).to_hex().to_string();
    if actual != digest {
        return Err(format!("log_blake3 mismatch: expected {digest}, computed {actual}"));
    }
    Ok(())
}

fn require_ordered_time(value: &Value) -> Result<(), String> {
    let started = field_u64(value, "started_unix_ms")?;
    let finished = field_u64(value, "finished_unix_ms")?;
    let duration = field_u64(value, "duration_ms")?;
    if finished < started {
        return Err("finished_unix_ms is before started_unix_ms".to_string());
    }
    if finished.saturating_sub(started) != duration {
        return Err("duration_ms does not match finished_unix_ms - started_unix_ms".to_string());
    }
    Ok(())
}

fn require_str(value: &Value, field: &str, expected: &str) -> Result<(), String> {
    let actual = field_str(value, field)?;
    if actual == expected {
        Ok(())
    } else {
        Err(format!("{field} mismatch: expected {expected:?}, got {actual:?}"))
    }
}

fn require_u64(value: &Value, field: &str, expected: u64) -> Result<(), String> {
    let actual = field_u64(value, field)?;
    if actual == expected {
        Ok(())
    } else {
        Err(format!("{field} mismatch: expected {expected}, got {actual}"))
    }
}

fn require_non_empty_str(value: &Value, field: &str) -> Result<(), String> {
    let actual = field_str(value, field)?;
    if actual.trim().is_empty() {
        Err(format!("{field} must not be empty"))
    } else {
        Ok(())
    }
}

fn field_str<'a>(value: &'a Value, field: &str) -> Result<&'a str, String> {
    value.get(field).and_then(Value::as_str).ok_or_else(|| format!("missing string field {field}"))
}

fn field_u64(value: &Value, field: &str) -> Result<u64, String> {
    value.get(field).and_then(Value::as_u64).ok_or_else(|| format!("missing integer field {field}"))
}

fn absolutize(base: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    }
}

fn run_self_test() -> Result<(), String> {
    let dir = env::temp_dir().join(format!("mantle-smoke-receipt-check-{}-{}", std::process::id(), unix_ms()));
    fs::create_dir_all(&dir).map_err(|err| format!("create {}: {err}", dir.display()))?;
    let log_path = dir.join("test.log");
    fs::write(&log_path, b"determinism smoke log\n").map_err(|err| format!("write self-test log: {err}"))?;
    let good = valid_receipt(&log_path)?;
    validate_receipt_value(&good, &dir, false)?;
    assert_rejected("mismatched workflow", mutate(&good, "workflow", json!("mantle-other-v1")), &dir)?;
    assert_rejected("unsupported version", mutate(&good, "version", json!(2)), &dir)?;
    assert_rejected("failed verdict", mutate(&good, "verdict", json!("failed")), &dir)?;
    assert_rejected("missing command", without(&good, "command"), &dir)?;
    assert_rejected("mismatched log digest", mutate(&good, "log_blake3", json!("0".repeat(64))), &dir)?;
    assert_rejected("weakened claim", mutate(&good, "bounded_claim", json!("too weak")), &dir)?;
    fs::remove_dir_all(&dir).map_err(|err| format!("remove {}: {err}", dir.display()))?;
    println!("release determinism smoke receipt checker self-test passed");
    Ok(())
}

fn valid_receipt(log_path: &Path) -> Result<Value, String> {
    let bytes = fs::read(log_path).map_err(|err| format!("read self-test log: {err}"))?;
    let digest = blake3::hash(&bytes).to_hex().to_string();
    Ok(json!({
        "workflow": WORKFLOW,
        "version": VERSION,
        "verdict": "passed",
        "bounded_claim": "The checked-in CLI regression generated a mantle-deterministic-proof-receipt-v2 receipt from two clean proof stores, bound exact source/recipe/tool/provider/policy/run-root identities, excluded published target authority, verified matching BLAKE3 artifact digest sets under supported sandbox evidence, and release verification accepted that generated proof. This does not claim compiler/verifier soundness.",
        "repo_root": "/repo",
        "git_head": "abc123",
        "git_status": "## main",
        "started_unix_ms": 10,
        "finished_unix_ms": 25,
        "duration_ms": 15,
        "command": EXPECTED_COMMAND,
        "test_name": TEST_NAME,
        "exit_code": 0,
        "log_path": log_path,
        "log_blake3": digest,
    }))
}

fn mutate(value: &Value, field: &str, replacement: Value) -> Value {
    let mut copy = value.clone();
    copy.as_object_mut().expect("self-test receipt is object").insert(field.to_string(), replacement);
    copy
}

fn without(value: &Value, field: &str) -> Value {
    let mut copy = value.clone();
    copy.as_object_mut().expect("self-test receipt is object").remove(field);
    copy
}

fn assert_rejected(label: &str, value: Value, receipt_dir: &Path) -> Result<(), String> {
    if validate_receipt_value(&value, receipt_dir, false).is_ok() {
        Err(format!("self-test expected rejection for {label}"))
    } else {
        Ok(())
    }
}

fn unix_ms() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis()
}
