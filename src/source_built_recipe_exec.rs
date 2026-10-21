//! Opt-in source recipe execution. No default scheduler path calls this module.
//! Every external command is isolated with explicitly bound inputs and a fresh
//! output directory. An inherited seccomp listener admits only the exact sandbox
//! launch followed by the exact produced image; every later exec is denied.
//! Output workspaces are fresh or bounded immutable Produced-state copies.
//! Source-owned exit sets accept only actual normal exits. Reserved status files
//! contain the decimal code and newline, never an author-predicted result.
//! Merged capture duplicates the stdout descriptor for stderr before launch;
//! stdout then contains the genuine merged writes and the stderr artifact is empty.
//! Generated argument requests and source-data operands share authenticated
//! per-producer snapshots before the exact protected leaf is released. Typed
//! runtime aliases expose only declared bounded Produced artifacts read-only.
//! Authenticated cwd selects one existing no-symlink typed Tree member. Workspace
//! aliases expose only captured output Trees in source-owned writable namespaces.
//! Installed-image paths bind and execute only the identical separately Produced
//! file, preserving actual image context without granting prefix execution rights.
//! Inclusive original timeout spans share monotonic deadlines across controller,
//! guard, leaf and observation work. Protected workers receive exact remaining
//! durations; expiry fails the attempt, never normal code124 or timeout recovery.
//! Compact controllers instantiate only actual turns through this same runner.
//! Source-authenticated deadlines survive continuation; explicit file limits
//! lower the real child RLIMIT_FSIZE independently of artifact Tree bounds.
//! Exact inherited fd0..63 OFDs are injected at the second exec notification,
//! after bwrap initialization, with declared closed slots closing on raw exec.
//! Actual argv0 is rendered separately from the authenticated executable image.
use std::collections::BTreeMap;
use std::fs::File;
use std::fs::OpenOptions;
use std::fs::{self};
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use std::time::Instant;

use crate::errors::RunError;
use crate::source_built_fixed_point::ProofPolicyRole;
use crate::source_built_fixed_point::StageAuthorityInput;
use crate::source_built_fixed_point_action_trust::ActionExecutable;
use crate::source_built_fixed_point_action_trust::ActionExecutionEvent;
use crate::source_built_fixed_point_action_trust::ActionInput;
use crate::source_built_fixed_point_action_trust::ActionSpec;
use crate::source_built_fixed_point_action_trust::ExecutionEventKind;
use crate::source_built_fixed_point_action_trust::GuardOutcome;
use crate::source_built_fixed_point_action_trust::PhaseOutcome;
use crate::source_built_fixed_point_action_trust::canonical_exit_status;
use crate::source_built_recipe::*;

#[path = "source_built_recipe_command_state.rs"]
mod command_state;

