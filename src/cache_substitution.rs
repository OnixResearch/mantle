//! Pure cache-substitution candidate model.
//!
//! Functional core for ordered substituter sets, trust-policy digests,
//! sanitized cache identities, configured priority, and deterministic
//! tie-breakers. No I/O, no network, no filesystem — the imperative shell
//! owns service construction and remote probes.

// r[impl cache_substitution.ordered_substituters]

use std::collections::BTreeSet;

use serde::Serialize;

/// Maximum number of configured substituters in one candidate set.
pub const MAX_CACHE_CANDIDATES: usize = 32;
/// Maximum length of a sanitized cache identity string.
pub const MAX_CACHE_IDENTITY_BYTES: usize = 512;

const TRUST_POLICY_DIGEST_BYTES: usize = 64;

/// A sanitized, ordered substituter candidate derived from configuration.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct CacheCandidate {
    /// Deterministic identity: sanitized base URL + trust-policy digest.
    pub cache_identity: String,
    /// Configured priority (lower = higher priority).
    pub priority: u32,
    /// BLAKE3 hex digest of the trust policy (verifying keys + store prefix).
    pub trust_policy_digest: String,
    /// Store prefix this candidate is compatible with.
    pub store_prefix: String,
    /// Whether this candidate requires network access.
    pub requires_network: bool,
}

/// A parsed and validated ordered substituter set.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CacheCandidateSet {
    pub candidates: Vec<CacheCandidate>,
}

pub struct CacheCandidateSetRequest<'a> {
    pub raw_urls: &'a [String],
    pub store_prefix: &'a str,
    pub trust_policy_digest: &'a str,
}

/// Reason code for cache admission or rejection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CacheAdmissionReason {
    /// Local PathInfo and complete castore content present.
    LocalHit,
    /// Remote substitution admitted after full verification.
    RemoteHit,
    /// No candidate had a matching PathInfo.
    Miss,
    /// Candidate URL was malformed.
    MalformedUrl,
    /// Duplicate cache identity with different trust material.
    DuplicateIdentityTrustMismatch,
    /// Signature verification failed.
    UntrustedSignature,
    /// Store prefix mismatch.
    StorePrefixMismatch,
    /// Fixed-output remote hit not admitted (requires local build).
    FixedOutputRemoteHit,
    /// Offline mode: network required but unavailable.
    OfflineNetworkRequired,
    /// Local PathInfo present but castore content incomplete.
    LocalCastoreIncomplete,
}

impl CacheAdmissionReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LocalHit => "local-hit",
            Self::RemoteHit => "remote-hit",
            Self::Miss => "miss",
            Self::MalformedUrl => "malformed-url",
            Self::DuplicateIdentityTrustMismatch => "duplicate-identity-trust-mismatch",
            Self::UntrustedSignature => "untrusted-signature",
            Self::StorePrefixMismatch => "store-prefix-mismatch",
            Self::FixedOutputRemoteHit => "fixed-output-remote-hit",
            Self::OfflineNetworkRequired => "offline-network-required",
            Self::LocalCastoreIncomplete => "local-castore-incomplete",
        }
    }

    pub fn is_hit(self) -> bool {
        matches!(self, Self::LocalHit | Self::RemoteHit)
    }
}

/// A single per-output cache admission event.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CacheAdmissionEvent {
    pub cache_identity: Option<String>,
    pub reason: CacheAdmissionReason,
    pub fallback: bool,
}

