use std::collections::BTreeMap;

use super::archive::inspect_archive;
use super::*;

const EXACT_BYTES: &[u8] = b"kernel-image";
const MODULE_BYTES: &[u8] = b"module-bytes";
const TEST_ARCHITECTURE: &str = "x86_64";
const TEST_OS: &str = "linux";
const UPDATE_FIXTURES_ENV: &str = "MANTLE_UPDATE_KERNEL_BUNDLE_OCI_FIXTURES";
const FIXTURE_DIRECTORY: &str = "tests/fixtures/kernel-bundle-oci";
const EXPECTED_IMPORTED_OBJECT_COUNT: usize = 4;
const IMPORTED_REF_DIGEST_MARKERS: [char; EXPECTED_IMPORTED_OBJECT_COUNT] = ['c', 'd', 'e', 'f'];
const COLLIDING_FILE_COUNT: usize = 2;

fn digest_hex(byte: char) -> String {
    std::iter::repeat_n(byte, BLAKE3_HEX_LENGTH).collect()
}

fn artifact_ref(byte: char) -> String {
    format!("{MANTLE_REF_PREFIX}{}", digest_hex(byte))
}

fn required_non_claims() -> Vec<String> {
    REQUIRED_NON_CLAIMS.iter().map(ToString::to_string).collect()
}

fn source_admission(object_ref: String, target_identity: String, spec_hash: &str) -> SourceArtifactAdmission {
    let digest = artifact_ref_digest(&object_ref).expect("fixture ref should be valid").to_string();
    SourceArtifactAdmission {
        spec_id: "kernel-bundles".to_string(),
        spec_version: "1".to_string(),
        spec_hash_algorithm: "blake3".to_string(),
        spec_hash: spec_hash.to_string(),
        validator_kind: "onix-kernel-bundle-v1".to_string(),
        validator_ref: "builtin:onix-kernel-bundle-v1".to_string(),
        artifact_kind: "kernel-bundle-component".to_string(),
        artifact_ref: object_ref,
        artifact_digest: Some(format!("blake3:{digest}")),
        target_identity: Some(target_identity),
        build_root: "fixture-build-root".to_string(),
        validation_result: STATUS_ADMITTED.to_string(),
        no_hidden_fallback: true,
    }
}

fn object_admission(object_ref: String, target_identity: String, spec_hash: &str) -> ProjectionObjectAdmission {
    reduce_source_admission(&source_admission(object_ref, target_identity, spec_hash))
        .expect("source admission should reduce")
}

fn source_admissions_for_projection(projection: &OciProjection) -> SourceAdmissionBundle {
    let admissions = projection
        .object_admissions
        .iter()
        .map(|admission| SourceArtifactAdmission {
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
            build_root: "fixture-build-root".to_string(),
            validation_result: admission.validation_result.clone(),
            no_hidden_fallback: admission.no_hidden_fallback,
        })
        .collect::<Vec<_>>();
    SourceAdmissionBundle {
        schema: SOURCE_ADMISSION_BUNDLE_SCHEMA.to_string(),
        admissions,
    }
}

fn build_export_plan(
    projection: &OciProjection,
    spec: &[u8],
    objects: &BTreeMap<String, MaterializedObject>,
) -> Result<ExportPlan, Vec<ProjectionIssue>> {
    super::build_export_plan(projection, spec, &source_admissions_for_projection(projection), objects)
}

