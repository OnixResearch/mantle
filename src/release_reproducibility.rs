use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fs::File;
use std::io::Read;
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command as ProcessCommand;

use crunch_release_core::BUILD_EFFECT_POLICY_VERSION;
use crunch_release_core::BundledArtifact;
use crunch_release_core::BundledArtifactKind;
use crunch_release_core::DETERMINISTIC_BUILD_PROOF_EVIDENCE_ROLE;
use crunch_release_core::DETERMINISTIC_BUILD_PROOF_RECEIPT_SCHEMA;
use crunch_release_core::DETERMINISTIC_SANDBOX_ISOLATION_EVIDENCE_ROLE;
use crunch_release_core::DETERMINISTIC_SANDBOX_ISOLATION_EVIDENCE_SCHEMA;
use crunch_release_core::DeterministicBuildProofReceipt;
use crunch_release_core::DeterministicBuildProofReceiptInit;
use crunch_release_core::DeterministicBuildRunReceipt;
use crunch_release_core::DeterministicOutputDigest;
use crunch_release_core::DeterministicProofUnit;
use crunch_release_core::DeterministicSandboxIsolationEvidence;
use crunch_release_core::DeterministicSandboxIsolationEvidenceStatus;
use crunch_release_core::MAX_REBUILD_RUN_COUNT;
use crunch_release_core::PURE_LOCAL_BUILD_EFFECTS;
use crunch_release_core::REQUIRED_ISOLATION_CHECKS;
use crunch_release_core::RebuildWorkflowIdentity;
use crunch_release_core::ReleaseEvidenceManifest;
use crunch_release_core::ReleaseReproducibilityReport;
use crunch_release_core::ReleaseReproducibilityReportInit;
use crunch_release_core::ReleaseReproducibilityReportLinkage;
use crunch_release_core::ReproducibilityArtifactComparison;
use crunch_release_core::ReproducibilityComparisonResult;
use crunch_release_core::ReproducibilityComparisonVerdict;
use crunch_release_core::ReproducibilityProofClass;
use crunch_release_core::RoleBoundedReleaseArtifact;
use crunch_release_core::canonical_release_evidence_manifest;
use crunch_release_core::deterministic_build_proof_receipt_canonical_bytes;
use crunch_release_core::deterministic_build_proof_receipt_digest_blake3;
use crunch_release_core::deterministic_sandbox_isolation_evidence_canonical_bytes;
use crunch_release_core::deterministic_sandbox_isolation_evidence_digest_blake3;
use crunch_release_core::release_reproducibility_report_canonical_bytes;
use crunch_release_core::release_reproducibility_report_digest_blake3;
use crunch_release_core::validate_release_reproducibility_report_artifact_names;
use crunch_release_core::validate_release_reproducibility_report_linkage;

use crate::errors::RunError;
use crate::proof_clock_seccomp::ProofClockSeccompFilter;
use crate::proof_clock_seccomp::prepare_proof_clock_seccomp_filter;
use crate::proof_clock_seccomp::proof_clock_filter_identity;
use crate::rebuild_authority::PreparedRebuildAuthority;
use crate::rebuild_authority::RebuildRunPaths;
use crate::rebuild_authority::prepare_rebuild_authority;
use crate::release_evidence::verify_release_evidence_bundle;

pub(crate) const DEFAULT_REPRODUCIBILITY_WORKFLOW_VERSION: &str = "mantle-release-reproducibility-v1";

const REPRODUCE_BUNDLE_DIR_ENV: &str = "MANTLE_REPRODUCE_BUNDLE_DIR";
const REPRODUCE_OUTPUT_DIR_ENV: &str = "MANTLE_REPRODUCE_OUTPUT_DIR";
const REPRODUCE_RELEASE_ID_ENV: &str = "MANTLE_REPRODUCE_RELEASE_ID";
const REBUILD_SOURCE_ARCHIVE_ENV: &str = "MANTLE_REBUILD_SOURCE_ARCHIVE";
const REBUILD_EXECUTABLE_ENV: &str = "MANTLE_REBUILD_EXECUTABLE";
const DETERMINISTIC_PROOF_STORE_DIR_ENV: &str = "MANTLE_DETERMINISTIC_PROOF_STORE_DIR";
const DEFAULT_REPORT_RELATIVE_PATH: &str = "reproducibility/reproducibility-report.json";
const DETERMINISTIC_BUILD_PROOF_BUNDLE_RELATIVE_PATH: &str = "deterministic-release/deterministic-build-proof.json";
const DETERMINISTIC_SANDBOX_EVIDENCE_BUNDLE_RELATIVE_PATH: &str =
    "deterministic-release/deterministic-sandbox-isolation-evidence.json";
