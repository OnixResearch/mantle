//! Binary cache pull: import narinfo + NAR files into the local store.

use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use futures::TryStreamExt;
use nix_compat::narinfo::NarInfo;
use nix_compat::narinfo::VerifyingKey;
use nix_compat::nixbase32;
use nix_compat::store_path::StorePath;
use reqwest::StatusCode;
use reqwest::redirect::Policy;
use snix_castore::Node;
use snix_store::nar::ingest_nar_and_hash;
use snix_store::path_info::PathInfo;
use snix_store::pathinfoservice::PathInfoService;
use tokio::io::AsyncRead;
use tokio_util::io::StreamReader;
use url::Url;

use crate::Error;
use crate::export::export_castore_to_disk;
use crate::handle::StoreHandle;

/// Maximum number of requested/importable paths in one pull.
const MAX_PULL_PATHS: usize = 1_000_000;
/// Maximum directory entries to scan while enumerating local narinfo files.
const MAX_PULL_DIR_SCAN_ENTRIES: usize = 2_000_000;
/// HTTP connect timeout for remote cache pulls.
const HTTP_PULL_CONNECT_TIMEOUT_SECS: u64 = 30;
/// HTTP read timeout for remote cache pulls.
const HTTP_PULL_READ_TIMEOUT_SECS: u64 = 300;
/// Maximum redirect hops for HTTP pull requests.
const HTTP_PULL_MAX_REDIRECT_HOPS: usize = 10;
/// Default user-agent prefix for HTTP pull requests.
const HTTP_PULL_USER_AGENT_PREFIX: &str = "crunch";
/// Cache metadata file name.
const NIX_CACHE_INFO_FILE_NAME: &str = "nix-cache-info";

/// Options controlling pull behavior.
#[derive(Debug, Clone)]
pub struct PullOptions {
    /// Accept unsigned/unverified narinfos.
    pub trust_unsigned: bool,
    /// Trusted public keys for signature verification.
    pub trusted_public_keys: Vec<VerifyingKey>,
}

/// Source transport for a pull operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PullSource {
    Directory(PathBuf),
    Http(Url),
}

/// Record of a single successfully imported store path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PulledPath {
    pub store_path: String,
    pub nar_hash_hex: String,
    pub nar_size: u64,
}

/// Summary of a completed pull operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PullReport {
    pub imported_count: u32,
    pub skipped_already_present_count: u32,
    pub skipped_untrusted_count: u32,
    pub skipped_hash_mismatch_count: u32,
    pub skipped_missing_nar_count: u32,
    pub skipped_parse_error_count: u32,
    pub skipped_store_dir_mismatch_count: u32,
    pub total_nar_bytes: u64,
    pub paths: Vec<PulledPath>,
}

/// Import PathInfo entries from a flat Nix binary cache directory.
///
/// Scans `*.narinfo` files in `source`, verifies signatures, ingests
/// the referenced NAR into castore, persists PathInfo, and exports
/// the output to the local store directory on disk.
///
/// If `paths_filter` is `Some`, only narinfos whose store path matches
/// one of the given selectors (substring match) are imported.
pub async fn import_paths_from_cache_dir(
    handle: &StoreHandle,
    source: &Path,
    paths_filter: Option<&[String]>,
    options: &PullOptions,
) -> Result<PullReport, Error> {
    assert!(!source.as_os_str().is_empty(), "pull source must not be empty");

    if !source.exists() {
        return Err(Error::Store(format!("pull source directory does not exist: {}", source.display())));
    }

    let narinfo_files = scan_narinfo_files(source).await?;
    assert!(narinfo_files.len() <= MAX_PULL_PATHS, "pull scan exceeds limit of {MAX_PULL_PATHS}");

    let mut pull_result = PullReport {
        imported_count: 0,
        skipped_already_present_count: 0,
        skipped_untrusted_count: 0,
        skipped_hash_mismatch_count: 0,
        skipped_missing_nar_count: 0,
        skipped_parse_error_count: 0,
        skipped_store_dir_mismatch_count: 0,
        total_nar_bytes: 0,
        paths: Vec::with_capacity(narinfo_files.len().min(256)),
    };
    let context = DirectoryPullContext {
        handle,
        source,
        paths_filter,
        options,
    };

    for narinfo_path in &narinfo_files {
        pull_single_narinfo(&context, narinfo_path, &mut pull_result).await?;
    }

    Ok(pull_result)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HttpPullClientConfig {
    connect_timeout_secs: u64,
    read_timeout_secs: u64,
    max_redirect_hops: usize,
}

fn http_pull_client_config() -> HttpPullClientConfig {
    HttpPullClientConfig {
        connect_timeout_secs: HTTP_PULL_CONNECT_TIMEOUT_SECS,
        read_timeout_secs: HTTP_PULL_READ_TIMEOUT_SECS,
        max_redirect_hops: HTTP_PULL_MAX_REDIRECT_HOPS,
    }
}

fn http_pull_user_agent() -> String {
    format!("{HTTP_PULL_USER_AGENT_PREFIX}/{}", env!("CARGO_PKG_VERSION"))
}

fn build_http_pull_client() -> Result<reqwest::Client, Error> {
    let config = http_pull_client_config();
    reqwest::Client::builder()
        .user_agent(http_pull_user_agent())
        .connect_timeout(Duration::from_secs(config.connect_timeout_secs))
        .read_timeout(Duration::from_secs(config.read_timeout_secs))
        .redirect(Policy::none())
        .build()
        .map_err(|e| Error::Store(format!("building HTTP pull client: {e}")))
}

fn validate_http_cache_url(cache_url: &Url) -> Result<(), Error> {
    if !matches!(cache_url.scheme(), "http" | "https") {
        return Err(Error::Store(format!("HTTP pull requires http:// or https:// source, got {}", cache_url.scheme())));
    }
    if !cache_url.username().is_empty() || cache_url.password().is_some() {
        return Err(Error::Store(format!("HTTP pull source must not include URL credentials: {cache_url}")));
    }
    Ok(())
}

fn normalize_http_cache_base_url(cache_url: &Url) -> Url {
    let mut normalized = cache_url.clone();
    normalized.set_query(None);
    normalized.set_fragment(None);
    let current_path = normalized.path().to_string();
    if !current_path.ends_with('/') {
        normalized.set_path(&format!("{current_path}/"));
    }
    normalized
}

fn narinfo_url_for_digest(cache_url: &Url, digest: [u8; 20]) -> Result<Url, Error> {
    let narinfo_name = format!("{}.narinfo", nixbase32::encode(&digest));
    cache_url
        .join(&narinfo_name)
        .map_err(|e| Error::Store(format!("joining cache URL {cache_url} with {narinfo_name}: {e}")))
}

fn nix_cache_info_url(cache_url: &Url) -> Result<Url, Error> {
    cache_url
        .join(NIX_CACHE_INFO_FILE_NAME)
        .map_err(|e| Error::Store(format!("joining cache URL {cache_url} with {NIX_CACHE_INFO_FILE_NAME}: {e}")))
}

fn parse_store_dir_from_nix_cache_info(cache_info_text: &str) -> Option<String> {
    for raw_line in cache_info_text.lines() {
        let trimmed = raw_line.trim();
        let Some(store_dir_value) = trimmed.strip_prefix("StoreDir:") else {
            continue;
        };
        let parsed_store_dir = store_dir_value.trim();
        if !parsed_store_dir.is_empty() {
            return Some(parsed_store_dir.to_string());
        }
    }
    None
}

async fn fetch_remote_nix_cache_info(client: &reqwest::Client, cache_url: &Url) -> Result<Option<String>, String> {
    let cache_info_url = nix_cache_info_url(cache_url).map_err(|e| e.to_string())?;
    let response = client
        .get(cache_info_url.clone())
        .send()
        .await
        .map_err(|e| format!("requesting {cache_info_url}: {e}"))?;

    if response.status() == StatusCode::NOT_FOUND {
        return Ok(None);
    }
    if !response.status().is_success() {
        return Err(format!("requesting {cache_info_url}: HTTP {}", response.status()));
    }

    let cache_info_text = response.text().await.map_err(|e| format!("reading {cache_info_url}: {e}"))?;
    parse_store_dir_from_nix_cache_info(&cache_info_text)
        .ok_or_else(|| format!("{cache_info_url} does not contain a parseable StoreDir"))
        .map(Some)
}

async fn validate_remote_store_dir(
    client: &reqwest::Client,
    cache_url: &Url,
    local_store_dir: &str,
) -> Result<(), Error> {
    match fetch_remote_nix_cache_info(client, cache_url).await {
        Ok(Some(remote_store_dir)) => {
            if remote_store_dir != local_store_dir {
                return Err(Error::Store(format!(
                    "remote cache {cache_url} uses StoreDir {remote_store_dir}, expected {local_store_dir}"
                )));
            }
        }
        Ok(None) => {
            tracing::warn!(cache_url = %cache_url, "remote nix-cache-info missing; proceeding without StoreDir preflight");
        }
        Err(detail) => {
            tracing::warn!(cache_url = %cache_url, "remote nix-cache-info unavailable or invalid: {detail}");
        }
    }
    Ok(())
}

fn verify_narinfo_signatures(narinfo: &NarInfo<'_>, store_dir: &str, options: &PullOptions) -> bool {
    if options.trust_unsigned {
        return true;
    }
    let fingerprint = narinfo.fingerprint_with_store_dir(store_dir);
    narinfo
        .signatures
        .iter()
        .any(|signature| options.trusted_public_keys.iter().any(|key| key.verify(&fingerprint, signature)))
}

async fn fetch_http_narinfo_text(
    client: &reqwest::Client,
    cache_url: &Url,
    digest: [u8; 20],
) -> Result<Option<String>, String> {
    let narinfo_url = narinfo_url_for_digest(cache_url, digest).map_err(|e| e.to_string())?;
    let response =
        client.get(narinfo_url.clone()).send().await.map_err(|e| format!("requesting {narinfo_url}: {e}"))?;

    if matches!(response.status(), StatusCode::NOT_FOUND | StatusCode::FORBIDDEN) {
        return Ok(None);
    }
    if !response.status().is_success() {
        return Err(format!("requesting {narinfo_url}: HTTP {}", response.status()));
    }

    response.text().await.map(Some).map_err(|e| format!("reading {narinfo_url}: {e}"))
}

fn resolve_relative_nar_url(cache_url: &Url, narinfo_url_field: &str) -> Result<Url, String> {
    let has_scheme = Url::parse(narinfo_url_field).is_ok();
    if has_scheme || narinfo_url_field.starts_with("//") || narinfo_url_field.starts_with('/') {
        return Err(format!("narinfo URL must be relative, got {narinfo_url_field}"));
    }
    cache_url
        .join(narinfo_url_field)
        .map_err(|e| format!("joining cache URL {cache_url} with {narinfo_url_field}: {e}"))
}

pub async fn import_paths_from_http_cache(
    handle: &StoreHandle,
    cache_url: &Url,
    paths: &[StorePath<String>],
    options: &PullOptions,
) -> Result<PullReport, Error> {
    assert!(!cache_url.as_str().is_empty(), "cache URL must not be empty");
    assert!(paths.len() <= MAX_PULL_PATHS, "pull batch exceeds limit of {MAX_PULL_PATHS}");

    validate_http_cache_url(cache_url)?;

    let client = build_http_pull_client()?;
    let normalized_cache_url = normalize_http_cache_base_url(cache_url);
    validate_remote_store_dir(&client, &normalized_cache_url, handle.store_dir()).await?;

    let mut pull_result = PullReport {
        imported_count: 0,
        skipped_already_present_count: 0,
        skipped_untrusted_count: 0,
        skipped_hash_mismatch_count: 0,
        skipped_missing_nar_count: 0,
        skipped_parse_error_count: 0,
        skipped_store_dir_mismatch_count: 0,
        total_nar_bytes: 0,
        paths: Vec::with_capacity(paths.len().min(256)),
    };
    let context = HttpPullContext {
        handle,
        client: &client,
        cache_url: &normalized_cache_url,
        options,
    };

    for requested_path in paths {
        pull_single_http_path(&context, requested_path, &mut pull_result).await?;
    }

    Ok(pull_result)
}

struct HttpPullContext<'a> {
    handle: &'a StoreHandle,
    client: &'a reqwest::Client,
    cache_url: &'a Url,
    options: &'a PullOptions,
}