fn projection(spec: &[u8]) -> OciProjection {
    let spec_hash = blake3_hex(spec);
    let exact_identity = format!("onix:component:blake3:{}", digest_hex('1'));
    let module_identity = format!("onix:component:blake3:{}", digest_hex('2'));
    let mut projection = OciProjection {
        schema: OCI_PROJECTION_SCHEMA.to_string(),
        frontend_spec: FrontendSpecBinding {
            id: "kernel-bundles".to_string(),
            version: "1".to_string(),
            hash_algorithm: "blake3".to_string(),
            hash: spec_hash.clone(),
        },
        admission: ProjectionAdmission {
            validation_result: STATUS_ADMITTED.to_string(),
            projection_blake3: None,
            no_hidden_fallback: true,
        },
        platform: Platform {
            architecture: TEST_ARCHITECTURE.to_string(),
            os: TEST_OS.to_string(),
        },
        bounds: ProjectionBounds::default(),
        archive_policy: ArchivePolicy::default(),
        object_admissions: vec![
            object_admission(artifact_ref('a'), exact_identity.clone(), &spec_hash),
            object_admission(artifact_ref('b'), module_identity.clone(), &spec_hash),
        ],
        layers: vec![
            ProjectionLayer {
                role: "base-kernel".to_string(),
                media_type: "application/vnd.onix.kernel.v1".to_string(),
                mode: LayerMode::ExactBlob,
                entries: vec![ProjectionEntry {
                    relative_path: "boot/vmlinuz".to_string(),
                    object_ref: artifact_ref('a'),
                    identity: exact_identity.clone(),
                    size_bytes: EXACT_BYTES.len() as u64,
                }],
                pack_identity: None,
                annotations: BTreeMap::new(),
            },
            ProjectionLayer {
                role: "module-pack".to_string(),
                media_type: "application/vnd.onix.module-pack.v1.tar".to_string(),
                mode: LayerMode::CanonicalArchive,
                entries: vec![ProjectionEntry {
                    relative_path: "usr/lib/modules".to_string(),
                    object_ref: artifact_ref('b'),
                    identity: module_identity.clone(),
                    size_bytes: MODULE_BYTES.len() as u64,
                }],
                pack_identity: Some(format!("onix:module-pack:blake3:{}", digest_hex('3'))),
                annotations: BTreeMap::from([("io.onix.pack.kind".to_string(), "module".to_string())]),
            },
        ],
        required_annotations: BTreeMap::from([
            ("io.multikernel.kbi.id".to_string(), format!("kbi:sha256:{}", digest_hex('7'))),
            ("org.opencontainers.image.title".to_string(), "Onix kernel bundle".to_string()),
            ("org.onix.bundle.schema".to_string(), "onix-kernel-bundle-v1".to_string()),
        ]),
        expected_external_digests: vec![ExternalDigestExpectation {
            role: EXPECTED_DIGEST_ROLE_OCI_LAYER_BLOB.to_string(),
            subject_identity: exact_identity.clone(),
            digest: sha256_digest(EXACT_BYTES),
        }],
        round_trip: RoundTripExpectation {
            kernel_build_identity: format!("onix:kernel-build:blake3:{}", digest_hex('4')),
            bundle_identity: format!("onix:kernel-bundle:blake3:{}", digest_hex('5')),
            manifest_identity: format!("onix:manifest:blake3:{}", digest_hex('6')),
            component_identities: vec![exact_identity, module_identity],
            pack_identities: vec![format!("onix:module-pack:blake3:{}", digest_hex('3'))],
        },
        non_claims: required_non_claims(),
    };
    seal_projection(&mut projection).expect("test projection should seal");
    projection
}

fn objects() -> BTreeMap<String, MaterializedObject> {
    BTreeMap::from([
        (artifact_ref('a'), MaterializedObject {
            artifact_ref: artifact_ref('a'),
            entries: vec![ObjectEntry {
                relative_path: ".".to_string(),
                kind: ObjectEntryKind::File,
                data: EXACT_BYTES.to_vec(),
                link_target: None,
            }],
        }),
        (artifact_ref('b'), MaterializedObject {
            artifact_ref: artifact_ref('b'),
            entries: vec![
                ObjectEntry {
                    relative_path: ".".to_string(),
                    kind: ObjectEntryKind::Directory,
                    data: Vec::new(),
                    link_target: None,
                },
                ObjectEntry {
                    relative_path: "kernel/module.ko".to_string(),
                    kind: ObjectEntryKind::File,
                    data: MODULE_BYTES.to_vec(),
                    link_target: None,
                },
            ],
        }),
    ])
}

fn assert_json_fixture<T: Serialize>(name: &str, value: &T) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_DIRECTORY).join(name);
    let mut rendered = serde_json::to_vec_pretty(value).expect("golden value should serialize");
    rendered.push(b'\n');
    if std::env::var_os(UPDATE_FIXTURES_ENV).is_some() {
        std::fs::create_dir_all(path.parent().expect("fixture path should have a parent"))
            .expect("fixture directory should be created");
        std::fs::write(&path, &rendered).expect("fixture should update");
        return;
    }
    let expected =
        std::fs::read(&path).unwrap_or_else(|error| panic!("reading golden fixture {}: {error}", path.display()));
    assert_eq!(rendered, expected, "golden fixture drifted: {}", path.display());
}

fn layout_facts(plan: &ExportPlan, report: Option<OciExportReport>) -> LayoutFacts {
    LayoutFacts {
        oci_layout_bytes: plan.oci_layout_bytes.clone(),
        index_bytes: plan.index_bytes.clone(),
        blobs: plan.blobs.iter().map(|blob| (blob.digest.clone(), blob.bytes.clone())).collect(),
        export_report: report,
    }
}

fn rewrite_manifest(facts: &mut LayoutFacts, mutate: impl FnOnce(&mut OciManifestDocument)) {
    let mut index: OciIndexDocument = serde_json::from_slice(&facts.index_bytes).expect("fixture index should parse");
    let old_digest = index.manifests[0].digest.clone();
    let old_bytes = facts.blobs.remove(&old_digest).expect("fixture manifest blob should exist");
    let mut manifest: OciManifestDocument = serde_json::from_slice(&old_bytes).expect("fixture manifest should parse");
    mutate(&mut manifest);
    let new_bytes = serde_json::to_vec(&manifest).expect("mutated manifest should serialize");
    let new_digest = sha256_digest(&new_bytes);
    index.manifests[0].digest = new_digest.clone();
    index.manifests[0].size = new_bytes.len() as u64;
    facts.index_bytes = serde_json::to_vec(&index).expect("mutated index should serialize");
    facts.blobs.insert(new_digest, new_bytes);
    facts.export_report = None;
}

