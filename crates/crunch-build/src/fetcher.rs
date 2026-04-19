//! Builtin fetcher execution: download, extract, and verify.
//!
//! When a derivation has `builder = "builtin:fetchurl"`, the build
//! orchestrator bypasses the sandbox and calls into this module.
//! Adapted from snix-redox's `fetchers.rs`.

use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::{self};
use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

use nix_compat::derivation::Derivation;
use nix_compat::nixhash::CAHash;
use nix_compat::nixhash::HashAlgo;
use nix_compat::nixhash::NixHash;
use tracing::info;
use url::Url;

/// Maximum download size: 4 GiB. Prevents unbounded memory/disk use
/// from a misbehaving or malicious server.
const MAX_DOWNLOAD_BYTES: u64 = 4_294_967_296;

/// Maximum number of entries to extract from a tar archive.
/// Prevents zip-bomb style attacks and accidental extraction of
/// enormous archives.
const MAX_TAR_ENTRIES: u32 = 500_000;

/// Maximum number of symlink expansions while validating existing paths in the
/// extraction tree. Bounds attacker-controlled symlink cycles.
const MAX_TAR_SYMLINK_EXPANSIONS: u32 = 64;

// ── Fetch description ──────────────────────────────────────────────────

/// What kind of fetch to perform, parsed from the derivation environment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fetch {
    /// Download a single file. Hash is over raw bytes.
    Url { url: Url, exp_hash: Option<NixHash> },
    /// Download a tarball, decompress, extract, strip top-level dir.
    /// Hash is NAR of the unpacked tree.
    Tarball { url: Url, exp_nar_sha256: Option<[u8; 32]> },
    /// Download a NAR archive and extract it.
    Nar { url: Url, hash: NixHash },
    /// Download a file and mark it executable.
    Executable { url: Url, hash: NixHash },
    /// Clone a git repository at a specific revision.
    Git {
        url: String,
        rev: String,
        exp_hash: Option<NixHash>,
    },
}

/// Errors specific to fetch operations.
#[derive(Debug, thiserror::Error)]
pub enum FetchError {
    #[error("not a builtin fetcher: builder = {0:?}")]
    NotFetcher(String),

    #[error("fetcher derivation missing 'url' in environment")]
    MissingUrl,

    #[error("invalid URL: {0}")]
    InvalidUrl(String),

    #[error("fetcher derivation has no fixed-output hash")]
    NotFixedOutput,

    #[error("fetchGit: missing 'rev' in environment")]
    MissingGitRev,

    #[error("HTTP fetch failed for {url}: {reason}")]
    HttpError { url: String, reason: String },

    #[error("tar extraction failed: {0}")]
    TarError(String),

    #[error("git fetch failed: {0}")]
    GitError(String),

    #[error("decompression failed: {0}")]
    DecompressError(String),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("hash mismatch for {name}: expected {expected}, got {actual}")]
    HashMismatch {
        name: String,
        expected: String,
        actual: String,
    },
}

// ── Parse derivation → Fetch ───────────────────────────────────────────

/// Returns `true` if this derivation uses a builtin fetcher.
pub fn is_builtin_fetcher(derivation: &Derivation) -> bool {
    derivation.builder.starts_with("builtin:")
}

/// Parse a derivation's environment into a typed `Fetch`.
///
/// Expects `builder = "builtin:fetchurl"` and the derivation's env to
/// contain `url` (plus optional `unpack`, `type`, `rev`, `executable`).
pub fn parse_fetch(derivation: &Derivation) -> Result<Fetch, FetchError> {
    assert!(!derivation.builder.is_empty(), "derivation builder must not be empty");
    assert!(!derivation.outputs.is_empty(), "derivation must have outputs");
    if !is_builtin_fetcher(derivation) {
        return Err(FetchError::NotFetcher(derivation.builder.clone()));
    }

    let env = &derivation.environment;

    // Git fetch: env.type == "git"
    let fetch_type = env_str(env, "type");
    if fetch_type.as_deref() == Some("git") {
        let url = env_str(env, "url").ok_or(FetchError::MissingUrl)?;
        let rev = env_str(env, "rev").ok_or(FetchError::MissingGitRev)?;
        let exp_hash = extract_expected_hash(derivation);
        return Ok(Fetch::Git { url, rev, exp_hash });
    }

    let url_str = env_str(env, "url").ok_or(FetchError::MissingUrl)?;
    let url = Url::parse(&url_str).map_err(|e| FetchError::InvalidUrl(format!("{url_str}: {e}")))?;

    let should_unpack = env.get("unpack").is_some_and(|v| v == "1");
    let is_executable = env.get("executable").is_some_and(|v| v == "1");

    // Determine fetch variant from derivation's ca_hash and flags.
    let ca_hash = derivation.outputs.get("out").and_then(|o| o.ca_hash.as_ref());

    match ca_hash {
        Some(CAHash::Flat(hash)) if is_executable => Ok(Fetch::Executable {
            url,
            hash: hash.clone(),
        }),
        Some(CAHash::Flat(hash)) => Ok(Fetch::Url {
            url,
            exp_hash: Some(hash.clone()),
        }),
        Some(CAHash::Nar(hash)) if should_unpack => Ok(Fetch::Tarball {
            url,
            exp_nar_sha256: expected_nar_sha256(hash),
        }),
        Some(CAHash::Nar(hash)) => Ok(Fetch::Nar {
            url,
            hash: hash.clone(),
        }),
        None => {
            // No ca_hash — treat as plain URL fetch without hash verification.
            if should_unpack {
                Ok(Fetch::Tarball {
                    url,
                    exp_nar_sha256: None,
                })
            } else {
                Ok(Fetch::Url { url, exp_hash: None })
            }
        }
        Some(CAHash::Text(_)) => Err(FetchError::NotFixedOutput),
    }
}

fn env_str(env: &std::collections::BTreeMap<String, bstr::BString>, key: &str) -> Option<String> {
    env.get(key).and_then(|v| String::from_utf8(Vec::from(v.clone())).ok())
}

fn expected_nar_sha256(hash: &NixHash) -> Option<[u8; 32]> {
    match hash {
        NixHash::Sha256(d) => Some(*d),
        _ => None,
    }
}

fn extract_expected_hash(derivation: &Derivation) -> Option<NixHash> {
    derivation.outputs.get("out").and_then(|o| o.ca_hash.as_ref()).and_then(|ca| match ca {
        CAHash::Flat(h) | CAHash::Nar(h) => Some(h.clone()),
        CAHash::Text(_) => None,
    })
}

// ── Fetch execution ────────────────────────────────────────────────────

/// Execute a fetch: download the resource and write it to `out_path`.
///
/// This is a blocking (sync) function — call from `spawn_blocking`.
pub fn fetch_to_store(fetch: &Fetch, out_path: &str) -> Result<(), FetchError> {
    assert!(!out_path.is_empty(), "output path must not be empty");
    debug_assert!(!Path::new(out_path).as_os_str().is_empty(), "output path must be valid");
    if let Some(parent) = Path::new(out_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    match fetch {
        Fetch::Url { url, .. } => {
            info!(url = %url, "fetching file");
            fetch_flat(url.as_str(), out_path)?;
        }
        Fetch::Tarball { url, .. } => {
            info!(url = %url, "fetching tarball");
            fetch_and_unpack(url.as_str(), out_path)?;
        }
        Fetch::Nar { url, .. } => {
            info!(url = %url, "fetching NAR");
            fetch_flat(url.as_str(), out_path)?;
        }
        Fetch::Executable { url, .. } => {
            info!(url = %url, "fetching executable");
            fetch_flat(url.as_str(), out_path)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(out_path, std::fs::Permissions::from_mode(0o755))?;
            }
        }
        Fetch::Git { url, rev, .. } => {
            info!(url = %url, rev = %rev, "fetching git");
            fetch_git(url, rev, out_path)?;
        }
    }

    Ok(())
}

// ── Hash verification ──────────────────────────────────────────────────

/// Verify the fetched output's hash against the expected hash from the
/// `Fetch` variant.
///
/// - Flat fetches: hash the raw file bytes.
/// - Tarball/NAR/Git: hash the NAR serialization of the output tree.
///
/// `nar_hash_fn` is called for recursive (NAR) hashes — the caller
/// provides the NAR computation since it requires async castore services.
/// For flat hashes, this function reads the file directly.
pub fn verify_flat_hash(out_path: &str, expected: &NixHash, name: &str) -> Result<(), FetchError> {
    use sha2::Digest;
    assert!(!out_path.is_empty(), "output path must not be empty");
    assert!(!name.is_empty(), "name must not be empty for hash verification");

    let content = std::fs::read(out_path)?;

    let actual_bytes: Vec<u8> = match expected.algo() {
        HashAlgo::Sha256 => {
            let h = sha2::Sha256::digest(&content);
            h.to_vec()
        }
        HashAlgo::Sha512 => {
            let h = sha2::Sha512::digest(&content);
            h.to_vec()
        }
        HashAlgo::Sha1 => {
            let h = sha1::Sha1::digest(&content);
            h.to_vec()
        }
        HashAlgo::Md5 => {
            let h = md5::Md5::digest(&content);
            h.to_vec()
        }
        HashAlgo::Blake3 => {
            let h = blake3::hash(&content);
            h.as_bytes().to_vec()
        }
    };

    if actual_bytes.as_slice() != expected.digest_as_bytes() {
        // Best-effort cleanup of bad output.
        if let Err(e) = std::fs::remove_file(out_path) {
            tracing::debug!(path = %out_path, error = %e, "cleanup after hash mismatch: remove_file failed");
        }
        if let Err(e) = std::fs::remove_dir_all(out_path) {
            tracing::debug!(path = %out_path, error = %e, "cleanup after hash mismatch: remove_dir_all failed");
        }
        return Err(FetchError::HashMismatch {
            name: name.to_string(),
            expected: nix_hash_to_sri(expected),
            actual: format!("{}-{}", hash_algo_prefix(expected.algo()), data_encoding::BASE64.encode(&actual_bytes),),
        });
    }

    Ok(())
}

/// Format a NixHash as an SRI string (e.g., "sha256-base64...").
pub fn nix_hash_to_sri(hash: &NixHash) -> String {
    format!("{}-{}", hash_algo_prefix(hash.algo()), data_encoding::BASE64.encode(hash.digest_as_bytes()),)
}

fn hash_algo_prefix(algo: HashAlgo) -> &'static str {
    match algo {
        HashAlgo::Md5 => "md5",
        HashAlgo::Sha1 => "sha1",
        HashAlgo::Sha256 => "sha256",
        HashAlgo::Sha512 => "sha512",
        HashAlgo::Blake3 => "blake3",
    }
}

// ── Download helpers (sync, ureq) ──────────────────────────────────────

/// Connect timeout for HTTP fetches (seconds).
const FETCH_CONNECT_TIMEOUT_SECS: u64 = 60;
/// Total read timeout for HTTP fetches (seconds).
const FETCH_READ_TIMEOUT_SECS: u64 = 600;

fn fetch_agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(std::time::Duration::from_secs(FETCH_READ_TIMEOUT_SECS)))
        .timeout_connect(Some(std::time::Duration::from_secs(FETCH_CONNECT_TIMEOUT_SECS)))
        .build()
        .new_agent()
}

/// Download a URL and write the raw content to a file.
/// Tiger Style: bounded read — stops at MAX_DOWNLOAD_BYTES.
#[allow(tigerstyle::ambiguous_params)] // url vs filesystem output path: distinct domains
pub(crate) fn fetch_flat(url: &str, out: &str) -> Result<(), FetchError> {
    let reader: Box<dyn Read + Send> = open_url_reader(url)?;
    let mut bounded_reader = reader.take(MAX_DOWNLOAD_BYTES);
    let mut file = std::fs::File::create(out)?;
    let bytes_written = io::copy(&mut bounded_reader, &mut file)?;
    debug_assert!(bytes_written <= MAX_DOWNLOAD_BYTES);
    Ok(())
}

/// Download a tarball, decompress, and extract to a directory.
#[allow(tigerstyle::ambiguous_params)] // url vs filesystem output path: distinct domains
pub fn fetch_and_unpack(url: &str, out: &str) -> Result<(), FetchError> {
    let reader = open_url_reader(url)?;
    let decompressed = decompress_reader(url, reader)?;
    extract_tar(decompressed, out)?;
    Ok(())
}

/// Open a reader for a URL. Supports http(s):// and file:// schemes.
fn open_url_reader(url: &str) -> Result<Box<dyn Read + Send>, FetchError> {
    if let Some(path) = url.strip_prefix("file://") {
        let file = std::fs::File::open(path).map_err(|e| FetchError::HttpError {
            url: url.to_string(),
            reason: format!("opening local file: {e}"),
        })?;
        Ok(Box::new(io::BufReader::new(file)))
    } else {
        let resp = fetch_agent().get(url).call().map_err(|e| FetchError::HttpError {
            url: url.to_string(),
            reason: e.to_string(),
        })?;
        Ok(Box::new(resp.into_body().into_reader()))
    }
}

// ── Decompression ──────────────────────────────────────────────────────

/// Select a decompressor based on URL suffix. Falls back to gzip magic
/// detection, then raw read.
pub fn decompress_reader(url: &str, reader: impl Read + Send + 'static) -> Result<Box<dyn Read + Send>, FetchError> {
    assert!(!url.is_empty(), "url must not be empty for decompression");
    debug_assert!(url.contains('.') || url.contains('/'), "url should contain a path separator or extension");
    if url.ends_with(".gz") || url.ends_with(".tgz") {
        return Ok(Box::new(flate2::read::GzDecoder::new(reader)));
    }
    if url.ends_with(".xz") || url.ends_with(".txz") {
        let mut input = io::BufReader::new(reader);
        let mut output = Vec::new();
        lzma_rs::xz_decompress(&mut input, &mut output).map_err(|e| FetchError::DecompressError(format!("xz: {e}")))?;
        return Ok(Box::new(io::Cursor::new(output)));
    }
    if url.ends_with(".bz2") || url.ends_with(".tbz2") {
        return Ok(Box::new(bzip2_rs::DecoderReader::new(reader)));
    }
    if url.ends_with(".zst") || url.ends_with(".zstd") {
        return Ok(Box::new(
            ruzstd::decoding::StreamingDecoder::new(reader)
                .map_err(|e| FetchError::DecompressError(format!("zstd: {e}")))?,
        ));
    }
    if url.ends_with(".tar") || url.ends_with(".nar") {
        return Ok(Box::new(reader));
    }
    // Unknown suffix — try gzip magic, fall back to raw.
    let mut compressed = Vec::new();
    io::BufReader::new(reader).read_to_end(&mut compressed)?;
    if compressed.len() >= 2 && compressed[0] == 0x1f && compressed[1] == 0x8b {
        return Ok(Box::new(flate2::read::GzDecoder::new(io::Cursor::new(compressed))));
    }
    Ok(Box::new(io::Cursor::new(compressed)))
}

