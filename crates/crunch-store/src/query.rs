// HARDENING-BACKLOG 2026-09-09: pre-existing tigerstyle findings in this file are
// recorded in .cairn/archive/1970-01-01-complete-store-capability-migration/evidence/
// tigerstyle-remaining-2026-09-09.log and scheduled for the standalone store-shell
// hardening pass. Scoped to the lint categories present at recording time.
#![allow(tigerstyle::sentinel_fallback)]

//! Store query operations: list, info, verify.
//!
//! These operate on a PathInfoService and are independent of the build
//! engine. Callable from the CLI or as a library.

use async_trait::async_trait;
use crunch_gc_core::retention::RuntimeClosureExplanation;
use crunch_gc_core::retention::RuntimeClosureLimits;
use crunch_gc_core::retention::RuntimeClosureRequest;
use crunch_gc_core::retention::explain_runtime_closure;
use crunch_nar::CaseHackPolicy;
use crunch_nar::FilesystemNarRequest;
use futures::StreamExt;
use nix_compat::narinfo::SigningKey;
use nix_compat::narinfo::VerifyingKey;
use nix_compat::nixhash::HashAlgo;
use nix_compat::store_path::StorePath;
use snix_store::path_info::PathInfo;
use snix_store::pathinfoservice::PathInfoService;

use crate::Error;
use crate::StoreBackend;
use crate::StoreHandle;
use crate::closure::LocalMetadataRead;
use crate::closure::RuntimeClosureMetadataPolicy;
use crate::closure::RuntimeMetadataSource;
use crate::closure::collect_runtime_closure_metadata_from_source;
use crate::gc::saturating_u32;

/// Info for a single store path, returned by `store_info`.
#[derive(Debug)]
pub struct PathInfoDetail {
    pub store_path: String,
    pub nar_size: u64,
    pub nar_sha256: Vec<u8>,
    pub deriver: Option<String>,
    pub references: Vec<String>,
    pub signatures: Vec<String>,
    pub ca: Option<String>,
    pub node: String,
}

/// Result of a single path verification.
#[derive(Debug)]
pub enum VerifyResult {
    Ok(String),
    Missing(String),
    Mismatch {
        path: String,
        stored_hash: String,
        actual_hash: String,
        stored_size: u64,
        actual_size: u64,
    },
}

/// Result of signature verification for a single PathInfo.
#[derive(Debug, Clone)]
pub struct SignatureVerifyResult {
    pub path: String,
    pub trusted_count: u32,
    pub untrusted_names: Vec<String>,
    pub total_signatures: u32,
}

impl SignatureVerifyResult {
    pub fn is_trusted(&self) -> bool {
        self.trusted_count > 0
    }
}

/// List all store paths from a PathInfoService.
///
/// Returns (store_path, deriver_name, nar_size) tuples.
pub async fn store_list(svc: &dyn PathInfoService) -> Result<Vec<(String, String, u64)>, Error> {
    const MAX_LIST_ENTRIES: usize = 1_000_000;
    let mut stream = svc.list();
    let mut results = Vec::with_capacity(256);
    for _ in 0..MAX_LIST_ENTRIES {
        let Some(result) = stream.next().await else { break };
        let pi = result.map_err(|e| Error::PathInfoService(format!("listing: {e}")))?;
        let deriver_name = pi.deriver.as_ref().map(|d| d.name().to_string()).unwrap_or_else(|| "-".to_string());
        results.push((pi.store_path.to_string(), deriver_name, pi.nar_size));
    }
    Ok(results)
}

/// Get detailed PathInfo for paths matching a substring filter.
pub async fn store_info(svc: &dyn PathInfoService, path_filter: &str) -> Result<Vec<PathInfoDetail>, Error> {
    assert!(!path_filter.is_empty(), "store_info: empty path_filter would match everything");

    const MAX_SCAN_ENTRIES: u32 = 1_000_000;
    let mut stream = svc.list();
    let mut scanned_count: u32 = 0;
    let mut results = Vec::with_capacity(64);
    for _ in 0..MAX_SCAN_ENTRIES {
        let Some(result) = stream.next().await else { break };
        scanned_count = scanned_count.saturating_add(1);
        let pi = result.map_err(|e| Error::PathInfoService(format!("listing: {e}")))?;
        let sp_str = pi.store_path.to_string();
        if !sp_str.contains(path_filter) {
            continue;
        }
        results.push(PathInfoDetail {
            store_path: sp_str,
            nar_size: pi.nar_size,
            nar_sha256: pi.nar_sha256.to_vec(),
            deriver: pi.deriver.map(|d| d.to_string()),
            references: pi.references.iter().map(|r| r.to_string()).collect(),
            signatures: pi.signatures.iter().map(|s| s.to_string()).collect(),
            ca: pi.ca.map(|c| format!("{c:?}")),
            node: format!("{:?}", pi.node),
        });
    }
    if let Ok(n) = u64::try_from(results.len()) {
        assert!(n <= u64::from(scanned_count), "results cannot exceed scanned paths");
    }
    Ok(results)
}