fn rewrite_config_diff_id(facts: &mut LayoutFacts, replacement: String) {
    let mut index: OciIndexDocument = serde_json::from_slice(&facts.index_bytes).expect("fixture index should parse");
    let old_manifest_digest = index.manifests[0].digest.clone();
    let old_manifest_bytes = facts.blobs.remove(&old_manifest_digest).expect("fixture manifest should exist");
    let mut manifest: OciManifestDocument =
        serde_json::from_slice(&old_manifest_bytes).expect("fixture manifest should parse");
    let old_config_digest = manifest.config.digest.clone();
    let old_config_bytes = facts.blobs.remove(&old_config_digest).expect("fixture config should exist");
    let mut config: serde_json::Value = serde_json::from_slice(&old_config_bytes).expect("fixture config should parse");
    config["rootfs"]["diff_ids"][0] = serde_json::Value::String(replacement);
    let new_config_bytes = serde_json::to_vec(&config).expect("mutated config should serialize");
    let new_config_digest = sha256_digest(&new_config_bytes);
    manifest.config.digest = new_config_digest.clone();
    manifest.config.size = new_config_bytes.len() as u64;
    let new_manifest_bytes = serde_json::to_vec(&manifest).expect("mutated manifest should serialize");
    let new_manifest_digest = sha256_digest(&new_manifest_bytes);
    index.manifests[0].digest = new_manifest_digest.clone();
    index.manifests[0].size = new_manifest_bytes.len() as u64;
    facts.index_bytes = serde_json::to_vec(&index).expect("mutated index should serialize");
    facts.blobs.insert(new_config_digest, new_config_bytes);
    facts.blobs.insert(new_manifest_digest, new_manifest_bytes);
    facts.export_report = None;
}

#[test]
fn deterministic_projection_round_trips_as_admitted() {
    let spec = b"accepted Onix kernel-bundle spec";
    let projection = projection(spec);
    let first = build_export_plan(&projection, spec, &objects()).expect("projection should build");
    let second = build_export_plan(&projection, spec, &objects()).expect("projection should rebuild");
    assert_eq!(first.index_bytes, second.index_bytes);
    assert_eq!(first.blobs, second.blobs);
    assert_eq!(first.layout_blake3, second.layout_blake3);
    assert_eq!(first.layers.len(), projection.layers.len());
    assert_eq!(
        first.manifest_descriptor.annotations.get("io.multikernel.kbi.id"),
        projection.required_annotations.get("io.multikernel.kbi.id")
    );
    let manifest_bytes = first
        .blobs
        .iter()
        .find(|blob| blob.digest == first.manifest_descriptor.digest)
        .expect("manifest blob should exist");
    let manifest: OciManifestDocument = serde_json::from_slice(&manifest_bytes.bytes).expect("manifest should parse");
    let module_descriptor = manifest
        .layers
        .iter()
        .find(|descriptor| {
            descriptor.annotations.get("org.mantle.layer.role").is_some_and(|role| role == "module-pack")
        })
        .expect("module descriptor should exist");
    assert_eq!(module_descriptor.annotations.get("io.onix.pack.kind").map(String::as_str), Some("module"));
    assert!(!first.layers[0].canonical_archive);
    assert!(first.layers[1].canonical_archive);
    assert_eq!(
        first.layers[0].blob_blake3,
        blake3_hex(EXACT_BYTES),
        "exact-file BLAKE3 must remain distinct from the OCI SHA-256 descriptor"
    );

    let export = export_report(&first).expect("export report should serialize");
    verify_export_report(&export).expect("export receipt should verify");
    let imported = validate_import(&layout_facts(&first, Some(export))).expect("layout should import");
    assert_eq!(imported.state, STATUS_ADMITTED);
    assert_eq!(imported.projection_blake3.as_deref(), Some(first.projection_blake3.as_str()));
    assert_eq!(imported.round_trip, Some(projection.round_trip));
    assert!(!imported.imported, "pure validation must not claim CAS admission");
}

