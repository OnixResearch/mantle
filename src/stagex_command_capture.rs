//! Opt-in capture for concrete StageX commands. This is not an ActionTrustPlan:
//! callers must independently enumerate the complete transition before enabling it.
//! Only the local runner writes observations; serialized observations grant no authority.
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs::File;
use std::fs::OpenOptions;
use std::fs::{self};
use std::io::Read;
use std::io::Write;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use serde::Serialize;

use super::MesLibraryPlanError;
use crate::protected_exec::ProtectedSeccompAuditEvent;
use crate::protected_exec_seccomp::ProtectedSeccompSupervisor;

const FILE_BYTES_MAX: u64 = 512 * 1024 * 1024;
const JOURNAL_BYTES_MAX: usize = 64 * 1024 * 1024;
const COMMAND_COUNT_MAX: usize = 100_000;

#[derive(Clone, Debug, Serialize)]
pub(crate) enum ContentAuthority {
    Fixed { digest_blake3: String },
    Produced { command_id: String, output_id: String },
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct PlannedInput {
    pub path: PathBuf,
    pub authority: ContentAuthority,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct PlannedOutput {
    pub output_id: String,
    pub path: PathBuf,
    pub bytes_max: u64,
    pub expected_digest_blake3: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct PlannedChild {
    pub executable: PlannedInput,
    pub minimum_count: u32,
    pub maximum_count: u32,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct PlannedCommand {
    pub command_id: String,
    pub executable: PlannedInput,
    pub arguments: Vec<String>,
    pub current_dir: PathBuf,
    pub environment: BTreeMap<String, String>,
    pub inputs: Vec<PlannedInput>,
    pub outputs: Vec<PlannedOutput>,
    /// Includes nested shell/make/compiler children, but not the direct launch.
    pub children: Vec<PlannedChild>,
    /// Negative smoke commands must explicitly declare their observed exit code.
    pub expected_exit_code: i32,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct FileObservation {
    pub path: PathBuf,
    pub bytes_len: u64,
    pub digest_blake3: String,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct CommandObservation {
    pub command_id: Option<String>,
    pub executable: PathBuf,
    pub arguments: Vec<String>,
    pub raw_arguments: Vec<Vec<u8>>,
    pub current_dir: PathBuf,
    pub environment: BTreeMap<String, String>,
    pub executable_identity: Option<FileObservation>,
    pub inputs: Vec<FileObservation>,
    pub outputs: Vec<(String, FileObservation)>,
    pub child_pid: Option<u32>,
    pub exit_code: Option<i32>,
    pub elapsed_ms: u64,
    pub protected_exec_events: Vec<ProtectedSeccompAuditEvent>,
    pub failure: Option<String>,
}

struct State {
    plan: Vec<PlannedCommand>,
    next: usize,
    active: bool,
    failed: bool,
    produced: BTreeMap<(String, String), FileObservation>,
    observations: Vec<CommandObservation>,
    supervisor: Arc<ProtectedSeccompSupervisor>,
    audit_cursor: usize,
    journal: File,
    journal_bytes: usize,
}

thread_local! {
    static CAPTURE: RefCell<Option<Rc<RefCell<State>>>> = const { RefCell::new(None) };
}

/// Thread-bound guard; cannot migrate a proof scope onto an unprotected thread.
#[must_use]
pub(crate) struct StagexCommandCapture {
    state: Rc<RefCell<State>>,
}

impl StagexCommandCapture {
    pub(crate) fn begin(
        plan: Vec<PlannedCommand>,
        supervisor: Arc<ProtectedSeccompSupervisor>,
        journal_path: &Path,
    ) -> Result<Self, MesLibraryPlanError> {
        validate_plan(&plan)?;
        CAPTURE.with(|slot| {
            if slot.borrow().is_some() {
                return Err(failure("nested StageX command capture"));
            }
            let mut journal = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(journal_path)
                .map_err(|error| failure(format!("creating command capture journal: {error}")))?;
            // Persist the independent topology before any scoped command may launch.
            let header =
                serde_json::to_vec(&serde_json::json!({"format":"mantle-stagex-command-plan-v1", "commands": &plan}))
                    .map_err(|error| failure(format!("serializing command plan: {error}")))?;
            if header.len() >= JOURNAL_BYTES_MAX {
                return Err(failure("StageX command plan exceeds journal bound"));
            }
            journal
                .write_all(&header)
                .and_then(|()| journal.write_all(b"\n"))
                .and_then(|()| journal.sync_data())
                .map_err(|error| failure(format!("persisting command plan: {error}")))?;
            let audit_cursor = supervisor.audit_events().len();
            let state = Rc::new(RefCell::new(State {
                plan,
                next: 0,
                active: false,
                failed: false,
                produced: BTreeMap::new(),
                observations: Vec::new(),
                supervisor,
                audit_cursor,
                journal,
                journal_bytes: header.len() + 1,
            }));
            *slot.borrow_mut() = Some(Rc::clone(&state));
            Ok(Self { state })
        })
    }

    pub(crate) fn finish(self) -> Result<Vec<CommandObservation>, MesLibraryPlanError> {
        let mut state = self.state.borrow_mut();
        if state.active || state.failed || state.next != state.plan.len() {
            let message = format!(
                "incomplete StageX command capture: observed {} of {} commands, active={}, failed={}",
                state.next,
                state.plan.len(),
                state.active,
                state.failed
            );
            state.write_record(&serde_json::json!({"failure": &message}))?;
            state.failed = true;
            return Err(failure(message));
        }
        let events = state.supervisor.audit_events();
        if events.len() != state.audit_cursor {
            let trailing = &events[state.audit_cursor..];
            state.write_record(&serde_json::json!({"failure":"protected executions outside command boundary", "protected_exec_events":trailing}))?;
            state.failed = true;
            return Err(failure("protected executions outside StageX command boundary"));
        }
        state.write_record(&serde_json::json!({"complete":true}))?;
        Ok(std::mem::take(&mut state.observations))
    }
}

impl Drop for StagexCommandCapture {
    fn drop(&mut self) {
        CAPTURE.with(|slot| {
            slot.borrow_mut().take();
        });
        let mut state = self.state.borrow_mut();
        // Best effort only during unwinding; explicit finish reports all I/O errors.
        if state.active || state.failed || state.next != state.plan.len() {
            let events = state.supervisor.audit_events();
            let _ = state.write_record(&serde_json::json!({"aborted":true, "protected_exec_events":events}));
        }
    }
}

pub(super) struct PendingCommand {
    state: Rc<RefCell<State>>,
    command: PlannedCommand,
    observation: CommandObservation,
}

pub(super) fn before_command<S: AsRef<std::ffi::OsStr>>(
    executable: &Path,
    arguments: &[S],
    current_dir: &Path,
    environment: &BTreeMap<String, String>,
) -> Result<Option<PendingCommand>, MesLibraryPlanError> {
    CAPTURE.with(|slot| {
        let Some(state) = slot.borrow().as_ref().map(Rc::clone) else {
            return Ok(None);
        };
        let mut state_ref = state.borrow_mut();
        use std::os::unix::ffi::OsStrExt;
        let valid_arguments = arguments.iter().all(|argument| argument.as_ref().to_str().is_some());
        let raw_arguments = arguments.iter().map(|argument| argument.as_ref().as_bytes().to_vec()).collect();
        let arguments = arguments.iter().map(|argument| argument.as_ref().to_string_lossy().into_owned()).collect();
        let mut observation = CommandObservation {
            command_id: None,
            executable: executable.to_path_buf(),
            arguments,
            raw_arguments,
            current_dir: current_dir.to_path_buf(),
            environment: environment.clone(),
            executable_identity: None,
            inputs: Vec::new(),
            outputs: Vec::new(),
            child_pid: None,
            exit_code: None,
            elapsed_ms: 0,
            protected_exec_events: Vec::new(),
            failure: None,
        };
        let command = state_ref.plan.get(state_ref.next).cloned();
        let checked = (|| {
            let events = state_ref.supervisor.audit_events();
            if events.len() != state_ref.audit_cursor {
                observation.protected_exec_events = events[state_ref.audit_cursor..].to_vec();
                state_ref.audit_cursor = events.len();
                return Err(failure("protected execution outside StageX command boundary"));
            }
            if !valid_arguments {
                return Err(failure("non-UTF8 StageX command argument"));
            }
            if state_ref.active || state_ref.failed {
                return Err(failure("command capture already active or failed"));
            }
            let command = command.as_ref().ok_or_else(|| failure("unplanned StageX command"))?;
            observation.command_id = Some(command.command_id.clone());
            if command.executable.path != executable
                || command.arguments != observation.arguments
                || command.current_dir != current_dir
                || command.environment != *environment
            {
                return Err(failure(format!("StageX command drift: {}", command.command_id)));
            }
            observation.executable_identity = Some(state_ref.measure_input(&command.executable)?);
            for input in &command.inputs {
                observation.inputs.push(state_ref.measure_input(input)?);
            }
            for child in &command.children {
                state_ref.expected_identity(&child.executable)?;
            }
            // A preexisting output cannot stand in for a local build completion.
            for output in &command.outputs {
                match fs::symlink_metadata(&output.path) {
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => return Err(failure(format!("checking output absence: {error}"))),
                    Ok(_) => return Err(failure(format!("preexisting command output: {}", output.path.display()))),
                }
            }
            Ok(())
        })();
        if let Err(error) = checked {
            observation.failure = Some(error.to_string());
            state_ref.failed = true;
            state_ref.write_record(&observation)?;
            return Err(error);
        }
        state_ref.write_record(&serde_json::json!({"launch_pending": &observation}))?;
        state_ref.active = true;
        drop(state_ref);
        Ok(Some(PendingCommand {
            state,
            command: command.expect("checked command"),
            observation,
        }))
    })
}

impl PendingCommand {
    pub(super) fn complete(
        mut self,
        child_pid: Option<u32>,
        exit_code: Option<i32>,
        elapsed_ms: u64,
        process_failure: Option<String>,
    ) -> Result<(), MesLibraryPlanError> {
        self.observation.child_pid = child_pid;
        self.observation.exit_code = exit_code;
        self.observation.elapsed_ms = elapsed_ms;
        let mut state = self.state.borrow_mut();
        let quiescence = state.supervisor.wait_for_audit_quiescence();
        let events = state.supervisor.audit_events();
        self.observation.protected_exec_events = events[state.audit_cursor..].to_vec();
        state.audit_cursor = events.len();
        // Measure every declared local output even on failed/denied commands.
        // Missing outputs stay failures; they never acquire producer authority.
        let mut output_failure = None;
        for output in &self.command.outputs {
            match measure_file(&output.path, output.bytes_max) {
                Ok(measured) => {
                    if output
                        .expected_digest_blake3
                        .as_ref()
                        .is_some_and(|expected| expected != &measured.digest_blake3)
                    {
                        output_failure
                            .get_or_insert_with(|| format!("StageX output digest drift: {}", output.output_id));
                    }
                    self.observation.outputs.push((output.output_id.clone(), measured));
                }
                Err(error) => {
                    output_failure.get_or_insert_with(|| error.to_string());
                }
            }
        }
        let checked = (|| {
            quiescence.map_err(|error| failure(format!("waiting for protected command events: {error}")))?;
            if let Some(message) = process_failure {
                return Err(failure(message));
            }
            if exit_code != Some(self.command.expected_exit_code) {
                return Err(failure("unexpected StageX command exit status"));
            }
            let direct = self.observation.executable_identity.as_ref().expect("measured executable");
            validate_exec_events(&self.command, direct, child_pid, &self.observation.protected_exec_events, &state)?;
            if let Some(message) = output_failure {
                return Err(failure(message));
            }
            Ok(())
        })();
        if let Err(error) = &checked {
            self.observation.failure = Some(error.to_string());
            state.failed = true;
        }
        // Raw events and measured outputs are durable even when validation fails.
        state.write_record(&self.observation)?;
        state.active = false;
        checked?;
        for (output_id, measured) in &self.observation.outputs {
            state.produced.insert((self.command.command_id.clone(), output_id.clone()), measured.clone());
        }
        state.next += 1;
        state.observations.push(self.observation);
        Ok(())
    }
}

impl State {
    fn write_record(&mut self, value: &impl Serialize) -> Result<(), MesLibraryPlanError> {
        let result = (|| {
            let bytes =
                serde_json::to_vec(value).map_err(|error| failure(format!("serializing command evidence: {error}")))?;
            let next = self
                .journal_bytes
                .checked_add(bytes.len() + 1)
                .ok_or_else(|| failure("command journal size overflow"))?;
            if next > JOURNAL_BYTES_MAX {
                return Err(failure("command journal bound exceeded"));
            }
            self.journal
                .write_all(&bytes)
                .and_then(|()| self.journal.write_all(b"\n"))
                .and_then(|()| self.journal.sync_data())
                .map_err(|error| failure(format!("persisting command evidence: {error}")))?;
            self.journal_bytes = next;
            Ok(())
        })();
        if result.is_err() {
            self.failed = true;
        }
        result
    }

    fn expected_identity(&self, input: &PlannedInput) -> Result<String, MesLibraryPlanError> {
        match &input.authority {
            ContentAuthority::Fixed { digest_blake3 } => Ok(digest_blake3.clone()),
            ContentAuthority::Produced { command_id, output_id } => {
                let observed = self
                    .produced
                    .get(&(command_id.clone(), output_id.clone()))
                    .ok_or_else(|| failure(format!("producer not observed: {command_id}/{output_id}")))?;
                if observed.path != input.path {
                    return Err(failure("produced input path differs from producer output"));
                }
                Ok(observed.digest_blake3.clone())
            }
        }
    }

    fn measure_input(&self, input: &PlannedInput) -> Result<FileObservation, MesLibraryPlanError> {
        let measured = measure_file(&input.path, FILE_BYTES_MAX)?;
        if self.expected_identity(input)? != measured.digest_blake3 {
            return Err(failure(format!("StageX input digest drift: {}", input.path.display())));
        }
        Ok(measured)
    }
}

fn validate_exec_events(
    command: &PlannedCommand,
    direct: &FileObservation,
    child_pid: Option<u32>,
    events: &[ProtectedSeccompAuditEvent],
    state: &State,
) -> Result<(), MesLibraryPlanError> {
    let mut direct_count = 0;
    let mut child_counts = vec![0_u32; command.children.len()];
    for event in events {
        if event.phase != "protected"
            || event.policy_decision != "allowed"
            || event.inventory_entry_id.is_none()
            || !matches!(event.syscall.as_str(), "execve" | "execveat")
        {
            return Err(failure("denied or unauthenticated StageX protected execution"));
        }
        if Some(event.pid) == child_pid
            && event.resolved_host_path == direct.path
            && event.digest_hex == direct.digest_blake3
            && direct_count == 0
        {
            direct_count += 1;
            continue;
        }
        let mut matched = false;
        for (index, child) in command.children.iter().enumerate() {
            if event.resolved_host_path == child.executable.path
                && event.digest_hex == state.expected_identity(&child.executable)?
            {
                child_counts[index] =
                    child_counts[index].checked_add(1).ok_or_else(|| failure("child exec count overflow"))?;
                matched = true;
                break;
            }
        }
        if !matched {
            return Err(failure(format!("unplanned protected StageX child: {}", event.resolved_host_path.display())));
        }
    }
    if direct_count != 1 {
        return Err(failure("missing direct protected StageX exec event"));
    }
    for (child, count) in command.children.iter().zip(child_counts) {
        if count < child.minimum_count || count > child.maximum_count {
            return Err(failure("StageX child exec count outside declared bounds"));
        }
    }
    Ok(())
}

fn validate_plan(plan: &[PlannedCommand]) -> Result<(), MesLibraryPlanError> {
    if plan.is_empty() || plan.len() > COMMAND_COUNT_MAX {
        return Err(failure("empty or oversized StageX command topology"));
    }
    let mut commands = BTreeSet::new();
    let mut outputs = BTreeMap::<(String, String), PathBuf>::new();
    let mut paths = BTreeSet::new();
    for command in plan {
        if command.command_id.is_empty() || !commands.insert(command.command_id.clone()) {
            return Err(failure("empty or duplicate StageX command ID"));
        }
        validate_path(&command.current_dir)?;
        for input in std::iter::once(&command.executable)
            .chain(command.inputs.iter())
            .chain(command.children.iter().map(|child| &child.executable))
        {
            validate_path(&input.path)?;
            match &input.authority {
                ContentAuthority::Fixed { digest_blake3 } => validate_digest(digest_blake3)?,
                ContentAuthority::Produced { command_id, output_id } => {
                    if outputs.get(&(command_id.clone(), output_id.clone())) != Some(&input.path) {
                        return Err(failure("StageX generated input has no preceding exact producer"));
                    }
                }
            }
        }
        let mut child_paths = BTreeSet::new();
        for child in &command.children {
            if child.maximum_count == 0
                || child.minimum_count > child.maximum_count
                || !child_paths.insert(&child.executable.path)
            {
                return Err(failure("invalid or duplicate StageX child bounds"));
            }
        }
        for output in &command.outputs {
            validate_path(&output.path)?;
            if output.output_id.is_empty()
                || output.bytes_max == 0
                || output.bytes_max > FILE_BYTES_MAX
                || outputs
                    .insert((command.command_id.clone(), output.output_id.clone()), output.path.clone())
                    .is_some()
                || !paths.insert(output.path.clone())
            {
                return Err(failure("invalid or multiply produced StageX command output"));
            }
            if let Some(digest) = &output.expected_digest_blake3 {
                validate_digest(digest)?;
            }
        }
    }
    Ok(())
}

fn validate_path(path: &Path) -> Result<(), MesLibraryPlanError> {
    if !path.is_absolute() || path.components().any(|part| matches!(part, Component::ParentDir | Component::CurDir)) {
        return Err(failure("noncanonical StageX command path"));
    }
    Ok(())
}

fn validate_digest(digest: &str) -> Result<(), MesLibraryPlanError> {
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)) {
        return Err(failure("invalid StageX command content digest"));
    }
    Ok(())
}

fn measure_file(path: &Path, bytes_max: u64) -> Result<FileObservation, MesLibraryPlanError> {
    let metadata =
        fs::symlink_metadata(path).map_err(|error| failure(format!("measuring {}: {error}", path.display())))?;
    if !metadata.is_file() || metadata.len() > bytes_max {
        return Err(failure(format!("nonregular or oversized StageX input/output: {}", path.display())));
    }
    let mut file = File::open(path).map_err(|error| failure(format!("opening {}: {error}", path.display())))?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0_u8; 64 * 1024];
    let mut bytes_len = 0_u64;
    loop {
        let count = file.read(&mut buffer).map_err(|error| failure(format!("hashing {}: {error}", path.display())))?;
        if count == 0 {
            break;
        }
        bytes_len += count as u64;
        if bytes_len > bytes_max {
            return Err(failure("StageX measured file grew beyond bound"));
        }
        hasher.update(&buffer[..count]);
    }
    if bytes_len != metadata.len() {
        return Err(failure("StageX measured file changed length while hashing"));
    }
    Ok(FileObservation {
        path: path.to_path_buf(),
        bytes_len,
        digest_blake3: hasher.finalize().to_hex().to_string(),
    })
}

fn failure(message: impl Into<String>) -> MesLibraryPlanError {
    MesLibraryPlanError::Limit(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protected_exec::ProtectedExecPolicy;
    use crate::protected_exec::blake3_file_hex;
    use crate::protected_exec_seccomp::install_current_thread_exec_supervisor;

    const DRIVER_MODE: &str = "MANTLE_STAGEX_CAPTURE_TEST_MODE";

    fn fixed(path: PathBuf, digest: &str) -> PlannedInput {
        PlannedInput {
            path,
            authority: ContentAuthority::Fixed {
                digest_blake3: digest.to_owned(),
            },
        }
    }

    fn fixture_command(
        id: &str,
        executable: PlannedInput,
        root: &Path,
        arguments: Vec<String>,
        output: &Path,
    ) -> PlannedCommand {
        PlannedCommand {
            command_id: id.to_owned(),
            executable,
            arguments,
            current_dir: root.to_path_buf(),
            environment: BTreeMap::new(),
            inputs: Vec::new(),
            outputs: vec![PlannedOutput {
                output_id: "output".to_owned(),
                path: output.to_path_buf(),
                bytes_max: FILE_BYTES_MAX,
                expected_digest_blake3: None,
            }],
            children: Vec::new(),
            expected_exit_code: 0,
        }
    }

    fn fixture_executable(name: &str) -> PathBuf {
        let path = std::env::var_os("PATH").unwrap();
        std::env::split_paths(&path)
            .map(|directory| directory.join(name))
            .find(|candidate| candidate.is_file())
            .map(|candidate| fs::canonicalize(candidate).unwrap())
            .unwrap_or_else(|| panic!("test executable is absent: {name}"))
    }

    /// Arguments that select `cp` behavior from the resolved copy executable.
    ///
    /// nixpkgs installs coreutils as one multi-call binary. Its canonical path,
    /// and any renamed copy of it, must name the selected program explicitly.
    fn copy_program_arguments(executable: &Path) -> Vec<String> {
        if executable.file_name().is_some_and(|name| name == "coreutils") {
            vec!["--coreutils-prog=cp".to_owned()]
        } else {
            Vec::new()
        }
    }

    fn run(command: &PlannedCommand, stderr: &Path) -> Result<(), MesLibraryPlanError> {
        super::super::run_bounded_process(
            &command.executable.path,
            &command.arguments,
            &command.current_dir,
            &command.environment,
            stderr,
        )
    }

    #[test]
    fn protected_commands_bind_producers_and_preserve_rejections() {
        if let Ok(mode) = std::env::var(DRIVER_MODE) {
            run_protected_driver(&mode);
            return;
        }
        let name = std::thread::current().name().unwrap().to_owned();
        for mode in ["positive", "drift", "unknown-child", "missing", "cached-output"] {
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", &name, "--nocapture"])
                .env(DRIVER_MODE, mode)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{mode}: stdout={} stderr={}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    fn run_protected_driver(mode: &str) {
        let root = tempfile::tempdir().unwrap();
        // Keep measured and copied inputs independent of the debug libtest binary's size.
        let executable = fixture_executable("cp");
        let copy_program = copy_program_arguments(&executable);
        let digest = measure_file(&executable, FILE_BYTES_MAX).unwrap().digest_blake3;
        let generated = root.path().join("generated-helper");
        let output = root.path().join("output");
        let payload = root.path().join("payload");
        fs::write(&payload, b"stagex-command-output\n").unwrap();
        let payload_input = fixed(payload.clone(), &blake3_file_hex(&payload).unwrap());
        let mut first = fixture_command(
            "produce-helper",
            fixed(executable.clone(), &digest),
            root.path(),
            copy_program
                .iter()
                .cloned()
                .chain([
                    "--".to_owned(),
                    executable.display().to_string(),
                    generated.display().to_string(),
                ])
                .collect(),
            &generated,
        );
        first.inputs.push(first.executable.clone());
        let mut second = fixture_command(
            "consume-helper",
            PlannedInput {
                path: generated.clone(),
                authority: ContentAuthority::Produced {
                    command_id: first.command_id.clone(),
                    output_id: "output".to_owned(),
                },
            },
            root.path(),
            copy_program
                .iter()
                .cloned()
                .chain([
                    "--".to_owned(),
                    payload.display().to_string(),
                    output.display().to_string(),
                ])
                .collect(),
            &output,
        );
        second.inputs.push(payload_input.clone());
        let policy = ProtectedExecPolicy::from_stagex_plan(executable, digest, &["fixture".to_owned()], &[]).unwrap();
        let supervisor = Arc::new(install_current_thread_exec_supervisor(policy).unwrap());
        let journal = root.path().join("commands.jsonl");
        let plan = if mode == "unknown-child" {
            let shell = fixture_executable("bash");
            let shell_digest = measure_file(&shell, FILE_BYTES_MAX).unwrap().digest_blake3;
            supervisor
                .promote_verified_output("fixture", &[], &[crate::protected_exec::PromotedExecutable {
                    path: shell.clone(),
                    digest_hex: shell_digest.clone(),
                }])
                .unwrap();
            let mut nested = fixture_command(
                "nested",
                fixed(shell, &shell_digest),
                root.path(),
                ["-c".to_owned(), "\"$@\"; :".to_owned(), "fixture".to_owned()]
                    .into_iter()
                    .chain([first.executable.path.display().to_string()])
                    .chain(copy_program.iter().cloned())
                    .chain([
                        "--".to_owned(),
                        payload.display().to_string(),
                        output.display().to_string(),
                    ])
                    .collect(),
                &output,
            );
            nested.inputs = vec![first.executable.clone(), payload_input];
            vec![nested]
        } else {
            vec![first.clone(), second.clone()]
        };
        let capture = StagexCommandCapture::begin(plan.clone(), Arc::clone(&supervisor), &journal).unwrap();
        if mode == "missing" {
            assert!(capture.finish().is_err());
            assert!(!generated.exists());
            assert!(!output.exists());
            assert!(supervisor.audit_events().is_empty());
        } else if mode == "cached-output" {
            fs::write(&generated, b"cached").unwrap();
            assert!(run(&first, &root.path().join("first.stderr")).is_err());
            assert_eq!(fs::read(&generated).unwrap(), b"cached");
            assert!(!output.exists());
            assert!(supervisor.audit_events().is_empty());
            assert!(capture.finish().is_err());
        } else if mode == "unknown-child" {
            assert!(run(&plan[0], &root.path().join("first.stderr")).is_err());
            assert_eq!(supervisor.audit_events().len(), 2);
            assert_eq!(fs::read(&output).unwrap(), b"stagex-command-output\n");
            assert!(capture.finish().is_err());
        } else {
            run(&first, &root.path().join("first.stderr")).unwrap();
            supervisor
                .promote_verified_output("fixture", &[], &[crate::protected_exec::PromotedExecutable {
                    path: generated.clone(),
                    digest_hex: blake3_file_hex(&generated).unwrap(),
                }])
                .unwrap();
            if mode == "drift" {
                // The produced helper keeps its read-only source mode; substitute the file itself.
                fs::remove_file(&generated).unwrap();
                fs::write(&generated, b"substituted executable").unwrap();
                assert!(run(&second, &root.path().join("second.stderr")).is_err());
                assert!(!output.exists());
                assert_eq!(supervisor.audit_events().len(), 1);
                assert!(capture.finish().is_err());
            } else {
                run(&second, &root.path().join("second.stderr")).unwrap();
                let records = capture.finish().unwrap();
                assert_eq!(records.len(), 2);
                assert_eq!(
                    records[0].outputs[0].1.digest_blake3,
                    records[1].executable_identity.as_ref().unwrap().digest_blake3
                );
                assert_eq!(
                    records[1].outputs[0].1.digest_blake3,
                    blake3::hash(b"stagex-command-output\n").to_hex().to_string()
                );
                for record in records {
                    assert_eq!(record.protected_exec_events.len(), 1);
                    assert_eq!(Some(record.protected_exec_events[0].pid), record.child_pid);
                }
            }
        }
        let rows = fs::read_to_string(journal)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
            .collect::<Vec<_>>();
        if mode == "positive" {
            assert_eq!(rows.last().unwrap()["complete"], true);
        } else {
            assert!(rows.iter().any(|row| row.get("failure").is_some_and(|value| !value.is_null()) || row.get("aborted").is_some()));
        }
    }

    #[test]
    fn generated_authority_requires_preceding_exact_producer() {
        let root = Path::new("/stagex-proof");
        let first = fixture_command(
            "produce",
            fixed(root.join("seed"), &"a".repeat(64)),
            root,
            Vec::new(),
            &root.join("helper"),
        );
        let mut second = fixture_command(
            "consume",
            PlannedInput {
                path: root.join("helper"),
                authority: ContentAuthority::Produced {
                    command_id: "produce".to_owned(),
                    output_id: "output".to_owned(),
                },
            },
            root,
            Vec::new(),
            &root.join("result"),
        );
        validate_plan(&[first.clone(), second.clone()]).unwrap();
        assert!(validate_plan(&[second.clone(), first.clone()]).is_err());
        second.executable.path = root.join("same-name-other-root/helper");
        assert!(validate_plan(&[first.clone(), second.clone()]).is_err());
        second.executable.path = root.join("helper");
        second.outputs[0].path = first.outputs[0].path.clone();
        assert!(validate_plan(&[first, second]).is_err());
    }
}
