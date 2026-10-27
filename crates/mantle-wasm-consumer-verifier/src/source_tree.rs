//! Mantle v1 framing over a bounded-tree plan, not a second tree walker.

use std::borrow::ToOwned;
use std::io::Read;
use std::io::Write;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::path::PathBuf;
use std::string::String;
use std::string::ToString;
use std::vec::Vec;

use bounded_tree_cap::PreparedTree;
use bounded_tree_core::EntryKind;
use bounded_tree_core::LimitValues;
use bounded_tree_core::SymlinkPolicy;
use bounded_tree_core::TreeLimits;
use cap_fs_ext::DirExt;
use cap_fs_ext::FollowSymlinks;
use cap_fs_ext::OpenOptionsFollowExt;
use cap_fs_ext::OpenOptionsSyncExt;
use cap_std::fs::Dir;
use cap_std::fs::File;
use cap_std::fs::MetadataExt;
use cap_std::fs::OpenOptions;

use crate::shell::ShellError;

pub(crate) const MAX_TREE_ENTRIES: u64 = 10_000;
pub(crate) const MAX_TREE_DEPTH: u64 = 64;
pub(crate) const MAX_TREE_BYTES: u64 = 536_870_912;
const STREAM_BUFFER_BYTES: usize = 65_536;

/// Resolve each parent separately: capability containment alone permits internal
/// symlinks, whereas the published verifier rejects them in every component.
pub(crate) fn open_parent(root: &Dir, relative: &Path, create: bool) -> Result<(Dir, std::ffi::OsString), ShellError> {
    let mut parent = root.try_clone().map_err(|_| ShellError::MemberUnavailable)?;
    let mut components = relative.components();
    let leaf = components.next_back().ok_or(ShellError::InvalidMemberPath)?;
    for component in components {
        if create {
            match parent.create_dir(component.as_os_str()) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(_) => return Err(ShellError::DestinationUnavailable),
            }
        }
        parent = parent.open_dir_nofollow(component.as_os_str()).map_err(|_| ShellError::MemberUnavailable)?;
    }
    Ok((parent, leaf.as_os_str().to_owned()))
}

/// Nonblocking protects the metadata/open race from regular-file-to-FIFO
/// substitution. Opened metadata, rather than the earlier name, is authoritative.
pub(crate) fn open_regular(parent: &Dir, name: &Path, expected_bytes: u64) -> Result<File, ShellError> {
    if !parent.symlink_metadata(name).map_err(|_| ShellError::MemberUnavailable)?.is_file() {
        return Err(ShellError::UnsupportedMember);
    }
    let mut options = OpenOptions::new();
    options.read(true).follow(FollowSymlinks::No).nonblock(true);
    let file = parent.open_with(name, &options).map_err(|_| ShellError::MemberUnavailable)?;
    check_file(&file, expected_bytes)?;
    Ok(file)
}

fn check_file(file: &File, expected_bytes: u64) -> Result<(), ShellError> {
    let metadata = file.metadata().map_err(|_| ShellError::MemberUnavailable)?;
    if !metadata.is_file() {
        return Err(ShellError::UnsupportedMember);
    }
    if metadata.len() != expected_bytes {
        return Err(ShellError::MemberMismatch);
    }
    Ok(())
}

/// Stream exactly the declared file bytes plus one EOF probe. Optionally copy
/// into a newly created file while updating Mantle's enclosing tree hash.
pub(crate) fn stream_file(
    file: &mut File,
    mut output: Option<&mut File>,
    expected_bytes: u64,
    mut enclosing: Option<&mut blake3::Hasher>,
) -> Result<blake3::Hash, ShellError> {
    debug_assert!(expected_bytes <= crate::shell::MAX_MEMBER_BYTES);
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0_u8; STREAM_BUFFER_BYTES];
    let mut remaining = expected_bytes;
    while remaining > 0 {
        let capacity_bytes =
            usize::try_from(remaining.min(STREAM_BUFFER_BYTES as u64)).map_err(|_| ShellError::BoundExceeded)?;
        let count = file.read(&mut buffer[..capacity_bytes]).map_err(|_| ShellError::MemberUnavailable)?;
        debug_assert!(count <= capacity_bytes);
        if count == 0 {
            return Err(ShellError::MemberMismatch);
        }
        if let Some(writer) = output.as_deref_mut() {
            writer.write_all(&buffer[..count]).map_err(|_| ShellError::DestinationUnavailable)?;
        }
        if let Some(tree) = enclosing.as_deref_mut() {
            tree.update(&buffer[..count]);
        }
        hasher.update(&buffer[..count]);
        remaining -= count as u64;
    }
    if file.read(&mut buffer[..1]).map_err(|_| ShellError::MemberUnavailable)? != 0 {
        return Err(ShellError::MemberMismatch);
    }
    check_file(file, expected_bytes)?;
    Ok(hasher.finalize())
}

