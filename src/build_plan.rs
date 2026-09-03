// machine-artifact-public: build.build-plan-report
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::Path;
use std::path::PathBuf;

use clap::ValueEnum;
use crunch_action_result_core::DiscoveredActionResultCandidate;
use crunch_action_result_core::StrongReuseRequest;
use crunch_action_result_core::plan_strong_reuse;
use crunch_build::HermeticityMode;
use crunch_build::action_result::action_ref_for_derivation;
use crunch_build::action_result::candidate_admission_facts;
use crunch_build::action_result::discovery_runtime_report;
use crunch_build::action_result::policy_refs_for_derivation;
use crunch_build::action_result::trust_policy_for_action;
use crunch_build::signing;
use nix_compat::derivation::Derivation;
use nix_compat::narinfo::VerifyingKey;
use nix_compat::store_path::StorePath;
use serde::Serialize;
use snix_store::path_info::PathInfo;

use crate::build_cmd::BuildOutputMode;
use crate::build_cmd::load_configured_trusted_public_keys;
use crate::build_report::BuildJsonCacheAdmission;
use crate::errors::RunError;
use crate::operator_diagnostics::DoctorProfile;
use crate::operator_diagnostics::DoctorRequest;
use crate::operator_diagnostics::collect_doctor_report;
use crate::source_built_derivation_action_plan::EagerDerivationActionPlan;
use crate::source_built_derivation_action_plan::EagerDerivationActionPlanInput;
use crate::source_built_derivation_action_plan::EagerDerivationEntry;
use crate::source_built_derivation_action_plan::plan_eager_derivation_actions;

const PLAN_REPORT_SCHEMA: &str = "crunch-build-plan-v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum PlanAction {
    Cached,
    Substitute,
    Build,
    PreflightError,
}

