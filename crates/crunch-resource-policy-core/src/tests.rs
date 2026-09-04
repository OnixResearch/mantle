use alloc::format;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use super::*;

const NOW_UNIX_S: u64 = 5_000_000;
const RECENT_UNIX_S: u64 = 4_999_940;
const STALE_UNIX_S: u64 = 2_407_999;
const SMALL_MEMORY_BYTES: u64 = 1_024;
const MINIMUM_MEMORY_BYTES: u64 = 512;
const MEDIUM_MEMORY_BYTES: u64 = 2_048;
const LARGE_MEMORY_BYTES: u64 = 4_096;
const SMALL_SCRATCH_BYTES: u64 = 512;
const MINIMUM_SCRATCH_BYTES: u64 = 256;
const LARGE_SCRATCH_BYTES: u64 = 2_048;
const OBSERVED_MEMORY_BYTES: u64 = 1_500;
const OBSERVED_SCRATCH_BYTES: u64 = 700;
const SMALL_CHARGE_UNITS: u64 = 100;
const MEDIUM_CHARGE_UNITS: u64 = 200;
const LARGE_CHARGE_UNITS: u64 = 300;
const QUOTA_UNITS: u64 = 1_000;
const LOW_QUOTA_UNITS: u64 = 10;
const WINDOW_START_UNIX_S: u64 = 10_000;
const WINDOW_END_UNIX_S: u64 = 10_600;
const CPU_MS: u64 = 2_000;
const MEMORY_BYTE_MS: u64 = 4_000;
const TRANSFER_BYTES: u64 = 2_000;
const BENCHMARK_LATENCY_MS: u64 = 100;
const BENCHMARK_MEMORY_BYTES: u64 = 512;
const BENCHMARK_TRANSFER_BYTES: u64 = 256;
const BENCHMARK_USAGE_UNITS: u64 = 10;
const ONIXOS_SOURCE_BYTE: char = 'a';
const OVERFLOW_DIVISOR: u64 = u64::MAX;
const OVERFLOW_TRANSFER_BYTES: u64 = 2;
const OUTLIER_MEASUREMENT_BYTES_MAX: u64 = 1_499;
const EXPECTED_ACCOUNTING_CHARGE_UNITS: u64 = 8;
const TEST_CPU_UNITS: u32 = 2;
const SMALL_CLASS_ORDINAL: u32 = 1;
const MEDIUM_CLASS_ORDINAL: u32 = 2;
const LARGE_CLASS_ORDINAL: u32 = 3;
const OBSERVATION_QUEUE_MS: u64 = 10;
const OBSERVATION_EXECUTION_MS: u64 = 20;
const OBSERVATION_CPU_TIME_MS: u64 = 15;
const OBSERVATION_IO_BYTES: u64 = 100;
const OBSERVATION_TRANSFER_BYTES: u64 = 50;
const REQUIRED_FAULT_CASE_COUNT: usize = 6;

type ReuseFactMutation = fn(&mut ResultReuseFacts);
type ReuseRejectionCase = (ReuseFactMutation, ReuseReasonCode);

fn digest(byte: char) -> String {
    core::iter::repeat_n(byte, BLAKE3_HEX_CHARS).collect()
}

#[allow(
    tigerstyle::ambiguous_params,
    reason = "fixture memory and scratch byte arguments are named at every call site"
)]
fn quantities(memory_bytes: u64, scratch_bytes: u64) -> ResourceQuantities {
    ResourceQuantities {
        cpu_units: TEST_CPU_UNITS,
        memory_bytes,
        scratch_bytes,
    }
}

fn platform() -> PlatformRequirements {
    PlatformRequirements {
        architecture: "x86_64".into(),
        platform: "x86_64-linux".into(),
        kvm_required: true,
        trust_tier: "trusted-builder".into(),
        isolation: "microvm".into(),
        required_features: vec!["avx2".into()],
    }
}

fn action_family() -> ActionFamilyIdentity {
    derive_action_family_identity(ActionFamilyInput {
        request_kind: "derivation".into(),
        system: "x86_64-linux".into(),
        builder_class: "rust".into(),
        sandbox_mode: "microvm".into(),
        network_mode: "none".into(),
        required_features: vec!["avx2".into()],
        semantic_accelerator_classes: Vec::new(),
    })
    .unwrap()
}

