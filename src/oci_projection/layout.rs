use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;

use super::ArchivePolicy;
use super::CompressionProfile;
use super::ExportPlan;
use super::FRONTEND_SPEC_HASH_ANNOTATION;
use super::FRONTEND_SPEC_ID_ANNOTATION;
use super::FRONTEND_SPEC_VERSION_ANNOTATION;
use super::IMPORT_HASH_DOMAIN;
use super::ImportedObjectRecord;
use super::LAYOUT_HASH_DOMAIN;
use super::LayerDigestRecord;
use super::LayoutFacts;
use super::MaterializedObject;
use super::OCI_BLOB_MAX_BYTES;
use super::OCI_CONFIG_MEDIA_TYPE;
use super::OCI_EXPORT_REPORT_FILENAME;
use super::OCI_IMPORT_REPORT_SCHEMA;
use super::OCI_INDEX_MEDIA_TYPE;
use super::OCI_LAYER_MAX_COUNT;
use super::OCI_LAYOUT_VERSION;
use super::OCI_MANIFEST_MEDIA_TYPE;
use super::OciDescriptor;
use super::OciImportReport;
use super::OciIndexDocument;
use super::OciLayoutDocument;
use super::OciManifestDocument;
use super::OciPlatform;
use super::OciProjection;
use super::PACK_IDENTITY_ANNOTATION;
use super::PROJECTION_ANNOTATION;
use super::PlannedBlob;
use super::ProjectionIssue;
use super::ROLE_ANNOTATION;
use super::SCHEMA_VERSION;
use super::SOURCE_IDENTITIES_ANNOTATION;
use super::STATUS_ADMITTED;
use super::STATUS_COMPATIBILITY_ONLY;
use super::archive::ArchiveInspection;
use super::archive::ArchiveLimits;
use super::archive::inspect_archive_with_options;
use super::archive::layer_payload;
use super::archive::uncompressed_sha256;
use super::blake3_hex;
use super::contains_overclaim;
use super::contains_sensitive_fragment;
use super::is_sha256_digest;
use super::issue;
use super::report_identity;
use super::safe_text;
use super::sha256_digest;
use super::valid_media_type;
use super::validate_annotations;
use super::verify_export_report;

const ROOTFS_TYPE: &str = "layers";
const MAX_IMPORT_ENTRIES: usize = 16_384;
const MAX_IMPORT_DEPTH: usize = 64;
const METADATA_BLOB_COUNT: usize = 2;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct OciRootFs {
    #[serde(rename = "type")]
    rootfs_type: String,
    diff_ids: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
struct OciConfigLabels {
    #[serde(rename = "Labels")]
    labels: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct OciImageConfig {
    architecture: String,
    os: String,
    rootfs: OciRootFs,
    #[serde(default)]
    config: OciConfigLabels,
}

struct LayerBuildProduct {
    descriptor: OciDescriptor,
    record: LayerDigestRecord,
    blob: PlannedBlob,
    diff_id: String,
}

struct LayerBuildProducts {
    descriptors: Vec<OciDescriptor>,
    records: Vec<LayerDigestRecord>,
    blobs: Vec<PlannedBlob>,
    diff_ids: Vec<String>,
}

struct ParsedManifest<'a> {
    descriptor: OciDescriptor,
    bytes: &'a [u8],
    document: OciManifestDocument,
}

struct ValidatedImage<'a> {
    manifest: ParsedManifest<'a>,
    config_bytes: &'a [u8],
    config: OciImageConfig,
    layer_bytes: Vec<&'a [u8]>,
    layout_blake3: String,
    is_admitted: bool,
}

struct ExportReportRelation<'a> {
    report: &'a super::OciExportReport,
    layout_blake3: &'a str,
    manifest_descriptor: &'a OciDescriptor,
    manifest: &'a OciManifestDocument,
    config: &'a OciImageConfig,
    layer_bytes: &'a [&'a [u8]],
}

struct LayerReportRelation<'a> {
    record: &'a LayerDigestRecord,
    descriptor: &'a OciDescriptor,
    uncompressed_sha256: &'a str,
    bytes: &'a [u8],
    index: usize,
}

struct ImportStateFields {
    state: String,
    frontend_spec: Option<super::FrontendSpecBinding>,
    object_admissions_blake3: Option<String>,
    projection_blake3: Option<String>,
    round_trip: Option<super::RoundTripExpectation>,
}

