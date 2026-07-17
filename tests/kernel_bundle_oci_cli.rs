use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;

use assert_cmd::Command;
use mantle::oci_projection::ArchivePolicy;
use mantle::oci_projection::EXPECTED_DIGEST_ROLE_OCI_LAYER_BLOB;
use mantle::oci_projection::ExternalDigestExpectation;
use mantle::oci_projection::FrontendSpecBinding;
use mantle::oci_projection::LayerMode;
use mantle::oci_projection::OCI_EXPORT_REPORT_FILENAME;
use mantle::oci_projection::OCI_INDEX_FILENAME;
use mantle::oci_projection::OCI_LAYOUT_FILENAME;
use mantle::oci_projection::OciExportReport;
use mantle::oci_projection::OciImportReport;
use mantle::oci_projection::OciProjection;
use mantle::oci_projection::Platform;
use mantle::oci_projection::ProjectionAdmission;
use mantle::oci_projection::ProjectionBounds;
use mantle::oci_projection::ProjectionEntry;
use mantle::oci_projection::ProjectionLayer;
use mantle::oci_projection::ProjectionObjectAdmission;
use mantle::oci_projection::RoundTripExpectation;
use mantle::oci_projection::SOURCE_ADMISSION_BUNDLE_SCHEMA;
use mantle::oci_projection::SourceAdmissionBundle;
use mantle::oci_projection::SourceArtifactAdmission;
use mantle::oci_projection::blake3_hex;
use mantle::oci_projection::reduce_source_admission;
use mantle::oci_projection::seal_projection;
use mantle::oci_projection::sha256_digest;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use serde::Deserialize;
use serde::Serialize;
use tempfile::TempDir;

const ARTIFACT_IMPORT_REPORT_SCHEMA: &str = "mantle-frontend-artifact-store-v1";
const ARTIFACT_REF_PREFIX: &str = "mantle://blake3/";
const ARTIFACT_DIGEST_PREFIX: &str = "blake3:";
const EXAMPLE_ROOT: &str = "examples/projects/kernel-bundle-oci-local";
const KERNEL_RELATIVE_PATH: &str = "fixtures/vmlinuz";
const MODULES_RELATIVE_PATH: &str = "fixtures/modules";
const MODULE_FILE_RELATIVE_PATH: &str = "fixtures/modules/kernel/module.ko";
const SPEC_RELATIVE_PATH: &str = "fixtures/kernel-bundles-spec.md";
const PROJECTION_RELATIVE_PATH: &str = "fixtures/projection.json";
const SOURCE_ADMISSIONS_RELATIVE_PATH: &str = "fixtures/source-admissions.json";
const UPDATE_EXAMPLE_ENV: &str = "MANTLE_UPDATE_KERNEL_BUNDLE_OCI_EXAMPLE";
const HEX_CHARS_PER_BYTE: usize = 2;
const BLAKE3_HEX_LENGTH: usize = blake3::OUT_LEN * HEX_CHARS_PER_BYTE;
const EXPECTED_LAYER_COUNT: usize = 2;
const TAMPER_BYTES: &[u8] = b"tampered";
const SPEC_ID: &str = "kernel-bundles";
const SPEC_VERSION: &str = "1";
const VALIDATOR_KIND: &str = "onix-kernel-bundle-v1";
const VALIDATOR_REF: &str = "builtin:onix-kernel-bundle-v1";
const ARTIFACT_KIND: &str = "kernel-bundle-component";
const BUILD_ROOT: &str = "gallery-build-root";

#[derive(Debug, Deserialize)]
struct ArtifactImportReport {
    schema: String,
    artifact_ref: String,
    artifact_digest: String,
}