#[allow(
    tigerstyle::ambiguous_params,
    reason = "fixture class byte and charge arguments use named constants at every call site"
)]
fn machine_class(
    class_id: &str,
    ordinal: u32,
    memory_bytes: u64,
    scratch_bytes: u64,
    charge_units: u64,
) -> MachineClass {
    MachineClass {
        class_id: class_id.into(),
        endpoint_ids: vec![format!("endpoint-{class_id}")],
        ordinal,
        architecture: "x86_64".into(),
        platform: "x86_64-linux".into(),
        kvm_available: true,
        trust_tier: "trusted-builder".into(),
        isolation: "microvm".into(),
        features: vec!["avx2".into()],
        capacity: quantities(memory_bytes, scratch_bytes),
        reservation_charge_units: charge_units,
        available: true,
        onixos_source_blake3: digest(ONIXOS_SOURCE_BYTE),
    }
}

fn classes() -> Vec<MachineClass> {
    vec![
        machine_class("small", SMALL_CLASS_ORDINAL, SMALL_MEMORY_BYTES, SMALL_SCRATCH_BYTES, SMALL_CHARGE_UNITS),
        machine_class("medium", MEDIUM_CLASS_ORDINAL, MEDIUM_MEMORY_BYTES, LARGE_SCRATCH_BYTES, MEDIUM_CHARGE_UNITS),
        machine_class("large", LARGE_CLASS_ORDINAL, LARGE_MEMORY_BYTES, LARGE_SCRATCH_BYTES, LARGE_CHARGE_UNITS),
    ]
}

fn controls(mode: ResourcePolicyMode) -> ResourceFeatureControls {
    ResourceFeatureControls {
        mode,
        project_opted_in: true,
        historical_selection_enabled: true,
        oom_retry_enabled: true,
        quota_enforcement_enabled: true,
        result_sharing_enabled: true,
    }
}

fn observation(label: &str, observed_at_unix_s: u64) -> ResourceObservation {
    build_resource_observation(ResourceObservationInput {
        attempt_id: format!("attempt-{label}"),
        action_family_blake3: action_family().identity_blake3,
        platform_identity: "x86_64-linux".into(),
        machine_class_id: "medium".into(),
        declared: quantities(MINIMUM_MEMORY_BYTES, MINIMUM_SCRATCH_BYTES),
        selected: quantities(MEDIUM_MEMORY_BYTES, LARGE_SCRATCH_BYTES),
        timing: ResourceTiming {
            queue_ms: OBSERVATION_QUEUE_MS,
            execution_ms: OBSERVATION_EXECUTION_MS,
            terminal_ms: 1,
        },
        measurements: ResourceMeasurements {
            cpu_time_ms: OBSERVATION_CPU_TIME_MS,
            peak_memory_bytes: OBSERVED_MEMORY_BYTES,
            scratch_peak_bytes: OBSERVED_SCRATCH_BYTES,
            io_bytes: OBSERVATION_IO_BYTES,
            transfer_bytes: OBSERVATION_TRANSFER_BYTES,
            wall_time_ms: OBSERVATION_EXECUTION_MS,
        },
        oom_evidence: OomEvidence {
            category: OomEvidenceCategory::None,
            platform: "x86_64-linux".into(),
            evidence_ref: None,
            trusted: false,
        },
        terminal_outcome: TerminalOutcome::Succeeded,
        retry_predecessor_attempt_id: None,
        collector_id: "mantle-worker".into(),
        collector_version: "v1".into(),
        compatibility_policy_id: ResourceSelectionPolicy::default().policy_id,
        observed_at_unix_s,
        trusted: true,
    })
    .unwrap()
}

fn selection_request(mode: ResourcePolicyMode) -> ResourceSelectionRequest {
    ResourceSelectionRequest {
        now_unix_s: NOW_UNIX_S,
        action_family: action_family(),
        declared: DeclaredResourceRequirements {
            minima: quantities(MINIMUM_MEMORY_BYTES, MINIMUM_SCRATCH_BYTES),
            platform: platform(),
        },
        machine_classes: classes(),
        quota: QuotaFacts {
            project_remaining_units: QUOTA_UNITS,
            account_remaining_units: QUOTA_UNITS,
        },
        observations: vec![observation("one", RECENT_UNIX_S), observation("two", RECENT_UNIX_S)],
        policy: ResourceSelectionPolicy::default(),
        controls: controls(mode),
    }
}

