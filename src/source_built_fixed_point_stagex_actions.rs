//! Source-selected StageX programs. Lineage is an independent topology input,
//! not an execution transcript. Admission checks every original node and edge;
//! it does not infer a leaf expansion from legacy reports or shell observations.
use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde::Deserialize;

use crate::errors::RunError;
use crate::source_built_fixed_point::SourceAuthorityRole;
use crate::source_built_fixed_point::SourceBuiltFixedPointPlan;
use crate::source_built_fixed_point_action_trust::ActionExecutionEvent;
use crate::source_built_fixed_point_action_trust::StageActionAdapter;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::PreparedRecipe;
use crate::source_built_recipe::RecipeOperation;
use crate::source_built_recipe::RecipePaths;
use crate::source_built_recipe::RecipeRun;
use crate::source_built_recipe::RecipeStep;
use crate::source_built_recipe::SourceRecipe;
use crate::source_built_recipe::WordPart;
use crate::source_built_recipe::{self as recipe};

const TRANSITION: &str = "stagex-transition";
const PUBLICATION: &str = "stagex-provider-publication";
const INDEX_SCHEMA: &str = "mantle-stagex-source-expansion-v1";
const TOOL_EXPORTS: [&str; 14] = [
    "tcc",
    "tcc-selfhost",
    "tcc-0.9.26",
    "as",
    "ld",
    "ar",
    "ranlib",
    "nm",
    "objcopy",
    "objdump",
    "readelf",
    "size",
    "strings",
    "strip",
];

