use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::ReleaseEvidenceError;
use crate::manifest::BLAKE3_HEX_LENGTH_CHARS;
use crate::manifest::FunctionAddressReleaseVerification;
use crate::opaque_evidence::FUNCTION_ADDRESS_CLAIM_SCOPE;
use crate::opaque_evidence::FUNCTION_ADDRESS_DISPOSITION_INVALID;
use crate::opaque_evidence::FUNCTION_ADDRESS_DISPOSITION_PRESENT;
use crate::opaque_evidence::FUNCTION_ADDRESS_EVIDENCE_ROLE;
use crate::opaque_evidence::FUNCTION_ADDRESS_EVIDENCE_SCHEMA;
use crate::opaque_evidence::FUNCTION_ADDRESS_MODE_OPTIONAL;
use crate::opaque_evidence::FUNCTION_ADDRESS_MODE_REQUIRED;
use crate::opaque_evidence::FUNCTION_ADDRESS_OPAQUE_BOUNDARY;
use crate::opaque_evidence::KAMACITE_FUNCTION_ADDRESS_RECEIPT_ROLE;
use crate::opaque_evidence::KAMACITE_FUNCTION_ADDRESS_RECEIPT_SCHEMA;
use crate::opaque_evidence::VALENCE_FUNCTION_ADDRESS_RECEIPT_ROLE;
use crate::opaque_evidence::VALENCE_FUNCTION_ADDRESS_RECEIPT_SCHEMA;

// machine-artifact-public: release.function-address-binding

pub const FUNCTION_ADDRESS_BINDING_SCHEMA_VERSION: &str = "mantle.function-address-binding.v1";
pub const FUNCTION_ADDRESS_BINDING_CLAIM_SCOPE: &str = "function_address.v1.lifecycle_policy_conformance";
pub const FUNCTION_ADDRESS_BINDING_VERDICT_PASS: &str = "PASS";
pub const FUNCTION_ADDRESS_BINDING_VERDICT_FAIL: &str = "FAIL";
pub const FUNCTION_ADDRESS_BINDING_CAIRN_NON_CLAIM_BOUNDARY: &str = "Cairn function-address evidence proves lifecycle policy conformance over supplied receipts only; it does not prove Rust semantic correctness, compiler correctness, build correctness, verifier soundness, or release eligibility outside the configured policy";
pub const MIN_FUNCTION_ADDRESS_BINDING_ROLES_COUNT: u32 = 2;
pub const MAX_FUNCTION_ADDRESS_BINDING_ROLES_COUNT: u32 = 3;
pub const MAX_FUNCTION_ADDRESS_BINDING_DIAGNOSTICS_COUNT: u32 = 64;
pub const MIN_FUNCTION_ADDRESS_BINDING_TEXT_BYTES: u32 = 1;
pub const MAX_FUNCTION_ADDRESS_BINDING_TEXT_BYTES: u32 = 4_096;

const KAMACITE_METADATA_INDEX: usize = 2;

// r[impl mantle.release_provenance.function_address_binding_schema.contract]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FunctionAddressBindingReceipt {
    pub schema_version: String,
    pub disposition: String,
    pub valid: bool,
    pub verdict: String,
    pub receipt_hash: String,
    pub claim_scope: String,
    pub sidecar_digest: String,
    pub valence_receipt_digest: String,
    pub source_archive_digest: String,
    pub release_binary_digest: String,
    pub kamacite_receipt_digest: Option<String>,
    pub roles: Vec<String>,
    pub schemas: Vec<String>,
    pub mantle_non_claim_boundary: String,
    pub non_claim_boundary: String,
    pub verification_summary: FunctionAddressReleaseVerification,
}

#[derive(Serialize)]
struct FunctionAddressBindingHashMaterial<'a> {
    schema_version: &'a str,
    disposition: &'a str,
    valid: bool,
    verdict: &'a str,
    claim_scope: &'a str,
    sidecar_digest: &'a str,
    valence_receipt_digest: &'a str,
    source_archive_digest: &'a str,
    release_binary_digest: &'a str,
    kamacite_receipt_digest: Option<&'a str>,
    roles: &'a [String],
    schemas: &'a [String],
    mantle_non_claim_boundary: &'a str,
    non_claim_boundary: &'a str,
    verification_summary: &'a FunctionAddressReleaseVerification,
}

struct FunctionAddressBindingLinks {
    sidecar_digest: String,
    valence_receipt_digest: String,
    source_archive_digest: String,
    release_binary_digest: String,
    kamacite_receipt_digest: Option<String>,
    roles: Vec<String>,
    schemas: Vec<String>,
}

