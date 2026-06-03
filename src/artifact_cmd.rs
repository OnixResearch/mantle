use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;

use serde_json::Value;

use crate::errors::RunError;
use crate::frontend_artifact_export::FRONTEND_ARTIFACT_EXPORT_MODE_ARCHIVE;
use crate::frontend_artifact_export::FRONTEND_ARTIFACT_EXPORT_MODE_DIRECTORY;
use crate::frontend_artifact_export::FRONTEND_ARTIFACT_EXPORT_MODE_STREAM;
use crate::frontend_artifact_export::FrontendArtifactExportContent;
use crate::frontend_artifact_export::FrontendArtifactExportDiagnostic;
use crate::frontend_artifact_export::FrontendArtifactExportExpectation;
use crate::frontend_artifact_export::FrontendArtifactExportReport;
use crate::frontend_artifact_export::FrontendArtifactExportRequest;
use crate::frontend_artifact_export::export_frontend_artifact;
use crate::frontend_artifact_export::render_frontend_artifact_export_receipt;
use crate::frontend_artifact_spec::FrontendArtifactAdmissionAttestation;

const PROVENANCE_PAIR_SEPARATOR: char = '=';
const EXPORT_FAILURE_EXIT_CODE: u8 = 1;

pub fn cmd_artifact(action: crate::ArtifactAction, current_dir: &Path, json: bool) -> Result<(), RunError> {
    match action {
        crate::ArtifactAction::Export {
            artifact_ref,
            attestation,
            materialized_path,
            artifact_digest,
            spec_id,
            spec_version,
            spec_hash,
            destination_mode,
            content_provenance,
            receipt_out,
        } => cmd_artifact_export(ArtifactExportShellRequest {
            current_dir,
            json,
            artifact_ref: &artifact_ref,
            attestation_path: &attestation,
            materialized_path: &materialized_path,
            artifact_digest: &artifact_digest,
            spec_id: spec_id.as_deref(),
            spec_version: spec_version.as_deref(),
            spec_hash: spec_hash.as_deref(),
            destination_mode: &destination_mode,
            content_provenance: &content_provenance,
            receipt_out: receipt_out.as_deref(),
        }),
    }
}

struct ArtifactExportShellRequest<'a> {
    current_dir: &'a Path,
    json: bool,
    artifact_ref: &'a str,
    attestation_path: &'a Path,
    materialized_path: &'a Path,
    artifact_digest: &'a str,
    spec_id: Option<&'a str>,
    spec_version: Option<&'a str>,
    spec_hash: Option<&'a str>,
    destination_mode: &'a str,
    content_provenance: &'a [String],
    receipt_out: Option<&'a Path>,
}

fn cmd_artifact_export(request: ArtifactExportShellRequest<'_>) -> Result<(), RunError> {
    let attestation_path = resolve_cli_path(request.current_dir, request.attestation_path);
    let materialized_path = resolve_cli_path(request.current_dir, request.materialized_path);
    let receipt_out = request.receipt_out.map(|path| resolve_cli_path(request.current_dir, path));
    let attestation = load_frontend_artifact_attestation(&attestation_path)?;
    let provenance = parse_content_provenance(request.content_provenance)?;
    let content =
        export_content_if_available(request.artifact_ref, request.artifact_digest, &materialized_path, provenance);
    let expectation = FrontendArtifactExportExpectation {
        artifact_ref: request.artifact_ref,
        artifact_digest: Some(request.artifact_digest),
        spec_id: request.spec_id,
        spec_version: request.spec_version,
        spec_hash: request.spec_hash,
    };
    let content_ref = content.as_ref();
    let export_request = FrontendArtifactExportRequest {
        expectation,
        attestation: Some(&attestation),
        content: content_ref,
        destination_mode: request.destination_mode,
        supported_destination_modes: supported_destination_modes(),
        no_hidden_fallback: true,
    };
    let report = export_frontend_artifact(&export_request);
    emit_export_report(&report, receipt_out.as_deref(), request.json)
}

fn supported_destination_modes() -> &'static [&'static str] {
    &[
        FRONTEND_ARTIFACT_EXPORT_MODE_DIRECTORY,
        FRONTEND_ARTIFACT_EXPORT_MODE_ARCHIVE,
        FRONTEND_ARTIFACT_EXPORT_MODE_STREAM,
    ]
}

