use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;

use crate::Component;
use crate::ComponentKind;
use crate::ErrorCode;
use crate::ExitKind;
use crate::MAX_COMPONENTS;
use crate::MAX_DEPENDENCIES_PER_COMPONENT;
use crate::MAX_LABEL_BYTES;
use crate::MAX_OBSERVATION_STATES;
use crate::MAX_USER_STATES;
use crate::Observation;
use crate::READINESS_SCHEMA;
use crate::ReadinessError;
use crate::Snapshot;

pub(crate) struct Admitted<'a> {
    pub(crate) indices: BTreeMap<&'a str, usize>,
    pub(crate) observations: Vec<Option<&'a Observation<'a>>>,
    pub(crate) dependency_order: Vec<usize>,
}

pub(crate) fn admit<'a>(snapshot: &Snapshot<'a>) -> Result<Admitted<'a>, ReadinessError<'a>> {
    if snapshot.schema != READINESS_SCHEMA {
        return Err(ReadinessError::new(ErrorCode::UnsupportedSchema, None, None));
    }
    if snapshot.components.len() > MAX_COMPONENTS as usize {
        return Err(ReadinessError::new(ErrorCode::TooManyComponents, None, None));
    }
    let mut indices = BTreeMap::new();
    for (index, component) in snapshot.components.iter().enumerate() {
        validate_component(component)?;
        if indices.insert(component.id, index).is_some() {
            return Err(ReadinessError::new(ErrorCode::DuplicateComponent, Some(component.id), None));
        }
    }
    let dependency_order = validate_graph(snapshot.components, &indices)?;
    let observations = validate_observations(snapshot, &indices)?;
    Ok(Admitted {
        indices,
        observations,
        dependency_order,
    })
}

fn validate_component<'a>(component: &Component<'a>) -> Result<(), ReadinessError<'a>> {
    if !valid_label(component.id) {
        return Err(ReadinessError::new(ErrorCode::InvalidId, Some(component.id), None));
    }
    if component.depends_on.len() > MAX_DEPENDENCIES_PER_COMPONENT as usize {
        return Err(ReadinessError::new(ErrorCode::TooManyDependencies, Some(component.id), None));
    }
    if component.user_states.len() > MAX_USER_STATES as usize {
        return Err(ReadinessError::new(ErrorCode::TooManyStates, Some(component.id), None));
    }
    validate_policy_shape(component)?;
    for (position, state) in component.user_states.iter().enumerate() {
        if !valid_label(state) || builtin(state) {
            return Err(ReadinessError::new(ErrorCode::InvalidUserState, Some(component.id), Some(state)));
        }
        if component.user_states[..position].contains(state) {
            return Err(ReadinessError::new(ErrorCode::DuplicateUserState, Some(component.id), Some(state)));
        }
    }
    Ok(())
}

fn validate_policy_shape<'a>(component: &Component<'a>) -> Result<(), ReadinessError<'a>> {
    match (component.kind, component.restart_policy, component.stage_requirements) {
        (ComponentKind::Service, None, _) => {
            Err(ReadinessError::new(ErrorCode::MissingPolicy, Some(component.id), None))
        }
        (ComponentKind::Service, Some(_), Some(_)) => {
            Err(ReadinessError::new(ErrorCode::UnexpectedStageRequirements, Some(component.id), None))
        }
        (ComponentKind::ProofStage, Some(_), _) => {
            Err(ReadinessError::new(ErrorCode::UnexpectedPolicy, Some(component.id), None))
        }
        (ComponentKind::ProofStage, None, None) => {
            Err(ReadinessError::new(ErrorCode::MissingStageRequirements, Some(component.id), None))
        }
        _ => Ok(()),
    }
}

fn validate_graph<'a>(
    components: &'a [Component<'a>],
    indices: &BTreeMap<&'a str, usize>,
) -> Result<Vec<usize>, ReadinessError<'a>> {
    let mut marks = vec![0_u8; components.len()];
    let mut order = Vec::with_capacity(components.len());
    for index in 0..components.len() {
        visit(index, components, indices, &mut marks, &mut order)?;
    }
    Ok(order)
}

fn visit<'a>(
    index: usize,
    components: &'a [Component<'a>],
    indices: &BTreeMap<&'a str, usize>,
    marks: &mut [u8],
    order: &mut Vec<usize>,
) -> Result<(), ReadinessError<'a>> {
    if marks[index] == 2 {
        return Ok(());
    }
    let component = &components[index];
    if marks[index] == 1 {
        return Err(ReadinessError::new(ErrorCode::CyclicDependency, Some(component.id), None));
    }
    marks[index] = 1;
    let mut seen = 0_u128;
    for dependency in component.depends_on {
        let Some(&dependent_index) = indices.get(dependency) else {
            return Err(ReadinessError::new(ErrorCode::UnknownDependency, Some(component.id), Some(dependency)));
        };
        let bit = 1_u128 << dependent_index;
        if seen & bit != 0 {
            return Err(ReadinessError::new(ErrorCode::DuplicateDependency, Some(component.id), Some(dependency)));
        }
        seen |= bit;
        visit(dependent_index, components, indices, marks, order)?;
    }
    marks[index] = 2;
    order.push(index);
    Ok(())
}