/// Parse a comma-separated substituter URL list into a validated candidate set.
///
/// Each raw URL is sanitized to a base URL (scheme + host + port + path,
/// no query/fragment/userinfo). Duplicate identities with the same trust
/// policy are collapsed; duplicates with different trust material are rejected.
pub fn parse_cache_candidate_set(
    request: CacheCandidateSetRequest<'_>,
) -> Result<CacheCandidateSet, CacheCandidateError> {
    debug_assert!(MAX_CACHE_CANDIDATES > 0);
    debug_assert_eq!(TRUST_POLICY_DIGEST_BYTES, blake3::OUT_LEN.saturating_mul(2));
    if request.raw_urls.len() > MAX_CACHE_CANDIDATES {
        return Err(CacheCandidateError::TooManyCandidates {
            actual: request.raw_urls.len(),
            limit: MAX_CACHE_CANDIDATES,
        });
    }
    validate_trust_policy_digest(request.trust_policy_digest)?;
    if !request.store_prefix.starts_with('/') {
        return Err(CacheCandidateError::InvalidStorePrefix(request.store_prefix.to_string()));
    }

    let mut candidates = Vec::with_capacity(request.raw_urls.len());
    let mut seen_identities = BTreeSet::new();

    for (index, raw_url) in request.raw_urls.iter().enumerate() {
        let identity = sanitize_cache_identity(raw_url).ok_or_else(|| CacheCandidateError::MalformedUrl {
            index,
            url: raw_url.clone(),
        })?;
        if identity.len() > MAX_CACHE_IDENTITY_BYTES {
            return Err(CacheCandidateError::IdentityTooLong {
                index,
                len: identity.len(),
                limit: MAX_CACHE_IDENTITY_BYTES,
            });
        }
        if !seen_identities.insert(identity.clone()) {
            // Duplicate identity with same trust policy: skip (same cache).
            continue;
        }
        let is_network_required = !identity.starts_with("file://");
        let priority = u32::try_from(index).map_err(|_| CacheCandidateError::PriorityOverflow { index })?;
        candidates.push(CacheCandidate {
            cache_identity: identity,
            priority,
            trust_policy_digest: request.trust_policy_digest.to_string(),
            store_prefix: request.store_prefix.to_string(),
            requires_network: is_network_required,
        });
    }

    // Stable sort by priority preserves configuration order for ties.
    candidates.sort_by_key(|c| c.priority);
    Ok(CacheCandidateSet { candidates })
}

/// Sanitize a raw substituter URL to a deterministic cache identity.
///
/// Strips query, fragment, and userinfo. Keeps scheme, host, port, and path.
/// Returns `None` if the URL is missing a scheme or host.
pub fn sanitize_cache_identity(raw_url: &str) -> Option<String> {
    debug_assert!(MAX_CACHE_IDENTITY_BYTES > 0);
    debug_assert!(MAX_CACHE_CANDIDATES > 0);
    let parsed = url::Url::parse(raw_url).ok()?;
    if parsed.scheme().is_empty() {
        return None;
    }
    // file:// URLs have no host — use scheme + path as the identity.
    if parsed.scheme() == "file" {
        let path = parsed.path();
        if path.is_empty() || path == "/" {
            return None;
        }
        return Some(format!("file://{path}"));
    }
    parsed.host_str()?;
    let mut sanitized = parsed.clone();
    sanitized.set_query(None);
    sanitized.set_fragment(None);
    sanitized.set_username("").ok()?;
    sanitized.set_password(None).ok()?;
    let port_suffix = sanitized.port().map(|port_number| format!(":{port_number}")).unwrap_or_default();
    let path = sanitized.path();
    if path == "/" {
        Some(format!("{}://{}{}", sanitized.scheme(), sanitized.host_str()?, port_suffix))
    } else {
        Some(format!("{}://{}{}{}", sanitized.scheme(), sanitized.host_str()?, port_suffix, path))
    }
}

/// Split a comma-separated substituters string into individual URLs.
///
/// Whitespace around each URL is trimmed. Empty entries are skipped.
pub fn split_substituter_urls(input: &str) -> Vec<String> {
    input.split(',').map(|part| part.trim().to_string()).filter(|part| !part.is_empty()).collect()
}

/// Check whether two candidates share the same identity but different trust material.
pub fn candidates_have_trust_conflict(a: &CacheCandidate, b: &CacheCandidate) -> bool {
    a.cache_identity == b.cache_identity && a.trust_policy_digest != b.trust_policy_digest
}

