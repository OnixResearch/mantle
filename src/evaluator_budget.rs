use std::ffi::OsString;
use std::fs;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Child;
use std::process::Command;
use std::process::ExitStatus;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use crunch_eval_budget_core::ABSOLUTE_PROTOCOL_BYTES_MAX;
use crunch_eval_budget_core::EvaluationBudgetPolicy;
use crunch_eval_budget_core::EvaluationBudgetReport;
use crunch_eval_budget_core::EvaluationMode;
use crunch_eval_budget_core::EvaluationOperation;
use crunch_eval_budget_core::EvaluatorObservation;
use crunch_eval_budget_core::EvaluatorWorkerRequest;
use crunch_eval_budget_core::EvaluatorWorkerResponse;
use crunch_eval_budget_core::FRAME_HEADER_BYTES;
use crunch_eval_budget_core::ImportDescriptor;
use crunch_eval_budget_core::MechanismSupport;
use crunch_eval_budget_core::MetricFact;
use crunch_eval_budget_core::MetricFactStatus;
use crunch_eval_budget_core::ProcessObservation;
use crunch_eval_budget_core::REQUEST_SCHEMA;
use crunch_eval_budget_core::RESPONSE_SCHEMA;
use crunch_eval_budget_core::TeardownFacts;
use crunch_eval_budget_core::TerminalDisposition;
use crunch_eval_budget_core::TerminalFacts;
use crunch_eval_budget_core::WorkerExitKind;
use crunch_eval_budget_core::WorkerResponseStatus;
use crunch_eval_budget_core::build_report;
use crunch_eval_budget_core::decode_frame_header;
use crunch_eval_budget_core::frame_payload;
use crunch_eval_budget_core::metric_fact;
use crunch_eval_budget_core::prepare_request;
use crunch_eval_budget_core::reject_trailing_data;
use crunch_eval_budget_core::truncate_diagnostics;
use crunch_eval_budget_core::validate_policy;
use crunch_eval_budget_core::validate_prepared_request;
use crunch_eval_budget_core::validate_worker_response;

use crate::build_cmd::build_import_paths;
use crate::errors::RunError;

const POLICY_FILE_BYTES_MAX: u64 = 1_048_576;
const WORKER_STATUS_POLL_INTERVAL_MS: u64 = 5;
const WORKER_IDENTITY: &str = "mantle-linked-nickel-worker-v1";
const MEMORY_LIMIT_EXIT_CODE: i32 = 78;
const CPU_SECONDS_MILLISECOND_DIVISOR: u64 = 1_000;
const CPU_HARD_LIMIT_EXTRA_SECONDS: u64 = 1;
const LINUX_MAX_RSS_UNIT_BYTES: u64 = 1_024;
const REPORT_TEMP_SUFFIX: &str = ".mantle-evaluation-report.tmp";
#[cfg(debug_assertions)]
const TEST_WORKER_FIXTURE_ENV: &str = "MANTLE_TEST_EVALUATOR_WORKER_FIXTURE";
#[cfg(debug_assertions)]
const TEST_CANCEL_AFTER_MS_ENV: &str = "MANTLE_TEST_EVALUATOR_CANCEL_AFTER_MS";
#[cfg(debug_assertions)]
const TEST_FORCE_REAP_FAILURE_ENV: &str = "MANTLE_TEST_EVALUATOR_FORCE_REAP_FAILURE";
#[cfg(debug_assertions)]
const FIXTURE_OUTPUT_EXTRA_BYTES: u64 = 1_024;
#[cfg(debug_assertions)]
const FIXTURE_SLEEP_EXTRA_MS: u64 = 100;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvaluationSelection {
    WholeValue,
    SelectedRoots(Vec<String>),
    AllRoots,
}

#[derive(Debug)]
pub struct BudgetedEvaluation {
    pub output_json: Option<String>,
    pub report: EvaluationBudgetReport,
}

#[derive(Debug)]
struct BoundedCapture {
    bytes: Vec<u8>,
    truncated: bool,
}

#[derive(Debug)]
struct WorkerEvaluation {
    output_json: String,
    discovered_root_count: Option<u32>,
    selected_root_count: u32,
    explicit_top_level_root_force_count: u32,
}

#[derive(Debug)]
struct WorkerRun {
    response: Option<EvaluatorWorkerResponse>,
    facts: TerminalFacts,
    wall_time_ms: u64,
    stderr: BoundedCapture,
    protocol_error: Option<String>,
}

#[derive(Debug)]
struct WaitResult {
    status: ExitStatus,
    cancellation_requested: bool,
    deadline_exceeded: bool,
    terminate_sent: bool,
    kill_sent: bool,
    reaped: bool,
}

pub fn run_budgeted_file(
    file: &Path,
    import_paths: &[PathBuf],
    policy_path: &Path,
    report_path: &Path,
    selection: EvaluationSelection,
) -> Result<BudgetedEvaluation, RunError> {
    let policy = read_policy(policy_path)?;
    let support = host_mechanism_support();
    let policy_ref = validate_policy(&policy, &support).map_err(budget_error)?;
    let request = prepare_file_request(file, import_paths, policy.clone(), &policy_ref, selection)?;
    verify_import_surface(&request)?;
    let cancellation = Arc::new(AtomicBool::new(false));
    schedule_test_cancellation(&policy, cancellation.clone())?;
    let result = match policy.mode {
        EvaluationMode::ObserveOnly => run_observe_only(&request, &policy_ref)?,
        EvaluationMode::Enforce => run_enforced(request, &policy_ref, cancellation)?,
    };
    write_report(report_path, &result.report)?;
    Ok(result)
}

pub fn worker_main() -> Result<(), RunError> {
    let request_bytes = read_one_frame(io::stdin(), ABSOLUTE_PROTOCOL_BYTES_MAX).map_err(protocol_error)?;
    let request: EvaluatorWorkerRequest = postcard::from_bytes(&request_bytes)
        .map_err(|error| RunError::Internal(format!("evaluation-budget-request-decode:{error}")))?;
    let support = host_mechanism_support();
    let policy_ref = validate_policy(&request.policy, &support).map_err(budget_error)?;
    validate_prepared_request(&request, &request.policy, &policy_ref).map_err(budget_error)?;
    verify_import_surface(&request)?;
    apply_import_confinement(&request.imports)?;
    let response = execute_worker_request(&request)?;
    write_worker_response(&response, request.policy.response_bytes_max)
}

