// machine-artifact-public: source-bundle.plan-report
// machine-artifact-public: source-bundle.verify-report
// machine-artifact-public: source-bundle.offline-preflight-report
// machine-artifact-public: source-bundle.manifest-artifacts
// machine-artifact-public: source-bundle.self-build-hydration-report
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use data_encoding::HEXLOWER;
use nix_compat::nixhash::HashAlgo;
use nix_compat::nixhash::NixHash;
use serde::Deserialize;
use serde::Serialize;
use url::Url;

use crate::errors::RunError;

pub const SOURCE_BUNDLE_FORMAT: &str = "mantle-source-bundle-v1";
pub const SOURCE_OFFLINE_PREFLIGHT_FORMAT: &str = "mantle-source-offline-preflight-v1";
pub const BOOTSTRAP_SOURCE_PROFILE_FORMAT: &str = "mantle-bootstrap-source-profile-v1";
pub const SELF_BUILD_HYDRATION_REPORT_FORMAT: &str = "mantle-self-build-source-hydration-v1";
pub const SOURCE_BUNDLE_VERSION: u32 = 1;
pub const SOURCE_BUNDLE_NON_CLAIM: &str =
    "source bundle evidence proves declared source/input availability and identity only";
pub const BOOTSTRAP_SOURCE_PROFILE_NON_CLAIM: &str =
    "bootstrap source profile proves source/input availability and identity only";
pub const SOURCE_NEXT_ACTION_EXPORT_IMPORT_PIN: &str = "mantle source bundle export --build-root <root.ncl> --to <bundle.json>; mantle source bundle import --from <bundle.json> --pin";
pub const SOURCE_NEXT_ACTION_REEXPORT_IMPORT_PIN: &str = "mantle source bundle export --build-root <root.ncl> --to <bundle.json>; mantle source bundle import --from <bundle.json> --pin";
pub const SOURCE_NEXT_ACTION_PIN_IMPORTED: &str = "mantle source bundle import --from <bundle.json> --pin";
pub const SOURCE_NEXT_ACTION_INSPECT_ADAPTER: &str =
    "inspect source-bundle adapter metadata and use a supported offline source adapter";
pub const SOURCE_NEXT_ACTION_TRUST_PROVENANCE: &str =
    "import source state carrying trusted-provenance metadata, or rerun an explicit non-offline workflow";
pub const SOURCE_NEXT_ACTION_DECLARE_SOURCE: &str = "export/import/pin the required source bundle, or rerun without --offline-source-preflight when live fetches are intended";
pub const MAX_SOURCE_RECORDS: usize = 65_536;
pub const MAX_SOURCE_FILES_PER_RECORD: usize = 262_144;
pub const MAX_SOURCE_FILE_BYTES: u64 = 67_108_864;
pub const MAX_SOURCE_TOTAL_BYTES: u64 = 1_099_511_627_776;
pub const MAX_SOURCE_ID_BYTES: usize = 512;
pub const MAX_ADAPTER_METADATA_BYTES: usize = 8_192;
pub const MAX_SOURCE_RECORD_METADATA_BYTES: usize = 16_384;
pub const MAX_DERIVED_SOURCE_WALK_NODES: usize = 65_536;

const DERIVATION_FILE_DEPTH_MAX: u32 = 128;
const DERIVATION_FILE_COUNT_MAX: u32 = 4_096;
const DERIVATION_FILE_PATH_BYTES_MAX: usize = 4_096;
const DERIVATION_FILE_EXTENSION: &str = "ncl";
const SINGLE_DERIVATION_FILE_ROOT_COUNT: usize = 1;
#[cfg(test)]
const MAX_DERIVED_SOURCE_WALK_ITEMS: usize = MAX_DERIVED_SOURCE_WALK_NODES.saturating_add(MAX_SOURCE_RECORDS);
const MAX_GIT_REF_INDIRECTIONS: usize = 16;
const BOOTSTRAP_BASE_RECORD_COUNT: usize = 2;
const REQUIRED_HYDRATION_RECORD_CLASS_COUNT: usize = 3;
const REQUIRED_HYDRATION_RECORD_COUNT_PER_CLASS: usize = 1;
const OFFLINE_BLOCKER_CLASS_COUNT: usize = 6;
const BLAKE3_HEX_BYTES: usize = 64;
const GIT_OBJECT_ID_HEX_BYTES: usize = 40;
const SYMLINK_PAYLOAD_BYTES: u64 = 0;
const MIN_SOURCE_FILE_CHUNK_COUNT: u32 = 2;
#[cfg(unix)]
const UNIX_EXECUTABLE_FILE_MODE: u32 = 0o755;
#[cfg(unix)]
const UNIX_REGULAR_FILE_MODE: u32 = 0o644;

const ADAPTER_EXTRA_TRUSTED_PROVENANCE_KEY: &str = "trusted-provenance";
const ADAPTER_EXTRA_UNSUPPORTED_KEY: &str = "unsupported";
const ADAPTER_EXTRA_TRUE_VALUE: &str = "true";
const BUILTIN_FETCHURL_BUILDER: &str = "builtin:fetchurl";
const DERIVED_FIXED_URL_ID_PREFIX: &str = "fixed-url";
const DERIVED_STORE_PATH_ID_PREFIX: &str = "store-path";
const DERIVED_VCS_ID_PREFIX: &str = "vcs-snapshot";
const FETCH_ENV_EXECUTABLE_KEY: &str = "executable";
const FETCH_ENV_FETCH_POLICY_KEY: &str = "fetch_policy";
const FETCH_ENV_REV_KEY: &str = "rev";
const FETCH_ENV_TYPE_KEY: &str = "type";
const FETCH_ENV_UNPACK_KEY: &str = "unpack";
const FETCH_ENV_URL_KEY: &str = "url";
const FETCH_ENV_TYPE_GIT: &str = "git";
const DOT_GIT_DIR_NAME: &str = ".git";
const FILE_URL_SCHEME: &str = "file";
const GIT_DIR_POINTER_PREFIX: &str = "gitdir:";
const GIT_HEAD_REF: &str = "HEAD";
const GIT_PACKED_REFS_FILE: &str = "packed-refs";
const GIT_REF_PREFIX: &str = "ref: ";
const GIT_REFS_PREFIX: &str = "refs/";
const PACKED_REF_COMMENT_PREFIX: char = '#';
const PACKED_REF_PEELED_PREFIX: char = '^';
const RECORD_CONTENT_FILES_MARKER: &[u8] = b"files\0";
const RECORD_CONTENT_KIND_MARKER: &[u8] = b"kind\0";
const RECORD_CONTENT_METADATA_MARKER: &[u8] = b"metadata\0";
const SOURCE_OFFLINE_PREFLIGHT_EMPTY_MARKER: &[u8] = b"mantle-source-offline-preflight-empty\0";
const SOURCE_OFFLINE_PREFLIGHT_STATE_MARKER: &[u8] = b"mantle-source-offline-preflight-state\0";
const RECORD_METADATA_BUILDER_KEY: &str = "builder";
const RECORD_METADATA_HASH_ALGO_KEY: &str = "hash_algo";
const RECORD_METADATA_HASH_KEY: &str = "hash";
const RECORD_METADATA_HASH_MODE_KEY: &str = "hash_mode";
const RECORD_METADATA_NAME_KEY: &str = "name";
const RECORD_METADATA_PAYLOAD_ENCODING_KEY: &str = "payload_encoding";
const RECORD_METADATA_SOURCE_KIND_KEY: &str = "source_kind";
const RECORD_METADATA_STORE_PATH_KEY: &str = "store_path";
const RECORD_METADATA_URL_KEY: &str = "url";
const RECORD_METADATA_PROFILE_MODE_KEY: &str = "bootstrap_profile_mode";
const RECORD_METADATA_PROFILE_CLASS_KEY: &str = "bootstrap_profile_class";
const RECORD_METADATA_PROVIDER_KIND_KEY: &str = "provider_kind";
const RECORD_METADATA_PROVIDER_SCHEMA_KEY: &str = "provider_schema_version";
const TARBALL_ARCHIVE_PAYLOAD_ENCODING: &str = "tarball-archive-v1";

const BOOTSTRAP_PROVIDER_KIND_LEGACY_SEED: &str = "musl.cc-native-reduced-v1";
const BOOTSTRAP_PROVIDER_KIND_SOURCE_ROOT: &str = "source-root-v1";
const BOOTSTRAP_PROFILE_CLASS_PROVIDER_ARCHIVE: &str = "provider-archive";
const BOOTSTRAP_PROFILE_CLASS_PROVIDER_MANIFEST: &str = "provider-manifest";
const BOOTSTRAP_PROFILE_CLASS_BOOTSTRAP_SOURCE: &str = "bootstrap-source";
const BOOTSTRAP_PROFILE_CLASS_MANTLE_SOURCE: &str = "mantle-source";
const BOOTSTRAP_PROFILE_CLASS_VENDOR_DEPS: &str = "vendored-cargo-inputs";
const BOOTSTRAP_PROFILE_CLASS_TOOLCHAIN_SOURCE_ROOT: &str = "toolchain-source-root";
const BOOTSTRAP_PROFILE_CLASS_PROOF_INPUT: &str = "proof-input";
const BOOTSTRAP_PROFILE_INDEX_WIDTH: usize = 4;

const RECORD_SPEC_SEPARATOR: char = ':';
const SOURCE_STATE_DIR: &str = "source-bundles";
const SOURCE_RECORDS_DIR: &str = "records";
const SOURCE_PINS_DIR: &str = "pins";
const TEMP_FILE_EXTENSION: &str = "tmp";
const HYDRATION_STAGING_PREFIX: &str = ".mantle-self-build-hydration-";
const VENDOR_DEPS_DIR_NAME: &str = "vendor-deps";

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chunk_index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chunk_count: Option<u32>,
    pub blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceFileType {
    Regular,
    Symlink,
}

fn empty_source_record_metadata() -> BTreeMap<String, String> {
    BTreeMap::new()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceRecord {
    pub kind: SourceRecordKind,
    pub identity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store_prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adapter: Option<SourceAdapterMetadata>,
    #[serde(default = "empty_source_record_metadata", skip_serializing_if = "BTreeMap::is_empty")]
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
    pub untrusted_records: Vec<String>,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceOfflinePreflightReport {
    pub format: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manifest_blake3: Option<String>,
    pub source_state_blake3: String,
    pub ready_class: SourceReadiness,
    pub record_count: u32,
    pub missing_records: Vec<String>,
    pub stale_records: Vec<String>,
    pub unsupported_records: Vec<String>,
    pub untrusted_records: Vec<String>,
    pub network_required_records: Vec<String>,
    pub unpinned_records: Vec<String>,
    pub next_actions: Vec<SourceOfflinePreflightNextAction>,
    pub records: Vec<SourceRecordSummary>,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceOfflinePreflightNextAction {
    pub blocker_class: &'static str,
    pub command_hint: &'static str,
    pub description: &'static str,
}

#[derive(Debug)]
pub struct SourceFetchOverridePlan {
    pub report: SourceOfflinePreflightReport,
    pub overrides: Vec<crunch_build::FetchSourceOverride>,
    _scratch_dirs: Vec<tempfile::TempDir>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BootstrapSourceBundleMode {
    LegacySeed,
    SourceRoot,
    SelfBuildProof,
    FreshCloneInputs,
    FreshCloneFixedPoint,
}

impl BootstrapSourceBundleMode {
    pub fn parse(value: &str) -> Result<Self, RunError> {
        match value {
            "legacy-seed" => Ok(Self::LegacySeed),
            "source-root" => Ok(Self::SourceRoot),
            "self-build-proof" => Ok(Self::SelfBuildProof),
            "fresh-clone-inputs" => Ok(Self::FreshCloneInputs),
            "fresh-clone-fixed-point" => Ok(Self::FreshCloneFixedPoint),
            other => Err(RunError::Internal(format!("unsupported bootstrap source profile mode '{other}'"))),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::LegacySeed => "legacy-seed",
            Self::SourceRoot => "source-root",
            Self::SelfBuildProof => "self-build-proof",
            Self::FreshCloneInputs => "fresh-clone-inputs",
            Self::FreshCloneFixedPoint => "fresh-clone-fixed-point",
        }
    }

    fn expected_provider_kind(self) -> &'static str {
        match self {
            Self::LegacySeed | Self::SelfBuildProof | Self::FreshCloneInputs | Self::FreshCloneFixedPoint => {
                BOOTSTRAP_PROVIDER_KIND_LEGACY_SEED
            }
            Self::SourceRoot => BOOTSTRAP_PROVIDER_KIND_SOURCE_ROOT,
        }
    }

    fn requires_full_self_build_inputs(self) -> bool {
        matches!(self, Self::SelfBuildProof)
    }

    fn requires_vendor_inputs(self) -> bool {
        matches!(self, Self::SelfBuildProof | Self::FreshCloneInputs | Self::FreshCloneFixedPoint)
    }

    fn requires_bootstrap_sources(self) -> bool {
        !matches!(self, Self::FreshCloneInputs | Self::FreshCloneFixedPoint)
    }

    fn requires_supplemental_fetch_closure(self) -> bool {
        matches!(self, Self::FreshCloneFixedPoint)
    }
}

#[derive(Debug, Clone)]
pub struct BootstrapSourceBundleProfileInput {
    pub mode: BootstrapSourceBundleMode,
    pub provider_archive: PathBuf,
    pub provider_manifest: PathBuf,
    pub bootstrap_sources: Vec<PathBuf>,
    pub mantle_source: Option<PathBuf>,
    pub vendor_deps: Option<PathBuf>,
    pub toolchain_source_root: Option<PathBuf>,
    pub proof_inputs: Vec<PathBuf>,
    pub supplemental_records: Vec<SourceRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BootstrapSourceBundleProfileReport {
    pub format: &'static str,
    pub mode: BootstrapSourceBundleMode,
    pub manifest_blake3: String,
    pub required_record_count: u32,
    pub provider_kind: String,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SelfBuildHydrationReport {
    pub format: &'static str,
    pub manifest_blake3: String,
    pub vendor_content_blake3: String,
    pub provider_archive_content_blake3: String,
    pub imported_record_count: u32,
    pub existing_record_count: u32,
    pub pinned: bool,
    pub non_claim: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SelfBuildHydrationPlan {
    vendor_record_index: usize,
    provider_archive_record_index: usize,
    provider_manifest_record_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BootstrapProviderProfileMetadata {
    provider_kind: String,
    schema_version: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceFetchBlocker {
    MissingSourceState,
    StaleSourceState,
    UnsupportedSourceAdapter,
    UntrustedSourceAdapter,
    NetworkRequiredSource,
    UnpinnedSourceState,
    MissingMaterializedPayload,
}

impl SourceFetchBlocker {
    pub fn reason_code(self) -> &'static str {
        match self {
            Self::MissingSourceState => "missing-source-state",
            Self::StaleSourceState => "stale-source-state",
            Self::UnsupportedSourceAdapter => "unsupported-source-adapter",
            Self::UntrustedSourceAdapter => "untrusted-source-adapter",
            Self::NetworkRequiredSource => "network-required-source",
            Self::UnpinnedSourceState => "unpinned-source-state",
            Self::MissingMaterializedPayload => "missing-materialized-payload",
        }
    }
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
    Untrusted,
    NetworkRequired,
    Unpinned,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSpec {
    pub kind: SourceRecordKind,
    pub identity: String,
    pub path: PathBuf,
    pub adapter: Option<SourceAdapterMetadata>,
}

struct SourceRecordPathRequest<'a> {
    kind: SourceRecordKind,
    identity: String,
    path: &'a Path,
    store_prefix: &'a str,
    metadata: BTreeMap<String, String>,
    adapter: Option<SourceAdapterMetadata>,
    is_skipping_git_dir: bool,
}

struct BootstrapProfileRecordRequest<'a> {
    kind: SourceRecordKind,
    identity: String,
    path: &'a Path,
    mode: BootstrapSourceBundleMode,
    class: &'a str,
    provider_metadata: Option<&'a BootstrapProviderProfileMetadata>,
    store_prefix: &'a str,
}

struct BootstrapProfileSequenceRequest<'a> {
    paths: &'a [PathBuf],
    kind: SourceRecordKind,
    mode: BootstrapSourceBundleMode,
    class: &'a str,
    store_prefix: &'a str,
}

struct SourceEntryCollection {
    files: Vec<SourceFileEntry>,
    payload_bytes: u64,
    visited_nodes_len: usize,
}

struct Blake3HexValidation<'a> {
    value: &'a str,
    label: &'a str,
}

struct OfflineBlockerSets<'a> {
    missing_ids: &'a [String],
    stale_ids: &'a [String],
    unsupported_ids: &'a [String],
    untrusted_ids: &'a [String],
    network_required_ids: &'a [String],
    unpinned_ids: &'a [String],
}

struct OfflineNextActionRule {
    is_enabled: bool,
    blocker_class: &'static str,
    command_hint: &'static str,
    description: &'static str,
}

struct GitPackedRefMatch<'a> {
    line: &'a str,
    revision: &'a str,
}

struct StorePathLookup<'a> {
    store_prefix: &'a str,
    logical_store_path: &'a str,
}

struct StorePathSourceRequest<'a> {
    source_path: &'a str,
    store_prefix: &'a str,
}

struct SourceBundleCliContext<'a> {
    state_dir: &'a Path,
    store_prefix: &'a str,
    is_json_output: bool,
}

