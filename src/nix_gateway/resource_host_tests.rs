use super::*;

fn private_tempdir() -> tempfile::TempDir {
    use std::os::unix::fs::PermissionsExt as _;
    let directory = tempfile::tempdir().unwrap();
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    directory
}

fn features() -> policy::Features {
    policy::Features {
        architecture: "x86_64".into(),
        platform: "linux".into(),
        trust_domain: "fixture".into(),
        isolation: "sandbox".into(),
        kvm: false,
    }
}
fn quantities(memory_bytes: u64) -> policy::Quantities {
    policy::Quantities {
        cpu_units: 1,
        memory_bytes,
        scratch_bytes: 1024,
    }
}
fn requirements(memory_bytes: u64) -> RemoteResourceRequirements {
    RemoteResourceRequirements {
        quantities: crunch_build::distributed::RemoteResourceVector {
            cpu_units: 1,
            memory_bytes,
            scratch_bytes: 1024,
            accelerators: Vec::new(),
            named_tokens: Vec::new(),
        },
        semantic_accelerator_classes: Vec::new(),
    }
}
fn config() -> ResourcePolicyConfig {
    let classes = [("small", 1024, 1_u32), ("large", 2048, 2_u32)]
        .into_iter()
        .map(|(id, memory, order)| ResourceClassConfig {
            id: id.into(),
            features: features(),
            quantities: quantities(memory),
            order,
            reservation_units: u64::from(order) * 10,
            worker_endpoint_id: "worker".into(),
            inventory: RemoteWorkerResourceInventory {
                total: requirements(2048).quantities,
            },
        })
        .collect();
    let projects = ["project", "other"]
        .into_iter()
        .map(|name| {
            (name.into(), ProjectQuotaConfig {
                account: "account".into(),
                project_units: 100,
                account_units: 100,
            })
        })
        .collect();
    ResourcePolicyConfig {
        selection: policy::SelectionPolicy {
            version: 1,
            min_samples: 1,
            max_age_s: 100,
            margin_basis_points: 0,
            max_measurement: policy::Quantities {
                cpu_units: 1024,
                memory_bytes: 1 << 30,
                scratch_bytes: 1 << 20,
            },
            max_duration_ms: 60_000,
            max_io_bytes: 1 << 20,
        },
        retry: RetryPolicy {
            version: 1,
            max_retries: 2,
            largest_class_order: 2,
            max_cumulative_units: 100,
            max_wall_ms: 60_000,
        },
        mode: policy::SelectionMode::ProjectOptIn,
        selection_enabled: true,
        oom_retry_enabled: true,
        quota_enforcement_enabled: true,
        sharing_enabled: true,
        opted_in_projects: ["project".into(), "other".into()].into_iter().collect(),
        projects,
        schedule: quota::UnitSchedule {
            version: 1,
            unit: "fixture-compute-units".into(),
            window_id: "fixture-window".into(),
            window_start_s: 1,
            window_end_s: u64::MAX,
            cpu_ms_per_unit: 1000,
            memory_byte_ms_per_unit: 1 << 30,
            executor_output_bytes_per_unit: 1 << 20,
        },
        classes,
        sharing_scopes: Vec::new(),
    }
}
fn binding() -> AttemptBinding {
    AttemptBinding {
        job: RemoteJobId::new("job").unwrap(),
        attempt: RemoteAttemptId::new("attempt").unwrap(),
        fence: RemoteFenceGeneration::INITIAL,
        project: "project".into(),
        account: "account".into(),
        request_id: "resource-request".into(),
        request_digest: "a".repeat(64),
    }
}
fn select(host: &ResourcePolicyHost, project: &str, memory: u64) -> SelectedResources {
    host.select(SelectionRequest {
        project,
        account: "account",
        request_id: "resource-request",
        request_digest: &"a".repeat(64),
        requirements: &requirements(memory),
        features: &features(),
        public_action_kind: "build",
        now_unix_s: 100,
        live_leases: &[],
    })
    .unwrap()
}

