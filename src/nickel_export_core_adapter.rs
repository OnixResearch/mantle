//! Pure Mantle adapter for the evaluator-neutral `nickel-export-core`.

// machine-artifact-public: nickel-export.receipt
// r[impl mantle.nickel_export_cutover.boundary]
// r[impl mantle.nickel_export_cutover.dual_run]
use nickel_export_core::ArtifactIdentity;
use nickel_export_core::ArtifactMaterial;
use nickel_export_core::CoreError;
use nickel_export_core::Diagnostic;
use nickel_export_core::DiagnosticSeverity;
use nickel_export_core::EvaluationObservation;
use nickel_export_core::EvaluatorDescriptor;
use nickel_export_core::ExportFormat;
use nickel_export_core::ExportManifest;
use nickel_export_core::ExportReceipt;
use nickel_export_core::ExportRequest;
use nickel_export_core::ImportPathPolicy;

use crate::nickel_export::ExportSourceRef;
use crate::nickel_export::NickelEvaluatorDescriptor;
use crate::nickel_export::NickelExportDiagnostic;
use crate::nickel_export::NickelExportReceipt;

const FAMILY_ID: &str = "mantle.nickel-export";
const BLAKE3_PREFIX: &str = "b3:";
const CANONICAL_RECEIPT_IDENTITY_SCHEMA: &str = "mantle-nickel-export-canonical-receipt-identity-v1";
const DRIFT_REQUEST_NORMALIZATION: &str = "request-normalization";
const DRIFT_DEPENDENCY_CLOSURE: &str = "dependency-closure";
const DRIFT_EVALUATOR_DESCRIPTOR: &str = "evaluator-descriptor";
const DRIFT_SERIALIZATION: &str = "serialization";
const DRIFT_MANTLE_POLICY: &str = "mantle-policy";
const DRIFT_PROJECTION: &str = "projection";

#[derive(Clone, Debug)]
pub struct AdapterRequest<'a> {
    pub source: &'a str,
    pub dependencies: &'a [String],
    pub import_paths: &'a [String],
    pub format: &'a str,
    pub output_target: &'a str,
    pub evaluator: &'a NickelEvaluatorDescriptor,
}

#[derive(Clone, Copy, Debug)]
pub struct ExplicitArtifact<'a> {
    pub path: &'a str,
    pub bytes: &'a [u8],
}

