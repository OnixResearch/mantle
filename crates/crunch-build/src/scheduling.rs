//! Pure deterministic priority kernel for Mantle's lazy build goals.
//!
//! This module owns only immutable scheduling facts, validation, bounded
//! known-graph pressure propagation, eligibility normalization, age classes,
//! total ordering, and redacted decision evidence. `worker.rs` remains the
//! imperative shell that mutates the goal registry, advances epochs, dispatches
//! builds, and records returned evidence.
//!
//! r[impl build_scheduling.deterministic_priority_kernel]
//! r[impl build_scheduling.lazy_known_critical_path]
//! r[impl build_scheduling.resource_locality_preference]
//! r[impl build_scheduling.deterministic_starvation_bound]
//! r[impl build_scheduling.priority_decision_evidence]

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;

use crate::goal::MAX_GOALS;

pub const SCHEDULING_POLICY_SCHEMA: &str = "mantle-scheduling-policy-v1";
pub const SCHEDULING_HISTORY_SCHEMA: &str = "mantle-scheduling-history-v1";
pub const PRIORITY_DECISION_SCHEMA: &str = "mantle-priority-decision-v1";
pub const DEFAULT_SCHEDULING_POLICY_ID: &str = "mantle-lazy-priority-v1";
pub const KNOWN_GRAPH_BASIS: &str = "known-graph";
pub const SCHEDULING_CLAIM_SCOPE: &str = "configured-known-fact-ordering";
pub const MAX_SCHEDULING_GOALS: u32 = MAX_GOALS;
pub const MAX_READY_GOALS: u32 = MAX_GOALS;
pub const MAX_HISTORY_ENTRIES: u32 = MAX_GOALS;
pub const MAX_HISTORY_KEY_BYTES_TOTAL: u32 = 4_194_304;
pub const MAX_HISTORY_SNAPSHOT_BYTES: u32 = 8_388_608;
pub const MAX_PRIORITY_DECISIONS: u32 = MAX_GOALS;
pub const MAX_EDGES_PER_GOAL_BUDGET: u32 = 64;
pub const MAX_KNOWN_GRAPH_EDGES: u32 = MAX_SCHEDULING_GOALS.saturating_mul(MAX_EDGES_PER_GOAL_BUDGET);
pub const MAX_BLOCKED_ROOT_PRESSURE: u32 = 256;
pub const MAX_SCHEDULING_EVENTS_PER_GOAL: u32 = 8;
pub const MAX_SCHEDULING_EPOCH: u32 = MAX_SCHEDULING_GOALS.saturating_mul(MAX_SCHEDULING_EVENTS_PER_GOAL);
pub const MAX_POLICY_ID_BYTES: u32 = 128;
pub const MAX_GOAL_KEY_BYTES: u32 = 4096;
pub const DEFAULT_AGED_AFTER_EPOCHS: u32 = 4;
pub const DEFAULT_PROTECTED_AFTER_EPOCHS: u32 = 8;
const PREFERENCE_FIELD_COUNT: u32 = 3;
const STRUCTURAL_DURATION_UNITS: u32 = 1;
const SHORT_DURATION_UNITS: u32 = 2;
const MEDIUM_DURATION_UNITS: u32 = 4;
const LONG_DURATION_UNITS: u32 = 8;
const MAX_KNOWN_CRITICAL_PATH_WORK_UNITS: u32 = MAX_SCHEDULING_GOALS.saturating_mul(LONG_DURATION_UNITS);
const PRIORITY_NON_CLAIM_COUNT: usize = 5;
const HEX_LOWER_CHARS_PER_BYTE: usize = 2;
const BLAKE3_HEX_LENGTH: usize = blake3::OUT_LEN.saturating_mul(HEX_LOWER_CHARS_PER_BYTE);
const ADJACENT_PAIR_WINDOW: usize = 2;

pub const PRIORITY_NON_CLAIMS: [&str; PRIORITY_NON_CLAIM_COUNT] = [
    "global-optimality",
    "future-graph-knowledge",
    "execution-success",
    "output-trust",
    "release-reproducibility",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PreferenceField {
    KnownGraph,
    ResourceFit,
    LocalityTransfer,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchedulingPolicy {
    pub schema: String,
    pub policy_id: String,
    pub preference_order: Vec<PreferenceField>,
    pub aged_after_epochs: u32,
    pub protected_after_epochs: u32,
}

impl Default for SchedulingPolicy {
    fn default() -> Self {
        Self {
            schema: SCHEDULING_POLICY_SCHEMA.to_string(),
            policy_id: DEFAULT_SCHEDULING_POLICY_ID.to_string(),
            preference_order: vec![
                PreferenceField::KnownGraph,
                PreferenceField::ResourceFit,
                PreferenceField::LocalityTransfer,
            ],
            aged_after_epochs: DEFAULT_AGED_AFTER_EPOCHS,
            protected_after_epochs: DEFAULT_PROTECTED_AFTER_EPOCHS,
        }
    }
}

impl SchedulingPolicy {
    pub fn validate(&self) -> Result<(), SchedulingError> {
        validate_policy_identity(self)?;
        validate_preference_order(&self.preference_order)?;
        if self.aged_after_epochs == 0 {
            return Err(SchedulingError::InvalidPolicy {
                reason: "aged_after_epochs must be greater than zero".to_string(),
            });
        }
        if self.protected_after_epochs <= self.aged_after_epochs {
            return Err(SchedulingError::InvalidPolicy {
                reason: "protected_after_epochs must be greater than aged_after_epochs".to_string(),
            });
        }
        if self.protected_after_epochs > MAX_SCHEDULING_EPOCH {
            return Err(SchedulingError::InvalidPolicy {
                reason: format!("protected_after_epochs exceeds scheduling epoch limit {MAX_SCHEDULING_EPOCH}"),
            });
        }
        debug_assert!(self.aged_after_epochs < self.protected_after_epochs);
        debug_assert!(self.protected_after_epochs <= MAX_SCHEDULING_EPOCH);
        Ok(())
    }

    pub fn digest_blake3(&self) -> Result<String, SchedulingError> {
        self.validate()?;
        let canonical = serde_json::to_vec(self).map_err(|source| SchedulingError::SerializePolicy {
            reason: source.to_string(),
        })?;
        let digest = blake3::hash(&canonical);
        debug_assert!(!canonical.is_empty());
        debug_assert_eq!(digest.as_bytes().len(), blake3::OUT_LEN);
        Ok(digest.to_hex().to_string())
    }
}

fn validate_policy_identity(policy: &SchedulingPolicy) -> Result<(), SchedulingError> {
    if policy.schema != SCHEDULING_POLICY_SCHEMA {
        return Err(SchedulingError::InvalidPolicy {
            reason: "unsupported policy schema".to_string(),
        });
    }
    validate_bounded_identifier(BoundedIdentifier {
        field: "policy_id",
        value: &policy.policy_id,
        max_bytes: MAX_POLICY_ID_BYTES,
    })?;
    debug_assert_eq!(policy.schema, SCHEDULING_POLICY_SCHEMA);
    debug_assert!(!policy.policy_id.is_empty());
    Ok(())
}

fn validate_preference_order(order: &[PreferenceField]) -> Result<(), SchedulingError> {
    let actual_count = len_as_u32("preference_order", order.len())?;
    if actual_count != PREFERENCE_FIELD_COUNT {
        return Err(SchedulingError::InvalidPolicy {
            reason: format!(
                "preference_order must contain exactly {PREFERENCE_FIELD_COUNT} fields; got {actual_count}"
            ),
        });
    }
    let unique: BTreeSet<PreferenceField> = order.iter().copied().collect();
    let unique_count = len_as_u32("preference_order unique fields", unique.len())?;
    if unique_count != PREFERENCE_FIELD_COUNT {
        return Err(SchedulingError::InvalidPolicy {
            reason: "preference_order must contain each field exactly once".to_string(),
        });
    }
    debug_assert_eq!(actual_count, PREFERENCE_FIELD_COUNT);
    debug_assert_eq!(unique_count, PREFERENCE_FIELD_COUNT);
    Ok(())
}

struct BoundedIdentifier<'a> {
    field: &'static str,
    value: &'a str,
    max_bytes: u32,
}

fn validate_bounded_identifier(input: BoundedIdentifier<'_>) -> Result<(), SchedulingError> {
    let BoundedIdentifier {
        field,
        value,
        max_bytes,
    } = input;
    if value.is_empty() {
        return Err(SchedulingError::InvalidIdentifier {
            field,
            reason: "must not be empty".to_string(),
        });
    }
    let actual_bytes = len_as_u32(field, value.len())?;
    if actual_bytes > max_bytes {
        return Err(SchedulingError::InvalidIdentifier {
            field,
            reason: format!("exceeds {max_bytes} bytes"),
        });
    }
    if !value.bytes().all(is_safe_identifier_byte) {
        return Err(SchedulingError::InvalidIdentifier {
            field,
            reason: "contains characters outside [A-Za-z0-9._-]".to_string(),
        });
    }
    debug_assert!(actual_bytes > 0);
    debug_assert!(actual_bytes <= max_bytes);
    Ok(())
}

fn is_safe_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-')
}

fn goal_identity_blake3(value: &str) -> String {
    let digest = blake3::hash(value.as_bytes()).to_hex().to_string();
    debug_assert_eq!(digest.len(), BLAKE3_HEX_LENGTH);
    debug_assert!(!digest.chars().any(char::is_control));
    digest
}

struct GoalKeyInput<'a> {
    field: &'static str,
    value: &'a str,
}

