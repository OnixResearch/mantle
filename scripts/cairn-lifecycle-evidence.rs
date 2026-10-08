#!/usr/bin/env -S cargo -Zscript
---
[package]
edition = "2024"
---

//! Capture and evaluate Cairn lifecycle evidence for a single change. The core
//! decision logic is pure over strings so missing transcripts, unchecked tasks,
//! failed gates, dropped requirement IDs, missing archive paths, and stale status
//! claims can be tested without invoking Cairn.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_ROOT: &str = ".";
const DEFAULT_CAIRN: &str = "path:../cairn#cairn";
const CAIRN_ENV: &str = "CAIRN";
const ARCHIVE_DATE_ENV: &str = "CAIRN_ARCHIVE_DATE";
// Lifecycle state lives under `.cairn/` since the 2026-09-09 consolidation; the
// legacy top-level `cairn/` tree no longer exists.
const CHANGES_DIR: &str = ".cairn/changes";
const ARCHIVE_DIR: &str = ".cairn/archive";
const EVIDENCE_DIR_NAME: &str = "evidence";
const EVIDENCE_FILE_NAME: &str = "lifecycle-runner.md";
const SECONDS_PER_DAY: u64 = 86_400;
const COMMANDS_FILE_NAME: &str = "commands.txt";
const TASKS_FILE_NAME: &str = "tasks.md";
const EVIDENCE_FIXTURE_NAME: &str = "evidence.md";
const GATE_FIXTURE_NAME: &str = "gate.json";
const ACCEPTED_SPEC_FIXTURE_NAME: &str = "accepted-spec.md";
const STATUS_FIXTURE_NAME: &str = "status.txt";
const ARCHIVE_PATH_FIXTURE_NAME: &str = "archive-path.txt";
const POST_ARCHIVE_VALIDATION_FIXTURE_NAME: &str = "post-archive-validation.json";
const READY_STATUS: &str = "ready";
const BLOCKED_STATUS: &str = "blocked";
const MAX_OUTPUT_BYTES: usize = 16 * 1024;
const SHELL_PROGRAM: &str = "/bin/sh";
const SHELL_COMMAND_FLAG: &str = "-c";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let args = Args::parse(env::args().skip(1))?;
    if args.help {
        print_usage();
        return Ok(());
    }
    if args.self_test {
        return run_self_test();
    }
    if let Some(fixture) = args.fixture.as_ref() {
        return run_fixture(fixture, &args.requirement_ids);
    }
    if args.change.is_empty() {
        return Err("--change is required outside --fixture mode".to_string());
    }
    if args.dry_run {
        return run_real_dry_run(&args);
    }
    run_real_execute(&args)
}

#[derive(Debug, Default)]
struct Args {
    root: PathBuf,
    cairn: String,
    change: String,
    commands: Option<PathBuf>,
    fixture: Option<PathBuf>,
    requirement_ids: Vec<String>,
    accepted_spec: Option<PathBuf>,
    archive_date: String,
    dry_run: bool,
    self_test: bool,
    help: bool,
}

impl Args {
    fn parse<I>(mut args: I) -> Result<Self, String>
    where
        I: Iterator<Item = String>,
    {
        let mut parsed = Args {
            root: PathBuf::from(DEFAULT_ROOT),
            cairn: env::var(CAIRN_ENV).unwrap_or_else(|_| DEFAULT_CAIRN.to_string()),
            ..Args::default()
        };
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--root" => parsed.root = PathBuf::from(next_arg(&mut args, "--root")?),
                "--cairn" => parsed.cairn = next_arg(&mut args, "--cairn")?,
                "--change" => parsed.change = next_arg(&mut args, "--change")?,
                "--commands" => parsed.commands = Some(PathBuf::from(next_arg(&mut args, "--commands")?)),
                "--fixture" => parsed.fixture = Some(PathBuf::from(next_arg(&mut args, "--fixture")?)),
                "--requirement-id" => parsed.requirement_ids.push(next_arg(&mut args, "--requirement-id")?),
                "--accepted-spec" => parsed.accepted_spec = Some(PathBuf::from(next_arg(&mut args, "--accepted-spec")?)),
                "--archive-date" => parsed.archive_date = next_arg(&mut args, "--archive-date")?,
                "--dry-run" => parsed.dry_run = true,
                "--self-test" => parsed.self_test = true,
                "--help" | "-h" => parsed.help = true,
                other => return Err(format!("unknown argument: {other}")),
            }
        }
        if parsed.archive_date.is_empty() {
            parsed.archive_date = env::var(ARCHIVE_DATE_ENV).unwrap_or_else(|_| utc_today());
        }
        validate_archive_date(&parsed.archive_date)?;
        Ok(parsed)
    }
}

