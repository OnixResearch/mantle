//! Source-bound Cargo-free Mantle stages and their actual replay orchestration.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::process::Child;
use std::process::Command;

use crate::errors::RunError;
use crate::source_built_fixed_point::SourceAuthorityRole;
use crate::source_built_fixed_point::SourceBuiltFixedPointPlan;
use crate::source_built_fixed_point_action_trust::ActionExecutable;
use crate::source_built_fixed_point_action_trust::ActionInput;
use crate::source_built_fixed_point_action_trust::ActionSpec;
use crate::source_built_fixed_point_action_trust::StageActionAdapter;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::PreparedRecipe;
use crate::source_built_recipe::RecipeOperation;
use crate::source_built_recipe::RecipePaths;
use crate::source_built_recipe::RecipeStep;
use crate::source_built_recipe::RecipeWord;
use crate::source_built_recipe::WordPart;
use crate::source_built_recipe::{self};

pub(crate) struct AdapterPaths<'a> {
    pub(crate) stage_paths: [RecipePaths<'a>; 2],
    pub(crate) recipe_paths: [&'a str; 2],
    pub(crate) future_rustc: &'a Path,
    pub(crate) future_rustc_host: &'a str,
    pub(crate) targets: &'a [String],
}

pub(crate) struct PreparedMantleActions {
    recipes: [PreparedRecipe; 2],
    adapters: [StageActionAdapter; 2],
    semantic_unit_counts: [u32; 2],
    stage_roots: [PathBuf; 2],
    host_executable: PathBuf,
}

impl PreparedMantleActions {
    pub(crate) fn adapters(&self) -> &[StageActionAdapter; 2] {
        &self.adapters
    }
    pub(crate) fn recipes(&self) -> &[PreparedRecipe; 2] {
        &self.recipes
    }
}

/// Both source programs and both Cargo-free inventories are admitted before
/// either stage executes. No future compiler or stage1 executable is queried.
pub(crate) fn plan_actions(
    proof: &SourceBuiltFixedPointPlan,
    source_root: &Path,
    paths: &AdapterPaths<'_>,
    prior: &[&PreparedRecipe],
    host_mantle_digest: &str,
) -> Result<PreparedMantleActions, RunError> {
    let roots = [&paths.stage_paths[0], &paths.stage_paths[1]];
    if roots[0].runtime_root.starts_with(roots[1].runtime_root)
        || roots[1].runtime_root.starts_with(roots[0].runtime_root)
    {
        return Err(failure("stage1 and stage2 require independent fresh execution roots".into()));
    }
    for paths in roots {
        if paths.source_roots.get(&SourceAuthorityRole::MantleSource).map(PathBuf::as_path) != Some(source_root) {
            return Err(failure("recipe source root differs from Cargo-free source inventory".into()));
        }
    }
    let rust_provider = prior
        .iter()
        .find(|recipe| recipe.adapter().stage_id == "full-source-rust-provider")
        .ok_or_else(|| failure("missing source-produced Rust recipe".into()))?;
    let rustc = rust_provider.export("rustc")?;
    if rustc.kind != ArtifactKind::Executable {
        return Err(failure("Rust compiler export is not an executable".into()));
    }
    let first = source_built_recipe::prepare(
        proof,
        "mantle-stage1",
        SourceAuthorityRole::MantleSource,
        paths.recipe_paths[0],
        &paths.stage_paths[0],
        prior,
        host_mantle_digest,
    )?;
    let mut second_prior = prior.to_vec();
    second_prior.push(&first);
    let mut second = source_built_recipe::prepare(
        proof,
        "mantle-stage2",
        SourceAuthorityRole::MantleSource,
        paths.recipe_paths[1],
        &paths.stage_paths[1],
        &second_prior,
        host_mantle_digest,
    )?;
    let first_count = validate_unit_inventory(
        source_root,
        paths.future_rustc,
        paths.future_rustc_host,
        paths.targets,
        &paths.stage_paths[0],
        &first,
        &rustc.reference,
    )?;
    let second_count = validate_unit_inventory(
        source_root,
        paths.future_rustc,
        paths.future_rustc_host,
        paths.targets,
        &paths.stage_paths[1],
        &second,
        &rustc.reference,
    )?;
    let first_binary = first.export("mantle-stage1")?;
    let first_reference = first_binary.reference.clone();
    for step in &second.recipe.steps {
        if matches!(step.operation, RecipeOperation::Exec { .. }) {
            continue;
        }
        let id = format!("mantle-stage2/{}", step.id);
        let action = second
            .adapter
            .actions
            .iter_mut()
            .find(|action| action.id == id)
            .ok_or_else(|| failure("stage2 builtin lacks a canonical action".into()))?;
        let ArtifactRef::Produced { action_id, output_id } = &first_reference else {
            return Err(failure("stage1 Mantle is not individually produced".into()));
        };
        action.executable = ActionExecutable::Produced {
            action_id: action_id.clone(),
            output_id: output_id.clone(),
        };
        let input = ActionInput::Produced {
            action_id: action_id.clone(),
            output_id: output_id.clone(),
        };
        if !action.inputs.contains(&input) {
            action.inputs.push(input);
        }
        if !action.producer_actions.contains(action_id) {
            action.producer_actions.push(action_id.clone());
            action.producer_actions.sort();
        }
    }
    let mut first_adapter = first.adapter().clone();
    let mut second_adapter = second.adapter().clone();
    add_orchestrator(proof, &mut first_adapter, ActionExecutable::Fixed {
        digest_blake3: host_mantle_digest.to_string(),
    });
    let ArtifactRef::Produced { action_id, output_id } = first_reference else {
        return Err(failure("stage1 binary lacks an actual producer".into()));
    };
    add_orchestrator(proof, &mut second_adapter, ActionExecutable::Produced { action_id, output_id });
    let stage_roots = [stage_root(&first)?, stage_root(&second)?];
    if stage_roots[0].starts_with(&stage_roots[1])
        || stage_roots[1].starts_with(&stage_roots[0])
        || stage_roots[0].file_name() != Some(std::ffi::OsStr::new("stage1"))
        || stage_roots[1].file_name() != Some(std::ffi::OsStr::new("stage2"))
        || stage_roots[0].parent() != stage_roots[1].parent()
        || stage_roots[0].parent().and_then(Path::file_name) != Some(std::ffi::OsStr::new("cargo-free-fixed-point"))
    {
        return Err(failure(
            "Mantle stage roots must be independent cargo-free-fixed-point/stage1 and stage2 directories".into(),
        ));
    }
    if first
        .source_recipe()
        .steps
        .iter()
        .chain(&second.source_recipe().steps)
        .any(|step| step.id == "orchestrator")
    {
        return Err(failure("source recipe collides with mandatory orchestrator action".into()));
    }
    let host_executable =
        std::env::current_exe().map_err(|error| failure(format!("resolving host Mantle: {error}")))?;
    Ok(PreparedMantleActions {
        recipes: [first, second],
        adapters: [first_adapter, second_adapter],
        semantic_unit_counts: [first_count, second_count],
        stage_roots,
        host_executable,
    })
}

fn stage_root(recipe: &PreparedRecipe) -> Result<PathBuf, RunError> {
    recipe
        .runtime_root
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| failure("recipe runtime lacks a stage root".into()))
}

