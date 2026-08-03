//! Provider-neutral discovery, transfer, and publication for shared Rust results.
//!
//! Result sources expose a domain-specific candidate protocol. The local
//! `RustCache` remains the only component that admits castore content and
//! materializes compiler artifacts.

use std::collections::BTreeSet;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use crunch_rust_cache_core::LocalCachePolicy;
use crunch_rust_cache_core::LocalCandidateFacts;
use crunch_rust_cache_core::RustUnitAction;
use crunch_rust_cache_core::RustUnitResult;
use crunch_rust_cache_core::plan_local_reuse;
use crunch_rust_cache_core::shared::ExpectedRustResultRefs;
use crunch_rust_cache_core::shared::MAX_SHARED_ENVELOPE_BYTES;
use crunch_rust_cache_core::shared::RustResultObjectIdentity;
use crunch_rust_cache_core::shared::RustResultProducerIdentity;
use crunch_rust_cache_core::shared::RustResultTrustPolicy;
use crunch_rust_cache_core::shared::SHARED_RUST_ENVELOPE_REF_PREFIX;
use crunch_rust_cache_core::shared::SHARED_RUST_OBJECT_REF_PREFIX;
use crunch_rust_cache_core::shared::SignedRustResultEnvelope;
use crunch_rust_cache_core::shared::evaluate_rust_result_authority;
use crunch_rust_cache_core::shared::sign_rust_result_envelope;
use crunch_rust_cache_core::shared::validate_signed_rust_result_envelope;
use serde::Deserialize;
use serde::Serialize;
use snix_store::nar::ingest_nar_and_hash;
use snix_store::nar::write_nar;
use tempfile::Builder;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio_util::io::ReaderStream;

use crate::Error;
use crate::RustCache;
use crate::artifact_bytes;
use crate::bounded_count;
use crate::node_from_identity;
use crate::node_identity;
use crate::scan_output_artifacts;

pub const SHARED_CACHE_POLICY_SCHEMA: &str = "mantle-shared-rust-cache-policy-v1";
pub const SHARED_CANDIDATE_SCHEMA: &str = "mantle-shared-rust-candidate-v1";
pub const SHARED_CACHE_REPORT_SCHEMA: &str = "mantle-shared-rust-cache-report-v1";
pub const SHARED_CACHE_DISABLED: &str = "shared-cache-disabled";
pub const SHARED_CACHE_MISS: &str = "shared-cache-miss";
pub const SHARED_CACHE_OFFLINE_MISS: &str = "shared-cache-offline-miss";
pub const SHARED_CACHE_HIT: &str = "shared-cache-remote-hit";
pub const SHARED_CACHE_REJECTED: &str = "shared-cache-rejected";
pub const SHARED_CACHE_CONFLICT: &str = "shared-cache-conflict";
pub const MAX_SHARED_SOURCES: usize = 16;
pub const MAX_SHARED_CANDIDATES: usize = crunch_rust_cache_core::MAX_RESULT_CANDIDATES;
pub const MAX_SHARED_METADATA_BYTES: u64 = 16_777_216;
pub const MAX_SHARED_TRANSFER_BYTES: u64 = crunch_rust_cache_core::MAX_TREE_BYTES;
pub const MAX_SOURCE_ID_BYTES: usize = 256;

const SHARED_PROTOCOL_DIRECTORY: &str = "rust-unit-v1";
const OBJECT_DIRECTORY: &str = "objects";
const ENVELOPE_DIRECTORY: &str = "envelopes";
const ACTION_DIRECTORY: &str = "actions";
const JSON_SUFFIX: &str = ".json";
const NAR_SUFFIX: &str = ".nar";
const SOURCE_ID_PREFIX: &str = "mantle-rust-source://blake3/";
const SOURCE_ID_DOMAIN: &[u8] = b"mantle.shared-rust.source.v1";
const DOMAIN_SEPARATOR: u8 = 0;
const PRIVATE_DIRECTORY_MODE: u32 = 0o700;
const NO_FOLLOW_FLAG: i32 = libc::O_NOFOLLOW;
const COPY_BUFFER_BYTES: usize = 65_536;
const NAR_DUPLEX_BUFFER_BYTES: usize = 65_536;
const HTTP_STATUS_REDIRECT_MIN: u16 = 300;
const HTTP_STATUS_REDIRECT_MAX: u16 = 399;
const EXTRA_EOF_READ_ITERATIONS: u64 = 2;

#[derive(Clone, Copy)]
struct ValidationCode<'a>(&'a str);

#[derive(Clone, Copy)]
struct TypedRefRule<'a> {
    prefix: &'a str,
    code: ValidationCode<'a>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SharedRustCachePolicy {
    pub schema: String,
    pub policy_id: String,
    pub reads_enabled: bool,
    pub publishes_enabled: bool,
    pub offline: bool,
    pub execute_after_rejection: bool,
    pub block_on_conflict: bool,
    pub max_sources: u32,
    pub max_candidates: u32,
    pub max_metadata_bytes: u64,
    pub max_transfer_bytes: u64,
    pub max_redirects: u32,
    pub max_retries: u32,
}

