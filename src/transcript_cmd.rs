// machine-artifact-public: transcript.evidence-reports
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use mantle_application_contract::ApplicationOutcome;
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
use mantle_application_contract::classify_observations;
use mantle_application_contract::plan_effects;
use serde::Deserialize;
use serde::Serialize;
use tempfile::TempDir;

use crate::errors::RunError;

const TRANSCRIPT_TMP_ENV: &str = "MANTLE_TRANSCRIPT_TMP";
const MANTLE_PREFIX: &str = "mantle";
const TEMP_PLACEHOLDER: &str = "${MANTLE_TRANSCRIPT_TMP}";
const MAX_TRANSCRIPT_BYTES: u64 = 16_777_216;
const MAX_TRANSCRIPT_LINES: usize = 100_000;
const MAX_TRANSCRIPT_BLOCKS: usize = 10_000;
const TRANSCRIPT_EVIDENCE_SCHEMA: &str = "mantle-transcript-output-v1";
const TRANSCRIPT_INPUT_EFFECT: &str = "transcript-input-read";
const TRANSCRIPT_WRITE_EFFECT: &str = "transcript-scratch-and-evidence-write";
const TRANSCRIPT_PROCESS_EFFECT: &str = "transcript-process-run";
const TRANSCRIPT_READBACK_EFFECT: &str = "transcript-evidence-readback";

trait TranscriptPort {
    fn read_input(&mut self, path: &Path) -> Result<String, RunError>;
    fn prepare_scratch(
        &mut self,
        in_place: bool,
        backend: crunch_store::StoreBackend,
        state_dir: &Path,
    ) -> Result<TranscriptScratch, RunError>;
    fn execute(
        &mut self,
        transcript: &Transcript,
        mantle_bin: &Path,
        scratch: &TranscriptScratch,
        work_dir: &Path,
    ) -> TranscriptExecution;
    fn publish(&mut self, path: &Path, evidence: TranscriptEvidence) -> Result<blake3::Hash, RunError>;
    fn readback(&mut self, path: &Path) -> Result<blake3::Hash, RunError>;
}

struct LocalTranscriptPort;

impl TranscriptPort for LocalTranscriptPort {
    fn read_input(&mut self, path: &Path) -> Result<String, RunError> {
        let metadata = fs::metadata(path)
            .map_err(|err| RunError::Internal(format!("reading transcript `{}` metadata: {err}", path.display())))?;
        if metadata.len() > MAX_TRANSCRIPT_BYTES {
            return Err(RunError::Internal(format!(
                "transcript `{}` exceeds {MAX_TRANSCRIPT_BYTES} bytes",
                path.display()
            )));
        }
        fs::read_to_string(path)
            .map_err(|err| RunError::Internal(format!("reading transcript `{}`: {err}", path.display())))
    }

    fn prepare_scratch(
        &mut self,
        in_place: bool,
        backend: crunch_store::StoreBackend,
        state_dir: &Path,
    ) -> Result<TranscriptScratch, RunError> {
        TranscriptScratch::new(in_place, backend, state_dir)
    }

    fn execute(
        &mut self,
        transcript: &Transcript,
        mantle_bin: &Path,
        scratch: &TranscriptScratch,
        work_dir: &Path,
    ) -> TranscriptExecution {
        execute_transcript(transcript, mantle_bin, scratch, work_dir)
    }

    fn publish(&mut self, path: &Path, evidence: TranscriptEvidence) -> Result<blake3::Hash, RunError> {
        write_evidence(path, evidence)
    }

    fn readback(&mut self, path: &Path) -> Result<blake3::Hash, RunError> {
        let bytes = fs::read(path).map_err(|err| {
            RunError::Internal(format!("reading transcript output artifact `{}`: {err}", path.display()))
        })?;
        Ok(blake3::hash(&bytes))
    }
}

