// machine-artifact-public: filegen.command-reports
use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use crunch_project::CurrentFileFact;
use crunch_project::CurrentFileState;
use crunch_project::FilegenAction;
use crunch_project::FilegenBlocker;
use crunch_project::FilegenOperation;
use crunch_project::FilegenPlan;
use crunch_project::FilegenPlanRequest;
use crunch_project::GeneratedFileDeclaration;
use crunch_project::GeneratedFileMaterialization;
use crunch_project::plan_file_generation;
use crunch_project::verify_filegen_apply_plan;
use serde::Deserialize;
use serde::Serialize;

use crate::errors::RunError;

const DEFAULT_MANIFEST_FILE: &str = "mantle-project.ncl";
const FILEGEN_STATE_FILE: &str = ".mantle/filegen-state.json";
const FILEGEN_STATE_SCHEMA: &str = "mantle-project-filegen-state-v1";
const APPLY_FAILURE_EXIT_CODE: u8 = 3;
const MAX_FILEGEN_FILES: usize = 4096;
const MAX_FILEGEN_FACTS: usize = MAX_FILEGEN_FILES.saturating_mul(2);

#[derive(Debug, Clone)]
pub struct FilegenPlanOptions<'a> {
    pub root: &'a Path,
    pub manifest: &'a Path,
    pub plan_out: Option<&'a Path>,
    pub json: bool,
}

#[derive(Debug, Clone)]
pub struct FilegenApplyOptions<'a> {
    pub root: &'a Path,
    pub manifest: &'a Path,
    pub reviewed_plan: &'a Path,
    pub json: bool,
}

#[derive(Debug, Deserialize)]
struct FilegenManifest {
    #[serde(default = "empty_generated_file_declarations")]
    files: Vec<GeneratedFileDeclaration>,
}

