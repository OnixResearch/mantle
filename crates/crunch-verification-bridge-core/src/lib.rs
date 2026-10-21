#![no_std]
//! Identity-only consumers of externally authored contracts. No theorem admission.
extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

mod producer;
pub use producer::BoundProducerHandoff;
pub use producer::MAX_TOOL_GRAPH_BYTES;
pub use producer::ProducerBinding;
pub use producer::bind_octet_job;

pub const MAX_DOCUMENT_BYTES: usize = 1_048_576;
pub const MAX_MATERIAL_BYTES: usize = 8_388_608;
pub const MAX_MATERIALS: usize = 128;
pub const MAX_TOTAL_BYTES: usize = 33_554_432;

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

pub fn relative_path(path: String) -> Result<String, String> {
    if !is_relative_path(&path) {
        return Err(format!("unsafe artifact path {path}"));
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

fn document(bytes: &[u8]) -> Result<Value, String> {
    if bytes.is_empty() || bytes.len() > MAX_DOCUMENT_BYTES {
        return Err("document exceeds bridge byte bound or is empty".to_string());
    }
    serde_json::from_slice(bytes).map_err(|error| format!("external JSON: {error}"))
}

fn text<'a>(value: &'a Value, pointer: &str) -> Result<&'a str, String> {
    value
        .pointer(pointer)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("missing external field {pointer}"))
}

fn equal(actual: &str, expected: &str, field: &str) -> Result<(), String> {
    if actual != expected {
        return Err(format!("{field} mismatch: expected {expected}, observed {actual}"));
    }
    Ok(())
}

fn digest_equal(actual: &str, expected: &str, field: &str) -> Result<(), String> {
    equal(
        actual.strip_prefix("b3:").unwrap_or(actual),
        expected.strip_prefix("b3:").unwrap_or(expected),
        field,
    )
}

fn validate_inputs(inputs: &PackagingInputs, materials: &[Material]) -> Result<(), String> {
    if inputs.source_revision.len() != 40 || !inputs.source_revision.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("source_revision must be an immutable Git object ID".to_string());
    }
    for digest in [
        &inputs.source_identity,
        &inputs.profile_identity,
        &inputs.provider_identity,
        &inputs.tool_contract_identity,
        &inputs.options_identity,
    ] {
        let valid = digest.strip_prefix("b3:").is_some_and(|hex| {
            hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        });
        if !valid {
            return Err("packaging identities must be canonical b3 digests".to_string());
        }
    }
    if materials.is_empty() || materials.len() > MAX_MATERIALS {
        return Err("material count exceeds bridge bound or is empty".to_string());
    }
    let mut total = 0usize;
    for (index, material) in materials.iter().enumerate() {
        if !is_relative_path(&material.path) || material.bytes.len() > MAX_MATERIAL_BYTES {
            return Err(format!("invalid or oversized material {}", material.path));
        }
        total = total.checked_add(material.bytes.len()).ok_or("material size overflow")?;
        if total > MAX_TOTAL_BYTES {
            return Err("aggregate material byte bound exceeded".to_string());
        }
        if materials[..index].iter().any(|prior| prior.path == material.path) {
            return Err(format!("duplicate material {}", material.path));
        }
    }
    Ok(())
}

fn material<'a>(materials: &'a [Material], path: &str, digest: &str) -> Result<&'a [u8], String> {
    if !is_relative_path(path) {
        return Err(format!("unsafe external artifact path {path}"));
    }
    let bytes = &materials
        .iter()
        .find(|row| row.path == path)
        .ok_or_else(|| format!("missing payload {path}"))?
        .bytes;
    digest_equal(&self::digest(bytes), digest, path)?;
    Ok(bytes)
}

