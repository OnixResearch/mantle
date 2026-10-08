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
use mantle_application_contract::ApplicationOutcome;
use mantle_application_contract::CapabilityError;
use mantle_application_contract::CommandFamily;
use mantle_application_contract::EffectId;
use mantle_application_contract::EffectKind;
use mantle_application_contract::EffectMeasure;
use mantle_application_contract::EffectOutput;
use mantle_application_contract::EffectPlan;
use mantle_application_contract::EffectSpec;
use mantle_application_contract::ExpectedOutput;
use mantle_application_contract::Observation;
use mantle_application_contract::ObservationStatus;
use mantle_application_contract::PlanError;
use mantle_application_contract::classify_observations;
use mantle_application_contract::plan_effects;
use serde::Deserialize;
use serde::Serialize;

use crate::errors::RunError;

const DEFAULT_MANIFEST_FILE: &str = "mantle-project.ncl";
const FILEGEN_STATE_FILE: &str = ".mantle/filegen-state.json";
const FILEGEN_STATE_SCHEMA: &str = "mantle-project-filegen-state-v1";
const APPLY_FAILURE_EXIT_CODE: u8 = 3;
const MAX_FILEGEN_FILES: usize = 4096;
const MAX_FILEGEN_FACTS: usize = MAX_FILEGEN_FILES.saturating_mul(2);

const _: () = {
    assert!(MAX_FILEGEN_FILES > 0);
    assert!(MAX_FILEGEN_FACTS >= MAX_FILEGEN_FILES);
};

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

const INPUT_READ: &str = "filegen-input-read";
const REVIEWED_READ: &str = "reviewed-plan-read";
const PLAN_WRITE: &str = "filegen-plan-write";
const PLAN_READBACK: &str = "filegen-plan-readback";
const APPLY_WRITE: &str = "filegen-apply-write";
const APPLY_READBACK: &str = "filegen-apply-readback";

struct FilegenInputs {
    declarations: Vec<GeneratedFileDeclaration>,
    facts: Vec<CurrentFileFact>,
}

/// Input authority shared by the two operation-specific ports.
trait FilegenInputPort {
    fn read_inputs(&self, root: &Path, manifest: &Path) -> Result<PortFact<FilegenInputs>, FilegenPortError>;
}

/// The planning port publishes reviewed plans, not generated-file apply operations.
trait FilegenPlanPort: FilegenInputPort {
    fn write_plan(&self, path: &Path, plan: &FilegenPlan) -> Result<PortFact<()>, FilegenPortError>;
    fn read_back_plan(&self, path: &Path, plan: &FilegenPlan) -> Result<PortFact<()>, FilegenPortError>;
}

/// Apply reads the reviewed plan, writes generated files and checks read-back.
trait FilegenApplyPort: FilegenInputPort {
    fn read_plan(&self, path: &Path) -> Result<PortFact<FilegenPlan>, FilegenPortError>;
    fn apply(&self, root: &Path, plan: &FilegenPlan) -> Result<PortFact<()>, FilegenPortError>;
    fn read_back(&self, root: &Path, plan: &FilegenPlan) -> Result<PortFact<()>, FilegenPortError>;
}

struct PortFact<T> {
    value: T,
    observation: Observation,
}

impl<T> PortFact<T> {
    fn succeeded(value: T, effect_id: &str, kind: EffectKind, output: EffectOutput) -> Self {
        Self {
            value,
            observation: Observation {
                effect_id: EffectId(effect_id.to_string()),
                kind,
                status: ObservationStatus::Succeeded,
                output,
                usage: EffectMeasure::Calls(1),
                diagnostics_code: None,
            },
        }
    }
}

#[derive(Debug)]
struct FilegenPortError {
    capability: CapabilityError,
    effect_id: &'static str,
    kind: EffectKind,
    eval: bool,
}

