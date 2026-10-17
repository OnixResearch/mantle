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
    if u32::try_from(snapshot.components.len()).map_or(true, |count| count > MAX_COMPONENTS) {
        return Err(ReadinessError::new(ErrorCode::TooManyComponents, None, None));
    }
    let mut indices = BTreeMap::new();
    for (index, component) in snapshot.components.iter().enumerate() {
        validate_component(component)?;
        if indices.insert(component.id, index).is_some() {
            return Err(ReadinessError::new(ErrorCode::DuplicateComponent, Some(component.id), None));
        }
    }
    debug_assert_eq!(indices.len(), snapshot.components.len());
    let dependency_order = validate_graph(snapshot.components, &indices)?;
    let observations = validate_observations(snapshot, &indices)?;
    debug_assert_eq!(dependency_order.len(), snapshot.components.len());
    debug_assert_eq!(observations.len(), snapshot.components.len());
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
    if u32::try_from(component.depends_on.len()).map_or(true, |count| count > MAX_DEPENDENCIES_PER_COMPONENT) {
        return Err(ReadinessError::new(ErrorCode::TooManyDependencies, Some(component.id), None));
    }
    if u32::try_from(component.user_states.len()).map_or(true, |count| count > MAX_USER_STATES) {
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
    debug_assert!(component.user_states.iter().all(|state| valid_label(state) && !builtin(state)));
    debug_assert!(
        component
            .user_states
            .iter()
            .enumerate()
            .all(|(position, state)| !component.user_states[..position].contains(state))
    );
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

// The traversal's visited-dependency bitset must represent every admitted component.
const _: () = assert!(MAX_COMPONENTS <= u128::BITS);

struct VisitFrame {
    index: usize,
    next_dependency: usize,
    seen: u128,
}

struct GraphState {
    marks: Vec<u8>,
    order: Vec<usize>,
    stack: Vec<VisitFrame>,
}
fn validate_graph<'a>(
    components: &'a [Component<'a>],
    indices: &BTreeMap<&'a str, usize>,
) -> Result<Vec<usize>, ReadinessError<'a>> {
    let mut state = GraphState {
        marks: vec![0_u8; components.len()],
        order: Vec::with_capacity(components.len()),
        stack: Vec::with_capacity(components.len()),
    };
    for index in 0..components.len() {
        visit(index, components, indices, &mut state)?;
    }
    debug_assert!(state.stack.is_empty());
    debug_assert_eq!(state.order.len(), components.len());
    Ok(state.order)
}

fn visit<'a>(
    index: usize,
    components: &'a [Component<'a>],
    indices: &BTreeMap<&'a str, usize>,
    state: &mut GraphState,
) -> Result<(), ReadinessError<'a>> {
    let GraphState { marks, order, stack } = state;
    debug_assert_eq!(marks.len(), components.len());
    debug_assert!(stack.is_empty());
    if marks[index] == 2 {
        return Ok(());
    }
    if u32::try_from(stack.len()).map_or(true, |depth| depth >= MAX_COMPONENTS) {
        return Err(ReadinessError::new(ErrorCode::TooManyComponents, None, None));
    }
    marks[index] = 1;
    stack.push(VisitFrame {
        index,
        next_dependency: 0,
        seen: 0,
    });
    // Each frame resumes at its next edge, so dependency and error order match recursive DFS.
    while let Some(frame) = stack.last_mut() {
        let component = &components[frame.index];
        if frame.next_dependency == component.depends_on.len() {
            marks[frame.index] = 2;
            order.push(frame.index);
            stack.pop();
            continue;
        }
        let dependency = component.depends_on[frame.next_dependency];
        frame.next_dependency += 1;
        let Some(&dependent_index) = indices.get(dependency) else {
            return Err(ReadinessError::new(ErrorCode::UnknownDependency, Some(component.id), Some(dependency)));
        };
        debug_assert!(dependent_index < components.len());
        let bit = 1_u128 << dependent_index;
        if frame.seen & bit != 0 {
            return Err(ReadinessError::new(ErrorCode::DuplicateDependency, Some(component.id), Some(dependency)));
        }
        frame.seen |= bit;
        if marks[dependent_index] == 1 {
            return Err(ReadinessError::new(ErrorCode::CyclicDependency, Some(components[dependent_index].id), None));
        }
        if marks[dependent_index] == 2 {
            continue;
        }
        if u32::try_from(stack.len()).map_or(true, |depth| depth >= MAX_COMPONENTS) {
            return Err(ReadinessError::new(ErrorCode::TooManyComponents, None, None));
        }
        marks[dependent_index] = 1;
        stack.push(VisitFrame {
            index: dependent_index,
            next_dependency: 0,
            seen: 0,
        });
    }
    debug_assert!(stack.is_empty());
    Ok(())
}

fn validate_observations<'a>(
    snapshot: &Snapshot<'a>,
    indices: &BTreeMap<&'a str, usize>,
) -> Result<Vec<Option<&'a Observation<'a>>>, ReadinessError<'a>> {
    if u32::try_from(snapshot.observations.len()).map_or(true, |count| count > MAX_COMPONENTS) {
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
    if u32::try_from(observation.states.len()).map_or(true, |count| count > MAX_OBSERVATION_STATES) {
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
    debug_assert_eq!(component.id, observation.id);
    debug_assert!(
        observation
            .states
            .iter()
            .enumerate()
            .all(|(position, state)| !observation.states[..position].contains(state))
    );
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
    debug_assert!(observation.proof_completion.is_none());
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
        debug_assert!(!has(observation.states, "complete") && !has(observation.states, "failed"));
        debug_assert!(!has(observation.states, "ready") || observation.request_acknowledged);
        return Ok(());
    }
    if observation.states.iter().any(|state| *state != "failed" && *state != "complete") {
        return Err(invalid());
    }
    let is_complete = observation.exit == Some(ExitKind::Normal) && observation.request_acknowledged;
    if has(observation.states, "complete") != is_complete && !observation.states.is_empty() {
        return Err(invalid());
    }
    if !observation.states.is_empty() && has(observation.states, "failed") == is_complete {
        return Err(invalid());
    }
    debug_assert!(observation.exit.is_some());
    debug_assert!(observation.states.iter().all(|state| *state == "complete" || *state == "failed"));
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
    debug_assert!(observation.exit.is_none() && !observation.request_acknowledged);
    let is_complete = has(observation.states, "complete");
    let is_failed = has(observation.states, "failed");
    if is_complete && is_failed {
        return Err(invalid());
    }
    if (is_complete || is_failed) && observation.states.len() != 1 {
        return Err(invalid());
    }
    debug_assert!(!(is_complete && is_failed));
    debug_assert!(!is_complete && !is_failed || observation.states.len() == 1);
    match (is_complete, observation.proof_completion) {
        (false, None) => Ok(()),
        (true, Some(facts)) => validate_completion(component, facts),
        _ => Err(ReadinessError::new(ErrorCode::InvalidProofCompletion, Some(component.id), None)),
    }
}

fn validate_completion<'a>(
    component: &Component<'a>,
    facts: crate::ProofCompletion<'a>,
) -> Result<(), ReadinessError<'a>> {
    let Some(requirements) = component.stage_requirements else {
        return Err(ReadinessError::new(ErrorCode::MissingStageRequirements, Some(component.id), None));
    };
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
    if bytes.is_empty()
        || u32::try_from(bytes.len()).map_or(true, |count| count > MAX_LABEL_BYTES)
        || !bytes[0].is_ascii_lowercase()
    {
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