async fn pull_single_http_path(
    context: &HttpPullContext<'_>,
    requested_path: &StorePath<String>,
    result: &mut PullReport,
) -> Result<(), Error> {
    assert!(!requested_path.name().is_empty());
    assert!(!context.handle.store_dir().is_empty());
    let requested_digest: [u8; 20] = *requested_path.digest();
    let requested_path_text = requested_path.to_string();
    if context
        .handle
        .pathinfo_service()
        .get(requested_digest)
        .await
        .map_err(|e| Error::PathInfoService(format!("checking existing PathInfo: {e}")))?
        .is_some()
    {
        result.skipped_already_present_count = result.skipped_already_present_count.saturating_add(1);
        return Ok(());
    }
    let Some(narinfo_text) = fetch_http_narinfo_for_path(context, requested_digest, &requested_path_text, result).await
    else {
        return Ok(());
    };
    let Some(narinfo) = parse_http_narinfo(
        HttpNarinfoParseInput {
            text: &narinfo_text,
            requested_path_text: &requested_path_text,
            store_dir: context.handle.store_dir(),
            options: context.options,
        },
        result,
    ) else {
        return Ok(());
    };
    let Some(ingested) = ingest_http_nar(context, &narinfo, &requested_path_text, result).await? else {
        return Ok(());
    };
    if ingested.nar_sha256 != narinfo.nar_hash {
        result.skipped_hash_mismatch_count = result.skipped_hash_mismatch_count.saturating_add(1);
        return Ok(());
    }
    let path_info = PathInfo {
        store_path: requested_path.clone(),
        node: ingested.node,
        references: narinfo.references.iter().map(StorePath::to_owned).collect(),
        nar_sha256: ingested.nar_sha256,
        nar_size: ingested.nar_size_bytes,
        signatures: narinfo.signatures.iter().map(|signature| signature.to_owned()).collect(),
        deriver: narinfo.deriver.as_ref().map(StorePath::to_owned),
        ca: narinfo.ca.clone(),
    };
    persist_pulled_admission(
        context.handle,
        PulledPathAdmission {
            path_info,
            transport: PullTransport::Http,
        },
        result,
    )
    .await
}

async fn fetch_http_narinfo_for_path(
    context: &HttpPullContext<'_>,
    requested_digest: [u8; 20],
    requested_path_text: &str,
    result: &mut PullReport,
) -> Option<String> {
    assert!(!requested_path_text.is_empty());
    assert!(matches!(context.cache_url.scheme(), "http" | "https"));
    match fetch_http_narinfo_text(context.client, context.cache_url, requested_digest).await {
        Ok(Some(text)) => Some(text),
        Ok(None) => {
            result.skipped_missing_nar_count = result.skipped_missing_nar_count.saturating_add(1);
            None
        }
        Err(detail) => {
            tracing::warn!(requested_path = %requested_path_text, "failed to fetch HTTP narinfo: {detail}");
            result.skipped_parse_error_count = result.skipped_parse_error_count.saturating_add(1);
            None
        }
    }
}

struct HttpNarinfoParseInput<'a> {
    text: &'a str,
    requested_path_text: &'a str,
    store_dir: &'a str,
    options: &'a PullOptions,
}

fn parse_http_narinfo<'a>(input: HttpNarinfoParseInput<'a>, result: &mut PullReport) -> Option<NarInfo<'a>> {
    assert!(!input.requested_path_text.is_empty());
    assert!(!input.store_dir.is_empty());
    let narinfo = match NarInfo::parse_with_store_dir(input.text, input.store_dir) {
        Ok(narinfo) => narinfo,
        Err(error) => {
            if matches!(error, nix_compat::narinfo::Error::InvalidStorePath(_)) {
                result.skipped_store_dir_mismatch_count = result.skipped_store_dir_mismatch_count.saturating_add(1);
            } else {
                result.skipped_parse_error_count = result.skipped_parse_error_count.saturating_add(1);
            }
            tracing::warn!(requested_path = %input.requested_path_text, "failed to parse HTTP narinfo: {error}");
            return None;
        }
    };
    if narinfo.store_path.to_string() != input.requested_path_text {
        tracing::warn!(requested_path = %input.requested_path_text, narinfo_store_path = %narinfo.store_path, "HTTP narinfo store path mismatch");
        result.skipped_parse_error_count = result.skipped_parse_error_count.saturating_add(1);
        return None;
    }
    if !verify_narinfo_signatures(&narinfo, input.store_dir, input.options) {
        result.skipped_untrusted_count = result.skipped_untrusted_count.saturating_add(1);
        return None;
    }
    Some(narinfo)
}

struct IngestedNar {
    node: Node,
    nar_sha256: [u8; 32],
    nar_size_bytes: u64,
}

async fn ingest_http_nar(
    context: &HttpPullContext<'_>,
    narinfo: &NarInfo<'_>,
    requested_path_text: &str,
    result: &mut PullReport,
) -> Result<Option<IngestedNar>, Error> {
    assert!(!requested_path_text.is_empty());
    assert!(matches!(context.cache_url.scheme(), "http" | "https"));
    let nar_url = match resolve_relative_nar_url(context.cache_url, narinfo.url) {
        Ok(url) => url,
        Err(detail) => {
            tracing::warn!(requested_path = %requested_path_text, "rejecting HTTP narinfo URL: {detail}");
            result.skipped_parse_error_count = result.skipped_parse_error_count.saturating_add(1);
            return Ok(None);
        }
    };
    let nar_response = match context.client.get(nar_url.clone()).send().await {
        Ok(response) if response.status().is_success() => response,
        Ok(response) if response.status().is_redirection() => {
            tracing::warn!(requested_path = %requested_path_text, nar_url = %nar_url, "HTTP NAR redirect rejected with status {}", response.status());
            result.skipped_parse_error_count = result.skipped_parse_error_count.saturating_add(1);
            return Ok(None);
        }
        Ok(response) => {
            tracing::warn!(requested_path = %requested_path_text, nar_url = %nar_url, "HTTP NAR download failed with status {}", response.status());
            result.skipped_missing_nar_count = result.skipped_missing_nar_count.saturating_add(1);
            return Ok(None);
        }
        Err(error) => {
            tracing::warn!(requested_path = %requested_path_text, nar_url = %nar_url, "HTTP NAR request failed: {error}");
            result.skipped_parse_error_count = result.skipped_parse_error_count.saturating_add(1);
            return Ok(None);
        }
    };
    let response_stream = nar_response.bytes_stream().map_err(|error| std::io::Error::other(error.to_string()));
    let response_reader = tokio::io::BufReader::new(StreamReader::new(response_stream));
    let mut nar_reader: Box<dyn AsyncRead + Send + Unpin> = match narinfo.compression {
        None => Box::new(response_reader),
        Some("bzip2") => Box::new(async_compression::tokio::bufread::BzDecoder::new(response_reader)),
        Some("gzip") => Box::new(async_compression::tokio::bufread::GzipDecoder::new(response_reader)),
        Some("xz") => Box::new(async_compression::tokio::bufread::XzDecoder::new(response_reader)),
        Some("zstd") => Box::new(async_compression::tokio::bufread::ZstdDecoder::new(response_reader)),
        Some(compression) => {
            tracing::warn!(requested_path = %requested_path_text, "unsupported HTTP NAR compression: {compression}");
            result.skipped_parse_error_count = result.skipped_parse_error_count.saturating_add(1);
            return Ok(None);
        }
    };
    let ingestion = ingest_nar_and_hash(
        context.handle.blob_service(),
        context.handle.directory_service(),
        &mut nar_reader,
        &narinfo.ca,
    )
    .await;
    match ingestion {
        Ok((node, nar_sha256, nar_size_bytes)) => Ok(Some(IngestedNar {
            node,
            nar_sha256,
            nar_size_bytes,
        })),
        Err(error) => {
            tracing::warn!(requested_path = %requested_path_text, nar_url = %nar_url, "HTTP NAR ingestion failed: {error}");
            result.skipped_hash_mismatch_count = result.skipped_hash_mismatch_count.saturating_add(1);
            Ok(None)
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum PullTransport {
    Directory,
    Http,
}

impl PullTransport {
    fn label(self) -> &'static str {
        match self {
            Self::Directory => "directory",
            Self::Http => "HTTP",
        }
    }

    fn export_error(self, failure: PullExportFailure<'_>) -> Error {
        match self {
            Self::Directory => {
                Error::Export(format!("exporting imported path {}: {}", failure.abs_path, failure.error))
            }
            Self::Http => {
                Error::Export(format!("exporting imported HTTP path {}: {}", failure.abs_path, failure.error))
            }
        }
    }
}

struct PullExportFailure<'a> {
    abs_path: &'a str,
    error: &'a str,
}

struct PulledPathAdmission {
    path_info: PathInfo,
    transport: PullTransport,
}

async fn persist_pulled_admission(
    handle: &StoreHandle,
    admission: PulledPathAdmission,
    result: &mut PullReport,
) -> Result<(), Error> {
    assert!(!admission.path_info.store_path.name().is_empty());
    assert_eq!(admission.path_info.store_path.digest().len(), 20);
    let output_path = admission.path_info.store_path.clone();
    let node = admission.path_info.node.clone();
    handle
        .pathinfo_service()
        .put(admission.path_info.clone())
        .await
        .map_err(|error| Error::PathInfoService(format!("persisting imported PathInfo: {error}")))?;
    let abs_path = output_path.to_absolute_path_with_prefix(handle.output_dir_str());
    if !std::path::Path::new(&abs_path).exists() {
        match export_castore_to_disk(&node, &abs_path, &handle.blob_service(), &handle.directory_service()).await {
            Ok(()) => {}
            Err(error) if error.contains("Read-only file system") || error.contains("Permission denied") => {
                tracing::warn!(path = %abs_path, source = admission.transport.label(), "could not export imported path to disk (read-only store)");
            }
            Err(error) => {
                return Err(admission.transport.export_error(PullExportFailure {
                    abs_path: &abs_path,
                    error: &error,
                }));
            }
        }
    }
    result.imported_count = result.imported_count.saturating_add(1);
    result.total_nar_bytes = result.total_nar_bytes.saturating_add(admission.path_info.nar_size);
    result.paths.push(PulledPath {
        store_path: output_path.to_string(),
        nar_hash_hex: data_encoding::HEXLOWER.encode(&admission.path_info.nar_sha256),
        nar_size: admission.path_info.nar_size,
    });
    Ok(())
}

async fn scan_narinfo_files(source: &Path) -> Result<Vec<std::path::PathBuf>, Error> {
    let mut entries = tokio::fs::read_dir(source)
        .await
        .map_err(|e| Error::Store(format!("reading pull source {}: {e}", source.display())))?;

    let mut narinfo_files = Vec::with_capacity(256);
    for _ in 0..MAX_PULL_DIR_SCAN_ENTRIES {
        let Some(entry) = entries.next_entry().await.map_err(|e| Error::Store(format!("scanning pull source: {e}")))?
        else {
            break;
        };

        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "narinfo") {
            narinfo_files.push(path);
        }
    }
    Ok(narinfo_files)
}

