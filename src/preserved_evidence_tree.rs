use std::ffi::OsString;
use std::fs;
use std::io::Read;
use std::io::Write;
#[cfg(unix)]
use std::os::unix::ffi::OsStringExt;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use cap_fs_ext::DirExt;
use cap_fs_ext::FollowSymlinks;
use cap_fs_ext::OpenOptionsFollowExt;
use cap_std::fs::Dir;
use cap_std::fs::Metadata;
#[cfg(unix)]
use cap_std::fs::MetadataExt;
use cap_std::fs::OpenOptions;

use crate::errors::RunError;
use crate::release_capability::ReleaseCapabilityRoot;
use crate::release_capability::ReleaseRootKind;
use crate::release_capability::ValidatedReleasePath;

const DIGEST_DOMAIN: &[u8] = b"mantle-preserved-evidence-tree-v1\0";
const HASH_BUFFER_BYTES: usize = 8_192;
const EMPTY_TREE_ACCOUNTED_SIZE_BYTES: u64 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PreservedEvidenceTreeLimits {
    pub(crate) entries_count_max: u32,
    pub(crate) depth_count_max: u32,
    pub(crate) path_bytes_max: u32,
    pub(crate) symlink_target_bytes_max: u32,
    pub(crate) total_file_bytes_max: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PreservedEvidenceTreeIdentity {
    pub(crate) total_file_bytes: u64,
    pub(crate) digest_blake3: String,
    pub(crate) entry_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EntryKind {
    Directory,
    File,
    Symlink,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EntryObservation {
    relative_path: String,
    kind: EntryKind,
    mode: u32,
    file_bytes: u64,
    symlink_target: Option<Vec<u8>>,
}

// r[impl bootstrap_inventory.source_built_mantle_fixed_point]
/// Hash preserved proof evidence without resolving or admitting symlink targets.
///
/// This observation-only path must not be used for source admission or copying.
pub(crate) fn hash_preserved_evidence_tree(
    path: &Path,
    limits: PreservedEvidenceTreeLimits,
) -> Result<PreservedEvidenceTreeIdentity, RunError> {
    let (source_root, entries, total_file_bytes, entries_count_max) = prepare_tree_observation(path, limits)?;
    let digest_blake3 = hash_entries(&source_root, &entries)?;
    let identity = observed_identity(&entries, total_file_bytes, digest_blake3)?;
    assert!(entries.len() <= entries_count_max);
    assert!(total_file_bytes <= limits.total_file_bytes_max);
    Ok(identity)
}

// r[impl bootstrap_inventory.source_built_mantle_checkpoint_reuse]
/// Copy opaque proof evidence without following symlinks, then remeasure it.
///
/// The destination must not exist. The copied tree is removed on any error.
pub(crate) fn copy_preserved_evidence_tree(
    source: &Path,
    destination: &Path,
    limits: PreservedEvidenceTreeLimits,
) -> Result<PreservedEvidenceTreeIdentity, RunError> {
    if destination.exists() {
        return Err(RunError::Internal(format!(
            "preserved evidence copy destination exists: {}",
            destination.display()
        )));
    }
    let expected = hash_preserved_evidence_tree(source, limits)?;
    let (source_root, entries, _total_file_bytes, entries_count_max) = prepare_tree_observation(source, limits)?;
    fs::create_dir(destination).map_err(|error| {
        RunError::Internal(format!("creating preserved evidence destination {}: {error}", destination.display()))
    })?;
    let copy_result = execute_preserved_copy(&source_root, destination, &entries);
    if let Err(error) = copy_result {
        return Err(copy_cleanup_error(error, remove_failed_copy(destination)));
    }
    let observed = hash_preserved_evidence_tree(destination, limits)?;
    if observed != expected {
        let mismatch = RunError::Internal(format!(
            "preserved evidence copy identity mismatch: expected {}, observed {}",
            expected.digest_blake3, observed.digest_blake3
        ));
        return Err(copy_cleanup_error(mismatch, remove_failed_copy(destination)));
    }
    assert!(entries.len() <= entries_count_max);
    assert_eq!(expected, observed);
    Ok(observed)
}

fn prepare_tree_observation(
    path: &Path,
    limits: PreservedEvidenceTreeLimits,
) -> Result<(ReleaseCapabilityRoot, Vec<EntryObservation>, u64, usize), RunError> {
    validate_limits(limits)?;
    let source_root =
        ReleaseCapabilityRoot::open_ambient_nofollow(ReleaseRootKind::ReleaseTreeSource, path).map_err(|error| {
            RunError::Internal(format!("opening no-follow preserved evidence root {}: {error}", path.display()))
        })?;
    let entries_count_max = usize::try_from(limits.entries_count_max)
        .map_err(|_| RunError::Internal("preserved evidence entry limit does not fit usize".to_string()))?;
    let mut entries = Vec::new();
    let mut total_file_bytes = 0_u64;
    observe_directory(source_root.dir(), "", limits, entries_count_max, &mut total_file_bytes, &mut entries)?;
    entries.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    assert!(entries.len() <= entries_count_max);
    Ok((source_root, entries, total_file_bytes, entries_count_max))
}

fn observed_identity(
    entries: &[EntryObservation],
    total_file_bytes: u64,
    digest_blake3: String,
) -> Result<PreservedEvidenceTreeIdentity, RunError> {
    let entry_count = u32::try_from(entries.len())
        .map_err(|_| RunError::Internal("preserved evidence entry count does not fit u32".to_string()))?;
    let accounted_file_bytes = if total_file_bytes == 0 {
        EMPTY_TREE_ACCOUNTED_SIZE_BYTES
    } else {
        total_file_bytes
    };
    assert_eq!(digest_blake3.len(), blake3::OUT_LEN.saturating_mul(2));
    Ok(PreservedEvidenceTreeIdentity {
        total_file_bytes: accounted_file_bytes,
        digest_blake3,
        entry_count,
    })
}

fn execute_preserved_copy(
    source_root: &ReleaseCapabilityRoot,
    destination_root: &Path,
    entries: &[EntryObservation],
) -> Result<(), RunError> {
    let mut directories = Vec::new();
    for entry in entries {
        let destination = destination_root.join(&entry.relative_path);
        match entry.kind {
            EntryKind::Directory => {
                fs::create_dir(&destination)
                    .map_err(|error| copy_io_error("creating directory", &entry.relative_path, error))?;
                directories.push((destination, entry.mode));
            }
            EntryKind::File => copy_preserved_file(source_root, destination_root, entry)?,
            EntryKind::Symlink => copy_preserved_symlink(source_root, destination_root, entry)?,
        }
    }
    for (path, mode) in directories.into_iter().rev() {
        set_copied_mode(&path, mode)?;
    }
    assert_eq!(source_root.kind(), ReleaseRootKind::ReleaseTreeSource);
    debug_assert!(destination_root.is_dir());
    Ok(())
}

fn copy_preserved_file(
    source_root: &ReleaseCapabilityRoot,
    destination_root: &Path,
    entry: &EntryObservation,
) -> Result<(), RunError> {
    let (parent, name) = open_parent_directory_nofollow(source_root.dir(), &entry.relative_path)?;
    let metadata = parent
        .symlink_metadata(&name)
        .map_err(|error| evidence_io_error("revalidating copied file metadata", &entry.relative_path, error))?;
    revalidate_entry(entry, &metadata)?;
    let mut source = open_file_nofollow(&parent, &name)
        .map_err(|error| evidence_io_error("opening copied file", &entry.relative_path, error))?;
    let opened = source
        .metadata()
        .map_err(|error| evidence_io_error("reading copied file metadata", &entry.relative_path, error))?;
    revalidate_entry(entry, &opened)?;
    let destination = destination_root.join(&entry.relative_path);
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&destination)
        .map_err(|error| copy_io_error("creating file", &entry.relative_path, error))?;
    copy_exact_file_bytes(&mut source, &mut output, entry)?;
    set_copied_mode(&destination, entry.mode)
}

fn copy_exact_file_bytes(
    source: &mut cap_std::fs::File,
    destination: &mut fs::File,
    entry: &EntryObservation,
) -> Result<(), RunError> {
    let mut remaining = entry.file_bytes;
    let mut buffer = [0_u8; HASH_BUFFER_BYTES];
    while remaining > 0 {
        let read_max = usize::try_from(remaining.min(HASH_BUFFER_BYTES as u64))
            .map_err(|_| RunError::Internal("preserved copy byte bound does not fit usize".to_string()))?;
        let count = source
            .read(&mut buffer[..read_max])
            .map_err(|error| copy_io_error("reading file", &entry.relative_path, error))?;
        if count == 0 {
            return Err(RunError::Internal(format!("preserved evidence file shrank at {}", entry.relative_path)));
        }
        destination
            .write_all(&buffer[..count])
            .map_err(|error| copy_io_error("writing file", &entry.relative_path, error))?;
        remaining = remaining
            .checked_sub(
                u64::try_from(count)
                    .map_err(|_| RunError::Internal("preserved copy byte count does not fit u64".to_string()))?,
            )
            .ok_or_else(|| RunError::Internal("preserved copy byte count underflowed".to_string()))?;
    }
    let extra = source
        .read(&mut buffer[..1])
        .map_err(|error| copy_io_error("checking file growth", &entry.relative_path, error))?;
    if extra != 0 {
        return Err(RunError::Internal(format!("preserved evidence file grew at {}", entry.relative_path)));
    }
    destination
        .sync_all()
        .map_err(|error| copy_io_error("synchronizing file", &entry.relative_path, error))?;
    assert_eq!(remaining, 0);
    Ok(())
}

#[cfg(unix)]
fn copy_preserved_symlink(
    source_root: &ReleaseCapabilityRoot,
    destination_root: &Path,
    entry: &EntryObservation,
) -> Result<(), RunError> {
    let (parent, name) = open_parent_directory_nofollow(source_root.dir(), &entry.relative_path)?;
    let metadata = parent
        .symlink_metadata(&name)
        .map_err(|error| evidence_io_error("revalidating copied symlink metadata", &entry.relative_path, error))?;
    revalidate_entry(entry, &metadata)?;
    let target = parent
        .read_link_contents(&name)
        .map_err(|error| evidence_io_error("reading copied symlink target", &entry.relative_path, error))?;
    let target_bytes = target.into_os_string().as_encoded_bytes().to_vec();
    if entry.symlink_target.as_deref() != Some(target_bytes.as_slice()) {
        return Err(RunError::Internal(format!(
            "preserved evidence symlink target changed at {}",
            entry.relative_path
        )));
    }
    let destination = destination_root.join(&entry.relative_path);
    std::os::unix::fs::symlink(OsString::from_vec(target_bytes), &destination)
        .map_err(|error| copy_io_error("creating symlink", &entry.relative_path, error))?;
    debug_assert!(fs::symlink_metadata(destination).is_ok_and(|metadata| metadata.file_type().is_symlink()));
    Ok(())
}

#[cfg(not(unix))]
fn copy_preserved_symlink(
    _source_root: &ReleaseCapabilityRoot,
    _destination_root: &Path,
    entry: &EntryObservation,
) -> Result<(), RunError> {
    Err(RunError::Internal(format!(
        "preserved evidence symlink copy requires Unix at {}",
        entry.relative_path
    )))
}

#[cfg(unix)]
fn set_copied_mode(path: &Path, mode: u32) -> Result<(), RunError> {
    const PERMISSION_BITS_MASK: u32 = 0o7_777;
    let permissions = fs::Permissions::from_mode(mode & PERMISSION_BITS_MASK);
    fs::set_permissions(path, permissions)
        .map_err(|error| RunError::Internal(format!("setting preserved evidence mode {}: {error}", path.display())))
}

#[cfg(not(unix))]
fn set_copied_mode(_path: &Path, _mode: u32) -> Result<(), RunError> {
    Ok(())
}

fn remove_failed_copy(destination: &Path) -> Result<(), RunError> {
    match fs::remove_dir_all(destination) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(RunError::Internal(format!(
            "removing failed preserved evidence copy {}: {error}",
            destination.display()
        ))),
    }
}

fn copy_cleanup_error(primary: RunError, cleanup: Result<(), RunError>) -> RunError {
    match cleanup {
        Ok(()) => primary,
        Err(cleanup_error) => RunError::Internal(format!("{primary}; cleanup failed: {cleanup_error}")),
    }
}

fn copy_io_error(action: &str, relative_path: &str, error: std::io::Error) -> RunError {
    RunError::Internal(format!("{action} for preserved evidence copy at {relative_path}: {error}"))
}

fn validate_limits(limits: PreservedEvidenceTreeLimits) -> Result<(), RunError> {
    let release_limits = crunch_release_core::TreeCopyLimits::RELEASE_BUNDLE;
    let positive = limits.entries_count_max > 0
        && limits.depth_count_max > 0
        && limits.path_bytes_max > 0
        && limits.symlink_target_bytes_max > 0
        && limits.total_file_bytes_max > 0;
    if !positive {
        return Err(RunError::Internal("preserved evidence tree limits must be positive".to_string()));
    }
    if limits.depth_count_max > release_limits.depth_count_max
        || limits.path_bytes_max > release_limits.path_bytes_max
        || limits.symlink_target_bytes_max > release_limits.path_bytes_max
    {
        return Err(RunError::Internal(
            "preserved evidence path limits exceed the release capability contract".to_string(),
        ));
    }
    assert!(limits.entries_count_max > 0);
    assert!(limits.total_file_bytes_max > 0);
    Ok(())
}

fn observe_directory(
    current: &Dir,
    prefix: &str,
    limits: PreservedEvidenceTreeLimits,
    entries_count_max: usize,
    total_file_bytes: &mut u64,
    entries: &mut Vec<EntryObservation>,
) -> Result<(), RunError> {
    let remaining_entries = entries_count_max
        .checked_sub(entries.len())
        .ok_or_else(|| RunError::Internal("preserved evidence remaining entry budget underflowed".to_string()))?;
    let mut names = directory_names(current, prefix, remaining_entries)?;
    names.sort();
    for name in names {
        let relative_path = evidence_relative_path(prefix, &name, limits)?;
        require_entry_capacity(entries, entries_count_max, &relative_path)?;
        let metadata = current
            .symlink_metadata(&name)
            .map_err(|error| evidence_io_error("reading entry metadata", &relative_path, error))?;
        let entry = observe_entry(current, &name, &relative_path, &metadata, limits)?;
        account_file_bytes(&entry, total_file_bytes, limits.total_file_bytes_max)?;
        let is_directory = entry.kind == EntryKind::Directory;
        entries.push(entry);
        if is_directory {
            let child = current
                .open_dir_nofollow(&name)
                .map_err(|error| evidence_io_error("opening child directory", &relative_path, error))?;
            observe_directory(&child, &relative_path, limits, entries_count_max, total_file_bytes, entries)?;
        }
    }
    debug_assert!(entries.len() <= entries_count_max);
    Ok(())
}

fn directory_names(current: &Dir, relative_path: &str, names_count_max: usize) -> Result<Vec<String>, RunError> {
    let directory_entries =
        current.entries().map_err(|error| evidence_io_error("reading directory", relative_path, error))?;
    let mut names = Vec::new();
    for entry in directory_entries {
        if names.len() >= names_count_max {
            return Err(RunError::Internal(format!(
                "preserved evidence entry count exceeds the remaining budget at {relative_path}"
            )));
        }
        let name = entry
            .map_err(|error| evidence_io_error("reading directory entry", relative_path, error))?
            .file_name()
            .into_string()
            .map_err(|_| RunError::Internal("preserved evidence entry name is not valid UTF-8".to_string()))?;
        names.push(name);
    }
    assert!(names.len() <= names_count_max);
    debug_assert!(relative_path.is_empty() || !relative_path.starts_with('/'));
    Ok(names)
}

fn evidence_relative_path(prefix: &str, name: &str, limits: PreservedEvidenceTreeLimits) -> Result<String, RunError> {
    let relative_path = if prefix.is_empty() {
        name.to_string()
    } else {
        format!("{prefix}/{name}")
    };
    let path_bytes = u32::try_from(relative_path.len())
        .map_err(|_| RunError::Internal("preserved evidence path length does not fit u32".to_string()))?;
    let depth_count = u32::try_from(relative_path.split('/').count())
        .map_err(|_| RunError::Internal("preserved evidence path depth does not fit u32".to_string()))?;
    if path_bytes > limits.path_bytes_max {
        return Err(RunError::Internal(format!(
            "preserved evidence path bytes exceed {} at {relative_path}",
            limits.path_bytes_max
        )));
    }
    if depth_count > limits.depth_count_max {
        return Err(RunError::Internal(format!(
            "preserved evidence path depth exceeds {} at {relative_path}",
            limits.depth_count_max
        )));
    }
    ValidatedReleasePath::new(&relative_path).map_err(|error| {
        RunError::Internal(format!("invalid preserved evidence relative path {relative_path}: {error:?}"))
    })?;
    assert!(!relative_path.is_empty());
    debug_assert!(!Path::new(&relative_path).is_absolute());
    Ok(relative_path)
}

fn require_entry_capacity(
    entries: &[EntryObservation],
    entries_count_max: usize,
    relative_path: &str,
) -> Result<(), RunError> {
    let next_count = entries
        .len()
        .checked_add(1)
        .ok_or_else(|| RunError::Internal("preserved evidence entry count overflowed usize".to_string()))?;
    if next_count > entries_count_max {
        return Err(RunError::Internal(format!(
            "preserved evidence entry count exceeds {entries_count_max} at {relative_path}"
        )));
    }
    assert!(entries.len() < entries_count_max);
    debug_assert!(!relative_path.is_empty());
    Ok(())
}

fn observe_entry(
    current: &Dir,
    name: &str,
    relative_path: &str,
    metadata: &Metadata,
    limits: PreservedEvidenceTreeLimits,
) -> Result<EntryObservation, RunError> {
    let (kind, file_bytes, symlink_target) = if metadata.file_type().is_symlink() {
        let target = current
            .read_link_contents(name)
            .map_err(|error| evidence_io_error("reading symlink target", relative_path, error))?;
        let bytes = target.into_os_string().as_encoded_bytes().to_vec();
        let target_bytes = u32::try_from(bytes.len())
            .map_err(|_| RunError::Internal("preserved evidence symlink target length does not fit u32".to_string()))?;
        if target_bytes > limits.symlink_target_bytes_max {
            return Err(RunError::Internal(format!(
                "preserved evidence symlink target exceeds {} bytes at {relative_path}",
                limits.symlink_target_bytes_max
            )));
        }
        (EntryKind::Symlink, 0, Some(bytes))
    } else if metadata.is_dir() {
        (EntryKind::Directory, 0, None)
    } else if metadata.is_file() {
        (EntryKind::File, metadata.len(), None)
    } else {
        return Err(RunError::Internal(format!(
            "preserved evidence tree contains unsupported entry at {relative_path}"
        )));
    };
    assert!(file_bytes == 0 || kind == EntryKind::File);
    debug_assert!(symlink_target.is_none() || kind == EntryKind::Symlink);
    Ok(EntryObservation {
        relative_path: relative_path.to_string(),
        kind,
        mode: metadata_mode(metadata),
        file_bytes,
        symlink_target,
    })
}

fn account_file_bytes(
    entry: &EntryObservation,
    total_file_bytes: &mut u64,
    total_file_bytes_max: u64,
) -> Result<(), RunError> {
    if entry.kind != EntryKind::File {
        return Ok(());
    }
    *total_file_bytes = total_file_bytes
        .checked_add(entry.file_bytes)
        .ok_or_else(|| RunError::Internal("preserved evidence total file bytes overflowed u64".to_string()))?;
    if *total_file_bytes > total_file_bytes_max {
        return Err(RunError::Internal(format!(
            "preserved evidence total file bytes exceed {total_file_bytes_max} at {}",
            entry.relative_path
        )));
    }
    assert!(*total_file_bytes <= total_file_bytes_max);
    debug_assert!(entry.file_bytes <= *total_file_bytes);
    Ok(())
}

fn hash_entries(source_root: &ReleaseCapabilityRoot, entries: &[EntryObservation]) -> Result<String, RunError> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(DIGEST_DOMAIN);
    let entry_count = u64::try_from(entries.len())
        .map_err(|_| RunError::Internal("preserved evidence entry count does not fit u64".to_string()))?;
    hasher.update(&entry_count.to_le_bytes());
    for entry in entries {
        hash_entry(source_root, entry, &mut hasher)?;
    }
    let digest = hasher.finalize().to_hex().to_string();
    assert_eq!(digest.len(), blake3::OUT_LEN.saturating_mul(2));
    debug_assert!(entries.windows(2).all(|pair| pair[0].relative_path < pair[1].relative_path));
    Ok(digest)
}