#[cfg(target_os = "linux")]
#[path = "source_built_recipe_exec_descriptor_transport.rs"]
mod descriptor_transport;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct RecipeCompletion {
    pub action_id: String,
    pub exit_code: i32,
    pub stdout_path: PathBuf,
    pub stdout_digest_blake3: String,
    pub stderr_path: PathBuf,
    pub stderr_digest_blake3: String,
}
#[derive(Clone, Debug, serde::Serialize)]
struct RawExec {
    pid: u32,
    syscall: i32,
    tracee_path: PathBuf,
    resolved_path: PathBuf,
    digest_blake3: String,
    allowed: bool,
    reason: String,
    is_exec: bool,
}
struct LaunchResult {
    status: Option<i32>,
    exec_count: usize,
    violation: bool,
    failure: Option<String>,
}
struct PendingObservation {
    row: RawExec,
    decision: std::sync::mpsc::SyncSender<bool>,
}
fn create(path: &Path) -> Result<File, RunError> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|e| error(format!("creating {}: {e}", path.display())))
}
fn journal(file: &mut File, value: &impl serde::Serialize) -> Result<(), RunError> {
    let bytes = serde_json::to_vec(value).map_err(|e| error(e.to_string()))?;
    file.write_all(&bytes)
        .and_then(|()| file.write_all(b"\n"))
        .and_then(|()| file.sync_data())
        .map_err(|e| error(format!("preserving execution provenance: {e}")))
}
pub(crate) fn input_digests(prepared: &PreparedRecipe, action: &ActionSpec) -> Result<Vec<String>, RunError> {
    action
        .inputs
        .iter()
        .map(|input| input_digest(prepared, &prepared.bindings, input, None, None))
        .collect()
}
fn input_digest(
    prepared: &PreparedRecipe,
    bindings: &impl ArtifactBindings,
    input: &ActionInput,
    phases: Option<&BTreeMap<String, PhaseOutcome>>,
    controllers: Option<&BTreeMap<String, crate::source_built_fixed_point_action_trust::ActionControllerOutcome>>,
) -> Result<String, RunError> {
    match input {
        ActionInput::Stage {
            authority: StageAuthorityInput::Source { role },
        } => {
            let path = prepared.sources.get(role).ok_or_else(|| error("missing source path"))?;
            let measured = digest(path)?.1;
            let admitted = prepared
                .proof
                .source_inputs
                .iter()
                .find(|s| s.role == *role)
                .ok_or_else(|| error("missing source role"))?;
            if measured != admitted.digest_blake3 {
                return Err(error("source input changed after planning"));
            }
            Ok(measured)
        }
        ActionInput::Stage {
            authority: StageAuthorityInput::Policy { role },
        } => Ok(match role {
            ProofPolicyRole::Closure => &prepared.proof.policies.closure_policy_digest_blake3,
            ProofPolicyRole::Hermeticity => &prepared.proof.policies.hermeticity_policy_digest_blake3,
            ProofPolicyRole::ProtectedExecution => &prepared.proof.policies.protected_execution_policy_digest_blake3,
            ProofPolicyRole::Effect => &prepared.proof.policies.effect_policy_digest_blake3,
            ProofPolicyRole::Normalization => &prepared.proof.policies.normalization_policy_digest_blake3,
        }
        .clone()),
        ActionInput::Produced { action_id, output_id } => {
            let reference = ArtifactRef::Produced {
                action_id: action_id.clone(),
                output_id: output_id.clone(),
            };
            let binding = bindings.get(&reference).ok_or_else(|| error("unknown producer at runtime"))?;
            let (size, hash) = digest(&binding.path)?;
            if size > binding.bytes_max {
                return Err(error("produced input exceeds declared bound"));
            }
            Ok(hash)
        }
        ActionInput::PhaseOutcome { action_id } => phases
            .and_then(|phases| phases.get(action_id))
            .ok_or_else(|| error("missing authenticated phase outcome"))?
            .digest()
            .map_err(error),
        ActionInput::ControllerOutcome { controller_id } => controllers
            .and_then(|outcomes| outcomes.get(controller_id))
            .ok_or_else(|| error("missing actual authenticated controller done outcome"))?
            .digest()
            .map_err(error),
        _ => Err(error("path-only stage output authority")),
    }
}
pub(crate) fn event(
    action: &ActionSpec,
    kind: ExecutionEventKind,
    exe: String,
    inputs: Vec<String>,
    outputs: Vec<(String, String)>,
    completed: bool,
    violation: bool,
    elapsed: u64,
    disk: u64,
) -> ActionExecutionEvent {
    ActionExecutionEvent {
        action_id: action.id.clone(),
        kind,
        normal_exit_code: None,
        executable_digest_blake3: exe,
        executable_producer: match &action.executable {
            ActionExecutable::Produced { action_id, output_id } => Some((action_id.clone(), output_id.clone())),
            _ => None,
        },
        inputs: action.inputs.clone(),
        input_digests_blake3: inputs,
        produced_outputs: outputs,
        locality: "local".into(),
        completed_locally: completed,
        cache_hit: false,
        live_fetch: false,
        cargo_invocation: false,
        fallback: false,
        protected_exec_violation: violation,
        elapsed_seconds: elapsed,
        disk_bytes: disk,
    }
}
struct CommandOperands<'a> {
    prepared: &'a PreparedRecipe,
    dynamic_bindings: Option<&'a BTreeMap<ArtifactRef, ArtifactBinding>>,
    step: &'a RecipeStep,
    action: &'a ActionSpec,
    measured_inputs: &'a [String],
    known_outputs: &'a BTreeMap<ArtifactRef, (PathBuf, String)>,
    texts: BTreeMap<ArtifactRef, String>,
}
impl<'a> CommandOperands<'a> {
    fn binding(&self, reference: &ArtifactRef) -> &'a ArtifactBinding {
        self.dynamic_bindings
            .and_then(|bindings| bindings.get(reference))
            .or_else(|| self.prepared.bindings.get(reference))
            .expect("admitted command binding")
    }
    fn bytes(&self, name: &str, maximum: u64, purpose: &str) -> Result<Vec<u8>, RunError> {
        let reference = &self.step.inputs[name];
        let ArtifactRef::Produced { action_id, output_id } = reference else {
            return Err(error(format!("{purpose} lacks Produced authority")));
        };
        let binding = self.binding(reference);
        if binding.kind != ArtifactKind::Text {
            return Err(error(format!("{purpose} is not exact Produced Text")));
        }
        let index = self.action.inputs.iter().position(|input| matches!(input, ActionInput::Produced { action_id: producer, output_id: output } if producer == action_id && output == output_id))
            .ok_or_else(|| error(format!("{purpose} is not a measured action input")))?;
        if self
            .known_outputs
            .get(reference)
            .is_some_and(|(_, authenticated)| authenticated != &self.measured_inputs[index])
        {
            return Err(error(format!("{purpose} differs from its authenticated producer bytes")));
        }
        let limit = binding.bytes_max.min(maximum);
        let metadata = fs::symlink_metadata(&binding.path).map_err(|e| error(e.to_string()))?;
        if !metadata.is_file() || metadata.len() > limit {
            return Err(error(format!("{purpose} exceeds bound or is not a regular file")));
        }
        let mut bytes = Vec::with_capacity(metadata.len() as usize);
        File::open(&binding.path)
            .map_err(|e| error(e.to_string()))?
            .take(limit + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| error(e.to_string()))?;
        if bytes.len() as u64 > limit || blake3::hash(&bytes).to_hex().as_str() != self.measured_inputs[index] {
            return Err(error(format!("{purpose} differs from its authenticated producer bytes")));
        }
        Ok(bytes)
    }
    fn authenticate_produced(&self, name: &str, kind: ArtifactKind, purpose: &str) -> Result<String, RunError> {
        let reference = &self.step.inputs[name];
        let ArtifactRef::Produced { action_id, output_id } = reference else {
            return Err(error(format!("{purpose} requires Produced authority")));
        };
        let binding = self.binding(reference);
        if binding.kind != kind {
            return Err(error(format!("{purpose} has the wrong artifact kind")));
        }
        let index = self.action.inputs.iter().position(|input| matches!(input, ActionInput::Produced { action_id: producer, output_id: output } if producer == action_id && output == output_id))
            .ok_or_else(|| error(format!("{purpose} is not a measured input")))?;
        let hash = verify_output(binding)?.1;
        if hash != self.measured_inputs[index]
            || self.known_outputs.get(reference).is_some_and(|(_, expected)| expected != &hash)
        {
            return Err(error(format!("{purpose} differs from authenticated whole input")));
        }
        Ok(hash)
    }
    fn ensure_text(&mut self, name: &str, purpose: &str) -> Result<(), RunError> {
        let reference = &self.step.inputs[name];
        if !self.texts.contains_key(reference) {
            let bytes = self.bytes(name, RECIPE_BYTES_MAX, purpose)?;
            let text = String::from_utf8(bytes).map_err(|e| error(format!("{purpose} is not UTF-8: {e}")))?;
            if text.contains('\0') {
                return Err(error(format!("{purpose} contains NUL")));
            }
            self.texts.insert(reference.clone(), text);
        }
        Ok(())
    }
    fn invocation(&mut self) -> Result<Option<RecipeInvocation>, RunError> {
        let step = self.step;
        let RecipeOperation::Exec {
            invocation_input: Some(name),
            ..
        } = &step.operation
        else {
            return Ok(None);
        };
        self.ensure_text(name, "invocation request")?;
        let bindings = BindingOverlay {
            base: &self.prepared.bindings,
            dynamic: self.dynamic_bindings,
        };
        let invocation = decode_invocation(step, self.texts[&step.inputs[name]].as_bytes(), &bindings)?;
        validate_workspace_mounts(step, &invocation, Some(&self.prepared.proof.logical_store_prefix))?;
        Ok(Some(invocation))
    }
    fn word(&mut self, word: &RecipeWord) -> Result<String, RunError> {
        let mut result = String::new();
        for part in &word.parts {
            match part {
                WordPart::Literal { text } => result.push_str(text),
                WordPart::Input { name, path } => {
                    result.push_str("/inputs/");
                    result.push_str(name);
                    if !path.is_empty() {
                        result.push('/');
                        result.push_str(path);
                    }
                }
                WordPart::Output { name, path } => {
                    result.push_str("/outputs/");
                    result.push_str(name);
                    if !path.is_empty() {
                        result.push('/');
                        result.push_str(path);
                    }
                }
                WordPart::Runtime { mount, path } | WordPart::Workspace { mount, path } => {
                    result.push_str(mount);
                    if !path.is_empty() {
                        result.push('/');
                        result.push_str(path);
                    }
                }
                WordPart::DirectorySeparator {} => result.push('/'),
                WordPart::CwdRelative { path } => result.push_str(path),
                WordPart::TextInput { name } | WordPart::TextSlice { name, .. } => {
                    self.ensure_text(name, "text operand")?;
                    let reference = &self.step.inputs[name];
                    let whole = &self.texts[reference];
                    let text = if let WordPart::TextSlice {
                        offset_bytes,
                        length_bytes,
                        ..
                    } = part
                    {
                        let end = offset_bytes
                            .checked_add(*length_bytes)
                            .ok_or_else(|| error("text slice byte range overflow"))?;
                        let start =
                            usize::try_from(*offset_bytes).map_err(|_| error("text slice byte offset overflow"))?;
                        let end = usize::try_from(end).map_err(|_| error("text slice byte length overflow"))?;
                        whole.get(start..end).ok_or_else(|| error("text slice exceeds actual bytes or splits UTF-8"))?
                    } else {
                        whole.as_str()
                    };
                    if text.len() > WORD_BYTES_MAX - result.len() {
                        return Err(error("text operand exceeds command word bound"));
                    }
                    result.push_str(text);
                }
            }
            if result.len() > WORD_BYTES_MAX {
                return Err(error(format!("command word exceeds bound for {}", self.step.id)));
            }
        }
        Ok(result)
    }
}
fn copy_tree(source: &Path, target: &Path, budget: &mut u64) -> Result<(), RunError> {
    let metadata = fs::symlink_metadata(source).map_err(|e| error(e.to_string()))?;
    if metadata.is_file() {
        *budget = budget.checked_sub(metadata.len()).ok_or_else(|| error("copy exceeds output byte bound"))?;
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| error(e.to_string()))?;
        }
        let mut input = File::open(source).map_err(|e| error(e.to_string()))?;
        let mut output = create(target)?;
        std::io::copy(&mut input, &mut output).map_err(|e| error(e.to_string()))?;
        output.sync_all().map_err(|e| error(e.to_string()))?;
        fs::set_permissions(target, metadata.permissions()).map_err(|e| error(e.to_string()))?;
        return Ok(());
    }
    if metadata.file_type().is_symlink() {
        *budget = budget.checked_sub(metadata.len()).ok_or_else(|| error("copy exceeds output byte bound"))?;
        let link = fs::read_link(source).map_err(|e| error(e.to_string()))?;
        if link.is_absolute() || link.components().any(|p| matches!(p, std::path::Component::ParentDir)) {
            return Err(error("assembled source tree symlink escapes its artifact"));
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| error(e.to_string()))?;
        }
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(link, target).map_err(|e| error(e.to_string()))?;
            return Ok(());
        }
        #[cfg(not(unix))]
        {
            return Err(error("tree symlinks require Unix"));
        }
    }
    if !metadata.is_dir() {
        return Err(error("unsupported tree entry"));
    }
    fs::create_dir_all(target).map_err(|e| error(e.to_string()))?;
    let mut entries = fs::read_dir(source)
        .map_err(|e| error(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| error(e.to_string()))?;
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        copy_tree(&entry.path(), &target.join(entry.file_name()), budget)?;
    }
    Ok(())
}
fn installed_image_member(root: &Path, member: &str, expected_image: &str) -> Result<PathBuf, RunError> {
    relative(member, false)?;
    if !fs::symlink_metadata(root).map_err(|e| error(e.to_string()))?.is_dir() {
        return Err(error("installed image root is not an actual directory"));
    }
    let selected = source_member(root, member)?;
    let metadata = fs::symlink_metadata(&selected).map_err(|e| error(e.to_string()))?;
    if !metadata.is_file() {
        return Err(error("installed image member is not a regular executable file"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o111 == 0 {
            return Err(error("installed image member lacks execute permission"));
        }
    }
    if digest(&selected)?.1 != expected_image {
        return Err(error("installed image member differs from fixed Produced executable"));
    }
    Ok(selected)
}

fn rooted_directory(root: &Path, member: &str) -> Result<PathBuf, RunError> {
    if !fs::symlink_metadata(root).map_err(|e| error(e.to_string()))?.is_dir() {
        return Err(error("rooted directory anchor is not an actual directory"));
    }
    let selected = source_member(root, member)?;
    if !fs::symlink_metadata(&selected).map_err(|e| error(e.to_string()))?.is_dir() {
        return Err(error("selected rooted member is not an actual directory"));
    }
    Ok(selected)
}

// Pending wiring in the source-only preparation path (Cairn prove-source-built-mantle-fixed-point
// I3).
#[allow(dead_code)]
fn observe_phase(
    prepared: &PreparedRecipe,
    step: &RecipeStep,
    output_dir: &Path,
    phases: &BTreeMap<String, PhaseOutcome>,
    action: &ActionSpec,
    measured_inputs: &[String],
    known_outputs: &BTreeMap<ArtifactRef, (PathBuf, String)>,
) -> Result<(), RunError> {
    observe_phase_bindings(
        prepared,
        &prepared.bindings,
        step,
        output_dir,
        phases,
        action,
        measured_inputs,
        known_outputs,
    )
}
fn observe_phase_bindings(
    prepared: &PreparedRecipe,
    bindings: &impl ArtifactBindings,
    step: &RecipeStep,
    output_dir: &Path,
    phases: &BTreeMap<String, PhaseOutcome>,
    action: &ActionSpec,
    measured_inputs: &[String],
    known_outputs: &BTreeMap<ArtifactRef, (PathBuf, String)>,
) -> Result<(), RunError> {
    let RecipeOperation::ObservePhase { phase, output, state } = &step.operation else {
        return Err(error("non-observer passed to phase observer"));
    };
    let phase_id = format!("{}/{}", prepared.recipe.stage_id, phase);
    let outcome = phases.get(&phase_id).ok_or_else(|| error("phase has no authenticated decision or completion"))?;
    let target = output_dir.join(output);
    fs::create_dir(&target).map_err(|e| error(e.to_string()))?;
    let mut budget = step.outputs[output].bytes_max;
    let encoded = serde_json::to_vec(outcome).map_err(|e| error(e.to_string()))?;
    budget = budget
        .checked_sub(encoded.len() as u64)
        .ok_or_else(|| error("phase envelope exceeds output bound"))?;
    let mut outcome_file = create(&target.join("outcome.json"))?;
    outcome_file
        .write_all(&encoded)
        .and_then(|()| outcome_file.sync_all())
        .map_err(|e| error(e.to_string()))?;
    if let PhaseOutcome::Executed { output_digests } = outcome {
        fs::create_dir(target.join("outputs")).map_err(|e| error(e.to_string()))?;
        for (output_id, expected) in output_digests {
            let reference = ArtifactRef::Produced {
                action_id: phase_id.clone(),
                output_id: output_id.clone(),
            };
            let binding = bindings.get(&reference).ok_or_else(|| error("unknown observed phase output"))?;
            if verify_output(binding)?.1 != *expected {
                return Err(error("observed phase output digest drift"));
            }
            let destination = if state.as_ref().is_some_and(|state| state.output == *output_id) {
                target.join("state")
            } else {
                target.join("outputs").join(output_id)
            };
            copy_tree(&binding.path, &destination, &mut budget)?;
        }
    } else if let Some(state) = state {
        let reference = &step.inputs[&state.skipped_input];
        let binding = &bindings[reference];
        let hash = verify_output(binding)?.1;
        let ArtifactRef::Produced { action_id, output_id } = reference else {
            return Err(error("phase prior state lacks Produced authority"));
        };
        let index = action.inputs.iter().position(|input| matches!(input, ActionInput::Produced { action_id: producer, output_id: output } if producer == action_id && output == output_id))
            .ok_or_else(|| error("phase prior state is not a measured input"))?;
        if hash != measured_inputs[index] || known_outputs.get(reference).is_some_and(|(_, expected)| expected != &hash)
        {
            return Err(error("phase prior state differs from authenticated whole input"));
        }
        let selected = rooted_directory(&binding.path, &state.skipped_path)?;
        copy_tree(&selected, &target.join("state"), &mut budget)?;
    }
    Ok(())
}

/// Guard outcome, accepted exit codes, bound inputs, and their digests.
type GuardDecision = Option<(GuardOutcome, Vec<i32>, Vec<ActionInput>, Vec<String>)>;

fn guard_decision(
    bindings: &impl ArtifactBindings,
    step: &RecipeStep,
    outputs: &BTreeMap<ArtifactRef, (PathBuf, String)>,
) -> Result<GuardDecision, RunError> {
    let Some(guard) = operation_guard(&step.operation) else {
        return Ok(None);
    };
    let mut codes = Vec::with_capacity(guard.conditions.len());
    let mut inputs = Vec::with_capacity(guard.conditions.len());
    let mut hashes = Vec::with_capacity(guard.conditions.len());
    for condition in &guard.conditions {
        let reference = &step.inputs[&condition.status_input];
        let (path, authenticated) =
            outputs.get(reference).ok_or_else(|| error("guard status producer has not actually completed"))?;
        let binding = &bindings[reference];
        let (size, measured) = verify_output(binding)?;
        if size > 4 || &measured != authenticated {
            return Err(error("guard status changed after actual completion"));
        }
        let bytes = fs::read(path).map_err(|e| error(e.to_string()))?;
        let text = std::str::from_utf8(&bytes).map_err(|_| error("noncanonical actual exit status"))?;
        let code = text
            .strip_suffix('\n')
            .and_then(|text| text.parse::<i32>().ok())
            .ok_or_else(|| error("noncanonical actual exit status"))?;
        let (canonical, length) = canonical_exit_status(code).map_err(error)?;
        if bytes != canonical[..length] {
            return Err(error("noncanonical actual exit status"));
        }
        let ArtifactRef::Produced { action_id, output_id } = reference else {
            return Err(error("nonproduced guard status"));
        };
        codes.push(code);
        inputs.push(ActionInput::Produced {
            action_id: action_id.clone(),
            output_id: output_id.clone(),
        });
        hashes.push(measured);
        if condition.accepted_codes.binary_search(&code).is_err() {
            return Ok(Some((GuardOutcome::Skipped, codes, inputs, hashes)));
        }
    }
    Ok(Some((GuardOutcome::Active, codes, inputs, hashes)))
}

fn builtin(prepared: &PreparedRecipe, step: &RecipeStep, output_dir: &Path) -> Result<(), RunError> {
    let input = |name: &str| -> Result<&ArtifactBinding, RunError> {
        prepared.bindings.get(&step.inputs[name]).ok_or_else(|| error("unbound builtin input"))
    };
    match &step.operation {
        RecipeOperation::WriteText { output, text } => {
            let mut file = create(&output_dir.join(output))?;
            file.write_all(text.as_bytes()).and_then(|()| file.sync_all()).map_err(|e| error(e.to_string()))?;
        }
        RecipeOperation::CopyText { input: name, output } => {
            let binding = input(name)?;
            let (size, _) = digest(&binding.path)?;
            if size > step.outputs[output].bytes_max {
                return Err(error("copied source text exceeds bound"));
            }
            let text =
                fs::read_to_string(&binding.path).map_err(|e| error(format!("source copy is not UTF-8 text: {e}")))?;
            validate_text(&text)?;
            let mut file = create(&output_dir.join(output))?;
            file.write_all(text.as_bytes()).and_then(|()| file.sync_all()).map_err(|e| error(e.to_string()))?;
        }
        RecipeOperation::ExtractArchive {
            input: name,
            output,
            format,
        } => {
            let file = File::open(&input(name)?.path).map_err(|e| error(e.to_string()))?;
            // Shared XZ/LZMA decoding currently materializes a Vec; archive
            // member/output limits below do not claim bounded decoder memory.
            let decoder = crunch_build::fetcher::decompress_reader(format.suffix(), file)
                .map_err(|e| error(format!("source archive decompression: {e}")))?;
            let mut archive = tar::Archive::new(decoder);
            let target = output_dir.join(output);
            fs::create_dir(&target).map_err(|e| error(e.to_string()))?;
            let mut count = 0usize;
            let mut bytes = 0u64;
            for entry in archive.entries().map_err(|e| error(e.to_string()))? {
                let mut entry = entry.map_err(|e| error(e.to_string()))?;
                count += 1;
                if count > 1_000_000 {
                    return Err(error("source archive entry bound"));
                }
                bytes = bytes.checked_add(entry.size()).ok_or_else(|| error("source archive byte overflow"))?;
                if bytes > step.outputs[output].bytes_max {
                    return Err(error("source archive byte bound"));
                }
                let path = entry.path().map_err(|e| error(e.to_string()))?.into_owned();
                if path
                    .components()
                    .any(|p| !matches!(p, std::path::Component::Normal(_) | std::path::Component::CurDir))
                {
                    return Err(error("source archive path escape"));
                }
                if !entry.header().entry_type().is_file() && !entry.header().entry_type().is_dir() {
                    return Err(error("source archive links/special files require explicit source operations"));
                }
                if !entry.unpack_in(&target).map_err(|e| error(e.to_string()))? {
                    return Err(error("source archive unpack escape"));
                }
            }
        }
        RecipeOperation::MaterializeRecord {
            role,
            record_identity,
            output,
        } => {
            let path = prepared.sources.get(role).ok_or_else(|| error("unbound source manifest"))?;
            let manifest = crate::source_bundle::read_source_bundle(path)?;
            let mut records = manifest.records.iter().filter(|r| r.identity == *record_identity);
            let record = records.next().ok_or_else(|| error("unknown materialization source record"))?;
            if records.next().is_some() {
                return Err(error("duplicate materialization record"));
            }
            crate::source_bundle::materialize_source_record_payload(record, &output_dir.join(output))?;
        }
        RecipeOperation::MaterializeFetcher {
            role,
            record_identity,
            output,
        } => {
            let record = authenticated_fetcher_record(&prepared.proof, &prepared.sources, *role, record_identity)?;
            if step.outputs[output].kind != fetcher_output_kind(&record)? {
                return Err(error("fetcher source output kind changed after planning"));
            }
            // The existing decoder may retain an archive sibling while
            // checking its recursive pin. Keep that scratch out of published
            // outputs. Normal output verification applies the declared bound
            // and artifact kind before any result is promoted as evidence.
            let scratch = tempfile::tempdir_in(output_dir).map_err(|e| error(e.to_string()))?;
            let target = scratch.path().join("source");
            crate::source_bundle::materialize_source_record_for_offline_use(&record, &target)?;
            fs::rename(target, output_dir.join(output)).map_err(|e| error(e.to_string()))?;
        }
        RecipeOperation::AssembleTree { output, entries } => {
            let target = output_dir.join(output);
            fs::create_dir(&target).map_err(|e| error(e.to_string()))?;
            let mut budget = step.outputs[output].bytes_max;
            for (relative, name) in entries {
                copy_tree(&input(name)?.path, &target.join(relative), &mut budget)?;
            }
        }
        RecipeOperation::ObservePhase { .. } => {
            return Err(error("phase observation requires authenticated runtime state"));
        }
        RecipeOperation::Exec { .. } => return Err(error("external command passed to source builtin")),
        RecipeOperation::Controller { .. } => return Err(error("controller requires the admitted bounded runtime")),
    }
    Ok(())
}
fn verify_output(binding: &ArtifactBinding) -> Result<(u64, String), RunError> {
    let metadata = fs::symlink_metadata(&binding.path)
        .map_err(|e| error(format!("missing actual output {}: {e}", binding.path.display())))?;
    if (binding.kind == ArtifactKind::Tree) != metadata.is_dir() || (!metadata.is_dir() && !metadata.is_file()) {
        return Err(error("output kind mismatch or output symlink"));
    }
    let (size, hash) = digest(&binding.path)?;
    if size > binding.bytes_max {
        return Err(error("actual output byte bound exceeded"));
    }
    if binding.kind == ArtifactKind::Text {
        let text = fs::read_to_string(&binding.path).map_err(|e| error(format!("text output is binary: {e}")))?;
        validate_text(&text)?;
    }
    if binding.kind == ArtifactKind::Executable {
        let mut file = File::open(&binding.path).map_err(|e| error(e.to_string()))?;
        let mut magic = [0u8; 4];
        file.read_exact(&mut magic).map_err(|e| error(e.to_string()))?;
        if magic != *b"\x7fELF" {
            return Err(error("produced executable must be an actual ELF image, not an opaque script"));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if metadata.permissions().mode() & 0o111 == 0 {
                return Err(error("produced executable lacks execute permission"));
            }
        }
    }
    Ok((size, hash))
}
struct RecipeDeadlines {
    global: Instant,
    spans: Vec<(usize, usize, u64)>,
    next: usize,
    active: Vec<(usize, Instant)>,
}
impl RecipeDeadlines {
    fn new(recipe: &SourceRecipe, seconds: u64, start: Instant) -> Result<Self, RunError> {
        Ok(Self {
            global: start.checked_add(Duration::from_secs(seconds)).ok_or_else(|| error("recipe deadline overflow"))?,
            spans: timeout_spans(recipe)?,
            next: 0,
            active: Vec::new(),
        })
    }
    fn begin(&mut self, index: usize, now: Instant) -> Result<(), RunError> {
        while self.active.last().is_some_and(|(last, _)| *last < index) {
            self.active.pop();
        }
        while let Some(&(first, last, seconds)) = self.spans.get(self.next) {
            if first != index {
                break;
            }
            let own = now
                .checked_add(Duration::from_secs(seconds))
                .ok_or_else(|| error("source group deadline overflow"))?;
            self.active.push((last, own.min(self.deadline())));
            self.next += 1;
        }
        self.check(now)
    }
    fn deadline(&self) -> Instant {
        self.active.last().map_or(self.global, |(_, deadline)| *deadline)
    }
    fn check(&self, now: Instant) -> Result<(), RunError> {
        if now >= self.deadline() {
            return Err(error("actual recipe/source timeout deadline exceeded"));
        }
        Ok(())
    }
}

#[path = "source_built_recipe_exec_controller.rs"]
mod controller_runtime;
#[path = "source_built_recipe_exec_scopes.rs"]
mod scopes;
use scopes::SourceScopes;

struct RecipeExecution<'a> {
    prepared: &'a PreparedRecipe,
    raw: File,
    outputs: BTreeMap<ArtifactRef, (PathBuf, String)>,
    completions: Vec<RecipeCompletion>,
    phase_outcomes: BTreeMap<String, PhaseOutcome>,
    dynamic_bindings: BTreeMap<ArtifactRef, ArtifactBinding>,
    controller_outcomes: BTreeMap<String, crate::source_built_fixed_point_action_trust::ActionControllerOutcome>,
    source_scopes: SourceScopes,
    deadlines: RecipeDeadlines,
    total_disk: u64,
}
impl RecipeExecution<'_> {
    fn step(
        &mut self,
        step: &RecipeStep,
        actions: &BTreeMap<&str, &ActionSpec>,
        emit: &mut dyn FnMut(ActionExecutionEvent) -> Result<(), RunError>,
    ) -> Result<(), RunError> {
        let prepared = self.prepared;
        let raw = &mut self.raw;
        let outputs = &mut self.outputs;
        let completions = &mut self.completions;
        let phase_outcomes = &mut self.phase_outcomes;
        let deadlines = &self.deadlines;
        let total_disk = &mut self.total_disk;
        let dynamic_bindings = &self.dynamic_bindings;
        let bindings = BindingOverlay {
            base: &prepared.bindings,
            dynamic: Some(dynamic_bindings),
        };
        let controller_outcomes = &self.controller_outcomes;
        let source_scopes = &mut self.source_scopes;
        source_scopes.check()?;
        let action_id = format!("{}/{}", prepared.recipe.stage_id, step.id);
        let action = actions.get(action_id.as_str()).copied().ok_or_else(|| error("missing lowered action"))?;
        let step_start = Instant::now();
        let decision = match guard_decision(&bindings, step, outputs) {
            Ok(decision) => decision,
            Err(failure) => {
                journal(raw, &serde_json::json!({"action":action_id,"failure":failure.to_string()}))?;
                return Err(failure);
            }
        };
        if let Some((outcome, exit_codes, guard_inputs, hashes)) = decision {
            for planned in [
                actions.get(format!("{action_id}.sandbox").as_str()).copied(),
                Some(action),
            ]
            .into_iter()
            .flatten()
            {
                let mut decision = event(
                    planned,
                    ExecutionEventKind::GuardDecision {
                        outcome,
                        exit_codes: exit_codes.clone(),
                    },
                    String::new(),
                    hashes.clone(),
                    Vec::new(),
                    false,
                    false,
                    step_start.elapsed().as_secs(),
                    0,
                );
                decision.executable_producer = None;
                decision.inputs = guard_inputs.clone();
                journal(raw, &decision)?;
                emit(decision)?;
            }
            if outcome == GuardOutcome::Skipped {
                phase_outcomes.insert(action_id.clone(), PhaseOutcome::Skipped {
                    guard_codes: exit_codes,
                });
                deadlines.check(Instant::now())?;
                return Ok(());
            }
        }
        let output_dir = prepared.runtime_root.join(&step.id).join("outputs");
        fs::create_dir_all(&output_dir).map_err(|e| error(e.to_string()))?;
        let measure_inputs = |planned: &ActionSpec| -> Result<Vec<String>, RunError> {
            planned
                .inputs
                .iter()
                .map(|input| input_digest(prepared, &bindings, input, Some(phase_outcomes), Some(controller_outcomes)))
                .collect()
        };
        let inputs = match measure_inputs(action) {
            Ok(d) => d,
            Err(e) => {
                journal(raw, &serde_json::json!({"action":action_id,"failure":e.to_string()}))?;
                return Err(e);
            }
        };
        let executable_digest = if matches!(step.operation, RecipeOperation::Exec { .. }) {
            match &action.executable {
                ActionExecutable::Fixed { digest_blake3 } => digest_blake3.clone(),
                ActionExecutable::Produced { action_id, output_id } => {
                    digest(
                        &bindings[&ArtifactRef::Produced {
                            action_id: action_id.clone(),
                            output_id: output_id.clone(),
                        }]
                            .path,
                    )?
                    .1
                }
            }
        } else {
            digest(&std::env::current_exe().map_err(|e| error(e.to_string()))?)?.1
        };
        // Guard decisions precede all filesystem output creation and executable reads.
        let mut actual_exit = None;
        let mut observed_violation = false;
        deadlines.check(Instant::now())?;
        let result = if let RecipeOperation::Exec { exit_policy, .. } = &step.operation {
            let sandbox = actions
                .get(format!("{action_id}.sandbox").as_str())
                .copied()
                .ok_or_else(|| error("missing sandbox action"))?;
            let sandbox_inputs = measure_inputs(sandbox)?;
            let mut index = 0usize;
            let mut operands = CommandOperands {
                prepared,
                dynamic_bindings: Some(dynamic_bindings),
                step,
                action,
                measured_inputs: &inputs,
                known_outputs: outputs,
                texts: BTreeMap::new(),
            };
            let invocation = match operands.invocation() {
                Ok(invocation) => invocation,
                Err(failure) => {
                    journal(raw, &serde_json::json!({"action":action_id,"failure":failure.to_string()}))?;
                    return Err(failure);
                }
            };
            if let Some(request) = &invocation {
                source_scopes.update(request.process_attributes.as_ref())?;
            }
            let launch = match launch(
                prepared,
                step,
                &output_dir,
                invocation.as_ref(),
                &mut operands,
                source_scopes.deadline(deadlines.deadline()),
                &mut |observation| {
                    journal(raw, observation)?;
                    if !observation.is_exec && observation.allowed {
                        return Ok(());
                    }
                    let planned = if index == 0 { sandbox } else { action };
                    let expected = if index == 0 {
                        &prepared.sandbox_digest
                    } else {
                        &executable_digest
                    };
                    let violation = !observation.allowed
                        || index > 1
                        || !observation.is_exec
                        || &observation.digest_blake3 != expected;
                    observed_violation |= violation;
                    let result = emit(event(
                        planned,
                        ExecutionEventKind::ProtectedExec,
                        observation.digest_blake3.clone(),
                        if index == 0 {
                            sandbox_inputs.clone()
                        } else {
                            inputs.clone()
                        },
                        Vec::new(),
                        observation.allowed,
                        violation,
                        step_start.elapsed().as_secs(),
                        0,
                    ));
                    if observation.is_exec {
                        index += 1;
                    }
                    result
                },
            ) {
                Ok(launch) => launch,
                Err(failure) => {
                    journal(raw, &serde_json::json!({"action":action_id,"failure":failure.to_string()}))?;
                    return Err(failure);
                }
            };
            actual_exit = launch.status;
            observed_violation |= launch.violation;
            let normal_completion = launch.failure.is_none() && launch.exec_count == 2 && !launch.violation;
            let status_result = if let Some(code) = launch.status.filter(|_| normal_completion) {
                journal(raw, &serde_json::json!({"action":action_id,"normal_exit_code":code}))?;
                if step.outputs.contains_key("exit-status") {
                    // Reserved producer fact: a leaf-written file cannot replace
                    // the executor's observed status (create_new rejects it).
                    create(&output_dir.join("exit-status")).and_then(|mut file| {
                        writeln!(file, "{code}").and_then(|()| file.sync_all()).map_err(|e| error(e.to_string()))
                    })
                } else {
                    Ok(())
                }
            } else {
                Ok(())
            };
            let success = launch.status.is_some_and(|code| exit_policy.accepts(code))
                && normal_completion
                && status_result.is_ok();
            let sandbox_outputs = vec![
                (sandbox.output_ids[0].clone(), digest(&output_dir.join("stderr"))?.1),
                (sandbox.output_ids[1].clone(), digest(&output_dir.join("stdout"))?.1),
            ];
            emit(event(
                sandbox,
                ExecutionEventKind::BuildExecution,
                prepared.sandbox_digest.clone(),
                sandbox_inputs,
                sandbox_outputs,
                success,
                launch.violation,
                step_start.elapsed().as_secs(),
                digest(&output_dir)?.0,
            ))?;
            status_result.and_then(|()| {
                if success {
                    Ok(())
                } else {
                    Err(error(format!("protected leaf failed: exit {:?}, failure {:?}", launch.status, launch.failure)))
                }
            })
        } else {
            let expected = match &action.executable {
                ActionExecutable::Fixed { digest_blake3 } => digest_blake3.clone(),
                ActionExecutable::Produced { action_id, output_id } => {
                    digest(
                        &bindings[&ArtifactRef::Produced {
                            action_id: action_id.clone(),
                            output_id: output_id.clone(),
                        }]
                            .path,
                    )?
                    .1
                }
            };
            if executable_digest != expected {
                observed_violation = true;
                Err(error("source operation executable differs from its measured producer authority"))
            } else {
                deadlines.check(Instant::now())?;
                if matches!(step.operation, RecipeOperation::ObservePhase { .. }) {
                    observe_phase_bindings(
                        prepared,
                        &bindings,
                        step,
                        &output_dir,
                        phase_outcomes,
                        action,
                        &inputs,
                        outputs,
                    )
                } else {
                    builtin(prepared, step, &output_dir)
                }
            }
        };
        let result = result.and_then(|()| {
            deadlines.check(Instant::now())?;
            for entry in fs::read_dir(&output_dir).map_err(|e| error(e.to_string()))? {
                let name = entry.map_err(|e| error(e.to_string()))?.file_name();
                if !name.to_str().is_some_and(|name| step.outputs.contains_key(name)) {
                    return Err(error("leaf created an undeclared output outside its published artifact set"));
                }
            }
            Ok(())
        });
        if let Err(failure) = result {
            journal(raw, &serde_json::json!({"action":action_id,"failure":failure.to_string()}))?;
            emit(event(
                action,
                ExecutionEventKind::BuildExecution,
                executable_digest,
                inputs,
                Vec::new(),
                false,
                observed_violation,
                step_start.elapsed().as_secs(),
                digest(&output_dir)?.0,
            ))?;
            return Err(failure);
        }
        let mut measured = Vec::new();
        let mut step_disk = 0u64;
        for output_id in step.outputs.keys() {
            let reference = ArtifactRef::Produced {
                action_id: action_id.clone(),
                output_id: output_id.clone(),
            };
            let binding = &bindings[&reference];
            let optional_failure = actual_exit.is_some_and(|code| code != 0)
                && matches!(&step.operation, RecipeOperation::Exec {
                    exit_policy: RecipeExitPolicy::OneOf { .. } | RecipeExitPolicy::Normal,
                    ..
                })
                && step.outputs.contains_key("exit-status")
                && !["stdout", "stderr", "exit-status"].contains(&output_id.as_str());
            if optional_failure
                && fs::symlink_metadata(&binding.path).is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound)
            {
                continue;
            }
            let (size, hash) = match verify_output(binding) {
                Ok(o) => o,
                Err(failure) => {
                    journal(
                        raw,
                        &serde_json::json!({"action":action_id,"output":output_id,"failure":failure.to_string()}),
                    )?;
                    emit(event(
                        action,
                        ExecutionEventKind::BuildExecution,
                        executable_digest,
                        inputs,
                        Vec::new(),
                        false,
                        false,
                        step_start.elapsed().as_secs(),
                        step_disk,
                    ))?;
                    return Err(failure);
                }
            };
            step_disk = step_disk.checked_add(size).ok_or_else(|| error("disk accounting overflow"))?;
            measured.push((output_id.clone(), hash.clone()));
            outputs.insert(reference, (binding.path.clone(), hash));
        }
        *total_disk = total_disk.checked_add(step_disk).ok_or_else(|| error("disk accounting overflow"))?;
        let bounded =
            *total_disk <= prepared.proof.resource_bounds.disk_bytes_max && deadlines.check(Instant::now()).is_ok();
        journal(
            raw,
            &serde_json::json!({"action":action_id,"actual_outputs":measured,"disk_bytes":step_disk,"completed":bounded}),
        )?;
        let phase_outcome = PhaseOutcome::Executed {
            output_digests: measured.iter().cloned().collect(),
        };
        let mut completion_event = event(
            action,
            ExecutionEventKind::BuildExecution,
            executable_digest,
            inputs,
            measured,
            bounded,
            false,
            step_start.elapsed().as_secs(),
            step_disk,
        );
        completion_event.normal_exit_code = actual_exit;
        emit(completion_event)?;
        phase_outcomes.insert(action_id.clone(), phase_outcome);
        if matches!(step.operation, RecipeOperation::Exec { .. }) {
            completions.push(RecipeCompletion {
                action_id: action_id.clone(),
                exit_code: actual_exit.ok_or_else(|| error("missing actual command completion"))?,
                stdout_path: output_dir.join("stdout"),
                stdout_digest_blake3: digest(&output_dir.join("stdout"))?.1,
                stderr_path: output_dir.join("stderr"),
                stderr_digest_blake3: digest(&output_dir.join("stderr"))?.1,
            });
        }
        if !bounded {
            return Err(error("actual recipe resource bound exceeded"));
        }
        deadlines.check(Instant::now())?;
        source_scopes.check()?;
        Ok(())
    }
}