#[test]
fn deterministic_gzip_profile_round_trips_exactly() {
    let spec = b"accepted Onix kernel-bundle spec";
    let mut projection = projection(spec);
    projection.archive_policy.compression = CompressionProfile::GzipDeterministicV1;
    projection
        .layers
        .iter_mut()
        .find(|layer| layer.role == "module-pack")
        .expect("module layer should exist")
        .media_type = "application/vnd.onix.module-pack.v1.tar+gzip".to_string();
    seal_projection(&mut projection).expect("gzip projection should reseal");
    let first = build_export_plan(&projection, spec, &objects()).expect("gzip projection should build");
    let second = build_export_plan(&projection, spec, &objects()).expect("gzip projection should rebuild");
    let first_archive =
        first.layers.iter().find(|layer| layer.canonical_archive).expect("first gzip layer should exist");
    let second_archive =
        second.layers.iter().find(|layer| layer.canonical_archive).expect("second gzip layer should exist");
    assert_eq!(first_archive.compression, CompressionProfile::GzipDeterministicV1);
    assert_eq!(first_archive.blob_sha256, second_archive.blob_sha256);
    let report = export_report(&first).expect("gzip export report should build");
    let imported =
        validate_import(&layout_facts(&first, Some(report))).expect("canonical gzip layout should re-import");
    assert_eq!(imported.state, STATUS_ADMITTED);
}

#[test]
fn canonical_layer_bytes_ignore_traversal_and_ambient_metadata_identity() {
    let spec = b"accepted Onix kernel-bundle spec";
    let first_projection = projection(spec);
    let first_plan = build_export_plan(&first_projection, spec, &objects()).expect("first projection should build");

    let old_ref = artifact_ref('b');
    let new_ref = artifact_ref('d');
    let mut second_projection = first_projection.clone();
    second_projection
        .layers
        .iter_mut()
        .find(|layer| layer.role == "module-pack")
        .expect("module layer should exist")
        .entries[0]
        .object_ref = new_ref.clone();
    let admission = second_projection
        .object_admissions
        .iter_mut()
        .find(|value| value.artifact_ref == old_ref)
        .expect("module admission should exist");
    let target_identity = admission.target_identity.clone().expect("module target identity should exist");
    *admission = reduce_source_admission(&source_admission(new_ref.clone(), target_identity, &blake3_hex(spec)))
        .expect("replacement source admission should reduce");
    seal_projection(&mut second_projection).expect("second projection should reseal");
    let mut second_objects = objects();
    let mut module_object = second_objects.remove(&old_ref).expect("module object should exist");
    module_object.artifact_ref = new_ref.clone();
    module_object.entries.reverse();
    second_objects.insert(new_ref, module_object);
    let second_plan =
        build_export_plan(&second_projection, spec, &second_objects).expect("second projection should build");

    let first_layer = first_plan
        .layers
        .iter()
        .find(|layer| layer.canonical_archive)
        .expect("first archive layer should exist");
    let second_layer = second_plan
        .layers
        .iter()
        .find(|layer| layer.canonical_archive)
        .expect("second archive layer should exist");
    assert_ne!(first_plan.projection_blake3, second_plan.projection_blake3);
    assert_eq!(first_layer.blob_sha256, second_layer.blob_sha256);
    assert_eq!(first_layer.blob_blake3, second_layer.blob_blake3);
}

#[test]
fn external_layout_remains_compatibility_only() {
    let spec = b"accepted Onix kernel-bundle spec";
    let projection = projection(spec);
    let plan = build_export_plan(&projection, spec, &objects()).expect("projection should build");
    let imported = validate_import(&layout_facts(&plan, None)).expect("safe external layout should inspect");
    assert_eq!(imported.state, STATUS_COMPATIBILITY_ONLY);
    assert!(imported.projection_blake3.is_none());
    assert!(imported.round_trip.is_none());
    assert!(imported.objects.iter().any(|object| object.archive_profile == "external-safe-inspection"));
}

#[test]
fn rejects_stale_missing_and_reordered_source_admissions_without_leaking_build_roots() {
    let spec = b"accepted Onix kernel-bundle spec";
    let projection = projection(spec);
    let mut stale = source_admissions_for_projection(&projection);
    stale.admissions[0].validator_ref = "stale-validator".to_string();
    let stale_issues = validate_source_admissions(&projection, &stale);
    assert!(stale_issues.iter().any(|value| value.code == "source-admission-mismatch"));

    let mut incomplete = source_admissions_for_projection(&projection);
    incomplete.admissions.pop();
    let incomplete_issues = validate_source_admissions(&projection, &incomplete);
    assert!(incomplete_issues.iter().any(|value| value.code == "source-admission-set-mismatch"));

    let mut reordered = source_admissions_for_projection(&projection);
    reordered.admissions.reverse();
    let reordered_issues = validate_source_admissions(&projection, &reordered);
    assert!(reordered_issues.iter().any(|value| value.code == "non-canonical-source-admission-order"));

    let mut path_bundle = source_admissions_for_projection(&projection);
    let baseline_reduction =
        reduce_source_admission(&path_bundle.admissions[0]).expect("baseline source admission should reduce");
    let baseline_projection_identity = projection.admission.projection_blake3.clone();
    path_bundle.admissions[0].build_root = "/tmp/host-specific-root".to_string();
    let path_reduction =
        reduce_source_admission(&path_bundle.admissions[0]).expect("path-bearing source admission should reduce");
    assert_eq!(path_reduction, baseline_reduction);

    let mut path_projection = projection.clone();
    path_projection.object_admissions[0] = path_reduction;
    seal_projection(&mut path_projection).expect("path-bearing projection should reseal");
    assert_eq!(path_projection.admission.projection_blake3, baseline_projection_identity);
    let plan = super::build_export_plan(&path_projection, spec, &path_bundle, &objects())
        .expect("path-bearing source admission should validate without being copied");
    let report = export_report(&plan).expect("redacted export report should build");
    let report_json = serde_json::to_string(&report).expect("report should serialize");
    assert!(!report_json.contains(&path_bundle.admissions[0].build_root));
}

