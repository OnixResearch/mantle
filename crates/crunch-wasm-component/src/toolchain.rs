use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use crunch_wasm_component_core::Blake3Identity;
use serde_json::Value;

use crate::Error;
use crate::model::OCTET_CONFIG_BLAKE3;
use crate::model::OCTET_PACKAGE_NAME;
use crate::model::OCTET_PACKAGE_VERSION;
use crate::model::OCTET_PROFILE_ID;
use crate::model::OCTET_PROFILE_IDENTITY_BLAKE3;
use crate::model::OCTET_SOURCE_REPOSITORY;
use crate::model::OCTET_SOURCE_REVISION;
use crate::model::OCTET_WASM_TOOLS_COHORT_BLAKE3;
use crate::model::REQUIRED_TOOL_NAMES;
use crate::model::TOOLCHAIN_MANIFEST_SCHEMA;
use crate::model::ToolRecord;
use crate::model::ToolchainManifest;
use crate::model::VerifiedToolchain;
use crate::process::ToolLimits;
use crate::process::configure_process_group;
use crate::process::join_drain;
use crate::process::spawn_drain;
use crate::process::wait_bounded;

const HASH_BUFFER_CAPACITY_BYTES: usize = 64 * 1024;
const MAX_TOOL_BINARY_BYTES: u64 = 536_870_912;
const MAX_TOOLCHAIN_MANIFEST_BYTES: u64 = 1024 * 1024;
const MAX_VERSION_OUTPUT_BYTES: usize = 16_384;
const MAX_VERSION_OUTPUT_BYTES_U64: u64 = 16_384;
const VERSION_PROBE_TIMEOUT_MS: u64 = 10 * 1000;
const COHORT_CANONICAL_TERMINATOR: u8 = b'\n';