#[derive(Debug, Clone)]
pub struct TranscriptRunOptions<'a> {
    pub transcript: PathBuf,
    pub output: Option<PathBuf>,
    pub mantle_bin: Option<PathBuf>,
    pub backend: crunch_store::StoreBackend,
    pub allow_in_place: bool,
    pub state_dir: &'a Path,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TranscriptOptions {
    in_place: bool,
    reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BlockKind {
    Mantle,
    MantleError,
    Expect,
    ExpectJson,
    SetupHide,
    CleanupHide,
    Options,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Block {
    kind: BlockKind,
    body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Transcript {
    options: TranscriptOptions,
    blocks: Vec<Block>,
}

#[derive(Debug)]
struct CommandRun {
    visible_index: usize,
    command: String,
    expected_failure: bool,
    exit_code: Option<i32>,
    raw_output: String,
    normalized_output: String,
}

#[derive(Debug)]
struct TranscriptRunSummary {
    visible_steps: usize,
    output: PathBuf,
}

#[derive(Debug, Serialize)]
struct TranscriptEvidence {
    schema: &'static str,
    transcript: String,
    in_place: bool,
    visible_steps: usize,
    hidden_setups: Vec<HiddenRunEvidence>,
    hidden_cleanups: Vec<HiddenRunEvidence>,
    runs: Vec<RunEvidence>,
    cleanup_failures: Vec<String>,
}

#[derive(Debug, Clone)]
struct HiddenRun {
    label: &'static str,
    raw_output: String,
    normalized_output: String,
}

#[derive(Debug, Serialize)]
struct HiddenRunEvidence {
    label: &'static str,
    raw_output: String,
    normalized_output: String,
}

#[derive(Debug, Serialize)]
struct RunEvidence {
    step: usize,
    command: String,
    expected_failure: bool,
    exit_code: Option<i32>,
    raw_output: String,
    normalized_output: String,
}

pub fn cmd_transcript_run(options: TranscriptRunOptions<'_>) -> Result<(), RunError> {
    let summary = run_transcript(options)?;
    println!("transcript ok: {} visible step(s); output: {}", summary.visible_steps, summary.output.display());
    Ok(())
}

fn run_transcript(options: TranscriptRunOptions<'_>) -> Result<TranscriptRunSummary, RunError> {
    let plan = transcript_effect_plan()?;
    let mut observed = [
        transcript_observation(
            TRANSCRIPT_INPUT_EFFECT,
            EffectKind::ReadFiles,
            ObservationStatus::Skipped,
            0,
            EffectOutput::None,
        ),
        transcript_observation(
            TRANSCRIPT_WRITE_EFFECT,
            EffectKind::WriteFiles,
            ObservationStatus::Skipped,
            0,
            EffectOutput::None,
        ),
        transcript_observation(
            TRANSCRIPT_PROCESS_EFFECT,
            EffectKind::RunProcess,
            ObservationStatus::Skipped,
            0,
            EffectOutput::None,
        ),
        transcript_observation(
            TRANSCRIPT_READBACK_EFFECT,
            EffectKind::ReadFiles,
            ObservationStatus::Skipped,
            0,
            EffectOutput::None,
        ),
    ];
    let mut port = LocalTranscriptPort;
    let result = run_transcript_with_port(options, &mut port, &mut observed);
    match (classify_observations(&plan, &observed), result) {
        (ApplicationOutcome::Completed, Ok(summary)) => Ok(summary),
        (ApplicationOutcome::Failed { .. }, Err(error)) | (ApplicationOutcome::Contradicted { .. }, Err(error)) => {
            Err(error)
        }
        (outcome, _) => Err(RunError::Internal(format!("transcript effects inconsistent: {outcome:?}"))),
    }
}

fn transcript_effect_plan() -> Result<EffectPlan, RunError> {
    plan_effects(CommandFamily::Evaluation, &[
        EffectSpec {
            effect_id: TRANSCRIPT_INPUT_EFFECT,
            kind: EffectKind::ReadFiles,
            limit: EffectMeasure::Calls(1),
            expected_output: ExpectedOutput::None,
        },
        EffectSpec {
            effect_id: TRANSCRIPT_WRITE_EFFECT,
            kind: EffectKind::WriteFiles,
            limit: EffectMeasure::Calls(2),
            expected_output: ExpectedOutput::None,
        },
        EffectSpec {
            effect_id: TRANSCRIPT_PROCESS_EFFECT,
            kind: EffectKind::RunProcess,
            limit: EffectMeasure::Calls(MAX_TRANSCRIPT_BLOCKS as u32),
            expected_output: ExpectedOutput::None,
        },
        EffectSpec {
            effect_id: TRANSCRIPT_READBACK_EFFECT,
            kind: EffectKind::ReadFiles,
            limit: EffectMeasure::Calls(1),
            expected_output: ExpectedOutput::Identity(TRANSCRIPT_EVIDENCE_SCHEMA),
        },
    ])
    .map_err(|error| RunError::Internal(format!("planning transcript effects: {}", error.code())))
}

fn transcript_observation(
    id: &str,
    kind: EffectKind,
    status: ObservationStatus,
    calls: u32,
    output: EffectOutput,
) -> Observation {
    Observation {
        effect_id: EffectId(id.to_string()),
        kind,
        status,
        output,
        usage: EffectMeasure::Calls(calls),
        diagnostics_code: (status == ObservationStatus::Failed).then(|| format!("{id}-failed")),
    }
}

fn run_transcript_with_port(
    options: TranscriptRunOptions<'_>,
    port: &mut impl TranscriptPort,
    observed: &mut [Observation; 4],
) -> Result<TranscriptRunSummary, RunError> {
    let text = port.read_input(&options.transcript);
    observed[0].status = if text.is_ok() {
        ObservationStatus::Succeeded
    } else {
        ObservationStatus::Failed
    };
    observed[0].usage = EffectMeasure::Calls(1);
    observed[0].diagnostics_code = text.is_err().then(|| format!("{TRANSCRIPT_INPUT_EFFECT}-failed"));
    let transcript = parse_transcript(&text?)?;
    if transcript.options.in_place && !options.allow_in_place {
        return Err(RunError::Internal(
            "transcript requests in_place=true; rerun with --allow-in-place to execute it".to_string(),
        ));
    }
    debug_assert!(!transcript.blocks.is_empty());
    debug_assert!(transcript.blocks.len() <= MAX_TRANSCRIPT_BLOCKS);
    let output_path = options.output.unwrap_or_else(|| default_output_path(&options.transcript));
    let work_dir = transcript_work_dir(&options.transcript);
    let mantle_bin = options.mantle_bin.unwrap_or_else(default_mantle_bin);
    let scratch = port.prepare_scratch(transcript.options.in_place, options.backend, options.state_dir);
    observed[1].status = if scratch.is_ok() {
        ObservationStatus::Succeeded
    } else {
        ObservationStatus::Failed
    };
    observed[1].usage = EffectMeasure::Calls(1);
    observed[1].diagnostics_code = scratch.is_err().then(|| format!("{TRANSCRIPT_WRITE_EFFECT}-failed"));
    let scratch = scratch?;
    let execution = port.execute(&transcript, &mantle_bin, &scratch, &work_dir);
    observed[2].status = if execution.result.is_ok() {
        ObservationStatus::Succeeded
    } else {
        ObservationStatus::Failed
    };
    observed[2].usage = EffectMeasure::Calls(execution.attempted_processes);
    observed[2].diagnostics_code = execution.result.is_err().then(|| format!("{TRANSCRIPT_PROCESS_EFFECT}-failed"));
    debug_assert!(execution.result.is_err() || !execution.visible_runs.is_empty());
    debug_assert!(execution.visible_runs.len() <= transcript.blocks.len());
    let visible_steps = execution.visible_runs.len();
    let published = port.publish(&output_path, TranscriptEvidence {
        schema: TRANSCRIPT_EVIDENCE_SCHEMA,
        transcript: options.transcript.display().to_string(),
        in_place: transcript.options.in_place,
        visible_steps,
        hidden_setups: execution.hidden_setups.iter().map(HiddenRunEvidence::from).collect(),
        hidden_cleanups: execution.hidden_cleanups.iter().map(HiddenRunEvidence::from).collect(),
        runs: execution.visible_runs.iter().map(RunEvidence::from).collect(),
        cleanup_failures: execution.cleanup_failures,
    });
    observed[1].status = if published.is_ok() {
        ObservationStatus::Succeeded
    } else {
        ObservationStatus::Failed
    };
    observed[1].usage = EffectMeasure::Calls(2);
    observed[1].diagnostics_code = published.is_err().then(|| format!("{TRANSCRIPT_WRITE_EFFECT}-failed"));
    let expected_digest = published?;
    let persisted = port.readback(&output_path);
    observed[3].usage = EffectMeasure::Calls(1);
    match persisted {
        Ok(actual_digest) => {
            let matches = actual_digest == expected_digest;
            observed[3].output = if matches {
                EffectOutput::Identity(TRANSCRIPT_EVIDENCE_SCHEMA.to_string())
            } else {
                EffectOutput::None
            };
            observed[3].status = if matches {
                ObservationStatus::Succeeded
            } else {
                ObservationStatus::Failed
            };
            observed[3].diagnostics_code = (!matches).then(|| format!("{TRANSCRIPT_READBACK_EFFECT}-failed"));
            if !matches {
                return Err(RunError::Internal(format!(
                    "transcript output artifact `{}` does not match observed runs",
                    output_path.display()
                )));
            }
        }
        Err(error) => {
            observed[3].status = ObservationStatus::Failed;
            observed[3].diagnostics_code = Some(format!("{TRANSCRIPT_READBACK_EFFECT}-failed"));
            return Err(error);
        }
    }
    execution.result?;
    Ok(TranscriptRunSummary {
        visible_steps,
        output: output_path,
    })
}

fn transcript_work_dir(transcript_path: &Path) -> PathBuf {
    transcript_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}

struct TranscriptExecution {
    visible_runs: Vec<CommandRun>,
    hidden_setups: Vec<HiddenRun>,
    hidden_cleanups: Vec<HiddenRun>,
    cleanup_failures: Vec<String>,
    attempted_processes: u32,
    result: Result<(), RunError>,
}

struct TranscriptBlockContext<'a> {
    transcript: &'a Transcript,
    mantle_bin: &'a Path,
    scratch: &'a TranscriptScratch,
    work_dir: &'a Path,
}

struct VisibleBlockContext<'a> {
    block: &'a Block,
    is_in_place: bool,
    mantle_bin: &'a Path,
    scratch: &'a TranscriptScratch,
    work_dir: &'a Path,
}

