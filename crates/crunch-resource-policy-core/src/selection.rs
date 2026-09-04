use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use crate::BASIS_POINTS_DENOMINATOR;
use crate::BASIS_POINTS_DENOMINATOR_U32;
use crate::DeclaredResourceRequirements;
use crate::MAX_CLASS_ENDPOINTS;
use crate::MAX_MACHINE_CLASSES;
use crate::MAX_MARGIN_BASIS_POINTS;
use crate::MAX_OBSERVATION_AGE_SECS;
use crate::MAX_OBSERVATIONS;
use crate::MAX_POLICY_VERSION_BYTES;
use crate::MAX_RESOURCE_BYTES;
use crate::MachineClass;
use crate::ObservationFilterReport;
use crate::ObservationFilterRequest;
use crate::RESOURCE_POLICY_NON_CLAIM;
use crate::RESOURCE_SELECTION_DECISION_SCHEMA;
use crate::RESOURCE_SELECTION_POLICY_SCHEMA;
use crate::ResourcePolicyError;
use crate::ResourcePolicyMode;
use crate::ResourceQuantities;
use crate::ResourceSelectionDecision;
use crate::ResourceSelectionPolicy;
use crate::ResourceSelectionRequest;
use crate::SELECTION_DOMAIN;
use crate::SelectionFallbackRule;
use crate::SelectionReasonCode;
use crate::bounded_count;
use crate::canonical_blake3;
use crate::canonical_strings;
use crate::filter_resource_observations;
use crate::validate_action_family_identity;
use crate::validate_digest;
use crate::validate_id;

pub fn select_resource_class(
    mut request: ResourceSelectionRequest,
) -> Result<ResourceSelectionDecision, ResourcePolicyError> {
    validate_selection_request(&request)?;
    request.action_family = validate_action_family_identity(request.action_family)?;
    request.machine_classes = canonical_machine_classes(request.machine_classes)?;
    let filter = filter_selection_history(&mut request)?;
    let static_eligible = eligible_classes(&request.declared, request.declared.minima, &request.machine_classes);
    let static_class = static_eligible.first().copied().ok_or(ResourcePolicyError::NoEligibleClass)?;
    let mut history = resolve_history(&request, static_class, &filter)?;
    let is_enforcement_active = history_enforcement_active(request.controls);
    history.policy_class = apply_quota_policy(
        &request,
        static_class,
        history.policy_class,
        is_enforcement_active,
        &mut history.reason_codes,
    )?;
    let scheduled_class = scheduled_class(&request, static_class, history.policy_class, &mut history.reason_codes);
    let eligible_class_ids = static_eligible.iter().map(|class| class.class_id.clone()).collect::<Vec<_>>();
    let compatible_observation_blake3s =
        filter.accepted.iter().map(|observation| observation.observation_blake3.clone()).collect::<Vec<_>>();
    let mut decision = ResourceSelectionDecision {
        schema: RESOURCE_SELECTION_DECISION_SCHEMA.to_string(),
        decision_blake3: String::new(),
        policy_id: request.policy.policy_id.clone(),
        policy_version: request.policy.policy_version.clone(),
        action_family_blake3: request.action_family.identity_blake3.clone(),
        declared_minima: request.declared.minima,
        effective_minima: history.effective_minima,
        eligible_class_ids,
        static_class_id: static_class.class_id.clone(),
        policy_class_id: history.policy_class.class_id.clone(),
        scheduled_class_id: scheduled_class.class_id.clone(),
        compatible_observation_blake3s,
        history_decisions: filter.decisions,
        reason_codes: history.reason_codes,
        observe_only: request.controls.mode == ResourcePolicyMode::ObserveOnly,
        non_claim: RESOURCE_POLICY_NON_CLAIM.to_string(),
    };
    decision.decision_blake3 = selection_decision_identity(&decision)?;
    debug_assert!(!decision.eligible_class_ids.is_empty());
    debug_assert!(quantities_cover(decision.effective_minima, decision.declared_minima));
    Ok(decision)
}

