//! Pure OCI Distribution planning, metadata linkage, and registry receipts.
//!
//! r[impl kernel_bundle_oci.registry_transport]
//! r[impl kernel_bundle_oci.registry_admission]
//! r[impl kernel_bundle_oci.registry_receipts]

// machine-artifact-public: oci.registry-push-report
// machine-artifact-public: oci.registry-pull-report

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use nix_compat::narinfo::SignatureRef;
use nix_compat::narinfo::VerifyingKey;
use serde::Deserialize;
use serde::Serialize;
use url::Url;

use crate::oci_projection::LayoutFacts;
use crate::oci_projection::OCI_LAYER_MAX_COUNT;
use crate::oci_projection::OCI_MANIFEST_MEDIA_TYPE;
use crate::oci_projection::OciDescriptor;
use crate::oci_projection::OciExportReport;
use crate::oci_projection::OciImportReport;
use crate::oci_projection::OciIndexDocument;
use crate::oci_projection::OciManifestDocument;
use crate::oci_projection::is_blake3_hex;
use crate::oci_projection::is_sha256_digest;
use crate::oci_projection::sha256_digest;
use crate::oci_projection::validate_import;
use crate::oci_projection::verify_export_report;
use crate::oci_projection::verify_import_report;

pub const OCI_REGISTRY_PUSH_REPORT_SCHEMA: &str = "mantle-oci-registry-push-report-v2";
pub const OCI_REGISTRY_PULL_REPORT_SCHEMA: &str = "mantle-oci-registry-pull-report-v2";
pub const OCI_REGISTRY_TRUST_POLICY_SCHEMA: &str = "mantle-oci-registry-trust-policy-v1";
pub const OCI_REGISTRY_SIGNATURE_DOCUMENT_SCHEMA: &str = "mantle-oci-registry-signature-document-v1";
pub const OCI_REGISTRY_TRUST_STATEMENT_SCHEMA: &str = "mantle-oci-registry-trust-statement-v1";
pub const OCI_EMPTY_CONFIG_MEDIA_TYPE: &str = "application/vnd.oci.empty.v1+json";
pub const MANTLE_METADATA_ARTIFACT_TYPE: &str = "application/vnd.mantle.oci-metadata.v1";
pub const MANTLE_SIGNATURE_ARTIFACT_TYPE: &str = "application/vnd.mantle.oci-signature.v1";
pub const MANTLE_SIGNATURE_DOCUMENT_MEDIA_TYPE: &str = "application/vnd.mantle.oci-signature.v1+json";
pub const MANTLE_OCI_LAYOUT_MEDIA_TYPE: &str = "application/vnd.mantle.oci-layout.v1+json";
pub const MANTLE_OCI_INDEX_MEDIA_TYPE: &str = "application/vnd.mantle.oci-index.v1+json";
pub const MANTLE_OCI_EXPORT_REPORT_MEDIA_TYPE: &str = "application/vnd.mantle.oci-export-report.v1+json";
pub const METADATA_ROLE_ANNOTATION: &str = "org.mantle.metadata.role";
pub const METADATA_LAYOUT_ROLE: &str = "oci-layout";
pub const METADATA_INDEX_ROLE: &str = "index";
pub const METADATA_EXPORT_REPORT_ROLE: &str = "export-report";
pub const METADATA_REFERENCE_SUFFIX: &str = ".mantle-metadata";
pub const SIGNATURE_REFERENCE_SUFFIX: &str = ".mantle-signature";
pub const CREDENTIAL_MODE_ANONYMOUS: &str = "anonymous";
pub const CREDENTIAL_MODE_BEARER_FILE: &str = "explicit-bearer-file";

const REGISTRY_SCHEMA_VERSION: u16 = 2;
const TRUST_POLICY_SCHEMA_VERSION: u16 = 1;
const TRUST_STATEMENT_SCHEMA_VERSION: u16 = 1;
const SIGNATURE_DOCUMENT_SCHEMA_VERSION: u16 = 1;
const OCI_DOCUMENT_SCHEMA_VERSION: u16 = 2;
const REGISTRY_BYTES_MAX: usize = 2_048;
const REPOSITORY_BYTES_MAX: usize = 255;
const REPOSITORY_SEGMENT_BYTES_MAX: usize = 128;
const TAG_BYTES_MAX: usize = 128;
const TAG_BYTES_MIN: usize = 1;
const TRUST_DOMAIN_BYTES_MAX: usize = 128;
const SIGNER_NAME_BYTES_MAX: usize = 255;
const TRUST_POLICY_ENTRY_COUNT_MAX: usize = 64;
const SIGNATURE_COUNT_MAX: usize = 64;
const METADATA_BLOB_COUNT: usize = 3;
const SIGNATURE_BLOB_COUNT: usize = 1;
const REGISTRY_RECEIPT_BLOB_COUNT_MAX: u32 = 1_000_000;
const REGISTRY_RECEIPT_OBJECT_COUNT_MAX: u32 = 1_000_000;
const REGISTRY_RECEIPT_TRANSFER_BYTES_MAX: u64 = 1_099_511_627_776;
const REGISTRY_PUSH_RECEIPT_DOMAIN: &str = "mantle/oci/registry-push-report/v2";
const REGISTRY_PULL_RECEIPT_DOMAIN: &str = "mantle/oci/registry-pull-report/v2";
const TRUST_POLICY_DIGEST_DOMAIN: &str = "mantle/oci/registry-trust-policy/v1";
const PUBLIC_KEY_DIGEST_DOMAIN: &str = "mantle/oci/registry-public-key/v1";
const SIGNATURE_SUITE: &str = "ed25519-detached-v1";
const LAYOUT_DIGEST_ANNOTATION: &str = "org.mantle.layout.blake3";
const PROJECTION_DIGEST_ANNOTATION: &str = "org.mantle.projection.blake3";
const SIGNATURE_ROLE_ANNOTATION: &str = "org.mantle.signature.role";
const SIGNATURE_DOCUMENT_ROLE: &str = "signature-document";
const SIGNATURE_METADATA_DIGEST_ANNOTATION: &str = "org.mantle.signature.metadata-digest";
const SIGNATURE_TRUST_DOMAIN_ANNOTATION: &str = "org.mantle.signature.trust-domain";
const OCI_EMPTY_CONFIG_BYTES: &[u8] = b"{}";