#[cfg(debug_assertions)]
pub fn worker_fixture_main(behavior: &str) -> Result<(), RunError> {
    let request_bytes = read_one_frame(io::stdin(), ABSOLUTE_PROTOCOL_BYTES_MAX).map_err(protocol_error)?;
    let request: EvaluatorWorkerRequest = postcard::from_bytes(&request_bytes)
        .map_err(|error| RunError::Internal(format!("evaluation-budget-fixture-request-decode:{error}")))?;
    match behavior {
        "panic" => panic!("intentional evaluator worker fixture panic"),
        "signal" => {
            unsafe { libc::raise(libc::SIGTERM) };
            Err(RunError::Internal("evaluation worker signal fixture returned".to_string()))
        }
        "memory-limit" => std::process::exit(MEMORY_LIMIT_EXIT_CODE),
        "cpu-exhaustion" => {
            let mut counter = 0_u64;
            loop {
                counter = std::hint::black_box(counter.wrapping_add(1));
            }
        }
        "memory-exhaustion" => {
            let allocation_bytes = request
                .policy
                .peak_rss_bytes_max
                .checked_add(1)
                .ok_or_else(|| RunError::Internal("fixture allocation size overflow".to_string()))?;
            let allocation_bytes = usize::try_from(allocation_bytes)
                .map_err(|_| RunError::Internal("fixture allocation size overflow".to_string()))?;
            let mut allocation = Vec::<u8>::new();
            if allocation.try_reserve_exact(allocation_bytes).is_err() {
                std::process::exit(MEMORY_LIMIT_EXIT_CODE);
            }
            Err(RunError::Internal("memory exhaustion fixture exceeded the configured address space".to_string()))
        }
        "malformed-frame" => write_fixture_bytes(b"not-a-valid-frame"),
        "response-flood" => {
            let byte_count = request
                .policy
                .response_bytes_max
                .checked_add(FIXTURE_OUTPUT_EXTRA_BYTES)
                .ok_or_else(|| RunError::Internal("fixture response size overflow".to_string()))?;
            let byte_count = usize::try_from(byte_count)
                .map_err(|_| RunError::Internal("fixture response size overflow".to_string()))?;
            write_fixture_bytes(&vec![b'x'; byte_count])
        }
        "stderr-path" => {
            let protected =
                request.imports.first().map(|value| value.canonical_path.as_bytes()).unwrap_or(b"no-import-root");
            io::stderr()
                .write_all(protected)
                .map_err(|error| RunError::Internal(format!("writing fixture stderr path: {error}")))?;
            let response = execute_worker_request(&request)?;
            write_worker_response(&response, request.policy.response_bytes_max)
        }
        "stderr-flood" => {
            let byte_count = request
                .policy
                .stderr_bytes_max
                .checked_add(FIXTURE_OUTPUT_EXTRA_BYTES)
                .ok_or_else(|| RunError::Internal("fixture stderr size overflow".to_string()))?;
            let byte_count = usize::try_from(byte_count)
                .map_err(|_| RunError::Internal("fixture stderr size overflow".to_string()))?;
            io::stderr()
                .write_all(&vec![b'e'; byte_count])
                .map_err(|error| RunError::Internal(format!("writing fixture stderr: {error}")))?;
            let response = execute_worker_request(&request)?;
            write_worker_response(&response, request.policy.response_bytes_max)
        }
        "late-success" => {
            let sleep_ms = request
                .policy
                .wall_time_ms_max
                .checked_add(request.policy.shutdown_grace_ms)
                .and_then(|value| value.checked_add(FIXTURE_SLEEP_EXTRA_MS))
                .ok_or_else(|| RunError::Internal("fixture sleep overflow".to_string()))?;
            thread::sleep(Duration::from_millis(sleep_ms));
            let response = execute_worker_request(&request)?;
            write_worker_response(&response, request.policy.response_bytes_max)
        }
        "ignore-terminate" => {
            unsafe { libc::signal(libc::SIGTERM, libc::SIG_IGN) };
            let sleep_ms = request
                .policy
                .wall_time_ms_max
                .checked_add(request.policy.shutdown_grace_ms)
                .and_then(|value| value.checked_add(FIXTURE_SLEEP_EXTRA_MS))
                .ok_or_else(|| RunError::Internal("fixture sleep overflow".to_string()))?;
            thread::sleep(Duration::from_millis(sleep_ms));
            let response = execute_worker_request(&request)?;
            write_worker_response(&response, request.policy.response_bytes_max)
        }
        "success" => {
            let response = execute_worker_request(&request)?;
            write_worker_response(&response, request.policy.response_bytes_max)
        }
        _ => Err(RunError::Internal(format!("unknown evaluator worker fixture: {behavior}"))),
    }
}

#[cfg(not(debug_assertions))]
pub fn worker_fixture_main(_behavior: &str) -> Result<(), RunError> {
    Err(RunError::Internal("evaluator worker fixtures are unavailable in release builds".to_string()))
}

#[cfg(debug_assertions)]
fn write_fixture_bytes(bytes: &[u8]) -> Result<(), RunError> {
    let mut stdout = io::stdout().lock();
    stdout
        .write_all(bytes)
        .map_err(|error| RunError::Internal(format!("writing evaluator fixture output: {error}")))?;
    stdout
        .flush()
        .map_err(|error| RunError::Internal(format!("flushing evaluator fixture output: {error}")))
}

pub fn host_mechanism_support() -> MechanismSupport {
    #[cfg(target_os = "linux")]
    {
        MechanismSupport {
            worker_process: true,
            import_filesystem_confinement: landlock_abi_version().is_some(),
            wall_deadline: true,
            cpu_time_enforcement: true,
            address_space_enforcement: true,
            cpu_time_observation: true,
            peak_rss_observation: true,
            cpu_time_mechanism: "getrusage-self".to_string(),
            memory_enforcement_mechanism: "rlimit-as-address-space".to_string(),
            peak_rss_mechanism: "getrusage-self-maxrss-kibibytes".to_string(),
            import_confinement_mechanism: "landlock-read-file-and-read-dir".to_string(),
        }
    }
    #[cfg(not(target_os = "linux"))]
    MechanismSupport {
        worker_process: true,
        import_filesystem_confinement: false,
        wall_deadline: true,
        cpu_time_enforcement: false,
        address_space_enforcement: false,
        cpu_time_observation: false,
        peak_rss_observation: false,
        cpu_time_mechanism: "unavailable-host-mechanism".to_string(),
        memory_enforcement_mechanism: "unavailable-host-mechanism".to_string(),
        peak_rss_mechanism: "unavailable-host-mechanism".to_string(),
        import_confinement_mechanism: "unavailable-host-mechanism".to_string(),
    }
}

#[cfg(target_os = "linux")]
const LANDLOCK_CREATE_RULESET_VERSION: u32 = 1;
#[cfg(target_os = "linux")]
const LANDLOCK_RULE_PATH_BENEATH: u32 = 1;
#[cfg(target_os = "linux")]
const LANDLOCK_ACCESS_FS_READ_FILE: u64 = 1_u64 << 2;
#[cfg(target_os = "linux")]
const LANDLOCK_ACCESS_FS_READ_DIR: u64 = 1_u64 << 3;
#[cfg(target_os = "linux")]
const LANDLOCK_HANDLED_READ_ACCESS: u64 = LANDLOCK_ACCESS_FS_READ_FILE | LANDLOCK_ACCESS_FS_READ_DIR;

#[cfg(target_os = "linux")]
#[repr(C)]
struct LandlockRulesetAttr {
    handled_access_fs: u64,
}

#[cfg(target_os = "linux")]
#[repr(C)]
struct LandlockPathBeneathAttr {
    allowed_access: u64,
    parent_fd: i32,
}

#[cfg(target_os = "linux")]
fn landlock_abi_version() -> Option<i32> {
    let result = unsafe {
        libc::syscall(
            libc::SYS_landlock_create_ruleset,
            std::ptr::null::<LandlockRulesetAttr>(),
            0_usize,
            LANDLOCK_CREATE_RULESET_VERSION,
        )
    };
    i32::try_from(result).ok().filter(|version| *version > 0)
}

