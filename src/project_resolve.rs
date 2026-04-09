use std::io::Read;
use std::io::{self};
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use crunch_project::GitReference;
use crunch_project::HashAlgo;
use crunch_project::HashResolutionMode;
use crunch_project::RefreshResolver;
use digest::Digest;
use nix_compat::nixhash::NixHash;
use snix_castore::Node;
use snix_castore::blobservice::MemoryBlobService;
use snix_castore::directoryservice::RedbDirectoryService;
use snix_castore::directoryservice::RedbDirectoryServiceConfig;
use snix_castore::import::fs::ingest_path;
use snix_store::nar::write_nar;
use snix_store::utils::AsyncIoBridge;
use tempfile::tempdir;

const MAX_DOWNLOAD_BYTES: u64 = 4 * 1024 * 1024 * 1024;
const MAX_TREE_BYTES: u64 = 4 * 1024 * 1024 * 1024;
const MAX_TREE_ENTRIES: u32 = 500_000;
const FETCH_CONNECT_TIMEOUT_SECS: u64 = 60;
const FETCH_READ_TIMEOUT_SECS: u64 = 600;

pub struct LiveResolver {
    project_root: PathBuf,
}

impl LiveResolver {
    pub fn new(project_root: &Path) -> Self {
        assert!(project_root.components().next().is_some(), "project root must not be empty");
        Self {
            project_root: project_root.to_path_buf(),
        }
    }
}

impl RefreshResolver for LiveResolver {
    fn resolve_git_rev(
        &self,
        repository: &str,
        reference: &GitReference,
    ) -> Result<Option<String>, crunch_project::Error> {
        Ok(Some(resolve_git_reference(repository, reference)?))
    }

    fn hash_url_content(
        &self,
        url: &str,
        algo: &HashAlgo,
        mode: HashResolutionMode,
    ) -> Result<Option<String>, crunch_project::Error> {
        let sri = match mode {
            HashResolutionMode::Flat => hash_flat_url(url, algo)?,
            HashResolutionMode::Recursive => hash_tarball_url(url, algo)?,
        };
        Ok(Some(sri))
    }

    fn hash_git_checkout(
        &self,
        repository: &str,
        rev: &str,
        algo: &HashAlgo,
    ) -> Result<Option<String>, crunch_project::Error> {
        Ok(Some(hash_git_tree(repository, rev, algo)?))
    }

    fn hash_local_file(&self, path: &str, algo: &HashAlgo) -> Result<Option<String>, crunch_project::Error> {
        let resolved = resolve_local_path(&self.project_root, path);
        Ok(Some(hash_flat_file(&resolved, algo)?))
    }
}

fn resolve_git_reference(repository: &str, reference: &GitReference) -> Result<String, crunch_project::Error> {
    match reference {
        GitReference::Rev(rev) => {
            validate_git_rev(rev)?;
            Ok(rev.clone())
        }
        GitReference::Branch(branch) => {
            let output = run_git_ls_remote(repository, &[&format!("refs/heads/{branch}")])?;
            parse_branch_output(&output, branch)
        }
        GitReference::Tag(tag) => {
            let direct = format!("refs/tags/{tag}");
            let peeled = format!("refs/tags/{tag}^{{}}");
            let output = run_git_ls_remote(repository, &[&direct, &peeled])?;
            parse_tag_output(&output, tag)
        }
    }
}

fn validate_git_rev(rev: &str) -> Result<(), crunch_project::Error> {
    if rev.len() != 40 {
        return Err(crunch_project::Error::Validation(format!("git rev must be 40 hex characters, got {rev}")));
    }
    if !rev.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(crunch_project::Error::Validation(format!("git rev must be hex, got {rev}")));
    }
    Ok(())
}

