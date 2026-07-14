use std::collections::VecDeque;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use crunch_wasm_component_core::Blake3Identity;
use crunch_wasm_component_core::GeneratedInputPlan;
use crunch_wasm_component_core::OciSha256Digest;
use crunch_wasm_component_core::StoreObject;
use sha2::Digest;
use sha2::Sha256;

use crate::Error;

const MAX_TREE_ENTRIES: u32 = 10_000;
const MAX_TREE_DEPTH: u32 = 64;
const MAX_TREE_BYTES: u64 = 536_870_912;
const HASH_BUFFER_CAPACITY_BYTES: usize = 65_536;
const HASH_BUFFER_CAPACITY_BYTES_U64: u64 = 65_536;
const READ_EOF_PROBE_ITERATIONS: u64 = 1;

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
        for entry in read_entries_sorted(&source_dir)? {
            entry_count = entry_count
                .checked_add(1)
                .ok_or_else(|| Error::Invalid("source entry count overflowed u32".to_string()))?;
            if entry_count > MAX_TREE_ENTRIES {
                return Err(Error::Invalid(format!("source tree exceeds {MAX_TREE_ENTRIES} entries")));
            }
            let child_relative = relative.join(entry.file_name());
            validate_relative_path(&child_relative)?;
            let file_type =
                entry.file_type().map_err(|error| Error::io("reading source entry type", &entry.path(), error))?;
            let destination_path = destination.join(&child_relative);
            if file_type.is_symlink() {
                return Err(Error::Invalid(format!("source symlink is not admitted: {}", child_relative.display())));
            }
            if file_type.is_dir() {
                fs::create_dir(&destination_path)
                    .map_err(|error| Error::io("creating source directory", &destination_path, error))?;
                hash_record(&mut hasher, b"dir\0", &child_relative, 0_u64)?;
                queue.push_back((child_relative, depth.saturating_add(1)));
            } else if file_type.is_file() {
                let file_size_bytes = copy_file_hashed(&entry.path(), &destination_path, &child_relative, &mut hasher)?;
                byte_count = byte_count
                    .checked_add(file_size_bytes)
                    .ok_or_else(|| Error::Invalid("source byte count overflowed u64".to_string()))?;
                if byte_count > MAX_TREE_BYTES {
                    return Err(Error::Invalid(format!("source tree exceeds {MAX_TREE_BYTES} bytes")));
                }
            } else {
                return Err(Error::Invalid(format!("unsupported source entry: {}", child_relative.display())));
            }
        }
    }
    let digest = Blake3Identity::parse(blake3::Hasher::finalize(&hasher).to_hex().to_string())
        .map_err(|error| Error::Invalid(format!("encoding source digest: {error}")))?;
    debug_assert!(entry_count <= MAX_TREE_ENTRIES);
    debug_assert!(byte_count <= MAX_TREE_BYTES);
    Ok(StoreObject {
        logical_path: source.display().to_string(),
        digest_blake3: digest,
        size_bytes: byte_count,
    })
}

pub fn materialize_generated_inputs(root: &Path, plan: &GeneratedInputPlan) -> Result<(), Error> {
    for input in &plan.inputs {
        let path = root.join(&input.target);
        let parent = path
            .parent()
            .ok_or_else(|| Error::Invalid(format!("generated input has no parent: {}", input.target)))?;
        fs::create_dir_all(parent).map_err(|error| Error::io("creating generated input parent", parent, error))?;
        write_new(&path, input.content.as_bytes())?;
        let measured = Blake3Identity::from_slice(input.content.as_bytes());
        if measured != input.content_identity_blake3 {
            return Err(Error::Invalid(format!("generated input digest drifted: {}", input.target)));
        }
    }
    debug_assert_eq!(plan.inputs.len(), plan.inputs.iter().filter(|input| root.join(&input.target).is_file()).count());
    debug_assert!(plan.inputs.iter().all(|input| !input.target.is_empty()));
    Ok(())
}

pub fn write_json_new(path: &Path, value: &impl serde::Serialize) -> Result<(), Error> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| Error::Invalid(format!("serializing {}: {error}", path.display())))?;
    write_new(path, &bytes)
}

pub fn sha256_file(path: &Path) -> Result<OciSha256Digest, Error> {
    package_file_measurement(path).map(|measurement| measurement.1)
}

