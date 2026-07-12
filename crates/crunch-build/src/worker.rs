//! Worker: imperative shell that drives Goal state machines.
//!
//! The Worker owns the `GoalRegistry` and orchestrates builds by:
//! 1. Creating goals lazily via `want()` — deduplicates by drv path
//! 2. Inspecting deps from `DerivationRegistry` to wire waiters
//! 3. Dispatching Ready goals through `Builder::prepare_build()`
//! 4. Spawning sandbox builds on a `JoinSet` with `Semaphore` concurrency
//! 5. Completing builds via `Builder::finish_build()`, notifying waiters
//! 6. Repeating until all root goals are terminal
//!
//! The Worker borrows `&mut Builder` for I/O — it never does I/O itself.
//! Goal state transitions are validated by the pure `Goal` API.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;

use bstr::BString;
use nix_compat::derivation::Derivation;
use nix_compat::derivation::Output;
use nix_compat::nixhash::CAHash;
use nix_compat::nixhash::HashAlgo;
use nix_compat::nixhash::NixHash;
use nix_compat::store_path::StorePath;
use snix_build::buildservice::BuildService;
use tokio::sync::Semaphore;
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tracing::debug;
use tracing::info;

use crate::Error;
use crate::dynamic_plan::AddressingMode;
use crate::dynamic_plan::CanonicalDynamicPlanV1;
use crate::dynamic_plan::DeclaredSourceInput;
use crate::dynamic_plan::DynamicInput;
use crate::dynamic_plan::DynamicUnit;
use crate::dynamic_plan::FixedOutputHashAlgo;
use crate::dynamic_plan::FixedOutputMode;
use crate::dynamic_plan::FixedOutputSpec;
use crate::dynamic_plan::MAX_DYNAMIC_PLAN_BYTES;
use crate::dynamic_plan::decode_validated_plan_v1;
use crate::goal::Goal;
use crate::goal::GoalRegistry;
use crate::goal::GoalState;
use crate::goal::MAX_GOALS;
use crate::orchestrate::BuildOutcome;
use crate::orchestrate::Builder;
use crate::orchestrate::PrepareResult;
use crate::orchestrate::PreparedBuild;
use crate::registry::DerivationRegistry;
use crate::scheduling::EligiblePreferenceFacts;
use crate::scheduling::HistoryBasis;
use crate::scheduling::HistoryInput;
use crate::scheduling::KnownGoalFacts;
use crate::scheduling::KnownGraphFacts;
use crate::scheduling::KnownGraphPressure;
use crate::scheduling::MAX_KNOWN_GRAPH_EDGES;
use crate::scheduling::MAX_PRIORITY_DECISIONS;
use crate::scheduling::OperatorPolicyClass;
use crate::scheduling::PriorityDecisionEvidence;
use crate::scheduling::ReadyGoalFacts;
use crate::scheduling::SchedulingPolicy;
use crate::scheduling::advance_scheduling_epoch;
use crate::scheduling::priority_decision_evidence;
use crate::scheduling::rank_ready_goals;
use crate::scheduling::recompute_affected_pressure;

type PendingRegistryEntry = (
    StorePath<String>,
    [u8; 32],
    nix_compat::derivation::Derivation,
    bool,
    Vec<String>,
    Option<crunch_attestation::Claims>,
);
type CreatedGoal = (String, Vec<StorePath<String>>);

/// A derivation arriving from the eval thread.
///
/// Contains the root drv_path plus all newly-converted entries
/// (the root and its transitive deps). The Worker inserts these
/// into `DerivationRegistry` before calling `want()`, enabling
/// true eval/build overlap: leaf deps start building while later
/// roots are still being converted.
#[derive(Debug, Clone)]
pub struct EvalMessage {
    /// Human-readable label (e.g., package name).
    pub label: String,
    /// The root derivation's store path.
    pub drv_path: StorePath<String>,
    /// Derivation entries discovered during this root's conversion.
    /// Each tuple: (drv_path, hash_derivation_modulo, Derivation, is_ca, dynamic_plan_outputs,
    /// provenance_claims). Inserted into `DerivationRegistry` before `want()` so deps
    /// are known. Diamond deps already in the registry are skipped
    /// (insert is idempotent by drv path).
    pub new_entries: Vec<PendingRegistryEntry>,
}

/// Maximum supported concurrent in-flight build budget. The constructor,
/// dispatch shell, and semaphore all enforce this bound.
const MAX_IN_FLIGHT: u32 = 64;
const BLAKE3_HEX_CHARS_PER_BYTE: usize = 2;

/// A root goal that failed, with error context.
#[derive(Debug, Clone)]
pub struct FailedGoal {
    /// Absolute drv-path key.
    pub drv_key: String,
    /// Human-readable error message.
    pub error: String,
}

/// Result of running the Worker: outcomes for root goals.
#[derive(Debug)]
pub struct WorkerResult {
    /// Build outcomes for root derivations that succeeded.
    pub outcomes: Vec<BuildOutcome>,
    /// Root goals that failed (build error or dep failure).
    pub failed: Vec<FailedGoal>,
    /// Native dynamic-plan accepted/rejected rows discovered during the run.
    pub native_dynamic_plans: Vec<NativeDynamicPlanReport>,
    /// Bounded, redacted evidence for each selected ready goal.
    pub priority_decisions: Vec<PriorityDecisionEvidence>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeDynamicPlanReport {
    pub mode: String,
    pub producer_key: String,
    pub output_name: String,
    pub plan_artifact_path: Option<StorePath<String>>,
    pub raw_artifact_digest: Option<String>,
    pub canonical_plan_digest: Option<String>,
    pub accepted_unit_ids: Vec<String>,
    pub rejection_reason: Option<String>,
    pub scheduler_action: String,
}

/// Native dynamic plan accepted from a declared producer output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeDynamicPlanAccepted {
    pub producer_key: String,
    pub output_name: String,
    pub plan_artifact_path: StorePath<String>,
    pub raw_artifact_digest: String,
    pub canonical_plan_digest: String,
    pub accepted_unit_ids: Vec<String>,
    pub plan: CanonicalDynamicPlanV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeDynamicPlanRejectionKind {
    MissingOutput,
    NonRegularOutput,
    PlanTooLarge,
    InvalidPlan,
    ReadFailed,
}

impl NativeDynamicPlanRejectionKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MissingOutput => "missing-output",
            Self::NonRegularOutput => "non-regular-output",
            Self::PlanTooLarge => "plan-too-large",
            Self::InvalidPlan => "invalid-plan",
            Self::ReadFailed => "read-failed",
        }
    }
}

/// Native dynamic plan rejection tied to a producer output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeDynamicPlanRejection {
    pub producer_key: String,
    pub output_name: String,
    pub plan_artifact_path: Option<StorePath<String>>,
    pub raw_artifact_digest: Option<String>,
    pub kind: NativeDynamicPlanRejectionKind,
    pub detail: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NativeDynamicPlanScan {
    pub accepted: Vec<NativeDynamicPlanAccepted>,
    pub rejected: Vec<NativeDynamicPlanRejection>,
}

enum NativeDynamicPlanOutputScan {
    Accepted(NativeDynamicPlanAccepted),
    Rejected(NativeDynamicPlanRejection),
}

struct RegisteredNativeDynamicPlan {
    root_drv_paths: Vec<StorePath<String>>,
    unit_drv_paths: BTreeMap<String, StorePath<String>>,
}

impl NativeDynamicPlanScan {
    fn is_empty(&self) -> bool {
        self.accepted.is_empty() && self.rejected.is_empty()
    }
}

fn node_file_size(node: &snix_castore::Node) -> Option<u64> {
    match node {
        snix_castore::Node::File { size, .. } => Some(*size),
        snix_castore::Node::Directory { .. } | snix_castore::Node::Symlink { .. } => None,
    }
}

fn blake3_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

fn accepted_native_plan(
    producer_key: &str,
    output_name: &str,
    plan_artifact_path: StorePath<String>,
    raw_artifact_digest: String,
    plan: CanonicalDynamicPlanV1,
) -> NativeDynamicPlanAccepted {
    let accepted_unit_ids = plan.plan.units.iter().map(|unit| unit.id.clone()).collect();
    NativeDynamicPlanAccepted {
        producer_key: producer_key.to_string(),
        output_name: output_name.to_string(),
        plan_artifact_path,
        raw_artifact_digest,
        canonical_plan_digest: plan.digest.clone(),
        accepted_unit_ids,
        plan,
    }
}

fn native_plan_report_from_accepted(accepted: &NativeDynamicPlanAccepted) -> NativeDynamicPlanReport {
    NativeDynamicPlanReport {
        mode: "native".to_string(),
        producer_key: accepted.producer_key.clone(),
        output_name: accepted.output_name.clone(),
        plan_artifact_path: Some(accepted.plan_artifact_path.clone()),
        raw_artifact_digest: Some(accepted.raw_artifact_digest.clone()),
        canonical_plan_digest: Some(accepted.canonical_plan_digest.clone()),
        accepted_unit_ids: accepted.accepted_unit_ids.clone(),
        rejection_reason: None,
        scheduler_action: "registered-roots".to_string(),
    }
}

fn native_plan_report_from_rejection(rejected: &NativeDynamicPlanRejection) -> NativeDynamicPlanReport {
    NativeDynamicPlanReport {
        mode: "native".to_string(),
        producer_key: rejected.producer_key.clone(),
        output_name: rejected.output_name.clone(),
        plan_artifact_path: rejected.plan_artifact_path.clone(),
        raw_artifact_digest: rejected.raw_artifact_digest.clone(),
        canonical_plan_digest: None,
        accepted_unit_ids: Vec::new(),
        rejection_reason: Some(format!("{}: {}", rejected.kind.as_str(), rejected.detail)),
        scheduler_action: "rejected".to_string(),
    }
}

fn native_plan_reports_from_scan(scan: &NativeDynamicPlanScan) -> Vec<NativeDynamicPlanReport> {
    let mut reports = Vec::with_capacity(scan.accepted.len().saturating_add(scan.rejected.len()));
    reports.extend(scan.accepted.iter().map(native_plan_report_from_accepted));
    reports.extend(scan.rejected.iter().map(native_plan_report_from_rejection));
    sort_native_dynamic_plan_reports(&mut reports);
    reports
}

fn finish_worker_result(
    outcomes: Vec<BuildOutcome>,
    failed: Vec<FailedGoal>,
    mut native_dynamic_plans: Vec<NativeDynamicPlanReport>,
    priority_decisions: Vec<PriorityDecisionEvidence>,
) -> WorkerResult {
    sort_native_dynamic_plan_reports(&mut native_dynamic_plans);
    debug_assert!(u32::try_from(priority_decisions.len()).is_ok_and(|count| count <= MAX_PRIORITY_DECISIONS));
    debug_assert!(
        priority_decisions
            .iter()
            .all(|decision| decision.known_graph_basis == crate::scheduling::KNOWN_GRAPH_BASIS)
    );
    WorkerResult {
        outcomes,
        failed,
        native_dynamic_plans,
        priority_decisions,
    }
}

fn sort_native_dynamic_plan_reports(reports: &mut [NativeDynamicPlanReport]) {
    reports.sort_by(|left, right| {
        (&left.producer_key, &left.output_name, &left.mode, &left.scheduler_action).cmp(&(
            &right.producer_key,
            &right.output_name,
            &right.mode,
            &right.scheduler_action,
        ))
    });
}

fn native_plan_rejection(
    producer_key: &str,
    output_name: &str,
    plan_artifact_path: Option<StorePath<String>>,
    raw_artifact_digest: Option<String>,
    kind: NativeDynamicPlanRejectionKind,
    detail: String,
) -> NativeDynamicPlanRejection {
    NativeDynamicPlanRejection {
        producer_key: producer_key.to_string(),
        output_name: output_name.to_string(),
        plan_artifact_path,
        raw_artifact_digest,
        kind,
        detail,
    }
}

fn parse_dynamic_store_path(path: &str, store_dir: &str) -> Result<StorePath<String>, Error> {
    StorePath::from_absolute_path_with_prefix(path.as_bytes(), store_dir)
        .map_err(|_| Error::Store(format!("native dynamic plan store path is invalid for prefix {store_dir}: {path}")))
}

fn dynamic_sources_by_id(
    sources: &[DeclaredSourceInput],
    store_dir: &str,
) -> Result<BTreeMap<String, StorePath<String>>, Error> {
    let mut by_id = BTreeMap::new();
    for source in sources {
        let path = parse_dynamic_store_path(&source.path, store_dir)?;
        by_id.insert(source.id.clone(), path);
    }
    Ok(by_id)
}

fn parse_dynamic_fixed_output(spec: &FixedOutputSpec) -> Result<CAHash, Error> {
    let algo_name = match spec.algo {
        FixedOutputHashAlgo::Sha256 => "sha256",
        FixedOutputHashAlgo::Sha512 => "sha512",
        FixedOutputHashAlgo::Sha1 => "sha1",
        FixedOutputHashAlgo::Md5 => "md5",
        FixedOutputHashAlgo::Blake3 => "blake3",
    };
    let algo: HashAlgo = algo_name
        .parse()
        .map_err(|_| Error::Store(format!("unsupported native dynamic fixed-output hash algorithm: {algo_name}")))?;
    let hash = if spec.hash.contains('-') {
        NixHash::from_sri(&spec.hash)
            .map_err(|err| Error::Store(format!("invalid native dynamic fixed-output SRI hash: {err}")))?
    } else {
        let digest = data_encoding::HEXLOWER
            .decode(spec.hash.as_bytes())
            .map_err(|err| Error::Store(format!("invalid native dynamic fixed-output hex hash: {err}")))?;
        NixHash::from_algo_and_digest(algo, &digest)
            .map_err(|err| Error::Store(format!("invalid native dynamic fixed-output digest: {err}")))?
    };
    match spec.mode {
        FixedOutputMode::Flat => Ok(CAHash::Flat(hash)),
        FixedOutputMode::Recursive => Ok(CAHash::Nar(hash)),
    }
}

fn dynamic_plan_outputs_are_declared(unit: &DynamicUnit) -> bool {
    let outputs: BTreeSet<&str> = unit.derivation.outputs.iter().map(String::as_str).collect();
    unit.derivation.dynamic_plan_outputs.iter().all(|output| outputs.contains(output.as_str()))
}

fn unit_output_dependencies_registered(unit: &DynamicUnit, registered: &BTreeMap<String, StorePath<String>>) -> bool {
    unit.derivation.inputs.iter().all(|input| match input {
        DynamicInput::UnitOutput { unit, .. } => registered.contains_key(unit),
        DynamicInput::StorePath { .. } | DynamicInput::Source { .. } => true,
    })
}

fn register_native_dynamic_plan_units(
    accepted: &NativeDynamicPlanAccepted,
    known_paths: &mut DerivationRegistry,
) -> Result<RegisteredNativeDynamicPlan, Error> {
    let store_dir = known_paths.store_dir().to_string();
    let sources = dynamic_sources_by_id(&accepted.plan.plan.sources, &store_dir)?;
    let units_by_id: BTreeMap<String, &DynamicUnit> =
        accepted.plan.plan.units.iter().map(|unit| (unit.id.clone(), unit)).collect();
    let mut pending: BTreeSet<String> = units_by_id.keys().cloned().collect();
    let mut registered = BTreeMap::new();
    let iteration_limit = accepted.plan.plan.units.len();

    for _ in 0..iteration_limit {
        if pending.is_empty() {
            break;
        }
        let ready_units: Vec<String> = pending
            .iter()
            .filter(|unit_id| unit_output_dependencies_registered(units_by_id[unit_id.as_str()], &registered))
            .cloned()
            .collect();
        if ready_units.is_empty() {
            break;
        }
        for unit_id in ready_units {
            let unit = units_by_id[unit_id.as_str()];
            let drv_path = register_native_dynamic_unit(unit, &sources, &registered, known_paths, &store_dir)?;
            registered.insert(unit_id.clone(), drv_path);
            pending.remove(&unit_id);
        }
    }

    if !pending.is_empty() {
        return Err(Error::Store(format!(
            "native dynamic plan '{}' has unresolved unit dependencies: {}",
            accepted.canonical_plan_digest,
            pending.into_iter().collect::<Vec<_>>().join(", ")
        )));
    }

    let mut root_drv_paths = Vec::with_capacity(accepted.plan.plan.roots.len());
    for root in &accepted.plan.plan.roots {
        let drv_path = registered
            .get(root)
            .ok_or_else(|| Error::Store(format!("native dynamic plan root was not registered: {root}")))?;
        root_drv_paths.push(drv_path.clone());
    }

    Ok(RegisteredNativeDynamicPlan {
        root_drv_paths,
        unit_drv_paths: registered,
    })
}

