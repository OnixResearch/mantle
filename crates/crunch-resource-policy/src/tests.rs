use pretty_assertions::assert_eq;
use valence_core::build_service as valence;

use super::*;

const NOW_UNIX_S: u64 = 5_000_000;
const OBSERVED_AT_UNIX_S: u64 = 4_999_940;
const WINDOW_START_UNIX_S: u64 = 10_000;
const WINDOW_END_UNIX_S: u64 = 10_600;
const SMALL_MEMORY_BYTES: u64 = 1_024;
const MEDIUM_MEMORY_BYTES: u64 = 2_048;
const SMALL_SCRATCH_BYTES: u64 = 512;
const MEDIUM_SCRATCH_BYTES: u64 = 1_024;
const SMALL_CHARGE_UNITS: u64 = 100;
const MEDIUM_CHARGE_UNITS: u64 = 200;
const QUOTA_UNITS: u64 = 1_000;
const REQUEST_GENERATION: u32 = 1;
const RETRY_GENERATION: u32 = 2;
const CPU_UNITS: u32 = 2;
const OBSERVATION_SAMPLE_MIN: u32 = 1;
const SMALL_CLASS_ORDINAL: u32 = 1;
const MEDIUM_CLASS_ORDINAL: u32 = 2;
const OBSERVATION_QUEUE_MS: u64 = 10;
const OBSERVATION_EXECUTION_MS: u64 = 100;
const OBSERVATION_CPU_TIME_MS: u64 = 80;
const OBSERVATION_PEAK_MEMORY_BYTES: u64 = 1_224;
const OBSERVATION_IO_BYTES: u64 = 100;
const OBSERVATION_TRANSFER_BYTES: u64 = 50;
const EXPECTED_USAGE_RECORD_COUNT: usize = 2;
const EXPECTED_LEDGER_COMMITS_AFTER_RECONCILIATION: u32 = 2;

fn digest(byte: char) -> String {
    std::iter::repeat_n(byte, BLAKE3_HEX_CHARS).collect()
}

#[allow(
    tigerstyle::ambiguous_params,
    reason = "fixture memory and scratch byte arguments are named at every call site"
)]
fn quantities(memory_bytes: u64, scratch_bytes: u64) -> ResourceQuantities {
    ResourceQuantities {
        cpu_units: CPU_UNITS,
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

#[allow(
    tigerstyle::ambiguous_params,
    reason = "fixture capacity and charge arguments use named constants at every call site"
)]
fn class(id: &str, ordinal: u32, memory: u64, scratch: u64, charge: u64) -> MachineClass {
    MachineClass {
        class_id: id.into(),
        endpoint_ids: vec![format!("endpoint-{id}")],
        ordinal,
        architecture: "x86_64".into(),
        platform: "x86_64-linux".into(),
        kvm_available: true,
        trust_tier: "trusted-builder".into(),
        isolation: "microvm".into(),
        features: vec!["avx2".into()],
        capacity: quantities(memory, scratch),
        reservation_charge_units: charge,
        available: true,
        onixos_source_blake3: digest('a'),
    }
}

