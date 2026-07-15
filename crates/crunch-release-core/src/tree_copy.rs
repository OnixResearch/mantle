use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

pub const RELEASE_TREE_COPY_MAX_ENTRIES_COUNT: u32 = 4_096;
pub const RELEASE_TREE_COPY_MAX_DEPTH_COUNT: u32 = 128;
pub const RELEASE_TREE_COPY_MAX_PATH_BYTES: u32 = 4_096;

const WINDOWS_DRIVE_PREFIX_BYTES: usize = 2;

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

#[derive(Debug, Clone)]
struct ValidatedObservation {
    observation: TreeEntryObservation,
    components: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
struct SymlinkTargetInput<'a> {
    link_components: &'a [String],
    target: &'a str,
    link_path: &'a str,
    limits: TreeCopyLimits,
}

#[derive(Debug, Clone, Copy)]
struct NormalizedSymlinkTargetInput<'a> {
    link_path: &'a str,
    target_path: &'a str,
}

// r[impl mantle.release_provenance.bundle_tree_copy.plan]
// r[impl mantle.release_provenance.bundle_tree_copy.plan.invalid]
// r[impl mantle.release_provenance.bundle_tree_copy.symlink_policy]
pub fn plan_tree_copy(
    mut observations: Vec<TreeEntryObservation>,
    limits: TreeCopyLimits,
) -> Result<TreeCopyPlan, Vec<TreeCopyBlocker>> {
    let mut blockers = validate_limits(limits);
    if !blockers.is_empty() {
        return Err(blockers);
    }
    validate_entry_count(observations.len(), limits.entries_count_max)?;
    observations.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));

    blockers.extend(duplicate_path_blockers(&observations));
    let validated = validate_observations(observations, limits, &mut blockers);
    validate_parent_shapes(&validated, &mut blockers);
    validate_symlink_targets(&validated, limits, &mut blockers);
    sort_blockers(&mut blockers);
    if !blockers.is_empty() {
        return Err(blockers);
    }

    let entries = validated.into_iter().map(|entry| entry.observation).collect::<Vec<_>>();
    let operations = build_operations(&entries)?;
    debug_assert_eq!(entries.len(), operations.len());
    debug_assert!(operations_are_phase_ordered(&operations));
    Ok(TreeCopyPlan { entries, operations })
}

fn validate_limits(limits: TreeCopyLimits) -> Vec<TreeCopyBlocker> {
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
    sort_blockers(&mut blockers);
    blockers
}

fn validate_entry_count(entries_count: usize, entries_count_max: u32) -> Result<(), Vec<TreeCopyBlocker>> {
    let count = u32::try_from(entries_count).map_err(|_| {
        vec![blocker(
            TreeCopyBlockerKind::EntryCountOverflow,
            None,
            "tree copy entry count overflowed u32",
        )]
    })?;
    if count > entries_count_max {
        return Err(vec![blocker(
            TreeCopyBlockerKind::EntryLimitExceeded,
            None,
            &format!("tree copy entry count {count} exceeds {entries_count_max}"),
        )]);
    }
    Ok(())
}

fn duplicate_path_blockers(observations: &[TreeEntryObservation]) -> Vec<TreeCopyBlocker> {
    let mut blockers = Vec::with_capacity(observations.len());
    for pair in observations.windows(WINDOW_PAIR_COUNT) {
        if pair[0].relative_path == pair[1].relative_path {
            blockers.push(blocker(
                TreeCopyBlockerKind::DuplicatePath,
                Some(pair[0].relative_path.clone()),
                &format!("tree copy contains duplicate path {}", pair[0].relative_path),
            ));
        }
    }
    blockers
}

const WINDOW_PAIR_COUNT: usize = 2;

fn validate_observations(
    observations: Vec<TreeEntryObservation>,
    limits: TreeCopyLimits,
    blockers: &mut Vec<TreeCopyBlocker>,
) -> Vec<ValidatedObservation> {
    let mut validated = Vec::with_capacity(observations.len());
    for observation in observations {
        let components = match validate_entry_path(&observation.relative_path, limits) {
            Ok(components) => components,
            Err(path_blocker) => {
                blockers.push(path_blocker);
                continue;
            }
        };
        validate_entry_shape(&observation, blockers);
        validated.push(ValidatedObservation {
            observation,
            components,
        });
    }
    validated
}

