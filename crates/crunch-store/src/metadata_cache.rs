//! Advisory remote metadata cache persistence layer (I4).
//!
//! Persists [`MetadataCacheEntry`] records under the state directory using
//! a JSON backing file with bounded record count. Cache entries are keyed
//! by [`MetadataCacheKey`] and are advisory only: they accelerate plan/design
//! discovery but cannot admit outputs without final PathInfo signature, prefix,
//! content, castore, and attestation verification.
//!
//! r[impl cache_substitution.remote_metadata_cache]

use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

// ── Pure core types (I3) ────────────────────────────────────────────────

/// Classes of advisory remote metadata.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MetadataClass {
    /// Narinfo/reference presence confirmed.
    Narinfo,
    /// Negative miss (404 or not found).
    NegativeMiss,
    /// nix-cache-info preflight response.
    CachePreflight,
    /// Delta capability probe response.
    DeltaCapability,
}

impl MetadataClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Narinfo => "narinfo",
            Self::NegativeMiss => "negative-miss",
            Self::CachePreflight => "cache-preflight",
            Self::DeltaCapability => "delta-capability",
        }
    }
}

/// Schema version for metadata cache entries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MetadataSchemaVersion(pub u32);

impl MetadataSchemaVersion {
    /// Current schema version. Increment when the cache key or entry format
    /// changes in a backward-incompatible way.
    pub const CURRENT: MetadataSchemaVersion = MetadataSchemaVersion(1);
}

/// A derived cache key for advisory metadata.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MetadataCacheKey {
    pub cache_identity: String,
    pub trust_policy_digest: String,
    pub store_prefix: String,
    pub output_digest: String,
    pub metadata_class: MetadataClass,
    pub schema_version: MetadataSchemaVersion,
}

/// A persisted metadata cache entry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetadataCacheEntry {
    pub key: MetadataCacheKey,
    /// Unix timestamp (seconds) when this entry was created.
    pub created_at_secs: u64,
    /// Unix timestamp (seconds) when this entry expires.
    pub expires_at_secs: u64,
    /// Human-readable detail for diagnostics.
    pub detail: String,
}

/// Result of checking a metadata cache entry against current policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MetadataValidity {
    /// Entry is fresh and can be reused.
    Fresh,
    /// Entry has exceeded its TTL.
    Expired,
    /// Entry has a schema version that does not match the expected current version.
    SchemaMismatch,
    /// Entry was keyed under a different trust-policy digest.
    TrustPolicyMismatch,
    /// Entry was keyed under a different logical store prefix.
    StorePrefixMismatch,
    /// Explicit refresh policy is in effect — ignore cached metadata.
    ExplicitRefresh,
    /// Entry is missing (no metadata recorded).
    Missing,
    /// Entry cache identity does not match the current candidate.
    CacheIdentityMismatch,
    /// Entry output digest does not match the current request.
    OutputDigestMismatch,
}

impl MetadataValidity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Fresh => "fresh",
            Self::Expired => "expired",
            Self::SchemaMismatch => "schema-mismatch",
            Self::TrustPolicyMismatch => "trust-policy-mismatch",
            Self::StorePrefixMismatch => "store-prefix-mismatch",
            Self::ExplicitRefresh => "explicit-refresh",
            Self::Missing => "missing",
            Self::CacheIdentityMismatch => "cache-identity-mismatch",
            Self::OutputDigestMismatch => "output-digest-mismatch",
        }
    }

    pub fn is_reusable(self) -> bool {
        self == Self::Fresh
    }
}

/// Policy for how a metadata cache entry should be treated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefreshPolicy {
    /// Use cached metadata if fresh; fall back to live probe otherwise.
    Normal,
    /// Ignore all cached metadata even if fresh; force live probes.
    ForceRefresh,
}