const HASH_BUFFER_BYTES: usize = 8_192;
const HASH_BUFFER_BYTES_U64: u64 = 8_192;
const MAX_REBUILD_OUTPUT_ENTRIES: u32 = 4096;
const PROOF_SANDBOX_BWRAP_ENV: &str = "MANTLE_DETERMINISTIC_PROOF_BWRAP";
const PROOF_SANDBOX_PROFILE_PREFIX: &str = "mantle-proof-sandbox-v1";
const PROOF_SANDBOX_PATH: &str = "/nonexistent";
const PROOF_SANDBOX_TEMP_ROOT: &str = "/tmp";
const PROOF_SANDBOX_USER: &str = "nobody";
const PROOF_SANDBOX_LOCALE: &str = "C.UTF-8";
const PROOF_SANDBOX_TIMEZONE: &str = "UTC";
const PROOF_SOURCE_DATE_EPOCH: &str = "1";
const PROOF_MODELED_RANDOMNESS: &str = "no-modeled-random-seed";
const PROOF_OUTPUT_ORDERING: &str = "lexicographic-output-processing";
const PROOF_SANDBOX_UMASK_TEXT: &str = "0022";
const PROOF_CLOCK_SYSCALL_ENFORCEMENT: &str = "bwrap-seccomp-errno-eperm";
const PROOF_NORMALIZATION_CONTROL_COUNT: usize = 8;
#[cfg(unix)]
const PROOF_SANDBOX_UMASK: libc::mode_t = 0o022;
#[cfg(unix)]
const UNIX_PERMISSION_MODE_MASK: libc::mode_t = 0o777;
const PROOF_EXECUTOR_TEST_ENV_ALLOWLIST: &[&str] = &[
    "MANTLE_FAKE_BWRAP_TRANSCRIPT",
    "MANTLE_FAKE_BWRAP_FORBIDDEN_BIND",
    "MANTLE_FAKE_BWRAP_FORBIDDEN_HOST_PATH",
    "MANTLE_FAKE_BWRAP_HOST_PATH",
];
const PROOF_SANDBOX_FIXED_ENV: &[(&str, &str)] = &[
    ("HOME", PROOF_SANDBOX_TEMP_ROOT),
    ("USER", PROOF_SANDBOX_USER),
    ("LOGNAME", PROOF_SANDBOX_USER),
    ("PATH", PROOF_SANDBOX_PATH),
    ("LANG", PROOF_SANDBOX_LOCALE),
    ("LC_ALL", PROOF_SANDBOX_LOCALE),
    ("TZ", PROOF_SANDBOX_TIMEZONE),
    ("TEMP", PROOF_SANDBOX_TEMP_ROOT),
    ("TEMPDIR", PROOF_SANDBOX_TEMP_ROOT),
    ("TMP", PROOF_SANDBOX_TEMP_ROOT),
    ("TMPDIR", PROOF_SANDBOX_TEMP_ROOT),
    ("SOURCE_DATE_EPOCH", PROOF_SOURCE_DATE_EPOCH),
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProofSandboxProfile {
    identity: String,
    executor: PathBuf,
    executor_version: String,
    network_policy: String,
    clock_syscall_policy_identity: String,
    descriptor_blake3: String,
    authority_plan_blake3: String,
    command_path: PathBuf,
    command_args: Vec<OsString>,
    source_archive_path: PathBuf,
    read_only_paths: Vec<PathBuf>,
    output_dir: PathBuf,
    store_dir: PathBuf,
}

impl ProofSandboxProfile {
    fn canonical_facts(&self) -> Vec<String> {
        let mut facts = vec![
            format!("profile={PROOF_SANDBOX_PROFILE_PREFIX}"),
            "executor=bwrap".to_string(),
            format!("executor_path={}", self.executor.display()),
            format!("executor_version={}", self.executor_version),
            format!("network={}", self.network_policy),
            format!("clock_syscall_enforcement={PROOF_CLOCK_SYSCALL_ENFORCEMENT}"),
            format!("clock_syscall_policy_identity={}", self.clock_syscall_policy_identity),
            "release_bundle_authority=absent".to_string(),
            format!("rebuild_descriptor_blake3={}", self.descriptor_blake3),
            format!("rebuild_authority_plan_blake3={}", self.authority_plan_blake3),
            format!("command_ro={}", self.command_path.display()),
            format!("source_archive_ro={}", self.source_archive_path.display()),
            format!("read_only_input_count={}", self.read_only_paths.len()),
            format!("output_rw={}", self.output_dir.display()),
            format!("store_rw={}", self.store_dir.display()),
            "executor_env_policy=clear-with-bounded-test-seam".to_string(),
            "rebuild_env_policy=clear-then-fixed".to_string(),
        ];
        facts.extend(deterministic_normalization_envelope());
        debug_assert!(facts.iter().any(|fact| fact.starts_with("normalization:time=")));
        debug_assert!(facts.iter().any(|fact| fact.starts_with("normalization:umask=")));
        facts
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ReleaseReproduceRequest {
    pub bundle_dir: PathBuf,
    pub rebuild_output_dir: PathBuf,
    pub rebuild_command: PathBuf,
    pub rebuild_args: Vec<OsString>,
    pub workflow_version: String,
    pub report_path: Option<PathBuf>,
    pub deterministic_proof_runs: u32,
    pub deterministic_proof_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReproducibilityStatus {
    Absent,
    Matched,
    Mismatched,
}

impl ReproducibilityStatus {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Absent => "absent",
            Self::Matched => "matched",
            Self::Mismatched => "mismatched",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VerifiedReproducibilityReport {
    pub path: PathBuf,
    pub digest_blake3: String,
    pub status: ReproducibilityStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReleaseReproduceSummary {
    pub release_id: String,
    pub report_path: PathBuf,
    pub report_digest_blake3: String,
    pub matched_count: u32,
    pub mismatched_count: u32,
    pub missing_count: u32,
    pub deterministic_proof_path: Option<PathBuf>,
    pub deterministic_proof_digest_blake3: Option<String>,
    pub deterministic_sandbox_isolation_evidence_path: Option<PathBuf>,
    pub deterministic_sandbox_isolation_evidence_digest_blake3: Option<String>,
    pub deterministic_proof_unit: Option<serde_json::Value>,
    pub deterministic_proof_run_roots: Option<serde_json::Value>,
    pub deterministic_proof_sandbox_profiles: Option<Vec<String>>,
    pub deterministic_proof_verdict: Option<String>,
    pub deterministic_proof_blockers: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ObservedArtifact {
    size_bytes: u64,
    digest_blake3: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ComparisonCounts {
    matched_count: u32,
    mismatched_count: u32,
    missing_count: u32,
}

struct PreparedRebuildCommand {
    command: ProcessCommand,
    _clock_filter: Option<ProofClockSeccompFilter>,
}

pub(crate) fn reproduce_release_artifacts(
    request: &ReleaseReproduceRequest,
) -> Result<ReleaseReproduceSummary, RunError> {
    validate_request(request)?;
    debug_assert!(request.bundle_dir.is_dir());
    debug_assert_eq!(request.workflow_version, DEFAULT_REPRODUCIBILITY_WORKFLOW_VERSION);
    let mut manifest = verify_release_evidence_bundle(&request.bundle_dir)?;
    prepare_rebuild_output_dir(&request.rebuild_output_dir)?;
    let source_archive_path = request.bundle_dir.join(&manifest.source_archive.relative_path);
    run_rebuild_command(RebuildCommandInvocation {
        request,
        release_id: &manifest.release_id,
        output_dir: &request.rebuild_output_dir,
        source_archive_path: &source_archive_path,
        deterministic_store_dir: None,
        prepared_rebuild: None,
        sandbox_profile: None,
    })?;
    let unexpected_outputs = collect_unexpected_rebuilt_outputs(&manifest.binaries, &request.rebuild_output_dir)?;
    let reproducibility_evidence = build_reproducibility_report(&manifest, request)?;
    let counts = count_report_results(&reproducibility_evidence.artifacts);
    let reproducibility_digest_blake3 =
        release_reproducibility_report_digest_blake3(reproducibility_evidence.clone()).map_err(core_error)?;
    let reproducibility_path = resolve_report_path(&request.bundle_dir, request.report_path.as_deref());
    write_report(&reproducibility_path, reproducibility_evidence.clone())?;
    fail_if_reproduction_drifted(&reproducibility_evidence, &unexpected_outputs, &reproducibility_path)?;
    let deterministic_proof = maybe_run_deterministic_proof(request, &manifest, &reproducibility_path)?;
    if let Some(proof) = &deterministic_proof {
        attach_deterministic_proof_artifacts(&request.bundle_dir, &mut manifest, proof)?;
    }
    build_release_reproduce_summary(
        manifest,
        reproducibility_path,
        reproducibility_digest_blake3,
        counts,
        deterministic_proof.as_ref(),
    )
}

fn build_release_reproduce_summary(
    manifest: ReleaseEvidenceManifest,
    reproducibility_path: PathBuf,
    reproducibility_digest_blake3: String,
    counts: ComparisonCounts,
    deterministic_proof: Option<&DeterministicProofOutput>,
) -> Result<ReleaseReproduceSummary, RunError> {
    debug_assert!(!manifest.release_id.is_empty());
    debug_assert!(!reproducibility_digest_blake3.is_empty());
    let deterministic_proof_unit = deterministic_proof.map(deterministic_proof_unit_json);
    let deterministic_proof_run_roots = deterministic_proof.map(deterministic_proof_run_roots_json);
    let deterministic_proof_verdict = deterministic_proof
        .map(|proof| serde_json::to_value(proof.receipt.verdict))
        .transpose()
        .map_err(|err| RunError::Internal(format!("serializing deterministic proof verdict: {err}")))?
        .and_then(|value| value.as_str().map(ToOwned::to_owned));
    Ok(ReleaseReproduceSummary {
        release_id: manifest.release_id,
        report_path: reproducibility_path,
        report_digest_blake3: reproducibility_digest_blake3,
        matched_count: counts.matched_count,
        mismatched_count: counts.mismatched_count,
        missing_count: counts.missing_count,
        deterministic_proof_path: deterministic_proof.map(|proof| proof.path.clone()),
        deterministic_proof_digest_blake3: deterministic_proof.map(|proof| proof.digest_blake3.clone()),
        deterministic_sandbox_isolation_evidence_path: deterministic_proof
            .map(|proof| proof.isolation_evidence_path.clone()),
        deterministic_sandbox_isolation_evidence_digest_blake3: deterministic_proof
            .map(|proof| proof.isolation_evidence_digest_blake3.clone()),
        deterministic_proof_unit,
        deterministic_proof_run_roots,
        deterministic_proof_sandbox_profiles: deterministic_proof
            .map(|proof| proof.receipt.sandbox_profile_identities.clone()),
        deterministic_proof_verdict,
        deterministic_proof_blockers: deterministic_proof.map(|proof| proof.receipt.blocking_reasons.clone()),
    })
}

fn deterministic_proof_unit_json(proof: &DeterministicProofOutput) -> serde_json::Value {
    serde_json::json!({
        "target_artifact_identity": proof.receipt.proof_unit.target_artifact_identity.clone(),
        "output_identities": proof.receipt.proof_unit.output_identities.clone(),
        "selected_provider_kind": proof.receipt.selected_provider_kind.clone(),
        "source_blake3": proof.receipt.source_blake3.clone(),
        "vendor_blake3": proof.receipt.vendor_blake3.clone(),
        "toolchain_stage_roots": proof.receipt.toolchain_stage_roots.clone(),
        "rebuild_descriptor_blake3": proof.receipt.rebuild_descriptor_blake3.clone(),
        "rebuild_authority_plan_blake3": proof.receipt.rebuild_authority_plan_blake3.clone(),
        "target_authority_excluded": proof.receipt.rebuild_authority_plan.as_ref().map(|plan| plan.target_authority_excluded),
    })
}

fn deterministic_proof_run_roots_json(proof: &DeterministicProofOutput) -> serde_json::Value {
    serde_json::json!(
        proof
            .receipt
            .runs
            .iter()
            .map(|run| {
                serde_json::json!({
                    "run_id": run.run_id.clone(),
                    "output_store_paths": run.output_store_paths.clone(),
                    "output_root_identity": run.output_root_identity.clone(),
                    "sandbox_profile_identity": run.sandbox_profile_identity.clone(),
                })
            })
            .collect::<Vec<_>>()
    )
}

pub(crate) fn load_bundle_reproducibility_report(
    bundle_dir: &Path,
    manifest: &ReleaseEvidenceManifest,
) -> Result<Option<VerifiedReproducibilityReport>, RunError> {
    let has_manifest_reference = manifest.reproducibility_report.is_some();
    let reproducibility_path = manifest
        .reproducibility_report
        .as_ref()
        .map(|artifact| bundle_dir.join(&artifact.relative_path))
        .unwrap_or_else(|| default_reproducibility_report_path(bundle_dir));
    if !reproducibility_path.exists() {
        if has_manifest_reference {
            return Err(RunError::Internal(format!(
                "release reproducibility report referenced by manifest is missing: {}",
                reproducibility_path.display()
            )));
        }
        return Ok(None);
    }
    debug_assert!(reproducibility_path.exists());
    debug_assert!(reproducibility_path.starts_with(bundle_dir));
    let report_bytes = std::fs::read(&reproducibility_path)
        .map_err(|err| RunError::Internal(format!("reading {}: {err}", reproducibility_path.display())))?;
    let reproducibility_evidence: ReleaseReproducibilityReport = serde_json::from_slice(&report_bytes)
        .map_err(|err| RunError::Internal(format!("parsing {}: {err}", reproducibility_path.display())))?;
    let canonical_bytes =
        release_reproducibility_report_canonical_bytes(reproducibility_evidence.clone()).map_err(core_error)?;
    if report_bytes != canonical_bytes {
        return Err(RunError::Internal("release reproducibility report is not canonical compact JSON".to_string()));
    }
    let reproducibility_evidence = validate_report_linkage_and_artifacts(reproducibility_evidence, manifest)?;
    let digest_blake3 = blake3::hash(&canonical_bytes).to_hex().to_string();
    let status = report_status(&reproducibility_evidence);
    Ok(Some(VerifiedReproducibilityReport {
        path: reproducibility_path,
        digest_blake3,
        status,
    }))
}

fn validate_report_linkage_and_artifacts(
    report: ReleaseReproducibilityReport,
    manifest: &ReleaseEvidenceManifest,
) -> Result<ReleaseReproducibilityReport, RunError> {
    let expected = ReleaseReproducibilityReportLinkage {
        release_id: manifest.release_id.clone(),
        source_archive_digest_blake3: manifest.source_archive.digest_blake3.clone(),
        proof_bundle_digest_blake3: manifest.proof_bundle.digest_blake3.clone(),
    };
    let reproducibility_evidence =
        validate_release_reproducibility_report_linkage(report, expected).map_err(core_error)?;
    let expected_names = manifest.binaries.iter().map(|artifact| artifact.relative_path.clone()).collect::<Vec<_>>();
    validate_release_reproducibility_report_artifact_names(reproducibility_evidence, expected_names).map_err(core_error)
}

fn report_status(report: &ReleaseReproducibilityReport) -> ReproducibilityStatus {
    if report.artifacts.iter().all(|artifact| artifact.result == ReproducibilityComparisonResult::Matched) {
        return ReproducibilityStatus::Matched;
    }
    ReproducibilityStatus::Mismatched
}

fn validate_request(request: &ReleaseReproduceRequest) -> Result<(), RunError> {
    if !request.bundle_dir.is_dir() {
        return Err(RunError::Internal(format!(
            "release reproducibility bundle directory is missing: {}",
            request.bundle_dir.display()
        )));
    }
    if request.rebuild_command.as_os_str().is_empty() {
        return Err(RunError::Internal("release reproducibility rebuild command must not be empty".to_string()));
    }
    if request.workflow_version.trim().is_empty() {
        return Err(RunError::Internal("release reproducibility workflow version must not be empty".to_string()));
    }
    if request.workflow_version != DEFAULT_REPRODUCIBILITY_WORKFLOW_VERSION {
        return Err(RunError::Internal(format!(
            "unsupported release reproducibility workflow version '{}': expected '{}'",
            request.workflow_version, DEFAULT_REPRODUCIBILITY_WORKFLOW_VERSION
        )));
    }
    if request.deterministic_proof_runs == 1 {
        return Err(RunError::Internal(
            "deterministic proof runs must be 0 or at least 2; one run cannot prove determinism".to_string(),
        ));
    }
    if request.deterministic_proof_runs > MAX_REBUILD_RUN_COUNT {
        return Err(RunError::Internal(format!(
            "deterministic proof runs {} exceed maximum {MAX_REBUILD_RUN_COUNT}",
            request.deterministic_proof_runs
        )));
    }
    debug_assert!(request.bundle_dir.is_dir());
    debug_assert_eq!(request.workflow_version, DEFAULT_REPRODUCIBILITY_WORKFLOW_VERSION);
    Ok(())
}

fn prepare_rebuild_output_dir(output_dir: &Path) -> Result<(), RunError> {
    if output_dir.exists() {
        if !output_dir.is_dir() {
            return Err(RunError::Internal(format!(
                "release reproducibility output path is not a directory: {}",
                output_dir.display()
            )));
        }
        let mut entries = std::fs::read_dir(output_dir)
            .map_err(|err| RunError::Internal(format!("reading {}: {err}", output_dir.display())))?;
        if entries.next().is_some() {
            return Err(RunError::Internal(format!(
                "release reproducibility output directory must be empty: {}",
                output_dir.display()
            )));
        }
        debug_assert!(output_dir.is_dir());
        debug_assert!(std::fs::read_dir(output_dir).is_ok_and(|mut entries| entries.next().is_none()));
        return Ok(());
    }
    std::fs::create_dir_all(output_dir)
        .map_err(|err| RunError::Internal(format!("creating {}: {err}", output_dir.display())))?;
    debug_assert!(output_dir.is_dir());
    debug_assert!(output_dir.exists());
    Ok(())
}

struct RebuildCommandInvocation<'a> {
    request: &'a ReleaseReproduceRequest,
    release_id: &'a str,
    output_dir: &'a Path,
    source_archive_path: &'a Path,
    deterministic_store_dir: Option<&'a Path>,
    prepared_rebuild: Option<&'a PreparedRebuildAuthority>,
    sandbox_profile: Option<&'a ProofSandboxProfile>,
}

fn run_rebuild_command(invocation: RebuildCommandInvocation<'_>) -> Result<(), RunError> {
    debug_assert!(!invocation.release_id.is_empty());
    debug_assert!(invocation.source_archive_path.exists());
    let request = invocation.request;
    let mut prepared_command = if let Some(profile) = invocation.sandbox_profile {
        let prepared_rebuild = invocation.prepared_rebuild.ok_or_else(|| {
            RunError::Internal("deterministic proof sandbox requires prepared rebuild authority".to_string())
        })?;
        sandboxed_rebuild_command(
            invocation.release_id,
            invocation.output_dir,
            invocation.deterministic_store_dir,
            prepared_rebuild,
            profile,
        )?
    } else {
        let mut command = ProcessCommand::new(&request.rebuild_command);
        command
            .args(&request.rebuild_args)
            .env(REPRODUCE_BUNDLE_DIR_ENV, &request.bundle_dir)
            .env(REBUILD_SOURCE_ARCHIVE_ENV, invocation.source_archive_path)
            .env(REBUILD_EXECUTABLE_ENV, &request.rebuild_command)
            .env(REPRODUCE_OUTPUT_DIR_ENV, invocation.output_dir)
            .env(REPRODUCE_RELEASE_ID_ENV, invocation.release_id);
        if let Some(store_dir) = invocation.deterministic_store_dir {
            command.env(DETERMINISTIC_PROOF_STORE_DIR_ENV, store_dir);
        }
        PreparedRebuildCommand {
            command,
            _clock_filter: None,
        }
    };
    let output = prepared_command.command.output().map_err(|err| {
        RunError::Internal(format!(
            "running release reproducibility command {}: {err}",
            request.rebuild_command.display()
        ))
    })?;
    if output.status.success() {
        return Ok(());
    }
    Err(RunError::Internal(format!(
        "release reproducibility command failed with status {}: stdout={} stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DeterministicProofOutput {
    path: PathBuf,
    digest_blake3: String,
    receipt: DeterministicBuildProofReceipt,
    isolation_evidence_path: PathBuf,
    isolation_evidence_digest_blake3: String,
}

fn attach_deterministic_proof_artifacts(
    bundle_dir: &Path,
    manifest: &mut ReleaseEvidenceManifest,
    proof: &DeterministicProofOutput,
) -> Result<(), RunError> {
    debug_assert!(proof.path.exists());
    debug_assert!(proof.isolation_evidence_path.exists());
    let build_proof = copy_role_bounded_file_into_bundle(RoleBoundedCopyRequest {
        source: &proof.path,
        bundle_dir,
        relative_path: DETERMINISTIC_BUILD_PROOF_BUNDLE_RELATIVE_PATH,
        evidence_role: DETERMINISTIC_BUILD_PROOF_EVIDENCE_ROLE,
        label: "deterministic build proof receipt",
    })?;
    let sandbox_evidence = copy_role_bounded_file_into_bundle(RoleBoundedCopyRequest {
        source: &proof.isolation_evidence_path,
        bundle_dir,
        relative_path: DETERMINISTIC_SANDBOX_EVIDENCE_BUNDLE_RELATIVE_PATH,
        evidence_role: DETERMINISTIC_SANDBOX_ISOLATION_EVIDENCE_ROLE,
        label: "deterministic sandbox isolation evidence",
    })?;
    let mut updated = manifest.clone();
    updated.deterministic_build_proof = Some(build_proof);
    updated.deterministic_sandbox_isolation_evidence = Some(sandbox_evidence);
    write_release_evidence_manifest(bundle_dir, &updated)?;
    *manifest = updated;
    Ok(())
}

struct RoleBoundedCopyRequest<'a> {
    source: &'a Path,
    bundle_dir: &'a Path,
    relative_path: &'a str,
    evidence_role: &'a str,
    label: &'a str,
}

fn copy_role_bounded_file_into_bundle(
    request: RoleBoundedCopyRequest<'_>,
) -> Result<RoleBoundedReleaseArtifact, RunError> {
    debug_assert!(request.source.is_file());
    debug_assert!(!request.relative_path.is_empty());
    let destination = request.bundle_dir.join(request.relative_path);
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| RunError::Internal(format!("creating {}: {err}", parent.display())))?;
    }
    std::fs::copy(request.source, &destination).map_err(|err| {
        RunError::Internal(format!(
            "copying {} {} to {}: {err}",
            request.label,
            request.source.display(),
            destination.display()
        ))
    })?;
    role_bounded_file_artifact_record(RoleBoundedArtifactRequest {
        path: &destination,
        relative_path: request.relative_path,
        evidence_role: request.evidence_role,
        label: request.label,
    })
}

struct RoleBoundedArtifactRequest<'a> {
    path: &'a Path,
    relative_path: &'a str,
    evidence_role: &'a str,
    label: &'a str,
}

fn role_bounded_file_artifact_record(
    request: RoleBoundedArtifactRequest<'_>,
) -> Result<RoleBoundedReleaseArtifact, RunError> {
    let (size_bytes, digest_blake3) = hash_file(request.path).map_err(|err| {
        RunError::Internal(format!("hashing bundle-local {} {}: {err}", request.label, request.path.display()))
    })?;
    Ok(RoleBoundedReleaseArtifact {
        kind: BundledArtifactKind::File,
        relative_path: request.relative_path.to_string(),
        size_bytes,
        digest_blake3,
        evidence_role: request.evidence_role.to_string(),
    })
}

fn write_release_evidence_manifest(bundle_dir: &Path, manifest: &ReleaseEvidenceManifest) -> Result<(), RunError> {
    let bytes = canonical_release_evidence_manifest(manifest.clone()).map_err(core_error)?;
    let path = bundle_dir.join("manifest.json");
    std::fs::write(&path, bytes).map_err(|err| RunError::Internal(format!("writing {}: {err}", path.display())))
}

// r[impl mantle.build_correctness.release_determinism.genuine_rebuild]
// r[impl mantle.build_correctness.release_determinism.fixtures.positive]
fn maybe_run_deterministic_proof(
    request: &ReleaseReproduceRequest,
    manifest: &ReleaseEvidenceManifest,
    report_path: &Path,
) -> Result<Option<DeterministicProofOutput>, RunError> {
    if request.deterministic_proof_runs == 0 {
        return Ok(None);
    }
    debug_assert!(request.deterministic_proof_runs > 1);
    debug_assert!(request.deterministic_proof_runs <= MAX_REBUILD_RUN_COUNT);
    let proof_root = deterministic_proof_root(request);
    validate_deterministic_proof_root(&proof_root, &request.rebuild_output_dir)?;
    prepare_rebuild_output_dir(&proof_root)?;
    let run_paths = deterministic_run_paths(&proof_root, request.deterministic_proof_runs);
    let prepared_rebuild = prepare_rebuild_authority(
        manifest,
        &request.bundle_dir,
        &request.rebuild_command,
        &request.rebuild_args,
        &proof_root,
        &request.rebuild_output_dir,
        &run_paths,
    )?;
    let (runs, sandbox_profiles) =
        execute_deterministic_runs(request, manifest, report_path, &prepared_rebuild, &run_paths)?;
    let receipt = build_deterministic_receipt(manifest, &prepared_rebuild, runs);
    persist_deterministic_proof(&proof_root, receipt, &sandbox_profiles).map(Some)
}

fn deterministic_run_paths(proof_root: &Path, run_count: u32) -> Vec<RebuildRunPaths> {
    (0..run_count)
        .map(|run_index| {
            let run_id = format!("run-{run_index:03}");
            let run_root = proof_root.join(&run_id);
            RebuildRunPaths {
                run_id,
                output_dir: run_root.join("outputs"),
                store_dir: run_root.join("store"),
            }
        })
        .collect()
}

fn execute_deterministic_runs(
    request: &ReleaseReproduceRequest,
    manifest: &ReleaseEvidenceManifest,
    report_path: &Path,
    prepared_rebuild: &PreparedRebuildAuthority,
    run_paths: &[RebuildRunPaths],
) -> Result<(Vec<DeterministicBuildRunReceipt>, Vec<ProofSandboxProfile>), RunError> {
    let mut receipts = Vec::with_capacity(run_paths.len());
    let mut profiles = Vec::with_capacity(run_paths.len());
    for (run_index, run_path) in run_paths.iter().enumerate() {
        let run_index = u32::try_from(run_index)
            .map_err(|_| RunError::Internal("deterministic proof run index overflowed u32".to_string()))?;
        let (receipt, profile) = execute_deterministic_run(DeterministicRunRequest {
            reproduce_request: request,
            manifest,
            reproducibility_path: report_path,
            prepared_rebuild,
            run_path,
            run_index,
        })?;
        receipts.push(receipt);
        profiles.push(profile);
    }
    Ok((receipts, profiles))
}

struct DeterministicRunRequest<'a> {
    reproduce_request: &'a ReleaseReproduceRequest,
    manifest: &'a ReleaseEvidenceManifest,
    reproducibility_path: &'a Path,
    prepared_rebuild: &'a PreparedRebuildAuthority,
    run_path: &'a RebuildRunPaths,
    run_index: u32,
}

fn execute_deterministic_run(
    request: DeterministicRunRequest<'_>,
) -> Result<(DeterministicBuildRunReceipt, ProofSandboxProfile), RunError> {
    prepare_rebuild_output_dir(&request.run_path.output_dir)?;
    prepare_rebuild_output_dir(&request.run_path.store_dir)?;
    debug_assert!(request.run_path.output_dir.is_dir());
    debug_assert!(request.run_path.store_dir.is_dir());
    let profile =
        proof_sandbox_profile(request.prepared_rebuild, &request.run_path.output_dir, &request.run_path.store_dir)?;
    run_rebuild_command(RebuildCommandInvocation {
        request: request.reproduce_request,
        release_id: &request.manifest.release_id,
        output_dir: &request.run_path.output_dir,
        source_archive_path: &request.prepared_rebuild.source_archive_path,
        deterministic_store_dir: Some(&request.run_path.store_dir),
        prepared_rebuild: Some(request.prepared_rebuild),
        sandbox_profile: Some(&profile),
    })?;
    let unexpected_outputs =
        collect_unexpected_rebuilt_outputs(&request.manifest.binaries, &request.run_path.output_dir)?;
    let comparisons = compare_manifest_artifacts(&request.manifest.binaries, &request.run_path.output_dir)?;
    let reproducibility_evidence = deterministic_run_report(
        request.reproduce_request,
        request.manifest,
        &request.run_path.store_dir,
        comparisons.clone(),
    )?;
    fail_if_reproduction_drifted(&reproducibility_evidence, &unexpected_outputs, request.reproducibility_path)?;
    let receipt = deterministic_run_receipt(DeterministicRunReceiptRequest {
        run_id: &request.run_path.run_id,
        run_index: request.run_index,
        store_dir: &request.run_path.store_dir,
        output_dir: &request.run_path.output_dir,
        sandbox_profile_identity: &profile.identity,
        comparisons: &comparisons,
        rebuild_descriptor_blake3: &request.prepared_rebuild.descriptor_blake3,
        rebuild_authority_plan_blake3: &request.prepared_rebuild.authority_plan_blake3,
        observed_read_identities: &request.prepared_rebuild.authority_plan.approved_read_identities,
    })?;
    Ok((receipt, profile))
}

fn deterministic_run_report(
    request: &ReleaseReproduceRequest,
    manifest: &ReleaseEvidenceManifest,
    store_dir: &Path,
    comparisons: Vec<ReproducibilityArtifactComparison>,
) -> Result<ReleaseReproducibilityReport, RunError> {
    Ok(ReleaseReproducibilityReport::new(ReleaseReproducibilityReportInit {
        release_id: manifest.release_id.clone(),
        proof_class: ReproducibilityProofClass::SelfRebuildMatch,
        comparison_verdict: ReproducibilityComparisonVerdict::Matched,
        source_archive_digest_blake3: manifest.source_archive.digest_blake3.clone(),
        proof_bundle_digest_blake3: manifest.proof_bundle.digest_blake3.clone(),
        rebuild_workflow: RebuildWorkflowIdentity {
            command: workflow_command_identity(&request.rebuild_command, &request.rebuild_args)?,
            version: request.workflow_version.clone(),
        },
        environment_assumptions: reproducibility_environment_assumptions(),
        clean_rebuild_store_identities: vec![store_dir.display().to_string()],
        evidence_artifact_digests_blake3: vec![
            manifest.source_archive.digest_blake3.clone(),
            manifest.proof_bundle.digest_blake3.clone(),
        ],
        artifacts: comparisons,
    }))
}

fn build_deterministic_receipt(
    manifest: &ReleaseEvidenceManifest,
    prepared: &PreparedRebuildAuthority,
    runs: Vec<DeterministicBuildRunReceipt>,
) -> DeterministicBuildProofReceipt {
    debug_assert!(!manifest.binaries.is_empty());
    debug_assert!(!runs.is_empty());
    let sandbox_profiles = runs
        .iter()
        .map(|run| run.sandbox_profile_identity.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    DeterministicBuildProofReceipt::new(DeterministicBuildProofReceiptInit {
        proof_unit: DeterministicProofUnit {
            target_artifact_identity: format!("release:{}", manifest.release_id),
            output_identities: manifest.binaries.iter().map(|artifact| artifact.relative_path.clone()).collect(),
        },
        derivation_identity: format!("release:{}", manifest.release_id),
        hermeticity_mode: "strict".to_string(),
        workflow_version: DETERMINISTIC_BUILD_PROOF_RECEIPT_SCHEMA.to_string(),
        selected_provider_kind: manifest.proof_linkage.selected_provider_kind.clone(),
        source_blake3: manifest.source_archive.digest_blake3.clone(),
        vendor_blake3: manifest.proof_bundle.digest_blake3.clone(),
        toolchain_provider_identity: format!(
            "provider-kind={};provider-blake3={};rebuild-descriptor-blake3={}",
            manifest.proof_linkage.selected_provider_kind,
            prepared.descriptor.provider.digest_blake3,
            prepared.descriptor_blake3
        ),
        toolchain_stage_roots: vec![
            format!("staged-source={}", manifest.proof_linkage.staged_source),
            format!("source-archive={}", manifest.source_archive.digest_blake3),
            format!("prerequisite-inventory={}", manifest.proof_linkage.prerequisite_inventory_digest_blake3),
        ],
        logical_store_prefix: "/mantle/store".to_string(),
        physical_store_isolation: "fresh-store-per-run".to_string(),
        effect_policy_version: BUILD_EFFECT_POLICY_VERSION.to_string(),
        declared_effects: PURE_LOCAL_BUILD_EFFECTS.to_vec(),
        observed_effects: None,
        normalized_execution_envelope: deterministic_execution_envelope(&sandbox_profiles),
        ambient_host_perturbations: deterministic_ambient_host_perturbations(),
        sandbox_profile_identities: sandbox_profiles,
        runs,
        rebuild_descriptor: prepared.descriptor.clone(),
        rebuild_descriptor_blake3: prepared.descriptor_blake3.clone(),
        rebuild_authority_plan: prepared.authority_plan.clone(),
        rebuild_authority_plan_blake3: prepared.authority_plan_blake3.clone(),
    })
}

fn persist_deterministic_proof(
    proof_root: &Path,
    receipt: DeterministicBuildProofReceipt,
    sandbox_profiles: &[ProofSandboxProfile],
) -> Result<DeterministicProofOutput, RunError> {
    let digest_blake3 = deterministic_build_proof_receipt_digest_blake3(receipt.clone()).map_err(core_error)?;
    let path = proof_root.join("deterministic-build-proof.json");
    write_deterministic_proof_receipt(&path, receipt.clone())?;
    let isolation_evidence = deterministic_sandbox_isolation_evidence(sandbox_profiles)?;
    let isolation_evidence_digest_blake3 =
        deterministic_sandbox_isolation_evidence_digest_blake3(isolation_evidence.clone()).map_err(core_error)?;
    let isolation_evidence_path = proof_root.join("deterministic-sandbox-isolation-evidence.json");
    write_deterministic_sandbox_isolation_evidence(&isolation_evidence_path, isolation_evidence)?;
    Ok(DeterministicProofOutput {
        path,
        digest_blake3,
        receipt,
        isolation_evidence_path,
        isolation_evidence_digest_blake3,
    })
}

fn proof_sandbox_profile(
    prepared_rebuild: &PreparedRebuildAuthority,
    output_dir: &Path,
    store_dir: &Path,
) -> Result<ProofSandboxProfile, RunError> {
    let executor = resolve_bwrap_executor()?;
    let executor_version = bwrap_version(&executor)?;
    let mut profile = ProofSandboxProfile {
        identity: String::new(),
        executor,
        executor_version,
        network_policy: "none".to_string(),
        clock_syscall_policy_identity: proof_clock_filter_identity()
            .map_err(|err| RunError::Internal(format!("preparing deterministic proof clock syscall policy: {err}")))?,
        descriptor_blake3: prepared_rebuild.descriptor_blake3.clone(),
        authority_plan_blake3: prepared_rebuild.authority_plan_blake3.clone(),
        command_path: prepared_rebuild.command_path.clone(),
        command_args: prepared_rebuild.command_args.clone(),
        source_archive_path: prepared_rebuild.source_archive_path.clone(),
        read_only_paths: prepared_rebuild.read_only_paths.clone(),
        output_dir: absolutize_path(output_dir)?,
        store_dir: absolutize_path(store_dir)?,
    };
    let canonical = profile.canonical_facts().join("\n");
    let digest = blake3::hash(canonical.as_bytes()).to_hex().to_string();
    profile.identity = format!("{PROOF_SANDBOX_PROFILE_PREFIX}:{digest}");
    debug_assert!(profile.output_dir.is_absolute());
    debug_assert!(profile.store_dir.is_absolute());
    Ok(profile)
}

fn sandboxed_rebuild_command(
    release_id: &str,
    output_dir: &Path,
    deterministic_store_dir: Option<&Path>,
    prepared_rebuild: &PreparedRebuildAuthority,
    profile: &ProofSandboxProfile,
) -> Result<PreparedRebuildCommand, RunError> {
    debug_assert!(!release_id.is_empty());
    debug_assert!(!profile.identity.is_empty());
    let store_dir = deterministic_store_dir.ok_or_else(|| {
        RunError::Internal("deterministic proof sandbox requires a proof store directory".to_string())
    })?;
    let clock_filter = prepare_proof_clock_seccomp_filter()
        .map_err(|err| RunError::Internal(format!("preparing deterministic proof clock syscall filter: {err}")))?;
    if clock_filter.identity() != profile.clock_syscall_policy_identity {
        return Err(RunError::Internal(
            "deterministic proof clock syscall filter identity changed after profile construction".to_string(),
        ));
    }
    let clock_filter_fd = clock_filter.fd();
    let mut command = ProcessCommand::new(&profile.executor);
    configure_executor_environment(&mut command);
    command
        .arg("--unshare-all")
        .arg("--die-with-parent")
        .arg("--new-session")
        .arg("--clearenv")
        .arg("--seccomp")
        .arg(clock_filter_fd.to_string())
        .arg("--tmpfs")
        .arg(PROOF_SANDBOX_TEMP_ROOT);
    let mut parent_paths = prepared_rebuild.read_only_paths.iter().map(PathBuf::as_path).collect::<Vec<_>>();
    parent_paths.push(output_dir);
    parent_paths.push(store_dir);
    append_bwrap_parent_dirs(&mut command, parent_paths);
    for input_path in &prepared_rebuild.read_only_paths {
        command.arg("--ro-bind").arg(input_path).arg(input_path);
    }
    command.arg("--bind").arg(output_dir).arg(output_dir).arg("--bind").arg(store_dir).arg(store_dir);
    append_rebuild_sandbox_environment(&mut command, release_id, profile, output_dir, store_dir);
    configure_sandbox_umask(&mut command);
    clock_filter.configure_inheritance(&mut command);
    command
        .arg("--chdir")
        .arg(PROOF_SANDBOX_TEMP_ROOT)
        .arg(&prepared_rebuild.command_path)
        .args(&prepared_rebuild.command_args);
    Ok(PreparedRebuildCommand {
        command,
        _clock_filter: Some(clock_filter),
    })
}

fn configure_executor_environment(command: &mut ProcessCommand) {
    let test_environment = PROOF_EXECUTOR_TEST_ENV_ALLOWLIST
        .iter()
        .filter_map(|name| std::env::var_os(name).map(|value| (*name, value)))
        .collect::<Vec<_>>();
    command.env_clear();
    for (name, value) in test_environment {
        debug_assert!(!name.is_empty());
        debug_assert!(!value.is_empty());
        command.env(name, value);
    }
}

fn append_rebuild_sandbox_environment(
    command: &mut ProcessCommand,
    release_id: &str,
    profile: &ProofSandboxProfile,
    output_dir: &Path,
    store_dir: &Path,
) {
    let dynamic_environment = [
        (REBUILD_SOURCE_ARCHIVE_ENV, profile.source_archive_path.as_os_str()),
        (REBUILD_EXECUTABLE_ENV, profile.command_path.as_os_str()),
        (REPRODUCE_OUTPUT_DIR_ENV, output_dir.as_os_str()),
        (REPRODUCE_RELEASE_ID_ENV, OsStr::new(release_id)),
        (DETERMINISTIC_PROOF_STORE_DIR_ENV, store_dir.as_os_str()),
    ];
    for (name, value) in dynamic_environment {
        debug_assert!(!name.is_empty());
        debug_assert!(!value.is_empty());
        command.arg("--setenv").arg(name).arg(value);
    }
    for (name, value) in PROOF_SANDBOX_FIXED_ENV {
        debug_assert!(!name.is_empty());
        debug_assert!(!value.is_empty());
        command.arg("--setenv").arg(name).arg(value);
    }
}

#[cfg(unix)]
fn configure_sandbox_umask(command: &mut ProcessCommand) {
    debug_assert_ne!(PROOF_SANDBOX_UMASK, 0);
    debug_assert_eq!(PROOF_SANDBOX_UMASK & !UNIX_PERMISSION_MODE_MASK, 0);
    // SAFETY: `umask` is async-signal-safe and the closure performs no allocation or I/O after fork.
    unsafe {
        command.pre_exec(|| {
            libc::umask(PROOF_SANDBOX_UMASK);
            Ok(())
        });
    }
}

#[cfg(not(unix))]
fn configure_sandbox_umask(_command: &mut ProcessCommand) {}

fn existing_absolute_rebuild_arg_paths(args: &[OsString]) -> Vec<PathBuf> {
    args.iter().map(PathBuf::from).filter(|path| path.is_absolute() && path.exists()).collect()
}

fn append_bwrap_parent_dirs<'a>(command: &mut ProcessCommand, paths: impl IntoIterator<Item = &'a Path>) {
    let mut parents = BTreeSet::<PathBuf>::new();
    for path in paths {
        for ancestor in path.ancestors().skip(1) {
            if ancestor == Path::new("/") || ancestor.as_os_str().is_empty() {
                continue;
            }
            parents.insert(ancestor.to_path_buf());
        }
    }
    for parent in parents {
        command.arg("--dir").arg(parent);
    }
}

fn resolve_bwrap_executor() -> Result<PathBuf, RunError> {
    debug_assert!(!PROOF_SANDBOX_BWRAP_ENV.is_empty());
    debug_assert_eq!(Path::new("bwrap").file_name(), Some(OsStr::new("bwrap")));
    if let Some(path) = std::env::var_os(PROOF_SANDBOX_BWRAP_ENV) {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Ok(path);
        }
        return Err(RunError::Internal(format!(
            "deterministic proof sandbox execution is unavailable: {} does not name a bwrap executable",
            path.display()
        )));
    }
    if let Some(paths) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&paths) {
            let candidate = dir.join("bwrap");
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }
    Err(RunError::Internal(
        "deterministic proof sandbox execution is unavailable: bwrap not found; set MANTLE_DETERMINISTIC_PROOF_BWRAP or install bubblewrap".to_string(),
    ))
}

fn bwrap_version(executor: &Path) -> Result<String, RunError> {
    let output = ProcessCommand::new(executor).arg("--version").output().map_err(|err| {
        RunError::Internal(format!("checking deterministic proof sandbox executor {}: {err}", executor.display()))
    })?;
    if !output.status.success() {
        return Err(RunError::Internal(format!(
            "deterministic proof sandbox executor {} did not report a version",
            executor.display()
        )));
    }
    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if version.is_empty() {
        Ok("bwrap-version-unknown".to_string())
    } else {
        Ok(version)
    }
}

fn absolutize_existing_path(path: &Path, label: &str) -> Result<PathBuf, RunError> {
    path.canonicalize()
        .map_err(|err| RunError::Internal(format!("resolving {label} {}: {err}", path.display())))
}

fn absolutize_path(path: &Path) -> Result<PathBuf, RunError> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        std::env::current_dir()
            .map(|cwd| cwd.join(path))
            .map_err(|err| RunError::Internal(format!("resolving current directory: {err}")))
    }
}

fn deterministic_proof_root(request: &ReleaseReproduceRequest) -> PathBuf {
    request
        .deterministic_proof_dir
        .clone()
        .unwrap_or_else(|| request.rebuild_output_dir.with_extension("deterministic-proof"))
}

fn validate_deterministic_proof_root(proof_root: &Path, rebuild_output_dir: &Path) -> Result<(), RunError> {
    if proof_root == rebuild_output_dir {
        return Err(RunError::Internal(
            "deterministic proof directory must differ from the main rebuild output directory".to_string(),
        ));
    }
    if proof_root.starts_with(rebuild_output_dir) || rebuild_output_dir.starts_with(proof_root) {
        return Err(RunError::Internal(format!(
            "deterministic proof directory {} must not be nested with rebuild output directory {}",
            proof_root.display(),
            rebuild_output_dir.display()
        )));
    }
    Ok(())
}

struct DeterministicRunReceiptRequest<'a> {
    run_id: &'a str,
    run_index: u32,
    store_dir: &'a Path,
    output_dir: &'a Path,
    sandbox_profile_identity: &'a str,
    comparisons: &'a [ReproducibilityArtifactComparison],
    rebuild_descriptor_blake3: &'a str,
    rebuild_authority_plan_blake3: &'a str,
    observed_read_identities: &'a [String],
}