#[cfg(test)]
enum DerivationSourceWalkItem<'a> {
    Derivation(&'a crunch_glue::CrunchDerivation),
    StorePath(&'a str),
}

struct DerivationFileSourceWalker<'a> {
    import_paths: &'a [OsString],
    store_prefix: &'a str,
    records: Vec<SourceRecord>,
    visiting: BTreeSet<PathBuf>,
    completed_outputs: BTreeMap<PathBuf, BTreeSet<String>>,
    visited_derivation_count: usize,
    resolved_file_count: u32,
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

#[cfg(test)]
pub fn plan_source_bundle_from_derivations(
    roots: &[(String, crunch_glue::CrunchDerivation)],
    specs: &[SourceSpec],
    store_prefix: &str,
) -> Result<SourceBundleManifest, RunError> {
    let mut records = canonicalize_source_specs(specs, store_prefix)?;
    records.extend(collect_build_source_records(roots, store_prefix)?);
    assemble_source_bundle(records, store_prefix)
}

// r[impl bootstrap_inventory.offline_bootstrap_source_bundles]
pub fn plan_bootstrap_source_bundle_profile(
    input: &BootstrapSourceBundleProfileInput,
    store_prefix: &str,
) -> Result<SourceBundleManifest, RunError> {
    validate_bootstrap_profile_input(input, store_prefix)?;
    assert!(store_prefix.starts_with('/'));
    assert!(!input.mode.requires_bootstrap_sources() || !input.bootstrap_sources.is_empty());
    let provider_metadata = read_bootstrap_provider_profile_metadata(&input.provider_manifest)?;
    validate_bootstrap_provider_kind(input.mode, &provider_metadata.provider_kind)?;
    assert_eq!(provider_metadata.provider_kind, input.mode.expected_provider_kind());
    let profile_entries = bootstrap_profile_records_capacity(input)?;
    let mut records = Vec::with_capacity(profile_entries);
    append_bootstrap_provider_records(&mut records, input, store_prefix, &provider_metadata)?;
    append_bootstrap_profile_sequence(&mut records, BootstrapProfileSequenceRequest {
        paths: &input.bootstrap_sources,
        kind: SourceRecordKind::BootstrapArchive,
        mode: input.mode,
        class: BOOTSTRAP_PROFILE_CLASS_BOOTSTRAP_SOURCE,
        store_prefix,
    })?;
    append_optional_bootstrap_profile_records(&mut records, input, store_prefix)?;
    append_supplemental_profile_records(&mut records, input)?;
    append_bootstrap_profile_sequence(&mut records, BootstrapProfileSequenceRequest {
        paths: &input.proof_inputs,
        kind: SourceRecordKind::ProofInput,
        mode: input.mode,
        class: BOOTSTRAP_PROFILE_CLASS_PROOF_INPUT,
        store_prefix,
    })?;
    assert!(records.len() <= profile_entries);
    assemble_source_bundle(records, store_prefix)
}

fn bootstrap_profile_records_capacity(input: &BootstrapSourceBundleProfileInput) -> Result<usize, RunError> {
    let mut profile_entries = BOOTSTRAP_BASE_RECORD_COUNT;
    profile_entries = profile_entries
        .checked_add(input.bootstrap_sources.len())
        .ok_or_else(|| RunError::Internal("bootstrap profile record capacity overflow".to_string()))?;
    profile_entries = profile_entries
        .checked_add(input.proof_inputs.len())
        .ok_or_else(|| RunError::Internal("bootstrap profile record capacity overflow".to_string()))?;
    profile_entries = profile_entries
        .checked_add(input.supplemental_records.len())
        .ok_or_else(|| RunError::Internal("bootstrap profile record capacity overflow".to_string()))?;
    for is_present in [
        input.mantle_source.is_some(),
        input.vendor_deps.is_some(),
        input.toolchain_source_root.is_some(),
    ] {
        if is_present {
            profile_entries = profile_entries
                .checked_add(1)
                .ok_or_else(|| RunError::Internal("bootstrap profile record capacity overflow".to_string()))?;
        }
    }
    if profile_entries > MAX_SOURCE_RECORDS {
        return Err(RunError::Internal(format!("bootstrap profile source count exceeds {MAX_SOURCE_RECORDS}")));
    }
    assert!(profile_entries >= BOOTSTRAP_BASE_RECORD_COUNT);
    assert!(profile_entries <= MAX_SOURCE_RECORDS);
    Ok(profile_entries)
}

fn append_supplemental_profile_records(
    records: &mut Vec<SourceRecord>,
    input: &BootstrapSourceBundleProfileInput,
) -> Result<(), RunError> {
    let initial_len = records.len();
    records
        .try_reserve(input.supplemental_records.len())
        .map_err(|error| RunError::Internal(format!("reserving supplemental profile records: {error}")))?;
    records.extend(input.supplemental_records.iter().cloned());
    assert_eq!(records.len(), initial_len.saturating_add(input.supplemental_records.len()));
    assert!(records.len() <= MAX_SOURCE_RECORDS);
    Ok(())
}

fn append_bootstrap_provider_records(
    records: &mut Vec<SourceRecord>,
    input: &BootstrapSourceBundleProfileInput,
    store_prefix: &str,
    provider_metadata: &BootstrapProviderProfileMetadata,
) -> Result<(), RunError> {
    assert!(records.is_empty());
    records.push(bootstrap_profile_record(BootstrapProfileRecordRequest {
        kind: SourceRecordKind::BootstrapArchive,
        identity: "bootstrap-provider-archive".to_string(),
        path: &input.provider_archive,
        mode: input.mode,
        class: BOOTSTRAP_PROFILE_CLASS_PROVIDER_ARCHIVE,
        provider_metadata: None,
        store_prefix,
    })?);
    records.push(bootstrap_profile_record(BootstrapProfileRecordRequest {
        kind: SourceRecordKind::ProviderManifest,
        identity: "bootstrap-provider-manifest".to_string(),
        path: &input.provider_manifest,
        mode: input.mode,
        class: BOOTSTRAP_PROFILE_CLASS_PROVIDER_MANIFEST,
        provider_metadata: Some(provider_metadata),
        store_prefix,
    })?);
    assert_eq!(records.len(), BOOTSTRAP_BASE_RECORD_COUNT);
    Ok(())
}

fn append_bootstrap_profile_sequence(
    records: &mut Vec<SourceRecord>,
    request: BootstrapProfileSequenceRequest<'_>,
) -> Result<(), RunError> {
    let initial_records_len = records.len();
    records
        .try_reserve(request.paths.len())
        .map_err(|err| RunError::Internal(format!("reserving bootstrap profile records: {err}")))?;
    for (index, path) in request.paths.iter().enumerate() {
        records.push(bootstrap_profile_record(BootstrapProfileRecordRequest {
            kind: request.kind.clone(),
            identity: bootstrap_indexed_identity(request.class, index)?,
            path,
            mode: request.mode,
            class: request.class,
            provider_metadata: None,
            store_prefix: request.store_prefix,
        })?);
    }
    assert!(records.len() >= initial_records_len);
    assert!(records.len() <= MAX_SOURCE_RECORDS);
    Ok(())
}

fn append_optional_bootstrap_profile_records(
    records: &mut Vec<SourceRecord>,
    input: &BootstrapSourceBundleProfileInput,
    store_prefix: &str,
) -> Result<(), RunError> {
    let initial_records_len = records.len();
    let optional_records = [
        (
            SourceRecordKind::LocalPath,
            "mantle-source-tree",
            input.mantle_source.as_deref(),
            BOOTSTRAP_PROFILE_CLASS_MANTLE_SOURCE,
        ),
        (
            SourceRecordKind::PackageMirror,
            "vendored-cargo-inputs",
            input.vendor_deps.as_deref(),
            BOOTSTRAP_PROFILE_CLASS_VENDOR_DEPS,
        ),
        (
            SourceRecordKind::ToolchainSourceRoot,
            "bootstrap-toolchain-source-root",
            input.toolchain_source_root.as_deref(),
            BOOTSTRAP_PROFILE_CLASS_TOOLCHAIN_SOURCE_ROOT,
        ),
    ];
    records
        .try_reserve(optional_records.len())
        .map_err(|err| RunError::Internal(format!("reserving optional bootstrap profile records: {err}")))?;
    for (kind, identity, path, class) in optional_records {
        let Some(path) = path else { continue };
        records.push(bootstrap_profile_record(BootstrapProfileRecordRequest {
            kind,
            identity: identity.to_string(),
            path,
            mode: input.mode,
            class,
            provider_metadata: None,
            store_prefix,
        })?);
    }
    assert!(records.len() >= initial_records_len);
    assert!(records.len() <= MAX_SOURCE_RECORDS);
    Ok(())
}

pub fn bootstrap_source_bundle_profile_report(
    manifest: &SourceBundleManifest,
    mode: BootstrapSourceBundleMode,
) -> Result<BootstrapSourceBundleProfileReport, RunError> {
    validate_manifest(manifest)?;
    assert_eq!(manifest.format, SOURCE_BUNDLE_FORMAT);
    assert_eq!(manifest.non_claim, SOURCE_BUNDLE_NON_CLAIM);
    let provider = manifest
        .records
        .iter()
        .find(|record| {
            record.metadata.get(RECORD_METADATA_PROFILE_CLASS_KEY).map(String::as_str)
                == Some(BOOTSTRAP_PROFILE_CLASS_PROVIDER_MANIFEST)
        })
        .ok_or_else(|| RunError::Internal("bootstrap profile missing provider manifest record".to_string()))?;
    let provider_kind = provider
        .metadata
        .get(RECORD_METADATA_PROVIDER_KIND_KEY)
        .ok_or_else(|| RunError::Internal("bootstrap profile provider manifest missing provider kind".to_string()))?
        .clone();
    Ok(BootstrapSourceBundleProfileReport {
        format: BOOTSTRAP_SOURCE_PROFILE_FORMAT,
        mode,
        manifest_blake3: manifest.manifest_blake3.clone(),
        required_record_count: checked_u32(manifest.records.len(), "bootstrap profile record count")?,
        provider_kind,
        non_claim: BOOTSTRAP_SOURCE_PROFILE_NON_CLAIM,
    })
}

#[cfg(test)]
fn export_source_bundle_from_derivations(
    roots: &[(String, crunch_glue::CrunchDerivation)],
    specs: &[SourceSpec],
    store_prefix: &str,
) -> Result<SourceBundleManifest, RunError> {
    export_source_bundle_from_derivations_with_imported(roots, specs, store_prefix, &[])
}

#[cfg(test)]
pub fn export_source_bundle_from_derivations_with_state(
    roots: &[(String, crunch_glue::CrunchDerivation)],
    specs: &[SourceSpec],
    store_prefix: &str,
    state_dir: &Path,
) -> Result<SourceBundleManifest, RunError> {
    let available_sources = read_imported_source_records(state_dir)?;
    export_source_bundle_from_derivations_with_imported(roots, specs, store_prefix, &available_sources)
}

#[cfg(test)]
pub fn export_source_bundle_from_derivations_with_connected_fetch(
    roots: &[(String, crunch_glue::CrunchDerivation)],
    specs: &[SourceSpec],
    store_prefix: &str,
    state_dir: &Path,
) -> Result<SourceBundleManifest, RunError> {
    let available_sources = read_imported_source_records(state_dir)?;
    let mut records = canonicalize_source_specs(specs, store_prefix)?;
    let expected = normalize_source_records(collect_build_source_records(roots, store_prefix)?)?;
    records.extend(materialize_export_records_with_connected_fetch(&expected, &available_sources)?);
    assemble_source_bundle(records, store_prefix)
}

#[cfg(test)]
fn export_source_bundle_from_derivations_with_imported(
    roots: &[(String, crunch_glue::CrunchDerivation)],
    specs: &[SourceSpec],
    store_prefix: &str,
    imported_records: &[SourceRecord],
) -> Result<SourceBundleManifest, RunError> {
    let mut records = canonicalize_source_specs(specs, store_prefix)?;
    let expected = normalize_source_records(collect_build_source_records(roots, store_prefix)?)?;
    records.extend(materialize_export_records(&expected, imported_records)?);
    assemble_source_bundle(records, store_prefix)
}

#[cfg(test)]
pub fn collect_build_source_records(
    roots: &[(String, crunch_glue::CrunchDerivation)],
    store_prefix: &str,
) -> Result<Vec<SourceRecord>, RunError> {
    if !store_prefix.starts_with('/') {
        return Err(RunError::Internal(format!("store prefix must be absolute: {store_prefix}")));
    }
    if roots.len() > MAX_DERIVED_SOURCE_WALK_NODES {
        return Err(RunError::Internal(format!(
            "derived source roots exceed {MAX_DERIVED_SOURCE_WALK_NODES} derivation nodes"
        )));
    }
    assert!(store_prefix.starts_with('/'));
    assert!(roots.len() <= MAX_DERIVED_SOURCE_WALK_NODES);
    let mut pending = Vec::with_capacity(roots.len());
    for (_, root) in roots.iter().rev() {
        pending.push(DerivationSourceWalkItem::Derivation(root));
    }
    let mut records = Vec::new();
    walk_derivation_source_records(&mut pending, store_prefix, &mut records)?;
    assert!(pending.is_empty());
    assert!(records.len() <= MAX_SOURCE_RECORDS);
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
    assert!(!records.is_empty());
    assert!(records.len() <= MAX_SOURCE_RECORDS);
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

fn plan_self_build_hydration(
    manifest: &SourceBundleManifest,
    expected_manifest_blake3: &str,
) -> Result<SelfBuildHydrationPlan, RunError> {
    validate_manifest(manifest)?;
    validate_blake3_hex(Blake3HexValidation {
        value: expected_manifest_blake3,
        label: "expected source bundle manifest BLAKE3",
    })?;
    if manifest.manifest_blake3 != expected_manifest_blake3 {
        return Err(RunError::Internal(format!(
            "source bundle manifest BLAKE3 mismatch: expected {expected_manifest_blake3}, got {}",
            manifest.manifest_blake3
        )));
    }
    let vendor_record_index =
        unique_hydration_record_index(manifest, BOOTSTRAP_PROFILE_CLASS_VENDOR_DEPS, SourceRecordKind::PackageMirror)?;
    let provider_archive_record_index = unique_hydration_record_index(
        manifest,
        BOOTSTRAP_PROFILE_CLASS_PROVIDER_ARCHIVE,
        SourceRecordKind::BootstrapArchive,
    )?;
    let provider_manifest_record_index = unique_hydration_record_index(
        manifest,
        BOOTSTRAP_PROFILE_CLASS_PROVIDER_MANIFEST,
        SourceRecordKind::ProviderManifest,
    )?;
    let plan = SelfBuildHydrationPlan {
        vendor_record_index,
        provider_archive_record_index,
        provider_manifest_record_index,
    };
    validate_hydration_profile_linkage(manifest, &plan)?;
    assert!(plan.vendor_record_index < manifest.records.len());
    assert!(plan.provider_archive_record_index < manifest.records.len());
    Ok(plan)
}

fn unique_hydration_record_index(
    manifest: &SourceBundleManifest,
    profile_class: &str,
    expected_kind: SourceRecordKind,
) -> Result<usize, RunError> {
    let matches = manifest
        .records
        .iter()
        .enumerate()
        .filter(|(_, record)| {
            record.metadata.get(RECORD_METADATA_PROFILE_CLASS_KEY).map(String::as_str) == Some(profile_class)
        })
        .collect::<Vec<_>>();
    if matches.len() != REQUIRED_HYDRATION_RECORD_COUNT_PER_CLASS {
        return Err(RunError::Internal(format!(
            "self-build hydration requires exactly one {profile_class} record; found {}",
            matches.len()
        )));
    }
    let (index, record) = matches[0];
    if record.kind != expected_kind {
        return Err(RunError::Internal(format!(
            "self-build hydration {profile_class} record has wrong source kind: {:?}",
            record.kind
        )));
    }
    if record.files.is_empty() {
        return Err(RunError::Internal(format!(
            "self-build hydration {profile_class} record has no materialized payload"
        )));
    }
    assert!(index < manifest.records.len());
    assert!(!record.files.is_empty());
    Ok(index)
}

fn validate_hydration_profile_linkage(
    manifest: &SourceBundleManifest,
    plan: &SelfBuildHydrationPlan,
) -> Result<(), RunError> {
    let records = [
        &manifest.records[plan.vendor_record_index],
        &manifest.records[plan.provider_archive_record_index],
        &manifest.records[plan.provider_manifest_record_index],
    ];
    let mode_text = records[0]
        .metadata
        .get(RECORD_METADATA_PROFILE_MODE_KEY)
        .ok_or_else(|| RunError::Internal("self-build hydration vendor record is missing profile mode".to_string()))?;
    let mode = BootstrapSourceBundleMode::parse(mode_text)?;
    if !matches!(
        mode,
        BootstrapSourceBundleMode::LegacySeed
            | BootstrapSourceBundleMode::SelfBuildProof
            | BootstrapSourceBundleMode::FreshCloneInputs
            | BootstrapSourceBundleMode::FreshCloneFixedPoint
    ) {
        return Err(RunError::Internal(format!(
            "self-build hydration requires a legacy-seed, self-build-proof, fresh-clone-inputs, or fresh-clone-fixed-point profile, got {}",
            mode.as_str()
        )));
    }
    for record in records {
        if record.metadata.get(RECORD_METADATA_PROFILE_MODE_KEY).map(String::as_str) != Some(mode.as_str()) {
            return Err(RunError::Internal(
                "self-build hydration profile records do not share one profile mode".to_string(),
            ));
        }
    }
    let provider_kind = manifest.records[plan.provider_manifest_record_index]
        .metadata
        .get(RECORD_METADATA_PROVIDER_KIND_KEY)
        .ok_or_else(|| {
            RunError::Internal("self-build hydration provider manifest is missing provider kind".to_string())
        })?;
    validate_bootstrap_provider_kind(mode, provider_kind)?;
    assert_eq!(records.len(), REQUIRED_HYDRATION_RECORD_CLASS_COUNT);
    assert!(records.iter().all(|record| !record.files.is_empty()));
    Ok(())
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
    assert!(manifest.records.len() <= MAX_SOURCE_RECORDS);
    assert_eq!(manifest.non_claim, SOURCE_BUNDLE_NON_CLAIM);
    let records_dir = source_records_dir(state_dir);
    fs::create_dir_all(&records_dir)
        .map_err(|err| RunError::Internal(format!("creating source records dir {}: {err}", records_dir.display())))?;
    let mut written_records_len = 0u32;
    let mut skipped_present_records_len = 0u32;
    let mut summaries = Vec::with_capacity(manifest.records.len());
    for record in &manifest.records {
        let summary = summary_for_record(record)?;
        let target = records_dir.join(format!("{}.json", record.content_blake3));
        if target.exists() {
            skipped_present_records_len = skipped_present_records_len
                .checked_add(1)
                .ok_or_else(|| RunError::Internal("present source record count overflow".to_string()))?;
        } else {
            write_record_atomically(&target, record)?;
            written_records_len = written_records_len
                .checked_add(1)
                .ok_or_else(|| RunError::Internal("imported source record count overflow".to_string()))?;
        }
        summaries.push(summary);
    }
    if pin {
        write_pin_atomically(state_dir, manifest)?;
    }
    assert_eq!(summaries.len(), manifest.records.len());
    Ok(SourceBundleImportReport {
        imported_count: written_records_len,
        skipped_present_count: skipped_present_records_len,
        pinned: pin,
        manifest_blake3: manifest.manifest_blake3.clone(),
        records: summaries,
        non_claim: SOURCE_BUNDLE_NON_CLAIM,
    })
}

// r[impl bootstrap_inventory.fresh_clone_source_hydration]
pub fn hydrate_self_build_source_bundle(
    manifest: &SourceBundleManifest,
    expected_manifest_blake3: &str,
    checkout: &Path,
    state_dir: &Path,
) -> Result<SelfBuildHydrationReport, RunError> {
    let plan = plan_self_build_hydration(manifest, expected_manifest_blake3)?;
    let checkout = fs::canonicalize(checkout).map_err(|err| {
        RunError::Internal(format!("canonicalizing hydration checkout {}: {err}", checkout.display()))
    })?;
    if !checkout.is_dir() {
        return Err(RunError::Internal(format!("hydration checkout is not a directory: {}", checkout.display())));
    }
    let vendor_destination = checkout.join(VENDOR_DEPS_DIR_NAME);
    if vendor_destination.exists() {
        return Err(RunError::Internal(format!(
            "self-build hydration refuses to replace existing {}",
            vendor_destination.display()
        )));
    }
    validate_existing_source_state_for_hydration(manifest, state_dir)?;
    let staging = prepare_hydrated_vendor(&checkout, &manifest.records[plan.vendor_record_index])?;
    publish_hydrated_vendor(staging.path(), &vendor_destination)?;
    let import_report = match import_source_bundle(manifest, state_dir, true) {
        Ok(report) => report,
        Err(error) => return rollback_hydrated_vendor(&vendor_destination, error),
    };
    assert!(vendor_destination.is_dir());
    assert!(import_report.pinned);
    Ok(SelfBuildHydrationReport {
        format: SELF_BUILD_HYDRATION_REPORT_FORMAT,
        manifest_blake3: manifest.manifest_blake3.clone(),
        vendor_content_blake3: manifest.records[plan.vendor_record_index].content_blake3.clone(),
        provider_archive_content_blake3: manifest.records[plan.provider_archive_record_index].content_blake3.clone(),
        imported_record_count: import_report.imported_count,
        existing_record_count: import_report.skipped_present_count,
        pinned: import_report.pinned,
        non_claim: SOURCE_BUNDLE_NON_CLAIM,
    })
}

fn validate_existing_source_state_for_hydration(
    manifest: &SourceBundleManifest,
    state_dir: &Path,
) -> Result<(), RunError> {
    for record in &manifest.records {
        let target = source_records_dir(state_dir).join(format!("{}.json", record.content_blake3));
        if target.exists() && read_record(&target)? != *record {
            return Err(RunError::Internal(format!(
                "existing source state conflicts with hydration record {}",
                record.identity
            )));
        }
    }
    let pin_path = source_pins_dir(state_dir).join(format!("{}.json", manifest.manifest_blake3));
    if pin_path.exists() && read_source_bundle(&pin_path)? != *manifest {
        return Err(RunError::Internal("existing source pin conflicts with self-build hydration manifest".to_string()));
    }
    Ok(())
}

fn prepare_hydrated_vendor(checkout: &Path, vendor_record: &SourceRecord) -> Result<tempfile::TempDir, RunError> {
    assert!(checkout.is_dir());
    assert_eq!(vendor_record.kind, SourceRecordKind::PackageMirror);
    let staging = tempfile::Builder::new()
        .prefix(HYDRATION_STAGING_PREFIX)
        .tempdir_in(checkout)
        .map_err(|err| RunError::Internal(format!("creating vendor hydration staging root: {err}")))?;
    let cargo_dir = staging.path().join(".cargo");
    fs::create_dir(&cargo_dir)
        .map_err(|err| RunError::Internal(format!("creating hydration Cargo config dir: {err}")))?;
    copy_hydration_input(&checkout.join("Cargo.lock"), &staging.path().join("Cargo.lock"))?;
    copy_hydration_input(&checkout.join(".cargo").join("vendor-config.toml"), &cargo_dir.join("vendor-config.toml"))?;
    materialize_source_record_payload(vendor_record, &staging.path().join(VENDOR_DEPS_DIR_NAME))?;
    crate::self_build::require_checked_vendor_inputs(staging.path())?;
    assert!(staging.path().join(VENDOR_DEPS_DIR_NAME).is_dir());
    Ok(staging)
}

fn copy_hydration_input(source: &Path, destination: &Path) -> Result<(), RunError> {
    if !source.is_file() {
        return Err(RunError::Internal(format!("self-build hydration input is missing: {}", source.display())));
    }
    fs::copy(source, destination)
        .map_err(|err| RunError::Internal(format!("copying hydration input {}: {err}", source.display())))?;
    Ok(())
}

fn publish_hydrated_vendor(staging_root: &Path, destination: &Path) -> Result<(), RunError> {
    let staged_vendor = staging_root.join(VENDOR_DEPS_DIR_NAME);
    assert!(staged_vendor.is_dir());
    assert!(!destination.as_os_str().is_empty());
    crate::linux_rename::rename_path_no_replace(&staged_vendor, destination).map_err(|err| {
        RunError::Internal(format!(
            "publishing hydrated vendor directory without replacement to {}: {err}",
            destination.display()
        ))
    })
}

fn rollback_hydrated_vendor(
    vendor_destination: &Path,
    import_error: RunError,
) -> Result<SelfBuildHydrationReport, RunError> {
    assert!(vendor_destination.is_dir());
    assert!(!vendor_destination.as_os_str().is_empty());
    match fs::remove_dir_all(vendor_destination) {
        Ok(()) => Err(import_error),
        Err(rollback_error) => Err(RunError::Internal(format!(
            "{import_error}; hydration rollback failed for {}: {rollback_error}",
            vendor_destination.display()
        ))),
    }
}

pub fn verify_source_bundle_state(
    manifest: &SourceBundleManifest,
    state_dir: &Path,
) -> Result<SourceBundleVerifyReport, RunError> {
    validate_manifest(manifest)?;
    assert!(manifest.records.len() <= MAX_SOURCE_RECORDS);
    assert_eq!(manifest.non_claim, SOURCE_BUNDLE_NON_CLAIM);
    let manifest_entries = manifest.records.len();
    let mut missing_record_ids = Vec::with_capacity(manifest_entries);
    let mut stale_record_ids = Vec::with_capacity(manifest_entries);
    let mut rejected_adapter_ids = Vec::with_capacity(manifest_entries);
    let mut untrusted_record_ids = Vec::with_capacity(manifest_entries);
    for record in &manifest.records {
        classify_adapter_readiness(record, &mut rejected_adapter_ids, &mut untrusted_record_ids);
        let record_path = source_records_dir(state_dir).join(format!("{}.json", record.content_blake3));
        if !record_path.exists() {
            missing_record_ids.push(record.identity.clone());
            continue;
        }
        let stored = read_record(&record_path)?;
        if &stored != record {
            stale_record_ids.push(record.identity.clone());
        }
    }
    let ready_class =
        classify_source_state(&missing_record_ids, &stale_record_ids, &rejected_adapter_ids, &untrusted_record_ids);
    Ok(SourceBundleVerifyReport {
        manifest_blake3: manifest.manifest_blake3.clone(),
        ready_class,
        missing_records: missing_record_ids,
        stale_records: stale_record_ids,
        unsupported_records: rejected_adapter_ids,
        untrusted_records: untrusted_record_ids,
        non_claim: SOURCE_BUNDLE_NON_CLAIM,
    })
}

pub fn list_source_bundle(manifest: &SourceBundleManifest) -> Result<SourceBundlePlanReport, RunError> {
    plan_report(manifest)
}

#[cfg(test)]
pub fn offline_preflight_for_derivations(
    roots: &[(String, crunch_glue::CrunchDerivation)],
    state_dir: &Path,
    store_prefix: &str,
) -> Result<SourceOfflinePreflightReport, RunError> {
    let records = collect_build_source_records(roots, store_prefix)?;
    if records.is_empty() {
        return empty_offline_preflight_report();
    }
    let manifest = assemble_source_bundle(records, store_prefix)?;
    offline_preflight_for_manifest(&manifest, state_dir)
}

pub fn offline_preflight_for_file(
    file: &Path,
    import_paths: &[OsString],
    state_dir: &Path,
    store_prefix: &str,
) -> Result<SourceOfflinePreflightReport, RunError> {
    let records = collect_build_source_records_from_files(&[file.to_path_buf()], import_paths, store_prefix)?;
    offline_preflight_for_records(records, state_dir, store_prefix)
}

pub fn offline_preflight_for_build_roots(
    build_roots: &[PathBuf],
    import_paths: &[PathBuf],
    state_dir: &Path,
    store_prefix: &str,
) -> Result<SourceOfflinePreflightReport, RunError> {
    let evaluation_paths = crate::build_cmd::build_import_paths(import_paths)?;
    let records = collect_build_source_records_from_files(build_roots, &evaluation_paths, store_prefix)?;
    offline_preflight_for_records(records, state_dir, store_prefix)
}

fn offline_preflight_for_records(
    records: Vec<SourceRecord>,
    state_dir: &Path,
    store_prefix: &str,
) -> Result<SourceOfflinePreflightReport, RunError> {
    if records.is_empty() {
        return empty_offline_preflight_report();
    }
    let manifest = assemble_source_bundle(records, store_prefix)?;
    offline_preflight_for_manifest(&manifest, state_dir)
}

pub fn offline_preflight_for_manifest(
    manifest: &SourceBundleManifest,
    state_dir: &Path,
) -> Result<SourceOfflinePreflightReport, RunError> {
    validate_manifest(manifest)?;
    let available_sources = read_imported_source_records(state_dir)?;
    let pinned_sources = read_pinned_source_records(state_dir)?;
    classify_offline_preflight(manifest, &available_sources, &pinned_sources)
}

pub fn source_offline_preflight_is_ready(report: &SourceOfflinePreflightReport) -> bool {
    report.ready_class == SourceReadiness::Ready
}

pub fn source_fetch_override_plan_for_file(
    file: &Path,
    import_paths: &[OsString],
    state_dir: &Path,
    store_prefix: &str,
) -> Result<SourceFetchOverridePlan, RunError> {
    let records = collect_build_source_records_from_files(&[file.to_path_buf()], import_paths, store_prefix)?;
    source_fetch_override_plan_for_records(records, state_dir, store_prefix)
}

// r[impl source_transports.source_bundle_realizes_fetcher_inputs]
pub fn full_proof_source_fetch_override_plan(
    state_dir: &Path,
    manifest_blake3: &str,
) -> Result<SourceFetchOverridePlan, RunError> {
    validate_blake3_hex(Blake3HexValidation {
        value: manifest_blake3,
        label: "full-proof source manifest BLAKE3",
    })?;
    let pin_path = source_pins_dir(state_dir).join(format!("{manifest_blake3}.json"));
    let manifest = read_source_bundle(&pin_path).map_err(|error| {
        RunError::Internal(format!("reading pinned full-proof source manifest {}: {error}", pin_path.display()))
    })?;
    let hydration_plan = plan_self_build_hydration(&manifest, manifest_blake3)?;
    let mode = hydration_profile_mode(&manifest, &hydration_plan)?;
    if mode != BootstrapSourceBundleMode::FreshCloneFixedPoint {
        return Err(RunError::Internal(format!(
            "self-build offline source policy requires fresh-clone-fixed-point profile, got {}",
            mode.as_str()
        )));
    }
    let available_sources = read_imported_source_records(state_dir)?;
    let pinned_sources = read_pinned_source_records(state_dir)?;
    let report = classify_offline_preflight(&manifest, &available_sources, &pinned_sources)?;
    if !source_offline_preflight_is_ready(&report) {
        return Err(RunError::Internal(format!("full-proof source preflight is not ready: {:?}", report.ready_class)));
    }
    let (overrides, scratch_dirs) = source_fetch_overrides_for_manifest(
        &manifest,
        &available_sources,
        &pinned_sources,
        &report.source_state_blake3,
    )?;
    if overrides.is_empty() {
        return Err(RunError::Internal("full-proof source manifest has no fixed fetcher records".to_string()));
    }
    assert!(!overrides.is_empty());
    assert_eq!(report.manifest_blake3.as_deref(), Some(manifest_blake3));
    Ok(SourceFetchOverridePlan {
        report,
        overrides,
        _scratch_dirs: scratch_dirs,
    })
}

fn hydration_profile_mode(
    manifest: &SourceBundleManifest,
    plan: &SelfBuildHydrationPlan,
) -> Result<BootstrapSourceBundleMode, RunError> {
    let mode = manifest.records[plan.vendor_record_index]
        .metadata
        .get(RECORD_METADATA_PROFILE_MODE_KEY)
        .ok_or_else(|| RunError::Internal("self-build hydration vendor record is missing profile mode".to_string()))?;
    BootstrapSourceBundleMode::parse(mode)
}

pub fn bootstrap_legacy_seed_fetch_override_plan(
    state_dir: &Path,
    provider_raw_url: &str,
) -> Result<SourceFetchOverridePlan, RunError> {
    if provider_raw_url.is_empty() {
        return Err(RunError::Internal("bootstrap provider raw URL must not be empty".to_string()));
    }
    assert!(!provider_raw_url.is_empty());
    let available_sources = read_imported_source_records(state_dir)?;
    let pinned_sources = read_pinned_source_records(state_dir)?;
    let record = available_sources
        .iter()
        .find(|record| bootstrap_provider_archive_record_matches(record, BootstrapSourceBundleMode::LegacySeed))
        .ok_or_else(|| RunError::Internal("missing bootstrap provider archive source state".to_string()))?;
    if !source_record_is_pinned(record, record, &pinned_sources) {
        return Err(RunError::Internal("unpinned bootstrap provider archive source state".to_string()));
    }
    if record.files.is_empty() {
        return Err(RunError::Internal(
            "bootstrap provider archive source state has no materialized payload".to_string(),
        ));
    }
    assert!(source_record_is_pinned(record, record, &pinned_sources));
    assert!(!record.files.is_empty());
    let source_state_blake3 = digest_offline_preflight_state(None, std::slice::from_ref(record))?;
    let scratch_dir = tempfile::Builder::new()
        .prefix("mantle-bootstrap-source-fetch-")
        .tempdir()
        .map_err(|err| RunError::Internal(format!("creating bootstrap source fetch scratch dir: {err}")))?;
    let payload_path = scratch_dir.path().join("payload");
    materialize_source_record_payload(record, &payload_path)?;
    let preflight_receipt = SourceOfflinePreflightReport {
        format: SOURCE_OFFLINE_PREFLIGHT_FORMAT,
        manifest_blake3: None,
        source_state_blake3: source_state_blake3.clone(),
        ready_class: SourceReadiness::Ready,
        record_count: 1,
        missing_records: Vec::new(),
        stale_records: Vec::new(),
        unsupported_records: Vec::new(),
        untrusted_records: Vec::new(),
        network_required_records: Vec::new(),
        unpinned_records: Vec::new(),
        next_actions: Vec::new(),
        records: vec![summary_for_record(record)?],
        non_claim: SOURCE_BUNDLE_NON_CLAIM,
    };
    Ok(SourceFetchOverridePlan {
        report: preflight_receipt,
        overrides: vec![crunch_build::FetchSourceOverride {
            url: provider_raw_url.to_string(),
            kind: crunch_build::FetchSourceOverrideKind::Tarball,
            rev: None,
            payload_path,
            source_state_blake3,
        }],
        _scratch_dirs: vec![scratch_dir],
    })
}

#[cfg(test)]
pub fn source_fetch_override_plan_for_derivations(
    roots: &[(String, crunch_glue::CrunchDerivation)],
    state_dir: &Path,
    store_prefix: &str,
) -> Result<SourceFetchOverridePlan, RunError> {
    let records = collect_build_source_records(roots, store_prefix)?;
    source_fetch_override_plan_for_records(records, state_dir, store_prefix)
}

fn source_fetch_override_plan_for_records(
    records: Vec<SourceRecord>,
    state_dir: &Path,
    store_prefix: &str,
) -> Result<SourceFetchOverridePlan, RunError> {
    assert!(records.len() <= MAX_SOURCE_RECORDS);
    assert!(store_prefix.starts_with('/'));
    if records.is_empty() {
        return Ok(SourceFetchOverridePlan {
            report: empty_offline_preflight_report()?,
            overrides: Vec::new(),
            _scratch_dirs: Vec::new(),
        });
    }
    let manifest = assemble_source_bundle(records, store_prefix)?;
    let available_sources = read_imported_source_records(state_dir)?;
    let pinned_sources = read_pinned_source_records(state_dir)?;
    let preflight_receipt = classify_offline_preflight(&manifest, &available_sources, &pinned_sources)?;
    if !source_offline_preflight_is_ready(&preflight_receipt) {
        return Err(RunError::Internal(format!(
            "offline source preflight is not ready: {:?}",
            preflight_receipt.ready_class
        )));
    }
    let (overrides, scratch_dirs) = source_fetch_overrides_for_manifest(
        &manifest,
        &available_sources,
        &pinned_sources,
        &preflight_receipt.source_state_blake3,
    )?;
    Ok(SourceFetchOverridePlan {
        report: preflight_receipt,
        overrides,
        _scratch_dirs: scratch_dirs,
    })
}

fn canonicalize_source_spec(spec: &SourceSpec, store_prefix: &str) -> Result<SourceRecord, RunError> {
    source_record_from_path(SourceRecordPathRequest {
        kind: spec.kind.clone(),
        identity: spec.identity.clone(),
        path: &spec.path,
        store_prefix,
        metadata: BTreeMap::new(),
        adapter: spec.adapter.clone(),
        is_skipping_git_dir: false,
    })
}

fn source_record_from_path(request: SourceRecordPathRequest<'_>) -> Result<SourceRecord, RunError> {
    validate_identity(&request.identity)?;
    validate_adapter_metadata(request.adapter.as_ref())?;
    let (files, payload_bytes) = canonicalize_payload_entries(request.path, request.is_skipping_git_dir)?;
    let content_blake3 = digest_source_record_content(&request.kind, &request.metadata, &files)?;
    Ok(SourceRecord {
        store_prefix: record_store_prefix(&request.kind, request.store_prefix),
        kind: request.kind,
        identity: request.identity,
        adapter: request.adapter,
        metadata: request.metadata,
        payload_bytes,
        content_blake3,
        files,
    })
}

fn validate_bootstrap_profile_input(
    input: &BootstrapSourceBundleProfileInput,
    store_prefix: &str,
) -> Result<(), RunError> {
    if !store_prefix.starts_with('/') {
        return Err(RunError::Internal(format!("store prefix must be absolute: {store_prefix}")));
    }
    if input.mode.requires_bootstrap_sources() && input.bootstrap_sources.is_empty() {
        return Err(RunError::Internal("bootstrap profile requires at least one bootstrap source archive".to_string()));
    }
    if input.mode.requires_full_self_build_inputs() {
        if input.mantle_source.is_none() {
            return Err(RunError::Internal("self-build bootstrap profile requires --mantle-source".to_string()));
        }
        if input.proof_inputs.is_empty() {
            return Err(RunError::Internal(
                "self-build bootstrap profile requires at least one --proof-input".to_string(),
            ));
        }
    }
    if input.mode.requires_vendor_inputs() && input.vendor_deps.is_none() {
        return Err(RunError::Internal(format!("{} bootstrap profile requires --vendor-deps", input.mode.as_str())));
    }
    if input.bootstrap_sources.len() > MAX_SOURCE_RECORDS {
        return Err(RunError::Internal(format!("bootstrap profile source count exceeds {MAX_SOURCE_RECORDS}")));
    }
    if input.proof_inputs.len() > MAX_SOURCE_RECORDS || input.supplemental_records.len() > MAX_SOURCE_RECORDS {
        return Err(RunError::Internal(format!("bootstrap profile source count exceeds {MAX_SOURCE_RECORDS}")));
    }
    validate_supplemental_profile_records(input)?;
    assert!(store_prefix.starts_with('/'));
    assert!(!input.mode.requires_bootstrap_sources() || !input.bootstrap_sources.is_empty());
    assert!(!input.mode.requires_vendor_inputs() || input.vendor_deps.is_some());
    assert!(!input.mode.requires_supplemental_fetch_closure() || !input.supplemental_records.is_empty());
    Ok(())
}

fn validate_supplemental_profile_records(input: &BootstrapSourceBundleProfileInput) -> Result<(), RunError> {
    if input.mode.requires_supplemental_fetch_closure() && input.supplemental_records.is_empty() {
        return Err(RunError::Internal(
            "fresh-clone-fixed-point profile requires materialized --include-bundle fetch records".to_string(),
        ));
    }
    if !input.mode.requires_supplemental_fetch_closure() && !input.supplemental_records.is_empty() {
        return Err(RunError::Internal(format!(
            "{} profile does not accept supplemental source bundles",
            input.mode.as_str()
        )));
    }
    for record in &input.supplemental_records {
        if !source_record_is_fetcher_input(record) || record.files.is_empty() {
            return Err(RunError::Internal(format!(
                "supplemental full-proof record {} must be a materialized fixed fetcher input",
                record.identity
            )));
        }
        validate_source_record(record)?;
    }
    Ok(())
}

fn bootstrap_profile_record(request: BootstrapProfileRecordRequest<'_>) -> Result<SourceRecord, RunError> {
    let mut metadata = BTreeMap::new();
    metadata.insert(RECORD_METADATA_PROFILE_MODE_KEY.to_string(), request.mode.as_str().to_string());
    metadata.insert(RECORD_METADATA_PROFILE_CLASS_KEY.to_string(), request.class.to_string());
    if let Some(provider_metadata) = request.provider_metadata {
        metadata.insert(RECORD_METADATA_PROVIDER_KIND_KEY.to_string(), provider_metadata.provider_kind.clone());
        metadata.insert(RECORD_METADATA_PROVIDER_SCHEMA_KEY.to_string(), provider_metadata.schema_version.clone());
    }
    let is_skipping_git_dir = request.class == BOOTSTRAP_PROFILE_CLASS_MANTLE_SOURCE;
    source_record_from_path(SourceRecordPathRequest {
        kind: request.kind,
        identity: request.identity,
        path: request.path,
        store_prefix: request.store_prefix,
        metadata,
        adapter: None,
        is_skipping_git_dir,
    })
}

fn bootstrap_indexed_identity(class: &str, index: usize) -> Result<String, RunError> {
    let display_index = index
        .checked_add(1)
        .ok_or_else(|| RunError::Internal("bootstrap profile index overflow".to_string()))?;
    let identity = format!("{class}-{display_index:0width$}", width = BOOTSTRAP_PROFILE_INDEX_WIDTH);
    validate_identity(&identity)?;
    Ok(identity)
}

fn read_bootstrap_provider_profile_metadata(path: &Path) -> Result<BootstrapProviderProfileMetadata, RunError> {
    let bytes = fs::read(path)
        .map_err(|err| RunError::Internal(format!("reading bootstrap provider manifest {}: {err}", path.display())))?;
    let value = serde_json::from_slice::<serde_json::Value>(&bytes)
        .map_err(|err| RunError::Internal(format!("parsing bootstrap provider manifest {}: {err}", path.display())))?;
    let provider_kind = provider_kind_from_json(&value).ok_or_else(|| {
        RunError::Internal("bootstrap provider manifest missing provider_kind or provider.id".to_string())
    })?;
    let schema_version = provider_schema_from_json(&value);
    validate_provider_manifest_has_boundary_metadata(&value)?;
    Ok(BootstrapProviderProfileMetadata {
        provider_kind,
        schema_version,
    })
}

fn provider_kind_from_json(value: &serde_json::Value) -> Option<String> {
    value
        .get("provider_kind")
        .and_then(serde_json::Value::as_str)
        .or_else(|| value.get("provider_id").and_then(serde_json::Value::as_str))
        .or_else(|| value.get("provider").and_then(|provider| provider.get("id")).and_then(serde_json::Value::as_str))
        .map(str::to_string)
}

fn provider_schema_from_json(value: &serde_json::Value) -> String {
    value
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
        .map(|version| version.to_string())
        .unwrap_or_else(|| "1".to_string())
}

fn validate_provider_manifest_has_boundary_metadata(value: &serde_json::Value) -> Result<(), RunError> {
    if value.get("reduction").is_none() {
        return Err(RunError::Internal("bootstrap provider manifest missing reduced-provider provenance".to_string()));
    }
    if value.get("normalized_seed_contract").is_none() && value.get("target").is_none() {
        return Err(RunError::Internal(
            "bootstrap provider manifest missing normalized seed contract facts".to_string(),
        ));
    }
    Ok(())
}

fn validate_bootstrap_provider_kind(mode: BootstrapSourceBundleMode, provider_kind: &str) -> Result<(), RunError> {
    let expected = mode.expected_provider_kind();
    if provider_kind == expected {
        return Ok(());
    }
    Err(RunError::Internal(format!(
        "bootstrap provider kind mismatch for {}: expected {expected}, got {provider_kind}",
        mode.as_str()
    )))
}

fn canonicalize_payload_entries(
    path: &Path,
    is_skipping_git_dir: bool,
) -> Result<(Vec<SourceFileEntry>, u64), RunError> {
    canonicalize_payload_entries_with_policy(path, is_skipping_git_dir, false)
}

fn canonicalize_fetch_payload_entries(
    path: &Path,
    is_skipping_git_dir: bool,
) -> Result<(Vec<SourceFileEntry>, u64), RunError> {
    canonicalize_payload_entries_with_policy(path, is_skipping_git_dir, true)
}

fn canonicalize_payload_entries_with_policy(
    path: &Path,
    is_skipping_git_dir: bool,
    allow_large_file_chunks: bool,
) -> Result<(Vec<SourceFileEntry>, u64), RunError> {
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
    assert!(root.is_absolute());
    assert!(relative_root.is_absolute());
    let mut collection = SourceEntryCollection {
        files: Vec::new(),
        payload_bytes: 0,
        visited_nodes_len: 0,
    };
    collect_source_entries(&relative_root, &root, is_skipping_git_dir, allow_large_file_chunks, &mut collection)?;
    collection
        .files
        .sort_by(|left, right| left.path.cmp(&right.path).then(left.chunk_index.cmp(&right.chunk_index)));
    assert!(collection.files.len() <= MAX_SOURCE_FILES_PER_RECORD);
    assert!(collection.payload_bytes <= MAX_SOURCE_TOTAL_BYTES);
    Ok((collection.files, collection.payload_bytes))
}

fn collect_source_entries(
    root: &Path,
    current: &Path,
    is_skipping_git_dir: bool,
    allow_large_file_chunks: bool,
    collection: &mut SourceEntryCollection,
) -> Result<(), RunError> {
    assert!(current.starts_with(root));
    assert!(collection.files.len() <= MAX_SOURCE_FILES_PER_RECORD);
    let mut pending_paths = Vec::with_capacity(1);
    pending_paths.push(current.to_path_buf());
    for _ in 0..MAX_SOURCE_FILES_PER_RECORD {
        let Some(next_path) = pending_paths.pop() else {
            assert!(pending_paths.is_empty());
            assert!(collection.visited_nodes_len <= MAX_SOURCE_FILES_PER_RECORD);
            return Ok(());
        };
        collection.visited_nodes_len = collection
            .visited_nodes_len
            .checked_add(1)
            .ok_or_else(|| RunError::Internal("source entry walk count overflow".to_string()))?;
        let metadata = fs::symlink_metadata(&next_path)
            .map_err(|err| RunError::Internal(format!("reading source metadata {}: {err}", next_path.display())))?;
        if metadata.is_dir() {
            let children = read_sorted_source_children(&next_path, is_skipping_git_dir)?;
            reserve_pending_source_paths(&mut pending_paths, children.len())?;
            pending_paths.extend(children.into_iter().rev());
            continue;
        }
        let files = source_file_entries(root, &next_path, &metadata, allow_large_file_chunks)?;
        for file in files {
            append_source_file_entry(collection, file)?;
        }
    }
    Err(RunError::Internal(format!("source entry walk exceeds {MAX_SOURCE_FILES_PER_RECORD} nodes")))
}

fn read_sorted_source_children(current: &Path, is_skipping_git_dir: bool) -> Result<Vec<PathBuf>, RunError> {
    assert!(current.is_absolute());
    let entries = fs::read_dir(current)
        .map_err(|err| RunError::Internal(format!("reading source dir {}: {err}", current.display())))?;
    let mut children = Vec::new();
    for entry in entries {
        let entry = entry
            .map_err(|err| RunError::Internal(format!("reading source dir entry {}: {err}", current.display())))?;
        if is_skipping_git_dir && entry.file_name().to_str() == Some(DOT_GIT_DIR_NAME) {
            continue;
        }
        if children.len() >= MAX_SOURCE_FILES_PER_RECORD {
            return Err(RunError::Internal(format!("source entry count exceeds {MAX_SOURCE_FILES_PER_RECORD}")));
        }
        children
            .try_reserve(1)
            .map_err(|err| RunError::Internal(format!("reserving source directory entries: {err}")))?;
        children.push(entry.path());
    }
    children.sort();
    assert!(children.len() <= MAX_SOURCE_FILES_PER_RECORD);
    Ok(children)
}

fn reserve_pending_source_paths(pending_paths: &mut Vec<PathBuf>, additional_len: usize) -> Result<(), RunError> {
    let pending_len = pending_paths
        .len()
        .checked_add(additional_len)
        .ok_or_else(|| RunError::Internal("pending source path count overflow".to_string()))?;
    if pending_len > MAX_SOURCE_FILES_PER_RECORD {
        return Err(RunError::Internal(format!("pending source path count exceeds {MAX_SOURCE_FILES_PER_RECORD}")));
    }
    pending_paths
        .try_reserve(additional_len)
        .map_err(|err| RunError::Internal(format!("reserving pending source paths: {err}")))?;
    Ok(())
}

fn source_file_entries(
    root: &Path,
    current: &Path,
    metadata: &fs::Metadata,
    allow_large_file_chunks: bool,
) -> Result<Vec<SourceFileEntry>, RunError> {
    let relative = safe_relative_path(root, current)?;
    if metadata.file_type().is_symlink() {
        return Ok(vec![symlink_source_file_entry(current, relative)?]);
    }
    if !metadata.is_file() {
        return Err(RunError::Internal(format!("unsupported source file kind at {}", current.display())));
    }
    regular_source_file_entries(current, relative, metadata, allow_large_file_chunks)
}

fn symlink_source_file_entry(current: &Path, relative: String) -> Result<SourceFileEntry, RunError> {
    let target = fs::read_link(current)
        .map_err(|err| RunError::Internal(format!("reading source symlink {}: {err}", current.display())))?;
    let target_text = safe_symlink_target(&target)?;
    let digest = blake3::hash(format!("symlink\0{relative}\0{target_text}").as_bytes());
    Ok(SourceFileEntry {
        path: relative,
        file_type: SourceFileType::Symlink,
        executable: false,
        size: SYMLINK_PAYLOAD_BYTES,
        content_hex: None,
        symlink_target: Some(target_text),
        chunk_index: None,
        chunk_count: None,
        blake3: digest.to_hex().to_string(),
    })
}

fn regular_source_file_entries(
    current: &Path,
    relative: String,
    metadata: &fs::Metadata,
    allow_large_file_chunks: bool,
) -> Result<Vec<SourceFileEntry>, RunError> {
    assert!(metadata.is_file());
    assert!(!relative.is_empty());
    let size_bytes = metadata.len();
    if size_bytes <= MAX_SOURCE_FILE_BYTES {
        let content = fs::read(current)
            .map_err(|err| RunError::Internal(format!("reading source file {}: {err}", current.display())))?;
        return Ok(vec![regular_source_file_entry_from_content(
            relative,
            is_executable(metadata),
            content,
            None,
        )]);
    }
    if !allow_large_file_chunks {
        return Err(RunError::Internal(format!(
            "source file {} is {size_bytes} bytes, limit {MAX_SOURCE_FILE_BYTES}",
            current.display()
        )));
    }
    chunked_source_file_entries(current, relative, metadata, size_bytes)
}

fn regular_source_file_entry_from_content(
    path: String,
    executable: bool,
    content: Vec<u8>,
    chunk: Option<(u32, u32)>,
) -> SourceFileEntry {
    let size = u64::try_from(content.len()).expect("source chunk length fits in u64");
    let digest = blake3::hash(&content).to_hex().to_string();
    let (chunk_index, chunk_count) = chunk.map_or((None, None), |(index, count)| (Some(index), Some(count)));
    SourceFileEntry {
        path,
        file_type: SourceFileType::Regular,
        executable,
        size,
        content_hex: Some(HEXLOWER.encode(&content)),
        symlink_target: None,
        chunk_index,
        chunk_count,
        blake3: digest,
    }
}

fn source_file_chunk_sizes(size_bytes: u64, chunk_size_bytes_max: u64) -> Result<Vec<u64>, RunError> {
    if size_bytes == 0 || chunk_size_bytes_max == 0 {
        return Err(RunError::Internal("source file chunk sizes require positive inputs".to_string()));
    }
    let chunk_count = size_bytes
        .checked_add(chunk_size_bytes_max - 1)
        .ok_or_else(|| RunError::Internal("source file chunk count overflow".to_string()))?
        / chunk_size_bytes_max;
    let chunk_count_usize = usize::try_from(chunk_count)
        .map_err(|_| RunError::Internal("source file chunk count does not fit in usize".to_string()))?;
    if chunk_count_usize > MAX_SOURCE_FILES_PER_RECORD {
        return Err(RunError::Internal(format!("source file chunk count exceeds {MAX_SOURCE_FILES_PER_RECORD}")));
    }
    let mut sizes = Vec::with_capacity(chunk_count_usize);
    let mut remaining_bytes = size_bytes;
    for _ in 0..chunk_count_usize {
        let chunk_size = remaining_bytes.min(chunk_size_bytes_max);
        sizes.push(chunk_size);
        remaining_bytes = remaining_bytes.saturating_sub(chunk_size);
    }
    assert_eq!(remaining_bytes, 0);
    Ok(sizes)
}

fn chunked_source_file_entries(
    current: &Path,
    relative: String,
    metadata: &fs::Metadata,
    size_bytes: u64,
) -> Result<Vec<SourceFileEntry>, RunError> {
    let chunk_sizes = source_file_chunk_sizes(size_bytes, MAX_SOURCE_FILE_BYTES)?;
    let chunk_count = u32::try_from(chunk_sizes.len())
        .map_err(|_| RunError::Internal(format!("source file chunk count overflow for {}", current.display())))?;
    if chunk_count < MIN_SOURCE_FILE_CHUNK_COUNT {
        return Err(RunError::Internal(format!("source file chunk count is invalid for {}", current.display())));
    }
    let mut input = fs::File::open(current)
        .map_err(|err| RunError::Internal(format!("opening source file {}: {err}", current.display())))?;
    let mut entries = Vec::with_capacity(chunk_sizes.len());
    for (chunk_index, chunk_size) in chunk_sizes.into_iter().enumerate() {
        let chunk_index = u32::try_from(chunk_index)
            .map_err(|_| RunError::Internal(format!("source file chunk index overflow for {}", current.display())))?;
        let mut content = vec![0; usize::try_from(chunk_size).expect("bounded source chunk size fits in usize")];
        input
            .read_exact(&mut content)
            .map_err(|err| RunError::Internal(format!("reading source file chunk {}: {err}", current.display())))?;
        entries.push(regular_source_file_entry_from_content(
            relative.clone(),
            is_executable(metadata),
            content,
            Some((chunk_index, chunk_count)),
        ));
    }
    let mut trailing = [0u8; 1];
    if input
        .read(&mut trailing)
        .map_err(|err| RunError::Internal(format!("checking source file end {}: {err}", current.display())))?
        != 0
    {
        return Err(RunError::Internal(format!("source file {} grew during chunking", current.display())));
    }
    Ok(entries)
}

fn append_source_file_entry(collection: &mut SourceEntryCollection, file: SourceFileEntry) -> Result<(), RunError> {
    if collection.files.len() >= MAX_SOURCE_FILES_PER_RECORD {
        return Err(RunError::Internal(format!("source file count exceeds {MAX_SOURCE_FILES_PER_RECORD}")));
    }
    collection.payload_bytes = collection
        .payload_bytes
        .checked_add(file.size)
        .ok_or_else(|| RunError::Internal("source payload byte count overflow".to_string()))?;
    if collection.payload_bytes > MAX_SOURCE_TOTAL_BYTES {
        return Err(RunError::Internal(format!("source payload bytes exceed {MAX_SOURCE_TOTAL_BYTES}")));
    }
    collection
        .files
        .try_reserve(1)
        .map_err(|err| RunError::Internal(format!("reserving source file entry: {err}")))?;
    collection.files.push(file);
    assert!(collection.files.len() <= MAX_SOURCE_FILES_PER_RECORD);
    assert!(collection.payload_bytes <= MAX_SOURCE_TOTAL_BYTES);
    Ok(())
}

fn safe_relative_path(root: &Path, current: &Path) -> Result<String, RunError> {
    let relative = current
        .strip_prefix(root)
        .map_err(|err| RunError::Internal(format!("source path escaped root: {err}")))?;
    let text = relative
        .to_str()
        .ok_or_else(|| RunError::Internal(format!("source path is not UTF-8: {}", current.display())))?;
    validate_source_relative_path_text(text)?;
    Ok(text.to_string())
}

fn validate_source_relative_path_text(text: &str) -> Result<(), RunError> {
    if text.is_empty() {
        return Err(RunError::Internal(format!("unsafe source relative path '{text}'")));
    }
    if text.starts_with('/') {
        return Err(RunError::Internal(format!("unsafe source relative path '{text}'")));
    }
    if text.contains('\\') {
        return Err(RunError::Internal(format!("unsafe source relative path '{text}'")));
    }
    for component in text.split('/') {
        if component.is_empty() || matches!(component, "." | "..") {
            return Err(RunError::Internal(format!("unsafe source relative path '{text}'")));
        }
    }
    assert!(!text.is_empty());
    assert!(!text.starts_with('/'));
    Ok(())
}

fn safe_symlink_target(target: &Path) -> Result<String, RunError> {
    let target_text = target
        .to_str()
        .ok_or_else(|| RunError::Internal(format!("symlink target is not UTF-8: {}", target.display())))?;
    validate_symlink_target_text(target_text)?;
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
    if adapter.adapter.is_empty() {
        return Err(RunError::Internal("adapter metadata missing lock/offline/generated-source identity".to_string()));
    }
    if adapter.lock_identity.is_empty() {
        return Err(RunError::Internal("adapter metadata missing lock/offline/generated-source identity".to_string()));
    }
    if adapter.offline_control.is_empty() {
        return Err(RunError::Internal("adapter metadata missing lock/offline/generated-source identity".to_string()));
    }
    if adapter.generated_source_boundary.is_empty() {
        return Err(RunError::Internal("adapter metadata missing lock/offline/generated-source identity".to_string()));
    }
    assert!(!adapter.adapter.is_empty());
    assert!(!adapter.lock_identity.is_empty());
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
    assert!(metadata.len() <= MAX_SOURCE_RECORD_METADATA_BYTES);
    assert!(files.len() <= MAX_SOURCE_FILES_PER_RECORD);
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
    if manifest.non_claim != SOURCE_BUNDLE_NON_CLAIM {
        return Err(RunError::Internal("source bundle non-claim boundary mismatch".to_string()));
    }
    assert_eq!(manifest.format, SOURCE_BUNDLE_FORMAT);
    assert_eq!(manifest.version, SOURCE_BUNDLE_VERSION);
    validate_manifest_roots(manifest)?;
    validate_manifest_record_order(&manifest.records)?;
    validate_manifest_store_prefix(manifest)?;
    reject_duplicate_records(&manifest.records)?;
    let expected = digest_manifest_without_digest(manifest)?;
    if expected != manifest.manifest_blake3 {
        return Err(RunError::Internal("source bundle manifest digest mismatch".to_string()));
    }
    Ok(())
}

fn normalize_source_records(mut records: Vec<SourceRecord>) -> Result<Vec<SourceRecord>, RunError> {
    records.sort_by_key(record_sort_key);
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

fn validate_manifest_roots(manifest: &SourceBundleManifest) -> Result<(), RunError> {
    let expected_roots = manifest.records.iter().map(|record| record.identity.clone()).collect::<Vec<_>>();
    if manifest.roots != expected_roots {
        return Err(RunError::Internal("source bundle roots do not match canonical record order".to_string()));
    }
    Ok(())
}

fn validate_manifest_record_order(records: &[SourceRecord]) -> Result<(), RunError> {
    let mut previous_key: Option<String> = None;
    for record in records {
        let current_key = record_sort_key(record);
        if let Some(previous_key) = previous_key.as_ref()
            && previous_key > &current_key
        {
            return Err(RunError::Internal("source bundle records are not in canonical order".to_string()));
        }
        previous_key = Some(current_key);
    }
    Ok(())
}

fn validate_manifest_store_prefix(manifest: &SourceBundleManifest) -> Result<(), RunError> {
    if !manifest.store_prefix.starts_with('/') {
        return Err(RunError::Internal(format!("store prefix must be absolute: {}", manifest.store_prefix)));
    }
    for record in &manifest.records {
        let expected = record_store_prefix(&record.kind, &manifest.store_prefix);
        if record.store_prefix != expected {
            return Err(RunError::Internal(format!("source record {} store prefix mismatch", record.identity)));
        }
    }
    Ok(())
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
    validate_source_record_payload_encoding(record)?;
    let rendered_metadata = serde_json::to_vec(&record.metadata)
        .map_err(|err| RunError::Internal(format!("serializing source metadata: {err}")))?;
    if rendered_metadata.len() > MAX_SOURCE_RECORD_METADATA_BYTES {
        return Err(RunError::Internal(format!(
            "source record metadata exceeds {MAX_SOURCE_RECORD_METADATA_BYTES} bytes"
        )));
    }
    validate_source_record_files(record)?;
    let expected_digest = digest_source_record_content(&record.kind, &record.metadata, &record.files)?;
    if expected_digest != record.content_blake3 {
        return Err(RunError::Internal(format!("source record {} content digest mismatch", record.identity)));
    }
    Ok(())
}

fn validate_source_record_payload_encoding(record: &SourceRecord) -> Result<(), RunError> {
    let Some(encoding) = record.metadata.get(RECORD_METADATA_PAYLOAD_ENCODING_KEY) else {
        return Ok(());
    };
    if encoding != TARBALL_ARCHIVE_PAYLOAD_ENCODING {
        return Err(RunError::Internal(format!("source record {} has unsupported payload encoding", record.identity)));
    }
    let is_tarball_fetch = matches!(record.kind, SourceRecordKind::FixedUrl | SourceRecordKind::BootstrapArchive)
        && record.metadata.get(FETCH_ENV_UNPACK_KEY).map(String::as_str) == Some("1")
        && record.metadata.contains_key(RECORD_METADATA_URL_KEY);
    if !is_tarball_fetch {
        return Err(RunError::Internal(format!(
            "source record {} uses tarball archive encoding outside a tarball fetch",
            record.identity
        )));
    }
    Ok(())
}

fn source_record_uses_tarball_archive_payload(record: &SourceRecord) -> bool {
    record.metadata.get(RECORD_METADATA_PAYLOAD_ENCODING_KEY).map(String::as_str)
        == Some(TARBALL_ARCHIVE_PAYLOAD_ENCODING)
}

fn validate_source_record_files(record: &SourceRecord) -> Result<(), RunError> {
    let mut total_bytes = 0u64;
    let mut case_folded_paths = BTreeMap::<String, String>::new();
    let mut previous_key: Option<(&str, Option<u32>)> = None;
    let requires_case_sensitive_paths = matches!(record.kind, SourceRecordKind::BootstrapArchive);
    for file in &record.files {
        validate_source_file_entry(file)?;
        let current_key = (file.path.as_str(), file.chunk_index);
        if previous_key.is_some_and(|previous| previous > current_key) {
            return Err(RunError::Internal(format!(
                "source record {} files are not in canonical order",
                record.identity
            )));
        }
        previous_key = Some(current_key);
        let case_key = file.path.to_lowercase();
        if let Some(first_path) = case_folded_paths.get(&case_key) {
            let is_exact_path = first_path == &file.path;
            if !is_exact_path && !requires_case_sensitive_paths {
                return Err(RunError::Internal(format!("source record {} has a path case collision", record.identity)));
            }
        } else {
            case_folded_paths.insert(case_key, file.path.clone());
        }
        total_bytes = total_bytes.checked_add(file.size).ok_or_else(|| {
            RunError::Internal(format!("source record {} payload byte count overflow", record.identity))
        })?;
    }
    validate_source_file_chunk_sequences(record)?;
    validate_no_symlink_descendants(record)?;
    if total_bytes != record.payload_bytes {
        return Err(RunError::Internal(format!("source record {} payload byte count mismatch", record.identity)));
    }
    Ok(())
}

fn validate_source_file_chunk_sequences(record: &SourceRecord) -> Result<(), RunError> {
    let mut group_start = 0usize;
    while group_start < record.files.len() {
        let path = record.files[group_start].path.as_str();
        let mut group_end = group_start + 1;
        while group_end < record.files.len() && record.files[group_end].path == path {
            group_end += 1;
        }
        validate_source_file_chunk_group(record, &record.files[group_start..group_end])?;
        group_start = group_end;
    }
    Ok(())
}

fn validate_source_file_chunk_group(record: &SourceRecord, files: &[SourceFileEntry]) -> Result<(), RunError> {
    assert!(!files.is_empty());
    let path = files[0].path.as_str();
    if files.len() == 1 && files[0].chunk_index.is_none() && files[0].chunk_count.is_none() {
        return Ok(());
    }
    let chunk_count = u32::try_from(files.len()).map_err(|_| {
        RunError::Internal(format!("source record {} chunk count overflow for {path}", record.identity))
    })?;
    if chunk_count < MIN_SOURCE_FILE_CHUNK_COUNT {
        return Err(RunError::Internal(format!(
            "source record {} has invalid chunk count for {path}",
            record.identity
        )));
    }
    let executable = files[0].executable;
    let mut logical_size_bytes = 0u64;
    for (expected_index, file) in files.iter().enumerate() {
        let expected_index = u32::try_from(expected_index).map_err(|_| {
            RunError::Internal(format!("source record {} chunk index overflow for {path}", record.identity))
        })?;
        let is_final_chunk = expected_index.checked_add(1) == Some(chunk_count);
        let invalid_chunk_size = file.size == 0 || (!is_final_chunk && file.size != MAX_SOURCE_FILE_BYTES);
        if file.file_type != SourceFileType::Regular
            || file.chunk_index != Some(expected_index)
            || file.chunk_count != Some(chunk_count)
            || file.executable != executable
            || invalid_chunk_size
        {
            return Err(RunError::Internal(format!(
                "source record {} has a non-canonical chunk sequence for {path}",
                record.identity
            )));
        }
        logical_size_bytes = logical_size_bytes.checked_add(file.size).ok_or_else(|| {
            RunError::Internal(format!("source record {} chunk size overflow for {path}", record.identity))
        })?;
    }
    if logical_size_bytes <= MAX_SOURCE_FILE_BYTES {
        return Err(RunError::Internal(format!("source record {} has unnecessary chunks for {path}", record.identity)));
    }
    Ok(())
}

fn validate_no_symlink_descendants(record: &SourceRecord) -> Result<(), RunError> {
    assert!(record.files.len() <= MAX_SOURCE_FILES_PER_RECORD);
    assert!(!record.identity.is_empty());
    let symlink_paths = record
        .files
        .iter()
        .filter(|file| matches!(file.file_type, SourceFileType::Symlink))
        .map(|file| file.path.as_str())
        .collect::<BTreeSet<_>>();
    for file in &record.files {
        let mut ancestor = file.path.as_str();
        while let Some((parent, _name)) = ancestor.rsplit_once('/') {
            if symlink_paths.contains(parent) {
                return Err(RunError::Internal(format!(
                    "source record {} has file {} below symlink {}",
                    record.identity, file.path, parent
                )));
            }
            ancestor = parent;
        }
    }
    Ok(())
}

fn validate_source_file_entry(file: &SourceFileEntry) -> Result<(), RunError> {
    validate_source_entry_path(&file.path)?;
    validate_blake3_hex(Blake3HexValidation {
        value: &file.blake3,
        label: "source file digest",
    })?;
    match file.file_type {
        SourceFileType::Regular => validate_regular_file_entry(file),
        SourceFileType::Symlink => validate_symlink_file_entry(file),
    }
}

fn validate_source_entry_path(path: &str) -> Result<(), RunError> {
    validate_source_relative_path_text(path)
}

fn validate_regular_file_entry(file: &SourceFileEntry) -> Result<(), RunError> {
    if file.symlink_target.is_some() {
        return Err(RunError::Internal(format!("regular source file {} carries a symlink target", file.path)));
    }
    if file.size > MAX_SOURCE_FILE_BYTES {
        return Err(RunError::Internal(format!(
            "regular source file entry {} exceeds {MAX_SOURCE_FILE_BYTES} bytes",
            file.path
        )));
    }
    if file.chunk_index.is_some() != file.chunk_count.is_some() {
        return Err(RunError::Internal(format!("regular source file {} has incomplete chunk metadata", file.path)));
    }
    let content = decode_regular_file_content(file)?;
    let content_len = u64::try_from(content.len())
        .map_err(|_| RunError::Internal(format!("regular source file {} length does not fit in u64", file.path)))?;
    if content_len != file.size {
        return Err(RunError::Internal(format!("regular source file {} size mismatch", file.path)));
    }
    let expected = blake3::hash(&content).to_hex().to_string();
    if expected != file.blake3 {
        return Err(RunError::Internal(format!("regular source file {} digest mismatch", file.path)));
    }
    Ok(())
}

fn decode_regular_file_content(file: &SourceFileEntry) -> Result<Vec<u8>, RunError> {
    let content_hex = file
        .content_hex
        .as_ref()
        .ok_or_else(|| RunError::Internal(format!("regular source file {} is missing payload bytes", file.path)))?;
    HEXLOWER.decode(content_hex.as_bytes()).map_err(|err| {
        RunError::Internal(format!("regular source file {} has invalid lowercase hex payload: {err}", file.path))
    })
}

fn validate_symlink_file_entry(file: &SourceFileEntry) -> Result<(), RunError> {
    if file.chunk_index.is_some() || file.chunk_count.is_some() {
        return Err(RunError::Internal(format!("symlink source file {} carries chunk metadata", file.path)));
    }
    if file.content_hex.is_some() {
        return Err(RunError::Internal(format!("symlink source file {} carries payload bytes", file.path)));
    }
    if file.executable {
        return Err(RunError::Internal(format!("symlink source file {} cannot be executable", file.path)));
    }
    if file.size != SYMLINK_PAYLOAD_BYTES {
        return Err(RunError::Internal(format!("symlink source file {} size mismatch", file.path)));
    }
    assert!(file.content_hex.is_none());
    assert!(!file.executable);
    let target = file
        .symlink_target
        .as_ref()
        .ok_or_else(|| RunError::Internal(format!("symlink source file {} is missing target", file.path)))?;
    validate_symlink_target_text(target)?;
    let expected = blake3::hash(format!("symlink\0{}\0{target}", file.path).as_bytes()).to_hex().to_string();
    if expected != file.blake3 {
        return Err(RunError::Internal(format!("symlink source file {} digest mismatch", file.path)));
    }
    Ok(())
}

fn validate_symlink_target_text(target: &str) -> Result<(), RunError> {
    if target.starts_with('/') || target.contains("..") || target.is_empty() {
        return Err(RunError::Internal(format!("unsafe source symlink target '{target}'")));
    }
    Ok(())
}

fn validate_blake3_hex(request: Blake3HexValidation<'_>) -> Result<(), RunError> {
    if request.value.len() != BLAKE3_HEX_BYTES {
        return Err(RunError::Internal(format!("{} must be {BLAKE3_HEX_BYTES} lowercase hex bytes", request.label)));
    }
    HEXLOWER
        .decode(request.value.as_bytes())
        .map_err(|err| RunError::Internal(format!("{} must be lowercase hex: {err}", request.label)))?;
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

fn classify_source_state(
    missing: &[String],
    stale: &[String],
    unsupported: &[String],
    untrusted: &[String],
) -> SourceReadiness {
    if !unsupported.is_empty() {
        return SourceReadiness::Unsupported;
    }
    if !stale.is_empty() {
        return SourceReadiness::Stale;
    }
    if !untrusted.is_empty() {
        return SourceReadiness::Untrusted;
    }
    if !missing.is_empty() {
        return SourceReadiness::Missing;
    }
    SourceReadiness::Ready
}

fn classify_offline_preflight(
    manifest: &SourceBundleManifest,
    available_sources: &[SourceRecord],
    pinned_sources: &[SourceRecord],
) -> Result<SourceOfflinePreflightReport, RunError> {
    validate_manifest(manifest)?;
    assert!(manifest.records.len() <= MAX_SOURCE_RECORDS);
    assert_eq!(manifest.non_claim, SOURCE_BUNDLE_NON_CLAIM);
    let manifest_entries = manifest.records.len();
    let mut matching_sources = Vec::with_capacity(manifest_entries);
    let mut missing_source_ids = Vec::with_capacity(manifest_entries);
    let mut stale_source_ids = Vec::with_capacity(manifest_entries);
    let mut rejected_adapter_ids = Vec::with_capacity(manifest_entries);
    let mut untrusted_source_ids = Vec::with_capacity(manifest_entries);
    let mut network_required_source_ids = Vec::with_capacity(manifest_entries);
    let mut unpinned_source_ids = Vec::with_capacity(manifest_entries);
    let mut summaries = Vec::with_capacity(manifest_entries);
    for expected in &manifest.records {
        summaries.push(summary_for_record(expected)?);
        classify_adapter_readiness(expected, &mut rejected_adapter_ids, &mut untrusted_source_ids);
        let stored_record = find_matching_source_record(expected, available_sources);
        if source_record_requires_network(expected, stored_record) {
            network_required_source_ids.push(expected.identity.clone());
            continue;
        }
        let Some(stored_record) = stored_record else {
            missing_source_ids.push(expected.identity.clone());
            continue;
        };
        if source_record_is_stale_for_preflight(expected, stored_record) {
            stale_source_ids.push(expected.identity.clone());
            continue;
        }
        if !source_record_is_pinned(expected, stored_record, pinned_sources) {
            unpinned_source_ids.push(expected.identity.clone());
        }
        matching_sources.push(stored_record.clone());
    }
    let blockers = OfflineBlockerSets {
        missing_ids: &missing_source_ids,
        stale_ids: &stale_source_ids,
        unsupported_ids: &rejected_adapter_ids,
        untrusted_ids: &untrusted_source_ids,
        network_required_ids: &network_required_source_ids,
        unpinned_ids: &unpinned_source_ids,
    };
    let ready_class = classify_offline_preflight_state(&blockers);
    let next_actions = offline_preflight_next_actions(&blockers);
    Ok(SourceOfflinePreflightReport {
        format: SOURCE_OFFLINE_PREFLIGHT_FORMAT,
        manifest_blake3: Some(manifest.manifest_blake3.clone()),
        source_state_blake3: digest_offline_preflight_state(Some(&manifest.manifest_blake3), &matching_sources)?,
        ready_class,
        record_count: checked_u32(manifest_entries, "source preflight record count")?,
        missing_records: missing_source_ids,
        stale_records: stale_source_ids,
        unsupported_records: rejected_adapter_ids,
        untrusted_records: untrusted_source_ids,
        network_required_records: network_required_source_ids,
        unpinned_records: unpinned_source_ids,
        next_actions,
        records: summaries,
        non_claim: SOURCE_BUNDLE_NON_CLAIM,
    })
}

fn empty_offline_preflight_report() -> Result<SourceOfflinePreflightReport, RunError> {
    Ok(SourceOfflinePreflightReport {
        format: SOURCE_OFFLINE_PREFLIGHT_FORMAT,
        manifest_blake3: None,
        source_state_blake3: blake3::hash(SOURCE_OFFLINE_PREFLIGHT_EMPTY_MARKER).to_hex().to_string(),
        ready_class: SourceReadiness::Ready,
        record_count: 0,
        missing_records: Vec::new(),
        stale_records: Vec::new(),
        unsupported_records: Vec::new(),
        untrusted_records: Vec::new(),
        network_required_records: Vec::new(),
        unpinned_records: Vec::new(),
        next_actions: Vec::new(),
        records: Vec::new(),
        non_claim: SOURCE_BUNDLE_NON_CLAIM,
    })
}

fn classify_offline_preflight_state(blockers: &OfflineBlockerSets<'_>) -> SourceReadiness {
    assert!(blockers.missing_ids.len() <= MAX_SOURCE_RECORDS);
    assert!(blockers.stale_ids.len() <= MAX_SOURCE_RECORDS);
    if !blockers.unsupported_ids.is_empty() {
        return SourceReadiness::Unsupported;
    }
    if !blockers.stale_ids.is_empty() {
        return SourceReadiness::Stale;
    }
    if !blockers.untrusted_ids.is_empty() {
        return SourceReadiness::Untrusted;
    }
    if !blockers.network_required_ids.is_empty() {
        return SourceReadiness::NetworkRequired;
    }
    if !blockers.missing_ids.is_empty() {
        return SourceReadiness::Missing;
    }
    if !blockers.unpinned_ids.is_empty() {
        return SourceReadiness::Unpinned;
    }
    SourceReadiness::Ready
}

fn offline_preflight_next_actions(blockers: &OfflineBlockerSets<'_>) -> Vec<SourceOfflinePreflightNextAction> {
    let rules = [
        OfflineNextActionRule {
            is_enabled: !blockers.missing_ids.is_empty(),
            blocker_class: "missing-source-state",
            command_hint: SOURCE_NEXT_ACTION_EXPORT_IMPORT_PIN,
            description: "export, transfer, import, and pin the source bundle for the selected root",
        },
        OfflineNextActionRule {
            is_enabled: !blockers.stale_ids.is_empty(),
            blocker_class: "stale-source-state",
            command_hint: SOURCE_NEXT_ACTION_REEXPORT_IMPORT_PIN,
            description: "refresh stale source state from the current root inputs before retrying offline preflight",
        },
        OfflineNextActionRule {
            is_enabled: !blockers.unsupported_ids.is_empty(),
            blocker_class: "unsupported-source-adapter",
            command_hint: SOURCE_NEXT_ACTION_INSPECT_ADAPTER,
            description: "offline preflight rejects adapters outside the supported source-bundle surface",
        },
        OfflineNextActionRule {
            is_enabled: !blockers.untrusted_ids.is_empty(),
            blocker_class: "untrusted-source-adapter",
            command_hint: SOURCE_NEXT_ACTION_TRUST_PROVENANCE,
            description: "trusted-provenance metadata is required before source readiness can be accepted",
        },
        OfflineNextActionRule {
            is_enabled: !blockers.network_required_ids.is_empty(),
            blocker_class: "network-required-source",
            command_hint: SOURCE_NEXT_ACTION_DECLARE_SOURCE,
            description: "offline mode requires declared local source state instead of live fetches",
        },
        OfflineNextActionRule {
            is_enabled: !blockers.unpinned_ids.is_empty(),
            blocker_class: "unpinned-source-state",
            command_hint: SOURCE_NEXT_ACTION_PIN_IMPORTED,
            description: "pin imported source records before treating source readiness as current evidence",
        },
    ];
    assert_eq!(rules.len(), OFFLINE_BLOCKER_CLASS_COUNT);
    let mut actions = Vec::with_capacity(OFFLINE_BLOCKER_CLASS_COUNT);
    for rule in rules {
        if rule.is_enabled {
            actions.push(SourceOfflinePreflightNextAction {
                blocker_class: rule.blocker_class,
                command_hint: rule.command_hint,
                description: rule.description,
            });
        }
    }
    assert!(actions.len() <= OFFLINE_BLOCKER_CLASS_COUNT);
    actions
}

fn source_record_requires_network(expected: &SourceRecord, imported: Option<&SourceRecord>) -> bool {
    if let Some(imported) = imported
        && !imported.files.is_empty()
    {
        return false;
    }
    if !source_record_is_fetcher_input(expected) {
        return false;
    }
    let Some(url) = expected.metadata.get(RECORD_METADATA_URL_KEY) else {
        return false;
    };
    local_file_url_path(url).is_none()
}

fn classify_adapter_readiness(
    record: &SourceRecord,
    unsupported_records: &mut Vec<String>,
    untrusted_records: &mut Vec<String>,
) {
    if adapter_declares_unsupported(record) {
        unsupported_records.push(record.identity.clone());
    }
    if adapter_declares_untrusted(record) {
        untrusted_records.push(record.identity.clone());
    }
}

fn adapter_declares_unsupported(record: &SourceRecord) -> bool {
    record
        .adapter
        .as_ref()
        .and_then(|adapter| adapter.extra.get(ADAPTER_EXTRA_UNSUPPORTED_KEY))
        .map(String::as_str)
        == Some(ADAPTER_EXTRA_TRUE_VALUE)
}

fn adapter_declares_untrusted(record: &SourceRecord) -> bool {
    record
        .adapter
        .as_ref()
        .and_then(|adapter| adapter.extra.get(ADAPTER_EXTRA_TRUSTED_PROVENANCE_KEY))
        .is_some_and(|value| value != ADAPTER_EXTRA_TRUE_VALUE)
}

fn source_record_is_stale_for_preflight(expected: &SourceRecord, imported: &SourceRecord) -> bool {
    if !source_record_metadata_matches_preflight(expected, imported) {
        return true;
    }
    if expected.files.is_empty() {
        return false;
    }
    expected.content_blake3 != imported.content_blake3 || expected.files != imported.files
}

fn source_record_metadata_matches_preflight(expected: &SourceRecord, imported: &SourceRecord) -> bool {
    expected.kind == imported.kind
        && expected.identity == imported.identity
        && expected.store_prefix == imported.store_prefix
        && expected.adapter == imported.adapter
        && expected.metadata == imported.metadata
}

fn source_record_is_pinned(expected: &SourceRecord, imported: &SourceRecord, pinned_records: &[SourceRecord]) -> bool {
    pinned_records.iter().any(|pinned| {
        source_record_metadata_matches_preflight(expected, pinned) && pinned.content_blake3 == imported.content_blake3
    })
}

fn find_matching_source_record<'a>(expected: &SourceRecord, records: &'a [SourceRecord]) -> Option<&'a SourceRecord> {
    records
        .iter()
        .find(|stored| source_record_metadata_matches_preflight(expected, stored) && !stored.files.is_empty())
        .or_else(|| records.iter().find(|stored| source_record_metadata_matches_preflight(expected, stored)))
        .or_else(|| records.iter().find(|stored| stored.identity == expected.identity))
}

fn digest_offline_preflight_state(
    manifest_blake3: Option<&str>,
    matching_records: &[SourceRecord],
) -> Result<String, RunError> {
    let mut records = matching_records.to_vec();
    records.sort_by_key(record_sort_key);
    let mut hasher = blake3::Hasher::new();
    hasher.update(SOURCE_OFFLINE_PREFLIGHT_STATE_MARKER);
    hasher.update(manifest_blake3.unwrap_or("none").as_bytes());
    hasher.update(b"\n");
    for record in &records {
        let encoded = serde_json::to_vec(record)
            .map_err(|err| RunError::Internal(format!("serializing source preflight state: {err}")))?;
        hasher.update(&encoded);
        hasher.update(b"\n");
    }
    Ok(hasher.finalize().to_hex().to_string())
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
    let record = serde_json::from_slice(&bytes)
        .map_err(|err| RunError::Internal(format!("parsing source record {}: {err}", path.display())))?;
    validate_source_record(&record)?;
    Ok(record)
}

fn read_imported_source_records(state_dir: &Path) -> Result<Vec<SourceRecord>, RunError> {
    read_source_records_from_dir(&source_records_dir(state_dir), "source record")
}

fn read_pinned_source_records(state_dir: &Path) -> Result<Vec<SourceRecord>, RunError> {
    let pins_dir = source_pins_dir(state_dir);
    let paths = sorted_json_paths(&pins_dir, "source pin")?;
    assert!(paths.len() <= MAX_SOURCE_RECORDS);
    assert!(pins_dir.starts_with(state_dir));
    let mut records = Vec::new();
    for path in &paths {
        let bytes = fs::read(path)
            .map_err(|err| RunError::Internal(format!("reading source pin {}: {err}", path.display())))?;
        let manifest = serde_json::from_slice::<SourceBundleManifest>(&bytes)
            .map_err(|err| RunError::Internal(format!("parsing source pin {}: {err}", path.display())))?;
        validate_manifest(&manifest)?;
        let additional_len = manifest.records.len();
        let next_len = records
            .len()
            .checked_add(additional_len)
            .ok_or_else(|| RunError::Internal("source pin record count overflow".to_string()))?;
        if next_len > MAX_SOURCE_RECORDS {
            return Err(RunError::Internal(format!("source pin records exceed {MAX_SOURCE_RECORDS}")));
        }
        records
            .try_reserve(additional_len)
            .map_err(|err| RunError::Internal(format!("reserving pinned source records: {err}")))?;
        for record in manifest.records {
            if records.len() >= MAX_SOURCE_RECORDS {
                return Err(RunError::Internal(format!("source pin records exceed {MAX_SOURCE_RECORDS}")));
            }
            records.push(record);
        }
    }
    assert!(records.len() <= MAX_SOURCE_RECORDS);
    Ok(records)
}

fn read_source_records_from_dir(dir: &Path, label: &str) -> Result<Vec<SourceRecord>, RunError> {
    let paths = sorted_json_paths(dir, label)?;
    let mut records = Vec::with_capacity(paths.len());
    for path in &paths {
        records.push(read_record(path)?);
    }
    Ok(records)
}

fn sorted_json_paths(dir: &Path, label: &str) -> Result<Vec<PathBuf>, RunError> {
    match fs::read_dir(dir) {
        Ok(entries) => {
            let mut paths = Vec::with_capacity(MAX_SOURCE_RECORDS);
            for entry in entries {
                let entry =
                    entry.map_err(|err| RunError::Internal(format!("reading {label} dir {}: {err}", dir.display())))?;
                let path = entry.path();
                if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                    if paths.len() >= MAX_SOURCE_RECORDS {
                        return Err(RunError::Internal(format!("{label} count exceeds {MAX_SOURCE_RECORDS}")));
                    }
                    paths.push(path);
                }
            }
            paths.sort();
            Ok(paths)
        }
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(Vec::new()),
        Err(err) => Err(RunError::Internal(format!("reading {label} dir {}: {err}", dir.display()))),
    }
}

fn source_fetch_overrides_for_manifest(
    manifest: &SourceBundleManifest,
    available_sources: &[SourceRecord],
    pinned_sources: &[SourceRecord],
    source_state_blake3: &str,
) -> Result<(Vec<crunch_build::FetchSourceOverride>, Vec<tempfile::TempDir>), RunError> {
    let manifest_entries = manifest.records.len();
    let mut overrides = Vec::with_capacity(manifest_entries);
    let mut scratch_dirs = Vec::with_capacity(manifest_entries);
    let mut override_payloads = BTreeMap::new();
    for expected in &manifest.records {
        if !source_record_is_fetcher_input(expected) {
            continue;
        }
        let stored_record = imported_source_record_satisfies_fetcher_input(expected, available_sources, pinned_sources)
            .map_err(|blocker| RunError::Internal(format!("{} for {}", blocker.reason_code(), expected.identity)))?;
        let payload_digest = digest_source_entries(&stored_record.files)?;
        let (source_override, scratch_dir) = source_fetch_override_for_record(stored_record, source_state_blake3)?;
        let override_key = source_fetch_override_key(&source_override);
        if !admit_source_override_mapping(&mut override_payloads, override_key, payload_digest, &source_override.url)? {
            continue;
        }
        overrides.push(source_override);
        scratch_dirs.push(scratch_dir);
    }
    Ok((overrides, scratch_dirs))
}

type SourceOverrideKey = (&'static str, String, Option<String>);

fn admit_source_override_mapping(
    override_payloads: &mut BTreeMap<SourceOverrideKey, String>,
    key: SourceOverrideKey,
    payload_digest: String,
    url: &str,
) -> Result<bool, RunError> {
    if let Some(existing_digest) = override_payloads.get(&key) {
        if existing_digest != &payload_digest {
            return Err(RunError::Internal(format!("conflicting source override mapping for {url}")));
        }
        return Ok(false);
    }
    override_payloads.insert(key, payload_digest);
    assert!(!override_payloads.is_empty());
    debug_assert!(override_payloads.len() <= MAX_SOURCE_RECORDS);
    Ok(true)
}

fn source_fetch_override_key(source_override: &crunch_build::FetchSourceOverride) -> SourceOverrideKey {
    let kind = match source_override.kind {
        crunch_build::FetchSourceOverrideKind::File => "file",
        crunch_build::FetchSourceOverrideKind::Tarball => "tarball",
        crunch_build::FetchSourceOverrideKind::Executable => "executable",
        crunch_build::FetchSourceOverrideKind::Git => "git",
    };
    (kind, source_override.url.clone(), source_override.rev.clone())
}

fn source_record_is_fetcher_input(record: &SourceRecord) -> bool {
    matches!(record.kind, SourceRecordKind::FixedUrl | SourceRecordKind::VcsSnapshot)
        || source_record_is_legacy_provider_fetch(record)
}

fn source_record_is_legacy_provider_fetch(record: &SourceRecord) -> bool {
    record.kind == SourceRecordKind::BootstrapArchive
        && record.metadata.get(RECORD_METADATA_URL_KEY).map(String::as_str)
            == Some(crate::bootstrap_source_root::LEGACY_MUSL_CC_URL)
        && record.metadata.get(RECORD_METADATA_HASH_KEY).map(String::as_str)
            == Some(crate::bootstrap_source_root::LEGACY_MUSL_CC_HASH)
}

fn bootstrap_provider_archive_record_matches(record: &SourceRecord, mode: BootstrapSourceBundleMode) -> bool {
    let record_mode = record.metadata.get(RECORD_METADATA_PROFILE_MODE_KEY).map(String::as_str);
    let is_legacy_compatible_mode = record_mode == Some(BootstrapSourceBundleMode::SelfBuildProof.as_str())
        || record_mode == Some(BootstrapSourceBundleMode::FreshCloneInputs.as_str())
        || record_mode == Some(BootstrapSourceBundleMode::FreshCloneFixedPoint.as_str());
    let is_mode_match = record_mode == Some(mode.as_str())
        || (mode == BootstrapSourceBundleMode::LegacySeed && is_legacy_compatible_mode);
    record.kind == SourceRecordKind::BootstrapArchive
        && is_mode_match
        && record.metadata.get(RECORD_METADATA_PROFILE_CLASS_KEY).map(String::as_str)
            == Some(BOOTSTRAP_PROFILE_CLASS_PROVIDER_ARCHIVE)
}

fn imported_source_record_satisfies_fetcher_input<'a>(
    expected: &SourceRecord,
    available_sources: &'a [SourceRecord],
    pinned_sources: &[SourceRecord],
) -> Result<&'a SourceRecord, SourceFetchBlocker> {
    if adapter_declares_unsupported(expected) {
        return Err(SourceFetchBlocker::UnsupportedSourceAdapter);
    }
    if adapter_declares_untrusted(expected) {
        return Err(SourceFetchBlocker::UntrustedSourceAdapter);
    }
    let stored_record = find_matching_source_record(expected, available_sources);
    if source_record_requires_network(expected, stored_record) {
        return Err(SourceFetchBlocker::NetworkRequiredSource);
    }
    let Some(stored_record) = stored_record else {
        return Err(SourceFetchBlocker::MissingSourceState);
    };
    if source_record_is_stale_for_preflight(expected, stored_record) {
        return Err(SourceFetchBlocker::StaleSourceState);
    }
    if !source_record_is_pinned(expected, stored_record, pinned_sources) {
        return Err(SourceFetchBlocker::UnpinnedSourceState);
    }
    if stored_record.files.is_empty() {
        return Err(SourceFetchBlocker::MissingMaterializedPayload);
    }
    assert!(source_record_is_pinned(expected, stored_record, pinned_sources));
    assert!(!stored_record.files.is_empty());
    Ok(stored_record)
}

