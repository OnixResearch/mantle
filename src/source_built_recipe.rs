//! Source-admitted local build programs with compact bounded source controllers.
//! Serialized recipes are source, never evidence; only lowering grants authority.
//! Raw record materialization preserves captured payloads; fetcher materialization
//! restores only original pinned fetch results, never configured/generated data.
//! Commands may use a fresh declared output tree as cwd and a bounded explicit
//! normal-exit policy. Optional exit-status outputs are observed producer facts.
//! Ordered guards consume only canonical actual prior command statuses. Inactive
//! commands publish a typed decision, never outputs, execution or completion.
//! ObservePhase authenticates the actual phase inventory or an honest skipped
//! envelope; a normalized no-symlink prior Tree member may forward a checkpoint.
//! Produced-only seeds copy bounded state without modifying authority.
//! Generated invocations promise declared typed input uses; every active leaf
//! discharges that promise before launch. TextInput/TextSlice are authenticated
//! program/data bytes, not inferred file reads; actual file operands remain typed.
//! Authenticated invocation cwd selects exactly one declared typed Tree root and
//! normalized directory member; workspace aliases only expose captured output Trees.
//! Installed-image context may select only a readonly Produced Tree member whose
//! executable bytes equal the command's separately fixed Produced executable.
//! Original timeout arenas are immutable nested/disjoint step spans, not renewed
//! leaf budgets. Any expiry rejects the attempt, including optional wrappers:
//! timeout recovery is not certified and no synthetic normal code124 is granted.
//! Archive extraction retains the original root and uses shared offline decoding;
//! DirectorySeparator spells only a typed Tree directory suffix, final or before
//! an exact ':' path-list delimiter; other slash concatenation remains forbidden.
//! Controller catalogs are immutable declarations; only actual source-selected
//! turns become protected actions, resolved through the shared pure authority.
//! Cwd-relative operands retain original spelling under an authenticated cwd;
//! argv0 is process context, never executable-image authority. Inherited fd0..63
//! tables describe exact post-exec files/OFD aliases or the fixed null device.
//! Scoped process attributes only lower real file limits and retain original
//! source-authenticated monotonic timeout deadlines across continuations.
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::io::Read;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use crate::errors::RunError;
use crate::source_built_fixed_point::ProofOutputRole;
use crate::source_built_fixed_point::SourceAuthorityRole;
use crate::source_built_fixed_point::SourceBuiltFixedPointPlan;
use crate::source_built_fixed_point::StageAuthorityInput;
use crate::source_built_fixed_point_action_trust::ActionExecutable;
use crate::source_built_fixed_point_action_trust::ActionExecutionEvent;
use crate::source_built_fixed_point_action_trust::ActionInput;
use crate::source_built_fixed_point_action_trust::ActionSpec;
use crate::source_built_fixed_point_action_trust::StageActionAdapter;

#[path = "source_built_recipe_controller.rs"]
pub(crate) mod controller;

