use std::path::Path;
use std::path::PathBuf;

use bounded_tree_core::EntryKind as SharedEntryKind;
use bounded_tree_core::LimitValues;
use bounded_tree_core::SymlinkPolicy;
use bounded_tree_core::TreeLimits;
use cap_std::ambient_authority;
use cap_std::fs::Dir;
use serde::Deserialize;
use serde::Serialize;

use crate::frontend_artifact_export::FRONTEND_ARTIFACT_EXPORT_DIGEST_PREFIX_BLAKE3;

pub const FRONTEND_ARTIFACT_STORE_REPORT_SCHEMA: &str = "mantle-frontend-artifact-store-v1";
pub const FRONTEND_ARTIFACT_STORE_MANIFEST_SCHEMA: &str = "mantle-frontend-artifact-store-manifest-v1";
pub const FRONTEND_ARTIFACT_REF_PREFIX_BLAKE3: &str = "mantle://blake3/";

const FRONTEND_ARTIFACT_STORE_ROOT_DIR: &str = "frontend-artifacts";
const FRONTEND_ARTIFACT_STORE_VERSION_DIR: &str = "v1";
const FRONTEND_ARTIFACT_STORE_ALGORITHM_DIR: &str = "blake3";
const FRONTEND_ARTIFACT_CONTENT_BASENAME: &str = "content";
const FRONTEND_ARTIFACT_PARTIAL_SUFFIX: &str = "partial";
const FRONTEND_ARTIFACT_MANIFEST_BASENAME: &str = "manifest.json";
const TREE_HASH_PREIMAGE_VERSION: &str = "mantle-frontend-artifact-tree-v1";
const TREE_FIELD_SEPARATOR: &str = "\u{0}";
const TREE_RECORD_SEPARATOR: &str = "\n";
const ROOT_RELATIVE_PATH: &str = ".";
const ENTRY_KIND_FILE: &str = "file";
const ENTRY_KIND_DIRECTORY: &str = "directory";
const ENTRY_KIND_SYMLINK: &str = "symlink";
const HEX_CHARS_PER_BYTE: usize = 2;
const BLAKE3_HEX_LENGTH: usize = blake3::OUT_LEN.saturating_mul(HEX_CHARS_PER_BYTE);
const FILE_READ_BUFFER_BYTES: usize = 65_536;
const MAX_TREE_ENTRIES: usize = 1_000_000;
const MAX_TREE_MEMBER_ENTRIES: usize = MAX_TREE_ENTRIES - 1;
const MAX_TREE_DEPTH: u64 = u64::MAX;
const MAX_TREE_PATH_BYTES: u64 = u64::MAX;
const MAX_TREE_FILE_BYTES: u64 = u64::MAX;
const MAX_TREE_TOTAL_BYTES: u64 = u64::MAX;
const MAX_SYMLINK_TARGET_BYTES: u64 = u64::MAX;
const EXECUTABLE_PERMISSION_MASK: u32 = 0o111;