struct HistoryResolution<'a> {
    policy_class: &'a MachineClass,
    effective_minima: ResourceQuantities,
    reason_codes: Vec<SelectionReasonCode>,
}

fn filter_selection_history(
    request: &mut ResourceSelectionRequest,
) -> Result<ObservationFilterReport, ResourcePolicyError> {
    let observations = core::mem::take(&mut request.observations);
    let filtered = filter_resource_observations(ObservationFilterRequest {
        now_unix_s: request.now_unix_s,
        action_family_blake3: request.action_family.identity_blake3.clone(),
        platform_identity: request.declared.platform.platform.clone(),
        compatibility_policy_id: request.policy.policy_id.clone(),
        known_machine_class_ids: request.machine_classes.iter().map(|class| class.class_id.clone()).collect(),
        observation_age_secs_max: request.policy.observation_age_secs_max,
        measurement_bytes_max: request.policy.measurement_bytes_max,
        observations,
    })?;
    debug_assert!(u32::try_from(filtered.accepted.len()).is_ok_and(|count| count <= MAX_OBSERVATIONS));
    debug_assert!(!filtered.decisions.is_empty());
    Ok(filtered)
}

fn resolve_history<'a>(
    request: &'a ResourceSelectionRequest,
    static_class: &'a MachineClass,
    filter: &ObservationFilterReport,
) -> Result<HistoryResolution<'a>, ResourcePolicyError> {
    let mut reason_codes = vec![SelectionReasonCode::StaticMinimumSelected];
    let is_enforcement_active = history_enforcement_active(request.controls);
    let mut policy_class = static_class;
    let mut effective_minima = request.declared.minima;
    if !request.controls.historical_selection_enabled {
        reason_codes.push(SelectionReasonCode::HistoricalSelectionDisabled);
        return Ok(HistoryResolution {
            policy_class,
            effective_minima,
            reason_codes,
        });
    }
    let accepted_count = u32::try_from(filter.accepted.len()).map_err(|_| ResourcePolicyError::TooManyObservations)?;
    if accepted_count < request.policy.sample_count_min {
        reason_codes.push(SelectionReasonCode::HistoryInsufficient);
        if request.policy.fallback == SelectionFallbackRule::RejectOnInsufficientHistory && is_enforcement_active {
            return Err(ResourcePolicyError::InsufficientHistory);
        }
        return Ok(HistoryResolution {
            policy_class,
            effective_minima,
            reason_codes,
        });
    }
    reason_codes.push(SelectionReasonCode::HistoryAccepted);
    effective_minima = historical_minima(request.declared.minima, &filter.accepted, &request.policy)?;
    if effective_minima != request.declared.minima {
        reason_codes.push(SelectionReasonCode::HistoryRaisedMinimum);
    }
    let historical_eligible = eligible_classes(&request.declared, effective_minima, &request.machine_classes);
    match historical_eligible.first().copied() {
        Some(class) => policy_class = class,
        None if is_enforcement_active => return Err(ResourcePolicyError::NoClassForHistoricalMinimum),
        None => reason_codes.push(SelectionReasonCode::HistoricalClassUnavailable),
    }
    debug_assert!(quantities_cover(effective_minima, request.declared.minima));
    debug_assert!(!reason_codes.is_empty());
    Ok(HistoryResolution {
        policy_class,
        effective_minima,
        reason_codes,
    })
}

