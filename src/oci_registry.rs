//! Pure OCI Distribution planning, metadata linkage, and registry receipts.
//!
//! r[impl kernel_bundle_oci.registry_transport]
//! r[impl kernel_bundle_oci.registry_admission]
//! r[impl kernel_bundle_oci.registry_receipts]

// machine-artifact-public: oci.registry-push-report
// machine-artifact-public: oci.registry-pull-report

use std::collections::BTreeMap;
use std::collections::BTreeSet;

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

pub const OCI_REGISTRY_PUSH_REPORT_SCHEMA: &str = "mantle-oci-registry-push-report-v1";
pub const OCI_REGISTRY_PULL_REPORT_SCHEMA: &str = "mantle-oci-registry-pull-report-v1";
pub const OCI_ARTIFACT_MANIFEST_MEDIA_TYPE: &str = "application/vnd.oci.artifact.manifest.v1+json";
pub const MANTLE_METADATA_ARTIFACT_TYPE: &str = "application/vnd.mantle.oci-metadata.v1";
pub const MANTLE_OCI_LAYOUT_MEDIA_TYPE: &str = "application/vnd.mantle.oci-layout.v1+json";
pub const MANTLE_OCI_INDEX_MEDIA_TYPE: &str = "application/vnd.mantle.oci-index.v1+json";
pub const MANTLE_OCI_EXPORT_REPORT_MEDIA_TYPE: &str = "application/vnd.mantle.oci-export-report.v1+json";
pub const METADATA_ROLE_ANNOTATION: &str = "org.mantle.metadata.role";
pub const METADATA_LAYOUT_ROLE: &str = "oci-layout";
pub const METADATA_INDEX_ROLE: &str = "index";
pub const METADATA_EXPORT_REPORT_ROLE: &str = "export-report";
pub const METADATA_REFERENCE_SUFFIX: &str = ".mantle-metadata";
pub const CREDENTIAL_MODE_ANONYMOUS: &str = "anonymous";
pub const CREDENTIAL_MODE_BEARER_FILE: &str = "explicit-bearer-file";

const REGISTRY_SCHEMA_VERSION: u16 = 1;
const OCI_DOCUMENT_SCHEMA_VERSION: u16 = 2;
const REGISTRY_BYTES_MAX: usize = 2_048;
const REPOSITORY_BYTES_MAX: usize = 255;
const REPOSITORY_SEGMENT_BYTES_MAX: usize = 128;
const TAG_BYTES_MAX: usize = 128;
const TAG_BYTES_MIN: usize = 1;
const METADATA_BLOB_COUNT: usize = 3;
const REGISTRY_RECEIPT_BLOB_COUNT_MAX: u32 = 1_000_000;
const REGISTRY_RECEIPT_OBJECT_COUNT_MAX: u32 = 1_000_000;
const REGISTRY_RECEIPT_TRANSFER_BYTES_MAX: u64 = 1_099_511_627_776;
const REGISTRY_PUSH_RECEIPT_DOMAIN: &str = "mantle/oci/registry-push-report/v1";
const REGISTRY_PULL_RECEIPT_DOMAIN: &str = "mantle/oci/registry-pull-report/v1";
const LAYOUT_DIGEST_ANNOTATION: &str = "org.mantle.layout.blake3";
const PROJECTION_DIGEST_ANNOTATION: &str = "org.mantle.projection.blake3";