/// Execute only the previously admitted immutable source program. Failed raw
/// records are fsynced before an error is returned and output roots are retained.
pub(crate) fn run(
    prepared: &PreparedRecipe,
    emit: &mut dyn FnMut(ActionExecutionEvent) -> Result<(), RunError>,
) -> Result<RecipeRun, RunError> {
    fs::create_dir(&prepared.runtime_root).map_err(|e| error(format!("fresh cache-free recipe root: {e}")))?;
    let mut raw = create(&prepared.runtime_root.join("execution.jsonl"))?;
    journal(
        &mut raw,
        &serde_json::json!({"schema":"mantle-source-recipe-execution-v1","proof":prepared.proof.plan_digest_blake3,"stage":prepared.recipe.stage_id}),
    )?;
    let mut execution = RecipeExecution {
        prepared,
        raw,
        outputs: BTreeMap::new(),
        completions: Vec::new(),
        phase_outcomes: BTreeMap::new(),
        dynamic_bindings: BTreeMap::new(),
        controller_outcomes: BTreeMap::new(),
        source_scopes: SourceScopes::default(),
        deadlines: RecipeDeadlines::new(
            &prepared.recipe,
            prepared.proof.resource_bounds.elapsed_seconds_max,
            Instant::now(),
        )?,
        total_disk: 0,
    };
    let actions = prepared
        .adapter
        .actions
        .iter()
        .map(|action| (action.id.as_str(), action))
        .collect::<BTreeMap<_, _>>();
    for (step_index, step) in prepared.recipe.steps.iter().enumerate() {
        execution.deadlines.begin(step_index, Instant::now())?;
        if let RecipeOperation::Controller { specification, .. } = &step.operation {
            controller_runtime::run(&mut execution, step, specification, emit)?;
        } else {
            execution.step(step, &actions, emit)?;
        }
    }
    journal(&mut execution.raw, &serde_json::json!({"complete":true,"output_count":execution.outputs.len()}))?;
    Ok(RecipeRun {
        outputs: execution.outputs,
        completions: execution.completions,
        phase_outcomes: execution.phase_outcomes,
    })
}
fn launch(
    prepared: &PreparedRecipe,
    step: &RecipeStep,
    output_dir: &Path,
    invocation: Option<&RecipeInvocation>,
    operands: &mut CommandOperands<'_>,
    deadline: Instant,
    observe: &mut dyn FnMut(&RawExec) -> Result<(), RunError>,
) -> Result<LaunchResult, RunError> {
    let RecipeOperation::Exec {
        executable_input,
        arguments,
        environment,
        runtime_mounts,
        cwd_input,
        cwd_output,
        stdin_input,
        merge_stderr,
        command_state,
        exit_policy,
        ..
    } = &step.operation
    else {
        return Err(error("non-command launch"));
    };
    let (arguments, environment) = invocation
        .map(|request| (&request.arguments, &request.environment))
        .unwrap_or((arguments, environment));
    let executable = operands.binding(&step.inputs[executable_input]);
    let expected_image = if invocation.is_some_and(|request| request.executable_path.is_some()) {
        operands.authenticate_produced(executable_input, ArtifactKind::Executable, "fixed installed executable")?
    } else {
        digest(&executable.path)?.1
    };
    let image_word = invocation.and_then(|request| request.executable_path.as_ref());
    let image_root = match image_word.map(|word| &word.parts[0]) {
        Some(WordPart::Input { name, path }) => Some((name, path)),
        Some(WordPart::Runtime { mount, path }) => Some((&runtime_mounts[mount], path)),
        None => None,
        _ => return Err(error("installed image reached launch without readonly Tree authority")),
    };
    let image_target = if let Some(word) = image_word {
        operands.word(word)?
    } else {
        format!("/inputs/{executable_input}")
    };
    if image_root.is_some()
        && (image_target.as_str() == prepared.sandbox_binary.to_string_lossy().as_ref()
            || ["/outputs", "/work", "/proc", "/dev", "/tmp"].iter().any(|reserved| {
                image_target == *reserved
                    || image_target.strip_prefix(*reserved).is_some_and(|tail| tail.starts_with('/'))
            }))
    {
        return Err(error("installed image path collides with a protected execution namespace"));
    }
    if digest(&prepared.sandbox_binary)?.1 != prepared.sandbox_digest {
        return Err(error("reviewed sandbox binary drift"));
    }
    let RecipeOperation::Exec { output_seeds, .. } = &step.operation else {
        unreachable!()
    };
    for (name, output) in &step.outputs {
        if let Some(input) = output_seeds.get(name) {
            let source = operands.binding(&step.inputs[input]);
            verify_output(source)?;
            let mut budget = output.bytes_max;
            copy_tree(&source.path, &output_dir.join(name), &mut budget)?;
        } else if output.kind == ArtifactKind::Tree {
            fs::create_dir(output_dir.join(name)).map_err(|e| error(format!("fresh command output tree: {e}")))?;
        }
    }
    if let Some(config) = command_state {
        command_state::prepare(config, output_dir, operands)?;
    }
    let stdout = create(&output_dir.join("stdout"))?;
    let stderr = create(&output_dir.join("stderr"))?;
    let stderr_capture = if *merge_stderr {
        stdout.try_clone().map_err(|e| error(e.to_string()))?
    } else {
        stderr
    };
    let mut command = Command::new(&prepared.sandbox_binary);
    command.env_clear();
    command.args([
        "--unshare-all",
        "--die-with-parent",
        "--new-session",
        "--clearenv",
        "--tmpfs",
        "/",
        "--dir",
        "/inputs",
        "--dir",
        "/outputs",
        "--dir",
        "/work",
        "--proc",
        "/proc",
        "--dev",
        "/dev",
        "--tmpfs",
        "/tmp",
    ]);
    for (name, reference) in &step.inputs {
        if is_guard_input(step, name) {
            continue;
        }
        let input = operands.binding(reference);
        if image_root.is_some_and(|(root, _)| root == name)
            || invocation.is_some_and(|request| request.readonly_input_roots.binary_search(name).is_ok())
        {
            operands.authenticate_produced(name, ArtifactKind::Tree, "readonly input Tree")?;
        } else if digest(&input.path)?.0 > input.bytes_max {
            return Err(error("bound command input oversized"));
        }
        command.arg("--ro-bind").arg(&input.path).arg(format!("/inputs/{name}"));
    }
    for (destination, name) in runtime_mounts {
        let input = operands.binding(&step.inputs[name]);
        command.arg("--ro-bind").arg(&input.path).arg(destination);
    }
    if let Some((name, member)) = image_root {
        installed_image_member(&operands.binding(&step.inputs[name]).path, member, &expected_image)?;
        // Bind only the already fixed individually Produced image at the same
        // authenticated installed path; no directory-prefix exec grant exists.
        command.arg("--ro-bind").arg(&executable.path).arg(&image_target);
    }
    command.arg("--bind").arg(output_dir).arg("/outputs");
    if let Some(request) = invocation {
        for (destination, mapping) in &request.workspace_mounts {
            let root = output_dir.join(&mapping.output);
            let selected = rooted_directory(&root, &mapping.path)?;
            command.arg("--bind").arg(selected).arg(destination);
        }
    }
    for (key, value) in environment {
        command.arg("--setenv").arg(key).arg(operands.word(value)?);
    }
    let cwd = if let Some(word) = invocation.and_then(|request| request.cwd.as_ref()) {
        let selected = match &word.parts[0] {
            WordPart::Input { name, path } => rooted_directory(&operands.binding(&step.inputs[name]).path, path)?,
            WordPart::Output { name, path } => rooted_directory(&output_dir.join(name), path)?,
            WordPart::Runtime { mount, path } => {
                rooted_directory(&operands.binding(&step.inputs[&runtime_mounts[mount]]).path, path)?
            }
            WordPart::Workspace { mount, path } => {
                let mapping = &invocation.unwrap().workspace_mounts[mount];
                let root = rooted_directory(&output_dir.join(&mapping.output), &mapping.path)?;
                rooted_directory(&root, path)?
            }
            _ => return Err(error("untyped invocation cwd reached launch")),
        };
        // Validate the real rooted member without replacing its authenticated
        // logical namespace spelling (including original source absolute paths).
        drop(selected);
        operands.word(word)?
    } else if let Some(name) = cwd_output {
        format!("/outputs/{name}")
    } else if let Some(name) = cwd_input {
        format!("/inputs/{name}")
    } else {
        "/work".into()
    };
    command.arg("--chdir").arg(cwd);
    if let Some(argv0) = invocation.and_then(|request| request.argv0.as_ref()) {
        command.arg("--argv0").arg(operands.word(argv0)?);
    }
    command.arg("--").arg(&image_target);
    for argument in arguments {
        command.arg(operands.word(argument)?);
    }
    let input = if let Some(name) = stdin_input {
        File::open(&operands.binding(&step.inputs[name]).path).map_err(|failure| error(failure.to_string()))?
    } else {
        File::open("/dev/null").map_err(|failure| error(failure.to_string()))?
    };
    let descriptors = if let Some(request) = invocation.filter(|request| request.descriptors.is_some()) {
        Some(command_state::Descriptors::prepare(request, output_dir, operands)?)
    } else {
        command_state
            .as_ref()
            .map(|_| command_state::Descriptors::retain(&input, &stdout, &stderr_capture))
            .transpose()?
    };
    let injected_descriptors = if invocation.is_some_and(|request| request.descriptors.is_some()) {
        descriptors.clone()
    } else {
        None
    };
    command.stdin(input).stdout(stdout).stderr(stderr_capture);
    let expected = vec![
        (prepared.sandbox_binary.clone(), prepared.sandbox_digest.clone()),
        (PathBuf::from(image_target), expected_image),
    ];
    let timeout = deadline
        .checked_duration_since(Instant::now())
        .filter(|remaining| !remaining.is_zero())
        .ok_or_else(|| error("source deadline exceeded before protected launch"))?;
    let file_limit = step.outputs.values().map(|o| o.bytes_max).max().unwrap_or(1);
    let file_limit = invocation
        .and_then(|request| request.process_attributes.as_ref())
        .and_then(|attributes| attributes.file_size_bytes_max)
        .map_or(file_limit, |source_limit| file_limit.min(source_limit));
    #[cfg(target_os = "linux")]
    if let Some(mask) = invocation
        .and_then(|request| request.process_attributes.as_ref())
        .and_then(|attributes| attributes.umask)
    {
        use std::os::unix::process::CommandExt;
        // Command runs this only in its forked child; never mutate the parent mask.
        unsafe {
            command.pre_exec(move || {
                libc::umask(libc::mode_t::from(mask));
                Ok(())
            });
        }
    }
    let (send, receive) = std::sync::mpsc::sync_channel::<PendingObservation>(1);
    let worker = std::thread::spawn(move || {
        let remaining = deadline.saturating_duration_since(Instant::now()).min(timeout);
        protected_launch(command, expected, remaining, file_limit, send, injected_descriptors)
    });
    let mut capture_failure = None;
    loop {
        match receive.recv_timeout(Duration::from_millis(20)) {
            Ok(pending) => {
                let admitted = match observe(&pending.row) {
                    Ok(()) => capture_failure.is_none(),
                    Err(error) => {
                        if capture_failure.is_none() {
                            capture_failure = Some(error);
                        }
                        false
                    }
                };
                let _ = pending.decision.send(admitted);
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) if !worker.is_finished() => continue,
            Err(_) => break,
        }
    }
    let result = worker.join().map_err(|_| error("protected leaf worker panicked"))?;
    if let Some(error) = capture_failure {
        return Err(error);
    }
    let result = result?;
    if let Some(config) = command_state
        && result.failure.is_none()
        && result.exec_count == 2
        && !result.violation
        && result.status.is_some_and(|code| exit_policy.accepts(code))
    {
        command_state::capture(
            config,
            step,
            output_dir,
            descriptors.as_ref().ok_or_else(|| error("missing retained command descriptors"))?,
        )?;
    }
    Ok(result)
}
#[cfg(not(target_os = "linux"))]
fn protected_launch(
    _command: Command,
    _expected: Vec<(PathBuf, String)>,
    _timeout: Duration,
    _file_limit: u64,
    _observations: std::sync::mpsc::SyncSender<PendingObservation>,
    _descriptors: Option<command_state::Descriptors>,
) -> Result<LaunchResult, RunError> {
    Err(error("source proof protected replay requires Linux"))
}
#[cfg(target_os = "linux")]
fn protected_launch(
    mut command: Command,
    expected: Vec<(PathBuf, String)>,
    timeout: Duration,
    file_limit: u64,
    observations: std::sync::mpsc::SyncSender<PendingObservation>,
    descriptors: Option<command_state::Descriptors>,
) -> Result<LaunchResult, RunError> {
    use std::os::unix::process::CommandExt;
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;
    use std::sync::atomic::Ordering;
    let continue_flags = u32::try_from(libc::SECCOMP_USER_NOTIF_FLAG_CONTINUE)
        .map_err(|_| error("seccomp continue flag does not fit response flags"))?;
    use std::os::fd::AsRawFd;
    let deadline = Instant::now().checked_add(timeout).ok_or_else(|| error("protected deadline overflow"))?;
    if timeout.is_zero() {
        return Err(error("protected deadline already expired"));
    }
    // RLIMIT_CPU is whole-second CPU accounting only. The monotonic wall
    // deadline retains the exact remaining Duration, including subsecond caps.
    let cpu_seconds = timeout.as_secs().saturating_add(u64::from(timeout.subsec_nanos() != 0));
    // libc fork holds allocator locks until clone returns. Intercepting that
    // clone would deadlock an observer that allocates its notification record.
    // Prepare the filter here, then install it in the child before its exec.
    let instructions = filter_instructions()?;
    // NOTIF_RECV can sleep after poll readiness is cancelled. It must never run
    // in a thread we join: a syscall-only child owns that interruptible sleep.
    let pump = NotificationPump::start()?;
    let socket = Arc::clone(&pump.socket);
    let stop = Arc::new(AtomicBool::new(false));
    let stopping = Arc::clone(&stop);
    let observer = std::thread::spawn(move || -> Result<(usize, bool), String> {
        let mut pump = pump;
        let result = (|| {
            let mut next = 0usize;
            let mut violation = false;
            let mut count = 0u32;
            while !stopping.load(Ordering::Acquire) {
                if Instant::now() >= deadline {
                    return Err("protected observation deadline exceeded".into());
                }
                let mut notification: libc::seccomp_notif = unsafe { std::mem::zeroed() };
                if !pump.receive(&mut notification)? {
                    continue;
                }
                let is_exec = notification.data.nr as i64 == libc::SYS_execve
                    || notification.data.nr as i64 == libc::SYS_execveat;
                let mut row = if is_exec {
                    observe_exec(&notification, expected.get(next).map(|(path, _)| path.as_path()))
                } else {
                    RawExec {
                        pid: notification.pid,
                        syscall: notification.data.nr,
                        tracee_path: PathBuf::new(),
                        resolved_path: PathBuf::new(),
                        digest_blake3: String::new(),
                        allowed: false,
                        reason: String::new(),
                        is_exec: false,
                    }
                };
                row.allowed = if is_exec {
                    expected
                        .get(next)
                        .is_some_and(|(path, hash)| row.tracee_path == *path && row.digest_blake3 == *hash)
                } else {
                    next < 2 || thread_clone(&notification)
                };
                row.reason = if row.allowed {
                    "exact source-planned transition"
                } else {
                    "unknown execution or unplanned child process"
                }
                .into();
                count = count.saturating_add(1);
                if count > 262_144 {
                    row.allowed = false;
                    row.reason = "protected notification bound exceeded".into();
                }
                let (reply, decision) = std::sync::mpsc::sync_channel(1);
                let mut pending = PendingObservation {
                    row: row.clone(),
                    decision: reply,
                };
                loop {
                    match observations.try_send(pending) {
                        Ok(()) => break,
                        Err(std::sync::mpsc::TrySendError::Full(value))
                            if !stopping.load(Ordering::Acquire) && Instant::now() < deadline =>
                        {
                            pending = value;
                            std::thread::sleep(Duration::from_millis(2));
                        }
                        Err(_) => return Err("protected observation capture unavailable".into()),
                    }
                }
                loop {
                    match decision.recv_timeout(Duration::from_millis(20)) {
                        Ok(true) => break,
                        Ok(false) | Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                            row.allowed = false;
                            break;
                        }
                        Err(std::sync::mpsc::RecvTimeoutError::Timeout)
                            if !stopping.load(Ordering::Acquire) && Instant::now() < deadline => {}
                        Err(_) => return Err("protected observation decision deadline exceeded".into()),
                    }
                }
                violation |= !row.allowed;
                if is_exec && row.allowed {
                    next += 1;
                }
                let response = libc::seccomp_notif_resp {
                    id: notification.id,
                    val: 0,
                    error: if row.allowed { 0 } else { -libc::EPERM },
                    flags: if row.allowed { continue_flags } else { 0 },
                };
                // The pump owns the sole listener, including when the tracee is
                // blocked in its first exec before CLOEXEC can take effect.
                let injections = if is_exec && row.allowed && next == 2 {
                    descriptors.as_ref().map(command_state::Descriptors::injections)
                } else {
                    None
                };
                pump.respond(&response, deadline, injections.as_ref())?;
            }
            Ok((next, violation))
        })();
        pump.finish()?;
        result
    });
    // Only allocation-free resource-limit and filter syscalls run after fork.
    unsafe {
        command.pre_exec(move || {
            let file = libc::rlimit {
                rlim_cur: file_limit,
                rlim_max: file_limit,
            };
            if libc::setrlimit(libc::RLIMIT_FSIZE, &file) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            let cpu = libc::rlimit {
                rlim_cur: cpu_seconds,
                rlim_max: cpu_seconds,
            };
            if libc::setrlimit(libc::RLIMIT_CPU, &cpu) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            let listener = install_filter(&instructions)?;
            let transfer = send_listener(socket.as_raw_fd(), listener);
            libc::close(listener);
            transfer?;
            Ok(())
        });
    }
    let mut failure = None;
    let mut status = None;
    let spawned = command.spawn();
    match spawned {
        Err(e) => failure = Some(format!("launch failed: {e}")),
        Ok(mut child) => loop {
            match child.try_wait() {
                Ok(Some(exit)) => {
                    status = exit.code();
                    break;
                }
                Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(10)),
                Ok(None) => {
                    let _ = child.kill();
                    failure = Some(match reap_killed_child(&mut child) {
                        Ok(()) => "wall-clock bound exceeded".into(),
                        Err(error) => error.to_string(),
                    });
                    break;
                }
                Err(e) => {
                    let _ = child.kill();
                    failure = Some(match reap_killed_child(&mut child) {
                        Ok(()) => format!("wait failed: {e}"),
                        Err(error) => error.to_string(),
                    });
                    break;
                }
            }
        },
    }
    stop.store(true, Ordering::Release);
    let observed = observer.join().map_err(|_| error("protected observer panicked"));
    let (exec_count, violation) = observed?.map_err(error)?;
    Ok(LaunchResult {
        status,
        exec_count,
        violation,
        failure,
    })
}
// Linux deliberately ignores O_NONBLOCK in NOTIF_RECV on older supported
// kernels. A killed tracee can invalidate poll readiness before that ioctl. The
// isolated pump below may block in it; SIGKILL targets only the owned pump
// process, including an already sleeping ioctl, never another Rust thread.
#[cfg(target_os = "linux")]
struct NotificationPump {
    pid: libc::pid_t,
    socket: std::sync::Arc<std::os::fd::OwnedFd>,
}

