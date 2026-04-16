use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use nix_compat::store_path::StorePath;
use serde::Deserialize;
use serde::Serialize;
use snix_store::pathinfoservice::PathInfoService;

use crate::Error;

const ROOTS_FILE_NAME: &str = "gc-roots.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GcRootSource {
    Build,
    SelfBuild,
    /// `crunch bootstrap --fetch` retained root.
    Bootstrap,
    Pin,
}

impl GcRootSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Build => "build",
            Self::SelfBuild => "self-build",
            Self::Bootstrap => "bootstrap",
            Self::Pin => "pin",
        }
    }
}

impl std::fmt::Display for GcRootSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GcRootRecord {
    pub logical_path: String,
    pub source: GcRootSource,
    pub created_unix_s: i64,
}

pub fn list_roots(state_dir: &Path) -> Result<Vec<GcRootRecord>, Error> {
    let registry = load_registry(state_dir)?;
    Ok(registry.into_values().collect())
}

pub async fn register_root(
    state_dir: &Path,
    store_dir: &str,
    pathinfo: &dyn PathInfoService,
    store_path: &StorePath<String>,
    source: GcRootSource,
) -> Result<GcRootRecord, Error> {
    assert!(!store_dir.is_empty(), "store_dir must not be empty");
    assert!(store_dir.starts_with('/'), "store_dir must be absolute");

    ensure_pathinfo_exists(pathinfo, store_path).await?;

    let logical_path = store_path.to_absolute_path_with_prefix(store_dir);
    let created_unix_s = current_unix_seconds()?;
    let record = GcRootRecord {
        logical_path: logical_path.clone(),
        source,
        created_unix_s,
    };

    let mut registry = load_registry(state_dir)?;
    registry.insert(logical_path, record.clone());
    save_registry(state_dir, &registry)?;
    Ok(record)
}

pub async fn pin_root(
    state_dir: &Path,
    store_dir: &str,
    pathinfo: &dyn PathInfoService,
    logical_path: &str,
) -> Result<GcRootRecord, Error> {
    let store_path = parse_logical_store_path(logical_path, store_dir)?;
    register_root(state_dir, store_dir, pathinfo, &store_path, GcRootSource::Pin).await
}

pub fn unpin_root(state_dir: &Path, store_dir: &str, logical_path: &str) -> Result<Option<GcRootRecord>, Error> {
    let normalized_path = normalize_logical_path(logical_path, store_dir)?;
    let mut registry = load_registry(state_dir)?;
    let removed = registry.remove(&normalized_path);
    save_registry(state_dir, &registry)?;
    Ok(removed)
}

pub(crate) fn load_registry(state_dir: &Path) -> Result<BTreeMap<String, GcRootRecord>, Error> {
    let path = roots_path(state_dir);
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
        Err(err) => {
            return Err(Error::RootRegistry(format!("reading {}: {err}", path.display())));
        }
    };
    let registry: BTreeMap<String, GcRootRecord> = serde_json::from_slice(&bytes)
        .map_err(|err| Error::RootRegistry(format!("parsing {}: {err}", path.display())))?;
    validate_registry(&registry)?;
    Ok(registry)
}

pub(crate) fn roots_path(state_dir: &Path) -> PathBuf {
    state_dir.join(ROOTS_FILE_NAME)
}

pub(crate) fn parse_logical_store_path(logical_path: &str, store_dir: &str) -> Result<StorePath<String>, Error> {
    StorePath::from_absolute_path_with_prefix(logical_path.as_bytes(), store_dir).map_err(|_| {
        Error::RootRegistry(format!("path is not under configured store prefix {store_dir}: {logical_path}"))
    })
}

fn normalize_logical_path(logical_path: &str, store_dir: &str) -> Result<String, Error> {
    let store_path = parse_logical_store_path(logical_path, store_dir)?;
    Ok(store_path.to_absolute_path_with_prefix(store_dir))
}

