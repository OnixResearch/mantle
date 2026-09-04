use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::HistoryDecision;
use crate::HistoryReasonCode;
use crate::MAX_MEASUREMENT_MILLISECONDS;
use crate::MAX_OBSERVATION_AGE_SECS;
use crate::MAX_OBSERVATIONS;
use crate::MAX_RESOURCE_BYTES;
use crate::OBSERVATION_DOMAIN;
use crate::ObservationFilterReport;
use crate::ObservationFilterRequest;
use crate::RESOURCE_OBSERVATION_SCHEMA;
use crate::RESOURCE_POLICY_NON_CLAIM;
use crate::ResourceObservation;
use crate::ResourceObservationInput;
use crate::ResourcePolicyError;
use crate::ResourceQuantities;
use crate::bounded_count;
use crate::canonical_blake3;
use crate::canonical_strings;
use crate::validate_digest;
use crate::validate_id;

pub fn build_resource_observation(input: ResourceObservationInput) -> Result<ResourceObservation, ResourcePolicyError> {
    validate_observation_input(&input)?;
    let observation_blake3 = canonical_blake3(OBSERVATION_DOMAIN, &input)?;
    let observation = ResourceObservation {
        schema: RESOURCE_OBSERVATION_SCHEMA.to_string(),
        observation_blake3,
        attempt_id: input.attempt_id,
        action_family_blake3: input.action_family_blake3,
        platform_identity: input.platform_identity,
        machine_class_id: input.machine_class_id,
        declared: input.declared,
        selected: input.selected,
        timing: input.timing,
        measurements: input.measurements,
        oom_evidence: input.oom_evidence,
        terminal_outcome: input.terminal_outcome,
        retry_predecessor_attempt_id: input.retry_predecessor_attempt_id,
        collector_id: input.collector_id,
        collector_version: input.collector_version,
        compatibility_policy_id: input.compatibility_policy_id,
        observed_at_unix_s: input.observed_at_unix_s,
        trusted: input.trusted,
        non_claim: RESOURCE_POLICY_NON_CLAIM.to_string(),
    };
    debug_assert_eq!(observation.schema, RESOURCE_OBSERVATION_SCHEMA);
    debug_assert_eq!(observation.non_claim, RESOURCE_POLICY_NON_CLAIM);
    Ok(observation)
}

pub fn validate_resource_observation(
    observation: ResourceObservation,
) -> Result<ResourceObservation, ResourcePolicyError> {
    if observation.schema != RESOURCE_OBSERVATION_SCHEMA || observation.non_claim != RESOURCE_POLICY_NON_CLAIM {
        return Err(ResourcePolicyError::InvalidSchema);
    }
    let rebuilt = build_resource_observation(ResourceObservationInput {
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
    })?;
    if observation != rebuilt {
        return Err(ResourcePolicyError::InvalidDigest);
    }
    debug_assert_eq!(observation.observation_blake3, rebuilt.observation_blake3);
    debug_assert_eq!(observation.schema, RESOURCE_OBSERVATION_SCHEMA);
    Ok(observation)
}

pub fn filter_resource_observations(
    mut request: ObservationFilterRequest,
) -> Result<ObservationFilterReport, ResourcePolicyError> {
    validate_filter_request(&request)?;
    request.known_machine_class_ids = canonical_strings(request.known_machine_class_ids, crate::MAX_MACHINE_CLASSES)?;
    request.observations.sort_by(|left, right| left.observation_blake3.cmp(&right.observation_blake3));
    let known = request.known_machine_class_ids.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let mut seen = BTreeSet::new();
    let mut accepted = Vec::with_capacity(request.observations.len());
    let mut decisions = Vec::with_capacity(request.observations.len().max(1));
    if request.observations.is_empty() {
        decisions.push(HistoryDecision {
            observation_blake3: None,
            accepted: false,
            reason: HistoryReasonCode::Missing,
        });
    }
    let observations = core::mem::take(&mut request.observations);
    for observation in observations {
        let reason = classify_observation(&request, &known, &mut seen, &observation);
        let is_accepted = reason == HistoryReasonCode::Accepted;
        decisions.push(HistoryDecision {
            observation_blake3: Some(observation.observation_blake3.clone()),
            accepted: is_accepted,
            reason,
        });
        if is_accepted {
            accepted.push(observation);
        }
    }
    debug_assert!(u32::try_from(accepted.len()).is_ok_and(|count| count <= MAX_OBSERVATIONS));
    debug_assert_eq!(
        decisions.len(),
        accepted.len().saturating_add(decisions.iter().filter(|row| !row.accepted).count())
    );
    Ok(ObservationFilterReport { accepted, decisions })
}

fn validate_observation_input(input: &ResourceObservationInput) -> Result<(), ResourcePolicyError> {
    validate_id(&input.attempt_id)?;
    validate_digest(&input.action_family_blake3)?;
    validate_id(&input.platform_identity)?;
    validate_id(&input.machine_class_id)?;
    validate_quantities(input.declared)?;
    validate_quantities(input.selected)?;
    validate_measurements(input)?;
    validate_id(&input.collector_id)?;
    validate_id(&input.collector_version)?;
    validate_id(&input.compatibility_policy_id)?;
    if let Some(predecessor) = &input.retry_predecessor_attempt_id {
        validate_id(predecessor)?;
    }
    validate_oom_evidence(input)?;
    debug_assert!(!input.attempt_id.is_empty());
    debug_assert!(input.selected.memory_bytes > 0);
    Ok(())
}