fn validate_entry_path(path: &str, limits: TreeCopyLimits) -> Result<Vec<String>, TreeCopyBlocker> {
    debug_assert!(limits.path_bytes_max > 0);
    debug_assert!(limits.depth_count_max > 0);
    validate_path_prefix(path)?;
    let path_bytes = u32::try_from(path.len())
        .map_err(|_| path_blocker(TreeCopyBlockerKind::PathTooLong, path, "tree copy path length overflowed u32"))?;
    if path_bytes > limits.path_bytes_max {
        return Err(path_blocker(
            TreeCopyBlockerKind::PathTooLong,
            path,
            format!("tree copy path exceeds {} bytes: {path}", limits.path_bytes_max),
        ));
    }
    let components = validate_entry_components(path)?;
    let depth_count = u32::try_from(components.len())
        .map_err(|_| path_blocker(TreeCopyBlockerKind::PathTooDeep, path, "tree copy path depth overflowed u32"))?;
    if depth_count > limits.depth_count_max {
        return Err(path_blocker(
            TreeCopyBlockerKind::PathTooDeep,
            path,
            format!("tree copy path depth {depth_count} exceeds {}: {path}", limits.depth_count_max),
        ));
    }
    Ok(components)
}

fn validate_path_prefix(path: &str) -> Result<(), TreeCopyBlocker> {
    if path.is_empty() {
        return Err(path_blocker(TreeCopyBlockerKind::EmptyPath, path, "tree copy path must not be empty"));
    }
    if path.starts_with('/') {
        return Err(path_blocker(TreeCopyBlockerKind::AbsolutePath, path, "tree copy path must be relative"));
    }
    if looks_like_windows_prefix(path) {
        return Err(path_blocker(
            TreeCopyBlockerKind::WindowsPrefix,
            path,
            "tree copy path must not use a drive prefix",
        ));
    }
    if path.contains('\\') {
        return Err(path_blocker(
            TreeCopyBlockerKind::BackslashSeparator,
            path,
            "tree copy path must use '/' separators",
        ));
    }
    debug_assert!(!path.is_empty());
    debug_assert!(!path.starts_with('/'));
    Ok(())
}

fn validate_entry_components(path: &str) -> Result<Vec<String>, TreeCopyBlocker> {
    let component_count = path.bytes().filter(|byte| *byte == b'/').count().saturating_add(1);
    let mut components = Vec::with_capacity(component_count);
    for component in path.split('/') {
        if component.is_empty() {
            return Err(path_blocker(
                TreeCopyBlockerKind::EmptyComponent,
                path,
                "tree copy path has an empty component",
            ));
        }
        if component == "." {
            return Err(path_blocker(
                TreeCopyBlockerKind::CurrentDirectoryComponent,
                path,
                "tree copy path has a current-directory component",
            ));
        }
        if component == ".." {
            return Err(path_blocker(
                TreeCopyBlockerKind::ParentDirectoryComponent,
                path,
                "tree copy path has a parent-directory component",
            ));
        }
        if component.contains('\0') {
            return Err(path_blocker(TreeCopyBlockerKind::InvalidCharacter, path, "tree copy path contains NUL"));
        }
        components.push(component.to_string());
    }
    debug_assert_eq!(components.len(), component_count);
    debug_assert_eq!(components.join("/"), path);
    Ok(components)
}