impl PlanAction {
    fn as_str(self) -> &'static str {
        match self {
            Self::Cached => "cached",
            Self::Substitute => "substitute",
            Self::Build => "build",
            Self::PreflightError => "preflight-error",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct BuildPlanEntry {
    pub drv_key: String,
    pub label: String,
    pub action: PlanAction,
    pub route_plan: crate::realization_routing::RoutePlanReport,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_admission: Option<BuildJsonCacheAdmission>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_result: Option<crunch_build::ActionResultRuntimeReport>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct BuildPlanReport {
    pub schema: &'static str,
    pub file: String,
    pub output_dir: String,
    pub state_dir: String,
    pub store_dir: String,
    pub entries: Vec<BuildPlanEntry>,
}

impl BuildPlanReport {
    fn render_human(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("build plan: {}\n", self.file));
        for entry in &self.entries {
            out.push_str(&format!("- {}: {}\n", entry.label, entry.action.as_str()));
            out.push_str(&format!(
                "  route: selected={} reason={}\n",
                entry.route_plan.selected_route.as_str(),
                entry.route_plan.selected_reason_code
            ));
            if let Some(detail) = &entry.route_plan.selected_detail {
                out.push_str(&format!("  route-detail: {detail}\n"));
            }
            if !entry.route_plan.rejected_routes.is_empty() {
                let rejected = entry
                    .route_plan
                    .rejected_routes
                    .iter()
                    .map(|route| format!("{}:{}", route.route.as_str(), route.reason_code))
                    .collect::<Vec<_>>()
                    .join(", ");
                out.push_str(&format!("  rejected: {rejected}\n"));
            }
            if let Some(detail) = &entry.detail {
                out.push_str(&format!("  {}\n", detail));
            }
            if let Some(action_result) = &entry.action_result {
                out.push_str(&render_action_result_plan_human(action_result));
            }
        }
        out
    }

    fn render_json(&self) -> Result<String, RunError> {
        serde_json::to_string_pretty(self).map_err(|e| RunError::Internal(format!("serializing build plan: {e}")))
    }

    fn has_preflight_errors(&self) -> bool {
        self.entries.iter().any(|entry| entry.action == PlanAction::PreflightError)
    }
}

fn render_action_result_plan_human(report: &crunch_build::ActionResultRuntimeReport) -> String {
    let selected = report.selected_result_ref.as_deref().unwrap_or("none");
    let source = report.selected_source_class.as_deref().unwrap_or("none");
    let source_id = report.selected_source_id.as_deref().unwrap_or("none");
    let conflict = report.conflict_class.as_deref().unwrap_or("none");
    let trust_basis = if report.trust_basis.is_empty() {
        "none".to_string()
    } else {
        report.trust_basis.join(",")
    };
    let mut rendered = format!(
        "  shared-action-result: disposition={} selected={} source={} source_id={} trust={} conflict={} non_claims={}\n",
        report.disposition,
        selected,
        source,
        source_id,
        trust_basis,
        conflict,
        report.non_claims.join(",")
    );
    for decision in &report.candidate_decisions {
        if decision.admitted {
            continue;
        }
        rendered.push_str(&format!(
            "  shared-action-result-rejected: result={} diagnostics={}\n",
            decision.result_ref,
            decision.diagnostics.join(",")
        ));
    }
    debug_assert!(!report.action_ref.is_empty());
    debug_assert!(!rendered.is_empty());
    rendered
}

pub struct BuildPlanConfig<'a> {
    pub file: &'a Path,
    pub import_paths: &'a [OsString],
    pub output_dir: &'a Path,
    pub state_dir: &'a Path,
    pub base_state_dirs: &'a [PathBuf],
    pub store_dir: &'a str,
    pub substituter_urls: &'a [String],
    pub signing_key_path: Option<&'a Path>,
    pub trusted_public_keys: Option<&'a [VerifyingKey]>,
    pub trust_unsigned: bool,
    pub remote_builder: Option<&'a crate::realization_routing::RemoteBuilderPlanFacts>,
    pub source_preflight: Option<&'a crate::source_bundle::SourceOfflinePreflightReport>,
    pub output_mode: BuildOutputMode,
}

pub fn cmd_build_plan(config: BuildPlanConfig<'_>) -> Result<(), RunError> {
    let plan_document = run_build_plan(&config)?;
    let rendered = match config.output_mode {
        BuildOutputMode::Human => plan_document.render_human(),
        BuildOutputMode::Json => plan_document.render_json()?,
        BuildOutputMode::EvaluationStream => {
            return Err(RunError::Internal("--evaluation-stream cannot be used with --plan".to_string()));
        }
    };

    if plan_document.has_preflight_errors() {
        match config.output_mode {
            BuildOutputMode::Human => eprintln!("{rendered}"),
            BuildOutputMode::Json => println!("{rendered}"),
            BuildOutputMode::EvaluationStream => {
                return Err(RunError::Internal("stream mode reached build-plan rendering".to_string()));
            }
        }
        return Err(RunError::Reported(1));
    }

    println!("{rendered}");
    Ok(())
}

fn run_build_plan(config: &BuildPlanConfig<'_>) -> Result<BuildPlanReport, RunError> {
    let rt = tokio::runtime::Runtime::new().map_err(|e| RunError::Internal(format!("tokio runtime: {e}")))?;
    rt.block_on(build_plan_report(config))
}

async fn build_plan_report(config: &BuildPlanConfig<'_>) -> Result<BuildPlanReport, RunError> {
    let evaluated = evaluate_roots(config.file, config.import_paths, config.store_dir).await?;
    let roots = evaluated.roots;
    let root_count = roots.len();
    let preflight_error = validate_plan_config(config);
    let doctor_preflight = collect_doctor_report(DoctorRequest {
        profile: DoctorProfile::Build,
        store_dir: config.output_dir,
        state_dir: config.state_dir,
    });
    let plan_store = PlanStore::open(
        config.state_dir,
        config.output_dir,
        config.store_dir,
        config.base_state_dirs,
        config.substituter_urls,
    )
    .await?;
    let trust = PlanTrust::load(
        config.signing_key_path,
        config.trusted_public_keys,
        config.state_dir,
        config.store_dir,
        config.trust_unsigned,
    )?;

    let mut entries = Vec::with_capacity(roots.len());
    for root in roots {
        let action = plan_root_action(PlanRootRequest {
            plan_store: &plan_store,
            trust: &trust,
            doctor_report: &doctor_preflight,
            preflight_error: preflight_error.as_deref(),
            remote_builder: config.remote_builder,
            source_preflight: config.source_preflight,
            root: &root,
        })
        .await?;
        entries.push(action);
    }
    entries.sort_by(|left, right| left.label.cmp(&right.label).then(left.drv_key.cmp(&right.drv_key)));
    debug_assert_eq!(entries.len(), root_count);
    debug_assert!(entries.capacity() >= entries.len());

    Ok(BuildPlanReport {
        schema: PLAN_REPORT_SCHEMA,
        file: config.file.display().to_string(),
        output_dir: config.output_dir.display().to_string(),
        state_dir: config.state_dir.display().to_string(),
        store_dir: config.store_dir.to_string(),
        entries,
    })
}

fn state_dir_is_empty(state_dir: &Path) -> Result<bool, RunError> {
    let mut entries = match std::fs::read_dir(state_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(true),
        Err(error) => {
            return Err(RunError::Internal(format!(
                "reading planning state directory {}: {error}",
                state_dir.display()
            )));
        }
    };
    Ok(entries.next().is_none())
}

struct EvaluatedDerivationGraph {
    roots: Vec<PlannedRoot>,
    entries: Vec<EagerDerivationEntry>,
}

pub(crate) fn capture_eager_derivation_action_plan(
    file: &Path,
    import_paths: &[OsString],
    store_dir: &str,
    stage_id: &str,
    sandbox_shell_digest_blake3: &str,
) -> Result<EagerDerivationActionPlan, RunError> {
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|error| RunError::Internal(format!("creating eager-plan runtime: {error}")))?;
    runtime.block_on(capture_eager_derivation_action_plan_async(
        file,
        import_paths,
        store_dir,
        stage_id,
        sandbox_shell_digest_blake3,
    ))
}

async fn capture_eager_derivation_action_plan_async(
    file: &Path,
    import_paths: &[OsString],
    store_dir: &str,
    stage_id: &str,
    sandbox_shell_digest_blake3: &str,
) -> Result<EagerDerivationActionPlan, RunError> {
    let evaluated = evaluate_roots(file, import_paths, store_dir).await?;
    let plan = plan_eager_derivation_actions(EagerDerivationActionPlanInput {
        stage_id: stage_id.to_string(),
        store_dir: store_dir.to_string(),
        sandbox_shell_digest_blake3: sandbox_shell_digest_blake3.to_string(),
        entries: evaluated.entries,
    })
    .map_err(|error| RunError::Build(format!("planning eager derivation actions: {error}")))?;
    assert!(!plan.actions.is_empty());
    debug_assert_eq!(usize::try_from(plan.action_count).ok(), Some(plan.actions.len()));
    Ok(plan)
}