// ── Tar extraction ─────────────────────────────────────────────────────

/// Extract a tar archive to a directory, stripping the top-level path
/// component (GitHub-style tarballs where all entries are under
/// `project-v1.0/`).
pub fn extract_tar<R: Read>(mut reader: R, out: &str) -> Result<(), FetchError> {
    assert!(!out.is_empty(), "output path must not be empty");
    let out_path = Path::new(out);
    assert!(!out_path.as_os_str().is_empty(), "output path must not be empty");
    std::fs::create_dir_all(out_path)?;

    let mut staged_tar = tempfile::tempfile()?;
    io::copy(&mut reader, &mut staged_tar)?;
    let prefix_to_strip = detect_common_tar_prefix(&mut staged_tar)?;
    staged_tar.seek(SeekFrom::Start(0))?;

    let mut archive = tar::Archive::new(staged_tar);
    archive.set_preserve_mtime(false);
    let mut entry_count: u32 = 0;
    let mut deferred_hardlinks: Vec<(std::path::PathBuf, std::path::PathBuf)> = Vec::new();

    for entry_result in archive.entries().map_err(|e| FetchError::TarError(e.to_string()))? {
        entry_count = entry_count.saturating_add(1);
        validate_tar_entry_count(entry_count)?;
        let mut entry = entry_result.map_err(|e| FetchError::TarError(e.to_string()))?;
        let entry_path = entry.path().map_err(|e| FetchError::TarError(e.to_string()))?.into_owned();
        validate_entry_not_absolute(&entry_path)?;
        extract_tar_entry(&mut entry, &entry_path, out_path, &prefix_to_strip, &mut deferred_hardlinks)?;
    }
    replay_deferred_hardlinks(&deferred_hardlinks)?;
    Ok(())
}

fn validate_tar_entry_count(entry_count: u32) -> Result<(), FetchError> {
    if entry_count > MAX_TAR_ENTRIES {
        return Err(FetchError::TarError(format!("tar archive exceeds {MAX_TAR_ENTRIES} entries")));
    }
    Ok(())
}

fn detect_common_tar_prefix<R: Read + Seek>(reader: &mut R) -> Result<Option<std::path::PathBuf>, FetchError> {
    assert!(reader.seek(SeekFrom::Start(0)).is_ok(), "reader must be seekable");
    reader.seek(SeekFrom::Start(0))?;
    let mut archive = tar::Archive::new(reader);
    archive.set_preserve_mtime(false);

    let mut common_prefix: Option<std::path::PathBuf> = None;
    let mut has_multi_component_entry = false;

    for entry_result in archive.entries().map_err(|e| FetchError::TarError(e.to_string()))? {
        let entry = entry_result.map_err(|e| FetchError::TarError(e.to_string()))?;
        let entry_path = entry.path().map_err(|e| FetchError::TarError(e.to_string()))?.into_owned();
        validate_entry_not_absolute(&entry_path)?;
        let mut components = entry_path.components();
        let Some(first_component) = components.next() else {
            continue;
        };
        let prefix_component = std::path::PathBuf::from(first_component.as_os_str());
        if let Some(existing_prefix) = &common_prefix {
            if *existing_prefix != prefix_component {
                return Ok(None);
            }
        } else {
            common_prefix = Some(prefix_component);
        }
        if components.next().is_some() {
            has_multi_component_entry = true;
        }
    }

    if has_multi_component_entry {
        debug_assert!(common_prefix.is_some(), "multi-component entries imply a prefix was found");
        return Ok(common_prefix);
    }
    Ok(None)
}

fn extract_tar_entry<R: Read>(
    entry: &mut tar::Entry<'_, R>,
    entry_path: &Path,
    out_path: &Path,
    prefix_to_strip: &Option<std::path::PathBuf>,
    deferred_hardlinks: &mut Vec<(std::path::PathBuf, std::path::PathBuf)>,
) -> Result<(), FetchError> {
    let relative = strip_tar_prefix(entry_path, prefix_to_strip);
    if relative.as_os_str().is_empty() || relative == Path::new(".") {
        return Ok(());
    }
    validate_relative_path(&relative)?;
    let entry_context = format!("tar entry path {}", relative.display());
    let resolved_dest = resolve_entry_destination_path(out_path, &relative, &entry_context)?;
    discard_shadowed_deferred_hardlinks(deferred_hardlinks, &resolved_dest);
    debug_assert!(resolved_dest.starts_with(out_path), "dest {resolved_dest:?} escapes out_path {out_path:?}");
    match entry.header().entry_type() {
        tar::EntryType::Regular | tar::EntryType::GNUSparse => write_regular_tar_entry(entry, &resolved_dest),
        tar::EntryType::Directory => prepare_directory_entry_dest(&resolved_dest),
        tar::EntryType::Symlink => create_tar_symlink(entry, &resolved_dest, out_path, prefix_to_strip, &relative),
        tar::EntryType::Link => {
            copy_tar_hardlink(entry, &resolved_dest, out_path, prefix_to_strip, &relative, deferred_hardlinks)
        }
        _ => Ok(()),
    }
}

fn write_regular_tar_entry<R: Read>(entry: &mut tar::Entry<'_, R>, dest: &Path) -> Result<(), FetchError> {
    prepare_non_directory_entry_dest(dest)?;
    let mut file = std::fs::File::create(dest)?;
    io::copy(entry, &mut file)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(mode) = entry.header().mode() {
            if let Err(e) = std::fs::set_permissions(dest, std::fs::Permissions::from_mode(mode)) {
                tracing::debug!(path = ?dest, error = %e, "set_permissions on tar entry failed");
            }
        }
    }
    Ok(())
}

fn create_tar_symlink<R: Read>(
    entry: &mut tar::Entry<'_, R>,
    dest: &Path,
    out_path: &Path,
    prefix_to_strip: &Option<std::path::PathBuf>,
    relative: &Path,
) -> Result<(), FetchError> {
    prepare_non_directory_entry_dest(dest)?;
    #[cfg(unix)]
    if let Some(target) = entry.link_name().map_err(|e| FetchError::TarError(e.to_string()))? {
        let target_path = target.as_ref();
        let normalized_target = normalize_tar_symlink_target(target_path, prefix_to_strip, relative)?;
        validate_symlink_target(&normalized_target, dest, relative, out_path)?;
        std::os::unix::fs::symlink(&normalized_target, dest)?;
    }
    Ok(())
}

fn copy_tar_hardlink<R: Read>(
    entry: &mut tar::Entry<'_, R>,
    dest: &Path,
    out_path: &Path,
    prefix_to_strip: &Option<std::path::PathBuf>,
    relative: &Path,
    deferred_hardlinks: &mut Vec<(std::path::PathBuf, std::path::PathBuf)>,
) -> Result<(), FetchError> {
    prepare_non_directory_entry_dest(dest)?;
    if let Some(link_target) = entry.link_name().map_err(|e| FetchError::TarError(e.to_string()))? {
        let stripped_target = match prefix_to_strip {
            Some(pfx) => link_target.strip_prefix(pfx.as_path()).unwrap_or(link_target.as_ref()).to_path_buf(),
            None => link_target.into_owned(),
        };
        let target_path = validate_hardlink_target(out_path, &stripped_target, relative)?;
        if target_path.exists() {
            std::fs::copy(&target_path, dest)?;
        } else {
            deferred_hardlinks.push((target_path, dest.to_path_buf()));
        }
    }
    Ok(())
}

fn discard_shadowed_deferred_hardlinks(
    deferred_hardlinks: &mut Vec<(std::path::PathBuf, std::path::PathBuf)>,
    dest_path: &Path,
) {
    assert!(!dest_path.as_os_str().is_empty(), "shadowed hardlink destination must not be empty");
    deferred_hardlinks.retain(|(_, pending_dest)| pending_dest != dest_path);
}

fn replay_deferred_hardlinks(
    deferred_hardlinks: &[(std::path::PathBuf, std::path::PathBuf)],
) -> Result<(), FetchError> {
    let deferred_count_u32 = u32::try_from(deferred_hardlinks.len())
        .map_err(|_| FetchError::TarError("too many deferred tar hardlinks".to_owned()))?;
    assert!(deferred_count_u32 <= MAX_TAR_ENTRIES, "deferred hardlinks exceed tar entry limit");
    if deferred_hardlinks.is_empty() {
        return Ok(());
    }

    let mut pending_hardlinks = deferred_hardlinks.to_vec();
    let mut rounds_remaining = deferred_count_u32;
    while !pending_hardlinks.is_empty() {
        if rounds_remaining == 0 {
            return Err(missing_deferred_hardlink_target_error(&pending_hardlinks[0].0));
        }
        pending_hardlinks = replay_deferred_hardlink_round(pending_hardlinks)?;
        rounds_remaining = rounds_remaining.saturating_sub(1);
    }
    Ok(())
}

fn replay_deferred_hardlink_round(
    pending_hardlinks: Vec<(std::path::PathBuf, std::path::PathBuf)>,
) -> Result<Vec<(std::path::PathBuf, std::path::PathBuf)>, FetchError> {
    assert!(!pending_hardlinks.is_empty(), "deferred hardlink round must have work");
    let pending_len = pending_hardlinks.len();
    let mut unresolved_hardlinks: Vec<(std::path::PathBuf, std::path::PathBuf)> = Vec::with_capacity(pending_len);
    let mut copied_count: u32 = 0;

    for (target_path, dest_path) in pending_hardlinks {
        if !target_path.exists() {
            unresolved_hardlinks.push((target_path, dest_path));
            continue;
        }
        copy_replayed_hardlink(&target_path, &dest_path)?;
        copied_count = copied_count.saturating_add(1);
    }

    assert!(unresolved_hardlinks.len() <= pending_len, "unresolved hardlinks grew during replay");
    if copied_count == 0 {
        return Err(missing_deferred_hardlink_target_error(&unresolved_hardlinks[0].0));
    }
    Ok(unresolved_hardlinks)
}

fn copy_replayed_hardlink(target_path: &Path, dest_path: &Path) -> Result<(), FetchError> {
    assert!(target_path.exists(), "replayed hardlink target must exist before copy");
    assert!(!dest_path.as_os_str().is_empty(), "replayed hardlink destination must not be empty");
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::copy(target_path, dest_path)?;
    Ok(())
}

fn missing_deferred_hardlink_target_error(target_path: &Path) -> FetchError {
    assert!(!target_path.as_os_str().is_empty(), "missing deferred hardlink target path must not be empty");
    FetchError::TarError(format!("tar hardlink target missing after extraction: {}", target_path.display()))
}

/// Strip the top-level directory prefix from a tar entry path.
fn strip_tar_prefix(entry_path: &Path, prefix: &Option<std::path::PathBuf>) -> std::path::PathBuf {
    match prefix {
        Some(pfx) => match entry_path.strip_prefix(pfx) {
            Ok(stripped) => stripped.to_path_buf(),
            Err(_) => entry_path.to_path_buf(),
        },
        None => entry_path.to_path_buf(),
    }
}

/// Reject tar entry paths that are absolute.
fn validate_entry_not_absolute(entry_path: &Path) -> Result<(), FetchError> {
    for component in entry_path.components() {
        if matches!(component, std::path::Component::RootDir | std::path::Component::Prefix(_)) {
            return Err(FetchError::TarError(format!("tar entry has absolute path: {}", entry_path.display())));
        }
    }
    Ok(())
}

/// Reject stripped tar entry paths with anything other than Normal components.
fn validate_relative_path(relative: &Path) -> Result<(), FetchError> {
    assert!(!relative.as_os_str().is_empty(), "relative path must not be empty");
    debug_assert!(!relative.has_root(), "validate_relative_path expects a relative path");
    for component in relative.components() {
        match component {
            std::path::Component::Normal(_) => {}
            std::path::Component::ParentDir => {
                return Err(FetchError::TarError(format!(
                    "tar entry contains '..' path traversal: {}",
                    relative.display()
                )));
            }
            _ => {
                return Err(FetchError::TarError(format!(
                    "tar entry has non-relative path component: {}",
                    relative.display()
                )));
            }
        }
    }
    Ok(())
}

/// Normalize a tar symlink target before materialization.
/// Absolute targets are rebased to the extracted tree root, then rewritten
/// as relative paths from the symlink's parent directory.
fn normalize_tar_symlink_target(
    target_path: &Path,
    prefix_to_strip: &Option<std::path::PathBuf>,
    relative: &Path,
) -> Result<PathBuf, FetchError> {
    if !target_path.has_root() {
        return Ok(target_path.to_path_buf());
    }
    let rooted_target = rooted_tar_symlink_target(target_path, prefix_to_strip, relative)?;
    let symlink_parent = relative.parent().unwrap_or(Path::new(""));
    Ok(relative_path_between(symlink_parent, &rooted_target))
}

fn rooted_tar_symlink_target(
    target_path: &Path,
    prefix_to_strip: &Option<std::path::PathBuf>,
    relative: &Path,
) -> Result<PathBuf, FetchError> {
    assert!(target_path.has_root(), "rooted tar symlink target must be absolute");
    let context = format!("tar symlink target {} -> {}", relative.display(), target_path.display());
    let target_without_root = strip_rooted_symlink_target(target_path, relative)?;
    let stripped_target = match prefix_to_strip {
        Some(prefix) => {
            target_without_root.strip_prefix(prefix.as_path()).unwrap_or(&target_without_root).to_path_buf()
        }
        None => target_without_root,
    };
    resolve_relative_within_root(Path::new(""), &stripped_target, &context)
}