impl Default for SharedRustCachePolicy {
    fn default() -> Self {
        Self {
            schema: SHARED_CACHE_POLICY_SCHEMA.to_string(),
            policy_id: "mantle-shared-rust-cache-default-v1".to_string(),
            reads_enabled: false,
            publishes_enabled: false,
            offline: false,
            execute_after_rejection: true,
            block_on_conflict: true,
            max_sources: MAX_SHARED_SOURCES as u32,
            max_candidates: MAX_SHARED_CANDIDATES as u32,
            max_metadata_bytes: MAX_SHARED_METADATA_BYTES,
            max_transfer_bytes: MAX_SHARED_TRANSFER_BYTES,
            max_redirects: 0,
            max_retries: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(deny_unknown_fields)]
pub struct SharedRustCandidateClaim {
    pub schema: String,
    pub action_ref: String,
    pub envelope_ref: String,
    pub result_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SharedSourceLookup {
    pub candidates: Vec<SharedRustCandidateClaim>,
    pub metadata_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SharedCandidateObservation {
    pub source_id: String,
    pub source_priority: u32,
    pub envelope_ref: Option<String>,
    pub result_ref: Option<String>,
    pub authority_disposition: String,
    pub accepted_verifier_blake3: Option<String>,
    pub reason_codes: Vec<String>,
    pub metadata_bytes: u64,
    pub transferred_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SharedRustCacheReport {
    pub schema: String,
    pub disposition: String,
    pub route: String,
    pub selected_source_id: Option<String>,
    pub selected_result_ref: Option<String>,
    pub candidate_count: u32,
    pub artifact_count: u32,
    pub transferred_bytes: u64,
    pub reused_bytes: u64,
    pub compiler_executed: bool,
    pub observations: Vec<SharedCandidateObservation>,
    pub publications: Vec<SharedPublicationObservation>,
    pub non_claims: Vec<String>,
}

impl SharedRustCacheReport {
    pub fn rejected(reason: &str) -> Self {
        let outcome = shared_rejection(reason);
        assert_eq!(outcome.disposition, SHARED_CACHE_REJECTED);
        assert!(!outcome.observations.is_empty());
        outcome
    }

    pub fn compiler_executed(mut self) -> Self {
        self.compiler_executed = true;
        assert!(self.compiler_executed);
        assert_ne!(self.disposition, SHARED_CACHE_HIT);
        self
    }
}

#[derive(Debug, Clone)]
pub struct SharedPublishRequest<'a> {
    pub result: &'a RustUnitResult,
    pub producer: RustResultProducerIdentity,
    pub signer_name: String,
    pub signing_key: &'a ed25519_dalek::SigningKey,
    pub policy: &'a SharedRustCachePolicy,
}

#[derive(Clone, Copy)]
pub struct SharedRestoreRequest<'a> {
    pub action: &'a RustUnitAction,
    pub output_dir: &'a Path,
    pub local_policy: &'a LocalCachePolicy,
    pub shared_policy: &'a SharedRustCachePolicy,
    pub trust_policy: &'a RustResultTrustPolicy,
    pub sources: &'a [Arc<dyn RustResultSource>],
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SharedPublishReport {
    pub source_id: String,
    pub result_ref: String,
    pub envelope_ref: String,
    pub object_ref: String,
    pub object_bytes: u64,
    pub publication_order: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SharedPublicationObservation {
    pub source_id: String,
    pub disposition: String,
    pub result_ref: Option<String>,
    pub envelope_ref: Option<String>,
    pub object_ref: Option<String>,
    pub object_bytes: u64,
    pub publication_order: Vec<String>,
    pub reason_codes: Vec<String>,
}

impl SharedPublicationObservation {
    pub fn accepted(report: SharedPublishReport) -> Self {
        let observation = Self {
            source_id: report.source_id,
            disposition: "published".to_string(),
            result_ref: Some(report.result_ref),
            envelope_ref: Some(report.envelope_ref),
            object_ref: Some(report.object_ref),
            object_bytes: report.object_bytes,
            publication_order: report.publication_order,
            reason_codes: Vec::new(),
        };
        assert!(observation.reason_codes.is_empty());
        assert_eq!(observation.publication_order, ["object", "envelope", "candidate"]);
        observation
    }

    pub fn rejected(source_id: String, reason: &str) -> Self {
        Self::rejected_with_result(source_id, None, reason)
    }

    pub fn rejected_for_result(source_id: String, result_ref: String, reason: &str) -> Self {
        Self::rejected_with_result(source_id, Some(result_ref), reason)
    }

    fn rejected_with_result(source_id: String, result_ref: Option<String>, reason: &str) -> Self {
        let observation = Self {
            source_id,
            disposition: "publication-rejected".to_string(),
            result_ref,
            envelope_ref: None,
            object_ref: None,
            object_bytes: 0,
            publication_order: Vec::new(),
            reason_codes: vec![reason.to_string()],
        };
        assert!(!observation.reason_codes.is_empty());
        assert!(observation.envelope_ref.is_none());
        observation
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceError {
    pub code: String,
}

impl SourceError {
    pub fn new(code: impl Into<String>) -> Self {
        let code = code.into();
        assert!(!code.is_empty());
        assert!(code.len() <= MAX_SOURCE_ID_BYTES);
        Self { code }
    }
}

impl std::fmt::Display for SourceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.code)
    }
}

impl std::error::Error for SourceError {}

#[async_trait]
pub trait RustResultSource: Send + Sync {
    fn source_id(&self) -> &str;
    fn is_remote(&self) -> bool;

    async fn lookup(
        &self,
        action_ref: &str,
        max_candidates: u32,
        max_metadata_bytes: u64,
    ) -> Result<SharedSourceLookup, SourceError>;

    async fn fetch_envelope(&self, envelope_ref: &str, max_metadata_bytes: u64) -> Result<Vec<u8>, SourceError>;

    async fn fetch_object(
        &self,
        object_ref: &str,
        destination: &Path,
        max_transfer_bytes: u64,
    ) -> Result<u64, SourceError>;

    async fn publish_object(&self, object_ref: &str, source: &Path, size_bytes: u64) -> Result<(), SourceError>;

    async fn publish_envelope(&self, envelope_ref: &str, bytes: &[u8]) -> Result<(), SourceError>;

    async fn publish_candidate(&self, candidate: &SharedRustCandidateClaim) -> Result<(), SourceError>;
}

#[derive(Debug, Clone)]
pub struct DirectoryRustResultSource {
    root: PathBuf,
    protocol_root: PathBuf,
    source_id: String,
    remote: bool,
}

impl DirectoryRustResultSource {
    pub fn open(root: PathBuf, remote: bool) -> Result<Self, Error> {
        create_private_directory(&root)?;
        reject_symlink(&root)?;
        let canonical = fs::canonicalize(&root).map_err(|source| Error::Io {
            context: "canonicalize-shared-source".to_string(),
            source,
        })?;
        let protocol_root = canonical.join(SHARED_PROTOCOL_DIRECTORY);
        for directory in [
            protocol_root.clone(),
            protocol_root.join(OBJECT_DIRECTORY),
            protocol_root.join(ENVELOPE_DIRECTORY),
            protocol_root.join(ACTION_DIRECTORY),
        ] {
            create_private_directory(&directory)?;
        }
        let source_id = sanitized_source_id(&canonical);
        assert!(protocol_root.starts_with(&canonical));
        assert!(source_id.starts_with(SOURCE_ID_PREFIX));
        Ok(Self {
            root: canonical,
            protocol_root,
            source_id,
            remote,
        })
    }

    fn object_path(&self, object_ref: &str) -> Result<PathBuf, SourceError> {
        let digest = typed_ref_digest(object_ref, TypedRefRule {
            prefix: SHARED_RUST_OBJECT_REF_PREFIX,
            code: ValidationCode("shared-object-ref-invalid"),
        })?;
        Ok(self.protocol_root.join(OBJECT_DIRECTORY).join(format!("{digest}{NAR_SUFFIX}")))
    }

    fn envelope_path(&self, envelope_ref: &str) -> Result<PathBuf, SourceError> {
        let digest = typed_ref_digest(envelope_ref, TypedRefRule {
            prefix: SHARED_RUST_ENVELOPE_REF_PREFIX,
            code: ValidationCode("shared-envelope-ref-invalid"),
        })?;
        Ok(self.protocol_root.join(ENVELOPE_DIRECTORY).join(format!("{digest}{JSON_SUFFIX}")))
    }

    fn action_path(&self, action_ref: &str) -> Result<PathBuf, SourceError> {
        let digest = typed_ref_digest(action_ref, TypedRefRule {
            prefix: crunch_rust_cache_core::RUST_ACTION_REF_PREFIX,
            code: ValidationCode("shared-action-ref-invalid"),
        })?;
        Ok(self.protocol_root.join(ACTION_DIRECTORY).join(digest))
    }

    fn candidate_path(&self, candidate: &SharedRustCandidateClaim) -> Result<PathBuf, SourceError> {
        let action_dir = self.action_path(&candidate.action_ref)?;
        let digest = typed_ref_digest(&candidate.envelope_ref, TypedRefRule {
            prefix: SHARED_RUST_ENVELOPE_REF_PREFIX,
            code: ValidationCode("shared-envelope-ref-invalid"),
        })?;
        Ok(action_dir.join(format!("{digest}{JSON_SUFFIX}")))
    }
}

#[async_trait]
impl RustResultSource for DirectoryRustResultSource {
    fn source_id(&self) -> &str {
        assert!(self.source_id.starts_with(SOURCE_ID_PREFIX));
        assert!(!self.root.as_os_str().is_empty());
        &self.source_id
    }

    fn is_remote(&self) -> bool {
        assert!(!self.source_id.is_empty());
        assert!(self.protocol_root.starts_with(&self.root));
        self.remote
    }

    async fn lookup(
        &self,
        action_ref: &str,
        max_candidates: u32,
        max_metadata_bytes: u64,
    ) -> Result<SharedSourceLookup, SourceError> {
        let action_dir = self.action_path(action_ref)?;
        let Some(entries) = read_dir_optional(&action_dir)? else {
            return Ok(SharedSourceLookup {
                candidates: Vec::new(),
                metadata_bytes: 0,
            });
        };
        load_candidate_entries(entries, action_ref, max_candidates, max_metadata_bytes)
    }

    async fn fetch_envelope(&self, envelope_ref: &str, max_metadata_bytes: u64) -> Result<Vec<u8>, SourceError> {
        let path = self.envelope_path(envelope_ref)?;
        read_source_file_bounded(&path, max_metadata_bytes, "shared-envelope")
    }

    async fn fetch_object(
        &self,
        object_ref: &str,
        destination: &Path,
        max_transfer_bytes: u64,
    ) -> Result<u64, SourceError> {
        let source = self.object_path(object_ref)?;
        copy_source_file_bounded(&source, destination, max_transfer_bytes).await
    }

    async fn publish_object(&self, object_ref: &str, source: &Path, size_bytes: u64) -> Result<(), SourceError> {
        let destination = self.object_path(object_ref)?;
        publish_immutable_file(&destination, source, size_bytes).await
    }

    async fn publish_envelope(&self, envelope_ref: &str, bytes: &[u8]) -> Result<(), SourceError> {
        let destination = self.envelope_path(envelope_ref)?;
        publish_immutable_bytes(&destination, bytes)
    }

    async fn publish_candidate(&self, candidate: &SharedRustCandidateClaim) -> Result<(), SourceError> {
        validate_candidate_claim(candidate)?;
        let action_dir = self.action_path(&candidate.action_ref)?;
        create_source_directory(&action_dir)?;
        let destination = self.candidate_path(candidate)?;
        let bytes = serde_json::to_vec(candidate).map_err(|_| SourceError::new("shared-candidate-json"))?;
        publish_immutable_bytes(&destination, &bytes)
    }
}

#[derive(Debug, Clone)]
pub struct HttpRustResultSource {
    base_url: url::Url,
    source_id: String,
    client: reqwest::Client,
}

impl HttpRustResultSource {
    pub fn new(base_url: &str, timeout: Duration) -> Result<Self, Error> {
        let mut parsed =
            url::Url::parse(base_url).map_err(|_| Error::State("shared-http-source-url-invalid".to_string()))?;
        if !matches!(parsed.scheme(), "http" | "https") {
            return Err(Error::State("shared-http-source-scheme-unsupported".to_string()));
        }
        if !parsed.username().is_empty() || parsed.password().is_some() {
            return Err(Error::State("shared-http-source-userinfo-rejected".to_string()));
        }
        if parsed.query().is_some() || parsed.fragment().is_some() {
            return Err(Error::State("shared-http-source-query-rejected".to_string()));
        }
        if timeout.is_zero() {
            return Err(Error::Bound("shared-http-timeout-invalid".to_string()));
        }
        parsed.set_query(None);
        parsed.set_fragment(None);
        let source_id = sanitized_source_bytes(parsed.as_str().as_bytes());
        let client = reqwest::Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(timeout)
            .build()
            .map_err(|_| Error::State("shared-http-client-build-failed".to_string()))?;
        assert!(source_id.starts_with(SOURCE_ID_PREFIX));
        assert!(matches!(parsed.scheme(), "http" | "https"));
        Ok(Self {
            base_url: parsed,
            source_id,
            client,
        })
    }

    fn endpoint(&self, class: &str, digest: &str, suffix: &str) -> Result<url::Url, SourceError> {
        validate_digest(digest, ValidationCode("shared-http-endpoint-digest-invalid"))?;
        let base_path = self.base_url.path().trim_end_matches('/');
        let path = format!("{base_path}/{SHARED_PROTOCOL_DIRECTORY}/{class}/{digest}{suffix}");
        let mut endpoint = self.base_url.clone();
        endpoint.set_path(&path);
        endpoint.set_query(None);
        endpoint.set_fragment(None);
        assert!(endpoint.as_str().contains(SHARED_PROTOCOL_DIRECTORY));
        assert!(endpoint.query().is_none());
        Ok(endpoint)
    }

    fn action_endpoint(&self, action_ref: &str) -> Result<url::Url, SourceError> {
        let digest = typed_ref_digest(action_ref, TypedRefRule {
            prefix: crunch_rust_cache_core::RUST_ACTION_REF_PREFIX,
            code: ValidationCode("shared-action-ref-invalid"),
        })?;
        self.endpoint(ACTION_DIRECTORY, digest, JSON_SUFFIX)
    }

    fn envelope_endpoint(&self, envelope_ref: &str) -> Result<url::Url, SourceError> {
        let digest = typed_ref_digest(envelope_ref, TypedRefRule {
            prefix: SHARED_RUST_ENVELOPE_REF_PREFIX,
            code: ValidationCode("shared-envelope-ref-invalid"),
        })?;
        self.endpoint(ENVELOPE_DIRECTORY, digest, JSON_SUFFIX)
    }

    fn object_endpoint(&self, object_ref: &str) -> Result<url::Url, SourceError> {
        let digest = typed_ref_digest(object_ref, TypedRefRule {
            prefix: SHARED_RUST_OBJECT_REF_PREFIX,
            code: ValidationCode("shared-object-ref-invalid"),
        })?;
        self.endpoint(OBJECT_DIRECTORY, digest, NAR_SUFFIX)
    }

    fn candidate_endpoint(&self, candidate: &SharedRustCandidateClaim) -> Result<url::Url, SourceError> {
        let action_digest = typed_ref_digest(&candidate.action_ref, TypedRefRule {
            prefix: crunch_rust_cache_core::RUST_ACTION_REF_PREFIX,
            code: ValidationCode("shared-action-ref-invalid"),
        })?;
        let envelope_digest = typed_ref_digest(&candidate.envelope_ref, TypedRefRule {
            prefix: SHARED_RUST_ENVELOPE_REF_PREFIX,
            code: ValidationCode("shared-envelope-ref-invalid"),
        })?;
        let class = format!("{ACTION_DIRECTORY}/{action_digest}");
        self.endpoint(&class, envelope_digest, JSON_SUFFIX)
    }

    async fn get(&self, endpoint: url::Url) -> Result<reqwest::Response, SourceError> {
        self.client.get(endpoint).send().await.map_err(classify_http_error)
    }

    async fn put_bytes(&self, endpoint: url::Url, bytes: &[u8]) -> Result<(), SourceError> {
        if bytes.is_empty() || bytes.len() as u64 > MAX_SHARED_METADATA_BYTES {
            return Err(SourceError::new("shared-http-publish-metadata-size-invalid"));
        }
        let put_endpoint = endpoint.clone();
        let response = self
            .client
            .put(put_endpoint)
            .header(reqwest::header::IF_NONE_MATCH, "*")
            .body(bytes.to_vec())
            .send()
            .await
            .map_err(classify_http_error)?;
        if response.status() == reqwest::StatusCode::PRECONDITION_FAILED {
            let existing = self.get(endpoint).await?;
            require_http_success(existing.status(), "shared-http-existing-metadata")?;
            let existing = bounded_http_bytes(existing, MAX_SHARED_METADATA_BYTES).await?;
            if existing != bytes {
                return Err(SourceError::new("shared-http-immutable-conflict"));
            }
            return Ok(());
        }
        require_http_success(response.status(), "shared-http-publish")?;
        assert!(!bytes.is_empty());
        assert!(bytes.len() as u64 <= MAX_SHARED_METADATA_BYTES);
        Ok(())
    }
}

#[async_trait]
impl RustResultSource for HttpRustResultSource {
    fn source_id(&self) -> &str {
        assert!(self.source_id.starts_with(SOURCE_ID_PREFIX));
        assert!(!self.base_url.as_str().is_empty());
        &self.source_id
    }

    fn is_remote(&self) -> bool {
        assert!(matches!(self.base_url.scheme(), "http" | "https"));
        assert!(!self.source_id.is_empty());
        true
    }

    async fn lookup(
        &self,
        action_ref: &str,
        max_candidates: u32,
        max_metadata_bytes: u64,
    ) -> Result<SharedSourceLookup, SourceError> {
        let response = self.get(self.action_endpoint(action_ref)?).await?;
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(SharedSourceLookup {
                candidates: Vec::new(),
                metadata_bytes: 0,
            });
        }
        require_http_success(response.status(), "shared-http-lookup")?;
        let bytes = bounded_http_bytes(response, max_metadata_bytes).await?;
        let mut lookup = serde_json::from_slice::<SharedSourceLookup>(&bytes)
            .map_err(|_| SourceError::new("shared-http-lookup-malformed"))?;
        validate_lookup_candidates(&lookup.candidates, action_ref, max_candidates)?;
        lookup.candidates.sort();
        lookup.candidates.dedup();
        let observed_metadata_bytes =
            u64::try_from(bytes.len()).map_err(|_| SourceError::new("shared-http-metadata-size-unrepresentable"))?;
        lookup.metadata_bytes = observed_metadata_bytes;
        let max_candidates =
            usize::try_from(max_candidates).map_err(|_| SourceError::new("shared-candidate-limit-unrepresentable"))?;
        assert!(lookup.candidates.len() <= max_candidates);
        assert_eq!(lookup.metadata_bytes, observed_metadata_bytes);
        Ok(lookup)
    }

    async fn fetch_envelope(&self, envelope_ref: &str, max_metadata_bytes: u64) -> Result<Vec<u8>, SourceError> {
        let response = self.get(self.envelope_endpoint(envelope_ref)?).await?;
        require_http_success(response.status(), "shared-http-envelope")?;
        bounded_http_bytes(response, max_metadata_bytes).await
    }

    async fn fetch_object(
        &self,
        object_ref: &str,
        destination: &Path,
        max_transfer_bytes: u64,
    ) -> Result<u64, SourceError> {
        let response = self.get(self.object_endpoint(object_ref)?).await?;
        require_http_success(response.status(), "shared-http-object")?;
        stream_http_object(response, destination, max_transfer_bytes).await
    }

    async fn publish_object(&self, object_ref: &str, source: &Path, size_bytes: u64) -> Result<(), SourceError> {
        if size_bytes == 0 || size_bytes > MAX_SHARED_TRANSFER_BYTES {
            return Err(SourceError::new("shared-http-object-size-invalid"));
        }
        let file = tokio::fs::File::open(source)
            .await
            .map_err(|_| SourceError::new("shared-http-object-open-failed"))?;
        let stream = ReaderStream::new(file.take(size_bytes));
        let body = reqwest::Body::wrap_stream(stream);
        let endpoint = self.object_endpoint(object_ref)?;
        let response = self
            .client
            .put(endpoint.clone())
            .header(reqwest::header::IF_NONE_MATCH, "*")
            .header(reqwest::header::CONTENT_LENGTH, size_bytes)
            .body(body)
            .send()
            .await
            .map_err(classify_http_error)?;
        if response.status() == reqwest::StatusCode::PRECONDITION_FAILED {
            let existing = self.get(endpoint).await?;
            require_http_success(existing.status(), "shared-http-existing-object")?;
            let staged = tempfile::NamedTempFile::new()
                .map_err(|_| SourceError::new("shared-http-existing-object-staging-failed"))?;
            let existing_size_bytes = stream_http_object(existing, staged.path(), size_bytes).await?;
            if existing_size_bytes != size_bytes {
                return Err(SourceError::new("shared-http-immutable-conflict"));
            }
            compare_existing_file(staged.path(), source, size_bytes)?;
            return Ok(());
        }
        require_http_success(response.status(), "shared-http-publish-object")?;
        assert!(size_bytes > 0);
        assert!(size_bytes <= MAX_SHARED_TRANSFER_BYTES);
        Ok(())
    }

    async fn publish_envelope(&self, envelope_ref: &str, bytes: &[u8]) -> Result<(), SourceError> {
        self.put_bytes(self.envelope_endpoint(envelope_ref)?, bytes).await
    }

    async fn publish_candidate(&self, candidate: &SharedRustCandidateClaim) -> Result<(), SourceError> {
        validate_candidate_claim(candidate)?;
        let bytes = serde_json::to_vec(candidate).map_err(|_| SourceError::new("shared-http-candidate-json"))?;
        self.put_bytes(self.candidate_endpoint(candidate)?, &bytes).await
    }
}

async fn bounded_http_bytes(mut response: reqwest::Response, max_bytes: u64) -> Result<Vec<u8>, SourceError> {
    let declared_bytes = response.content_length();
    if declared_bytes.is_some_and(|size| size > max_bytes) {
        return Err(SourceError::new("shared-http-metadata-too-large"));
    }
    let capacity_bytes = declared_bytes
        .map(|size| usize::try_from(size).map_err(|_| SourceError::new("shared-http-metadata-too-large")))
        .transpose()?
        .unwrap_or(0);
    let mut bytes = Vec::with_capacity(capacity_bytes);
    while let Some(chunk) = response.chunk().await.map_err(classify_http_error)? {
        let next = bytes
            .len()
            .checked_add(chunk.len())
            .ok_or_else(|| SourceError::new("shared-http-metadata-overflow"))?;
        if next as u64 > max_bytes {
            return Err(SourceError::new("shared-http-metadata-too-large"));
        }
        bytes.extend_from_slice(&chunk);
    }
    if declared_bytes.is_some_and(|size| size != bytes.len() as u64) {
        return Err(SourceError::new("shared-http-metadata-truncated"));
    }
    assert!(bytes.len() as u64 <= max_bytes);
    assert!(declared_bytes.is_none_or(|size| size == bytes.len() as u64));
    Ok(bytes)
}

async fn stream_http_object(
    mut response: reqwest::Response,
    destination: &Path,
    max_bytes: u64,
) -> Result<u64, SourceError> {
    let declared_bytes = response.content_length();
    if declared_bytes.is_some_and(|size| size > max_bytes) {
        return Err(SourceError::new("shared-http-object-too-large"));
    }
    let mut file = tokio::fs::File::create(destination)
        .await
        .map_err(|_| SourceError::new("shared-http-object-staging-open-failed"))?;
    let mut total = 0_u64;
    while let Some(chunk) = response.chunk().await.map_err(classify_http_error)? {
        total = total
            .checked_add(chunk.len() as u64)
            .ok_or_else(|| SourceError::new("shared-http-object-overflow"))?;
        if total > max_bytes {
            return Err(SourceError::new("shared-http-object-too-large"));
        }
        file.write_all(&chunk).await.map_err(|_| SourceError::new("shared-http-object-write-failed"))?;
    }
    file.sync_all().await.map_err(|_| SourceError::new("shared-http-object-sync-failed"))?;
    if declared_bytes.is_some_and(|size| size != total) {
        return Err(SourceError::new("shared-http-object-truncated"));
    }
    assert!(total <= max_bytes);
    assert!(declared_bytes.is_none_or(|size| size == total));
    Ok(total)
}

fn validate_lookup_candidates(
    candidates: &[SharedRustCandidateClaim],
    action_ref: &str,
    max_candidates: u32,
) -> Result<(), SourceError> {
    let max_candidates =
        usize::try_from(max_candidates).map_err(|_| SourceError::new("shared-candidate-limit-unrepresentable"))?;
    if candidates.len() > max_candidates {
        return Err(SourceError::new("shared-candidate-limit-exceeded"));
    }
    for candidate in candidates {
        validate_candidate_claim(candidate)?;
        if candidate.action_ref != action_ref {
            return Err(SourceError::new("shared-candidate-action-mismatch"));
        }
    }
    assert!(candidates.len() <= max_candidates);
    assert!(candidates.iter().all(|candidate| candidate.action_ref == action_ref));
    Ok(())
}

fn require_http_success(status: reqwest::StatusCode, class: &str) -> Result<(), SourceError> {
    let status_code = status.as_u16();
    if (HTTP_STATUS_REDIRECT_MIN..=HTTP_STATUS_REDIRECT_MAX).contains(&status_code) {
        return Err(SourceError::new(format!("{class}-redirect-rejected")));
    }
    if !status.is_success() {
        return Err(SourceError::new(format!("{class}-status-{status_code}")));
    }
    assert!(status.is_success());
    assert!(!(HTTP_STATUS_REDIRECT_MIN..=HTTP_STATUS_REDIRECT_MAX).contains(&status_code));
    Ok(())
}

fn classify_http_error(error: reqwest::Error) -> SourceError {
    if error.is_timeout() {
        return SourceError::new("shared-http-timeout");
    }
    if error.is_connect() {
        return SourceError::new("shared-http-connect-failed");
    }
    SourceError::new("shared-http-transport-failed")
}

fn validate_digest(digest: &str, code: ValidationCode<'_>) -> Result<(), SourceError> {
    if digest.len() != crunch_rust_cache_core::BLAKE3_HEX_CHARS
        || !digest.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(SourceError::new(code.0));
    }
    assert_eq!(digest.len(), crunch_rust_cache_core::BLAKE3_HEX_CHARS);
    assert!(digest.bytes().all(|byte| !byte.is_ascii_uppercase()));
    Ok(())
}

struct AdmittedCandidate {
    source_id: String,
    source_priority: u32,
    signed: SignedRustResultEnvelope,
    transferred_bytes: u64,
    accepted_verifier_blake3: String,
}

struct SharedDiscoveryContext<'a> {
    action: &'a RustUnitAction,
    local_policy: &'a LocalCachePolicy,
    shared_policy: &'a SharedRustCachePolicy,
    trust_policy: &'a RustResultTrustPolicy,
}

struct SharedDiscoveryState {
    seen_envelopes: BTreeSet<String>,
    admitted: Vec<AdmittedCandidate>,
    observations: Vec<SharedCandidateObservation>,
}

#[derive(Clone, Copy)]
struct CandidateSourceFacts<'a> {
    source: &'a dyn RustResultSource,
    priority: usize,
    lookup_bytes: u64,
}

