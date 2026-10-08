//! Confined imperative-shell adapters for external batch allocation.
//!
//! The pure protocol and reconciliation logic lives in
//! `crunch_build::distributed`. This module verifies configured executables,
//! runs them without a shell or inherited environment, enforces bounded I/O and
//! deadlines, and translates direct JSON or Slurm CLI output into the canonical
//! protocol.
//!
//! r[impl external_batch_dispatchers.adapter_confinement]
//! r[impl external_batch_dispatchers.slurm_adapter]

use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use std::thread;
use std::time::Duration;

use crunch_build::distributed::EXTERNAL_BATCH_PROTOCOL_SCHEMA;
use crunch_build::distributed::ExternalBatchJobState;
use crunch_build::distributed::ExternalBatchOperation;
use crunch_build::distributed::ExternalBatchOperationKind;
use crunch_build::distributed::ExternalBatchOperationResponse;
use crunch_build::distributed::MAX_EXTERNAL_BATCH_IDENTIFIER_BYTES;
use crunch_build::distributed::RemoteNamedResourceQuantity;
use crunch_build::distributed::validate_external_batch_response;

use crunch_remote_app::BatchDispatchCommand;
use crunch_remote_app::BatchDispatchFacts;
use crunch_remote_app::Capability;
use crunch_remote_app::ExternalBatchPort;
use crunch_remote_app::PortError;
use crunch_remote_core::effect::EffectSession;
use crunch_remote_core::effect::RemoteEffectBinding;

use crate::remote_farm_config::RemoteBatchDispatcherAdapter;
use crate::remote_farm_config::RemoteBatchDispatcherCommandProfile;
use crate::remote_farm_config::RemoteBatchDispatcherProfile;
use crate::remote_farm_config::RemoteBatchEnvironmentHandle;
use crate::remote_farm_config::validate_remote_batch_dispatcher_profile;

const EXECUTABLE_DIGEST_BUFFER_BYTES: usize = 65_536;
const PROCESS_OUTPUT_BUFFER_BYTES: usize = 8_192;
const PROCESS_POLL_INTERVAL_MS: u64 = 10;
const MILLISECONDS_PER_SECOND: u64 = 1_000;
const PROCESS_TEARDOWN_TIMEOUT_SECS: u64 = 5;
const MAX_BATCH_ENVIRONMENT_VALUE_BYTES: usize = 65_536;
const SLURM_MEBIBYTE_BYTES: u64 = 1024 * 1024;
const SLURM_STATE_PENDING: [&str; 3] = ["CONFIGURING", "PD", "PENDING"];
const SLURM_STATE_RUNNING: [&str; 3] = ["COMPLETING", "R", "RUNNING"];
const SLURM_STATE_SUCCEEDED: [&str; 2] = ["CD", "COMPLETED"];
const SLURM_STATE_CANCELLED: [&str; 2] = ["CA", "CANCELLED"];
const SLURM_STATE_FAILED: [&str; 7] = ["F", "FAILED", "NODE_FAIL", "NF", "OOM", "OUT_OF_MEMORY", "TIMEOUT"];

const _: () = assert!(PROCESS_OUTPUT_BUFFER_BYTES > 0);

struct BoundedReadAttemptFacts {
    total_bytes: u64,
    buffer_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ConfinedProcessOutput {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

pub trait ExternalBatchDispatcher {
    fn submit(
        &self,
        operation: &ExternalBatchOperation,
        observed_unix_s: u64,
    ) -> Result<ExternalBatchOperationResponse, String>;

    fn observe(
        &self,
        operation: &ExternalBatchOperation,
        observed_unix_s: u64,
    ) -> Result<ExternalBatchOperationResponse, String>;

    fn cancel(
        &self,
        operation: &ExternalBatchOperation,
        observed_unix_s: u64,
    ) -> Result<ExternalBatchOperationResponse, String>;

    fn reconcile(
        &self,
        operation: &ExternalBatchOperation,
        observed_unix_s: u64,
    ) -> Result<ExternalBatchOperationResponse, String>;
}

pub enum ConfiguredExternalBatchDispatcher<'a> {
    Direct(DirectProcessBatchDispatcher<'a>),
    Slurm(SlurmCliBatchDispatcher<'a>),
}

impl<'a> ConfiguredExternalBatchDispatcher<'a> {
    pub fn new(profile: &'a RemoteBatchDispatcherProfile) -> Self {
        match profile.adapter {
            RemoteBatchDispatcherAdapter::DirectProcessV1 => Self::Direct(DirectProcessBatchDispatcher { profile }),
            RemoteBatchDispatcherAdapter::SlurmCliV1 => Self::Slurm(SlurmCliBatchDispatcher { profile }),
        }
    }

    fn run_effect(
        &self,
        kind: ExternalBatchOperationKind,
        operation: &ExternalBatchOperation,
        observed_unix_s: u64,
    ) -> Result<ExternalBatchOperationResponse, String> {
        validate_operation_kind(kind, operation)?;
        let profile = match self {
            Self::Direct(adapter) => adapter.profile,
            Self::Slurm(adapter) => adapter.profile,
        };
        let timeout_secs = match kind {
            ExternalBatchOperationKind::Submit => profile.submit.timeout_secs,
            ExternalBatchOperationKind::Observe => profile.observe.timeout_secs,
            ExternalBatchOperationKind::Cancel => profile.cancel.timeout_secs,
            ExternalBatchOperationKind::Reconcile => profile.reconcile.timeout_secs,
        };
        let digest = blake3::Hash::from_hex(&operation.operation_id_blake3)
            .map_err(|_| "external-batch-operation-id-malformed".to_string())?;
        let mut session = EffectSession::for_attempt(
            &operation.operation_id_blake3,
            RemoteEffectBinding {
                job_id: &operation.job_id,
                attempt_id: &operation.attempt_id,
                fence_generation: operation.fence_generation,
            },
        )
        .map_err(|error| format!("external-batch-effect-{error:?}"))?;
        let adapter: &dyn ExternalBatchDispatcher = match self {
            Self::Direct(adapter) => adapter,
            Self::Slurm(adapter) => adapter,
        };
        let mut port = ConfiguredBatchPort {
            adapter,
            operation,
            kind,
            observed_unix_s,
            response: None,
        };
        crunch_remote_app::dispatch_external_batch(
            &mut session,
            &mut port,
            BatchDispatchCommand {
                operation_id_blake3: &operation.operation_id_blake3,
                operation_digest_blake3: *digest.as_bytes(),
                job_id: &operation.job_id,
                attempt_id: &operation.attempt_id,
                fence_generation: operation.fence_generation,
                observed_unix_s,
                timeout_secs,
            },
        )
        .map_err(|error| match error {
            crunch_remote_app::ApplicationError::Port(error) => error.reason,
            crunch_remote_app::ApplicationError::Decision(error) => format!("external-batch-effect-{error:?}"),
        })?;
        port.response.ok_or_else(|| "external-batch-observation-missing".to_string())
    }
}

struct ConfiguredBatchPort<'a> {
    adapter: &'a dyn ExternalBatchDispatcher,
    operation: &'a ExternalBatchOperation,
    kind: ExternalBatchOperationKind,
    observed_unix_s: u64,
    response: Option<ExternalBatchOperationResponse>,
}

impl ExternalBatchPort for ConfiguredBatchPort<'_> {
    fn dispatch(&mut self, command: BatchDispatchCommand<'_>) -> Result<BatchDispatchFacts, PortError> {
        let run = || -> Result<(ExternalBatchOperationResponse, BatchDispatchFacts), String> {
            if command.operation_id_blake3 != self.operation.operation_id_blake3
                || command.observed_unix_s != self.observed_unix_s
                || command.job_id != &self.operation.job_id
                || command.attempt_id != &self.operation.attempt_id
                || command.fence_generation != self.operation.fence_generation
            {
                return Err("external-batch-operation-binding-mismatch".to_string());
            }
            let response = match self.kind {
                ExternalBatchOperationKind::Submit => self.adapter.submit(self.operation, self.observed_unix_s),
                ExternalBatchOperationKind::Observe => self.adapter.observe(self.operation, self.observed_unix_s),
                ExternalBatchOperationKind::Cancel => self.adapter.cancel(self.operation, self.observed_unix_s),
                ExternalBatchOperationKind::Reconcile => self.adapter.reconcile(self.operation, self.observed_unix_s),
            }?;
            validate_external_batch_response(self.operation, &response)?;
            let digest = blake3::Hash::from_hex(&response.operation_id_blake3)
                .map_err(|_| "external-batch-response-operation-id-malformed".to_string())?;
            let state = match response.state {
                ExternalBatchJobState::Submitted => crunch_remote_core::external_batch::BatchState::Submitted,
                ExternalBatchJobState::Pending => crunch_remote_core::external_batch::BatchState::Pending,
                ExternalBatchJobState::Running => crunch_remote_core::external_batch::BatchState::Running,
                ExternalBatchJobState::Succeeded => crunch_remote_core::external_batch::BatchState::Succeeded,
                ExternalBatchJobState::Failed => crunch_remote_core::external_batch::BatchState::Failed,
                ExternalBatchJobState::Cancelled => crunch_remote_core::external_batch::BatchState::Cancelled,
                ExternalBatchJobState::Unknown => crunch_remote_core::external_batch::BatchState::Unknown,
            };
            let facts = BatchDispatchFacts {
                operation_digest_blake3: *digest.as_bytes(),
                observed_unix_s: response.observed_unix_s,
                state,
            };
            Ok((response, facts))
        };
        let (response, facts) = run().map_err(|reason| PortError {
            capability: Capability::ExternalBatch,
            reason,
        })?;
        self.response = Some(response);
        Ok(facts)
    }
}

impl ExternalBatchDispatcher for ConfiguredExternalBatchDispatcher<'_> {
    fn submit(
        &self,
        operation: &ExternalBatchOperation,
        observed_unix_s: u64,
    ) -> Result<ExternalBatchOperationResponse, String> {
        self.run_effect(ExternalBatchOperationKind::Submit, operation, observed_unix_s)
    }