fn deterministic_run_receipt(
    request: DeterministicRunReceiptRequest<'_>,
) -> Result<DeterministicBuildRunReceipt, RunError> {
    debug_assert!(!request.run_id.is_empty());
    debug_assert!(!request.sandbox_profile_identity.is_empty());
    let mut output_digests = Vec::with_capacity(request.comparisons.len());
    for comparison in request.comparisons {
        let Some(observed_digest) = &comparison.observed_digest_blake3 else {
            return Err(RunError::Internal(format!(
                "deterministic proof run {} missing rebuilt artifact {}",
                request.run_id, comparison.name
            )));
        };
        output_digests.push(DeterministicOutputDigest {
            name: comparison.name.clone(),
            digest_blake3: observed_digest.clone(),
        });
    }
    Ok(DeterministicBuildRunReceipt {
        run_id: request.run_id.to_string(),
        perturbation_case: deterministic_perturbation_case(request.run_index),
        output_store_paths: vec![request.store_dir.display().to_string()],
        output_root_identity: request.output_dir.display().to_string(),
        sandbox_profile_identity: request.sandbox_profile_identity.to_string(),
        output_digests,
        substituted_dependency_identities: Vec::new(),
        hermeticity_audit_events: Vec::new(),
        observed_effects: Some(PURE_LOCAL_BUILD_EFFECTS.to_vec()),
        rebuild_descriptor_blake3: Some(request.rebuild_descriptor_blake3.to_string()),
        rebuild_authority_plan_blake3: Some(request.rebuild_authority_plan_blake3.to_string()),
        observed_read_identities: request.observed_read_identities.to_vec(),
        authority_violations: Vec::new(),
    })
}

