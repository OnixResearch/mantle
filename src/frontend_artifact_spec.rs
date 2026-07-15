#![allow(dead_code)]

use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

pub const FRONTEND_ARTIFACT_VALIDATION_RESULT_ADMITTED: &str = "admitted";
pub const FRONTEND_ARTIFACT_HASH_ALGORITHM_BLAKE3: &str = "blake3";
pub const FRONTEND_ARTIFACT_VALIDATOR_KIND_ALLOWLIST_V1: &str = "manifest-kind-allowlist-v1";
pub const FRONTEND_ARTIFACT_KIND_ALLOWLIST_SCHEMA_V1: &str = "mantle-frontend-artifact-kind-allowlist-v1";
pub const FRONTEND_ARTIFACT_ADMISSION_SIDECAR_SCHEMA: &str = "mantle-frontend-artifact-admission-v1";
pub const FRONTEND_ARTIFACT_DIAG_MISSING_SPEC: &str = "frontend-artifact-missing-spec";
pub const FRONTEND_ARTIFACT_DIAG_EMPTY_FIELD: &str = "frontend-artifact-empty-field";
pub const FRONTEND_ARTIFACT_DIAG_UNSUPPORTED_HASH: &str = "frontend-artifact-unsupported-hash";
pub const FRONTEND_ARTIFACT_DIAG_INVALID_HASH: &str = "frontend-artifact-invalid-hash";
pub const FRONTEND_ARTIFACT_DIAG_HASH_MISMATCH: &str = "frontend-artifact-hash-mismatch";
pub const FRONTEND_ARTIFACT_DIAG_UNSUPPORTED_VALIDATOR: &str = "frontend-artifact-unsupported-validator";
pub const FRONTEND_ARTIFACT_DIAG_SPEC_BINDING_MISMATCH: &str = "frontend-artifact-spec-binding-mismatch";
pub const FRONTEND_ARTIFACT_DIAG_HIDDEN_FALLBACK: &str = "frontend-artifact-hidden-fallback";
pub const FRONTEND_ARTIFACT_DIAG_INVALID_SPEC_MATERIAL: &str = "frontend-artifact-invalid-spec-material";
pub const FRONTEND_ARTIFACT_DIAG_KIND_NOT_ALLOWED: &str = "frontend-artifact-kind-not-allowed";

