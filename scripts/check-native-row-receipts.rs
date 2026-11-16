//! Compile in the reviewed checkout, then invoke the compiled executable directly.
//! The included core/shell validators are the authoritative receipt convention.
// r[impl bootstrap_inventory.cold_stagex_native_row_adoption]

#[path = "../src/early_native_row_receipt.rs"]
mod early_native_row_receipt;
mod early_native_row_receipt_shell {
    include!("../src/early_native_row_receipt_shell.rs");

    pub(super) fn contracts() -> Vec<serde_json::Value> {
        vec![
            contract(&binutils_expectation(), BINUTILS_RECEIPT_PATH),
            contract(&gcc40_expectation(), GCC40_RECEIPT_PATH),
        ]
    }

    pub(super) fn contract(e: &RowExpectation, receipt_path: &str) -> serde_json::Value {
        serde_json::json!({
            "schema": e.schema, "row_id": e.row_id, "derivation": e.derivation,
            "receipt_path": receipt_path, "artifact_evidence_path": e.artifact_evidence_path,
            "artifact_evidence_schema": e.artifact_evidence_schema,
            "acceptance_evidence_path": e.acceptance_evidence_path,
            "acceptance_evidence_schema": e.acceptance_evidence_schema,
            "success_marker": e.success_marker, "sources": e.source_paths,
            "source_policies": e.source_policies.iter().map(|p| serde_json::json!({
                "path": p.path, "required": p.required, "forbidden": p.forbidden
            })).collect::<Vec<_>>(),
            "predecessors": e.predecessor_roles.iter().zip(e.predecessor_evidence_paths)
                .map(|(role, path)| serde_json::json!({"role": role, "path": path})).collect::<Vec<_>>(),
            "generated": e.generated_artifacts, "positive": e.positive_matrix,
            "rejection": e.rejection_matrix, "non_claims": e.required_non_claims
        })
    }
}
mod final_native_row_receipt_shell {
    include!("../src/final_native_row_receipt_shell.rs");

    pub(super) fn contracts() -> Vec<serde_json::Value> {
        vec![
            crate::early_native_row_receipt_shell::contract(&gcc47_expectation(), GCC47_RECEIPT_PATH),
            crate::early_native_row_receipt_shell::contract(&gcc10_expectation(), GCC10_RECEIPT_PATH),
            crate::early_native_row_receipt_shell::contract(&final_expectation(), FINAL_RECEIPT_PATH),
        ]
    }
}

use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use crunch_attestation::ArtifactAttestation;
use crunch_attestation::Canonicalize;

fn bound_view(root: &std::path::Path) -> Result<(), String> {
    let mut pending = vec![root.to_path_buf()];
    let mut files = 0usize;
    let mut directories = 0usize;
    let mut total = 0u64;
    while let Some(directory) = pending.pop() {
        directories += 1;
        if directories > 64 {
            return Err("verification view exceeds directory bound".into());
        }
        for entry in fs::read_dir(directory).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let metadata = fs::symlink_metadata(entry.path()).map_err(|e| e.to_string())?;
            if metadata.is_dir() {
                pending.push(entry.path());
            } else if metadata.is_file() {
                files += 1;
                total = total.checked_add(metadata.len()).ok_or("view byte count overflow")?;
            } else {
                return Err("verification view contains a symlink or special file".into());
            }
            if files > 256 || pending.len() > 64 || total > 64 * 1024 * 1024 {
                return Err("verification view exceeds file/directory/byte bounds".into());
            }
        }
    }
    Ok(())
}

fn run() -> Result<(), String> {
    let mut args = std::env::args_os().skip(1);
    let action = args.next().ok_or("expected --contract, --artifact FILE, or --view-root DIR")?;
    if action == "--contract" {
        if args.next().is_some() {
            return Err("unexpected argument after --contract".into());
        }
        let mut contracts = early_native_row_receipt_shell::contracts();
        contracts.extend(final_native_row_receipt_shell::contracts());
        println!("{}", serde_json::to_string(&contracts).map_err(|e| e.to_string())?);
    } else if action == "--public-key" {
        let path = PathBuf::from(args.next().ok_or("--public-key requires a key file")?);
        if args.next().is_some() {
            return Err("unexpected argument after --public-key".into());
        }
        let metadata = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if !metadata.is_file() || metadata.len() == 0 || metadata.len() > 4096 {
            return Err("signing key must be a bounded regular file".into());
        }
        let contents = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let keypair = crunch_build::signing::load_keypair(&contents)?;
        println!("{}", keypair.verifying_key);
    } else if action == "--artifact" {
        let path = PathBuf::from(args.next().ok_or("--artifact requires a file")?);
        if args.next().is_some() {
            return Err("unexpected argument after --artifact".into());
        }
        let metadata = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if !metadata.is_file() || metadata.len() == 0 || metadata.len() > 4 * 1024 * 1024 {
            return Err("artifact attestation must be a bounded regular file".into());
        }
        let attestation: ArtifactAttestation = serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?)
            .map_err(|e| format!("parse artifact attestation: {e}"))?;
        let digest = attestation.canonical_digest().map_err(|e| e.to_string())?.to_hex();
        println!("{}", serde_json::json!({"digest":digest, "attestation":attestation}));
    } else if action == "--view-root" {
        let root = PathBuf::from(args.next().ok_or("--view-root requires a directory")?);
        if args.next().is_some() {
            return Err("unexpected argument after --view-root".into());
        }
        if !root.is_dir() {
            return Err("verification view does not exist".into());
        }
        bound_view(&root)?;
        early_native_row_receipt_shell::validate_binutils_row(&root)?;
        early_native_row_receipt_shell::validate_gcc40_row(&root)?;
        final_native_row_receipt_shell::validate_gcc47_row(&root)?;
        final_native_row_receipt_shell::validate_gcc10_row(&root)?;
        final_native_row_receipt_shell::validate_final_musl_binutils_row(&root)?;
        println!(
            "{}",
            serde_json::json!({"validated":true,"rows":["binutils.tcc","gcc.4.0","gcc.4.7","gcc.10","full-musl-binutils"]})
        );
    } else {
        return Err(format!("unknown action: {}", action.to_string_lossy()));
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{}", serde_json::json!({"validated":false,"error":error}));
            ExitCode::FAILURE
        }
    }
}