pub(crate) const SCHEMA: &str = "mantle-source-recipe-v1";
pub(super) const RECIPE_BYTES_MAX: u64 = 64 * 1024 * 1024;
pub(crate) const STEP_COUNT_MAX: usize = 100_000;
pub(super) const WORD_BYTES_MAX: usize = 65_536;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecipePreparation {
    pub tool: String,
    pub provenance: String,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum ArtifactRef {
    Source {
        role: SourceAuthorityRole,
        path: String,
    },
    ManifestSource {
        role: SourceAuthorityRole,
        record_identity: String,
        member: String,
    },
    Produced {
        action_id: String,
        output_id: String,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ArtifactKind {
    Text,
    Object,
    Archive,
    Executable,
    Tree,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecipeOutput {
    pub kind: ArtifactKind,
    pub bytes_max: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum WordPart {
    Literal {
        text: String,
    },
    Input {
        name: String,
        path: String,
    },
    Output {
        name: String,
        path: String,
    },
    Runtime {
        mount: String,
        path: String,
    },
    Workspace {
        mount: String,
        path: String,
    },
    /// A source filename resolved relative to the authenticated invocation cwd.
    /// The original spelling is retained for unchanged relative-only consumers.
    #[serde(rename = "cwd_relative")]
    CwdRelative {
        path: String,
    },
    #[serde(rename = "directory_separator")]
    DirectorySeparator {},
    /// Authenticated source program/data bytes, never an implicit file-path claim.
    #[serde(rename = "text_input")]
    TextInput {
        name: String,
    },
    #[serde(rename = "text_slice")]
    TextSlice {
        name: String,
        offset_bytes: u64,
        length_bytes: u64,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecipeWord {
    pub parts: Vec<WordPart>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum RecipeArchiveFormat {
    Tar,
    TarGz,
    TarXz,
    TarLzma,
    TarBz2,
    TarZst,
}
impl RecipeArchiveFormat {
    pub(super) fn suffix(self) -> &'static str {
        match self {
            Self::Tar => "source.tar",
            Self::TarGz => "source.tar.gz",
            Self::TarXz => "source.tar.xz",
            Self::TarLzma => "source.tar.lzma",
            Self::TarBz2 => "source.tar.bz2",
            Self::TarZst => "source.tar.zst",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecipeWorkspaceMount {
    pub output: String,
    pub path: String,
}

/// A generated request selects only declared typed roots, never executable or dependency authority.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecipeInvocation {
    pub arguments: Vec<RecipeWord>,
    pub environment: BTreeMap<String, RecipeWord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<RecipeWord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub argv0: Option<RecipeWord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub executable_path: Option<RecipeWord>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub workspace_mounts: BTreeMap<String, RecipeWorkspaceMount>,
    /// Measured readonly implicit-I/O closure, not claims of individual file reads.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub readonly_input_roots: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process_attributes: Option<RecipeProcessAttributes>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub descriptors: Option<RecipeDescriptorTable>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecipeTimeoutScope {
    pub id: String,
    pub seconds_max: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub monotonic_deadline_ns: Option<u64>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecipeProcessAttributes {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_size_bytes_max: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub umask: Option<u16>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub timeout_scopes: Vec<RecipeTimeoutScope>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum RecipeDescriptorAccess {
    Read,
    Write,
    ReadWrite,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecipeDescriptor {
    pub target: RecipeDescriptorTarget,
    pub access: RecipeDescriptorAccess,
    pub flags: u32,
    pub offset: u64,
    pub alias: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum RecipeDescriptorTarget {
    File { path: RecipeWord },
    NullDevice,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecipeDescriptorTable {
    pub slots: Vec<Option<RecipeDescriptor>>,
}

fn deserialize_runtime_mounts<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeMap<String, String>, D::Error> {
    struct Mounts;
    impl<'de> serde::de::Visitor<'de> for Mounts {
        type Value = BTreeMap<String, String>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("distinct runtime mount destinations")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(self, mut access: A) -> Result<Self::Value, A::Error> {
            let mut mounts = BTreeMap::new();
            while let Some((destination, input)) = access.next_entry::<String, String>()? {
                if mounts.insert(destination, input).is_some() {
                    return Err(serde::de::Error::custom("duplicate runtime mount destination"));
                }
            }
            Ok(mounts)
        }
    }
    deserializer.deserialize_map(Mounts)
}

/// A source-owned normal-exit contract. Probe alternatives are explicit and
/// bounded; signals, missing status and execution failures are never accepted.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum RecipeExitPolicy {
    Exact {
        code: i32,
    },
    OneOf {
        codes: Vec<i32>,
    },
    /// Record any actual normal probe status; never signals or launch failures.
    Normal,
}
impl RecipeExitPolicy {
    fn validate(&self) -> Result<(), RunError> {
        match self {
            Self::Exact { code } if (0..=255).contains(code) => Ok(()),
            Self::OneOf { codes }
                if (2..=8).contains(&codes.len())
                    && codes.iter().all(|code| (0..=255).contains(code))
                    && codes.windows(2).all(|pair| pair[0] < pair[1]) =>
            {
                Ok(())
            }
            Self::Normal => Ok(()),
            _ => Err(error("exit policy must contain bounded distinct sorted normal exit codes")),
        }
    }
    pub(super) fn accepts(&self, code: i32) -> bool {
        match self {
            Self::Exact { code: expected } => code == *expected,
            Self::OneOf { codes } => codes.binary_search(&code).is_ok(),
            Self::Normal => (0..=255).contains(&code),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecipeGuardCondition {
    pub status_input: String,
    pub accepted_codes: Vec<i32>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecipeGuard {
    pub conditions: Vec<RecipeGuardCondition>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecipePhaseState {
    pub output: String,
    pub skipped_input: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub skipped_path: String,
}
/// Executor-owned observations for an unmodified original command and workspace.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecipeCommandState {
    pub workspace_output: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prior_times_input: Option<String>,
    pub times_output: String,
    pub descriptors_output: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub descriptor_files_output: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum RecipeOperation {
    Controller {
        specification: crate::source_built_stagex_programs::controller::ControllerSpec,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        guard: Option<RecipeGuard>,
    },
    Exec {
        executable_input: String,
        arguments: Vec<RecipeWord>,
        environment: BTreeMap<String, RecipeWord>,
        #[serde(deserialize_with = "deserialize_runtime_mounts")]
        runtime_mounts: BTreeMap<String, String>,
        cwd_input: Option<String>,
        #[serde(default)]
        cwd_output: Option<String>,
        stdin_input: Option<String>,
        /// Share stdout's open-file description with stderr, preserving actual write order.
        #[serde(default)]
        merge_stderr: bool,
        #[serde(default)]
        invocation_input: Option<String>,
        exit_policy: RecipeExitPolicy,
        guard: Option<RecipeGuard>,
        #[serde(default)]
        output_seeds: BTreeMap<String, String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        command_state: Option<RecipeCommandState>,
    },
    WriteText {
        output: String,
        text: String,
    },
    CopyText {
        input: String,
        output: String,
    },
    ExtractArchive {
        input: String,
        output: String,
        format: RecipeArchiveFormat,
    },
    MaterializeRecord {
        role: SourceAuthorityRole,
        record_identity: String,
        output: String,
    },
    /// Restore an original authenticated fetcher result, including its original
    /// archive decoding and recursive/flat source pin verification.
    MaterializeFetcher {
        role: SourceAuthorityRole,
        record_identity: String,
        output: String,
    },
    ObservePhase {
        phase: String,
        output: String,
        state: Option<RecipePhaseState>,
    },
    AssembleTree {
        output: String,
        entries: BTreeMap<String, String>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecipeStep {
    pub id: String,
    pub inputs: BTreeMap<String, ArtifactRef>,
    pub outputs: BTreeMap<String, RecipeOutput>,
    pub operation: RecipeOperation,
}
/// Original source timeout arena. Expiry always fails this proof attempt;
/// optional-wrapper timeout recovery is deliberately not certified.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecipeTimeoutGroup {
    pub first_step: String,
    pub last_step: String,
    pub seconds_max: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SourceRecipe {
    pub schema: String,
    pub stage_id: String,
    pub preparation: RecipePreparation,
    pub steps: Vec<RecipeStep>,
    #[serde(default)]
    pub timeout_groups: Vec<RecipeTimeoutGroup>,
    pub roots: Vec<ArtifactRef>,
    pub exports: BTreeMap<String, ArtifactRef>,
    pub semantic_roots: BTreeMap<String, Vec<ArtifactRef>>,
}
pub(crate) struct RecipePaths<'a> {
    pub source_roots: &'a BTreeMap<SourceAuthorityRole, PathBuf>,
    pub runtime_root: &'a Path,
    pub sandbox_binary: &'a Path,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ArtifactBinding {
    pub reference: ArtifactRef,
    pub path: PathBuf,
    pub kind: ArtifactKind,
    pub bytes_max: u64,
}

pub(super) trait ArtifactBindings: for<'a> std::ops::Index<&'a ArtifactRef, Output = ArtifactBinding> {
    fn get(&self, reference: &ArtifactRef) -> Option<&ArtifactBinding>;
}
impl ArtifactBindings for BTreeMap<ArtifactRef, ArtifactBinding> {
    fn get(&self, reference: &ArtifactRef) -> Option<&ArtifactBinding> {
        BTreeMap::get(self, reference)
    }
}
pub(super) struct BindingOverlay<'a> {
    pub base: &'a BTreeMap<ArtifactRef, ArtifactBinding>,
    pub dynamic: Option<&'a BTreeMap<ArtifactRef, ArtifactBinding>>,
}
impl ArtifactBindings for BindingOverlay<'_> {
    fn get(&self, reference: &ArtifactRef) -> Option<&ArtifactBinding> {
        self.dynamic.and_then(|bindings| bindings.get(reference)).or_else(|| self.base.get(reference))
    }
}
impl std::ops::Index<&ArtifactRef> for BindingOverlay<'_> {
    type Output = ArtifactBinding;
    fn index(&self, reference: &ArtifactRef) -> &Self::Output {
        self.get(reference).expect("admitted artifact binding")
    }
}

pub(super) trait MutableArtifactBindings: ArtifactBindings {
    fn contains_key(&self, reference: &ArtifactRef) -> bool {
        self.get(reference).is_some()
    }
    fn insert(&mut self, reference: ArtifactRef, binding: ArtifactBinding) -> Option<ArtifactBinding>;
}
impl MutableArtifactBindings for BTreeMap<ArtifactRef, ArtifactBinding> {
    fn insert(&mut self, reference: ArtifactRef, binding: ArtifactBinding) -> Option<ArtifactBinding> {
        BTreeMap::insert(self, reference, binding)
    }
}
pub(super) struct MutableBindingOverlay<'a> {
    pub base: &'a BTreeMap<ArtifactRef, ArtifactBinding>,
    pub dynamic: BTreeMap<ArtifactRef, ArtifactBinding>,
}
impl ArtifactBindings for MutableBindingOverlay<'_> {
    fn get(&self, reference: &ArtifactRef) -> Option<&ArtifactBinding> {
        self.dynamic.get(reference).or_else(|| self.base.get(reference))
    }
}
impl std::ops::Index<&ArtifactRef> for MutableBindingOverlay<'_> {
    type Output = ArtifactBinding;
    fn index(&self, reference: &ArtifactRef) -> &Self::Output {
        self.get(reference).expect("admitted template binding")
    }
}
impl MutableArtifactBindings for MutableBindingOverlay<'_> {
    fn insert(&mut self, reference: ArtifactRef, binding: ArtifactBinding) -> Option<ArtifactBinding> {
        self.dynamic.insert(reference, binding)
    }
}
pub(crate) struct PreparedRecipe {
    pub(super) proof: SourceBuiltFixedPointPlan,
    pub(super) recipe: SourceRecipe,
    pub(super) adapter: StageActionAdapter,
    pub(super) sources: BTreeMap<SourceAuthorityRole, PathBuf>,
    pub(super) bindings: BTreeMap<ArtifactRef, ArtifactBinding>,
    pub(super) runtime_root: PathBuf,
    pub(super) sandbox_binary: PathBuf,
    pub(super) sandbox_digest: String,
    pub(super) host_digest: String,
    // Authenticated manifest member hydration is source materialization, not
    // generated output. Its lifetime is bound to this immutable prepared plan.
    _source_cache: tempfile::TempDir,
}
pub(crate) struct RecipeRun {
    pub outputs: BTreeMap<ArtifactRef, (PathBuf, String)>,
    pub completions: Vec<crate::source_built_recipe_exec::RecipeCompletion>,
    // Pending wiring in the source-only preparation path (Cairn prove-source-built-mantle-fixed-point I3).
    #[allow(dead_code)]
    pub phase_outcomes: BTreeMap<String, crate::source_built_fixed_point_action_trust::PhaseOutcome>,
}
impl PreparedRecipe {
    pub(crate) fn adapter(&self) -> &StageActionAdapter {
        &self.adapter
    }
    pub(crate) fn export(&self, role: &str) -> Result<&ArtifactBinding, RunError> {
        let reference = self.recipe.exports.get(role).ok_or_else(|| error(format!("missing recipe export {role}")))?;
        self.bindings.get(reference).ok_or_else(|| error("unbound recipe export"))
    }
    pub(crate) fn semantic_roots(&self) -> &BTreeMap<String, Vec<ArtifactRef>> {
        &self.recipe.semantic_roots
    }
    pub(crate) fn source_recipe(&self) -> &SourceRecipe {
        &self.recipe
    }
}
pub(crate) fn error(message: impl Into<String>) -> RunError {
    RunError::Build(format!("source recipe: {}", message.into()))
}
pub(crate) fn relative(path: &str, empty: bool) -> Result<(), RunError> {
    if (path.is_empty() && !empty)
        || path.len() > 4096
        || path.contains('\0')
        || path.contains('\\')
        || (!path.is_empty() && Path::new(path).components().any(|part| !matches!(part, Component::Normal(_))))
        || (!path.is_empty()
            && Path::new(path).components().map(|p| p.as_os_str().to_string_lossy()).collect::<Vec<_>>().join("/")
                != path)
    {
        return Err(error(format!("non-normalized relative path {path:?}")));
    }
    Ok(())
}
fn label(value: &str) -> Result<(), RunError> {
    if value.is_empty()
        || matches!(value, "." | "..")
        || value.len() > 256
        || !value.bytes().all(|b| b.is_ascii_alphanumeric() || b"-_:.".contains(&b))
    {
        return Err(error("invalid recipe label"));
    }
    Ok(())
}
fn action_id(stage: &str, step: &str) -> String {
    format!("{stage}/{step}")
}
fn reference_dependencies(reference: &ArtifactRef) -> Option<&str> {
    match reference {
        ArtifactRef::Produced { action_id, .. } => Some(action_id),
        _ => None,
    }
}
fn validate_ref(reference: &ArtifactRef) -> Result<(), RunError> {
    match reference {
        ArtifactRef::Source { path, .. } => relative(path, true),
        ArtifactRef::ManifestSource {
            record_identity,
            member,
            ..
        } => {
            label(record_identity)?;
            relative(member, true)
        }
        ArtifactRef::Produced { action_id, output_id } => {
            if action_id.is_empty() || action_id.contains('\0') {
                return Err(error("empty producer action"));
            }
            label(output_id)
        }
    }
}
pub(super) fn timeout_spans(recipe: &SourceRecipe) -> Result<Vec<(usize, usize, u64)>, RunError> {
    if recipe.timeout_groups.len() > STEP_COUNT_MAX {
        return Err(error("excessive source timeout groups"));
    }
    let positions = recipe
        .steps
        .iter()
        .enumerate()
        .map(|(index, step)| (step.id.as_str(), index))
        .collect::<BTreeMap<_, _>>();
    let mut spans = Vec::with_capacity(recipe.timeout_groups.len());
    for group in &recipe.timeout_groups {
        label(&group.first_step)?;
        label(&group.last_step)?;
        let first = *positions.get(group.first_step.as_str()).ok_or_else(|| error("unknown timeout first step"))?;
        let last = *positions.get(group.last_step.as_str()).ok_or_else(|| error("unknown timeout last step"))?;
        if first > last || group.seconds_max == 0 {
            return Err(error("source timeout must be positive with ordered endpoints"));
        }
        spans.push((first, last, group.seconds_max));
    }
    spans.sort_unstable_by_key(|&(first, last, seconds)| (first, std::cmp::Reverse(last), seconds));
    let mut enclosing = Vec::<usize>::new();
    for &(first, last, _) in &spans {
        while enclosing.last().is_some_and(|end| *end < first) {
            enclosing.pop();
        }
        if enclosing.last().is_some_and(|end| last > *end) {
            return Err(error("crossing source timeout spans"));
        }
        enclosing.push(last);
    }
    Ok(spans)
}

/// Validate the SOURCE program, without granting executable/output authority.
/// A producer-first order is required; reachability is independently computed
/// backwards from requested products, not copied from the resulting action list.
pub(crate) fn validate_source_recipe(recipe: &SourceRecipe) -> Result<(), RunError> {
    if recipe.schema != SCHEMA
        || recipe.steps.is_empty()
        || recipe.steps.len() > STEP_COUNT_MAX
        || recipe.roots.is_empty()
        || recipe.preparation.tool.trim().is_empty()
        || recipe.preparation.provenance.trim().is_empty()
    {
        return Err(error("invalid schema, preparation provenance, or recipe bounds"));
    }
    label(&recipe.stage_id)?;
    timeout_spans(recipe)?;
    let mut producers = BTreeMap::<String, &RecipeStep>::new();
    for step in &recipe.steps {
        label(&step.id)?;
        if step.id.ends_with(".sandbox") || step.outputs.is_empty() {
            return Err(error("reserved step ID or empty output set"));
        }
        let id = action_id(&recipe.stage_id, &step.id);
        for (name, input) in &step.inputs {
            label(name)?;
            validate_ref(input)?;
            if let ArtifactRef::Produced { action_id, output_id } = input
                && action_id.strip_prefix(&recipe.stage_id).is_some_and(|tail| tail.starts_with('/'))
            {
                let producer = producers
                    .get(action_id)
                    .ok_or_else(|| error(format!("unknown, cyclic, or late producer {action_id}")))?;
                if !producer.outputs.contains_key(output_id) {
                    return Err(error("unknown produced input output"));
                }
                validate_guarded_reference(step, name, input, producer)?;
            }
        }
        for (name, output) in &step.outputs {
            label(name)?;
            if output.bytes_max == 0 {
                return Err(error("zero output bound"));
            }
        }
        validate_operation(step)?;
        validate_control_edges(step, &recipe.stage_id, &producers)?;
        if producers.insert(id, step).is_some() {
            return Err(error("duplicate recipe step"));
        }
    }
    for reference in recipe.roots.iter().chain(recipe.exports.values()).chain(recipe.semantic_roots.values().flatten())
    {
        if let ArtifactRef::Produced { action_id, .. } = reference
            && producers.get(action_id).is_some_and(|step| operation_guard(&step.operation).is_some())
        {
            return Err(error("guarded command outputs require explicit phase observation, not roots or exports"));
        }
    }
    let mut reachable = BTreeSet::<String>::new();
    let mut queue = Vec::new();
    for reference in &recipe.roots {
        validate_ref(reference)?;
        let ArtifactRef::Produced { action_id, output_id } = reference else {
            return Err(error("recipe root must be produced, not imported source"));
        };
        let producer = producers.get(action_id).ok_or_else(|| error("recipe root must name a local producer"))?;
        if !producer.outputs.contains_key(output_id) {
            return Err(error("unknown root output"));
        }
        queue.push(action_id.as_str());
    }
    while let Some(id) = queue.pop() {
        if reachable.insert(id.to_owned()) {
            let producer = producers[id];
            queue.extend(
                producer.inputs.values().filter_map(reference_dependencies).filter(|id| producers.contains_key(*id)),
            );
            if let RecipeOperation::ObservePhase { phase, .. } = &producer.operation {
                let phase_id = action_id(&recipe.stage_id, phase);
                let (id, _) = producers.get_key_value(&phase_id).ok_or_else(|| error("unknown observed phase"))?;
                queue.push(id.as_str());
            }
        }
    }
    if reachable.len() != recipe.steps.len() {
        return Err(error("unreachable or omitted command topology"));
    }
    for refs in recipe
        .exports
        .values()
        .map(std::slice::from_ref)
        .chain(recipe.semantic_roots.values().map(Vec::as_slice))
    {
        if refs.is_empty() {
            return Err(error("empty semantic root"));
        }
        for reference in refs {
            let ArtifactRef::Produced { action_id, output_id } = reference else {
                return Err(error("export/semantic root is imported output"));
            };
            if !reachable.contains(action_id)
                || !producers.get(action_id).is_some_and(|s| s.outputs.contains_key(output_id))
            {
                return Err(error("unknown or unreachable export/semantic output"));
            }
        }
    }
    Ok(())
}
fn input_exists(step: &RecipeStep, name: &str) -> Result<(), RunError> {
    if step.inputs.contains_key(name) {
        Ok(())
    } else {
        Err(error(format!("unknown input {name}")))
    }
}
fn output_kind(step: &RecipeStep, name: &str, kind: ArtifactKind) -> Result<(), RunError> {
    if step.outputs.get(name).is_some_and(|o| o.kind == kind) {
        Ok(())
    } else {
        Err(error("source operation has invalid output kind"))
    }
}
pub(super) fn operation_guard(operation: &RecipeOperation) -> Option<&RecipeGuard> {
    match operation {
        RecipeOperation::Exec { guard, .. } | RecipeOperation::Controller { guard, .. } => guard.as_ref(),
        _ => None,
    }
}
pub(super) fn is_guard_input(step: &RecipeStep, name: &str) -> bool {
    operation_guard(&step.operation).is_some_and(|guard| {
        guard.conditions.iter().any(|condition| {
            condition.status_input == name || step.inputs.get(&condition.status_input) == step.inputs.get(name)
        })
    })
}
fn lower_guard(
    step: &RecipeStep,
) -> Result<Option<crate::source_built_fixed_point_action_trust::ActionGuard>, RunError> {
    use crate::source_built_fixed_point_action_trust::ActionGuard;
    use crate::source_built_fixed_point_action_trust::ActionGuardCondition;
    use crate::source_built_fixed_point_action_trust::valid_guard_codes;
    let Some(guard) = operation_guard(&step.operation) else {
        return Ok(None);
    };
    if !(1..=8).contains(&guard.conditions.len()) {
        return Err(error("guard requires one to eight ordered conditions"));
    }
    let mut conditions = Vec::with_capacity(guard.conditions.len());
    let mut statuses = BTreeSet::new();
    for condition in &guard.conditions {
        if !valid_guard_codes(&condition.accepted_codes) {
            return Err(error("guard requires sorted distinct bounded normal exit codes"));
        }
        let Some(ArtifactRef::Produced { action_id, output_id }) = step.inputs.get(&condition.status_input) else {
            return Err(error("guard status must be a declared Produced input"));
        };
        if output_id != "exit-status" || !statuses.insert(action_id) {
            return Err(error("guard must consume distinct actual exit-status outputs"));
        }
        conditions.push(ActionGuardCondition {
            action_id: action_id.clone(),
            accepted_codes: condition.accepted_codes.clone(),
        });
    }
    Ok(Some(ActionGuard { conditions }))
}
fn validate_guarded_reference(
    step: &RecipeStep,
    _name: &str,
    reference: &ArtifactRef,
    producer: &RecipeStep,
) -> Result<(), RunError> {
    let ArtifactRef::Produced { action_id, output_id } = reference else {
        return Ok(());
    };
    let ordinary_output = !["stdout", "stderr", "exit-status"].contains(&output_id.as_str());
    let producer_guard = lower_guard(producer)?;
    let optional_failure = ordinary_output
        && matches!(&producer.operation, RecipeOperation::Exec {
            exit_policy: RecipeExitPolicy::OneOf { .. } | RecipeExitPolicy::Normal,
            ..
        })
        && producer.outputs.contains_key("exit-status");
    if producer_guard.is_none() && !optional_failure {
        return Ok(());
    }
    let Some(guard) = lower_guard(step)? else {
        return Err(error("guarded outputs require explicit phase observation or a dominating command guard"));
    };
    let Some(index) = guard.conditions.iter().position(|condition| condition.action_id == *action_id) else {
        return Err(error("guarded output requires its producer's actual exit-status predicate"));
    };
    if producer_guard
        .as_ref()
        .is_some_and(|producer_guard| !guard.conditions[..index].starts_with(&producer_guard.conditions))
    {
        return Err(error("guarded output requires an exactly dominating prior guard prefix"));
    }
    Ok(())
}
fn validate_control_edges(
    step: &RecipeStep,
    stage: &str,
    preceding: &BTreeMap<String, &RecipeStep>,
) -> Result<(), RunError> {
    if let Some(guard) = lower_guard(step)? {
        for condition in guard.conditions {
            let producer = preceding
                .get(&condition.action_id)
                .ok_or_else(|| error("guard status must name a prior local command"))?;
            if !matches!(producer.operation, RecipeOperation::Exec { .. } | RecipeOperation::Controller { .. })
                || !producer.outputs.get("exit-status").is_some_and(|output| output.kind == ArtifactKind::Text)
            {
                return Err(error("guard status is not an actual command's Text exit-status"));
            }
        }
    }
    if let RecipeOperation::ObservePhase { phase, state, .. } = &step.operation {
        let producer = preceding
            .get(&action_id(stage, phase))
            .ok_or_else(|| error("phase observation requires a prior local guarded command"))?;
        if operation_guard(&producer.operation).is_none() {
            return Err(error("phase observation requires a guarded command"));
        }
        if let Some(state) = state {
            output_kind(producer, &state.output, ArtifactKind::Tree)?;
            if let ArtifactRef::Produced { action_id: prior, .. } = &step.inputs[&state.skipped_input]
                && prior == &action_id(stage, phase)
            {
                return Err(error("skipped state cannot come from the skipped phase"));
            }
        }
    }
    Ok(())
}
fn validate_operation(step: &RecipeStep) -> Result<(), RunError> {
    match &step.operation {
        RecipeOperation::Controller { specification, .. } => controller::validate(step, specification)?,
        RecipeOperation::Exec {
            executable_input,
            arguments,
            environment,
            runtime_mounts,
            cwd_input,
            cwd_output,
            stdin_input,
            merge_stderr: _,
            invocation_input,
            exit_policy,
            guard: _,
            output_seeds,
            command_state,
        } => {
            input_exists(step, executable_input)?;
            lower_guard(step)?;
            if let Some(name) = invocation_input {
                input_exists(step, name)?;
                if !arguments.is_empty()
                    || !environment.is_empty()
                    || is_guard_input(step, name)
                    || !matches!(&step.inputs[name], ArtifactRef::Produced { output_id, .. } if output_id != "exit-status")
                    || !matches!(step.inputs[executable_input], ArtifactRef::Produced { .. })
                {
                    return Err(error(
                        "dynamic invocation requires a genuine ordinary Produced request and empty static arguments/environment",
                    ));
                }
            }
            for name in cwd_input
                .iter()
                .chain(stdin_input.iter())
                .chain(runtime_mounts.values())
                .chain(std::iter::once(executable_input))
            {
                if is_guard_input(step, name) {
                    return Err(error("guard status is control authority, not a command mount"));
                }
            }
            for (output, name) in output_seeds {
                input_exists(step, name)?;
                if !step.outputs.contains_key(output)
                    || ["stdout", "stderr", "exit-status"].contains(&output.as_str())
                    || !matches!(step.inputs[name], ArtifactRef::Produced { .. })
                    || is_guard_input(step, name)
                {
                    return Err(error("output seed must map an ordinary output to genuine Produced input"));
                }
            }
            if let Some(state) = command_state {
                output_kind(step, &state.workspace_output, ArtifactKind::Tree)?;
                for output in [&state.times_output, &state.descriptors_output] {
                    output_kind(step, output, ArtifactKind::Text)?;
                    if ["stdout", "stderr", "exit-status"].contains(&output.as_str())
                        || output_seeds.contains_key(output)
                    {
                        return Err(error("command observations require fresh nonreserved Text outputs"));
                    }
                }
                if state.times_output == state.descriptors_output {
                    return Err(error("command time and descriptor observations require distinct outputs"));
                }
                if let Some(output) = &state.descriptor_files_output {
                    output_kind(step, output, ArtifactKind::Tree)?;
                    if output == &state.workspace_output
                        || output == &state.times_output
                        || output == &state.descriptors_output
                    {
                        return Err(error("retained descriptor snapshots require their own declared Tree output"));
                    }
                }
                if let Some(name) = &state.prior_times_input {
                    input_exists(step, name)?;
                    if is_guard_input(step, name) || !matches!(step.inputs[name], ArtifactRef::Produced { .. }) {
                        return Err(error("prior command times require an actual Produced Text input"));
                    }
                }
            }
            let is_seed = matches!(step.inputs.get(executable_input),Some(ArtifactRef::Source{role:SourceAuthorityRole::StagexSeed,path}) if path.is_empty());
            if !is_seed && !matches!(step.inputs.get(executable_input), Some(ArtifactRef::Produced { .. })) {
                return Err(error("source executable is not the exact StageX seed"));
            }
            exit_policy.validate()?;
            if let Some(status) = step.outputs.get("exit-status")
                && (status.kind != ArtifactKind::Text || status.bytes_max < 4)
            {
                return Err(error("actual exit-status output must be Text with at least four bytes"));
            }
            if matches!(exit_policy, RecipeExitPolicy::Normal) && !step.outputs.contains_key("exit-status") {
                return Err(error("normal probe observation requires its actual exit-status output"));
            }
            if cwd_input.is_some() && cwd_output.is_some() {
                return Err(error("command cwd cannot name both an input and an output"));
            }
            if let Some(name) = cwd_output {
                output_kind(step, name, ArtifactKind::Tree)?;
            }
            for name in cwd_input.iter().chain(stdin_input.iter()) {
                input_exists(step, name)?;
            }
            for (destination, input) in runtime_mounts {
                let path = destination.strip_prefix('/').ok_or_else(|| error("runtime mount is not absolute"))?;
                relative(path, false)?;
                input_exists(step, input)?;
                if ["inputs", "outputs", "work", "proc", "dev", "tmp"].iter().any(|reserved| {
                    path == *reserved || path.strip_prefix(*reserved).is_some_and(|tail| tail.starts_with('/'))
                }) {
                    return Err(error("runtime mount shadows a reserved sandbox namespace"));
                }
                if !matches!(step.inputs[input], ArtifactRef::Produced { .. }) {
                    return Err(error("runtime loader/library must have a real producer"));
                }
                if Path::new(destination)
                    .ancestors()
                    .skip(1)
                    .filter_map(Path::to_str)
                    .any(|parent| runtime_mounts.contains_key(parent))
                {
                    return Err(error("runtime mount destinations duplicate or overlap an ancestor"));
                }
            }
            let stdout =
                step.outputs.get("stdout").ok_or_else(|| error("command must publish actual bounded stdout"))?;
            if !matches!(stdout.kind, ArtifactKind::Text | ArtifactKind::Object | ArtifactKind::Archive) {
                return Err(error("stdout must be a regular text/binary artifact, not a tree or executable"));
            }
            output_kind(step, "stderr", ArtifactKind::Text)?;
            for word in arguments.iter().chain(environment.values()) {
                validate_word(step, word)?;
            }
            for name in environment.keys() {
                if name.is_empty() || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
                    return Err(error("invalid environment name"));
                }
            }
        }
        RecipeOperation::WriteText { output, text } => {
            output_kind(step, output, ArtifactKind::Text)?;
            validate_text(text)?;
            if text.len() as u64 > step.outputs[output].bytes_max {
                return Err(error("text exceeds declared output bound"));
            }
        }
        RecipeOperation::CopyText { input, output } => {
            input_exists(step, input)?;
            output_kind(step, output, ArtifactKind::Text)?;
        }
        RecipeOperation::ExtractArchive { input, output, .. } => {
            input_exists(step, input)?;
            output_kind(step, output, ArtifactKind::Tree)?;
        }
        RecipeOperation::MaterializeRecord {
            record_identity,
            output,
            ..
        } => {
            label(record_identity)?;
            output_kind(step, output, ArtifactKind::Tree)?;
        }
        RecipeOperation::MaterializeFetcher {
            record_identity,
            output,
            ..
        } => {
            label(record_identity)?;
            if !step.outputs.get(output).is_some_and(|o| matches!(o.kind, ArtifactKind::Tree | ArtifactKind::Text)) {
                return Err(error("fetcher source output must be a tree or text artifact"));
            }
        }
        RecipeOperation::ObservePhase { phase, output, state } => {
            label(phase)?;
            output_kind(step, output, ArtifactKind::Tree)?;
            if let Some(state) = state {
                label(&state.output)?;
                relative(&state.skipped_path, true)?;
                input_exists(step, &state.skipped_input)?;
                if !matches!(step.inputs[&state.skipped_input], ArtifactRef::Produced { .. }) {
                    return Err(error("skipped prior state requires a genuine Produced Tree input"));
                }
            }
        }
        RecipeOperation::AssembleTree { output, entries } => {
            output_kind(step, output, ArtifactKind::Tree)?;
            if entries.is_empty() {
                return Err(error("empty assembled tree"));
            }
            for (path, name) in entries {
                relative(path, false)?;
                input_exists(step, name)?;
            }
        }
    }
    if !matches!(step.operation, RecipeOperation::Exec { .. } | RecipeOperation::Controller { .. })
        && step.outputs.len() != 1
    {
        return Err(error("source operation requires one exact output"));
    }
    if !matches!(step.operation, RecipeOperation::Exec { .. } | RecipeOperation::Controller { .. })
        && step.outputs.contains_key("exit-status")
    {
        return Err(error("reserved exit-status requires an actual command producer"));
    }
    Ok(())
}
fn validate_word(step: &RecipeStep, word: &RecipeWord) -> Result<(), RunError> {
    validate_word_scoped(step, word, None)
}
fn validate_word_scoped(
    step: &RecipeStep,
    word: &RecipeWord,
    workspaces: Option<&BTreeMap<String, RecipeWorkspaceMount>>,
) -> Result<(), RunError> {
    if word.parts.is_empty() || word.parts.len() > 4096 {
        return Err(error("empty or excessive command word"));
    }
    let mut bytes = 0usize;
    for (index, part) in word.parts.iter().enumerate() {
        match part {
            WordPart::Literal { text } => {
                bytes = bytes.saturating_add(text.len());
                // All filesystem paths are typed. This also rejects response-file
                // paths and embedded -I/host or --sysroot=/host spellings.
                if text.contains('/') || text.contains('\\') || text.contains('\0') {
                    return Err(error("untyped filesystem path in literal command word"));
                }
            }
            WordPart::Input { name, path } => {
                input_exists(step, name)?;
                if is_guard_input(step, name) {
                    return Err(error("guard status cannot be used as an ordinary command argument"));
                }
                relative(path, true)?;
            }
            WordPart::Output { name, path } => {
                if name == "exit-status" {
                    return Err(error("actual exit-status is executor-owned, not a command output path"));
                }
                if let RecipeOperation::Exec {
                    command_state: Some(state),
                    ..
                } = &step.operation
                    && (name == &state.times_output || name == &state.descriptors_output)
                {
                    return Err(error("command state observations are executor-owned, not command output paths"));
                }
                if !step.outputs.contains_key(name) {
                    return Err(error("unknown word output"));
                }
                relative(path, true)?;
            }
            WordPart::Runtime { mount, path } => {
                let RecipeOperation::Exec { runtime_mounts, .. } = &step.operation else {
                    return Err(error("runtime word requires a command"));
                };
                if !runtime_mounts.contains_key(mount) {
                    return Err(error("runtime word must name an exact declared mount"));
                }
                relative(path, true)?;
            }
            WordPart::Workspace { mount, path } => {
                if !workspaces.is_some_and(|mounts| mounts.contains_key(mount)) {
                    return Err(error("workspace word must name an exact invocation workspace mount"));
                }
                relative(path, true)?;
            }
            WordPart::CwdRelative { path } => {
                let member = path.strip_prefix("./").unwrap_or(path);
                relative(member, false)?;
                if word.parts.len() != 1 || workspaces.is_none() {
                    return Err(error(
                        "cwd-relative filename requires one whole operand in an authenticated invocation",
                    ));
                }
            }
            WordPart::DirectorySeparator {} => {
                let boundary = index + 1 == word.parts.len()
                    || matches!(word.parts.get(index + 1), Some(WordPart::Literal { text }) if text == ":" || text == ",");
                if !boundary
                    || index == 0
                    || !matches!(
                        word.parts[index - 1],
                        WordPart::Input { .. }
                            | WordPart::Output { .. }
                            | WordPart::Runtime { .. }
                            | WordPart::Workspace { .. }
                    )
                {
                    return Err(error(
                        "directory separator must follow a typed path and end a directory or precede exact ':' or ','",
                    ));
                }
                bytes = bytes.saturating_add(1);
            }
            WordPart::TextInput { name } | WordPart::TextSlice { name, .. } => {
                input_exists(step, name)?;
                if is_guard_input(step, name)
                    || !matches!(&step.inputs[name], ArtifactRef::Produced { output_id, .. } if output_id != "exit-status")
                {
                    return Err(error(
                        "text operand requires ordinary Produced Text, never exit-status or guard control",
                    ));
                }
            }
        }
        if let WordPart::TextSlice {
            offset_bytes,
            length_bytes,
            ..
        } = part
            && (offset_bytes.checked_add(*length_bytes).is_none_or(|end| end > RECIPE_BYTES_MAX)
                || *length_bytes > WORD_BYTES_MAX as u64)
        {
            return Err(error("text slice exceeds checked source/word byte bounds"));
        }
    }
    if bytes > WORD_BYTES_MAX {
        return Err(error("oversized command word"));
    }
    Ok(())
}

fn validate_word_binding(
    step: &RecipeStep,
    word: &RecipeWord,
    bindings: &impl ArtifactBindings,
) -> Result<(), RunError> {
    validate_word_binding_scoped(step, word, bindings, None)
}
fn validate_word_binding_scoped(
    step: &RecipeStep,
    word: &RecipeWord,
    bindings: &impl ArtifactBindings,
    workspaces: Option<&BTreeMap<String, RecipeWorkspaceMount>>,
) -> Result<(), RunError> {
    for (index, part) in word.parts.iter().enumerate() {
        match part {
            WordPart::Input { name, path }
                if !path.is_empty() && bindings[&step.inputs[name]].kind != ArtifactKind::Tree =>
            {
                return Err(error("input subpath requires a declared tree artifact"));
            }
            WordPart::Output { name, path } if !path.is_empty() && step.outputs[name].kind != ArtifactKind::Tree => {
                return Err(error("output subpath requires a declared tree artifact"));
            }
            WordPart::TextInput { name } | WordPart::TextSlice { name, .. }
                if bindings[&step.inputs[name]].kind != ArtifactKind::Text =>
            {
                return Err(error("text operand requires exact Produced Text kind"));
            }
            WordPart::Runtime { mount, path } if !path.is_empty() => {
                let RecipeOperation::Exec { runtime_mounts, .. } = &step.operation else {
                    return Err(error("runtime word requires a command"));
                };
                if bindings[&step.inputs[&runtime_mounts[mount]]].kind != ArtifactKind::Tree {
                    return Err(error("runtime subpath requires an exact declared Tree mount"));
                }
            }
            WordPart::DirectorySeparator {} => {
                let prior = index
                    .checked_sub(1)
                    .and_then(|prior| word.parts.get(prior))
                    .ok_or_else(|| error("directory separator lacks a typed path"))?;
                let kind = typed_root_kind(step, prior, bindings, workspaces)?;
                if kind != ArtifactKind::Tree {
                    return Err(error("directory separator requires a Tree root binding"));
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn typed_root_kind(
    step: &RecipeStep,
    part: &WordPart,
    bindings: &impl ArtifactBindings,
    workspaces: Option<&BTreeMap<String, RecipeWorkspaceMount>>,
) -> Result<ArtifactKind, RunError> {
    let input_kind = |name: &str| {
        step.inputs
            .get(name)
            .and_then(|reference| bindings.get(reference))
            .map(|binding| binding.kind)
            .ok_or_else(|| error("typed path lacks an existing input binding"))
    };
    match part {
        WordPart::Input { name, .. } => input_kind(name),
        WordPart::Output { name, .. } => step
            .outputs
            .get(name)
            .map(|output| output.kind)
            .ok_or_else(|| error("typed path lacks an existing output")),
        WordPart::Runtime { mount, .. } => {
            let RecipeOperation::Exec { runtime_mounts, .. } = &step.operation else {
                return Err(error("runtime path requires a command"));
            };
            input_kind(runtime_mounts.get(mount).ok_or_else(|| error("unknown runtime mount"))?)
        }
        WordPart::Workspace { mount, .. } => {
            let mapping =
                workspaces.and_then(|mounts| mounts.get(mount)).ok_or_else(|| error("unknown workspace mount"))?;
            step.outputs
                .get(&mapping.output)
                .map(|output| output.kind)
                .ok_or_else(|| error("workspace lacks a declared output"))
        }
        _ => Err(error("cwd/directory boundary requires a typed path")),
    }
}

fn path_overlap(left: &str, right: &str) -> bool {
    left == right
        || left.strip_prefix(right).is_some_and(|tail| tail.starts_with('/'))
        || right.strip_prefix(left).is_some_and(|tail| tail.starts_with('/'))
}

pub(super) fn validate_workspace_mounts(
    step: &RecipeStep,
    invocation: &RecipeInvocation,
    store_prefix: Option<&str>,
) -> Result<(), RunError> {
    let RecipeOperation::Exec { runtime_mounts, .. } = &step.operation else {
        return Err(error("workspace mounts require a command"));
    };
    for (destination, mapping) in &invocation.workspace_mounts {
        relative(destination.strip_prefix('/').ok_or_else(|| error("workspace destination must be absolute"))?, false)?;
        if ["/inputs", "/outputs", "/work", "/proc", "/dev"].iter().any(|reserved| {
            destination == *reserved || destination.strip_prefix(*reserved).is_some_and(|tail| tail.starts_with('/'))
        }) {
            return Err(error("workspace mount shadows a protected namespace"));
        }
        if let Some(prefix) = store_prefix {
            let original_store =
                destination.strip_prefix(prefix).is_some_and(|tail| tail.starts_with('/') && tail.len() > 1);
            if destination != "/tmp" && destination != "/build" && !original_store {
                return Err(error("workspace destination lies outside original writable namespaces"));
            }
        }
        relative(&mapping.path, true)?;
        output_kind(step, &mapping.output, ArtifactKind::Tree)?;
        let overlapping_workspace = destination
            .match_indices('/')
            .skip(1)
            .any(|(index, _)| invocation.workspace_mounts.contains_key(&destination[..index]));
        if overlapping_workspace || runtime_mounts.keys().any(|other| path_overlap(destination, other)) {
            return Err(error("workspace mounts overlap each other or a read-only runtime mount"));
        }
    }
    Ok(())
}

/// Dynamic dependency declarations are conditional promises, discharged before
/// each active leaf: every input needs an explicit typed use or readonly catalog.
/// Skipped leaves never decode a request or claim to consume absent inputs.
pub(super) fn decode_invocation(
    step: &RecipeStep,
    bytes: &[u8],
    bindings: &impl ArtifactBindings,
) -> Result<RecipeInvocation, RunError> {
    if bytes.len() as u64 > RECIPE_BYTES_MAX {
        return Err(error("invocation request exceeds source byte bound"));
    }
    let invocation: RecipeInvocation =
        serde_json::from_slice(bytes).map_err(|e| error(format!("invocation syntax: {e}")))?;
    struct Canonical<'a> {
        bytes: &'a [u8],
        position: usize,
    }
    impl std::io::Write for Canonical<'_> {
        fn write(&mut self, chunk: &[u8]) -> std::io::Result<usize> {
            let end = self
                .position
                .checked_add(chunk.len())
                .ok_or_else(|| std::io::Error::other("invocation length overflow"))?;
            if self.bytes.get(self.position..end) != Some(chunk) {
                return Err(std::io::Error::other("noncanonical invocation"));
            }
            self.position = end;
            Ok(chunk.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut canonical = Canonical { bytes, position: 0 };
    serde_json::to_writer(&mut canonical, &invocation).map_err(|e| error(format!("noncanonical invocation: {e}")))?;
    if canonical.position != bytes.len() {
        return Err(error("noncanonical invocation trailing bytes"));
    }
    let RecipeOperation::Exec {
        executable_input,
        runtime_mounts,
        cwd_input,
        cwd_output,
        stdin_input,
        invocation_input: Some(request),
        output_seeds,
        command_state,
        guard,
        ..
    } = &step.operation
    else {
        return Err(error("invocation decoder requires a dynamic command"));
    };
    validate_workspace_mounts(step, &invocation, None)?;
    if invocation
        .arguments
        .iter()
        .chain(invocation.environment.values())
        .chain(invocation.argv0.iter())
        .any(|word| word.parts.iter().any(|part| matches!(part, WordPart::CwdRelative { .. })))
        && invocation.cwd.is_none()
    {
        return Err(error("cwd-relative operand lacks an authenticated typed cwd"));
    }
    if let Some(attributes) = &invocation.process_attributes {
        if attributes.file_size_bytes_max == Some(0)
            || attributes.umask.is_some_and(|mask| mask > 0o777)
            || attributes.timeout_scopes.len() > 64
        {
            return Err(error("invalid source process attributes"));
        }
        let mut scopes = BTreeSet::new();
        for scope in &attributes.timeout_scopes {
            relative(&scope.id, false)?;
            if scope.seconds_max == 0 || scope.monotonic_deadline_ns.is_none() || !scopes.insert(&scope.id) {
                return Err(error("invalid, unanchored or duplicate original timeout scope"));
            }
        }
    }
    if let Some(cwd) = &invocation.cwd {
        if cwd_input.is_some() || cwd_output.is_some() {
            return Err(error("invocation cwd conflicts with a static cwd anchor"));
        }
        if cwd.parts.len() != 1
            || !matches!(
                cwd.parts[0],
                WordPart::Input { .. }
                    | WordPart::Output { .. }
                    | WordPart::Runtime { .. }
                    | WordPart::Workspace { .. }
            )
        {
            return Err(error("invocation cwd must contain exactly one typed directory path"));
        }
        validate_word_scoped(step, cwd, Some(&invocation.workspace_mounts))?;
        if typed_root_kind(step, &cwd.parts[0], bindings, Some(&invocation.workspace_mounts))? != ArtifactKind::Tree {
            return Err(error("invocation cwd requires an existing Tree root"));
        }
    }
    if let Some(image) = &invocation.executable_path {
        if !matches!(step.inputs[executable_input], ArtifactRef::Produced { .. }) || image.parts.len() != 1 {
            return Err(error(
                "installed image path requires an exact Produced executable and one readonly typed Tree member",
            ));
        }
        let (name, path) = match &image.parts[0] {
            WordPart::Input { name, path } => (name, path),
            WordPart::Runtime { mount, path } => {
                (runtime_mounts.get(mount).ok_or_else(|| error("unknown installed image runtime mount"))?, path)
            }
            _ => return Err(error("installed image path must use an Input or Runtime readonly Tree")),
        };
        relative(path, false)?;
        input_exists(step, name)?;
        if !matches!(step.inputs[name], ArtifactRef::Produced { .. })
            || typed_root_kind(step, &image.parts[0], bindings, None)? != ArtifactKind::Tree
        {
            return Err(error("installed image root must be a readonly Produced Tree"));
        }
    }
    let mut used = BTreeSet::new();
    used.insert(executable_input.as_str());
    used.insert(request.as_str());
    used.extend(runtime_mounts.values().map(String::as_str));
    used.extend(cwd_input.iter().chain(stdin_input.iter()).map(String::as_str));
    used.extend(output_seeds.values().map(String::as_str));
    if let Some(state) = command_state {
        used.extend(state.prior_times_input.iter().map(String::as_str));
    }
    if let Some(guard) = guard {
        used.extend(guard.conditions.iter().map(|condition| condition.status_input.as_str()));
    }
    if invocation.readonly_input_roots.len() > step.inputs.len()
        || invocation.readonly_input_roots.windows(2).any(|pair| pair[0] >= pair[1])
    {
        return Err(error("readonly input roots must be sorted unique declared names"));
    }
    for name in &invocation.readonly_input_roots {
        input_exists(step, name)?;
        let reference = &step.inputs[name];
        if !matches!(reference, ArtifactRef::Produced { .. })
            || bindings.get(reference).map(|binding| binding.kind) != Some(ArtifactKind::Tree)
        {
            return Err(error("readonly input roots require exact Produced Tree inputs"));
        }
        used.insert(name.as_str());
    }
    let descriptor_words = invocation.descriptors.iter().flat_map(|table| {
        table.slots.iter().flatten().filter_map(|descriptor| match &descriptor.target {
            RecipeDescriptorTarget::File { path } => Some(path),
            RecipeDescriptorTarget::NullDevice => None,
        })
    });
    if let Some(table) = &invocation.descriptors {
        if command_state.is_none() {
            return Err(error("actual descriptor inheritance requires executor-owned descriptor observations"));
        }
        if table.slots.len() != 64 {
            return Err(error("actual descriptor table must declare all 64 open or closed slots"));
        }
        for (index, entry) in table.slots.iter().enumerate() {
            let Some(descriptor) = entry else {
                continue;
            };
            if descriptor.alias as usize > index
                || descriptor.offset > i64::MAX as u64
                || descriptor.flags & !((libc::O_APPEND | libc::O_NONBLOCK) as u32) != 0
            {
                return Err(error("invalid actual descriptor path, flags, offset or alias"));
            }
            if let RecipeDescriptorTarget::File { path } = &descriptor.target {
                if path.parts.len() != 1 {
                    return Err(error("descriptor file requires one exact typed path"));
                }
                match &path.parts[0] {
                    WordPart::Input { .. } | WordPart::Runtime { .. }
                        if descriptor.access == RecipeDescriptorAccess::Read => {}
                    WordPart::Output { .. } | WordPart::Workspace { .. } => {}
                    _ => {
                        return Err(error(
                            "descriptor inheritance requires exact typed file authority, writable only in outputs",
                        ));
                    }
                }
            }
            if descriptor.alias as usize != index && table.slots[descriptor.alias as usize].as_ref() != Some(descriptor)
            {
                return Err(error("descriptor aliases must retain an identical prior primary OFD"));
            }
        }
    }
    for word in invocation
        .arguments
        .iter()
        .chain(invocation.environment.values())
        .chain(invocation.cwd.iter())
        .chain(invocation.argv0.iter())
        .chain(invocation.executable_path.iter())
        .chain(descriptor_words)
    {
        validate_word_scoped(step, word, Some(&invocation.workspace_mounts))?;
        validate_word_binding_scoped(step, word, bindings, Some(&invocation.workspace_mounts))?;
        for part in &word.parts {
            match part {
                WordPart::Input { name, .. } | WordPart::TextInput { name } | WordPart::TextSlice { name, .. } => {
                    used.insert(name.as_str());
                }
                WordPart::Runtime { mount, .. } => {
                    used.insert(runtime_mounts[mount].as_str());
                }
                _ => {}
            }
        }
    }
    for name in invocation.environment.keys() {
        if name.is_empty() || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
            return Err(error("invalid environment name"));
        }
    }
    if step.inputs.keys().any(|name| !used.contains(name.as_str())) {
        return Err(error("dynamic invocation has an unused declared input name"));
    }
    drop(used);
    Ok(invocation)
}

pub(crate) fn validate_text(text: &str) -> Result<(), RunError> {
    if text.contains('\0') || text.starts_with("\u{7f}ELF") || text.starts_with("MZ") || text.starts_with("!<arch>\n") {
        return Err(error("binary/provider payload cannot be admitted as source text"));
    }
    Ok(())
}
pub(crate) fn digest(path: &Path) -> Result<(u64, String), RunError> {
    let metadata = fs::symlink_metadata(path).map_err(|e| error(format!("measuring {}: {e}", path.display())))?;
    if metadata.is_file() {
        let mut file = fs::File::open(path).map_err(|e| error(e.to_string()))?;
        let mut hasher = blake3::Hasher::new();
        let mut buffer = [0u8; 65536];
        let mut size = 0u64;
        loop {
            let n = file.read(&mut buffer).map_err(|e| error(e.to_string()))?;
            if n == 0 {
                break;
            }
            size = size.checked_add(n as u64).ok_or_else(|| error("file size overflow"))?;
            hasher.update(&buffer[..n]);
        }
        if size != metadata.len() {
            return Err(error("file changed during measurement"));
        }
        return Ok((size, hasher.finalize().to_hex().to_string()));
    }
    if !metadata.is_dir() {
        return Err(error("artifact root must be a regular file or directory, never a symlink"));
    }
    let limits = crunch_release_core::TreeCopyLimits {
        entries_count_max: 1_000_000,
        ..crunch_release_core::TreeCopyLimits::RELEASE_BUNDLE
    };
    let (_, hash) =
        crate::release_tree_copy::hash_directory_tree_with_limits(path, limits).map_err(|e| error(e.to_string()))?;
    fn size(path: &Path, count: &mut usize) -> Result<u64, RunError> {
        let mut total = 0u64;
        for entry in fs::read_dir(path).map_err(|e| error(e.to_string()))? {
            *count += 1;
            if *count > 1_000_000 {
                return Err(error("tree entry bound"));
            }
            let entry = entry.map_err(|e| error(e.to_string()))?;
            let m = entry.path().symlink_metadata().map_err(|e| error(e.to_string()))?;
            let n = if m.is_dir() {
                size(&entry.path(), count)?
            } else {
                m.len()
            };
            total = total.checked_add(n).ok_or_else(|| error("tree size overflow"))?;
        }
        Ok(total)
    }
    Ok((size(path, &mut 0)?, hash))
}
fn source_digest(proof: &SourceBuiltFixedPointPlan, role: SourceAuthorityRole) -> Result<&str, RunError> {
    proof
        .source_inputs
        .iter()
        .find(|s| s.role == role)
        .map(|s| s.digest_blake3.as_str())
        .ok_or_else(|| error("source role not admitted"))
}
fn checked_source(
    proof: &SourceBuiltFixedPointPlan,
    roots: &BTreeMap<SourceAuthorityRole, PathBuf>,
    role: SourceAuthorityRole,
) -> Result<PathBuf, RunError> {
    let root = roots.get(&role).ok_or_else(|| error("missing physical source role"))?;
    if digest(root)?.1 != source_digest(proof, role)? {
        return Err(error("admitted source content drift"));
    }
    Ok(root.clone())
}
pub(crate) fn source_member(root: &Path, member: &str) -> Result<PathBuf, RunError> {
    relative(member, true)?;
    if member.is_empty() {
        return Ok(root.to_path_buf());
    }
    if !root.is_dir() {
        return Err(error("source role is not a directory"));
    }
    let mut path = root.to_path_buf();
    for part in Path::new(member).components() {
        path.push(part);
        if fs::symlink_metadata(&path).map_err(|e| error(e.to_string()))?.file_type().is_symlink() {
            return Err(error("source path traverses symlink"));
        }
    }
    Ok(path)
}

/// The AST-based resolver must discover every file import successfully. The
/// admitted source tree is rehashed by each caller around evaluation; a watch
/// hint alone is never accepted as a stable source snapshot.
pub(crate) fn validate_source_imports(root: &Path, file: &Path, search: &[std::ffi::OsString]) -> Result<(), RunError> {
    let dependencies = crunch_eval::watch_imports::source_dependencies(file, search)
        .map_err(|failure| error(format!("resolving source-program imports: {failure}")))?;
    for dependency in dependencies {
        let member = dependency.strip_prefix(root).map_err(|_| {
            error(format!("source program imports outside its admitted role: {}", dependency.display()))
        })?;
        let member = member.to_str().ok_or_else(|| error("non-UTF8 source program import"))?;
        let _ = source_member(root, member)?;
    }
    Ok(())
}
fn hydrate_reference(
    proof: &SourceBuiltFixedPointPlan,
    roots: &BTreeMap<SourceAuthorityRole, PathBuf>,
    cache: &Path,
    reference: &ArtifactRef,
) -> Result<PathBuf, RunError> {
    match reference {
        ArtifactRef::Source { role, path } => source_member(&checked_source(proof, roots, *role)?, path),
        ArtifactRef::ManifestSource {
            role,
            record_identity,
            member,
        } => {
            let manifest_path = checked_source(proof, roots, *role)?;
            let manifest = crate::source_bundle::read_source_bundle(&manifest_path)?;
            let mut records = manifest.records.iter().filter(|r| r.identity == *record_identity);
            let record = records.next().ok_or_else(|| error("unknown manifest source record"))?;
            if records.next().is_some() {
                return Err(error("duplicate manifest source record"));
            }
            let dest = cache.join(format!("{:?}-{record_identity}", role));
            if !dest.exists() {
                crate::source_bundle::materialize_source_record_payload(record, &dest)?;
            }
            source_member(&dest, member)
        }
        ArtifactRef::Produced { .. } => Err(error("produced output cannot be hydrated as source")),
    }
}
/// Resolve exactly one record from the source role authenticated by this proof.
/// Provider manifests and configured/generated toolchain roots are never fetches.
pub(super) fn authenticated_fetcher_record(
    proof: &SourceBuiltFixedPointPlan,
    roots: &BTreeMap<SourceAuthorityRole, PathBuf>,
    role: SourceAuthorityRole,
    identity: &str,
) -> Result<crate::source_bundle::SourceRecord, RunError> {
    let path = checked_source(proof, roots, role)?;
    let manifest = crate::source_bundle::read_source_bundle(&path)?;
    let mut records = manifest.records.into_iter().filter(|record| record.identity == identity);
    let record = records.next().ok_or_else(|| error("unknown fetcher source record"))?;
    if records.next().is_some() {
        return Err(error("duplicate fetcher source record"));
    }
    fetcher_output_kind(&record)?;
    Ok(record)
}
pub(super) fn fetcher_output_kind(record: &crate::source_bundle::SourceRecord) -> Result<ArtifactKind, RunError> {
    if !crate::source_bundle::source_record_is_fetcher_input(record) {
        return Err(error("source record is not an original fetcher input"));
    }
    // Git fetches intrinsically produce recursive source trees, independently
    // of the unpack flag used by URL fetches.
    Ok(
        if record.kind == crate::source_bundle::SourceRecordKind::VcsSnapshot
            || record.metadata.get("unpack").map(String::as_str) == Some("1")
        {
            ArtifactKind::Tree
        } else {
            ArtifactKind::Text
        },
    )
}
pub(crate) fn prepare(
    proof: &SourceBuiltFixedPointPlan,
    stage_id: &str,
    recipe_role: SourceAuthorityRole,
    recipe_path: &str,
    paths: &RecipePaths<'_>,
    prior: &[&PreparedRecipe],
    host_mantle_digest: &str,
) -> Result<PreparedRecipe, RunError> {
    prepare_from_reference(
        proof,
        stage_id,
        &ArtifactRef::Source {
            role: recipe_role,
            path: recipe_path.to_string(),
        },
        paths,
        prior,
        host_mantle_digest,
    )
}
fn prior_stage_output(
    proof: &SourceBuiltFixedPointPlan,
    prior: &[&PreparedRecipe],
    role: ProofOutputRole,
) -> Result<ArtifactRef, RunError> {
    let stage = proof
        .stages
        .iter()
        .find(|stage| stage.output == role)
        .ok_or_else(|| error("unknown prior stage output role"))?;
    let mut candidates = prior.iter().filter(|previous| previous.adapter.stage_id == stage.stage_id);
    let producer = candidates.next().ok_or_else(|| error(format!("missing actual prior stage {}", stage.stage_id)))?;
    if candidates.next().is_some() {
        return Err(error(format!("ambiguous actual prior stage {}", stage.stage_id)));
    }
    let output_id = match role {
        ProofOutputRole::StagexTransition => "stagex-transition",
        ProofOutputRole::StagexProvider => "stagex-provider",
        ProofOutputRole::FullSourceNativeProvider => "full-source-native-provider",
        ProofOutputRole::FullSourceRustProvider => "full-source-rust-provider",
        ProofOutputRole::MantleStage1 => "mantle-stage1",
        ProofOutputRole::MantleStage2 => "mantle-stage2",
    };
    let export = if role == ProofOutputRole::FullSourceNativeProvider {
        "native.provider"
    } else {
        output_id
    };
    let reference = &producer.export(export)?.reference;
    if !matches!(reference, ArtifactRef::Produced { action_id, output_id: actual }
        if actual == output_id && action_id.starts_with(&format!("{}/", stage.stage_id)))
    {
        return Err(error(format!("prior stage export is not its canonical produced output {output_id}")));
    }
    Ok(reference.clone())
}

#[allow(clippy::too_many_arguments)]
fn lower_step(
    proof: &SourceBuiltFixedPointPlan,
    stage_id: &str,
    stage_inputs: &[StageAuthorityInput],
    prior: &[&PreparedRecipe],
    paths: &RecipePaths<'_>,
    cache: &Path,
    host_mantle_digest: &str,
    sandbox_digest: &str,
    source_inputs: &[ActionInput],
    stage_outputs: &[ArtifactRef],
    step: &RecipeStep,
    bindings: &mut impl MutableArtifactBindings,
) -> Result<Vec<ActionSpec>, RunError> {
    let mut adapters = Vec::new();
    let id = action_id(stage_id, &step.id);
    let mut inputs = source_inputs.to_vec();
    for reference in step.inputs.values().chain(stage_outputs.iter()) {
        validate_ref(reference)?;
        let input = match reference {
            ArtifactRef::Produced { action_id, output_id } => {
                for previous in prior {
                    if let Some(local) =
                        action_id.strip_prefix(&previous.recipe.stage_id).and_then(|suffix| suffix.strip_prefix('/'))
                        && previous
                            .recipe
                            .steps
                            .iter()
                            .any(|producer| producer.id == local && operation_guard(&producer.operation).is_some())
                    {
                        return Err(error("prior guarded outputs require their explicit phase observation artifact"));
                    }
                }
                if !bindings.contains_key(reference) {
                    return Err(error(format!("unknown generated input {action_id}/{output_id}")));
                }
                ActionInput::Produced {
                    action_id: action_id.clone(),
                    output_id: output_id.clone(),
                }
            }
            ArtifactRef::Source { role, .. } | ArtifactRef::ManifestSource { role, .. } => {
                let authority = StageAuthorityInput::Source { role: *role };
                if !stage_inputs.contains(&authority) {
                    return Err(error("undeclared source role in recipe"));
                }
                if !bindings.contains_key(reference) {
                    let path = hydrate_reference(proof, paths.source_roots, cache, reference)?;
                    let kind = if path.is_dir() {
                        ArtifactKind::Tree
                    } else {
                        ArtifactKind::Text
                    };
                    bindings.insert(reference.clone(), ArtifactBinding {
                        reference: reference.clone(),
                        path,
                        kind,
                        bytes_max: proof.resource_bounds.disk_bytes_max,
                    });
                }
                ActionInput::Stage { authority }
            }
        };
        if !inputs.contains(&input) {
            inputs.push(input);
        }
    }
    if let RecipeOperation::MaterializeRecord { role, .. } | RecipeOperation::MaterializeFetcher { role, .. } =
        &step.operation
        && !stage_inputs.contains(&StageAuthorityInput::Source { role: *role })
    {
        return Err(error("source record materialization uses undeclared role"));
    }
    if let RecipeOperation::MaterializeFetcher {
        role,
        record_identity,
        output,
    } = &step.operation
    {
        let record = authenticated_fetcher_record(proof, paths.source_roots, *role, record_identity)?;
        output_kind(step, output, fetcher_output_kind(&record)?)?;
    }
    if let RecipeOperation::Exec {
        arguments,
        environment,
        invocation_input,
        runtime_mounts,
        command_state,
        ..
    } = &step.operation
    {
        for destination in runtime_mounts.keys() {
            let library = ["/lib/", "/lib64/", "/usr/lib/", "/usr/lib64/"]
                .iter()
                .any(|prefix| destination.starts_with(*prefix));
            let original_store = destination
                .strip_prefix(proof.logical_store_prefix.as_str())
                .is_some_and(|tail| tail.starts_with('/') && tail.len() > 1);
            if !library && !original_store && destination != "/mantle-rust-runtime" {
                return Err(error("runtime mount lies outside library/admitted logical-store runtime namespaces"));
            }
        }
        if let Some(name) = invocation_input
            && bindings[&step.inputs[name]].kind != ArtifactKind::Text
        {
            return Err(error("invocation request must be exact Produced Text"));
        }
        if let Some(name) = command_state.as_ref().and_then(|state| state.prior_times_input.as_ref())
            && bindings[&step.inputs[name]].kind != ArtifactKind::Text
        {
            return Err(error("prior command times must be exact Produced Text"));
        }
        for word in arguments.iter().chain(environment.values()) {
            validate_word_binding(step, word, bindings)?;
        }
    }
    let executable = match &step.operation {
        RecipeOperation::Exec { executable_input, .. } => match &step.inputs[executable_input] {
            ArtifactRef::Produced { action_id, output_id } => {
                if bindings[&step.inputs[executable_input]].kind != ArtifactKind::Executable {
                    return Err(error("executable must be an exact individually produced executable artifact"));
                }
                ActionExecutable::Produced {
                    action_id: action_id.clone(),
                    output_id: output_id.clone(),
                }
            }
            ArtifactRef::Source {
                role: SourceAuthorityRole::StagexSeed,
                path,
            } if path.is_empty() && stage_id == "stagex-transition" => ActionExecutable::Fixed {
                digest_blake3: source_digest(proof, SourceAuthorityRole::StagexSeed)?.to_string(),
            },
            _ => return Err(error("imported executable authority")),
        },
        _ => ActionExecutable::Fixed {
            digest_blake3: host_mantle_digest.to_string(),
        },
    };
    if let RecipeOperation::Exec { output_seeds, .. } = &step.operation {
        for (output, input) in output_seeds {
            if bindings[&step.inputs[input]].kind != step.outputs[output].kind
                || bindings[&step.inputs[input]].bytes_max > step.outputs[output].bytes_max
            {
                return Err(error("output seed must preserve its exact Produced kind and fit the output bound"));
            }
        }
    }
    if let RecipeOperation::ObservePhase { phase, state, .. } = &step.operation {
        if let Some(state) = state
            && bindings[&step.inputs[&state.skipped_input]].kind != ArtifactKind::Tree
        {
            return Err(error("phase prior state must be an exact Produced Tree"));
        }
        inputs.push(ActionInput::PhaseOutcome {
            action_id: action_id(stage_id, phase),
        });
    }
    let mut producers = BTreeSet::new();
    for input in &inputs {
        if let ActionInput::Produced { action_id, .. } | ActionInput::PhaseOutcome { action_id } = input {
            producers.insert(action_id.clone());
        }
    }
    if let ActionExecutable::Produced { action_id, .. } = &executable {
        producers.insert(action_id.clone());
    }
    let outputs = step.outputs.keys().cloned().collect::<Vec<_>>();
    let action = ActionSpec {
        id: id.clone(),
        stage_id: stage_id.to_string(),
        producer_actions: producers.into_iter().collect(),
        executable,
        guard: lower_guard(step)?,
        accepted_exit_codes: match &step.operation {
            RecipeOperation::Exec {
                exit_policy: RecipeExitPolicy::Exact { code },
                ..
            } => vec![*code],
            RecipeOperation::Exec {
                exit_policy: RecipeExitPolicy::OneOf { codes },
                ..
            } => codes.clone(),
            RecipeOperation::Exec {
                exit_policy: RecipeExitPolicy::Normal,
                ..
            } => (0..=255).collect(),
            _ => Vec::new(),
        },
        inputs: inputs.clone(),
        output_ids: outputs,
        protected_events_min: u32::from(matches!(step.operation, RecipeOperation::Exec { .. })),
        protected_events_max: u32::from(matches!(step.operation, RecipeOperation::Exec { .. })),
        build_events_min: 1,
        build_events_max: 1,
        elapsed_seconds_max: proof.resource_bounds.elapsed_seconds_max,
        disk_bytes_max: proof.resource_bounds.disk_bytes_max,
        local_only: true,
    };
    if matches!(step.operation, RecipeOperation::Exec { .. }) {
        let sandbox_id = format!("{id}.sandbox");
        let mut sandbox = action.clone();
        sandbox.id = sandbox_id.clone();
        sandbox.accepted_exit_codes.clear();
        sandbox.executable = ActionExecutable::Fixed {
            digest_blake3: sandbox_digest.to_string(),
        };
        sandbox.producer_actions = inputs
            .iter()
            .filter_map(|i| {
                if let ActionInput::Produced { action_id, .. } = i {
                    Some(action_id.clone())
                } else {
                    None
                }
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        sandbox.output_ids = vec![
            format!("sandbox-{}-stderr", step.id),
            format!("sandbox-{}-stdout", step.id),
        ];
        sandbox.output_ids.sort();
        adapters.push(sandbox);
    }
    adapters.push(action);
    for (output_id, output) in &step.outputs {
        let reference = ArtifactRef::Produced {
            action_id: id.clone(),
            output_id: output_id.clone(),
        };
        let binding = ArtifactBinding {
            reference: reference.clone(),
            path: paths.runtime_root.join(&step.id).join("outputs").join(output_id),
            kind: output.kind,
            bytes_max: output.bytes_max,
        };
        if bindings.insert(reference, binding).is_some() {
            return Err(error("duplicate output identity"));
        }
    }
    Ok(adapters)
}

pub(crate) fn prepare_from_reference(
    proof: &SourceBuiltFixedPointPlan,
    stage_id: &str,
    recipe_source: &ArtifactRef,
    paths: &RecipePaths<'_>,
    prior: &[&PreparedRecipe],
    host_mantle_digest: &str,
) -> Result<PreparedRecipe, RunError> {
    crate::source_built_fixed_point::validate_source_built_fixed_point_plan(proof).map_err(|e| error(e.to_string()))?;
    let stage = proof.stages.iter().find(|s| s.stage_id == stage_id).ok_or_else(|| error("unknown recipe stage"))?;
    let recipe_role = match recipe_source {
        ArtifactRef::Source { role, .. } | ArtifactRef::ManifestSource { role, .. } => *role,
        _ => return Err(error("recipe program cannot be a generated or externally supplied plan")),
    };
    if !stage.inputs.contains(&StageAuthorityInput::Source { role: recipe_role }) {
        return Err(error("recipe source role is not admitted to this stage"));
    }
    let cache = tempfile::tempdir().map_err(|e| error(e.to_string()))?;
    let path = hydrate_reference(proof, paths.source_roots, cache.path(), recipe_source)?;
    let metadata = fs::symlink_metadata(&path).map_err(|e| error(e.to_string()))?;
    if !metadata.is_file() || metadata.len() > RECIPE_BYTES_MAX {
        return Err(error("source recipe is not bounded regular source data"));
    }
    let bytes = fs::read(&path).map_err(|e| error(e.to_string()))?;
    let recipe: SourceRecipe =
        serde_json::from_slice(&bytes).map_err(|e| error(format!("source recipe syntax: {e}")))?;
    validate_source_recipe(&recipe)?;
    if recipe
        .timeout_groups
        .iter()
        .any(|group| group.seconds_max > proof.resource_bounds.elapsed_seconds_max)
    {
        return Err(error("source timeout exceeds proof elapsed bound"));
    }
    if recipe.stage_id != stage_id {
        return Err(error("cross-stage source recipe"));
    }
    if host_mantle_digest.len() != 64
        || !host_mantle_digest.bytes().all(|b| b.is_ascii_digit() || b"abcdef".contains(&b))
    {
        return Err(error("invalid host Mantle identity"));
    }
    if !paths.runtime_root.is_absolute() || !paths.sandbox_binary.is_absolute() {
        return Err(error("runtime and reviewed sandbox paths must be absolute"));
    }
    let sandbox_digest = digest(paths.sandbox_binary)?.1;
    let mut bindings = BTreeMap::<ArtifactRef, ArtifactBinding>::new();
    for previous in prior {
        if previous.proof.plan_digest_blake3 != proof.plan_digest_blake3 {
            return Err(error("cross-root prior recipe"));
        }
        for (reference, binding) in &previous.bindings {
            if matches!(reference, ArtifactRef::Produced { .. }) {
                if let Some(existing) = bindings.get(reference) {
                    if existing != binding {
                        return Err(error("conflicting prior producer output binding"));
                    }
                } else {
                    bindings.insert(reference.clone(), binding.clone());
                }
            }
        }
    }
    let mut adapters = Vec::new();
    let mut source_inputs = Vec::new();
    for authority in &stage.inputs {
        if !matches!(authority, StageAuthorityInput::Output { .. }) {
            source_inputs.push(ActionInput::Stage { authority: *authority });
        }
    }
    // Every declared prior stage-output edge is consumed as a real produced
    // identity, in addition to exact executable/data producer identities.
    let mut stage_outputs = Vec::new();
    for authority in &stage.inputs {
        if let StageAuthorityInput::Output { role } = authority {
            stage_outputs.push(prior_stage_output(proof, prior, *role)?);
        }
    }
    let mut reachable_ids = BTreeSet::new();
    let mut controllers = Vec::new();
    for step in &recipe.steps {
        if let RecipeOperation::Controller { specification, .. } = &step.operation {
            let mut controller = controller::lower(
                proof,
                stage_id,
                &stage.inputs,
                prior,
                paths,
                cache.path(),
                host_mantle_digest,
                &sandbox_digest,
                &source_inputs,
                &stage_outputs,
                specification,
                &mut bindings,
            )?;
            controller.guard = lower_guard(step)?;
            for (output_id, output) in &step.outputs {
                let reference = ArtifactRef::Produced {
                    action_id: action_id(stage_id, &step.id),
                    output_id: output_id.clone(),
                };
                let binding = ArtifactBinding {
                    reference: reference.clone(),
                    path: paths.runtime_root.join(&step.id).join("outputs").join(output_id),
                    kind: output.kind,
                    bytes_max: output.bytes_max,
                };
                if bindings.insert(reference, binding).is_some() {
                    return Err(error("duplicate compact controller output identity"));
                }
            }
            controllers.push(controller);
            continue;
        }
        let lowered = lower_step(
            proof,
            stage_id,
            &stage.inputs,
            prior,
            paths,
            cache.path(),
            host_mantle_digest,
            &sandbox_digest,
            &source_inputs,
            &stage_outputs,
            step,
            &mut bindings,
        )?;
        reachable_ids.extend(lowered.iter().map(|action| action.id.clone()));
        adapters.extend(lowered);
    }
    let count = adapters
        .iter()
        .try_fold(0u32, |sum, a| {
            sum.checked_add(a.protected_events_max)?
                .checked_add(a.build_events_max)?
                .checked_add(u32::from(a.guard.is_some()))
        })
        .ok_or_else(|| error("event bound overflow"))?;
    // Validate checked potential arithmetic without charging conditional turns
    // against the actual event ceiling. The shared recorder caps emitted events.
    for controller in &controllers {
        crate::source_built_fixed_point_action_trust::validate_controller_shape(controller, stage_id).map_err(error)?;
    }
    if count > proof.resource_bounds.protected_exec_events_max {
        return Err(error("recipe exceeds proof event bound"));
    }
    Ok(PreparedRecipe {
        proof: proof.clone(),
        recipe,
        adapter: StageActionAdapter {
            stage_id: stage_id.to_string(),
            reachable_action_ids: reachable_ids.into_iter().collect(),
            actions: adapters,
            controllers,
        },
        sources: paths.source_roots.clone(),
        bindings,
        runtime_root: paths.runtime_root.to_path_buf(),
        sandbox_binary: paths.sandbox_binary.to_path_buf(),
        sandbox_digest,
        host_digest: host_mantle_digest.to_string(),
        _source_cache: cache,
    })
}
pub(crate) fn run(
    prepared: &PreparedRecipe,
    emit: &mut dyn FnMut(ActionExecutionEvent) -> Result<(), RunError>,
) -> Result<RecipeRun, RunError> {
    crate::source_built_recipe_exec::run(prepared, emit)
}

pub(crate) fn read_manifest_source_member(
    proof: &SourceBuiltFixedPointPlan,
    role: SourceAuthorityRole,
    record_identity: &str,
    member: &str,
    paths: &RecipePaths<'_>,
) -> Result<Vec<u8>, RunError> {
    let cache = tempfile::tempdir().map_err(|e| error(e.to_string()))?;
    let reference = ArtifactRef::ManifestSource {
        role,
        record_identity: record_identity.to_string(),
        member: member.to_string(),
    };
    let path = hydrate_reference(proof, paths.source_roots, cache.path(), &reference)?;
    let metadata = fs::symlink_metadata(&path).map_err(|e| error(e.to_string()))?;
    if !metadata.is_file() || metadata.len() > RECIPE_BYTES_MAX {
        return Err(error("manifest recipe/index member is not bounded regular source data"));
    }
    fs::read(path).map_err(|e| error(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_built_fixed_point_planning::InitialOutputAuthorityState;
    use crate::source_built_fixed_point_planning::SourceBuiltFixedPointPlanInput;
    use crate::source_built_fixed_point_planning::plan_source_built_fixed_point;

    fn text_recipe(stage: &str, step: &str, output: &str, export: &str) -> SourceRecipe {
        let reference = ArtifactRef::Produced {
            action_id: action_id(stage, step),
            output_id: output.to_string(),
        };
        SourceRecipe {
            schema: SCHEMA.to_string(),
            stage_id: stage.to_string(),
            preparation: RecipePreparation {
                tool: "structural-fixture".into(),
                provenance: "unit source".into(),
            },
            timeout_groups: Vec::new(),
            steps: vec![RecipeStep {
                id: step.to_string(),
                inputs: BTreeMap::new(),
                outputs: BTreeMap::from([(output.to_string(), RecipeOutput {
                    kind: ArtifactKind::Text,
                    bytes_max: 64,
                })]),
                operation: RecipeOperation::WriteText {
                    output: output.to_string(),
                    text: "fixture".into(),
                },
            }],
            roots: vec![reference.clone()],
            exports: BTreeMap::from([(export.to_string(), reference)]),
            semantic_roots: BTreeMap::new(),
        }
    }

    // These source-only structural programs never execute or certify a bootstrap.
    fn stage_binding_fixture(directory: &Path) -> (SourceBuiltFixedPointPlan, BTreeMap<SourceAuthorityRole, PathBuf>) {
        let mut transition = text_recipe("stagex-transition", "transition", "stagex-transition", "stagex-transition");
        let helper = text_recipe("stagex-transition", "helper", "stagex-provider", "helper");
        transition.steps.extend(helper.steps);
        transition.roots.extend(helper.roots);
        transition.exports.extend(helper.exports);
        let recipes = [
            (SourceAuthorityRole::StagexLineage, transition),
            (
                SourceAuthorityRole::StagexSourceBundle,
                text_recipe("stagex-provider-publication", "publish", "stagex-provider", "stagex-provider"),
            ),
            (
                SourceAuthorityRole::NativeSourceBundle,
                text_recipe("full-source-native-provider", "native", "full-source-native-provider", "native.provider"),
            ),
        ];
        let mut sources = BTreeMap::new();
        for (index, (role, recipe)) in recipes.into_iter().enumerate() {
            let path = directory.join(format!("source-{index}.json"));
            fs::write(&path, serde_json::to_vec(&recipe).unwrap()).unwrap();
            sources.insert(role, path);
        }
        let mut original = crate::source_built_fixed_point_action_trust::tests::fixture().0;
        for source in &mut original.source_inputs {
            if let Some(path) = sources.get(&source.role) {
                (source.size_bytes, source.digest_blake3) = digest(path).unwrap();
            }
        }
        original.resource_bounds.disk_bytes_max = 1_000_000;
        let proof = plan_source_built_fixed_point(SourceBuiltFixedPointPlanInput {
            proof_id: original.proof_id,
            logical_store_prefix: original.logical_store_prefix,
            source_inputs: original.source_inputs,
            initial_output_authority: InitialOutputAuthorityState {
                stagex_transition_entries: 0,
                native_provider_entries: 0,
                rust_provider_entries: 0,
                mantle_output_entries: 0,
            },
            policies: original.policies,
            resource_bounds: original.resource_bounds,
        })
        .unwrap();
        (proof, sources)
    }

    fn prepare_binding_fixture(
        proof: &SourceBuiltFixedPointPlan,
        sources: &BTreeMap<SourceAuthorityRole, PathBuf>,
        runtime: &Path,
        stage: &str,
        role: SourceAuthorityRole,
        prior: &[&PreparedRecipe],
    ) -> Result<PreparedRecipe, RunError> {
        // This structural case never executes the sandbox. Measure the declared
        // small shell rather than repeatedly hashing the entire test harness.
        let sandbox = fs::canonicalize(env!("SNIX_BUILD_SANDBOX_SHELL")).unwrap();
        prepare_from_reference(
            proof,
            stage,
            &ArtifactRef::Source {
                role,
                path: String::new(),
            },
            &RecipePaths {
                source_roots: sources,
                runtime_root: runtime,
                sandbox_binary: &sandbox,
            },
            prior,
            &digest(&sandbox).unwrap().1,
        )
    }

    #[test]
    fn source_program_path_labels_reject_dot_components() {
        validate_source_recipe(&text_recipe("stagex-transition", "build.v1", "value.v1", "stagex-transition")).unwrap();
        for component in [".", ".."] {
            assert!(
                validate_source_recipe(&text_recipe("stagex-transition", component, "value", "stagex-transition"))
                    .is_err()
            );
            assert!(
                validate_source_recipe(&text_recipe("stagex-transition", "build", component, "stagex-transition"))
                    .is_err()
            );
        }
    }

    #[test]
    fn prior_stage_output_requires_its_canonical_producer() {
        let directory = tempfile::tempdir().unwrap();
        let (proof, sources) = stage_binding_fixture(directory.path());
        let transition = prepare_binding_fixture(
            &proof,
            &sources,
            &directory.path().join("transition"),
            "stagex-transition",
            SourceAuthorityRole::StagexLineage,
            &[],
        )
        .unwrap();
        assert!(
            prepare_binding_fixture(
                &proof,
                &sources,
                &directory.path().join("missing-publication"),
                "full-source-native-provider",
                SourceAuthorityRole::NativeSourceBundle,
                &[&transition]
            )
            .is_err()
        );
        let publication = prepare_binding_fixture(
            &proof,
            &sources,
            &directory.path().join("publication"),
            "stagex-provider-publication",
            SourceAuthorityRole::StagexSourceBundle,
            &[&transition],
        )
        .unwrap();
        let native = prepare_binding_fixture(
            &proof,
            &sources,
            &directory.path().join("native"),
            "full-source-native-provider",
            SourceAuthorityRole::NativeSourceBundle,
            &[&transition, &publication],
        )
        .unwrap();
        let action = native
            .adapter()
            .actions
            .iter()
            .find(|action| action.id == "full-source-native-provider/native")
            .unwrap();
        assert!(action.inputs.contains(&ActionInput::Produced {
            action_id: "stagex-provider-publication/publish".into(),
            output_id: "stagex-provider".into()
        }));
        assert!(!action.inputs.contains(&ActionInput::Produced {
            action_id: "stagex-transition/helper".into(),
            output_id: "stagex-provider".into()
        }));
    }

    #[test]
    fn source_program_imports_must_stay_in_the_authenticated_tree() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("sources");
        fs::create_dir(&source).unwrap();
        let program = source.join("route.ncl");
        fs::write(source.join("inside.ncl"), "{ value = 7 }").unwrap();
        fs::write(&program, "import \"inside.ncl\"").unwrap();
        validate_source_imports(&source, &program, &[]).unwrap();
        fs::write(directory.path().join("outside.ncl"), "{ value = 9 }").unwrap();
        fs::write(&program, "import \"../outside.ncl\"").unwrap();
        assert!(
            validate_source_imports(&source, &program, &[])
                .unwrap_err()
                .to_string()
                .contains("outside its admitted role")
        );
    }

    #[cfg(unix)]
    #[test]
    fn source_program_import_cannot_hide_unadmitted_bytes_behind_a_symlink() {
        let directory = tempfile::tempdir().unwrap();
        let source = directory.path().join("sources");
        fs::create_dir(&source).unwrap();
        fs::write(directory.path().join("outside.ncl"), "{ value = 9 }").unwrap();
        std::os::unix::fs::symlink(directory.path().join("outside.ncl"), source.join("inside.ncl")).unwrap();
        let program = source.join("route.ncl");
        fs::write(&program, "import \"inside.ncl\"").unwrap();
        assert!(validate_source_imports(&source, &program, &[]).unwrap_err().to_string().contains("symlink"));
    }

    #[test]
    fn generated_workspace_and_actual_status_have_bounded_declared_outputs() {
        let mut step = RecipeStep {
            id: "generator".into(),
            inputs: BTreeMap::from([("seed".into(), ArtifactRef::Source {
                role: SourceAuthorityRole::StagexSeed,
                path: String::new(),
            })]),
            outputs: BTreeMap::from([
                ("stdout".into(), RecipeOutput {
                    kind: ArtifactKind::Text,
                    bytes_max: 4096,
                }),
                ("stderr".into(), RecipeOutput {
                    kind: ArtifactKind::Text,
                    bytes_max: 4096,
                }),
                ("generated".into(), RecipeOutput {
                    kind: ArtifactKind::Tree,
                    bytes_max: 4096,
                }),
            ]),
            operation: RecipeOperation::Exec {
                command_state: None,
                executable_input: "seed".into(),
                arguments: Vec::new(),
                environment: BTreeMap::new(),
                runtime_mounts: BTreeMap::new(),
                cwd_input: None,
                cwd_output: Some("generated".into()),
                stdin_input: None,
                merge_stderr: false,
                invocation_input: None,
                exit_policy: RecipeExitPolicy::Exact { code: 0 },
                guard: None,
                output_seeds: BTreeMap::new(),
            },
        };
        validate_operation(&step).unwrap();
        step.outputs.get_mut("generated").unwrap().kind = ArtifactKind::Text;
        assert!(validate_operation(&step).is_err());
        step.outputs.get_mut("generated").unwrap().kind = ArtifactKind::Tree;
        let RecipeOperation::Exec { cwd_output, .. } = &mut step.operation else {
            unreachable!()
        };
        *cwd_output = Some("missing".into());
        assert!(validate_operation(&step).is_err());
        let RecipeOperation::Exec {
            cwd_input, cwd_output, ..
        } = &mut step.operation
        else {
            unreachable!()
        };
        *cwd_output = Some("generated".into());
        *cwd_input = Some("seed".into());
        assert!(validate_operation(&step).is_err());
        let RecipeOperation::Exec { cwd_input, .. } = &mut step.operation else {
            unreachable!()
        };
        *cwd_input = None;
        step.outputs.insert("exit-status".into(), RecipeOutput {
            kind: ArtifactKind::Text,
            bytes_max: 4,
        });
        validate_operation(&step).unwrap();
        step.outputs.get_mut("exit-status").unwrap().bytes_max = 3;
        assert!(validate_operation(&step).is_err());
        step.outputs.get_mut("exit-status").unwrap().bytes_max = 4;
        step.outputs.get_mut("exit-status").unwrap().kind = ArtifactKind::Executable;
        assert!(validate_operation(&step).is_err());
    }

    #[test]
    fn probe_exit_policy_requires_a_bounded_sorted_normal_exit_set() {
        let policy = RecipeExitPolicy::OneOf { codes: vec![0, 1] };
        policy.validate().unwrap();
        assert!(policy.accepts(0));
        assert!(policy.accepts(1));
        assert!(
            validate_source_recipe(&text_recipe("stagex-transition", "constant", "exit-status", "status")).is_err()
        );
        assert!(!policy.accepts(2));
        for codes in [
            vec![],
            vec![0],
            vec![0, 0],
            vec![1, 0],
            vec![-1, 0],
            vec![0, 256],
            (0..9).collect(),
        ] {
            assert!(RecipeExitPolicy::OneOf { codes }.validate().is_err());
        }
        for code in [-1, 256] {
            assert!(RecipeExitPolicy::Exact { code }.validate().is_err());
        }
        RecipeExitPolicy::Exact { code: 255 }.validate().unwrap();
    }

    fn guarded_chain_recipe() -> SourceRecipe {
        let mut recipe = text_recipe("stagex-transition", "unused", "value", "value");
        recipe.steps.clear();
        let mut statuses = Vec::<String>::new();
        for (id, artifact, kind) in [
            ("probe", "probe-output", ArtifactKind::Text),
            ("compile", "assembly", ArtifactKind::Text),
            ("assemble", "object", ArtifactKind::Object),
            ("link", "binary", ArtifactKind::Executable),
            ("run", "result", ArtifactKind::Text),
        ] {
            let mut inputs = BTreeMap::from([("seed".into(), ArtifactRef::Source {
                role: SourceAuthorityRole::StagexSeed,
                path: String::new(),
            })]);
            let mut conditions = Vec::new();
            for prior in &statuses {
                let name = format!("{prior}-status");
                inputs.insert(name.clone(), ArtifactRef::Produced {
                    action_id: action_id(&recipe.stage_id, prior),
                    output_id: "exit-status".into(),
                });
                conditions.push(RecipeGuardCondition {
                    status_input: name,
                    accepted_codes: vec![0],
                });
            }
            let executable_input = if id == "run" {
                inputs.insert("binary".into(), ArtifactRef::Produced {
                    action_id: action_id(&recipe.stage_id, "link"),
                    output_id: "binary".into(),
                });
                "binary"
            } else {
                "seed"
            };
            if id == "assemble" || id == "link" {
                let (prior, output) = if id == "assemble" {
                    ("compile", "assembly")
                } else {
                    ("assemble", "object")
                };
                inputs.insert("operand".into(), ArtifactRef::Produced {
                    action_id: action_id(&recipe.stage_id, prior),
                    output_id: output.into(),
                });
            }
            recipe.steps.push(RecipeStep {
                id: id.into(),
                inputs,
                outputs: BTreeMap::from([
                    ("stdout".into(), RecipeOutput {
                        kind: ArtifactKind::Text,
                        bytes_max: 64,
                    }),
                    ("stderr".into(), RecipeOutput {
                        kind: ArtifactKind::Text,
                        bytes_max: 64,
                    }),
                    ("exit-status".into(), RecipeOutput {
                        kind: ArtifactKind::Text,
                        bytes_max: 4,
                    }),
                    (artifact.into(), RecipeOutput { kind, bytes_max: 4096 }),
                ]),
                operation: RecipeOperation::Exec {
                    command_state: None,
                    executable_input: executable_input.into(),
                    arguments: Vec::new(),
                    environment: BTreeMap::new(),
                    runtime_mounts: BTreeMap::new(),
                    cwd_input: None,
                    cwd_output: None,
                    stdin_input: None,
                    merge_stderr: false,
                    invocation_input: None,
                    exit_policy: RecipeExitPolicy::OneOf { codes: vec![0, 1] },
                    guard: (!conditions.is_empty()).then_some(RecipeGuard { conditions }),
                    output_seeds: BTreeMap::new(),
                },
            });
            statuses.push(id.into());
        }
        recipe.steps.push(RecipeStep {
            id: "observed".into(),
            inputs: BTreeMap::new(),
            outputs: BTreeMap::from([("envelope".into(), RecipeOutput {
                kind: ArtifactKind::Tree,
                bytes_max: 8192,
            })]),
            operation: RecipeOperation::ObservePhase {
                phase: "run".into(),
                output: "envelope".into(),
                state: None,
            },
        });
        recipe.roots = vec![ArtifactRef::Produced {
            action_id: action_id(&recipe.stage_id, "observed"),
            output_id: "envelope".into(),
        }];
        recipe.exports.clear();
        recipe
    }

    #[test]
    fn guarded_compile_assemble_link_and_exact_binary_require_dominating_predicates() {
        let recipe = guarded_chain_recipe();
        validate_source_recipe(&recipe).unwrap();
        for index in [2, 3, 4] {
            let mut invalid = recipe.clone();
            let RecipeOperation::Exec { guard: Some(guard), .. } = &mut invalid.steps[index].operation else {
                unreachable!()
            };
            guard.conditions.remove(0);
            assert!(validate_source_recipe(&invalid).is_err());
        }
        let mut direct = recipe.clone();
        direct.roots.push(ArtifactRef::Produced {
            action_id: action_id(&recipe.stage_id, "link"),
            output_id: "binary".into(),
        });
        assert!(validate_source_recipe(&direct).is_err());
        let mut alias = recipe;
        let status = alias.steps[1].inputs["probe-status"].clone();
        alias.steps[1].inputs.insert("alias".into(), status);
        let RecipeOperation::Exec { arguments, .. } = &mut alias.steps[1].operation else {
            unreachable!()
        };
        arguments.push(RecipeWord {
            parts: vec![WordPart::Input {
                name: "alias".into(),
                path: String::new(),
            }],
        });
        assert!(validate_source_recipe(&alias).is_err());
    }

    #[test]
    fn output_seed_admission_preserves_produced_kind_and_declared_bound() {
        use crate::source_built_fixed_point_planning::InitialOutputAuthorityState;
        use crate::source_built_fixed_point_planning::SourceBuiltFixedPointPlanInput;
        use crate::source_built_fixed_point_planning::plan_source_built_fixed_point;
        for (kind, bound, accepted) in [
            (ArtifactKind::Text, 64, true),
            (ArtifactKind::Tree, 64, false),
            (ArtifactKind::Text, 63, false),
        ] {
            let root = tempfile::tempdir().unwrap();
            let (original, mut sources) = stage_binding_fixture(root.path());
            let seed = fs::canonicalize(env!("SNIX_BUILD_SANDBOX_SHELL")).unwrap();
            sources.insert(SourceAuthorityRole::StagexSeed, seed);
            let mut recipe = text_recipe("stagex-transition", "prior", "data", "unused");
            let prior = recipe.roots[0].clone();
            recipe.steps.push(RecipeStep {
                id: "seeded".into(),
                inputs: BTreeMap::from([
                    ("prior".into(), prior),
                    ("seed".into(), ArtifactRef::Source {
                        role: SourceAuthorityRole::StagexSeed,
                        path: String::new(),
                    }),
                ]),
                outputs: BTreeMap::from([
                    ("stdout".into(), RecipeOutput {
                        kind: ArtifactKind::Text,
                        bytes_max: 64,
                    }),
                    ("stderr".into(), RecipeOutput {
                        kind: ArtifactKind::Text,
                        bytes_max: 64,
                    }),
                    ("copy".into(), RecipeOutput { kind, bytes_max: bound }),
                ]),
                operation: RecipeOperation::Exec {
                    command_state: None,
                    executable_input: "seed".into(),
                    arguments: Vec::new(),
                    environment: BTreeMap::new(),
                    runtime_mounts: BTreeMap::new(),
                    cwd_input: None,
                    cwd_output: None,
                    stdin_input: None,
                    merge_stderr: false,
                    invocation_input: None,
                    exit_policy: RecipeExitPolicy::Exact { code: 0 },
                    guard: None,
                    output_seeds: BTreeMap::from([("copy".into(), "prior".into())]),
                },
            });
            recipe.roots = vec![ArtifactRef::Produced {
                action_id: "stagex-transition/seeded".into(),
                output_id: "copy".into(),
            }];
            recipe.exports.clear();
            fs::write(&sources[&SourceAuthorityRole::StagexLineage], serde_json::to_vec(&recipe).unwrap()).unwrap();
            let mut inputs = original.source_inputs;
            for input in &mut inputs {
                if let Some(path) = sources.get(&input.role) {
                    (input.size_bytes, input.digest_blake3) = digest(path).unwrap();
                }
            }
            let proof = plan_source_built_fixed_point(SourceBuiltFixedPointPlanInput {
                proof_id: original.proof_id,
                logical_store_prefix: original.logical_store_prefix,
                source_inputs: inputs,
                initial_output_authority: InitialOutputAuthorityState {
                    stagex_transition_entries: 0,
                    native_provider_entries: 0,
                    rust_provider_entries: 0,
                    mantle_output_entries: 0,
                },
                policies: original.policies,
                resource_bounds: original.resource_bounds,
            })
            .unwrap();
            let prepared = prepare_binding_fixture(
                &proof,
                &sources,
                &root.path().join("runtime"),
                "stagex-transition",
                SourceAuthorityRole::StagexLineage,
                &[],
            );
            if accepted {
                prepared.unwrap();
            } else {
                assert!(prepared.err().unwrap().to_string().contains("output seed"));
            }
            let RecipeOperation::Exec { output_seeds, .. } = &mut recipe.steps[1].operation else {
                unreachable!()
            };
            output_seeds.insert("copy".into(), "seed".into());
            assert!(validate_source_recipe(&recipe).is_err());
        }
    }

    fn invocation_fixture() -> (RecipeStep, BTreeMap<ArtifactRef, ArtifactBinding>, RecipeInvocation) {
        let mut step = guarded_chain_recipe().steps.remove(0);
        let executable = ArtifactRef::Produced {
            action_id: "stagex-transition/tool".into(),
            output_id: "binary".into(),
        };
        let request = ArtifactRef::Produced {
            action_id: "stagex-transition/controller".into(),
            output_id: "request".into(),
        };
        let data = ArtifactRef::Produced {
            action_id: "stagex-transition/source".into(),
            output_id: "tree".into(),
        };
        step.inputs = BTreeMap::from([
            ("seed".into(), executable),
            ("request".into(), request.clone()),
            ("data".into(), data.clone()),
        ]);
        let RecipeOperation::Exec {
            arguments,
            environment,
            invocation_input,
            ..
        } = &mut step.operation
        else {
            unreachable!()
        };
        arguments.clear();
        environment.clear();
        *invocation_input = Some("request".into());
        let bindings = BTreeMap::from([
            (request.clone(), ArtifactBinding {
                reference: request,
                path: PathBuf::from("request"),
                kind: ArtifactKind::Text,
                bytes_max: 4096,
            }),
            (data.clone(), ArtifactBinding {
                reference: data,
                path: PathBuf::from("data"),
                kind: ArtifactKind::Tree,
                bytes_max: 4096,
            }),
        ]);
        let invocation = RecipeInvocation {
            arguments: vec![RecipeWord {
                parts: vec![WordPart::Input {
                    name: "data".into(),
                    path: "source.c".into(),
                }],
            }],
            environment: BTreeMap::from([("MODE".into(), RecipeWord {
                parts: vec![WordPart::Literal {
                    text: "original".into(),
                }],
            })]),
            cwd: None,
            argv0: None,
            executable_path: None,
            workspace_mounts: BTreeMap::new(),
            readonly_input_roots: Vec::new(),
            process_attributes: None,
            descriptors: None,
        };
        (step, bindings, invocation)
    }

    #[test]
    fn canonical_invocation_preserves_typed_operands_and_rejects_alternative_encodings() {
        let (step, bindings, invocation) = invocation_fixture();
        validate_operation(&step).unwrap();
        let bytes = serde_json::to_vec(&invocation).unwrap();
        assert_eq!(decode_invocation(&step, &bytes, &bindings).unwrap(), invocation);
        let mut trailing = bytes.clone();
        trailing.push(b'\n');
        assert!(decode_invocation(&step, &trailing, &bindings).is_err());
        let mut unknown: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        unknown["executable"] = serde_json::json!("hidden");
        assert!(decode_invocation(&step, &serde_json::to_vec(&unknown).unwrap(), &bindings).is_err());
        assert!(decode_invocation(&step, br#"{"environment":{},"arguments":[]}"#, &bindings).is_err());
        assert!(decode_invocation(&step, br#"{"arguments":[],"arguments":[],"environment":{}}"#, &bindings).is_err());
    }

    #[test]
    fn invocation_cannot_invent_paths_names_control_reads_or_unused_dependencies() {
        let (step, bindings, invocation) = invocation_fixture();
        for part in [
            WordPart::Input {
                name: "missing".into(),
                path: String::new(),
            },
            WordPart::Input {
                name: "data".into(),
                path: "../escape".into(),
            },
            WordPart::Input {
                name: "request".into(),
                path: "not-a-tree".into(),
            },
            WordPart::Literal { text: "/bin/sh".into() },
            WordPart::Literal {
                text: "bad\0word".into(),
            },
            WordPart::Output {
                name: "exit-status".into(),
                path: String::new(),
            },
            WordPart::Output {
                name: "unknown".into(),
                path: String::new(),
            },
        ] {
            let mut altered = invocation.clone();
            altered.arguments[0].parts = vec![part];
            assert!(decode_invocation(&step, &serde_json::to_vec(&altered).unwrap(), &bindings).is_err());
        }
        let mut unused = step.clone();
        unused.inputs.insert("unused".into(), step.inputs["data"].clone());
        assert!(decode_invocation(&unused, &serde_json::to_vec(&invocation).unwrap(), &bindings).is_err());
        let mut controlled = step.clone();
        let RecipeOperation::Exec { guard, .. } = &mut controlled.operation else {
            unreachable!()
        };
        *guard = Some(RecipeGuard {
            conditions: vec![RecipeGuardCondition {
                status_input: "data".into(),
                accepted_codes: vec![0],
            }],
        });
        assert!(decode_invocation(&controlled, &serde_json::to_vec(&invocation).unwrap(), &bindings).is_err());
        let mut invalid_environment = invocation;
        invalid_environment.environment.insert("BAD-NAME".into(), RecipeWord {
            parts: vec![WordPart::Literal { text: "value".into() }],
        });
        assert!(decode_invocation(&step, &serde_json::to_vec(&invalid_environment).unwrap(), &bindings).is_err());
    }

    #[test]
    fn invocation_admission_rejects_static_override_or_imported_request_and_executable() {
        let (step, _, _) = invocation_fixture();
        let mut nonempty = step.clone();
        let RecipeOperation::Exec { arguments, .. } = &mut nonempty.operation else {
            unreachable!()
        };
        arguments.push(RecipeWord {
            parts: vec![WordPart::Literal {
                text: "override".into(),
            }],
        });
        assert!(validate_operation(&nonempty).is_err());
        for name in ["seed", "request"] {
            let mut imported = step.clone();
            imported.inputs.insert(name.into(), ArtifactRef::Source {
                role: SourceAuthorityRole::StagexSeed,
                path: String::new(),
            });
            assert!(validate_operation(&imported).is_err());
        }
    }

    #[test]
    fn typed_text_slices_and_runtime_words_preserve_declared_authority() {
        let (mut step, mut bindings, mut invocation) = invocation_fixture();
        let data = step.inputs["data"].clone();
        bindings.get_mut(&data).unwrap().kind = ArtifactKind::Text;
        for part in [WordPart::TextInput { name: "data".into() }, WordPart::TextSlice {
            name: "data".into(),
            offset_bytes: 2,
            length_bytes: 4,
        }] {
            invocation.arguments[0].parts = vec![part];
            decode_invocation(&step, &serde_json::to_vec(&invocation).unwrap(), &bindings).unwrap();
        }
        for (offset_bytes, length_bytes) in [(u64::MAX, 1), (0, WORD_BYTES_MAX as u64 + 1), (RECIPE_BYTES_MAX, 1)] {
            invocation.arguments[0].parts = vec![WordPart::TextSlice {
                name: "data".into(),
                offset_bytes,
                length_bytes,
            }];
            assert!(decode_invocation(&step, &serde_json::to_vec(&invocation).unwrap(), &bindings).is_err());
        }
        invocation.arguments[0].parts = vec![WordPart::TextInput { name: "data".into() }];
        bindings.get_mut(&data).unwrap().kind = ArtifactKind::Tree;
        assert!(decode_invocation(&step, &serde_json::to_vec(&invocation).unwrap(), &bindings).is_err());
        let RecipeOperation::Exec { runtime_mounts, .. } = &mut step.operation else {
            unreachable!()
        };
        runtime_mounts.insert("/mantle-rust-runtime".into(), "data".into());
        invocation.arguments[0].parts = vec![WordPart::Runtime {
            mount: "/mantle-rust-runtime".into(),
            path: "libc.so".into(),
        }];
        validate_operation(&step).unwrap();
        decode_invocation(&step, &serde_json::to_vec(&invocation).unwrap(), &bindings).unwrap();
        for (mount, path) in [("/undeclared", ""), ("/mantle-rust-runtime", "../outside")] {
            invocation.arguments[0].parts = vec![WordPart::Runtime {
                mount: mount.into(),
                path: path.into(),
            }];
            assert!(decode_invocation(&step, &serde_json::to_vec(&invocation).unwrap(), &bindings).is_err());
        }
        invocation.arguments[0].parts = vec![WordPart::Runtime {
            mount: "/mantle-rust-runtime".into(),
            path: "libc.so".into(),
        }];
        bindings.get_mut(&data).unwrap().kind = ArtifactKind::Object;
        assert!(decode_invocation(&step, &serde_json::to_vec(&invocation).unwrap(), &bindings).is_err());
    }

    #[test]
    fn runtime_destinations_reject_duplicate_ancestor_and_reserved_namespaces() {
        let (step, _, _) = invocation_fixture();
        for mounts in [
            BTreeMap::from([
                ("/usr/lib/runtime".into(), "data".into()),
                ("/usr/lib/runtime/subtree".into(), "data".into()),
            ]),
            BTreeMap::from([("/inputs/runtime".into(), "data".into())]),
            BTreeMap::from([("/".into(), "data".into())]),
            BTreeMap::from([("/proc/runtime".into(), "data".into())]),
            BTreeMap::from([("/tmp/runtime".into(), "data".into())]),
        ] {
            let mut invalid = step.clone();
            let RecipeOperation::Exec { runtime_mounts, .. } = &mut invalid.operation else {
                unreachable!()
            };
            *runtime_mounts = mounts;
            assert!(validate_operation(&invalid).is_err());
        }
        let mut duplicate =
            serde_json::Deserializer::from_str(r#"{"/usr/lib/runtime":"data","/usr/lib/runtime":"data"}"#);
        assert!(deserialize_runtime_mounts(&mut duplicate).is_err());
    }

    #[test]
    fn runtime_tree_prepare_requires_the_authenticated_configured_store_prefix() {
        use crate::source_built_fixed_point_planning::InitialOutputAuthorityState;
        use crate::source_built_fixed_point_planning::SourceBuiltFixedPointPlanInput;
        use crate::source_built_fixed_point_planning::plan_source_built_fixed_point;
        for (prefix, destination, accepted) in [
            ("/custom/store", "/custom/store/provider/lib", true),
            ("/custom/store", "/mantle/store/provider/lib", false),
            ("/custom/store", "/custom/storehouse/provider", false),
            ("/custom/store", "/custom/store", false),
            ("/custom/store", "/usr/lib/provider", true),
            ("/custom/store", "/mantle-rust-runtime", true),
            ("/tmp/store", "/tmp/store/provider", false),
        ] {
            let root = tempfile::tempdir().unwrap();
            let (original, mut sources) = stage_binding_fixture(root.path());
            sources
                .insert(SourceAuthorityRole::StagexSeed, fs::canonicalize(env!("SNIX_BUILD_SANDBOX_SHELL")).unwrap());
            let mut recipe = text_recipe("stagex-transition", "data", "text", "unused");
            let data = recipe.roots[0].clone();
            recipe.steps.push(RecipeStep {
                id: "runtime".into(),
                inputs: BTreeMap::from([("data".into(), data)]),
                outputs: BTreeMap::from([("tree".into(), RecipeOutput {
                    kind: ArtifactKind::Tree,
                    bytes_max: 4096,
                })]),
                operation: RecipeOperation::AssembleTree {
                    output: "tree".into(),
                    entries: BTreeMap::from([("metadata.txt".into(), "data".into())]),
                },
            });
            recipe.steps.push(RecipeStep {
                id: "command".into(),
                inputs: BTreeMap::from([
                    ("seed".into(), ArtifactRef::Source {
                        role: SourceAuthorityRole::StagexSeed,
                        path: String::new(),
                    }),
                    ("runtime".into(), ArtifactRef::Produced {
                        action_id: "stagex-transition/runtime".into(),
                        output_id: "tree".into(),
                    }),
                ]),
                outputs: BTreeMap::from([
                    ("stdout".into(), RecipeOutput {
                        kind: ArtifactKind::Text,
                        bytes_max: 64,
                    }),
                    ("stderr".into(), RecipeOutput {
                        kind: ArtifactKind::Text,
                        bytes_max: 64,
                    }),
                ]),
                operation: RecipeOperation::Exec {
                    command_state: None,
                    executable_input: "seed".into(),
                    arguments: vec![RecipeWord {
                        parts: vec![WordPart::Runtime {
                            mount: destination.into(),
                            path: String::new(),
                        }],
                    }],
                    environment: BTreeMap::new(),
                    runtime_mounts: BTreeMap::from([(destination.into(), "runtime".into())]),
                    cwd_input: None,
                    cwd_output: None,
                    stdin_input: None,
                    merge_stderr: false,
                    invocation_input: None,
                    exit_policy: RecipeExitPolicy::Exact { code: 0 },
                    guard: None,
                    output_seeds: BTreeMap::new(),
                },
            });
            recipe.roots = vec![ArtifactRef::Produced {
                action_id: "stagex-transition/command".into(),
                output_id: "stdout".into(),
            }];
            recipe.exports.clear();
            fs::write(&sources[&SourceAuthorityRole::StagexLineage], serde_json::to_vec(&recipe).unwrap()).unwrap();
            let mut inputs = original.source_inputs;
            for input in &mut inputs {
                if let Some(path) = sources.get(&input.role) {
                    (input.size_bytes, input.digest_blake3) = digest(path).unwrap();
                }
            }
            let proof = plan_source_built_fixed_point(SourceBuiltFixedPointPlanInput {
                proof_id: original.proof_id,
                logical_store_prefix: prefix.into(),
                source_inputs: inputs,
                initial_output_authority: InitialOutputAuthorityState {
                    stagex_transition_entries: 0,
                    native_provider_entries: 0,
                    rust_provider_entries: 0,
                    mantle_output_entries: 0,
                },
                policies: original.policies,
                resource_bounds: original.resource_bounds,
            })
            .unwrap();
            let result = prepare_binding_fixture(
                &proof,
                &sources,
                &root.path().join("runtime"),
                "stagex-transition",
                SourceAuthorityRole::StagexLineage,
                &[],
            );
            if accepted {
                result.unwrap();
            } else {
                assert!(result.err().unwrap().to_string().contains("runtime mount"));
            }
        }
    }

    #[test]
    fn directory_separator_requires_typed_tree_directory_boundary_and_keeps_negative_members_lexical() {
        let (mut step, mut bindings, _) = invocation_fixture();
        let data = step.inputs["data"].clone();
        let RecipeOperation::Exec { runtime_mounts, .. } = &mut step.operation else {
            unreachable!()
        };
        runtime_mounts.insert("/mantle-rust-runtime".into(), "data".into());
        step.outputs.insert("tree".into(), RecipeOutput {
            kind: ArtifactKind::Tree,
            bytes_max: 4096,
        });
        for typed in [
            WordPart::Input {
                name: "data".into(),
                path: "not-created/include".into(),
            },
            WordPart::Runtime {
                mount: "/mantle-rust-runtime".into(),
                path: String::new(),
            },
            WordPart::Output {
                name: "tree".into(),
                path: "not-created".into(),
            },
        ] {
            let word = RecipeWord {
                parts: vec![
                    WordPart::Literal { text: "-B".into() },
                    typed,
                    WordPart::DirectorySeparator {},
                ],
            };
            validate_word(&step, &word).unwrap();
            validate_word_binding(&step, &word, &bindings).unwrap();
            let mut escape = word.clone();
            escape.parts.push(WordPart::Literal { text: "..".into() });
            assert!(validate_word(&step, &escape).is_err());
            let mut doubled = word;
            doubled.parts.push(WordPart::DirectorySeparator {});
            assert!(validate_word(&step, &doubled).is_err());
        }
        for parts in [
            vec![WordPart::DirectorySeparator {}],
            vec![WordPart::Literal { text: "-B".into() }, WordPart::DirectorySeparator {}],
            vec![
                WordPart::TextInput { name: "data".into() },
                WordPart::DirectorySeparator {},
            ],
        ] {
            assert!(validate_word(&step, &RecipeWord { parts }).is_err());
        }
        let first = WordPart::Input {
            name: "data".into(),
            path: "not-created/include".into(),
        };
        let second = WordPart::Runtime {
            mount: "/mantle-rust-runtime".into(),
            path: "also-not-created".into(),
        };
        let directories = RecipeWord {
            parts: vec![
                first.clone(),
                WordPart::DirectorySeparator {},
                WordPart::Literal { text: ":".into() },
                second,
                WordPart::DirectorySeparator {},
            ],
        };
        validate_word(&step, &directories).unwrap();
        validate_word_binding(&step, &directories, &bindings).unwrap();
        let trailing_empty = RecipeWord {
            parts: vec![first.clone(), WordPart::DirectorySeparator {}, WordPart::Literal {
                text: ":".into(),
            }],
        };
        validate_word(&step, &trailing_empty).unwrap();
        validate_word_binding(&step, &trailing_empty, &bindings).unwrap();
        for suffix in ["suffix", ":suffix", "/", "../", ".."] {
            let invalid = RecipeWord {
                parts: vec![first.clone(), WordPart::DirectorySeparator {}, WordPart::Literal {
                    text: suffix.into(),
                }],
            };
            assert!(validate_word(&step, &invalid).is_err());
        }
        let traversal = RecipeWord {
            parts: vec![
                WordPart::Input {
                    name: "data".into(),
                    path: "../outside".into(),
                },
                WordPart::DirectorySeparator {},
                WordPart::Literal { text: ":".into() },
                first,
                WordPart::DirectorySeparator {},
            ],
        };
        assert!(validate_word(&step, &traversal).is_err());
        bindings.get_mut(&data).unwrap().kind = ArtifactKind::Object;
        let word = RecipeWord {
            parts: vec![
                WordPart::Input {
                    name: "data".into(),
                    path: String::new(),
                },
                WordPart::DirectorySeparator {},
            ],
        };
        validate_word(&step, &word).unwrap();
        assert!(validate_word_binding(&step, &word, &bindings).is_err());
        assert!(validate_word_binding(&step, &directories, &bindings).is_err());
        assert_eq!(
            serde_json::to_string(&WordPart::DirectorySeparator {}).unwrap(),
            r#"{"kind":"directory_separator"}"#
        );
        let separator: WordPart = serde_json::from_str(r#"{"kind":"directory_separator"}"#).unwrap();
        assert_eq!(separator, WordPart::DirectorySeparator {});
        assert!(serde_json::from_str::<WordPart>(r#"{"kind":"directory_separator","path":".."}"#).is_err());
    }

    #[test]
    fn source_timeout_spans_accept_nested_arenas_and_reject_crossing_unknown_or_reversed_endpoints() {
        let mut recipe = text_recipe("stagex-transition", "first", "value", "value");
        for id in ["second", "third", "fourth"] {
            let mut step = recipe.steps[0].clone();
            step.id = id.into();
            recipe.steps.push(step);
        }
        let group = |first: &str, last: &str, seconds| RecipeTimeoutGroup {
            first_step: first.into(),
            last_step: last.into(),
            seconds_max: seconds,
        };
        recipe.timeout_groups = vec![
            group("second", "third", 2),
            group("first", "fourth", 10),
            group("first", "fourth", 9),
        ];
        assert_eq!(timeout_spans(&recipe).unwrap(), vec![(0, 3, 9), (0, 3, 10), (1, 2, 2)]);
        for invalid in [
            vec![group("first", "third", 10), group("second", "fourth", 2)],
            vec![group("first", "missing", 1)],
            vec![group("third", "second", 1)],
            vec![group("first", "first", 0)],
            vec![group("../first", "fourth", 1)],
        ] {
            recipe.timeout_groups = invalid;
            assert!(timeout_spans(&recipe).is_err());
        }
    }

    #[test]
    fn preparation_rejects_source_timeout_above_global_proof_cap() {
        for oversized in [false, true] {
            let root = tempfile::tempdir().unwrap();
            let (original, sources) = stage_binding_fixture(root.path());
            let path = &sources[&SourceAuthorityRole::StagexLineage];
            let mut recipe: SourceRecipe = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
            recipe.timeout_groups = vec![RecipeTimeoutGroup {
                first_step: recipe.steps[0].id.clone(),
                last_step: recipe.steps.last().unwrap().id.clone(),
                seconds_max: original.resource_bounds.elapsed_seconds_max + u64::from(oversized),
            }];
            fs::write(path, serde_json::to_vec(&recipe).unwrap()).unwrap();
            let mut inputs = original.source_inputs;
            for input in &mut inputs {
                if let Some(path) = sources.get(&input.role) {
                    (input.size_bytes, input.digest_blake3) = digest(path).unwrap();
                }
            }
            let proof = plan_source_built_fixed_point(SourceBuiltFixedPointPlanInput {
                proof_id: original.proof_id,
                logical_store_prefix: original.logical_store_prefix,
                source_inputs: inputs,
                initial_output_authority: InitialOutputAuthorityState {
                    stagex_transition_entries: 0,
                    native_provider_entries: 0,
                    rust_provider_entries: 0,
                    mantle_output_entries: 0,
                },
                policies: original.policies,
                resource_bounds: original.resource_bounds,
            })
            .unwrap();
            let result = prepare_binding_fixture(
                &proof,
                &sources,
                &root.path().join("runtime"),
                "stagex-transition",
                SourceAuthorityRole::StagexLineage,
                &[],
            );
            if oversized {
                assert!(result.err().unwrap().to_string().contains("source timeout exceeds proof elapsed bound"));
            } else {
                result.unwrap();
            }
        }
    }

    #[test]
    fn invocation_cwd_requires_one_typed_tree_path_and_rejects_static_conflicts() {
        let (mut step, mut bindings, mut invocation) = invocation_fixture();
        invocation.cwd = Some(RecipeWord {
            parts: vec![WordPart::Input {
                name: "data".into(),
                path: "package/nested".into(),
            }],
        });
        decode_invocation(&step, &serde_json::to_vec(&invocation).unwrap(), &bindings).unwrap();
        for invalid in [
            "/outside",
            "../outside",
            "package/../outside",
            "package//nested",
            "package/",
            ".",
            "package\0nested",
        ] {
            invocation.cwd = Some(RecipeWord {
                parts: vec![WordPart::Input {
                    name: "data".into(),
                    path: invalid.into(),
                }],
            });
            assert!(decode_invocation(&step, &serde_json::to_vec(&invocation).unwrap(), &bindings).is_err());
        }
        for parts in [
            vec![],
            vec![WordPart::Literal { text: "package".into() }],
            vec![WordPart::Input {
                name: "missing".into(),
                path: String::new(),
            }],
            vec![WordPart::TextInput { name: "data".into() }],
            vec![
                WordPart::Input {
                    name: "data".into(),
                    path: String::new(),
                },
                WordPart::DirectorySeparator {},
            ],
        ] {
            invocation.cwd = Some(RecipeWord { parts });
            assert!(decode_invocation(&step, &serde_json::to_vec(&invocation).unwrap(), &bindings).is_err());
        }
        invocation.cwd = Some(RecipeWord {
            parts: vec![WordPart::Input {
                name: "data".into(),
                path: String::new(),
            }],
        });
        let data = step.inputs["data"].clone();
        bindings.get_mut(&data).unwrap().kind = ArtifactKind::Text;
        assert!(
            decode_invocation(&step, &serde_json::to_vec(&invocation).unwrap(), &bindings)
                .err()
                .unwrap()
                .to_string()
                .contains("Tree root")
        );
        bindings.get_mut(&data).unwrap().kind = ArtifactKind::Tree;
        let RecipeOperation::Exec { cwd_input, .. } = &mut step.operation else {
            unreachable!()
        };
        *cwd_input = Some("data".into());
        assert!(
            decode_invocation(&step, &serde_json::to_vec(&invocation).unwrap(), &bindings)
                .err()
                .unwrap()
                .to_string()
                .contains("conflicts")
        );
        let RecipeOperation::Exec { cwd_input, .. } = &mut step.operation else {
            unreachable!()
        };
        *cwd_input = None;
        step.outputs.insert("workspace".into(), RecipeOutput {
            kind: ArtifactKind::Tree,
            bytes_max: 4096,
        });
        invocation.cwd = Some(RecipeWord {
            parts: vec![WordPart::Output {
                name: "workspace".into(),
                path: "package".into(),
            }],
        });
        decode_invocation(&step, &serde_json::to_vec(&invocation).unwrap(), &bindings).unwrap();
        step.outputs.get_mut("workspace").unwrap().kind = ArtifactKind::Text;
        assert!(decode_invocation(&step, &serde_json::to_vec(&invocation).unwrap(), &bindings).is_err());
    }

    #[test]
    fn invocation_cwd_field_is_canonical_and_empty_extension_is_omitted() {
        let (step, bindings, invocation) = invocation_fixture();
        let canonical = serde_json::to_string(&invocation).unwrap();
        let explicit_empty = format!("{},\"cwd\":null}}", &canonical[..canonical.len() - 1]);
        assert!(
            decode_invocation(&step, explicit_empty.as_bytes(), &bindings)
                .err()
                .unwrap()
                .to_string()
                .contains("noncanonical")
        );
        let duplicate = format!("{},\"cwd\":null,\"cwd\":null}}", &canonical[..canonical.len() - 1]);
        assert!(decode_invocation(&step, duplicate.as_bytes(), &bindings).is_err());
        let mut padded = vec![b' '];
        padded.extend_from_slice(canonical.as_bytes());
        assert!(decode_invocation(&step, &padded, &bindings).err().unwrap().to_string().contains("noncanonical"));
    }

    #[test]
    fn workspace_mounts_preserve_only_declared_output_tree_namespaces() {
        let (mut step, bindings, mut invocation) = invocation_fixture();
        step.outputs.insert("generated".into(), RecipeOutput {
            kind: ArtifactKind::Tree,
            bytes_max: 4096,
        });
        for destination in ["/tmp", "/build", "/custom/store/provider/tmp"] {
            invocation.workspace_mounts = BTreeMap::from([(destination.into(), RecipeWorkspaceMount {
                output: "generated".into(),
                path: "checkpoint/tmp".into(),
            })]);
            validate_workspace_mounts(&step, &invocation, Some("/custom/store")).unwrap();
        }
        for destination in [
            "/",
            "/home/user",
            "/inputs/workspace",
            "/outputs/generated",
            "/work",
            "/proc/self",
            "/dev",
            "/tmp/arbitrary",
            "/build/arbitrary",
            "/custom/store",
            "/custom/storeish/provider",
            "/mantle/store/provider",
            "/custom/store/a/../b",
            "relative",
        ] {
            invocation.workspace_mounts = BTreeMap::from([(destination.into(), RecipeWorkspaceMount {
                output: "generated".into(),
                path: String::new(),
            })]);
            assert!(validate_workspace_mounts(&step, &invocation, Some("/custom/store")).is_err(), "{destination}");
        }
        for (output, path) in [
            ("missing", ""),
            ("stdout", ""),
            ("generated", "../outside"),
            ("generated", "/absolute"),
        ] {
            invocation.workspace_mounts = BTreeMap::from([("/tmp".into(), RecipeWorkspaceMount {
                output: output.into(),
                path: path.into(),
            })]);
            assert!(validate_workspace_mounts(&step, &invocation, Some("/custom/store")).is_err());
        }
        invocation.workspace_mounts = BTreeMap::from([
            ("/custom/store/provider".into(), RecipeWorkspaceMount {
                output: "generated".into(),
                path: String::new(),
            }),
            ("/custom/store/provider/nested".into(), RecipeWorkspaceMount {
                output: "generated".into(),
                path: String::new(),
            }),
        ]);
        invocation.workspace_mounts.insert("/custom/store/provider-".into(), RecipeWorkspaceMount {
            output: "generated".into(),
            path: String::new(),
        });
        assert!(validate_workspace_mounts(&step, &invocation, Some("/custom/store")).is_err());
        invocation.workspace_mounts.remove("/custom/store/provider/nested");
        let RecipeOperation::Exec { runtime_mounts, .. } = &mut step.operation else {
            unreachable!()
        };
        runtime_mounts.insert("/custom/store/provider/lib".into(), "data".into());
        assert!(validate_workspace_mounts(&step, &invocation, Some("/custom/store")).is_err());
        let RecipeOperation::Exec { runtime_mounts, .. } = &mut step.operation else {
            unreachable!()
        };
        runtime_mounts.clear();
        invocation.workspace_mounts = BTreeMap::from([("/tmp".into(), RecipeWorkspaceMount {
            output: "generated".into(),
            path: String::new(),
        })]);
        invocation.arguments.push(RecipeWord {
            parts: vec![
                WordPart::Workspace {
                    mount: "/tmp".into(),
                    path: "not-created".into(),
                },
                WordPart::DirectorySeparator {},
            ],
        });
        invocation.cwd = Some(RecipeWord {
            parts: vec![WordPart::Workspace {
                mount: "/tmp".into(),
                path: String::new(),
            }],
        });
        decode_invocation(&step, &serde_json::to_vec(&invocation).unwrap(), &bindings).unwrap();
        invocation.workspace_mounts.clear();
        assert!(decode_invocation(&step, &serde_json::to_vec(&invocation).unwrap(), &bindings).is_err());
    }

    #[test]
    fn installed_image_path_is_only_a_readonly_produced_tree_member_not_an_image_selector() {
        let (mut step, mut bindings, mut invocation) = invocation_fixture();
        invocation.executable_path = Some(RecipeWord {
            parts: vec![WordPart::Input {
                name: "data".into(),
                path: "bin/llvm-config".into(),
            }],
        });
        decode_invocation(&step, &serde_json::to_vec(&invocation).unwrap(), &bindings).unwrap();
        let RecipeOperation::Exec { runtime_mounts, .. } = &mut step.operation else {
            unreachable!()
        };
        runtime_mounts.insert("/mantle/store/provider".into(), "data".into());
        invocation.executable_path = Some(RecipeWord {
            parts: vec![WordPart::Runtime {
                mount: "/mantle/store/provider".into(),
                path: "bin/llvm-config".into(),
            }],
        });
        decode_invocation(&step, &serde_json::to_vec(&invocation).unwrap(), &bindings).unwrap();
        for parts in [
            vec![],
            vec![WordPart::Literal {
                text: "llvm-config".into(),
            }],
            vec![WordPart::Output {
                name: "stdout".into(),
                path: "llvm-config".into(),
            }],
            vec![WordPart::Workspace {
                mount: "/tmp".into(),
                path: "llvm-config".into(),
            }],
            vec![WordPart::Input {
                name: "data".into(),
                path: String::new(),
            }],
            vec![WordPart::Input {
                name: "data".into(),
                path: "../llvm-config".into(),
            }],
            vec![
                WordPart::Input {
                    name: "data".into(),
                    path: "bin/llvm-config".into(),
                },
                WordPart::DirectorySeparator {},
            ],
        ] {
            invocation.executable_path = Some(RecipeWord { parts });
            assert!(decode_invocation(&step, &serde_json::to_vec(&invocation).unwrap(), &bindings).is_err());
        }
        invocation.executable_path = Some(RecipeWord {
            parts: vec![WordPart::Input {
                name: "data".into(),
                path: "bin/llvm-config".into(),
            }],
        });
        let data = step.inputs["data"].clone();
        bindings.get_mut(&data).unwrap().kind = ArtifactKind::Text;
        assert!(decode_invocation(&step, &serde_json::to_vec(&invocation).unwrap(), &bindings).is_err());
        bindings.get_mut(&data).unwrap().kind = ArtifactKind::Tree;
        let imported = ArtifactRef::Source {
            role: SourceAuthorityRole::StagexSourceBundle,
            path: String::new(),
        };
        let mut binding = bindings[&data].clone();
        binding.reference = imported.clone();
        bindings.insert(imported.clone(), binding);
        step.inputs.insert("data".into(), imported);
        assert!(decode_invocation(&step, &serde_json::to_vec(&invocation).unwrap(), &bindings).is_err());
    }

    #[test]
    fn phase_state_projection_source_accepts_only_normalized_relative_members() {
        let mut step = RecipeStep {
            id: "observe".into(),
            inputs: BTreeMap::from([("prior".into(), ArtifactRef::Produced {
                action_id: "stagex-transition/prior".into(),
                output_id: "tree".into(),
            })]),
            outputs: BTreeMap::from([("envelope".into(), RecipeOutput {
                kind: ArtifactKind::Tree,
                bytes_max: 4096,
            })]),
            operation: RecipeOperation::ObservePhase {
                phase: "conditional".into(),
                output: "envelope".into(),
                state: Some(RecipePhaseState {
                    output: "checkpoint".into(),
                    skipped_input: "prior".into(),
                    skipped_path: String::new(),
                }),
            },
        };
        for member in ["", "state", "checkpoint/state"] {
            let RecipeOperation::ObservePhase { state: Some(state), .. } = &mut step.operation else {
                unreachable!()
            };
            state.skipped_path = member.into();
            validate_operation(&step).unwrap();
        }
        for member in [
            "/state",
            "../state",
            "state/../state",
            "state//nested",
            "state/",
            "state\0nested",
        ] {
            let RecipeOperation::ObservePhase { state: Some(state), .. } = &mut step.operation else {
                unreachable!()
            };
            state.skipped_path = member.into();
            assert!(validate_operation(&step).is_err());
        }
    }

    #[test]
    fn relative_source_operands_require_cwd_authority_without_changing_original_spelling() {
        let (step, bindings, mut invocation) = invocation_fixture();
        invocation.cwd = Some(RecipeWord {
            parts: vec![WordPart::Input {
                name: "data".into(),
                path: "package".into(),
            }],
        });
        for spelling in ["conftest.c", "./conftest.c"] {
            invocation.arguments = vec![RecipeWord {
                parts: vec![WordPart::CwdRelative { path: spelling.into() }],
            }];
            let actual = decode_invocation(&step, &serde_json::to_vec(&invocation).unwrap(), &bindings).unwrap();
            assert_eq!(actual.arguments[0].parts, vec![WordPart::CwdRelative { path: spelling.into() }]);
        }
        for path in ["../conftest.c", "./../conftest.c", "/conftest.c", "package//conftest.c"] {
            invocation.arguments = vec![RecipeWord {
                parts: vec![WordPart::CwdRelative { path: path.into() }],
            }];
            assert!(decode_invocation(&step, &serde_json::to_vec(&invocation).unwrap(), &bindings).is_err());
        }
        invocation.arguments = vec![RecipeWord {
            parts: vec![WordPart::CwdRelative {
                path: "conftest.c".into(),
            }],
        }];
        invocation.cwd = None;
        assert!(decode_invocation(&step, &serde_json::to_vec(&invocation).unwrap(), &bindings).is_err());
    }

    #[test]
    fn inherited_descriptor_tables_reject_missing_slots_alias_divergence_and_readonly_root_writes() {
        let (mut step, bindings, mut invocation) = invocation_fixture();
        for output in ["times", "descriptors"] {
            step.outputs.insert(output.into(), RecipeOutput {
                kind: ArtifactKind::Text,
                bytes_max: 4096,
            });
        }
        step.outputs.insert("workspace".into(), RecipeOutput {
            kind: ArtifactKind::Tree,
            bytes_max: 4096,
        });
        let RecipeOperation::Exec { command_state, .. } = &mut step.operation else {
            unreachable!()
        };
        *command_state = Some(RecipeCommandState {
            workspace_output: "workspace".into(),
            prior_times_input: None,
            times_output: "times".into(),
            descriptors_output: "descriptors".into(),
            descriptor_files_output: None,
        });
        let primary = RecipeDescriptor {
            target: RecipeDescriptorTarget::NullDevice,
            access: RecipeDescriptorAccess::Write,
            flags: 0,
            offset: 0,
            alias: 1,
        };
        let mut slots = vec![None; 64];
        slots[1] = Some(primary.clone());
        slots[2] = Some(primary);
        invocation.descriptors = Some(RecipeDescriptorTable { slots });
        decode_invocation(&step, &serde_json::to_vec(&invocation).unwrap(), &bindings).unwrap();
        invocation.descriptors.as_mut().unwrap().slots[2].as_mut().unwrap().offset = 1;
        assert!(decode_invocation(&step, &serde_json::to_vec(&invocation).unwrap(), &bindings).is_err());
        invocation.descriptors.as_mut().unwrap().slots[2] = None;
        invocation.descriptors.as_mut().unwrap().slots[1].as_mut().unwrap().target = RecipeDescriptorTarget::File {
            path: RecipeWord {
                parts: vec![WordPart::Input {
                    name: "data".into(),
                    path: "log".into(),
                }],
            },
        };
        assert!(decode_invocation(&step, &serde_json::to_vec(&invocation).unwrap(), &bindings).is_err());
        invocation.descriptors.as_mut().unwrap().slots[1].as_mut().unwrap().target = RecipeDescriptorTarget::NullDevice;
        invocation.descriptors.as_mut().unwrap().slots.pop();
        assert!(decode_invocation(&step, &serde_json::to_vec(&invocation).unwrap(), &bindings).is_err());
    }

    #[test]
    fn readonly_input_catalog_accounts_only_declared_produced_trees() {
        let (mut step, mut bindings, mut invocation) = invocation_fixture();
        invocation.arguments.clear();
        assert!(decode_invocation(&step, &serde_json::to_vec(&invocation).unwrap(), &bindings).is_err());
        invocation.readonly_input_roots = vec!["data".into()];
        assert_eq!(
            decode_invocation(&step, &serde_json::to_vec(&invocation).unwrap(), &bindings)
                .unwrap()
                .readonly_input_roots,
            vec!["data"]
        );
        for roots in [vec!["data", "data"], vec!["seed"], vec!["unknown"], vec!["data/path"]] {
            invocation.readonly_input_roots = roots.into_iter().map(String::from).collect();
            assert!(decode_invocation(&step, &serde_json::to_vec(&invocation).unwrap(), &bindings).is_err());
        }
        invocation.readonly_input_roots = vec!["data".into()];
        let data = step.inputs["data"].clone();
        bindings.get_mut(&data).unwrap().kind = ArtifactKind::Executable;
        assert!(decode_invocation(&step, &serde_json::to_vec(&invocation).unwrap(), &bindings).is_err());
        let mut binding = bindings.remove(&data).unwrap();
        binding.kind = ArtifactKind::Tree;
        let source = ArtifactRef::Source {
            role: SourceAuthorityRole::StagexSeed,
            path: String::new(),
        };
        binding.reference = source.clone();
        bindings.insert(source.clone(), binding);
        step.inputs.insert("data".into(), source);
        assert!(decode_invocation(&step, &serde_json::to_vec(&invocation).unwrap(), &bindings).is_err());
    }

    #[test]
    fn readonly_input_catalog_keeps_uncatalogued_inputs_rejected_and_names_sorted() {
        let (mut step, mut bindings, mut invocation) = invocation_fixture();
        let extra = ArtifactRef::Produced {
            action_id: "stagex-transition/dependency".into(),
            output_id: "tree".into(),
        };
        bindings.insert(extra.clone(), ArtifactBinding {
            reference: extra.clone(),
            path: PathBuf::from("extra"),
            kind: ArtifactKind::Tree,
            bytes_max: 4096,
        });
        step.inputs.insert("extra".into(), extra);
        assert!(decode_invocation(&step, &serde_json::to_vec(&invocation).unwrap(), &bindings).is_err());
        invocation.readonly_input_roots = vec!["extra".into(), "data".into()];
        assert!(decode_invocation(&step, &serde_json::to_vec(&invocation).unwrap(), &bindings).is_err());
        invocation.readonly_input_roots = vec!["data".into(), "extra".into()];
        assert_eq!(
            decode_invocation(&step, &serde_json::to_vec(&invocation).unwrap(), &bindings)
                .unwrap()
                .readonly_input_roots,
            vec!["data", "extra"]
        );
    }

    #[test]
    fn source_creation_masks_reject_bits_outside_permissions() {
        let (step, bindings, mut invocation) = invocation_fixture();
        for mask in [0, 0o077, 0o777] {
            invocation.process_attributes = Some(RecipeProcessAttributes {
                umask: Some(mask),
                ..RecipeProcessAttributes::default()
            });
            assert_eq!(
                decode_invocation(&step, &serde_json::to_vec(&invocation).unwrap(), &bindings)
                    .unwrap()
                    .process_attributes
                    .unwrap()
                    .umask,
                Some(mask)
            );
        }
        invocation.process_attributes.as_mut().unwrap().umask = Some(0o1000);
        assert!(decode_invocation(&step, &serde_json::to_vec(&invocation).unwrap(), &bindings).is_err());
    }
}
