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
pub const MAX_RESOLVED_EFFECT_BYTES: usize = 8 * 1024 * 1024;
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
    let mut explicit = false;
    let mut previous_codegen = false;
    for arg in reviewable_args {
        let is_link_option = |option: &str| {
            option == "link-self-contained"
                || option.strip_prefix("link-self-contained").is_some_and(|suffix| suffix.starts_with('='))
        };
        if previous_codegen && is_link_option(arg) {
            explicit = true;
        }
        if arg.strip_prefix("-C").is_some_and(is_link_option) {
            explicit = true;
        }
        previous_codegen = arg == "-C";
    }
    let mut arguments = reviewable_args.to_vec();
    if !explicit {
        arguments.push(String::from("-C"));
        arguments.push(String::from("link-self-contained=no"));
    }
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
                    "windows-wtf16le" => bytes.len() % 2 == 0 && !bytes.chunks_exact(2).any(|unit| unit == [0, 0]),
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
    if planned.target_kind != "custom-build"
        || primary.facts.role != ProcessEffectRole::RustCompiler
        || primary.facts.attempt != 0
        || primary.facts.unit_id != planned.unit_id
        || primary.facts.effect_id != planned.effect_id
        || restored.effect_id != planned.effect_id
        || restored.unit_id != planned.unit_id
        || restored.compiler_resolved_blake3 != primary.resolved_blake3
    {
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
}

/// Admit already selected native derivations without rewriting their Cargo/rustc
/// semantics. The shell supplies a stable tie-break rank, and this core orders
/// internal producer edges. Paths and external artifacts remain declared inputs.
pub fn plan_existing_unit_effects(units: Vec<ExistingUnitFacts>) -> Result<Vec<ExistingUnitEffect>, Vec<PlanBlocker>> {
    let order = ordered_existing_unit_indices(&units)?;
    Ok(match order {
        None => units.into_iter().map(unit_into_effect).collect(),
        Some(indices) => {
            let mut slots = units.into_iter().map(Some).collect::<Vec<_>>();
            indices
                .into_iter()
                .map(|index| unit_into_effect(slots[index].take().expect("admitted unit index")))
                .collect()
        }
    })
}

fn ordered_existing_unit_indices(units: &[ExistingUnitFacts]) -> Result<Option<Vec<usize>>, Vec<PlanBlocker>> {
    let mut blockers = Vec::new();
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
    let mut by_id = BTreeMap::new();
    let mut by_order = BTreeSet::new();
    let mut already_ordered = true;
    let mut previous_order = None;
    for (index, unit) in units.iter().enumerate() {
        if let Some(previous) = previous_order {
            already_ordered &= previous < unit.execution_order;
        }
        previous_order = Some(unit.execution_order);
        if unit.unit_id.is_empty()
            || unit.package_id.is_empty()
            || unit.target_name.is_empty()
            || unit.target_kind.is_empty()
            || unit.execution_kind.is_empty()
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
        if !matches!(unit.execution_kind.as_str(), "target" | "host" | "host-dependency") {
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
        if unit.arguments.is_empty()
            || unit.expected_outputs.is_empty()
            || unit.arguments.iter().any(String::is_empty)
            || unit.input_identities.iter().any(String::is_empty)
            || unit.expected_outputs.iter().any(String::is_empty)
        {
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
    }
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
                already_ordered &= units[producer_index].execution_order < unit.execution_order;
            } else {
                blockers.push(PlanBlocker::new(
                    "missing-unit-producer",
                    &unit.unit_id,
                    "declared unit producer is absent from the selected topology",
                ));
            }
        }
    }
    if !blockers.is_empty() {
        blockers.sort();
        blockers.dedup();
        return Err(blockers);
    }
    if already_ordered {
        return Ok(None);
    }
    // The native derivation order is only a tie-break. The core derives the
    // executable order from resolved producer edges, and rejects cycles.
    let mut indegrees = BTreeMap::<&str, usize>::new();
    let mut consumers = BTreeMap::<&str, Vec<usize>>::new();
    for (index, unit) in units.iter().enumerate() {
        indegrees.insert(unit.unit_id.as_str(), unit.dependency_unit_ids.len());
        for producer in &unit.dependency_unit_ids {
            consumers.entry(producer.as_str()).or_default().push(index);
        }
    }
    let mut ready = BTreeSet::new();
    for unit in units {
        if indegrees[unit.unit_id.as_str()] == 0 {
            ready.insert((unit.execution_order, unit.unit_id.as_str()));
        }
    }
    let mut ordered = Vec::with_capacity(units.len());
    while let Some((_, id)) = ready.pop_first() {
        ordered.push(by_id[id]);
        if let Some(dependents) = consumers.get(id) {
            for &index in dependents {
                let dependent = &units[index];
                let remaining = indegrees.get_mut(dependent.unit_id.as_str()).expect("admitted unit");
                *remaining -= 1;
                if *remaining == 0 {
                    ready.insert((dependent.execution_order, dependent.unit_id.as_str()));
                }
            }
        }
    }
    if ordered.len() != units.len() {
        let mut cycle_blockers = units
            .iter()
            .filter(|unit| indegrees[unit.unit_id.as_str()] != 0)
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
    Ok(Some(ordered))
}

