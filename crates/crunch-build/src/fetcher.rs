//! Builtin fetcher execution: download, extract, and verify.
//!
//! When a derivation has `builder = "builtin:fetchurl"`, the build
//! orchestrator bypasses the sandbox and calls into this module.
//! Adapted from snix-redox's `fetchers.rs`.

use std::io::{self, Read};
use std::path::Path;

use nix_compat::derivation::Derivation;
use nix_compat::nixhash::{CAHash, HashAlgo, NixHash};
use tracing::info;
use url::Url;

/// Maximum download size: 4 GiB. Prevents unbounded memory/disk use
/// from a misbehaving or malicious server.
const MAX_DOWNLOAD_BYTES: u64 = 4 * 1024 * 1024 * 1024;

/// Maximum number of entries to extract from a tar archive.
/// Prevents zip-bomb style attacks and accidental extraction of
/// enormous archives.
const MAX_TAR_ENTRIES: u32 = 500_000;

// ── Fetch description ──────────────────────────────────────────────────

/// What kind of fetch to perform, parsed from the derivation environment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fetch {
    /// Download a single file. Hash is over raw bytes.
    Url {
        url: Url,
        exp_hash: Option<NixHash>,
    },
    /// Download a tarball, decompress, extract, strip top-level dir.
    /// Hash is NAR of the unpacked tree.
    Tarball {
        url: Url,
        exp_nar_sha256: Option<[u8; 32]>,
    },
    /// Download a NAR archive and extract it.
    Nar {
        url: Url,
        hash: NixHash,
    },
    /// Download a file and mark it executable.
    Executable {
        url: Url,
        hash: NixHash,
    },
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
    NotAFetcher(String),

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

    #[error("git not found in PATH — install git to use fetchGit")]
    GitNotFound,

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
    if !is_builtin_fetcher(derivation) {
        return Err(FetchError::NotAFetcher(derivation.builder.clone()));
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

    let unpack = env.get("unpack").is_some_and(|v| v == "1");
    let executable = env.get("executable").is_some_and(|v| v == "1");

    // Determine fetch variant from derivation's ca_hash and flags.
    let ca_hash = derivation
        .outputs
        .get("out")
        .and_then(|o| o.ca_hash.as_ref());

    match ca_hash {
        Some(CAHash::Flat(hash)) if executable => Ok(Fetch::Executable {
            url,
            hash: hash.clone(),
        }),
        Some(CAHash::Flat(hash)) => Ok(Fetch::Url {
            url,
            exp_hash: Some(hash.clone()),
        }),
        Some(CAHash::Nar(hash)) if unpack => Ok(Fetch::Tarball {
            url,
            exp_nar_sha256: match hash {
                NixHash::Sha256(d) => Some(*d),
                _ => None,
            },
        }),
        Some(CAHash::Nar(hash)) => {
            if unpack {
                Ok(Fetch::Tarball {
                    url,
                    exp_nar_sha256: match hash {
                        NixHash::Sha256(d) => Some(*d),
                        _ => None,
                    },
                })
            } else {
                Ok(Fetch::Nar {
                    url,
                    hash: hash.clone(),
                })
            }
        }
        None => {
            // No ca_hash — treat as plain URL fetch without hash verification.
            if unpack {
                Ok(Fetch::Tarball {
                    url,
                    exp_nar_sha256: None,
                })
            } else {
                Ok(Fetch::Url {
                    url,
                    exp_hash: None,
                })
            }
        }
        Some(CAHash::Text(_)) => Err(FetchError::NotFixedOutput),
    }
}

fn env_str(
    env: &std::collections::BTreeMap<String, bstr::BString>,
    key: &str,
) -> Option<String> {
    env.get(key).and_then(|v| String::from_utf8(Vec::from(v.clone())).ok())
}