async fn evaluate_roots(
    file: &Path,
    import_paths: &[OsString],
    store_dir: &str,
) -> Result<EvaluatedDerivationGraph, RunError> {
    let evaluated = crunch_pipeline::evaluate_derivations_eager(crunch_pipeline::EagerDerivationEvaluationConfig {
        file,
        import_paths,
        store_dir,
        max_jobs: 1,
    })
    .await
    .map_err(|error| RunError::Build(format!("eager derivation evaluation: {error}")))?;
    let entries = evaluated
        .entries
        .into_iter()
        .map(|(drv_path, _hdm, derivation, content_addressed, dynamic_plan_outputs, _provenance)| {
            EagerDerivationEntry {
                drv_path,
                derivation,
                content_addressed,
                dynamic_plan_outputs,
            }
        })
        .collect::<Vec<_>>();
    let derivations = entries
        .iter()
        .map(|entry| (entry.drv_path.to_absolute_path_with_prefix(store_dir), entry.derivation.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut roots = Vec::with_capacity(evaluated.roots.len());
    for root in evaluated.roots {
        let root_id = root.drv_path.to_absolute_path_with_prefix(store_dir);
        let derivation = derivations
            .get(&root_id)
            .cloned()
            .ok_or_else(|| RunError::Build(format!("eager derivation graph omitted root {root_id}")))?;
        roots.push(PlannedRoot {
            label: root.label,
            drv_path: root.drv_path,
            derivation,
        });
    }
    assert!(!roots.is_empty());
    debug_assert!(entries.len() >= roots.len());
    Ok(EvaluatedDerivationGraph { roots, entries })
}

fn validate_plan_config(config: &BuildPlanConfig<'_>) -> Option<String> {
    if !config.output_dir.exists() {
        return Some(format!("output store directory {} does not exist", config.output_dir.display()));
    }
    if config.store_dir.is_empty() {
        return Some("store_dir must not be empty".to_string());
    }
    if !config.store_dir.starts_with('/') {
        return Some(format!("store_dir must be an absolute path: {}", config.store_dir));
    }
    None
}

#[derive(Clone, Copy)]
struct PlanRootRequest<'a> {
    plan_store: &'a PlanStore,
    trust: &'a PlanTrust,
    doctor_report: &'a crate::operator_diagnostics::PreflightReport,
    preflight_error: Option<&'a str>,
    remote_builder: Option<&'a crate::realization_routing::RemoteBuilderPlanFacts>,
    source_preflight: Option<&'a crate::source_bundle::SourceOfflinePreflightReport>,
    root: &'a PlannedRoot,
}

struct CacheActionRequest<'a> {
    plan_store: &'a PlanStore,
    doctor_report: &'a crate::operator_diagnostics::PreflightReport,
    remote_builder: Option<&'a crate::realization_routing::RemoteBuilderPlanFacts>,
    source_preflight: Option<&'a crate::source_bundle::SourceOfflinePreflightReport>,
    root: &'a PlannedRoot,
    cache_status: CachePlanStatus,
}

async fn plan_root_action(request: PlanRootRequest<'_>) -> Result<BuildPlanEntry, RunError> {
    let mut entry = plan_root_action_without_store_evidence(request).await?;
    let overlay = request
        .plan_store
        .output_lookup
        .overlay_report()
        .map_err(|error| RunError::Internal(format!("building route overlay evidence: {error}")))?
        .map(|report| crate::realization_routing::RouteStoreOverlayEvidence {
            plan_blake3: report.plan_blake3,
            bases: report
                .bases
                .into_iter()
                .map(|base| crate::realization_routing::RouteStoreOverlayBaseEvidence {
                    layer_index: base.declaration_index.saturating_add(1),
                    descriptor_blake3: base.descriptor_blake3,
                    generation_blake3: base.generation_blake3,
                })
                .collect(),
        });
    let selected_layers = request.plan_store.route_layer_evidence(request.root, request.trust).await?;
    entry
        .route_plan
        .bind_store_evidence(overlay, selected_layers)
        .map_err(|error| RunError::Internal(format!("binding route store evidence: {error}")))?;
    Ok(entry)
}

async fn plan_root_action_without_store_evidence(request: PlanRootRequest<'_>) -> Result<BuildPlanEntry, RunError> {
    let PlanRootRequest {
        plan_store,
        trust,
        doctor_report,
        preflight_error,
        remote_builder,
        source_preflight,
        root,
    } = request;
    if let Some(detail) = preflight_error {
        return Ok(root.remote_aware_preflight_entry(
            &plan_store.store_dir,
            Some(detail.to_string()),
            remote_builder,
            source_preflight,
        ));
    }

    let action_result = plan_store.plan_action_result(root, trust).await?;
    debug_assert!(!root.drv_path.name().is_empty());
    debug_assert!(!action_result.action_ref.is_empty());
    if action_result.conflict_class.is_some() {
        let mut entry = root.remote_aware_preflight_entry(
            &plan_store.store_dir,
            Some("conflicting-action-results: strong shared reuse rejected".to_string()),
            remote_builder,
            source_preflight,
        );
        entry.action_result = Some(action_result);
        return Ok(entry);
    }
    if action_result.selected_result_ref.is_some() {
        let mut entry = root.plan_entry(
            &plan_store.store_dir,
            PlanAction::Cached,
            Some("shared-action-result=admitted".to_string()),
            remote_builder,
            source_preflight,
        );
        entry.action_result = Some(action_result);
        return Ok(entry);
    }

    let cache_status = plan_store.classify_cache(root, trust).await?;
    let mut entry = plan_cache_or_build_action(CacheActionRequest {
        plan_store,
        doctor_report,
        remote_builder,
        source_preflight,
        root,
        cache_status,
    });
    entry.action_result = Some(action_result);
    Ok(entry)
}