fn validate_entry_shape(observation: &TreeEntryObservation, blockers: &mut Vec<TreeCopyBlocker>) {
    if observation.kind == TreeEntryKind::Unsupported {
        blockers.push(path_blocker(
            TreeCopyBlockerKind::UnsupportedKind,
            &observation.relative_path,
            "tree copy entry kind is unsupported",
        ));
    }
    if observation.kind == TreeEntryKind::Symlink && observation.symlink_target.is_none() {
        blockers.push(path_blocker(
            TreeCopyBlockerKind::MissingSymlinkTarget,
            &observation.relative_path,
            "tree copy symlink must record a target",
        ));
    }
    if observation.kind != TreeEntryKind::Symlink && observation.symlink_target.is_some() {
        blockers.push(path_blocker(
            TreeCopyBlockerKind::UnexpectedSymlinkTarget,
            &observation.relative_path,
            "non-symlink tree copy entry must not record a symlink target",
        ));
    }
    if observation.kind == TreeEntryKind::Symlink && observation.symlink_target.is_none() {
        debug_assert!(blockers.iter().any(|blocker| blocker.kind == TreeCopyBlockerKind::MissingSymlinkTarget));
    }
    if observation.kind != TreeEntryKind::Symlink && observation.symlink_target.is_some() {
        debug_assert!(blockers.iter().any(|blocker| blocker.kind == TreeCopyBlockerKind::UnexpectedSymlinkTarget));
    }
}

fn validate_parent_shapes(entries: &[ValidatedObservation], blockers: &mut Vec<TreeCopyBlocker>) {
    debug_assert!(u32::try_from(entries.len()).is_ok());
    debug_assert!(entries.iter().all(|entry| !entry.components.is_empty()));
    let kinds = entries
        .iter()
        .map(|entry| (entry.observation.relative_path.clone(), entry.observation.kind))
        .collect::<BTreeMap<_, _>>();
    for entry in entries {
        let Some(parent_path) = parent_path(&entry.components) else {
            continue;
        };
        match kinds.get(&parent_path) {
            None => blockers.push(path_blocker(
                TreeCopyBlockerKind::MissingParent,
                &entry.observation.relative_path,
                format!("tree copy path {} has missing parent {parent_path}", entry.observation.relative_path),
            )),
            Some(TreeEntryKind::Directory) => {}
            Some(_) => blockers.push(path_blocker(
                TreeCopyBlockerKind::ParentNotDirectory,
                &entry.observation.relative_path,
                format!("tree copy path {} has non-directory parent {parent_path}", entry.observation.relative_path),
            )),
        }
    }
}

fn validate_symlink_targets(
    entries: &[ValidatedObservation],
    limits: TreeCopyLimits,
    blockers: &mut Vec<TreeCopyBlocker>,
) {
    debug_assert!(limits.path_bytes_max > 0);
    debug_assert!(limits.depth_count_max > 0);
    let kinds = entries
        .iter()
        .map(|entry| (entry.observation.relative_path.clone(), entry.observation.kind))
        .collect::<BTreeMap<_, _>>();
    for entry in entries {
        if entry.observation.kind != TreeEntryKind::Symlink {
            continue;
        }
        let Some(target) = entry.observation.symlink_target.as_deref() else {
            continue;
        };
        let target_input = SymlinkTargetInput {
            link_components: &entry.components,
            target,
            link_path: &entry.observation.relative_path,
            limits,
        };
        let normalized = match normalize_symlink_target(target_input) {
            Ok(normalized) => normalized,
            Err(target_blocker) => {
                blockers.push(target_blocker);
                continue;
            }
        };
        validate_normalized_symlink_target(
            NormalizedSymlinkTargetInput {
                link_path: &entry.observation.relative_path,
                target_path: &normalized,
            },
            &kinds,
            blockers,
        );
    }
}

