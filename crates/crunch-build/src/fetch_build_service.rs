//! FetchBuildService: `BuildService` for fetcher derivations.
//!
//! Wraps the download/extraction logic from `fetcher.rs` behind the
//! `BuildService` trait. Downloads resources, ingests them into castore,
//! and returns a `BuildResult` with the output node.
//!
//! Does NOT verify fixed-output hashes, persist PathInfo, or export to
//! disk — those stay in the shared `finish_build` path.

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;
use snix_build::buildservice::BuildOutput;
use snix_build::buildservice::BuildRequest;
use snix_build::buildservice::BuildResult;
use snix_build::buildservice::BuildService;
use tracing::info;
use url::Url;

use crate::fetcher::FetchError;
use crate::fetcher::{self};

/// The builder string that identifies builtin fetcher derivations.
pub const FETCH_BUILDER: &str = "builtin:fetchurl";
pub const FOREIGN_FETCH_CANDIDATES_ENV: &str = "__mantle_foreign_candidates";
pub const FOREIGN_FETCH_ATTEMPT_LOG_SCHEMA: &str = "mantle-foreign-fetch-attempts-v1";

const MAX_FOREIGN_FETCH_CANDIDATES: usize = 16;
const MAX_SOURCE_OVERRIDES: usize = 65_536;
const MAX_OVERRIDE_COPY_ENTRIES: usize = 262_144;
#[cfg(unix)]
const UNIX_EXECUTABLE_FILE_MODE: u32 = 0o755;

/// Returns `true` if this `BuildRequest` targets the builtin fetcher.
pub fn is_fetch_request(request: &BuildRequest) -> bool {
    request.command_args.first().is_some_and(|b| b == FETCH_BUILDER)
}

// ── FetchKind ──────────────────────────────────────────────────────────

/// What to download and how to process it.
///
/// Stripped-down variant of `fetcher::Fetch` without hash information.
/// Verification is the caller's concern (the shared post-build path).
#[derive(Clone, Debug, PartialEq, Eq)]
enum FetchKind {
    /// Download a single file.
    File { url: Url },
    /// Download a tarball, decompress, and extract.
    Tarball { url: Url },
    /// Download a single file, mark executable.
    Executable { url: Url },
    /// Clone a git repo at a specific revision.
    Git { url: String, rev: String },
}

impl FetchKind {
    fn source_identity(&self) -> String {
        match self {
            Self::File { url } => format!("kind=file url={url}"),
            Self::Tarball { url } => format!("kind=tarball url={url}"),
            Self::Executable { url } => format!("kind=executable url={url}"),
            Self::Git { url, rev } => format!("kind=git url={url} rev={rev}"),
        }
    }

    fn candidate(&self) -> &str {
        match self {
            Self::File { url } | Self::Tarball { url } | Self::Executable { url } => url.as_str(),
            Self::Git { url, .. } => url,
        }
    }