/// Validate that a trust-policy digest is 64-char lowercase hex (BLAKE3).
fn validate_trust_policy_digest(digest: &str) -> Result<(), CacheCandidateError> {
    if digest.len() != TRUST_POLICY_DIGEST_BYTES {
        return Err(CacheCandidateError::InvalidTrustPolicyDigest);
    }
    if !digest.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()) {
        return Err(CacheCandidateError::InvalidTrustPolicyDigest);
    }
    Ok(())
}

/// Errors from cache-candidate parsing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CacheCandidateError {
    TooManyCandidates { actual: usize, limit: usize },
    MalformedUrl { index: usize, url: String },
    IdentityTooLong { index: usize, len: usize, limit: usize },
    InvalidStorePrefix(String),
    InvalidTrustPolicyDigest,
    PriorityOverflow { index: usize },
}

impl std::fmt::Display for CacheCandidateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooManyCandidates { actual, limit } => {
                write!(f, "too many cache candidates: {actual} > {limit}")
            }
            Self::MalformedUrl { index, url } => {
                write!(f, "malformed substituter URL at index {index}: {url}")
            }
            Self::IdentityTooLong { index, len, limit } => {
                write!(f, "cache identity at index {index} is {len} bytes, limit {limit}")
            }
            Self::InvalidStorePrefix(prefix) => {
                write!(f, "store prefix must be absolute: {prefix}")
            }
            Self::InvalidTrustPolicyDigest => {
                write!(f, "trust policy digest must be 64 lowercase hex chars")
            }
            Self::PriorityOverflow { index } => {
                write!(f, "cache candidate priority does not fit u32: {index}")
            }
        }
    }
}

impl std::error::Error for CacheCandidateError {}

// ── Advisory Metadata Cache (I3) ─────────────────────────────────────────
// Pure core types live in crunch-store. Re-export for callers of
// cache_substitution.

pub use crunch_store::metadata_cache::MetadataCacheKey;
pub use crunch_store::metadata_cache::MetadataClass;
pub use crunch_store::metadata_cache::build_metadata_cache_key;

/// Convenience wrapper: build a [`MetadataCacheKey`] from a [`CacheCandidate`].
pub fn build_metadata_cache_key_from_candidate(
    candidate: &CacheCandidate,
    output_digest: &str,
    metadata_class: MetadataClass,
) -> MetadataCacheKey {
    build_metadata_cache_key(
        &candidate.cache_identity,
        &candidate.trust_policy_digest,
        &candidate.store_prefix,
        output_digest,
        metadata_class,
    )
}

#[cfg(test)]
mod tests {
    use crunch_store::metadata_cache::AdmissionSummary;
    use crunch_store::metadata_cache::DEFAULT_METADATA_TTL_SECS;
    use crunch_store::metadata_cache::MetadataSchemaVersion;
    use crunch_store::metadata_cache::MetadataValidity;
    use crunch_store::metadata_cache::NEGATIVE_MISS_TTL_SECS;
    use crunch_store::metadata_cache::RefreshPolicy;
    use crunch_store::metadata_cache::check_metadata_validity;
    use crunch_store::metadata_cache::metadata_ttl_for_class;
    use crunch_store::metadata_cache::new_metadata_entry;

    use super::*;

    const VALID_DIGEST: &str = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
    const STORE_PREFIX: &str = "/mantle/store";
    const _: () = assert!(NEGATIVE_MISS_TTL_SECS < DEFAULT_METADATA_TTL_SECS);

    fn parse_fixture(raw_urls: &[String], policy: (&str, &str)) -> Result<CacheCandidateSet, CacheCandidateError> {
        parse_cache_candidate_set(CacheCandidateSetRequest {
            raw_urls,
            store_prefix: policy.0,
            trust_policy_digest: policy.1,
        })
    }

    fn candidate(identity: &str, priority: u32) -> CacheCandidate {
        CacheCandidate {
            cache_identity: identity.to_string(),
            priority,
            trust_policy_digest: VALID_DIGEST.to_string(),
            store_prefix: STORE_PREFIX.to_string(),
            requires_network: !identity.starts_with("file://"),
        }
    }