impl FilegenPortError {
    fn from_run(effect_id: &'static str, kind: EffectKind, code: &str, error: RunError) -> Self {
        let eval = matches!(error, RunError::Eval(_));
        Self {
            capability: CapabilityError::new(code, error.message()),
            effect_id,
            kind,
            eval,
        }
    }

    fn into_parts(self) -> (Observation, RunError) {
        let Self {
            capability,
            effect_id,
            kind,
            eval,
        } = self;
        let CapabilityError { code, detail } = capability;
        let observation = Observation {
            effect_id: EffectId(effect_id.to_string()),
            kind,
            status: ObservationStatus::Failed,
            output: EffectOutput::None,
            usage: EffectMeasure::Calls(1),
            diagnostics_code: Some(code),
        };
        let error = if eval {
            RunError::Eval(detail)
        } else {
            RunError::Internal(detail)
        };
        (observation, error)
    }
}

struct FsFilegenPort;

impl FilegenInputPort for FsFilegenPort {
    fn read_inputs(&self, root: &Path, manifest: &Path) -> Result<PortFact<FilegenInputs>, FilegenPortError> {
        let declarations = load_filegen_declarations(root, manifest).map_err(|err| {
            FilegenPortError::from_run(INPUT_READ, EffectKind::ReadFiles, "filegen-manifest-read", err)
        })?;
        let state = load_filegen_state(root)
            .map_err(|err| FilegenPortError::from_run(INPUT_READ, EffectKind::ReadFiles, "filegen-state-read", err))?;
        let facts = current_file_facts(root, &declarations, &state)
            .map_err(|err| FilegenPortError::from_run(INPUT_READ, EffectKind::ReadFiles, "filegen-facts-read", err))?;
        Ok(PortFact::succeeded(
            FilegenInputs { declarations, facts },
            INPUT_READ,
            EffectKind::ReadFiles,
            EffectOutput::None,
        ))
    }
}

impl FilegenPlanPort for FsFilegenPort {
    fn write_plan(&self, path: &Path, plan: &FilegenPlan) -> Result<PortFact<()>, FilegenPortError> {
        write_json_file(path, plan, "filegen plan")
            .map_err(|err| FilegenPortError::from_run(PLAN_WRITE, EffectKind::WriteFiles, "filegen-plan-write", err))?;
        Ok(PortFact::succeeded((), PLAN_WRITE, EffectKind::WriteFiles, EffectOutput::None))
    }

    fn read_back_plan(&self, path: &Path, plan: &FilegenPlan) -> Result<PortFact<()>, FilegenPortError> {
        let observed: FilegenPlan = read_json_file(path, "filegen plan").map_err(|err| {
            FilegenPortError::from_run(PLAN_READBACK, EffectKind::ReadFiles, "filegen-plan-readback", err)
        })?;
        if observed != *plan {
            let mismatch =
                RunError::Internal(format!("filegen plan {} did not read back as published", path.display()));
            return Err(FilegenPortError::from_run(
                PLAN_READBACK,
                EffectKind::ReadFiles,
                "filegen-plan-readback-mismatch",
                mismatch,
            ));
        }
        Ok(PortFact::succeeded((), PLAN_READBACK, EffectKind::ReadFiles, EffectOutput::None))
    }
}

impl FilegenApplyPort for FsFilegenPort {
    fn read_plan(&self, path: &Path) -> Result<PortFact<FilegenPlan>, FilegenPortError> {
        let plan = read_json_file(path, "reviewed filegen plan").map_err(|err| {
            FilegenPortError::from_run(REVIEWED_READ, EffectKind::ReadFiles, "reviewed-plan-read", err)
        })?;
        Ok(PortFact::succeeded(plan, REVIEWED_READ, EffectKind::ReadFiles, EffectOutput::None))
    }