fn next_arg<I>(args: &mut I, flag: &str) -> Result<String, String>
where
    I: Iterator<Item = String>,
{
    args.next().ok_or_else(|| format!("missing value for {flag}"))
}

fn print_usage() {
    println!(
        "Usage: cargo -Zscript scripts/cairn-lifecycle-evidence.rs --change <name> --commands <commands.txt> [--cairn <flake-ref|binary>] [--archive-date YYYY-MM-DD] [--dry-run]"
    );
    println!("       cargo -Zscript scripts/cairn-lifecycle-evidence.rs --fixture <dir> --requirement-id <id>");
    println!("       cargo -Zscript scripts/cairn-lifecycle-evidence.rs --self-test");
    println!("--cairn defaults to ${CAIRN_ENV} or {DEFAULT_CAIRN}; a value containing '#' runs via `nix run`, anything else is executed directly.");
    println!("--archive-date defaults to ${ARCHIVE_DATE_ENV} or the current UTC date.");
}

// Days since 1970-01-01 to a proleptic Gregorian civil date (Hinnant's algorithm).
fn civil_date_from_days(days: u64) -> (u64, u64, u64) {
    let z = days + 719_468;
    let era = z / 146_097;
    let day_of_era = z - era * 146_097;
    let year_of_era = (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 { month_index + 3 } else { month_index - 9 };
    let year = year_of_era + era * 400 + u64::from(month <= 2);
    (year, month, day)
}

fn utc_today() -> String {
    let seconds = SystemTime::now().duration_since(UNIX_EPOCH).map(|elapsed| elapsed.as_secs()).unwrap_or(0);
    let (year, month, day) = civil_date_from_days(seconds / SECONDS_PER_DAY);
    format!("{year:04}-{month:02}-{day:02}")
}

fn validate_archive_date(date: &str) -> Result<(), String> {
    let bytes = date.as_bytes();
    let is_shaped = bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes.iter().enumerate().all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit());
    if !is_shaped || date < "2000-01-01" {
        return Err(format!("archive date must be YYYY-MM-DD on or after 2000-01-01, got `{date}`"));
    }
    Ok(())
}

fn cairn_command(cairn: &str) -> String {
    if cairn.contains('#') { format!("nix run {cairn} --") } else { cairn.to_string() }
}

fn change_dir(root: &Path, change: &str) -> PathBuf {
    root.join(CHANGES_DIR).join(change)
}

fn archive_path(archive_date: &str, change: &str) -> String {
    format!("{ARCHIVE_DIR}/{archive_date}-{change}")
}

#[derive(Debug)]
struct LifecycleState {
    commands: Vec<String>,
    tasks: String,
    evidence: String,
    gates: Vec<String>,
    accepted_spec: String,
    required_ids: Vec<String>,
    archive_path: String,
    status: String,
    post_archive_validation: String,
}

#[derive(Debug, PartialEq, Eq)]
struct LifecycleDecision {
    status: String,
    diagnostics: Vec<String>,
}