pub(crate) fn create_file(parent: &Dir, name: &Path) -> Result<File, ShellError> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true).follow(FollowSymlinks::No);
    parent.open_with(name, &options).map_err(|_| ShellError::DestinationUnavailable)
}

pub(crate) fn prepare(source: &Dir) -> Result<PreparedTree, ShellError> {
    // Producer checks directory depth when dequeuing it. A file directly under
    // directory depth 64 therefore has 65 components and remains admissible.
    let tree_bounds = TreeLimits::new(LimitValues {
        entries: MAX_TREE_ENTRIES,
        depth: MAX_TREE_DEPTH.saturating_add(1),
        path_bytes: MAX_TREE_DEPTH.saturating_add(1).saturating_mul(256),
        file_bytes: MAX_TREE_BYTES,
        total_bytes: MAX_TREE_BYTES,
        symlink_target_bytes: 1,
    })
    .map_err(|_| ShellError::BoundExceeded)?;
    let prepared = bounded_tree_cap::prepare(source, tree_bounds, SymlinkPolicy::Reject).map_err(ShellError::Tree)?;
    debug_assert!(prepared.plan().member_facts().len() as u64 <= MAX_TREE_ENTRIES);
    debug_assert_eq!(prepared.plan().operations().len(), prepared.plan().member_facts().len());
    if prepared
        .plan()
        .member_facts()
        .iter()
        .any(|fact| fact.kind() == EntryKind::Directory && fact.path().components().len() as u64 > MAX_TREE_DEPTH)
    {
        return Err(ShellError::BoundExceeded);
    }
    Ok(prepared)
}

// r[impl mantle.wasm_source_closure_trees.identity]
// r[impl mantle.wasm_source_closure_trees.bounds]
pub(crate) fn measure(source: &Dir, prepared: &PreparedTree) -> Result<(String, u64), ShellError> {
    debug_assert!(prepared.plan().member_facts().len() as u64 <= MAX_TREE_ENTRIES);
    let mut facts = prepared.plan().member_facts().iter().collect::<Vec<_>>();
    // BFS, sorted direct native filenames: at each depth, parents are visited
    // in lexicographic component order. Do not sort lossy UTF-8 text.
    facts.sort_by(|left, right| {
        let left = left.path().components();
        let right = right.path().components();
        left.len().cmp(&right.len()).then_with(|| left.cmp(right))
    });
    let mut hasher = blake3::Hasher::new();
    let mut total_bytes = 0_u64;
    for fact in facts {
        let path = native_path(fact.path().components());
        let text = path.to_string_lossy();
        let tag: &[u8] = match fact.kind() {
            EntryKind::Directory => b"dir\0",
            EntryKind::File => b"file\0",
            _ => return Err(ShellError::UnsupportedMember),
        };
        hasher.update(tag);
        hasher.update(&(text.len() as u64).to_le_bytes());
        hasher.update(text.as_bytes());
        hasher.update(&fact.file_bytes().to_le_bytes());
        if fact.kind() == EntryKind::File {
            let (parent, name) = open_parent(source, &path, false)?;
            let mut file = open_regular(&parent, Path::new(&name), fact.file_bytes())?;
            let digest = stream_file(&mut file, None, fact.file_bytes(), Some(&mut hasher))?;
            if fact.file_content().map(|value| *value.as_bytes()) != Some(*digest.as_bytes()) {
                return Err(ShellError::MemberMismatch);
            }
            total_bytes = total_bytes.checked_add(fact.file_bytes()).ok_or(ShellError::BoundExceeded)?;
        }
    }
    // Re-observe names, empty directories, kinds, and content. This detects
    // ordinary concurrent changes, but is deliberately not an atomic snapshot.
    if prepare(source)?.plan() != prepared.plan() {
        return Err(ShellError::MemberMismatch);
    }
    debug_assert!(total_bytes <= MAX_TREE_BYTES);
    Ok((hasher.finalize().to_hex().to_string(), total_bytes))
}

fn native_path(components: &[Vec<u8>]) -> PathBuf {
    components.iter().map(|part| std::ffi::OsStr::from_bytes(part)).collect()
}

pub(crate) fn check_directory_binding(parent: &Dir, name: &Path, opened: &Dir) -> Result<(), ShellError> {
    let named = parent.symlink_metadata(name).map_err(|_| ShellError::MemberUnavailable)?;
    let opened = opened.dir_metadata().map_err(|_| ShellError::MemberUnavailable)?;
    if !named.is_dir() || named.dev() != opened.dev() || named.ino() != opened.ino() {
        return Err(ShellError::MemberMismatch);
    }
    Ok(())
}
