use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use crunch_attestation::ReleaseAttestation;
use serde::Deserialize;
use serde::Serialize;

use crate::errors::RunError;
use crate::release_attestation::load_release_attestation_document;
use crate::release_evidence::DEFAULT_PROOF_WORKFLOW_COMMAND;
use crate::release_evidence::DEFAULT_PROOF_WORKFLOW_VERSION;
use crate::release_evidence::ReleaseEvidenceManifest;
use crate::release_evidence::compute_path_blake3_digest;
use crate::release_evidence::copy_directory_tree;
use crate::release_evidence::verify_release_evidence_bundle;
use crate::witness_handoff::WITNESS_REQUEST_FILE_NAME;
use crate::witness_handoff::WITNESS_REQUEST_SCHEMA;
use crate::witness_handoff::WitnessRequestDocument;

pub(crate) const WITNESS_REBUILD_AUDIT_SCHEMA: &str = "crunch-witness-rebuild-audit-v1";
pub(crate) const WITNESS_SCRATCH_ENV: &str = "CRUNCH_WITNESS_SCRATCH_DIR";
const MAX_EXPECTED_OUTPUTS: u32 = 16;
const SUPPORTED_WORKFLOW_OUTPUT_COUNT: u32 = 1;
const SCRATCH_REPO_DIR_NAME: &str = "source-tree";
const SCRATCH_REBUILT_OUTPUTS_DIR_NAME: &str = "rebuilt-outputs";
const SCRATCH_PROOF_BUNDLE_DIR_NAME: &str = "proof-bundle";
const SCRATCH_PROOF_WORK_DIR_NAME: &str = "proof-work";
const SCRATCH_AUDIT_DIR_NAME: &str = "witness-rebuild-audit";
const SCRATCH_TMP_DIR_NAME: &str = "tmp";
const SCRATCH_CARGO_TARGET_DIR_NAME: &str = "cargo-target";
const SCRATCH_VERIFICATION_DIR_NAME: &str = "release-verification";
const AUDIT_STDOUT_FILE_NAME: &str = "stdout.txt";
const AUDIT_STDERR_FILE_NAME: &str = "stderr.txt";
const PROOF_MANIFEST_FILE_NAME: &str = "manifest.json";
const WORKFLOW_DRIVER_OVERRIDE_ENV: &str = "CRUNCH_WITNESS_REBUILD_DRIVER";
const WORKFLOW_REQUEST_DIR_ENV: &str = "CRUNCH_WITNESS_REQUEST_DIR";
const WORKFLOW_RELEASE_BUNDLE_DIR_ENV: &str = "CRUNCH_WITNESS_RELEASE_BUNDLE_DIR";
const WORKFLOW_VERIFICATION_SEED_DIR_ENV: &str = "CRUNCH_WITNESS_VERIFICATION_SEED_DIR";
const WORKFLOW_RELEASE_ID_ENV: &str = "CRUNCH_WITNESS_RELEASE_ID";
const SELF_HOSTING_PROOF_BUNDLE_ENV: &str = "CRUNCH_SELF_HOSTING_PROOF_BUNDLE_DIR";
const SELF_HOSTING_PROOF_SCRATCH_ENV: &str = "CRUNCH_PROOF_SCRATCH_DIR";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExpectedRebuiltOutput {
    pub published_name: String,
    pub expected_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WitnessRebuildLayout {
    pub scratch_root: PathBuf,
    pub repo_dir: PathBuf,
    pub rebuilt_outputs_dir: PathBuf,
    pub proof_bundle_dir: PathBuf,
    pub proof_scratch_dir: PathBuf,
    pub audit_dir: PathBuf,
    pub tmp_dir: PathBuf,
    pub cargo_target_dir: PathBuf,
    pub verification_dir: PathBuf,
    pub stdout_log_path: PathBuf,
    pub stderr_log_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WitnessRebuildPlan {
    pub request_dir: PathBuf,
    pub request_path: PathBuf,
    pub release_id: String,
    pub request_schema: String,
    pub request_layout_version: u32,
    pub request_bundle_dir: PathBuf,
    pub request_verification_seed_dir: PathBuf,
    pub request_source_archive_path: PathBuf,
    pub workflow_command: String,
    pub workflow_version: String,
    pub expected_outputs: Vec<ExpectedRebuiltOutput>,
    pub scratch_layout: WitnessRebuildLayout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WitnessRebuildExecution {
    pub rebuilt_output_paths: Vec<PathBuf>,
    pub workflow_driver_path: PathBuf,
    pub output: Output,
    pub started_unix_ms: u64,
    pub finished_unix_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WitnessRebuildSuccess {
    pub rebuilt_output_paths: Vec<PathBuf>,
    pub workflow_driver_path: PathBuf,
    pub started_unix_ms: u64,
    pub finished_unix_ms: u64,
    pub output: Output,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct WitnessRebuildAuditMeta {
    pub schema: String,
    pub release_id: String,
    pub request_dir: String,
    pub request_schema: String,
    pub request_layout_version: u32,
    pub workflow_command: String,
    pub workflow_version: String,
    pub scratch_root: String,
    pub tmp_dir: String,
    pub cargo_target_dir: String,
    pub proof_bundle_dir: String,
    pub proof_scratch_dir: String,
    pub repo_dir: String,
    pub verification_dir: String,
    pub stdout_log_path: String,
    pub stderr_log_path: String,
    pub launched_command: Vec<String>,
    pub started_unix_ms: u64,
    pub finished_unix_ms: u64,
    pub status: String,
    pub rebuilt_outputs: Vec<WitnessRebuildAuditOutput>,
    pub witness_attestation_path: Option<String>,
    pub witness_signature_path: Option<String>,
    pub failure_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct WitnessRebuildAuditOutput {
    pub published_name: String,
    pub path: String,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RequestArtifactPaths {
    request_path: PathBuf,
    bundle_dir: PathBuf,
    verification_seed_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
struct ProofBundleManifest {
    stage2: ProofBundleStage,
}

#[derive(Debug, Deserialize)]
struct ProofBundleStage {
    report: ProofStageReport,
}

#[derive(Debug, Deserialize)]
struct ProofStageReport {
    output_binary: String,
}

pub(crate) fn default_witness_scratch_dir(request_dir: &Path) -> Result<PathBuf, RunError> {
    let parent = request_dir
        .parent()
        .ok_or_else(|| RunError::Internal(format!("witness request dir has no parent: {}", request_dir.display())))?;
    let file_name = request_dir.file_name().and_then(|value| value.to_str()).ok_or_else(|| {
        RunError::Internal(format!("witness request dir has no UTF-8 file name: {}", request_dir.display()))
    })?;
    Ok(parent.join(format!("{file_name}.work")))
}

pub(crate) fn plan_witness_rebuild(request_dir: &Path, scratch_root: &Path) -> Result<WitnessRebuildPlan, RunError> {
    let request = load_witness_request_document(request_dir)?;
    validate_request_schema(&request)?;
    let paths = resolve_request_artifact_paths(request_dir, &request)?;
    let manifest = verify_release_evidence_bundle(&paths.bundle_dir)?;
    let (release_attestation, _stored_path) = load_release_attestation_document(&paths.verification_seed_dir)?;
    validate_release_ids(&request.release_id, &manifest, &release_attestation)?;
    validate_supported_workflow_identity(&manifest.workflow.command, &manifest.workflow.version)?;
    let expected_outputs = build_expected_outputs(&manifest, &release_attestation)?;
    let request_source_archive_path = source_archive_path(&paths.bundle_dir, &manifest)?;
    let scratch_layout = derive_witness_rebuild_layout(scratch_root, &request.release_id)?;
    let request_path = paths.request_path;
    Ok(WitnessRebuildPlan {
        request_dir: request_dir.to_path_buf(),
        request_path,
        release_id: request.release_id,
        request_schema: request.schema,
        request_layout_version: request.request_layout_version,
        request_bundle_dir: paths.bundle_dir,
        request_verification_seed_dir: paths.verification_seed_dir,
        request_source_archive_path,
        workflow_command: manifest.workflow.command,
        workflow_version: manifest.workflow.version,
        expected_outputs,
        scratch_layout,
    })
}

pub(crate) fn prepare_witness_rebuild_scratch(plan: &WitnessRebuildPlan) -> Result<(), RunError> {
    prepare_witness_rebuild_root(&plan.scratch_layout)?;
    copy_request_seed_directory(plan)?;
    extract_source_archive(plan)?;
    Ok(())
}

pub(crate) fn run_witness_rebuild_workflow(plan: &WitnessRebuildPlan) -> Result<WitnessRebuildExecution, RunError> {
    let workflow_driver_path = resolve_workflow_driver_path(plan)?;
    let mut command = Command::new(&workflow_driver_path);
    command.current_dir(&plan.scratch_layout.repo_dir);
    command.env(SELF_HOSTING_PROOF_BUNDLE_ENV, &plan.scratch_layout.proof_bundle_dir);
    command.env(SELF_HOSTING_PROOF_SCRATCH_ENV, &plan.scratch_layout.proof_scratch_dir);
    command.env(WORKFLOW_REQUEST_DIR_ENV, &plan.request_dir);
    command.env(WORKFLOW_RELEASE_BUNDLE_DIR_ENV, &plan.request_bundle_dir);
    command.env(WORKFLOW_VERIFICATION_SEED_DIR_ENV, &plan.request_verification_seed_dir);
    command.env(WORKFLOW_RELEASE_ID_ENV, &plan.release_id);
    command.env("TMPDIR", &plan.scratch_layout.tmp_dir);
    command.env("CARGO_TARGET_DIR", &plan.scratch_layout.cargo_target_dir);
    let started_unix_ms = unix_time_ms_now()?;
    let output = command.output().map_err(|err| {
        RunError::Build(format!("running witness rebuild workflow {}: {err}", workflow_driver_path.display()))
    })?;
    let finished_unix_ms = unix_time_ms_now()?;
    write_command_logs(plan, &output)?;
    let rebuilt_output_paths = if output.status.success() {
        collect_rebuilt_output_paths(plan)?
    } else {
        Vec::new()
    };
    Ok(WitnessRebuildExecution {
        rebuilt_output_paths,
        workflow_driver_path,
        output,
        started_unix_ms,
        finished_unix_ms,
    })
}

pub(crate) fn validate_successful_rebuild(
    plan: &WitnessRebuildPlan,
    execution: &WitnessRebuildExecution,
) -> Result<(), RunError> {
    if !execution.output.status.success() {
        return Err(RunError::Build(format_workflow_failure(execution)));
    }
    validate_rebuilt_output_digests(&plan.expected_outputs, &execution.rebuilt_output_paths)
}

pub(crate) fn build_success_audit_meta(
    plan: &WitnessRebuildPlan,
    success: &WitnessRebuildSuccess,
    attestation_path: &Path,
    signature_path: &Path,
) -> Result<WitnessRebuildAuditMeta, RunError> {
    build_audit_meta(
        plan,
        &success.workflow_driver_path,
        &success.rebuilt_output_paths,
        success.started_unix_ms,
        success.finished_unix_ms,
        audit_status_success(),
        Some(attestation_path),
        Some(signature_path),
        None,
    )
}

pub(crate) fn build_failure_audit_meta(
    plan: &WitnessRebuildPlan,
    workflow_driver_path: &Path,
    rebuilt_output_paths: &[PathBuf],
    started_unix_ms: u64,
    finished_unix_ms: u64,
    failure_message: &str,
) -> Result<WitnessRebuildAuditMeta, RunError> {
    build_audit_meta(
        plan,
        workflow_driver_path,
        rebuilt_output_paths,
        started_unix_ms,
        finished_unix_ms,
        audit_status_failed(),
        None,
        None,
        Some(failure_message.to_string()),
    )
}

pub(crate) fn write_audit_meta(path: &Path, meta: &WitnessRebuildAuditMeta) -> Result<(), RunError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| RunError::Internal(format!("creating {}: {err}", parent.display())))?;
    }
    let bytes = serde_json::to_vec_pretty(meta)
        .map_err(|err| RunError::Internal(format!("serializing {}: {err}", path.display())))?;
    std::fs::write(path, bytes).map_err(|err| RunError::Internal(format!("writing {}: {err}", path.display())))
}

pub(crate) fn audit_meta_path(plan: &WitnessRebuildPlan) -> PathBuf {
    plan.scratch_layout.audit_dir.join("meta.json")
}

fn load_witness_request_document(request_dir: &Path) -> Result<WitnessRequestDocument, RunError> {
    let request_path = request_dir.join(WITNESS_REQUEST_FILE_NAME);
    let bytes = std::fs::read(&request_path)
        .map_err(|err| RunError::Internal(format!("reading {}: {err}", request_path.display())))?;
    serde_json::from_slice(&bytes)
        .map_err(|err| RunError::Internal(format!("parsing {}: {err}", request_path.display())))
}

fn validate_request_schema(request: &WitnessRequestDocument) -> Result<(), RunError> {
    if request.schema == WITNESS_REQUEST_SCHEMA {
        return Ok(());
    }
    Err(RunError::Internal(format!(
        "unsupported witness request schema '{}': expected {}",
        request.schema, WITNESS_REQUEST_SCHEMA
    )))
}

fn resolve_request_artifact_paths(
    request_dir: &Path,
    request: &WitnessRequestDocument,
) -> Result<RequestArtifactPaths, RunError> {
    let request_path = request_dir.join(WITNESS_REQUEST_FILE_NAME);
    let bundle_relative = parse_request_relative_path(&request.release_bundle_relative_path, "release bundle")?;
    let verification_relative =
        parse_request_relative_path(&request.verification_seed_relative_path, "verification seed")?;
    Ok(RequestArtifactPaths {
        request_path,
        bundle_dir: request_dir.join(bundle_relative),
        verification_seed_dir: request_dir.join(verification_relative),
    })
}

fn parse_request_relative_path(path_text: &str, label: &str) -> Result<PathBuf, RunError> {
    let candidate = PathBuf::from(path_text);
    if candidate.as_os_str().is_empty() {
        return Err(RunError::Internal(format!("{label} path must not be empty")));
    }
    if candidate.is_absolute() {
        return Err(RunError::Internal(format!("{label} path must be relative: {}", path_text)));
    }
    for component in candidate.components() {
        if matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_)) {
            return Err(RunError::Internal(format!(
                "{label} path must stay inside the request directory: {}",
                path_text
            )));
        }
    }
    Ok(candidate)
}

fn validate_release_ids(
    request_release_id: &str,
    manifest: &ReleaseEvidenceManifest,
    release_attestation: &ReleaseAttestation,
) -> Result<(), RunError> {
    if request_release_id != manifest.release_id {
        return Err(RunError::Internal(format!(
            "witness request release id '{}' does not match release evidence '{}'",
            request_release_id, manifest.release_id
        )));
    }
    if request_release_id != release_attestation.release_id {
        return Err(RunError::Internal(format!(
            "witness request release id '{}' does not match release attestation '{}'",
            request_release_id, release_attestation.release_id
        )));
    }
    Ok(())
}

fn validate_supported_workflow_identity(workflow_command: &str, workflow_version: &str) -> Result<(), RunError> {
    if workflow_command != DEFAULT_PROOF_WORKFLOW_COMMAND || workflow_version != DEFAULT_PROOF_WORKFLOW_VERSION {
        return Err(RunError::Internal(format!(
            "unsupported witness rebuild workflow '{}'/ '{}': expected '{}'/ '{}'",
            workflow_command, workflow_version, DEFAULT_PROOF_WORKFLOW_COMMAND, DEFAULT_PROOF_WORKFLOW_VERSION
        )));
    }
    Ok(())
}

fn build_expected_outputs(
    manifest: &ReleaseEvidenceManifest,
    release_attestation: &ReleaseAttestation,
) -> Result<Vec<ExpectedRebuiltOutput>, RunError> {
    let manifest_count_u32 = u32::try_from(manifest.binaries.len())
        .map_err(|_| RunError::Internal("release evidence binary count overflowed u32".to_string()))?;
    let attestation_count_u32 = u32::try_from(release_attestation.binary_digests.len())
        .map_err(|_| RunError::Internal("release attestation binary count overflowed u32".to_string()))?;
    if manifest_count_u32 != attestation_count_u32 {
        return Err(RunError::Internal(format!(
            "published output count mismatch: release evidence has {manifest_count_u32}, release attestation has {attestation_count_u32}"
        )));
    }
    if manifest_count_u32 == 0 {
        return Err(RunError::Internal("witness rebuild request has no published outputs".to_string()));
    }
    if manifest_count_u32 > MAX_EXPECTED_OUTPUTS {
        return Err(RunError::Internal(format!(
            "witness rebuild request has {manifest_count_u32} published outputs, limit is {MAX_EXPECTED_OUTPUTS}"
        )));
    }
    let mut outputs = Vec::with_capacity(manifest.binaries.len());
    for (index_usize, (manifest_binary, attested_binary)) in
        manifest.binaries.iter().zip(release_attestation.binary_digests.iter()).enumerate()
    {
        let index_u32 = u32::try_from(index_usize)
            .map_err(|_| RunError::Internal("published output index overflowed u32".to_string()))?;
        validate_output_name_match(index_u32, &manifest_binary.relative_path, &attested_binary.name)?;
        validate_output_digest_match(index_u32, &manifest_binary.digest_blake3, &attested_binary.digest)?;
        outputs.push(ExpectedRebuiltOutput {
            published_name: manifest_binary.relative_path.clone(),
            expected_digest_blake3: manifest_binary.digest_blake3.clone(),
        });
    }
    Ok(outputs)
}

fn validate_output_name_match(index_u32: u32, manifest_name: &str, attested_name: &str) -> Result<(), RunError> {
    if manifest_name == attested_name {
        return Ok(());
    }
    Err(RunError::Internal(format!(
        "published output name mismatch at index {index_u32}: release evidence has '{}', release attestation has '{}'",
        manifest_name, attested_name
    )))
}

fn validate_output_digest_match(index_u32: u32, manifest_digest: &str, attested_digest: &str) -> Result<(), RunError> {
    if manifest_digest == attested_digest {
        return Ok(());
    }
    Err(RunError::Internal(format!(
        "published output digest mismatch at index {index_u32}: release evidence has {}, release attestation has {}",
        manifest_digest, attested_digest
    )))
}

fn derive_witness_rebuild_layout(scratch_root: &Path, release_id: &str) -> Result<WitnessRebuildLayout, RunError> {
    validate_release_id(release_id)?;
    let audit_dir = scratch_root.join(SCRATCH_AUDIT_DIR_NAME);
    Ok(WitnessRebuildLayout {
        scratch_root: scratch_root.to_path_buf(),
        repo_dir: scratch_root.join(SCRATCH_REPO_DIR_NAME),
        rebuilt_outputs_dir: scratch_root.join(SCRATCH_REBUILT_OUTPUTS_DIR_NAME),
        proof_bundle_dir: scratch_root.join(SCRATCH_PROOF_BUNDLE_DIR_NAME),
        proof_scratch_dir: scratch_root.join(SCRATCH_PROOF_WORK_DIR_NAME),
        audit_dir: audit_dir.clone(),
        tmp_dir: scratch_root.join(SCRATCH_TMP_DIR_NAME),
        cargo_target_dir: scratch_root.join(SCRATCH_CARGO_TARGET_DIR_NAME),
        verification_dir: scratch_root.join(SCRATCH_VERIFICATION_DIR_NAME).join(release_id),
        stdout_log_path: audit_dir.join(AUDIT_STDOUT_FILE_NAME),
        stderr_log_path: audit_dir.join(AUDIT_STDERR_FILE_NAME),
    })
}

fn validate_release_id(release_id: &str) -> Result<(), RunError> {
    let trimmed = release_id.trim();
    if trimmed.is_empty() {
        return Err(RunError::Internal("release id must not be empty".to_string()));
    }
    if trimmed.contains('/') || trimmed.contains('\\') {
        return Err(RunError::Internal(format!("release id must not contain path separators: {}", trimmed)));
    }
    Ok(())
}

fn source_archive_path(bundle_dir: &Path, manifest: &ReleaseEvidenceManifest) -> Result<PathBuf, RunError> {
    let relative = parse_request_relative_path(&manifest.source_archive.relative_path, "source archive")?;
    Ok(bundle_dir.join(relative))
}

fn prepare_witness_rebuild_root(layout: &WitnessRebuildLayout) -> Result<(), RunError> {
    if !layout.scratch_root.exists() {
        return std::fs::create_dir_all(&layout.scratch_root)
            .map_err(|err| RunError::Internal(format!("creating {}: {err}", layout.scratch_root.display())));
    }
    validate_existing_scratch_root(&layout.scratch_root)
}

fn validate_existing_scratch_root(path: &Path) -> Result<(), RunError> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|err| RunError::Internal(format!("reading {} metadata: {err}", path.display())))?;
    if metadata.file_type().is_symlink() {
        return Err(RunError::Internal(format!(
            "witness rebuild scratch root must not be a symlink: {}",
            path.display()
        )));
    }
    if !metadata.is_dir() {
        return Err(RunError::Internal(format!("witness rebuild scratch root is not a directory: {}", path.display())));
    }
    let unexpected_entries = unexpected_scratch_root_entries(path)?;
    if unexpected_entries.is_empty() {
        return Ok(());
    }
    Err(RunError::Internal(format!(
        "witness rebuild scratch root must be empty except for helper-owned tmp/cargo-target directories: {} (unexpected entries: {})",
        path.display(),
        unexpected_entries.join(", ")
    )))
}

fn unexpected_scratch_root_entries(path: &Path) -> Result<Vec<String>, RunError> {
    let mut unexpected_entries = Vec::new();
    for entry_result in
        std::fs::read_dir(path).map_err(|err| RunError::Internal(format!("reading {}: {err}", path.display())))?
    {
        let entry =
            entry_result.map_err(|err| RunError::Internal(format!("reading {} entry: {err}", path.display())))?;
        let entry_name = entry.file_name().into_string().map_err(|_| {
            RunError::Internal(format!("witness rebuild scratch entry is not valid UTF-8 under {}", path.display()))
        })?;
        if is_helper_owned_scratch_entry(&entry_name) {
            validate_helper_owned_scratch_entry(&entry.path())?;
            continue;
        }
        unexpected_entries.push(entry_name);
    }
    unexpected_entries.sort();
    Ok(unexpected_entries)
}

fn is_helper_owned_scratch_entry(entry_name: &str) -> bool {
    entry_name == SCRATCH_TMP_DIR_NAME || entry_name == SCRATCH_CARGO_TARGET_DIR_NAME
}

fn validate_helper_owned_scratch_entry(path: &Path) -> Result<(), RunError> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|err| RunError::Internal(format!("reading {} metadata: {err}", path.display())))?;
    if metadata.file_type().is_symlink() {
        return Err(RunError::Internal(format!(
            "witness rebuild helper-owned scratch entry must not be a symlink: {}",
            path.display()
        )));
    }
    if metadata.is_dir() {
        return Ok(());
    }
    Err(RunError::Internal(format!(
        "witness rebuild helper-owned scratch entry must be a directory: {}",
        path.display()
    )))
}

