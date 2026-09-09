use std::ffi::OsString;
use std::io::Read;
use std::io::Write;
use std::path::Path;

use bounded_tree_cap::PreparedTree as SharedPreparedTree;
use bounded_tree_core::SymlinkPolicy;
use cap_fs_ext::DirExt;
use cap_fs_ext::FollowSymlinks;
use cap_fs_ext::OpenOptionsFollowExt;
use cap_std::fs::Dir;
use cap_std::fs::Metadata;
#[cfg(unix)]
use cap_std::fs::MetadataExt;
use cap_std::fs::OpenOptions;
#[cfg(unix)]
use cap_std::fs::OpenOptionsExt;
#[cfg(unix)]
use cap_std::fs::PermissionsExt;
use crunch_release_core::ChapterTransportEntryInput;
use crunch_release_core::ChapterTransportEntryKind;
use crunch_release_core::ChapterTransportMember;
use crunch_release_core::TreeCopyBlocker;
use crunch_release_core::TreeCopyLimits;
use crunch_release_core::TreeCopyPlan;
use crunch_release_core::TreeEntryKind;
use crunch_release_core::TreeEntryObservation;

use crate::errors::RunError;
use crate::release_capability::ReleaseCapabilityRoot;
use crate::release_capability::ReleaseRootKind;
use crate::release_capability::ValidatedReleasePath;

const TREE_COPY_BUFFER_BYTES: usize = 8_192;
const TREE_COPY_DIAGNOSTIC_BLOCKERS_MAX: usize = 16;
const EMPTY_DIRECTORY_ACCOUNTED_SIZE_BYTES: u64 = 1;
#[cfg(unix)]
const UNIX_PERMISSION_BITS_MASK: u32 = 0o7_777;
#[cfg(not(unix))]
const UNIX_PERMISSION_BITS_MASK: u32 = 0;
#[cfg(unix)]
const STAGING_FILE_MODE: u32 = 0o600;

#[derive(Debug)]
pub(crate) struct PreparedTreeCopy {
    source_root: ReleaseCapabilityRoot,
    shared: SharedPreparedTree,
    plan: TreeCopyPlan,
}

impl PreparedTreeCopy {
    pub(crate) fn artifact_identity(&self) -> Result<(u64, String), RunError> {
        hash_prepared_tree(self)
    }

    pub(crate) fn chapter_transport_entries(&self) -> Result<Vec<ChapterTransportEntryInput>, RunError> {
        let mut entries = Vec::with_capacity(self.plan.entries.len());
        for observation in &self.plan.entries {
            entries.push(chapter_transport_entry(self, observation)?);
        }
        debug_assert_eq!(entries.len(), self.plan.entries.len());
        debug_assert!(entries.capacity() >= entries.len());
        Ok(entries)
    }

    pub(crate) fn append_chapter_transport_member<W: Write>(
        &self,
        builder: &mut tar::Builder<W>,
        member: &ChapterTransportMember,
    ) -> Result<(), RunError> {
        append_chapter_transport_member(self, builder, member)
    }

    #[cfg(test)]
    fn plan(&self) -> &TreeCopyPlan {
        &self.plan
    }
}

// r[impl mantle.release_provenance.bundle_tree_copy.no_follow]
// r[impl mantle.release_provenance.bundle_tree_copy.plan]
pub(crate) fn prepare_tree_copy(source_dir: &Path) -> Result<PreparedTreeCopy, RunError> {
    prepare_tree_copy_with_limits(source_dir, TreeCopyLimits::RELEASE_BUNDLE)
}

fn prepare_tree_copy_with_limits(source_dir: &Path, limits: TreeCopyLimits) -> Result<PreparedTreeCopy, RunError> {
    let source_root = ReleaseCapabilityRoot::open_ambient_nofollow(ReleaseRootKind::ReleaseTreeSource, source_dir)
        .map_err(|error| {
            RunError::Internal(format!("opening no-follow tree source root {}: {error}", source_dir.display()))
        })?;
    let shared_limits = crunch_release_core::bounded_tree_limits(limits).map_err(tree_plan_error)?;
    let shared = bounded_tree_cap::prepare(source_root.dir(), shared_limits, SymlinkPolicy::PreserveInternal)
        .map_err(|error| shared_prepare_error(&error, limits))?;
    let plan = crunch_release_core::bounded_tree_plan(shared.plan()).map_err(tree_plan_error)?;
    let entries_count_max = usize::try_from(limits.entries_count_max)
        .map_err(|_| RunError::Internal("tree copy entry limit does not fit usize".to_string()))?;
    assert_eq!(source_root.kind(), ReleaseRootKind::ReleaseTreeSource);
    assert!(plan.entries.len() <= entries_count_max);
    Ok(PreparedTreeCopy {
        source_root,
        shared,
        plan,
    })
}

// r[impl mantle.release_provenance.bundle_tree_copy.destination_confinement]
pub(crate) fn copy_directory_tree(source_dir: &Path, dest_dir: &Path) -> Result<(), RunError> {
    let prepared = prepare_tree_copy(source_dir)?;
    copy_prepared_tree(prepared, dest_dir)
}

pub(crate) fn copy_prepared_tree(prepared: PreparedTreeCopy, dest_dir: &Path) -> Result<(), RunError> {
    let destination_root = prepare_empty_destination_root(dest_dir)?;
    execute_tree_copy_plan(&prepared, &destination_root)
}

pub(crate) fn copy_prepared_tree_into_capability(
    prepared: PreparedTreeCopy,
    destination_root: &ReleaseCapabilityRoot,
    relative_path: &Path,
) -> Result<(), RunError> {
    let relative = relative_path.to_str().ok_or_else(|| {
        RunError::Internal(format!("tree copy destination is not valid UTF-8: {}", relative_path.display()))
    })?;
    let validated = ValidatedReleasePath::new(relative)
        .map_err(|error| RunError::Internal(format!("invalid tree copy destination path {relative}: {error:?}")))?;
    let child_root = destination_root
        .create_dir_all_relative_nofollow(ReleaseRootKind::ReleaseTreeDestination, &validated)
        .map_err(|error| tree_io_error("creating capability-scoped destination root", relative, error))?;
    if !child_root
        .is_empty()
        .map_err(|error| tree_io_error("reading capability-scoped destination root", relative, error))?
    {
        return Err(RunError::Internal(format!("tree copy destination root must be empty: {relative}")));
    }
    assert_eq!(destination_root.kind(), ReleaseRootKind::ReleaseEvidence);
    assert_eq!(child_root.kind(), ReleaseRootKind::ReleaseTreeDestination);
    execute_tree_copy_plan(&prepared, &child_root)
}