fn register_native_dynamic_unit(
    unit: &DynamicUnit,
    sources: &BTreeMap<String, StorePath<String>>,
    registered_units: &BTreeMap<String, StorePath<String>>,
    known_paths: &mut DerivationRegistry,
    store_dir: &str,
) -> Result<StorePath<String>, Error> {
    if !dynamic_plan_outputs_are_declared(unit) {
        return Err(Error::Store(format!(
            "native dynamic unit '{}' declares dynamic_plan_outputs outside outputs",
            unit.id
        )));
    }

    let mut derivation = build_native_dynamic_derivation(unit, sources, registered_units, store_dir)?;
    for parent_drv_path in derivation.input_derivations.keys() {
        let parent_abs = parent_drv_path.to_absolute_path_with_prefix(store_dir);
        if known_paths.get_hdm_by_drv_path(&parent_abs).is_none() {
            return Err(Error::Store(format!(
                "native dynamic unit '{}' references unregistered parent derivation {parent_abs}",
                unit.id
            )));
        }
    }

    let hdm = derivation.hash_derivation_modulo(|parent_drv_path| {
        known_paths
            .get_hdm_by_drv_path(&parent_drv_path.to_absolute_path_with_prefix(store_dir))
            .unwrap_or([0u8; 32])
    });
    let is_ca = matches!(unit.derivation.addressing_mode, AddressingMode::ContentAddressed)
        && unit.derivation.fixed_output.is_none();
    derivation
        .calculate_output_paths_with_store_dir(&unit.derivation.name, &hdm, store_dir)
        .map_err(|err| Error::Store(format!("computing native dynamic output paths for '{}': {err}", unit.id)))?;
    if is_ca {
        for output in derivation.outputs.values_mut() {
            output.path = None;
        }
    }
    let drv_path = derivation
        .calculate_derivation_path_with_store_dir(&unit.derivation.name, store_dir)
        .map_err(|err| Error::Store(format!("computing native dynamic derivation path for '{}': {err}", unit.id)))?;
    known_paths.insert_with_dynamic_plan_outputs(
        drv_path.clone(),
        hdm,
        derivation,
        is_ca,
        unit.derivation.dynamic_plan_outputs.clone(),
        None,
    );
    Ok(drv_path)
}

fn build_native_dynamic_derivation(
    unit: &DynamicUnit,
    sources: &BTreeMap<String, StorePath<String>>,
    registered_units: &BTreeMap<String, StorePath<String>>,
    store_dir: &str,
) -> Result<Derivation, Error> {
    let ca_hash = unit.derivation.fixed_output.as_ref().map(parse_dynamic_fixed_output).transpose()?;
    let outputs = unit
        .derivation
        .outputs
        .iter()
        .map(|output_name| {
            (output_name.clone(), Output {
                path: None,
                ca_hash: if output_name == "out" { ca_hash.clone() } else { None },
            })
        })
        .collect();
    let mut environment: BTreeMap<String, BString> =
        unit.derivation.env.iter().map(|(key, value)| (key.clone(), value.as_bytes().into())).collect();
    environment.insert("system".to_string(), unit.derivation.system.as_bytes().into());
    environment.insert("builder".to_string(), unit.derivation.builder.as_bytes().into());
    environment.insert("name".to_string(), unit.derivation.name.as_bytes().into());
    environment.extend(unit.derivation.outputs.iter().map(|output_name| (output_name.clone(), BString::from(""))));
    environment.insert("outputs".to_string(), unit.derivation.outputs.join(" ").as_bytes().into());

    let (input_derivations, input_sources) = resolve_native_dynamic_inputs(unit, sources, registered_units, store_dir)?;
    Ok(Derivation {
        arguments: unit.derivation.args.clone(),
        builder: unit.derivation.builder.clone(),
        environment,
        input_derivations,
        input_sources,
        outputs,
        system: unit.derivation.system.clone(),
    })
}

fn resolve_native_dynamic_inputs(
    unit: &DynamicUnit,
    sources: &BTreeMap<String, StorePath<String>>,
    registered_units: &BTreeMap<String, StorePath<String>>,
    store_dir: &str,
) -> Result<(BTreeMap<StorePath<String>, BTreeSet<String>>, BTreeSet<StorePath<String>>), Error> {
    let mut input_derivations: BTreeMap<StorePath<String>, BTreeSet<String>> = BTreeMap::new();
    let mut input_sources = BTreeSet::new();
    for input in &unit.derivation.inputs {
        match input {
            DynamicInput::StorePath { path } => {
                input_sources.insert(parse_dynamic_store_path(path, store_dir)?);
            }
            DynamicInput::Source { source } => {
                let path = sources.get(source).ok_or_else(|| {
                    Error::Store(format!("native dynamic unit '{}' references unknown source {source}", unit.id))
                })?;
                input_sources.insert(path.clone());
            }
            DynamicInput::UnitOutput {
                unit: dependency,
                output,
            } => {
                let drv_path = registered_units.get(dependency).ok_or_else(|| {
                    Error::Store(format!(
                        "native dynamic unit '{}' references unregistered dynamic unit {dependency}",
                        unit.id
                    ))
                })?;
                input_derivations.entry(drv_path.clone()).or_default().insert(output.clone());
            }
        }
    }
    Ok((input_derivations, input_sources))
}

/// Mutable loop state shared across worker helper methods.
///
/// Groups the recurring `&mut` references that flow through
/// dispatch, completion, and wait helpers. Keeps the method
/// signatures under the 5-parameter limit.
struct WorkerLoopState<'a> {
    sem: Arc<Semaphore>,
    join_set: JoinSet<(String, Result<snix_build::buildservice::BuildResult, Error>)>,
    pending_meta: HashMap<String, PreparedBuild>,
    outcomes: &'a mut Vec<BuildOutcome>,
    failed: &'a mut Vec<FailedGoal>,
    native_dynamic_plans: &'a mut Vec<NativeDynamicPlanReport>,
    completed_count: u32,
}

/// Build scheduler that drives Goal state machines.
///
/// The Worker is the imperative shell. It holds mutable state
/// (goal registry, deterministic priority-ready set, in-flight tracking) and borrows
/// the Builder for all I/O. Goal transitions and scheduling are functional cores.
pub struct Worker {
    registry: GoalRegistry,
    ready_goals: BTreeMap<String, ReadyGoalFacts>,
    goal_preferences: BTreeMap<String, (OperatorPolicyClass, EligiblePreferenceFacts)>,
    scheduling_policy: SchedulingPolicy,
    scheduling_history: HistoryInput,
    scheduling_epoch: u32,
    pressure_cache: BTreeMap<String, KnownGraphPressure>,
    pressure_dirty_goals: BTreeSet<String>,
    priority_decisions: Vec<PriorityDecisionEvidence>,
    max_jobs: u32,
}

fn redacted_goal_identity(goal_key: &str) -> String {
    let digest = blake3::hash(goal_key.as_bytes()).to_hex().to_string();
    debug_assert_eq!(digest.len(), blake3::OUT_LEN * BLAKE3_HEX_CHARS_PER_BYTE);
    debug_assert!(!digest.chars().any(char::is_control));
    digest
}

impl Worker {
    /// Create a new Worker with the given concurrency limit.
    pub fn new(max_jobs: u32) -> Self {
        Self::with_scheduling_policy(max_jobs, SchedulingPolicy::default())
            .expect("default scheduling policy must be valid")
    }

    /// Create a Worker with an explicit, validated scheduling policy.
    pub fn with_scheduling_policy(max_jobs: u32, scheduling_policy: SchedulingPolicy) -> Result<Self, Error> {
        if max_jobs == 0 || max_jobs > MAX_IN_FLIGHT {
            return Err(Error::Store(format!("max_jobs must be between 1 and {MAX_IN_FLIGHT}; got {max_jobs}")));
        }
        debug_assert!(max_jobs >= 1, "max_jobs must be >= 1");
        debug_assert!(max_jobs <= MAX_IN_FLIGHT, "max_jobs exceeds MAX_IN_FLIGHT");
        scheduling_policy
            .validate()
            .map_err(|error| Error::Store(format!("invalid scheduling policy: {error}")))?;

        Ok(Self {
            registry: GoalRegistry::new(),
            ready_goals: BTreeMap::new(),
            goal_preferences: BTreeMap::new(),
            scheduling_policy,
            scheduling_history: HistoryInput::Missing,
            scheduling_epoch: 0,
            pressure_cache: BTreeMap::new(),
            pressure_dirty_goals: BTreeSet::new(),
            priority_decisions: Vec::new(),
            max_jobs,
        })
    }

    /// Replace optional duration-class history and invalidate only known graph
    /// components when the next priority snapshot is taken.
    pub fn set_scheduling_history(&mut self, scheduling_history: HistoryInput) {
        self.scheduling_history = scheduling_history;
        self.pressure_dirty_goals.extend(self.registry.iter().map(|(key, _)| key.to_string()));
        debug_assert_eq!(self.pressure_dirty_goals.len(), self.registry.iter().count());
        debug_assert!(u32::try_from(self.pressure_dirty_goals.len()).is_ok_and(|count| count <= MAX_GOALS));
    }

    /// Set provider-neutral preference facts for a known goal.
    pub fn set_goal_scheduling_preference(
        &mut self,
        goal_key: &str,
        operator_policy_class: OperatorPolicyClass,
        preference: EligiblePreferenceFacts,
    ) -> Result<(), Error> {
        if !self.registry.contains(goal_key) {
            return Err(Error::Store(format!(
                "scheduler preference references unknown goal identity blake3:{}",
                redacted_goal_identity(goal_key)
            )));
        }
        self.goal_preferences.insert(goal_key.to_string(), (operator_policy_class, preference));
        if let Some(ready) = self.ready_goals.get_mut(goal_key) {
            ready.operator_policy_class = operator_policy_class;
            ready.preference = preference;
        }
        debug_assert!(self.goal_preferences.contains_key(goal_key));
        debug_assert!(self.ready_goals.get(goal_key).is_none_or(|ready| {
            ready.operator_policy_class == operator_policy_class && ready.preference == preference
        }));
        Ok(())
    }

    fn enqueue_ready_goal(&mut self, goal_key: &str) -> Result<(), Error> {
        if self.ready_goals.contains_key(goal_key) {
            return Ok(());
        }
        let ready_goal_count = u32::try_from(self.ready_goals.len())
            .map_err(|_| Error::Store("ready goal count exceeds u32".to_string()))?;
        if ready_goal_count >= MAX_GOALS {
            return Err(Error::Store(format!("ready goal limit exceeded ({MAX_GOALS})")));
        }
        let goal = self.registry.get(goal_key).ok_or_else(|| {
            Error::Store(format!("ready goal identity is unknown: blake3:{}", redacted_goal_identity(goal_key)))
        })?;
        if goal.state != GoalState::Ready {
            return Err(Error::Store(format!(
                "cannot enqueue goal identity blake3:{} in state {:?}",
                redacted_goal_identity(goal_key),
                goal.state
            )));
        }
        let (operator_policy_class, preference) = self.goal_preferences.get(goal_key).copied().unwrap_or_default();
        let mut ready = ReadyGoalFacts::ordinary(goal_key.to_string(), self.scheduling_epoch);
        ready.operator_policy_class = operator_policy_class;
        ready.preference = preference;
        self.ready_goals.insert(goal_key.to_string(), ready);
        debug_assert!(self.ready_goals.contains_key(goal_key));
        debug_assert!(u32::try_from(self.ready_goals.len()).is_ok_and(|count| count <= MAX_GOALS));
        Ok(())
    }

    fn known_graph_facts(&self) -> Result<KnownGraphFacts, Error> {
        let mut nodes: BTreeMap<String, (Vec<String>, BTreeSet<String>, bool)> = BTreeMap::new();
        let mut edge_count = 0_u32;
        for (goal_key, goal) in self.registry.iter() {
            let goal_edge_count = u32::try_from(goal.waitees.len())
                .map_err(|_| Error::Store("goal dependency count exceeds u32".to_string()))?;
            edge_count = edge_count
                .checked_add(goal_edge_count)
                .ok_or_else(|| Error::Store("known graph edge count overflow".to_string()))?;
            if edge_count > MAX_KNOWN_GRAPH_EDGES {
                return Err(Error::Store(format!("known graph edge limit exceeded ({MAX_KNOWN_GRAPH_EDGES})")));
            }
            let mut waitees = goal.waitees.clone();
            waitees.sort();
            nodes.insert(goal_key.to_string(), (waitees, BTreeSet::new(), goal.is_root));
        }
        let dependency_edges: Vec<(String, String)> = nodes
            .iter()
            .flat_map(|(goal_key, (waitees, _, _))| {
                waitees.iter().map(|waitee| (goal_key.clone(), waitee.clone())).collect::<Vec<_>>()
            })
            .collect();
        for (waiter, waitee) in dependency_edges {
            if let Some((_, waiters, _)) = nodes.get_mut(&waitee) {
                waiters.insert(waiter);
            }
        }
        let goals: Vec<KnownGoalFacts> = nodes
            .into_iter()
            .map(|(goal_key, (waitees, waiters, requested_root))| KnownGoalFacts {
                goal_key,
                waitees,
                waiters: waiters.into_iter().collect(),
                requested_root,
            })
            .collect();
        debug_assert_eq!(self.registry.iter().count(), goals.len());
        debug_assert!(u32::try_from(goals.len()).is_ok_and(|count| count <= MAX_GOALS));
        debug_assert!(edge_count <= MAX_KNOWN_GRAPH_EDGES);
        Ok(KnownGraphFacts { goals })
    }

    fn refresh_pressure_cache(&mut self) -> Result<(HistoryBasis, Option<String>), Error> {
        let graph = self.known_graph_facts()?;
        let update = recompute_affected_pressure(
            &graph,
            &self.pressure_cache,
            &self.pressure_dirty_goals,
            &self.scheduling_policy,
            &self.scheduling_history,
        )
        .map_err(|error| Error::Store(format!("priority pressure: {error}")))?;
        self.pressure_cache = update.pressures;
        self.pressure_dirty_goals.clear();
        debug_assert_eq!(self.pressure_cache.len(), self.registry.iter().count());
        debug_assert!(self.pressure_dirty_goals.is_empty());
        Ok((update.history_basis, update.history_snapshot_digest_blake3))
    }

    fn select_next_ready_goal(&mut self) -> Result<Option<String>, Error> {
        if self.ready_goals.is_empty() {
            return Ok(None);
        }
        let next_epoch = advance_scheduling_epoch(self.scheduling_epoch)
            .map_err(|error| Error::Store(format!("priority epoch: {error}")))?;
        let (history_basis, history_digest) = self.refresh_pressure_cache()?;
        let ready_facts: Vec<ReadyGoalFacts> = self.ready_goals.values().cloned().collect();
        let ranked = rank_ready_goals(&self.scheduling_policy, next_epoch, &ready_facts, &self.pressure_cache)
            .map_err(|error| Error::Store(format!("priority ranking: {error}")))?;
        let evidence =
            priority_decision_evidence(&self.scheduling_policy, next_epoch, &ranked, history_basis, history_digest)
                .map_err(|error| Error::Store(format!("priority evidence: {error}")))?;
        let priority_decision_count = u32::try_from(self.priority_decisions.len())
            .map_err(|_| Error::Store("priority decision count exceeds u32".to_string()))?;
        if priority_decision_count >= MAX_PRIORITY_DECISIONS {
            return Err(Error::Store(format!("priority decision limit exceeded ({MAX_PRIORITY_DECISIONS})")));
        }
        let selected = ranked
            .first()
            .map(|candidate| candidate.goal_key.clone())
            .ok_or_else(|| Error::Store("ranked ready set was unexpectedly empty".to_string()))?;
        self.ready_goals.remove(&selected);
        self.priority_decisions.push(evidence);
        self.scheduling_epoch = next_epoch;
        debug_assert!(!self.ready_goals.contains_key(&selected));
        debug_assert_eq!(self.priority_decisions.len() as u32, next_epoch);
        Ok(Some(selected))
    }

    /// Lazily create a goal for a derivation. If the goal already
    /// exists, this is a no-op. Inspects the derivation's
    /// `input_derivations` to discover deps, creating sub-goals
    /// and wiring waiters.
    ///
    /// `is_root`: whether this was explicitly requested by the user.
    ///
    /// Two-pass BFS: first creates all goals (so deps exist before
    /// wiring), then wires waiter edges and inspects states.
    /// Uses an explicit queue instead of recursion.
    pub fn want(
        &mut self,
        drv_path: &StorePath<String>,
        known_paths: &DerivationRegistry,
        is_root: bool,
    ) -> Result<(), Error> {
        // Pass 1: BFS to create all goals. Collect (key, dep_paths)
        // for each newly created goal, in creation order.
        let created = self.create_goals_bfs(drv_path, known_paths, is_root)?;

        // Pass 2: Wire deps and inspect, in creation order (leaves
        // first since BFS processes deps before dependents).
        for (key, dep_drv_paths) in &created {
            self.wire_deps_and_inspect(key, dep_drv_paths)?;
        }

        Ok(())
    }

