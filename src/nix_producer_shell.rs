// r[impl nix_producer_adapter.bounded_execution]
// r[impl nix_producer_adapter.host_nix_backend]
// r[impl nix_producer_adapter.evaluation_boundary]
//
// Imperative shell for the `nix-producer-v1` contract. This module owns
// process launch, deadlines, resource limits, bounded output capture, `.drv`
// closure collection, and binary identity measurement. All admission and
// classification decisions delegate to the pure core in `nix_producer`.
//
// Evaluation boundary: the only backend commands this shell can launch are
// `fix instantiate` and `nix-instantiate`. No realization command (`build`,
// `run`, `switch`, `nix-build`, `nix-store --realise`) is representable here.
// Instantiation may write `.drv` files through a daemon; the shell copies the
// concrete closure into an owned directory and never queries or builds
// through the daemon.

use std::collections::BTreeSet;
use std::io::Read;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::ExitStatus;
use std::process::Stdio;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use crate::nix_producer::BackendRegistration;
use crate::nix_producer::BackendTrustPosture;
use crate::nix_producer::NIX_PRODUCER_CONTRACT_SCHEMA;
use crate::nix_producer::NixProducerRequest;
use crate::nix_producer::ProducerBackendKind;
use crate::nix_producer::ProducerBudget;
use crate::nix_producer::ProducerError;
use crate::nix_producer::ProducerErrorClass;
use crate::nix_producer::ProducerIdentityFacts;
use crate::nix_producer::ProducerOutcome;
use crate::nix_producer::ProducerSuccess;
use crate::nix_producer::ProducerTarget;
use crate::nix_producer::accept_success;
use crate::nix_producer::eval_args_digest;
use crate::nix_producer::select_backend;
use crate::nix_producer::validate_request;

const MAX_STDOUT_BYTES: u64 = 1_048_576;
const MAX_STDERR_BYTES: u64 = 1_048_576;
const MAX_STDERR_DETAIL_CHARS: usize = 512;
const MAX_SINGLE_DRV_BYTES: u64 = 67_108_864;
const WAIT_POLL: Duration = Duration::from_millis(10);
const SPAWN_ETXTBSY_RETRIES: u32 = 3;
const SPAWN_RETRY_BACKOFF: Duration = Duration::from_millis(20);
const ETXTBSY_RAW_OS_ERROR: i32 = 26;
const EXPR_FILE_NAME: &str = "producer-expr.nix";
const DRV_CLOSURE_DIR_NAME: &str = "drv-closure";
const DRV_EXTENSION: &str = ".drv";
const NIX_STORE_PREFIX: &str = "/nix/store";

/// How one backend binary is invoked and identified.
pub(crate) struct BackendRunConfig {
    pub(crate) binary_path: PathBuf,
    /// Operator-supplied version fact, recorded on identity (for example
    /// `0.3.0` for the pinned fix backend or `nix --version` output).
    pub(crate) version_label: String,
    /// Host filesystem root used to read backend-written store paths.
    /// Production runs use `/`; tests use an isolated fixture root.
    pub(crate) store_read_root: PathBuf,
    /// Owned scratch directory. Must be empty or absent before the run.
    pub(crate) work_dir: PathBuf,
}

