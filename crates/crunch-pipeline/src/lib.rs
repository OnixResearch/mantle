use std::collections::HashMap;
use std::ffi::OsString;
use std::path::PathBuf;

use crunch_build::{BuildOutcome, Builder, DerivationRegistry, EvalMessage, FailedGoal, Worker};
use crunch_glue::{ConversionCache, CrunchDerivation};
use nix_compat::store_path::StorePath;
use tokio::sync::mpsc;
use tracing::info;

#[derive(Debug, Clone)]
pub struct BuildConfig {
    pub file: PathBuf,
    pub import_paths: Vec<OsString>,
    pub output_dir: PathBuf,
    pub state_dir: PathBuf,
    pub store_dir: String,
    pub verbose: bool,
    pub max_jobs: u32,
    pub substituter_url: Option<String>,
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
        None => std::thread::available_parallelism()
            .map(|n| (n.get() as u32).min(MAX_JOBS_CAP))
            .unwrap_or(1),
    }
}

pub fn deserialize_derivations_from_json(
    json_str: &str,
) -> Result<Vec<(String, CrunchDerivation)>, Error> {
    let json_val: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| Error::Deserialize(format!("parsing JSON: {e}")))?;

    if let Some(arr) = json_val.as_array() {
        let mut derivations = Vec::new();
        for (i, elem) in arr.iter().enumerate() {
            let drv: CrunchDerivation = serde_json::from_value(elem.clone())
                .map_err(|e| Error::Deserialize(format!("deserializing derivation [{i}]: {e}")))?;
            derivations.push((drv.name.clone(), drv));
        }
        return Ok(derivations);
    }

    let Some(obj) = json_val.as_object() else {
        return Err(Error::Deserialize(
            "expected a Derivation record, array of Derivations, or record of Derivations"
                .to_string(),
        ));
    };

    if obj.get("name").is_some_and(|v| v.is_string()) {
        let drv: CrunchDerivation = serde_json::from_value(json_val)
            .map_err(|e| Error::Deserialize(format!("deserializing derivation: {e}")))?;
        let name = drv.name.clone();
        return Ok(vec![(name, drv)]);
    }

    let mut derivations = Vec::new();
    for (key, value) in obj {
        let drv: CrunchDerivation = serde_json::from_value(value.clone())
            .map_err(|e| Error::Deserialize(format!("deserializing derivation '{key}': {e}")))?;
        derivations.push((key.clone(), drv));
    }
    Ok(derivations)
}

pub fn parse_fod_mismatch_error(err: &str) -> Option<FodMismatch> {
    let rest = err.strip_prefix("FOD hash mismatch for ")?;
    let (name, rest) = rest.split_once(": expected ")?;
    let (expected_sri, actual_sri) = rest.split_once(", got ")?;
    Some(FodMismatch {
        name: name.to_string(),
        expected_sri: expected_sri.to_string(),
        actual_sri: actual_sri.to_string(),
    })
}