fn canonical_json<T: Serialize>(value: &T, path: &str) -> Result<Vec<u8>, ProjectionIssue> {
    serde_json::to_vec(value).map_err(|error| issue("json-serialization", path, &error.to_string()))
}

fn descriptor(media_type: &str, bytes: &[u8]) -> OciDescriptor {
    OciDescriptor {
        media_type: media_type.to_string(),
        digest: sha256_digest(bytes),
        size: bytes.len() as u64,
        annotations: BTreeMap::new(),
        platform: None,
    }
}

fn layer_annotations(projection: &OciProjection, index: usize) -> BTreeMap<String, String> {
    let layer = &projection.layers[index];
    let mut annotations = layer.annotations.clone();
    annotations.insert(ROLE_ANNOTATION.to_string(), layer.role.clone());
    annotations.insert(
        SOURCE_IDENTITIES_ANNOTATION.to_string(),
        layer.entries.iter().map(|entry| entry.identity.as_str()).collect::<Vec<_>>().join(","),
    );
    if let Some(identity) = &layer.pack_identity {
        annotations.insert(PACK_IDENTITY_ANNOTATION.to_string(), identity.clone());
    }
    annotations
}

fn config_labels(projection: &OciProjection, projection_blake3: &str) -> BTreeMap<String, String> {
    let mut labels = projection.required_annotations.clone();
    labels.insert(PROJECTION_ANNOTATION.to_string(), projection_blake3.to_string());
    labels.insert(FRONTEND_SPEC_ID_ANNOTATION.to_string(), projection.frontend_spec.id.clone());
    labels.insert(FRONTEND_SPEC_VERSION_ANNOTATION.to_string(), projection.frontend_spec.version.clone());
    labels.insert(FRONTEND_SPEC_HASH_ANNOTATION.to_string(), projection.frontend_spec.hash.clone());
    labels
}

fn deduplicate_blobs(mut blobs: Vec<PlannedBlob>) -> Result<Vec<PlannedBlob>, ProjectionIssue> {
    blobs.sort_by(|left, right| left.digest.cmp(&right.digest));
    for pair in blobs.windows(2) {
        if pair[0].digest == pair[1].digest {
            if pair[0].bytes != pair[1].bytes {
                return Err(issue(
                    "digest-collision",
                    &pair[0].digest,
                    "two distinct blobs share one OCI SHA-256 descriptor",
                ));
            }
        }
    }
    blobs.dedup_by(|left, right| left.digest == right.digest);
    Ok(blobs)
}

fn layout_identity<'a>(
    oci_layout_bytes: &[u8],
    index_bytes: &[u8],
    blobs: impl IntoIterator<Item = (&'a str, &'a [u8])>,
) -> String {
    let mut hasher = blake3::Hasher::new_derive_key(LAYOUT_HASH_DOMAIN);
    for bytes in [oci_layout_bytes, index_bytes] {
        hasher.update(&bytes.len().to_le_bytes());
        hasher.update(bytes);
    }
    for (digest, bytes) in blobs {
        let part_length_bytes = digest.len().saturating_add(bytes.len());
        hasher.update(&part_length_bytes.to_le_bytes());
        hasher.update(digest.as_bytes());
        hasher.update(bytes);
    }
    hasher.finalize().to_hex().to_string()
}

fn build_layer(
    projection: &OciProjection,
    objects: &BTreeMap<String, MaterializedObject>,
    index: usize,
) -> Result<LayerBuildProduct, ProjectionIssue> {
    let layer = &projection.layers[index];
    let payload = layer_payload(layer, objects, &projection.archive_policy, ArchiveLimits {
        max_depth: projection.bounds.max_depth,
        max_entries: projection.bounds.max_entries_per_layer,
    })?;
    let blob_sha256 = sha256_digest(&payload.blob);
    let uncompressed_sha256 = sha256_digest(&payload.uncompressed);
    let descriptor = OciDescriptor {
        media_type: layer.media_type.clone(),
        digest: blob_sha256.clone(),
        size: payload.blob.len() as u64,
        annotations: layer_annotations(projection, index),
        platform: None,
    };
    let record = LayerDigestRecord {
        role: layer.role.clone(),
        media_type: layer.media_type.clone(),
        source_object_refs: layer.entries.iter().map(|entry| entry.object_ref.clone()).collect(),
        source_identities: layer.entries.iter().map(|entry| entry.identity.clone()).collect(),
        uncompressed_sha256: uncompressed_sha256.clone(),
        blob_sha256: blob_sha256.clone(),
        blob_blake3: blake3_hex(&payload.blob),
        size_bytes: payload.blob.len() as u64,
        canonical_archive: payload.canonical_archive,
        compression: if payload.canonical_archive {
            projection.archive_policy.compression
        } else {
            CompressionProfile::None
        },
    };
    debug_assert_eq!(descriptor.digest, record.blob_sha256);
    debug_assert_eq!(descriptor.size, record.size_bytes);
    Ok(LayerBuildProduct {
        descriptor,
        record,
        blob: PlannedBlob {
            digest: blob_sha256,
            bytes: payload.blob,
        },
        diff_id: uncompressed_sha256,
    })
}

