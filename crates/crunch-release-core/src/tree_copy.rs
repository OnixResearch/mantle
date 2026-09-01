use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

pub const RELEASE_TREE_COPY_MAX_ENTRIES_COUNT: u32 = 4_096;
pub const RELEASE_TREE_COPY_MAX_DEPTH_COUNT: u32 = 128;
pub const RELEASE_TREE_COPY_MAX_PATH_BYTES: u32 = 4_096;

const WINDOWS_DRIVE_PREFIX_BYTES: usize = 2;
const DIRECTORY_OPERATION_PHASE: u8 = 0;
const FILE_OPERATION_PHASE: u8 = 1;
const SYMLINK_OPERATION_PHASE: u8 = 2;
const FILE_BYTES_COMPATIBILITY_LIMIT: u64 = u64::MAX;
const TOTAL_BYTES_COMPATIBILITY_LIMIT: u64 = u64::MAX;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeCopyLimits {
    pub entries_count_max: u32,
    pub depth_count_max: u32,
    pub path_bytes_max: u32,
}

impl TreeCopyLimits {
    pub const RELEASE_BUNDLE: Self = Self {
        entries_count_max: RELEASE_TREE_COPY_MAX_ENTRIES_COUNT,
        depth_count_max: RELEASE_TREE_COPY_MAX_DEPTH_COUNT,
        path_bytes_max: RELEASE_TREE_COPY_MAX_PATH_BYTES,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TreeEntryKind {
    Directory,
    File,
    Symlink,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeEntryObservation {
    pub relative_path: String,
    pub kind: TreeEntryKind,
    pub mode: u32,
    pub symlink_target: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeCopyOperation {
    CreateDirectory {
        relative_path: String,
        mode: u32,
    },
    CopyFile {
        relative_path: String,
        mode: u32,
    },
    CreateSymlink {
        relative_path: String,
        mode: u32,
        target: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeCopyPlan {
    pub entries: Vec<TreeEntryObservation>,
    pub operations: Vec<TreeCopyOperation>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TreeCopyBlockerKind {
    InvalidEntryLimit,
    InvalidDepthLimit,
    InvalidPathLimit,
    EntryCountOverflow,
    EntryLimitExceeded,
    EmptyPath,
    AbsolutePath,
    WindowsPrefix,
    BackslashSeparator,
    EmptyComponent,
    CurrentDirectoryComponent,
    ParentDirectoryComponent,
    InvalidCharacter,
    PathTooLong,
    PathTooDeep,
    DuplicatePath,
    MissingParent,
    ParentNotDirectory,
    UnsupportedKind,
    MissingSymlinkTarget,
    UnexpectedSymlinkTarget,
    AbsoluteSymlinkTarget,
    SymlinkTargetWindowsPrefix,
    SymlinkTargetBackslash,
    SymlinkTargetInvalidCharacter,
    SymlinkTargetTooLong,
    SymlinkTargetTooDeep,
    SymlinkTargetEscapes,
    SymlinkTargetSelf,
    SymlinkTargetMissing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeCopyBlocker {
    pub kind: TreeCopyBlockerKind,
    pub relative_path: Option<String>,
    pub message: String,
}

impl fmt::Display for TreeCopyBlocker {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

/// Map Mantle's retained release limits to the complete shared limit profile.
///
/// # Errors
///
/// Returns Mantle-compatible blockers when a retained limit is zero.
pub fn bounded_tree_limits(limits: TreeCopyLimits) -> Result<bounded_tree_core::TreeLimits, Vec<TreeCopyBlocker>> {
    let mut blockers = Vec::new();
    if limits.entries_count_max == 0 {
        blockers.push(blocker(TreeCopyBlockerKind::InvalidEntryLimit, None, "tree copy entry limit must be positive"));
    }
    if limits.depth_count_max == 0 {
        blockers.push(blocker(TreeCopyBlockerKind::InvalidDepthLimit, None, "tree copy depth limit must be positive"));
    }
    if limits.path_bytes_max == 0 {
        blockers.push(blocker(TreeCopyBlockerKind::InvalidPathLimit, None, "tree copy path limit must be positive"));
    }
    if !blockers.is_empty() {
        return Err(blockers);
    }
    debug_assert!(limits.entries_count_max > 0);
    debug_assert!(limits.path_bytes_max > 0);
    let values = bounded_tree_core::LimitValues {
        entries: u64::from(limits.entries_count_max),
        depth: u64::from(limits.depth_count_max),
        path_bytes: u64::from(limits.path_bytes_max),
        file_bytes: FILE_BYTES_COMPATIBILITY_LIMIT,
        total_bytes: TOTAL_BYTES_COMPATIBILITY_LIMIT,
        symlink_target_bytes: u64::from(limits.path_bytes_max),
    };
    bounded_tree_core::TreeLimits::new(values).map_err(|error| {
        vec![blocker(
            limit_error_kind(error.kind()),
            None,
            "shared tree limit admission rejected a Mantle compatibility limit",
        )]
    })
}

/// Convert a shared plan back into Mantle's retained release-plan DTO.
///
/// # Errors
///
/// Returns a blocker if shared exact bytes cannot be represented by the
/// retained UTF-8 compatibility surface.
pub fn bounded_tree_plan(plan: &bounded_tree_core::TreePlan) -> Result<TreeCopyPlan, Vec<TreeCopyBlocker>> {
    let mut entries = Vec::with_capacity(plan.member_facts().len());
    for fact in plan.member_facts() {
        let relative_path = relative_path_text(fact.path().components())?;
        let symlink_target =
            fact.symlink_target().map(|target| String::from_utf8(target.to_vec())).transpose().map_err(|_| {
                vec![path_blocker(PathBlockerInput {
                    kind: TreeCopyBlockerKind::SymlinkTargetInvalidCharacter,
                    path: &relative_path,
                    message: "tree copy symlink target is not valid UTF-8",
                })]
            })?;
        entries.push(TreeEntryObservation {
            relative_path,
            kind: from_shared_kind(fact.kind()),
            mode: fact.mode().bits(),
            symlink_target,
        });
    }

    entries.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));

    let mut operations = Vec::with_capacity(plan.operations().len());
    for operation in plan.operations() {
        let mode = plan
            .member_facts()
            .iter()
            .find(|fact| fact.path() == operation.path())
            .map(|fact| fact.mode().bits())
            .ok_or_else(|| {
                vec![blocker(
                    TreeCopyBlockerKind::MissingParent,
                    None,
                    "shared tree operation has no matching member fact",
                )]
            })?;
        operations.push(from_shared_operation(operation, mode)?);
    }
    operations
        .sort_by(|left, right| compatibility_operation_sort_key(left).cmp(&compatibility_operation_sort_key(right)));
    debug_assert_eq!(entries.len(), operations.len());
    Ok(TreeCopyPlan { entries, operations })
}

// r[impl mantle.release_provenance.bundle_tree_copy.plan]
// r[impl mantle.release_provenance.bundle_tree_copy.plan.invalid]
// r[impl mantle.release_provenance.bundle_tree_copy.symlink_policy]
// r[impl mantle.bounded_tree_adoption.release_copy]
pub fn plan_tree_copy(
    observations: Vec<TreeEntryObservation>,
    limits: TreeCopyLimits,
) -> Result<TreeCopyPlan, Vec<TreeCopyBlocker>> {
    let admitted_bounds = bounded_tree_limits(limits)?;
    let mut blockers = Vec::with_capacity(observations.len());
    let mut shared_observations = Vec::with_capacity(observations.len());
    for observation in observations {
        match to_shared_observation(observation) {
            Ok(observation) => shared_observations.push(observation),
            Err(blocker) => blockers.push(blocker),
        }
    }
    blockers.sort_by(blocker_order);
    if !blockers.is_empty() {
        return Err(blockers);
    }
    let request = bounded_tree_core::TreeRequest {
        observations: shared_observations,
        limits: admitted_bounds,
        symlink_policy: bounded_tree_core::SymlinkPolicy::PreserveInternal,
    };
    let plan = bounded_tree_core::plan_tree(request).map_err(|error| bounded_tree_blockers(&error))?;
    let compatibility_plan = bounded_tree_plan(&plan)?;
    debug_assert_eq!(compatibility_plan.entries.len(), compatibility_plan.operations.len());
    debug_assert!(
        u32::try_from(compatibility_plan.entries.len())
            .is_ok_and(|entry_count| entry_count <= RELEASE_TREE_COPY_MAX_ENTRIES_COUNT)
    );
    Ok(compatibility_plan)
}

/// Map shared admission blockers to Mantle's retained diagnostic surface.
#[must_use]
pub fn bounded_tree_blockers(error: &bounded_tree_core::PlanError) -> Vec<TreeCopyBlocker> {
    let mut blockers = error.blockers().iter().map(shared_blocker).collect::<Vec<_>>();
    blockers.sort_by(blocker_order);
    blockers
}

fn to_shared_observation(
    observation: TreeEntryObservation,
) -> Result<bounded_tree_core::RawObservation, TreeCopyBlocker> {
    let path_components = path_components(&observation.relative_path)?;
    validate_legacy_shape(&observation)?;
    let file_content = if observation.kind == TreeEntryKind::File {
        Some(bounded_tree_core::Blake3Digest::from_bytes(*blake3::hash(&[]).as_bytes()))
    } else {
        None
    };
    Ok(bounded_tree_core::RawObservation {
        path_components,
        kind: to_shared_kind(observation.kind),
        mode: bounded_tree_core::FileMode::new(observation.mode),
        file_bytes: 0,
        symlink_target: observation.symlink_target.map(String::into_bytes),
        file_content,
    })
}

fn path_components(path: &str) -> Result<Vec<Vec<u8>>, TreeCopyBlocker> {
    if path.is_empty() {
        return Err(path_blocker(PathBlockerInput {
            kind: TreeCopyBlockerKind::EmptyPath,
            path,
            message: "tree copy path must not be empty",
        }));
    }
    if path.starts_with('/') {
        return Err(path_blocker(PathBlockerInput {
            kind: TreeCopyBlockerKind::AbsolutePath,
            path,
            message: "tree copy path must be relative",
        }));
    }
    if looks_like_windows_prefix(path) {
        return Err(path_blocker(PathBlockerInput {
            kind: TreeCopyBlockerKind::WindowsPrefix,
            path,
            message: "tree copy path must not use a drive prefix",
        }));
    }
    if path.contains('\\') {
        return Err(path_blocker(PathBlockerInput {
            kind: TreeCopyBlockerKind::BackslashSeparator,
            path,
            message: "tree copy path must use '/' separators",
        }));
    }
    let components = path.split('/').map(|component| component.as_bytes().to_vec()).collect::<Vec<_>>();
    debug_assert!(!components.is_empty());
    debug_assert!(components.iter().all(|component| !component.is_empty()));
    Ok(components)
}

fn validate_legacy_shape(observation: &TreeEntryObservation) -> Result<(), TreeCopyBlocker> {
    if observation.kind == TreeEntryKind::Symlink {
        let target = observation.symlink_target.as_deref().ok_or_else(|| {
            path_blocker(PathBlockerInput {
                kind: TreeCopyBlockerKind::MissingSymlinkTarget,
                path: &observation.relative_path,
                message: "tree copy symlink must record a target",
            })
        })?;
        validate_legacy_target(LegacyTarget {
            relative_path: &observation.relative_path,
            target,
        })?;
    } else if observation.symlink_target.is_some() {
        return Err(path_blocker(PathBlockerInput {
            kind: TreeCopyBlockerKind::UnexpectedSymlinkTarget,
            path: &observation.relative_path,
            message: "non-symlink tree copy entry must not record a symlink target",
        }));
    }
    debug_assert_eq!(observation.kind == TreeEntryKind::Symlink, observation.symlink_target.is_some());
    debug_assert!(!observation.relative_path.is_empty());
    Ok(())
}

struct LegacyTarget<'a> {
    relative_path: &'a str,
    target: &'a str,
}

fn validate_legacy_target(input: LegacyTarget<'_>) -> Result<(), TreeCopyBlocker> {
    if input.target.is_empty() || input.target.starts_with('/') {
        return Err(path_blocker(PathBlockerInput {
            kind: TreeCopyBlockerKind::AbsoluteSymlinkTarget,
            path: input.relative_path,
            message: "tree copy symlink target must be non-empty and relative",
        }));
    }
    if looks_like_windows_prefix(input.target) {
        return Err(path_blocker(PathBlockerInput {
            kind: TreeCopyBlockerKind::SymlinkTargetWindowsPrefix,
            path: input.relative_path,
            message: "tree copy symlink target must not use a drive prefix",
        }));
    }
    if input.target.contains('\\') {
        return Err(path_blocker(PathBlockerInput {
            kind: TreeCopyBlockerKind::SymlinkTargetBackslash,
            path: input.relative_path,
            message: "tree copy symlink target must use '/' separators",
        }));
    }
    debug_assert!(!input.target.is_empty());
    debug_assert!(!input.target.starts_with('/'));
    Ok(())
}

fn from_shared_operation(
    operation: &bounded_tree_core::CopyOperation,
    compatibility_mode: u32,
) -> Result<TreeCopyOperation, Vec<TreeCopyBlocker>> {
    match operation {
        bounded_tree_core::CopyOperation::CreateDirectory { path, mode } => Ok(TreeCopyOperation::CreateDirectory {
            relative_path: relative_path_text(path.components())?,
            mode: mode.bits(),
        }),
        bounded_tree_core::CopyOperation::CopyFile { path, mode, .. } => Ok(TreeCopyOperation::CopyFile {
            relative_path: relative_path_text(path.components())?,
            mode: mode.bits(),
        }),
        bounded_tree_core::CopyOperation::CreateSymlink { path, target, .. } => {
            let relative_path = relative_path_text(path.components())?;
            let target = String::from_utf8(target.clone()).map_err(|_| {
                vec![path_blocker(PathBlockerInput {
                    kind: TreeCopyBlockerKind::SymlinkTargetInvalidCharacter,
                    path: &relative_path,
                    message: "tree copy symlink target is not valid UTF-8",
                })]
            })?;
            Ok(TreeCopyOperation::CreateSymlink {
                relative_path,
                mode: compatibility_mode,
                target,
            })
        }
    }
}

fn relative_path_text(components: &[Vec<u8>]) -> Result<String, Vec<TreeCopyBlocker>> {
    let mut parts = Vec::with_capacity(components.len());
    for component in components {
        parts.push(String::from_utf8(component.clone()).map_err(|_| {
            vec![blocker(
                TreeCopyBlockerKind::InvalidCharacter,
                None,
                "shared tree path is not valid UTF-8",
            )]
        })?);
    }
    Ok(parts.join("/"))
}

const fn to_shared_kind(kind: TreeEntryKind) -> bounded_tree_core::EntryKind {
    match kind {
        TreeEntryKind::Directory => bounded_tree_core::EntryKind::Directory,
        TreeEntryKind::File => bounded_tree_core::EntryKind::File,
        TreeEntryKind::Symlink => bounded_tree_core::EntryKind::Symlink,
        TreeEntryKind::Unsupported => bounded_tree_core::EntryKind::Unsupported,
    }
}

const fn from_shared_kind(kind: bounded_tree_core::EntryKind) -> TreeEntryKind {
    match kind {
        bounded_tree_core::EntryKind::Directory => TreeEntryKind::Directory,
        bounded_tree_core::EntryKind::File => TreeEntryKind::File,
        bounded_tree_core::EntryKind::Symlink => TreeEntryKind::Symlink,
        bounded_tree_core::EntryKind::Unsupported => TreeEntryKind::Unsupported,
    }
}

fn shared_blocker(blocker: &bounded_tree_core::Blocker) -> TreeCopyBlocker {
    let kind = shared_blocker_kind(blocker.kind());
    let relative_path = blocker.path_components().and_then(|components| relative_path_text(components).ok());
    TreeCopyBlocker {
        kind,
        message: shared_blocker_message(kind).to_string(),
        relative_path,
    }
}

const fn shared_blocker_message(kind: TreeCopyBlockerKind) -> &'static str {
    match kind {
        TreeCopyBlockerKind::SymlinkTargetEscapes => "tree copy symlink target escapes source root",
        TreeCopyBlockerKind::SymlinkTargetSelf => "tree copy symlink target resolves to the link itself",
        TreeCopyBlockerKind::SymlinkTargetMissing => "tree copy symlink target does not name a planned entry",
        TreeCopyBlockerKind::AbsoluteSymlinkTarget => "tree copy symlink target must be non-empty and relative",
        TreeCopyBlockerKind::UnsupportedKind => "tree copy entry kind is unsupported",
        _ => "shared bounded-tree admission rejected the tree entry",
    }
}

fn shared_blocker_kind(kind: bounded_tree_core::BlockerKind) -> TreeCopyBlockerKind {
    match kind {
        bounded_tree_core::BlockerKind::EntryLimitExceeded => TreeCopyBlockerKind::EntryLimitExceeded,
        bounded_tree_core::BlockerKind::EmptyPath => TreeCopyBlockerKind::EmptyPath,
        bounded_tree_core::BlockerKind::EmptyComponent => TreeCopyBlockerKind::EmptyComponent,
        bounded_tree_core::BlockerKind::CurrentDirectoryComponent => TreeCopyBlockerKind::CurrentDirectoryComponent,
        bounded_tree_core::BlockerKind::ParentDirectoryComponent => TreeCopyBlockerKind::ParentDirectoryComponent,
        bounded_tree_core::BlockerKind::PathContainsNul => TreeCopyBlockerKind::InvalidCharacter,
        bounded_tree_core::BlockerKind::PathContainsForwardSlash => TreeCopyBlockerKind::InvalidCharacter,
        bounded_tree_core::BlockerKind::PathContainsBackslash => TreeCopyBlockerKind::BackslashSeparator,
        bounded_tree_core::BlockerKind::PathAccountingOverflow => TreeCopyBlockerKind::PathTooLong,
        bounded_tree_core::BlockerKind::PathDepthExceeded => TreeCopyBlockerKind::PathTooDeep,
        bounded_tree_core::BlockerKind::PathBytesExceeded => TreeCopyBlockerKind::PathTooLong,
        bounded_tree_core::BlockerKind::FileBytesExceeded
        | bounded_tree_core::BlockerKind::TotalBytesExceeded
        | bounded_tree_core::BlockerKind::TotalBytesOverflow => TreeCopyBlockerKind::EntryCountOverflow,
        bounded_tree_core::BlockerKind::SymlinkTargetBytesExceeded => TreeCopyBlockerKind::SymlinkTargetTooLong,
        bounded_tree_core::BlockerKind::UnsupportedKind => TreeCopyBlockerKind::UnsupportedKind,
        bounded_tree_core::BlockerKind::InvalidDirectoryFacts | bounded_tree_core::BlockerKind::InvalidFileFacts => {
            TreeCopyBlockerKind::UnexpectedSymlinkTarget
        }
        bounded_tree_core::BlockerKind::InvalidSymlinkFacts => TreeCopyBlockerKind::MissingSymlinkTarget,
        bounded_tree_core::BlockerKind::DuplicatePath => TreeCopyBlockerKind::DuplicatePath,
        bounded_tree_core::BlockerKind::MissingParent => TreeCopyBlockerKind::MissingParent,
        bounded_tree_core::BlockerKind::ParentNotDirectory => TreeCopyBlockerKind::ParentNotDirectory,
        bounded_tree_core::BlockerKind::SymlinkRejected => TreeCopyBlockerKind::UnsupportedKind,
        bounded_tree_core::BlockerKind::SymlinkTargetNotRelative => TreeCopyBlockerKind::AbsoluteSymlinkTarget,
        bounded_tree_core::BlockerKind::SymlinkTargetContainsNul
        | bounded_tree_core::BlockerKind::SymlinkTargetMalformed => TreeCopyBlockerKind::SymlinkTargetInvalidCharacter,
        bounded_tree_core::BlockerKind::SymlinkTargetContainsBackslash => TreeCopyBlockerKind::SymlinkTargetBackslash,
        bounded_tree_core::BlockerKind::SymlinkTargetEscapes => TreeCopyBlockerKind::SymlinkTargetEscapes,
        bounded_tree_core::BlockerKind::SymlinkTargetMissing => TreeCopyBlockerKind::SymlinkTargetMissing,
        bounded_tree_core::BlockerKind::SymlinkTargetSelf => TreeCopyBlockerKind::SymlinkTargetSelf,
        _ => TreeCopyBlockerKind::InvalidCharacter,
    }
}

fn compatibility_operation_sort_key(operation: &TreeCopyOperation) -> (u8, u32, &str) {
    match operation {
        TreeCopyOperation::CreateDirectory { relative_path, .. } => {
            (DIRECTORY_OPERATION_PHASE, path_depth_count(relative_path), relative_path)
        }
        TreeCopyOperation::CopyFile { relative_path, .. } => (FILE_OPERATION_PHASE, 0, relative_path),
        TreeCopyOperation::CreateSymlink { relative_path, .. } => (SYMLINK_OPERATION_PHASE, 0, relative_path),
    }
}

fn path_depth_count(path: &str) -> u32 {
    debug_assert!(!path.is_empty());
    path.bytes().filter(|byte| *byte == b'/').fold(1_u32, |depth, _separator| depth.saturating_add(1))
}

fn limit_error_kind(kind: bounded_tree_core::LimitKind) -> TreeCopyBlockerKind {
    match kind {
        bounded_tree_core::LimitKind::Entries => TreeCopyBlockerKind::InvalidEntryLimit,
        bounded_tree_core::LimitKind::Depth => TreeCopyBlockerKind::InvalidDepthLimit,
        bounded_tree_core::LimitKind::PathBytes => TreeCopyBlockerKind::InvalidPathLimit,
        _ => TreeCopyBlockerKind::InvalidPathLimit,
    }
}

fn blocker(kind: TreeCopyBlockerKind, relative_path: Option<String>, message: &str) -> TreeCopyBlocker {
    TreeCopyBlocker {
        kind,
        relative_path,
        message: message.to_string(),
    }
}

struct PathBlockerInput<'a> {
    kind: TreeCopyBlockerKind,
    path: &'a str,
    message: &'a str,
}

fn path_blocker(input: PathBlockerInput<'_>) -> TreeCopyBlocker {
    blocker(input.kind, Some(input.path.to_string()), input.message)
}

fn blocker_order(left: &TreeCopyBlocker, right: &TreeCopyBlocker) -> core::cmp::Ordering {
    left.relative_path
        .cmp(&right.relative_path)
        .then_with(|| left.kind.cmp(&right.kind))
        .then_with(|| left.message.cmp(&right.message))
}

fn looks_like_windows_prefix(path: &str) -> bool {
    let bytes = path.as_bytes();
    bytes.len() >= WINDOWS_DRIVE_PREFIX_BYTES && bytes[1] == b':' && bytes[0].is_ascii_alphabetic()
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    const DIRECTORY_MODE: u32 = 0o040755;
    const FILE_MODE: u32 = 0o100644;
    const SYMLINK_MODE: u32 = 0o120777;
    const TWO_ENTRIES_COUNT: u32 = 2;
    const ONE_COMPONENT_DEPTH: u32 = 1;
    const SHORT_PATH_BYTES: u32 = 3;
    const SHORT_TARGET_BYTES: u32 = 4;
    const SYMLINK_OPERATION_INDEX: usize = 2;

    fn observation(path: &str, kind: TreeEntryKind) -> TreeEntryObservation {
        TreeEntryObservation {
            relative_path: path.to_string(),
            kind,
            mode: match kind {
                TreeEntryKind::Directory => DIRECTORY_MODE,
                TreeEntryKind::File => FILE_MODE,
                TreeEntryKind::Symlink => SYMLINK_MODE,
                TreeEntryKind::Unsupported => 0,
            },
            symlink_target: None,
        }
    }

    fn symlink(path: &str, target: &str) -> TreeEntryObservation {
        TreeEntryObservation {
            symlink_target: Some(target.to_string()),
            ..observation(path, TreeEntryKind::Symlink)
        }
    }

    fn plan(observations: Vec<TreeEntryObservation>) -> Result<TreeCopyPlan, Vec<TreeCopyBlocker>> {
        plan_tree_copy(observations, TreeCopyLimits::RELEASE_BUNDLE)
    }

    // r[verify mantle.release_provenance.bundle_tree_copy.fixtures.positive]
    // r[verify mantle.bounded_tree_adoption.parity]
    #[test]
    fn deterministic_plan_orders_directories_files_then_internal_symlinks() {
        let first = plan(vec![
            symlink("latest", "nested/data.txt"),
            observation("nested/data.txt", TreeEntryKind::File),
            observation("nested", TreeEntryKind::Directory),
        ])
        .unwrap();
        let second = plan(vec![
            observation("nested", TreeEntryKind::Directory),
            symlink("latest", "nested/data.txt"),
            observation("nested/data.txt", TreeEntryKind::File),
        ])
        .unwrap();

        assert_eq!(first, second);
        assert_eq!(first.entries[0].relative_path, "latest");
        assert!(matches!(first.operations[0], TreeCopyOperation::CreateDirectory { .. }));
        assert!(matches!(first.operations[1], TreeCopyOperation::CopyFile { .. }));
        assert!(matches!(first.operations[SYMLINK_OPERATION_INDEX], TreeCopyOperation::CreateSymlink {
            mode: SYMLINK_MODE,
            ..
        }));
    }

    // r[verify mantle.release_provenance.bundle_tree_copy.plan.invalid]
    // r[verify mantle.bounded_tree_adoption.parity]
    #[test]
    fn invalid_paths_parent_shapes_duplicates_and_special_files_are_rejected() {
        let cases = [
            (vec![observation("/absolute", TreeEntryKind::File)], TreeCopyBlockerKind::AbsolutePath),
            (vec![observation("../escape", TreeEntryKind::File)], TreeCopyBlockerKind::ParentDirectoryComponent),
            (vec![observation("missing/file", TreeEntryKind::File)], TreeCopyBlockerKind::MissingParent),
            (
                vec![
                    observation("parent", TreeEntryKind::File),
                    observation("parent/child", TreeEntryKind::File),
                ],
                TreeCopyBlockerKind::ParentNotDirectory,
            ),
            (
                vec![
                    observation("duplicate", TreeEntryKind::File),
                    observation("duplicate", TreeEntryKind::File),
                ],
                TreeCopyBlockerKind::DuplicatePath,
            ),
            (vec![observation("socket", TreeEntryKind::Unsupported)], TreeCopyBlockerKind::UnsupportedKind),
        ];

        for (observations, expected_kind) in cases {
            let blockers = plan(observations).unwrap_err();
            assert!(blockers.iter().any(|blocker| blocker.kind == expected_kind), "missing {expected_kind:?}");
        }
    }

    // r[verify mantle.release_provenance.bundle_tree_copy.fixtures.negative.target]
    // r[verify mantle.bounded_tree_adoption.parity]
    #[test]
    fn symlink_policy_accepts_only_relative_targets_naming_another_planned_entry() {
        let accepted = plan(vec![
            observation("target", TreeEntryKind::File),
            observation("nested", TreeEntryKind::Directory),
            symlink("nested/alias", "../target"),
        ]);
        assert!(accepted.is_ok());

        let cases = [
            (symlink("alias", "/absolute"), TreeCopyBlockerKind::AbsoluteSymlinkTarget),
            (symlink("alias", "../escape"), TreeCopyBlockerKind::SymlinkTargetEscapes),
            (symlink("alias", "missing"), TreeCopyBlockerKind::SymlinkTargetMissing),
            (symlink("alias", "alias"), TreeCopyBlockerKind::SymlinkTargetSelf),
        ];
        for (candidate, expected_kind) in cases {
            let blockers = plan(vec![candidate]).unwrap_err();
            assert!(blockers.iter().any(|blocker| blocker.kind == expected_kind), "missing {expected_kind:?}");
        }
    }

    #[test]
    fn compatibility_order_uses_mantle_string_paths_not_component_order() {
        let planned = plan(vec![
            observation("a", TreeEntryKind::Directory),
            observation("a/child", TreeEntryKind::File),
            observation("a-entry", TreeEntryKind::File),
        ])
        .unwrap();
        let entry_paths = planned.entries.iter().map(|entry| entry.relative_path.as_str()).collect::<Vec<_>>();
        let operation_paths = planned
            .operations
            .iter()
            .map(|operation| match operation {
                TreeCopyOperation::CreateDirectory { relative_path, .. }
                | TreeCopyOperation::CopyFile { relative_path, .. }
                | TreeCopyOperation::CreateSymlink { relative_path, .. } => relative_path.as_str(),
            })
            .collect::<Vec<_>>();

        assert_eq!(entry_paths, vec!["a", "a-entry", "a/child"]);
        assert_eq!(operation_paths, vec!["a", "a-entry", "a/child"]);
    }

    #[test]
    fn named_entry_depth_and_path_bounds_fail_closed() {
        let entry_limit = TreeCopyLimits {
            entries_count_max: 1,
            ..TreeCopyLimits::RELEASE_BUNDLE
        };
        let blockers = plan_tree_copy(
            vec![
                observation("first", TreeEntryKind::File),
                observation("second", TreeEntryKind::File),
            ],
            entry_limit,
        )
        .unwrap_err();
        assert_eq!(blockers[0].kind, TreeCopyBlockerKind::EntryLimitExceeded);

        let depth_limit = TreeCopyLimits {
            depth_count_max: ONE_COMPONENT_DEPTH,
            ..TreeCopyLimits::RELEASE_BUNDLE
        };
        let blockers = plan_tree_copy(
            vec![
                observation("dir", TreeEntryKind::Directory),
                observation("dir/file", TreeEntryKind::File),
            ],
            depth_limit,
        )
        .unwrap_err();
        assert!(blockers.iter().any(|blocker| blocker.kind == TreeCopyBlockerKind::PathTooDeep));

        let path_limit = TreeCopyLimits {
            path_bytes_max: SHORT_PATH_BYTES,
            ..TreeCopyLimits::RELEASE_BUNDLE
        };
        let blockers = plan_tree_copy(vec![observation("long", TreeEntryKind::File)], path_limit).unwrap_err();
        assert_eq!(blockers[0].kind, TreeCopyBlockerKind::PathTooLong);

        let target_limit = TreeCopyLimits {
            path_bytes_max: SHORT_TARGET_BYTES,
            ..TreeCopyLimits::RELEASE_BUNDLE
        };
        let blockers = plan_tree_copy(vec![observation("t", TreeEntryKind::File), symlink("a", "././t")], target_limit)
            .unwrap_err();
        assert_eq!(blockers[0].kind, TreeCopyBlockerKind::SymlinkTargetTooLong);
    }

    #[test]
    fn blocker_order_is_deterministic_by_path_and_kind() {
        let blockers = plan(vec![
            observation("z/file", TreeEntryKind::File),
            observation("a/file", TreeEntryKind::File),
        ])
        .unwrap_err();

        assert_eq!(blockers.len(), usize::try_from(TWO_ENTRIES_COUNT).unwrap());
        assert_eq!(blockers[0].relative_path.as_deref(), Some("a/file"));
        assert_eq!(blockers[1].relative_path.as_deref(), Some("z/file"));
    }

    #[test]
    fn zero_limits_are_rejected_before_entry_validation() {
        let limits = TreeCopyLimits {
            entries_count_max: 0,
            depth_count_max: 0,
            path_bytes_max: 0,
        };
        let blockers = plan_tree_copy(vec![observation("file", TreeEntryKind::File)], limits).unwrap_err();

        assert_eq!(blockers.len(), THREE_LIMIT_BLOCKERS_COUNT);
        assert!(blockers.iter().all(|blocker| blocker.relative_path.is_none()));
    }

    const THREE_LIMIT_BLOCKERS_COUNT: usize = 3;
}