#[test]
fn rejects_spec_projection_and_hidden_fallback_disagreement() {
    let spec = b"accepted Onix kernel-bundle spec";
    let mut projection = projection(spec);
    projection.frontend_spec.hash = digest_hex('f');
    projection.admission.no_hidden_fallback = false;
    projection.admission.projection_blake3 = Some(digest_hex('e'));
    let issues = validate_projection(&projection, spec);
    let codes = issues.iter().map(|value| value.code.as_str()).collect::<BTreeSet<_>>();
    assert!(codes.contains("spec-hash-mismatch"));
    assert!(codes.contains("frontend-not-admitted"));
    assert!(codes.contains("projection-hash-mismatch"));
}

#[test]
fn source_admission_bundle_rejects_unknown_material() {
    let spec = b"accepted Onix kernel-bundle spec";
    let projection = projection(spec);
    let bundle = source_admissions_for_projection(&projection);
    let mut unknown_bundle = serde_json::to_value(&bundle).expect("source admissions should serialize");
    unknown_bundle
        .as_object_mut()
        .expect("source bundle should be an object")
        .insert("fallback_path".to_string(), serde_json::Value::String("/tmp/object".to_string()));
    assert!(serde_json::from_value::<SourceAdmissionBundle>(unknown_bundle).is_err());

    let mut unknown_admission = serde_json::to_value(&bundle).expect("source admissions should serialize");
    unknown_admission["admissions"][0]["fallback_path"] = serde_json::Value::String("/tmp/object".to_string());
    assert!(serde_json::from_value::<SourceAdmissionBundle>(unknown_admission).is_err());
}

#[test]
fn rejects_wrong_digest_roles_and_noncanonical_order() {
    let spec = b"accepted Onix kernel-bundle spec";
    let mut projection = projection(spec);
    projection.layers.reverse();
    projection.layers[0].entries[0].object_ref = format!("sha256:{}", digest_hex('b'));
    projection.expected_external_digests[0].digest = format!("blake3:{}", digest_hex('a'));
    let issues = validate_projection(&projection, spec);
    assert!(issues.iter().any(|value| value.code == "non-canonical-layer-order"));
    assert!(issues.iter().any(|value| value.code == "unsafe-object-ref"));
    assert!(issues.iter().any(|value| value.code == "invalid-external-digest"));
}

#[test]
fn rejects_traversal_wrong_media_and_invalid_bounds() {
    let spec = b"accepted Onix kernel-bundle spec";
    let mut projection = projection(spec);
    projection.bounds.max_layers = 0;
    projection.layers[0].media_type = "invalid media type".to_string();
    projection.layers[0].entries[0].relative_path = "../escape".to_string();
    projection.layers[0]
        .annotations
        .insert("org.mantle.layer.role".to_string(), "frontend-override".to_string());
    projection.non_claims.push("bundle is bootable".to_string());
    seal_projection(&mut projection).expect("malformed fixture should still serialize and seal");
    let issues = validate_projection(&projection, spec);
    let codes = issues.iter().map(|value| value.code.as_str()).collect::<BTreeSet<_>>();
    assert!(codes.contains("invalid-bounds"));
    assert!(codes.contains("invalid-layer-count"));
    assert!(codes.contains("invalid-media-type"));
    assert!(codes.contains("duplicate-or-unsafe-path"));
    assert!(codes.contains("unsafe-annotation"));
    assert!(codes.contains("unsafe-non-claim"));
}