/// Source-authored ownership of concrete leaf commands, not semantic labels.
/// Outputs retain original upstream identities so edges can be checked against
/// the independently admitted lineage/route rather than this index's claims.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ExpandedNode {
    pub(crate) step_ids: BTreeSet<String>,
    pub(crate) outputs: BTreeMap<String, ArtifactRef>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TransitionIndex {
    schema: String,
    recipe_member: String,
    sources: BTreeMap<String, ArtifactRef>,
    patches: BTreeMap<String, ArtifactRef>,
    tools: BTreeMap<String, ExpandedNode>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PublicationIndex {
    schema: String,
    recipe_member: String,
    /// The fixed provider launch inventory, mapped to concrete local steps.
    validation_commands: BTreeMap<String, String>,
}

pub(crate) struct PreparedStagexActions {
    transition: PreparedRecipe,
    publication: PreparedRecipe,
}

impl PreparedStagexActions {
    pub(crate) fn transition_recipe(&self) -> &PreparedRecipe {
        &self.transition
    }
    pub(crate) fn publication_recipe(&self) -> &PreparedRecipe {
        &self.publication
    }
    pub(crate) fn transition_adapter(&self) -> &StageActionAdapter {
        self.transition.adapter()
    }
    pub(crate) fn publication_adapter(&self) -> &StageActionAdapter {
        self.publication.adapter()
    }
}

fn error(message: impl Into<String>) -> RunError {
    RunError::Build(format!("StageX source actions: {}", message.into()))
}

/// Both selectors must name source programs in the admitted StageX bundle.
/// Each has a sibling `<member>.expansion.json` source product. These inputs
/// are currently absent from the legacy transition route: neither its report,
/// plan, nor copied provider binaries can substitute for either source program.
pub(crate) fn prepare_actions(
    proof: &SourceBuiltFixedPointPlan,
    transition_ref: &ArtifactRef,
    publication_ref: &ArtifactRef,
    paths: &[RecipePaths<'_>; 2],
    prior: &[&PreparedRecipe],
    host_digest: &str,
) -> Result<PreparedStagexActions, RunError> {
    if paths[0].runtime_root == paths[1].runtime_root {
        return Err(error("transition and publication require distinct runtime roots"));
    }
    let (transition_member, transition_bytes) = index_source(proof, transition_ref, &paths[0])?;
    let (publication_member, publication_bytes) = index_source(proof, publication_ref, &paths[1])?;
    let transition_index: TransitionIndex = serde_json::from_slice(&transition_bytes)
        .map_err(|failure| error(format!("transition expansion source syntax: {failure}")))?;
    let publication_index: PublicationIndex = serde_json::from_slice(&publication_bytes)
        .map_err(|failure| error(format!("publication expansion source syntax: {failure}")))?;
    if transition_index.schema != INDEX_SCHEMA
        || transition_index.recipe_member != transition_member
        || publication_index.schema != INDEX_SCHEMA
        || publication_index.recipe_member != publication_member
    {
        return Err(error("source expansion index selects another schema/program"));
    }
    let lineage = read_lineage(proof, &paths[0])?;
    let transition = recipe::prepare_from_reference(proof, TRANSITION, transition_ref, &paths[0], prior, host_digest)?;
    validate_transition(&lineage, &transition_index, &transition)?;
    require_export(&transition, TRANSITION, ArtifactKind::Tree, Some(TRANSITION))?;
    let mut publication_prior = prior.to_vec();
    publication_prior.push(&transition);
    let publication = recipe::prepare_from_reference(
        proof,
        PUBLICATION,
        publication_ref,
        &paths[1],
        &publication_prior,
        host_digest,
    )?;
    validate_publication(&publication_index, &transition, &publication)?;
    Ok(PreparedStagexActions {
        transition,
        publication,
    })
}

pub(crate) fn run_transition(
    actions: &PreparedStagexActions,
    emit: &mut dyn FnMut(ActionExecutionEvent) -> Result<(), RunError>,
) -> Result<RecipeRun, RunError> {
    recipe::run(&actions.transition, emit)
}

pub(crate) fn run_publication(
    actions: &PreparedStagexActions,
    emit: &mut dyn FnMut(ActionExecutionEvent) -> Result<(), RunError>,
) -> Result<RecipeRun, RunError> {
    recipe::run(&actions.publication, emit)
}

fn index_source(
    proof: &SourceBuiltFixedPointPlan,
    reference: &ArtifactRef,
    paths: &RecipePaths<'_>,
) -> Result<(String, Vec<u8>), RunError> {
    let ArtifactRef::ManifestSource {
        role: SourceAuthorityRole::StagexSourceBundle,
        record_identity,
        member,
    } = reference
    else {
        return Err(error("source program selector must be ManifestSource StagexSourceBundle"));
    };
    validate_program_record(proof, paths, record_identity)?;
    let index_member = format!("{member}.expansion.json");
    let bytes = recipe::read_manifest_source_member(proof, SourceAuthorityRole::StagexSourceBundle, record_identity, &index_member, paths)
        .map_err(|failure| error(format!("missing authenticated leaf source-program input {record_identity}:{index_member}; retain all original generators as individually produced tools, not transition reports or imported binaries: {failure}")))?;
    Ok((member.clone(), bytes))
}

fn validate_program_record(
    proof: &SourceBuiltFixedPointPlan,
    paths: &RecipePaths<'_>,
    identity: &str,
) -> Result<(), RunError> {
    let manifest_path = paths
        .source_roots
        .get(&SourceAuthorityRole::StagexSourceBundle)
        .ok_or_else(|| error("missing StageX source bundle path"))?;
    let authority = proof
        .source_inputs
        .iter()
        .find(|input| input.role == SourceAuthorityRole::StagexSourceBundle)
        .ok_or_else(|| error("missing StageX source bundle authority"))?;
    if recipe::digest(manifest_path)?.1 != authority.digest_blake3 {
        return Err(error("StageX source bundle identity drift"));
    }
    let manifest = crate::source_bundle::read_source_bundle(manifest_path)?;
    let mut records = manifest.records.iter().filter(|record| record.identity == identity);
    let record = records.next().ok_or_else(|| error(format!("missing source program record {identity}")))?;
    if records.next().is_some()
        || !crate::source_bundle::source_record_is_fetcher_input(record)
        || record.files.is_empty()
        || record.payload_bytes > 64 * 1024 * 1024
        || record
            .files
            .iter()
            .any(|file| file.file_type != crate::source_bundle::SourceFileType::Regular || file.executable)
    {
        return Err(error("StageX source program record must contain bounded inert fetch-source text only"));
    }
    let temporary = tempfile::tempdir().map_err(|failure| error(failure.to_string()))?;
    let destination = temporary.path().join("program");
    crate::source_bundle::materialize_source_record_exact_payload(record, &destination)?;
    for file in &record.files {
        recipe::relative(&file.path, false)?;
        let bytes = std::fs::read(destination.join(&file.path)).map_err(|failure| error(failure.to_string()))?;
        let text = std::str::from_utf8(&bytes).map_err(|_| error("binary payload in StageX source-program record"))?;
        recipe::validate_text(text)?;
    }
    Ok(())
}

fn read_lineage(
    proof: &SourceBuiltFixedPointPlan,
    paths: &RecipePaths<'_>,
) -> Result<crunch_bootstrap_core::LineageManifest, RunError> {
    let path = paths
        .source_roots
        .get(&SourceAuthorityRole::StagexLineage)
        .ok_or_else(|| error("missing StageX lineage source path"))?;
    let authority = proof
        .source_inputs
        .iter()
        .find(|input| input.role == SourceAuthorityRole::StagexLineage)
        .ok_or_else(|| error("missing StageX lineage authority"))?;
    if recipe::digest(path)?.1 != authority.digest_blake3 {
        return Err(error("lineage source differs from authenticated proof input"));
    }
    let metadata = std::fs::symlink_metadata(path).map_err(|failure| error(failure.to_string()))?;
    if !metadata.is_file() || metadata.len() > 16 * 1024 * 1024 {
        return Err(error("lineage must be bounded regular source data"));
    }
    let bytes = std::fs::read(path).map_err(|failure| error(failure.to_string()))?;
    let lineage: crunch_bootstrap_core::LineageManifest =
        serde_json::from_slice(&bytes).map_err(|failure| error(format!("lineage source syntax: {failure}")))?;
    crunch_bootstrap_core::admit_lineage_manifest(&lineage).map_err(|failure| error(failure.to_string()))?;
    if !crunch_bootstrap_core::validate_provider_boundary(&lineage, &[]).is_valid() {
        return Err(error("original lineage lacks the complete normalized provider role inventory"));
    }
    let seed = proof
        .source_inputs
        .iter()
        .find(|input| input.role == SourceAuthorityRole::StagexSeed)
        .ok_or_else(|| error("missing seed authority"))?;
    if lineage.seed.seed_digest.algorithm != "blake3" || lineage.seed.seed_digest.hex_value != seed.digest_blake3 {
        return Err(error("lineage seed differs from admitted seed"));
    }
    Ok(lineage)
}

fn validate_transition(
    lineage: &crunch_bootstrap_core::LineageManifest,
    index: &TransitionIndex,
    prepared: &PreparedRecipe,
) -> Result<(), RunError> {
    let program = prepared.source_recipe();
    validate_partition(program, index.tools.values())?;
    if index.tools.keys().ne(lineage
        .transition_tools
        .iter()
        .map(|tool| tool.id.clone())
        .collect::<BTreeSet<_>>()
        .iter())
        || index.sources.keys().ne(lineage
            .source_artifacts
            .iter()
            .map(|source| source.id.clone())
            .collect::<BTreeSet<_>>()
            .iter())
        || index
            .patches
            .keys()
            .ne(lineage.patches.iter().map(|patch| patch.id.clone()).collect::<BTreeSet<_>>().iter())
    {
        return Err(error("expansion omits or invents original lineage tools, sources, or patches"));
    }
    let mut artifacts = index.sources.clone();
    artifacts.extend(index.patches.clone());
    artifacts.insert("seed".to_string(), ArtifactRef::Source {
        role: SourceAuthorityRole::StagexSeed,
        path: String::new(),
    });
    for source in &lineage.source_artifacts {
        require_source_digest(prepared, &index.sources[&source.id], &source.digest)?;
    }
    for patch in &lineage.patches {
        require_source_digest(prepared, &index.patches[&patch.id], &patch.digest)?;
    }
    for tool in &lineage.transition_tools {
        let expanded = &index.tools[&tool.id];
        if expanded.outputs.keys().ne(tool.output_artifact_ids.iter().cloned().collect::<BTreeSet<_>>().iter()) {
            return Err(error(format!("original output inventory changed for {}", tool.id)));
        }
        for (id, reference) in &expanded.outputs {
            if artifacts.insert(id.clone(), reference.clone()).is_some() {
                return Err(error("ambiguous original artifact identity"));
            }
        }
    }
    for generated in &lineage.generated_artifacts {
        if index
            .tools
            .get(&generated.producing_tool_id)
            .and_then(|node| node.outputs.get(&generated.id))
            .is_none()
        {
            return Err(error(format!("missing original generated artifact {}", generated.id)));
        }
    }
    for tool in &lineage.transition_tools {
        let node = &index.tools[&tool.id];
        for id in tool.source_artifact_ids.iter().chain(&tool.input_artifact_ids) {
            let input = artifacts.get(id).ok_or_else(|| error(format!("unknown original input {id}")))?;
            require_node_input(program, node, input)?;
        }
    }
    // Preserve graph edges independently of the tool input lists. Tool nodes
    // expand to their concrete outputs, never success labels or report digests.
    for edge in &lineage.stage_graph {
        // Edges into authenticated source/patch nodes describe admission, not
        // production by the seed. Do not invent an execution dependency for
        // source bytes that pre-exist the transition.
        if index.sources.contains_key(&edge.to_node_id) || index.patches.contains_key(&edge.to_node_id) {
            continue;
        }
        let from = node_references(&edge.from_node_id, &artifacts, &index.tools)?;
        let to = node_references(&edge.to_node_id, &artifacts, &index.tools)?;
        for input in from {
            if !to.iter().any(|output| depends_on(program, output, input)) {
                return Err(error(format!(
                    "expanded program drops lineage edge {} -> {}",
                    edge.from_node_id, edge.to_node_id
                )));
            }
        }
    }
    for output in &lineage.provider_outputs {
        let reference = artifacts
            .get(&output.producing_artifact_id)
            .ok_or_else(|| error("provider output lacks original producer"))?;
        if !depends_on(program, &prepared.export(TRANSITION)?.reference, reference) {
            return Err(error("canonical transition tree omits an original provider product"));
        }
    }
    Ok(())
}

fn node_references<'a>(
    id: &str,
    artifacts: &'a BTreeMap<String, ArtifactRef>,
    tools: &'a BTreeMap<String, ExpandedNode>,
) -> Result<Vec<&'a ArtifactRef>, RunError> {
    if let Some(reference) = artifacts.get(id) {
        return Ok(vec![reference]);
    }
    if let Some(node) = tools.get(id) {
        return Ok(node.outputs.values().collect());
    }
    Err(error(format!("unmapped original lineage node {id}")))
}

