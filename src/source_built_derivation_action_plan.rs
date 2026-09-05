//! Pure eager action plan for a converted Mantle derivation graph.
//!
//! Evaluation and file reads stay in the shell. This module accepts converted
//! derivations and returns a deterministic, local-only action authority plan.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt;

use nix_compat::derivation::Derivation;
use nix_compat::store_path::StorePath;
use serde::Deserialize;
use serde::Serialize;

pub(crate) const EAGER_DERIVATION_ACTION_PLAN_SCHEMA: &str = "mantle-eager-derivation-action-plan-v1";
pub(crate) const EAGER_DERIVATION_RECONCILIATION_SCHEMA: &str = "mantle-eager-derivation-reconciliation-v1";
const PLAN_DIGEST_CONTEXT: &str = "mantle-eager-derivation-action-plan-v1";
const RECONCILIATION_DIGEST_CONTEXT: &str = "mantle-eager-derivation-reconciliation-v1";
const ACTION_INPUT_DIGEST_CONTEXT: &str = "mantle-eager-derivation-action-inputs-v1";
const SANDBOX_SHELL_PATH: &str = "/bin/sh";
const BUILTIN_PREFIX: &str = "builtin:";
const BLAKE3_HEX_LENGTH: usize = 64;
const ACTION_COUNT_MAX: u32 = 16_384;
const OUTPUTS_PER_ACTION_DEBUG_MAX: usize = 8;
const TEXT_BYTES_MAX: usize = 4_096;

#[derive(Debug)]
pub(crate) struct EagerDerivationActionPlanInput {
    pub(crate) stage_id: String,
    pub(crate) store_dir: String,
    pub(crate) sandbox_shell_digest_blake3: String,
    pub(crate) entries: Vec<EagerDerivationEntry>,
}

