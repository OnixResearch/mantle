//! Deterministic unit, effect, and observation planning over admitted facts.

use alloc::collections::BTreeMap;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use serde::Deserialize;
use serde::Serialize;

use crate::digest::Blake3Digest;
use crate::digest::domain_digest;
use crate::model::MAX_ARGS_PER_EFFECT;
use crate::model::MAX_ENVIRONMENT_ENTRIES_PER_EFFECT;
use crate::model::PlanBlocker;
use crate::model::count_exceeds;

/// Accepted effect identity prefix for actual selected native units.
const EFFECT_ID_PREFIX: &str = "effect:";

const RESOLVED_EFFECT_DOMAIN: &[u8] = b"mantle-rust-resolved-unit-effect-v1\0";
const RESOLVED_OBSERVATION_DOMAIN: &[u8] = b"mantle-rust-resolved-unit-observation-v1\0";
const RESTORED_COMPILER_DOMAIN: &[u8] = b"mantle-rust-restored-compiler-artifact-v1\0";

/// Maximum total declared process-word and identity bytes for one attempt.
pub const MAX_RESOLVED_EFFECT_BYTES: usize = 8_388_608;
/// Maximum bytes in one argument, executable, environment value, or identity.
pub const MAX_PROCESS_WORD_BYTES: usize = 1024 * 1024;

/// Maximum admitted units in one plan.
pub const MAX_UNITS: u32 = 16_384;

/// Accepted cargo-free readiness blockers, in the legacy sorted class order.
pub fn classify_cargo_free_planning_blockers(readiness: [bool; 4]) -> Vec<String> {
    const CLASSES: [&str; 4] = [
        "native-package-target-planning-blocked",
        "native-unit-graph-planning-blocked",
        "native-host-unit-graph-planning-blocked",
        "unit-derivation-graph-blocked",
    ];
    let mut blockers: Vec<String> = readiness
        .into_iter()
        .zip(CLASSES)
        .filter(|(ready, _)| !*ready)
        .map(|(_, class)| String::from(class))
        .collect();
    blockers.sort();
    blockers
}

/// Preserve the selected rustc linker mode unless an explicit split or
/// joined `-C link-self-contained` option exists.
pub fn plan_native_runtime_arguments(reviewable_args: &[String]) -> Vec<String> {
    let mut is_explicit = false;
    let mut is_previous_codegen = false;
    for arg in reviewable_args {
        let is_link_option = |option: &str| {
            option == "link-self-contained"
                || option.strip_prefix("link-self-contained").is_some_and(|suffix| suffix.starts_with('='))
        };
        if is_previous_codegen && is_link_option(arg) {
            is_explicit = true;
        }
        if arg.strip_prefix("-C").is_some_and(is_link_option) {
            is_explicit = true;
        }
        is_previous_codegen = arg == "-C";
    }
    let mut arguments = reviewable_args.to_vec();
    if !is_explicit {
        arguments.push(String::from("-C"));
        arguments.push(String::from("link-self-contained=no"));
    }
    debug_assert!(arguments.len() >= reviewable_args.len());
    debug_assert!(is_explicit || arguments.len() == reviewable_args.len().saturating_add(2));
    arguments
}

/// Build profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BuildProfile {
    Dev,
    Release,
}

/// Opaque unit identity supplied by the selected native derivation graph.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UnitId(pub String);

/// Stable effect identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct EffectId(pub String);

/// A unit already selected by the native planner and normalized by the shell.
/// External source/toolchain inputs are identities, not dependency unit IDs.
/// `execution_order` is the bounded native topology order, not collection order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExistingUnitFacts {
    pub unit_id: String,
    pub package_id: String,
    pub target_name: String,
    pub target_kind: String,
    pub execution_kind: String,
    pub dependency_unit_ids: Vec<String>,
    pub arguments: Vec<String>,
    pub environment: Vec<(String, String)>,
    pub input_identities: Vec<String>,
    pub expected_outputs: Vec<String>,
    pub execution_order: u32,
}

/// An effect for an actual selected derivation. This is not a rustc execution receipt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExistingUnitEffect {
    pub effect_id: EffectId,
    pub unit_id: UnitId,
    pub package_id: String,
    pub target_name: String,
    pub target_kind: String,
    pub execution_kind: String,
    pub dependency_unit_ids: Vec<UnitId>,
    pub arguments: Vec<String>,
    pub environment: Vec<(String, String)>,
    pub input_identities: Vec<String>,
    pub expected_outputs: Vec<String>,
    pub execution_order: u32,
}
/// Resolved producer edges for a selected native topology participant.
/// Roots and dependencies retain the adapter's accepted graph-index order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExistingTopologyUnit {
    pub unit_id: String,
    pub dependencies: Vec<String>,
    pub execution_rank: u32,
}
/// Lossless adapter-encoded process word. Only the shell interprets `encoding`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "representation", content = "value", rename_all = "kebab-case")]
pub enum ProcessWord {
    Utf8(String),
    Opaque { encoding: String, bytes: Vec<u8> },
}

impl ProcessWord {
    fn byte_len(&self) -> usize {
        match self {
            Self::Utf8(value) => value.len(),
            Self::Opaque { encoding, bytes } => encoding.len().saturating_add(bytes.len()),
        }
    }

    fn is_valid(&self, nonempty: bool) -> bool {
        match self {
            Self::Utf8(value) => (!nonempty || !value.is_empty()) && !value.contains('\0'),
            Self::Opaque { encoding, bytes } => {
                if nonempty && bytes.is_empty() {
                    return false;
                }
                match encoding.as_str() {
                    "unix-os-bytes" => !bytes.contains(&0),
                    "windows-wtf16le" => bytes.len() % 2 == 0 && !bytes.as_chunks::<2>().0.contains(&[0, 0]),
                    _ => false,
                }
            }
        }
    }
}

/// Actual process role. A build-script run follows an observed compiler unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProcessEffectRole {
    RustCompiler,
    BuildScriptRun,
}

/// Observed artifact or metadata from a declared earlier producer unit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProducerArtifactObservation {
    pub producer_unit_id: UnitId,
    pub artifact_identity: String,
}