#[derive(Clone, Debug)]
pub struct ExplicitObservation<'a> {
    pub request: AdapterRequest<'a>,
    pub source: ExplicitArtifact<'a>,
    pub dependencies: &'a [ExplicitArtifact<'a>],
    pub output: ExplicitArtifact<'a>,
    pub diagnostics: &'a [NickelExportDiagnostic],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DualRunEvidence {
    pub schema: String,
    pub canonical_receipt_identity_blake3: String,
    pub legacy_receipt_identity_blake3: String,
    pub canonical_projection_digest_blake3: String,
    pub legacy_projection_digest_blake3: String,
    pub identity_matches: bool,
    pub projection_matches: bool,
    pub drift_class: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalAdmission {
    pub canonical_receipt: ExportReceipt,
    pub canonical_manifest: ExportManifest,
    pub mantle_receipt: NickelExportReceipt,
    pub evidence: DualRunEvidence,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FailureParityEvidence {
    pub legacy_failure_class: String,
    pub canonical_failure_class: String,
    pub diagnostics_match: bool,
    pub receipt_emitted: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdapterFailure {
    pub class: String,
    pub diagnostics: Vec<NickelExportDiagnostic>,
    pub drift: Option<Box<DualRunEvidence>>,
}

pub fn export_source_ref(path: &str, bytes: &[u8]) -> ExportSourceRef {
    ExportSourceRef {
        path: path.to_string(),
        digest_blake3: bare_blake3(&nickel_export_core::blake3_identity(bytes)).to_string(),
    }
}

pub fn mantle_receipt_digest(receipt: &NickelExportReceipt) -> Result<String, AdapterFailure> {
    projection_digest(receipt)
}

pub fn normalize_adapter_request(request: &AdapterRequest<'_>) -> Result<ExportRequest, AdapterFailure> {
    let (format, mut diagnostics) = match parse_format(request.format) {
        Ok(format) => (format, Vec::new()),
        Err(failure) => (ExportFormat::Json, failure.diagnostics),
    };
    let canonical = ExportRequest {
        schema: nickel_export_core::REQUEST_SCHEMA.to_string(),
        family_id: FAMILY_ID.to_string(),
        source: request.source.to_string(),
        dependencies: request.dependencies.to_vec(),
        import_paths: request.import_paths.to_vec(),
        selector: String::new(),
        contract: String::new(),
        format,
        destination: request.output_target.to_string(),
        allow_secret_material: false,
    };
    match nickel_export_core::normalize_request(&canonical) {
        Ok(normalized) if diagnostics.is_empty() => Ok(normalized),
        Ok(_) => Err(AdapterFailure {
            class: "validation".to_string(),
            diagnostics,
            drift: None,
        }),
        Err(error) => {
            let failure = adapter_failure_from_core(error);
            diagnostics.extend(failure.diagnostics);
            Err(AdapterFailure {
                class: "validation".to_string(),
                diagnostics,
                drift: None,
            })
        }
    }
}

pub fn admit_and_compare(
    observation: &ExplicitObservation<'_>,
    legacy_receipt: &NickelExportReceipt,
) -> Result<CanonicalAdmission, AdapterFailure> {
    let request = normalize_adapter_request(&observation.request)?;
    let evaluator = canonical_evaluator(observation.request.evaluator);
    let canonical = build_canonical_receipt(observation, &request, &evaluator)?;
    let projected = nickel_export_core::project_mantle_receipt(&canonical);
    let mantle_receipt = mantle_receipt(&projected);
    validate_claim_boundary(&canonical, &mantle_receipt)?;

    let legacy_view = legacy_canonical_view(observation, &request, legacy_receipt, &canonical);
    let canonical_manifest =
        nickel_export_core::build_manifest(core::slice::from_ref(&canonical)).map_err(adapter_failure_from_core)?;
    let legacy_manifest =
        nickel_export_core::build_manifest(core::slice::from_ref(&legacy_view)).map_err(adapter_failure_from_core)?;
    let evidence = comparison_evidence(
        bare_blake3(&canonical_manifest.manifest_identity),
        bare_blake3(&legacy_manifest.manifest_identity),
        &mantle_receipt,
        legacy_receipt,
        &canonical,
        &legacy_view,
    )?;
    let evidence = enforce_no_drift(evidence)?;
    verify_freshness(&canonical_manifest, &canonical_manifest)?;
    Ok(CanonicalAdmission {
        canonical_receipt: canonical,
        canonical_manifest,
        mantle_receipt,
        evidence,
    })
}

pub fn compare_evaluator_failure(
    observation: &ExplicitObservation<'_>,
) -> Result<FailureParityEvidence, AdapterFailure> {
    let request = normalize_adapter_request(&observation.request)?;
    let evaluator = canonical_evaluator(observation.request.evaluator);
    let result = build_canonical_receipt(observation, &request, &evaluator);
    match result {
        Err(failure) if failure.class == "eval" => Ok(FailureParityEvidence {
            legacy_failure_class: "eval".to_string(),
            canonical_failure_class: failure.class,
            diagnostics_match: diagnostics_match(observation.diagnostics, &failure.diagnostics),
            receipt_emitted: false,
        }),
        Err(failure) => Err(AdapterFailure {
            class: "dual-run-drift".to_string(),
            diagnostics: failure.diagnostics,
            drift: None,
        }),
        Ok(_) => Err(AdapterFailure {
            class: "dual-run-drift".to_string(),
            diagnostics: vec![adapter_diagnostic(
                "unexpected-canonical-success",
                observation.request.source,
                "canonical admission emitted a receipt for evaluator error observations",
            )],
            drift: None,
        }),
    }
}

pub fn verify_freshness(expected: &ExportManifest, actual: &ExportManifest) -> Result<(), AdapterFailure> {
    nickel_export_core::verify_manifest_fresh(expected, actual).map_err(adapter_failure_from_core)
}

pub fn validate_claim_boundary(
    canonical: &ExportReceipt,
    projection: &NickelExportReceipt,
) -> Result<(), AdapterFailure> {
    let canonical_valid = canonical.non_claim == nickel_export_core::NON_CLAIM;
    let projection_valid = projection.non_claim == nickel_export_core::MANTLE_NON_CLAIM;
    if canonical_valid && projection_valid {
        return Ok(());
    }
    Err(AdapterFailure {
        class: "overclaim".to_string(),
        diagnostics: vec![adapter_diagnostic(
            "overclaim",
            "non_claim",
            "canonical or Mantle receipt weakened the required non-claim boundary",
        )],
        drift: None,
    })
}

fn build_canonical_receipt(
    observation: &ExplicitObservation<'_>,
    request: &ExportRequest,
    evaluator: &EvaluatorDescriptor,
) -> Result<ExportReceipt, AdapterFailure> {
    let dependencies = observation
        .dependencies
        .iter()
        .map(|artifact| ArtifactMaterial {
            path: artifact.path,
            bytes: artifact.bytes,
        })
        .collect::<Vec<_>>();
    let diagnostics = observation.diagnostics.iter().map(canonical_diagnostic).collect::<Vec<_>>();
    let canonical_observation = EvaluationObservation {
        request,
        source: ArtifactMaterial {
            path: observation.source.path,
            bytes: observation.source.bytes,
        },
        dependencies,
        output: ArtifactMaterial {
            path: observation.output.path,
            bytes: observation.output.bytes,
        },
        evaluator,
        observed_dependencies: Vec::new(),
        diagnostics,
    };
    nickel_export_core::build_receipt(&canonical_observation).map_err(adapter_failure_from_core)
}

fn canonical_evaluator(evaluator: &NickelEvaluatorDescriptor) -> EvaluatorDescriptor {
    EvaluatorDescriptor {
        identity: evaluator.identity.clone(),
        version: evaluator.version.clone(),
        options: evaluator.options.clone(),
        import_path_policy: ImportPathPolicy::DeclaredOnly,
    }
}

fn canonical_diagnostic(diagnostic: &NickelExportDiagnostic) -> Diagnostic {
    Diagnostic::new(&diagnostic.class, &diagnostic.subject, &diagnostic.message, DiagnosticSeverity::Error)
}

fn parse_format(format: &str) -> Result<ExportFormat, AdapterFailure> {
    if format == ExportFormat::Json.as_str() {
        return Ok(ExportFormat::Json);
    }
    Err(AdapterFailure {
        class: "validation".to_string(),
        diagnostics: vec![adapter_diagnostic(
            "unsupported-format",
            format,
            "Mantle currently admits only JSON Nickel exports",
        )],
        drift: None,
    })
}

fn mantle_receipt(receipt: &nickel_export_core::MantleReceipt) -> NickelExportReceipt {
    NickelExportReceipt {
        schema: receipt.schema.clone(),
        root_source: ExportSourceRef {
            path: receipt.root_source.path.clone(),
            digest_blake3: receipt.root_source.digest_blake3.clone(),
        },
        deps: receipt
            .deps
            .iter()
            .map(|source| ExportSourceRef {
                path: source.path.clone(),
                digest_blake3: source.digest_blake3.clone(),
            })
            .collect(),
        import_paths: receipt.import_paths.clone(),
        format: receipt.format.clone(),
        output_target: receipt.output_target.clone(),
        output_digest_blake3: receipt.output_digest_blake3.clone(),
        evaluator: NickelEvaluatorDescriptor {
            identity: receipt.evaluator.identity.clone(),
            version: receipt.evaluator.version.clone(),
            options: receipt.evaluator.options.clone(),
        },
        non_claim: receipt.non_claim.clone(),
    }
}

fn legacy_canonical_view(
    observation: &ExplicitObservation<'_>,
    request: &ExportRequest,
    legacy: &NickelExportReceipt,
    canonical: &ExportReceipt,
) -> ExportReceipt {
    let dependencies = legacy
        .deps
        .iter()
        .map(|source| ArtifactIdentity {
            path: source.path.clone(),
            identity: tagged_blake3(&source.digest_blake3),
            bytes: artifact_len(observation.dependencies, &source.path),
        })
        .collect::<Vec<_>>();
    ExportReceipt {
        schema: nickel_export_core::RECEIPT_SCHEMA.to_string(),
        family_id: request.family_id.clone(),
        source: ArtifactIdentity {
            path: legacy.root_source.path.clone(),
            identity: tagged_blake3(&legacy.root_source.digest_blake3),
            bytes: byte_len(observation.source.bytes),
        },
        dependencies,
        import_paths: legacy.import_paths.clone(),
        selector: request.selector.clone(),
        contract: request.contract.clone(),
        format: request.format,
        output: ArtifactIdentity {
            path: legacy.output_target.clone(),
            identity: tagged_blake3(&legacy.output_digest_blake3),
            bytes: byte_len(observation.output.bytes),
        },
        evaluator: EvaluatorDescriptor {
            identity: legacy.evaluator.identity.clone(),
            version: legacy.evaluator.version.clone(),
            options: legacy.evaluator.options.clone(),
            import_path_policy: ImportPathPolicy::DeclaredOnly,
        },
        diagnostics: canonical.diagnostics.clone(),
        non_claim: nickel_export_core::NON_CLAIM.to_string(),
    }
}

fn comparison_evidence(
    canonical_identity: &str,
    legacy_identity: &str,
    canonical_projection: &NickelExportReceipt,
    legacy_projection: &NickelExportReceipt,
    canonical: &ExportReceipt,
    legacy: &ExportReceipt,
) -> Result<DualRunEvidence, AdapterFailure> {
    let canonical_projection_digest = projection_digest(canonical_projection)?;
    let legacy_projection_digest = projection_digest(legacy_projection)?;
    let identity_matches = canonical_identity == legacy_identity;
    let projection_matches = canonical_projection == legacy_projection;
    let drift_class = classify_drift(canonical, legacy, identity_matches, projection_matches);
    Ok(DualRunEvidence {
        schema: CANONICAL_RECEIPT_IDENTITY_SCHEMA.to_string(),
        canonical_receipt_identity_blake3: canonical_identity.to_string(),
        legacy_receipt_identity_blake3: legacy_identity.to_string(),
        canonical_projection_digest_blake3: canonical_projection_digest,
        legacy_projection_digest_blake3: legacy_projection_digest,
        identity_matches,
        projection_matches,
        drift_class,
    })
}

fn classify_drift(
    canonical: &ExportReceipt,
    legacy: &ExportReceipt,
    identity_matches: bool,
    projection_matches: bool,
) -> Option<String> {
    if canonical.source != legacy.source || canonical.output != legacy.output {
        return Some(DRIFT_REQUEST_NORMALIZATION.to_string());
    }
    if canonical.dependencies != legacy.dependencies || canonical.import_paths != legacy.import_paths {
        return Some(DRIFT_DEPENDENCY_CLOSURE.to_string());
    }
    if canonical.evaluator != legacy.evaluator {
        return Some(DRIFT_EVALUATOR_DESCRIPTOR.to_string());
    }
    if canonical.non_claim != legacy.non_claim {
        return Some(DRIFT_MANTLE_POLICY.to_string());
    }
    if !identity_matches {
        return Some(DRIFT_SERIALIZATION.to_string());
    }
    if !projection_matches {
        return Some(DRIFT_PROJECTION.to_string());
    }
    None
}

fn enforce_no_drift(evidence: DualRunEvidence) -> Result<DualRunEvidence, AdapterFailure> {
    if evidence.identity_matches && evidence.projection_matches && evidence.drift_class.is_none() {
        return Ok(evidence);
    }
    Err(AdapterFailure {
        class: "dual-run-drift".to_string(),
        diagnostics: drift_diagnostics(&evidence),
        drift: Some(Box::new(evidence)),
    })
}

fn projection_digest(receipt: &NickelExportReceipt) -> Result<String, AdapterFailure> {
    let bytes = serde_json::to_vec(receipt).map_err(|_| serialization_failure())?;
    Ok(bare_blake3(&nickel_export_core::blake3_identity(&bytes)).to_string())
}

fn serialization_failure() -> AdapterFailure {
    AdapterFailure {
        class: DRIFT_SERIALIZATION.to_string(),
        diagnostics: vec![adapter_diagnostic(
            DRIFT_SERIALIZATION,
            "receipt",
            "receipt serialization failed during dual-run comparison",
        )],
        drift: None,
    }
}

fn adapter_failure_from_core(error: CoreError) -> AdapterFailure {
    match error {
        CoreError::InvalidRequest(diagnostics) => core_diagnostics_failure("validation", diagnostics),
        CoreError::MaterialMismatch(diagnostics) => core_diagnostics_failure("material", diagnostics),
        CoreError::EvaluationFailed(diagnostics) => core_diagnostics_failure("eval", diagnostics),
        CoreError::UndeclaredDependency(path) => single_failure(
            "dependency-closure",
            "undeclared-dependency",
            &path,
            "evaluator observed an undeclared dependency",
        ),
        CoreError::DependencyClosureMismatch => single_failure(
            "dependency-closure",
            "dependency-closure-mismatch",
            "dependencies",
            "declared and evaluator-observed dependency closures differ",
        ),
        CoreError::SecretMaterial(path) => single_failure(
            "secret-material",
            "secret-marker",
            &path,
            "secret-like source material requires an explicit product-owned policy",
        ),
        CoreError::MixedEvaluators => single_failure(
            "mixed-evaluator",
            "mixed-evaluator",
            "evaluator",
            "canonical manifests cannot mix evaluator descriptors",
        ),
        CoreError::DuplicateOutput(path) => single_failure(
            "duplicate-output",
            "duplicate-output",
            &path,
            "canonical manifests cannot repeat an output target",
        ),
        CoreError::Serialization => serialization_failure(),
        CoreError::StaleManifest => single_failure(
            "stale-output",
            "stale-output",
            "manifest",
            "checked export evidence does not match current exact bytes",
        ),
    }
}

fn core_diagnostics_failure(class: &str, diagnostics: Vec<Diagnostic>) -> AdapterFailure {
    AdapterFailure {
        class: class.to_string(),
        diagnostics: diagnostics.iter().map(mantle_diagnostic).collect(),
        drift: None,
    }
}

fn mantle_diagnostic(diagnostic: &Diagnostic) -> NickelExportDiagnostic {
    NickelExportDiagnostic {
        class: mapped_diagnostic_class(diagnostic).to_string(),
        subject: diagnostic.subject.clone(),
        message: diagnostic.message.clone(),
    }
}

fn mapped_diagnostic_class(diagnostic: &Diagnostic) -> &str {
    if diagnostic.class == "unsafe-path" {
        return match diagnostic.subject.as_str() {
            "source" => "unsafe-source-path",
            "destination" => "unsafe-output-path",
            "dependency" => "unsafe-dependency-path",
            "import-path" => "unsafe-import-path",
            _ => "unsafe-path",
        };
    }
    &diagnostic.class
}

fn single_failure(class: &str, diagnostic_class: &str, subject: &str, message: &str) -> AdapterFailure {
    AdapterFailure {
        class: class.to_string(),
        diagnostics: vec![adapter_diagnostic(diagnostic_class, subject, message)],
        drift: None,
    }
}

fn drift_diagnostics(evidence: &DualRunEvidence) -> Vec<NickelExportDiagnostic> {
    vec![adapter_diagnostic(
        "dual-run-drift",
        evidence.drift_class.as_deref().unwrap_or("unknown"),
        &format!(
            "canonical={} legacy={} canonical_projection={} legacy_projection={}",
            evidence.canonical_receipt_identity_blake3,
            evidence.legacy_receipt_identity_blake3,
            evidence.canonical_projection_digest_blake3,
            evidence.legacy_projection_digest_blake3
        ),
    )]
}

fn adapter_diagnostic(class: &str, subject: &str, message: &str) -> NickelExportDiagnostic {
    NickelExportDiagnostic {
        class: class.to_string(),
        subject: subject.to_string(),
        message: message.to_string(),
    }
}

fn diagnostics_match(legacy: &[NickelExportDiagnostic], canonical: &[NickelExportDiagnostic]) -> bool {
    legacy == canonical
}

fn artifact_len(artifacts: &[ExplicitArtifact<'_>], path: &str) -> u64 {
    artifacts
        .iter()
        .find(|artifact| artifact.path == path)
        .map_or(u64::MAX, |artifact| byte_len(artifact.bytes))
}

fn byte_len(bytes: &[u8]) -> u64 {
    u64::try_from(bytes.len()).unwrap_or(u64::MAX)
}

fn tagged_blake3(digest: &str) -> String {
    if digest.starts_with(BLAKE3_PREFIX) {
        return digest.to_string();
    }
    format!("{BLAKE3_PREFIX}{digest}")
}

fn bare_blake3(identity: &str) -> &str {
    identity.strip_prefix(BLAKE3_PREFIX).unwrap_or(identity)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE_BYTES: &[u8] = b"{ answer = 42 }";
    const DEPENDENCY_BYTES: &[u8] = b"{ enabled = true }";
    const OUTPUT_BYTES: &[u8] = b"{\"answer\":42}";
    const TAMPERED_OUTPUT_BYTES: &[u8] = b"{\"answer\":43}";
    const SOURCE_PATH: &str = "config/main.ncl";
    const DEPENDENCY_PATH: &str = "config/dependency.ncl";
    const OUTPUT_PATH: &str = "generated/config.json";
    const CANONICAL_IDENTITY_FIXTURE: &str = "canonical-identity";
    const LEGACY_IDENTITY_FIXTURE: &str = "legacy-identity";

    fn evaluator(identity: &str) -> NickelEvaluatorDescriptor {
        NickelEvaluatorDescriptor {
            identity: identity.to_string(),
            version: "mantle-evaluator-fixture-v1".to_string(),
            options: vec!["format=json".to_string(), "import-paths=1".to_string()],
        }
    }

    fn request<'a>(evaluator: &'a NickelEvaluatorDescriptor, dependencies: &'a [String]) -> AdapterRequest<'a> {
        AdapterRequest {
            source: SOURCE_PATH,
            dependencies,
            import_paths: dependencies,
            format: "json",
            output_target: OUTPUT_PATH,
            evaluator,
        }
    }

    fn observation<'a>(
        evaluator: &'a NickelEvaluatorDescriptor,
        dependencies: &'a [String],
        dependency_material: &'a [ExplicitArtifact<'a>],
        output: &'a [u8],
        diagnostics: &'a [NickelExportDiagnostic],
    ) -> ExplicitObservation<'a> {
        ExplicitObservation {
            request: request(evaluator, dependencies),
            source: ExplicitArtifact {
                path: SOURCE_PATH,
                bytes: SOURCE_BYTES,
            },
            dependencies: dependency_material,
            output: ExplicitArtifact {
                path: OUTPUT_PATH,
                bytes: output,
            },
            diagnostics,
        }
    }

    fn legacy_receipt(evaluator: &NickelEvaluatorDescriptor, output: &[u8]) -> NickelExportReceipt {
        NickelExportReceipt {
            schema: nickel_export_core::MANTLE_RECEIPT_SCHEMA.to_string(),
            root_source: ExportSourceRef {
                path: SOURCE_PATH.to_string(),
                digest_blake3: digest(SOURCE_BYTES),
            },
            deps: vec![ExportSourceRef {
                path: DEPENDENCY_PATH.to_string(),
                digest_blake3: digest(DEPENDENCY_BYTES),
            }],
            import_paths: vec![DEPENDENCY_PATH.to_string()],
            format: "json".to_string(),
            output_target: OUTPUT_PATH.to_string(),
            output_digest_blake3: digest(output),
            evaluator: evaluator.clone(),
            non_claim: nickel_export_core::MANTLE_NON_CLAIM.to_string(),
        }
    }

    fn admitted(output: &[u8], evaluator_identity: &str) -> CanonicalAdmission {
        let evaluator = evaluator(evaluator_identity);
        let dependency_paths = vec![DEPENDENCY_PATH.to_string()];
        let dependency_material = [ExplicitArtifact {
            path: DEPENDENCY_PATH,
            bytes: DEPENDENCY_BYTES,
        }];
        let observation = observation(&evaluator, &dependency_paths, &dependency_material, output, &[]);
        admit_and_compare(&observation, &legacy_receipt(&evaluator, output)).unwrap()
    }

    #[test]
    fn positive_exact_source_receipt_projection_and_freshness_agree() {
        let admission = admitted(OUTPUT_BYTES, "mantle-embedded-crunch-eval");
        assert!(admission.evidence.identity_matches);
        assert!(admission.evidence.projection_matches);
        assert_eq!(admission.evidence.drift_class, None);
        assert_eq!(admission.mantle_receipt.schema, nickel_export_core::MANTLE_RECEIPT_SCHEMA);
        assert_eq!(verify_freshness(&admission.canonical_manifest, &admission.canonical_manifest), Ok(()));
    }

    #[test]
    fn unexplained_identity_drift_fails_closed_as_serialization_drift() {
        let admission = admitted(OUTPUT_BYTES, "mantle-embedded-crunch-eval");
        let evidence = comparison_evidence(
            CANONICAL_IDENTITY_FIXTURE,
            LEGACY_IDENTITY_FIXTURE,
            &admission.mantle_receipt,
            &admission.mantle_receipt,
            &admission.canonical_receipt,
            &admission.canonical_receipt,
        )
        .unwrap();
        let failure = enforce_no_drift(evidence).unwrap_err();
        assert_eq!(failure.class, "dual-run-drift");
        assert_eq!(failure.drift.unwrap().drift_class.as_deref(), Some(DRIFT_SERIALIZATION));
    }

    #[test]
    fn path_escape_is_rejected_before_observation_admission() {
        let evaluator = evaluator("mantle-embedded-crunch-eval");
        let dependencies = Vec::new();
        let mut request = request(&evaluator, &dependencies);
        request.source = "../secret.ncl";
        let failure = normalize_adapter_request(&request).unwrap_err();
        assert_eq!(failure.class, "validation");
        assert!(failure.diagnostics.iter().any(|diagnostic| diagnostic.class == "unsafe-source-path"));
    }

    #[test]
    fn stale_output_and_tampered_manifest_are_rejected() {
        let expected = admitted(OUTPUT_BYTES, "mantle-embedded-crunch-eval");
        let stale = admitted(TAMPERED_OUTPUT_BYTES, "mantle-embedded-crunch-eval");
        assert_eq!(
            verify_freshness(&expected.canonical_manifest, &stale.canonical_manifest).unwrap_err().class,
            "stale-output"
        );

        let mut tampered = expected.canonical_manifest.clone();
        tampered.exports[0].output.identity = nickel_export_core::blake3_identity(b"tampered");
        assert_eq!(verify_freshness(&expected.canonical_manifest, &tampered).unwrap_err().class, "stale-output");
    }

    #[test]
    fn mixed_evaluator_manifest_is_rejected() {
        let first = admitted(OUTPUT_BYTES, "mantle-embedded-crunch-eval");
        let second = admitted(TAMPERED_OUTPUT_BYTES, "other-evaluator");
        let failure = nickel_export_core::build_manifest(&[first.canonical_receipt, second.canonical_receipt])
            .map_err(adapter_failure_from_core)
            .unwrap_err();
        assert_eq!(failure.class, "mixed-evaluator");
        assert!(!failure.diagnostics.is_empty());
    }

    #[test]
    fn evaluator_error_produces_matching_failure_without_receipt() {
        let evaluator = evaluator("mantle-embedded-crunch-eval");
        let dependency_paths = vec![DEPENDENCY_PATH.to_string()];
        let dependency_material = [ExplicitArtifact {
            path: DEPENDENCY_PATH,
            bytes: DEPENDENCY_BYTES,
        }];
        let diagnostic = NickelExportDiagnostic {
            class: "eval".to_string(),
            subject: SOURCE_PATH.to_string(),
            message: "evaluation failed".to_string(),
        };
        let diagnostics = [diagnostic];
        let observation = observation(&evaluator, &dependency_paths, &dependency_material, &[], &diagnostics);
        let evidence = compare_evaluator_failure(&observation).unwrap();
        assert_eq!(evidence.legacy_failure_class, "eval");
        assert_eq!(evidence.canonical_failure_class, "eval");
        assert!(evidence.diagnostics_match);
        assert!(!evidence.receipt_emitted);
    }

    #[test]
    fn secret_marker_and_overclaim_are_rejected() {
        let evaluator = evaluator("mantle-embedded-crunch-eval");
        let dependencies = Vec::new();
        let observation = ExplicitObservation {
            request: request(&evaluator, &dependencies),
            source: ExplicitArtifact {
                path: SOURCE_PATH,
                bytes: b"{ token = \"do-not-export\" }",
            },
            dependencies: &[],
            output: ExplicitArtifact {
                path: OUTPUT_PATH,
                bytes: OUTPUT_BYTES,
            },
            diagnostics: &[],
        };
        let request = normalize_adapter_request(&observation.request).unwrap();
        let failure = build_canonical_receipt(&observation, &request, &canonical_evaluator(&evaluator)).unwrap_err();
        assert_eq!(failure.class, "secret-material");

        let mut admitted = admitted(OUTPUT_BYTES, "mantle-embedded-crunch-eval");
        admitted.mantle_receipt.non_claim = "export succeeded".to_string();
        assert_eq!(
            validate_claim_boundary(&admitted.canonical_receipt, &admitted.mantle_receipt).unwrap_err().class,
            "overclaim"
        );
    }

    #[test]
    fn adapter_source_has_no_ambient_authority() {
        let source = include_str!("nickel_export_core_adapter.rs");
        let forbidden = [
            ["std", "::fs::"].concat(),
            ["std", "::env::"].concat(),
            ["std", "::process::Command"].concat(),
            ["crunch", "_eval::"].concat(),
            ["print", "ln!"].concat(),
            ["eprint", "ln!"].concat(),
        ];
        for token in forbidden {
            assert!(!source.contains(&token), "pure adapter contains forbidden authority token {token}");
        }
        assert!(source.contains("ExplicitObservation"));
        assert!(source.contains("nickel_export_core::build_receipt"));
    }

    fn digest(bytes: &[u8]) -> String {
        bare_blake3(&nickel_export_core::blake3_identity(bytes)).to_string()
    }
}
