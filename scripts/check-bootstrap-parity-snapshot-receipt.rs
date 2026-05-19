#!/usr/bin/env -S cargo -Zscript
---
[package]
edition = "2024"

[dependencies]
blake3 = "1.8.2"
serde_json = "1"
tempfile = "3"
---

//! Validate a `mantle-bootstrap-parity-snapshot-v1` receipt and its report digest.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde_json::{json, Value};

const WORKFLOW: &str = "mantle-bootstrap-parity-snapshot-v1";
const REPORT_SCHEMA: &str = "crunch-bootstrap-parity-gap-report-v1";

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
        return self_test();
    }
    let receipt_path = args.receipt_path.ok_or("receipt path is required unless --self-test is used")?;
    let summary = validate_receipt_path(&receipt_path)?;
    println!("bootstrap parity snapshot receipt valid");
    println!("  report: {}", summary.report_path.display());
    println!("  report_blake3: {}", summary.report_blake3);
    for (axis, count) in summary.axis_blockers {
        println!("  {axis}: blockers={count}");
    }
    Ok(())
}

#[derive(Debug)]
struct Args {
    receipt_path: Option<PathBuf>,
    self_test: bool,
    help: bool,
}

impl Args {
    fn parse<I>(mut args: I) -> Result<Self, String>
    where
        I: Iterator<Item = String>,
    {
        let mut parsed = Args { receipt_path: None, self_test: false, help: false };
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--self-test" => parsed.self_test = true,
                "-h" | "--help" => parsed.help = true,
                other if other.starts_with('-') => return Err(format!("unknown argument: {other}")),
                path => {
                    if parsed.receipt_path.is_some() {
                        return Err("only one receipt path may be provided".to_string());
                    }
                    parsed.receipt_path = Some(PathBuf::from(path));
                }
            }
        }
        Ok(parsed)
    }
}

fn print_usage() {
    println!("Usage: cargo -Zscript scripts/check-bootstrap-parity-snapshot-receipt.rs RECEIPT_JSON");
    println!("       cargo -Zscript scripts/check-bootstrap-parity-snapshot-receipt.rs --self-test");
}

#[derive(Debug)]
struct ValidationSummary {
    report_path: PathBuf,
    report_blake3: String,
    axis_blockers: Vec<(String, usize)>,
}

fn validate_receipt_path(receipt_path: &Path) -> Result<ValidationSummary, String> {
    let receipt_bytes = fs::read(receipt_path).map_err(|err| format!("read {}: {err}", receipt_path.display()))?;
    let receipt: Value = serde_json::from_slice(&receipt_bytes)
        .map_err(|err| format!("parse {}: {err}", receipt_path.display()))?;
    let base = receipt_path.parent().unwrap_or_else(|| Path::new("."));
    validate_receipt(base, &receipt)
}

fn validate_receipt(base: &Path, receipt: &Value) -> Result<ValidationSummary, String> {
    require_eq(receipt, "workflow", WORKFLOW)?;
    require_eq(receipt, "version", 1)?;
    require_eq(receipt, "verdict", "passed")?;
    require_eq(receipt, "report_schema", REPORT_SCHEMA)?;
    let claim = receipt["bounded_claim"].as_str().ok_or("bounded_claim must be a string")?;
    if !claim.contains("does not claim") || !claim.contains("parity completion") {
        return Err("bounded_claim must explicitly avoid completion claims".to_string());
    }

    let report_path = path_from_json(base, &receipt["report_path"], "report_path")?;
    let report_bytes = fs::read(&report_path).map_err(|err| format!("read {}: {err}", report_path.display()))?;
    let report_blake3 = blake3::hash(&report_bytes).to_hex().to_string();
    if receipt["report_blake3"].as_str() != Some(report_blake3.as_str()) {
        return Err("report_blake3 does not match report bytes".to_string());
    }

    let report: Value = serde_json::from_slice(&report_bytes)
        .map_err(|err| format!("parse report {}: {err}", report_path.display()))?;
    if report["schema"] != REPORT_SCHEMA {
        return Err(format!("unexpected report schema: {}", report["schema"]));
    }

    let axes = receipt["axes"].as_array().ok_or("receipt axes must be an array")?;
    let report_axes = report["axes"].as_array().ok_or("report axes must be an array")?;
    let mut axis_blockers = Vec::new();
    for name in ["live-bootstrap", "guix", "stagex"] {
        let receipt_axis = axes.iter().find(|axis| axis["axis"] == name).ok_or_else(|| format!("receipt missing axis {name}"))?;
        let report_axis = report_axes.iter().find(|axis| axis["axis"] == name).ok_or_else(|| format!("report missing axis {name}"))?;
        if receipt_axis["complete"] != report_axis["complete"] {
            return Err(format!("axis {name} complete mismatch"));
        }
        let report_blocking = report_axis["blocking_rows"].as_array().ok_or("report blocking_rows must be an array")?;
        let receipt_blocking = receipt_axis["blocking_rows"].as_array().ok_or("receipt blocking_rows must be an array")?;
        if receipt_blocking != report_blocking {
            return Err(format!("axis {name} blocking rows mismatch"));
        }
        if receipt_axis["blocking_count"].as_u64() != Some(report_blocking.len() as u64) {
            return Err(format!("axis {name} blocking_count mismatch"));
        }
        axis_blockers.push((name.to_string(), report_blocking.len()));
    }

    let rows = report["rows"].as_array().ok_or("report rows must be an array")?;
    if receipt["row_count"].as_u64() != Some(rows.len() as u64) {
        return Err("row_count mismatch".to_string());
    }
    let mut expected_partials = rows
        .iter()
        .filter(|row| row["status"] == "partial")
        .filter_map(|row| row["id"].as_str().map(ToOwned::to_owned))
        .collect::<Vec<_>>();
    expected_partials.sort();
    let actual_partials = receipt["partial_rows"]
        .as_array()
        .ok_or("partial_rows must be an array")?
        .iter()
        .map(|value| value.as_str().ok_or("partial row id must be a string").map(ToOwned::to_owned))
        .collect::<Result<Vec<_>, _>>()?;
    if actual_partials != expected_partials {
        return Err("partial_rows mismatch".to_string());
    }

    Ok(ValidationSummary { report_path, report_blake3, axis_blockers })
}