fn copy_request_seed_directory(plan: &WitnessRebuildPlan) -> Result<(), RunError> {
    std::fs::create_dir_all(&plan.scratch_layout.verification_dir).map_err(|err| {
        RunError::Internal(format!("creating {}: {err}", plan.scratch_layout.verification_dir.display()))
    })?;
    copy_directory_tree(&plan.request_verification_seed_dir, &plan.scratch_layout.verification_dir)
}

fn extract_source_archive(plan: &WitnessRebuildPlan) -> Result<(), RunError> {
    if !plan.request_source_archive_path.is_file() {
        return Err(RunError::Internal(format!(
            "witness rebuild source archive missing: {}",
            plan.request_source_archive_path.display()
        )));
    }
    let repo_dir_string = plan.scratch_layout.repo_dir.to_str().ok_or_else(|| {
        RunError::Internal(format!("path is not valid UTF-8: {}", plan.scratch_layout.repo_dir.display()))
    })?;
    let archive_url = format!("file://{}", plan.request_source_archive_path.display());
    crunch_build::fetcher::fetch_and_unpack(&archive_url, repo_dir_string)
        .map_err(|err| RunError::Internal(format!("extracting source archive: {err}")))
}

fn resolve_workflow_driver_path(plan: &WitnessRebuildPlan) -> Result<PathBuf, RunError> {
    if let Ok(override_path) = std::env::var(WORKFLOW_DRIVER_OVERRIDE_ENV) {
        return validate_workflow_driver_path(PathBuf::from(override_path));
    }
    let normalized_command = normalize_workflow_command(&plan.workflow_command)?;
    validate_workflow_driver_path(plan.scratch_layout.repo_dir.join(normalized_command))
}

