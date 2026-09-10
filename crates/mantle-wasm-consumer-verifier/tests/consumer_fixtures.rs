//! Consumer fixtures for the published verifier facade and file shell.
//!
//! Positive fixtures prove complete bundle consumption. Negative fixtures
//! cover schema, identity, stage linkage, paths, symlinks, file types,
//! lengths, drift, bounds, missing members, report leaks, and overclaims.
//! Fixtures never import a product runtime.

use std::fs;
use std::path::PathBuf;

use crunch_wasm_component_core::AotAdmission;
use crunch_wasm_component_core::Blake3Identity;
use crunch_wasm_component_core::ComponentStageKind;
use crunch_wasm_component_core::MaterializationBundle;
use crunch_wasm_component_core::MaterializationBundleRequest;
use crunch_wasm_component_core::OciSha256Digest;
use crunch_wasm_component_core::REQUIRED_RELEASE_ELIGIBILITY_NON_CLAIM;
use crunch_wasm_component_core::REQUIRED_RUNTIME_AUTHORITY_NON_CLAIM;
use crunch_wasm_component_core::StageReceiptReference;
use crunch_wasm_component_core::StoreObject;
use crunch_wasm_component_core::build_materialization_bundle;
use mantle_wasm_consumer_verifier::ConsumerLayer;
use mantle_wasm_consumer_verifier::ConsumerMemberStatus;
use mantle_wasm_consumer_verifier::ConsumerRoot;
use mantle_wasm_consumer_verifier::ConsumerVerificationStatus;
use mantle_wasm_consumer_verifier::ShellError;
use mantle_wasm_consumer_verifier::verify_consumer_bundle;

const MEMBER_CONTENT_BYTES: usize = 32;
const REQUIRED_STAGE_KIND_COUNT: usize = 8;

/// Deterministic member content: `seed` repeated to a fixed length.
fn member_bytes(seed: u8) -> Vec<u8> {
    vec![seed; MEMBER_CONTENT_BYTES]
}

/// Temporary capability root that records declared members as real files.
struct FixtureRoot {
    dir: PathBuf,
}