fn hash_entry(
    source_root: &ReleaseCapabilityRoot,
    entry: &EntryObservation,
    hasher: &mut blake3::Hasher,
) -> Result<(), RunError> {
    update_entry_identity(hasher, entry)?;
    let (parent, name) = open_parent_directory_nofollow(source_root.dir(), &entry.relative_path)?;
    let metadata = parent
        .symlink_metadata(&name)
        .map_err(|error| evidence_io_error("revalidating entry metadata", &entry.relative_path, error))?;
    revalidate_entry(entry, &metadata)?;
    match entry.kind {
        EntryKind::Directory => {
            hasher.update(b"dir\0");
        }
        EntryKind::File => hash_file_entry(&parent, name, entry, hasher)?,
        EntryKind::Symlink => hash_symlink_entry(&parent, &name, entry, hasher)?,
    }
    Ok(())
}

fn hash_file_entry(
    parent: &Dir,
    name: OsString,
    entry: &EntryObservation,
    hasher: &mut blake3::Hasher,
) -> Result<(), RunError> {
    let mut file = open_file_nofollow(parent, name)
        .map_err(|error| evidence_io_error("opening no-follow file", &entry.relative_path, error))?;
    let opened = file
        .metadata()
        .map_err(|error| evidence_io_error("reading opened file metadata", &entry.relative_path, error))?;
    revalidate_entry(entry, &opened)?;
    hasher.update(b"file\0");
    hasher.update(&entry.file_bytes.to_le_bytes());
    hash_open_file(&mut file, hasher, &entry.relative_path, entry.file_bytes)
}

