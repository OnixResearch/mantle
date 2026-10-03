// HARDENING-BACKLOG 2026-09-09: pre-existing tigerstyle findings in this file are
// recorded in the store-capability-migration change evidence and scheduled for the
// standalone hardening pass. Scoped to the lint categories present at recording time.
#![allow(tigerstyle::too_many_parameters)]

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
use nix_compat::store_path::build_ca_path_with_store_dir;
use nix_compat::store_path::StorePath;
use snix_build::buildservice::BuildService;
use tokio::sync::Semaphore;
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tracing::debug;
use tracing::info;

use crate::Error;
use crate::dynamic::DynamicAdmissionLimits;
use crate::dynamic::DynamicParentHashFact;
use crate::dynamic::ExistingDynamicRegistration;
use crate::dynamic::IdentityResolvedDynamicDerivation;
use crate::dynamic::RegistryReadyDynamicDerivation;
use crate::dynamic::ValidatedDynamicDerivation;
use crate::dynamic::parse_dynamic_candidate;
use crate::dynamic::plan_dynamic_registration;
use crate::dynamic::required_parent_paths;
use crate::dynamic::resolve_dynamic_identity;
use crate::dynamic::validate_dynamic_candidate;
use crate::dynamic::validate_registry_ready_batch;
use crate::dynamic_plan::AddressingMode;
use crate::dynamic_plan::CanonicalDynamicPlanV1;
use crate::dynamic_plan::CanonicalDynamicPlanV2;
use crate::dynamic_plan::DeclaredSourceInput;
use crate::dynamic_plan::DynamicInput;
use crate::dynamic_plan::DynamicPlaceholder;
use crate::dynamic_plan::DynamicUnit;
use crate::dynamic_plan::FixedOutputHashAlgo;
use crate::dynamic_plan::FixedOutputMode;
use crate::dynamic_plan::FixedOutputSpec;
use crate::dynamic_plan::MAX_DYNAMIC_PLAN_BYTES;
use crate::dynamic_plan::NarDigest;
use crate::dynamic_plan::OutputName;
use crate::dynamic_plan::SliceNodeKind;
use crate::dynamic_plan::SliceSource;
use crate::dynamic_plan::SliceTreeFact;
use crate::dynamic_plan::SourceId;
use crate::dynamic_plan::SourceV2;
use crate::dynamic_plan::StorePathString;
use crate::dynamic_plan::UnitId;
use crate::dynamic_plan::decode_validated_plan_v1;
use crate::dynamic_plan::decode_validated_plan_v2;
use crate::dynamic_plan::parse_dynamic_placeholders;
use crate::dynamic_plan::plan_slices;
use crate::dynamic_plan::resolve_dynamic_placeholders;
use crate::goal::Goal;
use crate::goal::GoalRegistry;
use crate::goal::GoalState;
use crate::goal::MAX_GOALS;
use crate::orchestrate::BuildOutcome;
use crate::orchestrate::Builder;
use crate::orchestrate::BuilderSourceSlice;
use crate::orchestrate::PrepareResult;
use crate::orchestrate::PreparedBuild;
use crate::registry::DerivationRegistry;
use crate::registry::MAX_ENTRIES;
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

fn observe_dynamic_parent_facts(
    validated: &ValidatedDynamicDerivation,
    known_paths: &DerivationRegistry,
) -> Vec<DynamicParentHashFact> {
    required_parent_paths(validated)
        .into_iter()
        .filter_map(|logical_path| {
            known_paths
                .get_hdm_by_drv_path(&logical_path)
                .map(|digest| DynamicParentHashFact::native(logical_path, digest))
        })
        .collect()
}

fn observe_existing_dynamic_registration(
    resolved: &IdentityResolvedDynamicDerivation,
    known_paths: &DerivationRegistry,
) -> Option<ExistingDynamicRegistration> {
    let logical_path = resolved.drv_path().to_absolute_path_with_prefix(resolved.logical_store_prefix());
    known_paths.get_by_drv_path(&logical_path).map(|entry| ExistingDynamicRegistration {
        logical_drv_path: logical_path,
        full_identity: entry.dynamic_admission_identity,
    })
}

struct DynamicPlaceholderBindings {
    sources: BTreeMap<SourceId, StorePathString>,
    unit_outputs: BTreeMap<(UnitId, OutputName), StorePathString>,
}

struct NativePlanIdentity<'a> {
    producer_key: &'a str,
    output_name: &'a str,
}

struct AcceptedNativePlanInput<'a> {
    identity: NativePlanIdentity<'a>,
    plan_artifact_path: StorePath<String>,
    raw_artifact_digest: String,
    plan: CanonicalNativePlan,
}

/// Probe only the version discriminator; serde ignores other fields without
/// allocating a full JSON value. The selected decoder then validates the whole
/// bounded plan, including unknown fields and malformed nested content.
#[derive(serde::Deserialize)]
struct NativePlanSchemaProbe<'a> {
    #[serde(borrow)]
    schema: &'a str,
}

struct ObservedPlanSubtree {
    observed: crunch_store::ObservedSourceSlice,
    digest: NarDigest,
    kind: SliceNodeKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonicalNativePlan {
    V1(CanonicalDynamicPlanV1),
    V2(CanonicalDynamicPlanV2),
}

impl CanonicalNativePlan {
    fn units(&self) -> &[DynamicUnit] {
        match self {
            Self::V1(plan) => &plan.plan.units,
            Self::V2(plan) => &plan.plan.units,
        }
    }

    fn roots(&self) -> &[UnitId] {
        match self {
            Self::V1(plan) => &plan.plan.roots,
            Self::V2(plan) => &plan.plan.roots,
        }
    }

