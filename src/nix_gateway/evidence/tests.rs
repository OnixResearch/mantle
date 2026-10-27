use crunch_remote_core::attempt::RemoteAttemptId;
use crunch_remote_core::attempt::RemoteFenceGeneration;
use crunch_remote_core::attempt::RemoteJobId;

use super::*;
use crate::remote_build::gateway::GatewayPhase;
use crate::remote_build::gateway::GatewayTransition;

// Adapter fixtures only. These do not stand in for retained runtime artifacts.
fn fixture() -> (GatewayAttemptRecord, quota::Entry) {
    let drv_path = "/nix/store/00000000000000000000000000000000-fixture.drv".to_owned();
    let digest = *blake3::hash(drv_path.as_bytes()).as_bytes();
    let binding = crate::remote_build::RemoteProductionAttemptBinding {
        job_id: RemoteJobId::new("job-fixture").unwrap(),
        attempt_id: RemoteAttemptId::new("attempt-fixture").unwrap(),
        fence_generation: RemoteFenceGeneration::INITIAL,
    };
    let record = GatewayAttemptRecord {
        public_id: "a".repeat(64),
        subject: "subject".into(),
        account: "account".into(),
        project: "project".into(),
        authority_policy: "b".repeat(64),
        authority_decision: "c".repeat(64),
        endpoint: "worker".into(),
        platform: "x86_64-linux".into(),
        reuse: None,
        published_evidence: None,
        retries: vec![],
        selection_json: None,
        transitions: vec![
            GatewayTransition {
                phase: GatewayPhase::Queued,
                sequence: 1,
                observed_unix_s: 1_760_000_000,
                reason: "admitted".into(),
            },
            GatewayTransition {
                phase: GatewayPhase::Running,
                sequence: 2,
                observed_unix_s: 1_760_000_001,
                reason: "dispatched".into(),
            },
        ],
        drv_path,
        request_digest: digest,
        request_id: "request-fixture".into(),
        binding: Some(binding.clone()),
        dispatch_started: true,
        cancel_requested: false,
        local_teardown_confirmed: false,
        recovery_uncertain: false,
        result_evidence: None,
        outputs: vec![],
        produced_outputs: vec![],
        usage: None,
        public_log: vec![],
    };
    let entry = quota::Entry {
        reservation: quota::Reservation {
            job: binding.job_id,
            attempt: binding.attempt_id,
            fence: binding.fence_generation,
            project: record.project.clone(),
            account: record.account.clone(),
            request_digest: hex(&digest),
            units: 100,
            schedule: quota::UnitSchedule {
                version: 1,
                unit: "compute-unit".into(),
                window_id: "window".into(),
                window_start_s: 1,
                window_end_s: 2_000_000_000,
                cpu_ms_per_unit: 1000,
                memory_byte_ms_per_unit: 1000,
                executor_output_bytes_per_unit: 1000,
            },
        },
        reconciliation: quota::Reconciliation::Reserved,
    };
    (record, entry)
}
fn produce<'a>(record: &'a GatewayAttemptRecord, entry: &quota::Entry) -> GatewayEvidence<'a> {
    let mut evidence = GatewayEvidence::new(record, [1; 32], entry).unwrap();
    evidence.record_durable_facts().unwrap();
    evidence
}
fn retained_output(record: &mut GatewayAttemptRecord) {
    record.produced_outputs = vec![crate::remote_build::RemoteProducedOutput {
        name: "out".into(),
        logical_path: record.outputs[0].clone(),
        content_digest_blake3: "e".repeat(64),
        size_bytes: 1,
        path_info_signing_key_id: "fixture-key".into(),
        artifact_attestation_digest_blake3: "f".repeat(64),
        path_info: None,
        nar_payload_digest_blake3: None,
        nar_payload_size_bytes: None,
    }];
}

