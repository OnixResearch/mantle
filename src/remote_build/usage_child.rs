//! Private, builder-local executor process boundary. Only non-secret configuration
//! is serialized; the child resolves the configured secret provider independently.
use std::borrow::Cow;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::time::Duration;

use serde::Deserialize;
use serde::Serialize;

use super::ConcreteBuildRequest;
use super::RemoteExecutablePlan;
use super::RemoteExecutionOutcome;
use super::RemoteInputUpload;
use super::RemoteLocalBuildExecutor;
use super::usage::RemoteUsageCollector;
use super::usage::RemoteUsageCompletion;

const MAX_EXECUTOR_INPUT_BYTES: usize = 16_777_216;
const CHILD_SCHEMA: &str = "mantle-private-usage-executor-v1";

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExecutorChildInput<'a> {
    schema: String,
    endpoint_id: String,
    coordinator_state_dir: PathBuf,
    state_dir: PathBuf,
    backend: String,
    output_dir: PathBuf,
    store_prefix: String,
    expected_public_key: String,
    trusted_public_keys: Vec<String>,
    secret_manifest: PathBuf,
    secret_profile: String,
    secret_provider: String,
    request: Cow<'a, ConcreteBuildRequest>,
    plan: Cow<'a, RemoteExecutablePlan>,
    upload: Cow<'a, RemoteInputUpload>,
    created_unix_s: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
enum ExecutorChildOutput {
    Success {
        outcome: RemoteExecutionOutcome,
    },
    /// Failure text stays in the child; only a validated debug reference crosses.
    Failed {
        worker_debug: Option<crate::remote_failure_debug::RemoteWorkerFailureDebugReport>,
    },
}

pub(super) struct MeasuredExecution {
    pub outcome: Result<RemoteExecutionOutcome, String>,
    pub completion: RemoteUsageCompletion,
}

/// Invoke a dedicated executor *on the builder host*, not a transport process.
/// Complete measurements exist only after exact-child wait4, EOF on both
/// pipes, successful result decoding, and existing output validation. The host
/// must bind the final transfer-enriched result before signing.
pub(super) fn execute_measured(
    executor: &RemoteLocalBuildExecutor,
    request: &ConcreteBuildRequest,
    plan: &RemoteExecutablePlan,
    upload: &RemoteInputUpload,
    created_unix_s: u64,
    secrets: &crate::remote_service_secrets::RemoteServiceSecretRequest,
    usage_cgroup_root: Option<&std::path::Path>,
) -> Result<MeasuredExecution, String> {
    let binding = request.production_attempt.as_ref().ok_or("remote-usage-attempt-missing")?;
    if executor.trust_unsigned || executor.verbose {
        return Err("remote-usage-executor-production-configuration-invalid".into());
    }
    // The input is an anonymous host-owned file, not a pipe which could block
    // indefinitely before the bounded output/wait collector starts running.
    let input = ExecutorChildInput {
        schema: CHILD_SCHEMA.into(),
        endpoint_id: executor.endpoint_id.clone(),
        coordinator_state_dir: executor.coordinator_state_dir.clone(),
        state_dir: executor.state_dir.clone(),
        backend: executor.backend.as_str().into(),
        output_dir: executor.output_dir.clone(),
        store_prefix: executor.store_prefix.clone(),
        expected_public_key: executor.keypair.verifying_key.to_string(),
        trusted_public_keys: executor.trusted_keys.iter().map(ToString::to_string).collect(),
        secret_manifest: secrets.manifest_path.clone(),
        secret_profile: secrets.profile.clone(),
        secret_provider: secrets.provider.clone(),
        request: Cow::Borrowed(request),
        plan: Cow::Borrowed(plan),
        upload: Cow::Borrowed(upload),
        created_unix_s,
    };
    let mut input_file = tempfile::tempfile().map_err(|_| "remote-usage-input-file")?;
    {
        let mut bounded = BoundedWriter {
            inner: &mut input_file,
            remaining: MAX_EXECUTOR_INPUT_BYTES,
        };
        serde_json::to_writer(&mut bounded, &input).map_err(|_| "remote-usage-input-limit-or-encoding")?;
    }
    input_file.seek(SeekFrom::Start(0)).map_err(|_| "remote-usage-input-seek")?;
    let executable = std::env::current_exe().map_err(|_| "remote-usage-executable-unavailable")?;
    let scratch = tempfile::Builder::new()
        .prefix("mantle-attempt-scratch-")
        .tempdir()
        .map_err(|_| "remote-usage-scratch-unavailable")?;
    let mut command = Command::new(executable);
    command
        .args(["remote", "usage-executor"])
        .stdin(Stdio::from(input_file))
        .env("TMPDIR", scratch.path());
    // Inherit the production worker's process group: cancellation belongs to
    // that host-controlled worker group, not an unrelated SSH/client process.
    let mut collector =
        RemoteUsageCollector::begin(binding, &request.request_id, Duration::from_secs(request.build_time_limit_secs))?
            .with_scratch_root(scratch.path());
    if let Some(root) = usage_cgroup_root {
        collector =
            collector.with_cgroup_root(root, request.resource_requirements.as_ref().map(|r| r.quantities.memory_bytes));
    }
    let completion = collector.spawn(&mut command)?.finish()?;
    let outcome = if !completion.status().success() || completion.stdout_truncated() {
        Err("remote-usage-executor-failed".into())
    } else {
        match serde_json::from_slice::<ExecutorChildOutput>(completion.stdout()) {
            Ok(ExecutorChildOutput::Success { outcome }) => {
                super::validate_remote_execution_outcome(&outcome, request, plan)
                    .and_then(|()| {
                        super::validate_remote_execution_output_pathinfos(
                            &outcome.outputs,
                            executor.keypair.verifying_key.name(),
                            &request.store_prefix,
                        )
                    })
                    .map(|()| outcome)
            }
            Ok(ExecutorChildOutput::Failed { worker_debug }) => Err(redacted_executor_failure(worker_debug)),
            // Fixed redacted failure, including OOM-killed children which
            // never got an opportunity to serialize a response.
            _ => Err("remote-usage-executor-failed".into()),
        }
    };
    Ok(MeasuredExecution { outcome, completion })
}