fn normalize_workflow_command(workflow_command: &str) -> Result<&str, RunError> {
    if workflow_command == DEFAULT_PROOF_WORKFLOW_COMMAND {
        return Ok("scripts/prove-self-hosting.sh");
    }
    Err(RunError::Internal(format!(
        "unsupported witness rebuild workflow command '{}': expected {}",
        workflow_command, DEFAULT_PROOF_WORKFLOW_COMMAND
    )))
}

fn validate_workflow_driver_path(path: PathBuf) -> Result<PathBuf, RunError> {
    if path.is_file() {
        return Ok(path);
    }
    Err(RunError::Internal(format!("witness rebuild workflow driver missing: {}", path.display())))
}

fn write_command_logs(plan: &WitnessRebuildPlan, output: &Output) -> Result<(), RunError> {
    std::fs::create_dir_all(&plan.scratch_layout.audit_dir)
        .map_err(|err| RunError::Internal(format!("creating {}: {err}", plan.scratch_layout.audit_dir.display())))?;
    std::fs::write(&plan.scratch_layout.stdout_log_path, &output.stdout).map_err(|err| {
        RunError::Internal(format!("writing {}: {err}", plan.scratch_layout.stdout_log_path.display()))
    })?;
    std::fs::write(&plan.scratch_layout.stderr_log_path, &output.stderr).map_err(|err| {
        RunError::Internal(format!("writing {}: {err}", plan.scratch_layout.stderr_log_path.display()))
    })?;
    Ok(())
}