struct TranscriptExecutionState {
    visible_runs: Vec<CommandRun>,
    is_last_error_pending: bool,
    pending_setups: Vec<String>,
    cleanups: Vec<String>,
    hidden_setups: Vec<HiddenRun>,
    result: Result<(), RunError>,
}

fn execute_transcript(
    transcript: &Transcript,
    mantle_bin: &Path,
    scratch: &TranscriptScratch,
    work_dir: &Path,
) -> TranscriptExecution {
    let block_count_max = transcript.blocks.len();
    let mut attempted_processes = 0;
    let mut state = TranscriptExecutionState {
        visible_runs: Vec::with_capacity(block_count_max),
        is_last_error_pending: false,
        pending_setups: Vec::with_capacity(block_count_max),
        cleanups: Vec::with_capacity(block_count_max),
        hidden_setups: Vec::with_capacity(block_count_max),
        result: Ok(()),
    };
    let block_context = TranscriptBlockContext {
        transcript,
        mantle_bin,
        scratch,
        work_dir,
    };
    for block in &transcript.blocks {
        if state.result.is_err() {
            break;
        }
        if let Err(error) = execute_transcript_block(block, &block_context, &mut state, &mut attempted_processes) {
            state.result = Err(error);
        }
    }
    validate_transcript_completion(&mut state);
    debug_assert!(state.visible_runs.len() <= transcript.blocks.len());
    debug_assert!(state.hidden_setups.len() <= transcript.blocks.len());
    let (hidden_cleanups, cleanup_failures) =
        run_transcript_cleanups(state.cleanups, scratch, work_dir, &mut attempted_processes);
    if state.result.is_ok() && !cleanup_failures.is_empty() {
        state.result =
            Err(RunError::Build(format!("hidden transcript cleanup failed: {}", cleanup_failures.join("; "))));
    }
    TranscriptExecution {
        visible_runs: state.visible_runs,
        hidden_setups: state.hidden_setups,
        hidden_cleanups,
        cleanup_failures,
        attempted_processes,
        result: state.result,
    }
}

