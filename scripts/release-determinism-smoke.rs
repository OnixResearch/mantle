#!/usr/bin/env -S cargo -Zscript
---
[package]
edition = "2024"

[dependencies]
blake3 = "1.8.2"
serde_json = "1"
---

//! Run the checked-in deterministic release proof smoke and write a concise
//! machine-readable receipt under `target/`.
//!
//! This is an operator rail around the real CLI regression. The regression
//! constructs release evidence, runs `mantle release reproduce` with two clean
//! deterministic proof stores, and verifies the generated proof with
//! `mantle release verify --require-deterministic-release`.

use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitCode;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use serde_json::json;

const WORKFLOW: &str = "mantle-release-determinism-smoke-v1";
const TEST_NAME: &str = "release_reproduce_generated_two_clean_store_proof_verifies_deterministic_release";
const DEFAULT_OUTPUT_DIR: &str = "target/release-determinism-smoke/latest";

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

    let repo_root = repo_root()?;
    let output_dir = absolutize(&repo_root, &args.output_dir);
    let log_path = output_dir.join("test.log");
    let receipt_path = output_dir.join("receipt.json");
    let command = vec![
        "cargo".to_string(),
        "test".to_string(),
        "--test".to_string(),
        "release_cli".to_string(),
        TEST_NAME.to_string(),
        "--".to_string(),
        "--nocapture".to_string(),
    ];

    if args.check {
        println!("workflow: {WORKFLOW}");
        println!("repo: {}", repo_root.display());
        println!("output-dir: {}", output_dir.display());
        println!("command: {}", command.join(" "));
        return Ok(());
    }

    fs::create_dir_all(&output_dir).map_err(|err| format!("create output dir {}: {err}", output_dir.display()))?;

    let started_unix_ms = unix_ms();
    let output = Command::new(&command[0])
        .args(&command[1..])
        .current_dir(&repo_root)
        .output()
        .map_err(|err| format!("launch {}: {err}", command.join(" ")))?;
    let finished_unix_ms = unix_ms();

    let mut log = Vec::new();
    log.extend_from_slice(b"$ ");
    log.extend_from_slice(command.join(" ").as_bytes());
    log.extend_from_slice(b"\n\n--- stdout ---\n");
    log.extend_from_slice(&output.stdout);
    log.extend_from_slice(b"\n--- stderr ---\n");
    log.extend_from_slice(&output.stderr);
    fs::write(&log_path, &log).map_err(|err| format!("write {}: {err}", log_path.display()))?;

    let log_digest_blake3 = blake3::hash(&log).to_hex().to_string();
    let git_head = command_text(&repo_root, "git", &["rev-parse", "HEAD"]).unwrap_or_else(|err| err);
    let git_status = command_text(&repo_root, "git", &["status", "--short", "--branch"]).unwrap_or_else(|err| err);
    let passed = output.status.success();
    let status_code = output.status.code();
    let receipt = json!({
        "workflow": WORKFLOW,
        "version": 1,
        "verdict": if passed { "passed" } else { "failed" },
        "bounded_claim": "The checked-in CLI regression generated a mantle-deterministic-proof-receipt-v2 receipt from two clean proof stores, bound exact source/recipe/tool/provider/policy/run-root identities, excluded published target authority, verified matching BLAKE3 artifact digest sets under supported sandbox evidence, and release verification accepted that generated proof. This does not claim compiler/verifier soundness.",
        "repo_root": repo_root,
        "git_head": git_head.trim(),
        "git_status": git_status.trim_end(),
        "started_unix_ms": started_unix_ms,
        "finished_unix_ms": finished_unix_ms,
        "duration_ms": finished_unix_ms.saturating_sub(started_unix_ms),
        "command": command,
        "test_name": TEST_NAME,
        "exit_code": status_code,
        "log_path": log_path,
        "log_blake3": log_digest_blake3,
    });
    let receipt_bytes = serde_json::to_vec_pretty(&receipt).map_err(|err| format!("serialize receipt: {err}"))?;
    fs::write(&receipt_path, [&receipt_bytes[..], b"\n"].concat())
        .map_err(|err| format!("write {}: {err}", receipt_path.display()))?;

    println!("receipt: {}", receipt_path.display());
    println!("log: {}", log_path.display());
    println!("verdict: {}", if passed { "passed" } else { "failed" });
    println!("log-blake3: {log_digest_blake3}");

    if passed {
        Ok(())
    } else {
        Err(format!("determinism smoke failed; see {}", log_path.display()))
    }
}

#[derive(Debug)]
struct Args {
    output_dir: PathBuf,
    check: bool,
    help: bool,
}

impl Args {
    fn parse<I>(mut args: I) -> Result<Self, String>
    where I: Iterator<Item = String> {
        let mut parsed = Args {
            output_dir: PathBuf::from(DEFAULT_OUTPUT_DIR),
            check: false,
            help: false,
        };
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--output-dir" => {
                    let value = args.next().ok_or("--output-dir requires a value")?;
                    parsed.output_dir = PathBuf::from(value);
                }
                "--check" => parsed.check = true,
                "-h" | "--help" => parsed.help = true,
                other => return Err(format!("unknown argument: {other}")),
            }
        }
        Ok(parsed)
    }
}

fn print_usage() {
    println!("Usage: cargo -Zscript scripts/release-determinism-smoke.rs [--check] [--output-dir DIR]");
    println!();
    println!("Runs the deterministic release proof CLI regression and writes:");
    println!("  DIR/receipt.json");
    println!("  DIR/test.log");
    println!("Default DIR: {DEFAULT_OUTPUT_DIR}");
}

fn repo_root() -> Result<PathBuf, String> {
    if let Ok(root) = env::var("CRUNCH_RELEASE_DETERMINISM_REPO_ROOT") {
        return Ok(PathBuf::from(root));
    }
    match command_text(Path::new("."), "git", &["rev-parse", "--show-toplevel"]) {
        Ok(root) => Ok(PathBuf::from(root.trim())),
        Err(_) => env::current_dir().map_err(|err| format!("resolve current directory: {err}")),
    }
}

fn command_text(cwd: &Path, program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|err| format!("launch {program}: {err}"))?;
    if !output.status.success() {
        return Err(format!("{program} {} failed: {}", args.join(" "), String::from_utf8_lossy(&output.stderr).trim()));
    }
    String::from_utf8(output.stdout).map_err(|err| format!("{program} output was not utf-8: {err}"))
}

fn absolutize(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

fn unix_ms() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis()
}
