//! Pure frontend-neutral OCI projection and descriptor logic.
//!
//! r[impl kernel_bundle_oci.projection]
//! r[impl kernel_bundle_oci.admission]
//! r[impl kernel_bundle_oci.layering]
//! r[impl kernel_bundle_oci.digest_roles]
//! r[impl kernel_bundle_oci.export]
//! r[impl kernel_bundle_oci.import]
//! r[related kernel_bundle_oci.reports]

// machine-artifact-public: oci.export-report
// machine-artifact-public: oci.import-report
use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;
use sha2::Sha256;

mod archive;
mod layout;
#[cfg(test)]
mod tests;

pub const OCI_PROJECTION_SCHEMA: &str = "mantle-oci-projection-v1";
pub const SOURCE_ADMISSION_BUNDLE_SCHEMA: &str = "mantle-oci-projection-admissions-v1";
pub const OCI_EXPORT_REPORT_SCHEMA: &str = "mantle-oci-export-report-v1";
pub const OCI_IMPORT_REPORT_SCHEMA: &str = "mantle-oci-import-report-v1";
pub const CANONICAL_ARCHIVE_SCHEMA: &str = "mantle-canonical-archive-v1";
pub const OCI_LAYOUT_FILENAME: &str = "oci-layout";
pub const OCI_INDEX_FILENAME: &str = "index.json";
pub const OCI_BLOB_DIR: &str = "blobs/sha256";
pub const OCI_EXPORT_REPORT_FILENAME: &str = "mantle-oci-export-report.json";
const MEBIBYTE_BYTES: u64 = 1_048_576;
const OCI_INPUT_LIMIT_MEBIBYTES: u64 = 4;
const OCI_DOCUMENT_LIMIT_MEBIBYTES: u64 = 4;
pub const OCI_INPUT_MAX_BYTES: u64 = OCI_INPUT_LIMIT_MEBIBYTES * MEBIBYTE_BYTES;
pub const OCI_DOCUMENT_MAX_BYTES: u64 = OCI_DOCUMENT_LIMIT_MEBIBYTES * MEBIBYTE_BYTES;
pub const OCI_BLOB_MAX_BYTES: u64 = 1_073_741_824;
pub const MANTLE_REF_PREFIX: &str = "mantle://blake3/";
pub const GENERIC_BLAKE3_REF_PREFIX: &str = "blake3:";
pub const SHA256_PREFIX: &str = "sha256:";
pub const EXPECTED_DIGEST_ROLE_OCI_LAYER_BLOB: &str = "oci_layer_blob_sha256";

pub(crate) const OCI_LAYOUT_VERSION: &str = "1.0.0";
pub(crate) const OCI_MANIFEST_MEDIA_TYPE: &str = "application/vnd.oci.image.manifest.v1+json";
pub(crate) const OCI_CONFIG_MEDIA_TYPE: &str = "application/vnd.oci.image.config.v1+json";
pub(crate) const OCI_INDEX_MEDIA_TYPE: &str = "application/vnd.oci.image.index.v1+json";
pub(crate) const SCHEMA_VERSION: u16 = 1;
pub(crate) const PROJECTION_HASH_DOMAIN: &str = "mantle/oci/projection/v1";
pub(crate) const LAYOUT_HASH_DOMAIN: &str = "mantle/oci/layout/v1";
pub(crate) const EXPORT_HASH_DOMAIN: &str = "mantle/oci/export-report/v1";
pub(crate) const IMPORT_HASH_DOMAIN: &str = "mantle/oci/import-report/v1";
pub(crate) const OBJECT_ADMISSION_HASH_DOMAIN: &str = "mantle/oci/object-admissions/v1";
pub(crate) const SOURCE_ADMISSION_HASH_DOMAIN: &str = "mantle/oci/source-admission/v1";
pub(crate) const STATUS_ADMITTED: &str = "admitted";
pub(crate) const STATUS_COMPATIBILITY_ONLY: &str = "compatibility-only";
pub(crate) const ROLE_ANNOTATION: &str = "org.mantle.layer.role";
pub(crate) const SOURCE_IDENTITIES_ANNOTATION: &str = "org.mantle.layer.source-identities";
pub(crate) const PACK_IDENTITY_ANNOTATION: &str = "org.mantle.layer.pack-identity";
pub(crate) const PROJECTION_ANNOTATION: &str = "org.mantle.projection.blake3";
pub(crate) const FRONTEND_SPEC_ID_ANNOTATION: &str = "org.mantle.frontend.spec.id";
pub(crate) const FRONTEND_SPEC_VERSION_ANNOTATION: &str = "org.mantle.frontend.spec.version";
pub(crate) const FRONTEND_SPEC_HASH_ANNOTATION: &str = "org.mantle.frontend.spec.blake3";

const HEX_CHARS_PER_BYTE: usize = 2;
const SHA256_DIGEST_BYTES: usize = 32;
const BLAKE3_HEX_LENGTH: usize = blake3::OUT_LEN * HEX_CHARS_PER_BYTE;
const SHA256_HEX_LENGTH: usize = SHA256_DIGEST_BYTES * HEX_CHARS_PER_BYTE;
pub(crate) const OCI_LAYER_MAX_COUNT: usize = 64;
const MAX_ENTRIES_HARD: usize = 16_384;
const MAX_DEPTH_HARD: usize = 64;
const MAX_TOTAL_BYTES_HARD: u64 = OCI_BLOB_MAX_BYTES;
const MAX_TEXT_BYTES: usize = 4_096;
const MAX_ANNOTATIONS: usize = 256;
const MAX_ISSUES: usize = 128;
const PAIR_WINDOW: usize = 2;
const CANONICAL_UID: u64 = 0;
const CANONICAL_GID: u64 = 0;
const CANONICAL_MTIME: u64 = 0;
const CANONICAL_FILE_MODE: u32 = 0o644;
const CANONICAL_DIRECTORY_MODE: u32 = 0o755;
const CANONICAL_SYMLINK_MODE: u32 = 0o777;
const RESERVED_ANNOTATION_KEYS: &[&str] = &[
    ROLE_ANNOTATION,
    SOURCE_IDENTITIES_ANNOTATION,
    PACK_IDENTITY_ANNOTATION,
    PROJECTION_ANNOTATION,
    FRONTEND_SPEC_ID_ANNOTATION,
    FRONTEND_SPEC_VERSION_ANNOTATION,
    FRONTEND_SPEC_HASH_ANNOTATION,
];
const OVERCLAIM_FRAGMENTS: &[&str] = &[
    "bootable",
    "deployable",
    "deployment approved",
    "kernel compatible",
    "registry published",
    "release eligible",
    "signature verified",
];
const SENSITIVE_ANNOTATION_FRAGMENTS: &[&str] = &[
    "api-key",
    "api_key",
    "authorization",
    "credential",
    "password",
    "private",
    "secret",
    "token",
];