fn validate_goal_key(input: GoalKeyInput<'_>) -> Result<(), SchedulingError> {
    let GoalKeyInput { field, value } = input;
    if value.is_empty() {
        return Err(SchedulingError::InvalidIdentifier {
            field,
            reason: "must not be empty".to_string(),
        });
    }
    let actual_bytes = len_as_u32(field, value.len())?;
    if actual_bytes > MAX_GOAL_KEY_BYTES {
        return Err(SchedulingError::InvalidIdentifier {
            field,
            reason: format!("exceeds {MAX_GOAL_KEY_BYTES} bytes"),
        });
    }
    if value.chars().any(char::is_control) {
        return Err(SchedulingError::InvalidIdentifier {
            field,
            reason: "contains a control character".to_string(),
        });
    }
    debug_assert!(actual_bytes > 0);
    debug_assert!(actual_bytes <= MAX_GOAL_KEY_BYTES);
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnownGoalFacts {
    pub goal_key: String,
    pub waitees: Vec<String>,
    pub waiters: Vec<String>,
    pub requested_root: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KnownGraphFacts {
    pub goals: Vec<KnownGoalFacts>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ValidatedGoalFacts {
    waitees: BTreeSet<String>,
    waiters: BTreeSet<String>,
    requested_root: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ValidatedKnownGraph {
    goals: BTreeMap<String, ValidatedGoalFacts>,
    edge_count: u32,
}

fn validate_known_graph(graph: &KnownGraphFacts) -> Result<ValidatedKnownGraph, SchedulingError> {
    let goal_count = len_as_u32("known graph goals", graph.goals.len())?;
    if goal_count > MAX_SCHEDULING_GOALS {
        return Err(SchedulingError::TooManyGoals {
            actual: goal_count,
            max: MAX_SCHEDULING_GOALS,
        });
    }
    let mut goals = BTreeMap::new();
    let mut edge_count = 0_u32;
    let mut reciprocal_edge_count = 0_u32;
    for goal in &graph.goals {
        validate_goal_key(GoalKeyInput {
            field: "goal_key",
            value: &goal.goal_key,
        })?;
        let waitees = unique_edges(EdgeInput {
            goal_key: &goal.goal_key,
            edge_kind: "waitees",
            edges: &goal.waitees,
        })?;
        let waiters = unique_edges(EdgeInput {
            goal_key: &goal.goal_key,
            edge_kind: "waiters",
            edges: &goal.waiters,
        })?;
        edge_count = bounded_edge_total(edge_count, waitees.len(), "known graph waitee edge count")?;
        reciprocal_edge_count =
            bounded_edge_total(reciprocal_edge_count, waiters.len(), "known graph waiter edge count")?;
        if goals
            .insert(goal.goal_key.clone(), ValidatedGoalFacts {
                waitees,
                waiters,
                requested_root: goal.requested_root,
            })
            .is_some()
        {
            return Err(SchedulingError::DuplicateGoal {
                goal_key: goal_identity_blake3(&goal.goal_key),
            });
        }
    }
    validate_reciprocal_edges(&goals)?;
    debug_assert_eq!(goals.len(), graph.goals.len());
    debug_assert_eq!(edge_count, reciprocal_edge_count);
    debug_assert!(edge_count <= MAX_KNOWN_GRAPH_EDGES);
    Ok(ValidatedKnownGraph { goals, edge_count })
}

fn bounded_edge_total(current: u32, additional: usize, field: &'static str) -> Result<u32, SchedulingError> {
    let next = current
        .checked_add(len_as_u32(field, additional)?)
        .ok_or(SchedulingError::ArithmeticOverflow { field })?;
    if next > MAX_KNOWN_GRAPH_EDGES {
        return Err(SchedulingError::TooManyEdges {
            actual: next,
            max: MAX_KNOWN_GRAPH_EDGES,
        });
    }
    debug_assert!(next >= current);
    debug_assert!(next <= MAX_KNOWN_GRAPH_EDGES);
    Ok(next)
}

struct EdgeInput<'a> {
    goal_key: &'a str,
    edge_kind: &'static str,
    edges: &'a [String],
}

fn unique_edges(input: EdgeInput<'_>) -> Result<BTreeSet<String>, SchedulingError> {
    let EdgeInput {
        goal_key,
        edge_kind,
        edges,
    } = input;
    let mut unique = BTreeSet::new();
    for edge in edges {
        validate_goal_key(GoalKeyInput {
            field: "edge goal key",
            value: edge,
        })?;
        if !unique.insert(edge.clone()) {
            return Err(SchedulingError::DuplicateEdge {
                goal_key: goal_identity_blake3(goal_key),
                edge_kind,
                referenced_goal: goal_identity_blake3(edge),
            });
        }
    }
    debug_assert_eq!(unique.len(), edges.len());
    debug_assert!(unique.iter().all(|edge| !edge.is_empty()));
    Ok(unique)
}

fn validate_reciprocal_edges(goals: &BTreeMap<String, ValidatedGoalFacts>) -> Result<(), SchedulingError> {
    for (goal_key, goal) in goals {
        for waitee in &goal.waitees {
            let dependency = goals.get(waitee).ok_or_else(|| SchedulingError::UnknownGoalReference {
                goal_key: goal_identity_blake3(goal_key),
                referenced_goal: goal_identity_blake3(waitee),
            })?;
            if !dependency.waiters.contains(goal_key) {
                return Err(SchedulingError::InconsistentEdge {
                    goal_key: goal_identity_blake3(goal_key),
                    referenced_goal: goal_identity_blake3(waitee),
                });
            }
        }
        for waiter in &goal.waiters {
            let dependent = goals.get(waiter).ok_or_else(|| SchedulingError::UnknownGoalReference {
                goal_key: goal_identity_blake3(goal_key),
                referenced_goal: goal_identity_blake3(waiter),
            })?;
            if !dependent.waitees.contains(goal_key) {
                return Err(SchedulingError::InconsistentEdge {
                    goal_key: goal_identity_blake3(goal_key),
                    referenced_goal: goal_identity_blake3(waiter),
                });
            }
        }
    }
    debug_assert!(goals.values().all(|goal| goal.waitees.len() <= goals.len()));
    debug_assert!(goals.values().all(|goal| goal.waiters.len() <= goals.len()));
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DurationClass {
    Structural,
    Short,
    Medium,
    Long,
}

impl DurationClass {
    fn work_units(self) -> u32 {
        match self {
            Self::Structural => STRUCTURAL_DURATION_UNITS,
            Self::Short => SHORT_DURATION_UNITS,
            Self::Medium => MEDIUM_DURATION_UNITS,
            Self::Long => LONG_DURATION_UNITS,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DurationHistorySnapshot {
    pub schema: String,
    pub policy_id: String,
    pub duration_classes: BTreeMap<String, DurationClass>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum HistoryInput {
    #[default]
    Missing,
    Fresh(DurationHistorySnapshot),
    Stale(DurationHistorySnapshot),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum HistoryBasis {
    FreshSnapshot,
    StructuralFallbackMissing,
    StructuralFallbackStale,
    StructuralFallbackIncompatible,
    StructuralFallbackOversized,
    StructuralFallbackUnknownGoal,
}

impl HistoryBasis {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FreshSnapshot => "fresh-snapshot",
            Self::StructuralFallbackMissing => "structural-fallback-missing",
            Self::StructuralFallbackStale => "structural-fallback-stale",
            Self::StructuralFallbackIncompatible => "structural-fallback-incompatible",
            Self::StructuralFallbackOversized => "structural-fallback-oversized",
            Self::StructuralFallbackUnknownGoal => "structural-fallback-unknown-goal",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedHistory {
    basis: HistoryBasis,
    snapshot_digest_blake3: Option<String>,
    duration_classes: BTreeMap<String, DurationClass>,
}

fn resolve_history(
    history: &HistoryInput,
    policy: &SchedulingPolicy,
    graph: &ValidatedKnownGraph,
) -> Result<ResolvedHistory, SchedulingError> {
    let (snapshot, stale) = match history {
        HistoryInput::Missing => {
            return Ok(structural_history(HistoryBasis::StructuralFallbackMissing, None));
        }
        HistoryInput::Fresh(snapshot) => (snapshot, false),
        HistoryInput::Stale(snapshot) => (snapshot, true),
    };
    if let Some(fallback_basis) = preflight_history_snapshot(snapshot)? {
        return Ok(structural_history(fallback_basis, None));
    }
    let Some(digest) = bounded_history_digest(snapshot)? else {
        return Ok(structural_history(HistoryBasis::StructuralFallbackOversized, None));
    };
    if stale {
        return Ok(structural_history(HistoryBasis::StructuralFallbackStale, Some(digest)));
    }
    if snapshot.schema != SCHEDULING_HISTORY_SCHEMA || snapshot.policy_id != policy.policy_id {
        return Ok(structural_history(HistoryBasis::StructuralFallbackIncompatible, Some(digest)));
    }
    if snapshot.duration_classes.keys().any(|key| !graph.goals.contains_key(key)) {
        return Ok(structural_history(HistoryBasis::StructuralFallbackUnknownGoal, Some(digest)));
    }
    debug_assert!(
        u32::try_from(snapshot.duration_classes.len()).is_ok_and(|entry_count| entry_count <= MAX_HISTORY_ENTRIES)
    );
    debug_assert!(snapshot.duration_classes.keys().all(|key| graph.goals.contains_key(key)));
    Ok(ResolvedHistory {
        basis: HistoryBasis::FreshSnapshot,
        snapshot_digest_blake3: Some(digest),
        duration_classes: snapshot.duration_classes.clone(),
    })
}

fn preflight_history_snapshot(snapshot: &DurationHistorySnapshot) -> Result<Option<HistoryBasis>, SchedulingError> {
    let entry_count = len_as_u32("history entries", snapshot.duration_classes.len())?;
    if entry_count > MAX_HISTORY_ENTRIES {
        return Ok(Some(HistoryBasis::StructuralFallbackOversized));
    }
    let schema_bytes = len_as_u32("history schema", snapshot.schema.len())?;
    let policy_id_bytes = len_as_u32("history policy id", snapshot.policy_id.len())?;
    if schema_bytes > MAX_POLICY_ID_BYTES || policy_id_bytes > MAX_POLICY_ID_BYTES {
        return Ok(Some(HistoryBasis::StructuralFallbackOversized));
    }
    if snapshot.schema.chars().any(char::is_control) || snapshot.policy_id.chars().any(char::is_control) {
        return Ok(Some(HistoryBasis::StructuralFallbackIncompatible));
    }
    let mut key_bytes_total = 0_u32;
    for goal_key in snapshot.duration_classes.keys() {
        let key_bytes = len_as_u32("history goal key", goal_key.len())?;
        if key_bytes > MAX_GOAL_KEY_BYTES {
            return Ok(Some(HistoryBasis::StructuralFallbackOversized));
        }
        if goal_key.chars().any(char::is_control) {
            return Ok(Some(HistoryBasis::StructuralFallbackIncompatible));
        }
        key_bytes_total = key_bytes_total.checked_add(key_bytes).ok_or(SchedulingError::ArithmeticOverflow {
            field: "history key bytes",
        })?;
        if key_bytes_total > MAX_HISTORY_KEY_BYTES_TOTAL {
            return Ok(Some(HistoryBasis::StructuralFallbackOversized));
        }
    }
    debug_assert!(entry_count <= MAX_HISTORY_ENTRIES);
    debug_assert!(key_bytes_total <= MAX_HISTORY_KEY_BYTES_TOTAL);
    Ok(None)
}

fn structural_history(basis: HistoryBasis, digest: Option<String>) -> ResolvedHistory {
    debug_assert_ne!(basis, HistoryBasis::FreshSnapshot);
    debug_assert!(digest.as_ref().is_none_or(|value| value.len() == BLAKE3_HEX_LENGTH));
    ResolvedHistory {
        basis,
        snapshot_digest_blake3: digest,
        duration_classes: BTreeMap::new(),
    }
}

fn bounded_history_digest(snapshot: &DurationHistorySnapshot) -> Result<Option<String>, SchedulingError> {
    let canonical = serde_json::to_vec(snapshot).map_err(|source| SchedulingError::SerializeHistory {
        reason: source.to_string(),
    })?;
    let canonical_bytes = len_as_u32("canonical history bytes", canonical.len())?;
    if canonical_bytes > MAX_HISTORY_SNAPSHOT_BYTES {
        return Ok(None);
    }
    debug_assert!(!canonical.is_empty());
    debug_assert!(canonical_bytes <= MAX_HISTORY_SNAPSHOT_BYTES);
    Ok(Some(blake3::hash(&canonical).to_hex().to_string()))
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KnownGraphPressure {
    pub known_critical_path_nodes: u32,
    pub known_critical_path_work_units: u32,
    pub blocked_root_count: u32,
    pub blocked_root_count_saturated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PressureUpdate {
    pub pressures: BTreeMap<String, KnownGraphPressure>,
    pub affected_goal_count: u32,
    pub history_basis: HistoryBasis,
    pub history_snapshot_digest_blake3: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct PressureAccumulator {
    path_nodes: u32,
    path_work_units: u32,
    roots: BTreeSet<u32>,
    roots_saturated: bool,
}

pub fn recompute_affected_pressure(
    graph: &KnownGraphFacts,
    previous: &BTreeMap<String, KnownGraphPressure>,
    changed_goal_keys: &BTreeSet<String>,
    policy: &SchedulingPolicy,
    history: &HistoryInput,
) -> Result<PressureUpdate, SchedulingError> {
    policy.validate()?;
    let graph = validate_known_graph(graph)?;
    validate_previous_pressures(previous)?;
    let resolved_history = resolve_history(history, policy, &graph)?;
    let affected = affected_goal_keys(&graph, previous, changed_goal_keys)?;
    let mut pressures = previous.clone();
    pressures.retain(|goal_key, _| graph.goals.contains_key(goal_key));
    if !affected.is_empty() {
        let recomputed = compute_component_pressures(&graph, &affected, &resolved_history)?;
        for (goal_key, pressure) in recomputed {
            pressures.insert(goal_key, pressure);
        }
    }
    let affected_goal_count = len_as_u32("affected goals", affected.len())?;
    debug_assert_eq!(pressures.len(), graph.goals.len());
    debug_assert!(affected_goal_count <= MAX_SCHEDULING_GOALS);
    Ok(PressureUpdate {
        pressures,
        affected_goal_count,
        history_basis: resolved_history.basis,
        history_snapshot_digest_blake3: resolved_history.snapshot_digest_blake3,
    })
}

fn validate_previous_pressures(previous: &BTreeMap<String, KnownGraphPressure>) -> Result<(), SchedulingError> {
    let previous_count = len_as_u32("previous pressure goals", previous.len())?;
    if previous_count > MAX_SCHEDULING_GOALS {
        return Err(SchedulingError::TooManyGoals {
            actual: previous_count,
            max: MAX_SCHEDULING_GOALS,
        });
    }
    for (goal_key, pressure) in previous {
        validate_goal_key(GoalKeyInput {
            field: "previous pressure goal key",
            value: goal_key,
        })?;
        validate_known_graph_pressure(goal_key, pressure)?;
    }
    debug_assert!(previous_count <= MAX_SCHEDULING_GOALS);
    debug_assert!(previous.keys().all(|goal_key| !goal_key.is_empty()));
    Ok(())
}

fn affected_goal_keys(
    graph: &ValidatedKnownGraph,
    previous: &BTreeMap<String, KnownGraphPressure>,
    changed_goal_keys: &BTreeSet<String>,
) -> Result<BTreeSet<String>, SchedulingError> {
    let changed_goal_count = len_as_u32("changed scheduling goals", changed_goal_keys.len())?;
    if changed_goal_count > MAX_SCHEDULING_GOALS {
        return Err(SchedulingError::TooManyGoals {
            actual: changed_goal_count,
            max: MAX_SCHEDULING_GOALS,
        });
    }
    for goal_key in changed_goal_keys {
        validate_goal_key(GoalKeyInput {
            field: "changed scheduling goal key",
            value: goal_key,
        })?;
    }
    let is_previous_key_set_valid = previous.keys().all(|goal_key| graph.goals.contains_key(goal_key));
    let is_every_missing_key_changed = graph
        .goals
        .keys()
        .filter(|goal_key| !previous.contains_key(*goal_key))
        .all(|goal_key| changed_goal_keys.contains(goal_key));
    if !is_previous_key_set_valid || !is_every_missing_key_changed {
        return Ok(graph.goals.keys().cloned().collect());
    }
    if changed_goal_keys.is_empty() {
        return Ok(BTreeSet::new());
    }
    let mut affected = BTreeSet::new();
    let mut frontier = BTreeSet::new();
    for goal_key in changed_goal_keys {
        if !graph.goals.contains_key(goal_key) {
            return Err(SchedulingError::UnknownChangedGoal {
                goal_key: goal_identity_blake3(goal_key),
            });
        }
        frontier.insert(goal_key.clone());
    }
    while let Some(goal_key) = frontier.pop_first() {
        if !affected.insert(goal_key.clone()) {
            continue;
        }
        let goal = graph.goals.get(&goal_key).ok_or_else(|| SchedulingError::UnknownChangedGoal {
            goal_key: goal_identity_blake3(&goal_key),
        })?;
        frontier.extend(goal.waitees.iter().filter(|key| !affected.contains(*key)).cloned());
        frontier.extend(goal.waiters.iter().filter(|key| !affected.contains(*key)).cloned());
        if len_as_u32("affected goals", affected.len())? > MAX_SCHEDULING_GOALS {
            return Err(SchedulingError::TooManyGoals {
                actual: len_as_u32("affected goals", affected.len())?,
                max: MAX_SCHEDULING_GOALS,
            });
        }
    }
    debug_assert!(affected.iter().all(|goal_key| graph.goals.contains_key(goal_key)));
    debug_assert!(affected.len() <= graph.goals.len());
    debug_assert!(changed_goal_count <= MAX_SCHEDULING_GOALS);
    Ok(affected)
}

fn compute_component_pressures(
    graph: &ValidatedKnownGraph,
    affected: &BTreeSet<String>,
    history: &ResolvedHistory,
) -> Result<BTreeMap<String, KnownGraphPressure>, SchedulingError> {
    let topological = topological_order(graph, affected)?;
    let root_ids = requested_root_ids(graph)?;
    let mut accumulators: BTreeMap<String, PressureAccumulator> = affected
        .iter()
        .map(|goal_key| {
            root_accumulator(goal_key, graph, history, &root_ids).map(|accumulator| (goal_key.clone(), accumulator))
        })
        .collect::<Result<_, SchedulingError>>()?;
    for goal_key in topological.iter().rev() {
        propagate_goal_pressure(goal_key, graph, affected, history, &mut accumulators)?;
    }
    let pressures: BTreeMap<String, KnownGraphPressure> = accumulators
        .into_iter()
        .map(|(goal_key, accumulator)| (goal_key, pressure_from_accumulator(accumulator)))
        .collect();
    debug_assert_eq!(topological.len(), affected.len());
    debug_assert_eq!(pressures.len(), affected.len());
    Ok(pressures)
}

fn requested_root_ids(graph: &ValidatedKnownGraph) -> Result<BTreeMap<String, u32>, SchedulingError> {
    let requested_root_count =
        len_as_u32("requested roots", graph.goals.values().filter(|goal| goal.requested_root).count())?;
    if requested_root_count > MAX_SCHEDULING_GOALS {
        return Err(SchedulingError::TooManyGoals {
            actual: requested_root_count,
            max: MAX_SCHEDULING_GOALS,
        });
    }
    let root_ids = graph
        .goals
        .iter()
        .filter(|(_, goal)| goal.requested_root)
        .enumerate()
        .map(|(root_id, (goal_key, _))| {
            u32::try_from(root_id).map(|root_id| (goal_key.clone(), root_id)).map_err(|_| {
                SchedulingError::LengthOverflow {
                    field: "requested root id",
                }
            })
        })
        .collect::<Result<BTreeMap<_, _>, SchedulingError>>()?;
    debug_assert_eq!(u32::try_from(root_ids.len()), Ok(requested_root_count));
    debug_assert!(root_ids.keys().all(|goal_key| graph.goals.contains_key(goal_key)));
    Ok(root_ids)
}

fn root_accumulator(
    goal_key: &str,
    graph: &ValidatedKnownGraph,
    history: &ResolvedHistory,
    root_ids: &BTreeMap<String, u32>,
) -> Result<PressureAccumulator, SchedulingError> {
    let is_requested_root = graph.goals.get(goal_key).is_some_and(|goal| goal.requested_root);
    if !is_requested_root {
        return Ok(PressureAccumulator::default());
    }
    let mut roots = BTreeSet::new();
    let root_id = root_ids.get(goal_key).copied().ok_or_else(|| SchedulingError::MissingPressure {
        goal_key: goal_identity_blake3(goal_key),
    })?;
    roots.insert(root_id);
    let work_units = duration_units(goal_key, history);
    debug_assert_eq!(roots.len(), 1);
    debug_assert!(work_units >= STRUCTURAL_DURATION_UNITS);
    Ok(PressureAccumulator {
        path_nodes: 1,
        path_work_units: work_units,
        roots,
        roots_saturated: false,
    })
}

fn topological_order(graph: &ValidatedKnownGraph, affected: &BTreeSet<String>) -> Result<Vec<String>, SchedulingError> {
    let mut remaining_dependencies = affected
        .iter()
        .map(|goal_key| {
            let goal = graph.goals.get(goal_key).ok_or_else(|| SchedulingError::UnknownChangedGoal {
                goal_key: goal_identity_blake3(goal_key),
            })?;
            let dependency_count = len_as_u32(
                "component dependencies",
                goal.waitees.iter().filter(|waitee| affected.contains(*waitee)).count(),
            )?;
            Ok((goal_key.clone(), dependency_count))
        })
        .collect::<Result<BTreeMap<_, _>, SchedulingError>>()?;
    let mut ready: BTreeSet<String> = remaining_dependencies
        .iter()
        .filter(|(_, dependency_count)| **dependency_count == 0)
        .map(|(goal_key, _)| goal_key.clone())
        .collect();
    let mut ordered = Vec::with_capacity(affected.len());
    while let Some(goal_key) = ready.pop_first() {
        ordered.push(goal_key.clone());
        let goal = graph.goals.get(&goal_key).ok_or_else(|| SchedulingError::UnknownChangedGoal {
            goal_key: goal_identity_blake3(&goal_key),
        })?;
        for waiter in goal.waiters.iter().filter(|waiter| affected.contains(*waiter)) {
            let remaining =
                remaining_dependencies.get_mut(waiter).ok_or_else(|| SchedulingError::UnknownChangedGoal {
                    goal_key: goal_identity_blake3(waiter),
                })?;
            *remaining = remaining.checked_sub(1).ok_or(SchedulingError::ArithmeticOverflow {
                field: "topological remaining dependencies",
            })?;
            if *remaining == 0 {
                ready.insert(waiter.clone());
            }
        }
    }
    if ordered.len() != affected.len() {
        return Err(SchedulingError::KnownGraphCycle {
            affected_goal_count: len_as_u32("affected goals", affected.len())?,
        });
    }
    debug_assert_eq!(ordered.len(), affected.len());
    debug_assert!(remaining_dependencies.values().all(|remaining| *remaining == 0));
    Ok(ordered)
}

fn propagate_goal_pressure(
    goal_key: &str,
    graph: &ValidatedKnownGraph,
    affected: &BTreeSet<String>,
    history: &ResolvedHistory,
    accumulators: &mut BTreeMap<String, PressureAccumulator>,
) -> Result<(), SchedulingError> {
    let source = accumulators.get(goal_key).cloned().ok_or_else(|| SchedulingError::MissingPressure {
        goal_key: goal_identity_blake3(goal_key),
    })?;
    if source.path_nodes == 0 {
        return Ok(());
    }
    let goal = graph.goals.get(goal_key).ok_or_else(|| SchedulingError::MissingPressure {
        goal_key: goal_identity_blake3(goal_key),
    })?;
    for waitee in goal.waitees.iter().filter(|waitee| affected.contains(*waitee)) {
        let waitee_work_unit_count = duration_units(waitee, history);
        let target = accumulators.get_mut(waitee).ok_or_else(|| SchedulingError::MissingPressure {
            goal_key: goal_identity_blake3(waitee),
        })?;
        merge_pressure(target, &source, waitee_work_unit_count)?;
    }
    debug_assert!(source.path_nodes > 0);
    debug_assert!(source.path_work_units > 0);
    Ok(())
}

fn merge_pressure(
    target: &mut PressureAccumulator,
    source: &PressureAccumulator,
    target_duration_units: u32,
) -> Result<(), SchedulingError> {
    let candidate_nodes = source.path_nodes.checked_add(1).ok_or(SchedulingError::ArithmeticOverflow {
        field: "known critical path nodes",
    })?;
    let candidate_work =
        source
            .path_work_units
            .checked_add(target_duration_units)
            .ok_or(SchedulingError::ArithmeticOverflow {
                field: "known critical path work units",
            })?;
    target.path_nodes = target.path_nodes.max(candidate_nodes);
    target.path_work_units = target.path_work_units.max(candidate_work);
    if source.roots_saturated {
        target.roots_saturated = true;
    }
    for root in &source.roots {
        if target.roots.contains(root) {
            continue;
        }
        if len_as_u32("blocked roots", target.roots.len())? >= MAX_BLOCKED_ROOT_PRESSURE {
            target.roots_saturated = true;
            break;
        }
        target.roots.insert(*root);
    }
    debug_assert!(target.path_nodes >= candidate_nodes);
    debug_assert!(target.path_work_units >= candidate_work);
    Ok(())
}

fn duration_units(goal_key: &str, history: &ResolvedHistory) -> u32 {
    let class = history.duration_classes.get(goal_key).copied().unwrap_or(DurationClass::Structural);
    let units = class.work_units();
    debug_assert!(units >= STRUCTURAL_DURATION_UNITS);
    debug_assert!(units <= LONG_DURATION_UNITS);
    units
}

fn pressure_from_accumulator(accumulator: PressureAccumulator) -> KnownGraphPressure {
    let blocked_root_count = u32::try_from(accumulator.roots.len()).unwrap_or(MAX_BLOCKED_ROOT_PRESSURE);
    debug_assert!(blocked_root_count <= MAX_BLOCKED_ROOT_PRESSURE);
    debug_assert_eq!(accumulator.path_nodes == 0, accumulator.path_work_units == 0);
    KnownGraphPressure {
        known_critical_path_nodes: accumulator.path_nodes,
        known_critical_path_work_units: accumulator.path_work_units,
        blocked_root_count,
        blocked_root_count_saturated: accumulator.roots_saturated,
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum OperatorPolicyClass {
    Background,
    #[default]
    Ordinary,
    Urgent,
}

impl OperatorPolicyClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Background => "background",
            Self::Ordinary => "ordinary",
            Self::Urgent => "urgent",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResourceFitClass {
    #[default]
    Unknown,
    Constrained,
    Compatible,
    Exact,
}

impl ResourceFitClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Constrained => "constrained",
            Self::Compatible => "compatible",
            Self::Exact => "exact",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ContentLocalityClass {
    #[default]
    Unknown,
    NoVerifiedContent,
    PartiallyPresent,
    FullyPresent,
}

impl ContentLocalityClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::NoVerifiedContent => "no-verified-content",
            Self::PartiallyPresent => "partially-present",
            Self::FullyPresent => "fully-present",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TransferCostClass {
    #[default]
    Unknown,
    Large,
    Medium,
    Small,
    None,
}

impl TransferCostClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Large => "large",
            Self::Medium => "medium",
            Self::Small => "small",
            Self::None => "none",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HardEligibilityFacts {
    pub capability_allowed: bool,
    pub output_trust_allowed: bool,
    pub upload_allowed: bool,
    pub network_allowed: bool,
    pub store_prefix_matches: bool,
    pub hard_resource_fit: bool,
}

impl HardEligibilityFacts {
    pub const ELIGIBLE: Self = Self {
        capability_allowed: true,
        output_trust_allowed: true,
        upload_allowed: true,
        network_allowed: true,
        store_prefix_matches: true,
        hard_resource_fit: true,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IneligibleReason {
    MissingCapability,
    OutputTrustRejected,
    UploadPolicyRejected,
    NetworkPolicyRejected,
    StorePrefixMismatch,
    HardResourceMismatch,
}

impl IneligibleReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MissingCapability => "missing-capability",
            Self::OutputTrustRejected => "output-trust-rejected",
            Self::UploadPolicyRejected => "upload-policy-rejected",
            Self::NetworkPolicyRejected => "network-policy-rejected",
            Self::StorePrefixMismatch => "store-prefix-mismatch",
            Self::HardResourceMismatch => "hard-resource-mismatch",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EligiblePreferenceFacts {
    pub(crate) resource_fit: ResourceFitClass,
    pub(crate) content_locality: ContentLocalityClass,
    pub(crate) transfer_cost: TransferCostClass,
}

impl EligiblePreferenceFacts {
    pub fn resource_fit(self) -> ResourceFitClass {
        self.resource_fit
    }

    pub fn content_locality(self) -> ContentLocalityClass {
        self.content_locality
    }

    pub fn transfer_cost(self) -> TransferCostClass {
        self.transfer_cost
    }
}

pub fn normalize_eligible_preference(
    eligibility: HardEligibilityFacts,
    resource_fit: ResourceFitClass,
    content_locality: ContentLocalityClass,
    transfer_cost: TransferCostClass,
) -> Result<EligiblePreferenceFacts, IneligibleReason> {
    if !eligibility.capability_allowed {
        return Err(IneligibleReason::MissingCapability);
    }
    if !eligibility.output_trust_allowed {
        return Err(IneligibleReason::OutputTrustRejected);
    }
    if !eligibility.upload_allowed {
        return Err(IneligibleReason::UploadPolicyRejected);
    }
    if !eligibility.network_allowed {
        return Err(IneligibleReason::NetworkPolicyRejected);
    }
    if !eligibility.store_prefix_matches {
        return Err(IneligibleReason::StorePrefixMismatch);
    }
    if !eligibility.hard_resource_fit {
        return Err(IneligibleReason::HardResourceMismatch);
    }
    debug_assert!(eligibility.capability_allowed);
    debug_assert!(eligibility.hard_resource_fit);
    Ok(EligiblePreferenceFacts {
        resource_fit,
        content_locality,
        transfer_cost,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum StarvationClass {
    Fresh,
    Aged,
    Protected,
}

impl StarvationClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fresh => "fresh",
            Self::Aged => "aged",
            Self::Protected => "protected",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadyGoalFacts {
    pub goal_key: String,
    pub ready_since_epoch: u32,
    pub operator_policy_class: OperatorPolicyClass,
    pub preference: EligiblePreferenceFacts,
}

impl ReadyGoalFacts {
    pub fn ordinary(goal_key: String, ready_since_epoch: u32) -> Self {
        debug_assert!(!goal_key.is_empty());
        debug_assert!(ready_since_epoch <= MAX_SCHEDULING_EPOCH);
        Self {
            goal_key,
            ready_since_epoch,
            operator_policy_class: OperatorPolicyClass::Ordinary,
            preference: EligiblePreferenceFacts::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PriorityTuple {
    pub operator_policy_class: OperatorPolicyClass,
    pub starvation_class: StarvationClass,
    pub age_epochs: u32,
    pub known_graph_pressure: KnownGraphPressure,
    pub resource_fit: ResourceFitClass,
    pub content_locality: ContentLocalityClass,
    pub transfer_cost: TransferCostClass,
    pub stable_goal_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RankedReadyGoal {
    pub goal_key: String,
    pub priority: PriorityTuple,
}

pub fn rank_ready_goals(
    policy: &SchedulingPolicy,
    scheduling_epoch: u32,
    ready_goals: &[ReadyGoalFacts],
    pressures: &BTreeMap<String, KnownGraphPressure>,
) -> Result<Vec<RankedReadyGoal>, SchedulingError> {
    policy.validate()?;
    if scheduling_epoch > MAX_SCHEDULING_EPOCH {
        return Err(SchedulingError::EpochLimitExceeded {
            actual: scheduling_epoch,
            max: MAX_SCHEDULING_EPOCH,
        });
    }
    let ready_count = len_as_u32("ready goals", ready_goals.len())?;
    if ready_count > MAX_READY_GOALS {
        return Err(SchedulingError::TooManyReadyGoals {
            actual: ready_count,
            max: MAX_READY_GOALS,
        });
    }
    let mut seen = BTreeSet::new();
    let mut ranked = Vec::with_capacity(ready_goals.len());
    for ready in ready_goals {
        validate_goal_key(GoalKeyInput {
            field: "ready goal key",
            value: &ready.goal_key,
        })?;
        if !seen.insert(ready.goal_key.clone()) {
            return Err(SchedulingError::DuplicateReadyGoal {
                goal_key: goal_identity_blake3(&ready.goal_key),
            });
        }
        let pressure = pressures.get(&ready.goal_key).cloned().ok_or_else(|| SchedulingError::MissingPressure {
            goal_key: goal_identity_blake3(&ready.goal_key),
        })?;
        validate_known_graph_pressure(&ready.goal_key, &pressure)?;
        ranked.push(rank_one_ready_goal(policy, scheduling_epoch, ready, pressure)?);
    }
    ranked.sort_by(|left, right| compare_ranked_ready_goals(policy, left, right));
    debug_assert_eq!(ranked.len(), ready_goals.len());
    debug_assert!(
        ranked
            .windows(ADJACENT_PAIR_WINDOW)
            .all(|pair| compare_ranked_ready_goals(policy, &pair[0], &pair[1]) != Ordering::Greater)
    );
    Ok(ranked)
}

fn validate_known_graph_pressure(goal_key: &str, pressure: &KnownGraphPressure) -> Result<(), SchedulingError> {
    let is_node_work_zero_state_consistent =
        (pressure.known_critical_path_nodes == 0) == (pressure.known_critical_path_work_units == 0);
    if !is_node_work_zero_state_consistent {
        return Err(invalid_pressure(PressureViolation {
            goal_key,
            reason: "critical-path node/work zero state disagrees",
        }));
    }
    if pressure.known_critical_path_nodes > MAX_SCHEDULING_GOALS {
        return Err(invalid_pressure(PressureViolation {
            goal_key,
            reason: "critical-path node count exceeds known-goal bound",
        }));
    }
    if pressure.known_critical_path_work_units > MAX_KNOWN_CRITICAL_PATH_WORK_UNITS {
        return Err(invalid_pressure(PressureViolation {
            goal_key,
            reason: "critical-path work exceeds bounded duration classes",
        }));
    }
    if pressure.blocked_root_count > MAX_BLOCKED_ROOT_PRESSURE {
        return Err(invalid_pressure(PressureViolation {
            goal_key,
            reason: "blocked-root count exceeds pressure bound",
        }));
    }
    if pressure.blocked_root_count_saturated && pressure.blocked_root_count != MAX_BLOCKED_ROOT_PRESSURE {
        return Err(invalid_pressure(PressureViolation {
            goal_key,
            reason: "saturated blocked-root count is not at its bound",
        }));
    }
    if pressure.blocked_root_count > 0 && pressure.known_critical_path_nodes == 0 {
        return Err(invalid_pressure(PressureViolation {
            goal_key,
            reason: "blocked-root pressure has no known path",
        }));
    }
    debug_assert!(is_node_work_zero_state_consistent);
    debug_assert!(pressure.blocked_root_count <= MAX_BLOCKED_ROOT_PRESSURE);
    Ok(())
}

struct PressureViolation<'a> {
    goal_key: &'a str,
    reason: &'static str,
}

fn invalid_pressure(violation: PressureViolation<'_>) -> SchedulingError {
    debug_assert!(!violation.goal_key.is_empty());
    debug_assert!(!violation.reason.is_empty());
    SchedulingError::InvalidPressure {
        goal_key: goal_identity_blake3(violation.goal_key),
        reason: violation.reason,
    }
}

fn rank_one_ready_goal(
    policy: &SchedulingPolicy,
    scheduling_epoch: u32,
    ready: &ReadyGoalFacts,
    pressure: KnownGraphPressure,
) -> Result<RankedReadyGoal, SchedulingError> {
    let age_epochs =
        scheduling_epoch
            .checked_sub(ready.ready_since_epoch)
            .ok_or_else(|| SchedulingError::EpochRegression {
                goal_key: goal_identity_blake3(&ready.goal_key),
                ready_since_epoch: ready.ready_since_epoch,
                scheduling_epoch,
            })?;
    let starvation_class = starvation_class(policy, age_epochs);
    debug_assert!(age_epochs <= scheduling_epoch);
    debug_assert!(!ready.goal_key.is_empty());
    Ok(RankedReadyGoal {
        goal_key: ready.goal_key.clone(),
        priority: PriorityTuple {
            operator_policy_class: ready.operator_policy_class,
            starvation_class,
            age_epochs,
            known_graph_pressure: pressure,
            resource_fit: ready.preference.resource_fit,
            content_locality: ready.preference.content_locality,
            transfer_cost: ready.preference.transfer_cost,
            stable_goal_key: ready.goal_key.clone(),
        },
    })
}

fn starvation_class(policy: &SchedulingPolicy, age_epochs: u32) -> StarvationClass {
    let class = if age_epochs >= policy.protected_after_epochs {
        StarvationClass::Protected
    } else if age_epochs >= policy.aged_after_epochs {
        StarvationClass::Aged
    } else {
        StarvationClass::Fresh
    };
    debug_assert!(policy.aged_after_epochs < policy.protected_after_epochs);
    debug_assert!(age_epochs < policy.aged_after_epochs || class != StarvationClass::Fresh);
    class
}

pub fn compare_ranked_ready_goals(
    policy: &SchedulingPolicy,
    left: &RankedReadyGoal,
    right: &RankedReadyGoal,
) -> Ordering {
    compare_priority_tuples(policy, &left.priority, &right.priority)
}

pub fn compare_priority_tuples(policy: &SchedulingPolicy, left: &PriorityTuple, right: &PriorityTuple) -> Ordering {
    let operator = compare_desc(left.operator_policy_class, right.operator_policy_class);
    if operator != Ordering::Equal {
        return operator;
    }
    let starvation = compare_desc(left.starvation_class, right.starvation_class);
    if starvation != Ordering::Equal {
        return starvation;
    }
    for field in &policy.preference_order {
        let ordering = compare_preference_field(*field, left, right);
        if ordering != Ordering::Equal {
            return ordering;
        }
    }
    left.stable_goal_key.cmp(&right.stable_goal_key)
}

fn compare_preference_field(field: PreferenceField, left: &PriorityTuple, right: &PriorityTuple) -> Ordering {
    match field {
        PreferenceField::KnownGraph => {
            compare_known_graph_pressure(&left.known_graph_pressure, &right.known_graph_pressure)
        }
        PreferenceField::ResourceFit => compare_desc(left.resource_fit, right.resource_fit),
        PreferenceField::LocalityTransfer => compare_locality_transfer(left, right),
    }
}

fn compare_known_graph_pressure(left: &KnownGraphPressure, right: &KnownGraphPressure) -> Ordering {
    compare_desc(left.blocked_root_count, right.blocked_root_count)
        .then_with(|| compare_desc(left.blocked_root_count_saturated, right.blocked_root_count_saturated))
        .then_with(|| compare_desc(left.known_critical_path_work_units, right.known_critical_path_work_units))
        .then_with(|| compare_desc(left.known_critical_path_nodes, right.known_critical_path_nodes))
}

fn compare_locality_transfer(left: &PriorityTuple, right: &PriorityTuple) -> Ordering {
    compare_desc(left.content_locality, right.content_locality)
        .then_with(|| compare_desc(left.transfer_cost, right.transfer_cost))
}

fn compare_desc<T: Ord>(left: T, right: T) -> Ordering {
    right.cmp(&left)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PrioritySelectionReason {
    SoleCandidate,
    OperatorPolicyClass,
    StarvationClass,
    KnownGraphPressure,
    ResourceFitClass,
    LocalityTransferClass,
    StableGoalKey,
}

impl PrioritySelectionReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SoleCandidate => "sole-candidate",
            Self::OperatorPolicyClass => "operator-policy-class",
            Self::StarvationClass => "starvation-class",
            Self::KnownGraphPressure => "known-graph-pressure",
            Self::ResourceFitClass => "resource-fit-class",
            Self::LocalityTransferClass => "locality-transfer-class",
            Self::StableGoalKey => "stable-goal-key",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PriorityCandidateEvidence {
    pub goal_key_blake3: String,
    pub operator_policy_class: OperatorPolicyClass,
    pub starvation_class: StarvationClass,
    pub age_epochs: u32,
    pub known_critical_path_nodes: u32,
    pub known_critical_path_work_units: u32,
    pub blocked_root_count: u32,
    pub blocked_root_count_saturated: bool,
    pub resource_fit_class: ResourceFitClass,
    pub content_locality_class: ContentLocalityClass,
    pub transfer_cost_class: TransferCostClass,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PriorityDecisionEvidence {
    pub schema: String,
    pub policy_id: String,
    pub policy_digest_blake3: String,
    pub scheduling_epoch: u32,
    pub selected_goal_key_blake3: String,
    pub competing_goal_count: u32,
    pub candidate_snapshot_digest_blake3: String,
    pub runner_up: Option<PriorityCandidateEvidence>,
    pub known_graph_basis: String,
    pub history_basis: HistoryBasis,
    pub history_snapshot_digest_blake3: Option<String>,
    pub operator_policy_class: OperatorPolicyClass,
    pub starvation_class: StarvationClass,
    pub age_epochs: u32,
    pub known_critical_path_nodes: u32,
    pub known_critical_path_work_units: u32,
    pub blocked_root_count: u32,
    pub blocked_root_count_saturated: bool,
    pub resource_fit_class: ResourceFitClass,
    pub content_locality_class: ContentLocalityClass,
    pub transfer_cost_class: TransferCostClass,
    pub selection_reason: PrioritySelectionReason,
    pub stable_tie_break_class: String,
    pub claim_scope: String,
    pub non_claims: Vec<String>,
}

fn priority_candidate_evidence(candidate: &RankedReadyGoal) -> PriorityCandidateEvidence {
    let pressure = &candidate.priority.known_graph_pressure;
    let evidence = PriorityCandidateEvidence {
        goal_key_blake3: goal_identity_blake3(&candidate.goal_key),
        operator_policy_class: candidate.priority.operator_policy_class,
        starvation_class: candidate.priority.starvation_class,
        age_epochs: candidate.priority.age_epochs,
        known_critical_path_nodes: pressure.known_critical_path_nodes,
        known_critical_path_work_units: pressure.known_critical_path_work_units,
        blocked_root_count: pressure.blocked_root_count,
        blocked_root_count_saturated: pressure.blocked_root_count_saturated,
        resource_fit_class: candidate.priority.resource_fit,
        content_locality_class: candidate.priority.content_locality,
        transfer_cost_class: candidate.priority.transfer_cost,
    };
    debug_assert_eq!(evidence.goal_key_blake3.len(), BLAKE3_HEX_LENGTH);
    debug_assert!(!candidate.goal_key.is_empty());
    evidence
}

fn priority_snapshot_digest(ranked: &[RankedReadyGoal]) -> Result<String, SchedulingError> {
    let candidates: Vec<PriorityCandidateEvidence> = ranked.iter().map(priority_candidate_evidence).collect();
    let canonical = serde_json::to_vec(&candidates).map_err(|source| SchedulingError::SerializePrioritySnapshot {
        reason: source.to_string(),
    })?;
    debug_assert_eq!(candidates.len(), ranked.len());
    debug_assert!(!canonical.is_empty());
    Ok(blake3::hash(&canonical).to_hex().to_string())
}

fn validate_ranked_snapshot(
    policy: &SchedulingPolicy,
    scheduling_epoch: u32,
    ranked: &[RankedReadyGoal],
) -> Result<(), SchedulingError> {
    policy.validate()?;
    let candidate_count = len_as_u32("ranked ready goals", ranked.len())?;
    if candidate_count > MAX_READY_GOALS {
        return Err(SchedulingError::TooManyReadyGoals {
            actual: candidate_count,
            max: MAX_READY_GOALS,
        });
    }
    if scheduling_epoch > MAX_SCHEDULING_EPOCH {
        return Err(SchedulingError::EpochLimitExceeded {
            actual: scheduling_epoch,
            max: MAX_SCHEDULING_EPOCH,
        });
    }
    let mut identities = BTreeSet::new();
    for candidate in ranked {
        validate_goal_key(GoalKeyInput {
            field: "ranked goal key",
            value: &candidate.goal_key,
        })?;
        if candidate.goal_key != candidate.priority.stable_goal_key {
            return Err(invalid_ranked_snapshot("candidate identity disagrees with stable tie-break key"));
        }
        if candidate.priority.age_epochs > scheduling_epoch {
            return Err(invalid_ranked_snapshot("candidate age exceeds scheduling epoch"));
        }
        if candidate.priority.starvation_class != starvation_class(policy, candidate.priority.age_epochs) {
            return Err(invalid_ranked_snapshot("candidate starvation class disagrees with policy age thresholds"));
        }
        validate_known_graph_pressure(&candidate.goal_key, &candidate.priority.known_graph_pressure)?;
        if !identities.insert(candidate.goal_key.as_str()) {
            return Err(invalid_ranked_snapshot("duplicate candidate identity"));
        }
    }
    if ranked
        .windows(ADJACENT_PAIR_WINDOW)
        .any(|pair| compare_ranked_ready_goals(policy, &pair[0], &pair[1]) == Ordering::Greater)
    {
        return Err(invalid_ranked_snapshot("candidates are not in comparator order"));
    }
    debug_assert_eq!(identities.len(), ranked.len());
    debug_assert!(candidate_count <= MAX_READY_GOALS);
    debug_assert!(scheduling_epoch <= MAX_SCHEDULING_EPOCH);
    Ok(())
}

fn invalid_ranked_snapshot(reason: &'static str) -> SchedulingError {
    debug_assert!(!reason.is_empty());
    debug_assert!(!reason.chars().any(char::is_control));
    SchedulingError::InvalidRankedSnapshot {
        reason: reason.to_string(),
    }
}

pub fn priority_decision_evidence(
    policy: &SchedulingPolicy,
    scheduling_epoch: u32,
    ranked: &[RankedReadyGoal],
    history_basis: HistoryBasis,
    history_snapshot_digest_blake3: Option<String>,
) -> Result<PriorityDecisionEvidence, SchedulingError> {
    validate_ranked_snapshot(policy, scheduling_epoch, ranked)?;
    validate_history_evidence(history_basis, history_snapshot_digest_blake3.as_deref())?;
    let selected = ranked.first().ok_or(SchedulingError::NoReadyCandidate)?;
    let competing_goal_count = len_as_u32("ranked ready goals", ranked.len())?;
    if competing_goal_count > MAX_READY_GOALS {
        return Err(SchedulingError::TooManyReadyGoals {
            actual: competing_goal_count,
            max: MAX_READY_GOALS,
        });
    }
    let selection_reason = ranked
        .get(1)
        .map(|runner_up| first_selection_reason(policy, &selected.priority, &runner_up.priority))
        .unwrap_or(PrioritySelectionReason::SoleCandidate);
    let pressure = &selected.priority.known_graph_pressure;
    let selected_candidate = priority_candidate_evidence(selected);
    let runner_up = ranked.get(1).map(priority_candidate_evidence);
    let candidate_snapshot_digest_blake3 = priority_snapshot_digest(ranked)?;
    let evidence = PriorityDecisionEvidence {
        schema: PRIORITY_DECISION_SCHEMA.to_string(),
        policy_id: policy.policy_id.clone(),
        policy_digest_blake3: policy.digest_blake3()?,
        scheduling_epoch,
        selected_goal_key_blake3: selected_candidate.goal_key_blake3,
        competing_goal_count,
        candidate_snapshot_digest_blake3,
        runner_up,
        known_graph_basis: KNOWN_GRAPH_BASIS.to_string(),
        history_basis,
        history_snapshot_digest_blake3,
        operator_policy_class: selected.priority.operator_policy_class,
        starvation_class: selected.priority.starvation_class,
        age_epochs: selected.priority.age_epochs,
        known_critical_path_nodes: pressure.known_critical_path_nodes,
        known_critical_path_work_units: pressure.known_critical_path_work_units,
        blocked_root_count: pressure.blocked_root_count,
        blocked_root_count_saturated: pressure.blocked_root_count_saturated,
        resource_fit_class: selected.priority.resource_fit,
        content_locality_class: selected.priority.content_locality,
        transfer_cost_class: selected.priority.transfer_cost,
        selection_reason,
        stable_tie_break_class: "stable-goal-key".to_string(),
        claim_scope: SCHEDULING_CLAIM_SCOPE.to_string(),
        non_claims: PRIORITY_NON_CLAIMS.iter().map(|value| (*value).to_string()).collect(),
    };
    debug_assert_eq!(evidence.selected_goal_key_blake3.len(), BLAKE3_HEX_LENGTH);
    debug_assert_eq!(evidence.non_claims.len(), PRIORITY_NON_CLAIM_COUNT);
    Ok(evidence)
}

fn validate_history_evidence(basis: HistoryBasis, digest_blake3: Option<&str>) -> Result<(), SchedulingError> {
    if let Some(digest) = digest_blake3 {
        let expected_bytes = BLAKE3_HEX_LENGTH;
        let is_lowercase_hex = digest.bytes().all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'));
        if digest.len() != expected_bytes || !is_lowercase_hex {
            return Err(SchedulingError::InvalidHistoryEvidence {
                reason: "snapshot digest must be 64 lowercase hexadecimal characters",
            });
        }
    }
    let is_digest_required = matches!(
        basis,
        HistoryBasis::FreshSnapshot
            | HistoryBasis::StructuralFallbackStale
            | HistoryBasis::StructuralFallbackUnknownGoal
    );
    let is_digest_forbidden =
        matches!(basis, HistoryBasis::StructuralFallbackMissing | HistoryBasis::StructuralFallbackOversized);
    if is_digest_required && digest_blake3.is_none() {
        return Err(SchedulingError::InvalidHistoryEvidence {
            reason: "history basis requires an admitted snapshot digest",
        });
    }
    if is_digest_forbidden && digest_blake3.is_some() {
        return Err(SchedulingError::InvalidHistoryEvidence {
            reason: "history basis forbids a snapshot digest",
        });
    }
    debug_assert!(!is_digest_required || digest_blake3.is_some());
    debug_assert!(!is_digest_forbidden || digest_blake3.is_none());
    Ok(())
}

fn first_selection_reason(
    policy: &SchedulingPolicy,
    selected: &PriorityTuple,
    runner_up: &PriorityTuple,
) -> PrioritySelectionReason {
    if selected.operator_policy_class != runner_up.operator_policy_class {
        return PrioritySelectionReason::OperatorPolicyClass;
    }
    if selected.starvation_class != runner_up.starvation_class {
        return PrioritySelectionReason::StarvationClass;
    }
    for field in &policy.preference_order {
        if compare_preference_field(*field, selected, runner_up) == Ordering::Equal {
            continue;
        }
        return match field {
            PreferenceField::KnownGraph => PrioritySelectionReason::KnownGraphPressure,
            PreferenceField::ResourceFit => PrioritySelectionReason::ResourceFitClass,
            PreferenceField::LocalityTransfer => PrioritySelectionReason::LocalityTransferClass,
        };
    }
    PrioritySelectionReason::StableGoalKey
}

pub fn advance_scheduling_epoch(current_epoch: u32) -> Result<u32, SchedulingError> {
    let next = current_epoch.checked_add(1).ok_or(SchedulingError::ArithmeticOverflow {
        field: "scheduling epoch",
    })?;
    if next > MAX_SCHEDULING_EPOCH {
        return Err(SchedulingError::EpochLimitExceeded {
            actual: next,
            max: MAX_SCHEDULING_EPOCH,
        });
    }
    debug_assert!(next > current_epoch);
    debug_assert!(next <= MAX_SCHEDULING_EPOCH);
    Ok(next)
}

fn len_as_u32(field: &'static str, len: usize) -> Result<u32, SchedulingError> {
    u32::try_from(len).map_err(|_| SchedulingError::LengthOverflow { field })
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SchedulingError {
    #[error("invalid scheduling policy: {reason}")]
    InvalidPolicy { reason: String },
    #[error("invalid scheduling identifier {field}: {reason}")]
    InvalidIdentifier { field: &'static str, reason: String },
    #[error("serializing scheduling policy: {reason}")]
    SerializePolicy { reason: String },
    #[error("serializing scheduling history: {reason}")]
    SerializeHistory { reason: String },
    #[error("serializing redacted priority snapshot: {reason}")]
    SerializePrioritySnapshot { reason: String },
    #[error("scheduling collection length overflow for {field}")]
    LengthOverflow { field: &'static str },
    #[error("known graph has {actual} goals; maximum is {max}")]
    TooManyGoals { actual: u32, max: u32 },
    #[error("known graph has {actual} edges; maximum is {max}")]
    TooManyEdges { actual: u32, max: u32 },
    #[error("duplicate known goal identity blake3:{goal_key}")]
    DuplicateGoal { goal_key: String },
    #[error("goal identity blake3:{goal_key} contains duplicate {edge_kind} edge to blake3:{referenced_goal}")]
    DuplicateEdge {
        goal_key: String,
        edge_kind: &'static str,
        referenced_goal: String,
    },
    #[error("goal identity blake3:{goal_key} references unknown goal identity blake3:{referenced_goal}")]
    UnknownGoalReference { goal_key: String, referenced_goal: String },
    #[error("goal edge between identities blake3:{goal_key} and blake3:{referenced_goal} is not reciprocal")]
    InconsistentEdge { goal_key: String, referenced_goal: String },
    #[error("changed scheduling goal identity is unknown: blake3:{goal_key}")]
    UnknownChangedGoal { goal_key: String },
    #[error("known graph component contains a cycle across {affected_goal_count} goals")]
    KnownGraphCycle { affected_goal_count: u32 },
    #[error("scheduling arithmetic overflow in {field}")]
    ArithmeticOverflow { field: &'static str },
    #[error("ready set has {actual} goals; maximum is {max}")]
    TooManyReadyGoals { actual: u32, max: u32 },
    #[error("duplicate ready goal identity blake3:{goal_key}")]
    DuplicateReadyGoal { goal_key: String },
    #[error("missing known-graph pressure for ready goal identity blake3:{goal_key}")]
    MissingPressure { goal_key: String },
    #[error("invalid known-graph pressure for ready goal identity blake3:{goal_key}: {reason}")]
    InvalidPressure { goal_key: String, reason: &'static str },
    #[error(
        "ready goal identity blake3:{goal_key} has future ready epoch {ready_since_epoch} relative to scheduling epoch {scheduling_epoch}"
    )]
    EpochRegression {
        goal_key: String,
        ready_since_epoch: u32,
        scheduling_epoch: u32,
    },
    #[error("scheduling epoch {actual} exceeds maximum {max}")]
    EpochLimitExceeded { actual: u32, max: u32 },
    #[error("priority evidence requires at least one ready candidate")]
    NoReadyCandidate,
    #[error("invalid ranked priority snapshot: {reason}")]
    InvalidRankedSnapshot { reason: String },
    #[error("invalid priority history evidence: {reason}")]
    InvalidHistoryEvidence { reason: &'static str },
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;

    const OPERATOR_CLASS_COUNT: u8 = 3;
    const STARVATION_CLASS_COUNT: u8 = 3;
    const RESOURCE_CLASS_COUNT: u8 = 4;
    const LOCALITY_CLASS_COUNT: u8 = 4;
    const TRANSFER_CLASS_COUNT: u8 = 5;
    const CLASS_INDEX_PARTIAL_OR_COMPATIBLE: u8 = 2;
    const CLASS_INDEX_SMALL_TRANSFER: u8 = 3;

    fn symbolic_tuple(stable_goal_key: String) -> PriorityTuple {
        PriorityTuple {
            operator_policy_class: symbolic_operator_class(kani::any()),
            starvation_class: symbolic_starvation_class(kani::any()),
            age_epochs: kani::any(),
            known_graph_pressure: KnownGraphPressure {
                known_critical_path_nodes: kani::any(),
                known_critical_path_work_units: kani::any(),
                blocked_root_count: kani::any(),
                blocked_root_count_saturated: kani::any(),
            },
            resource_fit: symbolic_resource_class(kani::any()),
            content_locality: symbolic_locality_class(kani::any()),
            transfer_cost: symbolic_transfer_class(kani::any()),
            stable_goal_key,
        }
    }

    fn symbolic_operator_class(value: u8) -> OperatorPolicyClass {
        match value % OPERATOR_CLASS_COUNT {
            0 => OperatorPolicyClass::Background,
            1 => OperatorPolicyClass::Ordinary,
            _ => OperatorPolicyClass::Urgent,
        }
    }

    fn symbolic_starvation_class(value: u8) -> StarvationClass {
        match value % STARVATION_CLASS_COUNT {
            0 => StarvationClass::Fresh,
            1 => StarvationClass::Aged,
            _ => StarvationClass::Protected,
        }
    }

    fn symbolic_resource_class(value: u8) -> ResourceFitClass {
        match value % RESOURCE_CLASS_COUNT {
            0 => ResourceFitClass::Unknown,
            1 => ResourceFitClass::Constrained,
            CLASS_INDEX_PARTIAL_OR_COMPATIBLE => ResourceFitClass::Compatible,
            _ => ResourceFitClass::Exact,
        }
    }

    fn symbolic_locality_class(value: u8) -> ContentLocalityClass {
        match value % LOCALITY_CLASS_COUNT {
            0 => ContentLocalityClass::Unknown,
            1 => ContentLocalityClass::NoVerifiedContent,
            CLASS_INDEX_PARTIAL_OR_COMPATIBLE => ContentLocalityClass::PartiallyPresent,
            _ => ContentLocalityClass::FullyPresent,
        }
    }

    fn symbolic_transfer_class(value: u8) -> TransferCostClass {
        match value % TRANSFER_CLASS_COUNT {
            0 => TransferCostClass::Unknown,
            1 => TransferCostClass::Large,
            CLASS_INDEX_PARTIAL_OR_COMPATIBLE => TransferCostClass::Medium,
            CLASS_INDEX_SMALL_TRANSFER => TransferCostClass::Small,
            _ => TransferCostClass::None,
        }
    }

    #[kani::proof]
    fn comparator_is_antisymmetric() {
        let policy = SchedulingPolicy::default();
        let left = symbolic_tuple("a".to_string());
        let right = symbolic_tuple("b".to_string());
        let forward = compare_priority_tuples(&policy, &left, &right);
        let reverse = compare_priority_tuples(&policy, &right, &left);
        assert_eq!(forward, reverse.reverse());
    }

    #[kani::proof]
    fn comparator_is_transitive() {
        let policy = SchedulingPolicy::default();
        let left = symbolic_tuple("a".to_string());
        let middle = symbolic_tuple("b".to_string());
        let right = symbolic_tuple("c".to_string());
        let left_before_middle = compare_priority_tuples(&policy, &left, &middle) != Ordering::Greater;
        let middle_before_right = compare_priority_tuples(&policy, &middle, &right) != Ordering::Greater;
        if left_before_middle && middle_before_right {
            assert_ne!(compare_priority_tuples(&policy, &left, &right), Ordering::Greater);
        }
    }

    #[kani::proof]
    fn epoch_advance_never_wraps() {
        let epoch: u32 = kani::any();
        kani::assume(epoch < MAX_SCHEDULING_EPOCH);
        let next = advance_scheduling_epoch(epoch).expect("bounded epoch advances");
        assert!(next > epoch);
        assert!(next <= MAX_SCHEDULING_EPOCH);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_EPOCH_FRESH: u32 = 1;
    const TEST_EPOCH_AGED: u32 = DEFAULT_AGED_AFTER_EPOCHS + TEST_EPOCH_FRESH;
    const TEST_EPOCH_PROTECTED: u32 = DEFAULT_PROTECTED_AFTER_EPOCHS + TEST_EPOCH_FRESH;
    const TEST_LONG_PATH_NODES: u32 = 4;
    const TEST_MEDIUM_PATH_NODES: u32 = 2;
    const TEST_ROOT_PRESSURE_HIGH: u32 = 3;
    const TEST_ROOT_PRESSURE_LOW: u32 = 1;
    const TEST_CHAIN_MID_PATH_NODES: u32 = 2;
    const TEST_CHAIN_LEAF_PATH_NODES: u32 = 3;
    const TEST_CHAIN_STRUCTURAL_WORK_UNITS: u32 = 2;
    const TEST_CHAIN_TOTAL_STRUCTURAL_WORK_UNITS: u32 = 3;
    const TEST_SHARED_BLOCKED_ROOTS: u32 = 2;
    const TEST_DISJOINT_AFFECTED_GOALS: u32 = 2;
    const TEST_DYNAMIC_AFFECTED_GOALS: u32 = 3;
    const TEST_HARD_BLOCKER_CASE_COUNT: usize = 6;
    const TEST_INVALID_PRESSURE_CASE_COUNT: usize = 5;

    fn node(key: &str, waitees: &[&str], waiters: &[&str], requested_root: bool) -> KnownGoalFacts {
        KnownGoalFacts {
            goal_key: key.to_string(),
            waitees: waitees.iter().map(|value| (*value).to_string()).collect(),
            waiters: waiters.iter().map(|value| (*value).to_string()).collect(),
            requested_root,
        }
    }

    fn pressure(path_nodes: u32, blocked_roots: u32) -> KnownGraphPressure {
        KnownGraphPressure {
            known_critical_path_nodes: path_nodes,
            known_critical_path_work_units: path_nodes,
            blocked_root_count: blocked_roots,
            blocked_root_count_saturated: false,
        }
    }

    fn ready(key: &str, ready_since_epoch: u32) -> ReadyGoalFacts {
        ReadyGoalFacts::ordinary(key.to_string(), ready_since_epoch)
    }

    fn ranked_tuple(
        key: &str,
        operator: OperatorPolicyClass,
        starvation: StarvationClass,
        path_nodes: u32,
        roots: u32,
        resource: ResourceFitClass,
        locality: ContentLocalityClass,
        transfer: TransferCostClass,
    ) -> RankedReadyGoal {
        RankedReadyGoal {
            goal_key: key.to_string(),
            priority: PriorityTuple {
                operator_policy_class: operator,
                starvation_class: starvation,
                age_epochs: 0,
                known_graph_pressure: pressure(path_nodes, roots),
                resource_fit: resource,
                content_locality: locality,
                transfer_cost: transfer,
                stable_goal_key: key.to_string(),
            },
        }
    }

    fn default_pressures(keys: &[&str]) -> BTreeMap<String, KnownGraphPressure> {
        keys.iter()
            .map(|key| ((*key).to_string(), pressure(TEST_ROOT_PRESSURE_LOW, TEST_ROOT_PRESSURE_LOW)))
            .collect()
    }

    fn chain_graph() -> KnownGraphFacts {
        KnownGraphFacts {
            goals: vec![
                node("root", &["mid"], &[], true),
                node("mid", &["leaf"], &["root"], false),
                node("leaf", &[], &["mid"], false),
            ],
        }
    }

    #[test]
    fn default_policy_is_valid_and_digest_stable() {
        let policy = SchedulingPolicy::default();
        let first = policy.digest_blake3().unwrap();
        let second = policy.digest_blake3().unwrap();

        assert_eq!(first, second);
        assert_eq!(first.len(), blake3::OUT_LEN * HEX_LOWER_CHARS_PER_BYTE);
        assert_eq!(u32::try_from(policy.preference_order.len()).unwrap(), PREFERENCE_FIELD_COUNT);
    }

    #[test]
    fn invalid_policy_rejects_duplicate_fields_and_bad_thresholds() {
        let mut duplicate = SchedulingPolicy::default();
        duplicate.preference_order = vec![
            PreferenceField::KnownGraph,
            PreferenceField::KnownGraph,
            PreferenceField::ResourceFit,
        ];
        let mut thresholds = SchedulingPolicy::default();
        thresholds.protected_after_epochs = thresholds.aged_after_epochs;
        let raw_private_policy_id = "/private/token=policy-secret";
        let mut private_policy = SchedulingPolicy::default();
        private_policy.policy_id = raw_private_policy_id.to_string();
        let private_policy_diagnostic = private_policy.validate().unwrap_err().to_string();
        let mut unknown_field = serde_json::to_value(SchedulingPolicy::default()).unwrap();
        unknown_field
            .as_object_mut()
            .unwrap()
            .insert("unknown_policy_field".to_string(), serde_json::Value::Bool(true));

        assert!(duplicate.validate().is_err());
        assert!(thresholds.validate().is_err());
        assert!(!private_policy_diagnostic.contains(raw_private_policy_id));
        assert!(!private_policy_diagnostic.contains("policy-secret"));
        assert!(serde_json::from_value::<SchedulingPolicy>(unknown_field).is_err());
    }

    // r[verify build_scheduling.deterministic_priority_kernel]
    #[test]
    fn comparator_totality_and_antisymmetry_hold_for_bounded_fact_matrix() {
        let policy = SchedulingPolicy::default();
        let candidates = comparator_candidates();
        for left in &candidates {
            for right in &candidates {
                let forward = compare_ranked_ready_goals(&policy, left, right);
                let reverse = compare_ranked_ready_goals(&policy, right, left);
                assert_eq!(forward, reverse.reverse());
                assert_eq!(forward == Ordering::Equal, left == right);
            }
        }
        assert!(!candidates.is_empty());
        assert!(u32::try_from(candidates.len()).is_ok_and(|count| count <= MAX_READY_GOALS));
    }

    // r[verify build_scheduling.deterministic_priority_kernel]
    #[test]
    fn comparator_transitivity_holds_for_bounded_fact_matrix() {
        let policy = SchedulingPolicy::default();
        let candidates = comparator_candidates();
        for left in &candidates {
            for middle in &candidates {
                for right in &candidates {
                    let left_before_middle = compare_ranked_ready_goals(&policy, left, middle) != Ordering::Greater;
                    let middle_before_right = compare_ranked_ready_goals(&policy, middle, right) != Ordering::Greater;
                    if left_before_middle && middle_before_right {
                        assert_ne!(compare_ranked_ready_goals(&policy, left, right), Ordering::Greater);
                    }
                }
            }
        }
        assert!(!candidates.is_empty());
        assert!(u32::try_from(candidates.len()).is_ok_and(|count| count <= MAX_READY_GOALS));
    }

    fn comparator_candidates() -> Vec<RankedReadyGoal> {
        vec![
            ranked_tuple(
                "a",
                OperatorPolicyClass::Ordinary,
                StarvationClass::Fresh,
                TEST_ROOT_PRESSURE_LOW,
                TEST_ROOT_PRESSURE_LOW,
                ResourceFitClass::Unknown,
                ContentLocalityClass::Unknown,
                TransferCostClass::Unknown,
            ),
            ranked_tuple(
                "b",
                OperatorPolicyClass::Urgent,
                StarvationClass::Fresh,
                TEST_ROOT_PRESSURE_LOW,
                TEST_ROOT_PRESSURE_LOW,
                ResourceFitClass::Compatible,
                ContentLocalityClass::PartiallyPresent,
                TransferCostClass::Small,
            ),
            ranked_tuple(
                "c",
                OperatorPolicyClass::Ordinary,
                StarvationClass::Protected,
                TEST_LONG_PATH_NODES,
                TEST_ROOT_PRESSURE_HIGH,
                ResourceFitClass::Exact,
                ContentLocalityClass::FullyPresent,
                TransferCostClass::None,
            ),
            ranked_tuple(
                "d",
                OperatorPolicyClass::Background,
                StarvationClass::Aged,
                TEST_MEDIUM_PATH_NODES,
                TEST_ROOT_PRESSURE_LOW,
                ResourceFitClass::Constrained,
                ContentLocalityClass::NoVerifiedContent,
                TransferCostClass::Large,
            ),
        ]
    }

    // r[verify build_scheduling.deterministic_priority_kernel]
    #[test]
    fn ranking_is_permutation_invariant_and_uses_stable_ties() {
        let policy = SchedulingPolicy::default();
        let pressures = default_pressures(&["a", "b", "c"]);
        let permutations = [
            vec![ready("c", 0), ready("a", 0), ready("b", 0)],
            vec![ready("b", 0), ready("c", 0), ready("a", 0)],
            vec![ready("a", 0), ready("b", 0), ready("c", 0)],
        ];
        let orders: Vec<Vec<String>> = permutations
            .iter()
            .map(|facts| {
                rank_ready_goals(&policy, TEST_EPOCH_FRESH, facts, &pressures)
                    .unwrap()
                    .into_iter()
                    .map(|ranked| ranked.goal_key)
                    .collect()
            })
            .collect();

        assert!(orders.windows(ADJACENT_PAIR_WINDOW).all(|pair| pair[0] == pair[1]));
        assert_eq!(orders[0], vec!["a", "b", "c"]);
    }

    #[test]
    fn equivalent_asynchronous_arrival_orders_replay_identically() {
        let policy = SchedulingPolicy::default();
        let pressures = BTreeMap::from([
            ("early-response".to_string(), pressure(TEST_MEDIUM_PATH_NODES, TEST_ROOT_PRESSURE_LOW)),
            ("late-response".to_string(), pressure(TEST_LONG_PATH_NODES, TEST_ROOT_PRESSURE_HIGH)),
        ]);
        let early_first = [ready("early-response", 0), ready("late-response", 0)];
        let late_first = [ready("late-response", 0), ready("early-response", 0)];

        let first = rank_ready_goals(&policy, TEST_EPOCH_FRESH, &early_first, &pressures).unwrap();
        let replay = rank_ready_goals(&policy, TEST_EPOCH_FRESH, &late_first, &pressures).unwrap();

        assert_eq!(first, replay);
        assert_eq!(first[0].goal_key, "late-response");
        assert_eq!(first[0].priority.starvation_class, StarvationClass::Fresh);
    }

    #[test]
    fn ranking_rejects_malformed_pressure_facts() {
        let cases = [
            KnownGraphPressure {
                known_critical_path_nodes: 0,
                known_critical_path_work_units: STRUCTURAL_DURATION_UNITS,
                blocked_root_count: 0,
                blocked_root_count_saturated: false,
            },
            pressure(MAX_SCHEDULING_GOALS + 1, TEST_ROOT_PRESSURE_LOW),
            KnownGraphPressure {
                known_critical_path_nodes: TEST_ROOT_PRESSURE_LOW,
                known_critical_path_work_units: MAX_KNOWN_CRITICAL_PATH_WORK_UNITS + 1,
                blocked_root_count: TEST_ROOT_PRESSURE_LOW,
                blocked_root_count_saturated: false,
            },
            pressure(TEST_ROOT_PRESSURE_LOW, MAX_BLOCKED_ROOT_PRESSURE + 1),
            KnownGraphPressure {
                known_critical_path_nodes: TEST_ROOT_PRESSURE_LOW,
                known_critical_path_work_units: TEST_ROOT_PRESSURE_LOW,
                blocked_root_count: TEST_ROOT_PRESSURE_LOW,
                blocked_root_count_saturated: true,
            },
        ];
        let ready = [ready("goal", 0)];
        for malformed in cases.iter().cloned() {
            let pressures = BTreeMap::from([("goal".to_string(), malformed)]);
            let result = rank_ready_goals(&SchedulingPolicy::default(), TEST_EPOCH_FRESH, &ready, &pressures);
            assert!(matches!(result, Err(SchedulingError::InvalidPressure { .. })));
        }
        assert_eq!(cases.len(), TEST_INVALID_PRESSURE_CASE_COUNT);
        assert_eq!(ready[0].goal_key, "goal");
    }

    #[test]
    fn graph_validation_rejects_unknown_duplicate_and_cyclic_facts() {
        let raw_private_goal = "/private/token=graph-secret";
        let unknown = KnownGraphFacts {
            goals: vec![node("root", &[raw_private_goal], &[], true)],
        };
        let duplicate = KnownGraphFacts {
            goals: vec![node("root", &[], &[], true), node("root", &[], &[], false)],
        };
        let cyclic = KnownGraphFacts {
            goals: vec![node("a", &["b"], &["b"], true), node("b", &["a"], &["a"], false)],
        };
        let policy = SchedulingPolicy::default();
        let unknown_error =
            recompute_affected_pressure(&unknown, &BTreeMap::new(), &BTreeSet::new(), &policy, &HistoryInput::Missing)
                .unwrap_err();
        let unknown_diagnostic = unknown_error.to_string();

        assert!(!unknown_diagnostic.contains(raw_private_goal));
        assert!(!unknown_diagnostic.contains("graph-secret"));
        assert!(unknown_diagnostic.contains("blake3:"));
        assert!(
            recompute_affected_pressure(
                &duplicate,
                &BTreeMap::new(),
                &BTreeSet::new(),
                &policy,
                &HistoryInput::Missing
            )
            .is_err()
        );
        assert!(matches!(
            recompute_affected_pressure(&cyclic, &BTreeMap::new(), &BTreeSet::new(), &policy, &HistoryInput::Missing),
            Err(SchedulingError::KnownGraphCycle { .. })
        ));
    }

    #[test]
    fn edge_budget_rejects_oversize_and_overflow_without_allocation() {
        let at_limit = bounded_edge_total(MAX_KNOWN_GRAPH_EDGES - 1, 1, "test edge count").unwrap();
        let oversized = bounded_edge_total(MAX_KNOWN_GRAPH_EDGES, 1, "test edge count");
        let overflow = bounded_edge_total(u32::MAX, 1, "test edge count");

        assert_eq!(at_limit, MAX_KNOWN_GRAPH_EDGES);
        assert!(matches!(oversized, Err(SchedulingError::TooManyEdges { .. })));
        assert!(matches!(overflow, Err(SchedulingError::ArithmeticOverflow { .. })));
    }

    #[test]
    fn pressure_arithmetic_overflow_rejects_without_partial_mutation() {
        let mut target = PressureAccumulator::default();
        let node_overflow = PressureAccumulator {
            path_nodes: u32::MAX,
            path_work_units: STRUCTURAL_DURATION_UNITS,
            roots: BTreeSet::new(),
            roots_saturated: false,
        };
        let work_overflow = PressureAccumulator {
            path_nodes: STRUCTURAL_DURATION_UNITS,
            path_work_units: u32::MAX,
            roots: BTreeSet::new(),
            roots_saturated: false,
        };

        let node_result = merge_pressure(&mut target, &node_overflow, STRUCTURAL_DURATION_UNITS);
        let work_result = merge_pressure(&mut target, &work_overflow, STRUCTURAL_DURATION_UNITS);

        assert!(matches!(node_result, Err(SchedulingError::ArithmeticOverflow { .. })));
        assert!(matches!(work_result, Err(SchedulingError::ArithmeticOverflow { .. })));
        assert_eq!(target, PressureAccumulator::default());
    }

    #[test]
    fn malformed_previous_pressure_cache_is_not_reused() {
        let previous = BTreeMap::from([("root".to_string(), KnownGraphPressure {
            known_critical_path_nodes: 0,
            known_critical_path_work_units: STRUCTURAL_DURATION_UNITS,
            blocked_root_count: 0,
            blocked_root_count_saturated: false,
        })]);

        let result = recompute_affected_pressure(
            &chain_graph(),
            &previous,
            &BTreeSet::new(),
            &SchedulingPolicy::default(),
            &HistoryInput::Missing,
        );

        assert!(matches!(result, Err(SchedulingError::InvalidPressure { .. })));
        assert_eq!(previous.len(), 1);
        assert_eq!(previous["root"].known_critical_path_work_units, STRUCTURAL_DURATION_UNITS);
    }

    #[test]
    fn oversized_graph_rejects_before_pressure_output() {
        let oversized_count = MAX_SCHEDULING_GOALS.checked_add(1).unwrap();
        let goals = (0..oversized_count).map(|index| node(&format!("goal-{index}"), &[], &[], false)).collect();
        let graph = KnownGraphFacts { goals };

        let error = recompute_affected_pressure(
            &graph,
            &BTreeMap::new(),
            &BTreeSet::new(),
            &SchedulingPolicy::default(),
            &HistoryInput::Missing,
        )
        .unwrap_err();
        assert!(matches!(error, SchedulingError::TooManyGoals { .. }));
        assert_eq!(oversized_count, MAX_SCHEDULING_GOALS + 1);
    }

    // r[verify build_scheduling.lazy_known_critical_path]
    #[test]
    fn chain_pressure_increases_toward_the_ready_leaf() {
        let update = recompute_affected_pressure(
            &chain_graph(),
            &BTreeMap::new(),
            &BTreeSet::new(),
            &SchedulingPolicy::default(),
            &HistoryInput::Missing,
        )
        .unwrap();

        assert_eq!(update.pressures["root"].known_critical_path_nodes, 1);
        assert_eq!(update.pressures["mid"].known_critical_path_nodes, TEST_CHAIN_MID_PATH_NODES);
        assert_eq!(update.pressures["leaf"].known_critical_path_nodes, TEST_CHAIN_LEAF_PATH_NODES);
        assert_eq!(update.history_basis, HistoryBasis::StructuralFallbackMissing);
    }

    // r[verify build_scheduling.lazy_known_critical_path]
    #[test]
    fn shared_dependency_counts_unique_blocked_roots_and_diamond_paths() {
        let graph = KnownGraphFacts {
            goals: vec![
                node("root-a", &["left"], &[], true),
                node("root-b", &["right"], &[], true),
                node("left", &["shared"], &["root-a"], false),
                node("right", &["shared"], &["root-b"], false),
                node("shared", &[], &["left", "right"], false),
            ],
        };
        let update = recompute_affected_pressure(
            &graph,
            &BTreeMap::new(),
            &BTreeSet::new(),
            &SchedulingPolicy::default(),
            &HistoryInput::Missing,
        )
        .unwrap();
        let shared = &update.pressures["shared"];

        assert_eq!(shared.blocked_root_count, TEST_SHARED_BLOCKED_ROOTS);
        assert_eq!(shared.known_critical_path_nodes, TEST_CHAIN_LEAF_PATH_NODES);
        assert!(!shared.blocked_root_count_saturated);
    }

    // r[verify build_scheduling.lazy_known_critical_path]
    #[test]
    fn affected_recompute_leaves_disjoint_component_unchanged() {
        let initial = KnownGraphFacts {
            goals: vec![
                node("root-a", &["leaf-a"], &[], true),
                node("leaf-a", &[], &["root-a"], false),
                node("root-b", &["leaf-b"], &[], true),
                node("leaf-b", &[], &["root-b"], false),
            ],
        };
        let first = recompute_affected_pressure(
            &initial,
            &BTreeMap::new(),
            &BTreeSet::new(),
            &SchedulingPolicy::default(),
            &HistoryInput::Missing,
        )
        .unwrap();
        let changed = BTreeSet::from(["root-a".to_string()]);
        let second = recompute_affected_pressure(
            &initial,
            &first.pressures,
            &changed,
            &SchedulingPolicy::default(),
            &HistoryInput::Missing,
        )
        .unwrap();

        assert_eq!(second.affected_goal_count, TEST_DISJOINT_AFFECTED_GOALS);
        assert_eq!(second.pressures["leaf-b"], first.pressures["leaf-b"]);
        assert_eq!(second.pressures.len(), first.pressures.len());
    }

    // r[verify build_scheduling.lazy_known_critical_path]
    #[test]
    fn dynamic_graph_extension_recomputes_only_connected_known_component() {
        let initial = KnownGraphFacts {
            goals: vec![
                node("root-a", &["leaf-a"], &[], true),
                node("leaf-a", &[], &["root-a"], false),
                node("root-b", &["leaf-b"], &[], true),
                node("leaf-b", &[], &["root-b"], false),
            ],
        };
        let first = recompute_affected_pressure(
            &initial,
            &BTreeMap::new(),
            &BTreeSet::new(),
            &SchedulingPolicy::default(),
            &HistoryInput::Missing,
        )
        .unwrap();
        let extended = KnownGraphFacts {
            goals: vec![
                node("dynamic-root", &["leaf-a"], &[], true),
                node("root-a", &["leaf-a"], &[], true),
                node("leaf-a", &[], &["dynamic-root", "root-a"], false),
                node("root-b", &["leaf-b"], &[], true),
                node("leaf-b", &[], &["root-b"], false),
            ],
        };
        let changed = BTreeSet::from(["dynamic-root".to_string(), "leaf-a".to_string()]);
        let second = recompute_affected_pressure(
            &extended,
            &first.pressures,
            &changed,
            &SchedulingPolicy::default(),
            &HistoryInput::Missing,
        )
        .unwrap();

        assert_eq!(second.affected_goal_count, TEST_DYNAMIC_AFFECTED_GOALS);
        assert_eq!(second.pressures["leaf-a"].blocked_root_count, TEST_SHARED_BLOCKED_ROOTS);
        assert_eq!(second.pressures["leaf-b"], first.pressures["leaf-b"]);
    }

    #[test]
    fn history_fresh_changes_work_basis_while_missing_stale_and_incompatible_fall_back() {
        let graph = chain_graph();
        let policy = SchedulingPolicy::default();
        let fresh_snapshot = DurationHistorySnapshot {
            schema: SCHEDULING_HISTORY_SCHEMA.to_string(),
            policy_id: policy.policy_id.clone(),
            duration_classes: BTreeMap::from([("mid".to_string(), DurationClass::Long)]),
        };
        let incompatible_snapshot = DurationHistorySnapshot {
            policy_id: "other-policy".to_string(),
            ..fresh_snapshot.clone()
        };
        let fresh = recompute_affected_pressure(
            &graph,
            &BTreeMap::new(),
            &BTreeSet::new(),
            &policy,
            &HistoryInput::Fresh(fresh_snapshot.clone()),
        )
        .unwrap();
        let stale = recompute_affected_pressure(
            &graph,
            &BTreeMap::new(),
            &BTreeSet::new(),
            &policy,
            &HistoryInput::Stale(fresh_snapshot),
        )
        .unwrap();
        let incompatible = recompute_affected_pressure(
            &graph,
            &BTreeMap::new(),
            &BTreeSet::new(),
            &policy,
            &HistoryInput::Fresh(incompatible_snapshot),
        )
        .unwrap();

        assert_eq!(fresh.history_basis, HistoryBasis::FreshSnapshot);
        assert_eq!(
            fresh.pressures["leaf"].known_critical_path_work_units,
            LONG_DURATION_UNITS + TEST_CHAIN_STRUCTURAL_WORK_UNITS
        );
        assert_eq!(stale.history_basis, HistoryBasis::StructuralFallbackStale);
        assert_eq!(incompatible.history_basis, HistoryBasis::StructuralFallbackIncompatible);
    }

    #[test]
    fn oversized_and_unknown_history_snapshots_fall_back_without_partial_admission() {
        let graph = chain_graph();
        let policy = SchedulingPolicy::default();
        let oversized_entry_count = MAX_HISTORY_ENTRIES.checked_add(1).unwrap();
        let oversized_classes =
            (0..oversized_entry_count).map(|index| (format!("history-{index}"), DurationClass::Long)).collect();
        let oversized = DurationHistorySnapshot {
            schema: SCHEDULING_HISTORY_SCHEMA.to_string(),
            policy_id: policy.policy_id.clone(),
            duration_classes: oversized_classes,
        };
        let unknown = DurationHistorySnapshot {
            schema: SCHEDULING_HISTORY_SCHEMA.to_string(),
            policy_id: policy.policy_id.clone(),
            duration_classes: BTreeMap::from([("unknown-goal".to_string(), DurationClass::Long)]),
        };
        let malformed = DurationHistorySnapshot {
            schema: SCHEDULING_HISTORY_SCHEMA.to_string(),
            policy_id: policy.policy_id.clone(),
            duration_classes: BTreeMap::from([("malformed\nkey".to_string(), DurationClass::Long)]),
        };
        let oversized_result = recompute_affected_pressure(
            &graph,
            &BTreeMap::new(),
            &BTreeSet::new(),
            &policy,
            &HistoryInput::Fresh(oversized),
        )
        .unwrap();
        let unknown_result = recompute_affected_pressure(
            &graph,
            &BTreeMap::new(),
            &BTreeSet::new(),
            &policy,
            &HistoryInput::Fresh(unknown),
        )
        .unwrap();
        let malformed_result = recompute_affected_pressure(
            &graph,
            &BTreeMap::new(),
            &BTreeSet::new(),
            &policy,
            &HistoryInput::Fresh(malformed),
        )
        .unwrap();

        assert_eq!(oversized_result.history_basis, HistoryBasis::StructuralFallbackOversized);
        assert_eq!(unknown_result.history_basis, HistoryBasis::StructuralFallbackUnknownGoal);
        assert_eq!(malformed_result.history_basis, HistoryBasis::StructuralFallbackIncompatible);
        assert!(malformed_result.history_snapshot_digest_blake3.is_none());
        assert_eq!(
            oversized_result.pressures["leaf"].known_critical_path_work_units,
            TEST_CHAIN_TOTAL_STRUCTURAL_WORK_UNITS
        );
    }

    #[test]
    fn configured_preference_order_changes_named_field_precedence() {
        let mut policy = SchedulingPolicy::default();
        policy.preference_order = vec![
            PreferenceField::ResourceFit,
            PreferenceField::KnownGraph,
            PreferenceField::LocalityTransfer,
        ];
        let pressures = BTreeMap::from([
            ("resource".to_string(), pressure(TEST_ROOT_PRESSURE_LOW, TEST_ROOT_PRESSURE_LOW)),
            ("path".to_string(), pressure(TEST_LONG_PATH_NODES, TEST_ROOT_PRESSURE_HIGH)),
        ]);
        let mut resource = ready("resource", 0);
        resource.preference.resource_fit = ResourceFitClass::Exact;
        let ranked = rank_ready_goals(&policy, TEST_EPOCH_FRESH, &[ready("path", 0), resource], &pressures).unwrap();

        assert_eq!(ranked[0].goal_key, "resource");
        assert_eq!(policy.preference_order[0], PreferenceField::ResourceFit);
    }

    // r[verify build_scheduling.resource_locality_preference]
    #[test]
    fn eligible_resource_and_locality_facts_improve_preference() {
        let policy = SchedulingPolicy::default();
        let pressures = default_pressures(&["local", "remote"]);
        let mut local = ready("local", 0);
        local.preference = normalize_eligible_preference(
            HardEligibilityFacts::ELIGIBLE,
            ResourceFitClass::Exact,
            ContentLocalityClass::FullyPresent,
            TransferCostClass::None,
        )
        .unwrap();
        let remote = ready("remote", 0);
        let ranked = rank_ready_goals(&policy, TEST_EPOCH_FRESH, &[remote, local], &pressures).unwrap();

        assert_eq!(ranked[0].goal_key, "local");
        assert_eq!(ranked[0].priority.resource_fit, ResourceFitClass::Exact);
    }

    // r[verify build_scheduling.resource_locality_preference]
    #[test]
    fn every_hard_blocker_remains_ineligible_regardless_of_preference() {
        let preferred = (ResourceFitClass::Exact, ContentLocalityClass::FullyPresent, TransferCostClass::None);
        let cases = [
            (
                HardEligibilityFacts {
                    capability_allowed: false,
                    ..HardEligibilityFacts::ELIGIBLE
                },
                IneligibleReason::MissingCapability,
            ),
            (
                HardEligibilityFacts {
                    output_trust_allowed: false,
                    ..HardEligibilityFacts::ELIGIBLE
                },
                IneligibleReason::OutputTrustRejected,
            ),
            (
                HardEligibilityFacts {
                    upload_allowed: false,
                    ..HardEligibilityFacts::ELIGIBLE
                },
                IneligibleReason::UploadPolicyRejected,
            ),
            (
                HardEligibilityFacts {
                    network_allowed: false,
                    ..HardEligibilityFacts::ELIGIBLE
                },
                IneligibleReason::NetworkPolicyRejected,
            ),
            (
                HardEligibilityFacts {
                    store_prefix_matches: false,
                    ..HardEligibilityFacts::ELIGIBLE
                },
                IneligibleReason::StorePrefixMismatch,
            ),
            (
                HardEligibilityFacts {
                    hard_resource_fit: false,
                    ..HardEligibilityFacts::ELIGIBLE
                },
                IneligibleReason::HardResourceMismatch,
            ),
        ];

        for (eligibility, expected) in cases {
            assert_eq!(
                normalize_eligible_preference(eligibility, preferred.0, preferred.1, preferred.2),
                Err(expected)
            );
        }
        assert_eq!(cases.len(), TEST_HARD_BLOCKER_CASE_COUNT);
    }

    // r[verify build_scheduling.deterministic_starvation_bound]
    #[test]
    fn continuously_ready_goal_becomes_protected_and_outranks_ordinary_pressure() {
        let policy = SchedulingPolicy::default();
        let pressures = BTreeMap::from([
            ("old".to_string(), pressure(TEST_ROOT_PRESSURE_LOW, TEST_ROOT_PRESSURE_LOW)),
            ("new".to_string(), pressure(TEST_LONG_PATH_NODES, TEST_ROOT_PRESSURE_HIGH)),
        ]);
        let ranked = rank_ready_goals(
            &policy,
            TEST_EPOCH_PROTECTED,
            &[ready("new", TEST_EPOCH_PROTECTED), ready("old", 0)],
            &pressures,
        )
        .unwrap();

        assert_eq!(ranked[0].goal_key, "old");
        assert_eq!(ranked[0].priority.starvation_class, StarvationClass::Protected);
        assert_eq!(ranked[1].priority.starvation_class, StarvationClass::Fresh);
    }

    // r[verify build_scheduling.deterministic_starvation_bound]
    #[test]
    fn recurring_higher_pressure_arrivals_lose_at_the_aged_epoch() {
        let policy = SchedulingPolicy::default();
        let old_pressure = pressure(TEST_ROOT_PRESSURE_LOW, TEST_ROOT_PRESSURE_LOW);
        let new_pressure = pressure(TEST_LONG_PATH_NODES, TEST_ROOT_PRESSURE_HIGH);
        let mut promotion_epoch = None;
        for epoch in TEST_EPOCH_FRESH..=policy.protected_after_epochs {
            let newcomer_key = format!("new-{epoch}");
            let pressures = BTreeMap::from([
                ("old".to_string(), old_pressure.clone()),
                (newcomer_key.clone(), new_pressure.clone()),
            ]);
            let ranked =
                rank_ready_goals(&policy, epoch, &[ready(&newcomer_key, epoch), ready("old", 0)], &pressures).unwrap();
            if ranked[0].goal_key == "old" {
                promotion_epoch = Some(epoch);
                break;
            }
        }

        assert_eq!(promotion_epoch, Some(policy.aged_after_epochs));
        assert!(policy.aged_after_epochs < policy.protected_after_epochs);
    }

    #[test]
    fn ineligible_goal_does_not_age_into_dispatch() {
        let blocked = HardEligibilityFacts {
            capability_allowed: false,
            ..HardEligibilityFacts::ELIGIBLE
        };
        let result = normalize_eligible_preference(
            blocked,
            ResourceFitClass::Exact,
            ContentLocalityClass::FullyPresent,
            TransferCostClass::None,
        );

        assert_eq!(result, Err(IneligibleReason::MissingCapability));
        assert_eq!(IneligibleReason::MissingCapability.as_str(), "missing-capability");
    }

    #[test]
    fn age_classes_are_bounded_and_future_ready_epoch_is_rejected() {
        let policy = SchedulingPolicy::default();
        let pressures = default_pressures(&["goal"]);
        let aged = rank_ready_goals(&policy, TEST_EPOCH_AGED, &[ready("goal", 0)], &pressures).unwrap();
        let future = rank_ready_goals(&policy, TEST_EPOCH_FRESH, &[ready("goal", TEST_EPOCH_AGED)], &pressures);

        assert_eq!(aged[0].priority.starvation_class, StarvationClass::Aged);
        assert!(matches!(future, Err(SchedulingError::EpochRegression { .. })));
    }

    // r[verify build_scheduling.priority_decision_evidence]
    #[test]
    fn evidence_is_replay_stable_and_redacts_raw_goal_and_sensitive_route_text() {
        let policy = SchedulingPolicy::default();
        let raw_goal = "/private/token=secret/root.drv";
        let ranked = vec![ranked_tuple(
            raw_goal,
            OperatorPolicyClass::Ordinary,
            StarvationClass::Fresh,
            TEST_MEDIUM_PATH_NODES,
            TEST_ROOT_PRESSURE_LOW,
            ResourceFitClass::Exact,
            ContentLocalityClass::FullyPresent,
            TransferCostClass::None,
        )];
        let first = priority_decision_evidence(
            &policy,
            TEST_EPOCH_FRESH,
            &ranked,
            HistoryBasis::StructuralFallbackMissing,
            None,
        )
        .unwrap();
        let second = priority_decision_evidence(
            &policy,
            TEST_EPOCH_FRESH,
            &ranked,
            HistoryBasis::StructuralFallbackMissing,
            None,
        )
        .unwrap();
        let json = serde_json::to_string(&first).unwrap();

        assert_eq!(first, second);
        assert!(!json.contains(raw_goal));
        assert!(!json.contains("token=secret"));
        assert_eq!(first.claim_scope, SCHEDULING_CLAIM_SCOPE);
        assert_eq!(first.selection_reason, PrioritySelectionReason::SoleCandidate);
        assert!(first.runner_up.is_none());
        assert_eq!(first.candidate_snapshot_digest_blake3.len(), blake3::OUT_LEN * HEX_LOWER_CHARS_PER_BYTE);
    }

    #[test]
    fn evidence_rejects_internally_inconsistent_ranked_snapshots() {
        let policy = SchedulingPolicy::default();
        let pressures = default_pressures(&["goal"]);
        let ranked = rank_ready_goals(&policy, TEST_EPOCH_FRESH, &[ready("goal", 0)], &pressures).unwrap();
        let mut mismatched_identity = ranked.clone();
        mismatched_identity[0].priority.stable_goal_key = "other".to_string();
        let mut future_age = ranked.clone();
        future_age[0].priority.age_epochs = TEST_EPOCH_FRESH.checked_add(1).unwrap();
        let mut wrong_starvation = ranked.clone();
        wrong_starvation[0].priority.starvation_class = StarvationClass::Protected;
        let mut invalid_pressure = ranked.clone();
        invalid_pressure[0].priority.known_graph_pressure.blocked_root_count = MAX_BLOCKED_ROOT_PRESSURE + 1;

        for invalid in [mismatched_identity, future_age, wrong_starvation] {
            let result = priority_decision_evidence(
                &policy,
                TEST_EPOCH_FRESH,
                &invalid,
                HistoryBasis::StructuralFallbackMissing,
                None,
            );
            assert!(matches!(result, Err(SchedulingError::InvalidRankedSnapshot { .. })));
        }
        let pressure_result = priority_decision_evidence(
            &policy,
            TEST_EPOCH_FRESH,
            &invalid_pressure,
            HistoryBasis::StructuralFallbackMissing,
            None,
        );
        assert!(matches!(pressure_result, Err(SchedulingError::InvalidPressure { .. })));
        let leaked_history = priority_decision_evidence(
            &policy,
            TEST_EPOCH_FRESH,
            &ranked,
            HistoryBasis::StructuralFallbackIncompatible,
            Some("/private/history-secret".to_string()),
        );
        let missing_fresh_digest =
            priority_decision_evidence(&policy, TEST_EPOCH_FRESH, &ranked, HistoryBasis::FreshSnapshot, None);
        assert!(matches!(leaked_history, Err(SchedulingError::InvalidHistoryEvidence { .. })));
        assert!(matches!(missing_fresh_digest, Err(SchedulingError::InvalidHistoryEvidence { .. })));
        assert_eq!(invalid_pressure[0].goal_key, "goal");
    }

    // r[verify build_scheduling.deterministic_priority_kernel]
    #[test]
    fn priority_fixture_improves_first_dispatch_pressure_relative_to_fifo() {
        let policy = SchedulingPolicy::default();
        let fifo = vec!["short".to_string(), "shared".to_string()];
        let pressures = BTreeMap::from([
            ("short".to_string(), pressure(TEST_ROOT_PRESSURE_LOW, TEST_ROOT_PRESSURE_LOW)),
            ("shared".to_string(), pressure(TEST_LONG_PATH_NODES, TEST_ROOT_PRESSURE_HIGH)),
        ]);
        let ranked =
            rank_ready_goals(&policy, TEST_EPOCH_FRESH, &[ready("short", 0), ready("shared", 0)], &pressures).unwrap();
        let decision = priority_decision_evidence(
            &policy,
            TEST_EPOCH_FRESH,
            &ranked,
            HistoryBasis::StructuralFallbackMissing,
            None,
        )
        .unwrap();
        let mut unsorted = ranked.clone();
        unsorted.reverse();
        let unsorted_result = priority_decision_evidence(
            &policy,
            TEST_EPOCH_FRESH,
            &unsorted,
            HistoryBasis::StructuralFallbackMissing,
            None,
        );
        let priority_order: Vec<String> = ranked.iter().map(|item| item.goal_key.clone()).collect();
        let fifo_first_pressure = pressures[&fifo[0]].blocked_root_count;
        let priority_first_pressure = pressures[&priority_order[0]].blocked_root_count;

        assert_eq!(fifo[0], "short");
        assert_eq!(priority_order[0], "shared");
        assert!(priority_first_pressure > fifo_first_pressure);
        assert_eq!(decision.competing_goal_count, u32::try_from(ADJACENT_PAIR_WINDOW).unwrap());
        assert!(decision.runner_up.is_some());
        assert_eq!(decision.selection_reason, PrioritySelectionReason::KnownGraphPressure);
        assert!(matches!(unsorted_result, Err(SchedulingError::InvalidRankedSnapshot { .. })));
    }

    #[test]
    fn epoch_advance_is_checked_and_bounded() {
        assert_eq!(advance_scheduling_epoch(0).unwrap(), 1);
        assert!(matches!(
            advance_scheduling_epoch(MAX_SCHEDULING_EPOCH),
            Err(SchedulingError::EpochLimitExceeded { .. })
        ));
    }
}