#[test]
fn durable_known_memory_changes_only_its_project_and_observe_only_keeps_static_class() {
    let directory = private_tempdir();
    let store = ResourceQuotaStore::provision(directory.path()).unwrap();
    let host = ResourcePolicyHost::open(config(), directory.path()).unwrap();
    let selected = select(&host, "project", 1500);
    let binding = binding();
    let entry = host.reserve(&selected, &binding, 100).unwrap();
    let context = store.snapshot().unwrap().resource_contexts.get(binding.attempt.as_str()).unwrap().clone();
    // Persisted fixture over the pure collector-facts boundary. It is not a
    // worker/VM campaign and cannot construct RemoteAuthenticatedUsage.
    let observation = policy::Observation {
        schema: policy::OBSERVATION_SCHEMA.into(),
        attempt_id: binding.attempt.clone(),
        action_family: context.action_family.clone(),
        features: context.features.clone(),
        policy_version: context.policy_version,
        machine_class: context.machine_class.clone(),
        declared: context.declared,
        selected: context.selected,
        measured: policy::Measurements {
            cpu_units: None,
            memory_bytes: Some(1500),
            scratch_bytes: None,
        },
        scratch_sampled_peak_bytes: Some(1),
        queue_ms: None,
        execution_ms: Some(10),
        terminal_unix_s: 100,
        cpu_ms: Some(1),
        io_bytes: None,
        transfer_bytes: None,
        executor_output_bytes: Some(0),
        process_peak_rss_bytes: None,
        oom: policy::OomCategory::None,
        outcome: policy::Outcome::Success,
        predecessor: None,
        collector_version: "authenticated-fixture-v2".into(),
        trusted_collector: true,
    };
    store
        .record_observation(&entry.reservation, quota::ObservationRecord {
            observation,
            usage_event_digest: "b".repeat(64),
        })
        .unwrap();
    let reopened = ResourcePolicyHost::open(config(), directory.path()).unwrap();
    assert_eq!(select(&reopened, "project", 1024).machine_class().id, "large");
    assert_eq!(select(&reopened, "other", 1024).machine_class().id, "small");
    let mut observing = config();
    observing.mode = policy::SelectionMode::ObserveOnly;
    let observing = ResourcePolicyHost::open(observing, directory.path()).unwrap();
    let report = select(&observing, "project", 1024);
    assert_eq!(report.machine_class().id, "small");
    let historical = report.evaluation().decision.historical_decision.as_ref().unwrap();
    assert_eq!(report.evaluation().classes[historical.class_index].id, "large");
    assert_eq!(historical.requested.scratch_bytes, 1024);
}

#[test]
fn stale_fence_and_changed_scope_cannot_mutate_reserved_accounting() {
    let directory = private_tempdir();
    let store = ResourceQuotaStore::provision(directory.path()).unwrap();
    let host = ResourcePolicyHost::open(config(), directory.path()).unwrap();
    let binding = binding();
    host.reserve(&select(&host, "project", 1024), &binding, 100).unwrap();
    let before = store.snapshot().unwrap();
    for changed in [
        AttemptBinding {
            fence: binding.fence.advance().unwrap(),
            ..binding.clone()
        },
        AttemptBinding {
            project: "other".into(),
            ..binding.clone()
        },
        AttemptBinding {
            account: "other-account".into(),
            ..binding.clone()
        },
        AttemptBinding {
            request_digest: "b".repeat(64),
            ..binding.clone()
        },
    ] {
        assert!(matches!(
            host.mark_pending(&changed, PendingUsage::Cancelled, 101),
            Err(ResourceHostError::BindingMismatch)
        ));
        assert_eq!(store.snapshot().unwrap(), before);
    }
    assert_eq!(host.summary("project", "account").unwrap().totals.project_reserved, 10);
}

#[test]
fn lost_worker_keeps_quota_and_records_unknown_not_zero_measurements() {
    let directory = private_tempdir();
    ResourceQuotaStore::provision(directory.path()).unwrap();
    let host = ResourcePolicyHost::open(config(), directory.path()).unwrap();
    let binding = binding();
    let selected = select(&host, "project", 1024);
    host.reserve(&selected, &binding, 100).unwrap();
    let pending = host.mark_pending(&binding, PendingUsage::WorkerLost, 101).unwrap();
    assert_eq!(pending.reconciliation, quota::Reconciliation::Pending(quota::PendingReason::WorkerLost));
    let observation = host.observation(&binding).unwrap().unwrap();
    assert_eq!(observation.outcome, policy::Outcome::WorkerLost);
    assert_eq!(observation.terminal_unix_s, 101);
    assert_eq!(observation.execution_ms, None);
    assert_eq!(observation.cpu_ms, None);
    assert_eq!(observation.measured.memory_bytes, None);
    assert!(!observation.trusted_collector);
    assert_eq!(host.summary("project", "account").unwrap().totals.project_reserved, 10);
    let decision = select(&host, "project", 1024);
    assert_eq!(decision.machine_class().id, "small");
    assert_eq!(decision.evaluation().decision.historical_decision.as_ref().unwrap().exclusions, vec![(
        0,
        policy::Reason::HistoryUntrusted
    )]);
}