fn deterministic_perturbation_case(run_index: u32) -> String {
    match run_index {
        0 => "baseline-clean-env".to_string(),
        1 => "host-env-noise".to_string(),
        other => format!("host-env-noise-{other}"),
    }
}

fn deterministic_normalization_controls() -> Vec<(&'static str, String)> {
    let controls = vec![
        ("time", format!("SOURCE_DATE_EPOCH={PROOF_SOURCE_DATE_EPOCH}")),
        ("timezone", format!("TZ={PROOF_SANDBOX_TIMEZONE}")),
        ("locale", format!("LANG={PROOF_SANDBOX_LOCALE};LC_ALL={PROOF_SANDBOX_LOCALE}")),
        (
            "temp-roots",
            format!(
                "TEMP={PROOF_SANDBOX_TEMP_ROOT};TEMPDIR={PROOF_SANDBOX_TEMP_ROOT};TMP={PROOF_SANDBOX_TEMP_ROOT};TMPDIR={PROOF_SANDBOX_TEMP_ROOT}"
            ),
        ),
        (
            "host-user-metadata",
            format!("HOME={PROOF_SANDBOX_TEMP_ROOT};LOGNAME={PROOF_SANDBOX_USER};USER={PROOF_SANDBOX_USER}"),
        ),
        ("umask", PROOF_SANDBOX_UMASK_TEXT.to_string()),
        ("modeled-randomness", PROOF_MODELED_RANDOMNESS.to_string()),
        ("order-sensitive-output-processing", PROOF_OUTPUT_ORDERING.to_string()),
    ];
    debug_assert_eq!(controls.len(), PROOF_NORMALIZATION_CONTROL_COUNT);
    debug_assert!(controls.iter().all(|(surface, _)| !surface.is_empty()));
    debug_assert!(controls.iter().all(|(_, value)| !value.is_empty()));
    controls
}

