//! Native provider action admission over the complete evaluated derivation
//! closure and a source-authenticated, expanded command program. Conventional
//! build summaries and output directories never grant child-tool authority.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use crate::errors::RunError;
use crate::source_built_fixed_point::SourceAuthorityRole;
use crate::source_built_fixed_point::SourceBuiltFixedPointPlan;
use crate::source_built_fixed_point_action_trust::ActionExecutable;
use crate::source_built_fixed_point_action_trust::ActionExecutionEvent;
use crate::source_built_fixed_point_action_trust::ActionInput;
use crate::source_built_fixed_point_action_trust::ActionSpec;
use crate::source_built_fixed_point_action_trust::ExecutionEventKind;
use crate::source_built_fixed_point_action_trust::StageActionAdapter;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::PreparedRecipe;
use crate::source_built_recipe::RecipePaths;
use crate::source_built_recipe::RecipeRun;
use crate::source_built_recipe::SourceRecipe;
use crate::source_built_recipe::{self as recipe};

const STAGE: &str = "full-source-native-provider";
const INDEX_SCHEMA: &str = "mantle-native-source-expansion-v1";
const SOURCE_BYTES_MAX: u64 = 64 * 1024 * 1024;
const ROOT_RECIPES: [&str; 7] = [
    "bootstrap/seed-full-toolchain.ncl",
    "bootstrap/make-4.4.1-gcc10.ncl",
    "bootstrap/linux-headers-6.6-gcc10.ncl",
    "bootstrap/busybox-1.37.0-gcc10.ncl",
    "bootstrap/cmake-3.31.8-gcc10.ncl",
    "bootstrap/python-3.13.5-gcc10.ncl",
    "bootstrap/perl-5.10.1-gcc10.ncl",
];
const EXECUTABLE_EXPORTS: [&str; 16] = [
    "native.cc1",
    "native.cc1plus",
    "native.as",
    "native.ld",
    "native.ar",
    "native.nm",
    "native.ranlib",
    "native.objcopy",
    "native.strip",
    "native.readelf",
    "native.loader",
    "host.make",
    "host.cmake",
    "host.python",
    "host.perl",
    "host.busybox",
];
const TREE_EXPORTS: [&str; 3] = ["native.provider", "native.sysroot", "linux.headers"];
const RUNTIME_EXPORTS: [&str; 4] = ["native.libc", "native.libgcc", "native.libgcc-eh", "native.libstdcxx"];

/// Each original derivation must be expanded, including source materialization
/// nodes. These are source program identities, not observed success records.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NativeExpandedDerivation {
    pub(crate) hash_derivation_modulo: String,
    pub(crate) input_derivations: BTreeMap<String, BTreeSet<String>>,
    pub(crate) input_sources: BTreeSet<String>,
    /// Every expanded command/file-generator step belongs to exactly one node.
    pub(crate) step_ids: BTreeSet<String>,
    /// Exact final outputs of the node, never an executable directory prefix.
    pub(crate) outputs: BTreeMap<String, ArtifactRef>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NativeExpansionIndex {
    pub(crate) schema: String,
    pub(crate) recipe_member: String,
    pub(crate) roots: Vec<String>,
    pub(crate) derivations: BTreeMap<String, NativeExpandedDerivation>,
}

pub(crate) struct AdapterPaths<'a> {
    pub(crate) recipe_paths: &'a RecipePaths<'a>,
    pub(crate) record_identity: &'a str,
    pub(crate) recipe_member: &'a str,
    pub(crate) index_member: &'a str,
    pub(crate) stagex_recipes: &'a [&'a PreparedRecipe],
    pub(crate) publication_path: &'a Path,
    pub(crate) proof_root: &'a Path,
}

pub(crate) struct PreparedNativeActions {
    prepared: PreparedRecipe,
    adapter: StageActionAdapter,
    publication_action: ActionSpec,
    publication_path: PathBuf,
    proof_digest: String,
    expected_provider_digest: String,
    source_manifest_digest: String,
    proof_root: PathBuf,
}
impl PreparedNativeActions {
    pub(crate) fn adapter(&self) -> &StageActionAdapter {
        &self.adapter
    }
    pub(crate) fn recipe(&self) -> &PreparedRecipe {
        &self.prepared
    }
    pub(crate) fn export(&self, role: &str) -> Result<&recipe::ArtifactBinding, RunError> {
        self.prepared.export(role)
    }
}

fn error(message: impl Into<String>) -> RunError {
    RunError::Build(format!("native source actions: {}", message.into()))
}