impl FixtureRoot {
    fn new(label: &str) -> Self {
        let mut dir = std::env::temp_dir();
        dir.push(format!(
            "mantle-wasm-consumer-fixture-{}-{}-{}",
            label,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock advances")
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("members")).expect("fixture root creates members dir");
        Self { dir }
    }

    /// Declare one store object and write its exact bytes under the root.
    fn object(&self, path: &str, seed: u8) -> StoreObject {
        let bytes = member_bytes(seed);
        let relative = path.strip_prefix('/').expect("fixture path is absolute");
        let target = self.dir.join(relative);
        fs::create_dir_all(target.parent().expect("fixture path has a parent")).expect("fixture dir create");
        fs::write(target, &bytes).expect("fixture member write");
        StoreObject {
            logical_path: String::from(path),
            digest_blake3: Blake3Identity::from_slice(&bytes),
            size_bytes: bytes.len() as u64,
        }
    }

    /// Declare a store object reference without writing bytes.
    fn dangling_object(&self, path: &str, seed: u8) -> StoreObject {
        let bytes = member_bytes(seed);
        StoreObject {
            logical_path: String::from(path),
            digest_blake3: Blake3Identity::from_slice(&bytes),
            size_bytes: bytes.len() as u64,
        }
    }

    fn root(&self) -> ConsumerRoot {
        ConsumerRoot::open(&self.dir).expect("fixture root opens")
    }
}

impl Drop for FixtureRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn stage_receipt(fixture: &FixtureRoot, first_seed: u8) -> Vec<StageReceiptReference> {
    let kinds = [
        ComponentStageKind::PackageResolution,
        ComponentStageKind::Lock,
        ComponentStageKind::BindingGeneration,
        ComponentStageKind::Compilation,
        ComponentStageKind::Composition,
        ComponentStageKind::Virtualization,
        ComponentStageKind::BuildValidation,
        ComponentStageKind::OctetValidation,
    ];
    let mut receipts = Vec::new();
    for (offset, kind) in kinds.into_iter().enumerate() {
        let seed = first_seed + offset as u8;
        let receipt = fixture.object(&format!("/members/receipt-{seed:02}.json"), seed);
        receipts.push(StageReceiptReference {
            stage_key: format!("stage-{seed:02}"),
            kind,
            receipt_blake3: receipt.digest_blake3.clone(),
            receipt,
            artifact: None,
        });
    }
    receipts
}

fn valid_bundle(fixture: &FixtureRoot) -> MaterializationBundle {
    let request = MaterializationBundleRequest {
        name: String::from("consumer-fixture-component"),
        manifest_blake3: Blake3Identity::from_slice(&member_bytes(1)),
        cohort_blake3: Blake3Identity::from_slice(&member_bytes(2)),
        wit_inputs: vec![fixture.object("/members/world.wit", 3)],
        package_inputs: vec![],
        source_closure: fixture.object("/members/source-closure.json", 4),
        lock: fixture.object("/members/wkg.lock", 5),
        final_portable: fixture.object("/members/final.wasm", 6),
        expected_octet_profile_blake3: Blake3Identity::from_slice(&member_bytes(7)),
        expected_runtime_profile_blake3: Blake3Identity::from_slice(&member_bytes(8)),
        stage_receipts: stage_receipt(fixture, 40),
        wizer: None,
        aot: None,
        non_claims: vec![
            String::from(REQUIRED_RUNTIME_AUTHORITY_NON_CLAIM),
            String::from(REQUIRED_RELEASE_ELIGIBILITY_NON_CLAIM),
            String::from("not-a-distribution"),
        ],
    };
    build_materialization_bundle(request).bundle.expect("fixture request must build a valid bundle")
}

#[test]
fn positive_complete_bundle_verifies_through_both_layers() {
    let fixture = FixtureRoot::new("positive");
    let bundle = valid_bundle(&fixture);
    let report = fixture.root().verify_bundle(bundle).expect("valid paths resolve");
    assert_eq!(report.status, ConsumerVerificationStatus::Verified);
    assert_eq!(report.layers_completed, vec![ConsumerLayer::Structural, ConsumerLayer::Bytes]);
    assert!(report.blockers.is_empty());
    assert!(report.members.iter().all(|member| member.status == Some(ConsumerMemberStatus::Matched)));
}

#[test]
fn facade_parity_matches_implementation_over_structural_corpus() {
    let fixture = FixtureRoot::new("parity");
    let bundle = valid_bundle(&fixture);
    let internal = crunch_wasm_component_core::verify_materialization_bundle(bundle.clone());
    let consumer = verify_consumer_bundle(bundle.clone());
    assert!(internal.bundle.is_some());
    assert_eq!(consumer.status, ConsumerVerificationStatus::Verified);
    assert!(consumer.blockers.is_empty());

    let mut tampered = bundle.clone();
    tampered.bundle_identity_blake3 = Blake3Identity::from_slice(&member_bytes(63));
    let internal_bad = crunch_wasm_component_core::verify_materialization_bundle(tampered.clone());
    let consumer_bad = verify_consumer_bundle(tampered);
    assert!(internal_bad.bundle.is_none());
    assert_eq!(consumer_bad.status, ConsumerVerificationStatus::Rejected);
    assert_eq!(consumer_bad.blockers.len(), internal_bad.blockers.len());

    let mut wrong_schema = bundle;
    wrong_schema.schema = String::from("unsupported-schema");
    let consumer_schema = verify_consumer_bundle(wrong_schema);
    assert_eq!(consumer_schema.status, ConsumerVerificationStatus::Rejected);
}

#[test]
fn negative_missing_stage_kind_rejects_before_bytes() {
    let fixture = FixtureRoot::new("missing-kind");
    let bundle = valid_bundle(&fixture);
    let mut request = request_from_bundle(&bundle);
    request.stage_receipts.retain(|receipt| receipt.kind != ComponentStageKind::Virtualization);
    assert_eq!(request.stage_receipts.len(), REQUIRED_STAGE_KIND_COUNT - 1);
    let reduced = build_materialization_bundle(request);
    assert!(reduced.bundle.is_none());
    assert!(reduced.blockers.iter().any(|blocker| blocker.code == "missing-bundle-stage-receipt"));
}

#[test]
fn negative_missing_member_blocks() {
    let fixture = FixtureRoot::new("missing-member");
    let bundle = valid_bundle(&fixture);
    fs::remove_file(fixture.dir.join("members/final.wasm")).expect("remove one member");
    let report = fixture.root().verify_bundle(bundle).expect("valid paths resolve");
    assert_eq!(report.status, ConsumerVerificationStatus::Blocked);
    assert!(report.members.iter().any(|member| member.status == Some(ConsumerMemberStatus::Missing)));
    assert!(report.members.iter().filter(|member| member.status == Some(ConsumerMemberStatus::Matched)).count() > 0);
}

#[test]
fn negative_drifted_member_bytes_reject() {
    let fixture = FixtureRoot::new("drift");
    let bundle = valid_bundle(&fixture);
    fs::write(fixture.dir.join("members/final.wasm"), member_bytes(9)).expect("rewrite drifted member");
    let report = fixture.root().verify_bundle(bundle).expect("valid paths resolve");
    assert_eq!(report.status, ConsumerVerificationStatus::Rejected);
    assert!(report.blockers.iter().any(|code| code == "consumer-member-bytes-drift"));
    assert!(report.members.iter().any(|member| member.status == Some(ConsumerMemberStatus::Mismatched)));
}

#[test]
fn negative_symlink_member_blocks() {
    let fixture = FixtureRoot::new("symlink");
    let bundle = valid_bundle(&fixture);
    fs::remove_file(fixture.dir.join("members/final.wasm")).expect("remove member for symlink");
    std::os::unix::fs::symlink("/etc/hostname", fixture.dir.join("members/final.wasm")).expect("create symlink member");
    let report = fixture.root().verify_bundle(bundle).expect("valid paths resolve");
    assert_eq!(report.status, ConsumerVerificationStatus::Blocked);
}

#[test]
fn negative_directory_member_blocks() {
    let fixture = FixtureRoot::new("directory-member");
    let bundle = valid_bundle(&fixture);
    fs::remove_file(fixture.dir.join("members/final.wasm")).expect("remove member for directory");
    fs::create_dir(fixture.dir.join("members/final.wasm")).expect("create directory member");
    let report = fixture.root().verify_bundle(bundle).expect("valid paths resolve");
    assert_eq!(report.status, ConsumerVerificationStatus::Blocked);
}

#[test]
fn negative_parent_traversal_is_rejected_by_the_shell() {
    let fixture = FixtureRoot::new("traversal");
    let mut bundle = valid_bundle(&fixture);
    let mut request = request_from_bundle(&bundle);
    request.lock = fixture.dangling_object("/../escape.lock", 5);
    bundle = rebuild(request);
    let error = fixture.root().verify_bundle(bundle).expect_err("traversal must fail the shell");
    assert!(matches!(error, ShellError::InvalidMemberPath));
}

#[test]
fn positive_shared_member_path_satisfies_both_roles() {
    let fixture = FixtureRoot::new("shared-path");
    let bundle = valid_bundle(&fixture);
    let shared_object = fixture.object("/members/shared.json", 5);
    let mut request = request_from_bundle(&bundle);
    request.lock = shared_object.clone();
    request.source_closure = shared_object.clone();
    let bundle = rebuild(request);
    let report = fixture.root().verify_bundle(bundle).expect("valid paths resolve");
    assert_eq!(report.status, ConsumerVerificationStatus::Verified);
    let shared: Vec<_> = report
        .members
        .iter()
        .filter(|member| member.role == "lock" || member.role == "source-closure")
        .collect();
    assert_eq!(shared.len(), 2);
    assert!(shared.iter().all(|member| member.status == Some(ConsumerMemberStatus::Matched)));
}

#[test]
fn negative_declared_oversize_member_rejects_as_drift() {
    let oversize_bytes: u64 = (1 << 30) + 1;
    let fixture = FixtureRoot::new("oversize");
    let bundle = valid_bundle(&fixture);
    let mut request = request_from_bundle(&bundle);
    let bytes = member_bytes(6);
    request.final_portable = StoreObject {
        logical_path: String::from("/members/final.wasm"),
        digest_blake3: Blake3Identity::from_slice(&bytes),
        size_bytes: oversize_bytes,
    };
    let bundle = rebuild(request);
    let report = fixture.root().verify_bundle(bundle).expect("valid paths resolve");
    assert_eq!(report.status, ConsumerVerificationStatus::Rejected);
    assert!(report.members.iter().any(|member| member.status == Some(ConsumerMemberStatus::Mismatched)));
}

#[test]
fn report_leaks_no_member_paths_and_binds_fixed_non_claims() {
    let fixture = FixtureRoot::new("leak");
    let bundle = valid_bundle(&fixture);
    let report = fixture.root().verify_bundle(bundle).expect("valid paths resolve");
    let serialized = serde_json::to_string(&report).expect("report serializes");
    assert!(!serialized.contains("/members/"));
    assert_eq!(report.non_claims.len(), 6);
    assert!(report.non_claims.iter().all(|claim| !claim.contains("proves")));
    assert_eq!(report.schema, "mantle-wasm-consumer-verification-report-v1");
}

#[test]
fn rejected_bundle_reports_carry_no_byte_layer_claims() {
    let fixture = FixtureRoot::new("rejected");
    let mut bundle = valid_bundle(&fixture);
    bundle.bundle_identity_blake3 = Blake3Identity::from_slice(&member_bytes(63));
    let report = verify_consumer_bundle(bundle);
    assert_eq!(report.status, ConsumerVerificationStatus::Rejected);
    assert!(report.members.iter().all(|member| member.status.is_none()));
    assert!(report.layers_completed.is_empty());
}

#[test]
fn kamacite_frozen_fixture_passes_structural_layer() {
    let raw = include_str!("fixtures/kamacite-mantle-hello-bundle.json");
    let bundle: MaterializationBundle = serde_json::from_str(raw).expect("frozen fixture parses");
    let report = verify_consumer_bundle(bundle.clone());
    assert_eq!(report.status, ConsumerVerificationStatus::Verified);
    assert_eq!(report.layers_completed, vec![ConsumerLayer::Structural]);

    // Byte layer over an empty capability root: every member is missing, so
    // the independent consumer observes `blocked`, never a byte claim.
    let fixture = FixtureRoot::new("kamacite-empty");
    let report = fixture.root().verify_bundle(bundle).expect("valid paths resolve");
    assert_eq!(report.status, ConsumerVerificationStatus::Blocked);
    assert!(report.members.iter().all(|member| member.status == Some(ConsumerMemberStatus::Missing)));
}

#[test]
fn second_consumer_world_and_member_graph_passes() {
    // A second consumer's world: two WIT inputs and a two-package member
    // graph with a carried AOT output bound to the final portable bytes.
    let fixture = FixtureRoot::new("second-consumer");
    let final_portable = fixture.object("/graphs/second/final.wasm", 21);
    let request = MaterializationBundleRequest {
        name: String::from("second-consumer-component"),
        manifest_blake3: Blake3Identity::from_slice(&member_bytes(31)),
        cohort_blake3: Blake3Identity::from_slice(&member_bytes(32)),
        wit_inputs: vec![
            fixture.object("/graphs/second/world-a.wit", 33),
            fixture.object("/graphs/second/world-b.wit", 34),
        ],
        package_inputs: vec![
            crunch_wasm_component_core::PackageMaterialization {
                package: String::from("second:alpha"),
                version: String::from("0.2.0"),
                registry: String::from("local"),
                protocol_digest: OciSha256Digest::parse(String::from(
                    "sha256:a11ba8ad11065a2b857612c159fa0132145b8ce13951a18c2a4650825dd7bfa0",
                ))
                .expect("fixture protocol digest parses"),
                object: fixture.object("/graphs/second/packages/alpha.wasm", 35),
            },
            crunch_wasm_component_core::PackageMaterialization {
                package: String::from("second:beta"),
                version: String::from("0.3.0"),
                registry: String::from("local"),
                protocol_digest: OciSha256Digest::parse(String::from(
                    "sha256:a11ba8ad11065a2b857612c159fa0132145b8ce13951a18c2a4650825dd7bfa0",
                ))
                .expect("fixture protocol digest parses"),
                object: fixture.object("/graphs/second/packages/beta.wasm", 36),
            },
        ],
        source_closure: fixture.object("/graphs/second/source.json", 37),
        lock: fixture.object("/graphs/second/wkg.lock", 38),
        final_portable: final_portable.clone(),
        expected_octet_profile_blake3: Blake3Identity::from_slice(&member_bytes(39)),
        expected_runtime_profile_blake3: Blake3Identity::from_slice(&member_bytes(41)),
        stage_receipts: {
            let mut receipts = stage_receipt(&fixture, 60);
            let artifact_receipt = fixture.object("/graphs/second/receipt-aot.json", 80);
            receipts.push(StageReceiptReference {
                stage_key: String::from("stage-aot"),
                kind: ComponentStageKind::Aot,
                receipt_blake3: artifact_receipt.digest_blake3.clone(),
                receipt: artifact_receipt,
                artifact: Some(final_portable),
            });
            receipts
        },
        wizer: None,
        aot: Some(AotAdmission {
            schema: String::from("mantle-wasm-aot-admission-v1"),
            source_component_blake3: Blake3Identity::from_slice(&member_bytes(21)),
            output: fixture.dangling_object("/graphs/second/final.aot.wasm", 42),
            target: String::from("x86_64-unknown-linux-gnu"),
            cpu_features: Vec::new(),
            trust_class: String::from("test"),
            wasmtime_configuration_blake3: Blake3Identity::from_slice(&member_bytes(43)),
            cohort_blake3: Blake3Identity::from_slice(&member_bytes(44)),
            wit_profile_blake3: Blake3Identity::from_slice(&member_bytes(45)),
            build_inputs_blake3: Blake3Identity::from_slice(&member_bytes(46)),
            receipt_blake3: Blake3Identity::from_slice(&member_bytes(47)),
        }),
        non_claims: vec![
            String::from(REQUIRED_RUNTIME_AUTHORITY_NON_CLAIM),
            String::from(REQUIRED_RELEASE_ELIGIBILITY_NON_CLAIM),
        ],
    };
    let built = build_materialization_bundle(request);
    let bundle = built.bundle.expect("second-consumer request builds");
    let report = fixture.root().verify_bundle(bundle).expect("valid paths resolve");
    assert_eq!(report.status, ConsumerVerificationStatus::Verified);
    assert!(report.members.iter().any(|member| member.role == "package-object"));
    assert!(report.members.iter().any(|member| member.role == "stage-artifact"));
}

fn request_from_bundle(bundle: &MaterializationBundle) -> crunch_wasm_component_core::MaterializationBundleRequest {
    crunch_wasm_component_core::MaterializationBundleRequest {
        name: bundle.name.clone(),
        manifest_blake3: bundle.manifest_blake3.clone(),
        cohort_blake3: bundle.cohort_blake3.clone(),
        wit_inputs: bundle.wit_inputs.clone(),
        package_inputs: bundle.package_inputs.clone(),
        source_closure: bundle.source_closure.clone(),
        lock: bundle.lock.clone(),
        final_portable: bundle.final_portable.clone(),
        expected_octet_profile_blake3: bundle.expected_octet_profile_blake3.clone(),
        expected_runtime_profile_blake3: bundle.expected_runtime_profile_blake3.clone(),
        stage_receipts: bundle.stage_receipts.clone(),
        wizer: bundle.wizer.clone(),
        aot: bundle.aot.clone(),
        non_claims: bundle.non_claims.clone(),
    }
}

fn rebuild(request: crunch_wasm_component_core::MaterializationBundleRequest) -> MaterializationBundle {
    crunch_wasm_component_core::build_materialization_bundle(request)
        .bundle
        .expect("fixture request rebuilds a bundle")
}
