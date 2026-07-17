use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use serde::Serialize;
use serde_json::Value;

use crate::errors::RunError;
use crate::frontend_artifact_export::FRONTEND_ARTIFACT_EXPORT_DIAG_DIGEST_MISMATCH;
use crate::frontend_artifact_export::FRONTEND_ARTIFACT_EXPORT_DIAG_UNSUPPORTED_REF_SCHEME;
use crate::frontend_artifact_export::FRONTEND_ARTIFACT_EXPORT_MODE_ARCHIVE;
use crate::frontend_artifact_export::FRONTEND_ARTIFACT_EXPORT_MODE_DIRECTORY;
use crate::frontend_artifact_export::FRONTEND_ARTIFACT_EXPORT_MODE_STREAM;
use crate::frontend_artifact_export::FrontendArtifactExportContent;
use crate::frontend_artifact_export::FrontendArtifactExportDiagnostic;
use crate::frontend_artifact_export::FrontendArtifactExportExpectation;
use crate::frontend_artifact_export::FrontendArtifactExportPreflightRequest;
use crate::frontend_artifact_export::FrontendArtifactExportReceipt;
use crate::frontend_artifact_export::FrontendArtifactExportReport;
use crate::frontend_artifact_export::FrontendArtifactExportRequest;
use crate::frontend_artifact_export::export_frontend_artifact;
use crate::frontend_artifact_export::render_frontend_artifact_export_receipt;
use crate::frontend_artifact_export::validate_frontend_artifact_export_preflight;
use crate::frontend_artifact_spec::FrontendArtifactAdmissionAttestation;
use crate::frontend_artifact_store::FrontendArtifactStoreImportReport;
use crate::frontend_artifact_store::artifact_digest_from_ref;
use crate::frontend_artifact_store::import_frontend_artifact;
use crate::frontend_artifact_store::materialize_frontend_artifact;
use crate::oci_projection::OciExportReport;
use crate::oci_projection::OciImportReport;
use crate::oci_projection_shell::ExportRequest as OciExportRequest;
use crate::oci_projection_shell::ImportRequest as OciImportRequest;
use crate::oci_projection_shell::export_oci_layout;
use crate::oci_projection_shell::import_oci_layout;
use crate::oci_registry::OciRegistryPullReport;
use crate::oci_registry::OciRegistryPushReport;
use crate::oci_registry::RegistryTargetInput;
use crate::oci_registry::validate_registry_target;
use crate::oci_registry_shell::RegistryPullRequest;
use crate::oci_registry_shell::RegistryPushRequest;
use crate::oci_registry_shell::load_registry_trust_policy;
use crate::oci_registry_shell::pull_registry_layout;
use crate::oci_registry_shell::push_registry_layout;

const PROVENANCE_PAIR_SEPARATOR: char = '=';
const EXPORT_FAILURE_EXIT_CODE: u8 = 1;
const CONTENT_PROVENANCE_ENTRY_COUNT_MAX: u32 = 4_096;

pub fn cmd_artifact(
    action: crate::ArtifactAction,
    current_dir: &Path,
    state_dir: &Path,
    json: bool,
) -> Result<(), RunError> {
    match action {
        crate::ArtifactAction::Export {
            artifact_ref,
            attestation,
            materialized_path,
            out,
            artifact_digest,
            spec_id,
            spec_version,
            spec_hash,
            destination_mode,
            content_provenance,
            receipt_out,
        } => cmd_artifact_export(ArtifactExportShellRequest {
            current_dir,
            state_dir,
            json,
            artifact_ref: &artifact_ref,
            attestation_path: &attestation,
            materialized_path: materialized_path.as_deref(),
            out_path: out.as_deref(),
            artifact_digest: &artifact_digest,
            spec_id: spec_id.as_deref(),
            spec_version: spec_version.as_deref(),
            spec_hash: spec_hash.as_deref(),
            destination_mode: &destination_mode,
            content_provenance: &content_provenance,
            receipt_out: receipt_out.as_deref(),
        }),
        crate::ArtifactAction::Import { path, report_out } => cmd_artifact_import(ArtifactImportShellRequest {
            current_dir,
            state_dir,
            json,
            source_path: &path,
            report_out: report_out.as_deref(),
        }),
        crate::ArtifactAction::OciExport {
            projection,
            spec_material,
            source_admissions,
            out,
        } => cmd_oci_export(OciExportShellRequest {
            current_dir,
            state_dir,
            json,
            projection_path: &projection,
            spec_material_path: &spec_material,
            source_admissions_path: &source_admissions,
            out_path: &out,
        }),
        crate::ArtifactAction::OciImport { layout, report_out } => {
            cmd_oci_import(current_dir, state_dir, json, &layout, &report_out)
        }
        crate::ArtifactAction::OciPush {
            layout,
            registry,
            repository,
            reference,
            trust_policy,
            signing_keys,
            bearer_token_file,
            allow_http,
            receipt_out,
        } => cmd_oci_push(OciPushShellRequest {
            current_dir,
            state_dir,
            json,
            layout: &layout,
            registry: &registry,
            repository: &repository,
            reference: &reference,
            trust_policy: &trust_policy,
            signing_keys: &signing_keys,
            bearer_token_file: bearer_token_file.as_deref(),
            allow_http,
            receipt_out: &receipt_out,
        }),
        crate::ArtifactAction::OciPull {
            registry,
            repository,
            reference,
            expected_manifest_digest,
            expected_metadata_manifest_digest,
            expected_signature_manifest_digest,
            trust_policy,
            bearer_token_file,
            allow_http,
            out,
            report_out,
            receipt_out,
        } => cmd_oci_pull(OciPullShellRequest {
            current_dir,
            state_dir,
            json,
            registry: &registry,
            repository: &repository,
            reference: &reference,
            expected_manifest_digest: &expected_manifest_digest,
            expected_metadata_manifest_digest: &expected_metadata_manifest_digest,
            expected_signature_manifest_digest: &expected_signature_manifest_digest,
            trust_policy: &trust_policy,
            bearer_token_file: bearer_token_file.as_deref(),
            allow_http,
            output_dir: &out,
            import_report_out: &report_out,
            receipt_out: &receipt_out,
        }),
    }
}