impl RustCache {
    pub fn restore_shared_blocking(&self, request: SharedRestoreRequest<'_>) -> Result<SharedRustCacheReport, Error> {
        let runtime =
            tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|source| Error::Io {
                context: "create-shared-restore-runtime".to_string(),
                source,
            })?;
        let outcome = runtime.block_on(self.restore_shared(request))?;
        assert!(!outcome.disposition.is_empty());
        assert!(outcome.candidate_count <= request.shared_policy.max_candidates);
        Ok(outcome)
    }

    pub fn publish_shared_blocking(
        &self,
        source: &dyn RustResultSource,
        request: SharedPublishRequest<'_>,
    ) -> Result<SharedPublishReport, Error> {
        let runtime =
            tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|source| Error::Io {
                context: "create-shared-publish-runtime".to_string(),
                source,
            })?;
        let publication_outcome = runtime.block_on(self.publish_shared(source, request))?;
        assert!(!publication_outcome.envelope_ref.is_empty());
        assert!(!publication_outcome.publication_order.is_empty());
        Ok(publication_outcome)
    }

    pub async fn restore_shared(&self, request: SharedRestoreRequest<'_>) -> Result<SharedRustCacheReport, Error> {
        validate_shared_inputs(request.action, request.local_policy, request.shared_policy, request.sources)?;
        if !request.shared_policy.reads_enabled {
            return Ok(shared_report(SharedReportKind {
                disposition: SHARED_CACHE_DISABLED,
                route: "disabled",
            }));
        }
        if request.output_dir.exists() {
            return Ok(shared_rejection("shared-cache-output-exists"));
        }
        let discovery = SharedDiscoveryContext {
            action: request.action,
            local_policy: request.local_policy,
            shared_policy: request.shared_policy,
            trust_policy: request.trust_policy,
        };
        let (admitted, observations) = self.discover_admitted(&discovery, request.sources).await?;
        self.finish_shared_restore(request.output_dir, request.local_policy, admitted, observations).await
    }

    pub async fn publish_shared(
        &self,
        source: &dyn RustResultSource,
        request: SharedPublishRequest<'_>,
    ) -> Result<SharedPublishReport, Error> {
        validate_shared_cache_policy(request.policy)?;
        crunch_rust_cache_core::validate_rust_result(request.result).map_err(Error::Core)?;
        if !request.policy.publishes_enabled {
            return Err(Error::State("shared-cache-publication-disabled".to_string()));
        }
        if request.policy.offline && source.is_remote() {
            return Err(Error::State("shared-cache-offline-publication-rejected".to_string()));
        }
        let staged = self.render_shared_object(request.result).await?;
        let signed = sign_rust_result_envelope(
            request.result.clone(),
            staged.identity.clone(),
            request.producer,
            request.signer_name,
            request.signing_key,
        )
        .map_err(Error::Core)?;
        let envelope_bytes =
            serde_json::to_vec(&signed).map_err(|error| Error::Json(format!("shared-envelope-encode:{error}")))?;
        let candidate = candidate_for_envelope(&signed)?;
        publish_ordered(source, &staged, &signed, &envelope_bytes, &candidate).await?;
        Ok(SharedPublishReport {
            source_id: source.source_id().to_string(),
            result_ref: request.result.result_ref.clone(),
            envelope_ref: signed.envelope.envelope_ref,
            object_ref: staged.identity.object_ref,
            object_bytes: staged.identity.size_bytes,
            publication_order: vec!["object".to_string(), "envelope".to_string(), "candidate".to_string()],
        })
    }

    async fn discover_admitted(
        &self,
        context: &SharedDiscoveryContext<'_>,
        sources: &[Arc<dyn RustResultSource>],
    ) -> Result<(Vec<AdmittedCandidate>, Vec<SharedCandidateObservation>), Error> {
        let max_candidates = usize::try_from(context.shared_policy.max_candidates)
            .map_err(|_| Error::Bound("shared-candidate-limit-unrepresentable".to_string()))?;
        let observations_per_source = max_candidates
            .checked_add(1)
            .ok_or_else(|| Error::Bound("shared-observation-limit-overflow".to_string()))?;
        let max_observations = sources
            .len()
            .checked_mul(observations_per_source)
            .ok_or_else(|| Error::Bound("shared-observation-limit-overflow".to_string()))?;
        let mut state = SharedDiscoveryState {
            seen_envelopes: BTreeSet::new(),
            admitted: Vec::with_capacity(max_candidates),
            observations: Vec::with_capacity(max_observations),
        };
        for (priority, source) in sources.iter().enumerate() {
            if context.shared_policy.offline && source.is_remote() {
                state.observations.push(offline_observation(source.source_id(), priority)?);
                continue;
            }
            self.discover_from_source(context, source.as_ref(), priority, &mut state).await?;
        }
        assert!(state.admitted.len() <= max_candidates);
        assert!(state.observations.len() <= max_observations);
        Ok((state.admitted, state.observations))
    }

    async fn discover_from_source(
        &self,
        context: &SharedDiscoveryContext<'_>,
        source: &dyn RustResultSource,
        priority: usize,
        state: &mut SharedDiscoveryState,
    ) -> Result<(), Error> {
        let lookup = match source
            .lookup(
                &context.action.action_ref,
                context.shared_policy.max_candidates,
                context.shared_policy.max_metadata_bytes,
            )
            .await
        {
            Ok(lookup) => lookup,
            Err(error) => {
                state.observations.push(source_error_observation(source.source_id(), priority, error.code)?);
                return Ok(());
            }
        };
        let max_candidates = usize::try_from(context.shared_policy.max_candidates)
            .map_err(|_| Error::Bound("shared-candidate-limit-unrepresentable".to_string()))?;
        for claim in lookup.candidates {
            if state.admitted.len() >= max_candidates {
                state.observations.push(source_error_observation(
                    source.source_id(),
                    priority,
                    "shared-candidate-limit-exceeded".to_string(),
                )?);
                break;
            }
            if !state.seen_envelopes.insert(claim.envelope_ref.clone()) {
                continue;
            }
            self.evaluate_source_candidate(
                context,
                CandidateSourceFacts {
                    source,
                    priority,
                    lookup_bytes: lookup.metadata_bytes,
                },
                claim,
                state,
            )
            .await?;
        }
        Ok(())
    }

    async fn evaluate_source_candidate(
        &self,
        context: &SharedDiscoveryContext<'_>,
        source_facts: CandidateSourceFacts<'_>,
        claim: SharedRustCandidateClaim,
        state: &mut SharedDiscoveryState,
    ) -> Result<(), Error> {
        let envelope_bytes = match source_facts
            .source
            .fetch_envelope(&claim.envelope_ref, context.shared_policy.max_metadata_bytes)
            .await
        {
            Ok(bytes) => bytes,
            Err(error) => {
                state.observations.push(candidate_error_observation(
                    source_facts.source.source_id(),
                    source_facts.priority,
                    &claim,
                    error.code,
                )?);
                return Ok(());
            }
        };
        let signed = match decode_candidate_envelope(&envelope_bytes, &claim) {
            Ok(signed) => signed,
            Err(reason) => {
                state.observations.push(candidate_error_observation(
                    source_facts.source.source_id(),
                    source_facts.priority,
                    &claim,
                    reason,
                )?);
                return Ok(());
            }
        };
        let authority = evaluate_rust_result_authority(&signed, context.trust_policy, ExpectedRustResultRefs {
            action_ref: &context.action.action_ref,
            result_ref: &claim.result_ref,
        });
        let envelope_size_bytes = u64::try_from(envelope_bytes.len())
            .map_err(|_| Error::Bound("shared-envelope-size-unrepresentable".to_string()))?;
        let metadata_bytes = envelope_size_bytes
            .checked_add(source_facts.lookup_bytes)
            .ok_or_else(|| Error::Bound("shared-metadata-byte-overflow".to_string()))?;
        if !authority.admitted {
            state.observations.push(authority_observation(AuthorityObservationInput {
                source_id: source_facts.source.source_id(),
                priority: source_facts.priority,
                claim: &claim,
                authority: &authority,
                metadata_bytes,
                transferred_bytes: 0,
            })?);
            return Ok(());
        }
        let transferred_bytes = match self
            .ensure_candidate_content(source_facts.source, &signed, context.local_policy, context.shared_policy)
            .await
        {
            Ok(bytes) => bytes,
            Err(error) => {
                state.observations.push(candidate_error_observation(
                    source_facts.source.source_id(),
                    source_facts.priority,
                    &claim,
                    stable_cache_error(&error),
                )?);
                return Ok(());
            }
        };
        state.observations.push(authority_observation(AuthorityObservationInput {
            source_id: source_facts.source.source_id(),
            priority: source_facts.priority,
            claim: &claim,
            authority: &authority,
            metadata_bytes,
            transferred_bytes,
        })?);
        let accepted_verifier_blake3 = authority
            .accepted_verifier_blake3
            .ok_or_else(|| Error::State("admitted-shared-verifier-missing".to_string()))?;
        state.admitted.push(AdmittedCandidate {
            source_id: source_facts.source.source_id().to_string(),
            source_priority: u32::try_from(source_facts.priority)
                .map_err(|_| Error::Bound("source-priority-overflow".to_string()))?,
            signed,
            transferred_bytes,
            accepted_verifier_blake3,
        });
        Ok(())
    }

    async fn ensure_candidate_content(
        &self,
        source: &dyn RustResultSource,
        signed: &SignedRustResultEnvelope,
        local_policy: &LocalCachePolicy,
        shared_policy: &SharedRustCachePolicy,
    ) -> Result<u64, Error> {
        let node = node_from_identity(&signed.envelope.result.input.root_node)?;
        let is_complete =
            crunch_store::recursive_castore_completeness(&*self.blob_service, &*self.directory_service, &node)
                .await
                .map_err(|error| Error::Castore(format!("shared-completeness:{error}")))?;
        let transferred = if is_complete {
            0
        } else {
            self.fetch_and_ingest_object(source, signed, shared_policy).await?
        };
        self.verify_shared_manifest(&signed.envelope.result, local_policy).await?;
        assert!(transferred <= shared_policy.max_transfer_bytes);
        assert!(!signed.envelope.result.input.artifacts.is_empty());
        Ok(transferred)
    }

    async fn fetch_and_ingest_object(
        &self,
        source: &dyn RustResultSource,
        signed: &SignedRustResultEnvelope,
        shared_policy: &SharedRustCachePolicy,
    ) -> Result<u64, Error> {
        let staged =
            Builder::new().prefix("shared-object-").tempfile_in(&self.staging_dir).map_err(|source| Error::Io {
                context: "create-shared-object-staging".to_string(),
                source,
            })?;
        let bytes = source
            .fetch_object(&signed.envelope.object.object_ref, staged.path(), shared_policy.max_transfer_bytes)
            .await
            .map_err(|error| Error::State(format!("shared-object-fetch:{}", error.code)))?;
        verify_staged_object(staged.path(), &signed.envelope.object)?;
        let _store_guard = crunch_store::StoreMutationGuard::acquire_wait(&self.state_dir)
            .map_err(|error| Error::State(format!("store-mutation-lock:{error}")))?;
        let mut reader = tokio::fs::File::open(staged.path()).await.map_err(|source| Error::Io {
            context: "open-shared-object-staging".to_string(),
            source,
        })?;
        let expected_ca_content_hash = None;
        let (node, _, _) = ingest_nar_and_hash(
            self.blob_service.clone(),
            self.directory_service.clone(),
            &mut reader,
            &expected_ca_content_hash,
        )
        .await
        .map_err(|error| Error::Castore(format!("shared-object-ingest:{error}")))?;
        let identity = node_identity(&node)?;
        if identity != signed.envelope.result.input.root_node {
            return Err(Error::State("shared-object-root-mismatch".to_string()));
        }
        let is_complete =
            crunch_store::recursive_castore_completeness(&*self.blob_service, &*self.directory_service, &node)
                .await
                .map_err(|error| Error::Castore(format!("shared-completeness:{error}")))?;
        if !is_complete {
            return Err(Error::State("shared-object-tree-incomplete".to_string()));
        }
        assert_eq!(bytes, signed.envelope.object.size_bytes);
        assert_eq!(identity, signed.envelope.result.input.root_node);
        Ok(bytes)
    }

    async fn verify_shared_manifest(
        &self,
        result: &RustUnitResult,
        local_policy: &LocalCachePolicy,
    ) -> Result<(), Error> {
        let temp =
            Builder::new().prefix("shared-verify-").tempdir_in(&self.staging_dir).map_err(|source| Error::Io {
                context: "create-shared-verify-staging".to_string(),
                source,
            })?;
        let node = node_from_identity(&result.input.root_node)?;
        let destination =
            temp.path().to_str().ok_or_else(|| Error::State("shared-verify-path-non-utf8".to_string()))?;
        crunch_store::export_castore_to_disk(&node, destination, &self.blob_service, &self.directory_service)
            .await
            .map_err(|error| Error::Castore(format!("shared-verify-export:{error}")))?;
        let observed = scan_output_artifacts(temp.path(), local_policy)?;
        if observed != result.input.artifacts {
            return Err(Error::State("shared-artifact-manifest-mismatch".to_string()));
        }
        assert_eq!(observed, result.input.artifacts);
        assert!(!observed.is_empty());
        Ok(())
    }

    async fn finish_shared_restore(
        &self,
        output_dir: &Path,
        local_policy: &LocalCachePolicy,
        admitted: Vec<AdmittedCandidate>,
        observations: Vec<SharedCandidateObservation>,
    ) -> Result<SharedRustCacheReport, Error> {
        if admitted.is_empty() {
            let disposition = if observations.iter().any(|item| item.reason_codes == [SHARED_CACHE_OFFLINE_MISS]) {
                SHARED_CACHE_OFFLINE_MISS
            } else if observations.is_empty() {
                SHARED_CACHE_MISS
            } else {
                SHARED_CACHE_REJECTED
            };
            let mut outcome = shared_report(SharedReportKind {
                disposition,
                route: "remote",
            });
            outcome.observations = observations;
            return Ok(outcome);
        }
        let facts = admitted
            .iter()
            .map(|candidate| LocalCandidateFacts {
                result: candidate.signed.envelope.result.clone(),
                content_complete: true,
                artifact_manifest_verified: true,
            })
            .collect::<Vec<_>>();
        let plan = plan_local_reuse(&admitted[0].signed.envelope.result.input.action_ref, local_policy, facts)
            .map_err(Error::Core)?;
        for candidate in &admitted {
            self.publish_record_and_index(&candidate.signed.envelope.result)?;
        }
        if plan.conflict_class.is_some() {
            let mut outcome = shared_report(SharedReportKind {
                disposition: SHARED_CACHE_CONFLICT,
                route: "remote",
            });
            outcome.candidate_count = bounded_count(admitted.len())?;
            outcome.observations = observations;
            return Ok(outcome);
        }
        self.materialize_admitted(output_dir, local_policy, &admitted, observations).await
    }

    async fn materialize_admitted(
        &self,
        output_dir: &Path,
        local_policy: &LocalCachePolicy,
        admitted: &[AdmittedCandidate],
        observations: Vec<SharedCandidateObservation>,
    ) -> Result<SharedRustCacheReport, Error> {
        let selected = &admitted[0];
        let result = &selected.signed.envelope.result;
        self.materialize_and_commit(result, output_dir, local_policy).await?;
        let reused_bytes = artifact_bytes(&result.input.artifacts)?;
        let transferred_bytes = admitted.iter().try_fold(0_u64, |total, candidate| {
            total
                .checked_add(candidate.transferred_bytes)
                .ok_or_else(|| Error::Bound("shared-transfer-byte-overflow".to_string()))
        })?;
        assert!(selected.source_priority < MAX_SHARED_SOURCES as u32);
        assert!(!selected.accepted_verifier_blake3.is_empty());
        Ok(SharedRustCacheReport {
            schema: SHARED_CACHE_REPORT_SCHEMA.to_string(),
            disposition: SHARED_CACHE_HIT.to_string(),
            route: "remote".to_string(),
            selected_source_id: Some(selected.source_id.clone()),
            selected_result_ref: Some(result.result_ref.clone()),
            candidate_count: bounded_count(admitted.len())?,
            artifact_count: bounded_count(result.input.artifacts.len())?,
            transferred_bytes,
            reused_bytes,
            compiler_executed: false,
            observations,
            publications: Vec::new(),
            non_claims: shared_non_claims(),
        })
    }

    async fn render_shared_object(&self, result: &RustUnitResult) -> Result<StagedSharedObject, Error> {
        let node = node_from_identity(&result.input.root_node)?;
        let is_complete =
            crunch_store::recursive_castore_completeness(&*self.blob_service, &*self.directory_service, &node)
                .await
                .map_err(|error| Error::Castore(format!("shared-publish-completeness:{error}")))?;
        if !is_complete {
            return Err(Error::State("shared-publish-tree-incomplete".to_string()));
        }
        let staged =
            Builder::new()
                .prefix("shared-publish-")
                .tempfile_in(&self.staging_dir)
                .map_err(|source| Error::Io {
                    context: "create-shared-publish-staging".to_string(),
                    source,
                })?;
        let mut file = tokio::fs::File::create(staged.path()).await.map_err(|source| Error::Io {
            context: "open-shared-publish-staging".to_string(),
            source,
        })?;
        let (mut reader, writer) = tokio::io::duplex(NAR_DUPLEX_BUFFER_BYTES);
        let render_node = node.clone();
        let blob_service = self.blob_service.clone();
        let directory_service = self.directory_service.clone();
        let render_task =
            tokio::spawn(async move { write_nar(writer, &render_node, blob_service, directory_service).await });
        tokio::io::copy(&mut reader, &mut file).await.map_err(|source| Error::Io {
            context: "copy-shared-publish-staging".to_string(),
            source,
        })?;
        render_task
            .await
            .map_err(|error| Error::Castore(format!("shared-publish-render-task:{error}")))?
            .map_err(|error| Error::Castore(format!("shared-publish-render:{error}")))?;
        file.sync_all().await.map_err(|source| Error::Io {
            context: "sync-shared-publish-staging".to_string(),
            source,
        })?;
        drop(file);
        let (size_bytes, digest) = hash_file_bounded(staged.path(), MAX_SHARED_TRANSFER_BYTES)?;
        let identity = RustResultObjectIdentity {
            object_ref: format!("{SHARED_RUST_OBJECT_REF_PREFIX}{digest}"),
            size_bytes,
        };
        assert!(size_bytes > 0);
        assert!(identity.object_ref.starts_with(SHARED_RUST_OBJECT_REF_PREFIX));
        Ok(StagedSharedObject { staged, identity })
    }
}