#[cfg(target_os = "linux")]
impl NotificationPump {
    fn start() -> Result<Self, RunError> {
        use std::os::fd::FromRawFd;
        use std::os::fd::OwnedFd;
        let mut sockets = [-1; 2];
        if unsafe {
            libc::socketpair(libc::AF_UNIX, libc::SOCK_SEQPACKET | libc::SOCK_CLOEXEC, 0, sockets.as_mut_ptr())
        } != 0
        {
            return Err(error(format!("creating protected notification socket: {}", std::io::Error::last_os_error())));
        }
        let parent = unsafe { OwnedFd::from_raw_fd(sockets[0]) };
        let child = unsafe { OwnedFd::from_raw_fd(sockets[1]) };
        let parent_pid = unsafe { libc::getpid() };
        // No atfork handlers and no shared address space. In the child, only
        // stack storage and direct kernel syscalls are used, then _exit: no
        // allocator, Rust destructors, stdio or inherited userspace locks.
        let pid = unsafe { libc::syscall(libc::SYS_clone, libc::SIGCHLD, 0, 0, 0, 0) };
        if pid == 0 {
            unsafe { notification_pump_child(sockets[0], sockets[1], parent_pid) }
        }
        if pid < 0 {
            return Err(error(format!("starting protected notification pump: {}", std::io::Error::last_os_error())));
        }
        drop(child);
        Ok(Self {
            pid: pid as libc::pid_t,
            socket: std::sync::Arc::new(parent),
        })
    }

