//! FetchBuildService: `BuildService` for fetcher derivations.
//!
//! Wraps the download/extraction logic from `fetcher.rs` behind the
//! `BuildService` trait. Downloads resources, ingests them into castore,
//! and returns a `BuildResult` with the output node.
//!
//! Does NOT verify fixed-output hashes, persist PathInfo, or export to
//! disk — those stay in the shared `finish_build` path.

use std::collections::BTreeSet;
use std::io;

use async_trait::async_trait;
use snix_build::buildservice::BuildOutput;
use snix_build::buildservice::BuildRequest;
use snix_build::buildservice::BuildResult;
use snix_build::buildservice::BuildService;
use snix_castore::blobservice::BlobService;
use snix_castore::directoryservice::DirectoryService;
use snix_castore::import::fs::ingest_path;
use tracing::info;
use url::Url;

use crate::fetcher::FetchError;
use crate::fetcher::{self};

/// The builder string that identifies builtin fetcher derivations.
pub const FETCH_BUILDER: &str = "builtin:fetchurl";

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
        return Err(FetchError::NotAFetcher(builder.to_string()));
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
                std::fs::set_permissions(out_path, std::fs::Permissions::from_mode(0o755))?;
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
pub struct FetchBuildService<BS, DS> {
    blob_service: BS,
    directory_service: DS,
}

impl<BS, DS> FetchBuildService<BS, DS> {
    pub fn new(blob_service: BS, directory_service: DS) -> Self {
        Self {
            blob_service,
            directory_service,
        }
    }
}

#[async_trait]
impl<BS, DS> BuildService for FetchBuildService<BS, DS>
where
    BS: BlobService + Clone + Send + Sync + 'static,
    DS: DirectoryService + Clone + Send + Sync + 'static,
{
    async fn do_build(&self, request: BuildRequest) -> io::Result<BuildResult> {
        // Tiger Style: assert the request has at least one output.
        debug_assert!(!request.outputs.is_empty(), "fetch request must have at least one output");

        let kind = parse_fetch_kind(&request).map_err(io::Error::other)?;

        // Download to a temp directory (cleaned up on drop).
        let tmp = tempfile::tempdir().map_err(|e| io::Error::other(format!("creating fetch temp dir: {e}")))?;
        let out_path = tmp.path().join("output");
        let out_str = out_path.to_str().ok_or_else(|| io::Error::other("temp path not valid UTF-8"))?.to_string();

        // Blocking download on a dedicated thread.
        let kind_clone = kind.clone();
        tokio::task::spawn_blocking(move || execute_fetch(&kind_clone, &out_str))
            .await
            .map_err(|e| io::Error::other(format!("fetch spawn_blocking: {e}")))?
            .map_err(io::Error::other)?;

        // Verify output was produced.
        if !out_path.exists() {
            return Err(io::Error::other(format!("fetcher did not produce output for {kind:?}")));
        }

        // Ingest into castore.
        let node =
            ingest_path::<_, _, _, &[u8]>(self.blob_service.clone(), self.directory_service.clone(), &out_path, None)
                .await
                .map_err(|e| io::Error::other(format!("ingesting fetch output: {e}")))?;

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

        Ok(BuildResult { outputs, log: None })
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
        assert!(matches!(err, FetchError::NotAFetcher(_)));
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

    // ── FetchBuildService::do_build ────────────────────────────────

    #[tokio::test]
    async fn do_build_fetches_local_file() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let svc = FetchBuildService::new(bs, ds);

        // Write a local file to serve via file:// URL.
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), b"hello fetch service").unwrap();
        let url = format!("file://{}", tmp.path().display());

        let req = fetch_request(vec![env("url", &url)]);
        let result = svc.do_build(req).await.unwrap();

        assert_eq!(result.outputs.len(), 1);
        assert!(matches!(result.outputs[0].node, Node::File { .. }));
        assert!(result.outputs[0].output_needles.is_empty());
        if let Node::File { size, .. } = &result.outputs[0].node {
            assert_eq!(*size, 19); // "hello fetch service" = 19 bytes
        }
    }

    #[tokio::test]
    async fn do_build_fetches_tarball() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let svc = FetchBuildService::new(bs, ds);

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
        let svc = FetchBuildService::new(bs, ds);

        let req = sandbox_request();
        let err = svc.do_build(req).await.unwrap_err();
        assert!(err.to_string().contains("not a builtin fetcher"));
    }

    #[tokio::test]
    async fn do_build_rejects_missing_url() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let svc = FetchBuildService::new(bs, ds);

        let req = fetch_request(vec![]);
        let err = svc.do_build(req).await.unwrap_err();
        assert!(err.to_string().contains("url"));
    }

    #[tokio::test]
    async fn do_build_executable_sets_mode() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let svc = FetchBuildService::new(bs, ds);

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
