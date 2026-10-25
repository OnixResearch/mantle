#![no_std]
#![feature(register_tool)]
#![register_tool(tigerstyle)]
// HARDENING-BACKLOG 2026-10-22: pre-existing tigerstyle findings in this file are
// recorded in .cairn/archive/2026-10-22-schedule-tigerstyle-hardening-backlog/evidence/
// tigerstyle-backlog-2026-10-22.log and scheduled for the standalone hardening
// pass. Scoped to the lint categories present at recording time.
#![allow(
    tigerstyle::explicit_defaults,
    reason = "the published observation DTO keeps its serde default option fields so absent identities stay readable"
)]
//! Identity-only consumers of externally authored contracts. No theorem admission.
extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

mod error;
pub use error::BridgeError;

mod producer;
pub use producer::BoundProducerHandoff;
pub use producer::MAX_TOOL_GRAPH_BYTES;
pub use producer::ProducerBinding;
pub use producer::bind_octet_job;

pub const MAX_DOCUMENT_BYTES: usize = 1_048_576;
pub const MAX_MATERIAL_BYTES: usize = 8_388_608;
pub const MAX_MATERIALS: usize = 128;
pub const MAX_TOTAL_BYTES: usize = 33_554_432;

/// The relative path of the Octet artifact-verification report.
const VERIFICATION_REPORT_PATH: &str = "producer-artifact-verification.json";

/// The relative path of the Octet producer provider profile.
const PROVIDER_PROFILE_PATH: &str = "proofs/aeneas/provider-profile.json";

/// The value one observed external field is required to carry.
///
/// The newtype keeps the required value distinguishable from the observed one
/// at the type level, so a caller cannot swap the two by position.
struct Expected<'a>(&'a str);

/// Packaging identities, not a producer job/profile or an external proof receipt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackagingInputs {
    pub source_revision: String,
    pub source_identity: String,
    pub profile_identity: String,
    pub provider_identity: String,
    pub tool_contract_identity: String,
    pub options_identity: String,
}

pub struct Material {
    pub path: String,
    pub bytes: Vec<u8>,
}
pub struct ObservationRequest {
    pub inputs: PackagingInputs,
    pub profile: Vec<u8>,
    pub receipt: Vec<u8>,
    pub materials: Vec<Material>,
}
pub struct ObservedHandoff {
    pub observation: Observation,
    pub profile: Vec<u8>,
    pub receipt: Vec<u8>,
    pub materials: Vec<Material>,
}
pub struct IdentifiedBytes {
    pub bytes: Vec<u8>,
    pub identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub schema: String,
    pub packaging: PackagingInputs,
    pub receipt_identity: String,
    pub association_identity: String,
    pub producer: String,
    pub externally_reported_outcome: String,
    pub checked_materials: Vec<String>,
    pub proof_acceptance: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_report_identity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub producer_job_identity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_graph_identity: Option<String>,
    pub boundary: String,
}

pub fn identity(bytes: Vec<u8>) -> IdentifiedBytes {
    let identity = digest(&bytes);
    IdentifiedBytes { bytes, identity }
}
fn digest(bytes: &[u8]) -> String {
    format!("b3:{}", blake3::hash(bytes).to_hex())
}

pub fn relative_path(path: String) -> Result<String, BridgeError> {
    if !is_relative_path(&path) {
        return Err(BridgeError::UnsafeArtifactPath(path));
    }
    Ok(path)
}
fn is_relative_path(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 512
        && !path.contains('\\')
        && !path.contains(':')
        && !path.contains('\0')
        && path.split('/').all(|part| !part.is_empty() && part != "." && part != "..")
}

fn document(bytes: &[u8]) -> Result<Value, BridgeError> {
    if bytes.is_empty() || bytes.len() > MAX_DOCUMENT_BYTES {
        return Err(BridgeError::DocumentByteBound);
    }
    serde_json::from_slice(bytes).map_err(BridgeError::ExternalJson)
}

fn text<'a>(value: &'a Value, pointer: &str) -> Result<&'a str, BridgeError> {
    value
        .pointer(pointer)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| BridgeError::MissingExternalField(pointer.to_string()))
}