fn source_fetch_override_for_record(
    record: &SourceRecord,
    source_state_blake3: &str,
) -> Result<(crunch_build::FetchSourceOverride, tempfile::TempDir), RunError> {
    validate_source_record(record)?;
    assert!(!record.identity.is_empty());
    assert!(record.files.len() <= MAX_SOURCE_FILES_PER_RECORD);
    let kind = source_fetch_override_kind(record)?;
    let url = record
        .metadata
        .get(RECORD_METADATA_URL_KEY)
        .ok_or_else(|| RunError::Internal(format!("source record {} is missing url metadata", record.identity)))?
        .clone();
    let rev = if kind == crunch_build::FetchSourceOverrideKind::Git {
        Some(
            record
                .metadata
                .get(FETCH_ENV_REV_KEY)
                .ok_or_else(|| {
                    RunError::Internal(format!("VCS source record {} is missing revision", record.identity))
                })?
                .clone(),
        )
    } else {
        None
    };
    let scratch_dir = tempfile::Builder::new()
        .prefix("mantle-source-fetch-")
        .tempdir()
        .map_err(|err| RunError::Internal(format!("creating source fetch scratch dir: {err}")))?;
    let payload_path = scratch_dir.path().join("payload");
    materialize_source_record_for_fetch_override(record, kind, &payload_path)?;
    Ok((
        crunch_build::FetchSourceOverride {
            url,
            kind,
            rev,
            payload_path,
            source_state_blake3: source_state_blake3.to_string(),
        },
        scratch_dir,
    ))
}