struct OptionalKamaciteLinks {
    digest: Option<String>,
    role: Option<String>,
    schema: Option<String>,
}

struct Blake3Field<'a> {
    value: &'a str,
    name: &'static str,
}

// r[impl mantle.release_provenance.function_address_binding_schema.render]
pub fn render_function_address_binding_receipt(
    verification: FunctionAddressReleaseVerification,
) -> Result<FunctionAddressBindingReceipt, ReleaseEvidenceError> {
    debug_assert!(!FUNCTION_ADDRESS_BINDING_SCHEMA_VERSION.is_empty());
    debug_assert_ne!(FUNCTION_ADDRESS_BINDING_VERDICT_PASS, FUNCTION_ADDRESS_BINDING_VERDICT_FAIL);
    let links = binding_links(&verification)?;
    let verdict = if verification.valid {
        FUNCTION_ADDRESS_BINDING_VERDICT_PASS
    } else {
        FUNCTION_ADDRESS_BINDING_VERDICT_FAIL
    };
    let mut receipt = FunctionAddressBindingReceipt {
        schema_version: FUNCTION_ADDRESS_BINDING_SCHEMA_VERSION.to_string(),
        disposition: verification.disposition.clone(),
        valid: verification.valid,
        verdict: verdict.to_string(),
        receipt_hash: String::new(),
        claim_scope: FUNCTION_ADDRESS_BINDING_CLAIM_SCOPE.to_string(),
        sidecar_digest: links.sidecar_digest,
        valence_receipt_digest: links.valence_receipt_digest,
        source_archive_digest: links.source_archive_digest,
        release_binary_digest: links.release_binary_digest,
        kamacite_receipt_digest: links.kamacite_receipt_digest,
        roles: links.roles,
        schemas: links.schemas,
        mantle_non_claim_boundary: verification.boundary.clone(),
        non_claim_boundary: FUNCTION_ADDRESS_BINDING_CAIRN_NON_CLAIM_BOUNDARY.to_string(),
        verification_summary: verification,
    };
    receipt.receipt_hash = function_address_binding_hash_material_digest(&receipt)?;
    validate_function_address_binding_receipt(receipt)
}

pub fn validate_function_address_binding_receipt(
    receipt: FunctionAddressBindingReceipt,
) -> Result<FunctionAddressBindingReceipt, ReleaseEvidenceError> {
    validate_binding_header(&receipt)?;
    validate_binding_state(&receipt)?;
    validate_binding_links(&receipt)?;
    validate_binding_metadata(&receipt)?;
    validate_binding_diagnostics(&receipt.verification_summary.diagnostics)?;
    validate_binding_receipt_hash(&receipt)?;
    Ok(receipt)
}

pub fn function_address_binding_receipt_canonical_bytes(
    receipt: FunctionAddressBindingReceipt,
) -> Result<Vec<u8>, ReleaseEvidenceError> {
    let receipt = validate_function_address_binding_receipt(receipt)?;
    serde_json::to_vec(&receipt)
        .map_err(|error| parse_error(format!("serializing function-address binding receipt: {error}")))
}

pub fn function_address_binding_receipt_digest_blake3(
    receipt: FunctionAddressBindingReceipt,
) -> Result<String, ReleaseEvidenceError> {
    let receipt = validate_function_address_binding_receipt(receipt)?;
    Ok(receipt.receipt_hash)
}

