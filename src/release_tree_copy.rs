use std::ffi::OsString;
use std::io::Read;
use std::io::Write;
use std::path::Path;

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
use crunch_release_core::TreeCopyBlocker;
use crunch_release_core::TreeCopyLimits;
use crunch_release_core::TreeCopyOperation;
use crunch_release_core::TreeCopyPlan;
use crunch_release_core::TreeEntryKind;
use crunch_release_core::TreeEntryObservation;
use crunch_release_core::plan_tree_copy;

use crate::errors::RunError;
use crate::release_capability::ReleaseCapabilityRoot;
use crate::release_capability::ReleaseRootKind;
use crate::release_capability::ValidatedReleasePath;

const TREE_COPY_BUFFER_BYTES: usize = 8_192;
const TREE_COPY_DIAGNOSTIC_BLOCKERS_MAX: usize = 16;
const EMPTY_DIRECTORY_ACCOUNTED_SIZE_BYTES: u64 = 1;
#[cfg(unix)]
const UNIX_PERMISSION_BITS_MASK: u32 = 0o7_777;
#[cfg(unix)]
const STAGING_DIRECTORY_MODE: u32 = 0o700;
#[cfg(unix)]
const STAGING_FILE_MODE: u32 = 0o600;

#[derive(Debug)]
pub(crate) struct PreparedTreeCopy {
    source_root: ReleaseCapabilityRoot,
    plan: TreeCopyPlan,
}

#[derive(Debug)]
struct PendingDirectory {
    relative_path: String,
    dir: Dir,
    depth_count: u32,
}

impl PreparedTreeCopy {
    pub(crate) fn artifact_identity(&self) -> Result<(u64, String), RunError> {
        hash_prepared_tree(self)
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
    let observations = observe_tree_entries(&source_root, limits)?;
    let plan = plan_tree_copy(observations, limits).map_err(tree_plan_error)?;
    let entries_count_max = usize::try_from(limits.entries_count_max)
        .map_err(|_| RunError::Internal("tree copy entry limit does not fit usize".to_string()))?;
    assert_eq!(source_root.kind(), ReleaseRootKind::ReleaseTreeSource);
    assert!(plan.entries.len() <= entries_count_max);
    Ok(PreparedTreeCopy { source_root, plan })
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
    let prepared = prepare_tree_copy(path)
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

fn observe_tree_entries(
    source_root: &ReleaseCapabilityRoot,
    limits: TreeCopyLimits,
) -> Result<Vec<TreeEntryObservation>, RunError> {
    let mut observations = Vec::new();
    let mut pending = vec![PendingDirectory {
        relative_path: String::new(),
        dir: source_root
            .dir()
            .try_clone()
            .map_err(|error| RunError::Internal(format!("cloning source root capability: {error}")))?,
        depth_count: 0,
    }];
    while let Some(directory) = pending.pop() {
        observe_directory_children(directory, limits, &mut observations, &mut pending)?;
    }
    Ok(observations)
}

fn observe_directory_children(
    directory: PendingDirectory,
    limits: TreeCopyLimits,
    observations: &mut Vec<TreeEntryObservation>,
    pending: &mut Vec<PendingDirectory>,
) -> Result<(), RunError> {
    let mut children = collect_sorted_child_names(&directory.dir, &directory.relative_path, limits.entries_count_max)?;
    debug_assert!(observations.len() <= observations.capacity());
    debug_assert!(pending.len() <= pending.capacity());
    children.reverse();
    for child_name in children {
        ensure_observation_capacity(observations.len(), limits.entries_count_max)?;
        let child_path = join_relative_path(RelativePathParts {
            parent: &directory.relative_path,
            child: &child_name,
        })?;
        let metadata = directory
            .dir
            .symlink_metadata(&child_name)
            .map_err(|error| tree_io_error("reading no-follow source metadata", &child_path, error))?;
        let kind = metadata_kind(&metadata);
        let target = read_observed_symlink_target(&directory.dir, SymlinkObservationRequest {
            child_name: &child_name,
            child_path: &child_path,
            kind,
        })?;
        observations.push(TreeEntryObservation {
            relative_path: child_path.clone(),
            kind,
            mode: metadata_mode(&metadata),
            symlink_target: target,
        });
        enqueue_child_directory(
            EnqueueDirectoryRequest {
                parent: &directory,
                child_name,
                child_path,
                kind,
                limits,
            },
            pending,
        )?;
    }
    Ok(())
}

fn collect_sorted_child_names(dir: &Dir, relative_path: &str, entries_count_max: u32) -> Result<Vec<String>, RunError> {
    let entries_count_max_items = usize::try_from(entries_count_max)
        .map_err(|_| RunError::Internal("tree copy child limit does not fit usize".to_string()))?;
    let entries = dir
        .entries()
        .map_err(|error| tree_io_error("reading source directory", relative_path, error))?
        .take(entries_count_max_items.saturating_add(1))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| tree_io_error("reading source directory entry", relative_path, error))?;
    if entries.len() > entries_count_max_items {
        return Err(RunError::Internal(format!(
            "tree copy directory exceeds {entries_count_max} children under {relative_path}"
        )));
    }
    let mut names = Vec::with_capacity(entries.len());
    for entry in entries {
        let name = entry.file_name().into_string().map_err(|_| {
            RunError::Internal(format!("tree copy source entry is not valid UTF-8 under {relative_path}"))
        })?;
        names.push(name);
    }
    names.sort();
    debug_assert!(names.len() <= entries_count_max_items);
    debug_assert!(names.capacity() >= names.len());
    Ok(names)
}

fn ensure_observation_capacity(entries_count: usize, entries_count_max: u32) -> Result<(), RunError> {
    let count = u32::try_from(entries_count)
        .map_err(|_| RunError::Internal("tree copy observation count overflowed u32".to_string()))?;
    if count >= entries_count_max {
        return Err(RunError::Internal(format!(
            "tree copy observation count exceeds {entries_count_max} before mutation"
        )));
    }
    Ok(())
}

struct RelativePathParts<'a> {
    parent: &'a str,
    child: &'a str,
}