fn normalize_symlink_target(input: SymlinkTargetInput<'_>) -> Result<String, TreeCopyBlocker> {
    debug_assert!(!input.link_components.is_empty());
    debug_assert!(input.limits.depth_count_max > 0);
    validate_symlink_target_prefix(input)?;
    validate_symlink_target_length(input)?;
    let resolved_slots_count = usize::try_from(input.limits.depth_count_max).map_err(|_| {
        target_blocker(
            TreeCopyBlockerKind::SymlinkTargetTooDeep,
            input.link_path,
            "tree copy symlink target depth limit does not fit usize",
        )
    })?;
    let parent_component_count = input.link_components.len().saturating_sub(1);
    let mut resolved = Vec::with_capacity(resolved_slots_count);
    resolved.extend_from_slice(&input.link_components[..parent_component_count]);
    for component in input.target.split('/') {
        if component.is_empty() || component == "." {
            continue;
        }
        if component == ".." {
            if resolved.pop().is_none() {
                return Err(target_blocker(
                    TreeCopyBlockerKind::SymlinkTargetEscapes,
                    input.link_path,
                    "tree copy symlink target escapes the planned root",
                ));
            }
            continue;
        }
        if component.contains('\0') {
            return Err(target_blocker(
                TreeCopyBlockerKind::SymlinkTargetInvalidCharacter,
                input.link_path,
                "tree copy symlink target contains NUL",
            ));
        }
        if resolved.len() >= resolved_slots_count {
            return Err(target_blocker(
                TreeCopyBlockerKind::SymlinkTargetTooDeep,
                input.link_path,
                "tree copy symlink target exceeds the configured depth bound",
            ));
        }
        resolved.push(component.to_string());
    }
    let depth_count = u32::try_from(resolved.len()).map_err(|_| {
        target_blocker(
            TreeCopyBlockerKind::SymlinkTargetTooDeep,
            input.link_path,
            "tree copy symlink target depth overflowed u32",
        )
    })?;
    if depth_count > input.limits.depth_count_max {
        return Err(target_blocker(
            TreeCopyBlockerKind::SymlinkTargetTooDeep,
            input.link_path,
            format!("tree copy symlink target depth {depth_count} exceeds {}", input.limits.depth_count_max),
        ));
    }
    Ok(resolved.join("/"))
}

fn validate_symlink_target_length(input: SymlinkTargetInput<'_>) -> Result<(), TreeCopyBlocker> {
    let target_bytes = u32::try_from(input.target.len()).map_err(|_| {
        target_blocker(
            TreeCopyBlockerKind::SymlinkTargetTooLong,
            input.link_path,
            "tree copy symlink target length overflowed u32",
        )
    })?;
    if target_bytes > input.limits.path_bytes_max {
        return Err(target_blocker(
            TreeCopyBlockerKind::SymlinkTargetTooLong,
            input.link_path,
            format!("tree copy symlink target exceeds {} bytes", input.limits.path_bytes_max),
        ));
    }
    Ok(())
}

fn validate_symlink_target_prefix(input: SymlinkTargetInput<'_>) -> Result<(), TreeCopyBlocker> {
    if input.target.is_empty() || input.target.starts_with('/') {
        return Err(target_blocker(
            TreeCopyBlockerKind::AbsoluteSymlinkTarget,
            input.link_path,
            "tree copy symlink target must be non-empty and relative",
        ));
    }
    if looks_like_windows_prefix(input.target) {
        return Err(target_blocker(
            TreeCopyBlockerKind::SymlinkTargetWindowsPrefix,
            input.link_path,
            "tree copy symlink target must not use a drive prefix",
        ));
    }
    if input.target.contains('\\') {
        return Err(target_blocker(
            TreeCopyBlockerKind::SymlinkTargetBackslash,
            input.link_path,
            "tree copy symlink target must use '/' separators",
        ));
    }
    debug_assert!(!input.target.is_empty());
    debug_assert!(!input.target.starts_with('/'));
    Ok(())
}

fn validate_normalized_symlink_target(
    input: NormalizedSymlinkTargetInput<'_>,
    kinds: &BTreeMap<String, TreeEntryKind>,
    blockers: &mut Vec<TreeCopyBlocker>,
) {
    if input.target_path == input.link_path {
        blockers.push(target_blocker(
            TreeCopyBlockerKind::SymlinkTargetSelf,
            input.link_path,
            "tree copy symlink target must name another planned entry",
        ));
        return;
    }
    if !kinds.contains_key(input.target_path) {
        blockers.push(target_blocker(
            TreeCopyBlockerKind::SymlinkTargetMissing,
            input.link_path,
            format!("tree copy symlink target does not name a planned entry: {}", input.target_path),
        ));
    }
}

fn build_operations(entries: &[TreeEntryObservation]) -> Result<Vec<TreeCopyOperation>, Vec<TreeCopyBlocker>> {
    let mut operations = Vec::with_capacity(entries.len());
    for entry in entries {
        operations.push(operation_for_entry(entry).map_err(|blocker| vec![blocker])?);
    }
    operations.sort_by(|left, right| operation_sort_key(left).cmp(&operation_sort_key(right)));
    debug_assert_eq!(operations.len(), entries.len());
    debug_assert!(operations_are_phase_ordered(&operations));
    Ok(operations)
}

