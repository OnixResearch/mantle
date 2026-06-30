use std::io::Read;
use std::io::Write;
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
use serde_json::Value;

use crate::errors::RunError;
use crate::release_attestation::load_release_attestation_document;
use crate::release_evidence::DEFAULT_PROOF_WORKFLOW_COMMAND;
use crate::release_evidence::DEFAULT_PROOF_WORKFLOW_VERSION;
use crate::release_evidence::ReleaseEvidenceManifest;
use crate::release_evidence::SOURCE_ACQUISITION_KIND_EXTERNAL_ARCHIVE;
use crate::release_evidence::SOURCE_ACQUISITION_KIND_GIT;
use crate::release_evidence::SourceAcquisition;
use crate::release_evidence::compute_path_blake3_digest;
use crate::release_evidence::copy_directory_tree;
use crate::release_evidence::verify_release_evidence_bundle;
use crate::release_source::write_git_source_archive;
use crate::witness_handoff::WITNESS_REQUEST_FILE_NAME;
use crate::witness_handoff::WITNESS_REQUEST_SCHEMA;
use crate::witness_handoff::WitnessRequestDocument;

pub(crate) const WITNESS_REBUILD_AUDIT_SCHEMA: &str = "mantle-witness-rebuild-audit-v1";
pub(crate) const WITNESS_SCRATCH_ENV: &str = "MANTLE_WITNESS_SCRATCH_DIR";
const MAX_EXPECTED_OUTPUTS: u32 = 16;
const PROOF_BINARY_CANDIDATE_LIMIT: usize = 5;
const DIAGNOSTIC_EXCERPT_LIMIT: usize = 5;
const DIAGNOSTIC_MARKERS: &[&str] = &[
    "path leak",
    "cargo-target",
    "OUT_DIR",
    "grammar.rs",
    "digest mismatch",
    "bootstrap",
];
const BOOTSTRAP_DIVERGENCE_ABSENT: &str = "absent";
const BOOTSTRAP_DIVERGENCE_CONVERGED: &str = "converged";
const BOOTSTRAP_DIVERGENCE_DIVERGED: &str = "diverged";
const BOOTSTRAP_DIVERGENCE_UNKNOWN_ROOT: &str = "unknown-bootstrap-output";
const BOOTSTRAP_DIVERGENCE_GCC_ROOT: &str = "gcc.drv";
const BOOTSTRAP_DIVERGENCE_BWRAP_ROOT: &str = "bwrap.drv";
const BOOTSTRAP_DIVERGENCE_BUSYBOX_ROOT: &str = "busybox.drv";
const BOOTSTRAP_DIVERGENCE_STAGE_MANTLE_ROOT: &str = "stage2-mantle";
const BOOTSTRAP_DIVERGENCE_ROOT_POINTER: &str = "/bootstrap_divergence/root";
const BOOTSTRAP_DIVERGENCE_STAGE1_DIGEST_POINTER: &str = "/bootstrap_divergence/stage1_digest_blake3";
const BOOTSTRAP_DIVERGENCE_STAGE2_DIGEST_POINTER: &str = "/bootstrap_divergence/stage2_digest_blake3";
const BOOTSTRAP_STAGE1_EQUALS_STAGE2_POINTER: &str = "/fixed_point/stage1_equals_stage2";
const BOOTSTRAP_BWRAP_EQUALS_POINTER: &str = "/fixed_point/stage0_bwrap_equals_stage2_bwrap";
const BOOTSTRAP_BUSYBOX_EQUALS_POINTER: &str = "/fixed_point/stage0_busybox_equals_stage2_busybox";
const BOOTSTRAP_STAGE1_DIGEST_POINTER: &str = "/binaries/stage1/digest_blake3";
const BOOTSTRAP_STAGE2_DIGEST_POINTER: &str = "/binaries/stage2/digest_blake3";
const BOOTSTRAP_STAGE0_BWRAP_DIGEST_POINTER: &str = "/tools/stage0_bwrap/digest_blake3";
const BOOTSTRAP_STAGE2_BWRAP_DIGEST_POINTER: &str = "/tools/stage2_bwrap/digest_blake3";
const BOOTSTRAP_STAGE0_BUSYBOX_DIGEST_POINTER: &str = "/tools/stage0_busybox/digest_blake3";
const BOOTSTRAP_STAGE2_BUSYBOX_DIGEST_POINTER: &str = "/tools/stage2_busybox/digest_blake3";
const SCRATCH_REPO_DIR_NAME: &str = "source-tree";
const SCRATCH_REBUILT_OUTPUTS_DIR_NAME: &str = "rebuilt-outputs";
const SCRATCH_PROOF_BUNDLE_DIR_NAME: &str = "proof-bundle";
const SCRATCH_PROVIDER_FIXED_POINT_PROOF_DIR_NAME: &str = "provider-fixed-point-proof";
const SCRATCH_PROOF_WORK_DIR_NAME: &str = "proof-work";
const SCRATCH_AUDIT_DIR_NAME: &str = "witness-rebuild-audit";
const SCRATCH_TMP_DIR_NAME: &str = "tmp";
const SCRATCH_CARGO_TARGET_DIR_NAME: &str = "cargo-target";
const SCRATCH_VERIFICATION_DIR_NAME: &str = "release-verification";
const SOURCE_ACQUISITION_ARCHIVE_FILE_NAME: &str = "source-acquisition.tar";
const SOURCE_ACQUISITION_GIT_WORK_DIR_NAME: &str = "git-source";
const AUDIT_STDOUT_FILE_NAME: &str = "stdout.txt";
const AUDIT_STDERR_FILE_NAME: &str = "stderr.txt";
const PROOF_MANIFEST_FILE_NAME: &str = "manifest.json";
const WORKFLOW_DRIVER_OVERRIDE_ENV: &str = "CRUNCH_WITNESS_REBUILD_DRIVER";
const WORKFLOW_REQUEST_DIR_ENV: &str = "CRUNCH_WITNESS_REQUEST_DIR";
const WORKFLOW_RELEASE_BUNDLE_DIR_ENV: &str = "CRUNCH_WITNESS_RELEASE_BUNDLE_DIR";
const WORKFLOW_VERIFICATION_SEED_DIR_ENV: &str = "CRUNCH_WITNESS_VERIFICATION_SEED_DIR";
const WORKFLOW_RELEASE_ID_ENV: &str = "CRUNCH_WITNESS_RELEASE_ID";
const WORKFLOW_REPO_DIR_ENV: &str = "CRUNCH_WITNESS_REBUILD_REPO_DIR";
const WORKFLOW_PROVIDER_FIXED_POINT_PROOF_BUNDLE_ENV: &str = "CRUNCH_WITNESS_PROVIDER_FIXED_POINT_PROOF_BUNDLE_DIR";
const SELF_HOSTING_PROOF_BUNDLE_ENV: &str = "CRUNCH_SELF_HOSTING_PROOF_BUNDLE_DIR";
const SELF_HOSTING_PROOF_SCRATCH_ENV: &str = "CRUNCH_PROOF_SCRATCH_DIR";
const SELF_HOSTING_LATER_STAGE_HERMETICITY_ENV: &str = "CRUNCH_SELF_HOSTING_LATER_STAGE_HERMETICITY_MODE";
const STRICT_HERMETICITY_MODE: &str = "strict";
const PROOF_MODE_FIXED_POINT: &str = "fixed-point";
const PROOF_MODE_NON_NIX_HOST: &str = "non-nix-host";
const WORKFLOW_NON_NIX_HOST_ARG: &str = "--non-nix-host";
const WORKFLOW_ENV_REMOVE: &[&str] = &[
    "CARGO_BUILD_RUSTC",
    "CARGO_BUILD_RUSTC_WRAPPER",
    "CARGO_BUILD_TARGET",
    "CARGO_ENCODED_RUSTFLAGS",
    "CRUNCH_PROOF_OPENSSL_PKGCONFIG",
    "CRUNCH_PROOF_RUSTUP_TOOLCHAIN",
    "CRUNCH_PROOF_SCRATCH_DIR",
    "CRUNCH_PROOF_STATIC_BUSYBOX_CANDIDATE",
    "CRUNCH_SELF_HOSTING_LATER_STAGE_HERMETICITY_MODE",
    "CRUNCH_SELF_HOSTING_PROOF_BUNDLE_DIR",
    "RUSTC",
    "RUSTC_WRAPPER",
    "RUSTFLAGS",
    "SCCACHE_DIR",
    "SCCACHE_ERROR_LOG",
    "SCCACHE_IGNORE_SERVER_IO_ERROR",
    "SCCACHE_LOG",
];
const WORKFLOW_ARGS_FIXED_POINT: &[&str] = &[];
const WORKFLOW_ARGS_NON_NIX_HOST: &[&str] = &[WORKFLOW_NON_NIX_HOST_ARG];
const SOURCE_ACQUISITION_MODE_COPIED: &str = "copied-source";
const SOURCE_ACQUISITION_MODE_EXTERNAL_ARCHIVE: &str = "external-archive-source";
const SOURCE_ACQUISITION_MODE_GIT: &str = "git-derived-source";
const SOURCE_ACQUISITION_STATUS_VERIFIED: &str = "verified";
const BYTES_PER_KIB: u64 = 1024;
const KIB_PER_MIB: u64 = 1024;
const MIB_PER_GIB: u64 = 1024;
const MAX_SOURCE_ARCHIVE_FETCH_GIB: u64 = 4;
const MAX_SOURCE_ARCHIVE_FETCH_BYTES: u64 = MAX_SOURCE_ARCHIVE_FETCH_GIB * MIB_PER_GIB * KIB_PER_MIB * BYTES_PER_KIB;
const SOURCE_FETCH_BUFFER_KIB: usize = 64;
const BYTES_PER_KIB_USIZE: usize = 1024;
const SOURCE_FETCH_BUFFER_BYTES: usize = SOURCE_FETCH_BUFFER_KIB * BYTES_PER_KIB_USIZE;

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
    pub provider_fixed_point_proof_dir: PathBuf,
    pub proof_scratch_dir: PathBuf,
    pub audit_dir: PathBuf,
    pub tmp_dir: PathBuf,
    pub cargo_target_dir: PathBuf,
    pub source_acquisition_archive_path: PathBuf,
    pub source_acquisition_git_work_dir: PathBuf,
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
    pub proof_mode: String,
    pub expected_outputs: Vec<ExpectedRebuiltOutput>,
    pub require_independent_source: bool,
    pub require_git_source: bool,
    pub source_acquisition: Option<SourceAcquisition>,
    pub scratch_layout: WitnessRebuildLayout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WitnessRebuildExecution {
    pub rebuilt_output_paths: Vec<PathBuf>,
    pub rebuilt_output_error: Option<String>,
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
    pub provider_fixed_point_proof_dir: String,
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
    pub diagnostics: Option<WitnessRebuildDiagnostics>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_acquisition: Option<WitnessSourceAcquisitionAudit>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct WitnessSourceAcquisitionAudit {
    pub mode: String,
    pub url: Option<String>,
    pub digest_blake3: Option<String>,
    pub fetched_path: Option<String>,
    pub commit: Option<String>,
    pub reference: Option<String>,
    pub tag: Option<String>,
    pub archive_profile: Option<String>,
    pub archive_version: Option<String>,
    pub status: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct WitnessRebuildAuditOutput {
    pub published_name: String,
    pub path: String,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct WitnessRebuildDiagnostics {
    pub expected_outputs: Vec<WitnessExpectedOutputDiagnostic>,
    pub available_proof_digests: Vec<WitnessAvailableProofDigest>,
    pub provider_proof_status: String,
    pub self_hosting_fixed_point_status: String,
    pub bootstrap_divergence: WitnessBootstrapDivergenceDiagnostic,
    pub path_leak_scan_excerpt: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct WitnessBootstrapDivergenceDiagnostic {
    pub status: String,
    pub root: Option<String>,
    pub stage1_digest_blake3: Option<String>,
    pub stage2_digest_blake3: Option<String>,
    pub path_leak_scan_summary: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct WitnessExpectedOutputDiagnostic {
    pub published_name: String,
    pub expected_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct WitnessAvailableProofDigest {
    pub role: String,
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
    #[serde(default)]
    binaries: Option<ProofBundleBinaries>,
    stage2: ProofBundleStage,
}

#[derive(Debug, Deserialize)]
struct ProofBundleBinaries {
    #[serde(default)]
    stage1: Option<ProofBundleBinary>,
    #[serde(default)]
    stage2: Option<ProofBundleBinary>,
}

#[derive(Debug, Deserialize)]
struct ProofBundleBinary {
    path: String,
    digest_blake3: String,
}

#[derive(Debug, Deserialize)]
struct ProofBundleStage {
    report: ProofStageReport,
}

#[derive(Debug, Deserialize)]
struct ProofStageReport {
    output_binary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RebuiltOutputCandidate {
    role: &'static str,
    path: PathBuf,
    digest_blake3: String,
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

pub(crate) fn plan_witness_rebuild(
    request_dir: &Path,
    scratch_root: &Path,
    require_independent_source: bool,
    require_git_source: bool,
) -> Result<WitnessRebuildPlan, RunError> {
    validate_source_replay_flags(require_independent_source, require_git_source)?;
    let request = load_witness_request_document(request_dir)?;
    validate_request_schema(&request)?;
    let paths = resolve_request_artifact_paths(request_dir, &request)?;
    let manifest = verify_release_evidence_bundle(&paths.bundle_dir)?;
    let (release_attestation, _stored_path) = load_release_attestation_document(&paths.verification_seed_dir)?;
    validate_release_ids(&request.release_id, &manifest, &release_attestation)?;
    validate_supported_workflow_identity(&manifest.workflow.command, &manifest.workflow.version)?;
    let expected_outputs = build_expected_outputs(&manifest, &release_attestation)?;
    let request_source_archive_path = source_archive_path(&paths.bundle_dir, &manifest)?;
    let source_acquisition = source_acquisition_for_plan(&manifest, require_independent_source, require_git_source)?;
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
        proof_mode: manifest.proof_linkage.proof_mode,
        expected_outputs,
        require_independent_source,
        require_git_source,
        source_acquisition,
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
    let workflow_args = workflow_args_for_proof_mode(&plan.proof_mode)?;
    let mut command = Command::new(&workflow_driver_path);
    command.current_dir(&plan.scratch_layout.repo_dir);
    command.args(workflow_args);
    sanitize_workflow_environment(&mut command);
    command.env(SELF_HOSTING_PROOF_BUNDLE_ENV, &plan.scratch_layout.proof_bundle_dir);
    command.env(SELF_HOSTING_PROOF_SCRATCH_ENV, &plan.scratch_layout.proof_scratch_dir);
    command.env(SELF_HOSTING_LATER_STAGE_HERMETICITY_ENV, STRICT_HERMETICITY_MODE);
    command.env(WORKFLOW_REQUEST_DIR_ENV, &plan.request_dir);
    command.env(WORKFLOW_RELEASE_BUNDLE_DIR_ENV, &plan.request_bundle_dir);
    command.env(WORKFLOW_VERIFICATION_SEED_DIR_ENV, &plan.request_verification_seed_dir);
    command.env(WORKFLOW_RELEASE_ID_ENV, &plan.release_id);
    command.env(WORKFLOW_REPO_DIR_ENV, &plan.scratch_layout.repo_dir);
    command.env(WORKFLOW_PROVIDER_FIXED_POINT_PROOF_BUNDLE_ENV, &plan.scratch_layout.provider_fixed_point_proof_dir);
    command.env("TMPDIR", &plan.scratch_layout.tmp_dir);
    command.env("CARGO_TARGET_DIR", &plan.scratch_layout.cargo_target_dir);
    let started_unix_ms = unix_time_ms_now()?;
    let output = command.output().map_err(|err| {
        RunError::Build(format!("running witness rebuild workflow {}: {err}", workflow_driver_path.display()))
    })?;
    let finished_unix_ms = unix_time_ms_now()?;
    write_command_logs(plan, &output)?;
    let (rebuilt_output_paths, rebuilt_output_error) = collect_workflow_rebuilt_outputs(plan, &output);
    Ok(WitnessRebuildExecution {
        rebuilt_output_paths,
        rebuilt_output_error,
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
    if let Some(message) = &execution.rebuilt_output_error {
        return Err(RunError::Build(message.clone()));
    }
    validate_rebuilt_output_digests(&plan.expected_outputs, &execution.rebuilt_output_paths)
}

fn collect_workflow_rebuilt_outputs(plan: &WitnessRebuildPlan, output: &Output) -> (Vec<PathBuf>, Option<String>) {
    if !output.status.success() {
        return (Vec::new(), None);
    }
    match collect_rebuilt_output_paths(plan) {
        Ok(paths) => (paths, None),
        Err(err) => (Vec::new(), Some(err.message().to_string())),
    }
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
        None,
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
        Some(build_failure_diagnostics(plan, failure_message)),
        None,
    )
}

pub(crate) fn build_prelaunch_failure_audit_meta(
    plan: &WitnessRebuildPlan,
    failure_message: &str,
) -> Result<WitnessRebuildAuditMeta, RunError> {
    let timestamp_ms = unix_time_ms_now()?;
    build_audit_meta(
        plan,
        Path::new("<not-launched>"),
        &[],
        timestamp_ms,
        timestamp_ms,
        audit_status_failed(),
        None,
        None,
        Some(failure_message.to_string()),
        Some(build_failure_diagnostics(plan, failure_message)),
        Some(failure_message),
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
        provider_fixed_point_proof_dir: scratch_root.join(SCRATCH_PROVIDER_FIXED_POINT_PROOF_DIR_NAME),
        proof_scratch_dir: scratch_root.join(SCRATCH_PROOF_WORK_DIR_NAME),
        audit_dir: audit_dir.clone(),
        tmp_dir: scratch_root.join(SCRATCH_TMP_DIR_NAME),
        cargo_target_dir: scratch_root.join(SCRATCH_CARGO_TARGET_DIR_NAME),
        source_acquisition_archive_path: scratch_root.join(SOURCE_ACQUISITION_ARCHIVE_FILE_NAME),
        source_acquisition_git_work_dir: scratch_root.join(SOURCE_ACQUISITION_GIT_WORK_DIR_NAME),
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

fn validate_source_replay_flags(require_independent_source: bool, require_git_source: bool) -> Result<(), RunError> {
    if require_independent_source && require_git_source {
        return Err(RunError::Internal(
            "--require-independent-source and --require-git-source are mutually exclusive".to_string(),
        ));
    }
    Ok(())
}

fn source_acquisition_for_plan(
    manifest: &ReleaseEvidenceManifest,
    require_independent_source: bool,
    require_git_source: bool,
) -> Result<Option<SourceAcquisition>, RunError> {
    if require_git_source {
        return required_source_acquisition_kind(manifest, SOURCE_ACQUISITION_KIND_GIT, "Git source").map(Some);
    }
    if require_independent_source {
        return required_source_acquisition_kind(
            manifest,
            SOURCE_ACQUISITION_KIND_EXTERNAL_ARCHIVE,
            "independent source acquisition",
        )
        .map(Some);
    }
    Ok(manifest.source_acquisition.clone())
}

fn required_source_acquisition_kind(
    manifest: &ReleaseEvidenceManifest,
    expected_kind: &str,
    requirement_label: &str,
) -> Result<SourceAcquisition, RunError> {
    let source_acquisition = manifest.source_acquisition.as_ref().ok_or_else(|| {
        RunError::Internal(format!("{requirement_label} required but release manifest has no source_acquisition entry"))
    })?;
    if source_acquisition.kind == expected_kind {
        return Ok(source_acquisition.clone());
    }
    Err(RunError::Internal(format!(
        "{requirement_label} required but release manifest source_acquisition.kind is {}",
        source_acquisition.kind
    )))
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
    let source_archive_path = source_archive_for_extraction(plan)?;
    if !source_archive_path.is_file() {
        return Err(RunError::Internal(format!(
            "witness rebuild source archive missing: {}",
            source_archive_path.display()
        )));
    }
    extract_source_archive_from_path(&source_archive_path, &plan.scratch_layout.repo_dir)
}

fn source_archive_for_extraction(plan: &WitnessRebuildPlan) -> Result<PathBuf, RunError> {
    if plan.require_git_source {
        let source_acquisition = plan.source_acquisition.as_ref().ok_or_else(|| {
            RunError::Internal("Git source required but release manifest has no source_acquisition entry".to_string())
        })?;
        regenerate_and_verify_git_source_acquisition(source_acquisition, &plan.scratch_layout)?;
        return Ok(plan.scratch_layout.source_acquisition_archive_path.clone());
    }
    if plan.require_independent_source {
        let source_acquisition = plan.source_acquisition.as_ref().ok_or_else(|| {
            RunError::Internal(
                "independent source acquisition required but release manifest has no source_acquisition entry"
                    .to_string(),
            )
        })?;
        fetch_and_verify_source_acquisition(source_acquisition, &plan.scratch_layout.source_acquisition_archive_path)?;
        return Ok(plan.scratch_layout.source_acquisition_archive_path.clone());
    }
    Ok(plan.request_source_archive_path.clone())
}

fn regenerate_and_verify_git_source_acquisition(
    source_acquisition: &SourceAcquisition,
    layout: &WitnessRebuildLayout,
) -> Result<(), RunError> {
    write_git_source_archive(
        source_acquisition,
        &layout.source_acquisition_git_work_dir,
        &layout.source_acquisition_archive_path,
    )?;
    let generated_digest = compute_path_blake3_digest(&layout.source_acquisition_archive_path)?;
    if generated_digest == source_acquisition.digest_blake3 {
        return Ok(());
    }
    Err(RunError::Internal(format!(
        "Git source archive digest mismatch: expected {}, generated {} from {} at commit {}",
        source_acquisition.digest_blake3,
        generated_digest,
        source_acquisition.url,
        source_acquisition.commit.as_deref().unwrap_or("<missing>")
    )))
}

fn extract_source_archive_from_path(source_archive_path: &Path, repo_dir: &Path) -> Result<(), RunError> {
    let repo_dir_string = repo_dir
        .to_str()
        .ok_or_else(|| RunError::Internal(format!("path is not valid UTF-8: {}", repo_dir.display())))?;
    let archive_url = format!("file://{}", source_archive_path.display());
    crunch_build::fetcher::fetch_and_unpack(&archive_url, repo_dir_string)
        .map_err(|err| RunError::Internal(format!("extracting source archive: {err}")))
}

fn fetch_and_verify_source_acquisition(
    source_acquisition: &SourceAcquisition,
    destination_path: &Path,
) -> Result<(), RunError> {
    fetch_source_archive_to_path(&source_acquisition.url, destination_path)?;
    let fetched_digest = compute_path_blake3_digest(destination_path)?;
    if fetched_digest == source_acquisition.digest_blake3 {
        return Ok(());
    }
    Err(RunError::Internal(format!(
        "independent source acquisition digest mismatch: expected {}, fetched {} from {}",
        source_acquisition.digest_blake3, fetched_digest, source_acquisition.url
    )))
}

fn fetch_source_archive_to_path(url: &str, destination_path: &Path) -> Result<(), RunError> {
    if let Some(parent) = destination_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| RunError::Internal(format!("creating {}: {err}", parent.display())))?;
    }
    let mut reader = open_source_archive_reader(url)?;
    let temporary_path = destination_path.with_extension("download");
    let mut output = std::fs::File::create(&temporary_path)
        .map_err(|err| RunError::Internal(format!("creating {}: {err}", temporary_path.display())))?;
    copy_source_archive_with_limit(&mut reader, &mut output, MAX_SOURCE_ARCHIVE_FETCH_BYTES)?;
    output
        .flush()
        .map_err(|err| RunError::Internal(format!("flushing {}: {err}", temporary_path.display())))?;
    drop(output);
    std::fs::rename(&temporary_path, destination_path).map_err(|err| {
        RunError::Internal(format!(
            "moving independent source archive {} to {}: {err}",
            temporary_path.display(),
            destination_path.display()
        ))
    })
}

fn open_source_archive_reader(url: &str) -> Result<Box<dyn Read>, RunError> {
    if let Some(path) = url.strip_prefix("file://") {
        let file = std::fs::File::open(path)
            .map_err(|err| RunError::Internal(format!("opening independent source archive {url}: {err}")))?;
        return Ok(Box::new(file));
    }
    if url.starts_with("http://") || url.starts_with("https://") {
        let response = ureq::get(url)
            .call()
            .map_err(|err| RunError::Internal(format!("fetching independent source archive {url}: {err}")))?;
        return Ok(Box::new(response.into_body().into_reader()));
    }
    Err(RunError::Internal(format!("unsupported independent source acquisition URL scheme: {url}")))
}

fn copy_source_archive_with_limit(
    reader: &mut dyn Read,
    writer: &mut dyn Write,
    limit_bytes: u64,
) -> Result<u64, RunError> {
    let mut total_bytes = 0u64;
    let mut buffer = [0u8; SOURCE_FETCH_BUFFER_BYTES];
    loop {
        let read_bytes = reader
            .read(&mut buffer)
            .map_err(|err| RunError::Internal(format!("reading independent source archive: {err}")))?;
        if read_bytes == 0 {
            return Ok(total_bytes);
        }
        let read_bytes_u64 = u64::try_from(read_bytes)
            .map_err(|_| RunError::Internal("independent source archive read size overflowed u64".to_string()))?;
        let next_total = total_bytes
            .checked_add(read_bytes_u64)
            .ok_or_else(|| RunError::Internal("independent source archive byte count overflowed u64".to_string()))?;
        if next_total > limit_bytes {
            return Err(RunError::Internal(format!(
                "independent source archive exceeds {limit_bytes} byte download limit"
            )));
        }
        writer
            .write_all(&buffer[..read_bytes])
            .map_err(|err| RunError::Internal(format!("writing independent source archive: {err}")))?;
        total_bytes = next_total;
    }
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

fn workflow_args_for_proof_mode(proof_mode: &str) -> Result<&'static [&'static str], RunError> {
    if proof_mode == PROOF_MODE_FIXED_POINT {
        return Ok(WORKFLOW_ARGS_FIXED_POINT);
    }
    if proof_mode == PROOF_MODE_NON_NIX_HOST {
        return Ok(WORKFLOW_ARGS_NON_NIX_HOST);
    }
    Err(RunError::Internal(format!("unsupported witness rebuild proof mode: {proof_mode}")))
}

fn sanitize_workflow_environment(command: &mut Command) {
    for env_name in WORKFLOW_ENV_REMOVE {
        command.env_remove(env_name);
    }
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
    let proof_manifest = load_proof_bundle_manifest(&plan.scratch_layout.proof_bundle_dir)?;
    let mut candidates = collect_rebuilt_output_candidates(&plan.scratch_layout.proof_bundle_dir, &proof_manifest)?;
    collect_provider_fixed_point_candidates(&plan.scratch_layout.provider_fixed_point_proof_dir, &mut candidates)?;
    assert!(candidates.len() <= PROOF_BINARY_CANDIDATE_LIMIT);
    copy_matching_rebuilt_outputs(plan, &candidates)
}

fn load_proof_bundle_manifest(proof_bundle_dir: &Path) -> Result<ProofBundleManifest, RunError> {
    let proof_manifest_path = proof_bundle_dir.join(PROOF_MANIFEST_FILE_NAME);
    let manifest_bytes = std::fs::read(&proof_manifest_path)
        .map_err(|err| RunError::Build(format!("reading {}: {err}", proof_manifest_path.display())))?;
    serde_json::from_slice(&manifest_bytes)
        .map_err(|err| RunError::Build(format!("parsing {}: {err}", proof_manifest_path.display())))
}

fn collect_rebuilt_output_candidates(
    proof_bundle_dir: &Path,
    proof_manifest: &ProofBundleManifest,
) -> Result<Vec<RebuiltOutputCandidate>, RunError> {
    let mut candidates = Vec::with_capacity(PROOF_BINARY_CANDIDATE_LIMIT);
    if let Some(binaries) = &proof_manifest.binaries {
        push_manifest_binary_candidate(&mut candidates, proof_bundle_dir, binaries.stage2.as_ref(), "binaries.stage2")?;
        push_manifest_binary_candidate(&mut candidates, proof_bundle_dir, binaries.stage1.as_ref(), "binaries.stage1")?;
    }
    push_legacy_stage2_candidate(&mut candidates, proof_bundle_dir, &proof_manifest.stage2.report.output_binary)?;
    if candidates.is_empty() {
        return Err(RunError::Build("witness rebuild proof bundle contains no rebuilt binary candidates".to_string()));
    }
    assert!(candidates.len() <= PROOF_BINARY_CANDIDATE_LIMIT);
    Ok(candidates)
}

fn collect_provider_fixed_point_candidates(
    provider_proof_dir: &Path,
    candidates: &mut Vec<RebuiltOutputCandidate>,
) -> Result<(), RunError> {
    if !provider_proof_dir.exists() {
        return Ok(());
    }
    if !provider_proof_dir.is_dir() {
        return Err(RunError::Build(format!(
            "witness rebuild provider fixed-point proof path is not a directory: {}",
            provider_proof_dir.display()
        )));
    }
    let verification = crate::cargo_free_self_build::verify_provider_fixed_point_proof_bundle(provider_proof_dir);
    if !verification.valid {
        return Err(RunError::Build(format!(
            "witness rebuild provider fixed-point proof is invalid: {}",
            verification.blockers.join("; ")
        )));
    }
    let stage_digest = verification.stage_binary_digest_blake3.as_deref().ok_or_else(|| {
        RunError::Build("witness rebuild provider fixed-point proof has no stage binary digest".to_string())
    })?;
    let meta = load_provider_fixed_point_meta(provider_proof_dir)?;
    push_provider_stage_candidate(candidates, provider_proof_dir, &meta, "stage2", stage_digest)?;
    push_provider_stage_candidate(candidates, provider_proof_dir, &meta, "stage1", stage_digest)?;
    Ok(())
}

fn load_provider_fixed_point_meta(provider_proof_dir: &Path) -> Result<Value, RunError> {
    let meta_path = provider_proof_dir.join("meta.json");
    let meta_bytes =
        std::fs::read(&meta_path).map_err(|err| RunError::Build(format!("reading {}: {err}", meta_path.display())))?;
    serde_json::from_slice(&meta_bytes)
        .map_err(|err| RunError::Build(format!("parsing {}: {err}", meta_path.display())))
}

fn push_provider_stage_candidate(
    candidates: &mut Vec<RebuiltOutputCandidate>,
    provider_proof_dir: &Path,
    meta: &Value,
    stage_name: &'static str,
    stage_digest: &str,
) -> Result<(), RunError> {
    let role = provider_fixed_point_stage_role(stage_name)?;
    let path = resolve_provider_fixed_point_stage_binary(provider_proof_dir, meta, stage_name, role)?;
    push_verified_candidate(candidates, role, path, Some(stage_digest))
}

fn provider_fixed_point_stage_role(stage_name: &str) -> Result<&'static str, RunError> {
    if stage_name == "stage2" {
        return Ok("provider-fixed-point.stage2");
    }
    if stage_name == "stage1" {
        return Ok("provider-fixed-point.stage1");
    }
    Err(RunError::Internal(format!("unsupported provider fixed-point stage: {stage_name}")))
}

fn resolve_provider_fixed_point_stage_binary(
    provider_proof_dir: &Path,
    meta: &Value,
    stage_name: &str,
    role: &str,
) -> Result<PathBuf, RunError> {
    let pointer = format!("/{stage_name}/binary");
    let recorded_path = meta.pointer(&pointer).and_then(Value::as_str).ok_or_else(|| {
        RunError::Build(format!("provider fixed-point proof metadata missing {role} binary path at {pointer}"))
    })?;
    validate_non_empty_recorded_path(recorded_path, role)?;
    let raw_path = PathBuf::from(recorded_path);
    let candidate_path = if raw_path.is_absolute() {
        raw_path
    } else {
        reject_escaping_relative_path(&raw_path, recorded_path, role)?;
        provider_proof_dir.join(raw_path)
    };
    validate_provider_candidate_containment(provider_proof_dir, &candidate_path, recorded_path, role)?;
    Ok(candidate_path)
}

fn validate_provider_candidate_containment(
    provider_proof_dir: &Path,
    candidate_path: &Path,
    recorded_path: &str,
    role: &str,
) -> Result<(), RunError> {
    if !candidate_path.is_file() {
        return Err(RunError::Build(format!(
            "witness rebuild proof candidate {role} is missing: {}",
            candidate_path.display()
        )));
    }
    let canonical_root = std::fs::canonicalize(provider_proof_dir).map_err(|err| {
        RunError::Build(format!(
            "canonicalizing provider fixed-point proof directory {}: {err}",
            provider_proof_dir.display()
        ))
    })?;
    let canonical_candidate = std::fs::canonicalize(candidate_path).map_err(|err| {
        RunError::Build(format!(
            "canonicalizing provider fixed-point proof candidate {}: {err}",
            candidate_path.display()
        ))
    })?;
    if canonical_candidate.starts_with(&canonical_root) {
        return Ok(());
    }
    Err(RunError::Build(format!(
        "{role} must stay inside provider fixed-point proof bundle: {recorded_path}"
    )))
}

fn push_manifest_binary_candidate(
    candidates: &mut Vec<RebuiltOutputCandidate>,
    proof_bundle_dir: &Path,
    binary: Option<&ProofBundleBinary>,
    role: &'static str,
) -> Result<(), RunError> {
    if let Some(binary) = binary {
        let path = resolve_contained_proof_bundle_artifact_path(proof_bundle_dir, &binary.path, role)?;
        push_verified_candidate(candidates, role, path, Some(&binary.digest_blake3))?;
    }
    Ok(())
}

fn push_legacy_stage2_candidate(
    candidates: &mut Vec<RebuiltOutputCandidate>,
    proof_bundle_dir: &Path,
    output_binary: &str,
) -> Result<(), RunError> {
    let role = "stage2.report.output_binary";
    let path = resolve_labeled_proof_bundle_artifact_path(proof_bundle_dir, output_binary, role)?;
    push_verified_candidate(candidates, role, path, None)
}

fn push_verified_candidate(
    candidates: &mut Vec<RebuiltOutputCandidate>,
    role: &'static str,
    path: PathBuf,
    recorded_digest: Option<&str>,
) -> Result<(), RunError> {
    if candidate_path_already_recorded(candidates, &path) {
        return Ok(());
    }
    if !path.is_file() {
        return Err(RunError::Build(format!("witness rebuild proof candidate {role} is missing: {}", path.display())));
    }
    let digest_blake3 = compute_path_blake3_digest(&path)?;
    if let Some(recorded) = recorded_digest {
        validate_recorded_candidate_digest(role, &path, recorded, &digest_blake3)?;
    }
    candidates.push(RebuiltOutputCandidate {
        role,
        path,
        digest_blake3,
    });
    Ok(())
}

fn candidate_path_already_recorded(candidates: &[RebuiltOutputCandidate], path: &Path) -> bool {
    candidates.iter().any(|candidate| candidate.path == path)
}

fn validate_recorded_candidate_digest(
    role: &str,
    path: &Path,
    recorded_digest: &str,
    actual_digest: &str,
) -> Result<(), RunError> {
    if recorded_digest == actual_digest {
        return Ok(());
    }
    Err(RunError::Build(format!(
        "proof manifest {role} digest mismatch for {}: recorded {}, got {}",
        path.display(),
        recorded_digest,
        actual_digest
    )))
}

fn copy_matching_rebuilt_outputs(
    plan: &WitnessRebuildPlan,
    candidates: &[RebuiltOutputCandidate],
) -> Result<Vec<PathBuf>, RunError> {
    let mut rebuilt_output_paths = Vec::with_capacity(plan.expected_outputs.len());
    for expected_output in &plan.expected_outputs {
        let candidate = select_rebuilt_output_candidate(expected_output, candidates)?;
        let staged_output_path = plan
            .scratch_layout
            .rebuilt_outputs_dir
            .join(parse_request_relative_path(&expected_output.published_name, "published output")?);
        copy_rebuilt_output(&candidate.path, &staged_output_path)?;
        rebuilt_output_paths.push(staged_output_path);
    }
    assert_eq!(rebuilt_output_paths.len(), plan.expected_outputs.len());
    Ok(rebuilt_output_paths)
}

fn select_rebuilt_output_candidate<'a>(
    expected_output: &ExpectedRebuiltOutput,
    candidates: &'a [RebuiltOutputCandidate],
) -> Result<&'a RebuiltOutputCandidate, RunError> {
    for candidate in candidates {
        if candidate.digest_blake3 == expected_output.expected_digest_blake3 {
            return Ok(candidate);
        }
    }
    Err(RunError::Build(format!(
        "witness rebuild workflow did not produce proof artifact matching {} digest {}; available proof digests: {}",
        expected_output.published_name,
        expected_output.expected_digest_blake3,
        format_candidate_digest_summary(candidates)
    )))
}

fn format_candidate_digest_summary(candidates: &[RebuiltOutputCandidate]) -> String {
    let mut parts = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        parts.push(format!("{}={}", candidate.role, candidate.digest_blake3));
    }
    parts.join(", ")
}

#[cfg(test)]
fn resolve_proof_bundle_artifact_path(proof_bundle_dir: &Path, recorded_path: &str) -> Result<PathBuf, RunError> {
    resolve_contained_proof_bundle_artifact_path(proof_bundle_dir, recorded_path, "proof manifest stage2 output_binary")
}

fn resolve_labeled_proof_bundle_artifact_path(
    proof_bundle_dir: &Path,
    recorded_path: &str,
    label: &str,
) -> Result<PathBuf, RunError> {
    validate_non_empty_recorded_path(recorded_path, label)?;
    let path = PathBuf::from(recorded_path);
    if path.is_absolute() {
        return Ok(path);
    }
    reject_escaping_relative_path(&path, recorded_path, label)?;
    Ok(proof_bundle_dir.join(path))
}

fn resolve_contained_proof_bundle_artifact_path(
    proof_bundle_dir: &Path,
    recorded_path: &str,
    label: &str,
) -> Result<PathBuf, RunError> {
    validate_non_empty_recorded_path(recorded_path, label)?;
    let path = PathBuf::from(recorded_path);
    if path.is_absolute() {
        return reject_absolute_proof_bundle_path(recorded_path, label);
    }
    reject_escaping_relative_path(&path, recorded_path, label)?;
    Ok(proof_bundle_dir.join(path))
}

fn validate_non_empty_recorded_path(recorded_path: &str, label: &str) -> Result<(), RunError> {
    if recorded_path.trim().is_empty() {
        return Err(RunError::Build(format!("{label} is empty")));
    }
    Ok(())
}

fn reject_absolute_proof_bundle_path(recorded_path: &str, label: &str) -> Result<PathBuf, RunError> {
    Err(RunError::Build(format!("{label} must stay inside proof bundle: {recorded_path}")))
}

fn reject_escaping_relative_path(path: &Path, recorded_path: &str, label: &str) -> Result<(), RunError> {
    for component in path.components() {
        if matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_)) {
            return Err(RunError::Build(format!("{label} must stay inside proof bundle: {recorded_path}")));
        }
    }
    Ok(())
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
    diagnostics: Option<WitnessRebuildDiagnostics>,
    source_acquisition_error: Option<&str>,
) -> Result<WitnessRebuildAuditMeta, RunError> {
    let rebuilt_outputs = build_audit_outputs(&plan.expected_outputs, rebuilt_output_paths)?;
    let launched_command = launched_workflow_command(workflow_driver_path, &plan.proof_mode)?;
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
        provider_fixed_point_proof_dir: plan.scratch_layout.provider_fixed_point_proof_dir.display().to_string(),
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
        diagnostics,
        source_acquisition: source_acquisition_audit(plan, source_acquisition_error),
    })
}

fn source_acquisition_audit(
    plan: &WitnessRebuildPlan,
    source_acquisition_error: Option<&str>,
) -> Option<WitnessSourceAcquisitionAudit> {
    let status = if source_acquisition_error.is_some() {
        audit_status_failed()
    } else {
        SOURCE_ACQUISITION_STATUS_VERIFIED
    };
    let Some(source_acquisition) = plan.source_acquisition.as_ref() else {
        return Some(WitnessSourceAcquisitionAudit {
            mode: SOURCE_ACQUISITION_MODE_COPIED.to_string(),
            url: None,
            digest_blake3: None,
            fetched_path: Some(plan.request_source_archive_path.display().to_string()),
            commit: None,
            reference: None,
            tag: None,
            archive_profile: None,
            archive_version: None,
            status: status.to_string(),
            error: source_acquisition_error.map(ToOwned::to_owned),
        });
    };
    let mode = if plan.require_git_source {
        SOURCE_ACQUISITION_MODE_GIT
    } else if plan.require_independent_source {
        SOURCE_ACQUISITION_MODE_EXTERNAL_ARCHIVE
    } else {
        SOURCE_ACQUISITION_MODE_COPIED
    };
    let fetched_path = if plan.require_git_source || plan.require_independent_source {
        plan.scratch_layout.source_acquisition_archive_path.display().to_string()
    } else {
        plan.request_source_archive_path.display().to_string()
    };
    Some(WitnessSourceAcquisitionAudit {
        mode: mode.to_string(),
        url: Some(source_acquisition.url.clone()),
        digest_blake3: Some(source_acquisition.digest_blake3.clone()),
        fetched_path: Some(fetched_path),
        commit: source_acquisition.commit.clone(),
        reference: source_acquisition.reference.clone(),
        tag: source_acquisition.tag.clone(),
        archive_profile: source_acquisition.archive_profile.clone(),
        archive_version: source_acquisition.archive_version.clone(),
        status: status.to_string(),
        error: source_acquisition_error.map(ToOwned::to_owned),
    })
}

fn launched_workflow_command(workflow_driver_path: &Path, proof_mode: &str) -> Result<Vec<String>, RunError> {
    let mut command = vec![workflow_driver_path.display().to_string()];
    for arg in workflow_args_for_proof_mode(proof_mode)? {
        command.push((*arg).to_string());
    }
    Ok(command)
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

fn build_failure_diagnostics(plan: &WitnessRebuildPlan, failure_message: &str) -> WitnessRebuildDiagnostics {
    let path_leak_scan_excerpt = collect_path_leak_scan_excerpt(plan, failure_message);
    let proof_manifest = load_optional_proof_manifest(&plan.scratch_layout.proof_bundle_dir);
    let bootstrap_divergence = bootstrap_divergence_diagnostic(proof_manifest.as_ref(), &path_leak_scan_excerpt);
    WitnessRebuildDiagnostics {
        expected_outputs: expected_output_diagnostics(&plan.expected_outputs),
        available_proof_digests: collect_available_proof_digest_diagnostics(plan),
        provider_proof_status: provider_proof_status_diagnostic(&plan.scratch_layout.provider_fixed_point_proof_dir),
        self_hosting_fixed_point_status: self_hosting_fixed_point_status_diagnostic(
            &plan.scratch_layout.proof_bundle_dir,
        ),
        bootstrap_divergence,
        path_leak_scan_excerpt,
    }
}

fn expected_output_diagnostics(expected_outputs: &[ExpectedRebuiltOutput]) -> Vec<WitnessExpectedOutputDiagnostic> {
    let mut diagnostics = Vec::with_capacity(expected_outputs.len());
    for expected in expected_outputs {
        diagnostics.push(WitnessExpectedOutputDiagnostic {
            published_name: expected.published_name.clone(),
            expected_digest_blake3: expected.expected_digest_blake3.clone(),
        });
    }
    diagnostics
}

fn collect_available_proof_digest_diagnostics(plan: &WitnessRebuildPlan) -> Vec<WitnessAvailableProofDigest> {
    let mut candidates = Vec::with_capacity(PROOF_BINARY_CANDIDATE_LIMIT);
    if let Ok(proof_manifest) = load_proof_bundle_manifest(&plan.scratch_layout.proof_bundle_dir) {
        if let Ok(mut proof_candidates) =
            collect_rebuilt_output_candidates(&plan.scratch_layout.proof_bundle_dir, &proof_manifest)
        {
            candidates.append(&mut proof_candidates);
        }
    }
    let mut provider_candidates = Vec::with_capacity(PROOF_BINARY_CANDIDATE_LIMIT);
    if collect_provider_fixed_point_candidates(
        &plan.scratch_layout.provider_fixed_point_proof_dir,
        &mut provider_candidates,
    )
    .is_ok()
    {
        candidates.append(&mut provider_candidates);
    }
    candidates.truncate(PROOF_BINARY_CANDIDATE_LIMIT);
    candidates
        .into_iter()
        .map(|candidate| WitnessAvailableProofDigest {
            role: candidate.role.to_string(),
            path: candidate.path.display().to_string(),
            digest_blake3: candidate.digest_blake3,
        })
        .collect()
}

fn provider_proof_status_diagnostic(provider_proof_dir: &Path) -> String {
    if !provider_proof_dir.exists() {
        return "absent".to_string();
    }
    if !provider_proof_dir.is_dir() {
        return format!("not-directory:{}", provider_proof_dir.display());
    }
    let verification = crate::cargo_free_self_build::verify_provider_fixed_point_proof_bundle(provider_proof_dir);
    if verification.valid {
        let digest = verification.stage_binary_digest_blake3.unwrap_or_else(|| "missing-stage-digest".to_string());
        return format!("valid:{digest}");
    }
    format!("invalid:{}", verification.blockers.join("; "))
}

fn self_hosting_fixed_point_status_diagnostic(proof_bundle_dir: &Path) -> String {
    let manifest_path = proof_bundle_dir.join(PROOF_MANIFEST_FILE_NAME);
    let Ok(bytes) = std::fs::read(&manifest_path) else {
        return format!("manifest-unreadable:{}", manifest_path.display());
    };
    let Ok(value) = serde_json::from_slice::<Value>(&bytes) else {
        return format!("manifest-invalid-json:{}", manifest_path.display());
    };
    if let Some(stage1_equals_stage2) = value.pointer(BOOTSTRAP_STAGE1_EQUALS_STAGE2_POINTER).and_then(Value::as_bool) {
        return format!("stage1_equals_stage2={stage1_equals_stage2}");
    }
    if let Some(stage2_digest) = value.pointer(BOOTSTRAP_STAGE2_DIGEST_POINTER).and_then(Value::as_str) {
        return format!("fixed-point-status-missing;stage2_digest={stage2_digest}");
    }
    "fixed-point-status-missing".to_string()
}

fn load_optional_proof_manifest(proof_bundle_dir: &Path) -> Option<Value> {
    let manifest_path = proof_bundle_dir.join(PROOF_MANIFEST_FILE_NAME);
    let bytes = std::fs::read(&manifest_path).ok()?;
    serde_json::from_slice::<Value>(&bytes).ok()
}

fn bootstrap_divergence_diagnostic(
    proof_manifest: Option<&Value>,
    path_leak_scan_summary: &[String],
) -> WitnessBootstrapDivergenceDiagnostic {
    if let Some(explicit) = explicit_bootstrap_divergence(proof_manifest, path_leak_scan_summary) {
        return explicit;
    }
    if let Some(derived) = derived_bootstrap_divergence(proof_manifest, path_leak_scan_summary) {
        return derived;
    }
    WitnessBootstrapDivergenceDiagnostic {
        status: BOOTSTRAP_DIVERGENCE_ABSENT.to_string(),
        root: None,
        stage1_digest_blake3: None,
        stage2_digest_blake3: None,
        path_leak_scan_summary: bounded_path_leak_scan_summary(path_leak_scan_summary),
    }
}

fn explicit_bootstrap_divergence(
    proof_manifest: Option<&Value>,
    path_leak_scan_summary: &[String],
) -> Option<WitnessBootstrapDivergenceDiagnostic> {
    let manifest = proof_manifest?;
    let root = manifest.pointer(BOOTSTRAP_DIVERGENCE_ROOT_POINTER).and_then(Value::as_str)?;
    if root.is_empty() {
        return None;
    }
    Some(WitnessBootstrapDivergenceDiagnostic {
        status: BOOTSTRAP_DIVERGENCE_DIVERGED.to_string(),
        root: Some(root.to_string()),
        stage1_digest_blake3: manifest
            .pointer(BOOTSTRAP_DIVERGENCE_STAGE1_DIGEST_POINTER)
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        stage2_digest_blake3: manifest
            .pointer(BOOTSTRAP_DIVERGENCE_STAGE2_DIGEST_POINTER)
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        path_leak_scan_summary: bounded_path_leak_scan_summary(path_leak_scan_summary),
    })
}

fn derived_bootstrap_divergence(
    proof_manifest: Option<&Value>,
    path_leak_scan_summary: &[String],
) -> Option<WitnessBootstrapDivergenceDiagnostic> {
    let manifest = proof_manifest?;
    if manifest.pointer(BOOTSTRAP_BWRAP_EQUALS_POINTER).and_then(Value::as_bool) == Some(false) {
        return Some(bootstrap_tool_divergence(
            BOOTSTRAP_DIVERGENCE_BWRAP_ROOT,
            manifest,
            BOOTSTRAP_STAGE0_BWRAP_DIGEST_POINTER,
            BOOTSTRAP_STAGE2_BWRAP_DIGEST_POINTER,
            path_leak_scan_summary,
        ));
    }
    if manifest.pointer(BOOTSTRAP_BUSYBOX_EQUALS_POINTER).and_then(Value::as_bool) == Some(false) {
        return Some(bootstrap_tool_divergence(
            BOOTSTRAP_DIVERGENCE_BUSYBOX_ROOT,
            manifest,
            BOOTSTRAP_STAGE0_BUSYBOX_DIGEST_POINTER,
            BOOTSTRAP_STAGE2_BUSYBOX_DIGEST_POINTER,
            path_leak_scan_summary,
        ));
    }
    if path_leak_scan_summary.iter().any(|line| line.contains(BOOTSTRAP_DIVERGENCE_GCC_ROOT)) {
        return Some(bootstrap_stage_divergence(BOOTSTRAP_DIVERGENCE_GCC_ROOT, manifest, path_leak_scan_summary));
    }
    if manifest.pointer(BOOTSTRAP_STAGE1_EQUALS_STAGE2_POINTER).and_then(Value::as_bool) == Some(false) {
        return Some(bootstrap_stage_divergence(
            BOOTSTRAP_DIVERGENCE_STAGE_MANTLE_ROOT,
            manifest,
            path_leak_scan_summary,
        ));
    }
    if proof_manifest_reports_convergence(manifest) {
        return Some(WitnessBootstrapDivergenceDiagnostic {
            status: BOOTSTRAP_DIVERGENCE_CONVERGED.to_string(),
            root: None,
            stage1_digest_blake3: manifest
                .pointer(BOOTSTRAP_STAGE1_DIGEST_POINTER)
                .and_then(Value::as_str)
                .map(ToOwned::to_owned),
            stage2_digest_blake3: manifest
                .pointer(BOOTSTRAP_STAGE2_DIGEST_POINTER)
                .and_then(Value::as_str)
                .map(ToOwned::to_owned),
            path_leak_scan_summary: bounded_path_leak_scan_summary(path_leak_scan_summary),
        });
    }
    None
}

fn bootstrap_tool_divergence(
    root: &str,
    manifest: &Value,
    stage1_digest_pointer: &str,
    stage2_digest_pointer: &str,
    path_leak_scan_summary: &[String],
) -> WitnessBootstrapDivergenceDiagnostic {
    WitnessBootstrapDivergenceDiagnostic {
        status: BOOTSTRAP_DIVERGENCE_DIVERGED.to_string(),
        root: Some(root.to_string()),
        stage1_digest_blake3: manifest.pointer(stage1_digest_pointer).and_then(Value::as_str).map(ToOwned::to_owned),
        stage2_digest_blake3: manifest.pointer(stage2_digest_pointer).and_then(Value::as_str).map(ToOwned::to_owned),
        path_leak_scan_summary: bounded_path_leak_scan_summary(path_leak_scan_summary),
    }
}

fn bootstrap_stage_divergence(
    root: &str,
    manifest: &Value,
    path_leak_scan_summary: &[String],
) -> WitnessBootstrapDivergenceDiagnostic {
    let resolved_root = if root.is_empty() {
        BOOTSTRAP_DIVERGENCE_UNKNOWN_ROOT
    } else {
        root
    };
    WitnessBootstrapDivergenceDiagnostic {
        status: BOOTSTRAP_DIVERGENCE_DIVERGED.to_string(),
        root: Some(resolved_root.to_string()),
        stage1_digest_blake3: manifest
            .pointer(BOOTSTRAP_STAGE1_DIGEST_POINTER)
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        stage2_digest_blake3: manifest
            .pointer(BOOTSTRAP_STAGE2_DIGEST_POINTER)
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        path_leak_scan_summary: bounded_path_leak_scan_summary(path_leak_scan_summary),
    }
}

fn proof_manifest_reports_convergence(manifest: &Value) -> bool {
    manifest.pointer(BOOTSTRAP_STAGE1_EQUALS_STAGE2_POINTER).and_then(Value::as_bool) == Some(true)
        && manifest.pointer(BOOTSTRAP_BWRAP_EQUALS_POINTER).and_then(Value::as_bool) == Some(true)
        && manifest.pointer(BOOTSTRAP_BUSYBOX_EQUALS_POINTER).and_then(Value::as_bool) == Some(true)
}

fn bounded_path_leak_scan_summary(path_leak_scan_summary: &[String]) -> Vec<String> {
    let mut summary = path_leak_scan_summary.to_vec();
    summary.truncate(DIAGNOSTIC_EXCERPT_LIMIT);
    summary
}

fn collect_path_leak_scan_excerpt(plan: &WitnessRebuildPlan, failure_message: &str) -> Vec<String> {
    let mut excerpts = Vec::with_capacity(DIAGNOSTIC_EXCERPT_LIMIT);
    collect_matching_excerpt_lines(failure_message, &mut excerpts);
    collect_matching_log_excerpt(&plan.scratch_layout.stderr_log_path, &mut excerpts);
    collect_matching_log_excerpt(&plan.scratch_layout.stdout_log_path, &mut excerpts);
    if excerpts.is_empty() {
        excerpts.push("no path-leak excerpts found in workflow logs".to_string());
    }
    excerpts.truncate(DIAGNOSTIC_EXCERPT_LIMIT);
    excerpts
}

fn collect_matching_log_excerpt(path: &Path, excerpts: &mut Vec<String>) {
    if excerpts.len() >= DIAGNOSTIC_EXCERPT_LIMIT {
        return;
    }
    let Ok(text) = std::fs::read_to_string(path) else {
        return;
    };
    collect_matching_excerpt_lines(&text, excerpts);
}

fn collect_matching_excerpt_lines(text: &str, excerpts: &mut Vec<String>) {
    for line in text.lines() {
        if excerpts.len() >= DIAGNOSTIC_EXCERPT_LIMIT {
            return;
        }
        if DIAGNOSTIC_MARKERS.iter().any(|marker| line.contains(marker)) {
            excerpts.push(line.to_string());
        }
    }
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

    const TEST_BLAKE3_HEX_LENGTH_CHARS: usize = 64;
    const TEST_PROVIDER_FIXED_POINT_STAGE_UNIT_COUNT: u32 = 2;
    const TEST_PROVIDER_TOOLCHAIN_MEMBER_COUNT: u32 = 17;
    const TEST_PROVIDER_ARTIFACT_COUNT: u32 = 6;
    const TEST_PROVIDER_SOURCE_COUNT: u32 = 2;
    const TEST_PROVIDER_RECEIPT_COUNT: u32 = 1;
    const TEST_REQUEST_LAYOUT_VERSION: u32 = 1;
    const TEST_SOURCE_ARCHIVE_FILE_MODE: u32 = 0o644;
    const TEST_SHA256_HEX_LENGTH_CHARS: usize = 64;

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
    fn resolve_proof_bundle_artifact_path_anchors_relative_paths() {
        let proof_bundle = PathBuf::from("/tmp/proof-bundle");
        let path = resolve_proof_bundle_artifact_path(&proof_bundle, "binaries/stage2-mantle").unwrap();
        assert_eq!(path, proof_bundle.join("binaries/stage2-mantle"));
    }

    #[test]
    fn resolve_proof_bundle_artifact_path_rejects_parent_escape() {
        let proof_bundle = PathBuf::from("/tmp/proof-bundle");
        let err = resolve_proof_bundle_artifact_path(&proof_bundle, "../stage2-mantle").unwrap_err();
        assert!(err.message().contains("must stay inside proof bundle"));
    }

    #[test]
    fn collect_rebuilt_output_paths_maps_two_expected_outputs_by_digest() {
        let temp = tempfile::tempdir().unwrap();
        let proof_bundle_dir = temp.path().join(SCRATCH_PROOF_BUNDLE_DIR_NAME);
        std::fs::create_dir_all(&proof_bundle_dir).unwrap();
        let stage1_digest = write_proof_binary(&proof_bundle_dir, "binaries/stage1-mantle", b"provider-stage");
        let stage2_digest = write_proof_binary(&proof_bundle_dir, "binaries/stage2-mantle", b"self-host-stage");
        write_proof_manifest(
            &proof_bundle_dir,
            serde_json::json!({
                "binaries": {
                    "stage1": {
                        "path": "binaries/stage1-mantle",
                        "digest_blake3": stage1_digest.clone(),
                    },
                    "stage2": {
                        "path": "binaries/stage2-mantle",
                        "digest_blake3": stage2_digest.clone(),
                    }
                },
                "stage2": {
                    "report": {
                        "output_binary": "binaries/stage2-mantle"
                    }
                }
            }),
        );
        let expected_outputs = vec![
            expected_output("binaries/01-mantle", &stage1_digest),
            expected_output("binaries/02-stage2-mantle", &stage2_digest),
        ];
        let plan = witness_rebuild_test_plan(temp.path(), expected_outputs);

        let rebuilt_output_paths = collect_rebuilt_output_paths(&plan).unwrap();

        let expected_stage1_path = temp.path().join(SCRATCH_REBUILT_OUTPUTS_DIR_NAME).join("binaries/01-mantle");
        let expected_stage2_path = temp.path().join(SCRATCH_REBUILT_OUTPUTS_DIR_NAME).join("binaries/02-stage2-mantle");
        assert_eq!(rebuilt_output_paths, vec![expected_stage1_path.clone(), expected_stage2_path.clone()]);
        assert_eq!(compute_path_blake3_digest(&expected_stage1_path).unwrap(), stage1_digest);
        assert_eq!(compute_path_blake3_digest(&expected_stage2_path).unwrap(), stage2_digest);
        validate_rebuilt_output_digests(&plan.expected_outputs, &rebuilt_output_paths).unwrap();
    }

    #[test]
    fn collect_rebuilt_output_paths_preserves_one_output_legacy_manifest() {
        let temp = tempfile::tempdir().unwrap();
        let proof_bundle_dir = temp.path().join(SCRATCH_PROOF_BUNDLE_DIR_NAME);
        std::fs::create_dir_all(&proof_bundle_dir).unwrap();
        let stage2_digest = write_proof_binary(&proof_bundle_dir, "binaries/stage2-mantle", b"legacy-stage");
        write_proof_manifest(
            &proof_bundle_dir,
            serde_json::json!({
                "stage2": {
                    "report": {
                        "output_binary": "binaries/stage2-mantle"
                    }
                }
            }),
        );
        let plan =
            witness_rebuild_test_plan(temp.path(), vec![expected_output("binaries/01-stage2-mantle", &stage2_digest)]);

        let rebuilt_output_paths = collect_rebuilt_output_paths(&plan).unwrap();

        let expected_path = temp.path().join(SCRATCH_REBUILT_OUTPUTS_DIR_NAME).join("binaries/01-stage2-mantle");
        assert_eq!(rebuilt_output_paths, vec![expected_path.clone()]);
        assert_eq!(compute_path_blake3_digest(&expected_path).unwrap(), stage2_digest);
        validate_rebuilt_output_digests(&plan.expected_outputs, &rebuilt_output_paths).unwrap();
    }

    #[test]
    fn collect_rebuilt_output_paths_rejects_missing_expected_digest() {
        let temp = tempfile::tempdir().unwrap();
        let proof_bundle_dir = temp.path().join(SCRATCH_PROOF_BUNDLE_DIR_NAME);
        std::fs::create_dir_all(&proof_bundle_dir).unwrap();
        let stage2_digest = write_proof_binary(&proof_bundle_dir, "binaries/stage2-mantle", b"other-stage");
        write_proof_manifest(
            &proof_bundle_dir,
            serde_json::json!({
                "binaries": {
                    "stage2": {
                        "path": "binaries/stage2-mantle",
                        "digest_blake3": stage2_digest.clone(),
                    }
                },
                "stage2": {
                    "report": {
                        "output_binary": "binaries/stage2-mantle"
                    }
                }
            }),
        );
        let missing_digest = "0".repeat(TEST_BLAKE3_HEX_LENGTH_CHARS);
        let plan =
            witness_rebuild_test_plan(temp.path(), vec![expected_output("binaries/missing-mantle", &missing_digest)]);

        let err = collect_rebuilt_output_paths(&plan).unwrap_err();

        assert!(err.message().contains("did not produce proof artifact matching binaries/missing-mantle"));
        assert!(err.message().contains(&missing_digest));
        assert!(err.message().contains(&stage2_digest));
    }

    #[test]
    fn failure_audit_reports_digest_mismatch_diagnostics() {
        let temp = tempfile::tempdir().unwrap();
        let proof_bundle_dir = temp.path().join(SCRATCH_PROOF_BUNDLE_DIR_NAME);
        std::fs::create_dir_all(&proof_bundle_dir).unwrap();
        let stage2_digest = write_proof_binary(&proof_bundle_dir, "binaries/stage2-mantle", b"other-stage");
        write_proof_manifest(
            &proof_bundle_dir,
            serde_json::json!({
                "binaries": {
                    "stage2": {
                        "path": "binaries/stage2-mantle",
                        "digest_blake3": stage2_digest.clone(),
                    }
                },
                "stage2": {
                    "report": {
                        "output_binary": "binaries/stage2-mantle"
                    }
                }
            }),
        );
        let missing_digest = "0".repeat(TEST_BLAKE3_HEX_LENGTH_CHARS);
        let plan =
            witness_rebuild_test_plan(temp.path(), vec![expected_output("binaries/02-stage2-mantle", &missing_digest)]);
        let failure_message = format!(
            "rebuilt output digest mismatch for binaries/02-stage2-mantle: expected {missing_digest}, got {stage2_digest}"
        );

        let meta =
            build_failure_audit_meta(&plan, Path::new("/tmp/workflow-driver"), &[], 0, 1, &failure_message).unwrap();
        let diagnostics = meta.diagnostics.expect("failure audit should carry diagnostics");

        assert_eq!(diagnostics.expected_outputs[0].expected_digest_blake3, missing_digest);
        assert!(
            diagnostics
                .available_proof_digests
                .iter()
                .any(|candidate| candidate.role == "binaries.stage2" && candidate.digest_blake3 == stage2_digest)
        );
        assert_eq!(diagnostics.provider_proof_status, "absent");
        assert!(diagnostics.self_hosting_fixed_point_status.contains("stage2_digest="));
        assert_eq!(diagnostics.bootstrap_divergence.status, BOOTSTRAP_DIVERGENCE_ABSENT);
        assert!(diagnostics.bootstrap_divergence.root.is_none());
        assert!(diagnostics.path_leak_scan_excerpt.iter().any(|excerpt| excerpt.contains("digest mismatch")));
    }

    #[test]
    fn bootstrap_divergence_diagnostic_reports_convergence() {
        let digest = "a".repeat(TEST_BLAKE3_HEX_LENGTH_CHARS);
        let manifest = serde_json::json!({
            "fixed_point": {
                "stage1_equals_stage2": true,
                "stage0_bwrap_equals_stage2_bwrap": true,
                "stage0_busybox_equals_stage2_busybox": true,
            },
            "binaries": {
                "stage1": { "digest_blake3": digest.clone() },
                "stage2": { "digest_blake3": digest.clone() },
            }
        });
        let path_leak_summary = vec!["no path-leak excerpts found in workflow logs".to_string()];

        let diagnostic = bootstrap_divergence_diagnostic(Some(&manifest), &path_leak_summary);

        assert_eq!(diagnostic.status, BOOTSTRAP_DIVERGENCE_CONVERGED);
        assert!(diagnostic.root.is_none());
        assert_eq!(diagnostic.stage1_digest_blake3.as_deref(), Some(digest.as_str()));
        assert_eq!(diagnostic.stage2_digest_blake3.as_deref(), Some(digest.as_str()));
        assert_eq!(diagnostic.path_leak_scan_summary, path_leak_summary);
    }

    #[test]
    fn failure_audit_reports_gcc_bootstrap_divergence_root() {
        let temp = tempfile::tempdir().unwrap();
        let proof_bundle_dir = temp.path().join(SCRATCH_PROOF_BUNDLE_DIR_NAME);
        std::fs::create_dir_all(&proof_bundle_dir).unwrap();
        let stage1_digest = "1".repeat(TEST_BLAKE3_HEX_LENGTH_CHARS);
        let stage2_digest = "2".repeat(TEST_BLAKE3_HEX_LENGTH_CHARS);
        write_proof_manifest(
            &proof_bundle_dir,
            serde_json::json!({
                "fixed_point": {
                    "stage1_equals_stage2": false,
                    "stage0_bwrap_equals_stage2_bwrap": true,
                    "stage0_busybox_equals_stage2_busybox": true,
                },
                "binaries": {
                    "stage1": { "digest_blake3": stage1_digest.clone() },
                    "stage2": {
                        "path": "binaries/stage2-mantle",
                        "digest_blake3": stage2_digest.clone(),
                    }
                },
                "stage2": {
                    "report": {
                        "output_binary": "binaries/stage2-mantle"
                    }
                }
            }),
        );
        let stderr_path = temp.path().join(SCRATCH_AUDIT_DIR_NAME).join(AUDIT_STDERR_FILE_NAME);
        std::fs::create_dir_all(stderr_path.parent().unwrap()).unwrap();
        std::fs::write(&stderr_path, "first divergent bootstrap output: gcc.drv output digest mismatch\n").unwrap();
        let missing_digest = "0".repeat(TEST_BLAKE3_HEX_LENGTH_CHARS);
        let plan =
            witness_rebuild_test_plan(temp.path(), vec![expected_output("binaries/02-stage2-mantle", &missing_digest)]);
        let failure_message = format!(
            "rebuilt output digest mismatch for binaries/02-stage2-mantle: expected {missing_digest}, got {stage2_digest}"
        );

        let meta =
            build_failure_audit_meta(&plan, Path::new("/tmp/workflow-driver"), &[], 0, 1, &failure_message).unwrap();
        let diagnostics = meta.diagnostics.expect("failure audit should carry diagnostics");

        assert_eq!(meta.status, audit_status_failed());
        assert!(meta.witness_attestation_path.is_none());
        assert_eq!(diagnostics.bootstrap_divergence.status, BOOTSTRAP_DIVERGENCE_DIVERGED);
        assert_eq!(diagnostics.bootstrap_divergence.root.as_deref(), Some(BOOTSTRAP_DIVERGENCE_GCC_ROOT));
        assert_eq!(diagnostics.bootstrap_divergence.stage1_digest_blake3.as_deref(), Some(stage1_digest.as_str()));
        assert_eq!(diagnostics.bootstrap_divergence.stage2_digest_blake3.as_deref(), Some(stage2_digest.as_str()));
        assert!(
            diagnostics
                .bootstrap_divergence
                .path_leak_scan_summary
                .iter()
                .any(|excerpt| excerpt.contains(BOOTSTRAP_DIVERGENCE_GCC_ROOT))
        );
    }

    #[test]
    fn validate_rebuilt_output_digests_rejects_stripped_equivalent_but_different_binary() {
        let temp = tempfile::tempdir().unwrap();
        let exact_bytes = b"mantle-binary";
        let stripped_equivalent_prefix = b"mantle-binary";
        let symbol_table_suffix = b"-debug-symbols";
        let expected_digest = blake3::hash(stripped_equivalent_prefix).to_hex().to_string();
        let rebuilt_path = temp.path().join("rebuilt-mantle");
        let mut rebuilt_bytes = stripped_equivalent_prefix.to_vec();
        rebuilt_bytes.extend_from_slice(symbol_table_suffix);
        std::fs::write(&rebuilt_path, rebuilt_bytes).unwrap();
        let expected_outputs = vec![expected_output("binaries/02-stage2-mantle", &expected_digest)];
        let rebuilt_paths = vec![rebuilt_path];

        let err = validate_rebuilt_output_digests(&expected_outputs, &rebuilt_paths).unwrap_err();

        assert!(err.message().contains("rebuilt output digest mismatch"));
        assert!(err.message().contains(&expected_digest));
        assert!(err.message().contains("binaries/02-stage2-mantle"));
        assert_eq!(exact_bytes, stripped_equivalent_prefix);
    }

    #[test]
    fn collect_rebuilt_output_paths_uses_provider_fixed_point_candidate_by_digest() {
        let temp = tempfile::tempdir().unwrap();
        let proof_bundle_dir = temp.path().join(SCRATCH_PROOF_BUNDLE_DIR_NAME);
        std::fs::create_dir_all(&proof_bundle_dir).unwrap();
        let self_hosted_digest = write_proof_binary(&proof_bundle_dir, "binaries/stage2-mantle", b"self-host-stage");
        write_proof_manifest(
            &proof_bundle_dir,
            serde_json::json!({
                "binaries": {
                    "stage2": {
                        "path": "binaries/stage2-mantle",
                        "digest_blake3": self_hosted_digest.clone(),
                    }
                },
                "stage2": {
                    "report": {
                        "output_binary": "binaries/stage2-mantle"
                    }
                }
            }),
        );
        let provider_proof_dir = temp.path().join(SCRATCH_PROVIDER_FIXED_POINT_PROOF_DIR_NAME);
        let provider_digest = write_provider_fixed_point_proof_bundle(&provider_proof_dir, b"provider-stage", None);
        let plan = witness_rebuild_test_plan(temp.path(), vec![
            expected_output("binaries/01-mantle", &provider_digest),
            expected_output("binaries/02-stage2-mantle", &self_hosted_digest),
        ]);

        let rebuilt_output_paths = collect_rebuilt_output_paths(&plan).unwrap();

        let provider_output_path = temp.path().join(SCRATCH_REBUILT_OUTPUTS_DIR_NAME).join("binaries/01-mantle");
        let self_hosted_output_path =
            temp.path().join(SCRATCH_REBUILT_OUTPUTS_DIR_NAME).join("binaries/02-stage2-mantle");
        assert_eq!(rebuilt_output_paths, vec![provider_output_path.clone(), self_hosted_output_path.clone()]);
        assert_eq!(compute_path_blake3_digest(&provider_output_path).unwrap(), provider_digest);
        assert_eq!(compute_path_blake3_digest(&self_hosted_output_path).unwrap(), self_hosted_digest);
        validate_rebuilt_output_digests(&plan.expected_outputs, &rebuilt_output_paths).unwrap();
    }

    #[test]
    fn collect_rebuilt_output_paths_rejects_invalid_provider_fixed_point_proof() {
        let temp = tempfile::tempdir().unwrap();
        let proof_bundle_dir = temp.path().join(SCRATCH_PROOF_BUNDLE_DIR_NAME);
        std::fs::create_dir_all(&proof_bundle_dir).unwrap();
        let self_hosted_digest = write_proof_binary(&proof_bundle_dir, "binaries/stage2-mantle", b"self-host-stage");
        write_proof_manifest(
            &proof_bundle_dir,
            serde_json::json!({
                "binaries": {
                    "stage2": {
                        "path": "binaries/stage2-mantle",
                        "digest_blake3": self_hosted_digest,
                    }
                },
                "stage2": {
                    "report": {
                        "output_binary": "binaries/stage2-mantle"
                    }
                }
            }),
        );
        let provider_proof_dir = temp.path().join(SCRATCH_PROVIDER_FIXED_POINT_PROOF_DIR_NAME);
        std::fs::create_dir_all(&provider_proof_dir).unwrap();
        std::fs::write(provider_proof_dir.join("meta.json"), br#"{}"#).unwrap();
        let plan = witness_rebuild_test_plan(temp.path(), vec![expected_output(
            "binaries/01-mantle",
            &"1".repeat(TEST_BLAKE3_HEX_LENGTH_CHARS),
        )]);

        let err = collect_rebuilt_output_paths(&plan).unwrap_err();

        assert!(err.message().contains("provider fixed-point proof is invalid"));
    }

    #[test]
    fn collect_rebuilt_output_paths_rejects_provider_fixed_point_stage_escape() {
        let temp = tempfile::tempdir().unwrap();
        let proof_bundle_dir = temp.path().join(SCRATCH_PROOF_BUNDLE_DIR_NAME);
        std::fs::create_dir_all(&proof_bundle_dir).unwrap();
        let self_hosted_digest = write_proof_binary(&proof_bundle_dir, "binaries/stage2-mantle", b"self-host-stage");
        write_proof_manifest(
            &proof_bundle_dir,
            serde_json::json!({
                "binaries": {
                    "stage2": {
                        "path": "binaries/stage2-mantle",
                        "digest_blake3": self_hosted_digest,
                    }
                },
                "stage2": {
                    "report": {
                        "output_binary": "binaries/stage2-mantle"
                    }
                }
            }),
        );
        let outside_binary = temp.path().join("outside-provider-mantle");
        std::fs::write(&outside_binary, b"provider-stage").unwrap();
        let provider_proof_dir = temp.path().join(SCRATCH_PROVIDER_FIXED_POINT_PROOF_DIR_NAME);
        let provider_digest = write_provider_fixed_point_proof_bundle(
            &provider_proof_dir,
            b"provider-stage",
            Some(outside_binary.as_path()),
        );
        let plan =
            witness_rebuild_test_plan(temp.path(), vec![expected_output("binaries/01-mantle", &provider_digest)]);

        let err = collect_rebuilt_output_paths(&plan).unwrap_err();

        assert!(
            err.message()
                .contains("provider-fixed-point.stage2 must stay inside provider fixed-point proof bundle")
        );
    }

    #[test]
    fn collect_rebuilt_output_paths_rejects_escaping_proof_binary_path() {
        let temp = tempfile::tempdir().unwrap();
        let proof_bundle_dir = temp.path().join(SCRATCH_PROOF_BUNDLE_DIR_NAME);
        std::fs::create_dir_all(&proof_bundle_dir).unwrap();
        write_proof_manifest(
            &proof_bundle_dir,
            serde_json::json!({
                "binaries": {
                    "stage1": {
                        "path": "../escape",
                        "digest_blake3": "0".repeat(TEST_BLAKE3_HEX_LENGTH_CHARS),
                    }
                },
                "stage2": {
                    "report": {
                        "output_binary": "binaries/stage2-mantle"
                    }
                }
            }),
        );
        let plan = witness_rebuild_test_plan(temp.path(), vec![expected_output(
            "binaries/01-mantle",
            &"0".repeat(TEST_BLAKE3_HEX_LENGTH_CHARS),
        )]);

        let err = collect_rebuilt_output_paths(&plan).unwrap_err();

        assert!(err.message().contains("binaries.stage1 must stay inside proof bundle"));
    }

    #[test]
    fn collect_rebuilt_output_paths_rejects_absolute_proof_binary_path() {
        let temp = tempfile::tempdir().unwrap();
        let proof_bundle_dir = temp.path().join(SCRATCH_PROOF_BUNDLE_DIR_NAME);
        std::fs::create_dir_all(&proof_bundle_dir).unwrap();
        let absolute_path = temp.path().join("outside-proof-bundle");
        std::fs::write(&absolute_path, b"outside").unwrap();
        write_proof_manifest(
            &proof_bundle_dir,
            serde_json::json!({
                "binaries": {
                    "stage1": {
                        "path": absolute_path.display().to_string(),
                        "digest_blake3": "0".repeat(TEST_BLAKE3_HEX_LENGTH_CHARS),
                    }
                },
                "stage2": {
                    "report": {
                        "output_binary": "binaries/stage2-mantle"
                    }
                }
            }),
        );
        let plan = witness_rebuild_test_plan(temp.path(), vec![expected_output(
            "binaries/01-mantle",
            &"0".repeat(TEST_BLAKE3_HEX_LENGTH_CHARS),
        )]);

        let err = collect_rebuilt_output_paths(&plan).unwrap_err();

        assert!(err.message().contains("binaries.stage1 must stay inside proof bundle"));
        assert!(err.message().contains(&absolute_path.display().to_string()));
    }

    fn write_proof_binary(proof_bundle_dir: &Path, relative_path: &str, bytes: &[u8]) -> String {
        let path = proof_bundle_dir.join(relative_path);
        let parent = path.parent().unwrap();
        std::fs::create_dir_all(parent).unwrap();
        std::fs::write(&path, bytes).unwrap();
        compute_path_blake3_digest(&path).unwrap()
    }

    fn write_proof_manifest(proof_bundle_dir: &Path, value: serde_json::Value) {
        std::fs::write(proof_bundle_dir.join(PROOF_MANIFEST_FILE_NAME), serde_json::to_vec(&value).unwrap()).unwrap();
    }

    fn write_provider_fixed_point_proof_bundle(
        provider_proof_dir: &Path,
        binary_bytes: &[u8],
        stage_binary_override: Option<&Path>,
    ) -> String {
        let binary_digest = blake3::hash(binary_bytes).to_hex().to_string();
        let policy_digest = "2".repeat(TEST_BLAKE3_HEX_LENGTH_CHARS);
        let metadata_digest = "3".repeat(TEST_BLAKE3_HEX_LENGTH_CHARS);
        let stage1_binary = provider_stage_binary_path(provider_proof_dir, "stage1", stage_binary_override);
        let stage2_binary = provider_stage_binary_path(provider_proof_dir, "stage2", stage_binary_override);
        write_provider_binary(&stage1_binary, binary_bytes);
        write_provider_binary(&stage2_binary, binary_bytes);
        write_provider_stage_receipt(&provider_proof_dir.join("stage1/receipt.json"));
        write_provider_stage_receipt(&provider_proof_dir.join("stage2/receipt.json"));
        write_provider_non_claims(provider_proof_dir);
        write_provider_preflight(provider_proof_dir, &policy_digest);
        let meta = serde_json::json!({
            "schema": "mantle-cargo-free-fixed-point-proof-v1",
            "status": "success",
            "root": "/repo/mantle",
            "bundle_dir": provider_proof_dir,
            "fixed_point": true,
            "stage1": provider_stage_summary_json(provider_proof_dir, "stage1", &stage1_binary, &binary_digest, &policy_digest),
            "stage2": provider_stage_summary_json(provider_proof_dir, "stage2", &stage2_binary, &binary_digest, &policy_digest),
            "rustc_compatibility": {
                "requested_rustc": "/provider/bin/rustc",
                "stage_rustc": "/provider/bin/rustc",
                "normalization": "none",
                "wrapper": null,
                "wrapper_blake3": null
            },
            "source_built_toolchain_closure": {
                "schema": "mantle-source-built-toolchain-closure-v1",
                "status": "enforced-source-built",
                "claim": true,
                "non_claim": null,
                "manifest_path": "/tmp/toolchain-closure.json",
                "policy_digest_blake3": policy_digest,
                "member_count": TEST_PROVIDER_TOOLCHAIN_MEMBER_COUNT,
                "source_built_member_count": TEST_PROVIDER_TOOLCHAIN_MEMBER_COUNT,
                "seed_exception_count": 0
            },
            "rust_source_provider": {
                "schema": "mantle-cargo-free-rust-source-provider-binding-v1",
                "status": "validated",
                "provider_dir": "/provider",
                "metadata_path": "/provider/share/mantle-rust-provider/provider.json",
                "metadata_digest_blake3": metadata_digest,
                "policy_digest_blake3": policy_digest,
                "host_triple": "x86_64-unknown-linux-musl",
                "target_triple": "x86_64-unknown-linux-musl",
                "artifact_count": TEST_PROVIDER_ARTIFACT_COUNT,
                "source_count": TEST_PROVIDER_SOURCE_COUNT,
                "receipt_count": TEST_PROVIDER_RECEIPT_COUNT,
                "rustc_path": "/provider/bin/rustc"
            },
            "blocker": null,
            "non_claims": [
                "not-crunch-bootstrap",
                "not-release-reproducibility",
                "not-full-cargo-compatibility"
            ]
        });
        std::fs::write(provider_proof_dir.join("meta.json"), serde_json::to_vec(&meta).unwrap()).unwrap();
        binary_digest
    }

    fn provider_stage_binary_path(
        provider_proof_dir: &Path,
        stage_name: &str,
        stage_binary_override: Option<&Path>,
    ) -> PathBuf {
        stage_binary_override
            .map(Path::to_path_buf)
            .unwrap_or_else(|| provider_proof_dir.join(stage_name).join("mantle"))
    }

    fn write_provider_binary(path: &Path, binary_bytes: &[u8]) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, binary_bytes).unwrap();
    }

    fn write_provider_stage_receipt(path: &Path) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let receipt = serde_json::json!({
            "topology_execution": {
                "execution_status": "success",
                "unit_executions": [
                    { "unit_id": "unit-1", "execution_status": "success" },
                    { "unit_id": "unit-2", "execution_status": "success" }
                ]
            }
        });
        std::fs::write(path, serde_json::to_vec(&receipt).unwrap()).unwrap();
    }

    fn write_provider_non_claims(provider_proof_dir: &Path) {
        std::fs::create_dir_all(provider_proof_dir).unwrap();
        std::fs::write(
            provider_proof_dir.join("non-claims.txt"),
            b"This proof does not claim Crunch bootstrap or release reproducibility.\nThis proof does not claim full Cargo compatibility.\n",
        )
        .unwrap();
    }

    fn write_provider_preflight(provider_proof_dir: &Path, policy_digest: &str) {
        let preflight = serde_json::json!({
            "schema": "mantle-cargo-free-fixed-point-proof-v1",
            "root": "/repo/mantle",
            "bundle_dir": provider_proof_dir,
            "source_built_toolchain_closure": {
                "status": "enforced-source-built",
                "claim": true,
                "policy_digest_blake3": policy_digest
            }
        });
        std::fs::write(provider_proof_dir.join("preflight.json"), serde_json::to_vec(&preflight).unwrap()).unwrap();
    }

    fn provider_stage_summary_json(
        provider_proof_dir: &Path,
        stage_name: &str,
        binary_path: &Path,
        binary_digest: &str,
        policy_digest: &str,
    ) -> serde_json::Value {
        serde_json::json!({
            "name": stage_name,
            "dir": provider_proof_dir.join(stage_name),
            "execution_dir": provider_proof_dir.join("execution"),
            "receipt": provider_proof_dir.join(stage_name).join("receipt.json"),
            "stderr": provider_proof_dir.join(stage_name).join("stderr.txt"),
            "status": provider_proof_dir.join(stage_name).join("status.txt"),
            "status_code": 0,
            "execution_status": "success",
            "cargo_marker_absent": true,
            "success": true,
            "unit_count": TEST_PROVIDER_FIXED_POINT_STAGE_UNIT_COUNT,
            "failed_unit_count": 0,
            "binary": binary_path,
            "binary_blake3": binary_digest,
            "smoke_status_code": 0,
            "source_built_toolchain_closure_policy_digest_blake3": policy_digest,
            "blocker": null
        })
    }

    fn expected_output(published_name: &str, expected_digest_blake3: &str) -> ExpectedRebuiltOutput {
        ExpectedRebuiltOutput {
            published_name: published_name.to_string(),
            expected_digest_blake3: expected_digest_blake3.to_string(),
        }
    }

    #[cfg(unix)]
    fn empty_success_output() -> Output {
        use std::os::unix::process::ExitStatusExt;

        Output {
            status: std::process::ExitStatus::from_raw(0),
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    }

    fn write_source_archive(path: &Path, member_path: &str, member_bytes: &[u8]) -> String {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let file = std::fs::File::create(path).unwrap();
        let mut builder = tar::Builder::new(file);
        let mut header = tar::Header::new_gnu();
        header.set_path(member_path).unwrap();
        header.set_size(u64::try_from(member_bytes.len()).unwrap());
        header.set_mode(TEST_SOURCE_ARCHIVE_FILE_MODE);
        header.set_cksum();
        builder.append(&header, member_bytes).unwrap();
        builder.finish().unwrap();
        compute_path_blake3_digest(path).unwrap()
    }

    fn write_fixture_file(path: &Path, content: &[u8]) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, content).unwrap();
    }

    fn find_git_binary() -> PathBuf {
        if let Ok(git) = std::env::var("GIT") {
            let path = PathBuf::from(git);
            if path.exists() {
                return path;
            }
        }
        for candidate in [
            "/usr/bin/git",
            "/bin/git",
            "/usr/local/bin/git",
            "/run/current-system/sw/bin/git",
        ] {
            let path = Path::new(candidate);
            if path.exists() {
                return path.to_path_buf();
            }
        }
        if let Ok(path_var) = std::env::var("PATH") {
            for dir in path_var.split(':') {
                let candidate = Path::new(dir).join("git");
                if candidate.exists() {
                    return candidate;
                }
            }
        }
        panic!("git not found for witness_rebuild tests");
    }

    fn run_git(repo_root: &Path, args: &[&str]) -> Output {
        Command::new(find_git_binary()).args(args).current_dir(repo_root).output().unwrap()
    }

    fn assert_git_ok(repo_root: &Path, args: &[&str]) {
        let output = run_git(repo_root, args);
        assert!(
            output.status.success(),
            "git {:?} failed: stdout={} stderr={}",
            args,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }

    fn git_stdout(repo_root: &Path, args: &[&str]) -> String {
        let output = run_git(repo_root, args);
        assert!(
            output.status.success(),
            "git {:?} failed: stdout={} stderr={}",
            args,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    fn cargo_sha256_hex(bytes: &[u8]) -> String {
        let mut hasher = <sha2::Sha256 as sha2::Digest>::new();
        <sha2::Sha256 as sha2::Digest>::update(&mut hasher, bytes);
        let digest = <sha2::Sha256 as sha2::Digest>::finalize(hasher);
        let encoded = data_encoding::HEXLOWER.encode(&digest);
        assert_eq!(encoded.len(), TEST_SHA256_HEX_LENGTH_CHARS);
        encoded
    }

    fn write_test_vendor_package(repo_root: &Path) {
        let manifest = b"[package]\nname=\"dep\"\nversion=\"0.1.0\"\n";
        let lib = b"pub fn dep() {}\n";
        write_fixture_file(&repo_root.join("vendor-deps/dep/Cargo.toml"), manifest);
        write_fixture_file(&repo_root.join("vendor-deps/dep/lib.rs"), lib);
        let manifest_digest = cargo_sha256_hex(manifest);
        let lib_digest = cargo_sha256_hex(lib);
        let checksum_manifest = format!(
            "{{\"files\":{{\"Cargo.toml\":\"{manifest_digest}\",\"lib.rs\":\"{lib_digest}\"}},\"package\":null}}",
        );
        write_fixture_file(&repo_root.join("vendor-deps/dep/.cargo-checksum.json"), checksum_manifest.as_bytes());
    }

    fn create_minimal_release_source_repo(repo_root: &Path) {
        write_fixture_file(&repo_root.join(".gitignore"), b"vendor-deps/\ntarget/\n.pi/\n");
        write_fixture_file(&repo_root.join(".cargo/vendor-config.toml"), b"directory = \"vendor-deps\"\n");
        write_test_vendor_package(repo_root);
        write_fixture_file(
            &repo_root.join("Cargo.lock"),
            b"[[package]]\nname = \"dep\"\nversion = \"0.1.0\"\nsource = \"git+https://example.invalid/dep.git#0123456789abcdef\"\n",
        );
        write_fixture_file(
            &repo_root.join("Cargo.toml"),
            b"[package]\nname=\"demo\"\nversion=\"0.1.0\"\nedition=\"2024\"\n",
        );
        write_fixture_file(&repo_root.join("README.md"), b"# demo\n");
        write_fixture_file(&repo_root.join("docs/bootstrap-stage0-inventory.md"), b"# inventory\n");
        write_fixture_file(&repo_root.join("scripts/prove-self-hosting.sh"), b"#!/usr/bin/env bash\n");
        write_fixture_file(&repo_root.join("tests/audit_support.rs"), b"pub fn audit() {}\n");
        write_fixture_file(&repo_root.join("tests/self_hosting.rs"), b"#[test]\nfn proof() {}\n");
        write_fixture_file(&repo_root.join("lib/lib.ncl"), b"{}\n");
        write_fixture_file(&repo_root.join("rust-toolchain.toml"), b"[toolchain]\nchannel=\"nightly\"\n");
        write_fixture_file(&repo_root.join("src/main.rs"), b"fn main() { println!(\"git-source\"); }\n");
    }

    fn create_minimal_git_release_source_repo(repo_root: &Path) -> String {
        create_minimal_release_source_repo(repo_root);
        assert_git_ok(repo_root, &["init"]);
        assert_git_ok(repo_root, &["config", "user.email", "pi@example.com"]);
        assert_git_ok(repo_root, &["config", "user.name", "Pi"]);
        assert_git_ok(repo_root, &["add", "."]);
        assert_git_ok(repo_root, &["add", "-f", "vendor-deps"]);
        assert_git_ok(repo_root, &["commit", "-m", "initial"]);
        git_stdout(repo_root, &["rev-parse", "HEAD"])
    }

    fn release_manifest_without_source_acquisition() -> ReleaseEvidenceManifest {
        let digest = "a".repeat(TEST_BLAKE3_HEX_LENGTH_CHARS);
        let artifact = crate::release_evidence::BundledArtifact {
            kind: crate::release_evidence::BundledArtifactKind::File,
            relative_path: "source.tar".to_string(),
            size_bytes: 1,
            digest_blake3: digest.clone(),
        };
        ReleaseEvidenceManifest {
            schema: crate::release_evidence::RELEASE_EVIDENCE_SCHEMA.to_string(),
            release_id: "test-release".to_string(),
            claim_scope: crate::release_evidence::CLAIM_SCOPE_PACKAGED_INTEGRITY.to_string(),
            workflow: crate::release_evidence::ReleaseWorkflowIdentity {
                command: DEFAULT_PROOF_WORKFLOW_COMMAND.to_string(),
                version: DEFAULT_PROOF_WORKFLOW_VERSION.to_string(),
            },
            source_archive: artifact.clone(),
            source_acquisition: None,
            binaries: vec![artifact.clone()],
            proof_bundle: artifact.clone(),
            prerequisite_inventory: artifact,
            provider_fixed_point_proof: None,
            reproducibility_report: None,
            deterministic_build_proof: None,
            deterministic_sandbox_isolation_evidence: None,
            independent_agreement_report: None,
            proof_linkage: crate::release_evidence::ReleaseProofLinkage {
                release_id: "test-release".to_string(),
                source_archive_digest_blake3: digest.clone(),
                proof_bundle_schema: crate::release_evidence::FULL_SELF_HOSTING_PROOF_SCHEMA.to_string(),
                proof_mode: PROOF_MODE_FIXED_POINT.to_string(),
                selected_provider_kind: "self-hosting".to_string(),
                staged_source: "source.tar".to_string(),
                stage2_binary_digest_blake3: digest.clone(),
                prerequisite_inventory_digest_blake3: digest.clone(),
                proof_manifest_digest_blake3: digest,
            },
        }
    }

    fn witness_rebuild_test_plan(
        scratch_root: &Path,
        expected_outputs: Vec<ExpectedRebuiltOutput>,
    ) -> WitnessRebuildPlan {
        WitnessRebuildPlan {
            request_dir: scratch_root.join("request"),
            request_path: scratch_root.join("request/request.json"),
            release_id: "test-release".to_string(),
            request_schema: WITNESS_REQUEST_SCHEMA.to_string(),
            request_layout_version: TEST_REQUEST_LAYOUT_VERSION,
            request_bundle_dir: scratch_root.join("request/release-evidence/test-release"),
            request_verification_seed_dir: scratch_root.join("request/release-verification/test-release"),
            request_source_archive_path: scratch_root.join("request/release-evidence/test-release/source.tar"),
            workflow_command: DEFAULT_PROOF_WORKFLOW_COMMAND.to_string(),
            workflow_version: DEFAULT_PROOF_WORKFLOW_VERSION.to_string(),
            proof_mode: PROOF_MODE_FIXED_POINT.to_string(),
            expected_outputs,
            require_independent_source: false,
            require_git_source: false,
            source_acquisition: None,
            scratch_layout: derive_witness_rebuild_layout(scratch_root, "test-release").unwrap(),
        }
    }

    #[test]
    fn workflow_args_preserve_non_nix_proof_mode_and_reject_unknown_modes() {
        let fixed_point = workflow_args_for_proof_mode(PROOF_MODE_FIXED_POINT).unwrap();
        assert!(fixed_point.is_empty(), "fixed-point workflow uses default helper mode");

        let non_nix = workflow_args_for_proof_mode(PROOF_MODE_NON_NIX_HOST).unwrap();
        assert_eq!(non_nix, [WORKFLOW_NON_NIX_HOST_ARG]);

        let err = workflow_args_for_proof_mode("future-mode").unwrap_err();
        assert!(err.message().contains("unsupported witness rebuild proof mode"));
    }

    #[test]
    fn launched_workflow_command_records_proof_mode_arguments() {
        let command =
            launched_workflow_command(Path::new("/tmp/prove-self-hosting.sh"), PROOF_MODE_NON_NIX_HOST).unwrap();
        assert_eq!(command, vec![
            "/tmp/prove-self-hosting.sh".to_string(),
            WORKFLOW_NON_NIX_HOST_ARG.to_string()
        ]);
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
    fn source_acquisition_for_plan_accepts_metadata_when_flagged() {
        let digest = "a".repeat(TEST_BLAKE3_HEX_LENGTH_CHARS);
        let source_url = "https://example.invalid/mantle-src.tar";
        let mut manifest = release_manifest_without_source_acquisition();
        manifest.source_acquisition = Some(SourceAcquisition::external_archive(source_url.to_string(), digest));

        let planned = source_acquisition_for_plan(&manifest, true, false).unwrap().unwrap();

        assert_eq!(planned.url, source_url);
        assert_eq!(planned.kind, crate::release_evidence::SOURCE_ACQUISITION_KIND_EXTERNAL_ARCHIVE);
    }

    #[test]
    fn source_acquisition_for_plan_requires_metadata_when_flagged() {
        let manifest = release_manifest_without_source_acquisition();

        let err = source_acquisition_for_plan(&manifest, true, false).unwrap_err();

        assert!(err.message().contains("independent source acquisition required"));
    }

    #[test]
    fn source_acquisition_for_plan_requires_git_kind_when_flagged() {
        let digest = "a".repeat(TEST_BLAKE3_HEX_LENGTH_CHARS);
        let mut manifest = release_manifest_without_source_acquisition();
        manifest.source_acquisition =
            Some(SourceAcquisition::external_archive("https://example.invalid/mantle-src.tar".to_string(), digest));

        let err = source_acquisition_for_plan(&manifest, false, true).unwrap_err();

        assert!(err.message().contains("Git source required"));
        assert!(err.message().contains(SOURCE_ACQUISITION_KIND_EXTERNAL_ARCHIVE));
    }

    #[test]
    fn validate_source_replay_flags_rejects_conflicting_strict_modes() {
        let err = validate_source_replay_flags(true, true).unwrap_err();

        assert!(err.message().contains("mutually exclusive"));
    }

    #[cfg(unix)]
    #[test]
    fn git_source_success_audit_records_derivation_details() {
        let temp = tempfile::tempdir().unwrap();
        let digest = "d".repeat(TEST_BLAKE3_HEX_LENGTH_CHARS);
        let commit = "a".repeat(TEST_BLAKE3_HEX_LENGTH_CHARS);
        let mut plan = witness_rebuild_test_plan(temp.path(), Vec::new());
        plan.require_git_source = true;
        plan.source_acquisition = Some(SourceAcquisition::git(
            "file:///tmp/mantle-origin.git".to_string(),
            commit.clone(),
            Some("refs/heads/main".to_string()),
            Some("v0.1.0".to_string()),
            digest.clone(),
        ));
        let success = WitnessRebuildSuccess {
            rebuilt_output_paths: Vec::new(),
            workflow_driver_path: PathBuf::from("/tmp/prove-self-hosting.sh"),
            started_unix_ms: 1,
            finished_unix_ms: 2,
            output: empty_success_output(),
        };

        let meta =
            build_success_audit_meta(&plan, &success, Path::new("/tmp/witness.json"), Path::new("/tmp/witness.sig"))
                .unwrap();
        let audit = meta.source_acquisition.expect("Git-source audit should be present");

        assert_eq!(audit.mode, SOURCE_ACQUISITION_MODE_GIT);
        assert_eq!(audit.url.as_deref(), Some("file:///tmp/mantle-origin.git"));
        assert_eq!(audit.digest_blake3.as_deref(), Some(digest.as_str()));
        assert_eq!(audit.commit.as_deref(), Some(commit.as_str()));
        assert_eq!(audit.reference.as_deref(), Some("refs/heads/main"));
        assert_eq!(audit.tag.as_deref(), Some("v0.1.0"));
        assert_eq!(audit.status, SOURCE_ACQUISITION_STATUS_VERIFIED);
        assert!(audit.error.is_none());
    }

    #[test]
    fn prepare_scratch_fetches_independent_source_archive_when_required() {
        let temp = tempfile::tempdir().unwrap();
        let scratch_root = temp.path().join("scratch");
        let request_root = temp.path().join("request");
        let request_seed_dir = request_root.join("release-verification/test-release");
        let request_archive = request_root.join("release-evidence/test-release/source.tar");
        let external_archive = temp.path().join("external-source.tar");
        write_source_archive(&request_archive, "source.txt", b"publisher-bundled-source");
        let external_digest = write_source_archive(&external_archive, "source.txt", b"independent-source");
        std::fs::create_dir_all(&request_seed_dir).unwrap();
        let mut plan = witness_rebuild_test_plan(&scratch_root, Vec::new());
        plan.request_dir = request_root.clone();
        plan.request_path = request_root.join("request.json");
        plan.request_source_archive_path = request_archive;
        plan.request_verification_seed_dir = request_seed_dir;
        plan.require_independent_source = true;
        plan.source_acquisition = Some(SourceAcquisition::external_archive(
            format!("file://{}", external_archive.display()),
            external_digest.clone(),
        ));

        prepare_witness_rebuild_scratch(&plan).unwrap();

        let extracted_bytes = std::fs::read(plan.scratch_layout.repo_dir.join("source.txt")).unwrap();
        assert_eq!(extracted_bytes, b"independent-source");
        assert_eq!(
            compute_path_blake3_digest(&plan.scratch_layout.source_acquisition_archive_path).unwrap(),
            external_digest
        );
    }

    #[test]
    fn source_acquisition_rejects_digest_mismatch_before_extraction() {
        let temp = tempfile::tempdir().unwrap();
        let scratch_root = temp.path().join("scratch");
        let external_archive = temp.path().join("external-source.tar");
        write_source_archive(&external_archive, "source.txt", b"tampered-source");
        let mut plan = witness_rebuild_test_plan(&scratch_root, Vec::new());
        plan.require_independent_source = true;
        plan.source_acquisition = Some(SourceAcquisition::external_archive(
            format!("file://{}", external_archive.display()),
            "0".repeat(TEST_BLAKE3_HEX_LENGTH_CHARS),
        ));

        let err = source_archive_for_extraction(&plan).unwrap_err();

        assert!(err.message().contains("independent source acquisition digest mismatch"));
        assert!(!plan.scratch_layout.repo_dir.exists(), "repo extraction must not start after digest mismatch");
    }

    #[test]
    fn prepare_scratch_fetches_git_source_archive_when_required() {
        let temp = tempfile::tempdir().unwrap();
        let origin = temp.path().join("origin");
        std::fs::create_dir_all(&origin).unwrap();
        let commit = create_minimal_git_release_source_repo(&origin);
        let branch_ref = format!("refs/heads/{}", git_stdout(&origin, &["branch", "--show-current"]));
        let expected_archive = temp.path().join("expected-source.tar");
        crate::release_source::write_tracked_source_archive(&origin, &expected_archive).unwrap();
        let expected_digest = compute_path_blake3_digest(&expected_archive).unwrap();
        let scratch_root = temp.path().join("scratch");
        let request_root = temp.path().join("request");
        let request_seed_dir = request_root.join("release-verification/test-release");
        let request_archive = request_root.join("release-evidence/test-release/source.tar");
        write_source_archive(&request_archive, "source.txt", b"publisher-bundled-source");
        std::fs::create_dir_all(&request_seed_dir).unwrap();
        let mut plan = witness_rebuild_test_plan(&scratch_root, Vec::new());
        plan.request_dir = request_root.clone();
        plan.request_path = request_root.join("request.json");
        plan.request_source_archive_path = request_archive;
        plan.request_verification_seed_dir = request_seed_dir;
        plan.require_git_source = true;
        plan.source_acquisition = Some(SourceAcquisition::git(
            format!("file://{}", origin.display()),
            commit,
            Some(branch_ref),
            None,
            expected_digest.clone(),
        ));

        prepare_witness_rebuild_scratch(&plan).unwrap();

        let extracted_main = std::fs::read_to_string(plan.scratch_layout.repo_dir.join("src/main.rs")).unwrap();
        assert!(extracted_main.contains("git-source"));
        assert!(!plan.scratch_layout.repo_dir.join("source.txt").exists());
        assert_eq!(
            compute_path_blake3_digest(&plan.scratch_layout.source_acquisition_archive_path).unwrap(),
            expected_digest
        );
    }

    #[test]
    fn git_source_acquisition_rejects_digest_mismatch_before_extraction() {
        let temp = tempfile::tempdir().unwrap();
        let origin = temp.path().join("origin");
        std::fs::create_dir_all(&origin).unwrap();
        let commit = create_minimal_git_release_source_repo(&origin);
        let branch_ref = format!("refs/heads/{}", git_stdout(&origin, &["branch", "--show-current"]));
        let mut plan = witness_rebuild_test_plan(&temp.path().join("scratch"), Vec::new());
        plan.require_git_source = true;
        plan.source_acquisition = Some(SourceAcquisition::git(
            format!("file://{}", origin.display()),
            commit,
            Some(branch_ref),
            None,
            "0".repeat(TEST_BLAKE3_HEX_LENGTH_CHARS),
        ));

        let err = source_archive_for_extraction(&plan).unwrap_err();

        assert!(err.message().contains("Git source archive digest mismatch"));
        assert!(err.message().contains("generated"));
        assert!(!plan.scratch_layout.repo_dir.exists(), "repo extraction must not start after digest mismatch");
    }

    #[cfg(unix)]
    #[test]
    fn source_acquisition_success_audit_records_verified_fetch() {
        let temp = tempfile::tempdir().unwrap();
        let digest = "b".repeat(TEST_BLAKE3_HEX_LENGTH_CHARS);
        let source_url = "file:///tmp/mantle-src.tar";
        let mut plan = witness_rebuild_test_plan(temp.path(), Vec::new());
        plan.require_independent_source = true;
        plan.source_acquisition = Some(SourceAcquisition::external_archive(source_url.to_string(), digest.clone()));
        let success = WitnessRebuildSuccess {
            rebuilt_output_paths: Vec::new(),
            workflow_driver_path: PathBuf::from("/tmp/prove-self-hosting.sh"),
            started_unix_ms: 1,
            finished_unix_ms: 2,
            output: empty_success_output(),
        };

        let meta =
            build_success_audit_meta(&plan, &success, Path::new("/tmp/witness.json"), Path::new("/tmp/witness.sig"))
                .unwrap();
        let audit = meta.source_acquisition.expect("independent-source audit should be present");

        assert_eq!(audit.mode, SOURCE_ACQUISITION_MODE_EXTERNAL_ARCHIVE);
        assert_eq!(audit.url.as_deref(), Some(source_url));
        assert_eq!(audit.digest_blake3.as_deref(), Some(digest.as_str()));
        assert_eq!(audit.status, SOURCE_ACQUISITION_STATUS_VERIFIED);
        assert!(audit.error.is_none());
    }

    #[test]
    fn source_acquisition_prelaunch_failure_audit_records_error() {
        let temp = tempfile::tempdir().unwrap();
        let digest = "c".repeat(TEST_BLAKE3_HEX_LENGTH_CHARS);
        let failure = "independent source acquisition digest mismatch: expected c, fetched d";
        let mut plan = witness_rebuild_test_plan(temp.path(), Vec::new());
        plan.require_independent_source = true;
        plan.source_acquisition =
            Some(SourceAcquisition::external_archive("file:///tmp/mantle-src.tar".to_string(), digest));

        let meta = build_prelaunch_failure_audit_meta(&plan, failure).unwrap();
        let audit = meta.source_acquisition.expect("failed independent-source audit should be present");

        assert_eq!(meta.status, audit_status_failed());
        assert_eq!(audit.status, audit_status_failed());
        assert_eq!(audit.error.as_deref(), Some(failure));
        assert_eq!(meta.launched_command[0], "<not-launched>");
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
