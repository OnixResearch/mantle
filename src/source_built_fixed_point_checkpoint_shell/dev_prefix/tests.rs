use super::*;
use crate::source_built_fixed_point::ProofOutputRole;
use crate::source_built_fixed_point_checkpoint::PAYLOAD_STAGEX_PROVIDER;
use crate::source_built_fixed_point_checkpoint::PAYLOAD_STAGEX_TRANSITION;
use crate::source_built_fixed_point_checkpoint::checkpoint_test_plan;

const TEST_DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const TEST_DISK_BYTES_MAX: u64 = 1_048_576;

struct Fixture {
    root: tempfile::TempDir,
    store: PathBuf,
    plan: SourceBuiltFixedPointPlan,
    sources: Vec<CheckpointPayloadSource>,
    observations: Vec<ProviderCheckpointStageObservation>,
    limits: ProviderCheckpointLimits,
}

impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let store = root.path().join("cache");
        let limits = provider_checkpoint_limits(TEST_DISK_BYTES_MAX);
        let mut fixture = Self {
            root,
            store,
            limits,
            plan: checkpoint_test_plan(TEST_DIGEST, TEST_DIGEST),
            sources: Vec::new(),
            observations: Vec::new(),
        };
        fixture.add_payload(
            PAYLOAD_STAGEX_TRANSITION,
            ProofOutputRole::StagexTransition,
            CheckpointPayloadKind::PreservedTree,
        );
        fixture
    }

    fn add_payload(&mut self, id: &str, role: ProofOutputRole, kind: CheckpointPayloadKind) {
        let path = self.root.path().join(id);
        fs::create_dir(&path).unwrap();
        fs::write(path.join("member"), id.as_bytes()).unwrap();
        let relative = Path::new(CHECKPOINT_PAYLOAD_ROOT).join(id);
        let identity = observe_payload(&path, id, &relative, kind, self.limits).unwrap();
        self.sources.push(CheckpointPayloadSource {
            payload_id: id.to_string(),
            source_path: path,
            relative_path: relative,
            kind,
        });
        self.observations.push(ProviderCheckpointStageObservation {
            output_role: role,
            output_digest_blake3: identity.digest_blake3.clone(),
            semantic_output_digest_blake3: if role == ProofOutputRole::StagexProvider {
                TEST_DIGEST.to_string()
            } else {
                identity.digest_blake3.clone()
            },
            payload_digest_blake3: identity.digest_blake3,
            execution_evidence_digest_blake3: TEST_DIGEST.to_string(),
            producer_executable_digest_blake3: TEST_DIGEST.to_string(),
        });
    }

    fn publish(&self) -> PublishedProviderCheckpoint {
        publish_dev_provider_prefix(
            &self.store,
            &self.plan,
            self.observations.clone(),
            &self.sources,
            TEST_DIGEST,
            self.limits,
        )
        .unwrap()
    }

    fn admit(&self, publication: &PublishedProviderCheckpoint) -> AdmittedProviderCheckpoint {
        admit_dev_provider_prefix(
            &self.store,
            &self.plan,
            &publication.checkpoint_digest_blake3,
            TEST_DIGEST,
            self.limits,
        )
        .unwrap()
        .unwrap()
    }
}

#[test]
fn dev_prefix_objects_are_shared_across_successive_prefixes_and_restore_fresh() {
    let mut fixture = Fixture::new();
    let first = fixture.publish();
    let first_admitted = fixture.admit(&first);
    let original_object =
        payload_path(&first.checkpoint_root, &first_admitted.manifest, &first_admitted.manifest.payloads[0]).unwrap();
    fixture.add_payload(PAYLOAD_STAGEX_PROVIDER, ProofOutputRole::StagexProvider, CheckpointPayloadKind::Directory);
    let second = fixture.publish();
    let admitted = fixture.admit(&second);
    let transition = payload_by_id(&admitted.manifest, PAYLOAD_STAGEX_TRANSITION).unwrap();
    assert_eq!(payload_path(&second.checkpoint_root, &admitted.manifest, transition).unwrap(), original_object);
    assert!(!second.checkpoint_root.join(CHECKPOINT_PAYLOAD_ROOT).exists());
    assert_ne!(first.checkpoint_digest_blake3, second.checkpoint_digest_blake3);
    let restore_root = fixture.root.path().join("fresh-staging");
    fs::create_dir(&restore_root).unwrap();
    let requests = admitted
        .manifest
        .payloads
        .iter()
        .map(|payload| CheckpointPayloadRestore {
            payload_id: payload.payload_id.clone(),
            destination_path: restore_root.join(&payload.payload_id),
            kind: payload.kind,
        })
        .collect::<Vec<_>>();
    let restored = restore_dev_provider_checkpoint(admitted, &requests, fixture.limits).unwrap();
    assert_eq!(restored.created_payload_paths.len(), requests.len());
    for request in requests {
        assert_eq!(fs::read(request.destination_path.join("member")).unwrap(), request.payload_id.as_bytes());
    }
}

