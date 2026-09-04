use std::collections::BTreeMap;

use crunch_resource_policy_core::MachineClass;
use crunch_resource_policy_core::ResourceObservation;
use crunch_resource_policy_core::ResourcePolicyError;
use crunch_resource_policy_core::ResourceRetryPlan;
use crunch_resource_policy_core::ResourceSelectionDecision;
use crunch_resource_policy_core::ResultReuseDecision;
use crunch_resource_policy_core::TerminalOutcome;
use crunch_resource_policy_core::UsageReconciliation;
use crunch_resource_policy_core::UsageReservation;
use valence_core::build_service as valence;

const VALENCE_CANONICALIZATION: &str = "valence-canonical-json-v1";
const VALENCE_PROFILE_VERSION: u32 = 1;
const RESOURCE_OBSERVATION_MEASUREMENT_COUNT: usize = 6;
const MAX_CLAIMED_OBSERVATION_CLASSES: usize = 5;
const USAGE_RECORDS_PER_INPUT: usize = 2;

#[derive(Clone, Copy, Debug)]
struct ClaimedClassFacts {
    has_build: bool,
    has_retry: bool,
    has_usage: bool,
    has_reuse: bool,
    has_external_status: bool,
}

#[derive(Clone, Debug)]
pub struct ValenceResourceContext {
    pub producer_policy_blake3: String,
    pub service_requests: Vec<valence::ServiceRequestRecord>,
    pub attempts: Vec<valence::AttemptRecord>,
    pub transitions: Vec<valence::AttemptTransitionRecord>,
    pub authority_decisions: Vec<valence::AuthorityDecisionRecord>,
    pub build_results: Vec<valence::BuildResultRecord>,
    pub orchestration_jobs: Vec<valence::OrchestrationRecord>,
    pub external_statuses: Vec<valence::ExternalStatusRecord>,
}

#[derive(Clone, Debug)]
pub struct ValenceUsageInput {
    pub reservation: UsageReservation,
    pub reconciliation: UsageReconciliation,
}