#[cfg(target_os = "linux")]
fn apply_import_confinement(imports: &[ImportDescriptor]) -> Result<(), RunError> {
    use std::os::unix::ffi::OsStrExt;

    if landlock_abi_version().is_none() {
        return Err(RunError::Eval("evaluation-budget-unsupported:import-filesystem-confinement".to_string()));
    }
    let ruleset = LandlockRulesetAttr {
        handled_access_fs: LANDLOCK_HANDLED_READ_ACCESS,
    };
    let ruleset_fd = unsafe {
        libc::syscall(libc::SYS_landlock_create_ruleset, &ruleset, std::mem::size_of::<LandlockRulesetAttr>(), 0_u32)
    };
    let ruleset_fd =
        i32::try_from(ruleset_fd).map_err(|_| RunError::Eval("evaluation-budget-landlock-ruleset".to_string()))?;
    if ruleset_fd < 0 {
        return Err(RunError::Eval(format!("evaluation-budget-landlock-ruleset:{}", io::Error::last_os_error())));
    }
    let result = (|| {
        for import in imports {
            let path = std::ffi::CString::new(Path::new(&import.canonical_path).as_os_str().as_bytes())
                .map_err(|_| RunError::Eval("evaluation-budget-invalid-import-path:nul".to_string()))?;
            let parent_fd = unsafe { libc::open(path.as_ptr(), libc::O_PATH | libc::O_CLOEXEC) };
            if parent_fd < 0 {
                return Err(RunError::Eval(format!("evaluation-budget-landlock-open:{}", io::Error::last_os_error())));
            }
            let rule = LandlockPathBeneathAttr {
                allowed_access: LANDLOCK_HANDLED_READ_ACCESS,
                parent_fd,
            };
            let add_result = unsafe {
                libc::syscall(libc::SYS_landlock_add_rule, ruleset_fd, LANDLOCK_RULE_PATH_BENEATH, &rule, 0_u32)
            };
            unsafe { libc::close(parent_fd) };
            if add_result != 0 {
                return Err(RunError::Eval(format!(
                    "evaluation-budget-landlock-add-rule:{}",
                    io::Error::last_os_error()
                )));
            }
        }
        let no_new_privileges = unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) };
        if no_new_privileges != 0 {
            return Err(RunError::Eval(format!(
                "evaluation-budget-landlock-no-new-privileges:{}",
                io::Error::last_os_error()
            )));
        }
        let restrict_result = unsafe { libc::syscall(libc::SYS_landlock_restrict_self, ruleset_fd, 0_u32) };
        if restrict_result != 0 {
            return Err(RunError::Eval(format!("evaluation-budget-landlock-restrict:{}", io::Error::last_os_error())));
        }
        Ok(())
    })();
    unsafe { libc::close(ruleset_fd) };
    result
}

#[cfg(not(target_os = "linux"))]
fn apply_import_confinement(_imports: &[ImportDescriptor]) -> Result<(), RunError> {
    Err(RunError::Eval("evaluation-budget-unsupported:import-filesystem-confinement".to_string()))
}

fn read_policy(path: &Path) -> Result<EvaluationBudgetPolicy, RunError> {
    let metadata = fs::metadata(path)
        .map_err(|error| RunError::Internal(format!("reading evaluation policy metadata: {error}")))?;
    if metadata.len() > POLICY_FILE_BYTES_MAX {
        return Err(RunError::Eval("evaluation-budget-limit-exceeded:policy-file-bytes".to_string()));
    }
    let bytes = fs::read(path).map_err(|error| RunError::Internal(format!("reading evaluation policy: {error}")))?;
    serde_json::from_slice(&bytes).map_err(|error| RunError::Eval(format!("evaluation-budget-policy-decode:{error}")))
}

fn prepare_file_request(
    file: &Path,
    import_paths: &[PathBuf],
    policy: EvaluationBudgetPolicy,
    policy_ref: &str,
    selection: EvaluationSelection,
) -> Result<EvaluatorWorkerRequest, RunError> {
    let source_bytes =
        fs::read(file).map_err(|error| RunError::Internal(format!("reading evaluation source: {error}")))?;
    let source_blake3 = blake3::hash(&source_bytes).to_hex().to_string();
    let imports = canonical_import_descriptors(file, import_paths)?;
    let import_entry_count = count_import_entries(&imports, policy.imported_modules_max)?;
    let source_name = file.file_name().and_then(|name| name.to_str()).unwrap_or("evaluation-input.ncl").to_string();
    let (operation, selected_roots) = match selection {
        EvaluationSelection::WholeValue => (EvaluationOperation::WholeValue, Vec::new()),
        EvaluationSelection::SelectedRoots(roots) => (EvaluationOperation::SelectedRoots, roots),
        EvaluationSelection::AllRoots => (EvaluationOperation::AllRoots, Vec::new()),
    };
    let request = EvaluatorWorkerRequest {
        schema: REQUEST_SCHEMA.to_string(),
        request_ref: String::new(),
        policy_ref: policy_ref.to_string(),
        policy: policy.clone(),
        source_name,
        source_blake3,
        source_bytes,
        imports,
        import_entry_count,
        selected_roots,
        evaluator_id: crunch_eval::EVALUATOR_ID.to_string(),
        evaluator_version: crunch_eval::EVALUATOR_VERSION.to_string(),
        operation,
    };
    prepare_request(request, &policy, policy_ref).map_err(budget_error)
}

fn canonical_import_descriptors(file: &Path, import_paths: &[PathBuf]) -> Result<Vec<ImportDescriptor>, RunError> {
    let mut requested = import_paths.to_vec();
    if let Some(parent) = file.parent() {
        requested.push(parent.to_path_buf());
    }
    let paths = build_import_paths(&requested)?;
    paths.iter().map(canonical_import_descriptor).collect()
}

fn canonical_import_descriptor(path: &OsString) -> Result<ImportDescriptor, RunError> {
    let path = PathBuf::from(path);
    let canonical = fs::canonicalize(&path)
        .map_err(|error| RunError::Eval(format!("evaluation-budget-invalid-import-path:{}:{error}", path.display())))?;
    let canonical_path = canonical
        .to_str()
        .ok_or_else(|| RunError::Eval("evaluation-budget-invalid-import-path:non-utf8".to_string()))?
        .to_string();
    Ok(ImportDescriptor { canonical_path })
}

fn count_import_entries(imports: &[ImportDescriptor], entries_max: u32) -> Result<u32, RunError> {
    let mut pending: Vec<PathBuf> = imports.iter().map(|import| PathBuf::from(&import.canonical_path)).collect();
    let mut entry_count = 0_u32;
    while let Some(directory) = pending.pop() {
        let entries = fs::read_dir(&directory).map_err(|error| {
            RunError::Eval(format!("evaluation-budget-import-read:{}:{error}", directory.display()))
        })?;
        for entry in entries {
            let entry = entry.map_err(|error| RunError::Eval(format!("evaluation-budget-import-read:{error}")))?;
            entry_count = entry_count
                .checked_add(1)
                .ok_or_else(|| RunError::Eval("evaluation-budget-limit-exceeded:import-entries".to_string()))?;
            if entry_count > entries_max {
                return Err(RunError::Eval("evaluation-budget-limit-exceeded:import-entries".to_string()));
            }
            let metadata = fs::symlink_metadata(entry.path())
                .map_err(|error| RunError::Eval(format!("evaluation-budget-import-metadata:{error}")))?;
            if metadata.file_type().is_symlink() {
                return Err(RunError::Eval("evaluation-budget-import-symlink-unsupported".to_string()));
            }
            if metadata.is_dir() {
                pending.push(entry.path());
            }
        }
    }
    Ok(entry_count)
}

fn verify_import_surface(request: &EvaluatorWorkerRequest) -> Result<(), RunError> {
    let observed = count_import_entries(&request.imports, request.policy.imported_modules_max)?;
    if observed != request.import_entry_count {
        return Err(RunError::Eval("evaluation-budget-import-surface-changed".to_string()));
    }
    Ok(())
}

fn run_observe_only(request: &EvaluatorWorkerRequest, policy_ref: &str) -> Result<BudgetedEvaluation, RunError> {
    let source = std::str::from_utf8(&request.source_bytes)
        .map_err(|error| RunError::Eval(format!("evaluation-budget-source-utf8:{error}")))?;
    let import_paths = request_import_paths(request);
    let started = Instant::now();
    let evaluated = evaluate_request_payload(request, source, &import_paths);
    let wall_time_ms = elapsed_millis(started)?;
    let (output_json, error_class, exit_kind, evaluator) = match evaluated {
        Ok(payload) => {
            let evaluator = successful_evaluator_observation(request, &payload, &[])?;
            (Some(payload.output_json), None, WorkerExitKind::SuccessResponse, evaluator)
        }
        Err(error) => {
            let rendered = error.to_string();
            let error_class = evaluation_error_class(&rendered);
            let evaluator = failed_evaluator_observation(Some(&rendered));
            (None, Some(error_class), WorkerExitKind::EvaluationErrorResponse, evaluator)
        }
    };
    let teardown = completed_teardown(output_json.is_some() || error_class.is_some());
    let report = build_report(
        &request.request_ref,
        policy_ref,
        EvaluationMode::ObserveOnly,
        TerminalFacts { teardown, exit_kind },
        observe_only_metrics(request, &evaluator, wall_time_ms),
        Some(evaluator),
        error_class.clone(),
        String::new(),
    )
    .map_err(budget_error)?;
    Ok(BudgetedEvaluation { output_json, report })
}

