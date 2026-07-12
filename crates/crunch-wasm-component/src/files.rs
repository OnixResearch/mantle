use std::collections::VecDeque;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

use crunch_wasm_component_core::{Blake3Identity, GeneratedInputPlan, OciSha256Digest, StoreObject};
use sha2::{Digest, Sha256};

use crate::Error;

const MAX_TREE_ENTRIES: u32 = 10_000;
const MAX_TREE_DEPTH: u32 = 64;
const MAX_TREE_BYTES: u64 = 512 * 1024 * 1024;
const HASH_BUFFER_CAPACITY_BYTES: usize = 64 * 1024;

pub fn copy_source_tree(source: &Path, destination: &Path) -> Result<StoreObject, Error> {
    validate_source_root(source, destination)?;
    fs::create_dir(destination).map_err(|error| Error::io("creating source stage", destination, error))?;
    let mut queue = VecDeque::from([(PathBuf::new(), 0_u32)]);
    let mut entry_count = 0_u32;
    let mut byte_count = 0_u64;
    let mut hasher = blake3::Hasher::new();
    while let Some((relative, depth)) = queue.pop_front() {
        if depth > MAX_TREE_DEPTH {
            return Err(Error::Invalid(format!("source tree exceeds depth {MAX_TREE_DEPTH}")));
        }
        let source_dir = source.join(&relative);
        let destination_dir = destination.join(&relative);
        for entry in read_entries_sorted(&source_dir)? {
            entry_count = entry_count.checked_add(1)
                .ok_or_else(|| Error::Invalid("source entry count overflowed u32".to_string()))?;
            if entry_count > MAX_TREE_ENTRIES {
                return Err(Error::Invalid(format!("source tree exceeds {MAX_TREE_ENTRIES} entries")));
            }
            let child_relative = relative.join(entry.file_name());
            validate_relative_path(&child_relative)?;
            let file_type = entry.file_type()
                .map_err(|error| Error::io("reading source entry type", &entry.path(), error))?;
            let destination_path = destination.join(&child_relative);
            if file_type.is_symlink() {
                return Err(Error::Invalid(format!("source symlink is not admitted: {}", child_relative.display())));
            }
            if file_type.is_dir() {
                fs::create_dir(&destination_path)
                    .map_err(|error| Error::io("creating source directory", &destination_path, error))?;
                hash_record(&mut hasher, b"dir\0", &child_relative, 0_u64);
                queue.push_back((child_relative, depth.saturating_add(1)));
            } else if file_type.is_file() {
                let file_size = copy_file_hashed(&entry.path(), &destination_path, &child_relative, &mut hasher)?;
                byte_count = byte_count.checked_add(file_size)
                    .ok_or_else(|| Error::Invalid("source byte count overflowed u64".to_string()))?;
                if byte_count > MAX_TREE_BYTES {
                    return Err(Error::Invalid(format!("source tree exceeds {MAX_TREE_BYTES} bytes")));
                }
            } else {
                return Err(Error::Invalid(format!("unsupported source entry: {}", child_relative.display())));
            }
        }
    }
    let digest = Blake3Identity::parse(hasher.finalize().to_hex().to_string())
        .map_err(|error| Error::Invalid(format!("encoding source digest: {error}")))?;
    debug_assert!(entry_count <= MAX_TREE_ENTRIES);
    debug_assert!(byte_count <= MAX_TREE_BYTES);
    Ok(StoreObject {
        logical_path: source.display().to_string(),
        digest_blake3: digest,
    })
}

pub fn materialize_generated_inputs(root: &Path, plans: &[GeneratedInputPlan]) -> Result<(), Error> {
    for plan in plans {
        let path = root.join(&plan.relative_path);
        let parent = path.parent()
            .ok_or_else(|| Error::Invalid(format!("generated input has no parent: {}", plan.relative_path)))?;
        fs::create_dir_all(parent).map_err(|error| Error::io("creating generated input parent", parent, error))?;
        write_new(&path, plan.content.as_bytes())?;
        let measured = Blake3Identity::from_slice(plan.content.as_bytes());
        if measured != plan.digest_blake3 {
            return Err(Error::Invalid(format!("generated input digest drifted: {}", plan.relative_path)));
        }
    }
    debug_assert_eq!(plans.len(), plans.iter().filter(|plan| root.join(&plan.relative_path).is_file()).count());
    debug_assert!(plans.iter().all(|plan| !plan.relative_path.is_empty()));
    Ok(())
}

pub fn write_json_new(path: &Path, value: &impl serde::Serialize) -> Result<(), Error> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| Error::Invalid(format!("serializing {}: {error}", path.display())))?;
    write_new(path, &bytes)
}