#[test]
fn action_family_is_canonical_and_rejects_control_bearing_unknown_fields() {
    let first = action_family();
    let mut reordered = ActionFamilyInput {
        request_kind: "derivation".into(),
        system: "x86_64-linux".into(),
        builder_class: "rust".into(),
        sandbox_mode: "microvm".into(),
        network_mode: "none".into(),
        required_features: vec!["sse4".into(), "avx2".into()],
        semantic_accelerator_classes: Vec::new(),
    };
    let ordered = derive_action_family_identity(reordered.clone()).unwrap();
    reordered.required_features.reverse();
    let replay = derive_action_family_identity(reordered).unwrap();
    assert_eq!(ordered, replay);
    assert_eq!(validate_action_family_identity(first.clone()).unwrap(), first);

    let mut value = serde_json::to_value(first).unwrap();
    value.as_object_mut().unwrap().insert("environment".into(), serde_json::json!({"TOKEN": "secret"}));
    assert!(serde_json::from_value::<ActionFamilyIdentity>(value).is_err());
}

#[test]
fn observations_are_tamper_evident_and_filter_reason_codes_are_explicit() {
    let valid = observation("valid", RECENT_UNIX_S);
    assert_eq!(validate_resource_observation(valid.clone()).unwrap(), valid);
    let mut tampered = valid.clone();
    tampered.measurements.peak_memory_bytes =
        tampered.measurements.peak_memory_bytes.checked_add(1).expect("fixture tamper increment fits");
    assert_eq!(validate_resource_observation(tampered), Err(ResourcePolicyError::InvalidDigest));

    let mut stale = observation("stale", STALE_UNIX_S);
    let mut untrusted = observation("untrusted", RECENT_UNIX_S);
    untrusted.trusted = false;
    untrusted.observation_blake3 = canonical_blake3_for_test(&untrusted);
    stale.trusted = true;
    let filtered_history = filter_resource_observations(ObservationFilterRequest {
        now_unix_s: NOW_UNIX_S,
        action_family_blake3: action_family().identity_blake3,
        platform_identity: "x86_64-linux".into(),
        compatibility_policy_id: ResourceSelectionPolicy::default().policy_id,
        known_machine_class_ids: vec!["medium".into()],
        observation_age_secs_max: DEFAULT_OBSERVATION_AGE_SECS_MAX,
        measurement_bytes_max: MAX_MEASUREMENT_BYTES,
        observations: vec![valid, stale, untrusted],
    })
    .unwrap();
    assert_eq!(filtered_history.accepted.len(), 1);
    assert!(filtered_history.decisions.iter().any(|row| row.reason == HistoryReasonCode::Stale));
    assert!(filtered_history.decisions.iter().any(|row| row.reason == HistoryReasonCode::Untrusted));

    let missing = filter_resource_observations(ObservationFilterRequest {
        now_unix_s: NOW_UNIX_S,
        action_family_blake3: action_family().identity_blake3,
        platform_identity: "x86_64-linux".into(),
        compatibility_policy_id: ResourceSelectionPolicy::default().policy_id,
        known_machine_class_ids: vec!["medium".into()],
        observation_age_secs_max: DEFAULT_OBSERVATION_AGE_SECS_MAX,
        measurement_bytes_max: MAX_MEASUREMENT_BYTES,
        observations: Vec::new(),
    })
    .unwrap();
    assert_eq!(missing.decisions[0].reason, HistoryReasonCode::Missing);

    let mut incompatible = observation("incompatible", RECENT_UNIX_S);
    incompatible.action_family_blake3 = digest('f');
    incompatible.observation_blake3 = canonical_blake3_for_test(&incompatible);
    let incompatible = filter_resource_observations(ObservationFilterRequest {
        now_unix_s: NOW_UNIX_S,
        action_family_blake3: action_family().identity_blake3,
        platform_identity: "x86_64-linux".into(),
        compatibility_policy_id: ResourceSelectionPolicy::default().policy_id,
        known_machine_class_ids: vec!["medium".into()],
        observation_age_secs_max: DEFAULT_OBSERVATION_AGE_SECS_MAX,
        measurement_bytes_max: MAX_MEASUREMENT_BYTES,
        observations: vec![incompatible],
    })
    .unwrap();
    assert_eq!(incompatible.decisions[0].reason, HistoryReasonCode::ActionFamilyMismatch);

    let outlier = filter_resource_observations(ObservationFilterRequest {
        now_unix_s: NOW_UNIX_S,
        action_family_blake3: action_family().identity_blake3,
        platform_identity: "x86_64-linux".into(),
        compatibility_policy_id: ResourceSelectionPolicy::default().policy_id,
        known_machine_class_ids: vec!["medium".into()],
        observation_age_secs_max: DEFAULT_OBSERVATION_AGE_SECS_MAX,
        measurement_bytes_max: OUTLIER_MEASUREMENT_BYTES_MAX,
        observations: vec![observation("outlier", RECENT_UNIX_S)],
    })
    .unwrap();
    assert_eq!(outlier.decisions[0].reason, HistoryReasonCode::MeasurementOutlier);
}