fn hash_symlink_entry(
    parent: &Dir,
    name: &OsString,
    entry: &EntryObservation,
    hasher: &mut blake3::Hasher,
) -> Result<(), RunError> {
    let target = parent
        .read_link_contents(name)
        .map_err(|error| evidence_io_error("revalidating symlink target", &entry.relative_path, error))?;
    let target_bytes = target.into_os_string().as_encoded_bytes().to_vec();
    let expected_target = entry.symlink_target.as_deref().ok_or_else(|| {
        RunError::Internal(format!("preserved evidence symlink target missing for {}", entry.relative_path))
    })?;
    if target_bytes != expected_target {
        return Err(RunError::Internal(format!(
            "preserved evidence symlink target changed at {}",
            entry.relative_path
        )));
    }
    let target_len = u64::try_from(target_bytes.len())
        .map_err(|_| RunError::Internal("preserved evidence symlink target length does not fit u64".to_string()))?;
    hasher.update(b"symlink\0");
    hasher.update(&target_len.to_le_bytes());
    hasher.update(&target_bytes);
    Ok(())
}

fn update_entry_identity(hasher: &mut blake3::Hasher, entry: &EntryObservation) -> Result<(), RunError> {
    let path_bytes = entry.relative_path.as_bytes();
    let path_len = u64::try_from(path_bytes.len())
        .map_err(|_| RunError::Internal("preserved evidence path length does not fit u64".to_string()))?;
    hasher.update(&path_len.to_le_bytes());
    hasher.update(path_bytes);
    hasher.update(&entry.mode.to_le_bytes());
    assert!(!entry.relative_path.is_empty());
    debug_assert!(!Path::new(&entry.relative_path).is_absolute());
    Ok(())
}

