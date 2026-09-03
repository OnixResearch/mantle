#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlannedEffectExecutionError<E> {
    PlanningBlocker(crunch_build_planning_core::BuildPlanningBlocker),
    EffectNotInPlan,
    Adapter(E),
}

pub fn execute_planned_effect<T, E, F>(
    decision: &crunch_build_planning_core::BuildPlanningDecision,
    effect: &crunch_build_planning_core::BuildPlanningEffect,
    current_facts_blake3: &str,
    executor: F,
) -> Result<(T, crunch_build_planning_core::AcceptedEffectObservation), PlannedEffectExecutionError<E>>
where
    F: FnOnce(
        &crunch_build_planning_core::BuildPlanningEffect,
    ) -> Result<(T, crunch_build_planning_core::BuildPlanningEffectObservation), E>,
{
    crunch_build_planning_core::validate_plan_freshness(decision, current_facts_blake3)
        .map_err(PlannedEffectExecutionError::PlanningBlocker)?;
    if !decision.effects.iter().any(|planned| planned == effect) {
        return Err(PlannedEffectExecutionError::EffectNotInPlan);
    }
    let (value, observation) = executor(effect).map_err(PlannedEffectExecutionError::Adapter)?;
    let accepted = crunch_build_planning_core::accept_effect_observation(effect, observation)
        .map_err(PlannedEffectExecutionError::PlanningBlocker)?;
    debug_assert_eq!(accepted.effect_id_blake3, effect.effect_id_blake3);
    debug_assert_eq!(effect.selected_route, decision.route_plan.selected_route);
    Ok((value, accepted))
}
