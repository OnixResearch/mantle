//! Independent pinned-Valence consumption of retained, digest-bound gateway evidence.
//! This checks supplied linkage, never worker authority or lifecycle acceptance.
use std::io::Read as _;
use std::io::Write as _;

use valence_core::build_service as v;

const MAX_EVIDENCE_BYTES: u64 = 1_048_576;

fn run() -> Result<bool, Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let (Some(path), Some(expected), None) = (args.next(), args.next(), args.next()) else {
        return Err("usage: resource_evidence_check <retained-bundle.json> <independently-retained-blake3>".into());
    };
    let expected = expected.to_str().ok_or("digest must be UTF-8")?;
    if expected.len() != 64 || !expected.bytes().all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)) {
        return Err("expected digest must be canonical BLAKE3 hex".into());
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)?.take(MAX_EVIDENCE_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_EVIDENCE_BYTES || blake3::hash(&bytes).to_hex().as_str() != expected {
        return Err("retained evidence size or independently retained digest mismatch".into());
    }
    let bundle: v::BuildServiceBundle = serde_json::from_slice(&bytes)?;
    if serde_json::to_vec(&bundle)? != bytes {
        return Err(
            "not exact producer encoding; duplicate or unknown fields and reformatted bytes are rejected".into()
        );
    }
    let report = v::validate_build_service_bundle(&bundle);
    let mut stdout = std::io::stdout().lock();
    let producer_roles_valid = bundle.resource_observations.iter().all(|r| r.role == v::BuildServiceRole::RecordedOnly)
        && bundle.usage_records.iter().all(|r| r.role == v::BuildServiceRole::RecordedOnly)
        && bundle.transitions.iter().all(|r| r.role == v::BuildServiceRole::RecordedOnly)
        && bundle.build_results.iter().all(|r| r.role == v::BuildServiceRole::Signed);
    let valid = report.valid && producer_roles_valid;
    serde_json::to_writer(
        &mut stdout,
        &serde_json::json!({
            "artifact_blake3": expected,
            "report": report,
            "producer_roles_valid": producer_roles_valid,
            "scope": "pinned Valence validation of supplied canonical identity, typed linkage, roles, privacy and non-claims only",
            "lifecycle_acceptance": false,
            "worker_authentication": false,
        }),
    )?;
    stdout.write_all(b"\n")?;
    Ok(valid)
}

fn main() -> std::process::ExitCode {
    match run() {
        Ok(true) => std::process::ExitCode::SUCCESS,
        Ok(false) => std::process::ExitCode::from(1),
        Err(error) => {
            eprintln!("resource-evidence-check: {error}");
            std::process::ExitCode::from(2)
        }
    }
}