fn classes() -> Vec<MachineClass> {
    vec![
        class("small", SMALL_CLASS_ORDINAL, SMALL_MEMORY_BYTES, SMALL_SCRATCH_BYTES, SMALL_CHARGE_UNITS),
        class("medium", MEDIUM_CLASS_ORDINAL, MEDIUM_MEMORY_BYTES, MEDIUM_SCRATCH_BYTES, MEDIUM_CHARGE_UNITS),
    ]
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

fn observation(attempt_id: &str) -> ResourceObservation {
    build_resource_observation(ResourceObservationInput {
        attempt_id: attempt_id.into(),
        action_family_blake3: action_family().identity_blake3,
        platform_identity: "x86_64-linux".into(),
        machine_class_id: "small".into(),
        declared: quantities(SMALL_MEMORY_BYTES, SMALL_SCRATCH_BYTES),
        selected: quantities(SMALL_MEMORY_BYTES, SMALL_SCRATCH_BYTES),
        timing: ResourceTiming {
            queue_ms: OBSERVATION_QUEUE_MS,
            execution_ms: OBSERVATION_EXECUTION_MS,
            terminal_ms: 1,
        },
        measurements: ResourceMeasurements {
            cpu_time_ms: OBSERVATION_CPU_TIME_MS,
            peak_memory_bytes: OBSERVATION_PEAK_MEMORY_BYTES,
            scratch_peak_bytes: SMALL_SCRATCH_BYTES,
            io_bytes: OBSERVATION_IO_BYTES,
            transfer_bytes: OBSERVATION_TRANSFER_BYTES,
            wall_time_ms: OBSERVATION_EXECUTION_MS,
        },
        oom_evidence: OomEvidence {
            category: OomEvidenceCategory::LinuxCgroupV2OomKill,
            platform: "x86_64-linux".into(),
            evidence_ref: Some(format!("cgroup-v2:oom-kill:{attempt_id}")),
            trusted: true,
        },
        terminal_outcome: TerminalOutcome::FailedOom,
        retry_predecessor_attempt_id: None,
        collector_id: "mantle-worker".into(),
        collector_version: "v1".into(),
        compatibility_policy_id: "selection-v1".into(),
        observed_at_unix_s: OBSERVED_AT_UNIX_S,
        trusted: true,
    })
    .unwrap()
}

fn controls() -> ResourceFeatureControls {
    ResourceFeatureControls {
        mode: ResourcePolicyMode::Enforce,
        project_opted_in: true,
        historical_selection_enabled: true,
        oom_retry_enabled: true,
        quota_enforcement_enabled: true,
        result_sharing_enabled: true,
    }
}

fn selection() -> ResourceSelectionDecision {
    let policy = ResourceSelectionPolicy {
        policy_id: "selection-v1".into(),
        sample_count_min: OBSERVATION_SAMPLE_MIN,
        ..ResourceSelectionPolicy::default()
    };
    let decision = select_resource_class(ResourceSelectionRequest {
        now_unix_s: NOW_UNIX_S,
        action_family: action_family(),
        declared: DeclaredResourceRequirements {
            minima: quantities(SMALL_MEMORY_BYTES, SMALL_SCRATCH_BYTES),
            platform: platform(),
        },
        machine_classes: classes(),
        quota: QuotaFacts {
            project_remaining_units: QUOTA_UNITS,
            account_remaining_units: QUOTA_UNITS,
        },
        observations: vec![observation("attempt-one")],
        policy,
        controls: controls(),
    })
    .unwrap();
    assert_eq!(decision.policy_id, "selection-v1");
    assert_eq!(decision.decision_blake3.len(), BLAKE3_HEX_CHARS);
    decision
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
        reserved_units: MEDIUM_CHARGE_UNITS,
        project_limit_units: QUOTA_UNITS,
        account_limit_units: QUOTA_UNITS,
        policy_id: "accounting-v1".into(),
        schedule: schedule(),
    }
}

fn reconciliation_request(attempt_id: &str) -> UsageReconciliationRequest {
    UsageReconciliationRequest {
        attempt_id: attempt_id.into(),
        kind: UsageReconciliationKind::Completed,
        observed: Some(UsageAmounts {
            cpu_ms: DEFAULT_UNIT_SCHEDULE_DIVISOR,
            memory_byte_ms: DEFAULT_UNIT_SCHEDULE_DIVISOR,
            transfer_bytes: DEFAULT_UNIT_SCHEDULE_DIVISOR,
        }),
        policy_id: "accounting-v1".into(),
        schedule: schedule(),
    }
}

#[derive(Default)]
struct MemoryLedger {
    state: UsageLedgerState,
    commits: u32,
    fail_load: bool,
    fail_commit: bool,
}

impl UsageLedgerPort for MemoryLedger {
    fn load_usage_ledger(&mut self) -> Result<UsageLedgerState, ResourcePolicyPortError> {
        if self.fail_load {
            return Err(ResourcePolicyPortError::new(ResourcePolicyCapability::UsageLedger, "load-failed", true));
        }
        Ok(self.state.clone())
    }

    fn compare_and_commit_usage_ledger(
        &mut self,
        expected_state_blake3: &str,
        next_state: &UsageLedgerState,
    ) -> Result<(), ResourcePolicyPortError> {
        if self.fail_commit {
            return Err(ResourcePolicyPortError::new(ResourcePolicyCapability::UsageLedger, "commit-failed", true));
        }
        let current = usage_ledger_identity(&self.state).unwrap();
        if current != expected_state_blake3 {
            return Err(ResourcePolicyPortError::new(ResourcePolicyCapability::UsageLedger, "compare-failed", false));
        }
        self.state = next_state.clone();
        self.commits = self.commits.checked_add(1).expect("fixture commit count fits");
        Ok(())
    }
}