struct PreparedCliFixture {
    _root: TempDir,
    import_state: PathBuf,
    import_report: PathBuf,
    layout: PathBuf,
    exported: OciExportReport,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn example_root() -> PathBuf {
    repo_root().join(EXAMPLE_ROOT)
}

fn mantle_cmd() -> Command {
    Command::cargo_bin("mantle").expect("mantle binary should be built")
}

fn identity(label: &str, byte: char) -> String {
    let digest = std::iter::repeat_n(byte, BLAKE3_HEX_LENGTH).collect::<String>();
    format!("onix:{label}:blake3:{digest}")
}

fn source_admission(
    object: &ArtifactImportReport,
    target_identity: String,
    spec_hash: &str,
) -> SourceArtifactAdmission {
    SourceArtifactAdmission {
        spec_id: SPEC_ID.to_string(),
        spec_version: SPEC_VERSION.to_string(),
        spec_hash_algorithm: "blake3".to_string(),
        spec_hash: spec_hash.to_string(),
        validator_kind: VALIDATOR_KIND.to_string(),
        validator_ref: VALIDATOR_REF.to_string(),
        artifact_kind: ARTIFACT_KIND.to_string(),
        artifact_ref: object.artifact_ref.clone(),
        artifact_digest: Some(object.artifact_digest.clone()),
        target_identity: Some(target_identity),
        build_root: BUILD_ROOT.to_string(),
        validation_result: "admitted".to_string(),
        no_hidden_fallback: true,
    }
}

fn reduced_admission(source: &SourceArtifactAdmission) -> ProjectionObjectAdmission {
    reduce_source_admission(source).expect("gallery source admission should reduce")
}

fn projection(
    spec_bytes: &[u8],
    kernel_bytes: &[u8],
    module_bytes: &[u8],
    kernel: &ArtifactImportReport,
    modules: &ArtifactImportReport,
) -> (OciProjection, SourceAdmissionBundle) {
    let spec_hash = blake3_hex(spec_bytes);
    let kernel_identity = identity("component", '1');
    let module_identity = identity("component", '2');
    let kernel_source = source_admission(kernel, kernel_identity.clone(), &spec_hash);
    let module_source = source_admission(modules, module_identity.clone(), &spec_hash);
    let sources = vec![kernel_source, module_source];
    let mut projection = OciProjection {
        schema: "mantle-oci-projection-v1".to_string(),
        frontend_spec: FrontendSpecBinding {
            id: SPEC_ID.to_string(),
            version: SPEC_VERSION.to_string(),
            hash_algorithm: "blake3".to_string(),
            hash: spec_hash,
        },
        admission: ProjectionAdmission {
            validation_result: "admitted".to_string(),
            projection_blake3: None,
            no_hidden_fallback: true,
        },
        platform: Platform {
            architecture: "x86_64".to_string(),
            os: "linux".to_string(),
        },
        bounds: ProjectionBounds::default(),
        archive_policy: ArchivePolicy::default(),
        object_admissions: sources.iter().map(reduced_admission).collect(),
        layers: gallery_layers(kernel_bytes, module_bytes, kernel, modules, &kernel_identity, &module_identity),
        required_annotations: gallery_annotations(),
        expected_external_digests: vec![ExternalDigestExpectation {
            role: EXPECTED_DIGEST_ROLE_OCI_LAYER_BLOB.to_string(),
            subject_identity: kernel_identity.clone(),
            digest: sha256_digest(kernel_bytes),
        }],
        round_trip: RoundTripExpectation {
            kernel_build_identity: identity("kernel-build", '4'),
            bundle_identity: identity("kernel-bundle", '5'),
            manifest_identity: identity("manifest", '6'),
            component_identities: vec![kernel_identity, module_identity],
            pack_identities: vec![identity("module-pack", '3')],
        },
        non_claims: gallery_non_claims(),
    };
    seal_projection(&mut projection).expect("gallery projection should seal");
    let source_bundle = SourceAdmissionBundle {
        schema: SOURCE_ADMISSION_BUNDLE_SCHEMA.to_string(),
        admissions: sources,
    };
    (projection, source_bundle)
}

fn gallery_layers(
    kernel_bytes: &[u8],
    module_bytes: &[u8],
    kernel: &ArtifactImportReport,
    modules: &ArtifactImportReport,
    kernel_identity: &str,
    module_identity: &str,
) -> Vec<ProjectionLayer> {
    vec![
        ProjectionLayer {
            role: "base-kernel".to_string(),
            media_type: "application/vnd.onix.kernel.v1".to_string(),
            mode: LayerMode::ExactBlob,
            entries: vec![ProjectionEntry {
                relative_path: "boot/vmlinuz".to_string(),
                object_ref: kernel.artifact_ref.clone(),
                identity: kernel_identity.to_string(),
                size_bytes: u64::try_from(kernel_bytes.len()).expect("kernel fixture size should fit u64"),
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
                object_ref: modules.artifact_ref.clone(),
                identity: module_identity.to_string(),
                size_bytes: u64::try_from(module_bytes.len()).expect("module fixture size should fit u64"),
            }],
            pack_identity: Some(identity("module-pack", '3')),
            annotations: BTreeMap::new(),
        },
    ]
}

fn gallery_annotations() -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            "io.multikernel.kbi.id".to_string(),
            format!("kbi:sha256:{}", std::iter::repeat_n('7', BLAKE3_HEX_LENGTH).collect::<String>()),
        ),
        ("org.onix.bundle.schema".to_string(), VALIDATOR_KIND.to_string()),
    ])
}