fn canonical_blake3_for_test(observation: &ResourceObservation) -> String {
    build_resource_observation(ResourceObservationInput {
        attempt_id: observation.attempt_id.clone(),
        action_family_blake3: observation.action_family_blake3.clone(),
        platform_identity: observation.platform_identity.clone(),
        machine_class_id: observation.machine_class_id.clone(),
        declared: observation.declared,
        selected: observation.selected,
        timing: observation.timing,
        measurements: observation.measurements,
        oom_evidence: observation.oom_evidence.clone(),
        terminal_outcome: observation.terminal_outcome,
        retry_predecessor_attempt_id: observation.retry_predecessor_attempt_id.clone(),
        collector_id: observation.collector_id.clone(),
        collector_version: observation.collector_version.clone(),
        compatibility_policy_id: observation.compatibility_policy_id.clone(),
        observed_at_unix_s: observation.observed_at_unix_s,
        trusted: observation.trusted,
    })
    .unwrap()
    .observation_blake3
}

#[test]
fn observe_only_records_historical_choice_but_schedules_static_class() {
    let decision = select_resource_class(selection_request(ResourcePolicyMode::ObserveOnly)).unwrap();
    assert_eq!(decision.static_class_id, "small");
    assert_eq!(decision.policy_class_id, "medium");
    assert_eq!(decision.scheduled_class_id, "small");
    assert!(decision.observe_only);
    assert!(decision.reason_codes.contains(&SelectionReasonCode::ObserveOnlyStaticRetained));
    validate_resource_selection_decision(decision).unwrap();
}

#[test]
fn opted_in_enforcement_selects_history_without_weakening_hard_requirements() {
    let decision = select_resource_class(selection_request(ResourcePolicyMode::Enforce)).unwrap();
    assert_eq!(decision.static_class_id, "small");
    assert_eq!(decision.scheduled_class_id, "medium");
    assert!(decision.effective_minima.memory_bytes >= OBSERVED_MEMORY_BYTES);

    let mut mismatch = selection_request(ResourcePolicyMode::Enforce);
    for class in &mut mismatch.machine_classes {
        class.kvm_available = false;
    }
    assert_eq!(select_resource_class(mismatch), Err(ResourcePolicyError::NoEligibleClass));
}

#[test]
fn selection_rejects_quota_and_feature_mismatch_only_when_enforced() {
    let mut enforced = selection_request(ResourcePolicyMode::Enforce);
    enforced.quota.project_remaining_units = LOW_QUOTA_UNITS;
    assert_eq!(select_resource_class(enforced), Err(ResourcePolicyError::QuotaDenied));

    let mut observed = selection_request(ResourcePolicyMode::ObserveOnly);
    observed.quota.project_remaining_units = LOW_QUOTA_UNITS;
    let decision = select_resource_class(observed).unwrap();
    assert_eq!(decision.scheduled_class_id, "small");
    assert!(decision.reason_codes.contains(&SelectionReasonCode::QuotaDeniedPolicyFallback));
}

#[test]
fn selection_is_deterministic_stably_ordered_monotonic_and_bounded() {
    let request = selection_request(ResourcePolicyMode::Enforce);
    let first = select_resource_class(request.clone()).unwrap();
    let mut reordered = request;
    reordered.machine_classes.reverse();
    reordered.observations.reverse();
    let replay = select_resource_class(reordered).unwrap();
    assert_eq!(first, replay);

    let mut previous_ordinal = 0_u32;
    for memory in [
        MINIMUM_MEMORY_BYTES,
        SMALL_MEMORY_BYTES,
        MEDIUM_MEMORY_BYTES,
        LARGE_MEMORY_BYTES,
    ] {
        let mut monotonic = selection_request(ResourcePolicyMode::Enforce);
        monotonic.controls.historical_selection_enabled = false;
        monotonic.observations.clear();
        monotonic.declared.minima.memory_bytes = memory;
        let decision = select_resource_class(monotonic).unwrap();
        let ordinal = classes().iter().find(|class| class.class_id == decision.scheduled_class_id).unwrap().ordinal;
        assert!(ordinal >= previous_ordinal);
        previous_ordinal = ordinal;
    }
    assert!(first.eligible_class_ids.len() <= usize::try_from(MAX_MACHINE_CLASSES).unwrap());
    assert!(first.history_decisions.len() <= usize::try_from(MAX_OBSERVATIONS).unwrap());
}