const REQUIRED_NON_CLAIMS: &[&str] = &[
    "credential possession is not authorization proof",
    "no bootability claim",
    "no deployability claim",
    "no exactly-once publication",
    "no kernel compatibility decision",
    "no registry trust claim",
    "no release eligibility claim",
    "no signature verification",
    "no tag immutability claim",
    "no upload resumption",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegistryTarget {
    pub registry: String,
    pub repository: String,
    pub reference: String,
    pub metadata_reference: String,
}

pub struct RegistryTargetInput<'a> {
    pub registry: &'a str,
    pub repository: &'a str,
    pub reference: &'a str,
    pub allow_http: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OciArtifactManifest {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u16,
    #[serde(rename = "mediaType")]
    pub media_type: String,
    #[serde(rename = "artifactType")]
    pub artifact_type: String,
    pub blobs: Vec<OciDescriptor>,
    pub subject: OciDescriptor,
    pub annotations: BTreeMap<String, String>,
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
    pub manifest_digest: String,
    pub metadata_manifest_digest: String,
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
    pub expected_manifest_digest: String,
    pub resolved_manifest_digest: String,
    pub expected_metadata_manifest_digest: String,
    pub resolved_metadata_manifest_digest: String,
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
    if !tag_is_valid(&metadata_reference) {
        return Err("registry reference leaves no room for the Mantle metadata suffix".to_string());
    }
    assert_ne!(input.reference, metadata_reference, "metadata reference must be distinct");
    assert!(metadata_reference.ends_with(METADATA_REFERENCE_SUFFIX), "metadata suffix must be preserved");
    Ok(RegistryTarget {
        registry,
        repository,
        reference: input.reference.to_string(),
        metadata_reference,
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
) -> Result<(OciArtifactManifest, Vec<u8>, OciDescriptor), String> {
    let blobs = vec![
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
    let manifest = OciArtifactManifest {
        schema_version: OCI_DOCUMENT_SCHEMA_VERSION,
        media_type: OCI_ARTIFACT_MANIFEST_MEDIA_TYPE.to_string(),
        artifact_type: MANTLE_METADATA_ARTIFACT_TYPE.to_string(),
        blobs,
        subject: input.main.clone(),
        annotations,
    };
    let bytes = canonical_json(&manifest)?;
    let manifest_descriptor = descriptor(OCI_ARTIFACT_MANIFEST_MEDIA_TYPE, &bytes)?;
    assert_eq!(manifest.blobs.len(), METADATA_BLOB_COUNT, "metadata manifest blob count must stay fixed");
    assert_eq!(manifest.subject.digest, input.main.digest, "metadata subject must bind the image manifest");
    Ok((manifest, bytes, manifest_descriptor))
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
    for (value, bytes) in metadata.blobs.iter().zip([
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
) -> Result<(OciArtifactManifest, OciDescriptor), String> {
    if sha256_digest(bytes) != expected_digest {
        return Err("Mantle metadata manifest digest mismatch".to_string());
    }
    let manifest: OciArtifactManifest =
        serde_json::from_slice(bytes).map_err(|error| format!("parsing Mantle metadata manifest: {error}"))?;
    if manifest.schema_version != OCI_DOCUMENT_SCHEMA_VERSION
        || manifest.media_type != OCI_ARTIFACT_MANIFEST_MEDIA_TYPE
        || manifest.artifact_type != MANTLE_METADATA_ARTIFACT_TYPE
    {
        return Err("Mantle metadata manifest version or media type is unsupported".to_string());
    }
    if manifest.subject.digest != main.digest
        || manifest.subject.size != main.size
        || manifest.subject.media_type != main.media_type
    {
        return Err("Mantle metadata subject does not match the resolved image manifest".to_string());
    }
    let roles = manifest.blobs.iter().filter_map(metadata_role).collect::<BTreeSet<_>>();
    let required = BTreeSet::from([METADATA_LAYOUT_ROLE, METADATA_INDEX_ROLE, METADATA_EXPORT_REPORT_ROLE]);
    if manifest.blobs.len() != METADATA_BLOB_COUNT
        || roles != required
        || !manifest.blobs.iter().all(metadata_descriptor_is_valid)
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
    let descriptor = descriptor(OCI_ARTIFACT_MANIFEST_MEDIA_TYPE, bytes)?;
    assert_eq!(descriptor.digest, expected_digest, "verified metadata digest must match expected digest");
    assert_eq!(roles.len(), METADATA_BLOB_COUNT, "verified metadata roles must be unique");
    Ok((manifest, descriptor))
}

pub fn pull_blob_descriptors(
    main_manifest_bytes: &[u8],
    metadata: &OciArtifactManifest,
) -> Result<Vec<OciDescriptor>, String> {
    let manifest: OciManifestDocument = serde_json::from_slice(main_manifest_bytes)
        .map_err(|error| format!("parsing registry image manifest: {error}"))?;
    let descriptor_count_max = OCI_LAYER_MAX_COUNT.saturating_add(METADATA_BLOB_COUNT).saturating_add(1);
    let candidates = std::iter::once(manifest.config)
        .chain(manifest.layers)
        .chain(metadata.blobs.iter().cloned())
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
    manifest: &OciArtifactManifest,
    role: &str,
    downloaded: &BTreeMap<String, Vec<u8>>,
) -> Result<Vec<u8>, String> {
    let descriptor = manifest
        .blobs
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
        manifest_digest: facts.plan.main_manifest_descriptor.digest.clone(),
        metadata_manifest_digest: facts.plan.metadata_manifest_descriptor.digest.clone(),
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
    if target.metadata_reference != report.metadata_reference || !validate_credential_mode(&report.credential_mode) {
        return Err("registry push report target or credential mode is invalid".to_string());
    }
    if !is_sha256_digest(&report.manifest_digest) || !is_sha256_digest(&report.metadata_manifest_digest) {
        return Err("registry push report manifest digest is invalid".to_string());
    }
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
        expected_manifest_digest: facts.expected_manifest_digest.to_string(),
        resolved_manifest_digest: facts.plan.main_manifest_descriptor.digest.clone(),
        expected_metadata_manifest_digest: facts.expected_metadata_manifest_digest.to_string(),
        resolved_metadata_manifest_digest: facts.plan.metadata_manifest_descriptor.digest.clone(),
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
    if target.metadata_reference != report.metadata_reference || !validate_credential_mode(&report.credential_mode) {
        return Err("registry pull report target or credential mode is invalid".to_string());
    }
    if report.expected_manifest_digest != report.resolved_manifest_digest {
        return Err("registry pull report image manifest binding is mutable".to_string());
    }
    if report.expected_metadata_manifest_digest != report.resolved_metadata_manifest_digest {
        return Err("registry pull report metadata manifest binding is mutable".to_string());
    }
    if !is_sha256_digest(&report.expected_manifest_digest)
        || !is_sha256_digest(&report.expected_metadata_manifest_digest)
    {
        return Err("registry pull report immutable manifest binding is invalid".to_string());
    }
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
    let mut canonical = report.clone();
    canonical.receipt_blake3.clear();
    if !is_blake3_hex(&report.receipt_blake3)
        || receipt_hash(REGISTRY_PULL_RECEIPT_DOMAIN, &canonical)? != report.receipt_blake3
    {
        return Err("registry pull report receipt BLAKE3 is invalid".to_string());
    }
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
                media_type: OCI_ARTIFACT_MANIFEST_MEDIA_TYPE.to_string(),
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
        let local = validate_registry_target(RegistryTargetInput {
            registry: "http://127.0.0.1:5000/",
            repository: "onix/kernel-bundle",
            reference: "gallery_1",
            allow_http: true,
        })
        .expect("explicit local HTTP should validate");
        assert_eq!(local.registry, "http://127.0.0.1:5000");
        assert_eq!(local.metadata_reference, "gallery_1.mantle-metadata");
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
        let report = build_push_report(PushReportFacts {
            target: &target,
            plan: &plan,
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
            expected_manifest_digest: plan.main_manifest_descriptor.digest.clone(),
            resolved_manifest_digest: plan.main_manifest_descriptor.digest,
            expected_metadata_manifest_digest: plan.metadata_manifest_descriptor.digest.clone(),
            resolved_metadata_manifest_digest: plan.metadata_manifest_descriptor.digest,
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
        let push = build_push_report(PushReportFacts {
            target: &target,
            plan: &plan,
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
            expected_manifest_digest: plan.main_manifest_descriptor.digest.clone(),
            resolved_manifest_digest: plan.main_manifest_descriptor.digest.clone(),
            expected_metadata_manifest_digest: plan.metadata_manifest_descriptor.digest.clone(),
            resolved_metadata_manifest_digest: plan.metadata_manifest_descriptor.digest.clone(),
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

        let mut wrong_subject = manifest.clone();
        wrong_subject.subject.digest = sha('f');
        let wrong_subject_bytes = canonical_json(&wrong_subject).unwrap();
        let wrong_subject_digest = sha256_digest(&wrong_subject_bytes);
        assert!(inspect_metadata_manifest(&wrong_subject_bytes, &wrong_subject_digest, &main).is_err());

        manifest.blobs[1].annotations = manifest.blobs[0].annotations.clone();
        let duplicate = canonical_json(&manifest).unwrap();
        let duplicate_digest = sha256_digest(&duplicate);
        assert!(inspect_metadata_manifest(&duplicate, &duplicate_digest, &main).is_err());

        let mut wrong_media = wrong_subject;
        wrong_media.subject = main.clone();
        wrong_media.blobs[0].media_type = MANTLE_OCI_INDEX_MEDIA_TYPE.to_string();
        let wrong_media_bytes = canonical_json(&wrong_media).unwrap();
        let wrong_media_digest = sha256_digest(&wrong_media_bytes);
        assert!(inspect_metadata_manifest(&wrong_media_bytes, &wrong_media_digest, &main).is_err());

        let mut extra_annotation = wrong_media;
        extra_annotation.blobs[0].media_type = MANTLE_OCI_LAYOUT_MEDIA_TYPE.to_string();
        extra_annotation.annotations.insert("org.onix.mantle.unexpected".to_string(), b3('3'));
        let extra_annotation_bytes = canonical_json(&extra_annotation).unwrap();
        let extra_annotation_digest = sha256_digest(&extra_annotation_bytes);
        assert!(inspect_metadata_manifest(&extra_annotation_bytes, &extra_annotation_digest, &main).is_err());
    }
}
