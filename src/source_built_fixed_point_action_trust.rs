//! Root-scoped action authority and execution reconciliation for the source-built proof.
//!
//! This module accepts static adapter facts, compact immutable controller
//! catalogs, and observed events. It does not discover actions, launch processes,
//! hash files, or infer authority from paths.
//! Guard decisions are control evidence, not execution. A genuine accepted
//! nonzero probe grants only its measured inventory; ordinary successful actions
//! retain complete declared output coverage and skipped actions grant none.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::io;

use serde::Deserialize;
use serde::Serialize;

use crate::source_built_fixed_point::ProofOutputRole;
use crate::source_built_fixed_point::ProofPolicyRole;
use crate::source_built_fixed_point::SourceBuiltFixedPointPlan;
use crate::source_built_fixed_point::SourceBuiltFixedPointStagePlan;
use crate::source_built_fixed_point::StageAuthorityInput;
use crate::source_built_fixed_point::validate_source_built_fixed_point_plan;

#[path = "source_built_fixed_point_action_controller.rs"]
mod controller;
pub(crate) use controller::*;

pub(crate) const ACTION_TRUST_SCHEMA: &str = "mantle-source-built-action-trust-v2";
pub(crate) const ACTION_RECONCILIATION_SCHEMA: &str = "mantle-source-built-action-reconciliation-v2";
const ACTION_LIMIT: usize = 262_144;
const DIGEST_LENGTH: usize = 64;
const ACTION_PLAN_DOMAIN: &str = "mantle-source-built-action-trust-v2";
const RECONCILIATION_DOMAIN: &str = "mantle-source-built-action-reconciliation-v2";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum ActionExecutable {
    Fixed { digest_blake3: String },
    Produced { action_id: String, output_id: String },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum ActionInput {
    Stage { authority: StageAuthorityInput },
    Produced { action_id: String, output_id: String },
    PhaseOutcome { action_id: String },
    ControllerOutcome { controller_id: String },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ActionGuardCondition {
    pub(crate) action_id: String,
    pub(crate) accepted_codes: Vec<i32>,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ActionGuard {
    pub(crate) conditions: Vec<ActionGuardCondition>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum GuardOutcome {
    Active,
    Skipped,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum PhaseOutcome {
    Executed { output_digests: BTreeMap<String, String> },
    Skipped { guard_codes: Vec<i32> },
}
impl PhaseOutcome {
    pub(crate) fn digest(&self) -> Result<String, String> {
        payload_digest("mantle-source-phase-outcome-v1", self)
    }
}
pub(crate) fn valid_guard_codes(codes: &[i32]) -> bool {
    (1..=8).contains(&codes.len())
        && codes.iter().all(|code| (0..=255).contains(code))
        && codes.windows(2).all(|pair| pair[0] < pair[1])
}
fn valid_exit_codes(codes: &[i32]) -> bool {
    valid_guard_codes(codes) || (codes.len() == 256 && codes.iter().copied().eq(0..=255))
}
pub(crate) fn canonical_exit_status(code: i32) -> Result<([u8; 4], usize), String> {
    let Ok(code) = u8::try_from(code) else {
        return Err("guard status is not a normal exit code".into());
    };
    let mut bytes = [0u8; 4];
    let mut length_bytes = 0usize;
    if code >= 100 {
        bytes[length_bytes] = b'0'.saturating_add(code / 100);
        length_bytes = length_bytes.saturating_add(1);
    }
    if code >= 10 {
        bytes[length_bytes] = b'0'.saturating_add((code / 10) % 10);
        length_bytes = length_bytes.saturating_add(1);
    }
    bytes[length_bytes] = b'0'.saturating_add(code % 10);
    bytes[length_bytes.saturating_add(1)] = b'\n';
    Ok((bytes, length_bytes.saturating_add(2)))
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ActionSpec {
    pub(crate) id: String,
    pub(crate) stage_id: String,
    pub(crate) producer_actions: Vec<String>,
    pub(crate) executable: ActionExecutable,
    pub(crate) guard: Option<ActionGuard>,
    /// Empty for builtins/sandbox actions; otherwise the exact normal exit contract.
    pub(crate) accepted_exit_codes: Vec<i32>,
    pub(crate) inputs: Vec<ActionInput>,
    pub(crate) output_ids: Vec<String>,
    pub(crate) protected_events_min: u32,
    pub(crate) protected_events_max: u32,
    pub(crate) build_events_min: u32,
    pub(crate) build_events_max: u32,
    pub(crate) elapsed_seconds_max: u64,
    pub(crate) disk_bytes_max: u64,
    pub(crate) local_only: bool,
}

/// Reachable IDs enumerate only static actions and must equal their definitions.
/// Controllers declare a finite template catalog separately; no potential turn
/// is expanded or counted as executed merely because it could be selected.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StageActionAdapter {
    pub(crate) stage_id: String,
    pub(crate) reachable_action_ids: Vec<String>,
    pub(crate) actions: Vec<ActionSpec>,
    pub(crate) controllers: Vec<ActionControllerSpec>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ActionTrustPlan {
    pub(crate) schema: String,
    pub(crate) proof_plan_digest_blake3: String,
    pub(crate) proof_root_identity_blake3: String,
    pub(crate) adapters: Vec<StageActionAdapter>,
    pub(crate) digest_blake3: String,
}

#[derive(Serialize)]
struct ActionPlanDigestPayload<'plan> {
    schema: &'plan str,
    proof_plan_digest_blake3: &'plan str,
    proof_root_identity_blake3: &'plan str,
    adapters: &'plan [StageActionAdapter],
    digest_blake3: &'static str,
}

pub(super) fn action_plan_digest(plan: &ActionTrustPlan) -> Result<String, String> {
    payload_digest(ACTION_PLAN_DOMAIN, &ActionPlanDigestPayload {
        schema: &plan.schema,
        proof_plan_digest_blake3: &plan.proof_plan_digest_blake3,
        proof_root_identity_blake3: &plan.proof_root_identity_blake3,
        adapters: &plan.adapters,
        digest_blake3: "",
    })
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ExecutionEventKind {
    ProtectedExec,
    BuildExecution,
    GuardDecision {
        outcome: GuardOutcome,
        exit_codes: Vec<i32>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ActionExecutionEvent {
    pub(crate) action_id: String,
    pub(crate) kind: ExecutionEventKind,
    pub(crate) normal_exit_code: Option<i32>,
    pub(crate) executable_digest_blake3: String,
    pub(crate) executable_producer: Option<(String, String)>,
    pub(crate) inputs: Vec<ActionInput>,
    /// Measured identities in the same order as `inputs`; policy entries carry
    /// their canonical bound policy identity rather than a filesystem path.
    pub(crate) input_digests_blake3: Vec<String>,
    pub(crate) produced_outputs: Vec<(String, String)>,
    pub(crate) locality: String,
    pub(crate) completed_locally: bool,
    pub(crate) cache_hit: bool,
    pub(crate) live_fetch: bool,
    pub(crate) cargo_invocation: bool,
    pub(crate) fallback: bool,
    pub(crate) protected_exec_violation: bool,
    pub(crate) elapsed_seconds: u64,
    pub(crate) disk_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ActionEventCoverage {
    pub(crate) action_id: String,
    pub(crate) protected_planned_min: u32,
    pub(crate) protected_planned_max: u32,
    pub(crate) protected_observed: u32,
    pub(crate) build_planned_min: u32,
    pub(crate) build_planned_max: u32,
    pub(crate) build_observed: u32,
    pub(crate) guard_outcome: Option<GuardOutcome>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ActionReconciliation {
    pub(crate) schema: String,
    pub(crate) plan_digest_blake3: String,
    pub(crate) observed_event_count: u32,
    pub(crate) observed_event_digest_blake3: String,
    pub(crate) action_coverage: Vec<ActionEventCoverage>,
    pub(crate) digest_blake3: String,
}

/// Portable logical root identity: filesystem relocation grants no authority.
pub(crate) fn action_root_identity(proof: &SourceBuiltFixedPointPlan) -> Result<String, String> {
    payload_digest("mantle-source-built-action-root-v1", &(&proof.proof_id, &proof.plan_digest_blake3))
}

pub(crate) fn validate_action_plan(proof: &SourceBuiltFixedPointPlan, plan: &ActionTrustPlan) -> Result<(), String> {
    validate_source_built_fixed_point_plan(proof).map_err(|error| error.to_string())?;
    validate_action_graph(proof, plan)?;
    valid_digest(&plan.digest_blake3)?;
    if plan.digest_blake3 != action_plan_digest(plan)? {
        return Err("action trust plan digest drift".into());
    }
    Ok(())
}

struct ActionGraphContext<'a> {
    proof: &'a SourceBuiltFixedPointPlan,
    plan: &'a ActionTrustPlan,
    controller_finishes: &'a [ActionSpec],
}

struct ActionGraphState<'a> {
    actions: BTreeMap<String, (usize, &'a ActionSpec)>,
    admitted_controllers: BTreeSet<String>,
    output_producers: BTreeMap<String, String>,
    total_events_max: u32,
    controller_potential_events_max: u64,
}

pub(super) fn validate_action_graph(proof: &SourceBuiltFixedPointPlan, plan: &ActionTrustPlan) -> Result<(), String> {
    if plan.schema != ACTION_TRUST_SCHEMA || plan.proof_plan_digest_blake3 != proof.plan_digest_blake3 {
        return Err("action trust schema or source proof identity mismatch".into());
    }
    valid_digest(&plan.proof_root_identity_blake3)?;
    if plan.proof_root_identity_blake3 != action_root_identity(proof)? {
        return Err("action trust root identity differs from the selected proof".into());
    }
    if plan.adapters.len() != proof.stages.len() {
        return Err("incomplete six-stage action adapters".into());
    }
    let declaration_count = plan
        .adapters
        .iter()
        .try_fold(0usize, |count, adapter| {
            adapter.controllers.iter().try_fold(count.checked_add(adapter.actions.len())?, |count, c| {
                count.checked_add(1)?.checked_add(c.templates.len())
            })
        })
        .ok_or("action declaration count overflow")?;
    if declaration_count > ACTION_LIMIT {
        return Err("root-scoped action declaration count exceeds proof limit".into());
    }
    let controller_finishes = controller_finish_specs(plan)?;
    let context = ActionGraphContext {
        proof,
        plan,
        controller_finishes: &controller_finishes,
    };
    let mut state = ActionGraphState {
        actions: BTreeMap::new(),
        admitted_controllers: BTreeSet::new(),
        output_producers: BTreeMap::new(),
        total_events_max: 0,
        controller_potential_events_max: 0,
    };
    for (stage_index, (stage, adapter)) in proof.stages.iter().zip(&plan.adapters).enumerate() {
        validate_stage_action_graph(&context, &mut state, stage_index, stage, adapter)?;
    }
    if state.total_events_max > proof.resource_bounds.protected_exec_events_max {
        return Err("planned action event count exceeds proof bound".into());
    }
    state
        .controller_potential_events_max
        .checked_add(u64::from(state.total_events_max))
        .ok_or("total potential event bound overflow")?;
    debug_assert_eq!(plan.adapters.len(), proof.stages.len());
    debug_assert!(declaration_count <= ACTION_LIMIT);
    Ok(())
}
fn validate_stage_action_graph<'a>(
    context: &ActionGraphContext<'a>,
    state: &mut ActionGraphState<'a>,
    stage_index: usize,
    stage: &SourceBuiltFixedPointStagePlan,
    adapter: &'a StageActionAdapter,
) -> Result<(), String> {
    if adapter.stage_id != stage.stage_id || (adapter.actions.is_empty() && adapter.controllers.is_empty()) {
        return Err(format!("missing or out-of-order actions for {}", stage.stage_id));
    }
    if adapter.actions.len() > ACTION_LIMIT || !strictly_sorted(&adapter.reachable_action_ids) {
        return Err(format!("incomplete or duplicate action enumeration for {}", stage.stage_id));
    }
    let actual_ids = adapter.actions.iter().map(|action| action.id.clone()).collect::<BTreeSet<_>>();
    let reachable_ids = adapter.reachable_action_ids.iter().cloned().collect::<BTreeSet<_>>();
    if actual_ids.len() != adapter.actions.len() || actual_ids != reachable_ids {
        return Err(format!("incomplete action adapter for {}", stage.stage_id));
    }
    validate_stage_authorities(stage, adapter, &state.output_producers)?;
    for controller in &adapter.controllers {
        // Conditional template positions are not admitted execution events.
        // Validate their arithmetic without charging unobserved potential turns.
        state.controller_potential_events_max = state
            .controller_potential_events_max
            .checked_add(validate_controller_shape(controller, &stage.stage_id)?)
            .ok_or("controller potential event bound overflow")?;
        validate_controller_root_identity(context.plan, controller)?;
    }
    for action in &adapter.actions {
        validate_stage_action(context, state, stage_index, action, adapter)?;
    }
    for controller in &adapter.controllers {
        admit_controller(context, state, stage_index, controller)?;
    }
    validate_stage_output(context, state, stage, adapter)?;
    debug_assert_eq!(adapter.stage_id, stage.stage_id);
    debug_assert!(!adapter.actions.is_empty() || !adapter.controllers.is_empty());
    Ok(())
}
fn validate_stage_action<'a>(
    context: &ActionGraphContext<'a>,
    state: &mut ActionGraphState<'a>,
    stage_index: usize,
    action: &'a ActionSpec,
    adapter: &'a StageActionAdapter,
) -> Result<(), String> {
    let needed_prefix = adapter
        .controllers
        .iter()
        .rposition(|c| action.producer_actions.contains(&c.id))
        .map_or(0, |index| index.saturating_add(1));
    for controller in adapter.controllers.iter().take(needed_prefix) {
        admit_controller(context, state, stage_index, controller)?;
    }
    if action.inputs.iter().any(|i| matches!(i, ActionInput::ControllerOutcome { .. })) {
        return Err("static action cannot claim a controller outcome".into());
    }
    if state.actions.len() >= ACTION_LIMIT {
        return Err("root-scoped action count exceeds proof limit".into());
    }
    validate_action(context.proof, stage_index, action, &state.actions)?;
    let count = action
        .protected_events_max
        .checked_add(action.build_events_max)
        .and_then(|count| count.checked_add(u32::from(action.guard.is_some())))
        .ok_or("event bound overflow")?;
    state.total_events_max = state.total_events_max.checked_add(count).ok_or("event bound overflow")?;
    if state.actions.insert(action.id.clone(), (stage_index, action)).is_some() {
        return Err(format!("duplicate root-scoped action {}", action.id));
    }
    debug_assert!(state.actions.len() <= ACTION_LIMIT);
    debug_assert!(state.total_events_max >= count);
    Ok(())
}
fn admit_controller<'a>(
    context: &ActionGraphContext<'a>,
    state: &mut ActionGraphState<'a>,
    stage_index: usize,
    controller: &ActionControllerSpec,
) -> Result<(), String> {
    if !state.admitted_controllers.insert(controller.id.clone()) {
        return Ok(());
    }
    validate_controller_graph(context.proof, stage_index, controller, &state.actions)?;
    let finish = context
        .controller_finishes
        .iter()
        .find(|a| a.id == controller.id)
        .ok_or("missing controller finish template")?;
    state.actions.insert(finish.id.clone(), (stage_index, finish));
    Ok(())
}
fn validate_controller_root_identity(plan: &ActionTrustPlan, controller: &ActionControllerSpec) -> Result<(), String> {
    let has_static_identity = plan.adapters.iter().flat_map(|a| &a.actions).any(|a| {
        a.id == controller.id
            || a.id.starts_with(&format!("{}/", controller.id))
            || a.id == format!("{}.sandbox", controller.id)
    });
    let has_nested_controller_identity = plan.adapters.iter().flat_map(|a| &a.controllers).any(|c| {
        c.id != controller.id
            && (c.id.starts_with(&format!("{}/", controller.id)) || controller.id.starts_with(&format!("{}/", c.id)))
    });
    let has_duplicate_controller =
        plan.adapters.iter().flat_map(|a| &a.controllers).filter(|c| c.id == controller.id).count() != 1;
    let invalid_checks = [
        has_static_identity,
        has_nested_controller_identity,
        has_duplicate_controller,
    ];
    if invalid_checks.iter().any(|is_invalid| *is_invalid) {
        return Err("duplicate controller/static root identity".into());
    }
    debug_assert!(!has_static_identity);
    debug_assert!(!has_nested_controller_identity && !has_duplicate_controller);
    Ok(())
}
fn validate_stage_output<'a>(
    context: &ActionGraphContext<'a>,
    state: &mut ActionGraphState<'a>,
    stage: &SourceBuiltFixedPointStagePlan,
    adapter: &'a StageActionAdapter,
) -> Result<(), String> {
    let stage_output_id = stage_output_id(stage.output);
    let mut matching = adapter
        .actions
        .iter()
        .chain(context.controller_finishes.iter().filter(|a| a.stage_id == stage.stage_id))
        .filter(|action| action.output_ids.iter().any(|id| id == stage_output_id));
    let Some(producer) = matching.next() else {
        return Err(format!("stage {} lacks producing action for {stage_output_id}", stage.stage_id));
    };
    if matching.next().is_some() {
        return Err(format!("stage {} has duplicate producing actions for {stage_output_id}", stage.stage_id));
    }
    if producer.guard.is_some() {
        return Err(format!("stage {} mandatory product cannot be a conditional output", stage.stage_id));
    }
    state.output_producers.insert(stage_output_id.to_string(), producer.id.clone());
    debug_assert!(!stage_output_id.is_empty());
    debug_assert!(producer.guard.is_none());
    Ok(())
}

fn static_template_producer<'a>(
    token: &'a str,
    references: Option<&'a BTreeMap<String, ActionControllerRef>>,
) -> Option<&'a str> {
    match references {
        None => Some(token),
        Some(references) => match references.get(token) {
            Some(ActionControllerRef::Static { action_id }) => Some(action_id.as_str()),
            Some(ActionControllerRef::Current { .. }) => None,
            Some(ActionControllerRef::PreviousOrEntry) => None,
            Some(ActionControllerRef::Entry) => None,
            None => None,
        },
    }
}

fn validate_stage_authorities(
    stage: &SourceBuiltFixedPointStagePlan,
    adapter: &StageActionAdapter,
    output_producers: &BTreeMap<String, String>,
) -> Result<(), String> {
    let actions = || {
        adapter.actions.iter().map(|action| (action, None)).chain(
            adapter
                .controllers
                .iter()
                .flat_map(|c| c.templates.iter().map(|t| (&t.action, Some(&t.references)))),
        )
    };
    for authority in &stage.inputs {
        if let StageAuthorityInput::Output { role } = authority {
            let output_id = stage_output_id(*role);
            let producer_id = output_producers
                .get(output_id)
                .ok_or_else(|| format!("stage {} lacks prior produced authority {output_id}", stage.stage_id))?;
            if !actions().any(|(action,references)| {
                action.inputs.iter().any(|input| {
                    matches!(input,
                    ActionInput::Produced { action_id, output_id: input_id }
                        if static_template_producer(action_id,references) == Some(producer_id.as_str()) && input_id == output_id)
                })
            }) {
                return Err(format!("stage {} does not consume produced authority {output_id}", stage.stage_id));
            }
            if *role == ProofOutputRole::MantleStage1
                && !actions().any(|(action,references)| {
                    matches!(&action.executable, ActionExecutable::Produced { action_id, output_id: executable_id }
                    if static_template_producer(action_id,references) == Some(producer_id.as_str()) && executable_id == output_id)
                })
            {
                return Err("stage2 must launch the produced stage1 Mantle executable".into());
            }
        } else if !actions().any(|(action, _)| action.inputs.contains(&ActionInput::Stage { authority: *authority })) {
            return Err(format!("stage {} action adapter omits declared source or policy authority", stage.stage_id));
        }
    }
    debug_assert!(!adapter.actions.is_empty() || !adapter.controllers.is_empty());
    debug_assert_eq!(adapter.stage_id, stage.stage_id);
    Ok(())
}

fn validate_action<'a>(
    proof: &SourceBuiltFixedPointPlan,
    stage_index: usize,
    action: &'a ActionSpec,
    preceding: &BTreeMap<String, (usize, &'a ActionSpec)>,
) -> Result<(), String> {
    let stage = &proof.stages[stage_index];
    if action.stage_id != stage.stage_id || action.id.is_empty() || !action.local_only {
        return Err(format!("action {} has wrong stage, empty identity, or remote locality", action.id));
    }
    if action
        .id
        .strip_prefix(stage.stage_id.as_str())
        .and_then(|suffix| suffix.strip_prefix('/'))
        .is_none_or(str::is_empty)
    {
        return Err(format!("action {} is outside its stage namespace", action.id));
    }
    let is_elapsed_bound_invalid =
        action.elapsed_seconds_max == 0 || action.elapsed_seconds_max > proof.resource_bounds.elapsed_seconds_max;
    let is_disk_bound_invalid =
        action.disk_bytes_max == 0 || action.disk_bytes_max > proof.resource_bounds.disk_bytes_max;
    let is_build_event_invalid = action.build_events_min == 0
        || action.protected_events_min > action.protected_events_max
        || action.build_events_min > action.build_events_max;
    let invalid_bounds = [is_elapsed_bound_invalid, is_disk_bound_invalid, is_build_event_invalid];
    if invalid_bounds.iter().any(|is_invalid| *is_invalid) {
        return Err(format!("action {} has invalid build/launch event or resource bounds", action.id));
    }
    validate_action_exit_contract(action)?;
    let is_producer_order_valid = strictly_sorted(&action.producer_actions);
    let is_output_order_valid = strictly_sorted(&action.output_ids);
    let is_authority_complete = !action.output_ids.is_empty() && !action.inputs.is_empty();
    if !is_producer_order_valid || !is_output_order_valid || !is_authority_complete {
        return Err(format!("action {} has incomplete input, producer, or output authority", action.id));
    }
    let mut needed_producers = BTreeSet::new();
    if let ActionExecutable::Fixed { digest_blake3 } = &action.executable {
        valid_digest(digest_blake3)?;
    } else if let ActionExecutable::Produced { action_id, output_id } = &action.executable {
        validate_produced_authority(
            &ProducedOutputRef {
                action_id: action_id.as_str(),
                output_id: output_id.as_str(),
            },
            stage_index,
            preceding,
        )?;
        validate_guarded_consumption(action, action_id.as_str(), preceding[action_id].1, true)?;
        needed_producers.insert(action_id.as_str());
    }
    validate_action_inputs(action, stage, stage_index, preceding, &mut needed_producers)?;
    if let Some(guard) = &action.guard {
        validate_action_guard(action, guard, preceding)?;
    }
    let declared_producers = action.producer_actions.iter().map(String::as_str).collect::<BTreeSet<_>>();
    if needed_producers != declared_producers {
        return Err(format!("action {} has missing or unused producer edges", action.id));
    }
    debug_assert!(!action.id.is_empty());
    debug_assert_eq!(action.stage_id, stage.stage_id);
    Ok(())
}
fn validate_action_exit_contract(action: &ActionSpec) -> Result<(), String> {
    if !action.accepted_exit_codes.is_empty()
        && (!valid_exit_codes(&action.accepted_exit_codes) || action.protected_events_min == 0)
    {
        return Err("normal exit contract requires a protected command and canonical exit codes".into());
    }
    if action.output_ids.iter().any(|id| id == "exit-status") && action.accepted_exit_codes.is_empty() {
        return Err("actual status output requires an exact normal exit contract".into());
    }
    if action.accepted_exit_codes.len() == 256 && !action.output_ids.iter().any(|id| id == "exit-status") {
        return Err("normal probe observation requires actual status output".into());
    }
    if action.accepted_exit_codes.len() > 1
        && action.output_ids.iter().any(|id| id == "exit-status")
        && ["stdout", "stderr"]
            .iter()
            .any(|required| !action.output_ids.iter().any(|id| id.as_str() == *required))
    {
        return Err("failable probe requires mandatory actual stdout/stderr/status inventory".into());
    }
    debug_assert!(action.accepted_exit_codes.len() <= 256);
    debug_assert!(action.protected_events_min <= action.protected_events_max);
    Ok(())
}
fn validate_action_inputs<'a>(
    action: &'a ActionSpec,
    stage: &SourceBuiltFixedPointStagePlan,
    stage_index: usize,
    preceding: &BTreeMap<String, (usize, &'a ActionSpec)>,
    needed_producers: &mut BTreeSet<&'a str>,
) -> Result<(), String> {
    for input in &action.inputs {
        match input {
            ActionInput::Stage { authority }
                if stage.inputs.contains(authority) && !matches!(authority, StageAuthorityInput::Output { .. }) => {}
            ActionInput::Stage { .. } => {
                return Err(format!("action {} has undeclared or path-only stage input", action.id));
            }
            ActionInput::Produced { action_id, output_id } => {
                validate_produced_authority(
                    &ProducedOutputRef {
                        action_id: action_id.as_str(),
                        output_id: output_id.as_str(),
                    },
                    stage_index,
                    preceding,
                )?;
                validate_guarded_consumption(
                    action,
                    action_id.as_str(),
                    preceding[action_id].1,
                    !["stdout", "stderr", "exit-status"].contains(&output_id.as_str()),
                )?;
                needed_producers.insert(action_id.as_str());
            }
            ActionInput::PhaseOutcome { action_id } => {
                let (producer_stage, producer) = preceding.get(action_id).ok_or("unknown phase outcome producer")?;
                if *producer_stage > stage_index || producer.guard.is_none() || action_id.ends_with(".sandbox") {
                    return Err("phase outcome must observe a preceding guarded command".into());
                }
                needed_producers.insert(action_id.as_str());
            }
            ActionInput::ControllerOutcome { .. } => {
                return Err("controller outcome outside a controller finish".into());
            }
        }
    }
    debug_assert!(!action.inputs.is_empty());
    debug_assert!(!action.id.is_empty());
    Ok(())
}
fn validate_action_guard(
    action: &ActionSpec,
    guard: &ActionGuard,
    preceding: &BTreeMap<String, (usize, &ActionSpec)>,
) -> Result<(), String> {
    if !(1..=8).contains(&guard.conditions.len()) || action.protected_events_min == 0 {
        return Err("invalid guard condition count or unprotected guarded action".into());
    }
    let mut statuses = BTreeSet::new();
    for condition in &guard.conditions {
        if !valid_guard_codes(&condition.accepted_codes)
            || !statuses.insert(&condition.action_id)
            || !action.inputs.contains(&ActionInput::Produced {
                action_id: condition.action_id.clone(),
                output_id: "exit-status".into(),
            })
        {
            return Err("guard must consume exact distinct bounded status inputs".into());
        }
        let producer = preceding.get(&condition.action_id).ok_or("guard status producer is not preceding")?.1;
        if producer.protected_events_min == 0 || !producer.output_ids.iter().any(|output| output == "exit-status") {
            return Err("guard status is not a protected command result".into());
        }
    }
    debug_assert!(statuses.len() == guard.conditions.len());
    debug_assert!(action.protected_events_min > 0);
    Ok(())
}

fn validate_guarded_consumption(
    action: &ActionSpec,
    producer_id: &str,
    producer: &ActionSpec,
    ordinary_output: bool,
) -> Result<(), String> {
    let is_optional_failure = ordinary_output
        && producer.accepted_exit_codes.len() > 1
        && producer.output_ids.iter().any(|id| id == "exit-status");
    if producer.guard.is_none() && !is_optional_failure {
        return Ok(());
    }
    let guard = action
        .guard
        .as_ref()
        .ok_or("direct guarded output requires phase observation or a dominating guard")?;
    let index = guard
        .conditions
        .iter()
        .position(|condition| condition.action_id == producer_id)
        .ok_or("guarded output requires its producer's actual status predicate")?;
    if let Some(producer_guard) = &producer.guard
        && !guard.conditions[..index].starts_with(&producer_guard.conditions)
    {
        return Err("guarded output lacks dominating prior guard conditions".into());
    }
    debug_assert!(index < guard.conditions.len());
    debug_assert_eq!(guard.conditions[index].action_id, producer_id);
    Ok(())
}

fn stage_output_id(role: ProofOutputRole) -> &'static str {
    match role {
        ProofOutputRole::StagexTransition => "stagex-transition",
        ProofOutputRole::StagexProvider => "stagex-provider",
        ProofOutputRole::FullSourceNativeProvider => "full-source-native-provider",
        ProofOutputRole::FullSourceRustProvider => "full-source-rust-provider",
        ProofOutputRole::MantleStage1 => "mantle-stage1",
        ProofOutputRole::MantleStage2 => "mantle-stage2",
    }
}

struct ProducedOutputRef<'a> {
    action_id: &'a str,
    output_id: &'a str,
}

fn validate_produced_authority(
    produced: &ProducedOutputRef<'_>,
    consumer_stage: usize,
    preceding: &BTreeMap<String, (usize, &ActionSpec)>,
) -> Result<(), String> {
    let Some((producer_stage, producer)) = preceding.get(produced.action_id) else {
        return Err(format!("generated executable/input has no producer action: {}", produced.action_id));
    };
    if *producer_stage > consumer_stage || !producer.output_ids.iter().any(|id| id == produced.output_id) {
        return Err(format!(
            "generated executable/input lacks producer output: {}/{}",
            produced.action_id, produced.output_id
        ));
    }
    Ok(())
}

// r[impl bootstrap_inventory.source_built_mantle_fixed_point]
/// The same admission transition is used before a live exec is released and
/// again by portable whole-journal reconciliation. Only producers with complete
/// required event and output coverage grant dependent authority. Failure latches.
#[derive(Default)]
pub(crate) struct ActionObservationState {
    counts: BTreeMap<String, (u32, u32)>,
    provisional_outputs: BTreeMap<String, BTreeMap<String, String>>,
    produced: BTreeMap<String, BTreeMap<String, String>>,
    guard_outcomes: BTreeMap<String, GuardOutcome>,
    phase_outcomes: BTreeMap<String, PhaseOutcome>,
    normal_exit_codes: BTreeMap<String, i32>,
    controllers: BTreeMap<String, ControllerObservation>,
    resolved: BTreeMap<String, ActionSpec>,
    recorded_events: u32,
    failed: bool,
}

impl ActionObservationState {
    pub(crate) fn record(
        &mut self,
        proof: &SourceBuiltFixedPointPlan,
        action: &ActionSpec,
        event: &ActionExecutionEvent,
    ) -> Result<(), String> {
        if self.failed {
            return Err("action observation admission already failed".into());
        }
        self.failed = true;
        if event.action_id != action.id {
            return Err(format!("unknown execution event for action {}", event.action_id));
        }
        if let ExecutionEventKind::GuardDecision { outcome, exit_codes } = &event.kind {
            self.record_guard_decision(action, event, *outcome, exit_codes)?;
            self.failed = false;
            return Ok(());
        }
        if action.guard.is_some() && self.guard_outcomes.get(&action.id) != Some(&GuardOutcome::Active) {
            return Err("guarded command execution lacks an active observed decision".into());
        }
        validate_observation(proof, action, event, &ObservationInputs {
            produced: &self.produced,
            phases: &self.phase_outcomes,
            controllers: &self.controllers,
        })?;
        let previous = self.counts.get(&action.id).copied().unwrap_or_default();
        if event.kind == ExecutionEventKind::ProtectedExec && !event.produced_outputs.is_empty() {
            return Err(format!("action {} protected launch cannot claim produced output", action.id));
        }
        if event.kind == ExecutionEventKind::BuildExecution && action.protected_events_min > 0 && previous.0 == 0 {
            return Err(format!("action {} build execution lacks preceding protected launch audit", action.id));
        }
        let mut next = previous;
        let count = match &event.kind {
            ExecutionEventKind::ProtectedExec => &mut next.0,
            ExecutionEventKind::BuildExecution => &mut next.1,
            ExecutionEventKind::GuardDecision { .. } => unreachable!(),
        };
        *count = count.checked_add(1).ok_or("action event count overflow")?;
        if next.0 > action.protected_events_max || next.1 > action.build_events_max {
            return Err(format!("action {} event bound exceeded", action.id));
        }
        let mut outputs = BTreeSet::new();
        for (output_id, digest) in &event.produced_outputs {
            valid_digest(digest)?;
            let is_unknown_output = !action.output_ids.contains(output_id);
            if is_unknown_output {
                return Err(format!("action {} has unknown or duplicate output", action.id));
            }
            let is_duplicate_output = !outputs.insert(output_id);
            let is_repeated_output = self.produced.get(&action.id).is_some_and(|prior| prior.contains_key(output_id))
                || self.provisional_outputs.get(&action.id).is_some_and(|prior| prior.contains_key(output_id));
            if is_duplicate_output || is_repeated_output {
                return Err(format!("action {} has unknown or duplicate output", action.id));
            }
        }
        self.counts.insert(action.id.clone(), next);
        if let Some(code) = event.normal_exit_code {
            if self.normal_exit_codes.get(&action.id).is_some_and(|prior| *prior != code) {
                return Err("normal exit code changed across completion".into());
            }
            self.normal_exit_codes.insert(action.id.clone(), code);
        }
        if !event.produced_outputs.is_empty() {
            let outputs = self.provisional_outputs.entry(action.id.clone()).or_default();
            outputs.extend(event.produced_outputs.iter().cloned());
        }
        // Measured bytes remain provisional until the producer's entire required
        // event and output inventory is covered, not merely the requested output.
        let is_event_minimum_covered = next.0 >= action.protected_events_min && next.1 >= action.build_events_min;
        let is_inventory_complete = self.provisional_outputs.get(&action.id).is_some_and(|outputs| {
            required_inventory_present(action, outputs, self.normal_exit_codes.get(&action.id).copied())
        });
        let is_promotable = is_event_minimum_covered && is_inventory_complete;
        if is_promotable && let Some((action_id, outputs)) = self.provisional_outputs.remove_entry(&action.id) {
            self.phase_outcomes.insert(action_id.clone(), PhaseOutcome::Executed {
                output_digests: outputs.clone(),
            });
            self.produced.insert(action_id, outputs);
        }
        self.failed = false;
        Ok(())
    }
    fn record_guard_decision(
        &mut self,
        action: &ActionSpec,
        event: &ActionExecutionEvent,
        outcome: GuardOutcome,
        codes: &[i32],
    ) -> Result<(), String> {
        let guard = action.guard.as_ref().ok_or("unguarded action has a guard decision")?;
        let is_duplicate_decision =
            self.guard_outcomes.contains_key(&action.id) || self.counts.contains_key(&action.id);
        let is_code_count_invalid = codes.is_empty() || codes.len() > guard.conditions.len();
        let is_input_count_mismatch =
            event.inputs.len() != codes.len() || event.input_digests_blake3.len() != codes.len();
        let is_execution_claimed = !event.executable_digest_blake3.is_empty()
            || event.executable_producer.is_some()
            || event.normal_exit_code.is_some()
            || !event.produced_outputs.is_empty()
            || event.completed_locally;
        let is_locality_invalid = event.locality != "local"
            || event.cache_hit
            || event.live_fetch
            || event.cargo_invocation
            || event.fallback
            || event.protected_exec_violation;
        let is_resource_invalid = event.elapsed_seconds > action.elapsed_seconds_max || event.disk_bytes != 0;
        let invalid_checks = [
            is_duplicate_decision,
            is_code_count_invalid,
            is_input_count_mismatch,
            is_execution_claimed,
            is_locality_invalid,
            is_resource_invalid,
        ];
        if invalid_checks.iter().any(|is_invalid| *is_invalid) {
            return Err("guard decision claims execution, unread inputs, duplicate control or outputs".into());
        }
        let mut is_active = true;
        for (index, (condition, code)) in guard.conditions.iter().zip(codes).enumerate() {
            if !is_active {
                return Err("guard evaluated past first inactive condition".into());
            }
            let input = ActionInput::Produced {
                action_id: condition.action_id.clone(),
                output_id: "exit-status".into(),
            };
            let (bytes, length) = canonical_exit_status(*code)?;
            let hash = blake3::hash(&bytes[..length]).to_hex().to_string();
            if event.inputs[index] != input
                || event.input_digests_blake3[index] != hash
                || self.produced.get(&condition.action_id).and_then(|outputs| outputs.get("exit-status")) != Some(&hash)
            {
                return Err("guard consumes a forged or incomplete producer status".into());
            }
            is_active = condition.accepted_codes.binary_search(code).is_ok();
        }
        let is_active_mismatch =
            is_active && (codes.len() != guard.conditions.len() || outcome != GuardOutcome::Active);
        let is_skipped_mismatch = !is_active && outcome != GuardOutcome::Skipped;
        if is_active_mismatch || is_skipped_mismatch {
            return Err("guard decision differs from observed source predicate".into());
        }
        self.guard_outcomes.insert(action.id.clone(), outcome);
        if outcome == GuardOutcome::Skipped {
            self.phase_outcomes.insert(action.id.clone(), PhaseOutcome::Skipped {
                guard_codes: codes.to_vec(),
            });
        }
        Ok(())
    }
}

pub(crate) fn reconcile_actions(
    proof: &SourceBuiltFixedPointPlan,
    plan: &ActionTrustPlan,
    events: &[ActionExecutionEvent],
) -> Result<ActionReconciliation, String> {
    validate_action_plan(proof, plan)?;
    let observed_count = u32::try_from(events.len()).map_err(|_| "execution event count exceeds u32")?;
    if observed_count > proof.resource_bounds.protected_exec_events_max {
        return Err("execution event count exceeds proof bound".into());
    }
    let mut state = ActionObservationState::default();
    for event in events {
        state.record_planned(proof, plan, event)?;
    }
    state.require_finished_controllers(plan)?;
    let actions = plan
        .adapters
        .iter()
        .flat_map(|adapter| &adapter.actions)
        .chain(state.resolved.values())
        .map(|action| (action.id.as_str(), action))
        .collect::<BTreeMap<_, _>>();
    let counts = state.counts.iter().map(|(action, count)| (action.as_str(), *count)).collect();
    let produced = state
        .produced
        .iter()
        .flat_map(|(action, outputs)| {
            outputs.iter().map(move |(output, digest)| ((action.as_str(), output.as_str()), digest.as_str()))
        })
        .collect();
    let action_coverage =
        collect_action_coverage(&actions, &counts, &produced, &state.guard_outcomes, &state.normal_exit_codes)?;
    let stage1_binary = observed_stage_output_digest(plan, &produced, &StageOutputRef {
        stage_id: "mantle-stage1",
        output_id: "mantle-stage1",
    })?;
    let stage2_binary = observed_stage_output_digest(plan, &produced, &StageOutputRef {
        stage_id: "mantle-stage2",
        output_id: "mantle-stage2",
    })?;
    if stage1_binary != stage2_binary {
        return Err("stage1 and stage2 Mantle binary BLAKE3 digests differ".into());
    }
    let event_digest = payload_digest(RECONCILIATION_DOMAIN, &events)?;
    let mut reconciliation = ActionReconciliation {
        schema: ACTION_RECONCILIATION_SCHEMA.into(),
        plan_digest_blake3: plan.digest_blake3.clone(),
        observed_event_count: observed_count,
        observed_event_digest_blake3: event_digest,
        action_coverage,
        digest_blake3: String::new(),
    };
    reconciliation.digest_blake3 = payload_digest(RECONCILIATION_DOMAIN, &reconciliation)?;
    debug_assert_eq!(reconciliation.observed_event_count, observed_count);
    debug_assert!(!reconciliation.digest_blake3.is_empty());
    Ok(reconciliation)
}

struct StageOutputRef<'a> {
    stage_id: &'a str,
    output_id: &'a str,
}