fn join_relative_path(parts: RelativePathParts<'_>) -> Result<String, RunError> {
    if parts.child.is_empty() || parts.child.contains('/') || parts.child.contains('\\') {
        return Err(RunError::Internal(format!("invalid tree copy child name under {}", parts.parent)));
    }
    if parts.parent.is_empty() {
        return Ok(parts.child.to_string());
    }
    Ok(format!("{}/{}", parts.parent, parts.child))
}

struct SymlinkObservationRequest<'a> {
    child_name: &'a str,
    child_path: &'a str,
    kind: TreeEntryKind,
}

fn read_observed_symlink_target(dir: &Dir, request: SymlinkObservationRequest<'_>) -> Result<Option<String>, RunError> {
    if request.kind != TreeEntryKind::Symlink {
        return Ok(None);
    }
    let target = dir
        .read_link_contents(request.child_name)
        .map_err(|error| tree_io_error("reading source symlink target", request.child_path, error))?;
    let target = target.into_os_string().into_string().map_err(|_| {
        RunError::Internal(format!("tree copy symlink target is not valid UTF-8: {}", request.child_path))
    })?;
    Ok(Some(target))
}

struct EnqueueDirectoryRequest<'a> {
    parent: &'a PendingDirectory,
    child_name: String,
    child_path: String,
    kind: TreeEntryKind,
    limits: TreeCopyLimits,
}

fn enqueue_child_directory(
    request: EnqueueDirectoryRequest<'_>,
    pending: &mut Vec<PendingDirectory>,
) -> Result<(), RunError> {
    if request.kind != TreeEntryKind::Directory {
        return Ok(());
    }
    let depth_count = request
        .parent
        .depth_count
        .checked_add(1)
        .ok_or_else(|| RunError::Internal("tree copy discovery depth overflowed u32".to_string()))?;
    if depth_count > request.limits.depth_count_max {
        return Ok(());
    }
    let dir = request
        .parent
        .dir
        .open_dir_nofollow(&request.child_name)
        .map_err(|error| tree_io_error("opening no-follow source directory", &request.child_path, error))?;
    pending.push(PendingDirectory {
        relative_path: request.child_path,
        dir,
        depth_count,
    });
    debug_assert!(depth_count > request.parent.depth_count);
    debug_assert!(depth_count <= request.limits.depth_count_max);
    Ok(())
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
    for operation in &prepared.plan.operations {
        execute_tree_copy_operation(prepared, destination_root, operation)?;
    }
    finalize_directory_modes(prepared, destination_root)?;
    Ok(())
}

fn execute_tree_copy_operation(
    prepared: &PreparedTreeCopy,
    destination_root: &ReleaseCapabilityRoot,
    operation: &TreeCopyOperation,
) -> Result<(), RunError> {
    match operation {
        TreeCopyOperation::CreateDirectory { relative_path, mode } => {
            revalidate_source_path(&prepared.source_root, relative_path, TreeEntryKind::Directory, *mode, None)?;
            create_destination_directory(destination_root, relative_path)
        }
        TreeCopyOperation::CopyFile { relative_path, mode } => {
            copy_regular_file(&prepared.source_root, destination_root, relative_path, *mode)
        }
        TreeCopyOperation::CreateSymlink {
            relative_path,
            mode,
            target,
        } => {
            revalidate_source_path(&prepared.source_root, relative_path, TreeEntryKind::Symlink, *mode, Some(target))?;
            create_destination_symlink(DestinationSymlink {
                destination_root,
                relative_path,
                target,
            })
        }
    }
}

