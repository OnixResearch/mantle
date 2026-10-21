//! Deterministic local fault-fixture command. This drives the real quota
//! filesystem adapter, not a hosted service, VM, or snapshot replay claim.
use std::os::unix::fs::DirBuilderExt;
use std::path::Path;

use crunch_build::distributed::QuotaFixtureStep as Step;
use crunch_build::distributed::QuotaStoreFault as Fault;
use crunch_build::distributed::ResourceQuotaStore;
use crunch_build::distributed::run_quota_fault_fixture;
use crunch_remote_core::attempt::RemoteAttemptId;
use crunch_remote_core::attempt::RemoteFenceGeneration;
use crunch_remote_core::attempt::RemoteJobId;
use crunch_remote_core::resource_quota::Limits;
use crunch_remote_core::resource_quota::PendingReason;
use crunch_remote_core::resource_quota::QuotaError;
use crunch_remote_core::resource_quota::Reconciliation;
use crunch_remote_core::resource_quota::Reservation;
use crunch_remote_core::resource_quota::UnitSchedule;
use crunch_remote_core::resource_quota::Usage;
use crunch_remote_core::resource_quota::UsageEvent;
use mantle_application_contract::gateway_operations::GatewayOperationEvidence;

use crate::errors::RunError;

pub(crate) fn run_quota_campaign(
    directory: &Path,
    seed: u64,
) -> Result<(serde_json::Value, GatewayOperationEvidence), RunError> {
    let denied = || RunError::Internal("gateway-fault-fixture-failed".into());
    // Existing directories are never reused: this fixture must not mutate an
    // operator's live ledger or silently overwrite previous campaign evidence.
    std::fs::DirBuilder::new().mode(0o700).create(directory).map_err(|_| denied())?;
    ResourceQuotaStore::provision(directory).map_err(|_| denied())?;
    let identity = blake3::hash(&seed.to_le_bytes()).to_hex().to_string();
    let reservation = Reservation {
        job: RemoteJobId::new(format!("fixture-job-{identity}")).map_err(|_| denied())?,
        attempt: RemoteAttemptId::new(format!("fixture-attempt-{identity}")).map_err(|_| denied())?,
        fence: RemoteFenceGeneration::new(1).map_err(|_| denied())?,
        project: "fixture-project".into(),
        account: "fixture-account".into(),
        request_digest: identity.clone(),
        units: 10,
        schedule: UnitSchedule {
            version: 1,
            unit: "fixture-resource-unit".into(),
            window_id: "fixture-window".into(),
            window_start_s: 1000,
            window_end_s: 2000,
            cpu_ms_per_unit: 10,
            memory_byte_ms_per_unit: 1000,
            executor_output_bytes_per_unit: 10,
        },
    };
    let limits = Limits {
        project_units: 10,
        account_units: 10,
    };
    let completed = UsageEvent::Complete {
        event_digest: identity.clone(),
        usage: Usage {
            cpu_ms: 10,
            memory_byte_ms: 0,
            executor_output_bytes: 0,
        },
        trusted_and_complete: true,
    };
    let mut interrupted = reservation.clone();
    interrupted.units = 9;
    let mut over_quota = reservation.clone();
    over_quota.attempt = RemoteAttemptId::new(format!("fixture-denied-{identity}")).map_err(|_| denied())?;
    let steps = vec![
        Step::Reserve {
            reservation: interrupted,
            limits,
            fault: Fault::BeforePublish,
        },
        Step::Reopen,
        Step::Reserve {
            reservation: reservation.clone(),
            limits,
            fault: Fault::None,
        },
        Step::Reserve {
            reservation: reservation.clone(),
            limits,
            fault: Fault::None,
        },
        Step::Reconcile {
            reservation: reservation.clone(),
            event: UsageEvent::WorkerLost,
            fault: Fault::None,
        },
        Step::Reconcile {
            reservation: reservation.clone(),
            event: completed.clone(),
            fault: Fault::AfterPublish,
        },
        Step::Reopen,
        Step::Reconcile {
            reservation: reservation.clone(),
            event: completed.clone(),
            fault: Fault::None,
        },
        Step::Reconcile {
            reservation: reservation.clone(),
            event: completed,
            fault: Fault::None,
        },
        Step::Reserve {
            reservation: over_quota,
            limits,
            fault: Fault::None,
        },
        Step::Reconcile {
            reservation: reservation.clone(),
            event: UsageEvent::Complete {
                event_digest: identity.clone(),
                usage: Usage {
                    cpu_ms: 20,
                    memory_byte_ms: 0,
                    executor_output_bytes: 0,
                },
                trusted_and_complete: true,
            },
            fault: Fault::None,
        },
    ];
    let report = run_quota_fault_fixture(directory, &steps).map_err(|_| denied())?;
    if report.results.len() != steps.len()
        || report.results[0] != Err(QuotaError::StoreUnavailable)
        || report.results[5] != Err(QuotaError::StoreUncertain)
        || report.results[9] != Err(QuotaError::Denied)
        || report.results[10] != Err(QuotaError::Conflict)
    {
        return Err(denied());
    }
    let held = report.results[4].as_ref().map_err(|_| denied())?.as_ref().ok_or_else(denied)?;
    if held.reconciliation != Reconciliation::Pending(PendingReason::WorkerLost) {
        return Err(denied());
    }
    let settled = report.results[7].as_ref().map_err(|_| denied())?.as_ref().ok_or_else(denied)?;
    if settled.reconciliation
        != (Reconciliation::Settled {
            event_digest: identity,
            charged_units: 1,
        })
        || report.results[7] != report.results[8]
        || report.ledger.entries.len() != 1
    {
        return Err(denied());
    }
    let summary = report.ledger.summary(&reservation).map_err(|_| denied())?;
    if summary.project_charged != 1
        || summary.account_charged != 1
        || summary.project_reserved != 0
        || summary.account_reserved != 0
    {
        return Err(denied());
    }
    let evidence = GatewayOperationEvidence::QuotaFixture {
        expected_steps: steps.len(),
        observed_steps: report.results.len(),
        expected_errors: 4,
        observed_errors: report.results.iter().filter(|result| result.is_err()).count(),
        ledger_entries: report.ledger.entries.len(),
        project_charged: summary.project_charged,
        account_charged: summary.account_charged,
        project_reserved: summary.project_reserved,
        account_reserved: summary.account_reserved,
    };
    Ok((
        serde_json::json!({"schema":"mantle-resource-fault-fixture-v1","seed":seed,
        "schedule":"reserve-before-publish,reopen,reserve,duplicate-reserve,worker-loss,reconcile-after-publish,reopen,reconcile,duplicate-completion,quota-denial,conflicting-completion",
        "evidence_class":report.evidence_class,"results":report.results,"ledger":report.ledger,
        "non_claims":report.non_claims,"vm_execution":false,"snapshot_backed_replay":false}),
        evidence,
    ))
}