/// Host-resolved process facts after binding and compiler-policy selection,
/// before execution. Attempt one is the bounded audit fallback.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedUnitFacts {
    pub effect_id: EffectId,
    pub unit_id: UnitId,
    pub role: ProcessEffectRole,
    pub attempt: u32,
    pub prior_attempt_identity: Option<Blake3Digest>,
    pub executable: ProcessWord,
    pub toolchain_identity: String,
    pub arguments: Vec<ProcessWord>,
    pub environment: Vec<(String, ProcessWord)>,
    pub working_directory: Option<ProcessWord>,
    pub input_identities: Vec<String>,
    pub expected_outputs: Vec<String>,
    pub dependency_observations: Vec<ProducerArtifactObservation>,
}

/// The exact process material admitted for execution by a shell adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedUnitEffect {
    pub facts: ResolvedUnitFacts,
    pub resolved_blake3: Blake3Digest,
}

/// Typed observation of exactly one resolved process attempt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedProcessObservation {
    pub effect_id: EffectId,
    pub unit_id: UnitId,
    pub resolved_blake3: Blake3Digest,
    pub role: ProcessEffectRole,
    pub attempt: u32,
    pub status: UnitObservationStatus,
    pub exit_code: Option<i32>,
    /// Logical expected output name and adapter-observed output identity.
    pub output_identities: Vec<(String, String)>,
}

/// A real, successful compiler cache restoration, not an executed process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CompilerRestoreKind {
    ReusedOutput,
    RestoredLocal,
    RestoredShared,
}

/// Cache receipt and re-digested output evidence after a compiler cache hit.
/// The artifact set digest is over the receipt's actual sorted output digests,
/// keeping this predecessor bounded even when the compiler emitted many files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestoredCompilerArtifactObservation {
    pub effect_id: EffectId,
    pub unit_id: UnitId,
    pub compiler_resolved_blake3: Blake3Digest,
    pub cache_receipt_identity: Blake3Digest,
    pub declared_outputs: Vec<String>,
    pub output_artifact_set_identity: Blake3Digest,
    pub output_artifact_count: u32,
    pub cache_kind: CompilerRestoreKind,
}

/// Admit an actual restored compiler output as causal evidence for running
/// its build script, without claiming a compiler process was executed.
pub fn classify_restored_compiler_artifact(
    planned: &ExistingUnitEffect,
    primary: &ResolvedUnitEffect,
    restored: &RestoredCompilerArtifactObservation,
) -> Result<Blake3Digest, PlanBlocker> {
    let subject = planned.unit_id.0.as_str();
    let is_primary_compiler = planned.target_kind == "custom-build"
        && primary.facts.role == ProcessEffectRole::RustCompiler
        && primary.facts.attempt == 0;
    let is_selected_primary = primary.facts.unit_id == planned.unit_id && primary.facts.effect_id == planned.effect_id;
    let is_matching_restoration = restored.effect_id == planned.effect_id
        && restored.unit_id == planned.unit_id
        && restored.compiler_resolved_blake3 == primary.resolved_blake3;
    if !(is_primary_compiler && is_selected_primary && is_matching_restoration) {
        return Err(PlanBlocker::new(
            "restored-compiler-predecessor",
            subject,
            "restored artifact must belong to the admitted primary compiler of this build script",
        ));
    }
    if restored.declared_outputs != planned.expected_outputs
        || primary.facts.expected_outputs != planned.expected_outputs
        || restored.output_artifact_count == 0
    {
        return Err(PlanBlocker::new(
            "restored-compiler-outputs",
            subject,
            "restored compiler receipt must cover the selected outputs and observed artifact set",
        ));
    }
    debug_assert_eq!(primary.facts.unit_id, planned.unit_id);
    debug_assert_eq!(restored.declared_outputs, planned.expected_outputs);
    domain_digest(RESTORED_COMPILER_DOMAIN, restored).map_err(|_| {
        PlanBlocker::new(
            "restored-compiler-identity",
            subject,
            "restored compiler artifact could not be canonically identified",
        )
    })
}

/// A build-script process after cache restoration has its own real process
/// observation, but its compiler predecessor is restored artifact evidence.
pub fn admit_resolved_build_script_after_cache(
    planned: &ExistingUnitEffect,
    facts: ResolvedUnitFacts,
    primary: &ResolvedUnitEffect,
    restored: &RestoredCompilerArtifactObservation,
) -> Result<ResolvedUnitEffect, Vec<PlanBlocker>> {
    let identity = classify_restored_compiler_artifact(planned, primary, restored).map_err(|blocker| vec![blocker])?;
    admit_resolved_unit_effect_with_restored(planned, facts, None, Some((restored, &identity)))
}

/// Observation status reported by an adapter after executing one effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UnitObservationStatus {
    Succeeded,
    Failed,
    Skipped,
}

/// One typed observation for a planned effect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitObservation {
    pub effect_id: EffectId,
    pub unit_id: UnitId,
    pub status: UnitObservationStatus,
    pub exit_code: Option<i32>,
    pub diagnostics_code: Option<String>,
}

/// Result of classifying observations against one plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanExecutionOutcome {
    /// Every planned effect observed `Succeeded`.
    Completed,
    /// Some effects failed; the count is exact and bounded.
    Failed { failed_effect_count: u32 },
    /// Observed effects do not match the plan: unknown or missing identities.
    Rejected {
        unknown_effect_count: u32,
        missing_effect_count: u32,
    },
    /// Supplied effects or observations exceed the admitted topology bound.
    InputLimitExceeded,
}

/// Admit already selected native derivations without rewriting their Cargo/rustc
/// semantics. The shell supplies a stable tie-break rank, and this core orders
/// internal producer edges. Paths and external artifacts remain declared inputs.
pub fn plan_existing_unit_effects(units: Vec<ExistingUnitFacts>) -> Result<Vec<ExistingUnitEffect>, Vec<PlanBlocker>> {
    let order = ordered_existing_unit_indices(&units)?;
    let Some(indices) = order else {
        return Ok(units.into_iter().map(unit_into_effect).collect());
    };
    let mut slots = units.into_iter().map(Some).collect::<Vec<_>>();
    let mut effects = Vec::with_capacity(indices.len());
    for index in indices {
        let Some(unit) = slots.get_mut(index).and_then(Option::take) else {
            return Err(vec![PlanBlocker::new(
                "unit-identity",
                "units",
                "ordered unit index is absent or repeated",
            )]);
        };
        effects.push(unit_into_effect(unit));
    }
    Ok(effects)
}