    fn receive(&self, notification: &mut libc::seccomp_notif) -> Result<bool, String> {
        use std::os::fd::AsRawFd;
        let fd = self.socket.as_raw_fd();
        let mut poll = libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        };
        let ready = unsafe { libc::poll(&mut poll, 1, 20) };
        if ready < 0 {
            let failure = std::io::Error::last_os_error();
            return if failure.kind() == std::io::ErrorKind::Interrupted {
                Ok(false)
            } else {
                Err(failure.to_string())
            };
        }
        if ready == 0 {
            return Ok(false);
        }
        // Socket nonblocking receive is documented, unlike listener ioctl
        // nonblocking behavior. Cancellation between poll and recv is harmless.
        let received = unsafe {
            libc::recv(
                fd,
                (notification as *mut libc::seccomp_notif).cast(),
                std::mem::size_of::<libc::seccomp_notif>(),
                libc::MSG_DONTWAIT | libc::MSG_TRUNC,
            )
        };
        if received == std::mem::size_of::<libc::seccomp_notif>() as isize {
            return Ok(true);
        }
        if received < 0
            && matches!(std::io::Error::last_os_error().raw_os_error(), Some(libc::EINTR) | Some(libc::EAGAIN))
        {
            return Ok(false);
        }
        Err("protected notification pump closed or sent an invalid packet".into())
    }

    fn respond(
        &self,
        response: &libc::seccomp_notif_resp,
        deadline: Instant,
        injections: Option<&[command_state::DescriptorInjection; 64]>,
    ) -> Result<(), String> {
        use std::os::fd::AsRawFd;
        let fd = self.socket.as_raw_fd();
        descriptor_transport::send(fd, response, injections)?;
        loop {
            // Normal leaf exit may precede the pump's acknowledgment. Drain the
            // outstanding response; only the common deadline can end this wait.
            if Instant::now() >= deadline {
                return Err("protected response deadline exceeded".into());
            }
            let mut poll = libc::pollfd {
                fd,
                events: libc::POLLIN,
                revents: 0,
            };
            if unsafe { libc::poll(&mut poll, 1, 20) } < 0 {
                if std::io::Error::last_os_error().kind() == std::io::ErrorKind::Interrupted {
                    continue;
                }
                return Err("polling protected response failed".into());
            }
            let mut errno = 0i32;
            let received = unsafe {
                libc::recv(
                    fd,
                    (&mut errno as *mut i32).cast(),
                    std::mem::size_of::<i32>(),
                    libc::MSG_DONTWAIT | libc::MSG_TRUNC,
                )
            };
            if received < 0
                && matches!(std::io::Error::last_os_error().raw_os_error(), Some(libc::EINTR) | Some(libc::EAGAIN))
            {
                continue;
            }
            if received != std::mem::size_of::<i32>() as isize {
                return Err("protected response acknowledgment missing or invalid".into());
            }
            if errno != 0 && errno != libc::ENOENT {
                return Err(format!(
                    "responding to protected notification: {}",
                    std::io::Error::from_raw_os_error(errno)
                ));
            }
            return Ok(());
        }
    }

    fn finish(&mut self) -> Result<(), String> {
        if self.pid == 0 {
            return Ok(());
        }
        // NOTIF_RECV is interruptible. Kill its sole owning process rather
        // than closing a descriptor underneath an in-flight file operation.
        unsafe {
            libc::kill(self.pid, libc::SIGKILL);
        }
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let result = unsafe { libc::waitpid(self.pid, std::ptr::null_mut(), libc::WNOHANG) };
            if result == self.pid
                || (result < 0 && std::io::Error::last_os_error().raw_os_error() == Some(libc::ECHILD))
            {
                self.pid = 0;
                return Ok(());
            }
            if Instant::now() >= deadline {
                return Err("protected notification pump did not exit before teardown deadline".into());
            }
            std::thread::sleep(Duration::from_millis(2));
        }
    }
}