/// Fails when one observed external field does not carry its expected value.
fn equal(actual: &str, expected: Expected<'_>, field: &str) -> Result<(), BridgeError> {
    let Expected(expected) = expected;
    if actual != expected {
        return Err(BridgeError::FieldMismatch(format!("{field} mismatch: expected {expected}, observed {actual}")));
    }
    Ok(())
}

/// Fails when one observed external digest does not carry its expected digest.
///
/// Both sides may carry the `b3:` prefix, which never changes the digest.
fn digest_equal(actual: &str, expected: Expected<'_>, field: &str) -> Result<(), BridgeError> {
    let Expected(expected) = expected;
    equal(
        actual.strip_prefix("b3:").unwrap_or(actual),
        Expected(expected.strip_prefix("b3:").unwrap_or(expected)),
        field,
    )
}

fn validate_inputs(inputs: &PackagingInputs, materials: &[Material]) -> Result<(), BridgeError> {
    validate_source_revision(&inputs.source_revision)?;
    validate_packaging_identities(inputs)?;
    validate_materials(materials)
}

/// The packaging source revision must be an immutable Git object ID.
fn validate_source_revision(source_revision: &str) -> Result<(), BridgeError> {
    if source_revision.len() != 40 || !source_revision.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(BridgeError::InvalidSourceRevision);
    }
    Ok(())
}

/// Every packaging identity must be a canonical `b3:` digest.
fn validate_packaging_identities(inputs: &PackagingInputs) -> Result<(), BridgeError> {
    for packaging_identity in [
        &inputs.source_identity,
        &inputs.profile_identity,
        &inputs.provider_identity,
        &inputs.tool_contract_identity,
        &inputs.options_identity,
    ] {
        if !is_canonical_digest(packaging_identity) {
            return Err(BridgeError::InvalidPackagingIdentity);
        }
    }
    Ok(())
}

