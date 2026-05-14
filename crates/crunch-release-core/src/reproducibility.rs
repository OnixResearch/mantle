use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::ReleaseEvidenceError;
use crate::manifest::u32_count;
use crate::manifest::validate_blake3_hex;
use crate::manifest::validation_error;

pub const RELEASE_REPRODUCIBILITY_REPORT_SCHEMA: &str = "crunch-release-reproducibility-report-v1";

const MAX_REPRODUCIBILITY_ARTIFACT_COUNT: u32 = 256;
const MAX_REPRODUCIBILITY_ARTIFACT_NAME_BYTES_COUNT: u32 = 512;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RebuildWorkflowIdentity {
    pub command: String,
    pub version: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReproducibilityComparisonResult {
    Matched,
    Mismatched,
    MissingRebuiltArtifact,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReproducibilityArtifactComparison {
    pub name: String,
    pub expected_size_bytes: u64,
    pub expected_digest_blake3: String,
    pub observed_size_bytes: Option<u64>,
    pub observed_digest_blake3: Option<String>,
    pub result: ReproducibilityComparisonResult,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseReproducibilityReport {
    pub schema: String,
    pub release_id: String,
    pub source_archive_digest_blake3: String,
    pub proof_bundle_digest_blake3: String,
    pub rebuild_workflow: RebuildWorkflowIdentity,
    pub artifacts: Vec<ReproducibilityArtifactComparison>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseReproducibilityReportInit {
    pub release_id: String,
    pub source_archive_digest_blake3: String,
    pub proof_bundle_digest_blake3: String,
    pub rebuild_workflow: RebuildWorkflowIdentity,
    pub artifacts: Vec<ReproducibilityArtifactComparison>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseReproducibilityReportLinkage {
    pub release_id: String,
    pub source_archive_digest_blake3: String,
    pub proof_bundle_digest_blake3: String,
}

impl ReleaseReproducibilityReport {
    pub fn new(init: ReleaseReproducibilityReportInit) -> Self {
        Self {
            schema: RELEASE_REPRODUCIBILITY_REPORT_SCHEMA.to_string(),
            release_id: init.release_id,
            source_archive_digest_blake3: init.source_archive_digest_blake3,
            proof_bundle_digest_blake3: init.proof_bundle_digest_blake3,
            rebuild_workflow: init.rebuild_workflow,
            artifacts: init.artifacts,
        }
    }
}

pub fn canonical_release_reproducibility_report(
    mut report: ReleaseReproducibilityReport,
) -> Result<ReleaseReproducibilityReport, ReleaseEvidenceError> {
    validate_report_header(&report)?;
    report.schema = RELEASE_REPRODUCIBILITY_REPORT_SCHEMA.to_string();
    report.artifacts.sort_by(|left, right| left.name.cmp(&right.name));
    validate_report_artifacts(&report.artifacts)?;
    Ok(report)
}

pub fn release_reproducibility_report_canonical_bytes(
    report: ReleaseReproducibilityReport,
) -> Result<Vec<u8>, ReleaseEvidenceError> {
    let canonical = canonical_release_reproducibility_report(report)?;
    // `serde_json::to_vec` emits compact JSON; validation above fixes ordering first.
    serde_json::to_vec(&canonical)
        .map_err(|err| ReleaseEvidenceError::Parse(format!("serializing release reproducibility report: {err}")))
}

pub fn release_reproducibility_report_digest_blake3(
    report: ReleaseReproducibilityReport,
) -> Result<String, ReleaseEvidenceError> {
    let canonical_bytes = release_reproducibility_report_canonical_bytes(report)?;
    Ok(blake3::hash(&canonical_bytes).to_hex().to_string())
}

pub fn validate_release_reproducibility_report_linkage(
    report: ReleaseReproducibilityReport,
    expected: ReleaseReproducibilityReportLinkage,
) -> Result<ReleaseReproducibilityReport, ReleaseEvidenceError> {
    let report = canonical_release_reproducibility_report(report)?;
    validate_linkage_field(&report.release_id, &expected.release_id, "release_id")?;
    validate_linkage_field(
        &report.source_archive_digest_blake3,
        &expected.source_archive_digest_blake3,
        "source_archive_digest_blake3",
    )?;
    validate_linkage_field(
        &report.proof_bundle_digest_blake3,
        &expected.proof_bundle_digest_blake3,
        "proof_bundle_digest_blake3",
    )?;
    Ok(report)
}

pub fn validate_release_reproducibility_report_artifact_names(
    report: ReleaseReproducibilityReport,
    expected_names: Vec<String>,
) -> Result<ReleaseReproducibilityReport, ReleaseEvidenceError> {
    let report = canonical_release_reproducibility_report(report)?;
    let actual_names = report.artifacts.iter().map(|artifact| artifact.name.clone()).collect::<Vec<_>>();
    let expected_names = normalize_expected_artifact_names(expected_names)?;
    if actual_names != expected_names {
        return Err(validation_error(format!(
            "release reproducibility report artifact set mismatch: expected {expected_names:?}, got {actual_names:?}"
        )));
    }
    Ok(report)
}

fn validate_linkage_field(actual: &str, expected: &str, field_name: &str) -> Result<(), ReleaseEvidenceError> {
    if actual != expected {
        return Err(validation_error(format!(
            "release reproducibility report {field_name} linkage mismatch: expected {expected}, got {actual}"
        )));
    }
    Ok(())
}

fn normalize_expected_artifact_names(mut names: Vec<String>) -> Result<Vec<String>, ReleaseEvidenceError> {
    let name_count =
        u32_count(names.len(), "release reproducibility report expected artifact name count overflowed u32")?;
    if name_count == 0 {
        return Err(validation_error(
            "release reproducibility report expected artifact names must not be empty".to_string(),
        ));
    }
    names.sort();
    let mut names_seen = BTreeSet::new();
    for name in &names {
        validate_artifact_name(name)?;
        if !names_seen.insert(name.clone()) {
            return Err(validation_error(format!(
                "release reproducibility report expected artifact names contain duplicate {name}"
            )));
        }
    }
    Ok(names)
}

fn validate_report_header(report: &ReleaseReproducibilityReport) -> Result<(), ReleaseEvidenceError> {
    if report.schema != RELEASE_REPRODUCIBILITY_REPORT_SCHEMA {
        return Err(validation_error(format!(
            "release reproducibility report schema must be {RELEASE_REPRODUCIBILITY_REPORT_SCHEMA}, got {}",
            report.schema
        )));
    }
    if report.release_id.trim().is_empty() {
        return Err(validation_error("release reproducibility report release_id must not be empty".to_string()));
    }
    validate_blake3_hex(&report.source_archive_digest_blake3, "source_archive_digest_blake3")?;
    validate_blake3_hex(&report.proof_bundle_digest_blake3, "proof_bundle_digest_blake3")?;
    if report.rebuild_workflow.command.trim().is_empty() {
        return Err(validation_error(
            "release reproducibility report rebuild_workflow.command must not be empty".to_string(),
        ));
    }
    if report.rebuild_workflow.version.trim().is_empty() {
        return Err(validation_error(
            "release reproducibility report rebuild_workflow.version must not be empty".to_string(),
        ));
    }
    Ok(())
}

fn validate_report_artifacts(artifacts: &[ReproducibilityArtifactComparison]) -> Result<(), ReleaseEvidenceError> {
    let artifact_count = u32_count(artifacts.len(), "release reproducibility report artifact count overflowed u32")?;
    if artifact_count == 0 {
        return Err(validation_error("release reproducibility report must record at least one artifact".to_string()));
    }
    if artifact_count > MAX_REPRODUCIBILITY_ARTIFACT_COUNT {
        return Err(validation_error(format!(
            "release reproducibility report records {artifact_count} artifacts, limit is {MAX_REPRODUCIBILITY_ARTIFACT_COUNT}"
        )));
    }

    let mut names_seen = BTreeSet::new();
    for artifact in artifacts {
        validate_artifact_comparison(artifact)?;
        if !names_seen.insert(artifact.name.clone()) {
            return Err(validation_error(format!(
                "release reproducibility report contains duplicate artifact name {}",
                artifact.name
            )));
        }
    }
    Ok(())
}

fn validate_artifact_comparison(artifact: &ReproducibilityArtifactComparison) -> Result<(), ReleaseEvidenceError> {
    validate_artifact_name(&artifact.name)?;
    if artifact.expected_size_bytes == 0 {
        return Err(validation_error(format!(
            "release reproducibility report artifact {} expected_size_bytes must be non-zero",
            artifact.name
        )));
    }
    validate_blake3_hex(
        &artifact.expected_digest_blake3,
        &format!("artifacts.{}.expected_digest_blake3", artifact.name),
    )?;
    validate_observed_fields(artifact)?;
    validate_result_consistency(artifact)
}

fn validate_artifact_name(name: &str) -> Result<(), ReleaseEvidenceError> {
    if name.trim().is_empty() {
        return Err(validation_error("release reproducibility report artifact name must not be empty".to_string()));
    }
    let name_len_bytes = u32_count(name.len(), "release reproducibility report artifact name length overflowed u32")?;
    if name_len_bytes > MAX_REPRODUCIBILITY_ARTIFACT_NAME_BYTES_COUNT {
        return Err(validation_error(format!(
            "release reproducibility report artifact name exceeds {MAX_REPRODUCIBILITY_ARTIFACT_NAME_BYTES_COUNT} bytes"
        )));
    }
    Ok(())
}

fn validate_observed_fields(artifact: &ReproducibilityArtifactComparison) -> Result<(), ReleaseEvidenceError> {
    if let Some(observed_size_bytes) = artifact.observed_size_bytes {
        if observed_size_bytes == 0 {
            return Err(validation_error(format!(
                "release reproducibility report artifact {} observed_size_bytes must be non-zero",
                artifact.name
            )));
        }
    }
    if let Some(observed_digest_blake3) = &artifact.observed_digest_blake3 {
        validate_blake3_hex(observed_digest_blake3, &format!("artifacts.{}.observed_digest_blake3", artifact.name))?;
    }
    Ok(())
}

fn validate_result_consistency(artifact: &ReproducibilityArtifactComparison) -> Result<(), ReleaseEvidenceError> {
    let has_observed_size = artifact.observed_size_bytes.is_some();
    let has_observed_digest = artifact.observed_digest_blake3.is_some();
    if has_observed_size != has_observed_digest {
        return Err(validation_error(format!(
            "release reproducibility report artifact {} must record observed size and digest together",
            artifact.name
        )));
    }

    match artifact.result {
        ReproducibilityComparisonResult::Matched => validate_matched_artifact(artifact),
        ReproducibilityComparisonResult::Mismatched => validate_mismatched_artifact(artifact),
        ReproducibilityComparisonResult::MissingRebuiltArtifact => validate_missing_artifact(artifact),
    }
}

fn validate_matched_artifact(artifact: &ReproducibilityArtifactComparison) -> Result<(), ReleaseEvidenceError> {
    let observed_size_bytes = require_observed_size(artifact)?;
    let observed_digest_blake3 = require_observed_digest(artifact)?;
    if observed_size_bytes != artifact.expected_size_bytes {
        return Err(validation_error(format!(
            "release reproducibility report artifact {} is matched but sizes differ",
            artifact.name
        )));
    }
    if observed_digest_blake3 != artifact.expected_digest_blake3 {
        return Err(validation_error(format!(
            "release reproducibility report artifact {} is matched but digests differ",
            artifact.name
        )));
    }
    Ok(())
}

fn validate_mismatched_artifact(artifact: &ReproducibilityArtifactComparison) -> Result<(), ReleaseEvidenceError> {
    let observed_size_bytes = require_observed_size(artifact)?;
    let observed_digest_blake3 = require_observed_digest(artifact)?;
    let size_matches = observed_size_bytes == artifact.expected_size_bytes;
    let digest_matches = observed_digest_blake3 == artifact.expected_digest_blake3;
    // A mismatch may be length-only or digest-only; reject only the fully identical case.
    if size_matches && digest_matches {
        return Err(validation_error(format!(
            "release reproducibility report artifact {} is mismatched but observed bytes match",
            artifact.name
        )));
    }
    Ok(())
}

fn validate_missing_artifact(artifact: &ReproducibilityArtifactComparison) -> Result<(), ReleaseEvidenceError> {
    if artifact.observed_size_bytes.is_some() || artifact.observed_digest_blake3.is_some() {
        return Err(validation_error(format!(
            "release reproducibility report artifact {} is missing but observed fields are present",
            artifact.name
        )));
    }
    Ok(())
}

fn require_observed_size(artifact: &ReproducibilityArtifactComparison) -> Result<u64, ReleaseEvidenceError> {
    artifact.observed_size_bytes.ok_or_else(|| {
        validation_error(format!(
            "release reproducibility report artifact {} must record observed_size_bytes",
            artifact.name
        ))
    })
}

fn require_observed_digest(artifact: &ReproducibilityArtifactComparison) -> Result<&str, ReleaseEvidenceError> {
    artifact.observed_digest_blake3.as_deref().ok_or_else(|| {
        validation_error(format!(
            "release reproducibility report artifact {} must record observed_digest_blake3",
            artifact.name
        ))
    })
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::manifest::BLAKE3_HEX_LENGTH_CHARS;

    const OBSERVED_SIZE_BYTES: u64 = 123;
    const DRIFTED_SIZE_BYTES: u64 = 124;

    fn sample_digest(seed: u8) -> String {
        let byte = format!("{:x}", seed % 16);
        byte.repeat(BLAKE3_HEX_LENGTH_CHARS)
    }

    fn sample_workflow() -> RebuildWorkflowIdentity {
        RebuildWorkflowIdentity {
            command: "./scripts/prove-self-hosting.sh".to_string(),
            version: "mantle-self-hosting-proof-v2".to_string(),
        }
    }

    fn matched_artifact(name: &str, seed: u8) -> ReproducibilityArtifactComparison {
        let digest = sample_digest(seed);
        ReproducibilityArtifactComparison {
            name: name.to_string(),
            expected_size_bytes: OBSERVED_SIZE_BYTES,
            expected_digest_blake3: digest.clone(),
            observed_size_bytes: Some(OBSERVED_SIZE_BYTES),
            observed_digest_blake3: Some(digest),
            result: ReproducibilityComparisonResult::Matched,
        }
    }

    fn sample_report() -> ReleaseReproducibilityReport {
        ReleaseReproducibilityReport::new(ReleaseReproducibilityReportInit {
            release_id: "crunch-0.1.0-rc1".to_string(),
            source_archive_digest_blake3: sample_digest(1),
            proof_bundle_digest_blake3: sample_digest(2),
            rebuild_workflow: sample_workflow(),
            artifacts: vec![
                matched_artifact("crunch-aarch64-linux", 3),
                matched_artifact("crunch-x86_64-linux", 4),
            ],
        })
    }

    fn sample_linkage() -> ReleaseReproducibilityReportLinkage {
        ReleaseReproducibilityReportLinkage {
            release_id: "crunch-0.1.0-rc1".to_string(),
            source_archive_digest_blake3: sample_digest(1),
            proof_bundle_digest_blake3: sample_digest(2),
        }
    }

    #[test]
    fn reproducibility_report_canonical_bytes_are_stable_and_sorted() {
        let mut first = sample_report();
        first.artifacts.reverse();
        let second = sample_report();

        let first_bytes = release_reproducibility_report_canonical_bytes(first).unwrap();
        let second_bytes = release_reproducibility_report_canonical_bytes(second).unwrap();

        assert_eq!(first_bytes, second_bytes);
        assert!(!first_bytes.contains(&b'\n'));
    }

    #[test]
    fn reproducibility_report_digest_is_blake3_of_canonical_bytes() {
        let report = sample_report();
        let canonical_bytes = release_reproducibility_report_canonical_bytes(report.clone()).unwrap();
        let expected_digest = blake3::hash(&canonical_bytes).to_hex().to_string();
        let actual_digest = release_reproducibility_report_digest_blake3(report).unwrap();

        assert_eq!(actual_digest, expected_digest);
        assert_eq!(actual_digest.len(), BLAKE3_HEX_LENGTH_CHARS);
    }

    #[test]
    fn reproducibility_report_rejects_matched_artifact_with_drift() {
        let mut report = sample_report();
        report.artifacts[0].observed_size_bytes = Some(DRIFTED_SIZE_BYTES);

        let err = release_reproducibility_report_canonical_bytes(report).unwrap_err();

        assert!(err.to_string().contains("matched but sizes differ"));
    }

    #[test]
    fn reproducibility_report_rejects_missing_artifact_with_observed_digest() {
        let mut report = sample_report();
        report.artifacts[0].observed_size_bytes = None;
        report.artifacts[0].result = ReproducibilityComparisonResult::MissingRebuiltArtifact;

        let err = release_reproducibility_report_canonical_bytes(report).unwrap_err();

        assert!(err.to_string().contains("observed size and digest together"));
    }

    #[test]
    fn reproducibility_report_accepts_mismatched_digest() {
        let mut report = sample_report();
        report.artifacts[0].result = ReproducibilityComparisonResult::Mismatched;
        report.artifacts[0].observed_digest_blake3 = Some(sample_digest(5));

        let canonical = canonical_release_reproducibility_report(report).unwrap();

        assert_eq!(canonical.artifacts[0].result, ReproducibilityComparisonResult::Mismatched);
    }

    #[test]
    fn reproducibility_report_accepts_missing_rebuilt_artifact_fixture() {
        let mut report = sample_report();
        report.artifacts[0].result = ReproducibilityComparisonResult::MissingRebuiltArtifact;
        report.artifacts[0].observed_size_bytes = None;
        report.artifacts[0].observed_digest_blake3 = None;

        let canonical = canonical_release_reproducibility_report(report).unwrap();

        assert_eq!(canonical.artifacts[0].result, ReproducibilityComparisonResult::MissingRebuiltArtifact);
    }

    #[test]
    fn reproducibility_report_rejects_output_name_drift_fixture() {
        let report = sample_report();
        let expected_names = vec!["crunch-x86_64-linux".to_string(), "renamed-crunch".to_string()];

        let err = validate_release_reproducibility_report_artifact_names(report, expected_names).unwrap_err();

        assert!(err.to_string().contains("artifact set mismatch"));
    }

    #[test]
    fn reproducibility_report_accepts_matching_artifact_names_out_of_order() {
        let report = sample_report();
        let expected_names = vec!["crunch-x86_64-linux".to_string(), "crunch-aarch64-linux".to_string()];

        let canonical = validate_release_reproducibility_report_artifact_names(report, expected_names).unwrap();

        assert_eq!(canonical.artifacts[0].name, "crunch-aarch64-linux");
    }

    #[test]
    fn reproducibility_report_rejects_proof_linkage_mismatch_fixture() {
        let report = sample_report();
        let mut expected = sample_linkage();
        expected.proof_bundle_digest_blake3 = sample_digest(6);

        let err = validate_release_reproducibility_report_linkage(report, expected).unwrap_err();

        assert!(err.to_string().contains("proof_bundle_digest_blake3 linkage mismatch"));
    }

    #[test]
    fn reproducibility_report_accepts_matching_linkage() {
        let report = sample_report();
        let expected = sample_linkage();

        let canonical = validate_release_reproducibility_report_linkage(report, expected).unwrap();

        assert_eq!(canonical.release_id, "crunch-0.1.0-rc1");
    }
}