/// Verify NAR hashes of store paths against what's on disk under `store_dir`.
///
/// `store_dir` is the physical directory where build outputs are exported
/// (the CLI `--store` value), not the logical store prefix. Optionally filters
/// to paths matching `path_filter`. Returns per-path results (Ok, Missing, or
/// Mismatch).
// r[impl store_transports.nix_archive_filesystem_observation]
// r[impl store_transports.nix_archive_boundary]
pub async fn store_verify(
    svc: &dyn PathInfoService,
    path_filter: Option<&str>,
    store_dir: &std::path::Path,
) -> Result<Vec<VerifyResult>, Error> {
    const MAX_VERIFY_ENTRIES: u32 = 1_000_000;
    const STORE_VERIFY_NAR_BYTES_MAX: u64 = 17_592_186_044_416;
    assert!(
        path_filter.is_none_or(|filter| !filter.is_empty()),
        "store_verify: use None instead of empty filter"
    );
    assert!(!store_dir.as_os_str().is_empty(), "store_verify: store_dir must not be empty");
    let mut stream = svc.list();
    let mut scanned_count: u32 = 0;
    let mut results = Vec::with_capacity(64);

    for _ in 0..MAX_VERIFY_ENTRIES {
        let Some(result) = stream.next().await else { break };
        scanned_count = scanned_count.saturating_add(1);
        let path_info = result.map_err(|error| Error::PathInfoService(format!("listing: {error}")))?;
        let logical_path = path_info.store_path.to_string();
        if path_filter.is_some_and(|filter| !logical_path.contains(filter)) {
            continue;
        }
        let physical_path = store_dir.join(&logical_path);
        match std::fs::symlink_metadata(&physical_path) {
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                results.push(VerifyResult::Missing(logical_path));
                continue;
            }
            Err(error) => {
                return Err(Error::Store(format!(
                    "inspecting filesystem NAR root {}: {error}",
                    physical_path.display()
                )));
            }
        }
        let request = FilesystemNarRequest::new(HashAlgo::Sha256, CaseHackPolicy::native(), STORE_VERIFY_NAR_BYTES_MAX);
        let observation = crunch_nar::observe_path_blocking(physical_path, request)
            .await
            .map_err(|error| Error::Store(format!("filesystem NAR observation for {logical_path}: {error}")))?;
        let actual_hash = observation.digest.digest_as_bytes();
        if actual_hash == path_info.nar_sha256 && observation.nar_size == path_info.nar_size {
            results.push(VerifyResult::Ok(logical_path));
        } else {
            results.push(VerifyResult::Mismatch {
                path: logical_path,
                stored_hash: data_encoding::HEXLOWER.encode(&path_info.nar_sha256),
                actual_hash: data_encoding::HEXLOWER.encode(actual_hash),
                stored_size: path_info.nar_size,
                actual_size: observation.nar_size,
            });
        }
    }
    assert!(results.len() <= usize::try_from(scanned_count).unwrap_or(usize::MAX));
    assert!(scanned_count <= MAX_VERIFY_ENTRIES);
    Ok(results)
}

/// Verify PathInfo signatures against trusted public keys.
///
/// `store_dir` is the logical store prefix (for example `/mantle/store`)
/// that was used when the signatures were created. Fingerprints embed the
/// prefix, so verification with the wrong prefix reports every signature as
/// untrusted.
pub async fn store_verify_signatures(
    svc: &dyn PathInfoService,
    path_filter: Option<&str>,
    trusted_keys: &[VerifyingKey],
    store_dir: &str,
) -> Result<Vec<SignatureVerifyResult>, Error> {
    use nix_compat::narinfo::fingerprint_with_store_dir;
    use nix_compat::store_path::StorePathRef;

    assert!(!trusted_keys.is_empty(), "verify_signatures requires at least one trusted key");
    assert!(!store_dir.is_empty(), "verify_signatures: store_dir must not be empty");

    const MAX_VERIFY_SIG_ENTRIES: usize = 1_000_000;
    let mut stream = svc.list();
    let mut results = Vec::with_capacity(64);

    for _ in 0..MAX_VERIFY_SIG_ENTRIES {
        let Some(result) = stream.next().await else { break };
        let pi = result.map_err(|e| Error::PathInfoService(format!("listing: {e}")))?;
        let path = pi.store_path.to_string();

        if let Some(filter) = path_filter
            && !path.contains(filter)
        {
            continue;
        }

        let store_path_ref: StorePathRef = pi.store_path.as_ref();
        let refs: Vec<StorePathRef> = pi.references.iter().map(|r| r.as_ref()).collect();
        let fp = fingerprint_with_store_dir(&store_path_ref, &pi.nar_sha256, pi.nar_size, refs.iter(), store_dir);

        let mut trusted_count: u32 = 0;
        let mut untrusted_names = Vec::with_capacity(pi.signatures.len());
        for sig in &pi.signatures {
            let sig_ref = sig.as_ref();
            let is_trusted = trusted_keys.iter().any(|key| key.verify(&fp, &sig_ref));
            if is_trusted {
                trusted_count = trusted_count.saturating_add(1);
            } else {
                untrusted_names.push(sig.name().clone());
            }
        }

        let total_signatures = saturating_u32(pi.signatures.len());
        assert!(
            total_signatures >= trusted_count,
            "trusted count ({trusted_count}) cannot exceed total signatures ({total_signatures})"
        );
        results.push(SignatureVerifyResult {
            path,
            trusted_count,
            untrusted_names,
            total_signatures,
        });
    }

    Ok(results)
}

/// Result of signing a single PathInfo.
#[derive(Debug)]
pub struct SignResult {
    pub store_path: String,
    /// True if the PathInfo had no signatures before this call.
    pub newly_signed: bool,
    /// True if the PathInfo already had signatures, and this call appended
    /// a signature from a different key.
    pub appended: bool,
    /// True if a signature from the same key was replaced.
    pub replaced: bool,
}