    /// BFS pass: create Goal structs for a root and all its transitive
    /// deps. Returns `(key, dep_paths)` for each newly created goal.
    fn create_goals_bfs(
        &mut self,
        drv_path: &StorePath<String>,
        known_paths: &DerivationRegistry,
        is_root: bool,
    ) -> Result<Vec<CreatedGoal>, Error> {
        let mut queue: VecDeque<(StorePath<String>, bool)> = VecDeque::new();
        queue.push_back((drv_path.clone(), is_root));

        let mut created: Vec<CreatedGoal> = Vec::with_capacity(64);
        // Tiger Style: fixed iteration limit.
        let goal_count_max: u32 = MAX_GOALS;
        let mut iterations: u32 = 0;

        while let Some((sp, root)) = queue.pop_front() {
            iterations = iterations.saturating_add(1);
            if iterations > goal_count_max {
                return Err(Error::Store(format!("want() BFS exceeded iteration limit ({goal_count_max})")));
            }

            let key = sp.to_absolute_path();

            // Already tracked — just upgrade to root if needed.
            if let Some(goal) = self.registry.get_mut(&key) {
                if root && !goal.is_root {
                    goal.is_root = true;
                    self.pressure_dirty_goals.insert(key);
                }
                continue;
            }

            // Look up derivation.
            let drv_abs = sp.to_absolute_path_with_prefix(known_paths.store_dir());
            let entry = known_paths
                .get_by_drv_path(&drv_abs)
                .ok_or_else(|| Error::DerivationNotFound { path: sp.clone() })?;
            let derivation = entry.derivation.clone();
            let dep_drv_paths: Vec<StorePath<String>> = derivation.input_derivations.keys().cloned().collect();

            let goal = if root {
                Goal::new_root(sp.clone(), derivation)
            } else {
                Goal::new(sp, derivation)
            };
            self.registry.insert(key.clone(), goal)?;
            self.pressure_dirty_goals.insert(key.clone());

            // Enqueue deps for creation.
            for dep_sp in &dep_drv_paths {
                if !self.registry.contains(&dep_sp.to_absolute_path()) {
                    queue.push_back((dep_sp.clone(), false));
                }
            }

            created.push((key, dep_drv_paths));
        }

        debug_assert!(iterations <= goal_count_max);
        Ok(created)
    }

    /// Wire waiter edges for a goal's deps and inspect (Pending →
    /// Waiting/Ready). All deps must already exist in the registry.
    fn wire_deps_and_inspect(&mut self, key: &str, dep_drv_paths: &[StorePath<String>]) -> Result<(), Error> {
        let unbuilt_dep_keys: Vec<String> = dep_drv_paths
            .iter()
            .map(|sp| sp.to_absolute_path())
            .filter(|dep_key| self.registry.get(dep_key).map(|g| g.state != GoalState::Done).unwrap_or(false))
            .collect();

        for dep_key in &unbuilt_dep_keys {
            if let Some(dep_goal) = self.registry.get_mut(dep_key) {
                dep_goal.waiters.push(key.to_string());
            }
            self.pressure_dirty_goals.insert(dep_key.clone());
        }
        self.pressure_dirty_goals.insert(key.to_string());

        let goal = self
            .registry
            .get_mut(key)
            .ok_or_else(|| Error::Store(format!("worker: goal not found in registry: {key}")))?;
        let state = goal.inspect(unbuilt_dep_keys)?;

        let became_ready = *state == GoalState::Ready;
        if became_ready {
            self.enqueue_ready_goal(key)?;
        }

        debug_assert_eq!(became_ready, self.ready_goals.contains_key(key));
        Ok(())
    }

    /// Run the build loop until all root goals are terminal.
    ///
    /// Dispatches Ready goals through the Builder, spawns sandbox
    /// builds on a JoinSet, and processes completions. Cache hits
    /// and fetchers complete synchronously via `prepare_build`.
    pub async fn run<BServ>(
        &mut self,
        builder: &mut Builder<BServ>,
        known_paths: &mut DerivationRegistry,
    ) -> Result<WorkerResult, Error>
    where
        BServ: BuildService + 'static,
    {
        let total_goals = self.registry.len();
        let root_count = self.registry.root_count();
        info!(goals = total_goals, roots = root_count, jobs = self.max_jobs, "worker starting");

        debug_assert!(root_count > 0, "no root goals to build");
        debug_assert!(total_goals <= MAX_GOALS, "goal count exceeds limit");

        let mut outcomes: Vec<BuildOutcome> = Vec::new();
        let mut failed: Vec<FailedGoal> = Vec::new();
        let mut native_dynamic_plans: Vec<NativeDynamicPlanReport> = Vec::new();
        let mut state = WorkerLoopState {
            sem: Arc::new(Semaphore::new(
                usize::try_from(self.max_jobs).map_err(|e| Error::Store(format!("max_jobs overflow: {e}")))?,
            )),
            join_set: JoinSet::new(),
            pending_meta: HashMap::new(),
            outcomes: &mut outcomes,
            failed: &mut failed,
            native_dynamic_plans: &mut native_dynamic_plans,
            completed_count: 0,
        };
        let iteration_count_max: u32 = total_goals.saturating_mul(4).max(16);

        for _ in 0..iteration_count_max {
            let dispatched = self.dispatch_ready(builder, known_paths, &mut state).await?;

            if self.registry.all_roots_terminal() {
                info!(
                    completed = state.completed_count,
                    succeeded = state.outcomes.len(),
                    failed = state.failed.len(),
                    "worker finished"
                );
                let priority_decisions = std::mem::take(&mut self.priority_decisions);
                return Ok(finish_worker_result(outcomes, failed, native_dynamic_plans, priority_decisions));
            }

            if state.join_set.is_empty() {
                if dispatched == 0 {
                    return Err(Error::Store("worker deadlock: no ready goals and no in-flight builds".into()));
                }
                continue;
            }

            let Some(join_result) = state.join_set.join_next().await else {
                continue;
            };
            self.process_join_result(join_result, builder, known_paths, &mut state).await?;
            state.completed_count = state.completed_count.saturating_add(1);
        }
        Err(Error::Store(format!("worker loop exceeded iteration limit ({iteration_count_max})")))
    }

    /// Run the build loop, receiving new roots from an eval channel.
    ///
    /// The eval thread converts derivations and sends `EvalMessage`s.
    /// The Worker calls `want()` for each arrival, interleaving new
    /// root discovery with build dispatch and completion. Builds
    /// start as soon as leaf derivations are ready — no waiting for
    /// all eval to finish.
    ///
    /// Terminates when: the channel is closed (eval done) AND all
    /// root goals are terminal.
    pub async fn run_streaming<BServ>(
        &mut self,
        builder: &mut Builder<BServ>,
        known_paths: &mut DerivationRegistry,
        rx: &mut mpsc::Receiver<EvalMessage>,
    ) -> Result<WorkerResult, Error>
    where
        BServ: BuildService + 'static,
    {
        let mut outcomes: Vec<BuildOutcome> = Vec::new();
        let mut failed: Vec<FailedGoal> = Vec::new();
        let mut native_dynamic_plans: Vec<NativeDynamicPlanReport> = Vec::new();
        let mut state = WorkerLoopState {
            sem: Arc::new(Semaphore::new(
                usize::try_from(self.max_jobs).map_err(|e| Error::Store(format!("max_jobs overflow: {e}")))?,
            )),
            join_set: JoinSet::new(),
            pending_meta: HashMap::new(),
            outcomes: &mut outcomes,
            failed: &mut failed,
            native_dynamic_plans: &mut native_dynamic_plans,
            completed_count: 0,
        };
        let mut is_eval_done = false;
        let iteration_count_max: u32 = MAX_GOALS.saturating_mul(4);

        info!(jobs = self.max_jobs, "worker streaming started");

        for _ in 0..iteration_count_max {
            if !is_eval_done {
                self.drain_eval_messages(rx, known_paths, &mut is_eval_done)?;
            }

            self.dispatch_ready(builder, known_paths, &mut state).await?;

            if is_eval_done && self.registry.all_roots_terminal() {
                info!(
                    completed = state.completed_count,
                    succeeded = state.outcomes.len(),
                    failed = state.failed.len(),
                    roots = self.registry.root_count(),
                    "worker streaming finished"
                );
                let priority_decisions = std::mem::take(&mut self.priority_decisions);
                return Ok(finish_worker_result(outcomes, failed, native_dynamic_plans, priority_decisions));
            }

            self.wait_for_event(&mut is_eval_done, rx, builder, known_paths, &mut state).await?;
        }
        Err(Error::Store(format!("worker loop exceeded iteration limit ({iteration_count_max})")))
    }

    /// Wait for either a build completion or an eval message.
    /// Centralized select! to avoid duplicating handle_build_completion.
    async fn wait_for_event<BServ>(
        &mut self,
        is_eval_done: &mut bool,
        rx: &mut mpsc::Receiver<EvalMessage>,
        builder: &mut Builder<BServ>,
        known_paths: &mut DerivationRegistry,
        state: &mut WorkerLoopState<'_>,
    ) -> Result<(), Error>
    where
        BServ: BuildService + 'static,
    {
        if *is_eval_done {
            let Some(join_result) = state.join_set.join_next().await else {
                return Ok(());
            };
            self.process_join_result(join_result, builder, known_paths, state).await?;
            state.completed_count = state.completed_count.saturating_add(1);
        } else if state.join_set.is_empty() {
            match rx.recv().await {
                Some(msg) => self.accept_eval_message(msg, known_paths)?,
                None => *is_eval_done = true,
            }
        } else {
            tokio::select! {
                result = state.join_set.join_next() => {
                    let Some(join_result) = result else { return Ok(()); };
                    self.process_join_result(join_result, builder, known_paths, state).await?;
                    state.completed_count = state.completed_count.saturating_add(1);
                }
                msg = rx.recv() => {
                    match msg {
                        Some(msg) => self.accept_eval_message(msg, known_paths)?,
                        None => *is_eval_done = true,
                    }
                }
            }
        }
        Ok(())
    }