struct StagedSharedObject {
    staged: tempfile::NamedTempFile,
    identity: RustResultObjectIdentity,
}

async fn publish_ordered(
    source: &dyn RustResultSource,
    staged: &StagedSharedObject,
    signed: &SignedRustResultEnvelope,
    envelope_bytes: &[u8],
    candidate: &SharedRustCandidateClaim,
) -> Result<(), Error> {
    source
        .publish_object(&staged.identity.object_ref, staged.staged.path(), staged.identity.size_bytes)
        .await
        .map_err(|error| Error::State(format!("shared-publish-object:{}", error.code)))?;
    source
        .publish_envelope(&signed.envelope.envelope_ref, envelope_bytes)
        .await
        .map_err(|error| Error::State(format!("shared-publish-envelope:{}", error.code)))?;
    source
        .publish_candidate(candidate)
        .await
        .map_err(|error| Error::State(format!("shared-publish-candidate:{}", error.code)))?;
    assert_eq!(candidate.envelope_ref, signed.envelope.envelope_ref);
    assert_eq!(candidate.result_ref, signed.envelope.result.result_ref);
    Ok(())
}

fn validate_shared_inputs(
    action: &RustUnitAction,
    local_policy: &LocalCachePolicy,
    shared_policy: &SharedRustCachePolicy,
    sources: &[Arc<dyn RustResultSource>],
) -> Result<(), Error> {
    crunch_rust_cache_core::validate_rust_action(action).map_err(Error::Core)?;
    crunch_rust_cache_core::validate_local_cache_policy(local_policy).map_err(Error::Core)?;
    validate_shared_cache_policy(shared_policy)?;
    let max_sources = usize::try_from(shared_policy.max_sources)
        .map_err(|_| Error::Bound("shared-source-limit-unrepresentable".to_string()))?;
    if sources.len() > max_sources {
        return Err(Error::Bound("shared-source-limit-exceeded".to_string()));
    }
    let identities = sources.iter().map(|source| source.source_id()).collect::<BTreeSet<_>>();
    if identities.len() != sources.len() {
        return Err(Error::State("shared-source-identity-duplicate".to_string()));
    }
    assert!(sources.len() <= max_sources);
    assert_eq!(identities.len(), sources.len());
    Ok(())
}

