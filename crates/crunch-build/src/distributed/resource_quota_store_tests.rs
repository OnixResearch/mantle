use crunch_remote_core::attempt::RemoteAttemptId;
use crunch_remote_core::attempt::RemoteFenceGeneration;
use crunch_remote_core::attempt::RemoteJobId;
use crunch_remote_core::resource_quota::PendingReason;
use crunch_remote_core::resource_quota::Reconciliation;
use crunch_remote_core::resource_quota::UnitSchedule;
use crunch_remote_core::resource_quota::Usage;

use super::*;
/// The quota store admits a private directory only, so the fixture creates one
/// with owner-only permissions instead of the default temporary mode.
fn private_tempdir() -> tempfile::TempDir {
    let directory = tempfile::tempdir().unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    directory
}
fn reservation(name: &str, project: &str) -> Reservation {
    Reservation {
        job: RemoteJobId::new("job").unwrap(),
        attempt: RemoteAttemptId::new(name).unwrap(),
        fence: RemoteFenceGeneration::INITIAL,
        project: project.into(),
        account: "account".into(),
        request_digest: "a".repeat(64),
        units: 10,
        schedule: UnitSchedule {
            version: 1,
            unit: "public-compute-unit".into(),
            window_id: "window-1".into(),
            window_start_s: 1,
            window_end_s: 100,
            cpu_ms_per_unit: 10,
            memory_byte_ms_per_unit: 100,
            executor_output_bytes_per_unit: 10,
        },
    }
}
fn limits() -> Limits {
    Limits {
        project_units: 10,
        account_units: 10,
    }
}
fn completion() -> UsageEvent {
    UsageEvent::Complete {
        event_digest: "b".repeat(64),
        usage: Usage {
            cpu_ms: 11,
            memory_byte_ms: 1,
            executor_output_bytes: 1,
        },
        trusted_and_complete: true,
    }
}
#[test]
fn durable_duplicate_reservation_and_completion_charge_once_across_reopen() {
    let directory = private_tempdir();
    let store = ResourceQuotaStore::provision(directory.path()).unwrap();
    let r = reservation("one", "project");
    let reserved = store.reserve(r.clone(), limits()).unwrap();
    assert_eq!(store.reserve(r.clone(), limits()).unwrap(), reserved);
    let settled = store.reconcile(&r, completion()).unwrap();
    assert_eq!(settled.reconciliation, Reconciliation::Settled {
        event_digest: "b".repeat(64),
        charged_units: 4
    });
    let reopened = ResourceQuotaStore::open(directory.path()).unwrap();
    assert_eq!(reopened.reconcile(&r, completion()).unwrap(), settled);
    let ledger = reopened.snapshot().unwrap();
    assert_eq!(ledger.revision, 2);
    let summary = ledger.summary(&r).unwrap();
    assert_eq!(summary.project_charged, 4);
    assert_eq!(summary.account_charged, 4);
    assert_eq!(summary.project_reserved, 0);
    let mut changed = r.clone();
    changed.account = "different".into();
    assert_eq!(reopened.reserve(changed, limits()), Err(QuotaError::Conflict));
}
#[test]
fn project_and_shared_account_limits_both_block_admission() {
    let mut ledger = Ledger::default();
    ledger.reserve(reservation("one", "project"), limits()).unwrap();
    assert_eq!(
        ledger.reserve(reservation("two", "project"), Limits {
            project_units: 10,
            account_units: 100
        }),
        Err(QuotaError::Denied)
    );
    assert_eq!(
        ledger.reserve(reservation("two", "other-project"), Limits {
            project_units: 100,
            account_units: 10
        }),
        Err(QuotaError::Denied)
    );
    assert_eq!(ledger.entries.len(), 1);
}
#[test]
fn worker_loss_partial_and_cancellation_hold_quota_until_final_observation() {
    for (event, expected) in [
        (UsageEvent::WorkerLost, PendingReason::WorkerLost),
        (UsageEvent::Partial, PendingReason::PartialObservation),
        (UsageEvent::Cancelled, PendingReason::CancelledWithoutFinalUsage),
    ] {
        let mut ledger = Ledger::default();
        let r = reservation("one", "project");
        ledger.reserve(r.clone(), limits()).unwrap();
        let pending = ledger.reconcile(&r, event).unwrap();
        assert_eq!(pending.reconciliation, Reconciliation::Pending(expected));
        assert_eq!(ledger.summary(&r).unwrap().account_reserved, 10);
        assert_eq!(ledger.reserve(reservation("two", "other-project"), limits()), Err(QuotaError::Denied));
        let settled = ledger.reconcile(&r, completion()).unwrap();
        assert!(matches!(settled.reconciliation, Reconciliation::Settled { charged_units: 4, .. }));
        assert_eq!(ledger.summary(&r).unwrap().account_reserved, 0);
    }
}
#[test]
fn only_verified_never_started_cancellation_can_release_zero() {
    let mut ledger = Ledger::default();
    let r = reservation("one", "project");
    ledger.reserve(r.clone(), limits()).unwrap();
    let event = |verified_never_started| UsageEvent::CancelledBeforeExecution {
        event_digest: "c".repeat(64),
        verified_never_started,
    };
    assert!(matches!(ledger.reconcile(&r, event(false)).unwrap().reconciliation, Reconciliation::Pending(_)));
    assert_eq!(ledger.summary(&r).unwrap().account_reserved, 10);
    assert!(matches!(ledger.reconcile(&r, event(true)).unwrap().reconciliation, Reconciliation::Settled {
        charged_units: 0,
        ..
    }));
    assert_eq!(ledger.summary(&r).unwrap().account_reserved, 0);
    assert_eq!(ledger.reconcile(&r, completion()), Err(QuotaError::Conflict));
}
#[test]
fn interrupted_mutations_recover_idempotently_and_do_not_grant_quota() {
    let directory = private_tempdir();
    let store = ResourceQuotaStore::provision(directory.path()).unwrap();
    let r = reservation("one", "project");
    let report = run_quota_fault_fixture(directory.path(), &[
        QuotaFixtureStep::Reserve {
            reservation: r.clone(),
            limits: limits(),
            fault: QuotaStoreFault::BeforePublish,
        },
        QuotaFixtureStep::Reserve {
            reservation: r.clone(),
            limits: limits(),
            fault: QuotaStoreFault::AfterPublish,
        },
        QuotaFixtureStep::Reopen,
        QuotaFixtureStep::Reserve {
            reservation: r.clone(),
            limits: limits(),
            fault: QuotaStoreFault::None,
        },
        QuotaFixtureStep::Reconcile {
            reservation: r.clone(),
            event: completion(),
            fault: QuotaStoreFault::AfterPublish,
        },
        QuotaFixtureStep::Reopen,
        QuotaFixtureStep::Reconcile {
            reservation: r.clone(),
            event: completion(),
            fault: QuotaStoreFault::None,
        },
    ])
    .unwrap();
    assert_eq!(report.results[0], Err(QuotaError::StoreUnavailable));
    assert_eq!(report.results[1], Err(QuotaError::StoreUncertain));
    assert_eq!(report.results[4], Err(QuotaError::StoreUncertain));
    assert_eq!(report.ledger.revision, 2);
    assert_eq!(report.ledger.summary(&r).unwrap().account_charged, 4);
    assert_eq!(report.evidence_class, "local-deterministic-fixture-only");
    assert!(report.non_claims.contains(&"no simulated service acceptance"));
    assert_eq!(store.reserve(reservation("two", "project"), limits()), Err(QuotaError::Denied));
}
#[test]
fn corrupt_missing_overflow_and_oversized_usage_fail_closed() {
    let directory = private_tempdir();
    assert!(matches!(ResourceQuotaStore::open(directory.path()), Err(QuotaError::StoreUnavailable)));
    let store = ResourceQuotaStore::provision(directory.path()).unwrap();
    let r = reservation("one", "project");
    store.reserve(r.clone(), limits()).unwrap();
    let oversized = UsageEvent::Complete {
        event_digest: "d".repeat(64),
        usage: Usage {
            cpu_ms: 1000,
            memory_byte_ms: 0,
            executor_output_bytes: 0,
        },
        trusted_and_complete: true,
    };
    assert_eq!(
        store.reconcile(&r, oversized).unwrap().reconciliation,
        Reconciliation::Pending(PendingReason::UsageExceedsReservation)
    );
    assert_eq!(store.snapshot().unwrap().summary(&r).unwrap().account_reserved, 10);
    let mut ledger = store.snapshot().unwrap();
    ledger.revision = u64::MAX;
    assert_eq!(ledger.reconcile(&r, completion()), Err(QuotaError::Overflow));
    assert_eq!(ledger.summary(&r).unwrap().account_reserved, 10);
    fs::write(store.state_path(), b"not-json").unwrap();
    assert_eq!(store.reserve(reservation("two", "project"), limits()), Err(QuotaError::StoreUnavailable));
    assert!(matches!(ResourceQuotaStore::provision(directory.path()), Err(QuotaError::StoreUnavailable)));
    fs::remove_file(store.state_path()).unwrap();
    assert!(matches!(ResourceQuotaStore::provision(directory.path()), Err(QuotaError::StoreUnavailable)));
}

