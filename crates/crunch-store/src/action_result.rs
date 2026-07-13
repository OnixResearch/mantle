//! Durable local and bounded HTTP shells for shared action-result discovery.
//!
//! CAS services, this action-result store, and build executors remain separate
//! interfaces. Indexes are advisory: this module validates immutable bytes and
//! publication ordering but does not admit outputs for reuse.

use std::collections::BTreeSet;
use std::collections::HashSet;
use std::fmt;
use std::io::ErrorKind;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::time::Duration;

use async_trait::async_trait;
use crunch_action_result_core::ACTION_REF_PREFIX;
use crunch_action_result_core::ACTION_RESULT_REF_PREFIX;
use crunch_action_result_core::ActionResultIndex;
use crunch_action_result_core::SignedActionResultRecord;
use crunch_action_result_core::canonical_action_result_index;
use crunch_action_result_core::canonical_index_bytes;
use crunch_action_result_core::canonical_signed_record_bytes;
use crunch_action_result_core::validate_action_result;
use crunch_action_result_core::validate_action_result_index;
use futures::StreamExt;
use reqwest::StatusCode;
use reqwest::Url;
use reqwest::header::CONTENT_LENGTH;
use reqwest::header::ETAG;
use reqwest::header::IF_MATCH;
use reqwest::header::IF_NONE_MATCH;
use serde::Deserialize;

pub const ACTION_RESULT_STORE_LAYOUT_VERSION: &str = "v1";
pub const LOCAL_ACTION_RESULT_SOURCE_ID: &str = "local-action-results";
pub const HTTP_ACTION_RESULT_SOURCE_CLASS: &str = "http";
pub const LOCAL_ACTION_RESULT_SOURCE_CLASS: &str = "local";
pub const DEFAULT_ACTION_RESULT_HTTP_TIMEOUT_MS: u64 = 5_000;
pub const MAX_ACTION_RESULT_HTTP_RETRIES: u32 = 4;
pub const MAX_ACTION_RESULT_SOURCES: usize = 16;
pub const MAX_ACTION_RESULT_TOTAL_CANDIDATES: usize = 256;

const ACTION_RESULT_POLICY_SCHEMA: &str = "mantle-action-result-runtime-policy-v1";
const ACTION_RESULT_POLICY_JSON: &str =
    include_str!("../../../config/action-result-policy/generated/action-result-policy.json");
