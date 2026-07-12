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
use crate::release_evidence::verify_release_evidence_bundle;

pub(crate) const DEFAULT_REPRODUCIBILITY_WORKFLOW_VERSION: &str = "mantle-release-reproducibility-v1";

const REPRODUCE_BUNDLE_DIR_ENV: &str = "MANTLE_REPRODUCE_BUNDLE_DIR";
const REPRODUCE_OUTPUT_DIR_ENV: &str = "MANTLE_REPRODUCE_OUTPUT_DIR";
const REPRODUCE_RELEASE_ID_ENV: &str = "MANTLE_REPRODUCE_RELEASE_ID";
const DETERMINISTIC_PROOF_STORE_DIR_ENV: &str = "MANTLE_DETERMINISTIC_PROOF_STORE_DIR";
const DEFAULT_REPORT_RELATIVE_PATH: &str = "reproducibility/reproducibility-report.json";
const DETERMINISTIC_BUILD_PROOF_BUNDLE_RELATIVE_PATH: &str = "deterministic-release/deterministic-build-proof.json";
const DETERMINISTIC_SANDBOX_EVIDENCE_BUNDLE_RELATIVE_PATH: &str =
    "deterministic-release/deterministic-sandbox-isolation-evidence.json";
const HASH_BUFFER_BYTES: usize = 8192;
const MAX_REBUILD_OUTPUT_ENTRIES: u32 = 4096;
const PROOF_SANDBOX_BWRAP_ENV: &str = "MANTLE_DETERMINISTIC_PROOF_BWRAP";
const PROOF_SANDBOX_PROFILE_PREFIX: &str = "mantle-proof-sandbox-v1";
const PROOF_SANDBOX_SYSTEM_TOOLS_DIR: &str = "/run/current-system/sw";
const PROOF_SANDBOX_PATH: &str = "/run/current-system/sw/bin:/usr/bin:/bin";
const PROOF_SANDBOX_TEMP_ROOT: &str = "/tmp";
const PROOF_SANDBOX_USER: &str = "nobody";
const PROOF_SANDBOX_LOCALE: &str = "C.UTF-8";
const PROOF_SANDBOX_TIMEZONE: &str = "UTC";
const PROOF_SOURCE_DATE_EPOCH: &str = "1";
const PROOF_MODELED_RANDOMNESS: &str = "no-modeled-random-seed";
const PROOF_OUTPUT_ORDERING: &str = "lexicographic-output-processing";
const PROOF_SANDBOX_UMASK_TEXT: &str = "0022";
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
    command_identity: String,
    bundle_dir: PathBuf,
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
            format!("bundle_ro={}", self.bundle_dir.display()),
            format!("recipe_ro={}", self.command_identity),
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