fn binding_links(
    verification: &FunctionAddressReleaseVerification,
) -> Result<FunctionAddressBindingLinks, ReleaseEvidenceError> {
    debug_assert_ne!(FUNCTION_ADDRESS_EVIDENCE_ROLE, VALENCE_FUNCTION_ADDRESS_RECEIPT_ROLE);
    debug_assert_ne!(FUNCTION_ADDRESS_EVIDENCE_SCHEMA, VALENCE_FUNCTION_ADDRESS_RECEIPT_SCHEMA);
    let sidecar_role = required_text(verification.sidecar_role.as_ref(), "verification_summary.sidecar_role")?;
    let sidecar_schema = required_text(verification.sidecar_schema.as_ref(), "verification_summary.sidecar_schema")?;
    let sidecar_digest =
        required_text(verification.sidecar_digest_blake3.as_ref(), "verification_summary.sidecar_digest_blake3")?;
    let valence_role =
        required_text(verification.valence_receipt_role.as_ref(), "verification_summary.valence_receipt_role")?;
    let valence_schema =
        required_text(verification.valence_receipt_schema.as_ref(), "verification_summary.valence_receipt_schema")?;
    let valence_digest = required_text(
        verification.valence_receipt_digest_blake3.as_ref(),
        "verification_summary.valence_receipt_digest_blake3",
    )?;
    required_text(
        verification.release_binary_relative_path.as_ref(),
        "verification_summary.release_binary_relative_path",
    )?;
    let source_digest = required_text(
        verification.source_archive_digest_blake3.as_ref(),
        "verification_summary.source_archive_digest_blake3",
    )?;
    let binary_digest = required_text(
        verification.release_binary_digest_blake3.as_ref(),
        "verification_summary.release_binary_digest_blake3",
    )?;
    let kamacite = optional_kamacite_links(verification)?;
    let mut roles = vec![sidecar_role, valence_role];
    let mut schemas = vec![sidecar_schema, valence_schema];
    if let (Some(role), Some(schema)) = (kamacite.role, kamacite.schema) {
        roles.push(role);
        schemas.push(schema);
    }
    Ok(FunctionAddressBindingLinks {
        sidecar_digest,
        valence_receipt_digest: valence_digest,
        source_archive_digest: source_digest,
        release_binary_digest: binary_digest,
        kamacite_receipt_digest: kamacite.digest,
        roles,
        schemas,
    })
}

fn optional_kamacite_links(
    verification: &FunctionAddressReleaseVerification,
) -> Result<OptionalKamaciteLinks, ReleaseEvidenceError> {
    match (
        verification.kamacite_receipt_digest_blake3.as_ref(),
        verification.kamacite_receipt_role.as_ref(),
        verification.kamacite_receipt_schema.as_ref(),
    ) {
        (None, None, None) => Ok(OptionalKamaciteLinks {
            digest: None,
            role: None,
            schema: None,
        }),
        (Some(digest), Some(role), Some(schema)) => Ok(OptionalKamaciteLinks {
            digest: Some(digest.clone()),
            role: Some(role.clone()),
            schema: Some(schema.clone()),
        }),
        _ => Err(validation_error(
            "function-address binding Kamacite digest, role, and schema must be all present or all absent",
        )),
    }
}

fn required_text(value: Option<&String>, field: &str) -> Result<String, ReleaseEvidenceError> {
    let Some(value) = value else {
        return Err(validation_error(format!("function-address binding missing {field}")));
    };
    if value.is_empty() {
        return Err(validation_error(format!("function-address binding {field} must not be empty")));
    }
    Ok(value.clone())
}

fn validate_binding_header(receipt: &FunctionAddressBindingReceipt) -> Result<(), ReleaseEvidenceError> {
    if receipt.schema_version != FUNCTION_ADDRESS_BINDING_SCHEMA_VERSION {
        return Err(validation_error(format!(
            "function-address binding schema_version must be {FUNCTION_ADDRESS_BINDING_SCHEMA_VERSION}"
        )));
    }
    if receipt.claim_scope != FUNCTION_ADDRESS_BINDING_CLAIM_SCOPE {
        return Err(validation_error(format!(
            "function-address binding claim_scope must be {FUNCTION_ADDRESS_BINDING_CLAIM_SCOPE}"
        )));
    }
    if receipt.mantle_non_claim_boundary != FUNCTION_ADDRESS_OPAQUE_BOUNDARY {
        return Err(validation_error("function-address binding weakens the Mantle non-claim boundary"));
    }
    if receipt.non_claim_boundary != FUNCTION_ADDRESS_BINDING_CAIRN_NON_CLAIM_BOUNDARY {
        return Err(validation_error("function-address binding weakens the Cairn non-claim boundary"));
    }
    Ok(())
}

fn validate_binding_state(receipt: &FunctionAddressBindingReceipt) -> Result<(), ReleaseEvidenceError> {
    debug_assert_ne!(FUNCTION_ADDRESS_MODE_OPTIONAL, FUNCTION_ADDRESS_MODE_REQUIRED);
    debug_assert_ne!(FUNCTION_ADDRESS_DISPOSITION_PRESENT, FUNCTION_ADDRESS_DISPOSITION_INVALID);
    let summary = &receipt.verification_summary;
    if summary.mode != FUNCTION_ADDRESS_MODE_OPTIONAL && summary.mode != FUNCTION_ADDRESS_MODE_REQUIRED {
        return Err(validation_error(format!(
            "function-address binding verification_summary.mode is unsupported: {}",
            summary.mode
        )));
    }
    if summary.required != (summary.mode == FUNCTION_ADDRESS_MODE_REQUIRED) {
        return Err(validation_error("function-address binding verification_summary.required does not match mode"));
    }
    if receipt.valid != summary.valid || receipt.disposition != summary.disposition {
        return Err(validation_error("function-address binding top-level state does not match verification_summary"));
    }
    let expected_verdict = if summary.valid {
        FUNCTION_ADDRESS_BINDING_VERDICT_PASS
    } else {
        FUNCTION_ADDRESS_BINDING_VERDICT_FAIL
    };
    if receipt.verdict != expected_verdict {
        return Err(validation_error("function-address binding verdict does not match validity"));
    }
    validate_summary_disposition(summary)
}