struct ArtifactExportShellRequest<'a> {
    current_dir: &'a Path,
    state_dir: &'a Path,
    json: bool,
    artifact_ref: &'a str,
    attestation_path: &'a Path,
    materialized_path: Option<&'a Path>,
    out_path: Option<&'a Path>,
    artifact_digest: &'a str,
    spec_id: Option<&'a str>,
    spec_version: Option<&'a str>,
    spec_hash: Option<&'a str>,
    destination_mode: &'a str,
    content_provenance: &'a [String],
    receipt_out: Option<&'a Path>,
}

struct ArtifactImportShellRequest<'a> {
    current_dir: &'a Path,
    state_dir: &'a Path,
    json: bool,
    source_path: &'a Path,
    report_out: Option<&'a Path>,
}

struct OciExportShellRequest<'a> {
    current_dir: &'a Path,
    state_dir: &'a Path,
    json: bool,
    projection_path: &'a Path,
    spec_material_path: &'a Path,
    source_admissions_path: &'a Path,
    out_path: &'a Path,
}

struct OciPushShellRequest<'a> {
    current_dir: &'a Path,
    state_dir: &'a Path,
    json: bool,
    layout: &'a Path,
    registry: &'a str,
    repository: &'a str,
    reference: &'a str,
    trust_policy: &'a Path,
    signing_keys: &'a [PathBuf],
    bearer_token_file: Option<&'a Path>,
    allow_http: bool,
    receipt_out: &'a Path,
}

struct OciPullShellRequest<'a> {
    current_dir: &'a Path,
    state_dir: &'a Path,
    json: bool,
    registry: &'a str,
    repository: &'a str,
    reference: &'a str,
    expected_manifest_digest: &'a str,
    expected_metadata_manifest_digest: &'a str,
    expected_signature_manifest_digest: &'a str,
    trust_policy: &'a Path,
    bearer_token_file: Option<&'a Path>,
    allow_http: bool,
    output_dir: &'a Path,
    import_report_out: &'a Path,
    receipt_out: &'a Path,
}

fn cmd_artifact_export(request: ArtifactExportShellRequest<'_>) -> Result<(), RunError> {
    let attestation_path = resolve_cli_path(request.current_dir, request.attestation_path);
    let receipt_out = request.receipt_out.map(|path| resolve_cli_path(request.current_dir, path));
    let attestation = load_frontend_artifact_attestation(&attestation_path)?;
    let provenance = parse_content_provenance(request.content_provenance)?;
    let expectation = FrontendArtifactExportExpectation {
        artifact_ref: request.artifact_ref,
        artifact_digest: Some(request.artifact_digest),
        spec_id: request.spec_id,
        spec_version: request.spec_version,
        spec_hash: request.spec_hash,
    };
    let preflight = FrontendArtifactExportPreflightRequest {
        expectation,
        attestation: Some(&attestation),
        destination_mode: request.destination_mode,
        supported_destination_modes: supported_destination_modes(),
        no_hidden_fallback: true,
    };
    assert_eq!(preflight.expectation.artifact_ref, request.artifact_ref);
    assert!(preflight.no_hidden_fallback, "artifact export shell must fail closed on fallback");
    let preflight_diagnostics = validate_frontend_artifact_export_preflight(&preflight);
    if !preflight_diagnostics.is_empty() {
        return emit_export_report(&failed_report(preflight_diagnostics), receipt_out.as_deref(), request.json);
    }
    if let Some(diagnostics) = storage_ref_diagnostics(&request) {
        return emit_export_report(&failed_report(diagnostics), receipt_out.as_deref(), request.json);
    }
    let content = resolve_export_content(&request, provenance)?;
    let content_ref = content.as_ref();
    let artifact_request = FrontendArtifactExportRequest {
        preflight,
        content: content_ref,
    };
    let outcome = export_frontend_artifact(&artifact_request);
    emit_export_report(&outcome, receipt_out.as_deref(), request.json)
}

fn cmd_artifact_import(request: ArtifactImportShellRequest<'_>) -> Result<(), RunError> {
    let source_path = resolve_cli_path(request.current_dir, request.source_path);
    let summary_path = request.report_out.map(|path| resolve_cli_path(request.current_dir, path));
    let outcome = import_frontend_artifact(&source_path, request.state_dir)
        .map_err(|err| RunError::Internal(format!("importing frontend artifact: {err}")))?;
    if let Some(path) = summary_path.as_deref() {
        write_json_output(path, &outcome)?;
    }
    emit_import_report(&outcome, summary_path.as_deref(), request.json)
}

