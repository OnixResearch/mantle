use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use mantle_application_contract::ARTIFACT_CALL_EFFECT;
use mantle_application_contract::ARTIFACT_EXPORT_ATTESTATION_EFFECT;
use mantle_application_contract::ARTIFACT_EXPORT_CONTENT_EFFECT;
use mantle_application_contract::ARTIFACT_EXPORT_RECEIPT_EFFECT;
use mantle_application_contract::ARTIFACT_REGISTRY_INPUT_EFFECT;
use mantle_application_contract::ARTIFACT_REGISTRY_RECEIPT_EFFECT;
use mantle_application_contract::ARTIFACT_REGISTRY_TRANSFER_EFFECT;
use mantle_application_contract::ApplicationOutcome;
use mantle_application_contract::ArtifactExportContentSource;
use mantle_application_contract::ArtifactExportPort;
use mantle_application_contract::ArtifactImportPort;
use mantle_application_contract::ArtifactPortCall;
use mantle_application_contract::ArtifactPortFact;
use mantle_application_contract::CapabilityError;
use mantle_application_contract::EffectId;
use mantle_application_contract::EffectKind;
use mantle_application_contract::EffectMeasure;
use mantle_application_contract::EffectOutput;
use mantle_application_contract::EffectPlan;
use mantle_application_contract::Observation;
use mantle_application_contract::ObservationStatus;
use mantle_application_contract::OciLayoutExportPort;
use mantle_application_contract::OciLayoutImportPort;
use mantle_application_contract::OciRegistryPullPort;
use mantle_application_contract::OciRegistryPushPort;
use mantle_application_contract::artifact_call_effect_plan;
use mantle_application_contract::artifact_export_effect_plan;
use mantle_application_contract::artifact_registry_effect_plan;
use mantle_application_contract::classify_observations;
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
use crate::frontend_artifact_store::frontend_artifact_is_available;
use crate::frontend_artifact_store::import_frontend_artifact;
use crate::frontend_artifact_store::materialize_frontend_artifact;
use crate::oci_projection::OCI_INDEX_FILENAME;
use crate::oci_projection::OCI_LAYOUT_FILENAME;
use crate::oci_projection::OciExportReport;
use crate::oci_projection::OciImportReport;
use crate::oci_projection_shell::ExportRequest as OciExportRequest;
use crate::oci_projection_shell::ImportRequest as OciImportRequest;
use crate::oci_projection_shell::export_oci_layout;
use crate::oci_projection_shell::import_oci_layout;
use crate::oci_registry::OciRegistryPullReport;
use crate::oci_registry::OciRegistryPushReport;
use crate::oci_registry::RegistryTarget;
use crate::oci_registry::RegistryTargetInput;
use crate::oci_registry::ValidatedRegistryTrustPolicy;
use crate::oci_registry::validate_registry_target;
use crate::oci_registry_shell::RegistryPullRequest;
use crate::oci_registry_shell::RegistryPushRequest;
use crate::oci_registry_shell::load_registry_trust_policy;
use crate::oci_registry_shell::pull_registry_layout;
use crate::oci_registry_shell::push_registry_layout;

const PROVENANCE_PAIR_SEPARATOR: char = '=';
const EXPORT_FAILURE_EXIT_CODE: u8 = 1;
const CONTENT_PROVENANCE_ENTRY_COUNT_MAX: u32 = 4_096;
/// Diagnostic code of an artifact adapter call that was refused.
const ARTIFACT_CALL_FAILED_CODE: &str = "artifact-call-failed";
/// Diagnostic code of a rejected export whose report names no diagnostic.
const ARTIFACT_EXPORT_REJECTED_CODE: &str = "artifact-export-rejected";

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

/// Plan the export, then read its attestation, resolve its content, and write its receipt through
/// the export port.
///
/// The plan exists before the attestation read. Admission keeps the `da00f5842` order: the
/// attestation read, the provenance parse, the preflight, the storage-ref check, and the
/// content-source check all run before the content is resolved. Every stop is classified before
/// anything is reported, and each keeps its `da00f5842` report, message, and exit.
fn cmd_artifact_export(request: ArtifactExportShellRequest<'_>) -> Result<(), RunError> {
    let attestation_path = resolve_cli_path(request.current_dir, request.attestation_path);
    let receipt_out = request.receipt_out.map(|path| resolve_cli_path(request.current_dir, path));
    let mut port = ArtifactExportFiles {
        current_dir: request.current_dir,
        state_dir: request.state_dir,
        artifact_ref: request.artifact_ref,
        artifact_digest: request.artifact_digest,
        attestation_path: &attestation_path,
        materialized_path: request.materialized_path,
        out_path: request.out_path,
        receipt_out: receipt_out.as_deref(),
    };
    run_artifact_export(&request, &attestation_path, receipt_out.as_deref(), &mut port)
}

fn run_artifact_export(
    request: &ArtifactExportShellRequest<'_>,
    attestation_path: &Path,
    receipt_out: Option<&Path>,
    port: &mut impl ArtifactExportPort<
        Attestation = FrontendArtifactAdmissionAttestation,
        Content = Option<ResolvedExportContent>,
        Receipt = FrontendArtifactExportReceipt,
    >,
) -> Result<(), RunError> {
    let content_destination = request
        .materialized_path
        .or(request.out_path)
        .map(|path| resolve_cli_path(request.current_dir, path));
    let source = if request.materialized_path.is_some() {
        ArtifactExportContentSource::MaterializedPath
    } else {
        ArtifactExportContentSource::Store
    };
    let content_identity = content_destination.as_deref().map(effect_path_identity).transpose()?;
    let receipt_identity = receipt_out.map(effect_path_identity).transpose()?;
    let plan = artifact_export_effect_plan(
        source,
        effect_path_identity(attestation_path)?,
        content_identity,
        receipt_identity,
    )
    .map_err(|error| RunError::Internal(format!("{}: {}", error.code, error.detail)))?;
    let mut observations = [
        skipped_observation(ARTIFACT_EXPORT_ATTESTATION_EFFECT, plan.effects[0].kind),
        skipped_observation(ARTIFACT_EXPORT_CONTENT_EFFECT, plan.effects[1].kind),
        skipped_observation(ARTIFACT_EXPORT_RECEIPT_EFFECT, EffectKind::WriteFiles),
    ];
    let (read, observed) = observed_port_call(ARTIFACT_EXPORT_ATTESTATION_EFFECT, port.read_attestation());
    observations[0] = observed;
    let attestation = match read {
        Ok(attestation) => attestation,
        Err(error) => {
            return Err(stopped_export(
                classify_export_observations(&plan, &observations),
                error,
                ARTIFACT_EXPORT_ATTESTATION_EFFECT,
            ));
        }
    };
    ensure_artifact_prefix_consistent(
        &plan,
        &observations[..plan.effects.len()],
        "artifact export",
        ARTIFACT_EXPORT_ATTESTATION_EFFECT,
    )?;
    let provenance = match parse_content_provenance(request.content_provenance) {
        Ok(provenance) => provenance,
        Err(rejection) => return Err(rejected_export(&plan, &observations, rejection)),
    };
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
        return report_rejected_export(&plan, &observations, &failed_report(preflight_diagnostics), request.json);
    }
    if let Some(diagnostics) = storage_ref_diagnostics(request) {
        let rejection = failed_report(diagnostics);
        return report_rejected_export(&plan, &observations, &rejection, request.json);
    }
    if source == ArtifactExportContentSource::Store && request.out_path.is_none() {
        let rejection =
            RunError::Internal("artifact export requires --materialized-path or storage-backed --out".to_string());
        return Err(rejected_export(&plan, &observations, rejection));
    }
    assert!(
        u32::try_from(provenance.len()).is_ok_and(|entry_count| entry_count <= CONTENT_PROVENANCE_ENTRY_COUNT_MAX),
        "parsed provenance must stay within the shell bound"
    );
    assert!(!request.artifact_ref.is_empty(), "preflight-admitted artifact ref must not be empty");
    let (resolution, observed) = observed_port_call(ARTIFACT_EXPORT_CONTENT_EFFECT, port.resolve_content());
    observations[1] = observed;
    let resolved = match resolution {
        Ok(resolved) => resolved,
        Err(error) => {
            return Err(stopped_export(
                classify_export_observations(&plan, &observations),
                error,
                ARTIFACT_EXPORT_CONTENT_EFFECT,
            ));
        }
    };
    if resolved.is_none() {
        // A missing source is a domain rejection, never proof that a path was produced.
        if observations[1].output != EffectOutput::None {
            return Err(contradictory_artifact_observation(
                "artifact export",
                ARTIFACT_EXPORT_CONTENT_EFFECT,
                ApplicationOutcome::Contradicted { effect_count: 1 },
                None,
            ));
        }
        observations[1].status = ObservationStatus::Failed;
    }
    ensure_artifact_prefix_consistent(
        &plan,
        &observations[..plan.effects.len()],
        "artifact export",
        ARTIFACT_EXPORT_CONTENT_EFFECT,
    )?;
    let content = resolved.map(|resolved| FrontendArtifactExportContent {
        artifact_ref: resolved.artifact_ref,
        artifact_digest: resolved.artifact_digest,
        materialized_path: resolved.materialized_path,
        provenance,
    });
    let outcome = export_frontend_artifact(&FrontendArtifactExportRequest {
        preflight,
        content: content.as_ref(),
    });
    if !outcome.exported {
        let code = outcome.diagnostics.first().map_or(ARTIFACT_EXPORT_REJECTED_CODE, |diag| diag.code.as_str());
        observations[1].status = ObservationStatus::Failed;
        observations[1].diagnostics_code = Some(code.to_string());
        return report_rejected_export(&plan, &observations, &outcome, request.json);
    }
    let Some(receipt) = outcome.receipt.as_ref() else {
        return Err(RunError::Internal("frontend artifact export reported success without a receipt".to_string()));
    };
    assert!(outcome.exported, "receipt emission requires a successful export outcome");
    assert!(!receipt.receipt_hash.is_empty(), "successful export receipt must carry identity");
    if receipt_out.is_some() {
        let (written, observed) = observed_port_call(ARTIFACT_EXPORT_RECEIPT_EFFECT, port.write_receipt(receipt));
        observations[2] = observed;
        if let Err(error) = written {
            return Err(stopped_export(
                classify_export_observations(&plan, &observations),
                error,
                ARTIFACT_EXPORT_RECEIPT_EFFECT,
            ));
        }
    }
    match classify_export_observations(&plan, &observations) {
        ApplicationOutcome::Completed => emit_exported_report(&outcome, receipt, receipt_out, request.json),
        other => Err(contradictory_artifact_observation(
            "artifact export",
            if receipt_out.is_some() {
                ARTIFACT_EXPORT_RECEIPT_EFFECT
            } else {
                ARTIFACT_EXPORT_CONTENT_EFFECT
            },
            other,
            None,
        )),
    }
}

