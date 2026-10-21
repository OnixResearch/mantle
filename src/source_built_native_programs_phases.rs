//! Source-owned GCC specs expansion followed by exact individual native leaves.
//! The GCC driver executes only `-###`; its original specs, not a replacement
//! flag table, choose the phase arguments. No process-launching driver is used
//! to compile, assemble or link an object.
#![allow(dead_code)] // Pending wiring in the source-only preparation path (Cairn prove-source-built-mantle-fixed-point I3).
use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::errors::RunError;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::RecipeExitPolicy;
use crate::source_built_recipe::RecipeGuard;
use crate::source_built_recipe::RecipeGuardCondition;
use crate::source_built_recipe::RecipeOperation;
use crate::source_built_recipe::RecipeOutput;
use crate::source_built_recipe::RecipeStep;
use crate::source_built_recipe::RecipeWord;
use crate::source_built_recipe::SourceRecipe;
use crate::source_built_recipe::WordPart;

pub(crate) const GCC_PHASE_HELPER_SOURCE: &str = include_str!("source_built_native_programs_gcc_phases.c");
const LOG_BYTES_MAX: u64 = 16 * 1024 * 1024;
const RESPONSE_BYTES_MAX: u64 = 1024 * 1024;

/// One original specs-selected phase. `expected_image` is the precise image
/// named by GCC, independently of `executable`, which is its exact Produced
/// executable binding. A tree prefix is never executable authority.
pub(crate) struct GccPhase {
    pub executable: ArtifactRef,
    pub expected_image: RecipeWord,
    pub inputs: BTreeMap<String, ArtifactRef>,
    pub outputs: BTreeMap<String, RecipeOutput>,
    /// Source virtual path prefix -> direct leaf virtual path prefix. Every
    /// destination must name one of this phase's declared inputs or outputs.
    pub path_mappings: BTreeMap<String, String>,
    pub runtime_mounts: BTreeMap<String, String>,
    /// Phase-only environment, such as the original collect2 query mode and
    /// its Produced driver metadata log. These do not affect the specs query.
    pub environment: BTreeMap<String, RecipeWord>,
    /// Fresh OUTPUT LABEL -> exact Produced INPUT LABEL, common immutable
    /// versioning before the unchanged tool executes.
    pub output_seeds: BTreeMap<String, String>,
    pub cwd_output: Option<String>,
    pub guard: Option<RecipeGuard>,
    pub exit_policy: RecipeExitPolicy,
    pub merge_stderr: bool,
    /// Produced canonical invocation for this exact executable. When selected,
    /// it supplies the response-file argument and real source environment;
    /// `environment` must be empty rather than silently inventing defaults.
    pub invocation_input: Option<String>,
}

pub(crate) struct GccInvocation {
    pub id: String,
    pub driver: ArtifactRef,
    pub splitter: ArtifactRef,
    pub inputs: BTreeMap<String, ArtifactRef>,
    /// The original driver arguments. The author adds only `-###`.
    pub arguments: Vec<RecipeWord>,
    pub environment: BTreeMap<String, RecipeWord>,
    pub runtime_mounts: BTreeMap<String, String>,
    pub cwd_input: Option<String>,
    pub phases: Vec<GccPhase>,
}

/// Response-producing original generator phases, such as collect2's guarded
/// non-collecting first-link query, feed this pipeline without inventing argv.
pub(crate) struct GccResponseInvocation {
    pub id: String,
    pub splitter: ArtifactRef,
    pub inputs: BTreeMap<String, ArtifactRef>,
    pub log: ArtifactRef,
    pub environment: BTreeMap<String, RecipeWord>,
    pub cwd_input: Option<String>,
    pub phases: Vec<GccPhase>,
}

pub(crate) struct GccExpansion {
    /// Assign these exact steps to the owning original native derivation.
    pub step_ids: BTreeSet<String>,
    pub phase_outputs: Vec<BTreeMap<String, ArtifactRef>>,
}