    fn with_candidate(&self, candidate: &str) -> Result<Self, FetchError> {
        let parsed = Url::parse(candidate).map_err(|error| FetchError::InvalidUrl(format!("{candidate}: {error}")))?;
        Ok(match self {
            Self::File { .. } => Self::File { url: parsed },
            Self::Tarball { .. } => Self::Tarball { url: parsed },
            Self::Executable { .. } => Self::Executable { url: parsed },
            Self::Git { rev, .. } => Self::Git {
                url: candidate.to_string(),
                rev: rev.clone(),
            },
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ForeignFetchAttempt {
    pub index: usize,
    pub candidate: String,
    pub classification: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ForeignFetchAttemptLog {
    pub schema: String,
    pub attempts: Vec<ForeignFetchAttempt>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FetchSourcePolicy {
    #[default]
    AllowNetwork,
    RequireOverride,
}

impl FetchSourcePolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AllowNetwork => "allow-network",
            Self::RequireOverride => "require-override",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "allow-network" => Some(Self::AllowNetwork),
            "require-override" => Some(Self::RequireOverride),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FetchSourceOverrideKind {
    File,
    Tarball,
    Executable,
    Git,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FetchSourceOverride {
    pub url: String,
    pub kind: FetchSourceOverrideKind,
    pub rev: Option<String>,
    pub payload_path: PathBuf,
    pub source_state_blake3: String,
}

impl FetchSourceOverride {
    fn matches(&self, fetch: &FetchKind) -> bool {
        match (self.kind, fetch) {
            (FetchSourceOverrideKind::File, FetchKind::File { url }) => self.url == url.as_str(),
            (FetchSourceOverrideKind::Tarball, FetchKind::Tarball { url }) => self.url == url.as_str(),
            (FetchSourceOverrideKind::Executable, FetchKind::Executable { url }) => self.url == url.as_str(),
            (FetchSourceOverrideKind::Git, FetchKind::Git { url, rev }) => {
                self.url == *url && self.rev.as_deref() == Some(rev.as_str())
            }
            _ => false,
        }
    }
}

/// Parse fetch parameters from a `BuildRequest`.
///
/// Reads `command_args[0]` for the builder selector and
/// `environment_vars` for `url`, `unpack`, `type`, `rev`, `executable`.
/// Does NOT extract hash information.
fn parse_fetch_kind(request: &BuildRequest) -> Result<FetchKind, FetchError> {
    // Tiger Style: precondition.
    debug_assert!(!request.command_args.is_empty(), "command_args must not be empty");

    if !is_fetch_request(request) {
        let builder = request.command_args.first().map(|s| s.as_str()).unwrap_or("(none)");
        return Err(FetchError::NotFetcher(builder.to_string()));
    }

    // Helper: find a string env var.
    let env_str = |key: &str| -> Option<String> {
        request
            .environment_vars
            .iter()
            .find(|e| e.key == key)
            .and_then(|e| String::from_utf8(e.value.to_vec()).ok())
    };
    // Helper: check a boolean flag env var ("1" = true).
    let env_flag =
        |key: &str| -> bool { request.environment_vars.iter().any(|e| e.key == key && e.value.as_ref() == b"1") };

    // Git fetch: type == "git"
    if env_str("type").as_deref() == Some("git") {
        let url = env_str("url").ok_or(FetchError::MissingUrl)?;
        let rev = env_str("rev").ok_or(FetchError::MissingGitRev)?;
        return Ok(FetchKind::Git { url, rev });
    }

    let url_str = env_str("url").ok_or(FetchError::MissingUrl)?;
    let url = Url::parse(&url_str).map_err(|e| FetchError::InvalidUrl(format!("{url_str}: {e}")))?;

    if env_flag("unpack") {
        Ok(FetchKind::Tarball { url })
    } else if env_flag("executable") {
        Ok(FetchKind::Executable { url })
    } else {
        Ok(FetchKind::File { url })
    }
}

fn ordered_fetch_kinds(request: &BuildRequest, primary: &FetchKind) -> Result<Vec<FetchKind>, FetchError> {
    let Some(serialized) =
        request.environment_vars.iter().find(|environment| environment.key == FOREIGN_FETCH_CANDIDATES_ENV)
    else {
        return Ok(vec![primary.clone()]);
    };
    let candidates = serde_json::from_slice::<Vec<String>>(serialized.value.as_ref())
        .map_err(|error| FetchError::InvalidCandidateList(format!("invalid JSON: {error}")))?;
    if candidates.is_empty() || candidates.len() > MAX_FOREIGN_FETCH_CANDIDATES {
        return Err(FetchError::InvalidCandidateList(format!(
            "candidate count must be between 1 and {MAX_FOREIGN_FETCH_CANDIDATES}"
        )));
    }
    let mut seen = BTreeSet::new();
    let mut kinds = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        if !seen.insert(candidate.clone()) {
            return Err(FetchError::InvalidCandidateList(format!("duplicate candidate: {candidate}")));
        }
        kinds.push(primary.with_candidate(&candidate)?);
    }
    if kinds[0] != *primary {
        return Err(FetchError::InvalidCandidateList("first candidate differs from the derivation URL".to_string()));
    }
    debug_assert!(!kinds.is_empty());
    debug_assert!(kinds.len() <= MAX_FOREIGN_FETCH_CANDIDATES);
    Ok(kinds)
}

fn fetch_error_is_candidate_unavailable(error: &FetchError) -> bool {
    matches!(error, FetchError::HttpError { .. } | FetchError::GitError(_) | FetchError::Io(_))
}

fn foreign_fetch_attempt_log(attempts: Vec<ForeignFetchAttempt>) -> Result<Option<String>, FetchError> {
    if attempts.is_empty() {
        return Ok(None);
    }
    serde_json::to_string(&ForeignFetchAttemptLog {
        schema: FOREIGN_FETCH_ATTEMPT_LOG_SCHEMA.to_string(),
        attempts,
    })
    .map(Some)
    .map_err(|error| FetchError::InvalidCandidateList(format!("serializing attempt log: {error}")))
}

fn remove_fetch_output(path: &Path) -> Result<(), FetchError> {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return Ok(());
    };
    if metadata.is_dir() {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }
    Ok(())
}

// ── Execute fetch (blocking) ──────────────────────────────────────────

/// Execute a fetch, writing output to `out_path`.
///
/// Blocking — call from `spawn_blocking`.
fn execute_fetch(kind: &FetchKind, out_path: &str) -> Result<(), FetchError> {
    if let Some(parent) = std::path::Path::new(out_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    match kind {
        FetchKind::File { url } => {
            info!(url = %url, "fetching file");
            fetcher::fetch_flat(url.as_str(), out_path)?;
        }
        FetchKind::Tarball { url } => {
            info!(url = %url, "fetching tarball");
            fetcher::fetch_and_unpack(url.as_str(), out_path)?;
        }
        FetchKind::Executable { url } => {
            info!(url = %url, "fetching executable");
            fetcher::fetch_flat(url.as_str(), out_path)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(out_path, std::fs::Permissions::from_mode(UNIX_EXECUTABLE_FILE_MODE))?;
            }
        }
        FetchKind::Git { url, rev } => {
            let short_rev = &rev[..rev.len().min(12)];
            info!(url = %url, rev = %short_rev, "fetching git");
            fetcher::fetch_git(url, rev, out_path)?;
        }
    }

    // Tiger Style: post-condition.
    debug_assert!(std::path::Path::new(out_path).exists(), "fetch must produce output at {out_path}");

    Ok(())
}

fn source_override_for_kind<'a>(
    kind: &FetchKind,
    source_overrides: &'a [FetchSourceOverride],
) -> Option<&'a FetchSourceOverride> {
    debug_assert!(source_overrides.len() <= MAX_SOURCE_OVERRIDES, "source override list must be bounded");
    source_overrides.iter().find(|source_override| source_override.matches(kind))
}

fn materialize_source_override(source_override: &FetchSourceOverride, out_path: &Path) -> Result<(), FetchError> {
    debug_assert!(!source_override.url.is_empty(), "source override URL must not be empty");
    debug_assert!(!source_override.source_state_blake3.is_empty(), "source state digest must not be empty");
    copy_override_payload(&source_override.payload_path, out_path)?;
    if source_override.kind == FetchSourceOverrideKind::Executable {
        set_executable_mode(out_path)?;
    }
    Ok(())
}

fn copy_override_payload(source: &Path, target: &Path) -> Result<(), FetchError> {
    let metadata = fs::symlink_metadata(source)?;
    if metadata.is_dir() {
        return copy_override_dir(source, target);
    }
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    copy_override_leaf(source, target, &metadata)
}

fn copy_override_dir(source: &Path, target: &Path) -> Result<(), FetchError> {
    assert!(source.is_dir(), "source override directory must exist");
    assert!(!target.as_os_str().is_empty(), "source override target must not be empty");
    let mut stack = vec![(source.to_path_buf(), target.to_path_buf())];
    let mut copied_entries = 0usize;
    while let Some((source_dir, target_dir)) = stack.pop() {
        copied_entries = copied_entries
            .checked_add(1)
            .ok_or_else(|| FetchError::Io(io::Error::other("source override copy entry count overflow")))?;
        if copied_entries > MAX_OVERRIDE_COPY_ENTRIES {
            return Err(FetchError::Io(io::Error::other(format!(
                "source override copy exceeds {MAX_OVERRIDE_COPY_ENTRIES} entries"
            ))));
        }
        fs::create_dir_all(&target_dir)?;
        let mut entries = fs::read_dir(&source_dir)?.collect::<Result<Vec<_>, _>>()?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let source_path = entry.path();
            let target_path = target_dir.join(entry.file_name());
            let metadata = fs::symlink_metadata(&source_path)?;
            if metadata.is_dir() {
                stack.push((source_path, target_path));
            } else {
                copy_override_leaf(&source_path, &target_path, &metadata)?;
            }
        }
    }
    Ok(())
}

fn copy_override_leaf(source: &Path, target: &Path, metadata: &fs::Metadata) -> Result<(), FetchError> {
    if metadata.file_type().is_symlink() {
        copy_override_symlink(source, target)?;
        return Ok(());
    }
    if !metadata.is_file() {
        return Err(FetchError::Io(io::Error::other(format!(
            "source override payload contains unsupported file kind at {}",
            source.display()
        ))));
    }
    fs::copy(source, target)?;
    fs::set_permissions(target, metadata.permissions())?;
    Ok(())
}

#[cfg(unix)]
fn copy_override_symlink(source: &Path, target: &Path) -> Result<(), FetchError> {
    use std::os::unix::fs::symlink;
    let link_target = fs::read_link(source)?;
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    symlink(link_target, target)?;
    Ok(())
}

#[cfg(not(unix))]
fn copy_override_symlink(source: &Path, _target: &Path) -> Result<(), FetchError> {
    Err(FetchError::Io(io::Error::other(format!(
        "source override symlink cannot be materialized on this platform: {}",
        source.display()
    ))))
}

#[cfg(unix)]
fn set_executable_mode(path: &Path) -> Result<(), FetchError> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(UNIX_EXECUTABLE_FILE_MODE))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_executable_mode(_path: &Path) -> Result<(), FetchError> {
    Ok(())
}

// ── FetchBuildService ─────────────────────────────────────────────────

/// `BuildService` implementation for fetcher derivations.
///
/// When `do_build` receives a request with `command_args[0] ==
/// "builtin:fetchurl"`, it parses the fetch parameters from
/// `environment_vars`, downloads or extracts the resource into a temp
/// directory, ingests the result into castore, and returns a
/// `BuildResult` with the output node.
///
/// Non-fetch requests are rejected with an error — use
/// `DispatchBuildService` to route between fetch and sandbox services.
pub struct FetchBuildService {
    store: crunch_store::BuildServiceStore,
    source_overrides: Vec<FetchSourceOverride>,
    source_policy: FetchSourcePolicy,
}

impl FetchBuildService {
    #[must_use]
    pub fn new(store: crunch_store::BuildServiceStore) -> Self {
        Self {
            store,
            source_overrides: Vec::new(),
            source_policy: FetchSourcePolicy::AllowNetwork,
        }
    }