fn collect_rebuilt_output_paths(plan: &WitnessRebuildPlan) -> Result<Vec<PathBuf>, RunError> {
    let proof_manifest_path = plan.scratch_layout.proof_bundle_dir.join(PROOF_MANIFEST_FILE_NAME);
    let manifest_bytes = std::fs::read(&proof_manifest_path)
        .map_err(|err| RunError::Build(format!("reading {}: {err}", proof_manifest_path.display())))?;
    let proof_manifest: ProofBundleManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|err| RunError::Build(format!("parsing {}: {err}", proof_manifest_path.display())))?;
    let stage2_binary_path = PathBuf::from(proof_manifest.stage2.report.output_binary);
    if !stage2_binary_path.is_file() {
        return Err(RunError::Build(format!(
            "witness rebuild workflow did not produce stage2 binary: {}",
            stage2_binary_path.display()
        )));
    }
    let expected_count_u32 = u32::try_from(plan.expected_outputs.len())
        .map_err(|_| RunError::Internal("expected rebuilt output count overflowed u32".to_string()))?;
    if expected_count_u32 != SUPPORTED_WORKFLOW_OUTPUT_COUNT {
        return Err(RunError::Build(format!(
            "rebuilt output count mismatch: supported workflow produces {SUPPORTED_WORKFLOW_OUTPUT_COUNT} output, request expects {expected_count_u32}"
        )));
    }
    let expected_output = plan.expected_outputs.first().ok_or_else(|| {
        RunError::Internal("witness rebuild expected outputs unexpectedly empty after validation".to_string())
    })?;
    let staged_output_path = plan
        .scratch_layout
        .rebuilt_outputs_dir
        .join(parse_request_relative_path(&expected_output.published_name, "published output")?);
    copy_rebuilt_output(&stage2_binary_path, &staged_output_path)?;
    Ok(vec![staged_output_path])
}