fn run_git_ls_remote(repository: &str, refs: &[&str]) -> Result<String, crunch_project::Error> {
    let git = find_git_binary()?;
    let output = Command::new(&git)
        .arg("ls-remote")
        .arg(repository)
        .args(refs)
        .output()
        .map_err(|err| crunch_project::Error::Manifest(format!("running git ls-remote for {repository}: {err}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(crunch_project::Error::Manifest(format!("git ls-remote failed for {repository}: {stderr}")));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn parse_branch_output(output: &str, branch: &str) -> Result<String, crunch_project::Error> {
    let wanted = format!("refs/heads/{branch}");
    for line in output.lines() {
        let Some((rev, reference)) = line.split_once('\t') else {
            continue;
        };
        if reference == wanted {
            validate_git_rev(rev)?;
            return Ok(rev.to_string());
        }
    }
    Err(crunch_project::Error::Manifest(format!("git branch not found: {branch}")))
}

fn parse_tag_output(output: &str, tag: &str) -> Result<String, crunch_project::Error> {
    let direct = format!("refs/tags/{tag}");
    let peeled = format!("refs/tags/{tag}^{{}}");
    let mut direct_rev: Option<String> = None;
    for line in output.lines() {
        let Some((rev, reference)) = line.split_once('\t') else {
            continue;
        };
        if reference == peeled {
            validate_git_rev(rev)?;
            return Ok(rev.to_string());
        }
        if reference == direct {
            validate_git_rev(rev)?;
            direct_rev = Some(rev.to_string());
        }
    }
    direct_rev.ok_or_else(|| crunch_project::Error::Manifest(format!("git tag not found: {tag}")))
}

fn hash_flat_url(url: &str, algo: &HashAlgo) -> Result<String, crunch_project::Error> {
    let reader = open_url_reader(url)?;
    let bounded = BoundedReader::new(reader, MAX_DOWNLOAD_BYTES, url.to_string());
    hash_reader_to_sri(bounded, algo, &format!("downloaded content for {url}"))
}

fn hash_tarball_url(url: &str, algo: &HashAlgo) -> Result<String, crunch_project::Error> {
    let reader = open_url_reader(url)?;
    let bounded = BoundedReader::new(reader, MAX_DOWNLOAD_BYTES, url.to_string());
    let unpacked = crunch_build::fetcher::decompress_reader(url, bounded)
        .map_err(|err| crunch_project::Error::Manifest(format!("decompressing {url}: {err}")))?;
    let dir = tempdir()?;
    let out = path_to_str(dir.path(), "temporary unpack directory")?;
    crunch_build::fetcher::extract_tar(unpacked, out)
        .map_err(|err| crunch_project::Error::Manifest(format!("extracting tarball {url}: {err}")))?;
    ensure_tree_within_limits(dir.path())?;
    hash_recursive_path(dir.path(), algo)
}

fn hash_git_tree(repository: &str, rev: &str, algo: &HashAlgo) -> Result<String, crunch_project::Error> {
    let git = find_git_binary()?;
    let dir = tempdir()?;
    let bare = dir.path().join("source.git");
    let checkout = dir.path().join("checkout");
    run_git_clone_bare(&git, repository, &bare)?;
    run_git_checkout_tree(&git, &bare, &checkout, rev)?;
    ensure_tree_within_limits(&checkout)?;
    hash_recursive_path(&checkout, algo)
}

fn run_git_clone_bare(git: &Path, repository: &str, bare: &Path) -> Result<(), crunch_project::Error> {
    let output = Command::new(git)
        .args(["clone", "--bare", repository])
        .arg(bare)
        .output()
        .map_err(|err| crunch_project::Error::Manifest(format!("running git clone for {repository}: {err}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(crunch_project::Error::Manifest(format!("git clone failed for {repository}: {stderr}")));
    }
    Ok(())
}

fn run_git_checkout_tree(git: &Path, bare: &Path, checkout: &Path, rev: &str) -> Result<(), crunch_project::Error> {
    std::fs::create_dir_all(checkout)?;
    let output = Command::new(git)
        .arg("--git-dir")
        .arg(bare)
        .arg("--work-tree")
        .arg(checkout)
        .args(["checkout", rev, "--", "."])
        .output()
        .map_err(|err| crunch_project::Error::Manifest(format!("running git checkout for {rev}: {err}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(crunch_project::Error::Manifest(format!("git checkout failed for {rev}: {stderr}")));
    }
    Ok(())
}

fn hash_flat_file(path: &Path, algo: &HashAlgo) -> Result<String, crunch_project::Error> {
    let file = std::fs::File::open(path)
        .map_err(|err| crunch_project::Error::Manifest(format!("opening {}: {err}", path.display())))?;
    let bounded = BoundedReader::new(io::BufReader::new(file), MAX_DOWNLOAD_BYTES, path.display().to_string());
    hash_reader_to_sri(bounded, algo, &format!("local file {}", path.display()))
}

fn hash_reader_to_sri<R: Read>(mut reader: R, algo: &HashAlgo, label: &str) -> Result<String, crunch_project::Error> {
    let mut buffer = [0u8; 64 * 1024];
    match algo {
        HashAlgo::Sha256 => {
            let mut hasher = sha2::Sha256::new();
            read_into_hasher(&mut reader, &mut buffer, &mut hasher, label)?;
            let hash: [u8; 32] = hasher.finalize().into();
            Ok(NixHash::Sha256(hash).to_sri_string())
        }
        HashAlgo::Sha512 => {
            let mut hasher = sha2::Sha512::new();
            read_into_hasher(&mut reader, &mut buffer, &mut hasher, label)?;
            let hash: [u8; 64] = hasher.finalize().into();
            Ok(NixHash::Sha512(Box::new(hash)).to_sri_string())
        }
        HashAlgo::Blake3 => {
            let mut hasher = blake3::Hasher::new();
            loop {
                let read = reader
                    .read(&mut buffer)
                    .map_err(|err| crunch_project::Error::Manifest(format!("reading {label}: {err}")))?;
                if read == 0 {
                    break;
                }
                hasher.update(&buffer[..read]);
            }
            Ok(NixHash::Blake3(*blake3::Hasher::finalize(&hasher).as_bytes()).to_sri_string())
        }
    }
}

fn read_into_hasher<R: Read, H: Digest>(
    reader: &mut R,
    buffer: &mut [u8],
    hasher: &mut H,
    label: &str,
) -> Result<(), crunch_project::Error> {
    loop {
        let read = reader
            .read(buffer)
            .map_err(|err| crunch_project::Error::Manifest(format!("reading {label}: {err}")))?;
        if read == 0 {
            return Ok(());
        }
        hasher.update(&buffer[..read]);
    }
}

fn hash_recursive_path(path: &Path, algo: &HashAlgo) -> Result<String, crunch_project::Error> {
    let root = path.to_path_buf();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|err| crunch_project::Error::Manifest(format!("creating hash runtime: {err}")))?;
    rt.block_on(async move {
        let blob_service = MemoryBlobService::default();
        let directory_service = RedbDirectoryService::new_temporary(
            "project-refresh".to_string(),
            RedbDirectoryServiceConfig::default(),
        )
        .map_err(|err| crunch_project::Error::Manifest(format!("creating temporary directory service: {err}")))?;
        let node = ingest_path::<_, _, _, &[u8]>(blob_service.clone(), directory_service.clone(), &root, None)
            .await
            .map_err(|err| crunch_project::Error::Manifest(format!("ingesting {}: {err}", root.display())))?;
        nar_hash_to_sri(&node, algo, blob_service, directory_service).await
    })
}

async fn nar_hash_to_sri(
    node: &Node,
    algo: &HashAlgo,
    blob_service: MemoryBlobService,
    directory_service: RedbDirectoryService,
) -> Result<String, crunch_project::Error> {
    let nix_hash = match algo {
        HashAlgo::Sha256 => {
            let mut hasher = sha2::Sha256::new();
            write_nar(AsyncIoBridge(&mut hasher), node, blob_service, directory_service)
                .await
                .map_err(|err| crunch_project::Error::Manifest(format!("writing NAR: {err}")))?;
            let hash: [u8; 32] = hasher.finalize().into();
            NixHash::Sha256(hash)
        }
        HashAlgo::Sha512 => {
            let mut hasher = sha2::Sha512::new();
            write_nar(AsyncIoBridge(&mut hasher), node, blob_service, directory_service)
                .await
                .map_err(|err| crunch_project::Error::Manifest(format!("writing NAR: {err}")))?;
            let hash: [u8; 64] = hasher.finalize().into();
            NixHash::Sha512(Box::new(hash))
        }
        HashAlgo::Blake3 => {
            let mut hasher = blake3::Hasher::new();
            write_nar(AsyncIoBridge(&mut hasher), node, blob_service, directory_service)
                .await
                .map_err(|err| crunch_project::Error::Manifest(format!("writing NAR: {err}")))?;
            NixHash::Blake3(*blake3::Hasher::finalize(&hasher).as_bytes())
        }
    };
    Ok(nix_hash.to_sri_string())
}

fn ensure_tree_within_limits(path: &Path) -> Result<(), crunch_project::Error> {
    let mut pending = vec![path.to_path_buf()];
    let mut entries_seen: u32 = 0;
    let mut bytes_total: u64 = 0;
    while let Some(current) = pending.pop() {
        entries_seen = entries_seen.saturating_add(1);
        if entries_seen > MAX_TREE_ENTRIES {
            return Err(crunch_project::Error::Manifest(format!(
                "materialized tree exceeds {MAX_TREE_ENTRIES} entries: {}",
                path.display()
            )));
        }
        let metadata = std::fs::symlink_metadata(&current)
            .map_err(|err| crunch_project::Error::Manifest(format!("stat {}: {err}", current.display())))?;
        if metadata.is_file() {
            bytes_total = bytes_total.saturating_add(metadata.len());
            if bytes_total > MAX_TREE_BYTES {
                return Err(crunch_project::Error::Manifest(format!(
                    "materialized tree exceeds {MAX_TREE_BYTES} bytes: {}",
                    path.display()
                )));
            }
            continue;
        }
        if !metadata.is_dir() {
            continue;
        }
        for child in std::fs::read_dir(&current)
            .map_err(|err| crunch_project::Error::Manifest(format!("read_dir {}: {err}", current.display())))?
        {
            let child = child
                .map_err(|err| crunch_project::Error::Manifest(format!("walking {}: {err}", current.display())))?;
            pending.push(child.path());
        }
    }
    Ok(())
}

fn resolve_local_path(project_root: &Path, path: &str) -> PathBuf {
    let source = Path::new(path);
    if source.is_absolute() {
        source.to_path_buf()
    } else {
        project_root.join(source)
    }
}

fn path_to_str<'a>(path: &'a Path, what: &str) -> Result<&'a str, crunch_project::Error> {
    path.to_str()
        .ok_or_else(|| crunch_project::Error::Manifest(format!("{what} is not valid UTF-8: {}", path.display())))
}

fn find_git_binary() -> Result<PathBuf, crunch_project::Error> {
    for candidate in [
        "/usr/bin/git",
        "/bin/git",
        "/usr/local/bin/git",
        "/run/current-system/sw/bin/git",
    ] {
        let path = Path::new(candidate);
        if path.exists() {
            return Ok(path.to_path_buf());
        }
    }
    if let Ok(user) = std::env::var("USER") {
        let profile = PathBuf::from(format!("/etc/profiles/per-user/{user}/bin/git"));
        if profile.exists() {
            return Ok(profile);
        }
    }
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in path_var.split(':') {
            let candidate = Path::new(dir).join("git");
            if candidate.exists() {
                return Ok(candidate);
            }
        }
    }
    Err(crunch_project::Error::Manifest("git not found in PATH".to_string()))
}

fn open_url_reader(url: &str) -> Result<Box<dyn Read + Send>, crunch_project::Error> {
    if let Some(path) = url.strip_prefix("file://") {
        let file = std::fs::File::open(path)
            .map_err(|err| crunch_project::Error::Manifest(format!("opening local file URL {url}: {err}")))?;
        return Ok(Box::new(io::BufReader::new(file)));
    }
    let response = fetch_agent()
        .get(url)
        .call()
        .map_err(|err| crunch_project::Error::Manifest(format!("downloading {url}: {err}")))?;
    Ok(Box::new(response.into_body().into_reader()))
}

fn fetch_agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(std::time::Duration::from_secs(FETCH_READ_TIMEOUT_SECS)))
        .timeout_connect(Some(std::time::Duration::from_secs(FETCH_CONNECT_TIMEOUT_SECS)))
        .build()
        .new_agent()
}

struct BoundedReader<R> {
    inner: R,
    limit_bytes: u64,
    seen_bytes: u64,
    label: String,
}

impl<R> BoundedReader<R> {
    fn new(inner: R, limit_bytes: u64, label: String) -> Self {
        assert!(limit_bytes > 0, "limit must be positive");
        assert!(!label.is_empty(), "label must not be empty");
        Self {
            inner,
            limit_bytes,
            seen_bytes: 0,
            label,
        }
    }
}

impl<R: Read> Read for BoundedReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.seen_bytes == self.limit_bytes {
            let mut extra = [0u8; 1];
            let read = self.inner.read(&mut extra)?;
            if read == 0 {
                return Ok(0);
            }
            return Err(io::Error::other(format!("download exceeds {} bytes: {}", self.limit_bytes, self.label)));
        }
        let remaining = self.limit_bytes.saturating_sub(self.seen_bytes);
        let allowed = remaining.min(buf.len() as u64) as usize;
        let read = self.inner.read(&mut buf[..allowed])?;
        self.seen_bytes = self.seen_bytes.saturating_add(read as u64);
        Ok(read)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_branch_output_returns_matching_rev() {
        let output = "0123456789abcdef0123456789abcdef01234567\trefs/heads/main\n";
        let rev = parse_branch_output(output, "main").unwrap();
        assert_eq!(rev, "0123456789abcdef0123456789abcdef01234567");
    }

    #[test]
    fn parse_tag_output_prefers_peeled_commit() {
        let output = concat!(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\trefs/tags/v1.0\n",
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\trefs/tags/v1.0^{}\n",
        );
        let rev = parse_tag_output(output, "v1.0").unwrap();
        assert_eq!(rev, "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
    }

    #[test]
    fn parse_tag_output_falls_back_to_direct_tag() {
        let output = "cccccccccccccccccccccccccccccccccccccccc\trefs/tags/v1.0\n";
        let rev = parse_tag_output(output, "v1.0").unwrap();
        assert_eq!(rev, "cccccccccccccccccccccccccccccccccccccccc");
    }

    #[test]
    fn validate_git_rev_rejects_short_rev() {
        let err = validate_git_rev("deadbeef").unwrap_err();
        assert!(err.to_string().contains("40 hex"));
    }
}