struct DirectoryPullContext<'a> {
    handle: &'a StoreHandle,
    source: &'a Path,
    paths_filter: Option<&'a [String]>,
    options: &'a PullOptions,
}

async fn pull_single_narinfo(
    context: &DirectoryPullContext<'_>,
    narinfo_path: &Path,
    result: &mut PullReport,
) -> Result<(), Error> {
    assert!(!narinfo_path.as_os_str().is_empty());
    assert!(!context.handle.store_dir().is_empty());
    let narinfo_text = match tokio::fs::read_to_string(narinfo_path).await {
        Ok(text) => text,
        Err(error) => {
            tracing::warn!(path = %narinfo_path.display(), "failed to read narinfo: {error}");
            result.skipped_parse_error_count = result.skipped_parse_error_count.saturating_add(1);
            return Ok(());
        }
    };
    let Some(narinfo) = parse_directory_narinfo(
        DirectoryNarinfoParseInput {
            text: &narinfo_text,
            path: narinfo_path,
            store_dir: context.handle.store_dir(),
        },
        result,
    ) else {
        return Ok(());
    };
    let Some(nar_file_path) = admitted_directory_nar_path(context, &narinfo, result).await? else {
        return Ok(());
    };
    let Some(ingested) = ingest_directory_nar(context, &narinfo, &nar_file_path, result).await? else {
        return Ok(());
    };
    if ingested.nar_sha256 != narinfo.nar_hash {
        result.skipped_hash_mismatch_count = result.skipped_hash_mismatch_count.saturating_add(1);
        return Ok(());
    }
    let path_info = PathInfo {
        store_path: narinfo.store_path.to_owned(),
        node: ingested.node,
        references: narinfo.references.iter().map(StorePath::to_owned).collect(),
        nar_sha256: ingested.nar_sha256,
        nar_size: ingested.nar_size_bytes,
        signatures: narinfo.signatures.iter().map(|signature| signature.to_owned()).collect(),
        deriver: narinfo.deriver.map(|deriver| deriver.to_owned()),
        ca: narinfo.ca.clone(),
    };
    persist_pulled_admission(
        context.handle,
        PulledPathAdmission {
            path_info,
            transport: PullTransport::Directory,
        },
        result,
    )
    .await
}

struct DirectoryNarinfoParseInput<'a> {
    text: &'a str,
    path: &'a Path,
    store_dir: &'a str,
}

fn parse_directory_narinfo<'a>(input: DirectoryNarinfoParseInput<'a>, result: &mut PullReport) -> Option<NarInfo<'a>> {
    assert!(!input.path.as_os_str().is_empty());
    assert!(!input.store_dir.is_empty());
    match NarInfo::parse_with_store_dir(input.text, input.store_dir) {
        Ok(narinfo) => Some(narinfo),
        Err(error) => {
            if input.store_dir != nix_compat::store_path::STORE_DIR && NarInfo::parse(input.text).is_ok() {
                result.skipped_store_dir_mismatch_count = result.skipped_store_dir_mismatch_count.saturating_add(1);
                return None;
            }
            tracing::warn!(path = %input.path.display(), "failed to parse narinfo: {error}");
            if matches!(error, nix_compat::narinfo::Error::InvalidStorePath(_)) {
                result.skipped_store_dir_mismatch_count = result.skipped_store_dir_mismatch_count.saturating_add(1);
            } else {
                result.skipped_parse_error_count = result.skipped_parse_error_count.saturating_add(1);
            }
            None
        }
    }
}

async fn admitted_directory_nar_path(
    context: &DirectoryPullContext<'_>,
    narinfo: &NarInfo<'_>,
    result: &mut PullReport,
) -> Result<Option<PathBuf>, Error> {
    let store_path_text = narinfo.store_path.to_string();
    assert!(!store_path_text.is_empty());
    assert!(!context.source.as_os_str().is_empty());
    if let Some(filter) = context.paths_filter
        && !filter.iter().any(|selector| store_path_text.contains(selector.as_str()))
    {
        return Ok(None);
    }
    if context
        .handle
        .pathinfo_service()
        .get(*narinfo.store_path.digest())
        .await
        .map_err(|error| Error::PathInfoService(format!("checking existing PathInfo: {error}")))?
        .is_some()
    {
        result.skipped_already_present_count = result.skipped_already_present_count.saturating_add(1);
        return Ok(None);
    }
    if !verify_narinfo_signatures(narinfo, context.handle.store_dir(), context.options) {
        result.skipped_untrusted_count = result.skipped_untrusted_count.saturating_add(1);
        return Ok(None);
    }
    let nar_file_path = context.source.join(narinfo.url);
    if !nar_file_path.exists() {
        result.skipped_missing_nar_count = result.skipped_missing_nar_count.saturating_add(1);
        return Ok(None);
    }
    Ok(Some(nar_file_path))
}