fn observed_stage_output_digest<'digest>(
    plan: &ActionTrustPlan,
    produced: &BTreeMap<(&str, &str), &'digest str>,
    output: &StageOutputRef<'_>,
) -> Result<&'digest str, String> {
    let stage = plan
        .adapters
        .iter()
        .find(|adapter| adapter.stage_id == output.stage_id)
        .ok_or_else(|| format!("missing output stage {}", output.stage_id))?;
    let producer_id = stage
        .actions
        .iter()
        .find(|action| action.output_ids.iter().any(|id| id == output.output_id))
        .map(|action| action.id.as_str())
        .or_else(|| {
            stage
                .controllers
                .iter()
                .find(|controller| {
                    controller.templates.iter().any(|template| {
                        template.role == ActionControllerRole::Finish
                            && !template.sandbox
                            && template.action.output_ids.iter().any(|id| id == output.output_id)
                    })
                })
                .map(|controller| controller.id.as_str())
        })
        .ok_or_else(|| format!("missing producer for {}/{}", output.stage_id, output.output_id))?;
    let digest = produced
        .get(&(producer_id, output.output_id))
        .copied()
        .ok_or_else(|| format!("missing observed output {}/{}", output.stage_id, output.output_id))?;
    debug_assert!(!digest.is_empty());
    debug_assert!(!producer_id.is_empty());
    Ok(digest)
}