fn require_source_digest(
    prepared: &PreparedRecipe,
    reference: &ArtifactRef,
    digest: &crunch_bootstrap_core::DigestEntry,
) -> Result<(), RunError> {
    if matches!(reference, ArtifactRef::Produced { .. }) || digest.algorithm != "blake3" {
        return Err(error("lineage source/patch must retain authenticated source BLAKE3 identity"));
    }
    let binding = prepared.bindings.get(reference).ok_or_else(|| error("unused/unbound original source input"))?;
    // Original archive lineage pins the canonical source RECORD content, not
    // the different runtime tree hash after authenticated archive hydration.
    if let ArtifactRef::ManifestSource {
        role,
        record_identity,
        member,
    } = reference
        && member.is_empty()
    {
        let manifest_path = prepared.sources.get(role).ok_or_else(|| error("unbound original source manifest"))?;
        let manifest = crate::source_bundle::read_source_bundle(manifest_path)?;
        let mut records = manifest.records.iter().filter(|record| record.identity == *record_identity);
        let record = records.next().ok_or_else(|| error("original source record missing"))?;
        if records.next().is_some()
            || !crate::source_bundle::source_record_is_fetcher_input(record)
            || record.content_blake3 != digest.hex_value
        {
            return Err(error("original source record identity drift"));
        }
        return Ok(());
    }
    if recipe::digest(&binding.path)?.1 != digest.hex_value {
        return Err(error("original source/patch identity drift"));
    }
    Ok(())
}