fn evaluate_lifecycle(state: &LifecycleState) -> LifecycleDecision {
    let mut diagnostics = Vec::new();
    check_commands(state, &mut diagnostics);
    check_tasks(state, &mut diagnostics);
    check_gates(state, &mut diagnostics);
    check_accepted_spec(state, &mut diagnostics);
    check_archive_path(state, &mut diagnostics);
    check_post_archive_validation(state, &mut diagnostics);
    check_status(state, &mut diagnostics);
    let status = if diagnostics.is_empty() { READY_STATUS } else { BLOCKED_STATUS };
    LifecycleDecision {
        status: status.to_string(),
        diagnostics,
    }
}

fn check_commands(state: &LifecycleState, diagnostics: &mut Vec<String>) {
    if state.commands.is_empty() {
        diagnostics.push("missing-command-list".to_string());
    }
    for command in &state.commands {
        let marker = format!("$ {command}");
        if !state.evidence.contains(&marker) {
            diagnostics.push(format!("missing-command-transcript:{command}"));
        }
    }
}

fn check_tasks(state: &LifecycleState, diagnostics: &mut Vec<String>) {
    for line in state.tasks.lines() {
        if line.trim_start().starts_with("- [") && !line.trim_start().starts_with("- [x]") {
            diagnostics.push("unchecked-tasks".to_string());
            return;
        }
    }
}

fn check_gates(state: &LifecycleState, diagnostics: &mut Vec<String>) {
    if state.gates.is_empty() {
        diagnostics.push("missing-cairn-gate".to_string());
    }
    for gate in &state.gates {
        if !gate.contains("\"valid\": true") || !gate.contains("\"verdict\": \"PASS\"") {
            diagnostics.push("failed-cairn-gate".to_string());
            return;
        }
    }
}

fn check_accepted_spec(state: &LifecycleState, diagnostics: &mut Vec<String>) {
    for id in &state.required_ids {
        let marker = format!("r[{id}]");
        if !state.accepted_spec.contains(&marker) {
            diagnostics.push(format!("sync-repair-required:{id}"));
        }
    }
}

fn check_archive_path(state: &LifecycleState, diagnostics: &mut Vec<String>) {
    if !state.archive_path.trim().starts_with(&format!("{ARCHIVE_DIR}/")) {
        diagnostics.push("missing-archive-path".to_string());
    }
}

fn check_post_archive_validation(state: &LifecycleState, diagnostics: &mut Vec<String>) {
    if !state.post_archive_validation.contains("\"valid\": true") {
        diagnostics.push("missing-post-archive-validation".to_string());
    }
}

fn check_status(state: &LifecycleState, diagnostics: &mut Vec<String>) {
    for line in state.status.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("M ") || trimmed.starts_with("D ") || trimmed.starts_with("A ") || trimmed.starts_with("??") {
            diagnostics.push("unexpected-dirty-status".to_string());
            return;
        }
    }
}

fn run_fixture(fixture: &Path, required_ids: &[String]) -> Result<(), String> {
    let state = fixture_state(fixture, required_ids)?;
    let decision = evaluate_lifecycle(&state);
    print_decision(&decision, &state);
    if decision.diagnostics.is_empty() {
        Ok(())
    } else {
        Err(decision.diagnostics.join("\n"))
    }
}

fn fixture_state(fixture: &Path, required_ids: &[String]) -> Result<LifecycleState, String> {
    let commands = read_commands(&fixture.join(COMMANDS_FILE_NAME))?;
    Ok(LifecycleState {
        commands,
        tasks: read_to_string(&fixture.join(TASKS_FILE_NAME))?,
        evidence: read_to_string(&fixture.join(EVIDENCE_FIXTURE_NAME))?,
        gates: vec![read_to_string(&fixture.join(GATE_FIXTURE_NAME))?],
        accepted_spec: read_to_string(&fixture.join(ACCEPTED_SPEC_FIXTURE_NAME))?,
        required_ids: required_ids.to_vec(),
        archive_path: read_to_string(&fixture.join(ARCHIVE_PATH_FIXTURE_NAME))?,
        status: read_to_string(&fixture.join(STATUS_FIXTURE_NAME))?,
        post_archive_validation: read_to_string(&fixture.join(POST_ARCHIVE_VALIDATION_FIXTURE_NAME))?,
    })
}