fn ordered_existing_unit_indices(units: &[ExistingUnitFacts]) -> Result<Option<Vec<usize>>, Vec<PlanBlocker>> {
    if units.is_empty() {
        return Err(vec![PlanBlocker::new(
            "missing-units",
            "units",
            "the selected unit topology is empty",
        )]);
    }
    if count_exceeds(units.len(), MAX_UNITS) {
        return Err(vec![PlanBlocker::new(
            "unit-limit",
            "units",
            "selected units exceed the admitted bound",
        )]);
    }
    let mut blockers = Vec::with_capacity(units.len());
    let (by_id, mut is_already_ordered) = validate_existing_units(units, &mut blockers);
    validate_existing_dependencies(units, &by_id, &mut is_already_ordered, &mut blockers);
    if !blockers.is_empty() {
        blockers.sort();
        blockers.dedup();
        return Err(blockers);
    }
    debug_assert_eq!(by_id.len(), units.len());
    debug_assert!(!count_exceeds(units.len(), MAX_UNITS));
    if is_already_ordered {
        return Ok(None);
    }
    order_selected_units(units, &by_id).map(Some)
}

fn validate_existing_units<'a>(
    units: &'a [ExistingUnitFacts],
    blockers: &mut Vec<PlanBlocker>,
) -> (BTreeMap<&'a str, usize>, bool) {
    let mut by_id = BTreeMap::new();
    let mut by_order = BTreeSet::new();
    let mut is_already_ordered = true;
    let mut previous_order = None;
    for (index, unit) in units.iter().enumerate() {
        if let Some(previous) = previous_order {
            is_already_ordered &= previous < unit.execution_order;
        }
        previous_order = Some(unit.execution_order);
        if [
            &unit.unit_id,
            &unit.package_id,
            &unit.target_name,
            &unit.target_kind,
            &unit.execution_kind,
        ]
        .iter()
        .any(|identity| identity.is_empty())
        {
            blockers.push(PlanBlocker::new(
                "unit-identity",
                "units",
                "unit, package, target and execution identities must be non-empty",
            ));
            continue;
        }
        if by_id.insert(unit.unit_id.as_str(), index).is_some() {
            blockers.push(PlanBlocker::new("duplicate-unit", &unit.unit_id, "selected unit identity is duplicated"));
        }
        if !by_order.insert(unit.execution_order) {
            blockers.push(PlanBlocker::new(
                "duplicate-execution-order",
                &unit.unit_id,
                "selected topology ranks must be unique",
            ));
        }
        debug_assert_eq!(by_id.get(unit.unit_id.as_str()), Some(&index));
        debug_assert!(by_order.contains(&unit.execution_order));
        validate_existing_unit_effect(unit, blockers);
    }
    (by_id, is_already_ordered)
}

fn validate_existing_unit_effect(unit: &ExistingUnitFacts, blockers: &mut Vec<PlanBlocker>) {
    let is_build_script_run = unit.execution_kind == "build-script-run" && unit.target_kind == "custom-build";
    if !matches!(unit.execution_kind.as_str(), "target" | "host" | "host-dependency") && !is_build_script_run {
        blockers.push(PlanBlocker::new(
            "unsupported-execution-kind",
            &unit.unit_id,
            "selected unit execution kind is outside the bounded native topology",
        ));
    }
    if count_exceeds(unit.arguments.len(), MAX_ARGS_PER_EFFECT) {
        blockers.push(PlanBlocker::new(
            "effect-argument-limit",
            &unit.unit_id,
            "effect arguments exceed the declared bound",
        ));
    }
    if count_exceeds(unit.environment.len(), MAX_ENVIRONMENT_ENTRIES_PER_EFFECT) {
        blockers.push(PlanBlocker::new(
            "effect-environment-limit",
            &unit.unit_id,
            "effect environment exceeds the declared bound",
        ));
    }
    if count_exceeds(unit.dependency_unit_ids.len(), MAX_UNITS)
        || count_exceeds(unit.input_identities.len(), MAX_ARGS_PER_EFFECT)
        || count_exceeds(unit.expected_outputs.len(), MAX_ARGS_PER_EFFECT)
    {
        blockers.push(PlanBlocker::new(
            "effect-input-output-limit",
            &unit.unit_id,
            "declared dependencies, input identities or outputs exceed the admitted bound",
        ));
    }
    let is_missing_values =
        unit.arguments.is_empty() || unit.expected_outputs.is_empty() || unit.arguments.iter().any(String::is_empty);
    let is_missing_identity =
        unit.input_identities.iter().any(String::is_empty) || unit.expected_outputs.iter().any(String::is_empty);
    if is_missing_values || is_missing_identity {
        blockers.push(PlanBlocker::new(
            "incomplete-unit-effect",
            &unit.unit_id,
            "effect arguments, input identities and expected outputs must be declared values",
        ));
    }
    let mut environment_names = BTreeSet::new();
    for (name, value) in &unit.environment {
        if name.is_empty() || value.contains('\0') || !environment_names.insert(name.as_str()) {
            blockers.push(PlanBlocker::new(
                "invalid-effect-environment",
                &unit.unit_id,
                "environment names must be non-empty and unique, values cannot contain NUL",
            ));
        }
    }
    debug_assert!(environment_names.iter().all(|name| !name.is_empty()));
    debug_assert!(environment_names.len() <= unit.environment.len());
}

