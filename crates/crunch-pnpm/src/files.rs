//! Filesystem admission for the pnpm shell. No acquisition or resolver authority.
use std::fs;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use crunch_pnpm_core::MAX_INPUT_BYTES;
use crunch_source_core::Blake3Digest;
use serde_json::Value;
pub type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

pub fn bytes(path: &Path) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    fs::File::open(path)?.take(MAX_INPUT_BYTES + 1).read_to_end(&mut data)?;
    if data.len() as u64 > MAX_INPUT_BYTES {
        return Err("input byte bound exceeded".into());
    }
    Ok(data)
}
pub fn document(path: &Path) -> Result<Vec<u8>> {
    if fs::metadata(path)?.len() > 2 * 1024 * 1024 {
        return Err("document byte bound exceeded".into());
    }
    bytes(path)
}

pub fn blob(path: &Path) -> Result<Blake3Digest> {
    let mut file = fs::File::open(path)?;
    let mut hash = blake3::Hasher::new();
    let mut buffer = [0_u8; 16 * 1024];
    let mut total = 0_u64;
    loop {
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
    Ok(Blake3Digest::parse(hash.finalize().to_hex().to_string()).map_err(|e| e.to_string())?)
}

/// Canonical tree identity: sorted paths, executable permission and exact bytes.
/// Symlinks bind their exact target bytes; callers must separately admit target authority.
pub fn tree(root: &Path) -> Result<Blake3Digest> {
    let mut hash = blake3::Hasher::new();
    hash.update(b"mantle-pnpm-tree-v1\0");
    let mut count = 0_u32;
    let mut size = 0_u64;
    walk(root, root, &mut hash, &mut count, &mut size)?;
    Ok(Blake3Digest::parse(hash.finalize().to_hex().to_string()).map_err(|e| e.to_string())?)
}
fn walk(root: &Path, path: &Path, hash: &mut blake3::Hasher, count: &mut u32, size: &mut u64) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    *count = count.checked_add(1).ok_or("tree entry overflow")?;
    if *count > 100_000 {
        return Err("tree entry bound exceeded".into());
    }
    let metadata = fs::symlink_metadata(path)?;
    let relative = path.strip_prefix(root)?.to_str().ok_or("non-UTF8 path")?;
    if relative.split('/').count() > 64 {
        return Err("tree depth bound exceeded".into());
    }
    hash.update(&(relative.len() as u64).to_le_bytes());
    hash.update(relative.as_bytes());
    if metadata.is_dir() {
        hash.update(b"directory\0");
        let mut entries = Vec::new();
        for entry in fs::read_dir(path)? {
            if entries.len() >= 100_000_u32.saturating_sub(*count) as usize {
                return Err("tree entry bound exceeded".into());
            }
            entries.push(entry?.path());
        }
        entries.sort();
        for entry in entries {
            walk(root, &entry, hash, count, size)?;
        }
    } else if metadata.is_file() {
        *size = size.checked_add(metadata.len()).ok_or("tree byte overflow")?;
        if *size > MAX_INPUT_BYTES {
            return Err("tree byte bound exceeded".into());
        }
        hash.update(b"file\0");
        hash.update(&(metadata.permissions().mode() & 0o111).to_le_bytes());
        hash.update(&metadata.len().to_le_bytes());
        hash.update_reader(fs::File::open(path)?)?;
    } else if metadata.file_type().is_symlink() {
        let target = fs::read_link(path)?;
        let target = target.to_str().ok_or("non-UTF8 symlink")?;
        hash.update(b"symlink\0");
        hash.update(&(target.len() as u64).to_le_bytes());
        hash.update(target.as_bytes());
    } else {
        return Err(format!("special file prohibited: {}", path.display()).into());
    }
    Ok(())
}
pub fn copy(root: &Path, output: &Path) -> Result<()> {
    copy_admitted(root, output)
}
fn copy_admitted(root: &Path, output: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(root)?;
    if metadata.is_file() {
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(root, output)?;
    } else if metadata.is_dir() {
        fs::create_dir_all(output)?;
        for entry in fs::read_dir(root)? {
            let entry = entry?;
            copy_admitted(&entry.path(), &output.join(entry.file_name()))?;
        }
    } else {
        return Err("input changed to symlink or special file".into());
    }
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
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Err("source/output/mirror symlinks prohibited".into());
    }
    if metadata.is_dir() {
        for entry in fs::read_dir(path)? {
            reject_links(&entry?.path())?;
        }
    }
    Ok(())
}

pub fn tool_links(path: &Path, roots: &[&Path]) -> Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        let resolved = fs::canonicalize(path)?;
        if !roots.iter().any(|root| resolved.starts_with(root)) {
            return Err(format!("tool symlink outside declared roots: {}", path.display()).into());
        }
    } else if metadata.is_dir() {
        for entry in fs::read_dir(path)? {
            tool_links(&entry?.path(), roots)?;
        }
    }
    Ok(())
}
