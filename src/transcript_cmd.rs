// machine-artifact-public: transcript.evidence-reports
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use serde::Deserialize;
use serde::Serialize;
use tempfile::TempDir;

use crate::errors::RunError;

const TRANSCRIPT_TMP_ENV: &str = "MANTLE_TRANSCRIPT_TMP";
const MANTLE_PREFIX: &str = "mantle";
const TEMP_PLACEHOLDER: &str = "${MANTLE_TRANSCRIPT_TMP}";

#[derive(Debug, Clone)]
pub struct TranscriptRunOptions {
    pub transcript: PathBuf,
    pub output: Option<PathBuf>,
    pub mantle_bin: Option<PathBuf>,
    pub allow_in_place: bool,
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

#[derive(Debug, Clone)]
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

pub fn cmd_transcript_run(options: TranscriptRunOptions) -> Result<(), RunError> {
    let summary = run_transcript(options)?;
    println!("transcript ok: {} visible step(s); output: {}", summary.visible_steps, summary.output.display());
    Ok(())
}

fn run_transcript(options: TranscriptRunOptions) -> Result<TranscriptRunSummary, RunError> {
    let text = fs::read_to_string(&options.transcript)
        .map_err(|err| RunError::Internal(format!("reading transcript `{}`: {err}", options.transcript.display())))?;
    let transcript = parse_transcript(&text)?;
    if transcript.options.in_place && !options.allow_in_place {
        return Err(RunError::Internal(
            "transcript requests in_place=true; rerun with --allow-in-place to execute it".to_string(),
        ));
    }

    let output_path = options.output.unwrap_or_else(|| default_output_path(&options.transcript));
    let work_dir = options
        .transcript
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    let mantle_bin = options.mantle_bin.unwrap_or_else(default_mantle_bin);
    let scratch = TranscriptScratch::new(transcript.options.in_place)?;
    let mut visible_runs = Vec::new();
    let mut last_run: Option<CommandRun> = None;
    let mut last_error_needs_expectation = false;
    let mut pending_setups = Vec::<String>::new();
    let mut cleanups = Vec::<String>::new();
    let mut hidden_setups = Vec::<HiddenRun>::new();
    let mut hidden_cleanups = Vec::<HiddenRun>::new();
    let mut result: Result<(), RunError> = Ok(());

    for block in &transcript.blocks {
        if result.is_err() {
            break;
        }
        match block.kind {
            BlockKind::Options => {}
            BlockKind::SetupHide => pending_setups.push(block.body.clone()),
            BlockKind::CleanupHide => cleanups.push(block.body.clone()),
            BlockKind::Expect | BlockKind::ExpectJson => {
                let run = match last_run.as_ref() {
                    Some(run) => run,
                    None => {
                        result =
                            Err(RunError::Internal("transcript expect block must follow a mantle command".to_string()));
                        continue;
                    }
                };
                result = verify_expectation(run, block);
                if result.is_ok() {
                    last_error_needs_expectation = false;
                }
            }
            BlockKind::Mantle | BlockKind::MantleError => {
                if last_error_needs_expectation {
                    result = Err(RunError::Internal(
                        "mantle:error block must be followed by an expect or expect:json block".to_string(),
                    ));
                    continue;
                }
                for setup in pending_setups.drain(..) {
                    match run_hidden_shell(&setup, &scratch, &work_dir, "setup") {
                        Ok(hidden) => hidden_setups.push(hidden),
                        Err(err) => {
                            result = Err(err);
                            break;
                        }
                    }
                }
                if result.is_err() {
                    continue;
                }
                let expect_failure = matches!(block.kind, BlockKind::MantleError);
                match run_visible_mantle(
                    &mantle_bin,
                    &block.body,
                    expect_failure,
                    &scratch,
                    &work_dir,
                    transcript.options.in_place,
                    visible_runs.len() + 1,
                ) {
                    Ok(run) => {
                        last_error_needs_expectation = run.expected_failure;
                        last_run = Some(run.clone());
                        visible_runs.push(run);
                    }
                    Err(err) => result = Err(err),
                }
            }
        }
    }

    if result.is_ok() && last_error_needs_expectation {
        result = Err(RunError::Internal(
            "mantle:error block must be followed by an expect or expect:json block".to_string(),
        ));
    }

    if result.is_ok() && !pending_setups.is_empty() {
        result =
            Err(RunError::Internal("transcript has setup:hide block without a following mantle command".to_string()));
    }

    let mut cleanup_failures = Vec::new();
    for cleanup in cleanups {
        match run_hidden_shell(&cleanup, &scratch, &work_dir, "cleanup") {
            Ok(hidden) => hidden_cleanups.push(hidden),
            Err(err) => cleanup_failures.push(err.to_string()),
        }
    }
    if result.is_ok() && !cleanup_failures.is_empty() {
        result = Err(RunError::Build(format!("hidden transcript cleanup failed: {}", cleanup_failures.join("; "))));
    }

    write_evidence(&output_path, TranscriptEvidence {
        schema: "mantle-transcript-output-v1",
        transcript: options.transcript.display().to_string(),
        in_place: transcript.options.in_place,
        visible_steps: visible_runs.len(),
        hidden_setups: hidden_setups.iter().map(HiddenRunEvidence::from).collect(),
        hidden_cleanups: hidden_cleanups.iter().map(HiddenRunEvidence::from).collect(),
        runs: visible_runs.iter().map(RunEvidence::from).collect(),
        cleanup_failures,
    })?;

    result?;
    Ok(TranscriptRunSummary {
        visible_steps: visible_runs.len(),
        output: output_path,
    })
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

fn write_evidence(path: &Path, evidence: TranscriptEvidence) -> Result<(), RunError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            RunError::Internal(format!("creating transcript output directory `{}`: {err}", parent.display()))
        })?;
    }
    let bytes = serde_json::to_vec_pretty(&evidence)
        .map_err(|err| RunError::Internal(format!("serializing transcript output artifact: {err}")))?;
    fs::write(path, bytes)
        .map_err(|err| RunError::Internal(format!("writing transcript output artifact `{}`: {err}", path.display())))
}