fn execute_transcript_block(
    block: &Block,
    context: &TranscriptBlockContext<'_>,
    state: &mut TranscriptExecutionState,
    attempted_processes: &mut u32,
) -> Result<(), RunError> {
    match block.kind {
        BlockKind::Options => Ok(()),
        BlockKind::SetupHide => {
            state.pending_setups.push(block.body.clone());
            Ok(())
        }
        BlockKind::CleanupHide => {
            state.cleanups.push(block.body.clone());
            Ok(())
        }
        BlockKind::Expect | BlockKind::ExpectJson => {
            let run = state.visible_runs.last().ok_or_else(|| {
                RunError::Internal("transcript expect block must follow a mantle command".to_string())
            })?;
            verify_expectation(run, block)?;
            state.is_last_error_pending = false;
            Ok(())
        }
        BlockKind::Mantle | BlockKind::MantleError => execute_visible_block(
            VisibleBlockContext {
                block,
                is_in_place: context.transcript.options.in_place,
                mantle_bin: context.mantle_bin,
                scratch: context.scratch,
                work_dir: context.work_dir,
            },
            state,
            attempted_processes,
        ),
    }
}

fn execute_visible_block(
    context: VisibleBlockContext<'_>,
    state: &mut TranscriptExecutionState,
    attempted_processes: &mut u32,
) -> Result<(), RunError> {
    let VisibleBlockContext {
        block,
        is_in_place,
        mantle_bin,
        scratch,
        work_dir,
    } = context;
    if state.is_last_error_pending {
        return Err(RunError::Internal(
            "mantle:error block must be followed by an expect or expect:json block".to_string(),
        ));
    }
    for setup in state.pending_setups.drain(..) {
        state.hidden_setups.push(run_hidden_shell(&setup, scratch, work_dir, "setup", attempted_processes)?);
    }
    let visible_index = state
        .visible_runs
        .len()
        .checked_add(1)
        .ok_or_else(|| RunError::Internal("transcript visible step index overflowed usize".to_string()))?;
    let run = run_visible_mantle(
        VisibleRunRequest {
            mantle_bin,
            command: &block.body,
            is_expected_failure: matches!(block.kind, BlockKind::MantleError),
            scratch,
            work_dir,
            is_in_place,
            visible_index,
        },
        attempted_processes,
    )?;
    state.is_last_error_pending = run.expected_failure;
    state.visible_runs.push(run);
    debug_assert!(!state.visible_runs.is_empty());
    Ok(())
}