/// True when one packaging identity is a canonical `b3:` digest.
fn is_canonical_digest(packaging_identity: &str) -> bool {
    packaging_identity.strip_prefix("b3:").is_some_and(|hex| {
        hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

/// Materials must be bounded in count and in aggregate byte size, and unique.
fn validate_materials(materials: &[Material]) -> Result<(), BridgeError> {
    if materials.is_empty() || materials.len() > MAX_MATERIALS {
        return Err(BridgeError::MaterialCountBound);
    }
    let mut total_bytes = 0usize;
    for (index, material) in materials.iter().enumerate() {
        validate_material(materials, material, index, &mut total_bytes)?;
    }
    debug_assert!(total_bytes <= MAX_TOTAL_BYTES);
    debug_assert!(materials.len() <= MAX_MATERIALS);
    Ok(())
}

/// One material must carry a safe relative path, a bounded size, and a unique path.
fn validate_material(
    materials: &[Material],
    material: &Material,
    index: usize,
    total_bytes: &mut usize,
) -> Result<(), BridgeError> {
    if !is_relative_path(&material.path) || material.bytes.len() > MAX_MATERIAL_BYTES {
        return Err(BridgeError::InvalidMaterial(material.path.clone()));
    }
    *total_bytes = total_bytes.checked_add(material.bytes.len()).ok_or(BridgeError::MaterialSizeOverflow)?;
    if *total_bytes > MAX_TOTAL_BYTES {
        return Err(BridgeError::TotalMaterialByteBound);
    }
    if materials[..index].iter().any(|prior| prior.path == material.path) {
        return Err(BridgeError::DuplicateMaterial(material.path.clone()));
    }
    Ok(())
}

/// Returns the bytes of one material after checking its path and its digest.
fn material<'a>(materials: &'a [Material], path: &str, digest: Expected<'_>) -> Result<&'a [u8], BridgeError> {
    if !is_relative_path(path) {
        return Err(BridgeError::UnsafeExternalArtifactPath(path.to_string()));
    }
    let bytes = &materials
        .iter()
        .find(|row| row.path == path)
        .ok_or_else(|| BridgeError::MissingPayload(path.to_string()))?
        .bytes;
    let Expected(digest) = digest;
    digest_equal(&self::digest(bytes), Expected(digest), path)?;
    Ok(bytes)
}

/// Checks only the source/profile/provider and selected generated-stage byte links.
/// Full Octet envelope validity MUST still be checked by Octet itself.
pub fn observe_octet(request: ObservationRequest) -> Result<ObservedHandoff, BridgeError> {
    let checked = observe_octet_inner(&request.inputs, &request.profile, &request.receipt, &request.materials)?;
    let observation = finish(request.inputs, &request.receipt, "Octet", checked)?;
    Ok(ObservedHandoff {
        observation,
        profile: request.profile,
        receipt: request.receipt,
        materials: request.materials,
    })
}

/// Link Octet's actual artifact-verification report and every report/envelope
/// payload. This records external acceptance, never reimplements its policy.
pub fn observe_octet_verified(request: ObservationRequest) -> Result<ObservedHandoff, BridgeError> {
    let mut checked = observe_octet_inner(&request.inputs, &request.profile, &request.receipt, &request.materials)?;
    let verification_bytes = verification_report_bytes(&request.materials)?;
    let verification = document(verification_bytes)?;
    validate_verification_report(&verification)?;
    let receipt = document(&request.receipt)?;
    let receipt_identity = digest(&request.receipt);
    let is_receipt_checked =
        check_verified_artifacts(&request.materials, &verification, &receipt, &receipt_identity, &mut checked)?;
    if !is_receipt_checked {
        return Err(BridgeError::UnboundExternalReceipt);
    }
    check_envelope_artifacts(&request.materials, &receipt)?;
    let verification_identity = digest(verification_bytes);
    checked.push(VERIFICATION_REPORT_PATH.to_string());
    debug_assert!(!receipt_identity.is_empty());
    debug_assert!(checked.iter().all(|path| is_relative_path(path)));
    let mut observation = finish(request.inputs, &request.receipt, "Octet", checked)?;
    observation.association_identity = verified_association_identity(VerifiedAssociation {
        observation_identity: &observation.association_identity,
        verification_identity: &verification_identity,
    })?;
    observation.external_report_identity = Some(verification_identity);
    observation.proof_acceptance = "externally-reported-by-octet".to_string();
    observation.boundary = "Octet's exact artifact-verification report records acceptance of this receipt and payloads; Mantle checks byte links only, not proof truth, verifier soundness, or release eligibility".to_string();
    Ok(ObservedHandoff {
        observation,
        profile: request.profile,
        receipt: request.receipt,
        materials: request.materials,
    })
}

/// The bytes of Octet's artifact-verification report.
fn verification_report_bytes(materials: &[Material]) -> Result<&[u8], BridgeError> {
    let bytes = &materials
        .iter()
        .find(|row| row.path == VERIFICATION_REPORT_PATH)
        .ok_or(BridgeError::MissingVerificationReport)?
        .bytes;
    Ok(bytes)
}

/// The artifact-verification report must declare a supported schema, a valid
/// status, and no diagnostics.
fn validate_verification_report(verification: &Value) -> Result<(), BridgeError> {
    if verification.get("schema_version").and_then(Value::as_u64) != Some(1) {
        return Err(BridgeError::UnsupportedVerificationSchema);
    }
    equal(text(verification, "/status")?, Expected("valid"), "external artifact verification")?;
    if !verification.get("diagnostics").and_then(Value::as_array).is_some_and(Vec::is_empty) {
        return Err(BridgeError::VerificationDiagnostics);
    }
    let rows = verification
        .get("artifacts")
        .and_then(Value::as_array)
        .ok_or(BridgeError::MissingVerifiedArtifacts)?;
    if rows.is_empty() || rows.len() > MAX_MATERIALS {
        return Err(BridgeError::VerifiedArtifactCountBound);
    }
    debug_assert!(rows.len() <= MAX_MATERIALS);
    Ok(())
}

/// Checks every artifact the report verified and records the checked paths.
///
/// Returns whether the report binds the exact external receipt.
fn check_verified_artifacts(
    materials: &[Material],
    verification: &Value,
    receipt: &Value,
    receipt_identity: &str,
    checked: &mut Vec<String>,
) -> Result<bool, BridgeError> {
    let rows = verification
        .get("artifacts")
        .and_then(Value::as_array)
        .ok_or(BridgeError::MissingVerifiedArtifacts)?;
    let mut is_receipt_checked = false;
    for row in rows {
        let path = text(row, "/artifact")?;
        let row_digest = text(row, "/blake3")?;
        material(materials, path, Expected(row_digest))?;
        checked.push(path.to_string());
        if digest_equal(row_digest, Expected(receipt_identity), "verified receipt").is_ok() {
            check_verified_receipt_row(row, receipt)?;
            is_receipt_checked = true;
        }
    }
    debug_assert!(rows.len() <= MAX_MATERIALS);
    debug_assert!(checked.iter().all(|path| is_relative_path(path)));
    Ok(is_receipt_checked)
}

/// The verified artifact that binds the receipt must name the external checker.
fn check_verified_receipt_row(row: &Value, receipt: &Value) -> Result<(), BridgeError> {
    equal(text(row, "/checker_id")?, Expected("octet-standards"), "external checker")?;
    equal(text(row, "/evidence_tier")?, Expected("formal_proof"), "external evidence tier")?;
    equal(text(row, "/checked_scope")?, Expected(text(receipt, "/checked_scope")?), "external checked scope")?;
    Ok(())
}

/// Checks every payload the receipt envelope lists.
fn check_envelope_artifacts(materials: &[Material], receipt: &Value) -> Result<(), BridgeError> {
    let envelope = receipt
        .get("artifact_identities")
        .and_then(Value::as_array)
        .ok_or(BridgeError::MissingEnvelopeArtifacts)?;
    for artifact in envelope {
        material(materials, text(artifact, "/locator")?, Expected(text(artifact, "/blake3")?))?;
    }
    Ok(())
}

/// The two identities one verified association binds.
struct VerifiedAssociation<'a> {
    observation_identity: &'a str,
    verification_identity: &'a str,
}

/// The verified association binds the observation to the exact external report.
fn verified_association_identity(association: VerifiedAssociation<'_>) -> Result<String, BridgeError> {
    let bytes = serde_json::to_vec(&(
        "mantle.external-verified-association.v1",
        association.observation_identity,
        association.verification_identity,
    ))
    .map_err(BridgeError::Serialization)?;
    Ok(digest(&bytes))
}

fn observe_octet_inner(
    inputs: &PackagingInputs,
    profile_bytes: &[u8],
    receipt_bytes: &[u8],
    materials: &[Material],
) -> Result<Vec<String>, BridgeError> {
    validate_inputs(inputs, materials)?;
    let profile = document(profile_bytes)?;
    let receipt = document(receipt_bytes)?;
    check_octet_schemas(&profile, &receipt)?;
    check_octet_profile_bytes(inputs, profile_bytes)?;
    let consumer = receipt.get("consumer").ok_or(BridgeError::MissingConsumerBinding)?;
    check_octet_consumer_binding(inputs, &profile, consumer)?;
    let source_path = check_octet_source_material(inputs, consumer, materials)?;
    let source = find_octet_profile_source(&profile, source_path)?;
    check_octet_provider_profile(inputs, materials)?;
    check_octet_options_contract(inputs, &profile)?;
    check_octet_outcome_binding(inputs, source, &receipt, consumer)?;
    let mut checked = alloc::vec![source_path.to_string(), PROVIDER_PROFILE_PATH.to_string()];
    check_octet_stage_artifacts(materials, &receipt, consumer, &mut checked)?;
    debug_assert!(!source_path.is_empty());
    debug_assert!(checked.iter().all(|path| is_relative_path(path)));
    Ok(checked)
}

/// The Octet profile and receipt must carry their published schema versions.
fn check_octet_schemas(profile: &Value, receipt: &Value) -> Result<(), BridgeError> {
    equal(text(profile, "/schema_version")?, Expected("octet-aeneas-consumer-profile/v1"), "profile schema")?;
    equal(text(receipt, "/schema_version")?, Expected("octet-formal-proof-envelope/v1"), "receipt schema")?;
    Ok(())
}

/// The producer profile bytes must hash to the selected profile identity.
fn check_octet_profile_bytes(inputs: &PackagingInputs, profile_bytes: &[u8]) -> Result<(), BridgeError> {
    equal(&digest(profile_bytes), Expected(&inputs.profile_identity), "profile bytes")?;
    Ok(())
}

/// The Octet consumer binding must name the selected profile, source, and provider.
fn check_octet_consumer_binding(
    inputs: &PackagingInputs,
    profile: &Value,
    consumer: &Value,
) -> Result<(), BridgeError> {
    equal(
        text(consumer, "/schema_version")?,
        Expected("octet-aeneas-consumer-receipt-binding/v1"),
        "consumer schema",
    )?;
    equal(text(consumer, "/profile_identity")?, Expected(&inputs.profile_identity), "receipt profile")?;
    equal(text(consumer, "/profile_id")?, Expected(text(profile, "/profile_id")?), "profile id")?;
    equal(text(consumer, "/source_identity")?, Expected(&inputs.source_identity), "source identity")?;
    equal(text(consumer, "/provider_profile_identity")?, Expected(&inputs.provider_identity), "provider")?;
    equal(
        text(profile, "/provider/profile_identity")?,
        Expected(&inputs.provider_identity),
        "profile provider",
    )?;
    equal(
        text(consumer, "/compatibility_matrix_identity")?,
        Expected(text(profile, "/provider/compatibility_matrix_identity")?),
        "compatibility matrix",
    )?;
    debug_assert!(inputs.source_revision.len() == 40);
    debug_assert!(inputs.profile_identity.starts_with("b3:"));
    Ok(())
}

/// The Octet source path must be a material whose bytes hash to the source identity.
fn check_octet_source_material<'a>(
    inputs: &PackagingInputs,
    consumer: &'a Value,
    materials: &[Material],
) -> Result<&'a str, BridgeError> {
    let source_path = text(consumer, "/source_path")?;
    material(materials, source_path, Expected(&inputs.source_identity))?;
    Ok(source_path)
}