async fn ingest_directory_nar(
    context: &DirectoryPullContext<'_>,
    narinfo: &NarInfo<'_>,
    nar_file_path: &Path,
    result: &mut PullReport,
) -> Result<Option<IngestedNar>, Error> {
    assert!(!nar_file_path.as_os_str().is_empty());
    assert!(!narinfo.store_path.name().is_empty());
    let nar_file = tokio::fs::File::open(nar_file_path)
        .await
        .map_err(|error| Error::Store(format!("opening NAR {}: {error}", nar_file_path.display())))?;
    let mut nar_reader = tokio::io::BufReader::new(nar_file);
    let ingestion = ingest_nar_and_hash(
        context.handle.blob_service(),
        context.handle.directory_service(),
        &mut nar_reader,
        &narinfo.ca,
    )
    .await;
    match ingestion {
        Ok((node, nar_sha256, nar_size_bytes)) => Ok(Some(IngestedNar {
            node,
            nar_sha256,
            nar_size_bytes,
        })),
        Err(error) => {
            tracing::warn!(path = %nar_file_path.display(), "NAR ingestion failed (corrupt data?): {error}");
            result.skipped_hash_mismatch_count = result.skipped_hash_mismatch_count.saturating_add(1);
            Ok(None)
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::cloned_ref_to_slice_refs,
    clippy::type_complexity,
    clippy::useless_conversion
)]
mod tests {
    use std::collections::BTreeMap;
    use std::io::Read;
    use std::io::Write;
    use std::net::TcpListener;
    use std::net::TcpStream;
    use std::num::NonZeroUsize;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering;
    use std::thread::JoinHandle;
    use std::thread::{self};
    use std::time::Duration;

    use async_trait::async_trait;
    use futures::stream::BoxStream;
    use nix_compat::nixbase32;
    use snix_castore::Node;
    use snix_castore::blobservice::BlobService;
    use snix_castore::blobservice::MemoryBlobService;
    use snix_castore::directoryservice::DirectoryService;
    use snix_castore::directoryservice::RedbDirectoryService;
    use snix_castore::directoryservice::RedbDirectoryServiceConfig;
    use snix_store::pathinfoservice::LruPathInfoService;
    use tokio::io::AsyncWriteExt;
    use xz2::write::XzEncoder;

    use super::*;
    use crate::StoreConfig;
    use crate::StoreFallbackMode;
    use crate::StoreHandleServices;
    use crate::push::PushOptions;
    use crate::push::export_paths_to_cache_dir;

    /// Build a minimal signed PathInfo with a single blob in the store.
    async fn make_signed_pathinfo(handle: &StoreHandle, name: &str, content: &[u8]) -> PathInfo {
        make_signed_pathinfo_with_references(handle, name, content, vec![]).await
    }

    async fn make_signed_pathinfo_with_references(
        handle: &StoreHandle,
        name: &str,
        content: &[u8],
        references: Vec<StorePath<String>>,
    ) -> PathInfo {
        let mut writer = handle.blob_service().open_write().await;
        writer.write_all(content).await.unwrap();
        let blob_digest = writer.close().await.unwrap();

        let node = Node::File {
            digest: blob_digest,
            size: content.len() as u64,
            executable: false,
        };
        let nar_buf = render_nar_bytes(handle, &node).await;

        let nar_sha256: [u8; 32] = {
            use sha2::Digest;
            sha2::Sha256::digest(&nar_buf).into()
        };

        let mut digest = [0u8; 20];
        for (i, b) in name.as_bytes().iter().enumerate().take(20) {
            digest[i] = *b;
        }
        let store_path = StorePath::from_name_and_digest_fixed(name, digest).unwrap();

        let mut pi = PathInfo {
            store_path,
            node,
            references,
            nar_sha256,
            nar_size: nar_buf.len() as u64,
            signatures: vec![],
            deriver: None,
            ca: None,
        };

        sign_pathinfo(&mut pi, handle.store_dir());
        pi
    }

    fn test_signing_key() -> nix_compat::narinfo::SigningKey<ed25519_dalek::SigningKey> {
        let secret = ed25519_dalek::SigningKey::from_bytes(&[42u8; 32]);
        nix_compat::narinfo::SigningKey::new("test-key-1".to_string(), secret)
    }

    fn test_verifying_key() -> VerifyingKey {
        let secret = ed25519_dalek::SigningKey::from_bytes(&[42u8; 32]);
        let public = ed25519_dalek::VerifyingKey::from(&secret);
        VerifyingKey::new("test-key-1".to_string(), public)
    }

    fn sign_pathinfo(pi: &mut PathInfo, store_dir: &str) {
        let signing_key = test_signing_key();
        let store_path_ref: nix_compat::store_path::StorePathRef = pi.store_path.as_ref();
        let refs: Vec<nix_compat::store_path::StorePathRef> = pi.references.iter().map(StorePath::as_ref).collect();
        let fp = nix_compat::narinfo::fingerprint_with_store_dir(
            &store_path_ref,
            &pi.nar_sha256,
            pi.nar_size,
            refs.iter(),
            store_dir,
        );
        let sig = signing_key.sign(fp.as_bytes()).to_owned();
        pi.signatures.push(sig);
    }

    async fn render_nar_bytes(handle: &StoreHandle, node: &Node) -> Vec<u8> {
        use snix_store::nar::write_nar;
        use tokio::io::AsyncReadExt;

        let (mut reader, writer) = tokio::io::duplex(64 * 1024);
        let node = node.clone();
        let bs = handle.blob_service();
        let ds = handle.directory_service();
        let write_task = tokio::spawn(async move { write_nar(writer, &node, bs, ds).await });
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await.unwrap();
        write_task.await.unwrap().unwrap();
        buf
    }

    async fn open_test_store_with_store_dir(dir: &Path, store_dir: &str) -> StoreHandle {
        let state_dir = dir.join("state");
        let output_dir = dir.join("output");
        std::fs::create_dir_all(&state_dir).unwrap();
        std::fs::create_dir_all(&output_dir).unwrap();
        StoreHandle::open(StoreConfig {
            state_dir,
            output_dir,
            remote_cache_urls: Vec::new(),
            fallback_mode: StoreFallbackMode::Practical,
            store_dir: store_dir.to_string(),
            base_state_dirs: Vec::new(),
        })
        .await
        .unwrap()
    }

    async fn open_test_store(dir: &Path) -> StoreHandle {
        open_test_store_with_store_dir(dir, "/nix/store").await
    }

    fn default_test_pathinfo_service() -> Arc<dyn PathInfoService> {
        Arc::new(LruPathInfoService::with_capacity("pull-test-pathinfo".to_string(), NonZeroUsize::new(32).unwrap()))
            as Arc<dyn PathInfoService>
    }

    fn test_service_backed_store(
        state_dir: &Path,
        output_dir_str: String,
        store_dir: &str,
        pathinfo_service: Arc<dyn PathInfoService>,
    ) -> StoreHandle {
        std::fs::create_dir_all(state_dir).unwrap();
        let blob_service = Arc::new(MemoryBlobService::default()) as Arc<dyn BlobService>;
        let directory_service = Arc::new(
            RedbDirectoryService::new_temporary(
                format!("pull-test-{}", store_dir.replace('/', "_")),
                RedbDirectoryServiceConfig::default(),
            )
            .unwrap(),
        ) as Arc<dyn DirectoryService>;
        StoreHandle::from_services_with_store_dir(
            StoreHandleServices {
                blob_service,
                directory_service,
                pathinfo_service,
                remote_pathinfo: None,
                state_dir: state_dir.to_path_buf(),
                output_dir_str,
                publishers: Vec::new(),
            },
            store_dir.to_string(),
        )
    }

    struct PutFailingPathInfoService {
        inner: Arc<dyn PathInfoService>,
        detail: String,
    }

    #[async_trait]
    impl PathInfoService for PutFailingPathInfoService {
        async fn get(&self, digest: [u8; 20]) -> Result<Option<PathInfo>, snix_store::pathinfoservice::Error> {
            self.inner.get(digest).await
        }

        async fn put(&self, _path_info: PathInfo) -> Result<PathInfo, snix_store::pathinfoservice::Error> {
            Err(std::io::Error::other(self.detail.clone()).into())
        }

        fn list(&self) -> BoxStream<'static, Result<PathInfo, snix_store::pathinfoservice::Error>> {
            self.inner.list()
        }
    }

    fn default_pull_options() -> PullOptions {
        PullOptions {
            trust_unsigned: false,
            trusted_public_keys: vec![test_verifying_key()],
        }
    }

    #[derive(Debug, Clone, Default, PartialEq, Eq)]
    struct HttpRequestStats {
        nix_cache_info_requests: u32,
        narinfo_requests: u32,
        nar_requests: u32,
        request_paths: Vec<String>,
        last_user_agent: Option<String>,
    }

    #[derive(Debug, Clone)]
    enum HttpResponse {
        Fixed {
            status_line: String,
            headers: Vec<(String, String)>,
            body: Vec<u8>,
        },
        DropConnection,
    }

    impl HttpResponse {
        fn ok_text(body: String) -> Self {
            Self::Fixed {
                status_line: "HTTP/1.1 200 OK".to_string(),
                headers: vec![("Content-Type".to_string(), "text/plain".to_string())],
                body: body.into_bytes(),
            }
        }

        fn ok_bytes(body: Vec<u8>) -> Self {
            Self::Fixed {
                status_line: "HTTP/1.1 200 OK".to_string(),
                headers: vec![("Content-Type".to_string(), "application/octet-stream".to_string())],
                body,
            }
        }

        fn not_found() -> Self {
            Self::Fixed {
                status_line: "HTTP/1.1 404 Not Found".to_string(),
                headers: vec![("Content-Type".to_string(), "text/plain".to_string())],
                body: b"not found".to_vec(),
            }
        }

        fn redirect(location: &str) -> Self {
            Self::Fixed {
                status_line: "HTTP/1.1 302 Found".to_string(),
                headers: vec![("Location".to_string(), location.to_string())],
                body: Vec::new(),
            }
        }
    }

    struct HttpTestServer {
        base_url: Url,
        stats: Arc<Mutex<HttpRequestStats>>,
        stop: Arc<AtomicBool>,
        handle: Option<JoinHandle<()>>,
    }

    impl HttpTestServer {
        fn spawn(routes: BTreeMap<String, HttpResponse>) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
            listener.set_nonblocking(true).expect("nonblocking listener");
            let addr = listener.local_addr().expect("listener addr");
            let stats = Arc::new(Mutex::new(HttpRequestStats::default()));
            let stop = Arc::new(AtomicBool::new(false));
            let stats_ref = Arc::clone(&stats);
            let stop_ref = Arc::clone(&stop);
            let routes_ref = Arc::new(routes);
            let routes_thread = Arc::clone(&routes_ref);
            let handle = thread::spawn(move || {
                while !stop_ref.load(Ordering::SeqCst) {
                    match listener.accept() {
                        Ok((mut stream, _peer)) => {
                            serve_http_request(&mut stream, &routes_thread, &stats_ref);
                        }
                        Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(10));
                        }
                        Err(err) => panic!("accept failed: {err}"),
                    }
                }
            });

            Self {
                base_url: format!("http://{addr}/").parse().expect("base url"),
                stats,
                stop,
                handle: Some(handle),
            }
        }

        fn stats(&self) -> HttpRequestStats {
            self.stats.lock().expect("stats lock").clone()
        }
    }

    impl Drop for HttpTestServer {
        fn drop(&mut self) {
            self.stop.store(true, Ordering::SeqCst);
            let host = self.base_url.host_str().expect("host");
            let port = self.base_url.port_or_known_default().expect("port");
            let _ = TcpStream::connect((host, port));
            if let Some(handle) = self.handle.take() {
                handle.join().expect("server join");
            }
        }
    }

    fn serve_http_request(
        stream: &mut TcpStream,
        routes: &BTreeMap<String, HttpResponse>,
        stats: &Arc<Mutex<HttpRequestStats>>,
    ) {
        let mut request_bytes = [0u8; 4096];
        let bytes_read = stream.read(&mut request_bytes).expect("read request");
        let request = String::from_utf8_lossy(&request_bytes[..bytes_read]);
        let path = request.lines().next().and_then(|line| line.split_whitespace().nth(1)).unwrap_or("/").to_string();
        let user_agent = request.lines().find_map(|line| {
            let lower = line.to_ascii_lowercase();
            if lower.starts_with("user-agent:") {
                line.split_once(':').map(|(_, value)| value.trim().to_string())
            } else {
                None
            }
        });

        {
            let mut guard = stats.lock().expect("stats lock");
            guard.request_paths.push(path.clone());
            guard.last_user_agent = user_agent;
            if path == format!("/{NIX_CACHE_INFO_FILE_NAME}") {
                guard.nix_cache_info_requests = guard.nix_cache_info_requests.saturating_add(1);
            } else if path.ends_with(".narinfo") {
                guard.narinfo_requests = guard.narinfo_requests.saturating_add(1);
            } else if path.contains("/nar/") {
                guard.nar_requests = guard.nar_requests.saturating_add(1);
            }
        }

        let response = routes.get(&path).cloned().unwrap_or_else(HttpResponse::not_found);
        match response {
            HttpResponse::DropConnection => {}
            HttpResponse::Fixed {
                status_line,
                headers,
                body,
            } => {
                let mut response_bytes =
                    format!("{status_line}\r\nContent-Length: {}\r\nConnection: close\r\n", body.len()).into_bytes();
                for (name, value) in headers {
                    response_bytes.extend_from_slice(format!("{name}: {value}\r\n").as_bytes());
                }
                response_bytes.extend_from_slice(b"\r\n");
                response_bytes.extend_from_slice(&body);
                stream.write_all(&response_bytes).expect("write response");
                stream.flush().expect("flush response");
            }
        }
    }

    fn cache_routes(cache_dir: &Path) -> BTreeMap<String, HttpResponse> {
        let mut routes = BTreeMap::new();
        routes.insert(
            format!("/{NIX_CACHE_INFO_FILE_NAME}"),
            HttpResponse::ok_text(std::fs::read_to_string(cache_dir.join(NIX_CACHE_INFO_FILE_NAME)).unwrap()),
        );
        for entry in std::fs::read_dir(cache_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "narinfo") {
                let route_path = format!("/{}", entry.file_name().to_string_lossy());
                routes.insert(route_path, HttpResponse::ok_text(std::fs::read_to_string(path).unwrap()));
            }
        }
        let nar_dir = cache_dir.join("nar");
        for entry in std::fs::read_dir(nar_dir).unwrap() {
            let entry = entry.unwrap();
            let route_path = format!("/nar/{}", entry.file_name().to_string_lossy());
            routes.insert(route_path, HttpResponse::ok_bytes(std::fs::read(entry.path()).unwrap()));
        }
        routes
    }

    fn narinfo_file_name(store_path: &StorePath<String>) -> String {
        format!("{}.narinfo", nixbase32::encode(store_path.digest()))
    }

    fn read_narinfo_text(cache_dir: &Path, store_path: &StorePath<String>) -> String {
        std::fs::read_to_string(cache_dir.join(narinfo_file_name(store_path))).unwrap()
    }

    fn replace_narinfo_field(narinfo_text: &str, field: &str, new_value: &str) -> String {
        narinfo_text
            .lines()
            .map(|line| {
                if line.starts_with(&format!("{field}:")) {
                    format!("{field}: {new_value}")
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
    }

    fn compress_xz(bytes: &[u8]) -> Vec<u8> {
        const XZ_COMPRESSION_LEVEL: u32 = 6;
        let mut encoder = XzEncoder::new(Vec::new(), XZ_COMPRESSION_LEVEL);
        encoder.write_all(bytes).unwrap();
        encoder.finish().unwrap()
    }

    fn compress_gzip(bytes: &[u8]) -> Vec<u8> {
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(bytes).unwrap();
        encoder.finish().unwrap()
    }

    fn compress_bzip2(bytes: &[u8]) -> Vec<u8> {
        let mut encoder = bzip2::write::BzEncoder::new(Vec::new(), bzip2::Compression::default());
        encoder.write_all(bytes).unwrap();
        encoder.finish().unwrap()
    }

    fn compress_zstd(bytes: &[u8]) -> Vec<u8> {
        const ZSTD_COMPRESSION_LEVEL: i32 = 0;
        zstd::stream::encode_all(bytes, ZSTD_COMPRESSION_LEVEL).unwrap()
    }

    fn nix_base32_sha256(bytes: &[u8]) -> String {
        use sha2::Digest;
        let digest: [u8; 32] = sha2::Sha256::digest(bytes).into();
        nixbase32::encode(&digest)
    }

    // --- Push → Pull round-trip ---

    #[tokio::test]
    async fn pull_single_signed_path_round_trip() {
        let tmp = tempfile::tempdir().unwrap();

        // Push side: build and push a path.
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let pi = make_signed_pathinfo(&push_store, "hello", b"hello world").await;
        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&push_store, &[pi.clone()], &cache_dir, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();

        // Pull side: import into a fresh store.
        let pull_store = open_test_store(&tmp.path().join("pull")).await;
        let report = import_paths_from_cache_dir(&pull_store, &cache_dir, None, &default_pull_options()).await.unwrap();

        assert_eq!(report.imported_count, 1);
        assert_eq!(report.skipped_already_present_count, 0);
        assert_eq!(report.skipped_untrusted_count, 0);
        assert_eq!(report.skipped_hash_mismatch_count, 0);
        assert_eq!(report.skipped_missing_nar_count, 0);
        assert_eq!(report.skipped_parse_error_count, 0);
        assert_eq!(report.paths.len(), 1);
        assert_eq!(report.paths[0].store_path, pi.store_path.to_string());
        assert_eq!(report.total_nar_bytes, pi.nar_size);

        // Verify PathInfo was persisted.
        let imported_pi = pull_store
            .pathinfo_service()
            .get((*pi.store_path.digest()).into())
            .await
            .unwrap()
            .expect("imported PathInfo must exist");
        assert_eq!(imported_pi.nar_sha256, pi.nar_sha256);
        assert_eq!(imported_pi.nar_size, pi.nar_size);
        assert_eq!(imported_pi.store_path, pi.store_path);
    }

    #[tokio::test]
    async fn pull_skips_already_present() {
        let tmp = tempfile::tempdir().unwrap();
        let store = open_test_store(&tmp.path().join("store")).await;
        let pi = make_signed_pathinfo(&store, "hello", b"hello world").await;

        // Push and pull once.
        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&store, &[pi.clone()], &cache_dir, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();

        let pull_store = open_test_store(&tmp.path().join("pull")).await;
        import_paths_from_cache_dir(&pull_store, &cache_dir, None, &default_pull_options()).await.unwrap();

        // Pull again — should skip.
        let report = import_paths_from_cache_dir(&pull_store, &cache_dir, None, &default_pull_options()).await.unwrap();

        assert_eq!(report.imported_count, 0);
        assert_eq!(report.skipped_already_present_count, 1);
    }

    #[tokio::test]
    async fn pull_rejects_untrusted_signature() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let pi = make_signed_pathinfo(&push_store, "hello", b"hello world").await;

        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&push_store, &[pi], &cache_dir, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();

        // Pull with a different trusted key — should reject.
        let other_secret = ed25519_dalek::SigningKey::from_bytes(&[99u8; 32]);
        let other_public = ed25519_dalek::VerifyingKey::from(&other_secret);
        let other_key = VerifyingKey::new("other-key-1".to_string(), other_public);

        let pull_store = open_test_store(&tmp.path().join("pull")).await;
        let report = import_paths_from_cache_dir(&pull_store, &cache_dir, None, &PullOptions {
            trust_unsigned: false,
            trusted_public_keys: vec![other_key],
        })
        .await
        .unwrap();

        assert_eq!(report.imported_count, 0);
        assert_eq!(report.skipped_untrusted_count, 1);
    }

    #[tokio::test]
    async fn pull_accepts_unsigned_when_trust_unsigned() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let mut pi = make_signed_pathinfo(&push_store, "hello", b"hello world").await;
        pi.signatures.clear(); // make unsigned

        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&push_store, &[pi], &cache_dir, &PushOptions { trust_unsigned: true })
            .await
            .unwrap();

        let pull_store = open_test_store(&tmp.path().join("pull")).await;
        let report = import_paths_from_cache_dir(&pull_store, &cache_dir, None, &PullOptions {
            trust_unsigned: true,
            trusted_public_keys: vec![],
        })
        .await
        .unwrap();

        assert_eq!(report.imported_count, 1);
    }

    #[tokio::test]
    async fn pull_detects_nar_hash_mismatch() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let pi = make_signed_pathinfo(&push_store, "hello", b"hello world").await;

        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&push_store, &[pi.clone()], &cache_dir, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();

        // Corrupt the NAR file.
        let nar_hash_b32 = nixbase32::encode(&pi.nar_sha256);
        let nar_path = cache_dir.join("nar").join(format!("{nar_hash_b32}.nar"));
        std::fs::write(&nar_path, b"corrupted data that is not a valid nar").unwrap();

        let pull_store = open_test_store(&tmp.path().join("pull")).await;
        let report = import_paths_from_cache_dir(&pull_store, &cache_dir, None, &default_pull_options()).await.unwrap();

        // NAR ingestion will fail because the corrupted data isn't valid NAR format.
        // That manifests as a hash mismatch or an ingestion error. Either way, not imported.
        assert_eq!(report.imported_count, 0);
    }

    #[tokio::test]
    async fn pull_skips_missing_nar() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let pi = make_signed_pathinfo(&push_store, "hello", b"hello world").await;

        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&push_store, &[pi.clone()], &cache_dir, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();

        // Remove the NAR file.
        let nar_hash_b32 = nixbase32::encode(&pi.nar_sha256);
        let nar_path = cache_dir.join("nar").join(format!("{nar_hash_b32}.nar"));
        std::fs::remove_file(&nar_path).unwrap();

        let pull_store = open_test_store(&tmp.path().join("pull")).await;
        let report = import_paths_from_cache_dir(&pull_store, &cache_dir, None, &default_pull_options()).await.unwrap();

        assert_eq!(report.imported_count, 0);
        assert_eq!(report.skipped_missing_nar_count, 1);
    }

    #[tokio::test]
    async fn pull_rejects_store_dir_mismatch() {
        let tmp = tempfile::tempdir().unwrap();

        // Push from a /nix/store store.
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let pi = make_signed_pathinfo(&push_store, "hello", b"hello world").await;

        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&push_store, &[pi], &cache_dir, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();

        // Pull into a /crunch/store store — should reject.
        let pull_dir = tmp.path().join("pull");
        let state_dir = pull_dir.join("state");
        let output_dir = pull_dir.join("output");
        std::fs::create_dir_all(&state_dir).unwrap();
        std::fs::create_dir_all(&output_dir).unwrap();
        let pull_store = StoreHandle::open(StoreConfig {
            state_dir,
            output_dir,
            remote_cache_urls: Vec::new(),
            fallback_mode: StoreFallbackMode::Practical,
            store_dir: "/crunch/store".to_string(),
            base_state_dirs: Vec::new(),
        })
        .await
        .unwrap();

        let report = import_paths_from_cache_dir(&pull_store, &cache_dir, None, &PullOptions {
            trust_unsigned: true,
            trusted_public_keys: vec![],
        })
        .await
        .unwrap();

        assert_eq!(report.imported_count, 0);
        assert_eq!(report.skipped_store_dir_mismatch_count, 1);
    }

    #[tokio::test]
    async fn pull_with_path_filter() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let a = make_signed_pathinfo(&push_store, "pkg-a", b"aaa").await;
        let b = make_signed_pathinfo(&push_store, "pkg-b", b"bbb").await;
        let c = make_signed_pathinfo(&push_store, "pkg-c", b"ccc").await;

        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&push_store, &[a.clone(), b.clone(), c.clone()], &cache_dir, &PushOptions {
            trust_unsigned: false,
        })
        .await
        .unwrap();

        let pull_store = open_test_store(&tmp.path().join("pull")).await;
        let report = import_paths_from_cache_dir(
            &pull_store,
            &cache_dir,
            Some(&["pkg-a".to_string(), "pkg-c".to_string()]),
            &default_pull_options(),
        )
        .await
        .unwrap();

        assert_eq!(report.imported_count, 2);
        let imported_names: Vec<&str> = report.paths.iter().map(|p| p.store_path.as_str()).collect();
        assert!(imported_names.iter().any(|s| s.contains("pkg-a")));
        assert!(imported_names.iter().any(|s| s.contains("pkg-c")));
        assert!(!imported_names.iter().any(|s| s.contains("pkg-b")));
    }

    #[tokio::test]
    async fn pull_multiple_paths() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let a = make_signed_pathinfo(&push_store, "pkg-a", b"aaa").await;
        let b = make_signed_pathinfo(&push_store, "pkg-b", b"bbb").await;
        let c = make_signed_pathinfo(&push_store, "pkg-c", b"ccc").await;

        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&push_store, &[a, b, c], &cache_dir, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();

        let pull_store = open_test_store(&tmp.path().join("pull")).await;
        let report = import_paths_from_cache_dir(&pull_store, &cache_dir, None, &default_pull_options()).await.unwrap();

        assert_eq!(report.imported_count, 3);
        assert_eq!(report.paths.len(), 3);
    }

    #[tokio::test]
    async fn http_pull_single_signed_path_round_trip() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let pi = make_signed_pathinfo(&push_store, "http-hello", b"hello over http").await;
        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&push_store, &[pi.clone()], &cache_dir, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();

        let server = HttpTestServer::spawn(cache_routes(&cache_dir));
        let pull_store = open_test_store(&tmp.path().join("pull")).await;
        let report = import_paths_from_http_cache(
            &pull_store,
            &server.base_url,
            std::slice::from_ref(&pi.store_path),
            &default_pull_options(),
        )
        .await
        .unwrap();

        assert_eq!(report.imported_count, 1);
        assert_eq!(report.skipped_missing_nar_count, 0);
        let imported_pi = pull_store
            .pathinfo_service()
            .get((*pi.store_path.digest()).into())
            .await
            .unwrap()
            .expect("imported PathInfo must exist");
        assert_eq!(imported_pi.store_path, pi.store_path);
        assert_eq!(server.stats().narinfo_requests, 1);
        assert_eq!(server.stats().nar_requests, 1);
    }

    #[tokio::test]
    async fn http_pull_skips_already_present_without_narinfo_request() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let pi = make_signed_pathinfo(&push_store, "http-present", b"present").await;
        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&push_store, &[pi.clone()], &cache_dir, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();
        let server = HttpTestServer::spawn(cache_routes(&cache_dir));

        let pull_store = open_test_store(&tmp.path().join("pull")).await;
        let first_report = import_paths_from_http_cache(
            &pull_store,
            &server.base_url,
            std::slice::from_ref(&pi.store_path),
            &default_pull_options(),
        )
        .await
        .unwrap();
        assert_eq!(first_report.imported_count, 1);
        let after_first_stats = server.stats();
        let second_report = import_paths_from_http_cache(
            &pull_store,
            &server.base_url,
            std::slice::from_ref(&pi.store_path),
            &default_pull_options(),
        )
        .await
        .unwrap();
        let after_second_stats = server.stats();

        assert_eq!(second_report.imported_count, 0);
        assert_eq!(second_report.skipped_already_present_count, 1);
        assert_eq!(
            after_second_stats.nix_cache_info_requests,
            after_first_stats.nix_cache_info_requests.saturating_add(1)
        );
        assert_eq!(after_first_stats.narinfo_requests, after_second_stats.narinfo_requests);
        assert_eq!(after_first_stats.nar_requests, after_second_stats.nar_requests);
    }

    #[tokio::test]
    async fn http_pull_skips_missing_narinfo_404() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let pi = make_signed_pathinfo(&push_store, "http-404", b"missing narinfo").await;
        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&push_store, &[pi.clone()], &cache_dir, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();
        let mut routes = cache_routes(&cache_dir);
        routes.remove(&format!("/{}", narinfo_file_name(&pi.store_path)));
        let server = HttpTestServer::spawn(routes);

        let pull_store = open_test_store(&tmp.path().join("pull")).await;
        let report = import_paths_from_http_cache(
            &pull_store,
            &server.base_url,
            std::slice::from_ref(&pi.store_path),
            &default_pull_options(),
        )
        .await
        .unwrap();

        assert_eq!(report.imported_count, 0);
        assert_eq!(report.skipped_missing_nar_count, 1);
    }

    #[tokio::test]
    async fn http_pull_maps_narinfo_http_5xx_to_parse_error_count() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let pi = make_signed_pathinfo(&push_store, "http-narinfo-500", b"narinfo 500").await;
        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&push_store, &[pi.clone()], &cache_dir, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();
        let mut routes = cache_routes(&cache_dir);
        routes.insert(format!("/{}", narinfo_file_name(&pi.store_path)), HttpResponse::Fixed {
            status_line: "HTTP/1.1 500 Internal Server Error".to_string(),
            headers: vec![("Content-Type".to_string(), "text/plain".to_string())],
            body: b"boom".to_vec(),
        });
        let server = HttpTestServer::spawn(routes);
        let pull_store = open_test_store(&tmp.path().join("pull")).await;
        let report = import_paths_from_http_cache(
            &pull_store,
            &server.base_url,
            std::slice::from_ref(&pi.store_path),
            &default_pull_options(),
        )
        .await
        .unwrap();

        assert_eq!(report.imported_count, 0);
        assert_eq!(report.skipped_parse_error_count, 1);
    }

    #[tokio::test]
    async fn http_pull_maps_narinfo_http_403_to_missing_nar_count() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let pi = make_signed_pathinfo(&push_store, "http-narinfo-403", b"narinfo 403").await;
        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&push_store, &[pi.clone()], &cache_dir, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();
        let mut routes = cache_routes(&cache_dir);
        routes.insert(format!("/{}", narinfo_file_name(&pi.store_path)), HttpResponse::Fixed {
            status_line: "HTTP/1.1 403 Forbidden".to_string(),
            headers: vec![("Content-Type".to_string(), "text/plain".to_string())],
            body: b"forbidden".to_vec(),
        });
        let server = HttpTestServer::spawn(routes);
        let pull_store = open_test_store(&tmp.path().join("pull")).await;
        let report = import_paths_from_http_cache(
            &pull_store,
            &server.base_url,
            std::slice::from_ref(&pi.store_path),
            &default_pull_options(),
        )
        .await
        .unwrap();

        assert_eq!(report.imported_count, 0);
        assert_eq!(report.skipped_missing_nar_count, 1);
    }

    #[tokio::test]
    async fn http_pull_maps_other_narinfo_http_statuses_to_parse_error_count() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let pi = make_signed_pathinfo(&push_store, "http-narinfo-status", b"narinfo status").await;
        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&push_store, &[pi.clone()], &cache_dir, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();
        let cases = [
            ("400", "HTTP/1.1 400 Bad Request"),
            ("401", "HTTP/1.1 401 Unauthorized"),
            ("410", "HTTP/1.1 410 Gone"),
        ];

        for (case_name, status_line) in cases {
            let mut routes = cache_routes(&cache_dir);
            routes.insert(format!("/{}", narinfo_file_name(&pi.store_path)), HttpResponse::Fixed {
                status_line: status_line.to_string(),
                headers: vec![("Content-Type".to_string(), "text/plain".to_string())],
                body: case_name.as_bytes().to_vec(),
            });
            let server = HttpTestServer::spawn(routes);
            let pull_store = open_test_store(&tmp.path().join(format!("pull-narinfo-{case_name}"))).await;
            let report = import_paths_from_http_cache(
                &pull_store,
                &server.base_url,
                std::slice::from_ref(&pi.store_path),
                &default_pull_options(),
            )
            .await
            .unwrap();

            assert_eq!(report.imported_count, 0, "case={case_name}");
            assert_eq!(report.skipped_parse_error_count, 1, "case={case_name}");
        }
    }

    #[tokio::test]
    async fn http_pull_rejects_malformed_narinfo_text() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let pi = make_signed_pathinfo(&push_store, "http-bad-narinfo", b"bad narinfo").await;
        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&push_store, &[pi.clone()], &cache_dir, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();
        let mut routes = cache_routes(&cache_dir);
        routes.insert(
            format!("/{}", narinfo_file_name(&pi.store_path)),
            HttpResponse::ok_text("not a narinfo document\n".to_string()),
        );
        let server = HttpTestServer::spawn(routes);
        let pull_store = open_test_store(&tmp.path().join("pull")).await;
        let report = import_paths_from_http_cache(
            &pull_store,
            &server.base_url,
            std::slice::from_ref(&pi.store_path),
            &default_pull_options(),
        )
        .await
        .unwrap();

        assert_eq!(report.imported_count, 0);
        assert_eq!(report.skipped_parse_error_count, 1);
    }

    #[tokio::test]
    async fn http_pull_rejects_untrusted_signature() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let pi = make_signed_pathinfo(&push_store, "http-untrusted", b"signed").await;
        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&push_store, &[pi.clone()], &cache_dir, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();
        let server = HttpTestServer::spawn(cache_routes(&cache_dir));

        let other_secret = ed25519_dalek::SigningKey::from_bytes(&[99u8; 32]);
        let other_public = ed25519_dalek::VerifyingKey::from(&other_secret);
        let other_key = VerifyingKey::new("other-key-1".to_string(), other_public);
        let pull_store = open_test_store(&tmp.path().join("pull")).await;
        let report = import_paths_from_http_cache(
            &pull_store,
            &server.base_url,
            std::slice::from_ref(&pi.store_path),
            &PullOptions {
                trust_unsigned: false,
                trusted_public_keys: vec![other_key],
            },
        )
        .await
        .unwrap();

        assert_eq!(report.imported_count, 0);
        assert_eq!(report.skipped_untrusted_count, 1);
        assert_eq!(server.stats().nar_requests, 0);
    }

    #[tokio::test]
    async fn http_pull_rejects_absolute_nar_url_without_download() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let pi = make_signed_pathinfo(&push_store, "http-absolute-url", b"ssrf").await;
        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&push_store, &[pi.clone()], &cache_dir, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();
        let mut routes = cache_routes(&cache_dir);
        let narinfo_path = format!("/{}", narinfo_file_name(&pi.store_path));
        let narinfo_text = read_narinfo_text(&cache_dir, &pi.store_path);
        routes.insert(
            narinfo_path,
            HttpResponse::ok_text(replace_narinfo_field(&narinfo_text, "URL", "https://evil.example.com/out.nar")),
        );
        let server = HttpTestServer::spawn(routes);

        let pull_store = open_test_store(&tmp.path().join("pull")).await;
        let report = import_paths_from_http_cache(
            &pull_store,
            &server.base_url,
            std::slice::from_ref(&pi.store_path),
            &default_pull_options(),
        )
        .await
        .unwrap();

        assert_eq!(report.imported_count, 0);
        assert_eq!(report.skipped_parse_error_count, 1);
        assert_eq!(server.stats().nar_requests, 0);
    }

    #[tokio::test]
    async fn http_pull_accepts_unknown_key_signature_when_trust_unsigned() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let pi = make_signed_pathinfo(&push_store, "http-unknown-key-ok", b"unknown key").await;
        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&push_store, &[pi.clone()], &cache_dir, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();
        let server = HttpTestServer::spawn(cache_routes(&cache_dir));

        let pull_store = open_test_store(&tmp.path().join("pull")).await;
        let report = import_paths_from_http_cache(
            &pull_store,
            &server.base_url,
            std::slice::from_ref(&pi.store_path),
            &PullOptions {
                trust_unsigned: true,
                trusted_public_keys: vec![],
            },
        )
        .await
        .unwrap();

        assert_eq!(report.imported_count, 1);
    }

    #[tokio::test]
    async fn http_pull_rejects_unsigned_when_trust_unsigned_is_false() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let mut pi = make_signed_pathinfo(&push_store, "http-unsigned-reject", b"unsigned reject").await;
        pi.signatures.clear();
        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&push_store, &[pi.clone()], &cache_dir, &PushOptions { trust_unsigned: true })
            .await
            .unwrap();
        let server = HttpTestServer::spawn(cache_routes(&cache_dir));

        let pull_store = open_test_store(&tmp.path().join("pull-reject-unsigned")).await;
        let report = import_paths_from_http_cache(
            &pull_store,
            &server.base_url,
            std::slice::from_ref(&pi.store_path),
            &default_pull_options(),
        )
        .await
        .unwrap();

        assert_eq!(report.imported_count, 0);
        assert_eq!(report.skipped_untrusted_count, 1);
        assert_eq!(server.stats().nar_requests, 0);
    }

    #[tokio::test]
    async fn http_pull_accepts_unsigned_when_trust_unsigned() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let mut pi = make_signed_pathinfo(&push_store, "http-unsigned", b"unsigned").await;
        pi.signatures.clear();
        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&push_store, &[pi.clone()], &cache_dir, &PushOptions { trust_unsigned: true })
            .await
            .unwrap();
        let server = HttpTestServer::spawn(cache_routes(&cache_dir));

        let pull_store = open_test_store(&tmp.path().join("pull")).await;
        let report = import_paths_from_http_cache(
            &pull_store,
            &server.base_url,
            std::slice::from_ref(&pi.store_path),
            &PullOptions {
                trust_unsigned: true,
                trusted_public_keys: vec![],
            },
        )
        .await
        .unwrap();

        assert_eq!(report.imported_count, 1);
    }

    #[tokio::test]
    async fn http_pull_missing_and_malformed_nix_cache_info_both_proceed() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let pi = make_signed_pathinfo(&push_store, "http-cache-info", b"cache-info").await;
        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&push_store, &[pi.clone()], &cache_dir, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();

        let mut missing_routes = cache_routes(&cache_dir);
        missing_routes.remove(&format!("/{NIX_CACHE_INFO_FILE_NAME}"));
        let missing_server = HttpTestServer::spawn(missing_routes);
        let missing_store = open_test_store(&tmp.path().join("pull-missing")).await;
        let missing_report = import_paths_from_http_cache(
            &missing_store,
            &missing_server.base_url,
            std::slice::from_ref(&pi.store_path),
            &default_pull_options(),
        )
        .await
        .unwrap();
        assert_eq!(missing_report.imported_count, 1);

        let mut malformed_routes = cache_routes(&cache_dir);
        malformed_routes.insert(
            format!("/{NIX_CACHE_INFO_FILE_NAME}"),
            HttpResponse::ok_text("WantMassQuery: 1\nPriority: 30\n".to_string()),
        );
        let malformed_server = HttpTestServer::spawn(malformed_routes);
        let malformed_store = open_test_store(&tmp.path().join("pull-malformed")).await;
        let malformed_report = import_paths_from_http_cache(
            &malformed_store,
            &malformed_server.base_url,
            std::slice::from_ref(&pi.store_path),
            &default_pull_options(),
        )
        .await
        .unwrap();
        assert_eq!(malformed_report.imported_count, 1);
    }

    #[tokio::test]
    async fn http_pull_store_dir_mismatch_is_hard_error_before_narinfo_fetch() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let pi = make_signed_pathinfo(&push_store, "http-store-dir", b"dir mismatch").await;
        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&push_store, &[pi.clone()], &cache_dir, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();
        let mut routes = cache_routes(&cache_dir);
        routes.insert(
            format!("/{NIX_CACHE_INFO_FILE_NAME}"),
            HttpResponse::ok_text("StoreDir: /crunch/store\nWantMassQuery: 1\nPriority: 30\n".to_string()),
        );
        let server = HttpTestServer::spawn(routes);

        let state_dir = tmp.path().join("pull-state");
        let output_dir = tmp.path().join("pull-output");
        std::fs::create_dir_all(&state_dir).unwrap();
        std::fs::create_dir_all(&output_dir).unwrap();
        let pull_store = StoreHandle::open(StoreConfig {
            state_dir,
            output_dir,
            remote_cache_urls: Vec::new(),
            fallback_mode: StoreFallbackMode::Practical,
            store_dir: "/nix/store".to_string(),
            base_state_dirs: Vec::new(),
        })
        .await
        .unwrap();

        let error = import_paths_from_http_cache(
            &pull_store,
            &server.base_url,
            std::slice::from_ref(&pi.store_path),
            &default_pull_options(),
        )
        .await
        .unwrap_err();
        let stats = server.stats();

        assert!(error.to_string().contains("uses StoreDir /crunch/store"));
        assert_eq!(stats.narinfo_requests, 0);
    }

    #[tokio::test]
    async fn http_pull_handles_compressed_xz_nar() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let pi = make_signed_pathinfo(&push_store, "http-xz", b"compressed nar").await;
        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&push_store, &[pi.clone()], &cache_dir, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();

        let nar_hash_b32 = nixbase32::encode(&pi.nar_sha256);
        let original_nar_path = cache_dir.join("nar").join(format!("{nar_hash_b32}.nar"));
        let compressed_nar = compress_xz(&std::fs::read(&original_nar_path).unwrap());
        let compressed_name = format!("{nar_hash_b32}.nar.xz");
        let mut routes = cache_routes(&cache_dir);
        routes.insert(format!("/nar/{compressed_name}"), HttpResponse::ok_bytes(compressed_nar.clone()));
        let narinfo_text = read_narinfo_text(&cache_dir, &pi.store_path);
        let narinfo_text = replace_narinfo_field(&narinfo_text, "URL", &format!("nar/{compressed_name}"));
        let narinfo_text = replace_narinfo_field(&narinfo_text, "Compression", "xz");
        let narinfo_text =
            replace_narinfo_field(&narinfo_text, "FileHash", &format!("sha256:{}", nix_base32_sha256(&compressed_nar)));
        let narinfo_text = replace_narinfo_field(&narinfo_text, "FileSize", &compressed_nar.len().to_string());
        routes.insert(format!("/{}", narinfo_file_name(&pi.store_path)), HttpResponse::ok_text(narinfo_text));
        let server = HttpTestServer::spawn(routes);

        let pull_store = open_test_store(&tmp.path().join("pull")).await;
        let report = import_paths_from_http_cache(
            &pull_store,
            &server.base_url,
            std::slice::from_ref(&pi.store_path),
            &default_pull_options(),
        )
        .await
        .unwrap();

        assert_eq!(report.imported_count, 1);
    }

    #[tokio::test]
    async fn http_pull_does_not_recurse_into_missing_references() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let referenced = make_signed_pathinfo(&push_store, "http-ref", b"ref").await;
        let root = make_signed_pathinfo_with_references(&push_store, "http-root", b"root", vec![
            referenced.store_path.clone(),
        ])
        .await;
        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&push_store, &[root.clone()], &cache_dir, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();
        let server = HttpTestServer::spawn(cache_routes(&cache_dir));

        let pull_store = open_test_store(&tmp.path().join("pull")).await;
        let report = import_paths_from_http_cache(
            &pull_store,
            &server.base_url,
            std::slice::from_ref(&root.store_path),
            &default_pull_options(),
        )
        .await
        .unwrap();
        let imported_pi = pull_store
            .pathinfo_service()
            .get((*root.store_path.digest()).into())
            .await
            .unwrap()
            .expect("imported root must exist");

        assert_eq!(report.imported_count, 1);
        assert_eq!(imported_pi.references, vec![referenced.store_path.clone()]);
        assert_eq!(server.stats().narinfo_requests, 1);
    }

    #[tokio::test]
    async fn http_pull_parses_references_with_local_store_prefix() {
        let tmp = tempfile::tempdir().unwrap();
        let store_dir = "/crunch/store";
        let push_store = open_test_store_with_store_dir(&tmp.path().join("push"), store_dir).await;
        let referenced = make_signed_pathinfo(&push_store, "http-ref-prefix", b"ref").await;
        let root = make_signed_pathinfo_with_references(&push_store, "http-root-prefix", b"root", vec![
            referenced.store_path.clone(),
        ])
        .await;
        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&push_store, &[root.clone()], &cache_dir, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();
        let server = HttpTestServer::spawn(cache_routes(&cache_dir));

        let pull_store = open_test_store_with_store_dir(&tmp.path().join("pull"), store_dir).await;
        let report = import_paths_from_http_cache(
            &pull_store,
            &server.base_url,
            std::slice::from_ref(&root.store_path),
            &default_pull_options(),
        )
        .await
        .unwrap();
        let imported_pi = pull_store
            .pathinfo_service()
            .get((*root.store_path.digest()).into())
            .await
            .unwrap()
            .expect("imported root must exist");

        assert_eq!(report.imported_count, 1);
        assert_eq!(imported_pi.references, vec![referenced.store_path.clone()]);
        assert_eq!(
            imported_pi.references[0].to_absolute_path_with_prefix(store_dir),
            referenced.store_path.to_absolute_path_with_prefix(store_dir)
        );
    }

    #[tokio::test]
    async fn http_pull_rejects_malformed_or_wrong_prefix_references_before_persistence() {
        let tmp = tempfile::tempdir().unwrap();
        let store_dir = "/crunch/store";
        let push_store = open_test_store_with_store_dir(&tmp.path().join("push"), store_dir).await;
        let referenced = make_signed_pathinfo(&push_store, "http-ref-invalid", b"ref").await;
        let root = make_signed_pathinfo_with_references(&push_store, "http-root-invalid", b"root", vec![
            referenced.store_path.clone(),
        ])
        .await;
        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&push_store, &[root.clone()], &cache_dir, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();
        let narinfo_text = read_narinfo_text(&cache_dir, &root.store_path);
        let cases = [
            ("malformed", "not-a-store-path".to_string()),
            ("absolute-local-prefix", format!("{store_dir}/{}", referenced.store_path)),
            ("wrong-prefix", format!("/nix/store/{}", referenced.store_path)),
        ];

        for (case_name, invalid_references) in cases {
            let mut routes = cache_routes(&cache_dir);
            routes.insert(
                format!("/{}", narinfo_file_name(&root.store_path)),
                HttpResponse::ok_text(replace_narinfo_field(&narinfo_text, "References", &invalid_references)),
            );
            let server = HttpTestServer::spawn(routes);
            let pull_store =
                open_test_store_with_store_dir(&tmp.path().join(format!("pull-{case_name}")), store_dir).await;
            let report = import_paths_from_http_cache(
                &pull_store,
                &server.base_url,
                std::slice::from_ref(&root.store_path),
                &PullOptions {
                    trust_unsigned: true,
                    trusted_public_keys: vec![],
                },
            )
            .await
            .unwrap();
            let imported_pi = pull_store.pathinfo_service().get((*root.store_path.digest()).into()).await.unwrap();

            assert_eq!(report.imported_count, 0, "case={case_name}");
            assert_eq!(report.skipped_parse_error_count, 1, "case={case_name}");
            assert_eq!(report.skipped_store_dir_mismatch_count, 0, "case={case_name}");
            assert!(imported_pi.is_none(), "case={case_name}");
        }
    }

    #[tokio::test]
    async fn http_pull_network_failure_continues_for_remaining_paths() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let ok_pi = make_signed_pathinfo(&push_store, "http-ok", b"ok").await;
        let drop_pi = make_signed_pathinfo(&push_store, "http-drop", b"drop").await;
        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&push_store, &[ok_pi.clone(), drop_pi.clone()], &cache_dir, &PushOptions {
            trust_unsigned: false,
        })
        .await
        .unwrap();
        let drop_nar_route = format!("/nar/{}.nar", nixbase32::encode(&drop_pi.nar_sha256));
        let mut routes = cache_routes(&cache_dir);
        routes.insert(drop_nar_route, HttpResponse::DropConnection);
        let server = HttpTestServer::spawn(routes);

        let pull_store = open_test_store(&tmp.path().join("pull")).await;
        let requested_paths = vec![ok_pi.store_path.clone(), drop_pi.store_path.clone()];
        let report =
            import_paths_from_http_cache(&pull_store, &server.base_url, &requested_paths, &default_pull_options())
                .await
                .unwrap();

        assert_eq!(report.imported_count, 1);
        assert_eq!(report.skipped_parse_error_count, 1);
    }

    #[tokio::test]
    async fn http_pull_nix_cache_info_client_error_warning_proceeds() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let pi = make_signed_pathinfo(&push_store, "http-cache-info-client-error", b"client error").await;
        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&push_store, &[pi.clone()], &cache_dir, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();
        let cases = [("401", "HTTP/1.1 401 Unauthorized"), ("410", "HTTP/1.1 410 Gone")];

        for (case_name, status_line) in cases {
            let mut routes = cache_routes(&cache_dir);
            routes.insert(format!("/{NIX_CACHE_INFO_FILE_NAME}"), HttpResponse::Fixed {
                status_line: status_line.to_string(),
                headers: vec![("Content-Type".to_string(), "text/plain".to_string())],
                body: case_name.as_bytes().to_vec(),
            });
            let server = HttpTestServer::spawn(routes);
            let pull_store = open_test_store(&tmp.path().join(format!("pull-cache-info-{case_name}"))).await;
            let report = import_paths_from_http_cache(
                &pull_store,
                &server.base_url,
                std::slice::from_ref(&pi.store_path),
                &default_pull_options(),
            )
            .await
            .unwrap();

            assert_eq!(report.imported_count, 1, "case={case_name}");
        }
    }

    #[tokio::test]
    async fn http_pull_nix_cache_info_redirect_rejection_warns_and_proceeds() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let pi = make_signed_pathinfo(&push_store, "http-cache-info-redirect", b"redirected cache info").await;
        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&push_store, &[pi.clone()], &cache_dir, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();
        let mut routes = cache_routes(&cache_dir);
        routes.insert(format!("/{NIX_CACHE_INFO_FILE_NAME}"), HttpResponse::redirect("/elsewhere"));
        let server = HttpTestServer::spawn(routes);
        let pull_store = open_test_store(&tmp.path().join("pull")).await;
        let report = import_paths_from_http_cache(
            &pull_store,
            &server.base_url,
            std::slice::from_ref(&pi.store_path),
            &default_pull_options(),
        )
        .await
        .unwrap();

        assert_eq!(report.imported_count, 1);
    }

    #[tokio::test]
    async fn http_pull_maps_nar_http_failure_to_missing_nar_count() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let pi = make_signed_pathinfo(&push_store, "http-nar-500", b"nar 500").await;
        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&push_store, &[pi.clone()], &cache_dir, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();
        let nar_route = format!("/nar/{}.nar", nixbase32::encode(&pi.nar_sha256));
        let mut routes = cache_routes(&cache_dir);
        routes.insert(nar_route, HttpResponse::Fixed {
            status_line: "HTTP/1.1 500 Internal Server Error".to_string(),
            headers: vec![("Content-Type".to_string(), "text/plain".to_string())],
            body: b"boom".to_vec(),
        });
        let server = HttpTestServer::spawn(routes);

        let pull_store = open_test_store(&tmp.path().join("pull")).await;
        let report = import_paths_from_http_cache(
            &pull_store,
            &server.base_url,
            std::slice::from_ref(&pi.store_path),
            &default_pull_options(),
        )
        .await
        .unwrap();

        assert_eq!(report.imported_count, 0);
        assert_eq!(report.skipped_missing_nar_count, 1);
    }

    #[tokio::test]
    async fn http_pull_maps_other_nar_http_statuses_to_missing_nar_count() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let pi = make_signed_pathinfo(&push_store, "http-nar-status", b"nar status").await;
        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&push_store, &[pi.clone()], &cache_dir, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();
        let nar_route = format!("/nar/{}.nar", nixbase32::encode(&pi.nar_sha256));
        let cases = [
            ("401", "HTTP/1.1 401 Unauthorized"),
            ("403", "HTTP/1.1 403 Forbidden"),
            ("410", "HTTP/1.1 410 Gone"),
        ];

        for (case_name, status_line) in cases {
            let mut routes = cache_routes(&cache_dir);
            routes.insert(nar_route.clone(), HttpResponse::Fixed {
                status_line: status_line.to_string(),
                headers: vec![("Content-Type".to_string(), "text/plain".to_string())],
                body: case_name.as_bytes().to_vec(),
            });
            let server = HttpTestServer::spawn(routes);
            let pull_store = open_test_store(&tmp.path().join(format!("pull-nar-{case_name}"))).await;
            let report = import_paths_from_http_cache(
                &pull_store,
                &server.base_url,
                std::slice::from_ref(&pi.store_path),
                &default_pull_options(),
            )
            .await
            .unwrap();

            assert_eq!(report.imported_count, 0, "case={case_name}");
            assert_eq!(report.skipped_missing_nar_count, 1, "case={case_name}");
        }
    }

    #[tokio::test]
    async fn http_pull_blocks_cross_scheme_redirects() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let pi = make_signed_pathinfo(&push_store, "http-redirect", b"redirect").await;
        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&push_store, &[pi.clone()], &cache_dir, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();
        let nar_route = format!("/nar/{}.nar", nixbase32::encode(&pi.nar_sha256));
        let mut routes = cache_routes(&cache_dir);
        routes.insert(nar_route, HttpResponse::redirect("file:///tmp/evil.nar"));
        let server = HttpTestServer::spawn(routes);

        let pull_store = open_test_store(&tmp.path().join("pull")).await;
        let report = import_paths_from_http_cache(
            &pull_store,
            &server.base_url,
            std::slice::from_ref(&pi.store_path),
            &default_pull_options(),
        )
        .await
        .unwrap();

        assert_eq!(report.imported_count, 0);
        assert_eq!(report.skipped_parse_error_count, 1);
    }

    #[tokio::test]
    async fn http_pull_handles_gzip_bzip2_and_zstd_nar() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let pi = make_signed_pathinfo(&push_store, "http-multi-codec", b"multi codec").await;
        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&push_store, &[pi.clone()], &cache_dir, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();
        let nar_hash_b32 = nixbase32::encode(&pi.nar_sha256);
        let original_nar_path = cache_dir.join("nar").join(format!("{nar_hash_b32}.nar"));
        let original_nar = std::fs::read(&original_nar_path).unwrap();
        let cases: [(&str, fn(&[u8]) -> Vec<u8>); 3] = [
            ("gzip", compress_gzip),
            ("bzip2", compress_bzip2),
            ("zstd", compress_zstd),
        ];

        for (compression, encode) in cases {
            let compressed_nar = encode(&original_nar);
            let compressed_name = format!("{nar_hash_b32}.nar.{compression}");
            let mut routes = cache_routes(&cache_dir);
            routes.insert(format!("/nar/{compressed_name}"), HttpResponse::ok_bytes(compressed_nar.clone()));
            let narinfo_text = read_narinfo_text(&cache_dir, &pi.store_path);
            let narinfo_text = replace_narinfo_field(&narinfo_text, "URL", &format!("nar/{compressed_name}"));
            let narinfo_text = replace_narinfo_field(&narinfo_text, "Compression", compression);
            let narinfo_text = replace_narinfo_field(
                &narinfo_text,
                "FileHash",
                &format!("sha256:{}", nix_base32_sha256(&compressed_nar)),
            );
            let narinfo_text = replace_narinfo_field(&narinfo_text, "FileSize", &compressed_nar.len().to_string());
            routes.insert(format!("/{}", narinfo_file_name(&pi.store_path)), HttpResponse::ok_text(narinfo_text));
            let server = HttpTestServer::spawn(routes);
            let pull_store = open_test_store(&tmp.path().join(format!("pull-{compression}"))).await;
            let report = import_paths_from_http_cache(
                &pull_store,
                &server.base_url,
                std::slice::from_ref(&pi.store_path),
                &default_pull_options(),
            )
            .await
            .unwrap();
            assert_eq!(report.imported_count, 1, "compression={compression}");
        }
    }

    #[tokio::test]
    async fn http_pull_continues_after_narinfo_fetch_transport_failure() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let ok_pi = make_signed_pathinfo(&push_store, "http-narinfo-ok", b"ok").await;
        let drop_pi = make_signed_pathinfo(&push_store, "http-narinfo-drop", b"drop").await;
        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&push_store, &[ok_pi.clone(), drop_pi.clone()], &cache_dir, &PushOptions {
            trust_unsigned: false,
        })
        .await
        .unwrap();
        let mut routes = cache_routes(&cache_dir);
        routes.insert(format!("/{}", narinfo_file_name(&drop_pi.store_path)), HttpResponse::DropConnection);
        let server = HttpTestServer::spawn(routes);

        let pull_store = open_test_store(&tmp.path().join("pull")).await;
        let requested_paths = vec![ok_pi.store_path.clone(), drop_pi.store_path.clone()];
        let report =
            import_paths_from_http_cache(&pull_store, &server.base_url, &requested_paths, &default_pull_options())
                .await
                .unwrap();

        assert_eq!(report.imported_count, 1);
        assert_eq!(report.skipped_parse_error_count, 1);
    }

    #[tokio::test]
    async fn http_pull_keeps_path_prefixed_cache_urls_stable() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let pi = make_signed_pathinfo(&push_store, "http-prefixed-base", b"prefixed").await;
        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&push_store, &[pi.clone()], &cache_dir, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();
        let mut prefixed_routes = BTreeMap::new();
        for (route, response) in cache_routes(&cache_dir) {
            prefixed_routes.insert(format!("/cache{}", route), response);
        }
        let server = HttpTestServer::spawn(prefixed_routes);
        let base_cases = [
            format!("{}{}", server.base_url, "cache"),
            format!("{}{}", server.base_url, "cache/"),
            format!("{}{}", server.base_url, "cache?ignored=1#fragment"),
            format!("{}{}", server.base_url, "cache/?ignored=1#fragment"),
        ];

        for (index, base_case) in base_cases.iter().enumerate() {
            let pull_store = open_test_store(&tmp.path().join(format!("pull-prefix-{index}"))).await;
            let cache_url = Url::parse(base_case).unwrap();
            let report = import_paths_from_http_cache(
                &pull_store,
                &cache_url,
                std::slice::from_ref(&pi.store_path),
                &default_pull_options(),
            )
            .await
            .unwrap();
            assert_eq!(report.imported_count, 1, "base_case={base_case}");
        }
    }

    #[tokio::test]
    async fn http_pull_rejects_narinfo_store_path_prefix_mismatch_after_matching_preflight() {
        let tmp = tempfile::tempdir().unwrap();
        let store_dir = "/crunch/store";
        let push_store = open_test_store_with_store_dir(&tmp.path().join("push"), store_dir).await;
        let pi = make_signed_pathinfo(&push_store, "http-prefix-mismatch", b"prefix mismatch").await;
        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&push_store, &[pi.clone()], &cache_dir, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();
        let mut routes = cache_routes(&cache_dir);
        let narinfo_text = read_narinfo_text(&cache_dir, &pi.store_path);
        routes.insert(
            format!("/{}", narinfo_file_name(&pi.store_path)),
            HttpResponse::ok_text(replace_narinfo_field(
                &narinfo_text,
                "StorePath",
                &format!("/nix/store/{}", pi.store_path),
            )),
        );
        let server = HttpTestServer::spawn(routes);
        let pull_store = open_test_store_with_store_dir(&tmp.path().join("pull"), store_dir).await;
        let report = import_paths_from_http_cache(
            &pull_store,
            &server.base_url,
            std::slice::from_ref(&pi.store_path),
            &default_pull_options(),
        )
        .await
        .unwrap();

        assert_eq!(report.imported_count, 0);
        assert_eq!(report.skipped_store_dir_mismatch_count, 1);
        assert_eq!(server.stats().nar_requests, 0);
    }

    #[tokio::test]
    async fn http_pull_rejects_cache_base_url_userinfo() {
        let tmp = tempfile::tempdir().unwrap();
        let pull_store = open_test_store(&tmp.path().join("pull")).await;
        let cache_url = Url::parse("http://user@cache.example.com/cache").unwrap();
        let requested_path = StorePath::from_name_and_digest_fixed("userinfo", [7u8; 20]).unwrap();
        let error = import_paths_from_http_cache(
            &pull_store,
            &cache_url,
            std::slice::from_ref(&requested_path),
            &default_pull_options(),
        )
        .await
        .unwrap_err();

        assert!(error.to_string().contains("must not include URL credentials"));
    }

    #[tokio::test]
    async fn http_pull_detects_store_path_mismatch_and_client_metadata() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let pi = make_signed_pathinfo(&push_store, "http-mismatch", b"mismatch").await;
        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&push_store, &[pi.clone()], &cache_dir, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();
        let digest_text = nixbase32::encode(pi.store_path.digest());
        let mismatched_store_path = format!("/nix/store/{digest_text}-other-name");
        let mut routes = cache_routes(&cache_dir);
        let narinfo_text = read_narinfo_text(&cache_dir, &pi.store_path);
        routes.insert(
            format!("/{}", narinfo_file_name(&pi.store_path)),
            HttpResponse::ok_text(replace_narinfo_field(&narinfo_text, "StorePath", &mismatched_store_path)),
        );
        let server = HttpTestServer::spawn(routes);
        let pull_store = open_test_store(&tmp.path().join("pull")).await;
        let report = import_paths_from_http_cache(
            &pull_store,
            &server.base_url,
            std::slice::from_ref(&pi.store_path),
            &PullOptions {
                trust_unsigned: true,
                trusted_public_keys: vec![],
            },
        )
        .await
        .unwrap();
        let config = http_pull_client_config();
        let stats = server.stats();

        assert_eq!(report.imported_count, 0);
        assert_eq!(report.skipped_parse_error_count, 1);
        assert_eq!(config.connect_timeout_secs, HTTP_PULL_CONNECT_TIMEOUT_SECS);
        assert_eq!(config.read_timeout_secs, HTTP_PULL_READ_TIMEOUT_SECS);
        assert_eq!(config.max_redirect_hops, HTTP_PULL_MAX_REDIRECT_HOPS);
        assert_eq!(stats.last_user_agent, Some(http_pull_user_agent()));
    }

    #[tokio::test]
    async fn http_pull_pathinfo_persistence_failure_is_fatal() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let pi = make_signed_pathinfo(&push_store, "http-put-fail", b"put fail").await;
        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&push_store, &[pi.clone()], &cache_dir, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();
        let server = HttpTestServer::spawn(cache_routes(&cache_dir));
        let state_dir = tmp.path().join("pull-state");
        let output_dir = tmp.path().join("pull-output");
        std::fs::create_dir_all(&output_dir).unwrap();
        let pull_store = test_service_backed_store(
            &state_dir,
            output_dir.display().to_string(),
            "/nix/store",
            Arc::new(PutFailingPathInfoService {
                inner: default_test_pathinfo_service(),
                detail: "injected pathinfo put failure".to_string(),
            }) as Arc<dyn PathInfoService>,
        );

        let error = import_paths_from_http_cache(
            &pull_store,
            &server.base_url,
            std::slice::from_ref(&pi.store_path),
            &default_pull_options(),
        )
        .await
        .unwrap_err();
        let imported_path = pi.store_path.to_absolute_path_with_prefix(pull_store.output_dir_str());

        assert!(error.to_string().contains("persisting imported PathInfo"));
        assert!(error.to_string().contains("injected pathinfo put failure"));
        assert!(!std::path::Path::new(&imported_path).exists());
    }

    #[tokio::test]
    async fn http_pull_export_failure_after_persistence_is_fatal() {
        let tmp = tempfile::tempdir().unwrap();
        let push_store = open_test_store(&tmp.path().join("push")).await;
        let pi = make_signed_pathinfo(&push_store, "http-export-fail", b"export fail").await;
        let cache_dir = tmp.path().join("cache");
        export_paths_to_cache_dir(&push_store, &[pi.clone()], &cache_dir, &PushOptions { trust_unsigned: false })
            .await
            .unwrap();
        let server = HttpTestServer::spawn(cache_routes(&cache_dir));
        let state_dir = tmp.path().join("pull-state");
        let output_root = tmp.path().join("not-a-directory");
        std::fs::write(&output_root, b"file blocks export path").unwrap();
        let pull_store = test_service_backed_store(
            &state_dir,
            output_root.display().to_string(),
            "/nix/store",
            default_test_pathinfo_service(),
        );

        let error = import_paths_from_http_cache(
            &pull_store,
            &server.base_url,
            std::slice::from_ref(&pi.store_path),
            &default_pull_options(),
        )
        .await
        .unwrap_err();
        let imported_pi = pull_store
            .pathinfo_service()
            .get((*pi.store_path.digest()).into())
            .await
            .unwrap()
            .expect("PathInfo must persist before export fails");
        let imported_path = pi.store_path.to_absolute_path_with_prefix(pull_store.output_dir_str());

        assert!(error.to_string().contains("exporting imported HTTP path"));
        assert_eq!(imported_pi.store_path, pi.store_path);
        assert!(!std::path::Path::new(&imported_path).exists());
    }

    #[tokio::test]
    async fn pull_nonexistent_source_returns_error() {
        let tmp = tempfile::tempdir().unwrap();
        let store = open_test_store(&tmp.path().join("store")).await;

        let result =
            import_paths_from_cache_dir(&store, &tmp.path().join("nonexistent"), None, &default_pull_options()).await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("does not exist"), "error: {err}");
    }
}
