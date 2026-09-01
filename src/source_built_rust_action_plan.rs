//! Pure action authority for source-built Rust unit execution.
//!
//! The shell owns files, seccomp installation, and process execution. This
//! module validates pre-execution unit authority and reconciles normalized
//! child-process observations after execution.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt;
use std::path::Path;

use serde::Deserialize;
use serde::Serialize;

pub(crate) const RUST_CHILD_ACTION_PLAN_SCHEMA: &str = "mantle-source-built-rust-child-action-plan-v1";
pub(crate) const RUST_CHILD_ACTION_RECONCILIATION_SCHEMA: &str =
    "mantle-source-built-rust-child-action-reconciliation-v1";
pub(crate) const RUST_CHILD_ACTION_AUTHORITY_SCHEMA: &str = "mantle-source-built-rust-child-action-authority-v1";
pub(crate) const RUST_CHILD_ACTION_AUDIT_SCHEMA: &str = "mantle-source-built-rust-child-action-audit-v1";
const PLAN_DIGEST_CONTEXT: &str = "mantle-source-built-rust-child-action-plan-v1";
const RECONCILIATION_DIGEST_CONTEXT: &str = "mantle-source-built-rust-child-action-reconciliation-v1";
const AUTHORITY_DIGEST_CONTEXT: &str = "mantle-source-built-rust-child-action-authority-v1";
const AUDIT_DIGEST_CONTEXT: &str = "mantle-source-built-rust-child-action-audit-v1";
const UNIT_OUTPUT_IDENTITY_CONTEXT: &str = "mantle-source-built-rust-unit-output-v1";
const ACTION_ID_CONTEXT: &str = "mantle-source-built-rust-action-id-v1";
const SOURCE_IDENTITY_CONTEXT: &str = "mantle-source-built-rust-source-identity-v1";
const BLAKE3_HEX_LENGTH: usize = 64;
const RUST_UNIT_COUNT_MAX: u32 = 16_384;
const RUST_ACTION_COUNT_MAX: u32 = RUST_UNIT_COUNT_MAX * 2;
const RUST_EXEC_EVENT_COUNT_MAX: u32 = 262_144;
const FIXED_EXECUTABLE_COUNT_MAX: u32 = 256;
const TEXT_BYTES_MAX: usize = 4_096;
const ACTION_EVENT_COUNT_MIN: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RustChildActionPlanInput {
    pub(crate) stage_id: String,
    pub(crate) resources: RustActionResourceLimits,
    pub(crate) fixed_executables: Vec<RustFixedExecutableAuthority>,
    pub(crate) units: Vec<RustUnitActionInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RustActionResourceLimits {
    pub(crate) parallel_jobs_max: u32,
    pub(crate) open_file_descriptors_max: u32,
    pub(crate) storage_bytes_max: u64,
    pub(crate) exec_events_per_action_max: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RustChildActionAuthority {
    pub(crate) schema: String,
    pub(crate) stage_id: String,
    pub(crate) resources: RustActionResourceLimits,
    pub(crate) fixed_executables: Vec<RustFixedExecutableAuthority>,
    pub(crate) authority_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum RustFixedExecutableKind {
    Rustc,
    Linker,
    CCompiler,
    CxxCompiler,
    PkgConfig,
    NativeHelper,
    Shell,
    CompilerPolicyAdapter,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RustFixedExecutableAuthority {
    pub(crate) authority_id: String,
    pub(crate) producer_action_id: String,
    pub(crate) output_identity_blake3: String,
    pub(crate) path: String,
    pub(crate) digest_blake3: String,
    pub(crate) kind: RustFixedExecutableKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RustUnitActionInput {
    pub(crate) unit_id: String,
    pub(crate) target_kind: String,
    pub(crate) producer_unit_ids: Vec<String>,
    pub(crate) input_authority_ids: Vec<String>,
    pub(crate) declared_outputs: Vec<String>,
    pub(crate) source_digest_blake3: String,
    pub(crate) rustc_args_digest_blake3: String,
    pub(crate) environment_digest_blake3: String,
}

#[derive(Debug, Default)]
struct RustDependencyProducerIndex {
    target_lib_by_package: BTreeMap<(String, String), BTreeSet<String>>,
    target_lib_by_exact_name: BTreeMap<(String, String, String), BTreeSet<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RustChildActionPlan {
    pub(crate) schema: String,
    pub(crate) stage_id: String,
    pub(crate) adapter: String,
    pub(crate) resources: RustActionResourceLimits,
    pub(crate) fixed_executables: Vec<RustFixedExecutableAuthority>,
    pub(crate) action_count: u32,
    pub(crate) local_only: bool,
    pub(crate) cache_only_completion_allowed: bool,
    pub(crate) actions: Vec<RustChildAction>,
    pub(crate) blockers: Vec<String>,
    pub(crate) plan_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RustChildAction {
    pub(crate) action_id: String,
    pub(crate) action_id_blake3: String,
    pub(crate) unit_id_blake3: String,
    pub(crate) phase: RustChildActionPhase,
    pub(crate) producer_action_ids: Vec<String>,
    pub(crate) input_authority_ids: Vec<String>,
    pub(crate) output_identities_blake3: Vec<String>,
    pub(crate) executable: RustChildExecutableAuthority,
    pub(crate) permitted_child_fixed_authority_ids: Vec<String>,
    pub(crate) resources: RustActionResourceLimits,
    pub(crate) local_only: bool,
    pub(crate) event_count_min: u32,
    pub(crate) event_count_max: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum RustChildActionPhase {
    CompileUnit,
    RunBuildScript,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub(crate) enum RustChildExecutableAuthority {
    Fixed {
        authority_id: String,
    },
    Produced {
        producer_action_id: String,
        output_identity_blake3: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RustChildExecObservation {
    pub(crate) action_id: String,
    pub(crate) executable: RustObservedExecutableAuthority,
    pub(crate) resolved_path: String,
    pub(crate) digest_blake3: String,
    pub(crate) policy_decision: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub(crate) enum RustObservedExecutableAuthority {
    Fixed {
        authority_id: String,
    },
    Produced {
        producer_action_id: String,
        output_identity_blake3: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RustChildActionAudit {
    pub(crate) schema: String,
    pub(crate) action_plan_digest_blake3: String,
    pub(crate) raw_event_count: u32,
    pub(crate) assigned_event_count: u32,
    pub(crate) raw_events: Vec<crate::protected_exec::ProtectedSeccompAuditEvent>,
    pub(crate) observations: Vec<RustChildExecObservation>,
    pub(crate) promotions: Vec<crate::protected_exec::OutputPromotionRecord>,
    pub(crate) audit_digest_blake3: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RustChildActionReconciliation {
    pub(crate) schema: String,
    pub(crate) action_plan_digest_blake3: String,
    pub(crate) planned_action_count: u32,
    pub(crate) matched_action_count: u32,
    pub(crate) observed_event_count: u32,
    pub(crate) matched_event_count: u32,
    pub(crate) unknown_event_ids_blake3: Vec<String>,
    pub(crate) denied_event_ids_blake3: Vec<String>,
    pub(crate) drifted_event_ids_blake3: Vec<String>,
    pub(crate) missing_action_ids_blake3: Vec<String>,
    pub(crate) overbound_action_ids_blake3: Vec<String>,
    pub(crate) local_only: bool,
    pub(crate) blockers: Vec<String>,
    pub(crate) reconciliation_digest_blake3: String,
}

impl RustChildActionReconciliation {
    pub(crate) fn is_complete(&self) -> bool {
        self.blockers.is_empty()
            && self.planned_action_count == self.matched_action_count
            && self.observed_event_count == self.matched_event_count
            && self.local_only
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RustChildActionPlanErrorKind {
    InvalidInput,
    IncompleteGraph,
    #[cfg(test)]
    IncompleteReconciliation,
    Serialization,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RustChildActionPlanError {
    pub(crate) kind: RustChildActionPlanErrorKind,
    pub(crate) message: String,
}

impl fmt::Display for RustChildActionPlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl std::error::Error for RustChildActionPlanError {}

pub(crate) fn rust_child_action_authority(
    stage_id: String,
    resources: RustActionResourceLimits,
    mut fixed_executables: Vec<RustFixedExecutableAuthority>,
) -> Result<RustChildActionAuthority, RustChildActionPlanError> {
    validate_text("stage_id", &stage_id)?;
    validate_resources(&resources)?;
    let fixed = normalized_fixed_executables(fixed_executables.clone())?;
    sole_rustc_authority(&fixed)?;
    fixed_executables = fixed.into_values().collect();
    let mut authority = RustChildActionAuthority {
        schema: RUST_CHILD_ACTION_AUTHORITY_SCHEMA.to_string(),
        stage_id,
        resources,
        fixed_executables,
        authority_digest_blake3: String::new(),
    };
    authority.authority_digest_blake3 = authority_digest(&authority)?;
    validate_rust_child_action_authority(&authority)?;
    assert!(!authority.fixed_executables.is_empty());
    assert_eq!(authority.authority_digest_blake3.len(), BLAKE3_HEX_LENGTH);
    Ok(authority)
}

pub(crate) fn validate_rust_child_action_authority(
    authority: &RustChildActionAuthority,
) -> Result<(), RustChildActionPlanError> {
    if authority.schema != RUST_CHILD_ACTION_AUTHORITY_SCHEMA {
        return Err(plan_error(
            RustChildActionPlanErrorKind::InvalidInput,
            "unsupported Rust child action authority schema",
        ));
    }
    validate_text("stage_id", &authority.stage_id)?;
    validate_resources(&authority.resources)?;
    validate_digest("authority", &authority.authority_digest_blake3)?;
    let fixed = normalized_fixed_executables(authority.fixed_executables.clone())?;
    sole_rustc_authority(&fixed)?;
    if fixed.len() != authority.fixed_executables.len() {
        return Err(plan_error(
            RustChildActionPlanErrorKind::InvalidInput,
            "Rust child action authority executable count changed",
        ));
    }
    if authority_digest(authority)? != authority.authority_digest_blake3 {
        return Err(plan_error(
            RustChildActionPlanErrorKind::InvalidInput,
            "Rust child action authority digest mismatch",
        ));
    }
    Ok(())
}

pub(crate) fn plan_rust_child_actions(
    input: RustChildActionPlanInput,
) -> Result<RustChildActionPlan, RustChildActionPlanError> {
    validate_plan_input(&input)?;
    let units = normalized_units(input.units)?;
    validate_unit_graph(&units)?;
    let fixed = normalized_fixed_executables(input.fixed_executables)?;
    let rustc = sole_rustc_authority(&fixed)?;
    let child_authority_ids = fixed.keys().cloned().collect::<Vec<_>>();
    let mut actions = Vec::with_capacity(units.len().saturating_mul(2));
    for unit in units.values() {
        let compile = compile_action(unit, &units, rustc, &child_authority_ids, &input.resources)?;
        let compile_id = compile.action_id.clone();
        actions.push(compile);
        if unit.target_kind == "custom-build" {
            actions.push(build_script_action(unit, &compile_id, &child_authority_ids, &input.resources)?);
        }
    }
    actions.sort_by(|left, right| left.action_id.cmp(&right.action_id));
    let action_count = bounded_count("Rust child action", actions.len(), RUST_ACTION_COUNT_MAX)?;
    let mut plan = RustChildActionPlan {
        schema: RUST_CHILD_ACTION_PLAN_SCHEMA.to_string(),
        stage_id: input.stage_id,
        adapter: "native-rust-unit-graph-v1".to_string(),
        resources: input.resources,
        fixed_executables: fixed.into_values().collect(),
        action_count,
        local_only: true,
        cache_only_completion_allowed: false,
        actions,
        blockers: Vec::new(),
        plan_digest_blake3: String::new(),
    };
    plan.plan_digest_blake3 = plan_digest(&plan)?;
    validate_rust_child_action_plan(&plan)?;
    assert_eq!(usize::try_from(plan.action_count).ok(), Some(plan.actions.len()));
    debug_assert!(!plan.actions.is_empty());
    Ok(plan)
}

fn validate_plan_input(input: &RustChildActionPlanInput) -> Result<(), RustChildActionPlanError> {
    validate_text("stage_id", &input.stage_id)?;
    validate_resources(&input.resources)?;
    bounded_count("fixed executable", input.fixed_executables.len(), FIXED_EXECUTABLE_COUNT_MAX)?;
    bounded_count("Rust unit", input.units.len(), RUST_UNIT_COUNT_MAX)?;
    if input.fixed_executables.is_empty() || input.units.is_empty() {
        return Err(plan_error(
            RustChildActionPlanErrorKind::InvalidInput,
            "Rust child action plan requires fixed executables and units",
        ));
    }
    Ok(())
}

fn validate_resources(resources: &RustActionResourceLimits) -> Result<(), RustChildActionPlanError> {
    if resources.parallel_jobs_max == 0
        || resources.open_file_descriptors_max == 0
        || resources.storage_bytes_max == 0
        || resources.exec_events_per_action_max < ACTION_EVENT_COUNT_MIN
    {
        return Err(plan_error(
            RustChildActionPlanErrorKind::InvalidInput,
            "Rust child action resource limits must be positive",
        ));
    }
    Ok(())
}

fn normalized_units(
    units: Vec<RustUnitActionInput>,
) -> Result<BTreeMap<String, RustUnitActionInput>, RustChildActionPlanError> {
    let mut normalized = BTreeMap::new();
    for mut unit in units {
        validate_unit(&unit)?;
        unit.producer_unit_ids.sort();
        unit.producer_unit_ids.dedup();
        unit.input_authority_ids.sort();
        unit.input_authority_ids.dedup();
        unit.declared_outputs.sort();
        unit.declared_outputs.dedup();
        let unit_id = unit.unit_id.clone();
        if normalized.insert(unit_id.clone(), unit).is_some() {
            return Err(plan_error(
                RustChildActionPlanErrorKind::IncompleteGraph,
                &format!("duplicate Rust unit action input: {unit_id}"),
            ));
        }
    }
    Ok(normalized)
}

fn validate_unit(unit: &RustUnitActionInput) -> Result<(), RustChildActionPlanError> {
    validate_text("unit_id", &unit.unit_id)?;
    validate_text("target_kind", &unit.target_kind)?;
    validate_digest("source", &unit.source_digest_blake3)?;
    validate_digest("rustc args", &unit.rustc_args_digest_blake3)?;
    validate_digest("environment", &unit.environment_digest_blake3)?;
    if unit.declared_outputs.is_empty() {
        return Err(plan_error(
            RustChildActionPlanErrorKind::InvalidInput,
            &format!("Rust unit {} has no declared outputs", unit.unit_id),
        ));
    }
    for producer in &unit.producer_unit_ids {
        validate_text("producer_unit_id", producer)?;
    }
    for authority in &unit.input_authority_ids {
        validate_text("input_authority_id", authority)?;
    }
    for output in &unit.declared_outputs {
        validate_text("declared_output", output)?;
    }
    Ok(())
}

fn normalized_fixed_executables(
    executables: Vec<RustFixedExecutableAuthority>,
) -> Result<BTreeMap<String, RustFixedExecutableAuthority>, RustChildActionPlanError> {
    let mut normalized = BTreeMap::new();
    let mut paths = BTreeSet::new();
    for executable in executables {
        validate_fixed_executable(&executable)?;
        if !paths.insert(executable.path.clone()) {
            return Err(plan_error(
                RustChildActionPlanErrorKind::InvalidInput,
                &format!("duplicate fixed executable path: {}", executable.path),
            ));
        }
        let authority_id = executable.authority_id.clone();
        if normalized.insert(authority_id.clone(), executable).is_some() {
            return Err(plan_error(
                RustChildActionPlanErrorKind::InvalidInput,
                &format!("duplicate fixed executable authority: {authority_id}"),
            ));
        }
    }
    Ok(normalized)
}

fn validate_fixed_executable(executable: &RustFixedExecutableAuthority) -> Result<(), RustChildActionPlanError> {
    validate_text("authority_id", &executable.authority_id)?;
    validate_text("producer_action_id", &executable.producer_action_id)?;
    validate_digest("output identity", &executable.output_identity_blake3)?;
    validate_digest("executable", &executable.digest_blake3)?;
    if !Path::new(&executable.path).is_absolute() {
        return Err(plan_error(
            RustChildActionPlanErrorKind::InvalidInput,
            &format!("fixed executable path is not absolute: {}", executable.path),
        ));
    }
    if executable.output_identity_blake3 != executable.digest_blake3 {
        return Err(plan_error(
            RustChildActionPlanErrorKind::InvalidInput,
            &format!("fixed executable output identity differs from content digest: {}", executable.authority_id),
        ));
    }
    Ok(())
}

fn sole_rustc_authority(
    fixed: &BTreeMap<String, RustFixedExecutableAuthority>,
) -> Result<&RustFixedExecutableAuthority, RustChildActionPlanError> {
    let rustc = fixed
        .values()
        .filter(|authority| authority.kind == RustFixedExecutableKind::Rustc)
        .collect::<Vec<_>>();
    match rustc.as_slice() {
        [authority] => Ok(*authority),
        _ => Err(plan_error(
            RustChildActionPlanErrorKind::InvalidInput,
            &format!("Rust child action plan requires one rustc authority, found {}", rustc.len()),
        )),
    }
}

fn validate_unit_graph(units: &BTreeMap<String, RustUnitActionInput>) -> Result<(), RustChildActionPlanError> {
    let mut incoming = units.keys().map(|unit_id| (unit_id.clone(), 0_u32)).collect::<BTreeMap<_, _>>();
    let mut consumers = BTreeMap::<String, Vec<String>>::new();
    for unit in units.values() {
        for producer in &unit.producer_unit_ids {
            if producer == &unit.unit_id || !units.contains_key(producer) {
                return Err(plan_error(
                    RustChildActionPlanErrorKind::IncompleteGraph,
                    &format!("Rust unit {} has invalid producer {producer}", unit.unit_id),
                ));
            }
            increment_count(incoming.get_mut(&unit.unit_id), "Rust unit incoming edge")?;
            consumers.entry(producer.clone()).or_default().push(unit.unit_id.clone());
        }
    }
    let mut ready =
        incoming.iter().filter(|(_, count)| **count == 0).map(|(id, _)| id.clone()).collect::<BTreeSet<_>>();
    let mut visited = 0_u32;
    while let Some(unit_id) = ready.pop_first() {
        visited = visited
            .checked_add(1)
            .ok_or_else(|| plan_error(RustChildActionPlanErrorKind::InvalidInput, "Rust unit visit count overflow"))?;
        for consumer in consumers.get(&unit_id).into_iter().flatten() {
            let count = incoming.get_mut(consumer).expect("consumer was validated");
            *count = count.checked_sub(1).expect("incoming edge count is positive");
            if *count == 0 {
                ready.insert(consumer.clone());
            }
        }
    }
    if usize::try_from(visited).ok() != Some(units.len()) {
        return Err(plan_error(
            RustChildActionPlanErrorKind::IncompleteGraph,
            "Rust unit action graph contains a cycle",
        ));
    }
    Ok(())
}

fn increment_count(count: Option<&mut u32>, label: &str) -> Result<(), RustChildActionPlanError> {
    let count = count.ok_or_else(|| plan_error(RustChildActionPlanErrorKind::IncompleteGraph, label))?;
    *count = count.checked_add(1).ok_or_else(|| plan_error(RustChildActionPlanErrorKind::InvalidInput, label))?;
    Ok(())
}

fn compile_action(
    unit: &RustUnitActionInput,
    units: &BTreeMap<String, RustUnitActionInput>,
    rustc: &RustFixedExecutableAuthority,
    child_authority_ids: &[String],
    resources: &RustActionResourceLimits,
) -> Result<RustChildAction, RustChildActionPlanError> {
    let action_id = compile_action_id(&unit.unit_id);
    let producer_action_ids = unit
        .producer_unit_ids
        .iter()
        .map(|producer| units.get(producer).map(|_| compile_action_id(producer)))
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| plan_error(RustChildActionPlanErrorKind::IncompleteGraph, "Rust compile producer is missing"))?;
    let mut inputs = unit.input_authority_ids.clone();
    inputs.extend([
        format!("source:{}", unit.source_digest_blake3),
        format!("rustc-args:{}", unit.rustc_args_digest_blake3),
        format!("environment:{}", unit.environment_digest_blake3),
    ]);
    inputs.sort();
    inputs.dedup();
    let outputs = unit_output_identities(unit)?;
    Ok(action(
        action_id,
        &unit.unit_id,
        RustChildActionPhase::CompileUnit,
        producer_action_ids,
        inputs,
        outputs,
        RustChildExecutableAuthority::Fixed {
            authority_id: rustc.authority_id.clone(),
        },
        child_authority_ids,
        resources,
    ))
}

fn build_script_action(
    unit: &RustUnitActionInput,
    compile_action_id: &str,
    child_authority_ids: &[String],
    resources: &RustActionResourceLimits,
) -> Result<RustChildAction, RustChildActionPlanError> {
    let executable_identity = unit_output_identity(unit)?;
    let action_id = run_action_id(&unit.unit_id);
    Ok(action(
        action_id,
        &unit.unit_id,
        RustChildActionPhase::RunBuildScript,
        vec![compile_action_id.to_string()],
        vec![format!("produced-build-script:{executable_identity}")],
        vec![digest_text("build-script-metadata", &unit.unit_id)],
        RustChildExecutableAuthority::Produced {
            producer_action_id: compile_action_id.to_string(),
            output_identity_blake3: executable_identity,
        },
        child_authority_ids,
        resources,
    ))
}

#[allow(clippy::too_many_arguments)]
fn action(
    action_id: String,
    unit_id: &str,
    phase: RustChildActionPhase,
    mut producer_action_ids: Vec<String>,
    mut input_authority_ids: Vec<String>,
    mut output_identities_blake3: Vec<String>,
    executable: RustChildExecutableAuthority,
    child_authority_ids: &[String],
    resources: &RustActionResourceLimits,
) -> RustChildAction {
    producer_action_ids.sort();
    producer_action_ids.dedup();
    input_authority_ids.sort();
    input_authority_ids.dedup();
    output_identities_blake3.sort();
    output_identities_blake3.dedup();
    RustChildAction {
        action_id_blake3: digest_text(ACTION_ID_CONTEXT, &action_id),
        action_id,
        unit_id_blake3: digest_text("mantle-source-built-rust-unit-id-v1", unit_id),
        phase,
        producer_action_ids,
        input_authority_ids,
        output_identities_blake3,
        executable,
        permitted_child_fixed_authority_ids: child_authority_ids.to_vec(),
        resources: resources.clone(),
        local_only: true,
        event_count_min: ACTION_EVENT_COUNT_MIN,
        event_count_max: resources.exec_events_per_action_max,
    }
}

fn unit_output_identities(unit: &RustUnitActionInput) -> Result<Vec<String>, RustChildActionPlanError> {
    unit.declared_outputs
        .iter()
        .map(|output| {
            validate_text("declared output", output)?;
            Ok(digest_text(UNIT_OUTPUT_IDENTITY_CONTEXT, &format!("{}\0{output}", unit.unit_id)))
        })
        .collect()
}

fn unit_output_identity(unit: &RustUnitActionInput) -> Result<String, RustChildActionPlanError> {
    let identities = unit_output_identities(unit)?;
    let bytes = serde_json::to_vec(&identities)
        .map_err(|error| plan_error(RustChildActionPlanErrorKind::Serialization, &error.to_string()))?;
    Ok(digest_bytes(UNIT_OUTPUT_IDENTITY_CONTEXT, &bytes))
}

fn compile_action_id(unit_id: &str) -> String {
    format!("rust-unit:{}:compile", digest_text("mantle-source-built-rust-unit-id-v1", unit_id))
}

fn run_action_id(unit_id: &str) -> String {
    format!("rust-unit:{}:run-build-script", digest_text("mantle-source-built-rust-unit-id-v1", unit_id))
}

pub(crate) fn rust_child_action_for_unit<'a>(
    plan: &'a RustChildActionPlan,
    unit_id: &str,
    phase: RustChildActionPhase,
) -> Result<&'a RustChildAction, RustChildActionPlanError> {
    validate_text("unit_id", unit_id)?;
    let action_id = match phase {
        RustChildActionPhase::CompileUnit => compile_action_id(unit_id),
        RustChildActionPhase::RunBuildScript => run_action_id(unit_id),
    };
    let matches = plan.actions.iter().filter(|action| action.action_id == action_id).collect::<Vec<_>>();
    match matches.as_slice() {
        [action] => Ok(*action),
        _ => Err(plan_error(
            RustChildActionPlanErrorKind::IncompleteGraph,
            &format!("Rust action plan has {} matches for unit phase", matches.len()),
        )),
    }
}

pub(crate) fn validate_rust_child_action_plan(plan: &RustChildActionPlan) -> Result<(), RustChildActionPlanError> {
    if plan.schema != RUST_CHILD_ACTION_PLAN_SCHEMA || plan.adapter != "native-rust-unit-graph-v1" {
        return Err(plan_error(
            RustChildActionPlanErrorKind::InvalidInput,
            "unsupported Rust child action plan schema",
        ));
    }
    validate_text("stage_id", &plan.stage_id)?;
    validate_resources(&plan.resources)?;
    validate_digest("plan", &plan.plan_digest_blake3)?;
    if !plan.local_only || plan.cache_only_completion_allowed || !plan.blockers.is_empty() {
        return Err(plan_error(
            RustChildActionPlanErrorKind::InvalidInput,
            "Rust child action plan weakens local execution policy",
        ));
    }
    if usize::try_from(plan.action_count).ok() != Some(plan.actions.len()) || plan.actions.is_empty() {
        return Err(plan_error(RustChildActionPlanErrorKind::InvalidInput, "Rust child action count is inconsistent"));
    }
    validate_plan_authorities(plan)?;
    validate_plan_actions(plan)?;
    if plan_digest(plan)? != plan.plan_digest_blake3 {
        return Err(plan_error(RustChildActionPlanErrorKind::InvalidInput, "Rust child action plan digest mismatch"));
    }
    Ok(())
}

fn validate_plan_authorities(plan: &RustChildActionPlan) -> Result<(), RustChildActionPlanError> {
    let fixed = normalized_fixed_executables(plan.fixed_executables.clone())?;
    sole_rustc_authority(&fixed)?;
    if fixed.len() != plan.fixed_executables.len() {
        return Err(plan_error(RustChildActionPlanErrorKind::InvalidInput, "Rust fixed authority count changed"));
    }
    Ok(())
}

fn validate_plan_actions(plan: &RustChildActionPlan) -> Result<(), RustChildActionPlanError> {
    let fixed_ids = plan
        .fixed_executables
        .iter()
        .map(|authority| authority.authority_id.clone())
        .collect::<BTreeSet<_>>();
    let action_ids = plan.actions.iter().map(|action| action.action_id.clone()).collect::<BTreeSet<_>>();
    if action_ids.len() != plan.actions.len() {
        return Err(plan_error(RustChildActionPlanErrorKind::IncompleteGraph, "Rust child action IDs are not unique"));
    }
    for action in &plan.actions {
        validate_action(action, &action_ids, &fixed_ids, &plan.resources)?;
    }
    Ok(())
}

fn validate_action(
    action: &RustChildAction,
    action_ids: &BTreeSet<String>,
    fixed_ids: &BTreeSet<String>,
    resources: &RustActionResourceLimits,
) -> Result<(), RustChildActionPlanError> {
    validate_text("action_id", &action.action_id)?;
    validate_digest("action_id", &action.action_id_blake3)?;
    validate_digest("unit_id", &action.unit_id_blake3)?;
    if action.resources != *resources || !action.local_only {
        return Err(plan_error(RustChildActionPlanErrorKind::InvalidInput, "Rust action resource or locality drift"));
    }
    if action.event_count_min == 0 || action.event_count_max < action.event_count_min {
        return Err(plan_error(RustChildActionPlanErrorKind::InvalidInput, "Rust action event bounds are invalid"));
    }
    if action.producer_action_ids.iter().any(|producer| !action_ids.contains(producer)) {
        return Err(plan_error(
            RustChildActionPlanErrorKind::IncompleteGraph,
            "Rust action references an unknown producer",
        ));
    }
    if action.permitted_child_fixed_authority_ids.iter().any(|authority| !fixed_ids.contains(authority)) {
        return Err(plan_error(
            RustChildActionPlanErrorKind::InvalidInput,
            "Rust action references unknown child authority",
        ));
    }
    validate_action_executable(&action.executable, action_ids, fixed_ids)
}

fn validate_action_executable(
    executable: &RustChildExecutableAuthority,
    action_ids: &BTreeSet<String>,
    fixed_ids: &BTreeSet<String>,
) -> Result<(), RustChildActionPlanError> {
    match executable {
        RustChildExecutableAuthority::Fixed { authority_id } if fixed_ids.contains(authority_id) => Ok(()),
        RustChildExecutableAuthority::Produced {
            producer_action_id,
            output_identity_blake3,
        } if action_ids.contains(producer_action_id) => validate_digest("produced executable", output_identity_blake3),
        RustChildExecutableAuthority::Fixed { .. } | RustChildExecutableAuthority::Produced { .. } => Err(plan_error(
            RustChildActionPlanErrorKind::IncompleteGraph,
            "Rust action executable authority is not bound",
        )),
    }
}

pub(crate) fn protected_exec_inputs(
    plan: &RustChildActionPlan,
) -> Result<(Vec<String>, Vec<crate::protected_exec::PlannedExecutable>), RustChildActionPlanError> {
    validate_rust_child_action_plan(plan)?;
    let mut producer_action_ids = plan
        .fixed_executables
        .iter()
        .map(|authority| authority.producer_action_id.clone())
        .chain(plan.actions.iter().map(|action| action.action_id.clone()))
        .collect::<Vec<_>>();
    producer_action_ids.sort();
    producer_action_ids.dedup();
    let mut executables = plan
        .fixed_executables
        .iter()
        .map(|authority| crate::protected_exec::PlannedExecutable {
            authorization_id: authority.authority_id.clone(),
            source_stage_id: authority.producer_action_id.clone(),
            path: authority.path.clone().into(),
            digest_hex: authority.digest_blake3.clone(),
        })
        .collect::<Vec<_>>();
    executables.sort_by(|left, right| left.authorization_id.cmp(&right.authorization_id));
    assert!(!producer_action_ids.is_empty());
    assert_eq!(executables.len(), plan.fixed_executables.len());
    Ok((producer_action_ids, executables))
}

pub(crate) fn rust_child_action_audit(
    plan: &RustChildActionPlan,
    raw_events: Vec<crate::protected_exec::ProtectedSeccompAuditEvent>,
    observations: Vec<RustChildExecObservation>,
    promotions: Vec<crate::protected_exec::OutputPromotionRecord>,
    assigned_event_count: usize,
) -> Result<RustChildActionAudit, RustChildActionPlanError> {
    validate_rust_child_action_plan(plan)?;
    let raw_event_count = bounded_count("Rust child raw event", raw_events.len(), RUST_EXEC_EVENT_COUNT_MAX)?;
    let assigned_event_count =
        bounded_count("Rust child assigned event", assigned_event_count, RUST_EXEC_EVENT_COUNT_MAX)?;
    if raw_event_count != assigned_event_count || observations.len() != raw_events.len() {
        return Err(plan_error(
            RustChildActionPlanErrorKind::InvalidInput,
            "Rust child action audit event counts are not closed",
        ));
    }
    let mut audit = RustChildActionAudit {
        schema: RUST_CHILD_ACTION_AUDIT_SCHEMA.to_string(),
        action_plan_digest_blake3: plan.plan_digest_blake3.clone(),
        raw_event_count,
        assigned_event_count,
        raw_events,
        observations,
        promotions,
        audit_digest_blake3: String::new(),
    };
    audit.audit_digest_blake3 = audit_digest(&audit)?;
    validate_rust_child_action_audit(plan, &audit)?;
    assert_eq!(audit.raw_event_count, audit.assigned_event_count);
    assert_eq!(audit.audit_digest_blake3.len(), BLAKE3_HEX_LENGTH);
    Ok(audit)
}

pub(crate) fn validate_rust_child_action_audit(
    plan: &RustChildActionPlan,
    audit: &RustChildActionAudit,
) -> Result<(), RustChildActionPlanError> {
    validate_rust_child_action_plan(plan)?;
    if audit.schema != RUST_CHILD_ACTION_AUDIT_SCHEMA
        || audit.action_plan_digest_blake3 != plan.plan_digest_blake3
        || usize::try_from(audit.raw_event_count).ok() != Some(audit.raw_events.len())
        || audit.raw_event_count != audit.assigned_event_count
        || audit.observations.len() != audit.raw_events.len()
    {
        return Err(plan_error(
            RustChildActionPlanErrorKind::InvalidInput,
            "Rust child action audit identity or count drift",
        ));
    }
    validate_digest("audit", &audit.audit_digest_blake3)?;
    if audit_digest(audit)? != audit.audit_digest_blake3 {
        return Err(plan_error(RustChildActionPlanErrorKind::InvalidInput, "Rust child action audit digest mismatch"));
    }
    Ok(())
}

pub(crate) fn rust_child_action_reconciliation(
    plan: &RustChildActionPlan,
    observations: &[RustChildExecObservation],
) -> Result<RustChildActionReconciliation, RustChildActionPlanError> {
    validate_rust_child_action_plan(plan)?;
    let observed_event_count = bounded_count("Rust child observation", observations.len(), RUST_EXEC_EVENT_COUNT_MAX)?;
    let actions = plan.actions.iter().map(|action| (action.action_id.as_str(), action)).collect::<BTreeMap<_, _>>();
    let fixed = plan
        .fixed_executables
        .iter()
        .map(|authority| (authority.authority_id.as_str(), authority))
        .collect::<BTreeMap<_, _>>();
    let mut state = ReconciliationState::default();
    for observation in observations {
        reconcile_observation(observation, &actions, &fixed, &mut state)?;
    }
    let mut reconciliation = finish_reconciliation(plan, observed_event_count, state)?;
    reconciliation.reconciliation_digest_blake3 = reconciliation_digest(&reconciliation)?;
    validate_rust_child_action_reconciliation(plan, &reconciliation)?;
    Ok(reconciliation)
}

pub(crate) fn validate_rust_child_action_reconciliation(
    plan: &RustChildActionPlan,
    reconciliation: &RustChildActionReconciliation,
) -> Result<(), RustChildActionPlanError> {
    validate_rust_child_action_plan(plan)?;
    if reconciliation.schema != RUST_CHILD_ACTION_RECONCILIATION_SCHEMA
        || reconciliation.action_plan_digest_blake3 != plan.plan_digest_blake3
        || reconciliation.planned_action_count != plan.action_count
    {
        return Err(plan_error(
            RustChildActionPlanErrorKind::InvalidInput,
            "Rust child action reconciliation identity drift",
        ));
    }
    validate_digest("reconciliation", &reconciliation.reconciliation_digest_blake3)?;
    if reconciliation.matched_action_count > reconciliation.planned_action_count
        || reconciliation.matched_event_count > reconciliation.observed_event_count
        || reconciliation_digest(reconciliation)? != reconciliation.reconciliation_digest_blake3
    {
        return Err(plan_error(
            RustChildActionPlanErrorKind::InvalidInput,
            "Rust child action reconciliation count or digest drift",
        ));
    }
    Ok(())
}

#[cfg(test)]
pub(crate) fn reconcile_rust_child_actions(
    plan: &RustChildActionPlan,
    observations: &[RustChildExecObservation],
) -> Result<RustChildActionReconciliation, RustChildActionPlanError> {
    let reconciliation = rust_child_action_reconciliation(plan, observations)?;
    if !reconciliation.is_complete() {
        return Err(plan_error(
            RustChildActionPlanErrorKind::IncompleteReconciliation,
            &format!("Rust child action reconciliation is incomplete: {}", reconciliation.blockers.join("; ")),
        ));
    }
    Ok(reconciliation)
}

#[derive(Default)]
struct ReconciliationState {
    matched_event_count: u32,
    counts_by_action: BTreeMap<String, u32>,
    unknown: Vec<String>,
    denied: Vec<String>,
    drifted: Vec<String>,
}

fn reconcile_observation(
    observation: &RustChildExecObservation,
    actions: &BTreeMap<&str, &RustChildAction>,
    fixed: &BTreeMap<&str, &RustFixedExecutableAuthority>,
    state: &mut ReconciliationState,
) -> Result<(), RustChildActionPlanError> {
    validate_observation(observation)?;
    let event_id = observation_id(observation)?;
    let Some(action) = actions.get(observation.action_id.as_str()) else {
        state.unknown.push(event_id);
        return Ok(());
    };
    if observation.policy_decision != "allowed" {
        state.denied.push(event_id);
        return Ok(());
    }
    if !observation_matches(action, observation, fixed)? {
        state.drifted.push(event_id);
        return Ok(());
    }
    state.matched_event_count = state
        .matched_event_count
        .checked_add(1)
        .ok_or_else(|| plan_error(RustChildActionPlanErrorKind::InvalidInput, "matched Rust event count overflow"))?;
    let count = state.counts_by_action.entry(action.action_id.clone()).or_insert(0);
    *count = count.checked_add(1).ok_or_else(|| {
        plan_error(RustChildActionPlanErrorKind::InvalidInput, "per-action Rust event count overflow")
    })?;
    Ok(())
}

fn validate_observation(observation: &RustChildExecObservation) -> Result<(), RustChildActionPlanError> {
    validate_text("observed action_id", &observation.action_id)?;
    validate_text("observed path", &observation.resolved_path)?;
    validate_digest("observed executable", &observation.digest_blake3)?;
    validate_text("observed policy decision", &observation.policy_decision)?;
    if !Path::new(&observation.resolved_path).is_absolute() {
        return Err(plan_error(
            RustChildActionPlanErrorKind::InvalidInput,
            "observed Rust executable path is relative",
        ));
    }
    Ok(())
}

fn observation_matches(
    action: &RustChildAction,
    observation: &RustChildExecObservation,
    fixed: &BTreeMap<&str, &RustFixedExecutableAuthority>,
) -> Result<bool, RustChildActionPlanError> {
    match &observation.executable {
        RustObservedExecutableAuthority::Fixed { authority_id } => {
            let Some(authority) = fixed.get(authority_id.as_str()) else {
                return Ok(false);
            };
            let permitted = direct_fixed_authority(&action.executable) == Some(authority_id.as_str())
                || action.permitted_child_fixed_authority_ids.iter().any(|candidate| candidate == authority_id);
            Ok(permitted
                && authority.path == observation.resolved_path
                && authority.digest_blake3 == observation.digest_blake3)
        }
        RustObservedExecutableAuthority::Produced {
            producer_action_id,
            output_identity_blake3,
        } => match &action.executable {
            RustChildExecutableAuthority::Produced {
                producer_action_id: expected_producer,
                output_identity_blake3: expected_output,
            } => Ok(producer_action_id == expected_producer && output_identity_blake3 == expected_output),
            RustChildExecutableAuthority::Fixed { .. } => Ok(false),
        },
    }
}

fn direct_fixed_authority(executable: &RustChildExecutableAuthority) -> Option<&str> {
    match executable {
        RustChildExecutableAuthority::Fixed { authority_id } => Some(authority_id),
        RustChildExecutableAuthority::Produced { .. } => None,
    }
}

fn finish_reconciliation(
    plan: &RustChildActionPlan,
    observed_event_count: u32,
    mut state: ReconciliationState,
) -> Result<RustChildActionReconciliation, RustChildActionPlanError> {
    let mut missing = Vec::new();
    let mut overbound = Vec::new();
    let mut matched_action_count = 0_u32;
    for action in &plan.actions {
        let count = state.counts_by_action.get(&action.action_id).copied().unwrap_or(0);
        if count < action.event_count_min {
            missing.push(action.action_id_blake3.clone());
        } else if count > action.event_count_max {
            overbound.push(action.action_id_blake3.clone());
        } else {
            matched_action_count = matched_action_count.checked_add(1).ok_or_else(|| {
                plan_error(RustChildActionPlanErrorKind::InvalidInput, "matched Rust action count overflow")
            })?;
        }
    }
    normalize_digest_list(&mut state.unknown);
    normalize_digest_list(&mut state.denied);
    normalize_digest_list(&mut state.drifted);
    normalize_digest_list(&mut missing);
    normalize_digest_list(&mut overbound);
    let blockers = reconciliation_blockers(&state, &missing, &overbound);
    Ok(RustChildActionReconciliation {
        schema: RUST_CHILD_ACTION_RECONCILIATION_SCHEMA.to_string(),
        action_plan_digest_blake3: plan.plan_digest_blake3.clone(),
        planned_action_count: plan.action_count,
        matched_action_count,
        observed_event_count,
        matched_event_count: state.matched_event_count,
        unknown_event_ids_blake3: state.unknown,
        denied_event_ids_blake3: state.denied,
        drifted_event_ids_blake3: state.drifted,
        missing_action_ids_blake3: missing,
        overbound_action_ids_blake3: overbound,
        local_only: true,
        blockers,
        reconciliation_digest_blake3: String::new(),
    })
}

fn reconciliation_blockers(state: &ReconciliationState, missing: &[String], overbound: &[String]) -> Vec<String> {
    let mut blockers = Vec::new();
    if !state.unknown.is_empty() {
        blockers.push(format!("unknown Rust child events: {}", state.unknown.len()));
    }
    if !state.denied.is_empty() {
        blockers.push(format!("denied Rust child events: {}", state.denied.len()));
    }
    if !state.drifted.is_empty() {
        blockers.push(format!("drifted Rust child events: {}", state.drifted.len()));
    }
    if !missing.is_empty() {
        blockers.push(format!("missing Rust actions: {}", missing.len()));
    }
    if !overbound.is_empty() {
        blockers.push(format!("overbound Rust actions: {}", overbound.len()));
    }
    blockers
}

fn observation_id(observation: &RustChildExecObservation) -> Result<String, RustChildActionPlanError> {
    let bytes = serde_json::to_vec(&(
        &observation.action_id,
        &observation.resolved_path,
        &observation.digest_blake3,
        &observation.policy_decision,
    ))
    .map_err(|error| plan_error(RustChildActionPlanErrorKind::Serialization, &error.to_string()))?;
    Ok(digest_bytes("mantle-source-built-rust-observed-child-v1", &bytes))
}

fn plan_digest(plan: &RustChildActionPlan) -> Result<String, RustChildActionPlanError> {
    let mut canonical = plan.clone();
    canonical.fixed_executables.sort_by(|left, right| left.authority_id.cmp(&right.authority_id));
    canonical.actions.sort_by(|left, right| left.action_id.cmp(&right.action_id));
    canonical.plan_digest_blake3.clear();
    let bytes = serde_json::to_vec(&canonical)
        .map_err(|error| plan_error(RustChildActionPlanErrorKind::Serialization, &error.to_string()))?;
    Ok(digest_bytes(PLAN_DIGEST_CONTEXT, &bytes))
}

fn authority_digest(authority: &RustChildActionAuthority) -> Result<String, RustChildActionPlanError> {
    let mut canonical = authority.clone();
    canonical.fixed_executables.sort_by(|left, right| left.authority_id.cmp(&right.authority_id));
    canonical.authority_digest_blake3.clear();
    let bytes = serde_json::to_vec(&canonical)
        .map_err(|error| plan_error(RustChildActionPlanErrorKind::Serialization, &error.to_string()))?;
    Ok(digest_bytes(AUTHORITY_DIGEST_CONTEXT, &bytes))
}

fn audit_digest(audit: &RustChildActionAudit) -> Result<String, RustChildActionPlanError> {
    let mut canonical = audit.clone();
    canonical.audit_digest_blake3.clear();
    let bytes = serde_json::to_vec(&canonical)
        .map_err(|error| plan_error(RustChildActionPlanErrorKind::Serialization, &error.to_string()))?;
    Ok(digest_bytes(AUDIT_DIGEST_CONTEXT, &bytes))
}

fn reconciliation_digest(reconciliation: &RustChildActionReconciliation) -> Result<String, RustChildActionPlanError> {
    let mut canonical = reconciliation.clone();
    canonical.reconciliation_digest_blake3.clear();
    let bytes = serde_json::to_vec(&canonical)
        .map_err(|error| plan_error(RustChildActionPlanErrorKind::Serialization, &error.to_string()))?;
    Ok(digest_bytes(RECONCILIATION_DIGEST_CONTEXT, &bytes))
}

fn normalize_digest_list(values: &mut Vec<String>) {
    values.sort();
    values.dedup();
}

fn validate_text(label: &str, value: &str) -> Result<(), RustChildActionPlanError> {
    if value.trim().is_empty() || value.len() > TEXT_BYTES_MAX || value.contains('\0') {
        return Err(plan_error(
            RustChildActionPlanErrorKind::InvalidInput,
            &format!("invalid Rust child action {label}"),
        ));
    }
    Ok(())
}

fn validate_digest(label: &str, digest: &str) -> Result<(), RustChildActionPlanError> {
    if digest.len() != BLAKE3_HEX_LENGTH
        || !digest.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(plan_error(
            RustChildActionPlanErrorKind::InvalidInput,
            &format!("invalid Rust child action {label} BLAKE3"),
        ));
    }
    Ok(())
}

fn bounded_count(label: &str, count: usize, maximum: u32) -> Result<u32, RustChildActionPlanError> {
    let count = u32::try_from(count)
        .map_err(|_| plan_error(RustChildActionPlanErrorKind::InvalidInput, &format!("{label} count exceeds u32")))?;
    if count > maximum {
        return Err(plan_error(
            RustChildActionPlanErrorKind::InvalidInput,
            &format!("{label} count {count} exceeds {maximum}"),
        ));
    }
    Ok(count)
}

fn digest_text(context: &str, text: &str) -> String {
    digest_bytes(context, text.as_bytes())
}

fn digest_bytes(context: &str, bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(context.as_bytes());
    hasher.update(&[0]);
    hasher.update(bytes);
    hasher.finalize().to_hex().to_string()
}

fn plan_error(kind: RustChildActionPlanErrorKind, message: &str) -> RustChildActionPlanError {
    RustChildActionPlanError {
        kind,
        message: message.to_string(),
    }
}

#[cfg(test)]
pub(crate) fn rust_unit_action_inputs_from_graph(
    graph: &crate::rust_plan::UnitDerivationGraphSummary,
) -> Result<Vec<RustUnitActionInput>, RustChildActionPlanError> {
    let unit_ids = graph.derivations.iter().map(|unit| unit.unit_id.clone()).collect::<BTreeSet<_>>();
    if unit_ids.len() != graph.derivations.len() {
        return Err(plan_error(
            RustChildActionPlanErrorKind::IncompleteGraph,
            "Rust unit derivation graph contains duplicate unit IDs",
        ));
    }
    rust_unit_action_inputs_from_graph_selection(graph, &unit_ids)
}

pub(crate) fn rust_unit_action_inputs_from_graph_selection(
    graph: &crate::rust_plan::UnitDerivationGraphSummary,
    selected_unit_ids: &BTreeSet<String>,
) -> Result<Vec<RustUnitActionInput>, RustChildActionPlanError> {
    if !graph.ready || !graph.blockers.is_empty() {
        return Err(plan_error(
            RustChildActionPlanErrorKind::IncompleteGraph,
            "Rust unit derivation graph is not ready for action planning",
        ));
    }
    if selected_unit_ids.is_empty() {
        return Err(plan_error(
            RustChildActionPlanErrorKind::InvalidInput,
            "Rust child action execution scope is empty",
        ));
    }
    let derivations = graph
        .derivations
        .iter()
        .filter(|unit| selected_unit_ids.contains(&unit.unit_id))
        .cloned()
        .collect::<Vec<_>>();
    if derivations.len() != selected_unit_ids.len() {
        return Err(plan_error(
            RustChildActionPlanErrorKind::IncompleteGraph,
            "Rust child action execution scope contains an unknown or duplicate unit",
        ));
    }
    bounded_count("Rust graph unit", derivations.len(), RUST_UNIT_COUNT_MAX)?;
    let producer_index = rust_dependency_producer_index(&derivations)?;
    let mut units = Vec::with_capacity(derivations.len());
    for unit in &derivations {
        units.push(rust_unit_action_input(unit, &producer_index)?);
    }
    assert_eq!(units.len(), selected_unit_ids.len());
    debug_assert!(!units.is_empty());
    Ok(units)
}

fn rust_dependency_producer_index(
    derivations: &[crate::rust_plan::RustUnitDerivationSummary],
) -> Result<RustDependencyProducerIndex, RustChildActionPlanError> {
    let mut index = RustDependencyProducerIndex::default();
    for producer in derivations {
        if producer.execution_kind != "target" {
            continue;
        }
        if producer.target_kind != "lib" {
            continue;
        }
        validate_text("graph producer unit", &producer.unit_id)?;
        validate_text("graph producer package", &producer.package_id)?;
        validate_text("graph producer target", &producer.target_name)?;
        validate_text("graph producer triple", &producer.selected_triple)?;
        let package_key = (producer.package_id.clone(), producer.selected_triple.clone());
        index.target_lib_by_package.entry(package_key).or_default().insert(producer.unit_id.clone());
        let exact_key = (
            producer.package_id.clone(),
            producer.selected_triple.clone(),
            normalized_dependency_name(&producer.target_name),
        );
        index.target_lib_by_exact_name.entry(exact_key).or_default().insert(producer.unit_id.clone());
    }
    assert!(index.target_lib_by_exact_name.len() <= derivations.len());
    assert!(index.target_lib_by_package.len() <= derivations.len());
    Ok(index)
}

fn normalized_dependency_name(name: &str) -> String {
    name.replace('-', "_")
}

fn rust_unit_action_input(
    unit: &crate::rust_plan::RustUnitDerivationSummary,
    producer_index: &RustDependencyProducerIndex,
) -> Result<RustUnitActionInput, RustChildActionPlanError> {
    if unit.derivation.builder != "rustc" {
        return Err(plan_error(
            RustChildActionPlanErrorKind::IncompleteGraph,
            &format!("Rust unit {} uses unsupported builder {}", unit.unit_id, unit.derivation.builder),
        ));
    }
    let source_digest_blake3 = canonical_source_identity_blake3(&unit.source_digest)?;
    let producer_unit_ids = rust_unit_producer_ids(unit, producer_index)?;
    let input_authority_ids = rust_unit_input_authority_ids(unit, &source_digest_blake3, producer_index)?;
    let environment_digest_blake3 = canonical_environment_digest(&unit.derivation.env)?;
    Ok(RustUnitActionInput {
        unit_id: unit.unit_id.clone(),
        target_kind: unit.target_kind.clone(),
        producer_unit_ids,
        input_authority_ids,
        declared_outputs: unit.derivation.outputs.clone(),
        source_digest_blake3,
        rustc_args_digest_blake3: unit.rustc_args_digest_blake3.clone(),
        environment_digest_blake3,
    })
}

fn canonical_source_identity_blake3(
    source_digest: &crate::rust_plan::SourceDigest,
) -> Result<String, RustChildActionPlanError> {
    validate_text("source digest algorithm", &source_digest.algorithm)?;
    validate_text("source digest value", &source_digest.value)?;
    let framed = format!("{}\0{}", source_digest.algorithm, source_digest.value);
    let digest = digest_text(SOURCE_IDENTITY_CONTEXT, &framed);
    assert_eq!(digest.len(), BLAKE3_HEX_LENGTH);
    Ok(digest)
}

fn rust_unit_producer_ids(
    unit: &crate::rust_plan::RustUnitDerivationSummary,
    producer_index: &RustDependencyProducerIndex,
) -> Result<Vec<String>, RustChildActionPlanError> {
    let mut producers = Vec::new();
    for artifact in &unit.dependency_artifacts {
        producers.push(rust_dependency_artifact_producer(unit, artifact, producer_index)?);
    }
    for (label, producer) in rust_unit_optional_non_dependency_producers(unit) {
        let producer = producer.ok_or_else(|| {
            plan_error(
                RustChildActionPlanErrorKind::IncompleteGraph,
                &format!("Rust unit {} has unbound {label} producer", unit.unit_id),
            )
        })?;
        producers.push(producer.to_string());
    }
    producers.sort();
    producers.dedup();
    Ok(producers)
}

fn rust_unit_optional_non_dependency_producers(
    unit: &crate::rust_plan::RustUnitDerivationSummary,
) -> Vec<(&'static str, Option<&str>)> {
    let mut producers = Vec::new();
    producers.extend(
        unit.consumed_host_artifacts
            .iter()
            .map(|artifact| ("host artifact", artifact.producer_unit_id.as_deref())),
    );
    producers.extend(
        unit.metadata_dependencies
            .iter()
            .map(|dependency| ("build metadata", dependency.producer_unit_id.as_deref())),
    );
    producers
}

fn rust_dependency_artifact_producer(
    unit: &crate::rust_plan::RustUnitDerivationSummary,
    artifact: &crate::rust_plan::RustDependencyArtifact,
    producer_index: &RustDependencyProducerIndex,
) -> Result<String, RustChildActionPlanError> {
    if let Some(producer) = &artifact.producer_unit_id {
        validate_text("dependency producer", producer)?;
        return Ok(producer.clone());
    }
    let host_matches = consumed_host_dependency_producers(unit, artifact);
    if let Some(producer) = unique_dependency_producer(unit, artifact, "host", &host_matches)? {
        return Ok(producer);
    }
    if unit.execution_kind != "target" {
        return Err(unbound_dependency_producer_error(unit, artifact));
    }
    let graph_matches = indexed_target_dependency_producers(unit, artifact, producer_index);
    if let Some(producer) = unique_dependency_producer(unit, artifact, "graph", &graph_matches)? {
        return Ok(producer);
    }
    Err(unbound_dependency_producer_error(unit, artifact))
}

fn consumed_host_dependency_producers(
    unit: &crate::rust_plan::RustUnitDerivationSummary,
    artifact: &crate::rust_plan::RustDependencyArtifact,
) -> BTreeSet<String> {
    let package_matches = unit
        .consumed_host_artifacts
        .iter()
        .filter(|candidate| candidate.package_id == artifact.package_id)
        .filter_map(|candidate| candidate.producer_unit_id.clone())
        .collect::<BTreeSet<_>>();
    let artifact_name = normalized_dependency_name(&artifact.name);
    let exact_matches = unit
        .consumed_host_artifacts
        .iter()
        .filter(|candidate| candidate.package_id == artifact.package_id)
        .filter(|candidate| normalized_dependency_name(&candidate.target_name) == artifact_name)
        .filter_map(|candidate| candidate.producer_unit_id.clone())
        .collect::<BTreeSet<_>>();
    if exact_matches.is_empty() {
        package_matches
    } else {
        exact_matches
    }
}

fn indexed_target_dependency_producers(
    unit: &crate::rust_plan::RustUnitDerivationSummary,
    artifact: &crate::rust_plan::RustDependencyArtifact,
    producer_index: &RustDependencyProducerIndex,
) -> BTreeSet<String> {
    let exact_key = (
        artifact.package_id.clone(),
        unit.selected_triple.clone(),
        normalized_dependency_name(&artifact.name),
    );
    if let Some(exact) = producer_index.target_lib_by_exact_name.get(&exact_key)
        && !exact.is_empty()
    {
        return exact.clone();
    }
    let package_key = (artifact.package_id.clone(), unit.selected_triple.clone());
    producer_index.target_lib_by_package.get(&package_key).cloned().unwrap_or_default()
}

fn unique_dependency_producer(
    unit: &crate::rust_plan::RustUnitDerivationSummary,
    artifact: &crate::rust_plan::RustDependencyArtifact,
    source: &str,
    matches: &BTreeSet<String>,
) -> Result<Option<String>, RustChildActionPlanError> {
    if matches.is_empty() {
        return Ok(None);
    }
    if matches.len() > 1 {
        return Err(ambiguous_dependency_producer_error(unit, artifact, source, matches.len()));
    }
    let producer = matches.iter().next().ok_or_else(|| unbound_dependency_producer_error(unit, artifact))?;
    validate_text("dependency producer", producer)?;
    Ok(Some(producer.clone()))
}

fn unbound_dependency_producer_error(
    unit: &crate::rust_plan::RustUnitDerivationSummary,
    artifact: &crate::rust_plan::RustDependencyArtifact,
) -> RustChildActionPlanError {
    plan_error(
        RustChildActionPlanErrorKind::IncompleteGraph,
        &format!(
            "Rust unit {} has unbound dependency artifact producer for {} from {}",
            unit.unit_id, artifact.name, artifact.package_id
        ),
    )
}

fn ambiguous_dependency_producer_error(
    unit: &crate::rust_plan::RustUnitDerivationSummary,
    artifact: &crate::rust_plan::RustDependencyArtifact,
    source: &str,
    count: usize,
) -> RustChildActionPlanError {
    plan_error(
        RustChildActionPlanErrorKind::IncompleteGraph,
        &format!(
            "Rust unit {} has {count} {source} producers for dependency artifact {} from {}",
            unit.unit_id, artifact.name, artifact.package_id
        ),
    )
}

fn rust_unit_input_authority_ids(
    unit: &crate::rust_plan::RustUnitDerivationSummary,
    source_digest_blake3: &str,
    producer_index: &RustDependencyProducerIndex,
) -> Result<Vec<String>, RustChildActionPlanError> {
    let mut authorities = vec![
        format!("package:{}", unit.package_id),
        format!("source:{source_digest_blake3}"),
        format!("target:{}:{}", unit.selected_triple, unit.target_name),
    ];
    for artifact in &unit.dependency_artifacts {
        let producer = rust_dependency_artifact_producer(unit, artifact, producer_index)?;
        authorities.push(format!("dependency:{producer}:{}:{}", artifact.package_id, artifact.name));
    }
    for artifact in &unit.consumed_host_artifacts {
        let producer = artifact.producer_unit_id.as_deref().ok_or_else(|| {
            plan_error(
                RustChildActionPlanErrorKind::IncompleteGraph,
                &format!("Rust unit {} has unbound host artifact producer", unit.unit_id),
            )
        })?;
        authorities.push(format!("host:{producer}:{}:{}", artifact.package_id, artifact.target_name));
    }
    authorities.sort();
    authorities.dedup();
    Ok(authorities)
}

fn canonical_environment_digest(environment: &BTreeMap<String, String>) -> Result<String, RustChildActionPlanError> {
    let bytes = serde_json::to_vec(environment)
        .map_err(|error| plan_error(RustChildActionPlanErrorKind::Serialization, &error.to_string()))?;
    Ok(digest_bytes("mantle-source-built-rust-environment-v1", &bytes))
}

pub(crate) fn fixed_executable_authorities_from_toolchain_closure(
    manifest: &crate::source_toolchain_closure::ToolchainClosureManifest,
) -> Result<Vec<RustFixedExecutableAuthority>, RustChildActionPlanError> {
    if !manifest.seed_exceptions.is_empty() {
        return Err(plan_error(
            RustChildActionPlanErrorKind::InvalidInput,
            "Rust action authority rejects toolchain seed exceptions",
        ));
    }
    let mut authorities_by_path = BTreeMap::<String, RustFixedExecutableAuthority>::new();
    for member in &manifest.members {
        let Some(kind) = fixed_executable_kind(member.role) else {
            continue;
        };
        let authority = fixed_executable_authority(member, kind)?;
        if let Some(existing) = authorities_by_path.get(&authority.path) {
            if existing.digest_blake3 != authority.digest_blake3 {
                return Err(plan_error(
                    RustChildActionPlanErrorKind::InvalidInput,
                    &format!("toolchain executable path has conflicting bytes: {}", authority.path),
                ));
            }
            if fixed_kind_priority(&existing.kind) >= fixed_kind_priority(&authority.kind) {
                continue;
            }
        }
        authorities_by_path.insert(authority.path.clone(), authority);
    }
    let mut authorities = authorities_by_path.into_values().collect::<Vec<_>>();
    authorities.sort_by(|left, right| left.authority_id.cmp(&right.authority_id));
    if authorities.is_empty() {
        return Err(plan_error(
            RustChildActionPlanErrorKind::InvalidInput,
            "toolchain closure has no executable authorities",
        ));
    }
    Ok(authorities)
}

fn fixed_kind_priority(kind: &RustFixedExecutableKind) -> u8 {
    match kind {
        RustFixedExecutableKind::Rustc => 7,
        RustFixedExecutableKind::CompilerPolicyAdapter => 6,
        RustFixedExecutableKind::CCompiler | RustFixedExecutableKind::CxxCompiler => 5,
        RustFixedExecutableKind::Linker => 4,
        RustFixedExecutableKind::PkgConfig => 3,
        RustFixedExecutableKind::Shell => 2,
        RustFixedExecutableKind::NativeHelper => 1,
    }
}

fn fixed_executable_kind(role: crate::source_toolchain_closure::ToolchainRole) -> Option<RustFixedExecutableKind> {
    use crate::source_toolchain_closure::ToolchainRole;
    match role {
        ToolchainRole::Rustc => Some(RustFixedExecutableKind::Rustc),
        ToolchainRole::Linker => Some(RustFixedExecutableKind::Linker),
        ToolchainRole::CCompiler => Some(RustFixedExecutableKind::CCompiler),
        ToolchainRole::CxxCompiler => Some(RustFixedExecutableKind::CxxCompiler),
        ToolchainRole::PkgConfig => Some(RustFixedExecutableKind::PkgConfig),
        ToolchainRole::NativeHelper => Some(RustFixedExecutableKind::NativeHelper),
        ToolchainRole::Sysroot | ToolchainRole::CrtObject | ToolchainRole::RuntimeLibrary => None,
    }
}

fn fixed_executable_authority(
    member: &crate::source_toolchain_closure::ToolchainClosureMember,
    kind: RustFixedExecutableKind,
) -> Result<RustFixedExecutableAuthority, RustChildActionPlanError> {
    use crate::source_toolchain_closure::ToolchainTrust;
    if member.trust != ToolchainTrust::SourceBuilt {
        return Err(plan_error(
            RustChildActionPlanErrorKind::InvalidInput,
            &format!("toolchain executable {} is not source-built", member.name),
        ));
    }
    let source = member.source.as_ref().ok_or_else(|| {
        plan_error(
            RustChildActionPlanErrorKind::IncompleteGraph,
            &format!("toolchain executable {} lacks source authority", member.name),
        )
    })?;
    let receipt = member.build_receipt.as_ref().ok_or_else(|| {
        plan_error(
            RustChildActionPlanErrorKind::IncompleteGraph,
            &format!("toolchain executable {} lacks build receipt authority", member.name),
        )
    })?;
    validate_digest("toolchain executable", &member.content_digest_blake3)?;
    validate_digest("toolchain source", &source.digest_blake3)?;
    validate_digest("toolchain build receipt", &receipt.digest_blake3)?;
    let authority_material = serde_json::to_vec(&(
        member.role,
        &member.name,
        &member.execution_path,
        &member.content_digest_blake3,
        source,
        receipt,
    ))
    .map_err(|error| plan_error(RustChildActionPlanErrorKind::Serialization, &error.to_string()))?;
    let authority_id =
        format!("toolchain:{}", digest_bytes("mantle-source-built-rust-fixed-executable-v1", &authority_material));
    Ok(RustFixedExecutableAuthority {
        authority_id,
        producer_action_id: format!("toolchain-build:{}:{}", receipt.name, receipt.digest_blake3),
        output_identity_blake3: member.content_digest_blake3.clone(),
        path: member.execution_path.clone(),
        digest_blake3: member.content_digest_blake3.clone(),
        kind,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIGEST_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const DIGEST_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
    const DIGEST_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    const DIGEST_D: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
    const TEST_EXEC_EVENTS_PER_ACTION_MAX: u32 = 8;
    const TEST_PARALLEL_JOBS_MAX: u32 = 2;
    const TEST_OPEN_FILE_DESCRIPTORS_MAX: u32 = 64;
    const TEST_STORAGE_BYTES_MAX: u64 = 1_048_576;

    fn resources() -> RustActionResourceLimits {
        RustActionResourceLimits {
            parallel_jobs_max: TEST_PARALLEL_JOBS_MAX,
            open_file_descriptors_max: TEST_OPEN_FILE_DESCRIPTORS_MAX,
            storage_bytes_max: TEST_STORAGE_BYTES_MAX,
            exec_events_per_action_max: TEST_EXEC_EVENTS_PER_ACTION_MAX,
        }
    }

    fn fixed(
        authority_id: &str,
        path: &str,
        digest: &str,
        kind: RustFixedExecutableKind,
    ) -> RustFixedExecutableAuthority {
        RustFixedExecutableAuthority {
            authority_id: authority_id.to_string(),
            producer_action_id: format!("provider:{authority_id}"),
            output_identity_blake3: digest.to_string(),
            path: path.to_string(),
            digest_blake3: digest.to_string(),
            kind,
        }
    }

    fn unit(unit_id: &str, target_kind: &str, producers: &[&str], digest: &str) -> RustUnitActionInput {
        RustUnitActionInput {
            unit_id: unit_id.to_string(),
            target_kind: target_kind.to_string(),
            producer_unit_ids: producers.iter().map(|producer| (*producer).to_string()).collect(),
            input_authority_ids: vec![format!("package:{unit_id}")],
            declared_outputs: vec![format!("out/{unit_id}")],
            source_digest_blake3: digest.to_string(),
            rustc_args_digest_blake3: DIGEST_C.to_string(),
            environment_digest_blake3: DIGEST_D.to_string(),
        }
    }

    fn input() -> RustChildActionPlanInput {
        RustChildActionPlanInput {
            stage_id: "mantle-stage1".to_string(),
            resources: resources(),
            fixed_executables: vec![
                fixed("rustc", "/provider/bin/rustc", DIGEST_A, RustFixedExecutableKind::Rustc),
                fixed("linker", "/provider/bin/ld", DIGEST_B, RustFixedExecutableKind::Linker),
            ],
            units: vec![
                unit("dep", "lib", &[], DIGEST_A),
                unit("build", "custom-build", &["dep"], DIGEST_B),
            ],
        }
    }

    fn fixed_observation(
        action: &RustChildAction,
        authority_id: &str,
        path: &str,
        digest: &str,
    ) -> RustChildExecObservation {
        RustChildExecObservation {
            action_id: action.action_id.clone(),
            executable: RustObservedExecutableAuthority::Fixed {
                authority_id: authority_id.to_string(),
            },
            resolved_path: path.to_string(),
            digest_blake3: digest.to_string(),
            policy_decision: "allowed".to_string(),
        }
    }

    fn raw_event(path: &str, digest: &str) -> crate::protected_exec::ProtectedSeccompAuditEvent {
        crate::protected_exec::ProtectedSeccompAuditEvent {
            pid: 1,
            syscall: "execve".to_string(),
            executable_path: path.into(),
            tracee_path: path.into(),
            resolved_host_path: path.into(),
            digest_hex: digest.to_string(),
            reason: "test action allowed".to_string(),
            phase: "protected".to_string(),
            inventory_entry_id: Some("test-entry".to_string()),
            policy_decision: "allowed".to_string(),
        }
    }

    fn produced_observation(action: &RustChildAction) -> RustChildExecObservation {
        let RustChildExecutableAuthority::Produced {
            producer_action_id,
            output_identity_blake3,
        } = &action.executable
        else {
            panic!("build-script action must use produced authority");
        };
        RustChildExecObservation {
            action_id: action.action_id.clone(),
            executable: RustObservedExecutableAuthority::Produced {
                producer_action_id: producer_action_id.clone(),
                output_identity_blake3: output_identity_blake3.clone(),
            },
            resolved_path: "/execution/build-script".to_string(),
            digest_blake3: DIGEST_C.to_string(),
            policy_decision: "allowed".to_string(),
        }
    }

    fn graph_unit(
        unit_id: &str,
        target_kind: &str,
        dependency: Option<&str>,
    ) -> crate::rust_plan::RustUnitDerivationSummary {
        let dependency_artifacts = dependency
            .map(|producer| {
                vec![crate::rust_plan::RustDependencyArtifact {
                    package_id: producer.to_string(),
                    name: producer.to_string(),
                    producer_unit_id: Some(producer.to_string()),
                    artifact: format!("/execution/{producer}.rlib"),
                }]
            })
            .unwrap_or_default();
        crate::rust_plan::RustUnitDerivationSummary {
            unit_id: unit_id.to_string(),
            package_id: format!("package:{unit_id}"),
            target_name: unit_id.to_string(),
            target_kind: target_kind.to_string(),
            execution_kind: "host".to_string(),
            selected_triple: "x86_64-unknown-linux-musl".to_string(),
            rustc_metadata_hash: DIGEST_A.to_string(),
            crate_types: vec!["bin".to_string()],
            mode: "build".to_string(),
            profile: "release".to_string(),
            source_digest: crate::rust_plan::SourceDigest {
                algorithm: "blake3".to_string(),
                value: DIGEST_B.to_string(),
            },
            dependency_artifacts,
            consumed_host_artifacts: Vec::new(),
            metadata_dependencies: Vec::new(),
            generated_metadata: None,
            derivation: crate::rust_plan::ReviewableRustDerivation {
                name: unit_id.to_string(),
                builder: "rustc".to_string(),
                system: "x86_64-linux".to_string(),
                args: vec![format!("src/{unit_id}.rs")],
                outputs: vec![format!("out/{unit_id}")],
                env: BTreeMap::from([("PROFILE".to_string(), "release".to_string())]),
                inputs: Vec::new(),
                addressing_mode: "input-addressed".to_string(),
            },
            rustc_args_digest_blake3: DIGEST_C.to_string(),
        }
    }

    fn graph() -> crate::rust_plan::UnitDerivationGraphSummary {
        crate::rust_plan::UnitDerivationGraphSummary {
            derivation_count: 2,
            host_unit_count: 2,
            host_artifact_count: 1,
            ready: true,
            digest_blake3: DIGEST_D.to_string(),
            derivations: vec![
                graph_unit("dep", "lib", None),
                graph_unit("build", "custom-build", Some("dep")),
            ],
            blockers: Vec::new(),
        }
    }

    fn closure_member(
        name: &str,
        path: &str,
        role: crate::source_toolchain_closure::ToolchainRole,
        digest: &str,
    ) -> crate::source_toolchain_closure::ToolchainClosureMember {
        crate::source_toolchain_closure::ToolchainClosureMember {
            role,
            name: name.to_string(),
            execution_path: path.to_string(),
            content_digest_blake3: digest.to_string(),
            trust: crate::source_toolchain_closure::ToolchainTrust::SourceBuilt,
            source: Some(crate::source_toolchain_closure::ToolchainSourceIdentity {
                kind: crate::source_toolchain_closure::ToolchainSourceKind::Generated,
                name: format!("source:{name}"),
                digest_blake3: DIGEST_C.to_string(),
            }),
            build_receipt: Some(crate::source_toolchain_closure::ToolchainBuildReceiptIdentity {
                kind: crate::source_toolchain_closure::ToolchainBuildReceiptKind::MantleDerivation,
                name: format!("receipt:{name}"),
                digest_blake3: DIGEST_D.to_string(),
            }),
        }
    }

    fn closure() -> crate::source_toolchain_closure::ToolchainClosureManifest {
        crate::source_toolchain_closure::ToolchainClosureManifest {
            schema: crate::source_toolchain_closure::SOURCE_BUILT_TOOLCHAIN_CLOSURE_SCHEMA.to_string(),
            members: vec![
                closure_member(
                    "rustc",
                    "/provider/bin/rustc",
                    crate::source_toolchain_closure::ToolchainRole::Rustc,
                    DIGEST_A,
                ),
                closure_member(
                    "ld",
                    "/provider/bin/ld",
                    crate::source_toolchain_closure::ToolchainRole::Linker,
                    DIGEST_B,
                ),
            ],
            seed_exceptions: Vec::new(),
        }
    }

    #[test]
    fn rust_graph_action_selection_excludes_unexecuted_units_and_rejects_unknown_scope() {
        const EXPECTED_SELECTED_UNIT_COUNT: usize = 2;
        let mut graph = graph();
        graph.derivations.push(graph_unit("unexecuted", "example", None));
        graph.derivation_count = graph.derivations.len();
        let selected = BTreeSet::from(["dep".to_string(), "build".to_string()]);
        let units = rust_unit_action_inputs_from_graph_selection(&graph, &selected).unwrap();
        let mut unknown = selected.clone();
        unknown.insert("missing".to_string());
        let unknown_error = rust_unit_action_inputs_from_graph_selection(&graph, &unknown).unwrap_err();
        let empty_error = rust_unit_action_inputs_from_graph_selection(&graph, &BTreeSet::new()).unwrap_err();

        assert_eq!(units.len(), EXPECTED_SELECTED_UNIT_COUNT);
        assert!(units.iter().any(|unit| unit.unit_id == "dep"));
        assert!(units.iter().any(|unit| unit.unit_id == "build"));
        assert!(!units.iter().any(|unit| unit.unit_id == "unexecuted"));
        assert!(unknown_error.message.contains("unknown or duplicate unit"));
        assert!(empty_error.message.contains("execution scope is empty"));
    }

    #[test]
    fn rust_graph_source_identities_are_blake3_framed_across_input_algorithms() {
        let mut path_graph = graph();
        for unit in &mut path_graph.derivations {
            unit.source_digest.algorithm = "blake3-tree-v1".to_string();
        }
        let path_units = rust_unit_action_inputs_from_graph(&path_graph).unwrap();
        let mut registry_graph = path_graph.clone();
        for unit in &mut registry_graph.derivations {
            unit.source_digest.algorithm = "cargo-checksum-sha256".to_string();
        }
        let registry_units = rust_unit_action_inputs_from_graph(&registry_graph).unwrap();
        let mut invalid_graph = path_graph;
        invalid_graph.derivations[0].source_digest.algorithm.clear();
        let invalid_error = rust_unit_action_inputs_from_graph(&invalid_graph).unwrap_err();

        assert_eq!(path_units[0].source_digest_blake3.len(), BLAKE3_HEX_LENGTH);
        assert_eq!(registry_units[0].source_digest_blake3.len(), BLAKE3_HEX_LENGTH);
        assert_ne!(path_units[0].source_digest_blake3, DIGEST_B);
        assert_ne!(path_units[0].source_digest_blake3, registry_units[0].source_digest_blake3);
        assert_eq!(invalid_error.kind, RustChildActionPlanErrorKind::InvalidInput);
        assert!(invalid_error.to_string().contains("source digest algorithm"));
    }

    #[test]
    fn rust_graph_dependency_uses_unique_consumed_host_producer_fallback() {
        let mut fallback_graph = graph();
        let build = &mut fallback_graph.derivations[1];
        build.dependency_artifacts[0].producer_unit_id = None;
        build.consumed_host_artifacts.push(crate::rust_plan::RustHostArtifact {
            package_id: "dep".to_string(),
            target_name: "dep".to_string(),
            target_kind: "proc-macro".to_string(),
            producer_unit_id: Some("dep".to_string()),
            artifact: "host-artifact:dep".to_string(),
            metadata_digest_blake3: None,
        });

        let units = rust_unit_action_inputs_from_graph(&fallback_graph).unwrap();
        let build_input = units.iter().find(|unit| unit.unit_id == "build").expect("build unit action input");
        let mut ambiguous_graph = fallback_graph;
        ambiguous_graph.derivations[1].consumed_host_artifacts.push(crate::rust_plan::RustHostArtifact {
            package_id: "dep".to_string(),
            target_name: "dep".to_string(),
            target_kind: "proc-macro".to_string(),
            producer_unit_id: Some("other-dep".to_string()),
            artifact: "host-artifact:other-dep".to_string(),
            metadata_digest_blake3: None,
        });
        let ambiguous_error = rust_unit_action_inputs_from_graph(&ambiguous_graph).unwrap_err();

        assert!(build_input.producer_unit_ids.contains(&"dep".to_string()));
        assert!(build_input.input_authority_ids.contains(&"dependency:dep:dep:dep".to_string()));
        assert_eq!(ambiguous_error.kind, RustChildActionPlanErrorKind::IncompleteGraph);
        assert!(ambiguous_error.message.contains("2 host producers"));
    }

    #[test]
    fn rust_graph_dependency_uses_unique_target_lib_producer_fallback() {
        let mut target_graph = graph();
        for unit in &mut target_graph.derivations {
            unit.execution_kind = "target".to_string();
        }
        target_graph.derivations[1].dependency_artifacts[0].producer_unit_id = None;
        target_graph.derivations[1].dependency_artifacts[0].package_id = "package:dep".to_string();

        let units = rust_unit_action_inputs_from_graph(&target_graph).unwrap();
        let build_input = units.iter().find(|unit| unit.unit_id == "build").expect("build unit action input");
        let mut ambiguous_graph = target_graph;
        let mut duplicate = ambiguous_graph.derivations[0].clone();
        duplicate.unit_id = "dep-other".to_string();
        ambiguous_graph.derivations.push(duplicate);
        ambiguous_graph.derivation_count = ambiguous_graph.derivations.len();
        let ambiguous_error = rust_unit_action_inputs_from_graph(&ambiguous_graph).unwrap_err();

        assert!(build_input.producer_unit_ids.contains(&"dep".to_string()));
        assert!(build_input.input_authority_ids.contains(&"dependency:dep:package:dep:dep".to_string()));
        assert_eq!(ambiguous_error.kind, RustChildActionPlanErrorKind::IncompleteGraph);
        assert!(ambiguous_error.message.contains("2 graph producers"));
    }

    #[test]
    fn rust_graph_and_toolchain_adapters_preserve_producer_authority() {
        let units = rust_unit_action_inputs_from_graph(&graph()).unwrap();
        let authorities = fixed_executable_authorities_from_toolchain_closure(&closure()).unwrap();
        let plan = plan_rust_child_actions(RustChildActionPlanInput {
            stage_id: "mantle-stage1".to_string(),
            resources: resources(),
            fixed_executables: authorities,
            units,
        })
        .unwrap();

        assert_eq!(plan.action_count, 3);
        assert_eq!(plan.fixed_executables.len(), 2);
        assert!(plan.fixed_executables.iter().all(|authority| !authority.producer_action_id.is_empty()));
        assert!(plan.actions.iter().any(|action| !action.producer_action_ids.is_empty()));
    }

    #[test]
    fn rust_graph_and_toolchain_adapters_reject_unbound_or_seed_authority() {
        let mut unbound = graph();
        unbound.derivations[1].dependency_artifacts[0].producer_unit_id = None;
        let graph_error = rust_unit_action_inputs_from_graph(&unbound).unwrap_err();
        let mut seeded = closure();
        seeded.seed_exceptions.push(crate::source_toolchain_closure::ToolchainSeedException {
            name: "ambient-shell".to_string(),
            reason: "negative fixture".to_string(),
            content_digest_blake3: DIGEST_A.to_string(),
        });
        let closure_error = fixed_executable_authorities_from_toolchain_closure(&seeded).unwrap_err();

        assert_eq!(graph_error.kind, RustChildActionPlanErrorKind::IncompleteGraph);
        assert!(graph_error.message.contains("unbound dependency artifact producer"));
        assert_eq!(closure_error.kind, RustChildActionPlanErrorKind::InvalidInput);
        assert!(closure_error.message.contains("seed exceptions"));
    }

    #[test]
    fn rust_child_authority_is_deterministic_and_rejects_digest_drift() {
        let first =
            rust_child_action_authority("mantle-stage1".to_string(), resources(), input().fixed_executables).unwrap();
        let repeated =
            rust_child_action_authority("mantle-stage1".to_string(), resources(), input().fixed_executables).unwrap();
        let mut changed = first.clone();
        changed.fixed_executables[0].digest_blake3 = DIGEST_D.to_string();

        let error = validate_rust_child_action_authority(&changed).unwrap_err();

        assert_eq!(first.authority_digest_blake3, repeated.authority_digest_blake3);
        assert_eq!(error.kind, RustChildActionPlanErrorKind::InvalidInput);
        assert!(error.message.contains("output identity differs") || error.message.contains("digest mismatch"));
    }

    #[test]
    fn rust_child_plan_binds_fixed_and_produced_executable_authority() {
        let plan = plan_rust_child_actions(input()).unwrap();
        let repeated = plan_rust_child_actions(input()).unwrap();

        assert_eq!(plan.action_count, 3);
        assert_eq!(plan.plan_digest_blake3, repeated.plan_digest_blake3);
        assert!(plan.actions.iter().all(|action| action.local_only));
        assert!(
            plan.actions
                .iter()
                .any(|action| matches!(action.executable, RustChildExecutableAuthority::Produced { .. }))
        );
        assert!(plan.actions.iter().all(|action| action.event_count_max == TEST_EXEC_EVENTS_PER_ACTION_MAX));
    }

    #[test]
    fn rust_child_plan_exports_exact_protected_exec_policy_inputs() {
        let plan = plan_rust_child_actions(input()).unwrap();
        let (producers, executables) = protected_exec_inputs(&plan).unwrap();
        let policy = crate::protected_exec::ProtectedExecPolicy::from_action_plan(&producers, &executables).unwrap();

        assert_eq!(executables.len(), plan.fixed_executables.len());
        assert!(producers.iter().any(|producer| producer == "provider:rustc"));
        assert!(producers.iter().any(|producer| producer == &plan.actions[0].action_id));
        assert_eq!(policy.inventory_digest_blake3().len(), BLAKE3_HEX_LENGTH);
    }

    #[test]
    fn rust_child_plan_rejects_unknown_producer_and_path_only_authority() {
        let mut unknown = input();
        unknown.units[0].producer_unit_ids.push("missing".to_string());
        let unknown_error = plan_rust_child_actions(unknown).unwrap_err();
        let mut path_only = input();
        path_only.fixed_executables[0].producer_action_id.clear();
        let path_error = plan_rust_child_actions(path_only).unwrap_err();

        assert_eq!(unknown_error.kind, RustChildActionPlanErrorKind::IncompleteGraph);
        assert!(unknown_error.message.contains("invalid producer"));
        assert_eq!(path_error.kind, RustChildActionPlanErrorKind::InvalidInput);
        assert!(path_error.message.contains("producer_action_id"));
    }

    #[test]
    fn rust_child_reconciliation_accepts_exact_fixed_and_produced_events() {
        let plan = plan_rust_child_actions(input()).unwrap();
        let dep = plan
            .actions
            .iter()
            .find(|action| action.phase == RustChildActionPhase::CompileUnit && action.producer_action_ids.is_empty())
            .unwrap();
        let build_compile = plan
            .actions
            .iter()
            .find(|action| action.phase == RustChildActionPhase::CompileUnit && !action.producer_action_ids.is_empty())
            .unwrap();
        let build_run =
            plan.actions.iter().find(|action| action.phase == RustChildActionPhase::RunBuildScript).unwrap();
        let observations = vec![
            fixed_observation(dep, "rustc", "/provider/bin/rustc", DIGEST_A),
            fixed_observation(dep, "linker", "/provider/bin/ld", DIGEST_B),
            fixed_observation(build_compile, "rustc", "/provider/bin/rustc", DIGEST_A),
            produced_observation(build_run),
        ];

        let reconciliation = reconcile_rust_child_actions(&plan, &observations).unwrap();

        assert!(reconciliation.is_complete());
        assert_eq!(reconciliation.matched_action_count, plan.action_count);
        assert_eq!(reconciliation.matched_event_count, 4);
        assert!(reconciliation.blockers.is_empty());
    }

    #[test]
    fn rust_child_audit_binds_raw_and_normalized_events_and_rejects_tamper() {
        let plan = plan_rust_child_actions(input()).unwrap();
        let action = &plan.actions[0];
        let observations = vec![fixed_observation(action, "rustc", "/provider/bin/rustc", DIGEST_A)];
        let audit = rust_child_action_audit(
            &plan,
            vec![raw_event("/provider/bin/rustc", DIGEST_A)],
            observations,
            Vec::new(),
            1,
        )
        .unwrap();
        let mut changed = audit.clone();
        changed.raw_events[0].reason = "changed".to_string();

        let error = validate_rust_child_action_audit(&plan, &changed).unwrap_err();

        assert_eq!(audit.raw_event_count, audit.assigned_event_count);
        assert_eq!(audit.audit_digest_blake3.len(), BLAKE3_HEX_LENGTH);
        assert_eq!(error.kind, RustChildActionPlanErrorKind::InvalidInput);
        assert!(error.message.contains("audit digest mismatch"));
    }

    #[test]
    fn rust_child_reconciliation_rejects_denied_unknown_and_missing_events() {
        let plan = plan_rust_child_actions(input()).unwrap();
        let action = &plan.actions[0];
        let mut denied = fixed_observation(action, "rustc", "/provider/bin/rustc", DIGEST_A);
        denied.policy_decision = "denied".to_string();
        let mut unknown = fixed_observation(action, "linker", "/provider/bin/ld", DIGEST_B);
        unknown.action_id = "unknown-action".to_string();

        let observations = [denied, unknown];
        let report = rust_child_action_reconciliation(&plan, &observations).unwrap();
        let error = reconcile_rust_child_actions(&plan, &observations).unwrap_err();

        assert!(!report.is_complete());
        assert_eq!(report.denied_event_ids_blake3.len(), 1);
        assert_eq!(report.unknown_event_ids_blake3.len(), 1);
        assert_eq!(error.kind, RustChildActionPlanErrorKind::IncompleteReconciliation);
        assert!(error.message.contains("denied Rust child events"));
        assert!(error.message.contains("unknown Rust child events"));
        assert!(error.message.contains("missing Rust actions"));
    }

    #[test]
    fn rust_child_reconciliation_rejects_digest_drift() {
        let plan = plan_rust_child_actions(input()).unwrap();
        let action = &plan.actions[0];
        let observation = fixed_observation(action, "rustc", "/provider/bin/rustc", DIGEST_D);

        let error = reconcile_rust_child_actions(&plan, &[observation]).unwrap_err();

        assert_eq!(error.kind, RustChildActionPlanErrorKind::IncompleteReconciliation);
        assert!(error.message.contains("drifted Rust child events"));
        assert!(error.message.contains("missing Rust actions"));
    }
}