fn run_enforced(
    request: EvaluatorWorkerRequest,
    policy_ref: &str,
    cancellation: Arc<AtomicBool>,
) -> Result<BudgetedEvaluation, RunError> {
    let worker = run_owned_worker(&request, cancellation)?;
    let response = worker.response.as_ref();
    let evaluator = response.map(|value| value.evaluator.clone());
    let output_json = response.and_then(|value| value.output_json.clone());
    let error_class = response.and_then(|value| value.error_class.clone()).or_else(|| worker.protocol_error.clone());
    let metrics = enforced_metrics(&request, response, worker.wall_time_ms);
    let bounded_stderr = redact_worker_stderr(&worker.stderr.bytes, &request);
    let report = build_report(
        &request.request_ref,
        policy_ref,
        EvaluationMode::Enforce,
        worker.facts,
        metrics,
        evaluator,
        error_class,
        bounded_stderr,
    )
    .map_err(budget_error)?;
    Ok(BudgetedEvaluation { output_json, report })
}

fn run_owned_worker(request: &EvaluatorWorkerRequest, cancellation: Arc<AtomicBool>) -> Result<WorkerRun, RunError> {
    let payload = postcard::to_allocvec(request)
        .map_err(|error| RunError::Internal(format!("evaluation-budget-request-encode:{error}")))?;
    let framed = frame_payload(&payload, request.policy.request_bytes_max).map_err(budget_error)?;
    let mut child = spawn_worker(&request.policy)?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| RunError::Internal("evaluation worker stdout unavailable".to_string()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| RunError::Internal("evaluation worker stderr unavailable".to_string()))?;
    let stdout_limit = request
        .policy
        .response_bytes_max
        .checked_add(
            u64::try_from(FRAME_HEADER_BYTES).map_err(|_| RunError::Internal("frame header overflow".to_string()))?,
        )
        .ok_or_else(|| RunError::Internal("response capture limit overflow".to_string()))?;
    let stdout_reader = spawn_bounded_reader(stdout, stdout_limit);
    let stderr_reader = spawn_bounded_reader(stderr, request.policy.stderr_bytes_max);
    write_child_request(&mut child, &framed)?;
    let started = Instant::now();
    let mut wait = wait_for_worker(&mut child, &request.policy, &cancellation)?;
    apply_test_reap_failure(&mut wait);
    let wall_time_ms = elapsed_millis(started)?;
    let stdout = join_capture(stdout_reader, "stdout")?;
    let stderr = join_capture(stderr_reader, "stderr")?;
    let (response, exit_kind, protocol_error) = classify_worker_output(&wait.status, &stdout, &stderr, request);
    let teardown = TeardownFacts {
        cancellation_requested: wait.cancellation_requested,
        deadline_exceeded: wait.deadline_exceeded,
        terminate_sent: wait.terminate_sent,
        kill_sent: wait.kill_sent,
        reaped: wait.reaped,
        response_present: response.is_some(),
        stderr_bytes: u64::try_from(stderr.bytes.len()).unwrap_or(u64::MAX),
        stderr_truncated: stderr.truncated,
    };
    Ok(WorkerRun {
        response,
        facts: TerminalFacts { teardown, exit_kind },
        wall_time_ms,
        stderr,
        protocol_error,
    })
}

fn spawn_worker(policy: &EvaluationBudgetPolicy) -> Result<Child, RunError> {
    let executable =
        std::env::current_exe().map_err(|error| RunError::Internal(format!("resolving evaluator worker: {error}")))?;
    let mut command = Command::new(executable);
    configure_worker_command(&mut command);
    command.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).env_clear();
    configure_worker_limits(&mut command, policy)?;
    command.spawn().map_err(|error| RunError::Internal(format!("launching evaluator worker: {error}")))
}

#[cfg(target_os = "linux")]
fn configure_worker_limits(command: &mut Command, policy: &EvaluationBudgetPolicy) -> Result<(), RunError> {
    use std::os::unix::process::CommandExt;

    let address_space_bytes = libc::rlim_t::try_from(policy.peak_rss_bytes_max)
        .map_err(|_| RunError::Eval("evaluation-budget-invalid-limit:peak-rss-bytes".to_string()))?;
    let cpu_soft_seconds_u64 = policy.cpu_time_ms_max.saturating_add(CPU_SECONDS_MILLISECOND_DIVISOR.saturating_sub(1))
        / CPU_SECONDS_MILLISECOND_DIVISOR;
    let cpu_hard_seconds_u64 = cpu_soft_seconds_u64
        .checked_add(CPU_HARD_LIMIT_EXTRA_SECONDS)
        .ok_or_else(|| RunError::Eval("evaluation-budget-invalid-limit:cpu-time-ms".to_string()))?;
    let cpu_soft_seconds = libc::rlim_t::try_from(cpu_soft_seconds_u64)
        .map_err(|_| RunError::Eval("evaluation-budget-invalid-limit:cpu-time-ms".to_string()))?;
    let cpu_hard_seconds = libc::rlim_t::try_from(cpu_hard_seconds_u64)
        .map_err(|_| RunError::Eval("evaluation-budget-invalid-limit:cpu-time-ms".to_string()))?;
    unsafe {
        command.pre_exec(move || {
            set_resource_limit(libc::RLIMIT_AS, address_space_bytes, address_space_bytes)?;
            set_resource_limit(libc::RLIMIT_CPU, cpu_soft_seconds, cpu_hard_seconds)?;
            Ok(())
        });
    }
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn configure_worker_limits(_command: &mut Command, _policy: &EvaluationBudgetPolicy) -> Result<(), RunError> {
    Err(RunError::Eval("evaluation-budget-unsupported:worker-limits".to_string()))
}

#[cfg(all(target_os = "linux", target_env = "gnu"))]
type LinuxRlimitResource = libc::__rlimit_resource_t;
#[cfg(all(target_os = "linux", target_env = "musl"))]
type LinuxRlimitResource = libc::c_int;

#[cfg(target_os = "linux")]
fn set_resource_limit(resource: LinuxRlimitResource, soft: libc::rlim_t, hard: libc::rlim_t) -> io::Result<()> {
    let limit = libc::rlimit {
        rlim_cur: soft,
        rlim_max: hard,
    };
    let result = unsafe { libc::setrlimit(resource, &limit) };
    if result != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(all(test, target_os = "linux"))]
mod resource_limit_tests {
    use super::LinuxRlimitResource;

    fn accepts_resource(_resource: LinuxRlimitResource) {}

    #[test]
    fn evaluator_resource_ids_match_the_selected_linux_libc_signature() {
        accepts_resource(libc::RLIMIT_AS);
        accepts_resource(libc::RLIMIT_CPU);
        assert_ne!(libc::RLIMIT_AS, libc::RLIMIT_CPU);
    }
}

fn configure_worker_command(command: &mut Command) {
    #[cfg(debug_assertions)]
    if let Some(behavior) = std::env::var_os(TEST_WORKER_FIXTURE_ENV) {
        command.arg("evaluator-worker-fixture").arg(behavior);
        return;
    }
    command.arg("evaluator-worker");
}

fn write_child_request(child: &mut Child, framed: &[u8]) -> Result<(), RunError> {
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| RunError::Internal("evaluation worker stdin unavailable".to_string()))?;
    stdin
        .write_all(framed)
        .map_err(|error| RunError::Internal(format!("writing evaluator worker request: {error}")))?;
    stdin
        .flush()
        .map_err(|error| RunError::Internal(format!("flushing evaluator worker request: {error}")))?;
    drop(stdin);
    Ok(())
}