const _: () = {
    assert!(BLAKE3_HEX_LENGTH == blake3::OUT_LEN.saturating_mul(HEX_CHARS_PER_BYTE));
    assert!(MAX_TREE_DEPTH > 0);
    assert!(MAX_TREE_PATH_BYTES > 0);
    assert!(MAX_TREE_FILE_BYTES > 0);
    assert!(MAX_TREE_TOTAL_BYTES > 0);
    assert!(MAX_SYMLINK_TARGET_BYTES > 0);
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrontendArtifactStoreManifest {
    pub schema: String,
    pub artifact_ref: String,
    pub artifact_digest: String,
    pub content_kind: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrontendArtifactStoreImportReport {
    pub schema: String,
    pub artifact_ref: String,
    pub artifact_digest: String,
    pub stored_content_path: String,
    pub manifest_path: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredFrontendArtifact {
    pub artifact_ref: String,
    pub artifact_digest: String,
    pub content_path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ArtifactTreeEntry {
    relative_path: String,
    kind: String,
    digest_or_target: String,
    executable: bool,
}

struct CollectedArtifactTree {
    entries: Vec<ArtifactTreeEntry>,
    prepared_directory: Option<bounded_tree_cap::PreparedTree>,
}

/// Recompute a frontend artifact identity without mutating the content store.
///
/// OCI export uses this after materialization so projection admission binds the
/// exact metadata-aware object already accepted by the store.
pub fn frontend_artifact_identity(source: &Path) -> Result<String, String> {
    std::fs::symlink_metadata(source)
        .map_err(|err| format!("reading frontend artifact metadata for {}: {err}", source.display()))?;
    let collected = collect_artifact_tree_entries(source)?;
    root_content_kind(&collected.entries)?;
    Ok(artifact_ref_from_digest_hex(&hash_artifact_tree_entries(&collected.entries)))
}

pub fn import_frontend_artifact(source: &Path, state_dir: &Path) -> Result<FrontendArtifactStoreImportReport, String> {
    if !source.exists() {
        return Err(format!("frontend artifact source does not exist: {}", source.display()));
    }
    let collected = collect_artifact_tree_entries(source)?;
    let content_kind = root_content_kind(&collected.entries)?;
    let digest_hex = hash_artifact_tree_entries(&collected.entries);
    let artifact_ref = artifact_ref_from_digest_hex(&digest_hex);
    let artifact_digest = artifact_digest_from_digest_hex(&digest_hex);
    let content_path = stored_content_path(state_dir, &digest_hex);
    let manifest_path = stored_manifest_path(state_dir, &digest_hex);

    if !content_path.exists() {
        let partial_path = partial_content_path(state_dir, &digest_hex);
        remove_existing_path(&partial_path)?;
        copy_collected_artifact(source, &partial_path, &collected)?;
        std::fs::rename(&partial_path, &content_path)
            .map_err(|err| format!("renaming {} -> {}: {err}", partial_path.display(), content_path.display()))?;
    }

    let manifest = FrontendArtifactStoreManifest {
        schema: FRONTEND_ARTIFACT_STORE_MANIFEST_SCHEMA.to_string(),
        artifact_ref: artifact_ref.clone(),
        artifact_digest: artifact_digest.clone(),
        content_kind,
    };
    write_json_file(&manifest_path, &manifest)?;

    let outcome = FrontendArtifactStoreImportReport {
        schema: FRONTEND_ARTIFACT_STORE_REPORT_SCHEMA.to_string(),
        artifact_ref,
        artifact_digest,
        stored_content_path: content_path.display().to_string(),
        manifest_path: manifest_path.display().to_string(),
    };
    debug_assert_eq!(outcome.artifact_ref, manifest.artifact_ref);
    debug_assert_eq!(outcome.artifact_digest, manifest.artifact_digest);
    Ok(outcome)
}

pub fn materialize_frontend_artifact(
    state_dir: &Path,
    artifact_ref: &str,
    destination: &Path,
) -> Result<Option<StoredFrontendArtifact>, String> {
    let Some(digest_hex) = parse_artifact_ref_digest_hex(artifact_ref) else {
        return Err(format!(
            "frontend artifact ref must use {FRONTEND_ARTIFACT_REF_PREFIX_BLAKE3}<hex>; got {artifact_ref}"
        ));
    };
    let content_path = stored_content_path(state_dir, digest_hex);
    if !content_path.exists() {
        return Ok(None);
    }
    copy_path_to_destination(&content_path, destination)?;
    Ok(Some(StoredFrontendArtifact {
        artifact_ref: artifact_ref.to_string(),
        artifact_digest: artifact_digest_from_digest_hex(digest_hex),
        content_path: destination.to_path_buf(),
    }))
}

pub fn artifact_digest_from_ref(artifact_ref: &str) -> Option<String> {
    parse_artifact_ref_digest_hex(artifact_ref).map(artifact_digest_from_digest_hex)
}

/// Check that both content and store metadata exist without reading content bytes.
pub fn frontend_artifact_is_available(state_dir: &Path, artifact_ref: &str) -> Result<bool, String> {
    debug_assert!(!FRONTEND_ARTIFACT_REF_PREFIX_BLAKE3.is_empty());
    let Some(digest_hex) = parse_artifact_ref_digest_hex(artifact_ref) else {
        return Err(format!(
            "frontend artifact ref must use {FRONTEND_ARTIFACT_REF_PREFIX_BLAKE3}<hex>; got {artifact_ref}"
        ));
    };
    let content_path = stored_content_path(state_dir, digest_hex);
    let manifest_path = stored_manifest_path(state_dir, digest_hex);
    let is_content_available = match std::fs::symlink_metadata(&content_path) {
        Ok(_) => true,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(error) => return Err(format!("checking frontend artifact content availability: {error}")),
    };
    let is_manifest_available = match std::fs::symlink_metadata(&manifest_path) {
        Ok(metadata) => metadata.is_file() && !metadata.file_type().is_symlink(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(error) => return Err(format!("checking frontend artifact manifest availability: {error}")),
    };
    Ok(is_content_available && is_manifest_available)
}

fn stored_content_path(state_dir: &Path, digest_hex: &str) -> PathBuf {
    artifact_digest_dir(state_dir, digest_hex).join(FRONTEND_ARTIFACT_CONTENT_BASENAME)
}

fn partial_content_path(state_dir: &Path, digest_hex: &str) -> PathBuf {
    artifact_digest_dir(state_dir, digest_hex)
        .join(format!("{FRONTEND_ARTIFACT_CONTENT_BASENAME}.{FRONTEND_ARTIFACT_PARTIAL_SUFFIX}"))
}

fn stored_manifest_path(state_dir: &Path, digest_hex: &str) -> PathBuf {
    artifact_digest_dir(state_dir, digest_hex).join(FRONTEND_ARTIFACT_MANIFEST_BASENAME)
}

fn artifact_digest_dir(state_dir: &Path, digest_hex: &str) -> PathBuf {
    state_dir
        .join(FRONTEND_ARTIFACT_STORE_ROOT_DIR)
        .join(FRONTEND_ARTIFACT_STORE_VERSION_DIR)
        .join(FRONTEND_ARTIFACT_STORE_ALGORITHM_DIR)
        .join(digest_hex)
}

fn artifact_ref_from_digest_hex(digest_hex: &str) -> String {
    format!("{FRONTEND_ARTIFACT_REF_PREFIX_BLAKE3}{digest_hex}")
}

fn artifact_digest_from_digest_hex(digest_hex: &str) -> String {
    format!("{FRONTEND_ARTIFACT_EXPORT_DIGEST_PREFIX_BLAKE3}{digest_hex}")
}

fn parse_artifact_ref_digest_hex(artifact_ref: &str) -> Option<&str> {
    let digest_hex = artifact_ref.strip_prefix(FRONTEND_ARTIFACT_REF_PREFIX_BLAKE3)?;
    if is_lowercase_blake3_hex(digest_hex) {
        return Some(digest_hex);
    }
    None
}

fn is_lowercase_blake3_hex(value: &str) -> bool {
    value.len() == BLAKE3_HEX_LENGTH && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn collect_artifact_tree_entries(root: &Path) -> Result<CollectedArtifactTree, String> {
    let metadata =
        std::fs::symlink_metadata(root).map_err(|err| format!("reading metadata for {}: {err}", root.display()))?;
    if metadata.file_type().is_symlink() {
        return Ok(CollectedArtifactTree {
            entries: vec![symlink_entry(root, ROOT_RELATIVE_PATH.to_string())?],
            prepared_directory: None,
        });
    }
    if metadata.is_file() {
        return Ok(CollectedArtifactTree {
            entries: vec![file_entry(root, ROOT_RELATIVE_PATH.to_string())?],
            prepared_directory: None,
        });
    }
    if !metadata.is_dir() {
        return Err(format!("unsupported frontend artifact file type: {}", root.display()));
    }

    let prepared = prepare_frontend_tree(root)?;
    let mut entries = Vec::with_capacity(prepared.plan().member_facts().len().saturating_add(1));
    entries.push(ArtifactTreeEntry {
        relative_path: ROOT_RELATIVE_PATH.to_string(),
        kind: ENTRY_KIND_DIRECTORY.to_string(),
        digest_or_target: String::new(),
        executable: false,
    });
    for fact in prepared.plan().member_facts() {
        entries.push(frontend_entry_from_shared(fact)?);
    }
    entries.sort_by(|left, right| left.relative_path.cmp(&right.relative_path).then(left.kind.cmp(&right.kind)));
    debug_assert!(entries.len() <= MAX_TREE_ENTRIES);
    debug_assert!(entries.iter().any(|entry| entry.relative_path == ROOT_RELATIVE_PATH));
    Ok(CollectedArtifactTree {
        entries,
        prepared_directory: Some(prepared),
    })
}

fn frontend_tree_limits() -> Result<TreeLimits, String> {
    let entries = u64::try_from(MAX_TREE_MEMBER_ENTRIES)
        .map_err(|_| "frontend artifact member limit does not fit u64".to_string())?;
    TreeLimits::new(LimitValues {
        entries,
        depth: MAX_TREE_DEPTH,
        path_bytes: MAX_TREE_PATH_BYTES,
        file_bytes: MAX_TREE_FILE_BYTES,
        total_bytes: MAX_TREE_TOTAL_BYTES,
        symlink_target_bytes: MAX_SYMLINK_TARGET_BYTES,
    })
    .map_err(|error| format!("invalid frontend bounded-tree limit: {:?}", error.kind()))
}

fn prepare_frontend_tree(root: &Path) -> Result<bounded_tree_cap::PreparedTree, String> {
    let source = Dir::open_ambient_dir(root, ambient_authority())
        .map_err(|error| format!("opening frontend artifact capability root: {error}"))?;
    bounded_tree_cap::prepare(&source, frontend_tree_limits()?, SymlinkPolicy::PreserveInternal)
        .map_err(|error| frontend_tree_error("observing frontend artifact", &error))
}

fn frontend_entry_from_shared(fact: &bounded_tree_core::MemberFact) -> Result<ArtifactTreeEntry, String> {
    let relative_path = shared_relative_path(fact.path().components())?;
    match fact.kind() {
        SharedEntryKind::Directory => Ok(ArtifactTreeEntry {
            relative_path,
            kind: ENTRY_KIND_DIRECTORY.to_string(),
            digest_or_target: String::new(),
            executable: false,
        }),
        SharedEntryKind::File => {
            let digest = fact
                .file_content()
                .ok_or_else(|| format!("bounded-tree file fact omitted content identity: {relative_path}"))?;
            Ok(ArtifactTreeEntry {
                relative_path,
                kind: ENTRY_KIND_FILE.to_string(),
                digest_or_target: blake3_hex(digest.as_bytes()),
                executable: fact.mode().bits() & EXECUTABLE_PERMISSION_MASK != 0,
            })
        }
        SharedEntryKind::Symlink => {
            let target = fact
                .symlink_target()
                .ok_or_else(|| format!("bounded-tree symlink fact omitted target: {relative_path}"))?;
            let target = String::from_utf8(target.to_vec())
                .map_err(|_| format!("bounded-tree symlink target is not UTF-8: {relative_path}"))?;
            Ok(ArtifactTreeEntry {
                relative_path,
                kind: ENTRY_KIND_SYMLINK.to_string(),
                digest_or_target: target,
                executable: false,
            })
        }
        SharedEntryKind::Unsupported => Err(format!("bounded-tree admitted unsupported entry: {relative_path}")),
    }
}

fn shared_relative_path(components: &[Vec<u8>]) -> Result<String, String> {
    let mut parts = Vec::with_capacity(components.len());
    for component in components {
        parts.push(
            String::from_utf8(component.clone()).map_err(|_| "bounded-tree path component is not UTF-8".to_string())?,
        );
    }
    Ok(parts.join("/"))
}

fn blake3_hex(bytes: &[u8; blake3::OUT_LEN]) -> String {
    let mut hex = String::with_capacity(BLAKE3_HEX_LENGTH);
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut hex, "{byte:02x}").expect("writing to String cannot fail");
    }
    hex
}

fn frontend_tree_error(action: &str, error: &bounded_tree_cap::ShellError) -> String {
    let path = error
        .path_components()
        .iter()
        .map(|component| String::from_utf8_lossy(component))
        .collect::<Vec<_>>()
        .join("/");
    format!("bounded-tree {action} failed with {:?} at {path}", error.kind())
}

fn root_content_kind(entries: &[ArtifactTreeEntry]) -> Result<String, String> {
    let root = entries
        .iter()
        .find(|entry| entry.relative_path == ROOT_RELATIVE_PATH)
        .ok_or_else(|| "frontend artifact tree did not include a root entry".to_string())?;
    Ok(root.kind.clone())
}

fn file_entry(path: &Path, relative_path: String) -> Result<ArtifactTreeEntry, String> {
    Ok(ArtifactTreeEntry {
        relative_path,
        kind: ENTRY_KIND_FILE.to_string(),
        digest_or_target: hash_file_content(path)?,
        executable: is_executable(path)?,
    })
}

fn symlink_entry(path: &Path, relative_path: String) -> Result<ArtifactTreeEntry, String> {
    let target = std::fs::read_link(path).map_err(|err| format!("reading symlink {}: {err}", path.display()))?;
    let target_text = target
        .to_str()
        .ok_or_else(|| format!("symlink target for {} is not UTF-8", path.display()))?
        .to_string();
    Ok(ArtifactTreeEntry {
        relative_path,
        kind: ENTRY_KIND_SYMLINK.to_string(),
        digest_or_target: target_text,
        executable: false,
    })
}

fn hash_file_content(path: &Path) -> Result<String, String> {
    let mut file = std::fs::File::open(path).map_err(|err| format!("opening {}: {err}", path.display()))?;
    let file_size_bytes =
        file.metadata().map_err(|err| format!("reading metadata for {}: {err}", path.display()))?.len();
    let buffer_size_bytes = u64::try_from(FILE_READ_BUFFER_BYTES)
        .map_err(|_| "frontend artifact read buffer size does not fit u64".to_string())?;
    let maximum_read_count = file_size_bytes
        .div_ceil(buffer_size_bytes)
        .checked_add(1)
        .ok_or_else(|| format!("frontend artifact read count overflowed for {}", path.display()))?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0_u8; FILE_READ_BUFFER_BYTES];
    debug_assert!(maximum_read_count > 0);
    debug_assert_eq!(u64::try_from(buffer.len()), Ok(buffer_size_bytes));
    for _ in 0..maximum_read_count {
        let read_size_bytes =
            std::io::Read::read(&mut file, &mut buffer).map_err(|err| format!("reading {}: {err}", path.display()))?;
        if read_size_bytes == 0 {
            return Ok(hasher.finalize().to_hex().to_string());
        }
        hasher.update(&buffer[..read_size_bytes]);
    }
    Err(format!("frontend artifact file grew while hashing: {}", path.display()))
}

fn hash_artifact_tree_entries(entries: &[ArtifactTreeEntry]) -> String {
    let mut preimage = String::new();
    append_tree_field(&mut preimage, "preimage_version", TREE_HASH_PREIMAGE_VERSION);
    for entry in entries {
        append_tree_field(&mut preimage, "path", &entry.relative_path);
        append_tree_field(&mut preimage, "kind", &entry.kind);
        append_tree_field(&mut preimage, "digest_or_target", &entry.digest_or_target);
        append_tree_field(&mut preimage, "executable", if entry.executable { "true" } else { "false" });
        preimage.push_str(TREE_RECORD_SEPARATOR);
    }
    blake3::hash(preimage.as_bytes()).to_hex().to_string()
}

fn append_tree_field(preimage: &mut String, name: impl AsRef<str>, value: &str) {
    preimage.push_str(name.as_ref());
    preimage.push_str(TREE_FIELD_SEPARATOR);
    preimage.push_str(&value.len().to_string());
    preimage.push_str(TREE_FIELD_SEPARATOR);
    preimage.push_str(value);
    preimage.push_str(TREE_RECORD_SEPARATOR);
}

fn copy_collected_artifact(source: &Path, destination: &Path, collected: &CollectedArtifactTree) -> Result<(), String> {
    if let Some(prepared) = collected.prepared_directory.as_ref() {
        return copy_prepared_directory(prepared, destination);
    }
    copy_path_to_destination(source, destination)
}

fn copy_path_to_destination(source: &Path, destination: &Path) -> Result<(), String> {
    if destination.exists() {
        return Err(format!("frontend artifact destination already exists: {}", destination.display()));
    }
    let metadata =
        std::fs::symlink_metadata(source).map_err(|err| format!("reading metadata for {}: {err}", source.display()))?;
    if metadata.file_type().is_symlink() {
        return copy_symlink(source, destination);
    }
    if metadata.is_file() {
        return copy_file(source, destination);
    }
    if metadata.is_dir() {
        return copy_directory(source, destination);
    }
    Err(format!("unsupported frontend artifact file type: {}", source.display()))
}

fn copy_file(source: &Path, destination: &Path) -> Result<(), String> {
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent).map_err(|err| format!("creating {}: {err}", parent.display()))?;
    }
    std::fs::copy(source, destination)
        .map(|_| ())
        .map_err(|err| format!("copying {} -> {}: {err}", source.display(), destination.display()))
}

fn copy_directory(source: &Path, destination: &Path) -> Result<(), String> {
    let prepared = prepare_frontend_tree(source)?;
    copy_prepared_directory(&prepared, destination)
}

fn copy_prepared_directory(prepared: &bounded_tree_cap::PreparedTree, destination: &Path) -> Result<(), String> {
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent).map_err(|err| format!("creating {}: {err}", parent.display()))?;
    }
    std::fs::create_dir(destination).map_err(|err| format!("creating {}: {err}", destination.display()))?;
    let destination_root = Dir::open_ambient_dir(destination, ambient_authority())
        .map_err(|error| format!("opening frontend artifact destination capability: {error}"))?;
    bounded_tree_cap::execute(prepared, &destination_root)
        .map_err(|error| frontend_tree_error("copying frontend artifact", &error))?;
    debug_assert!(destination.is_dir());
    Ok(())
}