fn unit_into_effect(unit: ExistingUnitFacts) -> ExistingUnitEffect {
    let mut effect_id = String::with_capacity(EFFECT_ID_PREFIX.len() + unit.unit_id.len());
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
    let mut blockers = Vec::new();
    let subject = planned.unit_id.0.as_str();
    if facts.unit_id != planned.unit_id || facts.effect_id != planned.effect_id {
        blockers.push(PlanBlocker::new(
            "resolved-effect-identity",
            subject,
            "resolved effect must retain the selected unit and effect identities",
        ));
    }
    match (facts.role, facts.attempt) {
        (ProcessEffectRole::RustCompiler, 0)
            if prior.is_none() && restored.is_none() && facts.prior_attempt_identity.is_none() => {}
        (ProcessEffectRole::RustCompiler, 1) | (ProcessEffectRole::BuildScriptRun, 0) => {
            let expected_status = if facts.role == ProcessEffectRole::RustCompiler {
                UnitObservationStatus::Failed
            } else {
                UnitObservationStatus::Succeeded
            };
            let correct_role = facts.role != ProcessEffectRole::BuildScriptRun || planned.target_kind == "custom-build";
            let valid_prior = prior
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
                            _ => false,
                        }
                        && observation.attempt <= 1
                        && (facts.role != ProcessEffectRole::RustCompiler || observation.attempt == 0)
                })
                .and_then(|observation| domain_digest(RESOLVED_OBSERVATION_DOMAIN, observation).ok())
                .is_some_and(|identity| facts.prior_attempt_identity.as_ref() == Some(&identity));
            let valid_restored = facts.role == ProcessEffectRole::BuildScriptRun
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
            if !correct_role || !(valid_prior || valid_restored) {
                blockers.push(PlanBlocker::new(
                    "resolved-effect-predecessor",
                    subject,
                    "fallback and build-script processes require matching observed compiler or restored output evidence",
                ));
            }
        }
        _ => blockers.push(PlanBlocker::new(
            "unsupported-process-attempt",
            subject,
            "resolved process role or attempt is outside the bounded unit topology",
        )),
    }
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
        _ => {}
    }
    if !facts.executable.is_valid(true)
        || facts.toolchain_identity.is_empty()
        || facts.working_directory.as_ref().is_some_and(|cwd| !cwd.is_valid(true))
        || facts.arguments.iter().any(|arg| !arg.is_valid(false))
        || facts.environment.iter().any(|(_, value)| !value.is_valid(false))
        || facts.input_identities.iter().any(String::is_empty)
        || facts.expected_outputs.iter().any(String::is_empty)
    {
        blockers.push(PlanBlocker::new(
            "incomplete-resolved-effect",
            subject,
            "resolved executable, toolchain, arguments, environment, inputs and outputs must be declared values",
        ));
    }
    if count_exceeds(facts.arguments.len(), MAX_ARGS_PER_EFFECT)
        || count_exceeds(facts.environment.len(), MAX_ENVIRONMENT_ENTRIES_PER_EFFECT)
        || count_exceeds(facts.input_identities.len(), MAX_ARGS_PER_EFFECT)
        || count_exceeds(facts.expected_outputs.len(), MAX_ARGS_PER_EFFECT)
        || count_exceeds(facts.dependency_observations.len(), MAX_UNITS)
    {
        blockers.push(PlanBlocker::new(
            "resolved-effect-limit",
            subject,
            "resolved effect collections exceed the admitted bounds",
        ));
    }
    let mut total_bytes = facts
        .executable
        .byte_len()
        .saturating_add(facts.toolchain_identity.len())
        .saturating_add(facts.effect_id.0.len())
        .saturating_add(facts.unit_id.0.len())
        .saturating_add(facts.prior_attempt_identity.as_ref().map_or(0, |identity| identity.as_str().len()));
    let mut oversized_word = false;
    for word in facts.arguments.iter().chain(facts.environment.iter().map(|(_, value)| value)) {
        total_bytes = total_bytes.saturating_add(word.byte_len());
        oversized_word |= word.byte_len() > MAX_PROCESS_WORD_BYTES;
    }
    if oversized_word {
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
    for identity in &planned.input_identities {
        if !facts.input_identities.contains(identity) {
            blockers.push(PlanBlocker::new(
                "missing-resolved-input",
                subject,
                "a selected derivation input was omitted from the final process",
            ));
        }
    }
    let declared: BTreeSet<&str> = planned.dependency_unit_ids.iter().map(|id| id.0.as_str()).collect();
    let mut observed = BTreeSet::new();
    for producer in &facts.dependency_observations {
        total_bytes = total_bytes
            .saturating_add(producer.producer_unit_id.0.len())
            .saturating_add(producer.artifact_identity.len());
        if !declared.contains(producer.producer_unit_id.0.as_str())
            || producer.artifact_identity.is_empty()
            || producer.artifact_identity.len() > MAX_PROCESS_WORD_BYTES
            || !facts.input_identities.contains(&producer.artifact_identity)
            || !observed.insert(producer.producer_unit_id.0.as_str())
        {
            blockers.push(PlanBlocker::new(
                "invalid-producer-observation",
                subject,
                "resolved producer must be declared once with a matching artifact input identity",
            ));
        }
    }
    if total_bytes > MAX_RESOLVED_EFFECT_BYTES
        || facts.executable.byte_len() > MAX_PROCESS_WORD_BYTES
        || facts.working_directory.as_ref().is_some_and(|cwd| cwd.byte_len() > MAX_PROCESS_WORD_BYTES)
    {
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
    if !blockers.is_empty() {
        blockers.sort();
        blockers.dedup();
        return Err(blockers);
    }
    let resolved_blake3 = domain_digest(RESOLVED_EFFECT_DOMAIN, &facts).map_err(|_| {
        vec![PlanBlocker::new(
            "resolved-effect-identity-failed",
            subject,
            "resolved process facts could not be canonically identified",
        )]
    })?;
    Ok(ResolvedUnitEffect { facts, resolved_blake3 })
}

/// Bind one observed result to its exact admitted process attempt and outputs.
pub fn classify_resolved_process_observation(
    effect: &ResolvedUnitEffect,
    observation: &ResolvedProcessObservation,
) -> Result<Blake3Digest, PlanBlocker> {
    let facts = &effect.facts;
    if observation.effect_id != facts.effect_id
        || observation.unit_id != facts.unit_id
        || observation.resolved_blake3 != effect.resolved_blake3
        || observation.role != facts.role
        || observation.attempt != facts.attempt
    {
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
        if name.is_empty()
            || identity.is_empty()
            || !facts.expected_outputs.contains(name)
            || !names.insert(name.as_str())
        {
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
    let edge_count = units.iter().fold(0usize, |count, unit| count.saturating_add(unit.dependencies.len()));
    let traversal_steps = roots.len().saturating_add(edge_count).saturating_mul(2).saturating_add(1);
    let mut ordered = Vec::with_capacity(roots.len());
    let mut temporary = BTreeSet::new();
    let mut permanent = BTreeSet::new();
    let mut pending = Vec::with_capacity(traversal_steps.min(MAX_UNITS as usize));
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
    Ok(ordered)
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
    effects: impl Iterator<Item = (&'a EffectId, &'a UnitId)>,
    observations: &[UnitObservation],
) -> PlanExecutionOutcome {
    let planned: BTreeMap<&str, &str> = effects.map(|(effect, unit)| (effect.0.as_str(), unit.0.as_str())).collect();
    let mut seen = BTreeSet::new();
    let mut accepted = BTreeSet::new();
    let mut unknown: u32 = 0;
    let mut failed: u32 = 0;
    for observation in observations {
        let effect = observation.effect_id.0.as_str();
        if !seen.insert(effect) {
            unknown = unknown.saturating_add(1);
            continue;
        }
        if planned.get(effect).copied() != Some(observation.unit_id.0.as_str())
            || !observation_exit_is_consistent(observation)
        {
            unknown = unknown.saturating_add(1);
            continue;
        }
        accepted.insert(effect);
        if observation.status != UnitObservationStatus::Succeeded {
            failed = failed.saturating_add(1);
        }
    }
    let missing = planned.len().saturating_sub(accepted.len());
    if unknown > 0 || missing > 0 {
        return PlanExecutionOutcome::Rejected {
            unknown_effect_count: unknown,
            missing_effect_count: u32::try_from(missing).unwrap_or(u32::MAX),
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