fn retry_request() -> ResourceRetryRequest {
    ResourceRetryRequest {
        predecessor_attempt_id: "attempt-one".into(),
        predecessor_fence_generation: 1,
        current_machine_class_id: "small".into(),
        oom_evidence: OomEvidence {
            category: OomEvidenceCategory::LinuxCgroupV2OomKill,
            platform: "x86_64-linux".into(),
            evidence_ref: Some("cgroup-v2:oom-kill:attempt-one".into()),
            trusted: true,
        },
        retry_history: vec![RetryHistoryEntry {
            attempt_id: "attempt-one".into(),
            machine_class_id: "small".into(),
            charge_units: SMALL_CHARGE_UNITS,
            wall_time_ms: BENCHMARK_LATENCY_MS,
        }],
        eligible_classes: classes(),
        quota: QuotaFacts {
            project_remaining_units: QUOTA_UNITS,
            account_remaining_units: QUOTA_UNITS,
        },
        policy: ResourceRetryPolicy::default(),
        controls: controls(ResourcePolicyMode::Enforce),
    }
}

#[test]
fn rollback_controls_disable_one_authority_without_disabling_observation_or_selection() {
    let selection = select_resource_class(selection_request(ResourcePolicyMode::Enforce)).unwrap();
    assert_eq!(selection.scheduled_class_id, "medium");
    assert!(!selection.compatible_observation_blake3s.is_empty());

    let mut retry = retry_request();
    retry.controls.oom_retry_enabled = false;
    assert_eq!(retry_reason(retry), RetryReasonCode::RetryDisabled);
    assert_eq!(selection.scheduled_class_id, selection.policy_class_id);
}

#[test]
fn positive_oom_creates_strictly_larger_new_fenced_attempt_intent() {
    let plan = plan_resource_retry(retry_request()).unwrap();
    assert_eq!(plan.prior_machine_class_id, "small");
    assert_eq!(plan.next_machine_class_id, "medium");
    assert_eq!(
        plan.successor_fence_generation,
        plan.predecessor_fence_generation.checked_add(1).expect("fixture fence increment fits")
    );
    assert!(plan.new_fenced_attempt_required);
    validate_resource_retry_plan(plan).unwrap();
}

#[test]
fn retry_rejects_arbitrary_ambiguous_untrusted_exhausted_and_quota_failures() {
    let mut ambiguous = retry_request();
    ambiguous.oom_evidence.category = OomEvidenceCategory::AmbiguousExit;
    assert_eq!(retry_reason(ambiguous), RetryReasonCode::OomEvidenceMissing);

    let mut untrusted = retry_request();
    untrusted.oom_evidence.trusted = false;
    assert_eq!(retry_reason(untrusted), RetryReasonCode::OomEvidenceUntrusted);

    let mut exhausted = retry_request();
    exhausted.policy.retry_count_max = 1;
    assert_eq!(retry_reason(exhausted), RetryReasonCode::RetryLimitExhausted);

    let mut unavailable = retry_request();
    unavailable.eligible_classes.truncate(1);
    assert_eq!(retry_reason(unavailable), RetryReasonCode::LargerClassUnavailable);

    let mut quota = retry_request();
    quota.quota.project_remaining_units = LOW_QUOTA_UNITS;
    assert_eq!(retry_reason(quota), RetryReasonCode::QuotaDenied);

    let mut disabled = retry_request();
    disabled.controls.mode = ResourcePolicyMode::ObserveOnly;
    assert_eq!(retry_reason(disabled), RetryReasonCode::RetryDisabled);

    let mut platform = retry_request();
    platform.oom_evidence.category = OomEvidenceCategory::WindowsJobObjectMemoryLimit;
    assert_eq!(retry_reason(platform), RetryReasonCode::OomPlatformMismatch);

    let mut class_bound_case = retry_request();
    class_bound_case.policy.class_ordinal_max = SMALL_CLASS_ORDINAL;
    assert_eq!(retry_reason(class_bound_case), RetryReasonCode::ClassLimitExceeded);

    let mut charge_bound_case = retry_request();
    charge_bound_case.policy.cumulative_charge_units_max = SMALL_CHARGE_UNITS;
    assert_eq!(retry_reason(charge_bound_case), RetryReasonCode::ChargeLimitExceeded);

    let mut wall_time_bound_case = retry_request();
    wall_time_bound_case.policy.wall_time_ms_max = 1;
    assert_eq!(retry_reason(wall_time_bound_case), RetryReasonCode::WallTimeLimitExceeded);
}

fn retry_reason(request: ResourceRetryRequest) -> RetryReasonCode {
    match plan_resource_retry(request).unwrap_err() {
        ResourcePolicyError::RetryRejected(reason) => reason,
        error => panic!("unexpected retry error: {error}"),
    }
}