const REQUIRED_NON_CLAIMS: &[&str] = &[
    "credential possession is not authorization proof",
    "no bootability claim",
    "no deployability claim",
    "no exactly-once publication",
    "no kernel compatibility decision",
    "no registry authorization claim",
    "no release eligibility claim",
    "no revocation freshness or transparency claim",
    "no tag immutability claim",
    "no upload resumption",
    "signature verification authenticates only the immutable digest pair under supplied local policy",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegistryTarget {
    pub registry: String,
    pub repository: String,
    pub reference: String,
    pub metadata_reference: String,
    pub signature_reference: String,
}

pub struct RegistryTargetInput<'a> {
    pub registry: &'a str,
    pub repository: &'a str,
    pub reference: &'a str,
    pub allow_http: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OciCompanionManifest {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u16,
    #[serde(rename = "mediaType")]
    pub media_type: String,
    #[serde(rename = "artifactType")]
    pub artifact_type: String,
    pub config: OciDescriptor,
    pub layers: Vec<OciDescriptor>,
    pub subject: OciDescriptor,
    pub annotations: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegistryTrustPolicy {
    pub schema: String,
    pub schema_version: u16,
    pub trust_domain: String,
    pub allowed_repositories: Vec<String>,
    pub trusted_public_keys: Vec<String>,
    pub required_signers: Vec<String>,
    pub minimum_signatures: u16,
    pub revoked_public_key_blake3: Vec<String>,
}

pub struct ValidatedRegistryTrustPolicy {
    pub policy: RegistryTrustPolicy,
    pub policy_blake3: String,
    verifying_keys: Vec<VerifyingKey>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegistryTrustStatement {
    pub schema: String,
    pub schema_version: u16,
    pub signature_suite: String,
    pub trust_domain: String,
    pub manifest_digest: String,
    pub metadata_manifest_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegistryDetachedSignature {
    pub signer: String,
    pub signature: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegistrySignatureDocument {
    pub schema: String,
    pub schema_version: u16,
    pub statement: RegistryTrustStatement,
    pub signatures: Vec<RegistryDetachedSignature>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegistryTrustVerification {
    pub trust_domain: String,
    pub policy_blake3: String,
    pub verified_signers: Vec<String>,
    pub verified_public_key_blake3: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegistrySignaturePlan {
    pub document: RegistrySignatureDocument,
    pub document_descriptor: OciDescriptor,
    pub document_bytes: Vec<u8>,
    pub manifest: OciCompanionManifest,
    pub manifest_descriptor: OciDescriptor,
    pub manifest_bytes: Vec<u8>,
    pub verification: RegistryTrustVerification,
}

pub struct RegistrySignatureVerificationInput<'a> {
    pub bytes: &'a [u8],
    pub manifest_digest: &'a str,
    pub metadata_manifest_digest: &'a str,
    pub policy: &'a ValidatedRegistryTrustPolicy,
}

struct TrustStatementExpectation<'a> {
    manifest_digest: &'a str,
    metadata_manifest_digest: &'a str,
    policy: &'a ValidatedRegistryTrustPolicy,
}

struct SignatureManifestInput<'a> {
    main: &'a OciDescriptor,
    metadata_digest: &'a str,
    trust_domain: &'a str,
    document_descriptor: &'a OciDescriptor,
}

struct ReportTrustEvidenceInput<'a> {
    trust_domain: &'a str,
    policy_blake3: &'a str,
    verified_signers: &'a [String],
    verified_public_key_blake3: &'a [String],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegistryBlob {
    pub digest: String,
    pub bytes: Vec<u8>,
}

pub struct RegistryLayoutInput {
    pub oci_layout_bytes: Vec<u8>,
    pub index_bytes: Vec<u8>,
    pub descriptor_blobs: BTreeMap<String, Vec<u8>>,
    pub export_report_bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegistryPushPlan {
    pub main_manifest_descriptor: OciDescriptor,
    pub main_manifest_bytes: Vec<u8>,
    pub metadata_manifest_descriptor: OciDescriptor,
    pub metadata_manifest_bytes: Vec<u8>,
    pub blobs: Vec<RegistryBlob>,
    pub layout_blake3: String,
    pub projection_blake3: String,
    pub export_receipt_blake3: String,
}

pub struct RegistryPullInput {
    pub expected_manifest_digest: String,
    pub main_manifest_media_type: String,
    pub main_manifest_bytes: Vec<u8>,
    pub metadata_manifest_digest: String,
    pub metadata_manifest_bytes: Vec<u8>,
    pub downloaded_blobs: BTreeMap<String, Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegistryPullPlan {
    pub main_manifest_descriptor: OciDescriptor,
    pub metadata_manifest_descriptor: OciDescriptor,
    pub oci_layout_bytes: Vec<u8>,
    pub index_bytes: Vec<u8>,
    pub descriptor_blobs: BTreeMap<String, Vec<u8>>,
    pub export_report_bytes: Vec<u8>,
    pub layout_blake3: String,
    pub projection_blake3: String,
    pub export_receipt_blake3: String,
    pub import_preview: OciImportReport,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OciRegistryPushReport {
    pub schema: String,
    pub schema_version: u16,
    pub published: bool,
    pub registry: String,
    pub repository: String,
    pub reference: String,
    pub metadata_reference: String,
    pub signature_reference: String,
    pub manifest_digest: String,
    pub metadata_manifest_digest: String,
    pub signature_manifest_digest: String,
    pub trust_domain: String,
    pub policy_blake3: String,
    pub verified_signers: Vec<String>,
    pub verified_public_key_blake3: Vec<String>,
    pub layout_blake3: String,
    pub projection_blake3: String,
    pub export_receipt_blake3: String,
    pub uploaded_blobs: u32,
    pub reused_blobs: u32,
    pub transferred_bytes: u64,
    pub credential_mode: String,
    pub non_claims: Vec<String>,
    pub receipt_blake3: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OciRegistryPullReport {
    pub schema: String,
    pub schema_version: u16,
    pub pulled: bool,
    pub registry: String,
    pub repository: String,
    pub reference: String,
    pub metadata_reference: String,
    pub signature_reference: String,
    pub expected_manifest_digest: String,
    pub resolved_manifest_digest: String,
    pub expected_metadata_manifest_digest: String,
    pub resolved_metadata_manifest_digest: String,
    pub expected_signature_manifest_digest: String,
    pub resolved_signature_manifest_digest: String,
    pub trust_domain: String,
    pub policy_blake3: String,
    pub verified_signers: Vec<String>,
    pub verified_public_key_blake3: Vec<String>,
    pub layout_blake3: String,
    pub projection_blake3: String,
    pub downloaded_blobs: u32,
    pub downloaded_bytes: u64,
    pub credential_mode: String,
    pub import_state: String,
    pub imported_objects: u32,
    pub import_receipt_blake3: String,
    pub non_claims: Vec<String>,
    pub receipt_blake3: String,
}

pub struct PushReportFacts<'a> {
    pub target: &'a RegistryTarget,
    pub plan: &'a RegistryPushPlan,
    pub signature_plan: &'a RegistrySignaturePlan,
    pub uploaded_blobs: u32,
    pub reused_blobs: u32,
    pub transferred_bytes: u64,
    pub credential_mode: &'a str,
}

pub struct PullReportFacts<'a> {
    pub target: &'a RegistryTarget,
    pub plan: &'a RegistryPullPlan,
    pub expected_manifest_digest: &'a str,
    pub expected_metadata_manifest_digest: &'a str,
    pub expected_signature_manifest_digest: &'a str,
    pub signature_manifest_descriptor: &'a OciDescriptor,
    pub trust_verification: &'a RegistryTrustVerification,
    pub downloaded_blobs: u32,
    pub downloaded_bytes: u64,
    pub credential_mode: &'a str,
    pub import_report: &'a OciImportReport,
}

fn validate_registry_url(value: &str, allow_http: bool) -> Result<String, String> {
    if value.is_empty() || value.len() > REGISTRY_BYTES_MAX {
        return Err("registry URL is empty or exceeds the named byte bound".to_string());
    }
    let mut parsed = Url::parse(value).map_err(|error| format!("invalid registry URL: {error}"))?;
    if parsed.scheme() != "https" && !(allow_http && parsed.scheme() == "http") {
        return Err("registry URL must use HTTPS unless --allow-http is explicit".to_string());
    }
    if parsed.host_str().is_none() || !parsed.username().is_empty() || parsed.password().is_some() {
        return Err("registry URL must name an authority and must not contain userinfo".to_string());
    }
    if parsed.query().is_some() || parsed.fragment().is_some() || !matches!(parsed.path(), "" | "/") {
        return Err("registry URL must not contain a path, query, or fragment".to_string());
    }
    parsed.set_path("");
    let normalized = parsed.as_str().trim_end_matches('/').to_string();
    assert!(!normalized.is_empty(), "validated registry URL must not be empty");
    assert!(normalized.len() <= REGISTRY_BYTES_MAX, "normalized registry URL must stay bounded");
    Ok(normalized)
}

fn repository_segment_is_valid(segment: &str) -> bool {
    if segment.is_empty() || segment.len() > REPOSITORY_SEGMENT_BYTES_MAX {
        return false;
    }
    let mut bytes = segment.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
        return false;
    }
    segment
        .bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-'))
}

fn validate_repository(value: &str) -> Result<String, String> {
    if value.is_empty() || value.len() > REPOSITORY_BYTES_MAX {
        return Err("registry repository is empty or exceeds the named byte bound".to_string());
    }
    if !value.split('/').all(repository_segment_is_valid) {
        return Err("registry repository contains an invalid path segment".to_string());
    }
    assert!(!value.starts_with('/'), "validated repository must be relative");
    assert!(!value.ends_with('/'), "validated repository must not have an empty tail segment");
    Ok(value.to_string())
}

fn tag_is_valid(value: &str) -> bool {
    if value.len() < TAG_BYTES_MIN || value.len() > TAG_BYTES_MAX {
        return false;
    }
    let mut bytes = value.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    if !first.is_ascii_alphanumeric() && first != b'_' {
        return false;
    }
    bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'-'))
}

pub fn validate_registry_target(input: RegistryTargetInput<'_>) -> Result<RegistryTarget, String> {
    let registry = validate_registry_url(input.registry, input.allow_http)?;
    let repository = validate_repository(input.repository)?;
    if !tag_is_valid(input.reference) {
        return Err("registry reference is not a bounded OCI tag".to_string());
    }
    let metadata_reference = format!("{}{METADATA_REFERENCE_SUFFIX}", input.reference);
    let signature_reference = format!("{}{SIGNATURE_REFERENCE_SUFFIX}", input.reference);
    if !tag_is_valid(&metadata_reference) || !tag_is_valid(&signature_reference) {
        return Err("registry reference leaves no room for Mantle companion suffixes".to_string());
    }
    assert_ne!(input.reference, metadata_reference, "metadata reference must be distinct");
    assert_ne!(input.reference, signature_reference, "signature reference must be distinct");
    assert!(metadata_reference.ends_with(METADATA_REFERENCE_SUFFIX), "metadata suffix must be preserved");
    assert!(signature_reference.ends_with(SIGNATURE_REFERENCE_SUFFIX), "signature suffix must be preserved");
    Ok(RegistryTarget {
        registry,
        repository,
        reference: input.reference.to_string(),
        metadata_reference,
        signature_reference,
    })
}

pub fn registry_v2_url(target: &RegistryTarget) -> String {
    assert!(!target.registry.ends_with('/'), "normalized registry must not end with slash");
    assert!(!target.repository.is_empty(), "validated repository must not be empty");
    format!("{}/v2/", target.registry)
}

pub fn registry_blob_url(target: &RegistryTarget, digest: &str) -> Result<String, String> {
    if !is_sha256_digest(digest) {
        return Err("registry blob URL requires a SHA-256 digest".to_string());
    }
    assert!(!target.repository.starts_with('/'), "validated repository must be relative");
    assert!(digest.starts_with("sha256:"), "validated OCI digest must retain its algorithm");
    Ok(format!("{}/v2/{}/blobs/{digest}", target.registry, target.repository))
}

pub fn registry_upload_url(target: &RegistryTarget) -> String {
    assert!(!target.repository.starts_with('/'), "validated repository must be relative");
    assert!(!target.repository.ends_with('/'), "validated repository must not end with slash");
    format!("{}/v2/{}/blobs/uploads/", target.registry, target.repository)
}

pub fn registry_manifest_url(target: &RegistryTarget, reference: &str) -> Result<String, String> {
    if !tag_is_valid(reference) && !is_sha256_digest(reference) {
        return Err("registry manifest reference must be a bounded tag or SHA-256 digest".to_string());
    }
    assert!(!target.repository.starts_with('/'), "validated repository must be relative");
    assert!(!reference.is_empty(), "validated manifest reference must not be empty");
    Ok(format!("{}/v2/{}/manifests/{reference}", target.registry, target.repository))
}

fn same_authority(left: &Url, right: &Url) -> bool {
    left.scheme() == right.scheme()
        && left.host_str() == right.host_str()
        && left.port_or_known_default() == right.port_or_known_default()
}

pub struct FinalizeUploadInput<'a> {
    pub target: &'a RegistryTarget,
    pub location: &'a str,
    pub digest: &'a str,
}

struct MetadataDescriptorInput<'a> {
    role: &'a str,
    media_type: &'a str,
    bytes: &'a [u8],
}

struct MetadataManifestInput<'a> {
    main: &'a OciDescriptor,
    oci_layout_bytes: &'a [u8],
    index_bytes: &'a [u8],
    export_report_bytes: &'a [u8],
    layout_blake3: &'a str,
    projection_blake3: &'a str,
}

pub fn finalize_upload_url(input: FinalizeUploadInput<'_>) -> Result<String, String> {
    if input.location.is_empty() || input.location.len() > REGISTRY_BYTES_MAX {
        return Err("registry upload location is empty or exceeds the named bound".to_string());
    }
    if !is_sha256_digest(input.digest) {
        return Err("registry upload digest is invalid".to_string());
    }
    let base = Url::parse(&format!("{}/", input.target.registry))
        .map_err(|error| format!("invalid registry base URL: {error}"))?;
    let mut resolved =
        base.join(input.location).map_err(|error| format!("invalid registry upload location: {error}"))?;
    let expected_prefix = format!("/v2/{}/blobs/uploads/", input.target.repository);
    if !same_authority(&base, &resolved) {
        return Err("registry upload location escaped the configured authority".to_string());
    }
    if !resolved.username().is_empty() || resolved.password().is_some() {
        return Err("registry upload location contains forbidden userinfo".to_string());
    }
    if !resolved.path().starts_with(&expected_prefix) {
        return Err("registry upload location escaped the configured repository".to_string());
    }
    if resolved.fragment().is_some() {
        return Err("registry upload location contains a forbidden fragment".to_string());
    }
    if resolved.query_pairs().any(|(key, _)| key == "digest") {
        return Err("registry upload location already contains an ambiguous digest".to_string());
    }
    resolved.query_pairs_mut().append_pair("digest", input.digest);
    let output = resolved.to_string();
    assert!(output.starts_with(&input.target.registry), "final upload URL must retain registry authority");
    assert!(output.contains("digest=sha256%3A"), "final upload URL must bind the digest query");
    Ok(output)
}

fn descriptor(media_type: &str, bytes: &[u8]) -> Result<OciDescriptor, String> {
    if media_type.is_empty() || bytes.is_empty() {
        return Err("descriptor media type and bytes must be non-empty".to_string());
    }
    let size_bytes = u64::try_from(bytes.len()).map_err(|_| "descriptor byte length overflowed u64".to_string())?;
    let output = OciDescriptor {
        media_type: media_type.to_string(),
        digest: sha256_digest(bytes),
        size: size_bytes,
        annotations: BTreeMap::new(),
        platform: None,
    };
    assert!(!output.media_type.is_empty(), "descriptor media type must be explicit");
    assert!(output.size > 0, "descriptor byte size must be positive");
    Ok(output)
}

fn empty_config_descriptor() -> Result<OciDescriptor, String> {
    let output = descriptor(OCI_EMPTY_CONFIG_MEDIA_TYPE, OCI_EMPTY_CONFIG_BYTES)?;
    assert_eq!(output.media_type, OCI_EMPTY_CONFIG_MEDIA_TYPE);
    assert_eq!(output.size, OCI_EMPTY_CONFIG_BYTES.len() as u64);
    Ok(output)
}

fn metadata_descriptor(input: MetadataDescriptorInput<'_>) -> Result<OciDescriptor, String> {
    let mut output = descriptor(input.media_type, input.bytes)?;
    output.annotations.insert(METADATA_ROLE_ANNOTATION.to_string(), input.role.to_string());
    assert_eq!(output.annotations.len(), 1, "metadata descriptors must have one role annotation");
    assert_eq!(output.annotations.get(METADATA_ROLE_ANNOTATION).map(String::as_str), Some(input.role));
    Ok(output)
}

fn canonical_json<T: Serialize>(value: &T) -> Result<Vec<u8>, String> {
    let bytes = serde_json::to_vec(value).map_err(|error| format!("serializing registry JSON: {error}"))?;
    if bytes.is_empty() {
        return Err("registry JSON serialization produced empty bytes".to_string());
    }
    assert!(!bytes.ends_with(b"\n"), "canonical registry JSON must be compact");
    assert_eq!(bytes[0], b'{', "registry JSON document must be an object");
    Ok(bytes)
}

fn bounded_identifier(value: &str, bytes_max: usize) -> bool {
    if value.is_empty() || value.len() > bytes_max {
        return false;
    }
    value.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn domain_blake3(domain: &str, bytes: &[u8]) -> String {
    assert!(!domain.is_empty(), "BLAKE3 domain must be explicit");
    assert!(!bytes.is_empty(), "BLAKE3 input must not be empty");
    let mut hasher = blake3::Hasher::new_derive_key(domain);
    hasher.update(bytes);
    hasher.finalize().to_hex().to_string()
}

pub fn registry_public_key_blake3(key: &VerifyingKey) -> String {
    let encoded = key.to_string();
    let separator_index = key.name().len();
    assert_eq!(encoded.as_bytes().get(separator_index), Some(&b':'));
    let key_material = &encoded[separator_index.saturating_add(1)..];
    let digest = domain_blake3(PUBLIC_KEY_DIGEST_DOMAIN, key_material.as_bytes());
    assert!(is_blake3_hex(&digest), "public-key identity must be BLAKE3");
    assert!(!key.name().is_empty(), "public key must retain a signer name");
    digest
}

fn normalize_unique(values: &mut [String], label: &str) -> Result<(), String> {
    if values.is_empty() || values.len() > TRUST_POLICY_ENTRY_COUNT_MAX {
        return Err(format!("registry trust policy {label} count is outside the named bound"));
    }
    values.sort();
    if values.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(format!("registry trust policy {label} contains duplicates"));
    }
    assert!(!values.is_empty(), "normalized policy collection must not be empty");
    assert!(values.len() <= TRUST_POLICY_ENTRY_COUNT_MAX, "normalized policy collection must stay bounded");
    Ok(())
}