pub fn validate_resource_selection_decision(
    decision: ResourceSelectionDecision,
) -> Result<ResourceSelectionDecision, ResourcePolicyError> {
    if decision.schema != RESOURCE_SELECTION_DECISION_SCHEMA || decision.non_claim != RESOURCE_POLICY_NON_CLAIM {
        return Err(ResourcePolicyError::InvalidSchema);
    }
    validate_digest(&decision.decision_blake3)?;
    validate_digest(&decision.action_family_blake3)?;
    validate_id(&decision.policy_id)?;
    validate_id(&decision.policy_version)?;
    validate_id(&decision.static_class_id)?;
    validate_id(&decision.policy_class_id)?;
    validate_id(&decision.scheduled_class_id)?;
    bounded_count(decision.eligible_class_ids.len(), MAX_MACHINE_CLASSES, ResourcePolicyError::TooManyMachineClasses)?;
    bounded_count(decision.history_decisions.len(), MAX_OBSERVATIONS, ResourcePolicyError::TooManyObservations)?;
    if selection_decision_identity(&decision)? != decision.decision_blake3 {
        return Err(ResourcePolicyError::InvalidDigest);
    }
    debug_assert!(decision.eligible_class_ids.contains(&decision.static_class_id));
    debug_assert!(!decision.reason_codes.is_empty());
    Ok(decision)
}