/// Default TTL for metadata cache entries in seconds.
pub const DEFAULT_METADATA_TTL_SECS: u64 = 900; // 15 minutes
/// TTL for negative miss entries (shorter, since outputs may appear later).
pub const NEGATIVE_MISS_TTL_SECS: u64 = 300; // 5 minutes
/// Maximum number of metadata cache entries allowed.
pub const MAX_METADATA_CACHE_ENTRIES: usize = 10_000;

/// Typed dimensions for an advisory metadata-cache key.
#[derive(Debug, Clone, Copy)]
pub struct MetadataCacheKeyInput<'a> {
    pub cache_identity: &'a str,
    pub trust_policy_digest: &'a str,
    pub store_prefix: &'a str,
    pub output_digest: &'a str,
    pub metadata_class: MetadataClass,
}

/// Build a [`MetadataCacheKey`] from typed dimensions.
pub fn metadata_cache_key(input: MetadataCacheKeyInput<'_>) -> MetadataCacheKey {
    MetadataCacheKey {
        cache_identity: input.cache_identity.to_string(),
        trust_policy_digest: input.trust_policy_digest.to_string(),
        store_prefix: input.store_prefix.to_string(),
        output_digest: input.output_digest.to_string(),
        metadata_class: input.metadata_class,
        schema_version: MetadataSchemaVersion::CURRENT,
    }
}

// Stable workspace callers still use the positional API; keep it as a thin
// compatibility shell while all decision logic lives in the typed core above.
#[allow(
    tigerstyle::ambiguous_params,
    reason = "stable public compatibility wrapper delegates immediately to typed input"
)]
pub fn build_metadata_cache_key(
    cache_identity: &str,
    trust_policy_digest: &str,
    store_prefix: &str,
    output_digest: &str,
    metadata_class: MetadataClass,
) -> MetadataCacheKey {
    metadata_cache_key(MetadataCacheKeyInput {
        cache_identity,
        trust_policy_digest,
        store_prefix,
        output_digest,
        metadata_class,
    })
}

/// Check whether a cached metadata entry is still valid under current policy.
pub fn check_metadata_validity(
    entry: &MetadataCacheEntry,
    current_key: &MetadataCacheKey,
    current_time_secs: u64,
    refresh_policy: RefreshPolicy,
) -> MetadataValidity {
    assert_eq!(current_key.schema_version, MetadataSchemaVersion::CURRENT);
    assert!(!current_key.store_prefix.is_empty());
    if matches!(refresh_policy, RefreshPolicy::ForceRefresh) {
        return MetadataValidity::ExplicitRefresh;
    }
    if entry.key.cache_identity != current_key.cache_identity {
        return MetadataValidity::CacheIdentityMismatch;
    }
    if entry.key.schema_version != MetadataSchemaVersion::CURRENT {
        return MetadataValidity::SchemaMismatch;
    }
    if entry.key.trust_policy_digest != current_key.trust_policy_digest {
        return MetadataValidity::TrustPolicyMismatch;
    }
    if entry.key.store_prefix != current_key.store_prefix {
        return MetadataValidity::StorePrefixMismatch;
    }
    if entry.key.output_digest != current_key.output_digest {
        return MetadataValidity::OutputDigestMismatch;
    }
    if current_time_secs >= entry.expires_at_secs {
        return MetadataValidity::Expired;
    }
    MetadataValidity::Fresh
}

/// Compute the TTL seconds for a given metadata class.
pub fn metadata_ttl_for_class(class: MetadataClass) -> u64 {
    match class {
        MetadataClass::NegativeMiss => NEGATIVE_MISS_TTL_SECS,
        _ => DEFAULT_METADATA_TTL_SECS,
    }
}

/// Create a fresh [`MetadataCacheEntry`] at the given time.
pub fn new_metadata_entry(key: MetadataCacheKey, now_secs: u64, detail: String) -> MetadataCacheEntry {
    let ttl_secs = metadata_ttl_for_class(key.metadata_class);
    MetadataCacheEntry {
        key,
        created_at_secs: now_secs,
        expires_at_secs: now_secs.saturating_add(ttl_secs),
        detail,
    }
}

