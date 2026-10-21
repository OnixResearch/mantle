//! Source-owned original StageX recipe construction and publication.
//! Fetch decoding consumes both the exact selected original record and its
//! whole source manifest. Generated files and command statuses are Produced
//! facts, never preparation-time source authority. Publication is refused until
//! every original tool output is owned by an actual command in that tool.
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

use serde::Serialize;

use crate::errors::RunError;
use crate::source_built_fixed_point::SourceAuthorityRole;
use crate::source_built_program_authoring::AuthoredPrograms;
use crate::source_built_program_authoring::AuthoringRequest;
use crate::source_built_recipe::ArtifactKind;
use crate::source_built_recipe::ArtifactRef;
use crate::source_built_recipe::RecipeExitPolicy;
use crate::source_built_recipe::RecipeOperation;
use crate::source_built_recipe::RecipeOutput;
use crate::source_built_recipe::RecipePreparation;
use crate::source_built_recipe::RecipeStep;
use crate::source_built_recipe::RecipeWord;
use crate::source_built_recipe::SourceRecipe;
use crate::source_built_recipe::WordPart;
use crate::source_built_recipe::{self as recipe};

#[path = "source_built_stagex_programs_binutils.rs"]
mod binutils;
#[path = "source_built_stagex_programs_controller.rs"]
pub(crate) mod controller;
#[path = "source_built_stagex_programs_early.rs"]
mod early;
#[path = "source_built_stagex_programs_generators.rs"]
pub(crate) mod generators;
#[path = "source_built_stagex_programs_late.rs"]
mod late;
#[path = "source_built_stagex_programs_binutils_ldscripts.rs"]
mod ldscripts;
#[path = "source_built_stagex_programs_middle.rs"]
mod middle;
#[path = "source_built_stagex_programs_publication.rs"]
mod publication;
#[path = "source_built_stagex_programs_binutils_shellvm.rs"]
pub(crate) mod shellvm;

pub(super) const PROGRAM_RECORD: &str = "source-built-stagex-programs";
const TRANSITION: &str = "stagex-transition";
const INDEX_SCHEMA: &str = "mantle-stagex-source-expansion-v1";
pub(super) fn error(message: impl Into<String>) -> RunError {
    recipe::error(format!("StageX authoring: {}", message.into()))
}
pub(super) fn literal(text: &str) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Literal { text: text.into() }],
    }
}
pub(super) fn input(name: &str, path: &str) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Input {
            name: name.into(),
            path: path.into(),
        }],
    }
}
pub(super) fn output(name: &str, path: &str) -> RecipeWord {
    RecipeWord {
        parts: vec![WordPart::Output {
            name: name.into(),
            path: path.into(),
        }],
    }
}
pub(super) fn bounded(kind: ArtifactKind, bytes_max: u64) -> RecipeOutput {
    RecipeOutput { kind, bytes_max }
}
pub(super) fn program(stage: &str) -> SourceRecipe {
    SourceRecipe {
        timeout_groups: Vec::new(),
        schema: recipe::SCHEMA.into(),
        stage_id: stage.into(),
        preparation: RecipePreparation {
            tool: "mantle-source-built-stagex-author".into(),
            provenance: "original authenticated StageX source, recipe, generator and lineage lowering".into(),
        },
        steps: Vec::new(),
        roots: Vec::new(),
        exports: BTreeMap::new(),
        semantic_roots: BTreeMap::new(),
    }
}
#[derive(Serialize, Default)]
pub(super) struct ExpandedTool {
    pub step_ids: BTreeSet<String>,
    pub outputs: BTreeMap<String, ArtifactRef>,
}
#[derive(Serialize)]
struct TransitionIndex<'a> {
    schema: &'static str,
    recipe_member: &'static str,
    sources: &'a BTreeMap<String, ArtifactRef>,
    patches: &'a BTreeMap<String, ArtifactRef>,
    tools: &'a BTreeMap<String, ExpandedTool>,
}