struct BoundedProcessOutput {
    status: ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

fn shell_error(class: ProducerErrorClass, detail: impl Into<String>) -> ProducerError {
    let detail = detail.into();
    debug_assert!(!detail.is_empty(), "shell error detail must not be empty");
    debug_assert!(detail.chars().count() <= MAX_STDERR_DETAIL_CHARS, "shell error detail stays bounded");
    // ProducerError::new is private to the core; construct through the public
    // classification surface instead.
    ProducerError { class, detail }
}

fn backend_argv(request: &NixProducerRequest, expr_file: &Path, binary: &Path) -> Vec<String> {
    debug_assert!(binary.is_absolute(), "backend binary path must be absolute");
    let mut argv: Vec<String> = Vec::new();
    argv.push(binary.display().to_string());
    match request.backend {
        ProducerBackendKind::Fix => argv.push("instantiate".to_string()),
        ProducerBackendKind::HostNix => {}
    }
    argv.push(expr_file.display().to_string());
    let attribute = match &request.target {
        ProducerTarget::File { attribute, .. } => attribute.clone(),
        ProducerTarget::Expr { attribute, .. } => attribute.clone(),
    };
    if let Some(attribute) = attribute {
        argv.push("-A".to_string());
        argv.push(attribute);
    }
    for (key, value) in &request.eval_args {
        argv.push("--argstr".to_string());
        argv.push(key.clone());
        argv.push(value.clone());
    }
    argv
}

fn read_capped(reader: impl Read + Send + 'static, cap: u64) -> thread::JoinHandle<Vec<u8>> {
    debug_assert!(cap > 0, "output cap must be positive");
    thread::spawn(move || {
        let mut bounded = reader.take(cap.saturating_add(1));
        let mut buffer = Vec::new();
        let _ = bounded.read_to_end(&mut buffer);
        buffer
    })
}

fn execute_bounded(argv: &[String], budget: &ProducerBudget) -> Result<BoundedProcessOutput, ProducerError> {
    debug_assert!(!argv.is_empty(), "backend argv must not be empty");
    debug_assert!(budget.wall_time_ms_max > 0, "deadline must be positive");
    let mut command = Command::new(&argv[0]);
    command.args(&argv[1..]).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let memory_limit = budget.memory_bytes_max;
    if memory_limit > 0 {
        unsafe {
            command.pre_exec(move || {
                let limit = libc::rlimit {
                    rlim_cur: memory_limit,
                    rlim_max: memory_limit,
                };
                if libc::setrlimit(libc::RLIMIT_AS, &limit) == 0 {
                    Ok(())
                } else {
                    Err(std::io::Error::last_os_error())
                }
            });
        }
    }
    let mut spawn_attempt: u32 = 0;
    let mut child = loop {
        match command.spawn() {
            Ok(child) => break child,
            Err(error)
                if error.raw_os_error() == Some(ETXTBSY_RAW_OS_ERROR) && spawn_attempt < SPAWN_ETXTBSY_RETRIES =>
            {
                spawn_attempt += 1;
                thread::sleep(SPAWN_RETRY_BACKOFF);
            }
            Err(error) => {
                return Err(shell_error(
                    ProducerErrorClass::BackendUnavailable,
                    format!("failed to spawn backend: {error}"),
                ));
            }
        }
    };
    let stdout_handle = read_capped(child.stdout.take().expect("stdout piped"), MAX_STDOUT_BYTES);
    let stderr_handle = read_capped(child.stderr.take().expect("stderr piped"), MAX_STDERR_BYTES);
    let deadline = Instant::now() + Duration::from_millis(budget.wall_time_ms_max);
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(shell_error(
                        ProducerErrorClass::BudgetTimeout,
                        format!("backend exceeded wall-time budget of {} ms", budget.wall_time_ms_max),
                    ));
                }
                thread::sleep(WAIT_POLL);
            }
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(shell_error(
                    ProducerErrorClass::BackendUnavailable,
                    format!("failed to wait on backend: {error}"),
                ));
            }
        }
    };
    let stdout = stdout_handle.join().unwrap_or_default();
    let stderr = stderr_handle.join().unwrap_or_default();
    if stdout.len() as u64 > MAX_STDOUT_BYTES {
        return Err(shell_error(
            ProducerErrorClass::OutputTooLarge,
            format!("backend stdout exceeded {MAX_STDOUT_BYTES} bytes"),
        ));
    }
    Ok(BoundedProcessOutput { status, stdout, stderr })
}

fn stderr_detail(stderr: &[u8]) -> String {
    let text = String::from_utf8_lossy(stderr);
    let trimmed = text.trim();
    let bounded: String = trimmed.chars().take(MAX_STDERR_DETAIL_CHARS).collect();
    if bounded.is_empty() {
        "backend exited non-zero with empty stderr".to_string()
    } else {
        bounded
    }
}