/// Probe outcomes retain the real specs-driver status and every actual phase
/// envelope. A rejected option can fail before cc1 exists; later phases then
/// skip without manufacturing assembly, objects, binaries or execution records.
pub(crate) struct GccProbeExpansion {
    pub expansion: GccExpansion,
    pub specs_status: ArtifactRef,
    pub specs_outcome: ArtifactRef,
    pub phase_outcomes: Vec<ArtifactRef>,
    /// Bind these exact real status inputs and ordered predicates when the
    /// original probe next invokes collect2's actual linker or runs its binary.
    pub success_inputs: BTreeMap<String, ArtifactRef>,
    pub success_guard: RecipeGuard,
}

fn error(message: impl Into<String>) -> RunError {
    RunError::Build(format!("native GCC phase authoring: {}", message.into()))
}
fn produced(stage: &str, id: &str, output: &str) -> ArtifactRef {
    ArtifactRef::Produced {
        action_id: format!("{stage}/{id}"),
        output_id: output.to_string(),
    }
}
fn literal(text: impl Into<String>) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Literal { text: text.into() }],
    }
}
fn input(name: &str) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Input {
            name: name.to_string(),
            path: String::new(),
        }],
    }
}
fn output(name: &str) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Output {
            name: name.to_string(),
            path: String::new(),
        }],
    }
}
fn text_output(bytes_max: u64) -> RecipeOutput {
    RecipeOutput {
        kind: ArtifactKind::Text,
        bytes_max,
    }
}
fn require_produced(reference: &ArtifactRef) -> Result<(), RunError> {
    if matches!(reference, ArtifactRef::Produced { .. }) {
        Ok(())
    } else {
        Err(error("driver, splitter and phase tools must be exact Produced artifacts"))
    }
}
fn insert_input(
    inputs: &mut BTreeMap<String, ArtifactRef>,
    name: String,
    reference: ArtifactRef,
) -> Result<(), RunError> {
    if inputs.insert(name.clone(), reference).is_some() {
        return Err(error(format!("reserved or duplicate input {name}")));
    }
    Ok(())
}

fn inherit_guard_inputs(
    inputs: &mut BTreeMap<String, ArtifactRef>,
    phase: &GccPhase,
    guard: Option<&RecipeGuard>,
) -> Result<(), RunError> {
    if let Some(guard) = guard {
        for condition in &guard.conditions {
            if let Some(reference) = phase.inputs.get(&condition.status_input) {
                if let Some(existing) = inputs.get(&condition.status_input) {
                    if existing != reference {
                        return Err(error("guard status input identity collision"));
                    }
                } else {
                    inputs.insert(condition.status_input.clone(), reference.clone());
                }
            } else if !inputs.contains_key(&condition.status_input) {
                return Err(error(format!("unbound guard status {}", condition.status_input)));
            }
        }
    }
    Ok(())
}

/// The stable identity allows a subsequent phase to declare its preceding
/// assembly/object input before appending the complete source-owned pipeline.
pub(crate) fn gcc_phase_output(stage: &str, invocation: &str, phase: usize, output: &str) -> ArtifactRef {
    produced(stage, &format!("{invocation}.phase-{phase}"), output)
}

fn validate_mapping_destination(
    destination: &str,
    inputs: &BTreeMap<String, ArtifactRef>,
    outputs: &BTreeMap<String, RecipeOutput>,
    runtime_mounts: &BTreeMap<String, String>,
) -> Result<(), RunError> {
    let (rest, is_input) = if let Some(rest) = destination.strip_prefix("/inputs/") {
        (rest, true)
    } else if let Some(rest) = destination.strip_prefix("/outputs/") {
        (rest, false)
    } else {
        let root = runtime_mounts
            .keys()
            .find(|root| {
                destination == root.as_str()
                    || destination.strip_prefix(root.as_str()).is_some_and(|suffix| suffix.starts_with('/'))
            })
            .ok_or_else(|| error("phase path mapping leaves exact declared input/output/runtime roots"))?;
        crate::source_built_recipe::relative(
            destination.strip_prefix(root.as_str()).unwrap().strip_prefix('/').unwrap_or(""),
            true,
        )?;
        return Ok(());
    };
    crate::source_built_recipe::relative(rest, false)?;
    let name = rest.split('/').next().ok_or_else(|| error("empty mapped artifact"))?;
    if (is_input && !inputs.contains_key(name)) || (!is_input && !outputs.contains_key(name)) {
        return Err(error(format!("path mapping names undeclared artifact {name}")));
    }
    Ok(())
}

