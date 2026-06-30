use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use data_encoding::HEXLOWER;
use serde::Deserialize;
use serde::Serialize;
use url::Url;

use crate::errors::RunError;

pub const SOURCE_BUNDLE_FORMAT: &str = "mantle-source-bundle-v1";
pub const SOURCE_BUNDLE_VERSION: u32 = 1;
pub const SOURCE_BUNDLE_NON_CLAIM: &str =
    "source bundle evidence proves declared source/input availability and identity only";
pub const MAX_SOURCE_RECORDS: usize = 65_536;
pub const MAX_SOURCE_FILES_PER_RECORD: usize = 262_144;
pub const MAX_SOURCE_FILE_BYTES: u64 = 16_777_216;
pub const MAX_SOURCE_TOTAL_BYTES: u64 = 1_099_511_627_776;
pub const MAX_SOURCE_ID_BYTES: usize = 512;
pub const MAX_ADAPTER_METADATA_BYTES: usize = 8_192;
pub const MAX_SOURCE_RECORD_METADATA_BYTES: usize = 16_384;
pub const MAX_DERIVED_SOURCE_WALK_NODES: usize = 65_536;

const BUILTIN_FETCHURL_BUILDER: &str = "builtin:fetchurl";
const DERIVED_FIXED_URL_ID_PREFIX: &str = "fixed-url";
const DERIVED_STORE_PATH_ID_PREFIX: &str = "store-path";
const DERIVED_VCS_ID_PREFIX: &str = "vcs-snapshot";
const FETCH_ENV_EXECUTABLE_KEY: &str = "executable";
const FETCH_ENV_REV_KEY: &str = "rev";
const FETCH_ENV_TYPE_KEY: &str = "type";
const FETCH_ENV_UNPACK_KEY: &str = "unpack";
const FETCH_ENV_URL_KEY: &str = "url";
const FETCH_ENV_TYPE_GIT: &str = "git";
const DOT_GIT_DIR_NAME: &str = ".git";
const FILE_URL_SCHEME: &str = "file";
const RECORD_CONTENT_FILES_MARKER: &[u8] = b"files\0";
const RECORD_CONTENT_KIND_MARKER: &[u8] = b"kind\0";
const RECORD_CONTENT_METADATA_MARKER: &[u8] = b"metadata\0";
const RECORD_METADATA_BUILDER_KEY: &str = "builder";
const RECORD_METADATA_HASH_ALGO_KEY: &str = "hash_algo";
const RECORD_METADATA_HASH_KEY: &str = "hash";
const RECORD_METADATA_HASH_MODE_KEY: &str = "hash_mode";
const RECORD_METADATA_NAME_KEY: &str = "name";
const RECORD_METADATA_SOURCE_KIND_KEY: &str = "source_kind";
const RECORD_METADATA_STORE_PATH_KEY: &str = "store_path";
const RECORD_METADATA_URL_KEY: &str = "url";