/// The producer profile must list the same source path as the consumer binding.
fn find_octet_profile_source<'a>(profile: &'a Value, source_path: &str) -> Result<&'a Value, BridgeError> {
    let paths = profile.pointer("/source/paths").and_then(Value::as_array).ok_or(BridgeError::MissingSourcePaths)?;
    let source = paths
        .iter()
        .find(|row| row.get("path").and_then(Value::as_str) == Some(source_path))
        .ok_or(BridgeError::MissingProfileSource)?;
    Ok(source)
}

/// The provider profile must be a material, and the tool contract must equal the
/// provider identity.
fn check_octet_provider_profile(inputs: &PackagingInputs, materials: &[Material]) -> Result<(), BridgeError> {
    material(materials, PROVIDER_PROFILE_PATH, Expected(&inputs.provider_identity))?;
    equal(&inputs.tool_contract_identity, Expected(&inputs.provider_identity), "provider tool contract")?;
    Ok(())
}

/// The selected options identity must be the digest of the resource policy.
fn check_octet_options_contract(inputs: &PackagingInputs, profile: &Value) -> Result<(), BridgeError> {
    let resource_policy = profile.get("resource_policy").ok_or(BridgeError::MissingResourcePolicy)?;
    let policy_bytes = serde_json::to_vec(resource_policy).map_err(BridgeError::Serialization)?;
    equal(&inputs.options_identity, Expected(&digest(&policy_bytes)), "options contract")?;
    Ok(())
}