pub(crate) fn closure(
    source_root: &Path,
    store_prefix: &str,
) -> Result<crunch_pipeline::native_preflight::NativeDerivationClosure, RunError> {
    let files = ROOT_RECIPES.map(|relative| source_root.join(relative));
    let imports: [OsString; 2] = [
        source_root.join("lib").into_os_string(),
        source_root.join("bootstrap").into_os_string(),
    ];
    for file in &files {
        recipe::validate_source_imports(source_root, file, &imports)?;
    }
    crunch_pipeline::native_preflight::plan_derivation_closure(&files, &imports, store_prefix)
        .map_err(|failure| error(failure.to_string()))
}

/// Returns executable authority only after validating the independently evaluated
/// seven-root topology against the authenticated expansion. Retain the returned
/// object: execution accepts it, not externally supplied adapters or events.
pub(crate) fn prepare_actions(
    proof: &SourceBuiltFixedPointPlan,
    source_root: &Path,
    paths: &AdapterPaths<'_>,
    host_mantle_digest_blake3: &str,
) -> Result<PreparedNativeActions, RunError> {
    let source = proof
        .source_inputs
        .iter()
        .find(|input| input.role == SourceAuthorityRole::MantleSource)
        .ok_or_else(|| error("missing admitted Mantle recipe source tree"))?;
    if recipe::digest(source_root)?.1 != source.digest_blake3 {
        return Err(error("native derivation recipe source tree differs from proof authority"));
    }
    let graph = closure(source_root, &proof.logical_store_prefix)?;
    if recipe::digest(source_root)?.1 != source.digest_blake3 {
        return Err(error("native derivation source tree changed during evaluation"));
    }
    let bytes = manifest_member(proof, paths, paths.index_member)?;
    let index: NativeExpansionIndex =
        serde_json::from_slice(&bytes).map_err(|failure| error(format!("native expansion index: {failure}")))?;
    if index.recipe_member != paths.recipe_member {
        return Err(error("native expansion index selects a different source program"));
    }
    // Verify recipe record is source-data before common hydration/lowering.
    let _ = manifest_member(proof, paths, paths.recipe_member)?;
    let prepared = recipe::prepare_from_reference(
        proof,
        STAGE,
        &ArtifactRef::ManifestSource {
            role: SourceAuthorityRole::NativeSourceBundle,
            record_identity: paths.record_identity.to_string(),
            member: paths.recipe_member.to_string(),
        },
        paths.recipe_paths,
        paths.stagex_recipes,
        host_mantle_digest_blake3,
    )?;
    validate_expansion(&graph, &index, prepared.source_recipe(), &proof.logical_store_prefix)?;
    validate_exports(&prepared)?;
    if !paths.publication_path.is_absolute() {
        return Err(error("native publication requires an absolute artifact path"));
    }
    let mut references = BTreeSet::new();
    for role in EXECUTABLE_EXPORTS.into_iter().chain(TREE_EXPORTS).chain(RUNTIME_EXPORTS) {
        references.insert(prepared.export(role)?.reference.clone());
    }
    let inputs = references
        .into_iter()
        .map(|reference| {
            let ArtifactRef::Produced { action_id, output_id } = reference else {
                return Err(error("native publication consumes only actually produced artifacts"));
            };
            Ok(ActionInput::Produced { action_id, output_id })
        })
        .collect::<Result<Vec<_>, RunError>>()?;
    let producer_actions = inputs
        .iter()
        .filter_map(|input| match input {
            ActionInput::Produced { action_id, .. } => Some(action_id.clone()),
            _ => None,
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let publication_action = ActionSpec {
        id: format!("{STAGE}/publish-native-provider"),
        stage_id: STAGE.to_string(),
        producer_actions,
        guard: None,
        accepted_exit_codes: Vec::new(),
        executable: ActionExecutable::Fixed {
            digest_blake3: host_mantle_digest_blake3.to_string(),
        },
        inputs,
        output_ids: vec!["native-provider-publication".to_string()],
        protected_events_min: 0,
        protected_events_max: 0,
        build_events_min: 1,
        build_events_max: 1,
        elapsed_seconds_max: proof.resource_bounds.elapsed_seconds_max,
        disk_bytes_max: proof.resource_bounds.disk_bytes_max,
        local_only: true,
    };
    let mut adapter = prepared.adapter().clone();
    if adapter.reachable_action_ids.iter().any(|id| id == &publication_action.id) {
        return Err(error("native recipe collides with reserved publication action"));
    }
    adapter.reachable_action_ids.push(publication_action.id.clone());
    adapter.reachable_action_ids.sort();
    adapter.actions.push(publication_action.clone());
    let source_manifest_digest = proof
        .source_inputs
        .iter()
        .find(|input| input.role == SourceAuthorityRole::NativeSourceBundle)
        .ok_or_else(|| error("missing native manifest authority"))?
        .digest_blake3
        .clone();
    Ok(PreparedNativeActions {
        prepared,
        adapter,
        publication_action,
        publication_path: paths.publication_path.to_path_buf(),
        proof_digest: proof.plan_digest_blake3.clone(),
        expected_provider_digest: proof.policies.expected_native_provider_digest_blake3.clone(),
        source_manifest_digest,
        proof_root: paths.proof_root.to_path_buf(),
    })
}

/// No summary-to-event conversion: every record is emitted by the protected
/// source-program executor or an actual local publication operation.
pub(crate) fn run_with_action_observations(
    actions: &PreparedNativeActions,
    emit: &mut dyn FnMut(ActionExecutionEvent) -> Result<(), RunError>,
) -> Result<PathBuf, RunError> {
    let recipe = recipe::run(&actions.prepared, emit)?;
    let started = std::time::Instant::now();
    let current_executable = std::env::current_exe().map_err(|failure| error(failure.to_string()))?;
    let host_digest = recipe::digest(&current_executable)?.1;
    let mut event = ActionExecutionEvent {
        action_id: actions.publication_action.id.clone(),
        kind: ExecutionEventKind::BuildExecution,
        normal_exit_code: None,
        executable_digest_blake3: host_digest,
        executable_producer: None,
        inputs: actions.publication_action.inputs.clone(),
        input_digests_blake3: Vec::new(),
        produced_outputs: Vec::new(),
        locality: "local".to_string(),
        completed_locally: false,
        cache_hit: false,
        live_fetch: false,
        cargo_invocation: false,
        fallback: false,
        protected_exec_violation: false,
        elapsed_seconds: 0,
        disk_bytes: 0,
    };
    let result = publish(actions, &recipe, &mut event);
    event.elapsed_seconds = started.elapsed().as_secs();
    // Persist failure observations too; they intentionally cannot reconcile as
    // a completed local publication and cannot authorize a successful receipt.
    emit(event)?;
    result?;
    Ok(actions.publication_path.clone())
}

fn manifest_member(
    proof: &SourceBuiltFixedPointPlan,
    paths: &AdapterPaths<'_>,
    member: &str,
) -> Result<Vec<u8>, RunError> {
    recipe::relative(member, false)?;
    let manifest_path = paths
        .recipe_paths
        .source_roots
        .get(&SourceAuthorityRole::NativeSourceBundle)
        .ok_or_else(|| error("missing native source manifest"))?;
    let expected = proof
        .source_inputs
        .iter()
        .find(|input| input.role == SourceAuthorityRole::NativeSourceBundle)
        .ok_or_else(|| error("native source manifest is not admitted"))?;
    if recipe::digest(manifest_path)?.1 != expected.digest_blake3 {
        return Err(error("native source manifest content drift"));
    }
    let manifest = crate::source_bundle::read_source_bundle(manifest_path)?;
    let mut matching = manifest.records.iter().filter(|record| record.identity == paths.record_identity);
    let record = matching.next().ok_or_else(|| error("native source program record missing"))?;
    if matching.next().is_some() || !crate::source_bundle::source_record_is_fetcher_input(record) {
        return Err(error("native source program must be one authenticated fetch-source record"));
    }
    // The attachment may contain source text only. Produced code, libraries,
    // archives, symlinks and executable-mode files cannot enter through this edge.
    if record.files.is_empty()
        || record.payload_bytes > SOURCE_BYTES_MAX
        || record
            .files
            .iter()
            .any(|entry| entry.file_type != crate::source_bundle::SourceFileType::Regular || entry.executable)
    {
        return Err(error("native recipe record is not bounded inert source data"));
    }
    let temporary = tempfile::tempdir().map_err(|failure| error(failure.to_string()))?;
    let destination = temporary.path().join("source");
    crate::source_bundle::materialize_source_record_exact_payload(record, &destination)?;
    // Validate every attachment member, not only the selected recipe/index.
    for entry in &record.files {
        recipe::relative(&entry.path, false)?;
        let path = destination.join(&entry.path);
        let bytes = std::fs::read(path).map_err(|failure| error(failure.to_string()))?;
        let text = std::str::from_utf8(&bytes).map_err(|_| error("binary bytes in native source program record"))?;
        recipe::validate_text(text)?;
    }
    let selected = destination.join(member);
    let metadata = std::fs::symlink_metadata(&selected).map_err(|failure| error(failure.to_string()))?;
    if !metadata.is_file() || metadata.len() > SOURCE_BYTES_MAX {
        return Err(error("native source program member is not bounded regular source data"));
    }
    std::fs::read(selected).map_err(|failure| error(failure.to_string()))
}

fn validate_expansion(
    graph: &crunch_pipeline::native_preflight::NativeDerivationClosure,
    index: &NativeExpansionIndex,
    program: &SourceRecipe,
    store_prefix: &str,
) -> Result<(), RunError> {
    if index.schema != INDEX_SCHEMA || program.stage_id != STAGE {
        return Err(error("native expansion schema/stage mismatch"));
    }
    let expected_roots = graph
        .roots
        .iter()
        .map(|root| root.drv_path.to_absolute_path_with_prefix(store_prefix))
        .collect::<Vec<_>>();
    if index.roots != expected_roots || index.derivations.len() != graph.derivations.len() {
        return Err(error("native expansion omits or changes the seven-root derivation union"));
    }
    let local_steps = program.steps.iter().map(|step| (step.id.as_str(), step)).collect::<BTreeMap<_, _>>();
    let mut assigned = BTreeSet::new();
    for node in &graph.derivations {
        let key = node.drv_path.to_absolute_path_with_prefix(store_prefix);
        let expanded = index
            .derivations
            .get(&key)
            .ok_or_else(|| error(format!("missing native derivation expansion {key}")))?;
        let hdm = node.hash_derivation_modulo.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
        let dependencies = node
            .derivation
            .input_derivations
            .iter()
            .map(|(path, outputs)| (path.to_absolute_path_with_prefix(store_prefix), outputs.clone()))
            .collect::<BTreeMap<_, _>>();
        let sources = node
            .derivation
            .input_sources
            .iter()
            .map(|path| path.to_absolute_path_with_prefix(store_prefix))
            .collect::<BTreeSet<_>>();
        if expanded.hash_derivation_modulo != hdm
            || expanded.input_derivations != dependencies
            || expanded.input_sources != sources
        {
            return Err(error(format!("native derivation source/dependency identity drift: {key}")));
        }
        if expanded.step_ids.is_empty() || expanded.outputs.keys().ne(node.derivation.outputs.keys()) {
            return Err(error(format!("native derivation has incomplete command/output expansion: {key}")));
        }
        for step in &expanded.step_ids {
            if !local_steps.contains_key(step.as_str()) || !assigned.insert(step.as_str()) {
                return Err(error(format!("native step missing or assigned to multiple derivations: {step}")));
            }
        }
        for reference in expanded.outputs.values() {
            let ArtifactRef::Produced { action_id, output_id } = reference else {
                return Err(error("native derivation completion cannot be imported source"));
            };
            let id =
                action_id.strip_prefix(&format!("{STAGE}/")).ok_or_else(|| error("cross-stage native completion"))?;
            let step = local_steps.get(id).ok_or_else(|| error("unknown native completion producer"))?;
            if !expanded.step_ids.contains(id) || !step.outputs.contains_key(output_id) {
                return Err(error("native output lacks its exact local producer"));
            }
        }
        // Preserve every original derivation edge in concrete expanded inputs.
        for (dependency, selected) in &expanded.input_derivations {
            let dependency = index.derivations.get(dependency).ok_or_else(|| error("unknown indexed dependency"))?;
            for output in selected {
                let reference = dependency.outputs.get(output).ok_or_else(|| error("unknown dependency output"))?;
                if !expanded
                    .step_ids
                    .iter()
                    .any(|id| local_steps[id.as_str()].inputs.values().any(|input| input == reference))
                {
                    return Err(error(format!(
                        "native expanded commands omit selected producer output {output} consumed by {key}"
                    )));
                }
            }
        }
    }
    if assigned.len() != local_steps.len() {
        return Err(error("native source program has unclassified command/file-generator steps"));
    }
    Ok(())
}

fn validate_exports(prepared: &PreparedRecipe) -> Result<(), RunError> {
    for role in EXECUTABLE_EXPORTS {
        if prepared.export(role)?.kind != ArtifactKind::Executable {
            return Err(error(format!("native executable role is not an individual produced executable: {role}")));
        }
    }
    for role in TREE_EXPORTS {
        if prepared.export(role)?.kind != ArtifactKind::Tree {
            return Err(error(format!("native data-tree role has wrong artifact kind: {role}")));
        }
    }
    for role in RUNTIME_EXPORTS {
        let binding = prepared.export(role)?;
        if !matches!(binding.kind, ArtifactKind::Archive | ArtifactKind::Object | ArtifactKind::Executable) {
            return Err(error(format!("native runtime is not an individual produced binary: {role}")));
        }
    }
    let provider = prepared.export("native.provider")?;
    if !matches!(&provider.reference, ArtifactRef::Produced { output_id, .. } if output_id == "full-source-native-provider")
    {
        return Err(error("native provider export lacks canonical stage output identity"));
    }
    Ok(())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NativePublishedArtifact {
    pub(crate) producer: ArtifactRef,
    pub(crate) path: PathBuf,
    pub(crate) digest_blake3: String,
    pub(crate) size_bytes: u64,
    pub(crate) kind: ArtifactKind,
}

/// This is measured local publication, not a synthesized scheduler attestation
/// or an assertion that an unexecuted runtime-smoke matrix passed.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NativeProviderPublication {
    pub(crate) schema: String,
    pub(crate) proof_plan_digest_blake3: String,
    pub(crate) native_source_manifest_digest_blake3: String,
    pub(crate) expected_provider_digest_blake3: String,
    pub(crate) artifacts: BTreeMap<String, NativePublishedArtifact>,
}

fn publish(
    actions: &PreparedNativeActions,
    run: &RecipeRun,
    event: &mut ActionExecutionEvent,
) -> Result<NativeProviderPublication, RunError> {
    if !matches!(&actions.publication_action.executable, ActionExecutable::Fixed { digest_blake3 } if digest_blake3 == &event.executable_digest_blake3)
    {
        return Err(error("native publication host executable identity drift"));
    }
    let mut measured = BTreeMap::new();
    for input in &event.inputs {
        let ActionInput::Produced { action_id, output_id } = input else {
            return Err(error("publication input is not an exact generated artifact"));
        };
        let reference = ArtifactRef::Produced {
            action_id: action_id.clone(),
            output_id: output_id.clone(),
        };
        let (path, observed) = run.outputs.get(&reference).ok_or_else(|| {
            error(format!("native publication lacks actual producer completion {action_id}/{output_id}"))
        })?;
        let (size, digest) = recipe::digest(path)?;
        event.input_digests_blake3.push(digest.clone());
        if &digest != observed {
            return Err(error("native artifact changed after its observed publication"));
        }
        measured.insert(reference, (path.clone(), size, digest));
    }
    let mut artifacts = BTreeMap::new();
    for role in EXECUTABLE_EXPORTS.into_iter().chain(TREE_EXPORTS).chain(RUNTIME_EXPORTS) {
        let binding = actions.export(role)?;
        let (path, size, digest) = measured
            .get(&binding.reference)
            .ok_or_else(|| error("native exported artifact absent from actual completion records"))?;
        if path != &binding.path || *size > binding.bytes_max {
            return Err(error("native producer physical binding or size bound drift"));
        }
        artifacts.insert(role.to_string(), NativePublishedArtifact {
            producer: binding.reference.clone(),
            path: path
                .strip_prefix(&actions.proof_root)
                .map_err(|_| error("native artifact is outside the proof root"))?
                .to_path_buf(),
            digest_blake3: digest.clone(),
            size_bytes: *size,
            kind: binding.kind,
        });
    }
    if artifacts["native.provider"].digest_blake3 != actions.expected_provider_digest {
        return Err(error("actual normalized native provider digest differs from the required proof pin"));
    }
    let publication = NativeProviderPublication {
        schema: "mantle-source-native-provider-publication-v1".to_string(),
        proof_plan_digest_blake3: actions.proof_digest.clone(),
        native_source_manifest_digest_blake3: actions.source_manifest_digest.clone(),
        expected_provider_digest_blake3: actions.expected_provider_digest.clone(),
        artifacts,
    };
    let bytes = serde_json::to_vec(&publication).map_err(|failure| error(failure.to_string()))?;
    if bytes.len() as u64 > SOURCE_BYTES_MAX {
        return Err(error("native publication exceeds bounded metadata size"));
    }
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&actions.publication_path)
        .map_err(|failure| error(format!("creating native publication: {failure}")))?;
    file.write_all(&bytes)
        .and_then(|()| file.sync_all())
        .map_err(|failure| error(format!("persisting native publication: {failure}")))?;
    let (size, digest) = recipe::digest(&actions.publication_path)?;
    if digest != blake3::hash(&bytes).to_hex().to_string() || size != bytes.len() as u64 {
        return Err(error("native publication bytes changed while publishing"));
    }
    event.disk_bytes = size;
    event.completed_locally = true;
    event.produced_outputs.push(("native-provider-publication".to_string(), digest));
    Ok(publication)
}