fn validate_transcript_completion(state: &mut TranscriptExecutionState) {
    if state.result.is_ok() && state.is_last_error_pending {
        state.result = Err(RunError::Internal(
            "mantle:error block must be followed by an expect or expect:json block".to_string(),
        ));
    }
    if state.result.is_ok() && !state.pending_setups.is_empty() {
        state.result =
            Err(RunError::Internal("transcript has setup:hide block without a following mantle command".to_string()));
    }
}

fn run_transcript_cleanups(
    cleanups: Vec<String>,
    scratch: &TranscriptScratch,
    work_dir: &Path,
    attempted_processes: &mut u32,
) -> (Vec<HiddenRun>, Vec<String>) {
    let cleanup_count_max = cleanups.len();
    let mut hidden_cleanups = Vec::with_capacity(cleanup_count_max);
    let mut cleanup_failures = Vec::with_capacity(cleanup_count_max);
    for cleanup in cleanups {
        match run_hidden_shell(&cleanup, scratch, work_dir, "cleanup", attempted_processes) {
            Ok(hidden) => hidden_cleanups.push(hidden),
            Err(error) => cleanup_failures.push(error.to_string()),
        }
    }
    debug_assert!(hidden_cleanups.len() <= cleanup_count_max);
    debug_assert!(cleanup_failures.len() <= cleanup_count_max);
    (hidden_cleanups, cleanup_failures)
}

impl From<&CommandRun> for RunEvidence {
    fn from(run: &CommandRun) -> Self {
        Self {
            step: run.visible_index,
            command: run.command.clone(),
            expected_failure: run.expected_failure,
            exit_code: run.exit_code,
            raw_output: run.raw_output.clone(),
            normalized_output: run.normalized_output.clone(),
        }
    }
}

impl From<&HiddenRun> for HiddenRunEvidence {
    fn from(run: &HiddenRun) -> Self {
        Self {
            label: run.label,
            raw_output: run.raw_output.clone(),
            normalized_output: run.normalized_output.clone(),
        }
    }
}

fn default_output_path(transcript: &Path) -> PathBuf {
    let mut path = transcript.as_os_str().to_os_string();
    path.push(".output.json");
    PathBuf::from(path)
}

fn write_evidence(path: &Path, evidence: TranscriptEvidence) -> Result<blake3::Hash, RunError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            RunError::Internal(format!("creating transcript output directory `{}`: {err}", parent.display()))
        })?;
    }
    let bytes = serde_json::to_vec_pretty(&evidence)
        .map_err(|err| RunError::Internal(format!("serializing transcript output artifact: {err}")))?;
    let digest = blake3::hash(&bytes);
    fs::write(path, &bytes)
        .map_err(|err| RunError::Internal(format!("writing transcript output artifact `{}`: {err}", path.display())))?;
    Ok(digest)
}

fn default_mantle_bin() -> PathBuf {
    std::env::current_exe().unwrap_or_else(|_| PathBuf::from("mantle"))
}

struct TranscriptScratch {
    tmp: TempDir,
    store: PathBuf,
    state_dir: PathBuf,
    backend: crunch_store::StoreBackend,
}

impl TranscriptScratch {
    fn new(in_place: bool, backend: crunch_store::StoreBackend, admitted_state_dir: &Path) -> Result<Self, RunError> {
        let tmp = tempfile::Builder::new()
            .prefix("mantle-transcript-")
            .tempdir()
            .map_err(|err| RunError::Internal(format!("creating transcript tempdir: {err}")))?;
        let store = if in_place {
            PathBuf::from("/mantle/store")
        } else {
            tmp.path().join("store")
        };
        let state_dir = if in_place {
            admitted_state_dir.to_path_buf()
        } else {
            tmp.path().join("state")
        };
        fs::create_dir_all(&store).map_err(|err| RunError::Internal(format!("creating transcript store: {err}")))?;
        fs::create_dir_all(&state_dir)
            .map_err(|err| RunError::Internal(format!("creating transcript state-dir: {err}")))?;
        Ok(Self {
            tmp,
            store,
            state_dir,
            backend,
        })
    }

    fn tmp_path(&self) -> &Path {
        self.tmp.path()
    }
}