#[test]
fn application_commits_add_once_and_reuses_without_second_mutation() {
    let mut ledger = MemoryLedger::default();
    let first = reserve_usage(&mut ledger, reservation_request("attempt-one")).unwrap();
    assert!(first.durable);
    assert_eq!(ledger.commits, 1);
    let replay = reserve_usage(&mut ledger, reservation_request("attempt-one")).unwrap();
    assert_eq!(replay.plan.disposition, LedgerMutationDisposition::Reuse);
    assert_eq!(ledger.commits, 1);

    let reconciled = reconcile_usage(&mut ledger, reconciliation_request("attempt-one")).unwrap();
    assert!(reconciled.durable);
    assert_eq!(ledger.commits, EXPECTED_LEDGER_COMMITS_AFTER_RECONCILIATION);
    let duplicate = reconcile_usage(&mut ledger, reconciliation_request("attempt-one")).unwrap();
    assert_eq!(duplicate.plan.disposition, LedgerMutationDisposition::Reuse);
    assert_eq!(ledger.commits, EXPECTED_LEDGER_COMMITS_AFTER_RECONCILIATION);
}

#[test]
fn application_store_failure_never_grants_quota_or_mutates_loaded_state() {
    let mut ledger = MemoryLedger {
        fail_commit: true,
        ..MemoryLedger::default()
    };
    let before = usage_ledger_identity(&ledger.state).unwrap();
    let failure = reserve_usage(&mut ledger, reservation_request("attempt-one")).unwrap_err();
    assert!(matches!(failure, ResourcePolicyFailure::Port {
        quota_granted: false,
        ..
    }));
    assert_eq!(usage_ledger_identity(&ledger.state).unwrap(), before);
    assert_eq!(ledger.commits, 0);
}

fn valence_non_claims() -> Vec<String> {
    vec![VALENCE_REQUIRED_NON_CLAIM.into()]
}

fn service_request(id: &str, byte: char) -> valence::ServiceRequestRecord {
    valence::ServiceRequestRecord {
        request_id: id.into(),
        concrete_request_hash: digest(byte),
        idempotency_key: format!("idem-{id}"),
        role: valence::BuildServiceRole::Linked,
        non_claims: valence_non_claims(),
    }
}

fn attempt(
    id: &str,
    request_identity: String,
    generation: u32,
    parent_attempt_identity: Option<String>,
) -> valence::AttemptRecord {
    valence::AttemptRecord {
        attempt_id: id.into(),
        request_identity,
        generation,
        parent_attempt_identity,
        service_policy: "mantle-service-policy.v1".into(),
        role: valence::BuildServiceRole::Linked,
        non_claims: valence_non_claims(),
    }
}

fn retry_plan() -> ResourceRetryPlan {
    plan_resource_retry(ResourceRetryRequest {
        predecessor_attempt_id: "attempt-one".into(),
        predecessor_fence_generation: u64::from(REQUEST_GENERATION),
        current_machine_class_id: "small".into(),
        oom_evidence: observation("attempt-one").oom_evidence,
        retry_history: vec![RetryHistoryEntry {
            attempt_id: "attempt-one".into(),
            machine_class_id: "small".into(),
            charge_units: SMALL_CHARGE_UNITS,
            wall_time_ms: OBSERVATION_EXECUTION_MS,
        }],
        eligible_classes: classes(),
        quota: QuotaFacts {
            project_remaining_units: QUOTA_UNITS,
            account_remaining_units: QUOTA_UNITS,
        },
        policy: ResourceRetryPolicy::default(),
        controls: controls(),
    })
    .unwrap()
}

