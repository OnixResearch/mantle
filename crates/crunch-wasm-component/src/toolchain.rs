use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use crunch_wasm_component_core::Blake3Identity;
use serde_json::Value;

use crate::Error;
use crate::model::REQUIRED_TOOL_NAMES;
use crate::model::TOOLCHAIN_MANIFEST_SCHEMA;
use crate::model::ToolRecord;
use crate::model::ToolchainManifest;
use crate::model::VerifiedToolchain;

const HASH_BUFFER_CAPACITY_BYTES: usize = 64 * 1024;
const MAX_TOOL_BINARY_BYTES: u64 = 512 * 1024 * 1024;
const MAX_VERSION_OUTPUT_BYTES: usize = 16 * 1024;

pub fn verify_toolchain_manifest(path: &Path) -> Result<VerifiedToolchain, Error> {
    let bytes = fs::read(path).map_err(|error| Error::io("reading toolchain manifest", path, error))?;
    let manifest: ToolchainManifest = serde_json::from_slice(&bytes)
        .map_err(|error| Error::Invalid(format!("parsing toolchain manifest {}: {error}", path.display())))?;
    validate_manifest_shape(&manifest)?;
    verify_cohort_identity(&bytes, &manifest)?;
    let root = toolchain_root(path)?;
    verify_tools(&root, &manifest.tools)?;
    debug_assert_eq!(manifest.tools.len(), REQUIRED_TOOL_NAMES.len());
    debug_assert!(root.is_absolute());
    Ok(VerifiedToolchain { root, manifest })
}

impl VerifiedToolchain {
    pub fn tool_path(&self, name: &str) -> Result<PathBuf, Error> {
        let record = self
            .manifest
            .tools
            .iter()
            .find(|record| record.name == name)
            .ok_or_else(|| Error::Invalid(format!("toolchain manifest is missing `{name}`")))?;
        let path = self.root.join(&record.path);
        if !path.is_file() {
            return Err(Error::Invalid(format!("tool `{name}` is missing at {}", path.display())));
        }
        debug_assert!(path.is_absolute());
        debug_assert!(!record.path.is_empty());
        Ok(path)
    }

    pub fn tool_digest(&self, name: &str) -> Result<Blake3Identity, Error> {
        self.manifest
            .tools
            .iter()
            .find(|record| record.name == name)
            .map(|record| record.binary_digest_blake3.clone())
            .ok_or_else(|| Error::Invalid(format!("toolchain manifest is missing digest for `{name}`")))
    }
}

fn validate_manifest_shape(manifest: &ToolchainManifest) -> Result<(), Error> {
    if manifest.schema != TOOLCHAIN_MANIFEST_SCHEMA {
        return Err(Error::Invalid(format!("unsupported toolchain schema `{}`", manifest.schema)));
    }
    if manifest.rust_target != "wasm32-wasip2" {
        return Err(Error::Invalid(format!("unsupported Rust component target `{}`", manifest.rust_target)));
    }
    if manifest.tools.len() != REQUIRED_TOOL_NAMES.len() {
        return Err(Error::Invalid(format!(
            "toolchain contains {} tools, expected {}",
            manifest.tools.len(),
            REQUIRED_TOOL_NAMES.len()
        )));
    }
    let records = tool_record_index(&manifest.tools)?;
    for required in REQUIRED_TOOL_NAMES {
        if !records.contains_key(required) {
            return Err(Error::Invalid(format!("toolchain is missing required tool `{required}`")));
        }
    }
    debug_assert_eq!(records.len(), REQUIRED_TOOL_NAMES.len());
    debug_assert!(REQUIRED_TOOL_NAMES.iter().all(|name| records.contains_key(*name)));
    Ok(())
}

fn tool_record_index<'a>(tools: &'a [ToolRecord]) -> Result<BTreeMap<&'a str, &'a ToolRecord>, Error> {
    let mut records = BTreeMap::new();
    for record in tools {
        if record.name.is_empty() || record.version.is_empty() || record.version_output.is_empty() {
            return Err(Error::Invalid("tool records require name, version, and version output".to_string()));
        }
        if !safe_relative_path(Path::new(&record.path)) {
            return Err(Error::Invalid(format!("tool `{}` has unsafe relative path `{}`", record.name, record.path)));
        }
        if records.insert(record.name.as_str(), record).is_some() {
            return Err(Error::Invalid(format!("tool `{}` is duplicated", record.name)));
        }
    }
    Ok(records)
}