/// Checks only the source/profile/provider and selected generated-stage byte links.
/// Full Octet envelope validity MUST still be checked by Octet itself.
pub fn observe_octet(request: ObservationRequest) -> Result<ObservedHandoff, String> {
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
pub fn observe_octet_verified(request: ObservationRequest) -> Result<ObservedHandoff, String> {
    let mut checked = observe_octet_inner(&request.inputs, &request.profile, &request.receipt, &request.materials)?;
    let report_path = "producer-artifact-verification.json";
    let report_bytes = &request
        .materials
        .iter()
        .find(|row| row.path == report_path)
        .ok_or("missing external artifact-verification report")?
        .bytes;
    let report = document(report_bytes)?;
    if report.get("schema_version").and_then(Value::as_u64) != Some(1) {
        return Err("unsupported Octet artifact-verification schema".to_string());
    }
    equal(text(&report, "/status")?, "valid", "external artifact verification")?;
    if !report.get("diagnostics").and_then(Value::as_array).is_some_and(Vec::is_empty) {
        return Err("external artifact verification has diagnostics".to_string());
    }
    let rows = report.get("artifacts").and_then(Value::as_array).ok_or("missing verified artifacts")?;
    if rows.is_empty() || rows.len() > MAX_MATERIALS {
        return Err("invalid verified artifact count".to_string());
    }
    let receipt_identity = digest(&request.receipt);
    let receipt = document(&request.receipt)?;
    let mut receipt_checked = false;
    for row in rows {
        let path = text(row, "/artifact")?;
        let row_digest = text(row, "/blake3")?;
        material(&request.materials, path, row_digest)?;
        checked.push(path.to_string());
        if digest_equal(row_digest, &receipt_identity, "verified receipt").is_ok() {
            equal(text(row, "/checker_id")?, "octet-standards", "external checker")?;
            equal(text(row, "/evidence_tier")?, "formal_proof", "external evidence tier")?;
            equal(text(row, "/checked_scope")?, text(&receipt, "/checked_scope")?, "external checked scope")?;
            receipt_checked = true;
        }
    }
    if !receipt_checked {
        return Err("Octet report does not bind the exact external receipt".to_string());
    }
    let envelope = receipt.get("artifact_identities").and_then(Value::as_array).ok_or("missing envelope artifacts")?;
    for artifact in envelope {
        material(&request.materials, text(artifact, "/locator")?, text(artifact, "/blake3")?)?;
    }
    let report_identity = digest(report_bytes);
    checked.push(report_path.to_string());
    let mut observation = finish(request.inputs, &request.receipt, "Octet", checked)?;
    observation.association_identity = digest(
        &serde_json::to_vec(&(
            "mantle.external-verified-association.v1",
            &observation.association_identity,
            &report_identity,
        ))
        .map_err(|error| error.to_string())?,
    );
    observation.external_report_identity = Some(report_identity);
    observation.proof_acceptance = "externally-reported-by-octet".to_string();
    observation.boundary = "Octet's exact artifact-verification report records acceptance of this receipt and payloads; Mantle checks byte links only, not proof truth, verifier soundness, or release eligibility".to_string();
    Ok(ObservedHandoff {
        observation,
        profile: request.profile,
        receipt: request.receipt,
        materials: request.materials,
    })
}

fn observe_octet_inner(
    inputs: &PackagingInputs,
    profile_bytes: &[u8],
    receipt_bytes: &[u8],
    materials: &[Material],
) -> Result<Vec<String>, String> {
    validate_inputs(inputs, materials)?;
    let profile = document(profile_bytes)?;
    let receipt = document(receipt_bytes)?;
    equal(text(&profile, "/schema_version")?, "octet-aeneas-consumer-profile/v1", "profile schema")?;
    equal(text(&receipt, "/schema_version")?, "octet-formal-proof-envelope/v1", "receipt schema")?;
    equal(&digest(profile_bytes), &inputs.profile_identity, "profile bytes")?;
    let consumer = receipt.get("consumer").ok_or("missing Octet consumer binding")?;
    equal(text(consumer, "/schema_version")?, "octet-aeneas-consumer-receipt-binding/v1", "consumer schema")?;
    equal(text(consumer, "/profile_identity")?, &inputs.profile_identity, "receipt profile")?;
    equal(text(consumer, "/profile_id")?, text(&profile, "/profile_id")?, "profile id")?;
    equal(text(consumer, "/source_identity")?, &inputs.source_identity, "source identity")?;
    equal(text(consumer, "/provider_profile_identity")?, &inputs.provider_identity, "provider")?;
    equal(text(&profile, "/provider/profile_identity")?, &inputs.provider_identity, "profile provider")?;
    equal(
        text(consumer, "/compatibility_matrix_identity")?,
        text(&profile, "/provider/compatibility_matrix_identity")?,
        "compatibility matrix",
    )?;
    let source_path = text(consumer, "/source_path")?;
    material(materials, source_path, &inputs.source_identity)?;
    let paths = profile.pointer("/source/paths").and_then(Value::as_array).ok_or("missing source paths")?;
    let source = paths
        .iter()
        .find(|row| row.get("path").and_then(Value::as_str) == Some(source_path))
        .ok_or("source absent from producer profile")?;
    material(materials, "proofs/aeneas/provider-profile.json", &inputs.provider_identity)?;
    equal(&inputs.tool_contract_identity, &inputs.provider_identity, "provider tool contract")?;
    equal(
        &inputs.options_identity,
        &digest(
            &serde_json::to_vec(profile.get("resource_policy").ok_or("missing resource policy")?)
                .map_err(|error| error.to_string())?,
        ),
        "options contract",
    )?;
    equal(text(source, "/identity")?, &inputs.source_identity, "profile source")?;
    equal(text(&receipt, "/status")?, "passed", "externally reported status")?;
    equal(text(consumer, "/outcome")?, "supported", "externally reported outcome")?;
    let stages = consumer.get("stage_artifacts").and_then(Value::as_array).ok_or("missing stage artifacts")?;
    if stages.is_empty() || stages.len() > MAX_MATERIALS {
        return Err("invalid stage artifact count".to_string());
    }
    let artifacts =
        receipt.get("artifact_identities").and_then(Value::as_array).ok_or("missing artifact identities")?;
    if artifacts.len() > MAX_MATERIALS {
        return Err("artifact identity count exceeds bound".to_string());
    }
    let mut checked = alloc::vec![
        source_path.to_string(),
        "proofs/aeneas/provider-profile.json".to_string()
    ];
    for stage in stages {
        let digest = text(stage, "/blake3")?;
        let mut matched = false;
        for artifact in artifacts {
            if digest_equal(text(artifact, "/blake3")?, digest, "stage").is_ok() {
                let path = text(artifact, "/locator")?;
                material(materials, path, digest)?;
                checked.push(path.to_string());
                matched = true;
                break;
            }
        }
        if !matched {
            return Err(format!("stage {} has no envelope artifact", text(stage, "/role")?));
        }
    }
    Ok(checked)
}

/// Consumes Trellis's existing cross-verifier pilot profile and receipt; never
/// interprets Verus, Lean, theorem mappings, axioms, or admission policy.
pub fn observe_trellis(request: ObservationRequest) -> Result<ObservedHandoff, String> {
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
) -> Result<Vec<String>, String> {
    validate_inputs(inputs, materials)?;
    let profile = document(profile_bytes)?;
    let receipt = document(receipt_bytes)?;
    equal(text(&profile, "/schema_version")?, "trellis-aeneas-cross-verifier-profile/v1", "profile schema")?;
    equal(text(&receipt, "/schema_version")?, "trellis-aeneas-proof-receipt/v1", "receipt schema")?;
    equal(&digest(profile_bytes), &inputs.profile_identity, "profile bytes")?;
    equal(text(&receipt, "/profile_identity")?, &inputs.profile_identity, "receipt profile")?;
    equal(text(&receipt, "/profile_id")?, text(&profile, "/pilot_id")?, "profile id")?;
    equal(text(&receipt, "/source_identity")?, &inputs.source_identity, "source identity")?;
    equal(text(&profile, "/consumer/provider/profile_identity")?, &inputs.provider_identity, "provider")?;
    let source_path = text(&receipt, "/source_path")?;
    material(materials, source_path, &inputs.source_identity)?;
    equal(text(&profile, "/consumer/source/paths/0/identity")?, &inputs.source_identity, "profile source")?;
    equal(text(&profile, "/consumer/source/paths/0/path")?, source_path, "profile source path")?;
    equal(text(&receipt, "/outcome")?, "supported", "externally reported outcome")?;
    equal(
        &digest(
            &serde_json::to_vec(profile.get("provider_tools").ok_or("missing provider tools")?)
                .map_err(|error| error.to_string())?,
        ),
        &inputs.tool_contract_identity,
        "provider tool contract",
    )?;
    equal(
        &digest(
            &serde_json::to_vec(profile.get("extraction").ok_or("missing extraction options")?)
                .map_err(|error| error.to_string())?,
        ),
        &inputs.options_identity,
        "options contract",
    )?;
    let rows = receipt.get("artifacts").and_then(Value::as_array).ok_or("missing Trellis artifacts")?;
    let expected = profile.get("artifacts").and_then(Value::as_array).ok_or("missing profile artifacts")?;
    if rows.is_empty() || rows.len() > MAX_MATERIALS || rows != expected {
        return Err("receipt artifacts differ from producer profile".to_string());
    }
    let mut checked = alloc::vec![source_path.to_string()];
    let verus_path = text(&receipt, "/verus_receipt_path")?;
    equal(verus_path, text(&profile, "/verus/receipt_path")?, "verifier receipt path")?;
    material(materials, verus_path, text(&receipt, "/verus_receipt_identity")?)?;
    checked.push(verus_path.to_string());
    for row in rows {
        let path = text(row, "/path")?;
        let bytes = material(materials, path, text(row, "/blake3")?)?;
        if row.get("bytes").and_then(Value::as_u64) != Some(bytes.len() as u64) {
            return Err(format!("artifact byte count mismatch: {path}"));
        }
        checked.push(path.to_string());
    }
    Ok(checked)
}

fn finish(
    inputs: PackagingInputs,
    receipt: &[u8],
    producer: &str,
    mut checked: Vec<String>,
) -> Result<Observation, String> {
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
    .map_err(|error| error.to_string())?;
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