fn gallery_non_claims() -> Vec<String> {
    vec![
        "no registry publication".to_string(),
        "no kernel compatibility decision".to_string(),
        "no bootability claim".to_string(),
        "no deployability claim".to_string(),
        "no release eligibility claim".to_string(),
        "no signature policy claim".to_string(),
        "frontend semantics remain external".to_string(),
    ]
}

fn import_artifact(path: &Path, state: &Path, report_path: &Path) -> ArtifactImportReport {
    let output = mantle_cmd()
        .arg("--json")
        .arg("--state-dir")
        .arg(state)
        .args(["artifact", "import"])
        .arg(path)
        .arg("--report-out")
        .arg(report_path)
        .output()
        .expect("artifact import should run");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "artifact import failed: {stderr}");
    let report: ArtifactImportReport = serde_json::from_slice(&output.stdout).expect("artifact report should parse");
    assert_eq!(report.schema, ARTIFACT_IMPORT_REPORT_SCHEMA);
    let digest = report
        .artifact_ref
        .strip_prefix(ARTIFACT_REF_PREFIX)
        .expect("artifact ref should use the Mantle BLAKE3 scheme");
    assert_eq!(report.artifact_digest, format!("{ARTIFACT_DIGEST_PREFIX}{digest}"));
    report
}

fn rendered_json<T: Serialize>(value: &T) -> Vec<u8> {
    let mut bytes = serde_json::to_vec_pretty(value).expect("gallery JSON should serialize");
    bytes.push(b'\n');
    bytes
}

fn update_or_assert_fixture(path: &Path, expected: &[u8]) {
    if std::env::var_os(UPDATE_EXAMPLE_ENV).is_some() {
        std::fs::write(path, expected).expect("gallery fixture should update");
    }
    let observed =
        std::fs::read(path).unwrap_or_else(|error| panic!("reading gallery fixture {}: {error}", path.display()));
    assert_eq!(observed, expected, "gallery fixture drifted: {}", path.display());
}

fn prepare_cli_fixture() -> PreparedCliFixture {
    let root = TempDir::new().expect("temporary CLI fixture should exist");
    let source_state = root.path().join("source-state");
    let import_state = root.path().join("import-state");
    let example = example_root();
    let kernel =
        import_artifact(&example.join(KERNEL_RELATIVE_PATH), &source_state, &root.path().join("kernel-import.json"));
    let modules =
        import_artifact(&example.join(MODULES_RELATIVE_PATH), &source_state, &root.path().join("modules-import.json"));
    let spec_bytes = std::fs::read(example.join(SPEC_RELATIVE_PATH)).expect("spec fixture should be readable");
    let kernel_bytes = std::fs::read(example.join(KERNEL_RELATIVE_PATH)).expect("kernel fixture should be readable");
    let module_bytes =
        std::fs::read(example.join(MODULE_FILE_RELATIVE_PATH)).expect("module fixture should be readable");
    let (projection, source_admissions) = projection(&spec_bytes, &kernel_bytes, &module_bytes, &kernel, &modules);
    let projection_path = example.join(PROJECTION_RELATIVE_PATH);
    let source_admissions_path = example.join(SOURCE_ADMISSIONS_RELATIVE_PATH);
    update_or_assert_fixture(&projection_path, &rendered_json(&projection));
    update_or_assert_fixture(&source_admissions_path, &rendered_json(&source_admissions));
    let layout = root.path().join("layout");
    let output = mantle_cmd()
        .arg("--json")
        .arg("--state-dir")
        .arg(&source_state)
        .args(["artifact", "oci-export", "--projection"])
        .arg(&projection_path)
        .arg("--spec-material")
        .arg(example.join(SPEC_RELATIVE_PATH))
        .arg("--source-admissions")
        .arg(&source_admissions_path)
        .arg("--out")
        .arg(&layout)
        .output()
        .expect("OCI export should run");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "OCI export failed: {stderr}");
    let exported: OciExportReport = serde_json::from_slice(&output.stdout).expect("OCI export report should parse");
    assert!(exported.exported);
    assert_eq!(exported.layers.len(), EXPECTED_LAYER_COUNT);
    assert!(layout.join(OCI_LAYOUT_FILENAME).is_file());
    assert!(layout.join(OCI_INDEX_FILENAME).is_file());
    assert!(layout.join(OCI_EXPORT_REPORT_FILENAME).is_file());
    PreparedCliFixture {
        import_report: root.path().join("import-report.json"),
        layout,
        import_state,
        exported,
        _root: root,
    }
}