fn normalize_policy(mut policy: RegistryTrustPolicy) -> Result<RegistryTrustPolicy, String> {
    normalize_unique(&mut policy.allowed_repositories, "allowed repositories")?;
    normalize_unique(&mut policy.trusted_public_keys, "trusted public keys")?;
    normalize_unique(&mut policy.required_signers, "required signers")?;
    policy.revoked_public_key_blake3.sort();
    if policy.revoked_public_key_blake3.len() > TRUST_POLICY_ENTRY_COUNT_MAX
        || policy.revoked_public_key_blake3.windows(2).any(|pair| pair[0] == pair[1])
    {
        return Err("registry trust policy revoked-key identities are duplicated or exceed the named bound".to_string());
    }
    assert!(policy.allowed_repositories.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(policy.trusted_public_keys.windows(2).all(|pair| pair[0] < pair[1]));
    Ok(policy)
}

fn parse_policy_keys(policy: &RegistryTrustPolicy) -> Result<Vec<VerifyingKey>, String> {
    let mut keys = Vec::with_capacity(policy.trusted_public_keys.len());
    for encoded in &policy.trusted_public_keys {
        let key = VerifyingKey::parse(encoded)
            .map_err(|error| format!("invalid OCI registry trusted public key '{encoded}': {error}"))?;
        if !bounded_identifier(key.name(), SIGNER_NAME_BYTES_MAX) {
            return Err("OCI registry trusted public key has an invalid signer name".to_string());
        }
        keys.push(key);
    }
    assert_eq!(keys.len(), policy.trusted_public_keys.len());
    assert!(!keys.is_empty(), "validated trust policy must retain public keys");
    Ok(keys)
}

fn validate_policy_shape(policy: &RegistryTrustPolicy) -> Result<(), String> {
    if policy.schema != OCI_REGISTRY_TRUST_POLICY_SCHEMA || policy.schema_version != TRUST_POLICY_SCHEMA_VERSION {
        return Err("OCI registry trust policy schema is unsupported".to_string());
    }
    if !bounded_identifier(&policy.trust_domain, TRUST_DOMAIN_BYTES_MAX) {
        return Err("OCI registry trust policy domain is invalid".to_string());
    }
    if !policy.allowed_repositories.iter().all(|value| validate_repository(value).is_ok()) {
        return Err("OCI registry trust policy contains an invalid repository".to_string());
    }
    if !policy.required_signers.iter().all(|value| bounded_identifier(value, SIGNER_NAME_BYTES_MAX)) {
        return Err("OCI registry trust policy contains an invalid required signer".to_string());
    }
    if !policy.revoked_public_key_blake3.iter().all(|value| is_blake3_hex(value)) {
        return Err("OCI registry trust policy contains an invalid revoked-key BLAKE3".to_string());
    }
    Ok(())
}

fn validate_policy_authority(policy: &RegistryTrustPolicy, keys: &[VerifyingKey]) -> Result<(), String> {
    let key_digests = keys.iter().map(registry_public_key_blake3).collect::<BTreeSet<_>>();
    if key_digests.len() != keys.len() {
        return Err("OCI registry trust policy repeats one full public-key identity".to_string());
    }
    let revoked = policy.revoked_public_key_blake3.iter().collect::<BTreeSet<_>>();
    let active = keys.iter().filter(|key| !revoked.contains(&registry_public_key_blake3(key))).collect::<Vec<_>>();
    let minimum = usize::from(policy.minimum_signatures);
    if minimum == 0 || minimum > active.len() || minimum > SIGNATURE_COUNT_MAX {
        return Err(
            "OCI registry trust policy minimum signature count is unsatisfied or outside the named bound".to_string()
        );
    }
    for signer in &policy.required_signers {
        if !active.iter().any(|key| key.name() == signer) {
            return Err(format!("OCI registry trust policy required signer has no active trusted key: {signer}"));
        }
    }
    assert!(active.len() <= TRUST_POLICY_ENTRY_COUNT_MAX);
    assert!(minimum <= active.len());
    Ok(())
}

pub fn validate_registry_trust_policy(policy: RegistryTrustPolicy) -> Result<ValidatedRegistryTrustPolicy, String> {
    let policy = normalize_policy(policy)?;
    validate_policy_shape(&policy)?;
    let verifying_keys = parse_policy_keys(&policy)?;
    validate_policy_authority(&policy, &verifying_keys)?;
    let canonical = canonical_json(&policy)?;
    let policy_blake3 = domain_blake3(TRUST_POLICY_DIGEST_DOMAIN, &canonical);
    assert!(is_blake3_hex(&policy_blake3), "policy identity must be BLAKE3");
    assert!(!verifying_keys.is_empty(), "validated policy must retain trust roots");
    Ok(ValidatedRegistryTrustPolicy {
        policy,
        policy_blake3,
        verifying_keys,
    })
}

pub fn require_policy_repository(policy: &ValidatedRegistryTrustPolicy, repository: &str) -> Result<(), String> {
    let repository = validate_repository(repository)?;
    if policy.policy.allowed_repositories.binary_search(&repository).is_err() {
        return Err(format!("OCI registry trust policy does not authorize repository: {repository}"));
    }
    assert!(!policy.policy.trust_domain.is_empty());
    assert!(!policy.policy.allowed_repositories.is_empty());
    Ok(())
}

fn trust_statement(plan: &RegistryPushPlan, policy: &ValidatedRegistryTrustPolicy) -> RegistryTrustStatement {
    assert!(is_sha256_digest(&plan.main_manifest_descriptor.digest));
    assert!(is_sha256_digest(&plan.metadata_manifest_descriptor.digest));
    RegistryTrustStatement {
        schema: OCI_REGISTRY_TRUST_STATEMENT_SCHEMA.to_string(),
        schema_version: TRUST_STATEMENT_SCHEMA_VERSION,
        signature_suite: SIGNATURE_SUITE.to_string(),
        trust_domain: policy.policy.trust_domain.clone(),
        manifest_digest: plan.main_manifest_descriptor.digest.clone(),
        metadata_manifest_digest: plan.metadata_manifest_descriptor.digest.clone(),
    }
}

fn verify_statement(
    statement: &RegistryTrustStatement,
    expectation: TrustStatementExpectation<'_>,
) -> Result<(), String> {
    if statement.schema != OCI_REGISTRY_TRUST_STATEMENT_SCHEMA
        || statement.schema_version != TRUST_STATEMENT_SCHEMA_VERSION
        || statement.signature_suite != SIGNATURE_SUITE
    {
        return Err("OCI registry signed statement schema or signature suite is unsupported".to_string());
    }
    if statement.trust_domain != expectation.policy.policy.trust_domain
        || statement.manifest_digest != expectation.manifest_digest
        || statement.metadata_manifest_digest != expectation.metadata_manifest_digest
    {
        return Err("OCI registry signed statement domain or immutable digest pair drifted".to_string());
    }
    if !is_sha256_digest(expectation.manifest_digest) || !is_sha256_digest(expectation.metadata_manifest_digest) {
        return Err("OCI registry signed statement requires SHA-256 manifest identities".to_string());
    }
    assert!(bounded_identifier(&statement.trust_domain, TRUST_DOMAIN_BYTES_MAX));
    assert_eq!(statement.signature_suite, SIGNATURE_SUITE);
    Ok(())
}

fn detached_signatures(
    statement_bytes: &[u8],
    signing_keys: &[crunch_build::KeyPair],
) -> Result<Vec<RegistryDetachedSignature>, String> {
    if signing_keys.is_empty() || signing_keys.len() > SIGNATURE_COUNT_MAX {
        return Err("OCI registry signing-key count is outside the named bound".to_string());
    }
    let mut signatures = signing_keys
        .iter()
        .map(|keypair| {
            let signature = keypair.signing_key.sign(statement_bytes).to_owned().to_string();
            RegistryDetachedSignature {
                signer: keypair.verifying_key.name().to_string(),
                signature,
            }
        })
        .collect::<Vec<_>>();
    signatures.sort_by(|left, right| (&left.signer, &left.signature).cmp(&(&right.signer, &right.signature)));
    if signatures.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err("OCI registry signature document contains duplicate signatures".to_string());
    }
    assert!(!signatures.is_empty());
    assert!(signatures.len() <= SIGNATURE_COUNT_MAX);
    Ok(signatures)
}

fn verify_detached_signatures(
    statement_bytes: &[u8],
    signatures: &[RegistryDetachedSignature],
    policy: &ValidatedRegistryTrustPolicy,
) -> Result<RegistryTrustVerification, String> {
    if signatures.is_empty() || signatures.len() > SIGNATURE_COUNT_MAX {
        return Err("OCI registry signature count is outside the named bound".to_string());
    }
    let statement =
        std::str::from_utf8(statement_bytes).map_err(|_| "OCI registry signed statement is not UTF-8".to_string())?;
    let revoked = policy.policy.revoked_public_key_blake3.iter().collect::<BTreeSet<_>>();
    let mut signers = BTreeSet::new();
    let mut key_digests = BTreeSet::new();
    for detached in signatures {
        let signature = SignatureRef::parse(&detached.signature)
            .map_err(|error| format!("invalid OCI registry detached signature: {error}"))?;
        if *signature.name() != detached.signer.as_str() {
            return Err("OCI registry detached signature signer label drifted".to_string());
        }
        let matching = policy.verifying_keys.iter().filter(|key| key.name() == detached.signer);
        let mut is_verified = false;
        for key in matching {
            let digest = registry_public_key_blake3(key);
            if !revoked.contains(&digest) && key.verify(statement, &signature) {
                is_verified = true;
                signers.insert(detached.signer.clone());
                key_digests.insert(digest);
            }
        }
        if !is_verified {
            return Err(format!("OCI registry signature is not trusted for signer: {}", detached.signer));
        }
    }
    let minimum = usize::from(policy.policy.minimum_signatures);
    if key_digests.len() < minimum || !policy.policy.required_signers.iter().all(|required| signers.contains(required))
    {
        return Err("OCI registry signatures do not satisfy required signers or distinct-key threshold".to_string());
    }
    assert!(key_digests.len() >= minimum);
    assert!(!signers.is_empty());
    Ok(RegistryTrustVerification {
        trust_domain: policy.policy.trust_domain.clone(),
        policy_blake3: policy.policy_blake3.clone(),
        verified_signers: signers.into_iter().collect(),
        verified_public_key_blake3: key_digests.into_iter().collect(),
    })
}

fn required_non_claims() -> Vec<String> {
    let mut output = REQUIRED_NON_CLAIMS.iter().map(ToString::to_string).collect::<Vec<_>>();
    output.sort();
    assert_eq!(output.len(), REQUIRED_NON_CLAIMS.len(), "registry non-claim count must stay fixed");
    assert!(output.windows(2).all(|pair| pair[0] < pair[1]), "registry non-claims must be unique");
    output
}

fn validate_credential_mode(value: &str) -> bool {
    matches!(value, CREDENTIAL_MODE_ANONYMOUS | CREDENTIAL_MODE_BEARER_FILE)
}

fn verify_descriptor_bytes(value: &OciDescriptor, bytes: &[u8]) -> Result<(), String> {
    if !is_sha256_digest(&value.digest) || value.digest != sha256_digest(bytes) {
        return Err(format!("descriptor digest mismatch: {}", value.digest));
    }
    if value.size != u64::try_from(bytes.len()).map_err(|_| "descriptor byte length overflowed".to_string())? {
        return Err(format!("descriptor size mismatch: {}", value.digest));
    }
    assert!(!value.media_type.is_empty(), "verified descriptor media type must not be empty");
    assert!(!bytes.is_empty(), "verified descriptor bytes must not be empty");
    Ok(())
}

fn parse_export_report(bytes: &[u8]) -> Result<OciExportReport, String> {
    let outcome: OciExportReport =
        serde_json::from_slice(bytes).map_err(|error| format!("parsing OCI export report: {error}"))?;
    verify_export_report(&outcome)
        .map_err(|issue| format!("invalid OCI export report: {}: {}", issue.code, issue.message))?;
    assert!(outcome.exported, "verified export report must be successful");
    assert!(outcome.issues.is_empty(), "verified export report must not retain issues");
    Ok(outcome)
}

fn admitted_layout(facts: &LayoutFacts) -> Result<OciImportReport, String> {
    let outcome = validate_import(facts).map_err(|issues| {
        format!("OCI layout registry admission failed: {}", serde_json::to_string(&issues).unwrap_or_default())
    })?;
    if outcome.state != "admitted" || outcome.projection_blake3.is_none() {
        return Err("registry publication requires an admitted Mantle OCI layout".to_string());
    }
    assert_eq!(outcome.state, "admitted", "admitted layout preview must retain its state");
    assert!(outcome.issues.is_empty(), "admitted layout preview must not retain issues");
    Ok(outcome)
}