/// The exact error of an export that a port call stopped, once that stop classified as failed.
fn stopped_export(outcome: ApplicationOutcome, error: CapabilityError, effect_id: &str) -> RunError {
    match outcome {
        ApplicationOutcome::Failed { .. } => RunError::Internal(error.detail),
        other => contradictory_artifact_observation("artifact export", effect_id, other, Some(&error)),
    }
}

/// The exact error of an export that admission rejected before its content was resolved.
fn rejected_export(plan: &EffectPlan, observed: &[Observation; 3], rejection: RunError) -> RunError {
    match classify_export_observations(plan, observed) {
        ApplicationOutcome::Failed { .. } => rejection,
        other => contradictory_artifact_observation("artifact export", ARTIFACT_EXPORT_ATTESTATION_EFFECT, other, None),
    }
}

fn classify_export_observations(plan: &EffectPlan, observed: &[Observation; 3]) -> ApplicationOutcome {
    classify_observations(plan, &observed[..plan.effects.len()])
}

/// Report a rejected export once its independently observed effects classify it as failed.
fn report_rejected_export(
    plan: &EffectPlan,
    observed: &[Observation; 3],
    report: &FrontendArtifactExportReport,
    json: bool,
) -> Result<(), RunError> {
    debug_assert!(!report.exported);
    match classify_export_observations(plan, observed) {
        ApplicationOutcome::Failed { .. } => emit_failed_export_report(report, json),
        other => Err(contradictory_artifact_observation(
            "artifact export",
            if observed[1].status == ObservationStatus::Failed {
                ARTIFACT_EXPORT_CONTENT_EFFECT
            } else {
                ARTIFACT_EXPORT_ATTESTATION_EFFECT
            },
            other,
            None,
        )),
    }
}

/// Plan the import, run it through the import port, and classify the executed call before
/// reporting.
fn cmd_artifact_import(request: ArtifactImportShellRequest<'_>) -> Result<(), RunError> {
    let source_path = resolve_cli_path(request.current_dir, request.source_path);
    let summary_path = request.report_out.map(|path| resolve_cli_path(request.current_dir, path));
    let store_root = resolve_cli_path(request.current_dir, request.state_dir);
    let plan = planned_artifact_call(EffectKind::StoreAccess, &store_root)?;
    let mut port = ArtifactImportFiles {
        current_dir: request.current_dir,
        source: &source_path,
        state_dir: request.state_dir,
        report_out: summary_path.as_deref(),
    };
    let (imported, actual) = observed_port_call(ARTIFACT_CALL_EFFECT, port.import_artifact());
    let outcome = classified_artifact_call(&plan, actual, imported, "artifact import")?;
    emit_import_report(&outcome, summary_path.as_deref(), request.json)
}

/// The plan is fixed before the composite port is called; the port supplies the actual observation.
fn planned_artifact_call(kind: EffectKind, destination: &Path) -> Result<EffectPlan, RunError> {
    artifact_call_effect_plan(kind, effect_path_identity(destination)?)
        .map_err(|error| RunError::Internal(format!("{}: {}", error.code, error.detail)))
}

/// Effect identities are UTF-8; refuse an unrepresentable path before the port writes anything.
fn effect_path_identity(path: &Path) -> Result<&str, RunError> {
    path.to_str().ok_or_else(|| {
        RunError::Internal(format!(
            "artifact-effect-identity-non-utf8: effect authority cannot be represented: {path:?}"
        ))
    })
}

/// Keep the port's actual authority, output, and usage separate from the shell's plan.
fn observed_port_call<R>(effect_id: &str, called: ArtifactPortCall<R>) -> (Result<R, CapabilityError>, Observation) {
    let status = if called.result.is_ok() {
        ObservationStatus::Succeeded
    } else {
        ObservationStatus::Failed
    };
    let observed = Observation {
        effect_id: EffectId(effect_id.to_string()),
        kind: called.fact.kind,
        status,
        output: called.fact.output,
        usage: called.fact.usage,
        diagnostics_code: called.result.as_ref().err().map(|error| error.code.clone()),
    };
    (called.result, observed)
}

/// Executing an adapter, not observing its shell call site, consumes one call.
fn executed_artifact_port<R>(
    kind: EffectKind,
    execute: impl FnOnce(&mut ArtifactPortFact) -> Result<R, CapabilityError>,
) -> ArtifactPortCall<R> {
    let mut fact = ArtifactPortFact {
        kind,
        output: EffectOutput::None,
        usage: EffectMeasure::Calls(1),
    };
    let result = execute(&mut fact);
    ArtifactPortCall { result, fact }
}

fn skipped_observation(effect_id: &str, kind: EffectKind) -> Observation {
    Observation {
        effect_id: EffectId(effect_id.to_string()),
        kind,
        status: ObservationStatus::Skipped,
        output: EffectOutput::None,
        usage: EffectMeasure::Calls(0),
        diagnostics_code: None,
    }
}

/// An inconsistent adapter observation retains its effect and any port failure.
fn contradictory_artifact_observation(
    operation: &str,
    effect_id: &str,
    outcome: ApplicationOutcome,
    adapter_error: Option<&CapabilityError>,
) -> RunError {
    let detail = if let Some(error) = adapter_error {
        format!(
            "artifact-observations-contradicted: {operation} effect={effect_id} outcome={outcome:?} adapter={}: {}",
            error.code, error.detail,
        )
    } else {
        format!("artifact-observations-contradicted: {operation} effect={effect_id} outcome={outcome:?}")
    };
    RunError::Internal(detail)
}

/// A successful port result cannot authorize the next effect when its facts disagree.
/// Unrun effects are still Skipped, so a sound prefix classifies as Failed until complete.
fn ensure_artifact_prefix_consistent(
    plan: &EffectPlan,
    observed: &[Observation],
    operation: &str,
    just_executed: &str,
) -> Result<(), RunError> {
    match classify_observations(plan, observed) {
        outcome @ (ApplicationOutcome::Blocked { .. }
        | ApplicationOutcome::Contradicted { .. }
        | ApplicationOutcome::Rejected { .. }) => {
            Err(contradictory_artifact_observation(operation, just_executed, outcome, None))
        }
        ApplicationOutcome::Completed | ApplicationOutcome::Failed { .. } => Ok(()),
    }
}

/// A refused call preserves its exact error only if its independent observation classified as
/// failed.
fn classified_artifact_call<R>(
    plan: &EffectPlan,
    observed: Observation,
    called: Result<R, CapabilityError>,
    operation: &str,
) -> Result<R, RunError> {
    let outcome = classify_observations(plan, std::slice::from_ref(&observed));
    let effect_id = observed.effect_id.0.as_str();
    match (outcome, called) {
        (ApplicationOutcome::Completed, Ok(report)) => Ok(report),
        (ApplicationOutcome::Failed { .. }, Err(error)) => Err(RunError::Internal(error.detail)),
        (other, Err(error)) => Err(contradictory_artifact_observation(operation, effect_id, other, Some(&error))),
        (other, Ok(_)) => Err(contradictory_artifact_observation(operation, effect_id, other, None)),
    }
}

/// Filesystem adapter for one `artifact import`: the declared source, the state directory, and the
/// report path.
struct ArtifactImportFiles<'a> {
    current_dir: &'a Path,
    source: &'a Path,
    state_dir: &'a Path,
    report_out: Option<&'a Path>,
}

impl ArtifactImportPort for ArtifactImportFiles<'_> {
    type Report = FrontendArtifactStoreImportReport;

    fn import_artifact(&mut self) -> ArtifactPortCall<Self::Report> {
        executed_artifact_port(EffectKind::StoreAccess, |fact| {
            let report = import_frontend_artifact(self.source, self.state_dir)
                .map_err(|error| call_refused(format!("importing frontend artifact: {error}")))?;
            let root = resolve_cli_path(self.current_dir, self.state_dir);
            let stored = resolve_cli_path(self.current_dir, Path::new(&report.stored_content_path));
            let manifest = resolve_cli_path(self.current_dir, Path::new(&report.manifest_path));
            for actual in [&stored, &manifest] {
                if actual.strip_prefix(&root).is_err() {
                    fact.output = EffectOutput::Identity(actual.display().to_string());
                    return Err(call_refused(format!(
                        "frontend artifact store path escaped {}: {}",
                        root.display(),
                        actual.display()
                    )));
                }
            }
            if !frontend_artifact_is_available(self.state_dir, &report.artifact_ref)
                .map_err(|error| call_refused(format!("checking imported frontend artifact: {error}")))?
            {
                return Err(call_refused("imported frontend artifact did not read back from the store".to_string()));
            }
            fact.output = EffectOutput::Identity(root.display().to_string());
            if let Some(path) = self.report_out {
                write_json_output(path, &report).map_err(refused_with)?;
            }
            Ok(report)
        })
    }
}