fn parse_root_drv_path(stdout: &[u8]) -> Result<String, ProducerError> {
    let text = String::from_utf8_lossy(stdout);
    let path = text
        .lines()
        .rev()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .ok_or_else(|| shell_error(ProducerErrorClass::MalformedOutput, "backend printed no output path"))?;
    if path.chars().any(char::is_whitespace) {
        return Err(shell_error(ProducerErrorClass::MalformedOutput, "backend output path contains whitespace"));
    }
    if !path.starts_with(NIX_STORE_PREFIX) || !path.ends_with(DRV_EXTENSION) {
        return Err(shell_error(
            ProducerErrorClass::MalformedOutput,
            format!("backend output path is not a {NIX_STORE_PREFIX} .drv path"),
        ));
    }
    Ok(path.to_string())
}

fn physical_drv_path(store_read_root: &Path, logical_path: &str) -> PathBuf {
    debug_assert!(logical_path.starts_with('/'), "logical drv path must be absolute");
    store_read_root.join(logical_path.trim_start_matches('/'))
}

fn copy_drv_closure(
    root_logical_path: &str,
    store_read_root: &Path,
    dest_dir: &Path,
    budget: &ProducerBudget,
) -> Result<(u32, u64), ProducerError> {
    debug_assert!(!root_logical_path.is_empty(), "root drv path must not be empty");
    debug_assert!(dest_dir.is_absolute(), "closure destination must be absolute");
    std::fs::create_dir_all(dest_dir).map_err(|error| {
        shell_error(ProducerErrorClass::BackendUnavailable, format!("failed to create closure dir: {error}"))
    })?;
    let mut visited: BTreeSet<String> = BTreeSet::new();
    let mut stack: Vec<String> = vec![root_logical_path.to_string()];
    let mut total_bytes: u64 = 0;
    while let Some(logical_path) = stack.pop() {
        if !visited.insert(logical_path.clone()) {
            continue;
        }
        if visited.len() as u32 > budget.drv_file_count_max {
            return Err(shell_error(
                ProducerErrorClass::OutputTooLarge,
                format!("drv closure exceeds {} files", budget.drv_file_count_max),
            ));
        }
        let physical = physical_drv_path(store_read_root, &logical_path);
        let bytes = std::fs::read(&physical).map_err(|error| {
            shell_error(
                ProducerErrorClass::MalformedOutput,
                format!("cannot read backend-written drv {logical_path}: {error}"),
            )
        })?;
        if bytes.len() as u64 > MAX_SINGLE_DRV_BYTES {
            return Err(shell_error(
                ProducerErrorClass::OutputTooLarge,
                format!("drv {logical_path} exceeds {MAX_SINGLE_DRV_BYTES} bytes"),
            ));
        }
        total_bytes = total_bytes.saturating_add(bytes.len() as u64);
        if total_bytes > budget.output_bytes_max {
            return Err(shell_error(
                ProducerErrorClass::OutputTooLarge,
                format!("drv closure exceeds {} bytes", budget.output_bytes_max),
            ));
        }
        let derivation = nix_compat::derivation::Derivation::from_aterm_bytes(&bytes).map_err(|_| {
            shell_error(
                ProducerErrorClass::MalformedOutput,
                format!("backend-written drv failed ATerm parsing: {logical_path}"),
            )
        })?;
        for input_drv in derivation.input_derivations.keys() {
            stack.push(input_drv.to_absolute_path());
        }
        let basename = logical_path.rsplit('/').next().unwrap_or_default();
        debug_assert!(!basename.is_empty(), "drv basename must not be empty");
        std::fs::write(dest_dir.join(basename), &bytes).map_err(|error| {
            shell_error(ProducerErrorClass::BackendUnavailable, format!("failed to copy drv closure member: {error}"))
        })?;
    }
    debug_assert!(!visited.is_empty(), "closure must contain the root");
    let count = u32::try_from(visited.len())
        .map_err(|_| shell_error(ProducerErrorClass::OutputTooLarge, "drv closure file count overflowed u32"))?;
    Ok((count, total_bytes))
}

fn measure_binary_identity(kind: ProducerBackendKind, binary_path: &Path) -> Result<String, ProducerError> {
    match kind {
        ProducerBackendKind::Fix => {
            let bytes = std::fs::read(binary_path).map_err(|error| {
                shell_error(ProducerErrorClass::BackendUnavailable, format!("cannot read fix backend binary: {error}"))
            })?;
            debug_assert!(!bytes.is_empty(), "backend binary must not be empty");
            Ok(blake3::hash(&bytes).to_hex().to_string())
        }
        ProducerBackendKind::HostNix => Ok(binary_path.display().to_string()),
    }
}