fn default_mantle_bin() -> PathBuf {
    std::env::current_exe().unwrap_or_else(|_| PathBuf::from("mantle"))
}

struct TranscriptScratch {
    tmp: TempDir,
    store: PathBuf,
    state_dir: PathBuf,
}

impl TranscriptScratch {
    fn new(in_place: bool) -> Result<Self, RunError> {
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
            crate::build_cmd::state_dir()
        } else {
            tmp.path().join("state")
        };
        fs::create_dir_all(&store).map_err(|err| RunError::Internal(format!("creating transcript store: {err}")))?;
        fs::create_dir_all(&state_dir)
            .map_err(|err| RunError::Internal(format!("creating transcript state-dir: {err}")))?;
        Ok(Self { tmp, store, state_dir })
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
) -> Result<HiddenRun, RunError> {
    let output = Command::new("/bin/sh")
        .arg("-c")
        .arg(expand_temp(script, scratch))
        .current_dir(work_dir)
        .env(TRANSCRIPT_TMP_ENV, scratch.tmp_path())
        .output()
        .map_err(|err| RunError::Internal(format!("running hidden transcript {label}: {err}")))?;
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

fn run_visible_mantle(
    mantle_bin: &Path,
    command: &str,
    expect_failure: bool,
    scratch: &TranscriptScratch,
    work_dir: &Path,
    in_place: bool,
    visible_index: usize,
) -> Result<CommandRun, RunError> {
    let tokens = split_command(command)?;
    if tokens.first().map(String::as_str) != Some(MANTLE_PREFIX) {
        return Err(RunError::Internal(format!("visible transcript command must start with `mantle`: `{command}`")));
    }

    let mut cmd = Command::new(mantle_bin);
    if !in_place {
        if !tokens.iter().any(|token| token == "--store" || token.starts_with("--store=")) {
            cmd.arg("--store").arg(&scratch.store);
        }
        if !tokens.iter().any(|token| token == "--state-dir" || token.starts_with("--state-dir=")) {
            cmd.arg("--state-dir").arg(&scratch.state_dir);
        }
    }
    for token in tokens.iter().skip(1) {
        cmd.arg(expand_temp(token, scratch));
    }
    cmd.current_dir(work_dir).env(TRANSCRIPT_TMP_ENV, scratch.tmp_path());
    let output = cmd.output().map_err(|err| {
        RunError::Internal(format!("running transcript command `{command}` via `{}`: {err}", mantle_bin.display()))
    })?;
    let mut combined = String::new();
    combined.push_str(&String::from_utf8_lossy(&output.stdout));
    combined.push_str(&String::from_utf8_lossy(&output.stderr));
    if expect_failure && output.status.success() {
        return Err(RunError::Build(format!("transcript command `{command}` unexpectedly succeeded")));
    }
    if !expect_failure && !output.status.success() {
        return Err(RunError::Build(format!(
            "transcript command `{command}` failed with status {:?}\n{}",
            output.status.code(),
            combined
        )));
    }
    Ok(CommandRun {
        visible_index,
        command: command.to_string(),
        expected_failure: expect_failure,
        exit_code: output.status.code(),
        normalized_output: normalize_output(&combined, scratch),
        raw_output: combined,
    })
}

fn verify_expectation(run: &CommandRun, expectation: &Block) -> Result<(), RunError> {
    match expectation.kind {
        BlockKind::Expect => verify_text_expectation(run, &expectation.body),
        BlockKind::ExpectJson => verify_json_expectation(run, &expectation.body),
        _ => unreachable!("only expectation blocks are verified"),
    }
}

fn verify_text_expectation(run: &CommandRun, body: &str) -> Result<(), RunError> {
    let mut search_from = 0;
    let mut saw_expectation = false;
    for expected in expectation_lines(body) {
        saw_expectation = true;
        let Some(relative_index) = run.normalized_output[search_from..].find(expected) else {
            return Err(RunError::Build(format!(
                "transcript step {} output did not contain expected fragment `{}` for command `{}`\nnormalized output:\n{}",
                run.visible_index, expected, run.command, run.normalized_output
            )));
        };
        search_from += relative_index + expected.len();
    }
    if !saw_expectation {
        return Err(RunError::Internal("expect block must contain at least one expected fragment".to_string()));
    }
    Ok(())
}

fn verify_json_expectation(run: &CommandRun, body: &str) -> Result<(), RunError> {
    let json_source = json_output_slice(&run.raw_output);
    let value: serde_json::Value = serde_json::from_str(json_source).map_err(|err| {
        RunError::Build(format!(
            "transcript step {} expected JSON output for `{}`: {err}",
            run.visible_index, run.command
        ))
    })?;
    let mut saw_expectation = false;
    for line in expectation_lines(body) {
        saw_expectation = true;
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
    if !saw_expectation {
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
    let mut out = String::new();
    let mut previous_blank = false;
    for line in normalized.lines() {
        let trimmed = line.trim_end();
        let blank = trimmed.is_empty();
        if blank && previous_blank {
            continue;
        }
        out.push_str(trimmed);
        out.push('\n');
        previous_blank = blank;
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
    let mut blocks = Vec::new();
    let mut lines = text.lines();
    while let Some(line) = lines.next() {
        if !line.starts_with("```") {
            continue;
        }
        let info = line.trim_start_matches("```").trim();
        if info.is_empty() {
            continue;
        }
        let mut body = String::new();
        let mut closed = false;
        for body_line in lines.by_ref() {
            if body_line.starts_with("```") {
                closed = true;
                break;
            }
            body.push_str(body_line);
            body.push('\n');
        }
        if !closed {
            return Err(RunError::Internal(format!("transcript block `{info}` is missing closing fence")));
        }
        if let Some(kind) = parse_block_kind(info)? {
            blocks.push(Block { kind, body });
        }
    }
    Ok(blocks)
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
        let scratch = TranscriptScratch::new(false).unwrap();
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
