// machine-artifact-public: build.build-json-report
use std::path::Path;

use crunch_pipeline::BuildConfig;
use crunch_pipeline::PipelineResult;
use crunch_pipeline::drv_key_for;
use crunch_pipeline::label_for_key;
use crunch_release_core::AstGrepStructuralEvidence;
use nix_compat::store_path::StorePath;
use serde::Deserialize;
use serde::Serialize;
use snix_store::path_info::PathInfo;

use crate::ast_grep_evidence::AST_GREP_EVIDENCE_RELATIVE_PATH;
use crate::ast_grep_evidence::AstGrepEvidenceRead;
use crate::ast_grep_evidence::read_ast_grep_evidence;
use crate::build_failure::BuildFailureEnvelope;
use crate::build_failure::build_failure_envelopes;
use crate::build_log::DiagnosticPersistenceFailure;
use crate::build_log::existing_log_file_path;
use crate::frontend_artifact_spec::FrontendArtifactAdmissionAttestation;

const OFFLINE_CARGO_EVIDENCE_NEXT_ACTION: &str =
    "inspect share/mantle/offline-cargo-build.json and rebuild with mantle.offlineCargoPackage if the sidecar is stale";
const AST_GREP_EVIDENCE_NEXT_ACTION: &str = "regenerate share/mantle/ast-grep-structural-evidence.json with the pinned ast-grep toolchain and current BLAKE3 identities";

#[derive(Debug, Serialize)]
pub struct BuildJsonReport {
    pub schema: &'static str,
    pub file: String,
    pub output_dir: String,
    pub state_dir: String,
    pub store_dir: String,
    pub scheduler_policy: crunch_pipeline::SchedulingPolicy,
    pub hermeticity_mode: String,
    pub hermeticity_audit_events: Vec<BuildJsonHermeticityAuditEvent>,
    pub build_environment_reports: Vec<BuildJsonEnvironmentReport>,
    pub network_policy_reports: Vec<BuildJsonNetworkPolicyReport>,
    pub workspace_reports: Vec<crunch_build::WorkspaceExecutionReport>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub action_result_reports: Vec<crunch_build::ActionResultRuntimeReport>,
    pub native_dynamic_plans: Vec<BuildJsonNativeDynamicPlan>,
    pub scheduler_priority_decisions: Vec<crunch_pipeline::PriorityDecisionEvidence>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub remote_telemetry_events: Vec<crunch_build::distributed::RemoteTelemetryEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_observability: Option<Vec<BuildJsonRemoteObservability>>,
    pub frontend_artifact_attestations: Vec<FrontendArtifactAdmissionAttestation>,
    pub ast_grep_structural_evidence: Vec<BuildJsonAstGrepStructuralEvidence>,
    pub ast_grep_structural_evidence_diagnostics: Vec<BuildJsonAstGrepStructuralEvidenceDiagnostic>,
    pub cargo_build_evidence: Vec<BuildJsonCargoBuildEvidence>,
    pub cargo_build_evidence_diagnostics: Vec<BuildJsonCargoBuildEvidenceDiagnostic>,
    pub diagnostic_persistence_failures: Vec<DiagnosticPersistenceFailure>,
    pub counts: BuildJsonCounts,
    pub outcomes: Vec<BuildJsonOutcome>,
    pub failed: Vec<BuildFailureEnvelope>,
    pub fod_mismatches: Vec<BuildJsonFodMismatch>,
}

#[derive(Debug, Serialize)]
pub struct BuildJsonRemoteObservability {
    pub health: crate::remote_build::RemoteAttemptObservabilityHealth,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub immutable_log: Option<crate::remote_build::RemoteAttemptLogControlSummary>,
}

#[derive(Debug, Serialize)]
pub struct BuildJsonCounts {
    pub succeeded_total: u32,
    pub built_total: u32,
    pub cached_total: u32,
    pub failed_total: u32,
}

#[derive(Debug, Serialize)]
pub struct BuildJsonHermeticityAuditEvent {
    pub kind: String,
    pub detail: String,
}

#[derive(Debug, Serialize)]
pub struct BuildJsonEnvironmentReport {
    pub action_name: String,
    pub digest_algorithm: String,
    pub digest_blake3: Option<String>,
    pub variable_count: u32,
    pub rejections: Vec<BuildJsonEnvironmentRejection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_path: Option<BuildJsonSearchPathReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub determinism: Option<BuildJsonDeterminismNormalizationReport>,
}

#[derive(Debug, Serialize)]
pub struct BuildJsonEnvironmentRejection {
    pub variable: String,
    pub class: String,
    pub diagnostic: String,
    pub redacted: bool,
}