fn source_fetch_override_kind(record: &SourceRecord) -> Result<crunch_build::FetchSourceOverrideKind, RunError> {
    match record.kind {
        SourceRecordKind::VcsSnapshot => Ok(crunch_build::FetchSourceOverrideKind::Git),
        SourceRecordKind::FixedUrl => {
            if record.metadata.get(FETCH_ENV_UNPACK_KEY).map(String::as_str) == Some("1") {
                return Ok(crunch_build::FetchSourceOverrideKind::Tarball);
            }
            if record.metadata.get(FETCH_ENV_EXECUTABLE_KEY).map(String::as_str) == Some("1") {
                return Ok(crunch_build::FetchSourceOverrideKind::Executable);
            }
            Ok(crunch_build::FetchSourceOverrideKind::File)
        }
        SourceRecordKind::BootstrapArchive if source_record_is_legacy_provider_fetch(record) => {
            if record.metadata.get(FETCH_ENV_UNPACK_KEY).map(String::as_str) != Some("1") {
                return Err(RunError::Internal(format!(
                    "legacy provider source record {} is not an unpacked fetch",
                    record.identity
                )));
            }
            Ok(crunch_build::FetchSourceOverrideKind::Tarball)
        }
        SourceRecordKind::LocalPath
        | SourceRecordKind::PackageMirror
        | SourceRecordKind::BootstrapArchive
        | SourceRecordKind::ProviderManifest
        | SourceRecordKind::ToolchainSourceRoot
        | SourceRecordKind::ProofInput => {
            Err(RunError::Internal(format!("source record {} is not a fixed fetcher input", record.identity)))
        }
    }
}

pub(crate) fn materialize_source_record_for_offline_use(record: &SourceRecord, target: &Path) -> Result<(), RunError> {
    let kind = source_fetch_override_kind(record)?;
    materialize_source_record_for_fetch_override(record, kind, target)?;
    let expected_hash = expected_source_record_hash(record)?;
    verify_captured_source_record(record, target, &expected_hash)?;
    assert!(target.exists());
    assert!(!record.files.is_empty());
    Ok(())
}

fn materialize_source_record_for_fetch_override(
    record: &SourceRecord,
    kind: crunch_build::FetchSourceOverrideKind,
    payload_path: &Path,
) -> Result<(), RunError> {
    match kind {
        crunch_build::FetchSourceOverrideKind::File | crunch_build::FetchSourceOverrideKind::Executable => {
            materialize_flat_fetch_record_payload(record, payload_path)
        }
        crunch_build::FetchSourceOverrideKind::Tarball if source_record_uses_tarball_archive_payload(record) => {
            materialize_tarball_archive_fetch_record(record, payload_path)
        }
        crunch_build::FetchSourceOverrideKind::Tarball | crunch_build::FetchSourceOverrideKind::Git => {
            materialize_source_record_payload(record, payload_path)
        }
    }
}

fn materialize_tarball_archive_fetch_record(record: &SourceRecord, payload_path: &Path) -> Result<(), RunError> {
    validate_source_record(record)?;
    let url = record
        .metadata
        .get(RECORD_METADATA_URL_KEY)
        .ok_or_else(|| RunError::Internal(format!("source record {} is missing URL", record.identity)))?;
    let archive_path = payload_path.with_extension("source-archive");
    materialize_flat_fetch_record_payload(record, &archive_path)?;
    let payload_text = payload_path
        .to_str()
        .ok_or_else(|| RunError::Internal("offline tarball output path is not UTF-8".to_string()))?;
    crunch_build::fetcher::unpack_archive_file(url, &archive_path, payload_text)
        .map_err(|error| RunError::Internal(format!("unpacking offline source archive {}: {error}", record.identity)))
}

fn materialize_flat_fetch_record_payload(record: &SourceRecord, payload_path: &Path) -> Result<(), RunError> {
    validate_source_record(record)?;
    if record.files.is_empty() {
        return Err(RunError::Internal(format!("flat source record {} has no file payload", record.identity)));
    }
    let path = record.files[0].path.as_str();
    if record.files.iter().any(|file| file.path != path || file.file_type != SourceFileType::Regular) {
        return Err(RunError::Internal(format!(
            "flat source record {} must contain exactly one logical regular file payload",
            record.identity
        )));
    }
    if record.files[0].chunk_index.is_some() {
        return materialize_chunked_regular_file_entries_at_path(&record.files, payload_path);
    }
    if record.files.len() != 1 {
        return Err(RunError::Internal(format!(
            "flat source record {} must contain exactly one file payload",
            record.identity
        )));
    }
    materialize_regular_file_entry(&record.files[0], payload_path)
}

fn materialize_chunked_regular_file_entries_at_path(
    files: &[SourceFileEntry],
    output_path: &Path,
) -> Result<(), RunError> {
    assert!(files.len() >= usize::try_from(MIN_SOURCE_FILE_CHUNK_COUNT).expect("chunk count fits in usize"));
    let mut output = fs::File::create(output_path)
        .map_err(|err| RunError::Internal(format!("creating chunked source file {}: {err}", output_path.display())))?;
    for file in files {
        let content = decode_regular_file_content(file)?;
        output.write_all(&content).map_err(|err| {
            RunError::Internal(format!("writing chunked source file {}: {err}", output_path.display()))
        })?;
    }
    output
        .flush()
        .map_err(|err| RunError::Internal(format!("flushing chunked source file {}: {err}", output_path.display())))?;
    set_materialized_file_permissions(output_path, files[0].executable)
}

fn materialize_export_records(
    records: &[SourceRecord],
    imported_records: &[SourceRecord],
) -> Result<Vec<SourceRecord>, RunError> {
    let mut materialized = Vec::with_capacity(records.len());
    for record in records {
        let captured = if source_record_is_legacy_provider_fetch(record) {
            materialize_fetcher_record(record, imported_records, false)?
        } else {
            match record.kind {
                SourceRecordKind::FixedUrl => materialize_fetcher_record(record, imported_records, false)?,
                SourceRecordKind::VcsSnapshot => materialize_fetcher_record(record, imported_records, true)?,
                _ => record.clone(),
            }
        };
        materialized.push(captured);
    }
    Ok(materialized)
}