    pub fn with_source_overrides(mut self, source_overrides: Vec<FetchSourceOverride>) -> Self {
        assert!(source_overrides.len() <= MAX_SOURCE_OVERRIDES, "source override list exceeds fixed bound");
        self.source_overrides = source_overrides;
        self
    }

    pub fn with_source_policy(mut self, source_policy: FetchSourcePolicy) -> Self {
        self.source_policy = source_policy;
        self
    }
}

#[async_trait]
impl BuildService for FetchBuildService {
    async fn do_build(&self, request: BuildRequest) -> io::Result<BuildResult> {
        // Tiger Style: assert the request has at least one output.
        debug_assert!(!request.outputs.is_empty(), "fetch request must have at least one output");

        let primary_kind = parse_fetch_kind(&request).map_err(io::Error::other)?;
        let ordered_kinds = ordered_fetch_kinds(&request, &primary_kind).map_err(io::Error::other)?;
        let is_foreign_candidate_set =
            request.environment_vars.iter().any(|environment| environment.key == FOREIGN_FETCH_CANDIDATES_ENV);

        // Download to a temp directory (cleaned up on drop).
        let tmp = tempfile::tempdir().map_err(|e| io::Error::other(format!("creating fetch temp dir: {e}")))?;
        let out_path = tmp.path().join("output");
        let mut attempts = Vec::with_capacity(ordered_kinds.len());
        let mut selected_kind = None;
        let mut last_unavailable_error = None;
        for (index, kind) in ordered_kinds.iter().enumerate() {
            remove_fetch_output(&out_path).map_err(io::Error::other)?;
            let candidate = kind.candidate().to_string();
            if let Some(source_override) = source_override_for_kind(kind, &self.source_overrides) {
                info!(
                    url = %source_override.url,
                    source_state_blake3 = %source_override.source_state_blake3,
                    "materializing fetcher input from source state"
                );
                materialize_source_override(source_override, &out_path).map_err(io::Error::other)?;
                attempts.push(ForeignFetchAttempt {
                    index,
                    candidate,
                    classification: "selected-source-state".to_string(),
                });
                selected_kind = Some(kind.clone());
                break;
            }
            if self.source_policy == FetchSourcePolicy::RequireOverride {
                attempts.push(ForeignFetchAttempt {
                    index,
                    candidate,
                    classification: "unavailable-source-state".to_string(),
                });
                continue;
            }

            let kind_clone = kind.clone();
            let out_str = out_path.to_str().ok_or_else(|| io::Error::other("temp path not valid UTF-8"))?.to_string();
            let result = tokio::task::spawn_blocking(move || execute_fetch(&kind_clone, &out_str))
                .await
                .map_err(|error| io::Error::other(format!("fetch spawn_blocking: {error}")))?;
            match result {
                Ok(()) => {
                    attempts.push(ForeignFetchAttempt {
                        index,
                        candidate,
                        classification: "selected-network".to_string(),
                    });
                    selected_kind = Some(kind.clone());
                    break;
                }
                Err(error) if fetch_error_is_candidate_unavailable(&error) => {
                    attempts.push(ForeignFetchAttempt {
                        index,
                        candidate,
                        classification: "unavailable-network".to_string(),
                    });
                    last_unavailable_error = Some(error);
                }
                Err(error) => return Err(io::Error::other(error)),
            }
        }
        let kind = selected_kind.ok_or_else(|| match last_unavailable_error {
            Some(error) => io::Error::other(error),
            None if is_foreign_candidate_set => io::Error::other(format!(
                "offline source policy rejected all ordered builtin fetch candidates before network acquisition: {}",
                primary_kind.source_identity()
            )),
            None => io::Error::other(format!(
                "offline source policy rejected unmatched builtin fetch before network acquisition: {}",
                primary_kind.source_identity()
            )),
        })?;

        // Verify output was produced.
        if !out_path.exists() {
            return Err(io::Error::other(format!("fetcher did not produce output for {kind:?}")));
        }

        // Ingest into castore.
        let node = self
            .store
            .ingest_host_path(&out_path)
            .await
            .map_err(|error| io::Error::other(format!("ingesting fetch output: {error}")))?;

        // One BuildOutput per requested output.
        // Fetcher outputs have no self-references → empty needles.
        let outputs: Vec<BuildOutput> = request
            .outputs
            .iter()
            .map(|_| BuildOutput {
                node: node.clone(),
                output_needles: BTreeSet::new(),
            })
            .collect();

        // Tiger Style: post-condition.
        debug_assert_eq!(
            outputs.len(),
            request.outputs.len(),
            "must produce exactly one BuildOutput per requested output"
        );

        let log = if is_foreign_candidate_set {
            foreign_fetch_attempt_log(attempts).map_err(io::Error::other)?
        } else {
            None
        };
        Ok(BuildResult { outputs, log })
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use bytes::Bytes;
    use snix_build::buildservice::EnvVar;
    use snix_castore::Node;
    use snix_castore::blobservice::MemoryBlobService;
    use snix_castore::directoryservice::RedbDirectoryService;
    use snix_castore::directoryservice::RedbDirectoryServiceConfig;

    use super::*;

    fn tmp_ds() -> RedbDirectoryService {
        RedbDirectoryService::new_temporary("test".to_string(), RedbDirectoryServiceConfig::default()).unwrap()
    }

    fn test_fetch_service(
        blob_service: MemoryBlobService,
        directory_service: RedbDirectoryService,
    ) -> FetchBuildService {
        let parts = crate::test_support::pipeline_store_parts(blob_service, directory_service);
        FetchBuildService::new(parts.build_service_store)
    }

    fn env(key: &str, value: &str) -> EnvVar {
        EnvVar {
            key: key.to_string(),
            value: Bytes::from(value.as_bytes().to_vec()),
        }
    }

    fn fetch_request(env_vars: Vec<EnvVar>) -> BuildRequest {
        BuildRequest {
            command_args: vec![FETCH_BUILDER.to_string()],
            outputs: vec![PathBuf::from("nix/store/abc-test")],
            environment_vars: env_vars,
            ..BuildRequest::default()
        }
    }

    fn sandbox_request() -> BuildRequest {
        BuildRequest {
            command_args: vec!["/bin/sh".to_string(), "-c".to_string(), "echo hi".to_string()],
            outputs: vec![PathBuf::from("nix/store/abc-test")],
            ..BuildRequest::default()
        }
    }

    // ── is_fetch_request ───────────────────────────────────────────

    #[test]
    fn is_fetch_request_true_for_builtin() {
        let req = fetch_request(vec![env("url", "https://example.com/f.txt")]);
        assert!(is_fetch_request(&req));
    }

    #[test]
    fn is_fetch_request_false_for_sandbox() {
        let req = sandbox_request();
        assert!(!is_fetch_request(&req));
    }

    #[test]
    fn is_fetch_request_false_for_empty_args() {
        let req = BuildRequest::default();
        assert!(!is_fetch_request(&req));
    }

    // ── parse_fetch_kind ───────────────────────────────────────────

    #[test]
    fn parse_file_fetch() {
        let req = fetch_request(vec![env("url", "https://example.com/file.txt")]);
        let kind = parse_fetch_kind(&req).unwrap();
        assert!(matches!(kind, FetchKind::File { .. }));
        if let FetchKind::File { url } = &kind {
            assert_eq!(url.as_str(), "https://example.com/file.txt");
        }
    }

    #[test]
    fn parse_tarball_fetch() {
        let req = fetch_request(vec![env("url", "https://example.com/src.tar.gz"), env("unpack", "1")]);
        let kind = parse_fetch_kind(&req).unwrap();
        assert!(matches!(kind, FetchKind::Tarball { .. }));
    }

    #[test]
    fn parse_executable_fetch() {
        let req = fetch_request(vec![env("url", "https://example.com/run"), env("executable", "1")]);
        let kind = parse_fetch_kind(&req).unwrap();
        assert!(matches!(kind, FetchKind::Executable { .. }));
    }

    #[test]
    fn parse_git_fetch() {
        let req = fetch_request(vec![
            env("url", "https://github.com/user/repo.git"),
            env("type", "git"),
            env("rev", "abc123def456"),
        ]);
        let kind = parse_fetch_kind(&req).unwrap();
        assert!(matches!(kind, FetchKind::Git { .. }));
        if let FetchKind::Git { url, rev } = &kind {
            assert_eq!(url, "https://github.com/user/repo.git");
            assert_eq!(rev, "abc123def456");
        }
    }

    #[test]
    fn parse_rejects_non_fetcher() {
        let req = sandbox_request();
        let err = parse_fetch_kind(&req).unwrap_err();
        assert!(matches!(err, FetchError::NotFetcher(_)));
    }

    #[test]
    fn parse_missing_url() {
        let req = fetch_request(vec![]);
        let err = parse_fetch_kind(&req).unwrap_err();
        assert!(matches!(err, FetchError::MissingUrl));
    }

    #[test]
    fn parse_git_missing_rev() {
        let req = fetch_request(vec![env("url", "https://github.com/user/repo.git"), env("type", "git")]);
        let err = parse_fetch_kind(&req).unwrap_err();
        assert!(matches!(err, FetchError::MissingGitRev));
    }

    #[test]
    fn parse_invalid_url() {
        let req = fetch_request(vec![env("url", "not a url")]);
        let err = parse_fetch_kind(&req).unwrap_err();
        assert!(matches!(err, FetchError::InvalidUrl(_)));
    }

    #[test]
    fn parse_unpack_takes_priority_over_executable() {
        // If both flags are set, unpack wins (tarball mode).
        let req = fetch_request(vec![
            env("url", "https://example.com/f.tar.gz"),
            env("unpack", "1"),
            env("executable", "1"),
        ]);
        let kind = parse_fetch_kind(&req).unwrap();
        assert!(matches!(kind, FetchKind::Tarball { .. }));
    }

    #[test]
    fn ordered_foreign_candidates_preserve_declared_order_and_kind() {
        let candidates = serde_json::to_string(&vec![
            "https://example.com/source".to_string(),
            "https://mirror.example.com/source".to_string(),
        ])
        .unwrap();
        let request = fetch_request(vec![
            env("url", "https://example.com/source"),
            env("executable", "1"),
            env(FOREIGN_FETCH_CANDIDATES_ENV, &candidates),
        ]);
        let primary = parse_fetch_kind(&request).unwrap();

        let kinds = ordered_fetch_kinds(&request, &primary).unwrap();

        assert_eq!(kinds.len(), 2);
        assert!(kinds.iter().all(|kind| matches!(kind, FetchKind::Executable { .. })));
        assert_eq!(kinds[0].candidate(), "https://example.com/source");
        assert_eq!(kinds[1].candidate(), "https://mirror.example.com/source");
    }

    #[test]
    fn ordered_foreign_candidates_reject_malformed_duplicate_and_stale_primary() {
        let malformed = fetch_request(vec![
            env("url", "https://example.com/source"),
            env(FOREIGN_FETCH_CANDIDATES_ENV, "not-json"),
        ]);
        let primary = parse_fetch_kind(&malformed).unwrap();
        assert!(matches!(ordered_fetch_kinds(&malformed, &primary), Err(FetchError::InvalidCandidateList(_))));

        let duplicate_json = serde_json::to_string(&vec![
            "https://example.com/source".to_string(),
            "https://example.com/source".to_string(),
        ])
        .unwrap();
        let duplicate = fetch_request(vec![
            env("url", "https://example.com/source"),
            env(FOREIGN_FETCH_CANDIDATES_ENV, &duplicate_json),
        ]);
        let primary = parse_fetch_kind(&duplicate).unwrap();
        assert!(matches!(ordered_fetch_kinds(&duplicate, &primary), Err(FetchError::InvalidCandidateList(_))));

        let stale_json = serde_json::to_string(&vec!["https://mirror.example.com/source".to_string()]).unwrap();
        let stale = fetch_request(vec![
            env("url", "https://example.com/source"),
            env(FOREIGN_FETCH_CANDIDATES_ENV, &stale_json),
        ]);
        let primary = parse_fetch_kind(&stale).unwrap();
        assert!(matches!(ordered_fetch_kinds(&stale, &primary), Err(FetchError::InvalidCandidateList(_))));
    }

    // ── FetchBuildService::do_build ────────────────────────────────

    #[tokio::test]
    async fn do_build_fetches_local_file() {
        const LOCAL_PAYLOAD: &[u8] = b"hello fetch service";
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let svc = test_fetch_service(bs, ds);

        // Write a local file to serve via file:// URL.
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), LOCAL_PAYLOAD).unwrap();
        let url = format!("file://{}", tmp.path().display());