/// Sign PathInfo entries matching `path_filter`.
///
/// When `is_sign_all` is true, only unsigned PathInfos are updated. If a
/// PathInfo already has a signature from the same key name, it is replaced
/// instead of duplicated. Returns one result per updated path.
pub async fn store_sign(
    svc: &dyn PathInfoService,
    signing_key: &SigningKey<ed25519_dalek::SigningKey>,
    path_filter: Option<&str>,
    is_sign_all: bool,
    store_dir: &str,
) -> Result<Vec<SignResult>, Error> {
    use nix_compat::narinfo::Signature;
    use nix_compat::narinfo::fingerprint_with_store_dir;
    use nix_compat::store_path::StorePathRef;

    const SIGN_CANDIDATE_COUNT_LIMIT: usize = 4096;
    assert!(!signing_key.name().is_empty(), "signing key name must not be empty");
    assert!(!store_dir.is_empty(), "store_sign: store_dir must not be empty");

    let mut stream = svc.list();
    let mut to_update = Vec::with_capacity(SIGN_CANDIDATE_COUNT_LIMIT);
    let mut scanned_count: usize = 0;

    for _ in 0..=SIGN_CANDIDATE_COUNT_LIMIT {
        let Some(result) = stream.next().await else { break };
        scanned_count = scanned_count.saturating_add(1);
        if scanned_count > SIGN_CANDIDATE_COUNT_LIMIT {
            return Err(Error::PathInfoService(format!(
                "sign candidate scan exceeded limit {SIGN_CANDIDATE_COUNT_LIMIT}"
            )));
        }

        let pi = result.map_err(|e| Error::PathInfoService(format!("listing: {e}")))?;
        let sp_str = pi.store_path.to_string();

        if is_sign_all {
            if !pi.signatures.is_empty() {
                continue;
            }
        } else if let Some(filter) = path_filter {
            if !sp_str.contains(filter) {
                continue;
            }
        } else {
            continue;
        }

        to_update.push((sp_str, pi));
    }

    let mut results = Vec::with_capacity(to_update.len());
    for (sp_str, mut pi) in to_update {
        let sp_ref: StorePathRef = pi.store_path.as_ref();
        let refs: Vec<StorePathRef> = pi.references.iter().map(|r| r.as_ref()).collect();
        let fp = fingerprint_with_store_dir(&sp_ref, &pi.nar_sha256, pi.nar_size, refs.iter(), store_dir);

        let sig_ref = signing_key.sign(fp.as_bytes());
        let sig_owned: Signature<String> = sig_ref.to_owned();
        let key_name = signing_key.name();

        let is_already_signed = !pi.signatures.is_empty();
        let is_signed_by_same_key = pi.signatures.iter().any(|s| s.name().as_str() == key_name);

        if let Some(pos) = pi.signatures.iter().position(|s| s.name().as_str() == key_name) {
            pi.signatures[pos] = sig_owned;
        } else {
            pi.signatures.push(sig_owned);
        }

        svc.put(pi).await.map_err(|e| Error::PathInfoService(format!("persisting signed PathInfo: {e}")))?;

        let result = SignResult {
            store_path: sp_str,
            newly_signed: !is_already_signed,
            appended: is_already_signed && !is_signed_by_same_key,
            replaced: is_signed_by_same_key,
        };
        assert!(
            (result.newly_signed as u8)
                .saturating_add(result.appended as u8)
                .saturating_add(result.replaced as u8)
                == 1,
            "sign result must be exactly one of newly_signed, appended, or replaced"
        );
        results.push(result);
    }

    Ok(results)
}

#[derive(Clone, Debug)]
pub struct RuntimeClosureQueryOptions {
    pub limits: RuntimeClosureLimits,
    pub trusted_public_keys: Vec<VerifyingKey>,
    pub is_allow_unsigned_local: bool,
    pub is_allow_remote_metadata: bool,
    pub remote_source_id: Option<String>,
}

impl RuntimeClosureQueryOptions {
    #[must_use]
    pub fn local_only(is_allow_unsigned_local: bool) -> Self {
        Self {
            limits: RuntimeClosureLimits::standard(),
            trusted_public_keys: Vec::new(),
            is_allow_unsigned_local,
            is_allow_remote_metadata: false,
            remote_source_id: None,
        }
    }

    pub fn for_store(store: &StoreHandle) -> Result<Self, Error> {
        assert!(!store.store_dir().is_empty());
        assert!(!store.state_dir().as_os_str().is_empty());
        let trusted_public_keys = load_store_closure_trusted_keys(store)?;
        let is_allow_unsigned_local = store.backend().profile().unsigned_admission;
        let is_allow_remote_metadata = store.remote_pathinfo().is_some();
        Ok(Self {
            limits: RuntimeClosureLimits::standard(),
            trusted_public_keys,
            is_allow_unsigned_local,
            is_allow_remote_metadata,
            remote_source_id: None,
        })
    }

    pub fn with_base_state_dirs(mut self, base_state_dirs: &[std::path::PathBuf]) -> Result<Self, Error> {
        for base_dir in base_state_dirs {
            self.trusted_public_keys.extend(crate::overlay::load_layer_trust_keys(base_dir)?);
        }
        self.trusted_public_keys.sort_by_cached_key(ToString::to_string);
        self.trusted_public_keys.dedup();
        Ok(self)
    }

    pub fn with_trusted_public_keys(mut self, keys: &[VerifyingKey]) -> Self {
        self.trusted_public_keys.extend(keys.iter().cloned());
        self.trusted_public_keys.sort_by_cached_key(ToString::to_string);
        self.trusted_public_keys.dedup();
        self
    }
}

pub struct PathInfoRuntimeClosureRequest<'a> {
    pub local: &'a dyn PathInfoService,
    pub remote: Option<&'a dyn PathInfoService>,
    pub store_dir: &'a str,
    pub roots: &'a [StorePath<String>],
    pub options: RuntimeClosureQueryOptions,
}

pub async fn pathinfo_runtime_closure(
    request: PathInfoRuntimeClosureRequest<'_>,
) -> Result<RuntimeClosureExplanation, Error> {
    if request.roots.is_empty() {
        return Err(Error::Store("runtime closure requires at least one root".to_string()));
    }
    if request.store_dir.is_empty() {
        return Err(Error::Store("runtime closure store_dir must not be empty".to_string()));
    }
    debug_assert!(!request.roots.is_empty());
    debug_assert!(!request.store_dir.is_empty());
    let source = PlainPathInfoMetadataSource {
        local: request.local,
        remote: request.remote,
    };
    let policy = RuntimeClosureMetadataPolicy {
        store_dir: request.store_dir.to_string(),
        limits: request.options.limits,
        trusted_public_keys: request.options.trusted_public_keys,
        is_allow_unsigned_local: request.options.is_allow_unsigned_local,
        is_allow_remote_metadata: request.options.is_allow_remote_metadata,
        remote_source_id: request.options.remote_source_id,
    };
    let mut now_ms = unix_wall_clock_ms;
    let collected = collect_runtime_closure_metadata_from_source(request.roots, &source, &policy, &mut now_ms).await?;
    explain_from_collected(request.store_dir, request.roots, policy.limits, collected)
}