    // ── sanitize_cache_identity ────────────────────────────────────

    #[test]
    fn sanitize_strips_query_fragment_and_userinfo() {
        let identity = sanitize_cache_identity("https://user:pass@cache.example.com/path?secret=1#frag").unwrap();
        assert_eq!(identity, "https://cache.example.com/path");
    }

    #[test]
    fn sanitize_preserves_port() {
        let identity = sanitize_cache_identity("https://cache.example.com:8080/path").unwrap();
        assert_eq!(identity, "https://cache.example.com:8080/path");
    }

    #[test]
    fn sanitize_strips_trailing_slash_root() {
        let identity = sanitize_cache_identity("https://cache.example.com/").unwrap();
        assert_eq!(identity, "https://cache.example.com");
    }

    #[test]
    fn sanitize_rejects_missing_scheme() {
        assert!(sanitize_cache_identity("cache.example.com/path").is_none());
    }

    #[test]
    fn sanitize_rejects_missing_host_and_scheme() {
        assert!(sanitize_cache_identity("cache.example.com/path").is_none());
        assert!(sanitize_cache_identity("not-a-url").is_none());
    }

    // ── parse_cache_candidate_set ──────────────────────────────────

    #[test]
    fn parse_orders_candidates_by_configuration_order() {
        let urls = vec![
            "https://cache-a.example.com".to_string(),
            "https://cache-b.example.com".to_string(),
        ];
        let set = parse_fixture(&urls, (STORE_PREFIX, VALID_DIGEST)).unwrap();
        assert_eq!(set.candidates.len(), 2);
        assert_eq!(set.candidates[0].cache_identity, "https://cache-a.example.com");
        assert_eq!(set.candidates[0].priority, 0);
        assert_eq!(set.candidates[1].cache_identity, "https://cache-b.example.com");
        assert_eq!(set.candidates[1].priority, 1);
    }

    #[test]
    fn parse_deduplicates_same_identity_same_trust() {
        let urls = vec![
            "https://cache.example.com?k=1".to_string(),
            "https://cache.example.com?k=2".to_string(),
        ];
        let set = parse_fixture(&urls, (STORE_PREFIX, VALID_DIGEST)).unwrap();
        assert_eq!(set.candidates.len(), 1);
    }

    #[test]
    fn parse_rejects_malformed_url() {
        let urls = vec!["not a url".to_string()];
        let err = parse_fixture(&urls, (STORE_PREFIX, VALID_DIGEST)).unwrap_err();
        assert!(matches!(err, CacheCandidateError::MalformedUrl { index: 0, .. }));
    }

    #[test]
    fn parse_rejects_too_many_candidates() {
        let urls: Vec<String> =
            (0..MAX_CACHE_CANDIDATES + 1).map(|i| format!("https://cache-{i}.example.com")).collect();
        let err = parse_fixture(&urls, (STORE_PREFIX, VALID_DIGEST)).unwrap_err();
        assert!(matches!(err, CacheCandidateError::TooManyCandidates { .. }));
    }

    #[test]
    fn parse_rejects_invalid_store_prefix() {
        let urls = vec!["https://cache.example.com".to_string()];
        let err = parse_fixture(&urls, ("relative/path", VALID_DIGEST)).unwrap_err();
        assert!(matches!(err, CacheCandidateError::InvalidStorePrefix(_)));
    }

    #[test]
    fn parse_rejects_invalid_trust_policy_digest() {
        let urls = vec!["https://cache.example.com".to_string()];
        let err = parse_fixture(&urls, (STORE_PREFIX, "not-a-digest")).unwrap_err();
        assert!(matches!(err, CacheCandidateError::InvalidTrustPolicyDigest));
    }

    #[test]
    fn parse_marks_file_url_as_not_requiring_network() {
        let urls = vec!["file:///tmp/cache".to_string()];
        let set = parse_fixture(&urls, (STORE_PREFIX, VALID_DIGEST)).unwrap();
        assert!(!set.candidates[0].requires_network);
    }

