use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::manifest::ReleaseEvidenceManifest;
use crate::opaque_evidence::OPAQUE_EVIDENCE_KIND_PROOF;
use crate::opaque_evidence::OpaqueEvidenceCompatibilityProjection;
use crate::opaque_evidence::OpaqueEvidenceSidecarBinding;
use crate::opaque_evidence::TRELLIS_PROOF_PROFILE_VERSION;
use crate::opaque_evidence::opaque_evidence_sidecar_binding_diagnostics;

pub const TRELLIS_PROOF_MODE_OPTIONAL: &str = "optional";
pub const TRELLIS_PROOF_MODE_REQUIRED: &str = "required";
pub const TRELLIS_PROOF_DISPOSITION_ABSENT: &str = "absent";
pub const TRELLIS_PROOF_DISPOSITION_RECORDED_ONLY: &str = "recorded-only";
pub const TRELLIS_PROOF_DISPOSITION_INVALID: &str = "invalid";
pub const TRELLIS_PROOF_RELEASE_BOUNDARY: &str = "Mantle validates bounded Trellis proof artifact identities and release linkage only; canonical Preserves bytes remain opaque, Kamacite owns producer roles, Valence owns proof acceptance, and Cairn owns lifecycle readiness";
pub const TRELLIS_PROOF_ACCEPTANCE_AUTHORITY_BLOCKER: &str = "required Trellis proof evidence needs an upstream accepted-validation role; the registered profile supports Valence recorded_only validation only";
pub const MAX_TRELLIS_PROOF_DIAGNOSTICS_COUNT: u32 = 32;
const DUPLICATE_DETECTION_COUNT: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrellisProofArtifactObservations {
    pub canonical_envelope_size_bytes: u64,
    pub canonical_envelope_digest_blake3: String,
    pub valence_artifact_digest_blake3: String,
    pub valence_receipt_hash_blake3: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projection_artifact_digest_blake3: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projection_canonical_envelope_digest_blake3: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrellisProofReleaseVerification {
    pub mode: String,
    pub required: bool,
    pub valid: bool,
    pub disposition: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub producer_role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation_role: Option<String>,
    pub boundary: String,
    pub diagnostics: Vec<String>,
}

// r[impl mantle.release_provenance.trellis_proof_sidecars.validation]
// r[impl mantle.release_provenance.trellis_proof_sidecars.opaque]
#[must_use]
pub fn evaluate_trellis_proof_release_evidence(
    manifest: &ReleaseEvidenceManifest,
    mode: &str,
    observations: Option<&TrellisProofArtifactObservations>,
) -> TrellisProofReleaseVerification {
    let required = mode == TRELLIS_PROOF_MODE_REQUIRED;
    let mut diagnostics = validate_mode(mode);
    let matches = matching_profile_bindings(manifest);
    if matches.len() > 1 {
        diagnostics.push("multiple Trellis proof sidecar bindings match the registered profile".to_string());
        return finish_verification(mode, required, None, diagnostics);
    }
    let Some(binding) = matches.first().copied() else {
        validate_absent_binding(required, observations, &mut diagnostics);
        return finish_verification(mode, required, None, diagnostics);
    };
    diagnostics.extend(opaque_evidence_sidecar_binding_diagnostics(
        binding,
        &manifest.source_archive,
        &manifest.binaries,
        &manifest.external_evidence,
    ));
    validate_observations(&binding.binding, observations, &mut diagnostics);
    if required {
        diagnostics.push(TRELLIS_PROOF_ACCEPTANCE_AUTHORITY_BLOCKER.to_string());
    }
    finish_verification(mode, required, Some(&binding.binding), diagnostics)
}

fn matching_profile_bindings(
    manifest: &ReleaseEvidenceManifest,
) -> Vec<&crate::opaque_evidence::OpaqueEvidenceSidecarBindingReceipt> {
    let matches = manifest
        .opaque_evidence_sidecar_bindings
        .iter()
        .filter(|receipt| {
            receipt.binding.evidence_kind == OPAQUE_EVIDENCE_KIND_PROOF
                && receipt.binding.profile_version == TRELLIS_PROOF_PROFILE_VERSION
        })
        .take(DUPLICATE_DETECTION_COUNT)
        .collect::<Vec<_>>();
    debug_assert!(matches.len() <= DUPLICATE_DETECTION_COUNT);
    debug_assert!(matches.iter().all(|receipt| receipt.binding.evidence_kind == OPAQUE_EVIDENCE_KIND_PROOF));
    matches
}

fn validate_mode(mode: &str) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if !matches!(mode, TRELLIS_PROOF_MODE_OPTIONAL | TRELLIS_PROOF_MODE_REQUIRED) {
        diagnostics.push(format_mode_error(mode));
    }
    debug_assert!(matches!(mode, TRELLIS_PROOF_MODE_OPTIONAL | TRELLIS_PROOF_MODE_REQUIRED) || !diagnostics.is_empty());
    debug_assert!(diagnostics.len() <= 1);
    diagnostics
}

fn format_mode_error(mode: &str) -> String {
    alloc::format!("unsupported Trellis proof release mode: {mode}")
}