static ACTION_RESULT_RUNTIME_POLICY: OnceLock<ActionResultRuntimePolicy> = OnceLock::new();

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ActionResultRuntimePolicy {
    pub schema: String,
    pub hash_algorithm: String,
    pub sources: ActionResultSourcePolicy,
    pub trust: ActionResultTrustRuntimePolicy,
    pub limits: ActionResultLimitPolicy,
    pub offline: ActionResultOfflinePolicy,
    pub publication: ActionResultPublicationPolicy,
    pub claims: ActionResultClaimPolicy,
    pub gc: ActionResultGcPolicy,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ActionResultSourcePolicy {
    pub allowed_classes: Vec<String>,
    pub local_enabled: bool,
    pub http_enabled: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ActionResultTrustRuntimePolicy {
    pub key_source: String,
    pub require_record_signature: bool,
    pub require_pathinfo_signature: bool,
    pub require_producer_identity_match: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ActionResultLimitPolicy {
    pub max_sources: usize,
    pub max_candidates: usize,
    pub max_record_bytes: usize,
    pub max_index_bytes: usize,
    pub max_outputs: usize,
    pub max_signatures: usize,
    pub http_timeout_ms: u64,
    pub http_retry_attempts: u32,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ActionResultOfflinePolicy {
    pub local_lookup: bool,
    pub remote_lookup: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ActionResultPublicationPolicy {
    pub local_enabled: bool,
    pub http_enabled: bool,
    pub record_before_index: bool,
    pub atomic_visibility: bool,
    pub immutable_no_clobber: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ActionResultClaimPolicy {
    pub required_strength: String,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ActionResultGcPolicy {
    pub retention_basis: String,
    pub candidate_metadata_roots_outputs: bool,
    pub max_metadata_entries: usize,
}

pub fn action_result_runtime_policy() -> &'static ActionResultRuntimePolicy {
    ACTION_RESULT_RUNTIME_POLICY.get_or_init(|| {
        let policy: ActionResultRuntimePolicy = serde_json::from_str(ACTION_RESULT_POLICY_JSON)
            .expect("checked-in action-result policy must be valid JSON");
        assert_eq!(policy.schema, ACTION_RESULT_POLICY_SCHEMA);
        assert_eq!(policy.hash_algorithm, "BLAKE3");
        assert_eq!(policy.sources.allowed_classes, [LOCAL_ACTION_RESULT_SOURCE_CLASS, HTTP_ACTION_RESULT_SOURCE_CLASS]);
        assert_eq!(policy.claims.required_strength, crunch_action_result_core::STRONG_CLAIM);
        assert!(policy.claims.non_claims.contains(&"ca-mapping-presence-is-not-output-trust".to_string()));
        assert!(policy.claims.non_claims.contains(&"index-presence-is-not-output-trust".to_string()));
        assert_eq!(policy.limits.max_sources, MAX_ACTION_RESULT_SOURCES);
        assert_eq!(policy.limits.max_candidates, MAX_ACTION_RESULT_TOTAL_CANDIDATES);
        assert_eq!(policy.limits.max_record_bytes, crunch_action_result_core::MAX_ACTION_RESULT_RECORD_BYTES);
        assert_eq!(policy.limits.max_index_bytes, crunch_action_result_core::MAX_ACTION_RESULT_INDEX_BYTES);
        assert_eq!(policy.limits.max_outputs, crunch_action_result_core::MAX_ACTION_RESULT_OUTPUTS);
        assert_eq!(policy.limits.max_signatures, crunch_action_result_core::MAX_ACTION_RESULT_SIGNATURE_REFS);
        assert_eq!(policy.limits.http_timeout_ms, DEFAULT_ACTION_RESULT_HTTP_TIMEOUT_MS);
        assert_eq!(policy.limits.http_retry_attempts, MAX_ACTION_RESULT_HTTP_RETRIES);
        assert!(policy.trust.require_record_signature);
        assert!(policy.trust.require_pathinfo_signature);
        assert!(policy.trust.require_producer_identity_match);
        assert!(policy.publication.record_before_index);
        assert!(policy.publication.atomic_visibility);
        assert!(policy.publication.immutable_no_clobber);
        assert!(policy.offline.local_lookup);
        assert!(!policy.offline.remote_lookup);
        assert_eq!(policy.gc.retention_basis, "live-admitted-store-paths");
        assert!(!policy.gc.candidate_metadata_roots_outputs);
        policy
    })
}

const ACTION_RESULTS_DIR: &str = "action-results";
const RECORDS_DIR: &str = "records";
const INDEXES_DIR: &str = "indexes";
const RECORD_EXTENSION: &str = "json";
const INDEX_MARKER_EXTENSION: &str = "ref";
const HTTP_INDEX_FILE_EXTENSION: &str = "json";
const INDEX_MARKER_NEWLINE: &[u8] = b"\n";
const HTTP_USER_AGENT_PREFIX: &str = "mantle-action-result";
const TEMP_FILE_SEQUENCE_START: u64 = 1;
static TEMP_FILE_SEQUENCE: AtomicU64 = AtomicU64::new(TEMP_FILE_SEQUENCE_START);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionResultPublicationStatus {
    Published,
    Duplicate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionResultPublicationReport {
    pub result_ref: String,
    pub record_status: ActionResultPublicationStatus,
    pub index_status: ActionResultPublicationStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionResultLookup {
    pub source_id: String,
    pub source_class: String,
    pub index: ActionResultIndex,
    pub records: Vec<SignedActionResultRecord>,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionResultDiscoveryReport {
    pub lookups: Vec<ActionResultLookup>,
    pub diagnostics: Vec<String>,
    pub remote_sources_opened: u32,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ActionResultGcCandidates {
    pub record_paths: Vec<PathBuf>,
    pub index_marker_paths: Vec<PathBuf>,
}

pub fn local_action_result_gc_candidates(
    state_dir: &Path,
    live_store_paths: &BTreeSet<String>,
) -> Result<ActionResultGcCandidates, String> {
    let store = LocalActionResultStore::new(state_dir);
    let mut candidates = ActionResultGcCandidates::default();
    let mut retained_result_refs = HashSet::new();
    let mut visited_entries = 0usize;
    collect_record_gc_candidates(
        &store,
        live_store_paths,
        &mut retained_result_refs,
        &mut candidates,
        &mut visited_entries,
    )?;
    collect_index_gc_candidates(&store, &retained_result_refs, &mut candidates, &mut visited_entries)?;
    candidates.record_paths.sort();
    candidates.index_marker_paths.sort();
    Ok(candidates)
}

// r[impl cache_substitution.shared_action_result_discovery]
#[async_trait]
pub trait ActionResultStore: Send + Sync + fmt::Debug {
    fn source_id(&self) -> &str;
    fn source_class(&self) -> &str;
    async fn lookup(&self, action_ref: &str) -> Result<ActionResultLookup, String>;
    async fn publish(&self, record: &SignedActionResultRecord) -> Result<ActionResultPublicationReport, String>;
}

#[derive(Debug, Clone)]
pub struct LocalActionResultStore {
    root: PathBuf,
    source_id: String,
}

impl LocalActionResultStore {
    pub fn new(state_dir: &Path) -> Self {
        assert!(!state_dir.as_os_str().is_empty(), "state_dir must not be empty");
        Self {
            root: state_dir.join(ACTION_RESULTS_DIR).join(ACTION_RESULT_STORE_LAYOUT_VERSION),
            source_id: LOCAL_ACTION_RESULT_SOURCE_ID.to_string(),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn record_path(&self, result_ref: &str) -> Result<PathBuf, String> {
        let digest = ref_digest(result_ref, ACTION_RESULT_REF_PREFIX)?;
        Ok(self.root.join(RECORDS_DIR).join(format!("{digest}.{RECORD_EXTENSION}")))
    }

    fn index_dir(&self, action_ref: &str) -> Result<PathBuf, String> {
        let digest = ref_digest(action_ref, ACTION_REF_PREFIX)?;
        Ok(self.root.join(INDEXES_DIR).join(digest))
    }

    fn marker_path(&self, action_ref: &str, result_ref: &str) -> Result<PathBuf, String> {
        let result_digest = ref_digest(result_ref, ACTION_RESULT_REF_PREFIX)?;
        Ok(self.index_dir(action_ref)?.join(format!("{result_digest}.{INDEX_MARKER_EXTENSION}")))
    }

    fn lookup_sync(&self, action_ref: &str) -> Result<ActionResultLookup, String> {
        let result_refs = self.read_index_markers(action_ref)?;
        let index = canonical_action_result_index(action_ref.to_string(), result_refs)?;
        let mut records = Vec::with_capacity(index.result_refs.len());
        for result_ref in &index.result_refs {
            records.push(self.read_record(result_ref)?);
        }
        Ok(ActionResultLookup {
            source_id: self.source_id.clone(),
            source_class: LOCAL_ACTION_RESULT_SOURCE_CLASS.to_string(),
            index,
            records,
            diagnostics: Vec::new(),
        })
    }

    fn publish_sync(&self, signed: &SignedActionResultRecord) -> Result<ActionResultPublicationReport, String> {
        validate_signed_record(signed)?;
        let record_bytes = canonical_signed_record_bytes(signed)?;
        let record_path = self.record_path(&signed.record.result_ref)?;
        let record_status = publish_bytes_no_clobber(&record_path, &record_bytes)?;
        let marker_path = self.marker_path(&signed.record.action_ref, &signed.record.result_ref)?;
        let mut marker_bytes = signed.record.result_ref.as_bytes().to_vec();
        marker_bytes.extend_from_slice(INDEX_MARKER_NEWLINE);
        let index_status = publish_bytes_no_clobber(&marker_path, &marker_bytes)?;
        Ok(ActionResultPublicationReport {
            result_ref: signed.record.result_ref.clone(),
            record_status,
            index_status,
        })
    }

    fn read_index_markers(&self, action_ref: &str) -> Result<Vec<String>, String> {
        let index_dir = self.index_dir(action_ref)?;
        let read_dir = match std::fs::read_dir(&index_dir) {
            Ok(read_dir) => read_dir,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(format!("action-result-local-index-read:{}:{error}", index_dir.display())),
        };
        let mut result_refs = Vec::new();
        for entry in read_dir {
            if result_refs.len() >= action_result_runtime_policy().limits.max_candidates {
                return Err("action-result-local-index-candidate-limit-exceeded".to_string());
            }
            let entry = entry.map_err(|error| format!("action-result-local-index-entry:{error}"))?;
            let file_type =
                entry.file_type().map_err(|error| format!("action-result-local-index-entry-type:{error}"))?;
            if !file_type.is_file() {
                return Err("action-result-local-index-entry-not-file".to_string());
            }
            if entry.path().extension().and_then(|value| value.to_str()) != Some(INDEX_MARKER_EXTENSION) {
                return Err("action-result-local-index-entry-name-invalid".to_string());
            }
            let bytes = read_bounded_file(&entry.path(), crunch_action_result_core::MAX_ACTION_RESULT_INDEX_BYTES)?;
            let result_ref = String::from_utf8(bytes)
                .map_err(|_| "action-result-local-index-marker-not-utf8".to_string())?
                .trim()
                .to_string();
            ref_digest(&result_ref, ACTION_RESULT_REF_PREFIX)?;
            result_refs.push(result_ref);
        }
        result_refs.sort();
        result_refs.dedup();
        Ok(result_refs)
    }

    fn read_record(&self, result_ref: &str) -> Result<SignedActionResultRecord, String> {
        let path = self.record_path(result_ref)?;
        let bytes = read_bounded_file(&path, crunch_action_result_core::MAX_ACTION_RESULT_RECORD_BYTES)
            .map_err(|error| format!("action-result-local-record-unavailable:{result_ref}:{error}"))?;
        let signed: SignedActionResultRecord = serde_json::from_slice(&bytes)
            .map_err(|error| format!("action-result-local-record-json-invalid:{result_ref}:{error}"))?;
        validate_signed_record(&signed)?;
        if signed.record.result_ref != result_ref {
            return Err("action-result-local-record-index-mismatch".to_string());
        }
        Ok(signed)
    }
}

fn collect_record_gc_candidates(
    store: &LocalActionResultStore,
    live_store_paths: &BTreeSet<String>,
    retained_result_refs: &mut HashSet<String>,
    candidates: &mut ActionResultGcCandidates,
    visited_entries: &mut usize,
) -> Result<(), String> {
    let records_dir = store.root.join(RECORDS_DIR);
    let entries = match std::fs::read_dir(&records_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(format!("action-result-gc-record-dir:{}:{error}", records_dir.display())),
    };
    for entry in entries {
        check_gc_entry_budget(visited_entries)?;
        let entry = entry.map_err(|error| format!("action-result-gc-record-entry:{error}"))?;
        let path = entry.path();
        if !entry.file_type().map_err(|error| format!("action-result-gc-record-type:{error}"))?.is_file() {
            return Err("action-result-gc-record-entry-not-file".to_string());
        }
        let bytes = read_bounded_file(&path, action_result_runtime_policy().limits.max_record_bytes);
        let signed = bytes
            .ok()
            .and_then(|bytes| serde_json::from_slice::<SignedActionResultRecord>(&bytes).ok())
            .filter(|signed| validate_signed_record(signed).is_ok());
        let Some(signed) = signed else {
            candidates.record_paths.push(path);
            continue;
        };
        let all_outputs_live = signed.record.outputs.iter().all(|output| live_store_paths.contains(&output.store_path));
        if all_outputs_live {
            retained_result_refs.insert(signed.record.result_ref);
        } else {
            candidates.record_paths.push(path);
        }
    }
    Ok(())
}

fn collect_index_gc_candidates(
    store: &LocalActionResultStore,
    retained_result_refs: &HashSet<String>,
    candidates: &mut ActionResultGcCandidates,
    visited_entries: &mut usize,
) -> Result<(), String> {
    let indexes_dir = store.root.join(INDEXES_DIR);
    let action_dirs = match std::fs::read_dir(&indexes_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(format!("action-result-gc-index-dir:{}:{error}", indexes_dir.display())),
    };
    for action_dir in action_dirs {
        check_gc_entry_budget(visited_entries)?;
        let action_dir = action_dir.map_err(|error| format!("action-result-gc-index-entry:{error}"))?;
        if !action_dir.file_type().map_err(|error| format!("action-result-gc-index-type:{error}"))?.is_dir() {
            return Err("action-result-gc-index-entry-not-directory".to_string());
        }
        collect_action_index_gc_candidates(&action_dir.path(), retained_result_refs, candidates, visited_entries)?;
    }
    Ok(())
}

fn collect_action_index_gc_candidates(
    action_dir: &Path,
    retained_result_refs: &HashSet<String>,
    candidates: &mut ActionResultGcCandidates,
    visited_entries: &mut usize,
) -> Result<(), String> {
    let markers = std::fs::read_dir(action_dir)
        .map_err(|error| format!("action-result-gc-action-index:{}:{error}", action_dir.display()))?;
    for marker in markers {
        check_gc_entry_budget(visited_entries)?;
        let marker = marker.map_err(|error| format!("action-result-gc-index-marker:{error}"))?;
        let path = marker.path();
        if !marker.file_type().map_err(|error| format!("action-result-gc-marker-type:{error}"))?.is_file() {
            return Err("action-result-gc-index-marker-not-file".to_string());
        }
        let result_ref = read_bounded_file(&path, action_result_runtime_policy().limits.max_index_bytes)
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .map(|value| value.trim().to_string());
        if !result_ref.as_ref().is_some_and(|result_ref| retained_result_refs.contains(result_ref)) {
            candidates.index_marker_paths.push(path);
        }
    }
    Ok(())
}

fn check_gc_entry_budget(visited_entries: &mut usize) -> Result<(), String> {
    *visited_entries =
        visited_entries.checked_add(1).ok_or_else(|| "action-result-gc-entry-count-overflow".to_string())?;
    if *visited_entries > action_result_runtime_policy().gc.max_metadata_entries {
        return Err("action-result-gc-entry-limit-exceeded".to_string());
    }
    Ok(())
}

#[async_trait]
impl ActionResultStore for LocalActionResultStore {
    fn source_id(&self) -> &str {
        &self.source_id
    }

    fn source_class(&self) -> &str {
        LOCAL_ACTION_RESULT_SOURCE_CLASS
    }

    async fn lookup(&self, action_ref: &str) -> Result<ActionResultLookup, String> {
        self.lookup_sync(action_ref)
    }

    async fn publish(&self, record: &SignedActionResultRecord) -> Result<ActionResultPublicationReport, String> {
        self.publish_sync(record)
    }
}

#[derive(Clone)]
pub struct HttpActionResultStore {
    base_url: Url,
    client: reqwest::Client,
    source_id: String,
}

impl fmt::Debug for HttpActionResultStore {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HttpActionResultStore")
            .field("base_url", &self.base_url)
            .field("source_id", &self.source_id)
            .finish_non_exhaustive()
    }
}

impl HttpActionResultStore {
    pub fn new(base_url: Url, timeout: Duration) -> Result<Self, String> {
        validate_http_url(&base_url)?;
        if timeout.is_zero() {
            return Err("action-result-http-timeout-zero".to_string());
        }
        let base_url = normalize_http_base_url(&base_url);
        let client = reqwest::Client::builder()
            .user_agent(format!("{HTTP_USER_AGENT_PREFIX}/{}", env!("CARGO_PKG_VERSION")))
            .connect_timeout(timeout)
            .read_timeout(timeout)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|error| format!("action-result-http-client:{error}"))?;
        Ok(Self {
            source_id: base_url.as_str().to_string(),
            base_url,
            client,
        })
    }

    pub fn with_default_timeout(base_url: Url) -> Result<Self, String> {
        Self::new(base_url, Duration::from_millis(action_result_runtime_policy().limits.http_timeout_ms))
    }

    fn index_url(&self, action_ref: &str) -> Result<Url, String> {
        let digest = ref_digest(action_ref, ACTION_REF_PREFIX)?;
        self.base_url
            .join(&format!(
                "{ACTION_RESULTS_DIR}/{ACTION_RESULT_STORE_LAYOUT_VERSION}/{INDEXES_DIR}/{digest}.{HTTP_INDEX_FILE_EXTENSION}"
            ))
            .map_err(|error| format!("action-result-http-index-url:{error}"))
    }

    fn record_url(&self, result_ref: &str) -> Result<Url, String> {
        let digest = ref_digest(result_ref, ACTION_RESULT_REF_PREFIX)?;
        self.base_url
            .join(&format!(
                "{ACTION_RESULTS_DIR}/{ACTION_RESULT_STORE_LAYOUT_VERSION}/{RECORDS_DIR}/{digest}.{RECORD_EXTENSION}"
            ))
            .map_err(|error| format!("action-result-http-record-url:{error}"))
    }

    async fn fetch_index(&self, action_ref: &str) -> Result<(ActionResultIndex, Option<String>), String> {
        let url = self.index_url(action_ref)?;
        let response = self
            .client
            .get(url.clone())
            .send()
            .await
            .map_err(|error| format!("action-result-http-index-request:{url}:{error}"))?;
        if response.status() == StatusCode::NOT_FOUND {
            return Ok((canonical_action_result_index(action_ref.to_string(), Vec::new())?, None));
        }
        if !response.status().is_success() {
            return Err(format!("action-result-http-index-status:{url}:{}", response.status()));
        }
        let etag = response.headers().get(ETAG).and_then(|value| value.to_str().ok()).map(str::to_string);
        let bytes = read_bounded_response(response, crunch_action_result_core::MAX_ACTION_RESULT_INDEX_BYTES).await?;
        let index: ActionResultIndex = serde_json::from_slice(&bytes)
            .map_err(|error| format!("action-result-http-index-json-invalid:{url}:{error}"))?;
        validate_action_result_index(&index)?;
        if index.action_ref != action_ref {
            return Err("action-result-http-index-action-mismatch".to_string());
        }
        Ok((index, etag))
    }

    async fn fetch_record(&self, result_ref: &str) -> Result<SignedActionResultRecord, String> {
        let url = self.record_url(result_ref)?;
        let response = self
            .client
            .get(url.clone())
            .send()
            .await
            .map_err(|error| format!("action-result-http-record-request:{url}:{error}"))?;
        if !response.status().is_success() {
            return Err(format!("action-result-http-record-status:{url}:{}", response.status()));
        }
        let bytes = read_bounded_response(response, crunch_action_result_core::MAX_ACTION_RESULT_RECORD_BYTES).await?;
        let signed: SignedActionResultRecord = serde_json::from_slice(&bytes)
            .map_err(|error| format!("action-result-http-record-json-invalid:{url}:{error}"))?;
        validate_signed_record(&signed)?;
        if signed.record.result_ref != result_ref {
            return Err("action-result-http-record-index-mismatch".to_string());
        }
        Ok(signed)
    }

    async fn publish_record(&self, signed: &SignedActionResultRecord) -> Result<ActionResultPublicationStatus, String> {
        let bytes = canonical_signed_record_bytes(signed)?;
        let url = self.record_url(&signed.record.result_ref)?;
        let response = self
            .client
            .put(url.clone())
            .header(IF_NONE_MATCH, "*")
            .body(bytes.clone())
            .send()
            .await
            .map_err(|error| format!("action-result-http-record-publish:{url}:{error}"))?;
        if http_write_succeeded(response.status()) {
            return Ok(ActionResultPublicationStatus::Published);
        }
        if response.status() != StatusCode::PRECONDITION_FAILED {
            return Err(format!("action-result-http-record-publish-status:{url}:{}", response.status()));
        }
        let existing = self.fetch_record(&signed.record.result_ref).await?;
        let existing_bytes = canonical_signed_record_bytes(&existing)?;
        if existing_bytes != bytes {
            return Err("action-result-http-record-no-clobber-conflict".to_string());
        }
        Ok(ActionResultPublicationStatus::Duplicate)
    }

    async fn publish_index_candidate(
        &self,
        signed: &SignedActionResultRecord,
    ) -> Result<ActionResultPublicationStatus, String> {
        for _ in 0..action_result_runtime_policy().limits.http_retry_attempts {
            let (index, etag) = self.fetch_index(&signed.record.action_ref).await?;
            if index.result_refs.contains(&signed.record.result_ref) {
                return Ok(ActionResultPublicationStatus::Duplicate);
            }
            let mut result_refs = index.result_refs;
            result_refs.push(signed.record.result_ref.clone());
            let next = canonical_action_result_index(signed.record.action_ref.clone(), result_refs)?;
            let bytes = canonical_index_bytes(&next)?;
            let url = self.index_url(&signed.record.action_ref)?;
            let mut request = self.client.put(url.clone()).body(bytes);
            request = match etag {
                Some(etag) => request.header(IF_MATCH, etag),
                None => request.header(IF_NONE_MATCH, "*"),
            };
            let response =
                request.send().await.map_err(|error| format!("action-result-http-index-publish:{url}:{error}"))?;
            if http_write_succeeded(response.status()) {
                return Ok(ActionResultPublicationStatus::Published);
            }
            if response.status() != StatusCode::PRECONDITION_FAILED {
                return Err(format!("action-result-http-index-publish-status:{url}:{}", response.status()));
            }
        }
        Err("action-result-http-index-publication-retry-exhausted".to_string())
    }
}

#[async_trait]
impl ActionResultStore for HttpActionResultStore {
    fn source_id(&self) -> &str {
        &self.source_id
    }

    fn source_class(&self) -> &str {
        HTTP_ACTION_RESULT_SOURCE_CLASS
    }

    async fn lookup(&self, action_ref: &str) -> Result<ActionResultLookup, String> {
        let (index, _) = self.fetch_index(action_ref).await?;
        let mut records = Vec::with_capacity(index.result_refs.len());
        for result_ref in &index.result_refs {
            records.push(self.fetch_record(result_ref).await?);
        }
        Ok(ActionResultLookup {
            source_id: self.source_id.clone(),
            source_class: HTTP_ACTION_RESULT_SOURCE_CLASS.to_string(),
            index,
            records,
            diagnostics: Vec::new(),
        })
    }

    async fn publish(&self, signed: &SignedActionResultRecord) -> Result<ActionResultPublicationReport, String> {
        validate_signed_record(signed)?;
        let record_status = self.publish_record(signed).await?;
        let index_status = self.publish_index_candidate(signed).await?;
        Ok(ActionResultPublicationReport {
            result_ref: signed.record.result_ref.clone(),
            record_status,
            index_status,
        })
    }
}

#[derive(Debug)]
pub struct ActionResultStoreSet {
    local: Vec<Box<dyn ActionResultStore>>,
    remote: Vec<Box<dyn ActionResultStore>>,
    offline: bool,
    max_sources: usize,
    max_candidates: usize,
    publish_local_enabled: bool,
    publish_remote_enabled: bool,
}

impl ActionResultStoreSet {
    pub fn new(offline: bool) -> Self {
        Self {
            local: Vec::new(),
            remote: Vec::new(),
            offline,
            max_sources: action_result_runtime_policy().limits.max_sources,
            max_candidates: action_result_runtime_policy().limits.max_candidates,
            publish_local_enabled: action_result_runtime_policy().publication.local_enabled,
            publish_remote_enabled: action_result_runtime_policy().publication.http_enabled,
        }
    }

    pub fn add_local(&mut self, store: Box<dyn ActionResultStore>) {
        self.local.push(store);
    }

    pub fn add_remote(&mut self, store: Box<dyn ActionResultStore>) {
        self.remote.push(store);
    }

    pub async fn publish_local(
        &self,
        record: &SignedActionResultRecord,
    ) -> Result<Vec<ActionResultPublicationReport>, String> {
        if !self.publish_local_enabled {
            return Err("action-result-local-publication-policy-disabled".to_string());
        }
        let mut reports = Vec::with_capacity(self.local.len());
        for store in &self.local {
            reports.push(store.publish(record).await?);
        }
        Ok(reports)
    }

    pub async fn publish_remote(
        &self,
        record: &SignedActionResultRecord,
    ) -> Result<Vec<ActionResultPublicationReport>, String> {
        if !self.publish_remote_enabled {
            return Err("action-result-http-publication-policy-disabled".to_string());
        }
        if self.offline {
            return Err("action-result-offline-remote-publication-rejected".to_string());
        }
        let mut reports = Vec::with_capacity(self.remote.len());
        for store in &self.remote {
            reports.push(store.publish(record).await?);
        }
        Ok(reports)
    }

    pub async fn discover(&self, action_ref: &str) -> ActionResultDiscoveryReport {
        let mut report = ActionResultDiscoveryReport {
            lookups: Vec::new(),
            diagnostics: Vec::new(),
            remote_sources_opened: 0,
        };
        let mut attempted_sources = 0usize;
        discover_sources(
            &self.local,
            action_ref,
            false,
            self.max_sources,
            self.max_candidates,
            &mut attempted_sources,
            &mut report,
        )
        .await;
        if self.offline {
            if !self.remote.is_empty() {
                report.diagnostics.push("action-result-offline-remote-sources-skipped".to_string());
            }
            return report;
        }
        discover_sources(
            &self.remote,
            action_ref,
            true,
            self.max_sources,
            self.max_candidates,
            &mut attempted_sources,
            &mut report,
        )
        .await;
        report
    }
}

async fn discover_sources(
    stores: &[Box<dyn ActionResultStore>],
    action_ref: &str,
    is_remote: bool,
    max_sources: usize,
    max_candidates: usize,
    attempted_sources: &mut usize,
    report: &mut ActionResultDiscoveryReport,
) {
    for store in stores {
        if *attempted_sources >= max_sources {
            report.diagnostics.push("action-result-source-limit-exceeded".to_string());
            return;
        }
        *attempted_sources = attempted_sources.saturating_add(1);
        let current_candidates = report.lookups.iter().map(|lookup| lookup.records.len()).sum::<usize>();
        if current_candidates >= max_candidates {
            report.diagnostics.push("action-result-total-candidate-limit-exceeded".to_string());
            return;
        }
        if is_remote {
            report.remote_sources_opened = report.remote_sources_opened.saturating_add(1);
        }
        match store.lookup(action_ref).await {
            Ok(lookup) => {
                let remaining = max_candidates.saturating_sub(current_candidates);
                if lookup.records.len() > remaining {
                    report.diagnostics.push(format!(
                        "action-result-source-rejected:{}:action-result-total-candidate-limit-exceeded",
                        store.source_id()
                    ));
                    continue;
                }
                report.lookups.push(lookup);
            }
            Err(error) => {
                report.diagnostics.push(format!("action-result-source-rejected:{}:{error}", store.source_id()))
            }
        }
    }
}

fn validate_signed_record(signed: &SignedActionResultRecord) -> Result<(), String> {
    validate_action_result(&signed.record)?;
    if signed.record_signatures.is_empty() {
        return Err("action-result-record-signature-missing".to_string());
    }
    if signed.record_signatures.len() > action_result_runtime_policy().limits.max_signatures {
        return Err("action-result-record-signature-count-invalid".to_string());
    }
    let mut canonical_signatures = signed.record_signatures.clone();
    canonical_signatures.sort();
    canonical_signatures.dedup();
    if canonical_signatures != signed.record_signatures {
        return Err("action-result-record-signatures-not-canonical".to_string());
    }
    for signature in &signed.record_signatures {
        validate_detached_signature_shape(signature)?;
    }
    let bytes = canonical_signed_record_bytes(signed)?;
    if bytes.len() > crunch_action_result_core::MAX_ACTION_RESULT_RECORD_BYTES {
        return Err("action-result-signed-record-too-large".to_string());
    }
    Ok(())
}

fn validate_detached_signature_shape(
    signature: &crunch_action_result_core::DetachedRecordSignature,
) -> Result<(), String> {
    if signature.key_name.is_empty() || signature.key_name.len() > crunch_action_result_core::MAX_ACTION_RESULT_ID_BYTES
    {
        return Err("action-result-record-signature-key-invalid".to_string());
    }
    if signature.signature.is_empty()
        || signature.signature.len() > crunch_action_result_core::MAX_ACTION_RESULT_ID_BYTES
    {
        return Err("action-result-record-signature-value-invalid".to_string());
    }
    Ok(())
}

fn publish_bytes_no_clobber(path: &Path, bytes: &[u8]) -> Result<ActionResultPublicationStatus, String> {
    let parent = path.parent().ok_or_else(|| "action-result-publication-parent-missing".to_string())?;
    std::fs::create_dir_all(parent)
        .map_err(|error| format!("action-result-publication-create-dir:{}:{error}", parent.display()))?;
    let temp_path = unique_temp_path(path);
    let mut temp = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp_path)
        .map_err(|error| format!("action-result-publication-create-temp:{}:{error}", temp_path.display()))?;
    let publish_result = write_and_link_no_clobber(&mut temp, &temp_path, path, bytes);
    drop(temp);
    let remove_result = std::fs::remove_file(&temp_path);
    if let Err(error) = remove_result
        && error.kind() != ErrorKind::NotFound
    {
        return Err(format!("action-result-publication-remove-temp:{}:{error}", temp_path.display()));
    }
    publish_result
}

fn write_and_link_no_clobber(
    temp: &mut std::fs::File,
    temp_path: &Path,
    final_path: &Path,
    bytes: &[u8],
) -> Result<ActionResultPublicationStatus, String> {
    temp.write_all(bytes)
        .map_err(|error| format!("action-result-publication-write-temp:{}:{error}", temp_path.display()))?;
    temp.sync_all()
        .map_err(|error| format!("action-result-publication-sync-temp:{}:{error}", temp_path.display()))?;
    match std::fs::hard_link(temp_path, final_path) {
        Ok(()) => {
            sync_parent_directory(final_path)?;
            Ok(ActionResultPublicationStatus::Published)
        }
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {
            let existing = read_bounded_file(final_path, bytes.len().saturating_add(1))?;
            if existing == bytes {
                Ok(ActionResultPublicationStatus::Duplicate)
            } else {
                Err(format!("action-result-publication-no-clobber-conflict:{}", final_path.display()))
            }
        }
        Err(error) => {
            Err(format!("action-result-publication-link:{}->{}:{error}", temp_path.display(), final_path.display()))
        }
    }
}

fn sync_parent_directory(path: &Path) -> Result<(), String> {
    let parent = path.parent().ok_or_else(|| "action-result-publication-parent-missing".to_string())?;
    let directory = std::fs::File::open(parent)
        .map_err(|error| format!("action-result-publication-open-parent:{}:{error}", parent.display()))?;
    directory
        .sync_all()
        .map_err(|error| format!("action-result-publication-sync-parent:{}:{error}", parent.display()))
}

fn unique_temp_path(final_path: &Path) -> PathBuf {
    let sequence = TEMP_FILE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let process_id = std::process::id();
    let mut name = final_path.as_os_str().to_owned();
    name.push(format!(".tmp-{process_id}-{sequence}"));
    PathBuf::from(name)
}

fn read_bounded_file(path: &Path, byte_limit: usize) -> Result<Vec<u8>, String> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|error| format!("action-result-file-metadata:{}:{error}", path.display()))?;
    if !metadata.file_type().is_file() {
        return Err(format!("action-result-file-not-regular:{}", path.display()));
    }
    let size = usize::try_from(metadata.len()).map_err(|_| "action-result-file-size-overflow".to_string())?;
    if size > byte_limit {
        return Err(format!("action-result-file-too-large:{}", path.display()));
    }
    std::fs::read(path).map_err(|error| format!("action-result-file-read:{}:{error}", path.display()))
}

async fn read_bounded_response(response: reqwest::Response, byte_limit: usize) -> Result<Vec<u8>, String> {
    if let Some(content_length) = response.headers().get(CONTENT_LENGTH) {
        let content_length = content_length
            .to_str()
            .map_err(|_| "action-result-http-content-length-invalid".to_string())?
            .parse::<u64>()
            .map_err(|_| "action-result-http-content-length-invalid".to_string())?;
        let byte_limit_u64 =
            u64::try_from(byte_limit).map_err(|_| "action-result-http-byte-limit-overflow".to_string())?;
        if content_length > byte_limit_u64 {
            return Err("action-result-http-response-too-large".to_string());
        }
    }
    let mut bytes = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|error| format!("action-result-http-response-read:{error}"))?;
        let next_len = bytes
            .len()
            .checked_add(chunk.len())
            .ok_or_else(|| "action-result-http-response-size-overflow".to_string())?;
        if next_len > byte_limit {
            return Err("action-result-http-response-too-large".to_string());
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

fn validate_http_url(url: &Url) -> Result<(), String> {
    if !matches!(url.scheme(), "http" | "https") {
        return Err("action-result-http-url-scheme-unsupported".to_string());
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err("action-result-http-url-credentials-forbidden".to_string());
    }
    Ok(())
}

fn normalize_http_base_url(url: &Url) -> Url {
    let mut normalized = url.clone();
    normalized.set_query(None);
    normalized.set_fragment(None);
    let path = normalized.path().to_string();
    if !path.ends_with('/') {
        normalized.set_path(&format!("{path}/"));
    }
    normalized
}

fn http_write_succeeded(status: StatusCode) -> bool {
    status.is_success()
}

fn ref_digest<'a>(value: &'a str, prefix: &str) -> Result<&'a str, String> {
    let Some(digest) = value.strip_prefix(prefix) else {
        return Err("action-result-ref-prefix-invalid".to_string());
    };
    if digest.len() != crunch_action_result_core::BLAKE3_HEX_CHARS
        || !digest.chars().all(|character| character.is_ascii_digit() || ('a'..='f').contains(&character))
    {
        return Err("action-result-ref-digest-invalid".to_string());
    }
    Ok(digest)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::io::Read;
    use std::net::TcpListener;
    use std::net::TcpStream;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::atomic::AtomicBool;
    use std::thread::JoinHandle;

    use crunch_action_result_core::ACTION_RECEIPT_REF_PREFIX;
    use crunch_action_result_core::ActionResultOutput;
    use crunch_action_result_core::ActionResultRecordInput;
    use crunch_action_result_core::DetachedRecordSignature;
    use crunch_action_result_core::NETWORK_POLICY_REF_PREFIX;
    use crunch_action_result_core::OBJECT_REF_PREFIX;
    use crunch_action_result_core::PATH_INFO_REF_PREFIX;
    use crunch_action_result_core::PRODUCER_POLICY_REF_PREFIX;
    use crunch_action_result_core::PUBLICATION_POLICY_REF_PREFIX;
    use crunch_action_result_core::REFERENCE_SCAN_REF_PREFIX;
    use crunch_action_result_core::SANDBOX_POLICY_REF_PREFIX;
    use crunch_action_result_core::SIGNATURE_REF_PREFIX;
    use crunch_action_result_core::canonical_action_result;

    use super::*;

    fn typed_ref(prefix: &str, seed: &str) -> String {
        format!("{prefix}{}", blake3::hash(seed.as_bytes()).to_hex())
    }

    fn signed_record(seed: &str) -> SignedActionResultRecord {
        let record = canonical_action_result(ActionResultRecordInput {
            action_ref: typed_ref(ACTION_REF_PREFIX, "action"),
            outputs: vec![ActionResultOutput {
                name: "out".to_string(),
                object_ref: typed_ref(OBJECT_REF_PREFIX, seed),
                store_path: format!("/mantle/store/{seed}-out"),
                path_info_ref: typed_ref(PATH_INFO_REF_PREFIX, seed),
            }],
            action_receipt_ref: typed_ref(ACTION_RECEIPT_REF_PREFIX, seed),
            reference_scan_refs: vec![typed_ref(REFERENCE_SCAN_REF_PREFIX, seed)],
            sandbox_policy_ref: typed_ref(SANDBOX_POLICY_REF_PREFIX, "sandbox"),
            network_policy_ref: typed_ref(NETWORK_POLICY_REF_PREFIX, "network"),
            producer_identity: "builder-key-1".to_string(),
            producer_policy_ref: typed_ref(PRODUCER_POLICY_REF_PREFIX, "producer"),
            signature_refs: vec![typed_ref(SIGNATURE_REF_PREFIX, seed)],
            publication_policy_ref: typed_ref(PUBLICATION_POLICY_REF_PREFIX, "publication"),
            non_claims: vec![
                "ca-mapping-presence-is-not-output-trust".to_string(),
                "executor-correctness".to_string(),
                "index-presence-is-not-output-trust".to_string(),
            ],
        })
        .unwrap();
        SignedActionResultRecord {
            record,
            record_signatures: vec![DetachedRecordSignature {
                key_name: "builder-key-1".to_string(),
                signature: "signature".to_string(),
            }],
        }
    }

    #[tokio::test]
    async fn local_publish_is_atomic_no_clobber_and_duplicate_safe() {
        let temp = tempfile::tempdir().unwrap();
        let store = LocalActionResultStore::new(temp.path());
        let record = signed_record("same");

        let first = store.publish(&record).await.unwrap();
        let duplicate = store.publish(&record).await.unwrap();
        let lookup = store.lookup(&record.record.action_ref).await.unwrap();

        assert_eq!(first.record_status, ActionResultPublicationStatus::Published);
        assert_eq!(first.index_status, ActionResultPublicationStatus::Published);
        assert_eq!(duplicate.record_status, ActionResultPublicationStatus::Duplicate);
        assert_eq!(duplicate.index_status, ActionResultPublicationStatus::Duplicate);
        assert_eq!(lookup.records, vec![record]);
        assert_eq!(lookup.index.result_refs.len(), 1);
    }

    #[tokio::test]
    async fn gc_retains_metadata_only_while_outputs_are_independently_live() {
        let temp = tempfile::tempdir().unwrap();
        let store = LocalActionResultStore::new(temp.path());
        let record = signed_record("gc-policy");
        store.publish(&record).await.unwrap();
        let live_paths = BTreeSet::from([record.record.outputs[0].store_path.clone()]);

        let retained = local_action_result_gc_candidates(temp.path(), &live_paths).unwrap();
        let collectable = local_action_result_gc_candidates(temp.path(), &BTreeSet::new()).unwrap();

        assert!(retained.record_paths.is_empty());
        assert!(retained.index_marker_paths.is_empty());
        assert_eq!(collectable.record_paths.len(), 1);
        assert_eq!(collectable.index_marker_paths.len(), 1);
        assert!(!action_result_runtime_policy().gc.candidate_metadata_roots_outputs);
    }

    #[tokio::test]
    async fn duplicate_detached_signatures_are_rejected_before_publication() {
        let temp = tempfile::tempdir().unwrap();
        let store = LocalActionResultStore::new(temp.path());
        let mut record = signed_record("duplicate-signature");
        record.record_signatures.push(record.record_signatures[0].clone());

        let error = store.publish(&record).await.unwrap_err();

        assert_eq!(error, "action-result-record-signatures-not-canonical");
        assert!(!store.record_path(&record.record.result_ref).unwrap().exists());
    }

    #[tokio::test]
    async fn local_interrupted_publication_is_not_discoverable() {
        let temp = tempfile::tempdir().unwrap();
        let store = LocalActionResultStore::new(temp.path());
        let record = signed_record("interrupted");
        let record_path = store.record_path(&record.record.result_ref).unwrap();
        std::fs::create_dir_all(record_path.parent().unwrap()).unwrap();
        std::fs::write(&record_path, canonical_signed_record_bytes(&record).unwrap()).unwrap();

        let lookup = store.lookup(&record.record.action_ref).await.unwrap();

        assert!(lookup.index.result_refs.is_empty());
        assert!(lookup.records.is_empty());
        assert!(record_path.exists());
    }

    #[tokio::test]
    async fn local_dangling_or_poisoned_index_fails_closed() {
        let temp = tempfile::tempdir().unwrap();
        let store = LocalActionResultStore::new(temp.path());
        let record = signed_record("missing");
        let marker_path = store.marker_path(&record.record.action_ref, &record.record.result_ref).unwrap();
        std::fs::create_dir_all(marker_path.parent().unwrap()).unwrap();
        std::fs::write(&marker_path, format!("{}\n", record.record.result_ref)).unwrap();

        let error = store.lookup(&record.record.action_ref).await.unwrap_err();

        assert!(error.contains("action-result-local-record-unavailable"));
        assert!(!store.record_path(&record.record.result_ref).unwrap().exists());
    }

    #[tokio::test]
    async fn local_conflicting_existing_record_is_not_overwritten() {
        let temp = tempfile::tempdir().unwrap();
        let store = LocalActionResultStore::new(temp.path());
        let record = signed_record("conflict");
        let record_path = store.record_path(&record.record.result_ref).unwrap();
        std::fs::create_dir_all(record_path.parent().unwrap()).unwrap();
        std::fs::write(&record_path, b"poison").unwrap();

        let error = store.publish(&record).await.unwrap_err();
        let bytes = std::fs::read(&record_path).unwrap();

        assert!(error.contains("no-clobber-conflict"));
        assert_eq!(bytes, b"poison");
    }

    #[derive(Debug)]
    struct CountingRemoteStore {
        calls: Arc<Mutex<u32>>,
        records: usize,
    }

    #[async_trait]
    impl ActionResultStore for CountingRemoteStore {
        fn source_id(&self) -> &str {
            "counting-remote"
        }

        fn source_class(&self) -> &str {
            HTTP_ACTION_RESULT_SOURCE_CLASS
        }

        async fn lookup(&self, action_ref: &str) -> Result<ActionResultLookup, String> {
            let mut calls = self.calls.lock().unwrap();
            *calls = calls.saturating_add(1);
            Ok(ActionResultLookup {
                source_id: self.source_id().to_string(),
                source_class: self.source_class().to_string(),
                index: canonical_action_result_index(action_ref.to_string(), Vec::new()).unwrap(),
                records: vec![signed_record("bounded-source"); self.records],
                diagnostics: Vec::new(),
            })
        }

        async fn publish(&self, _record: &SignedActionResultRecord) -> Result<ActionResultPublicationReport, String> {
            Err("unused".to_string())
        }
    }

    #[tokio::test]
    async fn offline_discovery_never_opens_remote_sources() {
        let calls = Arc::new(Mutex::new(0));
        let mut stores = ActionResultStoreSet::new(true);
        stores.add_remote(Box::new(CountingRemoteStore {
            calls: calls.clone(),
            records: 0,
        }));

        let report = stores.discover(&typed_ref(ACTION_REF_PREFIX, "action")).await;

        assert_eq!(*calls.lock().unwrap(), 0);
        assert_eq!(report.remote_sources_opened, 0);
        assert!(report.diagnostics.contains(&"action-result-offline-remote-sources-skipped".to_string()));
    }

    #[tokio::test]
    async fn source_limit_bounds_empty_or_failing_remote_sources() {
        let calls = Arc::new(Mutex::new(0));
        let mut stores = ActionResultStoreSet::new(false);
        stores.max_sources = 1;
        stores.add_remote(Box::new(CountingRemoteStore {
            calls: calls.clone(),
            records: 0,
        }));
        stores.add_remote(Box::new(CountingRemoteStore {
            calls: calls.clone(),
            records: 0,
        }));

        let report = stores.discover(&typed_ref(ACTION_REF_PREFIX, "action")).await;

        assert_eq!(*calls.lock().unwrap(), 1);
        assert_eq!(report.remote_sources_opened, 1);
        assert!(report.diagnostics.contains(&"action-result-source-limit-exceeded".to_string()));
    }

    #[tokio::test]
    async fn aggregate_candidate_limit_rejects_the_offending_source_response() {
        let calls = Arc::new(Mutex::new(0));
        let mut stores = ActionResultStoreSet::new(false);
        stores.max_candidates = 1;
        stores.add_remote(Box::new(CountingRemoteStore {
            calls: calls.clone(),
            records: 2,
        }));

        let report = stores.discover(&typed_ref(ACTION_REF_PREFIX, "action")).await;

        assert_eq!(*calls.lock().unwrap(), 1);
        assert!(report.lookups.is_empty());
        assert_eq!(report.remote_sources_opened, 1);
        assert!(report.diagnostics[0].contains("action-result-total-candidate-limit-exceeded"));
    }

    #[test]
    fn http_urls_strip_query_fragment_and_preserve_cache_subpath() {
        let store = HttpActionResultStore::with_default_timeout(
            Url::parse("https://cache.example.test/sub/cache?token=secret#fragment").unwrap(),
        )
        .unwrap();
        let action_ref = typed_ref(ACTION_REF_PREFIX, "action");
        let url = store.index_url(&action_ref).unwrap();

        assert_eq!(url.query(), None);
        assert_eq!(url.fragment(), None);
        assert!(url.path().starts_with("/sub/cache/action-results/v1/indexes/"));
        assert!(!url.as_str().contains("secret"));
    }

    const HTTP_FIXTURE_POLL_MS: u64 = 5;
    const HTTP_FIXTURE_READ_TIMEOUT_MS: u64 = 250;
    const HTTP_TIMEOUT_TEST_CLIENT_MS: u64 = 25;
    const HTTP_TIMEOUT_TEST_SERVER_MS: u64 = 100;
    const HTTP_REQUEST_BYTES_MAX: usize = 2_097_152;
    const HTTP_READ_BUFFER_BYTES: usize = 4_096;

    #[derive(Debug, Clone)]
    struct HttpResource {
        bytes: Vec<u8>,
        etag: String,
    }

    #[derive(Debug, Default)]
    struct HttpFixtureState {
        resources: BTreeMap<String, HttpResource>,
        request_count: u32,
        fail_index_put: bool,
        delay_ms: u64,
    }

    struct HttpFixtureServer {
        base_url: Url,
        state: Arc<Mutex<HttpFixtureState>>,
        stop: Arc<AtomicBool>,
        thread: Option<JoinHandle<()>>,
    }

    impl HttpFixtureServer {
        fn spawn() -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            listener.set_nonblocking(true).unwrap();
            let address = listener.local_addr().unwrap();
            let state = Arc::new(Mutex::new(HttpFixtureState::default()));
            let stop = Arc::new(AtomicBool::new(false));
            let thread_state = state.clone();
            let thread_stop = stop.clone();
            let thread = std::thread::spawn(move || {
                while !thread_stop.load(Ordering::Relaxed) {
                    match listener.accept() {
                        Ok((mut stream, _)) => handle_http_fixture_request(&mut stream, &thread_state),
                        Err(error) if error.kind() == ErrorKind::WouldBlock => {
                            std::thread::sleep(Duration::from_millis(HTTP_FIXTURE_POLL_MS));
                        }
                        Err(_) => break,
                    }
                }
            });
            Self {
                base_url: Url::parse(&format!("http://{address}/")).unwrap(),
                state,
                stop,
                thread: Some(thread),
            }
        }

        fn store(&self) -> HttpActionResultStore {
            HttpActionResultStore::new(self.base_url.clone(), Duration::from_millis(HTTP_FIXTURE_READ_TIMEOUT_MS))
                .unwrap()
        }

        fn timeout_store(&self) -> HttpActionResultStore {
            HttpActionResultStore::new(self.base_url.clone(), Duration::from_millis(HTTP_TIMEOUT_TEST_CLIENT_MS))
                .unwrap()
        }
    }

    impl Drop for HttpFixtureServer {
        fn drop(&mut self) {
            self.stop.store(true, Ordering::Relaxed);
            if let Some(thread) = self.thread.take() {
                thread.join().unwrap();
            }
        }
    }

    fn handle_http_fixture_request(stream: &mut TcpStream, state: &Arc<Mutex<HttpFixtureState>>) {
        stream.set_read_timeout(Some(Duration::from_millis(HTTP_FIXTURE_READ_TIMEOUT_MS))).unwrap();
        let request = read_http_request(stream).unwrap();
        let delay_ms = state.lock().unwrap().delay_ms;
        if delay_ms > 0 {
            std::thread::sleep(Duration::from_millis(delay_ms));
        }
        let mut state = state.lock().unwrap();
        state.request_count = state.request_count.saturating_add(1);
        let response = plan_http_fixture_response(&request, &mut state);
        write_http_fixture_response(stream, response);
    }

    #[derive(Debug)]
    struct HttpFixtureRequest {
        method: String,
        path: String,
        headers: BTreeMap<String, String>,
        body: Vec<u8>,
    }

    fn read_http_request(stream: &mut TcpStream) -> Result<HttpFixtureRequest, String> {
        let mut bytes = Vec::new();
        let mut buffer = [0u8; HTTP_READ_BUFFER_BYTES];
        let header_end = loop {
            let count = stream.read(&mut buffer).map_err(|error| error.to_string())?;
            if count == 0 {
                return Err("fixture request ended before headers".to_string());
            }
            bytes.extend_from_slice(&buffer[..count]);
            if bytes.len() > HTTP_REQUEST_BYTES_MAX {
                return Err("fixture request exceeded bound".to_string());
            }
            if let Some(position) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
                break position.saturating_add(4);
            }
        };
        let header_text = String::from_utf8(bytes[..header_end].to_vec()).map_err(|error| error.to_string())?;
        let mut lines = header_text.split("\r\n");
        let request_line = lines.next().ok_or_else(|| "fixture request line missing".to_string())?;
        let mut request_parts = request_line.split_whitespace();
        let method = request_parts.next().ok_or_else(|| "fixture method missing".to_string())?.to_string();
        let path = request_parts.next().ok_or_else(|| "fixture path missing".to_string())?.to_string();
        let mut headers = BTreeMap::new();
        for line in lines.filter(|line| !line.is_empty()) {
            let (name, value) = line.split_once(':').ok_or_else(|| "fixture header invalid".to_string())?;
            headers.insert(name.to_ascii_lowercase(), value.trim().to_string());
        }
        let content_length = headers
            .get("content-length")
            .map(|value| value.parse::<usize>().map_err(|error| error.to_string()))
            .transpose()?
            .unwrap_or(0);
        while bytes.len().saturating_sub(header_end) < content_length {
            let count = stream.read(&mut buffer).map_err(|error| error.to_string())?;
            if count == 0 {
                return Err("fixture request body truncated".to_string());
            }
            bytes.extend_from_slice(&buffer[..count]);
            if bytes.len() > HTTP_REQUEST_BYTES_MAX {
                return Err("fixture request exceeded bound".to_string());
            }
        }
        let body_end = header_end.saturating_add(content_length);
        Ok(HttpFixtureRequest {
            method,
            path,
            headers,
            body: bytes[header_end..body_end].to_vec(),
        })
    }

    struct HttpFixtureResponse {
        status: &'static str,
        etag: Option<String>,
        body: Vec<u8>,
    }

    fn plan_http_fixture_response(request: &HttpFixtureRequest, state: &mut HttpFixtureState) -> HttpFixtureResponse {
        if request.method == "GET" {
            return match state.resources.get(&request.path) {
                Some(resource) => HttpFixtureResponse {
                    status: "200 OK",
                    etag: Some(resource.etag.clone()),
                    body: resource.bytes.clone(),
                },
                None => HttpFixtureResponse {
                    status: "404 Not Found",
                    etag: None,
                    body: Vec::new(),
                },
            };
        }
        if request.method != "PUT" {
            return HttpFixtureResponse {
                status: "405 Method Not Allowed",
                etag: None,
                body: Vec::new(),
            };
        }
        if state.fail_index_put && request.path.contains("/indexes/") {
            return HttpFixtureResponse {
                status: "500 Internal Server Error",
                etag: None,
                body: Vec::new(),
            };
        }
        if request.headers.get("if-none-match").map(String::as_str) == Some("*")
            && state.resources.contains_key(&request.path)
        {
            return HttpFixtureResponse {
                status: "412 Precondition Failed",
                etag: None,
                body: Vec::new(),
            };
        }
        if let Some(expected) = request.headers.get("if-match") {
            let matches = state.resources.get(&request.path).map(|resource| &resource.etag) == Some(expected);
            if !matches {
                return HttpFixtureResponse {
                    status: "412 Precondition Failed",
                    etag: None,
                    body: Vec::new(),
                };
            }
        }
        let etag = format!("\"v{}\"", state.request_count);
        state.resources.insert(request.path.clone(), HttpResource {
            bytes: request.body.clone(),
            etag: etag.clone(),
        });
        HttpFixtureResponse {
            status: "201 Created",
            etag: Some(etag),
            body: Vec::new(),
        }
    }

    fn write_http_fixture_response(stream: &mut TcpStream, response: HttpFixtureResponse) {
        let etag = response.etag.map(|value| format!("ETag: {value}\r\n")).unwrap_or_default();
        let headers = format!(
            "HTTP/1.1 {}\r\nContent-Length: {}\r\n{etag}Connection: close\r\n\r\n",
            response.status,
            response.body.len()
        );
        let _ = stream.write_all(headers.as_bytes());
        let _ = stream.write_all(&response.body);
        let _ = stream.flush();
    }

    #[tokio::test]
    async fn http_publication_is_record_first_discoverable_and_duplicate_safe() {
        let server = HttpFixtureServer::spawn();
        let store = server.store();
        let record = signed_record("http-happy");

        let first = store.publish(&record).await.unwrap();
        let duplicate = store.publish(&record).await.unwrap();
        let lookup = store.lookup(&record.record.action_ref).await.unwrap();

        assert_eq!(first.record_status, ActionResultPublicationStatus::Published);
        assert_eq!(first.index_status, ActionResultPublicationStatus::Published);
        assert_eq!(duplicate.record_status, ActionResultPublicationStatus::Duplicate);
        assert_eq!(duplicate.index_status, ActionResultPublicationStatus::Duplicate);
        assert_eq!(lookup.records, vec![record]);
    }

    #[tokio::test]
    async fn http_interrupted_index_publication_leaves_record_undiscoverable() {
        let server = HttpFixtureServer::spawn();
        server.state.lock().unwrap().fail_index_put = true;
        let store = server.store();
        let record = signed_record("http-interrupted");

        let error = store.publish(&record).await.unwrap_err();
        let lookup = store.lookup(&record.record.action_ref).await.unwrap();
        let record_path = store.record_url(&record.record.result_ref).unwrap().path().to_string();

        assert!(error.contains("index-publish-status"));
        assert!(lookup.records.is_empty());
        assert!(server.state.lock().unwrap().resources.contains_key(&record_path));
    }

    #[tokio::test]
    async fn http_corrupt_record_and_poisoned_index_fail_closed() {
        let server = HttpFixtureServer::spawn();
        let store = server.store();
        let record = signed_record("http-poison");
        store.publish(&record).await.unwrap();
        let record_path = store.record_url(&record.record.result_ref).unwrap().path().to_string();
        server.state.lock().unwrap().resources.get_mut(&record_path).unwrap().bytes = b"{corrupt".to_vec();

        let error = store.lookup(&record.record.action_ref).await.unwrap_err();

        assert!(error.contains("record-json-invalid"));
        assert_eq!(server.state.lock().unwrap().resources.get(&record_path).unwrap().bytes, b"{corrupt");
    }

    #[tokio::test]
    async fn http_timeout_rejects_source_without_fabricating_lookup() {
        let server = HttpFixtureServer::spawn();
        server.state.lock().unwrap().delay_ms = HTTP_TIMEOUT_TEST_SERVER_MS;
        let store = server.timeout_store();
        let action_ref = typed_ref(ACTION_REF_PREFIX, "action");

        let error = store.lookup(&action_ref).await.unwrap_err();

        assert!(error.contains("index-request"));
        assert!(server.state.lock().unwrap().resources.is_empty());
    }
}
