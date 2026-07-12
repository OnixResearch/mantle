use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::ReleaseEvidenceError;
use crate::manifest::BundledArtifact;
use crate::manifest::ExternalEvidence;
use crate::manifest::u32_count;
use crate::manifest::validate_blake3_hex;
use crate::manifest::validate_relative_member_path;
use crate::manifest::validation_error;

pub const OPAQUE_EVIDENCE_SIDECAR_BINDING_SCHEMA: &str = "mantle-opaque-evidence-sidecar-binding-v1";
pub const OPAQUE_EVIDENCE_SIDECAR_BINDING_RECEIPT_SCHEMA: &str = "mantle-opaque-evidence-sidecar-binding-receipt-v1";
pub const OPAQUE_EVIDENCE_KIND_FUNCTION_ADDRESS: &str = "function-address";
pub const OPAQUE_EVIDENCE_KIND_PROOF: &str = "proof";
pub const OPAQUE_EVIDENCE_KIND_LINT: &str = "lint";
pub const OPAQUE_EVIDENCE_KIND_DEPENDENCY: &str = "dependency";
pub const OPAQUE_EVIDENCE_KIND_BUILD: &str = "build";
pub const OPAQUE_EVIDENCE_KIND_ATTESTATION: &str = "attestation";
pub const OPAQUE_EVIDENCE_KIND_LIFECYCLE: &str = "lifecycle";
pub const OPAQUE_EVIDENCE_GENERIC_CLAIM_SCOPE: &str = "opaque-evidence-identity-linkage-only";
pub const OPAQUE_EVIDENCE_POLICY_KIND_UPSTREAM_PROFILE: &str = "upstream-profile-policy";
pub const OPAQUE_EVIDENCE_POLICY_KIND_MANTLE_RELEASE: &str = "mantle-release-policy";
pub const OPAQUE_EVIDENCE_REQUIRED_NON_CLAIM: &str =
    "Mantle validates opaque evidence metadata and typed BLAKE3 links only; upstream producers own payload semantics";

pub const FUNCTION_ADDRESS_EVIDENCE_ROLE: &str = "function-address-evidence-sidecar";
pub const FUNCTION_ADDRESS_EVIDENCE_SCHEMA: &str = "valence.function-address-evidence.v1";
pub const FUNCTION_ADDRESS_PROFILE_VERSION: &str = "function-address-evidence-v1";
pub const FUNCTION_ADDRESS_CLAIM_SCOPE: &str = "function-address-identity-linkage-only";
pub const VALENCE_FUNCTION_ADDRESS_RECEIPT_ROLE: &str = "valence-function-address-evidence-profile";
pub const VALENCE_FUNCTION_ADDRESS_RECEIPT_SCHEMA: &str = "function-address-evidence-v1";
pub const KAMACITE_FUNCTION_ADDRESS_RECEIPT_ROLE: &str = "kamacite-function-address-receipt";
pub const KAMACITE_FUNCTION_ADDRESS_RECEIPT_SCHEMA: &str = "kamacite.function-address-receipt.v1";
pub const FUNCTION_ADDRESS_MODE_OPTIONAL: &str = "optional";
pub const FUNCTION_ADDRESS_MODE_REQUIRED: &str = "required";
pub const FUNCTION_ADDRESS_DISPOSITION_ABSENT: &str = "absent";
pub const FUNCTION_ADDRESS_DISPOSITION_PRESENT: &str = "present";
pub const FUNCTION_ADDRESS_DISPOSITION_INVALID: &str = "invalid";
pub const FUNCTION_ADDRESS_OPAQUE_BOUNDARY: &str = "Mantle validates bundle-local function-address evidence path, digest, role, schema, claim scope, source archive identity, binary identity, and non-claims only; Octet owns Rust extraction, Kamacite owns portable receipts, and Valence owns evidence semantics";