/// Summary of where a cache admission decision came from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct AdmissionSummary {
    /// The ultimate admission reason (stable kebab-case string).
    pub reason: String,
    /// Whether the decision was accelerated by cached metadata.
    pub metadata_reused: bool,
    /// If metadata was consulted, its validity status.
    pub metadata_validity: Option<MetadataValidity>,
}

impl AdmissionSummary {
    pub fn from_reason(reason: &str) -> Self {
        Self {
            reason: reason.to_string(),
            metadata_reused: false,
            metadata_validity: None,
        }
    }

    pub fn with_reused_metadata(reason: &str, validity: MetadataValidity) -> Self {
        Self {
            reason: reason.to_string(),
            metadata_reused: true,
            metadata_validity: Some(validity),
        }
    }
}

// ── Persistence layer ───────────────────────────────────────────────────

/// File name for the advisory metadata cache database.
const METADATA_CACHE_FILENAME: &str = "advisory-meta-cache.json";

/// A bounded, persisted store of advisory remote metadata entries.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdvisoryMetadataCache {
    /// Deterministically keyed records. Uses a BTreeMap-like ordering via
    /// a sorted Vec to keep serde output stable and avoid pulling in extra
    /// dependencies.
    entries: Vec<MetadataCacheEntry>,
    /// Maximum number of entries allowed. When exceeded, the oldest entries
    /// by expiry are evicted on the next save.
    #[serde(skip)]
    max_entries: usize,
    /// Whether to force a refresh on the next read.
    #[serde(skip)]
    forced_refresh: bool,
}

impl AdvisoryMetadataCache {
    /// Default maximum number of entries.
    pub const DEFAULT_MAX_ENTRIES: usize = 10_000;

    /// Load from disk, or return empty if the file doesn't exist or is corrupt.
    pub fn load(state_dir: &Path) -> Self {
        let path = Self::file_path(state_dir);
        let entries = match std::fs::read_to_string(&path) {
            Ok(json) => serde_json::from_str::<Vec<MetadataCacheEntry>>(&json).unwrap_or_default(),
            Err(_) => Vec::new(),
        };
        Self {
            entries,
            max_entries: Self::DEFAULT_MAX_ENTRIES,
            forced_refresh: false,
        }
    }

    /// Save to disk. Evicts oldest entries if the count exceeds the maximum.
    pub fn save(&self, state_dir: &Path) {
        if let Err(err) = self.save_checked(state_dir) {
            tracing::warn!(
                path = %Self::file_path(state_dir).display(),
                err = %err,
                "failed to save advisory metadata cache"
            );
        }
    }

    /// Save to disk atomically and surface the error.
    pub fn save_checked(&self, state_dir: &Path) -> Result<(), String> {
        let mut entries = self.entries.clone();
        if entries.len() > self.max_entries {
            entries.sort_by_key(|e| std::cmp::Reverse(e.expires_at_secs));
            entries.truncate(self.max_entries);
        }
        let path = Self::file_path(state_dir);
        std::fs::create_dir_all(state_dir).map_err(|err| format!("creating {}: {err}", state_dir.display()))?;
        let json =
            serde_json::to_vec_pretty(&entries).map_err(|err| format!("serializing {}: {err}", path.display()))?;
        let tmp_path = state_dir.join(format!("{}.tmp", METADATA_CACHE_FILENAME));
        std::fs::write(&tmp_path, json).map_err(|err| format!("writing {}: {err}", tmp_path.display()))?;
        std::fs::rename(&tmp_path, &path)
            .map_err(|err| format!("renaming {} -> {}: {err}", tmp_path.display(), path.display()))?;
        Ok(())
    }

    /// Get an entry by key, if present and fresh.
    pub fn get(&self, key: &MetadataCacheKey) -> Option<&MetadataCacheEntry> {
        if self.forced_refresh {
            return None;
        }
        // Linear scan — bounded by max_entries (10k). For a hot path, a
        // HashMap would be better, but this is advisory-only metadata that
        // is consulted at most once per output per candidate per plan/build.
        self.entries.iter().find(|entry| entry.key == *key)
    }