pub(crate) fn copy_file_into_capability(
    source_path: &Path,
    destination_root: &ReleaseCapabilityRoot,
    relative_path: &Path,
) -> Result<(), RunError> {
    let absolute = std::path::absolute(source_path)
        .map_err(|error| RunError::Internal(format!("resolving file path {}: {error}", source_path.display())))?;
    let file_name = absolute
        .file_name()
        .ok_or_else(|| RunError::Internal(format!("file path has no final component: {}", source_path.display())))?;
    let parent_path = absolute
        .parent()
        .ok_or_else(|| RunError::Internal(format!("file path has no parent: {}", source_path.display())))?;
    let source_root = ReleaseCapabilityRoot::open_ambient_nofollow(ReleaseRootKind::ReleaseTreeSource, parent_path)
        .map_err(|error| RunError::Internal(format!("opening file parent {}: {error}", parent_path.display())))?;
    copy_capability_file(&source_root, file_name, destination_root, relative_path)
}

fn copy_capability_file(
    source_root: &ReleaseCapabilityRoot,
    file_name: &std::ffi::OsStr,
    destination_root: &ReleaseCapabilityRoot,
    relative_path: &Path,
) -> Result<(), RunError> {
    let observed = source_root.dir().symlink_metadata(file_name).map_err(|error| {
        tree_io_error("reading no-follow source file metadata", &source_label(relative_path), error)
    })?;
    let expected_mode = metadata_mode(&observed);
    validate_source_metadata(&source_label(relative_path), &observed, TreeEntryKind::File, expected_mode)?;
    let mut source = open_file_nofollow(source_root.dir(), file_name, false, 0)
        .map_err(|error| tree_io_error("opening no-follow source file", &source_label(relative_path), error))?;
    let opened = source
        .metadata()
        .map_err(|error| tree_io_error("reading opened source file metadata", &source_label(relative_path), error))?;
    validate_source_metadata(&source_label(relative_path), &opened, TreeEntryKind::File, expected_mode)?;
    let relative = source_label(relative_path);
    let validated = ValidatedReleasePath::new(&relative)
        .map_err(|error| RunError::Internal(format!("invalid bundle file destination {relative}: {error:?}")))?;
    let mut destination = destination_root
        .open_new_file_nofollow(&validated)
        .map_err(|error| tree_io_error("creating no-follow bundle file", &relative, error))?;
    copy_open_file_bytes(&mut source, &mut destination, &relative, opened.len())?;
    set_file_mode(&destination, expected_mode, &relative)?;
    assert_eq!(source_root.kind(), ReleaseRootKind::ReleaseTreeSource);
    assert_eq!(destination_root.kind(), ReleaseRootKind::ReleaseEvidence);
    Ok(())
}