fn export_content_if_available(
    artifact_ref: &str,
    artifact_digest: &str,
    materialized_path: &Path,
    provenance: BTreeMap<String, String>,
) -> Option<FrontendArtifactExportContent> {
    if !materialized_path.exists() {
        return None;
    }
    Some(FrontendArtifactExportContent {
        artifact_ref: artifact_ref.to_string(),
        artifact_digest: artifact_digest.to_string(),
        materialized_path: materialized_path.display().to_string(),
        provenance,
    })
}

fn emit_export_report(
    report: &FrontendArtifactExportReport,
    receipt_out: Option<&Path>,
    json: bool,
) -> Result<(), RunError> {
    if !report.exported {
        return emit_failed_export_report(report, json);
    }
    let Some(receipt) = report.receipt.as_ref() else {
        return Err(RunError::Internal("frontend artifact export reported success without a receipt".to_string()));
    };
    if let Some(path) = receipt_out {
        write_receipt(path, receipt)?;
    }
    if json {
        let rendered = serde_json::to_string_pretty(report)
            .map_err(|err| RunError::Internal(format!("serializing artifact export report: {err}")))?;
        println!("{rendered}");
        return Ok(());
    }
    println!("exported {}", receipt.artifact_ref);
    println!("receipt_hash: {}", receipt.receipt_hash);
    if let Some(path) = receipt_out {
        println!("receipt: {}", path.display());
    }
    Ok(())
}

fn emit_failed_export_report(report: &FrontendArtifactExportReport, json: bool) -> Result<(), RunError> {
    if json {
        let rendered = serde_json::to_string_pretty(report)
            .map_err(|err| RunError::Internal(format!("serializing artifact export report: {err}")))?;
        println!("{rendered}");
        return Err(RunError::Reported(EXPORT_FAILURE_EXIT_CODE));
    }
    Err(RunError::Build(format_export_diagnostics(&report.diagnostics)))
}

fn write_receipt(
    path: &Path,
    receipt: &crate::frontend_artifact_export::FrontendArtifactExportReceipt,
) -> Result<(), RunError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| RunError::Internal(format!("creating {}: {err}", parent.display())))?;
    }
    let rendered = render_frontend_artifact_export_receipt(receipt)
        .map_err(|err| RunError::Internal(format!("serializing {}: {err}", path.display())))?;
    std::fs::write(path, rendered).map_err(|err| RunError::Internal(format!("writing {}: {err}", path.display())))
}

fn load_frontend_artifact_attestation(path: &Path) -> Result<FrontendArtifactAdmissionAttestation, RunError> {
    let bytes = std::fs::read(path).map_err(|err| RunError::Internal(format!("reading {}: {err}", path.display())))?;
    let value: Value = serde_json::from_slice(&bytes)
        .map_err(|err| RunError::Internal(format!("parsing {} as JSON: {err}", path.display())))?;
    parse_attestation_value(value).map_err(|err| RunError::Internal(format!("parsing {}: {err}", path.display())))
}

fn parse_attestation_value(value: Value) -> Result<FrontendArtifactAdmissionAttestation, serde_json::Error> {
    if let Some(attestation_value) = value.get("attestation") {
        return serde_json::from_value(attestation_value.clone());
    }
    serde_json::from_value(value)
}

fn parse_content_provenance(entries: &[String]) -> Result<BTreeMap<String, String>, RunError> {
    let mut parsed = BTreeMap::new();
    for entry in entries {
        let Some((key, value)) = entry.split_once(PROVENANCE_PAIR_SEPARATOR) else {
            return Err(RunError::Internal(format!("content provenance entry '{entry}' must use KEY=VALUE syntax")));
        };
        if key.is_empty() {
            return Err(RunError::Internal("content provenance entry key must not be empty".to_string()));
        }
        parsed.insert(key.to_string(), value.to_string());
    }
    Ok(parsed)
}

fn format_export_diagnostics(diagnostics: &[FrontendArtifactExportDiagnostic]) -> String {
    if diagnostics.is_empty() {
        return "frontend artifact export failed without diagnostics".to_string();
    }
    diagnostics
        .iter()
        .map(|diag| format!("{} at {}: {}", diag.code, diag.path, diag.message))
        .collect::<Vec<String>>()
        .join("\n")
}