fn validate_observations<'a>(
    snapshot: &Snapshot<'a>,
    indices: &BTreeMap<&'a str, usize>,
) -> Result<Vec<Option<&'a Observation<'a>>>, ReadinessError<'a>> {
    if snapshot.observations.len() > MAX_COMPONENTS as usize {
        return Err(ReadinessError::new(ErrorCode::TooManyObservations, None, None));
    }
    let mut by_index = vec![None; snapshot.components.len()];
    for observation in snapshot.observations {
        let Some(&index) = indices.get(observation.id) else {
            return Err(ReadinessError::new(ErrorCode::UnknownComponent, Some(observation.id), None));
        };
        if by_index[index].replace(observation).is_some() {
            return Err(ReadinessError::new(ErrorCode::DuplicateObservation, Some(observation.id), None));
        }
        validate_observation(&snapshot.components[index], observation)?;
    }
    Ok(by_index)
}

fn validate_observation<'a>(
    component: &Component<'a>,
    observation: &Observation<'a>,
) -> Result<(), ReadinessError<'a>> {
    if observation.generation == 0 {
        return Err(ReadinessError::new(ErrorCode::InvalidGeneration, Some(component.id), None));
    }
    if observation.states.len() > MAX_OBSERVATION_STATES as usize {
        return Err(ReadinessError::new(ErrorCode::TooManyStates, Some(component.id), None));
    }
    for (position, state) in observation.states.iter().enumerate() {
        if !builtin(state) && !component.user_states.contains(state) {
            return Err(ReadinessError::new(ErrorCode::UnknownState, Some(component.id), Some(state)));
        }
        if observation.states[..position].contains(state) {
            return Err(ReadinessError::new(ErrorCode::DuplicateState, Some(component.id), Some(state)));
        }
    }
    match component.kind {
        ComponentKind::Service => validate_service_observation(observation),
        ComponentKind::ProofStage => validate_stage_observation(component, observation),
    }
}

fn validate_service_observation<'a>(observation: &Observation<'a>) -> Result<(), ReadinessError<'a>> {
    let invalid = || ReadinessError::new(ErrorCode::InvalidStateCombination, Some(observation.id), None);
    if observation.proof_completion.is_some() {
        return Err(ReadinessError::new(ErrorCode::InvalidProofCompletion, Some(observation.id), None));
    }
    if observation.exit.is_none() {
        if has(observation.states, "complete") || has(observation.states, "failed") {
            return Err(invalid());
        }
        if has(observation.states, "ready") && !has(observation.states, "started") {
            return Err(invalid());
        }
        if has(observation.states, "ready") && !observation.request_acknowledged {
            return Err(ReadinessError::new(ErrorCode::MissingRequestAcknowledgement, Some(observation.id), None));
        }
        if observation.request_acknowledged && !has(observation.states, "started") {
            return Err(invalid());
        }
        return Ok(());
    }
    if observation.states.iter().any(|state| *state != "failed" && *state != "complete") {
        return Err(invalid());
    }
    let complete = observation.exit == Some(ExitKind::Normal) && observation.request_acknowledged;
    if has(observation.states, "complete") != complete && !observation.states.is_empty() {
        return Err(invalid());
    }
    if !observation.states.is_empty() && has(observation.states, "failed") == complete {
        return Err(invalid());
    }
    Ok(())
}

fn validate_stage_observation<'a>(
    component: &Component<'a>,
    observation: &Observation<'a>,
) -> Result<(), ReadinessError<'a>> {
    let invalid = || ReadinessError::new(ErrorCode::InvalidStateCombination, Some(component.id), None);
    if observation.exit.is_some() || observation.request_acknowledged || has(observation.states, "ready") {
        return Err(invalid());
    }
    if has(observation.states, "started") && (has(observation.states, "failed") || has(observation.states, "complete"))
    {
        return Err(invalid());
    }
    let complete = has(observation.states, "complete");
    let failed = has(observation.states, "failed");
    if complete && failed {
        return Err(invalid());
    }
    if (complete || failed) && observation.states.len() != 1 {
        return Err(invalid());
    }
    match (complete, observation.proof_completion) {
        (false, None) => Ok(()),
        (true, Some(facts)) => validate_completion(component, facts),
        _ => Err(ReadinessError::new(ErrorCode::InvalidProofCompletion, Some(component.id), None)),
    }
}

fn validate_completion<'a>(
    component: &Component<'a>,
    facts: crate::ProofCompletion<'a>,
) -> Result<(), ReadinessError<'a>> {
    let requirements = component.stage_requirements.expect("admission requires proof-stage requirements");
    if !valid_digest(facts.stage_evidence_digest_blake3) || !valid_digest(facts.output_digest_blake3) {
        return Err(ReadinessError::new(ErrorCode::InvalidProofDigest, Some(component.id), None));
    }
    if !facts.execution_verified {
        return Err(ReadinessError::new(ErrorCode::InvalidProofCompletion, Some(component.id), None));
    }
    if requirements.require_action_reconciliation && !facts.action_reconciled {
        return Err(ReadinessError::new(ErrorCode::InvalidProofCompletion, Some(component.id), None));
    }
    if requirements.require_v2_receipt && !facts.v2_receipt_verified {
        return Err(ReadinessError::new(ErrorCode::InvalidProofCompletion, Some(component.id), None));
    }
    Ok(())
}

fn valid_label(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.is_empty() || bytes.len() > MAX_LABEL_BYTES as usize || !bytes[0].is_ascii_lowercase() {
        return false;
    }
    bytes[1..].iter().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn builtin(value: &str) -> bool {
    matches!(value, "started" | "ready" | "complete" | "failed")
}

pub(crate) fn has(states: &[&str], state: &str) -> bool {
    states.contains(&state)
}