const HEX_CHARS_PER_BYTE: usize = 2;
const BLAKE3_HEX_LENGTH: usize = blake3::OUT_LEN.saturating_mul(HEX_CHARS_PER_BYTE);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrontendArtifactSpecRef {
    pub id: String,
    pub version: String,
    pub validator_kind: String,
    pub validator_ref: String,
    pub hash_algorithm: String,
    pub hash: String,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrontendArtifactManifest {
    pub kind: String,
    pub artifact_ref: String,
    pub artifact_digest: Option<String>,
    pub target_identity: Option<String>,
    pub spec_id: String,
    pub spec_version: String,
    pub spec_hash: String,
    pub no_hidden_fallback: bool,
    pub provenance: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontendArtifactAdmissionRequest<'a> {
    pub spec: Option<&'a FrontendArtifactSpecRef>,
    pub manifest: &'a FrontendArtifactManifest,
    pub spec_material: &'a [u8],
    pub supported_validator_kinds: &'a [&'a str],
    pub build_root: &'a str,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrontendArtifactAdmissionDiagnostic {
    pub code: String,
    pub path: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrontendArtifactAdmissionAttestation {
    pub spec_id: String,
    pub spec_version: String,
    pub spec_hash_algorithm: String,
    pub spec_hash: String,
    pub validator_kind: String,
    pub validator_ref: String,
    pub artifact_kind: String,
    pub artifact_ref: String,
    pub artifact_digest: Option<String>,
    pub target_identity: Option<String>,
    pub build_root: String,
    pub validation_result: String,
    pub no_hidden_fallback: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrontendArtifactAdmissionSidecar {
    pub schema: &'static str,
    pub attestation: FrontendArtifactAdmissionAttestation,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
struct FrontendArtifactKindAllowlistSpec {
    schema: String,
    allowed_kinds: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrontendArtifactAdmissionReport {
    pub admitted: bool,
    pub attestation: Option<FrontendArtifactAdmissionAttestation>,
    pub diagnostics: Vec<FrontendArtifactAdmissionDiagnostic>,
}

pub fn admit_frontend_artifact(request: &FrontendArtifactAdmissionRequest<'_>) -> FrontendArtifactAdmissionReport {
    debug_assert!(!FRONTEND_ARTIFACT_ADMISSION_SIDECAR_SCHEMA.is_empty());
    debug_assert_eq!(BLAKE3_HEX_LENGTH, blake3::OUT_LEN.saturating_mul(HEX_CHARS_PER_BYTE));
    let mut diagnostics = Vec::new();
    let Some(spec) = request.spec else {
        diagnostics.push(diagnostic(
            FRONTEND_ARTIFACT_DIAG_MISSING_SPEC,
            "spec",
            "frontend artifact admission requires a spec reference",
        ));
        return report(None, diagnostics);
    };

    validate_spec_ref(spec, request.spec_material, request.supported_validator_kinds, &mut diagnostics);
    validate_manifest_binding(spec, request.manifest, &mut diagnostics);
    if diagnostics.is_empty() {
        execute_declared_validator(spec, request.manifest, request.spec_material, &mut diagnostics);
    }

    if diagnostics.is_empty() {
        return report(
            Some(FrontendArtifactAdmissionAttestation {
                spec_id: spec.id.clone(),
                spec_version: spec.version.clone(),
                spec_hash_algorithm: spec.hash_algorithm.clone(),
                spec_hash: spec.hash.clone(),
                validator_kind: spec.validator_kind.clone(),
                validator_ref: spec.validator_ref.clone(),
                artifact_kind: request.manifest.kind.clone(),
                artifact_ref: request.manifest.artifact_ref.clone(),
                artifact_digest: request.manifest.artifact_digest.clone(),
                target_identity: request.manifest.target_identity.clone(),
                build_root: request.build_root.to_string(),
                validation_result: FRONTEND_ARTIFACT_VALIDATION_RESULT_ADMITTED.to_string(),
                no_hidden_fallback: request.manifest.no_hidden_fallback,
            }),
            diagnostics,
        );
    }

    report(None, diagnostics)
}

pub fn render_frontend_artifact_admission_sidecar(
    attestation: FrontendArtifactAdmissionAttestation,
) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(&FrontendArtifactAdmissionSidecar {
        schema: FRONTEND_ARTIFACT_ADMISSION_SIDECAR_SCHEMA,
        attestation,
    })
}

fn report(
    attestation: Option<FrontendArtifactAdmissionAttestation>,
    diagnostics: Vec<FrontendArtifactAdmissionDiagnostic>,
) -> FrontendArtifactAdmissionReport {
    FrontendArtifactAdmissionReport {
        admitted: attestation.is_some(),
        attestation,
        diagnostics,
    }
}

fn validate_spec_ref(
    spec: &FrontendArtifactSpecRef,
    spec_material: &[u8],
    supported_validator_kinds: &[&str],
    diagnostics: &mut Vec<FrontendArtifactAdmissionDiagnostic>,
) {
    require_non_empty(&spec.id, "spec.id", diagnostics);
    require_non_empty(&spec.version, "spec.version", diagnostics);
    require_non_empty(&spec.validator_kind, "spec.validator_kind", diagnostics);
    require_non_empty(&spec.validator_ref, "spec.validator_ref", diagnostics);
    require_non_empty(&spec.hash_algorithm, "spec.hash_algorithm", diagnostics);
    require_non_empty(&spec.hash, "spec.hash", diagnostics);

    if !supported_validator_kinds.iter().any(|kind| *kind == spec.validator_kind) {
        diagnostics.push(diagnostic(
            FRONTEND_ARTIFACT_DIAG_UNSUPPORTED_VALIDATOR,
            "spec.validator_kind",
            format!("unsupported frontend artifact validator kind '{}'", spec.validator_kind),
        ));
    }

    validate_blake3_hash(spec, spec_material, diagnostics);
}

fn validate_blake3_hash(
    spec: &FrontendArtifactSpecRef,
    spec_material: &[u8],
    diagnostics: &mut Vec<FrontendArtifactAdmissionDiagnostic>,
) {
    debug_assert!(!FRONTEND_ARTIFACT_HASH_ALGORITHM_BLAKE3.is_empty());
    debug_assert_eq!(BLAKE3_HEX_LENGTH, blake3::OUT_LEN.saturating_mul(HEX_CHARS_PER_BYTE));
    if spec.hash_algorithm != FRONTEND_ARTIFACT_HASH_ALGORITHM_BLAKE3 {
        diagnostics.push(diagnostic(
            FRONTEND_ARTIFACT_DIAG_UNSUPPORTED_HASH,
            "spec.hash_algorithm",
            format!("unsupported frontend artifact spec hash algorithm '{}', expected blake3", spec.hash_algorithm),
        ));
        return;
    }
    if !is_blake3_hex(&spec.hash) {
        diagnostics.push(diagnostic(
            FRONTEND_ARTIFACT_DIAG_INVALID_HASH,
            "spec.hash",
            "frontend artifact spec hash must be a lowercase BLAKE3 hex digest",
        ));
        return;
    }
    let actual = blake3::hash(spec_material).to_hex().to_string();
    if actual != spec.hash {
        diagnostics.push(diagnostic(
            FRONTEND_ARTIFACT_DIAG_HASH_MISMATCH,
            "spec.hash",
            "frontend artifact spec hash does not match supplied spec material",
        ));
    }
}

fn execute_declared_validator(
    spec: &FrontendArtifactSpecRef,
    manifest: &FrontendArtifactManifest,
    spec_material: &[u8],
    diagnostics: &mut Vec<FrontendArtifactAdmissionDiagnostic>,
) {
    match spec.validator_kind.as_str() {
        FRONTEND_ARTIFACT_VALIDATOR_KIND_ALLOWLIST_V1 => {
            execute_kind_allowlist_validator(manifest, spec_material, diagnostics);
        }
        _ => diagnostics.push(diagnostic(
            FRONTEND_ARTIFACT_DIAG_UNSUPPORTED_VALIDATOR,
            "spec.validator_kind",
            format!("unsupported frontend artifact validator executor '{}'", spec.validator_kind),
        )),
    }
}

fn execute_kind_allowlist_validator(
    manifest: &FrontendArtifactManifest,
    spec_material: &[u8],
    diagnostics: &mut Vec<FrontendArtifactAdmissionDiagnostic>,
) {
    debug_assert!(!FRONTEND_ARTIFACT_KIND_ALLOWLIST_SCHEMA_V1.is_empty());
    debug_assert!(!FRONTEND_ARTIFACT_DIAG_KIND_NOT_ALLOWED.is_empty());
    let Ok(spec_material) = serde_json::from_slice::<FrontendArtifactKindAllowlistSpec>(spec_material) else {
        diagnostics.push(diagnostic(
            FRONTEND_ARTIFACT_DIAG_INVALID_SPEC_MATERIAL,
            "spec_material",
            "frontend artifact kind allowlist spec material must be valid JSON",
        ));
        return;
    };
    if spec_material.schema != FRONTEND_ARTIFACT_KIND_ALLOWLIST_SCHEMA_V1 {
        diagnostics.push(diagnostic(
            FRONTEND_ARTIFACT_DIAG_INVALID_SPEC_MATERIAL,
            "spec_material.schema",
            format!("frontend artifact kind allowlist schema must be '{}'", FRONTEND_ARTIFACT_KIND_ALLOWLIST_SCHEMA_V1),
        ));
    }
    if spec_material.allowed_kinds.is_empty() {
        diagnostics.push(diagnostic(
            FRONTEND_ARTIFACT_DIAG_INVALID_SPEC_MATERIAL,
            "spec_material.allowed_kinds",
            "frontend artifact kind allowlist must name at least one artifact kind",
        ));
    }
    if spec_material.allowed_kinds.iter().any(|kind| kind.is_empty()) {
        diagnostics.push(diagnostic(
            FRONTEND_ARTIFACT_DIAG_INVALID_SPEC_MATERIAL,
            "spec_material.allowed_kinds",
            "frontend artifact kind allowlist must not contain an empty kind",
        ));
    }
    if !spec_material.allowed_kinds.iter().any(|kind| kind == &manifest.kind) {
        diagnostics.push(diagnostic(
            FRONTEND_ARTIFACT_DIAG_KIND_NOT_ALLOWED,
            "manifest.kind",
            format!("frontend artifact kind '{}' is not admitted by the declared spec", manifest.kind),
        ));
    }
}

fn validate_manifest_binding(
    spec: &FrontendArtifactSpecRef,
    manifest: &FrontendArtifactManifest,
    diagnostics: &mut Vec<FrontendArtifactAdmissionDiagnostic>,
) {
    debug_assert!(!FRONTEND_ARTIFACT_DIAG_SPEC_BINDING_MISMATCH.is_empty());
    debug_assert!(!FRONTEND_ARTIFACT_DIAG_HIDDEN_FALLBACK.is_empty());
    require_non_empty(&manifest.kind, "manifest.kind", diagnostics);
    require_non_empty(&manifest.artifact_ref, "manifest.artifact_ref", diagnostics);
    require_non_empty(&manifest.spec_id, "manifest.spec_id", diagnostics);
    require_non_empty(&manifest.spec_version, "manifest.spec_version", diagnostics);
    require_non_empty(&manifest.spec_hash, "manifest.spec_hash", diagnostics);

    if manifest.spec_id != spec.id {
        diagnostics.push(diagnostic(
            FRONTEND_ARTIFACT_DIAG_SPEC_BINDING_MISMATCH,
            "manifest.spec_id",
            "artifact manifest spec id does not match admitted spec reference",
        ));
    }
    if manifest.spec_version != spec.version {
        diagnostics.push(diagnostic(
            FRONTEND_ARTIFACT_DIAG_SPEC_BINDING_MISMATCH,
            "manifest.spec_version",
            "artifact manifest spec version does not match admitted spec reference",
        ));
    }
    if manifest.spec_hash != spec.hash {
        diagnostics.push(diagnostic(
            FRONTEND_ARTIFACT_DIAG_SPEC_BINDING_MISMATCH,
            "manifest.spec_hash",
            "artifact manifest spec hash does not match admitted spec reference",
        ));
    }
    if !manifest.no_hidden_fallback {
        diagnostics.push(diagnostic(
            FRONTEND_ARTIFACT_DIAG_HIDDEN_FALLBACK,
            "manifest.no_hidden_fallback",
            "frontend artifact manifest must prove that no hidden fallback produced the artifact",
        ));
    }
}

fn require_non_empty(value: &str, path: impl AsRef<str>, diagnostics: &mut Vec<FrontendArtifactAdmissionDiagnostic>) {
    let path = path.as_ref();
    if value.is_empty() {
        diagnostics.push(diagnostic(
            FRONTEND_ARTIFACT_DIAG_EMPTY_FIELD,
            path,
            format!("frontend artifact admission field '{path}' must be non-empty"),
        ));
    }
}

fn is_blake3_hex(value: &str) -> bool {
    value.len() == BLAKE3_HEX_LENGTH && value.chars().all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_uppercase())
}

fn diagnostic(
    code: &'static str,
    path: impl Into<String>,
    message: impl Into<String>,
) -> FrontendArtifactAdmissionDiagnostic {
    FrontendArtifactAdmissionDiagnostic {
        code: code.to_string(),
        path: path.into(),
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const UNSUPPORTED_VALIDATOR_KIND_WASM: &str = "wasm";
    const SAMPLE_SPEC_ID: &str = "example.activation";
    const SAMPLE_SPEC_VERSION: &str = "1";
    const SAMPLE_VALIDATOR_REF: &str = "mantle://blake3/spec-validator";
    const SAMPLE_ARTIFACT_KIND: &str = "example-activation-closure";
    const SAMPLE_ONIX_ARTIFACT_KIND: &str = "mantle-onix-activation-closure";
    const SAMPLE_NIXOS_ARTIFACT_KIND: &str = "nixos-activation-closure";
    const SAMPLE_ROLE_ARTIFACT_KIND: &str = "onix-service-role";
    const SAMPLE_TAG_ARTIFACT_KIND: &str = "onix-tag";
    const SAMPLE_PROVIDER_ARTIFACT_KIND: &str = "onix-provider";
    const SAMPLE_ARTIFACT_REF: &str = "mantle://blake3/artifact";
    const SAMPLE_TARGET_IDENTITY: &str = "machine:demo";
    const SAMPLE_BUILD_ROOT: &str = "drv:demo";
    const SAMPLE_SPEC_MATERIAL: &[u8] = br#"{"schema":"mantle-frontend-artifact-kind-allowlist-v1","allowed_kinds":["example-activation-closure","mantle-onix-activation-closure","nixos-activation-closure","onix-service-role","onix-tag","onix-provider"]}"#;
    const DISALLOWED_ARTIFACT_KIND: &str = "not-in-spec";
    const BAD_BLAKE3_HEX: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaab";

    fn sample_hash() -> String {
        blake3::hash(SAMPLE_SPEC_MATERIAL).to_hex().to_string()
    }

    fn sample_spec() -> FrontendArtifactSpecRef {
        FrontendArtifactSpecRef {
            id: SAMPLE_SPEC_ID.to_string(),
            version: SAMPLE_SPEC_VERSION.to_string(),
            validator_kind: FRONTEND_ARTIFACT_VALIDATOR_KIND_ALLOWLIST_V1.to_string(),
            validator_ref: SAMPLE_VALIDATOR_REF.to_string(),
            hash_algorithm: FRONTEND_ARTIFACT_HASH_ALGORITHM_BLAKE3.to_string(),
            hash: sample_hash(),
            metadata: BTreeMap::new(),
        }
    }

    fn sample_manifest(spec: &FrontendArtifactSpecRef) -> FrontendArtifactManifest {
        FrontendArtifactManifest {
            kind: SAMPLE_ARTIFACT_KIND.to_string(),
            artifact_ref: SAMPLE_ARTIFACT_REF.to_string(),
            artifact_digest: Some("blake3:artifact-digest".to_string()),
            target_identity: Some(SAMPLE_TARGET_IDENTITY.to_string()),
            spec_id: spec.id.clone(),
            spec_version: spec.version.clone(),
            spec_hash: spec.hash.clone(),
            no_hidden_fallback: true,
            provenance: BTreeMap::from([("builder".to_string(), "mantle".to_string())]),
        }
    }

    fn sample_request<'a>(
        spec: Option<&'a FrontendArtifactSpecRef>,
        manifest: &'a FrontendArtifactManifest,
    ) -> FrontendArtifactAdmissionRequest<'a> {
        FrontendArtifactAdmissionRequest {
            spec,
            manifest,
            spec_material: SAMPLE_SPEC_MATERIAL,
            supported_validator_kinds: &[FRONTEND_ARTIFACT_VALIDATOR_KIND_ALLOWLIST_V1],
            build_root: SAMPLE_BUILD_ROOT,
        }
    }

    #[test]
    fn valid_frontend_artifact_spec_admits_manifest() {
        let spec = sample_spec();
        let manifest = sample_manifest(&spec);

        let report = admit_frontend_artifact(&sample_request(Some(&spec), &manifest));

        assert!(report.admitted, "{:#?}", report.diagnostics);
        assert!(report.diagnostics.is_empty());
        let attestation = report.attestation.expect("admission attestation");
        assert_eq!(attestation.spec_id, SAMPLE_SPEC_ID);
        assert_eq!(attestation.spec_version, SAMPLE_SPEC_VERSION);
        assert_eq!(attestation.spec_hash_algorithm, FRONTEND_ARTIFACT_HASH_ALGORITHM_BLAKE3);
        assert_eq!(attestation.spec_hash, sample_hash());
        assert_eq!(attestation.validator_kind, FRONTEND_ARTIFACT_VALIDATOR_KIND_ALLOWLIST_V1);
        assert_eq!(attestation.validator_ref, SAMPLE_VALIDATOR_REF);
        assert_eq!(attestation.artifact_kind, SAMPLE_ARTIFACT_KIND);
        assert_eq!(attestation.artifact_ref, SAMPLE_ARTIFACT_REF);
        assert_eq!(attestation.target_identity.as_deref(), Some(SAMPLE_TARGET_IDENTITY));
        assert_eq!(attestation.build_root, SAMPLE_BUILD_ROOT);
        assert_eq!(attestation.validation_result, FRONTEND_ARTIFACT_VALIDATION_RESULT_ADMITTED);
        assert!(attestation.no_hidden_fallback);
    }

    #[test]
    fn missing_spec_fails_closed() {
        let spec = sample_spec();
        let manifest = sample_manifest(&spec);

        let report = admit_frontend_artifact(&sample_request(None, &manifest));

        assert!(!report.admitted);
        assert!(report.attestation.is_none());
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diag| diag.code == FRONTEND_ARTIFACT_DIAG_MISSING_SPEC && diag.path == "spec")
        );
    }

    #[test]
    fn unsupported_validator_kind_fails_closed() {
        let mut spec = sample_spec();
        spec.validator_kind = UNSUPPORTED_VALIDATOR_KIND_WASM.to_string();
        let manifest = sample_manifest(&spec);

        let report = admit_frontend_artifact(&sample_request(Some(&spec), &manifest));

        assert!(!report.admitted);
        assert!(report.diagnostics.iter().any(|diag| diag.code == FRONTEND_ARTIFACT_DIAG_UNSUPPORTED_VALIDATOR));
    }

    #[test]
    fn spec_hash_mismatch_fails_closed() {
        let mut spec = sample_spec();
        spec.hash = BAD_BLAKE3_HEX.to_string();
        let manifest = sample_manifest(&spec);

        let report = admit_frontend_artifact(&sample_request(Some(&spec), &manifest));

        assert!(!report.admitted);
        assert!(report.diagnostics.iter().any(|diag| diag.code == FRONTEND_ARTIFACT_DIAG_HASH_MISMATCH));
    }

    #[test]
    fn manifest_spec_binding_mismatch_fails_closed() {
        let spec = sample_spec();
        let mut manifest = sample_manifest(&spec);
        manifest.spec_id = "other.activation".to_string();

        let report = admit_frontend_artifact(&sample_request(Some(&spec), &manifest));

        assert!(!report.admitted);
        assert!(
            report.diagnostics.iter().any(
                |diag| diag.code == FRONTEND_ARTIFACT_DIAG_SPEC_BINDING_MISMATCH && diag.path == "manifest.spec_id"
            )
        );
    }

    #[test]
    fn hidden_fallback_marker_fails_closed() {
        let spec = sample_spec();
        let mut manifest = sample_manifest(&spec);
        manifest.no_hidden_fallback = false;

        let report = admit_frontend_artifact(&sample_request(Some(&spec), &manifest));

        assert!(!report.admitted);
        assert!(report.diagnostics.iter().any(|diag| diag.code == FRONTEND_ARTIFACT_DIAG_HIDDEN_FALLBACK));
    }

    #[test]
    fn artifact_kind_not_allowed_fails_closed() {
        let spec = sample_spec();
        let mut manifest = sample_manifest(&spec);
        manifest.kind = DISALLOWED_ARTIFACT_KIND.to_string();

        let report = admit_frontend_artifact(&sample_request(Some(&spec), &manifest));

        assert!(!report.admitted);
        assert!(report.diagnostics.iter().any(|diag| diag.code == FRONTEND_ARTIFACT_DIAG_KIND_NOT_ALLOWED));
    }

    #[test]
    fn onix_like_kind_remains_frontend_data_under_spec() {
        let spec = sample_spec();
        let mut manifest = sample_manifest(&spec);
        manifest.kind = SAMPLE_ONIX_ARTIFACT_KIND.to_string();

        let report = admit_frontend_artifact(&sample_request(Some(&spec), &manifest));

        assert!(report.admitted, "{:#?}", report.diagnostics);
        let attestation = report.attestation.expect("admission attestation");
        assert_eq!(attestation.artifact_kind, SAMPLE_ONIX_ARTIFACT_KIND);
        assert_eq!(attestation.spec_id, SAMPLE_SPEC_ID);
    }

    #[test]
    fn frontend_semantic_terms_are_data_under_the_declared_spec() {
        let spec = sample_spec();
        for artifact_kind in [
            SAMPLE_ONIX_ARTIFACT_KIND,
            SAMPLE_NIXOS_ARTIFACT_KIND,
            SAMPLE_ROLE_ARTIFACT_KIND,
            SAMPLE_TAG_ARTIFACT_KIND,
            SAMPLE_PROVIDER_ARTIFACT_KIND,
        ] {
            let mut manifest = sample_manifest(&spec);
            manifest.kind = artifact_kind.to_string();

            let report = admit_frontend_artifact(&sample_request(Some(&spec), &manifest));

            assert!(report.admitted, "{artifact_kind}: {:#?}", report.diagnostics);
            assert_eq!(report.attestation.expect("admission attestation").artifact_kind, artifact_kind);
        }
    }

    #[test]
    fn admission_sidecar_serializes_attestation() {
        let spec = sample_spec();
        let manifest = sample_manifest(&spec);
        let report = admit_frontend_artifact(&sample_request(Some(&spec), &manifest));
        let sidecar =
            render_frontend_artifact_admission_sidecar(report.attestation.expect("admission attestation")).unwrap();
        let value: serde_json::Value = serde_json::from_str(&sidecar).unwrap();

        assert_eq!(value["schema"], FRONTEND_ARTIFACT_ADMISSION_SIDECAR_SCHEMA);
        assert_eq!(value["attestation"]["spec_id"], SAMPLE_SPEC_ID);
        assert_eq!(value["attestation"]["artifact_kind"], SAMPLE_ARTIFACT_KIND);
        assert_eq!(value["attestation"]["validation_result"], FRONTEND_ARTIFACT_VALIDATION_RESULT_ADMITTED);
    }
}