fn materialize_export_records_with_connected_fetch(
    records: &[SourceRecord],
    imported_records: &[SourceRecord],
) -> Result<Vec<SourceRecord>, RunError> {
    let mut materialized = Vec::with_capacity(records.len());
    let mut reusable_records = imported_records.to_vec();
    for record in records {
        let captured = if source_record_is_fetcher_input(record) {
            if let Some(stored) = find_matching_materialized_source_record(record, &reusable_records) {
                verify_imported_record_fixed_output(record, stored)?;
                stored.clone()
            } else {
                let fetched = fetch_and_materialize_source_record(record)?;
                reusable_records.push(fetched.clone());
                fetched
            }
        } else {
            record.clone()
        };
        materialized.push(captured);
    }
    Ok(materialized)
}

fn verify_imported_record_fixed_output(expected: &SourceRecord, stored: &SourceRecord) -> Result<(), RunError> {
    let expected_hash = expected_source_record_hash(expected)?;
    let kind = source_fetch_override_kind(stored)?;
    let scratch = tempfile::Builder::new()
        .prefix("mantle-source-imported-verify-")
        .tempdir()
        .map_err(|error| RunError::Internal(format!("creating imported source verification scratch: {error}")))?;
    let output = scratch.path().join("output");
    materialize_source_record_for_fetch_override(stored, kind, &output)?;
    verify_captured_source_record(expected, &output, &expected_hash)
}

fn fetch_and_materialize_source_record(record: &SourceRecord) -> Result<SourceRecord, RunError> {
    let expected_hash = expected_source_record_hash(record)?;
    let scratch = tempfile::Builder::new()
        .prefix("mantle-source-connected-fetch-")
        .tempdir()
        .map_err(|error| RunError::Internal(format!("creating connected source fetch scratch: {error}")))?;
    if source_record_uses_tarball_archive_payload(record) {
        return capture_tarball_archive_source_record(record, &expected_hash, scratch.path());
    }
    let fetch = fetch_from_source_record(record, &expected_hash)?;
    let output = scratch.path().join("output");
    let output_text = output
        .to_str()
        .ok_or_else(|| RunError::Internal("connected source fetch path is not UTF-8".to_string()))?;
    crunch_build::fetcher::fetch_to_store(&fetch, output_text)
        .map_err(|error| RunError::Internal(format!("capturing source record {}: {error}", record.identity)))?;
    verify_captured_source_record(record, &output, &expected_hash)?;
    materialize_source_record_from_path(record, &output, record.kind == SourceRecordKind::VcsSnapshot)
}

fn capture_tarball_archive_source_record(
    record: &SourceRecord,
    expected_hash: &NixHash,
    scratch: &Path,
) -> Result<SourceRecord, RunError> {
    let url = record
        .metadata
        .get(RECORD_METADATA_URL_KEY)
        .ok_or_else(|| RunError::Internal(format!("source record {} is missing URL", record.identity)))?;
    let archive = scratch.join("archive");
    let output = scratch.join("output");
    let archive_text = archive
        .to_str()
        .ok_or_else(|| RunError::Internal("connected source archive path is not UTF-8".to_string()))?;
    let output_text = output
        .to_str()
        .ok_or_else(|| RunError::Internal("connected source output path is not UTF-8".to_string()))?;
    crunch_build::fetcher::fetch_raw_to_file(url, archive_text)
        .map_err(|error| RunError::Internal(format!("capturing source archive {}: {error}", record.identity)))?;
    crunch_build::fetcher::unpack_archive_file(url, &archive, output_text).map_err(|error| {
        RunError::Internal(format!("unpacking captured source archive {}: {error}", record.identity))
    })?;
    verify_captured_source_record(record, &output, expected_hash)?;
    materialize_source_record_from_path(record, &archive, false)
}

fn expected_source_record_hash(record: &SourceRecord) -> Result<NixHash, RunError> {
    let algo_text = record
        .metadata
        .get(RECORD_METADATA_HASH_ALGO_KEY)
        .ok_or_else(|| RunError::Internal(format!("source record {} is missing hash algorithm", record.identity)))?;
    let algo = HashAlgo::try_from(algo_text.as_str()).map_err(|error| {
        RunError::Internal(format!("source record {} has invalid hash algorithm: {error}", record.identity))
    })?;
    let hash_text = record
        .metadata
        .get(RECORD_METADATA_HASH_KEY)
        .ok_or_else(|| RunError::Internal(format!("source record {} is missing expected hash", record.identity)))?;
    if hash_text.contains('-') {
        return NixHash::from_sri(hash_text).map_err(|error| {
            RunError::Internal(format!("source record {} has invalid SRI hash: {error}", record.identity))
        });
    }
    let digest = HEXLOWER.decode(hash_text.as_bytes()).map_err(|error| {
        RunError::Internal(format!("source record {} has invalid hex hash: {error}", record.identity))
    })?;
    NixHash::from_algo_and_digest(algo, &digest).map_err(|error| {
        RunError::Internal(format!("source record {} has invalid hash digest: {error}", record.identity))
    })
}

fn fetch_from_source_record(record: &SourceRecord, expected_hash: &NixHash) -> Result<crunch_build::Fetch, RunError> {
    let url_text = record
        .metadata
        .get(RECORD_METADATA_URL_KEY)
        .ok_or_else(|| RunError::Internal(format!("source record {} is missing URL", record.identity)))?;
    if record.kind == SourceRecordKind::VcsSnapshot {
        let rev = record
            .metadata
            .get(FETCH_ENV_REV_KEY)
            .ok_or_else(|| RunError::Internal(format!("source record {} is missing Git revision", record.identity)))?;
        return Ok(crunch_build::Fetch::Git {
            url: url_text.clone(),
            rev: rev.clone(),
            exp_hash: Some(expected_hash.clone()),
        });
    }
    let url = Url::parse(url_text)
        .map_err(|error| RunError::Internal(format!("source record {} has invalid URL: {error}", record.identity)))?;
    if record.metadata.get(FETCH_ENV_UNPACK_KEY).map(String::as_str) == Some("1") {
        return Ok(crunch_build::Fetch::Tarball {
            url,
            exp_nar_sha256: None,
        });
    }
    if record.metadata.get(FETCH_ENV_EXECUTABLE_KEY).map(String::as_str) == Some("1") {
        return Ok(crunch_build::Fetch::Executable {
            url,
            hash: expected_hash.clone(),
        });
    }
    Ok(crunch_build::Fetch::Url {
        url,
        exp_hash: Some(expected_hash.clone()),
    })
}

fn verify_captured_source_record(
    record: &SourceRecord,
    output: &Path,
    expected_hash: &NixHash,
) -> Result<(), RunError> {
    let mode = record
        .metadata
        .get(RECORD_METADATA_HASH_MODE_KEY)
        .ok_or_else(|| RunError::Internal(format!("source record {} is missing hash mode", record.identity)))?;
    let verification = match mode.as_str() {
        "flat" => crunch_build::fetcher::verify_flat_hash(
            output
                .to_str()
                .ok_or_else(|| RunError::Internal("connected source output path is not UTF-8".to_string()))?,
            expected_hash,
            &record.identity,
        ),
        "recursive" => crunch_build::fetcher::verify_recursive_hash(output, expected_hash, &record.identity),
        other => {
            return Err(RunError::Internal(format!(
                "source record {} has unsupported hash mode {other}",
                record.identity
            )));
        }
    };
    verification
        .map_err(|error| RunError::Internal(format!("validating captured source record {}: {error}", record.identity)))
}

fn materialize_fetcher_record(
    record: &SourceRecord,
    available_sources: &[SourceRecord],
    is_skipping_git_dir: bool,
) -> Result<SourceRecord, RunError> {
    if let Some(stored_record) = find_matching_materialized_source_record(record, available_sources) {
        return Ok(stored_record.clone());
    }
    materialize_file_url_record(record, is_skipping_git_dir)
}

fn find_matching_materialized_source_record<'a>(
    expected: &SourceRecord,
    records: &'a [SourceRecord],
) -> Option<&'a SourceRecord> {
    records
        .iter()
        .find(|stored| source_record_metadata_matches_preflight(expected, stored) && !stored.files.is_empty())
}

fn materialize_file_url_record(record: &SourceRecord, is_skipping_git_dir: bool) -> Result<SourceRecord, RunError> {
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
    if record.kind == SourceRecordKind::VcsSnapshot {
        verify_local_vcs_snapshot_revision(record, &payload_path)?;
    }
    materialize_source_record_from_path(record, &payload_path, is_skipping_git_dir)
}

fn verify_local_vcs_snapshot_revision(record: &SourceRecord, checkout_path: &Path) -> Result<(), RunError> {
    let requested_rev = record.metadata.get(FETCH_ENV_REV_KEY).ok_or_else(|| {
        RunError::Internal(format!("VCS source record {} is missing revision metadata", record.identity))
    })?;
    let git_dir = local_checkout_git_dir(checkout_path)?;
    let head_revision = resolve_git_revision(&git_dir, GIT_HEAD_REF)?;
    let requested_revision = resolve_git_revision(&git_dir, requested_rev)?;
    if head_revision != requested_revision {
        return Err(RunError::Internal(format!(
            "VCS source record {} checkout revision mismatch: {head_revision} != {requested_revision}",
            record.identity
        )));
    }
    Ok(())
}

fn local_checkout_git_dir(checkout_path: &Path) -> Result<PathBuf, RunError> {
    let dot_git = checkout_path.join(DOT_GIT_DIR_NAME);
    assert!(dot_git.starts_with(checkout_path));
    assert!(dot_git.ends_with(DOT_GIT_DIR_NAME));
    let metadata = fs::symlink_metadata(&dot_git)
        .map_err(|err| RunError::Internal(format!("reading local VCS metadata {}: {err}", dot_git.display())))?;
    if metadata.is_dir() {
        return Ok(dot_git);
    }
    if metadata.is_file() {
        let pointer = fs::read_to_string(&dot_git).map_err(|err| {
            RunError::Internal(format!("reading local VCS gitdir pointer {}: {err}", dot_git.display()))
        })?;
        let target = pointer.trim().strip_prefix(GIT_DIR_POINTER_PREFIX).ok_or_else(|| {
            RunError::Internal(format!("local VCS metadata {} is not a gitdir pointer", dot_git.display()))
        })?;
        if target.is_empty() || target.contains('\0') {
            return Err(RunError::Internal(format!("unsafe local VCS gitdir pointer in {}", dot_git.display())));
        }
        let target_path = PathBuf::from(target.trim());
        if target_path.is_absolute() {
            return Ok(target_path);
        }
        return Ok(checkout_path.join(target_path));
    }
    Err(RunError::Internal(format!("local VCS metadata {} is not a file or dir", dot_git.display())))
}

fn resolve_git_revision(git_dir: &Path, revision: &str) -> Result<String, RunError> {
    validate_git_revision_text(revision)?;
    assert!(!revision.is_empty());
    assert!(!revision.contains('\0'));
    let mut current_revision = revision.to_string();
    for _ in 0..MAX_GIT_REF_INDIRECTIONS {
        validate_git_revision_text(&current_revision)?;
        if is_git_object_id(&current_revision) {
            return Ok(current_revision);
        }
        let ref_text = read_git_ref_text(git_dir, &current_revision)?;
        if let Some(target_ref) = ref_text.strip_prefix(GIT_REF_PREFIX) {
            current_revision = target_ref.trim().to_string();
            continue;
        }
        if is_git_object_id(&ref_text) {
            return Ok(ref_text);
        }
        return Err(RunError::Internal(format!("local VCS ref {current_revision} does not resolve to an object id")));
    }
    Err(RunError::Internal(format!(
        "local VCS ref {revision} exceeds {MAX_GIT_REF_INDIRECTIONS} symbolic indirections"
    )))
}

fn read_git_ref_text(git_dir: &Path, rev: &str) -> Result<String, RunError> {
    let ref_path = git_dir.join(rev);
    match fs::read_to_string(&ref_path) {
        Ok(text) => return Ok(text.trim().to_string()),
        Err(err) if err.kind() == ErrorKind::NotFound => {}
        Err(err) => {
            return Err(RunError::Internal(format!("reading local VCS ref {}: {err}", ref_path.display())));
        }
    }
    read_git_packed_ref(git_dir, rev)
}

fn read_git_packed_ref(git_dir: &Path, rev: &str) -> Result<String, RunError> {
    let packed_refs_path = git_dir.join(GIT_PACKED_REFS_FILE);
    let text = fs::read_to_string(&packed_refs_path).map_err(|err| {
        RunError::Internal(format!("reading local VCS packed refs {} for {rev}: {err}", packed_refs_path.display()))
    })?;
    for line in text.lines() {
        if git_packed_ref_line_matches(GitPackedRefMatch { line, revision: rev }) {
            let object_id = line
                .split_whitespace()
                .next()
                .ok_or_else(|| RunError::Internal(format!("local VCS packed ref {rev} is malformed")))?;
            validate_git_object_id(object_id)?;
            return Ok(object_id.to_string());
        }
    }
    Err(RunError::Internal(format!("local VCS ref {rev} was not found")))
}

fn git_packed_ref_line_matches(request: GitPackedRefMatch<'_>) -> bool {
    if request.line.is_empty() {
        return false;
    }
    if request.line.starts_with(PACKED_REF_COMMENT_PREFIX) {
        return false;
    }
    if request.line.starts_with(PACKED_REF_PEELED_PREFIX) {
        return false;
    }
    request.line.split_whitespace().nth(1) == Some(request.revision)
}

fn validate_git_revision_text(revision: &str) -> Result<(), RunError> {
    if revision.is_empty() {
        return Err(RunError::Internal(format!("unsafe local VCS revision '{revision}'")));
    }
    if revision.contains('\0') {
        return Err(RunError::Internal(format!("unsafe local VCS revision '{revision}'")));
    }
    if revision.contains("..") {
        return Err(RunError::Internal(format!("unsafe local VCS revision '{revision}'")));
    }
    if revision.contains('\\') {
        return Err(RunError::Internal(format!("unsafe local VCS revision '{revision}'")));
    }
    if revision.starts_with('/') {
        return Err(RunError::Internal(format!("unsafe local VCS revision '{revision}'")));
    }
    assert!(!revision.is_empty());
    assert!(!revision.starts_with('/'));
    if revision == GIT_HEAD_REF || revision.starts_with(GIT_REFS_PREFIX) || is_git_object_id(revision) {
        return Ok(());
    }
    Err(RunError::Internal(format!("local VCS revision '{revision}' must be an object id or refs/* name")))
}

fn is_git_object_id(value: &str) -> bool {
    value.len() == GIT_OBJECT_ID_HEX_BYTES && HEXLOWER.decode(value.as_bytes()).is_ok()
}

fn validate_git_object_id(value: &str) -> Result<(), RunError> {
    if is_git_object_id(value) {
        return Ok(());
    }
    Err(RunError::Internal(format!(
        "local VCS object id must be {GIT_OBJECT_ID_HEX_BYTES} lowercase hex bytes"
    )))
}

pub(crate) fn materialize_source_record_from_path(
    record: &SourceRecord,
    payload_path: &Path,
    is_skipping_git_dir: bool,
) -> Result<SourceRecord, RunError> {
    let (files, payload_bytes) = if source_record_is_fetcher_input(record) {
        canonicalize_fetch_payload_entries(payload_path, is_skipping_git_dir)?
    } else {
        canonicalize_payload_entries(payload_path, is_skipping_git_dir)?
    };
    let content_blake3 = digest_source_record_content(&record.kind, &record.metadata, &files)?;
    Ok(SourceRecord {
        payload_bytes,
        content_blake3,
        files,
        ..record.clone()
    })
}

pub(crate) fn materialize_imported_source_record_for_store_path(
    state_dir: &Path,
    store_prefix: &str,
    logical_store_path: impl AsRef<str>,
    target: &Path,
) -> Result<bool, RunError> {
    let lookup = StorePathLookup {
        store_prefix,
        logical_store_path: logical_store_path.as_ref(),
    };
    let available_sources = read_imported_source_records(state_dir)?;
    for record in &available_sources {
        if imported_record_matches_store_path(record, &lookup) {
            materialize_source_record_payload(record, target)?;
            return Ok(true);
        }
    }
    Ok(false)
}

fn imported_record_matches_store_path(record: &SourceRecord, lookup: &StorePathLookup<'_>) -> bool {
    if record.files.is_empty() {
        return false;
    }
    if record.store_prefix.as_deref() != Some(lookup.store_prefix) {
        return false;
    }
    record.metadata.get(RECORD_METADATA_STORE_PATH_KEY).map(String::as_str) == Some(lookup.logical_store_path)
}

pub(crate) fn materialize_source_record_payload(record: &SourceRecord, target: &Path) -> Result<(), RunError> {
    validate_source_record(record)?;
    if record.files.is_empty() {
        return Err(RunError::Internal(format!("source record {} has no materialized payload", record.identity)));
    }
    fs::create_dir_all(target)
        .map_err(|err| RunError::Internal(format!("creating materialized source root {}: {err}", target.display())))?;
    materialize_source_record_files(record, target)?;
    let observed = materialize_source_record_from_path(record, target, false)?;
    if observed.files != record.files || observed.content_blake3 != record.content_blake3 {
        return Err(RunError::Internal(format!(
            "materialized source record {} does not preserve declared files and identity",
            record.identity
        )));
    }
    assert_eq!(observed.payload_bytes, record.payload_bytes);
    assert_eq!(observed.files.len(), record.files.len());
    Ok(())
}

fn materialize_source_record_files(record: &SourceRecord, target: &Path) -> Result<(), RunError> {
    let mut group_start = 0usize;
    while group_start < record.files.len() {
        let path = record.files[group_start].path.as_str();
        let mut group_end = group_start + 1;
        while group_end < record.files.len() && record.files[group_end].path == path {
            group_end += 1;
        }
        let files = &record.files[group_start..group_end];
        if files[0].chunk_index.is_some() {
            materialize_chunked_regular_file_entries(files, target)?;
        } else {
            materialize_source_file_entry(&files[0], target)?;
        }
        group_start = group_end;
    }
    Ok(())
}

fn prepare_source_output_path(file: &SourceFileEntry, target: &Path) -> Result<PathBuf, RunError> {
    let output_path = target.join(&file.path);
    if !output_path.starts_with(target) {
        return Err(RunError::Internal(format!("source file {} escapes materialization root", file.path)));
    }
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| RunError::Internal(format!("creating source payload dir {}: {err}", parent.display())))?;
    }
    Ok(output_path)
}

fn materialize_source_file_entry(file: &SourceFileEntry, target: &Path) -> Result<(), RunError> {
    validate_source_file_entry(file)?;
    if file.chunk_index.is_some() {
        return Err(RunError::Internal(format!("source file chunk {} requires grouped materialization", file.path)));
    }
    let output_path = prepare_source_output_path(file, target)?;
    match file.file_type {
        SourceFileType::Regular => materialize_regular_file_entry(file, &output_path),
        SourceFileType::Symlink => materialize_symlink_file_entry(file, &output_path),
    }
}

fn materialize_chunked_regular_file_entries(files: &[SourceFileEntry], target: &Path) -> Result<(), RunError> {
    assert!(files.len() >= usize::try_from(MIN_SOURCE_FILE_CHUNK_COUNT).expect("chunk count fits in usize"));
    assert!(files.iter().all(|file| file.path == files[0].path));
    let output_path = prepare_source_output_path(&files[0], target)?;
    materialize_chunked_regular_file_entries_at_path(files, &output_path)
}

fn materialize_regular_file_entry(file: &SourceFileEntry, output_path: &Path) -> Result<(), RunError> {
    let content = decode_regular_file_content(file)?;
    fs::write(output_path, content)
        .map_err(|err| RunError::Internal(format!("writing source payload file {}: {err}", output_path.display())))?;
    set_materialized_file_permissions(output_path, file.executable)
}

#[cfg(unix)]
fn set_materialized_file_permissions(output_path: &Path, executable: bool) -> Result<(), RunError> {
    use std::os::unix::fs::PermissionsExt;
    let mode = if executable {
        UNIX_EXECUTABLE_FILE_MODE
    } else {
        UNIX_REGULAR_FILE_MODE
    };
    let permissions = fs::Permissions::from_mode(mode);
    fs::set_permissions(output_path, permissions)
        .map_err(|err| RunError::Internal(format!("setting source payload file mode {}: {err}", output_path.display())))
}

#[cfg(not(unix))]
fn set_materialized_file_permissions(_output_path: &Path, _executable: bool) -> Result<(), RunError> {
    Ok(())
}

#[cfg(unix)]
fn materialize_symlink_file_entry(file: &SourceFileEntry, output_path: &Path) -> Result<(), RunError> {
    use std::os::unix::fs::symlink;
    let target = file
        .symlink_target
        .as_ref()
        .ok_or_else(|| RunError::Internal(format!("symlink source file {} is missing target", file.path)))?;
    symlink(target, output_path)
        .map_err(|err| RunError::Internal(format!("creating source payload symlink {}: {err}", output_path.display())))
}

#[cfg(not(unix))]
fn materialize_symlink_file_entry(file: &SourceFileEntry, _output_path: &Path) -> Result<(), RunError> {
    Err(RunError::Internal(format!(
        "symlink source file {} cannot be materialized on this platform",
        file.path
    )))
}

fn local_file_url_path(raw_url: &str) -> Option<PathBuf> {
    let parsed = Url::parse(raw_url).ok()?;
    if parsed.scheme() != FILE_URL_SCHEME {
        return None;
    }
    parsed.to_file_path().ok()
}

#[cfg(test)]
fn walk_derivation_source_records(
    pending: &mut Vec<DerivationSourceWalkItem<'_>>,
    store_prefix: &str,
    records: &mut Vec<SourceRecord>,
) -> Result<(), RunError> {
    assert!(pending.len() <= MAX_DERIVED_SOURCE_WALK_ITEMS);
    assert!(records.len() <= MAX_SOURCE_RECORDS);
    let mut visited_nodes_len = 0usize;
    for _ in 0..MAX_DERIVED_SOURCE_WALK_ITEMS {
        let Some(item) = pending.pop() else { return Ok(()) };
        match item {
            DerivationSourceWalkItem::Derivation(derivation) => {
                visited_nodes_len = visited_nodes_len
                    .checked_add(1)
                    .ok_or_else(|| RunError::Internal("derived source walk count overflow".to_string()))?;
                if visited_nodes_len > MAX_DERIVED_SOURCE_WALK_NODES {
                    return Err(RunError::Internal(format!(
                        "derived source walk exceeds {MAX_DERIVED_SOURCE_WALK_NODES} derivation nodes"
                    )));
                }
                if let Some(record) = fixed_fetcher_source_record(derivation)? {
                    push_bounded_source_record(records, record)?;
                }
                queue_derivation_inputs(derivation, pending)?;
            }
            DerivationSourceWalkItem::StorePath(source_path) => {
                let record = store_path_source_record(StorePathSourceRequest {
                    source_path,
                    store_prefix,
                })?;
                push_bounded_source_record(records, record)?;
            }
        }
    }
    Err(RunError::Internal(format!(
        "derived source walk exceeds {MAX_DERIVED_SOURCE_WALK_ITEMS} total items"
    )))
}

#[cfg(test)]
fn queue_derivation_inputs<'a>(
    derivation: &'a crunch_glue::CrunchDerivation,
    pending: &mut Vec<DerivationSourceWalkItem<'a>>,
) -> Result<(), RunError> {
    let next_len = pending
        .len()
        .checked_add(derivation.inputs.len())
        .ok_or_else(|| RunError::Internal("derived source walk queue overflow".to_string()))?;
    if next_len > MAX_DERIVED_SOURCE_WALK_ITEMS {
        return Err(RunError::Internal(format!(
            "derived source walk queue exceeds {MAX_DERIVED_SOURCE_WALK_ITEMS} items"
        )));
    }
    pending
        .try_reserve(derivation.inputs.len())
        .map_err(|err| RunError::Internal(format!("reserving derived source walk queue: {err}")))?;
    for input in derivation.inputs.iter().rev() {
        let item = match input {
            crunch_glue::Input::Source(source_path) => DerivationSourceWalkItem::StorePath(source_path),
            crunch_glue::Input::DerivationFile(reference) => {
                return Err(RunError::Internal(format!(
                    "derived source walk requires pipeline resolution for derivation-file input {}",
                    reference.path
                )));
            }
            crunch_glue::Input::ResolvedDerivation(reference) => {
                return Err(RunError::Internal(format!(
                    "derived source walk cannot recover source records from preconverted derivation {}",
                    reference.drv_path
                )));
            }
            crunch_glue::Input::OutputSelection(output) => DerivationSourceWalkItem::Derivation(&output.drv),
            crunch_glue::Input::Derivation(input_derivation) => DerivationSourceWalkItem::Derivation(input_derivation),
        };
        pending.push(item);
    }
    assert_eq!(pending.len(), next_len);
    assert!(pending.len() <= MAX_DERIVED_SOURCE_WALK_ITEMS);
    Ok(())
}

fn push_bounded_source_record(records: &mut Vec<SourceRecord>, record: SourceRecord) -> Result<(), RunError> {
    if records.len() >= MAX_SOURCE_RECORDS {
        return Err(RunError::Internal(format!("source record count exceeds {MAX_SOURCE_RECORDS}")));
    }
    records
        .try_reserve(1)
        .map_err(|err| RunError::Internal(format!("reserving derived source record: {err}")))?;
    records.push(record);
    Ok(())
}

fn fixed_fetcher_source_record(derivation: &crunch_glue::CrunchDerivation) -> Result<Option<SourceRecord>, RunError> {
    if derivation.builder != BUILTIN_FETCHURL_BUILDER {
        return Ok(None);
    }
    assert_eq!(derivation.builder, BUILTIN_FETCHURL_BUILDER);
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
    copy_optional_env_metadata(&derivation.env, &mut metadata, FETCH_ENV_FETCH_POLICY_KEY);
    copy_optional_env_metadata(&derivation.env, &mut metadata, FETCH_ENV_REV_KEY);
    copy_optional_env_metadata(&derivation.env, &mut metadata, FETCH_ENV_TYPE_KEY);
    copy_optional_env_metadata(&derivation.env, &mut metadata, FETCH_ENV_UNPACK_KEY);
    if metadata.get(FETCH_ENV_UNPACK_KEY).map(String::as_str) == Some("1") {
        metadata.insert(RECORD_METADATA_PAYLOAD_ENCODING_KEY.to_string(), TARBALL_ARCHIVE_PAYLOAD_ENCODING.to_string());
    }
    assert_eq!(metadata.get(RECORD_METADATA_URL_KEY), Some(url));
    let kind = if derivation.env.get(FETCH_ENV_TYPE_KEY).map(String::as_str) == Some(FETCH_ENV_TYPE_GIT) {
        SourceRecordKind::VcsSnapshot
    } else if metadata.get(RECORD_METADATA_URL_KEY).map(String::as_str)
        == Some(crate::bootstrap_source_root::LEGACY_MUSL_CC_URL)
        && metadata.get(RECORD_METADATA_HASH_KEY).map(String::as_str)
            == Some(crate::bootstrap_source_root::LEGACY_MUSL_CC_HASH)
    {
        SourceRecordKind::BootstrapArchive
    } else {
        SourceRecordKind::FixedUrl
    };
    if kind == SourceRecordKind::VcsSnapshot && !metadata.contains_key(FETCH_ENV_REV_KEY) {
        return Err(RunError::Internal(format!("git fetcher {} is missing env.rev", derivation.name)));
    }
    let id_prefix = if kind == SourceRecordKind::VcsSnapshot {
        DERIVED_VCS_ID_PREFIX
    } else {
        DERIVED_FIXED_URL_ID_PREFIX
    };
    Ok(Some(virtual_source_record(kind, id_prefix, None, metadata)?))
}