    /// Drain buffered eval messages without blocking.
    /// Tiger Style: fixed limit prevents unbounded iteration.
    fn drain_eval_messages(
        &mut self,
        rx: &mut mpsc::Receiver<EvalMessage>,
        known_paths: &mut DerivationRegistry,
        is_eval_done: &mut bool,
    ) -> Result<(), Error> {
        let drain_count_max: u32 = 256;
        for _ in 0..drain_count_max {
            match rx.try_recv() {
                Ok(msg) => {
                    self.accept_eval_message(msg, known_paths)?;
                }
                Err(mpsc::error::TryRecvError::Empty) => break,
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    *is_eval_done = true;
                    break;
                }
            }
        }
        Ok(())
    }

    /// Process a single eval message: insert new entries into the
    /// registry, then create a root goal.
    ///
    /// Entries arrive from the convert thread. Diamond deps that
    /// were already registered by an earlier message are silently
    /// skipped (DerivationRegistry.insert is idempotent by drv path).
    fn accept_eval_message(&mut self, msg: EvalMessage, known_paths: &mut DerivationRegistry) -> Result<(), Error> {
        debug!(
            label = %msg.label,
            drv = %msg.drv_path,
            new_entries = msg.new_entries.len(),
            "received derivation from eval",
        );

        for (drv_path, hdm, derivation, content_addressed, dynamic_plan_outputs, provenance_claims) in msg.new_entries {
            known_paths.insert_with_dynamic_plan_outputs(
                drv_path,
                hdm,
                derivation,
                content_addressed,
                dynamic_plan_outputs,
                provenance_claims,
            );
        }

        self.want(&msg.drv_path, known_paths, true)
    }

    /// Process a JoinSet result: unwrap the join, then handle success
    /// or failure. Build errors are caught and routed to
    /// instead of aborting the entire Worker.
    async fn process_join_result<BServ>(
        &mut self,
        join_result: Result<(String, Result<snix_build::buildservice::BuildResult, Error>), tokio::task::JoinError>,
        builder: &mut Builder<BServ>,
        known_paths: &mut DerivationRegistry,
        state: &mut WorkerLoopState<'_>,
    ) -> Result<(), Error>
    where
        BServ: BuildService + 'static,
    {
        match join_result {
            Ok((drv_key, Ok(build_result))) => {
                self.handle_build_completion(&drv_key, build_result, builder, known_paths, state).await
            }
            Ok((drv_key, Err(build_err))) => {
                let err_msg = format!("{build_err}");
                tracing::warn!(drv = %drv_key, err = %err_msg, "sandbox build failed");
                state.pending_meta.remove(&drv_key);
                self.fail_goal(&drv_key, &err_msg, state.failed)?;
                Ok(())
            }
            Err(join_err) => {
                tracing::error!(err = %join_err, "build task panicked");
                Err(Error::Store(format!("task join: {join_err}")))
            }
        }
    }

    /// Handle a completed sandbox build: finish_build, detect dynamic
    /// derivations, and notify waiters. Errors from finish_build are
    /// caught and routed to  for partial failure.
    async fn handle_build_completion<BServ>(
        &mut self,
        drv_key: &str,
        build_result: snix_build::buildservice::BuildResult,
        builder: &mut Builder<BServ>,
        known_paths: &mut DerivationRegistry,
        state: &mut WorkerLoopState<'_>,
    ) -> Result<(), Error>
    where
        BServ: BuildService + 'static,
    {
        let prepared = state
            .pending_meta
            .remove(drv_key)
            .ok_or_else(|| Error::Store(format!("BUG: completed build has no pending metadata: {drv_key}")))?;

        let outcome = match builder.finish_build(&prepared, build_result, known_paths).await {
            Ok(outcome) => outcome,
            Err(e) => {
                let err_msg = format!("{e}");
                tracing::warn!(drv = drv_key, err = %err_msg, "build failed, marking goal as failed");
                self.fail_goal(drv_key, &err_msg, state.failed)?;
                return Ok(());
            }
        };

        self.handle_completed_outcome(drv_key, outcome, builder, known_paths, state).await
    }

    async fn handle_completed_outcome<BServ>(
        &mut self,
        drv_key: &str,
        outcome: BuildOutcome,
        builder: &Builder<BServ>,
        known_paths: &mut DerivationRegistry,
        state: &mut WorkerLoopState<'_>,
    ) -> Result<(), Error>
    where
        BServ: BuildService + 'static,
    {
        let declared_native_outputs = self.declared_native_dynamic_outputs(&outcome, known_paths);
        let native_scan = self
            .scan_native_dynamic_plans(drv_key, &outcome, builder, known_paths.store_dir(), &declared_native_outputs)
            .await?;
        self.log_native_dynamic_plan_scan(&native_scan);
        self.register_accepted_native_dynamic_plans(&native_scan, known_paths)?;
        state.native_dynamic_plans.extend(native_plan_reports_from_scan(&native_scan));

        self.detect_dynamic_derivations(drv_key, &outcome, builder, known_paths, &declared_native_outputs)
            .await?;

        self.complete_goal(drv_key, outcome, state.outcomes, state.failed)
    }

    fn declared_native_dynamic_outputs(
        &self,
        outcome: &BuildOutcome,
        known_paths: &DerivationRegistry,
    ) -> BTreeSet<String> {
        let drv_abs = outcome.drv_path.to_absolute_path_with_prefix(known_paths.store_dir());
        let Some(entry) = known_paths.get_by_drv_path(&drv_abs) else {
            return BTreeSet::new();
        };
        entry.dynamic_plan_outputs.iter().cloned().collect()
    }

    async fn scan_native_dynamic_plans<BServ>(
        &self,
        producer_key: &str,
        outcome: &BuildOutcome,
        builder: &Builder<BServ>,
        store_prefix: &str,
        declared_outputs: &BTreeSet<String>,
    ) -> Result<NativeDynamicPlanScan, Error>
    where
        BServ: BuildService + 'static,
    {
        let mut scan = NativeDynamicPlanScan::default();
        for output_name in declared_outputs {
            match self
                .scan_declared_native_dynamic_output(producer_key, output_name, outcome, builder, store_prefix)
                .await?
            {
                NativeDynamicPlanOutputScan::Accepted(accepted) => scan.accepted.push(accepted),
                NativeDynamicPlanOutputScan::Rejected(rejected) => scan.rejected.push(rejected),
            }
        }
        Ok(scan)
    }

    async fn scan_declared_native_dynamic_output<BServ>(
        &self,
        producer_key: &str,
        output_name: &str,
        outcome: &BuildOutcome,
        builder: &Builder<BServ>,
        store_prefix: &str,
    ) -> Result<NativeDynamicPlanOutputScan, Error>
    where
        BServ: BuildService + 'static,
    {
        let Some(path_info) = outcome.outputs.get(output_name) else {
            return Ok(NativeDynamicPlanOutputScan::Rejected(native_plan_rejection(
                producer_key,
                output_name,
                None,
                None,
                NativeDynamicPlanRejectionKind::MissingOutput,
                "declared dynamic-plan output is missing from build result".to_string(),
            )));
        };

        let Some(size) = node_file_size(&path_info.node) else {
            return Ok(NativeDynamicPlanOutputScan::Rejected(native_plan_rejection(
                producer_key,
                output_name,
                Some(path_info.store_path.clone()),
                None,
                NativeDynamicPlanRejectionKind::NonRegularOutput,
                "declared dynamic-plan output is not a regular file".to_string(),
            )));
        };

        if size > MAX_DYNAMIC_PLAN_BYTES {
            return Ok(NativeDynamicPlanOutputScan::Rejected(native_plan_rejection(
                producer_key,
                output_name,
                Some(path_info.store_path.clone()),
                None,
                NativeDynamicPlanRejectionKind::PlanTooLarge,
                format!("declared dynamic-plan output is {size} bytes; limit is {MAX_DYNAMIC_PLAN_BYTES}"),
            )));
        }

        let content = match builder.read_blob(&path_info.node).await {
            Ok(content) => content,
            Err(err) => {
                return Ok(NativeDynamicPlanOutputScan::Rejected(native_plan_rejection(
                    producer_key,
                    output_name,
                    Some(path_info.store_path.clone()),
                    None,
                    NativeDynamicPlanRejectionKind::ReadFailed,
                    err.to_string(),
                )));
            }
        };
        let raw_digest = blake3_hex(&content);
        match decode_validated_plan_v1(&content, store_prefix) {
            Ok(plan) => Ok(NativeDynamicPlanOutputScan::Accepted(accepted_native_plan(
                producer_key,
                output_name,
                path_info.store_path.clone(),
                raw_digest,
                plan,
            ))),
            Err(err) => Ok(NativeDynamicPlanOutputScan::Rejected(native_plan_rejection(
                producer_key,
                output_name,
                Some(path_info.store_path.clone()),
                Some(raw_digest),
                NativeDynamicPlanRejectionKind::InvalidPlan,
                err.to_string(),
            ))),
        }
    }

    fn log_native_dynamic_plan_scan(&self, scan: &NativeDynamicPlanScan) {
        if scan.is_empty() {
            return;
        }
        for accepted in &scan.accepted {
            info!(
                producer = %accepted.producer_key,
                output = %accepted.output_name,
                plan_digest = %accepted.canonical_plan_digest,
                accepted_units = accepted.accepted_unit_ids.len(),
                "accepted native dynamic plan output"
            );
        }
        for rejected in &scan.rejected {
            tracing::warn!(
                producer = %rejected.producer_key,
                output = %rejected.output_name,
                reason = %rejected.kind.as_str(),
                detail = %rejected.detail,
                "rejected native dynamic plan output"
            );
        }
    }

    fn register_accepted_native_dynamic_plans(
        &mut self,
        scan: &NativeDynamicPlanScan,
        known_paths: &mut DerivationRegistry,
    ) -> Result<(), Error> {
        for accepted in &scan.accepted {
            let registered = register_native_dynamic_plan_units(accepted, known_paths)?;
            for root_drv_path in registered.root_drv_paths {
                self.want(&root_drv_path, known_paths, true)?;
            }
            debug_assert!(
                registered.unit_drv_paths.len() >= accepted.plan.plan.roots.len(),
                "registered native dynamic units must cover all roots"
            );
        }
        Ok(())
    }

    /// Inspect build outputs for `.drv` files. If found, parse and
    /// register them as new goals (dynamic derivations).
    async fn detect_dynamic_derivations<BServ>(
        &mut self,
        producer_key: &str,
        outcome: &BuildOutcome,
        builder: &Builder<BServ>,
        known_paths: &mut DerivationRegistry,
        native_dynamic_outputs: &BTreeSet<String>,
    ) -> Result<(), Error>
    where
        BServ: BuildService + 'static,
    {
        let discovered = self.scan_dynamic_derivations(outcome, builder, known_paths, native_dynamic_outputs).await?;
        if discovered.is_empty() {
            return Ok(());
        }
        self.activate_awaiting_dynamic_goals(producer_key, &discovered, known_paths)?;
        self.enqueue_discovered_dynamic_roots(&discovered, known_paths)?;
        Ok(())
    }

    async fn scan_dynamic_derivations<BServ>(
        &self,
        outcome: &BuildOutcome,
        builder: &Builder<BServ>,
        known_paths: &mut DerivationRegistry,
        native_dynamic_outputs: &BTreeSet<String>,
    ) -> Result<Vec<crate::dynamic::DynamicDrv>, Error>
    where
        BServ: BuildService + 'static,
    {
        let mut discovered: Vec<crate::dynamic::DynamicDrv> = Vec::with_capacity(outcome.outputs.len());
        for (output_name, path_info) in &outcome.outputs {
            if native_dynamic_outputs.contains(output_name) {
                continue;
            }
            if !crate::dynamic::is_drv_output(&path_info.store_path, &path_info.node) {
                continue;
            }
            let content = builder.read_blob(&path_info.node).await?;
            if let Some(drv) = crate::dynamic::parse_drv_bytes(&content)? {
                let sd = known_paths.store_dir().to_string();
                let drv_path = crate::dynamic::register_dynamic_drv(&drv, known_paths, &sd)?;
                info!(
                    producer = %outcome.drv_path.name(),
                    dynamic_drv = %drv_path.name(),
                    output = %output_name,
                    "detected dynamic derivation in build output"
                );
                discovered.push(crate::dynamic::DynamicDrv {
                    output_name: output_name.clone(),
                    drv_store_path: drv_path,
                    derivation: drv,
                });
            }
        }
        Ok(discovered)
    }

    fn activate_awaiting_dynamic_goals(
        &mut self,
        producer_key: &str,
        discovered: &[crate::dynamic::DynamicDrv],
        known_paths: &DerivationRegistry,
    ) -> Result<(), Error> {
        let awaiting = self.registry.awaiting_producer(producer_key);
        let Some(dyn_drv) = discovered.first() else {
            return Ok(());
        };
        for awaiting_key in &awaiting {
            let goal = self
                .registry
                .get_mut(awaiting_key)
                .ok_or_else(|| Error::Store(format!("worker: awaiting goal not found: {awaiting_key}")))?;
            goal.set_derivation(dyn_drv.derivation.clone())?;
            let dep_drv_paths: Vec<StorePath<String>> = dyn_drv.derivation.input_derivations.keys().cloned().collect();
            for dep_sp in &dep_drv_paths {
                let dep_key = dep_sp.to_absolute_path();
                if !self.registry.contains(&dep_key) {
                    let dep_abs = dep_sp.to_absolute_path_with_prefix(known_paths.store_dir());
                    if let Some(entry) = known_paths.get_by_drv_path(&dep_abs) {
                        let dep_goal = crate::goal::Goal::new(dep_sp.clone(), entry.derivation.clone());
                        self.registry.insert(dep_key.clone(), dep_goal)?;
                        self.pressure_dirty_goals.insert(dep_key.clone());
                    }
                }
            }
            self.wire_deps_and_inspect(awaiting_key, &dep_drv_paths)?;
        }
        Ok(())
    }

    fn enqueue_discovered_dynamic_roots(
        &mut self,
        discovered: &[crate::dynamic::DynamicDrv],
        known_paths: &mut DerivationRegistry,
    ) -> Result<(), Error> {
        for dyn_drv in discovered {
            let key = dyn_drv.drv_store_path.to_absolute_path();
            if self.registry.contains(&key) {
                continue;
            }
            self.want(&dyn_drv.drv_store_path, known_paths, true)?;
        }
        Ok(())
    }

    /// Dispatch all Ready goals. Returns the number dispatched.
    ///
    /// For each Ready goal:
    /// - Call `prepare_build` (may resolve as cache hit or fetcher)
    /// - If cache/fetcher: complete immediately, notify waiters
    /// - If sandbox build: spawn on JoinSet with Semaphore
    async fn dispatch_ready<BServ>(
        &mut self,
        builder: &mut Builder<BServ>,
        known_paths: &mut DerivationRegistry,
        state: &mut WorkerLoopState<'_>,
    ) -> Result<u32, Error>
    where
        BServ: BuildService + 'static,
    {
        let running_jobs = u32::try_from(state.join_set.len())
            .map_err(|_| Error::Store("running job count exceeds u32".to_string()))?;
        let mut available_build_slots = self.max_jobs.checked_sub(running_jobs).ok_or_else(|| {
            Error::Store(format!("running job count {running_jobs} exceeds configured max_jobs {}", self.max_jobs))
        })?;
        let mut dispatched: u32 = 0;
        while available_build_slots > 0 {
            let Some(drv_key) = self.select_next_ready_goal()? else {
                break;
            };
            let (drv_path, is_root, derivation) = self.ready_goal_inputs(&drv_key)?;
            let prepare_result = builder.prepare_build(&drv_path, derivation, known_paths, is_root).await;
            let spawned = self.handle_prepare_result(&drv_key, prepare_result, builder, known_paths, state).await?;
            if spawned {
                available_build_slots = available_build_slots
                    .checked_sub(1)
                    .ok_or_else(|| Error::Store("available build slot underflow".to_string()))?;
            }
            dispatched =
                dispatched.checked_add(1).ok_or_else(|| Error::Store("dispatch count overflow".to_string()))?;
        }
        debug_assert!(dispatched <= MAX_GOALS);
        debug_assert!(u32::try_from(state.join_set.len()).is_ok_and(|count| count <= self.max_jobs));
        Ok(dispatched)
    }

    fn ready_goal_inputs(
        &self,
        drv_key: &str,
    ) -> Result<(StorePath<String>, bool, Arc<nix_compat::derivation::Derivation>), Error> {
        let goal = self
            .registry
            .get(drv_key)
            .ok_or_else(|| Error::Store(format!("ready goal missing from registry: {drv_key}")))?;
        debug_assert_eq!(goal.state, GoalState::Ready, "goal selected from ready set but state is {:?}", goal.state);
        let derivation = goal
            .derivation
            .as_ref()
            .ok_or_else(|| Error::Store(format!("goal {drv_key} in Ready state but has no derivation")))?
            .clone();
        Ok((goal.drv_path.clone(), goal.is_root, derivation))
    }

    async fn handle_prepare_result<BServ>(
        &mut self,
        drv_key: &str,
        prepare_result: Result<PrepareResult, Error>,
        builder: &Builder<BServ>,
        known_paths: &mut DerivationRegistry,
        state: &mut WorkerLoopState<'_>,
    ) -> Result<bool, Error>
    where
        BServ: BuildService + 'static,
    {
        match prepare_result {
            Ok(PrepareResult::Done(outcome)) => {
                self.handle_completed_outcome(drv_key, outcome, builder, known_paths, state).await?;
                Ok(false)
            }
            Err(err) => {
                let err_msg = format!("{err}");
                tracing::warn!(drv = %drv_key, err = %err_msg, "prepare_build failed");
                self.fail_goal(drv_key, &err_msg, state.failed)?;
                Ok(false)
            }
            Ok(PrepareResult::NeedsBuild {
                prepared,
                build_request,
            }) => {
                self.spawn_prepared_build(drv_key, prepared, *build_request, builder, state)?;
                Ok(true)
            }
        }
    }

    fn spawn_prepared_build<BServ>(
        &mut self,
        drv_key: &str,
        prepared: PreparedBuild,
        build_request: snix_build::buildservice::BuildRequest,
        builder: &Builder<BServ>,
        state: &mut WorkerLoopState<'_>,
    ) -> Result<(), Error>
    where
        BServ: BuildService + 'static,
    {
        let goal = self
            .registry
            .get_mut(drv_key)
            .ok_or_else(|| Error::Store(format!("worker: goal vanished during dispatch: {drv_key}")))?;
        goal.mark_building()?;

        let replaced = state.pending_meta.insert(drv_key.to_string(), prepared);
        debug_assert!(replaced.is_none(), "pending_meta already had entry for {drv_key}");

        let bs = builder.build_service();
        let sem = state.sem.clone();
        let key = drv_key.to_string();
        state.join_set.spawn(async move {
            let _permit = sem.acquire_owned().await.map_err(|e| Error::Store(format!("semaphore: {e}")));
            let build_result = match _permit {
                Ok(_p) => bs.do_build(build_request).await.map_err(|e| Error::Store(format!("build: {e}"))),
                Err(e) => Err(e),
            };
            (key, build_result)
        });
        Ok(())
    }

    /// Mark a goal as Done, collect its outcome, and notify waiters.
    /// If a waiter becomes Ready, admit it to the priority-ready set.
    fn complete_goal(
        &mut self,
        drv_key: &str,
        outcome: BuildOutcome,
        outcomes: &mut Vec<BuildOutcome>,
        _failed: &mut Vec<FailedGoal>,
    ) -> Result<(), Error> {
        let goal = self
            .registry
            .get_mut(drv_key)
            .ok_or_else(|| Error::Store(format!("completing unknown goal: {drv_key}")))?;

        // Transition: Ready or Building → Done.
        // Cache hits are still in Ready state. Sandbox builds are in Building.
        match goal.state {
            GoalState::Ready => {
                // Cache hit / fetcher — skip Building, go directly to Done.
                goal.mark_building()?;
                goal.mark_done()?;
            }
            GoalState::Building => {
                goal.mark_done()?;
            }
            _ => {
                return Err(Error::Store(format!("completing goal {drv_key} in {:?} state", goal.state)));
            }
        }

        let is_root = goal.is_root;
        let waiters = std::mem::take(&mut goal.waiters);

        if is_root {
            outcomes.push(outcome);
        }

        debug!(drv = drv_key, waiters = waiters.len(), "goal completed");

        // Notify waiters.
        for waiter_key in &waiters {
            let waiter = self
                .registry
                .get_mut(waiter_key)
                .ok_or_else(|| Error::Store(format!("waiter goal missing: {waiter_key}")))?;

            let is_now_ready = waiter.notify_dep_done()?;
            if is_now_ready {
                self.enqueue_ready_goal(waiter_key)?;
            }
        }

        debug_assert!(u32::try_from(self.ready_goals.len()).is_ok_and(|count| count <= MAX_GOALS));
        Ok(())
    }

    /// Mark a goal as failed and propagate failure to all waiters.
    ///
    /// The error message is stored in the  for root goals
    /// so callers can report per-package errors.
    fn fail_goal(&mut self, drv_key: &str, error_msg: &str, failed: &mut Vec<FailedGoal>) -> Result<(), Error> {
        let goal = self
            .registry
            .get_mut(drv_key)
            .ok_or_else(|| Error::Store(format!("failing unknown goal: {drv_key}")))?;

        goal.mark_build_failed()?;

        let is_root = goal.is_root;
        let waiters = std::mem::take(&mut goal.waiters);
        let drv_name = goal.drv_path.name().to_string();

        if is_root {
            failed.push(FailedGoal {
                drv_key: drv_key.to_string(),
                error: error_msg.to_string(),
            });
        }

        info!(
            drv = drv_key,
            name = %drv_name,
            waiters = waiters.len(),
            "goal failed"
        );

        // Propagate failure to waiters.
        for waiter_key in &waiters {
            self.propagate_failure(waiter_key, &drv_name, failed)?;
        }

        Ok(())
    }

    /// Recursively propagate dep failure to waiting goals.
    fn propagate_failure(
        &mut self,
        drv_key: &str,
        failed_dep_name: &str,
        failed: &mut Vec<FailedGoal>,
    ) -> Result<(), Error> {
        let goal = self
            .registry
            .get_mut(drv_key)
            .ok_or_else(|| Error::Store(format!("propagating failure to unknown goal: {drv_key}")))?;

        // Only propagate to goals still Waiting or AwaitingDerivation.
        if !matches!(goal.state, GoalState::Waiting { .. } | GoalState::AwaitingDerivation) {
            return Ok(());
        }

        goal.notify_dep_failed()?;

        let is_root = goal.is_root;
        let waiters = std::mem::take(&mut goal.waiters);
        let drv_name = goal.drv_path.name().to_string();

        if is_root {
            failed.push(FailedGoal {
                drv_key: drv_key.to_string(),
                error: format!("dependency {failed_dep_name} failed"),
            });
        }

        for waiter_key in &waiters {
            self.propagate_failure(waiter_key, &drv_name, failed)?;
        }

        Ok(())
    }

    /// Read-only access to the goal registry (for testing/inspection).
    pub fn registry(&self) -> &GoalRegistry {
        &self.registry
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::collections::BTreeSet;

    use nix_compat::derivation::Derivation;
    use nix_compat::derivation::Output;

    use super::*;
    use crate::goal::GoalState;

    fn make_drv() -> Derivation {
        let mut outputs = BTreeMap::new();
        outputs.insert("out".to_string(), Output {
            path: None,
            ca_hash: None,
        });
        Derivation {
            arguments: vec![],
            builder: "/bin/sh".to_string(),
            environment: BTreeMap::new(),
            input_derivations: BTreeMap::new(),
            input_sources: BTreeSet::new(),
            outputs,
            system: "x86_64-linux".to_string(),
        }
    }

    fn make_drv_with_deps(deps: &[StorePath<String>]) -> Derivation {
        let mut drv = make_drv();
        for dep in deps {
            drv.input_derivations.insert(dep.clone(), BTreeSet::from(["out".into()]));
        }
        drv
    }

    fn fake_sp(name: &str) -> StorePath<String> {
        let mut digest = [0u8; 20];
        for (i, b) in name.bytes().enumerate() {
            digest[i % 20] ^= b;
        }
        StorePath::from_name_and_digest_fixed(name, digest).unwrap()
    }

    fn register_drv(kp: &mut DerivationRegistry, name: &str, drv: &Derivation) -> StorePath<String> {
        let sp = fake_sp(name);
        let aterm = drv.to_aterm_bytes();
        let aterm_hash = {
            use sha2::Digest;
            let mut hasher = sha2::Sha256::new();
            hasher.update(&aterm);
            let result = hasher.finalize();
            let mut hash = [0u8; 32];
            hash.copy_from_slice(&result);
            hash
        };
        let hdm = aterm_hash; // Use same for simplicity in tests.
        kp.insert(sp.clone(), hdm, drv.clone(), false, None);
        sp
    }

    // ── want() tests ────────────────────────────────────────────

    #[test]
    fn worker_rejects_zero_and_over_limit_job_budgets() {
        let over_limit = MAX_IN_FLIGHT.checked_add(1).unwrap();

        assert!(Worker::with_scheduling_policy(0, SchedulingPolicy::default()).is_err());
        assert!(Worker::with_scheduling_policy(over_limit, SchedulingPolicy::default()).is_err());
    }

    #[test]
    fn unknown_scheduling_preference_rejects_without_raw_identity_or_mutation() {
        let raw_goal = "/private/token=worker-secret/root.drv";
        let mut worker = Worker::new(1);

        let error = worker
            .set_goal_scheduling_preference(raw_goal, OperatorPolicyClass::Ordinary, EligiblePreferenceFacts::default())
            .unwrap_err();
        let diagnostic = error.to_string();

        assert!(!diagnostic.contains(raw_goal));
        assert!(!diagnostic.contains("worker-secret"));
        assert!(diagnostic.contains("blake3:"));
        assert!(worker.goal_preferences.is_empty());
    }

    #[test]
    fn want_single_leaf() {
        let mut kp = DerivationRegistry::default();
        let drv = make_drv();
        let sp = register_drv(&mut kp, "leaf.drv", &drv);

        let mut w = Worker::new(1);
        w.want(&sp, &kp, true).unwrap();

        assert_eq!(w.registry.len(), 1);
        let goal = w.registry.get(&sp.to_absolute_path()).unwrap();
        assert_eq!(goal.state, GoalState::Ready);
        assert!(goal.is_root);
        assert_eq!(w.ready_goals.len(), 1);
    }

    #[test]
    fn want_chain_creates_all_goals() {
        let mut kp = DerivationRegistry::default();

        let leaf_drv = make_drv();
        let leaf_sp = register_drv(&mut kp, "leaf.drv", &leaf_drv);

        let mid_drv = make_drv_with_deps(std::slice::from_ref(&leaf_sp));
        let mid_sp = register_drv(&mut kp, "mid.drv", &mid_drv);

        let top_drv = make_drv_with_deps(std::slice::from_ref(&mid_sp));
        let top_sp = register_drv(&mut kp, "top.drv", &top_drv);

        let mut w = Worker::new(1);
        w.want(&top_sp, &kp, true).unwrap();

        assert_eq!(w.registry.len(), 3);

        // Leaf is Ready (no deps).
        let leaf = w.registry.get(&leaf_sp.to_absolute_path()).unwrap();
        assert_eq!(leaf.state, GoalState::Ready);

        // Mid is Waiting (dep on leaf).
        let mid = w.registry.get(&mid_sp.to_absolute_path()).unwrap();
        assert_eq!(mid.state, GoalState::Waiting { remaining_deps: 1 });

        // Top is Waiting (dep on mid).
        let top = w.registry.get(&top_sp.to_absolute_path()).unwrap();
        assert_eq!(top.state, GoalState::Waiting { remaining_deps: 1 });
        assert!(top.is_root);

        // Only leaf is in the ready set.
        assert_eq!(w.ready_goals.len(), 1);
    }

    #[test]
    fn want_diamond_deduplicates() {
        let mut kp = DerivationRegistry::default();

        let shared_drv = make_drv();
        let shared_sp = register_drv(&mut kp, "shared.drv", &shared_drv);

        let left_drv = make_drv_with_deps(std::slice::from_ref(&shared_sp));
        let left_sp = register_drv(&mut kp, "left.drv", &left_drv);

        let right_drv = make_drv_with_deps(std::slice::from_ref(&shared_sp));
        let right_sp = register_drv(&mut kp, "right.drv", &right_drv);

        let top_drv = make_drv_with_deps(&[left_sp.clone(), right_sp.clone()]);
        let top_sp = register_drv(&mut kp, "top.drv", &top_drv);

        let mut w = Worker::new(2);
        w.want(&top_sp, &kp, true).unwrap();

        // 4 goals, not 5 — shared is deduped.
        assert_eq!(w.registry.len(), 4);

        // Shared has two waiters: left and right.
        let shared = w.registry.get(&shared_sp.to_absolute_path()).unwrap();
        assert_eq!(shared.waiters.len(), 2);
        assert_eq!(shared.state, GoalState::Ready);
    }

    #[test]
    fn want_duplicate_is_noop() {
        let mut kp = DerivationRegistry::default();
        let drv = make_drv();
        let sp = register_drv(&mut kp, "leaf.drv", &drv);

        let mut w = Worker::new(1);
        w.want(&sp, &kp, false).unwrap();
        w.want(&sp, &kp, false).unwrap();

        assert_eq!(w.registry.len(), 1);
    }

    #[test]
    fn duplicate_ready_admission_preserves_oldest_epoch() {
        let mut kp = DerivationRegistry::default();
        let drv = make_drv();
        let sp = register_drv(&mut kp, "dedup-ready.drv", &drv);
        let key = sp.to_absolute_path();
        let mut worker = Worker::new(1);
        worker.want(&sp, &kp, true).unwrap();
        let original_epoch = worker.ready_goals[&key].ready_since_epoch;
        worker.scheduling_epoch = crate::scheduling::DEFAULT_AGED_AFTER_EPOCHS;

        worker.enqueue_ready_goal(&key).unwrap();

        assert_eq!(worker.ready_goals.len(), 1);
        assert_eq!(worker.ready_goals[&key].ready_since_epoch, original_epoch);
        assert!(original_epoch < worker.scheduling_epoch);
    }

    #[test]
    fn want_upgrades_to_root() {
        let mut kp = DerivationRegistry::default();
        let drv = make_drv();
        let sp = register_drv(&mut kp, "leaf.drv", &drv);

        let mut w = Worker::new(1);
        w.want(&sp, &kp, false).unwrap();

        let g = w.registry.get(&sp.to_absolute_path()).unwrap();
        assert!(!g.is_root);

        w.want(&sp, &kp, true).unwrap();

        let g = w.registry.get(&sp.to_absolute_path()).unwrap();
        assert!(g.is_root);
    }

    #[test]
    fn want_unknown_drv_errors() {
        let kp = DerivationRegistry::default();
        let sp = fake_sp("unknown.drv");

        let mut w = Worker::new(1);
        assert!(w.want(&sp, &kp, true).is_err());
    }

    #[test]
    fn want_disjoint_trees() {
        let mut kp = DerivationRegistry::default();

        let a_drv = make_drv();
        let a_sp = register_drv(&mut kp, "a.drv", &a_drv);

        let b_drv = make_drv();
        let b_sp = register_drv(&mut kp, "b.drv", &b_drv);

        let mut w = Worker::new(2);
        w.want(&a_sp, &kp, true).unwrap();
        w.want(&b_sp, &kp, true).unwrap();

        assert_eq!(w.registry.len(), 2);
        assert_eq!(w.ready_goals.len(), 2);
        assert_eq!(w.registry.root_count(), 2);
    }

    // ── complete_goal / waiter notification ────────────────────

    #[test]
    fn complete_goal_notifies_waiters() {
        let mut kp = DerivationRegistry::default();

        let leaf_drv = make_drv();
        let leaf_sp = register_drv(&mut kp, "leaf.drv", &leaf_drv);

        let top_drv = make_drv_with_deps(std::slice::from_ref(&leaf_sp));
        let top_sp = register_drv(&mut kp, "top.drv", &top_drv);

        let mut w = Worker::new(1);
        w.want(&top_sp, &kp, true).unwrap();

        // Leaf is Ready, top is Waiting.
        let leaf_key = leaf_sp.to_absolute_path();
        let top_key = top_sp.to_absolute_path();
        assert_eq!(w.registry.get(&leaf_key).unwrap().state, GoalState::Ready);
        assert!(matches!(w.registry.get(&top_key).unwrap().state, GoalState::Waiting { remaining_deps: 1 }));

        // Simulate completing the leaf.
        let outcome = crate::orchestrate::BuildOutcome {
            drv_path: leaf_sp.clone(),
            outputs: std::collections::BTreeMap::new(),
            substitutions: std::collections::BTreeMap::new(),
            cached: false,
            log: None,
        };
        let mut outcomes = Vec::new();
        let mut failed = Vec::new();
        w.complete_goal(&leaf_key, outcome, &mut outcomes, &mut failed).unwrap();

        // Leaf should be Done.
        assert_eq!(w.registry.get(&leaf_key).unwrap().state, GoalState::Done);
        assert!(w.registry.get(&leaf_key).unwrap().waiters.is_empty());
        // Top should now be Ready (its sole dep completed).
        assert_eq!(w.registry.get(&top_key).unwrap().state, GoalState::Ready);
        // Top should be in the ready set.
        assert!(w.ready_goals.contains_key(&top_key));
        // Leaf is not a root, so outcomes should be empty.
        assert!(outcomes.is_empty());
    }

    #[test]
    fn complete_goal_collects_root_outcome() {
        let mut kp = DerivationRegistry::default();
        let drv = make_drv();
        let sp = register_drv(&mut kp, "root.drv", &drv);

        let mut w = Worker::new(1);
        w.want(&sp, &kp, true).unwrap();

        let key = sp.to_absolute_path();
        let outcome = crate::orchestrate::BuildOutcome {
            drv_path: sp.clone(),
            outputs: std::collections::BTreeMap::new(),
            substitutions: std::collections::BTreeMap::new(),
            cached: true,
            log: None,
        };
        let mut outcomes = Vec::new();
        let mut failed = Vec::new();
        w.complete_goal(&key, outcome, &mut outcomes, &mut failed).unwrap();

        assert_eq!(outcomes.len(), 1);
        assert!(outcomes[0].cached);
    }

    #[test]
    fn ready_goal_inputs_reuses_shared_derivation_arc() {
        let mut kp = DerivationRegistry::default();
        let drv = make_drv();
        let sp = register_drv(&mut kp, "root.drv", &drv);

        let mut w = Worker::new(1);
        w.want(&sp, &kp, true).unwrap();

        let key = sp.to_absolute_path();
        let registry_arc = kp.get_by_drv_path(&key).unwrap().derivation.clone();
        let goal_arc = w.registry.get(&key).unwrap().derivation.as_ref().unwrap().clone();
        let (_, _, ready_arc) = w.ready_goal_inputs(&key).unwrap();

        assert!(std::sync::Arc::ptr_eq(&registry_arc, &goal_arc));
        assert!(std::sync::Arc::ptr_eq(&goal_arc, &ready_arc));
    }

    #[test]
    fn complete_diamond_shared_dep_unblocks_both() {
        let mut kp = DerivationRegistry::default();

        let shared_drv = make_drv();
        let shared_sp = register_drv(&mut kp, "shared.drv", &shared_drv);

        let left_drv = make_drv_with_deps(std::slice::from_ref(&shared_sp));
        let left_sp = register_drv(&mut kp, "left.drv", &left_drv);

        let right_drv = make_drv_with_deps(std::slice::from_ref(&shared_sp));
        let right_sp = register_drv(&mut kp, "right.drv", &right_drv);

        let top_drv = make_drv_with_deps(&[left_sp.clone(), right_sp.clone()]);
        let top_sp = register_drv(&mut kp, "top.drv", &top_drv);

        let mut w = Worker::new(2);
        w.want(&top_sp, &kp, true).unwrap();

        // Complete shared dep.
        let shared_key = shared_sp.to_absolute_path();
        let outcome = crate::orchestrate::BuildOutcome {
            drv_path: shared_sp.clone(),
            outputs: std::collections::BTreeMap::new(),
            substitutions: std::collections::BTreeMap::new(),
            cached: false,
            log: None,
        };
        let mut outcomes = Vec::new();
        let mut failed = Vec::new();
        w.complete_goal(&shared_key, outcome, &mut outcomes, &mut failed).unwrap();

        // Both left and right should now be Ready.
        let left_key = left_sp.to_absolute_path();
        let right_key = right_sp.to_absolute_path();
        assert_eq!(w.registry.get(&left_key).unwrap().state, GoalState::Ready);
        assert_eq!(w.registry.get(&right_key).unwrap().state, GoalState::Ready);
        // Top still Waiting (needs left + right).
        let top_key = top_sp.to_absolute_path();
        assert!(matches!(w.registry.get(&top_key).unwrap().state, GoalState::Waiting { remaining_deps: 2 }));
    }

    // ── fail_goal / propagate_failure ─────────────────────────

    #[test]
    fn fail_goal_propagates_to_waiters() {
        let mut kp = DerivationRegistry::default();

        let leaf_drv = make_drv();
        let leaf_sp = register_drv(&mut kp, "leaf.drv", &leaf_drv);

        let mid_drv = make_drv_with_deps(std::slice::from_ref(&leaf_sp));
        let mid_sp = register_drv(&mut kp, "mid.drv", &mid_drv);

        let top_drv = make_drv_with_deps(std::slice::from_ref(&mid_sp));
        let top_sp = register_drv(&mut kp, "top.drv", &top_drv);

        let mut w = Worker::new(1);
        w.want(&top_sp, &kp, true).unwrap();

        // Leaf is Ready — simulate it being dispatched and starting.
        let leaf_key = leaf_sp.to_absolute_path();
        w.ready_goals.remove(&leaf_key); // remove leaf from ready set
        w.registry.get_mut(&leaf_key).unwrap().mark_building().unwrap();

        // Fail the leaf.
        let mut failed = Vec::new();
        w.fail_goal(&leaf_key, "leaf build error", &mut failed).unwrap();

        // Leaf should be Failed.
        assert_eq!(w.registry.get(&leaf_key).unwrap().state, GoalState::Failed);
        assert!(w.registry.get(&leaf_key).unwrap().waiters.is_empty());
        // Mid should be Failed (dep failed).
        let mid_key = mid_sp.to_absolute_path();
        assert_eq!(w.registry.get(&mid_key).unwrap().state, GoalState::Failed);
        assert!(w.registry.get(&mid_key).unwrap().waiters.is_empty());
        // Top should be Failed (transitive dep failed).
        let top_key = top_sp.to_absolute_path();
        assert_eq!(w.registry.get(&top_key).unwrap().state, GoalState::Failed);
        // Top is a root, so it should be in the failed list.
        assert!(failed.iter().any(|f| f.drv_key == top_key));
    }

    #[test]
    fn fail_goal_only_propagates_to_waiting() {
        let mut kp = DerivationRegistry::default();

        let shared_drv = make_drv();
        let shared_sp = register_drv(&mut kp, "shared.drv", &shared_drv);

        let good_drv = make_drv_with_deps(std::slice::from_ref(&shared_sp));
        let good_sp = register_drv(&mut kp, "good.drv", &good_drv);

        let bad_drv = make_drv();
        let bad_sp = register_drv(&mut kp, "bad.drv", &bad_drv);

        let mut w = Worker::new(2);
        w.want(&good_sp, &kp, true).unwrap();
        w.want(&bad_sp, &kp, true).unwrap();

        // bad is Ready, shared is Ready, good is Waiting.
        let bad_key = bad_sp.to_absolute_path();
        w.ready_goals.remove(&bad_key); // remove bad from ready set
        w.registry.get_mut(&bad_key).unwrap().mark_building().unwrap();

        // Fail bad. It has no waiters, so good (waiting on shared) should be unaffected.
        let mut failed = Vec::new();
        w.fail_goal(&bad_key, "bad build error", &mut failed).unwrap();

        assert_eq!(w.registry.get(&bad_key).unwrap().state, GoalState::Failed);
        // good is still Waiting on shared, not failed.
        let good_key = good_sp.to_absolute_path();
        assert!(matches!(w.registry.get(&good_key).unwrap().state, GoalState::Waiting { .. }));
        // bad is a root, so it should be in failed list.
        assert!(failed.iter().any(|f| f.drv_key == bad_key));
    }

    // ── run() integration tests ───────────────────────────────

    use std::path::PathBuf;

    use snix_castore::B3Digest;
    use snix_castore::Node;
    use snix_castore::blobservice::BlobService;
    use snix_castore::blobservice::MemoryBlobService;
    use snix_store::path_info::PathInfo;
    use tokio::io::AsyncWriteExt;

    use crate::dynamic_plan::MAX_DYNAMIC_PLAN_BYTES;
    use crate::orchestrate::Builder;
    use crate::test_support::MockBuildService;
    use crate::test_support::build_and_register;
    use crate::test_support::build_and_register_multi;
    use crate::test_support::test_keypair;
    use crate::test_support::test_pis;
    use crate::test_support::test_trusted_keys;
    use crate::test_support::tmp_ds;

    const TEST_NATIVE_PLAN_STORE_PATH: &str = "/nix/store/00000000000000000000000000000000-dynplan";
    const TEST_CHAIN_GOAL_COUNT: usize = 3;
    const TEST_CHAIN_LEAF_PATH_NODES: u32 = 3;
    const TEST_CHAIN_MID_PATH_NODES: u32 = 2;
    const TEST_SHARED_BLOCKED_ROOTS: u32 = 2;
    const TEST_PRIORITY_ROOT_OUTCOME_COUNT: usize = 3;
    const TEST_MAX_JOB_BUDGET: u32 = 2;
    const TEST_MAX_JOB_READY_ROOTS: usize = 3;
    const TEST_READY_ROOTS_AFTER_DISPATCH: usize = 1;
    const TEST_RUNNING_OVER_BUDGET: u32 = TEST_MAX_JOB_BUDGET + 1;

    fn make_test_builder(bs: MemoryBlobService) -> Builder<MockBuildService> {
        let ds = tmp_ds();
        let (mock, _) = MockBuildService::new(bs.clone());
        Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        )
    }

    async fn put_test_blob(bs: &MemoryBlobService, content: &[u8]) -> Node {
        let mut writer = BlobService::open_write(bs).await;
        writer.write_all(content).await.unwrap();
        let digest = writer.close().await.unwrap();
        Node::File {
            digest,
            size: content.len() as u64,
            executable: false,
        }
    }

    fn test_path_info(name: &str, node: Node) -> PathInfo {
        PathInfo {
            store_path: fake_sp(name),
            node,
            references: vec![],
            nar_size: 0,
            nar_sha256: [0u8; 32],
            signatures: vec![],
            deriver: None,
            ca: None,
        }
    }

    fn valid_native_plan_bytes() -> Vec<u8> {
        format!(
            r#"{{
  "schema": "mantle-plan-v1",
  "producer": {{ "logical_name": "producer", "goal_hint": null }},
  "sources": [],
  "units": [{{
    "id": "unit.main",
    "derivation": {{
      "name": "unit-main",
      "builder": "{TEST_NATIVE_PLAN_STORE_PATH}/bin/builder",
      "system": "x86_64-linux",
      "args": ["--build"],
      "outputs": ["out"],
      "env": {{}},
      "inputs": [],
      "fixed_output": null,
      "addressing_mode": "content-addressed",
      "sandbox": "native",
      "dynamic_plan_outputs": []
    }},
    "requested_outputs": ["out"],
    "policy": {{
      "sandbox": "inherit",
      "substitutions": "inherit",
      "store_prefix": "inherit",
      "host_paths": "none"
    }}
  }}],
  "roots": ["unit.main"],
  "provenance": {{}}
}}"#,
        )
        .into_bytes()
    }

    fn native_plan_with_dep_and_unused_bytes() -> Vec<u8> {
        format!(
            r#"{{
  "schema": "mantle-plan-v1",
  "producer": {{ "logical_name": "producer", "goal_hint": null }},
  "sources": [],
  "units": [
    {{
      "id": "unit.dep",
      "derivation": {{
        "name": "unit-dep",
        "builder": "{TEST_NATIVE_PLAN_STORE_PATH}/bin/builder",
        "system": "x86_64-linux",
        "args": ["--dep"],
        "outputs": ["out"],
        "env": {{}},
        "inputs": [],
        "fixed_output": null,
        "addressing_mode": "content-addressed",
        "sandbox": "native",
        "dynamic_plan_outputs": []
      }},
      "requested_outputs": ["out"],
      "policy": {{ "sandbox": "inherit", "substitutions": "inherit", "store_prefix": "inherit", "host_paths": "none" }}
    }},
    {{
      "id": "unit.root",
      "derivation": {{
        "name": "unit-root",
        "builder": "{TEST_NATIVE_PLAN_STORE_PATH}/bin/builder",
        "system": "x86_64-linux",
        "args": ["--root"],
        "outputs": ["out"],
        "env": {{}},
        "inputs": [{{ "kind": "unit_output", "unit": "unit.dep", "output": "out" }}],
        "fixed_output": null,
        "addressing_mode": "content-addressed",
        "sandbox": "native",
        "dynamic_plan_outputs": []
      }},
      "requested_outputs": ["out"],
      "policy": {{ "sandbox": "inherit", "substitutions": "inherit", "store_prefix": "inherit", "host_paths": "none" }}
    }},
    {{
      "id": "unit.unused",
      "derivation": {{
        "name": "unit-unused",
        "builder": "{TEST_NATIVE_PLAN_STORE_PATH}/bin/builder",
        "system": "x86_64-linux",
        "args": ["--unused"],
        "outputs": ["out"],
        "env": {{}},
        "inputs": [],
        "fixed_output": null,
        "addressing_mode": "content-addressed",
        "sandbox": "native",
        "dynamic_plan_outputs": []
      }},
      "requested_outputs": ["out"],
      "policy": {{ "sandbox": "inherit", "substitutions": "inherit", "store_prefix": "inherit", "host_paths": "none" }}
    }}
  ],
  "roots": ["unit.root"],
  "provenance": {{}}
}}"#,
        )
        .into_bytes()
    }

    fn declare_native_plan_output(kp: &mut DerivationRegistry, sp: &StorePath<String>, output_name: &str) {
        let abs = sp.to_absolute_path();
        let entry = kp.get_by_drv_path_mut(&abs).unwrap();
        entry.dynamic_plan_outputs = vec![output_name.to_string()];
    }

    #[tokio::test]
    async fn dispatch_at_capacity_retains_undispatched_ready_goal() {
        let bs = MemoryBlobService::default();
        let mut builder = make_test_builder(bs);
        let mut known_paths = DerivationRegistry::default();
        let roots: Vec<StorePath<String>> = (0..TEST_MAX_JOB_READY_ROOTS)
            .map(|index| build_and_register(&format!("capacity-{index}"), &[], &mut known_paths).0)
            .collect();
        let mut worker = Worker::new(TEST_MAX_JOB_BUDGET);
        for root in &roots {
            worker.want(root, &known_paths, true).unwrap();
        }
        let mut outcomes = Vec::new();
        let mut failed = Vec::new();
        let mut native_dynamic_plans = Vec::new();
        let mut state = WorkerLoopState {
            sem: Arc::new(Semaphore::new(usize::try_from(TEST_MAX_JOB_BUDGET).unwrap())),
            join_set: JoinSet::new(),
            pending_meta: HashMap::new(),
            outcomes: &mut outcomes,
            failed: &mut failed,
            native_dynamic_plans: &mut native_dynamic_plans,
            completed_count: 0,
        };

        let dispatched = worker.dispatch_ready(&mut builder, &mut known_paths, &mut state).await.unwrap();

        assert_eq!(dispatched, TEST_MAX_JOB_BUDGET);
        assert_eq!(state.join_set.len(), usize::try_from(TEST_MAX_JOB_BUDGET).unwrap());
        assert_eq!(worker.ready_goals.len(), TEST_READY_ROOTS_AFTER_DISPATCH);
        state.join_set.abort_all();
        for _ in 0..TEST_MAX_JOB_BUDGET {
            let _ = state.join_set.join_next().await;
        }
        assert!(state.join_set.is_empty());
    }

    #[tokio::test]
    async fn dispatch_rejects_running_count_over_budget_before_ready_mutation() {
        let bs = MemoryBlobService::default();
        let mut builder = make_test_builder(bs);
        let mut known_paths = DerivationRegistry::default();
        let (root, _) = build_and_register("over-capacity", &[], &mut known_paths);
        let mut worker = Worker::new(TEST_MAX_JOB_BUDGET);
        worker.want(&root, &known_paths, true).unwrap();
        let ready_before = worker.ready_goals.clone();
        let mut outcomes = Vec::new();
        let mut failed = Vec::new();
        let mut native_dynamic_plans = Vec::new();
        let mut state = WorkerLoopState {
            sem: Arc::new(Semaphore::new(usize::try_from(TEST_MAX_JOB_BUDGET).unwrap())),
            join_set: JoinSet::new(),
            pending_meta: HashMap::new(),
            outcomes: &mut outcomes,
            failed: &mut failed,
            native_dynamic_plans: &mut native_dynamic_plans,
            completed_count: 0,
        };
        for _ in 0..TEST_RUNNING_OVER_BUDGET {
            state.join_set.spawn(async { std::future::pending().await });
        }

        let result = worker.dispatch_ready(&mut builder, &mut known_paths, &mut state).await;

        assert!(result.unwrap_err().to_string().contains("exceeds configured max_jobs"));
        assert_eq!(worker.ready_goals, ready_before);
        assert!(worker.priority_decisions.is_empty());
        state.join_set.abort_all();
        while state.join_set.join_next().await.is_some() {}
        assert!(state.join_set.is_empty());
    }

    #[tokio::test]
    async fn run_single_leaf_builds_once() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (sp, _) = build_and_register("solo", &[], &mut kp);

        let mut w = Worker::new(1);
        w.want(&sp, &kp, true).unwrap();
        let result = w.run(&mut builder, &mut kp).await.unwrap();

        assert_eq!(result.outcomes.len(), 1);
        assert!(result.failed.is_empty());
        assert!(!result.outcomes[0].cached);

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 1);
    }

    #[tokio::test]
    async fn run_chain_builds_in_order() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (dep_path, _) = build_and_register("dep", &[], &mut kp);
        let (mid_path, _) = build_and_register("mid", &[(dep_path.clone(), "out")], &mut kp);
        let (top_path, _) = build_and_register("top", &[(mid_path.clone(), "out")], &mut kp);

        let mut w = Worker::new(1);
        w.want(&top_path, &kp, true).unwrap();
        let result = w.run(&mut builder, &mut kp).await.unwrap();

        assert_eq!(result.outcomes.len(), 1);
        assert!(result.outcomes[0].drv_path.name().contains("top"));
        assert_eq!(result.priority_decisions.len(), TEST_CHAIN_GOAL_COUNT);
        let path_nodes: Vec<u32> =
            result.priority_decisions.iter().map(|decision| decision.known_critical_path_nodes).collect();
        assert_eq!(path_nodes, vec![TEST_CHAIN_LEAF_PATH_NODES, TEST_CHAIN_MID_PATH_NODES, 1]);

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 3);

        // Verify ordering: dep before mid before top.
        let pos_dep = recorded.iter().position(|a| a.iter().any(|s| s.contains("dep"))).unwrap();
        let pos_mid = recorded.iter().position(|a| a.iter().any(|s| s.contains("mid"))).unwrap();
        let pos_top = recorded.iter().position(|a| a.iter().any(|s| s.contains("top"))).unwrap();
        assert!(pos_dep < pos_mid, "dep before mid");
        assert!(pos_mid < pos_top, "mid before top");
    }

    #[tokio::test]
    async fn run_diamond_builds_shared_once() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (shared, _) = build_and_register("shared", &[], &mut kp);
        let (left, _) = build_and_register("left", &[(shared.clone(), "out")], &mut kp);
        let (right, _) = build_and_register("right", &[(shared.clone(), "out")], &mut kp);

        let mut w = Worker::new(2);
        w.want(&left, &kp, true).unwrap();
        w.want(&right, &kp, true).unwrap();
        let result = w.run(&mut builder, &mut kp).await.unwrap();

        assert_eq!(result.outcomes.len(), 2);
        assert_eq!(result.priority_decisions.len(), TEST_CHAIN_GOAL_COUNT);
        assert_eq!(result.priority_decisions[0].blocked_root_count, TEST_SHARED_BLOCKED_ROOTS);
        assert_eq!(
            result.priority_decisions[0].selection_reason,
            crate::scheduling::PrioritySelectionReason::SoleCandidate
        );

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 3, "shared + left + right");
        let shared_builds = recorded.iter().filter(|a| a.iter().any(|s| s.contains("shared"))).count();
        assert_eq!(shared_builds, 1, "shared built exactly once");
    }

    // r[verify build_scheduling.lazy_known_critical_path]
    #[tokio::test]
    async fn worker_selects_shared_root_pressure_before_fifo_insertion() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());
        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );
        let mut kp = DerivationRegistry::default();
        let (short, _) = build_and_register("fifo-first-short", &[], &mut kp);
        let (shared, _) = build_and_register("pressure-shared", &[], &mut kp);
        let (left, _) = build_and_register("pressure-left", &[(shared.clone(), "out")], &mut kp);
        let (right, _) = build_and_register("pressure-right", &[(shared.clone(), "out")], &mut kp);

        let mut worker = Worker::new(1);
        worker.want(&short, &kp, true).unwrap();
        worker.want(&left, &kp, true).unwrap();
        worker.want(&right, &kp, true).unwrap();
        let result = worker.run(&mut builder, &mut kp).await.unwrap();
        let recorded = calls.lock().unwrap();

        assert_eq!(result.outcomes.len(), TEST_PRIORITY_ROOT_OUTCOME_COUNT);
        assert!(recorded[0].iter().any(|argument| argument.contains("pressure-shared")));
        assert_eq!(result.priority_decisions[0].blocked_root_count, TEST_SHARED_BLOCKED_ROOTS);
        assert_eq!(
            result.priority_decisions[0].selection_reason,
            crate::scheduling::PrioritySelectionReason::KnownGraphPressure
        );
    }

    // r[verify build_scheduling.lazy_known_critical_path]
    #[test]
    fn later_graph_arrival_does_not_relabel_prior_priority_evidence() {
        let mut kp = DerivationRegistry::default();
        let (first, _) = build_and_register("first-root", &[], &mut kp);
        let (later_leaf, _) = build_and_register("later-leaf", &[], &mut kp);
        let (later_root, _) = build_and_register("later-root", &[(later_leaf, "out")], &mut kp);
        let mut worker = Worker::new(1);
        worker.want(&first, &kp, true).unwrap();
        let selected = worker.select_next_ready_goal().unwrap();
        let prior = worker.priority_decisions[0].clone();

        worker.want(&later_root, &kp, true).unwrap();

        assert_eq!(selected, Some(first.to_absolute_path()));
        assert_eq!(worker.priority_decisions[0], prior);
        assert_eq!(worker.priority_decisions[0].competing_goal_count, 1);
    }

    #[test]
    fn worker_consumes_only_normalized_eligible_preference_facts() {
        let mut known_paths = DerivationRegistry::default();
        let (unknown, _) = build_and_register("unknown-route", &[], &mut known_paths);
        let (local, _) = build_and_register("local-route", &[], &mut known_paths);
        let local_key = local.to_absolute_path();
        let mut worker = Worker::new(1);
        worker.want(&unknown, &known_paths, true).unwrap();
        worker.want(&local, &known_paths, true).unwrap();
        let preference = crate::scheduling::normalize_eligible_preference(
            crate::scheduling::HardEligibilityFacts::ELIGIBLE,
            crate::scheduling::ResourceFitClass::Exact,
            crate::scheduling::ContentLocalityClass::FullyPresent,
            crate::scheduling::TransferCostClass::None,
        )
        .unwrap();
        worker
            .set_goal_scheduling_preference(&local_key, OperatorPolicyClass::Ordinary, preference)
            .unwrap();

        let selected = worker.select_next_ready_goal().unwrap();

        assert_eq!(selected, Some(local_key));
        assert_eq!(
            worker.priority_decisions[0].selection_reason,
            crate::scheduling::PrioritySelectionReason::ResourceFitClass
        );
    }

    #[test]
    fn malformed_graph_rejects_before_ready_set_mutation() {
        let mut kp = DerivationRegistry::default();
        let (root, _) = build_and_register("malformed-root", &[], &mut kp);
        let root_key = root.to_absolute_path();
        let mut worker = Worker::new(1);
        worker.want(&root, &kp, true).unwrap();
        worker
            .registry
            .get_mut(&root_key)
            .unwrap()
            .waitees
            .push("/nix/store/missing-malformed.drv".to_string());
        let ready_before = worker.ready_goals.clone();

        let error = worker.select_next_ready_goal().unwrap_err();

        assert!(error.to_string().contains("unknown goal"));
        assert_eq!(worker.ready_goals, ready_before);
        assert_eq!(worker.scheduling_epoch, 0);
        assert!(worker.priority_decisions.is_empty());
    }

    #[tokio::test]
    async fn run_disjoint_trees() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (a, _) = build_and_register("tree1-leaf", &[], &mut kp);
        let (b, _) = build_and_register("tree1-root", &[(a.clone(), "out")], &mut kp);
        let (c, _) = build_and_register("tree2-leaf", &[], &mut kp);
        let (d, _) = build_and_register("tree2-root", &[(c.clone(), "out")], &mut kp);

        let mut w = Worker::new(2);
        w.want(&b, &kp, true).unwrap();
        w.want(&d, &kp, true).unwrap();
        let result = w.run(&mut builder, &mut kp).await.unwrap();

        assert_eq!(result.outcomes.len(), 2);

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 4);

        // Each leaf before its root.
        let pos_a = recorded.iter().position(|a| a.iter().any(|s| s.contains("tree1-leaf"))).unwrap();
        let pos_b = recorded.iter().position(|a| a.iter().any(|s| s.contains("tree1-root"))).unwrap();
        let pos_c = recorded.iter().position(|a| a.iter().any(|s| s.contains("tree2-leaf"))).unwrap();
        let pos_d = recorded.iter().position(|a| a.iter().any(|s| s.contains("tree2-root"))).unwrap();
        assert!(pos_a < pos_b, "tree1-leaf before tree1-root");
        assert!(pos_c < pos_d, "tree2-leaf before tree2-root");
    }

    #[tokio::test]
    #[should_panic(expected = "no root goals to build")]
    async fn run_empty_worker_panics() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, _) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let mut w = Worker::new(1);

        // No roots — debug_assert catches this.
        let _ = w.run(&mut builder, &mut kp).await;
    }

    // ── run_streaming() tests ───────────────────────────────

    #[tokio::test]
    async fn streaming_single_root() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (sp, _) = build_and_register("solo", &[], &mut kp);

        let (tx, mut rx) = mpsc::channel::<EvalMessage>(16);
        tx.send(EvalMessage {
            label: "solo".into(),
            drv_path: sp.clone(),
            new_entries: vec![],
        })
        .await
        .unwrap();
        drop(tx); // Close channel — eval done.

        let mut w = Worker::new(1);
        let result = w.run_streaming(&mut builder, &mut kp, &mut rx).await.unwrap();

        assert_eq!(result.outcomes.len(), 1);
        assert!(result.failed.is_empty());
        assert_eq!(result.priority_decisions.len(), 1);
        assert_eq!(result.priority_decisions[0].scheduling_epoch, 1);

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 1);
    }

    #[tokio::test]
    async fn streaming_multiple_roots_arrive_incrementally() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (a, _) = build_and_register("pkg-a", &[], &mut kp);
        let (b, _) = build_and_register("pkg-b", &[], &mut kp);

        let (tx, mut rx) = mpsc::channel::<EvalMessage>(16);

        // Send first root.
        tx.send(EvalMessage {
            label: "pkg-a".into(),
            drv_path: a.clone(),
            new_entries: vec![],
        })
        .await
        .unwrap();
        // Send second root.
        tx.send(EvalMessage {
            label: "pkg-b".into(),
            drv_path: b.clone(),
            new_entries: vec![],
        })
        .await
        .unwrap();
        drop(tx);

        let mut w = Worker::new(2);
        let result = w.run_streaming(&mut builder, &mut kp, &mut rx).await.unwrap();

        assert_eq!(result.outcomes.len(), 2);

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 2);
    }

    #[tokio::test]
    async fn streaming_chain_with_shared_deps() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (shared, _) = build_and_register("shared", &[], &mut kp);
        let (top_a, _) = build_and_register("top-a", &[(shared.clone(), "out")], &mut kp);
        let (top_b, _) = build_and_register("top-b", &[(shared.clone(), "out")], &mut kp);

        let (tx, mut rx) = mpsc::channel::<EvalMessage>(16);
        tx.send(EvalMessage {
            label: "top-a".into(),
            drv_path: top_a.clone(),
            new_entries: vec![],
        })
        .await
        .unwrap();
        tx.send(EvalMessage {
            label: "top-b".into(),
            drv_path: top_b.clone(),
            new_entries: vec![],
        })
        .await
        .unwrap();
        drop(tx);

        let mut w = Worker::new(2);
        let result = w.run_streaming(&mut builder, &mut kp, &mut rx).await.unwrap();

        assert_eq!(result.outcomes.len(), 2);

        let recorded = calls.lock().unwrap();
        // shared + top-a + top-b = 3 builds (shared built once).
        assert_eq!(recorded.len(), 3);
        let shared_builds = recorded.iter().filter(|a| a.iter().any(|s| s.contains("shared"))).count();
        assert_eq!(shared_builds, 1);
    }

    #[tokio::test]
    async fn streaming_empty_channel_returns_empty() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, _) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (_tx, mut rx) = mpsc::channel::<EvalMessage>(16);
        drop(_tx); // Immediately close.

        let mut w = Worker::new(1);
        let result = w.run_streaming(&mut builder, &mut kp, &mut rx).await.unwrap();

        assert!(result.outcomes.is_empty());
        assert!(result.failed.is_empty());
    }

    #[tokio::test]
    async fn streaming_builds_start_before_channel_closes() {
        // Verify that builds begin while eval is still sending.
        // We send one root, let it build, then send another.
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (a, _) = build_and_register("first", &[], &mut kp);
        let (b, _) = build_and_register("second", &[], &mut kp);

        let (tx, mut rx) = mpsc::channel::<EvalMessage>(16);

        // Send first root only.
        tx.send(EvalMessage {
            label: "first".into(),
            drv_path: a.clone(),
            new_entries: vec![],
        })
        .await
        .unwrap();

        // Spawn a task that sends second root after a short delay
        // (simulating slow eval).
        let tx2 = tx.clone();
        let b2 = b.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            tx2.send(EvalMessage {
                label: "second".into(),
                drv_path: b2,
                new_entries: vec![],
            })
            .await
            .unwrap();
            // Drop tx2 but tx is still alive — don't close channel yet.
        });

        // Drop the original sender after a bit more delay.
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            drop(tx);
        });

        let mut w = Worker::new(2);
        let result = w.run_streaming(&mut builder, &mut kp, &mut rx).await.unwrap();

        assert_eq!(result.outcomes.len(), 2);

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 2);
    }

    // ── Entries-in-message tests ─────────────────────────────
    // These verify the eval/build streaming overlap: registry
    // entries arrive WITH the EvalMessage and are inserted by
    // accept_eval_message before want() is called.

    #[tokio::test]
    async fn streaming_entries_populate_registry_on_arrival() {
        // Registry starts EMPTY. Entries arrive via EvalMessage.
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        // Build the derivation entries outside the registry.
        let mut scratch_kp = DerivationRegistry::default();
        let (sp, drv) = build_and_register("via-msg", &[], &mut scratch_kp);
        let hdm = scratch_kp.get_hdm_by_drv_path(&sp.to_absolute_path()).unwrap();

        // Empty registry — the Worker has no knowledge of "via-msg" yet.
        let mut kp = DerivationRegistry::default();

        let (tx, mut rx) = mpsc::channel::<EvalMessage>(16);
        tx.send(EvalMessage {
            label: "via-msg".into(),
            drv_path: sp.clone(),
            new_entries: vec![(sp.clone(), hdm, drv, false, vec![], None)],
        })
        .await
        .unwrap();
        drop(tx);

        let mut w = Worker::new(1);
        let result = w.run_streaming(&mut builder, &mut kp, &mut rx).await.unwrap();

        // Registry was populated by the message.
        assert!(!kp.is_empty(), "registry should have entries from message");
        assert_eq!(result.outcomes.len(), 1);
        assert!(result.failed.is_empty());

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 1);
    }

    #[tokio::test]
    async fn streaming_entries_with_deps_arrive_incrementally() {
        // Two roots with a shared dep. First message carries
        // [shared, root_a]; second carries [root_b] (shared already
        // in registry from first message).
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        // Build derivation entries in a scratch registry.
        let mut scratch = DerivationRegistry::default();
        let (shared_sp, shared_drv) = build_and_register("shared", &[], &mut scratch);
        let (a_sp, a_drv) = build_and_register("root-a", &[(shared_sp.clone(), "out")], &mut scratch);
        let (b_sp, b_drv) = build_and_register("root-b", &[(shared_sp.clone(), "out")], &mut scratch);

        let shared_hdm = scratch.get_hdm_by_drv_path(&shared_sp.to_absolute_path()).unwrap();
        let a_hdm = scratch.get_hdm_by_drv_path(&a_sp.to_absolute_path()).unwrap();
        let b_hdm = scratch.get_hdm_by_drv_path(&b_sp.to_absolute_path()).unwrap();

        let mut kp = DerivationRegistry::default();

        let (tx, mut rx) = mpsc::channel::<EvalMessage>(16);
        // First root brings shared + root-a.
        tx.send(EvalMessage {
            label: "root-a".into(),
            drv_path: a_sp.clone(),
            new_entries: vec![
                (shared_sp.clone(), shared_hdm, shared_drv, false, vec![], None),
                (a_sp.clone(), a_hdm, a_drv, false, vec![], None),
            ],
        })
        .await
        .unwrap();
        // Second root only brings root-b (shared already known).
        tx.send(EvalMessage {
            label: "root-b".into(),
            drv_path: b_sp.clone(),
            new_entries: vec![(b_sp.clone(), b_hdm, b_drv, false, vec![], None)],
        })
        .await
        .unwrap();
        drop(tx);

        let mut w = Worker::new(2);
        let result = w.run_streaming(&mut builder, &mut kp, &mut rx).await.unwrap();

        assert_eq!(result.outcomes.len(), 2);
        assert!(result.failed.is_empty());

        // shared + root-a + root-b = 3 builds, shared only once.
        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 3);
        let shared_builds = recorded.iter().filter(|a| a.iter().any(|s| s.contains("shared"))).count();
        assert_eq!(shared_builds, 1);
    }

    // ── Native dynamic plan scanning tests ─────────────────

    #[tokio::test]
    async fn native_dynamic_plan_declared_output_is_accepted() {
        let bs = MemoryBlobService::default();
        let builder = make_test_builder(bs.clone());
        let plan_node = put_test_blob(&bs, &valid_native_plan_bytes()).await;

        let mut kp = DerivationRegistry::default();
        let (producer_sp, _) = build_and_register_multi("native-producer", &["out", "plan"], &[], &mut kp);
        declare_native_plan_output(&mut kp, &producer_sp, "plan");

        let mut outputs = BTreeMap::new();
        outputs.insert("plan".to_string(), test_path_info("native-plan", plan_node));
        let outcome = BuildOutcome {
            drv_path: producer_sp.clone(),
            outputs,
            substitutions: BTreeMap::new(),
            cached: false,
            log: None,
        };

        let worker = Worker::new(1);
        let declared = worker.declared_native_dynamic_outputs(&outcome, &kp);
        let scan = worker
            .scan_native_dynamic_plans(&producer_sp.to_absolute_path(), &outcome, &builder, kp.store_dir(), &declared)
            .await
            .unwrap();

        assert_eq!(declared, BTreeSet::from(["plan".to_string()]));
        assert_eq!(scan.accepted.len(), 1);
        assert!(scan.rejected.is_empty());
        assert_eq!(scan.accepted[0].output_name, "plan");
        assert_eq!(scan.accepted[0].accepted_unit_ids, vec!["unit.main".to_string()]);
        assert_eq!(scan.accepted[0].canonical_plan_digest.len(), crate::dynamic_plan::BLAKE3_HEX_BYTES);
    }

    #[tokio::test]
    async fn native_dynamic_plan_undeclared_output_is_ignored() {
        let bs = MemoryBlobService::default();
        let builder = make_test_builder(bs.clone());
        let plan_node = put_test_blob(&bs, &valid_native_plan_bytes()).await;

        let mut kp = DerivationRegistry::default();
        let (producer_sp, _) = build_and_register_multi("native-ignored", &["out", "plan"], &[], &mut kp);

        let mut outputs = BTreeMap::new();
        outputs.insert("plan".to_string(), test_path_info("ignored-plan", plan_node));
        let outcome = BuildOutcome {
            drv_path: producer_sp.clone(),
            outputs,
            substitutions: BTreeMap::new(),
            cached: false,
            log: None,
        };

        let worker = Worker::new(1);
        let declared = worker.declared_native_dynamic_outputs(&outcome, &kp);
        let scan = worker
            .scan_native_dynamic_plans(&producer_sp.to_absolute_path(), &outcome, &builder, kp.store_dir(), &declared)
            .await
            .unwrap();

        assert!(declared.is_empty());
        assert!(scan.is_empty());
    }

    #[tokio::test]
    async fn native_dynamic_plan_declared_output_rejections_are_structured() {
        let bs = MemoryBlobService::default();
        let builder = make_test_builder(bs.clone());
        let bad_node = put_test_blob(&bs, b"{not-json").await;
        let large_node = Node::File {
            digest: B3Digest::from(&[0u8; 32]),
            size: MAX_DYNAMIC_PLAN_BYTES + 1,
            executable: false,
        };
        let dir_node = Node::Directory {
            digest: B3Digest::from(&[1u8; 32]),
            size: 0,
        };

        let mut kp = DerivationRegistry::default();
        let output_names = &["out", "missing", "dir", "large", "bad"];
        let (producer_sp, _) = build_and_register_multi("native-reject", output_names, &[], &mut kp);
        let producer_abs = producer_sp.to_absolute_path();
        kp.get_by_drv_path_mut(&producer_abs).unwrap().dynamic_plan_outputs = vec![
            "missing".to_string(),
            "dir".to_string(),
            "large".to_string(),
            "bad".to_string(),
        ];

        let mut outputs = BTreeMap::new();
        outputs.insert("dir".to_string(), test_path_info("dir-plan", dir_node));
        outputs.insert("large".to_string(), test_path_info("large-plan", large_node));
        outputs.insert("bad".to_string(), test_path_info("bad-plan", bad_node));
        let outcome = BuildOutcome {
            drv_path: producer_sp.clone(),
            outputs,
            substitutions: BTreeMap::new(),
            cached: false,
            log: None,
        };

        let worker = Worker::new(1);
        let declared = worker.declared_native_dynamic_outputs(&outcome, &kp);
        let scan = worker
            .scan_native_dynamic_plans(&producer_abs, &outcome, &builder, kp.store_dir(), &declared)
            .await
            .unwrap();
        let by_output: BTreeMap<String, &NativeDynamicPlanRejection> =
            scan.rejected.iter().map(|rejection| (rejection.output_name.clone(), rejection)).collect();

        assert!(scan.accepted.is_empty());
        assert_eq!(by_output.len(), 4);
        assert_eq!(by_output["missing"].kind, NativeDynamicPlanRejectionKind::MissingOutput);
        assert_eq!(by_output["dir"].kind, NativeDynamicPlanRejectionKind::NonRegularOutput);
        assert_eq!(by_output["large"].kind, NativeDynamicPlanRejectionKind::PlanTooLarge);
        assert_eq!(by_output["bad"].kind, NativeDynamicPlanRejectionKind::InvalidPlan);
        assert!(by_output["bad"].raw_artifact_digest.is_some());
        assert!(by_output["large"].raw_artifact_digest.is_none());
        assert!(by_output.values().all(|rejection| rejection.producer_key == producer_abs));
    }

    fn native_report_row(producer_key: &str, output_name: &str) -> NativeDynamicPlanReport {
        NativeDynamicPlanReport {
            mode: "native".to_string(),
            producer_key: producer_key.to_string(),
            output_name: output_name.to_string(),
            plan_artifact_path: None,
            raw_artifact_digest: None,
            canonical_plan_digest: None,
            accepted_unit_ids: Vec::new(),
            rejection_reason: None,
            scheduler_action: "registered-roots".to_string(),
        }
    }

    #[test]
    fn native_dynamic_plan_reports_are_deterministically_sorted() {
        let result = finish_worker_result(
            Vec::new(),
            Vec::new(),
            vec![
                native_report_row("/store/z.drv", "plan-b"),
                native_report_row("/store/a.drv", "plan-a"),
            ],
            Vec::new(),
        );

        assert_eq!(result.native_dynamic_plans[0].producer_key, "/store/a.drv");
        assert_eq!(result.native_dynamic_plans[0].output_name, "plan-a");
        assert_eq!(result.native_dynamic_plans[1].producer_key, "/store/z.drv");
        assert_eq!(result.native_dynamic_plans[1].output_name, "plan-b");
    }

    #[tokio::test]
    async fn native_dynamic_plan_valid_output_schedules_root_unit_in_same_run() {
        let bs = MemoryBlobService::default();
        let mut drv_outputs = StdHashMap::new();
        drv_outputs.insert("native-scheduler".to_string(), valid_native_plan_bytes());
        let (mock, calls) = DrvProducingMockBuildService::new(bs.clone(), drv_outputs);
        let mut builder = Builder::new(
            bs,
            tmp_ds(),
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (producer_sp, _) = build_and_register_multi("native-scheduler", &["out", "plan"], &[], &mut kp);
        declare_native_plan_output(&mut kp, &producer_sp, "plan");

        let mut worker = Worker::new(1);
        worker.want(&producer_sp, &kp, true).unwrap();
        let result = worker.run(&mut builder, &mut kp).await.unwrap();
        let recorded = calls.lock().unwrap();

        assert!(result.failed.is_empty());
        assert_eq!(result.native_dynamic_plans.len(), 1);
        assert_eq!(result.native_dynamic_plans[0].mode, "native");
        assert_eq!(result.native_dynamic_plans[0].output_name, "plan");
        assert_eq!(result.native_dynamic_plans[0].scheduler_action, "registered-roots");
        assert_eq!(result.native_dynamic_plans[0].accepted_unit_ids, vec!["unit.main".to_string()]);
        assert!(result.native_dynamic_plans[0].canonical_plan_digest.is_some());
        assert!(result.native_dynamic_plans[0].rejection_reason.is_none());
        assert_eq!(recorded.len(), 2, "producer and one dynamic root should build: {recorded:?}");
        assert!(recorded.iter().any(|args| args.iter().any(|arg| arg.contains("native-scheduler"))));
        assert!(recorded.iter().any(|args| args.iter().any(|arg| arg == "--build")));
    }

    #[tokio::test]
    async fn native_dynamic_plan_rejected_output_schedules_no_units() {
        let bs = MemoryBlobService::default();
        let mut drv_outputs = StdHashMap::new();
        drv_outputs.insert("native-rejected".to_string(), b"{not-json".to_vec());
        let (mock, calls) = DrvProducingMockBuildService::new(bs.clone(), drv_outputs);
        let mut builder = Builder::new(
            bs,
            tmp_ds(),
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (producer_sp, _) = build_and_register_multi("native-rejected", &["out", "plan"], &[], &mut kp);
        declare_native_plan_output(&mut kp, &producer_sp, "plan");

        let mut worker = Worker::new(1);
        worker.want(&producer_sp, &kp, true).unwrap();
        let result = worker.run(&mut builder, &mut kp).await.unwrap();
        let recorded = calls.lock().unwrap();

        assert!(result.failed.is_empty());
        assert_eq!(result.native_dynamic_plans.len(), 1);
        assert_eq!(result.native_dynamic_plans[0].mode, "native");
        assert_eq!(result.native_dynamic_plans[0].scheduler_action, "rejected");
        assert!(result.native_dynamic_plans[0].accepted_unit_ids.is_empty());
        assert!(result.native_dynamic_plans[0].rejection_reason.as_deref().unwrap_or("").contains("invalid-plan"));
        assert_eq!(recorded.len(), 1, "rejected native plan must not schedule units: {recorded:?}");
        assert!(recorded[0].iter().any(|arg| arg.contains("native-rejected")));
    }

    #[tokio::test]
    async fn native_dynamic_plan_registers_all_units_but_wants_only_roots() {
        let bs = MemoryBlobService::default();
        let builder = make_test_builder(bs.clone());
        let plan_node = put_test_blob(&bs, &native_plan_with_dep_and_unused_bytes()).await;

        let mut kp = DerivationRegistry::default();
        let (producer_sp, _) = build_and_register_multi("native-registration", &["out", "plan"], &[], &mut kp);
        declare_native_plan_output(&mut kp, &producer_sp, "plan");

        let mut outputs = BTreeMap::new();
        outputs.insert("plan".to_string(), test_path_info("native-registration-plan", plan_node));
        let outcome = BuildOutcome {
            drv_path: producer_sp.clone(),
            outputs,
            substitutions: BTreeMap::new(),
            cached: false,
            log: None,
        };
        let worker = Worker::new(1);
        let declared = worker.declared_native_dynamic_outputs(&outcome, &kp);
        let scan = worker
            .scan_native_dynamic_plans(&producer_sp.to_absolute_path(), &outcome, &builder, kp.store_dir(), &declared)
            .await
            .unwrap();
        let registered = register_native_dynamic_plan_units(&scan.accepted[0], &mut kp).unwrap();

        let mut dynamic_worker = Worker::new(1);
        for root in &registered.root_drv_paths {
            dynamic_worker.want(root, &kp, true).unwrap();
        }
        let mut dynamic_builder = make_test_builder(bs);
        let result = dynamic_worker.run(&mut dynamic_builder, &mut kp).await.unwrap();

        assert!(registered.unit_drv_paths.contains_key("unit.dep"));
        assert!(registered.unit_drv_paths.contains_key("unit.root"));
        assert!(registered.unit_drv_paths.contains_key("unit.unused"));
        assert_eq!(registered.root_drv_paths.len(), 1);
        assert!(result.failed.is_empty());
        assert_eq!(result.outcomes.len(), 1);
        assert!(dynamic_worker.registry().get(&registered.unit_drv_paths["unit.dep"].to_absolute_path()).is_some());
        assert!(dynamic_worker.registry().get(&registered.unit_drv_paths["unit.root"].to_absolute_path()).is_some());
        assert!(
            dynamic_worker
                .registry()
                .get(&registered.unit_drv_paths["unit.unused"].to_absolute_path())
                .is_none()
        );
    }

    #[tokio::test]
    async fn declared_native_dynamic_output_is_not_compat_drv_discovery() {
        let bs = MemoryBlobService::default();
        let builder = make_test_builder(bs.clone());
        let mut inner_kp = DerivationRegistry::default();
        let (_, inner_drv) = build_and_register("native-skip-inner", &[], &mut inner_kp);
        let drv_node = put_test_blob(&bs, &inner_drv.to_aterm_bytes()).await;

        let mut kp = DerivationRegistry::default();
        let (producer_sp, _) = build_and_register_multi("native-skip-compat", &["out", "plan"], &[], &mut kp);
        declare_native_plan_output(&mut kp, &producer_sp, "plan");

        let mut outputs = BTreeMap::new();
        outputs.insert("plan".to_string(), test_path_info("declared-plan.drv", drv_node));
        let outcome = BuildOutcome {
            drv_path: producer_sp.clone(),
            outputs,
            substitutions: BTreeMap::new(),
            cached: false,
            log: None,
        };

        let worker = Worker::new(1);
        let declared = worker.declared_native_dynamic_outputs(&outcome, &kp);
        let discovered = worker.scan_dynamic_derivations(&outcome, &builder, &mut kp, &declared).await.unwrap();

        assert_eq!(declared, BTreeSet::from(["plan".to_string()]));
        assert!(discovered.is_empty(), "declared native output must not fall through to .drv compatibility");
    }

    // ── Dynamic derivation integration tests ─────────────────

    use std::collections::HashMap as StdHashMap;

    use crate::test_support::DrvProducingMockBuildService;

    /// Build a derivation that produces `.drv` ATerm output.
    /// The Worker should detect it, parse the inner .drv, register
    /// it, and build it automatically.
    #[tokio::test]
    async fn dynamic_drv_detected_and_built() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();

        // Create the "inner" derivation that will be discovered dynamically.
        let mut inner_kp = DerivationRegistry::default();
        let (_, inner_drv) = build_and_register("inner-hello", &[], &mut inner_kp);
        let inner_aterm = inner_drv.to_aterm_bytes();

        // The producer build will output this ATerm content.
        let mut drv_outputs = StdHashMap::new();
        // Match on the producer name substring.
        drv_outputs.insert("producer-gen.drv".to_string(), inner_aterm.clone());

        let (mock, calls) = DrvProducingMockBuildService::new(bs.clone(), drv_outputs);

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        // Register the producer derivation. Its output path name must
        // end in .drv for detection to work.
        let mut kp = DerivationRegistry::default();
        let (producer_sp, _) = build_and_register("producer-gen.drv", &[], &mut kp);

        let mut w = Worker::new(1);
        w.want(&producer_sp, &kp, true).unwrap();
        let result = w.run(&mut builder, &mut kp).await.unwrap();

        // The producer itself is a root, so at least 1 outcome.
        assert!(!result.outcomes.is_empty(), "should have at least the producer outcome");
        assert!(result.failed.is_empty(), "no failures expected");

        let recorded = calls.lock().unwrap();
        // Should have built the producer + the dynamically discovered inner drv.
        assert_eq!(
            recorded.len(),
            2,
            "expected 2 builds (producer + dynamic inner), got {}: {:?}",
            recorded.len(),
            *recorded
        );
    }

    /// Dynamic derivation via streaming: producer arrives over the
    /// eval channel, its output is a .drv, which is then built.
    #[tokio::test]
    async fn dynamic_drv_streaming() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();

        let mut inner_kp = DerivationRegistry::default();
        let (_, inner_drv) = build_and_register("streamed-inner", &[], &mut inner_kp);
        let inner_aterm = inner_drv.to_aterm_bytes();

        let mut drv_outputs = StdHashMap::new();
        drv_outputs.insert("stream-producer.drv".to_string(), inner_aterm);

        let (mock, calls) = DrvProducingMockBuildService::new(bs.clone(), drv_outputs);

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (producer_sp, _) = build_and_register("stream-producer.drv", &[], &mut kp);

        let (tx, mut rx) = mpsc::channel::<EvalMessage>(16);
        tx.send(EvalMessage {
            label: "stream-producer.drv".into(),
            drv_path: producer_sp.clone(),
            new_entries: vec![],
        })
        .await
        .unwrap();
        drop(tx);

        let mut w = Worker::new(1);
        let result = w.run_streaming(&mut builder, &mut kp, &mut rx).await.unwrap();

        assert!(!result.outcomes.is_empty());
        assert!(result.failed.is_empty());

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 2, "expected producer + dynamic inner: {:?}", *recorded);
    }

    /// Non-.drv output is NOT treated as a dynamic derivation.
    #[tokio::test]
    async fn non_drv_output_not_treated_as_dynamic() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();
        let (mock, calls) = MockBuildService::new(bs.clone());

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        // Name does NOT end in .drv.
        let (sp, _) = build_and_register("normal-pkg", &[], &mut kp);

        let mut w = Worker::new(1);
        w.want(&sp, &kp, true).unwrap();
        let result = w.run(&mut builder, &mut kp).await.unwrap();

        assert_eq!(result.outcomes.len(), 1);
        let recorded = calls.lock().unwrap();
        // Only 1 build — no dynamic discovery.
        assert_eq!(recorded.len(), 1);
    }

    // ── Partial failure integration tests ─────────────────────

    use crate::test_support::FailingMockBuildService;

    /// One root fails, the other succeeds. The Worker should continue
    /// building the successful root and report both outcomes.
    #[tokio::test]
    async fn partial_failure_one_fails_one_succeeds() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();

        // "bad-pkg" will fail, "good-pkg" will succeed.
        let (mock, calls) = FailingMockBuildService::new(bs.clone(), vec!["bad-pkg".to_string()]);

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (good_sp, _) = build_and_register("good-pkg", &[], &mut kp);
        let (bad_sp, _) = build_and_register("bad-pkg", &[], &mut kp);

        let mut w = Worker::new(2);
        w.want(&good_sp, &kp, true).unwrap();
        w.want(&bad_sp, &kp, true).unwrap();
        let result = w.run(&mut builder, &mut kp).await.unwrap();

        // good-pkg should succeed.
        assert_eq!(result.outcomes.len(), 1, "one root should succeed");
        assert!(result.outcomes[0].drv_path.name().contains("good-pkg"), "successful outcome should be good-pkg");

        // bad-pkg should be in the failed list.
        assert_eq!(result.failed.len(), 1, "one root should fail");
        assert!(result.failed[0].drv_key.contains("bad-pkg"), "failed goal should be bad-pkg");
        assert!(
            result.failed[0].error.contains("simulated build failure"),
            "error message should describe the failure: {}",
            result.failed[0].error
        );

        // Both builds were attempted.
        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 2, "both builds should be attempted");
    }

    /// A dep fails, its dependent root should also fail (propagated).
    /// An independent root should still succeed.
    #[tokio::test]
    async fn partial_failure_dep_fails_propagates() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();

        // "bad-dep" will fail.
        let (mock, _calls) = FailingMockBuildService::new(bs.clone(), vec!["bad-dep".to_string()]);

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (bad_dep, _) = build_and_register("bad-dep", &[], &mut kp);
        let (top_bad, _) = build_and_register("top-bad", &[(bad_dep.clone(), "out")], &mut kp);
        let (good_sp, _) = build_and_register("good-pkg", &[], &mut kp);

        let mut w = Worker::new(2);
        w.want(&top_bad, &kp, true).unwrap();
        w.want(&good_sp, &kp, true).unwrap();
        let result = w.run(&mut builder, &mut kp).await.unwrap();

        // good-pkg should succeed.
        assert_eq!(result.outcomes.len(), 1, "one root should succeed");
        assert!(result.outcomes[0].drv_path.name().contains("good-pkg"), "successful outcome should be good-pkg");

        // top-bad should fail because its dep bad-dep failed.
        assert_eq!(result.failed.len(), 1, "one root should fail");
        assert!(
            result.failed[0].drv_key.contains("top-bad"),
            "failed goal should be top-bad, got: {}",
            result.failed[0].drv_key
        );
        assert!(
            result.failed[0].error.contains("dependency"),
            "error should mention dependency failure: {}",
            result.failed[0].error
        );
    }

    /// All roots fail — Worker should complete without panic.
    #[tokio::test]
    async fn partial_failure_all_fail() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();

        let (mock, _) = FailingMockBuildService::new(bs.clone(), vec!["fail-a".to_string(), "fail-b".to_string()]);

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (a, _) = build_and_register("fail-a", &[], &mut kp);
        let (b, _) = build_and_register("fail-b", &[], &mut kp);

        let mut w = Worker::new(2);
        w.want(&a, &kp, true).unwrap();
        w.want(&b, &kp, true).unwrap();
        let result = w.run(&mut builder, &mut kp).await.unwrap();

        assert!(result.outcomes.is_empty(), "no outcomes on total failure");
        assert_eq!(result.failed.len(), 2, "both roots should fail");
    }

    /// Streaming partial failure: one root fails, second arrives later
    /// and succeeds.
    #[tokio::test]
    async fn partial_failure_streaming() {
        let bs = MemoryBlobService::default();
        let ds = tmp_ds();

        let (mock, _) = FailingMockBuildService::new(bs.clone(), vec!["stream-bad".to_string()]);

        let mut builder = Builder::new(
            bs,
            ds,
            mock,
            test_pis(),
            PathBuf::from("/nix/store"),
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );

        let mut kp = DerivationRegistry::default();
        let (bad_sp, _) = build_and_register("stream-bad", &[], &mut kp);
        let (good_sp, _) = build_and_register("stream-good", &[], &mut kp);

        let (tx, mut rx) = mpsc::channel::<EvalMessage>(16);
        tx.send(EvalMessage {
            label: "bad".into(),
            drv_path: bad_sp,
            new_entries: vec![],
        })
        .await
        .unwrap();
        tx.send(EvalMessage {
            label: "good".into(),
            drv_path: good_sp,
            new_entries: vec![],
        })
        .await
        .unwrap();
        drop(tx);

        let mut w = Worker::new(2);
        let result = w.run_streaming(&mut builder, &mut kp, &mut rx).await.unwrap();

        assert_eq!(result.outcomes.len(), 1, "one root should succeed");
        assert_eq!(result.failed.len(), 1, "one root should fail");
    }
}