fn source_label(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

pub(crate) fn hash_directory_tree(path: &Path) -> Result<(u64, String), RunError> {
    hash_directory_tree_with_limits(path, TreeCopyLimits::RELEASE_BUNDLE)
}

pub(crate) fn hash_directory_tree_with_limits(path: &Path, limits: TreeCopyLimits) -> Result<(u64, String), RunError> {
    let prepared = prepare_tree_copy_with_limits(path, limits)
        .map_err(|error| RunError::Internal(format!("expected directory artifact {}: {error}", path.display())))?;
    hash_prepared_tree(&prepared)
}

pub(crate) fn hash_file_nofollow(path: &Path) -> Result<(u64, String), RunError> {
    let absolute = std::path::absolute(path)
        .map_err(|error| RunError::Internal(format!("resolving file path {}: {error}", path.display())))?;
    let file_name = absolute
        .file_name()
        .ok_or_else(|| RunError::Internal(format!("file path has no final component: {}", path.display())))?;
    let parent_path = absolute
        .parent()
        .ok_or_else(|| RunError::Internal(format!("file path has no parent: {}", path.display())))?;
    let parent = ReleaseCapabilityRoot::open_ambient_nofollow(ReleaseRootKind::ReleaseTreeSource, parent_path)
        .map_err(|error| {
            RunError::Internal(format!("opening no-follow file parent {}: {error}", parent_path.display()))
        })?;
    let mut file = open_file_nofollow(parent.dir(), file_name, false, 0)
        .map_err(|error| RunError::Internal(format!("opening no-follow file {}: {error}", path.display())))?;
    let metadata = file
        .metadata()
        .map_err(|error| RunError::Internal(format!("reading file metadata {}: {error}", path.display())))?;
    if !metadata.is_file() {
        return Err(RunError::Internal(format!("expected regular file artifact: {}", path.display())));
    }
    let mut hasher = blake3::Hasher::new();
    hash_open_file(&mut file, &mut hasher, path.to_string_lossy().as_ref(), metadata.len())?;
    debug_assert_eq!(parent.kind(), ReleaseRootKind::ReleaseTreeSource);
    debug_assert!(metadata.is_file());
    Ok((metadata.len(), hasher.finalize().to_hex().to_string()))
}

fn shared_prepare_error(error: &bounded_tree_cap::ShellError, limits: TreeCopyLimits) -> RunError {
    if let bounded_tree_cap::ShellErrorKind::PlanRejected(plan) = error.kind() {
        return tree_plan_error(crunch_release_core::bounded_tree_blockers(plan));
    }
    let path = shared_error_path(error);
    if let bounded_tree_cap::ShellErrorKind::LimitExceeded(kind) = error.kind() {
        let message = match kind {
            bounded_tree_core::LimitKind::Entries => {
                format!("tree copy observation count exceeds {} at {path}", limits.entries_count_max)
            }
            bounded_tree_core::LimitKind::Depth => {
                format!("tree copy path depth exceeds {} at {path}", limits.depth_count_max)
            }
            bounded_tree_core::LimitKind::PathBytes => {
                format!("tree copy path bytes exceed {} at {path}", limits.path_bytes_max)
            }
            bounded_tree_core::LimitKind::SymlinkTargetBytes => {
                format!("tree copy symlink target exceeds {} bytes at {path}", limits.path_bytes_max)
            }
            bounded_tree_core::LimitKind::FileBytes => "tree copy file bytes exceed compatibility limit".to_string(),
            bounded_tree_core::LimitKind::TotalBytes => "tree copy total bytes exceed compatibility limit".to_string(),
            _ => "tree copy resource exceeds configured limit".to_string(),
        };
        return RunError::Internal(message);
    }
    shared_tree_error("observing and planning release tree", error)
}

fn shared_error_path(error: &bounded_tree_cap::ShellError) -> String {
    error
        .path_components()
        .iter()
        .map(|component| String::from_utf8_lossy(component))
        .collect::<Vec<_>>()
        .join("/")
}

fn shared_tree_error(action: &str, error: &bounded_tree_cap::ShellError) -> RunError {
    let relative_path = shared_error_path(error);
    let class = match error.kind() {
        bounded_tree_cap::ShellErrorKind::LimitExceeded(_) => "tree copy resource exceeds configured limit",
        bounded_tree_cap::ShellErrorKind::PlanRejected(plan)
            if plan
                .blockers()
                .iter()
                .any(|blocker| blocker.kind() == bounded_tree_core::BlockerKind::UnsupportedKind) =>
        {
            "tree copy plan rejected unsupported entry"
        }
        bounded_tree_cap::ShellErrorKind::PlanRejected(_) => "tree copy plan rejected",
        bounded_tree_cap::ShellErrorKind::SourceChanged(bounded_tree_cap::SourceChangeKind::Kind) => {
            "tree copy source type drift"
        }
        bounded_tree_cap::ShellErrorKind::SourceChanged(bounded_tree_cap::SourceChangeKind::Length)
        | bounded_tree_cap::ShellErrorKind::SourceChanged(bounded_tree_cap::SourceChangeKind::Content) => {
            "tree copy source size drift"
        }
        bounded_tree_cap::ShellErrorKind::SourceChanged(bounded_tree_cap::SourceChangeKind::Mode) => {
            "tree copy source mode drift"
        }
        bounded_tree_cap::ShellErrorKind::SourceChanged(bounded_tree_cap::SourceChangeKind::SymlinkTarget) => {
            "tree copy source target drift"
        }
        bounded_tree_cap::ShellErrorKind::SourceChanged(bounded_tree_cap::SourceChangeKind::Missing) => {
            "tree copy source is missing"
        }
        bounded_tree_cap::ShellErrorKind::DestinationNotEmpty => "tree copy destination root must be empty",
        bounded_tree_cap::ShellErrorKind::UnsupportedPlatformPath => "tree copy path is unsupported on this platform",
        bounded_tree_cap::ShellErrorKind::Io { .. } => "tree copy capability operation failed",
    };
    RunError::Internal(format!("{class} while {action} at {relative_path}"))
}

fn tree_plan_error(blockers: Vec<TreeCopyBlocker>) -> RunError {
    let total_count = blockers.len();
    let rendered = blockers
        .iter()
        .take(TREE_COPY_DIAGNOSTIC_BLOCKERS_MAX)
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("; ");
    RunError::Internal(format!("tree copy plan rejected with {total_count} blocker(s): {rendered}"))
}

fn prepare_empty_destination_root(dest_dir: &Path) -> Result<ReleaseCapabilityRoot, RunError> {
    let root =
        ReleaseCapabilityRoot::create_ambient_dir_all_nofollow(ReleaseRootKind::ReleaseTreeDestination, dest_dir)
            .map_err(|error| {
                RunError::Internal(format!("opening no-follow tree destination root {}: {error}", dest_dir.display()))
            })?;
    if !root
        .is_empty()
        .map_err(|error| RunError::Internal(format!("reading tree destination root {}: {error}", dest_dir.display())))?
    {
        return Err(RunError::Internal(format!("tree copy destination root must be empty: {}", dest_dir.display())));
    }
    assert_eq!(root.kind(), ReleaseRootKind::ReleaseTreeDestination);
    Ok(root)
}

// r[impl mantle.release_provenance.bundle_tree_copy.destination_confinement]
// r[impl mantle.release_provenance.bundle_tree_copy.fixtures.negative.type_drift]
fn execute_tree_copy_plan(
    prepared: &PreparedTreeCopy,
    destination_root: &ReleaseCapabilityRoot,
) -> Result<(), RunError> {
    assert_eq!(prepared.source_root.kind(), ReleaseRootKind::ReleaseTreeSource);
    assert_eq!(destination_root.kind(), ReleaseRootKind::ReleaseTreeDestination);
    bounded_tree_cap::execute(&prepared.shared, destination_root.dir())
        .map_err(|error| shared_tree_error("executing release tree copy", &error))?;
    Ok(())
}

fn revalidate_source_path(
    source_root: &ReleaseCapabilityRoot,
    relative_path: &str,
    expected_kind: TreeEntryKind,
    expected_mode: u32,
    expected_target: Option<&String>,
) -> Result<(), RunError> {
    let (parent, name) = open_parent_directory_nofollow(source_root.dir(), relative_path)?;
    let metadata = parent
        .symlink_metadata(&name)
        .map_err(|error| tree_io_error("revalidating source entry", relative_path, error))?;
    validate_source_metadata(relative_path, &metadata, expected_kind, expected_mode)?;
    if expected_kind == TreeEntryKind::Symlink {
        let actual_target = read_symlink_target(&parent, &name, relative_path)?;
        let expected_target = expected_target
            .ok_or_else(|| RunError::Internal(format!("planned symlink target missing for {relative_path}")))?;
        if &actual_target != expected_target {
            return Err(RunError::Internal(format!(
                "tree copy source target drift for {relative_path}: expected {expected_target}, found {actual_target}"
            )));
        }
    }
    debug_assert_eq!(metadata_kind(&metadata), expected_kind);
    debug_assert_eq!(metadata_mode(&metadata), expected_mode);
    Ok(())
}

fn validate_source_metadata(
    relative_path: &str,
    metadata: &Metadata,
    expected_kind: TreeEntryKind,
    expected_mode: u32,
) -> Result<(), RunError> {
    let actual_kind = metadata_kind(metadata);
    if actual_kind != expected_kind {
        return Err(RunError::Internal(format!(
            "tree copy source type drift for {relative_path}: expected {expected_kind:?}, found {actual_kind:?}"
        )));
    }
    let actual_mode = metadata_mode(metadata);
    if actual_mode != expected_mode {
        return Err(RunError::Internal(format!(
            "tree copy source mode drift for {relative_path}: expected {expected_mode:o}, found {actual_mode:o}"
        )));
    }
    Ok(())
}

fn open_parent_directory_nofollow(root: &Dir, relative_path: &str) -> Result<(Dir, OsString), RunError> {
    let label = "source";
    ValidatedReleasePath::new(relative_path).map_err(|error| {
        RunError::Internal(format!("invalid planned tree copy {label} path {relative_path}: {error:?}"))
    })?;
    let mut components = relative_path.split('/').collect::<Vec<_>>();
    let name = components
        .pop()
        .ok_or_else(|| RunError::Internal(format!("planned tree copy {label} path is empty")))?;
    let mut current = root
        .try_clone()
        .map_err(|error| RunError::Internal(format!("cloning {label} root capability: {error}")))?;
    for component in components {
        current = current
            .open_dir_nofollow(component)
            .map_err(|error| tree_io_error(format!("opening no-follow {label} parent"), relative_path, error))?;
    }
    Ok((current, OsString::from(name)))
}

fn open_file_nofollow(
    parent: &Dir,
    name: impl AsRef<Path>,
    create_new: bool,
    mode: u32,
) -> std::io::Result<cap_std::fs::File> {
    let mut options = OpenOptions::new();
    if create_new {
        options.write(true).create_new(true);
        set_open_file_mode(&mut options, mode);
    } else {
        options.read(true);
    }
    options.follow(FollowSymlinks::No);
    parent.open_with(name, &options)
}

fn copy_open_file_bytes(
    source: &mut cap_std::fs::File,
    destination: &mut cap_std::fs::File,
    relative_path: &str,
    expected_bytes: u64,
) -> Result<u64, RunError> {
    let mut buffer = [0_u8; TREE_COPY_BUFFER_BYTES];
    let buffer_bytes = u64::try_from(TREE_COPY_BUFFER_BYTES)
        .map_err(|_| RunError::Internal("tree copy buffer size overflowed u64".to_string()))?;
    let mut copied_bytes = 0_u64;
    while copied_bytes < expected_bytes {
        let remaining_bytes = expected_bytes
            .checked_sub(copied_bytes)
            .ok_or_else(|| RunError::Internal(format!("tree copy remaining bytes underflowed for {relative_path}")))?;
        let read_capacity_bytes = if remaining_bytes >= buffer_bytes {
            TREE_COPY_BUFFER_BYTES
        } else {
            usize::try_from(remaining_bytes).map_err(|_| {
                RunError::Internal(format!("tree copy remaining bytes overflowed usize for {relative_path}"))
            })?
        };
        let read_count = source
            .read(&mut buffer[..read_capacity_bytes])
            .map_err(|error| tree_io_error("reading source file", relative_path, error))?;
        if read_count == 0 {
            return Err(RunError::Internal(format!(
                "tree copy source size drift for {relative_path}: expected {expected_bytes}, read {copied_bytes}"
            )));
        }
        destination
            .write_all(&buffer[..read_count])
            .map_err(|error| tree_io_error("writing destination file", relative_path, error))?;
        copied_bytes =
            copied_bytes
                .checked_add(u64::try_from(read_count).map_err(|_| {
                    RunError::Internal(format!("tree copy byte count overflowed u64 for {relative_path}"))
                })?)
                .ok_or_else(|| RunError::Internal(format!("tree copy byte count overflowed for {relative_path}")))?;
    }
    let extra_count = source
        .read(&mut buffer[..1])
        .map_err(|error| tree_io_error("checking source file growth", relative_path, error))?;
    if extra_count != 0 {
        return Err(RunError::Internal(format!("tree copy source grew while reading {relative_path}")));
    }
    destination
        .flush()
        .map_err(|error| tree_io_error("flushing destination file", relative_path, error))?;
    debug_assert_eq!(copied_bytes, expected_bytes);
    debug_assert_eq!(extra_count, 0);
    Ok(copied_bytes)
}

fn hash_prepared_tree(prepared: &PreparedTreeCopy) -> Result<(u64, String), RunError> {
    let mut hasher = blake3::Hasher::new();
    let mut total_file_bytes = 0_u64;
    for entry in &prepared.plan.entries {
        total_file_bytes = total_file_bytes
            .checked_add(hash_prepared_tree_entry(prepared, entry, &mut hasher)?)
            .ok_or_else(|| RunError::Internal("tree hash total file bytes overflowed u64".to_string()))?;
    }
    if total_file_bytes == 0 {
        total_file_bytes = EMPTY_DIRECTORY_ACCOUNTED_SIZE_BYTES;
    }
    Ok((total_file_bytes, hasher.finalize().to_hex().to_string()))
}

fn hash_prepared_tree_entry(
    prepared: &PreparedTreeCopy,
    entry: &TreeEntryObservation,
    hasher: &mut blake3::Hasher,
) -> Result<u64, RunError> {
    update_entry_identity_hash(hasher, entry)?;
    debug_assert!(!entry.relative_path.is_empty());
    debug_assert_ne!(entry.kind, TreeEntryKind::Unsupported);
    match entry.kind {
        TreeEntryKind::Directory => {
            revalidate_source_path(
                &prepared.source_root,
                &entry.relative_path,
                TreeEntryKind::Directory,
                entry.mode,
                None,
            )?;
            hasher.update(b"dir\0");
            Ok(0)
        }
        TreeEntryKind::File => hash_prepared_file_entry(prepared, entry, hasher),
        TreeEntryKind::Symlink => hash_prepared_symlink_entry(prepared, entry, hasher),
        TreeEntryKind::Unsupported => {
            Err(RunError::Internal(format!("unsupported planned tree hash entry: {}", entry.relative_path)))
        }
    }
}

fn update_entry_identity_hash(hasher: &mut blake3::Hasher, entry: &TreeEntryObservation) -> Result<(), RunError> {
    let relative_bytes = entry.relative_path.as_bytes();
    let relative_len_bytes = u64::try_from(relative_bytes.len())
        .map_err(|_| RunError::Internal("tree entry relative path length overflowed u64".to_string()))?;
    hasher.update(&relative_len_bytes.to_le_bytes());
    hasher.update(relative_bytes);
    hasher.update(&entry.mode.to_le_bytes());
    Ok(())
}

fn hash_prepared_file_entry(
    prepared: &PreparedTreeCopy,
    entry: &TreeEntryObservation,
    hasher: &mut blake3::Hasher,
) -> Result<u64, RunError> {
    let (parent, name) = open_parent_directory_nofollow(prepared.source_root.dir(), &entry.relative_path)?;
    let mut file = open_file_nofollow(&parent, name, false, 0)
        .map_err(|error| tree_io_error("opening no-follow tree hash file", &entry.relative_path, error))?;
    let metadata = file
        .metadata()
        .map_err(|error| tree_io_error("reading tree hash file metadata", &entry.relative_path, error))?;
    validate_source_metadata(&entry.relative_path, &metadata, TreeEntryKind::File, entry.mode)?;
    hasher.update(b"file\0");
    hasher.update(&metadata.len().to_le_bytes());
    hash_open_file(&mut file, hasher, &entry.relative_path, metadata.len())?;
    debug_assert!(metadata.is_file());
    debug_assert_eq!(metadata_mode(&metadata), entry.mode);
    Ok(metadata.len())
}

fn hash_prepared_symlink_entry(
    prepared: &PreparedTreeCopy,
    entry: &TreeEntryObservation,
    hasher: &mut blake3::Hasher,
) -> Result<u64, RunError> {
    let target = entry
        .symlink_target
        .as_ref()
        .ok_or_else(|| RunError::Internal(format!("planned symlink target missing for {}", entry.relative_path)))?;
    revalidate_source_path(
        &prepared.source_root,
        &entry.relative_path,
        TreeEntryKind::Symlink,
        entry.mode,
        Some(target),
    )?;
    let target_bytes = target.as_bytes();
    debug_assert!(!entry.relative_path.is_empty());
    debug_assert!(!target_bytes.is_empty());
    hasher.update(b"symlink\0");
    let target_len_bytes = u64::try_from(target_bytes.len())
        .map_err(|_| RunError::Internal("tree symlink target length overflowed u64".to_string()))?;
    hasher.update(&target_len_bytes.to_le_bytes());
    hasher.update(target_bytes);
    Ok(0)
}

fn hash_open_file(
    file: &mut cap_std::fs::File,
    hasher: &mut blake3::Hasher,
    label: &str,
    expected_bytes: u64,
) -> Result<(), RunError> {
    let mut buffer = [0_u8; TREE_COPY_BUFFER_BYTES];
    let buffer_bytes = u64::try_from(TREE_COPY_BUFFER_BYTES)
        .map_err(|_| RunError::Internal("tree hash buffer size overflowed u64".to_string()))?;
    let mut hashed_bytes = 0_u64;
    while hashed_bytes < expected_bytes {
        let remaining_bytes = expected_bytes
            .checked_sub(hashed_bytes)
            .ok_or_else(|| RunError::Internal(format!("hash remaining bytes underflowed for {label}")))?;
        let read_capacity_bytes = if remaining_bytes >= buffer_bytes {
            TREE_COPY_BUFFER_BYTES
        } else {
            usize::try_from(remaining_bytes)
                .map_err(|_| RunError::Internal(format!("hash remaining bytes overflowed usize for {label}")))?
        };
        let bytes_read = file
            .read(&mut buffer[..read_capacity_bytes])
            .map_err(|error| RunError::Internal(format!("reading {label}: {error}")))?;
        if bytes_read == 0 {
            return Err(RunError::Internal(format!(
                "tree hash source size drift for {label}: expected {expected_bytes}, read {hashed_bytes}"
            )));
        }
        hasher.update(&buffer[..bytes_read]);
        hashed_bytes = hashed_bytes
            .checked_add(
                u64::try_from(bytes_read)
                    .map_err(|_| RunError::Internal(format!("tree hash byte count overflowed u64 for {label}")))?,
            )
            .ok_or_else(|| RunError::Internal(format!("tree hash byte count overflowed for {label}")))?;
    }
    let extra_count = file
        .read(&mut buffer[..1])
        .map_err(|error| RunError::Internal(format!("checking source file growth for {label}: {error}")))?;
    if extra_count != 0 {
        return Err(RunError::Internal(format!("tree hash source grew while reading {label}")));
    }
    debug_assert_eq!(hashed_bytes, expected_bytes);
    debug_assert_eq!(extra_count, 0);
    Ok(())
}

fn read_symlink_target(parent: &Dir, name: &OsString, relative_path: &str) -> Result<String, RunError> {
    parent
        .read_link_contents(name)
        .map_err(|error| tree_io_error("reading revalidated source symlink", relative_path, error))?
        .into_os_string()
        .into_string()
        .map_err(|_| RunError::Internal(format!("tree copy symlink target is not valid UTF-8: {relative_path}")))
}

fn metadata_kind(metadata: &Metadata) -> TreeEntryKind {
    if metadata.file_type().is_symlink() {
        return TreeEntryKind::Symlink;
    }
    if metadata.is_dir() {
        return TreeEntryKind::Directory;
    }
    if metadata.is_file() {
        return TreeEntryKind::File;
    }
    TreeEntryKind::Unsupported
}

#[cfg(unix)]
fn metadata_mode(metadata: &Metadata) -> u32 {
    MetadataExt::mode(metadata)
}

#[cfg(not(unix))]
fn metadata_mode(_metadata: &Metadata) -> u32 {
    0
}

#[cfg(unix)]
fn set_open_file_mode(options: &mut OpenOptions, _mode: u32) {
    options.mode(STAGING_FILE_MODE);
}

#[cfg(not(unix))]
fn set_open_file_mode(_options: &mut OpenOptions, _mode: u32) {}

#[cfg(unix)]
fn set_file_mode(file: &cap_std::fs::File, mode: u32, relative_path: &str) -> Result<(), RunError> {
    let permissions = cap_std::fs::Permissions::from_mode(mode & UNIX_PERMISSION_BITS_MASK);
    file.set_permissions(permissions)
        .map_err(|error| tree_io_error("setting destination file mode", relative_path, error))
}

#[cfg(not(unix))]
fn set_file_mode(_file: &cap_std::fs::File, _mode: u32, _relative_path: &str) -> Result<(), RunError> {
    Ok(())
}

fn tree_io_error(action: impl AsRef<str>, relative_path: &str, error: std::io::Error) -> RunError {
    RunError::Internal(format!("{} {relative_path}: {error}", action.as_ref()))
}

fn chapter_transport_entry(
    prepared: &PreparedTreeCopy,
    observation: &TreeEntryObservation,
) -> Result<ChapterTransportEntryInput, RunError> {
    let size_bytes = chapter_transport_entry_size(prepared, observation)?;
    let entry = ChapterTransportEntryInput {
        relative_path: observation.relative_path.clone(),
        kind: chapter_transport_kind(observation.kind),
        mode: observation.mode & UNIX_PERMISSION_BITS_MASK,
        size_bytes,
        symlink_target: observation.symlink_target.clone(),
    };
    debug_assert!(!entry.relative_path.is_empty());
    debug_assert!(entry.size_bytes == 0 || entry.kind == ChapterTransportEntryKind::File);
    Ok(entry)
}

fn chapter_transport_entry_size(
    prepared: &PreparedTreeCopy,
    observation: &TreeEntryObservation,
) -> Result<u64, RunError> {
    if observation.kind != TreeEntryKind::File {
        revalidate_source_path(
            &prepared.source_root,
            &observation.relative_path,
            observation.kind,
            observation.mode,
            observation.symlink_target.as_ref(),
        )?;
        return Ok(0);
    }
    let (parent, name) = open_parent_directory_nofollow(prepared.source_root.dir(), &observation.relative_path)?;
    let file = open_file_nofollow(&parent, name, false, 0)
        .map_err(|error| tree_io_error("opening chapter transport source file", &observation.relative_path, error))?;
    let metadata = file.metadata().map_err(|error| {
        tree_io_error("reading chapter transport source metadata", &observation.relative_path, error)
    })?;
    validate_source_metadata(&observation.relative_path, &metadata, TreeEntryKind::File, observation.mode)?;
    Ok(metadata.len())
}

fn chapter_transport_kind(kind: TreeEntryKind) -> ChapterTransportEntryKind {
    match kind {
        TreeEntryKind::Directory => ChapterTransportEntryKind::Directory,
        TreeEntryKind::File => ChapterTransportEntryKind::File,
        TreeEntryKind::Symlink => ChapterTransportEntryKind::Symlink,
        TreeEntryKind::Unsupported => ChapterTransportEntryKind::Unsupported,
    }
}

fn append_chapter_transport_member<W: Write>(
    prepared: &PreparedTreeCopy,
    builder: &mut tar::Builder<W>,
    member: &ChapterTransportMember,
) -> Result<(), RunError> {
    match member.kind {
        ChapterTransportEntryKind::Directory => append_transport_directory(prepared, builder, member),
        ChapterTransportEntryKind::File => append_transport_file(prepared, builder, member),
        ChapterTransportEntryKind::Symlink => append_transport_symlink(prepared, builder, member),
        ChapterTransportEntryKind::Unsupported => {
            Err(RunError::Internal(format!("unsupported chapter transport entry kind for {}", member.relative_path)))
        }
    }
}

fn append_transport_directory<W: Write>(
    prepared: &PreparedTreeCopy,
    builder: &mut tar::Builder<W>,
    member: &ChapterTransportMember,
) -> Result<(), RunError> {
    revalidate_transport_member(prepared, member, TreeEntryKind::Directory)?;
    let mut header = transport_header(member, tar::EntryType::Directory)?;
    builder
        .append_data(&mut header, &member.relative_path, std::io::empty())
        .map_err(|error| tree_io_error("appending chapter transport directory", &member.relative_path, error))?;
    debug_assert_eq!(member.size_bytes, 0);
    debug_assert!(member.symlink_target.is_none());
    Ok(())
}

fn append_transport_file<W: Write>(
    prepared: &PreparedTreeCopy,
    builder: &mut tar::Builder<W>,
    member: &ChapterTransportMember,
) -> Result<(), RunError> {
    let (parent, name) = open_parent_directory_nofollow(prepared.source_root.dir(), &member.relative_path)?;
    let mut file = open_file_nofollow(&parent, name, false, 0)
        .map_err(|error| tree_io_error("opening chapter transport file", &member.relative_path, error))?;
    let metadata = file
        .metadata()
        .map_err(|error| tree_io_error("reading chapter transport file metadata", &member.relative_path, error))?;
    validate_transport_metadata(member, &metadata, TreeEntryKind::File)?;
    let mut header = transport_header(member, tar::EntryType::Regular)?;
    builder
        .append_data(&mut header, &member.relative_path, &mut file)
        .map_err(|error| tree_io_error("appending chapter transport file", &member.relative_path, error))?;
    let mut trailing = [0_u8; 1];
    let trailing_count = file
        .read(&mut trailing)
        .map_err(|error| tree_io_error("checking chapter transport source growth", &member.relative_path, error))?;
    if trailing_count != 0 {
        return Err(RunError::Internal(format!(
            "chapter transport source grew while packing {}",
            member.relative_path
        )));
    }
    debug_assert_eq!(metadata.len(), member.size_bytes);
    debug_assert_eq!(trailing_count, 0);
    Ok(())
}

fn append_transport_symlink<W: Write>(
    prepared: &PreparedTreeCopy,
    builder: &mut tar::Builder<W>,
    member: &ChapterTransportMember,
) -> Result<(), RunError> {
    revalidate_transport_member(prepared, member, TreeEntryKind::Symlink)?;
    let target = member.symlink_target.as_ref().ok_or_else(|| {
        RunError::Internal(format!("chapter transport symlink target is missing for {}", member.relative_path))
    })?;
    let mut header = transport_header(member, tar::EntryType::Symlink)?;
    header
        .set_link_name(target)
        .map_err(|error| tree_io_error("setting chapter transport link target", &member.relative_path, error))?;
    header.set_cksum();
    builder
        .append_data(&mut header, &member.relative_path, std::io::empty())
        .map_err(|error| tree_io_error("appending chapter transport symlink", &member.relative_path, error))?;
    debug_assert_eq!(member.size_bytes, 0);
    debug_assert!(!target.is_empty());
    Ok(())
}

fn revalidate_transport_member(
    prepared: &PreparedTreeCopy,
    member: &ChapterTransportMember,
    expected_kind: TreeEntryKind,
) -> Result<(), RunError> {
    let (parent, name) = open_parent_directory_nofollow(prepared.source_root.dir(), &member.relative_path)?;
    let metadata = parent
        .symlink_metadata(&name)
        .map_err(|error| tree_io_error("revalidating chapter transport member", &member.relative_path, error))?;
    validate_transport_metadata(member, &metadata, expected_kind)?;
    if expected_kind == TreeEntryKind::Symlink {
        let actual_target = read_symlink_target(&parent, &name, &member.relative_path)?;
        if member.symlink_target.as_deref() != Some(actual_target.as_str()) {
            return Err(RunError::Internal(format!(
                "chapter transport source link target drift for {}",
                member.relative_path
            )));
        }
    }
    debug_assert_eq!(metadata_kind(&metadata), expected_kind);
    debug_assert_eq!(metadata_mode(&metadata) & UNIX_PERMISSION_BITS_MASK, member.mode);
    Ok(())
}

fn validate_transport_metadata(
    member: &ChapterTransportMember,
    metadata: &Metadata,
    expected_kind: TreeEntryKind,
) -> Result<(), RunError> {
    if metadata_kind(metadata) != expected_kind {
        return Err(RunError::Internal(format!("chapter transport source type drift for {}", member.relative_path)));
    }
    if metadata_mode(metadata) & UNIX_PERMISSION_BITS_MASK != member.mode {
        return Err(RunError::Internal(format!("chapter transport source mode drift for {}", member.relative_path)));
    }
    if expected_kind == TreeEntryKind::File && metadata.len() != member.size_bytes {
        return Err(RunError::Internal(format!(
            "chapter transport source size drift for {}: expected {}, found {}",
            member.relative_path,
            member.size_bytes,
            metadata.len()
        )));
    }
    Ok(())
}

fn transport_header(member: &ChapterTransportMember, entry_type: tar::EntryType) -> Result<tar::Header, RunError> {
    let mut header = tar::Header::new_gnu();
    header.set_entry_type(entry_type);
    header.set_mode(member.mode);
    header.set_uid(0);
    header.set_gid(0);
    header.set_mtime(0);
    header.set_size(member.size_bytes);
    header
        .set_username("")
        .map_err(|error| tree_io_error("setting chapter transport header username", &member.relative_path, error))?;
    header
        .set_groupname("")
        .map_err(|error| tree_io_error("setting chapter transport header group name", &member.relative_path, error))?;
    header.set_cksum();
    debug_assert_eq!(header.size().ok(), Some(member.size_bytes));
    debug_assert_eq!(header.entry_type(), entry_type);
    Ok(header)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SENTINEL_BYTES: &[u8] = b"sentinel";
    const FILE_BYTES: &[u8] = b"tree-copy-file";
    const SMALL_ENTRY_LIMIT: u32 = 1;
    const MULTI_ENTRY_LIMIT: u32 = 4;
    const SMALL_DEPTH_LIMIT: u32 = 1;
    const EXPECTED_POSITIVE_ENTRIES_COUNT: usize = 5;

    fn write_file(path: &Path, bytes: &[u8]) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, bytes).unwrap();
    }

    // r[verify mantle.release_provenance.bundle_tree_copy.fixtures.positive]
    #[test]
    #[cfg(unix)]
    fn nested_tree_and_internal_symlinks_copy_without_traversal_and_hash_stably() {
        use std::os::unix::fs::symlink;

        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        let destination = temp.path().join("destination");
        write_file(&source.join("nested/data.txt"), FILE_BYTES);
        write_file(&source.join("nested/other.txt"), FILE_BYTES);
        symlink("nested/data.txt", source.join("latest")).unwrap();
        symlink("nested", source.join("nested-link")).unwrap();

        let prepared = prepare_tree_copy(&source).unwrap();
        assert_eq!(prepared.plan().entries.len(), EXPECTED_POSITIVE_ENTRIES_COUNT);
        copy_prepared_tree(prepared, &destination).unwrap();
        let source_hash = hash_directory_tree(&source).unwrap();
        let destination_hash = hash_directory_tree(&destination).unwrap();

        assert_eq!(source_hash, destination_hash);
        assert_eq!(std::fs::read_link(destination.join("latest")).unwrap(), Path::new("nested/data.txt"));
        assert!(std::fs::symlink_metadata(destination.join("nested-link")).unwrap().file_type().is_symlink());
        std::fs::remove_file(source.join("latest")).unwrap();
        symlink("nested/other.txt", source.join("latest")).unwrap();
        let retargeted_hash = hash_directory_tree(&source).unwrap();
        assert_ne!(retargeted_hash, source_hash, "symlink target text must affect the tree digest");
    }

    #[test]
    fn configured_hash_limit_accepts_bounded_tree_and_rejects_excess_entries() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        write_file(&source.join("one"), FILE_BYTES);
        write_file(&source.join("two"), FILE_BYTES);
        let sufficient_limits = TreeCopyLimits {
            entries_count_max: MULTI_ENTRY_LIMIT,
            ..TreeCopyLimits::RELEASE_BUNDLE
        };
        let insufficient_limits = TreeCopyLimits {
            entries_count_max: SMALL_ENTRY_LIMIT,
            ..TreeCopyLimits::RELEASE_BUNDLE
        };

        let digest = hash_directory_tree_with_limits(&source, sufficient_limits).unwrap();
        let error = hash_directory_tree_with_limits(&source, insufficient_limits).unwrap_err();

        assert_eq!(digest.1.len(), blake3::OUT_LEN.saturating_mul(2));
        assert!(error.to_string().contains("exceeds"));
        assert!(error.to_string().contains("1"));
    }

    // r[verify mantle.release_provenance.chapter_transport.pack]
    #[test]
    fn chapter_transport_emission_rejects_source_size_drift() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        write_file(&source.join("manifest.json"), FILE_BYTES);
        let prepared = prepare_tree_copy(&source).unwrap();
        let inputs = prepared.chapter_transport_entries().unwrap();
        let index =
            crunch_release_core::plan_chapter_transport(inputs, blake3::hash(FILE_BYTES).to_hex().to_string()).unwrap();
        write_file(&source.join("manifest.json"), b"changed-size");
        let mut builder = tar::Builder::new(Vec::new());

        let error = prepared.append_chapter_transport_member(&mut builder, &index.chapters[0].members[0]).unwrap_err();

        let archive_bytes = builder.into_inner().unwrap();
        assert!(error.to_string().contains("size drift"), "{error}");
        assert!(!archive_bytes.is_empty());
        assert!(archive_bytes.iter().all(|byte| *byte == 0));
    }

    // r[verify mantle.release_provenance.bundle_tree_copy.fixtures.negative.target]
    #[test]
    #[cfg(unix)]
    fn source_root_and_target_symlink_matrix_fails_before_destination_mutation() {
        use std::os::unix::fs::symlink;

        let cases = ["/absolute", "../escape", "missing"];
        for target in cases {
            let temp = tempfile::tempdir().unwrap();
            let source = temp.path().join("source");
            let destination = temp.path().join("destination");
            std::fs::create_dir_all(&source).unwrap();
            symlink(target, source.join("link")).unwrap();

            let result = copy_directory_tree(&source, &destination);

            assert!(result.is_err(), "target {target} must fail");
            assert!(!destination.exists(), "invalid source plan must not create destination");
        }

        let temp = tempfile::tempdir().unwrap();
        let real_source = temp.path().join("real-source");
        let source_link = temp.path().join("source-link");
        let destination = temp.path().join("destination");
        std::fs::create_dir(&real_source).unwrap();
        symlink(&real_source, &source_link).unwrap();
        let result = copy_directory_tree(&source_link, &destination);
        assert!(result.is_err());
        assert!(!destination.exists());
    }

    // r[verify mantle.release_provenance.bundle_tree_copy.fixtures.negative.type_drift]
    #[test]
    #[cfg(unix)]
    fn source_file_type_swap_to_symlink_fails_without_touching_external_sentinel() {
        use std::os::unix::fs::symlink;

        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        let destination = temp.path().join("destination");
        let external = temp.path().join("external.txt");
        write_file(&source.join("file.txt"), FILE_BYTES);
        write_file(&external, SENTINEL_BYTES);
        let prepared = prepare_tree_copy(&source).unwrap();
        std::fs::remove_file(source.join("file.txt")).unwrap();
        symlink(&external, source.join("file.txt")).unwrap();

        let result = copy_prepared_tree(prepared, &destination);

        assert!(result.unwrap_err().to_string().contains("source type drift"));
        assert_eq!(std::fs::read(&external).unwrap(), SENTINEL_BYTES);
        assert!(!destination.join("file.txt").exists());
    }

    // r[verify mantle.release_provenance.bundle_tree_copy.fixtures.negative.type_drift]
    #[test]
    #[cfg(unix)]
    fn destination_parent_symlink_fails_without_external_write() {
        use std::os::unix::fs::symlink;

        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        let destination = temp.path().join("destination");
        let external = temp.path().join("external");
        write_file(&source.join("nested/file.txt"), FILE_BYTES);
        write_file(&external.join("file.txt"), SENTINEL_BYTES);
        let prepared = prepare_tree_copy(&source).unwrap();
        let destination_root = prepare_empty_destination_root(&destination).unwrap();
        symlink(&external, destination.join("nested")).unwrap();

        let result = execute_tree_copy_plan(&prepared, &destination_root);

        assert!(result.unwrap_err().to_string().contains("must be empty"));
        assert_eq!(std::fs::read(external.join("file.txt")).unwrap(), SENTINEL_BYTES);
    }

    // r[verify mantle.release_provenance.bundle_tree_copy.fixtures.negative.symlink_escape]
    #[test]
    #[cfg(unix)]
    fn destination_symlink_root_and_preexisting_symlink_fail_closed() {
        use std::os::unix::fs::symlink;

        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        let external = temp.path().join("external");
        let destination_link = temp.path().join("destination-link");
        write_file(&source.join("file.txt"), FILE_BYTES);
        write_file(&external.join("sentinel.txt"), SENTINEL_BYTES);
        symlink(&external, &destination_link).unwrap();

        let root_result = copy_directory_tree(&source, &destination_link);
        assert!(root_result.is_err());
        assert_eq!(std::fs::read(external.join("sentinel.txt")).unwrap(), SENTINEL_BYTES);

        let destination = temp.path().join("destination");
        std::fs::create_dir(&destination).unwrap();
        symlink(&external, destination.join("injected")).unwrap();
        let child_result = copy_directory_tree(&source, &destination);
        assert!(child_result.unwrap_err().to_string().contains("must be empty"));
        assert_eq!(std::fs::read(external.join("sentinel.txt")).unwrap(), SENTINEL_BYTES);
    }

    // r[verify mantle.release_provenance.bundle_tree_copy.plan.invalid]
    #[test]
    #[cfg(unix)]
    fn special_file_and_production_bound_exhaustion_fail_before_copy() {
        use std::os::unix::net::UnixListener;

        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        let destination = temp.path().join("destination");
        std::fs::create_dir(&source).unwrap();
        let _listener = UnixListener::bind(source.join("socket")).unwrap();
        let special_result = copy_directory_tree(&source, &destination);
        assert!(special_result.unwrap_err().to_string().contains("unsupported"));
        assert!(!destination.exists());

        let bounded_source = temp.path().join("bounded-source");
        write_file(&bounded_source.join("nested/second"), FILE_BYTES);
        let entry_limits = TreeCopyLimits {
            entries_count_max: SMALL_ENTRY_LIMIT,
            ..TreeCopyLimits::RELEASE_BUNDLE
        };
        let entry_result = prepare_tree_copy_with_limits(&bounded_source, entry_limits);
        let entry_error = entry_result.unwrap_err();
        assert!(entry_error.to_string().contains("observation count exceeds"), "unexpected error: {entry_error}");

        let depth_source = temp.path().join("depth-source");
        write_file(&depth_source.join("nested/file"), FILE_BYTES);
        let depth_limits = TreeCopyLimits {
            depth_count_max: SMALL_DEPTH_LIMIT,
            ..TreeCopyLimits::RELEASE_BUNDLE
        };
        let depth_result = prepare_tree_copy_with_limits(&depth_source, depth_limits);
        assert!(depth_result.unwrap_err().to_string().contains("path depth"));
    }
}