fn cmd_oci_export(request: OciExportShellRequest<'_>) -> Result<(), RunError> {
    let projection_path = resolve_cli_path(request.current_dir, request.projection_path);
    let spec_material_path = resolve_cli_path(request.current_dir, request.spec_material_path);
    let source_admissions_path = resolve_cli_path(request.current_dir, request.source_admissions_path);
    let out_path = resolve_cli_path(request.current_dir, request.out_path);
    let outcome = export_oci_layout(&OciExportRequest {
        projection_path: &projection_path,
        spec_material_path: &spec_material_path,
        source_admissions_path: &source_admissions_path,
        output_dir: &out_path,
        state_dir: request.state_dir,
    })
    .map_err(|error| RunError::Internal(format!("exporting OCI layout: {error}")))?;
    emit_oci_export_report(&outcome, &out_path, request.json)
}

fn cmd_oci_import(
    current_dir: &Path,
    state_dir: &Path,
    json: bool,
    layout: &Path,
    report_out: &Path,
) -> Result<(), RunError> {
    let layout = resolve_cli_path(current_dir, layout);
    let summary_path = resolve_cli_path(current_dir, report_out);
    let outcome = import_oci_layout(&OciImportRequest {
        layout_dir: &layout,
        report_path: &summary_path,
        state_dir,
    })
    .map_err(|error| RunError::Internal(format!("importing OCI layout: {error}")))?;
    emit_oci_import_report(&outcome, &summary_path, json)
}

fn registry_target(
    registry: &str,
    repository: &str,
    reference: &str,
    allow_http: bool,
) -> Result<crate::oci_registry::RegistryTarget, RunError> {
    validate_registry_target(RegistryTargetInput {
        registry,
        repository,
        reference,
        allow_http,
    })
    .map_err(|error| RunError::Internal(format!("validating OCI registry target: {error}")))
}

fn load_registry_signing_keys(
    current_dir: &Path,
    state_dir: &Path,
    paths: &[PathBuf],
) -> Result<Vec<crunch_build::KeyPair>, RunError> {
    if paths.is_empty() {
        return Err(RunError::Internal("OCI registry push requires at least one explicit signing key".to_string()));
    }
    let mut keys = Vec::with_capacity(paths.len());
    for path in paths {
        let resolved = resolve_cli_path(current_dir, path);
        let (keypair, _source_path) = crate::build_cmd::load_existing_signing_keypair(Some(&resolved), state_dir)?;
        keys.push(keypair);
    }
    assert_eq!(keys.len(), paths.len());
    assert!(!keys.is_empty(), "registry signing keys must not be empty");
    Ok(keys)
}

fn cmd_oci_push(request: OciPushShellRequest<'_>) -> Result<(), RunError> {
    let layout = resolve_cli_path(request.current_dir, request.layout);
    let receipt_out = resolve_cli_path(request.current_dir, request.receipt_out);
    require_absent_output(&receipt_out, "OCI registry push receipt")?;
    let policy_path = resolve_cli_path(request.current_dir, request.trust_policy);
    let trust_policy = load_registry_trust_policy(&policy_path)
        .map_err(|error| RunError::Internal(format!("loading OCI registry trust policy: {error}")))?;
    let signing_keys = load_registry_signing_keys(request.current_dir, request.state_dir, request.signing_keys)?;
    let bearer_token_file = request.bearer_token_file.map(|path| resolve_cli_path(request.current_dir, path));
    let target = registry_target(request.registry, request.repository, request.reference, request.allow_http)?;
    let report = push_registry_layout(RegistryPushRequest {
        target: &target,
        layout_dir: &layout,
        bearer_token_file: bearer_token_file.as_deref(),
        trust_policy: &trust_policy,
        signing_keys: &signing_keys,
    })
    .map_err(|error| RunError::Internal(format!("pushing OCI registry layout: {error}")))?;
    write_json_output_new(&receipt_out, &report)?;
    emit_oci_registry_push_report(&report, request.json)
}

fn cmd_oci_pull(request: OciPullShellRequest<'_>) -> Result<(), RunError> {
    let output_dir = resolve_cli_path(request.current_dir, request.output_dir);
    let import_report_out = resolve_cli_path(request.current_dir, request.import_report_out);
    let receipt_out = resolve_cli_path(request.current_dir, request.receipt_out);
    require_absent_output(&receipt_out, "OCI registry pull receipt")?;
    let policy_path = resolve_cli_path(request.current_dir, request.trust_policy);
    let trust_policy = load_registry_trust_policy(&policy_path)
        .map_err(|error| RunError::Internal(format!("loading OCI registry trust policy: {error}")))?;
    let bearer_token_file = request.bearer_token_file.map(|path| resolve_cli_path(request.current_dir, path));
    let target = registry_target(request.registry, request.repository, request.reference, request.allow_http)?;
    let report = pull_registry_layout(RegistryPullRequest {
        target: &target,
        expected_manifest_digest: request.expected_manifest_digest,
        expected_metadata_manifest_digest: request.expected_metadata_manifest_digest,
        expected_signature_manifest_digest: request.expected_signature_manifest_digest,
        trust_policy: &trust_policy,
        output_dir: &output_dir,
        state_dir: request.state_dir,
        import_report_path: &import_report_out,
        bearer_token_file: bearer_token_file.as_deref(),
    })
    .map_err(|error| RunError::Internal(format!("pulling OCI registry layout: {error}")))?;
    write_json_output_new(&receipt_out, &report)?;
    emit_oci_registry_pull_report(&report, request.json)
}