fn validate_selection_request(request: &ResourceSelectionRequest) -> Result<(), ResourcePolicyError> {
    validate_policy(&request.policy)?;
    validate_declared(&request.declared)?;
    bounded_count(request.machine_classes.len(), MAX_MACHINE_CLASSES, ResourcePolicyError::TooManyMachineClasses)?;
    bounded_count(request.observations.len(), MAX_OBSERVATIONS, ResourcePolicyError::TooManyObservations)?;
    if request.quota.project_remaining_units > crate::MAX_CHARGE_UNITS
        || request.quota.account_remaining_units > crate::MAX_CHARGE_UNITS
    {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    debug_assert!(u32::try_from(request.machine_classes.len()).is_ok_and(|count| count <= MAX_MACHINE_CLASSES));
    debug_assert!(u32::try_from(request.observations.len()).is_ok_and(|count| count <= MAX_OBSERVATIONS));
    Ok(())
}

fn validate_policy(policy: &ResourceSelectionPolicy) -> Result<(), ResourcePolicyError> {
    if policy.schema != RESOURCE_SELECTION_POLICY_SCHEMA {
        return Err(ResourcePolicyError::InvalidSchema);
    }
    validate_id(&policy.policy_id)?;
    validate_id(&policy.policy_version)?;
    if policy.policy_version.len() > MAX_POLICY_VERSION_BYTES {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    if policy.sample_count_min == 0 || policy.sample_count_min > MAX_OBSERVATIONS {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    if policy.observation_age_secs_max == 0 || policy.observation_age_secs_max > MAX_OBSERVATION_AGE_SECS {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    if policy.memory_margin_basis_points < BASIS_POINTS_DENOMINATOR_U32
        || policy.memory_margin_basis_points > MAX_MARGIN_BASIS_POINTS
    {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    if policy.scratch_margin_basis_points < BASIS_POINTS_DENOMINATOR_U32
        || policy.scratch_margin_basis_points > MAX_MARGIN_BASIS_POINTS
    {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    if policy.measurement_bytes_max == 0 || policy.measurement_bytes_max > MAX_RESOURCE_BYTES {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    debug_assert!(policy.sample_count_min <= MAX_OBSERVATIONS);
    debug_assert!(policy.measurement_bytes_max <= MAX_RESOURCE_BYTES);
    Ok(())
}

fn validate_declared(declared: &DeclaredResourceRequirements) -> Result<(), ResourcePolicyError> {
    validate_quantities(declared.minima)?;
    validate_id(&declared.platform.architecture)?;
    validate_id(&declared.platform.platform)?;
    validate_id(&declared.platform.trust_tier)?;
    validate_id(&declared.platform.isolation)?;
    canonical_strings(declared.platform.required_features.clone(), crate::MAX_FEATURES)?;
    debug_assert!(!declared.platform.architecture.is_empty());
    debug_assert!(declared.minima.cpu_units > 0);
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
    debug_assert!(quantities.memory_bytes > 0);
    debug_assert!(quantities.scratch_bytes <= MAX_RESOURCE_BYTES);
    Ok(())
}

fn canonical_machine_classes(mut classes: Vec<MachineClass>) -> Result<Vec<MachineClass>, ResourcePolicyError> {
    bounded_count(classes.len(), MAX_MACHINE_CLASSES, ResourcePolicyError::TooManyMachineClasses)?;
    for class in &mut classes {
        validate_machine_class(class)?;
        class.endpoint_ids = canonical_strings(class.endpoint_ids.clone(), MAX_CLASS_ENDPOINTS)?;
        class.features = canonical_strings(class.features.clone(), crate::MAX_FEATURES)?;
    }
    classes.sort_by(|left, right| left.ordinal.cmp(&right.ordinal).then(left.class_id.cmp(&right.class_id)));
    let mut ids = BTreeSet::new();
    let mut ordinals = BTreeSet::new();
    for class in &classes {
        if !ids.insert(class.class_id.as_str()) || !ordinals.insert(class.ordinal) {
            return Err(ResourcePolicyError::DuplicateIdentity);
        }
    }
    debug_assert!(classes.windows(crate::PAIR_WINDOW_SIZE).all(|pair| pair[0].ordinal < pair[1].ordinal));
    debug_assert_eq!(ids.len(), classes.len());
    Ok(classes)
}

fn validate_machine_class(class: &MachineClass) -> Result<(), ResourcePolicyError> {
    validate_id(&class.class_id)?;
    validate_id(&class.architecture)?;
    validate_id(&class.platform)?;
    validate_id(&class.trust_tier)?;
    validate_id(&class.isolation)?;
    validate_digest(&class.onixos_source_blake3)?;
    validate_quantities(class.capacity)?;
    if class.ordinal == 0 || class.ordinal > crate::DEFAULT_CLASS_ORDINAL_MAX {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    if class.reservation_charge_units == 0 || class.reservation_charge_units > crate::MAX_CHARGE_UNITS {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    if class.endpoint_ids.is_empty() {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    debug_assert!(class.ordinal > 0);
    debug_assert!(!class.endpoint_ids.is_empty());
    Ok(())
}

fn eligible_classes<'a>(
    declared: &DeclaredResourceRequirements,
    minima: ResourceQuantities,
    classes: &'a [MachineClass],
) -> Vec<&'a MachineClass> {
    let required_features = declared.platform.required_features.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let eligible = classes
        .iter()
        .filter(|class| class.available)
        .filter(|class| class.architecture == declared.platform.architecture)
        .filter(|class| class.platform == declared.platform.platform)
        .filter(|class| !declared.platform.kvm_required || class.kvm_available)
        .filter(|class| class.trust_tier == declared.platform.trust_tier)
        .filter(|class| class.isolation == declared.platform.isolation)
        .filter(|class| {
            let available = class.features.iter().map(String::as_str).collect::<BTreeSet<_>>();
            required_features.is_subset(&available)
        })
        .filter(|class| quantities_cover(class.capacity, minima))
        .collect::<Vec<_>>();
    debug_assert!(eligible.len() <= classes.len());
    debug_assert!(eligible.windows(crate::PAIR_WINDOW_SIZE).all(|pair| pair[0].ordinal < pair[1].ordinal));
    eligible
}

fn quantities_cover(available: ResourceQuantities, required: ResourceQuantities) -> bool {
    available.cpu_units >= required.cpu_units
        && available.memory_bytes >= required.memory_bytes
        && available.scratch_bytes >= required.scratch_bytes
}

fn historical_minima(
    declared: ResourceQuantities,
    observations: &[crate::ResourceObservation],
    policy: &ResourceSelectionPolicy,
) -> Result<ResourceQuantities, ResourcePolicyError> {
    let cpu_units = observations
        .iter()
        .fold(declared.cpu_units, |maximum, observation| maximum.max(observation.selected.cpu_units));
    let memory_peak = observations.iter().try_fold(declared.memory_bytes, |maximum, observation| {
        Ok::<u64, ResourcePolicyError>(maximum.max(observation.measurements.peak_memory_bytes))
    })?;
    let scratch_peak = observations.iter().try_fold(declared.scratch_bytes, |maximum, observation| {
        Ok::<u64, ResourcePolicyError>(maximum.max(observation.measurements.scratch_peak_bytes))
    })?;
    let memory_bytes = declared.memory_bytes.max(apply_margin(memory_peak, policy.memory_margin_basis_points)?);
    let scratch_bytes = declared.scratch_bytes.max(apply_margin(scratch_peak, policy.scratch_margin_basis_points)?);
    let effective = ResourceQuantities {
        cpu_units,
        memory_bytes,
        scratch_bytes,
    };
    validate_quantities(effective)?;
    debug_assert!(effective >= declared);
    debug_assert!(!observations.is_empty());
    Ok(effective)
}

fn apply_margin(value: u64, basis_points: u32) -> Result<u64, ResourcePolicyError> {
    let scaled = value.checked_mul(u64::from(basis_points)).ok_or(ResourcePolicyError::ArithmeticOverflow)?;
    let rounded = scaled
        .checked_add(BASIS_POINTS_DENOMINATOR.saturating_sub(1))
        .ok_or(ResourcePolicyError::ArithmeticOverflow)?;
    let result = rounded / BASIS_POINTS_DENOMINATOR;
    if result > MAX_RESOURCE_BYTES {
        return Err(ResourcePolicyError::InvalidBounds);
    }
    debug_assert!(result >= value);
    debug_assert!(basis_points >= BASIS_POINTS_DENOMINATOR_U32);
    Ok(result)
}

fn history_enforcement_active(controls: crate::ResourceFeatureControls) -> bool {
    controls.mode == ResourcePolicyMode::Enforce && controls.project_opted_in && controls.historical_selection_enabled
}

fn apply_quota_policy<'a>(
    request: &ResourceSelectionRequest,
    static_class: &'a MachineClass,
    policy_class: &'a MachineClass,
    is_enforcement_active: bool,
    reasons: &mut Vec<SelectionReasonCode>,
) -> Result<&'a MachineClass, ResourcePolicyError> {
    if !request.controls.quota_enforcement_enabled {
        reasons.push(SelectionReasonCode::QuotaEnforcementDisabled);
        return Ok(policy_class);
    }
    let charge_units = policy_class.reservation_charge_units;
    if charge_units <= request.quota.project_remaining_units && charge_units <= request.quota.account_remaining_units {
        reasons.push(SelectionReasonCode::QuotaAccepted);
        return Ok(policy_class);
    }
    if is_enforcement_active {
        return Err(ResourcePolicyError::QuotaDenied);
    }
    reasons.push(SelectionReasonCode::QuotaDeniedPolicyFallback);
    Ok(static_class)
}

fn scheduled_class<'a>(
    request: &ResourceSelectionRequest,
    static_class: &'a MachineClass,
    policy_class: &'a MachineClass,
    reasons: &mut Vec<SelectionReasonCode>,
) -> &'a MachineClass {
    if request.controls.mode == ResourcePolicyMode::ObserveOnly {
        reasons.push(SelectionReasonCode::ObserveOnlyStaticRetained);
        return static_class;
    }
    if !request.controls.project_opted_in {
        reasons.push(SelectionReasonCode::ProjectNotOptedIn);
        return static_class;
    }
    if !request.controls.historical_selection_enabled {
        return static_class;
    }
    policy_class
}

fn selection_decision_identity(decision: &ResourceSelectionDecision) -> Result<String, ResourcePolicyError> {
    let mut material = decision.clone();
    material.decision_blake3.clear();
    let identity = canonical_blake3(SELECTION_DOMAIN, &material)?;
    debug_assert!(crate::valid_blake3(&identity));
    debug_assert!(material.decision_blake3.is_empty());
    Ok(identity)
}