fn revalidate_entry(entry: &EntryObservation, metadata: &Metadata) -> Result<(), RunError> {
    let kind_matches = match entry.kind {
        EntryKind::Directory => metadata.is_dir(),
        EntryKind::File => metadata.is_file(),
        EntryKind::Symlink => metadata.file_type().is_symlink(),
    };
    if !kind_matches || metadata_mode(metadata) != entry.mode {
        return Err(RunError::Internal(format!(
            "preserved evidence entry type or mode changed at {}",
            entry.relative_path
        )));
    }
    if entry.kind == EntryKind::File && metadata.len() != entry.file_bytes {
        return Err(RunError::Internal(format!("preserved evidence file size changed at {}", entry.relative_path)));
    }
    assert!(kind_matches);
    debug_assert_eq!(metadata_mode(metadata), entry.mode);
    Ok(())
}

fn open_parent_directory_nofollow(root: &Dir, relative_path: &str) -> Result<(Dir, OsString), RunError> {
    ValidatedReleasePath::new(relative_path)
        .map_err(|error| RunError::Internal(format!("invalid preserved evidence path {relative_path}: {error:?}")))?;
    let mut components = relative_path.split('/').collect::<Vec<_>>();
    let name = components.pop().ok_or_else(|| RunError::Internal("preserved evidence path is empty".to_string()))?;
    let mut current = root
        .try_clone()
        .map_err(|error| evidence_io_error("cloning root capability", relative_path, error))?;
    for component in components {
        current = current
            .open_dir_nofollow(component)
            .map_err(|error| evidence_io_error("opening no-follow parent", relative_path, error))?;
    }
    assert!(!name.is_empty());
    debug_assert!(!relative_path.is_empty());
    Ok((current, OsString::from(name)))
}