        let req = fetch_request(vec![env("url", &url)]);
        let result = svc.do_build(req).await.unwrap();

        assert_eq!(result.outputs.len(), 1);
        assert!(matches!(result.outputs[0].node, Node::File { .. }));
        assert!(result.outputs[0].output_needles.is_empty());
        if let Node::File { size, .. } = &result.outputs[0].node {
            assert_eq!(*size, u64::try_from(LOCAL_PAYLOAD.len()).unwrap());
        }
    }

    #[tokio::test]
    async fn do_build_uses_matching_source_override_without_network() {
        const OFFLINE_PAYLOAD: &[u8] = b"offline payload";
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let payload = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(payload.path(), OFFLINE_PAYLOAD).unwrap();
        let source_override = FetchSourceOverride {
            url: "https://example.invalid/source.txt".to_string(),
            kind: FetchSourceOverrideKind::File,
            rev: None,
            payload_path: payload.path().to_path_buf(),
            source_state_blake3: blake3::hash(b"test-source-state").to_hex().to_string(),
        };
        let svc = test_fetch_service(bs, ds).with_source_overrides(vec![source_override]);

        let req = fetch_request(vec![env("url", "https://example.invalid/source.txt")]);
        let result = svc.do_build(req).await.unwrap();

        assert_eq!(result.outputs.len(), 1);
        assert!(matches!(result.outputs[0].node, Node::File { .. }));
        assert!(result.outputs[0].output_needles.is_empty());
        if let Node::File { size, .. } = &result.outputs[0].node {
            assert_eq!(*size, u64::try_from(OFFLINE_PAYLOAD.len()).unwrap());
        }
    }

