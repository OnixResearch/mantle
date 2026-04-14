use std::collections::HashMap;
use std::ffi::OsString;
use std::path::PathBuf;

use crunch_build::BuildOutcome;
use crunch_build::Builder;
use crunch_build::DerivationRegistry;
use crunch_build::DispatchBuildService;
use crunch_build::EvalMessage;
use crunch_build::FailedGoal;
use crunch_build::FetchBuildService;
use crunch_build::KeyPair;
use crunch_build::Worker;
use crunch_glue::ConversionCache;
use crunch_glue::CrunchDerivation;
use nix_compat::narinfo::VerifyingKey;
use nix_compat::store_path::StorePath;
use tokio::sync::mpsc;
use tracing::info;

pub struct BuildConfig {
    pub file: PathBuf,
    pub import_paths: Vec<OsString>,
    pub output_dir: PathBuf,
    pub state_dir: PathBuf,
    pub store_dir: String,
    pub verbose: bool,
    pub max_jobs: u32,
    pub substituter_url: Option<String>,
    /// Signing keypair — every build output gets signed.
    pub keypair: KeyPair,
    /// Trusted public keys for signature verification on cache hits.
    pub trusted_keys: Vec<VerifyingKey>,
    /// When true, skip signature verification on cache hits.
    pub trust_unsigned: bool,
}