pub const MAX_OPAQUE_EVIDENCE_BINDINGS_COUNT: u32 = 32;
const MAX_OPAQUE_EVIDENCE_POLICY_HASHES_COUNT: u32 = 16;
const MAX_OPAQUE_EVIDENCE_PROJECTIONS_COUNT: u32 = 16;
const MIN_OPAQUE_EVIDENCE_NON_CLAIMS_COUNT: u32 = 1;
const MAX_OPAQUE_EVIDENCE_NON_CLAIMS_COUNT: u32 = 32;
const MAX_OPAQUE_EVIDENCE_EXTERNAL_ROWS_COUNT: u32 = 64;
const MAX_OPAQUE_EVIDENCE_IDENTIFIER_BYTES_COUNT: u32 = 512;
const OPAQUE_EVIDENCE_BINDING_DIGEST_DOMAIN: &[u8] = b"mantle.opaque-evidence-sidecar-binding.v1\0";
const REQUIRED_POLICY_KINDS_COUNT: u32 = 2;
const REQUIRED_POLICY_KINDS: &[&str] = &[
    OPAQUE_EVIDENCE_POLICY_KIND_UPSTREAM_PROFILE,
    OPAQUE_EVIDENCE_POLICY_KIND_MANTLE_RELEASE,
];
const OPAQUE_EVIDENCE_OVERCLAIM_FRAGMENTS: &[&str] = &[
    "mantle verifies payload semantics",
    "mantle verifies valence semantics",
    "mantle proves release eligibility",
    "proves verifier soundness",
];
const FUNCTION_ADDRESS_OVERCLAIM_FRAGMENTS: &[&str] = &[
    "mantle verifies rust semantics",
    "mantle verifies function semantics",
    "mantle proves function correctness",
    "proves rust semantic correctness",
    "proves source-code correctness",
    "proves whole-program safety",
    "proves release eligibility",
    "proves verifier soundness",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpaqueEvidenceCanonicalEnvelopeLink {
    pub role: String,
    pub schema: String,
    pub relative_path: String,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpaqueEvidenceUpstreamValidationLink {
    pub role: String,
    pub schema: String,
    pub relative_path: String,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpaqueEvidenceSourceArtifactLink {
    pub relative_path: String,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpaqueEvidenceReleaseBinaryLink {
    pub relative_path: String,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpaqueEvidencePolicyHash {
    pub policy_kind: String,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpaqueEvidenceCompatibilityProjection {
    pub role: String,
    pub schema: String,
    pub relative_path: String,
    pub digest_blake3: String,
    pub canonical_envelope_digest_blake3: String,
}

fn empty_compatibility_projections() -> Vec<OpaqueEvidenceCompatibilityProjection> {
    Vec::new()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpaqueEvidenceSidecarBinding {
    pub schema: String,
    pub evidence_kind: String,
    pub profile_version: String,
    pub canonical_envelope: OpaqueEvidenceCanonicalEnvelopeLink,
    pub upstream_validation: OpaqueEvidenceUpstreamValidationLink,
    pub source_artifact: OpaqueEvidenceSourceArtifactLink,
    pub release_binary: OpaqueEvidenceReleaseBinaryLink,
    pub policy_hashes: Vec<OpaqueEvidencePolicyHash>,
    pub claim_scope: String,
    #[serde(default = "empty_compatibility_projections", skip_serializing_if = "Vec::is_empty")]
    pub compatibility_projections: Vec<OpaqueEvidenceCompatibilityProjection>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpaqueEvidenceSidecarBindingReceipt {
    pub schema: String,
    pub binding_digest_blake3: String,
    pub binding: OpaqueEvidenceSidecarBinding,
}

struct DiagnosticField<'a> {
    value: &'a str,
    field_name: &'a str,
}

struct LiteralDiagnosticField<'a> {
    actual: &'a str,
    expected: &'a str,
    field_name: &'a str,
}

struct ExternalArtifactFields<'a> {
    role: &'a str,
    schema: &'a str,
    relative_path: &'a str,
    digest_blake3: &'a str,
    field_name: &'a str,
}

struct CollectionCountBounds<'a> {
    actual_count: usize,
    minimum_count: u32,
    maximum_count: u32,
    field_name: &'a str,
}

// r[impl mantle.release_provenance.opaque_evidence_sidecar_binding.contract]
// r[impl mantle.release_provenance.opaque_evidence_sidecar_binding.links]
pub fn opaque_evidence_sidecar_binding_receipt(
    binding: OpaqueEvidenceSidecarBinding,
) -> Result<OpaqueEvidenceSidecarBindingReceipt, ReleaseEvidenceError> {
    let binding = canonical_opaque_evidence_sidecar_binding(binding)?;
    let binding_digest_blake3 = digest_canonical_binding(&binding)?;
    let receipt = OpaqueEvidenceSidecarBindingReceipt {
        schema: OPAQUE_EVIDENCE_SIDECAR_BINDING_RECEIPT_SCHEMA.to_string(),
        binding_digest_blake3,
        binding,
    };
    debug_assert_eq!(receipt.schema, OPAQUE_EVIDENCE_SIDECAR_BINDING_RECEIPT_SCHEMA);
    debug_assert!(!receipt.binding_digest_blake3.is_empty());
    Ok(receipt)
}

pub fn opaque_evidence_sidecar_binding_canonical_bytes(
    binding: OpaqueEvidenceSidecarBinding,
) -> Result<Vec<u8>, ReleaseEvidenceError> {
    let binding = canonical_opaque_evidence_sidecar_binding(binding)?;
    serialize_binding(&binding)
}

pub fn opaque_evidence_sidecar_binding_digest_blake3(
    binding: OpaqueEvidenceSidecarBinding,
) -> Result<String, ReleaseEvidenceError> {
    let binding = canonical_opaque_evidence_sidecar_binding(binding)?;
    digest_canonical_binding(&binding)
}

fn canonical_opaque_evidence_sidecar_binding(
    binding: OpaqueEvidenceSidecarBinding,
) -> Result<OpaqueEvidenceSidecarBinding, ReleaseEvidenceError> {
    let binding = normalize_binding(binding);
    let diagnostics = binding_metadata_diagnostics(&binding);
    if let Some(first) = diagnostics.first() {
        return Err(validation_error(format!("opaque evidence sidecar binding invalid: {first}")));
    }
    debug_assert!(diagnostics.is_empty());
    debug_assert!(!binding.policy_hashes.is_empty());
    Ok(binding)
}

fn normalize_binding(mut binding: OpaqueEvidenceSidecarBinding) -> OpaqueEvidenceSidecarBinding {
    binding.policy_hashes.sort_by(|left, right| {
        left.policy_kind.cmp(&right.policy_kind).then(left.digest_blake3.cmp(&right.digest_blake3))
    });
    binding.compatibility_projections.sort_by(|left, right| {
        left.role
            .cmp(&right.role)
            .then(left.schema.cmp(&right.schema))
            .then(left.relative_path.cmp(&right.relative_path))
    });
    binding.non_claims.sort();
    binding
}

fn serialize_binding(binding: &OpaqueEvidenceSidecarBinding) -> Result<Vec<u8>, ReleaseEvidenceError> {
    serde_json::to_vec(binding)
        .map_err(|error| validation_error(format!("serializing opaque evidence sidecar binding: {error}")))
}

fn digest_canonical_binding(binding: &OpaqueEvidenceSidecarBinding) -> Result<String, ReleaseEvidenceError> {
    let canonical_bytes = serialize_binding(binding)?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(OPAQUE_EVIDENCE_BINDING_DIGEST_DOMAIN);
    hasher.update(&canonical_bytes);
    Ok(hasher.finalize().to_hex().to_string())
}

// r[impl mantle.release_provenance.opaque_evidence_sidecar_binding.validation]
pub(crate) fn opaque_evidence_sidecar_binding_diagnostics(
    receipt: &OpaqueEvidenceSidecarBindingReceipt,
    source_archive: &BundledArtifact,
    binaries: &[BundledArtifact],
    external_evidence: &[ExternalEvidence],
) -> Vec<String> {
    let mut diagnostics = receipt_identity_diagnostics(receipt);
    diagnostics.extend(binding_metadata_diagnostics(&receipt.binding));
    diagnostics.extend(release_artifact_link_diagnostics(&receipt.binding, source_archive, binaries));
    diagnostics.extend(external_evidence_link_diagnostics(&receipt.binding, external_evidence));
    diagnostics
}

fn receipt_identity_diagnostics(receipt: &OpaqueEvidenceSidecarBindingReceipt) -> Vec<String> {
    let mut diagnostics = Vec::new();
    push_literal_diagnostic(
        LiteralDiagnosticField {
            actual: &receipt.schema,
            expected: OPAQUE_EVIDENCE_SIDECAR_BINDING_RECEIPT_SCHEMA,
            field_name: "receipt.schema",
        },
        &mut diagnostics,
    );
    push_blake3_diagnostic(
        DiagnosticField {
            value: &receipt.binding_digest_blake3,
            field_name: "receipt.binding_digest_blake3",
        },
        &mut diagnostics,
    );
    let normalized = normalize_binding(receipt.binding.clone());
    if normalized != receipt.binding {
        diagnostics.push("receipt.binding metadata is not in canonical order".to_string());
    }
    match digest_canonical_binding(&normalized) {
        Ok(expected_digest) if expected_digest != receipt.binding_digest_blake3 => diagnostics
            .push("receipt.binding_digest_blake3 does not match canonical typed binding metadata".to_string()),
        Ok(_) | Err(_) => {}
    }
    debug_assert!(
        receipt.schema == OPAQUE_EVIDENCE_SIDECAR_BINDING_RECEIPT_SCHEMA
            || diagnostics.iter().any(|diagnostic| diagnostic.contains("receipt.schema"))
    );
    debug_assert!(
        validate_blake3_hex(&receipt.binding_digest_blake3, "receipt.binding_digest_blake3").is_ok()
            || diagnostics.iter().any(|diagnostic| diagnostic.contains("receipt.binding_digest_blake3"))
    );
    diagnostics
}

fn binding_metadata_diagnostics(binding: &OpaqueEvidenceSidecarBinding) -> Vec<String> {
    let mut diagnostics = Vec::new();
    push_literal_diagnostic(
        LiteralDiagnosticField {
            actual: &binding.schema,
            expected: OPAQUE_EVIDENCE_SIDECAR_BINDING_SCHEMA,
            field_name: "binding.schema",
        },
        &mut diagnostics,
    );
    validate_evidence_kind_and_profile(binding, &mut diagnostics);
    validate_canonical_envelope_link(&binding.canonical_envelope, &mut diagnostics);
    validate_upstream_validation_link(&binding.upstream_validation, &mut diagnostics);
    validate_source_artifact_link(&binding.source_artifact, &mut diagnostics);
    validate_release_binary_link(&binding.release_binary, &mut diagnostics);
    validate_policy_hashes(&binding.policy_hashes, &mut diagnostics);
    validate_compatibility_projections(binding, &mut diagnostics);
    validate_binding_non_claims(binding, &mut diagnostics);
    debug_assert!(
        binding.schema == OPAQUE_EVIDENCE_SIDECAR_BINDING_SCHEMA
            || diagnostics.iter().any(|diagnostic| diagnostic.contains("binding.schema"))
    );
    debug_assert!(
        evidence_kind_is_supported(&binding.evidence_kind)
            || diagnostics.iter().any(|diagnostic| diagnostic.contains("evidence_kind is unsupported"))
    );
    diagnostics
}

fn validate_evidence_kind_and_profile(binding: &OpaqueEvidenceSidecarBinding, diagnostics: &mut Vec<String>) {
    validate_identifier(
        DiagnosticField {
            value: &binding.evidence_kind,
            field_name: "binding.evidence_kind",
        },
        diagnostics,
    );
    validate_identifier(
        DiagnosticField {
            value: &binding.profile_version,
            field_name: "binding.profile_version",
        },
        diagnostics,
    );
    if !evidence_kind_is_supported(&binding.evidence_kind) {
        diagnostics.push(format!("binding.evidence_kind is unsupported: {}", binding.evidence_kind));
        return;
    }
    let expected_claim_scope = expected_claim_scope(&binding.evidence_kind);
    push_literal_diagnostic(
        LiteralDiagnosticField {
            actual: &binding.claim_scope,
            expected: expected_claim_scope,
            field_name: "binding.claim_scope",
        },
        diagnostics,
    );
    if binding.evidence_kind == OPAQUE_EVIDENCE_KIND_FUNCTION_ADDRESS {
        validate_function_address_profile(binding, diagnostics);
    }
    debug_assert!(evidence_kind_is_supported(&binding.evidence_kind));
    debug_assert!(
        binding.claim_scope == expected_claim_scope
            || diagnostics.iter().any(|diagnostic| diagnostic.contains("binding.claim_scope"))
    );
}

fn validate_function_address_profile(binding: &OpaqueEvidenceSidecarBinding, diagnostics: &mut Vec<String>) {
    push_literal_diagnostic(
        LiteralDiagnosticField {
            actual: &binding.profile_version,
            expected: FUNCTION_ADDRESS_PROFILE_VERSION,
            field_name: "binding.profile_version",
        },
        diagnostics,
    );
    push_literal_diagnostic(
        LiteralDiagnosticField {
            actual: &binding.canonical_envelope.role,
            expected: FUNCTION_ADDRESS_EVIDENCE_ROLE,
            field_name: "binding.canonical_envelope.role",
        },
        diagnostics,
    );
    push_literal_diagnostic(
        LiteralDiagnosticField {
            actual: &binding.canonical_envelope.schema,
            expected: FUNCTION_ADDRESS_EVIDENCE_SCHEMA,
            field_name: "binding.canonical_envelope.schema",
        },
        diagnostics,
    );
    push_literal_diagnostic(
        LiteralDiagnosticField {
            actual: &binding.upstream_validation.role,
            expected: VALENCE_FUNCTION_ADDRESS_RECEIPT_ROLE,
            field_name: "binding.upstream_validation.role",
        },
        diagnostics,
    );
    push_literal_diagnostic(
        LiteralDiagnosticField {
            actual: &binding.upstream_validation.schema,
            expected: VALENCE_FUNCTION_ADDRESS_RECEIPT_SCHEMA,
            field_name: "binding.upstream_validation.schema",
        },
        diagnostics,
    );
    validate_function_address_projections(&binding.compatibility_projections, diagnostics);
    debug_assert!(
        binding.profile_version == FUNCTION_ADDRESS_PROFILE_VERSION
            || diagnostics.iter().any(|diagnostic| diagnostic.contains("binding.profile_version"))
    );
    debug_assert!(
        binding.canonical_envelope.role == FUNCTION_ADDRESS_EVIDENCE_ROLE
            || diagnostics.iter().any(|diagnostic| diagnostic.contains("binding.canonical_envelope.role"))
    );
}

fn validate_function_address_projections(
    projections: &[OpaqueEvidenceCompatibilityProjection],
    diagnostics: &mut Vec<String>,
) {
    for projection in projections {
        push_literal_diagnostic(
            LiteralDiagnosticField {
                actual: &projection.role,
                expected: KAMACITE_FUNCTION_ADDRESS_RECEIPT_ROLE,
                field_name: "binding.compatibility_projections.role",
            },
            diagnostics,
        );
        push_literal_diagnostic(
            LiteralDiagnosticField {
                actual: &projection.schema,
                expected: KAMACITE_FUNCTION_ADDRESS_RECEIPT_SCHEMA,
                field_name: "binding.compatibility_projections.schema",
            },
            diagnostics,
        );
    }
}

fn evidence_kind_is_supported(evidence_kind: &str) -> bool {
    matches!(
        evidence_kind,
        OPAQUE_EVIDENCE_KIND_FUNCTION_ADDRESS
            | OPAQUE_EVIDENCE_KIND_PROOF
            | OPAQUE_EVIDENCE_KIND_LINT
            | OPAQUE_EVIDENCE_KIND_DEPENDENCY
            | OPAQUE_EVIDENCE_KIND_BUILD
            | OPAQUE_EVIDENCE_KIND_ATTESTATION
            | OPAQUE_EVIDENCE_KIND_LIFECYCLE
    )
}

fn expected_claim_scope(evidence_kind: &str) -> &'static str {
    if evidence_kind == OPAQUE_EVIDENCE_KIND_FUNCTION_ADDRESS {
        return FUNCTION_ADDRESS_CLAIM_SCOPE;
    }
    OPAQUE_EVIDENCE_GENERIC_CLAIM_SCOPE
}

fn validate_canonical_envelope_link(link: &OpaqueEvidenceCanonicalEnvelopeLink, diagnostics: &mut Vec<String>) {
    validate_external_artifact_fields(
        ExternalArtifactFields {
            role: &link.role,
            schema: &link.schema,
            relative_path: &link.relative_path,
            digest_blake3: &link.digest_blake3,
            field_name: "binding.canonical_envelope",
        },
        diagnostics,
    );
}

fn validate_upstream_validation_link(link: &OpaqueEvidenceUpstreamValidationLink, diagnostics: &mut Vec<String>) {
    validate_external_artifact_fields(
        ExternalArtifactFields {
            role: &link.role,
            schema: &link.schema,
            relative_path: &link.relative_path,
            digest_blake3: &link.digest_blake3,
            field_name: "binding.upstream_validation",
        },
        diagnostics,
    );
}

fn validate_source_artifact_link(link: &OpaqueEvidenceSourceArtifactLink, diagnostics: &mut Vec<String>) {
    push_relative_path_diagnostic(
        DiagnosticField {
            value: &link.relative_path,
            field_name: "binding.source_artifact.relative_path",
        },
        diagnostics,
    );
    push_blake3_diagnostic(
        DiagnosticField {
            value: &link.digest_blake3,
            field_name: "binding.source_artifact.digest_blake3",
        },
        diagnostics,
    );
}

fn validate_release_binary_link(link: &OpaqueEvidenceReleaseBinaryLink, diagnostics: &mut Vec<String>) {
    push_relative_path_diagnostic(
        DiagnosticField {
            value: &link.relative_path,
            field_name: "binding.release_binary.relative_path",
        },
        diagnostics,
    );
    push_blake3_diagnostic(
        DiagnosticField {
            value: &link.digest_blake3,
            field_name: "binding.release_binary.digest_blake3",
        },
        diagnostics,
    );
}

fn validate_external_artifact_fields(fields: ExternalArtifactFields<'_>, diagnostics: &mut Vec<String>) {
    validate_identifier(
        DiagnosticField {
            value: fields.role,
            field_name: &format!("{}.role", fields.field_name),
        },
        diagnostics,
    );
    validate_identifier(
        DiagnosticField {
            value: fields.schema,
            field_name: &format!("{}.schema", fields.field_name),
        },
        diagnostics,
    );
    push_relative_path_diagnostic(
        DiagnosticField {
            value: fields.relative_path,
            field_name: &format!("{}.relative_path", fields.field_name),
        },
        diagnostics,
    );
    push_blake3_diagnostic(
        DiagnosticField {
            value: fields.digest_blake3,
            field_name: &format!("{}.digest_blake3", fields.field_name),
        },
        diagnostics,
    );
    debug_assert!(
        !fields.role.trim().is_empty()
            || diagnostics.iter().any(|diagnostic| diagnostic.contains(".role must not be empty"))
    );
    debug_assert!(
        validate_blake3_hex(fields.digest_blake3, "external artifact digest").is_ok()
            || diagnostics.iter().any(|diagnostic| diagnostic.contains(".digest_blake3"))
    );
}

fn validate_policy_hashes(policy_hashes: &[OpaqueEvidencePolicyHash], diagnostics: &mut Vec<String>) {
    debug_assert_eq!(u32::try_from(REQUIRED_POLICY_KINDS.len()).ok(), Some(REQUIRED_POLICY_KINDS_COUNT));
    if !validate_collection_count(
        CollectionCountBounds {
            actual_count: policy_hashes.len(),
            minimum_count: REQUIRED_POLICY_KINDS_COUNT,
            maximum_count: MAX_OPAQUE_EVIDENCE_POLICY_HASHES_COUNT,
            field_name: "binding.policy_hashes",
        },
        diagnostics,
    ) {
        return;
    }
    let mut seen_kinds = BTreeSet::new();
    for policy_hash in policy_hashes {
        validate_identifier(
            DiagnosticField {
                value: &policy_hash.policy_kind,
                field_name: "binding.policy_hashes.policy_kind",
            },
            diagnostics,
        );
        push_blake3_diagnostic(
            DiagnosticField {
                value: &policy_hash.digest_blake3,
                field_name: "binding.policy_hashes.digest_blake3",
            },
            diagnostics,
        );
        if !seen_kinds.insert(policy_hash.policy_kind.clone()) {
            diagnostics.push(format!("binding.policy_hashes duplicates policy kind {}", policy_hash.policy_kind));
        }
    }
    for required_kind in REQUIRED_POLICY_KINDS {
        if !seen_kinds.contains(*required_kind) {
            diagnostics.push(format!("binding.policy_hashes is missing required policy kind {required_kind}"));
        }
    }
}

fn validate_compatibility_projections(binding: &OpaqueEvidenceSidecarBinding, diagnostics: &mut Vec<String>) {
    if !validate_collection_count(
        CollectionCountBounds {
            actual_count: binding.compatibility_projections.len(),
            minimum_count: 0,
            maximum_count: MAX_OPAQUE_EVIDENCE_PROJECTIONS_COUNT,
            field_name: "binding.compatibility_projections",
        },
        diagnostics,
    ) {
        return;
    }
    let mut seen_roles = BTreeSet::new();
    for projection in &binding.compatibility_projections {
        validate_external_artifact_fields(
            ExternalArtifactFields {
                role: &projection.role,
                schema: &projection.schema,
                relative_path: &projection.relative_path,
                digest_blake3: &projection.digest_blake3,
                field_name: "binding.compatibility_projections",
            },
            diagnostics,
        );
        push_blake3_diagnostic(
            DiagnosticField {
                value: &projection.canonical_envelope_digest_blake3,
                field_name: "binding.compatibility_projections.canonical_envelope_digest_blake3",
            },
            diagnostics,
        );
        if projection.canonical_envelope_digest_blake3 != binding.canonical_envelope.digest_blake3 {
            diagnostics.push(
                "binding.compatibility_projections canonical envelope digest drifted from the authoritative envelope"
                    .to_string(),
            );
        }
        if !seen_roles.insert(projection.role.clone()) {
            diagnostics.push(format!("binding.compatibility_projections duplicates role {}", projection.role));
        }
    }
    debug_assert!(
        u32_count(binding.compatibility_projections.len(), "projection count overflowed u32")
            .is_ok_and(|count| count <= MAX_OPAQUE_EVIDENCE_PROJECTIONS_COUNT)
    );
    debug_assert!(
        binding
            .compatibility_projections
            .iter()
            .all(|projection| projection.canonical_envelope_digest_blake3 == binding.canonical_envelope.digest_blake3)
            || diagnostics.iter().any(|diagnostic| diagnostic.contains("digest drifted"))
    );
}

fn validate_binding_non_claims(binding: &OpaqueEvidenceSidecarBinding, diagnostics: &mut Vec<String>) {
    if !validate_collection_count(
        CollectionCountBounds {
            actual_count: binding.non_claims.len(),
            minimum_count: MIN_OPAQUE_EVIDENCE_NON_CLAIMS_COUNT,
            maximum_count: MAX_OPAQUE_EVIDENCE_NON_CLAIMS_COUNT,
            field_name: "binding.non_claims",
        },
        diagnostics,
    ) {
        return;
    }
    let mut seen_non_claims = BTreeSet::new();
    for non_claim in &binding.non_claims {
        validate_identifier(
            DiagnosticField {
                value: non_claim,
                field_name: "binding.non_claims",
            },
            diagnostics,
        );
        if !seen_non_claims.insert(non_claim.clone()) {
            diagnostics.push("binding.non_claims contains a duplicate entry".to_string());
        }
        if opaque_evidence_text_overclaims(non_claim) {
            diagnostics.push("binding.non_claims contains an opaque evidence overclaim".to_string());
        }
    }
    if !seen_non_claims.contains(OPAQUE_EVIDENCE_REQUIRED_NON_CLAIM) {
        diagnostics.push("binding.non_claims is missing the required opaque payload boundary".to_string());
    }
    let is_function_address = binding.evidence_kind == OPAQUE_EVIDENCE_KIND_FUNCTION_ADDRESS;
    let has_function_address_boundary = seen_non_claims.contains(FUNCTION_ADDRESS_OPAQUE_BOUNDARY);
    if let (true, false) = (is_function_address, has_function_address_boundary) {
        diagnostics.push("binding.non_claims is missing the function-address ownership boundary".to_string());
    }
    debug_assert!(u32_count(binding.non_claims.len(), "non-claim count overflowed u32").is_ok_and(|count| {
        (MIN_OPAQUE_EVIDENCE_NON_CLAIMS_COUNT..=MAX_OPAQUE_EVIDENCE_NON_CLAIMS_COUNT).contains(&count)
    }));
    debug_assert!(
        seen_non_claims.contains(OPAQUE_EVIDENCE_REQUIRED_NON_CLAIM)
            || diagnostics.iter().any(|diagnostic| diagnostic.contains("required opaque payload boundary"))
    );
}

fn opaque_evidence_text_overclaims(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    OPAQUE_EVIDENCE_OVERCLAIM_FRAGMENTS
        .iter()
        .chain(FUNCTION_ADDRESS_OVERCLAIM_FRAGMENTS.iter())
        .any(|fragment| lower.contains(fragment))
}

fn validate_identifier(field: DiagnosticField<'_>, diagnostics: &mut Vec<String>) {
    if field.value.trim().is_empty() {
        diagnostics.push(format!("{} must not be empty", field.field_name));
        return;
    }
    let Ok(byte_count) = u32_count(field.value.len(), &format!("{} length overflowed u32", field.field_name)) else {
        diagnostics.push(format!("{} length overflowed u32", field.field_name));
        return;
    };
    if byte_count > MAX_OPAQUE_EVIDENCE_IDENTIFIER_BYTES_COUNT {
        diagnostics.push(format!(
            "{} is {byte_count} bytes, limit is {MAX_OPAQUE_EVIDENCE_IDENTIFIER_BYTES_COUNT}",
            field.field_name
        ));
    }
}

fn validate_collection_count(bounds: CollectionCountBounds<'_>, diagnostics: &mut Vec<String>) -> bool {
    let Ok(actual_count_u32) = u32_count(bounds.actual_count, &format!("{} count overflowed u32", bounds.field_name))
    else {
        diagnostics.push(format!("{} count overflowed u32", bounds.field_name));
        return false;
    };
    if actual_count_u32 < bounds.minimum_count {
        diagnostics.push(format!("{} requires at least {} entries", bounds.field_name, bounds.minimum_count));
        return false;
    }
    if actual_count_u32 > bounds.maximum_count {
        diagnostics
            .push(format!("{} has {actual_count_u32} entries, limit is {}", bounds.field_name, bounds.maximum_count));
        return false;
    }
    true
}

fn release_artifact_link_diagnostics(
    binding: &OpaqueEvidenceSidecarBinding,
    source_archive: &BundledArtifact,
    binaries: &[BundledArtifact],
) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if binding.source_artifact.relative_path != source_archive.relative_path {
        diagnostics
            .push("binding.source_artifact.relative_path does not match the bundled source artifact".to_string());
    }
    if binding.source_artifact.digest_blake3 != source_archive.digest_blake3 {
        diagnostics
            .push("binding.source_artifact.digest_blake3 does not match the bundled source artifact".to_string());
    }
    validate_bound_release_binary(binding, binaries, &mut diagnostics);
    diagnostics
}

fn validate_bound_release_binary(
    binding: &OpaqueEvidenceSidecarBinding,
    binaries: &[BundledArtifact],
    diagnostics: &mut Vec<String>,
) {
    let Some(binary) = binaries.iter().find(|binary| binary.relative_path == binding.release_binary.relative_path)
    else {
        diagnostics.push("binding.release_binary.relative_path does not match a bundled release binary".to_string());
        return;
    };
    if binary.digest_blake3 != binding.release_binary.digest_blake3 {
        diagnostics.push("binding.release_binary.digest_blake3 does not match the bundled release binary".to_string());
    }
}

struct ExternalEvidenceLinkExpectation<'a> {
    role: &'a str,
    schema: &'a str,
    relative_path: &'a str,
    digest_blake3: &'a str,
    claim_scope: &'a str,
    required_non_claims: &'a [String],
    label: &'a str,
}

fn external_evidence_link_diagnostics(
    binding: &OpaqueEvidenceSidecarBinding,
    external_evidence: &[ExternalEvidence],
) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if !validate_collection_count(
        CollectionCountBounds {
            actual_count: external_evidence.len(),
            minimum_count: 0,
            maximum_count: MAX_OPAQUE_EVIDENCE_EXTERNAL_ROWS_COUNT,
            field_name: "external_evidence",
        },
        &mut diagnostics,
    ) {
        return diagnostics;
    }
    validate_external_evidence_link(
        ExternalEvidenceLinkExpectation {
            role: &binding.canonical_envelope.role,
            schema: &binding.canonical_envelope.schema,
            relative_path: &binding.canonical_envelope.relative_path,
            digest_blake3: &binding.canonical_envelope.digest_blake3,
            claim_scope: &binding.claim_scope,
            required_non_claims: &binding.non_claims,
            label: "canonical envelope",
        },
        external_evidence,
        &mut diagnostics,
    );
    validate_external_evidence_link(
        ExternalEvidenceLinkExpectation {
            role: &binding.upstream_validation.role,
            schema: &binding.upstream_validation.schema,
            relative_path: &binding.upstream_validation.relative_path,
            digest_blake3: &binding.upstream_validation.digest_blake3,
            claim_scope: &binding.claim_scope,
            required_non_claims: &binding.non_claims,
            label: "upstream validation",
        },
        external_evidence,
        &mut diagnostics,
    );
    for projection in &binding.compatibility_projections {
        validate_external_evidence_link(
            ExternalEvidenceLinkExpectation {
                role: &projection.role,
                schema: &projection.schema,
                relative_path: &projection.relative_path,
                digest_blake3: &projection.digest_blake3,
                claim_scope: &binding.claim_scope,
                required_non_claims: &binding.non_claims,
                label: "compatibility projection",
            },
            external_evidence,
            &mut diagnostics,
        );
    }
    debug_assert!(
        external_evidence.iter().any(|external| external.role == binding.canonical_envelope.role)
            || diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("canonical envelope external evidence is missing"))
    );
    debug_assert!(
        external_evidence.iter().any(|external| external.role == binding.upstream_validation.role)
            || diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("upstream validation external evidence is missing"))
    );
    diagnostics
}