pub async fn store_runtime_closure(
    store: &StoreHandle,
    roots: &[StorePath<String>],
    options: RuntimeClosureQueryOptions,
) -> Result<RuntimeClosureExplanation, Error> {
    if roots.is_empty() {
        return Err(Error::Store("runtime closure requires at least one root".to_string()));
    }
    if store.store_dir().is_empty() {
        return Err(Error::Store("runtime closure store_dir must not be empty".to_string()));
    }
    debug_assert!(!roots.is_empty());
    debug_assert!(!store.store_dir().is_empty());
    store.revalidate_overlay_bases()?;
    let source = StoreHandleMetadataSource { store };
    let policy = RuntimeClosureMetadataPolicy {
        store_dir: store.store_dir().to_string(),
        limits: options.limits,
        trusted_public_keys: options.trusted_public_keys,
        is_allow_unsigned_local: options.is_allow_unsigned_local,
        is_allow_remote_metadata: options.is_allow_remote_metadata,
        remote_source_id: options.remote_source_id,
    };
    let mut now_ms = unix_wall_clock_ms;
    let collected = collect_runtime_closure_metadata_from_source(roots, &source, &policy, &mut now_ms).await?;
    store.revalidate_overlay_bases()?;
    explain_from_collected(store.store_dir(), roots, policy.limits, collected)
}

fn explain_from_collected(
    store_dir: &str,
    roots: &[StorePath<String>],
    limits: RuntimeClosureLimits,
    collected: crate::closure::RuntimeClosureCollectedMetadata,
) -> Result<RuntimeClosureExplanation, Error> {
    assert!(!store_dir.is_empty());
    assert!(!roots.is_empty());
    let root_strings = roots.iter().map(|root| root.to_absolute_path_with_prefix(store_dir)).collect();
    explain_runtime_closure(RuntimeClosureRequest {
        store_prefix: store_dir.to_string(),
        roots: root_strings,
        limits,
        elapsed_ms: collected.elapsed_ms,
        members: collected.members,
    })
    .map_err(|error| Error::Store(format!("explaining runtime closure: {error:?}")))
}

fn unix_wall_clock_ms() -> u64 {
    match crate::roots::current_unix_seconds() {
        Ok(secs) => match u64::try_from(secs) {
            Ok(unsigned) => unsigned.saturating_mul(1_000),
            Err(_) => 0,
        },
        Err(_) => 0,
    }
}

fn append_public_keys_if_present(keys: &mut Vec<VerifyingKey>, path: &std::path::Path) -> Result<(), Error> {
    assert!(!path.as_os_str().is_empty());
    if !path.exists() {
        return Ok(());
    }
    let policy = crate::overlay::store_overlay_runtime_policy()?;
    keys.extend(crate::overlay::parse_public_keys(path, policy.limits.max_descriptor_bytes)?);
    debug_assert!(keys.iter().all(|key| !key.name().is_empty()));
    Ok(())
}

fn load_store_closure_trusted_keys(store: &StoreHandle) -> Result<Vec<VerifyingKey>, Error> {
    assert!(!store.state_dir().as_os_str().is_empty());
    assert!(!store.store_dir().is_empty());
    let mut keys = Vec::new();
    if store.backend() == StoreBackend::Casita {
        append_public_keys_if_present(&mut keys, &store.state_dir().join("casita-trusted-public-keys"))?;
        if keys.is_empty()
            && let Some(local_key) = crate::overlay::load_local_verifying_key(store.state_dir())?
        {
            keys.push(local_key);
        }
        return Ok(keys);
    }
    if store.overlay_report()?.is_some() {
        append_public_keys_if_present(&mut keys, &store.state_dir().join("overlay-trusted-public-keys"))?;
    }
    append_public_keys_if_present(&mut keys, &store.state_dir().join("trusted-public-keys"))?;
    if let Some(local_key) = crate::overlay::load_local_verifying_key(store.state_dir())? {
        keys.push(local_key);
    }
    keys.sort_by_cached_key(ToString::to_string);
    keys.dedup();
    Ok(keys)
}

struct PlainPathInfoMetadataSource<'a> {
    local: &'a dyn PathInfoService,
    remote: Option<&'a dyn PathInfoService>,
}

#[async_trait]
impl RuntimeMetadataSource for PlainPathInfoMetadataSource<'_> {
    async fn read_local(&self, path: &StorePath<String>) -> LocalMetadataRead {
        match self.local.get(*path.digest()).await {
            Ok(Some(pi)) => LocalMetadataRead::Found(Box::new(crate::layer::Layered::overlay(pi))),
            Ok(None) => LocalMetadataRead::Missing,
            Err(error) => LocalMetadataRead::Failed(format!("local PathInfo lookup failed: {error}")),
        }
    }

    async fn read_remote(&self, path: &StorePath<String>) -> Result<Option<PathInfo>, String> {
        let Some(remote) = self.remote else {
            return Ok(None);
        };
        remote.get(*path.digest()).await.map_err(|error| format!("remote PathInfo lookup failed: {error}"))
    }
}

struct StoreHandleMetadataSource<'a> {
    store: &'a StoreHandle,
}

#[async_trait]
impl RuntimeMetadataSource for StoreHandleMetadataSource<'_> {
    async fn read_local(&self, path: &StorePath<String>) -> LocalMetadataRead {
        match self.store.path_info_with_layer(path).await {
            Ok(Some(layered)) => LocalMetadataRead::Found(Box::new(layered)),
            Ok(None) => LocalMetadataRead::Missing,
            Err(error) => classify_store_metadata_error(error.to_string()),
        }
    }

    async fn read_remote(&self, path: &StorePath<String>) -> Result<Option<PathInfo>, String> {
        let Some(remote) = self.store.remote_pathinfo() else {
            return Ok(None);
        };
        remote.get(*path.digest()).await.map_err(|error| format!("remote PathInfo lookup failed: {error}"))
    }
}

fn classify_store_metadata_error(detail: String) -> LocalMetadataRead {
    assert!(!detail.is_empty());
    debug_assert!(detail.is_ascii() || !detail.is_empty());
    if detail.contains("overlay-pathinfo-digest-collision") {
        return LocalMetadataRead::IdentityConflict(detail);
    }
    if detail.contains("overlay-layer-trust-failure") || detail.contains("casita-signer-untrusted") {
        return LocalMetadataRead::UntrustedSource(detail);
    }
    LocalMetadataRead::Failed(detail)
}

