//! Filesystem admission for the pnpm shell. No acquisition or resolver authority.
use std::fs;
use std::io::Read;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;

use crunch_pnpm_core::MAX_INPUT_BYTES;
use crunch_source_core::Blake3Digest;
use serde_json::Value;
pub type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

/// Largest document admitted as a single parsed file.
const DOCUMENT_MAX_BYTES: u64 = 2_097_152;
/// Chunk size of the bounded read loops.
const CHUNK_BYTES: usize = 16 * 1024;
/// Read iterations: the input byte bound over the chunk size, plus the terminating empty read.
const MAX_CHUNKS: u64 = 32_769;
/// Largest number of entries admitted in one tree.
const MAX_TREE_ENTRIES: u32 = 100_000;
/// Largest number of path segments admitted in one tree.
const MAX_TREE_DEPTH: usize = 64;
/// Working-set size of the explicit traversal stacks.
const STACK_CAPACITY: usize = 64;

pub fn bytes(path: &Path) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    fs::File::open(path)?.take(MAX_INPUT_BYTES.saturating_add(1)).read_to_end(&mut data)?;
    if data.len() as u64 > MAX_INPUT_BYTES {
        return Err("input byte bound exceeded".into());
    }
    Ok(data)
}
pub fn document(path: &Path) -> Result<Vec<u8>> {
    if fs::metadata(path)?.len() > DOCUMENT_MAX_BYTES {
        return Err("document byte bound exceeded".into());
    }
    bytes(path)
}

pub fn blob(path: &Path) -> Result<Blake3Digest> {
    let mut file = fs::File::open(path)?;
    let mut hash = blake3::Hasher::new();
    let mut buffer = [0_u8; CHUNK_BYTES];
    let mut total = 0_u64;
    for _ in 0..MAX_CHUNKS {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        total = total.checked_add(count as u64).ok_or("blob byte overflow")?;
        if total > MAX_INPUT_BYTES {
            return Err("blob byte bound exceeded".into());
        }
        hash.update(&buffer[..count]);
    }
    debug_assert!(total <= MAX_INPUT_BYTES);
    Ok(Blake3Digest::parse(hash.finalize().to_hex().to_string()).map_err(|e| e.to_string())?)
}