fn plan_cache_or_build_action(request: CacheActionRequest<'_>) -> BuildPlanEntry {
    let CacheActionRequest {
        plan_store,
        doctor_report,
        remote_builder,
        source_preflight,
        root,
        cache_status,
    } = request;
    let decision =
        crunch_build_planning_core::select_build_action(crunch_build_planning_core::BuildActionObservationFacts {
            all_local: cache_status.all_local,
            any_remote: cache_status.any_remote,
            any_build: cache_status.any_build,
            source_bundle_present: source_preflight.is_some(),
            doctor_preflight_ok: doctor_report.ok,
        });
    match decision.action {
        crunch_build_planning_core::BuildActionClass::Cached => root.plan_entry(
            &plan_store.store_dir,
            PlanAction::Cached,
            cache_status.detail,
            remote_builder,
            source_preflight,
        ),
        crunch_build_planning_core::BuildActionClass::Substitute => root.plan_entry(
            &plan_store.store_dir,
            PlanAction::Substitute,
            cache_status.detail,
            remote_builder,
            source_preflight,
        ),
        crunch_build_planning_core::BuildActionClass::Build => root.plan_entry(
            &plan_store.store_dir,
            PlanAction::Build,
            cache_status.detail,
            remote_builder,
            source_preflight,
        ),
        crunch_build_planning_core::BuildActionClass::PreflightError => {
            let detail = failed_doctor_detail(doctor_report);
            root.remote_aware_preflight_entry(&plan_store.store_dir, Some(detail), remote_builder, source_preflight)
        }
    }
}

fn failed_doctor_detail(report: &crate::operator_diagnostics::PreflightReport) -> String {
    let failing_checks = report
        .checks
        .iter()
        .filter(|check| check.status == crate::operator_diagnostics::PreflightStatus::Failed)
        .map(|check| check.id)
        .collect::<Vec<_>>();
    let detail = format!("local build blocked by preflight checks: {}", failing_checks.join(", "));
    debug_assert!(!report.ok);
    debug_assert!(!detail.is_empty());
    detail
}

struct PlannedRoot {
    label: String,
    drv_path: StorePath<String>,
    derivation: Derivation,
}

impl PlannedRoot {
    fn plan_entry(
        &self,
        store_dir: &str,
        action: PlanAction,
        detail: Option<String>,
        remote_builder: Option<&crate::realization_routing::RemoteBuilderPlanFacts>,
        source_preflight: Option<&crate::source_bundle::SourceOfflinePreflightReport>,
    ) -> BuildPlanEntry {
        let source_facts = source_preflight.map(source_bundle_route_facts);
        let route_plan = crate::realization_routing::route_plan_for_build_action_with_remote_and_source(
            action.as_str(),
            detail.as_deref(),
            remote_builder,
            source_facts.as_ref(),
        );
        BuildPlanEntry {
            drv_key: self.drv_path.to_absolute_path_with_prefix(store_dir),
            label: self.label.clone(),
            action,
            route_plan,
            detail,
            cache_admission: None,
            action_result: None,
        }
    }

    fn remote_aware_preflight_entry(
        &self,
        store_dir: &str,
        detail: Option<String>,
        remote_builder: Option<&crate::realization_routing::RemoteBuilderPlanFacts>,
        source_preflight: Option<&crate::source_bundle::SourceOfflinePreflightReport>,
    ) -> BuildPlanEntry {
        let mut entry =
            self.plan_entry(store_dir, PlanAction::PreflightError, detail, remote_builder, source_preflight);
        if entry.route_plan.selected_route == crate::realization_routing::RouteClass::P2pRemoteBuilder {
            entry.action = PlanAction::Build;
        }
        entry
    }
}

fn source_bundle_route_facts(
    report: &crate::source_bundle::SourceOfflinePreflightReport,
) -> crate::realization_routing::SourceBundleRouteFacts {
    let is_required = report.record_count > 0;
    let is_ready = is_required && report.ready_class == crate::source_bundle::SourceReadiness::Ready;
    crate::realization_routing::SourceBundleRouteFacts {
        required: is_required,
        ready: is_ready,
        reason_code: source_readiness_reason_code(report.ready_class),
        detail: Some(format!(
            "source_state_blake3={}; records={}; non_claim={}",
            report.source_state_blake3,
            report.record_count,
            crate::source_bundle::SOURCE_BUNDLE_NON_CLAIM
        )),
    }
}

fn source_readiness_reason_code(readiness: crate::source_bundle::SourceReadiness) -> &'static str {
    match readiness {
        crate::source_bundle::SourceReadiness::Ready => "declared-source-bundle-ready",
        crate::source_bundle::SourceReadiness::Missing => "missing-source-state",
        crate::source_bundle::SourceReadiness::Stale => "stale-source-state",
        crate::source_bundle::SourceReadiness::Unsupported => "unsupported-source-adapter",
        crate::source_bundle::SourceReadiness::Untrusted => "untrusted-source-adapter",
        crate::source_bundle::SourceReadiness::NetworkRequired => "network-required-source",
        crate::source_bundle::SourceReadiness::Unpinned => "unpinned-source-state",
    }
}

fn action_result_plan_disposition(plan: &crunch_action_result_core::StrongReusePlan) -> &'static str {
    if plan.conflict_class.is_some() {
        return crunch_build::action_result::ACTION_RESULT_DISPOSITION_CONFLICT;
    }
    if plan.selected_result_ref.is_some() {
        return crunch_build::action_result::ACTION_RESULT_DISPOSITION_REUSED;
    }
    crunch_build::action_result::ACTION_RESULT_DISPOSITION_MISS
}