#[derive(Debug, Serialize)]
pub struct BuildJsonSearchPathReport {
    pub digest_algorithm: String,
    pub digest_blake3: Option<String>,
    pub entries: Vec<BuildJsonSearchPathEntry>,
    pub aliases: Vec<BuildJsonSearchPathAlias>,
    pub real_tool_refs: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct BuildJsonSearchPathEntry {
    pub path: String,
    pub kind: String,
    pub real_tool_ref: String,
}

#[derive(Debug, Serialize)]
pub struct BuildJsonSearchPathAlias {
    pub alias_path: String,
    pub real_tool_ref: String,
}

#[derive(Debug, Serialize)]
pub struct BuildJsonDeterminismNormalizationReport {
    pub digest_algorithm: String,
    pub policy_digest_blake3: String,
    pub controls: Vec<BuildJsonDeterminismControl>,
    pub unsupported_controls: Vec<String>,
    pub divergence: Option<BuildJsonOutputDivergenceDiagnostic>,
    pub strong_claim_blocked: bool,
}

#[derive(Debug, Serialize)]
pub struct BuildJsonDeterminismControl {
    pub surface: String,
    pub policy: String,
    pub value: String,
    pub enforcement: String,
}

#[derive(Debug, Serialize)]
pub struct BuildJsonOutputDivergenceDiagnostic {
    pub status: String,
    pub surface: String,
    pub left_digest_blake3: String,
    pub right_digest_blake3: String,
    pub diagnostic: String,
}

#[derive(Debug, Serialize)]
pub struct BuildJsonNetworkPolicyReport {
    pub action_name: String,
    pub mode: String,
    pub result: String,
    pub capability: Option<String>,
    pub policy_basis: Option<String>,
    pub audit_class: Option<String>,
    pub fixed_output: Option<BuildJsonFixedOutputNetworkDeclaration>,
    pub diagnostic: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BuildJsonFixedOutputNetworkDeclaration {
    pub url: Option<String>,
    pub hash: Option<String>,
    pub mode: Option<String>,
    pub retry_policy: String,
}

#[derive(Debug, Serialize)]
pub struct BuildJsonNativeDynamicPlan {
    pub mode: String,
    pub producer_key: String,
    pub output_name: String,
    pub plan_artifact_path: Option<String>,
    pub raw_artifact_digest: Option<String>,
    pub canonical_plan_digest: Option<String>,
    pub accepted_unit_ids: Vec<String>,
    pub rejection_reason: Option<String>,
    pub scheduler_action: String,
}

#[derive(Debug, Serialize)]
pub struct BuildJsonOutcome {
    pub drv_key: String,
    pub label: String,
    pub cached: bool,
    pub log_file: Option<String>,
    pub outputs: Vec<BuildJsonOutput>,
}

#[derive(Debug, Serialize)]
pub struct BuildJsonOutput {
    pub name: String,
    pub path: String,
    pub artifact_attestation: BuildJsonAttestationReference,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub substitution: Option<BuildJsonSubstitution>,
    /// Cache admission details for this output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_admission: Option<BuildJsonCacheAdmission>,
}

/// Cache admission details for a single output.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct BuildJsonCacheAdmission {
    /// Stable kebab-case reason code.
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct BuildJsonSubstitution {
    pub mode: String,
    pub transferred_bytes: u64,
    pub reused_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BuildJsonAttestationReference {
    pub logical_path: String,
    pub path: String,
}

#[derive(Debug, Serialize)]
pub struct BuildJsonFodMismatch {
    pub name: String,
    pub expected_sri: String,
    pub actual_sri: String,
}

#[derive(Debug, Serialize)]
pub struct BuildJsonAstGrepStructuralEvidence {
    pub label: String,
    pub output_name: String,
    pub evidence_path: String,
    pub sidecar_file_digest_blake3: String,
    pub sidecar_canonical_digest_blake3: String,
    pub evidence: AstGrepStructuralEvidence,
}

#[derive(Debug, Serialize)]
pub struct BuildJsonAstGrepStructuralEvidenceDiagnostic {
    pub label: String,
    pub output_name: String,
    pub evidence_path: String,
    pub blocker_class: String,
    pub message: String,
    pub next_action: &'static str,
}

#[derive(Debug, Serialize)]
pub struct BuildJsonCargoBuildEvidence {
    pub label: String,
    pub output_name: String,
    pub claim_class: String,
    pub project_build_status: String,
    pub evidence_path: String,
    pub evidence_schema: String,
    pub evidence_version: u32,
    pub evidence_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lockfile: Option<BuildJsonCargoSourceClosureEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cargo_command: Option<BuildJsonCargoCommandEvidence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_policy: Option<BuildJsonCargoNetworkPolicyEvidence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<BuildJsonCargoOutputEvidence>,
    pub source_closure: Vec<BuildJsonCargoSourceClosureEntry>,
    pub toolchain: BuildJsonCargoToolchainEvidence,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct BuildJsonCargoBuildEvidenceDiagnostic {
    pub label: String,
    pub output_name: String,
    pub evidence_path: String,
    pub blocker_class: String,
    pub message: String,
    pub next_action: &'static str,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BuildJsonCargoSourceClosureEntry {
    pub role: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub digest_blake3: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_digest_blake3: Option<String>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub identity_class: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BuildJsonCargoToolchainEvidence {
    pub cargo: String,
    pub rustc: String,
    pub linker: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct BuildJsonCargoCommandEvidence {
    pub program: String,
    pub args: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct BuildJsonCargoNetworkPolicyEvidence {
    pub mode: String,
    pub result: String,
    pub allow_undeclared_network: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct BuildJsonCargoOutputEvidence {
    pub binary: String,
    pub path: String,
}

pub fn render_build_json_report(
    config: &BuildConfig,
    result: &PipelineResult,
    logs_dir: &Path,
    diagnostic_persistence_failures: &[DiagnosticPersistenceFailure],
) -> Result<String, serde_json::Error> {
    render_build_json_report_with_frontend_artifact_attestations(
        config,
        result,
        logs_dir,
        diagnostic_persistence_failures,
        &[],
    )
}

pub fn render_build_json_report_with_frontend_artifact_attestations(
    config: &BuildConfig,
    result: &PipelineResult,
    logs_dir: &Path,
    diagnostic_persistence_failures: &[DiagnosticPersistenceFailure],
    frontend_artifact_attestations: &[FrontendArtifactAdmissionAttestation],
) -> Result<String, serde_json::Error> {
    let build_document =
        build_json_report(config, result, logs_dir, diagnostic_persistence_failures, frontend_artifact_attestations);
    serde_json::to_string_pretty(&build_document)
}

fn build_json_report(
    config: &BuildConfig,
    result: &PipelineResult,
    logs_dir: &Path,
    diagnostic_persistence_failures: &[DiagnosticPersistenceFailure],
    frontend_artifact_attestations: &[FrontendArtifactAdmissionAttestation],
) -> BuildJsonReport {
    debug_assert_eq!(config.hermeticity_mode, result.hermeticity_mode, "config/result hermeticity modes must match");
    let outcome_rows = build_outcome_reports(config, result, logs_dir);
    let (ast_grep_structural_evidence, ast_grep_structural_evidence_diagnostics) =
        build_ast_grep_structural_evidence_reports(&outcome_rows);
    let (cargo_build_evidence, cargo_build_evidence_diagnostics) = build_cargo_build_evidence_reports(&outcome_rows);
    let failure_rows = build_failure_envelopes(result, &config.store_dir, logs_dir);
    let counts = build_counts(&outcome_rows, &failure_rows);
    let hermeticity_audit_events = result
        .hermeticity_audit_events
        .iter()
        .map(|event| BuildJsonHermeticityAuditEvent {
            kind: event.kind.as_str().to_string(),
            detail: event.detail.clone(),
        })
        .collect();
    let fod_mismatches = result
        .fod_mismatches
        .iter()
        .map(|mismatch| BuildJsonFodMismatch {
            name: mismatch.name.clone(),
            expected_sri: mismatch.expected_sri.clone(),
            actual_sri: mismatch.actual_sri.clone(),
        })
        .collect();
    let build_environment_rows = build_environment_reports(result);
    let network_policy_rows = build_network_policy_reports(result);
    let workspace_rows = result.workspace_reports.clone();
    let action_result_rows = result.action_result_reports.clone();
    let native_dynamic_plans = build_native_dynamic_plan_reports(result, &config.store_dir);
    let scheduler_priority_decisions = result.priority_decisions.clone();
    let remote_telemetry_events = result
        .priority_decisions
        .iter()
        .filter_map(|decision| crunch_build::distributed::telemetry_for_priority_dispatch(decision).ok())
        .collect();

    BuildJsonReport {
        schema: "crunch-build-report-v1",
        file: config.file.display().to_string(),
        output_dir: config.output_dir.display().to_string(),
        state_dir: config.state_dir.display().to_string(),
        store_dir: config.store_dir.clone(),
        scheduler_policy: config.scheduling_policy.clone(),
        hermeticity_mode: result.hermeticity_mode.as_str().to_string(),
        hermeticity_audit_events,
        build_environment_reports: build_environment_rows,
        network_policy_reports: network_policy_rows,
        workspace_reports: workspace_rows,
        action_result_reports: action_result_rows,
        native_dynamic_plans,
        scheduler_priority_decisions,
        remote_telemetry_events,
        remote_observability: None,
        frontend_artifact_attestations: frontend_artifact_attestations.to_vec(),
        ast_grep_structural_evidence,
        ast_grep_structural_evidence_diagnostics,
        cargo_build_evidence,
        cargo_build_evidence_diagnostics,
        diagnostic_persistence_failures: diagnostic_persistence_failures.to_vec(),
        counts,
        outcomes: outcome_rows,
        failed: failure_rows,
        fod_mismatches,
    }
}

fn build_environment_reports(result: &PipelineResult) -> Vec<BuildJsonEnvironmentReport> {
    result
        .build_environment_reports
        .iter()
        .map(|row| BuildJsonEnvironmentReport {
            action_name: row.action_name.clone(),
            digest_algorithm: crunch_pipeline::BUILD_ENVIRONMENT_DIGEST_ALGORITHM.to_string(),
            digest_blake3: row.digest_blake3.clone(),
            variable_count: row.variable_count,
            rejections: row
                .rejections
                .iter()
                .map(|rejection| BuildJsonEnvironmentRejection {
                    variable: rejection.variable.clone(),
                    class: rejection.class.clone(),
                    diagnostic: rejection.diagnostic.clone(),
                    redacted: rejection.redacted,
                })
                .collect(),
            search_path: row.search_path.as_ref().map(|search_path| BuildJsonSearchPathReport {
                digest_algorithm: crunch_pipeline::SEARCH_PATH_DIGEST_ALGORITHM.to_string(),
                digest_blake3: search_path.digest_blake3.clone(),
                entries: search_path
                    .entries
                    .iter()
                    .map(|entry| BuildJsonSearchPathEntry {
                        path: entry.path.clone(),
                        kind: entry.kind.clone(),
                        real_tool_ref: entry.real_tool_ref.clone(),
                    })
                    .collect(),
                aliases: search_path
                    .aliases
                    .iter()
                    .map(|alias| BuildJsonSearchPathAlias {
                        alias_path: alias.alias_path.clone(),
                        real_tool_ref: alias.real_tool_ref.clone(),
                    })
                    .collect(),
                real_tool_refs: search_path.real_tool_refs.clone(),
            }),
            determinism: row.determinism.as_ref().map(|determinism| BuildJsonDeterminismNormalizationReport {
                digest_algorithm: crunch_pipeline::DETERMINISM_NORMALIZATION_DIGEST_ALGORITHM.to_string(),
                policy_digest_blake3: determinism.policy_digest_blake3.clone(),
                controls: determinism
                    .controls
                    .iter()
                    .map(|control| BuildJsonDeterminismControl {
                        surface: control.surface.clone(),
                        policy: control.policy.clone(),
                        value: control.value.clone(),
                        enforcement: control.enforcement.clone(),
                    })
                    .collect(),
                unsupported_controls: determinism.unsupported_controls.clone(),
                divergence: determinism.divergence.as_ref().map(|divergence| BuildJsonOutputDivergenceDiagnostic {
                    status: divergence.status.clone(),
                    surface: divergence.surface.clone(),
                    left_digest_blake3: divergence.left_digest_blake3.clone(),
                    right_digest_blake3: divergence.right_digest_blake3.clone(),
                    diagnostic: divergence.diagnostic.clone(),
                }),
                strong_claim_blocked: determinism.strong_claim_blocked,
            }),
        })
        .collect()
}

fn build_network_policy_reports(result: &PipelineResult) -> Vec<BuildJsonNetworkPolicyReport> {
    result
        .network_policy_reports
        .iter()
        .map(|row| BuildJsonNetworkPolicyReport {
            action_name: row.action_name.clone(),
            mode: row.mode.clone(),
            result: row.result.clone(),
            capability: row.capability.clone(),
            policy_basis: row.policy_basis.clone(),
            audit_class: row.audit_class.clone(),
            fixed_output: row.fixed_output.as_ref().map(|fixed| BuildJsonFixedOutputNetworkDeclaration {
                url: fixed.url.clone(),
                hash: fixed.hash.clone(),
                mode: fixed.mode.clone(),
                retry_policy: fixed.retry_policy.clone(),
            }),
            diagnostic: row.diagnostic.clone(),
        })
        .collect()
}

fn build_native_dynamic_plan_reports(result: &PipelineResult, store_dir: &str) -> Vec<BuildJsonNativeDynamicPlan> {
    result
        .native_dynamic_plans
        .iter()
        .map(|row| BuildJsonNativeDynamicPlan {
            mode: row.mode.clone(),
            producer_key: row.producer_key.clone(),
            output_name: row.output_name.clone(),
            plan_artifact_path: row
                .plan_artifact_path
                .as_ref()
                .map(|path: &StorePath<String>| path.to_absolute_path_with_prefix(store_dir)),
            raw_artifact_digest: row.raw_artifact_digest.clone(),
            canonical_plan_digest: row.canonical_plan_digest.clone(),
            accepted_unit_ids: row.accepted_unit_ids.clone(),
            rejection_reason: row.rejection_reason.clone(),
            scheduler_action: row.scheduler_action.clone(),
        })
        .collect()
}

fn build_counts(outcomes: &[BuildJsonOutcome], failed: &[BuildFailureEnvelope]) -> BuildJsonCounts {
    let succeeded_total = count_as_u32(outcomes.len());
    let failed_total = count_as_u32(failed.len());
    let cached_total = count_as_u32(outcomes.iter().filter(|outcome| outcome.cached).count());
    let built_total = succeeded_total.saturating_sub(cached_total);
    BuildJsonCounts {
        succeeded_total,
        built_total,
        cached_total,
        failed_total,
    }
}

fn build_outcome_reports(config: &BuildConfig, result: &PipelineResult, logs_dir: &Path) -> Vec<BuildJsonOutcome> {
    let output_dir_str = config.output_dir.to_str().unwrap_or(&config.store_dir);
    let mut outcome_rows: Vec<BuildJsonOutcome> = result
        .outcomes
        .iter()
        .map(|outcome| {
            let drv_key = drv_key_for(&config.store_dir, &outcome.drv_path);
            let label = label_for_key(result, &drv_key).unwrap_or(outcome.drv_path.name()).to_string();
            let mut outputs: Vec<BuildJsonOutput> = outcome
                .outputs
                .iter()
                .map(|(name, path_info): (&String, &PathInfo)| {
                    let logical_path = path_info.store_path.to_absolute_path_with_prefix(&config.store_dir);
                    let attestation_path = crunch_store::artifact_attestation_file_path(
                        &config.state_dir,
                        &config.store_dir,
                        &path_info.store_path,
                    );
                    BuildJsonOutput {
                        name: name.clone(),
                        path: path_info.store_path.to_absolute_path_with_prefix(output_dir_str),
                        artifact_attestation: BuildJsonAttestationReference {
                            logical_path,
                            path: attestation_path.display().to_string(),
                        },
                        substitution: outcome.substitutions.get(name).map(|report| BuildJsonSubstitution {
                            mode: report.mode.as_str().to_string(),
                            transferred_bytes: report.transferred_bytes,
                            reused_bytes: report.reused_bytes,
                            fallback_reason: report.fallback_reason.clone(),
                        }),
                        cache_admission: None,
                    }
                })
                .collect();
            outputs.sort_by(|left, right| left.name.cmp(&right.name));
            let log_file = success_log_file(logs_dir, outcome);
            BuildJsonOutcome {
                drv_key,
                label,
                cached: outcome.cached,
                log_file,
                outputs,
            }
        })
        .collect();
    outcome_rows.sort_by(|left, right| left.label.cmp(&right.label).then(left.drv_key.cmp(&right.drv_key)));
    debug_assert_eq!(outcome_rows.len(), result.outcomes.len());
    debug_assert!(outcome_rows.capacity() >= outcome_rows.len());
    outcome_rows
}

fn build_ast_grep_structural_evidence_reports(
    outcomes: &[BuildJsonOutcome],
) -> (Vec<BuildJsonAstGrepStructuralEvidence>, Vec<BuildJsonAstGrepStructuralEvidenceDiagnostic>) {
    let output_count_max = outcomes.iter().map(|outcome| outcome.outputs.len()).fold(0_usize, usize::saturating_add);
    let mut evidence_rows = Vec::with_capacity(output_count_max);
    let mut diagnostics = Vec::with_capacity(output_count_max);
    for outcome in outcomes {
        for output in &outcome.outputs {
            match ast_grep_structural_evidence_for_output(outcome, output) {
                AstGrepStructuralEvidenceOutcome::Absent => {}
                AstGrepStructuralEvidenceOutcome::Valid(report) => evidence_rows.push(report),
                AstGrepStructuralEvidenceOutcome::Invalid(diagnostic) => diagnostics.push(diagnostic),
            }
        }
    }
    evidence_rows.sort_by(|left, right| left.label.cmp(&right.label).then(left.output_name.cmp(&right.output_name)));
    diagnostics.sort_by(|left, right| left.label.cmp(&right.label).then(left.output_name.cmp(&right.output_name)));
    debug_assert!(evidence_rows.len().saturating_add(diagnostics.len()) <= output_count_max);
    debug_assert!(evidence_rows.capacity() >= evidence_rows.len());
    debug_assert!(diagnostics.capacity() >= diagnostics.len());
    (evidence_rows, diagnostics)
}

fn ast_grep_structural_evidence_for_output(
    outcome: &BuildJsonOutcome,
    output: &BuildJsonOutput,
) -> AstGrepStructuralEvidenceOutcome {
    let evidence_path = Path::new(&output.path).join(AST_GREP_EVIDENCE_RELATIVE_PATH);
    debug_assert!(!outcome.label.is_empty());
    debug_assert!(!output.name.is_empty());
    match read_ast_grep_evidence(&evidence_path) {
        AstGrepEvidenceRead::Missing => AstGrepStructuralEvidenceOutcome::Absent,
        AstGrepEvidenceRead::Valid(loaded) => {
            AstGrepStructuralEvidenceOutcome::Valid(BuildJsonAstGrepStructuralEvidence {
                label: outcome.label.clone(),
                output_name: output.name.clone(),
                evidence_path: evidence_path.display().to_string(),
                sidecar_file_digest_blake3: loaded.sidecar_file_digest_blake3,
                sidecar_canonical_digest_blake3: loaded.sidecar_canonical_digest_blake3,
                evidence: loaded.evidence,
            })
        }
        AstGrepEvidenceRead::Invalid { blocker_class, message } => {
            AstGrepStructuralEvidenceOutcome::Invalid(BuildJsonAstGrepStructuralEvidenceDiagnostic {
                label: outcome.label.clone(),
                output_name: output.name.clone(),
                evidence_path: evidence_path.display().to_string(),
                blocker_class: blocker_class.to_string(),
                message,
                next_action: AST_GREP_EVIDENCE_NEXT_ACTION,
            })
        }
    }
}

fn build_cargo_build_evidence_reports(
    outcomes: &[BuildJsonOutcome],
) -> (Vec<BuildJsonCargoBuildEvidence>, Vec<BuildJsonCargoBuildEvidenceDiagnostic>) {
    let output_count_max = outcomes.iter().map(|outcome| outcome.outputs.len()).fold(0_usize, usize::saturating_add);
    let mut evidence_rows = Vec::with_capacity(output_count_max);
    let mut diagnostics = Vec::with_capacity(output_count_max);
    for outcome in outcomes {
        for output in &outcome.outputs {
            match cargo_build_evidence_for_output(outcome, output) {
                CargoBuildEvidenceOutcome::Absent => {}
                CargoBuildEvidenceOutcome::Valid(report) => evidence_rows.push(report),
                CargoBuildEvidenceOutcome::Invalid(diagnostic) => diagnostics.push(diagnostic),
            }
        }
    }
    evidence_rows.sort_by(|left, right| left.label.cmp(&right.label).then(left.output_name.cmp(&right.output_name)));
    diagnostics.sort_by(|left, right| left.label.cmp(&right.label).then(left.output_name.cmp(&right.output_name)));
    debug_assert!(evidence_rows.len().saturating_add(diagnostics.len()) <= output_count_max);
    debug_assert!(evidence_rows.capacity() >= evidence_rows.len());
    debug_assert!(diagnostics.capacity() >= diagnostics.len());
    (evidence_rows, diagnostics)
}

fn cargo_build_evidence_for_output(outcome: &BuildJsonOutcome, output: &BuildJsonOutput) -> CargoBuildEvidenceOutcome {
    let evidence_path = Path::new(&output.path).join(crate::offline_cargo::OFFLINE_CARGO_EVIDENCE_RELATIVE_PATH);
    debug_assert!(!outcome.label.is_empty());
    debug_assert!(!output.name.is_empty());
    match read_offline_cargo_evidence(&evidence_path) {
        OfflineCargoEvidenceRead::Missing => CargoBuildEvidenceOutcome::Absent,
        OfflineCargoEvidenceRead::Valid(evidence) => CargoBuildEvidenceOutcome::Valid(BuildJsonCargoBuildEvidence {
            label: outcome.label.clone(),
            output_name: output.name.clone(),
            claim_class: evidence.claim_class,
            project_build_status: evidence.project_build_status,
            evidence_path: evidence_path.display().to_string(),
            evidence_schema: evidence.evidence_schema,
            evidence_version: evidence.evidence_version,
            evidence_kind: evidence.evidence_kind,
            target: evidence.target,
            profile: evidence.profile,
            binary: evidence.binary,
            lockfile: evidence.lockfile,
            cargo_command: evidence.cargo_command,
            network_policy: evidence.network_policy,
            output: evidence.output,
            source_closure: evidence.source_closure,
            toolchain: evidence.toolchain,
            non_claims: evidence.non_claims,
        }),
        OfflineCargoEvidenceRead::Invalid { blocker_class, message } => {
            CargoBuildEvidenceOutcome::Invalid(BuildJsonCargoBuildEvidenceDiagnostic {
                label: outcome.label.clone(),
                output_name: output.name.clone(),
                evidence_path: evidence_path.display().to_string(),
                blocker_class,
                message,
                next_action: OFFLINE_CARGO_EVIDENCE_NEXT_ACTION,
            })
        }
    }
}

fn read_offline_cargo_evidence(path: &Path) -> OfflineCargoEvidenceRead {
    if !path.is_file() {
        return OfflineCargoEvidenceRead::Missing;
    }
    debug_assert!(path.is_file());
    debug_assert!(!path.as_os_str().is_empty());
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) => {
            return OfflineCargoEvidenceRead::Invalid {
                blocker_class: "offline-cargo-evidence-read-error".to_string(),
                message: format!("offline Cargo evidence sidecar could not be read: {error}"),
            };
        }
    };
    let value: serde_json::Value = match serde_json::from_str(&text) {
        Ok(value) => value,
        Err(error) => {
            return OfflineCargoEvidenceRead::Invalid {
                blocker_class: "malformed-offline-cargo-evidence".to_string(),
                message: format!("offline Cargo evidence sidecar is not valid JSON: {error}"),
            };
        }
    };
    let schema = value.get("schema").and_then(serde_json::Value::as_str).unwrap_or_default();
    match schema {
        crate::offline_cargo::OFFLINE_CARGO_EVIDENCE_SCHEMA_V1 => read_legacy_offline_cargo_evidence(value),
        crate::offline_cargo::OFFLINE_CARGO_EVIDENCE_SCHEMA_V2 => read_digest_bound_offline_cargo_evidence(value),
        _ => OfflineCargoEvidenceRead::Invalid {
            blocker_class: "unsupported-offline-cargo-evidence-schema".to_string(),
            message: format!("unsupported offline Cargo evidence schema {schema}"),
        },
    }
}

fn read_legacy_offline_cargo_evidence(value: serde_json::Value) -> OfflineCargoEvidenceRead {
    let evidence = match serde_json::from_value::<crate::offline_cargo::OfflineCargoEvidenceV1File>(value) {
        Ok(evidence) => evidence,
        Err(error) => {
            return OfflineCargoEvidenceRead::Invalid {
                blocker_class: "malformed-offline-cargo-evidence".to_string(),
                message: format!("offline Cargo legacy evidence sidecar is malformed: {error}"),
            };
        }
    };
    let blockers = crate::offline_cargo::validate_legacy_evidence(&evidence);
    if !blockers.is_empty() {
        return blockers_to_invalid(blockers);
    }
    debug_assert!(blockers.is_empty());
    debug_assert_eq!(evidence.schema, crate::offline_cargo::OFFLINE_CARGO_EVIDENCE_SCHEMA_V1);
    OfflineCargoEvidenceRead::Valid(OfflineCargoEvidenceReport {
        claim_class: evidence.claim_class,
        project_build_status: evidence.project_build_status,
        evidence_schema: evidence.schema,
        evidence_version: crate::offline_cargo::OFFLINE_CARGO_EVIDENCE_VERSION_V1,
        evidence_kind: crate::offline_cargo::OFFLINE_CARGO_EVIDENCE_KIND_LEGACY.to_string(),
        target: None,
        profile: None,
        binary: None,
        lockfile: None,
        cargo_command: None,
        network_policy: None,
        output: None,
        source_closure: evidence.source_closure.into_iter().map(report_input).collect(),
        toolchain: report_toolchain(evidence.toolchain),
        non_claims: evidence.non_claims,
    })
}

fn read_digest_bound_offline_cargo_evidence(value: serde_json::Value) -> OfflineCargoEvidenceRead {
    let mut evidence = match serde_json::from_value::<crate::offline_cargo::OfflineCargoEvidenceV2File>(value) {
        Ok(evidence) => evidence,
        Err(error) => {
            return OfflineCargoEvidenceRead::Invalid {
                blocker_class: "malformed-offline-cargo-evidence".to_string(),
                message: format!("offline Cargo v2 evidence sidecar is malformed: {error}"),
            };
        }
    };
    if let Err((blocker_class, message)) = enrich_digest_bound_evidence(&mut evidence) {
        return OfflineCargoEvidenceRead::Invalid { blocker_class, message };
    }
    let blockers = crate::offline_cargo::validate_digest_bound_evidence(&evidence);
    if !blockers.is_empty() {
        return blockers_to_invalid(blockers);
    }
    debug_assert!(blockers.is_empty());
    debug_assert_eq!(evidence.schema, crate::offline_cargo::OFFLINE_CARGO_EVIDENCE_SCHEMA_V2);
    OfflineCargoEvidenceRead::Valid(OfflineCargoEvidenceReport {
        claim_class: evidence.claim_class,
        project_build_status: evidence.project_build_status,
        evidence_schema: evidence.schema,
        evidence_version: evidence.evidence_version,
        evidence_kind: crate::offline_cargo::OFFLINE_CARGO_EVIDENCE_KIND_DIGEST_BOUND.to_string(),
        target: Some(evidence.target),
        profile: Some(evidence.profile),
        binary: Some(evidence.binary),
        lockfile: Some(report_input(evidence.lockfile)),
        cargo_command: Some(BuildJsonCargoCommandEvidence {
            program: evidence.cargo_command.program,
            args: evidence.cargo_command.args,
        }),
        network_policy: Some(BuildJsonCargoNetworkPolicyEvidence {
            mode: evidence.network_policy.mode,
            result: evidence.network_policy.result,
            allow_undeclared_network: evidence.network_policy.allow_undeclared_network,
        }),
        output: Some(BuildJsonCargoOutputEvidence {
            binary: evidence.output.binary,
            path: evidence.output.path,
        }),
        source_closure: evidence.source_closure.into_iter().map(report_input).collect(),
        toolchain: report_toolchain(evidence.toolchain),
        non_claims: evidence.non_claims,
    })
}

fn enrich_digest_bound_evidence(
    evidence: &mut crate::offline_cargo::OfflineCargoEvidenceV2File,
) -> Result<(), (String, String)> {
    bind_file_digest(&mut evidence.lockfile)?;
    for input in &mut evidence.source_closure {
        if input.identity_class != crate::offline_cargo::OFFLINE_CARGO_IDENTITY_BLAKE3_CONTENT {
            continue;
        }
        bind_path_digest(input)?;
    }
    Ok(())
}

fn bind_file_digest(input: &mut crate::offline_cargo::OfflineCargoEvidenceInput) -> Result<(), (String, String)> {
    let path = std::path::PathBuf::from(&input.path);
    if !path.is_file() {
        return Err((
            "missing-lockfile-digest".to_string(),
            format!("offline Cargo lockfile evidence path is not a file: {}", input.path),
        ));
    }
    bind_actual_digest(input, &path)
}

fn bind_path_digest(input: &mut crate::offline_cargo::OfflineCargoEvidenceInput) -> Result<(), (String, String)> {
    let path = std::path::PathBuf::from(&input.path);
    if !path.exists() {
        return Err((
            "offline-cargo-evidence-digest-error".to_string(),
            format!("offline Cargo evidence input path is missing: {}", input.path),
        ));
    }
    bind_actual_digest(input, &path)
}

fn bind_actual_digest(
    input: &mut crate::offline_cargo::OfflineCargoEvidenceInput,
    path: &Path,
) -> Result<(), (String, String)> {
    let recorded_digest = input.digest_blake3.clone();
    let actual_digest = crate::release_evidence::compute_path_blake3_digest(path).map_err(|error| {
        (
            "offline-cargo-evidence-digest-error".to_string(),
            format!("offline Cargo evidence digest could not be computed for {}: {error}", path.display()),
        )
    })?;
    if input.expected_digest_blake3.is_none() {
        input.expected_digest_blake3 = recorded_digest;
    }
    input.digest_blake3 = Some(actual_digest);
    Ok(())
}

fn blockers_to_invalid(blockers: Vec<crate::offline_cargo::RustOfflineCargoBlocker>) -> OfflineCargoEvidenceRead {
    let first = &blockers[0];
    let additional = blockers.iter().skip(1).map(|blocker| blocker.class.as_str()).collect::<Vec<_>>();
    let message = if additional.is_empty() {
        first.message.clone()
    } else {
        format!("{}; additional blockers: {}", first.message, additional.join(","))
    };
    OfflineCargoEvidenceRead::Invalid {
        blocker_class: first.class.clone(),
        message,
    }
}

fn report_input(input: crate::offline_cargo::OfflineCargoEvidenceInput) -> BuildJsonCargoSourceClosureEntry {
    BuildJsonCargoSourceClosureEntry {
        role: input.role,
        path: input.path,
        digest_blake3: input.digest_blake3,
        expected_digest_blake3: input.expected_digest_blake3,
        identity_class: input.identity_class,
    }
}

fn report_toolchain(toolchain: crate::offline_cargo::OfflineCargoEvidenceToolchain) -> BuildJsonCargoToolchainEvidence {
    BuildJsonCargoToolchainEvidence {
        cargo: toolchain.cargo,
        rustc: toolchain.rustc,
        linker: toolchain.linker,
    }
}

enum AstGrepStructuralEvidenceOutcome {
    Absent,
    Valid(BuildJsonAstGrepStructuralEvidence),
    Invalid(BuildJsonAstGrepStructuralEvidenceDiagnostic),
}

enum CargoBuildEvidenceOutcome {
    Absent,
    Valid(BuildJsonCargoBuildEvidence),
    Invalid(BuildJsonCargoBuildEvidenceDiagnostic),
}

struct OfflineCargoEvidenceReport {
    claim_class: String,
    project_build_status: String,
    evidence_schema: String,
    evidence_version: u32,
    evidence_kind: String,
    target: Option<String>,
    profile: Option<String>,
    binary: Option<String>,
    lockfile: Option<BuildJsonCargoSourceClosureEntry>,
    cargo_command: Option<BuildJsonCargoCommandEvidence>,
    network_policy: Option<BuildJsonCargoNetworkPolicyEvidence>,
    output: Option<BuildJsonCargoOutputEvidence>,
    source_closure: Vec<BuildJsonCargoSourceClosureEntry>,
    toolchain: BuildJsonCargoToolchainEvidence,
    non_claims: Vec<String>,
}

enum OfflineCargoEvidenceRead {
    Missing,
    Valid(OfflineCargoEvidenceReport),
    Invalid { blocker_class: String, message: String },
}

fn success_log_file(logs_dir: &Path, outcome: &crunch_build::BuildOutcome) -> Option<String> {
    if outcome.log.is_none() && outcome.cached {
        return None;
    }
    existing_log_file_path(logs_dir, &outcome.drv_path)
}

fn count_as_u32(count: usize) -> u32 {
    assert!(u32::try_from(count).is_ok(), "count must fit in u32");
    count as u32
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use serde_json::json;

    use super::*;

    const SENSITIVE_PRIORITY_GOAL: &str = "/private/report-secret/root.drv";
    const SAMPLE_PRIORITY_EPOCH: u32 = 1;
    const SAMPLE_PRIORITY_PATH_NODES: u32 = 2;
    const TEST_WORKSPACE_DIGEST_HEX_LENGTH: usize = 64;

    fn sample_priority_decision() -> crunch_pipeline::PriorityDecisionEvidence {
        let policy = crunch_build::SchedulingPolicy::default();
        let ready = crunch_build::ReadyGoalFacts::ordinary(SENSITIVE_PRIORITY_GOAL.to_string(), 0);
        let pressures = std::collections::BTreeMap::from([(
            SENSITIVE_PRIORITY_GOAL.to_string(),
            crunch_build::KnownGraphPressure {
                known_critical_path_nodes: SAMPLE_PRIORITY_PATH_NODES,
                known_critical_path_work_units: SAMPLE_PRIORITY_PATH_NODES,
                blocked_root_count: SAMPLE_PRIORITY_EPOCH,
                blocked_root_count_saturated: false,
            },
        )]);
        let ranked = crunch_build::rank_ready_goals(&policy, SAMPLE_PRIORITY_EPOCH, &[ready], &pressures).unwrap();
        crunch_build::priority_decision_evidence(
            &policy,
            SAMPLE_PRIORITY_EPOCH,
            &ranked,
            crunch_build::HistoryBasis::StructuralFallbackMissing,
            None,
        )
        .unwrap()
    }

    #[test]
    fn count_as_u32_round_trips_small_values() {
        assert_eq!(count_as_u32(0), 0);
        assert_eq!(count_as_u32(7), 7);
    }

    #[test]
    fn build_counts_splits_cached_and_built() {
        let outcomes = vec![
            BuildJsonOutcome {
                drv_key: "a".into(),
                label: "a".into(),
                cached: false,
                log_file: None,
                outputs: Vec::new(),
            },
            BuildJsonOutcome {
                drv_key: "b".into(),
                label: "b".into(),
                cached: true,
                log_file: None,
                outputs: Vec::new(),
            },
        ];
        let failed = vec![BuildFailureEnvelope {
            root: "c".into(),
            drv_key: "c".into(),
            phase: crate::build_failure::FailurePhase::Build,
            error_class: crate::build_failure::FailureClass::Builder,
            message: "boom".into(),
            saved_log_path: Some("c.log".into()),
        }];

        let counts = build_counts(&outcomes, &failed);
        assert_eq!(counts.succeeded_total, 2);
        assert_eq!(counts.built_total, 1);
        assert_eq!(counts.cached_total, 1);
        assert_eq!(counts.failed_total, 1);
    }

    #[test]
    fn success_log_file_is_none_when_log_is_missing() {
        let logs_dir = tempfile::tempdir().unwrap();
        let drv_path = nix_compat::store_path::StorePath::from_name_and_digest_fixed("demo.drv", [1u8; 20]).unwrap();
        let outcome = crunch_build::BuildOutcome {
            drv_path,
            outputs: std::collections::BTreeMap::new(),
            substitutions: std::collections::BTreeMap::new(),
            cached: false,
            log: Some("body".into()),
        };
        let log_path = success_log_file(logs_dir.path(), &outcome);
        assert!(log_path.is_none());
    }

    #[test]
    fn build_json_failure_report_uses_typed_envelope_schema() {
        use std::collections::HashMap;

        let state_dir = tempfile::tempdir().unwrap();
        let output_dir = tempfile::tempdir().unwrap();
        let logs_dir = tempfile::tempdir().unwrap();
        let signing_key = crate::build_cmd::load_or_generate_signing_keypair(None, state_dir.path(), false).unwrap();
        let config = BuildConfig {
            file: output_dir.path().join("demo.ncl"),
            import_paths: Vec::new(),
            output_dir: output_dir.path().to_path_buf(),
            state_dir: state_dir.path().to_path_buf(),
            base_state_dirs: Vec::new(),
            store_dir: "/crunch/store".to_string(),
            verbose: false,
            max_jobs: 1,
            scheduling_policy: crunch_pipeline::SchedulingPolicy::default(),
            substituter_urls: Vec::new(),
            hermeticity_mode: crunch_pipeline::HermeticityMode::Practical,
            keypair: signing_key,
            trusted_keys: Vec::new(),
            trust_unsigned: false,
            root_retention_source: None,
            source_fetch_overrides: Vec::new(),
            remote_enabled: false,
        };
        let drv_path = nix_compat::store_path::StorePath::from_name_and_digest_fixed("demo.drv", [9u8; 20]).unwrap();
        crate::build_log::write_log_file(logs_dir.path(), &drv_path, "demo", false, "failure body").unwrap();
        let drv_key = drv_key_for(&config.store_dir, &drv_path);
        let result = PipelineResult {
            outcomes: Vec::new(),
            failed: vec![crunch_build::FailedGoal {
                drv_key: drv_key.clone(),
                origin_drv_key: drv_key.clone(),
                error: "FOD hash mismatch for demo: expected sha256-a, got sha256-b".to_string(),
                origin_error: "FOD hash mismatch for demo: expected sha256-a, got sha256-b".to_string(),
                build_log: None,
            }],
            fod_mismatches: vec![crunch_pipeline::FodMismatch {
                name: "demo".to_string(),
                expected_sri: "sha256-a".to_string(),
                actual_sri: "sha256-b".to_string(),
            }],
            root_labels: HashMap::from([(drv_key.clone(), "demo".to_string())]),
            hermeticity_mode: crunch_pipeline::HermeticityMode::Practical,
            hermeticity_audit_events: Vec::new(),
            build_environment_reports: Vec::new(),
            network_policy_reports: Vec::new(),
            workspace_reports: Vec::new(),
            action_result_reports: Vec::new(),
            native_dynamic_plans: Vec::new(),
            priority_decisions: Vec::new(),
        };

        let json_report = render_build_json_report(&config, &result, logs_dir.path(), &[]).unwrap();
        let json_value: serde_json::Value = serde_json::from_str(&json_report).unwrap();
        let saved_log_path = crate::build_log::existing_log_file_path(logs_dir.path(), &drv_path).unwrap();

        assert_eq!(
            json_value["failed"][0],
            json!({
                "root": "demo",
                "drv_key": drv_key,
                "phase": "build",
                "error_class": "fixed-output-hash-mismatch",
                "message": "FOD hash mismatch for demo: expected sha256-a, got sha256-b",
                "saved_log_path": saved_log_path,
            })
        );
    }

    #[test]
    fn build_json_preflight_failure_omits_saved_log_path_field() {
        use std::collections::HashMap;

        let state_dir = tempfile::tempdir().unwrap();
        let output_dir = tempfile::tempdir().unwrap();
        let logs_dir = tempfile::tempdir().unwrap();
        let signing_key = crate::build_cmd::load_or_generate_signing_keypair(None, state_dir.path(), false).unwrap();
        let config = BuildConfig {
            file: output_dir.path().join("demo.ncl"),
            import_paths: Vec::new(),
            output_dir: output_dir.path().to_path_buf(),
            state_dir: state_dir.path().to_path_buf(),
            base_state_dirs: Vec::new(),
            store_dir: "/crunch/store".to_string(),
            verbose: false,
            max_jobs: 1,
            scheduling_policy: crunch_pipeline::SchedulingPolicy::default(),
            substituter_urls: Vec::new(),
            hermeticity_mode: crunch_pipeline::HermeticityMode::Practical,
            keypair: signing_key,
            trusted_keys: Vec::new(),
            trust_unsigned: false,
            root_retention_source: None,
            source_fetch_overrides: Vec::new(),
            remote_enabled: false,
        };
        let drv_path = nix_compat::store_path::StorePath::from_name_and_digest_fixed("demo.drv", [10u8; 20]).unwrap();
        let drv_key = drv_key_for(&config.store_dir, &drv_path);
        let result = PipelineResult {
            outcomes: Vec::new(),
            failed: vec![crunch_build::FailedGoal {
                drv_key: drv_key.clone(),
                origin_drv_key: drv_key.clone(),
                error: "strict mode does not permit in-memory PathInfo fallback: broken redb".to_string(),
                origin_error: "strict mode does not permit in-memory PathInfo fallback: broken redb".to_string(),
                build_log: None,
            }],
            fod_mismatches: Vec::new(),
            root_labels: HashMap::from([(drv_key.clone(), "demo".to_string())]),
            hermeticity_mode: crunch_pipeline::HermeticityMode::Practical,
            hermeticity_audit_events: Vec::new(),
            build_environment_reports: Vec::new(),
            network_policy_reports: Vec::new(),
            workspace_reports: Vec::new(),
            action_result_reports: Vec::new(),
            native_dynamic_plans: Vec::new(),
            priority_decisions: Vec::new(),
        };

        let json_report = render_build_json_report(&config, &result, logs_dir.path(), &[]).unwrap();
        let json_value: serde_json::Value = serde_json::from_str(&json_report).unwrap();
        assert_eq!(json_value["failed"][0]["phase"], "preflight");
        assert_eq!(json_value["failed"][0]["error_class"], "preflight");
        assert!(json_value["failed"][0].get("saved_log_path").is_none());
    }

    #[test]
    fn build_json_report_includes_artifact_attestation_reference() {
        use std::collections::HashMap;

        let state_dir = tempfile::tempdir().unwrap();
        let output_dir = tempfile::tempdir().unwrap();
        let logs_dir = tempfile::tempdir().unwrap();
        let signing_key = crate::build_cmd::load_or_generate_signing_keypair(None, state_dir.path(), false).unwrap();
        let config = BuildConfig {
            file: output_dir.path().join("demo.ncl"),
            import_paths: Vec::new(),
            output_dir: output_dir.path().to_path_buf(),
            state_dir: state_dir.path().to_path_buf(),
            base_state_dirs: Vec::new(),
            store_dir: "/crunch/store".to_string(),
            verbose: false,
            max_jobs: 1,
            scheduling_policy: crunch_pipeline::SchedulingPolicy::default(),
            substituter_urls: Vec::new(),
            hermeticity_mode: crunch_pipeline::HermeticityMode::Practical,
            keypair: signing_key,
            trusted_keys: Vec::new(),
            trust_unsigned: false,
            root_retention_source: None,
            source_fetch_overrides: Vec::new(),
            remote_enabled: false,
        };
        let drv_path = nix_compat::store_path::StorePath::from_name_and_digest_fixed("demo.drv", [1u8; 20]).unwrap();
        let output_path = nix_compat::store_path::StorePath::from_name_and_digest_fixed("demo", [2u8; 20]).unwrap();
        let path_info = snix_store::path_info::PathInfo {
            store_path: output_path.clone(),
            node: snix_castore::Node::Symlink {
                target: snix_castore::SymlinkTarget::try_from("target").unwrap(),
            },
            references: vec![],
            nar_size: 1,
            nar_sha256: [0x11; 32],
            signatures: vec![],
            deriver: None,
            ca: None,
        };
        let outcome = crunch_build::BuildOutcome {
            drv_path: drv_path.clone(),
            outputs: std::collections::BTreeMap::from([("out".to_string(), path_info)]),
            substitutions: std::collections::BTreeMap::from([(
                "out".to_string(),
                crunch_store::OutputSubstitutionReport {
                    mode: crunch_store::OutputSubstitutionMode::Delta,
                    transferred_bytes: 12,
                    reused_bytes: 34,
                    metadata_reused: false,
                    fallback_reason: None,
                },
            )]),
            cached: false,
            log: None,
        };
        let result = PipelineResult {
            outcomes: vec![outcome],
            failed: Vec::new(),
            fod_mismatches: Vec::new(),
            root_labels: HashMap::from([(drv_key_for(&config.store_dir, &drv_path), "demo".to_string())]),
            hermeticity_mode: crunch_pipeline::HermeticityMode::Practical,
            hermeticity_audit_events: vec![crunch_pipeline::HermeticityAuditEvent::new(
                crunch_pipeline::HermeticityAuditKind::HostToolFallback,
                "using external bwrap",
            )],
            build_environment_reports: vec![crunch_pipeline::BuildEnvironmentReport {
                action_name: "demo".to_string(),
                digest_blake3: Some("env-digest".to_string()),
                variable_count: 3,
                rejections: Vec::new(),
                search_path: Some(crunch_pipeline::BuildSearchPathReport {
                    digest_blake3: Some("path-digest".to_string()),
                    entries: vec![crunch_pipeline::BuildSearchPathEntry {
                        path: "/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-tool/bin".to_string(),
                        kind: "declared-tool-ref".to_string(),
                        real_tool_ref: "/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-tool".to_string(),
                    }],
                    aliases: Vec::new(),
                    real_tool_refs: vec!["/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-tool".to_string()],
                }),
                determinism: Some(crunch_pipeline::BuildDeterminismNormalizationReport {
                    policy_digest_blake3: "determinism-digest".to_string(),
                    controls: vec![crunch_pipeline::BuildDeterminismControl {
                        surface: "time".to_string(),
                        policy: "fixed-env".to_string(),
                        value: "SOURCE_DATE_EPOCH=1".to_string(),
                        enforcement: "enforced".to_string(),
                    }],
                    unsupported_controls: Vec::new(),
                    divergence: None,
                    strong_claim_blocked: false,
                }),
            }],
            network_policy_reports: vec![crunch_pipeline::BuildNetworkPolicyReport {
                action_name: "demo".to_string(),
                mode: "offline".to_string(),
                result: "denied".to_string(),
                capability: None,
                policy_basis: Some("ordinary derivation network access is denied by default".to_string()),
                audit_class: None,
                fixed_output: None,
                diagnostic: None,
            }],
            workspace_reports: vec![crunch_build::WorkspaceExecutionReport {
                schema: crunch_build::WORKSPACE_EXECUTION_REPORT_SCHEMA.to_string(),
                mode: crunch_build::WorkspaceMode::MutableSession,
                workspace_id: Some("cargo-cache".to_string()),
                guest_path: crunch_build::DEFAULT_WORKSPACE_GUEST_PATH.to_string(),
                compatibility_digest_blake3: "a".repeat(TEST_WORKSPACE_DIGEST_HEX_LENGTH),
                warm_state_used: true,
                claim_class: crunch_build::WorkspaceClaimClass::PracticalMutableHistory,
                shared_action_publish_allowed: false,
                strong_shared_reuse_allowed: false,
                original_execution_hermetic: false,
                clean_comparison_performed: true,
                clean_comparison_matched: true,
                warm_output_set_digest_blake3: Some("warm-digest".to_string()),
                clean_output_set_digest_blake3: Some("warm-digest".to_string()),
                cleanup: crunch_build::WorkspaceCleanupDisposition::Released,
                cleanup_reason: crunch_build::WorkspaceReasonCode::Accepted,
                snapshot_ref: None,
            }],
            action_result_reports: vec![crunch_build::ActionResultRuntimeReport {
                schema: "mantle-action-result-runtime-report-v1".to_string(),
                phase: "discovery".to_string(),
                action_ref: "action-b3:demo".to_string(),
                disposition: "reused".to_string(),
                selected_result_ref: Some("result-b3:demo".to_string()),
                selected_source_id: Some("local-state".to_string()),
                selected_source_class: Some("local".to_string()),
                trust_basis: vec!["record-signature-verified:builder-key-1".to_string()],
                conflict_class: None,
                candidate_decisions: Vec::new(),
                publication_result_refs: Vec::new(),
                transfer: None,
                diagnostics: Vec::new(),
                non_claims: vec!["index-presence-is-not-output-trust".to_string()],
            }],
            native_dynamic_plans: vec![crunch_build::NativeDynamicPlanReport {
                mode: "native".to_string(),
                producer_key: drv_key_for(&config.store_dir, &drv_path),
                output_name: "plan".to_string(),
                plan_artifact_path: Some(output_path.clone()),
                raw_artifact_digest: Some("raw-digest".to_string()),
                canonical_plan_digest: Some("canonical-digest".to_string()),
                accepted_unit_ids: vec!["unit.build".to_string()],
                rejection_reason: None,
                scheduler_action: "registered-roots".to_string(),
            }],
            priority_decisions: vec![sample_priority_decision()],
        };

        let report = build_json_report(&config, &result, logs_dir.path(), &[], &[]);
        let logical_path = output_path.to_absolute_path_with_prefix(&config.store_dir);

        assert_eq!(report.hermeticity_mode, "practical");
        assert_eq!(report.hermeticity_audit_events.len(), 1);
        assert_eq!(report.hermeticity_audit_events[0].kind, "host-tool-fallback");
        assert_eq!(report.build_environment_reports.len(), 1);
        assert_eq!(report.build_environment_reports[0].action_name, "demo");
        assert_eq!(report.build_environment_reports[0].digest_blake3.as_deref(), Some("env-digest"));
        assert_eq!(report.build_environment_reports[0].variable_count, 3);
        assert!(report.build_environment_reports[0].rejections.is_empty());
        let search_path = report.build_environment_reports[0].search_path.as_ref().expect("search path report");
        assert_eq!(search_path.digest_algorithm, "blake3");
        assert_eq!(search_path.digest_blake3.as_deref(), Some("path-digest"));
        assert_eq!(search_path.entries[0].kind, "declared-tool-ref");
        assert_eq!(search_path.real_tool_refs, vec!["/mantle/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-tool".to_string()]);
        let determinism = report.build_environment_reports[0].determinism.as_ref().expect("determinism report");
        assert_eq!(determinism.digest_algorithm, "blake3");
        assert_eq!(determinism.policy_digest_blake3, "determinism-digest");
        assert_eq!(determinism.controls[0].surface, "time");
        assert!(!determinism.strong_claim_blocked);
        assert_eq!(report.scheduler_policy, crunch_pipeline::SchedulingPolicy::default());
        assert_eq!(report.network_policy_reports.len(), 1);
        assert_eq!(report.network_policy_reports[0].action_name, "demo");
        assert_eq!(report.network_policy_reports[0].mode, "offline");
        assert_eq!(report.network_policy_reports[0].result, "denied");
        assert_eq!(report.workspace_reports.len(), 1);
        assert_eq!(report.workspace_reports[0].mode, crunch_build::WorkspaceMode::MutableSession);
        assert!(report.workspace_reports[0].warm_state_used);
        assert!(!report.workspace_reports[0].shared_action_publish_allowed);
        assert!(!report.workspace_reports[0].original_execution_hermetic);
        assert!(report.workspace_reports[0].clean_comparison_matched);
        assert_eq!(report.action_result_reports.len(), 1);
        assert_eq!(report.action_result_reports[0].disposition, "reused");
        assert_eq!(report.action_result_reports[0].selected_source_class.as_deref(), Some("local"));
        assert!(
            report.action_result_reports[0]
                .non_claims
                .contains(&"index-presence-is-not-output-trust".to_string())
        );
        assert_eq!(report.native_dynamic_plans.len(), 1);
        assert_eq!(report.native_dynamic_plans[0].mode, "native");
        assert_eq!(report.native_dynamic_plans[0].output_name, "plan");
        assert_eq!(report.native_dynamic_plans[0].scheduler_action, "registered-roots");
        assert_eq!(report.native_dynamic_plans[0].accepted_unit_ids, vec!["unit.build".to_string()]);
        assert_eq!(report.scheduler_priority_decisions.len(), 1);
        let priority_json = serde_json::to_string(&report.scheduler_priority_decisions).unwrap();
        assert!(!priority_json.contains(SENSITIVE_PRIORITY_GOAL));
        assert!(priority_json.contains("configured-known-fact-ordering"));
        assert_eq!(
            report.native_dynamic_plans[0].plan_artifact_path.as_deref(),
            Some(output_path.to_absolute_path_with_prefix(&config.store_dir).as_str())
        );
        assert_eq!(report.outcomes.len(), 1);
        assert_eq!(report.outcomes[0].outputs.len(), 1);
        assert_eq!(report.outcomes[0].outputs[0].artifact_attestation.logical_path, logical_path);
        assert!(report.outcomes[0].outputs[0].artifact_attestation.path.contains("attestations/artifacts/"));
        let substitution = report.outcomes[0].outputs[0].substitution.as_ref().expect("substitution report");
        assert_eq!(substitution.mode, "delta");
        assert_eq!(substitution.transferred_bytes, 12);
        assert_eq!(substitution.reused_bytes, 34);
        assert!(substitution.fallback_reason.is_none());
    }

    #[test]
    fn build_json_report_surfaces_legacy_offline_cargo_evidence_sidecar() {
        use std::collections::HashMap;

        const MIN_MAX_JOBS: u32 = 1;
        const NAR_SIZE_BYTES: u64 = 1;
        const TRANSFERRED_BYTES: u64 = 0;
        const REUSED_BYTES: u64 = 0;
        const NAR_HASH_BYTE: u8 = 0x33;

        let state_dir = tempfile::tempdir().unwrap();
        let output_dir = tempfile::tempdir().unwrap();
        let logs_dir = tempfile::tempdir().unwrap();
        let signing_key = crate::build_cmd::load_or_generate_signing_keypair(None, state_dir.path(), false).unwrap();
        let config = BuildConfig {
            file: output_dir.path().join("demo.ncl"),
            import_paths: Vec::new(),
            output_dir: output_dir.path().to_path_buf(),
            state_dir: state_dir.path().to_path_buf(),
            base_state_dirs: Vec::new(),
            store_dir: "/crunch/store".to_string(),
            verbose: false,
            max_jobs: MIN_MAX_JOBS,
            scheduling_policy: crunch_pipeline::SchedulingPolicy::default(),
            substituter_urls: Vec::new(),
            hermeticity_mode: crunch_pipeline::HermeticityMode::Practical,
            keypair: signing_key,
            trusted_keys: Vec::new(),
            trust_unsigned: false,
            root_retention_source: None,
            source_fetch_overrides: Vec::new(),
            remote_enabled: false,
        };
        let drv_path = nix_compat::store_path::StorePath::from_name_and_digest_fixed("demo.drv", [5u8; 20]).unwrap();
        let output_path = nix_compat::store_path::StorePath::from_name_and_digest_fixed("demo", [6u8; 20]).unwrap();
        let exported_path =
            PathBuf::from(output_path.to_absolute_path_with_prefix(config.output_dir.to_str().unwrap()));
        let evidence_path = exported_path.join(crate::offline_cargo::OFFLINE_CARGO_EVIDENCE_RELATIVE_PATH);
        std::fs::create_dir_all(evidence_path.parent().unwrap()).unwrap();
        std::fs::write(
            &evidence_path,
            serde_json::json!({
                "schema": crate::offline_cargo::OFFLINE_CARGO_EVIDENCE_SCHEMA_V1,
                "claim_class": crate::offline_cargo::OFFLINE_CARGO_EVIDENCE_CLASS,
                "project_build_status": crate::offline_cargo::OFFLINE_CARGO_PROJECT_BUILD_STATUS,
                "source_closure": [{"role": "package-source", "path": "/crunch/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-demo-src"}],
                "toolchain": {
                    "cargo": "/crunch/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-rust/bin/cargo",
                    "rustc": "/crunch/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-rust/bin/rustc",
                    "linker": "/crunch/store/cccccccccccccccccccccccccccccccc-seed/bin/x86_64-linux-musl-gcc",
                },
                "non_claims": crate::offline_cargo::offline_cargo_non_claims(),
            })
            .to_string(),
        )
        .unwrap();
        let path_info = snix_store::path_info::PathInfo {
            store_path: output_path.clone(),
            node: snix_castore::Node::Symlink {
                target: snix_castore::SymlinkTarget::try_from("target").unwrap(),
            },
            references: vec![],
            nar_size: NAR_SIZE_BYTES,
            nar_sha256: [NAR_HASH_BYTE; 32],
            signatures: vec![],
            deriver: None,
            ca: None,
        };
        let outcome = crunch_build::BuildOutcome {
            drv_path: drv_path.clone(),
            outputs: std::collections::BTreeMap::from([("out".to_string(), path_info)]),
            substitutions: std::collections::BTreeMap::from([(
                "out".to_string(),
                crunch_store::OutputSubstitutionReport {
                    mode: crunch_store::OutputSubstitutionMode::Full,
                    transferred_bytes: TRANSFERRED_BYTES,
                    reused_bytes: REUSED_BYTES,
                    metadata_reused: false,
                    fallback_reason: None,
                },
            )]),
            cached: false,
            log: None,
        };
        let result = PipelineResult {
            outcomes: vec![outcome],
            failed: Vec::new(),
            fod_mismatches: Vec::new(),
            root_labels: HashMap::from([(drv_key_for(&config.store_dir, &drv_path), "demo".to_string())]),
            hermeticity_mode: crunch_pipeline::HermeticityMode::Practical,
            hermeticity_audit_events: Vec::new(),
            build_environment_reports: Vec::new(),
            network_policy_reports: Vec::new(),
            workspace_reports: Vec::new(),
            action_result_reports: Vec::new(),
            native_dynamic_plans: Vec::new(),
            priority_decisions: Vec::new(),
        };

        let report = build_json_report(&config, &result, logs_dir.path(), &[], &[]);

        assert_eq!(report.cargo_build_evidence.len(), 1);
        assert_eq!(report.cargo_build_evidence[0].label, "demo");
        assert_eq!(report.cargo_build_evidence[0].claim_class, crate::offline_cargo::OFFLINE_CARGO_EVIDENCE_CLASS);
        assert_eq!(
            report.cargo_build_evidence[0].project_build_status,
            crate::offline_cargo::OFFLINE_CARGO_PROJECT_BUILD_STATUS
        );
        assert_eq!(report.cargo_build_evidence[0].evidence_path, evidence_path.display().to_string());
        assert_eq!(
            report.cargo_build_evidence[0].evidence_schema,
            crate::offline_cargo::OFFLINE_CARGO_EVIDENCE_SCHEMA_V1
        );
        assert_eq!(
            report.cargo_build_evidence[0].evidence_kind,
            crate::offline_cargo::OFFLINE_CARGO_EVIDENCE_KIND_LEGACY
        );
        assert!(report.cargo_build_evidence[0].lockfile.is_none());
        assert!(report.cargo_build_evidence[0].non_claims.contains(&"not-full-cargo-compatibility".to_string()));
        assert!(report.cargo_build_evidence_diagnostics.is_empty());
    }

    #[test]
    fn build_json_report_surfaces_digest_bound_offline_cargo_evidence_sidecar() {
        const LOCK_CONTENT: &str = "# lock\nversion = 4\n";
        const SOURCE_CONTENT: &str = "fn main() {}\n";
        const VENDOR_CONTENT: &str = "vendor-checksum\n";

        let output_root = tempfile::tempdir().unwrap();
        let source_dir = output_root.path().join("source");
        let source_src_dir = source_dir.join("src");
        let vendor_dir = output_root.path().join("vendor");
        std::fs::create_dir_all(&source_src_dir).unwrap();
        std::fs::create_dir_all(&vendor_dir).unwrap();
        std::fs::write(source_dir.join("Cargo.lock"), LOCK_CONTENT).unwrap();
        std::fs::write(source_src_dir.join("main.rs"), SOURCE_CONTENT).unwrap();
        std::fs::write(vendor_dir.join("checksum.txt"), VENDOR_CONTENT).unwrap();
        let evidence_path = output_root.path().join(crate::offline_cargo::OFFLINE_CARGO_EVIDENCE_RELATIVE_PATH);
        std::fs::create_dir_all(evidence_path.parent().unwrap()).unwrap();
        std::fs::write(
            &evidence_path,
            serde_json::json!({
                "schema": crate::offline_cargo::OFFLINE_CARGO_EVIDENCE_SCHEMA_V2,
                "evidence_version": crate::offline_cargo::OFFLINE_CARGO_EVIDENCE_VERSION_V2,
                "claim_class": crate::offline_cargo::OFFLINE_CARGO_EVIDENCE_CLASS,
                "project_build_status": crate::offline_cargo::OFFLINE_CARGO_PROJECT_BUILD_STATUS,
                "target": "x86_64-unknown-linux-musl",
                "profile": "release",
                "binary": "demo",
                "lockfile": {
                    "role": crate::offline_cargo::OFFLINE_CARGO_LOCKFILE_ROLE,
                    "path": source_dir.join("Cargo.lock").display().to_string(),
                    "identity_class": crate::offline_cargo::OFFLINE_CARGO_IDENTITY_BLAKE3_CONTENT,
                },
                "source_closure": [
                    {
                        "role": "package-source",
                        "path": source_dir.display().to_string(),
                        "identity_class": crate::offline_cargo::OFFLINE_CARGO_IDENTITY_BLAKE3_CONTENT,
                    },
                    {
                        "role": "rust-toolchain",
                        "path": "/crunch/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-rust",
                        "identity_class": crate::offline_cargo::OFFLINE_CARGO_IDENTITY_STORE_PATH,
                    },
                    {
                        "role": "seed-toolchain",
                        "path": "/crunch/store/cccccccccccccccccccccccccccccccc-seed",
                        "identity_class": crate::offline_cargo::OFFLINE_CARGO_IDENTITY_STORE_PATH,
                    },
                    {
                        "role": "musl-runtime",
                        "path": "/crunch/store/dddddddddddddddddddddddddddddddd-musl",
                        "identity_class": crate::offline_cargo::OFFLINE_CARGO_IDENTITY_STORE_PATH,
                    },
                    {
                        "role": "vendored-dependencies",
                        "path": vendor_dir.display().to_string(),
                        "identity_class": crate::offline_cargo::OFFLINE_CARGO_IDENTITY_BLAKE3_CONTENT,
                    }
                ],
                "toolchain": {
                    "cargo": "/crunch/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-rust/bin/cargo",
                    "rustc": "/crunch/store/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb-rust/bin/rustc",
                    "linker": "/crunch/store/cccccccccccccccccccccccccccccccc-seed/bin/x86_64-linux-musl-gcc",
                },
                "cargo_command": {
                    "program": crate::offline_cargo::OFFLINE_CARGO_COMMAND_PROGRAM,
                    "args": ["build", "--locked", "--offline", "--release", "--bin", "demo", "--target", "x86_64-unknown-linux-musl"],
                },
                "network_policy": {
                    "mode": crate::offline_cargo::OFFLINE_CARGO_NETWORK_MODE,
                    "result": crate::offline_cargo::OFFLINE_CARGO_NETWORK_RESULT,
                    "allow_undeclared_network": false,
                },
                "output": {
                    "binary": "demo",
                    "path": output_root.path().join("bin/demo").display().to_string(),
                },
                "non_claims": crate::offline_cargo::offline_cargo_non_claims(),
            })
            .to_string(),
        )
        .unwrap();
        let output = BuildJsonOutput {
            name: "out".to_string(),
            path: output_root.path().display().to_string(),
            artifact_attestation: BuildJsonAttestationReference {
                logical_path: "/crunch/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-demo".to_string(),
                path: "attestation.json".to_string(),
            },
            substitution: None,
            cache_admission: None,
        };
        let outcome = build_json_outcome_with_output(output);

        let (reports, diagnostics) = build_cargo_build_evidence_reports(&[outcome]);
        let lock_digest = blake3::hash(LOCK_CONTENT.as_bytes()).to_hex().to_string();
        let source_digest = crate::release_evidence::compute_path_blake3_digest(&source_dir).unwrap();
        let vendor_digest = crate::release_evidence::compute_path_blake3_digest(&vendor_dir).unwrap();

        assert_eq!(reports.len(), 1);
        assert!(diagnostics.is_empty(), "diagnostics: {diagnostics:#?}");
        assert_eq!(reports[0].evidence_kind, crate::offline_cargo::OFFLINE_CARGO_EVIDENCE_KIND_DIGEST_BOUND);
        assert_eq!(reports[0].target.as_deref(), Some("x86_64-unknown-linux-musl"));
        assert_eq!(reports[0].profile.as_deref(), Some("release"));
        assert_eq!(reports[0].binary.as_deref(), Some("demo"));
        assert_eq!(reports[0].lockfile.as_ref().unwrap().digest_blake3.as_deref(), Some(lock_digest.as_str()));
        let package_source = reports[0]
            .source_closure
            .iter()
            .find(|entry| entry.role == "package-source")
            .expect("package source evidence");
        let vendor_source = reports[0]
            .source_closure
            .iter()
            .find(|entry| entry.role == "vendored-dependencies")
            .expect("vendor source evidence");
        assert_eq!(package_source.digest_blake3.as_deref(), Some(source_digest.as_str()));
        assert_eq!(vendor_source.digest_blake3.as_deref(), Some(vendor_digest.as_str()));
        assert_eq!(
            reports[0].network_policy.as_ref().unwrap().result,
            crate::offline_cargo::OFFLINE_CARGO_NETWORK_RESULT
        );
        assert!(reports[0].non_claims.contains(&"not-compiler-correctness".to_string()));
    }

    #[test]
    fn build_json_report_diagnoses_stale_digest_bound_offline_cargo_evidence_sidecar() {
        const LOCK_CONTENT: &str = "# lock\nversion = 4\n";
        const STALE_DIGEST: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

        let output_root = tempfile::tempdir().unwrap();
        let source_dir = output_root.path().join("source");
        let source_src_dir = source_dir.join("src");
        std::fs::create_dir_all(&source_src_dir).unwrap();
        std::fs::write(source_dir.join("Cargo.lock"), LOCK_CONTENT).unwrap();
        std::fs::write(source_src_dir.join("main.rs"), "fn main() {}\n").unwrap();
        let evidence_path = output_root.path().join(crate::offline_cargo::OFFLINE_CARGO_EVIDENCE_RELATIVE_PATH);
        std::fs::create_dir_all(evidence_path.parent().unwrap()).unwrap();
        std::fs::write(
            &evidence_path,
            serde_json::json!({
                "schema": crate::offline_cargo::OFFLINE_CARGO_EVIDENCE_SCHEMA_V2,
                "evidence_version": crate::offline_cargo::OFFLINE_CARGO_EVIDENCE_VERSION_V2,
                "claim_class": crate::offline_cargo::OFFLINE_CARGO_EVIDENCE_CLASS,
                "project_build_status": crate::offline_cargo::OFFLINE_CARGO_PROJECT_BUILD_STATUS,
                "target": "x86_64-unknown-linux-musl",
                "profile": "release",
                "binary": "demo",
                "lockfile": {
                    "role": crate::offline_cargo::OFFLINE_CARGO_LOCKFILE_ROLE,
                    "path": source_dir.join("Cargo.lock").display().to_string(),
                    "digest_blake3": STALE_DIGEST,
                    "identity_class": crate::offline_cargo::OFFLINE_CARGO_IDENTITY_BLAKE3_CONTENT,
                },
                "source_closure": [
                    {
                        "role": "package-source",
                        "path": source_dir.display().to_string(),
                        "identity_class": crate::offline_cargo::OFFLINE_CARGO_IDENTITY_BLAKE3_CONTENT,
                    },
                    {"role": "rust-toolchain", "path": "/crunch/store/rust", "identity_class": crate::offline_cargo::OFFLINE_CARGO_IDENTITY_STORE_PATH},
                    {"role": "seed-toolchain", "path": "/crunch/store/seed", "identity_class": crate::offline_cargo::OFFLINE_CARGO_IDENTITY_STORE_PATH},
                    {"role": "musl-runtime", "path": "/crunch/store/musl", "identity_class": crate::offline_cargo::OFFLINE_CARGO_IDENTITY_STORE_PATH}
                ],
                "toolchain": {
                    "cargo": "/crunch/store/rust/bin/cargo",
                    "rustc": "/crunch/store/rust/bin/rustc",
                    "linker": "/crunch/store/seed/bin/x86_64-linux-musl-gcc",
                },
                "cargo_command": {
                    "program": crate::offline_cargo::OFFLINE_CARGO_COMMAND_PROGRAM,
                    "args": ["build", "--locked", "--offline", "--release", "--bin", "demo", "--target", "x86_64-unknown-linux-musl"],
                },
                "network_policy": {
                    "mode": crate::offline_cargo::OFFLINE_CARGO_NETWORK_MODE,
                    "result": crate::offline_cargo::OFFLINE_CARGO_NETWORK_RESULT,
                    "allow_undeclared_network": false,
                },
                "output": {"binary": "demo", "path": output_root.path().join("bin/demo").display().to_string()},
                "non_claims": crate::offline_cargo::offline_cargo_non_claims(),
            })
            .to_string(),
        )
        .unwrap();
        let output = BuildJsonOutput {
            name: "out".to_string(),
            path: output_root.path().display().to_string(),
            artifact_attestation: BuildJsonAttestationReference {
                logical_path: "/crunch/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-demo".to_string(),
                path: "attestation.json".to_string(),
            },
            substitution: None,
            cache_admission: None,
        };
        let outcome = build_json_outcome_with_output(output);

        let (reports, diagnostics) = build_cargo_build_evidence_reports(&[outcome]);

        assert!(reports.is_empty());
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].blocker_class, "stale-lockfile-digest");
        assert!(diagnostics[0].message.contains("Cargo.lock digest differs"));
    }

    #[test]
    fn build_json_report_omits_missing_offline_cargo_evidence_sidecar() {
        let output_root = tempfile::tempdir().unwrap();
        let output = BuildJsonOutput {
            name: "out".to_string(),
            path: output_root.path().display().to_string(),
            artifact_attestation: BuildJsonAttestationReference {
                logical_path: "/crunch/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-demo".to_string(),
                path: "attestation.json".to_string(),
            },
            substitution: None,
            cache_admission: None,
        };
        let outcome = build_json_outcome_with_output(output);

        let (reports, diagnostics) = build_cargo_build_evidence_reports(&[outcome]);

        assert!(reports.is_empty());
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn build_json_report_diagnoses_malformed_offline_cargo_evidence_sidecar() {
        let output_root = tempfile::tempdir().unwrap();
        let evidence_path = output_root.path().join(crate::offline_cargo::OFFLINE_CARGO_EVIDENCE_RELATIVE_PATH);
        std::fs::create_dir_all(evidence_path.parent().unwrap()).unwrap();
        std::fs::write(&evidence_path, "not-json").unwrap();
        let output = BuildJsonOutput {
            name: "out".to_string(),
            path: output_root.path().display().to_string(),
            artifact_attestation: BuildJsonAttestationReference {
                logical_path: "/crunch/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-demo".to_string(),
                path: "attestation.json".to_string(),
            },
            substitution: None,
            cache_admission: None,
        };
        let outcome = build_json_outcome_with_output(output);

        let (reports, diagnostics) = build_cargo_build_evidence_reports(&[outcome]);

        assert!(reports.is_empty());
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].label, "demo");
        assert_eq!(diagnostics[0].output_name, "out");
        assert_eq!(diagnostics[0].evidence_path, evidence_path.display().to_string());
        assert_eq!(diagnostics[0].blocker_class, "malformed-offline-cargo-evidence");
        assert_eq!(diagnostics[0].next_action, OFFLINE_CARGO_EVIDENCE_NEXT_ACTION);
    }

    // r[verify mantle.ast_grep_structural_rails.sidecar]
    // r[verify mantle.ast_grep_structural_rails.identity]
    #[test]
    fn build_json_report_surfaces_valid_ast_grep_structural_evidence() {
        let output_root = tempfile::tempdir().unwrap();
        let evidence_path = output_root.path().join(AST_GREP_EVIDENCE_RELATIVE_PATH);
        let fixture = include_bytes!("../tests/fixtures/ast-grep-structural-evidence/positive-scan.json");
        std::fs::create_dir_all(evidence_path.parent().unwrap()).unwrap();
        std::fs::write(&evidence_path, fixture).unwrap();
        let outcome = build_json_outcome_with_output(ast_grep_test_output(output_root.path()));

        let (reports, diagnostics) = build_ast_grep_structural_evidence_reports(&[outcome]);

        assert_eq!(reports.len(), 1);
        assert!(diagnostics.is_empty());
        assert_eq!(reports[0].evidence.command.kind, crunch_release_core::AstGrepCommandKind::Scan);
        assert_eq!(reports[0].evidence_path, evidence_path.display().to_string());
        assert_eq!(reports[0].sidecar_file_digest_blake3, blake3::hash(fixture).to_hex().to_string());
    }

    #[test]
    fn build_json_report_diagnoses_stale_ast_grep_structural_evidence() {
        let output_root = tempfile::tempdir().unwrap();
        let evidence_path = output_root.path().join(AST_GREP_EVIDENCE_RELATIVE_PATH);
        let fixture = include_bytes!("../tests/fixtures/ast-grep-structural-evidence/negative-stale-rule-bundle.json");
        std::fs::create_dir_all(evidence_path.parent().unwrap()).unwrap();
        std::fs::write(&evidence_path, fixture).unwrap();
        let outcome = build_json_outcome_with_output(ast_grep_test_output(output_root.path()));

        let (reports, diagnostics) = build_ast_grep_structural_evidence_reports(&[outcome]);

        assert!(reports.is_empty());
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].blocker_class, "invalid-ast-grep-evidence");
        assert!(diagnostics[0].message.contains("rule_bundle_digest_blake3 is stale"));
        assert_eq!(diagnostics[0].next_action, AST_GREP_EVIDENCE_NEXT_ACTION);
    }