#[cfg(target_os = "linux")]
#[test]
fn actual_process_only_collector_neither_releases_hold_nor_promises_tree_memory() {
    let directory = private_tempdir();
    ResourceQuotaStore::provision(directory.path()).unwrap();
    let host = ResourcePolicyHost::open(config(), directory.path()).unwrap();
    let binding = binding();
    host.reserve(&select(&host, "project", 1024), &binding, 100).unwrap();
    let production = crate::remote_build::RemoteProductionAttemptBinding {
        job_id: binding.job.clone(),
        attempt_id: binding.attempt.clone(),
        fence_generation: binding.fence,
    };
    let mut command = std::process::Command::new(std::env::current_exe().unwrap());
    command.args(["--list", "--exact", "resource-child-does-not-match-any-test"]);
    let completed = crate::remote_build::usage::RemoteUsageCollector::begin(
        &production,
        &binding.request_id,
        std::time::Duration::from_secs(10),
    )
    .unwrap()
    .spawn(&mut command)
    .unwrap()
    .finish()
    .unwrap();
    assert!(completed.status().success());
    let usage = completed.bind_result(&"c".repeat(64)).unwrap();
    let result = host.reconcile(&binding, &usage, 102).unwrap();
    assert_eq!(result.reconciliation, quota::Reconciliation::Pending(quota::PendingReason::PartialObservation));
    let observation = host.observation(&binding).unwrap().unwrap();
    assert_eq!(observation.terminal_unix_s, 102);
    assert_eq!(observation.measured.memory_bytes, None);
    assert_eq!(observation.measured.cpu_units, None);
    assert_eq!(observation.measured.scratch_bytes, None);
    assert_eq!(observation.oom, policy::OomCategory::Unknown);
    assert_eq!(host.summary("project", "account").unwrap().totals.project_reserved, 10);
}

#[test]
fn public_summary_preserves_pending_reasons_scope_window_and_idempotent_charge() {
    let directory = private_tempdir();
    let store = ResourceQuotaStore::provision(directory.path()).unwrap();
    let host = ResourcePolicyHost::open(config(), directory.path()).unwrap();
    let selected = select(&host, "project", 1024);
    let first = host.reserve(&selected, &binding(), 100).unwrap();
    let events = [
        quota::UsageEvent::Cancelled,
        quota::UsageEvent::WorkerLost,
        quota::UsageEvent::Partial,
        quota::UsageEvent::Complete {
            event_digest: "b".repeat(64),
            usage: quota::Usage {
                cpu_ms: 1,
                memory_byte_ms: 0,
                executor_output_bytes: 0,
            },
            trusted_and_complete: false,
        },
        quota::UsageEvent::Complete {
            event_digest: "c".repeat(64),
            usage: quota::Usage {
                cpu_ms: 11_000,
                memory_byte_ms: 0,
                executor_output_bytes: 0,
            },
            trusted_and_complete: true,
        },
    ];
    for (index, event) in events.into_iter().enumerate() {
        let mut reservation = first.reservation.clone();
        reservation.attempt = RemoteAttemptId::new(format!("pending-{index}")).unwrap();
        store
            .reserve(reservation.clone(), quota::Limits {
                project_units: 100,
                account_units: 100,
            })
            .unwrap();
        store.reconcile(&reservation, event).unwrap();
    }
    let charge = quota::UsageEvent::Complete {
        event_digest: "d".repeat(64),
        usage: quota::Usage {
            cpu_ms: 1001,
            memory_byte_ms: 1,
            executor_output_bytes: 1,
        },
        trusted_and_complete: true,
    };
    store.reconcile(&first.reservation, charge.clone()).unwrap();
    store.reconcile(&first.reservation, charge).unwrap();
    let mut other = first.reservation.clone();
    other.attempt = RemoteAttemptId::new("other-project").unwrap();
    other.project = "other".into();
    store
        .reserve(other.clone(), quota::Limits {
            project_units: 100,
            account_units: 100,
        })
        .unwrap();
    let mut old = other;
    old.attempt = RemoteAttemptId::new("old-window").unwrap();
    old.schedule.window_id = "old-window".into();
    store
        .reserve(old, quota::Limits {
            project_units: 100,
            account_units: 100,
        })
        .unwrap();
    let public = host.summary("project", "account").unwrap().into_accounting();
    assert_eq!((public.project_reserved_units, public.project_charged_units), (50, 4));
    assert_eq!((public.account_reserved_units, public.account_charged_units), (60, 4));
    assert_eq!(public.project_reason_counts, UsageReasonCounts {
        settled: 1,
        cancelled_without_final_usage: 1,
        worker_lost: 1,
        partial_observation: 1,
        untrusted_observation: 1,
        usage_exceeds_reservation: 1,
        ..UsageReasonCounts::default()
    });
    assert_eq!(public.account_reason_counts.reserved, 1);
    assert!(matches!(host.summary("project", "wrong-account"), Err(ResourceHostError::BindingMismatch)));
}