    #[test]
    fn parse_marks_https_url_as_requiring_network() {
        let urls = vec!["https://cache.example.com".to_string()];
        let set = parse_fixture(&urls, (STORE_PREFIX, VALID_DIGEST)).unwrap();
        assert!(set.candidates[0].requires_network);
    }

    // ── candidates_have_trust_conflict ─────────────────────────────

    #[test]
    fn trust_conflict_detected_for_same_identity_different_digest() {
        let mut a = candidate("https://cache.example.com", 0);
        let b = candidate("https://cache.example.com", 1);
        a.trust_policy_digest = "1111111111111111111111111111111111111111111111111111111111111111".to_string();
        assert!(candidates_have_trust_conflict(&a, &b));
    }

    #[test]
    fn no_trust_conflict_for_different_identity() {
        let a = candidate("https://cache-a.example.com", 0);
        let b = candidate("https://cache-b.example.com", 1);
        assert!(!candidates_have_trust_conflict(&a, &b));
    }

    #[test]
    fn no_trust_conflict_for_same_identity_same_digest() {
        let a = candidate("https://cache.example.com", 0);
        let b = candidate("https://cache.example.com", 1);
        assert!(!candidates_have_trust_conflict(&a, &b));
    }

    // ── CacheAdmissionReason ───────────────────────────────────────

    #[test]
    fn admission_reason_is_hit_classifies_local_and_remote() {
        assert!(CacheAdmissionReason::LocalHit.is_hit());
        assert!(CacheAdmissionReason::RemoteHit.is_hit());
        assert!(!CacheAdmissionReason::Miss.is_hit());
        assert!(!CacheAdmissionReason::LocalCastoreIncomplete.is_hit());
    }

    #[test]
    fn admission_reason_as_str_is_stable() {
        assert_eq!(CacheAdmissionReason::LocalCastoreIncomplete.as_str(), "local-castore-incomplete");
        assert_eq!(CacheAdmissionReason::OfflineNetworkRequired.as_str(), "offline-network-required");
        assert_eq!(CacheAdmissionReason::DuplicateIdentityTrustMismatch.as_str(), "duplicate-identity-trust-mismatch");
    }

    // ── split_substituter_urls ────────────────────────────────────

    #[test]
    fn split_substituter_urls_splits_commas_and_trims() {
        let urls = split_substituter_urls("https://a.example.com, https://b.example.com");
        assert_eq!(urls, vec!["https://a.example.com", "https://b.example.com"]);
    }

    #[test]
    fn split_substituter_urls_skips_empty_entries() {
        let urls = split_substituter_urls("https://a.example.com,,");
        assert_eq!(urls, vec!["https://a.example.com"]);
    }

    #[test]
    fn split_substituter_urls_returns_single() {
        let urls = split_substituter_urls("https://cache.nixos.org");
        assert_eq!(urls, vec!["https://cache.nixos.org"]);
    }

    #[test]
    fn split_substituter_urls_returns_empty_for_empty_input() {
        assert!(split_substituter_urls("").is_empty());
        assert!(split_substituter_urls(",,").is_empty());
    }

    // ── CacheAdmissionEvent ────────────────────────────────────────