fn verify_cohort_identity(bytes: &[u8], manifest: &ToolchainManifest) -> Result<(), Error> {
    let mut value: Value = serde_json::from_slice(bytes)
        .map_err(|error| Error::Invalid(format!("parsing toolchain identity input: {error}")))?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| Error::Invalid("toolchain manifest must be a JSON object".to_string()))?;
    let removed = object.remove("cohort_identity_blake3");
    if removed.is_none() {
        return Err(Error::Invalid("toolchain manifest omits cohort identity".to_string()));
    }
    let canonical = serde_json::to_vec(&value)
        .map_err(|error| Error::Invalid(format!("serializing toolchain identity input: {error}")))?;
    let measured = Blake3Identity::from_slice(&canonical);
    if measured != manifest.cohort_identity_blake3 {
        return Err(Error::Invalid("toolchain cohort identity does not match canonical manifest fields".to_string()));
    }
    debug_assert!(!canonical.is_empty());
    debug_assert_eq!(measured, manifest.cohort_identity_blake3);
    Ok(())
}

fn verify_tools(root: &Path, tools: &[ToolRecord]) -> Result<(), Error> {
    for record in tools {
        let path = root.join(&record.path);
        let measured = hash_file_bounded(&path)?;
        if measured != record.binary_digest_blake3 {
            return Err(Error::Invalid(format!("tool `{}` binary digest drifted", record.name)));
        }
        let version_output = read_version_output(&path)?;
        if version_output != record.version_output || !version_output.contains(&record.version) {
            return Err(Error::Invalid(format!(
                "tool `{}` version drifted: expected `{}`, observed `{version_output}`",
                record.name, record.version_output
            )));
        }
    }
    debug_assert_eq!(tools.len(), REQUIRED_TOOL_NAMES.len());
    debug_assert!(tools.iter().all(|record| !record.binary_digest_blake3.clone().into_hex().is_empty()));
    Ok(())
}

pub(crate) fn hash_file_bounded(path: &Path) -> Result<Blake3Identity, Error> {
    let metadata = fs::metadata(path).map_err(|error| Error::io("reading file metadata", path, error))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > MAX_TOOL_BINARY_BYTES {
        return Err(Error::Invalid(format!("file {} has unsupported size {}", path.display(), metadata.len())));
    }
    let mut file = File::open(path).map_err(|error| Error::io("opening file for hashing", path, error))?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = vec![0_u8; HASH_BUFFER_CAPACITY_BYTES];
    let mut bytes_read = 0_u64;
    loop {
        let count = file.read(&mut buffer).map_err(|error| Error::io("hashing file", path, error))?;
        if count == 0 {
            break;
        }
        let count_u64 =
            u64::try_from(count).map_err(|_| Error::Invalid("hash read count overflowed u64".to_string()))?;
        bytes_read = bytes_read
            .checked_add(count_u64)
            .ok_or_else(|| Error::Invalid("hashed byte count overflowed u64".to_string()))?;
        if bytes_read > MAX_TOOL_BINARY_BYTES {
            return Err(Error::Invalid(format!("file {} exceeded the hashing bound", path.display())));
        }
        hasher.update(&buffer[..count]);
    }
    debug_assert_eq!(bytes_read, metadata.len());
    debug_assert!(bytes_read > 0);
    Blake3Identity::parse(hasher.finalize().to_hex().to_string())
        .map_err(|error| Error::Invalid(format!("encoding BLAKE3 for {}: {error}", path.display())))
}

fn read_version_output(path: &Path) -> Result<String, Error> {
    let output = Command::new(path)
        .arg("--version")
        .env_clear()
        .output()
        .map_err(|error| Error::io("executing tool version probe", path, error))?;
    if !output.status.success() {
        return Err(Error::Invalid(format!("tool version probe failed for {}", path.display())));
    }
    let mut bytes = output.stdout;
    bytes.extend_from_slice(&output.stderr);
    if bytes.len() > MAX_VERSION_OUTPUT_BYTES {
        return Err(Error::Invalid(format!("tool version output exceeded bound for {}", path.display())));
    }
    let output = String::from_utf8(bytes).map_err(|error| {
        Error::Invalid(format!("tool version output was not UTF-8 for {}: {error}", path.display()))
    })?;
    let first_line = output.lines().next().unwrap_or_default().trim().to_string();
    if first_line.is_empty() {
        return Err(Error::Invalid(format!("tool version output was empty for {}", path.display())));
    }
    debug_assert!(!first_line.contains('\n'));
    debug_assert!(first_line.len() <= MAX_VERSION_OUTPUT_BYTES);
    Ok(first_line)
}

fn toolchain_root(manifest_path: &Path) -> Result<PathBuf, Error> {
    let root = manifest_path.parent().and_then(Path::parent).and_then(Path::parent).ok_or_else(|| {
        Error::Invalid(format!("toolchain manifest path is too shallow: {}", manifest_path.display()))
    })?;
    if !root.is_absolute() {
        return Err(Error::Invalid("toolchain manifest must use an absolute path".to_string()));
    }
    Ok(root.to_path_buf())
}

fn safe_relative_path(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && !path.is_absolute()
        && path.components().all(|component| matches!(component, Component::Normal(_)))
}
