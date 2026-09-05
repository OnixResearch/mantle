use super::*;

fn prefix_payloads(completed_count: usize) -> Vec<CheckpointPayloadIdentity> {
    test_payloads()
        .into_iter()
        .filter(|payload| match completed_count {
            1 => payload.payload_id == PAYLOAD_STAGEX_TRANSITION,
            count if count == STAGEX_PROVIDER_STAGE_INDEX + 1 => {
                matches!(payload.payload_id.as_str(), PAYLOAD_STAGEX_TRANSITION | PAYLOAD_STAGEX_PROVIDER)
            }
            count if count == NATIVE_PROVIDER_STAGE_INDEX + 1 => !matches!(
                payload.payload_id.as_str(),
                PAYLOAD_RUST_PROVIDER | PAYLOAD_TOOLCHAIN_CLOSURE | PAYLOAD_RUST_ACTION_TRUST
            ),
            count if count == PROVIDER_CHECKPOINT_STAGE_COUNT => true,
            _ => false,
        })
        .collect()
}

fn prefix_manifest(completed_count: usize) -> ProviderCheckpointManifest {
    build_dev_provider_prefix_manifest(
        &test_plan(DIGEST_A, DIGEST_B),
        test_observations().into_iter().take(completed_count).collect(),
        prefix_payloads(completed_count),
    )
    .unwrap()
}

#[test]
fn dev_prefix_round_trips_each_completed_provider_boundary() {
    let plan = test_plan(DIGEST_A, DIGEST_B);
    for completed_count in 1..=PROVIDER_CHECKPOINT_STAGE_COUNT {
        let manifest = prefix_manifest(completed_count);
        assert_eq!(manifest.schema, DEV_PROVIDER_PREFIX_SCHEMA);
        let admitted = admit_dev_provider_checkpoint(&plan, &manifest, &manifest.payloads, DIGEST_B).unwrap();
        assert_eq!(usize::try_from(admitted.completed_stage_count).unwrap(), completed_count);
        assert_eq!(manifest.payloads, prefix_payloads(completed_count));
        assert!(admit_promoted_provider_checkpoint(&plan, &manifest, &manifest.payloads, DIGEST_B).is_err());
    }
}

#[test]
fn dev_prefix_rejects_gaps_extra_payloads_and_changed_observations() {
    let plan = test_plan(DIGEST_A, DIGEST_B);
    let mut missing = prefix_payloads(PROVIDER_CHECKPOINT_STAGE_COUNT);
    missing.remove(0);
    assert!(build_dev_provider_prefix_manifest(&plan, test_observations(), missing).is_err());
    let observations = test_observations().into_iter().take(1).collect();
    assert!(build_dev_provider_prefix_manifest(&plan, observations, test_payloads()).is_err());
    let mut manifest = prefix_manifest(1);
    manifest.stages[0].output_role = ProofOutputRole::StagexProvider;
    assert!(admit_dev_provider_checkpoint(&plan, &manifest, &manifest.payloads, DIGEST_B).is_err());
    let manifest = prefix_manifest(1);
    let mut observed = manifest.payloads.clone();
    observed[0].digest_blake3 = DIGEST_D.to_string();
    assert!(admit_dev_provider_checkpoint(&plan, &manifest, &observed, DIGEST_B).is_err());
}

#[test]
fn dev_prefix_cannot_relabel_itself_as_promoted_or_legacy_complete() {
    let plan = test_plan(DIGEST_A, DIGEST_B);
    let mut manifest = prefix_manifest(1);
    manifest.origin = ProofCheckpointOrigin::PromotedExecution;
    assert!(admit_promoted_provider_checkpoint(&plan, &manifest, &manifest.payloads, DIGEST_B).is_err());
    manifest.origin = ProofCheckpointOrigin::DevExecution;
    manifest.schema = PROVIDER_CHECKPOINT_SCHEMA.to_string();
    assert!(admit_dev_provider_checkpoint(&plan, &manifest, &manifest.payloads, DIGEST_B).is_err());
}

#[test]
fn dev_prefix_rejects_empty_and_reordered_stages() {
    let plan = test_plan(DIGEST_A, DIGEST_B);
    assert!(build_dev_provider_prefix_manifest(&plan, Vec::new(), Vec::new()).is_err());
    let mut observations = test_observations();
    observations.swap(0, STAGEX_PROVIDER_STAGE_INDEX);
    assert!(build_dev_provider_prefix_manifest(&plan, observations, test_payloads()).is_err());
}