pub fn validate_shared_cache_policy(policy: &SharedRustCachePolicy) -> Result<(), Error> {
    if policy.schema != SHARED_CACHE_POLICY_SCHEMA {
        return Err(Error::Core("shared-cache-policy-schema-unsupported".to_string()));
    }
    if policy.policy_id.is_empty() || policy.policy_id.len() > MAX_SOURCE_ID_BYTES {
        return Err(Error::Core("shared-cache-policy-id-invalid".to_string()));
    }
    if policy.max_sources == 0 || policy.max_sources > MAX_SHARED_SOURCES as u32 {
        return Err(Error::Bound("shared-source-limit-invalid".to_string()));
    }
    if policy.max_candidates == 0 || policy.max_candidates > MAX_SHARED_CANDIDATES as u32 {
        return Err(Error::Bound("shared-candidate-limit-invalid".to_string()));
    }
    if policy.max_metadata_bytes == 0 || policy.max_metadata_bytes > MAX_SHARED_METADATA_BYTES {
        return Err(Error::Bound("shared-metadata-limit-invalid".to_string()));
    }
    if policy.max_transfer_bytes == 0 || policy.max_transfer_bytes > MAX_SHARED_TRANSFER_BYTES {
        return Err(Error::Bound("shared-transfer-limit-invalid".to_string()));
    }
    if policy.max_redirects != 0 {
        return Err(Error::Bound("shared-redirect-limit-unsupported".to_string()));
    }
    if policy.max_retries != 0 {
        return Err(Error::Bound("shared-retry-limit-unsupported".to_string()));
    }
    assert!(policy.max_sources <= MAX_SHARED_SOURCES as u32);
    assert!(policy.max_transfer_bytes <= MAX_SHARED_TRANSFER_BYTES);
    Ok(())
}

fn candidate_for_envelope(signed: &SignedRustResultEnvelope) -> Result<SharedRustCandidateClaim, Error> {
    let candidate = SharedRustCandidateClaim {
        schema: SHARED_CANDIDATE_SCHEMA.to_string(),
        action_ref: signed.envelope.result.input.action_ref.clone(),
        envelope_ref: signed.envelope.envelope_ref.clone(),
        result_ref: signed.envelope.result.result_ref.clone(),
    };
    validate_candidate_claim(&candidate)
        .map_err(|error| Error::State(format!("shared-candidate-from-envelope:{}", error.code)))?;
    assert_eq!(candidate.envelope_ref, signed.envelope.envelope_ref);
    assert_eq!(candidate.result_ref, signed.envelope.result.result_ref);
    Ok(candidate)
}

fn validate_candidate_claim(candidate: &SharedRustCandidateClaim) -> Result<(), SourceError> {
    if candidate.schema != SHARED_CANDIDATE_SCHEMA {
        return Err(SourceError::new("shared-candidate-schema-unsupported"));
    }
    typed_ref_digest(&candidate.action_ref, TypedRefRule {
        prefix: crunch_rust_cache_core::RUST_ACTION_REF_PREFIX,
        code: ValidationCode("shared-candidate-action-ref-invalid"),
    })?;
    typed_ref_digest(&candidate.envelope_ref, TypedRefRule {
        prefix: SHARED_RUST_ENVELOPE_REF_PREFIX,
        code: ValidationCode("shared-candidate-envelope-ref-invalid"),
    })?;
    typed_ref_digest(&candidate.result_ref, TypedRefRule {
        prefix: crunch_rust_cache_core::RUST_RESULT_REF_PREFIX,
        code: ValidationCode("shared-candidate-result-ref-invalid"),
    })?;
    assert!(!candidate.action_ref.is_empty());
    assert!(!candidate.result_ref.is_empty());
    Ok(())
}

fn decode_candidate_envelope(
    bytes: &[u8],
    claim: &SharedRustCandidateClaim,
) -> Result<SignedRustResultEnvelope, String> {
    if bytes.len() > MAX_SHARED_ENVELOPE_BYTES {
        return Err("shared-envelope-too-large".to_string());
    }
    let signed = serde_json::from_slice::<SignedRustResultEnvelope>(bytes)
        .map_err(|_| "shared-envelope-malformed".to_string())?;
    validate_signed_rust_result_envelope(&signed)?;
    if signed.envelope.envelope_ref != claim.envelope_ref {
        return Err("shared-envelope-claim-mismatch".to_string());
    }
    if signed.envelope.result.input.action_ref != claim.action_ref {
        return Err("shared-envelope-action-claim-mismatch".to_string());
    }
    if signed.envelope.result.result_ref != claim.result_ref {
        return Err("shared-envelope-result-claim-mismatch".to_string());
    }
    assert_eq!(signed.envelope.envelope_ref, claim.envelope_ref);
    assert_eq!(signed.envelope.result.result_ref, claim.result_ref);
    Ok(signed)
}

fn verify_staged_object(path: &Path, identity: &RustResultObjectIdentity) -> Result<(), Error> {
    let (size_bytes, digest) = hash_file_bounded(path, identity.size_bytes)?;
    if size_bytes != identity.size_bytes {
        return Err(Error::State("shared-object-size-mismatch".to_string()));
    }
    let expected = identity
        .object_ref
        .strip_prefix(SHARED_RUST_OBJECT_REF_PREFIX)
        .ok_or_else(|| Error::State("shared-object-ref-invalid".to_string()))?;
    if digest != expected {
        return Err(Error::State("shared-object-digest-mismatch".to_string()));
    }
    assert_eq!(size_bytes, identity.size_bytes);
    assert_eq!(digest, expected);
    Ok(())
}

fn hash_file_bounded(path: &Path, max_bytes: u64) -> Result<(u64, String), Error> {
    let mut file = open_read_nofollow(path).map_err(|source| Error::Io {
        context: "open-shared-object-hash".to_string(),
        source,
    })?;
    let declared_bytes = file
        .metadata()
        .map_err(|source| Error::Io {
            context: "stat-shared-object-hash".to_string(),
            source,
        })?
        .len();
    if declared_bytes > max_bytes {
        return Err(Error::Bound("shared-object-byte-limit-exceeded".to_string()));
    }
    let max_read_iterations = bounded_read_iterations(declared_bytes, COPY_BUFFER_BYTES)
        .ok_or_else(|| Error::Bound("shared-object-read-iteration-overflow".to_string()))?;
    let mut buffer = vec![0_u8; COPY_BUFFER_BYTES];
    let mut hasher = blake3::Hasher::new();
    let mut total_bytes = 0_u64;
    for _read_index in 0..max_read_iterations {
        let count_bytes = file.read(&mut buffer).map_err(|source| Error::Io {
            context: "read-shared-object-hash".to_string(),
            source,
        })?;
        if count_bytes == 0 {
            break;
        }
        let count_bytes_u64 = u64::try_from(count_bytes)
            .map_err(|_| Error::Bound("shared-object-read-size-unrepresentable".to_string()))?;
        total_bytes = total_bytes
            .checked_add(count_bytes_u64)
            .ok_or_else(|| Error::Bound("shared-object-byte-overflow".to_string()))?;
        if total_bytes > max_bytes {
            return Err(Error::Bound("shared-object-byte-limit-exceeded".to_string()));
        }
        hasher.update(&buffer[..count_bytes]);
    }
    if total_bytes != declared_bytes {
        return Err(Error::State("shared-object-size-changed-during-read".to_string()));
    }
    let digest = hasher.finalize().to_hex().to_string();
    assert!(total_bytes <= max_bytes);
    assert_eq!(digest.len(), crunch_rust_cache_core::BLAKE3_HEX_CHARS);
    Ok((total_bytes, digest))
}

fn bounded_read_iterations(byte_count: u64, buffer_bytes: usize) -> Option<u64> {
    let buffer_bytes = u64::try_from(buffer_bytes).ok()?;
    if buffer_bytes == 0 {
        return None;
    }
    let iterations = byte_count.checked_div(buffer_bytes)?.checked_add(EXTRA_EOF_READ_ITERATIONS)?;
    assert!(iterations >= EXTRA_EOF_READ_ITERATIONS);
    assert!(buffer_bytes > 0);
    Some(iterations)
}

fn load_candidate_entries(
    entries: Vec<PathBuf>,
    action_ref: &str,
    max_candidates: u32,
    max_metadata_bytes: u64,
) -> Result<SharedSourceLookup, SourceError> {
    let max_candidates =
        usize::try_from(max_candidates).map_err(|_| SourceError::new("shared-candidate-limit-unrepresentable"))?;
    if entries.len() > max_candidates {
        return Err(SourceError::new("shared-candidate-limit-exceeded"));
    }
    let mut candidates = Vec::with_capacity(entries.len());
    let mut metadata_bytes = 0_u64;
    for path in entries {
        let remaining = max_metadata_bytes
            .checked_sub(metadata_bytes)
            .ok_or_else(|| SourceError::new("shared-metadata-limit-exceeded"))?;
        let bytes = read_source_file_bounded(&path, remaining, "shared-candidate")?;
        metadata_bytes = metadata_bytes
            .checked_add(bytes.len() as u64)
            .ok_or_else(|| SourceError::new("shared-metadata-overflow"))?;
        let candidate = serde_json::from_slice::<SharedRustCandidateClaim>(&bytes)
            .map_err(|_| SourceError::new("shared-candidate-malformed"))?;
        validate_candidate_claim(&candidate)?;
        if candidate.action_ref != action_ref {
            return Err(SourceError::new("shared-candidate-action-mismatch"));
        }
        candidates.push(candidate);
    }
    candidates.sort();
    candidates.dedup();
    assert!(candidates.len() <= max_candidates);
    assert!(metadata_bytes <= max_metadata_bytes);
    Ok(SharedSourceLookup {
        candidates,
        metadata_bytes,
    })
}

fn read_dir_optional(path: &Path) -> Result<Option<Vec<PathBuf>>, SourceError> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(SourceError::new("shared-action-index-stat-failed")),
    };
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Err(SourceError::new("shared-action-index-type-invalid"));
    }
    let iterator = fs::read_dir(path).map_err(|_| SourceError::new("shared-action-index-read-failed"))?;
    let mut paths = Vec::with_capacity(MAX_SHARED_CANDIDATES);
    for entry in iterator {
        let entry = entry.map_err(|_| SourceError::new("shared-action-index-entry-failed"))?;
        let file_type = entry.file_type().map_err(|_| SourceError::new("shared-action-index-type-failed"))?;
        if !file_type.is_file() || file_type.is_symlink() {
            return Err(SourceError::new("shared-action-index-entry-invalid"));
        }
        paths.push(entry.path());
        if paths.len() > MAX_SHARED_CANDIDATES {
            return Err(SourceError::new("shared-candidate-limit-exceeded"));
        }
    }
    paths.sort();
    assert!(paths.len() <= MAX_SHARED_CANDIDATES);
    assert!(paths.windows(2).all(|pair| pair[0] < pair[1]));
    Ok(Some(paths))
}

fn read_source_file_bounded(path: &Path, max_bytes: u64, class: &str) -> Result<Vec<u8>, SourceError> {
    let metadata = fs::symlink_metadata(path).map_err(|_| SourceError::new(format!("{class}-missing")))?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(SourceError::new(format!("{class}-type-invalid")));
    }
    if metadata.len() > max_bytes {
        return Err(SourceError::new(format!("{class}-too-large")));
    }
    let mut file = open_read_nofollow(path).map_err(|_| SourceError::new(format!("{class}-open-failed")))?;
    let capacity_bytes = usize::try_from(metadata.len()).map_err(|_| SourceError::new(format!("{class}-too-large")))?;
    let mut bytes = Vec::with_capacity(capacity_bytes);
    file.read_to_end(&mut bytes).map_err(|_| SourceError::new(format!("{class}-read-failed")))?;
    if bytes.len() as u64 != metadata.len() {
        return Err(SourceError::new(format!("{class}-size-changed")));
    }
    assert!(bytes.len() as u64 <= max_bytes);
    assert_eq!(bytes.len(), capacity_bytes);
    Ok(bytes)
}