/// Systems this change's backends may serve. Other platforms fail closed at
/// selection; the pinned fix toolchain is x86_64-Linux-only and the host-nix
/// backend is validated only on the same system class.
fn backend_supported_systems() -> BTreeSet<String> {
    BTreeSet::from(["x86_64-linux".to_string()])
}

/// Run one producer backend under the bounded policy and return a
/// contract-accepted success observation.
pub(crate) fn run_backend(
    request: &NixProducerRequest,
    config: &BackendRunConfig,
) -> Result<ProducerSuccess, ProducerError> {
    validate_request(request)?;
    debug_assert!(
        matches!(request.backend, ProducerBackendKind::Fix | ProducerBackendKind::HostNix),
        "only registered backend kinds reach the shell"
    );
    let binary_identity = measure_binary_identity(request.backend, &config.binary_path)?;
    let registration = BackendRegistration {
        kind: request.backend,
        binary_identity: binary_identity.clone(),
        supported_systems: backend_supported_systems(),
    };
    let selected = select_backend(&[registration], request.backend, &request.system)?;
    debug_assert_eq!(selected.binary_identity, binary_identity, "selection must return the measured backend identity");
    let expr_file = match &request.target {
        ProducerTarget::File { path, .. } => PathBuf::from(path),
        ProducerTarget::Expr { text, .. } => {
            std::fs::create_dir_all(&config.work_dir).map_err(|error| {
                shell_error(ProducerErrorClass::BackendUnavailable, format!("failed to create work dir: {error}"))
            })?;
            let expr_file = config.work_dir.join(EXPR_FILE_NAME);
            std::fs::write(&expr_file, text).map_err(|error| {
                shell_error(ProducerErrorClass::BackendUnavailable, format!("failed to write expression file: {error}"))
            })?;
            expr_file
        }
    };
    let argv = backend_argv(request, &expr_file, &config.binary_path);
    let output = execute_bounded(&argv, &request.budget)?;
    if !output.status.success() {
        return Err(shell_error(
            ProducerErrorClass::EvaluationFailed,
            format!("backend exited with {}: {}", output.status, stderr_detail(&output.stderr)),
        ));
    }
    let root_drv_path = parse_root_drv_path(&output.stdout)?;
    let closure_dir = config.work_dir.join(DRV_CLOSURE_DIR_NAME);
    let (drv_file_count, drv_dir_total_bytes) =
        copy_drv_closure(&root_drv_path, &config.store_read_root, &closure_dir, &request.budget)?;
    let identity = ProducerIdentityFacts {
        backend: request.backend,
        backend_version: config.version_label.clone(),
        binary_identity,
        trust_posture: match request.backend {
            ProducerBackendKind::Fix => BackendTrustPosture::PinnedMantleBuilt,
            ProducerBackendKind::HostNix => BackendTrustPosture::AmbientHost,
        },
        eval_args_digest_blake3: eval_args_digest(&request.eval_args, &request.system),
    };
    let success = ProducerSuccess {
        schema: NIX_PRODUCER_CONTRACT_SCHEMA.to_string(),
        drv_dir: closure_dir.display().to_string(),
        root_drv_path,
        drv_file_count,
        drv_dir_total_bytes,
        identity,
    };
    match accept_success(&success, request)? {
        ProducerOutcome::Success(accepted) => Ok(accepted),
        ProducerOutcome::Failure(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::os::unix::fs::PermissionsExt;

    use super::*;
    use crate::nix_producer::ProducerBudget;

    // r[verify nix_producer_adapter.bounded_execution]
    // r[verify nix_producer_adapter.host_nix_backend]

    const ROOT_DRV_NAME: &str = "ch49594n9avinrf8ip0aslidkc4lxkqv-foo.drv";
    const CHILD_DRV_NAME: &str = "ss2p4wmxijn652haqyd7dckxwl4c7hxx-bar.drv";
    const ROOT_DRV_ATERM: &str = "Derive([(\"out\",\"/nix/store/fhaj6gmwns62s6ypkcldbaj2ybvkhx3p-foo\",\"\",\"\")],[(\"/nix/store/ss2p4wmxijn652haqyd7dckxwl4c7hxx-bar.drv\",[\"out\"])],[],\":\",\":\",[],[(\"bar\",\"/nix/store/mp57d33657rf34lzvlbpfa1gjfv5gmpg-bar\"),(\"builder\",\":\"),(\"name\",\"foo\"),(\"out\",\"/nix/store/fhaj6gmwns62s6ypkcldbaj2ybvkhx3p-foo\"),(\"system\",\":\")])";
    const CHILD_DRV_ATERM: &str = "Derive([(\"out\",\"/nix/store/mp57d33657rf34lzvlbpfa1gjfv5gmpg-bar\",\"r:sha1\",\"0beec7b5ea3f0fdbc95d0dd47f3c5bc275da8a33\")],[],[],\":\",\":\",[],[(\"builder\",\":\"),(\"name\",\"bar\"),(\"out\",\"/nix/store/mp57d33657rf34lzvlbpfa1gjfv5gmpg-bar\"),(\"outputHash\",\"0beec7b5ea3f0fdbc95d0dd47f3c5bc275da8a33\"),(\"outputHashAlgo\",\"sha1\"),(\"outputHashMode\",\"recursive\"),(\"system\",\":\")])";

    fn fixture_store_root(root: &Path) {
        let store = root.join("nix/store");
        std::fs::create_dir_all(&store).unwrap();
        std::fs::write(store.join(ROOT_DRV_NAME), ROOT_DRV_ATERM).unwrap();
        std::fs::write(store.join(CHILD_DRV_NAME), CHILD_DRV_ATERM).unwrap();
    }

    fn fake_backend(dir: &Path, body: &str) -> PathBuf {
        let script = dir.join("fake-backend.sh");
        {
            let mut file = std::fs::File::create(&script).unwrap();
            use std::io::Write;
            file.write_all(body.as_bytes()).unwrap();
            file.sync_all().unwrap();
        }
        let mut permissions = std::fs::metadata(&script).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&script, permissions).unwrap();
        script
    }

    fn request() -> NixProducerRequest {
        NixProducerRequest {
            schema: NIX_PRODUCER_CONTRACT_SCHEMA.to_string(),
            backend: ProducerBackendKind::Fix,
            target: ProducerTarget::Expr {
                text: "derivation { name = \"foo\"; }".to_string(),
                attribute: None,
            },
            eval_args: BTreeMap::new(),
            system: "x86_64-linux".to_string(),
            budget: ProducerBudget::default(),
        }
    }

    fn config(binary: &Path, read_root: &Path, work: &Path) -> BackendRunConfig {
        BackendRunConfig {
            binary_path: binary.to_path_buf(),
            version_label: "test-fixture".to_string(),
            store_read_root: read_root.to_path_buf(),
            work_dir: work.to_path_buf(),
        }
    }

    #[test]
    fn backend_success_collects_two_node_closure() {
        let temp = tempfile::tempdir().unwrap();
        let read_root = temp.path().join("host");
        fixture_store_root(&read_root);
        let root_logical = format!("/nix/store/{ROOT_DRV_NAME}");
        let binary = fake_backend(temp.path(), &format!("#!/bin/sh\nprintf '%s\\n' '{root_logical}'\n"));
        let work = temp.path().join("work");
        let success = run_backend(&request(), &config(&binary, &read_root, &work)).unwrap();
        assert_eq!(success.root_drv_path, root_logical);
        assert_eq!(success.drv_file_count, 2);
        let closure_dir = work.join(DRV_CLOSURE_DIR_NAME);
        assert!(closure_dir.join(ROOT_DRV_NAME).is_file());
        assert!(closure_dir.join(CHILD_DRV_NAME).is_file());
        assert_eq!(success.identity.trust_posture, BackendTrustPosture::PinnedMantleBuilt);
        assert_eq!(success.identity.binary_identity.len(), 64);
    }

    #[test]
    fn host_nix_backend_records_ambient_posture() {
        let temp = tempfile::tempdir().unwrap();
        let read_root = temp.path().join("host");
        fixture_store_root(&read_root);
        let root_logical = format!("/nix/store/{ROOT_DRV_NAME}");
        let binary = fake_backend(temp.path(), &format!("#!/bin/sh\nprintf '%s\\n' '{root_logical}'\n"));
        let mut host_request = request();
        host_request.backend = ProducerBackendKind::HostNix;
        let work = temp.path().join("work");
        let success = run_backend(&host_request, &config(&binary, &read_root, &work)).unwrap();
        assert_eq!(success.identity.trust_posture, BackendTrustPosture::AmbientHost);
        assert_eq!(success.identity.binary_identity, binary.display().to_string());
    }

    #[test]
    fn backend_exit_failure_maps_to_evaluation_failed() {
        let temp = tempfile::tempdir().unwrap();
        let binary = fake_backend(temp.path(), "#!/bin/sh\necho 'eval error' >&2\nexit 1\n");
        let work = temp.path().join("work");
        let error = run_backend(&request(), &config(&binary, temp.path(), &work)).unwrap_err();
        assert_eq!(error.class, ProducerErrorClass::EvaluationFailed);
        assert!(error.detail.contains("eval error"));
    }

    #[test]
    fn backend_garbage_output_maps_to_malformed() {
        let temp = tempfile::tempdir().unwrap();
        let binary = fake_backend(temp.path(), "#!/bin/sh\necho 'not a store path'\n");
        let work = temp.path().join("work");
        let error = run_backend(&request(), &config(&binary, temp.path(), &work)).unwrap_err();
        assert_eq!(error.class, ProducerErrorClass::MalformedOutput);
    }

    #[test]
    fn backend_timeout_maps_to_budget_timeout() {
        let temp = tempfile::tempdir().unwrap();
        let binary = fake_backend(temp.path(), "#!/bin/sh\nsleep 5\n");
        let mut slow_request = request();
        slow_request.budget.wall_time_ms_max = 100;
        let work = temp.path().join("work");
        let error = run_backend(&slow_request, &config(&binary, temp.path(), &work)).unwrap_err();
        assert_eq!(error.class, ProducerErrorClass::BudgetTimeout);
    }

    #[test]
    fn missing_closure_member_maps_to_malformed() {
        let temp = tempfile::tempdir().unwrap();
        let read_root = temp.path().join("host");
        let store = read_root.join("nix/store");
        std::fs::create_dir_all(&store).unwrap();
        std::fs::write(store.join(ROOT_DRV_NAME), ROOT_DRV_ATERM).unwrap();
        let root_logical = format!("/nix/store/{ROOT_DRV_NAME}");
        let binary = fake_backend(temp.path(), &format!("#!/bin/sh\nprintf '%s\\n' '{root_logical}'\n"));
        let work = temp.path().join("work");
        let error = run_backend(&request(), &config(&binary, &read_root, &work)).unwrap_err();
        assert_eq!(error.class, ProducerErrorClass::MalformedOutput);
    }

    #[test]
    fn closure_count_budget_maps_to_output_too_large() {
        let temp = tempfile::tempdir().unwrap();
        let read_root = temp.path().join("host");
        fixture_store_root(&read_root);
        let root_logical = format!("/nix/store/{ROOT_DRV_NAME}");
        let binary = fake_backend(temp.path(), &format!("#!/bin/sh\nprintf '%s\\n' '{root_logical}'\n"));
        let mut tight_request = request();
        tight_request.budget.drv_file_count_max = 1;
        let work = temp.path().join("work");
        let error = run_backend(&tight_request, &config(&binary, &read_root, &work)).unwrap_err();
        assert_eq!(error.class, ProducerErrorClass::OutputTooLarge);
    }

    #[test]
    fn unspawnable_backend_maps_to_backend_unavailable() {
        let temp = tempfile::tempdir().unwrap();
        let missing = temp.path().join("no-such-binary");
        let work = temp.path().join("work");
        let error = run_backend(&request(), &config(&missing, temp.path(), &work)).unwrap_err();
        assert_eq!(error.class, ProducerErrorClass::BackendUnavailable);
    }
}