#[cfg(target_os = "linux")]
fn delegated_usage(root: &Path, binding: &AttemptBinding, memory_bytes: u64, kill: bool) -> RemoteAuthenticatedUsage {
    let production = crate::remote_build::RemoteProductionAttemptBinding {
        job_id: binding.job.clone(),
        attempt_id: binding.attempt.clone(),
        fence_generation: binding.fence,
    };
    let mut command = std::process::Command::new("python3");
    command.args([
        "-c",
        if kill {
            "import os,signal;os.kill(os.getpid(),signal.SIGKILL)"
        } else {
            "a=bytearray(96*1024*1024);print(len(a))"
        },
    ]);
    let completed = crate::remote_build::usage::RemoteUsageCollector::begin(
        &production,
        &binding.request_id,
        std::time::Duration::from_secs(10),
    )
    .unwrap()
    .with_cgroup_root(root, Some(memory_bytes))
    .spawn(&mut command)
    .unwrap()
    .finish()
    .unwrap();
    let digest = if completed.status().success() {
        "c".repeat(64)
    } else {
        crate::remote_build::usage::failure_result_digest(&production, &binding.request_id)
    };
    let usage = completed.bind_result(&digest).unwrap();
    let raw = ed25519_dalek::SigningKey::from_bytes(&[31_u8; 32]);
    let keypair = crunch_build::KeyPair {
        verifying_key: nix_compat::narinfo::VerifyingKey::new("oom-fixture".into(), raw.verifying_key()),
        signing_key: nix_compat::narinfo::SigningKey::new("oom-fixture".into(), raw),
    };
    let signed = usage.sign(&keypair).unwrap();
    assert!(signed.verify(&production, &binding.request_id, &"f".repeat(64), &keypair.verifying_key).is_err());
    signed.verify(&production, &binding.request_id, &digest, &keypair.verifying_key).unwrap()
}

/// Actual kernel and worker-signature path, never a supplied OOM flag. This
/// exercises the resource host, not gateway authority or remote dispatch.
#[cfg(target_os = "linux")]
#[test]
#[ignore = "requires python3 and MANTLE_TEST_CGROUP_ROOT: empty delegated cgroup-v2 cpu+memory subtree"]
fn delegated_oom_signed_successor_and_sigkill_negative() {
    let root = std::env::var_os("MANTLE_TEST_CGROUP_ROOT").expect("delegated cgroup root");
    let directory = private_tempdir();
    ResourceQuotaStore::provision(directory.path()).unwrap();
    let mut config = config();
    for (class, memory) in config.classes.iter_mut().zip([64 << 20, 256 << 20]) {
        class.quantities.memory_bytes = memory;
        class.inventory.total.memory_bytes = 256 << 20;
    }
    config.schedule.memory_byte_ms_per_unit = 1 << 50;
    let host = ResourcePolicyHost::open(config, directory.path()).unwrap();
    let binding = binding();
    let prior = select(&host, "project", 64 << 20);
    host.reserve(&prior, &binding, 100).unwrap();
    let usage = delegated_usage(Path::new(&root), &binding, 64 << 20, false);
    assert!(matches!(usage.oom_evidence(), RemoteUsageOomEvidence::DedicatedCgroupOom { oom_kill_delta: 1.. }));
    assert_eq!(usage.measurement_completeness(), RemoteUsageCompleteness::DedicatedCgroup);
    host.reconcile(&binding, &usage, 101).unwrap();
    let next = RemoteAttemptId::new("successor").unwrap();
    let next_fence = binding.fence.advance().unwrap();
    let context = policy::RetryContext {
        job: &binding.job,
        attempt: &binding.attempt,
        fence: binding.fence,
        next_attempt: &next,
        next_fence,
        failed: true,
        retries: 0,
        cumulative_units: 10,
        elapsed_ms: 1000,
        next_wall_budget_ms: 10_000,
    };
    let retry = host.plan_oom_retry(&prior, &binding, &usage, context.clone(), 101, &[]).unwrap();
    assert_eq!(retry.plan.predecessor, binding.attempt);
    assert_eq!(retry.plan.next_fence, next_fence);
    let successor = AttemptBinding {
        attempt: next.clone(),
        fence: next_fence,
        ..binding.clone()
    };
    host.reserve(&retry.selected, &successor, 101).unwrap();
    let completed = delegated_usage(Path::new(&root), &successor, 256 << 20, false);
    assert_eq!(completed.oom_evidence(), RemoteUsageOomEvidence::DedicatedCgroupNoOom);
    assert!(matches!(
        host.reconcile(&successor, &completed, 102).unwrap().reconciliation,
        quota::Reconciliation::Settled { .. }
    ));
    let killed = delegated_usage(Path::new(&root), &binding, 64 << 20, true);
    assert_eq!(killed.oom_evidence(), RemoteUsageOomEvidence::DedicatedCgroupNoOom);
    assert!(matches!(
        host.plan_oom_retry(&prior, &binding, &killed, context, 102, &[]),
        Err(ResourceHostError::Policy(policy::Reason::NotPositiveOom))
    ));
}