fn validate_existing_dependencies(
    units: &[ExistingUnitFacts],
    by_id: &BTreeMap<&str, usize>,
    is_already_ordered: &mut bool,
    blockers: &mut Vec<PlanBlocker>,
) {
    for unit in units {
        let mut seen_dependencies = BTreeSet::new();
        for dependency in &unit.dependency_unit_ids {
            if !seen_dependencies.insert(dependency.as_str()) {
                blockers.push(PlanBlocker::new(
                    "duplicate-unit-dependency",
                    &unit.unit_id,
                    "a unit producer edge is declared more than once",
                ));
            }
            if let Some(&producer_index) = by_id.get(dependency.as_str()) {
                *is_already_ordered &= units[producer_index].execution_order < unit.execution_order;
            } else {
                blockers.push(PlanBlocker::new(
                    "missing-unit-producer",
                    &unit.unit_id,
                    "declared unit producer is absent from the selected topology",
                ));
            }
        }
    }
}

struct SelectedUnitEdges<'a> {
    indegrees: BTreeMap<&'a str, u32>,
    consumers: BTreeMap<&'a str, Vec<usize>>,
}

fn selected_unit_edges(units: &[ExistingUnitFacts]) -> Result<SelectedUnitEdges<'_>, Vec<PlanBlocker>> {
    let mut indegrees = BTreeMap::<&str, u32>::new();
    let mut consumers = BTreeMap::<&str, Vec<usize>>::new();
    for (index, unit) in units.iter().enumerate() {
        let Ok(degree) = u32::try_from(unit.dependency_unit_ids.len()) else {
            return Err(vec![PlanBlocker::new(
                "effect-input-output-limit",
                &unit.unit_id,
                "declared dependencies, input identities or outputs exceed the admitted bound",
            )]);
        };
        if indegrees.len() >= units.len() {
            return Err(vec![PlanBlocker::new(
                "unit-limit",
                "units",
                "selected units exceed the admitted bound",
            )]);
        }
        indegrees.insert(unit.unit_id.as_str(), degree);
        for producer in &unit.dependency_unit_ids {
            if !consumers.contains_key(producer.as_str()) && consumers.len() >= units.len() {
                return Err(vec![PlanBlocker::new(
                    "unit-limit",
                    "units",
                    "selected units exceed the admitted bound",
                )]);
            }
            consumers.entry(producer.as_str()).or_default().push(index);
        }
    }
    debug_assert_eq!(indegrees.len(), units.len());
    debug_assert!(consumers.keys().all(|id| indegrees.contains_key(id)));
    Ok(SelectedUnitEdges { indegrees, consumers })
}

// The native derivation order is only a tie-break; producer edges determine execution.
fn order_selected_units(
    units: &[ExistingUnitFacts],
    by_id: &BTreeMap<&str, usize>,
) -> Result<Vec<usize>, Vec<PlanBlocker>> {
    let SelectedUnitEdges {
        mut indegrees,
        consumers,
    } = selected_unit_edges(units)?;
    let mut ready = BTreeSet::new();
    for unit in units {
        if indegrees.get(unit.unit_id.as_str()) == Some(&0) {
            ready.insert((unit.execution_order, unit.unit_id.as_str()));
        }
    }
    debug_assert!(ready.len() <= units.len());
    let mut ordered = Vec::with_capacity(units.len());
    while let Some((_, id)) = ready.pop_first() {
        let Some(&index) = by_id.get(id) else {
            return Err(vec![PlanBlocker::new(
                "missing-unit-producer",
                id,
                "declared unit producer is absent from the selected topology",
            )]);
        };
        ordered.push(index);
        for &dependent_index in consumers.get(id).into_iter().flatten() {
            let dependent = &units[dependent_index];
            let Some(degree) = indegrees.get_mut(dependent.unit_id.as_str()) else {
                return Err(vec![PlanBlocker::new(
                    "missing-unit-producer",
                    &dependent.unit_id,
                    "declared unit producer is absent from the selected topology",
                )]);
            };
            let Some(next) = degree.checked_sub(1) else {
                return Err(vec![PlanBlocker::new(
                    "dependency-cycle",
                    &dependent.unit_id,
                    "selected unit topology contains a dependency cycle",
                )]);
            };
            *degree = next;
            if next == 0 {
                ready.insert((dependent.execution_order, dependent.unit_id.as_str()));
            }
        }
    }
    if ordered.len() != units.len() {
        let mut cycle_blockers = units
            .iter()
            .filter(|unit| indegrees.get(unit.unit_id.as_str()).is_some_and(|degree| *degree != 0))
            .map(|unit| {
                PlanBlocker::new(
                    "dependency-cycle",
                    &unit.unit_id,
                    "selected unit topology contains a dependency cycle",
                )
            })
            .collect::<Vec<_>>();
        cycle_blockers.sort();
        return Err(cycle_blockers);
    }
    debug_assert!(ordered.iter().all(|index| *index < units.len()));
    Ok(ordered)
}

fn unit_into_effect(unit: ExistingUnitFacts) -> ExistingUnitEffect {
    let mut effect_id = String::with_capacity(EFFECT_ID_PREFIX.len().saturating_add(unit.unit_id.len()));
    effect_id.push_str(EFFECT_ID_PREFIX);
    effect_id.push_str(&unit.unit_id);
    ExistingUnitEffect {
        effect_id: EffectId(effect_id),
        unit_id: UnitId(unit.unit_id),
        package_id: unit.package_id,
        target_name: unit.target_name,
        target_kind: unit.target_kind,
        execution_kind: unit.execution_kind,
        dependency_unit_ids: unit.dependency_unit_ids.into_iter().map(UnitId).collect(),
        arguments: unit.arguments,
        environment: unit.environment,
        input_identities: unit.input_identities,
        expected_outputs: unit.expected_outputs,
        execution_order: unit.execution_order,
    }
}

/// Admit the final invocation after all declared producer, toolchain and
/// policy observations are available. The shell must execute the returned
/// executable, arguments, environment and cwd without reconstructing them.
pub fn admit_resolved_unit_effect(
    planned: &ExistingUnitEffect,
    facts: ResolvedUnitFacts,
    prior: Option<&ResolvedProcessObservation>,
) -> Result<ResolvedUnitEffect, Vec<PlanBlocker>> {
    admit_resolved_unit_effect_with_restored(planned, facts, prior, None)
}