#[test]
fn partial_and_failed_exports_preserve_actual_quota_and_recorded_roles() {
    let (mut record, mut entry) = fixture();
    let partial = produce(&record, &entry).finish().unwrap();
    assert!(partial.bundle.build_results.is_empty());
    assert_eq!(partial.bundle.profile.claimed_observation_classes, vec!["usage"]);
    assert_eq!(partial.bundle.transitions[0].prior_state, "queued");
    assert_eq!(partial.bundle.transitions[0].next_state, "running");
    assert_eq!(partial.bundle.transitions[0].sequence, 2);
    assert_eq!(partial.bundle.usage_records[0].role, v::BuildServiceRole::RecordedOnly);
    assert!(v::validate_build_service_bundle(&partial.bundle).valid);
    entry.reconciliation = quota::Reconciliation::Pending(quota::PendingReason::PartialObservation);
    record.transitions.push(GatewayTransition {
        phase: GatewayPhase::Failed,
        sequence: 3,
        observed_unix_s: 1_760_000_002,
        reason: "worker-lost".into(),
    });
    let failed = produce(&record, &entry).finish().unwrap();
    assert_eq!(failed.bundle.transitions[1].next_state, "failed");
    let outcome: serde_json::Value = serde_json::from_str(&failed.bundle.usage_records[1].outcome).unwrap();
    assert_eq!(outcome, serde_json::json!({"Pending":"PartialObservation"}));
    assert!(failed.bundle.build_results.is_empty());
}

#[test]
fn signed_result_fixture_is_separate_from_recorded_charge_and_nonclaims() {
    let (mut record, mut entry) = fixture();
    record.result_evidence = Some([2; 32]);
    record.outputs = vec!["/nix/store/00000000000000000000000000000000-output".into()];
    retained_output(&mut record);
    entry.reconciliation = quota::Reconciliation::Settled {
        event_digest: "d".repeat(64),
        charged_units: 12,
    };
    let mut evidence = produce(&record, &entry);
    evidence
        .record_result(
            vec![v::BuildOutputRef {
                path: record.outputs[0].clone(),
                content_hash: "e".repeat(64),
            }],
            "x86_64-linux",
            "fixture-policy",
            "public-signature-reference",
            "public-cas-observation",
        )
        .unwrap();
    let exported = evidence.finish().unwrap();
    assert_eq!(exported.bundle.build_results[0].role, v::BuildServiceRole::Signed);
    assert_eq!(exported.bundle.usage_records[1].role, v::BuildServiceRole::RecordedOnly);
    assert_eq!(exported.bundle.profile.non_claims, boundary());
    assert_eq!(verify_bytes(&exported.bytes, &exported.digest_blake3).unwrap(), exported.bundle);
}

#[test]
fn missing_mixed_scope_conflicting_charge_and_secret_facts_fail_closed() {
    let (record, mut entry) = fixture();
    entry.reservation.project = "foreign-project".into();
    assert!(GatewayEvidence::new(&record, [1; 32], &entry).is_err());
    let (mut record, mut entry) = fixture();
    record.binding = None;
    assert!(GatewayEvidence::new(&record, [1; 32], &entry).is_err());
    let (record, _) = fixture();
    entry.reconciliation = quota::Reconciliation::Settled {
        event_digest: "d".repeat(64),
        charged_units: 101,
    };
    assert!(GatewayEvidence::new(&record, [1; 32], &entry).is_err());
    let (record, entry) = fixture();
    assert!(GatewayEvidence::new(&record, [1; 32], &entry).unwrap().finish().is_err());
    let mut evidence = produce(&record, &entry);
    let mut duplicate = evidence.bundle.usage_records[0].clone();
    duplicate.outcome = "different-charge".into();
    evidence.bundle.usage_records.push(duplicate);
    assert!(evidence.finish().is_err());
    let mut evidence = produce(&record, &entry);
    evidence.bundle.authority_decisions[0].subject_projection = "Bearer credential".into();
    assert!(evidence.finish().is_err());
}