async fn copy_source_file_bounded(source: &Path, destination: &Path, max_bytes: u64) -> Result<u64, SourceError> {
    let metadata = fs::symlink_metadata(source).map_err(|_| SourceError::new("shared-object-missing"))?;
    if !metadata.file_type().is_file() || metadata.file_type().is_symlink() {
        return Err(SourceError::new("shared-object-type-invalid"));
    }
    if metadata.len() > max_bytes {
        return Err(SourceError::new("shared-object-too-large"));
    }
    let input = open_read_nofollow(source).map_err(|_| SourceError::new("shared-object-open-failed"))?;
    let mut input = tokio::fs::File::from_std(input);
    let mut output = tokio::fs::File::create(destination)
        .await
        .map_err(|_| SourceError::new("shared-object-staging-open-failed"))?;
    let copied = tokio::io::copy(&mut input, &mut output)
        .await
        .map_err(|_| SourceError::new("shared-object-copy-failed"))?;
    output.sync_all().await.map_err(|_| SourceError::new("shared-object-staging-sync-failed"))?;
    if copied != metadata.len() {
        return Err(SourceError::new("shared-object-truncated"));
    }
    assert!(copied <= max_bytes);
    assert_eq!(copied, metadata.len());
    Ok(copied)
}

async fn publish_immutable_file(destination: &Path, source: &Path, expected_bytes: u64) -> Result<(), SourceError> {
    if destination.exists() {
        return compare_existing_file(destination, source, expected_bytes);
    }
    let parent = destination.parent().ok_or_else(|| SourceError::new("shared-publish-parent-missing"))?;
    create_source_directory(parent)?;
    let staged = Builder::new()
        .prefix("publish-")
        .tempfile_in(parent)
        .map_err(|_| SourceError::new("shared-publish-staging-create-failed"))?;
    let copied = copy_source_file_bounded(source, staged.path(), expected_bytes).await?;
    if copied != expected_bytes {
        return Err(SourceError::new("shared-publish-object-size-mismatch"));
    }
    link_noclobber(staged.path(), destination)?;
    fsync_directory_source(parent)?;
    assert!(destination.is_file());
    assert_eq!(copied, expected_bytes);
    Ok(())
}

fn publish_immutable_bytes(destination: &Path, bytes: &[u8]) -> Result<(), SourceError> {
    if destination.exists() {
        let existing = read_source_file_bounded(destination, bytes.len() as u64, "shared-immutable")?;
        if existing != bytes {
            return Err(SourceError::new("shared-immutable-conflict"));
        }
        return Ok(());
    }
    let parent = destination.parent().ok_or_else(|| SourceError::new("shared-publish-parent-missing"))?;
    create_source_directory(parent)?;
    let mut staged = Builder::new()
        .prefix("publish-")
        .tempfile_in(parent)
        .map_err(|_| SourceError::new("shared-publish-staging-create-failed"))?;
    staged
        .as_file_mut()
        .write_all(bytes)
        .map_err(|_| SourceError::new("shared-publish-staging-write-failed"))?;
    staged
        .as_file_mut()
        .sync_all()
        .map_err(|_| SourceError::new("shared-publish-staging-sync-failed"))?;
    link_noclobber(staged.path(), destination)?;
    fsync_directory_source(parent)?;
    assert!(destination.is_file());
    assert!(!bytes.is_empty());
    Ok(())
}

fn compare_existing_file(destination: &Path, source: &Path, expected_bytes: u64) -> Result<(), SourceError> {
    let destination_hash = hash_source_file(destination, expected_bytes)?;
    let source_hash = hash_source_file(source, expected_bytes)?;
    if destination_hash != source_hash {
        return Err(SourceError::new("shared-immutable-conflict"));
    }
    assert_eq!(destination_hash, source_hash);
    assert!(destination.is_file());
    Ok(())
}

fn hash_source_file(path: &Path, max_bytes: u64) -> Result<(u64, String), SourceError> {
    let mut file = open_read_nofollow(path).map_err(|_| SourceError::new("shared-immutable-open-failed"))?;
    let declared_bytes = file.metadata().map_err(|_| SourceError::new("shared-immutable-stat-failed"))?.len();
    if declared_bytes > max_bytes {
        return Err(SourceError::new("shared-immutable-too-large"));
    }
    let max_read_iterations = bounded_read_iterations(declared_bytes, COPY_BUFFER_BYTES)
        .ok_or_else(|| SourceError::new("shared-immutable-read-iteration-overflow"))?;
    let mut buffer = vec![0_u8; COPY_BUFFER_BYTES];
    let mut hasher = blake3::Hasher::new();
    let mut total_bytes = 0_u64;
    for _read_index in 0..max_read_iterations {
        let count_bytes = file.read(&mut buffer).map_err(|_| SourceError::new("shared-immutable-read-failed"))?;
        if count_bytes == 0 {
            break;
        }
        let count_bytes_u64 =
            u64::try_from(count_bytes).map_err(|_| SourceError::new("shared-immutable-read-size-unrepresentable"))?;
        total_bytes = total_bytes
            .checked_add(count_bytes_u64)
            .ok_or_else(|| SourceError::new("shared-immutable-byte-overflow"))?;
        if total_bytes > max_bytes {
            return Err(SourceError::new("shared-immutable-too-large"));
        }
        hasher.update(&buffer[..count_bytes]);
    }
    if total_bytes != declared_bytes {
        return Err(SourceError::new("shared-immutable-size-changed-during-read"));
    }
    let digest = hasher.finalize().to_hex().to_string();
    assert!(total_bytes <= max_bytes);
    assert_eq!(digest.len(), crunch_rust_cache_core::BLAKE3_HEX_CHARS);
    Ok((total_bytes, digest))
}

fn link_noclobber(staged: &Path, destination: &Path) -> Result<(), SourceError> {
    match fs::hard_link(staged, destination) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let expected_bytes =
                fs::metadata(staged).map_err(|_| SourceError::new("shared-immutable-staging-stat-failed"))?.len();
            compare_existing_file(destination, staged, expected_bytes)
        }
        Err(_) => Err(SourceError::new("shared-immutable-commit-failed")),
    }
}

fn create_private_directory(path: &Path) -> Result<(), Error> {
    fs::create_dir_all(path).map_err(|source| Error::Io {
        context: "create-shared-source-directory".to_string(),
        source,
    })?;
    fs::set_permissions(path, std::os::unix::fs::PermissionsExt::from_mode(PRIVATE_DIRECTORY_MODE)).map_err(
        |source| Error::Io {
            context: "chmod-shared-source-directory".to_string(),
            source,
        },
    )?;
    reject_symlink(path)?;
    assert!(path.is_dir());
    assert!(!path.as_os_str().is_empty());
    Ok(())
}

fn create_source_directory(path: &Path) -> Result<(), SourceError> {
    fs::create_dir_all(path).map_err(|_| SourceError::new("shared-source-directory-create-failed"))?;
    fs::set_permissions(path, std::os::unix::fs::PermissionsExt::from_mode(PRIVATE_DIRECTORY_MODE))
        .map_err(|_| SourceError::new("shared-source-directory-chmod-failed"))?;
    let metadata = fs::symlink_metadata(path).map_err(|_| SourceError::new("shared-source-directory-stat-failed"))?;
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
        return Err(SourceError::new("shared-source-directory-invalid"));
    }
    assert!(path.is_dir());
    assert!(!path.as_os_str().is_empty());
    Ok(())
}

fn reject_symlink(path: &Path) -> Result<(), Error> {
    let metadata = fs::symlink_metadata(path).map_err(|source| Error::Io {
        context: "stat-shared-source-directory".to_string(),
        source,
    })?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_dir() {
        return Err(Error::State("shared-source-directory-invalid".to_string()));
    }
    assert!(metadata.file_type().is_dir());
    assert!(!metadata.file_type().is_symlink());
    Ok(())
}

fn open_read_nofollow(path: &Path) -> std::io::Result<File> {
    OpenOptions::new().read(true).custom_flags(NO_FOLLOW_FLAG).open(path)
}

fn fsync_directory_source(path: &Path) -> Result<(), SourceError> {
    let directory = open_read_nofollow(path).map_err(|_| SourceError::new("shared-publish-parent-open-failed"))?;
    directory.sync_all().map_err(|_| SourceError::new("shared-publish-parent-sync-failed"))?;
    assert!(path.is_dir());
    assert!(!path.as_os_str().is_empty());
    Ok(())
}

fn typed_ref_digest<'a>(value: &'a str, rule: TypedRefRule<'_>) -> Result<&'a str, SourceError> {
    let digest = value.strip_prefix(rule.prefix).ok_or_else(|| SourceError::new(rule.code.0))?;
    if digest.len() != crunch_rust_cache_core::BLAKE3_HEX_CHARS
        || !digest.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(SourceError::new(rule.code.0));
    }
    assert_eq!(digest.len(), crunch_rust_cache_core::BLAKE3_HEX_CHARS);
    assert!(value.starts_with(rule.prefix));
    Ok(digest)
}

fn sanitized_source_id(path: &Path) -> String {
    let source_id = sanitized_source_bytes(path.as_os_str().as_encoded_bytes());
    assert!(source_id.len() <= MAX_SOURCE_ID_BYTES);
    assert!(source_id.starts_with(SOURCE_ID_PREFIX));
    source_id
}

fn sanitized_source_bytes(bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(SOURCE_ID_DOMAIN);
    hasher.update(&[DOMAIN_SEPARATOR]);
    hasher.update(bytes);
    let digest = hasher.finalize().to_hex();
    let source_id = format!("{SOURCE_ID_PREFIX}{digest}");
    assert!(source_id.len() <= MAX_SOURCE_ID_BYTES);
    assert!(source_id.starts_with(SOURCE_ID_PREFIX));
    source_id
}

struct AuthorityObservationInput<'a> {
    source_id: &'a str,
    priority: usize,
    claim: &'a SharedRustCandidateClaim,
    authority: &'a crunch_rust_cache_core::shared::RustResultAuthorityDecision,
    metadata_bytes: u64,
    transferred_bytes: u64,
}

fn authority_observation(input: AuthorityObservationInput<'_>) -> Result<SharedCandidateObservation, Error> {
    Ok(SharedCandidateObservation {
        source_id: input.source_id.to_string(),
        source_priority: u32::try_from(input.priority)
            .map_err(|_| Error::Bound("source-priority-overflow".to_string()))?,
        envelope_ref: Some(input.claim.envelope_ref.clone()),
        result_ref: Some(input.claim.result_ref.clone()),
        authority_disposition: input.authority.authority_disposition.clone(),
        accepted_verifier_blake3: input.authority.accepted_verifier_blake3.clone(),
        reason_codes: input.authority.reason_codes.clone(),
        metadata_bytes: input.metadata_bytes,
        transferred_bytes: input.transferred_bytes,
    })
}

fn candidate_error_observation(
    source_id: &str,
    priority: usize,
    claim: &SharedRustCandidateClaim,
    reason: String,
) -> Result<SharedCandidateObservation, Error> {
    Ok(SharedCandidateObservation {
        source_id: source_id.to_string(),
        source_priority: u32::try_from(priority).map_err(|_| Error::Bound("source-priority-overflow".to_string()))?,
        envelope_ref: Some(claim.envelope_ref.clone()),
        result_ref: Some(claim.result_ref.clone()),
        authority_disposition: "rejected".to_string(),
        accepted_verifier_blake3: None,
        reason_codes: vec![reason],
        metadata_bytes: 0,
        transferred_bytes: 0,
    })
}

fn source_error_observation(
    source_id: &str,
    priority: usize,
    reason: String,
) -> Result<SharedCandidateObservation, Error> {
    Ok(SharedCandidateObservation {
        source_id: source_id.to_string(),
        source_priority: u32::try_from(priority).map_err(|_| Error::Bound("source-priority-overflow".to_string()))?,
        envelope_ref: None,
        result_ref: None,
        authority_disposition: "source-rejected".to_string(),
        accepted_verifier_blake3: None,
        reason_codes: vec![reason],
        metadata_bytes: 0,
        transferred_bytes: 0,
    })
}

fn offline_observation(source_id: &str, priority: usize) -> Result<SharedCandidateObservation, Error> {
    source_error_observation(source_id, priority, SHARED_CACHE_OFFLINE_MISS.to_string())
}

#[derive(Clone, Copy)]
struct SharedReportKind<'a> {
    disposition: &'a str,
    route: &'a str,
}

fn shared_report(kind: SharedReportKind<'_>) -> SharedRustCacheReport {
    SharedRustCacheReport {
        schema: SHARED_CACHE_REPORT_SCHEMA.to_string(),
        disposition: kind.disposition.to_string(),
        route: kind.route.to_string(),
        selected_source_id: None,
        selected_result_ref: None,
        candidate_count: 0,
        artifact_count: 0,
        transferred_bytes: 0,
        reused_bytes: 0,
        compiler_executed: false,
        observations: Vec::new(),
        publications: Vec::new(),
        non_claims: shared_non_claims(),
    }
}

fn shared_rejection(reason: &str) -> SharedRustCacheReport {
    let mut outcome = shared_report(SharedReportKind {
        disposition: SHARED_CACHE_REJECTED,
        route: "remote",
    });
    outcome.observations.push(SharedCandidateObservation {
        source_id: "local-preflight".to_string(),
        source_priority: 0,
        envelope_ref: None,
        result_ref: None,
        authority_disposition: "rejected".to_string(),
        accepted_verifier_blake3: None,
        reason_codes: vec![reason.to_string()],
        metadata_bytes: 0,
        transferred_bytes: 0,
    });
    assert_eq!(outcome.observations.len(), 1);
    assert_eq!(outcome.disposition, SHARED_CACHE_REJECTED);
    outcome
}