/// The profile source identity, the receipt status, and the consumer outcome
/// must agree with the packaging inputs.
fn check_octet_outcome_binding(
    inputs: &PackagingInputs,
    source: &Value,
    receipt: &Value,
    consumer: &Value,
) -> Result<(), BridgeError> {
    equal(text(source, "/identity")?, Expected(&inputs.source_identity), "profile source")?;
    equal(text(receipt, "/status")?, Expected("passed"), "externally reported status")?;
    equal(text(consumer, "/outcome")?, Expected("supported"), "externally reported outcome")?;
    Ok(())
}

/// Every Octet stage must have a matching envelope artifact whose bytes are a
/// material of the handoff.
fn check_octet_stage_artifacts(
    materials: &[Material],
    receipt: &Value,
    consumer: &Value,
    checked: &mut Vec<String>,
) -> Result<(), BridgeError> {
    let stages = consumer
        .get("stage_artifacts")
        .and_then(Value::as_array)
        .ok_or(BridgeError::MissingStageArtifacts)?;
    if stages.is_empty() || stages.len() > MAX_MATERIALS {
        return Err(BridgeError::StageArtifactCountBound);
    }
    let artifacts = receipt
        .get("artifact_identities")
        .and_then(Value::as_array)
        .ok_or(BridgeError::MissingArtifactIdentities)?;
    if artifacts.len() > MAX_MATERIALS {
        return Err(BridgeError::ArtifactIdentityCountBound);
    }
    for stage in stages {
        let stage_digest = text(stage, "/blake3")?;
        let Some(path) = find_stage_artifact(artifacts, stage_digest)? else {
            return Err(BridgeError::MissingStageEnvelope(text(stage, "/role")?.to_string()));
        };
        material(materials, path, Expected(stage_digest))?;
        checked.push(path.to_string());
    }
    debug_assert!(stages.len() <= MAX_MATERIALS);
    debug_assert!(artifacts.len() <= MAX_MATERIALS);
    Ok(())
}