fn validate_publication(
    index: &PublicationIndex,
    transition: &PreparedRecipe,
    prepared: &PreparedRecipe,
) -> Result<(), RunError> {
    require_export(prepared, "stagex-provider", ArtifactKind::Tree, Some("stagex-provider"))?;
    for role in TOOL_EXPORTS {
        require_export(prepared, role, ArtifactKind::Executable, None)?;
    }
    for role in ["musl.headers", "musl.libraries"] {
        require_export(prepared, role, ArtifactKind::Tree, None)?;
    }
    let launches = crate::stagex_provider::provider_validation_launches();
    let expected = launches.iter().map(|(id, _)| id.to_string()).collect::<BTreeSet<_>>();
    if index.validation_commands.keys().ne(expected.iter()) {
        return Err(error("publication omits original runtime supplement/positive/negative validation commands"));
    }
    let mut commands = BTreeSet::new();
    for (id, executable_path) in launches {
        let step_id = &index.validation_commands[id];
        if !commands.insert(step_id) {
            return Err(error("distinct publication launches share one leaf command"));
        }
        let step = prepared
            .source_recipe()
            .steps
            .iter()
            .find(|step| step.id == *step_id)
            .ok_or_else(|| error("unknown publication validation command"))?;
        let RecipeOperation::Exec {
            executable_input,
            exit_policy,
            ..
        } = &step.operation
        else {
            return Err(error("publication validation must execute an individual tool"));
        };
        let recipe::RecipeExitPolicy::Exact {
            code: expected_exit_code,
        } = exit_policy
        else {
            return Err(error("publication validation requires one exact original exit status"));
        };
        let expected = match id {
            "tcc-execution" | "positive-execution" => Some(42),
            "tcc-negative" | "as-negative" => None,
            _ => Some(0),
        };
        if expected.is_some_and(|code| *expected_exit_code != code) || (expected.is_none() && *expected_exit_code == 0)
        {
            return Err(error("original positive/negative provider validation status contract changed"));
        }
        let role = match executable_path {
            "bin/x86_64-linux-musl-tcc" => "tcc",
            "bin/x86_64-linux-musl-as" => "as",
            "bin/x86_64-linux-musl-ld" => "ld",
            ".validation/tcc-positive" | ".validation/binutils-positive" => {
                let executable = &step.inputs[executable_input];
                let compiler_command = if id == "tcc-execution" {
                    "tcc-positive"
                } else {
                    "ld-positive"
                };
                let producer = &index.validation_commands[compiler_command];
                if !matches!(executable, ArtifactRef::Produced { action_id, .. } if action_id == &format!("{PUBLICATION}/{producer}"))
                {
                    return Err(error("provider smoke does not execute its actual positive-build output"));
                }
                continue;
            }
            _ => return Err(error(format!("unrecognized upstream publication executable {executable_path}"))),
        };
        // Runtime-smoke executables are local generated leaves; provider tools
        // retain the exact produced executable export, not a tree subpath.
        if step.inputs.get(executable_input) != Some(&prepared.export(role)?.reference) {
            return Err(error("publication command selects a different produced tool"));
        }
    }
    let product = &prepared.export("stagex-provider")?.reference;
    if !depends_on(prepared.source_recipe(), product, &transition.export(TRANSITION)?.reference) {
        return Err(error("publication canonical tree drops its actual transition producer"));
    }
    for role in TOOL_EXPORTS {
        if !depends_on(prepared.source_recipe(), product, &prepared.export(role)?.reference) {
            return Err(error(format!("provider tree omits individual tool {role}")));
        }
    }
    Ok(())
}