struct PlanStore {
    store_dir: String,
    output_lookup: crunch_store::OutputLookup,
    build_service_store: crunch_store::BuildServiceStore,
    action_results: crunch_store::ActionResultPort,
    _ephemeral_state: Option<tempfile::TempDir>,
}

impl PlanStore {
    async fn open(
        state_dir: &Path,
        output_dir: &Path,
        store_dir: &str,
        base_state_dirs: &[PathBuf],
        substituter_urls: &[String],
    ) -> Result<Self, RunError> {
        let ephemeral_state = if state_dir_is_empty(state_dir)? {
            Some(
                tempfile::Builder::new()
                    .prefix("mantle-build-plan-state-")
                    .tempdir()
                    .map_err(|error| RunError::Internal(format!("creating ephemeral planning state: {error}")))?,
            )
        } else {
            None
        };
        let planning_state_dir = ephemeral_state.as_ref().map_or(state_dir, tempfile::TempDir::path);
        let parts = crunch_store::open_planning_store_parts(crunch_store::StoreConfig {
            state_dir: planning_state_dir.to_path_buf(),
            output_dir: output_dir.to_path_buf(),
            remote_cache_urls: substituter_urls.to_vec(),
            fallback_mode: crunch_store::StoreFallbackMode::Practical,
            store_dir: store_dir.to_string(),
            base_state_dirs: base_state_dirs.to_vec(),
        })
        .await
        .map_err(|error| RunError::Internal(format!("opening composed planning store: {error}")))?;
        Ok(Self {
            store_dir: store_dir.to_string(),
            output_lookup: parts.output_lookup,
            build_service_store: parts.build_service_store,
            action_results: parts.action_results,
            _ephemeral_state: ephemeral_state,
        })
    }

    async fn route_layer_evidence(
        &self,
        root: &PlannedRoot,
        trust: &PlanTrust,
    ) -> Result<Vec<crate::realization_routing::RouteStoreLayerEvidence>, RunError> {
        let overlay = self
            .output_lookup
            .overlay_report()
            .map_err(|error| RunError::Internal(format!("reading route overlay evidence: {error}")))?;
        let mut paths = BTreeMap::new();
        for input_source in &root.derivation.input_sources {
            paths.insert(input_source.to_string(), input_source.clone());
        }
        let drv_abs = root.drv_path.to_absolute_path_with_prefix(&self.store_dir);
        for (output_name, output) in &root.derivation.outputs {
            let ca_mapping = self
                .output_lookup
                .resolve_ca_mapping(&drv_abs, output_name)
                .map_err(|error| RunError::Internal(format!("resolving route CA mapping: {error}")))?;
            if let Some(output_path) = resolve_output_path(ResolveOutputRequest {
                output,
                mapped_ca_path: ca_mapping.as_deref(),
                store_dir: &self.store_dir,
            })? {
                paths.insert(output_path.to_string(), output_path);
            }
        }

        let mut evidence = Vec::new();
        for (store_path, path) in paths {
            let Some(layered) = self
                .output_lookup
                .find_with_layer(&path)
                .await
                .map_err(|error| RunError::Internal(format!("reading route layer for {path}: {error}")))?
            else {
                continue;
            };
            if !self
                .build_service_store
                .has_complete_content(&layered.value)
                .await
                .map_err(|error| RunError::Internal(format!("checking route content for {path}: {error}")))?
            {
                continue;
            }
            if !self.output_lookup.is_overlay_composed() && !trust.pathinfo_is_accepted(&layered.value) {
                continue;
            }
            let base = overlay.as_ref().and_then(|report| {
                let selected_index = layered.layer.service_index();
                report.bases.iter().find(|base| {
                    u32::try_from(base.declaration_index).is_ok_and(|index| index.saturating_add(1) == selected_index)
                })
            });
            evidence.push(crate::realization_routing::RouteStoreLayerEvidence {
                store_path,
                selected_layer: layered.layer.to_string(),
                base_descriptor_blake3: base.map(|base| base.descriptor_blake3.clone()),
                base_generation_blake3: base.map(|base| base.generation_blake3.clone()),
            });
        }
        Ok(evidence)
    }

    async fn plan_action_result(
        &self,
        root: &PlannedRoot,
        trust: &PlanTrust,
    ) -> Result<crunch_build::ActionResultRuntimeReport, RunError> {
        let action_ref = action_ref_for_derivation(&root.derivation, &self.store_dir);
        let discovery = self.action_results.discover(&action_ref).await;
        let candidate_count_max = discovery
            .lookups
            .iter()
            .try_fold(0_usize, |count, lookup| count.checked_add(lookup.records.len()))
            .ok_or_else(|| RunError::Internal("shared action-result candidate count overflowed usize".to_string()))?;
        let mut candidates = Vec::with_capacity(candidate_count_max);
        let mut sources = BTreeMap::new();
        for lookup in discovery.lookups {
            for signed_record in lookup.records {
                let result_ref = signed_record.record.result_ref.clone();
                let outputs = self.probe_local_action_outputs(&signed_record.record).await?;
                let facts = candidate_admission_facts(
                    lookup.source_id.clone(),
                    lookup.source_class.clone(),
                    &signed_record,
                    &root.derivation,
                    outputs.as_ref(),
                    &self.store_dir,
                    HermeticityMode::Practical,
                    &trust.trusted_keys,
                );
                sources.entry(result_ref).or_insert((lookup.source_id.clone(), lookup.source_class.clone()));
                candidates.push(DiscoveredActionResultCandidate { signed_record, facts });
            }
        }
        let policy_refs = policy_refs_for_derivation(&root.derivation, &self.store_dir, HermeticityMode::Practical)
            .map_err(RunError::Build)?;
        let plan = plan_strong_reuse(
            StrongReuseRequest {
                action_ref: action_ref.clone(),
                output_names: root.derivation.outputs.keys().cloned().collect(),
                policy: trust_policy_for_action(&policy_refs, &trust.trusted_keys),
            },
            candidates,
        )
        .map_err(RunError::Build)?;
        let selected_source = plan.selected_result_ref.as_ref().and_then(|result_ref| sources.get(result_ref).cloned());
        let disposition = action_result_plan_disposition(&plan);
        Ok(discovery_runtime_report(
            action_ref,
            disposition,
            plan,
            selected_source,
            None,
            discovery.diagnostics,
        ))
    }