fn collect_action_coverage(
    actions: &BTreeMap<&str, &ActionSpec>,
    counts: &BTreeMap<&str, (u32, u32)>,
    produced: &BTreeMap<(&str, &str), &str>,
    guards: &BTreeMap<String, GuardOutcome>,
    normal_exit_codes: &BTreeMap<String, i32>,
) -> Result<Vec<ActionEventCoverage>, String> {
    let mut coverage = Vec::with_capacity(actions.len());
    for (action_id, action) in actions {
        let (protected, build) = counts.get(action_id).copied().unwrap_or_default();
        let guard_outcome = guards.get(*action_id).copied();
        if action.guard.is_some() && guard_outcome.is_none() {
            return Err(format!("missing observed guard decision for action {action_id}"));
        }
        let is_skipped = guard_outcome == Some(GuardOutcome::Skipped);
        if (is_skipped
            && (protected != 0
                || build != 0
                || action.output_ids.iter().any(|output| produced.contains_key(&(*action_id, output.as_str())))))
            || (!is_skipped
                && (protected < action.protected_events_min
                    || build < action.build_events_min
                    || action
                        .output_ids
                        .iter()
                        .filter(|output| output_required(action, output, normal_exit_codes.get(*action_id).copied()))
                        .any(|output| !produced.contains_key(&(*action_id, output.as_str())))))
        {
            return Err(format!("missing execution event or output for action {action_id}"));
        }
        coverage.push(ActionEventCoverage {
            action_id: (*action_id).to_string(),
            protected_planned_min: action.protected_events_min,
            protected_planned_max: action.protected_events_max,
            protected_observed: protected,
            build_planned_min: action.build_events_min,
            build_planned_max: action.build_events_max,
            build_observed: build,
            guard_outcome,
        });
    }
    debug_assert_eq!(coverage.len(), actions.len());
    debug_assert!(actions.is_empty() || !coverage.is_empty());
    Ok(coverage)
}

