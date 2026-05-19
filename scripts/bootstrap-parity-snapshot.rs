#!/usr/bin/env -S cargo -Zscript
---
[package]
edition = "2024"

[dependencies]
blake3 = "1.8.2"
serde_json = "1"
---

//! Write a bounded, machine-readable snapshot receipt for `mantle bootstrap parity-report`.
//!
//! This is an operator rail for current bootstrap parity state. It records the
//! exact parity report, report BLAKE3, blocking rows per axis, and git state.
//! It does not claim that any parity axis is complete.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

const WORKFLOW: &str = "mantle-bootstrap-parity-snapshot-v1";
const REPORT_SCHEMA: &str = "crunch-bootstrap-parity-gap-report-v1";
const DEFAULT_OUTPUT_DIR: &str = "target/bootstrap-parity-snapshot/latest";

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
    let report_path = output_dir.join("report.json");
    let stderr_path = output_dir.join("stderr.log");
    let receipt_path = output_dir.join("receipt.json");
    let command = vec![
        "cargo".to_string(),
        "run".to_string(),
        "--quiet".to_string(),
        "--bin".to_string(),
        "mantle".to_string(),
        "--".to_string(),
        "--json".to_string(),
        "bootstrap".to_string(),
        "parity-report".to_string(),
    ];

    if args.check {
        println!("workflow: {WORKFLOW}");
        println!("repo: {}", repo_root.display());
        println!("output-dir: {}", output_dir.display());
        println!("command: {}", command.join(" "));
        return Ok(());
    }

    fs::create_dir_all(&output_dir).map_err(|err| format!("create {}: {err}", output_dir.display()))?;

    let started_unix_ms = unix_ms();
    let output = Command::new(&command[0])
        .args(&command[1..])
        .current_dir(&repo_root)
        .env("CARGO_BUILD_RUSTC_WRAPPER", "")
        .env("CARGO_TARGET_DIR", repo_root.join("target"))
        .env("TMPDIR", env::var("TMPDIR").unwrap_or_else(|_| "/tmp".to_string()))
        .output()
        .map_err(|err| format!("launch {}: {err}", command.join(" ")))?;
    let finished_unix_ms = unix_ms();

    fs::write(&report_path, &output.stdout).map_err(|err| format!("write {}: {err}", report_path.display()))?;
    fs::write(&stderr_path, &output.stderr).map_err(|err| format!("write {}: {err}", stderr_path.display()))?;

    let report: Value = serde_json::from_slice(&output.stdout)
        .map_err(|err| format!("parse parity report JSON from {}: {err}", report_path.display()))?;
    validate_report(&report)?;

    let axes = summarize_axes(&report)?;
    let partial_rows = partial_rows(&report)?;
    let report_blake3 = blake3::hash(&output.stdout).to_hex().to_string();
    let stderr_blake3 = blake3::hash(&output.stderr).to_hex().to_string();
    let git_head = command_text(&repo_root, "git", &["rev-parse", "HEAD"]).unwrap_or_else(|err| err);
    let git_status = command_text(&repo_root, "git", &["status", "--short", "--branch"]).unwrap_or_else(|err| err);
    let passed = output.status.success();

    let receipt = json!({
        "workflow": WORKFLOW,
        "version": 1,
        "verdict": if passed { "passed" } else { "failed" },
        "bounded_claim": "The saved report is a point-in-time bootstrap parity gap snapshot; incomplete axes and blocking rows remain blockers, and this receipt does not claim live-bootstrap, Guix, or StageX parity completion.",
        "repo_root": repo_root,
        "git_head": git_head.trim(),
        "git_status": git_status.trim_end(),
        "started_unix_ms": started_unix_ms,
        "finished_unix_ms": finished_unix_ms,
        "duration_ms": finished_unix_ms.saturating_sub(started_unix_ms),
        "command": command,
        "exit_code": output.status.code(),
        "report_path": report_path,
        "report_schema": REPORT_SCHEMA,
        "report_blake3": report_blake3,
        "stderr_path": stderr_path,
        "stderr_len": output.stderr.len(),
        "stderr_blake3": stderr_blake3,
        "row_count": report["rows"].as_array().map(|rows| rows.len()).unwrap_or(0),
        "partial_rows": partial_rows,
        "axes": axes,
    });
    let receipt_bytes = serde_json::to_vec_pretty(&receipt).map_err(|err| format!("serialize receipt: {err}"))?;
    fs::write(&receipt_path, [&receipt_bytes[..], b"\n"].concat())
        .map_err(|err| format!("write {}: {err}", receipt_path.display()))?;

    println!("receipt: {}", receipt_path.display());
    println!("report: {}", report_path.display());
    println!("verdict: {}", if passed { "passed" } else { "failed" });
    println!("report-blake3: {report_blake3}");
    for axis in receipt["axes"].as_array().unwrap() {
        println!(
            "{}: complete={} blockers={}",
            axis["axis"].as_str().unwrap(),
            axis["complete"].as_bool().unwrap(),
            axis["blocking_count"].as_u64().unwrap()
        );
    }

    if passed {
        Ok(())
    } else {
        Err(format!("parity snapshot command failed; see {}", stderr_path.display()))
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
    where
        I: Iterator<Item = String>,
    {
        let mut parsed = Args { output_dir: PathBuf::from(DEFAULT_OUTPUT_DIR), check: false, help: false };
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
    println!("Usage: cargo -Zscript scripts/bootstrap-parity-snapshot.rs [--check] [--output-dir DIR]");
    println!("Default DIR: {DEFAULT_OUTPUT_DIR}");
}

fn validate_report(report: &Value) -> Result<(), String> {
    if report["schema"] != REPORT_SCHEMA {
        return Err(format!("unexpected report schema: {}", report["schema"]));
    }
    let axes = report["axes"].as_array().ok_or("report axes must be an array")?;
    for expected in ["live-bootstrap", "guix", "stagex"] {
        if !axes.iter().any(|axis| axis["axis"] == expected) {
            return Err(format!("missing parity axis {expected}"));
        }
    }
    report["rows"].as_array().ok_or("report rows must be an array")?;
    Ok(())
}

fn summarize_axes(report: &Value) -> Result<Vec<Value>, String> {
    let axes = report["axes"].as_array().ok_or("report axes must be an array")?;
    axes.iter()
        .map(|axis| {
            let blocking_rows = axis["blocking_rows"].as_array().ok_or("axis blocking_rows must be an array")?;
            Ok(json!({
                "axis": axis["axis"],
                "complete": axis["complete"],
                "blocking_count": blocking_rows.len(),
                "blocking_rows": blocking_rows,
            }))
        })
        .collect()
}

fn partial_rows(report: &Value) -> Result<Vec<String>, String> {
    let rows = report["rows"].as_array().ok_or("report rows must be an array")?;
    let mut partial = rows
        .iter()
        .filter(|row| row["status"] == "partial")
        .filter_map(|row| row["id"].as_str().map(ToOwned::to_owned))
        .collect::<Vec<_>>();
    partial.sort();
    Ok(partial)
}

fn repo_root() -> Result<PathBuf, String> {
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
    if output.status.success() {
        String::from_utf8(output.stdout).map_err(|err| format!("decode {program} stdout: {err}"))
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

fn absolutize(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() { path.to_path_buf() } else { root.join(path) }
}

fn unix_ms() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis()
}