    async fn probe_local_action_outputs(
        &self,
        record: &crunch_action_result_core::ActionResultRecord,
    ) -> Result<Option<BTreeMap<String, PathInfo>>, RunError> {
        let mut outputs = BTreeMap::new();
        for output in &record.outputs {
            let store_path = StorePath::from_absolute_path_with_prefix(output.store_path.as_bytes(), &self.store_dir)
                .map_err(|_| {
                RunError::Build(format!("invalid shared action-result path: {}", output.store_path))
            })?;
            let Some(path_info) = self
                .output_lookup
                .find(&store_path)
                .await
                .map_err(|error| RunError::Internal(format!("shared action-result PathInfo probe: {error}")))?
            else {
                return Ok(None);
            };
            if path_info.store_path != store_path {
                return Ok(None);
            }
            if !self
                .build_service_store
                .has_complete_content(&path_info)
                .await
                .map_err(|error| RunError::Internal(format!("shared action-result object probe: {error}")))?
            {
                return Ok(None);
            }
            if outputs.insert(output.name.clone(), path_info).is_some() {
                return Ok(None);
            }
        }
        Ok(Some(outputs))
    }

    async fn classify_cache(&self, root: &PlannedRoot, trust: &PlanTrust) -> Result<CachePlanStatus, RunError> {
        let mut is_all_local = true;
        let mut is_any_remote = false;
        let mut is_any_build = false;
        let mut detail_parts = Vec::with_capacity(root.derivation.outputs.len());
        let is_fod = root.derivation.outputs.values().any(|output| output.ca_hash.is_some());
        let drv_abs = root.drv_path.to_absolute_path_with_prefix(&self.store_dir);

        for (output_name, output) in &root.derivation.outputs {
            let ca_mapping = self
                .output_lookup
                .resolve_ca_mapping(&drv_abs, output_name)
                .map_err(|error| RunError::Internal(format!("resolving composed CA mapping: {error}")))?;
            let Some(output_path) = resolve_output_path(ResolveOutputRequest {
                output,
                mapped_ca_path: ca_mapping.as_deref(),
                store_dir: &self.store_dir,
            })?
            else {
                is_all_local = false;
                is_any_build = true;
                detail_parts.push(format!("{output_name}=build"));
                continue;
            };

            let local_status = self.local_output_status(&output_path, trust).await?;
            match local_status {
                OutputPlan::Local => {
                    detail_parts.push(format!("{output_name}=cached"));
                }
                OutputPlan::Build(reason) => {
                    is_all_local = false;
                    is_any_build = true;
                    detail_parts.push(format!("{output_name}=build ({reason})"));
                }
                OutputPlan::Missing => {
                    is_all_local = false;
                    if !is_fod && self.remote_output_available(&output_path).await? {
                        is_any_remote = true;
                        detail_parts.push(format!("{output_name}=substitute"));
                    } else {
                        is_any_build = true;
                        detail_parts.push(format!("{output_name}=build"));
                    }
                }
            }
        }

        let detail = if detail_parts.is_empty() {
            None
        } else {
            Some(detail_parts.join(", "))
        };
        Ok(CachePlanStatus {
            all_local: is_all_local,
            any_remote: is_any_remote,
            any_build: is_any_build,
            detail,
        })
    }

    async fn local_output_status(
        &self,
        output_path: &StorePath<String>,
        trust: &PlanTrust,
    ) -> Result<OutputPlan, RunError> {
        let Some(layered_path_info) = self
            .output_lookup
            .find_with_layer(output_path)
            .await
            .map_err(|e| RunError::Internal(format!("PathInfo lookup for {output_path}: {e}")))?
        else {
            return Ok(OutputPlan::Missing);
        };

        let path_info = layered_path_info.value;
        if !self
            .build_service_store
            .has_complete_content(&path_info)
            .await
            .map_err(|error| RunError::Internal(format!("composed castore probe: {error}")))?
        {
            return Ok(OutputPlan::Build("castore content missing".to_string()));
        }
        if !self.output_lookup.is_overlay_composed() && !trust.pathinfo_is_accepted(&path_info) {
            return Ok(OutputPlan::Build("untrusted PathInfo".to_string()));
        }
        Ok(OutputPlan::Local)
    }

    async fn remote_output_available(&self, output_path: &StorePath<String>) -> Result<bool, RunError> {
        self.output_lookup
            .find_remote(output_path)
            .await
            .map(|found| found.is_some())
            .map_err(|e| RunError::Internal(format!("remote plan probe for {output_path}: {e}")))
    }
}

struct CachePlanStatus {
    all_local: bool,
    any_remote: bool,
    any_build: bool,
    detail: Option<String>,
}

enum OutputPlan {
    Local,
    Missing,
    Build(String),
}