fn operation_for_entry(entry: &TreeEntryObservation) -> Result<TreeCopyOperation, TreeCopyBlocker> {
    match entry.kind {
        TreeEntryKind::Directory => Ok(TreeCopyOperation::CreateDirectory {
            relative_path: entry.relative_path.clone(),
            mode: entry.mode,
        }),
        TreeEntryKind::File => Ok(TreeCopyOperation::CopyFile {
            relative_path: entry.relative_path.clone(),
            mode: entry.mode,
        }),
        TreeEntryKind::Symlink => symlink_operation(entry),
        TreeEntryKind::Unsupported => Err(path_blocker(
            TreeCopyBlockerKind::UnsupportedKind,
            &entry.relative_path,
            "unsupported entries cannot reach operation planning",
        )),
    }
}

fn symlink_operation(entry: &TreeEntryObservation) -> Result<TreeCopyOperation, TreeCopyBlocker> {
    let Some(target) = entry.symlink_target.clone() else {
        return Err(path_blocker(
            TreeCopyBlockerKind::MissingSymlinkTarget,
            &entry.relative_path,
            "validated symlink operation is missing its target",
        ));
    };
    Ok(TreeCopyOperation::CreateSymlink {
        relative_path: entry.relative_path.clone(),
        mode: entry.mode,
        target,
    })
}

fn operation_sort_key(operation: &TreeCopyOperation) -> (u8, u32, &str) {
    match operation {
        TreeCopyOperation::CreateDirectory { relative_path, .. } => {
            (DIRECTORY_PHASE, path_depth_count(relative_path), relative_path)
        }
        TreeCopyOperation::CopyFile { relative_path, .. } => (FILE_PHASE, 0, relative_path),
        TreeCopyOperation::CreateSymlink { relative_path, .. } => (SYMLINK_PHASE, 0, relative_path),
    }
}

const DIRECTORY_PHASE: u8 = 0;
const FILE_PHASE: u8 = 1;
const SYMLINK_PHASE: u8 = 2;

fn operations_are_phase_ordered(operations: &[TreeCopyOperation]) -> bool {
    operations
        .windows(WINDOW_PAIR_COUNT)
        .all(|pair| operation_sort_key(&pair[0]) <= operation_sort_key(&pair[1]))
}

fn path_depth_count(path: &str) -> u32 {
    debug_assert!(u32::try_from(path.len()).is_ok());
    debug_assert!(!path.is_empty());
    path.bytes()
        .filter(|byte| *byte == b'/')
        .fold(1_u32, |depth_count, _separator| depth_count.saturating_add(1))
}

fn parent_path(components: &[String]) -> Option<String> {
    if components.len() <= 1 {
        return None;
    }
    Some(components[..components.len().saturating_sub(1)].join("/"))
}

fn looks_like_windows_prefix(path: &str) -> bool {
    let bytes = path.as_bytes();
    bytes.len() >= WINDOWS_DRIVE_PREFIX_BYTES && bytes[1] == b':' && bytes[0].is_ascii_alphabetic()
}

fn blocker(kind: TreeCopyBlockerKind, relative_path: Option<String>, message: &str) -> TreeCopyBlocker {
    TreeCopyBlocker {
        kind,
        relative_path,
        message: message.to_string(),
    }
}

fn path_blocker(kind: TreeCopyBlockerKind, path: &str, message: impl AsRef<str>) -> TreeCopyBlocker {
    blocker(kind, Some(path.to_string()), message.as_ref())
}

fn target_blocker(kind: TreeCopyBlockerKind, link_path: &str, message: impl AsRef<str>) -> TreeCopyBlocker {
    path_blocker(kind, link_path, message)
}

fn sort_blockers(blockers: &mut [TreeCopyBlocker]) {
    blockers.sort_by(|left, right| {
        left.relative_path
            .cmp(&right.relative_path)
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| left.message.cmp(&right.message))
    });
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
        assert!(matches!(first.operations[SYMLINK_OPERATION_INDEX], TreeCopyOperation::CreateSymlink { .. }));
    }

    // r[verify mantle.release_provenance.bundle_tree_copy.plan.invalid]
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