fn emit_oci_registry_push_report(report: &OciRegistryPushReport, json: bool) -> Result<(), RunError> {
    if json {
        let rendered = serde_json::to_string_pretty(report)
            .map_err(|error| RunError::Internal(format!("serializing OCI registry push report: {error}")))?;
        println!("{rendered}");
    } else {
        println!("published OCI registry image {}:{}", report.repository, report.reference);
        println!("manifest_digest: {}", report.manifest_digest);
        println!("metadata_manifest_digest: {}", report.metadata_manifest_digest);
        println!("signature_manifest_digest: {}", report.signature_manifest_digest);
    }
    Ok(())
}

fn emit_oci_registry_pull_report(report: &OciRegistryPullReport, json: bool) -> Result<(), RunError> {
    if json {
        let rendered = serde_json::to_string_pretty(report)
            .map_err(|error| RunError::Internal(format!("serializing OCI registry pull report: {error}")))?;
        println!("{rendered}");
    } else {
        println!("pulled OCI registry image {}:{}", report.repository, report.reference);
        println!("manifest_digest: {}", report.resolved_manifest_digest);
        println!("signature_manifest_digest: {}", report.resolved_signature_manifest_digest);
        println!("import_state: {}", report.import_state);
    }
    Ok(())
}

fn emit_oci_export_report(report: &OciExportReport, out: &Path, json: bool) -> Result<(), RunError> {
    if json {
        let rendered = serde_json::to_string_pretty(report)
            .map_err(|error| RunError::Internal(format!("serializing OCI export report: {error}")))?;
        println!("{rendered}");
    } else {
        println!("exported OCI layout {}", out.display());
        println!("projection_blake3: {}", report.projection_blake3);
        println!("layout_blake3: {}", report.layout_blake3);
    }
    Ok(())
}

fn emit_oci_import_report(report: &OciImportReport, out: &Path, json: bool) -> Result<(), RunError> {
    if json {
        let rendered = serde_json::to_string_pretty(report)
            .map_err(|error| RunError::Internal(format!("serializing OCI import report: {error}")))?;
        println!("{rendered}");
    } else {
        println!("imported OCI layout as {}", report.state);
        println!("layout_blake3: {}", report.layout_blake3);
        println!("report: {}", out.display());
    }
    Ok(())
}

fn resolve_export_content(
    request: &ArtifactExportShellRequest<'_>,
    provenance: BTreeMap<String, String>,
) -> Result<Option<FrontendArtifactExportContent>, RunError> {
    assert!(
        u32::try_from(provenance.len()).is_ok_and(|entry_count| entry_count <= CONTENT_PROVENANCE_ENTRY_COUNT_MAX),
        "parsed provenance must stay within the shell bound"
    );
    assert!(!request.artifact_ref.is_empty(), "preflight-admitted artifact ref must not be empty");
    if let Some(materialized_path) = request.materialized_path {
        let materialized_path = resolve_cli_path(request.current_dir, materialized_path);
        return Ok(export_content_if_available(
            request.artifact_ref,
            &materialized_path,
            request.artifact_digest,
            provenance,
        ));
    }
    let Some(out_path) = request.out_path else {
        return Err(RunError::Internal(
            "artifact export requires --materialized-path or storage-backed --out".to_string(),
        ));
    };
    let out_path = resolve_cli_path(request.current_dir, out_path);
    let stored = materialize_frontend_artifact(request.state_dir, request.artifact_ref, &out_path)
        .map_err(|err| RunError::Internal(format!("materializing frontend artifact: {err}")))?;
    Ok(stored.map(|stored| FrontendArtifactExportContent {
        artifact_ref: stored.artifact_ref,
        artifact_digest: stored.artifact_digest,
        materialized_path: stored.content_path.display().to_string(),
        provenance,
    }))
}