    fn observe(
        &self,
        operation: &ExternalBatchOperation,
        observed_unix_s: u64,
    ) -> Result<ExternalBatchOperationResponse, String> {
        self.run_effect(ExternalBatchOperationKind::Observe, operation, observed_unix_s)
    }

    fn cancel(
        &self,
        operation: &ExternalBatchOperation,
        observed_unix_s: u64,
    ) -> Result<ExternalBatchOperationResponse, String> {
        self.run_effect(ExternalBatchOperationKind::Cancel, operation, observed_unix_s)
    }

    fn reconcile(
        &self,
        operation: &ExternalBatchOperation,
        observed_unix_s: u64,
    ) -> Result<ExternalBatchOperationResponse, String> {
        self.run_effect(ExternalBatchOperationKind::Reconcile, operation, observed_unix_s)
    }
}


pub struct DirectProcessBatchDispatcher<'a> {
    profile: &'a RemoteBatchDispatcherProfile,
}

impl ExternalBatchDispatcher for DirectProcessBatchDispatcher<'_> {
    fn submit(
        &self,
        operation: &ExternalBatchOperation,
        observed_unix_s: u64,
    ) -> Result<ExternalBatchOperationResponse, String> {
        self.run(ExternalBatchOperationKind::Submit, &self.profile.submit, operation, observed_unix_s)
    }

    fn observe(
        &self,
        operation: &ExternalBatchOperation,
        observed_unix_s: u64,
    ) -> Result<ExternalBatchOperationResponse, String> {
        self.run(ExternalBatchOperationKind::Observe, &self.profile.observe, operation, observed_unix_s)
    }

    fn cancel(
        &self,
        operation: &ExternalBatchOperation,
        observed_unix_s: u64,
    ) -> Result<ExternalBatchOperationResponse, String> {
        self.run(ExternalBatchOperationKind::Cancel, &self.profile.cancel, operation, observed_unix_s)
    }

    fn reconcile(
        &self,
        operation: &ExternalBatchOperation,
        observed_unix_s: u64,
    ) -> Result<ExternalBatchOperationResponse, String> {
        self.run(ExternalBatchOperationKind::Reconcile, &self.profile.reconcile, operation, observed_unix_s)
    }
}

impl DirectProcessBatchDispatcher<'_> {
    fn run(
        &self,
        expected_kind: ExternalBatchOperationKind,
        command: &RemoteBatchDispatcherCommandProfile,
        operation: &ExternalBatchOperation,
        observed_unix_s: u64,
    ) -> Result<ExternalBatchOperationResponse, String> {
        validate_operation_kind(expected_kind, operation)?;
        validate_observation_time(observed_unix_s)?;
        validate_runtime_dispatcher_operation(self.profile, expected_kind)?;
        let stdin = serde_json::to_vec(operation)
            .map_err(|error| format!("external-batch-request-serialize-failed: {error}"))?;
        let output = run_confined_process(command, &self.profile.environment_handles, &command.args, &stdin)?;
        let response: ExternalBatchOperationResponse = serde_json::from_slice(&output.stdout)
            .map_err(|_| "external-batch-direct-response-malformed".to_string())?;
        validate_external_batch_response(operation, &response)?;
        if response.observed_unix_s != observed_unix_s {
            return Err("external-batch-direct-observation-time-mismatch".to_string());
        }
        assert_eq!(response.operation, expected_kind);
        assert!(response.observed_unix_s > 0);
        Ok(response)
    }
}

pub struct SlurmCliBatchDispatcher<'a> {
    profile: &'a RemoteBatchDispatcherProfile,
}