fn build_layers(
    projection: &OciProjection,
    objects: &BTreeMap<String, MaterializedObject>,
) -> Result<LayerBuildProducts, ProjectionIssue> {
    let layer_count = projection.layers.len();
    let mut products = LayerBuildProducts {
        descriptors: Vec::with_capacity(layer_count),
        records: Vec::with_capacity(layer_count),
        blobs: Vec::with_capacity(layer_count),
        diff_ids: Vec::with_capacity(layer_count),
    };
    let mut seen_layer_digests = BTreeSet::new();
    let mut total_blob_bytes = 0_u64;
    for index in 0..layer_count {
        let product = build_layer(projection, objects, index)?;
        total_blob_bytes = total_blob_bytes.saturating_add(product.descriptor.size);
        if total_blob_bytes > projection.bounds.max_total_bytes {
            return Err(issue(
                "layer-byte-bound",
                &projection.layers[index].role,
                "projected layer bytes exceed the admitted total bound",
            ));
        }
        if !seen_layer_digests.insert(product.descriptor.digest.clone()) {
            return Err(issue(
                "duplicate-layer-descriptor",
                &projection.layers[index].role,
                "OCI layer descriptors must be unique",
            ));
        }
        products.descriptors.push(product.descriptor);
        products.records.push(product.record);
        products.blobs.push(product.blob);
        products.diff_ids.push(product.diff_id);
    }
    debug_assert_eq!(products.descriptors.len(), layer_count);
    debug_assert_eq!(products.records.len(), layer_count);
    Ok(products)
}

fn build_config_blob(
    projection: &OciProjection,
    projection_blake3: &str,
    diff_ids: Vec<String>,
) -> Result<(OciDescriptor, PlannedBlob), ProjectionIssue> {
    let config = OciImageConfig {
        architecture: projection.platform.architecture.clone(),
        os: projection.platform.os.clone(),
        rootfs: OciRootFs {
            rootfs_type: ROOTFS_TYPE.to_string(),
            diff_ids,
        },
        config: OciConfigLabels {
            labels: config_labels(projection, projection_blake3),
        },
    };
    let bytes = canonical_json(&config, "config")?;
    let config_descriptor = descriptor(OCI_CONFIG_MEDIA_TYPE, &bytes);
    debug_assert_eq!(config_descriptor.size, bytes.len() as u64);
    debug_assert_eq!(config_descriptor.digest, sha256_digest(&bytes));
    let config_blob = PlannedBlob {
        digest: config_descriptor.digest.clone(),
        bytes,
    };
    Ok((config_descriptor, config_blob))
}

fn build_manifest_blob(
    projection: &OciProjection,
    config_descriptor: &OciDescriptor,
    layer_descriptors: Vec<OciDescriptor>,
) -> Result<(OciDescriptor, PlannedBlob), ProjectionIssue> {
    let manifest = OciManifestDocument {
        schema_version: SCHEMA_VERSION,
        media_type: OCI_MANIFEST_MEDIA_TYPE.to_string(),
        config: config_descriptor.clone(),
        layers: layer_descriptors,
        annotations: projection.required_annotations.clone(),
    };
    let bytes = canonical_json(&manifest, "manifest")?;
    let manifest_descriptor = OciDescriptor {
        media_type: OCI_MANIFEST_MEDIA_TYPE.to_string(),
        digest: sha256_digest(&bytes),
        size: bytes.len() as u64,
        annotations: projection.required_annotations.clone(),
        platform: Some(OciPlatform {
            architecture: projection.platform.architecture.clone(),
            os: projection.platform.os.clone(),
        }),
    };
    debug_assert_eq!(manifest_descriptor.size, bytes.len() as u64);
    debug_assert_eq!(manifest_descriptor.digest, sha256_digest(&bytes));
    let manifest_blob = PlannedBlob {
        digest: manifest_descriptor.digest.clone(),
        bytes,
    };
    Ok((manifest_descriptor, manifest_blob))
}

