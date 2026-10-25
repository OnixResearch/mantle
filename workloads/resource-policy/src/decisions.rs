// SPDX-License-Identifier: AGPL-3.0-or-later
//! Actual production decisions; factual inputs are harness-owned, not worker authority.
use crunch_remote_core::attempt::RemoteAttemptId;
use crunch_remote_core::attempt::RemoteFenceGeneration;
use crunch_remote_core::attempt::RemoteJobId;
use crunch_remote_core::resource_policy::*;
use crunch_remote_core::resource_quota::*;

pub fn reservation(name: &str) -> Reservation {
    Reservation {
        job: RemoteJobId::new("resource-chaos-job").unwrap(),
        attempt: RemoteAttemptId::new(name).unwrap(),
        fence: RemoteFenceGeneration::INITIAL,
        project: "producer".into(),
        account: "account".into(),
        request_digest: "a".repeat(64),
        units: 10,
        schedule: UnitSchedule {
            version: 1,
            unit: "resource-unit".into(),
            window_id: "bounded-window".into(),
            window_start_s: 1,
            window_end_s: 100,
            cpu_ms_per_unit: 10,
            memory_byte_ms_per_unit: 10,
            executor_output_bytes_per_unit: 10,
        },
    }
}
pub fn complete() -> UsageEvent {
    UsageEvent::Complete {
        event_digest: "b".repeat(64),
        usage: Usage {
            cpu_ms: 10,
            memory_byte_ms: 10,
            executor_output_bytes: 10,
        },
        trusted_and_complete: true,
    }
}
pub fn controls() -> Controls {
    Controls {
        mode: SelectionMode::ProjectOptIn,
        project_opted_in: true,
        selection_enabled: true,
        oom_retry_enabled: true,
        quota_enforcement_enabled: true,
        sharing_enabled: true,
    }
}
fn class(memory: u64, order: u32) -> MachineClass {
    MachineClass {
        id: format!("class-{order}"),
        features: Features {
            architecture: "x86_64".into(),
            platform: "linux".into(),
            trust_domain: "harness".into(),
            isolation: "guest".into(),
            kvm: false,
        },
        quantities: Quantities {
            cpu_units: 1,
            memory_bytes: memory,
            scratch_bytes: 1024,
        },
        order,
        reservation_units: 10,
        available: true,
        existing_admission_compatible: true,
    }
}
/// Cases are explicit boundaries, not fabricated authenticated production observations.
pub fn retry(case: u32) -> Result<EscalationPlan, Reason> {
    let r = reservation("prior");
    let next = RemoteAttemptId::new("next").unwrap();
    let mut context = RetryContext {
        job: &r.job,
        attempt: &r.attempt,
        fence: r.fence,
        next_attempt: &next,
        next_fence: r.fence.advance().unwrap(),
        failed: true,
        retries: 0,
        cumulative_units: 10,
        elapsed_ms: 1,
        next_wall_budget_ms: 9,
    };
    let mut evidence = OomEvidence {
        job: &r.job,
        attempt: &r.attempt,
        fence: r.fence,
        platform: "linux",
        category: OomCategory::LinuxCgroupV2Kill,
        trusted_positive_platform_evidence: true,
    };
    let mut policy = EscalationPolicy {
        version: 1,
        max_retries: 1,
        largest_class_order: 2,
        max_cumulative_units: 20,
        max_wall_ms: 10,
    };
    let mut quota = QuotaFacts {
        project_remaining_units: 10,
        account_remaining_units: 10,
    };
    let mut larger = class(2048, 2);
    match case {
        1 => evidence.category = OomCategory::Unknown,
        2 => evidence.trusted_positive_platform_evidence = false,
        3 => context.retries = 1,
        4 => context.next_fence = r.fence,
        5 => context.elapsed_ms = 2,
        6 => policy.max_cumulative_units = 19,
        7 => quota.project_remaining_units = 9,
        8 => larger.available = false,
        9 => policy.version = 0,
        10 => evidence.fence = r.fence.advance().unwrap(),
        _ => {}
    }
    plan_escalation(EscalationRequest {
        controls: controls(),
        context,
        evidence,
        prior: &class(1024, 1),
        classes: &[larger],
        quota,
        policy,
    })
}
pub const RETRY_REASONS: [Reason; 10] = [
    Reason::NotPositiveOom,
    Reason::NotPositiveOom,
    Reason::RetryLimit,
    Reason::FenceInvalid,
    Reason::WallTimeLimit,
    Reason::ChargeLimit,
    Reason::QuotaDenied,
    Reason::ClassLimit,
    Reason::InvalidInput,
    Reason::FenceInvalid,
];
pub fn linked(plan: &EscalationPlan) -> bool {
    plan.predecessor.as_str() == "prior"
        && plan.next_attempt.as_str() == "next"
        && plan.next_fence.get() == 2
        && plan.class_index == 0
        && plan.policy_version == 1
}
pub fn sharing(cas_available: bool, scope_present: bool) -> Result<SharingLink, Reason> {
    let scope = SharingScope {
        id: "reviewed-sharing",
        producers: &["producer"],
        consumers: &["consumer"],
    };
    authorize_sharing(SharingRequest {
        controls: controls(),
        producer: "producer",
        consumer: "consumer",
        result: "result",
        scope: scope_present.then_some(scope),
        checks: SharingChecks {
            request_and_action_match: true,
            trusted_producer_signature: true,
            policy_compatible: true,
            platform_compatible: true,
            verified_output_identities: true,
            current_cas_available: cas_available,
        },
    })
}