#[cfg(unix)]
#[test]
fn quota_rejects_symlinks_public_directories_and_duplicate_ledger_keys() {
    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::fs::symlink;
    let directory = private_tempdir();
    let store = ResourceQuotaStore::provision(directory.path()).unwrap();
    let original = fs::read(store.state_path()).unwrap();
    let duplicate =
        String::from_utf8(original.clone())
            .unwrap()
            .replacen("\"revision\":0", "\"revision\":0,\"revision\":0", 1);
    fs::write(store.state_path(), duplicate).unwrap();
    assert!(matches!(store.snapshot(), Err(QuotaError::StoreUnavailable)));
    fs::write(store.state_path(), &original).unwrap();
    let moved = directory.path().join("retained-original.json");
    fs::rename(store.state_path(), &moved).unwrap();
    symlink(&moved, store.state_path()).unwrap();
    assert!(matches!(store.snapshot(), Err(QuotaError::StoreUnavailable)));
    fs::remove_file(store.state_path()).unwrap();
    fs::rename(&moved, store.state_path()).unwrap();
    let lock_path = directory.path().join("resource-quota.lock");
    fs::remove_file(&lock_path).unwrap();
    symlink(store.state_path(), &lock_path).unwrap();
    assert!(matches!(store.snapshot(), Err(QuotaError::StoreUnavailable)));
    fs::remove_file(lock_path).unwrap();
    fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o755)).unwrap();
    assert!(matches!(store.snapshot(), Err(QuotaError::StoreUnavailable)));
    fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o700)).unwrap();
    assert_eq!(store.snapshot().unwrap(), Ledger::default());
}