fn empty_generated_file_declarations() -> Vec<GeneratedFileDeclaration> {
    Vec::new()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct FilegenState {
    schema: String,
    files: BTreeMap<String, String>,
}

pub fn cmd_filegen_plan(options: FilegenPlanOptions<'_>) -> Result<(), RunError> {
    let declarations = load_filegen_declarations(options.root, options.manifest)?;
    let state = load_filegen_state(options.root)?;
    let facts = current_file_facts(options.root, &declarations, &state)?;
    let plan = plan_file_generation(FilegenPlanRequest {
        declarations,
        current_files: facts,
    });
    if let Some(plan_out) = options.plan_out {
        write_json_file(plan_out, &plan, "filegen plan")?;
    }
    render_filegen_plan(&plan, options.json)?;
    if plan.has_blockers() {
        return Err(RunError::Reported(APPLY_FAILURE_EXIT_CODE));
    }
    Ok(())
}

pub fn cmd_filegen_apply(options: FilegenApplyOptions<'_>) -> Result<(), RunError> {
    let reviewed_plan = read_json_file::<FilegenPlan>(options.reviewed_plan, "reviewed filegen plan")?;
    let declarations = load_filegen_declarations(options.root, options.manifest)?;
    let state = load_filegen_state(options.root)?;
    let facts = current_file_facts(options.root, &declarations, &state)?;
    let current_plan = plan_file_generation(FilegenPlanRequest {
        declarations,
        current_files: facts,
    });
    if let Err(blockers) = verify_filegen_apply_plan(&reviewed_plan, &current_plan) {
        render_filegen_blockers(&blockers, options.json)?;
        return Err(RunError::Reported(APPLY_FAILURE_EXIT_CODE));
    }
    apply_filegen_operations(options.root, &current_plan.operations)?;
    write_filegen_state(options.root, &current_plan)?;
    render_filegen_plan(&current_plan, options.json)
}

fn load_filegen_declarations(root: &Path, manifest: &Path) -> Result<Vec<GeneratedFileDeclaration>, RunError> {
    let manifest_path = if manifest.is_absolute() {
        manifest.to_path_buf()
    } else {
        root.join(manifest)
    };
    if !manifest_path.is_file() {
        return Err(RunError::Internal(format!(
            "{} not found; run `mantle init` or pass --manifest",
            manifest_path.display()
        )));
    }
    let evaluation_search_roots = vec![root.as_os_str().to_owned()];
    let manifest: FilegenManifest = crunch_eval::evaluate_and_deserialize(&manifest_path, &evaluation_search_roots)
        .map_err(|err| {
            RunError::Eval(format!("loading filegen declarations from {}: {err}", manifest_path.display()))
        })?;
    if manifest.files.len() > MAX_FILEGEN_FILES {
        return Err(RunError::Internal(format!(
            "too many generated file declarations: {} > {MAX_FILEGEN_FILES}",
            manifest.files.len()
        )));
    }
    debug_assert!(manifest_path.is_file());
    debug_assert!(manifest.files.len() <= MAX_FILEGEN_FILES);
    Ok(manifest.files)
}

fn current_file_facts(
    root: &Path,
    declarations: &[GeneratedFileDeclaration],
    state: &FilegenState,
) -> Result<Vec<CurrentFileFact>, RunError> {
    let fact_count = declarations
        .len()
        .checked_add(state.files.len())
        .ok_or_else(|| RunError::Internal("filegen fact count overflowed usize".to_string()))?;
    if fact_count > MAX_FILEGEN_FACTS {
        return Err(RunError::Internal(format!("too many current file facts: {fact_count} > {MAX_FILEGEN_FACTS}")));
    }
    let mut seen = BTreeMap::<String, ()>::new();
    let mut facts = Vec::with_capacity(fact_count);
    for declaration in declarations {
        let Some(target) = normalized_target_for_shell(&declaration.target) else {
            facts.push(CurrentFileFact {
                target: declaration.target.clone(),
                state: CurrentFileState::Missing,
            });
            continue;
        };
        if seen.insert(target.clone(), ()).is_none() {
            facts.push(current_file_fact(root, &target, state)?);
        }
    }
    for target in state.files.keys() {
        if seen.contains_key(target) {
            continue;
        }
        if normalized_target_for_shell(target).is_none() {
            facts.push(CurrentFileFact {
                target: target.clone(),
                state: CurrentFileState::Missing,
            });
            continue;
        }
        facts.push(current_file_fact(root, target, state)?);
    }
    debug_assert!(facts.len() <= fact_count);
    debug_assert!(seen.len() <= fact_count);
    Ok(facts)
}

fn current_file_fact(root: &Path, target: &str, state: &FilegenState) -> Result<CurrentFileFact, RunError> {
    debug_assert!(!target.is_empty());
    debug_assert!(normalized_target_for_shell(target).is_some());
    let path = root.join(target);
    let state_digest = state.files.get(target);
    let metadata = match fs::symlink_metadata(&path) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == ErrorKind::NotFound => {
            return Ok(CurrentFileFact {
                target: target.to_string(),
                state: CurrentFileState::Missing,
            });
        }
        Err(err) => {
            return Err(RunError::Internal(format!("reading generated file metadata {}: {err}", path.display())));
        }
    };
    if metadata.file_type().is_symlink() {
        let link_target = fs::read_link(&path)
            .map_err(|err| RunError::Internal(format!("reading generated symlink {}: {err}", path.display())))?;
        let link_text = link_target.to_string_lossy().into_owned();
        let digest = blake3_hex(link_text.as_bytes());
        let is_managed = state_digest.is_some_and(|expected| *expected == digest);
        return Ok(CurrentFileFact {
            target: target.to_string(),
            state: CurrentFileState::Symlink {
                target: link_text,
                managed: is_managed,
            },
        });
    }
    let bytes = fs::read(&path)
        .map_err(|err| RunError::Internal(format!("reading generated file {}: {err}", path.display())))?;
    let digest = blake3_hex(&bytes);
    let state = if state_digest.is_some_and(|expected| *expected == digest) {
        CurrentFileState::Managed { digest_blake3: digest }
    } else {
        CurrentFileState::Unmanaged { digest_blake3: digest }
    };
    Ok(CurrentFileFact {
        target: target.to_string(),
        state,
    })
}

fn apply_filegen_operations(root: &Path, operations: &[FilegenOperation]) -> Result<(), RunError> {
    if operations.len() > MAX_FILEGEN_FACTS {
        return Err(RunError::Internal(format!(
            "too many filegen operations: {} > {MAX_FILEGEN_FACTS}",
            operations.len()
        )));
    }
    debug_assert!(MAX_FILEGEN_FACTS >= MAX_FILEGEN_FILES);
    debug_assert!(operations.len() <= MAX_FILEGEN_FACTS);
    for operation in operations {
        match operation.action {
            FilegenAction::Create | FilegenAction::Update => write_filegen_operation(root, operation)?,
            FilegenAction::Unchanged | FilegenAction::Stale => {}
            FilegenAction::Conflict => {
                return Err(RunError::Internal(format!("filegen conflict reached apply for {}", operation.target)));
            }
        }
    }
    Ok(())
}