fn wait_for_worker(
    child: &mut Child,
    policy: &EvaluationBudgetPolicy,
    cancellation: &AtomicBool,
) -> Result<WaitResult, RunError> {
    let started = Instant::now();
    let deadline = Duration::from_millis(policy.wall_time_ms_max);
    loop {
        let cancellation_requested = cancellation.load(Ordering::Acquire);
        let deadline_exceeded = started.elapsed() >= deadline;
        if cancellation_requested || deadline_exceeded {
            return terminate_and_reap(child, policy, cancellation_requested, deadline_exceeded);
        }
        if let Some(status) =
            child.try_wait().map_err(|error| RunError::Internal(format!("polling evaluator worker: {error}")))?
        {
            return Ok(WaitResult {
                status,
                cancellation_requested: false,
                deadline_exceeded: false,
                terminate_sent: false,
                kill_sent: false,
                reaped: true,
            });
        }
        thread::sleep(Duration::from_millis(WORKER_STATUS_POLL_INTERVAL_MS));
    }
}

fn terminate_and_reap(
    child: &mut Child,
    policy: &EvaluationBudgetPolicy,
    cancellation_requested: bool,
    deadline_exceeded: bool,
) -> Result<WaitResult, RunError> {
    let terminate_sent = send_terminate(child);
    let grace = Duration::from_millis(policy.shutdown_grace_ms);
    let grace_started = Instant::now();
    while grace_started.elapsed() < grace {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| RunError::Internal(format!("polling terminated evaluator worker: {error}")))?
        {
            return Ok(WaitResult {
                status,
                cancellation_requested,
                deadline_exceeded,
                terminate_sent,
                kill_sent: false,
                reaped: true,
            });
        }
        thread::sleep(Duration::from_millis(WORKER_STATUS_POLL_INTERVAL_MS));
    }
    child.kill().map_err(|error| RunError::Internal(format!("killing evaluator worker: {error}")))?;
    let status = child.wait().map_err(|error| RunError::Internal(format!("reaping evaluator worker: {error}")))?;
    Ok(WaitResult {
        status,
        cancellation_requested,
        deadline_exceeded,
        terminate_sent,
        kill_sent: true,
        reaped: true,
    })
}

#[cfg(unix)]
fn send_terminate(child: &Child) -> bool {
    let pid = match i32::try_from(child.id()) {
        Ok(pid) => pid,
        Err(_) => return false,
    };
    unsafe { libc::kill(pid, libc::SIGTERM) == 0 }
}

#[cfg(not(unix))]
fn send_terminate(_child: &Child) -> bool {
    false
}

fn classify_worker_output(
    status: &ExitStatus,
    stdout: &BoundedCapture,
    stderr: &BoundedCapture,
    request: &EvaluatorWorkerRequest,
) -> (Option<EvaluatorWorkerResponse>, WorkerExitKind, Option<String>) {
    if stdout.truncated {
        return (None, WorkerExitKind::ResponseOverflow, Some("evaluation-budget-response-overflow".to_string()));
    }
    let decoded = decode_response(&stdout.bytes, request.policy.response_bytes_max);
    if let Ok(response) = decoded {
        if let Err(error) = validate_worker_response(&response, request) {
            return (None, WorkerExitKind::ProtocolFailure, Some(error.to_string()));
        }
        if !status.success() {
            return (
                None,
                classify_abnormal_status(status, stderr),
                Some("evaluation-budget-worker-nonzero-with-response".to_string()),
            );
        }
        let kind = match response.status {
            WorkerResponseStatus::Success => WorkerExitKind::SuccessResponse,
            WorkerResponseStatus::EvaluationError => WorkerExitKind::EvaluationErrorResponse,
            WorkerResponseStatus::ResponseOverflow => WorkerExitKind::ResponseOverflow,
        };
        return (Some(response), kind, None);
    }
    if status.success() {
        let error = decoded.err().unwrap_or_else(|| "evaluation-budget-response-missing".to_string());
        return (None, WorkerExitKind::ProtocolFailure, Some(error));
    }
    (None, classify_abnormal_status(status, stderr), decoded.err())
}

fn classify_abnormal_status(status: &ExitStatus, stderr: &BoundedCapture) -> WorkerExitKind {
    if status.code() == Some(MEMORY_LIMIT_EXIT_CODE) {
        return WorkerExitKind::MemoryLimitFailure;
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if status.signal() == Some(libc::SIGXCPU) {
            return WorkerExitKind::CpuLimitSignal;
        }
        if status.signal() == Some(libc::SIGABRT)
            && String::from_utf8_lossy(&stderr.bytes).contains("memory allocation")
        {
            return WorkerExitKind::MemoryLimitFailure;
        }
        if status.signal().is_some() {
            return WorkerExitKind::Signal;
        }
    }
    WorkerExitKind::Crash
}

fn execute_worker_request(request: &EvaluatorWorkerRequest) -> Result<EvaluatorWorkerResponse, RunError> {
    let source = std::str::from_utf8(&request.source_bytes)
        .map_err(|error| RunError::Eval(format!("evaluation-budget-source-utf8:{error}")))?;
    let import_paths = request_import_paths(request);
    let started = Instant::now();
    let evaluated = evaluate_request_payload(request, source, &import_paths);
    let wall_time_ms = elapsed_millis(started)?;
    let process = process_observation(wall_time_ms)?;
    let (status, output_json, error_class, diagnostics, evaluator) = match evaluated {
        Ok(payload) => {
            let diagnostics = Vec::new();
            let evaluator = successful_evaluator_observation(request, &payload, &diagnostics)?;
            (WorkerResponseStatus::Success, Some(payload.output_json), None, diagnostics, evaluator)
        }
        Err(error) => {
            let rendered = error.to_string();
            let error_class = evaluation_error_class(&rendered);
            let diagnostics = vec![rendered];
            let (diagnostics, _) =
                truncate_diagnostics(&diagnostics, request.policy.diagnostics_max, request.policy.diagnostic_bytes_max)
                    .map_err(budget_error)?;
            let evaluator = failed_evaluator_observation(diagnostics.first());
            (WorkerResponseStatus::EvaluationError, None, Some(error_class), diagnostics, evaluator)
        }
    };
    Ok(EvaluatorWorkerResponse {
        schema: RESPONSE_SCHEMA.to_string(),
        request_ref: request.request_ref.clone(),
        policy_ref: request.policy_ref.clone(),
        worker_identity: WORKER_IDENTITY.to_string(),
        status,
        output_json,
        error_class,
        diagnostics,
        evaluator,
        process,
    })
}

fn evaluation_error_class(rendered: &str) -> String {
    const LIMIT_CLASSES: [&str; 2] = [
        "evaluation-budget-limit-exceeded:discovered-roots",
        "evaluation-budget-limit-exceeded:selected-roots",
    ];
    for class in LIMIT_CLASSES {
        if rendered.contains(class) {
            return class.to_string();
        }
    }
    "nickel-evaluation-error".to_string()
}

