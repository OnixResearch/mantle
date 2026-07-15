use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::ReleaseEvidenceError;

pub const CAIRN_HANDOFF_INPUT_SCHEMA: &str = "mantle-cairn-release-handoff-input-v2";
pub const CAIRN_HANDOFF_VALIDATION_RECEIPT_SCHEMA: &str = "mantle-cairn-release-handoff-validation-v2";
pub const CAIRN_HANDOFF_VALIDATION_STATUS: &str = "validated-bundle-local";
pub const CAIRN_HANDOFF_AUTHENTICATION_STATUS: &str = "archive-authentication-prerequisite-bound-v1";
pub const CAIRN_HANDOFF_AUTHENTICATION_SCHEMA: &str = "mantle.cairn-authentication-dependency.v1";
pub const CAIRN_HANDOFF_AUTHENTICATION_CHANGE: &str = "authenticate-stack-provenance-inputs";
pub const CAIRN_HANDOFF_AUTHENTICATION_CAIRN_REVISION: &str = "f4a1f8df0d430c1b9431358a388ac1d3c1a823ec";
pub const CAIRN_HANDOFF_AUTHENTICATION_ARCHIVE_MANIFEST_BLAKE3: &str =
    "40ea9765488bd02e362f70d5c9c498932544c80f2231a70f9b5c3ef81cd7df83";
pub const CAIRN_HANDOFF_AUTHENTICATION_ARCHIVE_MUTATION_RECEIPT_BLAKE3: &str =
    "8a4250a7db47dd4c013467d65e5667af188aa66599a1b89df6788ef067598fa9";
pub const CAIRN_HANDOFF_AUTHENTICATION_RECEIPT_BLAKE3: &str =
    "bf33d82555c7bd3afcbc7adac743782327a5d08e536266f0dfda97d6fe342edf";
pub const CAIRN_HANDOFF_DISPOSITION_ABSENT: &str = "absent";
pub const CAIRN_HANDOFF_DISPOSITION_INVALID: &str = "invalid";
pub const CAIRN_HANDOFF_DISPOSITION_PRESENT: &str = "present";
pub const CAIRN_HANDOFF_BOUNDARY: &str = "Mantle validates bundle-local measured Cairn artifact, policy, role, schema, readiness, coverage, release-bundle linkage, and the pinned archived Cairn authenticated-input dependency only; Cairn owns lifecycle readiness";
pub const CAIRN_HANDOFF_AUTHENTICATION_BLOCKER: &str =
    "Cairn handoff authentication does not match the pinned archived authenticate-stack-provenance-inputs receipt";
pub const CAIRN_HANDOFF_NON_CLAIM_RELEASE_CORRECTNESS: &str = "not release correctness";
pub const CAIRN_HANDOFF_NON_CLAIM_BUILD_CORRECTNESS: &str = "not build correctness";
pub const CAIRN_HANDOFF_NON_CLAIM_SOURCE_CORRECTNESS: &str = "not source correctness";
pub const CAIRN_HANDOFF_NON_CLAIM_DEPLOYMENT_SAFETY: &str = "not deployment safety";

pub const MAX_CAIRN_HANDOFF_ROWS_COUNT: u32 = 128;
pub const MAX_CAIRN_HANDOFF_COVERS_COUNT: u32 = 256;
pub const MAX_CAIRN_HANDOFF_NON_CLAIMS_COUNT: u32 = 32;
pub const MAX_CAIRN_HANDOFF_BINARY_DIGESTS_COUNT: u32 = 16;
pub const MAX_CAIRN_HANDOFF_DIAGNOSTICS_COUNT: u32 = 128;
pub const MAX_CAIRN_HANDOFF_TEXT_BYTES_COUNT: u32 = 4096;