fn call_refused(detail: String) -> CapabilityError {
    debug_assert!(!detail.is_empty());
    CapabilityError {
        code: ARTIFACT_CALL_FAILED_CODE.to_string(),
        detail,
    }
}

/// Plan the layout export, run it through the export port, and classify the executed call before
/// reporting.
fn cmd_oci_export(request: OciExportShellRequest<'_>) -> Result<(), RunError> {
    let projection_path = resolve_cli_path(request.current_dir, request.projection_path);
    let spec_material_path = resolve_cli_path(request.current_dir, request.spec_material_path);
    let source_admissions_path = resolve_cli_path(request.current_dir, request.source_admissions_path);
    let out_path = resolve_cli_path(request.current_dir, request.out_path);
    let plan = planned_artifact_call(EffectKind::WriteFiles, &out_path)?;
    let mut port = OciLayoutExportFiles {
        request: OciExportRequest {
            projection_path: &projection_path,
            spec_material_path: &spec_material_path,
            source_admissions_path: &source_admissions_path,
            output_dir: &out_path,
            state_dir: request.state_dir,
        },
    };
    let (exported, actual) = observed_port_call(ARTIFACT_CALL_EFFECT, port.export_layout());
    let outcome = classified_artifact_call(&plan, actual, exported, "artifact oci-export")?;
    emit_oci_export_report(&outcome, &out_path, request.json)
}

/// Plan the layout import, run it through the import port, and classify the executed call before
/// reporting.
fn cmd_oci_import(
    current_dir: &Path,
    state_dir: &Path,
    json: bool,
    layout: &Path,
    report_out: &Path,
) -> Result<(), RunError> {
    let layout = resolve_cli_path(current_dir, layout);
    let summary_path = resolve_cli_path(current_dir, report_out);
    let store_root = resolve_cli_path(current_dir, state_dir);
    let plan = planned_artifact_call(EffectKind::StoreAccess, &store_root)?;
    let mut port = OciLayoutImportFiles {
        current_dir,
        request: OciImportRequest {
            layout_dir: &layout,
            report_path: &summary_path,
            state_dir,
        },
    };
    let (imported, actual) = observed_port_call(ARTIFACT_CALL_EFFECT, port.import_layout());
    let outcome = classified_artifact_call(&plan, actual, imported, "artifact oci-import")?;
    emit_oci_import_report(&outcome, &summary_path, json)
}

/// Filesystem adapter for one `artifact oci-export`, scoped to its declared paths.
struct OciLayoutExportFiles<'a> {
    request: OciExportRequest<'a>,
}

impl OciLayoutExportPort for OciLayoutExportFiles<'_> {
    type Report = OciExportReport;

    fn export_layout(&mut self) -> ArtifactPortCall<Self::Report> {
        executed_artifact_port(EffectKind::WriteFiles, |fact| {
            let report = export_oci_layout(&self.request)
                .map_err(|error| call_refused(format!("exporting OCI layout: {error}")))?;
            for name in [OCI_LAYOUT_FILENAME, OCI_INDEX_FILENAME] {
                let path = self.request.output_dir.join(name);
                let file = std::fs::File::open(&path).map_err(|error| {
                    call_refused(format!("reading published OCI layout {}: {error}", path.display()))
                })?;
                if !file
                    .metadata()
                    .map_err(|error| {
                        call_refused(format!("checking published OCI layout {}: {error}", path.display()))
                    })?
                    .is_file()
                {
                    return Err(call_refused(format!("published OCI layout is not a file: {}", path.display())));
                }
            }
            fact.output = EffectOutput::Identity(self.request.output_dir.display().to_string());
            Ok(report)
        })
    }
}

/// Filesystem adapter for one `artifact oci-import`, scoped to its declared paths.
struct OciLayoutImportFiles<'a> {
    current_dir: &'a Path,
    request: OciImportRequest<'a>,
}

impl OciLayoutImportPort for OciLayoutImportFiles<'_> {
    type Report = OciImportReport;

    fn import_layout(&mut self) -> ArtifactPortCall<Self::Report> {
        executed_artifact_port(EffectKind::StoreAccess, |fact| {
            let report = import_oci_layout(&self.request)
                .map_err(|error| call_refused(format!("importing OCI layout: {error}")))?;
            for object in &report.objects {
                let Some(artifact_ref) = object.artifact_ref.as_deref() else {
                    continue;
                };
                if !frontend_artifact_is_available(self.request.state_dir, artifact_ref)
                    .map_err(|error| call_refused(format!("checking imported OCI artifact: {error}")))?
                {
                    return Err(call_refused(format!("imported OCI artifact did not read back: {artifact_ref}")));
                }
            }
            if !self.request.report_path.is_file() {
                return Err(call_refused(format!(
                    "OCI import report did not read back: {}",
                    self.request.report_path.display()
                )));
            }
            let root = resolve_cli_path(self.current_dir, self.request.state_dir);
            fact.output = EffectOutput::Identity(root.display().to_string());
            Ok(report)
        })
    }
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
        let (keypair, _source_path) = crate::signing_key::load_existing_signing_keypair(Some(&resolved), state_dir)?;
        keys.push(keypair);
    }
    assert_eq!(keys.len(), paths.len());
    assert!(!keys.is_empty(), "registry signing keys must not be empty");
    Ok(keys)
}

/// Plan the push, then read its inputs, push, and write its receipt through the push port.
///
/// Admission keeps the `da00f5842` order: the receipt probe, the trust policy,
/// and the signing keys are read first, then the target is validated. A
/// rejected push never reaches the registry. Every stop is classified before
/// anything is reported, and each keeps its `da00f5842` message.
fn cmd_oci_push(request: OciPushShellRequest<'_>) -> Result<(), RunError> {
    let layout = resolve_cli_path(request.current_dir, request.layout);
    let receipt_out = resolve_cli_path(request.current_dir, request.receipt_out);
    let bearer_token_file = request.bearer_token_file.map(|path| resolve_cli_path(request.current_dir, path));
    let mut port = OciPushFiles {
        current_dir: request.current_dir,
        state_dir: request.state_dir,
        layout: &layout,
        receipt_out: &receipt_out,
        trust_policy: request.trust_policy,
        signing_keys: request.signing_keys,
        bearer_token_file: bearer_token_file.as_deref(),
    };
    run_oci_push(&request, &receipt_out, &mut port)
}

fn run_oci_push(
    request: &OciPushShellRequest<'_>,
    receipt_out: &Path,
    port: &mut impl OciRegistryPushPort<
        Inputs = (ValidatedRegistryTrustPolicy, Vec<crunch_build::KeyPair>),
        Admitted = OciPushAdmitted,
        Report = OciRegistryPushReport,
    >,
) -> Result<(), RunError> {
    let plan = planned_registry_transfer(receipt_out)?;
    let mut observed = registry_observations(&plan);
    let (inputs, observation) = observed_port_call(ARTIFACT_REGISTRY_INPUT_EFFECT, port.read_inputs());
    observed[0] = observation;
    let (trust_policy, signing_keys) = match inputs {
        Ok(inputs) => inputs,
        Err(error) => {
            return Err(stopped_transfer(
                classify_observations(&plan, &observed),
                error,
                "push",
                ARTIFACT_REGISTRY_INPUT_EFFECT,
            ));
        }
    };
    ensure_artifact_prefix_consistent(&plan, &observed, "push", ARTIFACT_REGISTRY_INPUT_EFFECT)?;
    let target = match registry_target(request.registry, request.repository, request.reference, request.allow_http) {
        Ok(target) => target,
        Err(rejection) => return Err(rejected_transfer(&plan, &observed, rejection, "push")),
    };
    let admitted = OciPushAdmitted {
        trust_policy,
        signing_keys,
        target,
    };
    let (transferred, observation) = observed_port_call(ARTIFACT_REGISTRY_TRANSFER_EFFECT, port.push(&admitted));
    observed[1] = observation;
    let report = match transferred {
        Ok(report) => report,
        Err(error) => {
            return Err(stopped_transfer(
                classify_observations(&plan, &observed),
                error,
                "push",
                ARTIFACT_REGISTRY_TRANSFER_EFFECT,
            ));
        }
    };
    ensure_artifact_prefix_consistent(&plan, &observed, "push", ARTIFACT_REGISTRY_TRANSFER_EFFECT)?;
    let (written, observation) = observed_port_call(ARTIFACT_REGISTRY_RECEIPT_EFFECT, port.write_receipt(&report));
    observed[2] = observation;
    if let Err(error) = written {
        return Err(stopped_transfer(
            classify_observations(&plan, &observed),
            error,
            "push",
            ARTIFACT_REGISTRY_RECEIPT_EFFECT,
        ));
    }
    recorded_transfer(&plan, &observed, "push")?;
    emit_oci_registry_push_report(&report, request.json)
}