#[cfg(target_os = "linux")]
impl Drop for NotificationPump {
    fn drop(&mut self) {
        // Covers installation errors and unwinding as well as normal teardown.
        // Only this child is reaped; no process-global signal state is touched.
        let _ = self.finish();
    }
}

#[cfg(target_os = "linux")]
fn send_listener(socket: i32, listener: i32) -> std::io::Result<()> {
    let mut byte = 1u8;
    let mut iovec = libc::iovec {
        iov_base: (&mut byte as *mut u8).cast(),
        iov_len: 1,
    };
    // usize storage supplies cmsghdr alignment on both supported architectures.
    let mut control = [0usize; 8];
    let mut message: libc::msghdr = unsafe { std::mem::zeroed() };
    message.msg_iov = &mut iovec;
    message.msg_iovlen = 1;
    message.msg_control = control.as_mut_ptr().cast();
    message.msg_controllen = unsafe { libc::CMSG_SPACE(std::mem::size_of::<i32>() as u32) } as usize;
    unsafe {
        let header = libc::CMSG_FIRSTHDR(&message);
        (*header).cmsg_level = libc::SOL_SOCKET;
        (*header).cmsg_type = libc::SCM_RIGHTS;
        (*header).cmsg_len = libc::CMSG_LEN(std::mem::size_of::<i32>() as u32) as usize;
        std::ptr::write_unaligned(libc::CMSG_DATA(header).cast::<i32>(), listener);
    }
    if unsafe { libc::sendmsg(socket, &message, libc::MSG_DONTWAIT | libc::MSG_NOSIGNAL) } != 1 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

/// Post-fork child. Never return into Rust runtime state inherited from the
/// multithreaded parent. All calls here are direct syscalls or async-safe _exit.
#[cfg(target_os = "linux")]
unsafe fn notification_pump_child(parent: i32, socket: i32, parent_pid: libc::pid_t) -> ! {
    unsafe {
        libc::syscall(libc::SYS_close, parent);
        if libc::syscall(libc::SYS_prctl, libc::PR_SET_PDEATHSIG, libc::SIGKILL, 0, 0, 0) != 0
            || libc::syscall(libc::SYS_getppid) != parent_pid as libc::c_long
        {
            libc::_exit(120);
        }
        // Concurrent launches must not leak another pump's listener into this
        // process. Keep only this private control socket before receiving ours.
        if socket > 0 && libc::syscall(libc::SYS_close_range, 0u32, socket as u32 - 1, 0u32) != 0 {
            libc::_exit(127);
        }
        if libc::syscall(libc::SYS_close_range, socket as u32 + 1, u32::MAX, 0u32) != 0 {
            libc::_exit(127);
        }
        let mut byte = 0u8;
        let mut iovec = libc::iovec {
            iov_base: (&mut byte as *mut u8).cast(),
            iov_len: 1,
        };
        let mut control = [0usize; 8];
        let mut message: libc::msghdr = std::mem::zeroed();
        message.msg_iov = &mut iovec;
        message.msg_iovlen = 1;
        message.msg_control = control.as_mut_ptr().cast();
        message.msg_controllen = std::mem::size_of_val(&control);
        if libc::syscall(libc::SYS_recvmsg, socket, &mut message, libc::MSG_CMSG_CLOEXEC) != 1 {
            libc::_exit(121);
        }
        let header = libc::CMSG_FIRSTHDR(&message);
        if header.is_null()
            || (*header).cmsg_level != libc::SOL_SOCKET
            || (*header).cmsg_type != libc::SCM_RIGHTS
            || (*header).cmsg_len != libc::CMSG_LEN(std::mem::size_of::<i32>() as u32) as usize
        {
            libc::_exit(122);
        }
        let listener = std::ptr::read_unaligned(libc::CMSG_DATA(header).cast::<i32>());
        loop {
            let mut notification: libc::seccomp_notif = std::mem::zeroed();
            if libc::syscall(libc::SYS_ioctl, listener, 0xC050_2100u64, &mut notification) != 0 {
                let errno = *libc::__errno_location();
                if errno == libc::EINTR || errno == libc::ENOENT {
                    continue;
                }
                libc::_exit(123);
            }
            if libc::syscall(
                libc::SYS_sendto,
                socket,
                &notification,
                std::mem::size_of::<libc::seccomp_notif>(),
                libc::MSG_NOSIGNAL,
                0,
                0,
            ) != std::mem::size_of::<libc::seccomp_notif>() as libc::c_long
            {
                libc::_exit(124);
            }
            let errno = descriptor_transport::receive_apply(socket, listener, notification.id);
            if libc::syscall(libc::SYS_sendto, socket, &errno, std::mem::size_of::<i32>(), libc::MSG_NOSIGNAL, 0, 0)
                != std::mem::size_of::<i32>() as libc::c_long
            {
                libc::_exit(126);
            }
        }
    }
}

#[cfg(target_os = "linux")]
fn reap_killed_child(child: &mut std::process::Child) -> Result<(), RunError> {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return Ok(()),
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(2)),
            Ok(None) => return Err(error("killed protected leaf did not exit before teardown deadline")),
            Err(failure) => return Err(error(format!("reaping protected leaf: {failure}"))),
        }
    }
}