const REQUIRED_NON_CLAIMS: &[&str] = &[
    "no registry publication",
    "no kernel compatibility decision",
    "no bootability claim",
    "no deployability claim",
    "no release eligibility claim",
    "no signature policy claim",
    "frontend semantics remain external",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompressionProfile {
    None,
    GzipDeterministicV1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayerMode {
    ExactBlob,
    CanonicalArchive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectEntryKind {
    Directory,
    File,
    Symlink,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionBounds {
    pub max_layers: usize,
    pub max_entries_per_layer: usize,
    pub max_depth: usize,
    pub max_total_bytes: u64,
}

impl Default for ProjectionBounds {
    fn default() -> Self {
        Self {
            max_layers: OCI_LAYER_MAX_COUNT,
            max_entries_per_layer: MAX_ENTRIES_HARD,
            max_depth: MAX_DEPTH_HARD,
            max_total_bytes: MAX_TOTAL_BYTES_HARD,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Platform {
    pub architecture: String,
    pub os: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArchivePolicy {
    pub schema: String,
    pub compression: CompressionProfile,
    pub uid: u64,
    pub gid: u64,
    pub mtime: u64,
    pub file_mode: u32,
    pub directory_mode: u32,
    pub symlink_mode: u32,
    pub relative_symlinks_only: bool,
}

impl Default for ArchivePolicy {
    fn default() -> Self {
        Self {
            schema: CANONICAL_ARCHIVE_SCHEMA.to_string(),
            compression: CompressionProfile::None,
            uid: CANONICAL_UID,
            gid: CANONICAL_GID,
            mtime: CANONICAL_MTIME,
            file_mode: CANONICAL_FILE_MODE,
            directory_mode: CANONICAL_DIRECTORY_MODE,
            symlink_mode: CANONICAL_SYMLINK_MODE,
            relative_symlinks_only: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrontendSpecBinding {
    pub id: String,
    pub version: String,
    pub hash_algorithm: String,
    pub hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceArtifactAdmission {
    pub spec_id: String,
    pub spec_version: String,
    pub spec_hash_algorithm: String,
    pub spec_hash: String,
    pub validator_kind: String,
    pub validator_ref: String,
    pub artifact_kind: String,
    pub artifact_ref: String,
    pub artifact_digest: Option<String>,
    pub target_identity: Option<String>,
    pub build_root: String,
    pub validation_result: String,
    pub no_hidden_fallback: bool,
}

#[derive(Serialize)]
struct SourceAdmissionIdentityMaterial<'a> {
    spec_id: &'a str,
    spec_version: &'a str,
    spec_hash_algorithm: &'a str,
    spec_hash: &'a str,
    validator_kind: &'a str,
    validator_ref: &'a str,
    artifact_kind: &'a str,
    artifact_ref: &'a str,
    artifact_digest: Option<&'a str>,
    target_identity: Option<&'a str>,
    validation_result: &'a str,
    no_hidden_fallback: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceAdmissionBundle {
    pub schema: String,
    pub admissions: Vec<SourceArtifactAdmission>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionObjectAdmission {
    pub spec_id: String,
    pub spec_version: String,
    pub spec_hash_algorithm: String,
    pub spec_hash: String,
    pub validator_kind: String,
    pub validator_ref: String,
    pub artifact_kind: String,
    pub artifact_ref: String,
    pub artifact_digest: Option<String>,
    pub target_identity: Option<String>,
    pub source_attestation_blake3: String,
    pub validation_result: String,
    pub no_hidden_fallback: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionAdmission {
    pub validation_result: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projection_blake3: Option<String>,
    pub no_hidden_fallback: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionEntry {
    pub relative_path: String,
    pub object_ref: String,
    pub identity: String,
    pub size_bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionLayer {
    pub role: String,
    pub media_type: String,
    pub mode: LayerMode,
    pub entries: Vec<ProjectionEntry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pack_identity: Option<String>,
    #[serde(default)]
    pub annotations: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalDigestExpectation {
    pub role: String,
    pub subject_identity: String,
    pub digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoundTripExpectation {
    pub kernel_build_identity: String,
    pub bundle_identity: String,
    pub manifest_identity: String,
    pub component_identities: Vec<String>,
    pub pack_identities: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OciProjection {
    pub schema: String,
    pub frontend_spec: FrontendSpecBinding,
    pub admission: ProjectionAdmission,
    pub platform: Platform,
    pub bounds: ProjectionBounds,
    pub archive_policy: ArchivePolicy,
    pub object_admissions: Vec<ProjectionObjectAdmission>,
    pub layers: Vec<ProjectionLayer>,
    #[serde(default)]
    pub required_annotations: BTreeMap<String, String>,
    pub expected_external_digests: Vec<ExternalDigestExpectation>,
    pub round_trip: RoundTripExpectation,
    #[serde(default)]
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionIssue {
    pub code: String,
    pub path: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectEntry {
    pub relative_path: String,
    pub kind: ObjectEntryKind,
    pub data: Vec<u8>,
    pub link_target: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaterializedObject {
    pub artifact_ref: String,
    pub entries: Vec<ObjectEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OciDescriptor {
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub digest: String,
    pub size: u64,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub annotations: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform: Option<OciPlatform>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OciPlatform {
    pub architecture: String,
    pub os: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OciLayoutDocument {
    #[serde(rename = "imageLayoutVersion")]
    pub image_layout_version: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OciIndexDocument {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u16,
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub manifests: Vec<OciDescriptor>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OciManifestDocument {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u16,
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub config: OciDescriptor,
    pub layers: Vec<OciDescriptor>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub annotations: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayerDigestRecord {
    pub role: String,
    pub media_type: String,
    pub source_object_refs: Vec<String>,
    pub source_identities: Vec<String>,
    pub uncompressed_sha256: String,
    pub blob_sha256: String,
    pub blob_blake3: String,
    pub size_bytes: u64,
    pub canonical_archive: bool,
    pub compression: CompressionProfile,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlannedBlob {
    pub digest: String,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportPlan {
    pub projection: OciProjection,
    pub projection_blake3: String,
    pub oci_layout_bytes: Vec<u8>,
    pub index_bytes: Vec<u8>,
    pub blobs: Vec<PlannedBlob>,
    pub layers: Vec<LayerDigestRecord>,
    pub manifest_descriptor: OciDescriptor,
    pub config_descriptor: OciDescriptor,
    pub layout_blake3: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OciExportReport {
    pub schema: String,
    pub schema_version: u16,
    pub exported: bool,
    pub frontend_spec: FrontendSpecBinding,
    pub admission_count: usize,
    pub object_admissions_blake3: String,
    pub no_hidden_fallback: bool,
    pub projection_blake3: String,
    pub projection: OciProjection,
    pub layout_blake3: String,
    pub manifest_descriptor: OciDescriptor,
    pub config_descriptor: OciDescriptor,
    pub layers: Vec<LayerDigestRecord>,
    pub round_trip: RoundTripExpectation,
    pub issues: Vec<ProjectionIssue>,
    pub non_claims: Vec<String>,
    pub receipt_blake3: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayoutFacts {
    pub oci_layout_bytes: Vec<u8>,
    pub index_bytes: Vec<u8>,
    pub blobs: BTreeMap<String, Vec<u8>>,
    pub export_report: Option<OciExportReport>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportedObjectRecord {
    pub role: String,
    pub media_type: String,
    pub oci_sha256: String,
    pub blob_blake3: String,
    pub size_bytes: u64,
    pub archive_profile: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_ref: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OciImportReport {
    pub schema: String,
    pub schema_version: u16,
    pub imported: bool,
    pub state: String,
    pub layout_blake3: String,
    pub manifest_descriptor: Option<OciDescriptor>,
    pub objects: Vec<ImportedObjectRecord>,
    pub frontend_spec: Option<FrontendSpecBinding>,
    pub object_admissions_blake3: Option<String>,
    pub projection_blake3: Option<String>,
    pub round_trip: Option<RoundTripExpectation>,
    pub issues: Vec<ProjectionIssue>,
    pub non_claims: Vec<String>,
    pub receipt_blake3: String,
}

fn issue(code: &str, path: impl Into<String>, message: &str) -> ProjectionIssue {
    ProjectionIssue {
        code: code.to_string(),
        path: path.into(),
        message: message.to_string(),
    }
}

fn lower_hex(value: &str, length: usize) -> bool {
    value.len() == length && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

pub fn is_blake3_hex(value: &str) -> bool {
    lower_hex(value, BLAKE3_HEX_LENGTH)
}

pub fn is_sha256_digest(value: &str) -> bool {
    value.strip_prefix(SHA256_PREFIX).is_some_and(|digest| lower_hex(digest, SHA256_HEX_LENGTH))
}

pub fn artifact_ref_digest(value: &str) -> Option<&str> {
    [MANTLE_REF_PREFIX, GENERIC_BLAKE3_REF_PREFIX]
        .into_iter()
        .find_map(|prefix| value.strip_prefix(prefix))
        .filter(|digest| is_blake3_hex(digest))
}

pub fn canonical_mantle_ref(value: &str) -> Option<String> {
    artifact_ref_digest(value).map(|digest| format!("{MANTLE_REF_PREFIX}{digest}"))
}

pub fn blake3_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

pub fn sha256_digest(bytes: &[u8]) -> String {
    format!("{SHA256_PREFIX}{:x}", <Sha256 as sha2::Digest>::digest(bytes))
}

pub(crate) fn domain_hash(domain: &str, parts: &[&[u8]]) -> String {
    assert!(!domain.is_empty(), "hash domain must be explicit");
    assert!(!parts.is_empty(), "hash material must not be empty");
    let mut hasher = blake3::Hasher::new_derive_key(domain);
    for part in parts {
        let part_len = u64::try_from(part.len());
        assert!(part_len.is_ok(), "hash material length must fit u64");
        hasher.update(&part_len.unwrap_or_default().to_le_bytes());
        hasher.update(part);
    }
    hasher.finalize().to_hex().to_string()
}

fn safe_text(value: &str) -> bool {
    !value.is_empty() && value.len() <= MAX_TEXT_BYTES && !value.bytes().any(|byte| byte.is_ascii_control())
}

pub fn safe_relative_path(value: &str, max_depth: usize) -> bool {
    if value.is_empty()
        || value.starts_with('/')
        || value.ends_with('/')
        || value.contains('\0')
        || value.contains('\\')
    {
        return false;
    }
    let components = value.split('/').collect::<Vec<_>>();
    components.len() <= max_depth && components.iter().all(|part| !part.is_empty() && *part != "." && *part != "..")
}

fn valid_media_type(value: &str) -> bool {
    safe_text(value)
        && value.contains('/')
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(byte, b'!' | b'#' | b'$' | b'&' | b'^' | b'_' | b'.' | b'+' | b'-' | b'/')
        })
}

fn push(issues: &mut Vec<ProjectionIssue>, item: ProjectionIssue) {
    if issues.len() < MAX_ISSUES {
        issues.push(item);
    }
}

fn contains_overclaim(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    OVERCLAIM_FRAGMENTS.iter().any(|fragment| lowered.contains(fragment))
}

fn contains_sensitive_fragment(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    SENSITIVE_ANNOTATION_FRAGMENTS.iter().any(|fragment| lowered.contains(fragment))
}

fn validate_annotations(annotations: &BTreeMap<String, String>, path: &str, issues: &mut Vec<ProjectionIssue>) {
    for (key, value) in annotations {
        let sensitive_key = contains_sensitive_fragment(key);
        let reserved_key = RESERVED_ANNOTATION_KEYS.contains(&key.as_str());
        let path_like_value = value.starts_with('/') || value.starts_with("file://");
        if !safe_text(key)
            || !safe_text(value)
            || sensitive_key
            || reserved_key
            || path_like_value
            || contains_overclaim(value)
        {
            push(
                issues,
                issue(
                    "unsafe-annotation",
                    format!("{path}.{key}"),
                    "annotations must be bounded, redaction-safe, and path-free",
                ),
            );
        }
    }
}

fn validate_archive_policy(policy: &ArchivePolicy, issues: &mut Vec<ProjectionIssue>) {
    let expected = ArchivePolicy::default();
    let valid = policy.schema == expected.schema
        && policy.uid == expected.uid
        && policy.gid == expected.gid
        && policy.mtime == expected.mtime
        && policy.file_mode == expected.file_mode
        && policy.directory_mode == expected.directory_mode
        && policy.symlink_mode == expected.symlink_mode
        && policy.relative_symlinks_only == expected.relative_symlinks_only;
    if !valid {
        push(
            issues,
            issue(
                "archive-policy-mismatch",
                "archive_policy",
                "archive policy must equal the complete mantle-canonical-archive-v1 profile",
            ),
        );
    }
}

fn validate_bounds(bounds: &ProjectionBounds, issues: &mut Vec<ProjectionIssue>) {
    let valid = bounds.max_layers > 0
        && bounds.max_layers <= OCI_LAYER_MAX_COUNT
        && bounds.max_entries_per_layer > 0
        && bounds.max_entries_per_layer <= MAX_ENTRIES_HARD
        && bounds.max_depth > 0
        && bounds.max_depth <= MAX_DEPTH_HARD
        && bounds.max_total_bytes > 0
        && bounds.max_total_bytes <= MAX_TOTAL_BYTES_HARD;
    if !valid {
        push(issues, issue("invalid-bounds", "bounds", "projection bounds exceed Mantle hard limits"));
    }
}

pub fn projection_identity(projection: &OciProjection) -> Result<String, String> {
    let mut material = projection.clone();
    material.admission.projection_blake3 = None;
    let bytes = serde_json::to_vec(&material).map_err(|error| error.to_string())?;
    Ok(domain_hash(PROJECTION_HASH_DOMAIN, &[&bytes]))
}

/// Canonicalize and seal a draft projection for frontend adapter use.
#[allow(dead_code)] // Public through the library; the binary only verifies sealed input.
pub fn seal_projection(projection: &mut OciProjection) -> Result<String, String> {
    projection.layers.sort_by(|left, right| left.role.cmp(&right.role));
    for layer in &mut projection.layers {
        layer.entries.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    }
    projection.object_admissions.sort_by(|left, right| left.artifact_ref.cmp(&right.artifact_ref));
    projection
        .expected_external_digests
        .sort_by(|left, right| left.subject_identity.cmp(&right.subject_identity));
    projection.round_trip.component_identities.sort();
    projection.non_claims.sort();
    projection.admission.projection_blake3 = None;
    let identity = projection_identity(projection)?;
    projection.admission.projection_blake3 = Some(identity.clone());
    Ok(identity)
}

pub fn source_admission_identity(admission: &SourceArtifactAdmission) -> Result<String, String> {
    let material = SourceAdmissionIdentityMaterial {
        spec_id: &admission.spec_id,
        spec_version: &admission.spec_version,
        spec_hash_algorithm: &admission.spec_hash_algorithm,
        spec_hash: &admission.spec_hash,
        validator_kind: &admission.validator_kind,
        validator_ref: &admission.validator_ref,
        artifact_kind: &admission.artifact_kind,
        artifact_ref: &admission.artifact_ref,
        artifact_digest: admission.artifact_digest.as_deref(),
        target_identity: admission.target_identity.as_deref(),
        validation_result: &admission.validation_result,
        no_hidden_fallback: admission.no_hidden_fallback,
    };
    let bytes = serde_json::to_vec(&material).map_err(|error| error.to_string())?;
    Ok(domain_hash(SOURCE_ADMISSION_HASH_DOMAIN, &[&bytes]))
}

pub fn reduce_source_admission(admission: &SourceArtifactAdmission) -> Result<ProjectionObjectAdmission, String> {
    Ok(ProjectionObjectAdmission {
        spec_id: admission.spec_id.clone(),
        spec_version: admission.spec_version.clone(),
        spec_hash_algorithm: admission.spec_hash_algorithm.clone(),
        spec_hash: admission.spec_hash.clone(),
        validator_kind: admission.validator_kind.clone(),
        validator_ref: admission.validator_ref.clone(),
        artifact_kind: admission.artifact_kind.clone(),
        artifact_ref: admission.artifact_ref.clone(),
        artifact_digest: admission.artifact_digest.clone(),
        target_identity: admission.target_identity.clone(),
        source_attestation_blake3: source_admission_identity(admission)?,
        validation_result: admission.validation_result.clone(),
        no_hidden_fallback: admission.no_hidden_fallback,
    })
}

pub fn validate_source_admissions(projection: &OciProjection, bundle: &SourceAdmissionBundle) -> Vec<ProjectionIssue> {
    let mut issues = Vec::new();
    if bundle.schema != SOURCE_ADMISSION_BUNDLE_SCHEMA {
        push(
            &mut issues,
            issue(
                "unknown-admission-bundle-schema",
                "source_admissions.schema",
                "source admission bundle schema is unsupported",
            ),
        );
    }
    let source_refs = bundle.admissions.iter().map(|admission| admission.artifact_ref.as_str()).collect::<Vec<_>>();
    if source_refs.is_empty() || !source_refs.windows(PAIR_WINDOW).all(|pair| pair[0] < pair[1]) {
        push(
            &mut issues,
            issue(
                "non-canonical-source-admission-order",
                "source_admissions.admissions",
                "source admissions must be non-empty and strictly ordered by artifact ref",
            ),
        );
    }
    let projected = projection
        .object_admissions
        .iter()
        .map(|admission| (admission.artifact_ref.as_str(), admission))
        .collect::<BTreeMap<_, _>>();
    let mut source_set = BTreeSet::new();
    for (index, admission) in bundle.admissions.iter().enumerate() {
        let path = format!("source_admissions.admissions[{index}]");
        let build_root_is_safe = safe_text(&admission.build_root);
        if !build_root_is_safe || !source_set.insert(admission.artifact_ref.as_str()) {
            push(
                &mut issues,
                issue(
                    "invalid-source-admission",
                    &path,
                    "source admission build root is invalid or artifact ref is duplicated",
                ),
            );
            continue;
        }
        match (reduce_source_admission(admission), projected.get(admission.artifact_ref.as_str())) {
            (Ok(reduced), Some(expected)) if &reduced == *expected => {}
            _ => push(
                &mut issues,
                issue(
                    "source-admission-mismatch",
                    path,
                    "source admission does not exactly reproduce its path-free projection reduction",
                ),
            ),
        }
    }
    let projected_set = projected.keys().copied().collect::<BTreeSet<_>>();
    if source_set != projected_set {
        push(
            &mut issues,
            issue(
                "source-admission-set-mismatch",
                "source_admissions.admissions",
                "source admissions must exactly cover projected object admissions",
            ),
        );
    }
    issues
}

pub fn validate_projection_structure(projection: &OciProjection) -> Vec<ProjectionIssue> {
    let mut issues = Vec::new();
    if projection.schema != OCI_PROJECTION_SCHEMA {
        push(&mut issues, issue("unknown-schema", "schema", "projection schema is unsupported"));
    }
    if projection.frontend_spec.hash_algorithm != "blake3" || !is_blake3_hex(&projection.frontend_spec.hash) {
        push(
            &mut issues,
            issue(
                "spec-hash-mismatch",
                "frontend_spec.hash",
                "frontend spec binding must use a lowercase BLAKE3 digest",
            ),
        );
    }
    if !safe_text(&projection.frontend_spec.id) || !safe_text(&projection.frontend_spec.version) {
        push(
            &mut issues,
            issue("invalid-spec-binding", "frontend_spec", "spec id and version must be bounded safe text"),
        );
    }
    if projection.admission.validation_result != STATUS_ADMITTED || !projection.admission.no_hidden_fallback {
        push(
            &mut issues,
            issue("frontend-not-admitted", "admission", "frontend admission and no-hidden-fallback are required"),
        );
    }
    match (projection_identity(projection), projection.admission.projection_blake3.as_deref()) {
        (Ok(actual), Some(declared)) if actual == declared => {}
        _ => push(
            &mut issues,
            issue(
                "projection-hash-mismatch",
                "admission.projection_blake3",
                "declared projection BLAKE3 does not match canonical projection",
            ),
        ),
    }
    validate_bounds(&projection.bounds, &mut issues);
    validate_archive_policy(&projection.archive_policy, &mut issues);
    if !safe_text(&projection.platform.architecture) || !safe_text(&projection.platform.os) {
        push(&mut issues, issue("invalid-platform", "platform", "OCI platform values must be bounded safe text"));
    }
    if projection.layers.is_empty() || projection.layers.len() > projection.bounds.max_layers {
        push(
            &mut issues,
            issue("invalid-layer-count", "layers", "layer count is empty or exceeds the admitted bound"),
        );
    }
    if projection.required_annotations.len() > MAX_ANNOTATIONS {
        push(&mut issues, issue("annotation-bound", "required_annotations", "too many annotations"));
    }
    validate_annotations(&projection.required_annotations, "required_annotations", &mut issues);
    let projected_refs = projection
        .layers
        .iter()
        .flat_map(|layer| layer.entries.iter())
        .map(|entry| entry.object_ref.as_str())
        .collect::<BTreeSet<_>>();
    let mut projected_identities = BTreeMap::<&str, BTreeSet<&str>>::new();
    for entry in projection.layers.iter().flat_map(|layer| layer.entries.iter()) {
        projected_identities.entry(entry.object_ref.as_str()).or_default().insert(entry.identity.as_str());
    }
    let projected_identity_set = projected_identities
        .values()
        .flat_map(|identities| identities.iter().copied())
        .collect::<BTreeSet<_>>();
    let expected_digest_subjects = projection
        .expected_external_digests
        .iter()
        .map(|expectation| expectation.subject_identity.as_str())
        .collect::<Vec<_>>();
    if !expected_digest_subjects.windows(PAIR_WINDOW).all(|pair| pair[0] < pair[1]) {
        push(
            &mut issues,
            issue(
                "non-canonical-external-digest-order",
                "expected_external_digests",
                "external digest expectations must be strictly ordered by subject identity",
            ),
        );
    }
    for (index, expectation) in projection.expected_external_digests.iter().enumerate() {
        if expectation.role != EXPECTED_DIGEST_ROLE_OCI_LAYER_BLOB
            || !safe_text(&expectation.subject_identity)
            || !projected_identity_set.contains(expectation.subject_identity.as_str())
            || !is_sha256_digest(&expectation.digest)
        {
            push(
                &mut issues,
                issue(
                    "invalid-external-digest",
                    format!("expected_external_digests[{index}]"),
                    "external digest expectations must bind a projected identity to an OCI layer SHA-256 role",
                ),
            );
        }
    }
    let admission_refs = projection
        .object_admissions
        .iter()
        .map(|admission| admission.artifact_ref.as_str())
        .collect::<Vec<_>>();
    if !admission_refs.windows(PAIR_WINDOW).all(|pair| pair[0] < pair[1]) {
        push(
            &mut issues,
            issue(
                "non-canonical-admission-order",
                "object_admissions",
                "object admissions must be strictly ordered by artifact ref",
            ),
        );
    }
    let mut admitted_refs = BTreeSet::new();
    for (index, admission) in projection.object_admissions.iter().enumerate() {
        let path = format!("object_admissions[{index}]");
        let expected_digest = artifact_ref_digest(&admission.artifact_ref).map(|digest| format!("blake3:{digest}"));
        let digest_matches = admission.artifact_digest.as_ref() == expected_digest.as_ref();
        let spec_matches = admission.spec_id == projection.frontend_spec.id
            && admission.spec_version == projection.frontend_spec.version
            && admission.spec_hash_algorithm == projection.frontend_spec.hash_algorithm
            && admission.spec_hash == projection.frontend_spec.hash;
        let target_matches = admission.target_identity.as_deref().is_some_and(|identity| {
            projected_identities
                .get(admission.artifact_ref.as_str())
                .is_some_and(|identities| identities.contains(identity))
        });
        if admission.validation_result != STATUS_ADMITTED
            || !admission.no_hidden_fallback
            || canonical_mantle_ref(&admission.artifact_ref).as_deref() != Some(admission.artifact_ref.as_str())
            || !digest_matches
            || !spec_matches
            || !target_matches
            || !safe_text(&admission.validator_kind)
            || !safe_text(&admission.validator_ref)
            || contains_sensitive_fragment(&admission.validator_ref)
            || admission.validator_ref.starts_with('/')
            || admission.validator_ref.starts_with("file://")
            || !safe_text(&admission.artifact_kind)
            || !is_blake3_hex(&admission.source_attestation_blake3)
            || !admitted_refs.insert(admission.artifact_ref.as_str())
        {
            push(
                &mut issues,
                issue(
                    "invalid-object-admission",
                    path,
                    "object admission must uniquely bind the exact spec, Mantle ref, BLAKE3 digest, validator, and no-fallback result",
                ),
            );
        }
    }
    if admitted_refs != projected_refs {
        push(
            &mut issues,
            issue(
                "object-admission-set-mismatch",
                "object_admissions",
                "object admissions must exactly cover projected object refs",
            ),
        );
    }
    let mut roles = BTreeSet::new();
    let mut total_declared = 0_u64;
    let layer_roles = projection.layers.iter().map(|layer| layer.role.as_str()).collect::<Vec<_>>();
    if !layer_roles.windows(PAIR_WINDOW).all(|pair| pair[0] < pair[1]) {
        push(&mut issues, issue("non-canonical-layer-order", "layers", "layers must be strictly ordered by role"));
    }
    for (layer_index, layer) in projection.layers.iter().enumerate() {
        let layer_path = format!("layers[{layer_index}]");
        if !safe_text(&layer.role) || !roles.insert(layer.role.clone()) {
            push(
                &mut issues,
                issue("duplicate-or-invalid-role", &layer_path, "layer roles must be unique bounded text"),
            );
        }
        if !valid_media_type(&layer.media_type) {
            push(
                &mut issues,
                issue("invalid-media-type", format!("{layer_path}.media_type"), "layer media type is invalid"),
            );
        }
        let tar_media = layer.media_type.ends_with(".tar") || layer.media_type.ends_with("+gzip");
        let archive_media_matches = match projection.archive_policy.compression {
            CompressionProfile::None => layer.media_type.ends_with(".tar"),
            CompressionProfile::GzipDeterministicV1 => layer.media_type.ends_with("+gzip"),
        };
        if (layer.mode == LayerMode::CanonicalArchive && !archive_media_matches)
            || (layer.mode == LayerMode::ExactBlob && tar_media)
        {
            push(
                &mut issues,
                issue(
                    "layer-mode-media-mismatch",
                    format!("{layer_path}.media_type"),
                    "layer mode, archive compression, and media type disagree",
                ),
            );
        }
        if layer.annotations.len() > MAX_ANNOTATIONS {
            push(
                &mut issues,
                issue("annotation-bound", format!("{layer_path}.annotations"), "too many layer annotations"),
            );
        }
        validate_annotations(&layer.annotations, &format!("{layer_path}.annotations"), &mut issues);
        if layer.entries.is_empty() || layer.entries.len() > projection.bounds.max_entries_per_layer {
            push(
                &mut issues,
                issue(
                    "invalid-entry-count",
                    format!("{layer_path}.entries"),
                    "layer entry count is empty or exceeds the admitted bound",
                ),
            );
        }
        if layer.mode == LayerMode::ExactBlob && layer.entries.len() != 1 {
            push(
                &mut issues,
                issue(
                    "exact-blob-cardinality",
                    format!("{layer_path}.entries"),
                    "exact blob layers require exactly one entry",
                ),
            );
        }
        let mut paths = BTreeSet::new();
        let entry_paths = layer.entries.iter().map(|entry| entry.relative_path.as_str()).collect::<Vec<_>>();
        if !entry_paths.windows(PAIR_WINDOW).all(|pair| pair[0] < pair[1]) {
            push(
                &mut issues,
                issue(
                    "non-canonical-entry-order",
                    format!("{layer_path}.entries"),
                    "entries must be strictly ordered by relative path",
                ),
            );
        }
        for (entry_index, entry) in layer.entries.iter().enumerate() {
            let entry_path = format!("{layer_path}.entries[{entry_index}]");
            if !safe_relative_path(&entry.relative_path, projection.bounds.max_depth)
                || !paths.insert(entry.relative_path.clone())
            {
                push(
                    &mut issues,
                    issue("duplicate-or-unsafe-path", entry_path.clone(), "entry path is duplicate or unsafe"),
                );
            }
            if canonical_mantle_ref(&entry.object_ref).as_deref() != Some(entry.object_ref.as_str()) {
                push(
                    &mut issues,
                    issue(
                        "unsafe-object-ref",
                        format!("{entry_path}.object_ref"),
                        "object ref must use mantle://blake3/<lowercase-hex>",
                    ),
                );
            }
            if !safe_text(&entry.identity) {
                push(
                    &mut issues,
                    issue(
                        "invalid-frontend-identity",
                        format!("{entry_path}.identity"),
                        "frontend identity must be bounded safe text",
                    ),
                );
            }
            total_declared = total_declared.saturating_add(entry.size_bytes);
        }
    }
    if total_declared > projection.bounds.max_total_bytes {
        push(
            &mut issues,
            issue("declared-size-bound", "layers", "declared source bytes exceed the admitted total bound"),
        );
    }
    let round_trip_identities = std::iter::once(projection.round_trip.kernel_build_identity.as_str())
        .chain(std::iter::once(projection.round_trip.bundle_identity.as_str()))
        .chain(std::iter::once(projection.round_trip.manifest_identity.as_str()))
        .chain(projection.round_trip.component_identities.iter().map(String::as_str))
        .chain(projection.round_trip.pack_identities.iter().map(String::as_str))
        .collect::<Vec<_>>();
    if round_trip_identities.iter().any(|identity| !safe_text(identity)) {
        push(
            &mut issues,
            issue("unsafe-round-trip-identity", "round_trip", "round-trip identities must be bounded safe text"),
        );
    }
    if !projection.round_trip.component_identities.windows(PAIR_WINDOW).all(|pair| pair[0] < pair[1])
        || !projection.round_trip.pack_identities.windows(PAIR_WINDOW).all(|pair| pair[0] < pair[1])
        || !projection.non_claims.windows(PAIR_WINDOW).all(|pair| pair[0] < pair[1])
    {
        push(
            &mut issues,
            issue("non-canonical-list-order", "round_trip", "identity and non-claim lists must be sorted and unique"),
        );
    }
    for required in REQUIRED_NON_CLAIMS {
        if !projection.non_claims.iter().any(|claim| claim == required) {
            push(&mut issues, issue("missing-non-claim", "non_claims", required));
        }
    }
    if projection.non_claims.iter().any(|claim| !safe_text(claim) || contains_overclaim(claim)) {
        push(
            &mut issues,
            issue("unsafe-non-claim", "non_claims", "non-claims must be bounded, safe, and non-promotional text"),
        );
    }
    issues
}

pub fn validate_projection(projection: &OciProjection, spec_material: &[u8]) -> Vec<ProjectionIssue> {
    let mut issues = validate_projection_structure(projection);
    if blake3_hex(spec_material) != projection.frontend_spec.hash {
        push(
            &mut issues,
            issue(
                "spec-hash-mismatch",
                "frontend_spec.hash",
                "frontend spec material does not match the admitted BLAKE3",
            ),
        );
    }
    issues
}

pub fn build_export_plan(
    projection: &OciProjection,
    spec_material: &[u8],
    source_admissions: &SourceAdmissionBundle,
    objects: &BTreeMap<String, MaterializedObject>,
) -> Result<ExportPlan, Vec<ProjectionIssue>> {
    let mut issues = validate_projection(projection, spec_material);
    issues.extend(validate_source_admissions(projection, source_admissions));
    let expected_refs = projection
        .layers
        .iter()
        .flat_map(|layer| layer.entries.iter())
        .map(|entry| entry.object_ref.clone())
        .collect::<BTreeSet<_>>();
    for expected in &expected_refs {
        if !objects.contains_key(expected) {
            push(&mut issues, issue("missing-object", expected, "admitted object was not materialized"));
        }
    }
    if objects.keys().cloned().collect::<BTreeSet<_>>() != expected_refs {
        push(
            &mut issues,
            issue("object-set-mismatch", "objects", "materialized objects must exactly equal projected refs"),
        );
    }
    for (key, object) in objects {
        if key != &object.artifact_ref || canonical_mantle_ref(key).as_deref() != Some(key.as_str()) {
            push(&mut issues, issue("object-identity-mismatch", key, "materialized object key and identity disagree"));
        }
    }
    if !issues.is_empty() {
        return Err(issues);
    }
    let plan = layout::build_layout(projection, objects).map_err(|error| vec![error])?;
    for (index, expectation) in projection.expected_external_digests.iter().enumerate() {
        let matching_layers = plan
            .layers
            .iter()
            .filter(|layer| layer.source_identities.contains(&expectation.subject_identity))
            .collect::<Vec<_>>();
        if matching_layers.len() != 1 || matching_layers[0].blob_sha256 != expectation.digest {
            push(
                &mut issues,
                issue(
                    "external-digest-mismatch",
                    format!("expected_external_digests[{index}]"),
                    "expected OCI layer SHA-256 does not match the planned layer bytes",
                ),
            );
        }
    }
    if issues.is_empty() { Ok(plan) } else { Err(issues) }
}

pub fn export_report(plan: &ExportPlan) -> Result<OciExportReport, String> {
    let admission_bytes = serde_json::to_vec(&plan.projection.object_admissions).map_err(|error| error.to_string())?;
    let object_admissions_blake3 = domain_hash(OBJECT_ADMISSION_HASH_DOMAIN, &[&admission_bytes]);
    let mut non_claims = REQUIRED_NON_CLAIMS.iter().map(ToString::to_string).collect::<Vec<_>>();
    non_claims.sort();
    let mut report = OciExportReport {
        schema: OCI_EXPORT_REPORT_SCHEMA.to_string(),
        schema_version: SCHEMA_VERSION,
        exported: true,
        frontend_spec: plan.projection.frontend_spec.clone(),
        admission_count: plan.projection.object_admissions.len(),
        object_admissions_blake3,
        no_hidden_fallback: plan.projection.admission.no_hidden_fallback,
        projection_blake3: plan.projection_blake3.clone(),
        projection: plan.projection.clone(),
        layout_blake3: plan.layout_blake3.clone(),
        manifest_descriptor: plan.manifest_descriptor.clone(),
        config_descriptor: plan.config_descriptor.clone(),
        layers: plan.layers.clone(),
        round_trip: plan.projection.round_trip.clone(),
        issues: Vec::new(),
        non_claims,
        receipt_blake3: String::new(),
    };
    report.receipt_blake3 = report_identity(EXPORT_HASH_DOMAIN, &report)?;
    Ok(report)
}

pub fn report_identity<T>(domain: &str, report: &T) -> Result<String, String>
where T: Serialize {
    let bytes = serde_json::to_vec(report).map_err(|error| error.to_string())?;
    Ok(domain_hash(domain, &[&bytes]))
}

pub fn verify_export_report(report: &OciExportReport) -> Result<(), ProjectionIssue> {
    let admission_bytes = serde_json::to_vec(&report.projection.object_admissions)
        .map_err(|error| issue("report-serialization", OCI_EXPORT_REPORT_FILENAME, &error.to_string()))?;
    let admissions_identity = domain_hash(OBJECT_ADMISSION_HASH_DOMAIN, &[&admission_bytes]);
    let projection_material_valid = validate_projection_structure(&report.projection).is_empty()
        && projection_identity(&report.projection).is_ok_and(|identity| identity == report.projection_blake3)
        && report.projection.schema == OCI_PROJECTION_SCHEMA
        && report.projection.frontend_spec == report.frontend_spec
        && report.projection.admission.projection_blake3.as_deref() == Some(report.projection_blake3.as_str())
        && report.projection.admission.no_hidden_fallback == report.no_hidden_fallback
        && report.projection.object_admissions.len() == report.admission_count
        && admissions_identity == report.object_admissions_blake3
        && report.projection.round_trip == report.round_trip
        && report.projection.layers.len() == report.layers.len()
        && report.projection.layers.iter().zip(&report.layers).all(|(projection_layer, report_layer)| {
            let expected_archive = projection_layer.mode == LayerMode::CanonicalArchive;
            let expected_compression = if expected_archive {
                report.projection.archive_policy.compression
            } else {
                CompressionProfile::None
            };
            projection_layer.role == report_layer.role
                && projection_layer.media_type == report_layer.media_type
                && projection_layer
                    .entries
                    .iter()
                    .map(|entry| entry.object_ref.as_str())
                    .eq(report_layer.source_object_refs.iter().map(String::as_str))
                && projection_layer
                    .entries
                    .iter()
                    .map(|entry| entry.identity.as_str())
                    .eq(report_layer.source_identities.iter().map(String::as_str))
                && expected_archive == report_layer.canonical_archive
                && expected_compression == report_layer.compression
        })
        && report.projection.expected_external_digests.iter().all(|expectation| {
            let matching_layers = report
                .layers
                .iter()
                .filter(|layer| layer.source_identities.contains(&expectation.subject_identity))
                .collect::<Vec<_>>();
            matching_layers.len() == 1 && matching_layers[0].blob_sha256 == expectation.digest
        });
    let layer_fields_valid = !report.layers.is_empty()
        && report.layers.iter().all(|layer| {
            safe_text(&layer.role)
                && valid_media_type(&layer.media_type)
                && is_sha256_digest(&layer.uncompressed_sha256)
                && is_sha256_digest(&layer.blob_sha256)
                && is_blake3_hex(&layer.blob_blake3)
                && !layer.source_object_refs.is_empty()
                && layer.source_object_refs.len() == layer.source_identities.len()
                && layer
                    .source_object_refs
                    .iter()
                    .all(|value| canonical_mantle_ref(value).as_deref() == Some(value.as_str()))
                && layer.source_identities.iter().all(|value| safe_text(value))
        });
    let non_claims_valid =
        REQUIRED_NON_CLAIMS.iter().all(|required| report.non_claims.iter().any(|value| value == required))
            && report.non_claims.iter().all(|value| safe_text(value) && !contains_overclaim(value));
    if report.schema != OCI_EXPORT_REPORT_SCHEMA
        || report.schema_version != SCHEMA_VERSION
        || !report.exported
        || report.frontend_spec.hash_algorithm != "blake3"
        || !is_blake3_hex(&report.frontend_spec.hash)
        || !is_blake3_hex(&report.projection_blake3)
        || !is_blake3_hex(&report.layout_blake3)
        || report.admission_count == 0
        || !is_blake3_hex(&report.object_admissions_blake3)
        || !report.no_hidden_fallback
        || !projection_material_valid
        || !is_sha256_digest(&report.manifest_descriptor.digest)
        || !is_sha256_digest(&report.config_descriptor.digest)
        || !layer_fields_valid
        || !non_claims_valid
        || !report.issues.is_empty()
    {
        return Err(issue(
            "invalid-export-report",
            OCI_EXPORT_REPORT_FILENAME,
            "export report is not a successful Mantle report",
        ));
    }
    let mut material = report.clone();
    let declared = std::mem::take(&mut material.receipt_blake3);
    let actual = report_identity(EXPORT_HASH_DOMAIN, &material)
        .map_err(|error| issue("report-serialization", OCI_EXPORT_REPORT_FILENAME, &error))?;
    if declared != actual {
        return Err(issue("export-receipt-mismatch", OCI_EXPORT_REPORT_FILENAME, "export receipt BLAKE3 is invalid"));
    }
    Ok(())
}

pub fn validate_import(facts: &LayoutFacts) -> Result<OciImportReport, Vec<ProjectionIssue>> {
    layout::validate_layout(facts)
}

pub fn attach_imported_refs(
    report: &mut OciImportReport,
    refs_by_sha256: &BTreeMap<String, String>,
) -> Result<(), String> {
    let admitted_refs = report
        .objects
        .iter()
        .map(|object| {
            refs_by_sha256
                .get(&object.oci_sha256)
                .cloned()
                .ok_or_else(|| format!("missing imported ref for {}", object.oci_sha256))
        })
        .collect::<Result<Vec<_>, _>>()?;
    for (object, artifact_ref) in report.objects.iter_mut().zip(admitted_refs) {
        object.artifact_ref = Some(artifact_ref);
    }
    report.imported = true;
    report.receipt_blake3.clear();
    report.receipt_blake3 = report_identity(IMPORT_HASH_DOMAIN, report)?;
    Ok(())
}

pub fn verify_import_report(report: &OciImportReport) -> Result<(), String> {
    let state_fields_valid = if report.state == STATUS_ADMITTED {
        report.frontend_spec.is_some()
            && report.object_admissions_blake3.as_deref().is_some_and(is_blake3_hex)
            && report.projection_blake3.as_deref().is_some_and(is_blake3_hex)
            && report.round_trip.is_some()
    } else if report.state == STATUS_COMPATIBILITY_ONLY {
        report.frontend_spec.is_none()
            && report.object_admissions_blake3.is_none()
            && report.projection_blake3.is_none()
            && report.round_trip.is_none()
    } else {
        false
    };
    let objects_valid = !report.objects.is_empty()
        && report.objects.iter().all(|object| {
            safe_text(&object.role)
                && valid_media_type(&object.media_type)
                && is_sha256_digest(&object.oci_sha256)
                && is_blake3_hex(&object.blob_blake3)
                && object
                    .artifact_ref
                    .as_deref()
                    .is_some_and(|value| canonical_mantle_ref(value).as_deref() == Some(value))
        });
    let non_claims_valid =
        REQUIRED_NON_CLAIMS.iter().all(|required| report.non_claims.iter().any(|value| value == required))
            && report.non_claims.iter().all(|value| safe_text(value) && !contains_overclaim(value));
    if report.schema != OCI_IMPORT_REPORT_SCHEMA
        || report.schema_version != SCHEMA_VERSION
        || !report.imported
        || !is_blake3_hex(&report.layout_blake3)
        || report.manifest_descriptor.as_ref().is_none_or(|descriptor| !is_sha256_digest(&descriptor.digest))
        || !state_fields_valid
        || !objects_valid
        || !non_claims_valid
        || !report.issues.is_empty()
    {
        return Err("import report structure or state fields are invalid".to_string());
    }
    let mut material = report.clone();
    let declared = std::mem::take(&mut material.receipt_blake3);
    let actual = report_identity(IMPORT_HASH_DOMAIN, &material)?;
    if declared != actual {
        return Err("import report receipt BLAKE3 is invalid".to_string());
    }
    Ok(())
}