pub(crate) fn verify_action_reconciliation(
    proof: &SourceBuiltFixedPointPlan,
    plan: &ActionTrustPlan,
    events: &[ActionExecutionEvent],
    observed: &ActionReconciliation,
) -> Result<(), String> {
    let expected = reconcile_actions(proof, plan, events)?;
    if observed != &expected {
        return Err("stored action reconciliation differs from independently recomputed execution evidence".into());
    }
    Ok(())
}

fn output_required(action: &ActionSpec, output: &str, code: Option<i32>) -> bool {
    let is_failed_probe = action.accepted_exit_codes.len() > 1
        && code.is_some_and(|code| code != 0)
        && action.output_ids.iter().any(|id| id == "exit-status");
    !is_failed_probe || ["stdout", "stderr", "exit-status"].contains(&output)
}
fn required_inventory_present(action: &ActionSpec, outputs: &BTreeMap<String, String>, code: Option<i32>) -> bool {
    action
        .output_ids
        .iter()
        .filter(|output| output_required(action, output, code))
        .all(|output| outputs.contains_key(output))
}

struct ObservationInputs<'a> {
    produced: &'a BTreeMap<String, BTreeMap<String, String>>,
    phases: &'a BTreeMap<String, PhaseOutcome>,
    controllers: &'a BTreeMap<String, ControllerObservation>,
}

