use alloc::vec;
use alloc::vec::Vec;

use crate::Component;
use crate::ComponentKind;
use crate::ErrorCode;
use crate::ExitKind;
use crate::Observation;
use crate::ReadinessError;
use crate::ReadinessReport;
use crate::ReadinessRow;
use crate::RestartAction;
use crate::RestartPolicy;
use crate::Snapshot;
use crate::admission::admit;
use crate::admission::has;

/// Derive a deterministic, non-evidence coordination snapshot from bounded facts.
pub fn evaluate<'a>(snapshot: &Snapshot<'a>) -> Result<ReadinessReport<'a>, ReadinessError<'a>> {
    let admitted = admit(snapshot)?;
    debug_assert_eq!(admitted.indices.len(), snapshot.components.len());
    debug_assert_eq!(admitted.dependency_order.len(), snapshot.components.len());
    let mut rows: Vec<Option<ReadinessRow<'a>>> = vec![None; snapshot.components.len()];
    for &index in &admitted.dependency_order {
        let declaration = &snapshot.components[index];
        let mut blocked_by = Vec::with_capacity(declaration.depends_on.len());
        for dependency in declaration.depends_on {
            let Some(&dependency_index) = admitted.indices.get(dependency) else {
                return Err(ReadinessError::new(ErrorCode::UnknownDependency, Some(declaration.id), Some(dependency)));
            };
            let Some(predecessor) = rows.get(dependency_index).and_then(Option::as_ref) else {
                return Err(ReadinessError::new(ErrorCode::CyclicDependency, Some(declaration.id), Some(dependency)));
            };
            if !predecessor.ready && !has(&predecessor.states, "complete") {
                blocked_by.push(*dependency);
            }
        }
        blocked_by.sort_unstable();
        let observation = admitted.observations[index];
        let row = derive_row(declaration, observation, blocked_by)?;
        rows[index] = Some(row);
    }
    let mut ordered = Vec::with_capacity(snapshot.components.len());
    for (&id, &index) in &admitted.indices {
        let Some(row) = rows.get_mut(index).and_then(Option::take) else {
            return Err(ReadinessError::new(ErrorCode::UnknownComponent, Some(id), None));
        };
        ordered.push(row);
    }
    Ok(ReadinessReport::new(ordered))
}

/// Apply one observation transition. A service must expose `started` first,
/// publish its terminal exit before a new generation, and acknowledge a real
/// request before asserting `ready`. The shell owns truthful observations.
pub fn advance<'a>(
    previous: &ReadinessReport<'_>,
    snapshot: &Snapshot<'a>,
) -> Result<ReadinessReport<'a>, ReadinessError<'a>> {
    if previous.schema != crate::READINESS_SCHEMA
        || previous.classification != crate::COORDINATION_CLASSIFICATION
        || previous.evidence_eligible
    {
        return Err(ReadinessError::new(ErrorCode::InvalidPreviousReport, None, None));
    }
    let admitted_readiness = evaluate(snapshot)?;
    debug_assert_eq!(admitted_readiness.components.len(), snapshot.components.len());
    debug_assert!(!admitted_readiness.evidence_eligible);
    for declaration in snapshot.components {
        let Ok(index) = admitted_readiness.components.binary_search_by_key(&declaration.id, |row| row.id) else {
            return Err(ReadinessError::new(ErrorCode::UnknownComponent, Some(declaration.id), None));
        };
        let current = &admitted_readiness.components[index];
        let old = previous
            .components
            .binary_search_by_key(&declaration.id, |row| row.id)
            .ok()
            .map(|position| &previous.components[position]);
        if old.is_some_and(|row| row.restart_policy != declaration.restart_policy) {
            return Err(ReadinessError::new(ErrorCode::DeclarationChanged, Some(declaration.id), None));
        }
        if declaration.kind == ComponentKind::ProofStage {
            let previous_generation = old.and_then(|row| row.generation);
            if let Some(generation) = previous_generation {
                if current.generation.is_some_and(|next| next < generation) {
                    return Err(ReadinessError::new(ErrorCode::StaleGeneration, Some(declaration.id), None));
                }
                if old.is_some_and(|row| has(&row.states, "failed") || has(&row.states, "complete"))
                    && (current.generation.is_none()
                        || current.generation == previous_generation
                            && old.is_some_and(|row| row.states != current.states))
                {
                    return Err(ReadinessError::new(ErrorCode::RevivedTerminal, Some(declaration.id), None));
                }
                if current.generation.is_some()
                    && current.generation != previous_generation
                    && old.is_none_or(|row| !has(&row.states, "failed") && !has(&row.states, "complete"))
                {
                    return Err(ReadinessError::new(ErrorCode::UnobservedExit, Some(declaration.id), None));
                }
            }
            if current.generation.is_some()
                && current.generation != previous_generation
                && !has(&current.states, "started")
            {
                return Err(ReadinessError::new(ErrorCode::UnobservedStart, Some(declaration.id), None));
            }
            if current.generation != previous_generation
                && has(&current.states, "started")
                && let Some(blocker) = current.blocked_by.first()
            {
                return Err(ReadinessError::new(ErrorCode::BlockedStart, Some(declaration.id), Some(blocker)));
            }
            continue;
        }
        validate_transition(declaration.id, old, current, snapshot)?;
    }
    Ok(admitted_readiness)
}