fn strip_rooted_symlink_target(target_path: &Path, relative: &Path) -> Result<PathBuf, FetchError> {
    assert!(target_path.has_root(), "rooted symlink target must be absolute");
    assert!(!relative.as_os_str().is_empty(), "relative path must not be empty");
    let mut stripped = PathBuf::new();
    let mut has_root_component = false;
    for component in target_path.components() {
        match component {
            std::path::Component::RootDir | std::path::Component::Prefix(_) => has_root_component = true,
            std::path::Component::CurDir => {}
            std::path::Component::Normal(name) => stripped.push(name),
            std::path::Component::ParentDir => stripped.push(".."),
        }
    }
    if !has_root_component {
        return Err(FetchError::TarError(format!(
            "tar symlink target is not rooted: {} -> {}",
            relative.display(),
            target_path.display()
        )));
    }
    Ok(stripped)
}

fn relative_path_between(from: &Path, to: &Path) -> PathBuf {
    assert!(!from.as_os_str().is_empty(), "from path must not be empty");
    assert!(!to.as_os_str().is_empty(), "to path must not be empty");
    let from_components: Vec<std::ffi::OsString> = from
        .components()
        .filter_map(|component| match component {
            std::path::Component::Normal(name) => Some(name.to_os_string()),
            std::path::Component::CurDir
            | std::path::Component::Prefix(_)
            | std::path::Component::RootDir
            | std::path::Component::ParentDir => None,
        })
        .collect();
    let to_components: Vec<std::ffi::OsString> = to
        .components()
        .filter_map(|component| match component {
            std::path::Component::Normal(name) => Some(name.to_os_string()),
            std::path::Component::CurDir
            | std::path::Component::Prefix(_)
            | std::path::Component::RootDir
            | std::path::Component::ParentDir => None,
        })
        .collect();

    let mut shared_prefix_len: usize = 0;
    let shared_depth_max = from_components.len().min(to_components.len());
    while shared_prefix_len < shared_depth_max {
        if from_components[shared_prefix_len] != to_components[shared_prefix_len] {
            break;
        }
        shared_prefix_len = shared_prefix_len.saturating_add(1);
    }

    let mut relative = PathBuf::new();
    for _ in from_components.iter().skip(shared_prefix_len) {
        relative.push("..");
    }
    for component in to_components.iter().skip(shared_prefix_len) {
        relative.push(component);
    }
    if relative.as_os_str().is_empty() {
        relative.push(".");
    }
    relative
}

/// Validate that a symlink target stays within the output directory.
/// Relative targets are resolved against the symlink's physical parent path
/// inside the extracted tree.
fn validate_symlink_target(
    target_path: &Path,
    dest: &Path,
    relative: &Path,
    out_path: &Path,
) -> Result<(), FetchError> {
    assert!(!target_path.has_root(), "normalized tar symlink target must be relative");
    let base_relative = symlink_parent_relative(dest, out_path, relative)?;
    let context = format!("tar symlink target {} -> {}", relative.display(), target_path.display());
    let raw_target = base_relative.join(target_path);
    let _ = resolve_existing_path_contained(out_path, &raw_target, &context, MAX_TAR_SYMLINK_EXPANSIONS)?;
    Ok(())
}

fn symlink_parent_relative(dest: &Path, out_path: &Path, relative: &Path) -> Result<PathBuf, FetchError> {
    let Some(parent) = dest.parent() else {
        return Ok(PathBuf::new());
    };
    let stripped = parent.strip_prefix(out_path).map_err(|_| {
        FetchError::TarError(format!("tar symlink target base escapes output dir: {}", relative.display()))
    })?;
    Ok(stripped.to_path_buf())
}

/// Validate that a hardlink's stripped target stays within the output tree.
fn validate_hardlink_target(out_path: &Path, stripped_target: &Path, relative: &Path) -> Result<PathBuf, FetchError> {
    if stripped_target.has_root() {
        return Err(FetchError::TarError(format!(
            "tar hardlink has absolute target: {} -> {}",
            relative.display(),
            stripped_target.display()
        )));
    }
    let context = format!("tar hardlink target {} -> {}", relative.display(), stripped_target.display());
    resolve_contained_host_path(out_path, stripped_target, &context)
}

fn resolve_relative_within_root(
    base_relative: &Path,
    candidate_path: &Path,
    context: &str,
) -> Result<PathBuf, FetchError> {
    assert!(!context.is_empty(), "context must describe the operation");
    assert!(!candidate_path.as_os_str().is_empty(), "candidate path must not be empty");
    let mut resolved = PathBuf::new();
    for component in base_relative.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::Normal(name) => resolved.push(name),
            _ => {
                return Err(FetchError::TarError(format!(
                    "{context} has invalid base path: {}",
                    base_relative.display()
                )));
            }
        }
    }
    for component in candidate_path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::Normal(name) => resolved.push(name),
            std::path::Component::ParentDir => {
                if !resolved.pop() {
                    return Err(FetchError::TarError(format!(
                        "{context} escapes output dir: {}",
                        candidate_path.display()
                    )));
                }
            }
            std::path::Component::RootDir | std::path::Component::Prefix(_) => {
                return Err(FetchError::TarError(format!("{context} is absolute: {}", candidate_path.display())));
            }
        }
    }
    Ok(resolved)
}

fn resolve_entry_destination_path(out_path: &Path, relative: &Path, context: &str) -> Result<PathBuf, FetchError> {
    assert!(!relative.as_os_str().is_empty(), "entry destination path must not be empty");
    let Some(leaf_name) = relative.file_name() else {
        return Err(FetchError::TarError(format!("{context} has no final path component: {}", relative.display())));
    };
    let parent_relative = relative.parent().unwrap_or(Path::new(""));
    let resolved_parent =
        resolve_existing_path_contained(out_path, parent_relative, context, MAX_TAR_SYMLINK_EXPANSIONS)?;
    let dest = out_path.join(resolved_parent).join(leaf_name);
    assert!(dest.starts_with(out_path), "entry destination must stay under output dir");
    Ok(dest)
}

fn prepare_directory_entry_dest(dest: &Path) -> Result<(), FetchError> {
    assert!(!dest.as_os_str().is_empty(), "directory entry destination must not be empty");
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    remove_existing_non_directory_entry(dest)?;
    std::fs::create_dir_all(dest)?;
    Ok(())
}

fn prepare_non_directory_entry_dest(dest: &Path) -> Result<(), FetchError> {
    assert!(!dest.as_os_str().is_empty(), "non-directory entry destination must not be empty");
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    remove_existing_non_directory_entry(dest)
}

fn remove_existing_non_directory_entry(dest: &Path) -> Result<(), FetchError> {
    let existing_metadata = match std::fs::symlink_metadata(dest) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(err) => {
            return Err(FetchError::TarError(format!(
                "failed to inspect existing tar destination {}: {err}",
                dest.display()
            )));
        }
    };
    if existing_metadata.is_dir() {
        return Ok(());
    }
    std::fs::remove_file(dest)?;
    Ok(())
}

fn resolve_contained_host_path(
    out_path: &Path,
    candidate_relative: &Path,
    context: &str,
) -> Result<PathBuf, FetchError> {
    let resolved_relative =
        resolve_existing_path_contained(out_path, candidate_relative, context, MAX_TAR_SYMLINK_EXPANSIONS)?;
    Ok(out_path.join(resolved_relative))
}

fn resolve_existing_path_contained(
    out_path: &Path,
    candidate_relative: &Path,
    context: &str,
    remaining_symlink_expansions: u32,
) -> Result<PathBuf, FetchError> {
    assert!(!out_path.as_os_str().is_empty(), "out_path must not be empty");
    assert!(!context.is_empty(), "context must describe the operation");
    let mut work = candidate_relative.to_path_buf();
    let mut expansions_left = remaining_symlink_expansions;

    // Outer loop: restarts resolution when a symlink redirects the path.
    // Bounded by the expansion limit plus one (for the initial non-symlink pass).
    for _ in 0..=MAX_TAR_SYMLINK_EXPANSIONS {
        let components: Vec<std::path::Component<'_>> = work.components().collect();
        let mut current = PathBuf::new();
        let mut was_redirected = false;

        for (ci, component) in components.iter().enumerate() {
            match component {
                std::path::Component::CurDir => {}
                std::path::Component::Normal(name) => {
                    current.push(name);
                    let host_path = out_path.join(&current);
                    let metadata = match std::fs::symlink_metadata(&host_path) {
                        Ok(metadata) => metadata,
                        Err(err) if err.kind() == io::ErrorKind::NotFound => continue,
                        Err(err) => return Err(FetchError::TarError(format!("{context}: {err}"))),
                    };
                    if !metadata.file_type().is_symlink() {
                        continue;
                    }
                    if expansions_left == 0 {
                        return Err(FetchError::TarError(format!(
                            "{context} exceeds {MAX_TAR_SYMLINK_EXPANSIONS} symlink expansions"
                        )));
                    }
                    expansions_left = expansions_left.saturating_sub(1);
                    let link_target = std::fs::read_link(&host_path)?;
                    let base_relative = current.parent().unwrap_or(Path::new(""));
                    let mut next = base_relative.join(&link_target);
                    // Append remaining unprocessed components.
                    for remaining in &components[ci.saturating_add(1)..] {
                        next.push(remaining.as_os_str());
                    }
                    work = next;
                    was_redirected = true;
                    break;
                }
                std::path::Component::ParentDir => {
                    if !current.pop() {
                        return Err(FetchError::TarError(format!(
                            "{context} escapes output dir: {}",
                            candidate_relative.display()
                        )));
                    }
                }
                std::path::Component::RootDir | std::path::Component::Prefix(_) => {
                    return Err(FetchError::TarError(format!(
                        "{context} is absolute: {}",
                        candidate_relative.display()
                    )));
                }
            }
        }
        if !was_redirected {
            return Ok(current);
        }
    }
    Err(FetchError::TarError(format!(
        "{context} exceeds {MAX_TAR_SYMLINK_EXPANSIONS} symlink expansions"
    )))
}

// ── Git fetch ──────────────────────────────────────────────────────────

const MAX_GIT_PATH_COMPONENTS: u32 = 1024;

/// Clone a git repository and materialize a specific revision without using
/// any host `git` binary.
#[allow(tigerstyle::ambiguous_params)] // url, rev, out: three distinct domains (origin, commit, filesystem)
pub(crate) fn fetch_git(url: &str, rev: &str, out: &str) -> Result<(), FetchError> {
    let out_path = Path::new(out);
    assert!(!url.is_empty(), "git url must not be empty");
    assert!(!out_path.as_os_str().is_empty(), "output path must not be empty");
    assert!(!rev.is_empty(), "git revision must not be empty");

    if out_path.exists() {
        info!(out = out, "using cached git fetch");
        return Ok(());
    }

    let tmp_parent = out_path.parent().unwrap_or(Path::new("/tmp"));
    std::fs::create_dir_all(tmp_parent)?;
    let tmp_dir = tempfile::Builder::new().prefix("crunch-fetchgit-").tempdir_in(tmp_parent)?;
    let repo_dir = tmp_dir.path().join("repo.git");
    let staged_out = tmp_dir.path().join("out");
    assert!(!repo_dir.exists(), "fresh temp repo path must be absent");
    assert!(!staged_out.exists(), "fresh staged output path must be absent");

    if let Some(local_repo_path) = local_git_repo_path(url)? {
        stage_local_git_tree(&local_repo_path, url, rev, &staged_out)?;
    } else {
        let repo = fetch_git_repo(url, &repo_dir)?;
        let tree_id = resolve_git_tree_id(&repo, url, rev)?;
        stage_git_tree(&repo, tree_id, &staged_out)?;
    }
    std::fs::rename(&staged_out, out_path)?;

    let short_rev = &rev[..rev.len().min(12)];
    info!(url = url, rev = short_rev, out = out, "git fetch complete");
    Ok(())
}

fn local_git_repo_path(url: &str) -> Result<Option<PathBuf>, FetchError> {
    let parsed = Url::parse(url).map_err(|err| FetchError::GitError(format!("invalid git URL '{url}': {err}")))?;
    if parsed.scheme() != "file" {
        return Ok(None);
    }
    let local_path =
        parsed.to_file_path().map_err(|()| FetchError::GitError(format!("invalid git file URL '{url}'")))?;
    Ok(Some(local_path))
}

#[allow(tigerstyle::ambiguous_params)] // url vs git rev: distinct domains
fn stage_local_git_tree(local_repo_path: &Path, url: &str, rev: &str, staged_out: &Path) -> Result<(), FetchError> {
    assert!(staged_out.file_name().is_some(), "staged output path must be named");
    let repo = gix::open_opts(local_repo_path, gix::open::Options::isolated())
        .map_err(|err| FetchError::GitError(format!("failed to open local git repository '{url}': {err}")))?;
    let tree_id = resolve_git_tree_id(&repo, url, rev)?;
    stage_git_tree(&repo, tree_id, staged_out)
}

fn fetch_git_repo(url: &str, repo_dir: &Path) -> Result<gix::Repository, FetchError> {
    let interrupt = AtomicBool::new(false);
    let mut prepare = gix::clone::PrepareFetch::new(
        url,
        repo_dir,
        gix::create::Kind::Bare,
        gix::create::Options {
            destination_must_be_empty: false,
            fs_capabilities: None,
        },
        gix::open::Options::isolated(),
    )
    .map_err(|err| FetchError::GitError(format!("failed to initialize isolated git fetch for '{url}': {err}")))?;

    let (repo, _outcome) = prepare
        .fetch_only(gix::progress::Discard, &interrupt)
        .map_err(|err| FetchError::GitError(format!("failed to fetch git repository '{url}': {err}")))?;
    assert!(repo.workdir().is_none(), "bare fetch repository must not have a workdir");
    assert!(repo.path().exists(), "fetched bare repository path must exist");
    Ok(repo)
}