fn build_index_bytes(manifest_descriptor: &OciDescriptor) -> Result<Vec<u8>, ProjectionIssue> {
    canonical_json(
        &OciIndexDocument {
            schema_version: SCHEMA_VERSION,
            media_type: OCI_INDEX_MEDIA_TYPE.to_string(),
            manifests: vec![manifest_descriptor.clone()],
        },
        "index",
    )
}

fn build_oci_layout_bytes() -> Result<Vec<u8>, ProjectionIssue> {
    canonical_json(
        &OciLayoutDocument {
            image_layout_version: OCI_LAYOUT_VERSION.to_string(),
        },
        "oci-layout",
    )
}

pub(super) fn build_layout(
    projection: &OciProjection,
    objects: &BTreeMap<String, MaterializedObject>,
) -> Result<ExportPlan, ProjectionIssue> {
    let projection_blake3 = projection
        .admission
        .projection_blake3
        .clone()
        .ok_or_else(|| issue("missing-projection-hash", "admission", "sealed projection identity is absent"))?;
    let layers = build_layers(projection, objects)?;
    let (config_descriptor, config_blob) = build_config_blob(projection, &projection_blake3, layers.diff_ids)?;
    let (manifest_descriptor, manifest_blob) = build_manifest_blob(projection, &config_descriptor, layers.descriptors)?;
    let index_bytes = build_index_bytes(&manifest_descriptor)?;
    let oci_layout_bytes = build_oci_layout_bytes()?;
    let mut blobs = layers.blobs;
    blobs.reserve(METADATA_BLOB_COUNT);
    blobs.push(config_blob);
    blobs.push(manifest_blob);
    let blobs = deduplicate_blobs(blobs)?;
    debug_assert!(blobs.iter().any(|blob| blob.digest == config_descriptor.digest));
    debug_assert!(blobs.iter().any(|blob| blob.digest == manifest_descriptor.digest));
    let layout_blake3 = layout_identity(
        &oci_layout_bytes,
        &index_bytes,
        blobs.iter().map(|blob| (blob.digest.as_str(), blob.bytes.as_slice())),
    );
    Ok(ExportPlan {
        projection: projection.clone(),
        projection_blake3,
        oci_layout_bytes,
        index_bytes,
        blobs,
        layers: layers.records,
        manifest_descriptor,
        config_descriptor,
        layout_blake3,
    })
}

fn parse_json<T: for<'de> Deserialize<'de>>(bytes: &[u8], path: &str) -> Result<T, ProjectionIssue> {
    serde_json::from_slice(bytes).map_err(|error| issue("invalid-json", path, &error.to_string()))
}

fn descriptor_bytes<'a>(
    descriptor: &OciDescriptor,
    blobs: &'a BTreeMap<String, Vec<u8>>,
    path: &str,
) -> Result<&'a [u8], ProjectionIssue> {
    if !is_sha256_digest(&descriptor.digest) {
        return Err(issue("invalid-descriptor-digest", path, "descriptor is not a lowercase SHA-256 digest"));
    }
    if descriptor.size > OCI_BLOB_MAX_BYTES {
        return Err(issue("descriptor-size-bound", path, "descriptor exceeds the OCI blob bound"));
    }
    let bytes = blobs
        .get(&descriptor.digest)
        .ok_or_else(|| issue("missing-descriptor-blob", path, "descriptor blob is absent"))?;
    if descriptor.size != bytes.len() as u64 || descriptor.digest != sha256_digest(bytes) {
        return Err(issue("descriptor-mismatch", path, "descriptor size or SHA-256 disagrees with exact blob bytes"));
    }
    Ok(bytes)
}