fn validate_observation(
    proof: &SourceBuiltFixedPointPlan,
    action: &ActionSpec,
    event: &ActionExecutionEvent,
    inputs: &ObservationInputs<'_>,
) -> Result<(), String> {
    valid_digest(&event.executable_digest_blake3)?;
    validate_observation_effects(action, event)?;
    validate_observation_exit_status(action, event)?;
    validate_observation_inputs(proof, action, event, inputs)?;
    validate_observation_executable(action, event, inputs.produced)?;
    debug_assert_eq!(event.inputs.len(), event.input_digests_blake3.len());
    debug_assert!(!action.id.is_empty());
    Ok(())
}
fn validate_observation_effects(action: &ActionSpec, event: &ActionExecutionEvent) -> Result<(), String> {
    let is_locality_invalid = event.locality != "local"
        || !event.completed_locally
        || event.cache_hit
        || event.live_fetch
        || event.cargo_invocation
        || event.fallback
        || event.protected_exec_violation;
    let is_input_mismatch = event.inputs != action.inputs || event.input_digests_blake3.len() != event.inputs.len();
    let is_resource_invalid =
        event.elapsed_seconds > action.elapsed_seconds_max || event.disk_bytes > action.disk_bytes_max;
    let invalid_checks = [is_locality_invalid, is_input_mismatch, is_resource_invalid];
    if invalid_checks.iter().any(|is_invalid| *is_invalid) {
        return Err(format!("action {} execution violates local source and effect authority", action.id));
    }
    Ok(())
}
fn validate_observation_exit_status(action: &ActionSpec, event: &ActionExecutionEvent) -> Result<(), String> {
    match (&event.kind, action.accepted_exit_codes.is_empty(), event.normal_exit_code) {
        (ExecutionEventKind::ProtectedExec, _, None) | (ExecutionEventKind::BuildExecution, true, None) => {}
        (ExecutionEventKind::BuildExecution, false, Some(code))
            if action.accepted_exit_codes.binary_search(&code).is_ok() =>
        {
            if action.output_ids.iter().any(|id| id == "exit-status") {
                let (bytes, length) = canonical_exit_status(code)?;
                let expected = blake3::hash(&bytes[..length]).to_hex().to_string();
                if !event.produced_outputs.iter().any(|(id, digest)| id == "exit-status" && digest == &expected) {
                    return Err("actual normal exit status differs from its reserved output".into());
                }
            }
        }
        _ => return Err("event normal exit does not satisfy exact source command policy".into()),
    }
    Ok(())
}
fn validate_observation_inputs(
    proof: &SourceBuiltFixedPointPlan,
    action: &ActionSpec,
    event: &ActionExecutionEvent,
    inputs: &ObservationInputs<'_>,
) -> Result<(), String> {
    for (input, observed_digest) in event.inputs.iter().zip(&event.input_digests_blake3) {
        valid_digest(observed_digest)?;
        let phase_digest;
        let expected_digest = match input {
            ActionInput::Stage { authority } => stage_input_digest(proof, *authority)?,
            ActionInput::Produced { action_id, output_id } => inputs
                .produced
                .get(action_id)
                .and_then(|outputs| outputs.get(output_id))
                .map(String::as_str)
                .ok_or_else(|| {
                    format!("action {} consumes unobserved producer output {action_id}/{output_id}", action.id)
                })?,
            ActionInput::PhaseOutcome { action_id } => {
                phase_digest = inputs.phases.get(action_id).ok_or("phase outcome is not observed")?.digest()?;
                phase_digest.as_str()
            }
            ActionInput::ControllerOutcome { controller_id } => {
                phase_digest = inputs
                    .controllers
                    .get(controller_id)
                    .and_then(|s| s.outcome.as_ref())
                    .ok_or("unobserved controller done outcome")?
                    .digest()?;
                phase_digest.as_str()
            }
        };
        if observed_digest != expected_digest {
            return Err(format!("action {} input content digest drift", action.id));
        }
    }
    debug_assert_eq!(event.inputs.len(), event.input_digests_blake3.len());
    debug_assert!(!action.id.is_empty());
    Ok(())
}
fn validate_observation_executable(
    action: &ActionSpec,
    event: &ActionExecutionEvent,
    produced: &BTreeMap<String, BTreeMap<String, String>>,
) -> Result<(), String> {
    match &action.executable {
        ActionExecutable::Fixed { digest_blake3 }
            if event.executable_producer.is_none() && &event.executable_digest_blake3 == digest_blake3 =>
        {
            Ok(())
        }
        ActionExecutable::Produced { action_id, output_id }
            if event
                .executable_producer
                .as_ref()
                .is_some_and(|(producer, output)| producer == action_id && output == output_id)
                && produced.get(action_id).and_then(|outputs| outputs.get(output_id)).map(String::as_str)
                    == Some(event.executable_digest_blake3.as_str()) =>
        {
            Ok(())
        }
        _ => Err(format!("action {} executable authority or producer digest drift", action.id)),
    }
}

fn stage_input_digest(proof: &SourceBuiltFixedPointPlan, authority: StageAuthorityInput) -> Result<&str, String> {
    match authority {
        StageAuthorityInput::Source { role } => proof
            .source_inputs
            .iter()
            .find(|source| source.role == role)
            .map(|source| source.digest_blake3.as_str())
            .ok_or_else(|| format!("missing source input authority {role:?}")),
        StageAuthorityInput::Policy { role } => Ok(match role {
            ProofPolicyRole::Closure => &proof.policies.closure_policy_digest_blake3,
            ProofPolicyRole::Hermeticity => &proof.policies.hermeticity_policy_digest_blake3,
            ProofPolicyRole::ProtectedExecution => &proof.policies.protected_execution_policy_digest_blake3,
            ProofPolicyRole::Effect => &proof.policies.effect_policy_digest_blake3,
            ProofPolicyRole::Normalization => &proof.policies.normalization_policy_digest_blake3,
        }),
        StageAuthorityInput::Output { .. } => Err("path-only output input lacks producer authority".into()),
    }
}

fn valid_digest(digest: &str) -> Result<(), String> {
    if digest.len() != DIGEST_LENGTH
        || !digest.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err("expected lowercase BLAKE3 digest".into());
    }
    Ok(())
}

fn strictly_sorted(values: &[String]) -> bool {
    values.windows(2).all(|pair| pair[0] < pair[1])
}

struct DigestSink<'hasher>(&'hasher mut blake3::Hasher);

impl io::Write for DigestSink<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn payload_digest<T: Serialize + ?Sized>(domain: &str, value: &T) -> Result<String, String> {
    let mut hasher = blake3::Hasher::new_derive_key(domain);
    serde_json::to_writer(DigestSink(&mut hasher), value)
        .map_err(|error| format!("serializing action evidence: {error}"))?;
    Ok(hasher.finalize().to_hex().to_string())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::source_built_fixed_point::ProofHermeticityMode;
    use crate::source_built_fixed_point::SourceAuthorityInput;
    use crate::source_built_fixed_point::SourceAuthorityRole;
    use crate::source_built_fixed_point::SourceBuiltFixedPointPolicies;
    use crate::source_built_fixed_point::SourceBuiltFixedPointResourceBounds;
    use crate::source_built_fixed_point::SourceContentKind;
    use crate::source_built_fixed_point_planning::InitialOutputAuthorityState;
    use crate::source_built_fixed_point_planning::SourceBuiltFixedPointPlanInput;
    use crate::source_built_fixed_point_planning::plan_actions;
    use crate::source_built_fixed_point_planning::plan_source_built_fixed_point;

    fn digest(character: char) -> String {
        character.to_string().repeat(DIGEST_LENGTH)
    }