fn valence_context() -> (ValenceResourceContext, String, String, String) {
    let producer_request = service_request("request-one", 'b');
    let consumer_request = service_request("request-two", 'c');
    let producer_request_identity = valence::service_request_identity(&producer_request);
    let consumer_request_identity = valence::service_request_identity(&consumer_request);
    let predecessor = attempt("attempt-one", producer_request_identity.clone(), REQUEST_GENERATION, None);
    let predecessor_identity = valence::attempt_identity(&predecessor);
    let successor =
        attempt("attempt-two", producer_request_identity.clone(), RETRY_GENERATION, Some(predecessor_identity));
    let successor_identity = valence::attempt_identity(&successor);
    let authority = valence::AuthorityDecisionRecord {
        decision_id: "reuse-authority".into(),
        attempt_identity: successor_identity.clone(),
        subject_projection: "basalt-public-subject:consumer".into(),
        policy_identity: digest('d'),
        operation: "build.reuse".into(),
        decision: "allow".into(),
        verifier_evidence_ref: Some("basalt-verification:resource-policy".into()),
        role: valence::BuildServiceRole::Verified,
        non_claims: valence_non_claims(),
    };
    let authority_identity = valence::authority_decision_identity(&authority);
    let result = valence::BuildResultRecord {
        result_id: "result-one".into(),
        request_identity: producer_request_identity.clone(),
        attempt_identity: successor_identity,
        platform: "x86_64-linux".into(),
        policy: "mantle-build-policy.v1".into(),
        outputs: vec![valence::BuildOutputRef {
            path: "out/bin/tool".into(),
            content_hash: digest('e'),
        }],
        producer_signature_ref: "mantle-signer:resource-policy".into(),
        cas_observation: "cas:present".into(),
        role: valence::BuildServiceRole::Signed,
        non_claims: valence_non_claims(),
    };
    let result_identity = valence::build_result_identity(&result);
    assert_ne!(producer_request_identity, consumer_request_identity);
    assert_ne!(result_identity, consumer_request_identity);
    (
        ValenceResourceContext {
            producer_policy_blake3: digest('f'),
            service_requests: vec![producer_request, consumer_request],
            attempts: vec![predecessor, successor],
            transitions: Vec::new(),
            authority_decisions: vec![authority],
            build_results: vec![result],
            orchestration_jobs: Vec::new(),
            external_statuses: Vec::new(),
        },
        result_identity,
        consumer_request_identity,
        authority_identity,
    )
}

#[allow(
    tigerstyle::ambiguous_params,
    reason = "Valence result, request, and authority identities are distinct typed roles"
)]
fn sharing_decision(
    result_identity: String,
    consumer_request_identity: String,
    authority_identity: String,
) -> ResultReuseDecision {
    decide_result_reuse(
        ResultSharingPolicy {
            schema: RESULT_SHARING_POLICY_SCHEMA.into(),
            policy_id: "sharing-v1".into(),
            scope: SharingScopeKind::Named,
            project_id: None,
            allowed_producers: vec!["project-a".into()],
            allowed_consumers: vec!["project-b".into()],
        },
        ResultReuseFacts {
            producer_project_id: "project-a".into(),
            consumer_project_id: "project-b".into(),
            producer_result_blake3: result_identity,
            consumer_request_blake3: consumer_request_identity,
            request_identity_matches: true,
            action_identity_matches: true,
            producer_signature_trusted: true,
            policy_compatible: true,
            platform_compatible: true,
            output_identity_verified: true,
            cas_available: true,
            strong_reuse_admitted: true,
            authorization_decision_blake3: authority_identity,
        },
        controls(),
    )
    .unwrap()
}

fn usage_evidence() -> ValenceUsageInput {
    let reserved = plan_usage_reservation(UsageLedgerState::default(), reservation_request("attempt-two")).unwrap();
    let reservation = reserved.plan_reservation();
    let reconciled = plan_usage_reconciliation(reserved.next_state, reconciliation_request("attempt-two")).unwrap();
    ValenceUsageInput {
        reservation,
        reconciliation: reconciled.reconciliation.unwrap(),
    }
}

trait PlanReservation {
    fn plan_reservation(&self) -> UsageReservation;
}

impl PlanReservation for UsageLedgerMutationPlan {
    fn plan_reservation(&self) -> UsageReservation {
        self.reservation.clone().unwrap()
    }
}