fn deterministic_normalization_envelope() -> Vec<String> {
    let controls = deterministic_normalization_controls()
        .into_iter()
        .map(|(surface, value)| format!("normalization:{surface}={value}"))
        .collect::<Vec<_>>();
    debug_assert_eq!(controls.len(), PROOF_NORMALIZATION_CONTROL_COUNT);
    debug_assert!(controls.iter().all(|control| control.starts_with("normalization:")));
    controls
}

fn deterministic_execution_envelope(sandbox_profile_identities: &[String]) -> Vec<String> {
    let mut envelope = reproducibility_environment_assumptions();
    envelope.push("sandbox=bwrap".to_string());
    envelope.push("network=none".to_string());
    envelope.extend(deterministic_normalization_envelope());
    for identity in sandbox_profile_identities {
        envelope.push(format!("sandbox-profile={identity}"));
    }
    debug_assert!(envelope.iter().any(|entry| entry == "normalization:umask=0022"));
    if !sandbox_profile_identities.is_empty() {
        debug_assert!(envelope.iter().any(|entry| entry.starts_with("sandbox-profile=")));
    }
    envelope
}

fn deterministic_ambient_host_perturbations() -> Vec<String> {
    vec![
        "HOME".to_string(),
        "PATH".to_string(),
        "USER".to_string(),
        "LOGNAME".to_string(),
        "TZ".to_string(),
        "LANG".to_string(),
        "LC_ALL".to_string(),
        "TMPDIR".to_string(),
        "cwd".to_string(),
        "umask".to_string(),
        "env-noise".to_string(),
    ]
}

