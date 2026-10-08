//! Root-scoped action authority and execution reconciliation for the source-built proof.
//!
//! This module accepts enumerated adapter facts and observed events; it does not
//! discover actions, launch processes, hash files, or infer authority from paths.

use std::collections::{BTreeMap, BTreeSet};
use std::io;

use serde::{Deserialize, Serialize};

use crate::source_built_fixed_point::{
    ProofOutputRole, SourceBuiltFixedPointPlan, SourceBuiltFixedPointStagePlan, StageAuthorityInput,
    validate_source_built_fixed_point_plan,
};

pub(crate) const ACTION_TRUST_SCHEMA: &str = "mantle-source-built-action-trust-v1";
pub(crate) const ACTION_RECONCILIATION_SCHEMA: &str = "mantle-source-built-action-reconciliation-v1";
const ACTION_LIMIT: usize = 262_144;
const DIGEST_LENGTH: usize = 64;
const ACTION_PLAN_DOMAIN: &str = "mantle-source-built-action-trust-v1";
const RECONCILIATION_DOMAIN: &str = "mantle-source-built-action-reconciliation-v1";

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
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ActionSpec {
    pub(crate) id: String,
    pub(crate) stage_id: String,
    pub(crate) producer_actions: Vec<String>,
    pub(crate) executable: ActionExecutable,
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

/// The adapter's independently enumerated reachable IDs must equal its concrete
/// action definitions. An unsupported action kind cannot be silently omitted.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StageActionAdapter {
    pub(crate) stage_id: String,
    pub(crate) reachable_action_ids: Vec<String>,
    pub(crate) actions: Vec<ActionSpec>,
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

fn action_plan_digest(plan: &ActionTrustPlan) -> Result<String, String> {
    payload_digest(ACTION_PLAN_DOMAIN, &ActionPlanDigestPayload {
        schema: &plan.schema,
        proof_plan_digest_blake3: &plan.proof_plan_digest_blake3,
        proof_root_identity_blake3: &plan.proof_root_identity_blake3,
        adapters: &plan.adapters,
        digest_blake3: "",
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ExecutionEventKind {
    ProtectedExec,
    BuildExecution,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ActionExecutionEvent {
    pub(crate) action_id: String,
    pub(crate) kind: ExecutionEventKind,
    pub(crate) executable_digest_blake3: String,
    pub(crate) executable_producer: Option<(String, String)>,
    pub(crate) inputs: Vec<ActionInput>,
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

// r[impl bootstrap_inventory.source_built_mantle_fixed_point]
pub(crate) fn plan_actions(
    proof: &SourceBuiltFixedPointPlan,
    proof_root_identity_blake3: String,
    adapters: Vec<StageActionAdapter>,
) -> Result<ActionTrustPlan, String> {
    validate_source_built_fixed_point_plan(proof).map_err(|error| error.to_string())?;
    valid_digest(&proof_root_identity_blake3)?;
    let mut plan = ActionTrustPlan {
        schema: ACTION_TRUST_SCHEMA.into(),
        proof_plan_digest_blake3: proof.plan_digest_blake3.clone(),
        proof_root_identity_blake3,
        adapters,
        digest_blake3: String::new(),
    };
    validate_action_graph(proof, &plan)?;
    plan.digest_blake3 = action_plan_digest(&plan)?;
    Ok(plan)
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

fn validate_action_graph(proof: &SourceBuiltFixedPointPlan, plan: &ActionTrustPlan) -> Result<(), String> {
    if plan.schema != ACTION_TRUST_SCHEMA || plan.proof_plan_digest_blake3 != proof.plan_digest_blake3 {
        return Err("action trust schema or source proof identity mismatch".into());
    }
    valid_digest(&plan.proof_root_identity_blake3)?;
    if plan.adapters.len() != proof.stages.len() {
        return Err("incomplete six-stage action adapters".into());
    }
    let mut actions = BTreeMap::new();
    let mut output_producers = BTreeMap::<String, String>::new();
    let mut total_events_max = 0u32;
    for (stage_index, (stage, adapter)) in proof.stages.iter().zip(&plan.adapters).enumerate() {
        if adapter.stage_id != stage.stage_id || adapter.actions.is_empty() {
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
        validate_stage_authorities(stage, adapter, &output_producers)?;
        for action in &adapter.actions {
            validate_action(proof, stage_index, action, &actions)?;
            let count = action.protected_events_max.checked_add(action.build_events_max).ok_or("event bound overflow")?;
            total_events_max = total_events_max.checked_add(count).ok_or("event bound overflow")?;
            if actions.insert(action.id.clone(), (stage_index, action)).is_some() {
                return Err(format!("duplicate root-scoped action {}", action.id));
            }
        }
        let stage_output_id = stage_output_id(stage.output);
        let mut matching = adapter.actions.iter().filter(|action| action.output_ids.iter().any(|id| id == stage_output_id));
        let Some(producer) = matching.next() else {
            return Err(format!("stage {} lacks producing action for {stage_output_id}", stage.stage_id));
        };
        if matching.next().is_some() {
            return Err(format!("stage {} has duplicate producing actions for {stage_output_id}", stage.stage_id));
        }
        output_producers.insert(stage_output_id.to_string(), producer.id.clone());
    }
    if total_events_max > proof.resource_bounds.protected_exec_events_max {
        return Err("planned action event count exceeds proof bound".into());
    }
    Ok(())
}

fn validate_stage_authorities(
    stage: &SourceBuiltFixedPointStagePlan,
    adapter: &StageActionAdapter,
    output_producers: &BTreeMap<String, String>,
) -> Result<(), String> {
    for authority in &stage.inputs {
        if let StageAuthorityInput::Output { role } = authority {
            let output_id = stage_output_id(*role);
            let producer_id = output_producers
                .get(output_id)
                .ok_or_else(|| format!("stage {} lacks prior produced authority {output_id}", stage.stage_id))?;
            if !adapter.actions.iter().any(|action| {
                action.inputs.iter().any(|input| matches!(input,
                    ActionInput::Produced { action_id, output_id: input_id }
                        if action_id == producer_id && input_id == output_id))
            }) {
                return Err(format!("stage {} does not consume produced authority {output_id}", stage.stage_id));
            }
            if *role == ProofOutputRole::MantleStage1 && !adapter.actions.iter().any(|action| {
                matches!(&action.executable, ActionExecutable::Produced { action_id, output_id: executable_id }
                    if action_id == producer_id && executable_id == output_id)
            }) {
                return Err("stage2 must launch the produced stage1 Mantle executable".into());
            }
        } else if !adapter.actions.iter().any(|action| {
            action.inputs.contains(&ActionInput::Stage { authority: *authority })
        }) {
            return Err(format!("stage {} action adapter omits declared source or policy authority", stage.stage_id));
        }
    }
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
    if action.elapsed_seconds_max == 0 || action.elapsed_seconds_max > proof.resource_bounds.elapsed_seconds_max
        || action.disk_bytes_max == 0 || action.disk_bytes_max > proof.resource_bounds.disk_bytes_max
        || action.build_events_min == 0
        || action.protected_events_min > action.protected_events_max
        || action.build_events_min > action.build_events_max
    {
        return Err(format!("action {} has invalid build/launch event or resource bounds", action.id));
    }
    if !strictly_sorted(&action.producer_actions) || !strictly_sorted(&action.output_ids)
        || action.output_ids.is_empty() || action.inputs.is_empty()
    {
        return Err(format!("action {} has incomplete input, producer, or output authority", action.id));
    }
    let mut needed_producers = BTreeSet::new();
    if let ActionExecutable::Fixed { digest_blake3 } = &action.executable {
        valid_digest(digest_blake3)?;
    } else if let ActionExecutable::Produced { action_id, output_id } = &action.executable {
        validate_produced_authority(action_id, output_id, stage_index, preceding)?;
        needed_producers.insert(action_id.as_str());
    }
    for input in &action.inputs {
        match input {
            ActionInput::Stage { authority } if stage.inputs.contains(authority)
                && !matches!(authority, StageAuthorityInput::Output { .. }) => {}
            ActionInput::Stage { .. } => return Err(format!("action {} has undeclared or path-only stage input", action.id)),
            ActionInput::Produced { action_id, output_id } => {
                validate_produced_authority(action_id, output_id, stage_index, preceding)?;
                needed_producers.insert(action_id);
            }
        }
    }
    let declared_producers = action.producer_actions.iter().map(String::as_str).collect::<BTreeSet<_>>();
    if needed_producers != declared_producers {
        return Err(format!("action {} has missing or unused producer edges", action.id));
    }
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

fn validate_produced_authority(
    producer_id: &str,
    output_id: &str,
    consumer_stage: usize,
    preceding: &BTreeMap<String, (usize, &ActionSpec)>,
) -> Result<(), String> {
    let Some((producer_stage, producer)) = preceding.get(producer_id) else {
        return Err(format!("generated executable/input has no producer action: {producer_id}"));
    };
    if *producer_stage > consumer_stage || !producer.output_ids.iter().any(|id| id == output_id) {
        return Err(format!("generated executable/input lacks producer output: {producer_id}/{output_id}"));
    }
    Ok(())
}

// r[impl bootstrap_inventory.source_built_mantle_fixed_point]
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
    let actions = plan.adapters.iter().flat_map(|adapter| &adapter.actions)
        .map(|action| (action.id.as_str(), action)).collect::<BTreeMap<_, _>>();
    let mut counts = BTreeMap::<&str, (u32, u32)>::new();
    let mut produced = BTreeMap::<(&str, &str), &str>::new();
    for event in events {
        let action = actions.get(event.action_id.as_str())
            .ok_or_else(|| format!("unknown execution event for action {}", event.action_id))?;
        validate_observation(action, event, &produced)?;
        if event.kind == ExecutionEventKind::ProtectedExec && !event.produced_outputs.is_empty() {
            return Err(format!("action {} protected launch cannot claim produced output", action.id));
        }
        if event.kind == ExecutionEventKind::BuildExecution
            && action.protected_events_min > 0
            && counts.get(action.id.as_str()).is_none_or(|(protected, _)| *protected == 0)
        {
            return Err(format!("action {} build execution lacks preceding protected launch audit", action.id));
        }
        let counts_for_action = counts.entry(action.id.as_str()).or_default();
        let count = match event.kind {
            ExecutionEventKind::ProtectedExec => &mut counts_for_action.0,
            ExecutionEventKind::BuildExecution => &mut counts_for_action.1,
        };
        *count = count.checked_add(1).ok_or("action event count overflow")?;
        if counts_for_action.0 > action.protected_events_max || counts_for_action.1 > action.build_events_max {
            return Err(format!("action {} event bound exceeded", action.id));
        }
        for (output_id, digest) in &event.produced_outputs {
            valid_digest(digest)?;
            if !action.output_ids.contains(output_id) || produced.insert((&action.id, output_id), digest).is_some() {
                return Err(format!("action {} has unknown or duplicate output", action.id));
            }
        }
    }
    let action_coverage = collect_action_coverage(&actions, &counts, &produced)?;
    let stage1_binary = observed_stage_output_digest(plan, &produced, "mantle-stage1", "mantle-stage1")?;
    let stage2_binary = observed_stage_output_digest(plan, &produced, "mantle-stage2", "mantle-stage2")?;
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
    Ok(reconciliation)
}

fn observed_stage_output_digest<'digest>(
    plan: &ActionTrustPlan,
    produced: &BTreeMap<(&str, &str), &'digest str>,
    stage_id: &str,
    output_id: &str,
) -> Result<&'digest str, String> {
    let stage = plan.adapters.iter().find(|adapter| adapter.stage_id == stage_id)
        .ok_or_else(|| format!("missing output stage {stage_id}"))?;
    let producer = stage.actions.iter().find(|action| action.output_ids.iter().any(|output| output == output_id))
        .ok_or_else(|| format!("missing producer for {stage_id}/{output_id}"))?;
    produced.get(&(producer.id.as_str(), output_id)).copied()
        .ok_or_else(|| format!("missing observed output {stage_id}/{output_id}"))
}

fn collect_action_coverage(
    actions: &BTreeMap<&str, &ActionSpec>,
    counts: &BTreeMap<&str, (u32, u32)>,
    produced: &BTreeMap<(&str, &str), &str>,
) -> Result<Vec<ActionEventCoverage>, String> {
    let mut coverage = Vec::with_capacity(actions.len());
    for (action_id, action) in actions {
        let (protected, build) = counts.get(action_id).copied().unwrap_or_default();
        if protected < action.protected_events_min || build < action.build_events_min
            || action.output_ids.iter().any(|output| !produced.contains_key(&(*action_id, output.as_str())))
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
        });
    }
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

fn validate_observation(
    action: &ActionSpec,
    event: &ActionExecutionEvent,
    produced: &BTreeMap<(&str, &str), &str>,
) -> Result<(), String> {
    valid_digest(&event.executable_digest_blake3)?;
    if event.locality != "local" || !event.completed_locally || event.cache_hit || event.live_fetch
        || event.cargo_invocation || event.fallback || event.protected_exec_violation
        || event.inputs != action.inputs || event.elapsed_seconds > action.elapsed_seconds_max
        || event.disk_bytes > action.disk_bytes_max
    {
        return Err(format!("action {} execution violates local source and effect authority", action.id));
    }
    match &action.executable {
        ActionExecutable::Fixed { digest_blake3 } if event.executable_producer.is_none()
            && &event.executable_digest_blake3 == digest_blake3 => Ok(()),
        ActionExecutable::Produced { action_id, output_id }
            if event.executable_producer.as_ref().is_some_and(|(producer, output)| producer == action_id && output == output_id)
                && produced.get(&(action_id.as_str(), output_id.as_str())).copied()
                    == Some(event.executable_digest_blake3.as_str()) => Ok(()),
        _ => Err(format!("action {} executable authority or producer digest drift", action.id)),
    }
}

fn valid_digest(digest: &str) -> Result<(), String> {
    if digest.len() != DIGEST_LENGTH || !digest.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)) {
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
mod tests {
    use super::*;
    use crate::source_built_fixed_point::{
        InitialOutputAuthorityState, ProofHermeticityMode, SourceAuthorityInput, SourceAuthorityRole,
        SourceBuiltFixedPointPlanInput, SourceBuiltFixedPointPolicies, SourceBuiltFixedPointResourceBounds,
        SourceContentKind, plan_source_built_fixed_point,
    };

    fn digest(character: char) -> String {
        character.to_string().repeat(DIGEST_LENGTH)
    }

    // Structural decisions only: these event facts do not represent a launched
    // process and cannot authorize a deterministic v2 proof receipt.
    fn fixture() -> (SourceBuiltFixedPointPlan, Vec<StageActionAdapter>, Vec<ActionExecutionEvent>) {
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
            let id = format!("action-{index}");
            let output_id = stage_output_id(stage.output).to_string();
            let inputs = stage
                .inputs
                .iter()
                .map(|authority| match authority {
                    StageAuthorityInput::Output { role } => {
                        let output = stage_output_id(*role);
                        ActionInput::Produced { action_id: producers[output].clone(), output_id: output.to_string() }
                    }
                    _ => ActionInput::Stage { authority: *authority },
                })
                .collect::<Vec<_>>();
            let mut producer_actions = inputs.iter().filter_map(|input| match input {
                ActionInput::Produced { action_id, .. } => Some(action_id.clone()),
                ActionInput::Stage { .. } => None,
            }).collect::<Vec<_>>();
            producer_actions.sort();
            producer_actions.dedup();
            let executable = if index == 5 {
                ActionExecutable::Produced { action_id: "action-4".into(), output_id: "mantle-stage1".into() }
            } else {
                ActionExecutable::Fixed { digest_blake3: digest('a') }
            };
            let event = ActionExecutionEvent {
                action_id: id.clone(),
                kind: ExecutionEventKind::BuildExecution,
                executable_digest_blake3: if index == 5 { digest('b') } else { digest('a') },
                executable_producer: if index == 5 { Some(("action-4".into(), "mantle-stage1".into())) } else { None },
                inputs: inputs.clone(),
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

    #[test]
    fn reconciles_synthetic_six_action_graph_without_proof_claim() {
        let (proof, adapters, events) = fixture();
        let plan = plan_actions(&proof, digest('f'), adapters).unwrap();
        let reconciliation = reconcile_actions(&proof, &plan, &events).unwrap();
        assert_eq!(reconciliation.observed_event_count, 12);
        assert_eq!(reconciliation.action_coverage.len(), 6);
        assert!(reconciliation.action_coverage.iter().all(|action| action.protected_observed == 1
            && action.build_observed == 1));
        assert_ne!(reconciliation.digest_blake3, plan.digest_blake3);
        verify_action_reconciliation(&proof, &plan, &events, &reconciliation).unwrap();
        let mut altered = reconciliation.clone();
        altered.action_coverage[0].protected_observed = 0;
        assert!(verify_action_reconciliation(&proof, &plan, &events, &altered).unwrap_err().contains("independently recomputed"));
    }

    #[test]
    fn requires_protected_and_build_events_for_supervised_action() {
        let (proof, adapters, events) = fixture();
        let plan = plan_actions(&proof, digest('f'), adapters).unwrap();
        let mut missing_protected = events.clone();
        missing_protected.remove(0);
        assert!(reconcile_actions(&proof, &plan, &missing_protected).unwrap_err().contains("preceding protected launch"));
        let mut missing_build = events.clone();
        missing_build.remove(1);
        assert!(reconcile_actions(&proof, &plan, &missing_build).unwrap_err().contains("missing"));
        let mut denial = events;
        denial[0].protected_exec_violation = true;
        assert!(reconcile_actions(&proof, &plan, &denial).unwrap_err().contains("authority"));
        let (proof, adapters, mut events) = fixture();
        let plan = plan_actions(&proof, digest('f'), adapters).unwrap();
        events[0].produced_outputs = events[1].produced_outputs.clone();
        assert!(reconcile_actions(&proof, &plan, &events).unwrap_err().contains("cannot claim produced output"));
    }

    #[test]
    fn publication_without_a_child_requires_its_real_build_event_not_a_fictional_launch() {
        let (proof, mut adapters, mut events) = fixture();
        adapters[1].actions[0].protected_events_min = 0;
        adapters[1].actions[0].protected_events_max = 0;
        events.remove(2);
        let plan = plan_actions(&proof, digest('f'), adapters).unwrap();
        let reconciliation = reconcile_actions(&proof, &plan, &events).unwrap();
        assert_eq!(reconciliation.action_coverage[1].protected_observed, 0);
        assert_eq!(reconciliation.action_coverage[1].build_observed, 1);
    }

    #[test]
    fn rejects_incomplete_adapter_and_path_only_generated_authority() {
        let (proof, mut adapters, _) = fixture();
        adapters[0].reachable_action_ids.push("unknown-action".into());
        assert!(plan_actions(&proof, digest('f'), adapters).unwrap_err().contains("incomplete action adapter"));
        let (proof, mut adapters, _) = fixture();
        adapters[5].actions[0].executable = ActionExecutable::Fixed { digest_blake3: digest('a') };
        assert!(plan_actions(&proof, digest('f'), adapters).unwrap_err().contains("stage2 must launch"));
        let (proof, mut adapters, _) = fixture();
        adapters[5].actions[0].executable = ActionExecutable::Produced {
            action_id: "unplanned-producer".into(),
            output_id: "mantle-stage1".into(),
        };
        assert!(plan_actions(&proof, digest('f'), adapters).unwrap_err().contains("stage1 Mantle executable"));
        let (proof, mut adapters, _) = fixture();
        adapters[0].actions[0].inputs.remove(0);
        assert!(plan_actions(&proof, digest('f'), adapters).unwrap_err().contains("omits declared source or policy"));
    }

    #[test]
    fn rejects_unknown_missing_drift_remote_cache_and_effect_events() {
        let (proof, adapters, events) = fixture();
        let plan = plan_actions(&proof, digest('f'), adapters).unwrap();
        let mut unknown = events.clone();
        unknown[0].action_id = "unplanned".into();
        assert!(reconcile_actions(&proof, &plan, &unknown).unwrap_err().contains("unknown"));
        assert!(reconcile_actions(&proof, &plan, &events[..11]).unwrap_err().contains("missing"));
        let mut drift = events.clone();
        drift[11].executable_digest_blake3 = digest('c');
        assert!(reconcile_actions(&proof, &plan, &drift).unwrap_err().contains("digest drift"));
        let mut output_mismatch = events.clone();
        output_mismatch[11].produced_outputs[0].1 = digest('c');
        assert!(reconcile_actions(&proof, &plan, &output_mismatch).unwrap_err().contains("binary BLAKE3 digests differ"));
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
    fn rejects_missing_producer_edges_nonlocal_actions_and_tampered_plan() {
        let (proof, mut adapters, _) = fixture();
        adapters[5].actions[0].producer_actions.clear();
        assert!(plan_actions(&proof, digest('f'), adapters).unwrap_err().contains("producer edges"));
        let (proof, mut adapters, _) = fixture();
        adapters[0].actions[0].local_only = false;
        assert!(plan_actions(&proof, digest('f'), adapters).unwrap_err().contains("remote locality"));
        let (proof, adapters, events) = fixture();
        let mut plan = plan_actions(&proof, digest('f'), adapters).unwrap();
        plan.adapters[0].actions[0].disk_bytes_max += 1;
        assert!(reconcile_actions(&proof, &plan, &events).unwrap_err().contains("digest drift"));
    }
}
