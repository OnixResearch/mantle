// machine-artifact-public: frontend.artifact-reports
use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

use crate::frontend_artifact_spec::FRONTEND_ARTIFACT_VALIDATION_RESULT_ADMITTED;
use crate::frontend_artifact_spec::FrontendArtifactAdmissionAttestation;

pub const FRONTEND_ARTIFACT_EXPORT_RECEIPT_SCHEMA: &str = "mantle-frontend-artifact-export-v1";
pub const FRONTEND_ARTIFACT_EXPORT_HASH_ALGORITHM_BLAKE3: &str = "blake3";
pub const FRONTEND_ARTIFACT_EXPORT_DIGEST_PREFIX_BLAKE3: &str = "blake3:";
pub const FRONTEND_ARTIFACT_EXPORT_REF_SCHEME_MANTLE: &str = "mantle://";
pub const FRONTEND_ARTIFACT_EXPORT_MODE_DIRECTORY: &str = "directory";
pub const FRONTEND_ARTIFACT_EXPORT_MODE_ARCHIVE: &str = "archive";
pub const FRONTEND_ARTIFACT_EXPORT_MODE_STREAM: &str = "stream";
pub const FRONTEND_ARTIFACT_EXPORT_DIAG_EMPTY_FIELD: &str = "frontend-artifact-export-empty-field";
pub const FRONTEND_ARTIFACT_EXPORT_DIAG_MISSING_ADMISSION_PROOF: &str =
    "frontend-artifact-export-missing-admission-proof";
pub const FRONTEND_ARTIFACT_EXPORT_DIAG_UNSUPPORTED_REF_SCHEME: &str =
    "frontend-artifact-export-unsupported-ref-scheme";
pub const FRONTEND_ARTIFACT_EXPORT_DIAG_UNSUPPORTED_DESTINATION_MODE: &str =
    "frontend-artifact-export-unsupported-destination-mode";
pub const FRONTEND_ARTIFACT_EXPORT_DIAG_PROOF_MISMATCH: &str = "frontend-artifact-export-proof-mismatch";
pub const FRONTEND_ARTIFACT_EXPORT_DIAG_INVALID_DIGEST: &str = "frontend-artifact-export-invalid-digest";
pub const FRONTEND_ARTIFACT_EXPORT_DIAG_DIGEST_MISMATCH: &str = "frontend-artifact-export-digest-mismatch";
pub const FRONTEND_ARTIFACT_EXPORT_DIAG_CONTENT_UNAVAILABLE: &str = "frontend-artifact-export-content-unavailable";
pub const FRONTEND_ARTIFACT_EXPORT_DIAG_HIDDEN_FALLBACK: &str = "frontend-artifact-export-hidden-fallback";