    /// Insert or update an entry.
    pub fn put(&mut self, entry: MetadataCacheEntry) {
        // Replace existing entry with the same key.
        if let Some(pos) = self.entries.iter().position(|e| e.key == entry.key) {
            self.entries[pos] = entry;
        } else {
            self.entries.push(entry);
        }
    }

    /// Remove an entry by key.
    pub fn remove(&mut self, key: &MetadataCacheKey) -> bool {
        let len_before = self.entries.len();
        self.entries.retain(|e| e.key != *key);
        self.entries.len() < len_before
    }

    /// Remove all expired entries and return the count removed.
    pub fn evict_expired(&mut self, now_secs: u64) -> u64 {
        let entry_count_before = self.entries.len();
        self.entries.retain(|e| e.expires_at_secs > now_secs);
        fixed_entry_count(entry_count_before.saturating_sub(self.entries.len()))
    }

    /// Total number of entries (including expired, if not yet evicted).
    pub fn len(&self) -> u64 {
        fixed_entry_count(self.entries.len())
    }

    /// Whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Set forced refresh mode. When true, `get()` always returns `None`.
    pub fn set_force_refresh(&mut self, enabled: bool) {
        self.forced_refresh = enabled;
    }

    fn file_path(state_dir: &Path) -> PathBuf {
        state_dir.join(METADATA_CACHE_FILENAME)
    }
}

