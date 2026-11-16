//! Operator-only adoption of two canonical outputs of an independently verified
//! completed v2 cold proof. No source producer, compiler or native row runs here.

use std::path::PathBuf;
use std::process::ExitCode;

use mantle::source_built_fixed_point_stagex_adoption::StagexAdoptionOptions;
use mantle::source_built_fixed_point_stagex_adoption::adopt_source_built_stagex;

fn run() -> Result<(), String> {
    let mut proof_root = None;
    let mut expected_plan = None;
    let mut expected_receipt = None;
    let mut store = None;
    let mut state_dir = None;
    let mut signing_key = None;
    let mut report = None;
    let mut backend = crunch_store::StoreBackend::Snix;
    let mut backend_seen = false;
    let mut json = false;
    let mut args = std::env::args_os().skip(1);
    while let Some(argument) = args.next() {
        let name = argument.to_str().ok_or("argument name must be UTF-8")?;
        if name == "--help" || name == "-h" {
            println!(
                "Usage: adopt-source-built-stagex --proof-root DIR --expected-plan-blake3 HEX --expected-receipt-bytes-blake3 HEX --store DIR --state-dir DIR --signing-key FILE --report FILE [--backend snix|casita] [--json]"
            );
            return Ok(());
        }
        if name == "--json" {
            if json {
                return Err("--json supplied twice".into());
            }
            json = true;
            continue;
        }
        let value = args.next().ok_or_else(|| format!("{name} requires a value"))?;
        match name {
            "--proof-root" if proof_root.is_none() => proof_root = Some(PathBuf::from(value)),
            "--expected-plan-blake3" if expected_plan.is_none() => {
                expected_plan = Some(value.into_string().map_err(|_| "plan digest must be UTF-8")?)
            }
            "--expected-receipt-bytes-blake3" if expected_receipt.is_none() => {
                expected_receipt = Some(value.into_string().map_err(|_| "receipt digest must be UTF-8")?)
            }
            "--store" if store.is_none() => store = Some(PathBuf::from(value)),
            "--state-dir" if state_dir.is_none() => state_dir = Some(PathBuf::from(value)),
            "--signing-key" if signing_key.is_none() => signing_key = Some(PathBuf::from(value)),
            "--report" if report.is_none() => report = Some(PathBuf::from(value)),
            "--backend" if !backend_seen => {
                backend = match value.to_str() {
                    Some("snix") => crunch_store::StoreBackend::Snix,
                    Some("casita") => crunch_store::StoreBackend::Casita,
                    _ => return Err("--backend must be snix or casita".into()),
                };
                backend_seen = true;
            }
            _ => return Err(format!("unknown or repeated argument: {name}")),
        }
    }
    let proof_root = proof_root.ok_or("--proof-root is required")?;
    let expected_plan = expected_plan.ok_or("--expected-plan-blake3 is required")?;
    let expected_receipt = expected_receipt.ok_or("--expected-receipt-bytes-blake3 is required")?;
    let store = store.ok_or("--store is required")?;
    let state_dir = state_dir.ok_or("--state-dir is required")?;
    let signing_key = signing_key.ok_or("--signing-key is required")?;
    let report = report.ok_or("--report is required")?;
    let result = adopt_source_built_stagex(&StagexAdoptionOptions {
        proof_root: &proof_root,
        expected_plan_blake3: &expected_plan,
        expected_receipt_bytes_blake3: &expected_receipt,
        output_dir: &store,
        state_dir: &state_dir,
        signing_key: &signing_key,
        report: &report,
        backend,
    })
    .map_err(|error| error.to_string())?;
    if json {
        println!("{}", serde_json::to_string(&result).map_err(|error| error.to_string())?);
    } else {
        println!(
            "Adopted signed StageX transition and provider from independently verified cold proof: {}",
            proof_root.display()
        );
        println!("Report: {}", report.display());
        println!(
            "No new provider construction, fixed point, compiler correctness, native-row acceptance or release authority is claimed."
        );
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{}", serde_json::json!({"error": error, "adopted": false}));
            ExitCode::FAILURE
        }
    }
}