#[allow(tigerstyle::ambiguous_params)] // url vs git rev: distinct domains
fn resolve_git_tree_id(repo: &gix::Repository, url: &str, rev: &str) -> Result<gix::hash::ObjectId, FetchError> {
    assert!(!url.is_empty(), "git url must not be empty");
    assert!(!rev.is_empty(), "git rev must not be empty");

    let requested = repo.rev_parse_single(rev).map_err(|_err| {
        FetchError::GitError(format!("requested revision '{rev}' could not be materialized from '{url}'"))
    })?;
    let object = requested.object().map_err(|_err| {
        FetchError::GitError(format!("requested revision '{rev}' could not be materialized from '{url}'"))
    })?;
    let tree = object.peel_to_tree().map_err(|_err| {
        FetchError::GitError(format!("requested revision '{rev}' could not be materialized from '{url}'"))
    })?;
    Ok(tree.id().detach())
}

fn stage_git_tree(repo: &gix::Repository, tree_id: gix::hash::ObjectId, staged_out: &Path) -> Result<(), FetchError> {
    assert!(!staged_out.exists(), "staged output directory must not already exist");
    std::fs::create_dir_all(staged_out)?;
    let (mut stream, _index) = repo
        .worktree_stream(tree_id)
        .map_err(|err| FetchError::GitError(format!("failed to prepare git tree materialization: {err}")))?;

    let mut entry_count_u32: u32 = 0;
    while let Some(mut entry) = stream
        .next_entry()
        .map_err(|err| FetchError::GitError(format!("failed to read git tree entry stream: {err}")))?
    {
        entry_count_u32 = entry_count_u32.saturating_add(1);
        if entry_count_u32 > MAX_TAR_ENTRIES {
            return Err(FetchError::GitError(format!("git checkout exceeds entry limit of {MAX_TAR_ENTRIES} entries")));
        }
        let dest = materialize_git_relative_path(staged_out, entry.relative_path())?;
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        write_git_entry(&dest, &mut entry)?;
    }

    assert!(staged_out.is_dir(), "staged git output must exist as a directory");
    Ok(())
}

fn materialize_git_relative_path(staged_out: &Path, relative: &gix::bstr::BStr) -> Result<PathBuf, FetchError> {
    assert!(!staged_out.as_os_str().is_empty(), "staged_out must not be empty");
    assert!(!relative.is_empty(), "git entry relative path must not be empty");
    let relative_path = gix::path::try_from_bstr(relative).map_err(|err| {
        FetchError::GitError(format!("git entry path '{relative}' is not valid on this platform: {err}"))
    })?;

    let mut component_count_u32: u32 = 0;
    let mut dest = PathBuf::from(staged_out);
    for component in relative_path.as_ref().components() {
        match component {
            std::path::Component::Normal(part) => {
                component_count_u32 = component_count_u32.saturating_add(1);
                if component_count_u32 > MAX_GIT_PATH_COMPONENTS {
                    return Err(FetchError::GitError(format!(
                        "git entry path '{relative}' exceeds component limit of {MAX_GIT_PATH_COMPONENTS}"
                    )));
                }
                dest.push(part);
            }
            std::path::Component::CurDir => {
                return Err(FetchError::GitError(format!("git entry path '{relative}' contains '.' component")));
            }
            std::path::Component::ParentDir => {
                return Err(FetchError::GitError(format!("git entry path '{relative}' contains '..' component")));
            }
            std::path::Component::RootDir | std::path::Component::Prefix(_) => {
                return Err(FetchError::GitError(format!("git entry path '{relative}' is absolute")));
            }
        }
    }

    if component_count_u32 == 0 {
        return Err(FetchError::GitError("git entry path must not be empty".to_string()));
    }
    assert!(dest.starts_with(staged_out), "git entry destination must stay under staged output");
    Ok(dest)
}

fn write_git_entry(dest: &Path, entry: &mut gix::worktree::stream::Entry<'_>) -> Result<(), FetchError> {
    assert!(entry.mode.is_blob_or_symlink(), "git stream only yields blobs or symlinks");
    if entry.mode.is_link() {
        write_git_symlink(dest, entry)?;
    } else {
        write_git_blob(dest, entry)?;
    }
    Ok(())
}

fn write_git_blob(dest: &Path, entry: &mut gix::worktree::stream::Entry<'_>) -> Result<(), FetchError> {
    assert!(entry.mode.is_blob(), "blob writer expects a blob entry");
    assert!(!entry.mode.is_link(), "blob writer must not receive symlink entries");

    let mut file = std::fs::File::create(dest)?;
    std::io::copy(entry, &mut file)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = if entry.mode.is_executable() { 0o755 } else { 0o644 };
        std::fs::set_permissions(dest, std::fs::Permissions::from_mode(mode))?;
    }
    Ok(())
}

fn write_git_symlink(dest: &Path, entry: &mut gix::worktree::stream::Entry<'_>) -> Result<(), FetchError> {
    assert!(entry.mode.is_link(), "symlink writer expects a symlink entry");
    let target_bytes = read_git_entry_bytes(entry)?;
    let target_path = git_symlink_target_path(&target_bytes)?;

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&target_path, dest)?;
        return Ok(());
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_file(&target_path, dest)?;
        return Ok(());
    }
    #[allow(unreachable_code)]
    Err(FetchError::GitError("git symlink materialization is unsupported on this platform".to_string()))
}

fn read_git_entry_bytes(entry: &mut gix::worktree::stream::Entry<'_>) -> Result<Vec<u8>, FetchError> {
    let capacity_bytes = entry.bytes_remaining().unwrap_or(0);
    let mut buf = Vec::with_capacity(capacity_bytes);
    entry.read_to_end(&mut buf)?;
    Ok(buf)
}