fn schedule() -> UsageUnitSchedule {
    UsageUnitSchedule {
        schedule_id: "public-units-v1".into(),
        cpu_ms_per_unit: DEFAULT_UNIT_SCHEDULE_DIVISOR,
        memory_byte_ms_per_unit: DEFAULT_UNIT_SCHEDULE_DIVISOR,
        transfer_bytes_per_unit: DEFAULT_UNIT_SCHEDULE_DIVISOR,
        charge_units_max: QUOTA_UNITS,
    }
}

fn reservation_request(attempt_id: &str) -> UsageReservationRequest {
    UsageReservationRequest {
        attempt_id: attempt_id.into(),
        project_id: "project-a".into(),
        account_id: "account-a".into(),
        window_start_unix_s: WINDOW_START_UNIX_S,
        window_end_unix_s: WINDOW_END_UNIX_S,
        reserved_units: SMALL_CHARGE_UNITS,
        project_limit_units: QUOTA_UNITS,
        account_limit_units: QUOTA_UNITS,
        policy_id: "accounting-v1".into(),
        schedule: schedule(),
    }
}

fn reconciliation_request(attempt_id: &str, kind: UsageReconciliationKind) -> UsageReconciliationRequest {
    UsageReconciliationRequest {
        attempt_id: attempt_id.into(),
        kind,
        observed: Some(UsageAmounts {
            cpu_ms: CPU_MS,
            memory_byte_ms: MEMORY_BYTE_MS,
            transfer_bytes: TRANSFER_BYTES,
        }),
        policy_id: "accounting-v1".into(),
        schedule: schedule(),
    }
}

#[test]
fn usage_reservation_and_reconciliation_are_idempotent_and_never_double_charge() {
    let added = plan_usage_reservation(UsageLedgerState::default(), reservation_request("attempt-a")).unwrap();
    assert_eq!(added.disposition, LedgerMutationDisposition::Add);
    assert!(added.quota_granted);
    let reused = plan_usage_reservation(added.next_state.clone(), reservation_request("attempt-a")).unwrap();
    assert_eq!(reused.disposition, LedgerMutationDisposition::Reuse);
    assert_eq!(reused.prior_state_blake3, reused.next_state_blake3);

    let reconciled = plan_usage_reconciliation(
        reused.next_state,
        reconciliation_request("attempt-a", UsageReconciliationKind::Completed),
    )
    .unwrap();
    let duplicate = plan_usage_reconciliation(
        reconciled.next_state.clone(),
        reconciliation_request("attempt-a", UsageReconciliationKind::Completed),
    )
    .unwrap();
    assert_eq!(duplicate.disposition, LedgerMutationDisposition::Reuse);
    assert_eq!(
        usage_totals_by_project(duplicate.next_state.clone()).unwrap()["project-a"],
        EXPECTED_ACCOUNTING_CHARGE_UNITS
    );
    let summary = summarize_usage_ledger(duplicate.next_state).unwrap();
    assert_eq!(summary.reservation_count, 1);
    assert_eq!(summary.reconciliation_count, 1);
    assert_eq!(summary.project_units[0].active_reserved_units, 0);
    assert_eq!(summary.project_units[0].charged_units, EXPECTED_ACCOUNTING_CHARGE_UNITS);
}

#[test]
fn accounting_rejects_conflict_quota_missing_overflow_and_preserves_explicit_terminal_states() {
    let added = plan_usage_reservation(UsageLedgerState::default(), reservation_request("attempt-a")).unwrap();
    let mut conflict = reservation_request("attempt-a");
    conflict.reserved_units = conflict.reserved_units.checked_add(1).expect("fixture conflict increment fits");
    assert_eq!(
        plan_usage_reservation(added.next_state.clone(), conflict),
        Err(ResourcePolicyError::ReservationConflict)
    );

    let mut denied = reservation_request("attempt-b");
    denied.project_limit_units = LOW_QUOTA_UNITS;
    assert_eq!(plan_usage_reservation(added.next_state.clone(), denied), Err(ResourcePolicyError::QuotaDenied));
    assert_eq!(
        plan_usage_reconciliation(
            added.next_state.clone(),
            reconciliation_request("missing", UsageReconciliationKind::Completed),
        ),
        Err(ResourcePolicyError::ReservationMissing)
    );
    let mut overflow = reconciliation_request("attempt-a", UsageReconciliationKind::Completed);
    overflow.observed = Some(UsageAmounts {
        cpu_ms: 0,
        memory_byte_ms: 0,
        transfer_bytes: OVERFLOW_TRANSFER_BYTES,
    });
    overflow.schedule.transfer_bytes_per_unit = OVERFLOW_DIVISOR;
    assert_eq!(
        plan_usage_reconciliation(added.next_state.clone(), overflow),
        Err(ResourcePolicyError::ArithmeticOverflow)
    );

    for (attempt, kind, expected) in [
        ("cancelled", UsageReconciliationKind::Cancelled, "usage-cancelled-measured"),
        ("lost", UsageReconciliationKind::WorkerLost, "usage-worker-lost-reservation-retained"),
        ("partial", UsageReconciliationKind::PartialObservation, "usage-partial-measured"),
    ] {
        let reserved = plan_usage_reservation(UsageLedgerState::default(), reservation_request(attempt)).unwrap();
        let outcome = plan_usage_reconciliation(reserved.next_state, reconciliation_request(attempt, kind)).unwrap();
        assert_eq!(outcome.reconciliation.unwrap().reason_code, expected);
    }
}