fn main_manifest(facts: &LayoutFacts, export_report: &OciExportReport) -> Result<(OciDescriptor, Vec<u8>), String> {
    let index: OciIndexDocument = serde_json::from_slice(&facts.index_bytes)
        .map_err(|error| format!("parsing OCI index for registry push: {error}"))?;
    if index.manifests.len() != 1 {
        return Err("registry publication requires exactly one image manifest".to_string());
    }
    let main = index.manifests[0].clone();
    if main != export_report.manifest_descriptor || main.media_type != OCI_MANIFEST_MEDIA_TYPE {
        return Err("OCI index and export report disagree on the image manifest descriptor".to_string());
    }
    let bytes = facts
        .blobs
        .get(&main.digest)
        .cloned()
        .ok_or_else(|| "OCI image manifest blob is missing from the layout".to_string())?;
    verify_descriptor_bytes(&main, &bytes)?;
    assert!(is_sha256_digest(&main.digest), "verified image manifest digest must be SHA-256");
    assert_eq!(main.size, bytes.len() as u64, "verified image manifest size must match");
    Ok((main, bytes))
}

fn build_metadata_manifest(
    input: MetadataManifestInput<'_>,
) -> Result<(OciCompanionManifest, Vec<u8>, OciDescriptor), String> {
    let layers = vec![
        metadata_descriptor(MetadataDescriptorInput {
            role: METADATA_LAYOUT_ROLE,
            media_type: MANTLE_OCI_LAYOUT_MEDIA_TYPE,
            bytes: input.oci_layout_bytes,
        })?,
        metadata_descriptor(MetadataDescriptorInput {
            role: METADATA_INDEX_ROLE,
            media_type: MANTLE_OCI_INDEX_MEDIA_TYPE,
            bytes: input.index_bytes,
        })?,
        metadata_descriptor(MetadataDescriptorInput {
            role: METADATA_EXPORT_REPORT_ROLE,
            media_type: MANTLE_OCI_EXPORT_REPORT_MEDIA_TYPE,
            bytes: input.export_report_bytes,
        })?,
    ];
    let annotations = BTreeMap::from([
        (LAYOUT_DIGEST_ANNOTATION.to_string(), input.layout_blake3.to_string()),
        (PROJECTION_DIGEST_ANNOTATION.to_string(), input.projection_blake3.to_string()),
    ]);
    let manifest = OciCompanionManifest {
        schema_version: OCI_DOCUMENT_SCHEMA_VERSION,
        media_type: OCI_MANIFEST_MEDIA_TYPE.to_string(),
        artifact_type: MANTLE_METADATA_ARTIFACT_TYPE.to_string(),
        config: empty_config_descriptor()?,
        layers,
        subject: input.main.clone(),
        annotations,
    };
    let bytes = canonical_json(&manifest)?;
    let manifest_descriptor = descriptor(OCI_MANIFEST_MEDIA_TYPE, &bytes)?;
    assert_eq!(manifest.layers.len(), METADATA_BLOB_COUNT, "metadata manifest layer count must stay fixed");
    assert_eq!(manifest.subject.digest, input.main.digest, "metadata subject must bind the image manifest");
    Ok((manifest, bytes, manifest_descriptor))
}

fn signature_document_descriptor(bytes: &[u8]) -> Result<OciDescriptor, String> {
    let mut output = descriptor(MANTLE_SIGNATURE_DOCUMENT_MEDIA_TYPE, bytes)?;
    output
        .annotations
        .insert(SIGNATURE_ROLE_ANNOTATION.to_string(), SIGNATURE_DOCUMENT_ROLE.to_string());
    assert_eq!(output.annotations.len(), 1);
    assert_eq!(output.annotations.get(SIGNATURE_ROLE_ANNOTATION).map(String::as_str), Some(SIGNATURE_DOCUMENT_ROLE));
    Ok(output)
}

fn build_signature_manifest(
    input: SignatureManifestInput<'_>,
) -> Result<(OciCompanionManifest, Vec<u8>, OciDescriptor), String> {
    if !is_sha256_digest(input.metadata_digest) || !bounded_identifier(input.trust_domain, TRUST_DOMAIN_BYTES_MAX) {
        return Err("OCI registry signature artifact input is invalid".to_string());
    }
    let manifest = OciCompanionManifest {
        schema_version: OCI_DOCUMENT_SCHEMA_VERSION,
        media_type: OCI_MANIFEST_MEDIA_TYPE.to_string(),
        artifact_type: MANTLE_SIGNATURE_ARTIFACT_TYPE.to_string(),
        config: empty_config_descriptor()?,
        layers: vec![input.document_descriptor.clone()],
        subject: input.main.clone(),
        annotations: BTreeMap::from([
            (SIGNATURE_METADATA_DIGEST_ANNOTATION.to_string(), input.metadata_digest.to_string()),
            (SIGNATURE_TRUST_DOMAIN_ANNOTATION.to_string(), input.trust_domain.to_string()),
        ]),
    };
    let bytes = canonical_json(&manifest)?;
    let descriptor = descriptor(OCI_MANIFEST_MEDIA_TYPE, &bytes)?;
    assert_eq!(manifest.layers.len(), SIGNATURE_BLOB_COUNT);
    assert_eq!(manifest.subject.digest, input.main.digest);
    Ok((manifest, bytes, descriptor))
}

pub fn verify_registry_signature_document(
    input: RegistrySignatureVerificationInput<'_>,
) -> Result<(RegistrySignatureDocument, RegistryTrustVerification), String> {
    let document: RegistrySignatureDocument = serde_json::from_slice(input.bytes)
        .map_err(|error| format!("parsing OCI registry signature document: {error}"))?;
    if document.schema != OCI_REGISTRY_SIGNATURE_DOCUMENT_SCHEMA
        || document.schema_version != SIGNATURE_DOCUMENT_SCHEMA_VERSION
        || canonical_json(&document)? != input.bytes
    {
        return Err("OCI registry signature document schema or canonical bytes are invalid".to_string());
    }
    verify_statement(&document.statement, TrustStatementExpectation {
        manifest_digest: input.manifest_digest,
        metadata_manifest_digest: input.metadata_manifest_digest,
        policy: input.policy,
    })?;
    if document
        .signatures
        .windows(2)
        .any(|pair| (&pair[0].signer, &pair[0].signature) >= (&pair[1].signer, &pair[1].signature))
    {
        return Err("OCI registry detached signatures are duplicated or not canonical".to_string());
    }
    let statement_bytes = canonical_json(&document.statement)?;
    let verification = verify_detached_signatures(&statement_bytes, &document.signatures, input.policy)?;
    assert_eq!(verification.trust_domain, document.statement.trust_domain);
    assert!(is_blake3_hex(&verification.policy_blake3));
    Ok((document, verification))
}

pub fn build_registry_signature_plan(
    plan: &RegistryPushPlan,
    policy: &ValidatedRegistryTrustPolicy,
    signing_keys: &[crunch_build::KeyPair],
) -> Result<RegistrySignaturePlan, String> {
    let statement = trust_statement(plan, policy);
    let statement_bytes = canonical_json(&statement)?;
    let signatures = detached_signatures(&statement_bytes, signing_keys)?;
    let document = RegistrySignatureDocument {
        schema: OCI_REGISTRY_SIGNATURE_DOCUMENT_SCHEMA.to_string(),
        schema_version: SIGNATURE_DOCUMENT_SCHEMA_VERSION,
        statement,
        signatures,
    };
    let document_bytes = canonical_json(&document)?;
    let (_, verification) = verify_registry_signature_document(RegistrySignatureVerificationInput {
        bytes: &document_bytes,
        manifest_digest: &plan.main_manifest_descriptor.digest,
        metadata_manifest_digest: &plan.metadata_manifest_descriptor.digest,
        policy,
    })?;
    let document_descriptor = signature_document_descriptor(&document_bytes)?;
    let (manifest, manifest_bytes, manifest_descriptor) = build_signature_manifest(SignatureManifestInput {
        main: &plan.main_manifest_descriptor,
        metadata_digest: &plan.metadata_manifest_descriptor.digest,
        trust_domain: &policy.policy.trust_domain,
        document_descriptor: &document_descriptor,
    })?;
    assert_eq!(verification.policy_blake3, policy.policy_blake3);
    assert_eq!(manifest.layers[0].digest, document_descriptor.digest);
    Ok(RegistrySignaturePlan {
        document,
        document_descriptor,
        document_bytes,
        manifest,
        manifest_descriptor,
        manifest_bytes,
        verification,
    })
}

pub fn inspect_signature_manifest(
    bytes: &[u8],
    expected_digest: &str,
    main: &OciDescriptor,
    metadata_digest: &str,
    policy: &ValidatedRegistryTrustPolicy,
) -> Result<(OciCompanionManifest, OciDescriptor, OciDescriptor), String> {
    if sha256_digest(bytes) != expected_digest {
        return Err("Mantle signature manifest digest mismatch".to_string());
    }
    let manifest: OciCompanionManifest =
        serde_json::from_slice(bytes).map_err(|error| format!("parsing Mantle signature manifest: {error}"))?;
    let is_subject_match = manifest.subject.digest == main.digest
        && manifest.subject.size == main.size
        && manifest.subject.media_type == main.media_type;
    let is_annotation_set_valid = manifest.annotations.len() == 2
        && manifest.annotations.get(SIGNATURE_METADATA_DIGEST_ANNOTATION).map(String::as_str) == Some(metadata_digest)
        && manifest.annotations.get(SIGNATURE_TRUST_DOMAIN_ANNOTATION).map(String::as_str)
            == Some(policy.policy.trust_domain.as_str());
    let is_manifest_identity_valid = manifest.schema_version == OCI_DOCUMENT_SCHEMA_VERSION
        && manifest.media_type == OCI_MANIFEST_MEDIA_TYPE
        && manifest.artifact_type == MANTLE_SIGNATURE_ARTIFACT_TYPE
        && manifest.config == empty_config_descriptor()?;
    if !is_manifest_identity_valid {
        return Err("Mantle signature manifest identity is invalid".to_string());
    }
    if !is_subject_match || !is_annotation_set_valid {
        return Err("Mantle signature manifest subject or annotations are invalid".to_string());
    }
    if manifest.layers.len() != SIGNATURE_BLOB_COUNT {
        return Err("Mantle signature manifest must contain exactly one signature document".to_string());
    }
    let document = manifest.layers[0].clone();
    let is_document_identity_valid = document.media_type == MANTLE_SIGNATURE_DOCUMENT_MEDIA_TYPE
        && document.size > 0
        && is_sha256_digest(&document.digest);
    let is_document_role_valid = document.annotations.len() == 1
        && document.annotations.get(SIGNATURE_ROLE_ANNOTATION).map(String::as_str) == Some(SIGNATURE_DOCUMENT_ROLE);
    if !is_document_identity_valid || !is_document_role_valid {
        return Err("Mantle signature document descriptor is invalid".to_string());
    }
    let manifest_descriptor = descriptor(OCI_MANIFEST_MEDIA_TYPE, bytes)?;
    assert_eq!(manifest_descriptor.digest, expected_digest);
    assert_eq!(manifest.layers.len(), SIGNATURE_BLOB_COUNT);
    Ok((manifest, manifest_descriptor, document))
}

fn insert_blob(blobs: &mut BTreeMap<String, Vec<u8>>, digest: String, bytes: Vec<u8>) -> Result<(), String> {
    if let Some(existing) = blobs.get(&digest) {
        if existing != &bytes {
            return Err(format!("registry blob digest collision for {digest}"));
        }
        return Ok(());
    }
    if digest != sha256_digest(&bytes) {
        return Err(format!("registry blob bytes do not match {digest}"));
    }
    blobs.insert(digest, bytes);
    Ok(())
}

