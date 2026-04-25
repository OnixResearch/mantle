use std::path::Path;
use std::path::PathBuf;

use crate::errors::RunError;
use crate::release_attestation::create_release_attestation;
use crate::release_attestation::create_witness_attestation;
use crate::release_attestation::default_verification_dir;
use crate::release_evidence::DEFAULT_PROOF_WORKFLOW_COMMAND;
use crate::release_evidence::DEFAULT_PROOF_WORKFLOW_VERSION;
use crate::release_evidence::ReleaseBundleCreateRequest;
use crate::release_evidence::create_release_evidence_bundle;
use crate::release_evidence::verify_release_evidence_bundle;
use crate::release_reproducibility::DEFAULT_REPRODUCIBILITY_WORKFLOW_VERSION;
use crate::release_reproducibility::ReleaseReproduceRequest;
use crate::release_reproducibility::ReproducibilityStatus;
use crate::release_reproducibility::VerifiedReproducibilityReport;
use crate::release_reproducibility::load_bundle_reproducibility_report;
use crate::release_reproducibility::reproduce_release_artifacts;
use crate::release_source::write_tracked_source_archive;
use crate::witness_handoff::create_witness_request_directory;
use crate::witness_handoff::default_witness_request_dir;
use crate::witness_rebuild::WITNESS_SCRATCH_ENV;
use crate::witness_rebuild::WitnessRebuildSuccess;
use crate::witness_rebuild::audit_meta_path;
use crate::witness_rebuild::build_failure_audit_meta;
use crate::witness_rebuild::build_success_audit_meta;
use crate::witness_rebuild::default_witness_scratch_dir;
use crate::witness_rebuild::plan_witness_rebuild;
use crate::witness_rebuild::prepare_witness_rebuild_scratch;
use crate::witness_rebuild::run_witness_rebuild_workflow;
use crate::witness_rebuild::validate_successful_rebuild;
use crate::witness_rebuild::write_audit_meta;

