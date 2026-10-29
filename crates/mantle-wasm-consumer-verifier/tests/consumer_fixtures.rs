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
        runtime_validation: crunch_wasm_component_core::RuntimeValidation::StandaloneSmoke,
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
fn negative_conflicting_member_identity_is_rejected() {
    let fixture = FixtureRoot::new("conflicting");
    let bundle = valid_bundle(&fixture);
    let mut request = request_from_bundle(&bundle);
    request.source_closure = fixture.dangling_object("/members/wkg.lock", 55);
    let bundle = rebuild(request);
    let error = fixture.root().verify_bundle(bundle).expect_err("conflicting identity must fail the shell");
    assert!(matches!(error, ShellError::ConflictingMemberIdentity));
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
        runtime_validation: crunch_wasm_component_core::RuntimeValidation::StandaloneSmoke,
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
        runtime_validation: bundle.runtime_validation.clone(),
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

fn stream_boundary_bytes() -> Vec<u8> {
    (0..65_537).map(|index| (index % 256) as u8).collect()
}

/// Build the closure with the real producer, without rewriting its identity.
fn producer_tree_bundle(fixture: &FixtureRoot) -> MaterializationBundle {
    use std::os::unix::ffi::OsStringExt;
    let bundle = valid_bundle(fixture);
    let input = fixture.dir.join("producer-input");
    fs::create_dir_all(input.join("a-dir/nested/empty")).unwrap();
    fs::create_dir_all(input.join("z-dir/empty")).unwrap();
    fs::write(input.join("a-dir/nested/lib.rs"), b"pub fn component() {}\n").unwrap();
    fs::write(input.join("a-dir.x"), stream_boundary_bytes()).unwrap();
    fs::write(input.join("z-dir/other"), b"other").unwrap();
    fs::write(input.join(std::ffi::OsString::from_vec(vec![b'z', 0xff])), b"native filename").unwrap();
    let mut source = crunch_wasm_component::copy_source_tree(&input, &fixture.dir.join("members/source-tree"))
        .expect("real producer copies nested/empty source tree");
    source.logical_path = String::from("/members/source-tree");
    let mut request = request_from_bundle(&bundle);
    request.source_closure = source;
    rebuild(request)
}

// r[verify mantle.wasm_source_closure_trees.identity]
// r[verify mantle.wasm_source_closure_trees.copy]
#[test]
fn producer_tree_verifies_and_copies_exactly_with_empty_directories() {
    use std::os::fd::AsRawFd;
    let fixture = FixtureRoot::new("producer-tree");
    let bundle = producer_tree_bundle(&fixture);
    let report = fixture.root().verify_bundle(bundle.clone()).unwrap();
    assert_eq!(report.status, ConsumerVerificationStatus::Verified);
    assert!(report.members.iter().all(|member| member.status == Some(ConsumerMemberStatus::Matched)));
    let destination = FixtureRoot::new("tree-copy");
    let opened = fs::File::open(&destination.dir).unwrap();
    let bridge = PathBuf::from(format!("/proc/self/fd/{}", opened.as_raw_fd()));
    let target = ConsumerRoot::open(&bridge).unwrap();
    let observation = fixture.root().copy_source_closure(&bundle, &target).unwrap();
    assert_eq!(observation.role, "source-closure");
    assert_eq!(observation.digest_blake3, bundle.source_closure.digest_blake3.to_string());
    assert_eq!(observation.size_bytes, bundle.source_closure.size_bytes);
    assert_eq!(observation.status, Some(ConsumerMemberStatus::Matched));
    let copied = destination.dir.join("members/source-tree");
    assert!(copied.join("a-dir/nested/empty").is_dir());
    assert!(copied.join("z-dir/empty").is_dir());
    assert_eq!(fs::read(copied.join("a-dir.x")).unwrap(), stream_boundary_bytes());
    let remeasured = crunch_wasm_component::copy_source_tree(&copied, &destination.dir.join("remeasured")).unwrap();
    assert_eq!(remeasured.digest_blake3, bundle.source_closure.digest_blake3);
    assert_eq!(remeasured.size_bytes, bundle.source_closure.size_bytes);
    assert!(matches!(
        fixture.root().copy_source_closure(&bundle, &target),
        Err(ShellError::DestinationUnavailable)
    ));
    assert_eq!(fs::read(copied.join("a-dir.x")).unwrap(), stream_boundary_bytes());
}

// r[verify mantle.wasm_source_closure_trees.identity]
#[test]
fn tree_bytes_name_empty_directory_and_size_drift_fail_closed() {
    for drift in ["bytes", "tail-byte", "name", "empty-dir", "size", "declared-size"] {
        let fixture = FixtureRoot::new(drift);
        let mut bundle = producer_tree_bundle(&fixture);
        let tree = fixture.dir.join("members/source-tree");
        match drift {
            "bytes" => fs::write(tree.join("z-dir/other"), b"OTHER").unwrap(),
            "tail-byte" => {
                let mut bytes = stream_boundary_bytes();
                bytes[65_536] ^= 0xff;
                fs::write(tree.join("a-dir.x"), bytes).unwrap();
            }
            "name" => fs::rename(tree.join("z-dir/other"), tree.join("z-dir/renamed")).unwrap(),
            "empty-dir" => fs::remove_dir(tree.join("z-dir/empty")).unwrap(),
            "size" => fs::write(tree.join("z-dir/other"), b"longer bytes").unwrap(),
            "declared-size" => {
                let mut request = request_from_bundle(&bundle);
                request.source_closure.size_bytes += 1;
                bundle = rebuild(request);
            }
            _ => unreachable!(),
        }
        let report = fixture.root().verify_bundle(bundle.clone()).unwrap();
        assert_eq!(report.status, ConsumerVerificationStatus::Rejected, "{drift}");
        let source = report.members.iter().find(|member| member.role == "source-closure").unwrap();
        assert_eq!(source.status, Some(ConsumerMemberStatus::Mismatched), "{drift}");
        let destination = FixtureRoot::new("rejected-copy");
        assert!(
            matches!(fixture.root().copy_source_closure(&bundle, &destination.root()), Err(ShellError::MemberMismatch)),
            "{drift}"
        );
        assert!(!destination.dir.join("members/source-tree").exists());
    }
}

// r[verify mantle.wasm_source_closure_trees.capabilities]
#[test]
fn tree_missing_symlink_fifo_and_socket_never_verify_or_copy() {
    use std::os::fd::AsRawFd;
    use std::os::unix::fs::symlink;
    for substitution in ["missing", "leaf-link", "parent-link", "closure-link", "fifo", "socket"] {
        let fixture = FixtureRoot::new(substitution);
        let bundle = producer_tree_bundle(&fixture);
        let tree = fixture.dir.join("members/source-tree");
        let mut listener = None;
        match substitution {
            "missing" => fs::remove_dir_all(&tree).unwrap(),
            "leaf-link" => {
                fs::remove_file(tree.join("z-dir/other")).unwrap();
                symlink(fixture.dir.join("producer-input/z-dir/other"), tree.join("z-dir/other")).unwrap();
            }
            "parent-link" => {
                fs::rename(tree.join("z-dir"), fixture.dir.join("saved-z-dir")).unwrap();
                symlink(fixture.dir.join("saved-z-dir"), tree.join("z-dir")).unwrap();
            }
            "closure-link" => {
                fs::rename(&tree, fixture.dir.join("saved-tree")).unwrap();
                symlink(fixture.dir.join("saved-tree"), &tree).unwrap();
            }
            "fifo" => {
                fs::remove_file(tree.join("z-dir/other")).unwrap();
                assert!(std::process::Command::new("mkfifo").arg(tree.join("z-dir/other")).status().unwrap().success());
            }
            "socket" => {
                fs::remove_file(tree.join("z-dir/other")).unwrap();
                let parent =
                    cap_std::fs::Dir::open_ambient_dir(tree.join("z-dir"), cap_std::ambient_authority()).unwrap();
                let socket_path = format!("/proc/self/fd/{}/other", parent.as_raw_fd());
                listener = Some(std::os::unix::net::UnixListener::bind(socket_path).unwrap());
            }
            _ => unreachable!(),
        }
        let report = fixture.root().verify_bundle(bundle.clone()).unwrap();
        assert_ne!(report.status, ConsumerVerificationStatus::Verified, "{substitution}");
        assert_ne!(
            report.members.iter().find(|member| member.role == "source-closure").unwrap().status,
            Some(ConsumerMemberStatus::Matched)
        );
        let destination = FixtureRoot::new("unsupported-copy");
        assert!(fixture.root().copy_source_closure(&bundle, &destination.root()).is_err(), "{substitution}");
        assert!(!destination.dir.join("members/source-tree").exists());
        drop(listener);
    }
}

// r[verify mantle.wasm_source_closure_trees.capabilities]
#[test]
fn opened_roots_survive_ambient_retargeting_and_reject_symlink_parents() {
    let fixture = FixtureRoot::new("root-retarget");
    let bundle = producer_tree_bundle(&fixture);
    let source = fixture.root();
    let destination = FixtureRoot::new("root-retarget-target");
    let target = destination.root();
    let source_saved = fixture.dir.with_extension("opened");
    let target_saved = destination.dir.with_extension("opened");
    fs::rename(&fixture.dir, &source_saved).unwrap();
    fs::rename(&destination.dir, &target_saved).unwrap();
    std::os::unix::fs::symlink(&destination.dir, &fixture.dir).unwrap();
    fs::create_dir(&destination.dir).unwrap();
    assert_eq!(source.verify_bundle(bundle.clone()).unwrap().status, ConsumerVerificationStatus::Verified);
    source.copy_source_closure(&bundle, &target).unwrap();
    assert!(target_saved.join("members/source-tree/z-dir/empty").is_dir());
    assert!(!destination.dir.join("members").exists());
    fs::remove_dir_all(&source_saved).unwrap();
    fs::remove_dir_all(&target_saved).unwrap();

    let fixture = FixtureRoot::new("symlink-parent");
    let bundle = valid_bundle(&fixture);
    fs::rename(fixture.dir.join("members"), fixture.dir.join("saved")).unwrap();
    std::os::unix::fs::symlink("saved", fixture.dir.join("members")).unwrap();
    assert_eq!(fixture.root().verify_bundle(bundle.clone()).unwrap().status, ConsumerVerificationStatus::Blocked);
    assert!(fixture.root().copy_source_closure(&bundle, &FixtureRoot::new("parent-link-copy").root()).is_err());
}

// r[verify mantle.wasm_source_closure_trees.bounds]
#[test]
fn trees_reject_entry_depth_and_byte_bound_overruns() {
    for bound in ["entries", "depth", "file-bytes", "declared-bytes"] {
        let fixture = FixtureRoot::new(bound);
        let mut bundle = producer_tree_bundle(&fixture);
        let tree = fixture.dir.join("members/source-tree");
        match bound {
            "entries" => {
                fs::remove_dir_all(&tree).unwrap();
                fs::create_dir(&tree).unwrap();
                for index in 0..10_001 {
                    fs::write(tree.join(format!("file-{index:05}")), b"").unwrap();
                }
            }
            "depth" => {
                let mut path = tree.clone();
                for _ in 0..65 {
                    path.push("d");
                }
                fs::create_dir_all(path).unwrap();
            }
            "file-bytes" => {
                fs::OpenOptions::new()
                    .write(true)
                    .open(tree.join("z-dir/other"))
                    .unwrap()
                    .set_len(536_870_913)
                    .unwrap();
            }
            "declared-bytes" => {
                let mut request = request_from_bundle(&bundle);
                request.source_closure.size_bytes = 536_870_913;
                bundle = rebuild(request);
            }
            _ => unreachable!(),
        }
        assert_eq!(
            fixture.root().verify_bundle(bundle.clone()).unwrap().status,
            ConsumerVerificationStatus::Rejected,
            "{bound}"
        );
        let destination = FixtureRoot::new("overbound-copy");
        assert!(fixture.root().copy_source_closure(&bundle, &destination.root()).is_err(), "{bound}");
        assert!(!destination.dir.join("members/source-tree").exists());
    }
}

// r[verify mantle.wasm_source_closure_trees.compatibility]
#[test]
fn file_source_closure_copies_without_clobbering_and_checks_declared_length() {
    let fixture = FixtureRoot::new("file-source-copy");
    let bundle = valid_bundle(&fixture);
    let destination = FixtureRoot::new("file-source-target");
    fixture.root().copy_source_closure(&bundle, &destination.root()).unwrap();
    let target = destination.dir.join("members/source-closure.json");
    assert_eq!(fs::read(&target).unwrap(), member_bytes(4));
    fs::write(&target, b"private existing content").unwrap();
    assert!(matches!(
        fixture.root().copy_source_closure(&bundle, &destination.root()),
        Err(ShellError::DestinationUnavailable)
    ));
    assert_eq!(fs::read(&target).unwrap(), b"private existing content");
    let mut request = request_from_bundle(&bundle);
    request.source_closure = bundle.lock.clone();
    request.source_closure.size_bytes += 1;
    let conflict = rebuild(request);
    assert!(matches!(fixture.root().verify_bundle(conflict.clone()), Err(ShellError::ConflictingMemberIdentity)));
    assert!(matches!(
        fixture.root().copy_source_closure(&conflict, &destination.root()),
        Err(ShellError::ConflictingMemberIdentity)
    ));
}

// r[verify mantle.wasm_source_closure_trees.bounds]
#[test]
fn producer_native_backslash_name_is_explicitly_unsupported_without_reframing() {
    let fixture = FixtureRoot::new("unsupported-native-path");
    let bundle = valid_bundle(&fixture);
    let input = fixture.dir.join("native-input");
    fs::create_dir(&input).unwrap();
    fs::write(input.join("native\\name"), b"exact native bytes").unwrap();
    let mut source = crunch_wasm_component::copy_source_tree(&input, &fixture.dir.join("members/native-tree")).unwrap();
    source.logical_path = String::from("/members/native-tree");
    let mut request = request_from_bundle(&bundle);
    request.source_closure = source;
    let bundle = rebuild(request);
    assert_ne!(fixture.root().verify_bundle(bundle.clone()).unwrap().status, ConsumerVerificationStatus::Verified);
    let destination = FixtureRoot::new("unsupported-native-copy");
    assert!(fixture.root().copy_source_closure(&bundle, &destination.root()).is_err());
    assert!(!destination.dir.join("members/native-tree").exists());
}

// r[verify mantle.wasm_source_closure_trees.compatibility]
#[test]
fn directory_source_shared_with_file_role_is_not_admitted() {
    let fixture = FixtureRoot::new("tree-file-role");
    let bundle = producer_tree_bundle(&fixture);
    let mut request = request_from_bundle(&bundle);
    request.lock = bundle.source_closure.clone();
    let bundle = rebuild(request);
    assert_eq!(fixture.root().verify_bundle(bundle.clone()).unwrap().status, ConsumerVerificationStatus::Blocked);
    let destination = FixtureRoot::new("tree-file-role-target");
    assert!(matches!(
        fixture.root().copy_source_closure(&bundle, &destination.root()),
        Err(ShellError::UnsupportedMember)
    ));
    assert!(!destination.dir.join("members/source-tree").exists());
}

// r[verify mantle.wasm_source_closure_trees.copy]
#[test]
fn source_copy_does_not_measure_unrelated_members_or_follow_destination_links() {
    let fixture = FixtureRoot::new("source-only-copy");
    let bundle = producer_tree_bundle(&fixture);
    fs::remove_file(fixture.dir.join("members/final.wasm")).unwrap();
    let destination = FixtureRoot::new("source-only-target");
    fixture.root().copy_source_closure(&bundle, &destination.root()).unwrap();
    assert!(destination.dir.join("members/source-tree/z-dir/empty").is_dir());
    assert!(!destination.dir.join("members/final.wasm").exists());
    assert_eq!(fixture.root().verify_bundle(bundle.clone()).unwrap().status, ConsumerVerificationStatus::Blocked);

    let linked_destination = FixtureRoot::new("linked-target");
    fs::remove_dir(linked_destination.dir.join("members")).unwrap();
    let outside = FixtureRoot::new("outside-target");
    std::os::unix::fs::symlink(&outside.dir, linked_destination.dir.join("members")).unwrap();
    assert!(fixture.root().copy_source_closure(&bundle, &linked_destination.root()).is_err());
    assert!(!outside.dir.join("source-tree").exists());
}

// r[verify mantle.wasm_source_closure_trees.bounds]
#[test]
fn producer_depth_64_directory_with_depth_65_file_remains_admitted() {
    let fixture = FixtureRoot::new("depth-boundary");
    let bundle = valid_bundle(&fixture);
    let input = fixture.dir.join("deep-input");
    let mut deepest = input.clone();
    for _ in 0..64 {
        deepest.push("d");
    }
    fs::create_dir_all(&deepest).unwrap();
    fs::write(deepest.join("file"), b"depth boundary").unwrap();
    let mut source = crunch_wasm_component::copy_source_tree(&input, &fixture.dir.join("members/deep-tree")).unwrap();
    source.logical_path = String::from("/members/deep-tree");
    let mut request = request_from_bundle(&bundle);
    request.source_closure = source;
    let bundle = rebuild(request);
    assert_eq!(fixture.root().verify_bundle(bundle.clone()).unwrap().status, ConsumerVerificationStatus::Verified);
    let destination = FixtureRoot::new("deep-target");
    fixture.root().copy_source_closure(&bundle, &destination.root()).unwrap();
    let copied = destination.dir.join("members/deep-tree").join(deepest.strip_prefix(&input).unwrap()).join("file");
    assert_eq!(fs::read(copied).unwrap(), b"depth boundary");
}

// r[verify mantle.wasm_source_closure_trees.compatibility]
#[test]
fn file_source_copy_rejects_content_length_and_missing_source() {
    for drift in ["content", "length", "missing"] {
        let fixture = FixtureRoot::new("file-copy-drift");
        let bundle = valid_bundle(&fixture);
        let source = fixture.dir.join("members/source-closure.json");
        match drift {
            "content" => fs::write(&source, member_bytes(99)).unwrap(),
            "length" => fs::write(&source, b"short").unwrap(),
            "missing" => fs::remove_file(&source).unwrap(),
            _ => unreachable!(),
        }
        let destination = FixtureRoot::new("file-drift-target");
        let result = fixture.root().copy_source_closure(&bundle, &destination.root());
        if drift == "missing" {
            assert!(matches!(result, Err(ShellError::MemberUnavailable)));
        } else {
            assert!(matches!(result, Err(ShellError::MemberMismatch)));
        }
        assert_eq!(fs::read(fixture.dir.join("members/wkg.lock")).unwrap(), member_bytes(5));
    }
}

// r[verify mantle.wasm_source_closure_trees.bounds]
#[test]
fn source_copy_accepts_member_limit_and_rejects_one_more_before_copy() {
    const MEMBER_LIMIT: usize = 8_192;
    let fixture = FixtureRoot::new("copy-member-limit");
    let mut bundle = valid_bundle(&fixture);
    let fixed_members = mantle_wasm_consumer_verifier::facade::declared_member_objects(&bundle)
        .len()
        .checked_sub(bundle.wit_inputs.len())
        .expect("fixture includes its WIT inputs");
    let wit = bundle.wit_inputs[0].clone();
    // This API copies source only; full structural admission is separate.
    let wit_count = MEMBER_LIMIT.checked_sub(fixed_members).expect("fixture fits the member limit");
    bundle.wit_inputs.resize(wit_count, wit.clone());
    let accepted = FixtureRoot::new("copy-member-limit-accepted");
    let observation = fixture.root().copy_source_closure(&bundle, &accepted.root()).unwrap();
    assert_eq!(observation.status, Some(ConsumerMemberStatus::Matched));
    assert_eq!(fs::read(accepted.dir.join("members/source-closure.json")).unwrap(), member_bytes(4));

    bundle.wit_inputs.push(wit);
    let rejected = FixtureRoot::new("copy-member-limit-rejected");
    assert!(matches!(
        fixture.root().copy_source_closure(&bundle, &rejected.root()),
        Err(ShellError::BoundExceeded)
    ));
    assert!(!rejected.dir.join("members/source-closure.json").exists());
}