pub fn sha256_file(path: &Path) -> Result<OciSha256Digest, Error> {
    let metadata = fs::metadata(path).map_err(|error| Error::io("reading package metadata", path, error))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > MAX_TREE_BYTES {
        return Err(Error::Invalid(format!("package file has unsupported size: {}", path.display())));
    }
    let mut file = File::open(path).map_err(|error| Error::io("opening package file", path, error))?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; HASH_BUFFER_CAPACITY_BYTES];
    let mut byte_count = 0_u64;
    loop {
        let count = file.read(&mut buffer).map_err(|error| Error::io("hashing package file", path, error))?;
        if count == 0 { break; }
        byte_count = byte_count.checked_add(u64::try_from(count).unwrap_or(u64::MAX))
            .ok_or_else(|| Error::Invalid("package byte count overflowed u64".to_string()))?;
        if byte_count > MAX_TREE_BYTES {
            return Err(Error::Invalid(format!("package exceeds {MAX_TREE_BYTES} bytes")));
        }
        hasher.update(&buffer[..count]);
    }
    let hex = format!("sha256:{:x}", hasher.finalize());
    OciSha256Digest::parse(hex).map_err(|error| Error::Invalid(format!("encoding package SHA-256: {error}")))
}

pub fn write_new(path: &Path, bytes: &[u8]) -> Result<(), Error> {
    let mut options = fs::OpenOptions::new();
    options.create_new(true).write(true);
    let mut file = options.open(path).map_err(|error| Error::io("creating file", path, error))?;
    file.write_all(bytes).map_err(|error| Error::io("writing file", path, error))?;
    file.sync_all().map_err(|error| Error::io("syncing file", path, error))?;
    debug_assert!(path.is_file());
    debug_assert_eq!(fs::metadata(path).map(|metadata| metadata.len()).ok(), u64::try_from(bytes.len()).ok());
    Ok(())
}

fn copy_file_hashed(
    source: &Path,
    destination: &Path,
    relative: &Path,
    hasher: &mut blake3::Hasher,
) -> Result<u64, Error> {
    let metadata = fs::metadata(source).map_err(|error| Error::io("reading source file metadata", source, error))?;
    if metadata.len() > MAX_TREE_BYTES {
        return Err(Error::Invalid(format!("source file exceeds {MAX_TREE_BYTES} bytes: {}", relative.display())));
    }
    hash_record(hasher, b"file\0", relative, metadata.len());
    let mut input = File::open(source).map_err(|error| Error::io("opening source file", source, error))?;
    let mut output = fs::OpenOptions::new().create_new(true).write(true).open(destination)
        .map_err(|error| Error::io("creating source copy", destination, error))?;
    let mut buffer = vec![0_u8; HASH_BUFFER_CAPACITY_BYTES];
    let mut copied = 0_u64;
    loop {
        let count = input.read(&mut buffer).map_err(|error| Error::io("reading source file", source, error))?;
        if count == 0 { break; }
        output.write_all(&buffer[..count]).map_err(|error| Error::io("writing source copy", destination, error))?;
        hasher.update(&buffer[..count]);
        copied = copied.checked_add(u64::try_from(count).unwrap_or(u64::MAX))
            .ok_or_else(|| Error::Invalid("copied byte count overflowed u64".to_string()))?;
    }
    if copied != metadata.len() {
        return Err(Error::Io(format!("source changed while copying: {}", source.display())));
    }
    debug_assert_eq!(copied, metadata.len());
    debug_assert!(destination.is_file());
    Ok(copied)
}

fn hash_record(hasher: &mut blake3::Hasher, tag: &[u8], path: &Path, size: u64) {
    let path = path.to_string_lossy();
    let path_len = u64::try_from(path.len()).unwrap_or(u64::MAX);
    hasher.update(tag);
    hasher.update(&path_len.to_le_bytes());
    hasher.update(path.as_bytes());
    hasher.update(&size.to_le_bytes());
}

fn read_entries_sorted(path: &Path) -> Result<Vec<fs::DirEntry>, Error> {
    let mut entries = fs::read_dir(path)
        .map_err(|error| Error::io("reading source directory", path, error))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| Error::io("reading source entry", path, error))?;
    entries.sort_by_key(fs::DirEntry::file_name);
    Ok(entries)
}

fn validate_source_root(source: &Path, destination: &Path) -> Result<(), Error> {
    if !source.is_absolute() || !source.is_dir() || !destination.is_absolute() || destination.exists() {
        return Err(Error::Invalid("source copy requires an absolute directory and a new absolute destination".to_string()));
    }
    if destination.starts_with(source) || source.starts_with(destination) {
        return Err(Error::Invalid("source and destination trees must not contain each other".to_string()));
    }
    Ok(())
}

pub fn validate_relative_path(path: &Path) -> Result<(), Error> {
    if path.as_os_str().is_empty() || path.is_absolute()
        || !path.components().all(|component| matches!(component, Component::Normal(_)))
    {
        return Err(Error::Invalid(format!("unsafe relative path `{}`", path.display())));
    }
    Ok(())
}