fn validate_summary_disposition(summary: &FunctionAddressReleaseVerification) -> Result<(), ReleaseEvidenceError> {
    if summary.valid {
        if summary.disposition != FUNCTION_ADDRESS_DISPOSITION_PRESENT || !summary.diagnostics.is_empty() {
            return Err(validation_error("valid function-address binding summary must be present with no diagnostics"));
        }
        return Ok(());
    }
    if summary.disposition != FUNCTION_ADDRESS_DISPOSITION_INVALID || summary.diagnostics.is_empty() {
        return Err(validation_error("invalid function-address binding summary must be invalid with diagnostics"));
    }
    Ok(())
}

fn validate_binding_links(receipt: &FunctionAddressBindingReceipt) -> Result<(), ReleaseEvidenceError> {
    validate_blake3_hex(Blake3Field {
        value: &receipt.sidecar_digest,
        name: "sidecar_digest",
    })?;
    validate_blake3_hex(Blake3Field {
        value: &receipt.valence_receipt_digest,
        name: "valence_receipt_digest",
    })?;
    validate_blake3_hex(Blake3Field {
        value: &receipt.source_archive_digest,
        name: "source_archive_digest",
    })?;
    validate_blake3_hex(Blake3Field {
        value: &receipt.release_binary_digest,
        name: "release_binary_digest",
    })?;
    if let Some(digest) = &receipt.kamacite_receipt_digest {
        validate_blake3_hex(Blake3Field {
            value: digest,
            name: "kamacite_receipt_digest",
        })?;
    }
    debug_assert_eq!(receipt.sidecar_digest.len(), BLAKE3_HEX_LENGTH_CHARS);
    debug_assert_eq!(receipt.valence_receipt_digest.len(), BLAKE3_HEX_LENGTH_CHARS);
    let summary = &receipt.verification_summary;
    require_equal_link(&receipt.sidecar_digest, summary.sidecar_digest_blake3.as_ref(), "sidecar_digest")?;
    require_equal_link(
        &receipt.valence_receipt_digest,
        summary.valence_receipt_digest_blake3.as_ref(),
        "valence_receipt_digest",
    )?;
    require_equal_link(
        &receipt.source_archive_digest,
        summary.source_archive_digest_blake3.as_ref(),
        "source_archive_digest",
    )?;
    require_equal_link(
        &receipt.release_binary_digest,
        summary.release_binary_digest_blake3.as_ref(),
        "release_binary_digest",
    )?;
    if receipt.kamacite_receipt_digest.as_ref() != summary.kamacite_receipt_digest_blake3.as_ref() {
        return Err(validation_error("function-address binding Kamacite receipt digest is stale"));
    }
    Ok(())
}

fn require_equal_link(actual: &str, expected: Option<&String>, field: &str) -> Result<(), ReleaseEvidenceError> {
    let Some(expected) = expected else {
        return Err(validation_error(format!("function-address binding verification_summary is missing {field}")));
    };
    if actual != expected {
        return Err(validation_error(format!("function-address binding {field} is stale")));
    }
    Ok(())
}

fn validate_binding_metadata(receipt: &FunctionAddressBindingReceipt) -> Result<(), ReleaseEvidenceError> {
    let links = binding_links(&receipt.verification_summary)?;
    if receipt.roles != links.roles {
        return Err(validation_error("function-address binding roles do not match verification_summary"));
    }
    if receipt.schemas != links.schemas {
        return Err(validation_error("function-address binding schemas do not match verification_summary"));
    }
    validate_expected_metadata(&receipt.roles, &receipt.schemas, receipt.kamacite_receipt_digest.is_some())?;
    if receipt.verification_summary.sidecar_claim_scope.as_deref() != Some(FUNCTION_ADDRESS_CLAIM_SCOPE) {
        return Err(validation_error(format!(
            "function-address binding verification_summary.sidecar_claim_scope must be {FUNCTION_ADDRESS_CLAIM_SCOPE}"
        )));
    }
    if receipt.verification_summary.boundary != FUNCTION_ADDRESS_OPAQUE_BOUNDARY {
        return Err(validation_error("function-address binding verification_summary weakens the Mantle boundary"));
    }
    Ok(())
}