async fn ensure_pathinfo_exists(pathinfo: &dyn PathInfoService, store_path: &StorePath<String>) -> Result<(), Error> {
    let digest = *store_path.digest();
    let stored = pathinfo
        .get(digest)
        .await
        .map_err(|err| Error::RootRegistry(format!("reading PathInfo for {store_path}: {err}")))?;
    let Some(stored) = stored else {
        return Err(Error::RootRegistry(format!("cannot pin nonexistent store path {store_path}")));
    };
    if stored.store_path != *store_path {
        return Err(Error::RootRegistry(format!(
            "PathInfo digest collision for {store_path}: stored path was {}",
            stored.store_path
        )));
    }
    Ok(())
}

fn save_registry(state_dir: &Path, registry: &BTreeMap<String, GcRootRecord>) -> Result<(), Error> {
    validate_registry(registry)?;
    std::fs::create_dir_all(state_dir)
        .map_err(|err| Error::RootRegistry(format!("creating {}: {err}", state_dir.display())))?;
    let path = roots_path(state_dir);
    let tmp_path = state_dir.join(format!("{}.tmp", ROOTS_FILE_NAME));
    let bytes = serde_json::to_vec_pretty(registry)
        .map_err(|err| Error::RootRegistry(format!("serializing {}: {err}", path.display())))?;
    std::fs::write(&tmp_path, bytes)
        .map_err(|err| Error::RootRegistry(format!("writing {}: {err}", tmp_path.display())))?;
    std::fs::rename(&tmp_path, &path)
        .map_err(|err| Error::RootRegistry(format!("renaming {} -> {}: {err}", tmp_path.display(), path.display())))?;
    Ok(())
}

fn validate_registry(registry: &BTreeMap<String, GcRootRecord>) -> Result<(), Error> {
    for (key, record) in registry {
        if key.is_empty() {
            return Err(Error::RootRegistry("root registry key must not be empty".to_string()));
        }
        if !key.starts_with('/') {
            return Err(Error::RootRegistry(format!("root registry key must be absolute: {key}")));
        }
        if record.logical_path != *key {
            return Err(Error::RootRegistry(format!(
                "root registry key/value mismatch: key={key}, value={}",
                record.logical_path
            )));
        }
    }
    Ok(())
}