fn run_real_dry_run(args: &Args) -> Result<(), String> {
    let command_path = args.commands.as_ref().ok_or_else(|| "--commands is required".to_string())?;
    let commands = read_commands(command_path)?;
    let tasks = read_to_string(&change_dir(&args.root, &args.change).join(TASKS_FILE_NAME))?;
    let state = LifecycleState {
        commands,
        tasks,
        evidence: String::new(),
        gates: Vec::new(),
        accepted_spec: String::new(),
        required_ids: args.requirement_ids.clone(),
        archive_path: archive_path(&args.archive_date, &args.change),
        status: String::new(),
        post_archive_validation: String::new(),
    };
    println!("cairn lifecycle evidence dry-run: {}", args.change);
    println!("cairn: {}", cairn_command(&args.cairn));
    println!("archive date: {}", args.archive_date);
    println!("explicit commands: {}", state.commands.len());
    println!("archive path: {}", state.archive_path);
    for command in &state.commands {
        println!("- {command}");
    }
    Ok(())
}

fn run_real_execute(args: &Args) -> Result<(), String> {
    let command_path = args.commands.as_ref().ok_or_else(|| "--commands is required".to_string())?;
    let commands = read_commands(command_path)?;
    if !change_dir(&args.root, &args.change).is_dir() {
        return Err(format!("active change not found: {}", change_dir(&args.root, &args.change).display()));
    }
    let mut evidence = String::new();
    append_header(&mut evidence, args);
    let outcome = run_all(&mut evidence, &commands, args);
    match &outcome {
        Ok(()) => evidence.push_str("\n## Result\n\nAll commands exited 0.\n"),
        Err(failure) => evidence.push_str(&format!("\n## Result\n\nFAILED: {failure}\nLater commands were not run.\n")),
    }
    // The transcript is written even when a command fails, so a blocked run leaves evidence. After a
    // successful archive the change directory has moved, so the transcript follows it there.
    let evidence_path = evidence_dir(args).join(EVIDENCE_FILE_NAME);
    ensure_parent(&evidence_path)?;
    fs::write(&evidence_path, evidence).map_err(|err| format!("write {}: {err}", evidence_path.display()))?;
    println!("wrote lifecycle evidence: {}", evidence_path.display());
    outcome
}

fn run_all(evidence: &mut String, commands: &[String], args: &Args) -> Result<(), String> {
    for command in commands.iter().cloned().chain(lifecycle_commands(args)) {
        append_command_output(evidence, &command, &args.root)?;
    }
    Ok(())
}

fn evidence_dir(args: &Args) -> PathBuf {
    let active = change_dir(&args.root, &args.change);
    let archived = args.root.join(archive_path(&args.archive_date, &args.change));
    if !active.is_dir() && archived.is_dir() {
        return archived.join(EVIDENCE_DIR_NAME);
    }
    active.join(EVIDENCE_DIR_NAME)
}

fn lifecycle_commands(args: &Args) -> Vec<String> {
    let cairn = cairn_command(&args.cairn);
    let change = &args.change;
    let date = &args.archive_date;
    vec![
        format!("{cairn} validate --root ."),
        format!("{cairn} gate proposal {change} --root ."),
        format!("{cairn} gate design {change} --root ."),
        format!("{cairn} gate tasks {change} --root ."),
        format!("{cairn} sync {change} --root ."),
        format!("{cairn} sync {change} --root . --execute"),
        format!("{ARCHIVE_DATE_ENV}={date} {cairn} archive {change} --root ."),
        format!("{ARCHIVE_DATE_ENV}={date} {cairn} archive {change} --root . --execute"),
        format!("{cairn} validate --root ."),
        "git status --short --branch".to_string(),
    ]
}

fn append_header(evidence: &mut String, args: &Args) {
    evidence.push_str(&format!("# {} lifecycle evidence\n\n", args.change));
    evidence.push_str("Generated by `scripts/cairn-lifecycle-evidence.rs`.\n");
    evidence.push_str(&format!("Cairn: `{}`; archive date: {}.\n", cairn_command(&args.cairn), args.archive_date));
}