const BLAKE3_HEX_LENGTH_CHARS: usize = 64;
const BUNDLE_BINDING_DOMAIN: &[u8] = b"mantle.cairn-handoff.bundle-binding.v1\0";
const RECEIPT_DOMAIN: &[u8] = b"mantle.cairn-handoff.validation-receipt.v2\0";
const REQUIRED_NON_CLAIM_FRAGMENT: &str = "not release correctness";
const SUPPORTED_ROLE_SCHEMA_PAIRS: &[(&str, &str)] = &[
    ("cairn-release-readiness-receipt", "cairn.release-readiness.v1"),
    ("cairn-change-validation-receipt", "cairn.change-validation.v1"),
    ("cairn-archive-evidence-index", "cairn.archive-index.v1"),
];
const OVERCLAIM_FRAGMENTS: &[&str] = &[
    "proves release correctness",
    "proves build correctness",
    "proves source correctness",
    "proves artifact correctness",
    "proves deployment safety",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CairnMeasuredArtifact {
    pub relative_path: String,
    pub size_bytes: u64,
    pub declared_digest_blake3: String,
    pub measured_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CairnReleaseEvidenceRow {
    pub artifact_id: String,
    pub role: String,
    pub schema_id: String,
    pub artifact: CairnMeasuredArtifact,
    pub cairn_policy: CairnMeasuredArtifact,
    pub release_readiness_id: String,
    pub covers: Vec<String>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CairnHandoffAuthenticationDependency {
    pub schema: String,
    pub change_name: String,
    pub cairn_revision: String,
    pub archive_manifest_blake3: String,
    pub archive_mutation_receipt_blake3: String,
    pub archive_receipt: CairnMeasuredArtifact,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CairnReleaseEvidenceHandoff {
    pub authentication: CairnHandoffAuthenticationDependency,
    pub rows: Vec<CairnReleaseEvidenceRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CairnReleaseBundleBinding {
    pub release_id: String,
    pub source_archive_digest_blake3: String,
    pub binary_digests_blake3: Vec<String>,
    pub proof_bundle_digest_blake3: String,
    pub prerequisite_inventory_digest_blake3: String,
    pub release_manifest_projection_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CairnReleaseEvidenceValidationReceipt {
    pub schema: String,
    pub validation_status: String,
    pub authentication_status: String,
    pub bundle_binding: CairnReleaseBundleBinding,
    pub bundle_binding_blake3: String,
    pub handoff: CairnReleaseEvidenceHandoff,
    pub boundary: String,
    pub non_claims: Vec<String>,
    pub receipt_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CairnReleaseEvidenceReport {
    pub valid: bool,
    pub diagnostics: Vec<String>,
}

fn absent_optional<T>() -> Option<T> {
    None
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CairnHandoffReleaseVerification {
    pub required: bool,
    pub valid: bool,
    pub disposition: String,
    #[serde(default = "absent_optional", skip_serializing_if = "Option::is_none")]
    pub bundle_binding_blake3: Option<String>,
    #[serde(default = "absent_optional", skip_serializing_if = "Option::is_none")]
    pub receipt_blake3: Option<String>,
    #[serde(default = "absent_optional", skip_serializing_if = "Option::is_none")]
    pub authentication_status: Option<String>,
    pub boundary: String,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
struct TextField<'a> {
    label: &'a str,
    value: &'a str,
}

// r[impl mantle.release_provenance.cairn_evidence_handoff.measured_inputs]
// r[impl mantle.release_provenance.cairn_evidence_handoff.production_wiring]
pub fn validate_cairn_release_evidence_handoff(handoff: &CairnReleaseEvidenceHandoff) -> CairnReleaseEvidenceReport {
    let mut diagnostics = Vec::new();
    validate_row_count(handoff.rows.len(), &mut diagnostics);
    validate_authentication_dependency(&handoff.authentication, &mut diagnostics);
    let mut artifact_ids = BTreeSet::new();
    let mut artifact_paths = BTreeSet::new();
    let mut readiness_ids = BTreeSet::new();
    for row in &handoff.rows {
        validate_row(row, &mut artifact_ids, &mut artifact_paths, &mut readiness_ids, &mut diagnostics);
    }
    if readiness_ids.len() > 1 {
        push_diagnostic(&mut diagnostics, "Cairn handoff rows do not share one release_readiness_id".to_string());
    }
    let is_valid = diagnostics.is_empty();
    debug_assert!(count_fits_limit(diagnostics.len(), MAX_CAIRN_HANDOFF_DIAGNOSTICS_COUNT));
    debug_assert_eq!(is_valid, diagnostics.is_empty());
    CairnReleaseEvidenceReport {
        valid: is_valid,
        diagnostics,
    }
}

pub fn cairn_release_evidence_validation_receipt(
    bundle_binding: CairnReleaseBundleBinding,
    handoff: CairnReleaseEvidenceHandoff,
) -> Result<CairnReleaseEvidenceValidationReceipt, ReleaseEvidenceError> {
    let bundle_binding = canonical_bundle_binding(bundle_binding);
    validate_bundle_binding(&bundle_binding)?;
    let handoff = canonical_handoff(handoff);
    let handoff_validation = validate_cairn_release_evidence_handoff(&handoff);
    if !handoff_validation.valid {
        return Err(validation_error(format!(
            "Cairn handoff validation failed: {}",
            handoff_validation.diagnostics.join("; ")
        )));
    }
    let bundle_binding_blake3 = cairn_release_bundle_binding_digest_blake3(&bundle_binding)?;
    let mut receipt = CairnReleaseEvidenceValidationReceipt {
        schema: CAIRN_HANDOFF_VALIDATION_RECEIPT_SCHEMA.to_string(),
        validation_status: CAIRN_HANDOFF_VALIDATION_STATUS.to_string(),
        authentication_status: CAIRN_HANDOFF_AUTHENTICATION_STATUS.to_string(),
        bundle_binding,
        bundle_binding_blake3,
        handoff,
        boundary: CAIRN_HANDOFF_BOUNDARY.to_string(),
        non_claims: required_non_claims(),
        receipt_blake3: String::new(),
    };
    receipt.receipt_blake3 = cairn_handoff_receipt_digest_blake3(&receipt)?;
    validate_cairn_release_evidence_validation_receipt(&receipt, &receipt.bundle_binding, &receipt.handoff)?;
    debug_assert_eq!(receipt.schema, CAIRN_HANDOFF_VALIDATION_RECEIPT_SCHEMA);
    debug_assert!(valid_blake3(&receipt.receipt_blake3));
    Ok(receipt)
}

// r[impl mantle.release_provenance.cairn_evidence_handoff.bypass_protection]
pub fn validate_cairn_release_evidence_validation_receipt(
    receipt: &CairnReleaseEvidenceValidationReceipt,
    expected_binding: &CairnReleaseBundleBinding,
    measured_handoff: &CairnReleaseEvidenceHandoff,
) -> Result<(), ReleaseEvidenceError> {
    validate_receipt_literals(receipt)?;
    let expected_binding = canonical_bundle_binding(expected_binding.clone());
    validate_bundle_binding(&expected_binding)?;
    if receipt.bundle_binding != expected_binding {
        return Err(validation_error("Cairn handoff validation receipt is bound to another release bundle"));
    }
    let binding_digest = cairn_release_bundle_binding_digest_blake3(&expected_binding)?;
    if receipt.bundle_binding_blake3 != binding_digest {
        return Err(validation_error("Cairn handoff validation receipt bundle binding digest is stale"));
    }
    let measured_handoff = canonical_handoff(measured_handoff.clone());
    let handoff_validation = validate_cairn_release_evidence_handoff(&measured_handoff);
    if !handoff_validation.valid {
        return Err(validation_error(format!(
            "Cairn measured inputs are invalid: {}",
            handoff_validation.diagnostics.join("; ")
        )));
    }
    if receipt.handoff != measured_handoff {
        return Err(validation_error("Cairn handoff validation receipt does not match measured bundle bytes"));
    }
    let receipt_digest = cairn_handoff_receipt_digest_blake3(receipt)?;
    if receipt.receipt_blake3 != receipt_digest {
        return Err(validation_error("Cairn handoff validation receipt digest is stale"));
    }
    debug_assert_eq!(receipt.bundle_binding, expected_binding);
    debug_assert_eq!(receipt.handoff, measured_handoff);
    Ok(())
}

pub fn evaluate_cairn_handoff_release_evidence(
    receipt: Option<&CairnReleaseEvidenceValidationReceipt>,
    expected_binding: &CairnReleaseBundleBinding,
    required: bool,
) -> CairnHandoffReleaseVerification {
    let Some(receipt) = receipt else {
        let diagnostics = if required {
            vec!["required Cairn handoff validation receipt is absent".to_string()]
        } else {
            Vec::new()
        };
        return CairnHandoffReleaseVerification {
            required,
            valid: !required,
            disposition: CAIRN_HANDOFF_DISPOSITION_ABSENT.to_string(),
            bundle_binding_blake3: None,
            receipt_blake3: None,
            authentication_status: None,
            boundary: CAIRN_HANDOFF_BOUNDARY.to_string(),
            diagnostics,
        };
    };
    let result = validate_cairn_release_evidence_validation_receipt(receipt, expected_binding, &receipt.handoff);
    let diagnostics = result.err().map(|error| vec![error.to_string()]).unwrap_or_default();
    let is_valid = diagnostics.is_empty();
    let disposition = if is_valid {
        CAIRN_HANDOFF_DISPOSITION_PRESENT
    } else {
        CAIRN_HANDOFF_DISPOSITION_INVALID
    };
    debug_assert_eq!(is_valid, disposition == CAIRN_HANDOFF_DISPOSITION_PRESENT);
    debug_assert!(!receipt.receipt_blake3.is_empty());
    CairnHandoffReleaseVerification {
        required,
        valid: is_valid,
        disposition: disposition.to_string(),
        bundle_binding_blake3: Some(receipt.bundle_binding_blake3.clone()),
        receipt_blake3: Some(receipt.receipt_blake3.clone()),
        authentication_status: Some(receipt.authentication_status.clone()),
        boundary: CAIRN_HANDOFF_BOUNDARY.to_string(),
        diagnostics,
    }
}

pub fn cairn_release_bundle_binding_digest_blake3(
    binding: &CairnReleaseBundleBinding,
) -> Result<String, ReleaseEvidenceError> {
    let binding = canonical_bundle_binding(binding.clone());
    validate_bundle_binding(&binding)?;
    let bytes = serde_json::to_vec(&binding)
        .map_err(|error| validation_error(format!("serializing Cairn bundle binding: {error}")))?;
    Ok(domain_digest(BUNDLE_BINDING_DOMAIN, &bytes))
}

fn cairn_handoff_receipt_digest_blake3(
    receipt: &CairnReleaseEvidenceValidationReceipt,
) -> Result<String, ReleaseEvidenceError> {
    let mut material = receipt.clone();
    material.receipt_blake3.clear();
    let bytes = serde_json::to_vec(&material)
        .map_err(|error| validation_error(format!("serializing Cairn receipt: {error}")))?;
    Ok(domain_digest(RECEIPT_DOMAIN, &bytes))
}

fn validate_receipt_literals(receipt: &CairnReleaseEvidenceValidationReceipt) -> Result<(), ReleaseEvidenceError> {
    if receipt.schema != CAIRN_HANDOFF_VALIDATION_RECEIPT_SCHEMA {
        return Err(validation_error("unsupported Cairn handoff validation receipt schema"));
    }
    if receipt.validation_status != CAIRN_HANDOFF_VALIDATION_STATUS {
        return Err(validation_error("Cairn handoff receipt does not record bundle-local validation"));
    }
    if receipt.authentication_status != CAIRN_HANDOFF_AUTHENTICATION_STATUS {
        return Err(validation_error(CAIRN_HANDOFF_AUTHENTICATION_BLOCKER));
    }
    if receipt.boundary != CAIRN_HANDOFF_BOUNDARY {
        return Err(validation_error("Cairn handoff receipt changes the Cairn lifecycle boundary"));
    }
    if receipt.non_claims != required_non_claims() {
        return Err(validation_error("Cairn handoff receipt non-claims are incomplete or non-canonical"));
    }
    Ok(())
}

// r[impl mantle.release_provenance.cairn_evidence_handoff.cross_repo_dependency]
fn validate_authentication_dependency(
    dependency: &CairnHandoffAuthenticationDependency,
    diagnostics: &mut Vec<String>,
) {
    debug_assert!(count_fits_limit(diagnostics.len(), MAX_CAIRN_HANDOFF_DIAGNOSTICS_COUNT));
    debug_assert!(!CAIRN_HANDOFF_AUTHENTICATION_SCHEMA.is_empty());
    if dependency.schema != CAIRN_HANDOFF_AUTHENTICATION_SCHEMA {
        push_diagnostic(diagnostics, "unsupported Cairn authentication dependency schema".to_string());
    }
    if dependency.change_name != CAIRN_HANDOFF_AUTHENTICATION_CHANGE {
        push_diagnostic(diagnostics, "Cairn authentication dependency names another change".to_string());
    }
    if dependency.cairn_revision != CAIRN_HANDOFF_AUTHENTICATION_CAIRN_REVISION {
        push_diagnostic(diagnostics, "Cairn authentication dependency revision is stale".to_string());
    }
    if dependency.archive_manifest_blake3 != CAIRN_HANDOFF_AUTHENTICATION_ARCHIVE_MANIFEST_BLAKE3 {
        push_diagnostic(diagnostics, "Cairn authentication archive manifest identity is stale".to_string());
    }
    if dependency.archive_mutation_receipt_blake3 != CAIRN_HANDOFF_AUTHENTICATION_ARCHIVE_MUTATION_RECEIPT_BLAKE3 {
        push_diagnostic(diagnostics, "Cairn authentication archive mutation receipt is stale".to_string());
    }
    validate_measured_artifact("authentication.archive_receipt", &dependency.archive_receipt, diagnostics);
    let has_expected_declared_digest =
        dependency.archive_receipt.declared_digest_blake3 == CAIRN_HANDOFF_AUTHENTICATION_RECEIPT_BLAKE3;
    let has_expected_measured_digest =
        dependency.archive_receipt.measured_digest_blake3 == CAIRN_HANDOFF_AUTHENTICATION_RECEIPT_BLAKE3;
    if !has_expected_declared_digest || !has_expected_measured_digest {
        push_diagnostic(
            diagnostics,
            "Cairn authentication archive receipt bytes are not the reviewed receipt".to_string(),
        );
    }
}

fn validate_row(
    row: &CairnReleaseEvidenceRow,
    artifact_ids: &mut BTreeSet<String>,
    artifact_paths: &mut BTreeSet<String>,
    readiness_ids: &mut BTreeSet<String>,
    diagnostics: &mut Vec<String>,
) {
    debug_assert!(count_fits_limit(diagnostics.len(), MAX_CAIRN_HANDOFF_DIAGNOSTICS_COUNT));
    validate_text(
        TextField {
            label: "artifact_id",
            value: &row.artifact_id,
        },
        diagnostics,
    );
    validate_text(
        TextField {
            label: "release_readiness_id",
            value: &row.release_readiness_id,
        },
        diagnostics,
    );
    validate_role_schema(row, diagnostics);
    validate_measured_artifact("artifact", &row.artifact, diagnostics);
    validate_measured_artifact("cairn_policy", &row.cairn_policy, diagnostics);
    validate_bounded_strings("covers", &row.covers, MAX_CAIRN_HANDOFF_COVERS_COUNT, diagnostics);
    validate_bounded_strings("non_claims", &row.non_claims, MAX_CAIRN_HANDOFF_NON_CLAIMS_COUNT, diagnostics);
    if !artifact_ids.insert(row.artifact_id.clone()) {
        push_diagnostic(diagnostics, format!("duplicate artifact_id {}", row.artifact_id));
    }
    if !artifact_paths.insert(row.artifact.relative_path.clone()) {
        push_diagnostic(diagnostics, format!("duplicate artifact path {}", row.artifact.relative_path));
    }
    if !artifact_paths.insert(row.cairn_policy.relative_path.clone()) {
        push_diagnostic(diagnostics, format!("duplicate artifact or policy path {}", row.cairn_policy.relative_path));
    }
    readiness_ids.insert(row.release_readiness_id.clone());
    debug_assert!(readiness_ids.contains(&row.release_readiness_id));
    let non_claim_text = row.non_claims.join(" ").to_ascii_lowercase();
    if !non_claim_text.contains(REQUIRED_NON_CLAIM_FRAGMENT) {
        push_diagnostic(diagnostics, format!("artifact {} omits the release-correctness non-claim", row.artifact_id));
    }
    for fragment in OVERCLAIM_FRAGMENTS {
        if non_claim_text.contains(fragment) {
            push_diagnostic(diagnostics, format!("artifact {} contains overclaiming text {fragment}", row.artifact_id));
        }
    }
}

fn validate_measured_artifact(label: &str, artifact: &CairnMeasuredArtifact, diagnostics: &mut Vec<String>) {
    debug_assert!(!label.trim().is_empty());
    validate_text(
        TextField {
            label: &format!("{label}.relative_path"),
            value: &artifact.relative_path,
        },
        diagnostics,
    );
    if artifact.relative_path.starts_with('/')
        || artifact.relative_path.split('/').any(|part| part.is_empty() || part == "." || part == "..")
    {
        push_diagnostic(diagnostics, format!("{label}.relative_path must be a normalized relative path"));
    }
    if !valid_blake3(&artifact.declared_digest_blake3) {
        push_diagnostic(diagnostics, format!("{label}.declared_digest_blake3 must be lowercase BLAKE3 hex"));
    }
    if !valid_blake3(&artifact.measured_digest_blake3) {
        push_diagnostic(diagnostics, format!("{label}.measured_digest_blake3 must be lowercase BLAKE3 hex"));
    }
    if artifact.declared_digest_blake3 != artifact.measured_digest_blake3 {
        push_diagnostic(diagnostics, format!("{label} declared digest does not match shell-measured bytes"));
    }
    debug_assert!(count_fits_limit(diagnostics.len(), MAX_CAIRN_HANDOFF_DIAGNOSTICS_COUNT));
}

fn validate_role_schema(row: &CairnReleaseEvidenceRow, diagnostics: &mut Vec<String>) {
    let is_role_schema_allowed = SUPPORTED_ROLE_SCHEMA_PAIRS
        .iter()
        .any(|(role, schema)| row.role == *role && row.schema_id == *schema);
    if !is_role_schema_allowed {
        push_diagnostic(diagnostics, format!("unsupported Cairn role/schema pair {}/{}", row.role, row.schema_id));
    }
}

fn validate_bundle_binding(binding: &CairnReleaseBundleBinding) -> Result<(), ReleaseEvidenceError> {
    let release_id_bytes = u32::try_from(binding.release_id.len())
        .map_err(|_| validation_error("Cairn bundle binding release_id length overflowed u32"))?;
    if binding.release_id.is_empty() || release_id_bytes > MAX_CAIRN_HANDOFF_TEXT_BYTES_COUNT {
        return Err(validation_error("Cairn bundle binding release_id is empty or oversized"));
    }
    validate_bundle_binding_digests(binding)?;
    validate_bundle_binding_binary_digests(&binding.binary_digests_blake3)?;
    debug_assert!(release_id_bytes > 0);
    debug_assert!(release_id_bytes <= MAX_CAIRN_HANDOFF_TEXT_BYTES_COUNT);
    Ok(())
}

fn validate_bundle_binding_digests(binding: &CairnReleaseBundleBinding) -> Result<(), ReleaseEvidenceError> {
    let digests = [
        &binding.source_archive_digest_blake3,
        &binding.proof_bundle_digest_blake3,
        &binding.prerequisite_inventory_digest_blake3,
        &binding.release_manifest_projection_blake3,
    ];
    if digests.into_iter().any(|digest| !valid_blake3(digest)) {
        return Err(validation_error("Cairn bundle binding contains an invalid BLAKE3 identity"));
    }
    Ok(())
}

fn validate_bundle_binding_binary_digests(binary_digests: &[String]) -> Result<(), ReleaseEvidenceError> {
    let binary_digest_count = u32::try_from(binary_digests.len())
        .map_err(|_| validation_error("Cairn bundle binding binary digest count overflowed u32"))?;
    if binary_digest_count == 0 || binary_digest_count > MAX_CAIRN_HANDOFF_BINARY_DIGESTS_COUNT {
        return Err(validation_error("Cairn bundle binding binary digest count is out of bounds"));
    }
    if binary_digests.iter().any(|digest| !valid_blake3(digest)) {
        return Err(validation_error("Cairn bundle binding contains an invalid binary BLAKE3 identity"));
    }
    Ok(())
}

fn validate_row_count(row_count: usize, diagnostics: &mut Vec<String>) {
    let Ok(row_count_u32) = u32::try_from(row_count) else {
        push_diagnostic(diagnostics, "Cairn handoff row count overflowed u32".to_string());
        return;
    };
    if row_count_u32 == 0 || row_count_u32 > MAX_CAIRN_HANDOFF_ROWS_COUNT {
        push_diagnostic(
            diagnostics,
            format!("Cairn handoff row count must be between 1 and {}", MAX_CAIRN_HANDOFF_ROWS_COUNT),
        );
    }
}

fn validate_bounded_strings(label: &str, values: &[String], maximum_count: u32, diagnostics: &mut Vec<String>) {
    let value_count = match u32::try_from(values.len()) {
        Ok(value_count) => value_count,
        Err(_) => {
            push_diagnostic(diagnostics, format!("{label} count overflowed u32"));
            return;
        }
    };
    if value_count == 0 || value_count > maximum_count {
        push_diagnostic(diagnostics, format!("{label} count must be between 1 and {maximum_count}"));
    }
    let mut unique = BTreeSet::new();
    for value in values {
        validate_text(TextField { label, value }, diagnostics);
        if !unique.insert(value) {
            push_diagnostic(diagnostics, format!("{label} contains a duplicate value"));
        }
    }
}

fn validate_text(field: TextField<'_>, diagnostics: &mut Vec<String>) {
    let value_bytes = match u32::try_from(field.value.len()) {
        Ok(value_bytes) => value_bytes,
        Err(_) => {
            push_diagnostic(diagnostics, format!("{} length overflowed u32", field.label));
            return;
        }
    };
    if field.value.is_empty() || value_bytes > MAX_CAIRN_HANDOFF_TEXT_BYTES_COUNT {
        push_diagnostic(diagnostics, format!("{} is empty or oversized", field.label));
    }
}

fn canonical_handoff(mut handoff: CairnReleaseEvidenceHandoff) -> CairnReleaseEvidenceHandoff {
    for row in &mut handoff.rows {
        row.covers.sort();
        row.non_claims.sort();
    }
    handoff.rows.sort_by(|left, right| left.artifact_id.cmp(&right.artifact_id));
    handoff
}

fn canonical_bundle_binding(mut binding: CairnReleaseBundleBinding) -> CairnReleaseBundleBinding {
    binding.binary_digests_blake3.sort();
    binding
}

fn required_non_claims() -> Vec<String> {
    let mut values = vec![
        CAIRN_HANDOFF_NON_CLAIM_RELEASE_CORRECTNESS.to_string(),
        CAIRN_HANDOFF_NON_CLAIM_BUILD_CORRECTNESS.to_string(),
        CAIRN_HANDOFF_NON_CLAIM_SOURCE_CORRECTNESS.to_string(),
        CAIRN_HANDOFF_NON_CLAIM_DEPLOYMENT_SAFETY.to_string(),
    ];
    values.sort();
    values
}

fn valid_blake3(value: &str) -> bool {
    value.len() == BLAKE3_HEX_LENGTH_CHARS
        && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn domain_digest(domain: &[u8], bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    hasher.update(bytes);
    hasher.finalize().to_hex().to_string()
}

fn push_diagnostic(diagnostics: &mut Vec<String>, diagnostic: String) {
    let Ok(diagnostic_count) = u32::try_from(diagnostics.len()) else {
        return;
    };
    if diagnostic_count < MAX_CAIRN_HANDOFF_DIAGNOSTICS_COUNT {
        diagnostics.push(diagnostic);
    }
}

fn count_fits_limit(count: usize, maximum_count: u32) -> bool {
    u32::try_from(count).is_ok_and(|count_u32| count_u32 <= maximum_count)
}

fn validation_error(message: impl Into<String>) -> ReleaseEvidenceError {
    ReleaseEvidenceError::Validation(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const TEST_ARTIFACT_SIZE_BYTES: u64 = 3;

    fn artifact(path: &str, digest: &str) -> CairnMeasuredArtifact {
        CairnMeasuredArtifact {
            relative_path: path.to_string(),
            size_bytes: TEST_ARTIFACT_SIZE_BYTES,
            declared_digest_blake3: digest.to_string(),
            measured_digest_blake3: digest.to_string(),
        }
    }

    fn authentication() -> CairnHandoffAuthenticationDependency {
        CairnHandoffAuthenticationDependency {
            schema: CAIRN_HANDOFF_AUTHENTICATION_SCHEMA.to_string(),
            change_name: CAIRN_HANDOFF_AUTHENTICATION_CHANGE.to_string(),
            cairn_revision: CAIRN_HANDOFF_AUTHENTICATION_CAIRN_REVISION.to_string(),
            archive_manifest_blake3: CAIRN_HANDOFF_AUTHENTICATION_ARCHIVE_MANIFEST_BLAKE3.to_string(),
            archive_mutation_receipt_blake3: CAIRN_HANDOFF_AUTHENTICATION_ARCHIVE_MUTATION_RECEIPT_BLAKE3.to_string(),
            archive_receipt: artifact(
                "cairn/authentication/archive-receipt.json",
                CAIRN_HANDOFF_AUTHENTICATION_RECEIPT_BLAKE3,
            ),
        }
    }

    fn handoff() -> CairnReleaseEvidenceHandoff {
        CairnReleaseEvidenceHandoff {
            authentication: authentication(),
            rows: vec![CairnReleaseEvidenceRow {
                artifact_id: "readiness".to_string(),
                role: "cairn-release-readiness-receipt".to_string(),
                schema_id: "cairn.release-readiness.v1".to_string(),
                artifact: artifact("cairn/readiness.json", DIGEST_A),
                cairn_policy: artifact("cairn/policy.ncl", DIGEST_B),
                release_readiness_id: "release-ready-1".to_string(),
                covers: vec!["mantle.release".to_string()],
                non_claims: vec!["not release correctness".to_string()],
            }],
        }
    }

    fn binding() -> CairnReleaseBundleBinding {
        CairnReleaseBundleBinding {
            release_id: "release-1".to_string(),
            source_archive_digest_blake3: DIGEST_A.to_string(),
            binary_digests_blake3: vec![DIGEST_B.to_string()],
            proof_bundle_digest_blake3: DIGEST_A.to_string(),
            prerequisite_inventory_digest_blake3: DIGEST_B.to_string(),
            release_manifest_projection_blake3: DIGEST_A.to_string(),
        }
    }

    #[test]
    fn measured_handoff_and_same_bundle_receipt_pass() {
        let receipt = cairn_release_evidence_validation_receipt(binding(), handoff()).expect("receipt");
        assert_eq!(receipt.authentication_status, CAIRN_HANDOFF_AUTHENTICATION_STATUS);
        assert!(validate_cairn_release_evidence_validation_receipt(&receipt, &binding(), &handoff()).is_ok());
    }

    #[test]
    fn stale_bytes_and_cross_bundle_reuse_fail_closed() {
        let receipt = cairn_release_evidence_validation_receipt(binding(), handoff()).expect("receipt");
        let mut stale = handoff();
        stale.rows[0].artifact.measured_digest_blake3 = DIGEST_B.to_string();
        assert!(validate_cairn_release_evidence_validation_receipt(&receipt, &binding(), &stale).is_err());
        let mut other_binding = binding();
        other_binding.release_id = "release-2".to_string();
        assert!(validate_cairn_release_evidence_validation_receipt(&receipt, &other_binding, &handoff()).is_err());
    }

    #[test]
    fn artifact_and_policy_paths_must_be_globally_unique() {
        let mut duplicate = handoff();
        duplicate.rows[0].cairn_policy.relative_path = duplicate.rows[0].artifact.relative_path.clone();
        let report = validate_cairn_release_evidence_handoff(&duplicate);
        assert!(!report.valid);
        assert!(report.diagnostics.iter().any(|item| item.contains("duplicate")));
    }

    #[test]
    fn stale_authentication_and_role_schema_swap_fail_closed() {
        let mut stale_authentication = handoff();
        stale_authentication.authentication.archive_receipt.measured_digest_blake3 = DIGEST_A.to_string();
        let report = validate_cairn_release_evidence_handoff(&stale_authentication);
        assert!(!report.valid);
        assert!(report.diagnostics.iter().any(|item| item.contains("reviewed receipt")));
        let mut receipt = cairn_release_evidence_validation_receipt(binding(), handoff()).expect("receipt");
        receipt.authentication_status = "not-authenticated".to_string();
        let error = validate_cairn_release_evidence_validation_receipt(&receipt, &binding(), &handoff())
            .expect_err("authentication downgrade");
        assert!(error.to_string().contains("pinned archived"));
        let mut wrong_pair = handoff();
        wrong_pair.rows[0].schema_id = "cairn.archive-index.v1".to_string();
        assert!(!validate_cairn_release_evidence_handoff(&wrong_pair).valid);
    }

    #[test]
    fn required_profile_rejects_missing_receipt() {
        let verification = evaluate_cairn_handoff_release_evidence(None, &binding(), true);
        assert!(!verification.valid);
        assert_eq!(verification.disposition, CAIRN_HANDOFF_DISPOSITION_ABSENT);
    }
}