/// Plan the pull, then read its inputs, pull, and write its receipt through the pull port.
///
/// Admission keeps the `da00f5842` order: the receipt probe and the trust
/// policy are read first, then the target is validated. A rejected pull never
/// reaches the registry. Every stop is classified before anything is reported,
/// and each keeps its `da00f5842` message.
fn cmd_oci_pull(request: OciPullShellRequest<'_>) -> Result<(), RunError> {
    let output_dir = resolve_cli_path(request.current_dir, request.output_dir);
    let import_report_out = resolve_cli_path(request.current_dir, request.import_report_out);
    let receipt_out = resolve_cli_path(request.current_dir, request.receipt_out);
    let bearer_token_file = request.bearer_token_file.map(|path| resolve_cli_path(request.current_dir, path));
    let mut port = OciPullFiles {
        current_dir: request.current_dir,
        state_dir: request.state_dir,
        output_dir: &output_dir,
        import_report_path: &import_report_out,
        receipt_out: &receipt_out,
        trust_policy: request.trust_policy,
        bearer_token_file: bearer_token_file.as_deref(),
        expected_manifest_digest: request.expected_manifest_digest,
        expected_metadata_manifest_digest: request.expected_metadata_manifest_digest,
        expected_signature_manifest_digest: request.expected_signature_manifest_digest,
    };
    run_oci_pull(&request, &receipt_out, &mut port)
}

fn run_oci_pull(
    request: &OciPullShellRequest<'_>,
    receipt_out: &Path,
    port: &mut impl OciRegistryPullPort<
        Inputs = ValidatedRegistryTrustPolicy,
        Admitted = OciPullAdmitted,
        Report = OciRegistryPullReport,
    >,
) -> Result<(), RunError> {
    let plan = planned_registry_transfer(receipt_out)?;
    let mut observed = registry_observations(&plan);
    let (inputs, observation) = observed_port_call(ARTIFACT_REGISTRY_INPUT_EFFECT, port.read_inputs());
    observed[0] = observation;
    let trust_policy = match inputs {
        Ok(trust_policy) => trust_policy,
        Err(error) => {
            return Err(stopped_transfer(
                classify_observations(&plan, &observed),
                error,
                "pull",
                ARTIFACT_REGISTRY_INPUT_EFFECT,
            ));
        }
    };
    ensure_artifact_prefix_consistent(&plan, &observed, "pull", ARTIFACT_REGISTRY_INPUT_EFFECT)?;
    let target = match registry_target(request.registry, request.repository, request.reference, request.allow_http) {
        Ok(target) => target,
        Err(rejection) => return Err(rejected_transfer(&plan, &observed, rejection, "pull")),
    };
    let admitted = OciPullAdmitted { trust_policy, target };
    let (transferred, observation) = observed_port_call(ARTIFACT_REGISTRY_TRANSFER_EFFECT, port.pull(&admitted));
    observed[1] = observation;
    let report = match transferred {
        Ok(report) => report,
        Err(error) => {
            return Err(stopped_transfer(
                classify_observations(&plan, &observed),
                error,
                "pull",
                ARTIFACT_REGISTRY_TRANSFER_EFFECT,
            ));
        }
    };
    ensure_artifact_prefix_consistent(&plan, &observed, "pull", ARTIFACT_REGISTRY_TRANSFER_EFFECT)?;
    let (written, observation) = observed_port_call(ARTIFACT_REGISTRY_RECEIPT_EFFECT, port.write_receipt(&report));
    observed[2] = observation;
    if let Err(error) = written {
        return Err(stopped_transfer(
            classify_observations(&plan, &observed),
            error,
            "pull",
            ARTIFACT_REGISTRY_RECEIPT_EFFECT,
        ));
    }
    recorded_transfer(&plan, &observed, "pull")?;
    emit_oci_registry_pull_report(&report, request.json)
}

fn registry_observations(plan: &EffectPlan) -> [Observation; 3] {
    [
        skipped_observation(ARTIFACT_REGISTRY_INPUT_EFFECT, plan.effects[0].kind),
        skipped_observation(ARTIFACT_REGISTRY_TRANSFER_EFFECT, plan.effects[1].kind),
        skipped_observation(ARTIFACT_REGISTRY_RECEIPT_EFFECT, plan.effects[2].kind),
    ]
}

/// The plan of one registry transfer, built before its first port call.
fn planned_registry_transfer(receipt: &Path) -> Result<EffectPlan, RunError> {
    artifact_registry_effect_plan(effect_path_identity(receipt)?)
        .map_err(|error| RunError::Internal(format!("{}: {}", error.code, error.detail)))
}

/// The exact error of a registry transfer that a port call stopped, once that stop classified as
/// failed.
fn stopped_transfer(outcome: ApplicationOutcome, error: CapabilityError, operation: &str, effect_id: &str) -> RunError {
    match outcome {
        ApplicationOutcome::Failed { .. } => RunError::Internal(error.detail),
        other => contradictory_artifact_observation(operation, effect_id, other, Some(&error)),
    }
}

/// The exact error of a registry transfer that target admission rejected, once the rejection is
/// classified.
fn rejected_transfer(plan: &EffectPlan, observed: &[Observation; 3], rejection: RunError, operation: &str) -> RunError {
    match classify_observations(plan, observed) {
        ApplicationOutcome::Failed { .. } => rejection,
        other => contradictory_artifact_observation(operation, ARTIFACT_REGISTRY_INPUT_EFFECT, other, None),
    }
}

/// Confirm that a transfer whose read, transfer, and receipt write all succeeded classifies as
/// complete.
fn recorded_transfer(plan: &EffectPlan, observed: &[Observation; 3], operation: &str) -> Result<(), RunError> {
    match classify_observations(plan, observed) {
        ApplicationOutcome::Completed => Ok(()),
        other => Err(contradictory_artifact_observation(operation, ARTIFACT_REGISTRY_RECEIPT_EFFECT, other, None)),
    }
}

/// The capability failure for an internal shell error, keeping its exact message.
///
/// Every adapter call wrapped here fails only with `RunError::Internal`.
fn refused_with(error: RunError) -> CapabilityError {
    debug_assert!(matches!(error, RunError::Internal(_)), "artifact adapters fail with internal errors");
    call_refused(match error {
        RunError::Internal(detail) => detail,
        other => other.to_string(),
    })
}

/// One admitted push: the loaded inputs and the validated target.
struct OciPushAdmitted {
    trust_policy: ValidatedRegistryTrustPolicy,
    signing_keys: Vec<crunch_build::KeyPair>,
    target: RegistryTarget,
}

/// Adapter for one `artifact oci-push`, scoped to its declared paths.
struct OciPushFiles<'a> {
    current_dir: &'a Path,
    state_dir: &'a Path,
    layout: &'a Path,
    receipt_out: &'a Path,
    trust_policy: &'a Path,
    signing_keys: &'a [PathBuf],
    bearer_token_file: Option<&'a Path>,
}

impl OciRegistryPushPort for OciPushFiles<'_> {
    type Inputs = (ValidatedRegistryTrustPolicy, Vec<crunch_build::KeyPair>);
    type Admitted = OciPushAdmitted;
    type Report = OciRegistryPushReport;

    fn read_inputs(&mut self) -> ArtifactPortCall<Self::Inputs> {
        executed_artifact_port(EffectKind::ReadFiles, |_| {
            require_absent_output(self.receipt_out, "OCI registry push receipt").map_err(refused_with)?;
            let policy_path = resolve_cli_path(self.current_dir, self.trust_policy);
            let trust_policy = load_registry_trust_policy(&policy_path)
                .map_err(|error| call_refused(format!("loading OCI registry trust policy: {error}")))?;
            let signing_keys = load_registry_signing_keys(self.current_dir, self.state_dir, self.signing_keys)
                .map_err(refused_with)?;
            Ok((trust_policy, signing_keys))
        })
    }

    fn push(&mut self, admitted: &Self::Admitted) -> ArtifactPortCall<Self::Report> {
        executed_artifact_port(EffectKind::UseNetwork, |_| {
            push_registry_layout(RegistryPushRequest {
                target: &admitted.target,
                layout_dir: self.layout,
                bearer_token_file: self.bearer_token_file,
                trust_policy: &admitted.trust_policy,
                signing_keys: &admitted.signing_keys,
            })
            .map_err(|error| call_refused(format!("pushing OCI registry layout: {error}")))
        })
    }

    fn write_receipt(&mut self, report: &Self::Report) -> ArtifactPortCall<()> {
        executed_artifact_port(EffectKind::WriteFiles, |fact| {
            write_json_output_new(self.receipt_out, report).map_err(refused_with)?;
            fact.output = EffectOutput::Identity(self.receipt_out.display().to_string());
            Ok(())
        })
    }
}

/// One admitted pull: the loaded trust policy and the validated target.
struct OciPullAdmitted {
    trust_policy: ValidatedRegistryTrustPolicy,
    target: RegistryTarget,
}

/// Adapter for one `artifact oci-pull`, scoped to its declared paths and expected digests.
struct OciPullFiles<'a> {
    current_dir: &'a Path,
    state_dir: &'a Path,
    output_dir: &'a Path,
    import_report_path: &'a Path,
    receipt_out: &'a Path,
    trust_policy: &'a Path,
    bearer_token_file: Option<&'a Path>,
    expected_manifest_digest: &'a str,
    expected_metadata_manifest_digest: &'a str,
    expected_signature_manifest_digest: &'a str,
}