#[test]
fn rejects_duplicate_descriptors_and_wrong_import_media_types() {
    let spec = b"accepted Onix kernel-bundle spec";
    let projection = projection(spec);
    let plan = build_export_plan(&projection, spec, &objects()).expect("projection should build");

    let mut duplicate = layout_facts(&plan, None);
    rewrite_manifest(&mut duplicate, |manifest| {
        manifest.layers[1] = manifest.layers[0].clone();
    });
    let duplicate_issues = validate_import(&duplicate).expect_err("duplicate descriptor must fail");
    assert!(duplicate_issues.iter().any(|value| value.code == "duplicate-descriptor"));

    let mut wrong_media = layout_facts(&plan, None);
    rewrite_manifest(&mut wrong_media, |manifest| {
        manifest.layers[0].media_type = "invalid media type".to_string();
    });
    let media_issues = validate_import(&wrong_media).expect_err("wrong layer media type must fail");
    assert!(media_issues.iter().any(|value| value.code == "invalid-media-type"));

    let mut wrong_diff_id = layout_facts(&plan, None);
    rewrite_config_diff_id(&mut wrong_diff_id, format!("sha256:{}", digest_hex('9')));
    let diff_id_issues = validate_import(&wrong_diff_id).expect_err("wrong rootfs diff id must fail");
    assert!(diff_id_issues.iter().any(|value| value.code == "diff-id-mismatch"));
}

#[test]
fn rejects_truncated_archives_and_uncompressed_archive_bombs() {
    let policy = ArchivePolicy::default();
    let truncated = inspect_archive(
        "application/vnd.onix.module-pack.v1.tar",
        b"truncated",
        &policy,
        MAX_DEPTH_HARD,
        MAX_ENTRIES_HARD,
        MAX_TOTAL_BYTES_HARD,
        false,
    )
    .expect_err("truncated archive must fail");
    assert_eq!(truncated.code, "truncated-archive");

    let mut hardlink_builder = tar::Builder::new(Vec::new());
    let mut hardlink_header = tar::Header::new_gnu();
    hardlink_header.set_entry_type(tar::EntryType::Link);
    hardlink_header.set_size(0);
    hardlink_header.set_mode(policy.file_mode);
    hardlink_header.set_uid(policy.uid);
    hardlink_header.set_gid(policy.gid);
    hardlink_header.set_mtime(policy.mtime);
    hardlink_header.set_link_name("target").expect("hard-link target should fit the test header");
    hardlink_header.set_cksum();
    hardlink_builder
        .append_data(&mut hardlink_header, "hard-link", std::io::empty())
        .expect("hard-link fixture should append");
    hardlink_builder.finish().expect("hard-link fixture should finish");
    let hardlink_bytes = hardlink_builder.into_inner().expect("hard-link fixture bytes should be available");
    let hardlink = inspect_archive(
        "application/vnd.onix.module-pack.v1.tar",
        &hardlink_bytes,
        &policy,
        MAX_DEPTH_HARD,
        MAX_ENTRIES_HARD,
        MAX_TOTAL_BYTES_HARD,
        false,
    )
    .expect_err("hard links must fail archive inspection");
    assert_eq!(hardlink.code, "unsupported-archive-entry");

    let spec = b"accepted Onix kernel-bundle spec";
    let projection = projection(spec);
    let plan = build_export_plan(&projection, spec, &objects()).expect("projection should build");
    let archive_record =
        plan.layers.iter().find(|record| record.canonical_archive).expect("archive record should exist");
    let archive_blob = plan
        .blobs
        .iter()
        .find(|blob| blob.digest == archive_record.blob_sha256)
        .expect("archive blob should exist");
    let bomb = inspect_archive(
        &archive_record.media_type,
        &archive_blob.bytes,
        &policy,
        MAX_DEPTH_HARD,
        MAX_ENTRIES_HARD,
        1,
        false,
    )
    .expect_err("archive expansion beyond the bound must fail");
    assert_eq!(bomb.code, "archive-byte-bound");
}

#[test]
fn rejects_unknown_behavior_fields_during_deserialization() {
    let spec = b"accepted Onix kernel-bundle spec";
    let projection = projection(spec);
    let mut value = serde_json::to_value(projection).expect("projection should serialize");
    value
        .as_object_mut()
        .expect("projection is an object")
        .insert("execute_makefile".to_string(), serde_json::Value::Bool(true));
    let error = serde_json::from_value::<OciProjection>(value).expect_err("unknown behavior fields must fail");
    assert!(error.to_string().contains("unknown field"));
}

#[test]
fn rejects_escaping_symlink_before_archive_construction() {
    let spec = b"accepted Onix kernel-bundle spec";
    let projection = projection(spec);
    let mut objects = objects();
    objects.get_mut(&artifact_ref('b')).expect("module object exists").entries.push(ObjectEntry {
        relative_path: "escape".to_string(),
        kind: ObjectEntryKind::Symlink,
        data: Vec::new(),
        link_target: Some("../../../../outside".to_string()),
    });
    let issues = build_export_plan(&projection, spec, &objects).expect_err("escaping links must fail");
    assert!(issues.iter().any(|value| value.code == "escaping-link"));
}