fn admit_resolved_unit_effect_with_restored(
    planned: &ExistingUnitEffect,
    facts: ResolvedUnitFacts,
    prior: Option<&ResolvedProcessObservation>,
    restored: Option<(&RestoredCompilerArtifactObservation, &Blake3Digest)>,
) -> Result<ResolvedUnitEffect, Vec<PlanBlocker>> {
    let mut blockers = Vec::with_capacity(16);
    let subject = planned.unit_id.0.as_str();
    if facts.unit_id != planned.unit_id || facts.effect_id != planned.effect_id {
        blockers.push(PlanBlocker::new(
            "resolved-effect-identity",
            subject,
            "resolved effect must retain the selected unit and effect identities",
        ));
    }
    validate_resolved_predecessor(planned, &facts, prior, restored, &mut blockers);
    validate_resolved_fields(planned, &facts, &mut blockers);
    let total_bytes = validate_resolved_words(&facts, subject, &mut blockers);
    validate_resolved_dependencies(planned, &facts, subject, total_bytes, &mut blockers);
    if !blockers.is_empty() {
        blockers.sort();
        blockers.dedup();
        return Err(blockers);
    }
    debug_assert_eq!(facts.unit_id, planned.unit_id);
    debug_assert_eq!(facts.effect_id, planned.effect_id);
    let resolved_blake3 = domain_digest(RESOLVED_EFFECT_DOMAIN, &facts).map_err(|_| {
        vec![PlanBlocker::new(
            "resolved-effect-identity-failed",
            subject,
            "resolved process facts could not be canonically identified",
        )]
    })?;
    Ok(ResolvedUnitEffect { facts, resolved_blake3 })
}

fn validate_resolved_predecessor(
    planned: &ExistingUnitEffect,
    facts: &ResolvedUnitFacts,
    prior: Option<&ResolvedProcessObservation>,
    restored: Option<(&RestoredCompilerArtifactObservation, &Blake3Digest)>,
    blockers: &mut Vec<PlanBlocker>,
) {
    let subject = planned.unit_id.0.as_str();
    match (facts.role, facts.attempt) {
        (ProcessEffectRole::RustCompiler, 0)
            if prior.is_none() && restored.is_none() && facts.prior_attempt_identity.is_none() => {}
        (ProcessEffectRole::RustCompiler, 1) | (ProcessEffectRole::BuildScriptRun, 0) => {
            let expected_status = if facts.role == ProcessEffectRole::RustCompiler {
                UnitObservationStatus::Failed
            } else {
                UnitObservationStatus::Succeeded
            };
            let is_correct_role =
                facts.role != ProcessEffectRole::BuildScriptRun || planned.target_kind == "custom-build";
            let is_valid_prior = prior
                .filter(|observation| {
                    observation.effect_id == planned.effect_id
                        && observation.unit_id == planned.unit_id
                        && observation.role == ProcessEffectRole::RustCompiler
                        && observation.status == expected_status
                        && match observation.status {
                            UnitObservationStatus::Succeeded => {
                                observation.exit_code == Some(0) && !observation.output_identities.is_empty()
                            }
                            UnitObservationStatus::Failed => observation.exit_code.is_some_and(|code| code != 0),
                            UnitObservationStatus::Skipped => false,
                        }
                        && observation.attempt <= 1
                        && (facts.role != ProcessEffectRole::RustCompiler || observation.attempt == 0)
                })
                .and_then(|observation| domain_digest(RESOLVED_OBSERVATION_DOMAIN, observation).ok())
                .is_some_and(|identity| facts.prior_attempt_identity.as_ref() == Some(&identity));
            let is_valid_restored = facts.role == ProcessEffectRole::BuildScriptRun
                && prior.is_none()
                && restored.is_some_and(|(observation, identity)| {
                    facts.prior_attempt_identity.as_ref() == Some(identity)
                        && facts
                            .input_identities
                            .iter()
                            .any(|input| input.as_str() == observation.cache_receipt_identity.as_str())
                        && facts
                            .input_identities
                            .iter()
                            .any(|input| input.as_str() == observation.output_artifact_set_identity.as_str())
                });
            if !is_correct_role || !(is_valid_prior || is_valid_restored) {
                blockers.push(PlanBlocker::new(
                    "resolved-effect-predecessor",
                    subject,
                    "fallback and build-script processes require matching observed compiler or restored output evidence",
                ));
            }
            if is_correct_role && (is_valid_prior || is_valid_restored) {
                debug_assert!(facts.prior_attempt_identity.is_some());
                debug_assert!(facts.role != ProcessEffectRole::BuildScriptRun || planned.target_kind == "custom-build");
            }
        }
        _ => blockers.push(PlanBlocker::new(
            "unsupported-process-attempt",
            subject,
            "resolved process role or attempt is outside the bounded unit topology",
        )),
    }
}