const HEX_CHARS_PER_BYTE: usize = 2;
const BLAKE3_HEX_LENGTH: usize = blake3::OUT_LEN.saturating_mul(HEX_CHARS_PER_BYTE);
const RECEIPT_PREIMAGE_VERSION: &str = "mantle-frontend-artifact-export-receipt-preimage-v1";
const RECEIPT_FIELD_SEPARATOR: &str = "\u{0}";
const RECEIPT_RECORD_SEPARATOR: &str = "\n";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontendArtifactExportExpectation<'a> {
    pub artifact_ref: &'a str,
    pub artifact_digest: Option<&'a str>,
    pub spec_id: Option<&'a str>,
    pub spec_version: Option<&'a str>,
    pub spec_hash: Option<&'a str>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrontendArtifactExportContent {
    pub artifact_ref: String,
    pub artifact_digest: String,
    pub materialized_path: String,
    pub provenance: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontendArtifactExportPreflightRequest<'a> {
    pub expectation: FrontendArtifactExportExpectation<'a>,
    pub attestation: Option<&'a FrontendArtifactAdmissionAttestation>,
    pub destination_mode: &'a str,
    pub supported_destination_modes: &'a [&'a str],
    pub no_hidden_fallback: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontendArtifactExportRequest<'a> {
    pub preflight: FrontendArtifactExportPreflightRequest<'a>,
    pub content: Option<&'a FrontendArtifactExportContent>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrontendArtifactExportDiagnostic {
    pub code: String,
    pub path: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrontendArtifactExportReceipt {
    pub schema: String,
    pub artifact_ref: String,
    pub artifact_digest: String,
    pub materialized_path: String,
    pub destination_mode: String,
    pub spec_id: String,
    pub spec_version: String,
    pub spec_hash: String,
    pub validator_kind: String,
    pub validator_ref: String,
    pub validation_result: String,
    pub artifact_kind: String,
    pub build_root: String,
    pub content_provenance: BTreeMap<String, String>,
    pub no_hidden_fallback: bool,
    pub receipt_hash_algorithm: String,
    pub receipt_hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct FrontendArtifactExportReceiptMaterial {
    schema: String,
    artifact_ref: String,
    artifact_digest: String,
    materialized_path: String,
    destination_mode: String,
    spec_id: String,
    spec_version: String,
    spec_hash: String,
    validator_kind: String,
    validator_ref: String,
    validation_result: String,
    artifact_kind: String,
    build_root: String,
    content_provenance: BTreeMap<String, String>,
    no_hidden_fallback: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrontendArtifactExportReport {
    pub exported: bool,
    pub receipt: Option<FrontendArtifactExportReceipt>,
    pub diagnostics: Vec<FrontendArtifactExportDiagnostic>,
}

pub fn validate_frontend_artifact_export_preflight(
    request: &FrontendArtifactExportPreflightRequest<'_>,
) -> Vec<FrontendArtifactExportDiagnostic> {
    let mut diagnostics = Vec::new();
    validate_request_shape(request, &mut diagnostics);
    validate_expected_spec_digest(request.expectation.spec_hash, "expectation.spec_hash", &mut diagnostics);
    validate_expected_artifact_digest(
        request.expectation.artifact_digest,
        "expectation.artifact_digest",
        &mut diagnostics,
    );
    validate_attestation(request, &mut diagnostics);
    diagnostics
}

pub fn export_frontend_artifact(request: &FrontendArtifactExportRequest<'_>) -> FrontendArtifactExportReport {
    let mut diagnostics = validate_frontend_artifact_export_preflight(&request.preflight);
    validate_content(request, &mut diagnostics);

    if diagnostics.is_empty()
        && let (Some(attestation), Some(content)) = (request.preflight.attestation, request.content)
    {
        return report(Some(receipt_from_validated(request, attestation, content)), diagnostics);
    }

    report(None, diagnostics)
}

pub fn render_frontend_artifact_export_receipt(
    receipt: &FrontendArtifactExportReceipt,
) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(receipt)
}

fn validate_request_shape(
    request: &FrontendArtifactExportPreflightRequest<'_>,
    diagnostics: &mut Vec<FrontendArtifactExportDiagnostic>,
) {
    debug_assert!(!FRONTEND_ARTIFACT_EXPORT_REF_SCHEME_MANTLE.is_empty());
    debug_assert!(!FRONTEND_ARTIFACT_EXPORT_DIAG_HIDDEN_FALLBACK.is_empty());
    require_non_empty(request.expectation.artifact_ref, "expectation.artifact_ref", diagnostics);
    require_non_empty(request.destination_mode, "destination_mode", diagnostics);
    if !request.expectation.artifact_ref.starts_with(FRONTEND_ARTIFACT_EXPORT_REF_SCHEME_MANTLE) {
        diagnostics.push(diagnostic(
            FRONTEND_ARTIFACT_EXPORT_DIAG_UNSUPPORTED_REF_SCHEME,
            "expectation.artifact_ref",
            "frontend artifact export requires a mantle:// artifact ref",
        ));
    }
    if !request.supported_destination_modes.contains(&request.destination_mode) {
        diagnostics.push(diagnostic(
            FRONTEND_ARTIFACT_EXPORT_DIAG_UNSUPPORTED_DESTINATION_MODE,
            "destination_mode",
            format!("unsupported frontend artifact export destination mode '{}'", request.destination_mode),
        ));
    }
    if !request.no_hidden_fallback {
        diagnostics.push(diagnostic(
            FRONTEND_ARTIFACT_EXPORT_DIAG_HIDDEN_FALLBACK,
            "no_hidden_fallback",
            "frontend artifact export must prove that no hidden Nix runtime fallback was used",
        ));
    }
}

fn validate_attestation(
    request: &FrontendArtifactExportPreflightRequest<'_>,
    diagnostics: &mut Vec<FrontendArtifactExportDiagnostic>,
) {
    debug_assert!(!FRONTEND_ARTIFACT_EXPORT_DIAG_MISSING_ADMISSION_PROOF.is_empty());
    debug_assert!(!FRONTEND_ARTIFACT_VALIDATION_RESULT_ADMITTED.is_empty());
    let Some(attestation) = request.attestation else {
        diagnostics.push(diagnostic(
            FRONTEND_ARTIFACT_EXPORT_DIAG_MISSING_ADMISSION_PROOF,
            "attestation",
            "frontend artifact export requires successful spec-admission attestation",
        ));
        return;
    };

    require_non_empty(&attestation.spec_id, "attestation.spec_id", diagnostics);
    require_non_empty(&attestation.spec_version, "attestation.spec_version", diagnostics);
    require_non_empty(&attestation.spec_hash, "attestation.spec_hash", diagnostics);
    require_non_empty(&attestation.validator_kind, "attestation.validator_kind", diagnostics);
    require_non_empty(&attestation.validator_ref, "attestation.validator_ref", diagnostics);
    require_non_empty(&attestation.artifact_ref, "attestation.artifact_ref", diagnostics);
    require_non_empty(&attestation.artifact_kind, "attestation.artifact_kind", diagnostics);
    require_non_empty(&attestation.build_root, "attestation.build_root", diagnostics);
    validate_expected_spec_digest(Some(&attestation.spec_hash), "attestation.spec_hash", diagnostics);
    validate_expected_artifact_digest(
        attestation.artifact_digest.as_deref(),
        "attestation.artifact_digest",
        diagnostics,
    );

    require_equal(
        attestation.artifact_ref.as_str(),
        request.expectation.artifact_ref,
        "attestation.artifact_ref",
        "attestation artifact ref does not match requested artifact ref",
        diagnostics,
    );
    require_optional_equal(
        request.expectation.spec_id,
        &attestation.spec_id,
        "attestation.spec_id",
        "attestation spec id does not match requested spec id",
        diagnostics,
    );
    require_optional_equal(
        request.expectation.spec_version,
        &attestation.spec_version,
        "attestation.spec_version",
        "attestation spec version does not match requested spec version",
        diagnostics,
    );
    require_optional_equal(
        request.expectation.spec_hash,
        &attestation.spec_hash,
        "attestation.spec_hash",
        "attestation spec hash does not match requested spec hash",
        diagnostics,
    );

    if attestation.validation_result != FRONTEND_ARTIFACT_VALIDATION_RESULT_ADMITTED {
        diagnostics.push(diagnostic(
            FRONTEND_ARTIFACT_EXPORT_DIAG_MISSING_ADMISSION_PROOF,
            "attestation.validation_result",
            "frontend artifact export requires an admitted validation result",
        ));
    }
    if !attestation.no_hidden_fallback {
        diagnostics.push(diagnostic(
            FRONTEND_ARTIFACT_EXPORT_DIAG_HIDDEN_FALLBACK,
            "attestation.no_hidden_fallback",
            "frontend artifact admission attestation must prove that no hidden fallback produced the artifact",
        ));
    }
}

fn validate_content(
    request: &FrontendArtifactExportRequest<'_>,
    diagnostics: &mut Vec<FrontendArtifactExportDiagnostic>,
) {
    debug_assert!(!FRONTEND_ARTIFACT_EXPORT_DIAG_CONTENT_UNAVAILABLE.is_empty());
    debug_assert!(!FRONTEND_ARTIFACT_EXPORT_DIAG_DIGEST_MISMATCH.is_empty());
    let Some(content) = request.content else {
        diagnostics.push(diagnostic(
            FRONTEND_ARTIFACT_EXPORT_DIAG_CONTENT_UNAVAILABLE,
            "content",
            "frontend artifact export content is unavailable",
        ));
        return;
    };

    require_non_empty(&content.artifact_ref, "content.artifact_ref", diagnostics);
    require_non_empty(&content.artifact_digest, "content.artifact_digest", diagnostics);
    require_non_empty(&content.materialized_path, "content.materialized_path", diagnostics);
    validate_required_artifact_digest(&content.artifact_digest, "content.artifact_digest", diagnostics);
    require_equal(
        &content.artifact_ref,
        request.preflight.expectation.artifact_ref,
        "content.artifact_ref",
        "exported content artifact ref does not match requested artifact ref",
        diagnostics,
    );
    require_optional_digest_equal(
        request.preflight.expectation.artifact_digest,
        &content.artifact_digest,
        "content.artifact_digest",
        "exported content digest does not match requested artifact digest",
        diagnostics,
    );
    if let Some(attestation) = request.preflight.attestation {
        require_optional_digest_equal(
            attestation.artifact_digest.as_deref(),
            &content.artifact_digest,
            "content.artifact_digest",
            "exported content digest does not match admission attestation digest",
            diagnostics,
        );
    }
}

fn receipt_from_validated(
    request: &FrontendArtifactExportRequest<'_>,
    attestation: &FrontendArtifactAdmissionAttestation,
    content: &FrontendArtifactExportContent,
) -> FrontendArtifactExportReceipt {
    debug_assert!(request.preflight.no_hidden_fallback);
    debug_assert_eq!(attestation.validation_result, FRONTEND_ARTIFACT_VALIDATION_RESULT_ADMITTED);
    let material = FrontendArtifactExportReceiptMaterial {
        schema: FRONTEND_ARTIFACT_EXPORT_RECEIPT_SCHEMA.to_string(),
        artifact_ref: request.preflight.expectation.artifact_ref.to_string(),
        artifact_digest: content.artifact_digest.clone(),
        materialized_path: content.materialized_path.clone(),
        destination_mode: request.preflight.destination_mode.to_string(),
        spec_id: attestation.spec_id.clone(),
        spec_version: attestation.spec_version.clone(),
        spec_hash: attestation.spec_hash.clone(),
        validator_kind: attestation.validator_kind.clone(),
        validator_ref: attestation.validator_ref.clone(),
        validation_result: attestation.validation_result.clone(),
        artifact_kind: attestation.artifact_kind.clone(),
        build_root: attestation.build_root.clone(),
        content_provenance: content.provenance.clone(),
        no_hidden_fallback: true,
    };
    let receipt_hash = export_receipt_hash(&material);
    FrontendArtifactExportReceipt {
        schema: material.schema,
        artifact_ref: material.artifact_ref,
        artifact_digest: material.artifact_digest,
        materialized_path: material.materialized_path,
        destination_mode: material.destination_mode,
        spec_id: material.spec_id,
        spec_version: material.spec_version,
        spec_hash: material.spec_hash,
        validator_kind: material.validator_kind,
        validator_ref: material.validator_ref,
        validation_result: material.validation_result,
        artifact_kind: material.artifact_kind,
        build_root: material.build_root,
        content_provenance: material.content_provenance,
        no_hidden_fallback: material.no_hidden_fallback,
        receipt_hash_algorithm: FRONTEND_ARTIFACT_EXPORT_HASH_ALGORITHM_BLAKE3.to_string(),
        receipt_hash,
    }
}

fn export_receipt_hash(material: &FrontendArtifactExportReceiptMaterial) -> String {
    let preimage = export_receipt_preimage(material);
    blake3::hash(preimage.as_bytes()).to_hex().to_string()
}

fn export_receipt_preimage(material: &FrontendArtifactExportReceiptMaterial) -> String {
    debug_assert!(!RECEIPT_PREIMAGE_VERSION.is_empty());
    debug_assert!(!RECEIPT_FIELD_SEPARATOR.is_empty());
    let mut preimage = String::new();
    append_receipt_field(&mut preimage, "preimage_version", RECEIPT_PREIMAGE_VERSION);
    append_receipt_field(&mut preimage, "schema", &material.schema);
    append_receipt_field(&mut preimage, "artifact_ref", &material.artifact_ref);
    append_receipt_field(&mut preimage, "artifact_digest", &material.artifact_digest);
    append_receipt_field(&mut preimage, "materialized_path", &material.materialized_path);
    append_receipt_field(&mut preimage, "destination_mode", &material.destination_mode);
    append_receipt_field(&mut preimage, "spec_id", &material.spec_id);
    append_receipt_field(&mut preimage, "spec_version", &material.spec_version);
    append_receipt_field(&mut preimage, "spec_hash", &material.spec_hash);
    append_receipt_field(&mut preimage, "validator_kind", &material.validator_kind);
    append_receipt_field(&mut preimage, "validator_ref", &material.validator_ref);
    append_receipt_field(&mut preimage, "validation_result", &material.validation_result);
    append_receipt_field(&mut preimage, "artifact_kind", &material.artifact_kind);
    append_receipt_field(&mut preimage, "build_root", &material.build_root);
    append_receipt_field(
        &mut preimage,
        "no_hidden_fallback",
        if material.no_hidden_fallback { "true" } else { "false" },
    );
    for (key, value) in &material.content_provenance {
        append_receipt_field(&mut preimage, format!("content_provenance.{key}"), value);
    }
    debug_assert!(preimage.contains(RECEIPT_PREIMAGE_VERSION));
    debug_assert!(!preimage.is_empty());
    preimage
}

fn append_receipt_field(preimage: &mut String, key: impl AsRef<str>, value: &str) {
    preimage.push_str(key.as_ref());
    preimage.push_str(RECEIPT_FIELD_SEPARATOR);
    preimage.push_str(value);
    preimage.push_str(RECEIPT_RECORD_SEPARATOR);
}

fn report(
    receipt: Option<FrontendArtifactExportReceipt>,
    diagnostics: Vec<FrontendArtifactExportDiagnostic>,
) -> FrontendArtifactExportReport {
    FrontendArtifactExportReport {
        exported: receipt.is_some(),
        receipt,
        diagnostics,
    }
}

fn require_non_empty(value: &str, path: impl AsRef<str>, diagnostics: &mut Vec<FrontendArtifactExportDiagnostic>) {
    let path = path.as_ref();
    if value.is_empty() {
        diagnostics.push(diagnostic(
            FRONTEND_ARTIFACT_EXPORT_DIAG_EMPTY_FIELD,
            path,
            format!("frontend artifact export field '{path}' must be non-empty"),
        ));
    }
}

fn require_equal(
    actual: &str,
    expected: impl AsRef<str>,
    path: &'static str,
    message: impl AsRef<str>,
    diagnostics: &mut Vec<FrontendArtifactExportDiagnostic>,
) {
    if actual != expected.as_ref() {
        diagnostics.push(diagnostic(FRONTEND_ARTIFACT_EXPORT_DIAG_PROOF_MISMATCH, path, message.as_ref()));
    }
}

fn require_optional_equal(
    expected: Option<&str>,
    actual: impl AsRef<str>,
    path: &'static str,
    message: impl AsRef<str>,
    diagnostics: &mut Vec<FrontendArtifactExportDiagnostic>,
) {
    let Some(expected) = expected else {
        return;
    };
    if actual.as_ref() != expected {
        diagnostics.push(diagnostic(FRONTEND_ARTIFACT_EXPORT_DIAG_PROOF_MISMATCH, path, message.as_ref()));
    }
}

fn require_optional_digest_equal(
    expected: Option<&str>,
    actual: impl AsRef<str>,
    path: &'static str,
    message: impl AsRef<str>,
    diagnostics: &mut Vec<FrontendArtifactExportDiagnostic>,
) {
    let Some(expected) = expected else {
        return;
    };
    if actual.as_ref() != expected {
        diagnostics.push(diagnostic(FRONTEND_ARTIFACT_EXPORT_DIAG_DIGEST_MISMATCH, path, message.as_ref()));
    }
}

fn validate_expected_spec_digest(
    digest: Option<&str>,
    path: &'static str,
    diagnostics: &mut Vec<FrontendArtifactExportDiagnostic>,
) {
    let Some(digest) = digest else {
        return;
    };
    if !is_blake3_hex(digest) {
        diagnostics.push(diagnostic(
            FRONTEND_ARTIFACT_EXPORT_DIAG_INVALID_DIGEST,
            path,
            "frontend artifact export spec hash must be a lowercase BLAKE3 hex digest",
        ));
    }
}

fn validate_expected_artifact_digest(
    digest: Option<&str>,
    path: &'static str,
    diagnostics: &mut Vec<FrontendArtifactExportDiagnostic>,
) {
    let Some(digest) = digest else {
        return;
    };
    validate_required_artifact_digest(digest, path, diagnostics);
}

fn validate_required_artifact_digest(
    digest: &str,
    path: impl AsRef<str>,
    diagnostics: &mut Vec<FrontendArtifactExportDiagnostic>,
) {
    let path = path.as_ref();
    if !is_blake3_prefixed_digest(digest) {
        diagnostics.push(diagnostic(
            FRONTEND_ARTIFACT_EXPORT_DIAG_INVALID_DIGEST,
            path,
            "frontend artifact export artifact digest must be blake3:<lowercase-hex>",
        ));
    }
}

fn is_blake3_prefixed_digest(value: &str) -> bool {
    let Some(hex) = value.strip_prefix(FRONTEND_ARTIFACT_EXPORT_DIGEST_PREFIX_BLAKE3) else {
        return false;
    };
    is_blake3_hex(hex)
}

fn is_blake3_hex(value: &str) -> bool {
    value.len() == BLAKE3_HEX_LENGTH && value.chars().all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_uppercase())
}