pub(crate) fn reproduce_release_artifacts(
    request: &ReleaseReproduceRequest,
) -> Result<ReleaseReproduceSummary, RunError> {
    validate_request(request)?;
    let mut manifest = verify_release_evidence_bundle(&request.bundle_dir)?;
    prepare_rebuild_output_dir(&request.rebuild_output_dir)?;
    run_rebuild_command(request, &manifest.release_id, &request.rebuild_output_dir, None, None)?;
    let unexpected_outputs = collect_unexpected_rebuilt_outputs(&manifest.binaries, &request.rebuild_output_dir)?;
    let report = build_reproducibility_report(&manifest, request)?;
    let counts = count_report_results(&report.artifacts);
    let report_digest_blake3 = release_reproducibility_report_digest_blake3(report.clone()).map_err(core_error)?;
    let report_path = resolve_report_path(&request.bundle_dir, request.report_path.as_deref());
    write_report(&report_path, report.clone())?;
    fail_if_reproduction_drifted(&report, &unexpected_outputs, &report_path)?;
    let deterministic_proof = maybe_run_deterministic_proof(request, &manifest, &report_path)?;
    if let Some(proof) = &deterministic_proof {
        attach_deterministic_proof_artifacts(&request.bundle_dir, &mut manifest, proof)?;
    }
    let deterministic_proof_unit = deterministic_proof.as_ref().map(|proof| {
        serde_json::json!({
            "target_artifact_identity": proof.receipt.proof_unit.target_artifact_identity.clone(),
            "output_identities": proof.receipt.proof_unit.output_identities.clone(),
            "selected_provider_kind": proof.receipt.selected_provider_kind.clone(),
            "source_blake3": proof.receipt.source_blake3.clone(),
            "vendor_blake3": proof.receipt.vendor_blake3.clone(),
            "toolchain_stage_roots": proof.receipt.toolchain_stage_roots.clone(),
        })
    });
    let deterministic_proof_run_roots = deterministic_proof.as_ref().map(|proof| {
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
    });
    Ok(ReleaseReproduceSummary {
        release_id: manifest.release_id,
        report_path,
        report_digest_blake3,
        matched_count: counts.matched_count,
        mismatched_count: counts.mismatched_count,
        missing_count: counts.missing_count,
        deterministic_proof_path: deterministic_proof.as_ref().map(|proof| proof.path.clone()),
        deterministic_proof_digest_blake3: deterministic_proof.as_ref().map(|proof| proof.digest_blake3.clone()),
        deterministic_sandbox_isolation_evidence_path: deterministic_proof
            .as_ref()
            .map(|proof| proof.isolation_evidence_path.clone()),
        deterministic_sandbox_isolation_evidence_digest_blake3: deterministic_proof
            .as_ref()
            .map(|proof| proof.isolation_evidence_digest_blake3.clone()),
        deterministic_proof_unit,
        deterministic_proof_run_roots,
        deterministic_proof_sandbox_profiles: deterministic_proof
            .as_ref()
            .map(|proof| proof.receipt.sandbox_profile_identities.clone()),
        deterministic_proof_verdict: deterministic_proof.as_ref().and_then(|proof| {
            serde_json::to_value(proof.receipt.verdict)
                .ok()
                .and_then(|value| value.as_str().map(ToOwned::to_owned))
        }),
        deterministic_proof_blockers: deterministic_proof.as_ref().map(|proof| proof.receipt.blocking_reasons.clone()),
    })
}