fn validate_resolved_fields(planned: &ExistingUnitEffect, facts: &ResolvedUnitFacts, blockers: &mut Vec<PlanBlocker>) {
    let subject = planned.unit_id.0.as_str();
    match facts.role {
        ProcessEffectRole::RustCompiler if facts.expected_outputs != planned.expected_outputs => {
            blockers.push(PlanBlocker::new(
                "resolved-effect-outputs",
                subject,
                "compiler outputs must match the selected derivation",
            ));
        }
        ProcessEffectRole::BuildScriptRun
            if facts.expected_outputs.len() != 2
                || facts.expected_outputs[0] != "OUT_DIR"
                || facts.expected_outputs[1] != "metadata-stdout" =>
        {
            blockers.push(PlanBlocker::new(
                "resolved-effect-outputs",
                subject,
                "build-script run must declare OUT_DIR and metadata stdout observations",
            ));
        }
        ProcessEffectRole::RustCompiler | ProcessEffectRole::BuildScriptRun => {}
    }
    let is_invalid_command = !facts.executable.is_valid(true)
        || facts.toolchain_identity.is_empty()
        || facts.working_directory.as_ref().is_some_and(|cwd| !cwd.is_valid(true));
    let is_invalid_words = facts.arguments.iter().any(|arg| !arg.is_valid(false))
        || facts.environment.iter().any(|(_, value)| !value.is_valid(false));
    let is_invalid_identities =
        facts.input_identities.iter().any(String::is_empty) || facts.expected_outputs.iter().any(String::is_empty);
    if is_invalid_command || is_invalid_words || is_invalid_identities {
        blockers.push(PlanBlocker::new(
            "incomplete-resolved-effect",
            subject,
            "resolved executable, toolchain, arguments, environment, inputs and outputs must be declared values",
        ));
    }
    let is_large_words = count_exceeds(facts.arguments.len(), MAX_ARGS_PER_EFFECT)
        || count_exceeds(facts.environment.len(), MAX_ENVIRONMENT_ENTRIES_PER_EFFECT);
    let is_large_identities = count_exceeds(facts.input_identities.len(), MAX_ARGS_PER_EFFECT)
        || count_exceeds(facts.expected_outputs.len(), MAX_ARGS_PER_EFFECT)
        || count_exceeds(facts.dependency_observations.len(), MAX_UNITS);
    if is_large_words || is_large_identities {
        blockers.push(PlanBlocker::new(
            "resolved-effect-limit",
            subject,
            "resolved effect collections exceed the admitted bounds",
        ));
    }
    if !is_invalid_command {
        debug_assert!(facts.executable.is_valid(true));
        debug_assert!(!facts.toolchain_identity.is_empty());
    }
}

fn validate_resolved_words(facts: &ResolvedUnitFacts, subject: &str, blockers: &mut Vec<PlanBlocker>) -> usize {
    let mut total_bytes = facts
        .executable
        .byte_len()
        .saturating_add(facts.toolchain_identity.len())
        .saturating_add(facts.effect_id.0.len())
        .saturating_add(facts.unit_id.0.len())
        .saturating_add(facts.prior_attempt_identity.as_ref().map_or(0, |identity| identity.as_str().len()));
    let mut is_word_over_limit_bytes = false;
    for word in facts.arguments.iter().chain(facts.environment.iter().map(|(_, value)| value)) {
        total_bytes = total_bytes.saturating_add(word.byte_len());
        is_word_over_limit_bytes |= word.byte_len() > MAX_PROCESS_WORD_BYTES;
        debug_assert!(total_bytes >= word.byte_len());
    }
    if is_word_over_limit_bytes {
        blockers.push(PlanBlocker::new(
            "resolved-process-word-limit",
            subject,
            "a resolved process word exceeds the admitted byte bound",
        ));
    }
    if let Some(cwd) = &facts.working_directory {
        total_bytes = total_bytes.saturating_add(cwd.byte_len());
    }
    for (name, _) in &facts.environment {
        total_bytes = total_bytes.saturating_add(name.len());
    }
    for identity in facts.input_identities.iter().chain(&facts.expected_outputs) {
        total_bytes = total_bytes.saturating_add(identity.len());
        debug_assert!(total_bytes >= identity.len());
        if identity.len() > MAX_PROCESS_WORD_BYTES {
            blockers.push(PlanBlocker::new(
                "resolved-identity-limit",
                subject,
                "a resolved effect identity exceeds the admitted byte bound",
            ));
        }
    }
    let mut names = BTreeSet::new();
    for (name, _) in &facts.environment {
        if name.is_empty() || name.contains('\0') || !names.insert(name.as_str()) {
            blockers.push(PlanBlocker::new(
                "invalid-resolved-environment",
                subject,
                "resolved child environment keys must be unique and non-empty",
            ));
        }
    }
    total_bytes
}

fn validate_resolved_dependencies(
    planned: &ExistingUnitEffect,
    facts: &ResolvedUnitFacts,
    subject: &str,
    mut total_bytes: usize,
    blockers: &mut Vec<PlanBlocker>,
) {
    for identity in &planned.input_identities {
        if !facts.input_identities.contains(identity) {
            blockers.push(PlanBlocker::new(
                "missing-resolved-input",
                subject,
                "a selected derivation input was omitted from the final process",
            ));
        }
    }
    let base_bytes = total_bytes;
    let declared: BTreeSet<&str> = planned.dependency_unit_ids.iter().map(|id| id.0.as_str()).collect();
    let mut observed = BTreeSet::new();
    for producer in &facts.dependency_observations {
        total_bytes = total_bytes
            .saturating_add(producer.producer_unit_id.0.len())
            .saturating_add(producer.artifact_identity.len());
        let is_invalid_producer = !declared.contains(producer.producer_unit_id.0.as_str())
            || producer.artifact_identity.is_empty()
            || producer.artifact_identity.len() > MAX_PROCESS_WORD_BYTES;
        let is_invalid_input = !facts.input_identities.contains(&producer.artifact_identity);
        if is_invalid_producer || is_invalid_input || !observed.insert(producer.producer_unit_id.0.as_str()) {
            blockers.push(PlanBlocker::new(
                "invalid-producer-observation",
                subject,
                "resolved producer must be declared once with a matching artifact input identity",
            ));
        }
    }
    debug_assert!(total_bytes >= base_bytes);
    debug_assert!(observed.iter().all(|id| declared.contains(id)));
    let is_large_process = total_bytes > MAX_RESOLVED_EFFECT_BYTES
        || facts.executable.byte_len() > MAX_PROCESS_WORD_BYTES
        || facts.working_directory.as_ref().is_some_and(|cwd| cwd.byte_len() > MAX_PROCESS_WORD_BYTES);
    if is_large_process {
        blockers.push(PlanBlocker::new(
            "resolved-effect-byte-limit",
            subject,
            "resolved process facts exceed the admitted byte bound",
        ));
    }
    if observed.len() != declared.len() {
        blockers.push(PlanBlocker::new(
            "missing-producer-observation",
            subject,
            "every selected dependency producer must have an observed artifact identity",
        ));
    }
}