pub(crate) fn read_source_file_bounded(path: &Path, max_bytes: u64, label: &str) -> Result<Vec<u8>, Error> {
    let (mut file, metadata) = open_regular_source(path, max_bytes, label)?;
    let initial_size_bytes = metadata.len();
    let capacity_bytes =
        usize::try_from(initial_size_bytes).map_err(|_| Error::Invalid(format!("{label} size exceeds usize")))?;
    let mut bytes = Vec::with_capacity(capacity_bytes);
    let mut buffer = vec![0_u8; HASH_BUFFER_CAPACITY_BYTES];
    let iteration_count_max = read_iteration_count(initial_size_bytes);
    for _ in 0..iteration_count_max {
        let count_bytes =
            file.read(&mut buffer).map_err(|error| Error::io(&format!("reading {label}"), path, error))?;
        if count_bytes == 0 {
            let final_size_bytes =
                file.metadata().map_err(|error| Error::io(&format!("remeasuring {label}"), path, error))?.len();
            let bytes_read =
                u64::try_from(bytes.len()).map_err(|_| Error::Invalid(format!("{label} bytes read exceeds u64")))?;
            if bytes_read != initial_size_bytes || final_size_bytes != initial_size_bytes {
                return Err(Error::Io(format!("{label} size changed while reading: {}", path.display())));
            }
            debug_assert_eq!(bytes_read, final_size_bytes);
            debug_assert!(bytes_read <= max_bytes);
            return Ok(bytes);
        }
        bytes.extend_from_slice(&buffer[..count_bytes]);
        let bytes_read =
            u64::try_from(bytes.len()).map_err(|_| Error::Invalid(format!("{label} bytes read exceeds u64")))?;
        if bytes_read > max_bytes {
            return Err(Error::Invalid(format!("{label} exceeded {max_bytes} bytes while reading")));
        }
    }
    Err(Error::Io(format!("{label} did not reach EOF within its byte bound: {}", path.display())))
}

fn read_iteration_count(size_bytes: u64) -> u64 {
    size_bytes.div_ceil(HASH_BUFFER_CAPACITY_BYTES_U64).saturating_add(READ_EOF_PROBE_ITERATIONS)
}

pub(crate) fn copy_regular_file_new(source: &Path, destination: &Path, label: &str) -> Result<u64, Error> {
    let (mut input, metadata) = open_regular_source(source, MAX_TREE_BYTES, label)?;
    let mut output = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(destination)
        .map_err(|error| Error::io(&format!("creating {label} copy"), destination, error))?;
    let options = CopyOpenFileOptions {
        initial_size_bytes: metadata.len(),
        source,
        destination,
        label,
    };
    match copy_open_regular_file(&mut input, &mut output, options) {
        Ok(copied_bytes) => {
            debug_assert!(destination.is_file());
            debug_assert!(copied_bytes > 0);
            Ok(copied_bytes)
        }
        Err(original) => {
            drop(output);
            cleanup_partial_copy(destination, original)
        }
    }
}

struct CopyOpenFileOptions<'a> {
    initial_size_bytes: u64,
    source: &'a Path,
    destination: &'a Path,
    label: &'a str,
}

fn copy_open_regular_file(input: &mut File, output: &mut File, options: CopyOpenFileOptions<'_>) -> Result<u64, Error> {
    let mut buffer = vec![0_u8; HASH_BUFFER_CAPACITY_BYTES];
    let mut copied_bytes = 0_u64;
    let iteration_count_max = read_iteration_count(options.initial_size_bytes);
    for _ in 0..iteration_count_max {
        let count_bytes = input
            .read(&mut buffer)
            .map_err(|error| Error::io(&format!("reading {}", options.label), options.source, error))?;
        if count_bytes == 0 {
            let final_size_bytes = input
                .metadata()
                .map_err(|error| Error::io(&format!("remeasuring {}", options.label), options.source, error))?
                .len();
            if copied_bytes != options.initial_size_bytes || final_size_bytes != options.initial_size_bytes {
                return Err(Error::Io(format!(
                    "{} changed while copying: {}",
                    options.label,
                    options.source.display()
                )));
            }
            output
                .sync_all()
                .map_err(|error| Error::io(&format!("syncing {}", options.label), options.destination, error))?;
            debug_assert_eq!(copied_bytes, final_size_bytes);
            debug_assert!(copied_bytes > 0);
            return Ok(copied_bytes);
        }
        output
            .write_all(&buffer[..count_bytes])
            .map_err(|error| Error::io(&format!("writing {}", options.label), options.destination, error))?;
        let count_bytes_u64 = u64::try_from(count_bytes)
            .map_err(|_| Error::Invalid(format!("{} read size exceeds u64", options.label)))?;
        copied_bytes = copied_bytes
            .checked_add(count_bytes_u64)
            .ok_or_else(|| Error::Invalid(format!("{} byte count overflowed u64", options.label)))?;
    }
    Err(Error::Io(format!(
        "{} did not reach EOF within its byte bound: {}",
        options.label,
        options.source.display()
    )))
}