#[cfg(test)]
mod tests {
    use nix_compat::store_path::StorePath;
    use snix_castore::Node;
    use snix_castore::SymlinkTarget;
    use snix_store::path_info::PathInfo;
    use snix_store::pathinfoservice::LruPathInfoService;
    use snix_store::pathinfoservice::PathInfoService;

    use super::*;

    /// Existing test vectors sign with the plain `fingerprint()` helper, which
    /// embeds the nix-compat default store dir.
    const TEST_STORE_DIR: &str = nix_compat::store_path::STORE_DIR;

    fn test_pathinfo_service() -> LruPathInfoService {
        LruPathInfoService::with_capacity("query-test".to_string(), std::num::NonZeroUsize::new(64).unwrap())
    }

    fn test_keypair() -> (SigningKey<ed25519_dalek::SigningKey>, VerifyingKey) {
        nix_compat::narinfo::parse_keypair(
            "cache.example.com-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==",
        )
        .unwrap()
    }

    fn other_signing_key() -> SigningKey<ed25519_dalek::SigningKey> {
        SigningKey::new("backup-cache-1".to_string(), ed25519_dalek::SigningKey::from_bytes(&[2u8; 32]))
    }

    fn other_verifying_key() -> VerifyingKey {
        nix_compat::narinfo::VerifyingKey::parse("cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY=")
            .unwrap()
    }

    fn dummy_pathinfo(name: &str) -> PathInfo {
        let digest = *blake3::hash(name.as_bytes()).as_bytes();
        let mut store_digest = [0u8; 20];
        store_digest.copy_from_slice(&digest[..20]);
        let store_path = StorePath::from_name_and_digest_fixed(name, store_digest).unwrap();
        PathInfo {
            store_path,
            node: Node::Symlink {
                target: SymlinkTarget::try_from("target").unwrap(),
            },
            references: vec![],
            nar_size: 42,
            nar_sha256: [9u8; 32],
            signatures: vec![],
            deriver: None,
            ca: None,
        }
    }