pub(crate) fn require_export(
    prepared: &PreparedRecipe,
    role: &str,
    kind: ArtifactKind,
    canonical: Option<&str>,
) -> Result<(), RunError> {
    let binding = prepared.export(role)?;
    let ArtifactRef::Produced { action_id, output_id } = &binding.reference else {
        return Err(error("export must be individually produced"));
    };
    if binding.kind != kind
        || !action_id.starts_with(&format!("{}/", prepared.adapter().stage_id))
        || canonical.is_some_and(|id| output_id != id)
    {
        return Err(error(format!("invalid source-produced export {role}")));
    }
    Ok(())
}

/// Check actual operand references, not merely declared-but-unused input labels.
pub(crate) fn used_inputs(step: &RecipeStep) -> BTreeSet<&ArtifactRef> {
    let mut names = BTreeSet::<&str>::new();
    match &step.operation {
        RecipeOperation::Controller { specification, guard } => {
            use crate::source_built_stagex_programs::controller::ControllerExecutable;

            // The compact catalog is consumed by the shared controller
            // templates: source roots, command roots and exact image selectors.
            // Unrelated step input labels are not dependency authority.
            let mut references = BTreeSet::from([&specification.controller]);
            references.extend(specification.inputs.values());
            for slot in &specification.slots {
                references.extend(slot.inputs.values());
                match &slot.executable {
                    ControllerExecutable::Fixed(image) => {
                        references.insert(image);
                    }
                    ControllerExecutable::Adapted {
                        executable,
                        original_image,
                    } => {
                        references.insert(executable);
                        references.insert(original_image);
                    }
                    ControllerExecutable::Inventory(images) => {
                        references.extend(images.iter().map(|(_, image)| image));
                    }
                    // Generated images depend on the controller's own earlier
                    // producer slots, not a new external executable grant.
                    ControllerExecutable::RequestOutput { .. } => {}
                }
            }
            if let Some(guard) = guard {
                references
                    .extend(guard.conditions.iter().filter_map(|condition| step.inputs.get(&condition.status_input)));
            }
            return references;
        }
        RecipeOperation::Exec {
            executable_input,
            arguments,
            environment,
            runtime_mounts,
            cwd_input,
            stdin_input,
            guard,
            output_seeds,
            invocation_input,
            ..
        } => {
            names.insert(executable_input);
            names.extend(cwd_input.iter().map(String::as_str));
            names.extend(stdin_input.iter().map(String::as_str));
            names.extend(runtime_mounts.values().map(String::as_str));
            names.extend(output_seeds.values().map(String::as_str));
            names.extend(invocation_input.iter().map(String::as_str));
            if invocation_input.is_some() {
                // Actual words do not exist at source admission. The executor
                // authenticates the produced invocation and requires every
                // declared input name to be consumed before launching a leaf.
                names.extend(step.inputs.keys().map(String::as_str));
            }
            if let Some(guard) = guard {
                names.extend(guard.conditions.iter().map(|condition| condition.status_input.as_str()));
            }
            for word in arguments.iter().chain(environment.values()) {
                for part in &word.parts {
                    match part {
                        WordPart::Input { name, .. }
                        | WordPart::TextInput { name }
                        | WordPart::TextSlice { name, .. } => {
                            names.insert(name);
                        }
                        WordPart::Runtime { mount, .. } => {
                            if let Some(name) = runtime_mounts.get(mount) {
                                names.insert(name);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        RecipeOperation::CopyText { input, .. } | RecipeOperation::ExtractArchive { input, .. } => {
            names.insert(input);
        }
        RecipeOperation::AssembleTree { entries, .. } => {
            names.extend(entries.values().map(String::as_str));
        }
        RecipeOperation::MaterializeFetcher {
            role, record_identity, ..
        } => {
            // Hydration consumes the exact manifest root it authenticates and
            // the selected record root, never arbitrary labels or submembers.
            names.extend(step.inputs.iter().filter_map(|(name, reference)| match reference {
                ArtifactRef::ManifestSource {
                    role: input_role,
                    record_identity: input_identity,
                    member,
                } if input_role == role && input_identity == record_identity && member.is_empty() => {
                    Some(name.as_str())
                }
                ArtifactRef::Source { role: input_role, path } if input_role == role && path.is_empty() => {
                    Some(name.as_str())
                }
                _ => None,
            }));
        }
        RecipeOperation::ObservePhase { state, .. } => {
            if let Some(state) = state {
                names.insert(&state.skipped_input);
            }
        }
        RecipeOperation::WriteText { .. } | RecipeOperation::MaterializeRecord { .. } => {}
    }
    names.into_iter().filter_map(|name| step.inputs.get(name)).collect()
}

pub(crate) fn depends_on(program: &SourceRecipe, output: &ArtifactRef, input: &ArtifactRef) -> bool {
    if output == input {
        return true;
    }
    let producer = |reference: &ArtifactRef| {
        let ArtifactRef::Produced { action_id, .. } = reference else {
            return None;
        };
        program.steps.iter().find(|step| action_id == &format!("{}/{}", program.stage_id, step.id))
    };
    let mut pending = producer(output).into_iter().collect::<Vec<_>>();
    let mut seen = BTreeSet::new();
    while let Some(step) = pending.pop() {
        if !seen.insert(step.id.as_str()) {
            continue;
        }
        for reference in used_inputs(step) {
            if reference == input {
                return true;
            }
            pending.extend(producer(reference));
        }
        if let RecipeOperation::ObservePhase { phase, state, .. } = &step.operation
            && let Some(observed) = program.steps.iter().find(|candidate| candidate.id == *phase)
        {
            // The typed outcome depends on the actual phase, not on an
            // invented output that would be absent when the phase is skipped.
            if let Some(state) = state
                && let ArtifactRef::Produced { action_id, output_id } = input
                && action_id == &format!("{}/{}", program.stage_id, phase)
                && output_id == &state.output
                && observed.outputs.contains_key(output_id)
            {
                return true;
            }
            pending.push(observed);
        }
    }
    false
}

pub(crate) fn require_node_input(
    program: &SourceRecipe,
    node: &ExpandedNode,
    input: &ArtifactRef,
) -> Result<(), RunError> {
    if !program
        .steps
        .iter()
        .filter(|step| node.step_ids.contains(&step.id))
        .any(|step| used_inputs(step).contains(input))
    {
        return Err(error("expanded commands omit an original source/bootstrap dependency"));
    }
    Ok(())
}

pub(crate) fn validate_partition<'a>(
    program: &SourceRecipe,
    nodes: impl Iterator<Item = &'a ExpandedNode>,
) -> Result<(), RunError> {
    let steps = program.steps.iter().map(|step| (step.id.as_str(), step)).collect::<BTreeMap<_, _>>();
    let mut assigned = BTreeSet::new();
    for node in nodes {
        if node.step_ids.is_empty() || node.outputs.is_empty() {
            return Err(error("missing concrete original-node expansion"));
        }
        for id in &node.step_ids {
            if !steps.contains_key(id.as_str()) || !assigned.insert(id.as_str()) {
                return Err(error("unknown or multiply assigned leaf step"));
            }
        }
        if !node.step_ids.iter().any(|id| matches!(steps[id.as_str()].operation, RecipeOperation::Exec { .. })) {
            return Err(error("original build/generator node lacks an actual command"));
        }
        for reference in node.outputs.values() {
            let ArtifactRef::Produced { action_id, output_id } = reference else {
                return Err(error("original build output is imported source"));
            };
            let id = action_id
                .strip_prefix(&format!("{}/", program.stage_id))
                .ok_or_else(|| error("original output has another stage producer"))?;
            if !node.step_ids.contains(id) || !steps.get(id).is_some_and(|step| step.outputs.contains_key(output_id)) {
                return Err(error("original output lacks its exact owned leaf producer"));
            }
        }
    }
    if assigned.len() != steps.len() {
        return Err(error("unclassified source/generator/command leaf steps"));
    }
    for step in &program.steps {
        if used_inputs(step).len() != step.inputs.values().collect::<BTreeSet<_>>().len() {
            return Err(error("unused inputs cannot stand in for original command dependencies"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(path: &str) -> ArtifactRef {
        ArtifactRef::Source {
            role: SourceAuthorityRole::StagexSourceBundle,
            path: path.into(),
        }
    }

    fn product(step: &str, output: &str) -> ArtifactRef {
        ArtifactRef::Produced {
            action_id: format!("stagex-transition/{step}"),
            output_id: output.into(),
        }
    }

    fn phase() -> RecipeStep {
        RecipeStep {
            id: "phase".into(),
            inputs: BTreeMap::from([
                ("tool".into(), source("tool.c")),
                ("status".into(), product("probe", "exit-status")),
                ("previous".into(), product("prepare", "state")),
            ]),
            outputs: BTreeMap::from([("state".into(), recipe::RecipeOutput {
                kind: ArtifactKind::Tree,
                bytes_max: 4096,
            })]),
            operation: RecipeOperation::Exec {
                command_state: None,
                executable_input: "tool".into(),
                arguments: Vec::new(),
                environment: BTreeMap::new(),
                runtime_mounts: BTreeMap::new(),
                cwd_input: None,
                cwd_output: None,
                stdin_input: None,
                merge_stderr: false,
                invocation_input: None,
                exit_policy: recipe::RecipeExitPolicy::Exact { code: 0 },
                guard: Some(recipe::RecipeGuard {
                    conditions: vec![recipe::RecipeGuardCondition {
                        status_input: "status".into(),
                        accepted_codes: vec![0],
                    }],
                }),
                output_seeds: BTreeMap::from([("state".into(), "previous".into())]),
            },
        }
    }

    fn program(steps: Vec<RecipeStep>) -> SourceRecipe {
        SourceRecipe {
            schema: recipe::SCHEMA.into(),
            stage_id: "stagex-transition".into(),
            preparation: recipe::RecipePreparation {
                tool: "dependency-fixture".into(),
                provenance: "inert source graph".into(),
            },
            steps,
            roots: Vec::new(),
            exports: BTreeMap::new(),
            semantic_roots: BTreeMap::new(),
            timeout_groups: Vec::new(),
        }
    }

    #[test]
    fn phase_observation_retains_guard_seed_and_both_real_state_dependencies() {
        let phase = phase();
        let observer = RecipeStep {
            id: "observe".into(),
            inputs: BTreeMap::from([("previous".into(), product("prepare", "state"))]),
            outputs: BTreeMap::from([("state".into(), recipe::RecipeOutput {
                kind: ArtifactKind::Tree,
                bytes_max: 4096,
            })]),
            operation: RecipeOperation::ObservePhase {
                phase: "phase".into(),
                output: "outcome".into(),
                state: Some(recipe::RecipePhaseState {
                    output: "state".into(),
                    skipped_input: "previous".into(),
                    skipped_path: String::new(),
                }),
            },
        };
        let graph = program(vec![phase, observer]);
        let result = product("observe", "state");
        for dependency in [
            source("tool.c"),
            product("probe", "exit-status"),
            product("prepare", "state"),
            product("phase", "state"),
        ] {
            assert!(depends_on(&graph, &result, &dependency), "lost actual phase dependency {dependency:?}");
        }
        assert!(!depends_on(&graph, &result, &source("unrelated.c")));
        assert!(!depends_on(&graph, &result, &product("phase", "invented-output")));
    }

    #[test]
    fn unused_labels_cannot_satisfy_original_node_dependencies() {
        let mut phase = phase();
        let unused = source("unread.c");
        phase.inputs.insert("unused".into(), unused.clone());
        let graph = program(vec![phase]);
        let node = ExpandedNode {
            step_ids: BTreeSet::from(["phase".into()]),
            outputs: BTreeMap::from([("state".into(), product("phase", "state"))]),
        };
        let error = validate_partition(&graph, std::iter::once(&node)).unwrap_err();
        assert!(error.to_string().contains("unused inputs"));
        assert!(require_node_input(&graph, &node, &unused).unwrap_err().to_string().contains("omit"));
        assert!(!depends_on(&graph, &product("phase", "state"), &unused));
    }

    #[test]
    fn fetcher_hydration_consumes_only_exact_manifest_and_selected_record_roots() {
        let selected = ArtifactRef::ManifestSource {
            role: SourceAuthorityRole::StagexSourceBundle,
            record_identity: "original".into(),
            member: String::new(),
        };
        let wrong = ArtifactRef::ManifestSource {
            role: SourceAuthorityRole::StagexSourceBundle,
            record_identity: "other".into(),
            member: String::new(),
        };
        let step = RecipeStep {
            id: "hydrate".into(),
            inputs: BTreeMap::from([
                ("manifest".into(), source("")),
                ("source".into(), selected.clone()),
                ("wrong".into(), wrong.clone()),
                ("member".into(), source("subtree")),
            ]),
            outputs: BTreeMap::new(),
            operation: RecipeOperation::MaterializeFetcher {
                role: SourceAuthorityRole::StagexSourceBundle,
                record_identity: "original".into(),
                output: "tree".into(),
            },
        };
        let consumed = used_inputs(&step);
        assert!(consumed.contains(&selected));
        assert!(consumed.contains(&source("")));
        assert!(!consumed.contains(&wrong));
        assert!(!consumed.contains(&source("subtree")));
    }
}