    fn digest(&self) -> &str {
        match self {
            Self::V1(plan) => plan.digest.as_str(),
            Self::V2(plan) => plan.digest.as_str(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeDynamicSourceSliceReport {
    pub source_id: String,
    pub producer_output: String,
    pub subpath: String,
    pub declared_nar_blake3: String,
    pub observed_nar_blake3: Option<String>,
    pub admitted_store_path: Option<StorePath<String>>,
    pub disposition: String,
}

struct NativePlanRejectionInput<'a> {
    identity: NativePlanIdentity<'a>,
    plan_artifact_path: Option<StorePath<String>>,
    raw_artifact_digest: Option<String>,
    kind: NativeDynamicPlanRejectionKind,
    detail: String,
}

struct DynamicStorePathInput<'a> {
    path: &'a str,
    store_dir: &'a str,
}

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
const BLAKE3_HEX_LENGTH: usize = blake3::OUT_LEN.saturating_mul(BLAKE3_HEX_CHARS_PER_BYTE);
const MAX_FAILED_BUILD_LOG_BYTES: usize = 1_048_576;

/// A root goal that failed, with error context.
#[derive(Debug, Clone)]
pub struct FailedGoal {
    /// Absolute drv-path key.
    pub drv_key: String,
    /// Derivation that caused this root failure.
    pub origin_drv_key: String,
    /// Human-readable error message at the selected root.
    pub error: String,
    /// Original error from the derivation that caused the root failure.
    pub origin_error: String,
    /// Bounded build-service log retained when output admission fails after execution.
    pub build_log: Option<String>,
}

/// Result of running the Worker: outcomes for root goals.
#[derive(Debug)]
pub struct WorkerResult {
    /// Build outcomes for root derivations that succeeded.
    pub outcomes: Vec<BuildOutcome>,
    /// Build outcomes for every completed goal, including dependencies.
    pub all_outcomes: Vec<BuildOutcome>,
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
    pub source_slices: Vec<NativeDynamicSourceSliceReport>,
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
    pub plan: CanonicalNativePlan,
    pub source_slices: Vec<NativeDynamicSourceSliceReport>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeDynamicPlanRejectionKind {
    MissingOutput,
    NonRegularOutput,
    PlanTooLarge,
    InvalidPlan,
    SliceOutputUndeclared,
    SliceAbsent,
    SliceSymlinkTraversal,
    SliceDigestMismatch,
    SliceLimit,
    SliceConflict,
    SlicePublication,
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
            Self::SliceOutputUndeclared => "slice-output-undeclared",
            Self::SliceAbsent => "slice-absent",
            Self::SliceSymlinkTraversal => "slice-symlink-traversal",
            Self::SliceDigestMismatch => "slice-digest-mismatch",
            Self::SliceLimit => "slice-limit",
            Self::SliceConflict => "slice-conflict",
            Self::SlicePublication => "slice-publication-failed",
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
    pub source_slices: Vec<NativeDynamicSourceSliceReport>,
    pub detail: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NativeDynamicPlanScan {
    pub accepted: Vec<NativeDynamicPlanAccepted>,
    pub rejected: Vec<NativeDynamicPlanRejection>,
}

enum NativeDynamicPlanOutputScan {
    Accepted(Box<NativeDynamicPlanAccepted>),
    Rejected(NativeDynamicPlanRejection),
}

struct RegisteredNativeDynamicPlan {
    root_drv_paths: Vec<StorePath<String>>,
    unit_drv_paths: BTreeMap<UnitId, StorePath<String>>,
}

/// Only prospective V2 units are staged; the existing registry stays borrowed.
struct V2StagedUnit {
    drv_path: StorePath<String>,
    hdm: [u8; 32],
    derivation: Derivation,
    content_addressed: bool,
    dynamic_plan_outputs: Vec<String>,
}

struct V2Registration<'a> {
    live: &'a DerivationRegistry,
    staged: BTreeMap<String, V2StagedUnit>,
    order: Vec<String>,
}

struct PreparedV2Registration {
    staged: BTreeMap<String, V2StagedUnit>,
    order: Vec<String>,
    registered: RegisteredNativeDynamicPlan,
}

impl V2Registration<'_> {
    fn preflight_goals(&self, worker: &Worker, roots: &[StorePath<String>]) -> Result<(), Error> {
        let mut created = BTreeSet::new();
        for root in roots {
            let mut queue = VecDeque::from([root.clone()]);
            let mut iterations = 0_u32;
            while let Some(path) = queue.pop_front() {
                iterations = iterations.saturating_add(1);
                if iterations > MAX_GOALS {
                    return Err(Error::Store(format!("want() BFS exceeded iteration limit ({MAX_GOALS})")));
                }
                let key = path.to_absolute_path();
                if worker.registry.contains(&key) || created.contains(&key) {
                    continue;
                }
                let absolute = path.to_absolute_path_with_prefix(self.live.store_dir());
                let derivation = if let Some(unit) = self.staged.get(&absolute) {
                    &unit.derivation
                } else {
                    &self
                        .live
                        .get_by_drv_path(&absolute)
                        .ok_or_else(|| Error::DerivationNotFound { path: path.clone() })?
                        .derivation
                };
                if worker.registry.len() as usize + created.len() >= MAX_GOALS as usize {
                    return Err(Error::Store(format!("goal registry at capacity ({MAX_GOALS})")));
                }
                created.insert(key);
                for dependency in derivation.input_derivations.keys() {
                    if !worker.registry.contains(&dependency.to_absolute_path()) {
                        queue.push_back(dependency.clone());
                    }
                }
            }
        }
        Ok(())
    }
}

trait NativeUnitLookup {
    fn native_hdm(&self, path: &str) -> Option<[u8; 32]>;
    fn native_output(&self, path: &str, output: &str) -> Option<StorePath<String>>;
}

impl NativeUnitLookup for DerivationRegistry {
    fn native_hdm(&self, path: &str) -> Option<[u8; 32]> {
        self.get_hdm_by_drv_path(path)
    }

    fn native_output(&self, path: &str, output: &str) -> Option<StorePath<String>> {
        self.get_output_path(path, output)
    }
}

impl NativeUnitLookup for V2Registration<'_> {
    fn native_hdm(&self, path: &str) -> Option<[u8; 32]> {
        self.staged.get(path).map(|unit| unit.hdm).or_else(|| self.live.native_hdm(path))
    }

    fn native_output(&self, path: &str, output: &str) -> Option<StorePath<String>> {
        self.staged.get(path).map_or_else(
            || self.live.native_output(path, output),
            |unit| unit.derivation.outputs.get(output).and_then(|value| value.path.clone()),
        )
    }
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

fn accepted_native_plan(input: AcceptedNativePlanInput<'_>) -> NativeDynamicPlanAccepted {
    let accepted_unit_ids = input.plan.units().iter().map(|unit| unit.id.as_str().to_owned()).collect();
    NativeDynamicPlanAccepted {
        producer_key: input.identity.producer_key.to_string(),
        output_name: input.identity.output_name.to_string(),
        plan_artifact_path: input.plan_artifact_path,
        raw_artifact_digest: input.raw_artifact_digest,
        canonical_plan_digest: input.plan.digest().to_owned(),
        accepted_unit_ids,
        source_slices: Vec::new(),
        plan: input.plan,
    }
}

fn rejected_accepted_slice_plan(
    accepted: &NativeDynamicPlanAccepted,
    mut rows: Vec<NativeDynamicSourceSliceReport>,
    source_id: Option<&SourceId>,
    kind: NativeDynamicPlanRejectionKind,
    detail: String,
) -> NativeDynamicPlanRejection {
    for row in &mut rows {
        row.admitted_store_path = None;
        row.disposition = if source_id.is_some_and(|id| row.source_id == id.as_str()) {
            kind.as_str().to_string()
        } else {
            "not-admitted".to_string()
        };
    }
    NativeDynamicPlanRejection {
        producer_key: accepted.producer_key.clone(),
        output_name: accepted.output_name.clone(),
        plan_artifact_path: Some(accepted.plan_artifact_path.clone()),
        raw_artifact_digest: Some(accepted.raw_artifact_digest.clone()),
        kind,
        detail,
        source_slices: rows,
    }
}

fn slice_rejection_kind(kind: crate::dynamic_plan::SliceRejectionKind) -> NativeDynamicPlanRejectionKind {
    use crate::dynamic_plan::SliceRejectionKind;
    match kind {
        SliceRejectionKind::OutputUndeclared => NativeDynamicPlanRejectionKind::SliceOutputUndeclared,
        SliceRejectionKind::Absent => NativeDynamicPlanRejectionKind::SliceAbsent,
        SliceRejectionKind::SymlinkTraversal => NativeDynamicPlanRejectionKind::SliceSymlinkTraversal,
        SliceRejectionKind::DigestMismatch => NativeDynamicPlanRejectionKind::SliceDigestMismatch,
        SliceRejectionKind::Limit => NativeDynamicPlanRejectionKind::SliceLimit,
        SliceRejectionKind::Conflict => NativeDynamicPlanRejectionKind::SliceConflict,
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
        source_slices: accepted.source_slices.clone(),
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
        source_slices: rejected.source_slices.clone(),
        rejection_reason: Some(format!("{}: {}", rejected.kind.as_str(), rejected.detail)),
        scheduler_action: "rejected".to_string(),
    }
}

fn native_plan_reports_from_scan(scan: &NativeDynamicPlanScan) -> Vec<NativeDynamicPlanReport> {
    let mut native_plan_rows = Vec::with_capacity(scan.accepted.len().saturating_add(scan.rejected.len()));
    native_plan_rows.extend(scan.accepted.iter().map(native_plan_report_from_accepted));
    native_plan_rows.extend(scan.rejected.iter().map(native_plan_report_from_rejection));
    sort_native_dynamic_plan_reports(&mut native_plan_rows);
    native_plan_rows
}

fn bounded_failed_build_log(log: Option<&str>) -> Option<String> {
    log.filter(|value| value.len() <= MAX_FAILED_BUILD_LOG_BYTES).map(str::to_string)
}

fn finish_worker_result(
    outcomes: Vec<BuildOutcome>,
    all_outcomes: Vec<BuildOutcome>,
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
        all_outcomes,
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

fn native_plan_rejection(input: NativePlanRejectionInput<'_>) -> NativeDynamicPlanRejection {
    NativeDynamicPlanRejection {
        producer_key: input.identity.producer_key.to_string(),
        output_name: input.identity.output_name.to_string(),
        plan_artifact_path: input.plan_artifact_path,
        raw_artifact_digest: input.raw_artifact_digest,
        kind: input.kind,
        source_slices: Vec::new(),
        detail: input.detail,
    }
}

fn parse_dynamic_store_path(input: DynamicStorePathInput<'_>) -> Result<StorePath<String>, Error> {
    StorePath::from_absolute_path_with_prefix(input.path.as_bytes(), input.store_dir).map_err(|_| {
        Error::Store(format!(
            "native dynamic plan store path is invalid for prefix {}: {}",
            input.store_dir, input.path
        ))
    })
}

fn admit_dynamic_binding_path(value: String, store_dir: &str) -> Result<StorePathString, Error> {
    StorePathString::new(value, store_dir)
        .map_err(|error| Error::Store(format!("admitting native dynamic path binding: {error}")))
}

fn dynamic_sources_by_id(
    sources: &[DeclaredSourceInput],
    store_dir: &str,
) -> Result<BTreeMap<SourceId, StorePath<String>>, Error> {
    let source_count_max = sources.len();
    let mut by_id = BTreeMap::new();
    for source in sources {
        let path = parse_dynamic_store_path(DynamicStorePathInput {
            path: source.path.as_str(),
            store_dir,
        })?;
        if by_id.len() >= source_count_max {
            return Err(Error::Store("native dynamic source map exceeded declared source bound".to_string()));
        }
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
    let ca_hash = match spec.mode {
        FixedOutputMode::Flat => CAHash::Flat(hash),
        FixedOutputMode::Recursive => CAHash::Nar(hash),
    };
    debug_assert_eq!(ca_hash.hash().algo(), algo);
    debug_assert_eq!(matches!(&ca_hash, CAHash::Flat(_)), matches!(spec.mode, FixedOutputMode::Flat));
    Ok(ca_hash)
}

fn dynamic_plan_outputs_are_declared(unit: &DynamicUnit) -> bool {
    let outputs = unit.derivation.outputs.iter().collect::<BTreeSet<_>>();
    unit.derivation.dynamic_plan_outputs.iter().all(|output| outputs.contains(output))
}

fn unit_output_dependencies_registered(unit: &DynamicUnit, registered: &BTreeMap<UnitId, StorePath<String>>) -> bool {
    unit.derivation.inputs.iter().all(|input| match input {
        DynamicInput::UnitOutput { unit, .. } => registered.contains_key(unit),
        DynamicInput::StorePath { .. } | DynamicInput::Source { .. } => true,
    })
}

fn v2_sources_by_id(
    accepted: &NativeDynamicPlanAccepted,
    rows: &[NativeDynamicSourceSliceReport],
    store_dir: &str,
) -> Result<BTreeMap<SourceId, StorePath<String>>, Error> {
    let CanonicalNativePlan::V2(plan) = &accepted.plan else {
        return Err(Error::Store("expected V2 native dynamic plan".to_owned()));
    };
    let mut by_id = BTreeMap::new();
    for source in &plan.plan.sources {
        let path = match source {
            SourceV2::StorePath(source) => parse_dynamic_store_path(DynamicStorePathInput {
                path: source.path.as_str(),
                store_dir,
            })?,
            SourceV2::Slice(slice) => rows
                .iter()
                .find(|row| row.source_id == slice.id.as_str())
                .and_then(|row| row.admitted_store_path.clone())
                .ok_or_else(|| Error::Store(format!("source slice '{}' was not admitted", slice.id)))?,
        };
        by_id.insert(source.id().clone(), path);
    }
    Ok(by_id)
}

fn prepare_v2_registration(
    accepted: &NativeDynamicPlanAccepted,
    rows: &[NativeDynamicSourceSliceReport],
    known_paths: &DerivationRegistry,
    worker: &Worker,
) -> Result<PreparedV2Registration, Error> {
    let store_dir = known_paths.store_dir();
    let sources = v2_sources_by_id(accepted, rows, store_dir)?;
    let units_by_id = accepted.plan.units().iter().map(|unit| (unit.id.clone(), unit)).collect::<BTreeMap<_, _>>();
    let mut pending = units_by_id.keys().cloned().collect::<BTreeSet<_>>();
    let mut registered = BTreeMap::new();
    let mut stage = V2Registration {
        live: known_paths,
        staged: BTreeMap::new(),
        order: Vec::new(),
    };
    for _ in 0..units_by_id.len() {
        if pending.is_empty() {
            break;
        }
        let ready_units = pending
            .iter()
            .filter(|id| {
                units_by_id.get(*id).is_some_and(|unit| unit_output_dependencies_registered(unit, &registered))
            })
            .cloned()
            .collect::<Vec<_>>();
        if ready_units.is_empty() {
            break;
        }
        for id in ready_units {
            let unit = units_by_id[&id];
            let prepared = prepare_native_dynamic_unit(unit, &sources, &registered, &stage, store_dir)?;
            let path = prepared.drv_path.clone();
            let absolute = path.to_absolute_path_with_prefix(store_dir);
            if let Some(existing) = stage.staged.get(&absolute) {
                if existing.hdm != prepared.hdm
                    || existing.content_addressed != prepared.content_addressed
                    || existing.dynamic_plan_outputs != prepared.dynamic_plan_outputs
                    || existing.derivation.to_aterm_bytes_with_store_dir(store_dir)
                        != prepared.derivation.to_aterm_bytes_with_store_dir(store_dir)
                {
                    return Err(Error::Store(format!("dynamic duplicate `{absolute}` changed admitted identity")));
                }
            } else if let Some(existing) = known_paths.get_by_drv_path(&absolute) {
                if existing.hash_derivation_modulo != prepared.hdm
                    || existing.content_addressed != prepared.content_addressed
                    || existing.dynamic_plan_outputs != prepared.dynamic_plan_outputs
                    || existing.derivation.to_aterm_bytes_with_store_dir(store_dir)
                        != prepared.derivation.to_aterm_bytes_with_store_dir(store_dir)
                {
                    return Err(Error::Store(format!(
                        "dynamic insertion `{absolute}` collides with an existing registry entry"
                    )));
                }
            } else {
                if known_paths.len() as usize + stage.staged.len() + 1 > MAX_ENTRIES as usize {
                    return Err(Error::Store(format!("dynamic registry exceeds MAX_ENTRIES ({MAX_ENTRIES})")));
                }
                stage.order.push(absolute.clone());
                stage.staged.insert(absolute, prepared);
            }
            registered.insert(id.clone(), path);
            pending.remove(&id);
        }
    }
    if !pending.is_empty() {
        return Err(Error::Store(format!(
            "native dynamic plan '{}' has unresolved unit dependencies: {}",
            accepted.canonical_plan_digest,
            pending.into_iter().map(|unit| unit.to_string()).collect::<Vec<_>>().join(", ")
        )));
    }
    let root_drv_paths = accepted
        .plan
        .roots()
        .iter()
        .map(|root| {
            registered
                .get(root)
                .cloned()
                .ok_or_else(|| Error::Store(format!("native dynamic plan root was not registered: {root}")))
        })
        .collect::<Result<Vec<_>, _>>()?;
    stage.preflight_goals(worker, &root_drv_paths)?;
    Ok(PreparedV2Registration {
        staged: stage.staged,
        order: stage.order,
        registered: RegisteredNativeDynamicPlan {
            root_drv_paths,
            unit_drv_paths: registered,
        },
    })
}

fn prepare_native_dynamic_unit(
    unit: &DynamicUnit,
    sources: &BTreeMap<SourceId, StorePath<String>>,
    registered_units: &BTreeMap<UnitId, StorePath<String>>,
    known_paths: &impl NativeUnitLookup,
    store_dir: &str,
) -> Result<V2StagedUnit, Error> {
    if !dynamic_plan_outputs_are_declared(unit) {
        return Err(Error::Store(format!(
            "native dynamic unit '{}' declares dynamic_plan_outputs outside outputs",
            unit.id
        )));
    }

    let mut derivation = build_native_dynamic_derivation(unit, sources, registered_units, known_paths, store_dir)?;
    let mut parent_hdms = BTreeMap::new();
    for parent_drv_path in derivation.input_derivations.keys() {
        let parent_abs = parent_drv_path.to_absolute_path_with_prefix(store_dir);
        let digest = known_paths.native_hdm(&parent_abs).ok_or_else(|| {
            Error::Store(format!(
                "native dynamic unit '{}' references unregistered parent derivation {parent_abs}",
                unit.id
            ))
        })?;
        parent_hdms.insert(parent_drv_path.as_ref(), digest);
    }
    let hdm = derivation.hash_derivation_modulo_with_store_dir(|parent| parent_hdms[parent], store_dir);
    let content_addressed = matches!(unit.derivation.addressing_mode, AddressingMode::ContentAddressed)
        && unit.derivation.fixed_output.is_none();
    derivation
        .calculate_output_paths_with_store_dir(&unit.derivation.name, &hdm, store_dir)
        .map_err(|err| Error::Store(format!("computing native dynamic output paths for '{}': {err}", unit.id)))?;
    if content_addressed {
        for output in derivation.outputs.values_mut() {
            output.path = None;
        }
    }
    let drv_path = derivation
        .calculate_derivation_path_with_store_dir(&unit.derivation.name, store_dir)
        .map_err(|err| Error::Store(format!("computing native dynamic derivation path for '{}': {err}", unit.id)))?;
    Ok(V2StagedUnit {
        drv_path,
        hdm,
        derivation,
        content_addressed,
        dynamic_plan_outputs: unit
            .derivation
            .dynamic_plan_outputs
            .iter()
            .map(|output| output.as_str().to_owned())
            .collect(),
    })
}

fn register_native_dynamic_plan_units(
    accepted: &NativeDynamicPlanAccepted,
    known_paths: &mut DerivationRegistry,
) -> Result<RegisteredNativeDynamicPlan, Error> {
    let CanonicalNativePlan::V1(plan) = &accepted.plan else {
        return Err(Error::Store("expected V1 native dynamic plan".to_owned()));
    };
    let store_dir = known_paths.store_dir().to_string();
    let sources = dynamic_sources_by_id(&plan.plan.sources, &store_dir)?;
    let units_by_id = plan.plan.units.iter()
        .map(|unit| (unit.id.clone(), unit))
        .collect::<BTreeMap<UnitId, &DynamicUnit>>();
    let mut pending = units_by_id.keys().cloned().collect::<BTreeSet<UnitId>>();
    let registered_unit_count_max = units_by_id.len();
    let unit_iteration_count_max = plan.plan.units.len();
    let mut registered = BTreeMap::new();
    debug_assert_eq!(pending.len(), registered_unit_count_max);

    for _ in 0..unit_iteration_count_max {
        if pending.is_empty() {
            break;
        }
        let ready_units = pending
            .iter()
            .filter(|unit_id| {
                units_by_id.get(*unit_id).is_some_and(|unit| unit_output_dependencies_registered(unit, &registered))
            })
            .cloned()
            .collect::<Vec<UnitId>>();
        if ready_units.is_empty() {
            break;
        }
        for unit_id in ready_units {
            let Some(unit) = units_by_id.get(&unit_id) else {
                return Err(Error::Store(format!("native dynamic unit disappeared during registration: {unit_id}")));
            };
            let drv_path = register_native_dynamic_unit(unit, &sources, &registered, known_paths, &store_dir)?;
            if registered.len() >= registered_unit_count_max {
                return Err(Error::Store("native dynamic unit map exceeded validated plan bound".to_string()));
            }
            registered.insert(unit_id.clone(), drv_path);
            pending.remove(&unit_id);
        }
    }

    if !pending.is_empty() {
        return Err(Error::Store(format!(
            "native dynamic plan '{}' has unresolved unit dependencies: {}",
            accepted.canonical_plan_digest,
            pending.into_iter().map(|unit| unit.to_string()).collect::<Vec<_>>().join(", ")
        )));
    }

    debug_assert_eq!(registered.len(), registered_unit_count_max);
    debug_assert!(plan.plan.roots.len() <= registered.len());
    let mut root_drv_paths = Vec::with_capacity(plan.plan.roots.len());
    for root in &plan.plan.roots {
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
    sources: &BTreeMap<SourceId, StorePath<String>>,
    registered_units: &BTreeMap<UnitId, StorePath<String>>,
    known_paths: &mut DerivationRegistry,
    store_dir: &str,
) -> Result<StorePath<String>, Error> {
    if !dynamic_plan_outputs_are_declared(unit) {
        return Err(Error::Store(format!(
            "native dynamic unit '{}' declares dynamic_plan_outputs outside outputs",
            unit.id
        )));
    }

    let mut derivation = build_native_dynamic_derivation(unit, sources, registered_units, known_paths, store_dir)?;
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
    debug_assert_eq!(derivation.outputs.len(), unit.derivation.outputs.len());
    debug_assert!(derivation.input_derivations.keys().all(|parent_drv_path| {
        let parent_abs = parent_drv_path.to_absolute_path_with_prefix(store_dir);
        known_paths.get_hdm_by_drv_path(&parent_abs).is_some()
    }));
    known_paths.insert_with_dynamic_plan_outputs(
        drv_path.clone(),
        hdm,
        derivation,
        is_ca,
        unit.derivation.dynamic_plan_outputs.iter().map(|output| output.as_str().to_owned()).collect(),
        None,
    );
    Ok(drv_path)
}

fn build_native_dynamic_derivation(
    unit: &DynamicUnit,
    sources: &BTreeMap<SourceId, StorePath<String>>,
    registered_units: &BTreeMap<UnitId, StorePath<String>>,
    known_paths: &impl NativeUnitLookup,
    store_dir: &str,
) -> Result<Derivation, Error> {
    let ca_hash = unit.derivation.fixed_output.as_ref().map(parse_dynamic_fixed_output).transpose()?;
    let outputs = unit
        .derivation
        .outputs
        .iter()
        .map(|output_name| {
            (output_name.as_str().to_owned(), Output {
                path: None,
                ca_hash: if output_name.as_str() == "out" {
                    ca_hash.clone()
                } else {
                    None
                },
            })
        })
        .collect();
    let bindings = dynamic_placeholder_bindings(unit, sources, registered_units, known_paths, store_dir)?;
    let arguments = unit
        .derivation
        .args
        .iter()
        .map(|value| resolve_dynamic_value(unit, value, &bindings))
        .collect::<Result<Vec<_>, _>>()?;
    let mut environment = unit
        .derivation
        .env
        .iter()
        .map(|(key, value)| {
            resolve_dynamic_value(unit, value, &bindings).map(|resolved| (key.clone(), BString::from(resolved)))
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    environment.insert("system".to_string(), unit.derivation.system.as_bytes().into());
    environment.insert("builder".to_string(), unit.derivation.builder.as_str().as_bytes().into());
    environment.insert("name".to_string(), unit.derivation.name.as_bytes().into());
    environment.extend(
        unit.derivation
            .outputs
            .iter()
            .map(|output_name| (output_name.as_str().to_owned(), BString::from(""))),
    );
    let output_names = unit.derivation.outputs.iter().map(OutputName::as_str).collect::<Vec<_>>().join(" ");
    environment.insert("outputs".to_string(), output_names.as_bytes().into());

    let (input_derivations, input_sources) = resolve_native_dynamic_inputs(unit, sources, registered_units, store_dir)?;
    assert_eq!(arguments.len(), unit.derivation.args.len());
    assert!(bindings.sources.len() <= unit.derivation.inputs.len());
    Ok(Derivation {
        arguments,
        builder: unit.derivation.builder.as_str().to_owned(),
        environment,
        input_derivations,
        input_sources,
        outputs,
        system: unit.derivation.system.clone(),
    })
}

fn dynamic_placeholder_bindings(
    unit: &DynamicUnit,
    sources: &BTreeMap<SourceId, StorePath<String>>,
    registered_units: &BTreeMap<UnitId, StorePath<String>>,
    known_paths: &impl NativeUnitLookup,
    store_dir: &str,
) -> Result<DynamicPlaceholderBindings, Error> {
    let required_outputs = unit
        .derivation
        .args
        .iter()
        .chain(unit.derivation.env.values())
        .map(|value| parse_dynamic_placeholders(value))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| Error::Store(format!("parsing native dynamic placeholders for unit '{}': {error}", unit.id)))?
        .into_iter()
        .flatten()
        .filter_map(|placeholder| match placeholder {
            DynamicPlaceholder::UnitOutput { unit, output } => Some((unit, output)),
            DynamicPlaceholder::Source { .. } => None,
        })
        .collect::<BTreeSet<_>>();
    let binding_count_max = unit.derivation.inputs.len();
    let mut source_bindings = BTreeMap::new();
    let mut output_bindings = BTreeMap::new();
    for input in &unit.derivation.inputs {
        match input {
            DynamicInput::Source { source } => {
                let path = sources.get(source).ok_or_else(|| {
                    Error::Store(format!("native dynamic unit '{}' references unknown source {source}", unit.id))
                })?;
                if source_bindings.len() >= binding_count_max {
                    return Err(Error::Store("native dynamic source bindings exceeded input bound".to_string()));
                }
                let binding_path = admit_dynamic_binding_path(path.to_absolute_path_with_prefix(store_dir), store_dir)?;
                source_bindings.insert(source.clone(), binding_path);
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
                let drv_abs = drv_path.to_absolute_path_with_prefix(store_dir);
                let binding_key = (dependency.clone(), output.clone());
                if let Some(path) = known_paths.native_output(&drv_abs, output.as_str()) {
                    if output_bindings.len() >= binding_count_max {
                        return Err(Error::Store("native dynamic output bindings exceeded input bound".to_string()));
                    }
                    let binding_path =
                        admit_dynamic_binding_path(path.to_absolute_path_with_prefix(store_dir), store_dir)?;
                    output_bindings.insert(binding_key, binding_path);
                } else if required_outputs.contains(&binding_key) {
                    return Err(Error::Store(format!(
                        "native dynamic unit '{}' requires a statically known path for {dependency}:{output}",
                        unit.id
                    )));
                }
            }
            DynamicInput::StorePath { .. } => {}
        }
    }
    assert!(source_bindings.len() <= unit.derivation.inputs.len());
    assert!(output_bindings.len() <= unit.derivation.inputs.len());
    Ok(DynamicPlaceholderBindings {
        sources: source_bindings,
        unit_outputs: output_bindings,
    })
}

fn resolve_dynamic_value(
    unit: &DynamicUnit,
    value: &str,
    bindings: &DynamicPlaceholderBindings,
) -> Result<String, Error> {
    resolve_dynamic_placeholders(value, &bindings.sources, &bindings.unit_outputs)
        .map_err(|error| Error::Store(format!("resolving native dynamic placeholders for unit '{}': {error}", unit.id)))
}

type NativeDynamicInputDerivations = BTreeMap<StorePath<String>, BTreeSet<String>>;
type NativeDynamicInputSources = BTreeSet<StorePath<String>>;
type NativeDynamicInputs = (NativeDynamicInputDerivations, NativeDynamicInputSources);

fn resolve_native_dynamic_inputs(
    unit: &DynamicUnit,
    sources: &BTreeMap<SourceId, StorePath<String>>,
    registered_units: &BTreeMap<UnitId, StorePath<String>>,
    store_dir: &str,
) -> Result<NativeDynamicInputs, Error> {
    let mut input_derivations: BTreeMap<StorePath<String>, BTreeSet<String>> = BTreeMap::new();
    let mut input_sources = BTreeSet::new();
    for input in &unit.derivation.inputs {
        match input {
            DynamicInput::StorePath { path } => {
                input_sources.insert(parse_dynamic_store_path(DynamicStorePathInput {
                    path: path.as_str(),
                    store_dir,
                })?);
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
                input_derivations.entry(drv_path.clone()).or_default().insert(output.as_str().to_owned());
            }
        }
    }
    debug_assert!(input_derivations.len() <= unit.derivation.inputs.len());
    debug_assert!(input_sources.len() <= unit.derivation.inputs.len());
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
    all_outcomes: &'a mut Vec<BuildOutcome>,
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
    debug_assert_eq!(digest.len(), BLAKE3_HEX_LENGTH);
    debug_assert!(!digest.chars().any(char::is_control));
    digest
}

impl Worker {
    /// Create a test Worker with the default scheduling policy.
    #[cfg(test)]
    pub(crate) fn new(max_jobs: u32) -> Self {
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
        let node_count = u32::try_from(self.registry.iter().count())
            .map_err(|_| Error::Store("known graph node count exceeds u32".to_string()))?;
        if node_count > MAX_GOALS {
            return Err(Error::Store(format!("known graph node limit exceeded ({MAX_GOALS})")));
        }
        let mut edge_count = 0_u32;
        let mut nodes: BTreeMap<String, (Vec<String>, BTreeSet<String>, bool)> = self
            .registry
            .iter()
            .map(|(goal_key, goal)| {
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
                Ok((goal_key.to_string(), (waitees, BTreeSet::new(), goal.is_root)))
            })
            .collect::<Result<_, Error>>()?;
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

        let is_ready_after_inspection = *state == GoalState::Ready;
        if is_ready_after_inspection {
            self.enqueue_ready_goal(key)?;
        }

        debug_assert_eq!(is_ready_after_inspection, self.ready_goals.contains_key(key));
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
        let mut all_outcomes: Vec<BuildOutcome> = Vec::new();
        let mut failed: Vec<FailedGoal> = Vec::new();
        let mut native_dynamic_plans: Vec<NativeDynamicPlanReport> = Vec::new();
        let mut state = WorkerLoopState {
            sem: Arc::new(Semaphore::new(
                usize::try_from(self.max_jobs).map_err(|e| Error::Store(format!("max_jobs overflow: {e}")))?,
            )),
            join_set: JoinSet::new(),
            pending_meta: HashMap::new(),
            outcomes: &mut outcomes,
            all_outcomes: &mut all_outcomes,
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
                return Ok(finish_worker_result(
                    outcomes,
                    all_outcomes,
                    failed,
                    native_dynamic_plans,
                    priority_decisions,
                ));
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
        let mut all_outcomes: Vec<BuildOutcome> = Vec::new();
        let mut failed: Vec<FailedGoal> = Vec::new();
        let mut native_dynamic_plans: Vec<NativeDynamicPlanReport> = Vec::new();
        let mut state = WorkerLoopState {
            sem: Arc::new(Semaphore::new(
                usize::try_from(self.max_jobs).map_err(|e| Error::Store(format!("max_jobs overflow: {e}")))?,
            )),
            join_set: JoinSet::new(),
            pending_meta: HashMap::new(),
            outcomes: &mut outcomes,
            all_outcomes: &mut all_outcomes,
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
                return Ok(finish_worker_result(
                    outcomes,
                    all_outcomes,
                    failed,
                    native_dynamic_plans,
                    priority_decisions,
                ));
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
                self.fail_goal(&drv_key, &err_msg, None, state.failed)?;
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

        let build_log = bounded_failed_build_log(build_result.log.as_deref());
        let outcome = match builder.finish_build(&prepared, build_result, known_paths).await {
            Ok(outcome) => outcome,
            Err(e) => {
                let err_msg = format!("{e}");
                tracing::warn!(drv = drv_key, err = %err_msg, "build failed, marking goal as failed");
                self.fail_goal(drv_key, &err_msg, build_log.as_deref(), state.failed)?;
                return Ok(());
            }
        };

        self.handle_completed_outcome(drv_key, outcome, builder, known_paths, state).await
    }

    async fn handle_completed_outcome<BServ>(
        &mut self,
        drv_key: &str,
        outcome: BuildOutcome,
        builder: &mut Builder<BServ>,
        known_paths: &mut DerivationRegistry,
        state: &mut WorkerLoopState<'_>,
    ) -> Result<(), Error>
    where
        BServ: BuildService + 'static,
    {
        let declared_native_outputs = self.declared_native_dynamic_outputs(&outcome, known_paths);
        let mut native_scan = self
            .scan_native_dynamic_plans(drv_key, &outcome, builder, known_paths.store_dir(), &declared_native_outputs)
            .await?;
        for mut accepted in native_scan.accepted.drain(..) {
            let mut staged = None;
            match self.admit_v2_source_slices(&mut accepted, &outcome, known_paths, builder, &mut staged).await? {
                Some(rejection) => native_scan.rejected.push(rejection),
                None => {
                    state.native_dynamic_plans.push(native_plan_report_from_accepted(&accepted));
                    self.register_accepted_native_dynamic_plan(accepted, staged, known_paths)?;
                }
            }
        }
        self.log_native_dynamic_plan_scan(&native_scan);
        state.native_dynamic_plans.extend(native_plan_reports_from_scan(&native_scan));

        self.detect_dynamic_derivations(drv_key, &outcome, builder, known_paths, &declared_native_outputs)
            .await?;

        self.complete_goal(drv_key, outcome, state.outcomes, state.all_outcomes, state.failed)
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
                NativeDynamicPlanOutputScan::Accepted(accepted) => scan.accepted.push(*accepted),
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
            return Ok(NativeDynamicPlanOutputScan::Rejected(native_plan_rejection(NativePlanRejectionInput {
                identity: NativePlanIdentity {
                    producer_key,
                    output_name,
                },
                plan_artifact_path: None,
                raw_artifact_digest: None,
                kind: NativeDynamicPlanRejectionKind::MissingOutput,
                detail: "declared dynamic-plan output is missing from build result".to_string(),
            })));
        };

        let Some(size) = node_file_size(&path_info.node) else {
            return Ok(NativeDynamicPlanOutputScan::Rejected(native_plan_rejection(NativePlanRejectionInput {
                identity: NativePlanIdentity {
                    producer_key,
                    output_name,
                },
                plan_artifact_path: Some(path_info.store_path.clone()),
                raw_artifact_digest: None,
                kind: NativeDynamicPlanRejectionKind::NonRegularOutput,
                detail: "declared dynamic-plan output is not a regular file".to_string(),
            })));
        };

        if size > MAX_DYNAMIC_PLAN_BYTES {
            return Ok(NativeDynamicPlanOutputScan::Rejected(native_plan_rejection(NativePlanRejectionInput {
                identity: NativePlanIdentity {
                    producer_key,
                    output_name,
                },
                plan_artifact_path: Some(path_info.store_path.clone()),
                raw_artifact_digest: None,
                kind: NativeDynamicPlanRejectionKind::PlanTooLarge,
                detail: format!("declared dynamic-plan output is {size} bytes; limit is {MAX_DYNAMIC_PLAN_BYTES}"),
            })));
        }

        let content = match builder.read_blob(&path_info.node).await {
            Ok(content) => content,
            Err(err) => {
                return Ok(NativeDynamicPlanOutputScan::Rejected(native_plan_rejection(NativePlanRejectionInput {
                    identity: NativePlanIdentity {
                        producer_key,
                        output_name,
                    },
                    plan_artifact_path: Some(path_info.store_path.clone()),
                    raw_artifact_digest: None,
                    kind: NativeDynamicPlanRejectionKind::ReadFailed,
                    detail: err.to_string(),
                })));
            }
        };
        let raw_digest = blake3_hex(&content);
        let is_v2 = serde_json::from_slice::<NativePlanSchemaProbe<'_>>(&content)
            .is_ok_and(|probe| probe.schema == crate::dynamic_plan::MANTLE_PLAN_V2_SCHEMA);
        let decoded = if is_v2 {
            decode_validated_plan_v2(&content, store_prefix).map(CanonicalNativePlan::V2)
        } else {
            decode_validated_plan_v1(&content, store_prefix).map(CanonicalNativePlan::V1)
        };
        match decoded {
            Ok(plan) => {
                Ok(NativeDynamicPlanOutputScan::Accepted(Box::new(accepted_native_plan(AcceptedNativePlanInput {
                    identity: NativePlanIdentity {
                        producer_key,
                        output_name,
                    },
                    plan_artifact_path: path_info.store_path.clone(),
                    raw_artifact_digest: raw_digest,
                    plan,
                }))))
            }
            Err(err) => Ok(NativeDynamicPlanOutputScan::Rejected(native_plan_rejection(NativePlanRejectionInput {
                identity: NativePlanIdentity {
                    producer_key,
                    output_name,
                },
                plan_artifact_path: Some(path_info.store_path.clone()),
                raw_artifact_digest: Some(raw_digest),
                kind: NativeDynamicPlanRejectionKind::InvalidPlan,
                detail: err.to_string(),
            }))),
        }
    }

    /// Publish the whole validated source batch before any unit or goal mutation.
    /// Ordinary pre-commit failures reject only this plan; uncertain commits abort
    /// the run rather than making a false "rejected without publication" claim.
    async fn admit_v2_source_slices<BServ>(
        &self,
        accepted: &mut NativeDynamicPlanAccepted,
        outcome: &BuildOutcome,
        known_paths: &DerivationRegistry,
        builder: &mut Builder<BServ>,
        prepared: &mut Option<PreparedV2Registration>,
    ) -> Result<Option<NativeDynamicPlanRejection>, Error>
    where
        BServ: BuildService + 'static,
    {
        let CanonicalNativePlan::V2(plan) = &accepted.plan else {
            return Ok(None);
        };
        let slices = plan
            .plan
            .sources
            .iter()
            .filter_map(|source| match source {
                SourceV2::Slice(slice) => Some(slice),
                SourceV2::StorePath(_) => None,
            })
            .collect::<Vec<&SliceSource>>();
        let mut rows = slices
            .iter()
            .map(|slice| NativeDynamicSourceSliceReport {
                source_id: slice.id.as_str().to_owned(),
                producer_output: slice.producer_output.as_str().to_owned(),
                subpath: slice.subpath.clone(),
                declared_nar_blake3: slice.nar_blake3.as_str().to_owned(),
                observed_nar_blake3: None,
                admitted_store_path: None,
                disposition: "not-admitted".to_string(),
            })
            .collect::<Vec<_>>();
        let producer_path = outcome.drv_path.to_absolute_path_with_prefix(known_paths.store_dir());
        let declared_by_producer = known_paths.get_by_drv_path(&producer_path).map(|entry| &entry.derivation.outputs);
        let declared_outputs = slices
            .iter()
            .filter(|slice| {
                slice.producer_output.as_str() != accepted.output_name
                    && declared_by_producer.is_some_and(|outputs| outputs.contains_key(slice.producer_output.as_str()))
            })
            .map(|slice| slice.producer_output.clone())
            .collect::<BTreeSet<_>>();
        let mut observations = Vec::with_capacity(slices.len());
        let mut observations_by_source = BTreeMap::new();
        let mut observations_by_subtree: BTreeMap<(&OutputName, &str), usize> = BTreeMap::new();
        let mut facts = Vec::with_capacity(slices.len());
        for (slice, row) in slices.iter().zip(&mut rows) {
            let Some(output) = outcome
                .outputs
                .get(slice.producer_output.as_str())
                .filter(|_| declared_outputs.contains(&slice.producer_output))
            else {
                facts.push(SliceTreeFact {
                    source_id: slice.id.clone(),
                    producer_output: slice.producer_output.clone(),
                    subpath: slice.subpath.clone(),
                    kind: SliceNodeKind::Absent,
                    traversed_symlink: false,
                    observed_nar_blake3: None,
                    nar_bytes: 0,
                });
                continue;
            };
            let subtree_key = (&slice.producer_output, slice.subpath.as_str());
            let observation_index = if let Some(index) = observations_by_subtree.get(&subtree_key) {
                *index
            } else {
                let observation = match builder.observe_source_slice(&output.node, &slice.subpath).await {
                    Ok(observation) => observation,
                    Err(error) => {
                        let message = error.to_string();
                        let kind = if message.contains("verified-source-slice-absent") {
                            SliceNodeKind::Absent
                        } else if message.contains("verified-source-slice-symlink-traversal") {
                            SliceNodeKind::Symlink
                        } else {
                            let rejection_kind = if message.contains("verified-source-slice-limit") {
                                NativeDynamicPlanRejectionKind::SliceLimit
                            } else {
                                NativeDynamicPlanRejectionKind::ReadFailed
                            };
                            return Ok(Some(rejected_accepted_slice_plan(
                                accepted,
                                rows,
                                Some(&slice.id),
                                rejection_kind,
                                format!("source {} subpath {}: {message}", slice.id, slice.subpath),
                            )));
                        };
                        facts.push(SliceTreeFact {
                            source_id: slice.id.clone(),
                            producer_output: slice.producer_output.clone(),
                            subpath: slice.subpath.clone(),
                            kind,
                            traversed_symlink: kind == SliceNodeKind::Symlink,
                            observed_nar_blake3: None,
                            nar_bytes: 0,
                        });
                        continue;
                    }
                };
                let digest = NarDigest::new(data_encoding::HEXLOWER.encode(&observation.nar_blake3()))
                    .expect("the store observation supplies exactly one BLAKE3 digest");
                let kind = match observation.node() {
                    snix_castore::Node::File { .. } => SliceNodeKind::File,
                    snix_castore::Node::Directory { .. } => SliceNodeKind::Directory,
                    snix_castore::Node::Symlink { .. } => SliceNodeKind::Symlink,
                };
                let index = observations.len();
                observations.push(ObservedPlanSubtree {
                    observed: observation,
                    digest,
                    kind,
                });
                observations_by_subtree.insert(subtree_key, index);
                index
            };
            let observation = &observations[observation_index];
            let digest = &observation.digest;
            row.observed_nar_blake3 = Some(digest.as_str().to_owned());
            facts.push(SliceTreeFact {
                source_id: slice.id.clone(),
                producer_output: slice.producer_output.clone(),
                subpath: slice.subpath.clone(),
                kind: observation.kind,
                traversed_symlink: false,
                observed_nar_blake3: Some(digest.clone()),
                nar_bytes: observation.observed.nar_size(),
            });
            observations_by_source.insert(slice.id.clone(), observation_index);
        }
        let planned = match plan_slices(&slices, &declared_outputs, &facts) {
            Ok(planned) => planned,
            Err(rejection) => {
                return Ok(Some(rejected_accepted_slice_plan(
                    accepted,
                    rows,
                    Some(&rejection.source_id),
                    slice_rejection_kind(rejection.kind),
                    format!("source {}: {}", rejection.source_id, rejection.detail),
                )));
            }
        };
        let publications =
            planned.iter().filter(|slice| slice.publication_source_id == slice.source_id).collect::<Vec<_>>();
        let requests = publications
            .iter()
            .map(|slice| BuilderSourceSlice {
                observed: &observations[observations_by_source[&slice.source_id]].observed,
                store_name: &slice.store_name,
            })
            .collect::<Vec<_>>();
        // Project exactly the path the batch publisher will sign, then
        // validate the entire V2 unit graph and root closure before any write.
        let mut projected = BTreeMap::new();
        for slice in &publications {
            let observation = &observations[observations_by_source[&slice.source_id]].observed;
            let ca = CAHash::Nar(NixHash::Sha256(observation.nar_sha256()));
            let path = match build_ca_path_with_store_dir(
                &slice.store_name,
                &ca,
                Vec::<&str>::new(),
                false,
                builder.store_dir(),
            ) {
                Ok(path) => path,
                Err(error) => {
                    return Ok(Some(rejected_accepted_slice_plan(
                        accepted,
                        rows,
                        Some(&slice.source_id),
                        NativeDynamicPlanRejectionKind::SlicePublication,
                        error.to_string(),
                    )));
                }
            };
            projected.insert(slice.source_id.clone(), path);
        }
        for (slice, row) in planned.iter().zip(&mut rows) {
            row.admitted_store_path = Some(
                projected
                    .get(&slice.publication_source_id)
                    .expect("the canonical publication owner has a projected path")
                    .clone(),
            );
        }
        let staged = match prepare_v2_registration(accepted, &rows, known_paths, self) {
            Ok(staged) => staged,
            Err(error) => {
                return Ok(Some(rejected_accepted_slice_plan(
                    accepted,
                    rows,
                    None,
                    NativeDynamicPlanRejectionKind::InvalidPlan,
                    error.to_string(),
                )));
            }
        };
        let published = match builder.admit_source_slice_batch(&requests).await {
            Ok(published) => published,
            Err(error) => {
                if matches!(&error, Error::Store(detail) if detail.contains("verified-source-batch-publication-uncertain"))
                {
                    return Err(error);
                }
                return Ok(Some(rejected_accepted_slice_plan(
                    accepted,
                    rows,
                    None,
                    NativeDynamicPlanRejectionKind::SlicePublication,
                    error.to_string(),
                )));
            }
        };
        if published.len() != publications.len() {
            return Err(Error::Store(
                "verified source batch returned an incomplete result after publication".to_string(),
            ));
        }
        for (slice, result) in publications.iter().zip(&published) {
            if result.nar_blake3 != observations[observations_by_source[&slice.source_id]].observed.nar_blake3()
                || result.nar_size != slice.nar_bytes
                || result.logical_store_path
                    != projected[&slice.source_id].to_absolute_path_with_prefix(builder.store_dir())
            {
                return Err(Error::Store(
                    "verified source batch returned a mismatched result after publication".to_string(),
                ));
            }
        }
        for row in &mut rows {
            row.disposition = "admitted".to_string();
        }
        accepted.source_slices = rows;
        *prepared = Some(staged);
        Ok(None)
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

    fn register_accepted_native_dynamic_plan(
        &mut self,
        accepted: NativeDynamicPlanAccepted,
        staged: Option<PreparedV2Registration>,
        known_paths: &mut DerivationRegistry,
    ) -> Result<(), Error> {
        let registered = if let Some(mut staged) = staged {
            for path in staged.order {
                let unit = staged.staged.remove(&path).ok_or_else(|| {
                    Error::Store(format!("staged native dynamic unit disappeared after publication: {path}"))
                })?;
                known_paths.insert_with_dynamic_plan_outputs(
                    unit.drv_path,
                    unit.hdm,
                    unit.derivation,
                    unit.content_addressed,
                    unit.dynamic_plan_outputs,
                    None,
                );
            }
            staged.registered
        } else {
            register_native_dynamic_plan_units(&accepted, known_paths)?
        };
        for root_drv_path in &registered.root_drv_paths {
            self.want(root_drv_path, known_paths, true)?;
        }
        debug_assert!(registered.unit_drv_paths.len() >= accepted.plan.roots().len());
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

    // r[impl dynamic_derivation_admission.registry_boundary]
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
        let admission_policy = DynamicAdmissionLimits::default();
        let store_prefix = known_paths.store_dir().to_string();
        let mut output_names = Vec::with_capacity(outcome.outputs.len());
        let mut ready = Vec::with_capacity(outcome.outputs.len());
        for (output_name, path_info) in &outcome.outputs {
            if native_dynamic_outputs.contains(output_name)
                || !crate::dynamic::is_drv_output(&path_info.store_path, &path_info.node)
            {
                continue;
            }
            let content = builder.read_blob(&path_info.node).await?;
            let Some(parsed) =
                parse_dynamic_candidate(&content, &path_info.store_path, &store_prefix, admission_policy)?
            else {
                continue;
            };
            let validated = validate_dynamic_candidate(parsed, &store_prefix, admission_policy)?;
            let parent_facts = observe_dynamic_parent_facts(&validated, known_paths);
            let resolved = resolve_dynamic_identity(validated, &parent_facts)?;
            let existing = observe_existing_dynamic_registration(&resolved, known_paths);
            output_names.push(output_name.clone());
            ready.push(plan_dynamic_registration(resolved, existing.as_ref())?);
        }
        let selected = validate_registry_ready_batch(&ready)?;
        let mut selected_output_names = Vec::with_capacity(selected.len());
        let mut selected_ready = Vec::with_capacity(selected.len());
        for (index, (output_name, item)) in output_names.into_iter().zip(ready).enumerate() {
            let index = u32::try_from(index)
                .map_err(|_| Error::Store("dynamic admission batch index exceeds u32".to_string()))?;
            if selected.contains(&index) {
                selected_output_names.push(output_name);
                selected_ready.push(item);
            }
        }
        debug_assert_eq!(selected_ready.len(), selected.len(), "selected batch must be complete");
        self.apply_dynamic_registration_batch(outcome, known_paths, selected_output_names, selected_ready)
    }

    fn apply_dynamic_registration_batch(
        &self,
        outcome: &BuildOutcome,
        known_paths: &mut DerivationRegistry,
        output_names: Vec<String>,
        ready: Vec<RegistryReadyDynamicDerivation>,
    ) -> Result<Vec<crate::dynamic::DynamicDrv>, Error> {
        let mut discovered = Vec::with_capacity(ready.len());
        for (output_name, item) in output_names.into_iter().zip(ready) {
            known_paths.insert_registry_ready_dynamic(&item)?;
            info!(
                producer = %outcome.drv_path.name(),
                dynamic_drv = %item.drv_path().name(),
                output = %output_name,
                "admitted dynamic derivation in build output"
            );
            discovered.push(item.into_dynamic_drv(output_name));
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
            let is_spawned = self.handle_prepare_result(&drv_key, prepare_result, builder, known_paths, state).await?;
            if is_spawned {
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
        builder: &mut Builder<BServ>,
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
                self.fail_goal(drv_key, &err_msg, None, state.failed)?;
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
        all_outcomes: &mut Vec<BuildOutcome>,
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

        all_outcomes.push(outcome.clone());
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
    /// The error message is stored in `FailedGoal` for root goals.
    /// so callers can report per-package errors.
    fn fail_goal(
        &mut self,
        drv_key: &str,
        error_msg: &str,
        build_log: Option<&str>,
        failed: &mut Vec<FailedGoal>,
    ) -> Result<(), Error> {
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
                origin_drv_key: drv_key.to_string(),
                error: error_msg.to_string(),
                origin_error: error_msg.to_string(),
                build_log: build_log.map(str::to_string),
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
            self.propagate_failure(waiter_key, drv_key, &drv_name, error_msg, build_log, failed)?;
        }

        Ok(())
    }

    /// Recursively propagate dep failure to waiting goals.
    fn propagate_failure(
        &mut self,
        drv_key: &str,
        origin_drv_key: &str,
        failed_dep_name: &str,
        origin_error: &str,
        build_log: Option<&str>,
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
                origin_drv_key: origin_drv_key.to_string(),
                error: format!("dependency {failed_dep_name} failed"),
                origin_error: origin_error.to_string(),
                build_log: build_log.map(str::to_string),
            });
        }

        for waiter_key in &waiters {
            self.propagate_failure(waiter_key, origin_drv_key, &drv_name, origin_error, build_log, failed)?;
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
    use std::sync::atomic::AtomicUsize;
    use std::sync::atomic::Ordering;

    use nix_compat::derivation::Derivation;
    use nix_compat::derivation::Output;

    use super::*;
    use crate::goal::GoalState;

    #[test]
    fn failed_build_log_retention_is_bounded() {
        let retained = bounded_failed_build_log(Some("bounded"));
        let oversized = "x".repeat(MAX_FAILED_BUILD_LOG_BYTES.saturating_add(1));

        assert_eq!(retained.as_deref(), Some("bounded"));
        assert!(bounded_failed_build_log(Some(&oversized)).is_none());
        assert!(bounded_failed_build_log(None).is_none());
    }

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
        let mut all_outcomes = Vec::new();
        let mut failed = Vec::new();
        w.complete_goal(&leaf_key, outcome, &mut outcomes, &mut all_outcomes, &mut failed).unwrap();

        // Leaf should be Done.
        assert_eq!(w.registry.get(&leaf_key).unwrap().state, GoalState::Done);
        assert!(w.registry.get(&leaf_key).unwrap().waiters.is_empty());
        // Top should now be Ready (its sole dep completed).
        assert_eq!(w.registry.get(&top_key).unwrap().state, GoalState::Ready);
        // Top should be in the ready set.
        assert!(w.ready_goals.contains_key(&top_key));
        // Leaf is not a root, so root outcomes stay empty.
        assert!(outcomes.is_empty());
        assert_eq!(all_outcomes.len(), 1);
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
        let mut all_outcomes = Vec::new();
        let mut failed = Vec::new();
        w.complete_goal(&key, outcome, &mut outcomes, &mut all_outcomes, &mut failed).unwrap();

        assert_eq!(outcomes.len(), 1);
        assert_eq!(all_outcomes.len(), 1);
        assert_eq!(all_outcomes[0].drv_path, outcomes[0].drv_path);
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
        let mut all_outcomes = Vec::new();
        let mut failed = Vec::new();
        w.complete_goal(&shared_key, outcome, &mut outcomes, &mut all_outcomes, &mut failed).unwrap();

        assert_eq!(all_outcomes.len(), 1);
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
        const BUILD_LOG: &str = "bounded-build-log";
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
        w.fail_goal(&leaf_key, "leaf build error", Some(BUILD_LOG), &mut failed).unwrap();

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
        let root_failure = failed.iter().find(|failure| failure.drv_key == top_key).unwrap();
        assert_eq!(root_failure.origin_drv_key, leaf_key);
        assert_eq!(root_failure.error, "dependency mid.drv failed");
        assert_eq!(root_failure.origin_error, "leaf build error");
        assert_eq!(root_failure.build_log.as_deref(), Some(BUILD_LOG));
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
        w.fail_goal(&bad_key, "bad build error", None, &mut failed).unwrap();

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
    use snix_castore::Directory;
    use snix_castore::Node;
    use snix_castore::PathComponent;
    use snix_castore::blobservice::BlobService;
    use snix_castore::blobservice::MemoryBlobService;
    use snix_castore::directoryservice::DirectoryService;
    use snix_store::path_info::PathInfo;
    use snix_store::pathinfoservice::PathInfoService;
    use snix_store::pathinfoservice::RedbPathInfoService;
    use snix_store::pathinfoservice::RedbPathInfoServiceConfig;
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

    type SourceSliceTestBuilder = (
        Builder<MockBuildService>,
        Arc<dyn DirectoryService>,
        Arc<dyn PathInfoService>,
        Arc<std::sync::Mutex<Vec<Vec<String>>>>,
    );

    fn source_slice_test_builder(bs: &MemoryBlobService) -> SourceSliceTestBuilder {
        source_slice_test_builder_with_store_dir(bs, nix_compat::store_path::STORE_DIR)
    }

    fn source_slice_test_builder_with_store_dir(bs: &MemoryBlobService, store_dir: &str) -> SourceSliceTestBuilder {
        let ds: Arc<dyn DirectoryService> = Arc::new(tmp_ds());
        let pis: Arc<dyn PathInfoService> = Arc::new(
            RedbPathInfoService::new_temporary(
                "dynamic-source-slices".to_string(),
                RedbPathInfoServiceConfig::default(),
            )
            .unwrap(),
        );
        let (mock, calls) = MockBuildService::new(bs.clone());
        let builder = Builder::with_state_dir(
            Arc::new(bs.clone()),
            ds.clone(),
            mock,
            pis.clone(),
            PathBuf::from("/nix/store"),
            None,
            None,
            store_dir,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );
        (builder, ds, pis, calls)
    }

    struct RejectingSourceBatchPathInfo {
        inner: Arc<dyn PathInfoService>,
        attempted_batches: Arc<AtomicUsize>,
        observed_batches: mpsc::UnboundedSender<Vec<[u8; 20]>>,
    }

    #[async_trait::async_trait]
    impl PathInfoService for RejectingSourceBatchPathInfo {
        async fn get(&self, digest: [u8; 20]) -> Result<Option<PathInfo>, snix_store::pathinfoservice::Error> {
            self.inner.get(digest).await
        }

        async fn put(&self, path_info: PathInfo) -> Result<PathInfo, snix_store::pathinfoservice::Error> {
            self.inner.put(path_info).await
        }

        async fn put_batch_atomic(
            &self,
            path_infos: Vec<PathInfo>,
        ) -> Result<Vec<PathInfo>, snix_store::pathinfoservice::Error> {
            self.attempted_batches.fetch_add(1, Ordering::SeqCst);
            self.observed_batches
                .send(path_infos.iter().map(|info| *info.store_path.digest()).collect())
                .expect("batch observer must remain available");
            Err(std::io::Error::other("injected atomic PathInfo batch rejection").into())
        }

        fn list(&self) -> futures::stream::BoxStream<'static, Result<PathInfo, snix_store::pathinfoservice::Error>> {
            self.inner.list()
        }
    }

    async fn stored_test_directory(
        ds: &Arc<dyn DirectoryService>,
        children: impl IntoIterator<Item = (PathComponent, Node)>,
    ) -> Node {
        let directory = Directory::try_from_iter(children).unwrap();
        let size = directory.size();
        let digest = ds.put(directory).await.unwrap();
        Node::Directory { digest, size }
    }

    async fn source_slice_tree(bs: &MemoryBlobService, ds: &Arc<dyn DirectoryService>, outside: &[u8]) -> Node {
        let leaf = put_test_blob(bs, b"slice content stays the same").await;
        let package = stored_test_directory(ds, [("file.txt".try_into().unwrap(), leaf)]).await;
        let packages = stored_test_directory(ds, [
            ("a".try_into().unwrap(), package.clone()),
            ("b".try_into().unwrap(), package),
            ("link".try_into().unwrap(), Node::Symlink {
                target: snix_castore::SymlinkTarget::try_from("a").unwrap(),
            }),
        ])
        .await;
        let changed = put_test_blob(bs, outside).await;
        stored_test_directory(ds, [
            ("packages".try_into().unwrap(), packages),
            ("outside.txt".try_into().unwrap(), changed),
        ])
        .await
    }

    fn v2_plan_for_source_slices(expected_digest: &str, second_subpath: &str, second_digest: &str) -> Vec<u8> {
        let mut plan: serde_json::Value = serde_json::from_slice(&native_plan_with_placeholders_bytes()).unwrap();
        plan["schema"] = serde_json::json!("mantle-plan-v2");
        plan["sources"] = serde_json::json!([
            {
                "id": "src.main",
                "producer_output": "sources",
                "subpath": "packages/a",
                "store_name": "unchanged-package",
                "nar_blake3": expected_digest,
            },
            {
                "id": "src.other",
                "producer_output": "sources",
                "subpath": second_subpath,
                "store_name": "unchanged-package",
                "nar_blake3": second_digest,
            },
        ]);
        serde_json::to_vec(&plan).unwrap()
    }

    async fn source_slice_plan_outcome(
        bs: &MemoryBlobService,
        plan_bytes: &[u8],
        source_root: Node,
        producer_name: &str,
        known_paths: &mut DerivationRegistry,
    ) -> (StorePath<String>, BuildOutcome) {
        let (producer, _) = build_and_register_multi(producer_name, &["out", "plan", "sources"], &[], known_paths);
        declare_native_plan_output(known_paths, &producer, "plan");
        let plan_node = put_test_blob(bs, plan_bytes).await;
        let outcome = BuildOutcome {
            drv_path: producer.clone(),
            outputs: BTreeMap::from([
                ("plan".to_string(), test_path_info("source-slice-plan", plan_node)),
                ("sources".to_string(), test_path_info("source-slice-root", source_root)),
            ]),
            substitutions: BTreeMap::new(),
            cached: false,
            log: None,
        };
        (producer, outcome)
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

    fn native_plan_with_placeholders_bytes() -> Vec<u8> {
        format!(
            r#"{{
  "schema": "mantle-plan-v1",
  "producer": {{ "logical_name": "producer", "goal_hint": null }},
  "sources": [{{ "id": "src.main", "path": "{TEST_NATIVE_PLAN_STORE_PATH}", "nar_blake3": null }}],
  "units": [
    {{
      "id": "unit.dep",
      "derivation": {{
        "name": "unit-dep",
        "builder": "{TEST_NATIVE_PLAN_STORE_PATH}/bin/builder",
        "system": "x86_64-linux",
        "args": ["--source={{{{mantle-source:src.main}}}}"],
        "outputs": ["out"],
        "env": {{ "SOURCE": "{{{{mantle-source:src.main}}}}" }},
        "inputs": [{{ "kind": "source", "source": "src.main" }}],
        "fixed_output": null,
        "addressing_mode": "input-addressed",
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
        "args": ["--dependency={{{{mantle-unit-output:unit.dep:out}}}}"],
        "outputs": ["out"],
        "env": {{ "DEPENDENCY": "{{{{mantle-unit-output:unit.dep:out}}}}" }},
        "inputs": [{{ "kind": "unit_output", "unit": "unit.dep", "output": "out" }}],
        "fixed_output": null,
        "addressing_mode": "input-addressed",
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
        let abs = sp.to_absolute_path_with_prefix(kp.store_dir());
        let entry = kp.get_by_drv_path_mut(&abs).unwrap();
        entry.dynamic_plan_outputs = vec![output_name.to_string()];
    }

    #[test]
    fn native_dynamic_registration_resolves_declared_placeholders_and_rejects_unknown_ca_paths() {
        let canonical = decode_validated_plan_v1(&native_plan_with_placeholders_bytes(), "/nix/store").unwrap();
        let sources = dynamic_sources_by_id(&canonical.plan.sources, "/nix/store").unwrap();
        let dep = canonical.plan.units.iter().find(|unit| unit.id.as_str() == "unit.dep").unwrap();
        let root = canonical.plan.units.iter().find(|unit| unit.id.as_str() == "unit.root").unwrap();
        let mut registry = DerivationRegistry::default();
        let mut registered = BTreeMap::new();

        let dep_drv = register_native_dynamic_unit(dep, &sources, &registered, &mut registry, "/nix/store").unwrap();
        registered.insert(dep.id.clone(), dep_drv.clone());
        let dep_output = registry.get_output_path(&dep_drv.to_absolute_path(), "out").unwrap().to_absolute_path();
        let root_drv = register_native_dynamic_unit(root, &sources, &registered, &mut registry, "/nix/store").unwrap();
        let root_entry = registry.get_by_drv_path(&root_drv.to_absolute_path()).unwrap();
        let dep_entry = registry.get_by_drv_path(&dep_drv.to_absolute_path()).unwrap();

        assert_eq!(dep_entry.derivation.environment["SOURCE"].as_slice(), TEST_NATIVE_PLAN_STORE_PATH.as_bytes());
        assert_eq!(root_entry.derivation.environment["DEPENDENCY"].as_slice(), dep_output.as_bytes());
        assert_eq!(root_entry.derivation.arguments, vec![format!("--dependency={dep_output}")]);
        assert!(!root_entry.derivation.arguments[0].contains("{{mantle-"));

        let mut ca_dep = dep.clone();
        ca_dep.derivation.addressing_mode = AddressingMode::ContentAddressed;
        let mut ca_registry = DerivationRegistry::default();
        let ca_drv =
            register_native_dynamic_unit(&ca_dep, &sources, &BTreeMap::new(), &mut ca_registry, "/nix/store").unwrap();
        let ca_registered = BTreeMap::from([(ca_dep.id.clone(), ca_drv)]);
        let error =
            register_native_dynamic_unit(root, &sources, &ca_registered, &mut ca_registry, "/nix/store").unwrap_err();
        assert!(error.to_string().contains("requires a statically known path"));
        assert!(error.to_string().contains("unit.dep:out"));
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
        let mut all_outcomes = Vec::new();
        let mut failed = Vec::new();
        let mut native_dynamic_plans = Vec::new();
        let mut state = WorkerLoopState {
            sem: Arc::new(Semaphore::new(usize::try_from(TEST_MAX_JOB_BUDGET).unwrap())),
            join_set: JoinSet::new(),
            pending_meta: HashMap::new(),
            outcomes: &mut outcomes,
            all_outcomes: &mut all_outcomes,
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
        let mut all_outcomes = Vec::new();
        let mut failed = Vec::new();
        let mut native_dynamic_plans = Vec::new();
        let mut state = WorkerLoopState {
            sem: Arc::new(Semaphore::new(usize::try_from(TEST_MAX_JOB_BUDGET).unwrap())),
            join_set: JoinSet::new(),
            pending_meta: HashMap::new(),
            outcomes: &mut outcomes,
            all_outcomes: &mut all_outcomes,
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

    #[cfg(debug_assertions)]
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

    #[cfg(not(debug_assertions))]
    #[test]
    fn run_empty_worker_panics() {
        eprintln!("SKIP run_empty_worker_panics: debug assertions are disabled in release builds");
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
    async fn native_v2_custom_store_prefix_admits_declared_producer_output() {
        let store_dir = "/mantle/store";
        let bs = MemoryBlobService::default();
        let (mut builder, ds, _, _) = source_slice_test_builder_with_store_dir(&bs, store_dir);
        let source_root = source_slice_tree(&bs, &ds, b"outside").await;
        let observed = builder.observe_source_slice(&source_root, "packages/a").await.unwrap();
        let digest = data_encoding::HEXLOWER.encode(&observed.nar_blake3());
        let plan = String::from_utf8(v2_plan_for_source_slices(&digest, "packages/b", &digest))
            .unwrap()
            .replace("/nix/store/", "/mantle/store/");
        let mut known_paths = DerivationRegistry::new(store_dir);
        let (producer, outcome) =
            source_slice_plan_outcome(&bs, plan.as_bytes(), source_root, "prefixed-producer", &mut known_paths).await;
        let worker = Worker::new(1);
        let scan = worker
            .scan_native_dynamic_plans(
                &producer.to_absolute_path(),
                &outcome,
                &builder,
                store_dir,
                &BTreeSet::from(["plan".to_string()]),
            )
            .await
            .unwrap();
        let mut accepted = scan.accepted.into_iter().next().expect("valid v2 plan");
        let mut prepared = None;
        let rejection = worker
            .admit_v2_source_slices(&mut accepted, &outcome, &known_paths, &mut builder, &mut prepared)
            .await
            .unwrap();
        assert!(rejection.is_none(), "{rejection:?}");
        assert!(accepted
            .source_slices
            .iter()
            .all(|slice| slice.disposition == "admitted" && slice.admitted_store_path.is_some()));
        assert!(prepared.is_some());
    }

    #[tokio::test]
    async fn native_v2_slice_publication_survives_two_producer_reruns() {
        let bs = MemoryBlobService::default();
        let (mut builder, ds, pis, _) = source_slice_test_builder(&bs);
        let mut previous: Option<(Node, StorePath<String>, StorePath<String>)> = None;
        for (run, outside) in [b"first outside".as_slice(), b"second outside".as_slice()].into_iter().enumerate() {
            let source_root = source_slice_tree(&bs, &ds, outside).await;
            let observed = builder.observe_source_slice(&source_root, "packages/a").await.unwrap();
            let nar_blake3 = data_encoding::HEXLOWER.encode(&observed.nar_blake3());
            let plan = v2_plan_for_source_slices(&nar_blake3, "packages/b", &nar_blake3);
            let mut known_paths = DerivationRegistry::default();
            let (producer, outcome) = source_slice_plan_outcome(
                &bs,
                &plan,
                source_root.clone(),
                &format!("source-producer-{run}"),
                &mut known_paths,
            )
            .await;
            let mut worker = Worker::new(1);
            let scan = worker
                .scan_native_dynamic_plans(
                    &producer.to_absolute_path(),
                    &outcome,
                    &builder,
                    known_paths.store_dir(),
                    &BTreeSet::from(["plan".to_string()]),
                )
                .await
                .unwrap();
            let mut accepted = scan.accepted.into_iter().next().expect("valid v2 plan");
            let mut staged = None;
            assert!(
                worker
                    .admit_v2_source_slices(&mut accepted, &outcome, &known_paths, &mut builder, &mut staged)
                    .await
                    .unwrap()
                    .is_none()
            );
            let rows = &accepted.source_slices;
            assert_eq!(rows.iter().map(|row| row.source_id.as_str()).collect::<Vec<_>>(), ["src.main", "src.other"]);
            assert!(
                rows.iter().all(|row| row.disposition == "admitted"
                    && row.observed_nar_blake3.as_deref() == Some(nar_blake3.as_str()))
            );
            let source_path = rows[0].admitted_store_path.clone().unwrap();
            assert_eq!(rows[1].admitted_store_path.as_ref(), Some(&source_path));
            let signed = pis.get(*source_path.digest()).await.unwrap().expect("signed source PathInfo committed");
            assert_eq!(signed.store_path, source_path);
            assert_eq!(signed.ca, Some(CAHash::Nar(NixHash::Sha256(observed.nar_sha256()))));
            assert!(
                crate::signing::verify_pathinfo_signatures_with_store_dir(&signed, &test_trusted_keys(), "/nix/store",)
                    .is_trusted()
            );
            let dependency = staged.as_ref().expect("V2 registration was staged").registered.unit_drv_paths
                [&UnitId::new("unit.dep".to_string()).unwrap()]
                .clone();
            worker.register_accepted_native_dynamic_plan(accepted, staged, &mut known_paths).unwrap();
            let dependency_entry = known_paths.get_by_drv_path(&dependency.to_absolute_path()).unwrap();
            assert!(dependency_entry.derivation.input_sources.contains(&source_path));
            let source_absolute = source_path.to_absolute_path();
            assert!(dependency_entry.derivation.arguments.iter().any(|arg| arg.contains(&source_absolute)));
            if let Some((prior_root, prior_source, prior_unit)) = &previous {
                assert_ne!(source_root, *prior_root, "outside bytes must change producer output");
                assert_eq!(&source_path, prior_source, "slice path must not depend on producer output");
                assert_eq!(&dependency, prior_unit, "unit identity must depend only on its slice and unchanged inputs");
            }
            previous = Some((source_root, source_path, dependency));
            let result = worker.run(&mut builder, &mut known_paths).await.unwrap();
            assert!(result.failed.is_empty());
        }
    }

    #[tokio::test]
    async fn native_v2_late_unit_failure_keeps_slices_unpublished() {
        let bs = MemoryBlobService::default();
        let (mut builder, ds, pis, _) = source_slice_test_builder(&bs);
        let source_root = source_slice_tree(&bs, &ds, b"outside").await;
        let observed = builder.observe_source_slice(&source_root, "packages/a").await.unwrap();
        let nar_blake3 = data_encoding::HEXLOWER.encode(&observed.nar_blake3());
        let ca = CAHash::Nar(NixHash::Sha256(observed.nar_sha256()));
        let expected_path: StorePath<String> = nix_compat::store_path::build_ca_path_with_store_dir(
            "unchanged-package",
            &ca,
            Vec::<&str>::new(),
            false,
            "/nix/store",
        )
        .unwrap();
        let mut plan: serde_json::Value =
            serde_json::from_slice(&v2_plan_for_source_slices(&nar_blake3, "packages/b", &nar_blake3)).unwrap();
        plan["units"][1]["derivation"]["name"] = serde_json::json!("x".repeat(220));
        let plan = serde_json::to_vec(&plan).unwrap();
        let mut known_paths = DerivationRegistry::default();
        let (producer, outcome) =
            source_slice_plan_outcome(&bs, &plan, source_root, "late-unit-producer", &mut known_paths).await;
        let mut worker = Worker::new(1);
        worker.want(&producer, &known_paths, true).unwrap();
        let registry_before = known_paths.len();
        let goals_before = worker.registry.len();
        let ready_before = worker.ready_goals.clone();
        let epoch_before = worker.scheduling_epoch;

        let mut scan = worker
            .scan_native_dynamic_plans(
                &producer.to_absolute_path(),
                &outcome,
                &builder,
                known_paths.store_dir(),
                &BTreeSet::from(["plan".to_string()]),
            )
            .await
            .unwrap();
        let accepted = std::mem::take(&mut scan.accepted);
        for mut plan in accepted {
            let mut staged = None;
            if let Some(rejection) = worker
                .admit_v2_source_slices(&mut plan, &outcome, &known_paths, &mut builder, &mut staged)
                .await
                .unwrap()
            {
                scan.rejected.push(rejection);
            } else {
                scan.accepted.push(plan);
            }
        }
        let reports = native_plan_reports_from_scan(&scan);
        assert!(scan.accepted.is_empty(), "late unit must reject before registration");
        assert!(
            pis.get(*expected_path.digest()).await.unwrap().is_none(),
            "late unit published signed slice PathInfo"
        );
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].scheduler_action, "rejected");
        assert!(reports[0].rejection_reason.as_deref().unwrap_or("").starts_with("invalid-plan:"));
        assert_eq!(reports[0].source_slices.len(), 2);
        assert!(reports[0].source_slices.iter().all(|row| row.admitted_store_path.is_none()));
        assert_eq!(known_paths.len(), registry_before, "no earlier dependency may be registered");
        assert_eq!(worker.registry.len(), goals_before);
        assert_eq!(worker.ready_goals, ready_before);
        assert_eq!(worker.scheduling_epoch, epoch_before);
        assert!(worker.priority_decisions.is_empty());
    }

    #[tokio::test]
    async fn native_v2_goal_capacity_rejects_before_slice_publication() {
        let bs = MemoryBlobService::default();
        let (mut builder, ds, pis, _) = source_slice_test_builder(&bs);
        let source_root = source_slice_tree(&bs, &ds, b"outside").await;
        let observed = builder.observe_source_slice(&source_root, "packages/a").await.unwrap();
        let digest = data_encoding::HEXLOWER.encode(&observed.nar_blake3());
        let expected_path: StorePath<String> = build_ca_path_with_store_dir(
            "unchanged-package",
            &CAHash::Nar(NixHash::Sha256(observed.nar_sha256())),
            Vec::<&str>::new(),
            false,
            "/nix/store",
        )
        .unwrap();
        let mut known_paths = DerivationRegistry::default();
        let (producer, outcome) = source_slice_plan_outcome(
            &bs,
            &v2_plan_for_source_slices(&digest, "packages/b", &digest),
            source_root,
            "v2-goal-capacity-producer",
            &mut known_paths,
        )
        .await;
        let mut worker = Worker::new(1);
        worker.want(&producer, &known_paths, true).unwrap();
        let existing_derivation = known_paths.get_by_drv_path(&producer.to_absolute_path()).unwrap().derivation.clone();
        for index in 0..(MAX_GOALS - worker.registry.len()) {
            let path = fake_sp(&format!("capacity-{index:05}.drv"));
            worker
                .registry
                .insert(path.to_absolute_path(), Goal::new(path, existing_derivation.clone()))
                .unwrap();
        }
        assert_eq!(worker.registry.len(), MAX_GOALS);
        let registry_before = known_paths.len();
        let ready_before = worker.ready_goals.clone();
        let epoch_before = worker.scheduling_epoch;
        let scan = worker
            .scan_native_dynamic_plans(
                &producer.to_absolute_path(),
                &outcome,
                &builder,
                known_paths.store_dir(),
                &BTreeSet::from(["plan".to_string()]),
            )
            .await
            .unwrap();
        let mut accepted = scan.accepted.into_iter().next().expect("valid V2 plan before goal admission");
        let rejection = worker
            .admit_v2_source_slices(&mut accepted, &outcome, &known_paths, &mut builder, &mut None)
            .await
            .unwrap()
            .expect("new V2 root must reject when goal registry is full");
        let report = native_plan_report_from_rejection(&rejection);
        assert_eq!(rejection.kind, NativeDynamicPlanRejectionKind::InvalidPlan);
        assert!(rejection.detail.contains("goal registry at capacity"));
        assert!(
            pis.get(*expected_path.digest()).await.unwrap().is_none(),
            "full goal registry published signed slice PathInfo"
        );
        assert_eq!(report.scheduler_action, "rejected");
        assert!(report.rejection_reason.as_deref().unwrap().starts_with("invalid-plan:"));
        assert_eq!(report.source_slices.len(), 2);
        assert!(report.source_slices.iter().all(|row| row.admitted_store_path.is_none()));
        assert_eq!(known_paths.len(), registry_before, "V2 units must not register before goal admission");
        assert_eq!(worker.registry.len(), MAX_GOALS);
        assert_eq!(worker.registry.get(&producer.to_absolute_path()).unwrap().state, GoalState::Ready);
        assert_eq!(worker.ready_goals, ready_before);
        assert_eq!(worker.scheduling_epoch, epoch_before);
        assert!(worker.priority_decisions.is_empty());
    }

    #[tokio::test]
    async fn native_v2_registry_collision_rejects_before_slice_publication() {
        let bs = MemoryBlobService::default();
        let (mut builder, ds, pis, _) = source_slice_test_builder(&bs);
        let source_root = source_slice_tree(&bs, &ds, b"outside").await;
        let observed = builder.observe_source_slice(&source_root, "packages/a").await.unwrap();
        let digest = data_encoding::HEXLOWER.encode(&observed.nar_blake3());
        let expected_path = build_ca_path_with_store_dir(
            "unchanged-package",
            &CAHash::Nar(NixHash::Sha256(observed.nar_sha256())),
            Vec::<&str>::new(),
            false,
            "/nix/store",
        )
        .unwrap();
        let mut known_paths = DerivationRegistry::default();
        let (producer, outcome) = source_slice_plan_outcome(
            &bs,
            &v2_plan_for_source_slices(&digest, "packages/b", &digest),
            source_root,
            "v2-registry-collision-producer",
            &mut known_paths,
        )
        .await;
        let mut worker = Worker::new(1);
        worker.want(&producer, &known_paths, true).unwrap();
        let scan = worker
            .scan_native_dynamic_plans(
                &producer.to_absolute_path(),
                &outcome,
                &builder,
                known_paths.store_dir(),
                &BTreeSet::from(["plan".to_string()]),
            )
            .await
            .unwrap();
        let mut accepted = scan.accepted.into_iter().next().expect("valid V2 plan before registry preflight");
        let projected = [("src.main", "packages/a"), ("src.other", "packages/b")]
            .into_iter()
            .map(|(id, subpath)| NativeDynamicSourceSliceReport {
                source_id: id.to_string(),
                producer_output: "sources".to_string(),
                subpath: subpath.to_string(),
                declared_nar_blake3: digest.clone(),
                observed_nar_blake3: Some(digest.clone()),
                admitted_store_path: Some(expected_path.clone()),
                disposition: "not-admitted".to_string(),
            })
            .collect::<Vec<_>>();
        let staged = prepare_v2_registration(&accepted, &projected, &known_paths, &worker).unwrap();
        let colliding_unit = staged.staged.values().next().expect("at least one prospective unit");
        let colliding_path = colliding_unit.drv_path.to_absolute_path();
        let colliding_hdm = [0x5a; 32];
        assert_ne!(colliding_unit.hdm, colliding_hdm);
        known_paths.insert_with_dynamic_plan_outputs(
            colliding_unit.drv_path.clone(),
            colliding_hdm,
            colliding_unit.derivation.clone(),
            colliding_unit.content_addressed,
            colliding_unit.dynamic_plan_outputs.clone(),
            None,
        );
        let registry_before = known_paths.len();
        let goals_before = worker.registry.len();
        let ready_before = worker.ready_goals.clone();
        let epoch_before = worker.scheduling_epoch;

        let rejection = worker
            .admit_v2_source_slices(&mut accepted, &outcome, &known_paths, &mut builder, &mut None)
            .await
            .unwrap()
            .expect("conflicting V2 derivation must reject before publication");
        let report = native_plan_report_from_rejection(&rejection);
        assert_eq!(rejection.kind, NativeDynamicPlanRejectionKind::InvalidPlan);
        assert!(rejection.detail.contains("collides with an existing registry entry"));
        assert!(pis.get(*expected_path.digest()).await.unwrap().is_none());
        assert_eq!(known_paths.len(), registry_before);
        assert_eq!(known_paths.get_hdm_by_drv_path(&colliding_path), Some(colliding_hdm));
        assert_eq!(worker.registry.len(), goals_before);
        assert_eq!(worker.ready_goals, ready_before);
        assert_eq!(worker.scheduling_epoch, epoch_before);
        assert!(worker.priority_decisions.is_empty());
        assert_eq!(report.scheduler_action, "rejected");
        assert_eq!(report.source_slices.len(), 2);
        assert!(report.source_slices.iter().all(|row| row.admitted_store_path.is_none()));
    }

    #[tokio::test]
    async fn native_v2_registry_capacity_rejects_before_slice_publication() {
        let bs = MemoryBlobService::default();
        let (mut builder, ds, pis, _) = source_slice_test_builder(&bs);
        let source_root = source_slice_tree(&bs, &ds, b"outside").await;
        let observed = builder.observe_source_slice(&source_root, "packages/a").await.unwrap();
        let digest = data_encoding::HEXLOWER.encode(&observed.nar_blake3());
        let expected_path: StorePath<String> = build_ca_path_with_store_dir(
            "unchanged-package",
            &CAHash::Nar(NixHash::Sha256(observed.nar_sha256())),
            Vec::<&str>::new(),
            false,
            "/nix/store",
        )
        .unwrap();
        let mut known_paths = DerivationRegistry::default();
        let (producer, outcome) = source_slice_plan_outcome(
            &bs,
            &v2_plan_for_source_slices(&digest, "packages/b", &digest),
            source_root,
            "v2-registry-capacity-producer",
            &mut known_paths,
        )
        .await;
        let mut worker = Worker::new(1);
        worker.want(&producer, &known_paths, true).unwrap();
        let template = known_paths.get_by_drv_path(&producer.to_absolute_path()).unwrap().derivation.clone();
        for index in 0..(MAX_ENTRIES as usize - 1 - known_paths.len() as usize) {
            known_paths.insert_with_dynamic_plan_outputs(
                fake_sp(&format!("registry-capacity-{index:05}.drv")),
                [index as u8; 32],
                template.clone(),
                false,
                Vec::new(),
                None,
            );
        }
        assert_eq!(known_paths.len(), MAX_ENTRIES - 1);
        let registry_before = known_paths.len();
        let goals_before = worker.registry.len();
        let ready_before = worker.ready_goals.clone();
        let epoch_before = worker.scheduling_epoch;
        let scan = worker
            .scan_native_dynamic_plans(
                &producer.to_absolute_path(),
                &outcome,
                &builder,
                known_paths.store_dir(),
                &BTreeSet::from(["plan".to_string()]),
            )
            .await
            .unwrap();
        let mut accepted = scan.accepted.into_iter().next().expect("valid V2 plan before registry capacity");
        let rejection = worker
            .admit_v2_source_slices(&mut accepted, &outcome, &known_paths, &mut builder, &mut None)
            .await
            .unwrap()
            .expect("two new V2 units cannot fit one remaining registry slot");
        let report = native_plan_report_from_rejection(&rejection);
        assert_eq!(rejection.kind, NativeDynamicPlanRejectionKind::InvalidPlan);
        assert!(rejection.detail.contains("dynamic registry exceeds MAX_ENTRIES"));
        assert!(pis.get(*expected_path.digest()).await.unwrap().is_none());
        assert_eq!(known_paths.len(), registry_before);
        assert_eq!(worker.registry.len(), goals_before);
        assert_eq!(worker.registry.get(&producer.to_absolute_path()).unwrap().state, GoalState::Ready);
        assert_eq!(worker.ready_goals, ready_before);
        assert_eq!(worker.scheduling_epoch, epoch_before);
        assert!(worker.priority_decisions.is_empty());
        assert_eq!(report.scheduler_action, "rejected");
        assert_eq!(report.source_slices.len(), 2);
        assert!(report.source_slices.iter().all(|row| row.admitted_store_path.is_none()));
    }

    #[tokio::test]
    async fn native_v2_atomic_pathinfo_batch_rejection_keeps_both_slices_unpublished() {
        let bs = MemoryBlobService::default();
        let ds: Arc<dyn DirectoryService> = Arc::new(tmp_ds());
        let persisted: Arc<dyn PathInfoService> = Arc::new(
            RedbPathInfoService::new_temporary(
                "dynamic-source-slices-injected-batch-failure".to_string(),
                RedbPathInfoServiceConfig::default(),
            )
            .unwrap(),
        );
        let attempted_batches = Arc::new(AtomicUsize::new(0));
        let (observed_batches_tx, mut observed_batches_rx) = mpsc::unbounded_channel();
        let injecting: Arc<dyn PathInfoService> = Arc::new(RejectingSourceBatchPathInfo {
            inner: persisted.clone(),
            attempted_batches: attempted_batches.clone(),
            observed_batches: observed_batches_tx,
        });
        let (mock, _) = MockBuildService::new(bs.clone());
        let mut builder = Builder::with_state_dir(
            Arc::new(bs.clone()),
            ds.clone(),
            mock,
            injecting,
            PathBuf::from("/nix/store"),
            None,
            None,
            nix_compat::store_path::STORE_DIR,
            test_keypair(),
            test_trusted_keys(),
            true,
            false,
        );
        let first_leaf = put_test_blob(&bs, b"first distinct slice").await;
        let second_leaf = put_test_blob(&bs, b"second distinct slice").await;
        let first = stored_test_directory(&ds, [("file.txt".try_into().unwrap(), first_leaf)]).await;
        let second = stored_test_directory(&ds, [("file.txt".try_into().unwrap(), second_leaf)]).await;
        let packages = stored_test_directory(&ds, [
            ("a".try_into().unwrap(), first),
            ("b".try_into().unwrap(), second),
        ])
        .await;
        let source_root = stored_test_directory(&ds, [("packages".try_into().unwrap(), packages)]).await;
        let observed_first = builder.observe_source_slice(&source_root, "packages/a").await.unwrap();
        let observed_second = builder.observe_source_slice(&source_root, "packages/b").await.unwrap();
        let first_digest = data_encoding::HEXLOWER.encode(&observed_first.nar_blake3());
        let second_digest = data_encoding::HEXLOWER.encode(&observed_second.nar_blake3());
        assert_ne!(first_digest, second_digest);
        let expected_paths: [StorePath<String>; 2] = [
            ("unchanged-package", observed_first.nar_sha256()),
            ("other-package", observed_second.nar_sha256()),
        ]
        .map(|(name, nar_sha256)| {
            build_ca_path_with_store_dir(
                name,
                &CAHash::Nar(NixHash::Sha256(nar_sha256)),
                Vec::<&str>::new(),
                false,
                "/nix/store",
            )
            .unwrap()
        });
        assert_ne!(expected_paths[0], expected_paths[1]);
        let mut plan: serde_json::Value =
            serde_json::from_slice(&v2_plan_for_source_slices(&first_digest, "packages/b", &second_digest)).unwrap();
        plan["sources"][1]["store_name"] = serde_json::json!("other-package");
        let mut known_paths = DerivationRegistry::default();
        let (producer, outcome) = source_slice_plan_outcome(
            &bs,
            &serde_json::to_vec(&plan).unwrap(),
            source_root,
            "v2-batch-failure-producer",
            &mut known_paths,
        )
        .await;
        let mut worker = Worker::new(1);
        worker.want(&producer, &known_paths, true).unwrap();
        let registry_before = known_paths.len();
        let goals_before = worker.registry.len();
        let ready_before = worker.ready_goals.clone();
        let epoch_before = worker.scheduling_epoch;
        let scan = worker
            .scan_native_dynamic_plans(
                &producer.to_absolute_path(),
                &outcome,
                &builder,
                known_paths.store_dir(),
                &BTreeSet::from(["plan".to_string()]),
            )
            .await
            .unwrap();
        let mut accepted = scan.accepted.into_iter().next().expect("valid V2 plan before batch publication");
        let rejection = worker
            .admit_v2_source_slices(&mut accepted, &outcome, &known_paths, &mut builder, &mut None)
            .await
            .unwrap()
            .expect("injected atomic PathInfo batch failure must reject the whole plan");
        let report = native_plan_report_from_rejection(&rejection);
        assert_eq!(attempted_batches.load(Ordering::SeqCst), 1, "failure must reach the batch boundary");
        assert_eq!(
            observed_batches_rx.try_recv().unwrap(),
            expected_paths.iter().map(|path| *path.digest()).collect::<Vec<_>>()
        );
        assert!(observed_batches_rx.try_recv().is_err(), "only one atomic batch may be attempted");
        assert_eq!(rejection.kind, NativeDynamicPlanRejectionKind::SlicePublication);
        assert!(rejection.detail.contains("verified-source-batch-publication-rejected"));
        for path in &expected_paths {
            assert!(persisted.get(*path.digest()).await.unwrap().is_none(), "{path} was partially published");
        }
        assert_eq!(known_paths.len(), registry_before);
        assert_eq!(worker.registry.len(), goals_before);
        assert_eq!(worker.registry.get(&producer.to_absolute_path()).unwrap().state, GoalState::Ready);
        assert_eq!(worker.ready_goals, ready_before);
        assert_eq!(worker.scheduling_epoch, epoch_before);
        assert!(worker.priority_decisions.is_empty());
        assert_eq!(report.scheduler_action, "rejected");
        assert_eq!(report.source_slices.len(), 2);
        assert!(report.source_slices.iter().all(|row| row.admitted_store_path.is_none()));
    }

    #[tokio::test]
    async fn native_v2_slice_failure_leaves_store_registry_goals_scheduler_and_report_unadmitted() {
        let bs = MemoryBlobService::default();
        let (mut builder, ds, pis, _) = source_slice_test_builder(&bs);
        let source_root = source_slice_tree(&bs, &ds, b"outside").await;
        let observed = builder.observe_source_slice(&source_root, "packages/a").await.unwrap();
        let actual_digest = data_encoding::HEXLOWER.encode(&observed.nar_blake3());
        let ca = CAHash::Nar(NixHash::Sha256(observed.nar_sha256()));
        let expected_path: StorePath<String> = nix_compat::store_path::build_ca_path_with_store_dir(
            "unchanged-package",
            &ca,
            Vec::<&str>::new(),
            false,
            "/nix/store",
        )
        .unwrap();
        for (second_subpath, second_digest, reason, producer_output) in [
            ("packages/b", "0".repeat(64), "slice-digest-mismatch", "sources"),
            ("packages/missing", actual_digest.clone(), "slice-absent", "sources"),
            ("packages/link", actual_digest.clone(), "slice-symlink-traversal", "sources"),
            ("packages/link/file.txt", actual_digest.clone(), "slice-symlink-traversal", "sources"),
            ("packages/a", actual_digest.clone(), "slice-output-undeclared", "plan"),
            ("packages/a", actual_digest.clone(), "slice-output-undeclared", "rogue"),
        ] {
            let mut plan: serde_json::Value =
                serde_json::from_slice(&v2_plan_for_source_slices(&actual_digest, second_subpath, &second_digest))
                    .unwrap();
            plan["sources"][1]["producer_output"] = serde_json::json!(producer_output);
            let plan = serde_json::to_vec(&plan).unwrap();
            let mut known_paths = DerivationRegistry::default();
            let (producer, mut outcome) = source_slice_plan_outcome(
                &bs,
                &plan,
                source_root.clone(),
                "source-rejection-producer",
                &mut known_paths,
            )
            .await;
            if producer_output == "rogue" {
                outcome
                    .outputs
                    .insert("rogue".to_string(), test_path_info("undeclared-rogue-sources", source_root.clone()));
            }
            let mut worker = Worker::new(1);
            worker.want(&producer, &known_paths, true).unwrap();
            let registry_before = known_paths.len();
            let goal_before = worker.registry.len();
            let ready_before = worker.ready_goals.clone();
            let epoch_before = worker.scheduling_epoch;
            let scan = worker
                .scan_native_dynamic_plans(
                    &producer.to_absolute_path(),
                    &outcome,
                    &builder,
                    known_paths.store_dir(),
                    &BTreeSet::from(["plan".to_string()]),
                )
                .await
                .unwrap();
            let mut accepted = scan.accepted.into_iter().next().expect("valid v2 plan before content admission");
            let rejected = worker
                .admit_v2_source_slices(&mut accepted, &outcome, &known_paths, &mut builder, &mut None)
                .await
                .unwrap()
                .expect("source mismatch must reject the whole plan");
            let rows = native_plan_reports_from_scan(&NativeDynamicPlanScan {
                accepted: Vec::new(),
                rejected: vec![rejected],
            });
            assert_eq!(rows[0].scheduler_action, "rejected");
            assert!(rows[0].rejection_reason.as_deref().unwrap().starts_with(reason));
            assert_eq!(rows[0].source_slices.iter().map(|row| row.source_id.as_str()).collect::<Vec<_>>(), [
                "src.main",
                "src.other"
            ]);
            assert!(rows[0].source_slices.iter().all(|row| row.admitted_store_path.is_none()));
            assert_eq!(rows[0].source_slices[1].subpath, second_subpath);
            assert_eq!(rows[0].source_slices[1].disposition, reason);
            assert_eq!(rows[0].source_slices[1].producer_output, producer_output);
            assert!(
                pis.get(*expected_path.digest()).await.unwrap().is_none(),
                "failed batch must publish no first slice"
            );
            assert_eq!(known_paths.len(), registry_before);
            assert_eq!(worker.registry.len(), goal_before);
            assert_eq!(worker.ready_goals, ready_before);
            assert_eq!(worker.scheduling_epoch, epoch_before);
            assert!(worker.priority_decisions.is_empty());
        }
    }

    #[tokio::test]
    async fn native_v2_slice_aggregate_byte_limit_preserves_scheduler_and_publication() {
        let bs = MemoryBlobService::default();
        let (mut builder, ds, pis, _) = source_slice_test_builder(&bs);
        let payload = vec![b'x'; 4 * 1024 * 1024];
        let large_file = put_test_blob(&bs, &payload).await;
        let source_root = stored_test_directory(&ds, [("large".try_into().unwrap(), large_file)]).await;
        let observation = builder.observe_source_slice(&source_root, "large").await.unwrap();
        assert!(observation.nar_size() > payload.len() as u64, "NAR headers push 256 slices over 1 GiB");
        let digest = data_encoding::HEXLOWER.encode(&observation.nar_blake3());
        let mut plan: serde_json::Value = serde_json::from_slice(&valid_native_plan_bytes()).unwrap();
        plan["schema"] = serde_json::json!("mantle-plan-v2");
        plan["sources"] = (0..crate::dynamic_plan::MAX_PLAN_SLICES)
            .map(|index| {
                serde_json::json!({
                    "id": format!("src.{index:03}"),
                    "producer_output": "sources",
                    "subpath": "large",
                    "store_name": "large-package",
                    "nar_blake3": digest,
                })
            })
            .collect::<Vec<_>>()
            .into();
        let mut known_paths = DerivationRegistry::default();
        let (producer, outcome) = source_slice_plan_outcome(
            &bs,
            &serde_json::to_vec(&plan).unwrap(),
            source_root,
            "source-limit-producer",
            &mut known_paths,
        )
        .await;
        let mut worker = Worker::new(1);
        worker.want(&producer, &known_paths, true).unwrap();
        let registry_before = known_paths.len();
        let ready_before = worker.ready_goals.clone();
        let scan = worker
            .scan_native_dynamic_plans(
                &producer.to_absolute_path(),
                &outcome,
                &builder,
                known_paths.store_dir(),
                &BTreeSet::from(["plan".to_string()]),
            )
            .await
            .unwrap();
        let mut accepted = scan.accepted.into_iter().next().expect("bounded v2 plan");
        let rejected = worker
            .admit_v2_source_slices(&mut accepted, &outcome, &known_paths, &mut builder, &mut None)
            .await
            .unwrap()
            .expect("aggregate NAR byte limit must reject");
        assert_eq!(rejected.kind, NativeDynamicPlanRejectionKind::SliceLimit);
        let report = native_plan_report_from_rejection(&rejected);
        assert_eq!(report.source_slices.len(), crate::dynamic_plan::MAX_PLAN_SLICES as usize);
        assert!(report.source_slices.iter().all(|row| row.admitted_store_path.is_none()));
        assert_eq!(report.source_slices.last().unwrap().disposition, "slice-limit");
        let ca = CAHash::Nar(NixHash::Sha256(observation.nar_sha256()));
        let expected_path: StorePath<String> = nix_compat::store_path::build_ca_path_with_store_dir(
            "large-package",
            &ca,
            Vec::<&str>::new(),
            false,
            "/nix/store",
        )
        .unwrap();
        assert!(pis.get(*expected_path.digest()).await.unwrap().is_none());
        assert_eq!(known_paths.len(), registry_before);
        assert_eq!(worker.registry.len(), 1);
        assert_eq!(worker.ready_goals, ready_before);
        assert_eq!(worker.scheduling_epoch, 0);
        assert!(worker.priority_decisions.is_empty());
    }

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
            source_slices: Vec::new(),
            rejection_reason: None,
            scheduler_action: "registered-roots".to_string(),
        }
    }

    #[test]
    fn native_dynamic_plan_reports_are_deterministically_sorted() {
        let result = finish_worker_result(
            Vec::new(),
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

        let dep_id = UnitId::new("unit.dep").unwrap();
        let root_id = UnitId::new("unit.root").unwrap();
        let unused_id = UnitId::new("unit.unused").unwrap();
        assert!(registered.unit_drv_paths.contains_key(&dep_id));
        assert!(registered.unit_drv_paths.contains_key(&root_id));
        assert!(registered.unit_drv_paths.contains_key(&unused_id));
        assert_eq!(registered.root_drv_paths.len(), 1);
        assert!(result.failed.is_empty());
        assert_eq!(result.outcomes.len(), 1);
        assert!(dynamic_worker.registry().get(&registered.unit_drv_paths[&dep_id].to_absolute_path()).is_some());
        assert!(dynamic_worker.registry().get(&registered.unit_drv_paths[&root_id].to_absolute_path()).is_some());
        assert!(dynamic_worker.registry().get(&registered.unit_drv_paths[&unused_id].to_absolute_path()).is_none());
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

    #[tokio::test]
    // r[verify dynamic_derivation_admission.registry_boundary]
    async fn dynamic_missing_parent_leaves_registry_scheduler_and_reports_unchanged() {
        let bs = MemoryBlobService::default();
        let builder = make_test_builder(bs.clone());
        let mut scratch = DerivationRegistry::default();
        let (parent_path, _) = build_and_register("missing-parent", &[], &mut scratch);
        let (_, child) = build_and_register("dynamic-child", &[(parent_path, "out")], &mut scratch);
        let node = put_test_blob(&bs, &child.to_aterm_bytes()).await;

        let mut known_paths = DerivationRegistry::default();
        let (producer_path, _) = build_and_register("dynamic-producer", &[], &mut known_paths);
        let mut outputs = BTreeMap::new();
        outputs.insert("drv".to_string(), test_path_info("dynamic-child.drv", node));
        let outcome = BuildOutcome {
            drv_path: producer_path.clone(),
            outputs,
            substitutions: BTreeMap::new(),
            cached: false,
            log: None,
        };
        let mut worker = Worker::new(1);
        let before = (
            known_paths.len(),
            worker.registry.len(),
            worker.ready_goals.len(),
            worker.pressure_dirty_goals.len(),
            worker.priority_decisions.len(),
            worker.scheduling_epoch,
        );

        let error = worker
            .detect_dynamic_derivations(
                &producer_path.to_absolute_path(),
                &outcome,
                &builder,
                &mut known_paths,
                &BTreeSet::new(),
            )
            .await
            .unwrap_err();
        let after = (
            known_paths.len(),
            worker.registry.len(),
            worker.ready_goals.len(),
            worker.pressure_dirty_goals.len(),
            worker.priority_decisions.len(),
            worker.scheduling_epoch,
        );

        assert!(error.to_string().contains("dynamic-admission-missing-parent"));
        assert_eq!(after, before);
        assert_eq!(known_paths.len(), 1);
    }

    #[tokio::test]
    async fn dynamic_exact_duplicate_batch_inserts_one_registry_entry() {
        let bs = MemoryBlobService::default();
        let builder = make_test_builder(bs.clone());
        let mut scratch = DerivationRegistry::default();
        let (_, dynamic) = build_and_register("duplicate-dynamic", &[], &mut scratch);
        let node = put_test_blob(&bs, &dynamic.to_aterm_bytes()).await;

        let mut known_paths = DerivationRegistry::default();
        let (producer_path, _) = build_and_register("duplicate-producer", &[], &mut known_paths);
        let path_info = test_path_info("duplicate-dynamic.drv", node);
        let mut outputs = BTreeMap::new();
        outputs.insert("first".to_string(), path_info.clone());
        outputs.insert("second".to_string(), path_info);
        let outcome = BuildOutcome {
            drv_path: producer_path,
            outputs,
            substitutions: BTreeMap::new(),
            cached: false,
            log: None,
        };
        let worker = Worker::new(1);
        let before_entries = known_paths.len();
        let discovered = worker
            .scan_dynamic_derivations(&outcome, &builder, &mut known_paths, &BTreeSet::new())
            .await
            .unwrap();

        assert_eq!(discovered.len(), 1);
        assert_eq!(known_paths.len(), before_entries + 1);
        assert_eq!(discovered[0].output_name, "first");
    }

    #[tokio::test]
    async fn dynamic_registry_collision_changes_no_worker_or_registry_state() {
        let bs = MemoryBlobService::default();
        let builder = make_test_builder(bs.clone());
        let mut scratch = DerivationRegistry::default();
        let (_, dynamic) = build_and_register("colliding-dynamic", &[], &mut scratch);
        let bytes = dynamic.to_aterm_bytes();
        let node = put_test_blob(&bs, &bytes).await;
        let candidate = test_path_info("colliding-dynamic.drv", node);

        let parsed = parse_dynamic_candidate(
            &bytes,
            &candidate.store_path,
            nix_compat::store_path::STORE_DIR,
            DynamicAdmissionLimits::default(),
        )
        .unwrap()
        .unwrap();
        let validated =
            validate_dynamic_candidate(parsed, nix_compat::store_path::STORE_DIR, DynamicAdmissionLimits::default())
                .unwrap();
        let resolved = resolve_dynamic_identity(validated, &[]).unwrap();
        let ready = plan_dynamic_registration(resolved, None).unwrap();

        let mut known_paths = DerivationRegistry::default();
        known_paths.insert(
            ready.drv_path().clone(),
            ready.hash_derivation_modulo(),
            ready.derivation().clone(),
            ready.content_addressed(),
            None,
        );
        let (producer_path, _) = build_and_register("collision-producer", &[], &mut known_paths);
        let mut outputs = BTreeMap::new();
        outputs.insert("drv".to_string(), candidate);
        let outcome = BuildOutcome {
            drv_path: producer_path,
            outputs,
            substitutions: BTreeMap::new(),
            cached: false,
            log: None,
        };
        let worker = Worker::new(1);
        let before = (known_paths.len(), worker.registry.len(), worker.ready_goals.len());
        let error = worker
            .scan_dynamic_derivations(&outcome, &builder, &mut known_paths, &BTreeSet::new())
            .await
            .unwrap_err();
        let after = (known_paths.len(), worker.registry.len(), worker.ready_goals.len());

        assert!(error.to_string().contains("dynamic-admission-path-collision"));
        assert_eq!(after, before);
        assert_eq!(known_paths.len(), 2);
    }

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