    fn apply(&self, root: &Path, plan: &FilegenPlan) -> Result<PortFact<()>, FilegenPortError> {
        apply_filegen_operations(root, &plan.operations).map_err(|err| {
            FilegenPortError::from_run(APPLY_WRITE, EffectKind::WriteFiles, "filegen-files-write", err)
        })?;
        write_filegen_state(root, plan).map_err(|err| {
            FilegenPortError::from_run(APPLY_WRITE, EffectKind::WriteFiles, "filegen-state-write", err)
        })?;
        Ok(PortFact::succeeded((), APPLY_WRITE, EffectKind::WriteFiles, EffectOutput::None))
    }

    fn read_back(&self, root: &Path, plan: &FilegenPlan) -> Result<PortFact<()>, FilegenPortError> {
        let observed_schema = verify_filegen_readback(root, plan).map_err(|err| {
            FilegenPortError::from_run(APPLY_READBACK, EffectKind::ReadFiles, FILEGEN_APPLY_STATE_CODE, err)
        })?;
        Ok(PortFact::succeeded(
            (),
            APPLY_READBACK,
            EffectKind::ReadFiles,
            EffectOutput::Identity(observed_schema),
        ))
    }
}

enum FilegenCommandError {
    Plan(PlanError),
    Run(RunError),
}

impl From<PlanError> for FilegenCommandError {
    fn from(error: PlanError) -> Self {
        Self::Plan(error)
    }
}

impl From<RunError> for FilegenCommandError {
    fn from(error: RunError) -> Self {
        Self::Run(error)
    }
}

impl FilegenCommandError {
    fn into_run(self) -> RunError {
        match self {
            Self::Plan(error) => {
                RunError::Internal(format!("filegen effect plan rejected ({}): {error:?}", error.code()))
            }
            Self::Run(error) => error,
        }
    }
}

fn observe_filegen<T>(
    plan: &EffectPlan,
    observations: &mut Vec<Observation>,
    result: Result<PortFact<T>, FilegenPortError>,
) -> Result<T, RunError> {
    match result {
        Ok(fact) => {
            observations.push(fact.observation);
            Ok(fact.value)
        }
        Err(error) => {
            let (observation, error) = error.into_parts();
            observations.push(observation);
            for remaining in plan.effects.iter().skip(observations.len()) {
                observations.push(Observation {
                    effect_id: remaining.effect_id.clone(),
                    kind: remaining.kind,
                    status: ObservationStatus::Skipped,
                    output: EffectOutput::None,
                    usage: EffectMeasure::Calls(0),
                    diagnostics_code: Some("prior-effect-failed".to_string()),
                });
            }
            match classify_observations(plan, observations) {
                ApplicationOutcome::Failed { .. } => Err(error),
                other => Err(RunError::Internal(format!("filegen observations were inconsistent: {other:?}"))),
            }
        }
    }
}

fn complete_filegen(plan: &EffectPlan, observations: &[Observation]) -> Result<(), RunError> {
    match classify_observations(plan, observations) {
        ApplicationOutcome::Completed => Ok(()),
        other => Err(RunError::Internal(format!("filegen observations were inconsistent: {other:?}"))),
    }
}

fn spec<'a>(effect_id: &'a str, kind: EffectKind, expected_output: ExpectedOutput<'a>) -> EffectSpec<'a> {
    EffectSpec {
        effect_id,
        kind,
        limit: EffectMeasure::Calls(1),
        expected_output,
    }
}

pub fn cmd_filegen_plan(options: FilegenPlanOptions<'_>) -> Result<(), RunError> {
    run_filegen_plan(&FsFilegenPort, options).map_err(FilegenCommandError::into_run)
}