fn validate_config(
    descriptor: &OciDescriptor,
    bytes: &[u8],
    layer_count: usize,
) -> Result<OciImageConfig, ProjectionIssue> {
    if descriptor.media_type != OCI_CONFIG_MEDIA_TYPE {
        return Err(issue("config-media-type", "manifest.config", "OCI config media type is unsupported"));
    }
    let config: OciImageConfig = parse_json(bytes, "config")?;
    if !safe_text(&config.architecture) || !safe_text(&config.os) {
        return Err(issue("invalid-config-platform", "config", "OCI config platform values must be bounded safe text"));
    }
    if config.rootfs.rootfs_type != ROOTFS_TYPE || config.rootfs.diff_ids.len() != layer_count {
        return Err(issue("rootfs-shape", "config.rootfs", "rootfs diff-id count must equal the layer count"));
    }
    if config.rootfs.diff_ids.iter().any(|digest| !is_sha256_digest(digest)) {
        return Err(issue("invalid-diff-id", "config.rootfs.diff_ids", "rootfs diff ids must be SHA-256 digests"));
    }
    Ok(config)
}

fn index_has_supported_shape(index: &OciIndexDocument) -> bool {
    if index.schema_version != SCHEMA_VERSION {
        return false;
    }
    if index.media_type != OCI_INDEX_MEDIA_TYPE {
        return false;
    }
    index.manifests.len() == 1
}

fn manifest_has_supported_shape(manifest: &OciManifestDocument) -> bool {
    if manifest.schema_version != SCHEMA_VERSION {
        return false;
    }
    if manifest.media_type != OCI_MANIFEST_MEDIA_TYPE {
        return false;
    }
    if manifest.layers.is_empty() {
        return false;
    }
    manifest.layers.len() <= OCI_LAYER_MAX_COUNT
}

fn validate_descriptor_closure(
    facts: &LayoutFacts,
    manifest_descriptor: &OciDescriptor,
    manifest: &OciManifestDocument,
) -> Result<(), ProjectionIssue> {
    let mut descriptor_digests = BTreeSet::new();
    for descriptor in std::iter::once(manifest_descriptor)
        .chain(std::iter::once(&manifest.config))
        .chain(manifest.layers.iter())
    {
        if !descriptor_digests.insert(descriptor.digest.clone()) {
            return Err(issue("duplicate-descriptor", "manifest", "layout contains duplicate blob descriptors"));
        }
    }
    if facts.blobs.keys().cloned().collect::<BTreeSet<_>>() != descriptor_digests {
        return Err(issue(
            "descriptor-closure-mismatch",
            "blobs",
            "blob facts must exactly equal the referenced descriptor closure",
        ));
    }
    Ok(())
}

fn parse_manifest(facts: &LayoutFacts) -> Result<ParsedManifest<'_>, ProjectionIssue> {
    let layout: OciLayoutDocument = parse_json(&facts.oci_layout_bytes, "oci-layout")?;
    if layout.image_layout_version != OCI_LAYOUT_VERSION {
        return Err(issue("oci-layout-version", "oci-layout", "OCI layout version is unsupported"));
    }
    let index: OciIndexDocument = parse_json(&facts.index_bytes, "index.json")?;
    if !index_has_supported_shape(&index) {
        return Err(issue(
            "oci-index-shape",
            "index.json",
            "OCI index must contain exactly one supported image manifest",
        ));
    }
    let manifest_descriptor = index.manifests.first().cloned().ok_or_else(|| {
        issue("oci-index-shape", "index.json", "OCI index must contain exactly one supported image manifest")
    })?;
    let mut annotation_issues = Vec::new();
    validate_annotations(&manifest_descriptor.annotations, "index.manifests[0].annotations", &mut annotation_issues);
    if let Some(annotation_issue) = annotation_issues.into_iter().next() {
        return Err(annotation_issue);
    }
    if manifest_descriptor.media_type != OCI_MANIFEST_MEDIA_TYPE {
        return Err(issue("manifest-media-type", "index.manifests[0]", "OCI manifest media type is unsupported"));
    }
    let manifest_bytes = descriptor_bytes(&manifest_descriptor, &facts.blobs, "manifest")?;
    let manifest: OciManifestDocument = parse_json(manifest_bytes, "manifest")?;
    if !manifest_has_supported_shape(&manifest) {
        return Err(issue("manifest-shape", "manifest", "manifest shape or layer count is unsupported"));
    }
    validate_descriptor_closure(facts, &manifest_descriptor, &manifest)?;
    debug_assert_eq!(index.manifests.len(), 1);
    debug_assert_eq!(manifest.schema_version, SCHEMA_VERSION);
    Ok(ParsedManifest {
        descriptor: manifest_descriptor,
        bytes: manifest_bytes,
        document: manifest,
    })
}