/// The fixed failure, plus the child's debug reference only if it revalidates here.
fn redacted_executor_failure(
    worker_debug: Option<crate::remote_failure_debug::RemoteWorkerFailureDebugReport>,
) -> String {
    let reference = worker_debug.and_then(crate::remote_failure_debug::RemoteWorkerFailureDebugReport::validated);
    let mut reason = String::from("remote-usage-executor-failed");
    if let Some(reference) = reference {
        reason.push_str(&reference.render_fields());
    }
    debug_assert!(reason.starts_with("remote-usage-executor-failed"));
    reason
}

struct BoundedWriter<W> {
    inner: W,
    remaining: usize,
}
impl<W: Write> Write for BoundedWriter<W> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.remaining {
            return Err(std::io::Error::other("private executor wire limit"));
        }
        let written = self.inner.write(bytes)?;
        self.remaining -= written;
        Ok(written)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

/// Hidden `remote usage-executor` entrypoint. No credentials are accepted in
/// JSON, argv or environment. Failure output is deliberately a fixed variant.
pub fn run_stdio_executor_child() -> Result<(), String> {
    let operation_plan = super::operation::plan(super::operation::RemoteOperation::UsageExecutor)
        .map_err(|_| "remote-usage-executor-effect-plan-invalid")?;
    let mut bytes = Vec::new();
    std::io::stdin()
        .lock()
        .take(MAX_EXECUTOR_INPUT_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "remote-usage-executor-input-invalid")?;
    if bytes.len() > MAX_EXECUTOR_INPUT_BYTES {
        return Err("remote-usage-executor-input-too-large".into());
    }
    let input: ExecutorChildInput<'_> =
        serde_json::from_slice(&bytes).map_err(|_| "remote-usage-executor-input-invalid")?;
    drop(bytes);
    let response = match execute_child(input, &operation_plan) {
        Ok(outcome) => ExecutorChildOutput::Success { outcome },
        Err(reason) => ExecutorChildOutput::Failed {
            worker_debug: crate::remote_failure_debug::parse_remote_worker_failure_debug_report(&reason),
        },
    };
    let stdout = std::io::stdout();
    let mut bounded = BoundedWriter {
        inner: stdout.lock(),
        remaining: super::usage::MAX_USAGE_STDOUT_BYTES,
    };
    serde_json::to_writer(&mut bounded, &response).map_err(|_| "remote-usage-executor-output-invalid")?;
    bounded.flush().map_err(|_| "remote-usage-executor-output-invalid".into())
}

fn execute_child(
    input: ExecutorChildInput<'_>,
    operation_plan: &super::operation::RemoteOperationPlan,
) -> Result<RemoteExecutionOutcome, String> {
    if input.schema != CHILD_SCHEMA
        || input.request.production_attempt.is_none()
        || input.request.request_id != input.plan.request_id
        || input.request.request_id != input.upload.request_id
        || input.request.store_prefix != input.store_prefix
    {
        return Err("remote-usage-executor-binding-invalid".into());
    }
    super::validate_remote_execution_plan(&input.plan)?;
    let keys = crate::remote_service_secrets::resolve_remote_service_keys_bounded(
        &crate::remote_service_secrets::RemoteServiceSecretRequest {
            manifest_path: input.secret_manifest,
            profile: input.secret_profile,
            provider: input.secret_provider,
        },
    )
    .map_err(|_| "remote-usage-executor-secret-unavailable")?;
    if keys.result_signing_key.verifying_key.to_string() != input.expected_public_key {
        return Err("remote-usage-executor-key-changed".into());
    }
    let trusted_keys = input
        .trusted_public_keys
        .iter()
        .map(|key| {
            nix_compat::narinfo::VerifyingKey::parse(key).map_err(|_| "remote-usage-executor-public-key-invalid")
        })
        .collect::<Result<Vec<_>, _>>()?;
    let executor = RemoteLocalBuildExecutor {
        endpoint_id: input.endpoint_id,
        coordinator_state_dir: input.coordinator_state_dir,
        state_dir: input.state_dir,
        backend: input.backend.parse().map_err(|_| "remote-usage-executor-backend-invalid")?,
        output_dir: input.output_dir,
        store_prefix: input.store_prefix,
        keypair: keys.result_signing_key,
        trusted_keys,
        trust_unsigned: false,
        verbose: false,
    };
    let outcome = executor.execute_at_unix_s(&input.request, &input.plan, &input.upload, input.created_unix_s)?;
    super::validate_remote_execution_outcome(&outcome, &input.request, &input.plan)?;
    super::validate_remote_execution_output_pathinfos(
        &outcome.outputs,
        &executor.signing_key_id(),
        &input.request.store_prefix,
    )?;
    super::operation::classify(operation_plan, super::operation::RemoteOperationFacts::Outputs {
        expected: input.request.expected_outputs.len(),
        admitted: outcome.outputs.len(),
        unevidenced: outcome
            .outputs
            .iter()
            .filter(|output| {
                !super::is_blake3_hex_digest(&output.artifact_attestation_digest_blake3) || output.path_info.is_none()
            })
            .count(),
    })
    .map_err(|_| "remote-usage-executor-output-evidence-invalid")?;
    Ok(outcome)
}