impl OciRegistryPullPort for OciPullFiles<'_> {
    type Inputs = ValidatedRegistryTrustPolicy;
    type Admitted = OciPullAdmitted;
    type Report = OciRegistryPullReport;

    fn read_inputs(&mut self) -> ArtifactPortCall<Self::Inputs> {
        executed_artifact_port(EffectKind::ReadFiles, |_| {
            require_absent_output(self.receipt_out, "OCI registry pull receipt").map_err(refused_with)?;
            let policy_path = resolve_cli_path(self.current_dir, self.trust_policy);
            load_registry_trust_policy(&policy_path)
                .map_err(|error| call_refused(format!("loading OCI registry trust policy: {error}")))
        })
    }

    fn pull(&mut self, admitted: &Self::Admitted) -> ArtifactPortCall<Self::Report> {
        executed_artifact_port(EffectKind::UseNetwork, |_| {
            pull_registry_layout(RegistryPullRequest {
                target: &admitted.target,
                expected_manifest_digest: self.expected_manifest_digest,
                expected_metadata_manifest_digest: self.expected_metadata_manifest_digest,
                expected_signature_manifest_digest: self.expected_signature_manifest_digest,
                trust_policy: &admitted.trust_policy,
                output_dir: self.output_dir,
                state_dir: self.state_dir,
                import_report_path: self.import_report_path,
                bearer_token_file: self.bearer_token_file,
            })
            .map_err(|error| call_refused(format!("pulling OCI registry layout: {error}")))
        })
    }

    fn write_receipt(&mut self, report: &Self::Report) -> ArtifactPortCall<()> {
        executed_artifact_port(EffectKind::WriteFiles, |fact| {
            write_json_output_new(self.receipt_out, report).map_err(refused_with)?;
            fact.output = EffectOutput::Identity(self.receipt_out.display().to_string());
            Ok(())
        })
    }
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

/// Content facts one export resolution observed.
struct ResolvedExportContent {
    artifact_ref: String,
    artifact_digest: String,
    materialized_path: String,
}

/// Adapter for one `artifact export`, scoped to its declared attestation, content, and receipt
/// paths.
struct ArtifactExportFiles<'a> {
    current_dir: &'a Path,
    state_dir: &'a Path,
    artifact_ref: &'a str,
    artifact_digest: &'a str,
    attestation_path: &'a Path,
    materialized_path: Option<&'a Path>,
    out_path: Option<&'a Path>,
    receipt_out: Option<&'a Path>,
}

impl ArtifactExportPort for ArtifactExportFiles<'_> {
    type Attestation = FrontendArtifactAdmissionAttestation;
    type Content = Option<ResolvedExportContent>;
    type Receipt = FrontendArtifactExportReceipt;

    fn read_attestation(&mut self) -> ArtifactPortCall<Self::Attestation> {
        executed_artifact_port(EffectKind::ReadFiles, |fact| {
            let attestation = load_frontend_artifact_attestation(self.attestation_path).map_err(refused_with)?;
            fact.output = EffectOutput::Identity(self.attestation_path.display().to_string());
            Ok(attestation)
        })
    }

    fn resolve_content(&mut self) -> ArtifactPortCall<Self::Content> {
        if let Some(materialized_path) = self.materialized_path {
            return executed_artifact_port(EffectKind::ReadFiles, |fact| {
                let materialized_path = resolve_cli_path(self.current_dir, materialized_path);
                if !materialized_path.exists() {
                    return Ok(None);
                }
                let actual_path = materialized_path.display().to_string();
                fact.output = EffectOutput::Identity(actual_path.clone());
                Ok(Some(ResolvedExportContent {
                    artifact_ref: self.artifact_ref.to_string(),
                    artifact_digest: self.artifact_digest.to_string(),
                    materialized_path: actual_path,
                }))
            });
        }
        executed_artifact_port(EffectKind::StoreAccess, |fact| {
            let Some(out_path) = self.out_path else {
                return Err(call_refused(
                    "artifact export requires --materialized-path or storage-backed --out".to_string(),
                ));
            };
            let out_path = resolve_cli_path(self.current_dir, out_path);
            let stored = materialize_frontend_artifact(self.state_dir, self.artifact_ref, &out_path)
                .map_err(|err| call_refused(format!("materializing frontend artifact: {err}")))?;
            Ok(stored.map(|stored| {
                let actual_path = stored.content_path.display().to_string();
                fact.output = EffectOutput::Identity(actual_path.clone());
                ResolvedExportContent {
                    artifact_ref: stored.artifact_ref,
                    artifact_digest: stored.artifact_digest,
                    materialized_path: actual_path,
                }
            }))
        })
    }

    fn write_receipt(&mut self, receipt: &Self::Receipt) -> ArtifactPortCall<()> {
        executed_artifact_port(EffectKind::WriteFiles, |fact| {
            let Some(path) = self.receipt_out else {
                return Err(call_refused("artifact export has no requested receipt path".to_string()));
            };
            write_receipt(path, receipt).map_err(refused_with)?;
            if !path.is_file() {
                return Err(call_refused(format!("artifact export receipt did not read back: {}", path.display())));
            }
            fact.output = EffectOutput::Identity(path.display().to_string());
            Ok(())
        })
    }
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