#[test]
fn rejects_archive_path_collisions() {
    let spec = b"accepted Onix kernel-bundle spec";
    let mut projection = projection(spec);
    projection.layers[1].entries[0].size_bytes = (MODULE_BYTES.len() * COLLIDING_FILE_COUNT) as u64;
    seal_projection(&mut projection).expect("collision fixture should reseal");
    let mut objects = objects();
    objects.get_mut(&artifact_ref('b')).expect("module object exists").entries.push(ObjectEntry {
        relative_path: "kernel/module.ko".to_string(),
        kind: ObjectEntryKind::File,
        data: MODULE_BYTES.to_vec(),
        link_target: None,
    });
    let issues = build_export_plan(&projection, spec, &objects).expect_err("archive collisions must fail");
    assert!(issues.iter().any(|value| value.code == "archive-path-collision"));
}

#[test]
fn rejects_descriptor_tampering_before_import() {
    let spec = b"accepted Onix kernel-bundle spec";
    let projection = projection(spec);
    let plan = build_export_plan(&projection, spec, &objects()).expect("projection should build");
    let export = export_report(&plan).expect("report should build");
    let mut facts = layout_facts(&plan, Some(export));
    let layer_digest = plan.layers[0].blob_sha256.clone();
    facts.blobs.get_mut(&layer_digest).expect("layer blob exists").push(0);
    let issues = validate_import(&facts).expect_err("tampered descriptor bytes must fail");
    assert!(issues.iter().any(|value| value.code == "descriptor-mismatch"));
}

#[test]
fn rejects_noncanonical_mantle_archive_but_allows_safe_external_archive() {
    let spec = b"accepted Onix kernel-bundle spec";
    let projection = projection(spec);
    let plan = build_export_plan(&projection, spec, &objects()).expect("projection should build");
    let archive_record = plan.layers.iter().find(|record| record.canonical_archive).expect("archive layer exists");
    let archive_bytes = plan
        .blobs
        .iter()
        .find(|blob| blob.digest == archive_record.blob_sha256)
        .expect("archive blob exists")
        .bytes
        .clone();
    assert_eq!(
        inspect_archive(
            &archive_record.media_type,
            &archive_bytes,
            &ArchivePolicy::default(),
            MAX_DEPTH_HARD,
            MAX_ENTRIES_HARD,
            MAX_TOTAL_BYTES_HARD,
            true,
        )
        .expect("canonical profile should reproduce"),
        "mantle-canonical-exact"
    );
    assert_eq!(
        inspect_archive(
            &archive_record.media_type,
            &archive_bytes,
            &ArchivePolicy::default(),
            MAX_DEPTH_HARD,
            MAX_ENTRIES_HARD,
            MAX_TOTAL_BYTES_HARD,
            false,
        )
        .expect("external safety inspection should pass"),
        "external-safe-inspection"
    );
}

#[test]
fn golden_reports_cover_full_minimal_and_both_import_states() {
    let spec = b"accepted Onix kernel-bundle spec";
    let full_projection = projection(spec);
    let full_plan = build_export_plan(&full_projection, spec, &objects()).expect("full projection should build");
    let full_export = export_report(&full_plan).expect("full report should build");
    assert_json_fixture("mantle-export-full.json", &full_export);
    assert_json_fixture("mantle-projection-full.json", &full_projection);
    assert_json_fixture("mantle-source-admissions-full.json", &source_admissions_for_projection(&full_projection));

    let mut minimal_projection = full_projection.clone();
    minimal_projection.layers.truncate(1);
    minimal_projection.object_admissions.truncate(1);
    minimal_projection.round_trip.component_identities.truncate(1);
    minimal_projection.round_trip.pack_identities.clear();
    seal_projection(&mut minimal_projection).expect("minimal projection should reseal");
    let minimal_objects = BTreeMap::from([(
        artifact_ref('a'),
        objects().remove(&artifact_ref('a')).expect("minimal object should exist"),
    )]);
    let minimal_plan =
        build_export_plan(&minimal_projection, spec, &minimal_objects).expect("minimal projection should build");
    let minimal_export = export_report(&minimal_plan).expect("minimal report should build");
    assert_json_fixture("mantle-export-minimal.json", &minimal_export);

    let mut admitted =
        validate_import(&layout_facts(&full_plan, Some(full_export))).expect("admitted import should validate");
    let refs = admitted
        .objects
        .iter()
        .zip(IMPORTED_REF_DIGEST_MARKERS)
        .map(|(object, marker)| (object.oci_sha256.clone(), artifact_ref(marker)))
        .collect::<BTreeMap<_, _>>();
    attach_imported_refs(&mut admitted, &refs).expect("admitted refs should attach");
    verify_import_report(&admitted).expect("admitted report should verify");
    assert_json_fixture("mantle-import-admitted.json", &admitted);

    let mut compatibility =
        validate_import(&layout_facts(&full_plan, None)).expect("compatibility import should validate");
    attach_imported_refs(&mut compatibility, &refs).expect("compatibility refs should attach");
    verify_import_report(&compatibility).expect("compatibility report should verify");
    assert_json_fixture("mantle-import-compatibility-only.json", &compatibility);

    let mut unknown_field =
        serde_json::to_value(full_projection).expect("projection should serialize for negative fixture");
    unknown_field
        .as_object_mut()
        .expect("projection should be an object")
        .insert("execute_makefile".to_string(), serde_json::Value::Bool(true));
    assert_json_fixture("mantle-projection-unknown-field.json", &unknown_field);
    assert!(serde_json::from_value::<OciProjection>(unknown_field).is_err());
}