/// Finds the envelope artifact whose digest equals one stage digest.
fn find_stage_artifact<'a>(artifacts: &'a [Value], stage_digest: &str) -> Result<Option<&'a str>, BridgeError> {
    for artifact in artifacts {
        if digest_equal(text(artifact, "/blake3")?, Expected(stage_digest), "stage").is_ok() {
            return Ok(Some(text(artifact, "/locator")?));
        }
    }
    Ok(None)
}

/// Consumes Trellis's existing cross-verifier pilot profile and receipt; never
/// interprets Verus, Lean, theorem mappings, axioms, or admission policy.
pub fn observe_trellis(request: ObservationRequest) -> Result<ObservedHandoff, BridgeError> {
    let checked = observe_trellis_inner(&request.inputs, &request.profile, &request.receipt, &request.materials)?;
    let observation = finish(request.inputs, &request.receipt, "Trellis", checked)?;
    Ok(ObservedHandoff {
        observation,
        profile: request.profile,
        receipt: request.receipt,
        materials: request.materials,
    })
}
fn observe_trellis_inner(
    inputs: &PackagingInputs,
    profile_bytes: &[u8],
    receipt_bytes: &[u8],
    materials: &[Material],
) -> Result<Vec<String>, BridgeError> {
    validate_inputs(inputs, materials)?;
    let profile = document(profile_bytes)?;
    let receipt = document(receipt_bytes)?;
    check_trellis_binding(inputs, &profile, &receipt, profile_bytes)?;
    let source_path = check_trellis_source(inputs, &profile, &receipt, materials)?;
    check_trellis_options_contract(inputs, &profile)?;
    let mut checked = alloc::vec![source_path.to_string()];
    check_trellis_artifacts(materials, &profile, &receipt, &mut checked)?;
    debug_assert!(!source_path.is_empty());
    debug_assert!(checked.iter().all(|path| is_relative_path(path)));
    Ok(checked)
}

/// The Trellis profile and receipt must carry their published schema versions
/// and name the selected profile, source, and provider.
fn check_trellis_binding(
    inputs: &PackagingInputs,
    profile: &Value,
    receipt: &Value,
    profile_bytes: &[u8],
) -> Result<(), BridgeError> {
    equal(
        text(profile, "/schema_version")?,
        Expected("trellis-aeneas-cross-verifier-profile/v1"),
        "profile schema",
    )?;
    equal(text(receipt, "/schema_version")?, Expected("trellis-aeneas-proof-receipt/v1"), "receipt schema")?;
    equal(&digest(profile_bytes), Expected(&inputs.profile_identity), "profile bytes")?;
    equal(text(receipt, "/profile_identity")?, Expected(&inputs.profile_identity), "receipt profile")?;
    equal(text(receipt, "/profile_id")?, Expected(text(profile, "/pilot_id")?), "profile id")?;
    equal(text(receipt, "/source_identity")?, Expected(&inputs.source_identity), "source identity")?;
    equal(
        text(profile, "/consumer/provider/profile_identity")?,
        Expected(&inputs.provider_identity),
        "provider",
    )?;
    debug_assert!(inputs.source_revision.len() == 40);
    debug_assert!(inputs.profile_identity.starts_with("b3:"));
    Ok(())
}