fn copy_rebuilt_output(source_path: &Path, dest_path: &Path) -> Result<(), RunError> {
    let parent = dest_path
        .parent()
        .ok_or_else(|| RunError::Internal(format!("rebuilt output path has no parent: {}", dest_path.display())))?;
    std::fs::create_dir_all(parent)
        .map_err(|err| RunError::Internal(format!("creating {}: {err}", parent.display())))?;
    std::fs::copy(source_path, dest_path).map_err(|err| {
        RunError::Internal(format!("copying {} to {}: {err}", source_path.display(), dest_path.display()))
    })?;
    Ok(())
}

fn validate_rebuilt_output_digests(
    expected_outputs: &[ExpectedRebuiltOutput],
    rebuilt_output_paths: &[PathBuf],
) -> Result<(), RunError> {
    let expected_count_u32 = u32::try_from(expected_outputs.len())
        .map_err(|_| RunError::Internal("expected rebuilt output count overflowed u32".to_string()))?;
    let actual_count_u32 = u32::try_from(rebuilt_output_paths.len())
        .map_err(|_| RunError::Internal("actual rebuilt output count overflowed u32".to_string()))?;
    if expected_count_u32 != actual_count_u32 {
        return Err(RunError::Build(format!(
            "rebuilt output count mismatch: expected {expected_count_u32}, got {actual_count_u32}"
        )));
    }
    for (expected_output, rebuilt_output_path) in expected_outputs.iter().zip(rebuilt_output_paths.iter()) {
        let actual_digest = compute_path_blake3_digest(rebuilt_output_path)?;
        if actual_digest != expected_output.expected_digest_blake3 {
            return Err(RunError::Build(format!(
                "rebuilt output digest mismatch for {}: expected {}, got {}",
                expected_output.published_name, expected_output.expected_digest_blake3, actual_digest
            )));
        }
    }
    Ok(())
}