/// Append an entire specs query -> strict response production -> direct phase
/// pipeline atomically. Callers retain original source/configuration/generator
/// inputs; this function never invents flags, sysroots, objects or runtimes.
///
/// `collect2` is deliberately not accepted as `ld`: if the original driver
/// chooses collect2, its real source-owned phases require separate expansion.
pub(crate) fn append_gcc_invocation(
    program: &mut SourceRecipe,
    invocation: GccInvocation,
) -> Result<GccExpansion, RunError> {
    append_gcc_invocation_inner(program, invocation, None)
}

/// The original source controller emits the specs request with `-###` already
/// appended and the actual environment. Static arguments/environment must be
/// empty; this function cannot silently alter authenticated request bytes.
pub(crate) fn append_gcc_request_invocation(
    program: &mut SourceRecipe,
    invocation: GccInvocation,
    request_input: String,
) -> Result<GccExpansion, RunError> {
    append_gcc_invocation_inner(program, invocation, Some(request_input))
}

fn append_gcc_invocation_inner(
    program: &mut SourceRecipe,
    invocation: GccInvocation,
    request_input: Option<String>,
) -> Result<GccExpansion, RunError> {
    validate_specs_request(&invocation, request_input.as_deref())?;
    require_produced(&invocation.driver)?;
    let mut steps = Vec::with_capacity(1);
    let specs_id = format!("{}.specs", invocation.id);
    let query_guard = invocation.phases.first().and_then(|phase| phase.guard.clone());
    let mut query_inputs = invocation.inputs.clone();
    if let Some(phase) = invocation.phases.first() {
        inherit_guard_inputs(&mut query_inputs, phase, query_guard.as_ref())?;
    }
    insert_input(&mut query_inputs, "gcc-specs-driver".into(), invocation.driver)?;
    let mut arguments = invocation.arguments;
    if request_input.is_none() {
        arguments.push(literal("-###"));
    }
    steps.push(RecipeStep {
        id: specs_id.clone(),
        inputs: query_inputs,
        outputs: BTreeMap::from([
            ("stdout".into(), text_output(LOG_BYTES_MAX)),
            ("stderr".into(), text_output(LOG_BYTES_MAX)),
        ]),
        operation: RecipeOperation::Exec {
            executable_input: "gcc-specs-driver".into(),
            arguments,
            environment: invocation.environment.clone(),
            runtime_mounts: invocation.runtime_mounts,
            cwd_input: invocation.cwd_input.clone(),
            cwd_output: None,
            output_seeds: BTreeMap::new(),
            command_state: None,
            guard: query_guard,
            stdin_input: None,
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
            merge_stderr: false,
            invocation_input: request_input.clone(),
        },
    });
    let mut phase_inputs = invocation.inputs;
    let environment_source = request_input.as_ref().map(|name| phase_inputs[name].clone());
    if let Some(request) = &request_input {
        phase_inputs.remove(request);
    }
    let response = GccResponseInvocation {
        id: invocation.id,
        splitter: invocation.splitter,
        inputs: phase_inputs,
        log: produced(&program.stage_id, &specs_id, "stderr"),
        environment: invocation.environment,
        cwd_input: invocation.cwd_input,
        phases: invocation.phases,
    };
    append_response_steps(program, response, steps, environment_source)
}

/// Run the same original specs selection for a compiler feature probe, allowing
/// its normal rejection exit. The first rejecting actual command short-circuits
/// every subsequent child, matching GCC's original execute boundary.
pub(crate) fn append_gcc_probe_invocation(
    program: &mut SourceRecipe,
    invocation: GccInvocation,
) -> Result<GccProbeExpansion, RunError> {
    append_gcc_probe_invocation_inner(program, invocation, None)
}

pub(crate) fn append_gcc_probe_request_invocation(
    program: &mut SourceRecipe,
    invocation: GccInvocation,
    request_input: String,
) -> Result<GccProbeExpansion, RunError> {
    append_gcc_probe_invocation_inner(program, invocation, Some(request_input))
}