#[derive(Debug)]
pub(crate) struct EagerDerivationEntry {
    pub(crate) drv_path: StorePath<String>,
    pub(crate) derivation: Derivation,
    pub(crate) content_addressed: bool,
    pub(crate) dynamic_plan_outputs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct EagerDerivationActionPlan {
    pub(crate) schema: String,
    pub(crate) stage_id: String,
    pub(crate) adapter: String,
    pub(crate) store_dir: String,
    pub(crate) sandbox_shell_digest_blake3: String,
    pub(crate) action_count: u32,
    pub(crate) local_only: bool,
    pub(crate) cache_only_completion_allowed: bool,
    pub(crate) actions: Vec<DerivationAction>,
    pub(crate) blockers: Vec<String>,
    pub(crate) plan_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DerivationAction {
    pub(crate) action_id: String,
    pub(crate) action_id_blake3: String,
    pub(crate) observed_goal_key_blake3: String,
    pub(crate) producer_action_ids: Vec<String>,
    pub(crate) input_source_ids: Vec<String>,
    pub(crate) outputs: Vec<DerivationActionOutput>,
    pub(crate) executable: DerivationExecutableAuthority,
    pub(crate) system: String,
    pub(crate) input_envelope_digest_blake3: String,
    pub(crate) local_only: bool,
    pub(crate) event_count_min: u32,
    pub(crate) event_count_max: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DerivationActionOutput {
    pub(crate) name: String,
    pub(crate) identity: String,
    pub(crate) content_addressed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub(crate) enum DerivationExecutableAuthority {
    Builtin {
        identity: String,
    },
    FixedSandboxShell {
        logical_path: String,
        digest_blake3: String,
    },
    Produced {
        producer_action_id: String,
        output_name: String,
        output_identity: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EagerDerivationActionPlanErrorKind {
    InvalidInput,
    IncompleteGraph,
    IncompleteReconciliation,
    UnsupportedDynamicAction,
    UnboundExecutable,
    Serialization,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EagerDerivationActionPlanError {
    pub(crate) kind: EagerDerivationActionPlanErrorKind,
    pub(crate) message: String,
}

impl fmt::Display for EagerDerivationActionPlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl std::error::Error for EagerDerivationActionPlanError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct EagerDerivationReconciliation {
    pub(crate) schema: String,
    pub(crate) action_plan_digest_blake3: String,
    pub(crate) planned_action_count: u32,
    pub(crate) matched_action_count: u32,
    pub(crate) observed_event_count: u32,
    pub(crate) matched_event_count: u32,
    pub(crate) observed_event_ids_blake3: Vec<String>,
    pub(crate) unknown_event_ids_blake3: Vec<String>,
    pub(crate) missing_action_ids_blake3: Vec<String>,
    pub(crate) overbound_event_ids_blake3: Vec<String>,
    pub(crate) local_only: bool,
    pub(crate) cache_only_completion_count: u32,
    pub(crate) blockers: Vec<String>,
    pub(crate) reconciliation_digest_blake3: String,
}

impl EagerDerivationReconciliation {
    pub(crate) fn is_complete(&self) -> bool {
        self.blockers.is_empty()
            && self.planned_action_count == self.matched_action_count
            && self.observed_event_count == self.matched_event_count
            && self.local_only
            && self.cache_only_completion_count == 0
    }
}

pub(crate) fn plan_eager_derivation_actions(
    input: EagerDerivationActionPlanInput,
) -> Result<EagerDerivationActionPlan, EagerDerivationActionPlanError> {
    validate_text("stage_id", &input.stage_id)?;
    validate_store_dir(&input.store_dir)?;
    validate_digest("sandbox shell", &input.sandbox_shell_digest_blake3)?;
    let entry_count = bounded_count("derivation action", input.entries.len())?;
    if entry_count == 0 {
        return Err(plan_error(
            EagerDerivationActionPlanErrorKind::InvalidInput,
            "eager derivation action plan requires at least one entry",
        ));
    }
    let entries = normalized_entries(input.entries, &input.store_dir)?;
    let outputs = output_producers(&entries, &input.store_dir)?;
    let action_ids = entries.keys().cloned().collect::<BTreeSet<_>>();
    let mut actions = Vec::with_capacity(entries.len());
    for (action_id, entry) in &entries {
        actions.push(action_from_entry(
            action_id,
            entry,
            &action_ids,
            &outputs,
            &input.store_dir,
            &input.sandbox_shell_digest_blake3,
        )?);
    }
    actions.sort_by(|left, right| left.action_id.cmp(&right.action_id));
    let mut plan = EagerDerivationActionPlan {
        schema: EAGER_DERIVATION_ACTION_PLAN_SCHEMA.to_string(),
        stage_id: input.stage_id,
        adapter: "native-derivation-graph-v1".to_string(),
        store_dir: input.store_dir,
        sandbox_shell_digest_blake3: input.sandbox_shell_digest_blake3,
        action_count: entry_count,
        local_only: true,
        cache_only_completion_allowed: false,
        actions,
        blockers: Vec::new(),
        plan_digest_blake3: String::new(),
    };
    plan.plan_digest_blake3 = action_plan_digest(&plan)?;
    validate_eager_derivation_action_plan(&plan)?;
    assert_eq!(plan.actions.len(), entries.len());
    debug_assert_eq!(plan.action_count, entry_count);
    Ok(plan)
}

pub(crate) fn compose_eager_derivation_action_plans(
    stage_id: &str,
    plans: &[EagerDerivationActionPlan],
) -> Result<EagerDerivationActionPlan, EagerDerivationActionPlanError> {
    validate_text("composite stage_id", stage_id)?;
    let first = plans.first().ok_or_else(|| {
        plan_error(
            EagerDerivationActionPlanErrorKind::InvalidInput,
            "composite eager derivation plan requires at least one plan",
        )
    })?;
    validate_eager_derivation_action_plan(first)?;
    let mut actions = BTreeMap::new();
    for plan in plans {
        validate_eager_derivation_action_plan(plan)?;
        if plan.store_dir != first.store_dir || plan.sandbox_shell_digest_blake3 != first.sandbox_shell_digest_blake3 {
            return Err(plan_error(
                EagerDerivationActionPlanErrorKind::InvalidInput,
                "composite eager derivation plans disagree on store or sandbox-shell authority",
            ));
        }
        for action in &plan.actions {
            match actions.entry(action.action_id.clone()) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(action.clone());
                }
                std::collections::btree_map::Entry::Occupied(mut entry) => {
                    let existing = entry.get_mut();
                    let existing_event_count_max = existing.event_count_max;
                    existing.event_count_max = action.event_count_max;
                    if *existing != *action {
                        return Err(plan_error(
                            EagerDerivationActionPlanErrorKind::IncompleteGraph,
                            &format!("composite eager derivation action changed identity: {}", action.action_id),
                        ));
                    }
                    existing.event_count_max =
                        existing_event_count_max.checked_add(action.event_count_max).ok_or_else(|| {
                            plan_error(
                                EagerDerivationActionPlanErrorKind::InvalidInput,
                                "composite derivation action event bound overflows u32",
                            )
                        })?;
                }
            }
        }
    }
    let action_count = bounded_count("composite derivation action", actions.len())?;
    let mut composite = EagerDerivationActionPlan {
        schema: EAGER_DERIVATION_ACTION_PLAN_SCHEMA.to_string(),
        stage_id: stage_id.to_string(),
        adapter: "native-derivation-graph-v1".to_string(),
        store_dir: first.store_dir.clone(),
        sandbox_shell_digest_blake3: first.sandbox_shell_digest_blake3.clone(),
        action_count,
        local_only: true,
        cache_only_completion_allowed: false,
        actions: actions.into_values().collect(),
        blockers: Vec::new(),
        plan_digest_blake3: String::new(),
    };
    composite.plan_digest_blake3 = action_plan_digest(&composite)?;
    validate_eager_derivation_action_plan(&composite)?;
    assert_eq!(composite.action_count, action_count);
    debug_assert!(!composite.actions.is_empty());
    Ok(composite)
}

pub(crate) fn reconcile_eager_derivation_actions(
    plan: &EagerDerivationActionPlan,
    observed_goal_ids_blake3: &[String],
) -> Result<EagerDerivationReconciliation, EagerDerivationActionPlanError> {
    validate_eager_derivation_action_plan(plan)?;
    let observed_event_count = bounded_count("observed derivation event", observed_goal_ids_blake3.len())?;
    let planned_by_goal = plan
        .actions
        .iter()
        .map(|action| (action.observed_goal_key_blake3.clone(), action.action_id_blake3.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut observed_counts = BTreeMap::<String, u32>::new();
    for observed in observed_goal_ids_blake3 {
        validate_digest("observed derivation goal", observed)?;
        let count = observed_counts.entry(observed.clone()).or_insert(0);
        *count = count.checked_add(1).ok_or_else(|| {
            plan_error(
                EagerDerivationActionPlanErrorKind::InvalidInput,
                "observed derivation event count overflows u32",
            )
        })?;
    }
    let unknown_event_ids_blake3 = observed_counts
        .keys()
        .filter(|observed| !planned_by_goal.contains_key(*observed))
        .cloned()
        .collect::<Vec<_>>();
    let action_by_goal = plan
        .actions
        .iter()
        .map(|action| (action.observed_goal_key_blake3.as_str(), action))
        .collect::<BTreeMap<_, _>>();
    let missing_action_ids_blake3 = action_by_goal
        .iter()
        .filter(|(goal, action)| observed_counts.get(**goal).copied().unwrap_or(0) < action.event_count_min)
        .map(|(_goal, action)| action.action_id_blake3.clone())
        .collect::<Vec<_>>();
    let overbound_event_ids_blake3 = action_by_goal
        .iter()
        .filter(|(goal, action)| observed_counts.get(**goal).copied().unwrap_or(0) > action.event_count_max)
        .map(|(goal, _action)| (*goal).to_string())
        .collect::<Vec<_>>();
    let matched_action_count = bounded_count(
        "matched derivation action",
        action_by_goal
            .iter()
            .filter(|(goal, action)| {
                let count = observed_counts.get(**goal).copied().unwrap_or(0);
                count >= action.event_count_min && count <= action.event_count_max
            })
            .count(),
    )?;
    let matched_event_count = observed_counts
        .iter()
        .filter(|(goal, _count)| planned_by_goal.contains_key(*goal))
        .try_fold(0_u32, |total, (_goal, count)| total.checked_add(*count))
        .ok_or_else(|| {
            plan_error(EagerDerivationActionPlanErrorKind::InvalidInput, "matched derivation event count overflows u32")
        })?;
    let mut blockers = Vec::new();
    push_blocker(&mut blockers, !unknown_event_ids_blake3.is_empty(), "unknown-derivation-events");
    push_blocker(&mut blockers, !missing_action_ids_blake3.is_empty(), "missing-derivation-actions");
    push_blocker(&mut blockers, !overbound_event_ids_blake3.is_empty(), "derivation-event-count-bound-violations");
    let observed_event_ids_blake3 = observed_goal_ids_blake3.to_vec();
    let mut reconciliation = EagerDerivationReconciliation {
        schema: EAGER_DERIVATION_RECONCILIATION_SCHEMA.to_string(),
        action_plan_digest_blake3: plan.plan_digest_blake3.clone(),
        planned_action_count: plan.action_count,
        matched_action_count,
        observed_event_count,
        matched_event_count,
        observed_event_ids_blake3,
        unknown_event_ids_blake3,
        missing_action_ids_blake3,
        overbound_event_ids_blake3,
        local_only: true,
        cache_only_completion_count: 0,
        blockers,
        reconciliation_digest_blake3: String::new(),
    };
    reconciliation.reconciliation_digest_blake3 = reconciliation_digest(&reconciliation)?;
    assert!(reconciliation.matched_action_count <= reconciliation.planned_action_count);
    debug_assert_eq!(reconciliation.schema, EAGER_DERIVATION_RECONCILIATION_SCHEMA);
    Ok(reconciliation)
}

pub(crate) fn require_complete_eager_derivation_reconciliation(
    reconciliation: &EagerDerivationReconciliation,
) -> Result<(), EagerDerivationActionPlanError> {
    validate_digest("derivation reconciliation", &reconciliation.reconciliation_digest_blake3)?;
    let expected = reconciliation_digest(reconciliation)?;
    if expected != reconciliation.reconciliation_digest_blake3 || !reconciliation.is_complete() {
        return Err(plan_error(
            EagerDerivationActionPlanErrorKind::IncompleteReconciliation,
            "eager derivation action reconciliation is incomplete or has a digest mismatch",
        ));
    }
    assert_eq!(expected, reconciliation.reconciliation_digest_blake3);
    debug_assert!(reconciliation.blockers.is_empty());
    Ok(())
}

pub(crate) fn validate_eager_derivation_action_plan(
    plan: &EagerDerivationActionPlan,
) -> Result<(), EagerDerivationActionPlanError> {
    if plan.schema != EAGER_DERIVATION_ACTION_PLAN_SCHEMA
        || plan.adapter != "native-derivation-graph-v1"
        || !plan.local_only
        || plan.cache_only_completion_allowed
        || !plan.blockers.is_empty()
    {
        return Err(plan_error(
            EagerDerivationActionPlanErrorKind::InvalidInput,
            "eager derivation action plan identity or policy is invalid",
        ));
    }
    validate_text("stage_id", &plan.stage_id)?;
    validate_store_dir(&plan.store_dir)?;
    validate_digest("sandbox shell", &plan.sandbox_shell_digest_blake3)?;
    validate_digest("plan", &plan.plan_digest_blake3)?;
    let action_count = bounded_count("derivation action", plan.actions.len())?;
    if action_count == 0 || action_count != plan.action_count {
        return Err(plan_error(
            EagerDerivationActionPlanErrorKind::InvalidInput,
            "eager derivation action count does not match its action list",
        ));
    }
    validate_actions(&plan.actions, &plan.store_dir)?;
    let expected_digest = action_plan_digest(plan)?;
    if expected_digest != plan.plan_digest_blake3 {
        return Err(plan_error(
            EagerDerivationActionPlanErrorKind::InvalidInput,
            "eager derivation action plan digest does not match canonical content",
        ));
    }
    assert_eq!(action_count, plan.action_count);
    debug_assert_eq!(expected_digest, plan.plan_digest_blake3);
    Ok(())
}

fn normalized_entries(
    entries: Vec<EagerDerivationEntry>,
    store_dir: &str,
) -> Result<BTreeMap<String, EagerDerivationEntry>, EagerDerivationActionPlanError> {
    let mut normalized = BTreeMap::new();
    for entry in entries {
        let action_id = entry.drv_path.to_absolute_path_with_prefix(store_dir);
        validate_text("derivation action id", &action_id)?;
        if !entry.dynamic_plan_outputs.is_empty() {
            return Err(plan_error(
                EagerDerivationActionPlanErrorKind::UnsupportedDynamicAction,
                &format!("derivation action {action_id} declares runtime dynamic outputs"),
            ));
        }
        if normalized.insert(action_id.clone(), entry).is_some() {
            return Err(plan_error(
                EagerDerivationActionPlanErrorKind::IncompleteGraph,
                &format!("derivation action graph duplicates {action_id}"),
            ));
        }
    }
    assert!(!normalized.is_empty());
    debug_assert!(normalized.len() <= ACTION_COUNT_MAX as usize);
    Ok(normalized)
}

fn output_producers(
    entries: &BTreeMap<String, EagerDerivationEntry>,
    store_dir: &str,
) -> Result<BTreeMap<String, (String, String)>, EagerDerivationActionPlanError> {
    let mut outputs = BTreeMap::new();
    for (action_id, entry) in entries {
        for (output_name, output) in &entry.derivation.outputs {
            let Some(path) = output.path.as_ref() else {
                continue;
            };
            let identity = path.to_absolute_path_with_prefix(store_dir);
            if outputs.insert(identity.clone(), (action_id.clone(), output_name.clone())).is_some() {
                return Err(plan_error(
                    EagerDerivationActionPlanErrorKind::IncompleteGraph,
                    &format!("derivation output identity is produced twice: {identity}"),
                ));
            }
        }
    }
    debug_assert!(outputs.len() <= entries.len().saturating_mul(OUTPUTS_PER_ACTION_DEBUG_MAX));
    Ok(outputs)
}

fn action_from_entry(
    action_id: &str,
    entry: &EagerDerivationEntry,
    action_ids: &BTreeSet<String>,
    output_producers: &BTreeMap<String, (String, String)>,
    store_dir: &str,
    sandbox_shell_digest: &str,
) -> Result<DerivationAction, EagerDerivationActionPlanError> {
    let derivation = &entry.derivation;
    let producer_action_ids = producer_actions(action_id, derivation, action_ids, store_dir)?;
    let input_source_ids = derivation
        .input_sources
        .iter()
        .map(|source| source.to_absolute_path_with_prefix(store_dir))
        .collect::<Vec<_>>();
    let outputs = action_outputs(action_id, derivation, entry.content_addressed, store_dir)?;
    let executable = executable_authority(&derivation.builder, output_producers, sandbox_shell_digest)?;
    let input_envelope_digest_blake3 = input_envelope_digest(derivation)?;
    let action = DerivationAction {
        action_id: action_id.to_string(),
        action_id_blake3: blake3::hash(action_id.as_bytes()).to_hex().to_string(),
        observed_goal_key_blake3: blake3::hash(entry.drv_path.to_absolute_path().as_bytes()).to_hex().to_string(),
        producer_action_ids,
        input_source_ids,
        outputs,
        executable,
        system: derivation.system.clone(),
        input_envelope_digest_blake3,
        local_only: true,
        event_count_min: 1,
        event_count_max: 1,
    };
    assert_eq!(action.action_id_blake3.len(), BLAKE3_HEX_LENGTH);
    assert_eq!(action.observed_goal_key_blake3.len(), BLAKE3_HEX_LENGTH);
    debug_assert!(!action.outputs.is_empty());
    Ok(action)
}

fn producer_actions(
    action_id: &str,
    derivation: &Derivation,
    action_ids: &BTreeSet<String>,
    store_dir: &str,
) -> Result<Vec<String>, EagerDerivationActionPlanError> {
    let mut producers = Vec::with_capacity(derivation.input_derivations.len());
    for producer in derivation.input_derivations.keys() {
        let producer_id = producer.to_absolute_path_with_prefix(store_dir);
        if producer_id == action_id || !action_ids.contains(&producer_id) {
            return Err(plan_error(
                EagerDerivationActionPlanErrorKind::IncompleteGraph,
                &format!("derivation action {action_id} has missing or cyclic producer {producer_id}"),
            ));
        }
        producers.push(producer_id);
    }
    producers.sort();
    producers.dedup();
    debug_assert!(producers.windows(2).all(|pair| pair[0] < pair[1]));
    Ok(producers)
}

fn action_outputs(
    action_id: &str,
    derivation: &Derivation,
    content_addressed: bool,
    store_dir: &str,
) -> Result<Vec<DerivationActionOutput>, EagerDerivationActionPlanError> {
    let mut outputs = Vec::with_capacity(derivation.outputs.len());
    for (name, output) in &derivation.outputs {
        validate_text("derivation output name", name)?;
        let identity = output.path.as_ref().map_or_else(
            || format!("content-addressed:{action_id}:{name}"),
            |path| path.to_absolute_path_with_prefix(store_dir),
        );
        outputs.push(DerivationActionOutput {
            name: name.clone(),
            identity,
            content_addressed,
        });
    }
    outputs.sort_by(|left, right| left.name.cmp(&right.name));
    if outputs.is_empty() {
        return Err(plan_error(
            EagerDerivationActionPlanErrorKind::IncompleteGraph,
            &format!("derivation action {action_id} has no outputs"),
        ));
    }
    Ok(outputs)
}

fn executable_authority(
    builder: &str,
    output_producers: &BTreeMap<String, (String, String)>,
    sandbox_shell_digest: &str,
) -> Result<DerivationExecutableAuthority, EagerDerivationActionPlanError> {
    if builder == SANDBOX_SHELL_PATH {
        return Ok(DerivationExecutableAuthority::FixedSandboxShell {
            logical_path: SANDBOX_SHELL_PATH.to_string(),
            digest_blake3: sandbox_shell_digest.to_string(),
        });
    }
    if let Some(name) = builder.strip_prefix(BUILTIN_PREFIX) {
        validate_text("builtin builder name", name)?;
        return Ok(DerivationExecutableAuthority::Builtin {
            identity: builder.to_string(),
        });
    }
    let produced_matches = output_producers
        .iter()
        .filter(|(output_identity, _producer)| {
            builder == output_identity.as_str()
                || builder.strip_prefix(output_identity.as_str()).is_some_and(|suffix| suffix.starts_with('/'))
        })
        .collect::<Vec<_>>();
    if produced_matches.len() == 1 {
        let (_output_identity, (producer_action_id, output_name)) = produced_matches[0];
        return Ok(DerivationExecutableAuthority::Produced {
            producer_action_id: producer_action_id.clone(),
            output_name: output_name.clone(),
            output_identity: builder.to_string(),
        });
    }
    if produced_matches.len() > 1 {
        return Err(plan_error(
            EagerDerivationActionPlanErrorKind::UnboundExecutable,
            &format!("derivation builder matches multiple produced authorities: {builder}"),
        ));
    }
    Err(plan_error(
        EagerDerivationActionPlanErrorKind::UnboundExecutable,
        &format!("derivation builder has no fixed or produced authority: {builder}"),
    ))
}

fn input_envelope_digest(derivation: &Derivation) -> Result<String, EagerDerivationActionPlanError> {
    #[derive(Serialize)]
    struct Envelope<'a> {
        arguments: &'a [String],
        environment: &'a BTreeMap<String, bstr::BString>,
        system: &'a str,
    }
    let bytes = serde_json::to_vec(&Envelope {
        arguments: &derivation.arguments,
        environment: &derivation.environment,
        system: &derivation.system,
    })
    .map_err(|error| {
        plan_error(
            EagerDerivationActionPlanErrorKind::Serialization,
            &format!("serializing derivation input envelope: {error}"),
        )
    })?;
    let mut hasher = blake3::Hasher::new_derive_key(ACTION_INPUT_DIGEST_CONTEXT);
    hasher.update(&bytes);
    let digest = hasher.finalize().to_hex().to_string();
    assert_eq!(digest.len(), BLAKE3_HEX_LENGTH);
    debug_assert!(!bytes.is_empty());
    Ok(digest)
}

fn validate_actions(actions: &[DerivationAction], store_dir: &str) -> Result<(), EagerDerivationActionPlanError> {
    let action_ids = actions.iter().map(|action| action.action_id.as_str()).collect::<BTreeSet<_>>();
    let observed_goal_ids =
        actions.iter().map(|action| action.observed_goal_key_blake3.as_str()).collect::<BTreeSet<_>>();
    if action_ids.len() != actions.len() || observed_goal_ids.len() != actions.len() {
        return Err(plan_error(
            EagerDerivationActionPlanErrorKind::IncompleteGraph,
            "eager derivation action plan contains duplicate action IDs",
        ));
    }
    for action in actions {
        validate_action(action, &action_ids, store_dir)?;
    }
    assert_eq!(action_ids.len(), actions.len());
    assert_eq!(observed_goal_ids.len(), actions.len());
    debug_assert!(!action_ids.is_empty());
    Ok(())
}

fn validate_action(
    action: &DerivationAction,
    action_ids: &BTreeSet<&str>,
    store_dir: &str,
) -> Result<(), EagerDerivationActionPlanError> {
    validate_text("derivation action id", &action.action_id)?;
    validate_digest("derivation action", &action.action_id_blake3)?;
    validate_digest("observed goal key", &action.observed_goal_key_blake3)?;
    validate_digest("derivation input envelope", &action.input_envelope_digest_blake3)?;
    let store_path: StorePath<String> =
        StorePath::from_absolute_path_with_prefix(action.action_id.as_bytes(), store_dir).map_err(|_| {
            plan_error(
                EagerDerivationActionPlanErrorKind::InvalidInput,
                &format!("derivation action has an invalid store identity: {}", action.action_id),
            )
        })?;
    let expected_observed_goal = blake3::hash(store_path.to_absolute_path().as_bytes()).to_hex().to_string();
    if action.observed_goal_key_blake3 != expected_observed_goal
        || !action.action_id.starts_with(store_dir)
        || !action.local_only
        || action.event_count_min != 1
        || action.event_count_max < action.event_count_min
        || action.event_count_max > ACTION_COUNT_MAX
        || action.outputs.is_empty()
    {
        return Err(plan_error(
            EagerDerivationActionPlanErrorKind::InvalidInput,
            &format!("derivation action {} has invalid local execution bounds", action.action_id),
        ));
    }
    if action.producer_action_ids.iter().any(|producer| !action_ids.contains(producer.as_str())) {
        return Err(plan_error(
            EagerDerivationActionPlanErrorKind::IncompleteGraph,
            &format!("derivation action {} references an unknown producer", action.action_id),
        ));
    }
    if let DerivationExecutableAuthority::Produced { producer_action_id, .. } = &action.executable
        && !action_ids.contains(producer_action_id.as_str())
    {
        return Err(plan_error(
            EagerDerivationActionPlanErrorKind::UnboundExecutable,
            &format!("derivation action {} has an unknown executable producer", action.action_id),
        ));
    }
    assert!(action.event_count_min <= action.event_count_max);
    debug_assert_eq!(action.action_id_blake3, blake3::hash(action.action_id.as_bytes()).to_hex().to_string());
    debug_assert_eq!(action.observed_goal_key_blake3, expected_observed_goal);
    Ok(())
}

fn reconciliation_digest(
    reconciliation: &EagerDerivationReconciliation,
) -> Result<String, EagerDerivationActionPlanError> {
    let mut canonical = reconciliation.clone();
    canonical.reconciliation_digest_blake3.clear();
    let bytes = serde_json::to_vec(&canonical).map_err(|error| {
        plan_error(
            EagerDerivationActionPlanErrorKind::Serialization,
            &format!("serializing eager derivation reconciliation: {error}"),
        )
    })?;
    let mut hasher = blake3::Hasher::new_derive_key(RECONCILIATION_DIGEST_CONTEXT);
    hasher.update(&bytes);
    let digest = hasher.finalize().to_hex().to_string();
    assert_eq!(digest.len(), BLAKE3_HEX_LENGTH);
    debug_assert!(!bytes.is_empty());
    Ok(digest)
}

fn action_plan_digest(plan: &EagerDerivationActionPlan) -> Result<String, EagerDerivationActionPlanError> {
    let mut canonical = plan.clone();
    canonical.plan_digest_blake3.clear();
    let bytes = serde_json::to_vec(&canonical).map_err(|error| {
        plan_error(
            EagerDerivationActionPlanErrorKind::Serialization,
            &format!("serializing eager derivation action plan: {error}"),
        )
    })?;
    let mut hasher = blake3::Hasher::new_derive_key(PLAN_DIGEST_CONTEXT);
    hasher.update(&bytes);
    let digest = hasher.finalize().to_hex().to_string();
    assert_eq!(digest.len(), BLAKE3_HEX_LENGTH);
    debug_assert!(!bytes.is_empty());
    Ok(digest)
}

fn push_blocker(blockers: &mut Vec<String>, blocked: bool, blocker: &str) {
    if blocked {
        blockers.push(blocker.to_string());
    }
    assert!(!blocker.is_empty());
    debug_assert!(!blocked || blockers.last().is_some_and(|last| last == blocker));
}

fn validate_store_dir(store_dir: &str) -> Result<(), EagerDerivationActionPlanError> {
    validate_text("store_dir", store_dir)?;
    if !store_dir.starts_with('/') || store_dir.ends_with('/') {
        return Err(plan_error(
            EagerDerivationActionPlanErrorKind::InvalidInput,
            "store_dir must be an absolute normalized path without a trailing slash",
        ));
    }
    Ok(())
}

fn validate_text(label: &str, value: &str) -> Result<(), EagerDerivationActionPlanError> {
    let valid = !value.is_empty()
        && value.len() <= TEXT_BYTES_MAX
        && !value.chars().any(char::is_control)
        && !value.split('/').any(|component| component == "..");
    if !valid {
        return Err(plan_error(
            EagerDerivationActionPlanErrorKind::InvalidInput,
            &format!("{label} is empty, oversized, unsafe, or contains control characters"),
        ));
    }
    Ok(())
}

fn validate_digest(label: &str, digest: &str) -> Result<(), EagerDerivationActionPlanError> {
    let valid = digest.len() == BLAKE3_HEX_LENGTH
        && digest.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if !valid {
        return Err(plan_error(
            EagerDerivationActionPlanErrorKind::InvalidInput,
            &format!("{label} is not a lowercase BLAKE3 digest"),
        ));
    }
    Ok(())
}

fn bounded_count(label: &str, count: usize) -> Result<u32, EagerDerivationActionPlanError> {
    let count = u32::try_from(count).map_err(|_| {
        plan_error(EagerDerivationActionPlanErrorKind::InvalidInput, &format!("{label} count exceeds u32"))
    })?;
    if count > ACTION_COUNT_MAX {
        return Err(plan_error(
            EagerDerivationActionPlanErrorKind::InvalidInput,
            &format!("{label} count {count} exceeds {ACTION_COUNT_MAX}"),
        ));
    }
    Ok(count)
}

fn plan_error(kind: EagerDerivationActionPlanErrorKind, message: &str) -> EagerDerivationActionPlanError {
    EagerDerivationActionPlanError {
        kind,
        message: message.to_string(),
    }
}

#[cfg(test)]
pub(crate) use tests::two_action_plan as eager_action_test_plan;

#[cfg(test)]
mod tests {
    use nix_compat::derivation::Output;

    use super::*;

    const STORE_DIR: &str = "/mantle/store";
    const SANDBOX_DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const UNKNOWN_GOAL_DIGEST: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const SOURCE_NAME: &str = "source";
    const BUILD_NAME: &str = "build";
    const DYNAMIC_OUTPUT_NAME: &str = "dynamic";
    const TEST_ACTION_COUNT: u32 = 2;
    const EXPECTED_RECONCILIATION_BLOCKER_COUNT: usize = 3;
    const STORE_PATH_DIGEST_BYTES: usize = 20;

    #[test]
    fn eager_plan_enumerates_transitive_producers_and_fixed_builders() {
        let plan = two_action_plan();

        assert_eq!(plan.action_count, TEST_ACTION_COUNT);
        assert_eq!(u32::try_from(plan.actions.len()).unwrap(), TEST_ACTION_COUNT);
        assert!(matches!(plan.actions[0].executable, DerivationExecutableAuthority::FixedSandboxShell { .. }));
        assert!(matches!(plan.actions[1].executable, DerivationExecutableAuthority::Builtin { .. }));
        assert_eq!(plan.actions[0].producer_action_ids.len(), 1);
        assert!(plan.blockers.is_empty());
    }

    #[test]
    fn eager_plan_composition_deduplicates_exact_actions_and_rejects_drift() {
        let first = two_action_plan();
        let mut changed = first.clone();
        changed.actions[0].input_envelope_digest_blake3 = UNKNOWN_GOAL_DIGEST.to_string();
        changed.plan_digest_blake3 = action_plan_digest(&changed).unwrap();

        let composite =
            compose_eager_derivation_action_plans("full-source-native-provider", &[first.clone(), first]).unwrap();
        let error = compose_eager_derivation_action_plans("full-source-native-provider", &[composite.clone(), changed])
            .unwrap_err();

        assert_eq!(composite.action_count, TEST_ACTION_COUNT);
        assert_eq!(error.kind, EagerDerivationActionPlanErrorKind::IncompleteGraph);
        assert!(error.message.contains("changed identity"));
    }

    #[test]
    fn eager_reconciliation_accepts_exact_events() {
        let plan = two_action_plan();
        let observed = plan.actions.iter().map(|action| action.observed_goal_key_blake3.clone()).collect::<Vec<_>>();

        let reconciliation = reconcile_eager_derivation_actions(&plan, &observed).unwrap();
        require_complete_eager_derivation_reconciliation(&reconciliation).unwrap();

        assert!(reconciliation.is_complete());
        assert_eq!(reconciliation.matched_action_count, TEST_ACTION_COUNT);
        assert_eq!(reconciliation.observed_event_count, TEST_ACTION_COUNT);
    }

    #[test]
    fn eager_reconciliation_rejects_unknown_missing_and_duplicate_events() {
        let plan = two_action_plan();
        let repeated = plan.actions[0].observed_goal_key_blake3.clone();
        let observed = vec![repeated.clone(), repeated, UNKNOWN_GOAL_DIGEST.to_string()];

        let reconciliation = reconcile_eager_derivation_actions(&plan, &observed).unwrap();
        let error = require_complete_eager_derivation_reconciliation(&reconciliation).unwrap_err();

        assert!(!reconciliation.is_complete());
        assert_eq!(reconciliation.blockers.len(), EXPECTED_RECONCILIATION_BLOCKER_COUNT);
        assert_eq!(error.kind, EagerDerivationActionPlanErrorKind::IncompleteReconciliation);
    }

    #[test]
    fn eager_plan_rejects_runtime_dynamic_derivations() {
        let dynamic = entry(SOURCE_NAME, builtin_derivation(SOURCE_NAME), false, vec![DYNAMIC_OUTPUT_NAME.to_string()]);

        let error = plan_eager_derivation_actions(EagerDerivationActionPlanInput {
            stage_id: "native-provider".to_string(),
            store_dir: STORE_DIR.to_string(),
            sandbox_shell_digest_blake3: SANDBOX_DIGEST.to_string(),
            entries: vec![dynamic],
        })
        .unwrap_err();

        assert_eq!(error.kind, EagerDerivationActionPlanErrorKind::UnsupportedDynamicAction);
        assert!(error.message.contains("runtime dynamic outputs"));
    }

    #[test]
    fn eager_plan_rejects_missing_producer_and_unbound_builder() {
        let missing = fake_store_path("missing");
        let with_missing = entry(BUILD_NAME, shell_derivation(BUILD_NAME, Some(missing)), false, Vec::new());
        let missing_error = plan_eager_derivation_actions(EagerDerivationActionPlanInput {
            stage_id: "native-provider".to_string(),
            store_dir: STORE_DIR.to_string(),
            sandbox_shell_digest_blake3: SANDBOX_DIGEST.to_string(),
            entries: vec![with_missing],
        })
        .unwrap_err();
        let unbound = entry(SOURCE_NAME, unbound_derivation(SOURCE_NAME), false, Vec::new());
        let unbound_error = plan_eager_derivation_actions(EagerDerivationActionPlanInput {
            stage_id: "native-provider".to_string(),
            store_dir: STORE_DIR.to_string(),
            sandbox_shell_digest_blake3: SANDBOX_DIGEST.to_string(),
            entries: vec![unbound],
        })
        .unwrap_err();

        assert_eq!(missing_error.kind, EagerDerivationActionPlanErrorKind::IncompleteGraph);
        assert_eq!(unbound_error.kind, EagerDerivationActionPlanErrorKind::UnboundExecutable);
    }

    pub(crate) fn two_action_plan() -> EagerDerivationActionPlan {
        let source = entry(SOURCE_NAME, builtin_derivation(SOURCE_NAME), false, Vec::new());
        let source_drv = source.drv_path.clone();
        let build = entry(BUILD_NAME, shell_derivation(BUILD_NAME, Some(source_drv)), false, Vec::new());
        plan_eager_derivation_actions(EagerDerivationActionPlanInput {
            stage_id: "native-provider".to_string(),
            store_dir: STORE_DIR.to_string(),
            sandbox_shell_digest_blake3: SANDBOX_DIGEST.to_string(),
            entries: vec![build, source],
        })
        .unwrap()
    }

    fn entry(
        name: &str,
        derivation: Derivation,
        content_addressed: bool,
        dynamic_plan_outputs: Vec<String>,
    ) -> EagerDerivationEntry {
        EagerDerivationEntry {
            drv_path: fake_store_path(&format!("{name}.drv")),
            derivation,
            content_addressed,
            dynamic_plan_outputs,
        }
    }

    fn builtin_derivation(name: &str) -> Derivation {
        derivation(name, "builtin:fetchurl", None)
    }

    fn shell_derivation(name: &str, producer: Option<StorePath<String>>) -> Derivation {
        derivation(name, SANDBOX_SHELL_PATH, producer)
    }

    fn unbound_derivation(name: &str) -> Derivation {
        derivation(name, "/unbound/tool", None)
    }

    fn derivation(name: &str, builder: &str, producer: Option<StorePath<String>>) -> Derivation {
        let mut outputs = BTreeMap::new();
        outputs.insert("out".to_string(), Output {
            path: Some(fake_store_path(name)),
            ca_hash: None,
        });
        let mut input_derivations = BTreeMap::new();
        if let Some(producer) = producer {
            input_derivations.insert(producer, BTreeSet::from(["out".to_string()]));
        }
        Derivation {
            arguments: vec!["build".to_string()],
            builder: builder.to_string(),
            environment: BTreeMap::from([("name".to_string(), bstr::BString::from(name))]),
            input_derivations,
            input_sources: BTreeSet::new(),
            outputs,
            system: "x86_64-linux".to_string(),
        }
    }

    fn fake_store_path(name: &str) -> StorePath<String> {
        StorePath::from_name_and_digest_fixed(name, [2_u8; STORE_PATH_DIGEST_BYTES]).unwrap()
    }
}