#[test]
fn retained_producer_signature_requires_full_key_material_not_a_matching_name() {
    use nix_compat::narinfo::SigningKey;
    use nix_compat::narinfo::VerifyingKey;
    let raw = ed25519_dalek::SigningKey::from_bytes(&[17_u8; 32]);
    let other = ed25519_dalek::SigningKey::from_bytes(&[18_u8; 32]);
    let signing = SigningKey::new("same-public-name".into(), raw.clone());
    let trusted = VerifyingKey::new("same-public-name".into(), raw.verifying_key()).to_string();
    let wrong = VerifyingKey::new("same-public-name".into(), other.verifying_key()).to_string();
    let mut info = snix_store::path_info::PathInfo {
        store_path: nix_compat::store_path::StorePath::from_name_and_digest_fixed("signed-fixture", [7_u8; 20])
            .unwrap(),
        node: snix_castore::Node::Symlink {
            target: snix_castore::SymlinkTarget::try_from("fixture-target").unwrap(),
        },
        references: Vec::new(),
        nar_size: 128,
        nar_sha256: [9_u8; 32],
        signatures: Vec::new(),
        deriver: None,
        ca: None,
    };
    crunch_build::sign_pathinfo_with_store_dir(&mut info, &signing, "/nix/store");
    let artifact = crunch_store::artifact_attestation_digest_for_pathinfo("/nix/store", &info, "out", None)
        .unwrap()
        .to_hex();
    let output = crate::remote_build::RemoteProducedOutput {
        name: "out".into(),
        logical_path: info.store_path.to_absolute_path(),
        content_digest_blake3: blake3::hash(&serde_json::to_vec(&info.node).unwrap()).to_hex().to_string(),
        size_bytes: info.nar_size,
        path_info_signing_key_id: trusted.clone(),
        artifact_attestation_digest_blake3: artifact,
        path_info: Some(info),
        nar_payload_digest_blake3: None,
        nar_payload_size_bytes: None,
    };
    let mut receipt = crunch_remote_core::receipt::OrderedOutputReceipt::new();
    receipt.append(crunch_remote_core::receipt::OutputReceiptFields {
        name: &output.name,
        logical_path: &output.logical_path,
        content_digest_blake3: &output.content_digest_blake3,
        artifact_attestation_digest_blake3: &output.artifact_attestation_digest_blake3,
        size_bytes: output.size_bytes,
        nar_payload_digest_blake3: None,
        nar_payload_size_bytes: None,
    });
    let digest = blake3::Hash::from_bytes(receipt.finish()).to_hex().to_string();
    let mut outputs = [output];
    assert_eq!(
        crate::remote_build::validate_retained_remote_produced_outputs(
            &outputs,
            &digest,
            "/nix/store",
            std::slice::from_ref(&trusted),
        ),
        Ok(())
    );
    assert_eq!(
        crate::remote_build::validate_retained_remote_produced_outputs(&outputs, &digest, "/nix/store", &[wrong])
            .unwrap_err(),
        "remote-retained-output-signature-untrusted"
    );
    outputs[0].path_info_signing_key_id = "same-public-name".into();
    assert_eq!(
        crate::remote_build::validate_retained_remote_produced_outputs(&outputs, &digest, "/nix/store", &[trusted])
            .unwrap_err(),
        "remote-retained-output-signature-untrusted"
    );
    // Metadata acceptance is not CAS availability: the concrete sharing path
    // additionally invokes the current signed action and exact measured CAS gate.
}