#[test]
fn valence_projection_links_selection_retry_usage_and_reuse_without_overclaim() {
    let (context, result_identity, consumer_identity, authority_identity) = valence_context();
    let decision = sharing_decision(result_identity.clone(), consumer_identity.clone(), authority_identity.clone());
    let projection = ValenceResourceProjection {
        decision_attempt_id: "attempt-two".into(),
        retry_successor_attempt_id: Some("attempt-two".into()),
        machine_classes: classes(),
        selection: selection(),
        observations: vec![observation("attempt-one")],
        retry: Some(retry_plan()),
        usage: vec![usage_evidence()],
        reuse: Some(ValenceReuseInput {
            decision,
            sharing_policy_id: "sharing-v1".into(),
            producer_result_identity: result_identity,
            consumer_request_identity: consumer_identity,
            authorization_decision_identity: Some(authority_identity),
            output_check_ref: Some("output-check:verified".into()),
            signature_ref: Some("mantle-signer:resource-policy".into()),
            cas_observation: Some("cas:present".into()),
        }),
    };
    let bundle = build_valence_resource_bundle(context, projection).unwrap();
    let valence_validation = valence::validate_build_service_bundle(&bundle);
    assert!(valence_validation.valid, "Valence issues: {:?}", valence_validation.issues);
    assert!(valence_validation.issues.is_empty());
    assert_eq!(bundle.resource_decisions.len(), 1);
    assert_eq!(bundle.retry_edges.len(), 1);
    assert_eq!(bundle.usage_records.len(), EXPECTED_USAGE_RECORD_COUNT);
    assert_eq!(bundle.reuse_edges.len(), 1);
}

#[derive(Default)]
struct EvidenceSink {
    published: u32,
    fail: bool,
}

impl ResourcePolicyEvidencePort for EvidenceSink {
    fn publish_valence_bundle(&mut self, _bundle: &valence::BuildServiceBundle) -> Result<(), ResourcePolicyPortError> {
        if self.fail {
            return Err(ResourcePolicyPortError::new(
                ResourcePolicyCapability::EvidencePublication,
                "publication-failed",
                true,
            ));
        }
        self.published = self.published.checked_add(1).expect("fixture publication count fits");
        Ok(())
    }
}

#[test]
fn valence_publication_validates_before_effect_and_reports_port_failure() {
    let (context, result_identity, consumer_identity, authority_identity) = valence_context();
    let projection = ValenceResourceProjection {
        decision_attempt_id: "attempt-two".into(),
        retry_successor_attempt_id: Some("attempt-two".into()),
        machine_classes: classes(),
        selection: selection(),
        observations: vec![observation("attempt-one")],
        retry: Some(retry_plan()),
        usage: vec![usage_evidence()],
        reuse: Some(ValenceReuseInput {
            decision: sharing_decision(result_identity.clone(), consumer_identity.clone(), authority_identity.clone()),
            sharing_policy_id: "sharing-v1".into(),
            producer_result_identity: result_identity,
            consumer_request_identity: consumer_identity,
            authorization_decision_identity: Some(authority_identity),
            output_check_ref: Some("output-check:verified".into()),
            signature_ref: Some("mantle-signer:resource-policy".into()),
            cas_observation: Some("cas:present".into()),
        }),
    };
    let bundle = build_valence_resource_bundle(context, projection).unwrap();
    let mut sink = EvidenceSink::default();
    let valence_validation = publish_valence_evidence(&mut sink, bundle.clone()).unwrap();
    assert!(valence_validation.valid);
    assert_eq!(sink.published, 1);

    sink.fail = true;
    assert!(matches!(
        publish_valence_evidence(&mut sink, bundle),
        Err(ResourcePolicyFailure::Port {
            quota_granted: false,
            ..
        })
    ));
    assert_eq!(sink.published, 1);
}

#[test]
fn valence_projection_rejects_missing_attempt_linkage() {
    let (context, _, _, _) = valence_context();
    let projection = ValenceResourceProjection {
        decision_attempt_id: "missing-attempt".into(),
        retry_successor_attempt_id: None,
        machine_classes: classes(),
        selection: selection(),
        observations: vec![observation("attempt-one")],
        retry: None,
        usage: Vec::new(),
        reuse: None,
    };
    assert!(build_valence_resource_bundle(context, projection).is_err());
}

#[test]
fn type_shapes_do_not_expose_ambient_maps_or_secret_fields() {
    let serialized = serde_json::to_string(&selection()).unwrap();
    assert!(!serialized.contains("environment"));
    assert!(!serialized.contains("token"));
    assert!(!serialized.contains("raw_log"));
    assert!(!serialized.contains("credential"));
}