fn evaluate_request_payload(
    request: &EvaluatorWorkerRequest,
    source: &str,
    import_paths: &[OsString],
) -> Result<WorkerEvaluation, crunch_eval::Error> {
    match request.operation {
        EvaluationOperation::WholeValue => {
            let output_json = crunch_eval::evaluate_source_to_json(source, import_paths, &request.source_name)?;
            Ok(WorkerEvaluation {
                output_json,
                discovered_root_count: None,
                selected_root_count: 0,
                explicit_top_level_root_force_count: 0,
            })
        }
        EvaluationOperation::SelectedRoots | EvaluationOperation::AllRoots => {
            evaluate_root_request(request, source, import_paths)
        }
    }
}

fn evaluate_root_request(
    request: &EvaluatorWorkerRequest,
    source: &str,
    import_paths: &[OsString],
) -> Result<WorkerEvaluation, crunch_eval::Error> {
    use crunch_eval::session::EvaluationSession;
    use crunch_eval::session::RootShape;

    let mut session = EvaluationSession::open_named_str(source, import_paths, &request.source_name)?;
    let discovered_root_count = u32::try_from(session.root_labels().len())
        .map_err(|_| crunch_eval::Error::Boundary("discovered root count overflow".to_string()))?;
    if discovered_root_count > request.policy.discovered_roots_max {
        return Err(crunch_eval::Error::Boundary("evaluation-budget-limit-exceeded:discovered-roots".to_string()));
    }
    if matches!(request.operation, EvaluationOperation::AllRoots)
        && discovered_root_count > request.policy.selected_roots_max
    {
        return Err(crunch_eval::Error::Boundary("evaluation-budget-limit-exceeded:selected-roots".to_string()));
    }
    let shape = session.shape().clone();
    let roots: Vec<(String, serde_json::Value)> = match request.operation {
        EvaluationOperation::SelectedRoots => {
            let mut values = Vec::with_capacity(request.selected_roots.len());
            for root in &request.selected_roots {
                values.push((root.clone(), session.force_root(root)?));
            }
            values
        }
        EvaluationOperation::AllRoots => session.force_all_roots()?,
        EvaluationOperation::WholeValue => unreachable!("whole-value requests do not enter root evaluation"),
    };
    let selected_root_count = u32::try_from(roots.len())
        .map_err(|_| crunch_eval::Error::Boundary("selected root count overflow".to_string()))?;
    let explicit_top_level_root_force_count = session.explicit_force_count();
    let value = match request.operation {
        EvaluationOperation::SelectedRoots => root_object(&roots),
        EvaluationOperation::AllRoots => match shape {
            RootShape::Single => roots
                .first()
                .map(|(_, value)| value.clone())
                .ok_or_else(|| crunch_eval::Error::Boundary("single root result was empty".to_string()))?,
            RootShape::Array => serde_json::Value::Array(roots.iter().map(|(_, value)| value.clone()).collect()),
            RootShape::Record => root_object(&roots),
        },
        EvaluationOperation::WholeValue => unreachable!("whole-value requests do not enter root rendering"),
    };
    let output_json = serde_json::to_string_pretty(&value)
        .map_err(|error| crunch_eval::Error::Serde(format!("serializing root result: {error}")))?;
    Ok(WorkerEvaluation {
        output_json,
        discovered_root_count: Some(discovered_root_count),
        selected_root_count,
        explicit_top_level_root_force_count,
    })
}

fn root_object(roots: &[(String, serde_json::Value)]) -> serde_json::Value {
    let mut object = serde_json::Map::with_capacity(roots.len());
    for (label, value) in roots {
        object.insert(label.clone(), value.clone());
    }
    serde_json::Value::Object(object)
}

fn request_import_paths(request: &EvaluatorWorkerRequest) -> Vec<OsString> {
    request.imports.iter().map(|import| OsString::from(&import.canonical_path)).collect()
}

fn successful_evaluator_observation(
    _request: &EvaluatorWorkerRequest,
    payload: &WorkerEvaluation,
    diagnostics: &[String],
) -> Result<EvaluatorObservation, RunError> {
    let (diagnostic_count, diagnostic_bytes) = diagnostic_facts(diagnostics)?;
    Ok(EvaluatorObservation {
        imported_module_count: None,
        discovered_root_count: payload.discovered_root_count,
        selected_root_count: payload.selected_root_count,
        explicit_top_level_root_force_count: payload.explicit_top_level_root_force_count,
        actual_nonselected_evaluation_count: None,
        diagnostic_count,
        diagnostic_bytes,
    })
}

fn failed_evaluator_observation(error: Option<&String>) -> EvaluatorObservation {
    let diagnostic_count = if error.is_some() { 1 } else { 0 };
    let diagnostic_bytes = error.map(|value| u64::try_from(value.len()).unwrap_or(u64::MAX)).unwrap_or(0);
    EvaluatorObservation {
        imported_module_count: None,
        discovered_root_count: None,
        selected_root_count: 0,
        explicit_top_level_root_force_count: 0,
        actual_nonselected_evaluation_count: None,
        diagnostic_count,
        diagnostic_bytes,
    }
}

fn diagnostic_facts(diagnostics: &[String]) -> Result<(u32, u64), RunError> {
    let diagnostic_count = u32::try_from(diagnostics.len())
        .map_err(|_| RunError::Internal("evaluation diagnostic count overflow".to_string()))?;
    let diagnostic_bytes = diagnostics.iter().try_fold(0_u64, |total, diagnostic| {
        let bytes = u64::try_from(diagnostic.len())
            .map_err(|_| RunError::Internal("evaluation diagnostic bytes overflow".to_string()))?;
        total
            .checked_add(bytes)
            .ok_or_else(|| RunError::Internal("evaluation diagnostic bytes overflow".to_string()))
    })?;
    Ok((diagnostic_count, diagnostic_bytes))
}

#[cfg(target_os = "linux")]
fn process_observation(wall_time_ms: u64) -> Result<ProcessObservation, RunError> {
    let mut usage = std::mem::MaybeUninit::<libc::rusage>::zeroed();
    let result = unsafe { libc::getrusage(libc::RUSAGE_SELF, usage.as_mut_ptr()) };
    if result != 0 {
        return Err(RunError::Internal(format!(
            "observing evaluator worker resources: {}",
            io::Error::last_os_error()
        )));
    }
    let usage = unsafe { usage.assume_init() };
    let cpu_time_ms = timeval_millis(usage.ru_utime)?
        .checked_add(timeval_millis(usage.ru_stime)?)
        .ok_or_else(|| RunError::Internal("evaluator CPU time overflow".to_string()))?;
    let peak_rss_kib = u64::try_from(usage.ru_maxrss)
        .map_err(|_| RunError::Internal("evaluator peak RSS was negative".to_string()))?;
    let peak_rss_bytes = peak_rss_kib
        .checked_mul(LINUX_MAX_RSS_UNIT_BYTES)
        .ok_or_else(|| RunError::Internal("evaluator peak RSS overflow".to_string()))?;
    Ok(ProcessObservation {
        wall_time_ms,
        cpu_time_ms: Some(cpu_time_ms),
        peak_rss_bytes: Some(peak_rss_bytes),
    })
}

#[cfg(not(target_os = "linux"))]
fn process_observation(wall_time_ms: u64) -> Result<ProcessObservation, RunError> {
    Ok(ProcessObservation {
        wall_time_ms,
        cpu_time_ms: None,
        peak_rss_bytes: None,
    })
}

#[cfg(target_os = "linux")]
fn timeval_millis(value: libc::timeval) -> Result<u64, RunError> {
    const MICROSECONDS_PER_MILLISECOND: u64 = 1_000;
    const SECONDS_TO_MILLISECONDS: u64 = 1_000;
    let seconds =
        u64::try_from(value.tv_sec).map_err(|_| RunError::Internal("negative evaluator CPU seconds".to_string()))?;
    let microseconds = u64::try_from(value.tv_usec)
        .map_err(|_| RunError::Internal("negative evaluator CPU microseconds".to_string()))?;
    seconds
        .checked_mul(SECONDS_TO_MILLISECONDS)
        .and_then(|millis| millis.checked_add(microseconds / MICROSECONDS_PER_MILLISECOND))
        .ok_or_else(|| RunError::Internal("evaluator CPU time overflow".to_string()))
}