    #[test]
    fn build_json_report_omits_absent_ast_grep_structural_evidence() {
        let output_root = tempfile::tempdir().unwrap();
        let outcome = build_json_outcome_with_output(ast_grep_test_output(output_root.path()));

        let (reports, diagnostics) = build_ast_grep_structural_evidence_reports(&[outcome]);

        assert!(reports.is_empty());
        assert!(diagnostics.is_empty());
        assert!(!output_root.path().join(AST_GREP_EVIDENCE_RELATIVE_PATH).exists());
    }

    fn ast_grep_test_output(output_root: &Path) -> BuildJsonOutput {
        BuildJsonOutput {
            name: "out".to_string(),
            path: output_root.display().to_string(),
            artifact_attestation: BuildJsonAttestationReference {
                logical_path: "/crunch/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-demo".to_string(),
                path: "attestation.json".to_string(),
            },
            substitution: None,
            cache_admission: None,
        }
    }

    fn build_json_outcome_with_output(output: BuildJsonOutput) -> BuildJsonOutcome {
        BuildJsonOutcome {
            drv_key: "drv".to_string(),
            label: "demo".to_string(),
            cached: false,
            log_file: None,
            outputs: vec![output],
        }
    }

    #[test]
    fn render_build_json_report_serializes_frontend_artifact_attestation() {
        use std::collections::BTreeMap;
        use std::collections::HashMap;

        const MIN_MAX_JOBS: u32 = 1;
        const SPEC_ID: &str = "example.activation";
        const SPEC_VERSION: &str = "1";
        const VALIDATOR_REF: &str = "mantle://blake3/spec-validator";
        const ARTIFACT_KIND: &str = "example-activation-closure";
        const ARTIFACT_REF: &str = "mantle://blake3/artifact";
        const TARGET_IDENTITY: &str = "machine:demo";
        const BUILD_ROOT: &str = "drv:demo";
        const SPEC_MATERIAL: &[u8] = br#"{"schema":"mantle-frontend-artifact-kind-allowlist-v1","allowed_kinds":["example-activation-closure"]}"#;

        let state_dir = tempfile::tempdir().unwrap();
        let output_dir = tempfile::tempdir().unwrap();
        let logs_dir = tempfile::tempdir().unwrap();
        let signing_key = crate::build_cmd::load_or_generate_signing_keypair(None, state_dir.path(), false).unwrap();
        let config = BuildConfig {
            file: output_dir.path().join("demo.ncl"),
            import_paths: Vec::new(),
            output_dir: output_dir.path().to_path_buf(),
            state_dir: state_dir.path().to_path_buf(),
            base_state_dirs: Vec::new(),
            store_dir: "/crunch/store".to_string(),
            verbose: false,
            max_jobs: MIN_MAX_JOBS,
            scheduling_policy: crunch_pipeline::SchedulingPolicy::default(),
            substituter_urls: Vec::new(),
            hermeticity_mode: crunch_pipeline::HermeticityMode::Practical,
            keypair: signing_key,
            trusted_keys: Vec::new(),
            trust_unsigned: false,
            root_retention_source: None,
            source_fetch_overrides: Vec::new(),
            remote_enabled: false,
        };
        let result = PipelineResult {
            outcomes: Vec::new(),
            failed: Vec::new(),
            fod_mismatches: Vec::new(),
            root_labels: HashMap::new(),
            hermeticity_mode: crunch_pipeline::HermeticityMode::Practical,
            hermeticity_audit_events: Vec::new(),
            build_environment_reports: Vec::new(),
            network_policy_reports: Vec::new(),
            workspace_reports: Vec::new(),
            action_result_reports: Vec::new(),
            native_dynamic_plans: Vec::new(),
            priority_decisions: Vec::new(),
        };
        let spec_hash = blake3::hash(SPEC_MATERIAL).to_hex().to_string();
        let spec = crate::frontend_artifact_spec::FrontendArtifactSpecRef {
            id: SPEC_ID.to_string(),
            version: SPEC_VERSION.to_string(),
            validator_kind: crate::frontend_artifact_spec::FRONTEND_ARTIFACT_VALIDATOR_KIND_ALLOWLIST_V1.to_string(),
            validator_ref: VALIDATOR_REF.to_string(),
            hash_algorithm: crate::frontend_artifact_spec::FRONTEND_ARTIFACT_HASH_ALGORITHM_BLAKE3.to_string(),
            hash: spec_hash.clone(),
            metadata: BTreeMap::new(),
        };
        let manifest = crate::frontend_artifact_spec::FrontendArtifactManifest {
            kind: ARTIFACT_KIND.to_string(),
            artifact_ref: ARTIFACT_REF.to_string(),
            artifact_digest: Some("blake3:artifact-digest".to_string()),
            target_identity: Some(TARGET_IDENTITY.to_string()),
            spec_id: SPEC_ID.to_string(),
            spec_version: SPEC_VERSION.to_string(),
            spec_hash,
            no_hidden_fallback: true,
            provenance: BTreeMap::from([("builder".to_string(), "mantle".to_string())]),
        };
        let admission = crate::frontend_artifact_spec::admit_frontend_artifact(
            &crate::frontend_artifact_spec::FrontendArtifactAdmissionRequest {
                spec: Some(&spec),
                manifest: &manifest,
                spec_material: SPEC_MATERIAL,
                supported_validator_kinds: &[
                    crate::frontend_artifact_spec::FRONTEND_ARTIFACT_VALIDATOR_KIND_ALLOWLIST_V1,
                ],
                build_root: BUILD_ROOT,
            },
        );
        assert!(admission.admitted, "{:#?}", admission.diagnostics);
        let attestation = admission.attestation.expect("admission attestation");

        let json_report =
            render_build_json_report_with_frontend_artifact_attestations(&config, &result, logs_dir.path(), &[], &[
                attestation,
            ])
            .unwrap();
        let json_value: serde_json::Value = serde_json::from_str(&json_report).unwrap();
        let attestation = &json_value["frontend_artifact_attestations"][0];

        assert_eq!(attestation["spec_id"], SPEC_ID);
        assert_eq!(attestation["spec_version"], SPEC_VERSION);
        assert_eq!(
            attestation["validator_kind"],
            crate::frontend_artifact_spec::FRONTEND_ARTIFACT_VALIDATOR_KIND_ALLOWLIST_V1
        );
        assert_eq!(attestation["validator_ref"], VALIDATOR_REF);
        assert_eq!(attestation["artifact_kind"], ARTIFACT_KIND);
        assert_eq!(attestation["artifact_ref"], ARTIFACT_REF);
        assert_eq!(attestation["target_identity"], TARGET_IDENTITY);
        assert_eq!(attestation["build_root"], BUILD_ROOT);
        assert_eq!(
            attestation["validation_result"],
            crate::frontend_artifact_spec::FRONTEND_ARTIFACT_VALIDATION_RESULT_ADMITTED
        );
        assert_eq!(attestation["no_hidden_fallback"], true);
    }