#[derive(Debug)]
pub struct PipelineResult {
    pub outcomes: Vec<BuildOutcome>,
    pub failed: Vec<FailedGoal>,
    pub fod_mismatches: Vec<FodMismatch>,
    pub root_labels: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FodMismatch {
    pub name: String,
    pub expected_sri: String,
    pub actual_sri: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Eval(String),
    #[error("{0}")]
    Deserialize(String),
    #[error("{0}")]
    Convert(String),
    #[error("{0}")]
    Build(String),
    #[error("{0}")]
    Internal(String),
}

pub fn resolve_max_jobs(user: Option<u32>) -> u32 {
    const MAX_JOBS_CAP: u32 = 16;
    match user {
        Some(j) => j.clamp(1, MAX_JOBS_CAP),
        None => std::thread::available_parallelism().map(|n| (n.get() as u32).min(MAX_JOBS_CAP)).unwrap_or(1),
    }
}

pub fn parse_fod_mismatch_error(err: &str) -> Option<FodMismatch> {
    let rest = err.strip_prefix("FOD hash mismatch for ")?;
    let (name, rest) = rest.split_once(": expected ")?;
    let (expected_sri, actual_sri) = rest.split_once(", got ")?;
    let normalized_name = name.strip_suffix(".drv").unwrap_or(name);
    Some(FodMismatch {
        name: normalized_name.to_string(),
        expected_sri: expected_sri.to_string(),
        actual_sri: actual_sri.to_string(),
    })
}

pub async fn build(config: &BuildConfig) -> Result<PipelineResult, Error> {
    validate_build_config(config)?;

    let derivations =
        crunch_eval::evaluate_and_extract_named_roots::<CrunchDerivation>(&config.file, &config.import_paths).map_err(
            |e| match e {
                crunch_eval::Error::Eval(_) | crunch_eval::Error::Io(_) => Error::Eval(format!("{e}")),
                crunch_eval::Error::Serde(_) => Error::Deserialize(format!("{e}")),
            },
        )?;
    debug_assert!(!derivations.is_empty(), "must have at least one derivation");

    let store = crunch_store::StoreHandle::open(crunch_store::StoreConfig {
        state_dir: config.state_dir.clone(),
        output_dir: config.output_dir.clone(),
        remote_cache_url: config.substituter_url.clone(),
        store_dir: config.store_dir.clone(),
    })
    .await
    .map_err(|e| Error::Internal(format!("opening store: {e}")))?;

    #[cfg(target_os = "linux")]
    {
        return build_linux(config, store, derivations).await;
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = store;
        Err(Error::Build("building is only supported on Linux (requires bwrap)".to_string()))
    }
}

#[cfg(target_os = "linux")]
async fn build_linux(
    config: &BuildConfig,
    store: crunch_store::StoreHandle,
    derivations: Vec<(String, CrunchDerivation)>,
) -> Result<PipelineResult, Error> {
    use snix_build::buildservice::BubblewrapBuildService;

    let blob_service = store.blob_service();
    let directory_service = store.directory_service();
    let pathinfo_service = store.pathinfo_service();
    let remote_pathinfo = store.remote_pathinfo();
    let workdir = std::env::temp_dir().join("crunch-builds");
    std::fs::create_dir_all(&workdir).map_err(|e| Error::Internal(format!("create workdir: {e}")))?;

    let bwrap_service = BubblewrapBuildService::new(workdir, blob_service.clone(), directory_service.clone());
    let fetch_service = FetchBuildService::new(blob_service.clone(), directory_service.clone());
    let build_service = DispatchBuildService::new(fetch_service, bwrap_service);
    let mut builder = Builder::with_state_dir(
        blob_service,
        directory_service,
        build_service,
        pathinfo_service,
        config.output_dir.clone(),
        Some(store.state_dir().to_path_buf()),
        remote_pathinfo,
        &config.store_dir,
        config.keypair.clone(),
        config.trusted_keys.clone(),
        config.trust_unsigned,
        config.verbose,
    );

    let (tx, mut rx) = mpsc::channel::<EvalMessage>(16);
    let store_dir = config.store_dir.clone();
    let convert_handle = tokio::task::spawn_blocking(move || convert_all(derivations, &store_dir, tx));
    let mut known_paths = DerivationRegistry::new(&config.store_dir);
    let mut worker = Worker::new(config.max_jobs);
    let worker_run = worker.run_streaming(&mut builder, &mut known_paths, &mut rx).await;
    let root_drv_paths =
        convert_handle.await.map_err(|e| Error::Internal(format!("convert thread panicked: {e}")))??;
    let mut worker_result = match worker_run {
        Ok(result) => result,
        Err(err) => return Err(Error::Build(format!("{err}"))),
    };
    normalize_failed_goal_keys(&mut worker_result.failed, &config.store_dir);
    Ok(PipelineResult {
        root_labels: build_root_labels(&root_drv_paths, &config.store_dir),
        fod_mismatches: collect_fod_mismatches(&worker_result.failed),
        outcomes: worker_result.outcomes,
        failed: worker_result.failed,
    })
}

fn validate_build_config(config: &BuildConfig) -> Result<(), Error> {
    if !config.output_dir.exists() {
        return Err(Error::Internal(format!(
            "output store directory {} does not exist.\nCreate it with: sudo mkdir -p {0} && sudo chown $USER {0}",
            config.output_dir.display()
        )));
    }
    if config.max_jobs == 0 {
        return Err(Error::Internal("max_jobs must be at least 1".to_string()));
    }
    if config.store_dir.is_empty() {
        return Err(Error::Internal("store_dir must not be empty".to_string()));
    }
    if !config.store_dir.starts_with('/') {
        return Err(Error::Internal(format!("store_dir must be an absolute path: {}", config.store_dir,)));
    }
    Ok(())
}

fn convert_all(
    derivations: Vec<(String, CrunchDerivation)>,
    store_dir: &str,
    tx: mpsc::Sender<EvalMessage>,
) -> Result<Vec<(String, StorePath<String>)>, Error> {
    let mut cache = ConversionCache::new(store_dir);
    let mut drv_paths = Vec::new();

    for (label, drv) in &derivations {
        let (drv_path, _nix_drv) =
            crunch_glue::convert(drv, &mut cache).map_err(|e| Error::Convert(format!("{label}: {e}")))?;

        let new_entries = cache.drain_pending();
        info!(drv = %drv_path, label = %label, entries = new_entries.len(), "converted, sending to worker");

        tx.blocking_send(EvalMessage {
            label: label.clone(),
            drv_path: drv_path.clone(),
            new_entries,
        })
        .map_err(|e| Error::Internal(format!("channel send: {e}")))?;

        drv_paths.push((label.clone(), drv_path));
    }

    drop(tx);
    Ok(drv_paths)
}

fn build_root_labels(root_drv_paths: &[(String, StorePath<String>)], store_dir: &str) -> HashMap<String, String> {
    let mut labels = HashMap::new();
    for (label, drv_path) in root_drv_paths {
        let key = drv_path.to_absolute_path_with_prefix(store_dir);
        labels.insert(key, label.clone());
    }
    labels
}

fn normalize_failed_goal_keys(failed: &mut [FailedGoal], store_dir: &str) {
    for failed_goal in failed {
        if parse_drv_key(store_dir, &failed_goal.drv_key).is_some() {
            continue;
        }
        let Ok(drv_path) = StorePath::from_absolute_path(failed_goal.drv_key.as_bytes()) else {
            continue;
        };
        failed_goal.drv_key = drv_key_for(store_dir, &drv_path);
    }
}

fn collect_fod_mismatches(failed: &[FailedGoal]) -> Vec<FodMismatch> {
    failed.iter().filter_map(|failed_goal| parse_fod_mismatch_error(&failed_goal.error)).collect()
}

pub fn drv_key_for(store_dir: &str, drv_path: &StorePath<String>) -> String {
    drv_path.to_absolute_path_with_prefix(store_dir)
}

pub fn parse_drv_key(store_dir: &str, drv_key: &str) -> Option<StorePath<String>> {
    StorePath::from_absolute_path_with_prefix(drv_key.as_bytes(), store_dir).ok()
}

pub fn label_for_key<'a>(result: &'a PipelineResult, drv_key: &str) -> Option<&'a str> {
    result.root_labels.get(drv_key).map(String::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_max_jobs_default_in_range() {
        let jobs = resolve_max_jobs(None);
        assert!(jobs >= 1);
        assert!(jobs <= 16);
    }

    #[test]
    fn resolve_max_jobs_clamps_user_value() {
        assert_eq!(resolve_max_jobs(Some(0)), 1);
        assert_eq!(resolve_max_jobs(Some(1)), 1);
        assert_eq!(resolve_max_jobs(Some(99)), 16);
    }

    #[test]
    fn parse_fod_mismatch_valid() {
        let mismatch =
            parse_fod_mismatch_error("FOD hash mismatch for src: expected sha256-aaa, got sha256-bbb").unwrap();
        assert_eq!(mismatch.name, "src");
        assert_eq!(mismatch.expected_sri, "sha256-aaa");
        assert_eq!(mismatch.actual_sri, "sha256-bbb");
    }

    #[test]
    fn parse_fod_mismatch_invalid() {
        assert!(parse_fod_mismatch_error("something else").is_none());
        assert!(parse_fod_mismatch_error("FOD hash mismatch for x").is_none());
    }

    #[test]
    fn parse_fod_mismatch_edge_case_preserves_trailing_context() {
        let mismatch = parse_fod_mismatch_error(
            "FOD hash mismatch for src-1: expected sha256-aaa, got sha256-bbb (builder log follows)",
        )
        .unwrap();
        assert_eq!(mismatch.name, "src-1");
        assert_eq!(mismatch.expected_sri, "sha256-aaa");
        assert_eq!(mismatch.actual_sri, "sha256-bbb (builder log follows)");
    }

    #[test]
    fn parse_fod_mismatch_strips_drv_suffix() {
        let mismatch =
            parse_fod_mismatch_error("FOD hash mismatch for src-1.drv: expected sha256-aaa, got sha256-bbb").unwrap();
        assert_eq!(mismatch.name, "src-1");
        assert_eq!(mismatch.expected_sri, "sha256-aaa");
        assert_eq!(mismatch.actual_sri, "sha256-bbb");
    }

    #[test]
    fn parse_drv_key_round_trip() {
        let drv_path = StorePath::from_name_and_digest_fixed("hello.drv", [7u8; 20]).unwrap();
        let key = drv_key_for("/crunch/store", &drv_path);
        let reparsed = parse_drv_key("/crunch/store", &key).unwrap();
        assert_eq!(reparsed, drv_path);
    }

    #[test]
    fn normalize_failed_goal_keys_rewrites_nix_store_keys() {
        let drv_path: StorePath<String> = StorePath::from_name_and_digest_fixed("hello.drv", [9u8; 20]).unwrap();
        let mut failed = vec![FailedGoal {
            drv_key: drv_path.to_absolute_path(),
            error: "boom".to_string(),
        }];

        normalize_failed_goal_keys(&mut failed, "/crunch/store");

        assert_eq!(failed[0].drv_key, drv_path.to_absolute_path_with_prefix("/crunch/store"));
    }
}