impl ExternalBatchDispatcher for SlurmCliBatchDispatcher<'_> {
    fn submit(
        &self,
        operation: &ExternalBatchOperation,
        observed_unix_s: u64,
    ) -> Result<ExternalBatchOperationResponse, String> {
        validate_operation_kind(ExternalBatchOperationKind::Submit, operation)?;
        validate_observation_time(observed_unix_s)?;
        validate_runtime_dispatcher_operation(self.profile, ExternalBatchOperationKind::Submit)?;
        let args = slurm_submit_args(self.profile, operation)?;
        let output = run_confined_process(&self.profile.submit, &self.profile.environment_handles, &args, &[])?;
        let external_job_id = parse_slurm_submit_job_id(&output.stdout)?;
        response_for_slurm(
            operation,
            ExternalBatchJobState::Submitted,
            Some(external_job_id),
            "slurm-submit-accepted",
            observed_unix_s,
        )
    }

    fn observe(
        &self,
        operation: &ExternalBatchOperation,
        observed_unix_s: u64,
    ) -> Result<ExternalBatchOperationResponse, String> {
        self.run_observation(ExternalBatchOperationKind::Observe, &self.profile.observe, operation, observed_unix_s)
    }

    fn cancel(
        &self,
        operation: &ExternalBatchOperation,
        observed_unix_s: u64,
    ) -> Result<ExternalBatchOperationResponse, String> {
        validate_operation_kind(ExternalBatchOperationKind::Cancel, operation)?;
        validate_observation_time(observed_unix_s)?;
        let external_job_id = required_external_job_id(operation)?;
        validate_runtime_dispatcher_operation(self.profile, ExternalBatchOperationKind::Cancel)?;
        let mut args = self.profile.cancel.args.clone();
        args.push(external_job_id.to_string());
        run_confined_process(&self.profile.cancel, &self.profile.environment_handles, &args, &[])?;
        response_for_slurm(
            operation,
            ExternalBatchJobState::Cancelled,
            Some(external_job_id.to_string()),
            "slurm-cancel-accepted",
            observed_unix_s,
        )
    }

    fn reconcile(
        &self,
        operation: &ExternalBatchOperation,
        observed_unix_s: u64,
    ) -> Result<ExternalBatchOperationResponse, String> {
        self.run_observation(ExternalBatchOperationKind::Reconcile, &self.profile.reconcile, operation, observed_unix_s)
    }
}

impl SlurmCliBatchDispatcher<'_> {
    fn run_observation(
        &self,
        expected_kind: ExternalBatchOperationKind,
        command: &RemoteBatchDispatcherCommandProfile,
        operation: &ExternalBatchOperation,
        observed_unix_s: u64,
    ) -> Result<ExternalBatchOperationResponse, String> {
        validate_operation_kind(expected_kind, operation)?;
        validate_observation_time(observed_unix_s)?;
        validate_runtime_dispatcher_operation(self.profile, expected_kind)?;
        let mut args = command.args.clone();
        if let Some(external_job_id) = operation.external_job_id.as_deref() {
            args.push("--noheader".to_string());
            if expected_kind == ExternalBatchOperationKind::Reconcile {
                args.extend(["--allocations".to_string(), "--format=State".to_string()]);
            } else {
                args.push("--format=%T".to_string());
            }
            args.push("--jobs".to_string());
            args.push(external_job_id.to_string());
            let output = run_confined_process(command, &self.profile.environment_handles, &args, &[])?;
            let state = parse_slurm_job_state(&output.stdout)?;
            return response_for_slurm(
                operation,
                state,
                Some(external_job_id.to_string()),
                "slurm-state-observed",
                observed_unix_s,
            );
        }
        if expected_kind != ExternalBatchOperationKind::Reconcile {
            return Err("external-batch-external-job-id-missing".to_string());
        }
        // sbatch assigned --comment=<dispatch_id>; only one exact matching
        // scheduler row may recover an accepted submit whose job ID was lost.
        args.extend([
            "--noheader".to_string(),
            "--parsable2".to_string(),
            "--allocations".to_string(),
            "--format=JobIDRaw,Comment,State".to_string(),
        ]);
        if args.len() > crate::remote_farm_config::remote_batch_command_arg_limit() {
            return Err("external-batch-slurm-argument-limit-exceeded".to_string());
        }
        let output = run_confined_process(command, &self.profile.environment_handles, &args, &[])?;
        let (state, external_job_id, reason) =
            parse_slurm_reconcile_by_comment(&output.stdout, &operation.dispatch_id_blake3);
        response_for_slurm(operation, state, external_job_id, reason, observed_unix_s)
    }
}

fn run_confined_process(
    profile: &RemoteBatchDispatcherCommandProfile,
    environment_handles: &[RemoteBatchEnvironmentHandle],
    args: &[String],
    stdin: &[u8],
) -> Result<ConfinedProcessOutput, String> {
    verify_executable_digest(&profile.program, &profile.expected_digest_blake3)?;
    let environment = resolve_environment_handles(environment_handles)?;
    let mut child = spawn_confined_process(profile, args, &environment)?;
    let child_stdin = child.stdin.take().ok_or_else(|| "external-batch-stdin-pipe-missing".to_string())?;
    let stdin_payload = stdin.to_vec();
    let stdin_writer = thread::spawn(move || write_child_stdin(child_stdin, &stdin_payload));
    let stdout = child.stdout.take().ok_or_else(|| "external-batch-stdout-pipe-missing".to_string())?;
    let stderr = child.stderr.take().ok_or_else(|| "external-batch-stderr-pipe-missing".to_string())?;
    let stdout_limit_bytes = bounded_output_limit(profile.stdout_limit_bytes)?;
    let stderr_limit_bytes = bounded_output_limit(profile.stderr_limit_bytes)?;
    let stdout_reader = thread::spawn(move || read_bounded_output(stdout, stdout_limit_bytes));
    let stderr_reader = thread::spawn(move || read_bounded_output(stderr, stderr_limit_bytes));
    let status = wait_for_child(&mut child, profile.timeout_secs)?;
    join_stdin_writer(stdin_writer)?;
    let stdout = join_output_reader(stdout_reader, "stdout")?;
    let stderr = join_output_reader(stderr_reader, "stderr")?;
    if !status.success() {
        return Err("external-batch-process-exit-failed".to_string());
    }
    assert!(stdout.len() <= stdout_limit_bytes);
    assert!(stderr.len() <= stderr_limit_bytes);
    Ok(ConfinedProcessOutput { stdout, stderr })
}