/// Canonical tree identity: sorted paths, executable permission and exact bytes.
/// Symlinks bind their exact target bytes; callers must separately admit target authority.
pub fn tree(root: &Path) -> Result<Blake3Digest> {
    let mut hash = blake3::Hasher::new();
    hash.update(b"mantle-pnpm-tree-v1\0");
    let mut entry_count = 0_u32;
    let mut size_bytes = 0_u64;
    let mut pending = Vec::with_capacity(STACK_CAPACITY);
    pending.push(root.to_path_buf());
    while let Some(path) = pending.pop() {
        entry_count = entry_count.checked_add(1).ok_or("tree entry overflow")?;
        if entry_count > MAX_TREE_ENTRIES {
            return Err("tree entry bound exceeded".into());
        }
        let metadata = fs::symlink_metadata(&path)?;
        let relative = path.strip_prefix(root)?.to_str().ok_or("non-UTF8 path")?;
        if relative.split('/').count() > MAX_TREE_DEPTH {
            return Err("tree depth bound exceeded".into());
        }
        hash.update(&(relative.len() as u64).to_le_bytes());
        hash.update(relative.as_bytes());
        if metadata.is_dir() {
            hash.update(b"directory\0");
            let remaining = usize::try_from(MAX_TREE_ENTRIES.saturating_sub(entry_count))?;
            let mut entries = Vec::new();
            for entry in fs::read_dir(&path)? {
                if entries.len() >= remaining {
                    return Err("tree entry bound exceeded".into());
                }
                entries.push(entry?.path());
            }
            entries.sort();
            // The stack is reversed so the sorted entries are visited depth first, in order.
            for entry in entries.into_iter().rev() {
                pending.push(entry);
            }
        } else if metadata.is_file() {
            size_bytes = size_bytes.checked_add(metadata.len()).ok_or("tree byte overflow")?;
            if size_bytes > MAX_INPUT_BYTES {
                return Err("tree byte bound exceeded".into());
            }
            hash.update(b"file\0");
            hash.update(&(metadata.permissions().mode() & 0o111).to_le_bytes());
            hash.update(&metadata.len().to_le_bytes());
            hash.update_reader(fs::File::open(&path)?)?;
        } else if metadata.file_type().is_symlink() {
            let target = fs::read_link(&path)?;
            let target = target.to_str().ok_or("non-UTF8 symlink")?;
            hash.update(b"symlink\0");
            hash.update(&(target.len() as u64).to_le_bytes());
            hash.update(target.as_bytes());
        } else {
            return Err(format!("special file prohibited: {}", path.display()).into());
        }
    }
    debug_assert!(entry_count <= MAX_TREE_ENTRIES);
    debug_assert!(size_bytes <= MAX_INPUT_BYTES);
    Ok(Blake3Digest::parse(hash.finalize().to_hex().to_string()).map_err(|e| e.to_string())?)
}
pub fn copy(root: &Path, output: &Path) -> Result<()> {
    copy_admitted(root, output)
}
fn copy_admitted(root: &Path, output: &Path) -> Result<()> {
    let mut pending = Vec::with_capacity(STACK_CAPACITY);
    pending.push((root.to_path_buf(), output.to_path_buf()));
    let mut visited = 0_u64;
    while let Some((source, target)) = pending.pop() {
        let metadata = fs::symlink_metadata(&source)?;
        if metadata.is_file() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&source, &target)?;
        } else if metadata.is_dir() {
            fs::create_dir_all(&target)?;
            let mut entries = Vec::with_capacity(STACK_CAPACITY);
            for entry in fs::read_dir(&source)? {
                entries.push(entry?);
            }
            entries.reverse();
            for entry in entries {
                pending.push((entry.path(), target.join(entry.file_name())));
            }
        } else {
            return Err("input changed to symlink or special file".into());
        }
        visited = visited.saturating_add(1);
    }
    debug_assert!(visited > 0);
    debug_assert!(pending.is_empty());
    Ok(())
}
pub fn json(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&document(path)?)?)
}
pub fn write_json(path: &Path, value: &Value) -> Result<()> {
    let mut file = fs::OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(&serde_json::to_vec_pretty(value)?)?;
    Ok(())
}
pub fn child(root: &Path, member: &str) -> Result<PathBuf> {
    let member = crunch_pnpm_core::member(member.to_owned())?;
    Ok(root.join(member))
}

pub fn reject_links(path: &Path) -> Result<()> {
    let mut pending = Vec::with_capacity(STACK_CAPACITY);
    pending.push(path.to_path_buf());
    while let Some(current) = pending.pop() {
        let metadata = fs::symlink_metadata(&current)?;
        if metadata.file_type().is_symlink() {
            return Err("source/output/mirror symlinks prohibited".into());
        }
        if metadata.is_dir() {
            let mut entries = Vec::with_capacity(STACK_CAPACITY);
            for entry in fs::read_dir(&current)? {
                entries.push(entry?.path());
            }
            entries.reverse();
            pending.extend(entries);
        }
    }
    debug_assert!(pending.is_empty());
    Ok(())
}

pub fn tool_links(path: &Path, roots: &[&Path]) -> Result<()> {
    let mut pending = Vec::with_capacity(STACK_CAPACITY);
    pending.push(path.to_path_buf());
    while let Some(current) = pending.pop() {
        let metadata = fs::symlink_metadata(&current)?;
        if metadata.file_type().is_symlink() {
            let resolved = fs::canonicalize(&current)?;
            if !roots.iter().any(|root| resolved.starts_with(root)) {
                return Err(format!("tool symlink outside declared roots: {}", current.display()).into());
            }
        } else if metadata.is_dir() {
            let mut entries = Vec::with_capacity(STACK_CAPACITY);
            for entry in fs::read_dir(&current)? {
                entries.push(entry?.path());
            }
            entries.reverse();
            pending.extend(entries);
        }
    }
    debug_assert!(pending.is_empty());
    Ok(())
}