fn write_filegen_operation(root: &Path, operation: &FilegenOperation) -> Result<(), RunError> {
    debug_assert!(!FILEGEN_STATE_SCHEMA.is_empty());
    debug_assert!(MAX_FILEGEN_FILES > 0);
    let path = root.join(&operation.target);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| RunError::Internal(format!("creating generated file dir {}: {err}", parent.display())))?;
    }
    match operation.materialization {
        GeneratedFileMaterialization::Copy => write_file_atomic(&path, operation.content.as_bytes(), &operation.target),
        GeneratedFileMaterialization::Symlink => write_symlink(&path, &operation.content),
    }
}

#[cfg(unix)]
fn write_symlink(path: &Path, target: &str) -> Result<(), RunError> {
    debug_assert!(!FILEGEN_STATE_SCHEMA.is_empty());
    debug_assert!(MAX_FILEGEN_FILES > 0);
    match fs::remove_file(path) {
        Ok(()) => {}
        Err(err) if err.kind() == ErrorKind::NotFound => {}
        Err(err) => return Err(RunError::Internal(format!("removing old symlink {}: {err}", path.display()))),
    }
    std::os::unix::fs::symlink(target, path)
        .map_err(|err| RunError::Internal(format!("writing generated symlink {}: {err}", path.display())))
}

#[cfg(not(unix))]
fn write_symlink(path: &Path, _target: &str) -> Result<(), RunError> {
    Err(RunError::Internal(format!(
        "symlink generated files are unsupported on this platform: {}",
        path.display()
    )))
}

fn write_filegen_state(root: &Path, plan: &FilegenPlan) -> Result<(), RunError> {
    if plan.operations.len() > MAX_FILEGEN_FACTS {
        return Err(RunError::Internal(format!(
            "too many filegen state operations: {} > {MAX_FILEGEN_FACTS}",
            plan.operations.len()
        )));
    }
    let files = plan
        .operations
        .iter()
        .filter_map(|operation| match operation.action {
            FilegenAction::Create | FilegenAction::Update | FilegenAction::Unchanged => {
                Some((operation.target.clone(), operation.desired_digest_blake3.clone()))
            }
            FilegenAction::Stale | FilegenAction::Conflict => None,
        })
        .collect::<BTreeMap<_, _>>();
    debug_assert!(files.len() <= plan.operations.len());
    debug_assert!(files.len() <= MAX_FILEGEN_FACTS);
    let state = FilegenState {
        schema: FILEGEN_STATE_SCHEMA.to_string(),
        files,
    };
    write_json_file(&root.join(FILEGEN_STATE_FILE), &state, "filegen state")
}

fn load_filegen_state(root: &Path) -> Result<FilegenState, RunError> {
    let path = root.join(FILEGEN_STATE_FILE);
    if !path.exists() {
        return Ok(FilegenState {
            schema: FILEGEN_STATE_SCHEMA.to_string(),
            files: BTreeMap::new(),
        });
    }
    let state = read_json_file::<FilegenState>(&path, "filegen state")?;
    if state.files.len() > MAX_FILEGEN_FILES {
        return Err(RunError::Internal(format!(
            "too many filegen state entries: {} > {MAX_FILEGEN_FILES}",
            state.files.len()
        )));
    }
    if state.schema != FILEGEN_STATE_SCHEMA {
        return Err(RunError::Internal(format!(
            "unsupported filegen state schema {} in {}",
            state.schema,
            path.display()
        )));
    }
    debug_assert_eq!(state.schema, FILEGEN_STATE_SCHEMA);
    debug_assert!(state.files.len() <= MAX_FILEGEN_FILES);
    Ok(state)
}

fn render_filegen_plan(plan: &FilegenPlan, json: bool) -> Result<(), RunError> {
    debug_assert!(plan.operations.len() <= MAX_FILEGEN_FACTS);
    debug_assert!(!plan.non_claim.is_empty());
    if json {
        let rendered = serde_json::to_string_pretty(plan)
            .map_err(|err| RunError::Internal(format!("rendering filegen JSON: {err}")))?;
        println!("{rendered}");
        return Ok(());
    }
    for operation in &plan.operations {
        println!(
            "{} {} {} {}",
            action_label(operation.action),
            materialization_label(operation.materialization),
            operation.target,
            operation.desired_digest_blake3
        );
    }
    for blocker in &plan.blockers {
        eprintln!("blocker {} {}: {}", blocker.code, blocker.target, blocker.message);
    }
    if plan.blockers.is_empty() {
        eprintln!("filegen plan ok ({})", plan.non_claim);
    }
    Ok(())
}

fn render_filegen_blockers(blockers: &[FilegenBlocker], json: bool) -> Result<(), RunError> {
    if json {
        let rendered = serde_json::to_string_pretty(blockers)
            .map_err(|err| RunError::Internal(format!("rendering filegen blockers JSON: {err}")))?;
        println!("{rendered}");
        return Ok(());
    }
    for blocker in blockers {
        eprintln!("blocker {} {}: {}", blocker.code, blocker.target, blocker.message);
    }
    Ok(())
}

