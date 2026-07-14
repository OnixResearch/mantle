use std::ffi::OsString;
use std::path::Path;
use std::process::ExitCode;

use crunch_hardware_simulation_core::HardwarePlanRequest;
use crunch_hardware_simulation_core::build_hardware_plan;

const MAX_REQUEST_BYTES: u64 = 1_048_576;

fn main() -> ExitCode {
    match run(std::env::args_os().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("hardware simulation plan: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(mut args: impl Iterator<Item = OsString>) -> Result<(), String> {
    let request_path = args
        .next()
        .ok_or_else(|| String::from("usage: hardware_simulation_plan <request.json> <plan.json>"))?;
    let plan_path = args
        .next()
        .ok_or_else(|| String::from("usage: hardware_simulation_plan <request.json> <plan.json>"))?;
    if args.next().is_some() {
        return Err(String::from("unexpected extra argument"));
    }
    let request = read_request(Path::new(&request_path))?;
    let plan = build_hardware_plan(request).map_err(|diagnostics| diagnostics.join(", "))?;
    std::fs::write(&plan_path, &plan.canonical_bytes)
        .map_err(|error| format!("writing {}: {error}", Path::new(&plan_path).display()))?;
    println!("plan_blake3={}", plan.plan_blake3);
    println!("units={}", plan.plan.units.len());
    println!("roots={}", plan.plan.roots.len());
    Ok(())
}

fn read_request(path: &Path) -> Result<HardwarePlanRequest, String> {
    let metadata = std::fs::metadata(path).map_err(|error| format!("reading {} metadata: {error}", path.display()))?;
    if metadata.len() == 0 || metadata.len() > MAX_REQUEST_BYTES {
        return Err(format!("request byte count must be within 1..={MAX_REQUEST_BYTES}"));
    }
    let bytes = std::fs::read(path).map_err(|error| format!("reading {}: {error}", path.display()))?;
    let request = serde_json::from_slice(&bytes).map_err(|error| format!("decoding {}: {error}", path.display()))?;
    debug_assert!(!bytes.is_empty());
    debug_assert!(metadata.len() <= MAX_REQUEST_BYTES);
    Ok(request)
}