fn export_layout_mismatch() -> ProjectionIssue {
    issue(
        "export-layout-mismatch",
        OCI_EXPORT_REPORT_FILENAME,
        "export report does not bind this exact layout, config, and projection",
    )
}

fn validate_report_binding(relation: &ExportReportRelation<'_>) -> Result<(), ProjectionIssue> {
    if relation.report.layout_blake3 != relation.layout_blake3 {
        return Err(export_layout_mismatch());
    }
    if &relation.report.manifest_descriptor != relation.manifest_descriptor {
        return Err(export_layout_mismatch());
    }
    if relation.report.config_descriptor != relation.manifest.config {
        return Err(export_layout_mismatch());
    }
    let labels = &relation.config.config.labels;
    if labels.get(PROJECTION_ANNOTATION) != Some(&relation.report.projection_blake3) {
        return Err(export_layout_mismatch());
    }
    if labels.get(FRONTEND_SPEC_ID_ANNOTATION) != Some(&relation.report.frontend_spec.id) {
        return Err(export_layout_mismatch());
    }
    if labels.get(FRONTEND_SPEC_VERSION_ANNOTATION) != Some(&relation.report.frontend_spec.version) {
        return Err(export_layout_mismatch());
    }
    if labels.get(FRONTEND_SPEC_HASH_ANNOTATION) != Some(&relation.report.frontend_spec.hash) {
        return Err(export_layout_mismatch());
    }
    debug_assert_eq!(relation.report.layout_blake3, relation.layout_blake3);
    debug_assert_eq!(&relation.report.manifest_descriptor, relation.manifest_descriptor);
    Ok(())
}

fn validate_layer_record(relation: LayerReportRelation<'_>) -> Result<(), ProjectionIssue> {
    let mismatch = || {
        issue(
            "export-layer-mismatch",
            format!("layers[{}]", relation.index),
            "export report layer does not match its OCI descriptor",
        )
    };
    if relation.record.blob_sha256 != relation.descriptor.digest {
        return Err(mismatch());
    }
    if relation.record.blob_blake3 != blake3_hex(relation.bytes) {
        return Err(mismatch());
    }
    if relation.record.uncompressed_sha256 != relation.uncompressed_sha256 {
        return Err(mismatch());
    }
    if relation.record.size_bytes != relation.descriptor.size {
        return Err(mismatch());
    }
    if relation.record.media_type != relation.descriptor.media_type {
        return Err(mismatch());
    }
    debug_assert_eq!(relation.record.blob_sha256, relation.descriptor.digest);
    debug_assert_eq!(relation.record.size_bytes, relation.descriptor.size);
    Ok(())
}

fn validate_report_relation(relation: ExportReportRelation<'_>) -> Result<(), ProjectionIssue> {
    verify_export_report(relation.report)?;
    let admitted_refs = relation
        .report
        .layers
        .iter()
        .flat_map(|layer| layer.source_object_refs.iter())
        .collect::<BTreeSet<_>>();
    if relation.report.admission_count != admitted_refs.len() {
        return Err(issue(
            "export-admission-count",
            OCI_EXPORT_REPORT_FILENAME,
            "export report admission count does not cover the exact source-object set",
        ));
    }
    validate_report_binding(&relation)?;
    if relation.report.layers.len() != relation.manifest.layers.len() {
        return Err(issue(
            "export-layer-count",
            OCI_EXPORT_REPORT_FILENAME,
            "export report layer count disagrees with the manifest",
        ));
    }
    if relation.layer_bytes.len() != relation.manifest.layers.len() {
        return Err(issue(
            "export-layer-count",
            OCI_EXPORT_REPORT_FILENAME,
            "validated layer bytes disagree with the manifest",
        ));
    }
    for (index, ((record, descriptor), bytes)) in
        relation.report.layers.iter().zip(&relation.manifest.layers).zip(relation.layer_bytes).enumerate()
    {
        validate_layer_record(LayerReportRelation {
            record,
            descriptor,
            uncompressed_sha256: &relation.config.rootfs.diff_ids[index],
            bytes,
            index,
        })?;
    }
    debug_assert_eq!(relation.report.layers.len(), relation.manifest.layers.len());
    debug_assert_eq!(relation.layer_bytes.len(), relation.manifest.layers.len());
    Ok(())
}