pub(crate) fn cmd_release(
    action: crate::ReleaseAction,
    current_dir: &Path,
    state_dir: &Path,
    json: bool,
) -> Result<(), RunError> {
    match action {
        crate::ReleaseAction::Create {
            release_id,
            bundle_dir,
            binary,
            proof_bundle,
            workflow_command,
            workflow_version,
        } => cmd_release_create(
            current_dir,
            json,
            release_id,
            bundle_dir,
            binary,
            proof_bundle,
            workflow_command,
            workflow_version,
        ),
        crate::ReleaseAction::Verify { bundle_dir } => cmd_release_verify(json, bundle_dir),
        crate::ReleaseAction::Reproduce {
            bundle_dir,
            rebuild_output_dir,
            rebuild_command,
            rebuild_args,
            workflow_version,
            report_path,
        } => cmd_release_reproduce(
            current_dir,
            json,
            bundle_dir,
            rebuild_output_dir,
            rebuild_command,
            rebuild_args,
            workflow_version,
            report_path,
        ),
        crate::ReleaseAction::Attest {
            bundle_dir,
            verification_dir,
            signing_key,
        } => cmd_release_attest(current_dir, state_dir, json, bundle_dir, verification_dir, signing_key),
        crate::ReleaseAction::WitnessExport {
            bundle_dir,
            verification_dir,
            request_dir,
        } => cmd_release_witness_export(current_dir, json, bundle_dir, verification_dir, request_dir),
        crate::ReleaseAction::WitnessRebuild {
            request_dir,
            scratch_dir,
            check,
            identity,
            system,
            toolchain,
            host_class,
            signing_key,
        } => cmd_release_witness_rebuild(
            current_dir,
            state_dir,
            json,
            request_dir,
            scratch_dir,
            check,
            identity,
            system,
            toolchain,
            host_class,
            signing_key,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn cmd_release_create(
    current_dir: &Path,
    json: bool,
    release_id: String,
    bundle_dir: Option<PathBuf>,
    binary: Vec<PathBuf>,
    proof_bundle: PathBuf,
    workflow_command: String,
    workflow_version: String,
) -> Result<(), RunError> {
    let resolved_bundle_dir = resolve_bundle_dir(current_dir, &release_id, bundle_dir);
    let normalized_workflow_command = normalize_workflow_value(&workflow_command, DEFAULT_PROOF_WORKFLOW_COMMAND);
    let normalized_workflow_version = normalize_workflow_value(&workflow_version, DEFAULT_PROOF_WORKFLOW_VERSION);
    let source_archive_file = tempfile::NamedTempFile::new()
        .map_err(|err| RunError::Internal(format!("creating temp source archive file: {err}")))?;
    write_tracked_source_archive(current_dir, source_archive_file.path())?;
    let request = ReleaseBundleCreateRequest {
        release_id,
        bundle_dir: resolved_bundle_dir.clone(),
        source_archive_path: source_archive_file.path().to_path_buf(),
        binary_paths: resolve_input_paths(current_dir, binary),
        proof_bundle_dir: resolve_input_path(current_dir, proof_bundle),
        workflow_command: normalized_workflow_command,
        workflow_version: normalized_workflow_version,
    };
    let manifest = create_release_evidence_bundle(&request)?;
    if json {
        let rendered = serde_json::to_string(&manifest)
            .map_err(|err| RunError::Internal(format!("serializing release evidence manifest: {err}")))?;
        println!("{rendered}");
    } else {
        println!("release evidence bundle: {}", request.bundle_dir.display());
        println!("release id: {}", manifest.release_id);
        println!("source archive: {}", manifest.source_archive.relative_path);
        println!("proof bundle: {}", manifest.proof_bundle.relative_path);
        println!("binaries: {}", manifest.binaries.len());
        println!("manifest: manifest.json");
    }
    Ok(())
}

fn cmd_release_verify(json: bool, bundle_dir: PathBuf) -> Result<(), RunError> {
    let manifest = verify_release_evidence_bundle(&bundle_dir)?;
    let reproducibility = load_bundle_reproducibility_report(&bundle_dir, &manifest)?;
    if json {
        let rendered = serde_json::to_string(&manifest)
            .map_err(|err| RunError::Internal(format!("serializing verified release evidence manifest: {err}")))?;
        println!("{rendered}");
    } else {
        println!("release evidence verified: {}", bundle_dir.display());
        println!("release id: {}", manifest.release_id);
        println!("binaries: {}", manifest.binaries.len());
        println!("source digest: {}", manifest.source_archive.digest_blake3);
        println!("stage2 digest: {}", manifest.proof_linkage.stage2_binary_digest_blake3);
        println!("proof mode: {}", manifest.proof_linkage.proof_mode);
        print_reproducibility_summary(reproducibility.as_ref());
    }
    Ok(())
}

fn print_reproducibility_summary(reproducibility: Option<&VerifiedReproducibilityReport>) {
    match reproducibility {
        Some(report) => {
            println!("reproducibility: {}", report.status.as_str());
            println!("reproducibility report: {}", report.path.display());
            println!("reproducibility report digest: {}", report.digest_blake3);
        }
        None => {
            println!("reproducibility: {}", ReproducibilityStatus::Absent.as_str());
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn cmd_release_reproduce(
    current_dir: &Path,
    json: bool,
    bundle_dir: PathBuf,
    rebuild_output_dir: PathBuf,
    rebuild_command: PathBuf,
    rebuild_args: Vec<std::ffi::OsString>,
    workflow_version: String,
    report_path: Option<PathBuf>,
) -> Result<(), RunError> {
    let normalized_workflow_version =
        normalize_workflow_value(&workflow_version, DEFAULT_REPRODUCIBILITY_WORKFLOW_VERSION);
    let request = ReleaseReproduceRequest {
        bundle_dir: resolve_input_path(current_dir, bundle_dir),
        rebuild_output_dir: resolve_input_path(current_dir, rebuild_output_dir),
        rebuild_command: resolve_input_path(current_dir, rebuild_command),
        rebuild_args,
        workflow_version: normalized_workflow_version,
        report_path: report_path.map(|path| resolve_input_path(current_dir, path)),
    };
    let summary = reproduce_release_artifacts(&request)?;
    if json {
        let rendered = serde_json::json!({
            "release_id": summary.release_id,
            "report_path": summary.report_path.display().to_string(),
            "report_digest_blake3": summary.report_digest_blake3,
            "matched_count": summary.matched_count,
            "mismatched_count": summary.mismatched_count,
            "missing_count": summary.missing_count,
        });
        println!(
            "{}",
            serde_json::to_string(&rendered)
                .map_err(|err| RunError::Internal(format!("serializing reproducibility output: {err}")))?
        );
        return Ok(());
    }

    println!("release reproducibility report: {}", summary.report_path.display());
    println!("release id: {}", summary.release_id);
    println!("report digest: {}", summary.report_digest_blake3);
    println!("matched artifacts: {}", summary.matched_count);
    println!("mismatched artifacts: {}", summary.mismatched_count);
    println!("missing artifacts: {}", summary.missing_count);
    Ok(())
}

fn cmd_release_attest(
    current_dir: &Path,
    state_dir: &Path,
    json: bool,
    bundle_dir: PathBuf,
    verification_dir: Option<PathBuf>,
    signing_key: Option<PathBuf>,
) -> Result<(), RunError> {
    let resolved_bundle_dir = resolve_input_path(current_dir, bundle_dir);
    let verified_manifest = verify_release_evidence_bundle(&resolved_bundle_dir)?;
    let resolved_verification_dir = match verification_dir {
        Some(path) => resolve_input_path(current_dir, path),
        None => default_verification_dir(current_dir, &verified_manifest.release_id),
    };
    let created =
        create_release_attestation(&verified_manifest, &resolved_verification_dir, signing_key.as_deref(), state_dir)?;
    if json {
        let rendered = serde_json::json!({
            "release_id": created.attestation.release_id,
            "digest": created.digest_hex,
            "signer": created.signer_key_name,
            "attestation_path": created.attestation_path.display().to_string(),
            "signature_path": created.signature_path.display().to_string(),
        });
        println!(
            "{}",
            serde_json::to_string(&rendered)
                .map_err(|err| RunError::Internal(format!("serializing release attestation output: {err}")))?
        );
        return Ok(());
    }

    println!("release attestation: {}", created.attestation_path.display());
    println!("signature: {}", created.signature_path.display());
    println!("release id: {}", created.attestation.release_id);
    println!("digest: {}", created.digest_hex);
    println!("signer: {}", created.signer_key_name);
    Ok(())
}

fn cmd_release_witness_export(
    current_dir: &Path,
    json: bool,
    bundle_dir: PathBuf,
    verification_dir: Option<PathBuf>,
    request_dir: Option<PathBuf>,
) -> Result<(), RunError> {
    let resolved_bundle_dir = resolve_input_path(current_dir, bundle_dir);
    let verified_manifest = verify_release_evidence_bundle(&resolved_bundle_dir)?;
    let resolved_verification_dir = match verification_dir {
        Some(path) => resolve_input_path(current_dir, path),
        None => default_verification_dir(current_dir, &verified_manifest.release_id),
    };
    let resolved_request_dir = match request_dir {
        Some(path) => resolve_input_path(current_dir, path),
        None => default_witness_request_dir(current_dir, &verified_manifest.release_id),
    };
    let created = create_witness_request_directory(
        &verified_manifest,
        &resolved_bundle_dir,
        &resolved_verification_dir,
        &resolved_request_dir,
    )?;
    if json {
        let rendered = serde_json::json!({
            "kind": "crunch-witness-request",
            "release_id": created.release_id,
            "layout_version": created.layout_version,
            "request_dir": created.request_dir.display().to_string(),
            "request_path": created.request_path.display().to_string(),
            "release_bundle_path": created.release_bundle_path.display().to_string(),
            "verification_seed_path": created.verification_seed_path.display().to_string(),
        });
        println!(
            "{}",
            serde_json::to_string(&rendered)
                .map_err(|err| RunError::Internal(format!("serializing witness request output: {err}")))?
        );
        return Ok(());
    }

    println!("witness request: {}", created.request_dir.display());
    println!("request metadata: {}", created.request_path.display());
    println!("release id: {}", created.release_id);
    println!("layout version: {}", created.layout_version);
    println!("release bundle copy: {}", created.release_bundle_path.display());
    println!("verification seed: {}", created.verification_seed_path.display());
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn cmd_release_witness_rebuild(
    current_dir: &Path,
    state_dir: &Path,
    json: bool,
    request_dir: PathBuf,
    scratch_dir: Option<PathBuf>,
    check: bool,
    identity: Option<String>,
    system: Option<String>,
    toolchain: Option<String>,
    host_class: Option<String>,
    signing_key: Option<PathBuf>,
) -> Result<(), RunError> {
    let resolved_request_dir = resolve_input_path(current_dir, request_dir);
    let resolved_scratch_dir = resolve_witness_scratch_dir(current_dir, &resolved_request_dir, scratch_dir)?;
    let plan = plan_witness_rebuild(&resolved_request_dir, &resolved_scratch_dir)?;
    if check {
        return print_witness_rebuild_check(&plan, json);
    }

    let metadata = required_witness_rebuild_metadata(system, toolchain, host_class)?;
    prepare_witness_rebuild_scratch(&plan)?;
    let execution = run_witness_rebuild_workflow(&plan)?;
    if let Err(err) = validate_successful_rebuild(&plan, &execution) {
        let failure_meta = build_failure_audit_meta(
            &plan,
            &execution.workflow_driver_path,
            &execution.rebuilt_output_paths,
            execution.started_unix_ms,
            execution.finished_unix_ms,
            err.message(),
        )?;
        write_audit_meta(&audit_meta_path(&plan), &failure_meta)?;
        return Err(err);
    }
    let success = WitnessRebuildSuccess {
        rebuilt_output_paths: execution.rebuilt_output_paths,
        workflow_driver_path: execution.workflow_driver_path,
        started_unix_ms: execution.started_unix_ms,
        finished_unix_ms: execution.finished_unix_ms,
        output: execution.output,
    };
    let created = create_witness_attestation(
        &plan.scratch_layout.verification_dir,
        &success.rebuilt_output_paths,
        identity.as_deref(),
        &metadata.system,
        &metadata.toolchain,
        &metadata.host_class,
        signing_key.as_deref(),
        state_dir,
    )?;
    let audit_meta = build_success_audit_meta(&plan, &success, &created.attestation_path, &created.signature_path)?;
    write_audit_meta(&audit_meta_path(&plan), &audit_meta)?;
    print_witness_rebuild_success(&plan, &success, &created.attestation_path, &created.signature_path, json)
}

fn resolve_bundle_dir(current_dir: &Path, release_id: &str, bundle_dir: Option<PathBuf>) -> PathBuf {
    match bundle_dir {
        Some(path) => resolve_input_path(current_dir, path),
        None => current_dir.join("target").join("release-evidence").join(release_id),
    }
}

fn resolve_input_paths(current_dir: &Path, paths: Vec<PathBuf>) -> Vec<PathBuf> {
    paths.into_iter().map(|path| resolve_input_path(current_dir, path)).collect()
}

fn resolve_input_path(current_dir: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        current_dir.join(path)
    }
}

fn resolve_witness_scratch_dir(
    current_dir: &Path,
    request_dir: &Path,
    scratch_dir: Option<PathBuf>,
) -> Result<PathBuf, RunError> {
    if let Some(path) = scratch_dir {
        return Ok(resolve_input_path(current_dir, path));
    }
    if let Some(path_text) = std::env::var_os(WITNESS_SCRATCH_ENV) {
        return Ok(resolve_input_path(current_dir, PathBuf::from(path_text)));
    }
    default_witness_scratch_dir(request_dir)
}

fn normalize_workflow_value(value: &str, default_value: &str) -> String {
    if value.trim().is_empty() {
        default_value.to_string()
    } else {
        value.to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WitnessRebuildMetadata {
    system: String,
    toolchain: String,
    host_class: String,
}

fn required_witness_rebuild_metadata(
    system: Option<String>,
    toolchain: Option<String>,
    host_class: Option<String>,
) -> Result<WitnessRebuildMetadata, RunError> {
    let system = require_metadata_value(system, "--system")?;
    let toolchain = require_metadata_value(toolchain, "--toolchain")?;
    let host_class = require_metadata_value(host_class, "--host-class")?;
    Ok(WitnessRebuildMetadata {
        system,
        toolchain,
        host_class,
    })
}

fn require_metadata_value(value: Option<String>, flag_name: &str) -> Result<String, RunError> {
    let Some(value) = value else {
        return Err(RunError::Internal(format!("{} is required unless --check is used", flag_name)));
    };
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(RunError::Internal(format!("{} must not be empty", flag_name)));
    }
    Ok(trimmed.to_string())
}

fn print_witness_rebuild_check(plan: &crate::witness_rebuild::WitnessRebuildPlan, json: bool) -> Result<(), RunError> {
    let audit_path = audit_meta_path(plan);
    if json {
        let rendered = serde_json::json!({
            "kind": "crunch-witness-rebuild",
            "check_only": true,
            "release_id": plan.release_id,
            "request_dir": plan.request_dir.display().to_string(),
            "scratch_root": plan.scratch_layout.scratch_root.display().to_string(),
            "verification_dir": plan.scratch_layout.verification_dir.display().to_string(),
            "proof_bundle_dir": plan.scratch_layout.proof_bundle_dir.display().to_string(),
            "audit_meta_path": audit_path.display().to_string(),
            "workflow_command": plan.workflow_command,
            "workflow_version": plan.workflow_version,
        });
        println!(
            "{}",
            serde_json::to_string(&rendered)
                .map_err(|err| RunError::Internal(format!("serializing witness rebuild check output: {err}")))?
        );
        return Ok(());
    }

    println!("witness rebuild preflight OK: {}", plan.request_dir.display());
    println!("release id: {}", plan.release_id);
    println!("scratch root: {}", plan.scratch_layout.scratch_root.display());
    println!("verification output: {}", plan.scratch_layout.verification_dir.display());
    println!("proof bundle output: {}", plan.scratch_layout.proof_bundle_dir.display());
    println!("audit metadata: {}", audit_path.display());
    println!("check only: no rebuild executed, no witness sidecars written");
    Ok(())
}

fn print_witness_rebuild_success(
    plan: &crate::witness_rebuild::WitnessRebuildPlan,
    success: &WitnessRebuildSuccess,
    attestation_path: &Path,
    signature_path: &Path,
    json: bool,
) -> Result<(), RunError> {
    let audit_path = audit_meta_path(plan);
    if json {
        let rebuilt_outputs =
            success.rebuilt_output_paths.iter().map(|path| path.display().to_string()).collect::<Vec<_>>();
        let rendered = serde_json::json!({
            "kind": "crunch-witness-rebuild",
            "check_only": false,
            "release_id": plan.release_id,
            "request_dir": plan.request_dir.display().to_string(),
            "scratch_root": plan.scratch_layout.scratch_root.display().to_string(),
            "verification_dir": plan.scratch_layout.verification_dir.display().to_string(),
            "proof_bundle_dir": plan.scratch_layout.proof_bundle_dir.display().to_string(),
            "audit_meta_path": audit_path.display().to_string(),
            "workflow_driver_path": success.workflow_driver_path.display().to_string(),
            "attestation_path": attestation_path.display().to_string(),
            "signature_path": signature_path.display().to_string(),
            "rebuilt_outputs": rebuilt_outputs,
            "started_unix_ms": success.started_unix_ms,
            "finished_unix_ms": success.finished_unix_ms,
        });
        println!(
            "{}",
            serde_json::to_string(&rendered)
                .map_err(|err| RunError::Internal(format!("serializing witness rebuild output: {err}")))?
        );
        return Ok(());
    }

    println!("witness rebuild completed: {}", plan.request_dir.display());
    println!("release id: {}", plan.release_id);
    println!("scratch root: {}", plan.scratch_layout.scratch_root.display());
    println!("verification output: {}", plan.scratch_layout.verification_dir.display());
    println!("witness attestation: {}", attestation_path.display());
    println!("signature: {}", signature_path.display());
    println!("rebuild audit: {}", audit_path.display());
    Ok(())
}