fn sharing_facts() -> ResultReuseFacts {
    ResultReuseFacts {
        producer_project_id: "project-a".into(),
        consumer_project_id: "project-a".into(),
        producer_result_blake3: digest('b'),
        consumer_request_blake3: digest('c'),
        request_identity_matches: true,
        action_identity_matches: true,
        producer_signature_trusted: true,
        policy_compatible: true,
        platform_compatible: true,
        output_identity_verified: true,
        cas_available: true,
        strong_reuse_admitted: true,
        authorization_decision_blake3: digest('d'),
    }
}

fn private_policy() -> ResultSharingPolicy {
    ResultSharingPolicy {
        schema: RESULT_SHARING_POLICY_SCHEMA.into(),
        policy_id: "private-v1".into(),
        scope: SharingScopeKind::Private,
        project_id: None,
        allowed_producers: Vec::new(),
        allowed_consumers: Vec::new(),
    }
}

#[test]
fn result_sharing_is_private_by_default_and_named_scope_records_evidence_link() {
    let same = decide_result_reuse(private_policy(), sharing_facts(), controls(ResourcePolicyMode::Enforce)).unwrap();
    assert!(same.usable);
    assert!(same.evidence_link.is_some());

    let mut cross = sharing_facts();
    cross.consumer_project_id = "project-b".into();
    let denied = decide_result_reuse(private_policy(), cross.clone(), controls(ResourcePolicyMode::Enforce)).unwrap();
    assert_eq!(denied.reason, ReuseReasonCode::ConsumerScopeDenied);
    assert!(!denied.usable);

    let named = ResultSharingPolicy {
        schema: RESULT_SHARING_POLICY_SCHEMA.into(),
        policy_id: "named-v1".into(),
        scope: SharingScopeKind::Named,
        project_id: None,
        allowed_producers: vec!["project-a".into()],
        allowed_consumers: vec!["project-b".into()],
    };
    let allowed = decide_result_reuse(named, cross, controls(ResourcePolicyMode::Enforce)).unwrap();
    assert!(allowed.usable);
    validate_result_reuse_decision(allowed).unwrap();
}

#[test]
fn result_reuse_rejects_each_missing_authority_fact_without_mutation() {
    let cases: &[ReuseRejectionCase] = &[
        (|facts| facts.request_identity_matches = false, ReuseReasonCode::RequestMismatch),
        (|facts| facts.action_identity_matches = false, ReuseReasonCode::ActionMismatch),
        (|facts| facts.producer_signature_trusted = false, ReuseReasonCode::SignatureUntrusted),
        (|facts| facts.policy_compatible = false, ReuseReasonCode::PolicyMismatch),
        (|facts| facts.platform_compatible = false, ReuseReasonCode::PlatformMismatch),
        (|facts| facts.output_identity_verified = false, ReuseReasonCode::OutputIdentityInvalid),
        (|facts| facts.cas_available = false, ReuseReasonCode::CasUnavailable),
        (|facts| facts.strong_reuse_admitted = false, ReuseReasonCode::StrongReuseRejected),
    ];
    for (mutate, expected) in cases {
        let mut facts = sharing_facts();
        mutate(&mut facts);
        let decision = decide_result_reuse(private_policy(), facts, controls(ResourcePolicyMode::Enforce)).unwrap();
        assert_eq!(decision.reason, *expected);
        assert!(decision.evidence_link.is_none());
    }
}