#[cfg(unix)]
#[test]
fn quota_busy_lock_returns_without_waiting_for_owner_release() {
    let directory = private_tempdir();
    let store = ResourceQuotaStore::provision(directory.path()).unwrap();
    let held = OpenOptions::new()
        .read(true)
        .write(true)
        .open(directory.path().join("resource-quota.lock"))
        .unwrap();
    held.lock_exclusive().unwrap();
    // The owner remains locked throughout the call: an unbounded waiter hangs.
    assert!(matches!(store.snapshot(), Err(QuotaError::StoreUnavailable)));
    fs2::FileExt::unlock(&held).unwrap();
    assert_eq!(store.snapshot().unwrap(), Ledger::default());
}

#[test]
fn admitted_context_and_observation_survive_reopen_without_cross_attempt_authority() {
    use crunch_remote_core::resource_policy as policy;
    use crunch_remote_core::resource_quota::ObservationRecord;
    use crunch_remote_core::resource_quota::ResourceContext;
    let directory = private_tempdir();
    let store = ResourceQuotaStore::provision(directory.path()).unwrap();
    let r = reservation("observed", "project");
    let quantities = policy::Quantities {
        cpu_units: 1,
        memory_bytes: 200,
        scratch_bytes: 100,
    };
    let context = ResourceContext {
        declared: quantities,
        selected: quantities,
        features: policy::Features {
            architecture: "x86_64".into(),
            platform: "linux".into(),
            trust_domain: "fixture".into(),
            isolation: "sandbox".into(),
            kvm: false,
        },
        machine_class: "fixture-class".into(),
        action_family: "family".into(),
        policy_version: 1,
        predecessor: None,
    };
    store.reserve_with_resource_context(r.clone(), limits(), context.clone()).unwrap();
    let mut observation = policy::Observation {
        schema: policy::OBSERVATION_SCHEMA.into(),
        attempt_id: r.attempt.clone(),
        action_family: context.action_family.clone(),
        features: context.features.clone(),
        policy_version: 1,
        machine_class: context.machine_class.clone(),
        declared: quantities,
        selected: quantities,
        measured: policy::Measurements {
            cpu_units: None,
            memory_bytes: None,
            scratch_bytes: None,
        },
        scratch_sampled_peak_bytes: None,
        queue_ms: None,
        execution_ms: None,
        terminal_unix_s: 10,
        cpu_ms: None,
        io_bytes: None,
        transfer_bytes: None,
        executor_output_bytes: None,
        process_peak_rss_bytes: None,
        oom: policy::OomCategory::Unknown,
        outcome: policy::Outcome::WorkerLost,
        predecessor: None,
        collector_version: "coordinator-fixture-v2".into(),
        trusted_collector: false,
    };
    store
        .record_observation(&r, ObservationRecord {
            observation: observation.clone(),
            usage_event_digest: "c".repeat(64),
        })
        .unwrap();
    store.reconcile(&r, UsageEvent::WorkerLost).unwrap();
    assert_eq!(store.snapshot().unwrap().summary(&r).unwrap().account_reserved, 10);
    // A delayed authenticated report can refine unknown observations without
    // releasing quota or manufacturing a second attempt's collector authority.
    observation.trusted_collector = true;
    observation.outcome = policy::Outcome::Success;
    observation.collector_version = "authenticated-fixture-v2".into();
    observation.measured.memory_bytes = Some(150);
    observation.execution_ms = Some(10);
    observation.cpu_ms = Some(1);
    let record = ObservationRecord {
        observation: observation.clone(),
        usage_event_digest: "d".repeat(64),
    };
    store.record_observation(&r, record.clone()).unwrap();
    let reopened = ResourceQuotaStore::open(directory.path()).unwrap();
    reopened.record_observation(&r, record.clone()).unwrap();
    assert_eq!(reopened.snapshot().unwrap().observations.get(r.attempt.as_str()), Some(&record));
    let before_conflict = reopened.snapshot().unwrap();
    let mut conflicting_replay = record.clone();
    conflicting_replay.observation.measured.memory_bytes = Some(199);
    assert_eq!(reopened.record_observation(&r, conflicting_replay), Err(QuotaError::Conflict));
    assert_eq!(reopened.snapshot().unwrap(), before_conflict);
    assert_eq!(reopened.snapshot().unwrap().resource_contexts.get(r.attempt.as_str()), Some(&context));
    assert_eq!(reopened.snapshot().unwrap().summary(&r).unwrap().account_reserved, 10);
    let mut wrong = record.clone();
    wrong.observation.features.trust_domain = "other".into();
    assert_eq!(reopened.record_observation(&r, wrong), Err(QuotaError::Conflict));
    let mut other_attempt = r.clone();
    other_attempt.attempt = RemoteAttemptId::new("different").unwrap();
    assert_eq!(reopened.record_observation(&other_attempt, record), Err(QuotaError::UnknownAttempt));
    assert!(matches!(reopened.reconcile(&r, completion()).unwrap().reconciliation, Reconciliation::Settled {
        charged_units: 4,
        ..
    }));
}
