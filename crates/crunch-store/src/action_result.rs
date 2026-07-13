//! Durable local and bounded HTTP shells for shared action-result discovery.
//!
//! CAS services, this action-result store, and build executors remain separate
//! interfaces. Indexes are advisory: this module validates immutable bytes and
//! publication ordering but does not admit outputs for reuse.

use std::fmt;
use std::io::ErrorKind;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
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

pub const ACTION_RESULT_STORE_LAYOUT_VERSION: &str = "v1";
pub const LOCAL_ACTION_RESULT_SOURCE_ID: &str = "local-action-results";
pub const HTTP_ACTION_RESULT_SOURCE_CLASS: &str = "http";
pub const LOCAL_ACTION_RESULT_SOURCE_CLASS: &str = "local";
pub const DEFAULT_ACTION_RESULT_HTTP_TIMEOUT_MS: u64 = 5_000;
pub const MAX_ACTION_RESULT_HTTP_RETRIES: u32 = 4;
pub const MAX_ACTION_RESULT_TOTAL_CANDIDATES: usize = 256;

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
            if result_refs.len() >= MAX_ACTION_RESULT_TOTAL_CANDIDATES {
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
        Self::new(base_url, Duration::from_millis(DEFAULT_ACTION_RESULT_HTTP_TIMEOUT_MS))
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
        for _ in 0..MAX_ACTION_RESULT_HTTP_RETRIES {
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

#[derive(Debug, Default)]
pub struct ActionResultStoreSet {
    local: Vec<Box<dyn ActionResultStore>>,
    remote: Vec<Box<dyn ActionResultStore>>,
    offline: bool,
}

impl ActionResultStoreSet {
    pub fn new(offline: bool) -> Self {
        Self {
            local: Vec::new(),
            remote: Vec::new(),
            offline,
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
        discover_sources(&self.local, action_ref, false, &mut report).await;
        if self.offline {
            if !self.remote.is_empty() {
                report.diagnostics.push("action-result-offline-remote-sources-skipped".to_string());
            }
            return report;
        }
        discover_sources(&self.remote, action_ref, true, &mut report).await;
        report
    }
}

async fn discover_sources(
    stores: &[Box<dyn ActionResultStore>],
    action_ref: &str,
    is_remote: bool,
    report: &mut ActionResultDiscoveryReport,
) {
    for store in stores {
        if report.lookups.iter().map(|lookup| lookup.records.len()).sum::<usize>() >= MAX_ACTION_RESULT_TOTAL_CANDIDATES
        {
            report.diagnostics.push("action-result-total-candidate-limit-exceeded".to_string());
            return;
        }
        if is_remote {
            report.remote_sources_opened = report.remote_sources_opened.saturating_add(1);
        }
        match store.lookup(action_ref).await {
            Ok(lookup) => report.lookups.push(lookup),
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
    let bytes = canonical_signed_record_bytes(signed)?;
    if bytes.len() > crunch_action_result_core::MAX_ACTION_RESULT_RECORD_BYTES {
        return Err("action-result-signed-record-too-large".to_string());
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
        Ok(()) => Ok(ActionResultPublicationStatus::Published),
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
                records: Vec::new(),
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
        stores.add_remote(Box::new(CountingRemoteStore { calls: calls.clone() }));

        let report = stores.discover(&typed_ref(ACTION_REF_PREFIX, "action")).await;

        assert_eq!(*calls.lock().unwrap(), 0);
        assert_eq!(report.remote_sources_opened, 0);
        assert!(report.diagnostics.contains(&"action-result-offline-remote-sources-skipped".to_string()));
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