fn append_command_output(evidence: &mut String, command: &str, root: &Path) -> Result<(), String> {
    evidence.push_str(&format!("\n## Command\n\n```text\n$ {command}\n"));
    let output = Command::new(SHELL_PROGRAM)
        .arg(SHELL_COMMAND_FLAG)
        .arg(command)
        .current_dir(root)
        .output()
        .map_err(|err| format!("run command `{command}`: {err}"))?;
    append_bounded(evidence, &String::from_utf8_lossy(&output.stdout));
    append_bounded(evidence, &String::from_utf8_lossy(&output.stderr));
    evidence.push_str(&format!("[exit: {}]\n```\n", output.status.code().map_or("signal".to_string(), |code| code.to_string())));
    if output.status.success() {
        Ok(())
    } else {
        Err(format!("command failed: {command}"))
    }
}

fn append_bounded(target: &mut String, text: &str) {
    if text.len() <= MAX_OUTPUT_BYTES {
        target.push_str(text);
        return;
    }
    let mut end = MAX_OUTPUT_BYTES;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    target.push_str(&text[..end]);
    target.push_str("\n[output truncated]\n");
}

fn read_commands(path: &Path) -> Result<Vec<String>, String> {
    let contents = read_to_string(path)?;
    let commands = contents
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| !line.starts_with('#'))
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if commands.is_empty() {
        return Err(format!("{} contains no commands", path.display()));
    }
    Ok(commands)
}

fn read_to_string(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|err| format!("read {}: {err}", path.display()))
}

fn ensure_parent(path: &Path) -> Result<(), String> {
    let parent = path.parent().ok_or_else(|| format!("{} has no parent", path.display()))?;
    fs::create_dir_all(parent).map_err(|err| format!("create {}: {err}", parent.display()))
}

fn print_decision(decision: &LifecycleDecision, state: &LifecycleState) {
    println!("cairn lifecycle evidence: {}", decision.status);
    println!("commands: {}", state.commands.len());
    println!("archive_path: {}", state.archive_path.trim());
    if decision.diagnostics.is_empty() {
        println!("diagnostics: none");
    } else {
        println!("diagnostics:");
        for diagnostic in &decision.diagnostics {
            println!("- {diagnostic}");
        }
    }
}

fn run_self_test() -> Result<(), String> {
    let base = sample_state();
    expect_ready(&base)?;
    expect_blocked("unchecked task", with_tasks(&base, "- [ ] V1 unchecked\n"), "unchecked-tasks")?;
    expect_blocked("missing transcript", with_evidence(&base, ""), "missing-command-transcript")?;
    expect_blocked("failed gate", with_gate(&base, "{\"valid\":false,\"verdict\":\"FAIL\"}"), "failed-cairn-gate")?;
    expect_blocked("missing accepted id", with_accepted_spec(&base, "# spec\n"), "sync-repair-required")?;
    expect_blocked("dirty status", with_status(&base, "## main\n M src/main.rs\n"), "unexpected-dirty-status")?;
    expect_blocked("empty command list", with_commands(&base, Vec::new()), "missing-command-list")?;
    expect_blocked("missing post archive", with_post_archive(&base, "{}"), "missing-post-archive-validation")?;
    let legacy_archive = LifecycleState { archive_path: "cairn/archive/2026-07-03-fixture".to_string(), ..clone_state(&base) };
    expect_blocked("legacy archive path", legacy_archive, "missing-archive-path")?;
    expect_dates()?;
    expect_truncation()?;
    println!("cairn lifecycle evidence self-test passed");
    Ok(())
}

fn expect_dates() -> Result<(), String> {
    for (days, expected) in [(0, (1970, 1, 1)), (11_016, (2000, 2, 29)), (20_733, (2026, 10, 7)), (20_819, (2027, 1, 1))] {
        if civil_date_from_days(days) != expected {
            return Err(format!("civil date for day {days}: expected {expected:?}, got {:?}", civil_date_from_days(days)));
        }
    }
    for bad in ["1970-01-01", "2026-7-3", "2026/07/03", "", "20261007xx"] {
        if validate_archive_date(bad).is_ok() {
            return Err(format!("archive date `{bad}` should be rejected"));
        }
    }
    validate_archive_date("2026-10-07")
}