#[test]
fn reviewed_onix_snapshots_preserve_the_explicit_adapter_boundary() {
    const SPEC: &[u8] = include_bytes!("../../tests/fixtures/kernel-bundle-oci/onix-reviewed/kernel-bundles-spec.md");
    const FULL: &[u8] = include_bytes!("../../tests/fixtures/kernel-bundle-oci/onix-reviewed/full-expected.json");
    const MINIMAL: &[u8] = include_bytes!("../../tests/fixtures/kernel-bundle-oci/onix-reviewed/minimal-expected.json");
    const ADMITTED: &[u8] =
        include_bytes!("../../tests/fixtures/kernel-bundle-oci/onix-reviewed/import-admitted-expected.json");
    const COMPATIBILITY: &[u8] =
        include_bytes!("../../tests/fixtures/kernel-bundle-oci/onix-reviewed/import-compatibility-only-expected.json");
    assert_eq!(blake3_hex(SPEC), "a87e25785b74338bf1bc3c77a24f7cdf9ce17c3a581ae60f1174a2b8584bda01");
    assert_eq!(blake3_hex(FULL), "5389f7b9dc30cf6a08664c8a3db340d26ea9044f991f68b9108e2e9140bcb1ae");
    assert_eq!(blake3_hex(MINIMAL), "3df16f474661fa901dc13a6bc987eb44f335528b87758f754f1ae04fbf8dae68");
    assert_eq!(blake3_hex(ADMITTED), "e5200a4eed5893b6e7ef7ae1b0be8ff726f3c3e8c57ae93e4c87ac2b6abc4d30");
    assert_eq!(blake3_hex(COMPATIBILITY), "55b6a469e08540748af4cb871d00d057bec9ee62397a1e73c7aa45b95f50c67e");

    let full: serde_json::Value = serde_json::from_slice(FULL).expect("reviewed Onix fixture should parse");
    let raw_projection = full.get("projection").expect("Onix fixture should contain a projection").clone();
    assert_eq!(raw_projection.get("schema").and_then(serde_json::Value::as_str), Some(OCI_PROJECTION_SCHEMA));
    assert!(
        serde_json::from_value::<OciProjection>(raw_projection).is_err(),
        "raw frontend projection must not bypass Mantle admissions and archive policy"
    );

    let admitted: serde_json::Value = serde_json::from_slice(ADMITTED).expect("admitted response should parse");
    let compatibility: serde_json::Value =
        serde_json::from_slice(COMPATIBILITY).expect("compatibility response should parse");
    assert_eq!(admitted.get("state").and_then(serde_json::Value::as_str), Some(STATUS_ADMITTED));
    assert_eq!(compatibility.get("state").and_then(serde_json::Value::as_str), Some(STATUS_COMPATIBILITY_ONLY));
}

#[test]
fn imported_refs_are_attached_only_after_all_blobs_are_available() {
    let spec = b"accepted Onix kernel-bundle spec";
    let projection = projection(spec);
    let plan = build_export_plan(&projection, spec, &objects()).expect("projection should build");
    let mut report = validate_import(&layout_facts(&plan, None)).expect("layout should validate");
    let incomplete = BTreeMap::from([(report.objects[0].oci_sha256.clone(), artifact_ref('c'))]);
    assert!(attach_imported_refs(&mut report, &incomplete).is_err());
    assert!(!report.imported);
    assert!(report.objects.iter().all(|object| object.artifact_ref.is_none()));

    let refs = report
        .objects
        .iter()
        .map(|object| (object.oci_sha256.clone(), artifact_ref('c')))
        .collect::<BTreeMap<_, _>>();
    attach_imported_refs(&mut report, &refs).expect("complete admissions should attach");
    verify_import_report(&report).expect("completed import report should verify");
    let mut tampered = report.clone();
    tampered.layout_blake3 = digest_hex('d');
    assert!(verify_import_report(&tampered).is_err());
    let mut overclaim = report.clone();
    overclaim.non_claims.push("bundle is bootable".to_string());
    overclaim.receipt_blake3.clear();
    overclaim.receipt_blake3 = report_identity(IMPORT_HASH_DOMAIN, &overclaim).expect("overclaim fixture should hash");
    assert!(verify_import_report(&overclaim).is_err());
    assert!(report.imported);
    assert!(report.objects.iter().all(|object| object.artifact_ref.is_some()));
}