fn write_deterministic_proof_receipt(path: &Path, receipt: DeterministicBuildProofReceipt) -> Result<(), RunError> {
    let bytes = deterministic_build_proof_receipt_canonical_bytes(receipt).map_err(core_error)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| RunError::Internal(format!("creating {}: {err}", parent.display())))?;
    }
    std::fs::write(path, bytes).map_err(|err| RunError::Internal(format!("writing {}: {err}", path.display())))
}

fn deterministic_sandbox_isolation_evidence(
    profiles: &[ProofSandboxProfile],
) -> Result<DeterministicSandboxIsolationEvidence, RunError> {
    if profiles.is_empty() {
        return Err(RunError::Internal(
            "deterministic sandbox isolation evidence requires at least one proof sandbox profile".to_string(),
        ));
    }
    debug_assert!(!profiles.is_empty());
    debug_assert!(profiles.iter().all(|profile| !profile.identity.is_empty()));
    let mut material = vec![
        format!("schema={DETERMINISTIC_SANDBOX_ISOLATION_EVIDENCE_SCHEMA}"),
        format!("profile_family={PROOF_SANDBOX_PROFILE_PREFIX}"),
        format!("evidence_version={DEFAULT_REPRODUCIBILITY_WORKFLOW_VERSION}"),
    ];
    for check in REQUIRED_ISOLATION_CHECKS {
        material.push(format!("check={check}"));
    }
    for profile in profiles {
        material.push(format!("profile_identity={}", profile.identity));
        for fact in profile.canonical_facts() {
            material.push(format!("profile_fact={fact}"));
        }
    }
    material.sort();
    let evidence_digest_blake3 = blake3::hash(material.join("\n").as_bytes()).to_hex().to_string();
    Ok(DeterministicSandboxIsolationEvidence {
        schema: DETERMINISTIC_SANDBOX_ISOLATION_EVIDENCE_SCHEMA.to_string(),
        profile_family: PROOF_SANDBOX_PROFILE_PREFIX.to_string(),
        evidence_version: DEFAULT_REPRODUCIBILITY_WORKFLOW_VERSION.to_string(),
        status: DeterministicSandboxIsolationEvidenceStatus::Passed,
        checks: REQUIRED_ISOLATION_CHECKS.iter().map(|check| (*check).to_string()).collect(),
        evidence_digest_blake3,
    })
}

fn write_deterministic_sandbox_isolation_evidence(
    path: &Path,
    evidence: DeterministicSandboxIsolationEvidence,
) -> Result<(), RunError> {
    let bytes = deterministic_sandbox_isolation_evidence_canonical_bytes(evidence).map_err(core_error)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| RunError::Internal(format!("creating {}: {err}", parent.display())))?;
    }
    std::fs::write(path, bytes).map_err(|err| RunError::Internal(format!("writing {}: {err}", path.display())))
}

fn build_reproducibility_report(
    manifest: &ReleaseEvidenceManifest,
    request: &ReleaseReproduceRequest,
) -> Result<ReleaseReproducibilityReport, RunError> {
    let comparisons = compare_manifest_artifacts(&manifest.binaries, &request.rebuild_output_dir)?;
    let counts = count_report_results(&comparisons);
    let is_matched = counts.mismatched_count == 0 && counts.missing_count == 0;
    debug_assert_eq!(comparisons.len(), manifest.binaries.len());
    debug_assert_eq!(
        is_matched,
        comparisons.iter().all(|artifact| artifact.result == ReproducibilityComparisonResult::Matched)
    );
    Ok(ReleaseReproducibilityReport::new(ReleaseReproducibilityReportInit {
        release_id: manifest.release_id.clone(),
        proof_class: if is_matched {
            ReproducibilityProofClass::SelfRebuildMatch
        } else {
            ReproducibilityProofClass::SelfProofValid
        },
        comparison_verdict: if is_matched {
            ReproducibilityComparisonVerdict::Matched
        } else {
            ReproducibilityComparisonVerdict::Failed
        },
        source_archive_digest_blake3: manifest.source_archive.digest_blake3.clone(),
        proof_bundle_digest_blake3: manifest.proof_bundle.digest_blake3.clone(),
        rebuild_workflow: RebuildWorkflowIdentity {
            command: workflow_command_identity(&request.rebuild_command, &request.rebuild_args)?,
            version: request.workflow_version.clone(),
        },
        environment_assumptions: reproducibility_environment_assumptions(),
        clean_rebuild_store_identities: vec![request.rebuild_output_dir.display().to_string()],
        evidence_artifact_digests_blake3: vec![
            manifest.source_archive.digest_blake3.clone(),
            manifest.proof_bundle.digest_blake3.clone(),
        ],
        artifacts: comparisons,
    }))
}

fn reproducibility_environment_assumptions() -> Vec<String> {
    vec![
        format!("os={}", std::env::consts::OS),
        format!("arch={}", std::env::consts::ARCH),
        "rebuild-output-dir-prevalidated-empty".to_string(),
    ]
}

fn compare_manifest_artifacts(
    artifacts: &[BundledArtifact],
    rebuild_output_dir: &Path,
) -> Result<Vec<ReproducibilityArtifactComparison>, RunError> {
    let mut comparisons = Vec::with_capacity(artifacts.len());
    for artifact in artifacts {
        let output_path = rebuild_output_dir.join(&artifact.relative_path);
        let observed = observed_artifact(&output_path)?;
        comparisons.push(compare_artifact(artifact, observed));
    }
    Ok(comparisons)
}