pub async fn build(config: &BuildConfig) -> Result<PipelineResult, Error> {
    validate_build_config(config)?;

    let json_str = crunch_eval::evaluate_to_json(&config.file, &config.import_paths)
        .map_err(|e| Error::Eval(format!("{e}")))?;
    let derivations = deserialize_derivations_from_json(&json_str)?;
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
        use snix_build::buildservice::BubblewrapBuildService;

        let blob_service = store.blob_service();
        let directory_service = store.directory_service();
        let pathinfo_service = store.pathinfo_service();
        let remote_pathinfo = store.remote_pathinfo();

        let workdir = std::env::temp_dir().join("crunch-builds");
        std::fs::create_dir_all(&workdir)
            .map_err(|e| Error::Internal(format!("create workdir: {e}")))?;

        let build_service = BubblewrapBuildService::new(
            workdir,
            blob_service.clone(),
            directory_service.clone(),
        );

        let mut builder = Builder::with_state_dir(
            blob_service,
            directory_service,
            build_service,
            pathinfo_service,
            config.output_dir.clone(),
            Some(store.state_dir().to_path_buf()),
            remote_pathinfo,
            &config.store_dir,
            config.verbose,
        );

        let (tx, mut rx) = mpsc::channel::<EvalMessage>(16);
        let store_dir = config.store_dir.clone();
        let convert_handle = tokio::task::spawn_blocking(move || {
            convert_all(derivations, &store_dir, tx)
        });

        let mut known_paths = DerivationRegistry::new(&config.store_dir);
        let mut worker = Worker::new(config.max_jobs);
        let worker_result = worker
            .run_streaming(&mut builder, &mut known_paths, &mut rx)
            .await
            .map_err(|e| Error::Build(format!("{e}")));

        let root_drv_paths = convert_handle
            .await
            .map_err(|e| Error::Internal(format!("convert thread panicked: {e}")))??;
        let mut worker_result = worker_result?;
        normalize_failed_goal_keys(&mut worker_result.failed, &config.store_dir);
        let root_labels = build_root_labels(&root_drv_paths, &config.store_dir);
        let fod_mismatches = collect_fod_mismatches(&worker_result.failed);

        Ok(PipelineResult {
            outcomes: worker_result.outcomes,
            failed: worker_result.failed,
            fod_mismatches,
            root_labels,
        })
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = store;
        Err(Error::Build(
            "building is only supported on Linux (requires bwrap)".to_string(),
        ))
    }
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
        return Err(Error::Internal(format!(
            "store_dir must be an absolute path: {}",
            config.store_dir,
        )));
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
        let (drv_path, _nix_drv) = crunch_glue::convert(drv, &mut cache)
            .map_err(|e| Error::Convert(format!("{label}: {e}")))?;

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

fn build_root_labels(
    root_drv_paths: &[(String, StorePath<String>)],
    store_dir: &str,
) -> HashMap<String, String> {
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
    failed
        .iter()
        .filter_map(|failed_goal| parse_fod_mismatch_error(&failed_goal.error))
        .collect()
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

    fn json_single() -> &'static str {
        r#"{
            "name": "hello",
            "builder": "/bin/sh"
        }"#
    }

    fn json_package_set() -> &'static str {
        r#"{
            "hello": { "name": "hello", "builder": "/bin/sh" },
            "world": { "name": "world", "builder": "/bin/sh" }
        }"#
    }

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
    fn deserialize_single_derivation() {
        let derivations = deserialize_derivations_from_json(json_single()).unwrap();
        assert_eq!(derivations.len(), 1);
        assert_eq!(derivations[0].0, "hello");
        assert_eq!(derivations[0].1.name, "hello");
    }

    #[test]
    fn deserialize_package_set() {
        let derivations = deserialize_derivations_from_json(json_package_set()).unwrap();
        assert_eq!(derivations.len(), 2);
        assert!(derivations.iter().any(|(k, _)| k == "hello"));
        assert!(derivations.iter().any(|(k, _)| k == "world"));
    }

    #[test]
    fn deserialize_invalid_json_errors() {
        let err = deserialize_derivations_from_json("[").unwrap_err().to_string();
        assert!(err.contains("parsing JSON"));
    }

    #[test]
    fn parse_fod_mismatch_valid() {
        let mismatch = parse_fod_mismatch_error(
            "FOD hash mismatch for src: expected sha256-aaa, got sha256-bbb",
        )
        .unwrap();
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
    fn parse_drv_key_round_trip() {
        let drv_path = StorePath::from_name_and_digest_fixed("hello.drv", [7u8; 20]).unwrap();
        let key = drv_key_for("/crunch/store", &drv_path);
        let reparsed = parse_drv_key("/crunch/store", &key).unwrap();
        assert_eq!(reparsed, drv_path);
    }

    #[test]
    fn normalize_failed_goal_keys_rewrites_nix_store_keys() {
        let drv_path: StorePath<String> =
            StorePath::from_name_and_digest_fixed("hello.drv", [9u8; 20]).unwrap();
        let mut failed = vec![FailedGoal {
            drv_key: drv_path.to_absolute_path(),
            error: "boom".to_string(),
        }];

        normalize_failed_goal_keys(&mut failed, "/crunch/store");

        assert_eq!(failed[0].drv_key, drv_path.to_absolute_path_with_prefix("/crunch/store"));
    }
}