fn store_path_source_record(request: StorePathSourceRequest<'_>) -> Result<SourceRecord, RunError> {
    if !request.source_path.starts_with('/') {
        return Err(RunError::Internal(format!(
            "source input path must be absolute for source bundle planning: {}",
            request.source_path
        )));
    }
    let mut metadata = BTreeMap::new();
    metadata.insert(RECORD_METADATA_SOURCE_KIND_KEY.to_string(), "pre-existing-store-path".to_string());
    metadata.insert(RECORD_METADATA_STORE_PATH_KEY.to_string(), request.source_path.to_string());
    virtual_source_record(
        SourceRecordKind::ToolchainSourceRoot,
        DERIVED_STORE_PATH_ID_PREFIX,
        Some(request.store_prefix.to_string()),
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
    is_json_output: bool,
) -> Result<(), RunError> {
    let context = SourceBundleCliContext {
        state_dir,
        store_prefix,
        is_json_output,
    };
    match action {
        crate::SourceAction::Bundle { action } => cmd_source_bundle(action, &context),
    }
}

struct BootstrapProfileCliInput {
    mode: String,
    provider_archive: PathBuf,
    provider_manifest: PathBuf,
    bootstrap_sources: Vec<PathBuf>,
    mantle_source: Option<PathBuf>,
    vendor_deps: Option<PathBuf>,
    toolchain_source_root: Option<PathBuf>,
    proof_inputs: Vec<PathBuf>,
    include_bundles: Vec<PathBuf>,
    to: Option<PathBuf>,
    preflight: bool,
}

fn cmd_bootstrap_profile(
    input: BootstrapProfileCliInput,
    context: &SourceBundleCliContext<'_>,
) -> Result<(), RunError> {
    let mode = BootstrapSourceBundleMode::parse(&input.mode)?;
    let supplemental_records = read_supplemental_bundle_records(&input.include_bundles)?;
    let profile_input = BootstrapSourceBundleProfileInput {
        mode,
        provider_archive: input.provider_archive,
        provider_manifest: input.provider_manifest,
        bootstrap_sources: input.bootstrap_sources,
        mantle_source: input.mantle_source,
        vendor_deps: input.vendor_deps,
        toolchain_source_root: input.toolchain_source_root,
        proof_inputs: input.proof_inputs,
        supplemental_records,
    };
    let manifest = plan_bootstrap_source_bundle_profile(&profile_input, context.store_prefix)?;
    assert_eq!(manifest.format, SOURCE_BUNDLE_FORMAT);
    assert_eq!(manifest.non_claim, SOURCE_BUNDLE_NON_CLAIM);
    if let Some(path) = input.to {
        write_source_bundle(&path, &manifest)?;
    }
    if input.preflight {
        let preflight_receipt = offline_preflight_for_manifest(&manifest, context.state_dir)?;
        print_offline_preflight_report(&preflight_receipt, context.is_json_output)?;
        if source_offline_preflight_is_ready(&preflight_receipt) {
            return Ok(());
        }
        return Err(RunError::Reported(1));
    }
    let profile_receipt = bootstrap_source_bundle_profile_report(&manifest, mode)?;
    print_bootstrap_profile_report(&profile_receipt, context.is_json_output)
}

fn read_supplemental_bundle_records(paths: &[PathBuf]) -> Result<Vec<SourceRecord>, RunError> {
    let mut records = Vec::new();
    for path in paths {
        let manifest = read_source_bundle(path)?;
        let next_len = records
            .len()
            .checked_add(manifest.records.len())
            .ok_or_else(|| RunError::Internal("supplemental source record count overflow".to_string()))?;
        if next_len > MAX_SOURCE_RECORDS {
            return Err(RunError::Internal(format!("supplemental source record count exceeds {MAX_SOURCE_RECORDS}")));
        }
        records.extend(manifest.records);
    }
    assert!(records.len() <= MAX_SOURCE_RECORDS);
    Ok(records)
}

fn cmd_source_bundle(action: crate::SourceBundleAction, context: &SourceBundleCliContext<'_>) -> Result<(), RunError> {
    match action {
        crate::SourceBundleAction::Plan {
            sources,
            build_roots,
            import_paths,
        } => cmd_plan_source_bundle(&sources, &build_roots, &import_paths, context),
        crate::SourceBundleAction::Export {
            sources,
            build_roots,
            import_paths,
            to,
            fetch_missing,
        } => cmd_export_source_bundle(&sources, &build_roots, &import_paths, &to, fetch_missing, context),
        crate::SourceBundleAction::BootstrapProfile {
            mode,
            provider_archive,
            provider_manifest,
            bootstrap_sources,
            mantle_source,
            vendor_deps,
            toolchain_source_root,
            proof_inputs,
            include_bundles,
            to,
            preflight,
        } => cmd_bootstrap_profile(
            BootstrapProfileCliInput {
                mode,
                provider_archive,
                provider_manifest,
                bootstrap_sources,
                mantle_source,
                vendor_deps,
                toolchain_source_root,
                proof_inputs,
                include_bundles,
                to,
                preflight,
            },
            context,
        ),
        crate::SourceBundleAction::List { from } => cmd_list_source_bundle(&from, context),
        crate::SourceBundleAction::Import { from, pin } => cmd_import_source_bundle(&from, pin, context),
        crate::SourceBundleAction::HydrateSelfBuild {
            from,
            expected_manifest_blake3,
            checkout,
        } => cmd_hydrate_self_build_source_bundle(&from, &expected_manifest_blake3, &checkout, context),
        crate::SourceBundleAction::Verify { from, imported } => cmd_verify_source_bundle(&from, imported, context),
        crate::SourceBundleAction::Preflight {
            build_roots,
            import_paths,
        } => cmd_preflight_source_bundle(&build_roots, &import_paths, context),
    }
}

fn cmd_plan_source_bundle(
    sources: &[String],
    build_roots: &[PathBuf],
    import_paths: &[PathBuf],
    context: &SourceBundleCliContext<'_>,
) -> Result<(), RunError> {
    let manifest = plan_from_cli_inputs(sources, build_roots, import_paths, context.store_prefix)?;
    print_plan_report(&plan_report(&manifest)?, context.is_json_output)
}

fn cmd_export_source_bundle(
    sources: &[String],
    build_roots: &[PathBuf],
    import_paths: &[PathBuf],
    to: &Path,
    fetch_missing: bool,
    context: &SourceBundleCliContext<'_>,
) -> Result<(), RunError> {
    let manifest = export_from_cli_inputs(
        sources,
        build_roots,
        import_paths,
        context.store_prefix,
        context.state_dir,
        fetch_missing,
    )?;
    write_source_bundle(to, &manifest)?;
    print_plan_report(&plan_report(&manifest)?, context.is_json_output)
}

fn cmd_list_source_bundle(from: &Path, context: &SourceBundleCliContext<'_>) -> Result<(), RunError> {
    let manifest = read_source_bundle(from)?;
    print_plan_report(&list_source_bundle(&manifest)?, context.is_json_output)
}

fn cmd_import_source_bundle(from: &Path, pin: bool, context: &SourceBundleCliContext<'_>) -> Result<(), RunError> {
    let manifest = read_source_bundle(from)?;
    let operation_output = import_source_bundle(&manifest, context.state_dir, pin)?;
    print_import_report(&operation_output, context.is_json_output)
}

fn cmd_hydrate_self_build_source_bundle(
    from: &Path,
    expected_manifest_blake3: &str,
    checkout: &Path,
    context: &SourceBundleCliContext<'_>,
) -> Result<(), RunError> {
    let manifest = read_source_bundle(from)?;
    let report = hydrate_self_build_source_bundle(&manifest, expected_manifest_blake3, checkout, context.state_dir)?;
    print_self_build_hydration_report(&report, context.is_json_output)
}

fn cmd_verify_source_bundle(from: &Path, imported: bool, context: &SourceBundleCliContext<'_>) -> Result<(), RunError> {
    let manifest = read_source_bundle(from)?;
    let verify_receipt = if imported {
        verify_source_bundle_state(&manifest, context.state_dir)?
    } else {
        SourceBundleVerifyReport {
            manifest_blake3: manifest.manifest_blake3.clone(),
            ready_class: SourceReadiness::Ready,
            missing_records: Vec::new(),
            stale_records: Vec::new(),
            unsupported_records: Vec::new(),
            untrusted_records: Vec::new(),
            non_claim: SOURCE_BUNDLE_NON_CLAIM,
        }
    };
    print_verify_report(&verify_receipt, context.is_json_output)
}

fn cmd_preflight_source_bundle(
    build_roots: &[PathBuf],
    import_paths: &[PathBuf],
    context: &SourceBundleCliContext<'_>,
) -> Result<(), RunError> {
    let preflight_receipt =
        offline_preflight_for_build_roots(build_roots, import_paths, context.state_dir, context.store_prefix)?;
    print_offline_preflight_report(&preflight_receipt, context.is_json_output)?;
    if source_offline_preflight_is_ready(&preflight_receipt) {
        return Ok(());
    }
    Err(RunError::Reported(1))
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
    let evaluation_paths = crate::build_cmd::build_import_paths(import_paths)?;
    let mut records = canonicalize_source_specs(&specs, store_prefix)?;
    records.extend(collect_build_source_records_from_files(build_roots, &evaluation_paths, store_prefix)?);
    assemble_source_bundle(records, store_prefix)
}

fn export_from_cli_inputs(
    sources: &[String],
    build_roots: &[PathBuf],
    import_paths: &[PathBuf],
    store_prefix: &str,
    state_dir: &Path,
    fetch_missing: bool,
) -> Result<SourceBundleManifest, RunError> {
    let specs = sources.iter().map(|source| parse_source_spec(source)).collect::<Result<Vec<_>, _>>()?;
    if build_roots.is_empty() {
        return plan_source_bundle(&specs, store_prefix);
    }
    let evaluation_paths = crate::build_cmd::build_import_paths(import_paths)?;
    let derived_records = collect_build_source_records_from_files(build_roots, &evaluation_paths, store_prefix)?;
    let available_sources = read_imported_source_records(state_dir)?;
    let mut records = canonicalize_source_specs(&specs, store_prefix)?;
    let expected = normalize_source_records(derived_records)?;
    if fetch_missing {
        records.extend(materialize_export_records_with_connected_fetch(&expected, &available_sources)?);
    } else {
        records.extend(materialize_export_records(&expected, &available_sources)?);
    }
    assemble_source_bundle(records, store_prefix)
}

fn collect_build_source_records_from_files(
    build_roots: &[PathBuf],
    resolved_import_paths: &[OsString],
    store_prefix: &str,
) -> Result<Vec<SourceRecord>, RunError> {
    if build_roots.is_empty() || build_roots.len() > MAX_DERIVED_SOURCE_WALK_NODES {
        return Err(RunError::Internal(format!(
            "source bundle build-root count must be within 1..={MAX_DERIVED_SOURCE_WALK_NODES}"
        )));
    }
    let mut walker = DerivationFileSourceWalker::new(resolved_import_paths, store_prefix);
    for build_root in build_roots {
        walker.walk_root_file(build_root)?;
    }
    assert!(walker.visiting.is_empty());
    assert!(walker.records.len() <= MAX_SOURCE_RECORDS);
    Ok(walker.records)
}

impl<'a> DerivationFileSourceWalker<'a> {
    fn new(import_paths: &'a [OsString], store_prefix: &'a str) -> Self {
        assert!(store_prefix.starts_with('/'));
        const { assert!(DERIVATION_FILE_COUNT_MAX > 1) };
        Self {
            import_paths,
            store_prefix,
            records: Vec::new(),
            visiting: BTreeSet::new(),
            completed_outputs: BTreeMap::new(),
            visited_derivation_count: 0,
            resolved_file_count: 0,
        }
    }

    fn walk_root_file(&mut self, root_file: &Path) -> Result<(), RunError> {
        let canonical_root = root_file.canonicalize().map_err(|error| {
            RunError::Internal(format!("canonicalizing source-bundle root {}: {error}", root_file.display()))
        })?;
        let root_dir = canonical_root
            .parent()
            .ok_or_else(|| {
                RunError::Internal(format!("source-bundle root has no parent: {}", canonical_root.display()))
            })?
            .to_path_buf();
        let roots = evaluate_build_root_with_import_paths(&canonical_root, self.import_paths)?;
        for (_, derivation) in roots {
            self.walk_derivation(&root_dir, &canonical_root, &derivation, 0)?;
        }
        assert!(canonical_root.starts_with(&root_dir));
        debug_assert!(self.visiting.is_empty());
        Ok(())
    }

    fn walk_derivation(
        &mut self,
        root_dir: &Path,
        owner_file: &Path,
        derivation: &crunch_glue::CrunchDerivation,
        depth: u32,
    ) -> Result<(), RunError> {
        self.require_depth_and_node_capacity(depth)?;
        if let Some(record) = fixed_fetcher_source_record(derivation)? {
            push_bounded_source_record(&mut self.records, record)?;
        }
        for input in &derivation.inputs {
            self.walk_input(root_dir, owner_file, input, depth.saturating_add(1))?;
        }
        assert!(self.visited_derivation_count <= MAX_DERIVED_SOURCE_WALK_NODES);
        debug_assert!(self.records.len() <= MAX_SOURCE_RECORDS);
        Ok(())
    }

    fn walk_input(
        &mut self,
        root_dir: &Path,
        owner_file: &Path,
        input: &crunch_glue::Input,
        depth: u32,
    ) -> Result<(), RunError> {
        match input {
            crunch_glue::Input::Source(source_path) => {
                let record = store_path_source_record(StorePathSourceRequest {
                    source_path,
                    store_prefix: self.store_prefix,
                })?;
                push_bounded_source_record(&mut self.records, record)
            }
            crunch_glue::Input::DerivationFile(reference) => {
                self.walk_derivation_file(root_dir, owner_file, reference, depth)
            }
            crunch_glue::Input::ResolvedDerivation(reference) => Err(RunError::Internal(format!(
                "source-bundle walk cannot recover sources from resolved derivation {}",
                reference.drv_path
            ))),
            crunch_glue::Input::OutputSelection(output) => {
                self.walk_derivation(root_dir, owner_file, &output.drv, depth)
            }
            crunch_glue::Input::Derivation(derivation) => self.walk_derivation(root_dir, owner_file, derivation, depth),
        }
    }

    fn walk_derivation_file(
        &mut self,
        root_dir: &Path,
        owner_file: &Path,
        reference: &crunch_glue::DerivationFileRef,
        depth: u32,
    ) -> Result<(), RunError> {
        self.require_file_capacity(depth)?;
        let path = resolve_source_derivation_file(root_dir, owner_file, &reference.path)?;
        if let Some(outputs) = self.completed_outputs.get(&path) {
            return validate_source_derivation_output(&path, reference.output.as_deref(), outputs);
        }
        if !self.visiting.insert(path.clone()) {
            return Err(RunError::Internal(format!(
                "source-bundle derivation-file cycle detected at {}",
                path.display()
            )));
        }
        self.resolved_file_count = self.resolved_file_count.saturating_add(1);
        let roots = evaluate_build_root_with_import_paths(&path, self.import_paths)?;
        if roots.len() != SINGLE_DERIVATION_FILE_ROOT_COUNT {
            return Err(RunError::Internal(format!(
                "source-bundle derivation-file {} must evaluate to exactly one root, observed {}",
                path.display(),
                roots.len()
            )));
        }
        let (_, derivation) = roots.into_iter().next().ok_or_else(|| {
            RunError::Internal(format!("source-bundle derivation-file {} returned no root", path.display()))
        })?;
        let outputs = derivation.outputs.iter().cloned().collect::<BTreeSet<_>>();
        validate_source_derivation_output(&path, reference.output.as_deref(), &outputs)?;
        let result = self.walk_derivation(root_dir, &path, &derivation, depth.saturating_add(1));
        let removed = self.visiting.remove(&path);
        assert!(removed, "visited source-bundle derivation file must be removed");
        result?;
        self.completed_outputs.insert(path, outputs);
        Ok(())
    }

    fn require_depth_and_node_capacity(&mut self, depth: u32) -> Result<(), RunError> {
        if depth > DERIVATION_FILE_DEPTH_MAX {
            return Err(RunError::Internal(format!(
                "source-bundle derivation depth exceeds {DERIVATION_FILE_DEPTH_MAX}"
            )));
        }
        self.visited_derivation_count = self
            .visited_derivation_count
            .checked_add(1)
            .ok_or_else(|| RunError::Internal("source-bundle derivation count overflow".to_string()))?;
        if self.visited_derivation_count > MAX_DERIVED_SOURCE_WALK_NODES {
            return Err(RunError::Internal(format!(
                "source-bundle derivation walk exceeds {MAX_DERIVED_SOURCE_WALK_NODES} nodes"
            )));
        }
        Ok(())
    }

    fn require_file_capacity(&self, depth: u32) -> Result<(), RunError> {
        if depth > DERIVATION_FILE_DEPTH_MAX || self.resolved_file_count >= DERIVATION_FILE_COUNT_MAX {
            return Err(RunError::Internal(format!(
                "source-bundle derivation-file bounds exceeded: depth={depth}, files={}",
                self.resolved_file_count
            )));
        }
        assert!(self.resolved_file_count < DERIVATION_FILE_COUNT_MAX);
        debug_assert!(depth <= DERIVATION_FILE_DEPTH_MAX);
        Ok(())
    }
}

fn validate_source_derivation_output(
    path: &Path,
    requested_output: Option<&str>,
    outputs: &BTreeSet<String>,
) -> Result<(), RunError> {
    if outputs.is_empty() {
        return Err(RunError::Internal(format!("source-bundle derivation-file {} has no outputs", path.display())));
    }
    if let Some(output) = requested_output
        && !outputs.contains(output)
    {
        return Err(RunError::Internal(format!(
            "source-bundle derivation-file {} does not declare requested output {output}",
            path.display()
        )));
    }
    assert!(!outputs.is_empty());
    debug_assert!(requested_output.is_none_or(|output| outputs.contains(output)));
    Ok(())
}

fn resolve_source_derivation_file(root_dir: &Path, owner_file: &Path, reference: &str) -> Result<PathBuf, RunError> {
    let reference_path = Path::new(reference);
    let normalized = reference_path.components().all(|component| matches!(component, Component::Normal(_)));
    let expected_extension =
        reference_path.extension().and_then(|extension| extension.to_str()) == Some(DERIVATION_FILE_EXTENSION);
    if reference.is_empty()
        || reference.len() > DERIVATION_FILE_PATH_BYTES_MAX
        || reference_path.is_absolute()
        || !normalized
        || !expected_extension
    {
        return Err(RunError::Internal(format!(
            "source-bundle derivation-file input must be a bounded normalized relative .ncl path: {reference}"
        )));
    }
    let owner_dir = owner_file.parent().ok_or_else(|| {
        RunError::Internal(format!("source-bundle derivation-file owner has no parent: {}", owner_file.display()))
    })?;
    let candidate = owner_dir.join(reference_path);
    let canonical = candidate.canonicalize().map_err(|error| {
        RunError::Internal(format!("resolving source-bundle derivation-file {}: {error}", candidate.display()))
    })?;
    if !canonical.starts_with(root_dir) || !canonical.is_file() {
        return Err(RunError::Internal(format!(
            "source-bundle derivation-file escapes its root or is not a file: {}",
            canonical.display()
        )));
    }
    assert!(canonical.starts_with(root_dir));
    debug_assert_eq!(canonical.extension().and_then(|extension| extension.to_str()), Some(DERIVATION_FILE_EXTENSION));
    Ok(canonical)
}

fn evaluate_build_root_with_import_paths(
    build_root: &Path,
    resolved_import_paths: &[OsString],
) -> Result<Vec<(String, crunch_glue::CrunchDerivation)>, RunError> {
    let mut session = crunch_eval::session::EvaluationSession::open_file(build_root, resolved_import_paths)
        .map_err(|err| RunError::Eval(format!("opening build root {}: {err}", build_root.display())))?;
    session
        .force_all_roots::<crunch_glue::CrunchDerivation>()
        .map_err(|err| RunError::Eval(format!("evaluating source bundle build root {}: {err}", build_root.display())))
}

fn print_bootstrap_profile_report(
    report: &BootstrapSourceBundleProfileReport,
    json_output: bool,
) -> Result<(), RunError> {
    if json_output {
        println!("{}", render_json(report)?);
        return Ok(());
    }
    println!(
        "format={} mode={} records={} manifest_blake3={} provider_kind={}",
        report.format,
        report.mode.as_str(),
        report.required_record_count,
        report.manifest_blake3,
        report.provider_kind
    );
    eprintln!("non_claim={}", report.non_claim);
    Ok(())
}

fn print_self_build_hydration_report(report: &SelfBuildHydrationReport, json_output: bool) -> Result<(), RunError> {
    if json_output {
        println!("{}", render_json(report)?);
        return Ok(());
    }
    println!(
        "format={} manifest_blake3={} vendor_blake3={} provider_archive_blake3={} imported={} existing={} pinned={}",
        report.format,
        report.manifest_blake3,
        report.vendor_content_blake3,
        report.provider_archive_content_blake3,
        report.imported_record_count,
        report.existing_record_count,
        report.pinned,
    );
    eprintln!("non_claim={}", report.non_claim);
    Ok(())
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
        "manifest_blake3={} readiness={:?} missing={} stale={} unsupported={} untrusted={}",
        report.manifest_blake3,
        report.ready_class,
        report.missing_records.len(),
        report.stale_records.len(),
        report.unsupported_records.len(),
        report.untrusted_records.len()
    );
    eprintln!("non_claim={}", report.non_claim);
    Ok(())
}