fn current_unix_seconds() -> Result<i64, Error> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| Error::RootRegistry(format!("system clock before unix epoch: {err}")))?;
    i64::try_from(duration.as_secs()).map_err(|_| Error::RootRegistry("unix timestamp exceeds i64 range".to_string()))
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;
    use std::sync::Arc;

    use async_trait::async_trait;
    use futures::StreamExt;
    use futures::stream::BoxStream;
    use snix_castore::Node;
    use snix_castore::SymlinkTarget;
    use snix_store::path_info::PathInfo;
    use snix_store::pathinfoservice::LruPathInfoService;

    use super::*;

    fn store_path(name: &str, seed: u8) -> StorePath<String> {
        StorePath::from_name_and_digest_fixed(name, [seed; 20]).unwrap()
    }

    fn pathinfo_service() -> Arc<dyn PathInfoService> {
        Arc::new(LruPathInfoService::with_capacity("roots-test".to_string(), NonZeroUsize::new(32).unwrap()))
            as Arc<dyn PathInfoService>
    }

    fn signedless_pathinfo(store_path: StorePath<String>) -> PathInfo {
        PathInfo {
            store_path,
            node: Node::Symlink {
                target: SymlinkTarget::try_from("target").unwrap(),
            },
            references: vec![],
            nar_size: 1,
            nar_sha256: [1u8; 32],
            signatures: vec![],
            deriver: None,
            ca: None,
        }
    }

    #[tokio::test]
    async fn register_root_survives_reload() {
        let state_dir = tempfile::tempdir().unwrap();
        let pathinfo = pathinfo_service();
        let store_path = store_path("kept", 7);
        pathinfo.put(signedless_pathinfo(store_path.clone())).await.unwrap();

        let record = register_root(state_dir.path(), "/nix/store", pathinfo.as_ref(), &store_path, GcRootSource::Build)
            .await
            .unwrap();
        let listed = list_roots(state_dir.path()).unwrap();

        assert_eq!(listed, vec![record]);
        assert_eq!(listed[0].logical_path, store_path.to_absolute_path());
    }

    #[tokio::test]
    async fn registering_same_path_refreshes_single_record() {
        let state_dir = tempfile::tempdir().unwrap();
        let pathinfo = pathinfo_service();
        let store_path = store_path("refresh", 8);
        pathinfo.put(signedless_pathinfo(store_path.clone())).await.unwrap();

        let first = register_root(state_dir.path(), "/nix/store", pathinfo.as_ref(), &store_path, GcRootSource::Build)
            .await
            .unwrap();
        register_root(state_dir.path(), "/nix/store", pathinfo.as_ref(), &store_path, GcRootSource::Pin)
            .await
            .unwrap();
        let listed = list_roots(state_dir.path()).unwrap();

        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].logical_path, first.logical_path);
        assert_eq!(listed[0].source, GcRootSource::Pin);
        assert!(listed[0].created_unix_s >= first.created_unix_s);
    }

    #[derive(Default)]
    struct FailingPathInfoService;

    #[async_trait]
    impl PathInfoService for FailingPathInfoService {
        async fn get(&self, _digest: [u8; 20]) -> Result<Option<PathInfo>, snix_store::pathinfoservice::Error> {
            Err(std::io::Error::other("unreadable metadata").into())
        }

        async fn put(&self, path_info: PathInfo) -> Result<PathInfo, snix_store::pathinfoservice::Error> {
            Ok(path_info)
        }

        fn list(&self) -> BoxStream<'static, Result<PathInfo, snix_store::pathinfoservice::Error>> {
            futures::stream::empty().boxed()
        }
    }

    #[tokio::test]
    async fn pin_rejects_nonexistent_path() {
        let state_dir = tempfile::tempdir().unwrap();
        let pathinfo = pathinfo_service();
        let store_path = store_path("missing", 9);
        let logical_path = store_path.to_absolute_path();

        let err = pin_root(state_dir.path(), "/nix/store", pathinfo.as_ref(), &logical_path).await.unwrap_err();

        assert!(matches!(err, Error::RootRegistry(msg) if msg.contains("nonexistent")));
    }

    #[tokio::test]
    async fn pin_rejects_unreadable_metadata() {
        let state_dir = tempfile::tempdir().unwrap();
        let store_path = store_path("broken", 10);
        let logical_path = store_path.to_absolute_path();
        let service = FailingPathInfoService;

        let err = pin_root(state_dir.path(), "/nix/store", &service, &logical_path).await.unwrap_err();

        assert!(
            matches!(err, Error::RootRegistry(msg) if msg.contains("unreadable metadata") || msg.contains("reading PathInfo"))
        );
    }

    #[test]
    fn unpin_removes_existing_record() {
        let state_dir = tempfile::tempdir().unwrap();
        let path = store_path("hello", 10).to_absolute_path();
        let mut registry = BTreeMap::new();
        registry.insert(path.clone(), GcRootRecord {
            logical_path: path.clone(),
            source: GcRootSource::Pin,
            created_unix_s: 123,
        });
        save_registry(state_dir.path(), &registry).unwrap();

        let removed = unpin_root(state_dir.path(), "/nix/store", &path).unwrap();
        let listed = list_roots(state_dir.path()).unwrap();

        assert!(removed.is_some());
        assert!(listed.is_empty());
    }

    #[test]
    fn load_registry_rejects_mismatched_key_and_value() {
        let state_dir = tempfile::tempdir().unwrap();
        let path = roots_path(state_dir.path());
        let mut json = serde_json::Map::new();
        json.insert(
            store_path("good", 11).to_absolute_path(),
            serde_json::json!({
                "logical_path": store_path("bad", 12).to_absolute_path(),
                "source": "pin",
                "created_unix_s": 1,
            }),
        );
        std::fs::write(&path, serde_json::to_vec_pretty(&json).unwrap()).unwrap();

        let err = load_registry(state_dir.path()).unwrap_err();
        assert!(matches!(err, Error::RootRegistry(msg) if msg.contains("mismatch")));
    }
}