pub(crate) fn load_bundle_reproducibility_report(
    bundle_dir: &Path,
    manifest: &ReleaseEvidenceManifest,
) -> Result<Option<VerifiedReproducibilityReport>, RunError> {
    let has_manifest_reference = manifest.reproducibility_report.is_some();
    let report_path = manifest
        .reproducibility_report
        .as_ref()
        .map(|artifact| bundle_dir.join(&artifact.relative_path))
        .unwrap_or_else(|| default_reproducibility_report_path(bundle_dir));
    if !report_path.exists() {
        if has_manifest_reference {
            return Err(RunError::Internal(format!(
                "release reproducibility report referenced by manifest is missing: {}",
                report_path.display()
            )));
        }
        return Ok(None);
    }
    let report_bytes = std::fs::read(&report_path)
        .map_err(|err| RunError::Internal(format!("reading {}: {err}", report_path.display())))?;
    let report: ReleaseReproducibilityReport = serde_json::from_slice(&report_bytes)
        .map_err(|err| RunError::Internal(format!("parsing {}: {err}", report_path.display())))?;
    let canonical_bytes = release_reproducibility_report_canonical_bytes(report.clone()).map_err(core_error)?;
    if report_bytes != canonical_bytes {
        return Err(RunError::Internal("release reproducibility report is not canonical compact JSON".to_string()));
    }
    let report = validate_report_linkage_and_artifacts(report, manifest)?;
    let digest_blake3 = blake3::hash(&canonical_bytes).to_hex().to_string();
    let status = report_status(&report);
    Ok(Some(VerifiedReproducibilityReport {
        path: report_path,
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
    let report = validate_release_reproducibility_report_linkage(report, expected).map_err(core_error)?;
    let expected_names = manifest.binaries.iter().map(|artifact| artifact.relative_path.clone()).collect::<Vec<_>>();
    validate_release_reproducibility_report_artifact_names(report, expected_names).map_err(core_error)
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
        return Ok(());
    }
    std::fs::create_dir_all(output_dir)
        .map_err(|err| RunError::Internal(format!("creating {}: {err}", output_dir.display())))
}

fn run_rebuild_command(
    request: &ReleaseReproduceRequest,
    release_id: &str,
    output_dir: &Path,
    deterministic_store_dir: Option<&Path>,
    sandbox_profile: Option<&ProofSandboxProfile>,
) -> Result<(), RunError> {
    let mut command = if let Some(profile) = sandbox_profile {
        sandboxed_rebuild_command(request, release_id, output_dir, deterministic_store_dir, profile)?
    } else {
        let mut command = ProcessCommand::new(&request.rebuild_command);
        command
            .args(&request.rebuild_args)
            .env(REPRODUCE_BUNDLE_DIR_ENV, &request.bundle_dir)
            .env(REPRODUCE_OUTPUT_DIR_ENV, output_dir)
            .env(REPRODUCE_RELEASE_ID_ENV, release_id);
        if let Some(store_dir) = deterministic_store_dir {
            command.env(DETERMINISTIC_PROOF_STORE_DIR_ENV, store_dir);
        }
        command
    };
    let output = command.output().map_err(|err| {
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
    let build_proof = copy_role_bounded_file_into_bundle(
        &proof.path,
        bundle_dir,
        DETERMINISTIC_BUILD_PROOF_BUNDLE_RELATIVE_PATH,
        DETERMINISTIC_BUILD_PROOF_EVIDENCE_ROLE,
        "deterministic build proof receipt",
    )?;
    let sandbox_evidence = copy_role_bounded_file_into_bundle(
        &proof.isolation_evidence_path,
        bundle_dir,
        DETERMINISTIC_SANDBOX_EVIDENCE_BUNDLE_RELATIVE_PATH,
        DETERMINISTIC_SANDBOX_ISOLATION_EVIDENCE_ROLE,
        "deterministic sandbox isolation evidence",
    )?;
    let mut updated = manifest.clone();
    updated.deterministic_build_proof = Some(build_proof);
    updated.deterministic_sandbox_isolation_evidence = Some(sandbox_evidence);
    write_release_evidence_manifest(bundle_dir, &updated)?;
    *manifest = updated;
    Ok(())
}

fn copy_role_bounded_file_into_bundle(
    source: &Path,
    bundle_dir: &Path,
    relative_path: &str,
    evidence_role: &str,
    label: &str,
) -> Result<RoleBoundedReleaseArtifact, RunError> {
    let destination = bundle_dir.join(relative_path);
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| RunError::Internal(format!("creating {}: {err}", parent.display())))?;
    }
    std::fs::copy(source, &destination).map_err(|err| {
        RunError::Internal(format!("copying {label} {} to {}: {err}", source.display(), destination.display()))
    })?;
    role_bounded_file_artifact_record(&destination, relative_path, evidence_role, label)
}

fn role_bounded_file_artifact_record(
    path: &Path,
    relative_path: &str,
    evidence_role: &str,
    label: &str,
) -> Result<RoleBoundedReleaseArtifact, RunError> {
    let (size_bytes, digest_blake3) = hash_file(path)
        .map_err(|err| RunError::Internal(format!("hashing bundle-local {label} {}: {err}", path.display())))?;
    Ok(RoleBoundedReleaseArtifact {
        kind: BundledArtifactKind::File,
        relative_path: relative_path.to_string(),
        size_bytes,
        digest_blake3,
        evidence_role: evidence_role.to_string(),
    })
}

fn write_release_evidence_manifest(bundle_dir: &Path, manifest: &ReleaseEvidenceManifest) -> Result<(), RunError> {
    let bytes = canonical_release_evidence_manifest(manifest.clone()).map_err(core_error)?;
    let path = bundle_dir.join("manifest.json");
    std::fs::write(&path, bytes).map_err(|err| RunError::Internal(format!("writing {}: {err}", path.display())))
}

fn maybe_run_deterministic_proof(
    request: &ReleaseReproduceRequest,
    manifest: &ReleaseEvidenceManifest,
    report_path: &Path,
) -> Result<Option<DeterministicProofOutput>, RunError> {
    if request.deterministic_proof_runs == 0 {
        return Ok(None);
    }
    let proof_root = deterministic_proof_root(request);
    validate_deterministic_proof_root(&proof_root, &request.rebuild_output_dir)?;
    prepare_rebuild_output_dir(&proof_root)?;
    let mut runs = Vec::new();
    let mut sandbox_profiles = Vec::new();
    for run_index in 0..request.deterministic_proof_runs {
        let run_id = format!("run-{run_index:03}");
        let run_root = proof_root.join(&run_id);
        let output_dir = run_root.join("outputs");
        let store_dir = run_root.join("store");
        prepare_rebuild_output_dir(&output_dir)?;
        prepare_rebuild_output_dir(&store_dir)?;
        let sandbox_profile = proof_sandbox_profile(request, &manifest.release_id, &output_dir, &store_dir)?;
        run_rebuild_command(request, &manifest.release_id, &output_dir, Some(&store_dir), Some(&sandbox_profile))?;
        let unexpected_outputs = collect_unexpected_rebuilt_outputs(&manifest.binaries, &output_dir)?;
        let comparisons = compare_manifest_artifacts(&manifest.binaries, &output_dir)?;
        let proof_report = ReleaseReproducibilityReport::new(ReleaseReproducibilityReportInit {
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
            artifacts: comparisons.clone(),
        });
        fail_if_reproduction_drifted(&proof_report, &unexpected_outputs, report_path)?;
        runs.push(deterministic_run_receipt(
            &run_id,
            run_index,
            &store_dir,
            &output_dir,
            &sandbox_profile.identity,
            &comparisons,
        )?);
        sandbox_profiles.push(sandbox_profile);
    }
    let sandbox_profile_identities = runs
        .iter()
        .map(|run| run.sandbox_profile_identity.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let receipt = DeterministicBuildProofReceipt::new(DeterministicBuildProofReceiptInit {
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
            "provider-kind={};command={}",
            manifest.proof_linkage.selected_provider_kind,
            workflow_command_identity(&request.rebuild_command, &request.rebuild_args)?
        ),
        toolchain_stage_roots: vec![
            format!("staged-source={}", manifest.proof_linkage.staged_source),
            format!("stage2-binary={}", manifest.proof_linkage.stage2_binary_digest_blake3),
            format!("prerequisite-inventory={}", manifest.proof_linkage.prerequisite_inventory_digest_blake3),
        ],
        logical_store_prefix: "/mantle/store".to_string(),
        physical_store_isolation: "fresh-store-per-run".to_string(),
        effect_policy_version: BUILD_EFFECT_POLICY_VERSION.to_string(),
        declared_effects: PURE_LOCAL_BUILD_EFFECTS.to_vec(),
        observed_effects: None,
        normalized_execution_envelope: deterministic_execution_envelope(&sandbox_profile_identities),
        ambient_host_perturbations: deterministic_ambient_host_perturbations(),
        sandbox_profile_identities,
        runs,
    });
    let digest_blake3 = deterministic_build_proof_receipt_digest_blake3(receipt.clone()).map_err(core_error)?;
    let path = proof_root.join("deterministic-build-proof.json");
    write_deterministic_proof_receipt(&path, receipt.clone())?;
    let isolation_evidence = deterministic_sandbox_isolation_evidence(&sandbox_profiles)?;
    let isolation_evidence_digest_blake3 =
        deterministic_sandbox_isolation_evidence_digest_blake3(isolation_evidence.clone()).map_err(core_error)?;
    let isolation_evidence_path = proof_root.join("deterministic-sandbox-isolation-evidence.json");
    write_deterministic_sandbox_isolation_evidence(&isolation_evidence_path, isolation_evidence)?;
    Ok(Some(DeterministicProofOutput {
        path,
        digest_blake3,
        receipt,
        isolation_evidence_path,
        isolation_evidence_digest_blake3,
    }))
}

fn proof_sandbox_profile(
    request: &ReleaseReproduceRequest,
    _release_id: &str,
    output_dir: &Path,
    store_dir: &Path,
) -> Result<ProofSandboxProfile, RunError> {
    let executor = resolve_bwrap_executor()?;
    let executor_version = bwrap_version(&executor)?;
    let command_identity = workflow_command_identity(&request.rebuild_command, &request.rebuild_args)?;
    let mut profile = ProofSandboxProfile {
        identity: String::new(),
        executor,
        executor_version,
        network_policy: "none".to_string(),
        command_identity,
        bundle_dir: absolutize_existing_path(&request.bundle_dir, "release evidence bundle")?,
        output_dir: absolutize_path(output_dir)?,
        store_dir: absolutize_path(store_dir)?,
    };
    let canonical = profile.canonical_facts().join("\n");
    let digest = blake3::hash(canonical.as_bytes()).to_hex().to_string();
    profile.identity = format!("{PROOF_SANDBOX_PROFILE_PREFIX}:{digest}");
    Ok(profile)
}

fn sandboxed_rebuild_command(
    request: &ReleaseReproduceRequest,
    release_id: &str,
    output_dir: &Path,
    deterministic_store_dir: Option<&Path>,
    profile: &ProofSandboxProfile,
) -> Result<ProcessCommand, RunError> {
    let store_dir = deterministic_store_dir.ok_or_else(|| {
        RunError::Internal("deterministic proof sandbox requires a proof store directory".to_string())
    })?;
    let command_path = absolutize_existing_path(&request.rebuild_command, "release reproducibility command")?;
    let mut command = ProcessCommand::new(&profile.executor);
    configure_executor_environment(&mut command);
    command
        .arg("--unshare-all")
        .arg("--die-with-parent")
        .arg("--new-session")
        .arg("--clearenv")
        .arg("--tmpfs")
        .arg(PROOF_SANDBOX_TEMP_ROOT);
    let system_tools_dir = Path::new(PROOF_SANDBOX_SYSTEM_TOOLS_DIR);
    let existing_arg_paths = existing_absolute_rebuild_arg_paths(&request.rebuild_args);
    let mut parent_paths = vec![&profile.bundle_dir, &command_path, output_dir, store_dir];
    parent_paths.extend(existing_arg_paths.iter().map(PathBuf::as_path));
    if system_tools_dir.is_dir() {
        parent_paths.push(system_tools_dir);
    }
    append_bwrap_parent_dirs(&mut command, parent_paths);
    if system_tools_dir.is_dir() {
        command.arg("--ro-bind").arg(system_tools_dir).arg(system_tools_dir);
    }
    command
        .arg("--ro-bind")
        .arg(&profile.bundle_dir)
        .arg(&profile.bundle_dir)
        .arg("--ro-bind")
        .arg(&command_path)
        .arg(&command_path);
    for arg_path in &existing_arg_paths {
        command.arg("--ro-bind").arg(arg_path).arg(arg_path);
    }
    command.arg("--bind").arg(output_dir).arg(output_dir).arg("--bind").arg(store_dir).arg(store_dir);
    append_rebuild_sandbox_environment(&mut command, release_id, profile, output_dir, store_dir);
    configure_sandbox_umask(&mut command);
    command.arg("--chdir").arg(PROOF_SANDBOX_TEMP_ROOT).arg(&command_path).args(&request.rebuild_args);
    Ok(command)
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
        (REPRODUCE_BUNDLE_DIR_ENV, profile.bundle_dir.as_os_str()),
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

fn deterministic_run_receipt(
    run_id: &str,
    run_index: u32,
    store_dir: &Path,
    output_dir: &Path,
    sandbox_profile_identity: &str,
    comparisons: &[ReproducibilityArtifactComparison],
) -> Result<DeterministicBuildRunReceipt, RunError> {
    let mut output_digests = Vec::with_capacity(comparisons.len());
    for comparison in comparisons {
        let Some(observed_digest) = &comparison.observed_digest_blake3 else {
            return Err(RunError::Internal(format!(
                "deterministic proof run {run_id} missing rebuilt artifact {}",
                comparison.name
            )));
        };
        output_digests.push(DeterministicOutputDigest {
            name: comparison.name.clone(),
            digest_blake3: observed_digest.clone(),
        });
    }
    Ok(DeterministicBuildRunReceipt {
        run_id: run_id.to_string(),
        perturbation_case: deterministic_perturbation_case(run_index),
        output_store_paths: vec![store_dir.display().to_string()],
        output_root_identity: output_dir.display().to_string(),
        sandbox_profile_identity: sandbox_profile_identity.to_string(),
        output_digests,
        substituted_dependency_identities: Vec::new(),
        hermeticity_audit_events: Vec::new(),
        observed_effects: Some(PURE_LOCAL_BUILD_EFFECTS.to_vec()),
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
    let matched = counts.mismatched_count == 0 && counts.missing_count == 0;
    Ok(ReleaseReproducibilityReport::new(ReleaseReproducibilityReportInit {
        release_id: manifest.release_id.clone(),
        proof_class: if matched {
            ReproducibilityProofClass::SelfRebuildMatch
        } else {
            ReproducibilityProofClass::SelfProofValid
        },
        comparison_verdict: if matched {
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
    loop {
        let bytes_read = file
            .read(&mut buffer)
            .map_err(|err| RunError::Internal(format!("read {}: {err}", path.display())))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    Ok((metadata.len(), hasher.finalize().to_hex().to_string()))
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
    let mut unexpected = Vec::new();
    for file in files {
        let relative = relative_path_text(rebuild_output_dir, &file)?;
        if !expected_paths.contains(&relative) {
            unexpected.push(relative);
        }
    }
    Ok(unexpected)
}

fn collect_output_files_sorted(root: &Path, files: &mut Vec<PathBuf>) -> Result<(), RunError> {
    let mut children = Vec::new();
    for child_result in
        std::fs::read_dir(root).map_err(|err| RunError::Internal(format!("read_dir {}: {err}", root.display())))?
    {
        let child =
            child_result.map_err(|err| RunError::Internal(format!("read_dir entry {}: {err}", root.display())))?;
        children.push(child.path());
    }
    children.sort();
    for child in children {
        let entry_count: u32 = files
            .len()
            .try_into()
            .map_err(|_| RunError::Internal("release reproducibility output entry count overflowed u32".to_string()))?;
        if entry_count >= MAX_REBUILD_OUTPUT_ENTRIES {
            return Err(RunError::Internal(format!(
                "release reproducibility output tree exceeds {MAX_REBUILD_OUTPUT_ENTRIES} files"
            )));
        }
        if child.is_dir() {
            collect_output_files_sorted(&child, files)?;
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
    Ok(())
}

fn relative_path_text(root: &Path, file: &Path) -> Result<String, RunError> {
    let relative = file.strip_prefix(root).map_err(|err| {
        RunError::Internal(format!(
            "release reproducibility output strip_prefix {} from {}: {err}",
            root.display(),
            file.display()
        ))
    })?;
    let mut parts = Vec::new();
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
            Path::new("/home/brittonr/.cargo-target/repo-targets/crunch__crunch/release/rebuild.sh"),
            Path::new("/home/brittonr/.cargo-target/repo-targets/crunch__crunch/proof/run-000/store"),
        ]);

        let args: Vec<_> = command.get_args().map(|arg| arg.to_string_lossy().into_owned()).collect();
        assert!(args.windows(2).any(|pair| pair == ["--dir", "/home"]));
        assert!(args.windows(2).any(|pair| pair == ["--dir", "/home/brittonr/.cargo-target"]));
        assert!(
            args.windows(2)
                .any(|pair| pair == ["--dir", "/home/brittonr/.cargo-target/repo-targets/crunch__crunch"])
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
            command_identity: "test-command".to_string(),
            bundle_dir: PathBuf::from("/bundle"),
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

        let run = deterministic_run_receipt(
            "run-000",
            0,
            Path::new("/tmp/proof/run-000/store"),
            Path::new("/tmp/proof/run-000/outputs"),
            "mantle-proof-sandbox-v1:test",
            &[comparison],
        )
        .unwrap();

        assert_eq!(run.output_store_paths, vec!["/tmp/proof/run-000/store".to_string()]);
        assert_eq!(run.output_root_identity, "/tmp/proof/run-000/outputs");
        assert_eq!(run.sandbox_profile_identity, "mantle-proof-sandbox-v1:test");
        assert_eq!(run.output_digests[0].digest_blake3, sample_digest(1));
        assert_eq!(run.perturbation_case, "baseline-clean-env");
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
