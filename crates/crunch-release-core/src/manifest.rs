use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::ReleaseEvidenceError;
use crate::opaque_evidence::FUNCTION_ADDRESS_CLAIM_SCOPE;
use crate::opaque_evidence::FUNCTION_ADDRESS_DISPOSITION_ABSENT;
use crate::opaque_evidence::FUNCTION_ADDRESS_DISPOSITION_INVALID;
use crate::opaque_evidence::FUNCTION_ADDRESS_DISPOSITION_PRESENT;
use crate::opaque_evidence::FUNCTION_ADDRESS_EVIDENCE_ROLE;
use crate::opaque_evidence::FUNCTION_ADDRESS_EVIDENCE_SCHEMA;
use crate::opaque_evidence::FUNCTION_ADDRESS_MODE_OPTIONAL;
use crate::opaque_evidence::FUNCTION_ADDRESS_MODE_REQUIRED;
use crate::opaque_evidence::FUNCTION_ADDRESS_OPAQUE_BOUNDARY;
use crate::opaque_evidence::FUNCTION_ADDRESS_PRESERVES_PROFILE_VERSION;
use crate::opaque_evidence::FUNCTION_ADDRESS_PROFILE_VERSION;
use crate::opaque_evidence::KAMACITE_FUNCTION_ADDRESS_JSON_PROJECTION_ROLE;
#[cfg(test)]
use crate::opaque_evidence::KAMACITE_FUNCTION_ADDRESS_JSON_PROJECTION_SCHEMA;
#[cfg(test)]
use crate::opaque_evidence::KAMACITE_FUNCTION_ADDRESS_PRESERVES_ROLE;
#[cfg(test)]
use crate::opaque_evidence::KAMACITE_FUNCTION_ADDRESS_PRESERVES_SCHEMA;
use crate::opaque_evidence::KAMACITE_FUNCTION_ADDRESS_RECEIPT_ROLE;
use crate::opaque_evidence::KAMACITE_FUNCTION_ADDRESS_RECEIPT_SCHEMA;
use crate::opaque_evidence::MAX_OPAQUE_EVIDENCE_BINDINGS_COUNT;
#[cfg(test)]
use crate::opaque_evidence::OPAQUE_EVIDENCE_GENERIC_CLAIM_SCOPE;
use crate::opaque_evidence::OPAQUE_EVIDENCE_KIND_FUNCTION_ADDRESS;
#[cfg(test)]
use crate::opaque_evidence::OPAQUE_EVIDENCE_KIND_LIFECYCLE;
#[cfg(test)]
use crate::opaque_evidence::OPAQUE_EVIDENCE_POLICY_KIND_MANTLE_RELEASE;
#[cfg(test)]
use crate::opaque_evidence::OPAQUE_EVIDENCE_POLICY_KIND_UPSTREAM_PROFILE;
#[cfg(test)]
use crate::opaque_evidence::OPAQUE_EVIDENCE_REQUIRED_NON_CLAIM;
#[cfg(test)]
use crate::opaque_evidence::OPAQUE_EVIDENCE_SIDECAR_BINDING_SCHEMA;
#[cfg(test)]
use crate::opaque_evidence::OpaqueEvidenceCanonicalEnvelopeLink;
#[cfg(test)]
use crate::opaque_evidence::OpaqueEvidenceCompatibilityProjection;
#[cfg(test)]
use crate::opaque_evidence::OpaqueEvidencePolicyHash;
#[cfg(test)]
use crate::opaque_evidence::OpaqueEvidenceReleaseBinaryLink;
#[cfg(test)]
use crate::opaque_evidence::OpaqueEvidenceSidecarBinding;
use crate::opaque_evidence::OpaqueEvidenceSidecarBindingReceipt;
#[cfg(test)]
use crate::opaque_evidence::OpaqueEvidenceSourceArtifactLink;
#[cfg(test)]
use crate::opaque_evidence::OpaqueEvidenceUpstreamValidationLink;
use crate::opaque_evidence::VALENCE_FUNCTION_ADDRESS_RECEIPT_ROLE;
use crate::opaque_evidence::VALENCE_FUNCTION_ADDRESS_RECEIPT_SCHEMA;
use crate::opaque_evidence::opaque_evidence_sidecar_binding_diagnostics;
#[cfg(test)]
use crate::opaque_evidence::opaque_evidence_sidecar_binding_receipt;
use crate::source_archive::RELEASE_SOURCE_ARCHIVE_PROFILE;
use crate::source_archive::RELEASE_SOURCE_ARCHIVE_VERSION;

pub const RELEASE_EVIDENCE_SCHEMA: &str = "mantle-release-evidence-v1";
pub const FULL_SELF_HOSTING_PROOF_SCHEMA: &str = "mantle-self-hosting-proof-v2";
pub const CLAIM_SCOPE_PACKAGED_INTEGRITY: &str = "packaged-integrity-evidence";
pub const DEFAULT_PROOF_WORKFLOW_COMMAND: &str = "./scripts/prove-self-hosting.sh";
pub const DEFAULT_PROOF_WORKFLOW_VERSION: &str = "mantle-self-hosting-proof-v2";
pub const PROVIDER_FIXED_POINT_PROOF_EVIDENCE_ROLE: &str = "cargo-free-source-built-handoff-evidence";
pub const DETERMINISTIC_BUILD_PROOF_EVIDENCE_ROLE: &str = "deterministic-build-proof-receipt";
pub const DETERMINISTIC_SANDBOX_ISOLATION_EVIDENCE_ROLE: &str = "deterministic-sandbox-isolation-evidence";
pub const KANI_TOOLCHAIN_EVIDENCE_SCHEMA: &str = "mantle-kani-toolchain-evidence-v1";
pub const KANI_RECEIPT_EVIDENCE_ROLE: &str = "kani-model-check-receipt";
pub const KANI_EVIDENCE_CLAIM_SCOPE: &str = "kani-release-identity-linkage-only";
pub const KANI_VALENCE_SEMANTIC_ROLE: &str = "valence-validated-kani-external-evidence";
pub const KANI_NON_CLAIM_WHOLE_PROGRAM: &str = "kani.boundary.whole_program";
pub const KANI_NON_CLAIM_VERIFIER_SOUNDNESS: &str = "kani.boundary.verifier_soundness";
pub const KANI_NON_CLAIM_SEMANTICS: &str = "kani.boundary.semantics";
pub const KANI_NON_CLAIM_RELEASE_ELIGIBILITY: &str = "mantle.boundary.release_eligibility";
pub const KANI_SOLVER_KIND_CBMC_DEFAULT: &str = "cbmc-default";
pub const KANI_SOLVER_KIND_MINISAT: &str = "minisat";
pub const KANI_SOLVER_KIND_CADICAL: &str = "cadical";
pub const KANI_SOLVER_KIND_KISSAT: &str = "kissat";
pub const STACK_PROVENANCE_EVIDENCE_ROLE: &str = "stack-provenance-trace";
pub const STACK_PROVENANCE_SIDECAR_SCHEMA: &str = "valence.stack-provenance-sidecar.v1";
pub const STACK_PROVENANCE_GRAPH_REPORT_SCHEMA: &str = "valence.stack-provenance-graph-report.v1";
pub const VALENCE_STACK_PROVENANCE_RECEIPT_ROLE: &str = "valence-stack-provenance-graph-report";
pub const STACK_PROVENANCE_CLAIM_SCOPE: &str = "identity-linkage-sidecar";
pub const STACK_PROVENANCE_MODE_OPTIONAL: &str = "optional";
pub const STACK_PROVENANCE_MODE_REQUIRED: &str = "required";
pub const RELEASE_PROFILE_GENERIC: &str = "generic";
pub const RELEASE_PROFILE_ONIX_STACK: &str = "onix-stack";
pub const STACK_PROVENANCE_DISPOSITION_ABSENT: &str = "absent";
pub const STACK_PROVENANCE_DISPOSITION_PRESENT: &str = "present";
pub const STACK_PROVENANCE_DISPOSITION_INVALID: &str = "invalid";
pub const STACK_PROVENANCE_OPAQUE_BOUNDARY: &str = "Mantle validates bundle-local stack provenance path, digest, role, schema, claim scope, binary identity, and non-claims only; Valence owns stack semantics";
pub const SOURCE_ACQUISITION_KIND_EXTERNAL_ARCHIVE: &str = "external-archive";
pub const SOURCE_ACQUISITION_KIND_GIT: &str = "git";
pub const BLAKE3_HEX_LENGTH_CHARS: usize = 64;