    #[test]
    fn admission_event_serializes_stable_reason_code() {
        let event = CacheAdmissionEvent {
            cache_identity: Some("https://cache.example.com".to_string()),
            reason: CacheAdmissionReason::UntrustedSignature,
            fallback: false,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"reason\":\"untrusted-signature\""));
        assert!(json.contains("\"fallback\":false"));
    }

    // ── Metadata cache (I3) ───────────────────────────────────────

    fn meta_candidate() -> CacheCandidate {
        CacheCandidate {
            cache_identity: "https://cache.example.com".to_string(),
            priority: 0,
            trust_policy_digest: VALID_DIGEST.to_string(),
            store_prefix: STORE_PREFIX.to_string(),
            requires_network: true,
        }
    }

    #[test]
    fn metadata_class_as_str_is_stable() {
        assert_eq!(MetadataClass::Narinfo.as_str(), "narinfo");
        assert_eq!(MetadataClass::NegativeMiss.as_str(), "negative-miss");
        assert_eq!(MetadataClass::CachePreflight.as_str(), "cache-preflight");
        assert_eq!(MetadataClass::DeltaCapability.as_str(), "delta-capability");
    }

    #[test]
    fn metadata_schema_version_current_is_one() {
        assert_eq!(MetadataSchemaVersion::CURRENT, MetadataSchemaVersion(1));
    }

    #[test]
    fn build_metadata_cache_key_includes_all_dimensions() {
        let candidate = meta_candidate();
        let key = build_metadata_cache_key_from_candidate(&candidate, "abc123", MetadataClass::Narinfo);
        assert_eq!(key.cache_identity, "https://cache.example.com");
        assert_eq!(key.trust_policy_digest, VALID_DIGEST);
        assert_eq!(key.store_prefix, STORE_PREFIX);
        assert_eq!(key.output_digest, "abc123");
        assert_eq!(key.metadata_class, MetadataClass::Narinfo);
        assert_eq!(key.schema_version, MetadataSchemaVersion(1));
    }

    #[test]
    fn metadata_ttl_negative_miss_is_shorter() {
        assert_eq!(metadata_ttl_for_class(MetadataClass::NegativeMiss), NEGATIVE_MISS_TTL_SECS);
        assert_eq!(metadata_ttl_for_class(MetadataClass::Narinfo), DEFAULT_METADATA_TTL_SECS);
    }

    #[test]
    fn new_metadata_entry_sets_expiry_from_ttl() {
        let key = build_metadata_cache_key_from_candidate(&meta_candidate(), "digest", MetadataClass::Narinfo);
        let entry = new_metadata_entry(key.clone(), 1000, "probe ok".to_string());
        assert_eq!(entry.created_at_secs, 1000);
        assert_eq!(entry.expires_at_secs, 1000 + DEFAULT_METADATA_TTL_SECS);
        assert_eq!(entry.detail, "probe ok");
        assert_eq!(entry.key.output_digest, "digest");
    }

    #[test]
    fn check_metadata_validity_returns_fresh_for_matching_entry() {
        let key = build_metadata_cache_key_from_candidate(&meta_candidate(), "dig", MetadataClass::Narinfo);
        let entry = new_metadata_entry(key.clone(), 1000, String::new());
        let validity =
            check_metadata_validity(&entry, &key, 1000 + DEFAULT_METADATA_TTL_SECS - 1, RefreshPolicy::Normal);
        assert_eq!(validity, MetadataValidity::Fresh);
        assert!(validity.is_reusable());
    }

    #[test]
    fn check_metadata_validity_expired_when_past_expiry() {
        let key = build_metadata_cache_key_from_candidate(&meta_candidate(), "dig", MetadataClass::Narinfo);
        let entry = new_metadata_entry(key.clone(), 1000, String::new());
        let validity =
            check_metadata_validity(&entry, &key, 1000 + DEFAULT_METADATA_TTL_SECS + 1, RefreshPolicy::Normal);
        assert_eq!(validity, MetadataValidity::Expired);
        assert!(!validity.is_reusable());
    }

    #[test]
    fn check_metadata_validity_force_refresh_bypasses_fresh_entry() {
        let key = build_metadata_cache_key_from_candidate(&meta_candidate(), "dig", MetadataClass::Narinfo);
        let entry = new_metadata_entry(key.clone(), 1000, String::new());
        let validity = check_metadata_validity(&entry, &key, 1000, RefreshPolicy::ForceRefresh);
        assert_eq!(validity, MetadataValidity::ExplicitRefresh);
        assert!(!validity.is_reusable());
    }

    #[test]
    fn check_metadata_validity_detects_schema_mismatch() {
        let key = build_metadata_cache_key_from_candidate(&meta_candidate(), "dig", MetadataClass::Narinfo);
        let mut entry = new_metadata_entry(key.clone(), 1000, String::new());
        entry.key.schema_version = MetadataSchemaVersion(999);
        let validity = check_metadata_validity(&entry, &key, 1000, RefreshPolicy::Normal);
        assert_eq!(validity, MetadataValidity::SchemaMismatch);
    }

    #[test]
    fn check_metadata_validity_detects_trust_policy_mismatch() {
        let key = build_metadata_cache_key_from_candidate(&meta_candidate(), "dig", MetadataClass::Narinfo);
        let mut entry = new_metadata_entry(key.clone(), 1000, String::new());
        entry.key.trust_policy_digest = "0000000000000000000000000000000000000000000000000000000000000000".to_string();
        let validity = check_metadata_validity(&entry, &key, 1000, RefreshPolicy::Normal);
        assert_eq!(validity, MetadataValidity::TrustPolicyMismatch);
    }

    #[test]
    fn check_metadata_validity_detects_store_prefix_mismatch() {
        let mut key = build_metadata_cache_key_from_candidate(&meta_candidate(), "dig", MetadataClass::Narinfo);
        let entry = new_metadata_entry(key.clone(), 1000, String::new());
        key.store_prefix = "/different/store".to_string();
        let validity = check_metadata_validity(&entry, &key, 1000, RefreshPolicy::Normal);
        assert_eq!(validity, MetadataValidity::StorePrefixMismatch);
    }

    #[test]
    fn check_metadata_validity_detects_identity_mismatch() {
        let key = build_metadata_cache_key_from_candidate(&meta_candidate(), "dig", MetadataClass::Narinfo);
        let mut entry = new_metadata_entry(key.clone(), 1000, String::new());
        entry.key.cache_identity = "https://other.cache".to_string();
        let validity = check_metadata_validity(&entry, &key, 1000, RefreshPolicy::Normal);
        assert_eq!(validity, MetadataValidity::CacheIdentityMismatch);
    }

    #[test]
    fn check_metadata_validity_detects_output_digest_mismatch() {
        let key = build_metadata_cache_key_from_candidate(&meta_candidate(), "dig", MetadataClass::Narinfo);
        let mut entry = new_metadata_entry(key.clone(), 1000, String::new());
        entry.key.output_digest = "wrong-digest".to_string();
        let validity = check_metadata_validity(&entry, &key, 1000, RefreshPolicy::Normal);
        assert_eq!(validity, MetadataValidity::OutputDigestMismatch);
    }

    #[test]
    fn admission_summary_from_reason_does_not_claim_metadata_reuse() {
        let summary = AdmissionSummary::from_reason("miss");
        assert_eq!(summary.reason, "miss");
        assert!(!summary.metadata_reused);
        assert!(summary.metadata_validity.is_none());
    }

    #[test]
    fn admission_summary_with_reused_metadata_records_validity() {
        let summary = AdmissionSummary::with_reused_metadata("remote-hit", MetadataValidity::Fresh);
        assert_eq!(summary.reason, "remote-hit");
        assert!(summary.metadata_reused);
        assert_eq!(summary.metadata_validity, Some(MetadataValidity::Fresh));
    }

    #[test]
    fn metadata_validity_as_str_is_stable() {
        assert_eq!(MetadataValidity::Fresh.as_str(), "fresh");
        assert_eq!(MetadataValidity::Expired.as_str(), "expired");
        assert_eq!(MetadataValidity::ExplicitRefresh.as_str(), "explicit-refresh");
        assert_eq!(MetadataValidity::Missing.as_str(), "missing");
        assert_eq!(MetadataValidity::SchemaMismatch.as_str(), "schema-mismatch");
        assert_eq!(MetadataValidity::TrustPolicyMismatch.as_str(), "trust-policy-mismatch");
        assert_eq!(MetadataValidity::StorePrefixMismatch.as_str(), "store-prefix-mismatch");
        assert_eq!(MetadataValidity::CacheIdentityMismatch.as_str(), "cache-identity-mismatch");
        assert_eq!(MetadataValidity::OutputDigestMismatch.as_str(), "output-digest-mismatch");
    }
}
