//! Standalone, explicit-input driver; never inherits compiler configuration.
use std::io::Write;
use std::io::{self};
use std::path::Path;

use mantle_rust_differential::Options;
use mantle_rust_differential::Result;
use mantle_rust_differential::cohorts;
use mantle_rust_differential::fixtures;
use mantle_rust_differential::negative;
use mantle_rust_differential::process;
use mantle_rust_differential::runner;
use mantle_rust_differential::serial;
use serde::Serialize;

const USAGE: &str = "Usage: mantle-rust-differential baseline|serial|cohorts|negatives OPTIONS.json\n       mantle-rust-differential fixtures\n\nOPTIONS.json declares root, cargo, rustc, sysroot, linker, path, closures, host,\nwrapper (null unless explicitly enabled), and baseline (null for a new freeze).\nAll tool/closure paths must be resolved absolute paths; root must not exist.\nserial runs whole-unit rustc and accepts diagnostics=true (wrapper must be null).\ncohorts requires the explicitly built mantle-rust-wait-wrapper.\nExit: 0 = requested campaign qualified; 1 = nonpassing report; 2 = blocked/error.\nNo early-metadata mode is admitted; all reports are diagnostic evidence only.";
fn emit(value: &impl Serialize) -> Result<()> {
    let mut out = io::stdout().lock();
    serde_json::to_writer_pretty(&mut out, value).map_err(|e| e.to_string())?;
    writeln!(out).map_err(|e| e.to_string())
}
fn run() -> Result<i32> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() == 1 && matches!(args[0].to_str(), Some("--help" | "-h")) {
        println!("{USAGE}");
        return Ok(0);
    }
    if args.len() == 1 && args[0] == "fixtures" {
        let recipe = ["chain", "wide", "proc-macro", "generated"]
            .into_iter()
            .map(fixtures::generate)
            .collect::<Result<Vec<_>>>()?;
        emit(&recipe)?;
        return Ok(0);
    }
    if args.len() != 2 {
        return Err(USAGE.into());
    }
    let command = args[0].to_str().ok_or("command-must-be-UTF8")?;
    if !matches!(command, "baseline" | "serial" | "cohorts" | "negatives") {
        return Err(USAGE.into());
    }
    let bytes = process::read_bounded(Path::new(&args[1]), 64 * 1024)?;
    let options: Options = serde_json::from_slice(&bytes).map_err(|e| format!("invalid-options:{e}"))?;
    let root = options.root.clone();
    let (qualified, report) = match command {
        "baseline" => {
            let report = runner::execute(options)?;
            (report.complete, "report.json")
        }
        "serial" => {
            let report = serial::execute(options)?;
            (report.complete, "serial.json")
        }
        "cohorts" => {
            let report = cohorts::execute(options)?;
            (report.qualified, "cohorts.json")
        }
        "negatives" => {
            let report = negative::execute(options)?;
            (report.all_negatives_observed, "negatives.json")
        }
        _ => unreachable!(),
    };
    emit(
        &serde_json::json!({"campaign":command,"qualified":qualified,"report":root.join(report),"authority":"diagnostic-only"}),
    )?;
    Ok(if qualified { 0 } else { 1 })
}
fn main() {
    let code = match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("mantle-rust-differential: {error}");
            2
        }
    };
    std::process::exit(code);
}