fn verify_executable_digest(program: &Path, expected_digest_blake3: &str) -> Result<(), String> {
    let mut file = std::fs::File::open(program).map_err(|_| "external-batch-executable-open-failed".to_string())?;
    let executable_size_bytes =
        file.metadata().map_err(|_| "external-batch-executable-metadata-failed".to_string())?.len();
    let buffer_size_bytes = u64::try_from(EXECUTABLE_DIGEST_BUFFER_BYTES)
        .map_err(|_| "external-batch-executable-buffer-size-invalid".to_string())?;
    let read_attempt_count_max = bounded_read_attempt_count(BoundedReadAttemptFacts {
        total_bytes: executable_size_bytes,
        buffer_bytes: buffer_size_bytes,
    })?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0_u8; EXECUTABLE_DIGEST_BUFFER_BYTES];
    let mut is_complete = false;
    for _ in 0..read_attempt_count_max {
        let read = file.read(&mut buffer).map_err(|_| "external-batch-executable-read-failed".to_string())?;
        if read == 0 {
            is_complete = true;
            break;
        }
        hasher.update(&buffer[..read]);
    }
    if !is_complete {
        return Err("external-batch-executable-changed-during-read".to_string());
    }
    let observed = hasher.finalize().to_hex().to_string();
    if observed != expected_digest_blake3 {
        return Err("external-batch-executable-digest-mismatch".to_string());
    }
    assert_eq!(observed.len(), expected_digest_blake3.len());
    assert!(!observed.is_empty());
    Ok(())
}

fn resolve_environment_handles(
    handles: &[RemoteBatchEnvironmentHandle],
) -> Result<Vec<(String, std::ffi::OsString)>, String> {
    let mut environment = Vec::with_capacity(handles.len());
    for handle in handles {
        let value = std::env::var_os(&handle.name)
            .ok_or_else(|| "external-batch-environment-handle-unavailable".to_string())?;
        if value.as_encoded_bytes().len() > MAX_BATCH_ENVIRONMENT_VALUE_BYTES {
            return Err("external-batch-environment-handle-value-too-large".to_string());
        }
        environment.push((handle.name.clone(), value));
    }
    Ok(environment)
}

fn spawn_confined_process(
    profile: &RemoteBatchDispatcherCommandProfile,
    args: &[String],
    environment: &[(String, std::ffi::OsString)],
) -> Result<Child, String> {
    let mut command = Command::new(&profile.program);
    command
        .args(args)
        .env_clear()
        .envs(environment.iter().cloned())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command.spawn().map_err(|_| "external-batch-process-spawn-failed".to_string())
}

fn write_child_stdin(mut child_stdin: std::process::ChildStdin, stdin: &[u8]) -> Result<(), String> {
    child_stdin.write_all(stdin).map_err(|_| "external-batch-stdin-write-failed".to_string())?;
    child_stdin.flush().map_err(|_| "external-batch-stdin-flush-failed".to_string())?;
    Ok(())
}

fn join_stdin_writer(writer: thread::JoinHandle<Result<(), String>>) -> Result<(), String> {
    writer.join().map_err(|_| "external-batch-stdin-writer-panicked".to_string())?
}

fn wait_for_child(child: &mut Child, timeout_secs: u64) -> Result<std::process::ExitStatus, String> {
    let poll_attempt_count_max = process_poll_attempt_count(timeout_secs)?;
    for _ in 0..poll_attempt_count_max {
        if let Some(status) = child.try_wait().map_err(|_| "external-batch-process-wait-failed".to_string())? {
            return Ok(status);
        }
        thread::sleep(Duration::from_millis(PROCESS_POLL_INTERVAL_MS));
    }
    terminate_child(child)?;
    Err("external-batch-process-timeout".to_string())
}

fn terminate_child(child: &mut Child) -> Result<(), String> {
    child.kill().map_err(|_| "external-batch-process-kill-failed".to_string())?;
    let poll_attempt_count_max = process_poll_attempt_count(PROCESS_TEARDOWN_TIMEOUT_SECS)?;
    for _ in 0..poll_attempt_count_max {
        if child.try_wait().map_err(|_| "external-batch-process-reap-failed".to_string())?.is_some() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(PROCESS_POLL_INTERVAL_MS));
    }
    Err("external-batch-process-teardown-timeout".to_string())
}

fn process_poll_attempt_count(timeout_secs: u64) -> Result<u64, String> {
    let timeout_ms = timeout_secs
        .checked_mul(MILLISECONDS_PER_SECOND)
        .ok_or_else(|| "external-batch-timeout-overflow".to_string())?;
    timeout_ms
        .checked_div(PROCESS_POLL_INTERVAL_MS)
        .and_then(|count| count.checked_add(1))
        .ok_or_else(|| "external-batch-poll-attempt-count-overflow".to_string())
}

fn bounded_read_attempt_count(facts: BoundedReadAttemptFacts) -> Result<u64, String> {
    if facts.buffer_bytes == 0 {
        return Err("external-batch-read-buffer-empty".to_string());
    }
    facts
        .total_bytes
        .checked_div(facts.buffer_bytes)
        .and_then(|count| count.checked_add(2))
        .ok_or_else(|| "external-batch-read-attempt-count-overflow".to_string())
}

fn read_bounded_output(mut reader: impl Read, limit_bytes: usize) -> Result<Vec<u8>, String> {
    let read_limit_bytes =
        u64::try_from(limit_bytes).map_err(|_| "external-batch-output-limit-overflow".to_string())?;
    let buffer_size_bytes = u64::try_from(PROCESS_OUTPUT_BUFFER_BYTES)
        .map_err(|_| "external-batch-output-buffer-size-invalid".to_string())?;
    assert!(buffer_size_bytes > 0, "external batch read buffer conversion must stay positive");
    let read_attempt_count_max = bounded_read_attempt_count(BoundedReadAttemptFacts {
        total_bytes: read_limit_bytes,
        buffer_bytes: buffer_size_bytes,
    })?;
    let mut output = Vec::with_capacity(limit_bytes.min(PROCESS_OUTPUT_BUFFER_BYTES));
    let mut buffer = [0_u8; PROCESS_OUTPUT_BUFFER_BYTES];
    let mut is_complete = false;
    for _ in 0..read_attempt_count_max {
        let read = reader.read(&mut buffer).map_err(|_| "external-batch-process-output-read-failed".to_string())?;
        if read == 0 {
            is_complete = true;
            break;
        }
        let next_len =
            output.len().checked_add(read).ok_or_else(|| "external-batch-process-output-overflow".to_string())?;
        if next_len > limit_bytes {
            return Err("external-batch-process-output-limit-exceeded".to_string());
        }
        output.extend_from_slice(&buffer[..read]);
    }
    if !is_complete {
        return Err("external-batch-process-output-limit-exceeded".to_string());
    }
    debug_assert!(output.len() <= limit_bytes);
    Ok(output)
}

fn join_output_reader(reader: thread::JoinHandle<Result<Vec<u8>, String>>, stream: &str) -> Result<Vec<u8>, String> {
    reader.join().map_err(|_| format!("external-batch-{stream}-reader-panicked"))?
}

fn bounded_output_limit(limit_bytes: u64) -> Result<usize, String> {
    usize::try_from(limit_bytes).map_err(|_| "external-batch-output-limit-overflow".to_string())
}