fn import_object(
    role: String,
    descriptor: &OciDescriptor,
    bytes: &[u8],
    archive_profile: String,
) -> ImportedObjectRecord {
    ImportedObjectRecord {
        role,
        media_type: descriptor.media_type.clone(),
        oci_sha256: descriptor.digest.clone(),
        blob_blake3: blake3_hex(bytes),
        size_bytes: bytes.len() as u64,
        archive_profile,
        artifact_ref: None,
    }
}

fn validate_platform(manifest: &ParsedManifest<'_>, config: &OciImageConfig) -> Result<(), ProjectionIssue> {
    let is_mismatched = manifest
        .descriptor
        .platform
        .as_ref()
        .is_some_and(|platform| platform.architecture != config.architecture || platform.os != config.os);
    if is_mismatched {
        return Err(issue("platform-mismatch", "index.manifests[0].platform", "index and config platforms disagree"));
    }
    Ok(())
}

fn validate_image(facts: &LayoutFacts) -> Result<ValidatedImage<'_>, ProjectionIssue> {
    let manifest = parse_manifest(facts)?;
    let config_bytes = descriptor_bytes(&manifest.document.config, &facts.blobs, "config")?;
    let config = validate_config(&manifest.document.config, config_bytes, manifest.document.layers.len())?;
    validate_platform(&manifest, &config)?;
    let layer_bytes = manifest
        .document
        .layers
        .iter()
        .enumerate()
        .map(|(index, descriptor)| descriptor_bytes(descriptor, &facts.blobs, &format!("layers[{index}]")))
        .collect::<Result<Vec<_>, _>>()?;
    let layout_blake3 = layout_identity(
        &facts.oci_layout_bytes,
        &facts.index_bytes,
        facts.blobs.iter().map(|(digest, bytes)| (digest.as_str(), bytes.as_slice())),
    );
    let is_admitted = if let Some(export_receipt) = &facts.export_report {
        validate_report_relation(ExportReportRelation {
            report: export_receipt,
            layout_blake3: &layout_blake3,
            manifest_descriptor: &manifest.descriptor,
            manifest: &manifest.document,
            config: &config,
            layer_bytes: &layer_bytes,
        })?;
        true
    } else {
        false
    };
    debug_assert_eq!(layer_bytes.len(), manifest.document.layers.len());
    debug_assert_eq!(config.rootfs.diff_ids.len(), manifest.document.layers.len());
    Ok(ValidatedImage {
        manifest,
        config_bytes,
        config,
        layer_bytes,
        layout_blake3,
        is_admitted,
    })
}

fn validate_layer_role(role: &str, index: usize) -> Result<(), ProjectionIssue> {
    let path = || format!("layers[{index}].annotations");
    if !safe_text(role) {
        return Err(issue("unsafe-layer-role", path(), "layer role is not redaction-safe bounded text"));
    }
    if role.starts_with('/') {
        return Err(issue("unsafe-layer-role", path(), "layer role is not redaction-safe bounded text"));
    }
    if contains_sensitive_fragment(role) {
        return Err(issue("unsafe-layer-role", path(), "layer role is not redaction-safe bounded text"));
    }
    if contains_overclaim(role) {
        return Err(issue("unsafe-layer-role", path(), "layer role is not redaction-safe bounded text"));
    }
    Ok(())
}

fn import_layers(image: &ValidatedImage<'_>) -> Result<Vec<ImportedObjectRecord>, ProjectionIssue> {
    let layer_count = image.manifest.document.layers.len();
    let mut objects = Vec::with_capacity(layer_count);
    for (index, (descriptor, bytes)) in image.manifest.document.layers.iter().zip(&image.layer_bytes).enumerate() {
        if !valid_media_type(&descriptor.media_type) {
            return Err(issue(
                "invalid-media-type",
                format!("layers[{index}].media_type"),
                "layer media type is invalid",
            ));
        }
        if uncompressed_sha256(&descriptor.media_type, bytes, OCI_BLOB_MAX_BYTES)?
            != image.config.rootfs.diff_ids[index]
        {
            return Err(issue(
                "diff-id-mismatch",
                format!("config.rootfs.diff_ids[{index}]"),
                "rootfs diff id does not match the exact uncompressed layer bytes",
            ));
        }
        let archive_profile = inspect_archive_with_options(ArchiveInspection {
            media_type: &descriptor.media_type,
            blob: bytes,
            policy: &ArchivePolicy::default(),
            limits: ArchiveLimits {
                max_depth: MAX_IMPORT_DEPTH,
                max_entries: MAX_IMPORT_ENTRIES,
            },
            max_uncompressed_bytes: OCI_BLOB_MAX_BYTES,
            require_canonical: image.is_admitted,
        })?;
        let role = descriptor
            .annotations
            .get(ROLE_ANNOTATION)
            .cloned()
            .unwrap_or_else(|| format!("external-layer-{index}"));
        validate_layer_role(&role, index)?;
        objects.push(import_object(role, descriptor, bytes, archive_profile));
    }
    debug_assert_eq!(objects.len(), layer_count);
    debug_assert!(objects.iter().all(|object| valid_media_type(&object.media_type)));
    Ok(objects)
}