pub(super) struct TransitionBuilder {
    pub recipe: SourceRecipe,
    pub artifacts: BTreeMap<String, ArtifactRef>,
    pub tools: BTreeMap<String, ExpandedTool>,
    pub files: BTreeMap<String, String>,
    sources: BTreeMap<String, ArtifactRef>,
    patches: BTreeMap<String, ArtifactRef>,
    manifest: crate::source_bundle::SourceBundleManifest,
    scratch: tempfile::TempDir,
    hydrated: BTreeMap<String, PathBuf>,
    fetchers: BTreeMap<String, ArtifactRef>,
    source_owners: BTreeMap<String, String>,
    members: BTreeMap<(String, String), ArtifactRef>,
    policy_sources: Option<BTreeMap<String, String>>,
    original_tool_inputs: BTreeMap<String, Vec<String>>,
    original_tool_outputs: BTreeMap<String, Vec<String>>,
    graph_edges: Vec<(String, String)>,
}
impl TransitionBuilder {
    pub fn source(&self, id: &str) -> Result<ArtifactRef, RunError> {
        self.sources
            .get(id)
            .or_else(|| self.patches.get(id))
            .cloned()
            .ok_or_else(|| error(format!("unknown original source {id}")))
    }
    pub fn artifact(&self, id: &str) -> Result<ArtifactRef, RunError> {
        self.artifacts.get(id).cloned().ok_or_else(|| error(format!("missing original producer {id}")))
    }
    pub fn original_inputs(&self, tool: &str) -> Result<BTreeMap<String, ArtifactRef>, RunError> {
        self.original_tool_inputs
            .get(tool)
            .ok_or_else(|| error(format!("unknown original tool {tool}")))?
            .iter()
            .map(|id| Ok((id.clone(), self.artifact(id)?)))
            .collect()
    }
    pub fn predecessors(&self, tool: &str) -> Result<BTreeMap<String, ArtifactRef>, RunError> {
        let outputs =
            self.original_tool_outputs.get(tool).ok_or_else(|| error(format!("unknown original tool {tool}")))?;
        let mut predecessors = BTreeMap::new();
        for (from, to) in &self.graph_edges {
            if to != tool && !outputs.contains(to) {
                continue;
            }
            if let Some(reference) = self.artifacts.get(from) {
                predecessors.insert(from.clone(), reference.clone());
            } else if let Some(ids) = self.original_tool_outputs.get(from) {
                for id in ids {
                    predecessors.insert(id.clone(), self.artifact(id)?);
                }
            } else {
                return Err(error(format!("original predecessor {from} is not authored before {tool}")));
            }
        }
        Ok(predecessors)
    }
    pub fn member(&mut self, id: &str, path: &str) -> Result<ArtifactRef, RunError> {
        recipe::relative(path, true)?;
        let key = (id.to_string(), path.to_string());
        if let Some(reference) = self.members.get(&key) {
            return Ok(reference.clone());
        }
        let reference = self.source(id)?;
        let ArtifactRef::ManifestSource {
            role,
            record_identity,
            member,
        } = reference.clone()
        else {
            return if path.is_empty() {
                Ok(reference)
            } else {
                Err(error(format!("{id} is not an original source tree")))
            };
        };
        if path.is_empty() {
            return Ok(reference);
        }
        if !member.is_empty() {
            return Err(error(format!("{id} is not an original source tree")));
        }
        let record = self
            .manifest
            .records
            .iter()
            .find(|record| record.identity == record_identity)
            .ok_or_else(|| error("missing original source record"))?;
        if record.files.iter().any(|file| file.path == path) {
            return Ok(ArtifactRef::ManifestSource {
                role,
                record_identity,
                member: path.into(),
            });
        }
        // Archive members are not raw manifest members. Materialize the pinned
        // original fetcher result, then copy the actual member with produced cp.
        // No binary data or unpacked snapshots enter the inert program record.
        let tool = self
            .source_owners
            .get(id)
            .cloned()
            .ok_or_else(|| error(format!("no original source owner for {id}")))?;
        let tree = self.fetcher(&tool, id)?;
        let root = self.source_path(id)?;
        let original = recipe::source_member(&root, path)?;
        let bytes = std::fs::read(original).map_err(|e| error(e.to_string()))?;
        let kind = if std::str::from_utf8(&bytes).is_ok() {
            ArtifactKind::Text
        } else {
            ArtifactKind::Object
        };
        let outputs = self.add(&tool, RecipeStep {
            id: format!("original-member-{}", self.members.len()),
            inputs: BTreeMap::from([
                ("executable".into(), self.artifact("stage0-full-cp")?),
                ("tree".into(), tree),
            ]),
            outputs: BTreeMap::from([
                ("member".into(), bounded(kind, 64 * 1024 * 1024)),
                ("stdout".into(), bounded(ArtifactKind::Text, 1024 * 1024)),
                ("stderr".into(), bounded(ArtifactKind::Text, 1024 * 1024)),
                ("exit-status".into(), bounded(ArtifactKind::Text, 4)),
            ]),
            operation: RecipeOperation::Exec {
                invocation_input: None,
                command_state: None,
                merge_stderr: false,
                executable_input: "executable".into(),
                arguments: vec![input("tree", path), output("member", "")],
                environment: BTreeMap::new(),
                runtime_mounts: BTreeMap::new(),
                cwd_input: None,
                cwd_output: None,
                stdin_input: None,
                exit_policy: RecipeExitPolicy::Exact { code: 0 },
                output_seeds: BTreeMap::new(),
                guard: None,
            },
        })?;
        let reference = outputs["member"].clone();
        self.members.insert(key, reference.clone());
        self.artifacts.insert(format!("source-member:{id}:{path}"), reference.clone());
        Ok(reference)
    }
    /// Hydrate an original fetcher result at execution, retaining its raw record
    /// identity separately for lineage source admission.
    pub fn fetcher(&mut self, tool: &str, id: &str) -> Result<ArtifactRef, RunError> {
        let key = format!("{tool}:{id}");
        if let Some(reference) = self.fetchers.get(&key) {
            return Ok(reference.clone());
        }
        let source = self.source(id)?;
        let ArtifactRef::ManifestSource {
            role,
            record_identity,
            member,
        } = source.clone()
        else {
            return Ok(source);
        };
        if !member.is_empty() || record_identity == PROGRAM_RECORD {
            return Ok(source);
        }
        let record = self
            .manifest
            .records
            .iter()
            .find(|record| record.identity == record_identity)
            .ok_or_else(|| error("missing original fetcher record"))?;
        if !crate::source_bundle::source_record_is_fetcher_input(record) {
            return Err(error("original archive is not fetcher authority"));
        }
        let kind = if record.metadata.get("unpack").map(String::as_str) == Some("1")
            || record.kind == crate::source_bundle::SourceRecordKind::VcsSnapshot
        {
            ArtifactKind::Tree
        } else {
            ArtifactKind::Text
        };
        let inputs = BTreeMap::from([
            ("source".into(), source),
            ("manifest".into(), ArtifactRef::Source {
                role,
                path: String::new(),
            }),
        ]);
        let outputs = self.add(tool, RecipeStep {
            id: format!("materialize-{tool}-{id}"),
            inputs,
            outputs: BTreeMap::from([("source".into(), bounded(kind, 1024 * 1024 * 1024))]),
            operation: RecipeOperation::MaterializeFetcher {
                role,
                record_identity,
                output: "source".into(),
            },
        })?;
        let reference = outputs["source"].clone();
        self.artifacts.insert(format!("fetcher:{key}"), reference.clone());
        self.fetchers.insert(key, reference.clone());
        Ok(reference)
    }
    pub fn authored(&mut self, path: &str, text: String) -> Result<ArtifactRef, RunError> {
        recipe::relative(path, false)?;
        recipe::validate_text(&text)?;
        if let Some(old) = self.files.get(path) {
            if old != &text {
                return Err(error(format!("conflicting helper source {path}")));
            }
        } else {
            self.files.insert(path.into(), text);
        }
        Ok(ArtifactRef::ManifestSource {
            role: SourceAuthorityRole::StagexSourceBundle,
            record_identity: PROGRAM_RECORD.into(),
            member: path.into(),
        })
    }
    pub fn helper_source(&self) -> ArtifactRef {
        ArtifactRef::ManifestSource {
            role: SourceAuthorityRole::StagexSourceBundle,
            record_identity: PROGRAM_RECORD.into(),
            member: "helpers/stagex-source-transform.c".into(),
        }
    }
    pub fn source_path(&mut self, id: &str) -> Result<PathBuf, RunError> {
        if let Some(path) = self.hydrated.get(id) {
            return Ok(path.clone());
        }
        let reference = self.source(id)?;
        let path = match reference {
            ArtifactRef::ManifestSource {
                record_identity,
                member,
                ..
            } if record_identity == PROGRAM_RECORD => {
                let text = self.files.get(&member).ok_or_else(|| error("missing authored original source"))?;
                let path = self.scratch.path().join(format!("original-{}", self.hydrated.len()));
                std::fs::write(&path, text).map_err(|e| error(e.to_string()))?;
                path
            }
            ArtifactRef::ManifestSource {
                record_identity,
                member,
                ..
            } => {
                let record = self
                    .manifest
                    .records
                    .iter()
                    .find(|r| r.identity == record_identity)
                    .ok_or_else(|| error("original record missing"))?;
                let root = self.scratch.path().join(format!("record-{}", self.hydrated.len()));
                crate::source_bundle::materialize_source_record_for_offline_use(record, &root)?;
                if member.is_empty() { root } else { root.join(member) }
            }
            _ => return Err(error(format!("source {id} is not an inspectable original record"))),
        };
        self.hydrated.insert(id.into(), path.clone());
        Ok(path)
    }
    pub fn read_source(&mut self, id: &str, path: &str) -> Result<String, RunError> {
        recipe::relative(path, true)?;
        let root = self.source_path(id)?;
        let file = if path.is_empty() { root } else { root.join(path) };
        let meta = std::fs::symlink_metadata(&file).map_err(|e| error(e.to_string()))?;
        if !meta.is_file() || meta.len() > 64 * 1024 * 1024 {
            return Err(error("original source text exceeds bound or is not regular"));
        }
        std::fs::read_to_string(file).map_err(|e| error(e.to_string()))
    }
    pub fn members(&mut self, id: &str) -> Result<Vec<String>, RunError> {
        let root = self.source_path(id)?;
        let mut result = Vec::new();
        collect_members(&root, &root, &mut result)?;
        result.sort();
        Ok(result)
    }
    pub fn register_controller(
        &mut self,
        tool: &str,
        specification: controller::ControllerSpec,
    ) -> Result<controller::ControllerResult, RunError> {
        let id = specification.id.clone();
        let result = controller::append_controller(&mut self.recipe, specification)?;
        self.tools.entry(tool.into()).or_default().step_ids.insert(id);
        Ok(result)
    }
    pub fn add(&mut self, tool: &str, step: RecipeStep) -> Result<BTreeMap<String, ArtifactRef>, RunError> {
        if self.recipe.steps.iter().any(|old| old.id == step.id) {
            return Err(error(format!("duplicate leaf {}", step.id)));
        }
        let refs = step
            .outputs
            .keys()
            .map(|id| {
                (id.clone(), ArtifactRef::Produced {
                    action_id: format!("{TRANSITION}/{}", step.id),
                    output_id: id.clone(),
                })
            })
            .collect();
        self.tools.entry(tool.into()).or_default().step_ids.insert(step.id.clone());
        self.recipe.steps.push(step);
        Ok(refs)
    }
    pub fn publish(&mut self, tool: &str, id: &str, reference: ArtifactRef) -> Result<(), RunError> {
        if self.artifacts.contains_key(id) {
            return Err(error(format!("duplicate original output {id}")));
        }
        self.artifacts.insert(id.into(), reference.clone());
        self.tools.entry(tool.into()).or_default().outputs.insert(id.into(), reference);
        Ok(())
    }
}
fn collect_members(root: &Path, path: &Path, output: &mut Vec<String>) -> Result<(), RunError> {
    let meta = std::fs::symlink_metadata(path).map_err(|e| error(e.to_string()))?;
    if meta.is_dir() {
        for entry in std::fs::read_dir(path).map_err(|e| error(e.to_string()))? {
            collect_members(root, &entry.map_err(|e| error(e.to_string()))?.path(), output)?;
        }
    } else if path != root {
        output.push(
            path.strip_prefix(root)
                .map_err(|e| error(e.to_string()))?
                .to_str()
                .ok_or_else(|| error("non UTF-8 original source member"))?
                .replace('\\', "/"),
        );
    }
    Ok(())
}