fn fixed_entry_count(entry_count: usize) -> u64 {
    match u64::try_from(entry_count) {
        Ok(entry_count) => entry_count,
        Err(error) => {
            tracing::error!(entry_count, error = %error, "metadata cache entry count does not fit u64");
            std::process::abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_DIGEST: &str = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
    const STORE_PREFIX: &str = "/mantle/store";

    fn sample_key(class: MetadataClass) -> MetadataCacheKey {
        build_metadata_cache_key("https://cache.example.com", VALID_DIGEST, STORE_PREFIX, "digest-hex", class)
    }

    fn sample_entry(key: MetadataCacheKey, created: u64) -> MetadataCacheEntry {
        new_metadata_entry(key, created, "sample detail".to_string())
    }

    #[test]
    fn load_empty_cache_from_nonexistent_file() {
        let state_dir = tempfile::tempdir().unwrap();
        let cache = AdvisoryMetadataCache::load(state_dir.path());
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn put_and_get_roundtrip() {
        let state_dir = tempfile::tempdir().unwrap();
        let mut cache = AdvisoryMetadataCache::load(state_dir.path());
        let entry = sample_entry(sample_key(MetadataClass::Narinfo), 1000);
        let key = entry.key.clone();

        cache.put(entry);
        assert_eq!(cache.len(), 1);

        let retrieved = cache.get(&key);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().detail, "sample detail");
    }

    #[test]
    fn remove_removes_existing_entry() {
        let state_dir = tempfile::tempdir().unwrap();
        let mut cache = AdvisoryMetadataCache::load(state_dir.path());
        let entry = sample_entry(sample_key(MetadataClass::NegativeMiss), 1000);
        let key = entry.key.clone();

        cache.put(entry);
        assert_eq!(cache.len(), 1);

        assert!(cache.remove(&key));
        assert!(cache.is_empty());
    }

    #[test]
    fn remove_returns_false_for_missing_key() {
        let state_dir = tempfile::tempdir().unwrap();
        let mut cache = AdvisoryMetadataCache::load(state_dir.path());
        let key = sample_key(MetadataClass::Narinfo);
        assert!(!cache.remove(&key));
    }

    #[test]
    fn evict_expired_removes_only_expired_entries() {
        let state_dir = tempfile::tempdir().unwrap();
        let mut cache = AdvisoryMetadataCache::load(state_dir.path());

        let fresh_key = sample_key(MetadataClass::Narinfo);
        let mut fresh = sample_entry(fresh_key, 1000);
        // Make this entry have a very long TTL
        fresh.expires_at_secs = 10_000;

        let stale_key = {
            let mut k = sample_key(MetadataClass::NegativeMiss);
            k.output_digest = "stale-digest".to_string();
            k
        };
        let stale = sample_entry(stale_key, 1000);
        // Set expiry in the past
        // Force expiry by setting expires_at_secs explicitly
        let stale_forced = MetadataCacheEntry {
            key: stale.key.clone(),
            created_at_secs: 1000,
            expires_at_secs: 500, // expired before "now" (600)
            detail: "stale".to_string(),
        };

        cache.put(fresh);
        cache.put(stale_forced);

        assert_eq!(cache.len(), 2);

        let evicted = cache.evict_expired(600);
        assert_eq!(evicted, 1);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn put_replaces_existing_entry() {
        let state_dir = tempfile::tempdir().unwrap();
        let mut cache = AdvisoryMetadataCache::load(state_dir.path());
        let key = sample_key(MetadataClass::Narinfo);

        let mut first = sample_entry(key.clone(), 1000);
        first.detail = "first".to_string();
        cache.put(first);

        let mut second = sample_entry(key.clone(), 1100);
        second.detail = "second".to_string();
        cache.put(second);

        let retrieved = cache.get(&key);
        assert_eq!(retrieved.unwrap().detail, "second");
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn save_and_reload_persists_entries() {
        let state_dir = tempfile::tempdir().unwrap();
        {
            // Create and save.
            let mut cache = AdvisoryMetadataCache::load(state_dir.path());
            let entry = sample_entry(sample_key(MetadataClass::CachePreflight), 1000);
            cache.put(entry);
            cache.save(state_dir.path());

            // Verify file exists
            assert!(AdvisoryMetadataCache::file_path(state_dir.path()).exists());
        }
        {
            // Reload.
            let cache = AdvisoryMetadataCache::load(state_dir.path());
            assert_eq!(cache.len(), 1);
            assert_eq!(cache.get(&sample_key(MetadataClass::CachePreflight)).unwrap().detail, "sample detail");
        }
    }

    #[test]
    fn save_evicts_excess_entries() {
        let state_dir = tempfile::tempdir().unwrap();
        let mut cache = AdvisoryMetadataCache::load(state_dir.path());
        cache.max_entries = 2;

        for i in 0..5u64 {
            let mut key = sample_key(MetadataClass::Narinfo);
            key.output_digest = format!("digest-{i}");
            // Newer entries have higher created_at
            let entry = MetadataCacheEntry {
                key: key.clone(),
                created_at_secs: 1000 + i,
                expires_at_secs: 2000 + i,
                detail: format!("entry-{i}"),
            };
            cache.put(entry);
        }

        cache.save(state_dir.path());

        // Reload — should have at most 2 entries
        let reloaded = AdvisoryMetadataCache::load(state_dir.path());
        assert!(reloaded.len() <= 2);
    }

    #[test]
    fn force_refresh_disables_get() {
        let state_dir = tempfile::tempdir().unwrap();
        let mut cache = AdvisoryMetadataCache::load(state_dir.path());
        let entry = sample_entry(sample_key(MetadataClass::Narinfo), 1000);
        let key = entry.key.clone();
        cache.put(entry);
        assert!(cache.get(&key).is_some());

        cache.set_force_refresh(true);
        assert!(cache.get(&key).is_none());

        cache.set_force_refresh(false);
        assert!(cache.get(&key).is_some());
    }

    #[test]
    fn load_corrupt_file_returns_empty() {
        let state_dir = tempfile::tempdir().unwrap();
        let path = AdvisoryMetadataCache::file_path(state_dir.path());
        std::fs::write(&path, b"not valid json").unwrap();
        let cache = AdvisoryMetadataCache::load(state_dir.path());
        assert!(cache.is_empty());
    }
}