#[test]
fn dev_prefix_duplicate_publication_reuses_exact_objects() {
    let fixture = Fixture::new();
    let first = fixture.publish();
    let second = fixture.publish();
    assert_eq!(first.checkpoint_digest_blake3, second.checkpoint_digest_blake3);
    assert_eq!(second.disposition, CheckpointPublicationDisposition::ExistingIdentical);
}

#[test]
fn dev_prefix_mutated_object_rejects_without_overwrite() {
    let fixture = Fixture::new();
    let published = fixture.publish();
    let admitted = fixture.admit(&published);
    let object = payload_path(&published.checkpoint_root, &admitted.manifest, &admitted.manifest.payloads[0]).unwrap();
    fs::write(object.join("member"), b"changed").unwrap();
    assert!(
        admit_dev_provider_prefix(
            &fixture.store,
            &fixture.plan,
            &published.checkpoint_digest_blake3,
            TEST_DIGEST,
            fixture.limits
        )
        .is_err()
    );
    assert!(
        publish_dev_provider_prefix(
            &fixture.store,
            &fixture.plan,
            fixture.observations.clone(),
            &fixture.sources,
            TEST_DIGEST,
            fixture.limits
        )
        .is_err()
    );
    assert_eq!(fs::read(object.join("member")).unwrap(), b"changed");
}

#[test]
fn dev_prefix_rejection_preserves_an_identical_preexisting_restore() {
    let fixture = Fixture::new();
    let admitted = fixture.admit(&fixture.publish());
    let destination = fixture.root.path().join("existing-payload");
    copy_payload(&fixture.sources[0].source_path, &destination, fixture.sources[0].kind, fixture.limits).unwrap();
    let requests = [CheckpointPayloadRestore {
        payload_id: PAYLOAD_STAGEX_TRANSITION.to_string(),
        destination_path: destination.clone(),
        kind: fixture.sources[0].kind,
    }];
    let restored = restore_dev_provider_checkpoint(admitted, &requests, fixture.limits).unwrap();
    assert!(restored.created_payload_paths.is_empty());
    let result: Result<(), DevRestoreError> =
        validate_dev_provider_restore(restored, |_| Err(checkpoint_error("rejected fixture")));
    assert!(matches!(result, Err(DevRestoreError::Rejected(_))));
    assert_eq!(fs::read(destination.join("member")).unwrap(), PAYLOAD_STAGEX_TRANSITION.as_bytes());
}

#[test]
fn dev_prefix_invalid_shape_writes_no_cache_state() {
    let fixture = Fixture::new();
    assert!(
        publish_dev_provider_prefix(&fixture.store, &fixture.plan, Vec::new(), &[], TEST_DIGEST, fixture.limits)
            .is_err()
    );
    assert!(!fixture.store.exists());
}

#[cfg(unix)]
#[test]
fn dev_prefix_symlink_namespace_rejects_before_writes_through_it() {
    let fixture = Fixture::new();
    fs::create_dir(&fixture.store).unwrap();
    let outside = fixture.root.path().join("outside");
    fs::create_dir(&outside).unwrap();
    std::os::unix::fs::symlink(&outside, fixture.store.join(OBJECTS_SUBDIR)).unwrap();
    assert!(
        publish_dev_provider_prefix(
            &fixture.store,
            &fixture.plan,
            fixture.observations.clone(),
            &fixture.sources,
            TEST_DIGEST,
            fixture.limits
        )
        .is_err()
    );
    assert!(fs::read_dir(&outside).unwrap().next().is_none());
}

#[cfg(unix)]
#[test]
fn dev_prefix_symlink_manifest_rejects() {
    let fixture = Fixture::new();
    let published = fixture.publish();
    let moved = fixture.root.path().join("moved-manifest");
    fs::rename(&published.manifest_path, &moved).unwrap();
    std::os::unix::fs::symlink(&moved, &published.manifest_path).unwrap();
    assert!(
        admit_dev_provider_prefix(
            &fixture.store,
            &fixture.plan,
            &published.checkpoint_digest_blake3,
            TEST_DIGEST,
            fixture.limits
        )
        .is_err()
    );
    assert!(moved.is_file());
}