    #[tokio::test]
    async fn ordered_foreign_candidates_select_first_available_source_state() {
        const OFFLINE_PAYLOAD: &[u8] = b"mirror payload";
        let primary = "https://primary.example.invalid/source.txt";
        let mirror = "https://mirror.example.invalid/source.txt";
        let candidates = serde_json::to_string(&vec![primary.to_string(), mirror.to_string()]).unwrap();
        let payload = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(payload.path(), OFFLINE_PAYLOAD).unwrap();
        let source_override = FetchSourceOverride {
            url: mirror.to_string(),
            kind: FetchSourceOverrideKind::File,
            rev: None,
            payload_path: payload.path().to_path_buf(),
            source_state_blake3: blake3::hash(b"mirror-source-state").to_hex().to_string(),
        };
        let service = test_fetch_service(MemoryBlobService::default(), tmp_ds())
            .with_source_overrides(vec![source_override])
            .with_source_policy(FetchSourcePolicy::RequireOverride);
        let request = fetch_request(vec![env("url", primary), env(FOREIGN_FETCH_CANDIDATES_ENV, &candidates)]);

        let result = service.do_build(request).await.unwrap();
        let log: ForeignFetchAttemptLog = serde_json::from_str(result.log.as_deref().unwrap()).unwrap();

        assert_eq!(log.schema, FOREIGN_FETCH_ATTEMPT_LOG_SCHEMA);
        assert_eq!(log.attempts.len(), 2);
        assert_eq!(log.attempts[0].classification, "unavailable-source-state");
        assert_eq!(log.attempts[1].classification, "selected-source-state");
        assert_eq!(log.attempts[1].candidate, mirror);
    }