fn extract_expected_hash(derivation: &Derivation) -> Option<NixHash> {
    derivation
        .outputs
        .get("out")
        .and_then(|o| o.ca_hash.as_ref())
        .and_then(|ca| match ca {
            CAHash::Flat(h) | CAHash::Nar(h) => Some(h.clone()),
            CAHash::Text(_) => None,
        })
}

// ── Fetch execution ────────────────────────────────────────────────────

/// Execute a fetch: download the resource and write it to `out_path`.
///
/// This is a blocking (sync) function — call from `spawn_blocking`.
pub fn fetch_to_store(fetch: &Fetch, out_path: &str) -> Result<(), FetchError> {
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
pub fn verify_flat_hash(
    out_path: &str,
    expected: &NixHash,
    name: &str,
) -> Result<(), FetchError> {
    use sha2::Digest;

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
        // Clean up bad output.
        let _ = std::fs::remove_file(out_path);
        let _ = std::fs::remove_dir_all(out_path);
        return Err(FetchError::HashMismatch {
            name: name.to_string(),
            expected: nix_hash_to_sri(expected),
            actual: format!(
                "{}-{}",
                hash_algo_prefix(expected.algo()),
                data_encoding::BASE64.encode(&actual_bytes),
            ),
        });
    }

    Ok(())
}

/// Format a NixHash as an SRI string (e.g., "sha256-base64...").
pub fn nix_hash_to_sri(hash: &NixHash) -> String {
    format!(
        "{}-{}",
        hash_algo_prefix(hash.algo()),
        data_encoding::BASE64.encode(hash.digest_as_bytes()),
    )
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
fn fetch_flat(url: &str, out: &str) -> Result<(), FetchError> {
    let reader: Box<dyn Read + Send> = open_url_reader(url)?;
    let mut limited = reader.take(MAX_DOWNLOAD_BYTES);
    let mut file = std::fs::File::create(out)?;
    let bytes_written = io::copy(&mut limited, &mut file)?;
    debug_assert!(bytes_written <= MAX_DOWNLOAD_BYTES);
    Ok(())
}

/// Download a tarball, decompress, and extract to a directory.
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
        let resp = fetch_agent().get(url)
            .call()
            .map_err(|e| FetchError::HttpError {
                url: url.to_string(),
                reason: e.to_string(),
            })?;
        Ok(Box::new(resp.into_body().into_reader()))
    }
}

// ── Decompression ──────────────────────────────────────────────────────

/// Select a decompressor based on URL suffix. Falls back to gzip magic
/// detection, then raw read.
pub fn decompress_reader(
    url: &str,
    reader: impl Read + Send + 'static,
) -> Result<Box<dyn Read + Send>, FetchError> {
    if url.ends_with(".gz") || url.ends_with(".tgz") {
        Ok(Box::new(flate2::read::GzDecoder::new(reader)))
    } else if url.ends_with(".xz") || url.ends_with(".txz") {
        let mut input = io::BufReader::new(reader);
        let mut output = Vec::new();
        lzma_rs::xz_decompress(&mut input, &mut output)
            .map_err(|e| FetchError::DecompressError(format!("xz: {e}")))?;
        Ok(Box::new(io::Cursor::new(output)))
    } else if url.ends_with(".bz2") || url.ends_with(".tbz2") {
        Ok(Box::new(bzip2_rs::DecoderReader::new(reader)))
    } else if url.ends_with(".zst") || url.ends_with(".zstd") {
        Ok(Box::new(
            ruzstd::decoding::StreamingDecoder::new(reader)
                .map_err(|e| FetchError::DecompressError(format!("zstd: {e}")))?,
        ))
    } else if url.ends_with(".tar") || url.ends_with(".nar") {
        Ok(Box::new(reader))
    } else {
        // Unknown suffix — try gzip magic, fall back to raw.
        let mut compressed = Vec::new();
        io::BufReader::new(reader)
            .read_to_end(&mut compressed)?;
        if compressed.len() >= 2 && compressed[0] == 0x1f && compressed[1] == 0x8b {
            Ok(Box::new(flate2::read::GzDecoder::new(io::Cursor::new(
                compressed,
            ))))
        } else {
            Ok(Box::new(io::Cursor::new(compressed)))
        }
    }
}