fn copy_symlink(source: &Path, destination: &Path) -> Result<(), String> {
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent).map_err(|err| format!("creating {}: {err}", parent.display()))?;
    }
    let target = std::fs::read_link(source).map_err(|err| format!("reading symlink {}: {err}", source.display()))?;
    create_symlink(&target, destination)
}

#[cfg(unix)]
fn create_symlink(target: &Path, destination: &Path) -> Result<(), String> {
    std::os::unix::fs::symlink(target, destination)
        .map_err(|err| format!("creating symlink {} -> {}: {err}", destination.display(), target.display()))
}

#[cfg(not(unix))]
fn create_symlink(_target: &Path, destination: &Path) -> Result<(), String> {
    Err(format!(
        "symlink frontend artifact export is not supported on this platform: {}",
        destination.display()
    ))
}

#[cfg(unix)]
fn is_executable(path: &Path) -> Result<bool, String> {
    use std::os::unix::fs::PermissionsExt;
    let metadata = std::fs::metadata(path).map_err(|err| format!("reading metadata for {}: {err}", path.display()))?;
    Ok(metadata.permissions().mode() & EXECUTABLE_PERMISSION_MASK != 0)
}

#[cfg(not(unix))]
fn is_executable(_path: &Path) -> Result<bool, String> {
    Ok(false)
}