fn slurm_submit_args(
    profile: &RemoteBatchDispatcherProfile,
    operation: &ExternalBatchOperation,
) -> Result<Vec<String>, String> {
    let mut args = profile.submit.args.clone();
    args.push("--parsable".to_string());
    args.push(format!("--comment={}", operation.dispatch_id_blake3));
    args.push(format!("--cpus-per-task={}", operation.resources.cpu_units));
    let memory_mebibytes = operation.resources.memory_bytes.div_ceil(SLURM_MEBIBYTE_BYTES);
    let scratch_mebibytes = operation.resources.scratch_bytes.div_ceil(SLURM_MEBIBYTE_BYTES);
    args.push(format!("--mem={memory_mebibytes}M"));
    args.push(format!("--tmp={scratch_mebibytes}M"));
    append_slurm_named_resources(&mut args, "--gres", &operation.resources.accelerators);
    append_slurm_named_resources(&mut args, "--licenses", &operation.resources.named_tokens);
    args.push("--".to_string());
    args.push(profile.worker_program.display().to_string());
    args.extend(profile.worker_args.iter().cloned());
    args.extend([
        "--mantle-external-dispatch-id".to_string(),
        operation.dispatch_id_blake3.clone(),
        "--mantle-worker-endpoint".to_string(),
        operation.expected_worker_endpoint_id.clone(),
        "--mantle-worker-generation".to_string(),
        operation.dispatcher_generation.to_string(),
        "--mantle-fence-generation".to_string(),
        operation.fence_generation.get().to_string(),
    ]);
    if args.len() > crate::remote_farm_config::remote_batch_command_arg_limit() {
        return Err("external-batch-slurm-argument-limit-exceeded".to_string());
    }
    assert!(args.iter().any(|arg| arg == "--parsable"));
    assert!(args.iter().any(|arg| arg == "--"));
    Ok(args)
}

fn append_slurm_named_resources(args: &mut Vec<String>, flag: &str, resources: &[RemoteNamedResourceQuantity]) {
    for resource in resources {
        args.push(format!("{flag}={}:{}", resource.name, resource.quantity));
    }
}

fn parse_slurm_submit_job_id(stdout: &[u8]) -> Result<String, String> {
    let text = std::str::from_utf8(stdout).map_err(|_| "external-batch-slurm-submit-non-utf8".to_string())?;
    let first_line = text.lines().next().unwrap_or_default().trim();
    let external_job_id = first_line.split(';').next().unwrap_or_default();
    validate_slurm_job_id(external_job_id)?;
    Ok(external_job_id.to_string())
}

fn parse_slurm_job_state(stdout: &[u8]) -> Result<ExternalBatchJobState, String> {
    assert!(!SLURM_STATE_PENDING.is_empty(), "Slurm pending-state vocabulary must not be empty");
    assert!(!SLURM_STATE_FAILED.is_empty(), "Slurm failed-state vocabulary must not be empty");
    let text = std::str::from_utf8(stdout).map_err(|_| "external-batch-slurm-state-non-utf8".to_string())?;
    let state = text.lines().next().unwrap_or_default().trim().to_ascii_uppercase();
    if state.is_empty() {
        return Ok(ExternalBatchJobState::Unknown);
    }
    if SLURM_STATE_PENDING.contains(&state.as_str()) {
        return Ok(ExternalBatchJobState::Pending);
    }
    if SLURM_STATE_RUNNING.contains(&state.as_str()) {
        return Ok(ExternalBatchJobState::Running);
    }
    if SLURM_STATE_SUCCEEDED.contains(&state.as_str()) {
        return Ok(ExternalBatchJobState::Succeeded);
    }
    if SLURM_STATE_CANCELLED.contains(&state.as_str()) {
        return Ok(ExternalBatchJobState::Cancelled);
    }
    if SLURM_STATE_FAILED.contains(&state.as_str()) {
        return Ok(ExternalBatchJobState::Failed);
    }
    Err("external-batch-slurm-state-malformed".to_string())
}
fn parse_slurm_reconcile_by_comment(
    stdout: &[u8],
    dispatch_id_blake3: &str,
) -> (ExternalBatchJobState, Option<String>, &'static str) {
    let Ok(text) = std::str::from_utf8(stdout) else {
        return (ExternalBatchJobState::Unknown, None, "slurm-comment-observation-malformed");
    };
    let mut matched = None;
    for (index, line) in text.lines().enumerate() {
        if index >= crate::remote_build::MAX_REMOTE_STATUS_ITEMS {
            return (ExternalBatchJobState::Unknown, None, "slurm-comment-observation-limit-exceeded");
        }
        let mut fields = line.split('|');
        let (Some(job_id), Some(comment), Some(state)) = (fields.next(), fields.next(), fields.next()) else {
            return (ExternalBatchJobState::Unknown, None, "slurm-comment-observation-malformed");
        };
        if fields.next().is_some() {
            return (ExternalBatchJobState::Unknown, None, "slurm-comment-observation-malformed");
        }
        if comment != dispatch_id_blake3 {
            continue;
        }
        if matched.is_some() || validate_slurm_job_id(job_id.trim()).is_err() {
            return (ExternalBatchJobState::Unknown, None, "slurm-comment-observation-ambiguous");
        }
        let Ok(state) = parse_slurm_job_state(state.trim().as_bytes()) else {
            return (ExternalBatchJobState::Unknown, None, "slurm-comment-observation-malformed");
        };
        if state == ExternalBatchJobState::Unknown {
            return (ExternalBatchJobState::Unknown, None, "slurm-comment-observation-unresolved");
        }
        matched = Some((state, job_id.trim()));
    }
    match matched {
        Some((state, job_id)) => (state, Some(job_id.to_string()), "slurm-comment-job-observed"),
        None => (ExternalBatchJobState::Unknown, None, "slurm-comment-observation-unresolved"),
    }
}


fn validate_slurm_job_id(external_job_id: &str) -> Result<(), String> {
    if external_job_id.is_empty() || external_job_id.len() > MAX_EXTERNAL_BATCH_IDENTIFIER_BYTES {
        return Err("external-batch-slurm-job-id-length-invalid".to_string());
    }
    let is_valid = external_job_id
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'));
    if !is_valid {
        return Err("external-batch-slurm-job-id-invalid".to_string());
    }
    Ok(())
}

fn required_external_job_id(operation: &ExternalBatchOperation) -> Result<&str, String> {
    operation
        .external_job_id
        .as_deref()
        .ok_or_else(|| "external-batch-external-job-id-missing".to_string())
}