#[cfg(target_os = "linux")]
fn filter_instructions() -> Result<Vec<libc::sock_filter>, RunError> {
    let architecture = if cfg!(target_arch = "x86_64") {
        0xC000_003E
    } else if cfg!(target_arch = "aarch64") {
        0xC000_00B7
    } else {
        return Err(error("unsupported proof audit architecture"));
    };
    let statement = |code: u16, k: u32| libc::sock_filter { code, jt: 0, jf: 0, k };
    let jump = |k: u32, jt: u8, jf: u8| libc::sock_filter {
        code: (libc::BPF_JMP | libc::BPF_JEQ | libc::BPF_K) as u16,
        jt,
        jf,
        k,
    };
    let mut instructions = vec![
        statement((libc::BPF_LD | libc::BPF_W | libc::BPF_ABS) as u16, 4),
        jump(architecture, 1, 0),
        statement((libc::BPF_RET | libc::BPF_K) as u16, libc::SECCOMP_RET_KILL_PROCESS),
        statement((libc::BPF_LD | libc::BPF_W | libc::BPF_ABS) as u16, 0),
    ];
    #[cfg(target_arch = "x86_64")]
    {
        instructions.push(libc::sock_filter {
            code: (libc::BPF_JMP | libc::BPF_JSET | libc::BPF_K) as u16,
            jt: 0,
            jf: 1,
            k: 0x4000_0000,
        });
        instructions.push(statement((libc::BPF_RET | libc::BPF_K) as u16, libc::SECCOMP_RET_KILL_PROCESS));
    }
    // clone3 flags live in mutable tracee memory. CONTINUE cannot safely
    // authorize them; ENOSYS preserves libc's scalar-clone thread fallback.
    instructions.push(jump(libc::SYS_clone3 as u32, 0, 1));
    instructions.push(statement((libc::BPF_RET | libc::BPF_K) as u16, libc::SECCOMP_RET_ERRNO | libc::ENOSYS as u32));
    let mut syscalls = vec![libc::SYS_execve, libc::SYS_execveat, libc::SYS_clone];
    #[cfg(target_arch = "x86_64")]
    syscalls.extend([libc::SYS_fork, libc::SYS_vfork]);
    for syscall in syscalls {
        instructions.push(jump(syscall as u32, 0, 1));
        instructions.push(statement((libc::BPF_RET | libc::BPF_K) as u16, libc::SECCOMP_RET_USER_NOTIF));
    }
    instructions.push(statement((libc::BPF_RET | libc::BPF_K) as u16, libc::SECCOMP_RET_ALLOW));
    Ok(instructions)
}