fn format_workflow_failure(execution: &WitnessRebuildExecution) -> String {
    let stderr = String::from_utf8_lossy(&execution.output.stderr);
    let stdout = String::from_utf8_lossy(&execution.output.stdout);
    let status = render_exit_status(&execution.output);
    format!(
        "witness rebuild workflow {} failed with {}\nstdout:\n{}\nstderr:\n{}",
        execution.workflow_driver_path.display(),
        status,
        stdout.trim_end(),
        stderr.trim_end()
    )
}

fn render_exit_status(output: &Output) -> String {
    match output.status.code() {
        Some(code) => format!("exit code {code}"),
        None => "termination by signal".to_string(),
    }
}

fn build_audit_meta(
    plan: &WitnessRebuildPlan,
    workflow_driver_path: &Path,
    rebuilt_output_paths: &[PathBuf],
    started_unix_ms: u64,
    finished_unix_ms: u64,
    status: &str,
    attestation_path: Option<&Path>,
    signature_path: Option<&Path>,
    failure_message: Option<String>,
) -> Result<WitnessRebuildAuditMeta, RunError> {
    let rebuilt_outputs = build_audit_outputs(&plan.expected_outputs, rebuilt_output_paths)?;
    let launched_command = vec![workflow_driver_path.display().to_string()];
    Ok(WitnessRebuildAuditMeta {
        schema: WITNESS_REBUILD_AUDIT_SCHEMA.to_string(),
        release_id: plan.release_id.clone(),
        request_dir: plan.request_dir.display().to_string(),
        request_schema: plan.request_schema.clone(),
        request_layout_version: plan.request_layout_version,
        workflow_command: plan.workflow_command.clone(),
        workflow_version: plan.workflow_version.clone(),
        scratch_root: plan.scratch_layout.scratch_root.display().to_string(),
        tmp_dir: plan.scratch_layout.tmp_dir.display().to_string(),
        cargo_target_dir: plan.scratch_layout.cargo_target_dir.display().to_string(),
        proof_bundle_dir: plan.scratch_layout.proof_bundle_dir.display().to_string(),
        proof_scratch_dir: plan.scratch_layout.proof_scratch_dir.display().to_string(),
        repo_dir: plan.scratch_layout.repo_dir.display().to_string(),
        verification_dir: plan.scratch_layout.verification_dir.display().to_string(),
        stdout_log_path: plan.scratch_layout.stdout_log_path.display().to_string(),
        stderr_log_path: plan.scratch_layout.stderr_log_path.display().to_string(),
        launched_command,
        started_unix_ms,
        finished_unix_ms,
        status: status.to_string(),
        rebuilt_outputs,
        witness_attestation_path: attestation_path.map(|path| path.display().to_string()),
        witness_signature_path: signature_path.map(|path| path.display().to_string()),
        failure_message,
    })
}