fn response_for_slurm(
    operation: &ExternalBatchOperation,
    state: ExternalBatchJobState,
    external_job_id: Option<String>,
    reason_code: &str,
    observed_unix_s: u64,
) -> Result<ExternalBatchOperationResponse, String> {
    let response = ExternalBatchOperationResponse {
        schema: EXTERNAL_BATCH_PROTOCOL_SCHEMA.to_string(),
        operation: operation.operation,
        operation_id_blake3: operation.operation_id_blake3.clone(),
        dispatch_id_blake3: operation.dispatch_id_blake3.clone(),
        adapter_instance_id: operation.adapter_instance_id.clone(),
        dispatcher_generation: operation.dispatcher_generation,
        job_id: operation.job_id.clone(),
        attempt_id: operation.attempt_id.clone(),
        fence_generation: operation.fence_generation,
        state,
        external_job_id,
        reason_code: reason_code.to_string(),
        observed_unix_s,
        non_claim: crunch_build::distributed::EXTERNAL_BATCH_NON_CLAIM.to_string(),
    };
    validate_external_batch_response(operation, &response)?;
    assert_eq!(response.operation_id_blake3, operation.operation_id_blake3);
    assert_eq!(response.dispatch_id_blake3, operation.dispatch_id_blake3);
    Ok(response)
}

fn validate_runtime_dispatcher_operation(
    profile: &RemoteBatchDispatcherProfile,
    operation: ExternalBatchOperationKind,
) -> Result<(), String> {
    validate_remote_batch_dispatcher_profile(profile)?;
    if !profile.allowed_operations.contains(&operation) {
        return Err("external-batch-operation-not-allowed".to_string());
    }
    Ok(())
}

fn validate_operation_kind(
    expected: ExternalBatchOperationKind,
    operation: &ExternalBatchOperation,
) -> Result<(), String> {
    if operation.operation != expected {
        return Err("external-batch-operation-kind-mismatch".to_string());
    }
    Ok(())
}