fn action_label(action: FilegenAction) -> &'static str {
    match action {
        FilegenAction::Create => "create",
        FilegenAction::Update => "update",
        FilegenAction::Unchanged => "unchanged",
        FilegenAction::Stale => "stale",
        FilegenAction::Conflict => "conflict",
    }
}

fn materialization_label(materialization: GeneratedFileMaterialization) -> &'static str {
    match materialization {
        GeneratedFileMaterialization::Copy => "copy",
        GeneratedFileMaterialization::Symlink => "symlink",
    }
}

fn normalized_target_for_shell(target: &str) -> Option<String> {
    if target.is_empty() || target.starts_with('/') {
        return None;
    }
    let component_count = Path::new(target).components().count();
    let mut parts = Vec::with_capacity(component_count);
    for component in Path::new(target).components() {
        match component {
            Component::Normal(part) => parts.push(part.to_string_lossy().into_owned()),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    if parts.is_empty() {
        None
    } else {
        debug_assert!(parts.len() <= component_count);
        debug_assert!(!parts.iter().any(String::is_empty));
        Some(parts.join("/"))
    }
}

fn write_file_atomic(path: &Path, content: &[u8], label: &str) -> Result<(), RunError> {
    let tmp_path = atomic_tmp_path(path);
    debug_assert_ne!(tmp_path, path);
    debug_assert_eq!(tmp_path.parent(), path.parent());
    fs::write(&tmp_path, content)
        .map_err(|err| RunError::Internal(format!("writing temporary generated file {label}: {err}")))?;
    fs::rename(&tmp_path, path)
        .map_err(|err| RunError::Internal(format!("committing generated file {label}: {err}")))?;
    debug_assert!(path.exists());
    debug_assert!(!tmp_path.exists());
    Ok(())
}

fn atomic_tmp_path(path: &Path) -> PathBuf {
    let file_name = path.file_name().and_then(|name| name.to_str()).unwrap_or("filegen");
    path.with_file_name(format!("{file_name}.tmp"))
}

fn read_json_file<T: for<'de> Deserialize<'de>>(path: &Path, label: &str) -> Result<T, RunError> {
    let text = fs::read_to_string(path)
        .map_err(|err| RunError::Internal(format!("reading {label} {}: {err}", path.display())))?;
    serde_json::from_str(&text).map_err(|err| RunError::Internal(format!("parsing {label} {}: {err}", path.display())))
}

fn write_json_file<T: Serialize>(path: &Path, value: &T, label: &str) -> Result<(), RunError> {
    debug_assert!(!FILEGEN_STATE_SCHEMA.is_empty());
    debug_assert!(MAX_FILEGEN_FILES > 0);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| RunError::Internal(format!("creating {label} dir {}: {err}", parent.display())))?;
    }
    let json =
        serde_json::to_vec_pretty(value).map_err(|err| RunError::Internal(format!("serializing {label}: {err}")))?;
    write_file_atomic(path, &json, &path.display().to_string())
}

fn blake3_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

pub fn default_manifest_path() -> PathBuf {
    PathBuf::from(DEFAULT_MANIFEST_FILE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_target_normalization_rejects_absolute_and_parent_paths() {
        assert_eq!(normalized_target_for_shell("a/./b"), Some("a/b".to_string()));
        assert_eq!(normalized_target_for_shell("/abs"), None);
        assert_eq!(normalized_target_for_shell("../escape"), None);
    }

    #[test]
    fn action_and_materialization_labels_are_stable() {
        assert_eq!(action_label(FilegenAction::Create), "create");
        assert_eq!(action_label(FilegenAction::Conflict), "conflict");
        assert_eq!(materialization_label(GeneratedFileMaterialization::Copy), "copy");
        assert_eq!(materialization_label(GeneratedFileMaterialization::Symlink), "symlink");
    }

    #[test]
    fn unsafe_targets_do_not_probe_outside_project_root() {
        let tmp = tempfile::tempdir().unwrap();
        let state = FilegenState::default();
        let declarations = vec![GeneratedFileDeclaration {
            name: "escape".to_string(),
            target: "../outside".to_string(),
            content: crunch_project::GeneratedFileContent::Inline { text: "x".to_string() },
            materialization: GeneratedFileMaterialization::Copy,
            contract: None,
        }];

        let facts = current_file_facts(tmp.path(), &declarations, &state).unwrap();

        assert_eq!(facts[0].target, "../outside");
        assert_eq!(facts[0].state, CurrentFileState::Missing);
    }
}