fn import_state_fields(export_receipt: Option<&super::OciExportReport>) -> ImportStateFields {
    if let Some(export_receipt) = export_receipt {
        ImportStateFields {
            state: STATUS_ADMITTED.to_string(),
            frontend_spec: Some(export_receipt.frontend_spec.clone()),
            object_admissions_blake3: Some(export_receipt.object_admissions_blake3.clone()),
            projection_blake3: Some(export_receipt.projection_blake3.clone()),
            round_trip: Some(export_receipt.round_trip.clone()),
        }
    } else {
        ImportStateFields {
            state: STATUS_COMPATIBILITY_ONLY.to_string(),
            frontend_spec: None,
            object_admissions_blake3: None,
            projection_blake3: None,
            round_trip: None,
        }
    }
}

fn build_import_report(
    facts: &LayoutFacts,
    image: ValidatedImage<'_>,
    mut objects: Vec<ImportedObjectRecord>,
) -> Result<OciImportReport, ProjectionIssue> {
    objects.reserve(METADATA_BLOB_COUNT);
    objects.push(import_object(
        "oci-config".to_string(),
        &image.manifest.document.config,
        image.config_bytes,
        "exact-json".to_string(),
    ));
    objects.push(import_object(
        "oci-manifest".to_string(),
        &image.manifest.descriptor,
        image.manifest.bytes,
        "exact-json".to_string(),
    ));
    let state_fields = import_state_fields(facts.export_report.as_ref());
    let expected_object_count = image.manifest.document.layers.len().saturating_add(METADATA_BLOB_COUNT);
    debug_assert_eq!(objects.len(), expected_object_count);
    let mut output = OciImportReport {
        schema: OCI_IMPORT_REPORT_SCHEMA.to_string(),
        schema_version: SCHEMA_VERSION,
        imported: false,
        state: state_fields.state,
        layout_blake3: image.layout_blake3,
        manifest_descriptor: Some(image.manifest.descriptor),
        objects,
        frontend_spec: state_fields.frontend_spec,
        object_admissions_blake3: state_fields.object_admissions_blake3,
        projection_blake3: state_fields.projection_blake3,
        round_trip: state_fields.round_trip,
        issues: Vec::new(),
        non_claims: vec![
            "CAS admission is not release eligibility".to_string(),
            "archive safety is not kernel compatibility".to_string(),
            "compatibility-only imports do not recover frontend semantics".to_string(),
            "descriptor validation is not signature verification".to_string(),
            "frontend semantics remain external".to_string(),
            "no bootability claim".to_string(),
            "no deployability claim".to_string(),
            "no kernel compatibility decision".to_string(),
            "no registry publication".to_string(),
            "no release eligibility claim".to_string(),
            "no signature policy claim".to_string(),
        ],
        receipt_blake3: String::new(),
    };
    debug_assert!(output.receipt_blake3.is_empty());
    output.receipt_blake3 = report_identity(IMPORT_HASH_DOMAIN, &output)
        .map_err(|error| issue("import-report-identity", "report", &error))?;
    Ok(output)
}

fn validate_layout_inner(facts: &LayoutFacts) -> Result<OciImportReport, ProjectionIssue> {
    let image = validate_image(facts)?;
    let objects = import_layers(&image)?;
    build_import_report(facts, image, objects)
}

pub(super) fn validate_layout(facts: &LayoutFacts) -> Result<OciImportReport, Vec<ProjectionIssue>> {
    validate_layout_inner(facts).map_err(|error| vec![error])
}