pub fn build_registry_push_plan(input: RegistryLayoutInput) -> Result<RegistryPushPlan, String> {
    let projection_evidence = parse_export_report(&input.export_report_bytes)?;
    let facts = LayoutFacts {
        oci_layout_bytes: input.oci_layout_bytes,
        index_bytes: input.index_bytes,
        blobs: input.descriptor_blobs,
        export_report: Some(projection_evidence.clone()),
    };
    let admission = admitted_layout(&facts)?;
    let (main_manifest_descriptor, main_manifest_bytes) = main_manifest(&facts, &projection_evidence)?;
    let (metadata, metadata_manifest_bytes, metadata_manifest_descriptor) =
        build_metadata_manifest(MetadataManifestInput {
            main: &main_manifest_descriptor,
            oci_layout_bytes: &facts.oci_layout_bytes,
            index_bytes: &facts.index_bytes,
            export_report_bytes: &input.export_report_bytes,
            layout_blake3: &projection_evidence.layout_blake3,
            projection_blake3: &projection_evidence.projection_blake3,
        })?;
    let mut blob_map = facts.blobs.clone();
    blob_map.remove(&main_manifest_descriptor.digest);
    insert_blob(&mut blob_map, metadata.config.digest.clone(), OCI_EMPTY_CONFIG_BYTES.to_vec())?;
    for (value, bytes) in metadata.layers.iter().zip([
        facts.oci_layout_bytes.clone(),
        facts.index_bytes.clone(),
        input.export_report_bytes,
    ]) {
        insert_blob(&mut blob_map, value.digest.clone(), bytes)?;
    }
    let blobs = blob_map.into_iter().map(|(digest, bytes)| RegistryBlob { digest, bytes }).collect::<Vec<_>>();
    if blobs.is_empty() {
        return Err("registry push plan has no content blobs".to_string());
    }
    assert_eq!(
        admission.layout_blake3, projection_evidence.layout_blake3,
        "admission and export layout identities must agree"
    );
    assert!(
        blobs.iter().all(|blob| blob.digest == sha256_digest(&blob.bytes)),
        "push plan blobs must be verified"
    );
    Ok(RegistryPushPlan {
        main_manifest_descriptor,
        main_manifest_bytes,
        metadata_manifest_descriptor,
        metadata_manifest_bytes,
        blobs,
        layout_blake3: projection_evidence.layout_blake3,
        projection_blake3: projection_evidence.projection_blake3,
        export_receipt_blake3: projection_evidence.receipt_blake3,
    })
}

fn metadata_role(descriptor: &OciDescriptor) -> Option<&str> {
    descriptor.annotations.get(METADATA_ROLE_ANNOTATION).map(String::as_str)
}

fn metadata_descriptor_is_valid(descriptor: &OciDescriptor) -> bool {
    if descriptor.annotations.len() != 1 || !is_sha256_digest(&descriptor.digest) || descriptor.size == 0 {
        return false;
    }
    matches!(
        (metadata_role(descriptor), descriptor.media_type.as_str()),
        (Some(METADATA_LAYOUT_ROLE), MANTLE_OCI_LAYOUT_MEDIA_TYPE)
            | (Some(METADATA_INDEX_ROLE), MANTLE_OCI_INDEX_MEDIA_TYPE)
            | (Some(METADATA_EXPORT_REPORT_ROLE), MANTLE_OCI_EXPORT_REPORT_MEDIA_TYPE)
    )
}

pub fn inspect_metadata_manifest(
    bytes: &[u8],
    expected_digest: &str,
    main: &OciDescriptor,
) -> Result<(OciCompanionManifest, OciDescriptor), String> {
    if sha256_digest(bytes) != expected_digest {
        return Err("Mantle metadata manifest digest mismatch".to_string());
    }
    let manifest: OciCompanionManifest =
        serde_json::from_slice(bytes).map_err(|error| format!("parsing Mantle metadata manifest: {error}"))?;
    let is_manifest_identity_valid = manifest.schema_version == OCI_DOCUMENT_SCHEMA_VERSION
        && manifest.media_type == OCI_MANIFEST_MEDIA_TYPE
        && manifest.artifact_type == MANTLE_METADATA_ARTIFACT_TYPE
        && manifest.config == empty_config_descriptor()?;
    if !is_manifest_identity_valid {
        return Err("Mantle metadata manifest version or media type is unsupported".to_string());
    }
    if manifest.subject.digest != main.digest
        || manifest.subject.size != main.size
        || manifest.subject.media_type != main.media_type
    {
        return Err("Mantle metadata subject does not match the resolved image manifest".to_string());
    }
    let roles = manifest.layers.iter().filter_map(metadata_role).collect::<BTreeSet<_>>();
    let required = BTreeSet::from([METADATA_LAYOUT_ROLE, METADATA_INDEX_ROLE, METADATA_EXPORT_REPORT_ROLE]);
    if manifest.layers.len() != METADATA_BLOB_COUNT
        || roles != required
        || !manifest.layers.iter().all(metadata_descriptor_is_valid)
    {
        return Err("Mantle metadata manifest roles, media types, or descriptors are invalid".to_string());
    }
    let layout_identity = manifest.annotations.get(LAYOUT_DIGEST_ANNOTATION);
    let projection_identity = manifest.annotations.get(PROJECTION_DIGEST_ANNOTATION);
    if manifest.annotations.len() != 2
        || !layout_identity.is_some_and(|value| is_blake3_hex(value))
        || !projection_identity.is_some_and(|value| is_blake3_hex(value))
    {
        return Err("Mantle metadata manifest identity annotations are invalid".to_string());
    }
    let descriptor = descriptor(OCI_MANIFEST_MEDIA_TYPE, bytes)?;
    assert_eq!(descriptor.digest, expected_digest, "verified metadata digest must match expected digest");
    assert_eq!(roles.len(), METADATA_BLOB_COUNT, "verified metadata roles must be unique");
    Ok((manifest, descriptor))
}

pub fn pull_blob_descriptors(
    main_manifest_bytes: &[u8],
    metadata: &OciCompanionManifest,
) -> Result<Vec<OciDescriptor>, String> {
    let manifest: OciManifestDocument = serde_json::from_slice(main_manifest_bytes)
        .map_err(|error| format!("parsing registry image manifest: {error}"))?;
    let descriptor_count_max =
        OCI_LAYER_MAX_COUNT.saturating_add(METADATA_BLOB_COUNT).saturating_add(1).saturating_add(1);
    let candidates = std::iter::once(manifest.config)
        .chain(manifest.layers)
        .chain(std::iter::once(metadata.config.clone()))
        .chain(metadata.layers.iter().cloned())
        .collect::<Vec<_>>();
    if candidates.is_empty() || candidates.len() > descriptor_count_max {
        return Err("registry pull descriptor closure exceeds the named bound".to_string());
    }
    let mut unique = BTreeMap::<String, OciDescriptor>::new();
    for value in candidates {
        if !is_sha256_digest(&value.digest) || value.size == 0 {
            return Err("registry pull descriptor has an invalid digest or size".to_string());
        }
        if let Some(existing) = unique.get(&value.digest) {
            if existing.size != value.size {
                return Err("registry pull repeats a digest with another size".to_string());
            }
            continue;
        }
        if unique.len() >= descriptor_count_max {
            return Err("registry pull unique descriptor closure exceeds the named bound".to_string());
        }
        unique.insert(value.digest.clone(), value);
    }
    let output = unique.into_values().collect::<Vec<_>>();
    assert!(!output.is_empty(), "validated pull descriptor closure must not be empty");
    assert!(output.len() <= descriptor_count_max, "validated pull descriptor closure must stay bounded");
    Ok(output)
}

fn role_bytes(
    manifest: &OciCompanionManifest,
    role: &str,
    downloaded: &BTreeMap<String, Vec<u8>>,
) -> Result<Vec<u8>, String> {
    let descriptor = manifest
        .layers
        .iter()
        .find(|value| metadata_role(value) == Some(role))
        .ok_or_else(|| format!("Mantle metadata role is missing: {role}"))?;
    let bytes = downloaded
        .get(&descriptor.digest)
        .cloned()
        .ok_or_else(|| format!("downloaded Mantle metadata blob is missing: {role}"))?;
    verify_descriptor_bytes(descriptor, &bytes)?;
    assert_eq!(metadata_role(descriptor), Some(role), "selected metadata descriptor must preserve its role");
    assert!(!bytes.is_empty(), "verified metadata bytes must not be empty");
    Ok(bytes)
}

fn main_descriptor(media_type: &str, bytes: &[u8], expected_digest: &str) -> Result<OciDescriptor, String> {
    if media_type != OCI_MANIFEST_MEDIA_TYPE {
        return Err("registry image manifest media type is unsupported".to_string());
    }
    let output = descriptor(media_type, bytes)?;
    if output.digest != expected_digest {
        return Err("resolved registry tag does not match the expected image manifest digest".to_string());
    }
    assert_eq!(output.media_type, OCI_MANIFEST_MEDIA_TYPE, "main manifest media type must be canonical");
    assert_eq!(output.digest, expected_digest, "main manifest digest must be immutable");
    Ok(output)
}

fn descriptor_closure(
    main: &OciDescriptor,
    main_bytes: &[u8],
    index_bytes: &[u8],
    downloaded: &BTreeMap<String, Vec<u8>>,
) -> Result<BTreeMap<String, Vec<u8>>, String> {
    let index: OciIndexDocument =
        serde_json::from_slice(index_bytes).map_err(|error| format!("parsing pulled OCI index: {error}"))?;
    if index.manifests.len() != 1 || index.manifests[0] != *main {
        return Err("pulled OCI index does not bind the resolved image manifest exactly".to_string());
    }
    let manifest: OciManifestDocument =
        serde_json::from_slice(main_bytes).map_err(|error| format!("parsing pulled OCI image manifest: {error}"))?;
    let descriptors = std::iter::once(&manifest.config).chain(manifest.layers.iter()).collect::<Vec<_>>();
    let mut closure = BTreeMap::from([(main.digest.clone(), main_bytes.to_vec())]);
    for value in descriptors {
        let bytes = downloaded
            .get(&value.digest)
            .cloned()
            .ok_or_else(|| format!("pulled OCI descriptor blob is missing: {}", value.digest))?;
        verify_descriptor_bytes(value, &bytes)?;
        insert_blob(&mut closure, value.digest.clone(), bytes)?;
    }
    assert!(closure.contains_key(&main.digest), "descriptor closure must retain the image manifest");
    assert!(closure.len() >= 2, "descriptor closure must retain config plus manifest");
    Ok(closure)
}

pub fn build_registry_pull_plan(input: RegistryPullInput) -> Result<RegistryPullPlan, String> {
    if !is_sha256_digest(&input.expected_manifest_digest) || !is_sha256_digest(&input.metadata_manifest_digest) {
        return Err("registry pull manifest expectations must use SHA-256 descriptors".to_string());
    }
    let resolved_main =
        main_descriptor(&input.main_manifest_media_type, &input.main_manifest_bytes, &input.expected_manifest_digest)?;
    let (metadata, metadata_manifest_descriptor) =
        inspect_metadata_manifest(&input.metadata_manifest_bytes, &input.metadata_manifest_digest, &resolved_main)?;
    let main = metadata.subject.clone();
    let oci_layout_bytes = role_bytes(&metadata, METADATA_LAYOUT_ROLE, &input.downloaded_blobs)?;
    let index_bytes = role_bytes(&metadata, METADATA_INDEX_ROLE, &input.downloaded_blobs)?;
    let export_report_bytes = role_bytes(&metadata, METADATA_EXPORT_REPORT_ROLE, &input.downloaded_blobs)?;
    let projection_evidence = parse_export_report(&export_report_bytes)?;
    if metadata.annotations.get(LAYOUT_DIGEST_ANNOTATION) != Some(&projection_evidence.layout_blake3)
        || metadata.annotations.get(PROJECTION_DIGEST_ANNOTATION) != Some(&projection_evidence.projection_blake3)
    {
        return Err("Mantle metadata annotations disagree with the exact export report".to_string());
    }
    let descriptor_blobs =
        descriptor_closure(&main, &input.main_manifest_bytes, &index_bytes, &input.downloaded_blobs)?;
    let facts = LayoutFacts {
        oci_layout_bytes: oci_layout_bytes.clone(),
        index_bytes: index_bytes.clone(),
        blobs: descriptor_blobs.clone(),
        export_report: Some(projection_evidence.clone()),
    };
    let admission = admitted_layout(&facts)?;
    if admission.layout_blake3 != projection_evidence.layout_blake3 {
        return Err("pulled layout and export report BLAKE3 identities disagree".to_string());
    }
    assert_eq!(admission.projection_blake3.as_deref(), Some(projection_evidence.projection_blake3.as_str()));
    assert_eq!(main.digest, input.expected_manifest_digest, "pull plan must retain the expected immutable digest");
    Ok(RegistryPullPlan {
        main_manifest_descriptor: main,
        metadata_manifest_descriptor,
        oci_layout_bytes,
        index_bytes,
        descriptor_blobs,
        export_report_bytes,
        layout_blake3: projection_evidence.layout_blake3,
        projection_blake3: projection_evidence.projection_blake3,
        export_receipt_blake3: projection_evidence.receipt_blake3,
        import_preview: admission,
    })
}