fn run_filegen_plan(port: &impl FilegenPlanPort, options: FilegenPlanOptions<'_>) -> Result<(), FilegenCommandError> {
    let effects = if options.plan_out.is_some() {
        plan_effects(CommandFamily::Realization, &[
            spec(INPUT_READ, EffectKind::ReadFiles, ExpectedOutput::None),
            spec(PLAN_WRITE, EffectKind::WriteFiles, ExpectedOutput::None),
            spec(PLAN_READBACK, EffectKind::ReadFiles, ExpectedOutput::None),
        ])?
    } else {
        plan_effects(CommandFamily::Realization, &[spec(INPUT_READ, EffectKind::ReadFiles, ExpectedOutput::None)])?
    };
    let mut observations = Vec::with_capacity(effects.effects.len());
    let inputs = observe_filegen(&effects, &mut observations, port.read_inputs(options.root, options.manifest))?;
    let plan = plan_file_generation(FilegenPlanRequest {
        declarations: inputs.declarations,
        current_files: inputs.facts,
    });
    if let Some(path) = options.plan_out {
        observe_filegen(&effects, &mut observations, port.write_plan(path, &plan))?;
        observe_filegen(&effects, &mut observations, port.read_back_plan(path, &plan))?;
    }
    complete_filegen(&effects, &observations)?;
    render_filegen_plan(&plan, options.json)?;
    if plan.has_blockers() {
        Err(RunError::Reported(APPLY_FAILURE_EXIT_CODE).into())
    } else {
        Ok(())
    }
}

pub fn cmd_filegen_apply(options: FilegenApplyOptions<'_>) -> Result<(), RunError> {
    run_filegen_apply(&FsFilegenPort, options).map_err(FilegenCommandError::into_run)
}