fn validate_transition<'a>(
    id: &'a str,
    old: Option<&ReadinessRow<'_>>,
    current: &ReadinessRow<'a>,
    snapshot: &Snapshot<'a>,
) -> Result<(), ReadinessError<'a>> {
    debug_assert_eq!(current.id, id);
    debug_assert!(current.generation.is_none_or(|generation| generation > 0));
    let old_generation = old.and_then(|row| row.generation);
    let Some(new_generation) = current.generation else {
        if old_generation.is_some() {
            return Err(ReadinessError::new(ErrorCode::UnobservedExit, Some(id), None));
        }
        return Ok(());
    };
    if let Some(old_generation) = old_generation {
        if new_generation < old_generation {
            return Err(ReadinessError::new(ErrorCode::StaleGeneration, Some(id), None));
        }
        if new_generation == old_generation {
            if old.is_some_and(|row| has(&row.states, "failed") || has(&row.states, "complete"))
                && old.is_none_or(|row| row.states != current.states || row.restart_action != current.restart_action)
            {
                return Err(ReadinessError::new(ErrorCode::RevivedTerminal, Some(id), None));
            }
            if current.restart_action.is_some()
                && current.request_acknowledged
                && old.is_none_or(|row| !row.request_acknowledged)
            {
                return Err(ReadinessError::new(ErrorCode::MissingRequestAcknowledgement, Some(id), None));
            }
            return Ok(());
        }
        if !old.is_some_and(|row| has(&row.states, "failed") || has(&row.states, "complete")) {
            return Err(ReadinessError::new(ErrorCode::UnobservedExit, Some(id), None));
        }
        if old.is_some_and(|row| row.restart_action == Some(RestartAction::None)) {
            return Err(ReadinessError::new(ErrorCode::RestartDenied, Some(id), None));
        }
    }
    let Some(observation) = snapshot.observations.iter().find(|entry| entry.id == id) else {
        return Err(ReadinessError::new(ErrorCode::UnobservedStart, Some(id), None));
    };
    let is_invalid_start = !has(&current.states, "started") || has(&current.states, "ready");
    let has_advanced_observation = observation.request_acknowledged || observation.exit.is_some();
    if is_invalid_start || has_advanced_observation {
        return Err(ReadinessError::new(ErrorCode::UnobservedStart, Some(id), None));
    }
    if let Some(blocker) = current.blocked_by.first() {
        return Err(ReadinessError::new(ErrorCode::BlockedStart, Some(id), Some(blocker)));
    }
    Ok(())
}

/// Whether a *new* instance may start; an existing started instance is not a new start.
pub fn may_start(report: &ReadinessReport<'_>, id: &str) -> Option<bool> {
    let index = report.components.binary_search_by_key(&id, |row| row.id).ok()?;
    let row = &report.components[index];
    Some(row.generation.is_none() && row.blocked_by.is_empty())
}

fn derive_row<'a>(
    declaration: &Component<'a>,
    observation: Option<&'a Observation<'a>>,
    blocked_by: Vec<&'a str>,
) -> Result<ReadinessRow<'a>, ReadinessError<'a>> {
    debug_assert!(blocked_by.len() <= declaration.depends_on.len());
    debug_assert!(observation.is_none_or(|entry| entry.id == declaration.id));
    let mut states = Vec::new();
    let mut restart_action = None;
    let mut generation = None;
    if let Some(observation) = observation {
        generation = Some(observation.generation);
        if declaration.kind == ComponentKind::Service {
            let Some(policy) = declaration.restart_policy else {
                return Err(ReadinessError::new(ErrorCode::MissingPolicy, Some(declaration.id), None));
            };
            restart_action = project_service(observation, policy, &mut states);
        } else {
            states.extend_from_slice(observation.states);
        }
    }
    debug_assert!(u32::try_from(states.len()).is_ok_and(|count| count <= crate::MAX_OBSERVATION_STATES));
    let is_observed_ready = has(&states, "ready");
    let is_ready = is_observed_ready && blocked_by.is_empty();
    if is_observed_ready && !is_ready {
        states.retain(|state| *state != "ready");
    }
    if declaration.kind == ComponentKind::ProofStage && !blocked_by.is_empty() && has(&states, "complete") {
        return Err(ReadinessError::new(
            ErrorCode::BlockedCompletion,
            Some(declaration.id),
            blocked_by.first().copied(),
        ));
    }
    Ok(ReadinessRow {
        id: declaration.id,
        generation,
        states,
        restart_policy: declaration.restart_policy,
        restart_action,
        ready: is_ready,
        request_acknowledged: observation.is_some_and(|entry| entry.request_acknowledged),
        blocked_by,
    })
}

fn project_service<'a>(
    observation: &Observation<'a>,
    policy: RestartPolicy,
    states: &mut Vec<&'a str>,
) -> Option<RestartAction> {
    if let Some(exit) = observation.exit {
        let is_complete = exit == ExitKind::Normal && observation.request_acknowledged;
        states.push(if is_complete { "complete" } else { "failed" });
        return Some(restart_action(policy, exit));
    }
    states.extend_from_slice(observation.states);
    None
}

fn restart_action(policy: RestartPolicy, exit: ExitKind) -> RestartAction {
    match (policy, exit) {
        (RestartPolicy::Always, _) | (RestartPolicy::OnError, ExitKind::Abnormal) => RestartAction::Component,
        (RestartPolicy::All, ExitKind::Abnormal) => RestartAction::Group,
        _ => RestartAction::None,
    }
}