fn validate_quantities(quantities: ResourceQuantities) -> Result<(), ResourcePolicyError> {
    if quantities.cpu_units == 0 || quantities.cpu_units > crate::MAX_CPU_UNITS {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    if quantities.memory_bytes == 0 || quantities.memory_bytes > MAX_RESOURCE_BYTES {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    if quantities.scratch_bytes == 0 || quantities.scratch_bytes > MAX_RESOURCE_BYTES {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    debug_assert!(quantities.cpu_units > 0);
    debug_assert!(quantities.memory_bytes <= MAX_RESOURCE_BYTES);
    Ok(())
}

fn validate_measurements(input: &ResourceObservationInput) -> Result<(), ResourcePolicyError> {
    let timing = input.timing;
    let measurements = input.measurements;
    if timing.queue_ms > MAX_MEASUREMENT_MILLISECONDS
        || timing.execution_ms > MAX_MEASUREMENT_MILLISECONDS
        || timing.terminal_ms > MAX_MEASUREMENT_MILLISECONDS
    {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    if measurements.cpu_time_ms > MAX_MEASUREMENT_MILLISECONDS
        || measurements.wall_time_ms > MAX_MEASUREMENT_MILLISECONDS
    {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    if measurements.peak_memory_bytes > MAX_RESOURCE_BYTES || measurements.scratch_peak_bytes > MAX_RESOURCE_BYTES {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    if measurements.io_bytes > MAX_RESOURCE_BYTES || measurements.transfer_bytes > MAX_RESOURCE_BYTES {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    debug_assert!(measurements.peak_memory_bytes <= MAX_RESOURCE_BYTES);
    debug_assert!(measurements.wall_time_ms <= MAX_MEASUREMENT_MILLISECONDS);
    Ok(())
}

fn validate_oom_evidence(input: &ResourceObservationInput) -> Result<(), ResourcePolicyError> {
    validate_id(&input.oom_evidence.platform)?;
    if let Some(reference) = &input.oom_evidence.evidence_ref {
        validate_id(reference)?;
    }
    if input.oom_evidence.trusted && input.oom_evidence.evidence_ref.is_none() {
        return Err(ResourcePolicyError::InvalidIdentity);
    }
    debug_assert!(!input.oom_evidence.platform.is_empty());
    debug_assert!(!input.oom_evidence.trusted || input.oom_evidence.evidence_ref.is_some());
    Ok(())
}

fn validate_filter_request(request: &ObservationFilterRequest) -> Result<(), ResourcePolicyError> {
    validate_digest(&request.action_family_blake3)?;
    validate_id(&request.platform_identity)?;
    validate_id(&request.compatibility_policy_id)?;
    bounded_count(request.observations.len(), MAX_OBSERVATIONS, ResourcePolicyError::TooManyObservations)?;
    if request.observation_age_secs_max == 0 || request.observation_age_secs_max > MAX_OBSERVATION_AGE_SECS {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    if request.measurement_bytes_max == 0 || request.measurement_bytes_max > MAX_RESOURCE_BYTES {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    debug_assert!(request.observation_age_secs_max <= MAX_OBSERVATION_AGE_SECS);
    debug_assert!(request.measurement_bytes_max <= MAX_RESOURCE_BYTES);
    Ok(())
}

fn classify_observation(
    request: &ObservationFilterRequest,
    known: &BTreeSet<&str>,
    seen: &mut BTreeSet<String>,
    observation: &ResourceObservation,
) -> HistoryReasonCode {
    debug_assert!(request.observation_age_secs_max > 0);
    debug_assert!(u32::try_from(seen.len()).is_ok_and(|count| count <= MAX_OBSERVATIONS));
    if observation.schema != RESOURCE_OBSERVATION_SCHEMA {
        return HistoryReasonCode::SchemaUnsupported;
    }
    if validate_resource_observation(observation.clone()).is_err() {
        return HistoryReasonCode::DigestInvalid;
    }
    if !seen.insert(observation.observation_blake3.clone()) {
        return HistoryReasonCode::Duplicate;
    }
    if !observation.trusted {
        return HistoryReasonCode::Untrusted;
    }
    if observation.action_family_blake3 != request.action_family_blake3 {
        return HistoryReasonCode::ActionFamilyMismatch;
    }
    if observation.platform_identity != request.platform_identity {
        return HistoryReasonCode::PlatformMismatch;
    }
    if observation.compatibility_policy_id != request.compatibility_policy_id {
        return HistoryReasonCode::PolicyMismatch;
    }
    if !known.contains(observation.machine_class_id.as_str()) {
        return HistoryReasonCode::MachineClassUnknown;
    }
    if observation.observed_at_unix_s > request.now_unix_s {
        return HistoryReasonCode::FutureDated;
    }
    let age_secs = request.now_unix_s.saturating_sub(observation.observed_at_unix_s);
    if age_secs > request.observation_age_secs_max {
        return HistoryReasonCode::Stale;
    }
    let is_memory_outlier = observation.measurements.peak_memory_bytes > request.measurement_bytes_max
        || observation.measurements.scratch_peak_bytes > request.measurement_bytes_max;
    let is_io_outlier = observation.measurements.io_bytes > request.measurement_bytes_max
        || observation.measurements.transfer_bytes > request.measurement_bytes_max;
    if is_memory_outlier || is_io_outlier {
        return HistoryReasonCode::MeasurementOutlier;
    }
    HistoryReasonCode::Accepted
}