#[test]
fn oci_projection_commands_are_public_and_document_the_local_boundary() {
    mantle_cmd()
        .args(["artifact", "oci-export", "--help"])
        .assert()
        .success()
        .stdout(contains("atomic local OCI image layout"))
        .stdout(contains("--spec-material"))
        .stdout(contains("--source-admissions"))
        .stdout(contains("--projection"));

    mantle_cmd()
        .args(["artifact", "oci-import", "--help"])
        .assert()
        .success()
        .stdout(contains("local OCI image layout"))
        .stdout(contains("--report-out"));
}

#[test]
fn checked_gallery_fixture_round_trips_through_the_public_cli() {
    let fixture = prepare_cli_fixture();
    let output = mantle_cmd()
        .arg("--json")
        .arg("--state-dir")
        .arg(&fixture.import_state)
        .args(["artifact", "oci-import", "--layout"])
        .arg(&fixture.layout)
        .arg("--report-out")
        .arg(&fixture.import_report)
        .output()
        .expect("OCI import should run");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "OCI import failed: {stderr}");
    let imported: OciImportReport = serde_json::from_slice(&output.stdout).expect("OCI import report should parse");
    assert!(imported.imported);
    assert_eq!(imported.state, "admitted");
    assert!(imported.objects.iter().all(|object| object.artifact_ref.is_some()));
    assert!(fixture.import_report.is_file());
}

#[test]
fn checked_gallery_fixture_rejects_tampered_layer_before_import_report() {
    let fixture = prepare_cli_fixture();
    let layer_digest = fixture.exported.layers[0]
        .blob_sha256
        .strip_prefix("sha256:")
        .expect("layer digest should use SHA-256");
    let layer_path = fixture.layout.join("blobs/sha256").join(layer_digest);
    let mut bytes = std::fs::read(&layer_path).expect("layer should be readable");
    bytes.extend_from_slice(TAMPER_BYTES);
    std::fs::write(&layer_path, bytes).expect("layer tamper should write");
    mantle_cmd()
        .arg("--state-dir")
        .arg(&fixture.import_state)
        .args(["artifact", "oci-import", "--layout"])
        .arg(&fixture.layout)
        .arg("--report-out")
        .arg(&fixture.import_report)
        .assert()
        .failure()
        .stderr(contains("descriptor-mismatch"));
    assert!(!fixture.import_report.exists());
    assert!(!fixture.import_state.exists());
}

#[test]
fn oci_export_rejects_unsealed_unknown_projection_before_object_reads() {
    let temporary = TempDir::new().expect("temporary CLI fixture should exist");
    let state = temporary.path().join("state");
    let projection = temporary.path().join("projection.json");
    let spec = temporary.path().join("spec.md");
    let source_admissions = temporary.path().join("source-admissions.json");
    let output = temporary.path().join("layout");
    std::fs::write(&projection, b"{\"schema\":\"unknown\"}\n").expect("projection fixture should write");
    std::fs::write(&spec, b"bounded spec material\n").expect("spec fixture should write");
    std::fs::write(&source_admissions, b"{}\n").expect("source admission fixture should write");

    mantle_cmd()
        .arg("--state-dir")
        .arg(&state)
        .args(["artifact", "oci-export", "--projection"])
        .arg(&projection)
        .arg("--spec-material")
        .arg(&spec)
        .arg("--source-admissions")
        .arg(&source_admissions)
        .arg("--out")
        .arg(&output)
        .assert()
        .failure()
        .stderr(
            contains("parsing OCI projection")
                .or(contains("exporting OCI layout"))
                .and(contains("missing field")),
        );

    assert!(!output.exists(), "failed preflight must not publish a layout");
    assert!(!state.exists(), "failed preflight must not read or mutate CAS state");
}
