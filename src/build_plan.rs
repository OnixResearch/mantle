// machine-artifact-public: build.build-plan-report
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::num::NonZeroUsize;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

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
use crunch_glue::ConversionCache;
use crunch_glue::CrunchDerivation;
use crunch_store::ActionResultStoreSet;
use crunch_store::CaMappings;
use crunch_store::HttpActionResultStore;
use crunch_store::LocalActionResultStore;
use nix_compat::derivation::Derivation;
use nix_compat::narinfo::VerifyingKey;
use nix_compat::store_path::StorePath;
use serde::Serialize;
use snix_castore::Node;
use snix_castore::blobservice::BlobService;
use snix_castore::blobservice::MemoryBlobService;
use snix_castore::blobservice::ObjectStoreBlobService;
use snix_castore::directoryservice::DirectoryService;
use snix_castore::directoryservice::RedbDirectoryService;
use snix_castore::directoryservice::RedbDirectoryServiceConfig;
use snix_store::path_info::PathInfo;
use snix_store::pathinfoservice::LruPathInfoService;
use snix_store::pathinfoservice::NixHTTPPathInfoService;
use snix_store::pathinfoservice::NixHTTPPathInfoServiceConfig;
use snix_store::pathinfoservice::PathInfoService;
use snix_store::pathinfoservice::RedbPathInfoService;
use snix_store::pathinfoservice::RedbPathInfoServiceConfig;

use crate::build_cmd::BuildOutputMode;
use crate::build_cmd::load_configured_trusted_public_keys;
use crate::build_report::BuildJsonCacheAdmission;
use crate::errors::RunError;
use crate::operator_diagnostics::DoctorProfile;
use crate::operator_diagnostics::DoctorRequest;
use crate::operator_diagnostics::collect_doctor_report;