    // Structural decisions only: these event facts do not represent a launched
    // process and cannot authorize a deterministic v2 proof receipt.
    pub(crate) fn fixture() -> (SourceBuiltFixedPointPlan, Vec<StageActionAdapter>, Vec<ActionExecutionEvent>) {
        let source_inputs = [
            (SourceAuthorityRole::StagexSeed, SourceContentKind::RegularFile),
            (SourceAuthorityRole::StagexLineage, SourceContentKind::RegularFile),
            (SourceAuthorityRole::StagexSourceBundle, SourceContentKind::RegularFile),
            (SourceAuthorityRole::NativeSourceBundle, SourceContentKind::RegularFile),
            (SourceAuthorityRole::RustSourceArchiveSet, SourceContentKind::Directory),
            (SourceAuthorityRole::MantleSource, SourceContentKind::Directory),
            (SourceAuthorityRole::VendorInputs, SourceContentKind::Directory),
        ]
        .into_iter()
        .enumerate()
        .map(|(index, (role, kind))| SourceAuthorityInput {
            id: format!("source-{index}"),
            role,
            kind,
            digest_blake3: digest('a'),
            size_bytes: 1,
        })
        .collect();
        let proof = plan_source_built_fixed_point(SourceBuiltFixedPointPlanInput {
            proof_id: "action-test".into(),
            logical_store_prefix: "/mantle/store".into(),
            source_inputs,
            initial_output_authority: InitialOutputAuthorityState {
                stagex_transition_entries: 0,
                native_provider_entries: 0,
                rust_provider_entries: 0,
                mantle_output_entries: 0,
            },
            policies: SourceBuiltFixedPointPolicies {
                expected_native_provider_digest_blake3: digest('a'),
                closure_policy_digest_blake3: digest('b'),
                hermeticity_policy_digest_blake3: digest('c'),
                protected_execution_policy_digest_blake3: digest('d'),
                effect_policy_digest_blake3: digest('e'),
                normalization_policy_digest_blake3: digest('f'),
                hermeticity_mode: ProofHermeticityMode::Strict,
                live_fetch_allowed: false,
                cargo_invocation_allowed: false,
                ambient_discovery_allowed: false,
                fallback_allowed: false,
                provider_cache_completion_allowed: false,
            },
            resource_bounds: SourceBuiltFixedPointResourceBounds {
                elapsed_seconds_max: 1_000,
                disk_bytes_max: 1_000,
                open_file_descriptors_max: 1_024,
                protected_exec_events_max: 64,
                source_records_max: 64,
            },
        })
        .unwrap();
        let mut producers = BTreeMap::<String, String>::new();
        let mut adapters = Vec::new();
        let mut events = Vec::new();
        for (index, stage) in proof.stages.iter().enumerate() {
            let id = format!("{}/action-{index}", stage.stage_id);
            let output_id = stage_output_id(stage.output).to_string();
            let inputs = stage
                .inputs
                .iter()
                .map(|authority| match authority {
                    StageAuthorityInput::Output { role } => {
                        let output = stage_output_id(*role);
                        ActionInput::Produced {
                            action_id: producers[output].clone(),
                            output_id: output.to_string(),
                        }
                    }
                    _ => ActionInput::Stage { authority: *authority },
                })
                .collect::<Vec<_>>();
            let mut producer_actions = inputs
                .iter()
                .filter_map(|input| match input {
                    ActionInput::Produced { action_id, .. } => Some(action_id.clone()),
                    ActionInput::Stage { .. }
                    | ActionInput::PhaseOutcome { .. }
                    | ActionInput::ControllerOutcome { .. } => None,
                })
                .collect::<Vec<_>>();
            producer_actions.sort();
            producer_actions.dedup();
            let executable = if index == 5 {
                ActionExecutable::Produced {
                    action_id: producers["mantle-stage1"].clone(),
                    output_id: "mantle-stage1".into(),
                }
            } else {
                ActionExecutable::Fixed {
                    digest_blake3: digest('a'),
                }
            };
            let event = ActionExecutionEvent {
                action_id: id.clone(),
                kind: ExecutionEventKind::BuildExecution,
                normal_exit_code: None,
                executable_digest_blake3: if index == 5 { digest('b') } else { digest('a') },
                executable_producer: if index == 5 {
                    Some((producers["mantle-stage1"].clone(), "mantle-stage1".into()))
                } else {
                    None
                },
                inputs: inputs.clone(),
                input_digests_blake3: inputs
                    .iter()
                    .map(|input| match input {
                        ActionInput::Stage { authority } => stage_input_digest(&proof, *authority).unwrap().to_string(),
                        ActionInput::Produced { .. } => digest('b'),
                        ActionInput::PhaseOutcome { .. } | ActionInput::ControllerOutcome { .. } => unreachable!(),
                    })
                    .collect(),
                produced_outputs: vec![(output_id.clone(), digest('b'))],
                locality: "local".into(),
                completed_locally: true,
                cache_hit: false,
                live_fetch: false,
                cargo_invocation: false,
                fallback: false,
                protected_exec_violation: false,
                elapsed_seconds: 1,
                disk_bytes: 1,
            };
            let action = ActionSpec {
                id: id.clone(),
                stage_id: stage.stage_id.clone(),
                producer_actions,
                executable,
                guard: None,
                accepted_exit_codes: Vec::new(),
                inputs,
                output_ids: vec![output_id.clone()],
                protected_events_min: 1,
                protected_events_max: 1,
                build_events_min: 1,
                build_events_max: 1,
                elapsed_seconds_max: 10,
                disk_bytes_max: 10,
                local_only: true,
            };
            producers.insert(output_id, id.clone());
            adapters.push(StageActionAdapter {
                stage_id: stage.stage_id.clone(),
                reachable_action_ids: vec![id],
                controllers: Vec::new(),
                actions: vec![action],
            });
            let mut protected = event.clone();
            protected.kind = ExecutionEventKind::ProtectedExec;
            protected.produced_outputs.clear();
            events.push(protected);
            events.push(event);
        }
        (proof, adapters, events)
    }

    fn multi_output_producer_fixture(
        executable_dependency: bool,
    ) -> (SourceBuiltFixedPointPlan, Vec<StageActionAdapter>, Vec<ActionExecutionEvent>) {
        let (proof, mut adapters, mut events) = fixture();
        // Model a local tool-and-metadata producer ahead of the first stage's
        // real action. Its consumer exercises executable and input admission
        // independently, without relying on an undeclared stage authority.
        let mut producer = adapters[0].actions[0].clone();
        producer.id = format!("{}/toolchain", producer.stage_id);
        producer.output_ids = vec!["tool".into(), "tool-metadata".into()];
        producer.build_events_max = 2;
        let mut output = events[1].clone();
        output.action_id = producer.id.clone();
        output.produced_outputs = vec![("tool".into(), digest('b'))];
        let mut metadata = output.clone();
        metadata.produced_outputs = vec![("tool-metadata".into(), digest('c'))];
        let mut launch = output.clone();
        launch.kind = ExecutionEventKind::ProtectedExec;
        launch.produced_outputs.clear();
        let consumer = &mut adapters[0].actions[0];
        consumer.producer_actions.push(producer.id.clone());
        if executable_dependency {
            consumer.executable = ActionExecutable::Produced {
                action_id: producer.id.clone(),
                output_id: "tool".into(),
            };
            for event in &mut events[..2] {
                event.executable_digest_blake3 = digest('b');
                event.executable_producer = Some((producer.id.clone(), "tool".into()));
            }
        } else {
            let input = ActionInput::Produced {
                action_id: producer.id.clone(),
                output_id: "tool".into(),
            };
            consumer.inputs.push(input.clone());
            for event in &mut events[..2] {
                event.inputs.push(input.clone());
                event.input_digests_blake3.push(digest('b'));
            }
        }
        adapters[0].reachable_action_ids.push(producer.id.clone());
        adapters[0].reachable_action_ids.sort();
        adapters[0].actions.insert(0, producer);
        events.splice(0..0, [launch, output, metadata]);
        (proof, adapters, events)
    }

    fn assert_live_and_sealed_accept(
        proof: &SourceBuiltFixedPointPlan,
        plan: &ActionTrustPlan,
        events: &[ActionExecutionEvent],
    ) {
        reconcile_actions(proof, plan, events).unwrap();
        let actions = plan
            .adapters
            .iter()
            .flat_map(|adapter| &adapter.actions)
            .map(|action| (action.id.as_str(), action))
            .collect::<BTreeMap<_, _>>();
        let mut live = ActionObservationState::default();
        for event in events {
            live.record(proof, actions[event.action_id.as_str()], event).unwrap();
        }
    }

    fn assert_live_and_sealed_reject(
        proof: &SourceBuiltFixedPointPlan,
        plan: &ActionTrustPlan,
        events: &[ActionExecutionEvent],
        rejected_index: usize,
        expected_error: &str,
    ) {
        let sealed_error = reconcile_actions(proof, plan, events).unwrap_err();
        assert!(sealed_error.contains(expected_error), "{sealed_error}");
        let actions = plan
            .adapters
            .iter()
            .flat_map(|adapter| &adapter.actions)
            .map(|action| (action.id.as_str(), action))
            .collect::<BTreeMap<_, _>>();
        let mut live = ActionObservationState::default();
        for event in &events[..rejected_index] {
            live.record(proof, actions[event.action_id.as_str()], event).unwrap();
        }
        let rejected = &events[rejected_index];
        assert_eq!(live.record(proof, actions[rejected.action_id.as_str()], rejected).unwrap_err(), sealed_error);
        // Even a subsequently valid completion cannot clear a denied launch or
        // invalid report and turn a filtered live history into success.
        let retry = &events[rejected_index + 1];
        assert!(live.record(proof, actions[retry.action_id.as_str()], retry).unwrap_err().contains("already failed"));
    }

    #[test]
    fn complete_producer_inventory_admits_dependent_executable_and_input() {
        for executable_dependency in [false, true] {
            let (proof, adapters, events) = multi_output_producer_fixture(executable_dependency);
            let plan = plan_actions(&proof, adapters).unwrap();
            assert_live_and_sealed_accept(&proof, &plan, &events);
        }
    }

    #[test]
    fn partial_producer_inventory_denies_dependent_executable_and_input() {
        for executable_dependency in [false, true] {
            let (proof, adapters, mut events) = multi_output_producer_fixture(executable_dependency);
            let plan = plan_actions(&proof, adapters).unwrap();
            // Tool bytes are measured and event minima are satisfied, but the
            // producer's declared metadata does not arrive until after launch.
            events.swap(2, 3);
            let expected_error = if executable_dependency {
                "producer digest drift"
            } else {
                "unobserved producer output"
            };
            assert_live_and_sealed_reject(&proof, &plan, &events, 2, expected_error);
        }
    }