/// Bind one observed result to its exact admitted process attempt and outputs.
pub fn classify_resolved_process_observation(
    effect: &ResolvedUnitEffect,
    observation: &ResolvedProcessObservation,
) -> Result<Blake3Digest, PlanBlocker> {
    let facts = &effect.facts;
    let is_wrong_effect = observation.effect_id != facts.effect_id
        || observation.unit_id != facts.unit_id
        || observation.resolved_blake3 != effect.resolved_blake3;
    let is_wrong_attempt = observation.role != facts.role || observation.attempt != facts.attempt;
    if is_wrong_effect || is_wrong_attempt {
        return Err(PlanBlocker::new(
            "wrong-resolved-observation",
            &facts.unit_id.0,
            "process observation does not name the admitted role, attempt and resolved identity",
        ));
    }
    let consistency = UnitObservation {
        effect_id: observation.effect_id.clone(),
        unit_id: observation.unit_id.clone(),
        status: observation.status,
        exit_code: observation.exit_code,
        diagnostics_code: None,
    };
    if !observation_exit_is_consistent(&consistency) || observation.status == UnitObservationStatus::Skipped {
        return Err(PlanBlocker::new(
            "invalid-resolved-observation",
            &facts.unit_id.0,
            "process status and exit code contradict the admitted attempt",
        ));
    }
    let mut names = BTreeSet::new();
    for (name, identity) in &observation.output_identities {
        let is_invalid_identity = name.is_empty() || identity.is_empty() || !facts.expected_outputs.contains(name);
        if is_invalid_identity || !names.insert(name.as_str()) {
            return Err(PlanBlocker::new(
                "invalid-observed-output",
                &facts.unit_id.0,
                "observed output identity is empty, duplicated or undeclared",
            ));
        }
    }
    if observation.status == UnitObservationStatus::Succeeded && names.len() != facts.expected_outputs.len() {
        return Err(PlanBlocker::new(
            "missing-observed-output",
            &facts.unit_id.0,
            "every successful declared process output requires an observed identity",
        ));
    }
    debug_assert_eq!(observation.output_identities.len(), names.len());
    debug_assert!(names.iter().all(|name| facts.expected_outputs.iter().any(|output| output.as_str() == *name)));
    domain_digest(RESOLVED_OBSERVATION_DOMAIN, observation).map_err(|_| {
        PlanBlocker::new(
            "resolved-observation-identity-failed",
            &facts.unit_id.0,
            "observed process facts could not be canonically identified",
        )
    })
}

/// The accepted iterative native depth-first topology order, over resolved
/// producer IDs rather than raw Cargo JSON or filesystem paths.
pub fn order_existing_unit_topology(
    roots: &[String],
    units: &[ExistingTopologyUnit],
) -> Result<Vec<String>, PlanBlocker> {
    let by_id = validate_existing_topology(roots, units)?;
    let edge_count = units.iter().fold(0usize, |count, unit| count.saturating_add(unit.dependencies.len()));
    let traversal_steps = roots.len().saturating_add(edge_count).saturating_mul(2).saturating_add(1);
    let mut ordered = Vec::with_capacity(roots.len());
    let mut temporary = BTreeSet::new();
    let mut permanent = BTreeSet::new();
    let mut pending = Vec::with_capacity(traversal_steps.min(units.len()));
    for root in roots {
        if permanent.contains(root.as_str()) {
            continue;
        }
        pending.push((root.as_str(), false));
        for _ in 0..traversal_steps {
            let Some((id, exiting)) = pending.pop() else {
                break;
            };
            if exiting {
                temporary.remove(id);
                if permanent.insert(id) {
                    ordered.push(String::from(id));
                }
                continue;
            }
            if permanent.contains(id) {
                continue;
            }
            if !temporary.insert(id) {
                return Err(PlanBlocker::new(
                    "dependency-cycle",
                    id,
                    &alloc::format!("target topology contains a dependency cycle at unit {id}"),
                ));
            }
            pending.push((id, true));
            pending.extend(by_id[id].dependencies.iter().rev().map(|dependency| (dependency.as_str(), false)));
        }
        if !pending.is_empty() {
            return Err(PlanBlocker::new(
                "topology-traversal-limit-exceeded",
                root,
                &alloc::format!("target topology traversal exceeded {traversal_steps} bounded steps"),
            ));
        }
    }
    debug_assert!(ordered.len() <= units.len());
    debug_assert!(roots.iter().all(|root| permanent.contains(root.as_str())));
    Ok(ordered)
}

fn validate_existing_topology<'a>(
    roots: &[String],
    units: &'a [ExistingTopologyUnit],
) -> Result<BTreeMap<&'a str, &'a ExistingTopologyUnit>, PlanBlocker> {
    if count_exceeds(units.len(), MAX_UNITS) || count_exceeds(roots.len(), MAX_UNITS) {
        return Err(PlanBlocker::new("unit-limit", "units", "selected topology exceeds the admitted unit bound"));
    }
    let mut by_id = BTreeMap::new();
    let mut ranks = BTreeSet::new();
    for unit in units {
        if unit.unit_id.is_empty() {
            return Err(PlanBlocker::new("unit-identity", "units", "selected topology has an empty unit identity"));
        }
        if by_id.insert(unit.unit_id.as_str(), unit).is_some() || !ranks.insert(unit.execution_rank) {
            return Err(PlanBlocker::new(
                "duplicate-unit",
                &unit.unit_id,
                "selected topology contains a duplicate unit identity or rank",
            ));
        }
        if count_exceeds(unit.dependencies.len(), MAX_UNITS) {
            return Err(PlanBlocker::new(
                "unit-dependency-limit",
                &unit.unit_id,
                "selected topology dependencies exceed the admitted bound",
            ));
        }
    }
    for unit in units {
        for producer in &unit.dependencies {
            if !by_id.contains_key(producer.as_str()) {
                return Err(PlanBlocker::new(
                    "missing-unit-producer",
                    &unit.unit_id,
                    "selected topology has no declared producer unit",
                ));
            }
        }
    }
    for root in roots {
        if root.is_empty() || !by_id.contains_key(root.as_str()) {
            return Err(PlanBlocker::new(
                "missing-topology-root",
                if root.is_empty() { "roots" } else { root },
                "selected topology root is absent from supplied unit facts",
            ));
        }
    }
    debug_assert_eq!(by_id.len(), units.len());
    debug_assert!(roots.iter().all(|root| by_id.contains_key(root.as_str())));
    Ok(by_id)
}