const PLAN_REPORT_SCHEMA: &str = "crunch-build-plan-v1";
const MAX_LABELED_EVAL_ERROR_DEPTH: u32 = 64;
const EMPTY_PATHINFO_ENTRY_COUNT: usize = 32;

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
    };

    if plan_document.has_preflight_errors() {
        match config.output_mode {
            BuildOutputMode::Human => eprintln!("{rendered}"),
            BuildOutputMode::Json => println!("{rendered}"),
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
    let roots = evaluate_roots(config.file, config.import_paths, config.store_dir)?;
    let root_count = roots.len();
    let preflight_error = validate_plan_config(config);
    let doctor_preflight = collect_doctor_report(DoctorRequest {
        profile: DoctorProfile::Build,
        store_dir: config.output_dir,
        state_dir: config.state_dir,
    });
    let plan_store = PlanStore::open(config.state_dir, config.store_dir, config.substituter_urls).await?;
    let trust =
        PlanTrust::load(config.signing_key_path, config.trusted_public_keys, config.state_dir, config.trust_unsigned)?;

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

fn eval_error_is_build(err: &crunch_eval::Error) -> bool {
    let mut current = err;
    for _depth in 0..MAX_LABELED_EVAL_ERROR_DEPTH {
        match current {
            crunch_eval::Error::Eval(_) | crunch_eval::Error::Io(_) => return false,
            crunch_eval::Error::Boundary(_) | crunch_eval::Error::Serde(_) => return true,
            crunch_eval::Error::Labeled { source, .. } => current = source,
        }
    }
    true
}

fn evaluate_roots(file: &Path, import_paths: &[OsString], store_dir: &str) -> Result<Vec<PlannedRoot>, RunError> {
    let mut session = crunch_eval::session::EvaluationSession::open_file(file, import_paths)
        .map_err(|e| RunError::Eval(format!("{e}")))?;
    let derivations = session.force_all_roots::<CrunchDerivation>().map_err(|e| {
        if eval_error_is_build(&e) {
            return RunError::Build(format!("{e}"));
        }
        RunError::Eval(format!("{e}"))
    })?;
    let derivation_count = derivations.len();
    let mut cache = ConversionCache::new(store_dir);
    let mut roots = Vec::with_capacity(derivation_count);
    for (label, drv) in derivations {
        let (drv_path, derivation) =
            crunch_glue::convert(&drv, &mut cache).map_err(|e| RunError::Build(format!("{label}: {e}")))?;
        roots.push(PlannedRoot {
            label,
            drv_path,
            derivation,
        });
    }
    debug_assert_eq!(roots.len(), derivation_count);
    debug_assert!(roots.capacity() >= roots.len());
    Ok(roots)
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
    if cache_status.all_local {
        debug_assert!(!cache_status.any_build);
        debug_assert!(!cache_status.any_remote);
        return root.plan_entry(
            &plan_store.store_dir,
            PlanAction::Cached,
            cache_status.detail,
            remote_builder,
            source_preflight,
        );
    }
    if cache_status.any_remote && !cache_status.any_build && source_preflight.is_none() {
        return root.plan_entry(
            &plan_store.store_dir,
            PlanAction::Substitute,
            cache_status.detail,
            remote_builder,
            source_preflight,
        );
    }
    if doctor_report.ok {
        return root.plan_entry(
            &plan_store.store_dir,
            PlanAction::Build,
            cache_status.detail,
            remote_builder,
            source_preflight,
        );
    }
    let failing_checks = doctor_report
        .checks
        .iter()
        .filter(|check| check.status == crate::operator_diagnostics::PreflightStatus::Failed)
        .map(|check| check.id)
        .collect::<Vec<_>>();
    let detail = format!("local build blocked by preflight checks: {}", failing_checks.join(", "));
    root.remote_aware_preflight_entry(&plan_store.store_dir, Some(detail), remote_builder, source_preflight)
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

fn open_action_result_stores(state_dir: &Path, substituter_urls: &[String]) -> Result<ActionResultStoreSet, RunError> {
    let policy = crunch_store::action_result_runtime_policy();
    let mut stores = ActionResultStoreSet::new(substituter_urls.is_empty());
    if policy.sources.local_enabled {
        stores.add_local(Box::new(LocalActionResultStore::new(state_dir)));
    }
    if !policy.sources.http_enabled {
        return Ok(stores);
    }
    for url in substituter_urls {
        let parsed = url::Url::parse(url)
            .map_err(|error| RunError::Build(format!("invalid shared action-result source URL: {error}")))?;
        let store = HttpActionResultStore::with_default_timeout(parsed).map_err(RunError::Build)?;
        stores.add_remote(Box::new(store));
    }
    Ok(stores)
}

struct PlanStore {
    store_dir: String,
    local_pathinfo: Arc<dyn PathInfoService>,
    remote_pathinfo: Option<Arc<dyn PathInfoService>>,
    blob_service: Arc<dyn BlobService>,
    directory_service: Arc<dyn DirectoryService>,
    ca_mappings: CaMappings,
    action_result_stores: ActionResultStoreSet,
}

impl PlanStore {
    async fn open(state_dir: &Path, store_dir: &str, substituter_urls: &[String]) -> Result<Self, RunError> {
        let blob_service = open_blob_service(state_dir)?;
        let directory_service = open_directory_service(state_dir).await?;
        let local_pathinfo = open_pathinfo_service(state_dir).await?;
        let remote_pathinfo = match substituter_urls.first() {
            Some(url) => Some(open_remote_pathinfo(url, blob_service.clone(), directory_service.clone())?),
            None => None,
        };
        Ok(Self {
            store_dir: store_dir.to_string(),
            local_pathinfo,
            remote_pathinfo,
            blob_service,
            directory_service,
            ca_mappings: CaMappings::load(state_dir),
            action_result_stores: open_action_result_stores(state_dir, substituter_urls)?,
        })
    }

    async fn plan_action_result(
        &self,
        root: &PlannedRoot,
        trust: &PlanTrust,
    ) -> Result<crunch_build::ActionResultRuntimeReport, RunError> {
        let action_ref = action_ref_for_derivation(&root.derivation, &self.store_dir);
        let discovery = self.action_result_stores.discover(&action_ref).await;
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
                .local_pathinfo
                .get(*store_path.digest())
                .await
                .map_err(|error| RunError::Internal(format!("shared action-result PathInfo probe: {error}")))?
            else {
                return Ok(None);
            };
            if path_info.store_path != store_path {
                return Ok(None);
            }
            if !castore_has_content(&path_info, self.blob_service.as_ref(), self.directory_service.as_ref()).await? {
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
            let Some(output_path) = resolve_output_path(ResolveOutputRequest {
                drv_abs: &drv_abs,
                output_name,
                output,
                ca_mappings: &self.ca_mappings,
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
        let Some(path_info) = self
            .local_pathinfo
            .get(*output_path.digest())
            .await
            .map_err(|e| RunError::Internal(format!("PathInfo lookup for {output_path}: {e}")))?
        else {
            return Ok(OutputPlan::Missing);
        };

        if path_info.store_path != *output_path {
            return Ok(OutputPlan::Build("digest collision".to_string()));
        }
        if !castore_has_content(&path_info, self.blob_service.as_ref(), self.directory_service.as_ref()).await? {
            return Ok(OutputPlan::Build("castore content missing".to_string()));
        }
        if !trust.pathinfo_is_accepted(&path_info) {
            return Ok(OutputPlan::Build("untrusted PathInfo".to_string()));
        }
        Ok(OutputPlan::Local)
    }

    async fn remote_output_available(&self, output_path: &StorePath<String>) -> Result<bool, RunError> {
        let Some(remote) = &self.remote_pathinfo else {
            return Ok(false);
        };
        remote
            .get_references(*output_path.digest())
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
}

impl PlanTrust {
    fn load(
        signing_key_path: Option<&Path>,
        explicit_trusted_keys: Option<&[VerifyingKey]>,
        state_dir: &Path,
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
        })
    }

    fn pathinfo_is_accepted(&self, path_info: &PathInfo) -> bool {
        if self.trust_unsigned {
            return true;
        }
        signing::verify_pathinfo_signatures(path_info, &self.trusted_keys).is_trusted()
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
    drv_abs: &'a str,
    output_name: &'a str,
    output: &'a nix_compat::derivation::Output,
    ca_mappings: &'a CaMappings,
    store_dir: &'a str,
}

fn resolve_output_path(request: ResolveOutputRequest<'_>) -> Result<Option<StorePath<String>>, RunError> {
    if let Some(path) = &request.output.path {
        return Ok(Some(path.clone()));
    }
    let Some(mapped) = request.ca_mappings.get(request.drv_abs, request.output_name) else {
        return Ok(None);
    };
    let path = StorePath::from_absolute_path_with_prefix(mapped.as_bytes(), request.store_dir)
        .map_err(|_| RunError::Internal(format!("invalid CA mapping path: {mapped}")))?;
    Ok(Some(path))
}

fn open_blob_service(state_dir: &Path) -> Result<Arc<dyn BlobService>, RunError> {
    let blob_dir = state_dir.join("blobs");
    if !blob_dir.is_dir() {
        return Ok(Arc::new(empty_memory_blob_service()) as Arc<dyn BlobService>);
    }
    let svc = ObjectStoreBlobService::new_local(&blob_dir)
        .map_err(|e| RunError::Internal(format!("opening blob dir {}: {e}", blob_dir.display())))?;
    Ok(Arc::new(svc) as Arc<dyn BlobService>)
}

#[allow(
    tigerstyle::explicit_defaults,
    reason = "MemoryBlobService has private fields and exposes Default as its only direct constructor"
)]
fn empty_memory_blob_service() -> MemoryBlobService {
    MemoryBlobService::default()
}

async fn open_directory_service(state_dir: &Path) -> Result<Arc<dyn DirectoryService>, RunError> {
    debug_assert!(!state_dir.as_os_str().is_empty());
    let path = state_dir.join("directories.redb");
    debug_assert!(path.starts_with(state_dir));
    if !path.is_file() {
        let svc = RedbDirectoryService::new_temporary("plan-empty-directory".to_string(), RedbDirectoryServiceConfig {
            path: None,
            cache_size: None,
            read_only: false,
        })
        .map_err(|e| RunError::Internal(format!("creating empty directory service: {e}")))?;
        return Ok(Arc::new(svc) as Arc<dyn DirectoryService>);
    }

    let svc = RedbDirectoryService::new("plan-directory".to_string(), RedbDirectoryServiceConfig {
        path: Some(path.clone()),
        cache_size: None,
        read_only: true,
    })
    .await
    .map_err(|e| RunError::Internal(format!("opening {}: {e}", path.display())))?;
    Ok(Arc::new(svc) as Arc<dyn DirectoryService>)
}

async fn open_pathinfo_service(state_dir: &Path) -> Result<Arc<dyn PathInfoService>, RunError> {
    let path = state_dir.join("pathinfo.redb");
    if !path.is_file() {
        let pathinfo_entry_count_max = NonZeroUsize::new(EMPTY_PATHINFO_ENTRY_COUNT)
            .ok_or_else(|| RunError::Internal("empty plan PathInfo capacity must be non-zero".to_string()))?;
        let svc = LruPathInfoService::with_capacity("plan-empty-pathinfo".to_string(), pathinfo_entry_count_max);
        return Ok(Arc::new(svc) as Arc<dyn PathInfoService>);
    }

    let svc = RedbPathInfoService::new("plan-pathinfo".to_string(), RedbPathInfoServiceConfig {
        path: Some(path.clone()),
        cache_size: None,
        read_only: true,
    })
    .await
    .map_err(|e| RunError::Internal(format!("opening {}: {e}", path.display())))?;
    Ok(Arc::new(svc) as Arc<dyn PathInfoService>)
}

fn open_remote_pathinfo(
    url_str: &str,
    blob_service: Arc<dyn BlobService>,
    directory_service: Arc<dyn DirectoryService>,
) -> Result<Arc<dyn PathInfoService>, RunError> {
    let nix_url: url::Url = format!("nix+{url_str}")
        .parse()
        .map_err(|e| RunError::Internal(format!("invalid substituter URL '{url_str}': {e}")))?;
    let config: NixHTTPPathInfoServiceConfig = nix_url
        .try_into()
        .map_err(|e| RunError::Internal(format!("remote cache config for '{url_str}': {e}")))?;
    let svc = NixHTTPPathInfoService::try_build("plan-remote".to_string(), config, blob_service, directory_service)
        .map_err(|e| RunError::Internal(format!("building remote cache client: {e}")))?;
    Ok(Arc::new(svc) as Arc<dyn PathInfoService>)
}

async fn castore_has_content(
    path_info: &PathInfo,
    blob_service: &dyn BlobService,
    directory_service: &dyn DirectoryService,
) -> Result<bool, RunError> {
    match &path_info.node {
        Node::File { digest, .. } => {
            blob_service.has(digest).await.map_err(|e| RunError::Internal(format!("blob existence check: {e}")))
        }
        Node::Directory { digest, .. } => directory_service
            .get(digest)
            .await
            .map(|directory| directory.is_some())
            .map_err(|e| RunError::Internal(format!("directory existence check: {e}"))),
        Node::Symlink { .. } => Ok(true),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
