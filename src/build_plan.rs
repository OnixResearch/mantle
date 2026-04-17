use std::ffi::OsString;
use std::num::NonZeroUsize;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use clap::ValueEnum;
use crunch_build::signing;
use crunch_glue::ConversionCache;
use crunch_glue::CrunchDerivation;
use crunch_store::CaMappings;
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
use crate::errors::RunError;
use crate::operator_diagnostics::DoctorProfile;
use crate::operator_diagnostics::DoctorRequest;
use crate::operator_diagnostics::collect_doctor_report;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
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
            if let Some(detail) = &entry.detail {
                out.push_str(&format!("  {}\n", detail));
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

pub struct BuildPlanConfig<'a> {
    pub file: &'a Path,
    pub import_paths: &'a [OsString],
    pub output_dir: &'a Path,
    pub state_dir: &'a Path,
    pub store_dir: &'a str,
    pub substituter_url: Option<&'a str>,
    pub signing_key_path: Option<&'a Path>,
    pub trusted_public_keys: Option<&'a [VerifyingKey]>,
    pub trust_unsigned: bool,
    pub output_mode: BuildOutputMode,
}

pub fn cmd_build_plan(config: BuildPlanConfig<'_>) -> Result<(), RunError> {
    let report = run_build_plan(&config)?;
    let rendered = match config.output_mode {
        BuildOutputMode::Human => report.render_human(),
        BuildOutputMode::Json => report.render_json()?,
    };

    if report.has_preflight_errors() {
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
    let preflight_error = validate_plan_config(config);
    let doctor_report = collect_doctor_report(DoctorRequest {
        profile: DoctorProfile::Build,
        store_dir: config.output_dir,
        state_dir: config.state_dir,
    });
    let plan_store = PlanStore::open(config.state_dir, config.store_dir, config.substituter_url).await?;
    let trust =
        PlanTrust::load(config.signing_key_path, config.trusted_public_keys, config.state_dir, config.trust_unsigned)?;

    let mut entries = Vec::with_capacity(roots.len());
    for root in roots {
        let action = plan_root_action(&plan_store, &trust, &doctor_report, preflight_error.as_deref(), &root).await?;
        entries.push(action);
    }
    entries.sort_by(|left, right| left.label.cmp(&right.label).then(left.drv_key.cmp(&right.drv_key)));

    Ok(BuildPlanReport {
        schema: PLAN_REPORT_SCHEMA,
        file: config.file.display().to_string(),
        output_dir: config.output_dir.display().to_string(),
        state_dir: config.state_dir.display().to_string(),
        store_dir: config.store_dir.to_string(),
        entries,
    })
}

fn evaluate_roots(file: &Path, import_paths: &[OsString], store_dir: &str) -> Result<Vec<PlannedRoot>, RunError> {
    let mut session = crunch_eval::session::EvaluationSession::open_file(file, import_paths)
        .map_err(|e| RunError::Eval(format!("{e}")))?;
    let derivations = session.force_all_roots::<CrunchDerivation>().map_err(|e| match e {
        crunch_eval::Error::Eval(_) | crunch_eval::Error::Io(_) => RunError::Eval(format!("{e}")),
        crunch_eval::Error::Serde(_) => RunError::Build(format!("{e}")),
    })?;
    let mut cache = ConversionCache::new(store_dir);
    let mut roots = Vec::with_capacity(derivations.len());
    for (label, drv) in derivations {
        let (drv_path, derivation) =
            crunch_glue::convert(&drv, &mut cache).map_err(|e| RunError::Build(format!("{label}: {e}")))?;
        roots.push(PlannedRoot {
            label,
            drv_path,
            derivation,
        });
    }
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

async fn plan_root_action(
    plan_store: &PlanStore,
    trust: &PlanTrust,
    doctor_report: &crate::operator_diagnostics::PreflightReport,
    preflight_error: Option<&str>,
    root: &PlannedRoot,
) -> Result<BuildPlanEntry, RunError> {
    if let Some(detail) = preflight_error {
        return Ok(root.plan_entry(&plan_store.store_dir, PlanAction::PreflightError, Some(detail.to_string())));
    }

    let cache_status = plan_store.classify_cache(root, trust).await?;
    if cache_status.all_local {
        return Ok(root.plan_entry(&plan_store.store_dir, PlanAction::Cached, cache_status.detail));
    }
    if cache_status.any_remote && !cache_status.any_build {
        return Ok(root.plan_entry(&plan_store.store_dir, PlanAction::Substitute, cache_status.detail));
    }
    if doctor_report.ok {
        return Ok(root.plan_entry(&plan_store.store_dir, PlanAction::Build, cache_status.detail));
    }

    let failing_checks: Vec<&str> = doctor_report
        .checks
        .iter()
        .filter(|check| check.status == crate::operator_diagnostics::PreflightStatus::Failed)
        .map(|check| check.id)
        .collect();
    let detail = format!("local build blocked by preflight checks: {}", failing_checks.join(", "));
    Ok(root.plan_entry(&plan_store.store_dir, PlanAction::PreflightError, Some(detail)))
}

struct PlannedRoot {
    label: String,
    drv_path: StorePath<String>,
    derivation: Derivation,
}

impl PlannedRoot {
    fn plan_entry(&self, store_dir: &str, action: PlanAction, detail: Option<String>) -> BuildPlanEntry {
        BuildPlanEntry {
            drv_key: self.drv_path.to_absolute_path_with_prefix(store_dir),
            label: self.label.clone(),
            action,
            detail,
        }
    }
}

struct PlanStore {
    store_dir: String,
    local_pathinfo: Arc<dyn PathInfoService>,
    remote_pathinfo: Option<Arc<dyn PathInfoService>>,
    blob_service: Arc<dyn BlobService>,
    directory_service: Arc<dyn DirectoryService>,
    ca_mappings: CaMappings,
}

impl PlanStore {
    async fn open(state_dir: &Path, store_dir: &str, substituter_url: Option<&str>) -> Result<Self, RunError> {
        let blob_service = open_blob_service(state_dir)?;
        let directory_service = open_directory_service(state_dir).await?;
        let local_pathinfo = open_pathinfo_service(state_dir).await?;
        let remote_pathinfo = match substituter_url {
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
        })
    }

    async fn classify_cache(&self, root: &PlannedRoot, trust: &PlanTrust) -> Result<CachePlanStatus, RunError> {
        let mut all_local = true;
        let mut any_remote = false;
        let mut any_build = false;
        let mut detail_parts = Vec::new();
        let is_fod = root.derivation.outputs.values().any(|output| output.ca_hash.is_some());
        let drv_abs = root.drv_path.to_absolute_path_with_prefix(&self.store_dir);

        for (output_name, output) in &root.derivation.outputs {
            let Some(output_path) =
                resolve_output_path(&drv_abs, output_name, output, &self.ca_mappings, &self.store_dir)?
            else {
                all_local = false;
                any_build = true;
                detail_parts.push(format!("{output_name}=build"));
                continue;
            };

            let local_status = self.local_output_status(&output_path, trust).await?;
            match local_status {
                OutputPlan::Local => {
                    detail_parts.push(format!("{output_name}=cached"));
                }
                OutputPlan::Build(reason) => {
                    all_local = false;
                    any_build = true;
                    detail_parts.push(format!("{output_name}=build ({reason})"));
                }
                OutputPlan::Missing => {
                    all_local = false;
                    if !is_fod && self.remote_output_available(&output_path).await? {
                        any_remote = true;
                        detail_parts.push(format!("{output_name}=substitute"));
                    } else {
                        any_build = true;
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
            all_local,
            any_remote,
            any_build,
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

fn resolve_output_path(
    drv_abs: &str,
    output_name: &str,
    output: &nix_compat::derivation::Output,
    ca_mappings: &CaMappings,
    store_dir: &str,
) -> Result<Option<StorePath<String>>, RunError> {
    if let Some(path) = &output.path {
        return Ok(Some(path.clone()));
    }
    let Some(mapped) = ca_mappings.get(drv_abs, output_name) else {
        return Ok(None);
    };
    let path = StorePath::from_absolute_path_with_prefix(mapped.as_bytes(), store_dir)
        .map_err(|_| RunError::Internal(format!("invalid CA mapping path: {mapped}")))?;
    Ok(Some(path))
}

fn open_blob_service(state_dir: &Path) -> Result<Arc<dyn BlobService>, RunError> {
    let blob_dir = state_dir.join("blobs");
    if !blob_dir.is_dir() {
        return Ok(Arc::new(MemoryBlobService::default()) as Arc<dyn BlobService>);
    }
    let svc = ObjectStoreBlobService::new_local(&blob_dir)
        .map_err(|e| RunError::Internal(format!("opening blob dir {}: {e}", blob_dir.display())))?;
    Ok(Arc::new(svc) as Arc<dyn BlobService>)
}

async fn open_directory_service(state_dir: &Path) -> Result<Arc<dyn DirectoryService>, RunError> {
    let path = state_dir.join("directories.redb");
    if !path.is_file() {
        let svc = RedbDirectoryService::new_temporary(
            "plan-empty-directory".to_string(),
            RedbDirectoryServiceConfig::default(),
        )
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
        let svc = LruPathInfoService::with_capacity(
            "plan-empty-pathinfo".to_string(),
            NonZeroUsize::new(32).expect("non-zero pathinfo capacity"),
        );
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