/// The pre-exec child may use only allocation-free, async-signal-safe calls.
#[cfg(target_os = "linux")]
fn install_filter(instructions: &[libc::sock_filter]) -> std::io::Result<i32> {
    let program = libc::sock_fprog {
        len: u16::try_from(instructions.len()).map_err(|_| std::io::Error::from_raw_os_error(libc::EINVAL))?,
        filter: instructions.as_ptr().cast_mut(),
    };
    if unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) } != 0 {
        return Err(std::io::Error::last_os_error());
    }
    let fd = unsafe {
        libc::syscall(
            libc::SYS_seccomp,
            libc::SECCOMP_SET_MODE_FILTER,
            libc::SECCOMP_FILTER_FLAG_NEW_LISTENER,
            &program,
        )
    };
    if fd < 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(fd as i32)
}
#[cfg(target_os = "linux")]
fn observe_exec(notification: &libc::seccomp_notif, expected_path: Option<&Path>) -> RawExec {
    let pointer = if notification.data.nr as i64 == libc::SYS_execve {
        notification.data.args[0]
    } else {
        notification.data.args[1]
    };
    let mut bytes = Vec::new();
    let mut failure = None;
    for offset in 0..4096u64 {
        let mut byte = 0u8;
        let local = libc::iovec {
            iov_base: (&mut byte as *mut u8).cast(),
            iov_len: 1,
        };
        let remote = libc::iovec {
            iov_base: pointer.wrapping_add(offset) as *mut libc::c_void,
            iov_len: 1,
        };
        if unsafe { libc::process_vm_readv(notification.pid as i32, &local, 1, &remote, 1, 0) } != 1 {
            failure = Some("unreadable executable path");
            break;
        }
        if byte == 0 {
            break;
        }
        bytes.push(byte);
    }
    use std::os::unix::ffi::OsStringExt;
    let tracee_path = PathBuf::from(std::ffi::OsString::from_vec(bytes));
    let base = if tracee_path.is_absolute() {
        PathBuf::from(format!("/proc/{}/root", notification.pid))
    } else {
        PathBuf::from(format!("/proc/{}/cwd", notification.pid))
    };
    let resolved_path = base.join(tracee_path.strip_prefix("/").unwrap_or(&tracee_path));
    // Never open an unplanned path on behalf of a hostile tracee (e.g. a
    // proc file whose read blocks). Only the two immutable planned images
    // require measurement; every later exec is denied without opening it.
    let hash = if failure.is_none() && expected_path == Some(tracee_path.as_path()) {
        digest(&resolved_path).map(|(_, d)| d).unwrap_or_default()
    } else {
        String::new()
    };
    RawExec {
        pid: notification.pid,
        syscall: notification.data.nr,
        tracee_path,
        resolved_path,
        digest_blake3: hash,
        allowed: false,
        reason: failure.unwrap_or("pending exact action admission").into(),
        is_exec: true,
    }
}

#[cfg(target_os = "linux")]
fn thread_clone(notification: &libc::seccomp_notif) -> bool {
    if notification.data.nr as i64 != libc::SYS_clone {
        return false;
    }
    let flags = notification.data.args[0];
    flags & (libc::CLONE_THREAD | libc::CLONE_VM | libc::CLONE_SIGHAND) as u64
        == (libc::CLONE_THREAD | libc::CLONE_VM | libc::CLONE_SIGHAND) as u64
}

#[cfg(all(test, target_os = "linux"))]
#[path = "source_built_recipe_exec_tests.rs"]
mod tests;
