// HARDENING-BACKLOG 2026-10-22: pre-existing tigerstyle findings in this file are
// recorded in .cairn/archive/2026-10-22-schedule-tigerstyle-hardening-backlog/evidence/
// tigerstyle-backlog-2026-10-22.log and scheduled for the standalone hardening
// pass. Scoped to the lint categories present at recording time.
#![allow(tigerstyle::explicit_defaults)]
//! Pure compact controller authority. Only observed positions are materialized.
use super::*;

/// Start, resume and finish phases that every controller declares.
const CONTROLLER_BASE_PHASE_COUNT: usize = 6;
/// The largest phase count a single declared slot can contribute.
const CONTROLLER_SLOT_PHASE_COUNT: usize = 7;
/// Start, two representative turns and finish phases bound the sample catalog.
const CONTROLLER_SAMPLE_MULTIPLIER: usize = 4;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ActionControllerRole {
    Start,
    Prepare,
    Executable,
    Command,
    Observation,
    Resume,
    Finish,
}
impl ActionControllerRole {
    fn name(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Prepare => "prepare",
            Self::Executable => "executable",
            Self::Command => "command",
            Self::Observation => "observation",
            Self::Resume => "resume",
            Self::Finish => "finish",
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ActionControllerPosition {
    pub(crate) turn: u32,
    pub(crate) slot: Option<String>,
    pub(crate) role: ActionControllerRole,
    pub(crate) sandbox: bool,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum ActionControllerRef {
    Static {
        action_id: String,
    },
    Current {
        slot: Option<String>,
        role: ActionControllerRole,
        sandbox: bool,
    },
    PreviousOrEntry,
    Entry,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ActionControllerTemplate {
    pub(crate) role: ActionControllerRole,
    pub(crate) slot: Option<String>,
    pub(crate) sandbox: bool,
    pub(crate) action: ActionSpec,
    pub(crate) references: BTreeMap<String, ActionControllerRef>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ActionControllerSpec {
    pub(crate) id: String,
    pub(crate) commands_max: u32,
    pub(crate) slots: Vec<String>,
    pub(crate) templates: Vec<ActionControllerTemplate>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) guard: Option<ActionGuard>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ActionControllerOutcome {
    pub(crate) controller_id: String,
    pub(crate) completed_turns: u32,
    pub(crate) producer_action: String,
    pub(crate) output_digests: BTreeMap<String, String>,
}
impl ActionControllerOutcome {
    pub(crate) fn digest(&self) -> Result<String, String> {
        payload_digest("mantle-source-controller-outcome-v1", self)
    }
}
fn position(turn: u32, slot: Option<String>, role: ActionControllerRole, sandbox: bool) -> ActionControllerPosition {
    ActionControllerPosition {
        turn,
        slot,
        role,
        sandbox,
    }
}
pub(crate) fn controller_action_id(
    controller: &ActionControllerSpec,
    p: &ActionControllerPosition,
) -> Result<String, String> {
    use ActionControllerRole::*;
    if p.turn > controller.commands_max
        || (!matches!(p.role, Start | Finish) && p.turn == controller.commands_max)
        || (p.role == Start && p.turn != 0)
    {
        return Err("controller position exceeds command bound".into());
    }
    let is_slot_role = matches!(p.role, Prepare | Executable | Command | Observation);
    if is_slot_role != p.slot.is_some() || p.slot.as_ref().is_some_and(|s| !controller.slots.contains(s)) {
        return Err("controller position has unknown slot".into());
    }
    let mut id = match p.role {
        Start => format!("{}/start", controller.id),
        Finish => controller.id.clone(),
        _ => format!("{}/turn/{}/{}/{}", controller.id, p.turn, p.slot.as_deref().unwrap_or("_"), p.role.name()),
    };
    if p.sandbox {
        id.push_str(".sandbox");
    }
    debug_assert!(p.turn <= controller.commands_max);
    debug_assert_eq!(is_slot_role, p.slot.is_some());
    Ok(id)
}
pub(crate) fn resolve_controller_reference(
    controller: &ActionControllerSpec,
    p: &ActionControllerPosition,
    r: &ActionControllerRef,
) -> Result<String, String> {
    use ActionControllerRef::*;
    match r {
        Static { action_id } => Ok(action_id.clone()),
        Current { slot, role, sandbox } => {
            controller_action_id(controller, &position(p.turn, slot.clone(), *role, *sandbox))
        }
        Entry => controller_action_id(controller, &position(0, None, ActionControllerRole::Start, false)),
        PreviousOrEntry => {
            if p.turn == 0 {
                let entry = position(0, None, ActionControllerRole::Start, false);
                debug_assert_eq!(entry.turn, 0);
                controller_action_id(controller, &entry)
            } else {
                let previous = position(p.turn.saturating_sub(1), None, ActionControllerRole::Resume, false);
                debug_assert!(previous.turn < p.turn);
                controller_action_id(controller, &previous)
            }
        }
    }
}
pub(crate) fn resolve_controller_action(
    controller: &ActionControllerSpec,
    p: &ActionControllerPosition,
) -> Result<ActionSpec, String> {
    let template = controller
        .templates
        .iter()
        .find(|t| t.role == p.role && t.slot == p.slot && t.sandbox == p.sandbox)
        .ok_or("unknown controller action template")?;
    let mut action = template.action.clone();
    action.id = controller_action_id(controller, p)?;
    let resolve = |id: &mut String| -> Result<(), String> {
        let reference = template
            .references
            .get(id)
            .ok_or_else(|| format!("controller template lacks typed producer reference {id}"))?;
        *id = resolve_controller_reference(controller, p, reference)?;
        Ok(())
    };
    for id in &mut action.producer_actions {
        resolve(id)?;
    }
    action.producer_actions.sort();
    debug_assert!(action.producer_actions.windows(2).all(|pair| pair[0] <= pair[1]));
    if let ActionExecutable::Produced { action_id, .. } = &mut action.executable {
        resolve(action_id)?;
    }
    for input in &mut action.inputs {
        match input {
            ActionInput::Produced { action_id, .. } | ActionInput::PhaseOutcome { action_id } => resolve(action_id)?,
            ActionInput::ControllerOutcome { controller_id } => {
                if controller_id != &controller.id {
                    return Err("foreign controller outcome".into());
                }
            }
            ActionInput::Stage { .. } => {}
        }
    }
    if let Some(guard) = &mut action.guard {
        for condition in &mut guard.conditions {
            resolve(&mut condition.action_id)?;
        }
    }
    debug_assert_eq!(action.stage_id, template.action.stage_id);
    Ok(action)
}
pub(crate) fn controller_action_position(
    controller: &ActionControllerSpec,
    id: &str,
) -> Result<Option<ActionControllerPosition>, String> {
    let (base, sandbox) = id.strip_suffix(".sandbox").map_or((id, false), |base| (base, true));
    if base == controller.id {
        return Ok(Some(position(0, None, ActionControllerRole::Finish, sandbox)));
    }
    if base == format!("{}/start", controller.id) {
        return Ok(Some(position(0, None, ActionControllerRole::Start, sandbox)));
    }
    let Some(tail) = base.strip_prefix(&format!("{}/turn/", controller.id)) else {
        return Ok(None);
    };
    let parts = tail.split('/').collect::<Vec<_>>();
    if parts.len() != 3 {
        return Err("invalid controller action identity".into());
    }
    let turn = parts[0].parse::<u32>().map_err(|_| "invalid controller ordinal")?;
    let role = match parts[2] {
        "prepare" => ActionControllerRole::Prepare,
        "executable" => ActionControllerRole::Executable,
        "command" => ActionControllerRole::Command,
        "observation" => ActionControllerRole::Observation,
        "resume" => ActionControllerRole::Resume,
        _ => return Err("unknown controller role".into()),
    };
    let p = position(
        turn,
        if role == ActionControllerRole::Resume {
            None
        } else {
            Some(parts[1].into())
        },
        role,
        sandbox,
    );
    if controller_action_id(controller, &p)? != id {
        return Err("noncanonical controller action identity".into());
    }
    debug_assert!(id.ends_with(".sandbox") == p.sandbox);
    debug_assert!(p.slot.is_some() == (p.role != ActionControllerRole::Resume));
    Ok(Some(p))
}

#[derive(Default)]
pub(super) struct ControllerObservation {
    cursor: usize,
    turn: u32,
    selected: Option<String>,
    done: bool,
    finished: bool,
    skipped: bool,
    pub(super) outcome: Option<ActionControllerOutcome>,
}
struct ControllerPhaseCursor {
    is_done: bool,
    is_entry: bool,
    cursor: usize,
}
fn phase_template<'a>(
    controller: &'a ActionControllerSpec,
    phase: &ControllerPhaseCursor,
) -> Option<&'a ActionControllerTemplate> {
    controller
        .templates
        .iter()
        .filter(|t| {
            if phase.is_done {
                t.role == ActionControllerRole::Finish
            } else if phase.is_entry {
                t.role == ActionControllerRole::Start
            } else {
                !matches!(t.role, ActionControllerRole::Start | ActionControllerRole::Finish)
            }
        })
        .nth(phase.cursor)
}
impl ActionObservationState {
    #[cfg(test)]
    pub(crate) fn controller_outcome(&self, id: &str) -> Result<ActionControllerOutcome, String> {
        self.controllers
            .get(id)
            .and_then(|s| s.outcome.clone())
            .ok_or_else(|| "controller has no authenticated done outcome".into())
    }
    pub(crate) fn record_planned(
        &mut self,
        proof: &SourceBuiltFixedPointPlan,
        plan: &ActionTrustPlan,
        event: &ActionExecutionEvent,
    ) -> Result<(), String> {
        self.record_adapters(proof, &plan.adapters, event)
    }
    pub(crate) fn record_adapters(
        &mut self,
        proof: &SourceBuiltFixedPointPlan,
        adapters: &[StageActionAdapter],
        event: &ActionExecutionEvent,
    ) -> Result<(), String> {
        if self.failed {
            return Err("action observation admission already failed".into());
        }
        if self.recorded_events >= proof.resource_bounds.protected_exec_events_max {
            self.failed = true;
            return Err("execution event count exceeds proof bound".into());
        }
        let result = self.record_resolved(proof, adapters, event);
        if result.is_err() {
            self.failed = true;
        } else {
            self.recorded_events = self.recorded_events.checked_add(1).ok_or("execution event count overflow")?;
        }
        result
    }
    fn record_resolved(
        &mut self,
        proof: &SourceBuiltFixedPointPlan,
        adapters: &[StageActionAdapter],
        event: &ActionExecutionEvent,
    ) -> Result<(), String> {
        for adapter in adapters {
            if let Some(action) = adapter.actions.iter().find(|a| a.id == event.action_id) {
                return self.record(proof, action, event);
            }
            for controller in &adapter.controllers {
                if event.action_id == controller.id && matches!(event.kind, ExecutionEventKind::GuardDecision { .. }) {
                    let control = controller_guard_action(controller)?;
                    if self.controllers.contains_key(&controller.id) {
                        return Err("controller outer guard repeats or follows entry".into());
                    }
                    self.record(proof, &control, event)?;
                    let is_skipped = self.guard_outcomes.get(&controller.id) == Some(&GuardOutcome::Skipped);
                    self.controllers.insert(controller.id.clone(), ControllerObservation {
                        skipped: is_skipped,
                        ..ControllerObservation::default()
                    });
                    self.resolved.insert(control.id.clone(), control);
                    return Ok(());
                }
                let parsed = controller_action_position(controller, &event.action_id)?;
                if parsed.is_some()
                    && controller.guard.is_some()
                    && self.guard_outcomes.get(&controller.id) != Some(&GuardOutcome::Active)
                {
                    return Err("controller entry lacks active outer guard".into());
                }
                if let Some(mut p) = parsed {
                    let state = self.controllers.entry(controller.id.clone()).or_default();
                    if state.finished || state.skipped {
                        return Err("controller finish repeated or skipped controller executed".into());
                    }
                    let is_entry = state.outcome.is_none()
                        && state.turn == 0
                        && !self.phase_outcomes.contains_key(&format!("{}/start", controller.id));
                    let expected = phase_template(controller, &ControllerPhaseCursor {
                        is_done: state.done,
                        is_entry,
                        cursor: state.cursor,
                    })
                    .ok_or("controller phase sequence exhausted")?;
                    if p.role == ActionControllerRole::Finish {
                        p.turn = state.turn;
                    }
                    if p.turn != state.turn || p.role != expected.role || p.slot != expected.slot {
                        return Err("controller action sequence gap, repeat, or premature finish".into());
                    }
                    if !p.sandbox && expected.sandbox {
                        let sandbox_id =
                            controller_action_id(controller, &position(p.turn, p.slot.clone(), p.role, true))?;
                        if self.counts.get(&sandbox_id).is_none_or(|c| c.0 == 0)
                            && !self.guard_outcomes.contains_key(&sandbox_id)
                        {
                            return Err("controller child precedes protected sandbox admission".into());
                        }
                    } else if p.sandbox != expected.sandbox {
                        return Err("unexpected controller sandbox phase".into());
                    }
                    let action = resolve_controller_action(controller, &p)?;
                    self.record(proof, &action, event)?;
                    self.resolved.entry(action.id.clone()).or_insert_with(|| action.clone());
                    if !p.sandbox && self.phase_outcomes.contains_key(&action.id) {
                        if expected.sandbox {
                            let sandbox_id =
                                controller_action_id(controller, &position(p.turn, p.slot.clone(), p.role, true))?;
                            if !self.phase_outcomes.contains_key(&sandbox_id) {
                                return Err("controller child completes before sandbox coverage".into());
                            }
                        }
                        self.advance_controller(controller, &p, &action)?;
                    }
                    return Ok(());
                }
            }
        }
        Err(format!("unknown execution event for action {}", event.action_id))
    }
    fn advance_controller(
        &mut self,
        c: &ActionControllerSpec,
        p: &ActionControllerPosition,
        action: &ActionSpec,
    ) -> Result<(), String> {
        use ActionControllerRole::*;
        let s = self.controllers.get_mut(&c.id).ok_or("missing controller state")?;
        if !p.sandbox {
            match p.role {
                Prepare => {
                    let code = *self.normal_exit_codes.get(&action.id).ok_or("prepare lacks actual normal status")?;
                    if code == 0 {
                        if s.selected.replace(p.slot.clone().ok_or("prepare lacks slot")?).is_some() {
                            return Err("controller turn selected twice".into());
                        }
                    } else if code != 1 {
                        return Err("invalid prepare continuation status".into());
                    }
                }
                Command => {
                    let is_active = self.guard_outcomes.get(&action.id) == Some(&GuardOutcome::Active);
                    if is_active != (s.selected.as_ref() == p.slot.as_ref()) {
                        return Err("controller command selection differs from prepare".into());
                    }
                }
                Start | Resume => {
                    if p.role == Resume && s.selected.is_none() {
                        return Err("controller active turn has no selected command".into());
                    }
                    let code =
                        *self.normal_exit_codes.get(&action.id).ok_or("controller continuation lacks normal status")?;
                    if code != 0 && code != 1 {
                        return Err("invalid controller continuation status".into());
                    }
                    let outputs =
                        self.produced.get(&action.id).ok_or("controller continuation lacks complete checkpoint")?;
                    if action.output_ids.iter().any(|id| !outputs.contains_key(id)) {
                        return Err("controller continuation omitted actual state output".into());
                    }
                    if p.role == Resume {
                        s.turn = s.turn.checked_add(1).ok_or("controller ordinal overflow")?;
                    }
                    s.done = code == 1;
                    if !s.done && s.turn >= c.commands_max {
                        return Err("controller command bound exhausted before done".into());
                    }
                    if s.done {
                        s.outcome = Some(ActionControllerOutcome {
                            controller_id: c.id.clone(),
                            completed_turns: s.turn,
                            producer_action: action.id.clone(),
                            output_digests: outputs.clone(),
                        });
                    }
                    s.cursor = 0;
                    s.selected = None;
                    return Ok(());
                }
                Finish => {
                    if !s.done || s.outcome.is_none() {
                        return Err("controller finish precedes actual done".into());
                    }
                    let outputs =
                        self.produced.get(&action.id).ok_or("controller finish lacks complete actual outputs")?;
                    if action.output_ids.iter().any(|id| !outputs.contains_key(id)) {
                        return Err("controller finish omitted final rooted products".into());
                    }
                    s.finished = true;
                }
                _ => {}
            }
        }
        s.cursor =
            s.cursor.checked_add(if p.role == Observation { 1 } else { 2 }).ok_or("controller phase overflow")?;
        Ok(())
    }
    pub(super) fn require_finished_controllers(&self, plan: &ActionTrustPlan) -> Result<(), String> {
        for c in plan.adapters.iter().flat_map(|a| &a.controllers) {
            if !self.controllers.get(&c.id).is_some_and(|s| s.finished || s.skipped) {
                return Err(format!("incomplete controller {}", c.id));
            }
        }
        Ok(())
    }
}

pub(crate) fn validate_controller_shape(c: &ActionControllerSpec, stage: &str) -> Result<u64, String> {
    validate_controller_identity(c, stage)?;
    validate_controller_phase_catalog(c)?;
    let mut events = u64::from(c.guard.is_some());
    for template in &c.templates {
        validate_controller_template(c, stage, template)?;
        let template_events = controller_template_events(c, template)?;
        events = events.checked_add(template_events).ok_or("controller event overflow")?;
    }
    debug_assert!(events >= u64::from(c.guard.is_some()));
    Ok(events)
}
fn validate_controller_identity(c: &ActionControllerSpec, stage: &str) -> Result<(), String> {
    let is_command_bound_invalid = c.commands_max == 0
        || c.commands_max > crate::source_built_fixed_point::SOURCE_BUILT_FIXED_POINT_EXEC_EVENT_COUNT_MAX;
    let is_declaration_invalid = c.slots.is_empty() || c.templates.len() > ACTION_LIMIT;
    let is_identity_invalid = c.id.strip_prefix(stage).and_then(|s| s.strip_prefix('/')).is_none_or(str::is_empty)
        || c.id.ends_with(".sandbox");
    let is_component_invalid = c.id.split('/').chain(c.slots.iter().map(String::as_str)).any(|s| {
        s.is_empty()
            || s == "."
            || s == ".."
            || s.len() > 256
            || !s.bytes().all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b':' | b'.'))
    });
    let is_slot_invalid = c.slots.iter().any(|s| s.contains('/'));
    let invalid_checks = [
        is_command_bound_invalid,
        is_declaration_invalid,
        is_identity_invalid,
        is_component_invalid,
        is_slot_invalid,
    ];
    if invalid_checks.iter().any(|is_invalid| *is_invalid) {
        return Err("invalid compact controller identity or declaration bound".into());
    }
    if c.slots.iter().collect::<BTreeSet<_>>().len() != c.slots.len() {
        return Err("duplicate controller slot".into());
    }
    debug_assert!(!c.slots.is_empty());
    debug_assert!(c.templates.len() <= ACTION_LIMIT);
    Ok(())
}
fn validate_controller_phase_catalog(c: &ActionControllerSpec) -> Result<(), String> {
    use ActionControllerRole::*;
    let mut expected = Vec::with_capacity(
        c.slots
            .len()
            .saturating_mul(CONTROLLER_SLOT_PHASE_COUNT)
            .saturating_add(CONTROLLER_BASE_PHASE_COUNT),
    );
    expected.extend([(Start, None, true), (Start, None, false)]);
    for slot in &c.slots {
        expected.extend([
            (Prepare, Some(slot.clone()), true),
            (Prepare, Some(slot.clone()), false),
        ]);
        if c.templates.iter().any(|t| t.role == Executable && t.slot.as_ref() == Some(slot)) {
            expected.extend([
                (Executable, Some(slot.clone()), true),
                (Executable, Some(slot.clone()), false),
            ]);
        }
        expected.extend([
            (Command, Some(slot.clone()), true),
            (Command, Some(slot.clone()), false),
            (Observation, Some(slot.clone()), false),
        ]);
    }
    expected.extend([
        (Resume, None, true),
        (Resume, None, false),
        (Finish, None, true),
        (Finish, None, false),
    ]);
    let is_catalog_phase_count_mismatch = expected.len() != c.templates.len();
    let is_catalog_order_mismatch = expected.iter().zip(&c.templates).any(|((role, slot, sandbox), template)| {
        role != &template.role || slot != &template.slot || sandbox != &template.sandbox
    });
    if is_catalog_phase_count_mismatch || is_catalog_order_mismatch {
        return Err("controller template phase catalog is incomplete or out of order".into());
    }
    debug_assert!(!expected.is_empty());
    debug_assert_eq!(expected.len(), c.templates.len());
    Ok(())
}
fn validate_controller_template(
    c: &ActionControllerSpec,
    stage: &str,
    t: &ActionControllerTemplate,
) -> Result<(), String> {
    if t.action.stage_id != stage || !t.action.local_only {
        return Err("controller template crosses stage or locality".into());
    }
    let is_build_event_mismatch = t.action.build_events_min != 1 || t.action.build_events_max != 1;
    let is_protected_event_mismatch = t.action.protected_events_min
        != u32::from(t.role != ActionControllerRole::Observation)
        || t.action.protected_events_max != t.action.protected_events_min;
    if is_build_event_mismatch || is_protected_event_mismatch {
        return Err("controller phase requires exactly one actual completion and protected launch".into());
    }
    validate_controller_template_references(c, t)?;
    validate_controller_template_checkpoint(t)?;
    validate_controller_template_observation(t)?;
    validate_controller_template_resume(c, t)?;
    validate_controller_template_guard(c, t)?;
    validate_controller_template_outcome(c, t)?;
    debug_assert_eq!(t.action.build_events_min, 1);
    debug_assert_eq!(t.action.protected_events_max, t.action.protected_events_min);
    Ok(())
}
fn validate_controller_template_references(
    c: &ActionControllerSpec,
    t: &ActionControllerTemplate,
) -> Result<(), String> {
    for reference in t.references.values() {
        if let ActionControllerRef::Static { action_id } = reference
            && (action_id == &c.id
                || action_id.starts_with(&format!("{}/", c.id))
                || action_id == &format!("{}.sandbox", c.id))
        {
            return Err("dynamic controller producer cannot be demoted to static reference".into());
        }
    }
    Ok(())
}
fn validate_controller_template_checkpoint(t: &ActionControllerTemplate) -> Result<(), String> {
    use ActionControllerRole::*;
    if matches!(t.role, Prepare | Resume | Finish)
        && !t.action.inputs.iter().any(|input| {
            matches!(input, ActionInput::Produced { action_id, output_id }
            if output_id == "checkpoint" && t.references.get(action_id) == Some(&ActionControllerRef::PreviousOrEntry))
        })
    {
        return Err("controller phase lacks exact previous-or-entry checkpoint".into());
    }
    if matches!(t.role, Start | Resume) && !t.sandbox && !t.action.output_ids.iter().any(|id| id == "checkpoint") {
        return Err("controller continuation omits authenticated checkpoint output".into());
    }
    Ok(())
}
fn validate_controller_template_observation(t: &ActionControllerTemplate) -> Result<(), String> {
    if t.role == ActionControllerRole::Observation {
        let expected = ActionControllerRef::Current {
            slot: t.slot.clone(),
            role: ActionControllerRole::Command,
            sandbox: false,
        };
        if !t.action.inputs.iter().any(|input| {
            matches!(input, ActionInput::PhaseOutcome { action_id }
            if t.references.get(action_id)==Some(&expected))
        }) {
            return Err("controller observation omits actual selected/skipped command outcome".into());
        }
    }
    Ok(())
}
fn validate_controller_template_resume(c: &ActionControllerSpec, t: &ActionControllerTemplate) -> Result<(), String> {
    if t.role == ActionControllerRole::Resume {
        for slot in &c.slots {
            let expected = ActionControllerRef::Current {
                slot: Some(slot.clone()),
                role: ActionControllerRole::Observation,
                sandbox: false,
            };
            if !t.action.inputs.iter().any(|input| {
                matches!(input, ActionInput::Produced { action_id,output_id }
                if output_id=="phase" && t.references.get(action_id)==Some(&expected))
            }) {
                return Err("controller resume omits an actual command observation".into());
            }
        }
    }
    Ok(())
}
fn validate_controller_template_guard(c: &ActionControllerSpec, t: &ActionControllerTemplate) -> Result<(), String> {
    use ActionControllerRole::*;
    if matches!(t.role, Start | Resume | Prepare) && !t.sandbox && t.action.accepted_exit_codes != [0, 1] {
        return Err("controller control must observe exact paused/done status".into());
    }
    if matches!(t.role, Command | Executable) {
        let prepare = controller_action_id(c, &position(0, t.slot.clone(), Prepare, false))?;
        let action = resolve_controller_action(c, &position(0, t.slot.clone(), t.role, t.sandbox))?;
        let has_image = c.templates.iter().any(|other| other.role == Executable && other.slot == t.slot);
        let expected_count = if t.role == Command && has_image { 2 } else { 1 };
        if action.guard.as_ref().is_none_or(|g| {
            g.conditions.len() != expected_count
                || g.conditions[0].action_id != prepare
                || g.conditions[0].accepted_codes != [0]
                || (expected_count == 2
                    && (g.conditions[1].accepted_codes != [0]
                        || controller_action_id(c, &position(0, t.slot.clone(), Executable, false)).ok().as_ref()
                            != Some(&g.conditions[1].action_id)))
        }) {
            return Err("controller selected command requires exact prepare/image predicate".into());
        }
        debug_assert!(!prepare.is_empty());
        debug_assert!(expected_count >= 1);
    } else if t.action.guard.is_some() {
        return Err("controller noncommand phase cannot be guarded".into());
    }
    Ok(())
}
fn validate_controller_template_outcome(c: &ActionControllerSpec, t: &ActionControllerTemplate) -> Result<(), String> {
    if t.role == ActionControllerRole::Finish {
        if !t.action.inputs.contains(&ActionInput::ControllerOutcome {
            controller_id: c.id.clone(),
        }) {
            return Err("controller finish lacks authenticated outcome".into());
        }
    } else if t.action.inputs.iter().any(|i| matches!(i, ActionInput::ControllerOutcome { .. })) {
        return Err("premature controller outcome consumption".into());
    }
    Ok(())
}
/// Events one template contributes across every turn it may repeat.
fn controller_template_events(c: &ActionControllerSpec, t: &ActionControllerTemplate) -> Result<u64, String> {
    let count = u64::from(t.action.protected_events_max)
        .checked_add(u64::from(t.action.build_events_max))
        .and_then(|n| n.checked_add(u64::from(t.action.guard.is_some())))
        .ok_or("controller event overflow")?;
    let repeats = if matches!(t.role, ActionControllerRole::Start | ActionControllerRole::Finish) {
        1
    } else {
        u64::from(c.commands_max)
    };
    count.checked_mul(repeats).ok_or_else(|| "controller event multiplication overflow".to_string())
}

pub(super) fn controller_finish_specs(plan: &ActionTrustPlan) -> Result<Vec<ActionSpec>, String> {
    plan.adapters
        .iter()
        .flat_map(|a| &a.controllers)
        .map(|c| {
            let mut action = resolve_controller_action(c, &position(0, None, ActionControllerRole::Finish, false))?;
            action.guard = c.guard.clone();
            Ok(action)
        })
        .collect()
}
pub(super) fn validate_controller_graph(
    proof: &SourceBuiltFixedPointPlan,
    stage_index: usize,
    c: &ActionControllerSpec,
    preceding: &BTreeMap<String, (usize, &ActionSpec)>,
) -> Result<(), String> {
    if let Some(guard) = &c.guard {
        if !(1..=8).contains(&guard.conditions.len()) {
            return Err("invalid controller outer guard size".into());
        }
        let mut seen = BTreeSet::new();
        for condition in &guard.conditions {
            let producer =
                preceding.get(&condition.action_id).ok_or("controller guard is not an admitted static producer")?.1;
            let is_code_contract_invalid = !valid_guard_codes(&condition.accepted_codes);
            if is_code_contract_invalid {
                return Err("controller guard lacks exact prior static status authority".into());
            }
            let is_duplicate_condition = !seen.insert(&condition.action_id);
            let is_self_referential =
                condition.action_id == c.id || condition.action_id.starts_with(&format!("{}/", c.id));
            let is_producer_unprotected =
                producer.protected_events_min == 0 || !producer.output_ids.iter().any(|id| id == "exit-status");
            let invalid_checks = [is_duplicate_condition, is_self_referential, is_producer_unprotected];
            if invalid_checks.iter().any(|is_invalid| *is_invalid) {
                return Err("controller guard lacks exact prior static status authority".into());
            }
        }
    }
    // Two bounded representative turns check both Entry and Previous paths.
    // This is catalog-sized, independent of commands_max.
    let mut samples = Vec::with_capacity(c.templates.len().saturating_mul(CONTROLLER_SAMPLE_MULTIPLIER));
    for t in c.templates.iter().filter(|t| t.role == ActionControllerRole::Start) {
        samples.push(resolve_controller_action(c, &position(0, t.slot.clone(), t.role, t.sandbox))?);
    }
    for turn in 0..c.commands_max.min(2) {
        for t in c
            .templates
            .iter()
            .filter(|t| !matches!(t.role, ActionControllerRole::Start | ActionControllerRole::Finish))
        {
            samples.push(resolve_controller_action(c, &position(turn, t.slot.clone(), t.role, t.sandbox))?);
        }
    }
    for t in c.templates.iter().filter(|t| t.role == ActionControllerRole::Finish) {
        samples
            .push(resolve_controller_action(c, &position(c.commands_max.min(2), t.slot.clone(), t.role, t.sandbox))?);
    }
    // Controller statuses are continuation evidence, not optional probe output
    // authority. The runtime transition separately requires complete state.
    let mut checked = Vec::with_capacity(samples.len());
    for action in &samples {
        let mut normalized = action.clone();
        normalized.inputs.retain(|i| !matches!(i, ActionInput::ControllerOutcome { .. }));
        let mut graph = preceding.clone();
        for prior in &checked {
            let prior: &ActionSpec = prior;
            graph.insert(prior.id.clone(), (stage_index, prior));
        }
        validate_action(proof, stage_index, &normalized, &graph)?;
        let p = controller_action_position(c, &action.id)?.ok_or("missing controller sample position")?;
        if !p.sandbox && matches!(p.role, ActionControllerRole::Start | ActionControllerRole::Resume) {
            normalized.accepted_exit_codes = vec![0];
        }
        checked.push(normalized);
    }
    debug_assert!(samples.len() <= c.templates.len().saturating_mul(CONTROLLER_SAMPLE_MULTIPLIER));
    debug_assert_eq!(checked.len(), samples.len());
    Ok(())
}

/// Classification only; completion authority always requires record_planned.
pub(crate) fn adapter_action_spec(adapter: &StageActionAdapter, id: &str) -> Result<Option<ActionSpec>, String> {
    if let Some(action) = adapter.actions.iter().find(|a| a.id == id) {
        return Ok(Some(action.clone()));
    }
    for controller in &adapter.controllers {
        if let Some(p) = controller_action_position(controller, id)? {
            return resolve_controller_action(controller, &p).map(Some);
        }
    }
    Ok(None)
}

/// Control metadata only: a GuardDecision cannot release executable or outputs.
pub(crate) fn controller_guard_action(c: &ActionControllerSpec) -> Result<ActionSpec, String> {
    let guard = c.guard.clone().ok_or("unguarded controller has an outer decision")?;
    let condition_count = guard.conditions.len();
    let mut action = resolve_controller_action(c, &position(0, None, ActionControllerRole::Finish, false))?;
    action.inputs = guard
        .conditions
        .iter()
        .map(|condition| ActionInput::Produced {
            action_id: condition.action_id.clone(),
            output_id: "exit-status".into(),
        })
        .collect();
    action.producer_actions = guard
        .conditions
        .iter()
        .map(|condition| condition.action_id.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    action.guard = Some(guard);
    debug_assert_eq!(action.inputs.len(), condition_count);
    debug_assert!(action.guard.is_some());
    Ok(action)
}

#[cfg(test)]
#[path = "source_built_fixed_point_action_controller_tests.rs"]
mod tests;