fn validate_expected_metadata(
    roles: &[String],
    schemas: &[String],
    has_kamacite: bool,
) -> Result<(), ReleaseEvidenceError> {
    let expected_role_count = if has_kamacite {
        MAX_FUNCTION_ADDRESS_BINDING_ROLES_COUNT
    } else {
        MIN_FUNCTION_ADDRESS_BINDING_ROLES_COUNT
    };
    let role_count = bounded_count(roles.len(), "roles")?;
    let schema_count = bounded_count(schemas.len(), "schemas")?;
    debug_assert!(role_count >= MIN_FUNCTION_ADDRESS_BINDING_ROLES_COUNT);
    debug_assert!(schema_count <= MAX_FUNCTION_ADDRESS_BINDING_ROLES_COUNT);
    if role_count != expected_role_count || schema_count != expected_role_count {
        return Err(validation_error("function-address binding role/schema count is inconsistent"));
    }
    if roles.first().map(String::as_str) != Some(FUNCTION_ADDRESS_EVIDENCE_ROLE) {
        return Err(validation_error("function-address binding sidecar role metadata is invalid"));
    }
    if roles.get(1).map(String::as_str) != Some(VALENCE_FUNCTION_ADDRESS_RECEIPT_ROLE) {
        return Err(validation_error("function-address binding Valence role metadata is invalid"));
    }
    if schemas.first().map(String::as_str) != Some(FUNCTION_ADDRESS_EVIDENCE_SCHEMA) {
        return Err(validation_error("function-address binding sidecar schema metadata is invalid"));
    }
    if schemas.get(1).map(String::as_str) != Some(VALENCE_FUNCTION_ADDRESS_RECEIPT_SCHEMA) {
        return Err(validation_error("function-address binding Valence schema metadata is invalid"));
    }
    if has_kamacite {
        if roles.get(KAMACITE_METADATA_INDEX).map(String::as_str) != Some(KAMACITE_FUNCTION_ADDRESS_RECEIPT_ROLE) {
            return Err(validation_error("function-address binding Kamacite role metadata is invalid"));
        }
        if schemas.get(KAMACITE_METADATA_INDEX).map(String::as_str) != Some(KAMACITE_FUNCTION_ADDRESS_RECEIPT_SCHEMA) {
            return Err(validation_error("function-address binding Kamacite schema metadata is invalid"));
        }
    }
    Ok(())
}

fn validate_binding_diagnostics(diagnostics: &[String]) -> Result<(), ReleaseEvidenceError> {
    let count = u32::try_from(diagnostics.len())
        .map_err(|_| validation_error("function-address binding diagnostic count does not fit u32"))?;
    if count > MAX_FUNCTION_ADDRESS_BINDING_DIAGNOSTICS_COUNT {
        return Err(validation_error(format!(
            "function-address binding diagnostics exceed {MAX_FUNCTION_ADDRESS_BINDING_DIAGNOSTICS_COUNT} entries"
        )));
    }
    debug_assert!(count <= MAX_FUNCTION_ADDRESS_BINDING_DIAGNOSTICS_COUNT);
    debug_assert_eq!(count == 0, diagnostics.is_empty());
    for diagnostic in diagnostics {
        let byte_count = u32::try_from(diagnostic.len())
            .map_err(|_| validation_error("function-address binding diagnostic byte count does not fit u32"))?;
        if !(MIN_FUNCTION_ADDRESS_BINDING_TEXT_BYTES..=MAX_FUNCTION_ADDRESS_BINDING_TEXT_BYTES).contains(&byte_count) {
            return Err(validation_error(format!(
                "function-address binding diagnostic must contain {MIN_FUNCTION_ADDRESS_BINDING_TEXT_BYTES}..={MAX_FUNCTION_ADDRESS_BINDING_TEXT_BYTES} bytes"
            )));
        }
        if diagnostic.bytes().any(|byte| byte == 0) {
            return Err(validation_error("function-address binding diagnostic contains a NUL byte"));
        }
    }
    Ok(())
}