// ── Tar extraction ─────────────────────────────────────────────────────

/// Extract a tar archive to a directory, stripping the top-level path
/// component (GitHub-style tarballs where all entries are under
/// `project-v1.0/`).
pub fn extract_tar<R: Read>(reader: R, out: &str) -> Result<(), FetchError> {
    let out_path = Path::new(out);
    std::fs::create_dir_all(out_path)?;

    let mut archive = tar::Archive::new(reader);
    archive.set_preserve_mtime(false);

    let mut prefix_to_strip: Option<std::path::PathBuf> = None;
    let mut entry_count: u32 = 0;

    for entry_result in archive.entries().map_err(|e| FetchError::TarError(e.to_string()))? {
        // Tiger Style: fixed limit on tar entries.
        entry_count = entry_count.saturating_add(1);
        if entry_count > MAX_TAR_ENTRIES {
            return Err(FetchError::TarError(format!(
                "tar archive exceeds {MAX_TAR_ENTRIES} entries"
            )));
        }
        let mut entry = entry_result.map_err(|e| FetchError::TarError(e.to_string()))?;
        let entry_path = entry
            .path()
            .map_err(|e| FetchError::TarError(e.to_string()))?
            .into_owned();

        // Determine prefix from first non-empty entry.
        if prefix_to_strip.is_none() {
            if let Some(first_component) = entry_path.components().next() {
                prefix_to_strip = Some(std::path::PathBuf::from(first_component.as_os_str()));
            }
        }

        // Strip the top-level prefix.
        let relative = match &prefix_to_strip {
            Some(pfx) => match entry_path.strip_prefix(pfx) {
                Ok(stripped) => stripped.to_path_buf(),
                Err(_) => entry_path.clone(),
            },
            None => entry_path.clone(),
        };

        // Skip the top-level directory entry itself.
        if relative.as_os_str().is_empty() || relative == Path::new(".") {
            continue;
        }

        // Reject path traversal: no `..` components allowed.
        for component in relative.components() {
            if matches!(component, std::path::Component::ParentDir) {
                return Err(FetchError::TarError(format!(
                    "tar entry contains '..' path traversal: {}",
                    relative.display()
                )));
            }
        }

        let dest = out_path.join(&relative);

        // Belt-and-suspenders: verify dest is inside out_path.
        // canonicalize isn't usable (dest doesn't exist yet), so check
        // the prefix after join. Components() already rejected '..'.
        debug_assert!(
            dest.starts_with(out_path),
            "dest {dest:?} escapes out_path {out_path:?}"
        );

        match entry.header().entry_type() {
            tar::EntryType::Regular | tar::EntryType::GNUSparse => {
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut file = std::fs::File::create(&dest)?;
                io::copy(&mut entry, &mut file)?;

                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(mode) = entry.header().mode() {
                        let _ = std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(mode));
                    }
                }
            }
            tar::EntryType::Directory => {
                std::fs::create_dir_all(&dest)?;
            }
            tar::EntryType::Symlink => {
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                #[cfg(unix)]
                if let Some(target) = entry
                    .link_name()
                    .map_err(|e| FetchError::TarError(e.to_string()))?
                {
                    // Validate symlink target doesn't escape the output dir.
                    let target_path = target.as_ref();
                    let effective_target = if target_path.is_absolute() {
                        // Absolute symlinks in tarballs are self-referential
                        // (e.g., /lib/libc.so inside a toolchain tarball).
                        // Strip the tarball prefix and rewrite as relative
                        // to the output directory.
                        match &prefix_to_strip {
                            Some(pfx) => {
                                let stripped = target_path
                                    .strip_prefix("/")
                                    .unwrap_or(target_path)
                                    .strip_prefix(pfx)
                                    .unwrap_or(
                                        target_path.strip_prefix("/").unwrap_or(target_path)
                                    );
                                stripped.to_path_buf()
                            }
                            None => target_path
                                .strip_prefix("/")
                                .unwrap_or(target_path)
                                .to_path_buf(),
                        }
                    } else {
                        target_path.to_path_buf()
                    };
                    // Check the effective target doesn't escape via '..'.
                    let resolved = dest.parent().unwrap_or(out_path).join(&effective_target);
                    let mut depth: i32 = 0;
                    for comp in resolved.components() {
                        match comp {
                            std::path::Component::ParentDir => depth -= 1,
                            std::path::Component::Normal(_) => depth += 1,
                            _ => {}
                        }
                        if depth < 0 {
                            return Err(FetchError::TarError(format!(
                                "tar symlink escapes output dir: {} -> {}",
                                relative.display(),
                                target_path.display()
                            )));
                        }
                    }
                    std::os::unix::fs::symlink(&effective_target, &dest)?;
                }
            }
            tar::EntryType::Link => {
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                if let Some(link_target) = entry
                    .link_name()
                    .map_err(|e| FetchError::TarError(e.to_string()))?
                {
                    let stripped_target = match &prefix_to_strip {
                        Some(pfx) => link_target
                            .strip_prefix(pfx.as_path())
                            .unwrap_or(link_target.as_ref())
                            .to_path_buf(),
                        None => link_target.into_owned(),
                    };
                    let target_path = out_path.join(&stripped_target);
                    if target_path.exists() {
                        std::fs::copy(&target_path, &dest)?;
                    }
                }
            }
            _ => {
                // Skip pax headers, etc.
            }
        }
    }

    Ok(())
}