fn cleanup_partial_copy(destination: &Path, original: Error) -> Result<u64, Error> {
    match fs::remove_file(destination) {
        Ok(()) => Err(original),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Err(original),
        Err(error) => Err(Error::Io(format!("{original}; removing partial copy {}: {error}", destination.display()))),
    }
}

pub(crate) fn package_file_measurement(path: &Path) -> Result<(Blake3Identity, OciSha256Digest, u64), Error> {
    let (mut file, metadata) = open_regular_source(path, MAX_TREE_BYTES, "package file")?;
    if metadata.len() == 0 {
        return Err(Error::Invalid(format!("package file is empty: {}", path.display())));
    }
    let mut sha256 = Sha256::new();
    let mut blake3 = blake3::Hasher::new();
    let mut buffer = vec![0_u8; HASH_BUFFER_CAPACITY_BYTES];
    let mut byte_count = 0_u64;
    let iteration_count_max = read_iteration_count(metadata.len());
    let mut has_reached_eof = false;
    for _ in 0..iteration_count_max {
        let count_bytes = file.read(&mut buffer).map_err(|error| Error::io("hashing package file", path, error))?;
        if count_bytes == 0 {
            has_reached_eof = true;
            break;
        }
        let count_bytes_u64 =
            u64::try_from(count_bytes).map_err(|_| Error::Invalid("package read size exceeds u64".to_string()))?;
        byte_count = byte_count
            .checked_add(count_bytes_u64)
            .ok_or_else(|| Error::Invalid("package byte count overflowed u64".to_string()))?;
        if byte_count > MAX_TREE_BYTES {
            return Err(Error::Invalid(format!("package exceeds {MAX_TREE_BYTES} bytes")));
        }
        sha256.update(&buffer[..count_bytes]);
        blake3.update(&buffer[..count_bytes]);
    }
    if !has_reached_eof {
        return Err(Error::Io(format!("package did not reach EOF within its byte bound: {}", path.display())));
    }
    let final_size_bytes = file.metadata().map_err(|error| Error::io("remeasuring package file", path, error))?.len();
    if byte_count != metadata.len() || final_size_bytes != metadata.len() {
        return Err(Error::Io(format!("package file size changed while hashing: {}", path.display())));
    }
    let sha256 = OciSha256Digest::parse(format!("sha256:{:x}", sha256.finalize()))
        .map_err(|error| Error::Invalid(format!("encoding package SHA-256: {error}")))?;
    let blake3 = Blake3Identity::parse(blake3::Hasher::finalize(&blake3).to_hex().to_string())
        .map_err(|error| Error::Invalid(format!("encoding package BLAKE3: {error}")))?;
    debug_assert_eq!(byte_count, final_size_bytes);
    debug_assert!(byte_count > 0);
    Ok((blake3, sha256, byte_count))
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
    let (mut input, metadata) = open_regular_source(source, MAX_TREE_BYTES, "source file")?;
    hash_record(hasher, b"file\0", relative, metadata.len())?;
    let mut output = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(destination)
        .map_err(|error| Error::io("creating source copy", destination, error))?;
    let mut buffer = vec![0_u8; HASH_BUFFER_CAPACITY_BYTES];
    let mut copied_bytes = 0_u64;
    let iteration_count_max = read_iteration_count(metadata.len());
    for _ in 0..iteration_count_max {
        let count_bytes = input.read(&mut buffer).map_err(|error| Error::io("reading source file", source, error))?;
        if count_bytes == 0 {
            let final_size_bytes =
                input.metadata().map_err(|error| Error::io("remeasuring source file", source, error))?.len();
            if copied_bytes != metadata.len() || final_size_bytes != metadata.len() {
                return Err(Error::Io(format!("source changed while copying: {}", source.display())));
            }
            debug_assert_eq!(copied_bytes, metadata.len());
            debug_assert!(destination.is_file());
            return Ok(copied_bytes);
        }
        output
            .write_all(&buffer[..count_bytes])
            .map_err(|error| Error::io("writing source copy", destination, error))?;
        hasher.update(&buffer[..count_bytes]);
        let count_bytes_u64 =
            u64::try_from(count_bytes).map_err(|_| Error::Invalid("source read size exceeds u64".to_string()))?;
        copied_bytes = copied_bytes
            .checked_add(count_bytes_u64)
            .ok_or_else(|| Error::Invalid("copied byte count overflowed u64".to_string()))?;
    }
    Err(Error::Io(format!("source did not reach EOF within its byte bound: {}", source.display())))
}