fn benchmark_workload() -> ResourceBenchmarkWorkload {
    ResourceBenchmarkWorkload {
        workload_id: "fixed-output-write".into(),
        source: BenchmarkSource {
            repository: "github.com/nixbuild/nixbench".into(),
            revision: "b256cd275d8c79ba485be8d317005f973879825a".into(),
            source_blake3: "737a5a42181fdd59ba451159db7558c516f31ea82ffcc771f66a0f2d6421a5a8".into(),
            license: "Apache-2.0".into(),
            license_blake3: "14ec1590aae4c4e763d123c31085d5d37703f7a838e589b0d356058ad99177dd".into(),
            adaptation: "fixed-seed-size-compressibility-and-cpu-shape-only".into(),
        },
        fixed_input_blake3: digest('e'),
        platform: platform(),
        expected_completion_class: "succeeded".into(),
        latency_ms_max: BENCHMARK_LATENCY_MS,
        memory_bytes_max: BENCHMARK_MEMORY_BYTES,
        transfer_bytes_max: BENCHMARK_TRANSFER_BYTES,
        usage_units_max: BENCHMARK_USAGE_UNITS,
    }
}

fn benchmark_observation() -> ResourceBenchmarkObservation {
    ResourceBenchmarkObservation {
        workload_id: "fixed-output-write".into(),
        static_class_id: "small".into(),
        policy_class_id: "medium".into(),
        completion_class: "succeeded".into(),
        compatible: true,
        correct_output: true,
        throughput_units: 1,
        latency_ms: BENCHMARK_LATENCY_MS,
        memory_bytes: BENCHMARK_MEMORY_BYTES,
        transfer_bytes: BENCHMARK_TRANSFER_BYTES,
        usage_units: BENCHMARK_USAGE_UNITS,
    }
}

#[test]
fn benchmark_separates_correctness_compatibility_and_bounded_measurements() {
    let benchmark_evidence = build_resource_benchmark_report(
        vec![benchmark_workload()],
        vec![benchmark_observation()],
        "static-v1".into(),
        "history-v1".into(),
    )
    .unwrap();
    assert!(benchmark_evidence.correctness_passed);
    assert!(benchmark_evidence.compatibility_passed);
    validate_resource_benchmark_report(benchmark_evidence).unwrap();

    let mut slow = benchmark_observation();
    slow.latency_ms = slow.latency_ms.checked_add(1).expect("fixture latency increment fits");
    let slow_benchmark_evidence = build_resource_benchmark_report(
        vec![benchmark_workload()],
        vec![slow],
        "static-v1".into(),
        "history-v1".into(),
    )
    .unwrap();
    assert!(slow_benchmark_evidence.correctness_passed);
    assert!(!slow_benchmark_evidence.compatibility_passed);
}

#[allow(
    tigerstyle::ambiguous_params,
    reason = "fixture case ID and reason code are distinct named string fields"
)]
fn fault_case(kind: ResourceFaultKind, id: &str, reason: &str, mutation: bool) -> ResourceFaultCase {
    ResourceFaultCase {
        case_id: id.into(),
        kind,
        expected_reason_code: reason.into(),
        expected_mutation: mutation,
    }
}

#[test]
fn chaoscontrol_campaign_requires_all_six_bounded_fault_classes() {
    let campaign = ResourceFaultCampaign {
        schema: RESOURCE_FAULT_CAMPAIGN_SCHEMA.into(),
        chaoscontrol_revision: "31300fa1a2d29c7496e8316f065c156f80343143".into(),
        chaoscontrol_contract_path: "crates/chaoscontrol-sim-core/src/runtime_capacity.rs".into(),
        chaoscontrol_contract_blake3: digest('f'),
        cases: vec![
            fault_case(ResourceFaultKind::PositiveOom, "oom", "positive-oom-evidence", true),
            fault_case(ResourceFaultKind::AmbiguousFailure, "ambiguous", "oom-evidence-missing", false),
            fault_case(ResourceFaultKind::WorkerLoss, "worker-loss", "usage-worker-lost-reservation-retained", true),
            fault_case(ResourceFaultKind::DuplicateCompletion, "duplicate", "usage-reconciliation-reused", false),
            fault_case(ResourceFaultKind::AccountingInterruption, "accounting", "accounting-store-failed", false),
            fault_case(ResourceFaultKind::CasUnavailable, "cas", "result-cas-unavailable", false),
        ],
        non_claim: RESOURCE_POLICY_NON_CLAIM.into(),
    };
    assert_eq!(validate_resource_fault_campaign(campaign.clone()).unwrap().cases.len(), REQUIRED_FAULT_CASE_COUNT);
    let mut missing = campaign;
    missing.cases.pop();
    assert_eq!(validate_resource_fault_campaign(missing), Err(ResourcePolicyError::BenchmarkRejected));
}