// ── Git fetch ──────────────────────────────────────────────────────────

/// Clone a git repository and checkout a specific revision.
///
/// Shells out to the `git` binary. The `.git` directory is stripped.
fn fetch_git(url: &str, rev: &str, out: &str) -> Result<(), FetchError> {
    // Cache check: if output already exists, skip.
    if Path::new(out).exists() {
        info!(out = out, "using cached git fetch");
        return Ok(());
    }

    let git = find_git()?;

    let tmp_parent = Path::new(out)
        .parent()
        .unwrap_or(Path::new("/tmp"));
    std::fs::create_dir_all(tmp_parent)?;
    let tmp_bare = format!("{out}.git-bare-tmp");
    let _ = std::fs::remove_dir_all(&tmp_bare);

    // Clone bare (faster — no working tree).
    let short_rev = &rev[..rev.len().min(12)];
    info!(url = url, rev = short_rev, "git clone");
    let clone_out = std::process::Command::new(&git)
        .args(["clone", "--bare", url, &tmp_bare])
        .output()
        .map_err(|e| FetchError::GitError(format!("running git clone: {e}")))?;

    if !clone_out.status.success() {
        let stderr = String::from_utf8_lossy(&clone_out.stderr);
        let _ = std::fs::remove_dir_all(&tmp_bare);
        return Err(FetchError::GitError(format!(
            "git clone failed for '{url}':\n{stderr}"
        )));
    }

    // Checkout specific rev into output dir.
    std::fs::create_dir_all(out)?;
    let checkout_out = std::process::Command::new(&git)
        .args(["--git-dir", &tmp_bare, "--work-tree", out, "checkout", rev, "--", "."])
        .output()
        .map_err(|e| FetchError::GitError(format!("running git checkout: {e}")))?;

    if !checkout_out.status.success() {
        let stderr = String::from_utf8_lossy(&checkout_out.stderr);
        let _ = std::fs::remove_dir_all(&tmp_bare);
        let _ = std::fs::remove_dir_all(out);
        return Err(FetchError::GitError(format!(
            "git checkout failed for rev '{rev}':\n{stderr}"
        )));
    }

    // Clean up bare clone.
    let _ = std::fs::remove_dir_all(&tmp_bare);

    info!(url = url, rev = short_rev, out = out, "git fetch complete");
    Ok(())
}