fn validate_external_evidence_link(
    expected: ExternalEvidenceLinkExpectation<'_>,
    external_evidence: &[ExternalEvidence],
    diagnostics: &mut Vec<String>,
) {
    let Some(external) = external_evidence.iter().find(|external| external.role == expected.role) else {
        diagnostics.push(format!("opaque evidence {} external evidence is missing", expected.label));
        return;
    };
    debug_assert_eq!(external.role, expected.role);
    if external.schema != expected.schema {
        diagnostics.push(format!("opaque evidence {} schema does not match declared metadata", expected.label));
    }
    if external.relative_path != expected.relative_path {
        diagnostics.push(format!("opaque evidence {} path does not match declared metadata", expected.label));
    }
    if external.digest_blake3 != expected.digest_blake3 {
        diagnostics.push(format!("opaque evidence {} digest does not match declared metadata", expected.label));
    }
    if external.claim_scope != expected.claim_scope {
        diagnostics.push(format!("opaque evidence {} claim scope does not match declared metadata", expected.label));
    }
    if !validate_collection_count(
        CollectionCountBounds {
            actual_count: external.non_claims.len(),
            minimum_count: MIN_OPAQUE_EVIDENCE_NON_CLAIMS_COUNT,
            maximum_count: MAX_OPAQUE_EVIDENCE_NON_CLAIMS_COUNT,
            field_name: "external_evidence.non_claims",
        },
        diagnostics,
    ) {
        return;
    }
    for required_non_claim in expected.required_non_claims {
        if !external.non_claims.contains(required_non_claim) {
            diagnostics.push(format!("opaque evidence {} non-claims are weaker than the binding", expected.label));
        }
    }
    debug_assert!(
        expected.required_non_claims.iter().all(|required| external.non_claims.contains(required))
            || diagnostics.iter().any(|diagnostic| diagnostic.contains("non-claims are weaker"))
    );
}

fn push_literal_diagnostic(field: LiteralDiagnosticField<'_>, diagnostics: &mut Vec<String>) {
    if field.actual != field.expected {
        diagnostics.push(format!("{} must be {}, got {}", field.field_name, field.expected, field.actual));
    }
}

fn push_relative_path_diagnostic(field: DiagnosticField<'_>, diagnostics: &mut Vec<String>) {
    if let Err(error) = validate_relative_member_path(field.value, field.field_name) {
        diagnostics.push(error.to_string());
    }
}

fn push_blake3_diagnostic(field: DiagnosticField<'_>, diagnostics: &mut Vec<String>) {
    if let Err(error) = validate_blake3_hex(field.value, field.field_name) {
        diagnostics.push(error.to_string());
    }
}