fn receipt_hash<T: Serialize>(domain: &str, value: &T) -> Result<String, String> {
    let bytes = serde_json::to_vec(value).map_err(|error| format!("serializing registry receipt: {error}"))?;
    let mut hasher = blake3::Hasher::new_derive_key(domain);
    hasher.update(&bytes);
    let digest = hasher.finalize().to_hex().to_string();
    assert!(is_blake3_hex(&digest), "registry receipt identity must be BLAKE3");
    assert!(!bytes.is_empty(), "registry receipt material must not be empty");
    Ok(digest)
}

fn verify_report_trust_evidence(input: ReportTrustEvidenceInput<'_>) -> Result<(), String> {
    if !bounded_identifier(input.trust_domain, TRUST_DOMAIN_BYTES_MAX) || !is_blake3_hex(input.policy_blake3) {
        return Err("registry report trust domain or policy identity is invalid".to_string());
    }
    if input.verified_signers.is_empty() || input.verified_signers.len() > SIGNATURE_COUNT_MAX {
        return Err("registry report verified signer count is invalid".to_string());
    }
    if !input.verified_signers.windows(2).all(|pair| pair[0] < pair[1]) {
        return Err("registry report verified signers are duplicated or unsorted".to_string());
    }
    if !input.verified_signers.iter().all(|value| bounded_identifier(value, SIGNER_NAME_BYTES_MAX)) {
        return Err("registry report verified signer name is invalid".to_string());
    }
    if input.verified_public_key_blake3.is_empty() || input.verified_public_key_blake3.len() > SIGNATURE_COUNT_MAX {
        return Err("registry report verified public-key count is invalid".to_string());
    }
    if !input.verified_public_key_blake3.windows(2).all(|pair| pair[0] < pair[1]) {
        return Err("registry report verified public-key identities are duplicated or unsorted".to_string());
    }
    if !input.verified_public_key_blake3.iter().all(|value| is_blake3_hex(value)) {
        return Err("registry report verified public-key identity is invalid".to_string());
    }
    assert!(!input.verified_signers.is_empty());
    assert!(!input.verified_public_key_blake3.is_empty());
    Ok(())
}

pub fn build_push_report(facts: PushReportFacts<'_>) -> Result<OciRegistryPushReport, String> {
    if !validate_credential_mode(facts.credential_mode) {
        return Err("registry push credential mode is unsupported".to_string());
    }
    if facts.uploaded_blobs.saturating_add(facts.reused_blobs) == 0 {
        return Err("registry push receipt must account for at least one blob".to_string());
    }
    let mut outcome = OciRegistryPushReport {
        schema: OCI_REGISTRY_PUSH_REPORT_SCHEMA.to_string(),
        schema_version: REGISTRY_SCHEMA_VERSION,
        published: true,
        registry: facts.target.registry.clone(),
        repository: facts.target.repository.clone(),
        reference: facts.target.reference.clone(),
        metadata_reference: facts.target.metadata_reference.clone(),
        signature_reference: facts.target.signature_reference.clone(),
        manifest_digest: facts.plan.main_manifest_descriptor.digest.clone(),
        metadata_manifest_digest: facts.plan.metadata_manifest_descriptor.digest.clone(),
        signature_manifest_digest: facts.signature_plan.manifest_descriptor.digest.clone(),
        trust_domain: facts.signature_plan.verification.trust_domain.clone(),
        policy_blake3: facts.signature_plan.verification.policy_blake3.clone(),
        verified_signers: facts.signature_plan.verification.verified_signers.clone(),
        verified_public_key_blake3: facts.signature_plan.verification.verified_public_key_blake3.clone(),
        layout_blake3: facts.plan.layout_blake3.clone(),
        projection_blake3: facts.plan.projection_blake3.clone(),
        export_receipt_blake3: facts.plan.export_receipt_blake3.clone(),
        uploaded_blobs: facts.uploaded_blobs,
        reused_blobs: facts.reused_blobs,
        transferred_bytes: facts.transferred_bytes,
        credential_mode: facts.credential_mode.to_string(),
        non_claims: required_non_claims(),
        receipt_blake3: String::new(),
    };
    outcome.receipt_blake3 = receipt_hash(REGISTRY_PUSH_RECEIPT_DOMAIN, &outcome)?;
    verify_push_report(&outcome)?;
    assert!(outcome.published, "verified push report must be successful");
    assert!(is_blake3_hex(&outcome.receipt_blake3), "push report receipt must use BLAKE3");
    Ok(outcome)
}

pub fn verify_push_report(report: &OciRegistryPushReport) -> Result<(), String> {
    if report.schema != OCI_REGISTRY_PUSH_REPORT_SCHEMA
        || report.schema_version != REGISTRY_SCHEMA_VERSION
        || !report.published
    {
        return Err("registry push report schema or status is invalid".to_string());
    }
    let target = validate_registry_target(RegistryTargetInput {
        registry: &report.registry,
        repository: &report.repository,
        reference: &report.reference,
        allow_http: true,
    })?;
    if target.metadata_reference != report.metadata_reference
        || target.signature_reference != report.signature_reference
        || !validate_credential_mode(&report.credential_mode)
    {
        return Err("registry push report target or credential mode is invalid".to_string());
    }
    if !is_sha256_digest(&report.manifest_digest)
        || !is_sha256_digest(&report.metadata_manifest_digest)
        || !is_sha256_digest(&report.signature_manifest_digest)
    {
        return Err("registry push report manifest digest is invalid".to_string());
    }
    verify_report_trust_evidence(ReportTrustEvidenceInput {
        trust_domain: &report.trust_domain,
        policy_blake3: &report.policy_blake3,
        verified_signers: &report.verified_signers,
        verified_public_key_blake3: &report.verified_public_key_blake3,
    })?;
    if ![
        &report.layout_blake3,
        &report.projection_blake3,
        &report.export_receipt_blake3,
    ]
    .into_iter()
    .all(|value| is_blake3_hex(value))
    {
        return Err("registry push report Mantle digest is invalid".to_string());
    }
    if report.non_claims != required_non_claims() {
        return Err("registry push report non-claims are invalid".to_string());
    }
    let blob_count_total = report
        .uploaded_blobs
        .checked_add(report.reused_blobs)
        .ok_or_else(|| "registry push report blob count overflowed".to_string())?;
    if blob_count_total == 0 || blob_count_total > REGISTRY_RECEIPT_BLOB_COUNT_MAX {
        return Err("registry push report blob accounting is outside the named bound".to_string());
    }
    if report.transferred_bytes > REGISTRY_RECEIPT_TRANSFER_BYTES_MAX {
        return Err("registry push report transfer bytes exceed the named bound".to_string());
    }
    let mut canonical = report.clone();
    canonical.receipt_blake3.clear();
    if !is_blake3_hex(&report.receipt_blake3)
        || receipt_hash(REGISTRY_PUSH_RECEIPT_DOMAIN, &canonical)? != report.receipt_blake3
    {
        return Err("registry push report receipt BLAKE3 is invalid".to_string());
    }
    assert_eq!(target.registry, report.registry, "verified push registry must be normalized");
    assert!(report.non_claims.len() >= METADATA_BLOB_COUNT, "verified push report must retain non-claims");
    Ok(())
}

pub fn build_pull_report(facts: PullReportFacts<'_>) -> Result<OciRegistryPullReport, String> {
    if !validate_credential_mode(facts.credential_mode) || facts.import_report.state != "admitted" {
        return Err("registry pull receipt requires an admitted import and supported credential mode".to_string());
    }
    verify_import_report(facts.import_report)?;
    let admitted_objects_count = u32::try_from(facts.import_report.objects.len())
        .map_err(|_| "registry pull imported object count overflowed".to_string())?;
    let mut outcome = OciRegistryPullReport {
        schema: OCI_REGISTRY_PULL_REPORT_SCHEMA.to_string(),
        schema_version: REGISTRY_SCHEMA_VERSION,
        pulled: true,
        registry: facts.target.registry.clone(),
        repository: facts.target.repository.clone(),
        reference: facts.target.reference.clone(),
        metadata_reference: facts.target.metadata_reference.clone(),
        signature_reference: facts.target.signature_reference.clone(),
        expected_manifest_digest: facts.expected_manifest_digest.to_string(),
        resolved_manifest_digest: facts.plan.main_manifest_descriptor.digest.clone(),
        expected_metadata_manifest_digest: facts.expected_metadata_manifest_digest.to_string(),
        resolved_metadata_manifest_digest: facts.plan.metadata_manifest_descriptor.digest.clone(),
        expected_signature_manifest_digest: facts.expected_signature_manifest_digest.to_string(),
        resolved_signature_manifest_digest: facts.signature_manifest_descriptor.digest.clone(),
        trust_domain: facts.trust_verification.trust_domain.clone(),
        policy_blake3: facts.trust_verification.policy_blake3.clone(),
        verified_signers: facts.trust_verification.verified_signers.clone(),
        verified_public_key_blake3: facts.trust_verification.verified_public_key_blake3.clone(),
        layout_blake3: facts.plan.layout_blake3.clone(),
        projection_blake3: facts.plan.projection_blake3.clone(),
        downloaded_blobs: facts.downloaded_blobs,
        downloaded_bytes: facts.downloaded_bytes,
        credential_mode: facts.credential_mode.to_string(),
        import_state: facts.import_report.state.clone(),
        imported_objects: admitted_objects_count,
        import_receipt_blake3: facts.import_report.receipt_blake3.clone(),
        non_claims: required_non_claims(),
        receipt_blake3: String::new(),
    };
    outcome.receipt_blake3 = receipt_hash(REGISTRY_PULL_RECEIPT_DOMAIN, &outcome)?;
    verify_pull_report(&outcome)?;
    assert!(outcome.pulled, "verified pull report must be successful");
    assert_eq!(outcome.import_state, "admitted", "verified pull report must retain admission state");
    Ok(outcome)
}

fn verify_pull_manifest_bindings(report: &OciRegistryPullReport) -> Result<(), String> {
    if report.expected_manifest_digest != report.resolved_manifest_digest {
        return Err("registry pull report image manifest binding is mutable".to_string());
    }
    if report.expected_metadata_manifest_digest != report.resolved_metadata_manifest_digest {
        return Err("registry pull report metadata manifest binding is mutable".to_string());
    }
    if report.expected_signature_manifest_digest != report.resolved_signature_manifest_digest {
        return Err("registry pull report signature manifest binding is mutable".to_string());
    }
    let is_primary_digest_pair_valid = is_sha256_digest(&report.expected_manifest_digest)
        && is_sha256_digest(&report.expected_metadata_manifest_digest);
    if !is_primary_digest_pair_valid || !is_sha256_digest(&report.expected_signature_manifest_digest) {
        return Err("registry pull report immutable manifest binding is invalid".to_string());
    }
    assert_eq!(report.expected_manifest_digest, report.resolved_manifest_digest);
    assert_eq!(report.expected_signature_manifest_digest, report.resolved_signature_manifest_digest);
    Ok(())
}

fn verify_pull_report_accounting(report: &OciRegistryPullReport) -> Result<(), String> {
    if report.import_state != "admitted" {
        return Err("registry pull report admission state is invalid".to_string());
    }
    if report.imported_objects == 0 || report.imported_objects > REGISTRY_RECEIPT_OBJECT_COUNT_MAX {
        return Err("registry pull report imported-object accounting is outside the named bound".to_string());
    }
    if report.downloaded_blobs == 0 || report.downloaded_blobs > REGISTRY_RECEIPT_BLOB_COUNT_MAX {
        return Err("registry pull report blob accounting is outside the named bound".to_string());
    }
    if report.downloaded_bytes == 0 || report.downloaded_bytes > REGISTRY_RECEIPT_TRANSFER_BYTES_MAX {
        return Err("registry pull report transfer bytes are outside the named bound".to_string());
    }
    if report.non_claims != required_non_claims() {
        return Err("registry pull report non-claims are invalid".to_string());
    }
    assert!(report.imported_objects > 0);
    assert!(report.downloaded_bytes > 0);
    Ok(())
}

fn verify_pull_report_receipt(report: &OciRegistryPullReport) -> Result<(), String> {
    let mut canonical = report.clone();
    canonical.receipt_blake3.clear();
    if !is_blake3_hex(&report.receipt_blake3)
        || receipt_hash(REGISTRY_PULL_RECEIPT_DOMAIN, &canonical)? != report.receipt_blake3
    {
        return Err("registry pull report receipt BLAKE3 is invalid".to_string());
    }
    assert!(is_blake3_hex(&report.receipt_blake3));
    assert!(canonical.receipt_blake3.is_empty());
    Ok(())
}