struct PlanTrust {
    trust_unsigned: bool,
    trusted_keys: Vec<VerifyingKey>,
    store_dir: String,
}

impl PlanTrust {
    fn load(
        signing_key_path: Option<&Path>,
        explicit_trusted_keys: Option<&[VerifyingKey]>,
        state_dir: &Path,
        store_dir: &str,
        trust_unsigned: bool,
    ) -> Result<Self, RunError> {
        let mut trusted_keys = Vec::new();
        if let Some(local_key) = load_local_verifying_key(signing_key_path, state_dir)? {
            trusted_keys.push(local_key);
        }
        if let Some(configured) = load_configured_trusted_public_keys(explicit_trusted_keys, state_dir)? {
            trusted_keys.extend(configured);
        }
        Ok(Self {
            trust_unsigned,
            trusted_keys,
            store_dir: store_dir.to_string(),
        })
    }

    fn pathinfo_is_accepted(&self, path_info: &PathInfo) -> bool {
        if self.trust_unsigned {
            return true;
        }
        signing::verify_pathinfo_signatures_with_store_dir(path_info, &self.trusted_keys, &self.store_dir).is_trusted()
    }
}

fn load_local_verifying_key(
    signing_key_path: Option<&Path>,
    state_dir: &Path,
) -> Result<Option<VerifyingKey>, RunError> {
    let key_path = match signing_key_path {
        Some(path) => path.to_path_buf(),
        None => config_dir_or(state_dir).join("signing-key"),
    };
    if !key_path.exists() {
        return Ok(None);
    }
    let contents = std::fs::read_to_string(&key_path)
        .map_err(|e| RunError::Internal(format!("reading signing key {}: {e}", key_path.display())))?;
    let keypair = signing::load_keypair(&contents)
        .map_err(|e| RunError::Internal(format!("parsing signing key {}: {e}", key_path.display())))?;
    Ok(Some(keypair.verifying_key))
}

fn config_dir_or(state_dir: &Path) -> PathBuf {
    std::env::var("CRUNCH_CONFIG_DIR").map(PathBuf::from).unwrap_or_else(|_| state_dir.to_path_buf())
}

struct ResolveOutputRequest<'a> {
    output: &'a nix_compat::derivation::Output,
    mapped_ca_path: Option<&'a str>,
    store_dir: &'a str,
}