/// The Trellis source path must be a material whose bytes hash to the source
/// identity, and the pilot profile must name the same source.
fn check_trellis_source<'a>(
    inputs: &PackagingInputs,
    profile: &'a Value,
    receipt: &'a Value,
    materials: &[Material],
) -> Result<&'a str, BridgeError> {
    let source_path = text(receipt, "/source_path")?;
    material(materials, source_path, Expected(&inputs.source_identity))?;
    equal(
        text(profile, "/consumer/source/paths/0/identity")?,
        Expected(&inputs.source_identity),
        "profile source",
    )?;
    equal(text(profile, "/consumer/source/paths/0/path")?, Expected(source_path), "profile source path")?;
    equal(text(receipt, "/outcome")?, Expected("supported"), "externally reported outcome")?;
    Ok(source_path)
}

/// The selected tool contract and options identities must be the digests of the
/// provider tools and the extraction options.
fn check_trellis_options_contract(inputs: &PackagingInputs, profile: &Value) -> Result<(), BridgeError> {
    let provider_tools = profile.get("provider_tools").ok_or(BridgeError::MissingProviderTools)?;
    let provider_tools_bytes = serde_json::to_vec(provider_tools).map_err(BridgeError::Serialization)?;
    equal(&digest(&provider_tools_bytes), Expected(&inputs.tool_contract_identity), "provider tool contract")?;
    let extraction = profile.get("extraction").ok_or(BridgeError::MissingExtractionOptions)?;
    let extraction_bytes = serde_json::to_vec(extraction).map_err(BridgeError::Serialization)?;
    equal(&digest(&extraction_bytes), Expected(&inputs.options_identity), "options contract")?;
    Ok(())
}

/// The receipt artifacts must match the profile artifacts, and every artifact
/// must be a material whose recorded byte count is exact.
fn check_trellis_artifacts(
    materials: &[Material],
    profile: &Value,
    receipt: &Value,
    checked: &mut Vec<String>,
) -> Result<(), BridgeError> {
    let rows = receipt.get("artifacts").and_then(Value::as_array).ok_or(BridgeError::MissingTrellisArtifacts)?;
    let expected = profile.get("artifacts").and_then(Value::as_array).ok_or(BridgeError::MissingProfileArtifacts)?;
    if rows.is_empty() || rows.len() > MAX_MATERIALS || rows != expected {
        return Err(BridgeError::ProfileArtifactMismatch);
    }
    let verus_path = text(receipt, "/verus_receipt_path")?;
    equal(verus_path, Expected(text(profile, "/verus/receipt_path")?), "verifier receipt path")?;
    material(materials, verus_path, Expected(text(receipt, "/verus_receipt_identity")?))?;
    checked.push(verus_path.to_string());
    for row in rows {
        let path = text(row, "/path")?;
        let bytes = material(materials, path, Expected(text(row, "/blake3")?))?;
        if row.get("bytes").and_then(Value::as_u64) != Some(bytes.len() as u64) {
            return Err(BridgeError::ArtifactByteCountMismatch(path.to_string()));
        }
        checked.push(path.to_string());
    }
    debug_assert!(rows.len() <= MAX_MATERIALS);
    debug_assert!(checked.iter().all(|path| is_relative_path(path)));
    Ok(())
}

fn finish(
    inputs: PackagingInputs,
    receipt: &[u8],
    producer: &str,
    mut checked: Vec<String>,
) -> Result<Observation, BridgeError> {
    checked.sort();
    checked.dedup();
    let receipt_identity = digest(receipt);
    let association = serde_json::to_vec(&(
        "mantle.external-evidence-association.v1",
        &inputs,
        &receipt_identity,
        producer,
        &checked,
    ))
    .map_err(BridgeError::Serialization)?;
    debug_assert!(!producer.is_empty());
    debug_assert!(checked.windows(2).all(|pair| pair.first() != pair.last()));
    Ok(Observation {
        schema: "mantle-external-evidence-observation-v1".to_string(),
        packaging: inputs, receipt_identity, association_identity: digest(&association),
        producer: producer.to_string(), externally_reported_outcome: "supported".to_string(),
        checked_materials: checked,
        proof_acceptance: "not-evaluated-by-mantle".to_string(),
        external_report_identity: None,
        producer_job_identity: None,
        tool_graph_identity: None,
        boundary: "Mantle checks selected exact-byte links only; producer receipts remain opaque external evidence, not build success, theorem admission, or release eligibility".to_string(),
    })
}