fn shared_non_claims() -> Vec<String> {
    vec![
        "compiler-correctness-not-proven".to_string(),
        "full-cargo-compatibility-not-proven".to_string(),
        "universal-reproducibility-not-proven".to_string(),
        "remote-executor-correctness-not-proven".to_string(),
        "release-eligibility-not-proven".to_string(),
    ]
}

fn stable_cache_error(error: &Error) -> String {
    match error {
        Error::Core(_) => "shared-candidate-core-rejected",
        Error::Io { .. } => "shared-candidate-io-rejected",
        Error::Json(_) => "shared-candidate-json-rejected",
        Error::Castore(_) => "shared-candidate-castore-rejected",
        Error::Bound(_) => "shared-candidate-bound-rejected",
        Error::State(_) => "shared-candidate-state-rejected",
        Error::LockTimeout => "shared-candidate-lock-timeout",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::io::Read as _;
    use std::io::Write as _;
    use std::net::TcpListener;
    use std::sync::atomic::AtomicU32;
    use std::sync::atomic::Ordering;
    use std::thread;

    use crunch_rust_cache_core::RustBuildFact;
    use crunch_rust_cache_core::RustSemanticArgument;
    use crunch_rust_cache_core::RustUnitActionInput;
    use crunch_rust_cache_core::canonical_rust_action;
    use crunch_rust_cache_core::shared::SHARED_RUST_TRUST_POLICY_SCHEMA;
    use crunch_rust_cache_core::shared::TrustedRustResultKey;
    use snix_castore::blobservice::BlobService;
    use snix_castore::blobservice::MemoryBlobService;
    use snix_castore::directoryservice::DirectoryService;
    use snix_castore::directoryservice::RedbDirectoryService;
    use snix_castore::directoryservice::RedbDirectoryServiceConfig;

    use super::*;
    use crate::PublishRequest;

    const TEST_KEY_BYTE: u8 = 41;
    const TEST_OTHER_KEY_BYTE: u8 = 43;
    const RECEIPT_DIGEST_CHAR: char = 'd';
    const ACTION_DIGEST_CHAR: char = 'a';
    const SOURCE_OPEN_COUNT: u32 = 0;
    const HTTP_TEST_READ_BYTES: usize = 4_096;
    const HTTP_TEST_TIMEOUT: Duration = Duration::from_millis(25);
    const HTTP_TEST_DELAY: Duration = Duration::from_millis(100);
    const HTTP_TEST_NO_DELAY: Duration = Duration::from_millis(0);
    const HTTP_TEST_BODY_LIMIT: u64 = 16;

    #[tokio::test]
    async fn clean_client_restores_signed_remote_result() {
        let root = tempfile::tempdir().unwrap();
        let producer = test_cache(&root.path().join("producer")).await;
        let client = test_cache(&root.path().join("client")).await;
        let source = Arc::new(DirectoryRustResultSource::open(root.path().join("shared"), true).unwrap());
        let action = test_action();
        let output = root.path().join("producer-output");
        write_output(&output, b"shared-result");
        let result = producer
            .publish(PublishRequest {
                action: &action,
                output_dir: &output,
                producer_receipt_ref: &receipt_ref(),
                policy: &local_policy(),
            })
            .await
            .unwrap();
        let signing_key = signing_key(TEST_KEY_BYTE);
        let trust = trust_policy(&signing_key);
        producer
            .publish_shared(source.as_ref(), shared_publish_request(&result, &signing_key, &shared_policy()))
            .await
            .unwrap();
        let restored = root.path().join("restored");

        let report = client
            .restore_shared(SharedRestoreRequest {
                action: &action,
                output_dir: &restored,
                local_policy: &local_policy(),
                shared_policy: &shared_policy(),
                trust_policy: &trust,
                sources: &[source],
            })
            .await
            .unwrap();

        assert_eq!(report.disposition, SHARED_CACHE_HIT);
        assert_eq!(fs::read(restored.join("artifact.rlib")).unwrap(), b"shared-result");
        assert!(report.transferred_bytes > 0);
        assert!(!report.compiler_executed);
    }

    #[tokio::test]
    async fn offline_policy_never_opens_remote_source() {
        let root = tempfile::tempdir().unwrap();
        let client = test_cache(&root.path().join("client")).await;
        let opens = Arc::new(AtomicU32::new(SOURCE_OPEN_COUNT));
        let source: Arc<dyn RustResultSource> = Arc::new(CountingRemoteSource { opens: opens.clone() });
        let mut policy = shared_policy();
        policy.offline = true;

        let report = client
            .restore_shared(SharedRestoreRequest {
                action: &test_action(),
                output_dir: &root.path().join("output"),
                local_policy: &local_policy(),
                shared_policy: &policy,
                trust_policy: &trust_policy(&signing_key(TEST_KEY_BYTE)),
                sources: &[source],
            })
            .await
            .unwrap();

        assert_eq!(report.disposition, SHARED_CACHE_OFFLINE_MISS);
        assert_eq!(opens.load(Ordering::SeqCst), SOURCE_OPEN_COUNT);
        assert_eq!(report.observations[0].reason_codes, [SHARED_CACHE_OFFLINE_MISS]);
    }

    #[tokio::test]
    async fn missing_object_rejects_candidate_without_output() {
        let fixture = published_fixture().await;
        let object_path = fixture.source.object_path(&fixture.publication.object_ref).unwrap();
        fs::remove_file(object_path).unwrap();
        let output = fixture.root.path().join("missing-output");

        let source: Arc<dyn RustResultSource> = fixture.source.clone();
        let report = fixture
            .client
            .restore_shared(SharedRestoreRequest {
                action: &fixture.action,
                output_dir: &output,
                local_policy: &local_policy(),
                shared_policy: &shared_policy(),
                trust_policy: &fixture.trust,
                sources: &[source],
            })
            .await
            .unwrap();

        assert_eq!(report.disposition, SHARED_CACHE_REJECTED);
        assert!(!output.exists());
        assert!(report.observations.iter().any(|item| item.reason_codes == ["shared-candidate-state-rejected"]));
    }

    #[tokio::test]
    async fn wrong_full_key_rejects_signed_candidate() {
        let fixture = published_fixture().await;
        let output = fixture.root.path().join("untrusted-output");
        let wrong_trust = trust_policy(&signing_key(TEST_OTHER_KEY_BYTE));

        let source: Arc<dyn RustResultSource> = fixture.source.clone();
        let report = fixture
            .client
            .restore_shared(SharedRestoreRequest {
                action: &fixture.action,
                output_dir: &output,
                local_policy: &local_policy(),
                shared_policy: &shared_policy(),
                trust_policy: &wrong_trust,
                sources: &[source],
            })
            .await
            .unwrap();

        assert_eq!(report.disposition, SHARED_CACHE_REJECTED);
        assert!(!output.exists());
        assert!(
            report
                .observations
                .iter()
                .any(|item| { item.reason_codes.contains(&"shared-rust-verifier-key-untrusted".to_string()) })
        );
    }

    #[tokio::test]
    async fn candidate_marker_is_last_visibility_edge() {
        let fixture = published_fixture_without_candidate().await;
        let lookup = fixture
            .source
            .lookup(&fixture.action.action_ref, MAX_SHARED_CANDIDATES as u32, MAX_SHARED_METADATA_BYTES)
            .await
            .unwrap();
        assert!(lookup.candidates.is_empty());
        assert!(fixture.source.object_path(&fixture.publication.object_ref).unwrap().is_file());
        assert!(fixture.source.envelope_path(&fixture.publication.envelope_ref).unwrap().is_file());
    }

    #[tokio::test]
    async fn concurrent_exact_publication_deduplicates_without_overwrite() {
        let fixture = published_fixture_without_candidate().await;
        let envelope_bytes = read_source_file_bounded(
            &fixture.source.envelope_path(&fixture.publication.envelope_ref).unwrap(),
            MAX_SHARED_METADATA_BYTES,
            "test-envelope",
        )
        .unwrap();
        let signed: SignedRustResultEnvelope = serde_json::from_slice(&envelope_bytes).unwrap();
        let candidate = candidate_for_envelope(&signed).unwrap();

        let (left, right) =
            tokio::join!(fixture.source.publish_candidate(&candidate), fixture.source.publish_candidate(&candidate));

        let successes = [left.is_ok(), right.is_ok()].into_iter().filter(|value| *value).count();
        assert_eq!(successes, 2);
        let lookup = fixture
            .source
            .lookup(&fixture.action.action_ref, MAX_SHARED_CANDIDATES as u32, MAX_SHARED_METADATA_BYTES)
            .await
            .unwrap();
        assert_eq!(lookup.candidates, [candidate]);
    }

    #[tokio::test]
    async fn concurrent_different_immutable_writers_never_overwrite() {
        let root = tempfile::tempdir().unwrap();
        let destination = root.path().join("immutable.json");
        let left_bytes = b"left-result".to_vec();
        let right_bytes = b"right-result".to_vec();

        let (left, right) = tokio::join!(async { publish_immutable_bytes(&destination, &left_bytes) }, async {
            publish_immutable_bytes(&destination, &right_bytes)
        });

        assert_ne!(left.is_ok(), right.is_ok());
        let final_bytes = fs::read(&destination).unwrap();
        assert!(final_bytes == left_bytes || final_bytes == right_bytes);
        assert_ne!(left_bytes, right_bytes);
    }

    #[tokio::test]
    async fn http_redirect_is_rejected_without_following_location() {
        let response = concat!(
            "HTTP/1.1 302 Found\r\n",
            "Location: http://127.0.0.1:1/forbidden\r\n",
            "Content-Length: 0\r\n",
            "Connection: close\r\n\r\n"
        );
        let (base_url, server) = spawn_http_response(response.as_bytes().to_vec(), HTTP_TEST_NO_DELAY);
        let source = HttpRustResultSource::new(&base_url, HTTP_TEST_DELAY).unwrap();

        let error = source
            .lookup(&test_action().action_ref, MAX_SHARED_CANDIDATES as u32, MAX_SHARED_METADATA_BYTES)
            .await
            .unwrap_err();

        assert_eq!(error.code, "shared-http-lookup-redirect-rejected");
        server.join().unwrap();
    }

    #[tokio::test]
    async fn http_timeout_is_bounded_and_rejected() {
        let response = concat!("HTTP/1.1 404 Not Found\r\n", "Content-Length: 0\r\n", "Connection: close\r\n\r\n");
        let (base_url, server) = spawn_http_response(response.as_bytes().to_vec(), HTTP_TEST_DELAY);
        let source = HttpRustResultSource::new(&base_url, HTTP_TEST_TIMEOUT).unwrap();

        let error = source
            .lookup(&test_action().action_ref, MAX_SHARED_CANDIDATES as u32, MAX_SHARED_METADATA_BYTES)
            .await
            .unwrap_err();

        assert_eq!(error.code, "shared-http-timeout");
        server.join().unwrap();
    }

    #[tokio::test]
    async fn http_declared_metadata_limit_is_enforced_before_body_read() {
        let response = concat!("HTTP/1.1 200 OK\r\n", "Content-Length: 1024\r\n", "Connection: close\r\n\r\n");
        let (base_url, server) = spawn_http_response(response.as_bytes().to_vec(), HTTP_TEST_NO_DELAY);
        let source = HttpRustResultSource::new(&base_url, HTTP_TEST_DELAY).unwrap();

        let error = source
            .lookup(&test_action().action_ref, MAX_SHARED_CANDIDATES as u32, HTTP_TEST_BODY_LIMIT)
            .await
            .unwrap_err();

        assert_eq!(error.code, "shared-http-metadata-too-large");
        server.join().unwrap();
    }

    #[tokio::test]
    async fn corrupt_object_is_rejected_before_local_admission() {
        let fixture = published_fixture().await;
        let object_path = fixture.source.object_path(&fixture.publication.object_ref).unwrap();
        let mut bytes = fs::read(&object_path).unwrap();
        bytes[0] ^= 1;
        fs::write(&object_path, bytes).unwrap();
        let output = fixture.root.path().join("corrupt-output");
        let source: Arc<dyn RustResultSource> = fixture.source.clone();

        let report = fixture
            .client
            .restore_shared(SharedRestoreRequest {
                action: &fixture.action,
                output_dir: &output,
                local_policy: &local_policy(),
                shared_policy: &shared_policy(),
                trust_policy: &fixture.trust,
                sources: &[source],
            })
            .await
            .unwrap();

        assert_eq!(report.disposition, SHARED_CACHE_REJECTED);
        assert!(!output.exists());
        assert!(report.observations.iter().any(|item| item.reason_codes == ["shared-candidate-state-rejected"]));
    }

    #[tokio::test]
    async fn truncated_object_rejects_incomplete_tree_transfer() {
        let fixture = published_fixture().await;
        let object_path = fixture.source.object_path(&fixture.publication.object_ref).unwrap();
        let file = OpenOptions::new().write(true).open(&object_path).unwrap();
        let truncated_bytes = fixture.publication.object_bytes.checked_sub(1).unwrap();
        file.set_len(truncated_bytes).unwrap();
        let output = fixture.root.path().join("truncated-output");
        let source: Arc<dyn RustResultSource> = fixture.source.clone();

        let report = fixture
            .client
            .restore_shared(SharedRestoreRequest {
                action: &fixture.action,
                output_dir: &output,
                local_policy: &local_policy(),
                shared_policy: &shared_policy(),
                trust_policy: &fixture.trust,
                sources: &[source],
            })
            .await
            .unwrap();

        assert_eq!(report.disposition, SHARED_CACHE_REJECTED);
        assert!(!output.exists());
        assert!(report.observations.iter().any(|item| item.reason_codes == ["shared-candidate-state-rejected"]));
    }

    #[test]
    fn malformed_candidate_reference_is_rejected_before_discovery() {
        let fixture = fixture_candidate_claim();
        let mut malformed = fixture;
        malformed.result_ref = "mantle-rust-result://blake3/not-hex".to_string();

        let error = validate_candidate_claim(&malformed).unwrap_err();

        assert_eq!(error.code, "shared-candidate-result-ref-invalid");
        assert!(!error.code.is_empty());
    }

    #[tokio::test]
    async fn conflicting_admissible_results_block_strong_reuse() {
        let root = tempfile::tempdir().unwrap();
        let producer = test_cache(&root.path().join("producer")).await;
        let client = test_cache(&root.path().join("client")).await;
        let source = Arc::new(DirectoryRustResultSource::open(root.path().join("shared"), true).unwrap());
        let action = test_action();
        let output = root.path().join("producer-output");
        let signing_key = signing_key(TEST_KEY_BYTE);
        let trust = trust_policy(&signing_key);
        let local_policy = local_policy();
        let shared_policy = shared_policy();
        write_output(&output, b"first");
        let first = producer
            .publish(PublishRequest {
                action: &action,
                output_dir: &output,
                producer_receipt_ref: &receipt_ref(),
                policy: &local_policy,
            })
            .await
            .unwrap();
        fs::remove_dir_all(&output).unwrap();
        write_output(&output, b"second");
        let second = producer
            .publish(PublishRequest {
                action: &action,
                output_dir: &output,
                producer_receipt_ref: &receipt_ref(),
                policy: &local_policy,
            })
            .await
            .unwrap();
        producer
            .publish_shared(source.as_ref(), shared_publish_request(&first, &signing_key, &shared_policy))
            .await
            .unwrap();
        producer
            .publish_shared(source.as_ref(), shared_publish_request(&second, &signing_key, &shared_policy))
            .await
            .unwrap();
        let restored = root.path().join("conflict-output");
        let remote_source: Arc<dyn RustResultSource> = source;

        let report = client
            .restore_shared(SharedRestoreRequest {
                action: &action,
                output_dir: &restored,
                local_policy: &local_policy,
                shared_policy: &shared_policy,
                trust_policy: &trust,
                sources: &[remote_source],
            })
            .await
            .unwrap();

        assert_eq!(report.disposition, SHARED_CACHE_CONFLICT);
        assert_eq!(report.candidate_count, 2);
        assert!(report.selected_result_ref.is_none());
        assert!(!restored.exists());
    }

    #[test]
    fn oversized_candidate_index_is_rejected_before_file_reads() {
        let entries = vec![PathBuf::from("one"), PathBuf::from("two")];
        let error =
            load_candidate_entries(entries, &test_action().action_ref, 1, MAX_SHARED_METADATA_BYTES).unwrap_err();

        assert_eq!(error.code, "shared-candidate-limit-exceeded");
        assert!(!error.code.is_empty());
    }

    struct PublishedFixture {
        root: tempfile::TempDir,
        client: RustCache,
        source: Arc<DirectoryRustResultSource>,
        action: RustUnitAction,
        trust: RustResultTrustPolicy,
        publication: SharedPublishReport,
    }

    async fn published_fixture() -> PublishedFixture {
        let root = tempfile::tempdir().unwrap();
        let producer = test_cache(&root.path().join("producer")).await;
        let client = test_cache(&root.path().join("client")).await;
        let source = Arc::new(DirectoryRustResultSource::open(root.path().join("shared"), true).unwrap());
        let action = test_action();
        let output = root.path().join("producer-output");
        write_output(&output, b"fixture");
        let result = producer
            .publish(PublishRequest {
                action: &action,
                output_dir: &output,
                producer_receipt_ref: &receipt_ref(),
                policy: &local_policy(),
            })
            .await
            .unwrap();
        let signing_key = signing_key(TEST_KEY_BYTE);
        let trust = trust_policy(&signing_key);
        let publication = producer
            .publish_shared(source.as_ref(), shared_publish_request(&result, &signing_key, &shared_policy()))
            .await
            .unwrap();
        PublishedFixture {
            root,
            client,
            source,
            action,
            trust,
            publication,
        }
    }

    async fn published_fixture_without_candidate() -> PublishedFixture {
        let fixture = published_fixture().await;
        let envelope_bytes = read_source_file_bounded(
            &fixture.source.envelope_path(&fixture.publication.envelope_ref).unwrap(),
            MAX_SHARED_METADATA_BYTES,
            "test-envelope",
        )
        .unwrap();
        let signed: SignedRustResultEnvelope = serde_json::from_slice(&envelope_bytes).unwrap();
        let candidate = candidate_for_envelope(&signed).unwrap();
        fs::remove_file(fixture.source.candidate_path(&candidate).unwrap()).unwrap();
        fixture
    }

    async fn test_cache(state_dir: &Path) -> RustCache {
        let blob_service = Arc::new(MemoryBlobService::default()) as Arc<dyn BlobService>;
        let directory_service = Arc::new(
            RedbDirectoryService::new_temporary(
                format!("shared-test-{}", blake3::hash(state_dir.as_os_str().as_encoded_bytes()).to_hex()),
                RedbDirectoryServiceConfig::default(),
            )
            .unwrap(),
        ) as Arc<dyn DirectoryService>;
        RustCache::new(state_dir.to_path_buf(), blob_service, directory_service).unwrap()
    }

    fn shared_publish_request<'a>(
        result: &'a RustUnitResult,
        signing_key: &'a ed25519_dalek::SigningKey,
        policy: &'a SharedRustCachePolicy,
    ) -> SharedPublishRequest<'a> {
        SharedPublishRequest {
            result,
            producer: RustResultProducerIdentity {
                producer_id: "test-producer".to_string(),
                producer_policy_id: "test-producer-policy-v1".to_string(),
            },
            signer_name: "test-key".to_string(),
            signing_key,
            policy,
        }
    }

    fn trust_policy(signing_key: &ed25519_dalek::SigningKey) -> RustResultTrustPolicy {
        RustResultTrustPolicy {
            schema: SHARED_RUST_TRUST_POLICY_SCHEMA.to_string(),
            policy_id: "test-trust-policy-v1".to_string(),
            accepted_producer_policy_ids: vec!["test-producer-policy-v1".to_string()],
            trusted_keys: vec![TrustedRustResultKey {
                signer_name: "test-key".to_string(),
                verifier_key_hex: data_encoding::HEXLOWER.encode(signing_key.verifying_key().as_bytes()),
            }],
        }
    }

    fn local_policy() -> LocalCachePolicy {
        LocalCachePolicy {
            reads_enabled: true,
            writes_enabled: true,
            ..LocalCachePolicy::default()
        }
    }

    fn shared_policy() -> SharedRustCachePolicy {
        SharedRustCachePolicy {
            reads_enabled: true,
            publishes_enabled: true,
            ..SharedRustCachePolicy::default()
        }
    }

    fn signing_key(byte: u8) -> ed25519_dalek::SigningKey {
        ed25519_dalek::SigningKey::from_bytes(&[byte; crunch_rust_cache_core::shared::ED25519_PUBLIC_KEY_BYTES])
    }

    fn receipt_ref() -> String {
        format!(
            "mantle-rust-receipt://blake3/{}",
            std::iter::repeat_n(RECEIPT_DIGEST_CHAR, crunch_rust_cache_core::BLAKE3_HEX_CHARS).collect::<String>()
        )
    }

    fn test_action() -> RustUnitAction {
        let digest =
            std::iter::repeat_n(ACTION_DIGEST_CHAR, crunch_rust_cache_core::BLAKE3_HEX_CHARS).collect::<String>();
        canonical_rust_action(RustUnitActionInput {
            unit_id: "unit".to_string(),
            package_id: "package".to_string(),
            crate_name: "crate".to_string(),
            target_kind: "lib".to_string(),
            execution_kind: "target".to_string(),
            host_triple: "x86_64-unknown-linux-gnu".to_string(),
            target_triple: "x86_64-unknown-linux-gnu".to_string(),
            profile: "debug".to_string(),
            mode: "build".to_string(),
            features: Vec::new(),
            source_digest_blake3: digest.clone(),
            compiler_digest_blake3: digest.clone(),
            compiler_version_digest_blake3: digest.clone(),
            toolchain_closure_digest_blake3: digest.clone(),
            execution_platform_digest_blake3: digest.clone(),
            semantic_arguments: vec![RustSemanticArgument {
                value: "--crate-type=lib".to_string(),
                contains_absolute_path: false,
                absolute_paths_classified: false,
            }],
            admitted_environment: BTreeMap::new(),
            dependency_artifacts: Vec::new(),
            host_artifacts: Vec::new(),
            build_script_facts: vec![RustBuildFact {
                name: "none".to_string(),
                value_digest_blake3: digest.clone(),
            }],
            native_link_facts: Vec::new(),
            compiler_policy_digest_blake3: digest,
        })
        .unwrap()
    }

    fn fixture_candidate_claim() -> SharedRustCandidateClaim {
        let action = test_action();
        let claim = SharedRustCandidateClaim {
            schema: SHARED_CANDIDATE_SCHEMA.to_string(),
            action_ref: action.action_ref,
            envelope_ref: format!(
                "{}{}",
                SHARED_RUST_ENVELOPE_REF_PREFIX,
                "b".repeat(crunch_rust_cache_core::BLAKE3_HEX_CHARS)
            ),
            result_ref: format!(
                "{}{}",
                crunch_rust_cache_core::RUST_RESULT_REF_PREFIX,
                "c".repeat(crunch_rust_cache_core::BLAKE3_HEX_CHARS)
            ),
        };
        validate_candidate_claim(&claim).unwrap();
        assert!(!claim.action_ref.is_empty());
        assert!(!claim.envelope_ref.is_empty());
        claim
    }

    fn write_output(output: &Path, bytes: &[u8]) {
        fs::create_dir_all(output).unwrap();
        fs::write(output.join("artifact.rlib"), bytes).unwrap();
    }

    fn spawn_http_response(response: Vec<u8>, delay: Duration) -> (String, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0_u8; HTTP_TEST_READ_BYTES];
            let count = stream.read(&mut request).unwrap();
            assert!(count > 0);
            assert!(request[..count].starts_with(b"GET "));
            thread::sleep(delay);
            let _write_result = stream.write_all(&response);
        });
        let url = format!("http://{address}/cache");
        assert!(url.starts_with("http://127.0.0.1:"));
        assert!(!url.contains('@'));
        (url, handle)
    }

    struct CountingRemoteSource {
        opens: Arc<AtomicU32>,
    }

    #[async_trait]
    impl RustResultSource for CountingRemoteSource {
        fn source_id(&self) -> &str {
            "mantle-rust-source://blake3/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        }

        fn is_remote(&self) -> bool {
            true
        }

        async fn lookup(
            &self,
            _action_ref: &str,
            _max_candidates: u32,
            _max_metadata_bytes: u64,
        ) -> Result<SharedSourceLookup, SourceError> {
            self.opens.fetch_add(1, Ordering::SeqCst);
            Err(SourceError::new("offline-source-opened"))
        }

        async fn fetch_envelope(&self, _envelope_ref: &str, _max_metadata_bytes: u64) -> Result<Vec<u8>, SourceError> {
            self.opens.fetch_add(1, Ordering::SeqCst);
            Err(SourceError::new("offline-source-opened"))
        }

        async fn fetch_object(
            &self,
            _object_ref: &str,
            _destination: &Path,
            _max_transfer_bytes: u64,
        ) -> Result<u64, SourceError> {
            self.opens.fetch_add(1, Ordering::SeqCst);
            Err(SourceError::new("offline-source-opened"))
        }

        async fn publish_object(&self, _object_ref: &str, _source: &Path, _size_bytes: u64) -> Result<(), SourceError> {
            self.opens.fetch_add(1, Ordering::SeqCst);
            Err(SourceError::new("offline-source-opened"))
        }

        async fn publish_envelope(&self, _envelope_ref: &str, _bytes: &[u8]) -> Result<(), SourceError> {
            self.opens.fetch_add(1, Ordering::SeqCst);
            Err(SourceError::new("offline-source-opened"))
        }

        async fn publish_candidate(&self, _candidate: &SharedRustCandidateClaim) -> Result<(), SourceError> {
            self.opens.fetch_add(1, Ordering::SeqCst);
            Err(SourceError::new("offline-source-opened"))
        }
    }
}
