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
    raw_urls: &[String],
    store_prefix: &str,
    trust_policy_digest: &str,
) -> Result<CacheCandidateSet, CacheCandidateError> {
    if raw_urls.len() > MAX_CACHE_CANDIDATES {
        return Err(CacheCandidateError::TooManyCandidates {
            actual: raw_urls.len(),
            limit: MAX_CACHE_CANDIDATES,
        });
    }
    validate_trust_policy_digest(trust_policy_digest)?;
    if !store_prefix.starts_with('/') {
        return Err(CacheCandidateError::InvalidStorePrefix(store_prefix.to_string()));
    }

    let mut candidates = Vec::with_capacity(raw_urls.len());
    let mut seen_identities = BTreeSet::new();

    for (index, raw_url) in raw_urls.iter().enumerate() {
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
        let requires_network = !identity.starts_with("file://");
        candidates.push(CacheCandidate {
            cache_identity: identity,
            priority: u32::try_from(index).unwrap_or(u32::MAX),
            trust_policy_digest: trust_policy_digest.to_string(),
            store_prefix: store_prefix.to_string(),
            requires_network,
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
    if parsed.host_str().is_none() {
        return None;
    }
    let mut sanitized = parsed.clone();
    sanitized.set_query(None);
    sanitized.set_fragment(None);
    sanitized.set_username("");
    sanitized.set_password(None);
    let port = sanitized.port().map(|p| format!(":{p}")).unwrap_or_default();
    let path = sanitized.path();
    if path == "/" {
        Some(format!("{}://{}{}", sanitized.scheme(), sanitized.host_str()?, port))
    } else {
        Some(format!("{}://{}{}{}", sanitized.scheme(), sanitized.host_str()?, port, path))
    }
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
        }
    }
}

impl std::error::Error for CacheCandidateError {}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_DIGEST: &str = "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
    const STORE_PREFIX: &str = "/mantle/store";

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
        let set = parse_cache_candidate_set(&urls, STORE_PREFIX, VALID_DIGEST).unwrap();
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
        let set = parse_cache_candidate_set(&urls, STORE_PREFIX, VALID_DIGEST).unwrap();
        assert_eq!(set.candidates.len(), 1);
    }

    #[test]
    fn parse_rejects_malformed_url() {
        let urls = vec!["not a url".to_string()];
        let err = parse_cache_candidate_set(&urls, STORE_PREFIX, VALID_DIGEST).unwrap_err();
        assert!(matches!(err, CacheCandidateError::MalformedUrl { index: 0, .. }));
    }

    #[test]
    fn parse_rejects_too_many_candidates() {
        let urls: Vec<String> =
            (0..MAX_CACHE_CANDIDATES + 1).map(|i| format!("https://cache-{i}.example.com")).collect();
        let err = parse_cache_candidate_set(&urls, STORE_PREFIX, VALID_DIGEST).unwrap_err();
        assert!(matches!(err, CacheCandidateError::TooManyCandidates { .. }));
    }

    #[test]
    fn parse_rejects_invalid_store_prefix() {
        let urls = vec!["https://cache.example.com".to_string()];
        let err = parse_cache_candidate_set(&urls, "relative/path", VALID_DIGEST).unwrap_err();
        assert!(matches!(err, CacheCandidateError::InvalidStorePrefix(_)));
    }

    #[test]
    fn parse_rejects_invalid_trust_policy_digest() {
        let urls = vec!["https://cache.example.com".to_string()];
        let err = parse_cache_candidate_set(&urls, STORE_PREFIX, "not-a-digest").unwrap_err();
        assert!(matches!(err, CacheCandidateError::InvalidTrustPolicyDigest));
    }

    #[test]
    fn parse_marks_file_url_as_not_requiring_network() {
        let urls = vec!["file:///tmp/cache".to_string()];
        let set = parse_cache_candidate_set(&urls, STORE_PREFIX, VALID_DIGEST).unwrap();
        assert!(!set.candidates[0].requires_network);
    }

    #[test]
    fn parse_marks_https_url_as_requiring_network() {
        let urls = vec!["https://cache.example.com".to_string()];
        let set = parse_cache_candidate_set(&urls, STORE_PREFIX, VALID_DIGEST).unwrap();
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
}