fn storage_ref_diagnostics(request: &ArtifactExportShellRequest<'_>) -> Option<Vec<FrontendArtifactExportDiagnostic>> {
    assert!(!FRONTEND_ARTIFACT_EXPORT_DIAG_UNSUPPORTED_REF_SCHEME.is_empty());
    assert!(!FRONTEND_ARTIFACT_EXPORT_DIAG_DIGEST_MISMATCH.is_empty());
    if request.materialized_path.is_some() {
        return None;
    }
    let Some(actual_digest) = artifact_digest_from_ref(request.artifact_ref) else {
        return Some(vec![FrontendArtifactExportDiagnostic {
            code: FRONTEND_ARTIFACT_EXPORT_DIAG_UNSUPPORTED_REF_SCHEME.to_string(),
            path: "expectation.artifact_ref".to_string(),
            message: "storage-backed frontend artifact export requires a mantle://blake3/<lowercase-hex> artifact ref"
                .to_string(),
        }]);
    };
    if actual_digest == request.artifact_digest {
        return None;
    }
    Some(vec![FrontendArtifactExportDiagnostic {
        code: FRONTEND_ARTIFACT_EXPORT_DIAG_DIGEST_MISMATCH.to_string(),
        path: "expectation.artifact_digest".to_string(),
        message: "requested artifact digest does not match the storage-backed artifact ref".to_string(),
    }])
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
    materialized_path: &Path,
    artifact_digest: &str,
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
    assert!(report.exported, "receipt emission requires a successful export outcome");
    assert!(!receipt.receipt_hash.is_empty(), "successful export receipt must carry identity");
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

fn emit_import_report(
    report: &FrontendArtifactStoreImportReport,
    report_out: Option<&Path>,
    json: bool,
) -> Result<(), RunError> {
    if json {
        let rendered = serde_json::to_string_pretty(report)
            .map_err(|err| RunError::Internal(format!("serializing artifact import report: {err}")))?;
        println!("{rendered}");
        return Ok(());
    }
    println!("imported {}", report.artifact_ref);
    println!("artifact_digest: {}", report.artifact_digest);
    if let Some(path) = report_out {
        println!("report: {}", path.display());
    }
    Ok(())
}

fn failed_report(diagnostics: Vec<FrontendArtifactExportDiagnostic>) -> FrontendArtifactExportReport {
    FrontendArtifactExportReport {
        exported: false,
        receipt: None,
        diagnostics,
    }
}

fn write_receipt(path: &Path, receipt: &FrontendArtifactExportReceipt) -> Result<(), RunError> {
    let rendered = render_frontend_artifact_export_receipt(receipt)
        .map_err(|err| RunError::Internal(format!("serializing {}: {err}", path.display())))?;
    write_text_output(path, &rendered)
}

fn write_json_output(path: &Path, value: &impl Serialize) -> Result<(), RunError> {
    let rendered = serde_json::to_string_pretty(value)
        .map_err(|err| RunError::Internal(format!("serializing {}: {err}", path.display())))?;
    write_text_output(path, &rendered)
}

fn require_absent_output(path: &Path, label: &str) -> Result<(), RunError> {
    if path.exists() {
        return Err(RunError::Internal(format!("{label} already exists: {}", path.display())));
    }
    assert!(!label.is_empty(), "output label must be explicit");
    assert!(!path.as_os_str().is_empty(), "output path must not be empty");
    Ok(())
}

fn write_json_output_new(path: &Path, value: &impl Serialize) -> Result<(), RunError> {
    require_absent_output(path, "registry receipt")?;
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent)
        .map_err(|error| RunError::Internal(format!("creating {}: {error}", parent.display())))?;
    let mut rendered = serde_json::to_vec_pretty(value)
        .map_err(|error| RunError::Internal(format!("serializing {}: {error}", path.display())))?;
    rendered.push(b'\n');
    let mut temporary = tempfile::NamedTempFile::new_in(parent)
        .map_err(|error| RunError::Internal(format!("creating temporary receipt in {}: {error}", parent.display())))?;
    temporary
        .write_all(&rendered)
        .and_then(|()| temporary.as_file().sync_all())
        .map_err(|error| RunError::Internal(format!("writing temporary receipt for {}: {error}", path.display())))?;
    temporary
        .persist_noclobber(path)
        .map_err(|error| RunError::Internal(format!("publishing {}: {}", path.display(), error.error)))?;
    assert!(path.is_file(), "published registry receipt must be a file");
    assert!(!rendered.is_empty(), "published registry receipt bytes must not be empty");
    Ok(())
}

