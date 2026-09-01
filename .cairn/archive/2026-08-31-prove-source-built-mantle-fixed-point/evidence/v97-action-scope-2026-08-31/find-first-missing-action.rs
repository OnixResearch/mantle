#!/usr/bin/env -S cargo -Zscript
---cargo
[package]
edition = "2024"

[dependencies]
blake3 = "1"
serde_json = "1"
---

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::process::ExitCode;

const ARG_COUNT: usize = 4;
const UNIT_ID_CONTEXT: &str = "mantle-source-built-rust-unit-id-v1";

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
        return Err("usage: find-first-missing-action <plan> <reconciliation> <topology-order>".to_string());
    }
    let plan = read_json(&args[1])?;
    let reconciliation = read_json(&args[2])?;
    let order = read_json(&args[3])?;
    let missing = reconciliation["missing_action_ids_blake3"]
        .as_array()
        .ok_or_else(|| "missing action list is absent".to_string())?
        .iter()
        .map(|value| value.as_str().ok_or_else(|| "missing action ID is not a string".to_string()))
        .collect::<Result<BTreeSet<_>, _>>()?;
    let actions = plan["actions"].as_array().ok_or_else(|| "plan actions are absent".to_string())?;
    let mut compile_by_unit = BTreeMap::new();
    for action in actions {
        if action["phase"].as_str() != Some("compile-unit") {
            continue;
        }
        let unit = action["unit_id_blake3"].as_str().ok_or_else(|| "unit digest is absent".to_string())?;
        if compile_by_unit.insert(unit, action).is_some() {
            return Err(format!("duplicate compile action for unit {unit}"));
        }
    }
    let order = order.as_array().ok_or_else(|| "topology order is not an array".to_string())?;
    let mut missing_in_order = Vec::new();
    for (index, value) in order.iter().enumerate() {
        let unit_id = value.as_str().ok_or_else(|| "topology unit ID is not a string".to_string())?;
        let unit_digest = digest_text(UNIT_ID_CONTEXT, unit_id);
        let action = compile_by_unit
            .get(unit_digest.as_str())
            .ok_or_else(|| format!("topology unit has no compile action: {unit_id}"))?;
        let action_digest = action["action_id_blake3"]
            .as_str()
            .ok_or_else(|| "action digest is absent".to_string())?;
        if !missing.contains(action_digest) {
            continue;
        }
        let authorities = action["input_authority_ids"]
            .as_array()
            .ok_or_else(|| "input authority is absent".to_string())?;
        let package = authority_with_prefix(authorities, "package:");
        let target = authority_with_prefix(authorities, "target:");
        missing_in_order.push((index, unit_id, action_digest, package, target));
    }
    for (index, unit_id, action_digest, package, target) in &missing_in_order {
        println!("index={index}\tunit={unit_id}\taction={action_digest}\t{package}\t{target}");
    }
    if missing_in_order.len() != missing.len() {
        return Err(format!(
            "resolved {} of {} missing actions in topology order",
            missing_in_order.len(),
            missing.len()
        ));
    }
    if missing_in_order.is_empty() {
        return Err("topology order has no missing action".to_string());
    }
    Ok(())
}

fn read_json(path: &str) -> Result<serde_json::Value, String> {
    serde_json::from_slice(&fs::read(path).map_err(|error| error.to_string())?)
        .map_err(|error| format!("parse {path}: {error}"))
}

fn digest_text(context: &str, text: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(context.as_bytes());
    hasher.update(&[0]);
    hasher.update(text.as_bytes());
    hasher.finalize().to_hex().to_string()
}

fn authority_with_prefix<'a>(authorities: &'a [serde_json::Value], prefix: &str) -> &'a str {
    authorities
        .iter()
        .filter_map(serde_json::Value::as_str)
        .find(|authority| authority.starts_with(prefix))
        .unwrap_or("<missing-authority>")
}