    #[tokio::test]
    async fn store_sign_adds_signature() {
        let svc = test_pathinfo_service();
        let (signing_key, _verifying_key) = test_keypair();
        let pi = dummy_pathinfo("signed-path");
        let digest = *pi.store_path.digest();
        svc.put(pi).await.unwrap();

        let results = store_sign(&svc, &signing_key, Some("signed-path"), false, TEST_STORE_DIR).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].newly_signed);
        assert!(!results[0].appended);
        assert!(!results[0].replaced);

        let stored = svc.get(digest).await.unwrap().unwrap();
        assert_eq!(stored.signatures.len(), 1);
        assert_eq!(stored.signatures[0].name().as_str(), "cache.example.com-1");
    }

    #[tokio::test]
    async fn store_sign_replaces_same_key_signature() {
        let svc = test_pathinfo_service();
        let (signing_key, _verifying_key) = test_keypair();
        let mut pi = dummy_pathinfo("replace-path");

        let refs: Vec<_> = pi.references.iter().map(|r| r.as_ref()).collect();
        let fp = nix_compat::narinfo::fingerprint(&pi.store_path.as_ref(), &pi.nar_sha256, pi.nar_size, refs.iter());
        pi.signatures.push(signing_key.sign(fp.as_bytes()).to_owned());

        let digest = *pi.store_path.digest();
        svc.put(pi).await.unwrap();

        let results = store_sign(&svc, &signing_key, Some("replace-path"), false, TEST_STORE_DIR).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(!results[0].newly_signed);
        assert!(!results[0].appended);
        assert!(results[0].replaced);

        let stored = svc.get(digest).await.unwrap().unwrap();
        assert_eq!(stored.signatures.len(), 1);
    }

    #[tokio::test]
    async fn store_sign_appends_different_key_signature() {
        let svc = test_pathinfo_service();
        let (signing_key, _verifying_key) = test_keypair();
        let other_key = other_signing_key();
        let mut pi = dummy_pathinfo("append-path");

        let refs: Vec<_> = pi.references.iter().map(|r| r.as_ref()).collect();
        let fp = nix_compat::narinfo::fingerprint(&pi.store_path.as_ref(), &pi.nar_sha256, pi.nar_size, refs.iter());
        pi.signatures.push(other_key.sign(fp.as_bytes()).to_owned());

        let digest = *pi.store_path.digest();
        svc.put(pi).await.unwrap();

        let results = store_sign(&svc, &signing_key, Some("append-path"), false, TEST_STORE_DIR).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(!results[0].newly_signed);
        assert!(results[0].appended);
        assert!(!results[0].replaced);

        let stored = svc.get(digest).await.unwrap().unwrap();
        assert_eq!(stored.signatures.len(), 2);
    }

    #[tokio::test]
    async fn store_sign_all_skips_already_signed_entries() {
        let svc = test_pathinfo_service();
        let (signing_key, _verifying_key) = test_keypair();
        let other_key = other_signing_key();

        let unsigned = dummy_pathinfo("unsigned-path");
        svc.put(unsigned).await.unwrap();

        let mut signed = dummy_pathinfo("already-signed-path");
        let refs: Vec<_> = signed.references.iter().map(|r| r.as_ref()).collect();
        let fp = nix_compat::narinfo::fingerprint(
            &signed.store_path.as_ref(),
            &signed.nar_sha256,
            signed.nar_size,
            refs.iter(),
        );
        signed.signatures.push(other_key.sign(fp.as_bytes()).to_owned());
        let signed_digest = *signed.store_path.digest();
        svc.put(signed).await.unwrap();

        let results = store_sign(&svc, &signing_key, None, true, TEST_STORE_DIR).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].store_path.ends_with("-unsigned-path"));
        assert!(results[0].newly_signed);
        assert!(!results[0].appended);
        assert!(!results[0].replaced);

        let stored = svc.get(signed_digest).await.unwrap().unwrap();
        assert_eq!(stored.signatures.len(), 1);
        assert_eq!(stored.signatures[0].name().as_str(), "backup-cache-1");
    }

    #[tokio::test]
    async fn verify_signatures_accepts_trusted_key() {
        let svc = test_pathinfo_service();
        let (signing_key, verifying_key) = test_keypair();
        let mut pi = dummy_pathinfo("trusted-path");

        let refs: Vec<_> = pi.references.iter().map(|r| r.as_ref()).collect();
        let fp = nix_compat::narinfo::fingerprint(&pi.store_path.as_ref(), &pi.nar_sha256, pi.nar_size, refs.iter());
        pi.signatures.push(signing_key.sign(fp.as_bytes()).to_owned());
        svc.put(pi).await.unwrap();

        let results =
            store_verify_signatures(&svc, Some("trusted-path"), &[verifying_key], TEST_STORE_DIR).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].is_trusted());
        assert_eq!(results[0].trusted_count, 1);
        assert!(results[0].untrusted_names.is_empty());
    }

    #[tokio::test]
    async fn verify_signatures_reports_untrusted_signer() {
        let svc = test_pathinfo_service();
        let (signing_key, _verifying_key) = test_keypair();
        let mut pi = dummy_pathinfo("untrusted-path");

        let refs: Vec<_> = pi.references.iter().map(|r| r.as_ref()).collect();
        let fp = nix_compat::narinfo::fingerprint(&pi.store_path.as_ref(), &pi.nar_sha256, pi.nar_size, refs.iter());
        pi.signatures.push(signing_key.sign(fp.as_bytes()).to_owned());
        svc.put(pi).await.unwrap();

        let results = store_verify_signatures(&svc, Some("untrusted-path"), &[other_verifying_key()], TEST_STORE_DIR)
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
        assert!(!results[0].is_trusted());
        assert_eq!(results[0].trusted_count, 0);
        assert_eq!(results[0].total_signatures, 1);
        assert_eq!(results[0].untrusted_names, vec!["cache.example.com-1".to_string()]);
    }

    /// Compute the NAR hash of an on-disk path the same way `store_verify` does,
    /// so a positive test can pin the exact expected hash.
    async fn nar_hash_of_disk_path(path: &std::path::Path) -> ([u8; 32], u64) {
        const TEST_NAR_BYTES_MAX: u64 = 1_048_576;
        let observation = crunch_nar::observe_path_blocking(
            path.to_path_buf(),
            FilesystemNarRequest::new(HashAlgo::Sha256, CaseHackPolicy::native(), TEST_NAR_BYTES_MAX),
        )
        .await
        .unwrap();
        let hash: [u8; 32] = observation.digest.digest_as_bytes().try_into().unwrap();
        assert_eq!(observation.algorithm, HashAlgo::Sha256);
        assert!(observation.nar_size > 0);
        (hash, observation.nar_size)
    }

    fn file_pathinfo(store_path: StorePath<String>, nar_sha256: [u8; 32], nar_size: u64) -> PathInfo {
        PathInfo {
            store_path,
            node: Node::Symlink {
                target: SymlinkTarget::try_from("target").unwrap(),
            },
            references: vec![],
            nar_size,
            nar_sha256,
            signatures: vec![],
            deriver: None,
            ca: None,
        }
    }

    // r[verify store_transports.nix_archive_filesystem_observation]
    #[tokio::test]
    async fn store_verify_ok_for_exported_path_in_custom_store_dir() {
        let svc = test_pathinfo_service();
        let store_dir = tempfile::tempdir().unwrap();

        let pi = dummy_pathinfo("verify-ok-path");
        let sp_str = pi.store_path.to_string();
        std::fs::write(store_dir.path().join(&sp_str), b"verify-ok").unwrap();
        let (nar_sha256, nar_size) = nar_hash_of_disk_path(&store_dir.path().join(&sp_str)).await;
        svc.put(file_pathinfo(pi.store_path, nar_sha256, nar_size)).await.unwrap();

        let results = store_verify(&svc, None, store_dir.path()).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(matches!(&results[0], VerifyResult::Ok(p) if *p == sp_str));
    }

    #[tokio::test]
    async fn store_verify_missing_for_path_not_on_disk_in_custom_store_dir() {
        let svc = test_pathinfo_service();
        let store_dir = tempfile::tempdir().unwrap();

        let pi = dummy_pathinfo("verify-missing-path");
        let sp_str = pi.store_path.to_string();
        svc.put(pi).await.unwrap();

        let results = store_verify(&svc, None, store_dir.path()).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(matches!(&results[0], VerifyResult::Missing(p) if *p == sp_str));
    }

    // r[verify store_transports.nix_archive_filesystem_observation]
    #[tokio::test]
    async fn store_verify_mismatch_for_tampered_disk_content() {
        let svc = test_pathinfo_service();
        let store_dir = tempfile::tempdir().unwrap();
        let path_info = dummy_pathinfo("verify-mismatch-path");
        let logical_path = path_info.store_path.to_string();
        let good_path = store_dir.path().join(format!("{logical_path}.good"));
        std::fs::write(&good_path, b"original").unwrap();
        let (nar_sha256, nar_size) = nar_hash_of_disk_path(&good_path).await;
        svc.put(file_pathinfo(path_info.store_path, nar_sha256, nar_size)).await.unwrap();
        std::fs::write(store_dir.path().join(&logical_path), b"tampered and larger").unwrap();

        let results = store_verify(&svc, None, store_dir.path()).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(matches!(
            &results[0],
            VerifyResult::Mismatch {
                path,
                stored_size,
                actual_size,
                ..
            } if *path == logical_path && stored_size != actual_size
        ));
    }

    // r[verify store_transports.nix_archive_filesystem_observation]
    #[tokio::test]
    async fn store_verify_read_failure_does_not_persist_pathinfo() {
        let svc = test_pathinfo_service();
        let store_dir = tempfile::tempdir().unwrap();
        let path_info = dummy_pathinfo("verify-read-failure-path");
        let digest = *path_info.store_path.digest();
        let logical_path = path_info.store_path.to_string();
        svc.put(path_info).await.unwrap();
        let physical_path = store_dir.path().join(&logical_path);
        // A socket cannot be observed as a NAR node and fails identically for
        // every effective uid, unlike a mode-000 file that root can still read.
        let _socket = std::os::unix::net::UnixListener::bind(&physical_path).expect("bind an unobservable store entry");

        let result = store_verify(&svc, None, store_dir.path()).await;
        let retained = svc.get(digest).await.unwrap();
        assert!(result.is_err(), "an unobservable path must fail verification");
        assert!(retained.is_some(), "PathInfo must be retained after an observation failure");
    }

    const CUSTOM_PREFIX: &str = "/mantle/store";

    #[tokio::test]
    async fn store_sign_then_verify_roundtrips_under_custom_prefix() {
        let svc = test_pathinfo_service();
        let (signing_key, verifying_key) = test_keypair();
        let pi = dummy_pathinfo("prefix-roundtrip-path");
        svc.put(pi).await.unwrap();

        let sign_results =
            store_sign(&svc, &signing_key, Some("prefix-roundtrip-path"), false, CUSTOM_PREFIX).await.unwrap();
        assert_eq!(sign_results.len(), 1);
        assert!(sign_results[0].newly_signed);

        let verify_results =
            store_verify_signatures(&svc, Some("prefix-roundtrip-path"), &[verifying_key], CUSTOM_PREFIX)
                .await
                .unwrap();
        assert_eq!(verify_results.len(), 1);
        assert!(verify_results[0].is_trusted());
        assert_eq!(verify_results[0].trusted_count, 1);
    }

    #[tokio::test]
    async fn store_verify_signatures_rejects_wrong_prefix() {
        let svc = test_pathinfo_service();
        let (signing_key, verifying_key) = test_keypair();
        let pi = dummy_pathinfo("prefix-mismatch-path");
        svc.put(pi).await.unwrap();

        store_sign(&svc, &signing_key, Some("prefix-mismatch-path"), false, CUSTOM_PREFIX).await.unwrap();

        let verify_results =
            store_verify_signatures(&svc, Some("prefix-mismatch-path"), &[verifying_key], TEST_STORE_DIR)
                .await
                .unwrap();
        assert_eq!(verify_results.len(), 1);
        assert!(!verify_results[0].is_trusted());
        assert_eq!(verify_results[0].trusted_count, 0);
    }

    #[tokio::test]
    async fn pathinfo_runtime_closure_explains_diamond_and_counts_shared_once() {
        let svc = test_pathinfo_service();
        let make = |name: &str, seed: u8, size: u64, refs: Vec<StorePath<String>>| {
            let sp = StorePath::from_name_and_digest_fixed(name, [seed; 20]).unwrap();
            PathInfo {
                store_path: sp,
                node: Node::Symlink {
                    target: SymlinkTarget::try_from("x").unwrap(),
                },
                references: refs,
                nar_size: size,
                nar_sha256: [seed; 32],
                signatures: vec![],
                deriver: None,
                ca: None,
            }
        };
        let leaf = make("leaf", 4, 400, vec![]);
        let left = make("left", 2, 20, vec![leaf.store_path.clone()]);
        let right = make("right", 3, 30, vec![leaf.store_path.clone()]);
        let root = make("root", 1, 10, vec![left.store_path.clone(), right.store_path.clone()]);
        for pi in [leaf, left, right, root.clone()] {
            svc.put(pi).await.unwrap();
        }

        let explanation = pathinfo_runtime_closure(PathInfoRuntimeClosureRequest {
            local: &svc,
            remote: None,
            store_dir: CUSTOM_PREFIX,
            roots: &[root.store_path],
            options: RuntimeClosureQueryOptions::local_only(true),
        })
        .await
        .unwrap();
        assert_eq!(explanation.completeness, crunch_gc_core::retention::RuntimeClosureCompleteness::Complete);
        assert_eq!(explanation.observed_member_count, 4);
        assert_eq!(explanation.observed_known_nar_bytes, 460);
    }

    #[tokio::test]
    async fn store_runtime_closure_on_snix_and_casita_never_mutates_roots() {
        for backend in [StoreBackend::Snix, StoreBackend::Casita] {
            let dir = tempfile::tempdir().unwrap();
            let state_dir = dir.path().join("state");
            let output_dir = dir.path().join("store");
            let (signing_key, verifying_key) = test_keypair();
            if backend == StoreBackend::Casita {
                std::fs::create_dir_all(&state_dir).unwrap();
                std::fs::write(state_dir.join("casita-trusted-public-keys"), format!("{}\n", verifying_key)).unwrap();
            }
            let store = StoreHandle::open(crate::StoreConfig::new(
                backend,
                state_dir.clone(),
                output_dir.clone(),
                CUSTOM_PREFIX.to_string(),
            ))
            .await
            .unwrap();

            use snix_store::nar::NarCalculationService as _;
            let mut writer = store.blob_service().open_write().await;
            tokio::io::AsyncWriteExt::write_all(&mut writer, b"hello closure").await.unwrap();
            let blob_digest = writer.close().await.unwrap();
            let final_node = Node::File {
                digest: blob_digest,
                size: 13,
                executable: false,
            };
            let (nar_size, nar_sha256) =
                snix_store::nar::SimpleRenderer::new(store.blob_service(), store.directory_service())
                    .calculate_nar(&final_node)
                    .await
                    .unwrap();
            let root = StorePath::from_name_and_digest_fixed("app", [11u8; 20]).unwrap();
            let empty_refs: [nix_compat::store_path::StorePathRef<'_>; 0] = [];
            let fp = nix_compat::narinfo::fingerprint_with_store_dir(
                &root.as_ref(),
                &nar_sha256,
                nar_size,
                empty_refs.iter(),
                CUSTOM_PREFIX,
            );
            let sig = signing_key.sign(fp.as_bytes()).to_owned();
            let path_info = PathInfo {
                store_path: root.clone(),
                node: final_node.clone(),
                references: Vec::new(),
                nar_size,
                nar_sha256,
                signatures: vec![sig],
                deriver: None,
                ca: None,
            };
            let mut store = store;
            store
                .persist_and_export_signed_output(crate::PersistOutputRequest {
                    output_name: "out",
                    output_path: &root,
                    path_info,
                    final_node,
                    provenance: None,
                    is_root: false,
                    root_source: None,
                })
                .await
                .unwrap();

            let roots_before = store.list_retained_roots().unwrap();
            let options = RuntimeClosureQueryOptions::for_store(&store).unwrap();
            let explanation = store_runtime_closure(&store, &[root], options).await.unwrap();
            let roots_after = store.list_retained_roots().unwrap();
            assert_eq!(roots_before, roots_after);
            assert_eq!(explanation.completeness, crunch_gc_core::retention::RuntimeClosureCompleteness::Complete);
            assert_eq!(explanation.observed_member_count, 1);
            assert!(explanation.observed_known_nar_bytes > 0);
        }
    }

    #[tokio::test]
    async fn store_runtime_closure_single_snix_store_with_signing_key_admits_trusted_and_imported_other_signer() {
        let dir = tempfile::tempdir().unwrap();
        let state_dir = dir.path().join("state");
        let output_dir = dir.path().join("store");
        std::fs::create_dir_all(&state_dir).unwrap();

        let local_key_line =
            "local-build-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==";
        std::fs::write(state_dir.join("signing-key"), format!("{local_key_line}\n")).unwrap();
        let (local_signing, _) = nix_compat::narinfo::parse_keypair(local_key_line).unwrap();

        let configured_raw = ed25519_dalek::SigningKey::from_bytes(&[71_u8; 32]);
        let configured_verifying = VerifyingKey::new("configured-1".to_string(), configured_raw.verifying_key());
        let configured_signing = SigningKey::new("configured-1".to_string(), configured_raw);
        std::fs::write(state_dir.join("trusted-public-keys"), format!("{configured_verifying}\n")).unwrap();

        let other_signing = other_signing_key();

        let store = StoreHandle::open(crate::StoreConfig::new(
            StoreBackend::Snix,
            state_dir,
            output_dir,
            CUSTOM_PREFIX.to_string(),
        ))
        .await
        .unwrap();

        let sign_pi =
            |name: &str, seed: u8, refs: Vec<StorePath<String>>, signer: &SigningKey<ed25519_dalek::SigningKey>| {
                let mut pi = dummy_pathinfo(name);
                pi.store_path = StorePath::from_name_and_digest_fixed(name, [seed; 20]).unwrap();
                pi.references = refs;
                let ref_refs: Vec<_> = pi.references.iter().map(StorePath::as_ref).collect();
                let fp = nix_compat::narinfo::fingerprint_with_store_dir(
                    &pi.store_path.as_ref(),
                    &pi.nar_sha256,
                    pi.nar_size,
                    ref_refs.iter(),
                    CUSTOM_PREFIX,
                );
                pi.signatures.push(signer.sign(fp.as_bytes()).to_owned());
                pi
            };

        let other_dep = sign_pi("other-dep", 81, vec![], &other_signing);
        let configured_dep = sign_pi("configured-dep", 82, vec![], &configured_signing);
        let local_root = sign_pi(
            "local-root",
            83,
            vec![other_dep.store_path.clone(), configured_dep.store_path.clone()],
            &local_signing,
        );
        for pi in [other_dep.clone(), configured_dep.clone(), local_root.clone()] {
            store.pathinfo_service().put(pi).await.unwrap();
        }

        let options = RuntimeClosureQueryOptions::for_store(&store).unwrap();
        let explanation =
            store_runtime_closure(&store, std::slice::from_ref(&local_root.store_path), options).await.unwrap();
        assert_eq!(explanation.completeness, crunch_gc_core::retention::RuntimeClosureCompleteness::Complete);
        assert_eq!(explanation.observed_member_count, 3);

        let local_member = explanation
            .members
            .iter()
            .find(|m| m.path_id == local_root.store_path.to_absolute_path_with_prefix(CUSTOM_PREFIX))
            .unwrap();
        assert!(matches!(local_member.trust, crunch_gc_core::retention::RuntimeClosureTrustStatus::Trusted { .. }));

        let configured_member = explanation
            .members
            .iter()
            .find(|m| m.path_id == configured_dep.store_path.to_absolute_path_with_prefix(CUSTOM_PREFIX))
            .unwrap();
        assert!(matches!(
            configured_member.trust,
            crunch_gc_core::retention::RuntimeClosureTrustStatus::Trusted { .. }
        ));

        let other_member = explanation
            .members
            .iter()
            .find(|m| m.path_id == other_dep.store_path.to_absolute_path_with_prefix(CUSTOM_PREFIX))
            .unwrap();
        assert_eq!(other_member.state, crunch_gc_core::retention::RuntimeClosureMemberState::Observed);
        assert!(matches!(other_member.trust, crunch_gc_core::retention::RuntimeClosureTrustStatus::Untrusted { .. }));
    }

    #[tokio::test]
    async fn pathinfo_runtime_closure_and_store_runtime_closure_reject_empty_roots_without_panicking() {
        let svc = test_pathinfo_service();
        let empty_pi_err = pathinfo_runtime_closure(PathInfoRuntimeClosureRequest {
            local: &svc,
            remote: None,
            store_dir: CUSTOM_PREFIX,
            roots: &[],
            options: RuntimeClosureQueryOptions::local_only(true),
        })
        .await
        .unwrap_err();
        assert!(empty_pi_err.to_string().contains("at least one root"), "{empty_pi_err}");

        let root = dummy_pathinfo("valid-root").store_path;
        let empty_dir_err = pathinfo_runtime_closure(PathInfoRuntimeClosureRequest {
            local: &svc,
            remote: None,
            store_dir: "",
            roots: std::slice::from_ref(&root),
            options: RuntimeClosureQueryOptions::local_only(true),
        })
        .await
        .unwrap_err();
        assert!(empty_dir_err.to_string().contains("store_dir must not be empty"), "{empty_dir_err}");

        let dir = tempfile::tempdir().unwrap();
        let store = StoreHandle::open(crate::StoreConfig::new(
            StoreBackend::Snix,
            dir.path().join("state"),
            dir.path().join("store"),
            CUSTOM_PREFIX.to_string(),
        ))
        .await
        .unwrap();
        let options = RuntimeClosureQueryOptions::for_store(&store).unwrap();
        let store_err = store_runtime_closure(&store, &[], options).await.unwrap_err();
        assert!(store_err.to_string().contains("at least one root"), "{store_err}");
    }
}