fn write_text_output(path: &Path, rendered: &str) -> Result<(), RunError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| RunError::Internal(format!("creating {}: {err}", parent.display())))?;
    }
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
    let entry_count = u32::try_from(entries.len())
        .map_err(|_| RunError::Internal("content provenance entry count overflowed u32".to_string()))?;
    if entry_count > CONTENT_PROVENANCE_ENTRY_COUNT_MAX {
        return Err(RunError::Internal(format!(
            "content provenance entry count exceeds {CONTENT_PROVENANCE_ENTRY_COUNT_MAX}"
        )));
    }
    let entry_count_max = usize::try_from(CONTENT_PROVENANCE_ENTRY_COUNT_MAX)
        .map_err(|_| RunError::Internal("content provenance bound overflowed usize".to_string()))?;
    assert!(entry_count_max > 0, "content provenance entry bound must be positive");
    assert!(entries.len() <= entry_count_max, "content provenance entries must stay bounded");
    let mut parsed = BTreeMap::new();
    for entry in entries {
        let Some((key, value)) = entry.split_once(PROVENANCE_PAIR_SEPARATOR) else {
            return Err(RunError::Internal(format!("content provenance entry '{entry}' must use KEY=VALUE syntax")));
        };
        if key.is_empty() {
            return Err(RunError::Internal("content provenance entry key must not be empty".to_string()));
        }
        if parsed.len() >= entry_count_max {
            return Err(RunError::Internal("content provenance map exceeded its admitted bound".to_string()));
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
    const SAMPLE_REJECTED_VALIDATION_RESULT: &str = "rejected";
    const SAMPLE_PROVENANCE_ENTRY: &str = "build_report=report-b3";
    const STORED_ARTIFACT_CONTENT: &[u8] = b"stored frontend artifact";

    fn sample_digest(seed: &[u8]) -> String {
        format!("{}{}", FRONTEND_ARTIFACT_EXPORT_DIGEST_PREFIX_BLAKE3, blake3::hash(seed).to_hex())
    }

    fn sample_spec_hash() -> String {
        blake3::hash(b"sample spec").to_hex().to_string()
    }

    fn sample_attestation(artifact_ref: &str, artifact_digest: &str) -> FrontendArtifactAdmissionAttestation {
        FrontendArtifactAdmissionAttestation {
            spec_id: SAMPLE_SPEC_ID.to_string(),
            spec_version: SAMPLE_SPEC_VERSION.to_string(),
            spec_hash_algorithm: FRONTEND_ARTIFACT_HASH_ALGORITHM_BLAKE3.to_string(),
            spec_hash: sample_spec_hash(),
            validator_kind: FRONTEND_ARTIFACT_VALIDATOR_KIND_ALLOWLIST_V1.to_string(),
            validator_ref: SAMPLE_VALIDATOR_REF.to_string(),
            artifact_kind: SAMPLE_ARTIFACT_KIND.to_string(),
            artifact_ref: artifact_ref.to_string(),
            artifact_digest: Some(artifact_digest.to_string()),
            target_identity: None,
            build_root: SAMPLE_BUILD_ROOT.to_string(),
            validation_result: FRONTEND_ARTIFACT_VALIDATION_RESULT_ADMITTED.to_string(),
            no_hidden_fallback: true,
        }
    }

    fn sample_legacy_attestation(artifact_digest: &str) -> FrontendArtifactAdmissionAttestation {
        sample_attestation(SAMPLE_ARTIFACT_REF, artifact_digest)
    }

    fn hidden_fallback_attestation(artifact_ref: &str, artifact_digest: &str) -> FrontendArtifactAdmissionAttestation {
        let mut attestation = sample_attestation(artifact_ref, artifact_digest);
        attestation.no_hidden_fallback = false;
        attestation
    }

    fn rejected_attestation(artifact_ref: &str, artifact_digest: &str) -> FrontendArtifactAdmissionAttestation {
        let mut attestation = sample_attestation(artifact_ref, artifact_digest);
        attestation.validation_result = SAMPLE_REJECTED_VALIDATION_RESULT.to_string();
        attestation
    }

    fn write_json(path: &Path, value: &impl serde::Serialize) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create parent dir");
        }
        let text = serde_json::to_string_pretty(value).expect("serialize JSON");
        std::fs::write(path, text).expect("write JSON");
    }

    fn legacy_export_action(
        attestation_path: PathBuf,
        materialized_path: PathBuf,
        receipt_path: Option<PathBuf>,
        artifact_digest: String,
    ) -> crate::ArtifactAction {
        crate::ArtifactAction::Export {
            artifact_ref: SAMPLE_ARTIFACT_REF.to_string(),
            attestation: attestation_path,
            materialized_path: Some(materialized_path),
            out: None,
            artifact_digest,
            spec_id: Some(SAMPLE_SPEC_ID.to_string()),
            spec_version: Some(SAMPLE_SPEC_VERSION.to_string()),
            spec_hash: Some(sample_spec_hash()),
            destination_mode: FRONTEND_ARTIFACT_EXPORT_MODE_DIRECTORY.to_string(),
            content_provenance: vec![SAMPLE_PROVENANCE_ENTRY.to_string()],
            receipt_out: receipt_path,
        }
    }

    #[test]
    fn artifact_import_cli_writes_store_report() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = temp.path().join("artifact.txt");
        let report_path = temp.path().join("report.json");
        std::fs::write(&source, STORED_ARTIFACT_CONTENT).expect("write source");

        cmd_artifact(
            crate::ArtifactAction::Import {
                path: source,
                report_out: Some(report_path.clone()),
            },
            temp.path(),
            temp.path(),
            false,
        )
        .expect("artifact import succeeds");

        let report: FrontendArtifactStoreImportReport =
            serde_json::from_slice(&std::fs::read(&report_path).expect("read report")).expect("parse report");
        assert!(report.artifact_ref.starts_with("mantle://blake3/"));
        assert!(report.artifact_digest.starts_with(FRONTEND_ARTIFACT_EXPORT_DIGEST_PREFIX_BLAKE3));
    }

    #[test]
    fn artifact_export_cli_materializes_from_artifact_store() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = temp.path().join("artifact.txt");
        let attestation_path = temp.path().join("attestation.json");
        let exported_path = temp.path().join("exported.txt");
        let receipt_path = temp.path().join("receipt.json");
        std::fs::write(&source, STORED_ARTIFACT_CONTENT).expect("write source");
        let import_report = import_frontend_artifact(&source, temp.path()).expect("import artifact");
        let attestation = sample_attestation(&import_report.artifact_ref, &import_report.artifact_digest);
        write_json(&attestation_path, &attestation);

        cmd_artifact(
            crate::ArtifactAction::Export {
                artifact_ref: import_report.artifact_ref.clone(),
                attestation: attestation_path,
                materialized_path: None,
                out: Some(exported_path.clone()),
                artifact_digest: import_report.artifact_digest.clone(),
                spec_id: Some(SAMPLE_SPEC_ID.to_string()),
                spec_version: Some(SAMPLE_SPEC_VERSION.to_string()),
                spec_hash: Some(sample_spec_hash()),
                destination_mode: FRONTEND_ARTIFACT_EXPORT_MODE_DIRECTORY.to_string(),
                content_provenance: vec![SAMPLE_PROVENANCE_ENTRY.to_string()],
                receipt_out: Some(receipt_path.clone()),
            },
            temp.path(),
            temp.path(),
            false,
        )
        .expect("artifact export succeeds");

        assert_eq!(std::fs::read(&exported_path).expect("read exported"), STORED_ARTIFACT_CONTENT);
        let receipt: FrontendArtifactExportReceipt =
            serde_json::from_slice(&std::fs::read(&receipt_path).expect("read receipt")).expect("parse receipt");
        assert_eq!(receipt.schema, FRONTEND_ARTIFACT_EXPORT_RECEIPT_SCHEMA);
        assert_eq!(receipt.artifact_ref, import_report.artifact_ref);
        assert_eq!(receipt.artifact_digest, import_report.artifact_digest);
        assert_eq!(receipt.materialized_path, exported_path.display().to_string());
        assert_eq!(receipt.content_provenance["build_report"], "report-b3");
    }

    #[test]
    fn artifact_export_cli_rejects_hidden_fallback_before_materialization() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = temp.path().join("artifact.txt");
        let attestation_path = temp.path().join("attestation.json");
        let exported_path = temp.path().join("exported.txt");
        std::fs::write(&source, STORED_ARTIFACT_CONTENT).expect("write source");
        let import_report = import_frontend_artifact(&source, temp.path()).expect("import artifact");
        let attestation = hidden_fallback_attestation(&import_report.artifact_ref, &import_report.artifact_digest);
        write_json(&attestation_path, &attestation);

        let err = cmd_artifact(
            crate::ArtifactAction::Export {
                artifact_ref: import_report.artifact_ref,
                attestation: attestation_path,
                materialized_path: None,
                out: Some(exported_path.clone()),
                artifact_digest: import_report.artifact_digest,
                spec_id: Some(SAMPLE_SPEC_ID.to_string()),
                spec_version: Some(SAMPLE_SPEC_VERSION.to_string()),
                spec_hash: Some(sample_spec_hash()),
                destination_mode: FRONTEND_ARTIFACT_EXPORT_MODE_DIRECTORY.to_string(),
                content_provenance: Vec::new(),
                receipt_out: None,
            },
            temp.path(),
            temp.path(),
            false,
        )
        .expect_err("hidden fallback rejected");

        assert!(err.to_string().contains("frontend-artifact-export-hidden-fallback"));
        assert!(!exported_path.exists());
    }

    #[test]
    fn artifact_export_cli_rejects_missing_admission_before_materialization() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = temp.path().join("artifact.txt");
        let attestation_path = temp.path().join("attestation.json");
        let exported_path = temp.path().join("exported.txt");
        std::fs::write(&source, STORED_ARTIFACT_CONTENT).expect("write source");
        let import_report = import_frontend_artifact(&source, temp.path()).expect("import artifact");
        let attestation = rejected_attestation(&import_report.artifact_ref, &import_report.artifact_digest);
        write_json(&attestation_path, &attestation);

        let err = cmd_artifact(
            crate::ArtifactAction::Export {
                artifact_ref: import_report.artifact_ref,
                attestation: attestation_path,
                materialized_path: None,
                out: Some(exported_path.clone()),
                artifact_digest: import_report.artifact_digest,
                spec_id: Some(SAMPLE_SPEC_ID.to_string()),
                spec_version: Some(SAMPLE_SPEC_VERSION.to_string()),
                spec_hash: Some(sample_spec_hash()),
                destination_mode: FRONTEND_ARTIFACT_EXPORT_MODE_DIRECTORY.to_string(),
                content_provenance: Vec::new(),
                receipt_out: None,
            },
            temp.path(),
            temp.path(),
            false,
        )
        .expect_err("missing admission rejected");

        assert!(err.to_string().contains("frontend-artifact-export-missing-admission-proof"));
        assert!(!exported_path.exists());
    }

    #[test]
    fn artifact_export_cli_rejects_wrong_ref_before_materialization() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = temp.path().join("artifact.txt");
        let attestation_path = temp.path().join("attestation.json");
        let exported_path = temp.path().join("exported.txt");
        std::fs::write(&source, STORED_ARTIFACT_CONTENT).expect("write source");
        let import_report = import_frontend_artifact(&source, temp.path()).expect("import artifact");
        let other_digest_hex = blake3::hash(b"other ref").to_hex().to_string();
        let other_ref = format!("mantle://blake3/{other_digest_hex}");
        let attestation = sample_attestation(&other_ref, &import_report.artifact_digest);
        write_json(&attestation_path, &attestation);

        let err = cmd_artifact(
            crate::ArtifactAction::Export {
                artifact_ref: import_report.artifact_ref,
                attestation: attestation_path,
                materialized_path: None,
                out: Some(exported_path.clone()),
                artifact_digest: import_report.artifact_digest,
                spec_id: Some(SAMPLE_SPEC_ID.to_string()),
                spec_version: Some(SAMPLE_SPEC_VERSION.to_string()),
                spec_hash: Some(sample_spec_hash()),
                destination_mode: FRONTEND_ARTIFACT_EXPORT_MODE_DIRECTORY.to_string(),
                content_provenance: Vec::new(),
                receipt_out: None,
            },
            temp.path(),
            temp.path(),
            false,
        )
        .expect_err("wrong ref rejected");

        assert!(err.to_string().contains("frontend-artifact-export-proof-mismatch"));
        assert!(!exported_path.exists());
    }

    #[test]
    fn artifact_export_cli_rejects_digest_mismatch_before_materialization() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = temp.path().join("artifact.txt");
        let attestation_path = temp.path().join("attestation.json");
        let exported_path = temp.path().join("exported.txt");
        std::fs::write(&source, STORED_ARTIFACT_CONTENT).expect("write source");
        let import_report = import_frontend_artifact(&source, temp.path()).expect("import artifact");
        let wrong_digest = sample_digest(b"wrong digest");
        let attestation = sample_attestation(&import_report.artifact_ref, &wrong_digest);
        write_json(&attestation_path, &attestation);

        let err = cmd_artifact(
            crate::ArtifactAction::Export {
                artifact_ref: import_report.artifact_ref,
                attestation: attestation_path,
                materialized_path: None,
                out: Some(exported_path.clone()),
                artifact_digest: wrong_digest,
                spec_id: Some(SAMPLE_SPEC_ID.to_string()),
                spec_version: Some(SAMPLE_SPEC_VERSION.to_string()),
                spec_hash: Some(sample_spec_hash()),
                destination_mode: FRONTEND_ARTIFACT_EXPORT_MODE_DIRECTORY.to_string(),
                content_provenance: Vec::new(),
                receipt_out: None,
            },
            temp.path(),
            temp.path(),
            false,
        )
        .expect_err("digest mismatch rejected");

        assert!(err.to_string().contains("frontend-artifact-export-digest-mismatch"));
        assert!(!exported_path.exists());
    }

    #[test]
    fn artifact_export_cli_writes_receipt_for_admitted_artifact() {
        let temp = tempfile::tempdir().expect("tempdir");
        let artifact_digest = sample_digest(b"artifact");
        let attestation = sample_legacy_attestation(&artifact_digest);
        let attestation_path = temp.path().join("attestation.json");
        let materialized_path = temp.path().join("artifact-dir");
        let receipt_path = temp.path().join("receipt.json");
        std::fs::create_dir_all(&materialized_path).expect("materialized artifact dir");
        write_json(&attestation_path, &attestation);

        cmd_artifact(
            legacy_export_action(
                attestation_path,
                materialized_path,
                Some(receipt_path.clone()),
                artifact_digest.clone(),
            ),
            temp.path(),
            temp.path(),
            false,
        )
        .expect("artifact export succeeds");

        let receipt: FrontendArtifactExportReceipt =
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
        let attestation = sample_legacy_attestation(&artifact_digest);
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
            legacy_export_action(attestation_path, materialized_path, Some(receipt_path.clone()), artifact_digest),
            temp.path(),
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
        let attestation = sample_legacy_attestation(&artifact_digest);
        let attestation_path = temp.path().join("attestation.json");
        let materialized_path = temp.path().join("artifact-dir");
        std::fs::create_dir_all(&materialized_path).expect("materialized artifact dir");
        write_json(&attestation_path, &attestation);

        let mut action = legacy_export_action(attestation_path, materialized_path, None, artifact_digest);
        let crate::ArtifactAction::Export { artifact_ref, .. } = &mut action else {
            unreachable!("legacy action is export")
        };
        *artifact_ref = "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-demo".to_string();
        let err = cmd_artifact(action, temp.path(), temp.path(), false).expect_err("non-mantle ref rejected");

        assert!(err.to_string().contains("frontend-artifact-export-unsupported-ref-scheme"));
    }

    #[test]
    fn artifact_export_cli_rejects_missing_materialized_content() {
        let temp = tempfile::tempdir().expect("tempdir");
        let artifact_digest = sample_digest(b"artifact");
        let attestation = sample_legacy_attestation(&artifact_digest);
        let attestation_path = temp.path().join("attestation.json");
        let missing_path = temp.path().join("missing-artifact-dir");
        write_json(&attestation_path, &attestation);

        let err = cmd_artifact(
            legacy_export_action(attestation_path, missing_path, None, artifact_digest),
            temp.path(),
            temp.path(),
            false,
        )
        .expect_err("missing content rejected");

        assert!(err.to_string().contains("frontend-artifact-export-content-unavailable"));
    }

    #[test]
    fn artifact_export_cli_rejects_missing_stored_content() {
        let temp = tempfile::tempdir().expect("tempdir");
        let missing_digest_hex = blake3::hash(b"missing stored content").to_hex().to_string();
        let artifact_ref = format!("mantle://blake3/{missing_digest_hex}");
        let artifact_digest = format!("{FRONTEND_ARTIFACT_EXPORT_DIGEST_PREFIX_BLAKE3}{missing_digest_hex}");
        let attestation = sample_attestation(&artifact_ref, &artifact_digest);
        let attestation_path = temp.path().join("attestation.json");
        let exported_path = temp.path().join("exported");
        write_json(&attestation_path, &attestation);

        let err = cmd_artifact(
            crate::ArtifactAction::Export {
                artifact_ref,
                attestation: attestation_path,
                materialized_path: None,
                out: Some(exported_path),
                artifact_digest,
                spec_id: Some(SAMPLE_SPEC_ID.to_string()),
                spec_version: Some(SAMPLE_SPEC_VERSION.to_string()),
                spec_hash: Some(sample_spec_hash()),
                destination_mode: FRONTEND_ARTIFACT_EXPORT_MODE_DIRECTORY.to_string(),
                content_provenance: Vec::new(),
                receipt_out: None,
            },
            temp.path(),
            temp.path(),
            false,
        )
        .expect_err("missing stored content rejected");

        assert!(err.to_string().contains("frontend-artifact-export-content-unavailable"));
    }

    #[test]
    fn parse_content_provenance_requires_key_value_entries() {
        let err =
            parse_content_provenance(&["missing-separator".to_string()]).expect_err("invalid provenance rejected");
        assert!(err.to_string().contains("KEY=VALUE"));
    }
}