    #[test]
    fn pending_producer_event_coverage_denies_dependent_executable_and_input() {
        for executable_dependency in [false, true] {
            for pending_kind in [ExecutionEventKind::ProtectedExec, ExecutionEventKind::BuildExecution] {
                let (proof, mut adapters, mut events) = multi_output_producer_fixture(executable_dependency);
                let producer = &mut adapters[0].actions[0];
                match pending_kind {
                    ExecutionEventKind::ProtectedExec => {
                        producer.protected_events_min = 2;
                        producer.protected_events_max = 2;
                    }
                    ExecutionEventKind::BuildExecution => producer.build_events_min = 2,
                    ExecutionEventKind::GuardDecision { .. } => unreachable!(),
                }
                // Report the complete inventory in the first build event, then
                // require one more launch/build observation before granting it.
                let metadata = events.remove(2);
                events[1].produced_outputs.extend(metadata.produced_outputs);
                let mut completion = events[0].clone();
                completion.kind = pending_kind;
                events.insert(2, completion);
                let plan = plan_actions(&proof, adapters).unwrap();
                assert_live_and_sealed_accept(&proof, &plan, &events);
                events.swap(2, 3);
                let expected_error = if executable_dependency {
                    "producer digest drift"
                } else {
                    "unobserved producer output"
                };
                assert_live_and_sealed_reject(&proof, &plan, &events, 2, expected_error);
            }
        }
    }

    #[test]
    fn rejects_repeated_outputs_before_and_after_producer_completion() {
        for completed in [false, true] {
            let (proof, mut adapters, mut events) = multi_output_producer_fixture(true);
            adapters[0].actions[0].build_events_max = 3;
            let plan = plan_actions(&proof, adapters).unwrap();
            let repeat = events[1].clone();
            let rejected_index = if completed { 3 } else { 2 };
            events.insert(rejected_index, repeat);
            assert_live_and_sealed_reject(&proof, &plan, &events, rejected_index, "unknown or duplicate output");
        }
    }

    #[test]
    fn rejects_unknown_output_before_producer_completion() {
        let (proof, adapters, mut events) = multi_output_producer_fixture(true);
        let plan = plan_actions(&proof, adapters).unwrap();
        events[1].produced_outputs[0].0 = "undeclared-tool".into();
        assert_live_and_sealed_reject(&proof, &plan, &events, 1, "unknown or duplicate output");
    }

    #[test]
    fn rejects_unknown_live_event_identity_and_latches_failure() {
        let (proof, adapters, events) = fixture();
        let plan = plan_actions(&proof, adapters).unwrap();
        let mut unknown = events.clone();
        unknown[0].action_id = "unplanned".into();
        let sealed_error = reconcile_actions(&proof, &plan, &unknown).unwrap_err();
        assert!(sealed_error.contains("unknown execution event"));
        let action = &plan.adapters[0].actions[0];
        let mut live = ActionObservationState::default();
        assert_eq!(live.record(&proof, action, &unknown[0]).unwrap_err(), sealed_error);
        assert!(live.record(&proof, action, &events[0]).unwrap_err().contains("already failed"));
    }

    #[test]
    fn reconciles_synthetic_six_action_graph_without_proof_claim() {
        let (proof, adapters, events) = fixture();
        let plan = plan_actions(&proof, adapters).unwrap();
        let reconciliation = reconcile_actions(&proof, &plan, &events).unwrap();
        assert_eq!(reconciliation.observed_event_count, 12);
        assert_eq!(reconciliation.action_coverage.len(), 6);
        assert!(
            reconciliation
                .action_coverage
                .iter()
                .all(|action| action.protected_observed == 1 && action.build_observed == 1)
        );
        assert_ne!(reconciliation.digest_blake3, plan.digest_blake3);
        verify_action_reconciliation(&proof, &plan, &events, &reconciliation).unwrap();
        let actions = plan
            .adapters
            .iter()
            .flat_map(|adapter| &adapter.actions)
            .map(|action| (action.id.as_str(), action))
            .collect::<BTreeMap<_, _>>();
        let mut live = ActionObservationState::default();
        for event in &events {
            live.record(&proof, actions[event.action_id.as_str()], event).unwrap();
        }
        for coverage in &reconciliation.action_coverage {
            assert_eq!(live.counts[&coverage.action_id], (coverage.protected_observed, coverage.build_observed));
        }
        assert_eq!(
            live.produced[&events[9].action_id]["mantle-stage1"],
            live.produced[&events[11].action_id]["mantle-stage2"]
        );
        let mut altered = reconciliation.clone();
        altered.action_coverage[0].protected_observed = 0;
        assert!(
            verify_action_reconciliation(&proof, &plan, &events, &altered)
                .unwrap_err()
                .contains("independently recomputed")
        );
    }

    #[test]
    fn requires_protected_and_build_events_for_supervised_action() {
        let (proof, adapters, events) = fixture();
        let plan = plan_actions(&proof, adapters).unwrap();
        let mut missing_protected = events.clone();
        missing_protected.remove(0);
        assert!(
            reconcile_actions(&proof, &plan, &missing_protected)
                .unwrap_err()
                .contains("preceding protected launch")
        );
        let mut missing_build = events.clone();
        missing_build.remove(1);
        assert!(reconcile_actions(&proof, &plan, &missing_build).unwrap_err().contains("unobserved producer output"));
        let mut denial = events;
        denial[0].protected_exec_violation = true;
        assert!(reconcile_actions(&proof, &plan, &denial).unwrap_err().contains("authority"));
        let (proof, adapters, mut events) = fixture();
        let plan = plan_actions(&proof, adapters).unwrap();
        events[0].produced_outputs = events[1].produced_outputs.clone();
        assert!(reconcile_actions(&proof, &plan, &events).unwrap_err().contains("cannot claim produced output"));
    }

    #[test]
    fn publication_without_a_child_requires_its_real_build_event_not_a_fictional_launch() {
        let (proof, mut adapters, mut events) = fixture();
        adapters[1].actions[0].protected_events_min = 0;
        adapters[1].actions[0].protected_events_max = 0;
        let publication_id = adapters[1].actions[0].id.clone();
        events.remove(2);
        let plan = plan_actions(&proof, adapters).unwrap();
        let reconciliation = reconcile_actions(&proof, &plan, &events).unwrap();
        let publication =
            reconciliation.action_coverage.iter().find(|coverage| coverage.action_id == publication_id).unwrap();
        assert_eq!(publication.protected_observed, 0);
        assert_eq!(publication.build_observed, 1);
    }

    #[test]
    fn rejects_incomplete_adapter_and_path_only_generated_authority() {
        let (proof, mut adapters, _) = fixture();
        adapters[0].reachable_action_ids.push("unknown-action".into());
        assert!(plan_actions(&proof, adapters).unwrap_err().contains("incomplete action adapter"));
        let (proof, mut adapters, _) = fixture();
        adapters[5].actions[0].executable = ActionExecutable::Fixed {
            digest_blake3: digest('a'),
        };
        assert!(plan_actions(&proof, adapters).unwrap_err().contains("stage2 must launch"));
        let (proof, mut adapters, _) = fixture();
        adapters[5].actions[0].executable = ActionExecutable::Produced {
            action_id: "unplanned-producer".into(),
            output_id: "mantle-stage1".into(),
        };
        assert!(plan_actions(&proof, adapters).unwrap_err().contains("stage1 Mantle executable"));
        let (proof, mut adapters, _) = fixture();
        adapters[0].actions[0].inputs.remove(0);
        assert!(plan_actions(&proof, adapters).unwrap_err().contains("omits declared source or policy"));
    }

    #[test]
    fn rejects_unknown_missing_drift_remote_cache_and_effect_events() {
        let (proof, adapters, events) = fixture();
        let plan = plan_actions(&proof, adapters).unwrap();
        let mut unknown = events.clone();
        unknown[0].action_id = "unplanned".into();
        assert!(reconcile_actions(&proof, &plan, &unknown).unwrap_err().contains("unknown"));
        assert!(reconcile_actions(&proof, &plan, &events[..11]).unwrap_err().contains("missing"));
        let mut drift = events.clone();
        drift[11].executable_digest_blake3 = digest('c');
        assert!(reconcile_actions(&proof, &plan, &drift).unwrap_err().contains("digest drift"));
        let mut output_mismatch = events.clone();
        output_mismatch[11].produced_outputs[0].1 = digest('c');
        assert!(
            reconcile_actions(&proof, &plan, &output_mismatch)
                .unwrap_err()
                .contains("binary BLAKE3 digests differ")
        );
        let mut remote = events.clone();
        remote[0].locality = "remote".into();
        assert!(reconcile_actions(&proof, &plan, &remote).unwrap_err().contains("local"));
        let mut cache = events.clone();
        cache[0].cache_hit = true;
        assert!(reconcile_actions(&proof, &plan, &cache).unwrap_err().contains("authority"));
        let mut fetch = events.clone();
        fetch[0].live_fetch = true;
        assert!(reconcile_actions(&proof, &plan, &fetch).unwrap_err().contains("authority"));
        let mut cargo = events.clone();
        cargo[0].cargo_invocation = true;
        assert!(reconcile_actions(&proof, &plan, &cargo).unwrap_err().contains("authority"));
        let mut fallback = events.clone();
        fallback[0].fallback = true;
        assert!(reconcile_actions(&proof, &plan, &fallback).unwrap_err().contains("authority"));
        let mut producer_drift = events.clone();
        producer_drift[11].executable_producer = Some(("action-0".into(), "stagex-transition".into()));
        assert!(reconcile_actions(&proof, &plan, &producer_drift).unwrap_err().contains("producer digest drift"));
        let mut undeclared = events.clone();
        undeclared[0].inputs.clear();
        assert!(reconcile_actions(&proof, &plan, &undeclared).unwrap_err().contains("authority"));
        let mut extra = events.clone();
        extra.insert(0, events[0].clone());
        assert!(reconcile_actions(&proof, &plan, &extra).unwrap_err().contains("event bound"));
    }

    #[test]
    fn rejects_input_consumption_before_producer_completion() {
        let (proof, adapters, mut events) = fixture();
        let plan = plan_actions(&proof, adapters).unwrap();
        // The second stage uses a fixed executable, so executable authority alone
        // cannot enforce the dependency on the transition's produced input.
        events.swap(1, 2);
        let sealed_error = reconcile_actions(&proof, &plan, &events).unwrap_err();
        assert!(sealed_error.contains("consumes unobserved producer output"));
        let actions = plan
            .adapters
            .iter()
            .flat_map(|adapter| &adapter.actions)
            .map(|action| (action.id.as_str(), action))
            .collect::<BTreeMap<_, _>>();
        let mut live = ActionObservationState::default();
        live.record(&proof, actions[events[0].action_id.as_str()], &events[0]).unwrap();
        assert_eq!(live.record(&proof, actions[events[1].action_id.as_str()], &events[1]).unwrap_err(), sealed_error);
        // Restoring the omitted producer after denial cannot erase the rejected
        // attempt and admit a filtered stream as live success.
        assert!(
            live.record(&proof, actions[events[2].action_id.as_str()], &events[2])
                .unwrap_err()
                .contains("already failed")
        );
    }