fn open_file_nofollow(parent: &Dir, name: impl AsRef<Path>) -> std::io::Result<cap_std::fs::File> {
    let mut options = OpenOptions::new();
    options.read(true);
    options.follow(FollowSymlinks::No);
    parent.open_with(name, &options)
}

fn hash_open_file(
    file: &mut cap_std::fs::File,
    hasher: &mut blake3::Hasher,
    label: &str,
    expected_bytes: u64,
) -> Result<(), RunError> {
    let mut buffer = [0_u8; HASH_BUFFER_BYTES];
    let buffer_bytes = u64::try_from(HASH_BUFFER_BYTES)
        .map_err(|_| RunError::Internal("preserved evidence hash buffer size overflowed u64".to_string()))?;
    let mut hashed_bytes = 0_u64;
    while hashed_bytes < expected_bytes {
        let remaining_bytes = expected_bytes
            .checked_sub(hashed_bytes)
            .ok_or_else(|| RunError::Internal(format!("preserved evidence remaining bytes underflowed for {label}")))?;
        let read_capacity_bytes = if remaining_bytes >= buffer_bytes {
            HASH_BUFFER_BYTES
        } else {
            usize::try_from(remaining_bytes).map_err(|_| {
                RunError::Internal(format!("preserved evidence remaining bytes overflowed usize for {label}"))
            })?
        };
        let bytes_read = file
            .read(&mut buffer[..read_capacity_bytes])
            .map_err(|error| evidence_io_error("reading file", label, error))?;
        if bytes_read == 0 {
            return Err(RunError::Internal(format!(
                "preserved evidence file shrank at {label}: expected {expected_bytes}, read {hashed_bytes}"
            )));
        }
        hasher.update(&buffer[..bytes_read]);
        hashed_bytes =
            hashed_bytes
                .checked_add(u64::try_from(bytes_read).map_err(|_| {
                    RunError::Internal(format!("preserved evidence byte count overflowed u64 for {label}"))
                })?)
                .ok_or_else(|| RunError::Internal(format!("preserved evidence byte count overflowed for {label}")))?;
    }
    require_hash_eof(file, &mut buffer, label)?;
    assert_eq!(hashed_bytes, expected_bytes);
    debug_assert!(expected_bytes == 0 || hashed_bytes > 0);
    Ok(())
}