fn add_orchestrator(proof: &SourceBuiltFixedPointPlan, adapter: &mut StageActionAdapter, executable: ActionExecutable) {
    let mut inputs = adapter.actions[0].inputs.clone();
    if let ActionExecutable::Produced { action_id, output_id } = &executable {
        let input = ActionInput::Produced {
            action_id: action_id.clone(),
            output_id: output_id.clone(),
        };
        if !inputs.contains(&input) {
            inputs.push(input);
        }
    }
    let producer_actions = inputs
        .iter()
        .filter_map(|input| match input {
            ActionInput::Produced { action_id, .. } => Some(action_id.clone()),
            _ => None,
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let id = format!("{}/orchestrator", adapter.stage_id);
    adapter.reachable_action_ids.push(id.clone());
    adapter.reachable_action_ids.sort();
    adapter.actions.insert(0, ActionSpec {
        id,
        stage_id: adapter.stage_id.clone(),
        producer_actions,
        guard: None,
        accepted_exit_codes: Vec::new(),
        executable,
        inputs,
        output_ids: vec!["stderr".into(), "stdout".into()],
        protected_events_min: 1,
        protected_events_max: 1,
        build_events_min: 1,
        build_events_max: 1,
        elapsed_seconds_max: proof.resource_bounds.elapsed_seconds_max,
        disk_bytes_max: proof.resource_bounds.disk_bytes_max,
        local_only: true,
    });
}

fn validate_unit_inventory(
    source_root: &Path,
    rustc: &Path,
    compiler_host: &str,
    targets: &[String],
    paths: &RecipePaths<'_>,
    recipe: &PreparedRecipe,
    compiler: &ArtifactRef,
) -> Result<u32, RunError> {
    let vendor = paths
        .source_roots
        .get(&SourceAuthorityRole::VendorInputs)
        .ok_or_else(|| failure("missing independently admitted vendor inputs".into()))?;
    let options = crate::rust_plan::RustPlanOptions {
        root: source_root.to_path_buf(),
        cargo: PathBuf::from("cargo-forbidden"),
        rustc: rustc.to_path_buf(),
        host_triple: compiler_host.to_string(),
        targets: targets.to_vec(),
        profile: crate::rust_plan::default_profile(),
        profile_table: crate::rust_plan::load_root_profile_table(source_root)?,
        workspace_members: BTreeSet::new(),
        features: Vec::new(),
        all_features: false,
        no_default_features: false,
        no_cargo_oracle: true,
        deterministic_release_paths: true,
        path_remaps: crate::rust_plan::deterministic_release_path_remaps(source_root, Some(paths.runtime_root)),
    };
    let topology = crate::rust_plan::plan_cargo_free_unit_topology(&options, std::slice::from_ref(vendor))?;
    let mut expected = BTreeSet::new();
    for effect in &topology.effects {
        expected.insert(format!("rust-unit:{}", effect.unit_id.0));
        if effect.target_kind == "custom-build" {
            expected.insert(format!("build-script:{}", effect.unit_id.0));
        }
    }
    if recipe.semantic_roots().keys().cloned().collect::<BTreeSet<_>>() != expected {
        return Err(failure("source recipe omits, duplicates, or adds Cargo-free semantic units/build scripts".into()));
    }
    let steps = recipe
        .source_recipe()
        .steps
        .iter()
        .map(|step| (format!("{}/{}", recipe.adapter().stage_id, step.id), step))
        .collect::<BTreeMap<_, _>>();
    let mut claimed = BTreeSet::new();
    for effect in &topology.effects {
        let references = &recipe.semantic_roots()[&format!("rust-unit:{}", effect.unit_id.0)];
        if references.is_empty() {
            return Err(failure("empty Rust unit product set".into()));
        }
        let ancestry = ancestors(references, &steps)?;
        let metadata = effect
            .arguments
            .iter()
            .find(|argument| argument.starts_with("metadata="))
            .ok_or_else(|| failure("source unit lacks compiler disambiguator".into()))?;
        let mut compiles =
            ancestry.iter().filter_map(|id| steps.get(id).map(|step| (id, *step))).filter(|(_, step)| {
                match &step.operation {
                    RecipeOperation::Exec {
                        executable_input,
                        arguments,
                        ..
                    } => {
                        step.inputs.get(executable_input) == Some(compiler)
                            && arguments
                                .iter()
                                .filter_map(literal_word)
                                .any(|argument| argument == *metadata || argument == format!("-C{metadata}"))
                    }
                    _ => false,
                }
            });
        let (compile_id, compile) = compiles.next().ok_or_else(|| {
            failure(format!("unit {} has no actual source-produced compiler action", effect.unit_id.0))
        })?;
        if compiles.next().is_some() || !claimed.insert(compile_id.clone()) {
            return Err(failure("ambiguous or shared compiler unit action".into()));
        }
        validate_primary_source(effect, compile, paths)?;
        for dependency in &effect.dependency_unit_ids {
            let key = format!("rust-unit:{}", dependency.0);
            let products = recipe
                .semantic_roots()
                .get(&key)
                .ok_or_else(|| failure("unit dependency lacks semantic producer".into()))?;
            if !products.iter().any(|reference| matches!(reference, ArtifactRef::Produced { action_id, .. } if ancestry.contains(action_id))) {
                return Err(failure(format!("unit {} omits exact producer {}", effect.unit_id.0, dependency.0)));
            }
        }
        if effect.target_kind == "custom-build" {
            let generated = &recipe.semantic_roots()[&format!("build-script:{}", effect.unit_id.0)];
            let generated_ancestry = ancestors(generated, &steps)?;
            if !references.iter().any(|reference| matches!(reference, ArtifactRef::Produced { action_id, .. } if generated_ancestry.contains(action_id))) {
                return Err(failure("build-script generated products omit compiled script producer".into()));
            }
        }
    }
    let binary = recipe.export(&recipe.adapter().stage_id)?;
    if binary.kind != ArtifactKind::Executable
        || !matches!(&binary.reference, ArtifactRef::Produced { output_id, .. } if output_id == &recipe.adapter().stage_id)
    {
        return Err(failure("Mantle stage output is not its canonical produced executable".into()));
    }
    let main = topology
        .effects
        .iter()
        .filter(|effect| effect.target_name == "mantle" && effect.target_kind == "bin")
        .collect::<Vec<_>>();
    if main.len() != 1
        || !recipe.semantic_roots()[&format!("rust-unit:{}", main[0].unit_id.0)].contains(&binary.reference)
    {
        return Err(failure("canonical Mantle binary does not match unique Cargo-free Mantle unit".into()));
    }
    validate_smoke(recipe, &steps, &binary.reference)?;
    u32::try_from(topology.effects.len()).map_err(|_| failure("semantic unit count exceeds u32".into()))
}

fn ancestors(references: &[ArtifactRef], steps: &BTreeMap<String, &RecipeStep>) -> Result<BTreeSet<String>, RunError> {
    let mut queue = references.to_vec();
    let mut visited = BTreeSet::new();
    while let Some(reference) = queue.pop() {
        if let ArtifactRef::Produced { action_id, output_id } = reference
            && let Some(step) = steps.get(&action_id)
        {
            if !step.outputs.contains_key(&output_id) {
                return Err(failure("semantic root names unknown output".into()));
            }
            if visited.insert(action_id) {
                queue.extend(step.inputs.values().cloned());
            }
        }
    }
    Ok(visited)
}

fn literal_word(word: &RecipeWord) -> Option<String> {
    let mut text = String::new();
    for part in &word.parts {
        match part {
            WordPart::Literal { text: part } => text.push_str(part),
            _ => return None,
        }
    }
    Some(text)
}

fn validate_primary_source(
    effect: &mantle_rust_plan_core::ExistingUnitEffect,
    step: &RecipeStep,
    paths: &RecipePaths<'_>,
) -> Result<(), RunError> {
    let source = effect
        .arguments
        .iter()
        .find(|argument| !argument.starts_with('-') && argument.ends_with(".rs"))
        .ok_or_else(|| failure("source unit lacks primary Rust source".into()))?;
    let source = Path::new(source);
    let materialized;
    let source = if source.is_absolute() {
        source
    } else {
        let root = paths
            .source_roots
            .get(&SourceAuthorityRole::MantleSource)
            .ok_or_else(|| failure("missing admitted Mantle source root".into()))?;
        materialized = root.join(source);
        &materialized
    };
    let RecipeOperation::Exec { arguments, .. } = &step.operation else {
        return Err(failure("compiler unit is not executable".into()));
    };
    let literal_arguments = arguments.iter().filter_map(literal_word).collect::<Vec<_>>();
    let crate_name = effect.target_name.replace('-', "_");
    if !literal_arguments.windows(2).any(|pair| pair[0] == "--crate-name" && pair[1] == crate_name)
        && !literal_arguments.iter().any(|argument| argument == &format!("--crate-name={crate_name}"))
    {
        return Err(failure("compiler action does not name its exact Cargo-free crate".into()));
    }
    for (role, root) in paths.source_roots {
        if !matches!(role, SourceAuthorityRole::MantleSource | SourceAuthorityRole::VendorInputs) {
            continue;
        }
        let Ok(relative) = source.strip_prefix(root) else {
            continue;
        };
        for word in arguments {
            if let [WordPart::Input { name, path }] = word.parts.as_slice()
                && let Some(ArtifactRef::Source {
                    role: input_role,
                    path: member,
                }) = step.inputs.get(name)
                && input_role == role
                && Path::new(member).join(path) == relative
            {
                return Ok(());
            }
        }
    }
    Err(failure(format!(
        "unit {} compiler does not consume its exact admitted primary source",
        effect.unit_id.0
    )))
}

fn validate_smoke(
    recipe: &PreparedRecipe,
    steps: &BTreeMap<String, &RecipeStep>,
    binary: &ArtifactRef,
) -> Result<(), RunError> {
    let smoke = recipe.export("mantle-smoke")?;
    let ArtifactRef::Produced { action_id, output_id } = &smoke.reference else {
        return Err(failure("smoke lacks producer".into()));
    };
    let step = steps.get(action_id).ok_or_else(|| failure("smoke producer is not a local step".into()))?;
    match &step.operation {
        RecipeOperation::Exec {
            executable_input,
            arguments,
            exit_policy: source_built_recipe::RecipeExitPolicy::Exact { code: 0 },
            guard: None,
            ..
        } if output_id == "stdout"
            && step.inputs.get(executable_input) == Some(binary)
            && arguments.iter().map(literal_word).collect::<Option<Vec<_>>>() == Some(vec!["--help".into()]) =>
        {
            Ok(())
        }
        _ => Err(failure("Mantle smoke must execute exactly the actual produced binary --help".into())),
    }
}

const REPLAY_FRAME_BYTES_MAX: usize = 64 * 1024 * 1024;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
// Replay frames are serialized directly; keeping the variants unboxed preserves
// the single-message wire shape that the replay reader admits.
#[allow(clippy::large_enum_variant)]
enum ReplayMessage {
    Event {
        event: crate::source_built_fixed_point_action_trust::ActionExecutionEvent,
    },
    Completed {
        completions: Vec<crate::source_built_recipe_exec::RecipeCompletion>,
    },
}

fn write_replay_frame(
    stream: &mut std::os::unix::net::UnixStream,
    bytes: &[u8],
    written: &mut u64,
) -> std::io::Result<()> {
    use std::io::Error;
    use std::io::ErrorKind;
    use std::io::Write;
    for mut part in [&(bytes.len() as u32).to_be_bytes()[..], bytes] {
        while !part.is_empty() {
            match stream.write(part) {
                Ok(0) => return Err(Error::from(ErrorKind::WriteZero)),
                Ok(count) => {
                    *written += count as u64;
                    part = &part[count..];
                }
                Err(error) if error.kind() == ErrorKind::Interrupted => {}
                Err(error) => return Err(error),
            }
        }
    }
    Ok(())
}

fn replay_transport_plan() -> Result<mantle_application_contract::EffectPlan, RunError> {
    use mantle_application_contract::CommandFamily;
    use mantle_application_contract::EffectKind;
    use mantle_application_contract::EffectMeasure;
    use mantle_application_contract::EffectSpec;
    use mantle_application_contract::ExpectedOutput;
    use mantle_application_contract::plan_effects;
    plan_effects(CommandFamily::Bootstrap, &[
        EffectSpec {
            effect_id: "source-replay-frame",
            kind: EffectKind::InterprocessCommunication,
            limit: EffectMeasure::Bytes(REPLAY_FRAME_BYTES_MAX as u64 + 4),
            expected_output: ExpectedOutput::None,
        },
        EffectSpec {
            effect_id: "source-replay-parent-admission",
            kind: EffectKind::InterprocessCommunication,
            limit: EffectMeasure::Bytes(1),
            expected_output: ExpectedOutput::Identity("parent-admitted"),
        },
    ])
    .map_err(|error| failure(format!("planning replay IPC: {}", error.code())))
}

fn send_message(
    stream: &mut std::os::unix::net::UnixStream,
    message: &ReplayMessage,
    plan: &mantle_application_contract::EffectPlan,
) -> Result<(), RunError> {
    use mantle_application_contract::ApplicationOutcome;
    use mantle_application_contract::EffectId;
    use mantle_application_contract::EffectKind;
    use mantle_application_contract::EffectMeasure;
    use mantle_application_contract::EffectOutput;
    use mantle_application_contract::Observation;
    use mantle_application_contract::ObservationStatus;
    use mantle_application_contract::classify_observations;
    let bytes = serde_json::to_vec(message).map_err(|error| failure(error.to_string()))?;
    if bytes.len() > REPLAY_FRAME_BYTES_MAX {
        return Err(failure("replay transport frame exceeds bound".into()));
    }
    let frame_bytes = bytes.len() as u64 + 4;
    let mut written = 0;
    let mut received = 0;
    let mut admitted = [0u8; 1];
    let result = write_replay_frame(stream, &bytes, &mut written)
        .and_then(|()| {
            stream.read_exact(&mut admitted)?;
            received = 1;
            Ok(())
        })
        .map_err(|error| failure(format!("transferring actual replay observation: {error}")))
        .and_then(|()| {
            if admitted == [1] {
                Ok(())
            } else {
                Err(failure("parent rejected actual replay observation".into()))
            }
        });
    let observations = [
        Observation {
            effect_id: EffectId("source-replay-frame".into()),
            kind: EffectKind::InterprocessCommunication,
            status: if written == frame_bytes {
                ObservationStatus::Succeeded
            } else {
                ObservationStatus::Failed
            },
            output: EffectOutput::None,
            usage: EffectMeasure::Bytes(written),
            diagnostics_code: None,
        },
        Observation {
            effect_id: EffectId("source-replay-parent-admission".into()),
            kind: EffectKind::InterprocessCommunication,
            status: if result.is_ok() {
                ObservationStatus::Succeeded
            } else {
                ObservationStatus::Failed
            },
            output: if result.is_ok() {
                EffectOutput::Identity("parent-admitted".into())
            } else {
                EffectOutput::None
            },
            usage: EffectMeasure::Bytes(received),
            diagnostics_code: None,
        },
    ];
    match (classify_observations(plan, &observations), result) {
        (ApplicationOutcome::Completed, Ok(())) => Ok(()),
        (ApplicationOutcome::Failed { .. }, Err(error)) => Err(error),
        (outcome, _) => Err(failure(format!("replay IPC observations inconsistent: {outcome:?}"))),
    }
}

/// Executed only by the genuinely launched host/stage1 replay child. The socket
/// transports observations synchronously; it cannot import evidence files.
pub(crate) fn replay_stage(
    actions: &PreparedMantleActions,
    stage_id: &str,
    stream: &mut std::os::unix::net::UnixStream,
) -> Result<(), RunError> {
    let index = match stage_id {
        "mantle-stage1" => 0,
        "mantle-stage2" => 1,
        _ => return Err(failure("replay stage must be stage1 or stage2".into())),
    };
    let transport = replay_transport_plan()?;
    let result = source_built_recipe::run(&actions.recipes[index], &mut |event| {
        send_message(stream, &ReplayMessage::Event { event }, &transport)
    })?;
    send_message(
        stream,
        &ReplayMessage::Completed {
            completions: result.completions,
        },
        &transport,
    )
}

/// Launches both actual replay processes. In particular stage2 is never built
/// by the original host after merely smoking the stage1 binary.
pub(crate) fn run_with_action_observations(
    actions: &PreparedMantleActions,
    proof_root: &Path,
    action_plan_digest: &str,
    bwrap: &Path,
    emit: &mut dyn FnMut(crate::source_built_fixed_point_action_trust::ActionExecutionEvent) -> Result<(), RunError>,
) -> Result<(), RunError> {
    for index in 0..2 {
        run_stage(actions, index, proof_root, action_plan_digest, bwrap, emit)?;
    }
    Ok(())
}

fn run_stage(
    actions: &PreparedMantleActions,
    index: usize,
    proof_root: &Path,
    action_plan_digest: &str,
    bwrap: &Path,
    emit: &mut dyn FnMut(crate::source_built_fixed_point_action_trust::ActionExecutionEvent) -> Result<(), RunError>,
) -> Result<(), RunError> {
    use std::io::Write;
    use std::os::fd::AsRawFd;
    use std::os::unix::process::CommandExt;

    use crate::source_built_fixed_point_action_trust::ExecutionEventKind;
    let prepared = &actions.recipes[index];
    let root = &actions.stage_roots[index];
    std::fs::create_dir(root).map_err(|error| failure(format!("creating fresh Mantle stage root: {error}")))?;
    let create = |path: &Path| {
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|error| failure(error.to_string()))
    };
    let stdout_path = root.join("orchestrator-stdout");
    let stderr_path = root.join("orchestrator-stderr");
    let stdout = create(&stdout_path)?;
    let stderr = create(&stderr_path)?;
    let executable = if index == 0 {
        &actions.host_executable
    } else {
        &actions.recipes[0].export("mantle-stage1")?.path
    };
    let measured = source_built_recipe::digest(executable)?.1;
    let orchestrator = &actions.adapters[index].actions[0];
    let expected = match &orchestrator.executable {
        ActionExecutable::Fixed { digest_blake3 } => digest_blake3.clone(),
        ActionExecutable::Produced { .. } => {
            source_built_recipe::digest(&actions.recipes[0].export("mantle-stage1")?.path)?.1
        }
    };
    if measured != expected {
        return Err(failure("orchestrator executable identity changed before launch".into()));
    }
    let inputs = crate::source_built_recipe_exec::input_digests(prepared, orchestrator)?;
    let (mut parent, child_stream) =
        std::os::unix::net::UnixStream::pair().map_err(|error| failure(error.to_string()))?;
    let timeout = std::time::Duration::from_secs(prepared.proof.resource_bounds.elapsed_seconds_max);
    parent
        .set_read_timeout(Some(timeout))
        .and_then(|()| parent.set_write_timeout(Some(timeout)))
        .map_err(|error| failure(error.to_string()))?;
    let fd = child_stream.as_raw_fd();
    let mut command = Command::new(executable);
    command.env_clear();
    command.stdin(std::process::Stdio::null()).stdout(stdout).stderr(stderr);
    command
        .args(["bootstrap", "source-built-replay", "--proof-root"])
        .arg(proof_root)
        .args([
            "--stage-id",
            prepared.adapter.stage_id.as_str(),
            "--capture-fd",
            &fd.to_string(),
            "--expected-proof-digest",
            prepared.proof.plan_digest_blake3.as_str(),
            "--expected-action-plan-digest",
            action_plan_digest,
            "--host-mantle-digest",
            prepared.host_digest.as_str(),
            "--bwrap",
        ])
        .arg(bwrap);
    let file_limit = prepared.proof.resource_bounds.disk_bytes_max;
    unsafe {
        command.pre_exec(move || {
            let flags = libc::fcntl(fd, libc::F_GETFD);
            if flags < 0 || libc::fcntl(fd, libc::F_SETFD, flags & !libc::FD_CLOEXEC) < 0 {
                return Err(std::io::Error::last_os_error());
            }
            let limit = libc::rlimit {
                rlim_cur: file_limit,
                rlim_max: file_limit,
            };
            if libc::setrlimit(libc::RLIMIT_FSIZE, &limit) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let started = std::time::Instant::now();
    let mut child = launch_orchestrator(&mut command, &expected, &mut |digest| {
        emit(crate::source_built_recipe_exec::event(
            orchestrator,
            ExecutionEventKind::ProtectedExec,
            digest.into(),
            inputs.clone(),
            Vec::new(),
            digest == expected,
            digest != expected,
            started.elapsed().as_secs(),
            0,
        ))
    })?;
    drop(child_stream);
    let stage_action_ids = prepared.adapter.actions.iter().map(|action| action.id.as_str()).collect::<BTreeSet<_>>();
    let result = (|| {
        let mut completions = None;
        let mut normal_exits = BTreeMap::new();
        let mut guard_outcomes = BTreeMap::new();
        loop {
            let mut length = [0u8; 4];
            parent
                .read_exact(&mut length)
                .map_err(|error| failure(format!("reading live replay frame: {error}")))?;
            let length = u32::from_be_bytes(length) as usize;
            if length == 0 || length > REPLAY_FRAME_BYTES_MAX {
                return Err(failure("invalid bounded replay frame".into()));
            }
            let mut bytes = vec![0u8; length];
            parent.read_exact(&mut bytes).map_err(|error| failure(error.to_string()))?;
            let message: ReplayMessage = serde_json::from_slice(&bytes)
                .map_err(|error| failure(format!("decoding live replay frame: {error}")))?;
            let admitted = match message {
                ReplayMessage::Event { event } => {
                    if !stage_action_ids.contains(event.action_id.as_str()) {
                        // Preserve unknown events in the root journal before admission fails.
                        emit(event)?;
                        return Err(failure("replay emitted an action outside its prepared stage".into()));
                    }
                    match &event.kind {
                        ExecutionEventKind::GuardDecision { outcome, .. } => {
                            guard_outcomes.insert(event.action_id.clone(), *outcome);
                        }
                        ExecutionEventKind::BuildExecution => {
                            if let Some(code) = event.normal_exit_code {
                                normal_exits.insert(event.action_id.clone(), code);
                            }
                        }
                        ExecutionEventKind::ProtectedExec => {}
                    }
                    emit(event)
                }
                ReplayMessage::Completed { completions: observed } => {
                    completions = Some(observed);
                    Ok(())
                }
            };
            let acknowledge = [u8::from(admitted.is_ok())];
            parent.write_all(&acknowledge).map_err(|error| failure(error.to_string()))?;
            admitted?;
            if completions.is_some() {
                break;
            }
        }
        let status = loop {
            if let Some(status) = child.try_wait().map_err(|error| failure(error.to_string()))? {
                break status;
            }
            if started.elapsed() >= timeout {
                return Err(failure("Mantle replay wall-clock bound exceeded".into()));
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        };
        if !status.success() {
            return Err(failure(format!("Mantle replay child exited {status}")));
        }
        let mut commands = completions.ok_or_else(|| failure("replay omitted actual completion records".into()))?;
        validate_completions(prepared, &commands, &normal_exits, &guard_outcomes)?;
        let stdout_digest = source_built_recipe::digest(&stdout_path)?.1;
        let stderr_digest = source_built_recipe::digest(&stderr_path)?.1;
        emit(crate::source_built_recipe_exec::event(
            orchestrator,
            ExecutionEventKind::BuildExecution,
            expected.clone(),
            inputs.clone(),
            vec![
                ("stdout".into(), stdout_digest.clone()),
                ("stderr".into(), stderr_digest.clone()),
            ],
            true,
            false,
            started.elapsed().as_secs(),
            source_built_recipe::digest(&stdout_path)?.0 + source_built_recipe::digest(&stderr_path)?.0,
        ))?;
        commands.push(crate::source_built_recipe_exec::RecipeCompletion {
            action_id: orchestrator.id.clone(),
            exit_code: status.code().ok_or_else(|| failure("replay died by signal".into()))?,
            stdout_path: stdout_path.clone(),
            stdout_digest_blake3: stdout_digest,
            stderr_path: stderr_path.clone(),
            stderr_digest_blake3: stderr_digest,
        });
        write_stage_manifest(actions, index, action_plan_digest, &commands, &expected)
    })();
    if result.is_err() {
        let _ = child.kill();
        let _ = child.wait();
    }
    result
}

fn validate_completions(
    prepared: &PreparedRecipe,
    commands: &[crate::source_built_recipe_exec::RecipeCompletion],
    normal_exits: &BTreeMap<String, i32>,
    guard_outcomes: &BTreeMap<String, crate::source_built_fixed_point_action_trust::GuardOutcome>,
) -> Result<(), RunError> {
    use crate::source_built_fixed_point_action_trust::GuardOutcome;
    let mut expected = Vec::new();
    for step in &prepared.recipe.steps {
        let RecipeOperation::Exec { guard, .. } = &step.operation else {
            continue;
        };
        if guard.is_some() {
            let id = format!("{}/{}", prepared.recipe.stage_id, step.id);
            match guard_outcomes.get(&id) {
                Some(GuardOutcome::Active) => {}
                Some(GuardOutcome::Skipped) if !normal_exits.contains_key(&id) => continue,
                _ => return Err(failure("replay phase lacks an authenticated active/skipped decision".into())),
            }
        }
        expected.push(step);
    }
    if expected.len() != commands.len() || normal_exits.len() != commands.len() {
        return Err(failure("replay completion inventory differs from prepared leaf topology".into()));
    }
    for (step, command) in expected.into_iter().zip(commands) {
        let RecipeOperation::Exec { exit_policy, .. } = &step.operation else {
            unreachable!()
        };
        let base = prepared.runtime_root.join(&step.id).join("outputs");
        if command.action_id != format!("{}/{}", prepared.recipe.stage_id, step.id)
            || !exit_policy.accepts(command.exit_code)
            || normal_exits.get(&command.action_id) != Some(&command.exit_code)
            || command.stdout_path != base.join("stdout")
            || command.stderr_path != base.join("stderr")
            || source_built_recipe::digest(&command.stdout_path)?.1 != command.stdout_digest_blake3
            || source_built_recipe::digest(&command.stderr_path)?.1 != command.stderr_digest_blake3
        {
            return Err(failure("actual replay completion/log differs from its immutable leaf binding".into()));
        }
    }
    Ok(())
}

fn write_stage_manifest(
    actions: &PreparedMantleActions,
    index: usize,
    action_plan_digest: &str,
    commands: &[crate::source_built_recipe_exec::RecipeCompletion],
    orchestrator_digest: &str,
) -> Result<(), RunError> {
    use std::io::Write;

    use crate::source_built_fixed_point_trust_report::*;
    let prepared = &actions.recipes[index];
    let root = &actions.stage_roots[index];
    let measure = |path: &Path| -> Result<MantleMeasuredFile, RunError> {
        let relative = path.strip_prefix(root).map_err(|_| failure("Mantle log escaped stage root".into()))?;
        let path = relative.to_str().ok_or_else(|| failure("non-UTF8 Mantle artifact path".into()))?.to_string();
        source_built_recipe::relative(&path, false)?;
        Ok(MantleMeasuredFile {
            path,
            digest_blake3: source_built_recipe::digest(&root.join(relative))?.1,
        })
    };
    let mut measured_commands = Vec::with_capacity(commands.len());
    let aggregate_path = root.join("all-stderr");
    let mut aggregate = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&aggregate_path)
        .map_err(|error| failure(error.to_string()))?;
    for command in commands {
        let stdout = measure(&command.stdout_path)?;
        let stderr = measure(&command.stderr_path)?;
        if stdout.digest_blake3 != command.stdout_digest_blake3 || stderr.digest_blake3 != command.stderr_digest_blake3
        {
            return Err(failure("completed command log changed before publication".into()));
        }
        let mut file = std::fs::File::open(&command.stderr_path).map_err(|error| failure(error.to_string()))?;
        std::io::copy(&mut file, &mut aggregate).map_err(|error| failure(error.to_string()))?;
        measured_commands.push(MantleCommandCompletion {
            action_id: command.action_id.clone(),
            exit_code: command.exit_code,
            stdout,
            stderr,
        });
    }
    aggregate.flush().and_then(|()| aggregate.sync_all()).map_err(|error| failure(error.to_string()))?;
    let receipts = MantleCommandReceipts {
        schema: MANTLE_COMMAND_RECEIPTS_SCHEMA.into(),
        commands: measured_commands,
    };
    let receipt_path = root.join("command-completions.json");
    crate::source_built_fixed_point_shell::write_json_create_new(&receipt_path, &receipts)?;
    let binary = prepared.export(&prepared.recipe.stage_id)?;
    let ArtifactRef::Produced { action_id, output_id } = &binary.reference else {
        return Err(failure("Mantle output lacks producer".into()));
    };
    let measured_binary = measure(&binary.path)?;
    let smoke = prepared.export("mantle-smoke")?;
    let ArtifactRef::Produced {
        action_id: smoke_action,
        ..
    } = &smoke.reference
    else {
        return Err(failure("smoke lacks producer".into()));
    };
    let smoke = receipts
        .commands
        .iter()
        .find(|command| &command.action_id == smoke_action)
        .ok_or_else(|| failure("smoke lacks an actual observed command completion".into()))?;
    let orchestrator = &actions.adapters[index].actions[0];
    let manifest = MantleStageManifest {
        schema: MANTLE_STAGE_MANIFEST_SCHEMA.into(),
        stage_id: prepared.recipe.stage_id.clone(),
        proof_plan_digest_blake3: prepared.proof.plan_digest_blake3.clone(),
        action_plan_digest_blake3: action_plan_digest.into(),
        closure_policy_digest_blake3: prepared.proof.policies.closure_policy_digest_blake3.clone(),
        binary: MantleStageBinary {
            path: measured_binary.path,
            digest_blake3: measured_binary.digest_blake3,
            producer_action_id: action_id.clone(),
            output_id: output_id.clone(),
        },
        orchestrator: MantleStageOrchestrator {
            action_id: orchestrator.id.clone(),
            executable_digest_blake3: orchestrator_digest.into(),
            executable_producer: match &orchestrator.executable {
                ActionExecutable::Produced { action_id, output_id } => Some((action_id.clone(), output_id.clone())),
                _ => None,
            },
        },
        smoke: MantleStageSmoke {
            action_id: smoke.action_id.clone(),
            exit_code: smoke.exit_code,
            stdout: smoke.stdout.clone(),
            stderr: smoke.stderr.clone(),
        },
        command_receipt: measure(&receipt_path)?,
        stderr: measure(&aggregate_path)?,
        semantic_unit_count: actions.semantic_unit_counts[index],
    };
    crate::source_built_fixed_point_shell::write_json_create_new(&root.join("stage-manifest.json"), &manifest)
}

/// Stops at the kernel's successful exec transition, before the new program can
/// run. The observer must durably accept the measured executable before detach.
/// Unlike an inherited seccomp filter, this one-transition observation does not
/// interfere with the replay child's independently protected leaf supervisors.
#[cfg(target_os = "linux")]
fn launch_orchestrator(
    command: &mut Command,
    expected_digest: &str,
    observe: &mut dyn FnMut(&str) -> Result<(), RunError>,
) -> Result<Child, RunError> {
    use std::os::unix::process::CommandExt;
    unsafe {
        command.pre_exec(|| {
            if libc::ptrace(
                libc::PTRACE_TRACEME,
                0,
                std::ptr::null_mut::<libc::c_void>(),
                std::ptr::null_mut::<libc::c_void>(),
            ) == -1
            {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let mut child = command.spawn().map_err(|error| failure(format!("launching replay orchestrator: {error}")))?;
    let admission = (|| {
        let pid = libc::pid_t::try_from(child.id()).map_err(|_| failure("orchestrator PID exceeds pid_t".into()))?;
        let mut status = 0;
        loop {
            let result = unsafe { libc::waitpid(pid, &mut status, 0) };
            if result == pid {
                break;
            }
            let error = std::io::Error::last_os_error();
            if result < 0 && error.kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            return Err(failure(format!("waiting for orchestrator exec stop: {error}")));
        }
        if !libc::WIFSTOPPED(status) || libc::WSTOPSIG(status) != libc::SIGTRAP {
            return Err(failure(format!("orchestrator did not stop at successful exec: wait status {status}")));
        }
        let executable = std::path::PathBuf::from(format!("/proc/{pid}/exe"));
        let digest = file_digest(&executable)?;
        // Preserve the real unexpected identity too; disagreement must never
        // discard a launch record or turn into a successful fallback.
        observe(&digest)?;
        if digest != expected_digest {
            return Err(failure("orchestrator executed a different executable identity".into()));
        }
        if unsafe {
            libc::ptrace(
                libc::PTRACE_DETACH,
                pid,
                std::ptr::null_mut::<libc::c_void>(),
                std::ptr::null_mut::<libc::c_void>(),
            )
        } == -1
        {
            return Err(failure(format!(
                "detaching admitted replay orchestrator: {}",
                std::io::Error::last_os_error()
            )));
        }
        Ok(())
    })();
    if let Err(error) = admission {
        let _ = child.kill();
        let _ = child.wait();
        return Err(error);
    }
    Ok(child)
}

#[cfg(not(target_os = "linux"))]
fn launch_orchestrator(
    _command: &mut Command,
    _expected_digest: &str,
    _observe: &mut dyn FnMut(&str) -> Result<(), RunError>,
) -> Result<Child, RunError> {
    Err(failure("protected source-built Mantle replay requires Linux exec-stop observation".into()))
}

fn file_digest(path: &std::path::Path) -> Result<String, RunError> {
    let mut file =
        std::fs::File::open(path).map_err(|error| failure(format!("opening {}: {error}", path.display())))?;
    if !file
        .metadata()
        .map_err(|error| failure(format!("inspecting {}: {error}", path.display())))?
        .is_file()
    {
        return Err(failure(format!("measured artifact is not a regular file: {}", path.display())));
    }
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count =
            file.read(&mut buffer).map_err(|error| failure(format!("measuring {}: {error}", path.display())))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(hasher.finalize().to_hex().to_string())
}

fn failure(message: String) -> RunError {
    RunError::Build(format!("source-built Mantle actions: {message}"))
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use std::process::Stdio;

    use super::*;
    #[test]
    fn replay_continuation_rejects_parent_refusal_and_missing_acknowledgement() {
        use std::io::Write;
        use std::os::unix::net::UnixStream;

        for acknowledgement in [Some(0u8), None] {
            let (mut child, mut parent) = UnixStream::pair().unwrap();
            let timeout = Some(std::time::Duration::from_secs(5));
            child.set_read_timeout(timeout).unwrap();
            parent.set_read_timeout(timeout).unwrap();
            let receiver = std::thread::spawn(move || {
                let mut length = [0; 4];
                parent.read_exact(&mut length).unwrap();
                let mut frame = vec![0; u32::from_be_bytes(length) as usize];
                parent.read_exact(&mut frame).unwrap();
                if let Some(acknowledgement) = acknowledgement {
                    parent.write_all(&[acknowledgement]).unwrap();
                }
            });
            let result = send_message(
                &mut child,
                &ReplayMessage::Completed {
                    completions: Vec::new(),
                },
                &replay_transport_plan().unwrap(),
            );
            receiver.join().unwrap();
            assert!(matches!(result, Err(RunError::Build(_))));
        }
    }

    fn fixture_executable(name: &str) -> PathBuf {
        assert!(!name.is_empty(), "test executable name must not be empty");
        let path = std::env::var_os("PATH").expect("orchestrator tests require PATH");
        for directory in std::env::split_paths(&path) {
            let candidate = directory.join(name);
            if candidate.is_file() {
                // Preserve the basename so a Nix coreutils multi-call alias
                // still selects the requested program after exec.
                let executable = std::path::absolute(candidate).unwrap();
                assert!(executable.is_absolute());
                return executable;
            }
        }
        panic!("test executable is absent: {name}");
    }

    #[test]
    fn protected_orchestrator_records_actual_exec_before_program_output() {
        let executable = fixture_executable("echo");
        let expected = file_digest(&executable).unwrap();
        let mut command = Command::new(&executable);
        command.arg("observed-exec").stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut observed = Vec::new();
        let child = launch_orchestrator(&mut command, &expected, &mut |digest| {
            observed.push(digest.to_owned());
            Ok(())
        })
        .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success());
        assert_eq!(output.stdout, b"observed-exec\n");
        assert_eq!(observed, [expected]);
    }

    #[test]
    fn protected_orchestrator_preserves_identity_before_rejecting_drift() {
        let executable = fixture_executable("echo");
        let actual = file_digest(&executable).unwrap();
        let mut command = Command::new(&executable);
        command.arg("must-not-run").stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut observed = Vec::new();
        let result = launch_orchestrator(&mut command, &"0".repeat(64), &mut |digest| {
            observed.push(digest.to_owned());
            Ok(())
        });
        assert!(result.err().unwrap().to_string().contains("different executable identity"));
        assert_eq!(observed, [actual]);
    }
}
