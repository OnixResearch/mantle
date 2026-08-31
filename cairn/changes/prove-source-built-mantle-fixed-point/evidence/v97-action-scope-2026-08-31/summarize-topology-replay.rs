#!/usr/bin/env -S cargo -Zscript
---cargo
[package]
edition = "2024"

[dependencies]
serde_json = "1"
---

use std::env;
use std::fs;
use std::process::ExitCode;

const ARG_COUNT: usize = 2;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != ARG_COUNT {
        return Err("usage: summarize-topology-replay <report>".to_string());
    }
    let value: serde_json::Value = serde_json::from_slice(&fs::read(&args[1]).map_err(|error| error.to_string())?)
        .map_err(|error| format!("parse report: {error}"))?;
    let topology = value
        .get("topology_execution")
        .ok_or_else(|| "topology_execution is absent".to_string())?;
    let status = topology["execution_status"]
        .as_str()
        .ok_or_else(|| "execution status is absent".to_string())?;
    let executions = topology["unit_executions"]
        .as_array()
        .ok_or_else(|| "unit executions are absent".to_string())?;
    let blocker = &topology["blocker"];
    println!("execution_status={status}");
    println!("unit_execution_count={}", executions.len());
    println!("blocker={}", serde_json::to_string(blocker).map_err(|error| error.to_string())?);
    Ok(())
}