#[test]
fn retained_content_tamper_unknown_fields_and_role_promotion_fail() {
    let (record, entry) = fixture();
    let exported = produce(&record, &entry).finish().unwrap();
    let mut bytes = exported.bytes.clone();
    bytes[0] = b'!';
    assert!(verify_bytes(&bytes, &exported.digest_blake3).is_err());
    let mut value = serde_json::to_value(&exported.bundle).unwrap();
    value["attempts"][0]["credential"] = "private-value".into();
    let bytes = serde_json::to_vec(&value).unwrap();
    assert!(verify_bytes(&bytes, blake3::hash(&bytes).to_hex().as_str()).is_err());
    let text = std::str::from_utf8(&exported.bytes).unwrap();
    let duplicate = format!("{{\"profile\":{{\"secret\":\"Bearer hidden\"}},{}", &text[1..]).into_bytes();
    assert!(verify_bytes(&duplicate, blake3::hash(&duplicate).to_hex().as_str()).is_err());
    let mut evidence = produce(&record, &entry);
    evidence.bundle.usage_records[0].role = v::BuildServiceRole::Verified;
    assert!(evidence.finish().is_err());
}

#[test]
fn unauthorized_reuse_is_rejected_despite_linked_result() {
    let (mut record, entry) = fixture();
    record.result_evidence = Some([2; 32]);
    record.outputs = vec!["/nix/store/00000000000000000000000000000000-output".into()];
    retained_output(&mut record);
    let mut evidence = produce(&record, &entry);
    let result = evidence
        .record_result(
            vec![v::BuildOutputRef {
                path: record.outputs[0].clone(),
                content_hash: "e".repeat(64),
            }],
            "x86_64-linux",
            "policy",
            "signature",
            "cas",
        )
        .unwrap();
    let mut consumer = evidence.bundle.service_requests[0].clone();
    consumer.request_id = "consumer".into();
    let consumer_identity = v::service_request_identity(&consumer);
    evidence.bundle.service_requests.push(consumer);
    // It is deliberately linked to the submit decision, not consumer result-read authority.
    evidence.bundle.reuse_edges.push(v::ReuseEdgeRecord {
        producer_result_identity: result,
        consumer_request_identity: consumer_identity,
        sharing_policy: "explicit".into(),
        authorization_decision_identity: Some(v::authority_decision_identity(&evidence.bundle.authority_decisions[0])),
        request_compatible: true,
        platform_compatible: true,
        output_check_ref: Some("checked".into()),
        signature_ref: Some("signature".into()),
        cas_observation: Some("cas".into()),
        usability_outcome: "usable".into(),
        role: v::BuildServiceRole::Verified,
        non_claims: boundary(),
    });
    assert!(evidence.finish().is_err());
}