fn git_symlink_target_path(target_bytes: &[u8]) -> Result<PathBuf, FetchError> {
    #[cfg(unix)]
    {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;
        Ok(PathBuf::from(OsString::from_vec(target_bytes.to_vec())))
    }
    #[cfg(not(unix))]
    {
        let target = std::str::from_utf8(target_bytes).map_err(|err| {
            FetchError::GitError(format!("git symlink target is not valid UTF-8 on this platform: {err}"))
        })?;
        Ok(PathBuf::from(target))
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use nix_compat::derivation::Output;

    use super::*;

    /// Build a minimal derivation with the given env, builder, and ca_hash.
    fn make_drv(builder: &str, env: Vec<(&str, &str)>, ca_hash: Option<CAHash>) -> Derivation {
        let mut environment: BTreeMap<String, bstr::BString> = BTreeMap::new();
        for (k, v) in env {
            environment.insert(k.to_string(), bstr::BString::from(v.as_bytes()));
        }

        let mut outputs = BTreeMap::new();
        outputs.insert("out".to_string(), Output { path: None, ca_hash });

        Derivation {
            builder: builder.to_string(),
            system: "builtin".to_string(),
            arguments: vec![],
            environment,
            input_derivations: BTreeMap::new(),
            input_sources: std::collections::BTreeSet::new(),
            outputs,
        }
    }

    static PATH_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    struct PathGuard {
        original: Option<std::ffi::OsString>,
    }

    impl PathGuard {
        fn set(path: &std::ffi::OsStr) -> Self {
            let original = std::env::var_os("PATH");
            unsafe { std::env::set_var("PATH", path) };
            Self { original }
        }
    }

    impl Drop for PathGuard {
        fn drop(&mut self) {
            match &self.original {
                Some(path) => unsafe { std::env::set_var("PATH", path) },
                None => unsafe { std::env::remove_var("PATH") },
            }
        }
    }

    fn run_git(dir: &Path, args: &[&str]) -> String {
        let output = std::process::Command::new("git").args(args).current_dir(dir).output().unwrap();
        assert!(output.status.success(), "git {:?} failed: {}", args, String::from_utf8_lossy(&output.stderr));
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    fn create_git_repo_with_history() -> (tempfile::TempDir, PathBuf, String, String) {
        let tempdir = tempfile::tempdir().unwrap();
        let repo = tempdir.path().join("repo");
        std::fs::create_dir_all(&repo).unwrap();

        run_git(&repo, &["init"]);
        run_git(&repo, &["branch", "-M", "main"]);
        run_git(&repo, &["config", "user.email", "test@example.com"]);
        run_git(&repo, &["config", "user.name", "Test User"]);

        std::fs::write(repo.join("hello.txt"), "first revision\n").unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "first"]);
        let first_rev = run_git(&repo, &["rev-parse", "HEAD"]);

        std::fs::write(repo.join("hello.txt"), "second revision\n").unwrap();
        std::fs::write(repo.join("extra.txt"), "present only in second revision\n").unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "second"]);
        let second_rev = run_git(&repo, &["rev-parse", "HEAD"]);

        (tempdir, repo, first_rev, second_rev)
    }

    fn file_url(path: &Path) -> String {
        format!("file://{}", path.display())
    }

    fn create_bare_git_repo_with_history() -> (tempfile::TempDir, PathBuf, String, String) {
        let (tempdir, repo, first_rev, second_rev) = create_git_repo_with_history();
        let bare_repo = tempdir.path().join("repo.git");
        run_git(tempdir.path(), &["clone", "--bare", repo.to_str().unwrap(), bare_repo.to_str().unwrap()]);
        assert!(bare_repo.exists(), "bare git fixture must exist");
        assert!(bare_repo.join("HEAD").exists(), "bare git fixture must contain HEAD");
        (tempdir, bare_repo, first_rev, second_rev)
    }

    #[cfg(unix)]
    fn write_fake_git_tool(dir: &Path, name: &str, marker: &Path, version: &str) {
        use std::os::unix::fs::PermissionsExt;

        let tool = dir.join(name);
        let script = format!(
            "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\n  printf '%s\\n' '{version}' >&2\nfi\nprintf '%s %s\\n' '{name}' \"$*\" >> '{}'\nexit 97\n",
            marker.display()
        );
        std::fs::write(&tool, script).unwrap();
        std::fs::set_permissions(&tool, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    #[cfg(unix)]
    fn make_fake_git_toolchain(dir: &Path, marker: &Path, version: &str) {
        std::fs::create_dir_all(dir).unwrap();
        write_fake_git_tool(dir, "git", marker, version);
        write_fake_git_tool(dir, "git-upload-pack", marker, version);
        write_fake_git_tool(dir, "ssh", marker, version);
    }

    #[cfg(unix)]
    struct GitDaemonGuard {
        stop: std::sync::Arc<AtomicBool>,
        thread: Option<std::thread::JoinHandle<()>>,
        _log_root: tempfile::TempDir,
        log_path: PathBuf,
        port: u16,
    }

    #[cfg(unix)]
    impl GitDaemonGuard {
        fn url_for(&self, repo_name: &str) -> String {
            assert!(!repo_name.is_empty(), "repo name must not be empty");
            format!("git://127.0.0.1:{}/{}", self.port, repo_name)
        }
    }

    #[cfg(unix)]
    impl Drop for GitDaemonGuard {
        fn drop(&mut self) {
            self.stop.store(true, std::sync::atomic::Ordering::SeqCst);
            let _ = std::net::TcpStream::connect(("127.0.0.1", self.port));
            if let Some(thread) = self.thread.take() {
                let _ = thread.join();
            }
        }
    }

    #[cfg(unix)]
    fn append_git_daemon_log_line(log_path: &Path, line: &str) {
        use std::io::Write;

        let mut log = std::fs::OpenOptions::new().create(true).append(true).open(log_path).unwrap();
        writeln!(log, "{line}").unwrap();
    }

    #[cfg(unix)]
    fn wait_for_inetd_child(
        child: &mut std::process::Child,
        stop: &AtomicBool,
        log_path: &Path,
        max_wait_attempts_u32: u32,
        wait_sleep_ms: u64,
    ) -> Result<(), String> {
        assert!(max_wait_attempts_u32 > 0, "inetd child wait attempts must be positive");
        assert!(wait_sleep_ms > 0, "inetd child wait sleep must be positive");

        for _attempt in 0..max_wait_attempts_u32 {
            if let Some(status) =
                child.try_wait().map_err(|err| format!("failed to poll inetd git daemon child: {err}"))?
            {
                if status.success() {
                    return Ok(());
                }
                let log = std::fs::read_to_string(log_path).unwrap_or_default();
                return Err(format!("inetd git daemon child exited with {status}: {log}"));
            }
            if stop.load(std::sync::atomic::Ordering::SeqCst) {
                let _ = child.kill();
                let _ = child.wait();
                return Ok(());
            }
            std::thread::sleep(std::time::Duration::from_millis(wait_sleep_ms));
        }

        let _ = child.kill();
        let status = child.wait().map_err(|err| format!("failed to reap timed-out inetd git daemon child: {err}"))?;
        let log = std::fs::read_to_string(log_path).unwrap_or_default();
        Err(format!(
            "inetd git daemon child exceeded timeout after {} ms and ended with {status}: {log}",
            max_wait_attempts_u32 as u64 * wait_sleep_ms
        ))
    }

    fn serve_git_daemon_connection(
        stream: std::net::TcpStream,
        repo_root: &Path,
        git_path: &Path,
        log_path: &Path,
        stop: &AtomicBool,
    ) -> Result<(), String> {
        use std::os::fd::FromRawFd;
        use std::os::fd::IntoRawFd;

        const MAX_CHILD_WAIT_ATTEMPTS: u32 = 400;
        const CHILD_WAIT_SLEEP_MS: u64 = 25;

        let stdout_stream = stream.try_clone().map_err(|err| format!("failed to clone git socket: {err}"))?;
        let stdin_file = unsafe { std::fs::File::from_raw_fd(stream.into_raw_fd()) };
        let stdout_file = unsafe { std::fs::File::from_raw_fd(stdout_stream.into_raw_fd()) };
        let stderr_log = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .map_err(|err| format!("failed to open git daemon log: {err}"))?;
        let mut child = std::process::Command::new(git_path)
            .arg("daemon")
            .arg("--inetd")
            .arg("--export-all")
            .arg(format!("--base-path={}", repo_root.display()))
            .arg(repo_root)
            .stdin(std::process::Stdio::from(stdin_file))
            .stdout(std::process::Stdio::from(stdout_file))
            .stderr(std::process::Stdio::from(stderr_log))
            .spawn()
            .map_err(|err| format!("failed to spawn inetd git daemon child: {err}"))?;
        wait_for_inetd_child(&mut child, stop, log_path, MAX_CHILD_WAIT_ATTEMPTS, CHILD_WAIT_SLEEP_MS)
    }

    #[cfg(unix)]
    fn host_git_path() -> PathBuf {
        let output = std::process::Command::new("which").arg("git").output().unwrap();
        assert!(output.status.success(), "which git must succeed before PATH poisoning");
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        assert!(!path.is_empty(), "which git must return a path");
        PathBuf::from(path)
    }

    #[cfg(unix)]
    fn spawn_git_daemon(repo_root: &Path, git_path: &Path) -> GitDaemonGuard {
        assert!(repo_root.is_dir(), "git daemon repo root must exist");
        assert!(git_path.is_absolute(), "git daemon helper must use an absolute git path");
        assert!(git_path.exists(), "git daemon helper git path must exist");

        let log_root = tempfile::tempdir().unwrap();
        let log_path = log_root.path().join("git-daemon.log");
        std::fs::File::create(&log_path).unwrap();
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        assert_ne!(port, 0, "git daemon listener must bind a real port");

        let stop = std::sync::Arc::new(AtomicBool::new(false));
        let stop_thread = std::sync::Arc::clone(&stop);
        let repo_root_buf = repo_root.to_path_buf();
        let git_path_buf = git_path.to_path_buf();
        let log_path_thread = log_path.clone();
        let thread = std::thread::spawn(move || {
            loop {
                let (stream, _addr) = listener.accept().unwrap();
                if stop_thread.load(std::sync::atomic::Ordering::SeqCst) {
                    break;
                }
                if let Err(err) =
                    serve_git_daemon_connection(stream, &repo_root_buf, &git_path_buf, &log_path_thread, &stop_thread)
                {
                    append_git_daemon_log_line(&log_path_thread, &err);
                }
            }
        });

        assert!(log_path.exists(), "git daemon log path must exist");
        GitDaemonGuard {
            stop,
            thread: Some(thread),
            _log_root: log_root,
            log_path,
            port,
        }
    }

    fn collect_checkout_snapshot(root: &Path) -> Vec<String> {
        fn visit(path: &Path, root: &Path, out: &mut Vec<String>) {
            let mut children = std::fs::read_dir(path).unwrap().map(|entry| entry.unwrap().path()).collect::<Vec<_>>();
            children.sort();
            for child in children {
                let rel = child.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/");
                let meta = std::fs::symlink_metadata(&child).unwrap();
                if meta.is_dir() {
                    out.push(format!("dir:{rel}"));
                    visit(&child, root, out);
                    continue;
                }
                if meta.file_type().is_symlink() {
                    let target = std::fs::read_link(&child).unwrap();
                    out.push(format!("symlink:{rel}:{}", target.display()));
                    continue;
                }
                let bytes = std::fs::read(&child).unwrap();
                out.push(format!("file:{rel}:{:?}", bytes));
            }
        }

        assert!(root.is_dir(), "checkout snapshot root must be a directory");
        let mut out = Vec::new();
        visit(root, root, &mut out);
        assert!(!out.iter().any(|entry| entry.contains(".git")), "snapshot must not contain .git entries: {out:?}");
        out
    }

    #[cfg(unix)]
    fn spawn_stuck_child() -> std::process::Child {
        let shell = Path::new("/bin/sh");
        assert!(shell.exists(), "test shell must exist");
        std::process::Command::new(shell).arg("-c").arg("while :; do :; done").spawn().unwrap()
    }

    #[cfg(unix)]
    #[test]
    fn wait_for_inetd_child_kills_on_stop_signal() {
        let tempdir = tempfile::tempdir().unwrap();
        let log_path = tempdir.path().join("git-daemon.log");
        std::fs::write(&log_path, "").unwrap();
        let stop = AtomicBool::new(true);
        let mut child = spawn_stuck_child();

        wait_for_inetd_child(&mut child, &stop, &log_path, 4, 5).unwrap();

        let status = child.try_wait().unwrap();
        assert!(status.is_some(), "stop path must reap inetd child");
        assert!(log_path.exists(), "stop-path log file must still exist");
    }

    #[cfg(unix)]
    #[test]
    fn wait_for_inetd_child_times_out_and_reaps_child() {
        let tempdir = tempfile::tempdir().unwrap();
        let log_path = tempdir.path().join("git-daemon.log");
        std::fs::write(&log_path, "").unwrap();
        let stop = AtomicBool::new(false);
        let mut child = spawn_stuck_child();

        let err = wait_for_inetd_child(&mut child, &stop, &log_path, 2, 5).unwrap_err();

        assert!(err.contains("exceeded timeout"), "timeout path must report timeout: {err}");
        let status = child.try_wait().unwrap();
        assert!(status.is_some(), "timeout path must reap inetd child");
    }

    // ── parse_fetch tests ──────────────────────────────────────────

    #[test]
    fn parse_fetch_url_flat() {
        let drv = make_drv(
            "builtin:fetchurl",
            vec![("url", "https://example.com/foo.txt")],
            Some(CAHash::Flat(NixHash::Sha256([0xAA; 32]))),
        );
        let fetch = parse_fetch(&drv).unwrap();
        assert!(matches!(fetch, Fetch::Url { .. }));
        if let Fetch::Url { url, exp_hash } = &fetch {
            assert_eq!(url.as_str(), "https://example.com/foo.txt");
            assert!(exp_hash.is_some());
        }
    }

    #[test]
    fn parse_fetch_tarball_unpack() {
        let drv = make_drv(
            "builtin:fetchurl",
            vec![("url", "https://example.com/src.tar.gz"), ("unpack", "1")],
            Some(CAHash::Nar(NixHash::Sha256([0xBB; 32]))),
        );
        let fetch = parse_fetch(&drv).unwrap();
        assert!(matches!(fetch, Fetch::Tarball { .. }));
        if let Fetch::Tarball { exp_nar_sha256, .. } = &fetch {
            assert_eq!(*exp_nar_sha256, Some([0xBB; 32]));
        }
    }

    #[test]
    fn parse_fetch_executable() {
        let drv = make_drv(
            "builtin:fetchurl",
            vec![("url", "https://example.com/run"), ("executable", "1")],
            Some(CAHash::Flat(NixHash::Sha256([0xCC; 32]))),
        );
        let fetch = parse_fetch(&drv).unwrap();
        assert!(matches!(fetch, Fetch::Executable { .. }));
    }

    #[test]
    fn parse_fetch_nar() {
        let drv = make_drv(
            "builtin:fetchurl",
            vec![("url", "https://cache.nixos.org/nar/abc.nar.xz")],
            Some(CAHash::Nar(NixHash::Sha256([0xDD; 32]))),
        );
        let fetch = parse_fetch(&drv).unwrap();
        assert!(matches!(fetch, Fetch::Nar { .. }));
    }

    #[test]
    fn parse_fetch_git() {
        let drv = make_drv(
            "builtin:fetchurl",
            vec![
                ("url", "https://github.com/user/repo.git"),
                ("type", "git"),
                ("rev", "abc123def456"),
            ],
            Some(CAHash::Nar(NixHash::Sha256([0xEE; 32]))),
        );
        let fetch = parse_fetch(&drv).unwrap();
        assert!(matches!(fetch, Fetch::Git { .. }));
        if let Fetch::Git { url, rev, .. } = &fetch {
            assert_eq!(url, "https://github.com/user/repo.git");
            assert_eq!(rev, "abc123def456");
        }
    }

    #[test]
    fn parse_fetch_git_missing_rev() {
        let drv =
            make_drv("builtin:fetchurl", vec![("url", "https://github.com/user/repo.git"), ("type", "git")], None);
        let err = parse_fetch(&drv).unwrap_err();
        assert!(matches!(err, FetchError::MissingGitRev));
    }

    #[test]
    fn parse_fetch_not_a_fetcher() {
        let drv = make_drv("/bin/sh", vec![], None);
        let err = parse_fetch(&drv).unwrap_err();
        assert!(matches!(err, FetchError::NotFetcher(_)));
    }

    #[test]
    fn parse_fetch_missing_url() {
        let drv = make_drv("builtin:fetchurl", vec![], None);
        let err = parse_fetch(&drv).unwrap_err();
        assert!(matches!(err, FetchError::MissingUrl));
    }

    #[test]
    fn parse_fetch_url_no_hash() {
        let drv = make_drv("builtin:fetchurl", vec![("url", "https://example.com/file.txt")], None);
        let fetch = parse_fetch(&drv).unwrap();
        assert!(matches!(fetch, Fetch::Url { exp_hash: None, .. }));
    }

    #[test]
    fn fetch_git_materializes_requested_revision_without_dot_git() {
        let _path_lock = PATH_MUTEX.lock().unwrap();
        let (_tempdir, repo, first_rev, second_rev) = create_git_repo_with_history();
        let out_root = tempfile::tempdir().unwrap();
        let out = out_root.path().join("git-checkout");

        fetch_git(&file_url(&repo), &first_rev, out.to_str().unwrap()).unwrap();

        assert!(out.is_dir(), "git fetch should produce a directory output");
        assert_eq!(std::fs::read_to_string(out.join("hello.txt")).unwrap(), "first revision\n");
        assert!(!out.join("extra.txt").exists(), "older revision should not contain later file");
        assert!(!out.join(".git").exists(), "fetchGit output must not contain .git metadata");
        assert_ne!(first_rev, second_rev, "history fixture must contain two distinct revisions");
    }

    #[test]
    fn fetch_git_succeeds_with_empty_path() {
        let _path_lock = PATH_MUTEX.lock().unwrap();
        let (_tempdir, repo, first_rev, _second_rev) = create_git_repo_with_history();
        let out_root = tempfile::tempdir().unwrap();
        let out = out_root.path().join("git-checkout");

        let _guard = PathGuard::set(std::ffi::OsStr::new(""));
        fetch_git(&file_url(&repo), &first_rev, out.to_str().unwrap()).unwrap();

        assert!(out.join("hello.txt").exists(), "fetchGit should not require host PATH git");
        assert_eq!(std::fs::read_to_string(out.join("hello.txt")).unwrap(), "first revision\n");
    }

    #[cfg(unix)]
    #[test]
    fn fetch_git_ignores_fake_git_binary_on_path() {
        let _path_lock = PATH_MUTEX.lock().unwrap();
        let (_tempdir, repo, first_rev, _second_rev) = create_git_repo_with_history();
        let fake_root = tempfile::tempdir().unwrap();
        let marker = fake_root.path().join("fake-git-invoked.txt");
        make_fake_git_toolchain(fake_root.path(), &marker, "git version 2.99-test");
        let out_root = tempfile::tempdir().unwrap();
        let out = out_root.path().join("git-checkout");

        let _guard = PathGuard::set(fake_root.path().as_os_str());
        fetch_git(&file_url(&repo), &first_rev, out.to_str().unwrap()).unwrap();

        assert!(out.join("hello.txt").exists(), "fetchGit should succeed with only fake git on PATH");
        assert!(!marker.exists(), "fetchGit must not invoke fake host git binaries");
    }

    #[cfg(unix)]
    #[test]
    fn fetch_git_git_transport_ignores_fake_git_versions_and_keeps_errors_crunch_owned() {
        let _path_lock = PATH_MUTEX.lock().unwrap();
        let (_tempdir, bare_repo, first_rev, _second_rev) = create_bare_git_repo_with_history();
        let git_path = host_git_path();
        let daemon = spawn_git_daemon(bare_repo.parent().unwrap(), &git_path);
        let repo_name = bare_repo.file_name().unwrap().to_string_lossy().to_string();
        let url = daemon.url_for(&repo_name);
        let out_root = tempfile::tempdir().unwrap();

        let fake_root_a = tempfile::tempdir().unwrap();
        let marker_a = fake_root_a.path().join("fake-git-a.txt");
        make_fake_git_toolchain(fake_root_a.path(), &marker_a, "git version 1.0-a");
        let out_a = out_root.path().join("git-checkout-a");
        let error_a = {
            let _guard = PathGuard::set(fake_root_a.path().as_os_str());
            fetch_git(&url, &first_rev, out_a.to_str().unwrap()).unwrap_or_else(|err| {
                let log = std::fs::read_to_string(&daemon.log_path).unwrap_or_default();
                panic!("remote fetch failed: {err}; git daemon log: {log}")
            });
            let invalid_rev = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef";
            fetch_git(&url, invalid_rev, out_root.path().join("git-error-a").to_str().unwrap())
                .unwrap_err()
                .to_string()
        };

        let fake_root_b = tempfile::tempdir().unwrap();
        let marker_b = fake_root_b.path().join("fake-git-b.txt");
        make_fake_git_toolchain(fake_root_b.path(), &marker_b, "git version 9.9-b");
        let out_b = out_root.path().join("git-checkout-b");
        let error_b = {
            let _guard = PathGuard::set(fake_root_b.path().as_os_str());
            fetch_git(&url, &first_rev, out_b.to_str().unwrap()).unwrap_or_else(|err| {
                let log = std::fs::read_to_string(&daemon.log_path).unwrap_or_default();
                panic!("remote fetch failed: {err}; git daemon log: {log}")
            });
            let invalid_rev = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef";
            fetch_git(&url, invalid_rev, out_root.path().join("git-error-b").to_str().unwrap())
                .unwrap_err()
                .to_string()
        };

        let snapshot_a = collect_checkout_snapshot(&out_a);
        let snapshot_b = collect_checkout_snapshot(&out_b);
        assert_eq!(snapshot_a, snapshot_b, "fake host git version must not change fetched tree");
        assert_eq!(std::fs::read_to_string(out_a.join("hello.txt")).unwrap(), "first revision\n");
        assert!(!out_a.join(".git").exists(), "remote fetchGit output must not contain .git");
        assert!(!out_b.join(".git").exists(), "remote fetchGit output must not contain .git");
        assert_eq!(
            error_a,
            format!(
                "git fetch failed: requested revision 'deadbeefdeadbeefdeadbeefdeadbeefdeadbeef' could not be materialized from '{url}'"
            )
        );
        assert_eq!(error_a, error_b, "fake host git version must not change invalid-rev error text");
        assert!(!error_a.contains("fatal:"), "remote invalid-rev error must stay crunch-owned: {error_a}");
        assert!(!marker_a.exists(), "remote git transport must not invoke PATH git helpers");
        assert!(!marker_b.exists(), "remote git transport must not invoke PATH git helpers");
    }

    #[test]
    fn fetch_git_invalid_revision_reports_crunch_owned_error() {
        let _path_lock = PATH_MUTEX.lock().unwrap();
        let (_tempdir, repo, _first_rev, _second_rev) = create_git_repo_with_history();
        let out_root = tempfile::tempdir().unwrap();
        let out = out_root.path().join("git-checkout");
        let url = file_url(&repo);
        let err = fetch_git(&url, "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef", out.to_str().unwrap()).unwrap_err();
        let message = err.to_string();

        assert_eq!(
            message,
            format!(
                "git fetch failed: requested revision 'deadbeefdeadbeefdeadbeefdeadbeefdeadbeef' could not be materialized from '{url}'"
            )
        );
        assert!(!message.contains("fatal:"), "error should not depend on host git stderr: {message}");
    }

    #[test]
    fn fetch_git_missing_repo_reports_crunch_owned_error() {
        let tempdir = tempfile::tempdir().unwrap();
        let missing_repo = tempdir.path().join("missing-repo");
        let out = tempdir.path().join("git-checkout");
        let url = file_url(&missing_repo);
        let err = fetch_git(&url, "refs/heads/main", out.to_str().unwrap()).unwrap_err();
        let message = err.to_string();

        assert!(
            message.starts_with(&format!("git fetch failed: failed to open local git repository '{url}':")),
            "unexpected fetch failure message: {message}"
        );
        assert!(!message.contains("fatal:"), "error should not depend on host git stderr: {message}");
    }

    #[test]
    fn fetch_git_non_test_implementation_has_no_host_git_discovery_patterns() {
        let source = include_str!("fetcher.rs");
        let non_test = source.split("#[cfg(test)]").next().unwrap();

        assert!(!non_test.contains("std::process::Command"));
        assert!(!non_test.contains("std::env::var(\"PATH\")"));
        assert!(!non_test.contains("git-upload-pack"));
        assert!(!non_test.contains("--version"));
        assert!(!non_test.contains("command -v git"));
        for probe in [
            "/usr/bin/git",
            "/bin/git",
            "/usr/local/bin/git",
            "/run/current-system/sw/bin/git",
            "/nix/store/",
        ] {
            assert!(!non_test.contains(probe), "non-test fetcher code must not probe host git path {probe}");
        }
    }

    // ── extract_tar tests ──────────────────────────────────────────

    #[test]
    fn validate_entry_not_absolute_rejects_root_component() {
        let err = validate_entry_not_absolute(Path::new("/escape.txt")).unwrap_err();
        let message = err.to_string();

        assert!(message.contains("absolute path"));
        assert!(message.contains("/escape.txt"));
    }

    #[test]
    fn resolve_relative_within_root_allows_in_tree_dotdot_target() {
        let resolved = resolve_relative_within_root(Path::new("sub"), Path::new("../real.txt"), "probe").unwrap();

        assert_eq!(resolved, PathBuf::from("real.txt"));
    }

    #[test]
    fn extract_tar_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("output");
        // An empty tar is 1024 zero bytes (two end-of-archive markers).
        let data = vec![0u8; 1024];
        let result = extract_tar(io::Cursor::new(data), out.to_str().unwrap());
        assert!(result.is_ok());
        assert!(out.is_dir());
    }

    #[test]
    fn extract_tar_preserves_flat_archive_without_wrapper_directory() {
        let tar_data = build_raw_tar(&[
            RawTarEntry::file("file.txt", b"root"),
            RawTarEntry::dir("dir/"),
            RawTarEntry::file("dir/nested.txt", b"nested"),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");

        extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap();
        assert_eq!(std::fs::read_to_string(out.join("file.txt")).unwrap(), "root");
        assert_eq!(std::fs::read_to_string(out.join("dir/nested.txt")).unwrap(), "nested");
    }

    #[test]
    fn extract_tar_preserves_wrapper_prefix_without_explicit_directory_entry() {
        let tar_data = build_raw_tar(&[
            RawTarEntry::file("proj-v1/Cargo.toml", b"[package]"),
            RawTarEntry::file("proj-v1/src/main.rs", b"fn main() {}"),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");

        extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap();
        assert_eq!(std::fs::read_to_string(out.join("Cargo.toml")).unwrap(), "[package]");
        assert_eq!(std::fs::read_to_string(out.join("src/main.rs")).unwrap(), "fn main() {}");
    }

    #[test]
    fn extract_tar_preserves_mixed_top_level_archive_without_stripping() {
        let tar_data = build_raw_tar(&[
            RawTarEntry::file("pkg/file.txt", b"payload"),
            RawTarEntry::file("LICENSE", b"license"),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");

        extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap();
        assert_eq!(std::fs::read_to_string(out.join("pkg/file.txt")).unwrap(), "payload");
        assert_eq!(std::fs::read_to_string(out.join("LICENSE")).unwrap(), "license");
    }

    #[test]
    fn extract_tar_single_file() {
        let tmp_src = tempfile::tempdir().unwrap();
        let inner = tmp_src.path().join("project-v1");
        std::fs::create_dir(&inner).unwrap();
        std::fs::write(inner.join("hello.txt"), "hello world").unwrap();

        let mut builder = tar::Builder::new(Vec::new());
        builder.append_dir_all("project-v1", &inner).unwrap();
        let tar_data = builder.into_inner().unwrap();

        let tmp_out = tempfile::tempdir().unwrap();
        let out = tmp_out.path().join("extracted");
        extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap();

        // Top-level "project-v1/" stripped.
        assert!(out.join("hello.txt").exists());
        assert_eq!(std::fs::read_to_string(out.join("hello.txt")).unwrap(), "hello world");
    }

    #[test]
    fn extract_tar_prefix_stripping() {
        let tmp_src = tempfile::tempdir().unwrap();
        let inner = tmp_src.path().join("nixpkgs-abc123");
        std::fs::create_dir_all(inner.join("pkgs/tools")).unwrap();
        std::fs::write(inner.join("pkgs/tools/rg.nix"), "{ }").unwrap();

        let mut builder = tar::Builder::new(Vec::new());
        builder.append_dir_all("nixpkgs-abc123", &inner).unwrap();
        let tar_data = builder.into_inner().unwrap();

        let tmp_out = tempfile::tempdir().unwrap();
        let out = tmp_out.path().join("extracted");
        extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap();

        assert!(out.join("pkgs/tools/rg.nix").exists());
    }

    #[test]
    fn extract_tar_symlink() {
        // Build a tar with an explicit symlink entry (append_dir_all
        // follows symlinks, so we construct the archive manually).
        let mut builder = tar::Builder::new(Vec::new());

        // Directory entry.
        let mut dir_hdr = tar::Header::new_gnu();
        dir_hdr.set_entry_type(tar::EntryType::Directory);
        dir_hdr.set_size(0);
        dir_hdr.set_mode(0o755);
        dir_hdr.set_cksum();
        builder.append_data(&mut dir_hdr, "pkg/", io::empty()).unwrap();

        // Regular file.
        let content = b"content";
        let mut file_hdr = tar::Header::new_gnu();
        file_hdr.set_entry_type(tar::EntryType::Regular);
        file_hdr.set_size(content.len() as u64);
        file_hdr.set_mode(0o644);
        file_hdr.set_cksum();
        builder.append_data(&mut file_hdr, "pkg/real.txt", io::Cursor::new(content)).unwrap();

        // Symlink entry.
        let mut sym_hdr = tar::Header::new_gnu();
        sym_hdr.set_entry_type(tar::EntryType::Symlink);
        sym_hdr.set_size(0);
        sym_hdr.set_mode(0o777);
        sym_hdr.set_link_name("real.txt").unwrap();
        sym_hdr.set_cksum();
        builder.append_data(&mut sym_hdr, "pkg/link.txt", io::empty()).unwrap();

        let tar_data = builder.into_inner().unwrap();

        let tmp_out = tempfile::tempdir().unwrap();
        let out = tmp_out.path().join("extracted");
        extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap();

        assert!(out.join("real.txt").exists());
        #[cfg(unix)]
        {
            assert!(out.join("link.txt").is_symlink());
            assert_eq!(std::fs::read_link(out.join("link.txt")).unwrap().to_str().unwrap(), "real.txt");
        }
    }

    // ── decompress_reader tests ────────────────────────────────────

    #[test]
    fn decompress_raw_tar() {
        let data = b"raw content";
        let mut out = Vec::new();
        decompress_reader("foo.tar", io::Cursor::new(data.to_vec())).unwrap().read_to_end(&mut out).unwrap();
        assert_eq!(out, data);
    }

    #[test]
    fn decompress_gzip() {
        use std::io::Write;

        use flate2::write::GzEncoder;

        let mut encoder = GzEncoder::new(Vec::new(), flate2::Compression::fast());
        encoder.write_all(b"gzipped content").unwrap();
        let compressed = encoder.finish().unwrap();

        let mut out = Vec::new();
        decompress_reader("file.tar.gz", io::Cursor::new(compressed))
            .unwrap()
            .read_to_end(&mut out)
            .unwrap();
        assert_eq!(out, b"gzipped content");
    }

    #[test]
    fn decompress_xz() {
        let mut compressed = Vec::new();
        lzma_rs::xz_compress(&mut io::Cursor::new(b"xz content"), &mut compressed).unwrap();

        let mut out = Vec::new();
        decompress_reader("file.tar.xz", io::Cursor::new(compressed))
            .unwrap()
            .read_to_end(&mut out)
            .unwrap();
        assert_eq!(out, b"xz content");
    }

    #[test]
    fn decompress_zstd() {
        let compressed = zstd::encode_all(io::Cursor::new(b"zstd content"), 1).unwrap();

        let mut out = Vec::new();
        decompress_reader("file.tar.zst", io::Cursor::new(compressed))
            .unwrap()
            .read_to_end(&mut out)
            .unwrap();
        assert_eq!(out, b"zstd content");
    }

    #[test]
    fn decompress_unknown_with_gzip_magic() {
        use std::io::Write;

        use flate2::write::GzEncoder;

        let mut encoder = GzEncoder::new(Vec::new(), flate2::Compression::fast());
        encoder.write_all(b"magic detected").unwrap();
        let compressed = encoder.finish().unwrap();

        // Unknown suffix but gzip magic bytes present.
        let mut out = Vec::new();
        decompress_reader("file.unknown", io::Cursor::new(compressed))
            .unwrap()
            .read_to_end(&mut out)
            .unwrap();
        assert_eq!(out, b"magic detected");
    }

    #[test]
    fn decompress_unknown_raw_fallback() {
        let data = b"just raw bytes, no magic";
        let mut out = Vec::new();
        decompress_reader("file.unknown", io::Cursor::new(data.to_vec()))
            .unwrap()
            .read_to_end(&mut out)
            .unwrap();
        assert_eq!(out, data);
    }

    // ── verify_flat_hash tests ─────────────────────────────────────

    #[test]
    fn verify_flat_hash_sha256_ok() {
        use sha2::Digest;
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), b"test content").unwrap();

        let digest: [u8; 32] = sha2::Sha256::digest(b"test content").into();
        let expected = NixHash::Sha256(digest);
        let result = verify_flat_hash(tmp.path().to_str().unwrap(), &expected, "test");
        assert!(result.is_ok());
    }

    #[test]
    fn verify_flat_hash_sha256_mismatch() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), b"actual content").unwrap();

        let expected = NixHash::Sha256([0xFF; 32]);
        let result = verify_flat_hash(tmp.path().to_str().unwrap(), &expected, "test");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, FetchError::HashMismatch { .. }));
    }

    // ── nix_hash_to_sri tests ──────────────────────────────────────

    #[test]
    fn sri_format() {
        let hash = NixHash::Sha256([0; 32]);
        let sri = nix_hash_to_sri(&hash);
        assert!(sri.starts_with("sha256-"));
    }

    // ── tar containment regression tests ──────────────────────────────

    /// Build a raw tar archive from explicit header entries. Allows
    /// crafting entries that high-level builders would reject.
    fn build_raw_tar(entries: &[RawTarEntry<'_>]) -> Vec<u8> {
        let mut buf = Vec::new();
        for entry in entries {
            let mut header = [0u8; 512];
            // name (0..100)
            let name = entry.path.as_bytes();
            assert!(name.len() < 100, "tar entry name too long for test helper");
            header[..name.len()].copy_from_slice(name);
            // mode (100..108)
            write_raw_octal(&mut header[100..108], entry.mode as u64);
            // size (124..136)
            write_raw_octal(&mut header[124..136], entry.data.len() as u64);
            // typeflag (156)
            header[156] = entry.typeflag;
            // linkname (157..257)
            if let Some(link) = entry.linkname {
                let link_bytes = link.as_bytes();
                assert!(link_bytes.len() < 100, "tar linkname too long for test helper");
                header[157..157 + link_bytes.len()].copy_from_slice(link_bytes);
            }
            // magic + version
            header[257..263].copy_from_slice(b"ustar\0");
            header[263..265].copy_from_slice(b"00");
            // checksum (148..156) — compute over header with checksum field as spaces
            header[148..156].fill(b' ');
            let cksum: u32 = header.iter().map(|b| *b as u32).sum();
            let cksum_str = format!("{cksum:06o}\0 ");
            header[148..156].copy_from_slice(cksum_str.as_bytes());

            buf.extend_from_slice(&header);
            buf.extend_from_slice(entry.data);
            // pad to 512-byte boundary
            let remainder = buf.len() % 512;
            if remainder != 0 {
                buf.extend(std::iter::repeat_n(0u8, 512 - remainder));
            }
        }
        // two end-of-archive blocks
        buf.extend_from_slice(&[0u8; 1024]);
        buf
    }

    fn write_raw_octal(field: &mut [u8], value: u64) {
        let width = field.len().saturating_sub(1);
        let s = format!("{value:0width$o}\0");
        field[..s.len()].copy_from_slice(s.as_bytes());
    }

    fn leak_test_str(value: String) -> &'static str {
        Box::leak(value.into_boxed_str())
    }

    struct RawTarEntry<'a> {
        path: &'a str,
        typeflag: u8,
        mode: u32,
        linkname: Option<&'a str>,
        data: &'a [u8],
    }

    impl<'a> RawTarEntry<'a> {
        fn dir(path: &'a str) -> Self {
            Self {
                path,
                typeflag: b'5',
                mode: 0o755,
                linkname: None,
                data: &[],
            }
        }
        fn file(path: &'a str, data: &'a [u8]) -> Self {
            Self {
                path,
                typeflag: b'0',
                mode: 0o644,
                linkname: None,
                data,
            }
        }
        fn symlink(path: &'a str, target: &'a str) -> Self {
            Self {
                path,
                typeflag: b'2',
                mode: 0o777,
                linkname: Some(target),
                data: &[],
            }
        }
        fn hardlink(path: &'a str, target: &'a str) -> Self {
            Self {
                path,
                typeflag: b'1',
                mode: 0o644,
                linkname: Some(target),
                data: &[],
            }
        }
    }

    #[test]
    fn extract_tar_rejects_absolute_entry_path() {
        let tar_data = build_raw_tar(&[RawTarEntry::file("/escape.txt", b"bad")]);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");

        let err = extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap_err();
        assert!(err.to_string().contains("absolute path"), "expected absolute path error: {err}");
        assert!(!tmp.path().join("escape.txt").exists(), "absolute entry must not escape");
    }

    #[cfg(unix)]
    #[test]
    fn extract_tar_rebases_absolute_symlink_target_within_output_root() {
        let tar_data = build_raw_tar(&[
            RawTarEntry::dir("pkg/"),
            RawTarEntry::dir("pkg/lib/"),
            RawTarEntry::file("pkg/lib/libc.so", b"libc"),
            RawTarEntry::symlink("pkg/lib/ld-musl-x86_64.so.1", "/lib/libc.so"),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");

        extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap();
        let link_path = out.join("lib/ld-musl-x86_64.so.1");
        assert!(link_path.is_symlink(), "rebased target must stay a symlink");
        assert_eq!(std::fs::read_link(&link_path).unwrap(), PathBuf::from("libc.so"));
    }

    #[cfg(unix)]
    #[test]
    fn extract_tar_rejects_symlink_parent_escape() {
        let tar_data = build_raw_tar(&[
            RawTarEntry::dir("pkg/"),
            RawTarEntry::symlink("pkg/out", "../escape"),
            RawTarEntry::file("pkg/out/file.txt", b"escaped"),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");
        let escape_dir = tmp.path().join("escape");
        std::fs::create_dir_all(&escape_dir).unwrap();

        let err = extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("escapes output dir") || msg.contains("symlink"), "expected escape error: {msg}");
        assert!(!escape_dir.join("file.txt").exists(), "write through symlink must not escape");
    }

    #[cfg(unix)]
    #[test]
    fn extract_tar_rejects_hardlink_escape() {
        let tar_data = build_raw_tar(&[
            RawTarEntry::dir("pkg/"),
            RawTarEntry::hardlink("pkg/copied.txt", "../escape-target.txt"),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");
        let escape_target = tmp.path().join("escape-target.txt");
        std::fs::write(&escape_target, "outside").unwrap();

        let err = extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap_err();
        assert!(err.to_string().contains("escapes output dir"), "expected escape error: {err}");
        assert!(!out.join("copied.txt").exists(), "hardlink escape must not materialize");
    }

    #[cfg(unix)]
    #[test]
    fn extract_tar_rejects_hardlink_target_escape_via_existing_symlink_with_dotdot() {
        let tar_data = build_raw_tar(&[
            RawTarEntry::dir("pkg/"),
            RawTarEntry::dir("pkg/base/"),
            RawTarEntry::dir("pkg/sub/"),
            RawTarEntry::file("pkg/escape.txt", b"inside"),
            RawTarEntry::symlink("pkg/base/alias", "../sub"),
            RawTarEntry::hardlink("pkg/copied.txt", "base/alias/../../escape.txt"),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");

        let err = extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap_err();
        assert!(err.to_string().contains("escapes output dir"), "expected escape error: {err}");
        assert!(!out.join("copied.txt").exists(), "escaping hardlink target must not materialize");
    }

    #[cfg(unix)]
    #[test]
    fn extract_tar_preserves_hardlink_target_via_existing_symlink_with_dotdot() {
        let tar_data = build_raw_tar(&[
            RawTarEntry::dir("pkg/"),
            RawTarEntry::dir("pkg/base/"),
            RawTarEntry::dir("pkg/sub/"),
            RawTarEntry::file("pkg/real.txt", b"payload"),
            RawTarEntry::symlink("pkg/base/alias", "../sub"),
            RawTarEntry::hardlink("pkg/copied.txt", "base/alias/../real.txt"),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");

        extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap();
        assert_eq!(std::fs::read_to_string(out.join("copied.txt")).unwrap(), "payload");
    }

    #[cfg(unix)]
    #[test]
    fn extract_tar_rejects_hardlink_absolute_target() {
        let tar_data = build_raw_tar(&[
            RawTarEntry::dir("pkg/"),
            RawTarEntry::hardlink("pkg/evil.txt", "/etc/shadow"),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");

        let err = extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap_err();
        assert!(err.to_string().contains("absolute target"), "expected absolute target error: {err}");
    }

    #[cfg(unix)]
    #[test]
    fn extract_tar_rejects_symlink_cycle_parent() {
        let tar_data = build_raw_tar(&[
            RawTarEntry::dir("pkg/"),
            RawTarEntry::symlink("pkg/a", "b"),
            RawTarEntry::symlink("pkg/b", "a"),
            RawTarEntry::file("pkg/a/file.txt", b"cycle"),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");

        let err = extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap_err();
        let message = err.to_string();
        assert!(message.contains("symlink expansions") || message.contains("escapes output dir"));
        assert!(!out.join("a/file.txt").exists(), "cyclic parent symlink must not materialize writes");
    }

    #[cfg(unix)]
    #[test]
    fn extract_tar_rejects_symlink_target_escape_after_parent_resolution() {
        let tar_data = build_raw_tar(&[
            RawTarEntry::dir("pkg/"),
            RawTarEntry::dir("pkg/a/"),
            RawTarEntry::symlink("pkg/a/link", ".."),
            RawTarEntry::symlink("pkg/a/link/s", "../escape_target"),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");

        let err = extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap_err();
        let message = err.to_string();
        assert!(message.contains("escapes output dir") || message.contains("absolute"));
        assert!(!out.join("s").exists(), "escaping symlink must not materialize at resolved root");
    }

    #[cfg(unix)]
    #[test]
    fn extract_tar_rejects_symlink_target_escape_via_existing_symlink_with_dotdot() {
        let tar_data = build_raw_tar(&[
            RawTarEntry::dir("pkg/"),
            RawTarEntry::dir("pkg/base/"),
            RawTarEntry::dir("pkg/sub/"),
            RawTarEntry::symlink("pkg/base/alias", "../sub"),
            RawTarEntry::symlink("pkg/base/evil.txt", "alias/../../escape.txt"),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");
        std::fs::write(tmp.path().join("escape.txt"), "outside").unwrap();

        let err = extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap_err();
        let message = err.to_string();
        assert!(message.contains("escapes output dir") || message.contains("absolute"));
        assert!(
            !out.join("base/evil.txt").exists(),
            "escaping symlink target must not materialize through existing symlink"
        );
    }

    #[cfg(unix)]
    #[test]
    fn extract_tar_preserves_symlink_target_via_existing_symlink_with_dotdot() {
        let tar_data = build_raw_tar(&[
            RawTarEntry::dir("pkg/"),
            RawTarEntry::dir("pkg/base/"),
            RawTarEntry::dir("pkg/sub/"),
            RawTarEntry::file("pkg/real.txt", b"payload"),
            RawTarEntry::symlink("pkg/base/alias", "../sub"),
            RawTarEntry::symlink("pkg/base/ok.txt", "alias/../real.txt"),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");

        extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap();
        let link_path = out.join("base/ok.txt");
        assert!(link_path.is_symlink());
        assert_eq!(std::fs::read_link(&link_path).unwrap(), Path::new("alias/../real.txt"));
        assert_eq!(std::fs::read_to_string(&link_path).unwrap(), "payload");
    }

    #[cfg(unix)]
    #[test]
    fn extract_tar_preserves_multi_hop_in_tree_symlink_parent() {
        let tar_data = build_raw_tar(&[
            RawTarEntry::dir("pkg/"),
            RawTarEntry::dir("pkg/real/"),
            RawTarEntry::symlink("pkg/hop1", "real"),
            RawTarEntry::symlink("pkg/hop2", "hop1"),
            RawTarEntry::file("pkg/hop2/file.txt", b"chain"),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");

        extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap();
        assert_eq!(std::fs::read_to_string(out.join("real/file.txt")).unwrap(), "chain");
    }

    #[cfg(unix)]
    #[test]
    fn extract_tar_preserves_symlink_chain_at_expansion_boundary() {
        let mut entries = vec![RawTarEntry::dir("pkg/"), RawTarEntry::dir("pkg/real/")];
        for index in 1_u32..64_u32 {
            let path_string = format!("pkg/h{index}");
            let target_string = format!("h{}", index + 1);
            entries.push(RawTarEntry::symlink(leak_test_str(path_string), leak_test_str(target_string)));
        }
        entries.push(RawTarEntry::symlink("pkg/h64", "real"));
        entries.push(RawTarEntry::file("pkg/h1/file.txt", b"boundary"));
        let tar_data = build_raw_tar(&entries);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");

        extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap();
        assert_eq!(std::fs::read_to_string(out.join("real/file.txt")).unwrap(), "boundary");
    }

    #[cfg(unix)]
    #[test]
    fn extract_tar_rejects_symlink_chain_past_expansion_boundary() {
        let mut entries = vec![RawTarEntry::dir("pkg/"), RawTarEntry::dir("pkg/real/")];
        for index in 1_u32..65_u32 {
            let path_string = format!("pkg/z{index}");
            let target_string = format!("z{}", index + 1);
            entries.push(RawTarEntry::symlink(leak_test_str(path_string), leak_test_str(target_string)));
        }
        entries.push(RawTarEntry::symlink("pkg/z65", "real"));
        entries.push(RawTarEntry::file("pkg/z1/file.txt", b"too-deep"));
        let tar_data = build_raw_tar(&entries);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");

        let err = extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap_err();
        let message = err.to_string();
        assert!(message.contains("symlink expansions") || message.contains("escapes output dir"));
        assert!(!out.join("z1/file.txt").exists(), "over-limit linear chain must not materialize writes");
    }

    #[cfg(unix)]
    #[test]
    fn extract_tar_preserves_symlink_creation_under_boundary_chain() {
        let mut entries = vec![RawTarEntry::dir("pkg/"), RawTarEntry::dir("pkg/real/")];
        for index in 1_u32..64_u32 {
            let path_string = format!("pkg/s{index}");
            let target_string = format!("s{}", index + 1);
            entries.push(RawTarEntry::symlink(leak_test_str(path_string), leak_test_str(target_string)));
        }
        entries.push(RawTarEntry::symlink("pkg/s64", "real"));
        entries.push(RawTarEntry::file("pkg/real/file.txt", b"payload"));
        entries.push(RawTarEntry::symlink("pkg/s1/link.txt", "file.txt"));
        let tar_data = build_raw_tar(&entries);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");

        extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap();
        let link_path = out.join("real/link.txt");
        assert!(link_path.is_symlink());
        assert_eq!(std::fs::read_link(&link_path).unwrap(), Path::new("file.txt"));
        assert_eq!(std::fs::read_to_string(&link_path).unwrap(), "payload");
    }

    #[cfg(unix)]
    #[test]
    fn extract_tar_preserves_directory_creation_under_boundary_chain() {
        let mut entries = vec![RawTarEntry::dir("pkg/"), RawTarEntry::dir("pkg/real/")];
        for index in 1_u32..64_u32 {
            let path_string = format!("pkg/d{index}");
            let target_string = format!("d{}", index + 1);
            entries.push(RawTarEntry::symlink(leak_test_str(path_string), leak_test_str(target_string)));
        }
        entries.push(RawTarEntry::symlink("pkg/d64", "real"));
        entries.push(RawTarEntry::dir("pkg/d1/nested/"));
        let tar_data = build_raw_tar(&entries);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");

        extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap();
        assert!(out.join("real/nested").is_dir());
    }

    #[cfg(unix)]
    #[test]
    fn extract_tar_preserves_hardlink_creation_under_boundary_chain() {
        let mut entries = vec![RawTarEntry::dir("pkg/"), RawTarEntry::dir("pkg/real/")];
        for index in 1_u32..64_u32 {
            let path_string = format!("pkg/k{index}");
            let target_string = format!("k{}", index + 1);
            entries.push(RawTarEntry::symlink(leak_test_str(path_string), leak_test_str(target_string)));
        }
        entries.push(RawTarEntry::symlink("pkg/k64", "real"));
        entries.push(RawTarEntry::file("pkg/real/file.txt", b"payload"));
        entries.push(RawTarEntry::hardlink("pkg/k1/copy.txt", "real/file.txt"));
        let tar_data = build_raw_tar(&entries);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");

        extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap();
        assert_eq!(std::fs::read_to_string(out.join("real/copy.txt")).unwrap(), "payload");
    }

    #[cfg(unix)]
    #[test]
    fn extract_tar_preserves_hardlink_target_traversal_under_boundary_chain() {
        let mut entries = vec![RawTarEntry::dir("pkg/"), RawTarEntry::dir("pkg/real/")];
        for index in 1_u32..64_u32 {
            let path_string = format!("pkg/t{index}");
            let target_string = format!("t{}", index + 1);
            entries.push(RawTarEntry::symlink(leak_test_str(path_string), leak_test_str(target_string)));
        }
        entries.push(RawTarEntry::symlink("pkg/t64", "real"));
        entries.push(RawTarEntry::file("pkg/real/file.txt", b"payload"));
        entries.push(RawTarEntry::hardlink("pkg/copy.txt", "t1/file.txt"));
        let tar_data = build_raw_tar(&entries);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");

        extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap();
        assert_eq!(std::fs::read_to_string(out.join("copy.txt")).unwrap(), "payload");
    }

    #[cfg(unix)]
    #[test]
    fn extract_tar_preserves_symlink_target_traversal_under_short_chain() {
        let mut entries = vec![RawTarEntry::dir("pkg/"), RawTarEntry::dir("pkg/real/")];
        for index in 1_u32..16_u32 {
            let path_string = format!("pkg/u{index}");
            let target_string = format!("u{}", index + 1);
            entries.push(RawTarEntry::symlink(leak_test_str(path_string), leak_test_str(target_string)));
        }
        entries.push(RawTarEntry::symlink("pkg/u16", "real"));
        entries.push(RawTarEntry::file("pkg/real/file.txt", b"payload"));
        entries.push(RawTarEntry::symlink("pkg/link.txt", "u1/file.txt"));
        let tar_data = build_raw_tar(&entries);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");

        extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap();
        let link_path = out.join("link.txt");
        assert!(link_path.is_symlink());
        assert_eq!(std::fs::read_link(&link_path).unwrap(), Path::new("u1/file.txt"));
        assert_eq!(std::fs::read_to_string(&link_path).unwrap(), "payload");
    }

    #[cfg(unix)]
    #[test]
    fn extract_tar_preserves_hardlink_target_normalization_under_boundary_chain() {
        let mut entries = vec![RawTarEntry::dir("pkg/"), RawTarEntry::dir("pkg/real/")];
        for index in 1_u32..64_u32 {
            let path_string = format!("pkg/n{index}");
            let target_string = format!("n{}", index + 1);
            entries.push(RawTarEntry::symlink(leak_test_str(path_string), leak_test_str(target_string)));
        }
        entries.push(RawTarEntry::symlink("pkg/n64", "real"));
        entries.push(RawTarEntry::file("pkg/real/file.txt", b"payload"));
        entries.push(RawTarEntry::hardlink("pkg/copy-normalized.txt", "n1/sub/../file.txt"));
        let tar_data = build_raw_tar(&entries);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");

        extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap();
        assert_eq!(std::fs::read_to_string(out.join("copy-normalized.txt")).unwrap(), "payload");
    }

    #[cfg(unix)]
    #[test]
    fn extract_tar_preserves_hardlink_via_symlink_target() {
        let tar_data = build_raw_tar(&[
            RawTarEntry::dir("pkg/"),
            RawTarEntry::file("pkg/real.txt", b"content"),
            RawTarEntry::symlink("pkg/link.txt", "real.txt"),
            RawTarEntry::hardlink("pkg/copy.txt", "link.txt"),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");

        extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap();
        assert_eq!(std::fs::read_to_string(out.join("copy.txt")).unwrap(), "content");
    }

    #[cfg(unix)]
    #[test]
    fn extract_tar_preserves_forward_hardlink_via_symlink_target() {
        let tar_data = build_raw_tar(&[
            RawTarEntry::dir("pkg/"),
            RawTarEntry::hardlink("pkg/copy.txt", "link.txt"),
            RawTarEntry::symlink("pkg/link.txt", "real.txt"),
            RawTarEntry::file("pkg/real.txt", b"content"),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");

        extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap();
        assert_eq!(std::fs::read_to_string(out.join("copy.txt")).unwrap(), "content");
    }

    #[cfg(unix)]
    #[test]
    fn extract_tar_preserves_forward_prefixed_hardlink_via_symlink_target() {
        let tar_data = build_raw_tar(&[
            RawTarEntry::dir("pkg/"),
            RawTarEntry::hardlink("pkg/copy.txt", "pkg/link.txt"),
            RawTarEntry::symlink("pkg/link.txt", "real.txt"),
            RawTarEntry::file("pkg/real.txt", b"content"),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");

        extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap();
        assert_eq!(std::fs::read_to_string(out.join("copy.txt")).unwrap(), "content");
    }

    #[cfg(unix)]
    #[test]
    fn extract_tar_preserves_chained_prefixed_hardlink_via_symlink_target() {
        let tar_data = build_raw_tar(&[
            RawTarEntry::dir("pkg/"),
            RawTarEntry::hardlink("pkg/copy-a.txt", "pkg/copy-b.txt"),
            RawTarEntry::hardlink("pkg/copy-b.txt", "pkg/link.txt"),
            RawTarEntry::symlink("pkg/link.txt", "real.txt"),
            RawTarEntry::file("pkg/real.txt", b"content"),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");

        extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap();
        assert_eq!(std::fs::read_to_string(out.join("copy-a.txt")).unwrap(), "content");
        assert_eq!(std::fs::read_to_string(out.join("copy-b.txt")).unwrap(), "content");
    }

    #[test]
    fn extract_tar_preserves_basic_directory_entry() {
        let tar_data = build_raw_tar(&[RawTarEntry::dir("pkg/"), RawTarEntry::dir("pkg/dir/")]);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");

        extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap();
        assert!(out.join("dir").is_dir());
    }

    #[cfg(unix)]
    #[test]
    fn extract_tar_preserves_forward_reference_hardlink() {
        let tar_data = build_raw_tar(&[
            RawTarEntry::dir("pkg/"),
            RawTarEntry::hardlink("pkg/copy.txt", "real.txt"),
            RawTarEntry::file("pkg/real.txt", b"payload"),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");

        extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap();
        assert_eq!(std::fs::read_to_string(out.join("copy.txt")).unwrap(), "payload");
    }

    #[cfg(unix)]
    #[test]
    fn extract_tar_preserves_forward_hardlink_overridden_by_later_regular() {
        let tar_data = build_raw_tar(&[
            RawTarEntry::dir("pkg/"),
            RawTarEntry::hardlink("pkg/copy.txt", "real.txt"),
            RawTarEntry::file("pkg/real.txt", b"target"),
            RawTarEntry::file("pkg/copy.txt", b"override"),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");

        extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap();
        assert_eq!(std::fs::read_to_string(out.join("copy.txt")).unwrap(), "override");
    }

    #[cfg(unix)]
    #[test]
    fn extract_tar_preserves_chained_forward_reference_hardlink() {
        let tar_data = build_raw_tar(&[
            RawTarEntry::dir("pkg/"),
            RawTarEntry::hardlink("pkg/copy-a.txt", "copy-b.txt"),
            RawTarEntry::hardlink("pkg/copy-b.txt", "real.txt"),
            RawTarEntry::file("pkg/real.txt", b"payload"),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");

        extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap();
        assert_eq!(std::fs::read_to_string(out.join("copy-a.txt")).unwrap(), "payload");
        assert_eq!(std::fs::read_to_string(out.join("copy-b.txt")).unwrap(), "payload");
    }

    #[cfg(unix)]
    #[test]
    fn extract_tar_preserves_prefixed_hardlink_target() {
        let tar_data = build_raw_tar(&[
            RawTarEntry::dir("pkg/"),
            RawTarEntry::file("pkg/real.txt", b"payload"),
            RawTarEntry::hardlink("pkg/copy.txt", "pkg/real.txt"),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");

        extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap();
        assert_eq!(std::fs::read_to_string(out.join("copy.txt")).unwrap(), "payload");
    }

    #[cfg(unix)]
    #[test]
    fn extract_tar_preserves_forward_prefixed_hardlink_target() {
        let tar_data = build_raw_tar(&[
            RawTarEntry::dir("pkg/"),
            RawTarEntry::hardlink("pkg/copy.txt", "pkg/real.txt"),
            RawTarEntry::file("pkg/real.txt", b"payload"),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");

        extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap();
        assert_eq!(std::fs::read_to_string(out.join("copy.txt")).unwrap(), "payload");
    }

    #[cfg(unix)]
    #[test]
    fn extract_tar_preserves_chained_prefixed_hardlink_target() {
        let tar_data = build_raw_tar(&[
            RawTarEntry::dir("pkg/"),
            RawTarEntry::hardlink("pkg/copy-a.txt", "pkg/copy-b.txt"),
            RawTarEntry::hardlink("pkg/copy-b.txt", "pkg/real.txt"),
            RawTarEntry::file("pkg/real.txt", b"payload"),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");

        extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap();
        assert_eq!(std::fs::read_to_string(out.join("copy-a.txt")).unwrap(), "payload");
        assert_eq!(std::fs::read_to_string(out.join("copy-b.txt")).unwrap(), "payload");
    }

    #[test]
    fn extract_tar_preserves_basic_direct_hardlink() {
        let tar_data = build_raw_tar(&[
            RawTarEntry::dir("pkg/"),
            RawTarEntry::file("pkg/real.txt", b"payload"),
            RawTarEntry::hardlink("pkg/copy.txt", "real.txt"),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");

        extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap();
        assert_eq!(std::fs::read_to_string(out.join("copy.txt")).unwrap(), "payload");
    }

    #[cfg(unix)]
    #[test]
    fn extract_tar_preserves_in_tree_relative_dotdot_hardlink() {
        let tar_data = build_raw_tar(&[
            RawTarEntry::dir("pkg/"),
            RawTarEntry::file("pkg/real.txt", b"content"),
            RawTarEntry::dir("pkg/sub/"),
            RawTarEntry::hardlink("pkg/sub/copy.txt", "sub/../real.txt"),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");

        extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap();
        assert_eq!(std::fs::read_to_string(out.join("real.txt")).unwrap(), "content");
        assert_eq!(std::fs::read_to_string(out.join("sub/copy.txt")).unwrap(), "content");
    }

    #[cfg(unix)]
    #[test]
    fn extract_tar_preserves_valid_in_tree_symlink() {
        let tar_data = build_raw_tar(&[
            RawTarEntry::dir("pkg/"),
            RawTarEntry::file("pkg/real.txt", b"content"),
            RawTarEntry::symlink("pkg/link.txt", "real.txt"),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");

        extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap();
        assert!(out.join("link.txt").is_symlink());
        assert_eq!(std::fs::read_link(out.join("link.txt")).unwrap(), Path::new("real.txt"));
        assert_eq!(std::fs::read_to_string(out.join("link.txt")).unwrap(), "content");
    }

    #[cfg(unix)]
    #[test]
    fn extract_tar_regular_entry_replaces_symlink() {
        let tar_data = build_raw_tar(&[
            RawTarEntry::dir("pkg/"),
            RawTarEntry::dir("pkg/sub/"),
            RawTarEntry::file("pkg/sub/real.txt", b"original"),
            RawTarEntry::symlink("pkg/link.txt", "sub/real.txt"),
            RawTarEntry::file("pkg/link.txt", b"overwrite"),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");

        extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap();
        assert!(!out.join("link.txt").is_symlink());
        assert_eq!(std::fs::read_to_string(out.join("link.txt")).unwrap(), "overwrite");
        assert_eq!(std::fs::read_to_string(out.join("sub/real.txt")).unwrap(), "original");
    }

    #[cfg(unix)]
    #[test]
    fn extract_tar_directory_entry_replaces_symlink() {
        let tar_data = build_raw_tar(&[
            RawTarEntry::dir("pkg/"),
            RawTarEntry::dir("pkg/src/"),
            RawTarEntry::file("pkg/src/data.txt", b"hello"),
            RawTarEntry::symlink("pkg/lnk", "src"),
            RawTarEntry::dir("pkg/lnk/"),
            RawTarEntry::file("pkg/lnk/own.txt", b"world"),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");

        extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap();
        assert!(!out.join("lnk").is_symlink());
        assert_eq!(std::fs::read_to_string(out.join("lnk/own.txt")).unwrap(), "world");
        assert_eq!(std::fs::read_to_string(out.join("src/data.txt")).unwrap(), "hello");
        assert!(!out.join("src/own.txt").exists());
    }

    #[cfg(unix)]
    #[test]
    fn extract_tar_preserves_directory_alias_symlink_parent() {
        let tar_data = build_raw_tar(&[
            RawTarEntry::dir("pkg/"),
            RawTarEntry::dir("pkg/usr/"),
            RawTarEntry::dir("pkg/usr/lib64/"),
            RawTarEntry::symlink("pkg/usr/lib", "lib64"),
            RawTarEntry::file("pkg/usr/lib/libfoo.so", b"payload"),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");

        extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap();
        assert_eq!(std::fs::read_to_string(out.join("usr/lib64/libfoo.so")).unwrap(), "payload");
    }

    #[cfg(unix)]
    #[test]
    fn extract_tar_preserves_in_tree_relative_dotdot_symlink() {
        // A symlink like sub/link -> ../real.txt that stays within the tree
        // must still be allowed.
        let tar_data = build_raw_tar(&[
            RawTarEntry::dir("pkg/"),
            RawTarEntry::file("pkg/real.txt", b"hello"),
            RawTarEntry::dir("pkg/sub/"),
            RawTarEntry::symlink("pkg/sub/link.txt", "../real.txt"),
        ]);
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");

        extract_tar(io::Cursor::new(tar_data), out.to_str().unwrap()).unwrap();
        assert!(out.join("sub/link.txt").is_symlink());
        assert_eq!(std::fs::read_link(out.join("sub/link.txt")).unwrap(), Path::new("../real.txt"));
        assert_eq!(std::fs::read_to_string(out.join("sub/link.txt")).unwrap(), "hello");
    }
}