fn write_worker_response(response: &EvaluatorWorkerResponse, bytes_max: u64) -> Result<(), RunError> {
    let encoded = postcard::to_allocvec(response)
        .map_err(|error| RunError::Internal(format!("evaluation-budget-response-encode:{error}")))?;
    let framed = match frame_payload(&encoded, bytes_max) {
        Ok(framed) => framed,
        Err(_) => {
            let overflow = overflow_response(response);
            let encoded = postcard::to_allocvec(&overflow)
                .map_err(|error| RunError::Internal(format!("evaluation-budget-overflow-response-encode:{error}")))?;
            frame_payload(&encoded, bytes_max).map_err(budget_error)?
        }
    };
    let mut stdout = io::stdout().lock();
    stdout
        .write_all(&framed)
        .map_err(|error| RunError::Internal(format!("writing evaluation worker response: {error}")))?;
    stdout
        .flush()
        .map_err(|error| RunError::Internal(format!("flushing evaluation worker response: {error}")))
}

fn overflow_response(response: &EvaluatorWorkerResponse) -> EvaluatorWorkerResponse {
    let mut overflow = response.clone();
    overflow.status = WorkerResponseStatus::ResponseOverflow;
    overflow.output_json = None;
    overflow.error_class = Some("evaluation-budget-response-overflow".to_string());
    overflow.diagnostics.clear();
    overflow.evaluator.diagnostic_count = 0;
    overflow.evaluator.diagnostic_bytes = 0;
    overflow
}

fn decode_response(bytes: &[u8], bytes_max: u64) -> Result<EvaluatorWorkerResponse, String> {
    let payload = decode_one_frame(bytes, bytes_max).map_err(|error| error.to_string())?;
    let response: EvaluatorWorkerResponse =
        postcard::from_bytes(payload).map_err(|error| format!("evaluation-budget-response-decode:{error}"))?;
    if response.schema != RESPONSE_SCHEMA {
        return Err("evaluation-budget-invalid-response-schema".to_string());
    }
    Ok(response)
}

fn read_one_frame(mut reader: impl Read, bytes_max: u64) -> Result<Vec<u8>, crunch_eval_budget_core::BudgetError> {
    let mut header = [0_u8; FRAME_HEADER_BYTES];
    reader
        .read_exact(&mut header)
        .map_err(|_| crunch_eval_budget_core::BudgetError::FrameHeaderIncomplete)?;
    let payload_len = decode_frame_header(&header, bytes_max)?;
    let mut payload = vec![0_u8; payload_len];
    reader
        .read_exact(&mut payload)
        .map_err(|_| crunch_eval_budget_core::BudgetError::FrameHeaderIncomplete)?;
    let mut trailing = [0_u8; 1];
    let trailing_count = reader.read(&mut trailing).map_err(|_| crunch_eval_budget_core::BudgetError::TrailingData)?;
    reject_trailing_data(trailing_count)?;
    Ok(payload)
}

fn decode_one_frame(bytes: &[u8], bytes_max: u64) -> Result<&[u8], crunch_eval_budget_core::BudgetError> {
    if bytes.len() < FRAME_HEADER_BYTES {
        return Err(crunch_eval_budget_core::BudgetError::FrameHeaderIncomplete);
    }
    let payload_len = decode_frame_header(&bytes[..FRAME_HEADER_BYTES], bytes_max)?;
    let frame_len = FRAME_HEADER_BYTES
        .checked_add(payload_len)
        .ok_or(crunch_eval_budget_core::BudgetError::FrameLengthOverflow)?;
    if bytes.len() < frame_len {
        return Err(crunch_eval_budget_core::BudgetError::FrameHeaderIncomplete);
    }
    reject_trailing_data(bytes.len().saturating_sub(frame_len))?;
    Ok(&bytes[FRAME_HEADER_BYTES..frame_len])
}

fn spawn_bounded_reader(
    mut reader: impl Read + Send + 'static,
    bytes_max: u64,
) -> thread::JoinHandle<Result<BoundedCapture, io::Error>> {
    thread::spawn(move || {
        let limit = usize::try_from(bytes_max).unwrap_or(usize::MAX);
        let mut retained = Vec::with_capacity(limit.min(POLICY_FILE_BYTES_MAX as usize));
        let mut chunk = [0_u8; 8_192];
        let mut truncated = false;
        loop {
            let count = reader.read(&mut chunk)?;
            if count == 0 {
                break;
            }
            let available = limit.saturating_sub(retained.len());
            let retained_count = count.min(available);
            retained.extend_from_slice(&chunk[..retained_count]);
            if retained_count < count {
                truncated = true;
            }
        }
        Ok(BoundedCapture {
            bytes: retained,
            truncated,
        })
    })
}

fn join_capture(
    handle: thread::JoinHandle<Result<BoundedCapture, io::Error>>,
    stream: &str,
) -> Result<BoundedCapture, RunError> {
    handle
        .join()
        .map_err(|_| RunError::Internal(format!("evaluation worker {stream} reader panicked")))?
        .map_err(|error| RunError::Internal(format!("reading evaluation worker {stream}: {error}")))
}

fn observe_only_metrics(
    request: &EvaluatorWorkerRequest,
    evaluator: &EvaluatorObservation,
    wall_time_ms: u64,
) -> Vec<MetricFact> {
    let mut metrics = common_request_metrics(request);
    append_evaluator_metrics(&mut metrics, Some(evaluator));
    metrics.push(metric_fact(
        "wall_time",
        "milliseconds",
        "observation",
        MetricFactStatus::Observed,
        Some(wall_time_ms),
        "parent-monotonic-clock",
        None,
    ));
    metrics.push(unavailable_metric(
        "cpu_time",
        "milliseconds",
        "in-process-path-has-no-operation-scoped-cpu-observation",
    ));
    metrics.push(unavailable_metric("peak_rss", "bytes", "in-process-path-has-no-operation-scoped-peak-rss"));
    metrics
}

fn enforced_metrics(
    request: &EvaluatorWorkerRequest,
    response: Option<&EvaluatorWorkerResponse>,
    wall_time_ms: u64,
) -> Vec<MetricFact> {
    let mut metrics = common_request_metrics(request);
    metrics.push(metric_fact(
        "wall_time",
        "milliseconds",
        "observation-and-deadline",
        MetricFactStatus::Enforced,
        Some(wall_time_ms),
        "owned-worker-parent-deadline",
        None,
    ));
    metrics.push(metric_fact(
        "cpu_time_limit",
        "milliseconds",
        "enforcement-policy",
        MetricFactStatus::Enforced,
        Some(request.policy.cpu_time_ms_max),
        "rlimit-cpu",
        None,
    ));
    metrics.push(metric_fact(
        "import_filesystem_confinement",
        "boolean",
        "enforcement-policy",
        MetricFactStatus::Enforced,
        Some(1),
        "landlock-read-file-and-read-dir",
        None,
    ));
    metrics.push(metric_fact(
        "address_space_limit",
        "bytes",
        "enforcement-policy",
        MetricFactStatus::Enforced,
        Some(request.policy.peak_rss_bytes_max),
        "rlimit-as-not-rss",
        None,
    ));
    append_evaluator_metrics(&mut metrics, response.map(|value| &value.evaluator));
    match response {
        Some(response) => {
            metrics.push(optional_observed_metric(
                "cpu_time",
                "milliseconds",
                response.process.cpu_time_ms,
                "worker-getrusage-self",
                "worker-did-not-report-cpu-time",
            ));
            metrics.push(optional_observed_metric(
                "peak_rss",
                "bytes",
                response.process.peak_rss_bytes,
                "worker-getrusage-self-maxrss",
                "worker-did-not-report-peak-rss",
            ));
        }
        None => {
            metrics.push(unavailable_metric("cpu_time", "milliseconds", "worker-ended-without-a-valid-response"));
            metrics.push(unavailable_metric("peak_rss", "bytes", "worker-ended-without-a-valid-response"));
        }
    }
    metrics
}