/// Print an admitted export, after its observations classified it complete.
fn emit_exported_report(
    report: &FrontendArtifactExportReport,
    receipt: &FrontendArtifactExportReceipt,
    receipt_out: Option<&Path>,
    json: bool,
) -> Result<(), RunError> {
    debug_assert!(report.exported);
    debug_assert!(report.receipt.as_ref() == Some(receipt));
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
    if !path.is_file() {
        return Err(RunError::Internal(format!(
            "published registry receipt did not read back as a file: {}",
            path.display()
        )));
    }
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

    #[derive(Clone, Copy)]
    enum ExportFactFault {
        AttestationKind,
        MissingContentOutput,
    }

    /// If the first fact is ignored, these later effects leave visible evidence.
    struct AdversarialExportPort {
        attestation: Option<FrontendArtifactAdmissionAttestation>,
        attestation_path: PathBuf,
        copied: PathBuf,
        receipt: PathBuf,
        fault: ExportFactFault,
    }

    impl ArtifactExportPort for AdversarialExportPort {
        type Attestation = FrontendArtifactAdmissionAttestation;
        type Content = Option<ResolvedExportContent>;
        type Receipt = FrontendArtifactExportReceipt;

        fn read_attestation(&mut self) -> ArtifactPortCall<Self::Attestation> {
            ArtifactPortCall {
                result: Ok(self.attestation.take().expect("the attestation is read once")),
                fact: ArtifactPortFact {
                    kind: if matches!(self.fault, ExportFactFault::AttestationKind) {
                        EffectKind::StoreAccess
                    } else {
                        EffectKind::ReadFiles
                    },
                    output: EffectOutput::Identity(self.attestation_path.display().to_string()),
                    usage: EffectMeasure::Calls(1),
                },
            }
        }

        fn resolve_content(&mut self) -> ArtifactPortCall<Self::Content> {
            if matches!(self.fault, ExportFactFault::MissingContentOutput) {
                return ArtifactPortCall {
                    result: Ok(None),
                    fact: ArtifactPortFact {
                        kind: EffectKind::ReadFiles,
                        output: EffectOutput::Identity(self.copied.display().to_string()),
                        usage: EffectMeasure::Calls(1),
                    },
                };
            }
            std::fs::write(&self.copied, b"unauthorized store copy").expect("later store copy marker");
            executed_artifact_port(EffectKind::StoreAccess, |_| {
                Err(call_refused("unexpected content resolution".to_string()))
            })
        }

        fn write_receipt(&mut self, _receipt: &Self::Receipt) -> ArtifactPortCall<()> {
            std::fs::write(&self.receipt, b"unauthorized receipt").expect("later receipt marker");
            executed_artifact_port(EffectKind::WriteFiles, |_| {
                Err(call_refused("unexpected receipt write".to_string()))
            })
        }
    }

    #[test]
    fn contradictory_export_attestation_stops_before_store_copy_and_receipt() {
        let temporary = tempfile::tempdir().expect("temporary artifact root");
        let source = temporary.path().join("source");
        std::fs::write(&source, STORED_ARTIFACT_CONTENT).expect("source file");
        let stored = import_frontend_artifact(&source, temporary.path()).expect("store source");
        let attestation = temporary.path().join("attestation.json");
        let copied = temporary.path().join("unauthorized-copy");
        let receipt = temporary.path().join("unauthorized-receipt");
        let spec_hash = sample_spec_hash();
        let provenance = vec![SAMPLE_PROVENANCE_ENTRY.to_string()];
        let request = ArtifactExportShellRequest {
            current_dir: temporary.path(),
            state_dir: temporary.path(),
            json: true,
            artifact_ref: &stored.artifact_ref,
            attestation_path: &attestation,
            materialized_path: None,
            out_path: Some(&copied),
            artifact_digest: &stored.artifact_digest,
            spec_id: Some(SAMPLE_SPEC_ID),
            spec_version: Some(SAMPLE_SPEC_VERSION),
            spec_hash: Some(&spec_hash),
            destination_mode: FRONTEND_ARTIFACT_EXPORT_MODE_DIRECTORY,
            content_provenance: &provenance,
            receipt_out: Some(&receipt),
        };
        let mut port = AdversarialExportPort {
            attestation: Some(sample_attestation(&stored.artifact_ref, &stored.artifact_digest)),
            attestation_path: attestation.clone(),
            copied: copied.clone(),
            receipt: receipt.clone(),
            fault: ExportFactFault::AttestationKind,
        };
        let error = run_artifact_export(&request, &attestation, Some(&receipt), &mut port)
            .expect_err("a mismatched read cannot authorize the store copy");
        assert_eq!(error.exit_code(), std::process::ExitCode::from(3));
        assert!(matches!(error, RunError::Internal(detail)
            if detail.contains("artifact-observations-contradicted")
                && detail.contains("effect=read-attestation")));
        assert!(!copied.exists(), "the store-copy adapter was never called");
        assert!(!receipt.exists(), "the receipt adapter was never called");
    }

    #[test]
    fn missing_export_content_cannot_claim_a_written_path_or_reach_receipt() {
        let temporary = tempfile::tempdir().expect("temporary artifact root");
        let attestation = temporary.path().join("attestation.json");
        let missing = temporary.path().join("missing-content");
        let receipt = temporary.path().join("unauthorized-receipt");
        let digest = sample_digest(b"missing");
        let spec_hash = sample_spec_hash();
        let provenance = [SAMPLE_PROVENANCE_ENTRY.to_string()];
        let request = ArtifactExportShellRequest {
            current_dir: temporary.path(),
            state_dir: temporary.path(),
            json: true,
            artifact_ref: SAMPLE_ARTIFACT_REF,
            attestation_path: &attestation,
            materialized_path: Some(&missing),
            out_path: None,
            artifact_digest: &digest,
            spec_id: Some(SAMPLE_SPEC_ID),
            spec_version: Some(SAMPLE_SPEC_VERSION),
            spec_hash: Some(&spec_hash),
            destination_mode: FRONTEND_ARTIFACT_EXPORT_MODE_DIRECTORY,
            content_provenance: &provenance,
            receipt_out: Some(&receipt),
        };
        let mut port = AdversarialExportPort {
            attestation: Some(sample_attestation(SAMPLE_ARTIFACT_REF, &digest)),
            attestation_path: attestation.clone(),
            copied: missing.clone(),
            receipt: receipt.clone(),
            fault: ExportFactFault::MissingContentOutput,
        };
        let error = run_artifact_export(&request, &attestation, Some(&receipt), &mut port)
            .expect_err("an absent source must not claim the planned output path");
        assert_eq!(error.exit_code(), std::process::ExitCode::from(3));
        assert!(matches!(error, RunError::Internal(detail)
            if detail.contains("artifact-observations-contradicted")
                && detail.contains("effect=resolve-content")));
        assert!(!receipt.exists(), "the receipt adapter was never called");
    }

    struct AdversarialRegistryPort {
        trust: Option<ValidatedRegistryTrustPolicy>,
        keys: Vec<crunch_build::KeyPair>,
        transfer: PathBuf,
        store: PathBuf,
        receipt: PathBuf,
    }

    impl AdversarialRegistryPort {
        fn read_fact<R>(&self, result: R) -> ArtifactPortCall<R> {
            ArtifactPortCall {
                result: Ok(result),
                fact: ArtifactPortFact {
                    kind: EffectKind::ReadFiles,
                    output: EffectOutput::Identity(self.transfer.display().to_string()),
                    usage: EffectMeasure::Calls(1),
                },
            }
        }

        fn unexpected_transfer<R>(&self) -> ArtifactPortCall<R> {
            std::fs::write(&self.transfer, b"unauthorized registry transfer").expect("later transfer marker");
            std::fs::write(&self.store, b"unauthorized pulled store").expect("later store marker");
            executed_artifact_port(EffectKind::UseNetwork, |_| {
                Err(call_refused("unexpected registry transfer".to_string()))
            })
        }

        fn unexpected_receipt(&self) -> ArtifactPortCall<()> {
            std::fs::write(&self.receipt, b"unauthorized registry receipt").expect("later receipt marker");
            executed_artifact_port(EffectKind::WriteFiles, |_| {
                Err(call_refused("unexpected registry receipt".to_string()))
            })
        }
    }

    impl OciRegistryPushPort for AdversarialRegistryPort {
        type Inputs = (ValidatedRegistryTrustPolicy, Vec<crunch_build::KeyPair>);
        type Admitted = OciPushAdmitted;
        type Report = OciRegistryPushReport;

        fn read_inputs(&mut self) -> ArtifactPortCall<Self::Inputs> {
            let trust = self.trust.take().expect("trust is read once");
            let keys = std::mem::take(&mut self.keys);
            self.read_fact((trust, keys))
        }

        fn push(&mut self, _admitted: &Self::Admitted) -> ArtifactPortCall<Self::Report> {
            self.unexpected_transfer()
        }

        fn write_receipt(&mut self, _report: &Self::Report) -> ArtifactPortCall<()> {
            self.unexpected_receipt()
        }
    }

    impl OciRegistryPullPort for AdversarialRegistryPort {
        type Inputs = ValidatedRegistryTrustPolicy;
        type Admitted = OciPullAdmitted;
        type Report = OciRegistryPullReport;

        fn read_inputs(&mut self) -> ArtifactPortCall<Self::Inputs> {
            let trust = self.trust.take().expect("trust is read once");
            self.read_fact(trust)
        }

        fn pull(&mut self, _admitted: &Self::Admitted) -> ArtifactPortCall<Self::Report> {
            self.unexpected_transfer()
        }

        fn write_receipt(&mut self, _report: &Self::Report) -> ArtifactPortCall<()> {
            self.unexpected_receipt()
        }
    }

    fn adversarial_registry_port(transfer: PathBuf, store: PathBuf, receipt: PathBuf) -> AdversarialRegistryPort {
        use crate::oci_registry::OCI_REGISTRY_TRUST_POLICY_SCHEMA;
        use crate::oci_registry::RegistryTrustPolicy;
        use crate::oci_registry::validate_registry_trust_policy;

        let keypair = crunch_build::load_keypair(
            "cache.example.com-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==",
        ).expect("deterministic test signing key");
        let policy = RegistryTrustPolicy {
            schema: OCI_REGISTRY_TRUST_POLICY_SCHEMA.to_string(),
            schema_version: 1,
            trust_domain: "onix-kernel-bundle".to_string(),
            allowed_repositories: vec!["onix/kernel-bundle".to_string()],
            trusted_public_keys: vec![keypair.verifying_key.to_string()],
            required_signers: vec![keypair.verifying_key.name().to_string()],
            minimum_signatures: 1,
            revoked_public_key_blake3: Vec::new(),
        };
        AdversarialRegistryPort {
            trust: Some(validate_registry_trust_policy(policy).expect("policy admits one known key")),
            keys: vec![keypair],
            transfer,
            store,
            receipt,
        }
    }

    #[test]
    fn contradictory_push_input_stops_before_network_and_receipt() {
        let temporary = tempfile::tempdir().expect("temporary registry root");
        let layout = temporary.path().join("layout");
        let policy = temporary.path().join("policy");
        let transfer = temporary.path().join("unauthorized-push");
        let store = temporary.path().join("unauthorized-store");
        let receipt = temporary.path().join("unauthorized-receipt");
        let request = OciPushShellRequest {
            current_dir: temporary.path(),
            state_dir: temporary.path(),
            json: true,
            layout: &layout,
            registry: "https://registry.example.test",
            repository: "onix/kernel-bundle",
            reference: "reviewed",
            trust_policy: &policy,
            signing_keys: &[],
            bearer_token_file: None,
            allow_http: false,
            receipt_out: &receipt,
        };
        let mut port = adversarial_registry_port(transfer.clone(), store.clone(), receipt.clone());
        let error = run_oci_push(&request, &receipt, &mut port)
            .expect_err("a mismatched input read cannot authorize a registry push");
        assert_eq!(error.exit_code(), std::process::ExitCode::from(3));
        assert!(matches!(error, RunError::Internal(detail)
            if detail.contains("artifact-observations-contradicted")
                && detail.contains("effect=read-files")));
        assert!(!transfer.exists(), "the network adapter was never called");
        assert!(!store.exists(), "the transfer did not touch the store");
        assert!(!receipt.exists(), "the receipt adapter was never called");
    }

    #[test]
    fn contradictory_pull_input_stops_before_network_store_and_receipt() {
        let temporary = tempfile::tempdir().expect("temporary registry root");
        let policy = temporary.path().join("policy");
        let layout = temporary.path().join("unauthorized-layout");
        let store = temporary.path().join("unauthorized-store");
        let import_report = temporary.path().join("unauthorized-import-report");
        let receipt = temporary.path().join("unauthorized-receipt");
        let expected_digest = format!("sha256:{}", "a".repeat(64));
        let request = OciPullShellRequest {
            current_dir: temporary.path(),
            state_dir: temporary.path(),
            json: true,
            registry: "https://registry.example.test",
            repository: "onix/kernel-bundle",
            reference: "reviewed",
            expected_manifest_digest: &expected_digest,
            expected_metadata_manifest_digest: &expected_digest,
            expected_signature_manifest_digest: &expected_digest,
            trust_policy: &policy,
            bearer_token_file: None,
            allow_http: false,
            output_dir: &layout,
            import_report_out: &import_report,
            receipt_out: &receipt,
        };
        let mut port = adversarial_registry_port(layout.clone(), store.clone(), receipt.clone());
        let error = run_oci_pull(&request, &receipt, &mut port)
            .expect_err("a mismatched input read cannot authorize a registry pull");
        assert_eq!(error.exit_code(), std::process::ExitCode::from(3));
        assert!(matches!(error, RunError::Internal(detail)
            if detail.contains("artifact-observations-contradicted")
                && detail.contains("effect=read-files")));
        assert!(!layout.exists(), "the network adapter was never called");
        assert!(!store.exists(), "the pulled bytes were not admitted to the store");
        assert!(!import_report.exists(), "no import report was written");
        assert!(!receipt.exists(), "the receipt adapter was never called");
    }

    #[test]
    fn artifact_cli_rejects_wrong_port_kind_destination_and_usage_as_internal_errors() {
        let plan = planned_artifact_call(EffectKind::StoreAccess, Path::new("/bound/state")).unwrap();
        let correct = ArtifactPortFact {
            kind: EffectKind::StoreAccess,
            output: EffectOutput::Identity("/bound/state".to_string()),
            usage: EffectMeasure::Calls(1),
        };
        for wrong in [
            ArtifactPortFact {
                kind: EffectKind::ReadFiles,
                ..correct.clone()
            },
            ArtifactPortFact {
                output: EffectOutput::Identity("/other/state".to_string()),
                ..correct.clone()
            },
            ArtifactPortFact {
                usage: EffectMeasure::Calls(2),
                ..correct.clone()
            },
        ] {
            let (result, observation) = observed_port_call(ARTIFACT_CALL_EFFECT, ArtifactPortCall {
                result: Ok(()),
                fact: wrong,
            });
            let error = classified_artifact_call(&plan, observation, result, "artifact import")
                .expect_err("a port's contradictory facts must stop CLI reporting");
            assert_eq!(error.exit_code(), std::process::ExitCode::from(3));
            assert!(matches!(error, RunError::Internal(detail)
                if detail.contains("artifact-observations-contradicted")
                    && detail.contains("effect=artifact-call")
                    && detail.contains("Contradicted")));
        }
        let (result, observation) = observed_port_call(ARTIFACT_CALL_EFFECT, ArtifactPortCall::<()> {
            result: Err(CapabilityError::new("store-refused", "store path escaped its admitted root")),
            fact: ArtifactPortFact {
                output: EffectOutput::Identity("/other/state".to_string()),
                ..correct
            },
        });
        let error = classified_artifact_call(&plan, observation, result, "artifact import")
            .expect_err("a contradictory failed port must preserve its capability error");
        assert_eq!(error.exit_code(), std::process::ExitCode::from(3));
        assert!(matches!(error, RunError::Internal(detail)
            if detail.contains("artifact-observations-contradicted")
                && detail.contains("effect=artifact-call")
                && detail.contains("adapter=store-refused: store path escaped its admitted root")));
    }

    #[cfg(unix)]
    #[test]
    fn artifact_import_refuses_unrepresentable_state_root_before_touching_store() {
        use std::os::unix::ffi::OsStringExt;

        let temporary = tempfile::tempdir().expect("temporary import root");
        let source = temporary.path().join("artifact.txt");
        std::fs::write(&source, STORED_ARTIFACT_CONTENT).expect("source file");
        let raw_name = std::ffi::OsString::from_vec(b"state-\xff".to_vec());
        let state_dir = temporary.path().join(raw_name);
        let error = cmd_artifact(
            crate::ArtifactAction::Import {
                path: source,
                report_out: None,
            },
            temporary.path(),
            &state_dir,
            true,
        )
        .expect_err("an unrepresentable output identity must be refused before store access");
        assert!(matches!(error, RunError::Internal(detail)
            if detail.contains("artifact-effect-identity-non-utf8")));
        assert!(!state_dir.exists(), "no store is opened before identity admission");
    }

    #[cfg(unix)]
    #[test]
    fn artifact_operations_refuse_non_utf8_effect_authority_before_any_port_call() {
        use std::os::unix::ffi::OsStringExt;

        let temporary = tempfile::tempdir().expect("temporary artifact root");
        let state_dir = temporary.path().join("state");
        let missing = temporary.path().join("missing");
        let invalid = temporary.path().join(std::ffi::OsString::from_vec(b"output-\xff".to_vec()));
        let cases = [
            (
                "oci-export",
                crate::ArtifactAction::OciExport {
                    projection: missing.clone(),
                    spec_material: missing.clone(),
                    source_admissions: missing.clone(),
                    out: invalid.clone(),
                },
                state_dir.as_path(),
            ),
            (
                "oci-import",
                crate::ArtifactAction::OciImport {
                    layout: missing.clone(),
                    report_out: temporary.path().join("import-report.json"),
                },
                invalid.as_path(),
            ),
            (
                "oci-push",
                crate::ArtifactAction::OciPush {
                    layout: missing.clone(),
                    registry: "registry.invalid".to_string(),
                    repository: "repo".to_string(),
                    reference: "tag".to_string(),
                    trust_policy: missing.clone(),
                    signing_keys: Vec::new(),
                    bearer_token_file: None,
                    allow_http: false,
                    receipt_out: invalid.clone(),
                },
                state_dir.as_path(),
            ),
            (
                "oci-pull",
                crate::ArtifactAction::OciPull {
                    registry: "registry.invalid".to_string(),
                    repository: "repo".to_string(),
                    reference: "tag".to_string(),
                    expected_manifest_digest: String::new(),
                    expected_metadata_manifest_digest: String::new(),
                    expected_signature_manifest_digest: String::new(),
                    trust_policy: missing.clone(),
                    bearer_token_file: None,
                    allow_http: false,
                    out: temporary.path().join("layout"),
                    report_out: temporary.path().join("pull-report.json"),
                    receipt_out: invalid.clone(),
                },
                state_dir.as_path(),
            ),
            (
                "export-attestation",
                legacy_export_action(invalid.clone(), missing.clone(), None, sample_digest(b"unread")),
                state_dir.as_path(),
            ),
            (
                "export-materialized",
                legacy_export_action(missing.clone(), invalid.clone(), None, sample_digest(b"unread")),
                state_dir.as_path(),
            ),
            (
                "export-receipt",
                legacy_export_action(missing.clone(), missing.clone(), Some(invalid.clone()), sample_digest(b"unread")),
                state_dir.as_path(),
            ),
            (
                "export-store-copy",
                crate::ArtifactAction::Export {
                    artifact_ref: SAMPLE_ARTIFACT_REF.to_string(),
                    attestation: missing.clone(),
                    materialized_path: None,
                    out: Some(invalid.clone()),
                    artifact_digest: sample_digest(b"unread"),
                    spec_id: Some(SAMPLE_SPEC_ID.to_string()),
                    spec_version: Some(SAMPLE_SPEC_VERSION.to_string()),
                    spec_hash: Some(sample_spec_hash()),
                    destination_mode: FRONTEND_ARTIFACT_EXPORT_MODE_DIRECTORY.to_string(),
                    content_provenance: vec![SAMPLE_PROVENANCE_ENTRY.to_string()],
                    receipt_out: None,
                },
                state_dir.as_path(),
            ),
        ];
        for (operation, action, state) in cases {
            let error = cmd_artifact(action, temporary.path(), state, true)
                .expect_err("unrepresentable effect identity must stop before a port executes");
            assert!(
                matches!(error, RunError::Internal(detail)
                if detail.contains("artifact-effect-identity-non-utf8")),
                "{operation}"
            );
            assert!(!invalid.exists(), "{operation} must not create the bound path");
            assert!(!state_dir.exists(), "{operation} must not touch the store");
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

    #[test]
    fn artifact_import_of_a_missing_source_writes_neither_store_nor_report() {
        let temp = tempfile::tempdir().expect("tempdir");
        let state_dir = temp.path().join("state");
        let report_path = temp.path().join("report.json");

        let err = cmd_artifact(
            crate::ArtifactAction::Import {
                path: temp.path().join("missing-source"),
                report_out: Some(report_path.clone()),
            },
            temp.path(),
            &state_dir,
            true,
        )
        .expect_err("a missing source must fail the import");

        assert!(
            matches!(&err, RunError::Internal(message) if message.starts_with("importing frontend artifact: ")),
            "{err:?}"
        );
        assert!(!state_dir.exists(), "a failed source read must not touch the store");
        assert!(!report_path.exists(), "a failed store import must not write a report");
    }

    #[test]
    fn artifact_import_report_write_failure_fails_after_the_store_import() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = temp.path().join("artifact.txt");
        let state_dir = temp.path().join("state");
        let blocking_file = temp.path().join("report-parent-is-a-file");
        let report_path = blocking_file.join("report.json");
        std::fs::write(&source, STORED_ARTIFACT_CONTENT).expect("write source");
        std::fs::write(&blocking_file, b"not a directory").expect("write blocking file");

        let err = cmd_artifact(
            crate::ArtifactAction::Import {
                path: source.clone(),
                report_out: Some(report_path.clone()),
            },
            temp.path(),
            &state_dir,
            false,
        )
        .expect_err("an unwritable report path must fail the import");

        assert!(matches!(&err, RunError::Internal(message) if message.starts_with("creating ")), "{err:?}");
        assert!(!report_path.exists());
        let artifact_ref =
            crate::frontend_artifact_store::frontend_artifact_identity(&source).expect("artifact identity");
        assert!(
            crate::frontend_artifact_store::frontend_artifact_is_available(&state_dir, &artifact_ref)
                .expect("store availability"),
            "the store import completes before the report write fails"
        );
    }

    #[test]
    fn artifact_import_stores_the_source_identity_and_reports_it() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = temp.path().join("artifact.txt");
        let state_dir = temp.path().join("state");
        let report_path = temp.path().join("reports").join("report.json");
        std::fs::write(&source, STORED_ARTIFACT_CONTENT).expect("write source");
        let artifact_ref =
            crate::frontend_artifact_store::frontend_artifact_identity(&source).expect("artifact identity");
        assert!(
            !crate::frontend_artifact_store::frontend_artifact_is_available(&state_dir, &artifact_ref)
                .expect("store availability"),
            "the store starts without the artifact"
        );

        cmd_artifact(
            crate::ArtifactAction::Import {
                path: source,
                report_out: Some(report_path.clone()),
            },
            temp.path(),
            &state_dir,
            true,
        )
        .expect("artifact import succeeds");

        let report: FrontendArtifactStoreImportReport =
            serde_json::from_slice(&std::fs::read(&report_path).expect("read report")).expect("parse report");
        assert_eq!(report.artifact_ref, artifact_ref);
        assert!(
            crate::frontend_artifact_store::frontend_artifact_is_available(&state_dir, &artifact_ref)
                .expect("store availability"),
            "a successful import leaves the artifact in the store"
        );
        assert_eq!(std::fs::read(&report.stored_content_path).expect("read stored content"), STORED_ARTIFACT_CONTENT);
    }

    #[test]
    fn artifact_oci_export_into_an_existing_output_fails_and_leaves_it_untouched() {
        let temp = tempfile::tempdir().expect("tempdir");
        let out = temp.path().join("existing-layout");
        std::fs::create_dir_all(&out).expect("existing output dir");
        std::fs::write(out.join("unrelated.txt"), b"keep me").expect("existing output file");

        let err = cmd_artifact(
            crate::ArtifactAction::OciExport {
                projection: temp.path().join("projection.json"),
                spec_material: temp.path().join("spec.json"),
                source_admissions: temp.path().join("admissions.json"),
                out: out.clone(),
            },
            temp.path(),
            &temp.path().join("state"),
            true,
        )
        .expect_err("an existing output must fail the export");

        assert!(
            matches!(&err, RunError::Internal(message)
                if message.starts_with("exporting OCI layout: OCI output already exists: ")),
            "{err:?}"
        );
        let entries = std::fs::read_dir(&out).expect("list output").count();
        assert_eq!(entries, 1, "the export must not add to an existing output");
        assert_eq!(std::fs::read(out.join("unrelated.txt")).expect("read kept file"), b"keep me");
    }

    #[test]
    fn artifact_oci_import_of_a_missing_layout_writes_no_report() {
        let temp = tempfile::tempdir().expect("tempdir");
        let report_path = temp.path().join("import-report.json");

        let err = cmd_artifact(
            crate::ArtifactAction::OciImport {
                layout: temp.path().join("missing-layout"),
                report_out: report_path.clone(),
            },
            temp.path(),
            &temp.path().join("state"),
            false,
        )
        .expect_err("a missing layout must fail the import");

        assert!(
            matches!(&err, RunError::Internal(message) if message.starts_with("importing OCI layout: ")),
            "{err:?}"
        );
        assert!(!report_path.exists(), "a refused layout import must not write its report");
    }

    #[test]
    fn artifact_oci_push_with_a_taken_receipt_stops_before_trust_inputs_and_keeps_the_receipt() {
        let temp = tempfile::tempdir().expect("tempdir");
        let receipt = temp.path().join("push-receipt.json");
        std::fs::write(&receipt, b"existing receipt").expect("existing receipt");

        let err = cmd_artifact(
            crate::ArtifactAction::OciPush {
                layout: temp.path().join("layout"),
                registry: "http://127.0.0.1:9".to_string(),
                repository: "mantle/demo".to_string(),
                reference: "v1".to_string(),
                trust_policy: temp.path().join("missing-policy.ncl"),
                signing_keys: vec![temp.path().join("missing-key")],
                bearer_token_file: None,
                allow_http: true,
                receipt_out: receipt.clone(),
            },
            temp.path(),
            &temp.path().join("state"),
            true,
        )
        .expect_err("a taken receipt path must stop the push");

        let expected = format!("OCI registry push receipt already exists: {}", receipt.display());
        assert!(matches!(&err, RunError::Internal(message) if *message == expected), "{err:?}");
        assert_eq!(std::fs::read(&receipt).expect("read receipt"), b"existing receipt");
    }

    #[test]
    fn artifact_oci_pull_with_an_unloadable_trust_policy_writes_no_outputs() {
        let temp = tempfile::tempdir().expect("tempdir");
        let policy = temp.path().join("policy.json");
        std::fs::write(&policy, b"{}").expect("policy file");
        let out = temp.path().join("pulled-layout");
        let report = temp.path().join("import-report.json");
        let receipt = temp.path().join("pull-receipt.json");

        let err = cmd_artifact(
            crate::ArtifactAction::OciPull {
                registry: "http://127.0.0.1:9".to_string(),
                repository: "mantle/demo".to_string(),
                reference: "v1".to_string(),
                expected_manifest_digest: "sha256:missing".to_string(),
                expected_metadata_manifest_digest: "sha256:missing".to_string(),
                expected_signature_manifest_digest: "sha256:missing".to_string(),
                trust_policy: policy,
                bearer_token_file: None,
                allow_http: true,
                out: out.clone(),
                report_out: report.clone(),
                receipt_out: receipt.clone(),
            },
            temp.path(),
            &temp.path().join("state"),
            false,
        )
        .expect_err("an unloadable trust policy must stop the pull");

        assert!(
            matches!(&err, RunError::Internal(message) if message.starts_with("loading OCI registry trust policy: ")),
            "{err:?}"
        );
        for path in [&out, &report, &receipt] {
            assert!(!path.exists(), "a refused pull must not create {}", path.display());
        }
    }

    fn stored_export_action(
        temp: &Path,
        attestation: &FrontendArtifactAdmissionAttestation,
        artifact_digest: &str,
        receipt_out: Option<PathBuf>,
    ) -> crate::ArtifactAction {
        let attestation_path = temp.join("stored-attestation.json");
        write_json(&attestation_path, attestation);
        crate::ArtifactAction::Export {
            artifact_ref: attestation.artifact_ref.clone(),
            attestation: attestation_path,
            materialized_path: None,
            out: Some(temp.join("exported").join("artifact.txt")),
            artifact_digest: artifact_digest.to_string(),
            spec_id: Some(SAMPLE_SPEC_ID.to_string()),
            spec_version: Some(SAMPLE_SPEC_VERSION.to_string()),
            spec_hash: Some(sample_spec_hash()),
            destination_mode: FRONTEND_ARTIFACT_EXPORT_MODE_DIRECTORY.to_string(),
            content_provenance: vec![SAMPLE_PROVENANCE_ENTRY.to_string()],
            receipt_out,
        }
    }

    #[test]
    fn artifact_export_with_an_unreadable_attestation_resolves_no_content() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = temp.path().join("artifact.txt");
        std::fs::write(&source, STORED_ARTIFACT_CONTENT).expect("write source");
        let import_report = import_frontend_artifact(&source, temp.path()).expect("import artifact");
        let exported = temp.path().join("exported").join("artifact.txt");
        let missing = temp.path().join("missing-attestation.json");

        let err = cmd_artifact(
            crate::ArtifactAction::Export {
                artifact_ref: import_report.artifact_ref,
                attestation: missing.clone(),
                materialized_path: None,
                out: Some(exported.clone()),
                artifact_digest: import_report.artifact_digest,
                spec_id: None,
                spec_version: None,
                spec_hash: None,
                destination_mode: FRONTEND_ARTIFACT_EXPORT_MODE_DIRECTORY.to_string(),
                content_provenance: Vec::new(),
                receipt_out: Some(temp.path().join("receipt.json")),
            },
            temp.path(),
            temp.path(),
            false,
        )
        .expect_err("an unreadable attestation must stop the export");

        let expected_prefix = format!("reading {}: ", missing.display());
        assert!(matches!(&err, RunError::Internal(message) if message.starts_with(&expected_prefix)), "{err:?}");
        assert!(!exported.exists(), "no content may be resolved before the attestation is read");
        assert!(!temp.path().join("receipt.json").exists());
    }

    #[test]
    fn artifact_export_receipt_write_failure_keeps_the_store_copy_and_writes_no_receipt() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = temp.path().join("artifact.txt");
        std::fs::write(&source, STORED_ARTIFACT_CONTENT).expect("write source");
        let import_report = import_frontend_artifact(&source, temp.path()).expect("import artifact");
        let attestation = sample_attestation(&import_report.artifact_ref, &import_report.artifact_digest);
        let blocking_file = temp.path().join("receipt-parent-is-a-file");
        std::fs::write(&blocking_file, b"not a directory").expect("write blocking file");
        let receipt = blocking_file.join("receipt.json");

        let err = cmd_artifact(
            stored_export_action(temp.path(), &attestation, &import_report.artifact_digest, Some(receipt.clone())),
            temp.path(),
            temp.path(),
            true,
        )
        .expect_err("an unwritable receipt path must fail the export");

        assert!(matches!(&err, RunError::Internal(message) if message.starts_with("creating ")), "{err:?}");
        let exported = temp.path().join("exported").join("artifact.txt");
        assert_eq!(std::fs::read(&exported).expect("read store copy"), STORED_ARTIFACT_CONTENT);
        assert!(!receipt.exists());
    }

    #[test]
    fn artifact_export_of_inadmissible_store_content_keeps_the_copy_and_reports_exit_one() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = temp.path().join("artifact.txt");
        std::fs::write(&source, STORED_ARTIFACT_CONTENT).expect("write source");
        let import_report = import_frontend_artifact(&source, temp.path()).expect("import artifact");
        let other_digest = sample_digest(b"another artifact");
        let attestation = sample_attestation(&import_report.artifact_ref, &other_digest);
        let receipt = temp.path().join("receipt.json");

        let err = cmd_artifact(
            stored_export_action(temp.path(), &attestation, &import_report.artifact_digest, Some(receipt.clone())),
            temp.path(),
            temp.path(),
            true,
        )
        .expect_err("content whose digest differs from the attestation must be rejected");

        assert!(matches!(err, RunError::Reported(EXPORT_FAILURE_EXIT_CODE)), "{err:?}");
        let exported = temp.path().join("exported").join("artifact.txt");
        assert_eq!(
            std::fs::read(&exported).expect("read store copy"),
            STORED_ARTIFACT_CONTENT,
            "the store copy runs before the export decision"
        );
        assert!(!receipt.exists(), "a rejected export must not write its receipt");
    }
}