/// The accepted Rust-plan surface classification, independent of presentation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustPlanCompatibilityDecision {
    pub compatibility_class: String,
    pub status: String,
    pub surface_ids: Vec<String>,
    pub blocker_classes: Vec<String>,
    pub non_claims: Vec<String>,
}

/// Preserve the existing Cargo-oracle / bounded-Cargo-free surface claims.
pub fn classify_rust_plan_compatibility(no_cargo_oracle: bool, blockers: &[String]) -> RustPlanCompatibilityDecision {
    let mut blocker_classes = blockers.to_vec();
    blocker_classes.sort();
    blocker_classes.dedup();
    let (compatibility_class, status, mut surface_ids) = if !no_cargo_oracle {
        ("cargo-oracle-evidence", "oracle", vec![String::from("cargo-oracle-reference")])
    } else if blockers.is_empty() {
        ("cargo-free-bounded-topology", "supported", vec![
            String::from("path-workspace-basic"),
            String::from("local-path-dependency"),
            String::from("workspace-inheritance"),
            String::from("source-closure-digest"),
            String::from("unit-graph-facts"),
        ])
    } else {
        ("blocked-unsupported-surface", "blocked", vec![String::from("blocked-unsupported-surface")])
    };
    surface_ids.sort();
    surface_ids.dedup();
    let mut non_claims = if no_cargo_oracle {
        vec![
            String::from("bounded-path-workspace-only"),
            String::from("not-full-cargo-feature-resolution"),
            String::from("declared-vendor-and-captured-git-only"),
            String::from("not-network-or-ambient-cargo-source-resolution"),
        ]
    } else {
        vec![String::from("cargo-used-for-oracle-metadata-and-unit-graph")]
    };
    non_claims.extend([
        String::from("not-full-cargo-compatibility"),
        String::from("not-compiler-correctness"),
        String::from("not-release-reproducibility"),
        String::from("not-bootstrap-correctness"),
    ]);
    non_claims.sort();
    non_claims.dedup();
    debug_assert!(blocker_classes.windows(2).all(|pair| pair[0] < pair[1]));
    debug_assert!(surface_ids.windows(2).all(|pair| pair[0] < pair[1]));
    RustPlanCompatibilityDecision {
        compatibility_class: String::from(compatibility_class),
        status: String::from(status),
        surface_ids,
        blocker_classes,
        non_claims,
    }
}

/// Classify actual native-unit observations without assigning synthetic unit IDs.
pub fn classify_existing_unit_observations(
    effects: &[ExistingUnitEffect],
    observations: &[UnitObservation],
) -> PlanExecutionOutcome {
    classify_observations(effects.iter().map(|effect| (&effect.effect_id, &effect.unit_id)), observations)
}

fn classify_observations<'a>(
    effects: impl core::iter::ExactSizeIterator<Item = (&'a EffectId, &'a UnitId)>,
    observations: &[UnitObservation],
) -> PlanExecutionOutcome {
    if count_exceeds(effects.len(), MAX_UNITS) || count_exceeds(observations.len(), MAX_UNITS) {
        return PlanExecutionOutcome::InputLimitExceeded;
    }
    let planned: BTreeMap<&str, &str> = effects.map(|(effect, unit)| (effect.0.as_str(), unit.0.as_str())).collect();
    let mut seen = BTreeSet::new();
    let mut accepted = BTreeSet::new();
    let mut unknown: u32 = 0;
    let mut failed: u32 = 0;
    for observation in observations {
        let effect = observation.effect_id.0.as_str();
        if !seen.insert(effect) {
            let Some(next) = unknown.checked_add(1) else {
                return PlanExecutionOutcome::InputLimitExceeded;
            };
            unknown = next;
            continue;
        }
        if planned.get(effect).copied() != Some(observation.unit_id.0.as_str())
            || !observation_exit_is_consistent(observation)
        {
            let Some(next) = unknown.checked_add(1) else {
                return PlanExecutionOutcome::InputLimitExceeded;
            };
            unknown = next;
            continue;
        }
        accepted.insert(effect);
        if observation.status != UnitObservationStatus::Succeeded {
            let Some(next) = failed.checked_add(1) else {
                return PlanExecutionOutcome::InputLimitExceeded;
            };
            failed = next;
        }
    }
    debug_assert!(accepted.iter().all(|effect| planned.contains_key(effect)));
    debug_assert!(accepted.len() <= seen.len());
    let Some(missing) = planned.len().checked_sub(accepted.len()) else {
        return PlanExecutionOutcome::InputLimitExceeded;
    };
    if unknown > 0 || missing > 0 {
        return PlanExecutionOutcome::Rejected {
            unknown_effect_count: unknown,
            missing_effect_count: match u32::try_from(missing) {
                Ok(count) => count,
                Err(_) => return PlanExecutionOutcome::InputLimitExceeded,
            },
        };
    }
    if failed > 0 {
        return PlanExecutionOutcome::Failed {
            failed_effect_count: failed,
        };
    }
    PlanExecutionOutcome::Completed
}

fn observation_exit_is_consistent(observation: &UnitObservation) -> bool {
    match observation.status {
        UnitObservationStatus::Succeeded => observation.exit_code.is_none_or(|code| code == 0),
        UnitObservationStatus::Failed => observation.exit_code.is_none_or(|code| code != 0),
        UnitObservationStatus::Skipped => observation.exit_code.is_none(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn observations_over_admitted_unit_limit_are_not_counted_as_valid_execution() {
        let observations = (0..=MAX_UNITS)
            .map(|_| UnitObservation {
                effect_id: EffectId(String::from("effect")),
                unit_id: UnitId(String::from("unit")),
                status: UnitObservationStatus::Succeeded,
                exit_code: Some(0),
                diagnostics_code: None,
            })
            .collect::<Vec<_>>();
        assert_eq!(classify_existing_unit_observations(&[], &observations), PlanExecutionOutcome::InputLimitExceeded);
    }
}