    #[test]
    fn rejects_foreign_root_even_with_recomputed_action_plan_digest() {
        let (proof, adapters, events) = fixture();
        let mut plan = plan_actions(&proof, adapters).unwrap();
        plan.proof_root_identity_blake3 = digest('f');
        plan.digest_blake3 = action_plan_digest(&plan).unwrap();
        assert!(
            reconcile_actions(&proof, &plan, &events)
                .unwrap_err()
                .contains("root identity differs from the selected proof")
        );
    }

    #[test]
    fn rejects_measured_source_and_generated_input_digest_drift() {
        let (proof, adapters, events) = fixture();
        let plan = plan_actions(&proof, adapters).unwrap();
        let mut source_drift = events.clone();
        source_drift[0].input_digests_blake3[0] = digest('c');
        assert!(reconcile_actions(&proof, &plan, &source_drift).unwrap_err().contains("input content digest drift"));
        let mut produced_drift = events.clone();
        produced_drift[2].input_digests_blake3[0] = digest('c');
        assert!(
            reconcile_actions(&proof, &plan, &produced_drift)
                .unwrap_err()
                .contains("input content digest drift")
        );
        let mut omitted = events;
        omitted[0].input_digests_blake3.clear();
        assert!(reconcile_actions(&proof, &plan, &omitted).is_err());
    }

    #[test]
    fn rejects_missing_producer_edges_nonlocal_actions_and_tampered_plan() {
        let (proof, mut adapters, _) = fixture();
        adapters[5].actions[0].producer_actions.clear();
        assert!(plan_actions(&proof, adapters).unwrap_err().contains("producer edges"));
        let (proof, mut adapters, _) = fixture();
        adapters[0].actions[0].local_only = false;
        assert!(plan_actions(&proof, adapters).unwrap_err().contains("remote locality"));
        let (proof, adapters, events) = fixture();
        let mut plan = plan_actions(&proof, adapters).unwrap();
        plan.adapters[0].actions[0].disk_bytes_max += 1;
        assert!(reconcile_actions(&proof, &plan, &events).unwrap_err().contains("digest drift"));
    }
    fn guarded_fixture(code: i32) -> (SourceBuiltFixedPointPlan, ActionTrustPlan, Vec<ActionExecutionEvent>) {
        let (proof, mut adapters, mut events) = fixture();
        let mut status = adapters[0].actions[0].clone();
        status.id = format!("{}/probe", status.stage_id);
        status.output_ids = vec!["exit-status".into(), "stderr".into(), "stdout".into()];
        status.accepted_exit_codes = vec![0, 1];
        let (bytes, length) = canonical_exit_status(code).unwrap();
        let hash = blake3::hash(&bytes[..length]).to_hex().to_string();
        let mut status_launch = events[0].clone();
        status_launch.action_id = status.id.clone();
        let mut status_done = events[1].clone();
        status_done.action_id = status.id.clone();
        status_done.produced_outputs = vec![
            ("exit-status".into(), hash.clone()),
            ("stderr".into(), digest('c')),
            ("stdout".into(), digest('d')),
        ];
        status_done.normal_exit_code = Some(code);
        let input = ActionInput::Produced {
            action_id: status.id.clone(),
            output_id: "exit-status".into(),
        };
        let mut conditional = status.clone();
        conditional.id = format!("{}/conditional", status.stage_id);
        conditional.guard = Some(ActionGuard {
            conditions: vec![ActionGuardCondition {
                action_id: status.id.clone(),
                accepted_codes: vec![0],
            }],
        });
        conditional.producer_actions = vec![status.id.clone()];
        conditional.inputs.push(input.clone());
        conditional.output_ids = vec!["artifact".into()];
        conditional.accepted_exit_codes.clear();
        let outcome = if code == 0 {
            GuardOutcome::Active
        } else {
            GuardOutcome::Skipped
        };
        let mut decision = status_launch.clone();
        decision.action_id = conditional.id.clone();
        decision.kind = ExecutionEventKind::GuardDecision {
            outcome,
            exit_codes: vec![code],
        };
        decision.executable_digest_blake3.clear();
        decision.inputs = vec![input];
        decision.input_digests_blake3 = vec![hash.clone()];
        decision.completed_locally = false;
        decision.disk_bytes = 0;
        let mut prefix = vec![status_launch.clone(), status_done, decision];
        let phase = if code == 0 {
            let mut launch = status_launch;
            launch.action_id = conditional.id.clone();
            launch.inputs = conditional.inputs.clone();
            launch.input_digests_blake3.push(hash);
            let mut done = launch.clone();
            done.kind = ExecutionEventKind::BuildExecution;
            done.produced_outputs = vec![("artifact".into(), digest('c'))];
            prefix.extend([launch, done]);
            PhaseOutcome::Executed {
                output_digests: BTreeMap::from([("artifact".into(), digest('c'))]),
            }
        } else {
            PhaseOutcome::Skipped {
                guard_codes: vec![code],
            }
        };
        let mut observer = status.clone();
        observer.id = format!("{}/observer", status.stage_id);
        observer.inputs.push(ActionInput::PhaseOutcome {
            action_id: conditional.id.clone(),
        });
        observer.producer_actions = vec![conditional.id.clone()];
        observer.output_ids = vec!["envelope".into()];
        observer.protected_events_min = 0;
        observer.protected_events_max = 0;
        observer.accepted_exit_codes.clear();
        let mut observed = events[1].clone();
        observed.action_id = observer.id.clone();
        observed.inputs = observer.inputs.clone();
        observed.input_digests_blake3.push(phase.digest().unwrap());
        observed.produced_outputs = vec![("envelope".into(), digest('d'))];
        prefix.push(observed);
        adapters[0].actions.splice(0..0, [status, conditional, observer]);
        adapters[0].reachable_action_ids = adapters[0].actions.iter().map(|action| action.id.clone()).collect();
        adapters[0].reachable_action_ids.sort();
        prefix.append(&mut events);
        let plan = plan_actions(&proof, adapters).unwrap();
        (proof, plan, prefix)
    }

    #[test]
    fn guarded_actual_status_controls_execution_and_authenticated_phase_outcome() {
        for code in [0, 1] {
            let (proof, plan, events) = guarded_fixture(code);
            assert_live_and_sealed_accept(&proof, &plan, &events);
            let reconciled = reconcile_actions(&proof, &plan, &events).unwrap();
            let conditional =
                reconciled.action_coverage.iter().find(|item| item.action_id.ends_with("/conditional")).unwrap();
            assert_eq!(
                (conditional.protected_observed, conditional.build_observed),
                if code == 0 { (1, 1) } else { (0, 0) }
            );
            assert_eq!(
                conditional.guard_outcome,
                Some(if code == 0 {
                    GuardOutcome::Active
                } else {
                    GuardOutcome::Skipped
                })
            );
        }
    }

    #[test]
    fn guarded_decision_rejects_forged_false_missing_duplicate_and_execution_claims() {
        let (proof, plan, events) = guarded_fixture(1);
        for variant in 0..6 {
            let mut altered = events.clone();
            match variant {
                0 => {
                    altered[2].kind = ExecutionEventKind::GuardDecision {
                        outcome: GuardOutcome::Active,
                        exit_codes: vec![1],
                    }
                }
                1 => {
                    altered[2].kind = ExecutionEventKind::GuardDecision {
                        outcome: GuardOutcome::Skipped,
                        exit_codes: vec![0],
                    }
                }
                2 => {
                    altered.remove(2);
                }
                3 => altered.insert(3, altered[2].clone()),
                4 => altered[2].completed_locally = true,
                _ => altered[2].produced_outputs.push(("artifact".into(), digest('c'))),
            }
            assert!(reconcile_actions(&proof, &plan, &altered).is_err(), "variant {variant}");
        }
        let (proof, plan, mut events) = guarded_fixture(0);
        events[2].kind = ExecutionEventKind::GuardDecision {
            outcome: GuardOutcome::Skipped,
            exit_codes: vec![0],
        };
        assert!(reconcile_actions(&proof, &plan, &events).is_err());
    }

    #[test]
    fn guarded_phase_outcome_cannot_be_forged_or_observed_before_decision() {
        let (proof, plan, events) = guarded_fixture(1);
        let mut forged = events.clone();
        forged[3].input_digests_blake3.last_mut().unwrap().clone_from(
            &PhaseOutcome::Executed {
                output_digests: BTreeMap::new(),
            }
            .digest()
            .unwrap(),
        );
        assert!(reconcile_actions(&proof, &plan, &forged).is_err());
        let mut early = events;
        early.swap(2, 3);
        assert!(reconcile_actions(&proof, &plan, &early).is_err());
    }

    #[test]
    fn accepted_failed_probe_grants_only_actual_outputs_and_rejects_status_drift() {
        let (proof, mut plan, events) = guarded_fixture(1);
        plan.adapters[0].actions[0].output_ids.insert(0, "absent-object".into());
        plan.digest_blake3 = action_plan_digest(&plan).unwrap();
        assert_live_and_sealed_accept(&proof, &plan, &events);
        let mut no_status = events.clone();
        no_status[1].produced_outputs.clear();
        assert!(reconcile_actions(&proof, &plan, &no_status).is_err());
        let mut false_success = events.clone();
        false_success[1].normal_exit_code = Some(0);
        assert!(reconcile_actions(&proof, &plan, &false_success).is_err());
        let mut unlisted = events;
        unlisted[1].normal_exit_code = Some(2);
        assert!(reconcile_actions(&proof, &plan, &unlisted).is_err());
        let (proof, mut plan, events) = guarded_fixture(0);
        plan.adapters[0].actions[0].output_ids.insert(0, "absent-object".into());
        plan.digest_blake3 = action_plan_digest(&plan).unwrap();
        assert!(reconcile_actions(&proof, &plan, &events).is_err());
    }

    #[test]
    fn mandatory_stage_product_cannot_escape_coverage_through_a_guard() {
        let (proof, mut plan, _) = guarded_fixture(0);
        let status = plan.adapters[0].actions[0].id.clone();
        let product = plan.adapters[0].actions.last_mut().unwrap();
        product.guard = Some(ActionGuard {
            conditions: vec![ActionGuardCondition {
                action_id: status.clone(),
                accepted_codes: vec![0],
            }],
        });
        product.inputs.push(ActionInput::Produced {
            action_id: status.clone(),
            output_id: "exit-status".into(),
        });
        product.producer_actions.push(status);
        plan.digest_blake3 = action_plan_digest(&plan).unwrap();
        assert!(validate_action_plan(&proof, &plan).err().unwrap().contains("mandatory product"));
    }
}