fn validate_specs_request(invocation: &GccInvocation, request: Option<&str>) -> Result<(), RunError> {
    if let Some(request) = request {
        if !invocation.arguments.is_empty() || !invocation.environment.is_empty() {
            return Err(error("authenticated specs request cannot coexist with static argv/environment"));
        }
        if invocation.cwd_input.is_some() {
            return Err(error(
                "authenticated specs request must carry its actual typed cwd rather than a static cwd input",
            ));
        }
        require_produced(invocation.inputs.get(request).ok_or_else(|| error("unbound specs invocation input"))?)?;
    }
    Ok(())
}

fn append_gcc_probe_invocation_inner(
    program: &mut SourceRecipe,
    mut invocation: GccInvocation,
    request_input: Option<String>,
) -> Result<GccProbeExpansion, RunError> {
    validate_specs_request(&invocation, request_input.as_deref())?;
    require_produced(&invocation.driver)?;
    if invocation.phases.is_empty() || invocation.phases.len() > 64 {
        return Err(error("GCC probe phase count must be 1..=64"));
    }
    let base_guard = invocation.phases[0].guard.clone();
    if invocation.phases.iter().any(|phase| phase.guard != base_guard) {
        return Err(error("probe phases must share their original outer guard"));
    }
    let specs_id = format!("{}.specs", invocation.id);
    inherit_guard_inputs(&mut invocation.inputs, &invocation.phases[0], base_guard.as_ref())?;
    let mut query_inputs = invocation.inputs.clone();
    insert_input(&mut query_inputs, "gcc-specs-driver".into(), invocation.driver)?;
    if request_input.is_none() {
        invocation.arguments.push(literal("-###"));
    }
    let query = RecipeStep {
        id: specs_id.clone(),
        inputs: query_inputs,
        outputs: BTreeMap::from([
            ("stdout".into(), text_output(LOG_BYTES_MAX)),
            ("stderr".into(), text_output(LOG_BYTES_MAX)),
            ("exit-status".into(), text_output(16)),
        ]),
        operation: RecipeOperation::Exec {
            executable_input: "gcc-specs-driver".into(),
            arguments: invocation.arguments,
            environment: invocation.environment.clone(),
            runtime_mounts: invocation.runtime_mounts,
            cwd_input: invocation.cwd_input.clone(),
            cwd_output: None,
            output_seeds: BTreeMap::new(),
            command_state: None,
            guard: base_guard.clone(),
            stdin_input: None,
            exit_policy: RecipeExitPolicy::Normal,
            merge_stderr: false,
            invocation_input: request_input.clone(),
        },
    };
    let environment_source = request_input.as_ref().map(|name| invocation.inputs[name].clone());
    if let Some(request) = &request_input {
        invocation.inputs.remove(request);
    }
    let specs_status = produced(&program.stage_id, &specs_id, "exit-status");
    let status_name = "gcc-probe-specs-status";
    insert_input(&mut invocation.inputs, status_name.into(), specs_status.clone())?;
    let mut conditions = base_guard.map(|guard| guard.conditions).unwrap_or_default();
    conditions.push(RecipeGuardCondition {
        status_input: status_name.into(),
        accepted_codes: vec![0],
    });
    let split_guard = RecipeGuard {
        conditions: conditions.clone(),
    };
    let mut prior_statuses = BTreeMap::<String, ArtifactRef>::new();
    for (ordinal, phase) in invocation.phases.iter_mut().enumerate() {
        for (name, reference) in &prior_statuses {
            insert_input(&mut phase.inputs, name.clone(), reference.clone())?;
        }
        phase.guard = Some(RecipeGuard {
            conditions: conditions.clone(),
        });
        phase.exit_policy = RecipeExitPolicy::OneOf { codes: vec![0, 1] };
        for (name, limit) in [
            ("stdout", LOG_BYTES_MAX),
            ("stderr", LOG_BYTES_MAX),
            ("exit-status", 16),
        ] {
            if let Some(existing) = phase.outputs.get(name) {
                if existing.kind != ArtifactKind::Text {
                    return Err(error(format!("probe reserved output {name} is not text")));
                }
            } else {
                phase.outputs.insert(name.into(), text_output(limit));
            }
        }
        let name = format!("gcc-probe-phase-{ordinal}-status");
        let status = gcc_phase_output(&program.stage_id, &invocation.id, ordinal, "exit-status");
        prior_statuses.insert(name.clone(), status);
        conditions.push(RecipeGuardCondition {
            status_input: name,
            accepted_codes: vec![0],
        });
    }
    let success_inputs = conditions
        .iter()
        .map(|condition| {
            let reference = invocation
                .inputs
                .get(&condition.status_input)
                .or_else(|| prior_statuses.get(&condition.status_input))
                .ok_or_else(|| error("unbound original probe success predicate"))?;
            Ok((condition.status_input.clone(), reference.clone()))
        })
        .collect::<Result<BTreeMap<_, _>, RunError>>()?;
    let success_guard = RecipeGuard { conditions };
    let mut observations = Vec::with_capacity(invocation.phases.len() + 1);
    let mut phase_outcomes = Vec::with_capacity(invocation.phases.len());
    let mut specs_outcome = None;
    let observed_phases = std::iter::once((specs_id.clone(), &query.outputs)).chain(
        invocation
            .phases
            .iter()
            .enumerate()
            .map(|(ordinal, phase)| (format!("{}.phase-{ordinal}", invocation.id), &phase.outputs)),
    );
    for (index, (phase, outputs)) in observed_phases.enumerate() {
        let id = format!("{phase}.observed");
        if program.steps.iter().any(|step| step.id == id) {
            return Err(error("GCC probe observation identity collision"));
        }
        let bytes_max = outputs.values().try_fold(64 * 1024u64, |sum, output| {
            sum.checked_add(output.bytes_max).ok_or_else(|| error("probe observation bound overflow"))
        })?;
        let reference = produced(&program.stage_id, &id, "outcome");
        observations.push(RecipeStep {
            id,
            inputs: BTreeMap::new(),
            outputs: BTreeMap::from([("outcome".into(), RecipeOutput {
                kind: ArtifactKind::Tree,
                bytes_max,
            })]),
            operation: RecipeOperation::ObservePhase {
                phase,
                output: "outcome".into(),
                state: None,
            },
        });
        if index == 0 {
            specs_outcome = Some(reference);
        } else {
            phase_outcomes.push(reference);
        }
    }
    let log = produced(&program.stage_id, &specs_id, "stderr");
    let mut expansion = append_response_steps_guarded(
        program,
        GccResponseInvocation {
            id: invocation.id,
            splitter: invocation.splitter,
            inputs: invocation.inputs,
            log,
            environment: invocation.environment,
            cwd_input: invocation.cwd_input,
            phases: invocation.phases,
        },
        vec![query],
        Some(split_guard),
        environment_source,
    )?;
    expansion.step_ids.extend(observations.iter().map(|step| step.id.clone()));
    program.steps.extend(observations);
    Ok(GccProbeExpansion {
        expansion,
        specs_status,
        specs_outcome: specs_outcome.expect("specs observation"),
        phase_outcomes,
        success_inputs,
        success_guard,
    })
}