fn run_hidden_shell(
    script: &str,
    scratch: &TranscriptScratch,
    work_dir: &Path,
    label: &'static str,
    attempted_processes: &mut u32,
) -> Result<HiddenRun, RunError> {
    debug_assert!(label == "setup" || label == "cleanup");
    debug_assert!(!work_dir.as_os_str().is_empty());
    let output = Command::new("/bin/sh")
        .arg("-c")
        .arg(expand_temp(script, scratch))
        .current_dir(work_dir)
        .env(TRANSCRIPT_TMP_ENV, scratch.tmp_path())
        .output();
    *attempted_processes += 1;
    let output = output.map_err(|err| RunError::Internal(format!("running hidden transcript {label}: {err}")))?;
    let mut combined = String::new();
    combined.push_str(&String::from_utf8_lossy(&output.stdout));
    combined.push_str(&String::from_utf8_lossy(&output.stderr));
    if output.status.success() {
        return Ok(HiddenRun {
            label,
            normalized_output: normalize_output(&combined, scratch),
            raw_output: combined,
        });
    }
    Err(RunError::Build(format!(
        "hidden transcript {label} failed with status {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )))
}

struct VisibleRunRequest<'a> {
    mantle_bin: &'a Path,
    command: &'a str,
    is_expected_failure: bool,
    scratch: &'a TranscriptScratch,
    work_dir: &'a Path,
    is_in_place: bool,
    visible_index: usize,
}

fn run_visible_mantle(request: VisibleRunRequest<'_>, attempted_processes: &mut u32) -> Result<CommandRun, RunError> {
    let tokens = split_command(request.command)?;
    if tokens.first().map(String::as_str) != Some(MANTLE_PREFIX) {
        return Err(RunError::Internal(format!(
            "visible transcript command must start with `mantle`: `{}`",
            request.command
        )));
    }
    debug_assert_eq!(tokens.first().map(String::as_str), Some(MANTLE_PREFIX));
    debug_assert!(request.visible_index > 0);

    let mut cmd = Command::new(request.mantle_bin);
    if !request.is_in_place {
        if !tokens.iter().any(|token| token == "--store" || token.starts_with("--store=")) {
            cmd.arg("--store").arg(&request.scratch.store);
        }
        if !tokens.iter().any(|token| token == "--state-dir" || token.starts_with("--state-dir=")) {
            cmd.arg("--state-dir").arg(&request.scratch.state_dir);
        }
    }
    if !tokens.iter().any(|token| token == "--store-backend" || token.starts_with("--store-backend=")) {
        cmd.arg("--store-backend").arg(request.scratch.backend.as_str());
    }
    for token in tokens.iter().skip(1) {
        cmd.arg(expand_temp(token, request.scratch));
    }
    cmd.current_dir(request.work_dir).env(TRANSCRIPT_TMP_ENV, request.scratch.tmp_path());
    let output = cmd.output();
    *attempted_processes += 1;
    let output = output.map_err(|err| {
        RunError::Internal(format!(
            "running transcript command `{}` via `{}`: {err}",
            request.command,
            request.mantle_bin.display()
        ))
    })?;
    let mut combined = String::new();
    combined.push_str(&String::from_utf8_lossy(&output.stdout));
    combined.push_str(&String::from_utf8_lossy(&output.stderr));
    if request.is_expected_failure && output.status.success() {
        return Err(RunError::Build(format!("transcript command `{}` unexpectedly succeeded", request.command)));
    }
    if !request.is_expected_failure && !output.status.success() {
        return Err(RunError::Build(format!(
            "transcript command `{}` failed with status {:?}\n{}",
            request.command,
            output.status.code(),
            combined
        )));
    }
    Ok(CommandRun {
        visible_index: request.visible_index,
        command: request.command.to_string(),
        expected_failure: request.is_expected_failure,
        exit_code: output.status.code(),
        normalized_output: normalize_output(&combined, request.scratch),
        raw_output: combined,
    })
}

fn verify_expectation(run: &CommandRun, expectation: &Block) -> Result<(), RunError> {
    match expectation.kind {
        BlockKind::Expect => verify_text_expectation(run, &expectation.body),
        BlockKind::ExpectJson => verify_json_expectation(run, &expectation.body),
        BlockKind::Mantle
        | BlockKind::MantleError
        | BlockKind::SetupHide
        | BlockKind::CleanupHide
        | BlockKind::Options => Err(RunError::Internal("only expectation blocks may be verified".to_string())),
    }
}

fn verify_text_expectation(run: &CommandRun, body: &str) -> Result<(), RunError> {
    let mut search_from = 0;
    let mut is_expectation_seen = false;
    for expected in expectation_lines(body) {
        is_expectation_seen = true;
        let Some(relative_index) = run.normalized_output[search_from..].find(expected) else {
            return Err(RunError::Build(format!(
                "transcript step {} output did not contain expected fragment `{}` for command `{}`\nnormalized output:\n{}",
                run.visible_index, expected, run.command, run.normalized_output
            )));
        };
        search_from = search_from
            .checked_add(relative_index)
            .and_then(|offset| offset.checked_add(expected.len()))
            .ok_or_else(|| RunError::Internal("transcript expectation search offset overflowed usize".to_string()))?;
    }
    if !is_expectation_seen {
        return Err(RunError::Internal("expect block must contain at least one expected fragment".to_string()));
    }
    debug_assert!(is_expectation_seen);
    debug_assert!(search_from <= run.normalized_output.len());
    Ok(())
}

fn verify_json_expectation(run: &CommandRun, body: &str) -> Result<(), RunError> {
    debug_assert!(run.visible_index > 0);
    debug_assert!(!run.command.is_empty());
    let json_source = json_output_slice(&run.raw_output);
    let value: serde_json::Value = serde_json::from_str(json_source).map_err(|err| {
        RunError::Build(format!(
            "transcript step {} expected JSON output for `{}`: {err}",
            run.visible_index, run.command
        ))
    })?;
    let mut is_expectation_seen = false;
    for line in expectation_lines(body) {
        is_expectation_seen = true;
        let Some((path, expected)) = line.split_once('=') else {
            return Err(RunError::Internal(format!("expect:json line must use dotted.path = value syntax: `{line}`")));
        };
        let path = path.trim();
        let expected = expected.trim().trim_matches('"');
        let actual = json_path_string(&value, path).ok_or_else(|| {
            RunError::Build(format!("transcript step {} JSON output missing path `{path}`", run.visible_index))
        })?;
        if actual != expected {
            return Err(RunError::Build(format!(
                "transcript step {} JSON path `{path}` was `{actual}`, expected `{expected}`",
                run.visible_index
            )));
        }
    }
    if !is_expectation_seen {
        return Err(RunError::Internal("expect:json block must contain at least one expected field".to_string()));
    }
    Ok(())
}

fn json_output_slice(output: &str) -> &str {
    let trimmed = output.trim();
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        return trimmed;
    }
    trimmed
        .lines()
        .rev()
        .map(str::trim)
        .find(|line| line.starts_with('{') || line.starts_with('['))
        .unwrap_or(trimmed)
}

fn json_path_string(value: &serde_json::Value, path: &str) -> Option<String> {
    let mut current = value;
    for part in path.split('.') {
        current = current.get(part)?;
    }
    match current {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Bool(b) => Some(b.to_string()),
        serde_json::Value::Number(n) => Some(n.to_string()),
        serde_json::Value::Null => Some("null".to_string()),
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => Some(current.to_string()),
    }
}

fn expectation_lines(body: &str) -> impl Iterator<Item = &str> {
    body.lines().map(str::trim).filter(|line| !line.is_empty() && !line.starts_with('#'))
}

fn normalize_output(output: &str, scratch: &TranscriptScratch) -> String {
    let temp = scratch.tmp_path().display().to_string();
    let store = scratch.store.display().to_string();
    let state = scratch.state_dir.display().to_string();
    let normalized = output
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace(&store, "${MANTLE_TRANSCRIPT_STORE}")
        .replace(&state, "${MANTLE_TRANSCRIPT_STATE}")
        .replace(&temp, "${MANTLE_TRANSCRIPT_TMP}");
    debug_assert!(!normalized.contains('\r'));
    debug_assert!(!normalized.contains(&temp));
    let mut out = String::with_capacity(normalized.len());
    let mut is_previous_line_blank = false;
    for line in normalized.lines() {
        let trimmed = line.trim_end();
        let is_blank = trimmed.is_empty();
        if is_blank && is_previous_line_blank {
            continue;
        }
        out.push_str(trimmed);
        out.push('\n');
        is_previous_line_blank = is_blank;
    }
    out.trim_end().to_string()
}

fn expand_temp(input: &str, scratch: &TranscriptScratch) -> String {
    input.replace(TEMP_PLACEHOLDER, &scratch.tmp_path().display().to_string())
}

fn split_command(command: &str) -> Result<Vec<String>, RunError> {
    let tokens = command.split_whitespace().map(str::to_string).collect::<Vec<_>>();
    if tokens.is_empty() {
        return Err(RunError::Internal("empty transcript command block".to_string()));
    }
    Ok(tokens)
}

fn parse_transcript(text: &str) -> Result<Transcript, RunError> {
    let blocks = parse_blocks(text)?;
    let mut options = TranscriptOptions {
        in_place: false,
        reason: None,
    };
    for block in &blocks {
        if block.kind == BlockKind::Options {
            options = parse_options(&block.body)?;
        }
    }
    if !blocks.iter().any(|block| matches!(block.kind, BlockKind::Mantle | BlockKind::MantleError)) {
        return Err(RunError::Internal("transcript has no mantle command block".to_string()));
    }
    Ok(Transcript { options, blocks })
}

fn parse_blocks(text: &str) -> Result<Vec<Block>, RunError> {
    let lines = text.lines().take(MAX_TRANSCRIPT_LINES.saturating_add(1)).collect::<Vec<_>>();
    if lines.len() > MAX_TRANSCRIPT_LINES {
        return Err(RunError::Internal(format!("transcript exceeds {MAX_TRANSCRIPT_LINES} lines")));
    }
    let mut blocks = Vec::with_capacity(lines.len().min(MAX_TRANSCRIPT_BLOCKS));
    let mut open_block: Option<(String, String)> = None;
    for line in lines {
        if open_block.is_some() {
            if line.starts_with("```") {
                close_transcript_block(&mut open_block, &mut blocks)?;
            } else if let Some((_info, body)) = open_block.as_mut() {
                body.push_str(line);
                body.push('\n');
            }
            continue;
        }
        if !line.starts_with("```") {
            continue;
        }
        let info = line.trim_start_matches("```").trim();
        if !info.is_empty() {
            open_block = Some((info.to_string(), String::new()));
        }
    }
    if let Some((info, _body)) = open_block {
        return Err(RunError::Internal(format!("transcript block `{info}` is missing closing fence")));
    }
    debug_assert!(blocks.len() <= MAX_TRANSCRIPT_BLOCKS);
    debug_assert!(blocks.capacity() >= blocks.len());
    Ok(blocks)
}

fn close_transcript_block(open_block: &mut Option<(String, String)>, blocks: &mut Vec<Block>) -> Result<(), RunError> {
    let (closed_info, closed_body) = open_block
        .take()
        .ok_or_else(|| RunError::Internal("transcript close requested without an open block".to_string()))?;
    let Some(kind) = parse_block_kind(&closed_info)? else {
        return Ok(());
    };
    if blocks.len() >= MAX_TRANSCRIPT_BLOCKS {
        return Err(RunError::Internal(format!("transcript exceeds {MAX_TRANSCRIPT_BLOCKS} executable blocks")));
    }
    blocks.push(Block {
        kind,
        body: closed_body,
    });
    debug_assert!(blocks.len() <= MAX_TRANSCRIPT_BLOCKS);
    debug_assert!(open_block.is_none());
    Ok(())
}

fn parse_block_kind(info: &str) -> Result<Option<BlockKind>, RunError> {
    match info {
        "mantle" => Ok(Some(BlockKind::Mantle)),
        "mantle:error" => Ok(Some(BlockKind::MantleError)),
        "expect" => Ok(Some(BlockKind::Expect)),
        "expect:json" => Ok(Some(BlockKind::ExpectJson)),
        "setup:hide" => Ok(Some(BlockKind::SetupHide)),
        "cleanup:hide" => Ok(Some(BlockKind::CleanupHide)),
        "transcript:options" => Ok(Some(BlockKind::Options)),
        _ if info.starts_with("mantle:") || info.starts_with("expect:") || info.ends_with(":hide") => {
            Err(RunError::Internal(format!("unknown transcript block `{info}`")))
        }
        _ => Ok(None),
    }
}

#[derive(Deserialize)]
struct RawOptions {
    in_place: Option<bool>,
    reason: Option<String>,
}

fn parse_options(body: &str) -> Result<TranscriptOptions, RunError> {
    let raw: RawOptions = serde_json::from_str(body).map_err(|err| {
        RunError::Internal(format!("transcript:options must be a JSON object with in_place/reason fields: {err}"))
    })?;
    Ok(TranscriptOptions {
        in_place: raw.in_place.unwrap_or(false),
        reason: raw.reason,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_rejects_unknown_closed_vocabulary() {
        let err = parse_transcript("```mantle:typo\nmantle doctor\n```\n").unwrap_err();
        assert!(err.message().contains("unknown transcript block `mantle:typo`"));
    }

    #[test]
    fn parser_ignores_non_transcript_fenced_blocks_as_prose() {
        let transcript = parse_transcript(
            r#"```sh
echo prose example
```
```mantle
mantle doctor
```
"#,
        )
        .unwrap();
        assert_eq!(transcript.blocks.len(), 1);
    }

    #[test]
    fn parser_reads_options_and_command_blocks() {
        let transcript = parse_transcript(
            r#"```transcript:options
{"in_place": true, "reason": "operator repro"}
```
```mantle:error
mantle eval missing.ncl
```
```expect
missing.ncl
```
"#,
        )
        .unwrap();
        assert!(transcript.options.in_place);
        assert_eq!(transcript.options.reason.as_deref(), Some("operator repro"));
        assert_eq!(transcript.blocks.len(), 3);
    }

    #[test]
    fn output_normalization_replaces_isolated_paths() {
        let scratch = TranscriptScratch::new(false, crunch_store::StoreBackend::Snix, Path::new("unused")).unwrap();
        let raw = format!(
            "{}\r\n{}\n{}\n\n\n",
            scratch.tmp_path().display(),
            scratch.store.display(),
            scratch.state_dir.display()
        );
        let normalized = normalize_output(&raw, &scratch);
        assert!(normalized.contains("${MANTLE_TRANSCRIPT_TMP}"));
        assert!(normalized.contains("${MANTLE_TRANSCRIPT_STORE}"));
        assert!(normalized.contains("${MANTLE_TRANSCRIPT_STATE}"));
        assert!(!normalized.contains("\r"));
        assert!(!normalized.contains("\n\n"));
    }
}
