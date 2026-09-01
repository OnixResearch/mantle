#!/usr/bin/env -S cargo -Zscript
---cargo
[package]
edition = "2024"

[dependencies]
serde_json = "1"
---

use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::process::ExitCode;

const ARG_COUNT: usize = 3;

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
        return Err("usage: summarize-missing-rust-actions <plan> <reconciliation>".to_string());
    }
    let plan: serde_json::Value = serde_json::from_slice(&fs::read(&args[1]).map_err(|error| error.to_string())?)
        .map_err(|error| format!("parse plan: {error}"))?;
    let reconciliation: serde_json::Value =
        serde_json::from_slice(&fs::read(&args[2]).map_err(|error| error.to_string())?)
            .map_err(|error| format!("parse reconciliation: {error}"))?;
    let missing = reconciliation["missing_action_ids_blake3"]
        .as_array()
        .ok_or_else(|| "missing action list is absent".to_string())?
        .iter()
        .map(|value| value.as_str().ok_or_else(|| "missing action ID is not a string".to_string()))
        .collect::<Result<BTreeSet<_>, _>>()?;
    let actions = plan["actions"].as_array().ok_or_else(|| "plan actions are absent".to_string())?;
    let mut found = BTreeSet::new();
    for action in actions {
        let digest = action["action_id_blake3"]
            .as_str()
            .ok_or_else(|| "action digest is absent".to_string())?;
        if !missing.contains(digest) {
            continue;
        }
        found.insert(digest);
        let authorities = action["input_authority_ids"]
            .as_array()
            .ok_or_else(|| format!("action {digest} has no input authority"))?;
        let package = authorities
            .iter()
            .filter_map(serde_json::Value::as_str)
            .find(|value| value.starts_with("package:"))
            .unwrap_or("package:<missing>");
        let target = authorities
            .iter()
            .filter_map(serde_json::Value::as_str)
            .find(|value| value.starts_with("target:"))
            .unwrap_or("target:<missing>");
        let producers = action["producer_action_ids"].as_array().map_or(0, Vec::len);
        println!(
            "{}\t{}\t{}\t{}\t{}\tproducers={}",
            digest,
            action["unit_id_blake3"].as_str().unwrap_or("<missing>"),
            action["phase"].as_str().unwrap_or("<missing>"),
            package,
            target,
            producers
        );
    }
    if found != missing {
        return Err(format!("resolved {} of {} missing actions", found.len(), missing.len()));
    }
    if missing.is_empty() {
        return Err("reconciliation has no missing actions".to_string());
    }
    Ok(())
}