fn revalidate_source_path(
    source_root: &ReleaseCapabilityRoot,
    relative_path: &str,
    expected_kind: TreeEntryKind,
    expected_mode: u32,
    expected_target: Option<&String>,
) -> Result<(), RunError> {
    let (parent, name) = open_parent_directory_nofollow(source_root.dir(), relative_path, TreePathRole::Source)?;
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

fn create_destination_directory(destination_root: &ReleaseCapabilityRoot, relative_path: &str) -> Result<(), RunError> {
    let (parent, name) =
        open_parent_directory_nofollow(destination_root.dir(), relative_path, TreePathRole::Destination)?;
    require_destination_entry_absent(&parent, &name, relative_path)?;
    parent
        .create_dir(&name)
        .map_err(|error| tree_io_error("creating destination directory", relative_path, error))?;
    let created = parent
        .open_dir_nofollow(&name)
        .map_err(|error| tree_io_error("opening created destination directory", relative_path, error))?;
    set_directory_mode(&created, staging_directory_mode(), relative_path)
}

fn copy_regular_file(
    source_root: &ReleaseCapabilityRoot,
    destination_root: &ReleaseCapabilityRoot,
    relative_path: &str,
    expected_mode: u32,
) -> Result<(), RunError> {
    let (source_parent, source_name) =
        open_parent_directory_nofollow(source_root.dir(), relative_path, TreePathRole::Source)?;
    let observed_metadata = source_parent
        .symlink_metadata(&source_name)
        .map_err(|error| tree_io_error("revalidating source file", relative_path, error))?;
    validate_source_metadata(relative_path, &observed_metadata, TreeEntryKind::File, expected_mode)?;
    let mut source = open_file_nofollow(&source_parent, &source_name, false, 0).map_err(|error| {
        RunError::Internal(format!("tree copy source type drift while opening {relative_path}: {error}"))
    })?;
    let source_metadata = source
        .metadata()
        .map_err(|error| tree_io_error("reading source file metadata", relative_path, error))?;
    validate_source_metadata(relative_path, &source_metadata, TreeEntryKind::File, expected_mode)?;

    let (destination_parent, destination_name) =
        open_parent_directory_nofollow(destination_root.dir(), relative_path, TreePathRole::Destination)?;
    require_destination_entry_absent(&destination_parent, &destination_name, relative_path)?;
    let mut destination = open_file_nofollow(&destination_parent, &destination_name, true, expected_mode)
        .map_err(|error| tree_io_error("creating no-follow destination file", relative_path, error))?;
    copy_open_file_bytes(&mut source, &mut destination, relative_path, source_metadata.len())?;
    set_file_mode(&destination, expected_mode, relative_path)?;
    debug_assert!(source_metadata.is_file());
    debug_assert_eq!(metadata_mode(&source_metadata), expected_mode);
    Ok(())
}

struct DestinationSymlink<'a> {
    destination_root: &'a ReleaseCapabilityRoot,
    relative_path: &'a str,
    target: &'a str,
}

fn create_destination_symlink(request: DestinationSymlink<'_>) -> Result<(), RunError> {
    let (parent, name) = open_parent_directory_nofollow(
        request.destination_root.dir(),
        request.relative_path,
        TreePathRole::Destination,
    )?;
    require_destination_entry_absent(&parent, &name, request.relative_path)?;
    DirExt::symlink(&parent, request.target, &name)
        .map_err(|error| tree_io_error("creating destination symlink", request.relative_path, error))
}

fn require_destination_entry_absent(parent: &Dir, name: &OsString, relative_path: &str) -> Result<(), RunError> {
    match parent.symlink_metadata(name) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(tree_io_error("checking destination entry", relative_path, error)),
        Ok(metadata) => Err(RunError::Internal(format!(
            "tree copy destination type drift for {relative_path}: unexpected {:?} entry already exists",
            metadata_kind(&metadata)
        ))),
    }
}