fn validate_observation_time(observed_unix_s: u64) -> Result<(), String> {
    if observed_unix_s == 0 {
        return Err("external-batch-observation-time-zero".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use crunch_build::distributed::ExternalBatchLimits;
    use crunch_build::distributed::ExternalBatchOperationInput;
    use crunch_build::distributed::RemoteResourceRequirements;
    use crunch_build::distributed::RemoteResourceVector;
    use crunch_build::distributed::plan_external_batch_operation;
    use crunch_remote_core::attempt::RemoteAttemptId;
    use crunch_remote_core::attempt::RemoteFenceGeneration;
    use crunch_remote_core::attempt::RemoteJobId;

    use super::*;

    const TEST_MEMORY_BYTES: u64 = 4_294_967_297;
    const TEST_SCRATCH_BYTES: u64 = 8_589_934_593;
    const TEST_TIMEOUT_SECS: u64 = 10;
    const TEST_OUTPUT_LIMIT_BYTES: u64 = 4_096;
    const TEST_RECONCILE_ATTEMPTS: u32 = 3;
    const TEST_OBSERVED_UNIX_S: u64 = 1_700_000_000;

    fn fixture_operation(kind: ExternalBatchOperationKind, external_job_id: Option<&str>) -> ExternalBatchOperation {
        plan_external_batch_operation(ExternalBatchOperationInput {
            kind,
            adapter_instance_id: "slurm-fixture".to_string(),
            dispatcher_profile_ref_blake3: blake3::hash(b"profile").to_hex().to_string(),
            provider_class: "slurm".to_string(),
            worker_bootstrap_ref_blake3: blake3::hash(b"bootstrap").to_hex().to_string(),
            semantic_capability_class: "x86_64-linux:kvm".to_string(),
            dispatcher_generation: 1,
            normalized_build_key: blake3::hash(b"build").to_hex().to_string(),
            job_id: RemoteJobId::new("job-1").expect("fixture job id"),
            attempt_id: RemoteAttemptId::new("attempt-1").expect("fixture attempt id"),
            fence_generation: RemoteFenceGeneration::INITIAL,
            expected_worker_endpoint_id: "worker-batch-1".to_string(),
            resource_requirements: RemoteResourceRequirements {
                quantities: RemoteResourceVector {
                    cpu_units: 2,
                    memory_bytes: TEST_MEMORY_BYTES,
                    scratch_bytes: TEST_SCRATCH_BYTES,
                    accelerators: vec![RemoteNamedResourceQuantity {
                        name: "nvidia-sm90".to_string(),
                        quantity: 1,
                    }],
                    named_tokens: vec![RemoteNamedResourceQuantity {
                        name: "linker-seat".to_string(),
                        quantity: 1,
                    }],
                },
                semantic_accelerator_classes: vec!["nvidia-sm90".to_string()],
            },
            limits: ExternalBatchLimits {
                startup_timeout_secs: TEST_TIMEOUT_SECS,
                terminal_timeout_secs: TEST_TIMEOUT_SECS,
                max_reconcile_attempts: TEST_RECONCILE_ATTEMPTS,
            },
            external_job_id: external_job_id.map(str::to_string),
        })
        .expect("fixture operation plans")
    }

    fn fake_slurm_profile(script: &Path) -> RemoteBatchDispatcherProfile {
        let command = |operation: &str| RemoteBatchDispatcherCommandProfile {
            program: script.to_path_buf(),
            expected_digest_blake3: executable_digest(script),
            args: vec![operation.to_string()],
            timeout_secs: TEST_TIMEOUT_SECS,
            stdout_limit_bytes: TEST_OUTPUT_LIMIT_BYTES,
            stderr_limit_bytes: TEST_OUTPUT_LIMIT_BYTES,
        };
        RemoteBatchDispatcherProfile {
            instance_id: "slurm-fixture".to_string(),
            generation: 1,
            adapter: RemoteBatchDispatcherAdapter::SlurmCliV1,
            protocol_schema: EXTERNAL_BATCH_PROTOCOL_SCHEMA.to_string(),
            allowed_operations: vec![
                ExternalBatchOperationKind::Submit,
                ExternalBatchOperationKind::Observe,
                ExternalBatchOperationKind::Cancel,
                ExternalBatchOperationKind::Reconcile,
            ],
            provider_class: "slurm".to_string(),
            submit: command("submit"),
            observe: command("observe"),
            cancel: command("cancel"),
            reconcile: command("reconcile"),
            worker_program: Path::new("/bin/true").to_path_buf(),
            worker_program_digest_blake3: blake3::hash(b"fixture-worker").to_hex().to_string(),
            worker_args: vec!["remote".to_string(), "serve".to_string()],
            environment_handles: Vec::new(),
            bootstrap_policy: crate::remote_farm_config::RemoteBatchBootstrapPolicy::FixedTemplateV1,
            redact_provider_output: true,
            startup_timeout_secs: TEST_TIMEOUT_SECS,
            terminal_timeout_secs: TEST_TIMEOUT_SECS,
            max_reconcile_attempts: TEST_RECONCILE_ATTEMPTS,
        }
    }

    fn write_fake_slurm(temp: &tempfile::TempDir) -> std::path::PathBuf {
        let script = temp.path().join("fake-slurm");
        std::fs::write(
            &script,
            "#!/bin/sh\noperation=$1\ncase \"$operation\" in\n  submit) mem=0; tmp=0; for arg do case \"$arg\" in --mem=4097M) mem=1 ;; --tmp=8193M) tmp=1 ;; esac; done; [ \"$mem\" -eq 1 ] && [ \"$tmp\" -eq 1 ] || exit 78; printf '4242;fixture\\n' ;;\n  observe|reconcile) printf 'RUNNING\\n' ;;\n  cancel) exit 0 ;;\n  malformed) printf 'NOT_A_SLURM_STATE\\n' ;;\n  flood) i=0; while [ \"$i\" -lt 8192 ]; do printf x; i=$((i + 1)); done ;;\n  hang) while :; do :; done ;;\n  *) exit 64 ;;\nesac\n",
        )
        .expect("fake Slurm fixture writes");
        let mut permissions = std::fs::metadata(&script).expect("fixture metadata reads").permissions();
        permissions.set_mode(0o700);
        std::fs::set_permissions(&script, permissions).expect("fixture mode applies");
        script
    }

    fn fake_slurm_suite_profile(temp: &tempfile::TempDir) -> RemoteBatchDispatcherProfile {
        let source = write_fake_slurm(temp);
        let mut profile = fake_slurm_profile(&source);
        for (command, name, operation) in [
            (&mut profile.submit, "sbatch", "submit"),
            (&mut profile.observe, "squeue", "observe"),
            (&mut profile.reconcile, "sacct", "reconcile"),
            (&mut profile.cancel, "scancel", "cancel"),
        ] {
            let target = temp.path().join(name);
            std::fs::copy(&source, &target).expect("exact fake Slurm executable copies");
            command.program = target.clone();
            command.expected_digest_blake3 = executable_digest(&target);
            command.args = vec![operation.to_string()];
        }
        profile
    }

    fn executable_digest(path: &Path) -> String {
        blake3::hash(&std::fs::read(path).expect("fixture executable reads")).to_hex().to_string()
    }

    fn write_direct_response_fixture(
        temp: &tempfile::TempDir,
        response: &ExternalBatchOperationResponse,
    ) -> std::path::PathBuf {
        let encoded = serde_json::to_string(response).expect("direct fixture response serializes");
        assert!(!encoded.contains('\''));
        let script = temp.path().join("fake-direct-dispatcher");
        let source = format!(
            "#!/bin/sh\nif [ \"${{HOME+x}}\" = x ]; then exit 64; fi\nIFS= read -r _request || [ -n \"$_request\" ] || exit 65\nprintf '%s\\n' '{encoded}'\n"
        );
        std::fs::write(&script, source).expect("direct dispatcher fixture writes");
        let mut permissions = std::fs::metadata(&script).expect("fixture metadata reads").permissions();
        permissions.set_mode(0o700);
        std::fs::set_permissions(&script, permissions).expect("fixture mode applies");
        script
    }

    #[test]
    fn direct_adapter_exchanges_canonical_json_with_clean_environment() {
        let temp = tempfile::tempdir().expect("temporary direct dispatcher dir");
        let operation = fixture_operation(ExternalBatchOperationKind::Submit, None);
        let response = ExternalBatchOperationResponse {
            schema: EXTERNAL_BATCH_PROTOCOL_SCHEMA.to_string(),
            operation: operation.operation,
            operation_id_blake3: operation.operation_id_blake3.clone(),
            dispatch_id_blake3: operation.dispatch_id_blake3.clone(),
            adapter_instance_id: operation.adapter_instance_id.clone(),
            dispatcher_generation: operation.dispatcher_generation,
            job_id: operation.job_id.clone(),
            attempt_id: operation.attempt_id.clone(),
            fence_generation: operation.fence_generation,
            state: ExternalBatchJobState::Submitted,
            external_job_id: Some("direct-42".to_string()),
            reason_code: "direct-submit-accepted".to_string(),
            observed_unix_s: TEST_OBSERVED_UNIX_S,
            non_claim: crunch_build::distributed::EXTERNAL_BATCH_NON_CLAIM.to_string(),
        };
        let script = write_direct_response_fixture(&temp, &response);
        let command = RemoteBatchDispatcherCommandProfile {
            program: script.clone(),
            expected_digest_blake3: executable_digest(&script),
            args: Vec::new(),
            timeout_secs: TEST_TIMEOUT_SECS,
            stdout_limit_bytes: TEST_OUTPUT_LIMIT_BYTES,
            stderr_limit_bytes: TEST_OUTPUT_LIMIT_BYTES,
        };
        let mut profile = RemoteBatchDispatcherProfile {
            instance_id: "slurm-fixture".to_string(),
            generation: 1,
            adapter: RemoteBatchDispatcherAdapter::DirectProcessV1,
            protocol_schema: EXTERNAL_BATCH_PROTOCOL_SCHEMA.to_string(),
            allowed_operations: vec![
                ExternalBatchOperationKind::Submit,
                ExternalBatchOperationKind::Observe,
                ExternalBatchOperationKind::Cancel,
                ExternalBatchOperationKind::Reconcile,
            ],
            provider_class: "direct".to_string(),
            submit: command.clone(),
            observe: command.clone(),
            cancel: command.clone(),
            reconcile: command,
            worker_program: Path::new("/bin/true").to_path_buf(),
            worker_program_digest_blake3: blake3::hash(b"fixture-worker").to_hex().to_string(),
            worker_args: Vec::new(),
            environment_handles: Vec::new(),
            bootstrap_policy: crate::remote_farm_config::RemoteBatchBootstrapPolicy::DirectWorkerExecV1,
            redact_provider_output: true,
            startup_timeout_secs: TEST_TIMEOUT_SECS,
            terminal_timeout_secs: TEST_TIMEOUT_SECS,
            max_reconcile_attempts: TEST_RECONCILE_ATTEMPTS,
        };
        let observed = ConfiguredExternalBatchDispatcher::new(&profile)
            .submit(&operation, TEST_OBSERVED_UNIX_S)
            .expect("direct canonical exchange succeeds through the application port");

        assert_eq!(observed, response);
        assert_eq!(observed.external_job_id.as_deref(), Some("direct-42"));
        let reconcile = fixture_operation(ExternalBatchOperationKind::Reconcile, None);
        let lookup_response = ExternalBatchOperationResponse {
            operation: reconcile.operation,
            operation_id_blake3: reconcile.operation_id_blake3.clone(),
            dispatch_id_blake3: reconcile.dispatch_id_blake3.clone(),
            job_id: reconcile.job_id.clone(),
            attempt_id: reconcile.attempt_id.clone(),
            fence_generation: reconcile.fence_generation,
            state: ExternalBatchJobState::Unknown,
            external_job_id: None,
            reason_code: "direct-comment-lookup-unresolved".to_string(),
            ..response
        };
        let encoded = serde_json::to_string(&lookup_response).expect("lookup response serializes");
        assert!(!encoded.contains('\''));
        let source = format!(
            "#!/bin/sh\nIFS= read -r request || [ -n \"$request\" ] || exit 65\ncase \"$request\" in\n *'\"operation\":\"reconcile\"'*'\"dispatch_id_blake3\":\"{}\"'*) ;;\n *) exit 66 ;;\nesac\ncase \"$request\" in\n *'\"external_job_id\"'*) exit 67 ;;\nesac\nprintf '%s\\n' '{encoded}'\n",
            reconcile.dispatch_id_blake3,
        );
        std::fs::write(&script, source).expect("exact comment lookup fixture writes");
        profile.reconcile.expected_digest_blake3 = executable_digest(&script);
        let looked_up = ConfiguredExternalBatchDispatcher::new(&profile)
            .reconcile(&reconcile, TEST_OBSERVED_UNIX_S)
            .expect("physical direct dispatcher accepts exact dispatch lookup with no external job id");
        assert_eq!(looked_up, lookup_response);
        assert_eq!(looked_up.external_job_id, None);
    }

    #[test]
    fn slurm_adapter_submits_observes_reconciles_and_cancels_fake_cli() {
        let temp = tempfile::tempdir().expect("temporary fake Slurm dir");
        let profile = fake_slurm_suite_profile(&temp);
        let adapter = ConfiguredExternalBatchDispatcher::new(&profile);
        let submit = adapter
            .submit(&fixture_operation(ExternalBatchOperationKind::Submit, None), TEST_OBSERVED_UNIX_S)
            .expect("fake Slurm submit succeeds");
        let observe = adapter
            .observe(&fixture_operation(ExternalBatchOperationKind::Observe, Some("4242")), TEST_OBSERVED_UNIX_S)
            .expect("fake Slurm observe succeeds");
        let reconcile = adapter
            .reconcile(&fixture_operation(ExternalBatchOperationKind::Reconcile, Some("4242")), TEST_OBSERVED_UNIX_S)
            .expect("fake Slurm reconcile succeeds");
        let cancel = adapter
            .cancel(&fixture_operation(ExternalBatchOperationKind::Cancel, Some("4242")), TEST_OBSERVED_UNIX_S)
            .expect("fake Slurm cancel succeeds");

        assert_eq!(submit.external_job_id.as_deref(), Some("4242"));
        assert_eq!(profile.submit.program.file_name().and_then(|name| name.to_str()), Some("sbatch"));
        assert_eq!(profile.observe.program.file_name().and_then(|name| name.to_str()), Some("squeue"));
        assert_eq!(profile.reconcile.program.file_name().and_then(|name| name.to_str()), Some("sacct"));
        assert_eq!(profile.cancel.program.file_name().and_then(|name| name.to_str()), Some("scancel"));
        assert_eq!(observe.state, ExternalBatchJobState::Running);
        assert_eq!(reconcile.state, ExternalBatchJobState::Running);
        assert_eq!(cancel.state, ExternalBatchJobState::Cancelled);
    }

    #[test]
    fn confined_process_rejects_digest_swap_and_output_flood() {
        let temp = tempfile::tempdir().expect("temporary fake Slurm dir");
        let script = write_fake_slurm(&temp);
        let mut swapped = fake_slurm_profile(&script);
        swapped.submit.expected_digest_blake3 = blake3::hash(b"different executable").to_hex().to_string();
        let digest_error = ConfiguredExternalBatchDispatcher::new(&swapped)
            .submit(&fixture_operation(ExternalBatchOperationKind::Submit, None), TEST_OBSERVED_UNIX_S)
            .expect_err("digest mismatch rejects execution");
        let mut flooded = fake_slurm_profile(&script);
        flooded.observe.args = vec!["flood".to_string()];
        flooded.observe.stdout_limit_bytes = 32;
        let flood_error = ConfiguredExternalBatchDispatcher::new(&flooded)
            .observe(&fixture_operation(ExternalBatchOperationKind::Observe, Some("4242")), TEST_OBSERVED_UNIX_S)
            .expect_err("bounded output rejects flood");
        let mut timed_out = fake_slurm_profile(&script);
        timed_out.observe.args = vec!["hang".to_string()];
        timed_out.observe.timeout_secs = 1;
        let timeout_error = ConfiguredExternalBatchDispatcher::new(&timed_out)
            .observe(&fixture_operation(ExternalBatchOperationKind::Observe, Some("4242")), TEST_OBSERVED_UNIX_S)
            .expect_err("hung provider command times out");
        let mut disabled = fake_slurm_profile(&script);
        disabled.allowed_operations.retain(|operation| *operation != ExternalBatchOperationKind::Cancel);
        let disabled_error = ConfiguredExternalBatchDispatcher::new(&disabled)
            .cancel(&fixture_operation(ExternalBatchOperationKind::Cancel, Some("4242")), TEST_OBSERVED_UNIX_S)
            .expect_err("disabled operation is rejected before execution");

        assert_eq!(digest_error, "external-batch-executable-digest-mismatch");
        assert_eq!(flood_error, "external-batch-process-output-limit-exceeded");
        assert_eq!(timeout_error, "external-batch-process-timeout");
        assert_eq!(disabled_error, "external-batch-operation-not-allowed");
    }

    #[test]
    fn slurm_adapter_rejects_malformed_state_without_leaking_output() {
        let temp = tempfile::tempdir().expect("temporary fake Slurm dir");
        let script = write_fake_slurm(&temp);
        let mut profile = fake_slurm_profile(&script);
        profile.observe.args = vec!["malformed".to_string()];
        let error = ConfiguredExternalBatchDispatcher::new(&profile)
            .observe(&fixture_operation(ExternalBatchOperationKind::Observe, Some("4242")), TEST_OBSERVED_UNIX_S)
            .expect_err("malformed scheduler state rejected");

        assert_eq!(error, "external-batch-slurm-state-malformed");
        assert!(!error.contains("NOT_A_SLURM_STATE"));
    }

    #[test]
    fn slurm_resource_projection_is_provider_specific_only_at_adapter_boundary() {
        let temp = tempfile::tempdir().expect("temporary fake Slurm dir");
        let script = write_fake_slurm(&temp);
        let profile = fake_slurm_profile(&script);
        let operation = fixture_operation(ExternalBatchOperationKind::Submit, None);
        let args = slurm_submit_args(&profile, &operation).expect("Slurm arguments project");

        assert!(args.iter().any(|arg| arg == "--cpus-per-task=2"));
        assert!(args.iter().any(|arg| arg == "--gres=nvidia-sm90:1"));
        assert!(args.iter().any(|arg| arg == "--licenses=linker-seat:1"));
        assert!(!args.iter().any(|arg| arg.contains("credential")));
    }
}