fn run_filegen_apply(
    port: &impl FilegenApplyPort,
    options: FilegenApplyOptions<'_>,
) -> Result<(), FilegenCommandError> {
    // Both phases are admitted before reading the reviewed plan or project.
    let reads = plan_effects(CommandFamily::Realization, &[
        spec(REVIEWED_READ, EffectKind::ReadFiles, ExpectedOutput::None),
        spec(INPUT_READ, EffectKind::ReadFiles, ExpectedOutput::None),
    ])?;
    let writes = plan_effects(CommandFamily::Realization, &[
        spec(APPLY_WRITE, EffectKind::WriteFiles, ExpectedOutput::None),
        spec(APPLY_READBACK, EffectKind::ReadFiles, ExpectedOutput::Identity(FILEGEN_STATE_SCHEMA)),
    ])?;
    let mut read_observations = Vec::with_capacity(reads.effects.len());
    let reviewed_plan = observe_filegen(&reads, &mut read_observations, port.read_plan(options.reviewed_plan))?;
    let inputs = observe_filegen(&reads, &mut read_observations, port.read_inputs(options.root, options.manifest))?;
    complete_filegen(&reads, &read_observations)?;
    let current_plan = plan_file_generation(FilegenPlanRequest {
        declarations: inputs.declarations,
        current_files: inputs.facts,
    });
    if let Err(blockers) = verify_filegen_apply_plan(&reviewed_plan, &current_plan) {
        render_filegen_blockers(&blockers, options.json)?;
        return Err(RunError::Reported(APPLY_FAILURE_EXIT_CODE).into());
    }
    let mut write_observations = Vec::with_capacity(writes.effects.len());
    observe_filegen(&writes, &mut write_observations, port.apply(options.root, &current_plan))?;
    observe_filegen(&writes, &mut write_observations, port.read_back(options.root, &current_plan))?;
    complete_filegen(&writes, &write_observations)?;
    render_filegen_plan(&current_plan, options.json)?;
    Ok(())
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

/// The file entries one plan implies in the state file.
///
/// A create, update, or unchanged operation records the digest the file is
/// expected to hold; a stale or conflicting operation records nothing, because
/// the plan did not put that file in the state it claims.
fn filegen_state_files_for_plan(plan: &FilegenPlan) -> BTreeMap<String, String> {
    debug_assert!(plan.operations.len() <= MAX_FILEGEN_FACTS);
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
    files
}

/// Diagnostic code for state or generated files that contradict the applied plan.
const FILEGEN_APPLY_STATE_CODE: &str = "filegen-apply-state-mismatch";

fn verify_filegen_readback(root: &Path, plan: &FilegenPlan) -> Result<String, RunError> {
    let mismatch = || RunError::Internal("filegen state did not read back as the plan that was applied".to_string());
    if !root.join(FILEGEN_STATE_FILE).is_file() {
        return Err(mismatch());
    }
    let state = load_filegen_state(root)?;
    if state.files != filegen_state_files_for_plan(plan) {
        return Err(mismatch());
    }
    for operation in &plan.operations {
        if !matches!(operation.action, FilegenAction::Create | FilegenAction::Update | FilegenAction::Unchanged) {
            continue;
        }
        let fact = current_file_fact(root, &operation.target, &state)?;
        let matches_plan = match (operation.materialization, fact.state) {
            (GeneratedFileMaterialization::Copy, CurrentFileState::Managed { digest_blake3 }) => {
                digest_blake3 == operation.desired_digest_blake3
            }
            (GeneratedFileMaterialization::Symlink, CurrentFileState::Symlink { target, managed }) => {
                managed && target == operation.content
            }
            _ => false,
        };
        if !matches_plan {
            return Err(mismatch());
        }
    }
    Ok(state.schema)
}

fn write_filegen_state(root: &Path, plan: &FilegenPlan) -> Result<(), RunError> {
    if plan.operations.len() > MAX_FILEGEN_FACTS {
        return Err(RunError::Internal(format!(
            "too many filegen state operations: {} > {MAX_FILEGEN_FACTS}",
            plan.operations.len()
        )));
    }
    let files = filegen_state_files_for_plan(plan);
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
    fn unsafe_targets_do_not_probe_outside_project_root() {
        let tmp = tempfile::tempdir().unwrap();
        let project_root = tmp.path().join("project");
        fs::create_dir(&project_root).unwrap();
        fs::write(tmp.path().join("outside"), b"not part of project").unwrap();
        let state = FilegenState::default();
        let declarations = vec![GeneratedFileDeclaration {
            name: "escape".to_string(),
            target: "../outside".to_string(),
            content: crunch_project::GeneratedFileContent::Inline { text: "x".to_string() },
            materialization: GeneratedFileMaterialization::Copy,
            contract: None,
        }];

        let facts = current_file_facts(&project_root, &declarations, &state).unwrap();

        assert_eq!(facts[0].target, "../outside");
        assert_eq!(facts[0].state, CurrentFileState::Missing);
    }
}

#[cfg(test)]
mod apply_state_classification_tests {
    use super::*;

    /// Build one operation with the given action and target.
    fn operation(target: &str, action: FilegenAction, digest: &str) -> FilegenOperation {
        FilegenOperation {
            name: target.to_string(),
            target: target.to_string(),
            action,
            materialization: GeneratedFileMaterialization::Copy,
            desired_digest_blake3: digest.to_string(),
            content: "generated".to_string(),
            contract_identity: None,
        }
    }

    /// Build one plan over the given operations.
    fn plan(operations: Vec<FilegenOperation>) -> FilegenPlan {
        FilegenPlan {
            schema: String::new(),
            operations,
            blockers: Vec::new(),
            non_claim: String::new(),
        }
    }

    #[test]
    fn the_state_map_records_written_and_unchanged_targets_only() {
        let plan = plan(vec![
            operation("created.ncl", FilegenAction::Create, "aa"),
            operation("updated.ncl", FilegenAction::Update, "bb"),
            operation("kept.ncl", FilegenAction::Unchanged, "cc"),
            operation("stale.ncl", FilegenAction::Stale, "dd"),
            operation("conflict.ncl", FilegenAction::Conflict, "ee"),
        ]);
        let files = filegen_state_files_for_plan(&plan);
        assert_eq!(files.len(), 3);
        assert_eq!(files.get("created.ncl").map(String::as_str), Some("aa"));
        assert_eq!(files.get("kept.ncl").map(String::as_str), Some("cc"));
        assert!(!files.contains_key("stale.ncl"));
        assert!(!files.contains_key("conflict.ncl"));
    }

    #[test]
    fn applied_files_and_state_must_both_read_back() {
        let temp = tempfile::tempdir().unwrap();
        let digest = blake3_hex(b"generated");
        let applied = plan(vec![
            operation("created.ncl", FilegenAction::Create, &digest),
            operation("updated.ncl", FilegenAction::Update, &digest),
        ]);
        apply_filegen_operations(temp.path(), &applied.operations).unwrap();
        write_filegen_state(temp.path(), &applied).unwrap();
        verify_filegen_readback(temp.path(), &applied).unwrap();

        fs::write(temp.path().join("updated.ncl"), b"wrong bytes").unwrap();
        assert!(verify_filegen_readback(temp.path(), &applied).is_err());
        fs::write(temp.path().join("updated.ncl"), b"generated").unwrap();
        fs::remove_file(temp.path().join("created.ncl")).unwrap();
        assert!(verify_filegen_readback(temp.path(), &applied).is_err());
        fs::write(temp.path().join("created.ncl"), b"generated").unwrap();
        fs::remove_file(temp.path().join(FILEGEN_STATE_FILE)).unwrap();
        assert!(verify_filegen_readback(temp.path(), &applied).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn changed_symlink_target_fails_applied_plan_readback() {
        let temp = tempfile::tempdir().unwrap();
        let target = "target-file";
        let digest = blake3_hex(target.as_bytes());
        let mut linked = operation("linked.ncl", FilegenAction::Create, &digest);
        linked.materialization = GeneratedFileMaterialization::Symlink;
        linked.content = target.to_string();
        let applied = plan(vec![linked]);
        FsFilegenPort.apply(temp.path(), &applied).unwrap();
        verify_filegen_readback(temp.path(), &applied).unwrap();

        let link = temp.path().join("linked.ncl");
        fs::remove_file(&link).unwrap();
        std::os::unix::fs::symlink("other-target", &link).unwrap();
        assert!(verify_filegen_readback(temp.path(), &applied).is_err());
    }

    #[test]
    fn other_state_and_partial_state_fail_closed() {
        let temp = tempfile::tempdir().unwrap();
        let digest = blake3_hex(b"generated");
        let written = plan(vec![operation("created.ncl", FilegenAction::Create, &digest)]);
        let other = plan(vec![operation("different.ncl", FilegenAction::Create, &digest)]);
        apply_filegen_operations(temp.path(), &written.operations).unwrap();
        write_filegen_state(temp.path(), &written).unwrap();
        assert!(verify_filegen_readback(temp.path(), &other).is_err());
        let partial = plan(vec![
            operation("created.ncl", FilegenAction::Create, &digest),
            operation("missing.ncl", FilegenAction::Create, &digest),
        ]);
        assert!(verify_filegen_readback(temp.path(), &partial).is_err());
        verify_filegen_readback(temp.path(), &written).unwrap();
    }

    #[test]
    fn actual_readback_rejects_wrong_identity_authority_output_and_limit() {
        let temp = tempfile::tempdir().unwrap();
        let digest = blake3_hex(b"generated");
        let applied = plan(vec![operation("created.ncl", FilegenAction::Create, &digest)]);
        FsFilegenPort.apply(temp.path(), &applied).unwrap();
        let observed = FsFilegenPort.read_back(temp.path(), &applied).unwrap().observation;
        let effects = plan_effects(CommandFamily::Realization, &[spec(
            APPLY_READBACK,
            EffectKind::ReadFiles,
            ExpectedOutput::Identity(FILEGEN_STATE_SCHEMA),
        )])
        .unwrap();
        assert!(matches!(
            classify_observations(&effects, std::slice::from_ref(&observed)),
            ApplicationOutcome::Completed
        ));

        let mut wrong = observed.clone();
        wrong.effect_id = EffectId("different-effect".to_string());
        assert!(matches!(classify_observations(&effects, &[wrong]), ApplicationOutcome::Rejected { .. }));
        let mut wrong = observed.clone();
        wrong.kind = EffectKind::WriteFiles;
        assert!(matches!(classify_observations(&effects, &[wrong]), ApplicationOutcome::Contradicted { .. }));
        let mut wrong = observed.clone();
        wrong.output = EffectOutput::Identity("different-state".to_string());
        assert!(matches!(classify_observations(&effects, &[wrong]), ApplicationOutcome::Contradicted { .. }));
        let mut wrong = observed;
        wrong.usage = EffectMeasure::Calls(2);
        assert!(matches!(classify_observations(&effects, &[wrong]), ApplicationOutcome::Contradicted { .. }));
    }

    #[test]
    fn plan_write_failure_is_classified_before_reporting() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("plan.json");
        fs::create_dir(&path).unwrap();
        let effects =
            plan_effects(CommandFamily::Realization, &[spec(PLAN_WRITE, EffectKind::WriteFiles, ExpectedOutput::None)])
                .unwrap();
        let mut observations = Vec::new();
        let error = observe_filegen(&effects, &mut observations, FsFilegenPort.write_plan(&path, &plan(vec![])))
            .expect_err("failed publication cannot report a successful plan");
        assert_eq!(error.kind(), "internal");
        assert_eq!(observations[0].diagnostics_code.as_deref(), Some("filegen-plan-write"));
        assert!(matches!(classify_observations(&effects, &observations), ApplicationOutcome::Failed { .. }));
    }

    #[test]
    fn tampered_published_plan_fails_real_readback() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("plan.json");
        let written = plan(vec![operation("created.ncl", FilegenAction::Create, "aa")]);
        let changed = plan(vec![operation("created.ncl", FilegenAction::Create, "bb")]);
        FsFilegenPort.write_plan(&path, &written).unwrap();
        write_json_file(&path, &changed, "filegen plan").unwrap();
        let effects = plan_effects(CommandFamily::Realization, &[spec(
            PLAN_READBACK,
            EffectKind::ReadFiles,
            ExpectedOutput::None,
        )])
        .unwrap();
        let mut observations = Vec::new();
        observe_filegen(&effects, &mut observations, FsFilegenPort.read_back_plan(&path, &written))
            .expect_err("a substituted plan cannot be reported as published");
        assert_eq!(observations[0].diagnostics_code.as_deref(), Some("filegen-plan-readback-mismatch"));
        assert!(matches!(classify_observations(&effects, &observations), ApplicationOutcome::Failed { .. }));
    }

    #[test]
    fn partial_apply_write_is_observed_as_failure() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("blocked"), b"not a directory").unwrap();
        let digest = blake3_hex(b"generated");
        let partial = plan(vec![
            operation("first.ncl", FilegenAction::Create, &digest),
            operation("blocked/second.ncl", FilegenAction::Create, &digest),
        ]);
        let effects = plan_effects(CommandFamily::Realization, &[spec(
            APPLY_WRITE,
            EffectKind::WriteFiles,
            ExpectedOutput::None,
        )])
        .unwrap();
        let mut observations = Vec::new();
        let error = observe_filegen(&effects, &mut observations, FsFilegenPort.apply(temp.path(), &partial))
            .expect_err("partial write cannot report success");
        assert_eq!(error.kind(), "internal");
        assert_eq!(observations[0].diagnostics_code.as_deref(), Some("filegen-files-write"));
        assert_eq!(fs::read(temp.path().join("first.ncl")).unwrap(), b"generated");
        assert!(!temp.path().join(FILEGEN_STATE_FILE).exists());
        assert!(matches!(classify_observations(&effects, &observations), ApplicationOutcome::Failed { .. }));
    }
}