fn append_evaluator_metrics(metrics: &mut Vec<MetricFact>, evaluator: Option<&EvaluatorObservation>) {
    let imported_modules = evaluator.and_then(|value| value.imported_module_count);
    let discovered_roots = evaluator.and_then(|value| value.discovered_root_count);
    let selected_roots = evaluator.map(|value| u64::from(value.selected_root_count));
    let diagnostics = evaluator.map(|value| u64::from(value.diagnostic_count));
    metrics.push(optional_observed_metric(
        "imported_modules",
        "count",
        imported_modules.map(u64::from),
        "linked-nickel-evaluator",
        "nickel-does-not-expose-imported-module-count",
    ));
    metrics.push(optional_observed_metric(
        "discovered_roots",
        "count",
        discovered_roots.map(u64::from),
        "evaluation-session-root-discovery",
        "whole-value-evaluation-does-not-run-root-discovery",
    ));
    metrics.push(optional_observed_metric(
        "evaluated_selected_roots",
        "count",
        selected_roots,
        "evaluation-session",
        "worker-ended-without-evaluator-observations",
    ));
    metrics.push(optional_observed_metric(
        "diagnostics",
        "count",
        diagnostics,
        "bounded-worker-diagnostics",
        "worker-ended-without-evaluator-observations",
    ));
}

fn common_request_metrics(request: &EvaluatorWorkerRequest) -> Vec<MetricFact> {
    let source_bytes = u64::try_from(request.source_bytes.len()).unwrap_or(u64::MAX);
    let import_roots = u64::try_from(request.imports.len()).unwrap_or(u64::MAX);
    let selected_roots = u64::try_from(request.selected_roots.len()).unwrap_or(u64::MAX);
    vec![
        metric_fact(
            "source_bytes",
            "bytes",
            "admitted-input",
            MetricFactStatus::Observed,
            Some(source_bytes),
            "request-envelope",
            None,
        ),
        metric_fact(
            "import_roots",
            "count",
            "admitted-input",
            MetricFactStatus::Observed,
            Some(import_roots),
            "request-envelope",
            None,
        ),
        metric_fact(
            "admitted_import_entries",
            "count",
            "admitted-input-upper-bound-not-imported-module-count",
            MetricFactStatus::Observed,
            Some(u64::from(request.import_entry_count)),
            "bounded-import-root-walk",
            None,
        ),
        metric_fact(
            "selected_roots",
            "count",
            "public-api-request",
            MetricFactStatus::Observed,
            Some(selected_roots),
            "request-envelope",
            None,
        ),
        metric_fact(
            "explicit_top_level_root_force_count",
            "count",
            "public-api-request-not-nickel-thunk-count",
            MetricFactStatus::Observed,
            Some(selected_roots),
            "request-envelope",
            None,
        ),
        unavailable_metric("actual_nonselected_evaluation_count", "count", "nickel-does-not-expose-this-observation"),
    ]
}

fn optional_observed_metric(
    name: &str,
    unit: &str,
    value: Option<u64>,
    mechanism: &str,
    unavailable_reason: &str,
) -> MetricFact {
    match value {
        Some(value) => metric_fact(name, unit, "observation", MetricFactStatus::Observed, Some(value), mechanism, None),
        None => unavailable_metric(name, unit, unavailable_reason),
    }
}

fn unavailable_metric(name: &str, unit: &str, reason: &str) -> MetricFact {
    metric_fact(name, unit, "observation", MetricFactStatus::Unavailable, None, "unavailable", Some(reason))
}

fn completed_teardown(response_present: bool) -> TeardownFacts {
    TeardownFacts {
        cancellation_requested: false,
        deadline_exceeded: false,
        terminate_sent: false,
        kill_sent: false,
        reaped: true,
        response_present,
        stderr_bytes: 0,
        stderr_truncated: false,
    }
}

fn redact_worker_stderr(bytes: &[u8], request: &EvaluatorWorkerRequest) -> String {
    let mut rendered = String::from_utf8_lossy(bytes).into_owned();
    for import in &request.imports {
        rendered = rendered.replace(&import.canonical_path, "<redacted-import-root>");
    }
    rendered
}

fn write_report(path: &Path, report: &EvaluationBudgetReport) -> Result<(), RunError> {
    let bytes = serde_json::to_vec_pretty(report)
        .map_err(|error| RunError::Internal(format!("encoding evaluation budget report: {error}")))?;
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .map_err(|error| RunError::Internal(format!("creating evaluation report directory: {error}")))?;
    let file_name = path.file_name().and_then(|name| name.to_str()).unwrap_or("evaluation-report.json");
    let temporary = parent.join(format!(".{file_name}{REPORT_TEMP_SUFFIX}"));
    fs::write(&temporary, bytes)
        .map_err(|error| RunError::Internal(format!("writing staged evaluation report: {error}")))?;
    fs::rename(&temporary, path)
        .map_err(|error| RunError::Internal(format!("publishing evaluation report: {error}")))?;
    Ok(())
}

#[cfg(debug_assertions)]
fn apply_test_reap_failure(wait: &mut WaitResult) {
    if std::env::var_os(TEST_FORCE_REAP_FAILURE_ENV).is_some() {
        wait.reaped = false;
    }
}

#[cfg(not(debug_assertions))]
fn apply_test_reap_failure(_wait: &mut WaitResult) {}

#[cfg(debug_assertions)]
fn schedule_test_cancellation(policy: &EvaluationBudgetPolicy, cancellation: Arc<AtomicBool>) -> Result<(), RunError> {
    let Some(value) = std::env::var_os(TEST_CANCEL_AFTER_MS_ENV) else {
        return Ok(());
    };
    let value = value.to_str().ok_or_else(|| RunError::Eval("invalid evaluator cancellation fixture".to_string()))?;
    let delay_ms: u64 =
        value.parse().map_err(|_| RunError::Eval("invalid evaluator cancellation fixture".to_string()))?;
    if delay_ms == 0 || delay_ms >= policy.wall_time_ms_max {
        return Err(RunError::Eval("invalid evaluator cancellation fixture".to_string()));
    }
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(delay_ms));
        cancellation.store(true, Ordering::Release);
    });
    Ok(())
}

#[cfg(not(debug_assertions))]
fn schedule_test_cancellation(
    _policy: &EvaluationBudgetPolicy,
    _cancellation: Arc<AtomicBool>,
) -> Result<(), RunError> {
    Ok(())
}

fn elapsed_millis(started: Instant) -> Result<u64, RunError> {
    u64::try_from(started.elapsed().as_millis())
        .map_err(|_| RunError::Internal("evaluation wall time overflow".to_string()))
}

fn budget_error(error: crunch_eval_budget_core::BudgetError) -> RunError {
    RunError::Eval(error.to_string())
}

fn protocol_error(error: crunch_eval_budget_core::BudgetError) -> RunError {
    RunError::Internal(error.to_string())
}

pub fn terminal_failure(result: &BudgetedEvaluation) -> Option<RunError> {
    match result.report.terminal_disposition {
        TerminalDisposition::Success => None,
        TerminalDisposition::EvaluationError => Some(RunError::Eval(
            result
                .report
                .error_class
                .clone()
                .unwrap_or_else(|| "evaluation-budget-evaluation-error".to_string()),
        )),
        disposition => Some(RunError::Eval(format!("evaluation-budget-terminal:{disposition:?}"))),
    }
}