    #[tokio::test]
    async fn ordered_foreign_candidates_report_exhaustion_before_network() {
        let primary = "https://primary.example.invalid/source.txt";
        let mirror = "https://mirror.example.invalid/source.txt";
        let candidates = serde_json::to_string(&vec![primary.to_string(), mirror.to_string()]).unwrap();
        let service = test_fetch_service(MemoryBlobService::default(), tmp_ds())
            .with_source_policy(FetchSourcePolicy::RequireOverride);
        let request = fetch_request(vec![env("url", primary), env(FOREIGN_FETCH_CANDIDATES_ENV, &candidates)]);

        let error = service.do_build(request).await.unwrap_err();
        let message = error.to_string();

        assert!(message.contains("rejected all ordered builtin fetch candidates"));
        assert!(message.contains("primary.example.invalid/source.txt"));
        assert!(!message.contains("mirror payload"));
    }

    #[tokio::test]
    async fn required_source_override_rejects_unmatched_fetch_before_network() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let svc = test_fetch_service(bs, ds).with_source_policy(FetchSourcePolicy::RequireOverride);
        let req = fetch_request(vec![env("url", "https://example.invalid/missing.txt")]);

        let error = svc.do_build(req).await.unwrap_err();

        assert!(error.to_string().contains("offline source policy rejected unmatched builtin fetch"));
        assert!(error.to_string().contains("example.invalid/missing.txt"));
    }

    #[test]
    fn source_override_requires_matching_git_revision() {
        const REQUESTED_REV: &str = "1111111111111111111111111111111111111111";
        const OTHER_REV: &str = "2222222222222222222222222222222222222222";
        let source_override = FetchSourceOverride {
            url: "https://example.invalid/repo.git".to_string(),
            kind: FetchSourceOverrideKind::Git,
            rev: Some(REQUESTED_REV.to_string()),
            payload_path: PathBuf::from("/tmp/source-state"),
            source_state_blake3: blake3::hash(b"test-source-state").to_hex().to_string(),
        };
        let matching = FetchKind::Git {
            url: "https://example.invalid/repo.git".to_string(),
            rev: REQUESTED_REV.to_string(),
        };
        let wrong_rev = FetchKind::Git {
            url: "https://example.invalid/repo.git".to_string(),
            rev: OTHER_REV.to_string(),
        };

        assert!(source_override.matches(&matching));
        assert!(!source_override.matches(&wrong_rev));
    }

    #[tokio::test]
    async fn do_build_fetches_tarball() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let svc = test_fetch_service(bs, ds);

        // Create a tarball with a single file.
        let tmp_src = tempfile::tempdir().unwrap();
        let inner = tmp_src.path().join("project-v1");
        std::fs::create_dir(&inner).unwrap();
        std::fs::write(inner.join("hello.txt"), "hello world").unwrap();

        let tar_file = tempfile::NamedTempFile::new().unwrap();
        {
            let gz = flate2::write::GzEncoder::new(
                std::fs::File::create(tar_file.path()).unwrap(),
                flate2::Compression::fast(),
            );
            let mut builder = tar::Builder::new(gz);
            builder.append_dir_all("project-v1", &inner).unwrap();
            builder.into_inner().unwrap().finish().unwrap();
        }

        let url = format!("file://{}", tar_file.path().display());
        let req = fetch_request(vec![env("url", &url), env("unpack", "1")]);
        let result = svc.do_build(req).await.unwrap();

        assert_eq!(result.outputs.len(), 1);
        // Tarball extraction produces a directory node.
        assert!(matches!(result.outputs[0].node, Node::Directory { .. }));
    }

    #[tokio::test]
    async fn do_build_rejects_non_fetch() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let svc = test_fetch_service(bs, ds);

        let req = sandbox_request();
        let err = svc.do_build(req).await.unwrap_err();
        assert!(err.to_string().contains("not a builtin fetcher"));
    }

    #[tokio::test]
    async fn do_build_rejects_missing_url() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let svc = test_fetch_service(bs, ds);

        let req = fetch_request(vec![]);
        let err = svc.do_build(req).await.unwrap_err();
        assert!(err.to_string().contains("url"));
    }

    #[tokio::test]
    async fn do_build_executable_sets_mode() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let svc = test_fetch_service(bs, ds);

        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), b"#!/bin/sh\necho hi").unwrap();
        let url = format!("file://{}", tmp.path().display());

        let req = fetch_request(vec![env("url", &url), env("executable", "1")]);
        let result = svc.do_build(req).await.unwrap();

        assert_eq!(result.outputs.len(), 1);
        if let Node::File { executable, .. } = &result.outputs[0].node {
            // After chmod 755, ingest_path should detect it as executable.
            assert!(*executable);
        } else {
            panic!("expected File node");
        }
    }
}