    #[test]
    fn render_build_json_report_serializes_full_substitution_fields_stably() {
        use std::collections::HashMap;

        let state_dir = tempfile::tempdir().unwrap();
        let output_dir = tempfile::tempdir().unwrap();
        let logs_dir = tempfile::tempdir().unwrap();
        let signing_key = crate::build_cmd::load_or_generate_signing_keypair(None, state_dir.path(), false).unwrap();
        let config = BuildConfig {
            file: output_dir.path().join("demo.ncl"),
            import_paths: Vec::new(),
            output_dir: output_dir.path().to_path_buf(),
            state_dir: state_dir.path().to_path_buf(),
            base_state_dirs: Vec::new(),
            store_dir: "/crunch/store".to_string(),
            verbose: false,
            max_jobs: 1,
            scheduling_policy: crunch_pipeline::SchedulingPolicy::default(),
            substituter_urls: Vec::new(),
            hermeticity_mode: crunch_pipeline::HermeticityMode::Impure,
            keypair: signing_key,
            trusted_keys: Vec::new(),
            trust_unsigned: false,
            root_retention_source: None,
            source_fetch_overrides: Vec::new(),
            remote_enabled: false,
        };
        let drv_path = nix_compat::store_path::StorePath::from_name_and_digest_fixed("demo.drv", [3u8; 20]).unwrap();
        let output_path = nix_compat::store_path::StorePath::from_name_and_digest_fixed("demo", [4u8; 20]).unwrap();
        let logical_path = output_path.to_absolute_path_with_prefix(&config.store_dir);
        let exported_path = output_path.to_absolute_path_with_prefix(config.output_dir.to_str().unwrap());
        let attestation_path =
            crunch_store::artifact_attestation_file_path(state_dir.path(), &config.store_dir, &output_path)
                .display()
                .to_string();
        let path_info = snix_store::path_info::PathInfo {
            store_path: output_path.clone(),
            node: snix_castore::Node::Symlink {
                target: snix_castore::SymlinkTarget::try_from("target").unwrap(),
            },
            references: vec![],
            nar_size: 1,
            nar_sha256: [0x22; 32],
            signatures: vec![],
            deriver: None,
            ca: None,
        };
        let outcome = crunch_build::BuildOutcome {
            drv_path: drv_path.clone(),
            outputs: std::collections::BTreeMap::from([("out".to_string(), path_info)]),
            substitutions: std::collections::BTreeMap::from([(
                "out".to_string(),
                crunch_store::OutputSubstitutionReport {
                    mode: crunch_store::OutputSubstitutionMode::Full,
                    transferred_bytes: 55,
                    reused_bytes: 0,
                    metadata_reused: false,
                    fallback_reason: Some("stream_application_failed".to_string()),
                },
            )]),
            cached: true,
            log: None,
        };
        let result = PipelineResult {
            outcomes: vec![outcome],
            failed: Vec::new(),
            fod_mismatches: Vec::new(),
            root_labels: HashMap::from([(drv_key_for(&config.store_dir, &drv_path), "demo".to_string())]),
            hermeticity_mode: crunch_pipeline::HermeticityMode::Impure,
            hermeticity_audit_events: vec![crunch_pipeline::HermeticityAuditEvent::new(
                crunch_pipeline::HermeticityAuditKind::ImpureModeSelected,
                "explicit --impure mode permits ambient host dependencies",
            )],
            build_environment_reports: Vec::new(),
            network_policy_reports: Vec::new(),
            workspace_reports: Vec::new(),
            action_result_reports: Vec::new(),
            native_dynamic_plans: Vec::new(),
            priority_decisions: Vec::new(),
        };

        let diagnostic_failures = vec![crate::build_log::DiagnosticPersistenceFailure::write_build_log(
            "demo",
            &logs_dir.path().join("demo.log"),
            &std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied"),
        )];
        let json_report = render_build_json_report(&config, &result, logs_dir.path(), &diagnostic_failures).unwrap();
        let json_value: serde_json::Value = serde_json::from_str(&json_report).unwrap();

        assert_eq!(
            json_value,
            json!({
                "schema": "crunch-build-report-v1",
                "file": config.file.display().to_string(),
                "output_dir": config.output_dir.display().to_string(),
                "state_dir": config.state_dir.display().to_string(),
                "store_dir": config.store_dir.clone(),
                "scheduler_policy": {
                    "schema": "mantle-scheduling-policy-v1",
                    "policy_id": "mantle-lazy-priority-v1",
                    "preference_order": ["known-graph", "resource-fit", "locality-transfer"],
                    "aged_after_epochs": crunch_build::scheduling::DEFAULT_AGED_AFTER_EPOCHS,
                    "protected_after_epochs": crunch_build::scheduling::DEFAULT_PROTECTED_AFTER_EPOCHS
                },
                "hermeticity_mode": "impure",
                "hermeticity_audit_events": [{
                    "kind": "impure-mode-selected",
                    "detail": "explicit --impure mode permits ambient host dependencies",
                }],
                "build_environment_reports": [],
                "network_policy_reports": [],
                "workspace_reports": [],
                "native_dynamic_plans": [],
                "scheduler_priority_decisions": [],
                "frontend_artifact_attestations": [],
                "ast_grep_structural_evidence": [],
                "ast_grep_structural_evidence_diagnostics": [],
                "cargo_build_evidence": [],
                "cargo_build_evidence_diagnostics": [],
                "diagnostic_persistence_failures": [{
                    "operation": "write-build-log",
                    "artifact": "build-log",
                    "label": "demo",
                    "attempted_path": logs_dir.path().join("demo.log").display().to_string(),
                    "error": "denied",
                }],
                "counts": {
                    "succeeded_total": 1,
                    "built_total": 0,
                    "cached_total": 1,
                    "failed_total": 0,
                },
                "outcomes": [{
                    "drv_key": drv_key_for(&config.store_dir, &drv_path),
                    "label": "demo",
                    "cached": true,
                    "log_file": null,
                    "outputs": [{
                        "name": "out",
                        "path": exported_path,
                        "artifact_attestation": {
                            "logical_path": logical_path.clone(),
                            "path": attestation_path,
                        },
                        "substitution": {
                            "mode": "full",
                            "transferred_bytes": 55,
                            "reused_bytes": 0,
                            "fallback_reason": "stream_application_failed",
                        }
                    }]
                }],
                "failed": [],
                "fod_mismatches": [],
            })
        );
    }
}