#[derive(Clone, Debug)]
pub struct ValenceReuseInput {
    pub decision: ResultReuseDecision,
    pub sharing_policy_id: String,
    pub producer_result_identity: String,
    pub consumer_request_identity: String,
    pub authorization_decision_identity: Option<String>,
    pub output_check_ref: Option<String>,
    pub signature_ref: Option<String>,
    pub cas_observation: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ValenceResourceProjection {
    pub decision_attempt_id: String,
    pub retry_successor_attempt_id: Option<String>,
    pub machine_classes: Vec<MachineClass>,
    pub selection: ResourceSelectionDecision,
    pub observations: Vec<ResourceObservation>,
    pub retry: Option<ResourceRetryPlan>,
    pub usage: Vec<ValenceUsageInput>,
    pub reuse: Option<ValenceReuseInput>,
}

pub fn build_valence_resource_bundle(
    context: ValenceResourceContext,
    projection: ValenceResourceProjection,
) -> Result<valence::BuildServiceBundle, crate::ResourcePolicyFailure> {
    validate_projection(&context, &projection)?;
    let attempt_identities = attempt_identities(&context.attempts);
    let machine_records = valence_machine_classes(&projection.machine_classes);
    let class_identities = machine_class_identities(&machine_records);
    let resource_observations = valence_resource_observations(&projection.observations, &attempt_identities)?;
    let resource_decisions = vec![valence_resource_decision(
        &projection,
        &attempt_identities,
        &class_identities,
        &resource_observations,
    )?];
    let retry_edges = valence_retry_edges(&projection, &attempt_identities, &class_identities)?;
    let usage_records = valence_usage_records(&projection.usage, &attempt_identities)?;
    let reuse_edges = valence_reuse_edges(&projection.reuse)?;
    let claimed_observation_classes = claimed_classes(ClaimedClassFacts {
        has_build: !context.build_results.is_empty(),
        has_retry: !retry_edges.is_empty(),
        has_usage: !usage_records.is_empty(),
        has_reuse: !reuse_edges.is_empty(),
        has_external_status: !context.external_statuses.is_empty(),
    });
    let bundle = valence::BuildServiceBundle {
        profile: valence::BuildServiceEvidenceProfile {
            profile_id: crunch_resource_policy_core::VALENCE_BUILD_SERVICE_PROFILE_ID.to_string(),
            profile_version: VALENCE_PROFILE_VERSION,
            canonicalization: VALENCE_CANONICALIZATION.to_string(),
            producer_policy: context.producer_policy_blake3,
            claimed_observation_classes,
            non_claims: valence_non_claims(),
        },
        service_requests: context.service_requests,
        attempts: context.attempts,
        transitions: context.transitions,
        authority_decisions: context.authority_decisions,
        machine_classes: machine_records,
        resource_decisions,
        resource_observations,
        retry_edges,
        usage_records,
        build_results: context.build_results,
        reuse_edges,
        orchestration_jobs: context.orchestration_jobs,
        external_statuses: context.external_statuses,
    };
    let valence_validation = valence::validate_build_service_bundle(&bundle);
    if !valence_validation.valid {
        let issue_codes =
            valence_validation.issues.iter().map(|issue| format!("{}:{}", issue.field, issue.code)).collect();
        return Err(crate::ResourcePolicyFailure::Valence { issue_codes });
    }
    debug_assert!(valence_validation.valid);
    debug_assert!(valence_validation.issues.is_empty());
    Ok(bundle)
}

fn validate_projection(
    context: &ValenceResourceContext,
    projection: &ValenceResourceProjection,
) -> Result<(), crate::ResourcePolicyFailure> {
    crunch_resource_policy_core::validate_resource_selection_decision(projection.selection.clone())?;
    for observation in &projection.observations {
        crunch_resource_policy_core::validate_resource_observation(observation.clone())?;
    }
    if let Some(retry) = &projection.retry {
        crunch_resource_policy_core::validate_resource_retry_plan(retry.clone())?;
        if projection.retry_successor_attempt_id.is_none() {
            return Err(ResourcePolicyError::InvalidIdentity.into());
        }
    } else if projection.retry_successor_attempt_id.is_some() {
        return Err(ResourcePolicyError::InvalidIdentity.into());
    }
    if context.service_requests.is_empty() || context.attempts.is_empty() {
        return Err(ResourcePolicyError::InvalidBounds.into());
    }
    if context.build_results.is_empty() || projection.machine_classes.is_empty() {
        return Err(ResourcePolicyError::InvalidBounds.into());
    }
    debug_assert!(!projection.decision_attempt_id.is_empty());
    debug_assert!(!projection.machine_classes.is_empty());
    Ok(())
}

fn attempt_identities(attempts: &[valence::AttemptRecord]) -> BTreeMap<String, String> {
    let identities = attempts
        .iter()
        .map(|attempt| (attempt.attempt_id.clone(), valence::attempt_identity(attempt)))
        .collect::<BTreeMap<_, _>>();
    debug_assert!(identities.len() <= attempts.len());
    debug_assert!(attempts.is_empty() || !identities.is_empty());
    identities
}

fn valence_machine_classes(classes: &[MachineClass]) -> Vec<valence::MachineClassRecord> {
    let records = classes
        .iter()
        .map(|class| valence::MachineClassRecord {
            class_id: class.class_id.clone(),
            platform: class.platform.clone(),
            resource_class: class.class_id.clone(),
            ordinal: class.ordinal,
            role: valence::BuildServiceRole::Linked,
            non_claims: valence_non_claims(),
        })
        .collect::<Vec<_>>();
    debug_assert_eq!(records.len(), classes.len());
    debug_assert!(records.iter().all(|record| record.ordinal > 0));
    records
}

fn machine_class_identities(records: &[valence::MachineClassRecord]) -> BTreeMap<String, String> {
    let identities = records
        .iter()
        .map(|record| (record.class_id.clone(), valence::machine_class_identity(record)))
        .collect::<BTreeMap<_, _>>();
    debug_assert!(identities.len() <= records.len());
    debug_assert!(records.is_empty() || !identities.is_empty());
    identities
}

fn valence_resource_decision(
    projection: &ValenceResourceProjection,
    attempt_identities: &BTreeMap<String, String>,
    class_identities: &BTreeMap<String, String>,
    observation_records: &[valence::ResourceObservationRecord],
) -> Result<valence::ResourceDecisionRecord, crate::ResourcePolicyFailure> {
    let attempt_identity = required_map_value(attempt_identities, &projection.decision_attempt_id)?;
    let selected_class_identity = required_map_value(class_identities, &projection.selection.scheduled_class_id)?;
    let eligible_class_identities = projection
        .selection
        .eligible_class_ids
        .iter()
        .map(|class_id| required_map_value(class_identities, class_id))
        .collect::<Result<Vec<_>, _>>()?;
    let declared = projection.selection.declared_minima;
    let record = valence::ResourceDecisionRecord {
        decision_id: projection.selection.decision_blake3.clone(),
        attempt_identity,
        declared_minima: vec![
            format!("cpu-units>={}", declared.cpu_units),
            format!("memory-bytes>={}", declared.memory_bytes),
            format!("scratch-bytes>={}", declared.scratch_bytes),
        ],
        eligible_class_identities,
        selected_class_identity,
        policy: projection.selection.policy_id.clone(),
        compatible_observation_identities: observation_records
            .iter()
            .map(valence::resource_observation_identity)
            .collect(),
        reasons: projection.selection.reason_codes.iter().map(|reason| reason.as_str().to_string()).collect(),
        role: valence::BuildServiceRole::Verified,
        non_claims: valence_non_claims(),
    };
    debug_assert!(!record.eligible_class_identities.is_empty());
    debug_assert!(!record.selected_class_identity.is_empty());
    Ok(record)
}

fn valence_resource_observations(
    observations: &[ResourceObservation],
    attempt_identities: &BTreeMap<String, String>,
) -> Result<Vec<valence::ResourceObservationRecord>, crate::ResourcePolicyFailure> {
    let records = observations
        .iter()
        .map(|observation| {
            let attempt_identity = required_map_value(attempt_identities, &observation.attempt_id)?;
            Ok(valence::ResourceObservationRecord {
                observation_id: observation.observation_blake3.clone(),
                attempt_identity,
                measurements: valence_measurements(observation),
                collector: format!("{}@{}", observation.collector_id, observation.collector_version),
                compatibility_scope: observation.compatibility_policy_id.clone(),
                terminal_outcome: terminal_outcome(observation.terminal_outcome).to_string(),
                role: valence::BuildServiceRole::RecordedOnly,
                non_claims: valence_non_claims(),
            })
        })
        .collect::<Result<Vec<_>, crate::ResourcePolicyFailure>>()?;
    debug_assert_eq!(records.len(), observations.len());
    debug_assert!(records.iter().all(|record| record.measurements.len() == RESOURCE_OBSERVATION_MEASUREMENT_COUNT));
    Ok(records)
}

fn valence_measurements(observation: &ResourceObservation) -> Vec<valence::ResourceMeasurement> {
    let measurements = observation.measurements;
    vec![
        measurement("cpu-time", measurements.cpu_time_ms, "ms"),
        measurement("peak-memory", measurements.peak_memory_bytes, "bytes"),
        measurement("scratch-peak", measurements.scratch_peak_bytes, "bytes"),
        measurement("io", measurements.io_bytes, "bytes"),
        measurement("transfer", measurements.transfer_bytes, "bytes"),
        measurement("wall-time", measurements.wall_time_ms, "ms"),
    ]
}

fn measurement(name: &str, value: u64, unit: &str) -> valence::ResourceMeasurement {
    valence::ResourceMeasurement {
        name: name.to_string(),
        value,
        unit: unit.to_string(),
    }
}

fn valence_retry_edges(
    projection: &ValenceResourceProjection,
    attempt_identities: &BTreeMap<String, String>,
    class_identities: &BTreeMap<String, String>,
) -> Result<Vec<valence::RetryEdgeRecord>, crate::ResourcePolicyFailure> {
    let Some(retry) = &projection.retry else {
        return Ok(Vec::new());
    };
    let successor_id = projection.retry_successor_attempt_id.as_ref().ok_or(ResourcePolicyError::InvalidIdentity)?;
    let record = valence::RetryEdgeRecord {
        predecessor_attempt_identity: required_map_value(attempt_identities, &retry.predecessor_attempt_id)?,
        successor_attempt_identity: required_map_value(attempt_identities, successor_id)?,
        trigger: "oom_escalation".to_string(),
        trigger_evidence: retry.trigger_evidence_ref.clone(),
        prior_class_identity: required_map_value(class_identities, &retry.prior_machine_class_id)?,
        next_class_identity: required_map_value(class_identities, &retry.next_machine_class_id)?,
        limit_state: "within-policy-limits".to_string(),
        policy: projection.selection.policy_id.clone(),
        role: valence::BuildServiceRole::Linked,
        non_claims: valence_non_claims(),
    };
    debug_assert_ne!(record.predecessor_attempt_identity, record.successor_attempt_identity);
    debug_assert_ne!(record.prior_class_identity, record.next_class_identity);
    Ok(vec![record])
}

fn valence_usage_records(
    usage: &[ValenceUsageInput],
    attempt_identities: &BTreeMap<String, String>,
) -> Result<Vec<valence::UsageRecord>, crate::ResourcePolicyFailure> {
    let mut records = Vec::with_capacity(usage.len().saturating_mul(USAGE_RECORDS_PER_INPUT));
    for input in usage {
        let attempt_identity = required_map_value(attempt_identities, &input.reservation.attempt_id)?;
        records.push(valence::UsageRecord {
            usage_id: input.reservation.reservation_blake3.clone(),
            attempt_identity: attempt_identity.clone(),
            usage_kind: "reservation".to_string(),
            unit_schedule: input.reservation.schedule_id.clone(),
            window: format!("{}/{}", input.reservation.window_start_unix_s, input.reservation.window_end_unix_s),
            scope: format!("project:{}", input.reservation.project_id),
            outcome: "reserved".to_string(),
            role: valence::BuildServiceRole::RecordedOnly,
            non_claims: valence_non_claims(),
        });
        records.push(valence::UsageRecord {
            usage_id: input.reconciliation.reconciliation_blake3.clone(),
            attempt_identity,
            usage_kind: "reconciliation".to_string(),
            unit_schedule: input.reconciliation.schedule_id.clone(),
            window: format!("{}/{}", input.reservation.window_start_unix_s, input.reservation.window_end_unix_s),
            scope: format!("project:{}", input.reservation.project_id),
            outcome: input.reconciliation.reason_code.clone(),
            role: valence::BuildServiceRole::RecordedOnly,
            non_claims: valence_non_claims(),
        });
    }
    debug_assert_eq!(records.len(), usage.len().saturating_mul(USAGE_RECORDS_PER_INPUT));
    debug_assert!(records.iter().all(|record| !record.attempt_identity.is_empty()));
    Ok(records)
}

fn valence_reuse_edges(
    reuse: &Option<ValenceReuseInput>,
) -> Result<Vec<valence::ReuseEdgeRecord>, crate::ResourcePolicyFailure> {
    let Some(reuse) = reuse else {
        return Ok(Vec::new());
    };
    crunch_resource_policy_core::validate_result_reuse_decision(reuse.decision.clone())?;
    let record = valence::ReuseEdgeRecord {
        producer_result_identity: reuse.producer_result_identity.clone(),
        consumer_request_identity: reuse.consumer_request_identity.clone(),
        sharing_policy: reuse.sharing_policy_id.clone(),
        authorization_decision_identity: reuse.authorization_decision_identity.clone(),
        request_compatible: reuse.decision.usable,
        platform_compatible: reuse.decision.usable,
        output_check_ref: reuse.output_check_ref.clone(),
        signature_ref: reuse.signature_ref.clone(),
        cas_observation: reuse.cas_observation.clone(),
        usability_outcome: if reuse.decision.usable { "usable" } else { "denied" }.to_string(),
        role: valence::BuildServiceRole::Verified,
        non_claims: valence_non_claims(),
    };
    debug_assert_eq!(record.usability_outcome == "usable", reuse.decision.usable);
    debug_assert!(!record.sharing_policy.is_empty());
    Ok(vec![record])
}

fn required_map_value(values: &BTreeMap<String, String>, key: &str) -> Result<String, crate::ResourcePolicyFailure> {
    let value = values.get(key).cloned().ok_or(ResourcePolicyError::InvalidIdentity)?;
    debug_assert!(!key.is_empty());
    debug_assert!(!value.is_empty());
    Ok(value)
}

fn claimed_classes(facts: ClaimedClassFacts) -> Vec<String> {
    let mut classes = Vec::with_capacity(MAX_CLAIMED_OBSERVATION_CLASSES);
    for (is_present, class) in [
        (facts.has_build, "build"),
        (facts.has_retry, "retry"),
        (facts.has_reuse, "reuse"),
        (facts.has_usage, "usage"),
        (facts.has_external_status, "external-status"),
    ] {
        if is_present {
            classes.push(class.to_string());
        }
    }
    debug_assert!(!classes.is_empty());
    debug_assert!(classes.len() <= MAX_CLAIMED_OBSERVATION_CLASSES);
    classes
}

fn terminal_outcome(outcome: TerminalOutcome) -> &'static str {
    match outcome {
        TerminalOutcome::Succeeded => "succeeded",
        TerminalOutcome::FailedOom => "failed-oom",
        TerminalOutcome::FailedOther => "failed-other",
        TerminalOutcome::Cancelled => "cancelled",
        TerminalOutcome::WorkerLost => "worker-lost",
    }
}

fn valence_non_claims() -> Vec<String> {
    vec![crunch_resource_policy_core::VALENCE_REQUIRED_NON_CLAIM.to_string()]
}