fn resolve_output_path(request: ResolveOutputRequest<'_>) -> Result<Option<StorePath<String>>, RunError> {
    if let Some(path) = &request.output.path {
        return Ok(Some(path.clone()));
    }
    let Some(mapped) = request.mapped_ca_path else {
        return Ok(None);
    };
    let path = StorePath::from_absolute_path_with_prefix(mapped.as_bytes(), request.store_dir)
        .map_err(|_| RunError::Internal(format!("invalid CA mapping path: {mapped}")))?;
    Ok(Some(path))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    const TEST_SANDBOX_SHELL_DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const DEPENDENCY_CHAIN_ACTION_COUNT_MIN: u32 = 2;
    const PRESERVED_NATIVE_REPORT_ROOT: &str = ".cairn/archive/2026-08-31-prove-source-built-mantle-fixed-point/evidence/action-trust-architecture-search-2026-08-23/preserved-native-reports";
    const PRESERVED_NATIVE_REPORTS: &[(&str, &str)] = &[
        ("full-source-native-provider", "bootstrap/seed-full-toolchain.ncl"),
        ("make", "bootstrap/make-4.4.1-gcc10.ncl"),
        ("linux-headers", "bootstrap/linux-headers-6.6-gcc10.ncl"),
        ("busybox", "bootstrap/busybox-1.37.0-gcc10.ncl"),
        ("cmake", "bootstrap/cmake-3.31.8-gcc10.ncl"),
        ("python", "bootstrap/python-3.13.5-gcc10.ncl"),
        ("perl", "bootstrap/perl-5.10.1-gcc10.ncl"),
    ];

    fn report_with_rejection() -> crunch_build::ActionResultRuntimeReport {
        crunch_build::ActionResultRuntimeReport {
            schema: "mantle-action-result-runtime-report-v1".to_string(),
            phase: "discovery".to_string(),
            action_ref: "action-b3:demo".to_string(),
            disposition: "conflict".to_string(),
            selected_result_ref: None,
            selected_source_id: None,
            selected_source_class: None,
            trust_basis: Vec::new(),
            conflict_class: Some("conflicting-action-results".to_string()),
            candidate_decisions: vec![crunch_action_result_core::CandidateDecision {
                result_ref: "result-b3:rejected".to_string(),
                source_id: "local-action-results".to_string(),
                source_class: "local".to_string(),
                admitted: false,
                diagnostics: vec!["record-signature-missing".to_string()],
                trust_basis: Vec::new(),
                output_set_digest_blake3: None,
            }],
            publication_result_refs: Vec::new(),
            transfer: None,
            diagnostics: Vec::new(),
            non_claims: vec!["index-presence-is-not-output-trust".to_string()],
        }
    }

    #[test]
    fn eager_capture_includes_transitive_dependency_actions() {
        let repository = Path::new(env!("CARGO_MANIFEST_DIR"));
        let file = repository.join("examples/dependency-chain.ncl");
        let import_paths = [repository.join("lib").into_os_string()];

        let plan = capture_eager_derivation_action_plan(
            &file,
            &import_paths,
            "/mantle/store",
            "native-provider",
            TEST_SANDBOX_SHELL_DIGEST,
        )
        .unwrap();

        assert!(plan.action_count >= DEPENDENCY_CHAIN_ACTION_COUNT_MIN);
        assert!(plan.actions.iter().any(|action| !action.producer_action_ids.is_empty()));
        assert!(plan.actions.iter().any(|action| {
            matches!(
                action.executable,
                crate::source_built_derivation_action_plan::DerivationExecutableAuthority::FixedSandboxShell { .. }
            )
        }));
        assert!(plan.blockers.is_empty());
    }

    #[test]
    fn full_source_native_plan_covers_preserved_worker_decisions() {
        let repository = Path::new(env!("CARGO_MANIFEST_DIR"));
        let file = repository.join("bootstrap/seed-full-toolchain.ncl");
        let import_paths = [
            repository.as_os_str().to_owned(),
            repository.join("lib").into_os_string(),
            repository.join("bootstrap").into_os_string(),
        ];

        let plan = capture_eager_derivation_action_plan(
            &file,
            &import_paths,
            "/mantle/store",
            "full-source-native-provider",
            TEST_SANDBOX_SHELL_DIGEST,
        )
        .unwrap();
        let report: serde_json::Value = serde_json::from_str(include_str!(
            "../.cairn/archive/2026-08-31-prove-source-built-mantle-fixed-point/evidence/v48-promoted-fixed-point-success-2026-08-23/provider-checkpoint-origin/native-provider.json"
        ))
        .unwrap();
        let observed = report["scheduler_priority_decisions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|decision| decision["selected_goal_key_blake3"].as_str().unwrap())
            .collect::<BTreeSet<_>>();
        let planned =
            plan.actions.iter().map(|action| action.observed_goal_key_blake3.as_str()).collect::<BTreeSet<_>>();
        let planned_root = plan
            .actions
            .iter()
            .find(|action| action.outputs.iter().any(|output| output.identity.ends_with("-full-source-seed-toolchain")))
            .unwrap();
        let observed_root = report["outcomes"][0]["drv_key"].as_str().unwrap();

        assert_eq!(planned.len(), observed.len());
        assert_eq!(planned_root.action_id, observed_root);
        assert_eq!(planned, observed);
        assert_eq!(planned.len(), plan.actions.len());
        assert_eq!(u32::try_from(observed.len()).unwrap(), plan.action_count);
        assert!(plan.blockers.is_empty());
    }

    #[test]
    #[ignore = "replays preserved full native build reports"]
    fn full_native_composite_plan_reconciles_all_preserved_worker_events() {
        let repository = Path::new(env!("CARGO_MANIFEST_DIR"));
        let import_paths = [
            repository.as_os_str().to_owned(),
            repository.join("lib").into_os_string(),
            repository.join("bootstrap").into_os_string(),
        ];
        let mut plans = Vec::new();
        let mut observed = Vec::new();
        for (label, relative_ncl) in PRESERVED_NATIVE_REPORTS {
            plans.push(
                capture_eager_derivation_action_plan(
                    &repository.join(relative_ncl),
                    &import_paths,
                    "/mantle/store",
                    &format!("full-source-native-provider:{label}"),
                    TEST_SANDBOX_SHELL_DIGEST,
                )
                .unwrap(),
            );
            let report: serde_json::Value = serde_json::from_slice(
                &std::fs::read(repository.join(PRESERVED_NATIVE_REPORT_ROOT).join(format!("{label}.json"))).unwrap(),
            )
            .unwrap();
            observed.extend(
                report["scheduler_priority_decisions"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|decision| decision["selected_goal_key_blake3"].as_str().unwrap().to_string()),
            );
        }

        let composite = crate::source_built_derivation_action_plan::compose_eager_derivation_action_plans(
            "full-source-native-provider",
            &plans,
        )
        .unwrap();
        let reconciliation =
            crate::source_built_derivation_action_plan::reconcile_eager_derivation_actions(&composite, &observed)
                .unwrap();
        assert!(reconciliation.is_complete(), "{reconciliation:#?}");
        crate::source_built_derivation_action_plan::require_complete_eager_derivation_reconciliation(&reconciliation)
            .unwrap();
        let planned_event_count_max = composite
            .actions
            .iter()
            .try_fold(0_u32, |total, action| total.checked_add(action.event_count_max))
            .unwrap();
        assert_eq!(reconciliation.matched_action_count, composite.action_count);
        assert_eq!(reconciliation.observed_event_count, planned_event_count_max);
        assert_eq!(reconciliation.matched_event_count, planned_event_count_max);
        assert!(reconciliation.blockers.is_empty());
    }

    #[test]
    fn planning_state_detects_missing_empty_and_populated_directories() {
        let root = tempfile::tempdir().unwrap();
        let missing = root.path().join("missing");
        let empty = root.path().join("empty");
        std::fs::create_dir(&empty).unwrap();

        assert!(state_dir_is_empty(&missing).unwrap());
        assert!(state_dir_is_empty(&empty).unwrap());

        std::fs::write(empty.join("pathinfo.redb"), b"fixture").unwrap();
        assert!(!state_dir_is_empty(&empty).unwrap());
    }

    #[test]
    fn action_result_plan_human_surfaces_conflicts_and_rejections() {
        let rendered = render_action_result_plan_human(&report_with_rejection());

        assert!(rendered.contains("disposition=conflict"));
        assert!(rendered.contains("conflict=conflicting-action-results"));
        assert!(rendered.contains("result=result-b3:rejected"));
        assert!(rendered.contains("record-signature-missing"));
        assert!(rendered.contains("index-presence-is-not-output-trust"));
        assert!(!rendered.contains("selected=result-b3:rejected"));
    }
}
