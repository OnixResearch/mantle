use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command as ProcessCommand;

use crunch_release_core::BundledArtifact;
use crunch_release_core::RebuildWorkflowIdentity;
use crunch_release_core::ReleaseEvidenceManifest;
use crunch_release_core::ReleaseReproducibilityReport;
use crunch_release_core::ReleaseReproducibilityReportInit;
use crunch_release_core::ReleaseReproducibilityReportLinkage;
use crunch_release_core::ReproducibilityArtifactComparison;
use crunch_release_core::ReproducibilityComparisonResult;
use crunch_release_core::release_reproducibility_report_canonical_bytes;
use crunch_release_core::release_reproducibility_report_digest_blake3;
use crunch_release_core::validate_release_reproducibility_report_artifact_names;
use crunch_release_core::validate_release_reproducibility_report_linkage;

use crate::errors::RunError;
use crate::release_evidence::verify_release_evidence_bundle;

pub(crate) const DEFAULT_REPRODUCIBILITY_WORKFLOW_VERSION: &str = "crunch-release-reproducibility-v1";

const REPRODUCE_BUNDLE_DIR_ENV: &str = "CRUNCH_REPRODUCE_BUNDLE_DIR";
const REPRODUCE_OUTPUT_DIR_ENV: &str = "CRUNCH_REPRODUCE_OUTPUT_DIR";
const REPRODUCE_RELEASE_ID_ENV: &str = "CRUNCH_REPRODUCE_RELEASE_ID";
const DEFAULT_REPORT_RELATIVE_PATH: &str = "reproducibility/reproducibility-report.json";
const HASH_BUFFER_BYTES: usize = 8192;
const MAX_REBUILD_OUTPUT_ENTRIES: u32 = 4096;

#[derive(Debug, Clone)]
pub(crate) struct ReleaseReproduceRequest {
    pub bundle_dir: PathBuf,
    pub rebuild_output_dir: PathBuf,
    pub rebuild_command: PathBuf,
    pub rebuild_args: Vec<OsString>,
    pub workflow_version: String,
    pub report_path: Option<PathBuf>,
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
    let manifest = verify_release_evidence_bundle(&request.bundle_dir)?;
    prepare_rebuild_output_dir(&request.rebuild_output_dir)?;
    run_rebuild_command(request, &manifest.release_id)?;
    let unexpected_outputs = collect_unexpected_rebuilt_outputs(&manifest.binaries, &request.rebuild_output_dir)?;
    let report = build_reproducibility_report(&manifest, request)?;
    let counts = count_report_results(&report.artifacts);
    let report_digest_blake3 = release_reproducibility_report_digest_blake3(report.clone()).map_err(core_error)?;
    let report_path = resolve_report_path(&request.bundle_dir, request.report_path.as_deref());
    write_report(&report_path, report.clone())?;
    fail_if_reproduction_drifted(&report, &unexpected_outputs, &report_path)?;
    Ok(ReleaseReproduceSummary {
        release_id: manifest.release_id,
        report_path,
        report_digest_blake3,
        matched_count: counts.matched_count,
        mismatched_count: counts.mismatched_count,
        missing_count: counts.missing_count,
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

fn run_rebuild_command(request: &ReleaseReproduceRequest, release_id: &str) -> Result<(), RunError> {
    let output = ProcessCommand::new(&request.rebuild_command)
        .args(&request.rebuild_args)
        .env(REPRODUCE_BUNDLE_DIR_ENV, &request.bundle_dir)
        .env(REPRODUCE_OUTPUT_DIR_ENV, &request.rebuild_output_dir)
        .env(REPRODUCE_RELEASE_ID_ENV, release_id)
        .output()
        .map_err(|err| {
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

fn build_reproducibility_report(
    manifest: &ReleaseEvidenceManifest,
    request: &ReleaseReproduceRequest,
) -> Result<ReleaseReproducibilityReport, RunError> {
    let comparisons = compare_manifest_artifacts(&manifest.binaries, &request.rebuild_output_dir)?;
    Ok(ReleaseReproducibilityReport::new(ReleaseReproducibilityReportInit {
        release_id: manifest.release_id.clone(),
        source_archive_digest_blake3: manifest.source_archive.digest_blake3.clone(),
        proof_bundle_digest_blake3: manifest.proof_bundle.digest_blake3.clone(),
        rebuild_workflow: RebuildWorkflowIdentity {
            command: workflow_command_identity(&request.rebuild_command, &request.rebuild_args)?,
            version: request.workflow_version.clone(),
        },
        artifacts: comparisons,
    }))
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

    fn sample_digest(seed: u8) -> String {
        let nibble = format!("{:x}", seed % 16);
        nibble.repeat(crunch_release_core::BLAKE3_HEX_LENGTH_CHARS)
    }

    fn sample_artifact() -> BundledArtifact {
        BundledArtifact {
            kind: crunch_release_core::BundledArtifactKind::File,
            relative_path: "binaries/01-crunch".to_string(),
            size_bytes: EXPECTED_SIZE_BYTES,
            digest_blake3: sample_digest(1),
        }
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
        assert_eq!(comparison.name, "binaries/01-crunch");
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