const MAX_BINARY_ARTIFACTS_COUNT: u32 = 16;
const MAX_RELATIVE_PATH_BYTES_COUNT: u32 = 4096;
const MAX_SOURCE_ACQUISITION_URL_BYTES_COUNT: u32 = 8192;
const MAX_SOURCE_ACQUISITION_REF_BYTES_COUNT: u32 = 512;
const GIT_SHA1_HEX_LENGTH_CHARS: usize = 40;
const GIT_SHA256_HEX_LENGTH_CHARS: usize = 64;
const KANI_REQUIRED_NON_CLAIMS: &[&str] = &[
    KANI_NON_CLAIM_WHOLE_PROGRAM,
    KANI_NON_CLAIM_VERIFIER_SOUNDNESS,
    KANI_NON_CLAIM_SEMANTICS,
    KANI_NON_CLAIM_RELEASE_ELIGIBILITY,
];
const KANI_SUPPORTED_SOLVER_KINDS: &[&str] = &[
    KANI_SOLVER_KIND_CBMC_DEFAULT,
    KANI_SOLVER_KIND_MINISAT,
    KANI_SOLVER_KIND_CADICAL,
    KANI_SOLVER_KIND_KISSAT,
];
const STACK_PROVENANCE_OVERCLAIM_FRAGMENTS: &[&str] = &[
    "mantle semantically verified",
    "mantle verifies octet",
    "mantle verifies trellis",
    "mantle verifies valence stack semantics",
    "mantle verifies cairn lifecycle semantics",
    "proves source-code correctness",
    "proves release eligibility",
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BundledArtifactKind {
    File,
    Directory,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundledArtifact {
    pub kind: BundledArtifactKind,
    pub relative_path: String,
    pub size_bytes: u64,
    pub digest_blake3: String,
}

pub fn validate_bundled_artifact_record(
    artifact: BundledArtifact,
    field_name: String,
) -> Result<BundledArtifact, ReleaseEvidenceError> {
    validate_bundled_artifact(&artifact, &field_name)?;
    Ok(artifact)
}

fn validate_bundled_artifact(artifact: &BundledArtifact, field_name: &str) -> Result<(), ReleaseEvidenceError> {
    validate_relative_member_path(&artifact.relative_path, field_name)?;
    if artifact.size_bytes == 0 {
        return Err(validation_error(format!("release evidence {field_name}.size_bytes must be non-zero")));
    }
    validate_blake3_hex(&artifact.digest_blake3, &format!("{field_name}.digest_blake3"))?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseWorkflowIdentity {
    pub command: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderFixedPointProofArtifact {
    pub kind: BundledArtifactKind,
    pub relative_path: String,
    pub size_bytes: u64,
    pub digest_blake3: String,
    pub evidence_role: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderFixedPointReleaseArtifactBinding {
    pub relative_path: String,
    pub digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleBoundedReleaseArtifact {
    pub kind: BundledArtifactKind,
    pub relative_path: String,
    pub size_bytes: u64,
    pub digest_blake3: String,
    pub evidence_role: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalEvidence {
    pub role: String,
    pub schema: String,
    pub relative_path: String,
    pub digest_blake3: String,
    pub claim_scope: String,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KaniSolverIdentity {
    pub kind: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KaniToolchainEvidence {
    pub schema: String,
    pub receipt_role: String,
    pub receipt_relative_path: String,
    pub receipt_digest_blake3: String,
    pub kani_version: String,
    pub rust_toolchain: String,
    pub cbmc_version: String,
    pub solver: KaniSolverIdentity,
    pub invocation_wrapper: String,
    pub closure_identity_blake3: String,
    pub expected_closure_identity_blake3: String,
    pub valence_semantic_role: String,
    pub claim_scope: String,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StackProvenanceReleaseEvidence {
    pub sidecar_role: String,
    pub sidecar_schema: String,
    pub sidecar_claim_scope: String,
    pub sidecar_relative_path: String,
    pub sidecar_digest_blake3: String,
    pub valence_receipt_role: String,
    pub valence_receipt_schema: String,
    pub valence_receipt_relative_path: String,
    pub valence_receipt_digest_blake3: String,
    pub release_binary_relative_path: String,
    pub release_binary_digest_blake3: String,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StackProvenanceReleaseVerification {
    pub mode: String,
    pub required: bool,
    pub valid: bool,
    pub disposition: String,
    pub sidecar_role: Option<String>,
    pub sidecar_schema: Option<String>,
    pub sidecar_claim_scope: Option<String>,
    pub sidecar_digest_blake3: Option<String>,
    pub valence_receipt_role: Option<String>,
    pub valence_receipt_schema: Option<String>,
    pub valence_receipt_digest_blake3: Option<String>,
    pub release_binary_relative_path: Option<String>,
    pub release_binary_digest_blake3: Option<String>,
    pub boundary: String,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionAddressReleaseEvidence {
    pub sidecar_role: String,
    pub sidecar_schema: String,
    pub sidecar_claim_scope: String,
    pub sidecar_relative_path: String,
    pub sidecar_digest_blake3: String,
    pub valence_receipt_role: String,
    pub valence_receipt_schema: String,
    pub valence_receipt_relative_path: String,
    pub valence_receipt_digest_blake3: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kamacite_receipt_role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kamacite_receipt_schema: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kamacite_receipt_relative_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kamacite_receipt_digest_blake3: Option<String>,
    pub source_archive_digest_blake3: String,
    pub release_binary_relative_path: String,
    pub release_binary_digest_blake3: String,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionAddressReleaseVerification {
    pub mode: String,
    pub required: bool,
    pub valid: bool,
    pub disposition: String,
    pub sidecar_role: Option<String>,
    pub sidecar_schema: Option<String>,
    pub sidecar_claim_scope: Option<String>,
    pub sidecar_digest_blake3: Option<String>,
    pub valence_receipt_role: Option<String>,
    pub valence_receipt_schema: Option<String>,
    pub valence_receipt_digest_blake3: Option<String>,
    pub valence_receipt_hash_blake3: Option<String>,
    pub kamacite_receipt_role: Option<String>,
    pub kamacite_receipt_schema: Option<String>,
    pub kamacite_receipt_digest_blake3: Option<String>,
    pub kamacite_receipt_hash_blake3: Option<String>,
    pub source_archive_digest_blake3: Option<String>,
    pub release_binary_relative_path: Option<String>,
    pub release_binary_digest_blake3: Option<String>,
    pub boundary: String,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceAcquisition {
    pub kind: String,
    pub url: String,
    pub digest_blake3: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archive_profile: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archive_version: Option<String>,
}

impl SourceAcquisition {
    pub fn external_archive(url: String, digest_blake3: String) -> Self {
        Self {
            kind: SOURCE_ACQUISITION_KIND_EXTERNAL_ARCHIVE.to_string(),
            url,
            digest_blake3,
            commit: None,
            reference: None,
            tag: None,
            archive_profile: None,
            archive_version: None,
        }
    }

    pub fn git(
        url: String,
        commit: String,
        reference: Option<String>,
        tag: Option<String>,
        digest_blake3: String,
    ) -> Self {
        Self {
            kind: SOURCE_ACQUISITION_KIND_GIT.to_string(),
            url,
            digest_blake3,
            commit: Some(commit),
            reference,
            tag,
            archive_profile: Some(RELEASE_SOURCE_ARCHIVE_PROFILE.to_string()),
            archive_version: Some(RELEASE_SOURCE_ARCHIVE_VERSION.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseProofLinkage {
    pub release_id: String,
    pub source_archive_digest_blake3: String,
    pub proof_bundle_schema: String,
    pub proof_mode: String,
    pub selected_provider_kind: String,
    pub staged_source: String,
    pub stage2_binary_digest_blake3: String,
    pub prerequisite_inventory_digest_blake3: String,
    pub proof_manifest_digest_blake3: String,
}

pub const PROVENANCE_COVERAGE_BOUNDARY: &str = "provenance coverage records identity and linkage only; it does not prove behavioral correctness, semantic equivalence, or that the binary satisfies the requirements";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvenanceCoverage {
    pub binary_hash: String,
    pub covered_source_ids: Vec<String>,
    pub covered_function_object_ids: Vec<String>,
    pub covered_requirement_ids: Vec<String>,
    pub coverage_boundary: String,
}

fn empty_opaque_evidence_sidecar_bindings() -> Vec<OpaqueEvidenceSidecarBindingReceipt> {
    Vec::new()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseEvidenceManifest {
    pub schema: String,
    pub release_id: String,
    pub claim_scope: String,
    pub workflow: ReleaseWorkflowIdentity,
    pub source_archive: BundledArtifact,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_acquisition: Option<SourceAcquisition>,
    pub binaries: Vec<BundledArtifact>,
    pub proof_bundle: BundledArtifact,
    pub prerequisite_inventory: BundledArtifact,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_fixed_point_proof: Option<ProviderFixedPointProofArtifact>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reproducibility_report: Option<BundledArtifact>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deterministic_build_proof: Option<RoleBoundedReleaseArtifact>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deterministic_sandbox_isolation_evidence: Option<RoleBoundedReleaseArtifact>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub independent_agreement_report: Option<BundledArtifact>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub external_evidence: Vec<ExternalEvidence>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub kani_toolchain_evidence: Vec<KaniToolchainEvidence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stack_provenance: Option<StackProvenanceReleaseEvidence>,
    #[serde(
        default = "empty_opaque_evidence_sidecar_bindings",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub opaque_evidence_sidecar_bindings: Vec<OpaqueEvidenceSidecarBindingReceipt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function_address_evidence: Option<FunctionAddressReleaseEvidence>,
    pub proof_linkage: ReleaseProofLinkage,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance_coverage: Option<ProvenanceCoverage>,
}

pub fn canonical_release_evidence_manifest(manifest: ReleaseEvidenceManifest) -> Result<Vec<u8>, ReleaseEvidenceError> {
    validate_release_evidence_manifest(&manifest)?;
    serde_json::to_vec(&manifest).map_err(|err| parse_error(format!("serializing release evidence manifest: {err}")))
}

pub fn validate_provider_fixed_point_release_artifact_binding(
    binaries: &[BundledArtifact],
    provider_stage_binary_digest_blake3: &str,
) -> Result<ProviderFixedPointReleaseArtifactBinding, ReleaseEvidenceError> {
    validate_blake3_hex(provider_stage_binary_digest_blake3, "provider_fixed_point.stage_binary_digest_blake3")?;
    validate_provider_binding_binary_count(binaries)?;
    for (index_usize, artifact) in binaries.iter().enumerate() {
        let index_u32 = u32_count(index_usize, "release evidence binary index overflowed u32")?;
        validate_bundled_artifact(artifact, &format!("binaries[{index_u32}]"))?;
        if artifact.digest_blake3 == provider_stage_binary_digest_blake3 {
            return Ok(ProviderFixedPointReleaseArtifactBinding {
                relative_path: artifact.relative_path.clone(),
                digest_blake3: artifact.digest_blake3.clone(),
            });
        }
    }
    Err(validation_error(
        "provider fixed-point proof stage binary digest does not match any bundled release binary artifact".to_string(),
    ))
}

fn validate_provider_binding_binary_count(binaries: &[BundledArtifact]) -> Result<(), ReleaseEvidenceError> {
    let binary_count = u32_count(binaries.len(), "release evidence binary artifact count overflowed u32")?;
    if binary_count == 0 {
        return Err(validation_error(
            "provider fixed-point release artifact binding requires at least one binary artifact".to_string(),
        ));
    }
    if binary_count > MAX_BINARY_ARTIFACTS_COUNT {
        return Err(validation_error(format!(
            "release evidence records {binary_count} binary artifacts, limit is {MAX_BINARY_ARTIFACTS_COUNT}"
        )));
    }
    Ok(())
}

fn validate_release_evidence_manifest(manifest: &ReleaseEvidenceManifest) -> Result<(), ReleaseEvidenceError> {
    validate_manifest_header(manifest)?;
    validate_manifest_artifacts(manifest)?;
    validate_source_acquisition(manifest)?;
    validate_manifest_linkage(manifest)?;
    validate_provenance_coverage(manifest)?;
    validate_external_evidence(manifest)?;
    validate_kani_toolchain_evidence(manifest)?;
    validate_stack_provenance_manifest_evidence(manifest)?;
    validate_opaque_evidence_sidecar_bindings(manifest)?;
    validate_function_address_manifest_evidence(manifest)?;
    Ok(())
}

fn validate_opaque_evidence_sidecar_bindings(manifest: &ReleaseEvidenceManifest) -> Result<(), ReleaseEvidenceError> {
    let binding_count = u32_count(
        manifest.opaque_evidence_sidecar_bindings.len(),
        "opaque evidence sidecar binding count overflowed u32",
    )?;
    if binding_count > MAX_OPAQUE_EVIDENCE_BINDINGS_COUNT {
        return Err(validation_error(format!(
            "release evidence records {binding_count} opaque evidence sidecar bindings, limit is {MAX_OPAQUE_EVIDENCE_BINDINGS_COUNT}"
        )));
    }
    let mut is_function_address_binding_seen = false;
    for (index_usize, receipt) in manifest.opaque_evidence_sidecar_bindings.iter().enumerate() {
        let index_u32 = u32_count(index_usize, "opaque evidence sidecar binding index overflowed u32")?;
        if receipt.binding.evidence_kind == OPAQUE_EVIDENCE_KIND_FUNCTION_ADDRESS {
            if is_function_address_binding_seen {
                return Err(validation_error(format!(
                    "release evidence opaque_evidence_sidecar_bindings[{index_u32}] duplicates function-address evidence"
                )));
            }
            is_function_address_binding_seen = true;
        }
        let diagnostics = opaque_evidence_sidecar_binding_diagnostics(
            receipt,
            &manifest.source_archive,
            &manifest.binaries,
            &manifest.external_evidence,
        );
        if let Some(first) = diagnostics.first() {
            return Err(validation_error(format!(
                "release evidence opaque_evidence_sidecar_bindings[{index_u32}] invalid: {first}"
            )));
        }
    }
    if manifest.function_address_evidence.is_some() && is_function_address_binding_seen {
        return Err(validation_error(
            "release evidence cannot contain both generic and legacy function-address bindings".to_string(),
        ));
    }
    debug_assert!(binding_count <= MAX_OPAQUE_EVIDENCE_BINDINGS_COUNT);
    debug_assert!(!is_function_address_binding_seen || manifest.function_address_evidence.is_none());
    Ok(())
}

pub fn stack_provenance_mode_for_release_profile(
    release_profile: &str,
    requested_mode: &str,
) -> Result<&'static str, ReleaseEvidenceError> {
    if !stack_provenance_mode_is_supported(requested_mode) {
        return Err(validation_error(format!("unsupported stack provenance mode: {requested_mode}")));
    }
    match release_profile {
        RELEASE_PROFILE_GENERIC => match requested_mode {
            STACK_PROVENANCE_MODE_OPTIONAL => Ok(STACK_PROVENANCE_MODE_OPTIONAL),
            STACK_PROVENANCE_MODE_REQUIRED => Ok(STACK_PROVENANCE_MODE_REQUIRED),
            _ => unreachable!("requested stack provenance mode was already validated"),
        },
        RELEASE_PROFILE_ONIX_STACK => Ok(STACK_PROVENANCE_MODE_REQUIRED),
        _ => Err(validation_error(format!("unsupported release profile: {release_profile}"))),
    }
}

pub fn evaluate_stack_provenance_release_evidence(
    manifest: &ReleaseEvidenceManifest,
    mode: &str,
) -> StackProvenanceReleaseVerification {
    let mut diagnostics = Vec::new();
    if !stack_provenance_mode_is_supported(mode) {
        diagnostics.push(format!("unsupported stack provenance mode: {mode}"));
    }
    let required = mode == STACK_PROVENANCE_MODE_REQUIRED;
    let Some(evidence) = &manifest.stack_provenance else {
        if required {
            diagnostics.push("required Valence stack provenance sidecar or receipt is missing".to_string());
        }
        return StackProvenanceReleaseVerification {
            mode: mode.to_string(),
            required,
            valid: diagnostics.is_empty(),
            disposition: STACK_PROVENANCE_DISPOSITION_ABSENT.to_string(),
            sidecar_role: None,
            sidecar_schema: None,
            sidecar_claim_scope: None,
            sidecar_digest_blake3: None,
            valence_receipt_role: None,
            valence_receipt_schema: None,
            valence_receipt_digest_blake3: None,
            release_binary_relative_path: None,
            release_binary_digest_blake3: None,
            boundary: STACK_PROVENANCE_OPAQUE_BOUNDARY.to_string(),
            diagnostics,
        };
    };
    diagnostics.extend(stack_provenance_evidence_diagnostics(manifest, evidence));
    let valid = diagnostics.is_empty();
    StackProvenanceReleaseVerification {
        mode: mode.to_string(),
        required,
        valid,
        disposition: if valid {
            STACK_PROVENANCE_DISPOSITION_PRESENT.to_string()
        } else {
            STACK_PROVENANCE_DISPOSITION_INVALID.to_string()
        },
        sidecar_role: Some(evidence.sidecar_role.clone()),
        sidecar_schema: Some(evidence.sidecar_schema.clone()),
        sidecar_claim_scope: Some(evidence.sidecar_claim_scope.clone()),
        sidecar_digest_blake3: Some(evidence.sidecar_digest_blake3.clone()),
        valence_receipt_role: Some(evidence.valence_receipt_role.clone()),
        valence_receipt_schema: Some(evidence.valence_receipt_schema.clone()),
        valence_receipt_digest_blake3: Some(evidence.valence_receipt_digest_blake3.clone()),
        release_binary_relative_path: Some(evidence.release_binary_relative_path.clone()),
        release_binary_digest_blake3: Some(evidence.release_binary_digest_blake3.clone()),
        boundary: STACK_PROVENANCE_OPAQUE_BOUNDARY.to_string(),
        diagnostics,
    }
}

pub fn function_address_evidence_mode_for_release_profile(
    release_profile: &str,
    requested_mode: &str,
) -> Result<&'static str, ReleaseEvidenceError> {
    if !function_address_mode_is_supported(requested_mode) {
        return Err(validation_error(format!("unsupported function-address evidence mode: {requested_mode}")));
    }
    match release_profile {
        RELEASE_PROFILE_GENERIC | RELEASE_PROFILE_ONIX_STACK => match requested_mode {
            FUNCTION_ADDRESS_MODE_OPTIONAL => Ok(FUNCTION_ADDRESS_MODE_OPTIONAL),
            FUNCTION_ADDRESS_MODE_REQUIRED => Ok(FUNCTION_ADDRESS_MODE_REQUIRED),
            _ => unreachable!("requested function-address evidence mode was already validated"),
        },
        _ => Err(validation_error(format!("unsupported release profile: {release_profile}"))),
    }
}

// r[impl mantle.release_provenance.opaque_evidence_sidecar_binding.function_address]
// r[impl mantle.release_provenance.opaque_evidence_sidecar_binding.opaque_core]
pub fn evaluate_function_address_release_evidence(
    manifest: &ReleaseEvidenceManifest,
    mode: &str,
) -> FunctionAddressReleaseVerification {
    debug_assert_ne!(FUNCTION_ADDRESS_MODE_OPTIONAL, FUNCTION_ADDRESS_MODE_REQUIRED);
    debug_assert_ne!(FUNCTION_ADDRESS_DISPOSITION_PRESENT, FUNCTION_ADDRESS_DISPOSITION_INVALID);
    let mut diagnostics = Vec::new();
    if !function_address_mode_is_supported(mode) {
        diagnostics.push(format!("unsupported function-address evidence mode: {mode}"));
    }
    let is_required = mode == FUNCTION_ADDRESS_MODE_REQUIRED;
    let mut generic_bindings = manifest
        .opaque_evidence_sidecar_bindings
        .iter()
        .filter(|receipt| opaque_binding_is_function_address_candidate(receipt));
    if let Some(receipt) = generic_bindings.next() {
        if generic_bindings.next().is_some() {
            diagnostics.push("multiple function-address opaque evidence bindings are present".to_string());
        }
        if manifest.function_address_evidence.is_some() {
            diagnostics.push("generic and legacy function-address evidence cannot both be present".to_string());
        }
        return generic_function_address_verification(manifest, receipt, mode, is_required, diagnostics);
    }
    if let Some(evidence) = &manifest.function_address_evidence {
        return legacy_function_address_verification(manifest, evidence, mode, is_required, diagnostics);
    }
    absent_function_address_verification(mode, is_required, diagnostics)
}

fn opaque_binding_is_function_address_candidate(receipt: &OpaqueEvidenceSidecarBindingReceipt) -> bool {
    let binding = &receipt.binding;
    if binding.evidence_kind == OPAQUE_EVIDENCE_KIND_FUNCTION_ADDRESS {
        return true;
    }
    if binding.profile_version == FUNCTION_ADDRESS_PROFILE_VERSION
        || binding.profile_version == FUNCTION_ADDRESS_PRESERVES_PROFILE_VERSION
    {
        return true;
    }
    binding.canonical_envelope.role == FUNCTION_ADDRESS_EVIDENCE_ROLE
}

fn generic_function_address_verification(
    manifest: &ReleaseEvidenceManifest,
    receipt: &OpaqueEvidenceSidecarBindingReceipt,
    mode: &str,
    is_required: bool,
    mut diagnostics: Vec<String>,
) -> FunctionAddressReleaseVerification {
    debug_assert!(opaque_binding_is_function_address_candidate(receipt));
    debug_assert!(manifest.opaque_evidence_sidecar_bindings.iter().any(|candidate| core::ptr::eq(candidate, receipt)));
    diagnostics.extend(opaque_evidence_sidecar_binding_diagnostics(
        receipt,
        &manifest.source_archive,
        &manifest.binaries,
        &manifest.external_evidence,
    ));
    let binding = &receipt.binding;
    let is_preserves = binding.profile_version == FUNCTION_ADDRESS_PRESERVES_PROFILE_VERSION;
    let projection_role = if is_preserves {
        KAMACITE_FUNCTION_ADDRESS_JSON_PROJECTION_ROLE
    } else {
        KAMACITE_FUNCTION_ADDRESS_RECEIPT_ROLE
    };
    let kamacite = binding.compatibility_projections.iter().find(|projection| projection.role == projection_role);
    present_function_address_verification(mode, is_required, diagnostics, FunctionAddressVerificationLinks {
        sidecar_role: binding.canonical_envelope.role.clone(),
        sidecar_schema: binding.canonical_envelope.schema.clone(),
        sidecar_claim_scope: binding.claim_scope.clone(),
        sidecar_digest_blake3: binding.canonical_envelope.digest_blake3.clone(),
        valence_receipt_role: binding.upstream_validation.role.clone(),
        valence_receipt_schema: binding.upstream_validation.schema.clone(),
        valence_receipt_digest_blake3: binding.upstream_validation.digest_blake3.clone(),
        valence_receipt_hash_blake3: binding.upstream_validation.receipt_hash_blake3.clone(),
        kamacite_receipt_role: kamacite.map(|projection| projection.role.clone()),
        kamacite_receipt_schema: kamacite.map(|projection| projection.schema.clone()),
        kamacite_receipt_digest_blake3: kamacite.map(|projection| projection.digest_blake3.clone()),
        kamacite_receipt_hash_blake3: kamacite.map(|projection| projection.canonical_envelope_digest_blake3.clone()),
        source_archive_digest_blake3: binding.source_artifact.digest_blake3.clone(),
        release_binary_relative_path: binding.release_binary.relative_path.clone(),
        release_binary_digest_blake3: binding.release_binary.digest_blake3.clone(),
    })
}

fn legacy_function_address_verification(
    manifest: &ReleaseEvidenceManifest,
    evidence: &FunctionAddressReleaseEvidence,
    mode: &str,
    is_required: bool,
    mut diagnostics: Vec<String>,
) -> FunctionAddressReleaseVerification {
    diagnostics.extend(function_address_evidence_diagnostics(manifest, evidence));
    present_function_address_verification(mode, is_required, diagnostics, FunctionAddressVerificationLinks {
        sidecar_role: evidence.sidecar_role.clone(),
        sidecar_schema: evidence.sidecar_schema.clone(),
        sidecar_claim_scope: evidence.sidecar_claim_scope.clone(),
        sidecar_digest_blake3: evidence.sidecar_digest_blake3.clone(),
        valence_receipt_role: evidence.valence_receipt_role.clone(),
        valence_receipt_schema: evidence.valence_receipt_schema.clone(),
        valence_receipt_digest_blake3: evidence.valence_receipt_digest_blake3.clone(),
        valence_receipt_hash_blake3: None,
        kamacite_receipt_role: evidence.kamacite_receipt_role.clone(),
        kamacite_receipt_schema: evidence.kamacite_receipt_schema.clone(),
        kamacite_receipt_digest_blake3: evidence.kamacite_receipt_digest_blake3.clone(),
        kamacite_receipt_hash_blake3: None,
        source_archive_digest_blake3: evidence.source_archive_digest_blake3.clone(),
        release_binary_relative_path: evidence.release_binary_relative_path.clone(),
        release_binary_digest_blake3: evidence.release_binary_digest_blake3.clone(),
    })
}

struct FunctionAddressVerificationLinks {
    sidecar_role: String,
    sidecar_schema: String,
    sidecar_claim_scope: String,
    sidecar_digest_blake3: String,
    valence_receipt_role: String,
    valence_receipt_schema: String,
    valence_receipt_digest_blake3: String,
    valence_receipt_hash_blake3: Option<String>,
    kamacite_receipt_role: Option<String>,
    kamacite_receipt_schema: Option<String>,
    kamacite_receipt_digest_blake3: Option<String>,
    kamacite_receipt_hash_blake3: Option<String>,
    source_archive_digest_blake3: String,
    release_binary_relative_path: String,
    release_binary_digest_blake3: String,
}

fn present_function_address_verification(
    mode: &str,
    is_required: bool,
    diagnostics: Vec<String>,
    links: FunctionAddressVerificationLinks,
) -> FunctionAddressReleaseVerification {
    let is_valid = diagnostics.is_empty();
    let verification = FunctionAddressReleaseVerification {
        mode: mode.to_string(),
        required: is_required,
        valid: is_valid,
        disposition: if is_valid {
            FUNCTION_ADDRESS_DISPOSITION_PRESENT.to_string()
        } else {
            FUNCTION_ADDRESS_DISPOSITION_INVALID.to_string()
        },
        sidecar_role: Some(links.sidecar_role),
        sidecar_schema: Some(links.sidecar_schema),
        sidecar_claim_scope: Some(links.sidecar_claim_scope),
        sidecar_digest_blake3: Some(links.sidecar_digest_blake3),
        valence_receipt_role: Some(links.valence_receipt_role),
        valence_receipt_schema: Some(links.valence_receipt_schema),
        valence_receipt_digest_blake3: Some(links.valence_receipt_digest_blake3),
        valence_receipt_hash_blake3: links.valence_receipt_hash_blake3,
        kamacite_receipt_role: links.kamacite_receipt_role,
        kamacite_receipt_schema: links.kamacite_receipt_schema,
        kamacite_receipt_digest_blake3: links.kamacite_receipt_digest_blake3,
        kamacite_receipt_hash_blake3: links.kamacite_receipt_hash_blake3,
        source_archive_digest_blake3: Some(links.source_archive_digest_blake3),
        release_binary_relative_path: Some(links.release_binary_relative_path),
        release_binary_digest_blake3: Some(links.release_binary_digest_blake3),
        boundary: FUNCTION_ADDRESS_OPAQUE_BOUNDARY.to_string(),
        diagnostics,
    };
    debug_assert_eq!(verification.valid, verification.diagnostics.is_empty());
    debug_assert_eq!(verification.disposition == FUNCTION_ADDRESS_DISPOSITION_PRESENT, verification.valid);
    verification
}

fn absent_function_address_verification(
    mode: &str,
    is_required: bool,
    mut diagnostics: Vec<String>,
) -> FunctionAddressReleaseVerification {
    if is_required {
        diagnostics.push("required function-address release evidence is missing".to_string());
    }
    let verification = FunctionAddressReleaseVerification {
        mode: mode.to_string(),
        required: is_required,
        valid: diagnostics.is_empty(),
        disposition: FUNCTION_ADDRESS_DISPOSITION_ABSENT.to_string(),
        sidecar_role: None,
        sidecar_schema: None,
        sidecar_claim_scope: None,
        sidecar_digest_blake3: None,
        valence_receipt_role: None,
        valence_receipt_schema: None,
        valence_receipt_digest_blake3: None,
        valence_receipt_hash_blake3: None,
        kamacite_receipt_role: None,
        kamacite_receipt_schema: None,
        kamacite_receipt_digest_blake3: None,
        kamacite_receipt_hash_blake3: None,
        source_archive_digest_blake3: None,
        release_binary_relative_path: None,
        release_binary_digest_blake3: None,
        boundary: FUNCTION_ADDRESS_OPAQUE_BOUNDARY.to_string(),
        diagnostics,
    };
    debug_assert_eq!(verification.disposition, FUNCTION_ADDRESS_DISPOSITION_ABSENT);
    debug_assert!(verification.sidecar_digest_blake3.is_none());
    verification
}

fn stack_provenance_mode_is_supported(mode: &str) -> bool {
    mode == STACK_PROVENANCE_MODE_OPTIONAL || mode == STACK_PROVENANCE_MODE_REQUIRED
}

fn function_address_mode_is_supported(mode: &str) -> bool {
    mode == FUNCTION_ADDRESS_MODE_OPTIONAL || mode == FUNCTION_ADDRESS_MODE_REQUIRED
}

fn validate_stack_provenance_manifest_evidence(manifest: &ReleaseEvidenceManifest) -> Result<(), ReleaseEvidenceError> {
    let Some(evidence) = &manifest.stack_provenance else {
        return Ok(());
    };
    let diagnostics = stack_provenance_evidence_diagnostics(manifest, evidence);
    if let Some(first) = diagnostics.first() {
        return Err(validation_error(format!("release evidence stack_provenance invalid: {first}")));
    }
    Ok(())
}

fn stack_provenance_evidence_diagnostics(
    manifest: &ReleaseEvidenceManifest,
    evidence: &StackProvenanceReleaseEvidence,
) -> Vec<String> {
    let mut diagnostics = Vec::new();
    validate_stack_provenance_literals(evidence, &mut diagnostics);
    validate_stack_provenance_paths_and_hashes(evidence, &mut diagnostics);
    validate_stack_provenance_non_claims(&evidence.non_claims, "stack_provenance.non_claims", &mut diagnostics);
    validate_stack_provenance_binary_link(manifest, evidence, &mut diagnostics);
    validate_stack_provenance_external_link(
        &manifest.external_evidence,
        &evidence.sidecar_role,
        &evidence.sidecar_schema,
        &evidence.sidecar_claim_scope,
        &evidence.sidecar_relative_path,
        &evidence.sidecar_digest_blake3,
        "sidecar",
        &mut diagnostics,
    );
    validate_stack_provenance_external_link(
        &manifest.external_evidence,
        &evidence.valence_receipt_role,
        &evidence.valence_receipt_schema,
        STACK_PROVENANCE_CLAIM_SCOPE,
        &evidence.valence_receipt_relative_path,
        &evidence.valence_receipt_digest_blake3,
        "Valence receipt",
        &mut diagnostics,
    );
    diagnostics
}

fn validate_stack_provenance_literals(evidence: &StackProvenanceReleaseEvidence, diagnostics: &mut Vec<String>) {
    push_literal_diagnostic(
        &evidence.sidecar_role,
        STACK_PROVENANCE_EVIDENCE_ROLE,
        "stack_provenance.sidecar_role",
        diagnostics,
    );
    push_literal_diagnostic(
        &evidence.sidecar_schema,
        STACK_PROVENANCE_SIDECAR_SCHEMA,
        "stack_provenance.sidecar_schema",
        diagnostics,
    );
    push_literal_diagnostic(
        &evidence.sidecar_claim_scope,
        STACK_PROVENANCE_CLAIM_SCOPE,
        "stack_provenance.sidecar_claim_scope",
        diagnostics,
    );
    push_literal_diagnostic(
        &evidence.valence_receipt_role,
        VALENCE_STACK_PROVENANCE_RECEIPT_ROLE,
        "stack_provenance.valence_receipt_role",
        diagnostics,
    );
    push_literal_diagnostic(
        &evidence.valence_receipt_schema,
        STACK_PROVENANCE_GRAPH_REPORT_SCHEMA,
        "stack_provenance.valence_receipt_schema",
        diagnostics,
    );
}

fn push_literal_diagnostic(actual: &str, expected: &str, field_name: &str, diagnostics: &mut Vec<String>) {
    if actual != expected {
        diagnostics.push(format!("{field_name} must be {expected}, got {actual}"));
    }
}

fn validate_stack_provenance_paths_and_hashes(
    evidence: &StackProvenanceReleaseEvidence,
    diagnostics: &mut Vec<String>,
) {
    push_relative_path_diagnostic(
        &evidence.sidecar_relative_path,
        "stack_provenance.sidecar_relative_path",
        diagnostics,
    );
    push_relative_path_diagnostic(
        &evidence.valence_receipt_relative_path,
        "stack_provenance.valence_receipt_relative_path",
        diagnostics,
    );
    push_relative_path_diagnostic(
        &evidence.release_binary_relative_path,
        "stack_provenance.release_binary_relative_path",
        diagnostics,
    );
    push_blake3_diagnostic(&evidence.sidecar_digest_blake3, "stack_provenance.sidecar_digest_blake3", diagnostics);
    push_blake3_diagnostic(
        &evidence.valence_receipt_digest_blake3,
        "stack_provenance.valence_receipt_digest_blake3",
        diagnostics,
    );
    push_blake3_diagnostic(
        &evidence.release_binary_digest_blake3,
        "stack_provenance.release_binary_digest_blake3",
        diagnostics,
    );
}

fn push_relative_path_diagnostic(path: &str, field_name: &str, diagnostics: &mut Vec<String>) {
    if let Err(error) = validate_relative_member_path(path, field_name) {
        diagnostics.push(error.to_string());
    }
}

fn push_blake3_diagnostic(value: &str, field_name: &str, diagnostics: &mut Vec<String>) {
    if let Err(error) = validate_blake3_hex(value, field_name) {
        diagnostics.push(error.to_string());
    }
}

fn validate_stack_provenance_binary_link(
    manifest: &ReleaseEvidenceManifest,
    evidence: &StackProvenanceReleaseEvidence,
    diagnostics: &mut Vec<String>,
) {
    let Some(binary) = manifest
        .binaries
        .iter()
        .find(|binary| binary.relative_path == evidence.release_binary_relative_path)
    else {
        diagnostics.push("stack_provenance.release_binary_relative_path does not match a bundled binary".to_string());
        return;
    };
    if binary.digest_blake3 != evidence.release_binary_digest_blake3 {
        diagnostics.push("stack_provenance.release_binary_digest_blake3 does not match the bundled binary".to_string());
    }
}

fn validate_stack_provenance_external_link(
    external_evidence: &[ExternalEvidence],
    role: &str,
    schema: &str,
    claim_scope: &str,
    relative_path: &str,
    digest_blake3: &str,
    label: &str,
    diagnostics: &mut Vec<String>,
) {
    let Some(external) = external_evidence.iter().find(|external| external.role == role) else {
        diagnostics.push(format!("stack provenance {label} external evidence is missing"));
        return;
    };
    if external.schema != schema {
        diagnostics.push(format!("stack provenance {label} schema does not match declared metadata"));
    }
    if external.claim_scope != claim_scope {
        diagnostics.push(format!("stack provenance {label} claim scope does not match declared metadata"));
    }
    if external.relative_path != relative_path {
        diagnostics.push(format!("stack provenance {label} path does not match declared metadata"));
    }
    if external.digest_blake3 != digest_blake3 {
        diagnostics.push(format!("stack provenance {label} digest does not match declared metadata"));
    }
    validate_stack_provenance_non_claims(&external.non_claims, label, diagnostics);
}

fn validate_stack_provenance_non_claims(non_claims: &[String], field_name: &str, diagnostics: &mut Vec<String>) {
    if !non_claims.iter().any(|non_claim| non_claim == STACK_PROVENANCE_OPAQUE_BOUNDARY) {
        diagnostics.push(format!("{field_name} missing Mantle opaque stack-provenance non-claim"));
    }
    for non_claim in non_claims {
        if stack_provenance_text_overclaims(non_claim) {
            diagnostics.push(format!("{field_name} contains stack provenance overclaim"));
        }
    }
}

fn stack_provenance_text_overclaims(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    STACK_PROVENANCE_OVERCLAIM_FRAGMENTS.iter().any(|fragment| lower.contains(fragment))
}

fn validate_function_address_manifest_evidence(manifest: &ReleaseEvidenceManifest) -> Result<(), ReleaseEvidenceError> {
    let Some(evidence) = &manifest.function_address_evidence else {
        return Ok(());
    };
    let diagnostics = function_address_evidence_diagnostics(manifest, evidence);
    if let Some(first) = diagnostics.first() {
        return Err(validation_error(format!("release evidence function_address_evidence invalid: {first}")));
    }
    Ok(())
}

fn function_address_evidence_diagnostics(
    manifest: &ReleaseEvidenceManifest,
    evidence: &FunctionAddressReleaseEvidence,
) -> Vec<String> {
    let mut diagnostics = Vec::new();
    validate_function_address_literals(evidence, &mut diagnostics);
    validate_function_address_paths_and_hashes(evidence, &mut diagnostics);
    validate_function_address_non_claims(
        &evidence.non_claims,
        "function_address_evidence.non_claims",
        &mut diagnostics,
    );
    validate_function_address_source_link(manifest, evidence, &mut diagnostics);
    validate_function_address_binary_link(manifest, evidence, &mut diagnostics);
    validate_function_address_external_link(
        &manifest.external_evidence,
        &evidence.sidecar_role,
        &evidence.sidecar_schema,
        &evidence.sidecar_claim_scope,
        &evidence.sidecar_relative_path,
        &evidence.sidecar_digest_blake3,
        "sidecar",
        &mut diagnostics,
    );
    validate_function_address_external_link(
        &manifest.external_evidence,
        &evidence.valence_receipt_role,
        &evidence.valence_receipt_schema,
        FUNCTION_ADDRESS_CLAIM_SCOPE,
        &evidence.valence_receipt_relative_path,
        &evidence.valence_receipt_digest_blake3,
        "Valence receipt",
        &mut diagnostics,
    );
    validate_optional_kamacite_function_address_external_link(manifest, evidence, &mut diagnostics);
    diagnostics
}

fn validate_function_address_literals(evidence: &FunctionAddressReleaseEvidence, diagnostics: &mut Vec<String>) {
    push_literal_diagnostic(
        &evidence.sidecar_role,
        FUNCTION_ADDRESS_EVIDENCE_ROLE,
        "function_address_evidence.sidecar_role",
        diagnostics,
    );
    push_literal_diagnostic(
        &evidence.sidecar_schema,
        FUNCTION_ADDRESS_EVIDENCE_SCHEMA,
        "function_address_evidence.sidecar_schema",
        diagnostics,
    );
    push_literal_diagnostic(
        &evidence.sidecar_claim_scope,
        FUNCTION_ADDRESS_CLAIM_SCOPE,
        "function_address_evidence.sidecar_claim_scope",
        diagnostics,
    );
    push_literal_diagnostic(
        &evidence.valence_receipt_role,
        VALENCE_FUNCTION_ADDRESS_RECEIPT_ROLE,
        "function_address_evidence.valence_receipt_role",
        diagnostics,
    );
    push_literal_diagnostic(
        &evidence.valence_receipt_schema,
        VALENCE_FUNCTION_ADDRESS_RECEIPT_SCHEMA,
        "function_address_evidence.valence_receipt_schema",
        diagnostics,
    );
}

fn validate_function_address_paths_and_hashes(
    evidence: &FunctionAddressReleaseEvidence,
    diagnostics: &mut Vec<String>,
) {
    push_relative_path_diagnostic(
        &evidence.sidecar_relative_path,
        "function_address_evidence.sidecar_relative_path",
        diagnostics,
    );
    push_relative_path_diagnostic(
        &evidence.valence_receipt_relative_path,
        "function_address_evidence.valence_receipt_relative_path",
        diagnostics,
    );
    push_relative_path_diagnostic(
        &evidence.release_binary_relative_path,
        "function_address_evidence.release_binary_relative_path",
        diagnostics,
    );
    push_blake3_diagnostic(
        &evidence.sidecar_digest_blake3,
        "function_address_evidence.sidecar_digest_blake3",
        diagnostics,
    );
    push_blake3_diagnostic(
        &evidence.valence_receipt_digest_blake3,
        "function_address_evidence.valence_receipt_digest_blake3",
        diagnostics,
    );
    push_blake3_diagnostic(
        &evidence.source_archive_digest_blake3,
        "function_address_evidence.source_archive_digest_blake3",
        diagnostics,
    );
    push_blake3_diagnostic(
        &evidence.release_binary_digest_blake3,
        "function_address_evidence.release_binary_digest_blake3",
        diagnostics,
    );
}

fn validate_function_address_source_link(
    manifest: &ReleaseEvidenceManifest,
    evidence: &FunctionAddressReleaseEvidence,
    diagnostics: &mut Vec<String>,
) {
    if manifest.source_archive.digest_blake3 != evidence.source_archive_digest_blake3 {
        diagnostics.push(
            "function_address_evidence.source_archive_digest_blake3 does not match the bundled source archive"
                .to_string(),
        );
    }
}

fn validate_function_address_binary_link(
    manifest: &ReleaseEvidenceManifest,
    evidence: &FunctionAddressReleaseEvidence,
    diagnostics: &mut Vec<String>,
) {
    let Some(binary) = manifest
        .binaries
        .iter()
        .find(|binary| binary.relative_path == evidence.release_binary_relative_path)
    else {
        diagnostics
            .push("function_address_evidence.release_binary_relative_path does not match a bundled binary".to_string());
        return;
    };
    if binary.digest_blake3 != evidence.release_binary_digest_blake3 {
        diagnostics.push(
            "function_address_evidence.release_binary_digest_blake3 does not match the bundled binary".to_string(),
        );
    }
}

fn validate_function_address_external_link(
    external_evidence: &[ExternalEvidence],
    role: &str,
    schema: &str,
    claim_scope: &str,
    relative_path: &str,
    digest_blake3: &str,
    label: &str,
    diagnostics: &mut Vec<String>,
) {
    let Some(external) = external_evidence.iter().find(|external| external.role == role) else {
        diagnostics.push(format!("function-address {label} external evidence is missing"));
        return;
    };
    if external.schema != schema {
        diagnostics.push(format!("function-address {label} schema does not match declared metadata"));
    }
    if external.claim_scope != claim_scope {
        diagnostics.push(format!("function-address {label} claim scope does not match declared metadata"));
    }
    if external.relative_path != relative_path {
        diagnostics.push(format!("function-address {label} path does not match declared metadata"));
    }
    if external.digest_blake3 != digest_blake3 {
        diagnostics.push(format!("function-address {label} digest does not match declared metadata"));
    }
    validate_function_address_non_claims(&external.non_claims, label, diagnostics);
}

fn validate_optional_kamacite_function_address_external_link(
    manifest: &ReleaseEvidenceManifest,
    evidence: &FunctionAddressReleaseEvidence,
    diagnostics: &mut Vec<String>,
) {
    match (
        &evidence.kamacite_receipt_role,
        &evidence.kamacite_receipt_schema,
        &evidence.kamacite_receipt_relative_path,
        &evidence.kamacite_receipt_digest_blake3,
    ) {
        (None, None, None, None) => {}
        (Some(role), Some(schema), Some(relative_path), Some(digest_blake3)) => {
            push_literal_diagnostic(
                role,
                KAMACITE_FUNCTION_ADDRESS_RECEIPT_ROLE,
                "function_address_evidence.kamacite_receipt_role",
                diagnostics,
            );
            push_literal_diagnostic(
                schema,
                KAMACITE_FUNCTION_ADDRESS_RECEIPT_SCHEMA,
                "function_address_evidence.kamacite_receipt_schema",
                diagnostics,
            );
            push_relative_path_diagnostic(
                relative_path,
                "function_address_evidence.kamacite_receipt_relative_path",
                diagnostics,
            );
            push_blake3_diagnostic(
                digest_blake3,
                "function_address_evidence.kamacite_receipt_digest_blake3",
                diagnostics,
            );
            validate_function_address_external_link(
                &manifest.external_evidence,
                role,
                schema,
                FUNCTION_ADDRESS_CLAIM_SCOPE,
                relative_path,
                digest_blake3,
                "Kamacite receipt",
                diagnostics,
            );
        }
        _ => diagnostics
            .push("function_address_evidence Kamacite receipt metadata must be all present or all absent".to_string()),
    }
}

fn validate_function_address_non_claims(non_claims: &[String], field_name: &str, diagnostics: &mut Vec<String>) {
    if !non_claims.iter().any(|non_claim| non_claim == FUNCTION_ADDRESS_OPAQUE_BOUNDARY) {
        diagnostics.push(format!("{field_name} missing Mantle opaque function-address non-claim"));
    }
    for non_claim in non_claims {
        if function_address_text_overclaims(non_claim) {
            diagnostics.push(format!("{field_name} contains function-address overclaim"));
        }
    }
}

fn function_address_text_overclaims(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    FUNCTION_ADDRESS_OVERCLAIM_FRAGMENTS.iter().any(|fragment| lower.contains(fragment))
}

fn validate_external_evidence(manifest: &ReleaseEvidenceManifest) -> Result<(), ReleaseEvidenceError> {
    let mut seen_roles = BTreeSet::new();
    for (index_usize, evidence) in manifest.external_evidence.iter().enumerate() {
        let index_u32 = u32_count(index_usize, "external evidence index overflowed u32")?;
        let field_name = format!("external_evidence[{index_u32}]");
        validate_external_evidence_entry(evidence, &field_name)?;
        if !seen_roles.insert(evidence.role.clone()) {
            return Err(validation_error(format!(
                "release evidence {field_name}.role duplicates another external evidence role"
            )));
        }
    }
    Ok(())
}

fn validate_external_evidence_entry(evidence: &ExternalEvidence, field_name: &str) -> Result<(), ReleaseEvidenceError> {
    if evidence.role.trim().is_empty() {
        return Err(validation_error(format!("release evidence {field_name}.role must not be empty")));
    }
    if evidence.schema.trim().is_empty() {
        return Err(validation_error(format!("release evidence {field_name}.schema must not be empty")));
    }
    validate_relative_member_path(&evidence.relative_path, &format!("{field_name}.relative_path"))?;
    validate_blake3_hex(&evidence.digest_blake3, &format!("{field_name}.digest_blake3"))?;
    if evidence.claim_scope.trim().is_empty() {
        return Err(validation_error(format!("release evidence {field_name}.claim_scope must not be empty")));
    }
    if evidence.non_claims.is_empty() {
        return Err(validation_error(format!("release evidence {field_name}.non_claims must not be empty")));
    }
    for (claim_index_usize, non_claim) in evidence.non_claims.iter().enumerate() {
        let claim_index_u32 = u32_count(claim_index_usize, "external evidence non-claim index overflowed u32")?;
        if non_claim.trim().is_empty() {
            return Err(validation_error(format!(
                "release evidence {field_name}.non_claims[{claim_index_u32}] must not be empty"
            )));
        }
    }
    Ok(())
}

fn validate_kani_toolchain_evidence(manifest: &ReleaseEvidenceManifest) -> Result<(), ReleaseEvidenceError> {
    let mut seen_roles = BTreeSet::new();
    for (index_usize, evidence) in manifest.kani_toolchain_evidence.iter().enumerate() {
        let index_u32 = u32_count(index_usize, "Kani toolchain evidence index overflowed u32")?;
        let field_name = format!("kani_toolchain_evidence[{index_u32}]");
        validate_kani_toolchain_entry(evidence, &field_name)?;
        if !seen_roles.insert(evidence.receipt_role.clone()) {
            return Err(validation_error(format!(
                "release evidence {field_name}.receipt_role duplicates another Kani receipt role"
            )));
        }
        validate_kani_external_evidence_link(evidence, &manifest.external_evidence, &field_name)?;
    }
    for external in &manifest.external_evidence {
        if external.role == KANI_RECEIPT_EVIDENCE_ROLE
            && !manifest.kani_toolchain_evidence.iter().any(|evidence| evidence.receipt_role == external.role)
        {
            return Err(validation_error(
                "release evidence Kani receipt external evidence requires matching kani_toolchain_evidence".to_string(),
            ));
        }
    }
    Ok(())
}

fn validate_kani_toolchain_entry(
    evidence: &KaniToolchainEvidence,
    field_name: &str,
) -> Result<(), ReleaseEvidenceError> {
    validate_required_literal_string(
        &evidence.schema,
        KANI_TOOLCHAIN_EVIDENCE_SCHEMA,
        &format!("{field_name}.schema"),
    )?;
    validate_non_empty_string(&evidence.receipt_role, &format!("{field_name}.receipt_role"))?;
    validate_relative_member_path(&evidence.receipt_relative_path, &format!("{field_name}.receipt_relative_path"))?;
    validate_blake3_hex(&evidence.receipt_digest_blake3, &format!("{field_name}.receipt_digest_blake3"))?;
    validate_non_empty_string(&evidence.kani_version, &format!("{field_name}.kani_version"))?;
    validate_non_empty_string(&evidence.rust_toolchain, &format!("{field_name}.rust_toolchain"))?;
    validate_non_empty_string(&evidence.cbmc_version, &format!("{field_name}.cbmc_version"))?;
    validate_kani_solver_identity(&evidence.solver, &format!("{field_name}.solver"))?;
    validate_non_empty_string(&evidence.invocation_wrapper, &format!("{field_name}.invocation_wrapper"))?;
    validate_blake3_hex(&evidence.closure_identity_blake3, &format!("{field_name}.closure_identity_blake3"))?;
    validate_blake3_hex(
        &evidence.expected_closure_identity_blake3,
        &format!("{field_name}.expected_closure_identity_blake3"),
    )?;
    if evidence.closure_identity_blake3 != evidence.expected_closure_identity_blake3 {
        return Err(validation_error(format!(
            "release evidence {field_name}.closure_identity_blake3 is stale or does not match expected_closure_identity_blake3"
        )));
    }
    validate_required_literal_string(
        &evidence.valence_semantic_role,
        KANI_VALENCE_SEMANTIC_ROLE,
        &format!("{field_name}.valence_semantic_role"),
    )?;
    validate_required_literal_string(
        &evidence.claim_scope,
        KANI_EVIDENCE_CLAIM_SCOPE,
        &format!("{field_name}.claim_scope"),
    )?;
    validate_kani_non_claims(&evidence.non_claims, &format!("{field_name}.non_claims"))
}

fn validate_kani_solver_identity(solver: &KaniSolverIdentity, field_name: &str) -> Result<(), ReleaseEvidenceError> {
    validate_non_empty_string(&solver.kind, &format!("{field_name}.kind"))?;
    validate_non_empty_string(&solver.version, &format!("{field_name}.version"))?;
    if !KANI_SUPPORTED_SOLVER_KINDS.contains(&solver.kind.as_str()) {
        return Err(validation_error(format!(
            "release evidence {field_name}.kind has unsupported Kani solver metadata: {}",
            solver.kind
        )));
    }
    Ok(())
}

fn validate_kani_external_evidence_link(
    evidence: &KaniToolchainEvidence,
    external_evidence: &[ExternalEvidence],
    field_name: &str,
) -> Result<(), ReleaseEvidenceError> {
    let Some(external) = external_evidence.iter().find(|external| external.role == evidence.receipt_role) else {
        return Err(validation_error(format!(
            "release evidence {field_name}.receipt_role does not match a bundled external evidence role"
        )));
    };
    if external.relative_path != evidence.receipt_relative_path {
        return Err(validation_error(format!(
            "release evidence {field_name}.receipt_relative_path does not match the Kani external evidence path"
        )));
    }
    if external.digest_blake3 != evidence.receipt_digest_blake3 {
        return Err(validation_error(format!(
            "release evidence {field_name}.receipt_digest_blake3 does not match the Kani external evidence digest"
        )));
    }
    if external.claim_scope != KANI_EVIDENCE_CLAIM_SCOPE {
        return Err(validation_error(format!(
            "release evidence external evidence for {field_name} must use claim_scope {KANI_EVIDENCE_CLAIM_SCOPE}"
        )));
    }
    validate_kani_non_claims(&external.non_claims, &format!("external evidence for {field_name}.non_claims"))
}

fn validate_kani_non_claims(non_claims: &[String], field_name: &str) -> Result<(), ReleaseEvidenceError> {
    if non_claims.is_empty() {
        return Err(validation_error(format!("release evidence {field_name} must not be empty")));
    }
    for required in KANI_REQUIRED_NON_CLAIMS {
        if !non_claims.iter().any(|non_claim| non_claim == required) {
            return Err(validation_error(format!(
                "release evidence {field_name} missing required Kani non-claim {required}"
            )));
        }
    }
    for (index_usize, non_claim) in non_claims.iter().enumerate() {
        let index_u32 = u32_count(index_usize, "Kani non-claim index overflowed u32")?;
        validate_non_empty_string(non_claim, &format!("{field_name}[{index_u32}]"))?;
    }
    Ok(())
}

fn validate_non_empty_string(value: &str, field_name: &str) -> Result<(), ReleaseEvidenceError> {
    if value.trim().is_empty() {
        return Err(validation_error(format!("release evidence {field_name} must not be empty")));
    }
    Ok(())
}

fn validate_required_literal_string(
    actual: &str,
    expected: &str,
    field_name: &str,
) -> Result<(), ReleaseEvidenceError> {
    validate_non_empty_string(actual, field_name)?;
    if actual != expected {
        return Err(validation_error(format!("release evidence {field_name} must be {expected}, got {actual}")));
    }
    Ok(())
}

fn validate_provenance_coverage(manifest: &ReleaseEvidenceManifest) -> Result<(), ReleaseEvidenceError> {
    let Some(coverage) = &manifest.provenance_coverage else {
        return Ok(());
    };
    validate_blake3_hex(&coverage.binary_hash, "provenance_coverage.binary_hash")?;
    if coverage.coverage_boundary != PROVENANCE_COVERAGE_BOUNDARY {
        return Err(validation_error(
            "provenance_coverage.coverage_boundary must match the required non-claim boundary".to_string(),
        ));
    }
    if coverage.covered_source_ids.is_empty()
        && coverage.covered_function_object_ids.is_empty()
        && coverage.covered_requirement_ids.is_empty()
    {
        return Err(validation_error(
            "provenance_coverage must record at least one covered source_id, function_object_id, or requirement_id"
                .to_string(),
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FullSelfHostingProofIdentityFields {
    pub schema: String,
    pub proof_mode: String,
    pub selected_provider_kind: String,
    pub staged_source: String,
    pub stage2_binary_digest_blake3: String,
    pub prerequisite_inventory_digest_blake3: String,
}

pub fn extract_full_self_hosting_proof_identity_fields(
    manifest_bytes: Vec<u8>,
) -> Result<FullSelfHostingProofIdentityFields, ReleaseEvidenceError> {
    let manifest: SelfHostingProofManifestView = serde_json::from_slice(&manifest_bytes).map_err(|err| {
        parse_error(format!("full proof artifact required: parsing full proof manifest failed: {err}"))
    })?;
    validate_full_proof_manifest(&manifest)?;
    Ok(FullSelfHostingProofIdentityFields {
        schema: manifest.schema,
        proof_mode: manifest.prerequisites.mode,
        selected_provider_kind: manifest.prerequisites.provider_kind,
        staged_source: manifest.staged_source,
        stage2_binary_digest_blake3: manifest.binaries.stage2.digest_blake3,
        prerequisite_inventory_digest_blake3: manifest.prerequisites.inventory_doc.digest_blake3,
    })
}

#[derive(Debug, Deserialize)]
struct SelfHostingProofManifestView {
    schema: String,
    staged_source: String,
    prerequisites: SelfHostingProofPrerequisitesView,
    binaries: SelfHostingProofBinariesView,
    tools: SelfHostingProofToolsView,
    fixed_point: SelfHostingProofFixedPointView,
    stage0: SelfHostingProofStageView,
    stage2: SelfHostingProofStageView,
}

#[derive(Debug, Deserialize)]
struct SelfHostingProofPrerequisitesView {
    mode: String,
    provider_kind: String,
    inventory_doc: SelfHostingProofHashedPath,
}

#[derive(Debug, Deserialize)]
struct SelfHostingProofBinariesView {
    stage1: SelfHostingProofHashedPath,
    stage2: SelfHostingProofHashedPath,
}

#[derive(Debug, Deserialize)]
struct SelfHostingProofToolsView {
    stage0_bwrap: SelfHostingProofHashedPath,
    stage0_busybox: SelfHostingProofHashedPath,
    stage2_bwrap: SelfHostingProofHashedPath,
    stage2_busybox: SelfHostingProofHashedPath,
}

#[derive(Debug, Deserialize)]
struct SelfHostingProofFixedPointView {
    stage1_equals_stage2: bool,
    stage0_bwrap_equals_stage2_bwrap: bool,
    stage0_busybox_equals_stage2_busybox: bool,
}

#[derive(Debug, Deserialize)]
struct SelfHostingProofStageView {
    report: SelfHostingProofReportView,
}

#[derive(Debug, Deserialize)]
struct SelfHostingProofReportView {
    staged_source: String,
    output_binary: String,
    busybox_path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SelfHostingProofHashedPath {
    path: String,
    size_bytes: u64,
    digest_blake3: String,
}

fn validate_manifest_header(manifest: &ReleaseEvidenceManifest) -> Result<(), ReleaseEvidenceError> {
    if manifest.schema != RELEASE_EVIDENCE_SCHEMA {
        return Err(validation_error(format!(
            "release evidence schema must be {RELEASE_EVIDENCE_SCHEMA}, got {}",
            manifest.schema
        )));
    }
    if manifest.release_id.trim().is_empty() {
        return Err(validation_error("release evidence release_id must not be empty".to_string()));
    }
    if manifest.claim_scope != CLAIM_SCOPE_PACKAGED_INTEGRITY {
        return Err(validation_error(format!(
            "release evidence claim_scope must be {CLAIM_SCOPE_PACKAGED_INTEGRITY}, got {}",
            manifest.claim_scope
        )));
    }
    if manifest.workflow.command.trim().is_empty() {
        return Err(validation_error("release evidence workflow.command must not be empty".to_string()));
    }
    if manifest.workflow.version.trim().is_empty() {
        return Err(validation_error("release evidence workflow.version must not be empty".to_string()));
    }
    Ok(())
}

fn validate_manifest_artifacts(manifest: &ReleaseEvidenceManifest) -> Result<(), ReleaseEvidenceError> {
    let binary_count = u32_count(manifest.binaries.len(), "release evidence binary artifact count overflowed u32")?;
    if binary_count == 0 {
        return Err(validation_error("release evidence must record at least one binary artifact".to_string()));
    }
    if binary_count > MAX_BINARY_ARTIFACTS_COUNT {
        return Err(validation_error(format!(
            "release evidence records {binary_count} binary artifacts, limit is {MAX_BINARY_ARTIFACTS_COUNT}"
        )));
    }

    let mut seen_paths = BTreeSet::new();
    validate_and_record_path(&manifest.source_archive, "source_archive", &mut seen_paths)?;
    validate_and_record_path(&manifest.proof_bundle, "proof_bundle", &mut seen_paths)?;
    validate_and_record_path(&manifest.prerequisite_inventory, "prerequisite_inventory", &mut seen_paths)?;
    if let Some(proof) = &manifest.provider_fixed_point_proof {
        validate_provider_fixed_point_proof_artifact(proof, &mut seen_paths)?;
    }
    if let Some(report) = &manifest.reproducibility_report {
        validate_and_record_path(report, "reproducibility_report", &mut seen_paths)?;
        if report.kind != BundledArtifactKind::File {
            return Err(validation_error(
                "release evidence reproducibility_report must be recorded as a file artifact".to_string(),
            ));
        }
    }
    if let Some(proof) = &manifest.deterministic_build_proof {
        validate_role_bounded_release_artifact(
            proof,
            "deterministic_build_proof",
            DETERMINISTIC_BUILD_PROOF_EVIDENCE_ROLE,
            "deterministic-release/deterministic-build-proof.json",
            &mut seen_paths,
        )?;
    }
    if let Some(evidence) = &manifest.deterministic_sandbox_isolation_evidence {
        validate_role_bounded_release_artifact(
            evidence,
            "deterministic_sandbox_isolation_evidence",
            DETERMINISTIC_SANDBOX_ISOLATION_EVIDENCE_ROLE,
            "deterministic-release/deterministic-sandbox-isolation-evidence.json",
            &mut seen_paths,
        )?;
    }
    if let Some(report) = &manifest.independent_agreement_report {
        validate_and_record_path(report, "independent_agreement_report", &mut seen_paths)?;
        if report.kind != BundledArtifactKind::File {
            return Err(validation_error(
                "release evidence independent_agreement_report must be recorded as a file artifact".to_string(),
            ));
        }
        if report.relative_path != "independent-agreement/agreement-report.json" {
            return Err(validation_error(
                "release evidence independent_agreement_report must be independent-agreement/agreement-report.json"
                    .to_string(),
            ));
        }
    }
    for (index_usize, artifact) in manifest.binaries.iter().enumerate() {
        let index_u32 = u32_count(index_usize, "release evidence binary index overflowed u32")?;
        validate_and_record_path(artifact, &format!("binaries[{index_u32}]"), &mut seen_paths)?;
    }
    for (index_usize, evidence) in manifest.external_evidence.iter().enumerate() {
        let index_u32 = u32_count(index_usize, "external evidence index overflowed u32")?;
        record_unique_artifact_path(&evidence.relative_path, &mut seen_paths).map_err(|err| {
            validation_error(format!("release evidence external_evidence[{index_u32}].relative_path conflict: {err}"))
        })?;
    }
    Ok(())
}

fn validate_and_record_path(
    artifact: &BundledArtifact,
    field_name: &str,
    seen_paths: &mut BTreeSet<String>,
) -> Result<(), ReleaseEvidenceError> {
    validate_bundled_artifact(artifact, field_name)?;
    record_unique_artifact_path(&artifact.relative_path, seen_paths)
}

fn validate_provider_fixed_point_proof_artifact(
    artifact: &ProviderFixedPointProofArtifact,
    seen_paths: &mut BTreeSet<String>,
) -> Result<(), ReleaseEvidenceError> {
    validate_bundled_artifact(&provider_fixed_point_bundled_artifact(artifact), "provider_fixed_point_proof")?;
    if artifact.kind != BundledArtifactKind::Directory {
        return Err(validation_error(
            "release evidence provider_fixed_point_proof must be recorded as a directory artifact".to_string(),
        ));
    }
    if artifact.evidence_role != PROVIDER_FIXED_POINT_PROOF_EVIDENCE_ROLE {
        return Err(validation_error(format!(
            "release evidence provider_fixed_point_proof.evidence_role must be {PROVIDER_FIXED_POINT_PROOF_EVIDENCE_ROLE}, got {}",
            artifact.evidence_role
        )));
    }
    record_unique_artifact_path(&artifact.relative_path, seen_paths)
}

fn provider_fixed_point_bundled_artifact(artifact: &ProviderFixedPointProofArtifact) -> BundledArtifact {
    BundledArtifact {
        kind: artifact.kind,
        relative_path: artifact.relative_path.clone(),
        size_bytes: artifact.size_bytes,
        digest_blake3: artifact.digest_blake3.clone(),
    }
}

fn validate_role_bounded_release_artifact(
    artifact: &RoleBoundedReleaseArtifact,
    field_name: &str,
    expected_role: &str,
    expected_relative_path: &str,
    seen_paths: &mut BTreeSet<String>,
) -> Result<(), ReleaseEvidenceError> {
    validate_bundled_artifact(&role_bounded_bundled_artifact(artifact), field_name)?;
    if artifact.kind != BundledArtifactKind::File {
        return Err(validation_error(format!("release evidence {field_name} must be recorded as a file artifact")));
    }
    if artifact.evidence_role != expected_role {
        return Err(validation_error(format!(
            "release evidence {field_name}.evidence_role must be {expected_role}, got {}",
            artifact.evidence_role
        )));
    }
    if artifact.relative_path != expected_relative_path {
        return Err(validation_error(format!("release evidence {field_name} must be {expected_relative_path}")));
    }
    record_unique_artifact_path(&artifact.relative_path, seen_paths)
}

fn role_bounded_bundled_artifact(artifact: &RoleBoundedReleaseArtifact) -> BundledArtifact {
    BundledArtifact {
        kind: artifact.kind,
        relative_path: artifact.relative_path.clone(),
        size_bytes: artifact.size_bytes,
        digest_blake3: artifact.digest_blake3.clone(),
    }
}

fn record_unique_artifact_path(
    relative_path: &str,
    seen_paths: &mut BTreeSet<String>,
) -> Result<(), ReleaseEvidenceError> {
    if !seen_paths.insert(relative_path.to_string()) {
        return Err(validation_error(format!(
            "release evidence contains duplicate bundle member path {relative_path}"
        )));
    }
    Ok(())
}

fn validate_source_acquisition(manifest: &ReleaseEvidenceManifest) -> Result<(), ReleaseEvidenceError> {
    let Some(source_acquisition) = &manifest.source_acquisition else {
        return Ok(());
    };
    validate_source_acquisition_digest(source_acquisition, manifest)?;
    match source_acquisition.kind.as_str() {
        SOURCE_ACQUISITION_KIND_EXTERNAL_ARCHIVE => validate_external_source_acquisition(source_acquisition),
        SOURCE_ACQUISITION_KIND_GIT => validate_git_source_acquisition(source_acquisition),
        other => Err(validation_error(format!(
            "release evidence source_acquisition.kind must be {SOURCE_ACQUISITION_KIND_EXTERNAL_ARCHIVE} or {SOURCE_ACQUISITION_KIND_GIT}, got {other}"
        ))),
    }
}

fn validate_source_acquisition_digest(
    source_acquisition: &SourceAcquisition,
    manifest: &ReleaseEvidenceManifest,
) -> Result<(), ReleaseEvidenceError> {
    validate_blake3_hex(&source_acquisition.digest_blake3, "source_acquisition.digest_blake3")?;
    if source_acquisition.digest_blake3 != manifest.source_archive.digest_blake3 {
        return Err(validation_error(
            "release evidence source_acquisition.digest_blake3 must match source_archive.digest_blake3".to_string(),
        ));
    }
    Ok(())
}

fn validate_external_source_acquisition(source_acquisition: &SourceAcquisition) -> Result<(), ReleaseEvidenceError> {
    validate_source_acquisition_url(
        &source_acquisition.url,
        &["file", "http", "https"],
        "file://, http://, or https://",
    )?;
    if source_acquisition.commit.is_some()
        || source_acquisition.reference.is_some()
        || source_acquisition.tag.is_some()
        || source_acquisition.archive_profile.is_some()
        || source_acquisition.archive_version.is_some()
    {
        return Err(validation_error(
            "release evidence external-archive source_acquisition must not carry Git metadata".to_string(),
        ));
    }
    Ok(())
}

fn validate_git_source_acquisition(source_acquisition: &SourceAcquisition) -> Result<(), ReleaseEvidenceError> {
    validate_source_acquisition_url(
        &source_acquisition.url,
        &["file", "http", "https", "git"],
        "file://, http://, https://, or git://",
    )?;
    let commit = required_source_acquisition_field(&source_acquisition.commit, "source_acquisition.commit")?;
    validate_git_commit_hex(commit)?;
    if let Some(reference) = &source_acquisition.reference {
        validate_git_ref_text(reference, "source_acquisition.reference")?;
    }
    if let Some(tag) = &source_acquisition.tag {
        validate_git_ref_text(tag, "source_acquisition.tag")?;
    }
    validate_required_literal_field(
        &source_acquisition.archive_profile,
        RELEASE_SOURCE_ARCHIVE_PROFILE,
        "source_acquisition.archive_profile",
    )?;
    validate_required_literal_field(
        &source_acquisition.archive_version,
        RELEASE_SOURCE_ARCHIVE_VERSION,
        "source_acquisition.archive_version",
    )?;
    Ok(())
}

fn required_source_acquisition_field<'a>(
    value: &'a Option<String>,
    field_name: &str,
) -> Result<&'a str, ReleaseEvidenceError> {
    let Some(value) = value.as_deref() else {
        return Err(validation_error(format!("release evidence {field_name} is required")));
    };
    if value.trim().is_empty() {
        return Err(validation_error(format!("release evidence {field_name} must not be empty")));
    }
    Ok(value)
}

fn validate_required_literal_field(
    value: &Option<String>,
    expected: &str,
    field_name: &str,
) -> Result<(), ReleaseEvidenceError> {
    let actual = required_source_acquisition_field(value, field_name)?;
    if actual != expected {
        return Err(validation_error(format!("release evidence {field_name} must be {expected}, got {actual}")));
    }
    Ok(())
}

fn validate_git_commit_hex(commit: &str) -> Result<(), ReleaseEvidenceError> {
    let len = commit.len();
    if len != GIT_SHA1_HEX_LENGTH_CHARS && len != GIT_SHA256_HEX_LENGTH_CHARS {
        return Err(validation_error(format!(
            "source_acquisition.commit must be {GIT_SHA1_HEX_LENGTH_CHARS} or {GIT_SHA256_HEX_LENGTH_CHARS} lowercase hex chars, got {len}"
        )));
    }
    if !commit.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()) {
        return Err(validation_error("source_acquisition.commit must contain lowercase hex only".to_string()));
    }
    Ok(())
}

fn validate_git_ref_text(reference: &str, field_name: &str) -> Result<(), ReleaseEvidenceError> {
    if reference.trim().is_empty() {
        return Err(validation_error(format!("release evidence {field_name} must not be empty")));
    }
    let byte_count = u32_count(reference.len(), &format!("release evidence {field_name} length overflowed u32"))?;
    if byte_count > MAX_SOURCE_ACQUISITION_REF_BYTES_COUNT {
        return Err(validation_error(format!(
            "release evidence {field_name} is {byte_count} bytes, limit is {MAX_SOURCE_ACQUISITION_REF_BYTES_COUNT}"
        )));
    }
    if reference.bytes().any(|byte| byte.is_ascii_control()) {
        return Err(validation_error(format!("release evidence {field_name} must not contain control characters")));
    }
    Ok(())
}

fn validate_source_acquisition_url(
    url: &str,
    allowed_schemes: &[&str],
    allowed_description: &str,
) -> Result<(), ReleaseEvidenceError> {
    if url.trim().is_empty() {
        return Err(validation_error("release evidence source_acquisition.url must not be empty".to_string()));
    }
    let byte_count = u32_count(url.len(), "release evidence source_acquisition.url length overflowed u32")?;
    if byte_count > MAX_SOURCE_ACQUISITION_URL_BYTES_COUNT {
        return Err(validation_error(format!(
            "release evidence source_acquisition.url is {byte_count} bytes, limit is {MAX_SOURCE_ACQUISITION_URL_BYTES_COUNT}"
        )));
    }
    let Some((scheme, rest)) = url.split_once("://") else {
        return Err(validation_error(format!(
            "release evidence source_acquisition.url must start with {allowed_description}"
        )));
    };
    if !allowed_schemes.contains(&scheme) {
        return Err(validation_error(format!(
            "release evidence source_acquisition.url must start with {allowed_description}"
        )));
    }
    reject_credential_bearing_url(rest)?;
    Ok(())
}

fn reject_credential_bearing_url(rest_after_scheme: &str) -> Result<(), ReleaseEvidenceError> {
    let authority = rest_after_scheme.split('/').next().unwrap_or("");
    if authority.contains('@') {
        return Err(validation_error(
            "release evidence source_acquisition.url must not contain credentials or URL userinfo".to_string(),
        ));
    }
    Ok(())
}

fn validate_manifest_linkage(manifest: &ReleaseEvidenceManifest) -> Result<(), ReleaseEvidenceError> {
    if manifest.proof_linkage.release_id != manifest.release_id {
        return Err(validation_error("release evidence proof linkage release_id must match release_id".to_string()));
    }
    validate_blake3_hex(
        &manifest.proof_linkage.source_archive_digest_blake3,
        "proof_linkage.source_archive_digest_blake3",
    )?;
    validate_blake3_hex(
        &manifest.proof_linkage.stage2_binary_digest_blake3,
        "proof_linkage.stage2_binary_digest_blake3",
    )?;
    validate_blake3_hex(
        &manifest.proof_linkage.prerequisite_inventory_digest_blake3,
        "proof_linkage.prerequisite_inventory_digest_blake3",
    )?;
    validate_blake3_hex(
        &manifest.proof_linkage.proof_manifest_digest_blake3,
        "proof_linkage.proof_manifest_digest_blake3",
    )?;
    if manifest.proof_linkage.proof_bundle_schema != FULL_SELF_HOSTING_PROOF_SCHEMA {
        return Err(validation_error(format!(
            "release evidence proof_bundle_schema must be {FULL_SELF_HOSTING_PROOF_SCHEMA}, got {}",
            manifest.proof_linkage.proof_bundle_schema
        )));
    }
    if manifest.proof_linkage.proof_mode.trim().is_empty() {
        return Err(validation_error("release evidence proof_mode must not be empty".to_string()));
    }
    validate_provider_kind(&manifest.proof_linkage.selected_provider_kind, "proof_linkage.selected_provider_kind")?;
    if manifest.proof_linkage.staged_source.trim().is_empty() {
        return Err(validation_error("release evidence staged_source must not be empty".to_string()));
    }
    if manifest.proof_bundle.kind != BundledArtifactKind::Directory {
        return Err(validation_error(
            "release evidence proof_bundle must be recorded as a directory artifact".to_string(),
        ));
    }
    if manifest.prerequisite_inventory.kind != BundledArtifactKind::File {
        return Err(validation_error(
            "release evidence prerequisite_inventory must be recorded as a file artifact".to_string(),
        ));
    }
    if manifest.proof_linkage.source_archive_digest_blake3 != manifest.source_archive.digest_blake3 {
        return Err(validation_error(
            "release evidence proof linkage source archive digest does not match bundled source archive".to_string(),
        ));
    }
    if manifest.proof_linkage.prerequisite_inventory_digest_blake3 != manifest.prerequisite_inventory.digest_blake3 {
        return Err(validation_error(
            "release evidence proof linkage prerequisite inventory digest does not match bundled prerequisite inventory"
                .to_string(),
        ));
    }
    if !manifest
        .binaries
        .iter()
        .any(|artifact| artifact.digest_blake3 == manifest.proof_linkage.stage2_binary_digest_blake3)
    {
        return Err(validation_error(
            "release evidence proof linkage stage2 digest does not match any bundled binary artifact".to_string(),
        ));
    }
    Ok(())
}

fn validate_full_proof_manifest(manifest: &SelfHostingProofManifestView) -> Result<(), ReleaseEvidenceError> {
    if manifest.schema != FULL_SELF_HOSTING_PROOF_SCHEMA {
        return Err(validation_error(format!(
            "full proof artifact required: expected schema {FULL_SELF_HOSTING_PROOF_SCHEMA}, got {}",
            manifest.schema
        )));
    }
    if manifest.staged_source.trim().is_empty() {
        return Err(validation_error("full proof artifact required: staged_source is missing".to_string()));
    }
    if manifest.prerequisites.mode.trim().is_empty() {
        return Err(validation_error("full proof artifact required: proof mode is missing".to_string()));
    }
    validate_provider_kind(&manifest.prerequisites.provider_kind, "prerequisites.provider_kind")?;

    validate_hashed_path(&manifest.prerequisites.inventory_doc, "prerequisites.inventory_doc")?;
    validate_hashed_path(&manifest.binaries.stage1, "binaries.stage1")?;
    validate_hashed_path(&manifest.binaries.stage2, "binaries.stage2")?;
    validate_hashed_path(&manifest.tools.stage0_bwrap, "tools.stage0_bwrap")?;
    validate_hashed_path(&manifest.tools.stage0_busybox, "tools.stage0_busybox")?;
    validate_hashed_path(&manifest.tools.stage2_bwrap, "tools.stage2_bwrap")?;
    validate_hashed_path(&manifest.tools.stage2_busybox, "tools.stage2_busybox")?;

    if !manifest.fixed_point.stage1_equals_stage2 {
        return Err(validation_error("full proof artifact required: stage1_equals_stage2 must be true".to_string()));
    }
    if !manifest.fixed_point.stage0_bwrap_equals_stage2_bwrap {
        return Err(validation_error(
            "full proof artifact required: stage0_bwrap_equals_stage2_bwrap must be true".to_string(),
        ));
    }
    if !manifest.fixed_point.stage0_busybox_equals_stage2_busybox {
        return Err(validation_error(
            "full proof artifact required: stage0_busybox_equals_stage2_busybox must be true".to_string(),
        ));
    }

    validate_stage_report(&manifest.stage0.report, &manifest.staged_source, "stage0.report")?;
    validate_stage_report(&manifest.stage2.report, &manifest.staged_source, "stage2.report")?;
    if manifest.stage2.report.output_binary != manifest.binaries.stage2.path {
        return Err(validation_error(
            "full proof artifact required: stage2 report output_binary must match binaries.stage2.path".to_string(),
        ));
    }
    Ok(())
}

fn validate_provider_kind(provider_kind: &str, field_name: &str) -> Result<(), ReleaseEvidenceError> {
    match provider_kind {
        "legacy-fetch" | "source-root" | "stagex-lineage" => Ok(()),
        "" => Err(validation_error(format!("{field_name} must not be empty"))),
        other => Err(validation_error(format!(
            "{field_name} must be one of legacy-fetch, source-root, stagex-lineage; got {other}"
        ))),
    }
}

fn validate_hashed_path(hashed: &SelfHostingProofHashedPath, field_name: &str) -> Result<(), ReleaseEvidenceError> {
    if hashed.path.trim().is_empty() {
        return Err(validation_error(format!("full proof artifact required: {field_name}.path is missing")));
    }
    if hashed.size_bytes == 0 {
        return Err(validation_error(format!(
            "full proof artifact required: {field_name}.size_bytes must be non-zero"
        )));
    }
    validate_blake3_hex(&hashed.digest_blake3, &format!("{field_name}.digest_blake3"))
}

fn validate_stage_report(
    report: &SelfHostingProofReportView,
    expected_staged_source: &str,
    field_name: &str,
) -> Result<(), ReleaseEvidenceError> {
    if report.staged_source != expected_staged_source {
        return Err(validation_error(format!(
            "full proof artifact required: {field_name}.staged_source does not match top-level staged_source"
        )));
    }
    if report.output_binary.trim().is_empty() {
        return Err(validation_error(format!("full proof artifact required: {field_name}.output_binary is missing")));
    }
    if report.busybox_path.is_none() {
        return Err(validation_error(format!("full proof artifact required: {field_name}.busybox_path is missing")));
    }
    Ok(())
}

pub(crate) fn validate_relative_member_path(path: &str, field_name: &str) -> Result<(), ReleaseEvidenceError> {
    if path.trim().is_empty() {
        return Err(validation_error(format!("release evidence {field_name} must not be empty")));
    }
    if path.starts_with('/') {
        return Err(validation_error(format!(
            "release evidence {field_name} must be relative, got absolute path {path}"
        )));
    }
    if path.split('/').any(|component| component == "..") {
        return Err(validation_error(format!("release evidence {field_name} must not escape the bundle root: {path}")));
    }
    let path_len_bytes = u32_count(path.len(), &format!("release evidence {field_name} length overflowed u32"))?;
    if path_len_bytes > MAX_RELATIVE_PATH_BYTES_COUNT {
        return Err(validation_error(format!(
            "release evidence {field_name} exceeds {MAX_RELATIVE_PATH_BYTES_COUNT} bytes"
        )));
    }
    Ok(())
}

pub(crate) fn validate_blake3_hex(digest_hex: &str, field_name: &str) -> Result<(), ReleaseEvidenceError> {
    if digest_hex.len() != BLAKE3_HEX_LENGTH_CHARS {
        return Err(validation_error(format!(
            "{field_name} must be {BLAKE3_HEX_LENGTH_CHARS} lowercase hex chars, got {}",
            digest_hex.len()
        )));
    }
    if !digest_hex.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()) {
        return Err(validation_error(format!("{field_name} must contain lowercase hex only")));
    }
    Ok(())
}

fn parse_error(message: String) -> ReleaseEvidenceError {
    ReleaseEvidenceError::Parse(message)
}

pub(crate) fn validation_error(message: String) -> ReleaseEvidenceError {
    ReleaseEvidenceError::Validation(message)
}

pub(crate) fn u32_count(count: usize, overflow_message: &str) -> Result<u32, ReleaseEvidenceError> {
    u32::try_from(count).map_err(|_| validation_error(overflow_message.to_string()))
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use pretty_assertions::assert_eq;
    use serde_json::json;

    use super::*;
    use crate::opaque_evidence::MAX_TRELLIS_PROOF_PRESERVES_SIDECAR_BYTES;
    use crate::opaque_evidence::TRELLIS_PROOF_KAMACITE_ROLE_FORMAL_PROOF_CANDIDATE;
    use crate::opaque_evidence::TRELLIS_PROOF_VALENCE_ROLE_RECORDED_ONLY;
    use crate::trellis_proof_binding::TRELLIS_PROOF_ACCEPTANCE_AUTHORITY_BLOCKER;
    use crate::trellis_proof_binding::TRELLIS_PROOF_DISPOSITION_ABSENT;
    use crate::trellis_proof_binding::TRELLIS_PROOF_DISPOSITION_INVALID;
    use crate::trellis_proof_binding::TRELLIS_PROOF_DISPOSITION_RECORDED_ONLY;
    use crate::trellis_proof_binding::TRELLIS_PROOF_MODE_OPTIONAL;
    use crate::trellis_proof_binding::TRELLIS_PROOF_MODE_REQUIRED;
    use crate::trellis_proof_binding::TrellisProofArtifactObservations;
    use crate::trellis_proof_binding::evaluate_trellis_proof_release_evidence;

    const SOURCE_ARCHIVE_DIGEST_SEED: u8 = 1;
    const RELEASE_BINARY_DIGEST_SEED: u8 = 3;
    const FUNCTION_ADDRESS_SIDECAR_DIGEST_SEED: u8 = 31;
    const FUNCTION_ADDRESS_VALENCE_DIGEST_SEED: u8 = 32;
    const FUNCTION_ADDRESS_KAMACITE_DIGEST_SEED: u8 = 33;
    const FUNCTION_ADDRESS_STALE_DIGEST_SEED: u8 = 34;
    const FUNCTION_ADDRESS_UPSTREAM_POLICY_DIGEST_SEED: u8 = 35;
    const FUNCTION_ADDRESS_MANTLE_POLICY_DIGEST_SEED: u8 = 36;
    const FUNCTION_ADDRESS_PRESERVES_DIGEST_SEED: u8 = 41;
    const FUNCTION_ADDRESS_JSON_PROJECTION_DIGEST_SEED: u8 = 42;
    const FUNCTION_ADDRESS_VALENCE_LOGICAL_HASH_SEED: u8 = 43;
    const FUNCTION_ADDRESS_PRESERVES_SIZE_BYTES: u64 = 256;
    const LIFECYCLE_ENVELOPE_DIGEST_SEED: u8 = 37;
    const LIFECYCLE_VALIDATION_DIGEST_SEED: u8 = 38;
    const LIFECYCLE_UPSTREAM_POLICY_DIGEST_SEED: u8 = 39;
    const LIFECYCLE_MANTLE_POLICY_DIGEST_SEED: u8 = 40;
    const DOMAIN_SEPARATION_ENVELOPE_DIGEST_SEED: u8 = 41;
    const DOMAIN_SEPARATION_VALIDATION_DIGEST_SEED: u8 = 42;
    const DOMAIN_SEPARATION_SOURCE_DIGEST_SEED: u8 = 43;
    const DOMAIN_SEPARATION_BINARY_DIGEST_SEED: u8 = 44;
    const DOMAIN_SEPARATION_POLICY_DIGEST_SEED: u8 = 45;
    const SECOND_LIFECYCLE_ENVELOPE_DIGEST_SEED: u8 = 46;
    const SECOND_LIFECYCLE_VALIDATION_DIGEST_SEED: u8 = 47;
    const TRELLIS_STALE_DIGEST_SEED: u8 = 14;

    type ManifestMutator = fn(&mut ReleaseEvidenceManifest);
    type ManifestFixtureCase = (&'static str, ManifestMutator, &'static str);

    fn sample_digest(seed: u8) -> String {
        let byte = format!("{:x}", seed % 16);
        byte.repeat(BLAKE3_HEX_LENGTH_CHARS)
    }

    fn sample_artifact(kind: BundledArtifactKind, relative_path: &str, seed: u8) -> BundledArtifact {
        BundledArtifact {
            kind,
            relative_path: relative_path.to_string(),
            size_bytes: 123,
            digest_blake3: sample_digest(seed),
        }
    }

    fn sample_provider_fixed_point_artifact(seed: u8) -> ProviderFixedPointProofArtifact {
        ProviderFixedPointProofArtifact {
            kind: BundledArtifactKind::Directory,
            relative_path: "proof/provider-fixed-point".to_string(),
            size_bytes: 123,
            digest_blake3: sample_digest(seed),
            evidence_role: PROVIDER_FIXED_POINT_PROOF_EVIDENCE_ROLE.to_string(),
        }
    }

    fn sample_role_bounded_artifact(relative_path: &str, role: &str, seed: u8) -> RoleBoundedReleaseArtifact {
        RoleBoundedReleaseArtifact {
            kind: BundledArtifactKind::File,
            relative_path: relative_path.to_string(),
            size_bytes: 123,
            digest_blake3: sample_digest(seed),
            evidence_role: role.to_string(),
        }
    }

    fn sample_external_evidence() -> ExternalEvidence {
        ExternalEvidence {
            role: "stack-provenance-trace".to_string(),
            schema: "valence.stack-provenance-adapter.v1".to_string(),
            relative_path: "external-evidence/stack-provenance.json".to_string(),
            digest_blake3: sample_digest(12),
            claim_scope: "identity-linkage-sidecar".to_string(),
            non_claims: vec!["not semantic validation by Mantle".to_string()],
        }
    }

    fn sample_kani_non_claims() -> Vec<String> {
        KANI_REQUIRED_NON_CLAIMS.iter().map(ToString::to_string).collect()
    }

    fn sample_kani_external_evidence() -> ExternalEvidence {
        ExternalEvidence {
            role: KANI_RECEIPT_EVIDENCE_ROLE.to_string(),
            schema: "cairn.kani_receipt.v1".to_string(),
            relative_path: "external-evidence/01-kani-receipt.json".to_string(),
            digest_blake3: sample_digest(13),
            claim_scope: KANI_EVIDENCE_CLAIM_SCOPE.to_string(),
            non_claims: sample_kani_non_claims(),
        }
    }

    fn sample_kani_toolchain_evidence() -> KaniToolchainEvidence {
        let external = sample_kani_external_evidence();
        KaniToolchainEvidence {
            schema: KANI_TOOLCHAIN_EVIDENCE_SCHEMA.to_string(),
            receipt_role: external.role,
            receipt_relative_path: external.relative_path,
            receipt_digest_blake3: external.digest_blake3,
            kani_version: "kani 0.63.0".to_string(),
            rust_toolchain: "rustc 1.91.0-nightly".to_string(),
            cbmc_version: "cbmc 6.4.0".to_string(),
            solver: KaniSolverIdentity {
                kind: KANI_SOLVER_KIND_CBMC_DEFAULT.to_string(),
                version: "cbmc-default".to_string(),
            },
            invocation_wrapper: "cargo kani --harness checked_add".to_string(),
            closure_identity_blake3: sample_digest(14),
            expected_closure_identity_blake3: sample_digest(14),
            valence_semantic_role: KANI_VALENCE_SEMANTIC_ROLE.to_string(),
            claim_scope: KANI_EVIDENCE_CLAIM_SCOPE.to_string(),
            non_claims: sample_kani_non_claims(),
        }
    }

    fn sample_stack_provenance_sidecar_evidence() -> ExternalEvidence {
        ExternalEvidence {
            role: STACK_PROVENANCE_EVIDENCE_ROLE.to_string(),
            schema: STACK_PROVENANCE_SIDECAR_SCHEMA.to_string(),
            relative_path: "external-evidence/01-stack-provenance-sidecar.json".to_string(),
            digest_blake3: sample_digest(21),
            claim_scope: STACK_PROVENANCE_CLAIM_SCOPE.to_string(),
            non_claims: vec![STACK_PROVENANCE_OPAQUE_BOUNDARY.to_string()],
        }
    }

    fn sample_stack_provenance_valence_receipt_evidence() -> ExternalEvidence {
        ExternalEvidence {
            role: VALENCE_STACK_PROVENANCE_RECEIPT_ROLE.to_string(),
            schema: STACK_PROVENANCE_GRAPH_REPORT_SCHEMA.to_string(),
            relative_path: "external-evidence/02-valence-stack-provenance-graph-report.json".to_string(),
            digest_blake3: sample_digest(22),
            claim_scope: STACK_PROVENANCE_CLAIM_SCOPE.to_string(),
            non_claims: vec![STACK_PROVENANCE_OPAQUE_BOUNDARY.to_string()],
        }
    }

    fn sample_stack_provenance_release_evidence() -> StackProvenanceReleaseEvidence {
        let sidecar = sample_stack_provenance_sidecar_evidence();
        let receipt = sample_stack_provenance_valence_receipt_evidence();
        StackProvenanceReleaseEvidence {
            sidecar_role: sidecar.role,
            sidecar_relative_path: sidecar.relative_path,
            sidecar_digest_blake3: sidecar.digest_blake3,
            sidecar_schema: sidecar.schema,
            valence_receipt_role: receipt.role,
            valence_receipt_relative_path: receipt.relative_path,
            valence_receipt_digest_blake3: receipt.digest_blake3,
            valence_receipt_schema: receipt.schema,
            release_binary_relative_path: "binaries/01-mantle".to_string(),
            release_binary_digest_blake3: sample_digest(3),
            sidecar_claim_scope: STACK_PROVENANCE_CLAIM_SCOPE.to_string(),
            non_claims: vec![STACK_PROVENANCE_OPAQUE_BOUNDARY.to_string()],
        }
    }

    fn sample_stack_provenance_manifest() -> ReleaseEvidenceManifest {
        let mut manifest = sample_manifest();
        manifest.external_evidence = vec![
            sample_stack_provenance_sidecar_evidence(),
            sample_stack_provenance_valence_receipt_evidence(),
        ];
        manifest.stack_provenance = Some(sample_stack_provenance_release_evidence());
        manifest
    }

    fn sample_function_address_non_claims() -> Vec<String> {
        vec![
            OPAQUE_EVIDENCE_REQUIRED_NON_CLAIM.to_string(),
            FUNCTION_ADDRESS_OPAQUE_BOUNDARY.to_string(),
        ]
    }

    fn sample_function_address_sidecar_evidence() -> ExternalEvidence {
        ExternalEvidence {
            role: FUNCTION_ADDRESS_EVIDENCE_ROLE.to_string(),
            schema: FUNCTION_ADDRESS_EVIDENCE_SCHEMA.to_string(),
            relative_path: "external-evidence/03-function-address-sidecar.json".to_string(),
            digest_blake3: sample_digest(FUNCTION_ADDRESS_SIDECAR_DIGEST_SEED),
            claim_scope: FUNCTION_ADDRESS_CLAIM_SCOPE.to_string(),
            non_claims: sample_function_address_non_claims(),
        }
    }

    fn sample_function_address_valence_receipt_evidence() -> ExternalEvidence {
        ExternalEvidence {
            role: VALENCE_FUNCTION_ADDRESS_RECEIPT_ROLE.to_string(),
            schema: VALENCE_FUNCTION_ADDRESS_RECEIPT_SCHEMA.to_string(),
            relative_path: "external-evidence/04-valence-function-address-profile.json".to_string(),
            digest_blake3: sample_digest(FUNCTION_ADDRESS_VALENCE_DIGEST_SEED),
            claim_scope: FUNCTION_ADDRESS_CLAIM_SCOPE.to_string(),
            non_claims: sample_function_address_non_claims(),
        }
    }

    fn sample_function_address_kamacite_receipt_evidence() -> ExternalEvidence {
        ExternalEvidence {
            role: KAMACITE_FUNCTION_ADDRESS_RECEIPT_ROLE.to_string(),
            schema: KAMACITE_FUNCTION_ADDRESS_RECEIPT_SCHEMA.to_string(),
            relative_path: "external-evidence/05-kamacite-function-address-receipt.preserves".to_string(),
            digest_blake3: sample_digest(FUNCTION_ADDRESS_KAMACITE_DIGEST_SEED),
            claim_scope: FUNCTION_ADDRESS_CLAIM_SCOPE.to_string(),
            non_claims: sample_function_address_non_claims(),
        }
    }

    fn sample_function_address_binding(with_kamacite: bool) -> OpaqueEvidenceSidecarBinding {
        let sidecar = sample_function_address_sidecar_evidence();
        let valence = sample_function_address_valence_receipt_evidence();
        let compatibility_projections = if with_kamacite {
            let kamacite = sample_function_address_kamacite_receipt_evidence();
            vec![OpaqueEvidenceCompatibilityProjection {
                role: kamacite.role,
                schema: kamacite.schema,
                relative_path: kamacite.relative_path,
                digest_blake3: kamacite.digest_blake3,
                canonical_envelope_digest_blake3: sidecar.digest_blake3.clone(),
            }]
        } else {
            vec![]
        };
        OpaqueEvidenceSidecarBinding {
            schema: OPAQUE_EVIDENCE_SIDECAR_BINDING_SCHEMA.to_string(),
            evidence_kind: OPAQUE_EVIDENCE_KIND_FUNCTION_ADDRESS.to_string(),
            profile_version: FUNCTION_ADDRESS_PROFILE_VERSION.to_string(),
            profile_roles: None,
            canonical_envelope: OpaqueEvidenceCanonicalEnvelopeLink {
                role: sidecar.role,
                schema: sidecar.schema,
                relative_path: sidecar.relative_path,
                digest_blake3: sidecar.digest_blake3,
                size_bytes: None,
            },
            upstream_validation: OpaqueEvidenceUpstreamValidationLink {
                role: valence.role,
                schema: valence.schema,
                relative_path: valence.relative_path,
                digest_blake3: valence.digest_blake3,
                receipt_hash_blake3: None,
            },
            source_artifact: OpaqueEvidenceSourceArtifactLink {
                relative_path: "source/mantle-src.tar".to_string(),
                digest_blake3: sample_digest(SOURCE_ARCHIVE_DIGEST_SEED),
            },
            release_binary: OpaqueEvidenceReleaseBinaryLink {
                relative_path: "binaries/01-mantle".to_string(),
                digest_blake3: sample_digest(RELEASE_BINARY_DIGEST_SEED),
            },
            policy_hashes: vec![
                OpaqueEvidencePolicyHash {
                    policy_kind: OPAQUE_EVIDENCE_POLICY_KIND_UPSTREAM_PROFILE.to_string(),
                    digest_blake3: sample_digest(FUNCTION_ADDRESS_UPSTREAM_POLICY_DIGEST_SEED),
                },
                OpaqueEvidencePolicyHash {
                    policy_kind: OPAQUE_EVIDENCE_POLICY_KIND_MANTLE_RELEASE.to_string(),
                    digest_blake3: sample_digest(FUNCTION_ADDRESS_MANTLE_POLICY_DIGEST_SEED),
                },
            ],
            claim_scope: FUNCTION_ADDRESS_CLAIM_SCOPE.to_string(),
            compatibility_projections,
            non_claims: sample_function_address_non_claims(),
        }
    }

    fn sample_legacy_function_address_release_evidence(with_kamacite: bool) -> FunctionAddressReleaseEvidence {
        let sidecar = sample_function_address_sidecar_evidence();
        let valence = sample_function_address_valence_receipt_evidence();
        let kamacite = with_kamacite.then(sample_function_address_kamacite_receipt_evidence);
        FunctionAddressReleaseEvidence {
            sidecar_role: sidecar.role,
            sidecar_schema: sidecar.schema,
            sidecar_claim_scope: sidecar.claim_scope,
            sidecar_relative_path: sidecar.relative_path,
            sidecar_digest_blake3: sidecar.digest_blake3,
            valence_receipt_role: valence.role,
            valence_receipt_schema: valence.schema,
            valence_receipt_relative_path: valence.relative_path,
            valence_receipt_digest_blake3: valence.digest_blake3,
            kamacite_receipt_role: kamacite.as_ref().map(|evidence| evidence.role.clone()),
            kamacite_receipt_schema: kamacite.as_ref().map(|evidence| evidence.schema.clone()),
            kamacite_receipt_relative_path: kamacite.as_ref().map(|evidence| evidence.relative_path.clone()),
            kamacite_receipt_digest_blake3: kamacite.as_ref().map(|evidence| evidence.digest_blake3.clone()),
            source_archive_digest_blake3: sample_digest(SOURCE_ARCHIVE_DIGEST_SEED),
            release_binary_relative_path: "binaries/01-mantle".to_string(),
            release_binary_digest_blake3: sample_digest(RELEASE_BINARY_DIGEST_SEED),
            non_claims: sample_function_address_non_claims(),
        }
    }

    fn sample_function_address_manifest(with_kamacite: bool) -> ReleaseEvidenceManifest {
        let mut manifest = sample_manifest();
        manifest.external_evidence = vec![
            sample_function_address_sidecar_evidence(),
            sample_function_address_valence_receipt_evidence(),
        ];
        if with_kamacite {
            manifest.external_evidence.push(sample_function_address_kamacite_receipt_evidence());
        }
        manifest.opaque_evidence_sidecar_bindings =
            vec![opaque_evidence_sidecar_binding_receipt(sample_function_address_binding(with_kamacite)).unwrap()];
        manifest
    }

    fn sample_preserves_sidecar_evidence() -> ExternalEvidence {
        ExternalEvidence {
            role: KAMACITE_FUNCTION_ADDRESS_PRESERVES_ROLE.to_string(),
            schema: KAMACITE_FUNCTION_ADDRESS_PRESERVES_SCHEMA.to_string(),
            relative_path: "external-evidence/05-kamacite-function-address-receipt.preserves".to_string(),
            digest_blake3: sample_digest(FUNCTION_ADDRESS_PRESERVES_DIGEST_SEED),
            claim_scope: FUNCTION_ADDRESS_CLAIM_SCOPE.to_string(),
            non_claims: sample_function_address_non_claims(),
        }
    }

    fn sample_preserves_json_projection_evidence() -> ExternalEvidence {
        ExternalEvidence {
            role: KAMACITE_FUNCTION_ADDRESS_JSON_PROJECTION_ROLE.to_string(),
            schema: KAMACITE_FUNCTION_ADDRESS_JSON_PROJECTION_SCHEMA.to_string(),
            relative_path: "external-evidence/06-kamacite-function-address-projection.json".to_string(),
            digest_blake3: sample_digest(FUNCTION_ADDRESS_JSON_PROJECTION_DIGEST_SEED),
            claim_scope: FUNCTION_ADDRESS_CLAIM_SCOPE.to_string(),
            non_claims: sample_function_address_non_claims(),
        }
    }

    fn sample_preserves_function_address_binding(with_json_projection: bool) -> OpaqueEvidenceSidecarBinding {
        let preserves = sample_preserves_sidecar_evidence();
        let valence = sample_function_address_valence_receipt_evidence();
        assert_eq!(preserves.role, KAMACITE_FUNCTION_ADDRESS_PRESERVES_ROLE);
        assert_eq!(valence.role, VALENCE_FUNCTION_ADDRESS_RECEIPT_ROLE);
        let compatibility_projections = with_json_projection
            .then(|| {
                let projection = sample_preserves_json_projection_evidence();
                OpaqueEvidenceCompatibilityProjection {
                    role: projection.role,
                    schema: projection.schema,
                    relative_path: projection.relative_path,
                    digest_blake3: projection.digest_blake3,
                    canonical_envelope_digest_blake3: preserves.digest_blake3.clone(),
                }
            })
            .into_iter()
            .collect();
        OpaqueEvidenceSidecarBinding {
            schema: OPAQUE_EVIDENCE_SIDECAR_BINDING_SCHEMA.to_string(),
            evidence_kind: OPAQUE_EVIDENCE_KIND_FUNCTION_ADDRESS.to_string(),
            profile_version: FUNCTION_ADDRESS_PRESERVES_PROFILE_VERSION.to_string(),
            profile_roles: None,
            canonical_envelope: OpaqueEvidenceCanonicalEnvelopeLink {
                role: preserves.role,
                schema: preserves.schema,
                relative_path: preserves.relative_path,
                digest_blake3: preserves.digest_blake3,
                size_bytes: Some(FUNCTION_ADDRESS_PRESERVES_SIZE_BYTES),
            },
            upstream_validation: OpaqueEvidenceUpstreamValidationLink {
                role: valence.role,
                schema: valence.schema,
                relative_path: valence.relative_path,
                digest_blake3: valence.digest_blake3,
                receipt_hash_blake3: Some(sample_digest(FUNCTION_ADDRESS_VALENCE_LOGICAL_HASH_SEED)),
            },
            source_artifact: OpaqueEvidenceSourceArtifactLink {
                relative_path: "source/mantle-src.tar".to_string(),
                digest_blake3: sample_digest(SOURCE_ARCHIVE_DIGEST_SEED),
            },
            release_binary: OpaqueEvidenceReleaseBinaryLink {
                relative_path: "binaries/01-mantle".to_string(),
                digest_blake3: sample_digest(RELEASE_BINARY_DIGEST_SEED),
            },
            policy_hashes: vec![
                OpaqueEvidencePolicyHash {
                    policy_kind: OPAQUE_EVIDENCE_POLICY_KIND_UPSTREAM_PROFILE.to_string(),
                    digest_blake3: sample_digest(FUNCTION_ADDRESS_UPSTREAM_POLICY_DIGEST_SEED),
                },
                OpaqueEvidencePolicyHash {
                    policy_kind: OPAQUE_EVIDENCE_POLICY_KIND_MANTLE_RELEASE.to_string(),
                    digest_blake3: sample_digest(FUNCTION_ADDRESS_MANTLE_POLICY_DIGEST_SEED),
                },
            ],
            claim_scope: FUNCTION_ADDRESS_CLAIM_SCOPE.to_string(),
            compatibility_projections,
            non_claims: sample_function_address_non_claims(),
        }
    }

    fn sample_preserves_function_address_manifest(with_json_projection: bool) -> ReleaseEvidenceManifest {
        let mut manifest = sample_manifest();
        manifest.external_evidence = vec![
            sample_preserves_sidecar_evidence(),
            sample_function_address_valence_receipt_evidence(),
        ];
        if with_json_projection {
            manifest.external_evidence.push(sample_preserves_json_projection_evidence());
        }
        manifest.opaque_evidence_sidecar_bindings = vec![
            opaque_evidence_sidecar_binding_receipt(sample_preserves_function_address_binding(with_json_projection))
                .unwrap(),
        ];
        manifest
    }

    fn sample_legacy_function_address_manifest(with_kamacite: bool) -> ReleaseEvidenceManifest {
        let mut manifest = sample_manifest();
        manifest.external_evidence = vec![
            sample_function_address_sidecar_evidence(),
            sample_function_address_valence_receipt_evidence(),
        ];
        if with_kamacite {
            manifest.external_evidence.push(sample_function_address_kamacite_receipt_evidence());
        }
        manifest.function_address_evidence = Some(sample_legacy_function_address_release_evidence(with_kamacite));
        manifest
    }

    #[derive(Debug, Deserialize)]
    struct TrellisProofNegativeCase {
        name: String,
        expected: String,
    }

    fn sample_trellis_proof_binding() -> OpaqueEvidenceSidecarBinding {
        serde_json::from_str(include_str!(
            "../../../tests/fixtures/trellis-proof-release-sidecars/binding.optional-recorded-only.valid.json"
        ))
        .expect("parse registered Trellis proof binding fixture")
    }

    fn sample_trellis_proof_manifest() -> ReleaseEvidenceManifest {
        let mut manifest = sample_manifest();
        let binding = sample_trellis_proof_binding();
        manifest.external_evidence = trellis_external_evidence(&binding);
        manifest.opaque_evidence_sidecar_bindings =
            vec![opaque_evidence_sidecar_binding_receipt(binding).expect("valid Trellis proof binding")];
        manifest
    }

    fn trellis_external_evidence(binding: &OpaqueEvidenceSidecarBinding) -> Vec<ExternalEvidence> {
        let mut evidence = vec![
            trellis_external_row(
                &binding.canonical_envelope.role,
                &binding.canonical_envelope.schema,
                &binding.canonical_envelope.relative_path,
                &binding.canonical_envelope.digest_blake3,
                binding,
            ),
            trellis_external_row(
                &binding.upstream_validation.role,
                &binding.upstream_validation.schema,
                &binding.upstream_validation.relative_path,
                &binding.upstream_validation.digest_blake3,
                binding,
            ),
        ];
        evidence.extend(binding.compatibility_projections.iter().map(|projection| {
            trellis_external_row(
                &projection.role,
                &projection.schema,
                &projection.relative_path,
                &projection.digest_blake3,
                binding,
            )
        }));
        evidence
    }

    fn trellis_external_row(
        role: &str,
        schema: &str,
        relative_path: &str,
        digest_blake3: &str,
        binding: &OpaqueEvidenceSidecarBinding,
    ) -> ExternalEvidence {
        ExternalEvidence {
            role: role.to_string(),
            schema: schema.to_string(),
            relative_path: relative_path.to_string(),
            digest_blake3: digest_blake3.to_string(),
            claim_scope: binding.claim_scope.clone(),
            non_claims: binding.non_claims.clone(),
        }
    }

    fn sample_trellis_observations(binding: &OpaqueEvidenceSidecarBinding) -> TrellisProofArtifactObservations {
        let projection = binding.compatibility_projections.first();
        TrellisProofArtifactObservations {
            canonical_envelope_size_bytes: binding.canonical_envelope.size_bytes.expect("fixture has bounded size"),
            canonical_envelope_digest_blake3: binding.canonical_envelope.digest_blake3.clone(),
            valence_artifact_digest_blake3: binding.upstream_validation.digest_blake3.clone(),
            valence_receipt_hash_blake3: binding
                .upstream_validation
                .receipt_hash_blake3
                .clone()
                .expect("fixture has Valence logical identity"),
            projection_artifact_digest_blake3: projection.map(|value| value.digest_blake3.clone()),
            projection_canonical_envelope_digest_blake3: projection
                .map(|value| value.canonical_envelope_digest_blake3.clone()),
        }
    }

    fn trellis_negative_cases() -> Vec<TrellisProofNegativeCase> {
        serde_json::from_str(include_str!("../../../tests/fixtures/trellis-proof-release-sidecars/negative-cases.json"))
            .expect("parse Trellis proof negative cases")
    }

    fn sample_lifecycle_opaque_binding_manifest() -> ReleaseEvidenceManifest {
        let mut manifest = sample_manifest();
        let non_claims = vec![OPAQUE_EVIDENCE_REQUIRED_NON_CLAIM.to_string()];
        let envelope = ExternalEvidence {
            role: "cairn-lifecycle-evidence-envelope".to_string(),
            schema: "cairn.lifecycle-evidence.v1".to_string(),
            relative_path: "external-evidence/06-cairn-lifecycle-evidence.bin".to_string(),
            digest_blake3: sample_digest(LIFECYCLE_ENVELOPE_DIGEST_SEED),
            claim_scope: OPAQUE_EVIDENCE_GENERIC_CLAIM_SCOPE.to_string(),
            non_claims: non_claims.clone(),
        };
        let validation = ExternalEvidence {
            role: "valence-lifecycle-evidence-validation".to_string(),
            schema: "valence.lifecycle-evidence-validation.v1".to_string(),
            relative_path: "external-evidence/07-valence-lifecycle-validation.bin".to_string(),
            digest_blake3: sample_digest(LIFECYCLE_VALIDATION_DIGEST_SEED),
            claim_scope: OPAQUE_EVIDENCE_GENERIC_CLAIM_SCOPE.to_string(),
            non_claims: non_claims.clone(),
        };
        let binding = OpaqueEvidenceSidecarBinding {
            schema: OPAQUE_EVIDENCE_SIDECAR_BINDING_SCHEMA.to_string(),
            evidence_kind: OPAQUE_EVIDENCE_KIND_LIFECYCLE.to_string(),
            profile_version: "lifecycle-evidence-v1".to_string(),
            profile_roles: None,
            canonical_envelope: OpaqueEvidenceCanonicalEnvelopeLink {
                role: envelope.role.clone(),
                schema: envelope.schema.clone(),
                relative_path: envelope.relative_path.clone(),
                digest_blake3: envelope.digest_blake3.clone(),
                size_bytes: None,
            },
            upstream_validation: OpaqueEvidenceUpstreamValidationLink {
                role: validation.role.clone(),
                schema: validation.schema.clone(),
                relative_path: validation.relative_path.clone(),
                digest_blake3: validation.digest_blake3.clone(),
                receipt_hash_blake3: None,
            },
            source_artifact: OpaqueEvidenceSourceArtifactLink {
                relative_path: manifest.source_archive.relative_path.clone(),
                digest_blake3: manifest.source_archive.digest_blake3.clone(),
            },
            release_binary: OpaqueEvidenceReleaseBinaryLink {
                relative_path: manifest.binaries[0].relative_path.clone(),
                digest_blake3: manifest.binaries[0].digest_blake3.clone(),
            },
            policy_hashes: vec![
                OpaqueEvidencePolicyHash {
                    policy_kind: OPAQUE_EVIDENCE_POLICY_KIND_UPSTREAM_PROFILE.to_string(),
                    digest_blake3: sample_digest(LIFECYCLE_UPSTREAM_POLICY_DIGEST_SEED),
                },
                OpaqueEvidencePolicyHash {
                    policy_kind: OPAQUE_EVIDENCE_POLICY_KIND_MANTLE_RELEASE.to_string(),
                    digest_blake3: sample_digest(LIFECYCLE_MANTLE_POLICY_DIGEST_SEED),
                },
            ],
            claim_scope: OPAQUE_EVIDENCE_GENERIC_CLAIM_SCOPE.to_string(),
            compatibility_projections: vec![],
            non_claims,
        };
        manifest.external_evidence = vec![envelope, validation];
        manifest.opaque_evidence_sidecar_bindings = vec![opaque_evidence_sidecar_binding_receipt(binding).unwrap()];
        manifest
    }

    fn sample_manifest() -> ReleaseEvidenceManifest {
        let stage2_binary = sample_artifact(BundledArtifactKind::File, "binaries/01-mantle", 3);
        let inventory = sample_artifact(BundledArtifactKind::File, "proof/inventory.md", 5);
        ReleaseEvidenceManifest {
            schema: RELEASE_EVIDENCE_SCHEMA.to_string(),
            release_id: "mantle-0.1.0-rc1".to_string(),
            claim_scope: CLAIM_SCOPE_PACKAGED_INTEGRITY.to_string(),
            workflow: ReleaseWorkflowIdentity {
                command: DEFAULT_PROOF_WORKFLOW_COMMAND.to_string(),
                version: DEFAULT_PROOF_WORKFLOW_VERSION.to_string(),
            },
            source_archive: sample_artifact(BundledArtifactKind::File, "source/mantle-src.tar", 1),
            source_acquisition: None,
            binaries: vec![stage2_binary.clone()],
            proof_bundle: sample_artifact(BundledArtifactKind::Directory, "proof/self-hosting", 7),
            prerequisite_inventory: inventory.clone(),
            provider_fixed_point_proof: None,
            reproducibility_report: None,
            deterministic_build_proof: None,
            deterministic_sandbox_isolation_evidence: None,
            independent_agreement_report: None,
            external_evidence: vec![],
            kani_toolchain_evidence: vec![],
            stack_provenance: None,
            opaque_evidence_sidecar_bindings: vec![],
            function_address_evidence: None,
            proof_linkage: ReleaseProofLinkage {
                release_id: "mantle-0.1.0-rc1".to_string(),
                source_archive_digest_blake3: sample_digest(1),
                proof_bundle_schema: FULL_SELF_HOSTING_PROOF_SCHEMA.to_string(),
                proof_mode: "fixed-point".to_string(),
                selected_provider_kind: "source-root".to_string(),
                staged_source: "/tmp/proof-store/abcd-mantle-src".to_string(),
                stage2_binary_digest_blake3: stage2_binary.digest_blake3,
                prerequisite_inventory_digest_blake3: inventory.digest_blake3,
                proof_manifest_digest_blake3: sample_digest(9),
            },
            provenance_coverage: None,
        }
    }

    fn sample_full_proof_manifest(inventory_digest: &str, stage2_digest: &str) -> Vec<u8> {
        serde_json::to_vec(&json!({
            "schema": FULL_SELF_HOSTING_PROOF_SCHEMA,
            "staged_source": "/tmp/proof-store/abcd-mantle-src",
            "prerequisites": {
                "mode": "fixed-point",
                "provider_kind": "source-root",
                "inventory_doc": {
                    "path": "/tmp/proof-bundle/stage0-prerequisites/inventory.md",
                    "size_bytes": 9,
                    "digest_blake3": inventory_digest
                }
            },
            "binaries": {
                "stage1": {
                    "path": "/tmp/proof-store/stage1-mantle/bin/mantle",
                    "size_bytes": 20,
                    "digest_blake3": sample_digest(10)
                },
                "stage2": {
                    "path": "/tmp/proof-store/stage2-mantle/bin/mantle",
                    "size_bytes": 13,
                    "digest_blake3": stage2_digest
                }
            },
            "tools": {
                "stage0_bwrap": {
                    "path": "/tmp/proof-store/stage0-bwrap/bin/bwrap",
                    "size_bytes": 22,
                    "digest_blake3": sample_digest(12)
                },
                "stage0_busybox": {
                    "path": "/tmp/proof-store/stage0-busybox/bin/busybox",
                    "size_bytes": 23,
                    "digest_blake3": sample_digest(13)
                },
                "stage2_bwrap": {
                    "path": "/tmp/proof-store/stage2-bwrap/bin/bwrap",
                    "size_bytes": 24,
                    "digest_blake3": sample_digest(14)
                },
                "stage2_busybox": {
                    "path": "/tmp/proof-store/stage2-busybox/bin/busybox",
                    "size_bytes": 25,
                    "digest_blake3": sample_digest(15)
                }
            },
            "fixed_point": {
                "stage1_equals_stage2": true,
                "stage0_bwrap_equals_stage2_bwrap": true,
                "stage0_busybox_equals_stage2_busybox": true
            },
            "stage0": {
                "report": {
                    "staged_source": "/tmp/proof-store/abcd-mantle-src",
                    "output_binary": "/tmp/proof-store/stage1-mantle/bin/mantle",
                    "busybox_path": "/tmp/proof-store/stage0-busybox/bin/busybox"
                }
            },
            "stage2": {
                "report": {
                    "staged_source": "/tmp/proof-store/abcd-mantle-src",
                    "output_binary": "/tmp/proof-store/stage2-mantle/bin/mantle",
                    "busybox_path": "/tmp/proof-store/stage2-busybox/bin/busybox"
                }
            }
        }))
        .unwrap()
    }

    #[test]
    fn canonical_bytes_are_stable_and_compact() {
        let manifest = sample_manifest();
        let first = canonical_release_evidence_manifest(manifest.clone()).unwrap();
        let second = canonical_release_evidence_manifest(manifest.clone()).unwrap();
        assert_eq!(first, second);
        assert!(!first.contains(&b'\n'));
    }

    #[test]
    fn validate_rejects_absolute_member_path() {
        let mut manifest = sample_manifest();
        manifest.source_archive.relative_path = "/tmp/source.tar".to_string();
        let err = canonical_release_evidence_manifest(manifest).unwrap_err();
        assert!(err.to_string().contains("must be relative"));
    }

    #[test]
    fn validate_accepts_matching_external_source_acquisition() {
        let mut manifest = sample_manifest();
        manifest.source_acquisition = Some(SourceAcquisition::external_archive(
            "https://example.invalid/mantle-src.tar".to_string(),
            manifest.source_archive.digest_blake3.clone(),
        ));

        let bytes = canonical_release_evidence_manifest(manifest).unwrap();
        let text = String::from_utf8(bytes).unwrap();

        assert!(text.contains("source_acquisition"));
        assert!(text.contains(SOURCE_ACQUISITION_KIND_EXTERNAL_ARCHIVE));
    }

    #[test]
    fn validate_rejects_source_acquisition_digest_mismatch() {
        let mut manifest = sample_manifest();
        manifest.source_acquisition = Some(SourceAcquisition::external_archive(
            "https://example.invalid/mantle-src.tar".to_string(),
            sample_digest(2),
        ));

        let err = canonical_release_evidence_manifest(manifest).unwrap_err();

        assert!(err.to_string().contains("source_acquisition.digest_blake3 must match"));
    }

    #[test]
    fn validate_rejects_unsupported_source_acquisition_url() {
        let mut manifest = sample_manifest();
        manifest.source_acquisition = Some(SourceAcquisition::external_archive(
            "git@example.invalid:mantle.git".to_string(),
            manifest.source_archive.digest_blake3.clone(),
        ));

        let err = canonical_release_evidence_manifest(manifest).unwrap_err();

        assert!(err.to_string().contains("must start with file://, http://, or https://"));
    }

    #[test]
    fn validate_accepts_git_source_acquisition() {
        let mut manifest = sample_manifest();
        manifest.source_acquisition = Some(SourceAcquisition::git(
            "file:///tmp/mantle-origin.git".to_string(),
            "a".repeat(GIT_SHA1_HEX_LENGTH_CHARS),
            Some("refs/heads/main".to_string()),
            Some("v0.1.0".to_string()),
            manifest.source_archive.digest_blake3.clone(),
        ));

        let bytes = canonical_release_evidence_manifest(manifest).unwrap();
        let text = String::from_utf8(bytes).unwrap();

        assert!(text.contains(SOURCE_ACQUISITION_KIND_GIT));
        assert!(text.contains(RELEASE_SOURCE_ARCHIVE_PROFILE));
        assert!(text.contains(RELEASE_SOURCE_ARCHIVE_VERSION));
    }

    #[test]
    fn validate_rejects_git_source_with_malformed_commit() {
        let mut manifest = sample_manifest();
        manifest.source_acquisition = Some(SourceAcquisition::git(
            "file:///tmp/mantle-origin.git".to_string(),
            "not-a-commit".to_string(),
            None,
            None,
            manifest.source_archive.digest_blake3.clone(),
        ));

        let err = canonical_release_evidence_manifest(manifest).unwrap_err();

        assert!(err.to_string().contains("source_acquisition.commit"));
    }

    #[test]
    fn validate_rejects_credential_bearing_git_source_url() {
        let mut manifest = sample_manifest();
        manifest.source_acquisition = Some(SourceAcquisition::git(
            "https://token@example.invalid/mantle.git".to_string(),
            "a".repeat(GIT_SHA1_HEX_LENGTH_CHARS),
            None,
            None,
            manifest.source_archive.digest_blake3.clone(),
        ));

        let err = canonical_release_evidence_manifest(manifest).unwrap_err();

        assert!(err.to_string().contains("must not contain credentials"));
    }

    #[test]
    fn validate_rejects_git_source_archive_digest_mismatch() {
        let mut manifest = sample_manifest();
        manifest.source_acquisition = Some(SourceAcquisition::git(
            "file:///tmp/mantle-origin.git".to_string(),
            "a".repeat(GIT_SHA1_HEX_LENGTH_CHARS),
            None,
            None,
            sample_digest(2),
        ));

        let err = canonical_release_evidence_manifest(manifest).unwrap_err();

        assert!(err.to_string().contains("source_acquisition.digest_blake3 must match"));
    }

    #[test]
    fn validate_accepts_provider_fixed_point_proof_artifact() {
        let mut manifest = sample_manifest();
        manifest.provider_fixed_point_proof = Some(sample_provider_fixed_point_artifact(8));

        let bytes = canonical_release_evidence_manifest(manifest).unwrap();
        let text = String::from_utf8(bytes).unwrap();

        assert!(text.contains("provider_fixed_point_proof"));
        assert!(text.contains(PROVIDER_FIXED_POINT_PROOF_EVIDENCE_ROLE));
    }

    #[test]
    fn validate_rejects_provider_fixed_point_proof_with_wrong_role() {
        let mut manifest = sample_manifest();
        let mut proof = sample_provider_fixed_point_artifact(8);
        proof.evidence_role = "release-reproducibility".to_string();
        manifest.provider_fixed_point_proof = Some(proof);

        let err = canonical_release_evidence_manifest(manifest).unwrap_err();

        assert!(err.to_string().contains("provider_fixed_point_proof.evidence_role"));
    }

    #[test]
    fn validate_rejects_provider_fixed_point_proof_file_kind() {
        let mut manifest = sample_manifest();
        let mut proof = sample_provider_fixed_point_artifact(8);
        proof.kind = BundledArtifactKind::File;
        manifest.provider_fixed_point_proof = Some(proof);

        let err = canonical_release_evidence_manifest(manifest).unwrap_err();

        assert!(err.to_string().contains("must be recorded as a directory artifact"));
    }

    #[test]
    fn provider_fixed_point_release_artifact_binding_accepts_matching_binary() {
        let manifest = sample_manifest();
        let stage_digest = manifest.binaries[0].digest_blake3.clone();

        let binding =
            validate_provider_fixed_point_release_artifact_binding(&manifest.binaries, &stage_digest).unwrap();

        assert_eq!(binding.relative_path, "binaries/01-mantle");
        assert_eq!(binding.digest_blake3, stage_digest);
    }

    #[test]
    fn provider_fixed_point_release_artifact_binding_rejects_mismatch() {
        let manifest = sample_manifest();
        let err =
            validate_provider_fixed_point_release_artifact_binding(&manifest.binaries, &sample_digest(12)).unwrap_err();

        assert!(err.to_string().contains("does not match any bundled release binary artifact"));
    }

    #[test]
    fn provider_fixed_point_release_artifact_binding_rejects_missing_binaries() {
        let err = validate_provider_fixed_point_release_artifact_binding(&[], &sample_digest(12)).unwrap_err();

        assert!(err.to_string().contains("at least one binary artifact"));
    }

    #[test]
    fn provider_fixed_point_release_artifact_binding_rejects_malformed_digest() {
        let manifest = sample_manifest();
        let err =
            validate_provider_fixed_point_release_artifact_binding(&manifest.binaries, "not-a-blake3").unwrap_err();

        assert!(err.to_string().contains("provider_fixed_point.stage_binary_digest_blake3"));
    }

    #[test]
    fn validate_accepts_deterministic_proof_artifacts() {
        let mut manifest = sample_manifest();
        manifest.deterministic_build_proof = Some(sample_role_bounded_artifact(
            "deterministic-release/deterministic-build-proof.json",
            DETERMINISTIC_BUILD_PROOF_EVIDENCE_ROLE,
            10,
        ));
        manifest.deterministic_sandbox_isolation_evidence = Some(sample_role_bounded_artifact(
            "deterministic-release/deterministic-sandbox-isolation-evidence.json",
            DETERMINISTIC_SANDBOX_ISOLATION_EVIDENCE_ROLE,
            11,
        ));

        let bytes = canonical_release_evidence_manifest(manifest).unwrap();
        let text = String::from_utf8(bytes).unwrap();

        assert!(text.contains("deterministic_build_proof"));
        assert!(text.contains(DETERMINISTIC_BUILD_PROOF_EVIDENCE_ROLE));
        assert!(text.contains("deterministic_sandbox_isolation_evidence"));
        assert!(text.contains(DETERMINISTIC_SANDBOX_ISOLATION_EVIDENCE_ROLE));
    }

    #[test]
    fn validate_rejects_deterministic_proof_with_wrong_role() {
        let mut manifest = sample_manifest();
        manifest.deterministic_build_proof = Some(sample_role_bounded_artifact(
            "deterministic-release/deterministic-build-proof.json",
            "wrong-role",
            10,
        ));

        let err = canonical_release_evidence_manifest(manifest).unwrap_err();

        assert!(err.to_string().contains("deterministic_build_proof.evidence_role"));
    }

    #[test]
    fn validate_rejects_deterministic_proof_with_wrong_path() {
        let mut manifest = sample_manifest();
        manifest.deterministic_sandbox_isolation_evidence = Some(sample_role_bounded_artifact(
            "deterministic-release/wrong.json",
            DETERMINISTIC_SANDBOX_ISOLATION_EVIDENCE_ROLE,
            11,
        ));

        let err = canonical_release_evidence_manifest(manifest).unwrap_err();

        assert!(err.to_string().contains("deterministic_sandbox_isolation_evidence"));
        assert!(err.to_string().contains("deterministic-release/deterministic-sandbox-isolation-evidence.json"));
    }

    #[test]
    fn validate_rejects_deterministic_proof_duplicate_path() {
        let mut manifest = sample_manifest();
        manifest.source_archive.relative_path = "deterministic-release/deterministic-build-proof.json".to_string();
        manifest.deterministic_build_proof = Some(sample_role_bounded_artifact(
            "deterministic-release/deterministic-build-proof.json",
            DETERMINISTIC_BUILD_PROOF_EVIDENCE_ROLE,
            10,
        ));

        let err = canonical_release_evidence_manifest(manifest).unwrap_err();

        assert!(err.to_string().contains("duplicate bundle member path"));
    }

    #[test]
    fn validate_rejects_prerequisite_inventory_linkage_mismatch() {
        let mut manifest = sample_manifest();
        manifest.proof_linkage.prerequisite_inventory_digest_blake3 = sample_digest(8);
        let err = canonical_release_evidence_manifest(manifest).unwrap_err();
        assert!(err.to_string().contains("prerequisite inventory digest does not match"));
    }

    #[test]
    fn validate_rejects_stage2_digest_not_present_in_binaries() {
        let mut manifest = sample_manifest();
        manifest.proof_linkage.stage2_binary_digest_blake3 = sample_digest(4);
        let err = canonical_release_evidence_manifest(manifest).unwrap_err();
        assert!(err.to_string().contains("does not match any bundled binary artifact"));
    }

    #[test]
    fn extract_full_proof_identity_fields_accepts_valid_manifest() {
        let manifest_bytes = sample_full_proof_manifest(&sample_digest(9), &sample_digest(11));
        let identity = extract_full_self_hosting_proof_identity_fields(manifest_bytes).unwrap();
        assert_eq!(identity.schema, FULL_SELF_HOSTING_PROOF_SCHEMA);
        assert_eq!(identity.proof_mode, "fixed-point");
        assert_eq!(identity.selected_provider_kind, "source-root");
        assert_eq!(identity.staged_source, "/tmp/proof-store/abcd-mantle-src");
        assert_eq!(identity.stage2_binary_digest_blake3, sample_digest(11));
        assert_eq!(identity.prerequisite_inventory_digest_blake3, sample_digest(9));
    }

    #[test]
    fn extract_full_proof_identity_fields_rejects_missing_provider_kind() {
        let mut manifest: serde_json::Value =
            serde_json::from_slice(&sample_full_proof_manifest(&sample_digest(9), &sample_digest(11))).unwrap();
        manifest["prerequisites"].as_object_mut().unwrap().remove("provider_kind");
        let err = extract_full_self_hosting_proof_identity_fields(serde_json::to_vec(&manifest).unwrap()).unwrap_err();
        assert!(err.to_string().contains("provider_kind"));
    }

    #[test]
    fn release_manifest_rejects_unknown_selected_provider_kind() {
        let mut manifest = sample_manifest();
        manifest.proof_linkage.selected_provider_kind = "mystery".to_string();
        let err = canonical_release_evidence_manifest(manifest).unwrap_err();
        assert!(err.to_string().contains("selected_provider_kind"));
    }

    #[test]
    fn extract_full_proof_identity_fields_rejects_wrong_schema() {
        let manifest_bytes = br#"{"schema":"fake-proof"}"#.to_vec();
        let err = extract_full_self_hosting_proof_identity_fields(manifest_bytes).unwrap_err();
        assert!(err.to_string().contains("full proof artifact required"));
    }

    #[test]
    fn validate_accepts_manifest_without_provenance_coverage() {
        let manifest = sample_manifest();
        assert!(manifest.provenance_coverage.is_none());
        let bytes = canonical_release_evidence_manifest(manifest).unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn validate_accepts_manifest_with_valid_external_evidence() {
        let mut manifest = sample_manifest();
        manifest.external_evidence = vec![sample_external_evidence()];
        let bytes = canonical_release_evidence_manifest(manifest).unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn validate_rejects_external_evidence_path_escape() {
        let mut manifest = sample_manifest();
        let mut evidence = sample_external_evidence();
        evidence.relative_path = "../stack-provenance.json".to_string();
        manifest.external_evidence = vec![evidence];
        let err = canonical_release_evidence_manifest(manifest).unwrap_err();
        assert!(err.to_string().contains("must not escape"));
    }

    #[test]
    fn validate_rejects_external_evidence_empty_role() {
        let mut manifest = sample_manifest();
        let mut evidence = sample_external_evidence();
        evidence.role = String::new();
        manifest.external_evidence = vec![evidence];
        let err = canonical_release_evidence_manifest(manifest).unwrap_err();
        assert!(err.to_string().contains("external_evidence[0].role"));
    }

    #[test]
    fn validate_rejects_external_evidence_empty_schema() {
        let mut manifest = sample_manifest();
        let mut evidence = sample_external_evidence();
        evidence.schema = String::new();
        manifest.external_evidence = vec![evidence];
        let err = canonical_release_evidence_manifest(manifest).unwrap_err();
        assert!(err.to_string().contains("external_evidence[0].schema"));
    }

    #[test]
    fn validate_rejects_external_evidence_invalid_digest() {
        let mut manifest = sample_manifest();
        let mut evidence = sample_external_evidence();
        evidence.digest_blake3 = "not-a-digest".to_string();
        manifest.external_evidence = vec![evidence];
        let err = canonical_release_evidence_manifest(manifest).unwrap_err();
        assert!(err.to_string().contains("external_evidence[0].digest_blake3"));
    }

    #[test]
    fn validate_rejects_external_evidence_empty_claim_scope() {
        let mut manifest = sample_manifest();
        let mut evidence = sample_external_evidence();
        evidence.claim_scope = String::new();
        manifest.external_evidence = vec![evidence];
        let err = canonical_release_evidence_manifest(manifest).unwrap_err();
        assert!(err.to_string().contains("external_evidence[0].claim_scope"));
    }

    #[test]
    fn validate_rejects_external_evidence_empty_non_claims() {
        let mut manifest = sample_manifest();
        let mut evidence = sample_external_evidence();
        evidence.non_claims = vec![];
        manifest.external_evidence = vec![evidence];
        let err = canonical_release_evidence_manifest(manifest).unwrap_err();
        assert!(err.to_string().contains("external_evidence[0].non_claims"));
    }

    #[test]
    fn validate_rejects_external_evidence_duplicate_role() {
        let mut manifest = sample_manifest();
        let mut second_evidence = sample_external_evidence();
        second_evidence.relative_path = "external-evidence/02-stack-provenance.json".to_string();
        second_evidence.digest_blake3 = sample_digest(18);
        manifest.external_evidence = vec![sample_external_evidence(), second_evidence];
        let err = canonical_release_evidence_manifest(manifest).unwrap_err();
        assert!(err.to_string().contains("duplicates another external evidence role"));
    }

    // r[verify mantle.release_provenance.fixture_matrix.positive]
    // r[verify mantle.release_provenance.valence_receipt_binding]
    #[test]
    fn validate_accepts_stack_provenance_release_evidence_with_matching_sidecars() {
        let mut manifest = sample_manifest();
        manifest.external_evidence = vec![
            sample_stack_provenance_sidecar_evidence(),
            sample_stack_provenance_valence_receipt_evidence(),
        ];
        manifest.stack_provenance = Some(sample_stack_provenance_release_evidence());

        let bytes = canonical_release_evidence_manifest(manifest).unwrap();
        let text = String::from_utf8(bytes).unwrap();

        assert!(text.contains(STACK_PROVENANCE_EVIDENCE_ROLE));
        assert!(text.contains(VALENCE_STACK_PROVENANCE_RECEIPT_ROLE));
        assert!(text.contains(STACK_PROVENANCE_OPAQUE_BOUNDARY));
    }

    // r[verify mantle.release_provenance.valence_receipt_binding.stale]
    #[test]
    fn validate_rejects_stack_provenance_digest_mismatch() {
        let mut manifest = sample_manifest();
        let mut stack_provenance = sample_stack_provenance_release_evidence();
        stack_provenance.sidecar_digest_blake3 = sample_digest(23);
        manifest.external_evidence = vec![
            sample_stack_provenance_sidecar_evidence(),
            sample_stack_provenance_valence_receipt_evidence(),
        ];
        manifest.stack_provenance = Some(stack_provenance);

        let err = canonical_release_evidence_manifest(manifest).unwrap_err();

        assert!(err.to_string().contains("sidecar digest"));
    }

    // r[verify mantle.release_provenance.opaque_boundary.overclaim]
    #[test]
    fn validate_rejects_stack_provenance_semantic_promotion() {
        let mut manifest = sample_manifest();
        let mut stack_provenance = sample_stack_provenance_release_evidence();
        stack_provenance.non_claims = vec![
            STACK_PROVENANCE_OPAQUE_BOUNDARY.to_string(),
            "Mantle verifies Valence stack semantics".to_string(),
        ];
        manifest.external_evidence = vec![
            sample_stack_provenance_sidecar_evidence(),
            sample_stack_provenance_valence_receipt_evidence(),
        ];
        manifest.stack_provenance = Some(stack_provenance);

        let err = canonical_release_evidence_manifest(manifest).unwrap_err();

        assert!(err.to_string().contains("overclaim"));
    }

    // r[verify mantle.release_provenance.stack_profile.required]
    // r[verify mantle.release_provenance.stack_profile.docs.generic]
    #[test]
    fn release_profile_maps_generic_and_onix_stack_modes() {
        let generic_optional =
            stack_provenance_mode_for_release_profile(RELEASE_PROFILE_GENERIC, STACK_PROVENANCE_MODE_OPTIONAL).unwrap();
        let generic_required =
            stack_provenance_mode_for_release_profile(RELEASE_PROFILE_GENERIC, STACK_PROVENANCE_MODE_REQUIRED).unwrap();
        let onix_from_optional =
            stack_provenance_mode_for_release_profile(RELEASE_PROFILE_ONIX_STACK, STACK_PROVENANCE_MODE_OPTIONAL)
                .unwrap();
        let onix_from_required =
            stack_provenance_mode_for_release_profile(RELEASE_PROFILE_ONIX_STACK, STACK_PROVENANCE_MODE_REQUIRED)
                .unwrap();

        assert_eq!(STACK_PROVENANCE_MODE_OPTIONAL, generic_optional);
        assert_eq!(STACK_PROVENANCE_MODE_REQUIRED, generic_required);
        assert_eq!(STACK_PROVENANCE_MODE_REQUIRED, onix_from_optional);
        assert_eq!(STACK_PROVENANCE_MODE_REQUIRED, onix_from_required);
    }

    // r[verify mantle.release_provenance.stack_profile.negative]
    #[test]
    fn release_profile_rejects_unsupported_profile_or_stack_mode() {
        let profile_err =
            stack_provenance_mode_for_release_profile("experimental", STACK_PROVENANCE_MODE_OPTIONAL).unwrap_err();
        let mode_err = stack_provenance_mode_for_release_profile(RELEASE_PROFILE_GENERIC, "ambient").unwrap_err();

        assert!(profile_err.to_string().contains("unsupported release profile"));
        assert!(mode_err.to_string().contains("unsupported stack provenance mode"));
    }

    // r[verify mantle.release_provenance.valence_required_policy.required_valid]
    #[test]
    fn stack_provenance_policy_accepts_required_valid_evidence() {
        let manifest = sample_stack_provenance_manifest();

        let result = evaluate_stack_provenance_release_evidence(&manifest, STACK_PROVENANCE_MODE_REQUIRED);

        assert!(result.valid);
        assert_eq!(STACK_PROVENANCE_DISPOSITION_PRESENT, result.disposition);
    }

    // r[verify mantle.release_provenance.valence_required_policy.optional_absent]
    #[test]
    fn stack_provenance_policy_accepts_optional_absent_evidence() {
        let manifest = sample_manifest();

        let result = evaluate_stack_provenance_release_evidence(&manifest, STACK_PROVENANCE_MODE_OPTIONAL);

        assert!(result.valid);
        assert_eq!(STACK_PROVENANCE_DISPOSITION_ABSENT, result.disposition);
        assert!(result.diagnostics.is_empty());
    }

    // r[verify mantle.release_provenance.fixture_matrix.positive]
    #[test]
    fn stack_provenance_policy_accepts_optional_present_evidence() {
        let manifest = sample_stack_provenance_manifest();

        let result = evaluate_stack_provenance_release_evidence(&manifest, STACK_PROVENANCE_MODE_OPTIONAL);

        assert!(result.valid);
        assert_eq!(STACK_PROVENANCE_DISPOSITION_PRESENT, result.disposition);
    }

    // r[verify mantle.release_provenance.valence_required_policy.required_missing]
    #[test]
    fn stack_provenance_policy_rejects_required_absent_evidence() {
        let manifest = sample_manifest();

        let result = evaluate_stack_provenance_release_evidence(&manifest, STACK_PROVENANCE_MODE_REQUIRED);

        assert!(!result.valid);
        assert_eq!(STACK_PROVENANCE_DISPOSITION_ABSENT, result.disposition);
        assert!(result.diagnostics.iter().any(|diagnostic| diagnostic.contains("required")));
    }

    // r[verify mantle.release_provenance.fixture_matrix]
    // r[verify mantle.release_provenance.fixture_matrix.negative]
    #[test]
    fn stack_provenance_policy_rejects_required_invalid_fixture_matrix() {
        fn missing_sidecar(manifest: &mut ReleaseEvidenceManifest) {
            manifest.external_evidence.retain(|evidence| evidence.role != STACK_PROVENANCE_EVIDENCE_ROLE);
        }
        fn wrong_role(manifest: &mut ReleaseEvidenceManifest) {
            manifest.stack_provenance.as_mut().unwrap().sidecar_role = "wrong-stack-role".to_string();
        }
        fn wrong_schema(manifest: &mut ReleaseEvidenceManifest) {
            manifest.stack_provenance.as_mut().unwrap().sidecar_schema = "wrong.stack.schema".to_string();
        }
        fn wrong_claim_scope(manifest: &mut ReleaseEvidenceManifest) {
            manifest.stack_provenance.as_mut().unwrap().sidecar_claim_scope = "semantic-stack-claim".to_string();
        }
        fn stale_sidecar_digest(manifest: &mut ReleaseEvidenceManifest) {
            manifest.stack_provenance.as_mut().unwrap().sidecar_digest_blake3 = sample_digest(23);
        }
        fn missing_valence_receipt(manifest: &mut ReleaseEvidenceManifest) {
            manifest.external_evidence.retain(|evidence| evidence.role != VALENCE_STACK_PROVENANCE_RECEIPT_ROLE);
        }
        fn stale_valence_receipt_digest(manifest: &mut ReleaseEvidenceManifest) {
            manifest.stack_provenance.as_mut().unwrap().valence_receipt_digest_blake3 = sample_digest(24);
        }
        fn missing_binary_identity(manifest: &mut ReleaseEvidenceManifest) {
            manifest.stack_provenance.as_mut().unwrap().release_binary_relative_path = "binaries/missing".to_string();
        }
        fn weakened_non_claims(manifest: &mut ReleaseEvidenceManifest) {
            manifest.stack_provenance.as_mut().unwrap().non_claims =
                vec!["Mantle verifies Valence stack semantics".to_string()];
        }

        let cases: &[ManifestFixtureCase] = &[
            ("missing sidecar", missing_sidecar, "sidecar external evidence is missing"),
            ("wrong role", wrong_role, "sidecar_role"),
            ("wrong schema", wrong_schema, "sidecar_schema"),
            ("wrong claim scope", wrong_claim_scope, "sidecar_claim_scope"),
            ("stale sidecar digest", stale_sidecar_digest, "sidecar digest"),
            ("missing Valence receipt", missing_valence_receipt, "Valence receipt external evidence is missing"),
            ("stale Valence receipt digest", stale_valence_receipt_digest, "Valence receipt digest"),
            ("missing binary identity", missing_binary_identity, "release_binary_relative_path"),
            ("weakened non-claims", weakened_non_claims, "non-claim"),
        ];

        for (name, mutate, expected_diagnostic) in cases {
            let mut manifest = sample_stack_provenance_manifest();
            mutate(&mut manifest);

            let result = evaluate_stack_provenance_release_evidence(&manifest, STACK_PROVENANCE_MODE_REQUIRED);
            let err = canonical_release_evidence_manifest(manifest).unwrap_err();

            assert!(!result.valid, "{name} should fail required policy");
            assert_eq!(STACK_PROVENANCE_DISPOSITION_INVALID, result.disposition, "{name}");
            assert!(
                result.diagnostics.iter().any(|diagnostic| diagnostic.contains(expected_diagnostic)),
                "{name} diagnostics should contain {expected_diagnostic:?}: {:?}",
                result.diagnostics
            );
            assert!(err.to_string().contains("stack_provenance invalid"));
        }
    }

    // r[verify mantle.release_provenance.opaque_evidence_sidecar_binding.positive]
    // r[verify mantle.release_provenance.opaque_evidence_sidecar_binding.function_address]
    #[test]
    fn validate_accepts_function_address_through_generic_binding_without_projection() {
        let manifest = sample_function_address_manifest(false);

        let bytes = canonical_release_evidence_manifest(manifest).unwrap();
        let text = String::from_utf8(bytes).unwrap();

        assert!(text.contains(OPAQUE_EVIDENCE_KIND_FUNCTION_ADDRESS));
        assert!(text.contains(VALENCE_FUNCTION_ADDRESS_RECEIPT_ROLE));
        assert!(text.contains(FUNCTION_ADDRESS_OPAQUE_BOUNDARY));
    }

    // r[verify mantle.release_provenance.opaque_evidence_sidecar_binding.positive]
    #[test]
    fn validate_accepts_function_address_through_generic_binding_with_projection() {
        let manifest = sample_function_address_manifest(true);

        let bytes = canonical_release_evidence_manifest(manifest).unwrap();
        let text = String::from_utf8(bytes).unwrap();

        assert!(text.contains(FUNCTION_ADDRESS_EVIDENCE_ROLE));
        assert!(text.contains(KAMACITE_FUNCTION_ADDRESS_RECEIPT_ROLE));
        assert!(text.contains(KAMACITE_FUNCTION_ADDRESS_RECEIPT_SCHEMA));
    }

    // r[verify mantle.release_provenance.opaque_evidence_sidecar_binding.positive]
    // r[verify mantle.release_provenance.opaque_evidence_sidecar_binding.opaque_core]
    #[test]
    fn validate_accepts_non_function_lifecycle_stub_as_opaque_metadata() {
        let manifest = sample_lifecycle_opaque_binding_manifest();

        let bytes = canonical_release_evidence_manifest(manifest).unwrap();
        let text = String::from_utf8(bytes).unwrap();

        assert!(text.contains(OPAQUE_EVIDENCE_KIND_LIFECYCLE));
        assert!(text.contains("cairn-lifecycle-evidence-envelope"));
        assert!(!text.contains("profile_payload"));
    }

    #[test]
    fn validate_accepts_multiple_profiles_for_same_non_function_kind() {
        let mut manifest = sample_lifecycle_opaque_binding_manifest();
        let mut second_binding = manifest.opaque_evidence_sidecar_bindings[0].binding.clone();
        second_binding.profile_version = "lifecycle-evidence-v2".to_string();
        second_binding.canonical_envelope.role = "cairn-lifecycle-evidence-envelope-v2".to_string();
        second_binding.canonical_envelope.schema = "cairn.lifecycle-evidence.v2".to_string();
        second_binding.canonical_envelope.relative_path =
            "external-evidence/08-cairn-lifecycle-evidence-v2.bin".to_string();
        second_binding.canonical_envelope.digest_blake3 = sample_digest(SECOND_LIFECYCLE_ENVELOPE_DIGEST_SEED);
        second_binding.upstream_validation.role = "valence-lifecycle-evidence-validation-v2".to_string();
        second_binding.upstream_validation.schema = "valence.lifecycle-evidence-validation.v2".to_string();
        second_binding.upstream_validation.relative_path =
            "external-evidence/09-valence-lifecycle-validation-v2.bin".to_string();
        second_binding.upstream_validation.digest_blake3 = sample_digest(SECOND_LIFECYCLE_VALIDATION_DIGEST_SEED);
        let second_non_claims = second_binding.non_claims.clone();
        manifest.external_evidence.extend([
            ExternalEvidence {
                role: second_binding.canonical_envelope.role.clone(),
                schema: second_binding.canonical_envelope.schema.clone(),
                relative_path: second_binding.canonical_envelope.relative_path.clone(),
                digest_blake3: second_binding.canonical_envelope.digest_blake3.clone(),
                claim_scope: second_binding.claim_scope.clone(),
                non_claims: second_non_claims.clone(),
            },
            ExternalEvidence {
                role: second_binding.upstream_validation.role.clone(),
                schema: second_binding.upstream_validation.schema.clone(),
                relative_path: second_binding.upstream_validation.relative_path.clone(),
                digest_blake3: second_binding.upstream_validation.digest_blake3.clone(),
                claim_scope: second_binding.claim_scope.clone(),
                non_claims: second_non_claims,
            },
        ]);
        manifest
            .opaque_evidence_sidecar_bindings
            .push(opaque_evidence_sidecar_binding_receipt(second_binding).unwrap());

        let bytes = canonical_release_evidence_manifest(manifest).unwrap();
        let text = String::from_utf8(bytes).unwrap();

        assert!(text.contains("lifecycle-evidence-v2"));
        assert!(text.contains("cairn-lifecycle-evidence-envelope-v2"));
        assert!(text.contains("valence-lifecycle-evidence-validation-v2"));
    }

    #[test]
    fn generic_binding_receipt_is_deterministic_and_policy_order_independent() {
        let first_binding = sample_function_address_binding(true);
        let mut second_binding = first_binding.clone();
        second_binding.policy_hashes.reverse();

        let first = opaque_evidence_sidecar_binding_receipt(first_binding).unwrap();
        let second = opaque_evidence_sidecar_binding_receipt(second_binding).unwrap();

        assert_eq!(first, second);
        assert_eq!(first.binding_digest_blake3.len(), BLAKE3_HEX_LENGTH_CHARS);
        assert_eq!(first.binding.evidence_kind, OPAQUE_EVIDENCE_KIND_FUNCTION_ADDRESS);
    }

    #[test]
    fn generic_binding_digest_changes_for_each_typed_link_domain() {
        let base = sample_function_address_binding(true);
        let base_digest = crate::opaque_evidence::opaque_evidence_sidecar_binding_digest_blake3(base.clone()).unwrap();

        let mut envelope_changed = base.clone();
        let envelope_digest = sample_digest(DOMAIN_SEPARATION_ENVELOPE_DIGEST_SEED);
        envelope_changed.canonical_envelope.digest_blake3 = envelope_digest.clone();
        envelope_changed.compatibility_projections[0].canonical_envelope_digest_blake3 = envelope_digest;
        let envelope_binding_digest =
            crate::opaque_evidence::opaque_evidence_sidecar_binding_digest_blake3(envelope_changed).unwrap();

        let mut validation_changed = base.clone();
        validation_changed.upstream_validation.digest_blake3 = sample_digest(DOMAIN_SEPARATION_VALIDATION_DIGEST_SEED);
        let validation_binding_digest =
            crate::opaque_evidence::opaque_evidence_sidecar_binding_digest_blake3(validation_changed).unwrap();

        let mut source_changed = base.clone();
        source_changed.source_artifact.digest_blake3 = sample_digest(DOMAIN_SEPARATION_SOURCE_DIGEST_SEED);
        let source_binding_digest =
            crate::opaque_evidence::opaque_evidence_sidecar_binding_digest_blake3(source_changed).unwrap();

        let mut binary_changed = base.clone();
        binary_changed.release_binary.digest_blake3 = sample_digest(DOMAIN_SEPARATION_BINARY_DIGEST_SEED);
        let binary_binding_digest =
            crate::opaque_evidence::opaque_evidence_sidecar_binding_digest_blake3(binary_changed).unwrap();

        let mut policy_changed = base;
        policy_changed.policy_hashes[0].digest_blake3 = sample_digest(DOMAIN_SEPARATION_POLICY_DIGEST_SEED);
        let policy_binding_digest =
            crate::opaque_evidence::opaque_evidence_sidecar_binding_digest_blake3(policy_changed).unwrap();

        assert_ne!(base_digest, envelope_binding_digest);
        assert_ne!(base_digest, validation_binding_digest);
        assert_ne!(base_digest, source_binding_digest);
        assert_ne!(base_digest, binary_binding_digest);
        assert_ne!(base_digest, policy_binding_digest);
    }

    #[test]
    fn legacy_function_address_manifest_remains_readable() {
        let manifest = sample_legacy_function_address_manifest(true);

        let bytes = canonical_release_evidence_manifest(manifest).unwrap();
        let text = String::from_utf8(bytes.clone()).unwrap();
        let decoded: ReleaseEvidenceManifest = serde_json::from_slice(&bytes).unwrap();
        let result = evaluate_function_address_release_evidence(&decoded, FUNCTION_ADDRESS_MODE_REQUIRED);

        assert!(!text.contains("opaque_evidence_sidecar_bindings"));
        assert!(decoded.opaque_evidence_sidecar_bindings.is_empty());
        assert!(result.valid);
        assert_eq!(FUNCTION_ADDRESS_DISPOSITION_PRESENT, result.disposition);
    }

    #[test]
    fn function_address_release_profile_maps_optional_and_required_modes() {
        let generic_optional =
            function_address_evidence_mode_for_release_profile(RELEASE_PROFILE_GENERIC, FUNCTION_ADDRESS_MODE_OPTIONAL)
                .unwrap();
        let generic_required =
            function_address_evidence_mode_for_release_profile(RELEASE_PROFILE_GENERIC, FUNCTION_ADDRESS_MODE_REQUIRED)
                .unwrap();
        let onix_optional = function_address_evidence_mode_for_release_profile(
            RELEASE_PROFILE_ONIX_STACK,
            FUNCTION_ADDRESS_MODE_OPTIONAL,
        )
        .unwrap();
        let onix_required = function_address_evidence_mode_for_release_profile(
            RELEASE_PROFILE_ONIX_STACK,
            FUNCTION_ADDRESS_MODE_REQUIRED,
        )
        .unwrap();

        assert_eq!(FUNCTION_ADDRESS_MODE_OPTIONAL, generic_optional);
        assert_eq!(FUNCTION_ADDRESS_MODE_REQUIRED, generic_required);
        assert_eq!(FUNCTION_ADDRESS_MODE_OPTIONAL, onix_optional);
        assert_eq!(FUNCTION_ADDRESS_MODE_REQUIRED, onix_required);
    }

    #[test]
    fn function_address_release_profile_rejects_unsupported_profile_or_mode() {
        let profile_err =
            function_address_evidence_mode_for_release_profile("experimental", FUNCTION_ADDRESS_MODE_OPTIONAL)
                .unwrap_err();
        let mode_err =
            function_address_evidence_mode_for_release_profile(RELEASE_PROFILE_GENERIC, "ambient").unwrap_err();

        assert!(profile_err.to_string().contains("unsupported release profile"));
        assert!(mode_err.to_string().contains("unsupported function-address evidence mode"));
    }

    #[test]
    fn function_address_policy_accepts_optional_absent_evidence() {
        let manifest = sample_manifest();

        let result = evaluate_function_address_release_evidence(&manifest, FUNCTION_ADDRESS_MODE_OPTIONAL);

        assert!(result.valid);
        assert_eq!(FUNCTION_ADDRESS_DISPOSITION_ABSENT, result.disposition);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn function_address_policy_accepts_optional_and_required_generic_binding() {
        let manifest = sample_function_address_manifest(true);

        let optional = evaluate_function_address_release_evidence(&manifest, FUNCTION_ADDRESS_MODE_OPTIONAL);
        let required = evaluate_function_address_release_evidence(&manifest, FUNCTION_ADDRESS_MODE_REQUIRED);

        assert!(optional.valid);
        assert!(required.valid);
        assert_eq!(FUNCTION_ADDRESS_DISPOSITION_PRESENT, optional.disposition);
        assert_eq!(FUNCTION_ADDRESS_DISPOSITION_PRESENT, required.disposition);
    }

    // r[verify mantle.release_provenance.function_address_preserves_sidecars.positive]
    // r[verify mantle.release_provenance.function_address_preserves_sidecars.validation]
    #[test]
    fn function_address_preserves_binding_passes_optional_and_required_modes() {
        let optional_manifest = sample_preserves_function_address_manifest(false);
        let required_manifest = sample_preserves_function_address_manifest(true);

        let optional = evaluate_function_address_release_evidence(&optional_manifest, FUNCTION_ADDRESS_MODE_OPTIONAL);
        let required_without_projection =
            evaluate_function_address_release_evidence(&optional_manifest, FUNCTION_ADDRESS_MODE_REQUIRED);
        let required = evaluate_function_address_release_evidence(&required_manifest, FUNCTION_ADDRESS_MODE_REQUIRED);
        let direct = crate::render_function_address_binding_from_preserves_manifest(
            required_manifest.clone(),
            FUNCTION_ADDRESS_MODE_REQUIRED.to_string(),
        )
        .expect("render Preserves-backed direct binding");
        let direct_without_projection = crate::render_function_address_binding_from_preserves_manifest(
            optional_manifest.clone(),
            FUNCTION_ADDRESS_MODE_REQUIRED.to_string(),
        )
        .expect("render Preserves-backed binding without JSON projection");

        assert!(canonical_release_evidence_manifest(optional_manifest).is_ok());
        assert!(canonical_release_evidence_manifest(required_manifest).is_ok());
        assert!(optional.valid);
        assert!(required_without_projection.valid);
        assert!(required.valid);
        assert_eq!(required.sidecar_role.as_deref(), Some(KAMACITE_FUNCTION_ADDRESS_PRESERVES_ROLE));
        assert_eq!(
            required.valence_receipt_hash_blake3,
            Some(sample_digest(FUNCTION_ADDRESS_VALENCE_LOGICAL_HASH_SEED))
        );
        assert_eq!(required.kamacite_receipt_hash_blake3, Some(sample_digest(FUNCTION_ADDRESS_PRESERVES_DIGEST_SEED)));
        assert!(direct.valid);
        assert_eq!(direct.sidecar_digest, sample_digest(FUNCTION_ADDRESS_PRESERVES_DIGEST_SEED));
        assert_eq!(direct.valence_receipt_digest, sample_digest(FUNCTION_ADDRESS_VALENCE_LOGICAL_HASH_SEED));
        assert_eq!(direct.kamacite_receipt_digest, Some(sample_digest(FUNCTION_ADDRESS_PRESERVES_DIGEST_SEED)));
        assert!(direct_without_projection.valid);
        assert!(direct_without_projection.kamacite_receipt_digest.is_none());
    }

    type PreservesBindingMutator = fn(&mut OpaqueEvidenceSidecarBinding);

    const PRESERVES_INVALID_METADATA_CASES: [(&str, PreservesBindingMutator, &str); 12] = [
        ("missing size", |binding| binding.canonical_envelope.size_bytes = None, "size_bytes is required"),
        (
            "oversized canonical sidecar",
            |binding| {
                binding.canonical_envelope.size_bytes =
                    Some(crate::MAX_FUNCTION_ADDRESS_PRESERVES_SIDECAR_BYTES.saturating_add(1))
            },
            "canonical_envelope.size_bytes must be in",
        ),
        (
            "missing Valence logical hash",
            |binding| binding.upstream_validation.receipt_hash_blake3 = None,
            "receipt_hash_blake3 is required",
        ),
        (
            "stale canonical hash",
            |binding| binding.canonical_envelope.digest_blake3 = sample_digest(FUNCTION_ADDRESS_STALE_DIGEST_SEED),
            "canonical envelope digest does not match declared metadata",
        ),
        (
            "projection drift",
            |binding| {
                binding.compatibility_projections[0].canonical_envelope_digest_blake3 =
                    sample_digest(FUNCTION_ADDRESS_STALE_DIGEST_SEED)
            },
            "canonical envelope digest drifted",
        ),
        (
            "wrong role",
            |binding| binding.canonical_envelope.role = "kamacite-semantic-proof".to_string(),
            "canonical_envelope.role",
        ),
        (
            "wrong schema",
            |binding| binding.canonical_envelope.schema = "kamacite.function-address.unknown".to_string(),
            "canonical_envelope.schema",
        ),
        (
            "wrong scope",
            |binding| binding.claim_scope = "function-address-semantic-correctness".to_string(),
            "binding.claim_scope",
        ),
        (
            "malformed hash",
            |binding| binding.canonical_envelope.digest_blake3 = "not-a-blake3".to_string(),
            "canonical_envelope.digest_blake3",
        ),
        (
            "source mismatch",
            |binding| binding.source_artifact.digest_blake3 = sample_digest(FUNCTION_ADDRESS_STALE_DIGEST_SEED),
            "source_artifact.digest_blake3 does not match",
        ),
        (
            "binary mismatch",
            |binding| binding.release_binary.digest_blake3 = sample_digest(FUNCTION_ADDRESS_STALE_DIGEST_SEED),
            "release_binary.digest_blake3 does not match",
        ),
        (
            "semantic overclaim",
            |binding| binding.non_claims.push("Mantle proves function correctness".to_string()),
            "opaque evidence overclaim",
        ),
    ];

    // r[verify mantle.release_provenance.function_address_preserves_sidecars.negative]
    #[test]
    fn function_address_preserves_binding_rejects_invalid_metadata_matrix() {
        for (name, mutate, expected) in PRESERVES_INVALID_METADATA_CASES {
            let mut manifest = sample_preserves_function_address_manifest(true);
            mutate(&mut manifest.opaque_evidence_sidecar_bindings[0].binding);
            let verification = evaluate_function_address_release_evidence(&manifest, FUNCTION_ADDRESS_MODE_REQUIRED);
            let canonical_error = canonical_release_evidence_manifest(manifest)
                .expect_err("invalid Preserves binding must not canonicalize");

            assert!(!verification.valid, "{name} unexpectedly passed");
            assert!(
                verification.diagnostics.iter().any(|diagnostic| diagnostic.contains(expected)),
                "{name} diagnostics missing {expected:?}: {:?}",
                verification.diagnostics
            );
            assert!(canonical_error.to_string().contains("opaque_evidence_sidecar_bindings"));
        }
    }

    #[test]
    fn function_address_policy_rejects_required_absent_evidence() {
        let manifest = sample_manifest();

        let result = evaluate_function_address_release_evidence(&manifest, FUNCTION_ADDRESS_MODE_REQUIRED);

        assert!(!result.valid);
        assert_eq!(FUNCTION_ADDRESS_DISPOSITION_ABSENT, result.disposition);
        assert!(result.diagnostics.iter().any(|diagnostic| diagnostic.contains("required")));
    }

    fn function_address_binding_mut(manifest: &mut ReleaseEvidenceManifest) -> &mut OpaqueEvidenceSidecarBinding {
        assert_eq!(manifest.opaque_evidence_sidecar_bindings.len(), 1);
        let receipt = &mut manifest.opaque_evidence_sidecar_bindings[0];
        assert_eq!(receipt.binding.evidence_kind, OPAQUE_EVIDENCE_KIND_FUNCTION_ADDRESS);
        &mut receipt.binding
    }

    // r[verify mantle.release_provenance.opaque_evidence_sidecar_binding.negative]
    #[test]
    fn function_address_generic_binding_rejects_invalid_fixture_matrix() {
        fn unknown_kind(manifest: &mut ReleaseEvidenceManifest) {
            function_address_binding_mut(manifest).evidence_kind = "unknown-evidence-kind".to_string();
        }
        fn missing_canonical_hash(manifest: &mut ReleaseEvidenceManifest) {
            function_address_binding_mut(manifest).canonical_envelope.digest_blake3.clear();
        }
        fn malformed_canonical_hash(manifest: &mut ReleaseEvidenceManifest) {
            function_address_binding_mut(manifest).canonical_envelope.digest_blake3 = "not-a-blake3".to_string();
        }
        fn stale_valence_hash(manifest: &mut ReleaseEvidenceManifest) {
            function_address_binding_mut(manifest).upstream_validation.digest_blake3 =
                sample_digest(FUNCTION_ADDRESS_STALE_DIGEST_SEED);
        }
        fn stale_binding_receipt(manifest: &mut ReleaseEvidenceManifest) {
            manifest.opaque_evidence_sidecar_bindings[0].binding_digest_blake3 =
                sample_digest(FUNCTION_ADDRESS_STALE_DIGEST_SEED);
        }
        fn duplicate_function_address_binding(manifest: &mut ReleaseEvidenceManifest) {
            let duplicate = manifest.opaque_evidence_sidecar_bindings[0].clone();
            manifest.opaque_evidence_sidecar_bindings.push(duplicate);
        }
        fn source_mismatch(manifest: &mut ReleaseEvidenceManifest) {
            function_address_binding_mut(manifest).source_artifact.digest_blake3 =
                sample_digest(FUNCTION_ADDRESS_STALE_DIGEST_SEED);
        }
        fn binary_mismatch(manifest: &mut ReleaseEvidenceManifest) {
            function_address_binding_mut(manifest).release_binary.digest_blake3 =
                sample_digest(FUNCTION_ADDRESS_STALE_DIGEST_SEED);
        }
        fn unsupported_claim_scope(manifest: &mut ReleaseEvidenceManifest) {
            function_address_binding_mut(manifest).claim_scope = "semantic-function-claim".to_string();
        }
        fn unsupported_profile_version(manifest: &mut ReleaseEvidenceManifest) {
            function_address_binding_mut(manifest).profile_version = "function-address-evidence-v2".to_string();
        }
        fn wrong_role(manifest: &mut ReleaseEvidenceManifest) {
            function_address_binding_mut(manifest).canonical_envelope.role = "wrong-function-role".to_string();
        }
        fn wrong_schema(manifest: &mut ReleaseEvidenceManifest) {
            function_address_binding_mut(manifest).canonical_envelope.schema = "wrong.function.schema".to_string();
        }
        fn projection_drift(manifest: &mut ReleaseEvidenceManifest) {
            function_address_binding_mut(manifest).compatibility_projections[0].canonical_envelope_digest_blake3 =
                sample_digest(FUNCTION_ADDRESS_STALE_DIGEST_SEED);
        }
        fn weakened_non_claims(manifest: &mut ReleaseEvidenceManifest) {
            function_address_binding_mut(manifest).non_claims = vec![FUNCTION_ADDRESS_OPAQUE_BOUNDARY.to_string()];
        }
        fn missing_mantle_policy_hash(manifest: &mut ReleaseEvidenceManifest) {
            function_address_binding_mut(manifest)
                .policy_hashes
                .retain(|policy| policy.policy_kind != OPAQUE_EVIDENCE_POLICY_KIND_MANTLE_RELEASE);
        }
        fn overclaim(manifest: &mut ReleaseEvidenceManifest) {
            function_address_binding_mut(manifest)
                .non_claims
                .push("Mantle proves function correctness".to_string());
        }
        fn missing_envelope_external(manifest: &mut ReleaseEvidenceManifest) {
            manifest.external_evidence.retain(|evidence| evidence.role != FUNCTION_ADDRESS_EVIDENCE_ROLE);
        }
        fn missing_valence_external(manifest: &mut ReleaseEvidenceManifest) {
            manifest.external_evidence.retain(|evidence| evidence.role != VALENCE_FUNCTION_ADDRESS_RECEIPT_ROLE);
        }

        let cases: &[ManifestFixtureCase] = &[
            ("unknown kind", unknown_kind, "evidence_kind is unsupported"),
            ("missing canonical hash", missing_canonical_hash, "canonical_envelope.digest_blake3"),
            ("malformed digest", malformed_canonical_hash, "canonical_envelope.digest_blake3"),
            ("stale Valence hash", stale_valence_hash, "upstream validation digest"),
            ("stale binding receipt", stale_binding_receipt, "binding_digest_blake3"),
            (
                "duplicate function-address binding",
                duplicate_function_address_binding,
                "multiple function-address",
            ),
            ("source mismatch", source_mismatch, "source_artifact.digest_blake3"),
            ("binary mismatch", binary_mismatch, "release_binary.digest_blake3"),
            ("unsupported claim scope", unsupported_claim_scope, "binding.claim_scope"),
            ("unsupported profile version", unsupported_profile_version, "binding.profile_version"),
            ("wrong role", wrong_role, "canonical_envelope.role"),
            ("wrong schema", wrong_schema, "canonical_envelope.schema"),
            ("projection drift", projection_drift, "canonical envelope digest drifted"),
            ("weakened non-claims", weakened_non_claims, "required opaque payload boundary"),
            ("missing policy hash", missing_mantle_policy_hash, "requires at least 2 entries"),
            ("overclaim", overclaim, "overclaim"),
            (
                "missing envelope external",
                missing_envelope_external,
                "canonical envelope external evidence is missing",
            ),
            (
                "missing Valence external",
                missing_valence_external,
                "upstream validation external evidence is missing",
            ),
        ];

        for (name, mutate, expected_diagnostic) in cases {
            let mut manifest = sample_function_address_manifest(true);
            mutate(&mut manifest);

            let result = evaluate_function_address_release_evidence(&manifest, FUNCTION_ADDRESS_MODE_REQUIRED);
            let err = canonical_release_evidence_manifest(manifest).unwrap_err();

            assert!(!result.valid, "{name} should fail required policy");
            assert_eq!(FUNCTION_ADDRESS_DISPOSITION_INVALID, result.disposition, "{name}");
            assert!(
                result.diagnostics.iter().any(|diagnostic| diagnostic.contains(expected_diagnostic)),
                "{name} diagnostics should contain {expected_diagnostic:?}: {:?}",
                result.diagnostics
            );
            assert!(err.to_string().contains("opaque_evidence_sidecar_bindings"));
        }
    }

    #[test]
    fn validate_accepts_kani_toolchain_evidence_with_matching_receipt() {
        let mut manifest = sample_manifest();
        manifest.external_evidence = vec![sample_kani_external_evidence()];
        manifest.kani_toolchain_evidence = vec![sample_kani_toolchain_evidence()];

        let bytes = canonical_release_evidence_manifest(manifest).unwrap();
        let text = String::from_utf8(bytes).unwrap();

        assert!(text.contains(KANI_TOOLCHAIN_EVIDENCE_SCHEMA));
        assert!(text.contains(KANI_VALENCE_SEMANTIC_ROLE));
    }

    #[test]
    fn validate_rejects_kani_missing_version() {
        let mut manifest = sample_manifest();
        let mut evidence = sample_kani_toolchain_evidence();
        evidence.kani_version = String::new();
        manifest.external_evidence = vec![sample_kani_external_evidence()];
        manifest.kani_toolchain_evidence = vec![evidence];

        let err = canonical_release_evidence_manifest(manifest).unwrap_err();

        assert!(err.to_string().contains("kani_version"));
    }

    #[test]
    fn validate_rejects_kani_stale_closure_identity() {
        let mut manifest = sample_manifest();
        let mut evidence = sample_kani_toolchain_evidence();
        evidence.expected_closure_identity_blake3 = sample_digest(15);
        manifest.external_evidence = vec![sample_kani_external_evidence()];
        manifest.kani_toolchain_evidence = vec![evidence];

        let err = canonical_release_evidence_manifest(manifest).unwrap_err();

        assert!(err.to_string().contains("stale"));
    }

    #[test]
    fn validate_rejects_kani_unsupported_solver_metadata() {
        let mut manifest = sample_manifest();
        let mut evidence = sample_kani_toolchain_evidence();
        evidence.solver.kind = "unsupported-solver".to_string();
        manifest.external_evidence = vec![sample_kani_external_evidence()];
        manifest.kani_toolchain_evidence = vec![evidence];

        let err = canonical_release_evidence_manifest(manifest).unwrap_err();

        assert!(err.to_string().contains("unsupported Kani solver"));
    }

    #[test]
    fn validate_rejects_kani_receipt_digest_mismatch() {
        let mut manifest = sample_manifest();
        let mut evidence = sample_kani_toolchain_evidence();
        evidence.receipt_digest_blake3 = sample_digest(15);
        manifest.external_evidence = vec![sample_kani_external_evidence()];
        manifest.kani_toolchain_evidence = vec![evidence];

        let err = canonical_release_evidence_manifest(manifest).unwrap_err();

        assert!(err.to_string().contains("receipt_digest_blake3"));
    }

    #[test]
    fn validate_rejects_kani_missing_required_non_claim() {
        let mut manifest = sample_manifest();
        let mut evidence = sample_kani_toolchain_evidence();
        evidence.non_claims.retain(|non_claim| non_claim != KANI_NON_CLAIM_SEMANTICS);
        manifest.external_evidence = vec![sample_kani_external_evidence()];
        manifest.kani_toolchain_evidence = vec![evidence];

        let err = canonical_release_evidence_manifest(manifest).unwrap_err();

        assert!(err.to_string().contains(KANI_NON_CLAIM_SEMANTICS));
    }

    #[test]
    fn validate_rejects_kani_semantic_promotion() {
        let mut manifest = sample_manifest();
        let mut evidence = sample_kani_toolchain_evidence();
        evidence.valence_semantic_role = "mantle-proves-kani-semantics".to_string();
        manifest.external_evidence = vec![sample_kani_external_evidence()];
        manifest.kani_toolchain_evidence = vec![evidence];

        let err = canonical_release_evidence_manifest(manifest).unwrap_err();

        assert!(err.to_string().contains("valence_semantic_role"));
    }

    #[test]
    fn validate_rejects_kani_receipt_without_toolchain_identity() {
        let mut manifest = sample_manifest();
        manifest.external_evidence = vec![sample_kani_external_evidence()];

        let err = canonical_release_evidence_manifest(manifest).unwrap_err();

        assert!(err.to_string().contains("requires matching kani_toolchain_evidence"));
    }

    #[test]
    fn validate_accepts_manifest_with_valid_provenance_coverage() {
        let mut manifest = sample_manifest();
        manifest.provenance_coverage = Some(ProvenanceCoverage {
            binary_hash: sample_digest(42),
            covered_source_ids: vec!["eq:bs_001".to_string()],
            covered_function_object_ids: vec!["b3:abc123".to_string()],
            covered_requirement_ids: vec!["r[cairn.pricing.black_scholes]".to_string()],
            coverage_boundary: PROVENANCE_COVERAGE_BOUNDARY.to_string(),
        });
        let bytes = canonical_release_evidence_manifest(manifest).unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn validate_rejects_provenance_coverage_with_empty_all_covered_ids() {
        let mut manifest = sample_manifest();
        manifest.provenance_coverage = Some(ProvenanceCoverage {
            binary_hash: sample_digest(42),
            covered_source_ids: vec![],
            covered_function_object_ids: vec![],
            covered_requirement_ids: vec![],
            coverage_boundary: PROVENANCE_COVERAGE_BOUNDARY.to_string(),
        });
        let err = canonical_release_evidence_manifest(manifest).unwrap_err();
        assert!(err.to_string().contains("at least one covered"));
    }

    #[test]
    fn validate_rejects_provenance_coverage_with_weakened_boundary() {
        let mut manifest = sample_manifest();
        manifest.provenance_coverage = Some(ProvenanceCoverage {
            binary_hash: sample_digest(42),
            covered_source_ids: vec!["eq:bs_001".to_string()],
            covered_function_object_ids: vec![],
            covered_requirement_ids: vec![],
            coverage_boundary: "this is fine".to_string(),
        });
        let err = canonical_release_evidence_manifest(manifest).unwrap_err();
        assert!(err.to_string().contains("coverage_boundary"));
    }

    #[test]
    fn validate_rejects_provenance_coverage_with_invalid_binary_hash() {
        let mut manifest = sample_manifest();
        manifest.provenance_coverage = Some(ProvenanceCoverage {
            binary_hash: "not-a-hash".to_string(),
            covered_source_ids: vec!["eq:bs_001".to_string()],
            covered_function_object_ids: vec![],
            covered_requirement_ids: vec![],
            coverage_boundary: PROVENANCE_COVERAGE_BOUNDARY.to_string(),
        });
        let err = canonical_release_evidence_manifest(manifest).unwrap_err();
        assert!(err.to_string().contains("provenance_coverage.binary_hash"));
    }

    // r[impl mantle.release_provenance.trellis_proof_sidecars.positive]
    #[test]
    fn trellis_proof_registered_fixture_is_optional_recorded_only() {
        let manifest = sample_trellis_proof_manifest();
        let observations = sample_trellis_observations(&manifest.opaque_evidence_sidecar_bindings[0].binding);

        let canonical = canonical_release_evidence_manifest(manifest.clone()).expect("canonical Trellis manifest");
        let verification =
            evaluate_trellis_proof_release_evidence(&manifest, TRELLIS_PROOF_MODE_OPTIONAL, Some(&observations));

        assert!(!canonical.is_empty());
        assert!(verification.valid, "{:?}", verification.diagnostics);
        assert_eq!(verification.disposition, TRELLIS_PROOF_DISPOSITION_RECORDED_ONLY);
        assert_eq!(verification.validation_role.as_deref(), Some(TRELLIS_PROOF_VALENCE_ROLE_RECORDED_ONLY));
    }

    #[test]
    fn trellis_formal_proof_candidate_remains_recorded_only_in_mantle() {
        let mut manifest = sample_trellis_proof_manifest();
        let mut binding = manifest.opaque_evidence_sidecar_bindings[0].binding.clone();
        binding.profile_roles.as_mut().expect("fixture roles").producer_role =
            TRELLIS_PROOF_KAMACITE_ROLE_FORMAL_PROOF_CANDIDATE.to_string();
        manifest.opaque_evidence_sidecar_bindings[0] =
            opaque_evidence_sidecar_binding_receipt(binding).expect("candidate binding remains shape-valid");
        let observations = sample_trellis_observations(&manifest.opaque_evidence_sidecar_bindings[0].binding);

        let verification =
            evaluate_trellis_proof_release_evidence(&manifest, TRELLIS_PROOF_MODE_OPTIONAL, Some(&observations));

        assert!(verification.valid, "{:?}", verification.diagnostics);
        assert_eq!(verification.disposition, TRELLIS_PROOF_DISPOSITION_RECORDED_ONLY);
        assert_eq!(verification.producer_role.as_deref(), Some(TRELLIS_PROOF_KAMACITE_ROLE_FORMAL_PROOF_CANDIDATE));
    }

    #[test]
    fn trellis_optional_mode_accepts_absence_without_observations() {
        let verification =
            evaluate_trellis_proof_release_evidence(&sample_manifest(), TRELLIS_PROOF_MODE_OPTIONAL, None);

        assert!(verification.valid);
        assert_eq!(verification.disposition, TRELLIS_PROOF_DISPOSITION_ABSENT);
        assert!(verification.diagnostics.is_empty());
    }

    #[test]
    fn trellis_required_mode_fails_closed_without_accepted_upstream_authority() {
        let manifest = sample_trellis_proof_manifest();
        let observations = sample_trellis_observations(&manifest.opaque_evidence_sidecar_bindings[0].binding);

        let verification =
            evaluate_trellis_proof_release_evidence(&manifest, TRELLIS_PROOF_MODE_REQUIRED, Some(&observations));

        assert!(!verification.valid);
        assert_eq!(verification.disposition, TRELLIS_PROOF_DISPOSITION_INVALID);
        assert!(verification.diagnostics.iter().any(|value| value == TRELLIS_PROOF_ACCEPTANCE_AUTHORITY_BLOCKER));
    }

    // r[impl mantle.release_provenance.trellis_proof_sidecars.negative]
    #[test]
    fn trellis_proof_negative_fixture_matrix_fails_closed() {
        for case in trellis_negative_cases() {
            let diagnostics = execute_trellis_negative_case(&case.name);
            assert!(
                diagnostics.contains(&case.expected),
                "case {} expected {:?}, got {:?}",
                case.name,
                case.expected,
                diagnostics
            );
        }
    }

    fn execute_trellis_negative_case(case_name: &str) -> String {
        let mut manifest = sample_trellis_proof_manifest();
        let mut observations = sample_trellis_observations(&manifest.opaque_evidence_sidecar_bindings[0].binding);
        if case_name == "missing-artifact-observations" {
            return evaluate_trellis_proof_release_evidence(&manifest, TRELLIS_PROOF_MODE_OPTIONAL, None)
                .diagnostics
                .join("\n");
        }
        if mutate_trellis_observations(case_name, &mut observations) {
            return evaluate_trellis_proof_release_evidence(
                &manifest,
                TRELLIS_PROOF_MODE_OPTIONAL,
                Some(&observations),
            )
            .diagnostics
            .join("\n");
        }
        let mut binding = manifest.opaque_evidence_sidecar_bindings[0].binding.clone();
        assert!(mutate_trellis_binding(case_name, &mut binding), "unknown Trellis negative case: {case_name}");
        match opaque_evidence_sidecar_binding_receipt(binding) {
            Ok(receipt) => manifest.opaque_evidence_sidecar_bindings[0] = receipt,
            Err(error) => return error.to_string(),
        }
        evaluate_trellis_proof_release_evidence(&manifest, TRELLIS_PROOF_MODE_OPTIONAL, Some(&observations))
            .diagnostics
            .join("\n")
    }

    fn mutate_trellis_observations(case_name: &str, observed: &mut TrellisProofArtifactObservations) -> bool {
        let stale = sample_digest(TRELLIS_STALE_DIGEST_SEED);
        match case_name {
            "stale-preserves-digest" => observed.canonical_envelope_digest_blake3 = stale,
            "stale-valence-artifact-digest" => observed.valence_artifact_digest_blake3 = stale,
            "stale-valence-logical-receipt-hash" => observed.valence_receipt_hash_blake3 = stale,
            "json-projection-identity-drift" => {
                observed.projection_canonical_envelope_digest_blake3 = Some(stale);
            }
            _ => return false,
        }
        true
    }

    fn mutate_trellis_binding(case_name: &str, binding: &mut OpaqueEvidenceSidecarBinding) -> bool {
        match case_name {
            "missing-canonical-hash" => binding.canonical_envelope.digest_blake3 = String::new(),
            "valence-identity-domain-conflation" => {
                binding.upstream_validation.receipt_hash_blake3 =
                    Some(binding.upstream_validation.digest_blake3.clone());
            }
            "stale-source-link" => binding.source_artifact.digest_blake3 = sample_digest(TRELLIS_STALE_DIGEST_SEED),
            "stale-binary-link" => binding.release_binary.digest_blake3 = sample_digest(TRELLIS_STALE_DIGEST_SEED),
            "missing-policy-hash" => {
                binding.policy_hashes.pop();
            }
            "malformed-policy-digest" => binding.policy_hashes[0].digest_blake3 = "not-a-digest".to_string(),
            "wrong-claim-scope" => binding.claim_scope = "proof-acceptance".to_string(),
            "weakened-non-claims" => binding.non_claims.retain(|value| !value.contains("reference input only")),
            "overclaiming-text" => binding.non_claims.push("Mantle proves release eligibility".to_string()),
            "wrong-upstream-role" => binding.upstream_validation.role = "octet".to_string(),
            "wrong-canonical-schema" => binding.canonical_envelope.schema = "trellis.proof.v0".to_string(),
            "unsupported-proof-profile" => binding.profile_version = "trellis-proof-v0".to_string(),
            "unauthorized-property-promotion" => {
                binding.profile_roles.as_mut().expect("fixture roles").validation_role = "property".to_string();
            }
            "producer-role-domain-substitution" => {
                binding.profile_roles.as_mut().expect("fixture roles").producer_role = "recorded_only".to_string();
            }
            "oversized-preserves-envelope" => {
                binding.canonical_envelope.size_bytes =
                    Some(MAX_TRELLIS_PROOF_PRESERVES_SIDECAR_BYTES.checked_add(1).expect("bounded fixture size"));
            }
            "projection-identity-domain-conflation" => {
                let projection = binding.compatibility_projections.first_mut().expect("fixture projection");
                projection.digest_blake3 = projection.canonical_envelope_digest_blake3.clone();
            }
            _ => return false,
        }
        true
    }
}