pub(crate) fn append_gcc_response_phases(
    program: &mut SourceRecipe,
    invocation: GccResponseInvocation,
) -> Result<GccExpansion, RunError> {
    append_response_steps(program, invocation, Vec::new(), None)
}

/// Expand an actual source-owned collect2 query while retaining the original
/// invocation's typed environment, mutable workspace and descriptor scope.
pub(crate) fn append_gcc_request_response_phases(
    program: &mut SourceRecipe,
    invocation: GccResponseInvocation,
    environment_source: ArtifactRef,
) -> Result<GccExpansion, RunError> {
    append_response_steps(program, invocation, Vec::new(), Some(environment_source))
}

fn append_response_steps(
    program: &mut SourceRecipe,
    invocation: GccResponseInvocation,
    steps: Vec<RecipeStep>,
    environment_source: Option<ArtifactRef>,
) -> Result<GccExpansion, RunError> {
    let split_guard = invocation.phases.first().and_then(|phase| phase.guard.clone());
    append_response_steps_guarded(program, invocation, steps, split_guard, environment_source)
}

fn append_response_steps_guarded(
    program: &mut SourceRecipe,
    invocation: GccResponseInvocation,
    mut steps: Vec<RecipeStep>,
    split_guard: Option<RecipeGuard>,
    environment_source: Option<ArtifactRef>,
) -> Result<GccExpansion, RunError> {
    require_produced(&invocation.splitter)?;
    require_produced(&invocation.log)?;
    if invocation.phases.is_empty() || invocation.phases.len() > 64 {
        return Err(error("GCC phase count must be 1..=64"));
    }
    steps.reserve(1 + 2 * invocation.phases.len());
    let split_id = format!("{}.split", invocation.id);
    let mut split_inputs = invocation.inputs.clone();
    inherit_guard_inputs(&mut split_inputs, &invocation.phases[0], split_guard.as_ref())?;
    insert_input(&mut split_inputs, "gcc-phase-splitter".into(), invocation.splitter)?;
    insert_input(&mut split_inputs, "gcc-phase-log".into(), invocation.log)?;
    let mut split_arguments = vec![input("gcc-phase-log")];
    if let Some(source) = &environment_source {
        insert_input(&mut split_inputs, "gcc-source-invocation".into(), source.clone())?;
        split_arguments.push(literal("--invocations"));
        split_arguments.push(input("gcc-source-invocation"));
    }
    let mut split_outputs = BTreeMap::new();
    let mut split_runtime_mounts = BTreeMap::new();
    let mut phase_steps = Vec::with_capacity(invocation.phases.len());
    let mut phase_outputs = Vec::with_capacity(invocation.phases.len());
    for (ordinal, mut phase) in invocation.phases.into_iter().enumerate() {
        if environment_source.is_some() {
            for root in phase.runtime_mounts.keys() {
                phase.path_mappings.entry(root.clone()).or_insert_with(|| root.clone());
            }
        }
        if phase.cwd_output.is_some() && invocation.cwd_input.is_some() {
            return Err(error("phase writable cwd conflicts with original input cwd"));
        }
        require_produced(&phase.executable)?;
        for part in &phase.expected_image.parts {
            let name = match part {
                WordPart::Runtime { mount, .. } => {
                    let name = phase
                        .runtime_mounts
                        .get(mount)
                        .ok_or_else(|| error("expected GCC image runtime is not declared"))?;
                    if let Some(previous) = split_runtime_mounts.insert(mount.clone(), name.clone())
                        && previous != *name
                    {
                        return Err(error("GCC image runtime alias collision"));
                    }
                    Some(name)
                }
                WordPart::Input { name, .. } | WordPart::TextInput { name } | WordPart::TextSlice { name, .. } => {
                    Some(name)
                }
                WordPart::Literal { .. } => None,
                WordPart::CwdRelative { path } => {
                    if phase.expected_image.parts.len() != 1 {
                        return Err(error("cwd-relative GCC image requires one whole operand"));
                    }
                    crate::source_built_recipe::relative(path.strip_prefix("./").unwrap_or(path), false)?;
                    None
                }
                WordPart::Output { .. } => return Err(error("expected GCC image cannot be a future splitter output")),
                WordPart::DirectorySeparator {} => {
                    return Err(error("a GCC phase executable image cannot have a directory suffix"));
                }
                WordPart::Workspace { .. } => {
                    return Err(error("expected GCC image cannot depend on a future mutable splitter workspace"));
                }
            };
            if let Some(name) = name {
                let reference = phase
                    .inputs
                    .get(name)
                    .or_else(|| invocation.inputs.get(name))
                    .ok_or_else(|| error("unbound expected GCC image operand"))?;
                if let Some(previous) = split_inputs.get(name) {
                    if previous != reference {
                        return Err(error("GCC image operand identity collision"));
                    }
                } else {
                    split_inputs.insert(name.clone(), reference.clone());
                }
            }
        }
        let phase_id = format!("{}.phase-{ordinal}", invocation.id);
        let map_id = format!("{}.paths-{ordinal}", invocation.id);
        let map_input = format!("gcc-phase-map-{ordinal}");
        let response_output = format!("response-{ordinal}");
        let mut phase_inputs = invocation.inputs.clone();
        for (name, reference) in phase.inputs {
            insert_input(&mut phase_inputs, name, reference)?;
        }
        insert_input(&mut phase_inputs, "gcc-phase-tool".into(), phase.executable)?;
        insert_input(
            &mut phase_inputs,
            "gcc-phase-response".into(),
            produced(&program.stage_id, &split_id, &response_output),
        )?;
        let typed_invocation = environment_source.is_some();
        if typed_invocation && phase.invocation_input.is_some() {
            return Err(error("source specs phases already derive their exact invocation from the source request"));
        }
        if typed_invocation {
            let data_output = format!("data-{ordinal}");
            insert_input(
                &mut phase_inputs,
                "gcc-phase-data".into(),
                produced(&program.stage_id, &split_id, &data_output),
            )?;
            split_outputs.insert(data_output.clone(), text_output(LOG_BYTES_MAX));
            let overlay_id = format!("{}.environment-{ordinal}", invocation.id);
            let overlay_input = format!("gcc-phase-environment-{ordinal}");
            let cwd = phase
                .cwd_output
                .as_ref()
                .map(|name| output(name))
                .or_else(|| invocation.cwd_input.as_ref().map(|name| input(name)));
            let overlay = crate::source_built_recipe::RecipeInvocation {
                arguments: Vec::new(),
                environment: phase.environment.clone(),
                cwd,
                argv0: None,
                executable_path: None,
                workspace_mounts: BTreeMap::new(),
                readonly_input_roots: Vec::new(),
                process_attributes: None,
                descriptors: None,
            };
            steps.push(RecipeStep {
                id: overlay_id.clone(),
                inputs: BTreeMap::new(),
                outputs: BTreeMap::from([("environment".into(), text_output(64 * 1024 * 1024))]),
                operation: RecipeOperation::WriteText {
                    output: "environment".into(),
                    text: serde_json::to_string(&overlay)
                        .map_err(|e| error(format!("original phase environment encoding: {e}")))?,
                },
            });
            insert_input(&mut split_inputs, overlay_input, produced(&program.stage_id, &overlay_id, "environment"))?;
        }
        let mut phase_environment = invocation.environment.clone();
        for (name, word) in phase.environment {
            if phase_environment.insert(name.clone(), word).is_some() {
                return Err(error(format!("phase environment shadows original driver policy {name}")));
            }
        }
        let mut map_text = String::new();
        for (source, destination) in phase.path_mappings {
            let source_relative = source
                .strip_prefix('/')
                .ok_or_else(|| error("mapping source must be an exact absolute virtual path"))?;
            crate::source_built_recipe::relative(source_relative, false)?;
            validate_mapping_destination(&destination, &phase_inputs, &phase.outputs, &phase.runtime_mounts)?;
            map_text.push_str(&source);
            map_text.push('\t');
            map_text.push_str(&destination);
            if let Some(root) = phase.runtime_mounts.keys().find(|root| {
                destination == root.as_str()
                    || destination.strip_prefix(root.as_str()).is_some_and(|suffix| suffix.starts_with('/'))
            }) {
                map_text.push('\t');
                map_text.push_str(root);
            }
            map_text.push('\n');
        }
        if map_text.is_empty() || map_text.len() > RESPONSE_BYTES_MAX as usize {
            return Err(error("empty or oversized phase path map"));
        }
        steps.push(RecipeStep {
            id: map_id.clone(),
            inputs: BTreeMap::new(),
            outputs: BTreeMap::from([("map".into(), text_output(RESPONSE_BYTES_MAX))]),
            operation: RecipeOperation::WriteText {
                output: "map".into(),
                text: map_text,
            },
        });
        insert_input(&mut split_inputs, map_input.clone(), produced(&program.stage_id, &map_id, "map"))?;
        // The splitter compares image spelling without resolving it. Preserve
        // cwd-relative spelling as metadata, not as a path in the splitter's cwd;
        // the phase still executes its exact Produced image in its original cwd.
        if phase
            .expected_image
            .parts
            .iter()
            .all(|part| matches!(part, WordPart::Literal { .. } | WordPart::CwdRelative { .. }))
        {
            let mut image = String::new();
            for part in phase.expected_image.parts {
                let text = match part {
                    WordPart::Literal { text } => text,
                    WordPart::CwdRelative { path } => path,
                    _ => unreachable!("exact image metadata"),
                };
                image.push_str(&text);
            }
            if image.is_empty() || image.len() > 65_536 || image.contains(['\0', '\n', '\r']) {
                return Err(error("invalid exact GCC image metadata"));
            }
            let image_id = format!("{}.image-{ordinal}", invocation.id);
            let image_input = format!("gcc-phase-image-{ordinal}");
            steps.push(RecipeStep {
                id: image_id.clone(),
                inputs: BTreeMap::new(),
                outputs: BTreeMap::from([("image".into(), text_output(65_536))]),
                operation: RecipeOperation::WriteText {
                    output: "image".into(),
                    text: image,
                },
            });
            insert_input(&mut split_inputs, image_input.clone(), produced(&program.stage_id, &image_id, "image"))?;
            split_arguments.push(RecipeWord {
                parts: vec![WordPart::TextInput { name: image_input }],
            });
        } else {
            split_arguments.push(phase.expected_image);
        }
        split_arguments.push(input(&map_input));
        split_arguments.push(output(&response_output));
        split_outputs.insert(
            response_output,
            text_output(if typed_invocation {
                64 * 1024 * 1024
            } else {
                RESPONSE_BYTES_MAX
            }),
        );
        if typed_invocation {
            split_arguments.push(output(&format!("data-{ordinal}")));
            split_arguments.push(input(&format!("gcc-phase-environment-{ordinal}")));
        }
        // These identities are available to guarded consumers only when the
        // common validator proves ordered producer-success domination. Keeping
        // them here does not assert that the original phase actually executed.
        phase_outputs.push(
            phase
                .outputs
                .keys()
                .map(|name| (name.clone(), produced(&program.stage_id, &phase_id, name)))
                .collect(),
        );
        let invocation_input = if typed_invocation {
            Some("gcc-phase-response".into())
        } else {
            phase.invocation_input
        };
        let arguments = if invocation_input.is_some() {
            if typed_invocation {
                phase_environment.clear();
            } else if !phase_environment.is_empty() {
                return Err(error("authenticated phase request cannot coexist with static environment"));
            }
            Vec::new()
        } else {
            vec![RecipeWord {
                parts: vec![WordPart::Literal { text: "@".into() }, WordPart::Input {
                    name: "gcc-phase-response".into(),
                    path: String::new(),
                }],
            }]
        };
        phase_steps.push(RecipeStep {
            id: phase_id,
            inputs: phase_inputs,
            outputs: phase.outputs,
            operation: RecipeOperation::Exec {
                executable_input: "gcc-phase-tool".into(),
                arguments,
                environment: phase_environment,
                runtime_mounts: phase.runtime_mounts,
                cwd_input: if typed_invocation {
                    None
                } else {
                    invocation.cwd_input.clone()
                },
                cwd_output: if typed_invocation { None } else { phase.cwd_output },
                output_seeds: phase.output_seeds,
                command_state: None,
                guard: phase.guard,
                stdin_input: None,
                exit_policy: phase.exit_policy,
                merge_stderr: phase.merge_stderr,
                invocation_input,
            },
        });
    }
    steps.push(RecipeStep {
        id: split_id,
        inputs: split_inputs,
        outputs: split_outputs,
        operation: RecipeOperation::Exec {
            executable_input: "gcc-phase-splitter".into(),
            arguments: split_arguments,
            environment: BTreeMap::new(),
            runtime_mounts: split_runtime_mounts,
            cwd_input: None,
            cwd_output: None,
            output_seeds: BTreeMap::new(),
            command_state: None,
            guard: split_guard,
            stdin_input: None,
            exit_policy: RecipeExitPolicy::Exact { code: 0 },
            merge_stderr: false,
            invocation_input: None,
        },
    });
    steps.extend(phase_steps);
    let step_ids = steps.iter().map(|step| step.id.clone()).collect::<BTreeSet<_>>();
    if step_ids.len() != steps.len() || program.steps.iter().any(|step| step_ids.contains(&step.id)) {
        return Err(error("GCC invocation collides with an existing step identity"));
    }
    program.steps.extend(steps);
    Ok(GccExpansion {
        step_ids,
        phase_outputs,
    })
}