pub fn print_offline_preflight_report(
    report: &SourceOfflinePreflightReport,
    is_json_output: bool,
) -> Result<(), RunError> {
    assert_eq!(report.format, SOURCE_OFFLINE_PREFLIGHT_FORMAT);
    assert_eq!(report.non_claim, SOURCE_BUNDLE_NON_CLAIM);
    if is_json_output {
        println!("{}", render_json(report)?);
        return Ok(());
    }
    println!(
        "format={} readiness={:?} records={} source_state_blake3={}",
        report.format, report.ready_class, report.record_count, report.source_state_blake3
    );
    if let Some(manifest_blake3) = &report.manifest_blake3 {
        println!("manifest_blake3={manifest_blake3}");
    }
    println!(
        "missing={} stale={} unsupported={} untrusted={} network_required={} unpinned={}",
        report.missing_records.len(),
        report.stale_records.len(),
        report.unsupported_records.len(),
        report.untrusted_records.len(),
        report.network_required_records.len(),
        report.unpinned_records.len()
    );
    for action in &report.next_actions {
        eprintln!(
            "next_action blocker_class={} command_hint={} description={}",
            action.blocker_class, action.command_hint, action.description
        );
    }
    for record in &report.records {
        println!(
            "SOURCE_PREFLIGHT_RECORD kind={:?} identity={} files={} payload_bytes={} blake3={}",
            record.kind, record.identity, record.file_count, record.payload_bytes, record.content_blake3
        );
    }
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

    fn write_test_tar_archive(source: &Path, archive: &Path) {
        let file = fs::File::create(archive).unwrap();
        let mut builder = tar::Builder::new(file);
        builder.append_dir_all("source", source).unwrap();
        builder.finish().unwrap();
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

    fn fixed_fetcher_with_flat_blake3(name: &str, url: &str, content: &[u8]) -> crunch_glue::CrunchDerivation {
        let mut derivation = fixed_fetcher(name, url);
        let hash = NixHash::Blake3(*blake3::hash(content).as_bytes()).to_sri_string();
        derivation.fixed_output = Some(crunch_glue::FixedOutput {
            hash,
            algo: "blake3".to_string(),
            mode: "flat".to_string(),
        });
        derivation
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

    fn materialized_record_from_payload(
        record: &SourceRecord,
        payload_path: &Path,
        skip_git_dir: bool,
    ) -> SourceRecord {
        let (files, payload_bytes) = if source_record_is_fetcher_input(record) {
            canonicalize_fetch_payload_entries(payload_path, skip_git_dir).unwrap()
        } else {
            canonicalize_payload_entries(payload_path, skip_git_dir).unwrap()
        };
        let content_blake3 = digest_source_record_content(&record.kind, &record.metadata, &files).unwrap();
        SourceRecord {
            payload_bytes,
            content_blake3,
            files,
            ..record.clone()
        }
    }

    fn package_adapter(
        adapter: &str,
        lock_identity: &str,
        offline_control: &str,
        boundary: &str,
    ) -> SourceAdapterMetadata {
        let mut extra = BTreeMap::new();
        extra.insert("mirror-kind".to_string(), format!("{adapter}-mirror"));
        SourceAdapterMetadata {
            adapter: adapter.to_string(),
            lock_identity: lock_identity.to_string(),
            offline_control: offline_control.to_string(),
            generated_source_boundary: boundary.to_string(),
            extra,
        }
    }

    fn non_cargo_adapter() -> SourceAdapterMetadata {
        package_adapter("npm", "package-lock:demo", "npm-cache-offline", "no-generated-node-modules")
    }

    fn cargo_adapter() -> SourceAdapterMetadata {
        package_adapter("cargo", "Cargo.lock:demo", "cargo-net-offline", "vendor-deps-only")
    }

    fn adapter_with_extra(key: &str, value: &str) -> SourceAdapterMetadata {
        let mut adapter = cargo_adapter();
        adapter.extra.insert(key.to_string(), value.to_string());
        adapter
    }

    fn write_provider_manifest(path: &Path, provider_kind: &str) {
        let json = serde_json::json!({
            "schema_version": 1,
            "provider_kind": provider_kind,
            "reduction": {
                "retained_tools": ["cc", "ar"],
                "dropped_components": ["locale-catalogs"]
            },
            "normalized_seed_contract": {
                "target": "x86_64-linux-musl",
                "dynamic_linker": "ld-musl-x86_64.so.1"
            }
        });
        fs::write(path, serde_json::to_string_pretty(&json).unwrap()).unwrap();
    }

    fn bootstrap_profile_fixture(temp: &Path) -> BootstrapSourceBundleProfileInput {
        let provider_archive = temp.join("provider-archive");
        let provider_manifest = temp.join("provider.json");
        let bootstrap_source = temp.join("bootstrap-src");
        let mantle_source = temp.join("mantle-src");
        let vendor_deps = temp.join("vendor-deps");
        let toolchain = temp.join("toolchain");
        let proof = temp.join("proof");
        write_fixture(&provider_archive);
        write_provider_manifest(&provider_manifest, BOOTSTRAP_PROVIDER_KIND_LEGACY_SEED);
        write_fixture(&bootstrap_source);
        write_fixture(&mantle_source);
        write_fixture(&vendor_deps);
        write_fixture(&toolchain);
        write_fixture(&proof);
        BootstrapSourceBundleProfileInput {
            mode: BootstrapSourceBundleMode::SelfBuildProof,
            provider_archive,
            provider_manifest,
            bootstrap_sources: vec![bootstrap_source],
            mantle_source: Some(mantle_source),
            vendor_deps: Some(vendor_deps),
            toolchain_source_root: Some(toolchain),
            proof_inputs: vec![proof],
            supplemental_records: Vec::new(),
        }
    }

    fn cargo_sha256_hex(bytes: &[u8]) -> String {
        use sha2::Digest as _;
        data_encoding::HEXLOWER.encode(&sha2::Sha256::digest(bytes))
    }

    fn write_hydration_checkout(checkout: &Path) {
        const PACKAGE_CHECKSUM: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        fs::create_dir_all(checkout.join(".cargo")).unwrap();
        fs::write(
            checkout.join(".cargo/vendor-config.toml"),
            "[source.crates-io]\nreplace-with = \"vendored-sources\"\n\n[source.vendored-sources]\ndirectory = \"vendor-deps\"\n",
        )
        .unwrap();
        fs::write(
            checkout.join("Cargo.lock"),
            format!(
                "version = 4\n\n[[package]]\nname = \"dep-a\"\nversion = \"0.1.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"{PACKAGE_CHECKSUM}\"\n"
            ),
        )
        .unwrap();
    }

    fn write_hydration_vendor(vendor: &Path, is_tampered: bool) {
        const PACKAGE_CHECKSUM: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let package = vendor.join("dep-a");
        let manifest = b"[package]\nname = \"dep-a\"\nversion = \"0.1.0\"\n";
        let valid_lib = b"pub fn dep_a() {}\n";
        let materialized_lib = if is_tampered {
            b"pub fn tampered() {}\n".as_slice()
        } else {
            valid_lib.as_slice()
        };
        fs::create_dir_all(package.join("src")).unwrap();
        fs::write(package.join("Cargo.toml"), manifest).unwrap();
        fs::write(package.join("src/lib.rs"), materialized_lib).unwrap();
        let checksum = serde_json::json!({
            "files": {
                "Cargo.toml": cargo_sha256_hex(manifest),
                "src/lib.rs": cargo_sha256_hex(valid_lib),
            },
            "package": PACKAGE_CHECKSUM,
        });
        fs::write(package.join(".cargo-checksum.json"), serde_json::to_vec(&checksum).unwrap()).unwrap();
    }

    fn hydration_fixture(temp: &Path, is_vendor_tampered: bool) -> (SourceBundleManifest, PathBuf) {
        let input = bootstrap_profile_fixture(&temp.join("profile"));
        let vendor = input.vendor_deps.as_ref().unwrap();
        fs::remove_dir_all(vendor).unwrap();
        write_hydration_vendor(vendor, is_vendor_tampered);
        let manifest = plan_bootstrap_source_bundle_profile(&input, "/mantle/store").unwrap();
        let checkout = temp.join("fresh-clone");
        write_hydration_checkout(&checkout);
        (manifest, checkout)
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
    fn source_relative_path_accepts_double_dot_inside_safe_cargo_fixture_name() {
        let path = "json_scanner/tests/inputs/n_number_-1.0..json";

        validate_source_relative_path_text(path).unwrap();

        assert!(path.contains(".."));
        assert!(!path.split('/').any(|component| component == ".."));
    }

    #[test]
    fn source_relative_path_rejects_parent_and_empty_components() {
        let parent = validate_source_relative_path_text("package/../outside").unwrap_err();
        let empty = validate_source_relative_path_text("package//file").unwrap_err();

        assert!(parent.to_string().contains("unsafe source relative path"));
        assert!(empty.to_string().contains("unsafe source relative path"));
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

    #[cfg(target_os = "linux")]
    #[test]
    fn source_bundle_allows_case_sensitive_kernel_headers_only_for_bootstrap_archives() {
        let temp = tempfile::tempdir().unwrap();
        let headers = temp.path().join("headers");
        fs::create_dir_all(&headers).unwrap();
        fs::write(headers.join("xt_MARK.h"), b"upper\n").unwrap();
        fs::write(headers.join("xt_mark.h"), b"lower\n").unwrap();
        let bootstrap_spec = SourceSpec {
            kind: SourceRecordKind::BootstrapArchive,
            identity: "legacy-provider-headers".to_string(),
            path: headers.clone(),
            adapter: None,
        };
        let portable_spec = SourceSpec {
            kind: SourceRecordKind::LocalPath,
            identity: "portable-input".to_string(),
            path: headers,
            adapter: None,
        };

        let manifest = plan_source_bundle(&[bootstrap_spec], "/mantle/store").unwrap();
        let error = plan_source_bundle(&[portable_spec], "/mantle/store").unwrap_err();

        assert_eq!(manifest.records[0].files.len(), 2);
        assert!(error.to_string().contains("path case collision"));
    }

    #[test]
    fn exact_legacy_provider_fetch_is_a_case_sensitive_override_input() {
        let mut derivation = fixed_fetcher("legacy-provider", crate::bootstrap_source_root::LEGACY_MUSL_CC_URL);
        derivation.env.insert(FETCH_ENV_UNPACK_KEY.to_string(), "1".to_string());
        derivation.fixed_output = Some(crunch_glue::FixedOutput {
            hash: crate::bootstrap_source_root::LEGACY_MUSL_CC_HASH.to_string(),
            algo: "sha256".to_string(),
            mode: "recursive".to_string(),
        });

        let record = fixed_fetcher_source_record(&derivation).unwrap().unwrap();
        let mut wrong_hash_record = record.clone();
        wrong_hash_record.metadata.insert(RECORD_METADATA_HASH_KEY.to_string(), "sha256-wrong".to_string());

        assert_eq!(record.kind, SourceRecordKind::BootstrapArchive);
        assert!(source_record_is_fetcher_input(&record));
        assert_eq!(source_fetch_override_kind(&record).unwrap(), crunch_build::FetchSourceOverrideKind::Tarball);
        assert!(!source_record_is_legacy_provider_fetch(&wrong_hash_record));
        assert!(source_fetch_override_kind(&wrong_hash_record).is_err());
    }

    #[test]
    fn source_bundle_accepts_bounded_legacy_provider_compiler_payload() {
        const LEGACY_PROVIDER_COMPILER_BYTES: u64 = 16_777_217;
        let temp = tempfile::tempdir().unwrap();
        let compiler = temp.path().join("cc1plus");
        fs::File::create(&compiler).unwrap().set_len(LEGACY_PROVIDER_COMPILER_BYTES).unwrap();
        let spec = SourceSpec {
            kind: SourceRecordKind::BootstrapArchive,
            identity: "legacy-provider-compiler".to_string(),
            path: compiler,
            adapter: None,
        };

        let manifest = plan_source_bundle(&[spec], "/mantle/store").unwrap();

        assert_eq!(manifest.records[0].payload_bytes, LEGACY_PROVIDER_COMPILER_BYTES);
        assert!(manifest.records[0].payload_bytes < MAX_SOURCE_FILE_BYTES);
    }

    #[test]
    fn source_bundle_rejects_payload_above_named_file_limit() {
        let temp = tempfile::tempdir().unwrap();
        let oversized = temp.path().join("oversized");
        fs::File::create(&oversized)
            .unwrap()
            .set_len(MAX_SOURCE_FILE_BYTES.checked_add(1).unwrap())
            .unwrap();
        let spec = SourceSpec {
            kind: SourceRecordKind::BootstrapArchive,
            identity: "oversized-provider-file".to_string(),
            path: oversized,
            adapter: None,
        };

        let error = plan_source_bundle(&[spec], "/mantle/store").unwrap_err();

        assert!(error.to_string().contains("limit"));
        assert!(error.to_string().contains(&MAX_SOURCE_FILE_BYTES.to_string()));
    }

    #[test]
    fn source_file_chunk_layout_is_bounded_exact_and_has_no_empty_tail() {
        const TEST_CHUNK_SIZE_BYTES_MAX: u64 = 8;
        const TEST_CHUNK_COUNT: u64 = 2;
        let exact_size_bytes = TEST_CHUNK_SIZE_BYTES_MAX.checked_mul(TEST_CHUNK_COUNT).unwrap();
        let partial_size_bytes = exact_size_bytes.checked_add(1).unwrap();

        let partial_tail = source_file_chunk_sizes(partial_size_bytes, TEST_CHUNK_SIZE_BYTES_MAX).unwrap();
        let exact_tail = source_file_chunk_sizes(exact_size_bytes, TEST_CHUNK_SIZE_BYTES_MAX).unwrap();

        assert_eq!(partial_tail, vec![TEST_CHUNK_SIZE_BYTES_MAX, TEST_CHUNK_SIZE_BYTES_MAX, 1]);
        assert_eq!(exact_tail, vec![TEST_CHUNK_SIZE_BYTES_MAX, TEST_CHUNK_SIZE_BYTES_MAX]);
        assert!(partial_tail.iter().all(|size| *size <= TEST_CHUNK_SIZE_BYTES_MAX));
        assert!(exact_tail.iter().all(|size| *size > 0));
    }

    #[test]
    fn source_file_chunk_layout_rejects_zero_and_excessive_counts() {
        let error_zero_size = source_file_chunk_sizes(0, MAX_SOURCE_FILE_BYTES).unwrap_err();
        let error_zero_bound = source_file_chunk_sizes(1, 0).unwrap_err();
        let excessive_size = u64::try_from(MAX_SOURCE_FILES_PER_RECORD).unwrap().checked_add(1).unwrap();
        let error_excessive_count = source_file_chunk_sizes(excessive_size, 1).unwrap_err();

        assert!(error_zero_size.to_string().contains("positive inputs"));
        assert!(error_zero_bound.to_string().contains("positive inputs"));
        assert!(error_excessive_count.to_string().contains("chunk count exceeds"));
    }

    #[test]
    fn source_bundle_accepts_language_neutral_package_adapters() {
        let temp = tempfile::tempdir().unwrap();
        let cargo_root = temp.path().join("cargo");
        let npm_root = temp.path().join("npm");
        write_fixture(&cargo_root);
        write_fixture(&npm_root);
        let specs = [
            SourceSpec {
                kind: SourceRecordKind::PackageMirror,
                identity: "cargo-mirror".to_string(),
                path: cargo_root,
                adapter: Some(cargo_adapter()),
            },
            SourceSpec {
                kind: SourceRecordKind::PackageMirror,
                identity: "npm-mirror".to_string(),
                path: npm_root,
                adapter: Some(non_cargo_adapter()),
            },
        ];
        let manifest = plan_source_bundle(&specs, "/mantle/store").unwrap();
        let adapters = manifest
            .records
            .iter()
            .map(|record| record.adapter.as_ref().unwrap().adapter.as_str())
            .collect::<Vec<_>>();

        assert_eq!(manifest.records.len(), specs.len());
        assert_eq!(adapters, vec!["cargo", "npm"]);
        assert!(manifest.records.iter().all(|record| record.kind == SourceRecordKind::PackageMirror));
    }

    #[test]
    fn source_bundle_rejects_unsafe_identity() {
        let err = parse_source_spec("local-path:../bad:/tmp").unwrap_err();
        assert!(err.to_string().contains("unsafe source identity"));
    }

    #[test]
    fn source_bundle_rejects_incomplete_adapter_metadata() {
        let temp = tempfile::tempdir().unwrap();
        write_fixture(temp.path());
        let mut adapter = non_cargo_adapter();
        adapter.generated_source_boundary.clear();
        let spec = SourceSpec {
            kind: SourceRecordKind::PackageMirror,
            identity: "npm-mirror".to_string(),
            path: temp.path().to_path_buf(),
            adapter: Some(adapter),
        };

        let err = plan_source_bundle(&[spec], "/mantle/store").unwrap_err();
        assert!(err.to_string().contains("generated-source identity"));
    }

    #[test]
    fn source_bundle_rejects_malformed_roots_and_record_order() {
        let temp = tempfile::tempdir().unwrap();
        let first_root = temp.path().join("first");
        let second_root = temp.path().join("second");
        write_fixture(&first_root);
        write_fixture(&second_root);
        let specs = [
            SourceSpec {
                kind: SourceRecordKind::LocalPath,
                identity: "first".to_string(),
                path: first_root,
                adapter: None,
            },
            SourceSpec {
                kind: SourceRecordKind::LocalPath,
                identity: "second".to_string(),
                path: second_root,
                adapter: None,
            },
        ];
        let manifest = plan_source_bundle(&specs, "/mantle/store").unwrap();

        let mut bad_roots = manifest.clone();
        bad_roots.roots = vec!["second".to_string(), "first".to_string()];
        bad_roots.manifest_blake3 = digest_manifest_without_digest(&bad_roots).unwrap();
        let roots_err = validate_manifest(&bad_roots).unwrap_err();
        assert!(roots_err.to_string().contains("roots do not match"));

        let mut bad_order = manifest;
        bad_order.records.reverse();
        bad_order.roots = bad_order.records.iter().map(|record| record.identity.clone()).collect();
        bad_order.manifest_blake3 = digest_manifest_without_digest(&bad_order).unwrap();
        let order_err = validate_manifest(&bad_order).unwrap_err();
        assert!(order_err.to_string().contains("canonical order"));
    }

    #[test]
    fn source_bundle_rejects_tampered_file_payload_and_store_prefix() {
        let temp = tempfile::tempdir().unwrap();
        let payload = temp.path().join("payload.txt");
        fs::write(&payload, b"payload").unwrap();
        let spec = SourceSpec {
            kind: SourceRecordKind::LocalPath,
            identity: "fixture".to_string(),
            path: payload,
            adapter: None,
        };
        let manifest = plan_source_bundle(&[spec], "/mantle/store").unwrap();

        let tampered_payload = b"tamper";
        let tampered_payload_len = u64::try_from(tampered_payload.len()).unwrap();
        let mut bad_payload = manifest.clone();
        bad_payload.records[0].files[0].content_hex = Some(HEXLOWER.encode(tampered_payload));
        bad_payload.records[0].files[0].size = tampered_payload_len;
        bad_payload.records[0].payload_bytes = tampered_payload_len;
        bad_payload.records[0].content_blake3 = digest_source_record_content(
            &bad_payload.records[0].kind,
            &bad_payload.records[0].metadata,
            &bad_payload.records[0].files,
        )
        .unwrap();
        bad_payload.manifest_blake3 = digest_manifest_without_digest(&bad_payload).unwrap();
        let payload_err = validate_manifest(&bad_payload).unwrap_err();
        assert!(payload_err.to_string().contains("digest mismatch"));

        let provider_root = temp.path().join("provider");
        write_fixture(&provider_root);
        let provider_spec = SourceSpec {
            kind: SourceRecordKind::ProviderManifest,
            identity: "provider".to_string(),
            path: provider_root,
            adapter: None,
        };
        let mut bad_prefix = plan_source_bundle(&[provider_spec], "/mantle/store").unwrap();
        bad_prefix.records[0].store_prefix = Some("/nix/store".to_string());
        bad_prefix.manifest_blake3 = digest_manifest_without_digest(&bad_prefix).unwrap();
        let prefix_err = validate_manifest(&bad_prefix).unwrap_err();
        assert!(prefix_err.to_string().contains("store prefix mismatch"));
    }

    #[test]
    fn source_bundle_covers_bootstrap_provider_toolchain_and_proof_records() {
        let temp = tempfile::tempdir().unwrap();
        let bootstrap_root = temp.path().join("bootstrap");
        let proof_root = temp.path().join("proof");
        let provider_root = temp.path().join("provider");
        let toolchain_root = temp.path().join("toolchain");
        write_fixture(&bootstrap_root);
        write_fixture(&proof_root);
        write_fixture(&provider_root);
        write_fixture(&toolchain_root);
        let specs = [
            SourceSpec {
                kind: SourceRecordKind::BootstrapArchive,
                identity: "bootstrap".to_string(),
                path: bootstrap_root,
                adapter: None,
            },
            SourceSpec {
                kind: SourceRecordKind::ProofInput,
                identity: "proof".to_string(),
                path: proof_root,
                adapter: None,
            },
            SourceSpec {
                kind: SourceRecordKind::ProviderManifest,
                identity: "provider".to_string(),
                path: provider_root,
                adapter: None,
            },
            SourceSpec {
                kind: SourceRecordKind::ToolchainSourceRoot,
                identity: "toolchain".to_string(),
                path: toolchain_root,
                adapter: None,
            },
        ];
        let manifest = plan_source_bundle(&specs, "/mantle/store").unwrap();

        assert_eq!(manifest.records.len(), specs.len());
        assert_eq!(manifest.records[0].kind, SourceRecordKind::BootstrapArchive);
        assert_eq!(manifest.records[1].kind, SourceRecordKind::ProofInput);
        assert_eq!(manifest.records[2].store_prefix.as_deref(), Some("/mantle/store"));
        assert_eq!(manifest.records[3].store_prefix.as_deref(), Some("/mantle/store"));
    }

    // r[verify bootstrap_inventory.offline_bootstrap_source_bundles]
    #[test]
    fn bootstrap_source_profile_exports_imports_and_preflights_ready() {
        let temp = tempfile::tempdir().unwrap();
        let input = bootstrap_profile_fixture(temp.path());
        let manifest = plan_bootstrap_source_bundle_profile(&input, "/mantle/store").unwrap();
        let report = bootstrap_source_bundle_profile_report(&manifest, input.mode).unwrap();
        let state_dir = temp.path().join("state");
        import_source_bundle(&manifest, &state_dir, true).unwrap();

        let preflight = offline_preflight_for_manifest(&manifest, &state_dir).unwrap();

        assert_eq!(report.format, BOOTSTRAP_SOURCE_PROFILE_FORMAT);
        assert_eq!(report.mode, BootstrapSourceBundleMode::SelfBuildProof);
        assert_eq!(report.required_record_count, 7);
        assert_eq!(report.provider_kind, BOOTSTRAP_PROVIDER_KIND_LEGACY_SEED);
        assert_eq!(report.non_claim, BOOTSTRAP_SOURCE_PROFILE_NON_CLAIM);
        assert_eq!(preflight.ready_class, SourceReadiness::Ready);
        assert_eq!(preflight.record_count, report.required_record_count);
        assert!(preflight.source_state_blake3.len() == BLAKE3_HEX_BYTES);
    }

    // r[verify bootstrap_inventory.fresh_clone_source_hydration]
    #[test]
    fn self_build_hydration_materializes_valid_vendor_and_pins_provider_state() {
        let temp = tempfile::tempdir().unwrap();
        let (manifest, checkout) = hydration_fixture(temp.path(), false);
        let state_dir = temp.path().join("state");

        let report =
            hydrate_self_build_source_bundle(&manifest, &manifest.manifest_blake3, &checkout, &state_dir).unwrap();
        let provider =
            bootstrap_legacy_seed_fetch_override_plan(&state_dir, "https://example.invalid/provider.tgz").unwrap();

        assert_eq!(report.format, SELF_BUILD_HYDRATION_REPORT_FORMAT);
        assert_eq!(report.manifest_blake3, manifest.manifest_blake3);
        assert!(report.imported_record_count > 0);
        assert!(report.pinned);
        assert!(checkout.join(VENDOR_DEPS_DIR_NAME).is_dir());
        crate::self_build::require_checked_vendor_inputs(&checkout).unwrap();
        assert_eq!(provider.report.ready_class, SourceReadiness::Ready);
        assert_eq!(provider.overrides.len(), 1);
        assert!(provider.overrides[0].payload_path.join("src/main.txt").is_file());
    }

    // r[verify bootstrap_inventory.fresh_clone_source_hydration]
    #[test]
    fn self_build_hydration_rejects_wrong_manifest_identity_without_outputs() {
        let temp = tempfile::tempdir().unwrap();
        let (manifest, checkout) = hydration_fixture(temp.path(), false);
        let state_dir = temp.path().join("state");
        let wrong_digest = "f".repeat(BLAKE3_HEX_BYTES);

        let error = hydrate_self_build_source_bundle(&manifest, &wrong_digest, &checkout, &state_dir).unwrap_err();

        assert!(error.to_string().contains("manifest BLAKE3 mismatch"));
        assert!(!checkout.join(VENDOR_DEPS_DIR_NAME).exists());
        assert!(!state_dir.exists());
    }

    // r[verify bootstrap_inventory.fresh_clone_source_hydration]
    #[test]
    fn self_build_hydration_rejects_missing_vendor_record_without_outputs() {
        let temp = tempfile::tempdir().unwrap();
        let (manifest, checkout) = hydration_fixture(temp.path(), false);
        let records = manifest
            .records
            .into_iter()
            .filter(|record| {
                record.metadata.get(RECORD_METADATA_PROFILE_CLASS_KEY).map(String::as_str)
                    != Some(BOOTSTRAP_PROFILE_CLASS_VENDOR_DEPS)
            })
            .collect::<Vec<_>>();
        let incomplete = assemble_source_bundle(records, "/mantle/store").unwrap();
        let state_dir = temp.path().join("state");

        let error = hydrate_self_build_source_bundle(&incomplete, &incomplete.manifest_blake3, &checkout, &state_dir)
            .unwrap_err();

        assert!(error.to_string().contains("exactly one vendored-cargo-inputs"));
        assert!(!checkout.join(VENDOR_DEPS_DIR_NAME).exists());
        assert!(!state_dir.exists());
    }

    // r[verify bootstrap_inventory.fresh_clone_source_hydration]
    #[test]
    fn self_build_hydration_preserves_existing_vendor_without_importing_state() {
        let temp = tempfile::tempdir().unwrap();
        let (manifest, checkout) = hydration_fixture(temp.path(), false);
        let destination = checkout.join(VENDOR_DEPS_DIR_NAME);
        fs::create_dir(&destination).unwrap();
        fs::write(destination.join("sentinel"), b"preserve").unwrap();
        let state_dir = temp.path().join("state");

        let error =
            hydrate_self_build_source_bundle(&manifest, &manifest.manifest_blake3, &checkout, &state_dir).unwrap_err();

        assert!(error.to_string().contains("refuses to replace"));
        assert_eq!(fs::read(destination.join("sentinel")).unwrap(), b"preserve");
        assert!(!state_dir.exists());
    }

    // r[verify bootstrap_inventory.fresh_clone_source_hydration]
    #[test]
    fn self_build_hydration_rejects_vendor_checksum_drift_before_publication() {
        let temp = tempfile::tempdir().unwrap();
        let (manifest, checkout) = hydration_fixture(temp.path(), true);
        let state_dir = temp.path().join("state");

        let error =
            hydrate_self_build_source_bundle(&manifest, &manifest.manifest_blake3, &checkout, &state_dir).unwrap_err();

        assert!(error.to_string().contains("vendor file checksum mismatch"));
        assert!(!checkout.join(VENDOR_DEPS_DIR_NAME).exists());
        assert!(!state_dir.exists());
    }

    // r[verify bootstrap_inventory.fresh_clone_source_hydration]
    #[test]
    fn self_build_hydration_rolls_back_vendor_when_state_persistence_fails() {
        let temp = tempfile::tempdir().unwrap();
        let (manifest, checkout) = hydration_fixture(temp.path(), false);
        let state_dir = temp.path().join("state-file");
        fs::write(&state_dir, b"not-a-directory").unwrap();

        let error =
            hydrate_self_build_source_bundle(&manifest, &manifest.manifest_blake3, &checkout, &state_dir).unwrap_err();

        assert!(error.to_string().contains("creating source records dir"));
        assert!(!checkout.join(VENDOR_DEPS_DIR_NAME).exists());
        assert!(state_dir.is_file());
    }

    #[test]
    fn bootstrap_source_profile_fetch_override_consumes_pinned_provider_archive() {
        let temp = tempfile::tempdir().unwrap();
        let input = bootstrap_profile_fixture(temp.path());
        let manifest = plan_bootstrap_source_bundle_profile(&input, "/mantle/store").unwrap();
        let state_dir = temp.path().join("state");
        import_source_bundle(&manifest, &state_dir, true).unwrap();

        let plan =
            bootstrap_legacy_seed_fetch_override_plan(&state_dir, "https://example.invalid/provider.tgz").unwrap();

        assert_eq!(plan.report.ready_class, SourceReadiness::Ready);
        assert_eq!(plan.overrides.len(), 1);
        assert_eq!(plan.overrides[0].kind, crunch_build::FetchSourceOverrideKind::Tarball);
        assert_eq!(plan.overrides[0].url, "https://example.invalid/provider.tgz");
        assert!(plan.overrides[0].payload_path.join("src/main.txt").exists());
    }

    #[test]
    fn fresh_clone_source_profile_fetch_override_consumes_pinned_provider_archive() {
        let temp = tempfile::tempdir().unwrap();
        let mut input = bootstrap_profile_fixture(temp.path());
        input.mode = BootstrapSourceBundleMode::FreshCloneInputs;
        input.bootstrap_sources.clear();
        input.mantle_source = None;
        input.toolchain_source_root = None;
        input.proof_inputs.clear();
        let manifest = plan_bootstrap_source_bundle_profile(&input, "/mantle/store").unwrap();
        let state_dir = temp.path().join("state");
        import_source_bundle(&manifest, &state_dir, true).unwrap();

        let plan =
            bootstrap_legacy_seed_fetch_override_plan(&state_dir, "https://example.invalid/provider.tgz").unwrap();

        assert_eq!(plan.report.ready_class, SourceReadiness::Ready);
        assert_eq!(plan.overrides.len(), 1);
        assert!(plan.overrides[0].payload_path.join("src/main.txt").exists());
    }

    #[test]
    fn bootstrap_fetch_override_scratch_lives_until_plan_drop() {
        let temp = tempfile::tempdir().unwrap();
        let input = bootstrap_profile_fixture(temp.path());
        let manifest = plan_bootstrap_source_bundle_profile(&input, "/mantle/store").unwrap();
        let state_dir = temp.path().join("state");
        import_source_bundle(&manifest, &state_dir, true).unwrap();
        let plan =
            bootstrap_legacy_seed_fetch_override_plan(&state_dir, "https://example.invalid/provider.tgz").unwrap();
        let payload_path = plan.overrides[0].payload_path.clone();
        let retained_overrides = plan.overrides.clone();

        assert!(payload_path.is_dir());
        assert_eq!(retained_overrides[0].payload_path, payload_path);
        drop(plan);
        assert!(!payload_path.exists());
    }

    #[test]
    fn bootstrap_source_profile_fetch_override_rejects_unpinned_provider_archive() {
        let temp = tempfile::tempdir().unwrap();
        let input = bootstrap_profile_fixture(temp.path());
        let manifest = plan_bootstrap_source_bundle_profile(&input, "/mantle/store").unwrap();
        let state_dir = temp.path().join("state");
        import_source_bundle(&manifest, &state_dir, false).unwrap();

        let err =
            bootstrap_legacy_seed_fetch_override_plan(&state_dir, "https://example.invalid/provider.tgz").unwrap_err();

        assert!(err.to_string().contains("unpinned bootstrap provider archive"));
    }

    #[test]
    fn fresh_clone_source_profile_requires_vendor_but_not_unrelated_proof_inputs() {
        let temp = tempfile::tempdir().unwrap();
        let mut input = bootstrap_profile_fixture(temp.path());
        input.mode = BootstrapSourceBundleMode::FreshCloneInputs;
        input.bootstrap_sources.clear();
        input.mantle_source = None;
        input.toolchain_source_root = None;
        input.proof_inputs.clear();

        let manifest = plan_bootstrap_source_bundle_profile(&input, "/mantle/store").unwrap();
        let report = bootstrap_source_bundle_profile_report(&manifest, input.mode).unwrap();

        assert_eq!(report.mode, BootstrapSourceBundleMode::FreshCloneInputs);
        assert_eq!(report.required_record_count, u32::try_from(REQUIRED_HYDRATION_RECORD_CLASS_COUNT).unwrap());
        assert!(manifest.records.iter().any(|record| {
            record.metadata.get(RECORD_METADATA_PROFILE_CLASS_KEY).map(String::as_str)
                == Some(BOOTSTRAP_PROFILE_CLASS_VENDOR_DEPS)
        }));
    }

    #[test]
    fn fresh_clone_fixed_point_profile_adds_materialized_fetches_without_widening_three_record_profile() {
        let temp = tempfile::tempdir().unwrap();
        let mut base_input = bootstrap_profile_fixture(temp.path());
        base_input.mode = BootstrapSourceBundleMode::FreshCloneInputs;
        base_input.bootstrap_sources.clear();
        base_input.mantle_source = None;
        base_input.toolchain_source_root = None;
        base_input.proof_inputs.clear();
        let base_manifest = plan_bootstrap_source_bundle_profile(&base_input, "/mantle/store").unwrap();
        assert_eq!(base_manifest.records.len(), REQUIRED_HYDRATION_RECORD_CLASS_COUNT);

        let payload = temp.path().join("fixed-payload.txt");
        fs::write(&payload, b"fixed payload").unwrap();
        let fetcher = fixed_fetcher("fixed-src", &file_url(&payload));
        let root = root_derivation(vec![crunch_glue::Input::Derivation(Box::new(fetcher))]);
        let planned =
            plan_source_bundle_from_derivations(&[("default".to_string(), root)], &[], "/mantle/store").unwrap();
        let supplemental = materialized_record_from_payload(&planned.records[0], &payload, false);
        let mut fixed_point_input = base_input;
        fixed_point_input.mode = BootstrapSourceBundleMode::FreshCloneFixedPoint;
        fixed_point_input.supplemental_records = vec![supplemental.clone()];

        let fixed_point_manifest = plan_bootstrap_source_bundle_profile(&fixed_point_input, "/mantle/store").unwrap();

        assert_eq!(fixed_point_manifest.records.len(), REQUIRED_HYDRATION_RECORD_CLASS_COUNT + 1);
        assert!(fixed_point_manifest.records.contains(&supplemental));
        assert_eq!(base_manifest.records.len(), REQUIRED_HYDRATION_RECORD_CLASS_COUNT);
    }

    #[test]
    fn full_proof_source_override_plan_uses_pinned_fixed_point_profile() {
        let temp = tempfile::tempdir().unwrap();
        let payload = temp.path().join("fixed-payload.txt");
        fs::write(&payload, b"fixed payload").unwrap();
        let fetcher = fixed_fetcher("fixed-src", &file_url(&payload));
        let root = root_derivation(vec![crunch_glue::Input::Derivation(Box::new(fetcher))]);
        let planned =
            plan_source_bundle_from_derivations(&[("default".to_string(), root)], &[], "/mantle/store").unwrap();
        let supplemental = materialized_record_from_payload(&planned.records[0], &payload, false);
        let mut input = bootstrap_profile_fixture(temp.path());
        input.mode = BootstrapSourceBundleMode::FreshCloneFixedPoint;
        input.bootstrap_sources.clear();
        input.mantle_source = None;
        input.toolchain_source_root = None;
        input.proof_inputs.clear();
        input.supplemental_records = vec![supplemental];
        let manifest = plan_bootstrap_source_bundle_profile(&input, "/mantle/store").unwrap();
        let state_dir = temp.path().join("state");
        import_source_bundle(&manifest, &state_dir, true).unwrap();

        let plan = full_proof_source_fetch_override_plan(&state_dir, &manifest.manifest_blake3).unwrap();

        assert_eq!(plan.report.ready_class, SourceReadiness::Ready);
        assert_eq!(plan.report.source_state_blake3, plan.overrides[0].source_state_blake3);
        assert_eq!(plan.overrides.len(), 1);
        assert_eq!(fs::read(&plan.overrides[0].payload_path).unwrap(), b"fixed payload");
    }

    #[test]
    fn fresh_clone_fixed_point_profile_rejects_missing_materialized_fetch_closure() {
        let temp = tempfile::tempdir().unwrap();
        let mut input = bootstrap_profile_fixture(temp.path());
        input.mode = BootstrapSourceBundleMode::FreshCloneFixedPoint;
        input.bootstrap_sources.clear();
        input.mantle_source = None;
        input.toolchain_source_root = None;
        input.proof_inputs.clear();
        input.supplemental_records.clear();

        let error = plan_bootstrap_source_bundle_profile(&input, "/mantle/store").unwrap_err();

        assert!(error.to_string().contains("requires materialized --include-bundle fetch records"));
        assert!(error.to_string().contains("fresh-clone-fixed-point"));
    }

    #[test]
    fn bootstrap_source_profile_rejects_missing_vendor_for_self_build() {
        let temp = tempfile::tempdir().unwrap();
        let mut input = bootstrap_profile_fixture(temp.path());
        input.vendor_deps = None;

        let err = plan_bootstrap_source_bundle_profile(&input, "/mantle/store").unwrap_err();

        assert!(err.to_string().contains("requires --vendor-deps"));
    }

    #[test]
    fn fresh_clone_source_profile_accepts_runtime_generated_provider_manifest() {
        let temp = tempfile::tempdir().unwrap();
        let mut input = bootstrap_profile_fixture(temp.path());
        input.mode = BootstrapSourceBundleMode::FreshCloneInputs;
        input.bootstrap_sources.clear();
        input.mantle_source = None;
        input.toolchain_source_root = None;
        input.proof_inputs.clear();
        let runtime_manifest = serde_json::json!({
            "provider_id": BOOTSTRAP_PROVIDER_KIND_LEGACY_SEED,
            "target": "x86_64-linux-musl",
            "reduction": {
                "retained_tools": ["cc", "ar"],
                "dropped_components": ["locale-catalogs"]
            }
        });
        fs::write(&input.provider_manifest, serde_json::to_vec_pretty(&runtime_manifest).unwrap()).unwrap();

        let manifest = plan_bootstrap_source_bundle_profile(&input, "/mantle/store").unwrap();
        let report = bootstrap_source_bundle_profile_report(&manifest, input.mode).unwrap();

        assert_eq!(report.provider_kind, BOOTSTRAP_PROVIDER_KIND_LEGACY_SEED);
        assert_eq!(report.mode, BootstrapSourceBundleMode::FreshCloneInputs);
    }

    #[test]
    fn bootstrap_source_profile_rejects_wrong_provider_kind() {
        let temp = tempfile::tempdir().unwrap();
        let input = bootstrap_profile_fixture(temp.path());
        write_provider_manifest(&input.provider_manifest, BOOTSTRAP_PROVIDER_KIND_SOURCE_ROOT);

        let err = plan_bootstrap_source_bundle_profile(&input, "/mantle/store").unwrap_err();

        assert!(err.to_string().contains("provider kind mismatch"));
    }

    #[test]
    fn bootstrap_source_profile_rejects_provider_manifest_without_reduction() {
        let temp = tempfile::tempdir().unwrap();
        let input = bootstrap_profile_fixture(temp.path());
        let bad = serde_json::json!({
            "schema_version": 1,
            "provider_kind": BOOTSTRAP_PROVIDER_KIND_LEGACY_SEED,
            "normalized_seed_contract": { "target": "x86_64-linux-musl" }
        });
        fs::write(&input.provider_manifest, serde_json::to_string_pretty(&bad).unwrap()).unwrap();

        let err = plan_bootstrap_source_bundle_profile(&input, "/mantle/store").unwrap_err();

        assert!(err.to_string().contains("reduced-provider provenance"));
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
    fn source_bundle_preserves_fetch_policy_metadata_from_fetch_derivation() {
        let mut fetcher = fixed_fetcher("crate-src", "https://static.example.invalid/crate.tar.gz");
        fetcher.env.insert(FETCH_ENV_FETCH_POLICY_KEY.to_string(), "build-fetch-action".to_string());
        let root = root_derivation(vec![crunch_glue::Input::Derivation(Box::new(fetcher))]);
        let manifest =
            plan_source_bundle_from_derivations(&[("default".to_string(), root)], &[], "/mantle/store").unwrap();
        let record = manifest.records.iter().find(|record| record.kind == SourceRecordKind::FixedUrl).unwrap();

        assert_eq!(record.metadata.get(FETCH_ENV_FETCH_POLICY_KEY).map(String::as_str), Some("build-fetch-action"));
        assert_eq!(
            record.metadata.get(RECORD_METADATA_URL_KEY).map(String::as_str),
            Some("https://static.example.invalid/crate.tar.gz")
        );
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
    fn source_bundle_rejects_vcs_snapshot_without_revision_identity() {
        let mut git = fixed_fetcher("repo-src", "https://example.invalid/repo.git");
        git.env.insert(FETCH_ENV_TYPE_KEY.to_string(), FETCH_ENV_TYPE_GIT.to_string());
        let root = root_derivation(vec![crunch_glue::Input::Derivation(Box::new(git))]);

        let err =
            plan_source_bundle_from_derivations(&[("default".to_string(), root)], &[], "/mantle/store").unwrap_err();

        assert!(err.to_string().contains("missing env.rev"));
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
    fn source_bundle_export_uses_imported_state_for_remote_fetcher_payload() {
        let temp = tempfile::tempdir().unwrap();
        let payload = temp.path().join("payload.txt");
        fs::write(&payload, b"payload").unwrap();
        let fetcher = fixed_fetcher("remote-src", "https://example.invalid/source.tar.gz");
        let root = root_derivation(vec![crunch_glue::Input::Derivation(Box::new(fetcher))]);
        let roots = [("default".to_string(), root.clone())];
        let planned = plan_source_bundle_from_derivations(&roots, &[], "/mantle/store").unwrap();
        let materialized = materialized_record_from_payload(&planned.records[0], &payload, false);
        let source_state_manifest = assemble_source_bundle(vec![materialized.clone()], "/mantle/store").unwrap();
        let state_dir = temp.path().join("state");
        import_source_bundle(&source_state_manifest, &state_dir, true).unwrap();

        let exported =
            export_source_bundle_from_derivations_with_state(&roots, &[], "/mantle/store", &state_dir).unwrap();

        assert_eq!(exported.records[0].files.len(), 1);
        assert_eq!(exported.records[0].files[0].path, "payload.txt");
        assert_eq!(exported.records[0].content_blake3, materialized.content_blake3);
        assert_eq!(exported.records[0].metadata, planned.records[0].metadata);
    }

    #[test]
    fn source_bundle_export_rejects_imported_remote_payload_with_wrong_metadata() {
        let temp = tempfile::tempdir().unwrap();
        let payload = temp.path().join("payload.txt");
        fs::write(&payload, b"payload").unwrap();
        let fetcher = fixed_fetcher("remote-src", "https://example.invalid/source.tar.gz");
        let root = root_derivation(vec![crunch_glue::Input::Derivation(Box::new(fetcher))]);
        let roots = [("default".to_string(), root.clone())];
        let planned = plan_source_bundle_from_derivations(&roots, &[], "/mantle/store").unwrap();
        let mut wrong = materialized_record_from_payload(&planned.records[0], &payload, false);
        wrong
            .metadata
            .insert(RECORD_METADATA_URL_KEY.to_string(), "https://example.invalid/other.tar.gz".to_string());
        wrong.content_blake3 = digest_source_record_content(&wrong.kind, &wrong.metadata, &wrong.files).unwrap();
        let source_state_manifest = assemble_source_bundle(vec![wrong], "/mantle/store").unwrap();
        let state_dir = temp.path().join("state");
        import_source_bundle(&source_state_manifest, &state_dir, true).unwrap();

        let err =
            export_source_bundle_from_derivations_with_state(&roots, &[], "/mantle/store", &state_dir).unwrap_err();

        assert!(err.to_string().contains("cannot materialize non-local source URL"));
    }

    #[test]
    fn source_bundle_export_materializes_local_vcs_snapshot_without_dot_git() {
        let temp = tempfile::tempdir().unwrap();
        let checkout = temp.path().join("checkout");
        let revision = "0123456789abcdef0123456789abcdef01234567";
        fs::create_dir_all(checkout.join("src")).unwrap();
        write_git_checkout_revision(&checkout, "refs/heads/main", revision);
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
    fn source_bundle_export_rejects_local_vcs_revision_mismatch() {
        let temp = tempfile::tempdir().unwrap();
        let checkout = temp.path().join("checkout");
        let head_revision = "0123456789abcdef0123456789abcdef01234567";
        let requested_revision = "89abcdef0123456789abcdef0123456789abcdef";
        fs::create_dir_all(checkout.join("src")).unwrap();
        write_git_checkout_revision(&checkout, "refs/heads/main", head_revision);
        fs::write(checkout.join("src/main.txt"), b"hello").unwrap();
        let mut git = fixed_fetcher("repo-src", &file_url(&checkout));
        git.env.insert(FETCH_ENV_TYPE_KEY.to_string(), FETCH_ENV_TYPE_GIT.to_string());
        git.env.insert(FETCH_ENV_REV_KEY.to_string(), requested_revision.to_string());
        let root = root_derivation(vec![crunch_glue::Input::Derivation(Box::new(git))]);

        let err =
            export_source_bundle_from_derivations(&[("default".to_string(), root)], &[], "/mantle/store").unwrap_err();

        assert!(err.to_string().contains("checkout revision mismatch"));
    }

    fn write_git_checkout_revision(checkout: &Path, ref_name: &str, revision: &str) {
        let git_dir = checkout.join(DOT_GIT_DIR_NAME);
        let ref_path = git_dir.join(ref_name);
        fs::create_dir_all(ref_path.parent().unwrap()).unwrap();
        fs::write(git_dir.join(GIT_HEAD_REF), format!("{GIT_REF_PREFIX}{ref_name}\n")).unwrap();
        fs::write(ref_path, format!("{revision}\n")).unwrap();
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
    fn source_offline_preflight_accepts_pinned_imported_file_payload() {
        let temp = tempfile::tempdir().unwrap();
        let payload = temp.path().join("payload.txt");
        fs::write(&payload, b"payload").unwrap();
        let fetcher = fixed_fetcher("file-src", &file_url(&payload));
        let root = root_derivation(vec![crunch_glue::Input::Derivation(Box::new(fetcher))]);
        let roots = [("default".to_string(), root.clone())];
        let exported = export_source_bundle_from_derivations(&roots, &[], "/mantle/store").unwrap();
        let state_dir = temp.path().join("state");
        import_source_bundle(&exported, &state_dir, true).unwrap();

        let report = offline_preflight_for_derivations(&roots, &state_dir, "/mantle/store").unwrap();

        assert_eq!(report.ready_class, SourceReadiness::Ready);
        assert_eq!(report.record_count, 1);
        assert!(report.manifest_blake3.is_some());
        assert!(report.missing_records.is_empty());
        assert!(report.network_required_records.is_empty());
        assert!(report.unpinned_records.is_empty());
        assert!(report.next_actions.is_empty());
    }

    #[test]
    fn source_offline_preflight_reports_missing_local_source_state() {
        let temp = tempfile::tempdir().unwrap();
        let payload = temp.path().join("payload.txt");
        fs::write(&payload, b"payload").unwrap();
        let fetcher = fixed_fetcher("file-src", &file_url(&payload));
        let root = root_derivation(vec![crunch_glue::Input::Derivation(Box::new(fetcher))]);

        let report = offline_preflight_for_derivations(
            &[("default".to_string(), root)],
            &temp.path().join("missing-state"),
            "/mantle/store",
        )
        .unwrap();

        assert_eq!(report.ready_class, SourceReadiness::Missing);
        assert_eq!(report.missing_records.len(), 1);
        assert!(report.network_required_records.is_empty());
        assert_eq!(report.next_actions.len(), 1);
        assert_eq!(report.next_actions[0].blocker_class, "missing-source-state");
        assert_eq!(report.next_actions[0].command_hint, SOURCE_NEXT_ACTION_EXPORT_IMPORT_PIN);
    }

    #[test]
    fn source_offline_preflight_reports_remote_network_requirement() {
        let temp = tempfile::tempdir().unwrap();
        let fetcher = fixed_fetcher("remote-src", "https://example.invalid/source.tar.gz");
        let root = root_derivation(vec![crunch_glue::Input::Derivation(Box::new(fetcher))]);

        let report = offline_preflight_for_derivations(
            &[("default".to_string(), root)],
            &temp.path().join("state"),
            "/mantle/store",
        )
        .unwrap();

        assert_eq!(report.ready_class, SourceReadiness::NetworkRequired);
        assert_eq!(report.network_required_records.len(), 1);
        assert!(report.missing_records.is_empty());
        assert_eq!(report.next_actions.len(), 1);
        assert_eq!(report.next_actions[0].blocker_class, "network-required-source");
        assert_eq!(report.next_actions[0].command_hint, SOURCE_NEXT_ACTION_DECLARE_SOURCE);
    }

    #[test]
    fn source_offline_preflight_rejects_unpinned_imported_source() {
        let temp = tempfile::tempdir().unwrap();
        let payload = temp.path().join("payload.txt");
        fs::write(&payload, b"payload").unwrap();
        let fetcher = fixed_fetcher("file-src", &file_url(&payload));
        let root = root_derivation(vec![crunch_glue::Input::Derivation(Box::new(fetcher))]);
        let roots = [("default".to_string(), root.clone())];
        let exported = export_source_bundle_from_derivations(&roots, &[], "/mantle/store").unwrap();
        let state_dir = temp.path().join("state");
        import_source_bundle(&exported, &state_dir, false).unwrap();

        let report = offline_preflight_for_derivations(&roots, &state_dir, "/mantle/store").unwrap();

        assert_eq!(report.ready_class, SourceReadiness::Unpinned);
        assert_eq!(report.unpinned_records.len(), 1);
        assert_eq!(report.next_actions.len(), 1);
        assert_eq!(report.next_actions[0].blocker_class, "unpinned-source-state");
        assert_eq!(report.next_actions[0].command_hint, SOURCE_NEXT_ACTION_PIN_IMPORTED);
    }

    #[test]
    fn connected_export_fetches_and_fixed_output_validates_missing_file_payload() {
        const PAYLOAD: &[u8] = b"connected source payload";
        let temp = tempfile::tempdir().unwrap();
        let payload = temp.path().join("payload.txt");
        fs::write(&payload, PAYLOAD).unwrap();
        let fetcher = fixed_fetcher_with_flat_blake3("connected-src", &file_url(&payload), PAYLOAD);
        let root = root_derivation(vec![crunch_glue::Input::Derivation(Box::new(fetcher))]);
        let roots = [("default".to_string(), root)];

        let manifest = export_source_bundle_from_derivations_with_connected_fetch(
            &roots,
            &[],
            "/mantle/store",
            &temp.path().join("empty-state"),
        )
        .unwrap();

        assert_eq!(manifest.records.len(), 1);
        assert_eq!(manifest.records[0].files.len(), 1);
        assert_eq!(manifest.records[0].payload_bytes, u64::try_from(PAYLOAD.len()).unwrap());
        assert!(manifest.records[0].files[0].content_hex.is_some());
    }

    #[test]
    fn connected_export_rejects_fixed_output_hash_mismatch() {
        const PAYLOAD: &[u8] = b"connected source payload";
        const WRONG_PAYLOAD: &[u8] = b"wrong source payload";
        let temp = tempfile::tempdir().unwrap();
        let payload = temp.path().join("payload.txt");
        fs::write(&payload, PAYLOAD).unwrap();
        let fetcher = fixed_fetcher_with_flat_blake3("connected-src", &file_url(&payload), WRONG_PAYLOAD);
        let root = root_derivation(vec![crunch_glue::Input::Derivation(Box::new(fetcher))]);
        let roots = [("default".to_string(), root)];

        let error = export_source_bundle_from_derivations_with_connected_fetch(
            &roots,
            &[],
            "/mantle/store",
            &temp.path().join("empty-state"),
        )
        .unwrap_err();

        let diagnostic = error.to_string();
        assert!(diagnostic.contains("hash mismatch"), "unexpected diagnostic: {diagnostic}");
        assert!(diagnostic.contains("validating captured source record"), "unexpected diagnostic: {diagnostic}");
    }

    // r[verify source_transports.source_bundle_realizes_fetcher_inputs]
    #[test]
    fn source_fetch_override_plan_materializes_pinned_file_fetcher_payload() {
        let temp = tempfile::tempdir().unwrap();
        let payload = temp.path().join("payload.txt");
        fs::write(&payload, b"payload").unwrap();
        let fetcher = fixed_fetcher("file-src", &file_url(&payload));
        let root = root_derivation(vec![crunch_glue::Input::Derivation(Box::new(fetcher))]);
        let roots = [("default".to_string(), root.clone())];
        let exported = export_source_bundle_from_derivations(&roots, &[], "/mantle/store").unwrap();
        let state_dir = temp.path().join("state");
        import_source_bundle(&exported, &state_dir, true).unwrap();

        let plan = source_fetch_override_plan_for_derivations(&roots, &state_dir, "/mantle/store").unwrap();

        assert_eq!(plan.report.ready_class, SourceReadiness::Ready);
        assert_eq!(plan.overrides.len(), 1);
        assert_eq!(plan.overrides[0].kind, crunch_build::FetchSourceOverrideKind::File);
        assert_eq!(plan.overrides[0].url, file_url(&payload));
        assert_eq!(fs::read(&plan.overrides[0].payload_path).unwrap(), b"payload");
        assert!(plan.overrides[0].source_state_blake3.len() == BLAKE3_HEX_BYTES);
    }

    #[test]
    fn source_fetch_override_plan_deduplicates_identical_url_kind_payloads() {
        let temp = tempfile::tempdir().unwrap();
        let payload = temp.path().join("payload.txt");
        fs::write(&payload, b"payload").unwrap();
        let payload_url = file_url(&payload);
        let first = fixed_fetcher("first-src", &payload_url);
        let second = fixed_fetcher("second-src", &payload_url);
        let root = root_derivation(vec![
            crunch_glue::Input::Derivation(Box::new(first)),
            crunch_glue::Input::Derivation(Box::new(second)),
        ]);
        let roots = [("default".to_string(), root.clone())];
        let exported = export_source_bundle_from_derivations(&roots, &[], "/mantle/store").unwrap();
        let state_dir = temp.path().join("state");
        import_source_bundle(&exported, &state_dir, true).unwrap();

        let plan = source_fetch_override_plan_for_derivations(&roots, &state_dir, "/mantle/store").unwrap();

        assert_eq!(plan.overrides.len(), 1);
        assert_eq!(plan.overrides[0].url, payload_url);
    }

    #[test]
    fn source_fetch_override_mapping_rejects_conflicting_payloads() {
        let url = "https://example.invalid/source.tar";
        let key: SourceOverrideKey = ("tarball", url.to_string(), None);
        let mut mappings = BTreeMap::new();

        assert!(admit_source_override_mapping(&mut mappings, key.clone(), "digest-a".to_string(), url).unwrap());
        let error = admit_source_override_mapping(&mut mappings, key, "digest-b".to_string(), url).unwrap_err();

        assert!(error.to_string().contains("conflicting source override mapping"));
        assert!(error.to_string().contains(url));
    }

    #[test]
    fn source_fetch_override_plan_rejects_unpinned_imported_fetcher_payload() {
        let temp = tempfile::tempdir().unwrap();
        let payload = temp.path().join("payload.txt");
        fs::write(&payload, b"payload").unwrap();
        let fetcher = fixed_fetcher("file-src", &file_url(&payload));
        let root = root_derivation(vec![crunch_glue::Input::Derivation(Box::new(fetcher))]);
        let roots = [("default".to_string(), root.clone())];
        let exported = export_source_bundle_from_derivations(&roots, &[], "/mantle/store").unwrap();
        let state_dir = temp.path().join("state");
        import_source_bundle(&exported, &state_dir, false).unwrap();

        let err = source_fetch_override_plan_for_derivations(&roots, &state_dir, "/mantle/store").unwrap_err();

        assert!(err.to_string().contains("offline source preflight is not ready"));
        assert!(err.to_string().contains("Unpinned"));
    }

    // r[verify source_transports.source_bundle_realizes_fetcher_inputs]
    #[test]
    fn source_fetch_override_plan_materializes_tarball_and_vcs_payloads() {
        let temp = tempfile::tempdir().unwrap();
        let tar_payload = temp.path().join("tar-payload");
        let tar_archive = temp.path().join("source.tar");
        let vcs_payload = temp.path().join("vcs-payload");
        write_fixture(&tar_payload);
        write_test_tar_archive(&tar_payload, &tar_archive);
        write_fixture(&vcs_payload);
        let mut tarball = fixed_fetcher("tar-src", "https://example.invalid/source.tar");
        tarball.env.insert(FETCH_ENV_UNPACK_KEY.to_string(), "1".to_string());
        let mut vcs = fixed_fetcher("repo-src", "https://example.invalid/repo.git");
        vcs.env.insert(FETCH_ENV_TYPE_KEY.to_string(), FETCH_ENV_TYPE_GIT.to_string());
        vcs.env
            .insert(FETCH_ENV_REV_KEY.to_string(), "0123456789abcdef0123456789abcdef01234567".to_string());
        let root = root_derivation(vec![
            crunch_glue::Input::Derivation(Box::new(tarball)),
            crunch_glue::Input::Derivation(Box::new(vcs)),
        ]);
        let roots = [("default".to_string(), root.clone())];
        let planned = plan_source_bundle_from_derivations(&roots, &[], "/mantle/store").unwrap();
        let tar_record = planned.records.iter().find(|record| record.kind == SourceRecordKind::FixedUrl).unwrap();
        let vcs_record = planned.records.iter().find(|record| record.kind == SourceRecordKind::VcsSnapshot).unwrap();
        let imported = vec![
            materialized_record_from_payload(tar_record, &tar_archive, false),
            materialized_record_from_payload(vcs_record, &vcs_payload, true),
        ];
        let state_dir = temp.path().join("state");
        let source_state_manifest = assemble_source_bundle(imported, "/mantle/store").unwrap();
        import_source_bundle(&source_state_manifest, &state_dir, true).unwrap();

        let plan = source_fetch_override_plan_for_derivations(&roots, &state_dir, "/mantle/store").unwrap();

        assert_eq!(plan.report.ready_class, SourceReadiness::Ready);
        assert_eq!(plan.overrides.len(), 2);
        let tar_override = plan
            .overrides
            .iter()
            .find(|source_override| source_override.kind == crunch_build::FetchSourceOverrideKind::Tarball)
            .unwrap();
        let vcs_override = plan
            .overrides
            .iter()
            .find(|source_override| source_override.kind == crunch_build::FetchSourceOverrideKind::Git)
            .unwrap();
        assert!(tar_override.payload_path.join("src/main.txt").exists());
        assert!(vcs_override.payload_path.join("src/main.txt").exists());
        assert_eq!(vcs_override.rev.as_deref(), Some("0123456789abcdef0123456789abcdef01234567"));
    }

    #[test]
    fn source_fetch_override_plan_rejects_wrong_vcs_revision_metadata() {
        let temp = tempfile::tempdir().unwrap();
        let vcs_payload = temp.path().join("vcs-payload");
        write_fixture(&vcs_payload);
        let expected_revision = "0123456789abcdef0123456789abcdef01234567";
        let wrong_revision = "89abcdef0123456789abcdef0123456789abcdef";
        let mut vcs = fixed_fetcher("repo-src", "https://example.invalid/repo.git");
        vcs.env.insert(FETCH_ENV_TYPE_KEY.to_string(), FETCH_ENV_TYPE_GIT.to_string());
        vcs.env.insert(FETCH_ENV_REV_KEY.to_string(), expected_revision.to_string());
        let root = root_derivation(vec![crunch_glue::Input::Derivation(Box::new(vcs))]);
        let roots = [("default".to_string(), root.clone())];
        let planned = plan_source_bundle_from_derivations(&roots, &[], "/mantle/store").unwrap();
        let mut wrong_record = materialized_record_from_payload(&planned.records[0], &vcs_payload, true);
        wrong_record.metadata.insert(FETCH_ENV_REV_KEY.to_string(), wrong_revision.to_string());
        wrong_record.content_blake3 =
            digest_source_record_content(&wrong_record.kind, &wrong_record.metadata, &wrong_record.files).unwrap();
        let state_dir = temp.path().join("state");
        let source_state_manifest = assemble_source_bundle(vec![wrong_record], "/mantle/store").unwrap();
        import_source_bundle(&source_state_manifest, &state_dir, true).unwrap();

        let err = source_fetch_override_plan_for_derivations(&roots, &state_dir, "/mantle/store").unwrap_err();

        assert!(err.to_string().contains("offline source preflight is not ready"));
        assert!(err.to_string().contains("Stale"));
    }

    #[test]
    fn source_offline_preflight_reports_unsupported_adapter() {
        let temp = tempfile::tempdir().unwrap();
        write_fixture(temp.path());
        let spec = SourceSpec {
            kind: SourceRecordKind::PackageMirror,
            identity: "unsupported-adapter".to_string(),
            path: temp.path().to_path_buf(),
            adapter: Some(adapter_with_extra(ADAPTER_EXTRA_UNSUPPORTED_KEY, ADAPTER_EXTRA_TRUE_VALUE)),
        };
        let manifest = plan_source_bundle(&[spec], "/mantle/store").unwrap();
        let report = offline_preflight_for_manifest(&manifest, &temp.path().join("state")).unwrap();

        assert_eq!(report.ready_class, SourceReadiness::Unsupported);
        assert_eq!(report.unsupported_records, vec!["unsupported-adapter".to_string()]);
        assert!(
            report.next_actions.iter().any(|action| action.blocker_class == "unsupported-source-adapter"
                && action.command_hint == SOURCE_NEXT_ACTION_INSPECT_ADAPTER),
            "next actions should include unsupported adapter remediation: {:#?}",
            report.next_actions
        );
    }

    #[test]
    fn source_offline_preflight_reports_untrusted_adapter() {
        let temp = tempfile::tempdir().unwrap();
        write_fixture(temp.path());
        let spec = SourceSpec {
            kind: SourceRecordKind::PackageMirror,
            identity: "untrusted-adapter".to_string(),
            path: temp.path().to_path_buf(),
            adapter: Some(adapter_with_extra(ADAPTER_EXTRA_TRUSTED_PROVENANCE_KEY, "false")),
        };
        let manifest = plan_source_bundle(&[spec], "/mantle/store").unwrap();
        import_source_bundle(&manifest, &temp.path().join("state"), true).unwrap();
        let report = offline_preflight_for_manifest(&manifest, &temp.path().join("state")).unwrap();

        assert_eq!(report.ready_class, SourceReadiness::Untrusted);
        assert_eq!(report.untrusted_records, vec!["untrusted-adapter".to_string()]);
        assert_eq!(report.next_actions.len(), 1);
        assert_eq!(report.next_actions[0].blocker_class, "untrusted-source-adapter");
        assert_eq!(report.next_actions[0].command_hint, SOURCE_NEXT_ACTION_TRUST_PROVENANCE);
    }

    #[test]
    fn source_offline_preflight_allows_roots_without_source_requirements() {
        let temp = tempfile::tempdir().unwrap();
        let root = root_derivation(Vec::new());

        let report = offline_preflight_for_derivations(
            &[("default".to_string(), root)],
            &temp.path().join("state"),
            "/mantle/store",
        )
        .unwrap();

        assert_eq!(report.ready_class, SourceReadiness::Ready);
        assert_eq!(report.record_count, 0);
        assert!(report.manifest_blake3.is_none());
        assert!(report.next_actions.is_empty());
    }

    #[test]
    fn source_bundle_walks_lazy_derivation_files_without_recursive_expansion() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("root.ncl");
        let source = temp.path().join("source.ncl");
        fs::write(&root, r#"{ name = "root", builder = "/bin/sh", inputs = [{ derivation_file = "source.ncl" }] }"#)
            .unwrap();
        fs::write(
            &source,
            r#"{
              name = "fixture-source",
              builder = "builtin:fetchurl",
              env = { url = "https://example.invalid/source.tar" },
              fixed_output = {
                hash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
                algo = "sha256",
                mode = "flat",
              },
            }"#,
        )
        .unwrap();

        let records = collect_build_source_records_from_files(&[root], &[], "/mantle/store").unwrap();

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].metadata.get(RECORD_METADATA_NAME_KEY).map(String::as_str), Some("fixture-source"));
        assert_eq!(
            records[0].metadata.get(RECORD_METADATA_URL_KEY).map(String::as_str),
            Some("https://example.invalid/source.tar")
        );
    }

    #[test]
    fn source_bundle_rejects_lazy_derivation_file_cycles() {
        let temp = tempfile::tempdir().unwrap();
        let first = temp.path().join("first.ncl");
        let second = temp.path().join("second.ncl");
        fs::write(&first, r#"{ name = "first", builder = "/bin/sh", inputs = [{ derivation_file = "second.ncl" }] }"#)
            .unwrap();
        fs::write(&second, r#"{ name = "second", builder = "/bin/sh", inputs = [{ derivation_file = "first.ncl" }] }"#)
            .unwrap();

        let error = collect_build_source_records_from_files(&[first], &[], "/mantle/store").unwrap_err();

        assert!(error.to_string().contains("derivation-file cycle detected"));
        assert!(!error.to_string().contains("resolved derivation"));
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

    #[test]
    fn source_bundle_rejects_file_below_symlink_record() {
        let content = b"payload";
        let regular = SourceFileEntry {
            path: "link/payload.txt".to_string(),
            file_type: SourceFileType::Regular,
            executable: false,
            size: u64::try_from(content.len()).unwrap(),
            content_hex: Some(HEXLOWER.encode(content)),
            symlink_target: None,
            chunk_index: None,
            chunk_count: None,
            blake3: blake3::hash(content).to_hex().to_string(),
        };
        let symlink = SourceFileEntry {
            path: "link".to_string(),
            file_type: SourceFileType::Symlink,
            executable: false,
            size: SYMLINK_PAYLOAD_BYTES,
            content_hex: None,
            symlink_target: Some("safe-target".to_string()),
            chunk_index: None,
            chunk_count: None,
            blake3: blake3::hash(b"symlink\0link\0safe-target").to_hex().to_string(),
        };
        let files = vec![symlink, regular];
        let content_blake3 =
            digest_source_record_content(&SourceRecordKind::LocalPath, &BTreeMap::new(), &files).unwrap();
        let record = SourceRecord {
            kind: SourceRecordKind::LocalPath,
            identity: "symlink-descendant".to_string(),
            store_prefix: None,
            adapter: None,
            metadata: BTreeMap::new(),
            payload_bytes: u64::try_from(content.len()).unwrap(),
            content_blake3,
            files,
        };

        let err = validate_source_record(&record).unwrap_err();
        assert!(err.to_string().contains("below symlink"));
    }
}