fn expect_truncation() -> Result<(), String> {
    let mut target = String::new();
    let text = "é".repeat(MAX_OUTPUT_BYTES);
    append_bounded(&mut target, &text);
    if !target.ends_with("[output truncated]\n") || target.len() > MAX_OUTPUT_BYTES + 32 {
        return Err("multi-byte output must truncate on a character boundary".to_string());
    }
    Ok(())
}

fn expect_ready(state: &LifecycleState) -> Result<(), String> {
    let decision = evaluate_lifecycle(state);
    if decision.diagnostics.is_empty() {
        Ok(())
    } else {
        Err(format!("expected ready, got {:?}", decision.diagnostics))
    }
}

fn expect_blocked(label: &str, state: LifecycleState, expected: &str) -> Result<(), String> {
    let decision = evaluate_lifecycle(&state);
    if decision.diagnostics.iter().any(|diagnostic| diagnostic.contains(expected)) {
        Ok(())
    } else {
        Err(format!("{label}: expected {expected}, got {:?}", decision.diagnostics))
    }
}

fn sample_state() -> LifecycleState {
    LifecycleState {
        commands: vec!["cargo test -p mantle fixture".to_string()],
        tasks: "- [x] V1 checked\n".to_string(),
        evidence: "$ cargo test -p mantle fixture\npass\n".to_string(),
        gates: vec!["{\"valid\": true, \"verdict\": \"PASS\"}".to_string()],
        accepted_spec: "r[verification_evidence.cairn_lifecycle_runner] requirement\n".to_string(),
        required_ids: vec!["verification_evidence.cairn_lifecycle_runner".to_string()],
        archive_path: ".cairn/archive/2026-07-03-fixture".to_string(),
        status: "## main...origin/main [ahead 1]\n".to_string(),
        post_archive_validation: "{\"valid\": true}".to_string(),
    }
}

fn with_tasks(state: &LifecycleState, tasks: &str) -> LifecycleState {
    let mut clone = clone_state(state);
    clone.tasks = tasks.to_string();
    clone
}

fn with_evidence(state: &LifecycleState, evidence: &str) -> LifecycleState {
    let mut clone = clone_state(state);
    clone.evidence = evidence.to_string();
    clone
}

fn with_gate(state: &LifecycleState, gate: &str) -> LifecycleState {
    let mut clone = clone_state(state);
    clone.gates = vec![gate.to_string()];
    clone
}

fn with_accepted_spec(state: &LifecycleState, accepted_spec: &str) -> LifecycleState {
    let mut clone = clone_state(state);
    clone.accepted_spec = accepted_spec.to_string();
    clone
}

fn with_status(state: &LifecycleState, status: &str) -> LifecycleState {
    let mut clone = clone_state(state);
    clone.status = status.to_string();
    clone
}

fn with_commands(state: &LifecycleState, commands: Vec<String>) -> LifecycleState {
    let mut clone = clone_state(state);
    clone.commands = commands;
    clone
}

fn with_post_archive(state: &LifecycleState, post_archive_validation: &str) -> LifecycleState {
    let mut clone = clone_state(state);
    clone.post_archive_validation = post_archive_validation.to_string();
    clone
}

fn clone_state(state: &LifecycleState) -> LifecycleState {
    LifecycleState {
        commands: state.commands.clone(),
        tasks: state.tasks.clone(),
        evidence: state.evidence.clone(),
        gates: state.gates.clone(),
        accepted_spec: state.accepted_spec.clone(),
        required_ids: state.required_ids.clone(),
        archive_path: state.archive_path.clone(),
        status: state.status.clone(),
        post_archive_validation: state.post_archive_validation.clone(),
    }
}