fn bounded_count(count: usize, field: &str) -> Result<u32, ReleaseEvidenceError> {
    let count = u32::try_from(count)
        .map_err(|_| validation_error(format!("function-address binding {field} count does not fit u32")))?;
    if !(MIN_FUNCTION_ADDRESS_BINDING_ROLES_COUNT..=MAX_FUNCTION_ADDRESS_BINDING_ROLES_COUNT).contains(&count) {
        return Err(validation_error(format!(
            "function-address binding {field} count must be in {MIN_FUNCTION_ADDRESS_BINDING_ROLES_COUNT}..={MAX_FUNCTION_ADDRESS_BINDING_ROLES_COUNT}"
        )));
    }
    debug_assert!(count >= MIN_FUNCTION_ADDRESS_BINDING_ROLES_COUNT);
    debug_assert!(count <= MAX_FUNCTION_ADDRESS_BINDING_ROLES_COUNT);
    Ok(count)
}

fn validate_binding_receipt_hash(receipt: &FunctionAddressBindingReceipt) -> Result<(), ReleaseEvidenceError> {
    validate_blake3_hex(Blake3Field {
        value: &receipt.receipt_hash,
        name: "receipt_hash",
    })?;
    let expected = function_address_binding_hash_material_digest(receipt)?;
    if receipt.receipt_hash != expected {
        return Err(validation_error("function-address binding receipt_hash does not match canonical material"));
    }
    Ok(())
}

fn function_address_binding_hash_material_digest(
    receipt: &FunctionAddressBindingReceipt,
) -> Result<String, ReleaseEvidenceError> {
    let material = FunctionAddressBindingHashMaterial {
        schema_version: &receipt.schema_version,
        disposition: &receipt.disposition,
        valid: receipt.valid,
        verdict: &receipt.verdict,
        claim_scope: &receipt.claim_scope,
        sidecar_digest: &receipt.sidecar_digest,
        valence_receipt_digest: &receipt.valence_receipt_digest,
        source_archive_digest: &receipt.source_archive_digest,
        release_binary_digest: &receipt.release_binary_digest,
        kamacite_receipt_digest: receipt.kamacite_receipt_digest.as_deref(),
        roles: &receipt.roles,
        schemas: &receipt.schemas,
        mantle_non_claim_boundary: &receipt.mantle_non_claim_boundary,
        non_claim_boundary: &receipt.non_claim_boundary,
        verification_summary: &receipt.verification_summary,
    };
    let bytes = serde_json::to_vec(&material)
        .map_err(|error| parse_error(format!("serializing function-address binding hash material: {error}")))?;
    let digest = blake3::hash(&bytes).to_hex().to_string();
    debug_assert_eq!(digest.len(), BLAKE3_HEX_LENGTH_CHARS);
    debug_assert!(digest.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)));
    Ok(digest)
}

fn validate_blake3_hex(field: Blake3Field<'_>) -> Result<(), ReleaseEvidenceError> {
    if field.value.len() != BLAKE3_HEX_LENGTH_CHARS {
        return Err(validation_error(format!(
            "function-address binding {} must be {BLAKE3_HEX_LENGTH_CHARS} lowercase BLAKE3 hex characters",
            field.name
        )));
    }
    if !field.value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)) {
        return Err(validation_error(format!(
            "function-address binding {} must be {BLAKE3_HEX_LENGTH_CHARS} lowercase BLAKE3 hex characters",
            field.name
        )));
    }
    Ok(())
}

fn validation_error(message: impl Into<String>) -> ReleaseEvidenceError {
    ReleaseEvidenceError::Validation(message.into())
}

