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
    let mut rows: Vec<Option<ReadinessRow<'a>>> = vec![None; snapshot.components.len()];
    for &index in &admitted.dependency_order {
        let declaration = &snapshot.components[index];
        let mut blocked_by = Vec::new();
        for dependency in declaration.depends_on {
            let dependency_index = admitted.indices[dependency];
            let predecessor = rows[dependency_index].as_ref().expect("topological predecessor is present");
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
    for &index in admitted.indices.values() {
        ordered.push(rows[index].take().expect("admitted component is projected"));
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
    let report = evaluate(snapshot)?;
    for declaration in snapshot.components {
        let index = report
            .components
            .binary_search_by_key(&declaration.id, |row| row.id)
            .expect("declaration has report row");
        let current = &report.components[index];
        let old = previous
            .components
            .binary_search_by_key(&declaration.id, |row| row.id)
            .ok()
            .map(|position| &previous.components[position]);
        if declaration.kind == ComponentKind::ProofStage {
            let previous_generation = old.and_then(|row| row.generation);
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
    Ok(report)
}

fn validate_transition<'a>(
    id: &'a str,
    old: Option<&ReadinessRow<'_>>,
    current: &ReadinessRow<'a>,
    snapshot: &Snapshot<'a>,
) -> Result<(), ReadinessError<'a>> {
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
            return Ok(());
        }
        if !old.is_some_and(|row| has(&row.states, "failed") || has(&row.states, "complete")) {
            return Err(ReadinessError::new(ErrorCode::UnobservedExit, Some(id), None));
        }
        if old.is_some_and(|row| row.restart_action == Some(RestartAction::None)) {
            return Err(ReadinessError::new(ErrorCode::RestartDenied, Some(id), None));
        }
    }
    let observation = snapshot.observations.iter().find(|entry| entry.id == id).expect("observed generation");
    if !has(&current.states, "started")
        || has(&current.states, "ready")
        || observation.request_acknowledged
        || observation.exit.is_some()
    {
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
    let mut states = Vec::new();
    let mut restart_action = None;
    let mut generation = None;
    if let Some(observation) = observation {
        generation = Some(observation.generation);
        if declaration.kind == ComponentKind::Service {
            restart_action = project_service(observation, declaration.restart_policy, &mut states);
        } else {
            states.extend_from_slice(observation.states);
        }
    }
    let observed_ready = has(&states, "ready");
    let ready = observed_ready && blocked_by.is_empty();
    if observed_ready && !ready {
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
        ready,
        blocked_by,
    })
}

fn project_service<'a>(
    observation: &Observation<'a>,
    policy: Option<RestartPolicy>,
    states: &mut Vec<&'a str>,
) -> Option<RestartAction> {
    if let Some(exit) = observation.exit {
        let complete = exit == ExitKind::Normal && observation.request_acknowledged;
        states.push(if complete { "complete" } else { "failed" });
        return Some(restart_action(policy.expect("admitted service has policy"), exit));
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