fn diagnostic(
    code: &'static str,
    path: impl Into<String>,
    message: impl Into<String>,
) -> FrontendArtifactExportDiagnostic {
    FrontendArtifactExportDiagnostic {
        code: code.to_string(),
        path: path.into(),
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend_artifact_spec::FRONTEND_ARTIFACT_HASH_ALGORITHM_BLAKE3;
    use crate::frontend_artifact_spec::FRONTEND_ARTIFACT_VALIDATOR_KIND_ALLOWLIST_V1;

    const SAMPLE_SPEC_ID: &str = "example.activation";
    const SAMPLE_SPEC_VERSION: &str = "1";
    const SAMPLE_SPEC_HASH_MATERIAL: &[u8] = b"sample spec";
    const SAMPLE_VALIDATOR_REF: &str = "mantle://blake3/spec-validator";
    const SAMPLE_ARTIFACT_KIND: &str = "example-activation-closure";
    const SAMPLE_ONIX_ARTIFACT_KIND: &str = "mantle-onix-activation-closure";
    const SAMPLE_ARTIFACT_REF: &str = "mantle://blake3/artifact";
    const OTHER_ARTIFACT_REF: &str = "mantle://blake3/other-artifact";
    const SAMPLE_BUILD_ROOT: &str = "drv:demo";
    const SAMPLE_MATERIALIZED_PATH: &str = "/tmp/mantle-export/artifact";
    const SAMPLE_PROVENANCE_KEY: &str = "build_report";
    const SAMPLE_PROVENANCE_VALUE: &str = "report-b3";
    const INVALID_NIX_STORE_REF: &str = "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-demo";

    fn sample_spec_hash() -> String {
        blake3::hash(SAMPLE_SPEC_HASH_MATERIAL).to_hex().to_string()
    }

    fn sample_artifact_digest() -> String {
        format!("{}{}", FRONTEND_ARTIFACT_EXPORT_DIGEST_PREFIX_BLAKE3, blake3::hash(b"artifact content").to_hex())
    }

    fn other_artifact_digest() -> String {
        format!(
            "{}{}",
            FRONTEND_ARTIFACT_EXPORT_DIGEST_PREFIX_BLAKE3,
            blake3::hash(b"other artifact content").to_hex()
        )
    }

    fn sample_attestation(artifact_kind: &str) -> FrontendArtifactAdmissionAttestation {
        FrontendArtifactAdmissionAttestation {
            spec_id: SAMPLE_SPEC_ID.to_string(),
            spec_version: SAMPLE_SPEC_VERSION.to_string(),
            spec_hash_algorithm: FRONTEND_ARTIFACT_HASH_ALGORITHM_BLAKE3.to_string(),
            spec_hash: sample_spec_hash(),
            validator_kind: FRONTEND_ARTIFACT_VALIDATOR_KIND_ALLOWLIST_V1.to_string(),
            validator_ref: SAMPLE_VALIDATOR_REF.to_string(),
            artifact_kind: artifact_kind.to_string(),
            artifact_ref: SAMPLE_ARTIFACT_REF.to_string(),
            artifact_digest: Some(sample_artifact_digest()),
            target_identity: None,
            build_root: SAMPLE_BUILD_ROOT.to_string(),
            validation_result: FRONTEND_ARTIFACT_VALIDATION_RESULT_ADMITTED.to_string(),
            no_hidden_fallback: true,
        }
    }

    fn sample_expectation<'a>(artifact_digest: &'a str, spec_hash: &'a str) -> FrontendArtifactExportExpectation<'a> {
        FrontendArtifactExportExpectation {
            artifact_ref: SAMPLE_ARTIFACT_REF,
            artifact_digest: Some(artifact_digest),
            spec_id: Some(SAMPLE_SPEC_ID),
            spec_version: Some(SAMPLE_SPEC_VERSION),
            spec_hash: Some(spec_hash),
        }
    }

    fn sample_content() -> FrontendArtifactExportContent {
        FrontendArtifactExportContent {
            artifact_ref: SAMPLE_ARTIFACT_REF.to_string(),
            artifact_digest: sample_artifact_digest(),
            materialized_path: SAMPLE_MATERIALIZED_PATH.to_string(),
            provenance: BTreeMap::from([(SAMPLE_PROVENANCE_KEY.to_string(), SAMPLE_PROVENANCE_VALUE.to_string())]),
        }
    }

    fn sample_request<'a>(
        expectation: FrontendArtifactExportExpectation<'a>,
        attestation: Option<&'a FrontendArtifactAdmissionAttestation>,
        content: Option<&'a FrontendArtifactExportContent>,
    ) -> FrontendArtifactExportRequest<'a> {
        FrontendArtifactExportRequest {
            preflight: FrontendArtifactExportPreflightRequest {
                expectation,
                attestation,
                destination_mode: FRONTEND_ARTIFACT_EXPORT_MODE_DIRECTORY,
                supported_destination_modes: &[
                    FRONTEND_ARTIFACT_EXPORT_MODE_DIRECTORY,
                    FRONTEND_ARTIFACT_EXPORT_MODE_ARCHIVE,
                    FRONTEND_ARTIFACT_EXPORT_MODE_STREAM,
                ],
                no_hidden_fallback: true,
            },
            content,
        }
    }

    #[test]
    fn valid_admitted_artifact_exports_with_receipt() {
        let artifact_digest = sample_artifact_digest();
        let spec_hash = sample_spec_hash();
        let attestation = sample_attestation(SAMPLE_ARTIFACT_KIND);
        let content = sample_content();
        let expectation = sample_expectation(&artifact_digest, &spec_hash);

        let report = export_frontend_artifact(&sample_request(expectation, Some(&attestation), Some(&content)));

        assert!(report.exported, "{:#?}", report.diagnostics);
        assert!(report.diagnostics.is_empty());
        let receipt = report.receipt.expect("export receipt");
        assert_eq!(receipt.schema, FRONTEND_ARTIFACT_EXPORT_RECEIPT_SCHEMA);
        assert_eq!(receipt.artifact_ref, SAMPLE_ARTIFACT_REF);
        assert_eq!(receipt.artifact_digest, artifact_digest);
        assert_eq!(receipt.materialized_path, SAMPLE_MATERIALIZED_PATH);
        assert_eq!(receipt.destination_mode, FRONTEND_ARTIFACT_EXPORT_MODE_DIRECTORY);
        assert_eq!(receipt.spec_id, SAMPLE_SPEC_ID);
        assert_eq!(receipt.spec_version, SAMPLE_SPEC_VERSION);
        assert_eq!(receipt.spec_hash, spec_hash);
        assert_eq!(receipt.validation_result, FRONTEND_ARTIFACT_VALIDATION_RESULT_ADMITTED);
        assert_eq!(receipt.artifact_kind, SAMPLE_ARTIFACT_KIND);
        assert_eq!(receipt.content_provenance[SAMPLE_PROVENANCE_KEY], SAMPLE_PROVENANCE_VALUE);
        assert_eq!(receipt.receipt_hash_algorithm, FRONTEND_ARTIFACT_EXPORT_HASH_ALGORITHM_BLAKE3);
        assert!(is_blake3_hex(&receipt.receipt_hash));
        assert!(receipt.no_hidden_fallback);
    }

    #[test]
    fn missing_attestation_fails_before_export() {
        let artifact_digest = sample_artifact_digest();
        let spec_hash = sample_spec_hash();
        let content = sample_content();
        let expectation = sample_expectation(&artifact_digest, &spec_hash);

        let report = export_frontend_artifact(&sample_request(expectation, None, Some(&content)));

        assert!(!report.exported);
        assert!(report.receipt.is_none());
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diag| diag.code == FRONTEND_ARTIFACT_EXPORT_DIAG_MISSING_ADMISSION_PROOF)
        );
    }

    #[test]
    fn unsupported_ref_scheme_fails_closed() {
        let artifact_digest = sample_artifact_digest();
        let spec_hash = sample_spec_hash();
        let attestation = sample_attestation(SAMPLE_ARTIFACT_KIND);
        let content = sample_content();
        let expectation = FrontendArtifactExportExpectation {
            artifact_ref: INVALID_NIX_STORE_REF,
            artifact_digest: Some(&artifact_digest),
            spec_id: Some(SAMPLE_SPEC_ID),
            spec_version: Some(SAMPLE_SPEC_VERSION),
            spec_hash: Some(&spec_hash),
        };

        let report = export_frontend_artifact(&sample_request(expectation, Some(&attestation), Some(&content)));

        assert!(!report.exported);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diag| diag.code == FRONTEND_ARTIFACT_EXPORT_DIAG_UNSUPPORTED_REF_SCHEME)
        );
    }

    #[test]
    fn wrong_artifact_ref_fails_closed() {
        let artifact_digest = sample_artifact_digest();
        let spec_hash = sample_spec_hash();
        let attestation = sample_attestation(SAMPLE_ARTIFACT_KIND);
        let content = sample_content();
        let expectation = FrontendArtifactExportExpectation {
            artifact_ref: OTHER_ARTIFACT_REF,
            artifact_digest: Some(&artifact_digest),
            spec_id: Some(SAMPLE_SPEC_ID),
            spec_version: Some(SAMPLE_SPEC_VERSION),
            spec_hash: Some(&spec_hash),
        };

        let report = export_frontend_artifact(&sample_request(expectation, Some(&attestation), Some(&content)));

        assert!(!report.exported);
        assert!(report.diagnostics.iter().any(|diag| diag.code == FRONTEND_ARTIFACT_EXPORT_DIAG_PROOF_MISMATCH));
    }

    #[test]
    fn digest_mismatch_fails_closed() {
        let artifact_digest = other_artifact_digest();
        let spec_hash = sample_spec_hash();
        let attestation = sample_attestation(SAMPLE_ARTIFACT_KIND);
        let content = sample_content();
        let expectation = sample_expectation(&artifact_digest, &spec_hash);

        let report = export_frontend_artifact(&sample_request(expectation, Some(&attestation), Some(&content)));

        assert!(!report.exported);
        assert!(report.diagnostics.iter().any(|diag| diag.code == FRONTEND_ARTIFACT_EXPORT_DIAG_DIGEST_MISMATCH));
    }

    #[test]
    fn missing_content_fails_closed() {
        let artifact_digest = sample_artifact_digest();
        let spec_hash = sample_spec_hash();
        let attestation = sample_attestation(SAMPLE_ARTIFACT_KIND);
        let expectation = sample_expectation(&artifact_digest, &spec_hash);

        let report = export_frontend_artifact(&sample_request(expectation, Some(&attestation), None));

        assert!(!report.exported);
        assert!(report.diagnostics.iter().any(|diag| diag.code == FRONTEND_ARTIFACT_EXPORT_DIAG_CONTENT_UNAVAILABLE));
    }

    #[test]
    fn hidden_fallback_marker_fails_closed() {
        let artifact_digest = sample_artifact_digest();
        let spec_hash = sample_spec_hash();
        let attestation = sample_attestation(SAMPLE_ARTIFACT_KIND);
        let content = sample_content();
        let expectation = sample_expectation(&artifact_digest, &spec_hash);
        let mut request = sample_request(expectation, Some(&attestation), Some(&content));
        request.preflight.no_hidden_fallback = false;

        let report = export_frontend_artifact(&request);

        assert!(!report.exported);
        assert!(report.diagnostics.iter().any(|diag| diag.code == FRONTEND_ARTIFACT_EXPORT_DIAG_HIDDEN_FALLBACK));
    }

    #[test]
    fn onix_like_kind_remains_opaque_when_attested() {
        let artifact_digest = sample_artifact_digest();
        let spec_hash = sample_spec_hash();
        let attestation = sample_attestation(SAMPLE_ONIX_ARTIFACT_KIND);
        let content = sample_content();
        let expectation = sample_expectation(&artifact_digest, &spec_hash);

        let report = export_frontend_artifact(&sample_request(expectation, Some(&attestation), Some(&content)));

        assert!(report.exported, "{:#?}", report.diagnostics);
        let receipt = report.receipt.expect("export receipt");
        assert_eq!(receipt.artifact_kind, SAMPLE_ONIX_ARTIFACT_KIND);
        assert_eq!(receipt.spec_id, SAMPLE_SPEC_ID);
    }

    #[test]
    fn export_receipt_serializes_receipt_hash() {
        let artifact_digest = sample_artifact_digest();
        let spec_hash = sample_spec_hash();
        let attestation = sample_attestation(SAMPLE_ARTIFACT_KIND);
        let content = sample_content();
        let expectation = sample_expectation(&artifact_digest, &spec_hash);
        let report = export_frontend_artifact(&sample_request(expectation, Some(&attestation), Some(&content)));
        let receipt = report.receipt.expect("export receipt");

        let rendered = render_frontend_artifact_export_receipt(&receipt).expect("receipt JSON serializes");
        let value: serde_json::Value = serde_json::from_str(&rendered).expect("receipt JSON parses");

        assert_eq!(value["schema"], FRONTEND_ARTIFACT_EXPORT_RECEIPT_SCHEMA);
        assert_eq!(value["artifact_ref"], SAMPLE_ARTIFACT_REF);
        assert_eq!(value["receipt_hash_algorithm"], FRONTEND_ARTIFACT_EXPORT_HASH_ALGORITHM_BLAKE3);
        assert_eq!(value["receipt_hash"], receipt.receipt_hash);
    }
}