pub fn verify_pull_report(report: &OciRegistryPullReport) -> Result<(), String> {
    if report.schema != OCI_REGISTRY_PULL_REPORT_SCHEMA
        || report.schema_version != REGISTRY_SCHEMA_VERSION
        || !report.pulled
    {
        return Err("registry pull report schema or status is invalid".to_string());
    }
    let target = validate_registry_target(RegistryTargetInput {
        registry: &report.registry,
        repository: &report.repository,
        reference: &report.reference,
        allow_http: true,
    })?;
    if target.metadata_reference != report.metadata_reference
        || target.signature_reference != report.signature_reference
        || !validate_credential_mode(&report.credential_mode)
    {
        return Err("registry pull report target or credential mode is invalid".to_string());
    }
    verify_pull_manifest_bindings(report)?;
    verify_report_trust_evidence(ReportTrustEvidenceInput {
        trust_domain: &report.trust_domain,
        policy_blake3: &report.policy_blake3,
        verified_signers: &report.verified_signers,
        verified_public_key_blake3: &report.verified_public_key_blake3,
    })?;
    if ![
        &report.layout_blake3,
        &report.projection_blake3,
        &report.import_receipt_blake3,
    ]
    .into_iter()
    .all(|value| is_blake3_hex(value))
    {
        return Err("registry pull report Mantle digest is invalid".to_string());
    }
    verify_pull_report_accounting(report)?;
    verify_pull_report_receipt(report)?;
    assert_eq!(target.repository, report.repository, "verified pull repository must be canonical");
    assert_eq!(
        report.expected_manifest_digest, report.resolved_manifest_digest,
        "verified pull digest must be immutable"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHA256_HEX_LENGTH: usize = 64;
    const BLAKE3_HEX_LENGTH: usize = 64;
    const UPDATE_FIXTURES_ENV: &str = "MANTLE_UPDATE_OCI_REGISTRY_REPORT_FIXTURES";
    const PUSH_FIXTURE_PATH: &str = "tests/fixtures/kernel-bundle-oci/registry-push-report.json";
    const PULL_FIXTURE_PATH: &str = "tests/fixtures/kernel-bundle-oci/registry-pull-report.json";
    const TEST_KEYPAIR: &str =
        "cache.example.com-1:cCta2MEsRNuYCgWYyeRXLyfoFpKhQJKn8gLMeXWAb7vIpRKKo/3JoxJ24OYa3DxT2JVV38KjK/1ywHWuMe2JEw==";
    const SECOND_KEYPAIR: &str =
        "do.not.use:sGPzxuK5WvWPraytx+6sjtaff866sYlfvErE6x0hFEhy5eqe7OVZ8ZMqZ/ME/HaRdKGNGvJkyGKXYTaeA6lR3A==";

    fn sha(byte: char) -> String {
        format!("sha256:{}", std::iter::repeat_n(byte, SHA256_HEX_LENGTH).collect::<String>())
    }

    fn b3(byte: char) -> String {
        std::iter::repeat_n(byte, BLAKE3_HEX_LENGTH).collect::<String>()
    }

    fn target() -> RegistryTarget {
        validate_registry_target(RegistryTargetInput {
            registry: "https://registry.example.test",
            repository: "onix/kernel-bundle",
            reference: "reviewed",
            allow_http: false,
        })
        .expect("target should validate")
    }

    fn test_keypair(encoded: &str) -> crunch_build::KeyPair {
        crunch_build::load_keypair(encoded).expect("test registry keypair should parse")
    }

    fn trust_policy(keypairs: &[crunch_build::KeyPair], minimum_signatures: u16) -> ValidatedRegistryTrustPolicy {
        let policy = RegistryTrustPolicy {
            schema: OCI_REGISTRY_TRUST_POLICY_SCHEMA.to_string(),
            schema_version: TRUST_POLICY_SCHEMA_VERSION,
            trust_domain: "onix-kernel-bundle".to_string(),
            allowed_repositories: vec!["onix/kernel-bundle".to_string()],
            trusted_public_keys: keypairs.iter().map(|keypair| keypair.verifying_key.to_string()).collect(),
            required_signers: keypairs.iter().map(|keypair| keypair.verifying_key.name().to_string()).collect(),
            minimum_signatures,
            revoked_public_key_blake3: Vec::new(),
        };
        validate_registry_trust_policy(policy).expect("test registry trust policy should validate")
    }

    fn push_plan() -> RegistryPushPlan {
        RegistryPushPlan {
            main_manifest_descriptor: OciDescriptor {
                media_type: OCI_MANIFEST_MEDIA_TYPE.to_string(),
                digest: sha('a'),
                size: 10,
                annotations: BTreeMap::new(),
                platform: None,
            },
            main_manifest_bytes: b"manifest-a".to_vec(),
            metadata_manifest_descriptor: OciDescriptor {
                media_type: OCI_MANIFEST_MEDIA_TYPE.to_string(),
                digest: sha('b'),
                size: 10,
                annotations: BTreeMap::new(),
                platform: None,
            },
            metadata_manifest_bytes: b"metadata-b".to_vec(),
            blobs: vec![RegistryBlob {
                digest: sha('c'),
                bytes: b"blob-c".to_vec(),
            }],
            layout_blake3: b3('1'),
            projection_blake3: b3('2'),
            export_receipt_blake3: b3('3'),
        }
    }

    #[test]
    fn target_validation_accepts_https_and_explicit_local_http() {
        let secure = target();
        assert_eq!(secure.registry, "https://registry.example.test");
        assert_eq!(secure.metadata_reference, "reviewed.mantle-metadata");
        assert_eq!(secure.signature_reference, "reviewed.mantle-signature");
        let local = validate_registry_target(RegistryTargetInput {
            registry: "http://127.0.0.1:5000/",
            repository: "onix/kernel-bundle",
            reference: "gallery_1",
            allow_http: true,
        })
        .expect("explicit local HTTP should validate");
        assert_eq!(local.registry, "http://127.0.0.1:5000");
        assert_eq!(local.metadata_reference, "gallery_1.mantle-metadata");
        assert_eq!(local.signature_reference, "gallery_1.mantle-signature");
    }

    #[test]
    fn target_validation_rejects_unsafe_authority_and_names() {
        for input in [
            RegistryTargetInput {
                registry: "http://registry.example.test",
                repository: "onix/kernel",
                reference: "tag",
                allow_http: false,
            },
            RegistryTargetInput {
                registry: "https://token@registry.example.test",
                repository: "onix/kernel",
                reference: "tag",
                allow_http: false,
            },
            RegistryTargetInput {
                registry: "https://registry.example.test/prefix",
                repository: "onix/kernel",
                reference: "tag",
                allow_http: false,
            },
            RegistryTargetInput {
                registry: "https://registry.example.test",
                repository: "Onix/kernel",
                reference: "tag",
                allow_http: false,
            },
            RegistryTargetInput {
                registry: "https://registry.example.test",
                repository: "onix/kernel",
                reference: "bad:tag",
                allow_http: false,
            },
        ] {
            assert!(validate_registry_target(input).is_err());
        }
    }

    #[test]
    fn registry_urls_confine_uploads_to_the_configured_authority() {
        let target = target();
        assert_eq!(registry_v2_url(&target), "https://registry.example.test/v2/");
        assert_eq!(
            registry_manifest_url(&target, "reviewed").unwrap(),
            "https://registry.example.test/v2/onix/kernel-bundle/manifests/reviewed"
        );
        let digest = sha('a');
        let finalized = finalize_upload_url(FinalizeUploadInput {
            target: &target,
            location: "/v2/onix/kernel-bundle/blobs/uploads/upload-1?state=bounded",
            digest: &digest,
        })
        .unwrap();
        assert!(finalized.contains("state=bounded"));
        assert!(finalized.contains("digest=sha256%3A"));
        assert!(
            finalize_upload_url(FinalizeUploadInput {
                target: &target,
                location: "https://evil.example.test/upload",
                digest: &digest,
            })
            .is_err()
        );
        assert!(
            finalize_upload_url(FinalizeUploadInput {
                target: &target,
                location: "/v2/other/blobs/uploads/upload-1",
                digest: &digest,
            })
            .is_err()
        );
        assert!(
            finalize_upload_url(FinalizeUploadInput {
                target: &target,
                location: "/v2/onix/kernel-bundle/blobs/uploads/upload-1?digest=sha256%3Aevil",
                digest: &digest,
            })
            .is_err()
        );
    }

    #[test]
    fn push_report_binds_accounting_and_rejects_receipt_tamper() {
        let target = target();
        let plan = push_plan();
        let keys = vec![test_keypair(TEST_KEYPAIR)];
        let policy = trust_policy(&keys, 1);
        let signature_plan = build_registry_signature_plan(&plan, &policy, &keys).unwrap();
        let report = build_push_report(PushReportFacts {
            target: &target,
            plan: &plan,
            signature_plan: &signature_plan,
            uploaded_blobs: 2,
            reused_blobs: 3,
            transferred_bytes: 42,
            credential_mode: CREDENTIAL_MODE_BEARER_FILE,
        })
        .expect("push report should build");
        assert_eq!(report.manifest_digest, sha('a'));
        assert_eq!(report.uploaded_blobs + report.reused_blobs, 5);
        let mut tampered = report.clone();
        tampered.transferred_bytes = tampered.transferred_bytes.saturating_add(1);
        assert!(verify_push_report(&tampered).is_err());
        let mut oversized = report;
        oversized.uploaded_blobs = REGISTRY_RECEIPT_BLOB_COUNT_MAX.saturating_add(1);
        assert!(verify_push_report(&oversized).is_err());
    }

    #[test]
    fn pull_report_requires_immutable_digest_and_admitted_import() {
        let target = target();
        let plan = push_plan();
        let mut report = OciRegistryPullReport {
            schema: OCI_REGISTRY_PULL_REPORT_SCHEMA.to_string(),
            schema_version: REGISTRY_SCHEMA_VERSION,
            pulled: true,
            registry: target.registry.clone(),
            repository: target.repository.clone(),
            reference: target.reference.clone(),
            metadata_reference: target.metadata_reference.clone(),
            signature_reference: target.signature_reference.clone(),
            expected_manifest_digest: plan.main_manifest_descriptor.digest.clone(),
            resolved_manifest_digest: plan.main_manifest_descriptor.digest,
            expected_metadata_manifest_digest: plan.metadata_manifest_descriptor.digest.clone(),
            resolved_metadata_manifest_digest: plan.metadata_manifest_descriptor.digest,
            expected_signature_manifest_digest: sha('c'),
            resolved_signature_manifest_digest: sha('c'),
            trust_domain: "onix-kernel-bundle".to_string(),
            policy_blake3: b3('5'),
            verified_signers: vec!["cache.example.com-1".to_string()],
            verified_public_key_blake3: vec![b3('6')],
            layout_blake3: plan.layout_blake3,
            projection_blake3: plan.projection_blake3,
            downloaded_blobs: 4,
            downloaded_bytes: 128,
            credential_mode: CREDENTIAL_MODE_ANONYMOUS.to_string(),
            import_state: "admitted".to_string(),
            imported_objects: 3,
            import_receipt_blake3: b3('4'),
            non_claims: required_non_claims(),
            receipt_blake3: String::new(),
        };
        report.receipt_blake3 = receipt_hash(REGISTRY_PULL_RECEIPT_DOMAIN, &report).unwrap();
        verify_pull_report(&report).expect("pull report should verify");
        let mut oversized = report.clone();
        oversized.downloaded_bytes = REGISTRY_RECEIPT_TRANSFER_BYTES_MAX.saturating_add(1);
        assert!(verify_pull_report(&oversized).is_err());
        report.resolved_manifest_digest = sha('f');
        assert!(verify_pull_report(&report).is_err());
    }

    #[test]
    fn signature_plan_satisfies_two_key_policy_and_binds_both_manifest_digests() {
        let plan = push_plan();
        let keys = vec![test_keypair(TEST_KEYPAIR), test_keypair(SECOND_KEYPAIR)];
        let policy = trust_policy(&keys, 2);
        let signature_plan = build_registry_signature_plan(&plan, &policy, &keys).unwrap();
        let (_, verification) = verify_registry_signature_document(RegistrySignatureVerificationInput {
            bytes: &signature_plan.document_bytes,
            manifest_digest: &plan.main_manifest_descriptor.digest,
            metadata_manifest_digest: &plan.metadata_manifest_descriptor.digest,
            policy: &policy,
        })
        .unwrap();
        assert_eq!(verification.verified_signers.len(), 2);
        assert_eq!(verification.verified_public_key_blake3.len(), 2);
        assert_eq!(signature_plan.manifest.subject, plan.main_manifest_descriptor);
        assert_eq!(
            signature_plan.manifest.annotations.get(SIGNATURE_METADATA_DIGEST_ANNOTATION),
            Some(&plan.metadata_manifest_descriptor.digest)
        );
        assert!(
            verify_registry_signature_document(RegistrySignatureVerificationInput {
                bytes: &signature_plan.document_bytes,
                manifest_digest: &sha('f'),
                metadata_manifest_digest: &plan.metadata_manifest_descriptor.digest,
                policy: &policy,
            })
            .is_err()
        );
    }

    #[test]
    fn same_name_key_rotation_counts_distinct_keys_without_inflating_required_signers() {
        let plan = push_plan();
        let (_, second_material) = SECOND_KEYPAIR.split_once(':').unwrap();
        let rotated = format!("cache.example.com-1:{second_material}");
        let keys = vec![test_keypair(TEST_KEYPAIR), test_keypair(&rotated)];
        let policy = validate_registry_trust_policy(RegistryTrustPolicy {
            schema: OCI_REGISTRY_TRUST_POLICY_SCHEMA.to_string(),
            schema_version: TRUST_POLICY_SCHEMA_VERSION,
            trust_domain: "onix-kernel-bundle".to_string(),
            allowed_repositories: vec!["onix/kernel-bundle".to_string()],
            trusted_public_keys: keys.iter().map(|keypair| keypair.verifying_key.to_string()).collect(),
            required_signers: vec!["cache.example.com-1".to_string()],
            minimum_signatures: 2,
            revoked_public_key_blake3: Vec::new(),
        })
        .unwrap();
        let signature_plan = build_registry_signature_plan(&plan, &policy, &keys).unwrap();
        assert_eq!(signature_plan.verification.verified_signers, vec!["cache.example.com-1"]);
        assert_eq!(signature_plan.verification.verified_public_key_blake3.len(), 2);
        assert!(build_registry_signature_plan(&plan, &policy, &keys[..1]).is_err());

        let (_, duplicate_material) = TEST_KEYPAIR.split_once(':').unwrap();
        let duplicate_key = test_keypair(&format!("rotation-alias:{duplicate_material}"));
        let duplicate_policy = RegistryTrustPolicy {
            schema: OCI_REGISTRY_TRUST_POLICY_SCHEMA.to_string(),
            schema_version: TRUST_POLICY_SCHEMA_VERSION,
            trust_domain: "onix-kernel-bundle".to_string(),
            allowed_repositories: vec!["onix/kernel-bundle".to_string()],
            trusted_public_keys: vec![
                keys[0].verifying_key.to_string(),
                duplicate_key.verifying_key.to_string(),
            ],
            required_signers: vec!["cache.example.com-1".to_string(), "rotation-alias".to_string()],
            minimum_signatures: 2,
            revoked_public_key_blake3: Vec::new(),
        };
        assert!(validate_registry_trust_policy(duplicate_policy).is_err());
    }

    #[test]
    fn signature_verification_rejects_unknown_revoked_domain_and_duplicate_evidence() {
        let plan = push_plan();
        let signer = vec![test_keypair(TEST_KEYPAIR)];
        let policy = trust_policy(&signer, 1);
        let signature_plan = build_registry_signature_plan(&plan, &policy, &signer).unwrap();

        let unknown = vec![test_keypair(SECOND_KEYPAIR)];
        let unknown_policy = trust_policy(&unknown, 1);
        assert!(
            verify_registry_signature_document(RegistrySignatureVerificationInput {
                bytes: &signature_plan.document_bytes,
                manifest_digest: &plan.main_manifest_descriptor.digest,
                metadata_manifest_digest: &plan.metadata_manifest_descriptor.digest,
                policy: &unknown_policy,
            })
            .is_err()
        );

        let mut revoked_policy = policy.policy.clone();
        revoked_policy.revoked_public_key_blake3 = vec![registry_public_key_blake3(&signer[0].verifying_key)];
        assert!(validate_registry_trust_policy(revoked_policy).is_err());

        let mut wrong_domain = signature_plan.document.clone();
        wrong_domain.statement.trust_domain = "another-domain".to_string();
        let wrong_domain_bytes = canonical_json(&wrong_domain).unwrap();
        assert!(
            verify_registry_signature_document(RegistrySignatureVerificationInput {
                bytes: &wrong_domain_bytes,
                manifest_digest: &plan.main_manifest_descriptor.digest,
                metadata_manifest_digest: &plan.metadata_manifest_descriptor.digest,
                policy: &policy,
            })
            .is_err()
        );

        let mut duplicate = signature_plan.document;
        duplicate.signatures.push(duplicate.signatures[0].clone());
        let duplicate_bytes = canonical_json(&duplicate).unwrap();
        assert!(
            verify_registry_signature_document(RegistrySignatureVerificationInput {
                bytes: &duplicate_bytes,
                manifest_digest: &plan.main_manifest_descriptor.digest,
                metadata_manifest_digest: &plan.metadata_manifest_descriptor.digest,
                policy: &policy,
            })
            .is_err()
        );
    }

    #[test]
    fn signature_manifest_rejects_subject_metadata_and_domain_drift() {
        let plan = push_plan();
        let keys = vec![test_keypair(TEST_KEYPAIR)];
        let policy = trust_policy(&keys, 1);
        let signature_plan = build_registry_signature_plan(&plan, &policy, &keys).unwrap();
        inspect_signature_manifest(
            &signature_plan.manifest_bytes,
            &signature_plan.manifest_descriptor.digest,
            &plan.main_manifest_descriptor,
            &plan.metadata_manifest_descriptor.digest,
            &policy,
        )
        .unwrap();

        let mut wrong_config = signature_plan.manifest.clone();
        wrong_config.config.digest = sha('e');
        let wrong_config_bytes = canonical_json(&wrong_config).unwrap();
        let wrong_config_digest = sha256_digest(&wrong_config_bytes);
        assert!(
            inspect_signature_manifest(
                &wrong_config_bytes,
                &wrong_config_digest,
                &plan.main_manifest_descriptor,
                &plan.metadata_manifest_descriptor.digest,
                &policy,
            )
            .is_err()
        );

        let mut drifted = signature_plan.manifest;
        drifted.annotations.insert(SIGNATURE_METADATA_DIGEST_ANNOTATION.to_string(), sha('f'));
        let drifted_bytes = canonical_json(&drifted).unwrap();
        let drifted_digest = sha256_digest(&drifted_bytes);
        assert!(
            inspect_signature_manifest(
                &drifted_bytes,
                &drifted_digest,
                &plan.main_manifest_descriptor,
                &plan.metadata_manifest_descriptor.digest,
                &policy,
            )
            .is_err()
        );
    }

    fn update_or_assert_fixture(path: &str, value: &impl Serialize) {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
        let mut bytes = serde_json::to_vec_pretty(value).expect("registry fixture should serialize");
        bytes.push(b'\n');
        if std::env::var_os(UPDATE_FIXTURES_ENV).is_some() {
            std::fs::write(&path, &bytes).unwrap_or_else(|error| panic!("writing {}: {error}", path.display()));
        }
        let expected = std::fs::read(&path).unwrap_or_else(|error| panic!("reading {}: {error}", path.display()));
        assert_eq!(bytes, expected, "registry fixture drifted: {}", path.display());
    }

    #[test]
    fn registry_report_golden_fixtures_match_rust_serialization() {
        let target = target();
        let plan = push_plan();
        let keys = vec![test_keypair(TEST_KEYPAIR)];
        let policy = trust_policy(&keys, 1);
        let signature_plan = build_registry_signature_plan(&plan, &policy, &keys).unwrap();
        let push = build_push_report(PushReportFacts {
            target: &target,
            plan: &plan,
            signature_plan: &signature_plan,
            uploaded_blobs: 2,
            reused_blobs: 3,
            transferred_bytes: 42,
            credential_mode: CREDENTIAL_MODE_BEARER_FILE,
        })
        .unwrap();
        let mut pull = OciRegistryPullReport {
            schema: OCI_REGISTRY_PULL_REPORT_SCHEMA.to_string(),
            schema_version: REGISTRY_SCHEMA_VERSION,
            pulled: true,
            registry: target.registry.clone(),
            repository: target.repository.clone(),
            reference: target.reference.clone(),
            metadata_reference: target.metadata_reference.clone(),
            signature_reference: target.signature_reference.clone(),
            expected_manifest_digest: plan.main_manifest_descriptor.digest.clone(),
            resolved_manifest_digest: plan.main_manifest_descriptor.digest.clone(),
            expected_metadata_manifest_digest: plan.metadata_manifest_descriptor.digest.clone(),
            resolved_metadata_manifest_digest: plan.metadata_manifest_descriptor.digest.clone(),
            expected_signature_manifest_digest: signature_plan.manifest_descriptor.digest.clone(),
            resolved_signature_manifest_digest: signature_plan.manifest_descriptor.digest.clone(),
            trust_domain: signature_plan.verification.trust_domain.clone(),
            policy_blake3: signature_plan.verification.policy_blake3.clone(),
            verified_signers: signature_plan.verification.verified_signers.clone(),
            verified_public_key_blake3: signature_plan.verification.verified_public_key_blake3.clone(),
            layout_blake3: plan.layout_blake3.clone(),
            projection_blake3: plan.projection_blake3.clone(),
            downloaded_blobs: 6,
            downloaded_bytes: 256,
            credential_mode: CREDENTIAL_MODE_BEARER_FILE.to_string(),
            import_state: "admitted".to_string(),
            imported_objects: 4,
            import_receipt_blake3: b3('4'),
            non_claims: required_non_claims(),
            receipt_blake3: String::new(),
        };
        pull.receipt_blake3 = receipt_hash(REGISTRY_PULL_RECEIPT_DOMAIN, &pull).unwrap();
        verify_pull_report(&pull).unwrap();
        update_or_assert_fixture(PUSH_FIXTURE_PATH, &push);
        update_or_assert_fixture(PULL_FIXTURE_PATH, &pull);
    }

    #[test]
    fn metadata_manifest_accepts_exact_shape_and_rejects_subject_role_media_or_annotation_drift() {
        let main_bytes = b"main-manifest";
        let main = descriptor(OCI_MANIFEST_MEDIA_TYPE, main_bytes).unwrap();
        let layout_blake3 = b3('1');
        let projection_blake3 = b3('2');
        let (mut manifest, _, _) = build_metadata_manifest(MetadataManifestInput {
            main: &main,
            oci_layout_bytes: b"layout",
            index_bytes: b"index",
            export_report_bytes: b"report",
            layout_blake3: &layout_blake3,
            projection_blake3: &projection_blake3,
        })
        .unwrap();
        let valid_bytes = canonical_json(&manifest).unwrap();
        let valid_digest = sha256_digest(&valid_bytes);
        inspect_metadata_manifest(&valid_bytes, &valid_digest, &main).unwrap();

        let mut wrong_config = manifest.clone();
        wrong_config.config.digest = sha('e');
        let wrong_config_bytes = canonical_json(&wrong_config).unwrap();
        let wrong_config_digest = sha256_digest(&wrong_config_bytes);
        assert!(inspect_metadata_manifest(&wrong_config_bytes, &wrong_config_digest, &main).is_err());

        let mut wrong_subject = manifest.clone();
        wrong_subject.subject.digest = sha('f');
        let wrong_subject_bytes = canonical_json(&wrong_subject).unwrap();
        let wrong_subject_digest = sha256_digest(&wrong_subject_bytes);
        assert!(inspect_metadata_manifest(&wrong_subject_bytes, &wrong_subject_digest, &main).is_err());

        manifest.layers[1].annotations = manifest.layers[0].annotations.clone();
        let duplicate = canonical_json(&manifest).unwrap();
        let duplicate_digest = sha256_digest(&duplicate);
        assert!(inspect_metadata_manifest(&duplicate, &duplicate_digest, &main).is_err());

        let mut wrong_media = wrong_subject;
        wrong_media.subject = main.clone();
        wrong_media.layers[0].media_type = MANTLE_OCI_INDEX_MEDIA_TYPE.to_string();
        let wrong_media_bytes = canonical_json(&wrong_media).unwrap();
        let wrong_media_digest = sha256_digest(&wrong_media_bytes);
        assert!(inspect_metadata_manifest(&wrong_media_bytes, &wrong_media_digest, &main).is_err());

        let mut extra_annotation = wrong_media;
        extra_annotation.layers[0].media_type = MANTLE_OCI_LAYOUT_MEDIA_TYPE.to_string();
        extra_annotation.annotations.insert("org.onix.mantle.unexpected".to_string(), b3('3'));
        let extra_annotation_bytes = canonical_json(&extra_annotation).unwrap();
        let extra_annotation_digest = sha256_digest(&extra_annotation_bytes);
        assert!(inspect_metadata_manifest(&extra_annotation_bytes, &extra_annotation_digest, &main).is_err());
    }
}