const RECORD_SPEC_SEPARATOR: char = ':';
const SOURCE_STATE_DIR: &str = "source-bundles";
const SOURCE_RECORDS_DIR: &str = "records";
const SOURCE_PINS_DIR: &str = "pins";
const TEMP_FILE_EXTENSION: &str = "tmp";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum SourceRecordKind {
    FixedUrl,
    LocalPath,
    VcsSnapshot,
    PackageMirror,
    BootstrapArchive,
    ProviderManifest,
    ToolchainSourceRoot,
    ProofInput,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceAdapterMetadata {
    pub adapter: String,
    pub lock_identity: String,
    pub offline_control: String,
    pub generated_source_boundary: String,
    pub extra: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceFileEntry {
    pub path: String,
    pub file_type: SourceFileType,
    pub executable: bool,
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_hex: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symlink_target: Option<String>,
    pub blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceFileType {
    Regular,
    Symlink,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRecord {
    pub kind: SourceRecordKind,
    pub identity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store_prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adapter: Option<SourceAdapterMetadata>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, String>,
    pub payload_bytes: u64,
    pub content_blake3: String,
    pub files: Vec<SourceFileEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceBundleManifest {
    pub format: String,
    pub version: u32,
    pub store_prefix: String,
    pub roots: Vec<String>,
    pub records: Vec<SourceRecord>,
    pub manifest_blake3: String,
    pub non_claim: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceBundlePlanReport {
    pub format: &'static str,
    pub store_prefix: String,
    pub record_count: u32,
    pub payload_bytes: u64,
    pub ready_class: SourceReadiness,
    pub records: Vec<SourceRecordSummary>,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceBundleImportReport {
    pub imported_count: u32,
    pub skipped_present_count: u32,
    pub pinned: bool,
    pub manifest_blake3: String,
    pub records: Vec<SourceRecordSummary>,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceBundleVerifyReport {
    pub manifest_blake3: String,
    pub ready_class: SourceReadiness,
    pub missing_records: Vec<String>,
    pub stale_records: Vec<String>,
    pub unsupported_records: Vec<String>,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceRecordSummary {
    pub kind: SourceRecordKind,
    pub identity: String,
    pub payload_bytes: u64,
    pub content_blake3: String,
    pub file_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceReadiness {
    Ready,
    Missing,
    Stale,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSpec {
    pub kind: SourceRecordKind,
    pub identity: String,
    pub path: PathBuf,
    pub adapter: Option<SourceAdapterMetadata>,
}

impl std::str::FromStr for SourceRecordKind {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "fixed-url" => Ok(Self::FixedUrl),
            "local-path" => Ok(Self::LocalPath),
            "vcs-snapshot" => Ok(Self::VcsSnapshot),
            "package-mirror" => Ok(Self::PackageMirror),
            "bootstrap-archive" => Ok(Self::BootstrapArchive),
            "provider-manifest" => Ok(Self::ProviderManifest),
            "toolchain-source-root" => Ok(Self::ToolchainSourceRoot),
            "proof-input" => Ok(Self::ProofInput),
            other => Err(format!("unsupported source kind '{other}'")),
        }
    }
}

pub fn parse_source_spec(raw: &str) -> Result<SourceSpec, RunError> {
    let parts = raw.splitn(3, RECORD_SPEC_SEPARATOR).collect::<Vec<_>>();
    if parts.len() != 3 {
        return Err(RunError::Internal(format!("source spec must be kind:identity:path, got '{raw}'")));
    }
    let kind = parts[0].parse::<SourceRecordKind>().map_err(RunError::Internal)?;
    validate_identity(parts[1])?;
    Ok(SourceSpec {
        kind,
        identity: parts[1].to_string(),
        path: PathBuf::from(parts[2]),
        adapter: None,
    })
}

pub fn plan_source_bundle(specs: &[SourceSpec], store_prefix: &str) -> Result<SourceBundleManifest, RunError> {
    let records = canonicalize_source_specs(specs, store_prefix)?;
    assemble_source_bundle(records, store_prefix)
}

pub fn plan_source_bundle_from_derivations(
    roots: &[(String, crunch_glue::CrunchDerivation)],
    specs: &[SourceSpec],
    store_prefix: &str,
) -> Result<SourceBundleManifest, RunError> {
    let mut records = canonicalize_source_specs(specs, store_prefix)?;
    records.extend(collect_build_source_records(roots, store_prefix)?);
    assemble_source_bundle(records, store_prefix)
}

pub fn export_source_bundle_from_derivations(
    roots: &[(String, crunch_glue::CrunchDerivation)],
    specs: &[SourceSpec],
    store_prefix: &str,
) -> Result<SourceBundleManifest, RunError> {
    let mut records = canonicalize_source_specs(specs, store_prefix)?;
    records.extend(materialize_export_records(&collect_build_source_records(roots, store_prefix)?)?);
    assemble_source_bundle(records, store_prefix)
}

pub fn collect_build_source_records(
    roots: &[(String, crunch_glue::CrunchDerivation)],
    store_prefix: &str,
) -> Result<Vec<SourceRecord>, RunError> {
    if !store_prefix.starts_with('/') {
        return Err(RunError::Internal(format!("store prefix must be absolute: {store_prefix}")));
    }
    let mut records = Vec::new();
    let mut visited_count = 0usize;
    for (_, root) in roots {
        collect_derivation_source_records(root, store_prefix, &mut records, &mut visited_count)?;
    }
    Ok(records)
}

fn canonicalize_source_specs(specs: &[SourceSpec], store_prefix: &str) -> Result<Vec<SourceRecord>, RunError> {
    if specs.len() > MAX_SOURCE_RECORDS {
        return Err(RunError::Internal(format!("source record count exceeds {MAX_SOURCE_RECORDS}")));
    }
    let mut records = Vec::with_capacity(specs.len());
    for spec in specs {
        records.push(canonicalize_source_spec(spec, store_prefix)?);
    }
    Ok(records)
}

fn assemble_source_bundle(records: Vec<SourceRecord>, store_prefix: &str) -> Result<SourceBundleManifest, RunError> {
    if records.is_empty() {
        return Err(RunError::Internal("source bundle requires at least one --source or --build-root".to_string()));
    }
    if records.len() > MAX_SOURCE_RECORDS {
        return Err(RunError::Internal(format!("source record count exceeds {MAX_SOURCE_RECORDS}")));
    }
    if !store_prefix.starts_with('/') {
        return Err(RunError::Internal(format!("store prefix must be absolute: {store_prefix}")));
    }

    let records = normalize_source_records(records)?;
    let roots = records.iter().map(|record| record.identity.clone()).collect::<Vec<_>>();
    let mut manifest = SourceBundleManifest {
        format: SOURCE_BUNDLE_FORMAT.to_string(),
        version: SOURCE_BUNDLE_VERSION,
        store_prefix: store_prefix.to_string(),
        roots,
        records,
        manifest_blake3: String::new(),
        non_claim: SOURCE_BUNDLE_NON_CLAIM.to_string(),
    };
    manifest.manifest_blake3 = digest_manifest_without_digest(&manifest)?;
    Ok(manifest)
}

pub fn plan_report(manifest: &SourceBundleManifest) -> Result<SourceBundlePlanReport, RunError> {
    validate_manifest(manifest)?;
    Ok(SourceBundlePlanReport {
        format: SOURCE_BUNDLE_FORMAT,
        store_prefix: manifest.store_prefix.clone(),
        record_count: checked_u32(manifest.records.len(), "source record count")?,
        payload_bytes: manifest.records.iter().map(|record| record.payload_bytes).sum(),
        ready_class: SourceReadiness::Ready,
        records: manifest.records.iter().map(summary_for_record).collect::<Result<Vec<_>, _>>()?,
        non_claim: SOURCE_BUNDLE_NON_CLAIM,
    })
}

pub fn write_source_bundle(path: &Path, manifest: &SourceBundleManifest) -> Result<(), RunError> {
    validate_manifest(manifest)?;
    let rendered = serde_json::to_string_pretty(manifest)
        .map_err(|err| RunError::Internal(format!("serializing source bundle: {err}")))?;
    fs::write(path, format!("{rendered}\n"))
        .map_err(|err| RunError::Internal(format!("writing source bundle {}: {err}", path.display())))
}

pub fn read_source_bundle(path: &Path) -> Result<SourceBundleManifest, RunError> {
    let bytes =
        fs::read(path).map_err(|err| RunError::Internal(format!("reading source bundle {}: {err}", path.display())))?;
    let manifest = serde_json::from_slice::<SourceBundleManifest>(&bytes)
        .map_err(|err| RunError::Internal(format!("parsing source bundle {}: {err}", path.display())))?;
    validate_manifest(&manifest)?;
    Ok(manifest)
}

pub fn import_source_bundle(
    manifest: &SourceBundleManifest,
    state_dir: &Path,
    pin: bool,
) -> Result<SourceBundleImportReport, RunError> {
    validate_manifest(manifest)?;
    let records_dir = source_records_dir(state_dir);
    fs::create_dir_all(&records_dir)
        .map_err(|err| RunError::Internal(format!("creating source records dir {}: {err}", records_dir.display())))?;
    let mut imported_count = 0u32;
    let mut skipped_present_count = 0u32;
    let mut summaries = Vec::with_capacity(manifest.records.len());
    for record in &manifest.records {
        let summary = summary_for_record(record)?;
        let target = records_dir.join(format!("{}.json", record.content_blake3));
        if target.exists() {
            skipped_present_count = skipped_present_count.saturating_add(1);
        } else {
            write_record_atomically(&target, record)?;
            imported_count = imported_count.saturating_add(1);
        }
        summaries.push(summary);
    }
    if pin {
        write_pin_atomically(state_dir, manifest)?;
    }
    Ok(SourceBundleImportReport {
        imported_count,
        skipped_present_count,
        pinned: pin,
        manifest_blake3: manifest.manifest_blake3.clone(),
        records: summaries,
        non_claim: SOURCE_BUNDLE_NON_CLAIM,
    })
}

pub fn verify_source_bundle_state(
    manifest: &SourceBundleManifest,
    state_dir: &Path,
) -> Result<SourceBundleVerifyReport, RunError> {
    validate_manifest(manifest)?;
    let mut missing_records = Vec::new();
    let mut stale_records = Vec::new();
    let unsupported_records = Vec::new();
    for record in &manifest.records {
        let record_path = source_records_dir(state_dir).join(format!("{}.json", record.content_blake3));
        if !record_path.exists() {
            missing_records.push(record.identity.clone());
            continue;
        }
        let stored = read_record(&record_path)?;
        if &stored != record {
            stale_records.push(record.identity.clone());
        }
    }
    let ready_class = classify_source_state(&missing_records, &stale_records, &unsupported_records);
    Ok(SourceBundleVerifyReport {
        manifest_blake3: manifest.manifest_blake3.clone(),
        ready_class,
        missing_records,
        stale_records,
        unsupported_records,
        non_claim: SOURCE_BUNDLE_NON_CLAIM,
    })
}

pub fn list_source_bundle(manifest: &SourceBundleManifest) -> Result<SourceBundlePlanReport, RunError> {
    plan_report(manifest)
}

fn canonicalize_source_spec(spec: &SourceSpec, store_prefix: &str) -> Result<SourceRecord, RunError> {
    let (files, total_bytes) = canonicalize_payload_entries(&spec.path, false)?;
    validate_adapter_metadata(spec.adapter.as_ref())?;
    let content_blake3 = digest_source_record_content(&spec.kind, &BTreeMap::new(), &files)?;
    Ok(SourceRecord {
        kind: spec.kind.clone(),
        identity: spec.identity.clone(),
        store_prefix: record_store_prefix(&spec.kind, store_prefix),
        adapter: spec.adapter.clone(),
        metadata: BTreeMap::new(),
        payload_bytes: total_bytes,
        content_blake3,
        files,
    })
}

fn canonicalize_payload_entries(path: &Path, skip_git_dir: bool) -> Result<(Vec<SourceFileEntry>, u64), RunError> {
    let root = fs::canonicalize(path)
        .map_err(|err| RunError::Internal(format!("canonicalizing source path {}: {err}", path.display())))?;
    let metadata = fs::symlink_metadata(&root)
        .map_err(|err| RunError::Internal(format!("reading source metadata {}: {err}", root.display())))?;
    let relative_root = if metadata.is_file() || metadata.file_type().is_symlink() {
        root.parent()
            .ok_or_else(|| RunError::Internal(format!("source path {} has no parent", root.display())))?
            .to_path_buf()
    } else {
        root.clone()
    };
    let mut files = Vec::new();
    let mut total_bytes = 0u64;
    collect_source_entries(&relative_root, &root, &mut files, &mut total_bytes, skip_git_dir)?;
    files.sort_by(|left, right| left.path.cmp(&right.path));
    if files.len() > MAX_SOURCE_FILES_PER_RECORD {
        return Err(RunError::Internal(format!("source file count exceeds {MAX_SOURCE_FILES_PER_RECORD}")));
    }
    Ok((files, total_bytes))
}

fn collect_source_entries(
    root: &Path,
    current: &Path,
    files: &mut Vec<SourceFileEntry>,
    total_bytes: &mut u64,
    skip_git_dir: bool,
) -> Result<(), RunError> {
    let metadata = fs::symlink_metadata(current)
        .map_err(|err| RunError::Internal(format!("reading source metadata {}: {err}", current.display())))?;
    if metadata.is_dir() {
        let mut entries = fs::read_dir(current)
            .map_err(|err| RunError::Internal(format!("reading source dir {}: {err}", current.display())))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| RunError::Internal(format!("reading source dir entry {}: {err}", current.display())))?;
        entries.sort_by_key(|entry| entry.path());
        for entry in entries {
            if skip_git_dir && entry.file_name().to_str() == Some(DOT_GIT_DIR_NAME) {
                continue;
            }
            collect_source_entries(root, &entry.path(), files, total_bytes, skip_git_dir)?;
        }
        return Ok(());
    }
    let relative = safe_relative_path(root, current)?;
    if metadata.file_type().is_symlink() {
        let target = fs::read_link(current)
            .map_err(|err| RunError::Internal(format!("reading source symlink {}: {err}", current.display())))?;
        let target_text = safe_symlink_target(&target)?;
        let digest = blake3::hash(format!("symlink\0{relative}\0{target_text}").as_bytes());
        files.push(SourceFileEntry {
            path: relative,
            file_type: SourceFileType::Symlink,
            executable: false,
            size: 0,
            content_hex: None,
            symlink_target: Some(target_text),
            blake3: digest.to_hex().to_string(),
        });
        return Ok(());
    }
    if !metadata.is_file() {
        return Err(RunError::Internal(format!("unsupported source file kind at {}", current.display())));
    }
    let size = metadata.len();
    if size > MAX_SOURCE_FILE_BYTES {
        return Err(RunError::Internal(format!(
            "source file {} is {size} bytes, limit {MAX_SOURCE_FILE_BYTES}",
            current.display()
        )));
    }
    *total_bytes = total_bytes
        .checked_add(size)
        .ok_or_else(|| RunError::Internal("source payload byte count overflow".to_string()))?;
    if *total_bytes > MAX_SOURCE_TOTAL_BYTES {
        return Err(RunError::Internal(format!("source payload bytes exceed {MAX_SOURCE_TOTAL_BYTES}")));
    }
    let content = fs::read(current)
        .map_err(|err| RunError::Internal(format!("reading source file {}: {err}", current.display())))?;
    let digest = blake3::hash(&content).to_hex().to_string();
    files.push(SourceFileEntry {
        path: relative,
        file_type: SourceFileType::Regular,
        executable: is_executable(&metadata),
        size,
        content_hex: Some(HEXLOWER.encode(&content)),
        symlink_target: None,
        blake3: digest,
    });
    Ok(())
}

fn safe_relative_path(root: &Path, current: &Path) -> Result<String, RunError> {
    let relative = current
        .strip_prefix(root)
        .map_err(|err| RunError::Internal(format!("source path escaped root: {err}")))?;
    let text = relative
        .to_str()
        .ok_or_else(|| RunError::Internal(format!("source path is not UTF-8: {}", current.display())))?;
    if text.is_empty() || text.starts_with('/') || text.contains("..") || text.contains('\\') {
        return Err(RunError::Internal(format!("unsafe source relative path '{text}'")));
    }
    Ok(text.to_string())
}

fn safe_symlink_target(target: &Path) -> Result<String, RunError> {
    let target_text = target
        .to_str()
        .ok_or_else(|| RunError::Internal(format!("symlink target is not UTF-8: {}", target.display())))?;
    if target.is_absolute() || target_text.contains("..") || target_text.is_empty() {
        return Err(RunError::Internal(format!("unsafe source symlink target '{target_text}'")));
    }
    Ok(target_text.to_string())
}

#[cfg(unix)]
fn is_executable(metadata: &fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    const UNIX_EXECUTE_BITS: u32 = 0o111;
    metadata.permissions().mode() & UNIX_EXECUTE_BITS != 0
}

#[cfg(not(unix))]
fn is_executable(_metadata: &fs::Metadata) -> bool {
    false
}

fn validate_identity(identity: &str) -> Result<(), RunError> {
    if identity.is_empty() || identity.len() > MAX_SOURCE_ID_BYTES {
        return Err(RunError::Internal(format!("invalid source identity length for '{identity}'")));
    }
    if identity.contains('\0') || identity.contains('/') || identity.contains("..") {
        return Err(RunError::Internal(format!("unsafe source identity '{identity}'")));
    }
    Ok(())
}

fn validate_adapter_metadata(adapter: Option<&SourceAdapterMetadata>) -> Result<(), RunError> {
    let Some(adapter) = adapter else { return Ok(()) };
    let rendered = serde_json::to_vec(adapter)
        .map_err(|err| RunError::Internal(format!("serializing adapter metadata: {err}")))?;
    if rendered.len() > MAX_ADAPTER_METADATA_BYTES {
        return Err(RunError::Internal(format!("adapter metadata exceeds {MAX_ADAPTER_METADATA_BYTES} bytes")));
    }
    if adapter.adapter.is_empty() || adapter.lock_identity.is_empty() || adapter.offline_control.is_empty() {
        return Err(RunError::Internal("adapter metadata missing lock/offline identity".to_string()));
    }
    Ok(())
}

fn record_store_prefix(kind: &SourceRecordKind, store_prefix: &str) -> Option<String> {
    match kind {
        SourceRecordKind::ProviderManifest | SourceRecordKind::ToolchainSourceRoot => Some(store_prefix.to_string()),
        _ => None,
    }
}

fn digest_source_entries(files: &[SourceFileEntry]) -> Result<String, RunError> {
    let mut hasher = blake3::Hasher::new();
    hash_source_entries(&mut hasher, files)?;
    Ok(hasher.finalize().to_hex().to_string())
}

fn hash_source_entries(hasher: &mut blake3::Hasher, files: &[SourceFileEntry]) -> Result<(), RunError> {
    for file in files {
        let encoded =
            serde_json::to_vec(file).map_err(|err| RunError::Internal(format!("serializing source entry: {err}")))?;
        hasher.update(&encoded);
        hasher.update(b"\n");
    }
    Ok(())
}

fn digest_manifest_without_digest(manifest: &SourceBundleManifest) -> Result<String, RunError> {
    let mut clone = manifest.clone();
    clone.manifest_blake3.clear();
    let encoded = serde_json::to_vec(&clone)
        .map_err(|err| RunError::Internal(format!("serializing source manifest for digest: {err}")))?;
    Ok(blake3::hash(&encoded).to_hex().to_string())
}

fn digest_virtual_source_record(
    kind: &SourceRecordKind,
    metadata: &BTreeMap<String, String>,
) -> Result<String, RunError> {
    let encoded = serde_json::to_vec(&(kind, metadata))
        .map_err(|err| RunError::Internal(format!("serializing source record metadata: {err}")))?;
    Ok(blake3::hash(&encoded).to_hex().to_string())
}

fn digest_source_record_content(
    kind: &SourceRecordKind,
    metadata: &BTreeMap<String, String>,
    files: &[SourceFileEntry],
) -> Result<String, RunError> {
    if metadata.is_empty() {
        return digest_source_entries(files);
    }
    if files.is_empty() {
        return digest_virtual_source_record(kind, metadata);
    }
    let mut hasher = blake3::Hasher::new();
    hasher.update(RECORD_CONTENT_KIND_MARKER);
    let encoded_kind =
        serde_json::to_vec(kind).map_err(|err| RunError::Internal(format!("serializing source record kind: {err}")))?;
    hasher.update(&encoded_kind);
    hasher.update(b"\n");
    hasher.update(RECORD_CONTENT_METADATA_MARKER);
    let encoded_metadata = serde_json::to_vec(metadata)
        .map_err(|err| RunError::Internal(format!("serializing source record metadata: {err}")))?;
    hasher.update(&encoded_metadata);
    hasher.update(b"\n");
    hasher.update(RECORD_CONTENT_FILES_MARKER);
    hash_source_entries(&mut hasher, files)?;
    Ok(hasher.finalize().to_hex().to_string())
}

fn validate_manifest(manifest: &SourceBundleManifest) -> Result<(), RunError> {
    if manifest.format != SOURCE_BUNDLE_FORMAT {
        return Err(RunError::Internal(format!("unsupported source bundle format {}", manifest.format)));
    }
    if manifest.version != SOURCE_BUNDLE_VERSION {
        return Err(RunError::Internal(format!("unsupported source bundle version {}", manifest.version)));
    }
    if manifest.records.len() > MAX_SOURCE_RECORDS {
        return Err(RunError::Internal(format!("source bundle records exceed {MAX_SOURCE_RECORDS}")));
    }
    reject_duplicate_records(&manifest.records)?;
    let expected = digest_manifest_without_digest(manifest)?;
    if expected != manifest.manifest_blake3 {
        return Err(RunError::Internal("source bundle manifest digest mismatch".to_string()));
    }
    Ok(())
}

fn normalize_source_records(mut records: Vec<SourceRecord>) -> Result<Vec<SourceRecord>, RunError> {
    records.sort_by(|left, right| record_sort_key(left).cmp(&record_sort_key(right)));
    let mut normalized = Vec::<SourceRecord>::with_capacity(records.len());
    for record in records {
        validate_source_record(&record)?;
        let key = record_sort_key(&record);
        if let Some(previous) = normalized.last()
            && record_sort_key(previous) == key
        {
            if previous == &record {
                continue;
            }
            return Err(RunError::Internal(format!("conflicting duplicate source record {key}")));
        }
        normalized.push(record);
    }
    Ok(normalized)
}

fn reject_duplicate_records(records: &[SourceRecord]) -> Result<(), RunError> {
    let mut seen = BTreeSet::new();
    for record in records {
        validate_source_record(record)?;
        let key = record_sort_key(record);
        if !seen.insert(key.clone()) {
            return Err(RunError::Internal(format!("duplicate source record {key}")));
        }
    }
    Ok(())
}

fn validate_source_record(record: &SourceRecord) -> Result<(), RunError> {
    validate_identity(&record.identity)?;
    validate_adapter_metadata(record.adapter.as_ref())?;
    let rendered_metadata = serde_json::to_vec(&record.metadata)
        .map_err(|err| RunError::Internal(format!("serializing source metadata: {err}")))?;
    if rendered_metadata.len() > MAX_SOURCE_RECORD_METADATA_BYTES {
        return Err(RunError::Internal(format!(
            "source record metadata exceeds {MAX_SOURCE_RECORD_METADATA_BYTES} bytes"
        )));
    }
    let expected_digest = digest_source_record_content(&record.kind, &record.metadata, &record.files)?;
    if expected_digest != record.content_blake3 {
        return Err(RunError::Internal(format!("source record {} content digest mismatch", record.identity)));
    }
    Ok(())
}

fn record_sort_key(record: &SourceRecord) -> String {
    format!("{:?}:{}", record.kind, record.identity)
}

fn summary_for_record(record: &SourceRecord) -> Result<SourceRecordSummary, RunError> {
    Ok(SourceRecordSummary {
        kind: record.kind.clone(),
        identity: record.identity.clone(),
        payload_bytes: record.payload_bytes,
        content_blake3: record.content_blake3.clone(),
        file_count: checked_u32(record.files.len(), "source file count")?,
    })
}

fn classify_source_state(missing: &[String], stale: &[String], unsupported: &[String]) -> SourceReadiness {
    if !unsupported.is_empty() {
        return SourceReadiness::Unsupported;
    }
    if !stale.is_empty() {
        return SourceReadiness::Stale;
    }
    if !missing.is_empty() {
        return SourceReadiness::Missing;
    }
    SourceReadiness::Ready
}

fn source_records_dir(state_dir: &Path) -> PathBuf {
    state_dir.join(SOURCE_STATE_DIR).join(SOURCE_RECORDS_DIR)
}

fn source_pins_dir(state_dir: &Path) -> PathBuf {
    state_dir.join(SOURCE_STATE_DIR).join(SOURCE_PINS_DIR)
}

fn write_record_atomically(target: &Path, record: &SourceRecord) -> Result<(), RunError> {
    let rendered = serde_json::to_string_pretty(record)
        .map_err(|err| RunError::Internal(format!("serializing source record: {err}")))?;
    let tmp = target.with_extension(TEMP_FILE_EXTENSION);
    fs::write(&tmp, format!("{rendered}\n"))
        .map_err(|err| RunError::Internal(format!("writing source record temp {}: {err}", tmp.display())))?;
    fs::rename(&tmp, target)
        .map_err(|err| RunError::Internal(format!("committing source record {}: {err}", target.display())))
}

fn write_pin_atomically(state_dir: &Path, manifest: &SourceBundleManifest) -> Result<(), RunError> {
    let pins_dir = source_pins_dir(state_dir);
    fs::create_dir_all(&pins_dir)
        .map_err(|err| RunError::Internal(format!("creating source pins dir {}: {err}", pins_dir.display())))?;
    let target = pins_dir.join(format!("{}.json", manifest.manifest_blake3));
    let tmp = target.with_extension(TEMP_FILE_EXTENSION);
    let rendered = serde_json::to_string_pretty(manifest)
        .map_err(|err| RunError::Internal(format!("serializing source pin: {err}")))?;
    fs::write(&tmp, format!("{rendered}\n"))
        .map_err(|err| RunError::Internal(format!("writing source pin temp {}: {err}", tmp.display())))?;
    fs::rename(&tmp, &target)
        .map_err(|err| RunError::Internal(format!("committing source pin {}: {err}", target.display())))
}

fn read_record(path: &Path) -> Result<SourceRecord, RunError> {
    let bytes =
        fs::read(path).map_err(|err| RunError::Internal(format!("reading source record {}: {err}", path.display())))?;
    serde_json::from_slice(&bytes)
        .map_err(|err| RunError::Internal(format!("parsing source record {}: {err}", path.display())))
}

fn materialize_export_records(records: &[SourceRecord]) -> Result<Vec<SourceRecord>, RunError> {
    let mut materialized = Vec::with_capacity(records.len());
    for record in records {
        materialized.push(match record.kind {
            SourceRecordKind::FixedUrl => materialize_file_url_record(record, false)?,
            SourceRecordKind::VcsSnapshot => materialize_file_url_record(record, true)?,
            _ => record.clone(),
        });
    }
    Ok(materialized)
}

fn materialize_file_url_record(record: &SourceRecord, skip_git_dir: bool) -> Result<SourceRecord, RunError> {
    let url_text = record
        .metadata
        .get(RECORD_METADATA_URL_KEY)
        .ok_or_else(|| RunError::Internal(format!("source record {} is missing url metadata", record.identity)))?;
    let payload_path = local_file_url_path(url_text).ok_or_else(|| {
        RunError::Internal(format!(
            "source bundle export cannot materialize non-local source URL for {}: {url_text}",
            record.identity
        ))
    })?;
    let (files, payload_bytes) = canonicalize_payload_entries(&payload_path, skip_git_dir)?;
    let content_blake3 = digest_source_record_content(&record.kind, &record.metadata, &files)?;
    Ok(SourceRecord {
        payload_bytes,
        content_blake3,
        files,
        ..record.clone()
    })
}

fn local_file_url_path(raw_url: &str) -> Option<PathBuf> {
    let parsed = Url::parse(raw_url).ok()?;
    if parsed.scheme() != FILE_URL_SCHEME {
        return None;
    }
    parsed.to_file_path().ok()
}

fn collect_derivation_source_records(
    derivation: &crunch_glue::CrunchDerivation,
    store_prefix: &str,
    records: &mut Vec<SourceRecord>,
    visited_count: &mut usize,
) -> Result<(), RunError> {
    *visited_count = visited_count
        .checked_add(1)
        .ok_or_else(|| RunError::Internal("derived source walk count overflow".to_string()))?;
    if *visited_count > MAX_DERIVED_SOURCE_WALK_NODES {
        return Err(RunError::Internal(format!(
            "derived source walk exceeds {MAX_DERIVED_SOURCE_WALK_NODES} derivation nodes"
        )));
    }
    if let Some(record) = fixed_fetcher_source_record(derivation)? {
        records.push(record);
    }
    for input in &derivation.inputs {
        match input {
            crunch_glue::Input::Source(source_path) => {
                records.push(store_path_source_record(source_path, store_prefix)?)
            }
            crunch_glue::Input::OutputSelection(output) => {
                collect_derivation_source_records(&output.drv, store_prefix, records, visited_count)?;
            }
            crunch_glue::Input::Derivation(input_derivation) => {
                collect_derivation_source_records(input_derivation, store_prefix, records, visited_count)?;
            }
        }
    }
    Ok(())
}

fn fixed_fetcher_source_record(derivation: &crunch_glue::CrunchDerivation) -> Result<Option<SourceRecord>, RunError> {
    if derivation.builder != BUILTIN_FETCHURL_BUILDER {
        return Ok(None);
    }
    let fixed_output = derivation.fixed_output.as_ref().ok_or_else(|| {
        RunError::Internal(format!("builtin fetcher {} cannot be source-bundled without fixed_output", derivation.name))
    })?;
    let url = derivation
        .env
        .get(FETCH_ENV_URL_KEY)
        .ok_or_else(|| RunError::Internal(format!("builtin fetcher {} is missing env.url", derivation.name)))?;
    let mut metadata = BTreeMap::new();
    metadata.insert(RECORD_METADATA_BUILDER_KEY.to_string(), derivation.builder.clone());
    metadata.insert(RECORD_METADATA_HASH_ALGO_KEY.to_string(), fixed_output.algo.clone());
    metadata.insert(RECORD_METADATA_HASH_KEY.to_string(), fixed_output.hash.clone());
    metadata.insert(RECORD_METADATA_HASH_MODE_KEY.to_string(), fixed_output.mode.clone());
    metadata.insert(RECORD_METADATA_NAME_KEY.to_string(), derivation.name.clone());
    metadata.insert(RECORD_METADATA_URL_KEY.to_string(), url.clone());
    copy_optional_env_metadata(&derivation.env, &mut metadata, FETCH_ENV_EXECUTABLE_KEY);
    copy_optional_env_metadata(&derivation.env, &mut metadata, FETCH_ENV_REV_KEY);
    copy_optional_env_metadata(&derivation.env, &mut metadata, FETCH_ENV_TYPE_KEY);
    copy_optional_env_metadata(&derivation.env, &mut metadata, FETCH_ENV_UNPACK_KEY);
    let kind = if derivation.env.get(FETCH_ENV_TYPE_KEY).map(String::as_str) == Some(FETCH_ENV_TYPE_GIT) {
        SourceRecordKind::VcsSnapshot
    } else {
        SourceRecordKind::FixedUrl
    };
    let id_prefix = if kind == SourceRecordKind::VcsSnapshot {
        DERIVED_VCS_ID_PREFIX
    } else {
        DERIVED_FIXED_URL_ID_PREFIX
    };
    Ok(Some(virtual_source_record(kind, id_prefix, None, metadata)?))
}

fn store_path_source_record(source_path: &str, store_prefix: &str) -> Result<SourceRecord, RunError> {
    if !source_path.starts_with('/') {
        return Err(RunError::Internal(format!(
            "source input path must be absolute for source bundle planning: {source_path}"
        )));
    }
    let mut metadata = BTreeMap::new();
    metadata.insert(RECORD_METADATA_SOURCE_KIND_KEY.to_string(), "pre-existing-store-path".to_string());
    metadata.insert(RECORD_METADATA_STORE_PATH_KEY.to_string(), source_path.to_string());
    virtual_source_record(
        SourceRecordKind::ToolchainSourceRoot,
        DERIVED_STORE_PATH_ID_PREFIX,
        Some(store_prefix.to_string()),
        metadata,
    )
}

fn copy_optional_env_metadata(
    env: &std::collections::HashMap<String, String>,
    metadata: &mut BTreeMap<String, String>,
    key: &str,
) {
    if let Some(value) = env.get(key) {
        metadata.insert(key.to_string(), value.clone());
    }
}

fn virtual_source_record(
    kind: SourceRecordKind,
    id_prefix: &str,
    store_prefix: Option<String>,
    metadata: BTreeMap<String, String>,
) -> Result<SourceRecord, RunError> {
    let digest = digest_virtual_source_record(&kind, &metadata)?;
    let identity = format!("{id_prefix}-{digest}");
    validate_identity(&identity)?;
    Ok(SourceRecord {
        kind,
        identity,
        store_prefix,
        adapter: None,
        metadata,
        payload_bytes: 0,
        content_blake3: digest,
        files: Vec::new(),
    })
}

fn checked_u32(count: usize, label: &str) -> Result<u32, RunError> {
    u32::try_from(count).map_err(|_| RunError::Internal(format!("{label} does not fit in u32: {count}")))
}

pub fn render_json(value: &impl Serialize) -> Result<String, RunError> {
    serde_json::to_string_pretty(value)
        .map_err(|err| RunError::Internal(format!("serializing source bundle report: {err}")))
}

pub fn cmd_source(
    action: crate::SourceAction,
    state_dir: &Path,
    store_prefix: &str,
    json_output: bool,
) -> Result<(), RunError> {
    match action {
        crate::SourceAction::Bundle { action } => cmd_source_bundle(action, state_dir, store_prefix, json_output),
    }
}

fn cmd_source_bundle(
    action: crate::SourceBundleAction,
    state_dir: &Path,
    store_prefix: &str,
    json_output: bool,
) -> Result<(), RunError> {
    match action {
        crate::SourceBundleAction::Plan {
            sources,
            build_roots,
            import_paths,
        } => {
            let manifest = plan_from_cli_inputs(&sources, &build_roots, &import_paths, store_prefix)?;
            print_plan_report(&plan_report(&manifest)?, json_output)
        }
        crate::SourceBundleAction::Export {
            sources,
            build_roots,
            import_paths,
            to,
        } => {
            let manifest = export_from_cli_inputs(&sources, &build_roots, &import_paths, store_prefix)?;
            write_source_bundle(&to, &manifest)?;
            print_plan_report(&plan_report(&manifest)?, json_output)
        }
        crate::SourceBundleAction::List { from } => {
            let manifest = read_source_bundle(&from)?;
            print_plan_report(&list_source_bundle(&manifest)?, json_output)
        }
        crate::SourceBundleAction::Import { from, pin } => {
            let manifest = read_source_bundle(&from)?;
            let report = import_source_bundle(&manifest, state_dir, pin)?;
            print_import_report(&report, json_output)
        }
        crate::SourceBundleAction::Verify { from, imported } => {
            let manifest = read_source_bundle(&from)?;
            let report = if imported {
                verify_source_bundle_state(&manifest, state_dir)?
            } else {
                SourceBundleVerifyReport {
                    manifest_blake3: manifest.manifest_blake3.clone(),
                    ready_class: SourceReadiness::Ready,
                    missing_records: Vec::new(),
                    stale_records: Vec::new(),
                    unsupported_records: Vec::new(),
                    non_claim: SOURCE_BUNDLE_NON_CLAIM,
                }
            };
            print_verify_report(&report, json_output)
        }
    }
}

fn plan_from_cli_inputs(
    sources: &[String],
    build_roots: &[PathBuf],
    import_paths: &[PathBuf],
    store_prefix: &str,
) -> Result<SourceBundleManifest, RunError> {
    let specs = sources.iter().map(|source| parse_source_spec(source)).collect::<Result<Vec<_>, _>>()?;
    if build_roots.is_empty() {
        return plan_source_bundle(&specs, store_prefix);
    }
    let roots = evaluate_build_roots(build_roots, import_paths)?;
    plan_source_bundle_from_derivations(&roots, &specs, store_prefix)
}

fn export_from_cli_inputs(
    sources: &[String],
    build_roots: &[PathBuf],
    import_paths: &[PathBuf],
    store_prefix: &str,
) -> Result<SourceBundleManifest, RunError> {
    let specs = sources.iter().map(|source| parse_source_spec(source)).collect::<Result<Vec<_>, _>>()?;
    if build_roots.is_empty() {
        return plan_source_bundle(&specs, store_prefix);
    }
    let roots = evaluate_build_roots(build_roots, import_paths)?;
    export_source_bundle_from_derivations(&roots, &specs, store_prefix)
}

fn evaluate_build_roots(
    build_roots: &[PathBuf],
    import_paths: &[PathBuf],
) -> Result<Vec<(String, crunch_glue::CrunchDerivation)>, RunError> {
    if build_roots.is_empty() {
        return Ok(Vec::new());
    }
    let full_import_paths = crate::build_cmd::build_import_paths(import_paths)?;
    let mut derivations = Vec::new();
    for build_root in build_roots {
        let mut session = crunch_eval::session::EvaluationSession::open_file(build_root, &full_import_paths)
            .map_err(|err| RunError::Eval(format!("opening build root {}: {err}", build_root.display())))?;
        let roots = session.force_all_roots::<crunch_glue::CrunchDerivation>().map_err(|err| {
            RunError::Eval(format!("evaluating source bundle build root {}: {err}", build_root.display()))
        })?;
        derivations.extend(roots);
    }
    Ok(derivations)
}

fn print_plan_report(report: &SourceBundlePlanReport, json_output: bool) -> Result<(), RunError> {
    if json_output {
        println!("{}", render_json(report)?);
        return Ok(());
    }
    println!(
        "format={} store_prefix={} records={} payload_bytes={} readiness={:?}",
        report.format, report.store_prefix, report.record_count, report.payload_bytes, report.ready_class
    );
    for record in &report.records {
        println!(
            "SOURCE_RECORD kind={:?} identity={} files={} payload_bytes={} blake3={}",
            record.kind, record.identity, record.file_count, record.payload_bytes, record.content_blake3
        );
    }
    eprintln!("non_claim={}", report.non_claim);
    Ok(())
}

fn print_import_report(report: &SourceBundleImportReport, json_output: bool) -> Result<(), RunError> {
    if json_output {
        println!("{}", render_json(report)?);
        return Ok(());
    }
    for record in &report.records {
        println!("SOURCE_IMPORT identity={} blake3={}", record.identity, record.content_blake3);
    }
    eprintln!(
        "imported={} skipped_present={} pinned={} manifest_blake3={}",
        report.imported_count, report.skipped_present_count, report.pinned, report.manifest_blake3
    );
    eprintln!("non_claim={}", report.non_claim);
    Ok(())
}

fn print_verify_report(report: &SourceBundleVerifyReport, json_output: bool) -> Result<(), RunError> {
    if json_output {
        println!("{}", render_json(report)?);
        return Ok(());
    }
    println!(
        "manifest_blake3={} readiness={:?} missing={} stale={} unsupported={}",
        report.manifest_blake3,
        report.ready_class,
        report.missing_records.len(),
        report.stale_records.len(),
        report.unsupported_records.len()
    );
    eprintln!("non_claim={}", report.non_claim);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_fixture(root: &Path) {
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/main.txt"), b"hello").unwrap();
    }

    fn fixed_fetcher(name: &str, url: &str) -> crunch_glue::CrunchDerivation {
        let mut env = std::collections::HashMap::new();
        env.insert(FETCH_ENV_URL_KEY.to_string(), url.to_string());
        crunch_glue::CrunchDerivation {
            name: name.to_string(),
            builder: BUILTIN_FETCHURL_BUILDER.to_string(),
            system: "x86_64-linux".to_string(),
            args: Vec::new(),
            outputs: vec!["out".to_string()],
            dynamic_plan_outputs: Vec::new(),
            env,
            inputs: Vec::new(),
            fixed_output: Some(crunch_glue::FixedOutput {
                hash: "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".to_string(),
                algo: "sha256".to_string(),
                mode: "flat".to_string(),
            }),
            addressing_mode: "input-addressed".to_string(),
            provenance: None,
        }
    }

    fn root_derivation(inputs: Vec<crunch_glue::Input>) -> crunch_glue::CrunchDerivation {
        crunch_glue::CrunchDerivation {
            name: "root".to_string(),
            builder: "/bin/sh".to_string(),
            system: "x86_64-linux".to_string(),
            args: vec!["-c".to_string(), "true".to_string()],
            outputs: vec!["out".to_string()],
            dynamic_plan_outputs: Vec::new(),
            env: std::collections::HashMap::new(),
            inputs,
            fixed_output: None,
            addressing_mode: "input-addressed".to_string(),
            provenance: None,
        }
    }

    fn file_url(path: &Path) -> String {
        if path.is_dir() {
            return Url::from_directory_path(path).unwrap().to_string();
        }
        Url::from_file_path(path).unwrap().to_string()
    }

    #[test]
    fn source_bundle_canonicalizes_equivalent_traversal() {
        let temp = tempfile::tempdir().unwrap();
        write_fixture(temp.path());
        let spec = SourceSpec {
            kind: SourceRecordKind::LocalPath,
            identity: "fixture".to_string(),
            path: temp.path().to_path_buf(),
            adapter: None,
        };
        let first = plan_source_bundle(std::slice::from_ref(&spec), "/mantle/store").unwrap();
        let second = plan_source_bundle(&[spec], "/mantle/store").unwrap();
        assert_eq!(first.manifest_blake3, second.manifest_blake3);
        assert_eq!(first.records[0].files[0].path, "src/main.txt");
    }

    #[test]
    fn source_bundle_accepts_single_file_payload_root() {
        let temp = tempfile::tempdir().unwrap();
        let source_file = temp.path().join("payload.txt");
        fs::write(&source_file, b"payload").unwrap();
        let spec = SourceSpec {
            kind: SourceRecordKind::LocalPath,
            identity: "fixture-file".to_string(),
            path: source_file,
            adapter: None,
        };
        let manifest = plan_source_bundle(&[spec], "/mantle/store").unwrap();

        assert_eq!(manifest.records[0].files.len(), 1);
        assert_eq!(manifest.records[0].files[0].path, "payload.txt");
        assert_eq!(manifest.records[0].payload_bytes, 7);
    }

    #[test]
    fn source_bundle_rejects_unsafe_identity() {
        let err = parse_source_spec("local-path:../bad:/tmp").unwrap_err();
        assert!(err.to_string().contains("unsafe source identity"));
    }

    #[test]
    fn source_bundle_derives_fetcher_and_store_path_inputs_from_build_root() {
        let fetcher = fixed_fetcher("crate-src", "https://static.example.invalid/crate.tar.gz");
        let store_path = "/nix/store/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa-bash".to_string();
        let root = root_derivation(vec![
            crunch_glue::Input::Derivation(Box::new(fetcher.clone())),
            crunch_glue::Input::Derivation(Box::new(fetcher)),
            crunch_glue::Input::Source(store_path.clone()),
            crunch_glue::Input::Source(store_path.clone()),
        ]);
        let manifest =
            plan_source_bundle_from_derivations(&[("default".to_string(), root)], &[], "/mantle/store").unwrap();

        assert_eq!(manifest.records.len(), 2);
        let fixed = manifest.records.iter().find(|record| record.kind == SourceRecordKind::FixedUrl).unwrap();
        assert_eq!(fixed.payload_bytes, 0);
        assert!(fixed.files.is_empty());
        assert_eq!(
            fixed.metadata.get(RECORD_METADATA_URL_KEY).map(String::as_str),
            Some("https://static.example.invalid/crate.tar.gz")
        );
        assert_eq!(fixed.metadata.get(RECORD_METADATA_HASH_ALGO_KEY).map(String::as_str), Some("sha256"));

        let store =
            manifest.records.iter().find(|record| record.kind == SourceRecordKind::ToolchainSourceRoot).unwrap();
        assert_eq!(store.store_prefix.as_deref(), Some("/mantle/store"));
        assert_eq!(store.metadata.get(RECORD_METADATA_STORE_PATH_KEY), Some(&store_path));
        assert!(store.files.is_empty());
    }

    #[test]
    fn source_bundle_derives_vcs_snapshot_from_git_fetcher() {
        let mut git = fixed_fetcher("repo-src", "https://example.invalid/repo.git");
        git.env.insert(FETCH_ENV_TYPE_KEY.to_string(), FETCH_ENV_TYPE_GIT.to_string());
        git.env.insert(FETCH_ENV_REV_KEY.to_string(), "refs/tags/v1".to_string());
        let root = root_derivation(vec![crunch_glue::Input::Derivation(Box::new(git))]);
        let manifest =
            plan_source_bundle_from_derivations(&[("default".to_string(), root)], &[], "/mantle/store").unwrap();

        assert_eq!(manifest.records.len(), 1);
        let record = &manifest.records[0];
        assert_eq!(record.kind, SourceRecordKind::VcsSnapshot);
        assert_eq!(record.metadata.get(FETCH_ENV_REV_KEY).map(String::as_str), Some("refs/tags/v1"));
        assert_eq!(record.metadata.get(FETCH_ENV_TYPE_KEY).map(String::as_str), Some(FETCH_ENV_TYPE_GIT));
    }

    #[test]
    fn source_bundle_export_materializes_local_file_fetcher_payload() {
        let temp = tempfile::tempdir().unwrap();
        let payload = temp.path().join("payload.txt");
        fs::write(&payload, b"payload").unwrap();
        let fetcher = fixed_fetcher("file-src", &file_url(&payload));
        let root = root_derivation(vec![crunch_glue::Input::Derivation(Box::new(fetcher))]);

        let planned =
            plan_source_bundle_from_derivations(&[("default".to_string(), root.clone())], &[], "/mantle/store")
                .unwrap();
        let exported =
            export_source_bundle_from_derivations(&[("default".to_string(), root)], &[], "/mantle/store").unwrap();

        assert!(planned.records[0].files.is_empty());
        assert_eq!(exported.records[0].kind, SourceRecordKind::FixedUrl);
        assert_eq!(exported.records[0].files.len(), 1);
        assert_eq!(exported.records[0].files[0].path, "payload.txt");
        assert_eq!(exported.records[0].payload_bytes, 7);
        assert_eq!(
            exported.records[0].metadata.get(RECORD_METADATA_URL_KEY),
            planned.records[0].metadata.get(RECORD_METADATA_URL_KEY)
        );
    }

    #[test]
    fn source_bundle_export_materializes_local_vcs_snapshot_without_dot_git() {
        let temp = tempfile::tempdir().unwrap();
        let checkout = temp.path().join("checkout");
        fs::create_dir_all(checkout.join("src")).unwrap();
        fs::create_dir_all(checkout.join(DOT_GIT_DIR_NAME)).unwrap();
        fs::write(checkout.join("src/main.txt"), b"hello").unwrap();
        fs::write(checkout.join(DOT_GIT_DIR_NAME).join("config"), b"secret").unwrap();
        let mut git = fixed_fetcher("repo-src", &file_url(&checkout));
        git.env.insert(FETCH_ENV_TYPE_KEY.to_string(), FETCH_ENV_TYPE_GIT.to_string());
        git.env.insert(FETCH_ENV_REV_KEY.to_string(), "refs/heads/main".to_string());
        let root = root_derivation(vec![crunch_glue::Input::Derivation(Box::new(git))]);

        let exported =
            export_source_bundle_from_derivations(&[("default".to_string(), root)], &[], "/mantle/store").unwrap();

        assert_eq!(exported.records[0].kind, SourceRecordKind::VcsSnapshot);
        let paths = exported.records[0].files.iter().map(|file| file.path.as_str()).collect::<Vec<_>>();
        assert_eq!(paths, vec!["src/main.txt"]);
        assert_eq!(exported.records[0].metadata.get(FETCH_ENV_REV_KEY).map(String::as_str), Some("refs/heads/main"));
    }

    #[test]
    fn source_bundle_export_rejects_remote_fetcher_without_local_payload() {
        let fetcher = fixed_fetcher("remote-src", "https://example.invalid/source.tar.gz");
        let root = root_derivation(vec![crunch_glue::Input::Derivation(Box::new(fetcher))]);

        let err =
            export_source_bundle_from_derivations(&[("default".to_string(), root)], &[], "/mantle/store").unwrap_err();
        assert!(err.to_string().contains("cannot materialize non-local source URL"));
    }

    #[test]
    fn source_bundle_rejects_unfixed_builtin_fetcher_in_build_root() {
        let mut fetcher = fixed_fetcher("bad-src", "https://example.invalid/bad.tar.gz");
        fetcher.fixed_output = None;
        let root = root_derivation(vec![crunch_glue::Input::Derivation(Box::new(fetcher))]);

        let err =
            plan_source_bundle_from_derivations(&[("default".to_string(), root)], &[], "/mantle/store").unwrap_err();
        assert!(err.to_string().contains("cannot be source-bundled without fixed_output"));
    }

    #[test]
    fn source_bundle_import_and_verify_are_idempotent() {
        let temp = tempfile::tempdir().unwrap();
        let source_root = temp.path().join("source");
        write_fixture(&source_root);
        let spec = SourceSpec {
            kind: SourceRecordKind::LocalPath,
            identity: "fixture".to_string(),
            path: source_root,
            adapter: None,
        };
        let manifest = plan_source_bundle(&[spec], "/mantle/store").unwrap();
        let state_dir = temp.path().join("state");
        let first = import_source_bundle(&manifest, &state_dir, true).unwrap();
        assert_eq!(first.imported_count, 1);
        assert!(first.pinned);
        let second = import_source_bundle(&manifest, &state_dir, true).unwrap();
        assert_eq!(second.imported_count, 0);
        assert_eq!(second.skipped_present_count, 1);
        let verify = verify_source_bundle_state(&manifest, &state_dir).unwrap();
        assert_eq!(verify.ready_class, SourceReadiness::Ready);
    }

    #[test]
    fn source_bundle_verify_reports_missing_state() {
        let temp = tempfile::tempdir().unwrap();
        write_fixture(temp.path());
        let spec = SourceSpec {
            kind: SourceRecordKind::LocalPath,
            identity: "fixture".to_string(),
            path: temp.path().to_path_buf(),
            adapter: None,
        };
        let manifest = plan_source_bundle(&[spec], "/mantle/store").unwrap();
        let verify = verify_source_bundle_state(&manifest, &temp.path().join("missing-state")).unwrap();
        assert_eq!(verify.ready_class, SourceReadiness::Missing);
        assert_eq!(verify.missing_records, vec!["fixture".to_string()]);
    }

    #[test]
    #[cfg(unix)]
    fn source_bundle_rejects_unsafe_symlink() {
        use std::os::unix::fs::symlink;
        let temp = tempfile::tempdir().unwrap();
        symlink("../escape", temp.path().join("bad-link")).unwrap();
        let spec = SourceSpec {
            kind: SourceRecordKind::LocalPath,
            identity: "fixture".to_string(),
            path: temp.path().to_path_buf(),
            adapter: None,
        };
        let err = plan_source_bundle(&[spec], "/mantle/store").unwrap_err();
        assert!(err.to_string().contains("unsafe source symlink"));
    }
}