fn validate_absent_binding(
    required: bool,
    observations: Option<&TrellisProofArtifactObservations>,
    diagnostics: &mut Vec<String>,
) {
    if required {
        diagnostics.push("required Trellis proof sidecar binding is absent".to_string());
    }
    if observations.is_some() {
        diagnostics.push("Trellis proof artifact observations require a matching sidecar binding".to_string());
    }
    debug_assert!(!required || !diagnostics.is_empty());
    debug_assert!(observations.is_none() || !diagnostics.is_empty());
}

fn validate_observations(
    binding: &OpaqueEvidenceSidecarBinding,
    observations: Option<&TrellisProofArtifactObservations>,
    diagnostics: &mut Vec<String>,
) {
    let Some(observed) = observations else {
        diagnostics.push("Trellis proof artifact observations are required for a present binding".to_string());
        return;
    };
    compare_u64(
        observed.canonical_envelope_size_bytes,
        binding.canonical_envelope.size_bytes,
        "canonical Preserves byte size",
        diagnostics,
    );
    compare_string(
        &observed.canonical_envelope_digest_blake3,
        &binding.canonical_envelope.digest_blake3,
        "canonical Preserves artifact digest",
        diagnostics,
    );
    compare_string(
        &observed.valence_artifact_digest_blake3,
        &binding.upstream_validation.digest_blake3,
        "Valence artifact digest",
        diagnostics,
    );
    compare_optional_string(
        Some(&observed.valence_receipt_hash_blake3),
        binding.upstream_validation.receipt_hash_blake3.as_ref(),
        "Valence logical receipt hash",
        diagnostics,
    );
    validate_projection_observations(&binding.compatibility_projections, observed, diagnostics);
}

fn validate_projection_observations(
    projections: &[OpaqueEvidenceCompatibilityProjection],
    observed: &TrellisProofArtifactObservations,
    diagnostics: &mut Vec<String>,
) {
    let projection = projections.first();
    compare_optional_string(
        observed.projection_artifact_digest_blake3.as_ref(),
        projection.map(|value| &value.digest_blake3),
        "JSON projection artifact digest",
        diagnostics,
    );
    compare_optional_string(
        observed.projection_canonical_envelope_digest_blake3.as_ref(),
        projection.map(|value| &value.canonical_envelope_digest_blake3),
        "JSON projection canonical Preserves identity",
        diagnostics,
    );
    debug_assert!(projections.len() <= 1 || !diagnostics.is_empty());
    debug_assert_eq!(projection.is_some(), !projections.is_empty());
}

fn compare_u64(actual: u64, expected: Option<u64>, label: &str, diagnostics: &mut Vec<String>) {
    match expected {
        Some(expected) if actual == expected => {}
        Some(expected) => diagnostics.push(alloc::format!("{label} mismatch: expected {expected}, got {actual}")),
        None => diagnostics.push(alloc::format!("{label} is missing from the typed binding")),
    }
    debug_assert!(expected == Some(actual) || diagnostics.iter().any(|diagnostic| diagnostic.contains(label)));
    debug_assert!(!label.is_empty());
}

fn compare_string(actual: &str, expected: &str, label: &str, diagnostics: &mut Vec<String>) {
    if actual != expected {
        diagnostics.push(alloc::format!("{label} mismatch: expected {expected}, got {actual}"));
    }
    debug_assert!(actual == expected || diagnostics.iter().any(|diagnostic| diagnostic.contains(label)));
    debug_assert!(!label.is_empty());
}

fn compare_optional_string(
    actual: Option<&String>,
    expected: Option<&String>,
    label: &str,
    diagnostics: &mut Vec<String>,
) {
    if actual != expected {
        diagnostics.push(alloc::format!("{label} mismatch"));
    }
    debug_assert!(actual == expected || diagnostics.iter().any(|diagnostic| diagnostic.contains(label)));
    debug_assert!(!label.is_empty());
}

fn finish_verification(
    mode: &str,
    required: bool,
    binding: Option<&OpaqueEvidenceSidecarBinding>,
    mut diagnostics: Vec<String>,
) -> TrellisProofReleaseVerification {
    diagnostics.sort();
    diagnostics.dedup();
    let diagnostics_limit = usize::try_from(MAX_TRELLIS_PROOF_DIAGNOSTICS_COUNT)
        .expect("Trellis proof diagnostic limit fits the target pointer width");
    diagnostics.truncate(diagnostics_limit);
    let valid = diagnostics.is_empty();
    let roles = binding.and_then(|value| value.profile_roles.as_ref());
    let disposition = if valid && binding.is_none() {
        TRELLIS_PROOF_DISPOSITION_ABSENT
    } else if valid {
        TRELLIS_PROOF_DISPOSITION_RECORDED_ONLY
    } else {
        TRELLIS_PROOF_DISPOSITION_INVALID
    };
    let verification = TrellisProofReleaseVerification {
        mode: mode.to_string(),
        required,
        valid,
        disposition: disposition.to_string(),
        producer_role: roles.map(|value| value.producer_role.clone()),
        validation_role: roles.map(|value| value.validation_role.clone()),
        boundary: TRELLIS_PROOF_RELEASE_BOUNDARY.to_string(),
        diagnostics,
    };
    debug_assert_eq!(verification.valid, verification.diagnostics.is_empty());
    debug_assert!(!verification.boundary.is_empty());
    verification
}