pub(crate) fn author(request: &AuthoringRequest<'_>) -> Result<AuthoredPrograms, RunError> {
    let lineage_path = request
        .source_roots
        .get(&SourceAuthorityRole::StagexLineage)
        .ok_or_else(|| error("missing lineage root"))?;
    let bundle_path = request
        .source_roots
        .get(&SourceAuthorityRole::StagexSourceBundle)
        .ok_or_else(|| error("missing original StageX source bundle"))?;
    for (role, path) in [
        (SourceAuthorityRole::StagexLineage, lineage_path),
        (SourceAuthorityRole::StagexSourceBundle, bundle_path),
    ] {
        let authority = request
            .proof
            .source_inputs
            .iter()
            .find(|s| s.role == role)
            .ok_or_else(|| error("missing original source authority"))?;
        if recipe::digest(path)?.1 != authority.digest_blake3 {
            return Err(error("original source root identity drift"));
        }
    }
    let bytes = std::fs::read(lineage_path).map_err(|e| error(e.to_string()))?;
    let lineage: crunch_bootstrap_core::LineageManifest =
        serde_json::from_slice(&bytes).map_err(|e| error(e.to_string()))?;
    crunch_bootstrap_core::admit_lineage_manifest(&lineage).map_err(|e| error(e.to_string()))?;
    let manifest = crate::source_bundle::read_source_bundle(bundle_path)?;
    let scratch = tempfile::Builder::new()
        .prefix("stagex-author-original-")
        .tempdir_in(request.scratch_root)
        .map_err(|e| error(e.to_string()))?;
    let mut builder = TransitionBuilder {
        recipe: program(TRANSITION),
        artifacts: BTreeMap::new(),
        tools: BTreeMap::new(),
        files: BTreeMap::new(),
        sources: BTreeMap::new(),
        patches: BTreeMap::new(),
        manifest,
        scratch,
        hydrated: BTreeMap::new(),
        fetchers: BTreeMap::new(),
        source_owners: BTreeMap::new(),
        members: BTreeMap::new(),
        policy_sources: None,
        original_tool_inputs: lineage
            .transition_tools
            .iter()
            .map(|tool| {
                (tool.id.clone(), tool.source_artifact_ids.iter().chain(&tool.input_artifact_ids).cloned().collect())
            })
            .collect(),
        original_tool_outputs: lineage
            .transition_tools
            .iter()
            .map(|tool| (tool.id.clone(), tool.output_artifact_ids.clone()))
            .collect(),
        graph_edges: lineage
            .stage_graph
            .iter()
            .map(|edge| (edge.from_node_id.clone(), edge.to_node_id.clone()))
            .collect(),
    };
    for tool in &lineage.transition_tools {
        for id in &tool.source_artifact_ids {
            builder.source_owners.entry(id.clone()).or_insert_with(|| tool.id.clone());
        }
    }
    builder.authored(
        "helpers/stagex-source-transform.c",
        include_str!("source_built_stagex_programs_transform.c").into(),
    )?;
    for source in &lineage.source_artifacts {
        let reference = bind_original(&mut builder, request, &source.id, &source.digest.hex_value)?;
        builder.sources.insert(source.id.clone(), reference.clone());
        builder.artifacts.insert(source.id.clone(), reference);
    }
    for patch in &lineage.patches {
        let reference = bind_original(&mut builder, request, &patch.id, &patch.digest.hex_value)?;
        builder.patches.insert(patch.id.clone(), reference.clone());
        builder.artifacts.insert(patch.id.clone(), reference);
    }
    builder.artifacts.insert("seed".into(), ArtifactRef::Source {
        role: SourceAuthorityRole::StagexSeed,
        path: String::new(),
    });
    // Source archive hydration is an explicit original source-materialization
    // phase for every original consumer, not a prepared tree imported as source.
    for tool in &lineage.transition_tools {
        for id in &tool.source_artifact_ids {
            if matches!(builder.source(id)?,ArtifactRef::ManifestSource {ref member,ref record_identity,..} if member.is_empty()&&record_identity!=PROGRAM_RECORD)
            {
                builder.fetcher(&tool.id, id)?;
            }
        }
    }
    seed_leaves(&mut builder)?;
    early::extend(&mut builder, request)?;
    middle::extend(&mut builder, request)?;
    late::extend(&mut builder, request)?;
    generators::extend(&mut builder, request)?;
    binutils::extend(&mut builder, request)?;
    for tool in &lineage.transition_tools {
        let expanded = builder
            .tools
            .get(&tool.id)
            .ok_or_else(|| error(format!("original tool {} has no authored commands", tool.id)))?;
        if expanded.step_ids.is_empty() {
            return Err(error(format!("original tool {} has no authored commands", tool.id)));
        }
        for id in &tool.output_artifact_ids {
            let reference = expanded
                .outputs
                .get(id)
                .ok_or_else(|| error(format!("original tool {} has no authored output {id}", tool.id)))?;
            let ArtifactRef::Produced { action_id, .. } = reference else {
                return Err(error(format!("original output {id} is not produced")));
            };
            let producer = action_id
                .strip_prefix(&format!("{TRANSITION}/"))
                .ok_or_else(|| error(format!("original output {id} belongs to another stage")))?;
            if !expanded.step_ids.contains(producer) || builder.artifacts.get(id) != Some(reference) {
                return Err(error(format!("original output {id} has inconsistent producer ownership")));
            }
        }
    }
    // The canonical transition retains every original output, including generator
    // products and observations, rather than replacing their ancestry with labels.
    let produced = builder
        .artifacts
        .iter()
        .filter(|(_, r)| matches!(r, ArtifactRef::Produced { .. }))
        .collect::<Vec<_>>();
    let inputs = produced
        .iter()
        .enumerate()
        .map(|(i, (_, reference))| (format!("artifact-{i}"), (*reference).clone()))
        .collect();
    let entries = produced
        .iter()
        .enumerate()
        .map(|(i, (id, _))| {
            (
                id.strip_prefix("member:").map(str::to_owned).unwrap_or_else(|| format!("original/{id}")),
                format!("artifact-{i}"),
            )
        })
        .collect();
    let step = RecipeStep {
        id: "canonical-transition".into(),
        inputs,
        outputs: BTreeMap::from([(
            TRANSITION.into(),
            bounded(ArtifactKind::Tree, request.proof.resource_bounds.disk_bytes_max),
        )]),
        operation: RecipeOperation::AssembleTree {
            output: TRANSITION.into(),
            entries,
        },
    };
    let canonical = ArtifactRef::Produced {
        action_id: format!("{TRANSITION}/{}", step.id),
        output_id: TRANSITION.into(),
    };
    // Canonical assembly belongs to the final original producer, not an extra
    // topology node invented by the sidecar.
    let final_tool = lineage.transition_tools.last().ok_or_else(|| error("empty original transition"))?.id.clone();
    builder.add(&final_tool, step)?;
    builder.recipe.roots = vec![canonical.clone()];
    builder.recipe.exports = builder
        .artifacts
        .iter()
        .filter(|(_, r)| matches!(r, ArtifactRef::Produced { .. }))
        .map(|(id, r)| (id.clone(), r.clone()))
        .collect();
    builder.recipe.exports.insert(TRANSITION.into(), canonical);
    recipe::validate_source_recipe(&builder.recipe)?;
    let index = TransitionIndex {
        schema: INDEX_SCHEMA,
        recipe_member: "stagex-transition.json",
        sources: &builder.sources,
        patches: &builder.patches,
        tools: &builder.tools,
    };
    builder.files.insert(
        "stagex-transition.json.expansion.json".into(),
        serde_json::to_string_pretty(&index).map_err(|e| error(e.to_string()))?,
    );
    builder.files.insert(
        "stagex-transition.json".into(),
        serde_json::to_string_pretty(&builder.recipe).map_err(|e| error(e.to_string()))?,
    );
    let mut exports = builder
        .recipe
        .exports
        .iter()
        .map(|(k, v)| (format!("{TRANSITION}:{k}"), v.clone()))
        .collect::<BTreeMap<_, _>>();
    publication::extend(&mut builder, request, &mut exports)?;
    Ok(AuthoredPrograms {
        files: builder.files,
        exports,
    })
}
fn bind_original(
    builder: &mut TransitionBuilder,
    request: &AuthoringRequest<'_>,
    id: &str,
    digest: &str,
) -> Result<ArtifactRef, RunError> {
    if id == "stage0-source-bundle" {
        return Ok(ArtifactRef::Source {
            role: SourceAuthorityRole::StagexSourceBundle,
            path: String::new(),
        });
    }
    let mut matches = BTreeSet::new();
    for record in builder.manifest.records.iter().filter(|r| crate::source_bundle::source_record_is_fetcher_input(r)) {
        if record.content_blake3 == digest {
            matches.insert(ArtifactRef::ManifestSource {
                role: SourceAuthorityRole::StagexSourceBundle,
                record_identity: record.identity.clone(),
                member: String::new(),
            });
        }
        for file in &record.files {
            if file.blake3 == digest && file.file_type == crate::source_bundle::SourceFileType::Regular {
                matches.insert(ArtifactRef::ManifestSource {
                    role: SourceAuthorityRole::StagexSourceBundle,
                    record_identity: record.identity.clone(),
                    member: file.path.clone(),
                });
            }
        }
    }
    if let Some(reference) = matches.into_iter().next() {
        return Ok(reference);
    }
    if digest == blake3::hash(b"").to_hex().as_str() {
        return builder.authored(&format!("original/{id}"), String::new());
    }
    // Policy helper/patch files already owned by the original repository are
    // copied as inert source only when their exact original lineage digest fits.
    if builder.policy_sources.is_none() {
        let root = request
            .source_roots
            .get(&SourceAuthorityRole::MantleSource)
            .ok_or_else(|| error(format!("original {id} is absent from source bundle")))?;
        let mut sources = BTreeMap::new();
        let bootstrap = root.join("bootstrap");
        let mut candidates = Vec::new();
        collect_members(&bootstrap, &bootstrap, &mut candidates)?;
        candidates.sort();
        for member in candidates {
            let path = bootstrap.join(member);
            let meta = std::fs::symlink_metadata(&path).map_err(|e| error(e.to_string()))?;
            if !meta.is_file() || meta.len() > 16 * 1024 * 1024 {
                continue;
            }
            let bytes = std::fs::read(path).map_err(|e| error(e.to_string()))?;
            if let Ok(text) = String::from_utf8(bytes) {
                sources.insert(blake3::hash(text.as_bytes()).to_hex().to_string(), text.clone());
                // Original local helper sources are also embedded verbatim in
                // checked recipe heredocs. Index those exact source slices by
                // the original lineage digest, never any generated file.
                for (start, _) in text.match_indices("<<") {
                    let tail = &text[start + 2..];
                    if tail.starts_with('<') {
                        continue;
                    }
                    let tail = tail.trim_start_matches([' ', '\t']);
                    let tail = tail.strip_prefix('-').unwrap_or(tail).trim_start_matches([' ', '\t']);
                    let tag = if let Some(rest) = tail.strip_prefix('\'') {
                        rest.split('\'').next().unwrap_or("")
                    } else if let Some(rest) = tail.strip_prefix('"') {
                        rest.split('"').next().unwrap_or("")
                    } else {
                        tail.split(|c: char| !c.is_ascii_alphanumeric() && c != '_').next().unwrap_or("")
                    };
                    if tag.is_empty() || !tag.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
                        continue;
                    }
                    let Some(line_end) = tail.find('\n') else {
                        continue;
                    };
                    let body = &tail[line_end + 1..];
                    let terminator = format!("\n{tag}\n");
                    if let Some(end) = body.find(&terminator) {
                        let source = &body[..end + 1];
                        sources.insert(blake3::hash(source.as_bytes()).to_hex().to_string(), source.to_string());
                    }
                }
            }
        }
        builder.policy_sources = Some(sources);
    }
    if let Some(text) = builder.policy_sources.as_ref().and_then(|sources| sources.get(digest)).cloned() {
        return builder.authored(&format!("original/{id}"), text);
    }
    Err(error(format!("authenticated original source/patch {id} ({digest}) is absent")))
}
fn seed_leaves(b: &mut TransitionBuilder) -> Result<(), RunError> {
    for (tool, id, executable, source, output_id, kind, args) in [
        (
            "hex0-seed-exec",
            "reproduce-hex0",
            "seed",
            "hex0-source",
            "hex0-reproduced",
            ArtifactKind::Executable,
            vec![input("source", ""), output("hex0-reproduced", "")],
        ),
        (
            "hex0-reproduced-exec",
            "materialize-kaem",
            "hex0-reproduced",
            "kaem-minimal-source",
            "kaem-0",
            ArtifactKind::Executable,
            vec![input("source", ""), output("kaem-0", "")],
        ),
        (
            "hex0-reproduced-exec",
            "kaem-smoke",
            "kaem-0",
            "kaem-smoke-script",
            "kaem-smoke-observation",
            ArtifactKind::Text,
            vec![input("source", "")],
        ),
    ] {
        let outputs = BTreeMap::from([
            ("stdout".into(), bounded(ArtifactKind::Text, 1024 * 1024)),
            ("stderr".into(), bounded(ArtifactKind::Text, 1024 * 1024)),
            ("exit-status".into(), bounded(ArtifactKind::Text, 4)),
        ]);
        let mut step = RecipeStep {
            id: id.into(),
            inputs: BTreeMap::from([
                ("executable".into(), b.artifact(executable)?),
                ("source".into(), b.source(source)?),
            ]),
            outputs,
            operation: RecipeOperation::Exec {
                invocation_input: None,
                command_state: None,
                merge_stderr: false,
                executable_input: "executable".into(),
                arguments: args,
                environment: BTreeMap::new(),
                runtime_mounts: BTreeMap::new(),
                cwd_input: None,
                cwd_output: None,
                stdin_input: None,
                exit_policy: RecipeExitPolicy::Exact { code: 0 },
                output_seeds: BTreeMap::new(),
                guard: None,
            },
        };
        if kind == ArtifactKind::Executable {
            step.outputs.insert(output_id.into(), bounded(kind, 64 * 1024 * 1024));
        }
        let produced = b.add(tool, step)?;
        if kind == ArtifactKind::Executable {
            b.publish(tool, output_id, produced[output_id].clone())?;
        } else {
            b.artifacts.insert(output_id.into(), produced["stdout"].clone());
        }
    }
    Ok(())
}