fn remove_existing_path(path: &Path) -> Result<(), String> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(format!("reading metadata for {}: {err}", path.display())),
    };
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        return std::fs::remove_dir_all(path).map_err(|err| format!("removing dir {}: {err}", path.display()));
    }
    std::fs::remove_file(path).map_err(|err| format!("removing file {}: {err}", path.display()))
}

fn write_json_file(path: &Path, value: &impl Serialize) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| format!("creating {}: {err}", parent.display()))?;
    }
    let bytes = serde_json::to_vec_pretty(value).map_err(|err| format!("serializing {}: {err}", path.display()))?;
    std::fs::write(path, bytes).map_err(|err| format!("writing {}: {err}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILE_CONTENT: &[u8] = b"hello artifact";
    const SECOND_FILE_CONTENT: &[u8] = b"nested artifact";

    #[test]
    fn import_and_materialize_file_roundtrip() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = temp.path().join("source.txt");
        let dest = temp.path().join("exported.txt");
        std::fs::write(&source, FILE_CONTENT).expect("write source");

        let report = import_frontend_artifact(&source, temp.path()).expect("import artifact");
        assert!(report.artifact_ref.starts_with(FRONTEND_ARTIFACT_REF_PREFIX_BLAKE3));
        assert!(report.artifact_digest.starts_with(FRONTEND_ARTIFACT_EXPORT_DIGEST_PREFIX_BLAKE3));

        let stored = materialize_frontend_artifact(temp.path(), &report.artifact_ref, &dest)
            .expect("materialize artifact")
            .expect("stored artifact");
        assert_eq!(stored.artifact_digest, report.artifact_digest);
        assert_eq!(std::fs::read(&dest).expect("read exported"), FILE_CONTENT);
    }

    #[test]
    fn import_and_materialize_directory_roundtrip() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = temp.path().join("source-dir");
        let nested = source.join("nested");
        let dest = temp.path().join("exported-dir");
        std::fs::create_dir_all(&nested).expect("create nested source");
        std::fs::write(source.join("root.txt"), FILE_CONTENT).expect("write root file");
        std::fs::write(nested.join("child.txt"), SECOND_FILE_CONTENT).expect("write nested file");

        let report = import_frontend_artifact(&source, temp.path()).expect("import artifact");
        let stored = materialize_frontend_artifact(temp.path(), &report.artifact_ref, &dest)
            .expect("materialize artifact")
            .expect("stored artifact");

        assert_eq!(stored.content_path, dest);
        assert_eq!(std::fs::read(dest.join("root.txt")).expect("read root"), FILE_CONTENT);
        assert_eq!(std::fs::read(dest.join("nested").join("child.txt")).expect("read child"), SECOND_FILE_CONTENT,);
    }

    #[test]
    fn materialize_missing_ref_returns_none() {
        let temp = tempfile::tempdir().expect("tempdir");
        let digest = blake3::hash(b"missing").to_hex().to_string();
        let artifact_ref = artifact_ref_from_digest_hex(&digest);
        let dest = temp.path().join("missing");

        let result = materialize_frontend_artifact(temp.path(), &artifact_ref, &dest).expect("missing lookup");
        assert!(result.is_none());
        assert!(!dest.exists());
    }

    #[test]
    fn materialize_rejects_unsupported_ref() {
        let temp = tempfile::tempdir().expect("tempdir");
        let err = materialize_frontend_artifact(temp.path(), "/nix/store/not-an-artifact", &temp.path().join("out"))
            .expect_err("unsupported ref rejected");
        assert!(err.contains(FRONTEND_ARTIFACT_REF_PREFIX_BLAKE3));
    }

    // r[verify mantle.bounded_tree_adoption.parity]
    #[test]
    #[cfg(unix)]
    fn shared_member_facts_preserve_frontend_tree_identity() {
        use std::os::unix::fs::PermissionsExt as _;
        use std::os::unix::fs::symlink;

        let temp = tempfile::tempdir().expect("tempdir");
        let source = temp.path().join("source-dir");
        let nested = source.join("nested");
        std::fs::create_dir_all(&nested).expect("create nested source");
        std::fs::write(source.join("root.txt"), FILE_CONTENT).expect("write root file");
        let executable = nested.join("tool");
        std::fs::write(&executable, SECOND_FILE_CONTENT).expect("write executable");
        let mut permissions = std::fs::metadata(&executable).expect("metadata").permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&executable, permissions).expect("set executable mode");
        symlink("nested/tool", source.join("latest")).expect("create internal symlink");

        let shared = frontend_artifact_identity(&source).expect("shared identity");
        let legacy = legacy_artifact_tree_entries(&source).expect("legacy entries");
        let expected = artifact_ref_from_digest_hex(&hash_artifact_tree_entries(&legacy));

        assert_eq!(shared, expected);
    }

    // r[verify mantle.bounded_tree_adoption.parity]
    #[test]
    #[cfg(unix)]
    fn external_symlink_is_rejected_before_import() {
        use std::os::unix::fs::symlink;

        let temp = tempfile::tempdir().expect("tempdir");
        let source = temp.path().join("source-dir");
        let external = temp.path().join("external-secret");
        std::fs::create_dir(&source).expect("create source");
        std::fs::write(&external, b"secret-marker").expect("write external");
        symlink(&external, source.join("escape")).expect("create external symlink");

        let error = frontend_artifact_identity(&source).expect_err("external symlink must fail");

        assert!(error.contains("PlanRejected"));
        assert!(!error.contains("secret-marker"));
    }

    // r[verify mantle.bounded_tree_adoption.parity]
    #[test]
    fn source_change_after_identity_fails_before_frontend_publication() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = temp.path().join("source-dir");
        let destination = temp.path().join("stored");
        std::fs::create_dir(&source).expect("create source");
        let member = source.join("member");
        std::fs::write(&member, FILE_CONTENT).expect("write member");
        let collected = collect_artifact_tree_entries(&source).expect("collect identity facts");
        std::fs::write(&member, SECOND_FILE_CONTENT).expect("change member");

        let error = copy_collected_artifact(&source, &destination, &collected).expect_err("source change must fail");

        assert!(error.contains("SourceChanged"));
        assert!(!destination.join("member").exists());
    }

    fn legacy_artifact_tree_entries(root: &Path) -> Result<Vec<ArtifactTreeEntry>, String> {
        let mut entries = Vec::new();
        let mut worklist = vec![root.to_path_buf()];
        while let Some(path) = worklist.pop() {
            if entries.len() >= MAX_TREE_ENTRIES {
                return Err(format!("legacy fixture exceeds {MAX_TREE_ENTRIES} entries"));
            }
            let metadata =
                std::fs::symlink_metadata(&path).map_err(|error| format!("legacy fixture metadata: {error}"))?;
            let relative_path = legacy_relative_path(root, &path)?;
            if metadata.file_type().is_symlink() {
                entries.push(symlink_entry(&path, relative_path)?);
            } else if metadata.is_file() {
                entries.push(file_entry(&path, relative_path)?);
            } else if metadata.is_dir() {
                entries.push(ArtifactTreeEntry {
                    relative_path,
                    kind: ENTRY_KIND_DIRECTORY.to_string(),
                    digest_or_target: String::new(),
                    executable: false,
                });
                let mut children = std::fs::read_dir(&path)
                    .map_err(|error| format!("legacy fixture directory: {error}"))?
                    .map(|entry| entry.map(|value| value.path()).map_err(|error| error.to_string()))
                    .collect::<Result<Vec<_>, _>>()?;
                children.sort_by(|left, right| left.file_name().cmp(&right.file_name()));
                for child in children.into_iter().rev() {
                    worklist.push(child);
                }
            } else {
                return Err("legacy fixture contains unsupported entry".to_string());
            }
        }
        entries.sort_by(|left, right| left.relative_path.cmp(&right.relative_path).then(left.kind.cmp(&right.kind)));
        Ok(entries)
    }

    fn legacy_relative_path(root: &Path, path: &Path) -> Result<String, String> {
        let relative = path.strip_prefix(root).map_err(|error| error.to_string())?;
        if relative.as_os_str().is_empty() {
            return Ok(ROOT_RELATIVE_PATH.to_string());
        }
        let mut parts = Vec::new();
        for component in relative.components() {
            let std::path::Component::Normal(part) = component else {
                return Err("legacy fixture path is not normal".to_string());
            };
            parts.push(part.to_str().ok_or_else(|| "legacy fixture path is not UTF-8".to_string())?);
        }
        Ok(parts.join("/"))
    }
}