/// Find the `git` binary in common locations or PATH.
fn find_git() -> Result<String, FetchError> {
    // Check well-known paths (FHS, Homebrew, NixOS).
    for path in [
        "/usr/bin/git",
        "/bin/git",
        "/usr/local/bin/git",
        "/run/current-system/sw/bin/git",
    ] {
        if Path::new(path).exists() {
            return Ok(path.to_string());
        }
    }
    // Check NixOS per-user profile paths.
    if let Ok(user) = std::env::var("USER") {
        let profile_path = format!("/etc/profiles/per-user/{user}/bin/git");
        if Path::new(&profile_path).exists() {
            return Ok(profile_path);
        }
    }
    // Fall back to scanning PATH directly.
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in path_var.split(':') {
            let candidate = Path::new(dir).join("git");
            if candidate.exists() {
                return Ok(candidate.to_string_lossy().to_string());
            }
        }
    }
    Err(FetchError::GitNotFound)
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use nix_compat::derivation::Output;
    use std::collections::BTreeMap;

    /// Build a minimal derivation with the given env, builder, and ca_hash.
    fn make_drv(
        builder: &str,
        env: Vec<(&str, &str)>,
        ca_hash: Option<CAHash>,
    ) -> Derivation {
        let mut environment: BTreeMap<String, bstr::BString> = BTreeMap::new();
        for (k, v) in env {
            environment.insert(k.to_string(), bstr::BString::from(v.as_bytes()));
        }

        let mut outputs = BTreeMap::new();
        outputs.insert(
            "out".to_string(),
            Output {
                path: None,
                ca_hash,
            },
        );

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
            vec![
                ("url", "https://example.com/src.tar.gz"),
                ("unpack", "1"),
            ],
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
            vec![
                ("url", "https://example.com/run"),
                ("executable", "1"),
            ],
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
        let drv = make_drv(
            "builtin:fetchurl",
            vec![
                ("url", "https://github.com/user/repo.git"),
                ("type", "git"),
            ],
            None,
        );
        let err = parse_fetch(&drv).unwrap_err();
        assert!(matches!(err, FetchError::MissingGitRev));
    }

    #[test]
    fn parse_fetch_not_a_fetcher() {
        let drv = make_drv("/bin/sh", vec![], None);
        let err = parse_fetch(&drv).unwrap_err();
        assert!(matches!(err, FetchError::NotAFetcher(_)));
    }

    #[test]
    fn parse_fetch_missing_url() {
        let drv = make_drv("builtin:fetchurl", vec![], None);
        let err = parse_fetch(&drv).unwrap_err();
        assert!(matches!(err, FetchError::MissingUrl));
    }

    #[test]
    fn parse_fetch_url_no_hash() {
        let drv = make_drv(
            "builtin:fetchurl",
            vec![("url", "https://example.com/file.txt")],
            None,
        );
        let fetch = parse_fetch(&drv).unwrap();
        assert!(matches!(fetch, Fetch::Url { exp_hash: None, .. }));
    }

    // ── extract_tar tests ──────────────────────────────────────────

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
        assert_eq!(
            std::fs::read_to_string(out.join("hello.txt")).unwrap(),
            "hello world"
        );
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
        decompress_reader("foo.tar", io::Cursor::new(data.to_vec()))
            .unwrap()
            .read_to_end(&mut out)
            .unwrap();
        assert_eq!(out, data);
    }

    #[test]
    fn decompress_gzip() {
        use flate2::write::GzEncoder;
        use std::io::Write;

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
        use flate2::write::GzEncoder;
        use std::io::Write;

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
}