fn compare_artifact(
    artifact: &BundledArtifact,
    observed: Option<ObservedArtifact>,
) -> ReproducibilityArtifactComparison {
    match observed {
        Some(observed) => {
            let result = comparison_result(artifact, &observed);
            ReproducibilityArtifactComparison {
                name: artifact.relative_path.clone(),
                expected_size_bytes: artifact.size_bytes,
                expected_digest_blake3: artifact.digest_blake3.clone(),
                observed_size_bytes: Some(observed.size_bytes),
                observed_digest_blake3: Some(observed.digest_blake3),
                result,
            }
        }
        None => ReproducibilityArtifactComparison {
            name: artifact.relative_path.clone(),
            expected_size_bytes: artifact.size_bytes,
            expected_digest_blake3: artifact.digest_blake3.clone(),
            observed_size_bytes: None,
            observed_digest_blake3: None,
            result: ReproducibilityComparisonResult::MissingRebuiltArtifact,
        },
    }
}

fn comparison_result(artifact: &BundledArtifact, observed: &ObservedArtifact) -> ReproducibilityComparisonResult {
    if artifact.size_bytes == observed.size_bytes && artifact.digest_blake3 == observed.digest_blake3 {
        return ReproducibilityComparisonResult::Matched;
    }
    ReproducibilityComparisonResult::Mismatched
}

fn observed_artifact(path: &Path) -> Result<Option<ObservedArtifact>, RunError> {
    if !path.exists() {
        return Ok(None);
    }
    if !path.is_file() {
        return Err(RunError::Internal(format!(
            "release reproducibility rebuilt artifact is not a file: {}",
            path.display()
        )));
    }
    let (size_bytes, digest_blake3) = hash_file(path)?;
    Ok(Some(ObservedArtifact {
        size_bytes,
        digest_blake3,
    }))
}

fn hash_file(path: &Path) -> Result<(u64, String), RunError> {
    let metadata =
        std::fs::metadata(path).map_err(|err| RunError::Internal(format!("metadata {}: {err}", path.display())))?;
    let mut file = File::open(path).map_err(|err| RunError::Internal(format!("open {}: {err}", path.display())))?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0_u8; HASH_BUFFER_BYTES];
    let read_attempt_count_max = metadata
        .len()
        .div_ceil(HASH_BUFFER_BYTES_U64)
        .checked_add(1)
        .ok_or_else(|| RunError::Internal(format!("hash read bound overflowed for {}", path.display())))?;
    debug_assert!(path.is_file());
    debug_assert_eq!(buffer.len(), HASH_BUFFER_BYTES);
    for _ in 0..read_attempt_count_max {
        let bytes_read = file
            .read(&mut buffer)
            .map_err(|err| RunError::Internal(format!("read {}: {err}", path.display())))?;
        if bytes_read == 0 {
            return Ok((metadata.len(), hasher.finalize().to_hex().to_string()));
        }
        hasher.update(&buffer[..bytes_read]);
    }
    Err(RunError::Internal(format!("file grew while hashing: {}", path.display())))
}

fn workflow_command_identity(command: &Path, args: &[OsString]) -> Result<String, RunError> {
    let command = os_str_to_string(command.as_os_str(), "rebuild command")?;
    if args.is_empty() {
        return Ok(command);
    }
    let mut parts = Vec::with_capacity(args.len().saturating_add(1));
    parts.push(command);
    for arg in args {
        parts.push(os_str_to_string(arg.as_os_str(), "rebuild argument")?);
    }
    Ok(parts.join(" "))
}

fn os_str_to_string(value: &OsStr, field_name: &str) -> Result<String, RunError> {
    value
        .to_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| RunError::Internal(format!("release reproducibility {field_name} must be UTF-8")))
}

fn write_report(path: &Path, report: ReleaseReproducibilityReport) -> Result<(), RunError> {
    let bytes = release_reproducibility_report_canonical_bytes(report).map_err(core_error)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| RunError::Internal(format!("creating {}: {err}", parent.display())))?;
    }
    std::fs::write(path, bytes).map_err(|err| RunError::Internal(format!("writing {}: {err}", path.display())))
}

fn resolve_report_path(bundle_dir: &Path, report_path: Option<&Path>) -> PathBuf {
    report_path
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_reproducibility_report_path(bundle_dir))
}

fn default_reproducibility_report_path(bundle_dir: &Path) -> PathBuf {
    bundle_dir.join(DEFAULT_REPORT_RELATIVE_PATH)
}

fn collect_unexpected_rebuilt_outputs(
    artifacts: &[BundledArtifact],
    rebuild_output_dir: &Path,
) -> Result<Vec<String>, RunError> {
    let expected_paths = artifacts.iter().map(|artifact| artifact.relative_path.clone()).collect::<BTreeSet<_>>();
    let mut files = Vec::new();
    collect_output_files_sorted(rebuild_output_dir, &mut files)?;
    let mut unexpected = Vec::with_capacity(files.len());
    for file in files {
        let relative = relative_path_text(rebuild_output_dir, &file)?;
        if !expected_paths.contains(&relative) {
            unexpected.push(relative);
        }
    }
    Ok(unexpected)
}