fn parse_error(message: impl Into<String>) -> ReleaseEvidenceError {
    ReleaseEvidenceError::Parse(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIDECAR_DIGEST_SEED: u8 = 1;
    const VALENCE_DIGEST_SEED: u8 = 2;
    const KAMACITE_DIGEST_SEED: u8 = 3;
    const SOURCE_DIGEST_SEED: u8 = 4;
    const BINARY_DIGEST_SEED: u8 = 5;
    const STALE_DIGEST_SEED: u8 = 9;
    const HEX_CHARS_PER_BYTE: usize = 2;

    fn digest(seed: u8) -> String {
        format!("{seed:02x}").repeat(BLAKE3_HEX_LENGTH_CHARS / HEX_CHARS_PER_BYTE)
    }

    fn verification(with_kamacite: bool) -> FunctionAddressReleaseVerification {
        FunctionAddressReleaseVerification {
            mode: FUNCTION_ADDRESS_MODE_REQUIRED.to_string(),
            required: true,
            valid: true,
            disposition: FUNCTION_ADDRESS_DISPOSITION_PRESENT.to_string(),
            sidecar_role: Some(FUNCTION_ADDRESS_EVIDENCE_ROLE.to_string()),
            sidecar_schema: Some(FUNCTION_ADDRESS_EVIDENCE_SCHEMA.to_string()),
            sidecar_claim_scope: Some(FUNCTION_ADDRESS_CLAIM_SCOPE.to_string()),
            sidecar_digest_blake3: Some(digest(SIDECAR_DIGEST_SEED)),
            valence_receipt_role: Some(VALENCE_FUNCTION_ADDRESS_RECEIPT_ROLE.to_string()),
            valence_receipt_schema: Some(VALENCE_FUNCTION_ADDRESS_RECEIPT_SCHEMA.to_string()),
            valence_receipt_digest_blake3: Some(digest(VALENCE_DIGEST_SEED)),
            kamacite_receipt_role: with_kamacite.then(|| KAMACITE_FUNCTION_ADDRESS_RECEIPT_ROLE.to_string()),
            kamacite_receipt_schema: with_kamacite.then(|| KAMACITE_FUNCTION_ADDRESS_RECEIPT_SCHEMA.to_string()),
            kamacite_receipt_digest_blake3: with_kamacite.then(|| digest(KAMACITE_DIGEST_SEED)),
            source_archive_digest_blake3: Some(digest(SOURCE_DIGEST_SEED)),
            release_binary_relative_path: Some("binaries/mantle".to_string()),
            release_binary_digest_blake3: Some(digest(BINARY_DIGEST_SEED)),
            boundary: FUNCTION_ADDRESS_OPAQUE_BOUNDARY.to_string(),
            diagnostics: Vec::new(),
        }
    }

    #[test]
    fn renderer_maps_successful_verification_to_direct_cairn_fields() {
        // r[verify mantle.release_provenance.function_address_binding_schema.positive]
        // r[verify mantle.release_provenance.function_address_binding_schema.cairn_mapping]
        // r[verify mantle.release_provenance.function_address_binding_schema.validation]
        let receipt = render_function_address_binding_receipt(verification(true)).expect("render receipt");
        let value = serde_json::to_value(&receipt).expect("serialize receipt");

        assert_eq!(value["schema_version"], FUNCTION_ADDRESS_BINDING_SCHEMA_VERSION);
        assert_eq!(value["valid"], true);
        assert_eq!(value["verdict"], FUNCTION_ADDRESS_BINDING_VERDICT_PASS);
        assert_eq!(value["claim_scope"], FUNCTION_ADDRESS_BINDING_CLAIM_SCOPE);
        assert_eq!(value["sidecar_digest"], digest(SIDECAR_DIGEST_SEED));
        assert_eq!(value["valence_receipt_digest"], digest(VALENCE_DIGEST_SEED));
        assert_eq!(value["source_archive_digest"], digest(SOURCE_DIGEST_SEED));
        assert_eq!(value["release_binary_digest"], digest(BINARY_DIGEST_SEED));
        assert_eq!(value["kamacite_receipt_digest"], digest(KAMACITE_DIGEST_SEED));
        assert_eq!(value["non_claim_boundary"], FUNCTION_ADDRESS_BINDING_CAIRN_NON_CLAIM_BOUNDARY);
        assert_eq!(receipt.receipt_hash.len(), BLAKE3_HEX_LENGTH_CHARS);
    }

    #[test]
    fn rust_renderer_matches_registered_positive_fixture() {
        let expected: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/function-address-release-binding/mantle-binding.valid.json"
        ))
        .expect("parse registered positive fixture");
        let receipt = render_function_address_binding_receipt(verification(true)).expect("render receipt");
        let actual = serde_json::to_value(receipt).expect("serialize receipt");

        assert_eq!(actual, expected);
        assert_eq!(actual["receipt_hash"].as_str().map(str::len), Some(BLAKE3_HEX_LENGTH_CHARS));
    }

    #[test]
    fn renderer_preserves_failure_verification_without_promoting_it() {
        // r[verify mantle.release_provenance.function_address_binding_schema.render]
        let mut summary = verification(true);
        summary.valid = false;
        summary.disposition = FUNCTION_ADDRESS_DISPOSITION_INVALID.to_string();
        summary.diagnostics = vec!["Valence digest does not match declared external evidence".to_string()];

        let receipt = render_function_address_binding_receipt(summary.clone()).expect("render failed receipt");

        assert!(!receipt.valid);
        assert_eq!(receipt.verdict, FUNCTION_ADDRESS_BINDING_VERDICT_FAIL);
        assert_eq!(receipt.disposition, FUNCTION_ADDRESS_DISPOSITION_INVALID);
        assert_eq!(receipt.verification_summary, summary);
        assert_eq!(receipt.mantle_non_claim_boundary, FUNCTION_ADDRESS_OPAQUE_BOUNDARY);
    }

    #[test]
    fn optional_kamacite_metadata_is_all_present_or_all_absent() {
        let without_kamacite = render_function_address_binding_receipt(verification(false)).expect("render receipt");
        assert!(without_kamacite.kamacite_receipt_digest.is_none());
        assert_eq!(
            u32::try_from(without_kamacite.roles.len()).expect("role count fits u32"),
            MIN_FUNCTION_ADDRESS_BINDING_ROLES_COUNT
        );
        assert_eq!(
            u32::try_from(without_kamacite.schemas.len()).expect("schema count fits u32"),
            MIN_FUNCTION_ADDRESS_BINDING_ROLES_COUNT
        );

        let mut partial = verification(false);
        partial.kamacite_receipt_digest_blake3 = Some(digest(KAMACITE_DIGEST_SEED));
        let error = render_function_address_binding_receipt(partial).expect_err("partial Kamacite metadata must fail");
        assert!(error.to_string().contains("all present or all absent"));
    }

    #[test]
    fn canonical_receipt_hash_is_deterministic_and_binds_diagnostics() {
        let first = render_function_address_binding_receipt(verification(true)).expect("render first");
        let second = render_function_address_binding_receipt(verification(true)).expect("render second");
        assert_eq!(first.receipt_hash, second.receipt_hash);
        assert_eq!(
            function_address_binding_receipt_canonical_bytes(first.clone()).expect("canonical first"),
            function_address_binding_receipt_canonical_bytes(second).expect("canonical second")
        );

        let mut failed = verification(true);
        failed.valid = false;
        failed.disposition = FUNCTION_ADDRESS_DISPOSITION_INVALID.to_string();
        failed.diagnostics = vec!["stale Valence digest".to_string()];
        let failed = render_function_address_binding_receipt(failed).expect("render failed receipt");
        assert_ne!(first.receipt_hash, failed.receipt_hash);
    }

    #[test]
    fn malformed_and_stale_binding_receipts_fail_closed() {
        // r[verify mantle.release_provenance.function_address_binding_schema.negative]
        let valid = render_function_address_binding_receipt(verification(true)).expect("render receipt");
        let cases: Vec<(&str, FunctionAddressBindingReceipt)> = vec![
            ("malformed receipt hash", mutate(valid.clone(), |receipt| receipt.receipt_hash = "bad".to_string())),
            (
                "stale Valence digest",
                mutate(valid.clone(), |receipt| {
                    receipt.valence_receipt_digest = digest(STALE_DIGEST_SEED);
                }),
            ),
            (
                "stale Kamacite digest",
                mutate(valid.clone(), |receipt| {
                    receipt.kamacite_receipt_digest = Some(digest(STALE_DIGEST_SEED));
                }),
            ),
            (
                "wrong scope",
                mutate(valid.clone(), |receipt| receipt.claim_scope = "semantic-correctness".to_string()),
            ),
            (
                "weakened Mantle boundary",
                mutate(valid.clone(), |receipt| receipt.mantle_non_claim_boundary = "identity only".to_string()),
            ),
            (
                "weakened Cairn boundary",
                mutate(valid.clone(), |receipt| receipt.non_claim_boundary = "lifecycle passed".to_string()),
            ),
        ];

        for (name, receipt) in cases {
            let error = validate_function_address_binding_receipt(receipt).expect_err(name);
            assert!(!error.to_string().is_empty(), "{name} must have a diagnostic");
        }
        let missing = serde_json::from_str::<FunctionAddressBindingReceipt>("{}");
        assert!(missing.is_err(), "missing required fields must fail deserialization");
    }

    #[test]
    fn receipt_hash_detects_non_linkage_tampering() {
        let valid = render_function_address_binding_receipt(verification(true)).expect("render receipt");
        let tampered =
            mutate(valid, |receipt| receipt.verification_summary.mode = FUNCTION_ADDRESS_MODE_OPTIONAL.to_string());
        let error = validate_function_address_binding_receipt(tampered).expect_err("tampered receipt must fail");
        assert!(error.to_string().contains("required does not match mode"));
        assert!(!error.to_string().contains("accepted"));
    }

    fn mutate(
        mut receipt: FunctionAddressBindingReceipt,
        mutation: impl FnOnce(&mut FunctionAddressBindingReceipt),
    ) -> FunctionAddressBindingReceipt {
        mutation(&mut receipt);
        receipt
    }
}
