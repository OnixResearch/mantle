//! Bounded fixture filesystem mechanics; production process observation lives in
//! crunch-rustc-wrapper.
use std::collections::BTreeMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::fs::{self};
use std::io::Read;
use std::io::Write;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Component;
use std::path::Path;

use crunch_rustc_wrapper::observed_process::FILES_MAX;
use crunch_rustc_wrapper::observed_process::private_dir;
use mantle_rust_differential_core::Digest;

use crate::Result;
use crate::err;
use crate::fixtures::Snapshot;

pub const SOURCE_BYTES: u64 = 2 * 1024 * 1024;
pub fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
        .map_err(err)?;
    file.write_all(bytes).map_err(err)?;
    file.sync_all().map_err(err)
}
pub fn read_bounded(path: &Path, bound: u64) -> Result<Vec<u8>> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
        .map_err(err)?;
    let metadata = file.metadata().map_err(err)?;
    if !metadata.is_file() || metadata.len() > bound {
        return Err(format!("file-bound:{}", path.display()));
    }
    let mut bytes = Vec::new();
    file.take(bound + 1).read_to_end(&mut bytes).map_err(err)?;
    if bytes.len() as u64 > bound {
        return Err("file-grew-beyond-bound".into());
    }
    Ok(bytes)
}
fn relative(path: &str) -> Result<&Path> {
    let path = Path::new(path);
    if path.as_os_str().is_empty() || path.components().any(|c| !matches!(c, Component::Normal(_))) {
        return Err("unsafe-snapshot-member".into());
    }
    Ok(path)
}
pub fn materialize(root: &Path, snapshot: &Snapshot) -> Result<()> {
    private_dir(root)?;
    let mut total = 0u64;
    for (name, bytes) in snapshot {
        total = total.checked_add(bytes.len() as u64).ok_or("source-size-overflow")?;
        if total > SOURCE_BYTES || snapshot.len() > 128 {
            return Err("source-bound".into());
        }
        let dest = root.join(relative(name)?);
        let mut current = root.to_path_buf();
        for component in relative(name)?.parent().ok_or("member-parent")?.components() {
            current.push(component);
            if !current.exists() {
                private_dir(&current)?;
            } else if !fs::symlink_metadata(&current).map_err(err)?.is_dir() {
                return Err("snapshot-parent-type".into());
            }
        }
        write_new(&dest, bytes)?;
    }
    verify(root, snapshot)
}
fn inventory_inner(root: &Path, prefix: &Path, files: &mut BTreeMap<String, Vec<u8>>, total: &mut u64) -> Result<()> {
    for entry in fs::read_dir(root.join(prefix)).map_err(err)? {
        let entry = entry.map_err(err)?;
        let p = prefix.join(entry.file_name());
        let m = entry.metadata().map_err(err)?;
        if entry.file_type().map_err(err)?.is_symlink() {
            return Err("snapshot-symlink".into());
        }
        if m.is_dir() {
            inventory_inner(root, &p, files, total)?;
        } else if m.is_file() {
            *total = total.checked_add(m.len()).ok_or("source-overflow")?;
            if *total > SOURCE_BYTES || files.len() >= 128 {
                return Err("source-inventory-bound".into());
            }
            files.insert(p.to_str().ok_or("nonutf8-member")?.into(), read_bounded(&root.join(p), SOURCE_BYTES)?);
        } else {
            return Err("snapshot-special-file".into());
        }
    }
    Ok(())
}
pub fn snapshot(root: &Path) -> Result<Snapshot> {
    let mut s = Snapshot::new();
    inventory_inner(root, Path::new(""), &mut s, &mut 0)?;
    Ok(s)
}
pub fn verify(root: &Path, expected: &Snapshot) -> Result<()> {
    if snapshot(root)? != *expected {
        return Err("snapshot-postimage-mismatch".into());
    }
    Ok(())
}
pub fn apply(root: &Path, before: &Snapshot, step: &crate::fixtures::Step) -> Result<()> {
    verify(root, before)?;
    for edit in &step.edits {
        if edit.matches != 1 || before.get(&edit.path) != edit.before.as_ref() {
            return Err("edit-preimage-or-count".into());
        }
        let path = root.join(relative(&edit.path)?);
        if edit.before.is_some() {
            if read_bounded(&path, SOURCE_BYTES)? != *edit.before.as_ref().ok_or("edit-before")? {
                return Err("edit-missed".into());
            }
            fs::remove_file(&path).map_err(err)?;
        }
        if let Some(bytes) = &edit.after {
            let parent = path.parent().ok_or("edit-parent")?;
            if !parent.exists() {
                private_dir(parent)?;
            }
            write_new(&path, bytes)?;
        }
    }
    verify(root, &step.snapshot)
}
/// Hash complete declared closures, with bounded traversal and explicit symlink bytes.
pub fn hash_closure(root: &Path) -> Result<Digest> {
    fn visit(path: &Path, logical: &Path, h: &mut blake3::Hasher, n: &mut u32, bytes: &mut u64) -> Result<()> {
        *n += 1;
        if *n > 200_000 {
            return Err("tool-closure-file-bound".into());
        }
        let m = fs::symlink_metadata(path).map_err(err)?;
        h.update(logical.as_os_str().as_encoded_bytes());
        h.update(&[0]);
        if m.file_type().is_symlink() {
            h.update(b"link");
            h.update(fs::read_link(path).map_err(err)?.as_os_str().as_encoded_bytes());
            return Ok(());
        }
        if m.is_dir() {
            let mut members = fs::read_dir(path)
                .map_err(err)?
                .map(|e| e.map(|e| e.file_name()))
                .collect::<std::io::Result<Vec<_>>>()
                .map_err(err)?;
            members.sort();
            for name in members {
                visit(&path.join(&name), &logical.join(&name), h, n, bytes)?;
            }
        } else if m.is_file() {
            *bytes = bytes.checked_add(m.len()).ok_or("closure-size")?;
            if *bytes > 16 * 1024 * 1024 * 1024 {
                return Err("tool-closure-byte-bound".into());
            }
            let mut file = File::open(path).map_err(err)?;
            let mut buffer = [0u8; 65536];
            loop {
                let size = file.read(&mut buffer).map_err(err)?;
                if size == 0 {
                    break;
                }
                h.update(&buffer[..size]);
            }
        } else {
            return Err("tool-closure-special-file".into());
        }
        Ok(())
    }
    let mut hasher = blake3::Hasher::new();
    visit(root, Path::new(""), &mut hasher, &mut 0, &mut 0)?;
    Digest::parse(hasher.finalize().to_hex().to_string()).map_err(err)
}
/// Remove only an already-finalized private reference tree, never a retained campaign.
pub fn remove_owned_tree(root: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(root).map_err(err)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() || metadata.uid() != unsafe { libc::geteuid() } {
        return Err("cleanup-root-ownership".into());
    }
    fn remove(path: &Path, count: &mut u32) -> Result<()> {
        for entry in fs::read_dir(path).map_err(err)? {
            let entry = entry.map_err(err)?;
            *count = count.checked_add(1).ok_or("cleanup-overflow")?;
            if *count > FILES_MAX {
                return Err("cleanup-file-bound".into());
            }
            if entry.file_type().map_err(err)?.is_dir() {
                remove(&entry.path(), count)?;
                fs::remove_dir(entry.path()).map_err(err)?;
            } else {
                fs::remove_file(entry.path()).map_err(err)?;
            }
        }
        Ok(())
    }
    remove(root, &mut 0)?;
    fs::remove_dir(root).map_err(err)
}