fn build_audit_outputs(
    expected_outputs: &[ExpectedRebuiltOutput],
    rebuilt_output_paths: &[PathBuf],
) -> Result<Vec<WitnessRebuildAuditOutput>, RunError> {
    let mut audit_outputs = Vec::with_capacity(rebuilt_output_paths.len());
    for (expected_output, rebuilt_output_path) in expected_outputs.iter().zip(rebuilt_output_paths.iter()) {
        let digest_blake3 = if rebuilt_output_path.exists() {
            compute_path_blake3_digest(rebuilt_output_path)?
        } else {
            String::new()
        };
        audit_outputs.push(WitnessRebuildAuditOutput {
            published_name: expected_output.published_name.clone(),
            path: rebuilt_output_path.display().to_string(),
            digest_blake3,
        });
    }
    Ok(audit_outputs)
}

fn unix_time_ms_now() -> Result<u64, RunError> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| RunError::Internal(format!("system clock before unix epoch: {err}")))?;
    u64::try_from(duration.as_millis())
        .map_err(|_| RunError::Internal("unix time overflowed u64 milliseconds".to_string()))
}

fn audit_status_success() -> &'static str {
    "success"
}

fn audit_status_failed() -> &'static str {
    "failed"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_witness_scratch_dir_appends_work_suffix() {
        let request_dir = PathBuf::from("/tmp/request-dir");
        let scratch_dir = default_witness_scratch_dir(&request_dir).unwrap();
        assert_eq!(scratch_dir, PathBuf::from("/tmp/request-dir.work"));
    }

    #[test]
    fn parse_request_relative_path_rejects_parent_components() {
        let err = parse_request_relative_path("../escape", "release bundle").unwrap_err();
        assert!(err.message().contains("must stay inside the request directory"));
    }

    #[test]
    fn validate_supported_workflow_identity_rejects_unknown_pair() {
        let err = validate_supported_workflow_identity("./scripts/other.sh", "v9").unwrap_err();
        assert!(err.message().contains("unsupported witness rebuild workflow"));
    }

    #[test]
    fn validate_existing_scratch_root_allows_helper_owned_dirs_only() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(temp.path().join(SCRATCH_TMP_DIR_NAME)).unwrap();
        std::fs::create_dir_all(temp.path().join(SCRATCH_CARGO_TARGET_DIR_NAME)).unwrap();

        let result = validate_existing_scratch_root(temp.path());
        assert!(result.is_ok(), "helper-owned dirs should be accepted: {result:?}");
    }

    #[cfg(unix)]
    #[test]
    fn validate_existing_scratch_root_rejects_symlinked_helper_owned_tmp_entry() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("tmp-target");
        let link = temp.path().join(SCRATCH_TMP_DIR_NAME);
        std::fs::create_dir_all(&target).unwrap();
        std::os::unix::fs::symlink(&target, &link).unwrap();

        let err = validate_existing_scratch_root(temp.path()).unwrap_err();
        assert!(err.message().contains("helper-owned scratch entry must not be a symlink"));
        assert!(err.message().contains(SCRATCH_TMP_DIR_NAME));
    }

    #[test]
    fn validate_existing_scratch_root_rejects_file_helper_owned_cargo_target_entry() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join(SCRATCH_CARGO_TARGET_DIR_NAME);
        std::fs::write(&file, b"x").unwrap();

        let err = validate_existing_scratch_root(temp.path()).unwrap_err();
        assert!(err.message().contains("helper-owned scratch entry must be a directory"));
        assert!(err.message().contains(SCRATCH_CARGO_TARGET_DIR_NAME));
    }

    #[test]
    fn validate_existing_scratch_root_rejects_unexpected_entries() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("unexpected"), b"x").unwrap();

        let err = validate_existing_scratch_root(temp.path()).unwrap_err();
        assert!(err.message().contains("unexpected entries: unexpected"));
    }

    #[cfg(unix)]
    #[test]
    fn validate_existing_scratch_root_rejects_symlink_root() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("target");
        let link = temp.path().join("link");
        std::fs::create_dir_all(&target).unwrap();
        std::os::unix::fs::symlink(&target, &link).unwrap();

        let err = validate_existing_scratch_root(&link).unwrap_err();
        assert!(err.message().contains("must not be a symlink"));
    }
}