fn resolve_cli_path(current_dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    current_dir.join(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend_artifact_export::FRONTEND_ARTIFACT_EXPORT_DIGEST_PREFIX_BLAKE3;
    use crate::frontend_artifact_export::FRONTEND_ARTIFACT_EXPORT_RECEIPT_SCHEMA;
    use crate::frontend_artifact_spec::FRONTEND_ARTIFACT_HASH_ALGORITHM_BLAKE3;
    use crate::frontend_artifact_spec::FRONTEND_ARTIFACT_VALIDATION_RESULT_ADMITTED;
    use crate::frontend_artifact_spec::FRONTEND_ARTIFACT_VALIDATOR_KIND_ALLOWLIST_V1;

    const SAMPLE_SPEC_ID: &str = "example.activation";
    const SAMPLE_SPEC_VERSION: &str = "1";
    const SAMPLE_VALIDATOR_REF: &str = "mantle://blake3/spec-validator";
    const SAMPLE_ARTIFACT_KIND: &str = "mantle-onix-activation-closure";
    const SAMPLE_ARTIFACT_REF: &str = "mantle://blake3/artifact";
    const SAMPLE_BUILD_ROOT: &str = "drv:demo";
    const SAMPLE_PROVENANCE_ENTRY: &str = "build_report=report-b3";

    fn sample_digest(seed: &[u8]) -> String {
        format!("{}{}", FRONTEND_ARTIFACT_EXPORT_DIGEST_PREFIX_BLAKE3, blake3::hash(seed).to_hex())
    }

    fn sample_spec_hash() -> String {
        blake3::hash(b"sample spec").to_hex().to_string()
    }

    fn sample_attestation(artifact_digest: &str) -> FrontendArtifactAdmissionAttestation {
        FrontendArtifactAdmissionAttestation {
            spec_id: SAMPLE_SPEC_ID.to_string(),
            spec_version: SAMPLE_SPEC_VERSION.to_string(),
            spec_hash_algorithm: FRONTEND_ARTIFACT_HASH_ALGORITHM_BLAKE3.to_string(),
            spec_hash: sample_spec_hash(),
            validator_kind: FRONTEND_ARTIFACT_VALIDATOR_KIND_ALLOWLIST_V1.to_string(),
            validator_ref: SAMPLE_VALIDATOR_REF.to_string(),
            artifact_kind: SAMPLE_ARTIFACT_KIND.to_string(),
            artifact_ref: SAMPLE_ARTIFACT_REF.to_string(),
            artifact_digest: Some(artifact_digest.to_string()),
            target_identity: None,
            build_root: SAMPLE_BUILD_ROOT.to_string(),
            validation_result: FRONTEND_ARTIFACT_VALIDATION_RESULT_ADMITTED.to_string(),
            no_hidden_fallback: true,
        }
    }

    fn write_json(path: &Path, value: &impl serde::Serialize) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create parent dir");
        }
        let text = serde_json::to_string_pretty(value).expect("serialize JSON");
        std::fs::write(path, text).expect("write JSON");
    }

    #[test]
    fn artifact_export_cli_writes_receipt_for_admitted_artifact() {
        let temp = tempfile::tempdir().expect("tempdir");
        let artifact_digest = sample_digest(b"artifact");
        let attestation = sample_attestation(&artifact_digest);
        let attestation_path = temp.path().join("attestation.json");
        let materialized_path = temp.path().join("artifact-dir");
        let receipt_path = temp.path().join("receipt.json");
        std::fs::create_dir_all(&materialized_path).expect("materialized artifact dir");
        write_json(&attestation_path, &attestation);

        cmd_artifact(
            crate::ArtifactAction::Export {
                artifact_ref: SAMPLE_ARTIFACT_REF.to_string(),
                attestation: attestation_path,
                materialized_path,
                artifact_digest: artifact_digest.clone(),
                spec_id: Some(SAMPLE_SPEC_ID.to_string()),
                spec_version: Some(SAMPLE_SPEC_VERSION.to_string()),
                spec_hash: Some(sample_spec_hash()),
                destination_mode: FRONTEND_ARTIFACT_EXPORT_MODE_DIRECTORY.to_string(),
                content_provenance: vec![SAMPLE_PROVENANCE_ENTRY.to_string()],
                receipt_out: Some(receipt_path.clone()),
            },
            temp.path(),
            false,
        )
        .expect("artifact export succeeds");

        let receipt: crate::frontend_artifact_export::FrontendArtifactExportReceipt =
            serde_json::from_slice(&std::fs::read(&receipt_path).expect("read receipt")).expect("parse receipt");
        assert_eq!(receipt.schema, FRONTEND_ARTIFACT_EXPORT_RECEIPT_SCHEMA);
        assert_eq!(receipt.artifact_ref, SAMPLE_ARTIFACT_REF);
        assert_eq!(receipt.artifact_digest, artifact_digest);
        assert_eq!(receipt.artifact_kind, SAMPLE_ARTIFACT_KIND);
        assert_eq!(receipt.content_provenance["build_report"], "report-b3");
    }

    #[test]
    fn artifact_export_cli_accepts_admission_sidecar() {
        let temp = tempfile::tempdir().expect("tempdir");
        let artifact_digest = sample_digest(b"artifact");
        let attestation = sample_attestation(&artifact_digest);
        let sidecar = serde_json::json!({
            "schema": "mantle-frontend-artifact-admission-v1",
            "attestation": attestation,
        });
        let attestation_path = temp.path().join("sidecar.json");
        let materialized_path = temp.path().join("artifact-dir");
        let receipt_path = temp.path().join("receipt.json");
        std::fs::create_dir_all(&materialized_path).expect("materialized artifact dir");
        write_json(&attestation_path, &sidecar);

        cmd_artifact(
            crate::ArtifactAction::Export {
                artifact_ref: SAMPLE_ARTIFACT_REF.to_string(),
                attestation: attestation_path,
                materialized_path,
                artifact_digest,
                spec_id: Some(SAMPLE_SPEC_ID.to_string()),
                spec_version: Some(SAMPLE_SPEC_VERSION.to_string()),
                spec_hash: Some(sample_spec_hash()),
                destination_mode: FRONTEND_ARTIFACT_EXPORT_MODE_DIRECTORY.to_string(),
                content_provenance: Vec::new(),
                receipt_out: Some(receipt_path.clone()),
            },
            temp.path(),
            false,
        )
        .expect("artifact export succeeds");

        assert!(receipt_path.exists());
    }

    #[test]
    fn artifact_export_cli_rejects_non_mantle_refs() {
        let temp = tempfile::tempdir().expect("tempdir");
        let artifact_digest = sample_digest(b"artifact");
        let attestation = sample_attestation(&artifact_digest);
        let attestation_path = temp.path().join("attestation.json");
        let materialized_path = temp.path().join("artifact-dir");
        std::fs::create_dir_all(&materialized_path).expect("materialized artifact dir");
        write_json(&attestation_path, &attestation);

        let err = cmd_artifact(
            crate::ArtifactAction::Export {
                artifact_ref: "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-demo".to_string(),
                attestation: attestation_path,
                materialized_path,
                artifact_digest,
                spec_id: Some(SAMPLE_SPEC_ID.to_string()),
                spec_version: Some(SAMPLE_SPEC_VERSION.to_string()),
                spec_hash: Some(sample_spec_hash()),
                destination_mode: FRONTEND_ARTIFACT_EXPORT_MODE_DIRECTORY.to_string(),
                content_provenance: Vec::new(),
                receipt_out: None,
            },
            temp.path(),
            false,
        )
        .expect_err("non-mantle ref rejected");

        assert!(err.to_string().contains("frontend-artifact-export-unsupported-ref-scheme"));
    }

    #[test]
    fn artifact_export_cli_rejects_missing_materialized_content() {
        let temp = tempfile::tempdir().expect("tempdir");
        let artifact_digest = sample_digest(b"artifact");
        let attestation = sample_attestation(&artifact_digest);
        let attestation_path = temp.path().join("attestation.json");
        let missing_path = temp.path().join("missing-artifact-dir");
        write_json(&attestation_path, &attestation);

        let err = cmd_artifact(
            crate::ArtifactAction::Export {
                artifact_ref: SAMPLE_ARTIFACT_REF.to_string(),
                attestation: attestation_path,
                materialized_path: missing_path,
                artifact_digest,
                spec_id: Some(SAMPLE_SPEC_ID.to_string()),
                spec_version: Some(SAMPLE_SPEC_VERSION.to_string()),
                spec_hash: Some(sample_spec_hash()),
                destination_mode: FRONTEND_ARTIFACT_EXPORT_MODE_DIRECTORY.to_string(),
                content_provenance: Vec::new(),
                receipt_out: None,
            },
            temp.path(),
            false,
        )
        .expect_err("missing content rejected");

        assert!(err.to_string().contains("frontend-artifact-export-content-unavailable"));
    }

    #[test]
    fn parse_content_provenance_requires_key_value_entries() {
        let err =
            parse_content_provenance(&["missing-separator".to_string()]).expect_err("invalid provenance rejected");
        assert!(err.to_string().contains("KEY=VALUE"));
    }
}