#[cfg(target_os = "linux")]
#[test]
fn publication_is_private_idempotent_and_never_follows_destination_symlink() {
    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::fs::symlink;
    let (record, entry) = fixture();
    let exported = produce(&record, &entry).finish().unwrap();
    let root = tempfile::tempdir().unwrap();
    std::fs::set_permissions(root.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let directory = std::fs::File::open(root.path()).unwrap();
    let leaf = exported.publish(&directory).unwrap();
    assert_eq!(exported.publish(&directory).unwrap(), leaf);
    assert_eq!(read_publication(&directory, &leaf).unwrap(), exported.bundle);
    assert!(read_publication(&directory, &format!("../{leaf}")).is_err());
    assert!(read_publication(&directory, &format!("/{leaf}")).is_err());
    let bytes = std::fs::read(root.path().join(&leaf)).unwrap();
    assert_eq!(verify_bytes(&bytes, &exported.digest_blake3).unwrap(), exported.bundle);
    let second = tempfile::tempdir().unwrap();
    std::fs::set_permissions(second.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let victim = second.path().join("victim");
    std::fs::write(&victim, b"preserved").unwrap();
    symlink(&victim, second.path().join(&leaf)).unwrap();
    assert!(exported.publish(&std::fs::File::open(second.path()).unwrap()).is_err());
    assert_eq!(std::fs::read(victim).unwrap(), b"preserved");
}

fn selection_fixture() -> crunch_build::distributed::PolicyEvaluation {
    use crunch_build::distributed::CatalogCandidate;
    use crunch_build::distributed::RemoteResourceRequirements;
    use crunch_build::distributed::RemoteResourceVector;
    use crunch_build::distributed::RemoteWorkerResourceInventory;
    use crunch_build::distributed::evaluate_resource_policy;
    let q = resource::Quantities {
        cpu_units: 1,
        memory_bytes: 1024,
        scratch_bytes: 1024,
    };
    let features = resource::Features {
        architecture: "x86_64".into(),
        platform: "linux".into(),
        trust_domain: "reviewed".into(),
        isolation: "sandbox".into(),
        kvm: false,
    };
    let class = resource::MachineClass {
        id: "small".into(),
        features: features.clone(),
        quantities: q,
        order: 1,
        reservation_units: 1,
        available: true,
        existing_admission_compatible: true,
    };
    let vector = RemoteResourceVector {
        cpu_units: 1,
        memory_bytes: 1024,
        scratch_bytes: 1024,
        accelerators: vec![],
        named_tokens: vec![],
    };
    let requirements = RemoteResourceRequirements {
        quantities: vector.clone(),
        semantic_accelerator_classes: vec![],
    };
    let inventory = RemoteWorkerResourceInventory { total: vector };
    let policy = resource::SelectionPolicy {
        version: 1,
        min_samples: 1,
        max_age_s: 100,
        margin_basis_points: 0,
        max_measurement: q,
        max_duration_ms: 1000,
        max_io_bytes: 1024,
    };
    let controls = resource::Controls {
        mode: resource::SelectionMode::ObserveOnly,
        project_opted_in: false,
        selection_enabled: false,
        oom_retry_enabled: false,
        quota_enforcement_enabled: false,
        sharing_enabled: false,
    };
    evaluate_resource_policy(
        &requirements,
        &features,
        "family",
        vec![CatalogCandidate {
            class,
            worker_endpoint_id: "worker",
            inventory: &inventory,
            live_leases: &[],
        }],
        &mut [],
        1_760_000_001,
        resource::QuotaFacts {
            project_remaining_units: 100,
            account_remaining_units: 100,
        },
        policy,
        controls,
    )
    .unwrap()
}

#[test]
fn selection_exports_replayed_policy_and_rejects_changed_replay_or_decision() {
    let (record, entry) = fixture();
    let evaluation = selection_fixture();
    let mut evidence = produce(&record, &entry);
    evidence.record_selection(&evaluation).unwrap();
    let exported = evidence.finish().unwrap();
    assert_eq!(exported.bundle.resource_decisions[0].role, v::BuildServiceRole::Verified);
    assert_eq!(
        exported.bundle.resource_decisions[0].selected_class_identity,
        v::machine_class_identity(&exported.bundle.machine_classes[0])
    );
    let mut changed = evaluation.clone();
    changed.replay_input_json[0] = b'!';
    assert!(produce(&record, &entry).record_selection(&changed).is_err());
    let mut changed = evaluation;
    changed.decision.active.as_mut().unwrap().class_index = 10;
    assert!(produce(&record, &entry).record_selection(&changed).is_err());
}

#[test]
fn explicit_reuse_keeps_producer_and_consumer_request_and_authority_distinct() {
    let (mut producer, entry) = fixture();
    producer.result_evidence = Some([2; 32]);
    producer.outputs = vec!["/nix/store/00000000000000000000000000000000-output".into()];
    retained_output(&mut producer);
    let mut evidence = produce(&producer, &entry);
    let result = evidence
        .record_result(
            vec![v::BuildOutputRef {
                path: producer.outputs[0].clone(),
                content_hash: "e".repeat(64),
            }],
            "x86_64-linux",
            "policy",
            "signature",
            "producer-cas",
        )
        .unwrap();
    let producer_bundle = evidence.finish().unwrap().bundle;
    let (mut consumer, mut entry) = fixture();
    consumer.public_id = "b".repeat(64);
    consumer.project = "consumer".into();
    let binding = consumer.binding.as_mut().unwrap();
    binding.job_id = RemoteJobId::new("consumer-job").unwrap();
    binding.attempt_id = RemoteAttemptId::new("consumer-attempt").unwrap();
    entry.reservation.job = binding.job_id.clone();
    entry.reservation.attempt = binding.attempt_id.clone();
    entry.reservation.project = consumer.project.clone();
    let mut evidence = produce(&consumer, &entry);
    let authority = evidence
        .record_authority(PublicAuthority {
            subject: &consumer.subject,
            account: &consumer.account,
            project: &consumer.project,
            policy_identity: &consumer.authority_policy,
            operation: "result-read",
            decision_id: &"d".repeat(64),
        })
        .unwrap();
    let controls = resource::Controls {
        mode: resource::SelectionMode::ProjectOptIn,
        project_opted_in: true,
        selection_enabled: false,
        oom_retry_enabled: false,
        quota_enforcement_enabled: false,
        sharing_enabled: true,
    };
    let checks = resource::SharingChecks {
        request_and_action_match: true,
        trusted_producer_signature: true,
        policy_compatible: true,
        platform_compatible: true,
        verified_output_identities: true,
        current_cas_available: true,
    };
    evidence
        .record_reuse(ReuseFacts {
            producer_bundle: &producer_bundle,
            producer_result_identity: &result,
            producer_project: "project",
            controls,
            scope: Some(resource::SharingScope {
                id: "reviewed-sharing",
                producers: &["project"],
                consumers: &["consumer"],
            }),
            checks,
            authorization_identity: &authority,
            output_check_ref: "current-output-check",
            signature_ref: "signature",
            cas_observation: "current-consumer-cas",
        })
        .unwrap();
    let bundle = evidence.finish().unwrap().bundle;
    assert_eq!(bundle.reuse_edges[0].authorization_decision_identity, Some(authority));
    assert_ne!(bundle.reuse_edges[0].consumer_request_identity, bundle.build_results[0].request_identity);
    assert!(v::validate_build_service_bundle(&bundle).valid);
}

#[test]
fn retry_requires_positive_platform_evidence_and_links_new_fenced_attempt() {
    let evaluation = selection_fixture();
    let prior = &evaluation.classes[0];
    let mut larger = prior.clone();
    larger.id = "large".into();
    larger.order = 2;
    larger.quantities.memory_bytes = 2048;
    let classes = [prior.clone(), larger];
    let (record, entry) = fixture();
    let binding = record.binding.as_ref().unwrap();
    let observation = resource::Observation {
        schema: resource::OBSERVATION_SCHEMA.into(),
        attempt_id: binding.attempt_id.clone(),
        action_family: "family".into(),
        features: prior.features.clone(),
        policy_version: 1,
        machine_class: prior.id.clone(),
        declared: prior.quantities,
        selected: prior.quantities,
        measured: prior.quantities.into(),
        scratch_sampled_peak_bytes: None,
        queue_ms: Some(0),
        execution_ms: Some(10),
        terminal_unix_s: 1_760_000_002,
        cpu_ms: Some(5),
        io_bytes: Some(0),
        transfer_bytes: Some(0),
        executor_output_bytes: None,
        process_peak_rss_bytes: None,
        oom: resource::OomCategory::LinuxCgroupV2Kill,
        outcome: resource::Outcome::Failed,
        predecessor: None,
        collector_version: "verified-cgroup-fixture-v1".into(),
        trusted_collector: true,
    };
    let mut predecessor = produce(&record, &entry);
    predecessor.record_observation(&observation).unwrap();
    let predecessor = predecessor.finish().unwrap().bundle;
    let mut successor = record.clone();
    successor.public_id = "b".repeat(64);
    let next = successor.binding.as_mut().unwrap();
    next.attempt_id = RemoteAttemptId::new("retry-attempt").unwrap();
    next.fence_generation = next.fence_generation.advance().unwrap();
    let mut next_entry = entry.clone();
    next_entry.reservation.attempt = next.attempt_id.clone();
    next_entry.reservation.fence = next.fence_generation;
    let next = next.clone();
    let context = resource::RetryContext {
        job: &binding.job_id,
        attempt: &binding.attempt_id,
        fence: binding.fence_generation,
        next_attempt: &next.attempt_id,
        next_fence: next.fence_generation,
        failed: true,
        retries: 0,
        cumulative_units: 1,
        elapsed_ms: 10,
        next_wall_budget_ms: 100,
    };
    let controls = resource::Controls {
        mode: resource::SelectionMode::ProjectOptIn,
        project_opted_in: true,
        selection_enabled: true,
        oom_retry_enabled: true,
        quota_enforcement_enabled: true,
        sharing_enabled: false,
    };
    let oom = resource::OomEvidence {
        job: &binding.job_id,
        attempt: &binding.attempt_id,
        fence: binding.fence_generation,
        platform: &prior.features.platform,
        category: resource::OomCategory::LinuxCgroupV2Kill,
        trusted_positive_platform_evidence: true,
    };
    let quota = resource::QuotaFacts {
        project_remaining_units: 100,
        account_remaining_units: 100,
    };
    let policy = resource::EscalationPolicy {
        version: 1,
        max_retries: 1,
        largest_class_order: 2,
        max_cumulative_units: 100,
        max_wall_ms: 1000,
    };
    let plan = resource::plan_escalation(resource::EscalationRequest {
        controls,
        context: context.clone(),
        evidence: oom.clone(),
        prior,
        classes: &classes,
        quota,
        policy,
    })
    .unwrap();
    let mut evidence = GatewayEvidence::new(&successor, [1; 32], &next_entry).unwrap();
    let mut ambiguous = oom.clone();
    ambiguous.trusted_positive_platform_evidence = false;
    assert!(
        evidence
            .record_retry(RetryFacts {
                predecessor_bundle: &predecessor,
                controls,
                context: context.clone(),
                evidence: ambiguous,
                prior,
                classes: &classes,
                quota,
                policy,
                plan: &plan
            })
            .is_err()
    );
    evidence
        .record_retry(RetryFacts {
            predecessor_bundle: &predecessor,
            controls,
            context,
            evidence: oom,
            prior,
            classes: &classes,
            quota,
            policy,
            plan: &plan,
        })
        .unwrap();
    evidence.record_durable_facts().unwrap();
    let bundle = evidence.finish().unwrap().bundle;
    assert_eq!(bundle.retry_edges[0].successor_attempt_identity, v::attempt_identity(&bundle.attempts[0]));
    assert_eq!(
        bundle.attempts[0].parent_attempt_identity,
        Some(bundle.retry_edges[0].predecessor_attempt_identity.clone())
    );
    assert!(v::validate_build_service_bundle(&bundle).valid);
}

#[test]
fn consumer_alias_has_no_new_execution_or_charge_and_requires_read_authority() {
    use crate::remote_build::gateway::GatewayReuseRecord;
    let (mut producer, entry) = fixture();
    producer.result_evidence = Some([2; 32]);
    producer.outputs = vec!["/nix/store/00000000000000000000000000000000-output".into()];
    retained_output(&mut producer);
    let mut evidence = produce(&producer, &entry);
    evidence
        .record_result(
            vec![v::BuildOutputRef {
                path: producer.outputs[0].clone(),
                content_hash: "e".repeat(64),
            }],
            "x86_64-linux",
            "policy",
            "signature",
            "producer-cas",
        )
        .unwrap();
    let exported = evidence.finish().unwrap();
    let mut alias = producer.clone();
    alias.public_id = "b".repeat(64);
    alias.project = "consumer".into();
    alias.reuse = Some(GatewayReuseRecord {
        producer_public_id: producer.public_id.clone(),
        producer_project: producer.project.clone(),
        producer_account: producer.account.clone(),
        producer_request_id: producer.request_id.clone(),
        producer_evidence: format!("{}.json", exported.digest_blake3),
        consumer_project: alias.project.clone(),
        sharing_scope: Some("explicit-scope".into()),
        result_evidence: [2; 32],
    });
    assert!(GatewayEvidence::new(&alias, [1; 32], &entry).is_err());
    assert!(GatewayEvidence::new_reuse(&alias, [1; 32], &exported.bundle).unwrap().finish().is_err());
    let mut evidence = GatewayEvidence::new_reuse(&alias, [1; 32], &exported.bundle).unwrap();
    assert!(evidence.record_durable_facts().is_err());
    let authority = evidence
        .record_authority(PublicAuthority {
            subject: &alias.subject,
            account: &alias.account,
            project: &alias.project,
            policy_identity: &alias.authority_policy,
            operation: "result-read",
            decision_id: &"d".repeat(64),
        })
        .unwrap();
    evidence.record_alias_reuse(&authority, "current-output-check", "current-cas").unwrap();
    let bundle = evidence.finish().unwrap().bundle;
    assert_eq!(bundle.usage_records, exported.bundle.usage_records);
    assert_eq!(bundle.attempts, exported.bundle.attempts);
    assert_eq!(bundle.build_results, exported.bundle.build_results);
    assert_eq!(bundle.reuse_edges[0].authorization_decision_identity, Some(authority));
    let mut altered = exported.bundle;
    altered.build_results[0].policy = "changed".into();
    assert!(GatewayEvidence::new_reuse(&alias, [1; 32], &altered).is_err());
}

#[test]
fn sampled_scratch_and_process_rss_never_become_complete_resource_dimensions() {
    let (record, entry) = fixture();
    let class = &selection_fixture().classes[0];
    let observation = resource::Observation {
        schema: resource::OBSERVATION_SCHEMA.into(),
        attempt_id: record.binding.as_ref().unwrap().attempt_id.clone(),
        action_family: "family".into(),
        features: class.features.clone(),
        policy_version: 1,
        machine_class: class.id.clone(),
        declared: class.quantities,
        selected: class.quantities,
        measured: resource::Measurements {
            cpu_units: None,
            memory_bytes: None,
            scratch_bytes: None,
        },
        scratch_sampled_peak_bytes: Some(4096),
        queue_ms: None,
        execution_ms: Some(10),
        terminal_unix_s: 1_760_000_002,
        cpu_ms: Some(0),
        io_bytes: None,
        transfer_bytes: None,
        executor_output_bytes: None,
        process_peak_rss_bytes: Some(2048),
        oom: resource::OomCategory::Unknown,
        outcome: resource::Outcome::Success,
        predecessor: None,
        collector_version: "process-only-fixture-v1".into(),
        trusted_collector: false,
    };
    let mut evidence = produce(&record, &entry);
    evidence.record_observation(&observation).unwrap();
    let exported = evidence.finish().unwrap();
    let row = &exported.bundle.resource_observations[0];
    assert_eq!(row.role, v::BuildServiceRole::RecordedOnly);
    assert!(row.measurements.iter().any(|m| m.name == "sampled-scratch-lower-bound" && m.value == 4096));
    assert!(row.measurements.iter().any(|m| m.name == "exact-child-peak-rss-not-tree-peak" && m.value == 2048));
    assert!(
        !row.measurements
            .iter()
            .any(|m| matches!(m.name.as_str(), "peak-memory" | "scratch-high-water" | "transfer"))
    );
    let scope: serde_json::Value = serde_json::from_str(&row.compatibility_scope).unwrap();
    assert_eq!(
        scope["known_resource_dimensions"],
        serde_json::json!({"cpu_units":null,"memory_bytes":null,"scratch_bytes":null})
    );
    for name in [
        "cpu_units",
        "peak_memory",
        "scratch_high_water",
        "transfer",
        "executor_output",
    ] {
        assert_eq!(scope["unknown_measurements"][name], true);
    }
    for name in ["cpu", "execution", "sampled_scratch_lower_bound", "process_peak_rss"] {
        assert_eq!(scope["unknown_measurements"][name], false);
    }
    assert_eq!(scope["trusted_collector"], false);
    assert_eq!(scope["oom"], "Unknown");
}