pub fn verify_toolchain_manifest(path: &Path) -> Result<VerifiedToolchain, Error> {
    let bytes = read_regular_file_bounded(path, MAX_TOOLCHAIN_MANIFEST_BYTES, "toolchain manifest")?;
    let manifest: ToolchainManifest = serde_json::from_slice(&bytes)
        .map_err(|error| Error::Invalid(format!("parsing toolchain manifest {}: {error}", path.display())))?;
    validate_manifest_shape(&manifest)?;
    verify_cohort_identity(&bytes, &manifest)?;
    let root = toolchain_root(path)?;
    verify_tools(&root, &manifest.tools)?;
    verify_octet_identity(&root, &manifest)?;
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
        reject_symlink_components(&path)?;
        let metadata = fs::symlink_metadata(&path).map_err(|error| Error::io("reading tool metadata", &path, error))?;
        if !metadata.file_type().is_file() {
            return Err(Error::Invalid(format!("tool `{name}` is not a no-follow regular file at {}", path.display())));
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

    pub fn octet_config_path(&self) -> Result<PathBuf, Error> {
        let path = self.root.join(&self.manifest.octet.config_path);
        reject_symlink_components(&path)?;
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| Error::io("reading Octet profile configuration metadata", &path, error))?;
        if !metadata.file_type().is_file() {
            return Err(Error::Invalid(format!(
                "Octet profile configuration is not a no-follow regular file: {}",
                path.display()
            )));
        }
        debug_assert!(path.is_absolute());
        debug_assert!(metadata.len() > 0);
        Ok(path)
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
    if !safe_relative_path(Path::new(&manifest.octet.config_path)) {
        return Err(Error::Invalid(format!(
            "Octet profile configuration has unsafe relative path `{}`",
            manifest.octet.config_path
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

fn tool_record_index(tools: &[ToolRecord]) -> Result<BTreeMap<&str, &ToolRecord>, Error> {
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
    if object.remove("cohort_identity_blake3").is_none() {
        return Err(Error::Invalid("toolchain manifest omits cohort identity".to_string()));
    }
    let mut canonical = serde_json::to_vec(&value)
        .map_err(|error| Error::Invalid(format!("serializing toolchain identity input: {error}")))?;
    canonical.push(COHORT_CANONICAL_TERMINATOR);
    let measured = Blake3Identity::from_slice(&canonical);
    if measured != manifest.cohort_identity_blake3 {
        return Err(Error::Invalid("toolchain cohort identity does not match canonical manifest fields".to_string()));
    }
    debug_assert!(!canonical.is_empty());
    debug_assert_eq!(measured, manifest.cohort_identity_blake3);
    Ok(())
}

fn verify_octet_identity(root: &Path, manifest: &ToolchainManifest) -> Result<(), Error> {
    let octet = &manifest.octet;
    let expected_cohort = pinned_identity(OCTET_WASM_TOOLS_COHORT_BLAKE3, OctetPinnedRole::Cohort)?;
    let expected_config = pinned_identity(OCTET_CONFIG_BLAKE3, OctetPinnedRole::Configuration)?;
    let expected_profile = pinned_identity(OCTET_PROFILE_IDENTITY_BLAKE3, OctetPinnedRole::Profile)?;
    if octet.source_repository != OCTET_SOURCE_REPOSITORY {
        return Err(octet_identity_drift());
    }
    if octet.source_revision != OCTET_SOURCE_REVISION {
        return Err(octet_identity_drift());
    }
    if octet.package_name != OCTET_PACKAGE_NAME || octet.package_version != OCTET_PACKAGE_VERSION {
        return Err(octet_identity_drift());
    }
    if octet.profile_id != OCTET_PROFILE_ID {
        return Err(octet_identity_drift());
    }
    if octet.config_digest_blake3 != expected_config || octet.profile_identity_blake3 != expected_profile {
        return Err(octet_identity_drift());
    }
    if octet.wasm_tools_cohort_identity_blake3 != expected_cohort {
        return Err(octet_identity_drift());
    }
    let config_path = root.join(&octet.config_path);
    if hash_file_bounded(&config_path)? != octet.config_digest_blake3 {
        return Err(Error::Invalid("Octet Wasm artifact profile configuration digest drifted".to_string()));
    }
    debug_assert_eq!(octet.source_revision.len(), OCTET_SOURCE_REVISION.len());
    debug_assert_eq!(octet.profile_id, OCTET_PROFILE_ID);
    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum OctetPinnedRole {
    Cohort,
    Configuration,
    Profile,
}

impl OctetPinnedRole {
    const fn label(self) -> &'static str {
        match self {
            Self::Cohort => "cohort",
            Self::Configuration => "configuration",
            Self::Profile => "profile",
        }
    }
}

fn octet_identity_drift() -> Error {
    Error::Invalid("Octet source revision, package, profile, or wasm-tools cohort identity drifted".to_string())
}

fn pinned_identity(value: &str, role: OctetPinnedRole) -> Result<Blake3Identity, Error> {
    let label = role.label();
    let identity = Blake3Identity::parse(value.to_string())
        .map_err(|error| Error::Invalid(format!("parsing pinned Octet {label} identity: {error}")))?;
    debug_assert_eq!(identity.clone().into_hex().len(), crunch_wasm_component_core::BLAKE3_HEX_LENGTH);
    debug_assert!(!label.is_empty());
    Ok(identity)
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
    reject_symlink_components(path)?;
    let link_metadata =
        fs::symlink_metadata(path).map_err(|error| Error::io("reading no-follow file metadata", path, error))?;
    if !link_metadata.file_type().is_file() {
        return Err(Error::Invalid(format!("file {} is not a no-follow regular file", path.display())));
    }
    let mut options = fs::OpenOptions::new();
    options.read(true).custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
    let mut file = options.open(path).map_err(|error| Error::io("opening no-follow file for hashing", path, error))?;
    let metadata = file.metadata().map_err(|error| Error::io("reading opened file metadata", path, error))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > MAX_TOOL_BINARY_BYTES {
        return Err(Error::Invalid(format!("file {} has unsupported size {}", path.display(), metadata.len())));
    }
    let mut hasher = blake3::Hasher::new();
    let mut buffer = vec![0_u8; HASH_BUFFER_CAPACITY_BYTES];
    let mut bytes_read = 0_u64;
    while bytes_read < metadata.len() {
        let count = file.read(&mut buffer).map_err(|error| Error::io("hashing file", path, error))?;
        if count == 0 {
            break;
        }
        let count_u64 =
            u64::try_from(count).map_err(|_| Error::Invalid("hash read count overflowed u64".to_string()))?;
        bytes_read = bytes_read
            .checked_add(count_u64)
            .ok_or_else(|| Error::Invalid("hashed byte count overflowed u64".to_string()))?;
        if bytes_read > metadata.len() {
            return Err(Error::Invalid(format!("file {} exceeded its measured size", path.display())));
        }
        hasher.update(&buffer[..count]);
    }
    let final_size_bytes = file.metadata().map_err(|error| Error::io("remeasuring hashed file", path, error))?.len();
    if bytes_read != metadata.len() || final_size_bytes != metadata.len() {
        return Err(Error::Io(format!("file size changed while hashing: {}", path.display())));
    }
    debug_assert_eq!(bytes_read, final_size_bytes);
    debug_assert!(bytes_read > 0);
    Blake3Identity::parse(hasher.finalize().to_hex().to_string())
        .map_err(|error| Error::Invalid(format!("encoding BLAKE3 for {}: {error}", path.display())))
}

fn read_regular_file_bounded(path: &Path, max_bytes: u64, label: &str) -> Result<Vec<u8>, Error> {
    reject_symlink_components(path)?;
    let link_metadata =
        fs::symlink_metadata(path).map_err(|error| Error::io(&format!("reading {label} metadata"), path, error))?;
    if !link_metadata.file_type().is_file() || link_metadata.len() == 0 || link_metadata.len() > max_bytes {
        return Err(Error::Invalid(format!("{label} is not a bounded no-follow regular file: {}", path.display())));
    }
    let mut options = fs::OpenOptions::new();
    options.read(true).custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
    let mut file = options.open(path).map_err(|error| Error::io(&format!("opening {label} no-follow"), path, error))?;
    let initial_size_bytes = file
        .metadata()
        .map_err(|error| Error::io(&format!("reading opened {label} metadata"), path, error))?
        .len();
    if initial_size_bytes == 0 || initial_size_bytes > max_bytes {
        return Err(Error::Invalid(format!("{label} exceeds its byte bound: {}", path.display())));
    }
    let capacity_bytes =
        usize::try_from(initial_size_bytes).map_err(|_| Error::Invalid(format!("{label} size exceeds usize")))?;
    let mut bytes = Vec::with_capacity(capacity_bytes);
    let mut buffer = vec![0_u8; HASH_BUFFER_CAPACITY_BYTES];
    let mut bytes_read = 0_u64;
    while bytes_read < initial_size_bytes {
        let count = file
            .read(&mut buffer)
            .map_err(|error| Error::io(&format!("reading bounded {label}"), path, error))?;
        if count == 0 {
            break;
        }
        let count_u64 = u64::try_from(count).map_err(|_| Error::Invalid(format!("{label} read count exceeds u64")))?;
        bytes_read = bytes_read
            .checked_add(count_u64)
            .ok_or_else(|| Error::Invalid(format!("{label} byte count overflowed u64")))?;
        if bytes_read > max_bytes {
            return Err(Error::Invalid(format!("{label} exceeded its byte bound while reading: {}", path.display())));
        }
        bytes.extend_from_slice(&buffer[..count]);
    }
    let final_size_bytes =
        file.metadata().map_err(|error| Error::io(&format!("remeasuring {label}"), path, error))?.len();
    if bytes_read != initial_size_bytes || final_size_bytes != initial_size_bytes {
        return Err(Error::Io(format!("{label} size changed while reading: {}", path.display())));
    }
    debug_assert_eq!(bytes_read, final_size_bytes);
    debug_assert!(bytes_read <= max_bytes);
    Ok(bytes)
}

fn read_version_output(path: &Path) -> Result<String, Error> {
    read_version_output_with_limits(path, ToolLimits {
        timeout_ms: VERSION_PROBE_TIMEOUT_MS,
        output_bytes: MAX_VERSION_OUTPUT_BYTES_U64,
    })
}

fn read_version_output_with_limits(path: &Path, limits: ToolLimits) -> Result<String, Error> {
    reject_symlink_components(path)?;
    let metadata =
        fs::symlink_metadata(path).map_err(|error| Error::io("reading version probe metadata", path, error))?;
    if !metadata.file_type().is_file() {
        return Err(Error::Invalid(format!("tool version probe is not a no-follow regular file: {}", path.display())));
    }
    let mut command = Command::new(path);
    command.arg("--version").env_clear().stdout(Stdio::piped()).stderr(Stdio::piped());
    configure_process_group(&mut command)?;
    let mut child = command.spawn().map_err(|error| Error::io("executing bounded tool version probe", path, error))?;
    let stdout = child.stdout.take().ok_or_else(|| Error::Tool("version stdout pipe unavailable".to_string()))?;
    let stderr = child.stderr.take().ok_or_else(|| Error::Tool("version stderr pipe unavailable".to_string()))?;
    let did_exceed_output_bound = Arc::new(AtomicBool::new(false));
    let output_bytes_observed = Arc::new(AtomicU64::new(0));
    let stdout_thread = spawn_drain(
        stdout,
        limits.output_bytes,
        Arc::clone(&did_exceed_output_bound),
        Arc::clone(&output_bytes_observed),
    );
    let stderr_thread = spawn_drain(
        stderr,
        limits.output_bytes,
        Arc::clone(&did_exceed_output_bound),
        Arc::clone(&output_bytes_observed),
    );
    let mut outcome = wait_bounded(&mut child, "tool-version", limits.timeout_ms, &did_exceed_output_bound)?;
    let mut bytes = join_drain(stdout_thread, "version stdout")?;
    let stderr = join_drain(stderr_thread, "version stderr")?;
    if did_exceed_output_bound.load(Ordering::Acquire) {
        outcome.failure = Some("output-limit-exceeded");
    }
    if let Some(failure) = outcome.failure {
        return Err(Error::Invalid(format!("tool version probe {failure} for {}", path.display())));
    }
    if !outcome.status.success() {
        return Err(Error::Invalid(format!("tool version probe failed for {}", path.display())));
    }
    bytes.extend_from_slice(&stderr);
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

fn reject_symlink_components(path: &Path) -> Result<(), Error> {
    let mut cursor = PathBuf::new();
    for component in path.components() {
        match component {
            Component::RootDir | Component::Prefix(_) => cursor.push(component.as_os_str()),
            Component::Normal(part) => {
                cursor.push(part);
                if fs::symlink_metadata(&cursor).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
                    return Err(Error::Invalid(format!(
                        "symlink file component is not admitted: {}",
                        cursor.display()
                    )));
                }
            }
            Component::CurDir | Component::ParentDir => {
                return Err(Error::Invalid(format!(
                    "non-canonical toolchain path is not admitted: {}",
                    path.display()
                )));
            }
        }
    }
    debug_assert!(!path.as_os_str().is_empty());
    debug_assert!(cursor.components().count() <= path.components().count());
    Ok(())
}

fn safe_relative_path(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && !path.is_absolute()
        && path.components().all(|component| matches!(component, Component::Normal(_)))
}

#[cfg(all(test, unix))]
#[path = "toolchain_tests.rs"]
mod tests;