fn finalize_directory_modes(
    prepared: &PreparedTreeCopy,
    destination_root: &ReleaseCapabilityRoot,
) -> Result<(), RunError> {
    let mut directories = prepared
        .plan
        .entries
        .iter()
        .filter(|entry| entry.kind == TreeEntryKind::Directory)
        .collect::<Vec<_>>();
    directories.sort_by(|left, right| {
        relative_depth_count(&right.relative_path)
            .cmp(&relative_depth_count(&left.relative_path))
            .then_with(|| right.relative_path.cmp(&left.relative_path))
    });
    debug_assert!(directories.len() <= prepared.plan.entries.len());
    debug_assert!(directories.iter().all(|entry| entry.kind == TreeEntryKind::Directory));
    for entry in directories {
        revalidate_source_path(
            &prepared.source_root,
            &entry.relative_path,
            TreeEntryKind::Directory,
            entry.mode,
            None,
        )?;
        let dir =
            open_directory_relative_nofollow(destination_root.dir(), &entry.relative_path, TreePathRole::Destination)?;
        set_directory_mode(&dir, entry.mode, &entry.relative_path)?;
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum TreePathRole {
    Source,
    Destination,
}

impl TreePathRole {
    fn as_str(self) -> &'static str {
        match self {
            Self::Source => "source",
            Self::Destination => "destination",
        }
    }
}

fn open_parent_directory_nofollow(
    root: &Dir,
    relative_path: &str,
    role: TreePathRole,
) -> Result<(Dir, OsString), RunError> {
    let label = role.as_str();
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

fn open_directory_relative_nofollow(root: &Dir, relative_path: &str, role: TreePathRole) -> Result<Dir, RunError> {
    let label = role.as_str();
    let (parent, name) = open_parent_directory_nofollow(root, relative_path, role)?;
    parent
        .open_dir_nofollow(name)
        .map_err(|error| tree_io_error(format!("opening no-follow {label} directory"), relative_path, error))
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
    let (parent, name) =
        open_parent_directory_nofollow(prepared.source_root.dir(), &entry.relative_path, TreePathRole::Source)?;
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

fn relative_depth_count(path: &str) -> u32 {
    path.bytes().fold(1_u32, |depth_count, byte| {
        if byte == b'/' {
            depth_count.saturating_add(1)
        } else {
            depth_count
        }
    })
}

#[cfg(unix)]
fn set_open_file_mode(options: &mut OpenOptions, _mode: u32) {
    options.mode(STAGING_FILE_MODE);
}

#[cfg(not(unix))]
fn set_open_file_mode(_options: &mut OpenOptions, _mode: u32) {}

#[cfg(unix)]
const fn staging_directory_mode() -> u32 {
    STAGING_DIRECTORY_MODE
}

#[cfg(not(unix))]
const fn staging_directory_mode() -> u32 {
    0
}

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

#[cfg(unix)]
fn set_directory_mode(dir: &Dir, mode: u32, relative_path: &str) -> Result<(), RunError> {
    let permissions = cap_std::fs::Permissions::from_mode(mode & UNIX_PERMISSION_BITS_MASK);
    dir.set_permissions(".", permissions)
        .map_err(|error| tree_io_error("setting destination directory mode", relative_path, error))
}

#[cfg(not(unix))]
fn set_directory_mode(_dir: &Dir, _mode: u32, _relative_path: &str) -> Result<(), RunError> {
    Ok(())
}

fn tree_io_error(action: impl AsRef<str>, relative_path: &str, error: std::io::Error) -> RunError {
    RunError::Internal(format!("{} {relative_path}: {error}", action.as_ref()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SENTINEL_BYTES: &[u8] = b"sentinel";
    const FILE_BYTES: &[u8] = b"tree-copy-file";
    const SMALL_ENTRY_LIMIT: u32 = 1;
    const SMALL_DEPTH_LIMIT: u32 = 1;
    const EXPECTED_POSITIVE_ENTRIES_COUNT: usize = 5;
    const DIRECTORY_OPERATION_INDEX: usize = 0;
    const FILE_OPERATION_INDEX: usize = 1;

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
    fn destination_parent_swap_to_symlink_fails_without_external_write() {
        use std::os::unix::fs::symlink;

        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        let destination = temp.path().join("destination");
        let external = temp.path().join("external");
        write_file(&source.join("nested/file.txt"), FILE_BYTES);
        write_file(&external.join("file.txt"), SENTINEL_BYTES);
        let prepared = prepare_tree_copy(&source).unwrap();
        let destination_root = prepare_empty_destination_root(&destination).unwrap();
        execute_tree_copy_operation(
            &prepared,
            &destination_root,
            &prepared.plan().operations[DIRECTORY_OPERATION_INDEX],
        )
        .unwrap();
        std::fs::remove_dir(destination.join("nested")).unwrap();
        symlink(&external, destination.join("nested")).unwrap();

        let result = execute_tree_copy_operation(
            &prepared,
            &destination_root,
            &prepared.plan().operations[FILE_OPERATION_INDEX],
        );

        assert!(result.unwrap_err().to_string().contains("no-follow destination parent"));
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