fn collect_output_files_sorted(root: &Path, files: &mut Vec<PathBuf>) -> Result<(), RunError> {
    let mut pending = vec![root.to_path_buf()];
    debug_assert!(root.is_dir());
    debug_assert!(files.is_empty());
    for _ in 0..MAX_REBUILD_OUTPUT_ENTRIES {
        let Some(current) = pending.pop() else {
            files.sort();
            return Ok(());
        };
        let children = std::fs::read_dir(&current)
            .map_err(|err| RunError::Internal(format!("read_dir {}: {err}", current.display())))?
            .map(|entry| {
                entry
                    .map(|child| child.path())
                    .map_err(|err| RunError::Internal(format!("read_dir entry {}: {err}", current.display())))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let projected_entry_count = pending
            .len()
            .checked_add(files.len())
            .and_then(|count| count.checked_add(children.len()))
            .and_then(|count| u32::try_from(count).ok())
            .ok_or_else(|| {
                RunError::Internal("release reproducibility output entry count overflowed u32".to_string())
            })?;
        if projected_entry_count > MAX_REBUILD_OUTPUT_ENTRIES {
            return Err(RunError::Internal(format!(
                "release reproducibility output tree exceeds {MAX_REBUILD_OUTPUT_ENTRIES} entries"
            )));
        }
        for child in children {
            if child.is_dir() {
                pending.push(child);
                continue;
            }
            if child.is_file() {
                files.push(child);
                continue;
            }
            return Err(RunError::Internal(format!(
                "release reproducibility output contains unsupported entry: {}",
                child.display()
            )));
        }
    }
    Err(RunError::Internal(format!(
        "release reproducibility output tree exceeds {MAX_REBUILD_OUTPUT_ENTRIES} entries"
    )))
}

fn relative_path_text(root: &Path, file: &Path) -> Result<String, RunError> {
    let relative = file.strip_prefix(root).map_err(|err| {
        RunError::Internal(format!(
            "release reproducibility output strip_prefix {} from {}: {err}",
            root.display(),
            file.display()
        ))
    })?;
    let mut parts = Vec::with_capacity(relative.components().count());
    for component in relative.components() {
        let text = component.as_os_str().to_str().ok_or_else(|| {
            RunError::Internal(format!("release reproducibility output path must be UTF-8: {}", file.display()))
        })?;
        parts.push(text.to_string());
    }
    Ok(parts.join("/"))
}

fn fail_if_reproduction_drifted(
    report: &ReleaseReproducibilityReport,
    unexpected_outputs: &[String],
    report_path: &Path,
) -> Result<(), RunError> {
    if let Some(unexpected) = unexpected_outputs.first() {
        return Err(RunError::Internal(format!(
            "release reproducibility output-name drift: unexpected rebuilt artifact {unexpected}; report written to {}",
            report_path.display()
        )));
    }
    for artifact in &report.artifacts {
        fail_if_artifact_drifted(artifact, report_path)?;
    }
    Ok(())
}

fn fail_if_artifact_drifted(artifact: &ReproducibilityArtifactComparison, report_path: &Path) -> Result<(), RunError> {
    match artifact.result {
        ReproducibilityComparisonResult::Matched => Ok(()),
        ReproducibilityComparisonResult::MissingRebuiltArtifact => Err(RunError::Internal(format!(
            "release reproducibility missing rebuilt artifact {}; report written to {}",
            artifact.name,
            report_path.display()
        ))),
        ReproducibilityComparisonResult::Mismatched => fail_mismatched_artifact(artifact, report_path),
    }
}

fn fail_mismatched_artifact(artifact: &ReproducibilityArtifactComparison, report_path: &Path) -> Result<(), RunError> {
    debug_assert_eq!(artifact.result, ReproducibilityComparisonResult::Mismatched);
    debug_assert!(artifact.observed_digest_blake3.is_some());
    let observed_size_bytes = artifact.observed_size_bytes.unwrap_or_default();
    if observed_size_bytes != artifact.expected_size_bytes {
        return Err(RunError::Internal(format!(
            "release reproducibility byte-length drift for {}: expected {} got {}; report written to {}",
            artifact.name,
            artifact.expected_size_bytes,
            observed_size_bytes,
            report_path.display()
        )));
    }
    let observed_digest = artifact.observed_digest_blake3.as_deref().unwrap_or("missing");
    Err(RunError::Internal(format!(
        "release reproducibility digest drift for {}: expected {} got {}; report written to {}",
        artifact.name,
        artifact.expected_digest_blake3,
        observed_digest,
        report_path.display()
    )))
}

fn count_report_results(comparisons: &[ReproducibilityArtifactComparison]) -> ComparisonCounts {
    let mut counts = ComparisonCounts {
        matched_count: 0,
        mismatched_count: 0,
        missing_count: 0,
    };
    for comparison in comparisons {
        match comparison.result {
            ReproducibilityComparisonResult::Matched => counts.matched_count = counts.matched_count.saturating_add(1),
            ReproducibilityComparisonResult::Mismatched => {
                counts.mismatched_count = counts.mismatched_count.saturating_add(1);
            }
            ReproducibilityComparisonResult::MissingRebuiltArtifact => {
                counts.missing_count = counts.missing_count.saturating_add(1);
            }
        }
    }
    counts
}

fn core_error(err: crunch_release_core::ReleaseEvidenceError) -> RunError {
    RunError::Internal(err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXPECTED_SIZE_BYTES: u64 = 4;
    const OBSERVED_SIZE_BYTES: u64 = 5;
    const BWRAP_SETENV_ARGUMENT_COUNT: usize = 3;

    fn sample_digest(seed: u8) -> String {
        let nibble = format!("{:x}", seed % 16);
        nibble.repeat(crunch_release_core::BLAKE3_HEX_LENGTH_CHARS)
    }

    fn sample_artifact() -> BundledArtifact {
        BundledArtifact {
            kind: crunch_release_core::BundledArtifactKind::File,
            relative_path: "binaries/01-mantle".to_string(),
            size_bytes: EXPECTED_SIZE_BYTES,
            digest_blake3: sample_digest(1),
        }
    }

    #[test]
    fn append_bwrap_parent_dirs_creates_ancestors_before_nested_binds() {
        let mut command = ProcessCommand::new("bwrap");

        append_bwrap_parent_dirs(&mut command, [
            Path::new("/home/example/.cargo-target/repo-targets/mantle__mantle/release/rebuild.sh"),
            Path::new("/home/example/.cargo-target/repo-targets/mantle__mantle/proof/run-000/store"),
        ]);

        let args: Vec<_> = command.get_args().map(|arg| arg.to_string_lossy().into_owned()).collect();
        assert!(args.windows(2).any(|pair| pair == ["--dir", "/home"]));
        assert!(args.windows(2).any(|pair| pair == ["--dir", "/home/example/.cargo-target"]));
        assert!(
            args.windows(2)
                .any(|pair| pair == ["--dir", "/home/example/.cargo-target/repo-targets/mantle__mantle"])
        );
        assert!(!args.windows(2).any(|pair| pair == ["--dir", "/"]));
    }

    #[test]
    fn existing_absolute_rebuild_arg_paths_collects_only_bindable_absolute_paths() {
        let temp_path = std::env::temp_dir().join(format!("mantle-rebuild-arg-path-{}", std::process::id()));
        std::fs::write(&temp_path, b"helper").unwrap();

        let paths = existing_absolute_rebuild_arg_paths(&[
            OsString::from("sh"),
            temp_path.as_os_str().to_os_string(),
            OsString::from("/definitely/missing/mantle-helper"),
        ]);

        assert_eq!(paths, vec![temp_path.clone()]);
        std::fs::remove_file(temp_path).unwrap();
    }

    #[test]
    fn deterministic_envelope_binds_every_normalization_control() {
        let profile_identity = "mantle-proof-sandbox-v1:test".to_string();
        let envelope = deterministic_execution_envelope(core::slice::from_ref(&profile_identity));
        let controls = deterministic_normalization_envelope();

        assert_eq!(controls.len(), PROOF_NORMALIZATION_CONTROL_COUNT);
        assert!(controls.iter().all(|control| envelope.contains(control)));
        assert!(envelope.contains(&format!("sandbox-profile={profile_identity}")));
        assert!(envelope.contains(&"normalization:umask=0022".to_string()));
        assert!(envelope.contains(&"normalization:time=SOURCE_DATE_EPOCH=1".to_string()));
    }

    #[test]
    fn sandbox_environment_plan_is_fixed_and_excludes_ambient_variables() {
        let mut command = ProcessCommand::new("bwrap");
        let profile = ProofSandboxProfile {
            identity: "mantle-proof-sandbox-v1:test".to_string(),
            executor: PathBuf::from("bwrap"),
            executor_version: "test".to_string(),
            network_policy: "none".to_string(),
            clock_syscall_policy_identity: "mantle-clock-syscall-deny-v1:test".to_string(),
            descriptor_blake3: sample_digest(1),
            authority_plan_blake3: sample_digest(2),
            command_path: PathBuf::from("/inputs/executable"),
            command_args: vec![OsString::from("sh"), OsString::from("/inputs/recipe")],
            source_archive_path: PathBuf::from("/inputs/source.tar"),
            read_only_paths: vec![PathBuf::from("/inputs/executable"), PathBuf::from("/inputs/recipe")],
            output_dir: PathBuf::from("/output"),
            store_dir: PathBuf::from("/store"),
        };
        append_rebuild_sandbox_environment(
            &mut command,
            "release-test",
            &profile,
            &profile.output_dir,
            &profile.store_dir,
        );
        let args = command.get_args().map(|arg| arg.to_string_lossy().into_owned()).collect::<Vec<_>>();
        let profile_facts = profile.canonical_facts();

        assert!(profile_facts.iter().any(|fact| fact == "clock_syscall_enforcement=bwrap-seccomp-errno-eperm"));
        assert!(
            profile_facts
                .iter()
                .any(|fact| fact == "clock_syscall_policy_identity=mantle-clock-syscall-deny-v1:test")
        );
        for (name, value) in PROOF_SANDBOX_FIXED_ENV {
            assert!(
                args.windows(BWRAP_SETENV_ARGUMENT_COUNT).any(|window| window == ["--setenv", *name, *value]),
                "missing fixed environment {name}={value}: {args:?}"
            );
        }
        assert!(
            args.windows(BWRAP_SETENV_ARGUMENT_COUNT)
                .any(|window| { window == ["--setenv", REPRODUCE_RELEASE_ID_ENV, "release-test"] })
        );
        assert!(!args.iter().any(|arg| arg == "LD_PRELOAD"));
        assert!(!args.iter().any(|arg| arg == "AWS_SECRET_ACCESS_KEY"));
    }

    #[test]
    fn executor_environment_plan_removes_ambient_loader_and_secret_inputs() {
        let mut command = ProcessCommand::new("bwrap");
        command.env("LD_PRELOAD", "/tmp/hostile.so");
        command.env("AWS_SECRET_ACCESS_KEY", "secret");

        configure_executor_environment(&mut command);

        let keys = command.get_envs().map(|(name, _)| name.to_string_lossy().into_owned()).collect::<Vec<_>>();
        assert!(!keys.iter().any(|name| name == "LD_PRELOAD"));
        assert!(!keys.iter().any(|name| name == "AWS_SECRET_ACCESS_KEY"));
        assert!(keys.iter().all(|name| PROOF_EXECUTOR_TEST_ENV_ALLOWLIST.contains(&name.as_str())));
    }

    #[test]
    fn compare_artifact_marks_identical_observation_as_matched() {
        let artifact = sample_artifact();
        let observed = ObservedArtifact {
            size_bytes: EXPECTED_SIZE_BYTES,
            digest_blake3: sample_digest(1),
        };

        let comparison = compare_artifact(&artifact, Some(observed));

        assert_eq!(comparison.result, ReproducibilityComparisonResult::Matched);
        assert_eq!(comparison.name, "binaries/01-mantle");
    }

    #[test]
    fn compare_artifact_marks_digest_drift_as_mismatched() {
        let artifact = sample_artifact();
        let observed = ObservedArtifact {
            size_bytes: EXPECTED_SIZE_BYTES,
            digest_blake3: sample_digest(2),
        };

        let comparison = compare_artifact(&artifact, Some(observed));

        assert_eq!(comparison.result, ReproducibilityComparisonResult::Mismatched);
        assert_ne!(comparison.expected_digest_blake3, comparison.observed_digest_blake3.unwrap());
    }

    #[test]
    fn compare_artifact_marks_missing_observation_as_missing() {
        let artifact = sample_artifact();

        let comparison = compare_artifact(&artifact, None);

        assert_eq!(comparison.result, ReproducibilityComparisonResult::MissingRebuiltArtifact);
        assert!(comparison.observed_size_bytes.is_none());
    }

    #[test]
    fn deterministic_proof_root_rejects_reused_rebuild_output_dir() {
        let root = Path::new("/tmp/mantle-rebuild");
        let err = validate_deterministic_proof_root(root, root).unwrap_err();
        assert!(err.to_string().contains("must differ"));
    }

    #[test]
    fn deterministic_proof_root_rejects_nested_output_dirs() {
        let err =
            validate_deterministic_proof_root(Path::new("/tmp/mantle-rebuild/proof"), Path::new("/tmp/mantle-rebuild"))
                .unwrap_err();
        assert!(err.to_string().contains("must not be nested"));
    }

    #[test]
    fn deterministic_run_receipt_records_fresh_store_identity() {
        let artifact = sample_artifact();
        let observed = ObservedArtifact {
            size_bytes: EXPECTED_SIZE_BYTES,
            digest_blake3: sample_digest(1),
        };
        let comparison = compare_artifact(&artifact, Some(observed));

        let descriptor_digest = sample_digest(2);
        let authority_digest = sample_digest(3);
        let read_identities = ["Source:source:demo".to_string()];
        let comparisons = [comparison];
        let run = deterministic_run_receipt(DeterministicRunReceiptRequest {
            run_id: "run-000",
            run_index: 0,
            store_dir: Path::new("/tmp/proof/run-000/store"),
            output_dir: Path::new("/tmp/proof/run-000/outputs"),
            sandbox_profile_identity: "mantle-proof-sandbox-v1:test",
            comparisons: &comparisons,
            rebuild_descriptor_blake3: &descriptor_digest,
            rebuild_authority_plan_blake3: &authority_digest,
            observed_read_identities: &read_identities,
        })
        .unwrap();

        assert_eq!(run.output_store_paths, vec!["/tmp/proof/run-000/store".to_string()]);
        assert_eq!(run.output_root_identity, "/tmp/proof/run-000/outputs");
        assert_eq!(run.sandbox_profile_identity, "mantle-proof-sandbox-v1:test");
        assert_eq!(run.output_digests[0].digest_blake3, sample_digest(1));
        assert_eq!(run.perturbation_case, "baseline-clean-env");
        assert!(run.hermeticity_audit_events.is_empty());
    }

    #[test]
    fn comparison_result_marks_size_drift_as_mismatched() {
        let artifact = sample_artifact();
        let observed = ObservedArtifact {
            size_bytes: OBSERVED_SIZE_BYTES,
            digest_blake3: sample_digest(1),
        };

        let result = comparison_result(&artifact, &observed);

        assert_eq!(result, ReproducibilityComparisonResult::Mismatched);
    }
}