fn require_hash_eof(file: &mut cap_std::fs::File, buffer: &mut [u8], label: &str) -> Result<(), RunError> {
    let extra_count = file
        .read(&mut buffer[..1])
        .map_err(|error| evidence_io_error("checking file growth", label, error))?;
    if extra_count != 0 {
        return Err(RunError::Internal(format!("preserved evidence file grew while hashing {label}")));
    }
    assert_eq!(extra_count, 0);
    debug_assert!(!buffer.is_empty());
    Ok(())
}

#[cfg(unix)]
fn metadata_mode(metadata: &Metadata) -> u32 {
    MetadataExt::mode(metadata)
}

#[cfg(not(unix))]
fn metadata_mode(_metadata: &Metadata) -> u32 {
    0
}

fn evidence_io_error(action: &str, relative_path: &str, error: std::io::Error) -> RunError {
    let location = if relative_path.is_empty() {
        String::from(".")
    } else {
        relative_path.to_string()
    };
    RunError::Internal(format!("{action} for preserved evidence at {location}: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILE_BYTES: &[u8] = b"preserved-evidence";
    const TEST_ENTRY_COUNT_MAX: u32 = 8;
    const EXPECTED_OPAQUE_TREE_ENTRY_COUNT: u32 = 3;

    fn limits(entries_count_max: u32, total_file_bytes_max: u64) -> PreservedEvidenceTreeLimits {
        let release_limits = crunch_release_core::TreeCopyLimits::RELEASE_BUNDLE;
        PreservedEvidenceTreeLimits {
            entries_count_max,
            depth_count_max: release_limits.depth_count_max,
            path_bytes_max: release_limits.path_bytes_max,
            symlink_target_bytes_max: release_limits.path_bytes_max,
            total_file_bytes_max,
        }
    }

    fn write_file(path: &Path, bytes: &[u8]) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, bytes).unwrap();
    }

    // r[verify bootstrap_inventory.source_built_mantle_fixed_point]
    #[test]
    fn evidence_profile_supports_more_entries_than_release_copy_policy() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        std::fs::create_dir(&source).unwrap();
        let evidence_entry_count = crunch_release_core::RELEASE_TREE_COPY_MAX_ENTRIES_COUNT.checked_add(1).unwrap();
        for index in 0..evidence_entry_count {
            write_file(&source.join(format!("entry-{index}")), b"");
        }
        let sufficient = limits(evidence_entry_count, EMPTY_TREE_ACCOUNTED_SIZE_BYTES);
        let insufficient = limits(evidence_entry_count.checked_sub(1).unwrap(), EMPTY_TREE_ACCOUNTED_SIZE_BYTES);

        let identity = hash_preserved_evidence_tree(&source, sufficient).unwrap();
        let release_error = crate::release_tree_copy::hash_directory_tree(&source).unwrap_err();
        let evidence_error = hash_preserved_evidence_tree(&source, insufficient).unwrap_err();
        let release_limit_message =
            format!("observation count exceeds {}", crunch_release_core::RELEASE_TREE_COPY_MAX_ENTRIES_COUNT);

        assert_eq!(identity.entry_count, evidence_entry_count);
        assert_eq!(identity.total_file_bytes, EMPTY_TREE_ACCOUNTED_SIZE_BYTES);
        assert_eq!(identity.digest_blake3.len(), blake3::OUT_LEN.saturating_mul(2));
        assert!(release_error.to_string().contains(&release_limit_message));
        assert!(evidence_error.to_string().contains("preserved evidence entry count exceeds"));
    }

    // r[verify bootstrap_inventory.source_built_mantle_fixed_point]
    #[test]
    #[cfg(unix)]
    fn opaque_links_are_observed_but_remain_rejected_as_sources() {
        use std::os::unix::fs::symlink;

        const FIRST_ABSOLUTE_TARGET: &str = "/proof-local/first";
        const SECOND_ABSOLUTE_TARGET: &str = "/proof-local/second";
        const ESCAPING_TARGET: &str = "../negative-fixture";

        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        write_file(&source.join("artifact"), FILE_BYTES);
        symlink(FIRST_ABSOLUTE_TARGET, source.join("absolute-link")).unwrap();
        symlink(ESCAPING_TARGET, source.join("escaping-link")).unwrap();
        let file_bytes = u64::try_from(FILE_BYTES.len()).unwrap();
        let tree_limits = limits(TEST_ENTRY_COUNT_MAX, file_bytes);

        let first = hash_preserved_evidence_tree(&source, tree_limits).unwrap();
        let admission_error = crate::release_tree_copy::prepare_tree_copy(&source).unwrap_err();
        let byte_bound_error =
            hash_preserved_evidence_tree(&source, limits(TEST_ENTRY_COUNT_MAX, file_bytes.checked_sub(1).unwrap()))
                .unwrap_err();
        std::fs::remove_file(source.join("absolute-link")).unwrap();
        symlink(SECOND_ABSOLUTE_TARGET, source.join("absolute-link")).unwrap();
        let changed = hash_preserved_evidence_tree(&source, tree_limits).unwrap();

        assert_eq!(first.entry_count, EXPECTED_OPAQUE_TREE_ENTRY_COUNT);
        assert!(admission_error.to_string().contains("symlink"));
        assert!(byte_bound_error.to_string().contains("total file bytes exceed"));
        assert_ne!(first.digest_blake3, changed.digest_blake3);
    }

    // r[verify bootstrap_inventory.source_built_mantle_checkpoint_reuse]
    #[test]
    #[cfg(unix)]
    fn checkpoint_copy_preserves_opaque_links_and_detaches_file_bytes() {
        use std::os::unix::fs::symlink;

        const OPAQUE_TARGET: &str = "../outside-proof";
        const CHANGED_BYTES: &[u8] = b"changed-after-copy";

        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        let destination = temp.path().join("destination");
        write_file(&source.join("nested/artifact"), FILE_BYTES);
        symlink(OPAQUE_TARGET, source.join("nested/opaque-link")).unwrap();
        let total_bytes = u64::try_from(FILE_BYTES.len()).unwrap();
        let tree_limits = limits(TEST_ENTRY_COUNT_MAX, total_bytes);

        let copied = copy_preserved_evidence_tree(&source, &destination, tree_limits).unwrap();
        std::fs::write(source.join("nested/artifact"), CHANGED_BYTES).unwrap();
        let destination_bytes = std::fs::read(destination.join("nested/artifact")).unwrap();
        let destination_target = std::fs::read_link(destination.join("nested/opaque-link")).unwrap();

        assert_eq!(destination_bytes, FILE_BYTES);
        assert_eq!(destination_target, Path::new(OPAQUE_TARGET));
        assert_eq!(copied, hash_preserved_evidence_tree(&destination, tree_limits).unwrap());
    }

    // r[verify bootstrap_inventory.source_built_mantle_checkpoint_reuse]
    #[test]
    fn checkpoint_copy_rejects_an_existing_destination() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        let destination = temp.path().join("destination");
        write_file(&source.join("artifact"), FILE_BYTES);
        write_file(&destination.join("sentinel"), b"keep");
        let total_bytes = u64::try_from(FILE_BYTES.len()).unwrap();

        let error =
            copy_preserved_evidence_tree(&source, &destination, limits(TEST_ENTRY_COUNT_MAX, total_bytes)).unwrap_err();

        assert!(error.to_string().contains("destination exists"));
        assert_eq!(std::fs::read(destination.join("sentinel")).unwrap(), b"keep");
    }
}