fn require_eq<T>(value: &Value, field: &str, expected: T) -> Result<(), String>
where
    T: Into<Value> + Clone,
{
    let expected = expected.into();
    if value[field] == expected {
        Ok(())
    } else {
        Err(format!("{field}: expected {expected}, got {}", value[field]))
    }
}

fn path_from_json(base: &Path, value: &Value, field: &str) -> Result<PathBuf, String> {
    let path = value.as_str().ok_or_else(|| format!("{field} must be a string path"))?;
    let path = PathBuf::from(path);
    Ok(if path.is_absolute() { path } else { base.join(path) })
}

fn self_test() -> Result<(), String> {
    let dir = tempfile::tempdir().map_err(|err| format!("create tempdir: {err}"))?;
    let report_path = dir.path().join("report.json");
    let report = json!({
        "schema": REPORT_SCHEMA,
        "axes": [
            {"axis": "live-bootstrap", "complete": false, "blocking_rows": ["gcc.4.0"]},
            {"axis": "guix", "complete": false, "blocking_rows": ["gcc.4.0", "crunch.self-build"]},
            {"axis": "stagex", "complete": false, "blocking_rows": ["crunch.self-build"]}
        ],
        "rows": [
            {"id": "gcc.4.0", "status": "partial"},
            {"id": "crunch.self-build", "status": "partial"},
            {"id": "seed", "status": "complete"}
        ]
    });
    let report_bytes = serde_json::to_vec_pretty(&report).map_err(|err| format!("serialize fixture report: {err}"))?;
    fs::write(&report_path, [&report_bytes[..], b"\n"].concat()).map_err(|err| format!("write report fixture: {err}"))?;
    let digest = blake3::hash(&fs::read(&report_path).unwrap()).to_hex().to_string();
    let receipt = json!({
        "workflow": WORKFLOW,
        "version": 1,
        "verdict": "passed",
        "bounded_claim": "The saved report is a point-in-time bootstrap parity gap snapshot; incomplete axes and blocking rows remain blockers, and this receipt does not claim live-bootstrap, Guix, or StageX parity completion.",
        "report_path": report_path,
        "report_schema": REPORT_SCHEMA,
        "report_blake3": digest,
        "row_count": 3,
        "partial_rows": ["crunch.self-build", "gcc.4.0"],
        "axes": [
            {"axis": "live-bootstrap", "complete": false, "blocking_count": 1, "blocking_rows": ["gcc.4.0"]},
            {"axis": "guix", "complete": false, "blocking_count": 2, "blocking_rows": ["gcc.4.0", "crunch.self-build"]},
            {"axis": "stagex", "complete": false, "blocking_count": 1, "blocking_rows": ["crunch.self-build"]}
        ]
    });
    validate_receipt(dir.path(), &receipt)?;

    let mut bad_digest = receipt.clone();
    bad_digest["report_blake3"] = json!("0".repeat(64));
    assert_rejected("report_blake3", dir.path(), &bad_digest)?;

    let mut bad_claim = receipt.clone();
    bad_claim["bounded_claim"] = json!("parity complete");
    assert_rejected("bounded claim", dir.path(), &bad_claim)?;

    let mut bad_axis = receipt.clone();
    bad_axis["axes"][0]["blocking_count"] = json!(99);
    assert_rejected("blocking_count", dir.path(), &bad_axis)?;

    println!("bootstrap parity snapshot receipt checker self-test passed");
    Ok(())
}

fn assert_rejected(label: &str, base: &Path, receipt: &Value) -> Result<(), String> {
    match validate_receipt(base, receipt) {
        Ok(_) => Err(format!("{label}: expected rejection")),
        Err(_) => Ok(()),
    }
}