fn open_regular_source(path: &Path, max_bytes: u64, label: &str) -> Result<(File, fs::Metadata), Error> {
    reject_symlink_components(path)?;
    let link_metadata =
        fs::symlink_metadata(path).map_err(|error| Error::io(&format!("reading {label} metadata"), path, error))?;
    if !link_metadata.file_type().is_file() {
        return Err(Error::Invalid(format!("{label} is not a no-follow regular file: {}", path.display())));
    }
    let mut options = fs::OpenOptions::new();
    options.read(true).custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
    let file = options.open(path).map_err(|error| Error::io(&format!("opening no-follow {label}"), path, error))?;
    let metadata = file
        .metadata()
        .map_err(|error| Error::io(&format!("reading opened {label} metadata"), path, error))?;
    if !metadata.is_file() || metadata.len() > max_bytes {
        return Err(Error::Invalid(format!("{label} exceeds {max_bytes} bytes: {}", path.display())));
    }
    debug_assert!(metadata.is_file());
    debug_assert!(metadata.len() <= max_bytes);
    Ok((file, metadata))
}

fn reject_symlink_components(path: &Path) -> Result<(), Error> {
    let mut cursor = PathBuf::new();
    for component in path.components() {
        match component {
            Component::RootDir | Component::Prefix(_) => cursor.push(component.as_os_str()),
            Component::Normal(part) => {
                cursor.push(part);
                if fs::symlink_metadata(&cursor).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
                    return Err(Error::Invalid(format!(
                        "symlink source component is not admitted: {}",
                        cursor.display()
                    )));
                }
            }
            Component::CurDir | Component::ParentDir => {
                return Err(Error::Invalid(format!("non-canonical source path is not admitted: {}", path.display())));
            }
        }
    }
    debug_assert!(path.components().all(|component| !matches!(component, Component::CurDir | Component::ParentDir)));
    debug_assert!(cursor.as_os_str().is_empty() || !cursor.to_string_lossy().contains("/../"));
    Ok(())
}

fn hash_record(hasher: &mut blake3::Hasher, tag: &[u8], path: &Path, size_bytes: u64) -> Result<(), Error> {
    let path = path.to_string_lossy();
    let path_length_bytes =
        u64::try_from(path.len()).map_err(|_| Error::Invalid("source path length exceeds u64".to_string()))?;
    hasher.update(tag);
    hasher.update(&path_length_bytes.to_le_bytes());
    hasher.update(path.as_bytes());
    hasher.update(&size_bytes.to_le_bytes());
    debug_assert_eq!(path_length_bytes, u64::try_from(path.len()).unwrap_or_default());
    debug_assert!(!tag.is_empty());
    Ok(())
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
    reject_symlink_components(source)?;
    if !source.is_absolute() || !source.is_dir() {
        return Err(Error::Invalid("source copy requires an absolute source directory".to_string()));
    }
    if !destination.is_absolute() || destination.exists() {
        return Err(Error::Invalid("source copy requires a new absolute destination".to_string()));
    }
    if destination.starts_with(source) || source.starts_with(destination) {
        return Err(Error::Invalid("source and destination trees must not contain each other".to_string()));
    }
    debug_assert!(source.is_absolute());
    debug_assert!(destination.is_absolute());
    Ok(())
}

pub fn validate_relative_path(path: &Path) -> Result<(), Error> {
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || !path.components().all(|component| matches!(component, Component::Normal(_)))
    {
        return Err(Error::Invalid(format!("unsafe relative path `{}`", path.display())));
    }
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use std::os::unix::fs::symlink;

    use super::copy_file_hashed;
    use super::read_source_file_bounded;
    use super::sha256_file;

    const TEST_READ_BOUND_BYTES: u64 = 1024;

    #[test]
    fn source_and_package_reads_reject_file_replaced_by_symlink() {
        let dir = tempfile::tempdir().unwrap();
        let outside = dir.path().join("outside");
        std::fs::write(&outside, b"outside").unwrap();
        let source = dir.path().join("source");
        symlink(&outside, &source).unwrap();
        let destination = dir.path().join("destination");
        let mut hasher = blake3::Hasher::new();

        let copy_error = copy_file_hashed(&source, &destination, std::path::Path::new("source"), &mut hasher)
            .expect_err("source replacement symlink must not be followed");
        assert!(copy_error.to_string().contains("symlink source component"));
        assert!(!destination.exists());

        let digest_error = sha256_file(&source).expect_err("package replacement symlink must not be followed");
        assert!(digest_error.to_string().contains("symlink source component"));

        let read_error = read_source_file_bounded(&source, TEST_READ_BOUND_BYTES, "checked lock")
            .expect_err("checked lock replacement symlink must not be followed");
        assert!(read_error.to_string().contains("symlink source component"));
    }
}
